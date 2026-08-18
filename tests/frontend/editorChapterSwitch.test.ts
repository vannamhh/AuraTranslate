/**
 * **Chuyển Chương trong Workspace** — Story 2.11 · FR26 · AC1 · AC2 · AC3 · AC4.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 MỆNH ĐỀ TRUNG TÂM CỦA TỆP NÀY LÀ MỘT **THỨ TỰ**, KHÔNG MỘT KẾT QUẢ
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔵 **SỬA 2026-08-18 (code review ba tầng).** Khối này từng biện minh cho thứ tự bằng một
 * mệnh đề **sai**: *"một lô flush đang bay đáp xuống mang `chapter_id` CŨ ⇒ Rust trả
 * `segment.unknown_ids` ⇒ bản dịch biến mất im lặng"*. `save_segment_targets`
 * (`segment.rs:1171-1193`) kiểm và ghi trên **chính `project.db` đang mở** và **không đọc
 * `OpenWork::chapter_id`**; Chương cũ vẫn còn nguyên trong cùng CSDL sau một lượt đổi Chương,
 * nên một lô tới trễ mang `chapter_id` cũ **ghi đúng vào Chương cũ**.
 *
 * ⇒ **Các ca dưới đây không đổi một dòng nào và vẫn đáng giữ** — thứ tự *flush xong → invoke*
 * là mệnh đề đúng, chỉ **lý do** đổi: nó bảo vệ tính nhất quán con trỏ/UI. Đường mất chữ
 * **thật** *(gõ tiếp trong cửa sổ giữa `invoke` và `resetEditorPanel()`)* được đóng ở
 * `editorPanelState.ts::noteEditorEdit` và có ca riêng ở cuối tệp này.
 *
 * ⇒ Không một phép khẳng định nào về **kết quả cuối** bắt được lỗi đó: cả hai thứ tự đều cho
 * ra *"Chương mới đang hiện"*. Vì thế tệp này ghi lại **thứ tự các lượt gọi IPC** vào một sổ
 * chung và khẳng định trên chính sổ đó. Cùng cách mà `libraryImport.ts:119-132` đã phải ghi ra
 * bằng chữ cho lượt đổi **Tác phẩm** — và ở đó lời giải cũng là **thứ tự**, không một `try/catch`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO VITEST CHỨ KHÔNG E2E — và vế nào KHÔNG ở đây
 * ═════════════════════════════════════════════════════════════════════════════════
 * Bốn đường nghiệm thu, bốn vai. Thứ tự flush-rồi-mới-đổi là **hành vi module thuần**;
 * `happy-dom` đủ và nhanh. Ba vế **không** thuộc tệp này:
 * - *"truy vấn Chương kề đúng ở biên và với `ord` thưa"* ⇒ **`cargo test`** hợp đồng
 *   (`project_contract.rs`), vì đó là hợp đồng **dữ liệu**;
 * - *"tiêu điểm không rơi về `body` sau lượt chuyển"* ⇒ **e2e**, `happy-dom` không phải WebKit;
 * - *"hai lệnh không giành hợp âm"* ⇒ cổng tĩnh `check:commands` Kiểm C.
 *
 * ⚠️ Và nhớ giới hạn đã trả giá: Story 2.5 có **74/74 vitest xanh** trên một sản phẩm mà
 * `isConfirmed` **luôn `false`** trong app thật — fixture chép tay mang sẵn một trường mà dây
 * không gửi. Lưới cho nửa Rust nằm ở `project_contract.rs`, **không** ở đây.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import {
  failNextSave,
  FIXTURE_CHAPTER_ID,
  FIXTURE_SEGMENTS,
  recordSave,
  resetRecorder,
  saveCalls,
} from './support/segmentFixture'
import type { ChapterSegment } from '../../src/config/segment'
import type { ChapterDirection, ChapterSwitchOutcome } from '../../src/config/chapter'

/**
 * 🔴 **SỔ THỨ TỰ — trái tim của tệp này.** Mọi lượt đi qua biên IPC đều đẩy một dòng vào đây,
 * nên một phép khẳng định trên mảng này **là** một phép khẳng định về thứ tự.
 *
 * ⚠️ Nó ghi cả `'save'` lẫn `'switch'` vào **cùng một** mảng, có chủ ý: hai sổ riêng đo được
 * *"cả hai đã chạy"* nhưng **không** đo được cái nào chạy trước.
 */
const soThuTu: string[] = []

/** Chương mà lượt `readOpenChapterSegments` kế tiếp trả về — đổi khi lượt chuyển thành công. */
const chuongDangMo = { value: FIXTURE_CHAPTER_ID }

/** Kết cục mà lượt `openAdjacentChapter` kế tiếp trả về. Đặt lại ở mỗi ca. */
const ketQuaChuyen: {
  value: { outcome: ChapterSwitchOutcome; chapterId: number | null }
} = { value: { outcome: 'moved', chapterId: 99 } }

/** Bật để lượt `openAdjacentChapter` kế tiếp trả về một lỗi IPC. */
const loiChuyen = { value: false }

/** Bật để lượt `setSegmentOmitted` kế tiếp bị Rust từ chối — dùng để nạp `omitError`. */
const boCatBoTruot = { value: false }

/** Thân `setSegmentOmitted` giả. */
async function catBoGia(segmentId: number, omitted: boolean) {
  if (boCatBoTruot.value) {
    boCatBoTruot.value = false
    return {
      outcome: null,
      error: {
        code: 'segment.chapter_not_found',
        message_key: 'err.segment.chapter_not_found',
        params: { segment_id: String(segmentId) },
        retryable: false,
      },
    }
  }
  return { outcome: { segment_id: segmentId, is_omitted: omitted }, error: null }
}

/**
 * Thân `readOpenChapterSegments` giả — **khác** `readFixture` dùng chung ở chỗ nó trả về
 * `chapter_id` **theo Chương đang mở**, không một hằng.
 *
 * ⚠️ Cần thế vì mệnh đề của AC1 là *"lưới nay mang Chương MỚI"*, và một fixture trả một
 * `chapter_id` cố định sẽ xanh kể cả khi lượt nạp lại chưa bao giờ chạy.
 */
async function docChuongDangMo(): Promise<{
  loaded: { chapter_id: number; segments: ChapterSegment[] }
  error: null
}> {
  soThuTu.push(`read:${chuongDangMo.value}`)
  return {
    loaded: {
      chapter_id: chuongDangMo.value,
      segments: FIXTURE_SEGMENTS.map((s) => ({ ...s })),
    },
    error: null,
  }
}

/**
 * Thân `saveSegmentTargets` giả — ghi **HAI** mốc: lúc lô rời webview và lúc nó đáp về.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO HAI MỐC — và đây là một khoảng hở ĐO ĐƯỢC trong chính tệp test này
 * ═════════════════════════════════════════════════════════════════════════════════
 * Bản đầu chỉ đẩy **một** mốc `save:` lúc vào, rồi khẳng định `iLuu < iChuyen`. Phép đột
 * biến của Task 6.5 đã **bác** nó: bỏ `await` trước lượt flush *(tức dựng lại đúng cuộc đua
 * mất dữ liệu mà tệp này tồn tại để chặn)* mà ca đó **vẫn xanh** — vì lời gọi flush vẫn
 * **bắt đầu** trước, và mốc "vào" chỉ chứng minh thứ tự **bắt đầu**.
 *
 * Mệnh đề thật không phải *"flush bắt đầu trước"* mà là **"flush ĐÃ XONG trước"** — AD-35:
 * *"một flush chỉ được coi là xong sau khi đã ghi vào WAL"*. Một lô còn đang bay lúc
 * `open_adjacent_chapter` dời con trỏ Chương sẽ đáp xuống mang `chapter_id` **CŨ**, và đó
 * chính là lượt mất chữ. ⇒ Mốc phải đo được là `save:done`.
 *
 * ⚠️ Bài học chung, không riêng ca này: *"một ca chưa bao giờ đỏ là một ca chưa ai biết nó
 * có chạy không"*. Ca này **đã** xanh suốt và vẫn không canh gì — nó chỉ lộ ra dưới một
 * phép đột biến.
 */
async function ghiRoiLuu(
  chapterId: number,
  edits: readonly { id: number; target_text: string }[],
) {
  soThuTu.push(`save:${chapterId}`)
  const ketQua = await recordSave(chapterId, edits)
  soThuTu.push(`save-done:${chapterId}`)
  return ketQua
}

/**
 * 🔵 **CODE REVIEW 2026-08-18** — móc chạy **BÊN TRONG** lượt `openAdjacentChapter`, tức đúng
 * trong cửa sổ giữa bước ② và bước ③ của `switchChapter`.
 *
 * 🔴 Đây là thứ duy nhất dựng lại được cuộc đua thật: bốn lỗ mà lượt rà ba tầng tìm ra đều đi
 * qua **sạch** cả 14 ca cũ, vì không ca nào có mặt **trong lúc** lượt IPC đang bay.
 */
const trongLucChuyen: { value: (() => void) | null } = { value: null }

/** Đếm lượt `readOpenChapter` — Panel Source phải nạp lại sau một lượt chuyển thành công. */
const docChuong: string[] = []

/** Thân `readOpenChapter` giả — trả nguyên văn **theo Chương đang mở**, cùng lý do `docChuongDangMo`. */
async function docNguyenVanGia() {
  docChuong.push(`chapter:${chuongDangMo.value}`)
  return {
    chapter: {
      chapter_id: chuongDangMo.value,
      source_text: `Nguyen van cua Chuong ${chuongDangMo.value}.`,
      source_lang: 'en',
    },
    error: null,
  }
}

/** Thân `openAdjacentChapter` giả. */
async function chuyenGia(direction: ChapterDirection) {
  soThuTu.push(`switch:${direction}`)

  // ⚠️ Chạy **trước** lượt trả về: `switchChapter` đang `await` ở đây, nên đây chính là cửa sổ
  // mà người dùng thật gõ được thêm một ký tự.
  trongLucChuyen.value?.()
  if (loiChuyen.value) {
    return {
      switched: null,
      error: {
        code: 'store.read_failed',
        message_key: 'err.store.read_failed',
        params: {},
        retryable: true,
      },
    }
  }
  const { outcome, chapterId } = ketQuaChuyen.value
  if (outcome === 'moved' && chapterId !== null) {
    // Nửa Rust dời con trỏ Chương NGAY tại lượt gọi này — đó chính là điều làm thứ tự
    // trở thành một mệnh đề về mất dữ liệu chứ không một mệnh đề về sự gọn gàng.
    chuongDangMo.value = chapterId
    return {
      switched: {
        outcome,
        chapter: { chapter_id: chapterId, source_text: 'Chuong moi.', source_lang: 'zh' },
      },
      error: null,
    }
  }
  return { switched: { outcome, chapter: null }, error: null }
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: docChuongDangMo,
    saveSegmentTargets: ghiRoiLuu,
    setSegmentOmitted: catBoGia,
  }
})

vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return { ...actual, openAdjacentChapter: chuyenGia, readOpenChapter: docNguyenVanGia }
})

async function tuoi() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const StatusBar = (await import('../../src/StatusBar.vue')).default
  await state.ensureSegmentsLoaded()
  return { state, StatusBar }
}

/** Nhường một vòng microtask cho lượt chuyển mà `goToNextChapter` phát bằng `void`. */
const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0))

/** Câu đang hiện trên thanh trạng thái, đọc qua **chính** component. */
function cauTrenThanh(wrapper: ReturnType<typeof mount>): string | null {
  const el = wrapper.find('.notice')
  return el.exists() ? el.text() : null
}

beforeEach(() => {
  resetRecorder()
  soThuTu.length = 0
  docChuong.length = 0
  trongLucChuyen.value = null
  chuongDangMo.value = FIXTURE_CHAPTER_ID
  ketQuaChuyen.value = { outcome: 'moved', chapterId: 99 }
  loiChuyen.value = false
  boCatBoTruot.value = false
})

describe('AC3 — flush XONG rồi mới đổi Chương, và thứ tự là mệnh đề', () => {
  it('🔴 văn bản đang gõ được flush TRƯỚC lượt `open_adjacent_chapter`, mang `chapter_id` CŨ', async () => {
    const { state } = await tuoi()
    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'chữ chưa lưu của Chương cũ')

    state.goToNextChapter()
    await settle()

    // 🔴 Mốc phải so là `save-done`, KHÔNG `save` — xem khối lý do ở [`ghiRoiLuu`]. Lô phải
    //    **đáp về** trước lượt chuyển, không chỉ **rời đi** trước.
    const iXong = soThuTu.indexOf(`save-done:${FIXTURE_CHAPTER_ID}`)
    const iChuyen = soThuTu.indexOf('switch:next')
    expect(iXong).toBeGreaterThanOrEqual(0)
    expect(iChuyen).toBeGreaterThanOrEqual(0)
    expect(iXong).toBeLessThan(iChuyen)

    // 🔴 Và lô mang `chapter_id` của Chương **CŨ** — nếu nó mang Chương mới thì Rust trả
    //    `segment.unknown_ids` và chữ biến mất im lặng.
    expect(saveCalls[0].chapterId).toBe(FIXTURE_CHAPTER_ID)
    expect(saveCalls[0].edits).toEqual([{ id: 11, target_text: 'chữ chưa lưu của Chương cũ' }])
  })

  it('🔴 flush TRƯỢT ⇒ lượt chuyển bị CHẶN, và `open_adjacent_chapter` KHÔNG chạy', async () => {
    // Khuôn đã có chữ ký ở `libraryImport.ts:145-150`: *"Người dùng bị cản một lượt và thấy
    // lý do; họ thử lại, hoặc họ chép bản dịch ra ngoài. Cả hai đường đều tốt hơn một lượt
    // mất chữ không ai biết."*
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'chữ sẽ không xuống được đĩa')
    failNextSave.value = true

    state.goToNextChapter()
    await settle()
    await wrapper.vm.$nextTick()

    expect(soThuTu).not.toContain('switch:next')
    expect(state.editorChapterId.value).toBe(FIXTURE_CHAPTER_ID)
    expect(cauTrenThanh(wrapper)).not.toBeNull()
    wrapper.unmount()
  })

  it('không có gì để lưu ⇒ vẫn chuyển được, và KHÔNG một lượt ghi rỗng nào', async () => {
    const { state } = await tuoi()

    state.goToNextChapter()
    await settle()

    expect(saveCalls.length).toBe(0)
    expect(soThuTu).toContain('switch:next')
    expect(state.editorChapterId.value).toBe(99)
  })
})

describe('AC1 · AC2 — Chương kề mở ra, và lưới nạp lại', () => {
  it('🔴 chuyển thành công ⇒ lưới mang Chương MỚI, không Chương cũ', async () => {
    const { state } = await tuoi()
    expect(state.editorChapterId.value).toBe(FIXTURE_CHAPTER_ID)

    state.goToNextChapter()
    await settle()

    expect(state.editorChapterId.value).toBe(99)
    // Lượt nạp lại phải THẬT SỰ chạy — `resetEditorPanel()` gỡ cờ `requested`, và một cài đặt
    // quên gỡ nó sẽ để lưới đứng nguyên trên ảnh chụp cũ mà không lỗi nào.
    expect(soThuTu.filter((d) => d.startsWith('read:'))).toEqual([
      `read:${FIXTURE_CHAPTER_ID}`,
      'read:99',
    ])
  })

  it('`prev` gửi đúng hướng xuống Rust — webview KHÔNG tự quyết Chương nào là kề', async () => {
    const { state } = await tuoi()

    state.goToPrevChapter()
    await settle()

    expect(soThuTu).toContain('switch:prev')
    expect(soThuTu).not.toContain('switch:next')
  })

  it('🔴 lượt chuyển dọn state của Chương cũ — `caretSegmentId` không sống sót', async () => {
    const { state } = await tuoi()
    state.setEditorCaret(12)
    expect(state.editorCaretSegmentId.value).toBe(12)

    state.goToNextChapter()
    await settle()

    // Câu số 12 của Chương MỚI là một hàng người dùng chưa từng chạm. Một caret sống sót là
    // cùng lớp lỗi mà `resetEditorPanel` đã phải vá cho `confirmError`/`caretPlacement`.
    expect(state.editorCaretSegmentId.value).toBeNull()
  })
})

describe('AC4 — biên Chương báo rõ, và KHÔNG quay vòng', () => {
  it('🔴 ở Chương cuối ⇒ câu nói về CHƯƠNG, không về CÂU', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    ketQuaChuyen.value = { outcome: 'at-last', chapterId: null }

    state.goToNextChapter()
    await settle()
    await wrapper.vm.$nextTick()

    // 🔴 Đây là mệnh đề chống *"màn hình nói dối"*: `panel.grid.nav_at_last` nói *"câu cuối
    //    Chương"*, và tái dùng nó ở đây trả lời sai chính câu hỏi người dùng vừa đặt.
    expect(cauTrenThanh(wrapper)).toBe(
      'Đã ở Chương cuối của Tác phẩm — không có Chương nào phía sau.',
    )
    expect(state.editorChapterId.value).toBe(FIXTURE_CHAPTER_ID)
    wrapper.unmount()
  })

  it('🔴 ở Chương đầu ⇒ câu ĐỐI XỨNG, và hai câu KHÁC NHAU', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    ketQuaChuyen.value = { outcome: 'at-first', chapterId: null }

    state.goToPrevChapter()
    await settle()
    await wrapper.vm.$nextTick()

    expect(cauTrenThanh(wrapper)).toBe(
      'Đã ở Chương đầu của Tác phẩm — không có Chương nào phía trước.',
    )
    expect(state.editorChapterId.value).toBe(FIXTURE_CHAPTER_ID)
    wrapper.unmount()
  })

  it('🔴 vượt biên KHÔNG nạp lại lưới — một lượt nạp thừa là một lượt IPC trên 9.850 hàng', async () => {
    const { state } = await tuoi()
    ketQuaChuyen.value = { outcome: 'at-last', chapterId: null }

    state.goToNextChapter()
    await settle()

    expect(soThuTu.filter((d) => d.startsWith('read:'))).toEqual([`read:${FIXTURE_CHAPTER_ID}`])
  })

  it('chuyển ĐƯỢC ⇒ thanh trạng thái im — một câu thừa đẩy mất mốc "Đã lưu"', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)

    state.goToNextChapter()
    await settle()
    await wrapper.vm.$nextTick()

    expect(cauTrenThanh(wrapper)).toBeNull()
    wrapper.unmount()
  })
})

describe('hai cửa chặn mà không AC nào nêu', () => {
  it('🔴 HAI lượt liên tiếp ⇒ ĐÚNG MỘT lượt `open_adjacent_chapter`', async () => {
    // Một lượt chuyển đi qua HAI lượt IPC nối tiếp, và phím không chờ ai. Không chốt thì
    // người dùng bấm hai lần và nhảy HAI Chương trong khi màn hình kịp nạp một.
    const { state } = await tuoi()

    state.goToNextChapter()
    state.goToNextChapter()
    await settle()

    expect(soThuTu.filter((d) => d === 'switch:next').length).toBe(1)
  })

  it('🔴 lỗi IPC ⇒ KHÔNG nạp lại lưới', async () => {
    // 🔵 **SỬA 2026-08-18 (code review ba tầng).** Ca này từng khẳng định thêm
    // `expect(state.editorLoadError.value?.code).toBe('store.read_failed')` — tức nó **khoá
    // lại** đúng khuyết tật: ghi `loadError` làm `editorHasLoaded()` trả `false` vĩnh viễn và
    // khoá chết cả lưới, mọi lệnh điều hướng, lẫn chính lượt thử lại. Một ca test khoá một
    // khuyết tật là một ca test làm khuyết tật ấy sống lâu hơn.
    // ⇒ Vế *"không nạp lại"* ở dưới **vẫn đúng và giữ nguyên**; vế kênh báo lỗi chuyển sang
    //   `describe('🔴 một lỗi IPC KHÔNG được khoá chết Editor')` ở cuối tệp.
    const { state } = await tuoi()
    loiChuyen.value = true

    state.goToNextChapter()
    await settle()

    expect(soThuTu.filter((d) => d.startsWith('read:'))).toEqual([`read:${FIXTURE_CHAPTER_ID}`])
  })
})

describe('chữ ký #8(a) — hai ô sót của `resetEditorPanel()`, vá cùng lượt', () => {
  it('🔴 `sourceCut` KHÔNG sống sót qua lượt đổi Chương', async () => {
    // Đường hỏng nếu ca này đỏ: người dùng `Mod`+click đánh dấu chỗ cắt ở câu số 11 của
    // Chương A, chuyển Chương, rồi `⌘/` — lệnh tách chạy trên `segment.id` 11 của Chương
    // MỚI, một hàng chưa từng nhìn thấy, trên dữ liệu AD-5 không cho hoàn tác.
    const { state } = await tuoi()
    state.setEditorCaret(11)
    state.setEditorSourceCut(11, 2)
    expect(state.editorSourceCut.value).not.toBeNull()

    state.goToNextChapter()
    await settle()

    expect(state.editorSourceCut.value).toBeNull()
  })

  it('🔴 `omitError` KHÔNG sống sót — ô CHƯA AI NÊU, và hôm nay nó vô hình', async () => {
    // ⚠️ Ô này lọt qua cả lượt rà 2.9 (vốn vá hai ô **cùng hạng**: `confirmError` ·
    //    `regroupError`) vì nó là ô duy nhất trong hạng đó **chưa component nào đọc**, tức
    //    biểu hiện của nó hôm nay là **0 pixel**. Đó là lý do để vá **bây giờ**, không phải
    //    lý do để hoãn: ngày một component đọc `editorOmitError`, khuyết tật đã sẵn sàng —
    //    một `IpcError` mang `params.segment_id` của Chương cũ, hiện trên Chương mới.
    const { state } = await tuoi()
    state.setEditorCaret(11)
    boCatBoTruot.value = true
    await state.setCurrentSegmentOmitted(true)
    expect(state.editorOmitError.value).not.toBeNull()

    state.goToNextChapter()
    await settle()

    expect(state.editorOmitError.value).toBeNull()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// 🔵 CODE REVIEW BA TẦNG 2026-08-18 — BỐN LỖ, VÀ CẢ BỐN ĐI QUA SẠCH 14 CA TRÊN
// ═════════════════════════════════════════════════════════════════════════════════
// Điểm chung của cả bốn: chúng sống **trong lúc** lượt IPC đang bay, hoặc ở một panel mà tệp
// này chưa từng nhìn tới. Mười bốn ca trên đo *"kết quả cuối"* và *"thứ tự các lượt gọi"* —
// hai phép đo đúng, và không phép nào trong hai phép ấy có mặt tại cửa sổ giữa ② và ③.
// ⇒ Bài học lặp lại của kho: *"một ca chưa bao giờ đỏ là một ca chưa ai biết nó canh gì"*.

describe('🔴 cửa sổ giữa ② và ③ — chữ gõ lúc lượt chuyển đang bay', () => {
  it('ký tự gõ trong lúc `open_adjacent_chapter` đang bay KHÔNG vào tập chờ để rồi bị vứt', async () => {
    // Đường hỏng nếu ca này đỏ: ① chứng minh tập chờ sạch → ② đang bay, người dùng gõ thêm
    // một ký tự → `flush.markChanged` nhận nó → ③ `resetEditorPanel()` gọi `flush.reset()`,
    // hàm **vứt vô điều kiện** → ký tự biến mất khỏi đĩa, không lỗi, không log, không cảnh
    // báo, trên dữ liệu mà AD-5 không cho hoàn tác.
    const { state } = await tuoi()
    state.setEditorCaret(11)

    trongLucChuyen.value = () => {
      state.noteEditorEdit(11, 'chu go trong cua so chet nguoi')
    }

    state.goToNextChapter()
    await settle()

    // 🔴 Mệnh đề là *"nó không bao giờ ĐƯỢC NHẬN"*, không *"nó được cứu"*. Ice ký đường khoá
    //    gõ: đóng cửa sổ từ gốc làm *"① là phép chứng minh duy nhất"* đúng theo cấu tạo.
    expect(state.editorEditedText.value.get(11)).toBeUndefined()
    // Và không một lô nào mang ký tự đó — kể cả một lô ghi vào Chương MỚI, thứ sẽ bị Rust
    // từ chối bằng `segment.unknown_ids` và biến một lượt cứu thành một lượt mất khác.
    expect(saveCalls.every((c) => c.edits.every((e) => !e.target_text.includes('cua so')))).toBe(
      true,
    )
    expect(state.editorChapterId.value).toBe(99)
  })

  it('cửa khoá NHẢ sau lượt chuyển — gõ tiếp trên Chương mới vẫn ăn', async () => {
    // Đối chứng cho ca trên: một cửa khoá không nhả là một Editor chỉ đọc vĩnh viễn, và đó
    // là cách một bản vá chống mất chữ tự biến thành một khuyết tật nặng hơn.
    const { state } = await tuoi()

    state.goToNextChapter()
    await settle()

    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'chu go sau khi da chuyen xong')
    expect(state.editorEditedText.value.get(11)).toBe('chu go sau khi da chuyen xong')
  })
})

describe('🔴 một lỗi IPC KHÔNG được khoá chết Editor', () => {
  it('lỗi ở lượt chuyển KHÔNG đầu độc `loadError`, và lượt sau vẫn thử lại được', async () => {
    // Đường hỏng nếu ca này đỏ: `editorHasLoaded()` kiểm `loadError === null`, và không đường
    // nào dọn `loadError` ngoài `resetEditorPanel()` — mà nhánh lỗi không gọi nó. ⇒ lưới bị
    // thay bằng một dòng lỗi, mọi lệnh điều hướng báo *"Chương đang tải"* (sai sự thật), và
    // chính `switchChapter` tự khoá mình ở cửa chặn đầu hàm. Lối thoát duy nhất: rời Tác phẩm.
    const { state } = await tuoi()
    loiChuyen.value = true

    state.goToNextChapter()
    await settle()

    expect(state.editorLoadError.value).toBeNull()
    expect(state.editorChapterId.value).toBe(FIXTURE_CHAPTER_ID)

    // 🔴 Vế quyết định: lượt THỨ HAI phải chạy thật, không bị cửa `editorHasLoaded()` nuốt.
    loiChuyen.value = false
    state.goToNextChapter()
    await settle()

    expect(soThuTu.filter((d) => d === 'switch:next').length).toBe(2)
    expect(state.editorChapterId.value).toBe(99)
  })

  it('lỗi ở lượt chuyển nói về CHUYỂN CHƯƠNG, không về một lượt nạp hỏng', async () => {
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    loiChuyen.value = true

    state.goToNextChapter()
    await settle()
    await wrapper.vm.$nextTick()

    expect(cauTrenThanh(wrapper)).toBe(
      'Chưa chuyển được Chương. Chương hiện tại vẫn mở nguyên — thử lại.',
    )
    wrapper.unmount()
  })
})

describe('🔴 câu chặn phải nói về thao tác người dùng VỪA LÀM', () => {
  it('flush trượt ⇒ câu nói *chưa CHUYỂN CHƯƠNG*, không *chưa XÁC NHẬN*', async () => {
    // 🔴 Đây đúng nguyên tắc mà Quyết định #5 của story này đã đặt cho biên Chương — *"tái
    //    dùng một câu của CÂU cho biên CHƯƠNG là để màn hình nói dối"* — nhưng nhánh flush
    //    cách đó bốn dòng vẫn mượn kênh `confirm`, và `vi.json:99` đọc nguyên văn
    //    *"…nên chưa **xác nhận**"* cho một người dùng vừa bấm `Mod+Alt+]`.
    // ⚠️ Ca cũ (`AC3`, nhánh flush trượt) chỉ khẳng định `not.toBeNull()` nên nó xanh với
    //    **bất kỳ** câu nào — đó là lý do lỗ này đi qua sạch cả 8 phép đột biến của Task 6.5.
    const { state, StatusBar } = await tuoi()
    const wrapper = mount(StatusBar)
    state.setEditorCaret(11)
    state.noteEditorEdit(11, 'chu se khong xuong duoc dia')
    failNextSave.value = true

    state.goToNextChapter()
    await settle()
    await wrapper.vm.$nextTick()

    const cau = cauTrenThanh(wrapper)
    expect(cau).toBe(
      'Chưa lưu được bản dịch nên chưa chuyển Chương. Bản dịch vẫn còn trên màn hình.',
    )
    expect(cau).not.toContain('xác nhận')
    wrapper.unmount()
  })
})

describe('🔴 Panel Source đi CÙNG LƯỢT với Panel Editor', () => {
  it('chuyển thành công ⇒ nguyên văn nạp lại theo Chương MỚI', async () => {
    // Đường hỏng nếu ca này đỏ: `chapterRequested` ở `sourcePanelState.ts` là một cache
    // module-level **không có khoá vô hiệu hoá**, và chỗ gọi `resetSourcePanel()` hôm nay vẫn
    // là đúng một — `libraryImport.ts::finishSubmit`. ⇒ lưới bản dịch sang Chương mới trong
    // khi nguyên văn, bảng âm Hán Việt và `source_lang` vẫn của Chương cũ: hai panel trên
    // cùng một màn hình nói về hai Chương khác nhau, không lỗi nào.
    const { state } = await tuoi()
    const source = await import('../../src/panels/sourcePanelState')
    await source.ensureChapterLoaded()
    expect(source.sourceChapter.value?.chapter_id).toBe(FIXTURE_CHAPTER_ID)

    state.goToNextChapter()
    await settle()

    expect(source.sourceChapter.value?.chapter_id).toBe(99)
    // Vứt state cũ là CHƯA ĐỦ: `resetSourcePanel()` gỡ cờ `chapterRequested` nhưng chỗ gọi
    // duy nhất của hàm nạp là `GridPanel.vue::onMounted`, mà panel sống trong `<KeepAlive>`.
    expect(docChuong).toEqual([`chapter:${FIXTURE_CHAPTER_ID}`, 'chapter:99'])
  })

  it('vượt biên ⇒ KHÔNG nạp lại nguyên văn — không có gì đổi để mà nạp', async () => {
    const { state } = await tuoi()
    const source = await import('../../src/panels/sourcePanelState')
    await source.ensureChapterLoaded()
    ketQuaChuyen.value = { outcome: 'at-last', chapterId: null }

    state.goToNextChapter()
    await settle()

    expect(docChuong).toEqual([`chapter:${FIXTURE_CHAPTER_ID}`])
  })
})
