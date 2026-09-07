/**
 * Nhánh DANH SÁCH URL của lớp phủ **Xem trước lượt nhập** — Story 6.7, FR122, AD-15/AD-40/AD-41.
 *
 * ⚠️ Khuôn `importPreviewNormalized.test.ts`/`importPreviewCleanup.test.ts`: `config/project.ts`
 * là biên IPC, giả lập bằng `vi.mock`, không gọi `@tauri-apps/api` thật.
 *
 * 🔴 AC7 chép khuôn HIỆU SỐ BA MOCK (`importPreviewNormalized.test.ts:107-120`) — ở đây SÁU
 * mock (ba cũ + ba mới của Story 6.7: `startUrlImport`/`reloadUrlImportItem`/
 * `removeUrlImportItem`), không `not.toHaveBeenCalled()`: hai con số hiện đúng trên
 * `libraryImport.ts::pastedUrlCount` (JS thuần, không đọc `importPreviewState`), rồi kiểm
 * TỔNG số lời gọi qua cả sáu mock vẫn là 0.
 *
 * ⚠️ **Thứ tự bắt buộc trong mỗi ca**: `freshState()` TRƯỚC, cấu hình `mockResolvedValue` SAU
 * — `freshState()` tự `mockReset()` mọi mock.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { ImportEncodingPreview, UrlImportBatchWire, UrlImportItemWire } from '../../src/config/project'
import type { IpcError } from '../../src/i18n'

const previewTextMock = vi.fn()
const previewFileMock = vi.fn()
const confirmMock = vi.fn()
const startUrlImportMock = vi.fn()
const reloadUrlImportItemMock = vi.fn()
const removeUrlImportItemMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (text: string, sourceLang: string) => previewTextMock(text, sourceLang),
    previewImportEncodingFromFile: (path: string, sourceLang: string) => previewFileMock(path, sourceLang),
    confirmImportWithEncoding: (name: string, sourceLang: string, genre: string, encoding: string) =>
      confirmMock(name, sourceLang, genre, encoding),
    startUrlImport: (urls: string[], sourceLang: string) => startUrlImportMock(urls, sourceLang),
    reloadUrlImportItem: (index: number, sourceLang: string) => reloadUrlImportItemMock(index, sourceLang),
    removeUrlImportItem: (index: number, sourceLang: string) => removeUrlImportItemMock(index, sourceLang),
  }
})

/** Tổng số lời gọi qua CẢ SÁU mock — nguồn duy nhất cho mọi khẳng định "0 lời gọi IPC"/"đúng
 * MỘT lời gọi IPC" trong tệp này. */
function totalIpcCalls(): number {
  return (
    previewTextMock.mock.calls.length +
    previewFileMock.mock.calls.length +
    confirmMock.mock.calls.length +
    startUrlImportMock.mock.calls.length +
    reloadUrlImportItemMock.mock.calls.length +
    removeUrlImportItemMock.mock.calls.length
  )
}

async function freshState() {
  vi.resetModules()
  previewTextMock.mockReset()
  previewFileMock.mockReset()
  confirmMock.mockReset()
  startUrlImportMock.mockReset()
  reloadUrlImportItemMock.mockReset()
  removeUrlImportItemMock.mockReset()
  const state = await import('../../src/importPreviewState')
  const libraryImport = await import('../../src/modes/libraryImport')
  return { state, libraryImport }
}

async function freshOverlay() {
  const { state, libraryImport } = await freshState()
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { state, libraryImport, ImportPreviewOverlay }
}

function item(url: string, ok: boolean, error: IpcError | null = null): UrlImportItemWire {
  return { url, ok, error: ok ? null : (error ?? sampleError(url)) }
}

function sampleError(url: string): IpcError {
  return {
    code: 'import.web_item_failed',
    message_key: 'err.import.web_timeout',
    params: { url },
    retryable: false,
  }
}

/** Xem trước bảng mã tối giản — hợp lệ về HÌNH DẠNG, đủ cho các ca không xoáy vào tầng 1/3/4. */
function minimalPreview(chapterCount: number): ImportEncodingPreview {
  return {
    confidence: 'self_declared',
    selected_encoding: 'UTF-8',
    candidates: [],
    self_declared_normalized: { text: '', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
    self_declared_cleanup: {
      text: '',
      spans: [],
      rules: [],
      window_truncated: false,
      final_text: 'noi dung da boc',
    },
    self_declared_chapters: {
      chapter_count: chapterCount,
      chapters: Array.from({ length: chapterCount }, (_, i) => ({ ord: i + 1, title: null, length: 10 })),
    },
  }
}

/** `domainLogDomainCount` khớp mặc định số URL — mỗi URL của các ca trong tệp này là một
 * host phân biệt (Story 6.8, NFR19). Tham số RIÊNG cho các ca cần một con số khác. */
function batchAllOk(urls: string[], domainLogDomainCount = urls.length): UrlImportBatchWire {
  return {
    items: urls.map((u) => item(u, true)),
    encoding_preview: minimalPreview(urls.length),
    domain_log_domain_count: domainLogDomainCount,
  }
}

function batchWithOneBroken(
  urls: string[],
  brokenIndex: number,
  domainLogDomainCount = urls.length,
): UrlImportBatchWire {
  return {
    items: urls.map((u, i) => item(u, i !== brokenIndex)),
    encoding_preview: null,
    domain_log_domain_count: domainLogDomainCount,
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('libraryImport — hai con số trước khi bấm nút là 0 lời gọi IPC', () => {
  it('dán N link hiện đúng N/N qua pastedUrlCount, KHÔNG một lời gọi IPC nào', async () => {
    const { libraryImport } = await freshState()

    libraryImport.pastedUrls.value = 'https://a.example/1\nhttps://b.example/2\nhttps://c.example/3'

    expect(libraryImport.pastedUrlCount.value).toBe(3)
    expect(libraryImport.pastedUrlLines.value).toEqual([
      'https://a.example/1',
      'https://b.example/2',
      'https://c.example/3',
    ])
    expect(totalIpcCalls()).toBe(0)
  })

  it('dòng rỗng/toàn khoảng trắng bị bỏ khi đếm', async () => {
    const { libraryImport } = await freshState()

    libraryImport.pastedUrls.value = 'https://a.example/1\n   \n\nhttps://b.example/2\n\t'

    expect(libraryImport.pastedUrlCount.value).toBe(2)
    expect(totalIpcCalls()).toBe(0)
  })

  it('ô rỗng ⇒ đếm 0, và nút coi như khoá (submitPastedUrls no-op)', async () => {
    const { libraryImport } = await freshState()

    libraryImport.pastedUrls.value = '   \n  '
    expect(libraryImport.pastedUrlCount.value).toBe(0)

    await libraryImport.submitPastedUrls()
    expect(totalIpcCalls()).toBe(0)
  })
})

describe('importPreviewState — openImportPreviewFromUrls giữ đúng thứ tự đã dán', () => {
  it('items khớp ĐÚNG thứ tự N link, cùng nội dung batch trả về', async () => {
    const { state } = await freshState()
    const urls = ['https://a.example/1', 'https://b.example/2', 'https://c.example/3']
    startUrlImportMock.mockResolvedValue({ batch: batchAllOk(urls), error: null })

    await state.openImportPreviewFromUrls('Ten', 'en', '', urls)

    expect(state.importPreviewUrlItems.value.map((it) => it.url)).toEqual(urls)
    expect(state.importPreviewUrlItems.value.every((it) => it.ok)).toBe(true)
    expect(state.importPreview.value).not.toBeNull()
    expect(state.importPreviewSelectedChapters.value?.chapter_count).toBe(3)
    expect(startUrlImportMock).toHaveBeenCalledTimes(1)
    expect(startUrlImportMock).toHaveBeenCalledWith(urls, 'en')
  })

  it('còn MỘT mục hỏng ⇒ `importPreview` là `null` — nút xác nhận khoá', async () => {
    const { state } = await freshState()
    const urls = ['https://a.example/1', 'https://b.example/2', 'https://c.example/3']
    startUrlImportMock.mockResolvedValue({ batch: batchWithOneBroken(urls, 1), error: null })

    await state.openImportPreviewFromUrls('Ten', 'en', '', urls)

    expect(state.importPreviewUrlItems.value[1]?.ok).toBe(false)
    expect(state.importPreview.value).toBeNull()

    // `confirmImportPreview` phải NO-OP (0 lời gọi IPC thêm) khi `preview === null`.
    const before = totalIpcCalls()
    const result = await state.confirmImportPreview()
    expect(result).toEqual({ created: null, error: null })
    expect(totalIpcCalls()).toBe(before)
  })
})

describe('importPreviewState — bỏ một mục: N−1 link · N−1 Chương, hai số cùng giảm', () => {
  it('removeImportPreviewUrlItem hạ CẢ hai số cùng lúc', async () => {
    const { state } = await freshState()
    const urls = ['https://a.example/1', 'https://b.example/2', 'https://c.example/3']
    startUrlImportMock.mockResolvedValue({ batch: batchAllOk(urls), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls)
    expect(state.importPreviewUrlItems.value.length).toBe(3)
    expect(state.importPreviewSelectedChapters.value?.chapter_count).toBe(3)

    const remainingUrls = [urls[0]!, urls[2]!]
    removeUrlImportItemMock.mockResolvedValue({ batch: batchAllOk(remainingUrls), error: null })
    await state.removeImportPreviewUrlItem(1)

    expect(state.importPreviewUrlItems.value.length).toBe(2)
    expect(state.importPreviewUrlItems.value.map((it) => it.url)).toEqual(remainingUrls)
    expect(state.importPreviewSelectedChapters.value?.chapter_count).toBe(2)
    expect(removeUrlImportItemMock).toHaveBeenCalledTimes(1)
    // 0 lời gọi mạng — `removeUrlImportItem` không phải một trong ba mock "tải" (start/reload).
    expect(startUrlImportMock).toHaveBeenCalledTimes(1) // chỉ lượt mở ban đầu
    expect(reloadUrlImportItemMock).not.toHaveBeenCalled()
  })
})

describe('importPreviewState — tải lại một mục: đúng MỘT vòng IPC', () => {
  it('reloadImportPreviewUrlItem gọi ĐÚNG một lần, các mock khác không đổi', async () => {
    const { state } = await freshState()
    const urls = ['https://a.example/1', 'https://b.example/2']
    startUrlImportMock.mockResolvedValue({ batch: batchWithOneBroken(urls, 1), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls)
    expect(state.importPreviewUrlItems.value[1]?.ok).toBe(false)

    const before = {
      previewText: previewTextMock.mock.calls.length,
      previewFile: previewFileMock.mock.calls.length,
      confirm: confirmMock.mock.calls.length,
      start: startUrlImportMock.mock.calls.length,
      remove: removeUrlImportItemMock.mock.calls.length,
    }

    reloadUrlImportItemMock.mockResolvedValue({ batch: batchAllOk(urls), error: null })
    await state.reloadImportPreviewUrlItem(1)

    expect(reloadUrlImportItemMock).toHaveBeenCalledTimes(1)
    expect(reloadUrlImportItemMock).toHaveBeenCalledWith(1, 'en')
    expect(previewTextMock.mock.calls.length).toBe(before.previewText)
    expect(previewFileMock.mock.calls.length).toBe(before.previewFile)
    expect(confirmMock.mock.calls.length).toBe(before.confirm)
    expect(startUrlImportMock.mock.calls.length).toBe(before.start)
    expect(removeUrlImportItemMock.mock.calls.length).toBe(before.remove)

    expect(state.importPreviewUrlItems.value.every((it) => it.ok)).toBe(true)
    expect(state.importPreview.value).not.toBeNull()
  })
})

describe('ImportPreviewOverlay.vue — nhánh URL dựng được không vỡ, nút xác nhận khoá đúng lúc', () => {
  it('toàn bộ mục OK: danh sách hiện, tầng 1-4 hiện, nút xác nhận KHÔNG bị khoá', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const urls = ['https://a.example/1', 'https://b.example/2']
    startUrlImportMock.mockResolvedValue({ batch: batchAllOk(urls), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.ip-url-list').exists()).toBe(true)
    expect(wrapper.findAll('.ip-url-item').length).toBe(2)
    const confirmButton = wrapper.find('.ip-act-primary')
    expect(confirmButton.exists()).toBe(true)
    expect((confirmButton.element as HTMLButtonElement).disabled).toBe(false)

    wrapper.unmount()
  })

  it('còn một mục hỏng: danh sách hiện, tầng 1-4 KHÔNG hiện, nút xác nhận BỊ khoá', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const urls = ['https://a.example/1', 'https://b.example/2']
    startUrlImportMock.mockResolvedValue({ batch: batchWithOneBroken(urls, 1), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.ip-url-list').exists()).toBe(true)
    expect(wrapper.findAll('.ip-url-item').length).toBe(2)
    expect(wrapper.find('.ip-tier-1').exists()).toBe(false)
    const confirmButton = wrapper.find('.ip-act-primary')
    expect(confirmButton.exists()).toBe(true)
    expect((confirmButton.element as HTMLButtonElement).disabled).toBe(true)

    wrapper.unmount()
  })
})

describe('ImportPreviewOverlay.vue — P5 (vòng rà đối kháng bước 4): nút xác nhận phải xét importPreviewUrlImportBusy', () => {
  it('mọi mục OK nhưng đang tải-lại/bỏ một mục ⇒ nút xác nhận VẪN bị khoá (cửa đua)', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const urls = ['https://a.example/1', 'https://b.example/2']
    startUrlImportMock.mockResolvedValue({ batch: batchAllOk(urls), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    // Trước khi bỏ mục: mọi mục OK, `importPreview` khác null, nút phải MỞ.
    expect((wrapper.find('.ip-act-primary').element as HTMLButtonElement).disabled).toBe(false)

    // Giữ lời hứa `removeUrlImportItem` CHƯA giải quyết — mô phỏng đúng cửa đua: lượt bỏ một
    // mục (mọi mục còn lại vẫn OK) đang CHẠY cùng lúc người dùng có thể bấm xác nhận.
    let resolveRemove!: (v: { batch: UrlImportBatchWire; error: null }) => void
    removeUrlImportItemMock.mockReturnValue(
      new Promise((resolve) => {
        resolveRemove = resolve
      }),
    )
    const removePromise = state.removeImportPreviewUrlItem(1)
    await wrapper.vm.$nextTick()

    expect(state.importPreviewUrlImportBusy.value).toBe(true)
    expect(state.importPreview.value).not.toBeNull()
    expect((wrapper.find('.ip-act-primary').element as HTMLButtonElement).disabled).toBe(true)

    resolveRemove({ batch: batchAllOk([urls[0]!]), error: null })
    await removePromise
    await wrapper.vm.$nextTick()
    expect(state.importPreviewUrlImportBusy.value).toBe(false)
    expect((wrapper.find('.ip-act-primary').element as HTMLButtonElement).disabled).toBe(false)

    wrapper.unmount()
  })
})

describe('ImportPreviewOverlay.vue — Story 6.8: dòng tóm tắt nhật ký domain (NFR19)', () => {
  it('0 domain ⇒ chân màn KHÔNG có dòng tóm tắt (chưa gọi mạng lần nào)', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const urls = ['https://a.example/1']
    // `domain_log_domain_count: 0` mô phỏng đúng "danh sách toàn mục hỏng ngay từ InvalidUrl"
    // — 0 lời gọi mạng thật ra tới, 0 bản ghi nhật ký.
    startUrlImportMock.mockResolvedValue({ batch: batchAllOk(urls, 0), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls)

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.ip-domain-log-summary').exists()).toBe(false)

    wrapper.unmount()
  })

  it('N domain ⇒ dòng tóm tắt hiện đúng số, kể cả khi CÒN MỘT MỤC HỎNG (importPreview === null)', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const urls = ['https://a.example/1', 'https://b.example/2']
    // 🔴 Đúng ca AC dựng ra để bắt: một mục hỏng làm BỐN TẦNG biến mất
    // (`importPreview === null`), NHƯNG mạng đã bị gọi (mục tốt VẪN tải) — dòng tóm tắt phải
    // sống sót qua đúng ca này, không được sống BÊN TRONG khối bốn tầng.
    startUrlImportMock.mockResolvedValue({ batch: batchWithOneBroken(urls, 1, 2), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls)
    expect(state.importPreview.value).toBeNull() // tiền điều kiện: đúng "bốn tầng biến mất"

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const summary = wrapper.find('.ip-domain-log-summary')
    expect(summary.exists()).toBe(true)
    expect(summary.text()).toContain('2')

    const viewButton = wrapper.find('.ip-domain-log-view')
    expect(viewButton.exists()).toBe(true)

    wrapper.unmount()
  })
})

describe('importPreviewState — resetImportPreview vứt sạch state của nhánh URL', () => {
  it('huỷ lớp phủ xoá `urlImportItems` và trả `lastSubmittedFrom` về null', async () => {
    const { state } = await freshState()
    const urls = ['https://a.example/1']
    startUrlImportMock.mockResolvedValue({ batch: batchAllOk(urls), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', urls)
    expect(state.importPreviewUrlItems.value.length).toBe(1)

    state.cancelImportPreview()

    expect(state.importPreviewUrlItems.value).toEqual([])
    expect(state.importPreviewLastSubmittedFrom.value).toBeNull()
    expect(state.importPreviewIsOpen.value).toBe(false)
  })
})
