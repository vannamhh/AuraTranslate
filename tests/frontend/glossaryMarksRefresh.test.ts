/**
 * Story 3.4b — ba hàng CÒN HỞ của I/O Matrix mà `glossaryMarksMap.test.ts` (hàm thuần) và
 * `glossaryHoverSelection.test.ts` (vế tiêu điểm) không chạm tới, rà bởi Ice 2026-08-21:
 *   ① *"IPC trả lỗi | kho đóng giữa chừng ⇒ KHÔNG đánh dấu gì; lỗi hiện qua tError(); KHÔNG
 *      coi như rỗng"*;
 *   ② *"Gộp/tách segment | applyRegroup chạy ⇒ Dấu làm mới; không dấu nào trỏ vào segment đã
 *      về hưu"*;
 *   ③ *"Thêm nhanh một thuật ngữ (3.3) | lưu thành công ⇒ Dấu xuất hiện KHÔNG CẦN mở lại
 *      Chương"*.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO MỖI CA DƯỚI ĐÂY LÀ MỘT ĐỐI CHỨNG "GỠ LỜI GỌI THÌ CA PHẢI ĐỎ"
 * ═════════════════════════════════════════════════════════════════════════════════
 * Ca ② và ③ KHÔNG kiểm "một hàm đã được gọi" (một spy) — chúng kiểm KẾT QUẢ quan sát được
 * trên DOM. Thiết kế: lượt IPC `glossaryMarksForChapter` ĐẦU TIÊN (lúc mở Chương) luôn trả
 * `marks: []` — tức KHÔNG dấu nào hiện trước thao tác đang kiểm. Chỉ lượt gọi THỨ HAI (do
 * `refreshGlossaryMarks()` phát ra sau gộp/thêm nhanh) mới trả một dấu thật. ⇒ Nếu ai đó gỡ
 * lời gọi `refreshGlossaryMarks()` ra khỏi `editorPanelState.ts`/`glossaryQuickAddState.ts`,
 * `glossaryMarks.value` giữ nguyên mảng RỖNG của lượt đầu, và dấu **không bao giờ xuất
 * hiện** — ca ĐỎ ngay, không cần đọc số lượt gọi.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { FIXTURE_CHAPTER_ID, FIXTURE_SEGMENTS, resetRecorder, saveCalls } from './support/segmentFixture'
import type { ChapterSegment, RegroupOutcome } from '../../src/config/segment'
import type { GlossaryMark } from '../../src/config/glossary'

/** Hàng đợi câu trả lời của `glossaryMarksForChapter` — tiêu thụ THEO THỨ TỰ lượt gọi, giữ
 * nguyên câu trả lời CUỐI nếu hàng đợi cạn. */
let hangDoiMarks: { marks: GlossaryMark[] | null; error: unknown }[] = []
let soLuotGoiMarks = 0
async function traDauGia() {
  soLuotGoiMarks += 1
  // ⚠️ Hàng đợi RỖNG là một nhánh CÓ TÊN, không một fallback ngầm sau `??`.
  // Lý do phải viết thế này: `tsconfig` không bật `noUncheckedIndexedAccess` nên phép tra chỉ
  // số khai là LUÔN có giá trị, và TypeScript còn **thu hẹp `const` theo giá trị gán** — nên
  // một lời khai `| undefined` rộng hơn cũng bị narrow ngược lại. Cả hai lối tắt đều sai luật
  // kho: gỡ `??` mở một ca sập (`Math.min(n, -1)` = **-1** khi hàng đợi rỗng), còn
  // `eslint-disable` là một miễn trừ cho một mệnh đề đang hỏng. Nêu điều kiện ra là cách duy
  // nhất khiến kiểu và mã nói cùng một câu.
  if (hangDoiMarks.length === 0) return { marks: [], error: null }
  return hangDoiMarks[Math.min(soLuotGoiMarks - 1, hangDoiMarks.length - 1)]
}

/** Con trỏ "Chương đang mở" DÙNG CHUNG cho `readOpenChapter`/`readOpenChapterSegments` giả —
 * mặc định luôn là `FIXTURE_CHAPTER_ID`; chỉ ca "chuyển Chương kề" (P3) đổi nó qua
 * `openAdjacentChapterGia`. */
const chuongDangMo = { value: FIXTURE_CHAPTER_ID }

/** Chương THỨ HAI, dùng riêng cho ca "chuyển Chương kề" — một segment DUY NHẤT, nội dung khác
 * hẳn `FIXTURE_SEGMENTS` để không lẫn mark của Chương cũ vào Chương mới một cách tình cờ. */
const CHUONG_B_ID = 99
const CHUONG_B_SEGMENTS: ChapterSegment[] = [
  {
    id: 50,
    ord: 1,
    source_text: '你好',
    target_text: 'Xin chao',
    is_paragraph_end: true,
    retired_at: null,
    status: 'draft',
    is_omitted: false,
    is_target_paragraph_end: true,
  },
]

async function docNguyenVanGia() {
  return {
    chapter: { chapter_id: chuongDangMo.value, source_text: 'khong dung o day', source_lang: 'zh' },
    error: null,
  }
}

async function docSegmentTheoChuong() {
  return {
    loaded: {
      chapter_id: chuongDangMo.value,
      segments: (chuongDangMo.value === CHUONG_B_ID ? CHUONG_B_SEGMENTS : FIXTURE_SEGMENTS).map((s) => ({ ...s })),
    },
    error: null,
  }
}

/** `openAdjacentChapter` giả — chuyển sang Chương B, cùng khuôn `chuyenGia` của
 * `editorChapterSwitch.test.ts`. */
async function chuyenChuongGia() {
  chuongDangMo.value = CHUONG_B_ID
  return {
    switched: {
      outcome: 'moved' as const,
      chapter: { chapter_id: CHUONG_B_ID, source_text: 'khong dung o day (B)', source_lang: 'zh' },
    },
    error: null,
  }
}

let ketQuaMerge: { outcome: RegroupOutcome | null; error: unknown } = { outcome: null, error: null }
async function mergeGia() {
  return ketQuaMerge
}

let ketQuaAddTerm: { value: number | null; error: unknown } = { value: 1, error: null }
async function addTermGia() {
  return ketQuaAddTerm
}

/** `lookupGlossaryTerm` giả — luôn "chưa có mục nào", tầng Tác phẩm không dùng được (chạy
 * ngoài Tauri thật). Đây LÀ điều kiện để `quickAddMode` thoát khỏi `'unknown'` và cho phép
 * `saveGlossaryQuickAdd()` chạy tới nhánh 'add' — không phải mệnh đề đang kiểm ở tệp này. */
async function lookupGia() {
  return { found: 'none' as const, workTierAvailable: false }
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: docSegmentTheoChuong,
    saveSegmentTargets: async (chapterId: number, edits: readonly { id: number; target_text: string }[]) => {
      saveCalls.push({ chapterId, edits: edits.map((e) => ({ ...e })) })
      return { outcome: { chapter_id: chapterId, saved: edits.length }, error: null }
    },
    mergeSegments: mergeGia,
  }
})
vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return { ...actual, readOpenChapter: docNguyenVanGia, openAdjacentChapter: chuyenChuongGia }
})
vi.mock('../../src/config/glossary', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/glossary')>()
  return { ...actual, glossaryMarksForChapter: traDauGia, addGlossaryTerm: addTermGia, lookupGlossaryTerm: lookupGia }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

const daMount: { unmount: () => void }[] = []
afterEach(() => {
  for (const w of daMount.splice(0)) w.unmount()
  document.body.innerHTML = ''
})

async function mountGrid() {
  vi.resetModules()
  const editorState = await import('../../src/panels/editorPanelState')
  const sourceState = await import('../../src/panels/sourcePanelState')
  const quickAdd = await import('../../src/glossaryQuickAddState')
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default

  const wrapper = mount(GridPanel, {
    props: { params: {} } as never,
    global: { stubs: STUBS },
    attachTo: document.body,
  })
  daMount.push(wrapper)

  await editorState.ensureSegmentsLoaded()
  await sourceState.ensureChapterLoaded()
  await wrapper.vm.$nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await wrapper.vm.$nextTick()

  return { editorState, sourceState, quickAdd, wrapper }
}

beforeEach(() => {
  resetRecorder()
  hangDoiMarks = [{ marks: [], error: null }]
  soLuotGoiMarks = 0
  ketQuaMerge = { outcome: null, error: null }
  ketQuaAddTerm = { value: 1, error: null }
  chuongDangMo.value = FIXTURE_CHAPTER_ID
})

describe('Story 3.4b — IPC trả lỗi: KHÔNG đánh dấu gì, KHÔNG coi như rỗng', () => {
  it('🔴 lỗi tải dấu ⇒ 0 mảnh mang lớp glossary-*, và `glossaryMarksHaveLoaded()` vẫn `false`', async () => {
    hangDoiMarks = [
      {
        marks: null,
        error: { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: true },
      },
    ]
    const { wrapper } = await mountGrid()

    const glossaryMarksState = await import('../../src/panels/glossaryMarksState')
    // 🔴 Mệnh đề PHÂN BIỆT: một danh sách RỖNG một mình không nói được "lỗi" hay "Chương sạch
    // thuật ngữ". `glossaryMarksHaveLoaded()` là vị từ phải hỏi TRƯỚC — nó `false` ở đây, khác
    // hẳn ca "Chương không có thuật ngữ nào" (nơi nó `true` VÀ marks rỗng).
    expect(glossaryMarksState.glossaryMarksHaveLoaded()).toBe(false)
    expect(glossaryMarksState.glossaryMarksError.value?.code).toBe('store.open_failed')
    expect(glossaryMarksState.glossaryMarks.value).toEqual([])

    // Không đánh dấu gì trên DOM.
    expect(document.querySelectorAll('.glossary-confirmed, .glossary-pending')).toHaveLength(0)
    wrapper.unmount()
  })

  it('🔴 lỗi hiện qua `tError()` ở StatusBar khi không có gì đang hover', async () => {
    hangDoiMarks = [
      {
        marks: null,
        error: { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: true },
      },
    ]
    await mountGrid()

    const StatusBar = (await import('../../src/StatusBar.vue')).default
    const bar = mount(StatusBar)
    daMount.push(bar)
    await bar.vm.$nextTick()

    // `err.store.open_failed` đi qua `tError()` — cùng khoá mà `store_is_missing()` phía Rust
    // dùng cho "global.db chưa mở" (`commands/glossary.rs`).
    expect(bar.find('.notice').exists()).toBe(true)
    expect(bar.find('.notice').text().length).toBeGreaterThan(0)
  })

  /**
   * 🔴 P7 (rà ba lớp LẦN BA, 2026-08-21) — bản vá `StatusBar.vue::GLOSSARY_MARKS_ERROR_DISPLAY_MS`
   * tự nó KHÔNG có ca nào canh: gỡ `setTimeout` khỏi bản vá đó thì câu lỗi che "Đã lưu N giây
   * trước" VĨNH VIỄN — đúng khuyết tật P7 đã vá — mà toàn bộ 323 ca cũ vẫn xanh. Ca này đóng
   * đúng lỗ đó, canh HAI mệnh đề (không chỉ "câu lỗi biến mất" — vế ĐÓ một mình bỏ sót đúng cái
   * hại mà P7 tồn tại để chống):
   *   ① lỗi tải dấu ⇒ câu lỗi hiện trên thanh (che "Đã lưu…");
   *   ② sau `GLOSSARY_MARKS_ERROR_DISPLAY_MS`, câu lỗi TẮT và "Đã lưu N giây trước" TRỞ LẠI.
   *
   * ⚠️ Đồng hồ giả CHỈ bật SAU khi `mountGrid()` (cần `setTimeout` THẬT để tự nhường vòng sự
   * kiện qua `await new Promise((r) => setTimeout(r, 0))`) và một lượt `flushEditorNow()` THẬT
   * (cùng lý do) đã xong — `setTimeout` mà ca này cần điều khiển là đúng cái do `watch` của
   * `StatusBar.vue` dựng LÚC lỗi được đặt, tức phải bật đồng hồ giả TRƯỚC bước đó. Tắt lại bằng
   * `vi.useRealTimers()` trong `finally` để không rò sang ca sau (`vi.useFakeTimers()` toàn cục
   * sống sót qua `afterEach` nếu quên tắt).
   */
  it('🔴 P7: câu lỗi hiện rồi TỰ TẮT sau GLOSSARY_MARKS_ERROR_DISPLAY_MS, "Đã lưu N giây trước" trở lại', async () => {
    const { editorState, wrapper } = await mountGrid()

    const StatusBar = (await import('../../src/StatusBar.vue')).default
    const glossaryMarksState = await import('../../src/panels/glossaryMarksState')
    const bar = mount(StatusBar)
    daMount.push(bar)

    // Một lượt LƯU thật trước — để "Đã lưu N giây trước" có nội dung THẬT để TRỞ LẠI sau khi
    // câu lỗi tắt. Không có bước này, vế ② không đối chứng được gì (không có gì để "trở lại").
    editorState.setEditorCaret(11)
    editorState.noteEditorEdit(11, 'một câu đã lưu trước khi lỗi xảy ra')
    await editorState.flushEditorNow()
    await bar.vm.$nextTick()
    expect(bar.find('.saved').exists()).toBe(true)
    expect(bar.find('.notice').exists()).toBe(false)

    try {
      // Đồng hồ GIẢ vào ĐÚNG SAU điểm này — `setTimeout` cần điều khiển là cái mà lượt ĐẶT LỖI
      // ngay dưới sẽ dựng, không phải cái mountGrid()/flush ở trên đã dùng và đã xong.
      vi.useFakeTimers()

      hangDoiMarks.push({
        marks: null,
        error: { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: true },
      })
      await glossaryMarksState.refreshGlossaryMarks(FIXTURE_CHAPTER_ID, FIXTURE_SEGMENTS, 'zh')
      await bar.vm.$nextTick()

      // ① Câu lỗi hiện, che "Đã lưu…".
      expect(bar.find('.notice').exists()).toBe(true)
      expect(bar.find('.saved').exists()).toBe(false)

      // Chưa hết hạn ⇒ câu lỗi vẫn còn — đối chứng rằng phép đo dưới đây THẬT SỰ phụ thuộc
      // thời gian, không phải một lượt xoá tức thời trùng hợp.
      await vi.advanceTimersByTimeAsync(7999)
      await bar.vm.$nextTick()
      expect(bar.find('.notice').exists()).toBe(true)

      // ② Qua mốc ⇒ câu lỗi TỰ TẮT, "Đã lưu N giây trước" TRỞ LẠI chỗ của nó.
      await vi.advanceTimersByTimeAsync(2)
      await bar.vm.$nextTick()
      expect(bar.find('.notice').exists()).toBe(false)
      expect(bar.find('.saved').exists()).toBe(true)
    } finally {
      vi.useRealTimers()
    }

    wrapper.unmount()
  })
})

describe('Story 3.4b — gộp segment làm MỚI dấu, không dấu nào trỏ vào hàng đã về hưu', () => {
  it('🔴 gỡ `refreshGlossaryMarks()` khỏi `applyRegroup` sẽ làm ca này ĐỎ: dấu chỉ xuất hiện SAU khi gộp', async () => {
    const { editorState, wrapper } = await mountGrid()

    // Trước khi gộp: lượt IPC đầu trả `marks: []` ⇒ 0 dấu.
    expect(document.querySelectorAll('.glossary-confirmed, .glossary-pending')).toHaveLength(0)

    // Gộp segment 12 ('二。') VÀO segment 11 ('一。') — Rust trả một hàng MỚI (id 99) thế chỗ
    // cả hai, đúng hình dạng `RegroupOutcome` thật.
    const hangMoi: ChapterSegment = {
      id: 99,
      ord: 1,
      source_text: '一。二。',
      target_text: 'Hắn đẩy cánh cửa ấy ra.Gió thổi tới từ cuối hành lang.',
      is_paragraph_end: true,
      retired_at: null,
      status: 'draft',
      is_omitted: false,
      is_target_paragraph_end: true,
    }
    const cu11 = FIXTURE_SEGMENTS.find((s) => s.id === 11)
    const cu12 = FIXTURE_SEGMENTS.find((s) => s.id === 12)
    ketQuaMerge = { outcome: { retired: [cu11!, cu12!], new_segments: [hangMoi] }, error: null }
    // Lượt IPC THỨ HAI (do `refreshGlossaryMarks` phát sau gộp) trả một dấu phủ '一' của hàng
    // MỚI (offset [0,1) trong `source_text` của hàng 99).
    hangDoiMarks.push({
      marks: [{ start: 0, end: 1, tier: 'global', is_confirmed: true, translation: 'Ghép', id: 1, source_term: 'thuật ngữ' }],
      error: null,
    })

    editorState.setEditorCaret(12)
    expect(await editorState.mergeCurrentSegment()).toBe('done')
    await wrapper.vm.$nextTick()

    // 🔴 Mệnh đề trung tâm: dấu XUẤT HIỆN trên hàng MỚI (99) — chỉ khả thi nếu
    // `refreshGlossaryMarks()` đã phát một lượt IPC thứ hai VÀ mảng mới đã được ánh xạ lại
    // qua `editorSegments.value` hiện hành (hàng 99, không phải 11/12 đã về hưu).
    const oMoi = document.querySelector('[data-col="src"][data-segment-id="99"] .glossary-confirmed')
    expect(oMoi).not.toBeNull()

    // Không hàng nào của segment ĐÃ VỀ HƯU còn tồn tại trong DOM để mang dấu.
    expect(document.querySelector('[data-col="src"][data-segment-id="11"]')).toBeNull()
    expect(document.querySelector('[data-col="src"][data-segment-id="12"]')).toBeNull()

    expect(soLuotGoiMarks).toBe(2)
    wrapper.unmount()
  })
})

describe('Story 3.4b — thêm nhanh một thuật ngữ làm dấu xuất hiện KHÔNG CẦN mở lại Chương', () => {
  it('🔴 gỡ `refreshGlossaryMarks()` khỏi `glossaryQuickAddState.ts` sẽ làm ca này ĐỎ: dấu chỉ xuất hiện SAU khi lưu', async () => {
    const { quickAdd, wrapper } = await mountGrid()

    expect(document.querySelectorAll('.glossary-confirmed, .glossary-pending')).toHaveLength(0)

    // Lượt IPC THỨ HAI (do lưu thành công phát ra) trả một dấu phủ '一' của segment 11.
    hangDoiMarks.push({
      marks: [{ start: 0, end: 1, tier: 'global', is_confirmed: true, translation: 'Một', id: 1, source_term: 'thuật ngữ' }],
      error: null,
    })

    quickAdd.openGlossaryQuickAdd('一')
    quickAdd.quickAddTranslation.value = 'Một'
    // Chờ lượt tra `lookupGlossaryTerm` (chạy ngoài Tauri ⇒ `found: 'unknown', error: null`)
    // để `quickAddMode` thoát khỏi `'unknown'` — cùng điều kiện mà `saveGlossaryQuickAdd` đòi.
    await new Promise((resolve) => setTimeout(resolve, 0))

    const ok = await quickAdd.saveGlossaryQuickAdd()
    expect(ok).toBe(true)
    await wrapper.vm.$nextTick()

    // 🔴 Mệnh đề trung tâm: dấu xuất hiện trên segment 11 MÀ KHÔNG một lượt `ensureChapterLoaded`/
    // mở Chương lại nào chạy giữa chừng — chỉ `saveGlossaryQuickAdd()` vừa chạy.
    const o = document.querySelector('[data-col="src"][data-segment-id="11"] .glossary-confirmed')
    expect(o).not.toBeNull()
    expect(soLuotGoiMarks).toBe(2)
    wrapper.unmount()
  })
})

describe('Story 3.4b — chuyển Chương KỀ nạp đúng dấu của Chương MỚI, không sót dấu Chương cũ', () => {
  it('🔴 gỡ `ensureGlossaryMarksLoaded()` khỏi `switchChapter()` sẽ làm ca này ĐỎ: sau chuyển, KHÔNG dấu nào hiện', async () => {
    // Chương A (mở lúc mount): mark phủ '你' của segment 11 — KHÔNG, segment 11 là '一。' —
    // dùng đúng nội dung FIXTURE: mark phủ '一' [0,1).
    hangDoiMarks = [
      { marks: [{ start: 0, end: 1, tier: 'global', is_confirmed: true, translation: 'Một (A)', id: 1, source_term: 'thuật ngữ' }], error: null },
    ]
    const { editorState, wrapper } = await mountGrid()
    await wrapper.vm.$nextTick()

    // Trước khi chuyển: dấu của Chương A phải có mặt — đối chứng rằng lượt mở ĐẦU tiên đã chạy.
    expect(
      document.querySelector('[data-col="src"][data-segment-id="11"] .glossary-confirmed'),
    ).not.toBeNull()

    // Lượt IPC THỨ HAI (do `switchChapter()` phát ra cho Chương B) trả một dấu KHÁC hẳn, phủ
    // '你' của segment 50 (Chương B, '你好').
    hangDoiMarks.push({
      marks: [{ start: 0, end: 1, tier: 'global', is_confirmed: true, translation: 'Chào (B)', id: 1, source_term: 'thuật ngữ' }],
      error: null,
    })

    editorState.goToNextChapter()
    // `switchChapter()` là một chuỗi `await` dài (flush → `open_adjacent_chapter` →
    // `ensureSegmentsLoaded` → `ensureChapterLoaded` → `ensureGlossaryMarksLoaded`) chạy dưới
    // một `void` ở `goToNextChapter()` — nhường đủ vòng microtask/macrotask cho TRỌN chuỗi đó.
    await new Promise((resolve) => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()
    await new Promise((resolve) => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()

    // 🔴 Mệnh đề trung tâm: dấu của Chương B có mặt trên segment 50 (hàng DUY NHẤT của Chương
    // B) — chỉ khả thi nếu `ensureGlossaryMarksLoaded()` đã phát một lượt IPC MỚI SAU khi
    // `chapterId`/`sourceChapter` đã khớp Chương B.
    const oB = document.querySelector('[data-col="src"][data-segment-id="50"] .glossary-confirmed')
    expect(oB).not.toBeNull()

    // Không hàng nào của Chương A còn tồn tại trong DOM để mang dấu SAI.
    expect(document.querySelector('[data-col="src"][data-segment-id="11"]')).toBeNull()
    expect(document.querySelector('[data-col="src"][data-segment-id="12"]')).toBeNull()
    expect(document.querySelector('[data-col="src"][data-segment-id="13"]')).toBeNull()

    // Đúng MỘT dấu trên toàn lưới — không dấu nào của A sống sót lẫn vào B.
    expect(document.querySelectorAll('.glossary-confirmed, .glossary-pending')).toHaveLength(1)

    expect(soLuotGoiMarks).toBe(2)
    wrapper.unmount()
  })
})

describe('Story 3.4b — `GridPanel → SourceHanViet` qua prop `:glossary-terms` (P4, bề mặt CHÍNH của FR50)', () => {
  it('🔴 chuyển tab Hán Việt ⇒ `.hv-word` mang ĐÚNG lớp dấu mà `GridPanel` tính cho segment đó', async () => {
    // Mark phủ '一' của segment 11 ('一。') — segment ĐẦU của FIXTURE, [0, 1).
    hangDoiMarks = [
      { marks: [{ start: 0, end: 1, tier: 'global', is_confirmed: true, translation: 'Một', id: 1, source_term: 'thuật ngữ' }], error: null },
    ]
    const { sourceState, wrapper } = await mountGrid()

    // Trước khi chuyển tab: đường CHỮ TRẦN đã đúng (đối chứng — không phải mệnh đề đang kiểm).
    expect(
      document.querySelector('[data-col="src"][data-segment-id="11"] .src-piece.glossary-confirmed'),
    ).not.toBeNull()

    // 🔴 Mệnh đề trung tâm: chuyển sang tab Hán Việt (`activeTab.value = 'han_viet'`, hàm THẬT
    // ở `sourcePanelState.ts:321`) — `GridPanel.vue` chuyển từ `v-else` (chữ trần) sang
    // `<SourceHanViet :glossary-terms="glossaryTermsOf(s.id).spans">`. Nếu `GridPanel` tính
    // SAI hoặc quên TRUYỀN prop đó, `SourceHanViet` (đã kiểm đúng ở `hanVietCutAnchors.test.ts`
    // với prop DỰNG TAY) vẫn nhận một mảng RỖNG và không cắt/tô gì — bề mặt vỡ mà cổng nào
    // cũng xanh, đúng cảnh báo P4.
    sourceState.selectSourceTab('han_viet')
    await wrapper.vm.$nextTick()

    expect(document.querySelector('[data-col="src"][data-segment-id="11"] .hv-word')).not.toBeNull()
    const hvWord = document.querySelector('[data-col="src"][data-segment-id="11"] .hv-word.glossary-confirmed')
    expect(hvWord).not.toBeNull()
    // Đối chứng ÂM: hai segment KHÔNG mang dấu không được nhuộm lây.
    expect(
      document.querySelector('[data-col="src"][data-segment-id="12"] .hv-word.glossary-confirmed'),
    ).toBeNull()

    wrapper.unmount()
  })
})

describe('Story 3.4b — AC "mở Chương rồi gõ 200 phím ⇒ đúng 1 lượt gọi" (P9, đếm THẬT)', () => {
  it('🔴 200 lượt `noteEditorEdit` (mô phỏng gõ) sau khi mở Chương KHÔNG phát thêm một lượt `glossaryMarksForChapter` nào', async () => {
    const { editorState } = await mountGrid()

    expect(soLuotGoiMarks).toBe(1) // đúng MỘT lượt cho lượt mở Chương — đối chứng khởi điểm.

    // Mô phỏng 200 phím gõ liên tiếp vào ô bản dịch của segment 11 — đúng đường sản phẩm
    // (`GridPanel.vue` gọi `noteEditorEdit` ở mỗi sự kiện `input` thật).
    let vanBan = ''
    for (let i = 0; i < 200; i += 1) {
      vanBan += 'a'
      editorState.noteEditorEdit(11, vanBan)
    }

    // 🔴 Mệnh đề trung tâm: KHÔNG một đường gõ nào — kể cả gián tiếp qua `noteEditorEdit` — gọi
    // tới `ensureGlossaryMarksLoaded`/`refreshGlossaryMarks`. Một lượt sửa sau này vô tình
    // thêm lời gọi trên đường gõ sẽ làm số này tăng, và ca này bắt được ngay — không cần đợi
    // một phiên NFR2 thật 30 phút.
    expect(soLuotGoiMarks).toBe(1)
  })
})

describe('Story 3.4b — `refreshGlossaryMarksAfterSave` (P11): nhánh guard "chưa mở Chương"', () => {
  it('🔴 lưu thành công MÀ chưa có Chương nào mở ⇒ KHÔNG lượt `glossaryMarksForChapter` nào, KHÔNG ném', async () => {
    vi.resetModules()
    const quickAdd = await import('../../src/glossaryQuickAddState')

    // ⚠️ CỐ Ý không gọi `ensureSegmentsLoaded()`/`ensureChapterLoaded()` — đúng ca "thêm nhanh
    // một thuật ngữ tầng Global từ một bề mặt KHÁC lưới" (Panel Lookup/AI Translation, FR48),
    // nơi chưa chắc đã có Chương nào đang mở. `editorChapterId`/`sourceChapter` ở nguyên `null`.
    quickAdd.openGlossaryQuickAdd('một thuật ngữ bất kỳ')
    quickAdd.quickAddTranslation.value = 'bản dịch'
    await new Promise((resolve) => setTimeout(resolve, 0)) // chờ `lookupGlossaryTerm` giả về

    const ok = await quickAdd.saveGlossaryQuickAdd()

    expect(ok).toBe(true) // lượt LƯU vẫn thành công — guard chỉ chặn vế LÀM MỚI DẤU.
    // 🔴 Mệnh đề trung tâm: `refreshGlossaryMarksAfterSave` phải THOÁT SỚM ở nhánh guard,
    // không gọi `refreshGlossaryMarks` (nên không một lượt `glossaryMarksForChapter` nào được
    // phát — mount THẬT của tệp này ở các ca khác luôn phát ĐÚNG một lượt lúc mở Chương; ở
    // đây chưa mở Chương nên con số đúng phải là KHÔNG).
    expect(soLuotGoiMarks).toBe(0)
  })
})

describe('Story 3.4b — điểm cắt NGƯỜI DÙNG rơi vào GIỮA một span thuật ngữ (P12)', () => {
  it('🔴 pendingCut ở offset 1 (giữa mark [0,2) phủ TRỌN segment 11) ⇒ hai mảnh, CẢ HAI vẫn mang lớp dấu, cut-mark đúng vị trí', async () => {
    // Mark phủ TRỌN segment 11 ('一。', 2 điểm mã) — [0, 2) — nên KHÔNG một biên thuật ngữ nào
    // được sinh ra (`glossaryMarksBySegment` chỉ thêm biên khi span dừng TRƯỚC cuối segment).
    // Điểm cắt duy nhất trong ca này phải đến từ CHÍNH `pendingCuts`.
    hangDoiMarks = [
      { marks: [{ start: 0, end: 2, tier: 'global', is_confirmed: true, translation: 'Cả câu', id: 1, source_term: 'thuật ngữ' }], error: null },
    ]
    const { editorState, wrapper } = await mountGrid()

    // Đối chứng TRƯỚC khi cắt: đúng MỘT mảnh, mang dấu, không `cut-mark` nào.
    let pieces = document.querySelectorAll('[data-col="src"][data-segment-id="11"] .src-piece')
    expect(pieces).toHaveLength(1)
    expect(pieces[0].classList.contains('glossary-confirmed')).toBe(true)
    expect(document.querySelectorAll('[data-col="src"][data-segment-id="11"] .cut-mark')).toHaveLength(0)

    // Người dùng đặt một điểm cắt CHỜ ở offset 1 — chính giữa '一' và '。', tức GIỮA span
    // thuật ngữ (không trùng ICU, không trùng biên thuật ngữ — vì span không SINH biên nào).
    editorState.setEditorSourceCut(11, 1)
    await wrapper.vm.$nextTick()

    const cell = document.querySelector('[data-col="src"][data-segment-id="11"]')
    expect(cell).not.toBeNull()
    pieces = cell!.querySelectorAll('.src-piece')
    // 🔴 Mệnh đề trung tâm ①: hai tập điểm (pendingCuts ∪ biên thuật ngữ) HỢP LẠI để cắt —
    // dù biên thuật ngữ ở đây RỖNG, pendingCut một mình vẫn phải cắt được.
    expect(pieces).toHaveLength(2)
    expect(pieces[0].getAttribute('data-src-start')).toBe('0')
    expect(pieces[1].getAttribute('data-src-start')).toBe('1')

    // 🔴 Mệnh đề trung tâm ②: CẢ HAI mảnh con vẫn mang lớp dấu — `glossarySpanAt` phải nhận ra
    // rằng cả hai đều nằm TRỌN trong span gốc [0,2), dù không mảnh nào còn dài bằng span đó.
    expect(pieces[0].classList.contains('glossary-confirmed')).toBe(true)
    expect(pieces[1].classList.contains('glossary-confirmed')).toBe(true)

    // 🔴 Mệnh đề trung tâm ③: `cut-mark` (dấu NGẮT ĐOẠN của người dùng) vẽ ĐÚNG MỘT lần, ngay
    // TRƯỚC mảnh thứ hai — không lẫn với việc tô màu thuật ngữ, và không nhân đôi.
    const cutMarks = cell!.querySelectorAll('.cut-mark')
    expect(cutMarks).toHaveLength(1)

    wrapper.unmount()
  })
})
