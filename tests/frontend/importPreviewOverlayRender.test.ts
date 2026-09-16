/**
 * Kết dựng THẬT của `ImportPreviewOverlay.vue` — Story 6.3, vòng rà đối kháng 2, mục 15/22.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO CA NÀY TỒN TẠI
 * ─────────────────────────────────────────────────────────────────────────────
 * Trước ca này, `ImportPreviewOverlay.vue` có **0** test — mọi khẳng định về nó chỉ đi qua
 * `importPreviewState.ts` (state thuần, không đụng DOM). Mục 22 của vòng rà đối kháng 2 đổi
 * `t(\`mode.library.preview.confidence_${…}\`)` (nội suy chuỗi khoá) thành
 * `t(confidenceMessageKey(…))`/`t(tierEmptyMessageKey(…))` (hàm ánh xạ `switch` CẠN) — nhưng
 * hai hàm đó sống TRONG `<script setup>`, không export được, nên đối chứng DUY NHẤT có thể
 * thấy chúng thật sự chạy đúng là DỰNG component và ĐỌC chữ trên màn hình.
 *
 * Test này CHỈ canh phần dựng ba khoá bị đổi (chip tin cậy + hai tầng rỗng) — KHÔNG một bộ
 * test đầy đủ cho toàn bộ overlay (focus trap, Tab, Esc, dải năm ứng viên, … đã có đối
 * chứng ở `importPreviewEncoding.test.ts` cho phần STATE của các hành vi đó, và vẫn CHƯA có
 * ở tầng dựng DOM — nợ còn lại, không phải phạm vi của ca này).
 *
 * ⚠️ Cùng khuôn `glossaryQueue.test.ts::freshOverlay` — `vi.resetModules()` TRƯỚC, rồi nạp
 * ĐỘNG cả state LẪN component trong CÙNG một lượt, để cả hai cùng một thể hiện module (một
 * `import` tĩnh component + một `import` động state sau `resetModules()` sẽ là HAI thể hiện
 * `importPreviewState.ts` khác nhau — component không thấy state test vừa đổi).
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'
import type { ChapterOriginWire, EncodingCandidateWire, ImportEncodingPreview } from '../../src/config/project'

/** Story 6.15 — bốn trường xuất xứ RỖNG, dùng làm giá trị mặc định cho mọi fixture
 * `ChapterSplitPreviewEntryWire` ở tệp này (không ca nào trong tệp cần một giá trị khác
 * rỗng — các ca đọc `origin` sống ở `importPreviewChapterOrigin.test.ts`). */
const ORIGIN_STUB: ChapterOriginWire = {
  author: null,
  site_name: null,
  url: null,
  published_at: null,
  author_confirmed: false,
  site_name_confirmed: false,
  url_confirmed: false,
  published_at_confirmed: false,
}


const previewTextMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (text: string) => previewTextMock(text),
  }
})

function candidate(over: Partial<EncodingCandidateWire> = {}): EncodingCandidateWire {
  return {
    label: 'UTF-8',
    encoding: 'UTF-8',
    preview: 'plain ascii',
    // Story 6.4 — bản chuẩn hoá đi kèm sẵn trên MỖI ô của dải; fixture phải mang trường
    // này để tầng MỚI (chuẩn hoá) hiện nội dung thật thay vì rơi vào nhánh rỗng
    // "undecodable" — hai chuyện khác hẳn nhau (candidate.normalized === null nghĩa là
    // "không ra chữ", không phải "chưa nạp xong fixture").
    normalized: { text: 'plain ascii', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
    // Story 6.5 — cùng lý do `normalized`: khối làm sạch đi kèm sẵn trên MỖI ô. `null` đồng
    // bộ với `normalized: null` (bảng mã "không ra chữ") — xem ca dành riêng cho nhánh đó.
    cleanup: { text: 'plain ascii', spans: [], rules: [], window_truncated: false, final_text: 'plain ascii' },
    // Story 6.6 — cùng lý do `cleanup`: khối tách Chương đi kèm sẵn trên MỖI ô. `null` đồng
    // bộ với `normalized: null`/`cleanup: null` (bảng mã "không ra chữ").
    chapters: {
      chapter_count: 1,
      chapters: [{ ord: 1, title: null, length: 11, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB, source_file: null }],
      broken_item_count: 0,
      needs_review_count: 0,
      clean_count: 1,
      any_signal_participated: false,
    },
    // Story 6.9 — khối tầng 2 (ranh giới bóc) đi kèm sẵn trên MỖI ô, cùng lý do `chapters`.
    // `null` đồng bộ với ba trường trên (bảng mã "không ra chữ") — xem ca dành riêng cho
    // nhánh có khối thật trong tệp test của story đó.
    blocks: null,
    ...over,
  }
}

function preview(over: Partial<ImportEncodingPreview> = {}): ImportEncodingPreview {
  return {
    selected_encoding: 'UTF-8',
    confidence: 'high',
    candidates: [candidate()],
    // candidates mac dinh KHONG rong -- doc .normalized/.cleanup cua ung vien, khong doc
    // hai truong nay.
    self_declared_normalized: null,
    self_declared_cleanup: null,
    self_declared_chapters: null,
    ...over,
  }
}

/**
 * `deps` — **THÊM (Story 6.10a)** para nạp `CommandDeps` THẬT qua `installCommands()`, cùng
 * khuôn `importPreviewBlocks.test.ts::freshOverlay`. Rỗng ở mọi ca CŨ (không đổi hành vi của
 * chúng — `installCommands({})` vẫn đăng ký đủ mọi command, chỉ thiếu dep thì `dispatch()`
 * gọi `portMissing(...)` thay vì ném).
 */
async function freshOverlay(deps: Partial<CommandDeps> = {}) {
  vi.resetModules()
  previewTextMock.mockReset()

  const commands = await import('../../src/commands')
  commands.installCommands(deps as CommandDeps)
  const state = await import('../../src/importPreviewState')
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { commands, state, ImportPreviewOverlay }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('ImportPreviewOverlay.vue — chip tin cậy + hai tầng rỗng dựng ĐÚNG chữ (mục 22)', () => {
  it.each([
    ['self_declared', 'Nguồn tự khai bảng mã'],
    ['high', 'Tự đoán · độ tin cậy cao'],
    ['low', 'Tự đoán · độ tin cậy thấp'],
  ] as const)('confidence=%s dựng đúng chip "%s"', async (confidence, expectedText) => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview({ confidence }), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'text', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    expect(wrapper.find('.ip-confidence-chip').text()).toContain(expectedText)

    wrapper.unmount()
    state.resetImportPreview()
  })

  // 🔵 SỬA 2026-09-05 (Story 6.5) — "tầng 2/3 rỗng" đã HẾT ĐÚNG cho tầng 3: nó nay CÓ THÂN
  // khi ứng viên mang `cleanup` (mặc định của `candidate()` từ story này). Chỉ tầng 2 còn
  // rỗng — ca dành cho tầng 3 rỗng (ứng viên "không ra chữ") đứng riêng ngay dưới.
  // 🔵 SỬA 2026-09-07 (Story 6.9) — lý do rỗng viết lại: "chưa dựng" đã hết đúng (tầng 2 nay
  // CÓ THÂN cho nhánh URL) — đường dán văn bản (nhánh của CHÍNH ca này) vẫn rỗng, nhưng vì
  // "nguồn này không bóc nội dung chính", không phải vì tính năng chưa tồn tại.
  it('tầng 2 rỗng (đường dán văn bản) dựng đúng lý do MỚI — không bóc, không phải "chưa dựng"', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'text', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    const reasons = wrapper.findAll('.ip-tier-empty-reason')
    expect(reasons).toHaveLength(1)
    expect(reasons[0]?.text()).not.toContain('Story 6.9')
    expect(reasons[0]?.text()).toContain('URL')

    wrapper.unmount()
    state.resetImportPreview()
  })

  // Story 6.4 — tầng chuẩn hoá xuống dòng/khoảng trắng CÓ THÂN, chèn giữa tầng 1 (bảng mã)
  // và tầng 2 (ranh giới nội dung, vẫn rỗng). Ứng viên có `normalized` ⇒ hiện văn bản thật
  // + hai số đếm, KHÔNG rơi vào nhánh rỗng "undecodable"/"no_candidate".
  it('tầng chuẩn hoá (Story 6.4) hiện văn bản thật + hai số đếm khi ứng viên có `normalized`', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({
      preview: preview({
        candidates: [
          candidate({
            normalized: {
              text: 'Han nhin ve phia ngon nui xa.',
              joined_lines: 1,
              blank_lines_removed: 2,
              window_truncated: true,
            },
          }),
        ],
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'text', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('.ip-normalized-text').text()).toBe('Han nhin ve phia ngon nui xa.')
    expect(wrapper.find('.ip-normalized-counts').text()).toContain('1')
    expect(wrapper.find('.ip-normalized-counts').text()).toContain('2')
    // `window_truncated: true` ⇒ ghi chú phạm vi cửa sổ phải hiện ra bằng chữ.
    expect(wrapper.find('.ip-normalized-window-note').exists()).toBe(true)
    // Tầng mới KHÔNG rơi vào nhánh rỗng — CHỈ tầng 2 còn `.ip-tier-empty-reason` (tầng 3 nay
    // CÓ THÂN, ứng viên mặc định mang `cleanup`).
    expect(wrapper.findAll('.ip-tier-empty-reason')).toHaveLength(1)

    wrapper.unmount()
    state.resetImportPreview()
  })

  // Story 6.4 — ứng viên "không ra chữ" (`normalized: null`) ⇒ tầng mới rơi vào nhánh rỗng
  // riêng của nó, KHÔNG hiện `.ip-normalized-text`/`.ip-normalized-window-note`.
  it('tầng chuẩn hoá rơi vào nhánh rỗng khi ứng viên đang chọn "không ra chữ"', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({
      // Story 6.5 — `cleanup: null` ĐỒNG BỘ với `normalized: null` (cùng một sự thật: bảng
      // mã này không ra chữ), đúng bất biến mà Rust luôn giữ giữa hai trường.
      preview: preview({ candidates: [candidate({ preview: null, normalized: null, cleanup: null })] }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'text', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('.ip-normalized-text').exists()).toBe(false)
    // Ba tầng rỗng: tầng chuẩn hoá (undecodable) + tầng 2 (6.9) + tầng 3 (undecodable).
    expect(wrapper.findAll('.ip-tier-empty-reason')).toHaveLength(3)

    wrapper.unmount()
    state.resetImportPreview()
  })

  // 🔴 Vá vòng rà 1, mục 1/2 — đối chứng DOM trực tiếp cho lỗi chính vòng rà 1 phát hiện:
  // đường DÁN VĂN BẢN TAY (0 ứng viên) phải hiện bản chuẩn hoá THẬT, không rơi vào
  // "Chưa chọn được ứng viên bảng mã nào".
  it('đường DÁN VĂN BẢN TAY (0 ứng viên) hiện bản chuẩn hoá của self_declared_normalized', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({
      preview: preview({
        confidence: 'self_declared',
        candidates: [],
        self_declared_normalized: {
          text: 'Van ban da dan roi noi lai.',
          joined_lines: 1,
          blank_lines_removed: 0,
          window_truncated: false,
        },
        // Story 6.5 — cùng lý do `self_declared_normalized` ngay trên: thiếu trường này thì
        // tầng 3 rơi vào nhánh rỗng "chưa chọn được ứng viên", và nhánh đó CHIA SẺ tiền tố
        // câu với đúng chuỗi ca này khẳng định VẮNG MẶT ở dưới — false negative nếu bỏ sót.
        self_declared_cleanup: {
          text: 'Van ban da dan roi noi lai.',
          spans: [],
          rules: [],
          window_truncated: false,
          final_text: 'Van ban da dan roi noi lai.',
        },
        // Story 6.6 — cùng lý do `self_declared_cleanup` ngay trên: thiếu trường này thì
        // tầng 4 rơi vào nhánh rỗng "chưa chọn được ứng viên", chia sẻ tiền tố câu với
        // đúng chuỗi ca này khẳng định VẮNG MẶT ở dưới.
        self_declared_chapters: {
          chapter_count: 1,
          chapters: [{ ord: 1, title: null, length: 27, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB, source_file: null }],
          broken_item_count: 0,
          needs_review_count: 0,
          clean_count: 1,
          any_signal_participated: false,
        },
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'van ban dan tay', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('.ip-normalized-text').text()).toBe('Van ban da dan roi noi lai.')
    // Nhãn số đếm phải đọc là "chỗ nối", không phải "dòng" — mục 2 của vá vòng rà 1.
    expect(wrapper.find('.ip-normalized-counts').text()).toContain('chỗ nối')
    expect(wrapper.find('.ip-normalized-counts').text()).not.toContain('dòng đã nối');
    // KHÔNG rơi vào nhánh "chưa chọn được ứng viên" — đúng cái lỗi vòng rà 1 bắt được.
    expect(wrapper.text()).not.toContain('Chưa chọn được ứng viên bảng mã nào')

    wrapper.unmount()
    state.resetImportPreview()
  })

  // Vá vòng rà 1, mục 2 — `self_declared_normalized.text === ''` phải có lời giải thích
  // riêng, không một đoạn `.ip-normalized-text` trống trơn.
  it('văn bản đã chuẩn hoá RỖNG hiện lời giải thích, không một đoạn trắng', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({
      preview: preview({
        confidence: 'self_declared',
        candidates: [],
        self_declared_normalized: { text: '', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', '   ', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('.ip-normalized-text').exists()).toBe(false)
    expect(wrapper.text()).toContain('Không có gì để hiện')

    wrapper.unmount()
    state.resetImportPreview()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.10a — con trỏ *Chương đang chọn*, tầng BÀN PHÍM DOM THẬT (`⌥←`/`⌥→`)
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 Đối chứng cho handler THỨ HAI (`onChapterCursorKeydown`, KHÔNG nới `onTier2Keydown`):
// `⌥` cộng một phím CŨNG thuộc tầng 2 (`j`) không được rơi vào nhánh tầng 2 — nếu nới vị từ
// chặn `altKey` của `onTier2Keydown` thay vì dựng handler riêng, `⌥`+`j` sẽ bắn nhầm
// `import.preview.block_next`.

describe('ImportPreviewOverlay.vue — con trỏ Chương DOM THẬT (`⌥←`/`⌥→`)', () => {
  it('⌥→ bắn import.preview.chapter_next, ⌥← bắn import.preview.chapter_prev', async () => {
    const nextMock = vi.fn()
    const prevMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({
      nextImportPreviewChapter: nextMock,
      prevImportPreviewChapter: prevMock,
    })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.ip-scrim')

    await scrim.trigger('keydown', { key: 'ArrowRight', altKey: true })
    expect(nextMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 'ArrowLeft', altKey: true })
    expect(prevMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('giữ ⌥→ cho auto-repeat KHÔNG bắn một tràng lệnh — chỉ lần đầu', async () => {
    const nextMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({ nextImportPreviewChapter: nextMock })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.ip-scrim')

    await scrim.trigger('keydown', { key: 'ArrowRight', altKey: true })
    await scrim.trigger('keydown', { key: 'ArrowRight', altKey: true, repeat: true })
    await scrim.trigger('keydown', { key: 'ArrowRight', altKey: true, repeat: true })
    expect(nextMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('mũi tên TRẦN (không `⌥`) KHÔNG bắn lệnh Chương — thiếu bổ trợ chính', async () => {
    const nextMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({ nextImportPreviewChapter: nextMock })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.ip-scrim')

    await scrim.trigger('keydown', { key: 'ArrowRight' })
    expect(nextMock).not.toHaveBeenCalled()

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('`⌥`+`j` KHÔNG bắn import.preview.block_next — handler tầng 2 vẫn chặn `altKey`', async () => {
    const nextBlockMock = vi.fn()
    const nextChapterMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({
      nextImportPreviewBlock: nextBlockMock,
      nextImportPreviewChapter: nextChapterMock,
    })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.ip-scrim')

    await scrim.trigger('keydown', { key: 'j', altKey: true })
    expect(nextBlockMock).not.toHaveBeenCalled()
    // `j` không phải `ArrowLeft`/`ArrowRight` — handler Chương cũng không bắn gì.
    expect(nextChapterMock).not.toHaveBeenCalled()

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('lớp phủ ĐÃ ĐÓNG — `⌥→`/`⌥←` không đổi gì (0 command nào bắn, cùng khuôn "không thao tác")', async () => {
    const nextMock = vi.fn()
    const prevMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({
      nextImportPreviewChapter: nextMock,
      prevImportPreviewChapter: prevMock,
    })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    // Đóng lớp phủ THẬT (huỷ) — `.ip-scrim` biến mất khỏi DOM (`v-if="importPreviewIsOpen"`).
    state.cancelImportPreview()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.ip-scrim').exists()).toBe(false)

    // Không còn `.ip-scrim` để mà bắn `keydown` lên — đúng nghĩa "không thao tác nào của màn
    // nhập xảy ra" (I/O Matrix spec 6.10a): không có phần tử nào trong cây của lớp phủ còn
    // sống để nhận sự kiện, nên `nextMock`/`prevMock` chắc chắn không thể bị gọi qua đường
    // bàn phím của NÓ nữa — đối chứng THÊM ở tầng state (`importPreviewChapterCursor.test`,
    // hàm `nextImportPreviewChapter()` gọi trực tiếp) đã khẳng định no-op tuyệt đối, xem
    // `importPreviewChapters.test.ts`.
    expect(nextMock).not.toHaveBeenCalled()
    expect(prevMock).not.toHaveBeenCalled()

    wrapper.unmount()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.10 — bộ lọc "cần xem", tầng BÀN PHÍM DOM THẬT (`⌥W`)
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 Đối chứng đỏ ③ của §Verification spec 6.10 — `⌥W` PHẢI so `event.code === 'KeyW'`, KHÔNG
// `event.key`: trên macOS `⌥W` gõ ra `∑`, nên mọi ca dưới đây dựng sự kiện với CẢ HAI trường
// (`code: 'KeyW'`, `key: '∑'`) — một handler lỡ so `event.key === 'w'` sẽ KHÔNG BAO GIỜ khớp
// và mọi ca "phải bắn lệnh" ở đây sẽ đỏ.

describe('ImportPreviewOverlay.vue — bộ lọc "cần xem" DOM THẬT (`⌥W`)', () => {
  it('⌥W (event.code === KeyW, event.key === macOS ∑) bắn import.preview.chapter_filter_toggle', async () => {
    const toggleMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({ toggleImportPreviewChapterFilter: toggleMock })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.ip-scrim')

    await scrim.trigger('keydown', { code: 'KeyW', key: '∑', altKey: true })
    expect(toggleMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('giữ ⌥W cho auto-repeat KHÔNG bắn một tràng lệnh — chỉ lần đầu', async () => {
    const toggleMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({ toggleImportPreviewChapterFilter: toggleMock })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.ip-scrim')

    await scrim.trigger('keydown', { code: 'KeyW', key: '∑', altKey: true })
    await scrim.trigger('keydown', { code: 'KeyW', key: '∑', altKey: true, repeat: true })
    await scrim.trigger('keydown', { code: 'KeyW', key: '∑', altKey: true, repeat: true })
    expect(toggleMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('`w` TRẦN (không `⌥`) KHÔNG bắn lệnh lọc', async () => {
    const toggleMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({ toggleImportPreviewChapterFilter: toggleMock })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.ip-scrim')

    await scrim.trigger('keydown', { code: 'KeyW', key: 'w' })
    expect(toggleMock).not.toHaveBeenCalled()

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('lớp phủ ĐÃ ĐÓNG — `⌥W` không đổi gì (0 command nào bắn)', async () => {
    const toggleMock = vi.fn()
    const { state, ImportPreviewOverlay } = await freshOverlay({ toggleImportPreviewChapterFilter: toggleMock })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    state.cancelImportPreview()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.ip-scrim').exists()).toBe(false)

    expect(toggleMock).not.toHaveBeenCalled()

    wrapper.unmount()
  })
})

/** N Chương với phán quyết `needs_review` đặt thẳng theo `flags` — Rust là nơi tính (AD-1),
 * fixture chở KẾT QUẢ của nó. */
function chaptersWithFlags(flags: boolean[]) {
  const needs = flags.filter(Boolean).length
  return {
    chapter_count: flags.length,
    chapters: flags.map((needsReview, i) => ({
      ord: i + 1,
      title: `Chuong ${i + 1}`,
      length: 100 + i,
      cleanup_match_count: 0,
      joined_line_count_in_chapter: null,
      needs_review: needsReview,
      review_causes: needsReview ? (['short_length'] as const).slice() : [],
        origin: ORIGIN_STUB, source_file: null,
    })),
    broken_item_count: 0,
    needs_review_count: needs,
    clean_count: flags.length - needs,
    any_signal_participated: true,
  }
}

describe('ImportPreviewOverlay.vue — bộ lọc "cần xem" đổi thứ HIỆN RA (Story 6.10)', () => {
  /**
   * I/O Matrix spec 6.10, hàng *"Bấm `⌥W`"* — vế **tầng 4**: co về Chương cần xem, và **bỏ
   * co gọn**.
   *
   * 🔴 **Vì sao phải khẳng định `⋯` BIẾN MẤT, không chỉ đếm hàng.** §Design Notes spec 6.10
   * chọn "lọc thì bỏ co gọn" chính để ca `aria-activedescendant` trỏ vào một hàng đã bị lọc
   * khỏi DOM **không tồn tại được**. Một phép lọc giữ nguyên co gọn vẫn cho đúng số hàng ở
   * fixture nhỏ — nên ca này gieo **mười** Chương để co gọn THẬT SỰ đang bật trước khi lọc,
   * và khẳng định cả hai chiều.
   */
  it('bật lọc: tầng 4 chỉ còn Chương cần xem, và `⋯` biến mất', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const flags = [false, true, false, false, false, false, false, true, false, false]
    previewTextMock.mockResolvedValue({
      preview: preview({ candidates: [candidate({ chapters: chaptersWithFlags(flags) })] }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'text', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })

    // Tien de: 10 Chuong, chua loc ⇒ khung nhin mac dinh CO GON that su dang bat.
    expect(wrapper.findAll('.ip-chapters-ellipsis').length).toBe(1)
    expect(wrapper.findAll('.ip-chapters-entry').length).toBe(6)

    state.toggleImportPreviewChapterFilter()
    await wrapper.vm.$nextTick()

    expect(state.importPreviewChapterFilterActive.value).toBe(true)
    expect(wrapper.findAll('.ip-chapters-entry').length).toBe(2)
    expect(wrapper.findAll('.ip-chapters-ellipsis').length).toBe(0)

    wrapper.unmount()
    state.resetImportPreview()
  })

  /**
   * I/O Matrix spec 6.10, hàng *"Bảng mã tin cậy thấp"* — cờ **cấp lượt nhập**, nằm **NGOÀI**
   * hai con số.
   *
   * 🔴 **Đây là quyết định #3 Ice ký 2026-09-08, và trước ca này KHÔNG ai canh nó.** Lý do nó
   * được ký: gắn cờ tin cậy thấp lên cả N Chương làm hai số thành `50/0` — tức bộ lọc mất tác
   * dụng **đúng lúc cần nhất**. Ca này gieo `confidence: 'low'` trên mười Chương mà chỉ hai
   * Chương cần xem, rồi khẳng định hai con số vẫn là `2/8` chứ KHÔNG phải `10/0`, trong khi
   * dòng cảnh báo tin cậy thấp **vẫn hiện** ở chỗ riêng của nó.
   */
  it('tin cậy thấp hiện cảnh báo RIÊNG, KHÔNG nhân thành N Chương cần xem', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const flags = [false, true, false, false, false, false, false, true, false, false]
    previewTextMock.mockResolvedValue({
      preview: preview({
        confidence: 'low',
        candidates: [candidate({ chapters: chaptersWithFlags(flags) })],
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'text', null)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('.ip-confidence-chip').text()).toContain('độ tin cậy thấp')

    const bar = wrapper.find('.ip-chapter-filter-bar').text()
    expect(bar).toContain('2 Chương cần xem')
    expect(bar).toContain('8 Chương sạch')
    // 🔴 Ve NGUOC quan trong nhat: tin cay thap KHONG duoc nhan thanh `10 can xem / 0 sach`.
    expect(bar).not.toContain('10 Chương cần xem')
    expect(bar).not.toContain('0 Chương sạch')
    expect(state.importPreviewSelectedChapters.value?.needs_review_count).toBe(2)
    expect(state.importPreviewSelectedChapters.value?.clean_count).toBe(8)

    wrapper.unmount()
    state.resetImportPreview()
  })
})
