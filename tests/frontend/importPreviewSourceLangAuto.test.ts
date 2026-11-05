import { beforeEach, describe, expect, it, vi } from 'vitest'

const previewTextMock = vi.fn()
const startUrlMock = vi.fn()
const rebuildUrlMock = vi.fn()
const confirmMock = vi.fn()
const estimateMock = vi.fn()
const reloadUrlMock = vi.fn()
const removeUrlMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (...args: unknown[]) => previewTextMock(...args),
    startUrlImport: (...args: unknown[]) => startUrlMock(...args),
    rebuildUrlImportPreview: (...args: unknown[]) => rebuildUrlMock(...args),
    confirmImportWithEncoding: (...args: unknown[]) => confirmMock(...args),
    estimateImportGlossary: (...args: unknown[]) => estimateMock(...args),
    reloadUrlImportItem: (...args: unknown[]) => reloadUrlMock(...args),
    removeUrlImportItem: (...args: unknown[]) => removeUrlMock(...args),
    cancelImportGlossaryEstimate: async () => undefined,
  }
})

function wirePreview(lang: string, auto: boolean) {
  return {
    source_lang: lang,
    source_lang_auto: auto,
    confidence: 'self_declared',
    selected_encoding: 'UTF-8',
    candidates: [],
    self_declared_normalized: { text: 'x', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
    self_declared_cleanup: null,
    self_declared_chapters: null,
  }
}

const UNDETECTABLE = {
  code: 'import.source_lang_undetectable',
  message_key: 'err.import.source_lang_undetectable',
  params: {},
  retryable: false,
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

async function freshState() {
  vi.resetModules()
  for (const mock of [previewTextMock, startUrlMock, rebuildUrlMock, confirmMock, estimateMock, reloadUrlMock, removeUrlMock]) {
    mock.mockReset()
  }
  estimateMock.mockResolvedValue({ estimate: null, error: null })
  return import('../../src/importPreviewState')
}

describe('chip Tự nhận trong importPreviewState', () => {
  beforeEach(() => vi.clearAllMocks())

  it('lượt xem trước đầu gửi auto; mọi lệnh sau gửi giá trị cụ thể Rust trả về, không bao giờ auto', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: wirePreview('zh', true), error: null })
    confirmMock.mockResolvedValue({ created: null, error: null })

    await state.openImportPreviewFromText('Ten', 'auto', '', '他走进房间。', null)

    expect(previewTextMock.mock.calls[0]![1]).toBe('auto')
    expect(state.importPreviewPendingSourceLang.value).toBe('zh')
    expect(state.importPreviewSourceLangAuto.value).toBe(true)

    await sleep(400)
    expect(estimateMock).toHaveBeenCalled()
    expect(estimateMock.mock.calls.every((call) => call[0] === 'zh')).toBe(true)

    await state.confirmImportPreview()
    expect(confirmMock.mock.calls[0]![1]).toBe('zh')
  })

  it('ước Glossary không chạy khi ngôn ngữ vẫn là auto', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: wirePreview('auto', false), error: null })

    await state.openImportPreviewFromText('Ten', 'auto', '', 'x', null)
    await sleep(400)

    expect(estimateMock).not.toHaveBeenCalled()
  })

  it('đổi sang Tiếng Anh ở màn xem trước chạy lại với en và tắt nhãn tự phát hiện', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValueOnce({ preview: wirePreview('zh', true), error: null })
    await state.openImportPreviewFromText('Ten', 'auto', '', 'abc', null)

    previewTextMock.mockResolvedValueOnce({ preview: wirePreview('en', false), error: null })
    await state.setImportPreviewSourceLang('en')

    expect(previewTextMock.mock.calls.at(-1)![1]).toBe('en')
    expect(state.importPreviewPendingSourceLang.value).toBe('en')
    expect(state.importPreviewSourceLangAuto.value).toBe(false)
  })

  it('đổi ngôn ngữ thất bại: nhãn và cờ tự phát hiện quay về giá trị cũ', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValueOnce({ preview: wirePreview('zh', true), error: null })
    await state.openImportPreviewFromText('Ten', 'auto', '', 'abc', null)

    previewTextMock.mockResolvedValueOnce({ preview: null, error: UNDETECTABLE })
    await state.setImportPreviewSourceLang('en')

    expect(state.importPreviewPendingSourceLang.value).toBe('zh')
    expect(state.importPreviewSourceLangAuto.value).toBe(true)
  })

  it('đích là Tác phẩm có sẵn: chip đổi ngôn ngữ không chạy', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: wirePreview('en', false), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'abc', 'work-1')
    const calls = previewTextMock.mock.calls.length

    await state.setImportPreviewSourceLang('zh')

    expect(previewTextMock.mock.calls.length).toBe(calls)
    expect(state.importPreviewPendingSourceLang.value).toBe('en')
  })

  it('không nhận ra ngôn ngữ: không mở màn xem trước, lỗi đi ra cho form, đọc một lần', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: null, error: UNDETECTABLE })

    await state.openImportPreviewFromText('Ten', 'auto', '', '123', null)

    expect(state.importPreviewIsOpen.value).toBe(false)
    expect(state.takeImportPreviewUndetectableLangError()).toEqual(UNDETECTABLE)
    expect(state.takeImportPreviewUndetectableLangError()).toBeNull()
  })

  it('đường URL: đổi ngôn ngữ dựng lại từ nội dung đã tải, không gọi tải lại', async () => {
    const state = await freshState()
    const batch = (lang: string, auto: boolean) => ({
      items: [{ url: 'https://example.com/a', ok: true, error: null }],
      encoding_preview: wirePreview(lang, auto),
      domain_log_domain_count: 1,
      duplicate_urls_dropped: 0,
    })
    startUrlMock.mockResolvedValue({ batch: batch('zh', true), error: null })
    await state.openImportPreviewFromUrls('Ten', 'auto', '', ['https://example.com/a'], null)
    expect(state.importPreviewPendingSourceLang.value).toBe('zh')

    rebuildUrlMock.mockResolvedValue({ batch: batch('en', false), error: null })
    await state.setImportPreviewSourceLang('en')

    expect(rebuildUrlMock).toHaveBeenCalledWith('en')
    expect(startUrlMock).toHaveBeenCalledTimes(1)
    expect(reloadUrlMock).not.toHaveBeenCalled()
    expect(state.importPreviewSourceLangAuto.value).toBe(false)
  })
})
