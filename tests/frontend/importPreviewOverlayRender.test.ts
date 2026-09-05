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
import type { EncodingCandidateWire, ImportEncodingPreview } from '../../src/config/project'

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
    ...over,
  }
}

async function freshOverlay() {
  vi.resetModules()
  previewTextMock.mockReset()

  const state = await import('../../src/importPreviewState')
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { state, ImportPreviewOverlay }
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
    await state.openImportPreviewFromText('Ten', 'en', '', 'text')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    expect(wrapper.find('.ip-confidence-chip').text()).toContain(expectedText)

    wrapper.unmount()
    state.resetImportPreview()
  })

  // 🔵 SỬA 2026-09-05 (Story 6.5) — "tầng 2/3 rỗng" đã HẾT ĐÚNG cho tầng 3: nó nay CÓ THÂN
  // khi ứng viên mang `cleanup` (mặc định của `candidate()` từ story này). Chỉ tầng 2 còn
  // rỗng — ca dành cho tầng 3 rỗng (ứng viên "không ra chữ") đứng riêng ngay dưới.
  it('tầng 2 rỗng dựng đúng lý do (story_6_9)', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'text')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    const reasons = wrapper.findAll('.ip-tier-empty-reason')
    expect(reasons).toHaveLength(1)
    expect(reasons[0]?.text()).toContain('Story 6.9')

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
    await state.openImportPreviewFromText('Ten', 'en', '', 'text')

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
    await state.openImportPreviewFromText('Ten', 'en', '', 'text')

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
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'van ban dan tay')

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
    await state.openImportPreviewFromText('Ten', 'en', '', '   ')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.find('.ip-normalized-text').exists()).toBe(false)
    expect(wrapper.text()).toContain('Không có gì để hiện')

    wrapper.unmount()
    state.resetImportPreview()
  })
})
