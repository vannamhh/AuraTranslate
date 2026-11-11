import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'
import type { BlockWire, ImportEncodingPreview, UrlImportBatchWire } from '../../src/config/project'
import type { QuickBlockEventWire } from '../../src/config/quickBatch'

const startUrlImportMock = vi.fn()
const previewChapterDetailMock = vi.fn()
const lookupManyMock = vi.fn()
const blocksMock = vi.fn()
const cancelMock = vi.fn()
const quickAiConfigGetMock = vi.fn()
const retranslateFlags: boolean[] = []

vi.mock('../../src/config/project', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/project')>()),
  startUrlImport: (urls: string[], sourceLang: string) => startUrlImportMock(urls, sourceLang),
  previewChapterDetail: (...args: unknown[]) => previewChapterDetailMock(...args),
}))

vi.mock('../../src/config/quickBatch', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/quickBatch')>()),
  quickTranslationLookupMany: (texts: string[]) => lookupManyMock(texts),
  translateQuickBlocks: (
    texts: string[],
    runId: number,
    onEvent: (event: QuickBlockEventWire) => void,
    retranslate = false,
  ) => {
    retranslateFlags.push(retranslate)
    return blocksMock(texts, runId, onEvent)
  },
}))

vi.mock('../../src/config/glossaryContextReference', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/glossaryContextReference')>()),
  cancelContextReference: (runId: number) => cancelMock(runId),
}))

vi.mock('../../src/config/quickAiConfig', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/quickAiConfig')>()),
  quickAiConfigGet: () => quickAiConfigGetMock(),
}))

const URLS = ['https://a.example/1']

function block(text: string, over: Partial<BlockWire> = {}): BlockWire {
  return { body: { kind: 'paragraph', text }, kept: true, confirmed: true, cleanup_spans: [], ...over }
}

const ORIGIN = {
  author: null,
  site_name: null,
  url: null,
  published_at: null,
  author_confirmed: false,
  site_name_confirmed: false,
  url_confirmed: false,
  published_at_confirmed: false,
}

function batch(blocks: BlockWire[], chapterCount = 1): UrlImportBatchWire {
  const preview: ImportEncodingPreview = {
    source_lang: 'zh',
    source_lang_auto: false,
    confidence: 'self_declared',
    selected_encoding: 'UTF-8',
    candidates: [
      {
        label: 'UTF-8',
        encoding: 'UTF-8',
        preview: 'x',
        normalized: { text: 'x', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        cleanup: { text: 'x', spans: [], rules: [], window_truncated: false, final_text: 'x' },
        chapters: {
          chapter_count: chapterCount,
          chapters: Array.from({ length: chapterCount }, (_, i) => ({
            ord: i + 1,
            title: null,
            length: 8,
            cleanup_match_count: 0,
            joined_line_count_in_chapter: null,
            needs_review: false,
            review_causes: [],
            chapter_origin: ORIGIN,
            source_file: null,
          })),
          broken_item_count: 0,
          needs_review_count: 0,
          clean_count: chapterCount,
          any_signal_participated: false,
          already_chaptered: false,
        },
        blocks: { blocks },
      },
    ],
    self_declared_normalized: null,
    self_declared_cleanup: null,
    self_declared_chapters: null,
  }
  return {
    items: URLS.map((url) => ({ url, ok: true, error: null })),
    encoding_preview: preview,
    domain_log_domain_count: 1,
    duplicate_urls_dropped: 0,
  }
}

const PAGE = [block('甲'), block('乙'), block('丙')]
const miss = { state: 'miss' }
const doneOutcome = (over: Record<string, unknown> = {}) => ({
  value: { state: 'done', model: 'm-1', usage: null, missing: [], deferred: 0, ...over },
  error: null,
})

async function fresh(deps: Partial<CommandDeps> = {}) {
  vi.resetModules()
  for (const m of [startUrlImportMock, previewChapterDetailMock, lookupManyMock, blocksMock, cancelMock, quickAiConfigGetMock]) {
    m.mockReset()
  }
  retranslateFlags.length = 0
  lookupManyMock.mockImplementation(async (texts: string[]) => ({ value: texts.map(() => miss), error: null }))
  quickAiConfigGetMock.mockResolvedValue({
    config: { fields: [], source: 'dedicated', key_configured: null, ready: true, not_ready: [] },
    error: null,
  })
  const commands = await import('../../src/commands')
  commands.installCommands(deps as CommandDeps)
  const state = await import('../../src/importPreviewState')
  const run = await import('../../src/quickTranslateRun')
  const Overlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { state, run, Overlay, commands }
}

type Fresh = Awaited<ReturnType<typeof fresh>>

async function open(state: Fresh['state'], blocks: BlockWire[] = PAGE, chapterCount = 1) {
  startUrlImportMock.mockResolvedValue({ batch: batch(blocks, chapterCount), error: null })
  await state.openImportPreviewFromUrls('Ten', 'zh', '', URLS, null)
  await flushPromises()
}

function see(state: Fresh['state'], ...indices: number[]) {
  state.setImportPreviewVisibleBlocks(indices)
  return flushPromises()
}

/** A batch call that the test finishes by hand. */
function pendingBatch() {
  let emit: (event: QuickBlockEventWire) => void = () => undefined
  let finish: (value: unknown) => void = () => undefined
  blocksMock.mockImplementation(
    (_texts: string[], _runId: number, onEvent: (event: QuickBlockEventWire) => void) =>
      new Promise((resolve) => {
        emit = onEvent
        finish = resolve
      }),
  )
  return { emit: (event: QuickBlockEventWire) => emit(event), finish: (value: unknown) => finish(value) }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('khối đang thấy, tra kho khi hiện', () => {
  it('tra kho đúng khối chữ đang thấy, trúng thì hiện ngay và không gọi AI', async () => {
    const { state } = await fresh()
    lookupManyMock.mockResolvedValue({ value: [{ state: 'hit', text: 'Giáp', model: 'm-1' }, miss], error: null })
    await open(state, [block('甲'), block('乙'), block('', { body: { kind: 'image', src: 'x', alt: null } })])

    await see(state, 0, 1, 2)

    expect(lookupManyMock).toHaveBeenCalledWith(['甲', '乙'])
    expect(state.importPreviewBlockTranslationAt(0)).toEqual({ kind: 'done', text: 'Giáp', model: 'm-1' })
    expect(state.importPreviewBlockTranslationAt(1)).toBeNull()
    expect(state.importPreviewBlockTranslationAt(2)).toBeNull()
    expect(blocksMock).not.toHaveBeenCalled()
  })

  it('khối đã tra không tra lại khi hiện lần nữa', async () => {
    const { state } = await fresh()
    await open(state)
    await see(state, 0)
    await see(state, 0, 1)

    expect(lookupManyMock).toHaveBeenCalledTimes(2)
    expect(lookupManyMock).toHaveBeenLastCalledWith(['乙'])
  })

  it('kho không mở được ghi nhận trạng thái riêng', async () => {
    const { state } = await fresh()
    lookupManyMock.mockResolvedValue({ value: [{ state: 'store_unavailable' }], error: null })
    await open(state)
    await see(state, 0)

    expect(state.importPreviewQuickBatchNote.value).toBe('store_unavailable')
  })

  it('đóng rồi mở lại màn xem trước: câu đã lưu hiện từ kho, 0 lời gọi dịch', async () => {
    const { state } = await fresh()
    await open(state)
    await see(state, 0)
    state.cancelImportPreview()
    lookupManyMock.mockResolvedValue({ value: [{ state: 'hit', text: 'Giáp', model: 'm-1' }], error: null })
    await open(state)
    await see(state, 0)

    expect(state.importPreviewBlockTranslationAt(0)).toEqual({ kind: 'done', text: 'Giáp', model: 'm-1' })
    expect(blocksMock).not.toHaveBeenCalled()
  })
})

describe('lô khối', () => {
  it('gửi khối chưa có theo thứ tự trang, chữ chảy dần rồi giữ lại cùng usage', async () => {
    const { state } = await fresh()
    lookupManyMock.mockResolvedValue({ value: [{ state: 'hit', text: 'Giáp', model: 'm-1' }, miss, miss], error: null })
    await open(state)
    await see(state, 2, 0, 1)
    const call = pendingBatch()

    const running = state.translateImportPreviewVisibleBlocks()
    expect(blocksMock).toHaveBeenCalledWith(['乙', '丙'], expect.any(Number), expect.any(Function))
    expect(state.importPreviewQuickBatchRunning.value).toBe(true)
    expect(state.importPreviewQuickBatchPendingCount.value).toBe(2)

    call.emit({ kind: 'started', indices: [0, 1] })
    call.emit({ kind: 'token', index: 0, text: 'Ất' })
    call.emit({ kind: 'token', index: 0, text: ' x' })
    expect(state.importPreviewBlockTranslationAt(1)).toEqual({ kind: 'running', text: 'Ất x' })
    call.emit({ kind: 'received', index: 0, text: 'Ất', model: 'm-1', saved: true })
    expect(state.importPreviewQuickBatchPendingCount.value).toBe(1)
    call.emit({ kind: 'received', index: 1, text: 'Bính', model: 'm-1', saved: true })
    const usage = { prompt_tokens: 1, completion_tokens: 2, total_tokens: 3, cost_usd: null }
    call.finish(doneOutcome({ usage }))
    await running

    expect(state.importPreviewQuickBatchRunning.value).toBe(false)
    expect(state.importPreviewBlockTranslationAt(1)).toEqual({ kind: 'done', text: 'Ất', model: 'm-1' })
    expect(state.importPreviewBlockTranslationAt(2)).toEqual({ kind: 'done', text: 'Bính', model: 'm-1' })
    expect(state.importPreviewQuickBatchUsage.value).toEqual({ usage, model: 'm-1' })
  })

  it('hai khối cùng chữ chỉ gửi một lần và cả hai hiện câu dịch', async () => {
    const { state } = await fresh()
    await open(state, [block('甲'), block(' 甲 '), block('乙')])
    await see(state, 0, 1, 2)
    const call = pendingBatch()

    const running = state.translateImportPreviewVisibleBlocks()
    expect(blocksMock.mock.calls[0]?.[0]).toEqual(['甲', '乙'])
    call.emit({ kind: 'received', index: 0, text: 'Giáp', model: 'm-1', saved: true })
    call.emit({ kind: 'received', index: 1, text: 'Ất', model: 'm-1', saved: true })
    call.finish(doneOutcome())
    await running

    expect(state.importPreviewBlockTranslationAt(0)?.kind).toBe('done')
    expect(state.importPreviewBlockTranslationAt(1)).toEqual(state.importPreviewBlockTranslationAt(0))
  })

  it('Huỷ giữa lô: khối đã đóng ở lại, khối còn lại về nút, lệnh huỷ đi đúng run_id, sự kiện muộn bị bỏ', async () => {
    const { state, run } = await fresh()
    await open(state)
    await see(state, 0, 1, 2)
    const call = pendingBatch()

    const running = state.translateImportPreviewVisibleBlocks()
    const runId = blocksMock.mock.calls[0]?.[1] as number
    call.emit({ kind: 'started', indices: [0, 1, 2] })
    call.emit({ kind: 'received', index: 0, text: 'Giáp', model: 'm-1', saved: true })
    call.emit({ kind: 'received', index: 1, text: 'Ất', model: 'm-1', saved: true })
    call.emit({ kind: 'token', index: 2, text: 'Bí' })

    state.cancelImportPreviewQuickBatch()
    call.emit({ kind: 'received', index: 2, text: 'Bính', model: 'm-1', saved: true })
    call.finish({ value: { state: 'cancelled' }, error: null })
    await running

    expect(cancelMock).toHaveBeenCalledWith(runId)
    expect(run.quickRunCurrent.value).toBeNull()
    expect(state.importPreviewQuickBatchRunning.value).toBe(false)
    expect(state.importPreviewBlockTranslationAt(0)?.kind).toBe('done')
    expect(state.importPreviewBlockTranslationAt(1)?.kind).toBe('done')
    expect(state.importPreviewBlockTranslationAt(2)).toBeNull()
  })

  it('khối thiếu hiện Thiếu kết quả, không lưu, không tự gửi lại; Dịch thử gửi riêng khối đó', async () => {
    const { state } = await fresh()
    await open(state)
    await see(state, 0, 1, 2)
    let call = pendingBatch()

    const running = state.translateImportPreviewVisibleBlocks()
    call.emit({ kind: 'started', indices: [0, 1, 2] })
    call.emit({ kind: 'received', index: 0, text: 'Giáp', model: 'm-1', saved: true })
    call.emit({ kind: 'received', index: 1, text: 'Ất', model: 'm-1', saved: true })
    call.finish(doneOutcome({ missing: [{ index: 2, reason: 'unclosed' }] }))
    await running

    expect(blocksMock).toHaveBeenCalledTimes(1)
    expect(state.importPreviewBlockTranslationAt(2)).toEqual({ kind: 'missing' })

    state.setImportPreviewBlockFocus(2)
    call = pendingBatch()
    const retry = state.translateImportPreviewFocusedBlock()
    expect(blocksMock).toHaveBeenLastCalledWith(['丙'], expect.any(Number), expect.any(Function))
    call.emit({ kind: 'received', index: 0, text: 'Bính', model: 'm-1', saved: true })
    call.finish(doneOutcome())
    await retry
    expect(state.importPreviewBlockTranslationAt(2)?.kind).toBe('done')
  })

  it('lỗi mạng giữa lô: khối đã đóng giữ, khối còn lại lỗi và bấm lại được', async () => {
    const { state } = await fresh()
    await open(state)
    await see(state, 0, 1)
    const call = pendingBatch()
    const error = { code: 'x', message_key: 'err.unknown', params: {}, retryable: true }

    const running = state.translateImportPreviewVisibleBlocks()
    call.emit({ kind: 'started', indices: [0, 1] })
    call.emit({ kind: 'received', index: 0, text: 'Giáp', model: 'm-1', saved: true })
    call.finish({ value: null, error })
    await running

    expect(state.importPreviewBlockTranslationAt(0)?.kind).toBe('done')
    expect(state.importPreviewBlockTranslationAt(1)).toEqual({ kind: 'failed', error })
    blocksMock.mockResolvedValue(doneOutcome())
    await state.translateImportPreviewVisibleBlocks()
    expect(blocksMock).toHaveBeenCalledTimes(2)
    expect(blocksMock.mock.calls[1]?.[0]).toEqual(['乙'])
  })

  it('khối quá trần không vào lượt: không tính vào Đang dịch N khối và không bị đánh lỗi khi mạng hỏng', async () => {
    const { state } = await fresh()
    await open(state)
    await see(state, 0, 1, 2)
    const call = pendingBatch()
    const error = { code: 'x', message_key: 'err.unknown', params: {}, retryable: true }

    const running = state.translateImportPreviewVisibleBlocks()
    call.emit({ kind: 'started', indices: [0, 1] })
    expect(state.importPreviewQuickBatchPendingCount.value).toBe(2)
    call.finish({ value: null, error })
    await running

    expect(state.importPreviewBlockTranslationAt(0)).toEqual({ kind: 'failed', error })
    expect(state.importPreviewBlockTranslationAt(2)).toBeNull()
  })

  it('chưa cấu hình, kho hỏng, bận: ghi nhận trạng thái và khối về nút', async () => {
    const { state } = await fresh()
    await open(state)
    await see(state, 0)
    for (const outcome of ['not_configured', 'store_unavailable', 'busy'] as const) {
      blocksMock.mockResolvedValue({ value: { state: outcome }, error: null })
      await state.translateImportPreviewVisibleBlocks()
      expect(state.importPreviewQuickBatchNote.value).toBe(outcome)
      expect(state.importPreviewBlockTranslationAt(0)).toBeNull()
    }
  })

  it('đổi Chương huỷ lượt, đóng màn huỷ lượt và quên câu dịch', async () => {
    const { state } = await fresh()
    previewChapterDetailMock.mockResolvedValue({
      detail: { cleanup: { text: '', spans: [], rules: [], window_truncated: false, final_text: 'x' }, blocks: { blocks: [block('丁')] } },
      error: null,
    })
    await open(state, PAGE, 2)
    await see(state, 0, 1)
    pendingBatch()
    void state.translateImportPreviewVisibleBlocks()
    expect(state.importPreviewQuickBatchRunning.value).toBe(true)

    state.nextImportPreviewChapter()
    await flushPromises()
    expect(state.importPreviewQuickBatchRunning.value).toBe(false)
    expect(cancelMock).toHaveBeenCalledTimes(1)

    state.prevImportPreviewChapter()
    await flushPromises()
    await see(state, 0)
    pendingBatch()
    void state.translateImportPreviewVisibleBlocks()
    state.cancelImportPreview()
    expect(cancelMock).toHaveBeenCalledTimes(2)
    expect(state.importPreviewBlockTranslations.value).toEqual({})
  })
})

describe('Dịch lại một khối', () => {
  async function translatedFirstBlock() {
    const f = await fresh()
    await open(f.state)
    await see(f.state, 0)
    const first = pendingBatch()
    const running = f.state.translateImportPreviewVisibleBlocks()
    first.emit({ kind: 'received', index: 0, text: 'Giáp', model: 'm-1', saved: true })
    first.finish(doneOutcome())
    await running
    f.state.setImportPreviewBlockFocus(0)
    return f
  }

  it('gửi riêng khối đó với retranslate và thay câu cũ bằng câu mới', async () => {
    const { state } = await translatedFirstBlock()
    const call = pendingBatch()
    const running = state.retranslateImportPreviewFocusedBlock()
    expect(blocksMock).toHaveBeenLastCalledWith(['甲'], expect.any(Number), expect.any(Function))
    expect(retranslateFlags).toEqual([false, true])
    call.emit({ kind: 'started', indices: [0] })
    expect(state.importPreviewBlockTranslationAt(0)?.kind).toBe('running')
    call.emit({ kind: 'received', index: 0, text: 'Giáp mới', model: 'm-2', saved: true })
    call.finish(doneOutcome({ model: 'm-2' }))
    await running
    expect(state.importPreviewBlockTranslationAt(0)).toEqual({ kind: 'done', text: 'Giáp mới', model: 'm-2' })
  })

  it('huỷ hoặc thiếu kết quả khi dịch lại thì câu cũ trở lại, vì kho vẫn giữ nó', async () => {
    const { state } = await translatedFirstBlock()
    let call = pendingBatch()
    let running = state.retranslateImportPreviewFocusedBlock()
    call.emit({ kind: 'started', indices: [0] })
    state.cancelImportPreviewQuickBatch()
    call.finish({ value: { state: 'cancelled' }, error: null })
    await running
    expect(state.importPreviewBlockTranslationAt(0)).toEqual({ kind: 'done', text: 'Giáp', model: 'm-1' })

    call = pendingBatch()
    running = state.retranslateImportPreviewFocusedBlock()
    call.emit({ kind: 'started', indices: [0] })
    call.finish(doneOutcome({ missing: [{ index: 0, reason: 'unclosed' }] }))
    await running
    expect(state.importPreviewBlockTranslationAt(0)).toEqual({ kind: 'done', text: 'Giáp', model: 'm-1' })
  })
})

describe('bận hai chiều', () => {
  it('Bảng chờ đang chạy thì lô không gọi và tự chạy được khi lượt kia xong', async () => {
    const { state, run } = await fresh()
    await open(state)
    await see(state, 0)
    const other = run.beginQuickRun('glossary_queue')
    expect(state.importPreviewQuickBatchBusyElsewhere.value).toBe(true)

    await state.translateImportPreviewVisibleBlocks()
    expect(blocksMock).not.toHaveBeenCalled()

    run.endQuickRun(other)
    expect(state.importPreviewQuickBatchBusyElsewhere.value).toBe(false)
    blocksMock.mockResolvedValue(doneOutcome())
    await state.translateImportPreviewVisibleBlocks()
    expect(blocksMock).toHaveBeenCalledTimes(1)
  })

  it('lô đang chạy làm Bảng chờ thấy bận, xong thì hết bận', async () => {
    const { state, run } = await fresh()
    await open(state)
    await see(state, 0)
    const call = pendingBatch()
    const running = state.translateImportPreviewVisibleBlocks()
    expect(run.quickRunBusyFor('glossary_queue').value).toBe(true)
    call.finish(doneOutcome())
    await running
    expect(run.quickRunBusyFor('glossary_queue').value).toBe(false)
  })
})

describe('ImportPreviewOverlay.vue — tầng khối', () => {
  async function mounted(deps: Partial<CommandDeps> = {}) {
    const f = await fresh({
      translateImportPreviewVisibleBlocks: () => void f.state.translateImportPreviewVisibleBlocks(),
      translateImportPreviewFocusedBlock: () => void f.state.translateImportPreviewFocusedBlock(),
      cancelImportPreviewQuickBatch: () => f.state.cancelImportPreviewQuickBatch(),
      cancelImportPreview: () => f.state.cancelImportPreview(),
      nextImportPreviewBlock: () => f.state.nextImportPreviewBlock(),
      prevImportPreviewBlock: () => f.state.prevImportPreviewBlock(),
      ...deps,
    })
    await open(f.state)
    await see(f.state, 0, 1, 2)
    const wrapper = mount(f.Overlay, { attachTo: document.body })
    await flushPromises()
    return { ...f, wrapper }
  }

  it('Esc đầu huỷ lượt và màn còn mở, Esc sau mới đóng; tiêu điểm không rơi xuống body', async () => {
    const { state, wrapper } = await mounted()
    pendingBatch()

    await wrapper.get('[data-ip-quick-translate]').trigger('click')
    await flushPromises()
    expect(wrapper.get('[data-ip-quick-running]').text()).toContain('3')
    expect(document.activeElement).not.toBe(document.body)

    await wrapper.get('.ip-screen').trigger('keydown', { key: 'Escape' })
    await flushPromises()
    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(state.importPreviewIsOpen.value).toBe(true)
    expect(wrapper.find('[data-ip-quick-cancel]').exists()).toBe(false)
    expect(document.activeElement).not.toBe(document.body)

    await wrapper.get('.ip-screen').trigger('keydown', { key: 'Escape' })
    expect(state.importPreviewIsOpen.value).toBe(false)
    wrapper.unmount()
  })

  it('nút dịch giữ tiêu điểm khi bấm chuột, nên Space sau cú bấm không gửi lượt nữa', async () => {
    const { state, wrapper } = await mounted()
    blocksMock.mockResolvedValue(doneOutcome())
    const button = wrapper.get('[data-ip-quick-translate]')
    const down = new MouseEvent('mousedown', { bubbles: true, cancelable: true })
    button.element.dispatchEvent(down)
    expect(down.defaultPrevented).toBe(true)

    await button.trigger('click')
    await flushPromises()
    expect(blocksMock).toHaveBeenCalledTimes(1)

    await wrapper.get('.ip-screen').trigger('keydown', { key: ' ' })
    await flushPromises()
    expect(blocksMock).toHaveBeenCalledTimes(1)
    expect(state.importPreviewQuickBatchRunning.value).toBe(false)
    wrapper.unmount()
  })

  it('nút của khối không có con trỏ ra khỏi thứ tự Tab, nên Enter không gửi nhầm khối con trỏ', async () => {
    const { state, wrapper } = await mounted()
    const call = pendingBatch()
    const running = state.translateImportPreviewVisibleBlocks()
    call.emit({ kind: 'received', index: 0, text: 'Giáp', model: 'm-1', saved: true })
    call.emit({ kind: 'received', index: 2, text: 'Bính', model: 'm-1', saved: true })
    call.finish(doneOutcome({ missing: [{ index: 1, reason: 'unclosed' }] }))
    await running
    state.setImportPreviewBlockFocus(0)
    await flushPromises()

    const own = wrapper.get('#ip-block-0 [data-ip-block-translation-retranslate]')
    expect(own.attributes('tabindex')).toBeUndefined()
    expect(wrapper.get('#ip-block-2 [data-ip-block-translation-retranslate]').attributes('tabindex')).toBe('-1')
    expect(wrapper.get('#ip-block-1 [data-ip-block-translation-retry]').attributes('tabindex')).toBe('-1')
    wrapper.unmount()
  })

  it('Rust trả chưa cấu hình mà Cài đặt đã sẵn sàng: báo lý do nhưng nút vẫn bấm lại được', async () => {
    const { state, wrapper } = await mounted()
    blocksMock.mockResolvedValue({ value: { state: 'not_configured' }, error: null })
    await state.translateImportPreviewVisibleBlocks()
    await flushPromises()

    expect(wrapper.find('[data-ip-quick-unconfigured]').exists()).toBe(true)
    expect(wrapper.get('[data-ip-quick-translate]').attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })

  it('xong lượt thì tiêu điểm về nút Dịch các khối đang thấy, không rơi xuống body', async () => {
    const { state, wrapper } = await mounted()
    ;(wrapper.get('[data-ip-quick-translate]').element as HTMLButtonElement).focus()
    const call = pendingBatch()
    const running = state.translateImportPreviewVisibleBlocks()
    await flushPromises()
    call.finish(doneOutcome())
    await running
    await flushPromises()

    expect(document.activeElement).toBe(wrapper.get('[data-ip-quick-translate]').element)
    wrapper.unmount()
  })

  it('J và K vẫn đổi con trỏ khối như trước', async () => {
    const { state, wrapper } = await mounted()
    await wrapper.get('.ip-screen').trigger('keydown', { key: 'j' })
    expect(state.importPreviewBlockFocusedIndex.value).toBe(1)
    await wrapper.get('.ip-screen').trigger('keydown', { key: 'k' })
    expect(state.importPreviewBlockFocusedIndex.value).toBe(0)
    wrapper.unmount()
  })

  it('câu dịch hiện dưới chữ gốc với dòng nguồn role=status; khối thiếu có Dịch thử; khối ảnh không có gì', async () => {
    const { state, wrapper } = await mounted()
    const call = pendingBatch()
    const running = state.translateImportPreviewVisibleBlocks()
    call.emit({ kind: 'started', indices: [0, 1, 2] })
    call.emit({ kind: 'received', index: 0, text: 'Giáp', model: 'm-1', saved: true })
    call.finish(doneOutcome({ missing: [{ index: 1, reason: 'missing' }, { index: 2, reason: 'missing' }] }))
    await running
    await flushPromises()

    const first = wrapper.get('#ip-block-0')
    expect(first.get('[data-ip-block-translation-text]').text()).toBe('Giáp')
    expect(first.get('[data-ip-block-translation-line]').attributes('role')).toBe('status')
    expect(first.get('[data-ip-block-translation-line]').text()).toContain('m-1')
    expect(wrapper.get('#ip-block-1').text()).toContain('Thiếu kết quả ở khối này.')
    expect(wrapper.find('#ip-block-1 [data-ip-block-translation-retry]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('chưa cấu hình AI: nút tắt kèm lý do và lối sang Cài đặt, không gọi mạng', async () => {
    const f = await fresh()
    quickAiConfigGetMock.mockResolvedValue({
      config: { fields: [], source: 'dedicated', key_configured: null, ready: false, not_ready: ['endpoint_missing'] },
      error: null,
    })
    await open(f.state)
    const wrapper = mount(f.Overlay, { attachTo: document.body })
    await flushPromises()

    expect(wrapper.get('[data-ip-quick-translate]').attributes('disabled')).toBeDefined()
    expect(wrapper.find('[data-ip-quick-unconfigured]').exists()).toBe(true)
    expect(wrapper.find('[data-ip-quick-open-settings]').exists()).toBe(true)
    expect(blocksMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('Bảng chờ đang chạy: nút tắt với Đang dịch ở màn khác. và tự bật lại', async () => {
    const { run, wrapper } = await mounted()
    const other = run.beginQuickRun('glossary_queue')
    await flushPromises()
    expect(wrapper.get('[data-ip-quick-translate]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-ip-quick-busy]').text()).toBe('Đang dịch ở màn khác.')

    run.endQuickRun(other)
    await flushPromises()
    expect(wrapper.get('[data-ip-quick-translate]').attributes('disabled')).toBeUndefined()
    expect(wrapper.find('[data-ip-quick-busy]').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('dọn kho dịch nhanh', () => {
  it('câu đã hiện về Chưa có và khối đang thấy được tra lại trên kho đã dọn', async () => {
    const { state, run } = await fresh()
    lookupManyMock.mockResolvedValue({ value: [{ state: 'hit', text: 'Giáp', model: 'm-1' }], error: null })
    await open(state)
    await see(state, 0)
    expect(state.importPreviewBlockTranslationAt(0)?.kind).toBe('done')

    lookupManyMock.mockImplementation(async (texts: string[]) => ({ value: texts.map(() => miss), error: null }))
    run.markQuickStorePurged()
    await flushPromises()

    expect(lookupManyMock).toHaveBeenCalledTimes(2)
    expect(state.importPreviewBlockTranslationAt(0)).toBeNull()
  })

  it('lô đang chạy bị bỏ: sự kiện và kết quả muộn không ghi lại, lượt chạy hết bận', async () => {
    const { state, run } = await fresh()
    await open(state)
    await see(state, 0, 1, 2)
    const call = pendingBatch()
    const running = state.translateImportPreviewVisibleBlocks()
    call.emit({ kind: 'started', indices: [0, 1, 2] })
    call.emit({ kind: 'received', index: 0, text: 'Giáp', model: 'm-1', saved: true })

    run.markQuickStorePurged()
    await flushPromises()
    call.emit({ kind: 'received', index: 1, text: 'Ất', model: 'm-1', saved: true })
    call.finish({ value: { state: 'cancelled' }, error: null })
    await running

    expect(state.importPreviewQuickBatchRunning.value).toBe(false)
    expect(run.quickRunCurrent.value).toBeNull()
    for (const index of [0, 1, 2]) expect(state.importPreviewBlockTranslationAt(index)).toBeNull()
  })
})
