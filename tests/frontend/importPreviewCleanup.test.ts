/**
 * Tầng làm sạch (tầng 3) của lớp phủ **Xem trước lượt nhập** — Story 6.5, FR124, AD-18.
 *
 * ⚠️ Khuôn `importPreviewNormalized.test.ts`/`importPreviewOverlayRender.test.ts`:
 * `config/project.ts` là biên IPC, giả lập bằng `vi.mock`, không gọi `@tauri-apps/api` thật.
 *
 * ⚠️ **Thứ tự bắt buộc trong mỗi ca**: `freshState()`/`freshOverlay()` TRƯỚC, cấu hình
 * `mockResolvedValue` SAU — cả hai tự `mockReset()` mọi mock.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type {
  CleanupPreviewWire,
  CleanupRuleReportWire,
  EncodingCandidateWire,
  ImportEncodingPreview,
} from '../../src/config/project'

const previewTextMock = vi.fn()
const previewFileMock = vi.fn()
const confirmMock = vi.fn()
const cleanupAddRuleMock = vi.fn()
const cleanupEditRuleMock = vi.fn()
const cleanupDeleteRuleMock = vi.fn()
const cleanupSetEnabledMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (text: string, sourceLang: string) => previewTextMock(text, sourceLang),
    previewImportEncodingFromFile: (path: string, sourceLang: string) => previewFileMock(path, sourceLang),
    confirmImportWithEncoding: (name: string, sourceLang: string, genre: string, encoding: string) =>
      confirmMock(name, sourceLang, genre, encoding),
    cleanupAddRule: (tier: string, pattern: string, kind: string) => cleanupAddRuleMock(tier, pattern, kind),
    cleanupEditRule: (tier: string, id: number, pattern: string, kind: string) =>
      cleanupEditRuleMock(tier, id, pattern, kind),
    cleanupDeleteRule: (tier: string, id: number) => cleanupDeleteRuleMock(tier, id),
    cleanupSetEnabled: (tier: string, id: number, enabled: boolean) => cleanupSetEnabledMock(tier, id, enabled),
  }
})

async function freshState() {
  vi.resetModules()
  previewTextMock.mockReset()
  previewFileMock.mockReset()
  confirmMock.mockReset()
  cleanupAddRuleMock.mockReset()
  cleanupEditRuleMock.mockReset()
  cleanupDeleteRuleMock.mockReset()
  cleanupSetEnabledMock.mockReset()
  return import('../../src/importPreviewState')
}

async function freshOverlay() {
  const state = await freshState()
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { state, ImportPreviewOverlay }
}

function rule(over: Partial<CleanupRuleReportWire> = {}): CleanupRuleReportWire {
  return {
    tier: 'global',
    id: 1,
    pattern: 'quang cao',
    kind: 'literal',
    enabled: true,
    count_in_chapter: 2,
    count_in_import: 2,
    ...over,
  }
}

function cleanup(over: Partial<CleanupPreviewWire> = {}): CleanupPreviewWire {
  return {
    text: 'dau quang cao cuoi',
    spans: [{ tier: 'global', id: 1, start: 4, end: 14 }],
    rules: [rule()],
    window_truncated: false,
    final_text: 'dau  cuoi',
    ...over,
  }
}

function candidate(over: Partial<EncodingCandidateWire> = {}): EncodingCandidateWire {
  return {
    label: 'UTF-8',
    encoding: 'UTF-8',
    preview: 'dau quang cao cuoi',
    normalized: { text: 'dau quang cao cuoi', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
    cleanup: cleanup(),
    ...over,
  }
}

function preview(over: Partial<ImportEncodingPreview> = {}): ImportEncodingPreview {
  return {
    confidence: 'high',
    selected_encoding: 'UTF-8',
    candidates: [candidate()],
    self_declared_normalized: null,
    self_declared_cleanup: null,
    ...over,
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('importPreviewState — importPreviewSelectedCleanup', () => {
  it('mặc định là khối làm sạch của ứng viên Rust đã chọn', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi')

    expect(state.importPreviewSelectedCleanup.value).toEqual(cleanup())
  })

  it('đổi ứng viên đổi NGAY khối làm sạch, 0 lời gọi IPC thêm', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({
      preview: preview({
        selected_encoding: 'UTF-8',
        candidates: [
          candidate({ encoding: 'UTF-8' }),
          candidate({
            label: 'GBK',
            encoding: 'GBK',
            preview: 'khac han',
            normalized: { text: 'khac han', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
            cleanup: cleanup({ text: 'khac han', spans: [], rules: [], final_text: 'khac han' }),
          }),
        ],
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi')

    const ipcCallsBefore =
      previewTextMock.mock.calls.length + previewFileMock.mock.calls.length + confirmMock.mock.calls.length

    state.selectImportPreviewCandidate('GBK')

    expect(state.importPreviewSelectedCleanup.value?.text).toBe('khac han')
    expect(state.importPreviewSelectedCleanup.value?.spans).toHaveLength(0)

    const ipcCallsAfter =
      previewTextMock.mock.calls.length + previewFileMock.mock.calls.length + confirmMock.mock.calls.length
    expect(ipcCallsAfter).toBe(ipcCallsBefore)
  })

  it('ứng viên "không ra chữ" (`cleanup: null`) đọc ra `null`', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({
      preview: preview({ candidates: [candidate({ preview: null, normalized: null, cleanup: null })] }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    expect(state.importPreviewSelectedCleanup.value).toBeNull()
  })

  it('đường DÁN VĂN BẢN TAY (0 ứng viên) đọc `self_declared_cleanup`, KHÔNG rơi về null', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({
      preview: preview({
        confidence: 'self_declared',
        candidates: [],
        self_declared_cleanup: cleanup({ text: 'van ban dan tay' }),
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'van ban dan tay')

    expect(state.importPreviewSelectedCandidate.value).toBeNull()
    expect(state.importPreviewSelectedCleanup.value?.text).toBe('van ban dan tay')
  })
})

describe('importPreviewState — bốn hành động CRUD luật làm sạch dựng lại xem trước', () => {
  it('thêm luật THÀNH CÔNG ⇒ gọi cleanupAddRule rồi TẢI LẠI xem trước bằng ĐÚNG văn bản đã dán', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi')
    previewTextMock.mockClear()

    cleanupAddRuleMock.mockResolvedValue({ ok: true, error: null })
    const reloaded = preview({ candidates: [candidate({ cleanup: cleanup({ rules: [rule(), rule({ id: 2 })] }) })] })
    previewTextMock.mockResolvedValue({ preview: reloaded, error: null })

    await state.addImportPreviewCleanupRule('global', 'quang cao', 'literal')

    expect(cleanupAddRuleMock).toHaveBeenCalledWith('global', 'quang cao', 'literal')
    // Tải lại phải dùng ĐÚNG văn bản đã dán ở lượt mở — JS không đọc lại được từ
    // `PendingImportSourceState` phía Rust (chỉ sống ở đó), nên phải tự giữ một bản.
    expect(previewTextMock).toHaveBeenCalledWith('dau quang cao cuoi', 'en')
    expect(state.importPreview.value?.candidates[0]?.cleanup?.rules).toHaveLength(2)
    expect(state.importPreviewCleanupActionError.value).toBeNull()
  })

  it('giữ NGUYÊN ứng viên bảng mã người dùng đã CHỌN TAY (khác mặc định) sau một lượt CRUD — gỡ nhánh giữ lựa chọn thì ca này đỏ, và hậu quả thật là ghi SAI bảng mã xuống đĩa', async () => {
    const state = await freshState()
    const twoCandidatesPreview = preview({
      selected_encoding: 'UTF-8', // mặc định Rust chọn
      candidates: [
        candidate({ encoding: 'UTF-8' }),
        candidate({
          encoding: 'GBK',
          preview: 'khac han',
          normalized: { text: 'khac han', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
          cleanup: cleanup({ text: 'khac han', spans: [], rules: [], final_text: 'khac han' }),
        }),
      ],
    })
    previewTextMock.mockResolvedValue({ preview: twoCandidatesPreview, error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi')

    // Người dùng CHỌN TAY một ứng viên KHÁC mặc định (GBK, khác 'UTF-8' mà Rust chọn).
    state.selectImportPreviewCandidate('GBK')
    expect(state.importPreviewSelectedEncoding.value).toBe('GBK')
    previewTextMock.mockClear()

    // CRUD một luật (thêm) — lượt tải lại trả về CÙNG dải hai ứng viên đó.
    cleanupAddRuleMock.mockResolvedValue({ ok: true, error: null })
    previewTextMock.mockResolvedValue({ preview: twoCandidatesPreview, error: null })
    await state.addImportPreviewCleanupRule('global', 'mau moi', 'literal')

    // Lựa chọn TAY phải còn nguyên — KHÔNG rơi về 'UTF-8' (mặc định của Rust) chỉ vì vừa
    // tải lại. Nếu nhánh giữ lựa chọn (`reloadImportPreviewAfterRuleChange`) bị gỡ, hàm sẽ
    // luôn gán `result.preview.selected_encoding`, và ca này đỏ ngay ở assert dưới.
    expect(state.importPreviewSelectedEncoding.value).toBe('GBK')
    expect(state.importPreviewSelectedCandidate.value?.encoding).toBe('GBK')
  })

  it('thêm luật TRƯỢT (mẫu rỗng/regex hỏng) ⇒ hiện lỗi, KHÔNG tải lại xem trước', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi')
    previewTextMock.mockClear()

    const err = { code: 'cleanup.invalid_regex', message_key: 'err.cleanup.invalid_regex', params: {}, retryable: false }
    cleanupAddRuleMock.mockResolvedValue({ ok: false, error: err })

    await state.addImportPreviewCleanupRule('global', '[unclosed', 'regex')

    expect(state.importPreviewCleanupActionError.value).toEqual(err)
    expect(previewTextMock).not.toHaveBeenCalled()
  })

  it('sửa luật THÀNH CÔNG ⇒ gọi cleanupEditRule rồi tải lại', async () => {
    const state = await freshState()
    previewFileMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromFile('Ten', 'en', '', '/tmp/x.txt')
    previewFileMock.mockClear()

    cleanupEditRuleMock.mockResolvedValue({ ok: true, error: null })
    previewFileMock.mockResolvedValue({ preview: preview(), error: null })

    await state.editImportPreviewCleanupRule('global', 1, 'mau moi', 'literal')

    expect(cleanupEditRuleMock).toHaveBeenCalledWith('global', 1, 'mau moi', 'literal')
    expect(previewFileMock).toHaveBeenCalledWith('/tmp/x.txt', 'en')
  })

  it('xoá luật là HAI NHỊP — nhịp một chỉ đổi trạng thái, KHÔNG gọi IPC; nhịp hai (gọi lại đúng luật) mới ghi thật rồi tải lại', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    previewTextMock.mockClear()

    cleanupDeleteRuleMock.mockResolvedValue({ ok: true, error: null })
    previewTextMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ cleanup: cleanup({ rules: [] }) })] }), error: null })

    // Nhịp MỘT — chỉ đổi trạng thái "chờ xác nhận", 0 lời gọi IPC.
    await state.deleteImportPreviewCleanupRule('global', 1)
    expect(cleanupDeleteRuleMock).not.toHaveBeenCalled()
    expect(previewTextMock).not.toHaveBeenCalled()

    // Nhịp HAI — gọi lại ĐÚNG (tier, id) đó ⇒ ghi thật rồi tải lại.
    await state.deleteImportPreviewCleanupRule('global', 1)
    expect(cleanupDeleteRuleMock).toHaveBeenCalledWith('global', 1)
    expect(state.importPreviewSelectedCleanup.value?.rules).toHaveLength(0)
  })

  it('xoá luật — gọi một (tier, id) KHÁC ở nhịp một huỷ luôn nhịp chờ của luật trước, không ghi gì', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    previewTextMock.mockClear()
    cleanupDeleteRuleMock.mockResolvedValue({ ok: true, error: null })

    await state.deleteImportPreviewCleanupRule('global', 1) // nhịp một, luật #1
    await state.deleteImportPreviewCleanupRule('global', 2) // đổi ý ⇒ nhịp một MỚI, luật #2
    expect(cleanupDeleteRuleMock).not.toHaveBeenCalled()

    // Gọi lại luật #1 (đã bị huỷ nhịp chờ) ⇒ lại là một nhịp MỘT mới, không ghi ngay.
    await state.deleteImportPreviewCleanupRule('global', 1)
    expect(cleanupDeleteRuleMock).not.toHaveBeenCalled()
  })

  it('bật/tắt luật là MỘT LƯỢT GHI THẬT (§Always spec 6.5) — gọi cleanupSetEnabled rồi tải lại', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    previewTextMock.mockClear()

    cleanupSetEnabledMock.mockResolvedValue({ ok: true, error: null })
    previewTextMock.mockResolvedValue({
      preview: preview({ candidates: [candidate({ cleanup: cleanup({ spans: [], rules: [rule({ enabled: false })] }) })] }),
      error: null,
    })

    await state.toggleImportPreviewCleanupRule('global', 1, false)

    expect(cleanupSetEnabledMock).toHaveBeenCalledWith('global', 1, false)
    expect(state.importPreviewSelectedCleanup.value?.spans).toHaveLength(0)
    expect(state.importPreviewSelectedCleanup.value?.rules[0]?.enabled).toBe(false)
  })
})

describe('ImportPreviewOverlay.vue — tầng 3 dựng đúng span/luật, và bốn hành động gọi đúng adapter', () => {
  it('dựng văn bản đánh dấu — mảnh bị luật BẬT phủ mang class gạch ngang, mảnh còn lại thì không', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    const struck = wrapper.findAll('.ip-cleanup-struck')
    expect(struck).toHaveLength(1)
    expect(struck[0]?.text()).toBe('quang cao')
    expect(wrapper.find('.ip-cleanup-text').text()).toBe('dau quang cao cuoi')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('danh sách luật hiện đúng mẫu, nhãn tầng, và hai số đếm', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    const ruleRow = wrapper.find('.ip-cleanup-rule')
    expect(ruleRow.find('.ip-cleanup-pattern').text()).toBe('quang cao')
    expect(ruleRow.find('.ip-cleanup-tier').text()).toBe('Toàn cục')
    expect(ruleRow.find('.ip-cleanup-meta').text()).toContain('2')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('tắt tick của một luật gọi `cleanupSetEnabled(tier, id, false)`', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi')
    cleanupSetEnabledMock.mockResolvedValue({ ok: true, error: null })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.find('.ip-cleanup-tick').setValue(false)

    expect(cleanupSetEnabledMock).toHaveBeenCalledWith('global', 1, false)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('nộp form "thêm luật mới" gọi `cleanupAddRule` với đúng mẫu/hình dạng/tầng đã chọn', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi')
    cleanupAddRuleMock.mockResolvedValue({ ok: true, error: null })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.find('.ip-cleanup-add-pattern').setValue('mau moi')
    await wrapper.find('.ip-cleanup-add').trigger('submit')

    expect(cleanupAddRuleMock).toHaveBeenCalledWith('global', 'mau moi', 'literal')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('nộp form xoá của một luật là HAI NHỊP — nộp lần một chỉ đổi nhãn nút, nộp lần hai (cùng hàng) mới gọi `cleanupDeleteRule(tier, id)`', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi')
    cleanupDeleteRuleMock.mockResolvedValue({ ok: true, error: null })
    previewTextMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ cleanup: cleanup({ rules: [] }) })] }), error: null })

    // Hai `<form class="ip-cleanup-action-form">` cùng tồn tại trên một hàng luật (sửa · xoá,
    // xem template) — form THỨ HAI là "xoá" theo đúng thứ tự khai trong `ImportPreviewOverlay.vue`.
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    const forms = wrapper.findAll('.ip-cleanup-action-form')
    expect(forms).toHaveLength(2)

    // Nộp lần MỘT — chỉ chuyển hàng sang trạng thái "chờ xác nhận", KHÔNG gọi IPC.
    await forms[1]?.trigger('submit')
    expect(cleanupDeleteRuleMock).not.toHaveBeenCalled()
    expect(wrapper.find('.ip-cleanup-delete-pending').exists()).toBe(true)

    // Nộp lần HAI, CÙNG hàng — xác nhận: ghi thật.
    await forms[1]?.trigger('submit')
    expect(cleanupDeleteRuleMock).toHaveBeenCalledWith('global', 1)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('bấm "Sửa" mở form nội tuyến; nộp "Lưu" gọi `cleanupEditRule` với văn bản/hình dạng vừa sửa', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi')

    // Form ĐẦU TIÊN trong hai `.ip-cleanup-action-form` của hàng là "sửa" (xem template).
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    const forms = wrapper.findAll('.ip-cleanup-action-form')
    await forms[0]?.trigger('submit')

    expect(wrapper.find('.ip-cleanup-edit-pattern').exists()).toBe(true)

    cleanupEditRuleMock.mockResolvedValue({ ok: true, error: null })
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await wrapper.find('.ip-cleanup-edit-pattern').setValue('mau da sua')
    await wrapper.find('.ip-cleanup-edit-form').trigger('submit')

    expect(cleanupEditRuleMock).toHaveBeenCalledWith('global', 1, 'mau da sua', 'literal')

    wrapper.unmount()
    state.resetImportPreview()
  })
})
