/**
 * **Nhịp ghi vị trí làm việc, và cửa chặn khi flush chưa sạch** — Story 5.7 (AC4/AC6/AC9).
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * HAI HÀNG CỦA §I/O MATRIX MÀ KHÔNG ĐƯỜNG NGHIỆM THU NÀO KHÁC CHẠM TỚI
 * ═════════════════════════════════════════════════════════════════════════════════
 * Bộ ca Rust của story này (`project_contract.rs`/`segment_contract.rs`) phủ trọn nửa **kho**:
 * vị trí ghi rồi đọc lại đúng, vị trí trỏ vào segment về hưu rơi về câu đầu, Chương rỗng cho
 * `None`. Bàn đo e2e phủ đường **đầu-tới-cuối** trong WKWebView thật. Hai hàng dưới đây thì
 * không thuộc cả hai:
 *
 * - **"Ghi vị trí"** — hàng nói *"một lượt `save_chapter_position` sau nhịp idle 500 ms /
 *   trần cứng 5 s"*. Đó là một mệnh đề về **NHỊP**, và nó chỉ đo được ở tầng hàm thuần:
 *   `createPositionFlush` nhận mọi thời điểm **qua tham số** (`positionFlush.ts` §⚠️), nên
 *   phép kiểm là **tất định và tức thời** — không `vi.useFakeTimers()`, không `sleep` thật.
 *   ⚠️ Bọc một đồng hồ giả ở đây là đổi một **bảo đảm** đã có lấy một **thói quen**
 *   (`tests/AGENTS.md`).
 * - **"Flush trượt lúc mở Tác phẩm/Chương khác"** — hàng nói *"CHẶN, `OpenWorkState` không
 *   đổi"*. `editorChapterSwitch.test.ts:267` đã canh đúng mệnh đề đó cho `switchChapter`
 *   (đường Chương **kề**, Story 2.11). `openChapterById` là một đường **THỨ HAI** tới cùng
 *   một chỗ nguy hiểm, và nó ra đời ở story này — một bộ xanh của đường thứ nhất **không**
 *   chứng minh đường thứ hai được canh (`AGENTS.md::Known pitfalls`, đã dính năm lần trong
 *   bảy ngày ở Epic 3).
 *
 * ⚠️ **Giới hạn thật, ghi ra thay vì để người sau tưởng đã phủ:** tệp này giả ở **biên IPC**
 * (`src/config/*`), nên nó không nói gì về nửa Rust của `save_chapter_position` — vế đó ở
 * `segment_contract.rs`. Và nó không đo hình học/cuộn: `happy-dom` không phải WebKit.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  failNextSave,
  FIXTURE_CHAPTER_ID,
  FIXTURE_SEGMENTS,
  recordSave,
  resetRecorder,
} from './support/segmentFixture'
import { createPositionFlush, POSITION_HARD_CAP_MS, POSITION_IDLE_MS } from '../../src/panels/positionFlush'

// ═════════════════════════════════════════════════════════════════════════════════
// § Hàng "Ghi vị trí" — nhịp idle + trần cứng, đo trên hàm THUẦN
// ═════════════════════════════════════════════════════════════════════════════════

describe('panels/positionFlush.ts — nhịp ghi vị trí (hàm thuần)', () => {
  it('một lượt rê caret ⇒ hạn ghi đúng `now + POSITION_IDLE_MS`', () => {
    const flush = createPositionFlush()
    const due = flush.markMoved(FIXTURE_CHAPTER_ID, 11, 1_000)

    expect(due).toBe(1_000 + POSITION_IDLE_MS)
    expect(flush.isDirty()).toBe(true)
    expect(flush.pending()).toEqual({ chapterId: FIXTURE_CHAPTER_ID, segmentId: 11 })
  })

  it('🔴 rê caret LIÊN TỤC ⇒ trần cứng THẮNG, hạn ghi KHÔNG trượt vô hạn', () => {
    const flush = createPositionFlush()
    // Lượt đầu ở t=0; sau đó rê đều mỗi 100 ms — dưới ngưỡng idle, nên một debounce THUẦN sẽ
    // không bao giờ tới hạn. Trần cứng là thứ phân biệt hai hành vi đó.
    flush.markMoved(FIXTURE_CHAPTER_ID, 11, 0)
    let due = 0
    for (let t = 100; t <= 6_000; t += 100) {
      due = flush.markMoved(FIXTURE_CHAPTER_ID, 11 + (t % 3), t)
    }

    expect(due).toBe(POSITION_HARD_CAP_MS)
    expect(due).toBeLessThan(6_000)
  })

  it('🔴 ghi xong ⇒ tập chờ SẠCH và chu kỳ bắt đầu lại từ đầu', () => {
    const flush = createPositionFlush()
    flush.markMoved(FIXTURE_CHAPTER_ID, 11, 1_000)
    const written = flush.pending()
    expect(written).not.toBeNull()

    flush.onFlushed(1_600, written!)

    expect(flush.isDirty()).toBe(false)
    expect(flush.pending()).toBeNull()
    expect(flush.deadline()).toBeNull()

    // Chu kỳ MỚI: trần cứng phải tính lại từ `now` của lượt rê kế, không từ lượt đầu tiên.
    flush.markMoved(FIXTURE_CHAPTER_ID, 12, 2_000)
    expect(flush.deadline()).toBe(2_000 + POSITION_IDLE_MS)
  })

  it('🔴 caret rê TIẾP trong lúc lượt ghi đang bay ⇒ lượt rê đó KHÔNG bị nuốt', () => {
    const flush = createPositionFlush()
    flush.markMoved(FIXTURE_CHAPTER_ID, 11, 1_000)
    const dangBay = flush.pending()!

    // Người dùng rê sang câu 13 trong lúc lô của câu 11 còn trên dây.
    flush.markMoved(FIXTURE_CHAPTER_ID, 13, 1_200)
    flush.onFlushed(1_400, dangBay)

    // Nếu `onFlushed` xoá trắng vô điều kiện thì vị trí 13 biến mất im lặng — không một dấu
    // hiệu nào báo, và mở lại Chương sẽ về câu 11.
    expect(flush.pending()).toEqual({ chapterId: FIXTURE_CHAPTER_ID, segmentId: 13 })
    expect(flush.isDirty()).toBe(true)
  })

  it('`reset()` VỨT vị trí đang chờ, không ghi — dùng khi Chương/Tác phẩm bị thay', () => {
    const flush = createPositionFlush()
    flush.markMoved(FIXTURE_CHAPTER_ID, 11, 1_000)

    flush.reset()

    expect(flush.pending()).toBeNull()
    expect(flush.isDirty()).toBe(false)
    expect(flush.deadline()).toBeNull()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// § Hàng "Flush trượt lúc mở Chương khác" — `openChapterById` phải CHẶN
// ═════════════════════════════════════════════════════════════════════════════════

/** Sổ lượt gọi `open_chapter` — mảng SỐNG, ca test đếm **số lượt IPC**, không hiệu ứng cuối. */
const luotMoChuong: number[] = []

/** `caret_segment_id` mà lượt `read_open_chapter_segments` kế tiếp trả về. Mặc định là segment
 * ĐẦU — đúng giá trị Rust trả cho một Chương chưa từng mở (AC5). */
const caretFromRust = { value: FIXTURE_SEGMENTS[0].id }

async function docSegmentGia() {
  return {
    loaded: {
      chapter_id: FIXTURE_CHAPTER_ID,
      segments: FIXTURE_SEGMENTS.map((s) => ({ ...s })),
      caret_segment_id: caretFromRust.value,
    },
    error: null,
  }
}

/** Sổ lượt `save_chapter_position` — mảng SỐNG, ca test đếm **số lượt IPC** và **tham số**. */
const luotGhiViTri: { chapterId: number; segmentId: number }[] = []

async function ghiViTriGia(chapterId: number, segmentId: number) {
  luotGhiViTri.push({ chapterId, segmentId })
  return { error: null }
}

async function moChuongGia(chapterId: number) {
  luotMoChuong.push(chapterId)
  return {
    chapter: { chapter_id: chapterId, source_text: 'Nguyen van.', source_lang: 'en' },
    error: null,
  }
}

async function docNguyenVanGia() {
  return {
    chapter: { chapter_id: FIXTURE_CHAPTER_ID, source_text: 'Nguyen van.', source_lang: 'en' },
    error: null,
  }
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: docSegmentGia,
    saveSegmentTargets: recordSave,
    saveChapterPosition: ghiViTriGia,
  }
})

vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return { ...actual, openChapter: moChuongGia, readOpenChapter: docNguyenVanGia }
})

async function tuoi() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  await state.ensureSegmentsLoaded()
  return state
}

describe('editorPanelState.ts::openChapterById — cửa chặn khi flush chưa sạch (AC9)', () => {
  beforeEach(() => {
    resetRecorder()
    luotMoChuong.length = 0
    luotGhiViTri.length = 0
    caretFromRust.value = FIXTURE_SEGMENTS[0].id
  })

  it('🔴 flush TRƯỢT ⇒ lượt mở Chương bị CHẶN, và `open_chapter` KHÔNG chạy', async () => {
    const state = await tuoi()
    state.noteEditorEdit(11, 'Chu nguoi dung vua go, chua xuong dia.')
    failNextSave.value = true

    const moDuoc = await state.openChapterById(99)

    expect(moDuoc).toBe(false)
    // Mệnh đề thật: **không một lượt IPC nào** dời con trỏ Chương. Một `expect` trên kết quả
    // trả về một mình sẽ xanh kể cả khi `open_chapter` đã chạy rồi mới trả `false`.
    expect(luotMoChuong).toEqual([])
  })

  it('không có gì để lưu ⇒ mở được, và ĐÚNG MỘT lượt `open_chapter`', async () => {
    const state = await tuoi()

    const moDuoc = await state.openChapterById(99)

    expect(moDuoc).toBe(true)
    expect(luotMoChuong).toEqual([99])
  })

  /**
   * 🔴 **Ca này ra đời từ một lượt e2e ĐỎ trong WKWebView thật (2026-08-29), không từ một
   * lượt đọc mã.** `openChapterById` bản đầu chép cửa `editorHasLoaded()` từ `switchChapter`,
   * và cửa ấy đọc `chapterId.value` — giá trị mà **chỉ** `GridPanel.vue::onMounted` đặt. Ở
   * màn hình Library lúc khởi động lạnh, Workspace **chưa mount lần nào**, nên đường chính
   * của cả story (Library → "Mở Chương" → Workspace) **không bao giờ chạy**, và nó hỏng
   * **không một lỗi nào**: hàm chỉ trả `false`, `setMode('workspace')` không chạy, màn hình
   * đứng im.
   *
   * ⚠️ **Mười ba ca cũ của tệp này và `editorChapterSwitch.test.ts` đều gọi
   * `ensureSegmentsLoaded()` ở `tuoi()` trước**, nên cả bộ đi qua SẠCH trên một sản phẩm
   * đang hỏng — đúng lớp lỗi mà `AGENTS.md::Known pitfalls` gọi tên. Ca dưới đây là ca DUY
   * NHẤT **không** nạp trước, và đó là toàn bộ giá trị của nó.
   */
  it('🔴 Editor CHƯA nạp lần nào (khởi động lạnh ở Library) ⇒ vẫn mở được Chương', async () => {
    vi.resetModules()
    const state = await import('../../src/panels/editorPanelState')
    // KHÔNG `ensureSegmentsLoaded()` — đây là trạng thái thật của một phiên vừa mở, khi
    // người dùng còn đang ở Library và Workspace chưa mount.
    expect(state.editorHasLoaded()).toBe(false)

    const moDuoc = await state.openChapterById(99)

    expect(moDuoc).toBe(true)
    expect(luotMoChuong).toEqual([99])
  })

  /**
   * 🔴 **VG2 — nửa ĐỌC của AC4/AC5 ở tầng frontend.** Rust tính `caret_segment_id` và
   * `segment_contract.rs` canh nó **tới biên IPC**; `GridPanel.vue` đặt caret **từ**
   * `editorCaretPlacement`. Giữa hai đầu ấy có đúng một dòng —
   * `caretPlacement.value = loaded?.caret_segment_id ?? null` trong `ensureSegmentsLoaded` —
   * và trước ca này, xoá dòng đó **không làm ca nào đỏ**: ca Rust chỉ thấy giá trị trên dây,
   * còn không ca frontend nào đọc `editorCaretPlacement` sau một lượt nạp Chương. Hệ quả nếu
   * nó hồi quy: caret **luôn** rơi về câu đầu, tức đúng giá trị hồi phòng của AC5 — một lỗi
   * đội lốt một hành vi hợp lệ.
   */
  it('🔴 `caret_segment_id` Rust trả về đi TRỌN tới `editorCaretPlacement`', async () => {
    caretFromRust.value = FIXTURE_SEGMENTS[1].id
    const state = await tuoi()

    expect(state.editorCaretPlacement.value).toBe(FIXTURE_SEGMENTS[1].id)
  })

  /**
   * 🔴 **VG3 — nửa GHI của AC4/AC6.** `setEditorCaret` là định nghĩa DUY NHẤT của *"rời
   * segment"*, và `GridPanel.vue` gọi nó ở mỗi lượt bấm/tiêu điểm câu — tức đây là đường mà
   * một người dùng THẬT đi qua. Bàn đo e2e cố ý **không** đi đường này (nó ghi vị trí bằng
   * `invoke('save_chapter_position')` trần, xem §Giới hạn của spec đó), và các ca
   * `createPositionFlush()` phía trên gọi `markMoved` BẰNG TAY. ⇒ Trước ca này, xoá hai dòng
   * móc nhịp ghi khỏi `setEditorCaret` không làm ca nào đỏ, và `chapter_position` sẽ **không
   * bao giờ được ghi trong dùng thật** — mở lại Chương luôn rơi về câu đầu, im lặng.
   */
  it('🔴 `setEditorCaret` móc vào nhịp ghi vị trí, và `flushChapterPositionNow` gửi đúng cặp', async () => {
    const state = await tuoi()
    luotGhiViTri.length = 0

    state.setEditorCaret(FIXTURE_SEGMENTS[1].id)
    await state.flushChapterPositionNow()

    expect(luotGhiViTri).toEqual([{ chapterId: FIXTURE_CHAPTER_ID, segmentId: FIXTURE_SEGMENTS[1].id }])
  })

  it('🔴 HAI lượt liên tiếp ⇒ ĐÚNG MỘT lượt `open_chapter` (cửa chống hai lượt cùng bay)', async () => {
    const state = await tuoi()

    const [a, b] = await Promise.all([state.openChapterById(99), state.openChapterById(98)])

    expect(luotMoChuong).toHaveLength(1)
    expect([a, b].filter(Boolean)).toHaveLength(1)
  })
})
