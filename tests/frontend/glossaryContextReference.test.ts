import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { GlossaryCandidate } from '../../src/config/glossary'
import type { CommandDeps } from '../../src/commands'

const pendingMock = vi.fn()
const lookupMock = vi.fn()
const contextLookupMock = vi.fn()
const translateMock = vi.fn()
const cancelMock = vi.fn()
const storeLookupMock = vi.fn()
const quickAiConfigGetMock = vi.fn()

vi.mock('../../src/config/glossary', () => ({
  pendingGlossaryCandidates: () => pendingMock(),
  lookupGlossaryTerm: (term: string) => lookupMock(term),
  approveGlossaryCandidate: async () => ({ value: 1, error: null }),
  rejectGlossaryCandidate: async () => ({ value: null, error: null }),
  countGlossaryEntries: async () => null,
}))

vi.mock('../../src/config/glossaryContextReference', () => ({
  lookupContextTranslation: (context: string) => contextLookupMock(context),
  quickTranslationLookup: (text: string) => storeLookupMock(text),
  translateContextReference: (context: string, runId: number, retranslate: boolean, onToken: (text: string) => void) =>
    translateMock(context, onToken, runId, retranslate),
  cancelContextReference: (runId: number) => cancelMock(runId),
}))

vi.mock('../../src/config/quickAiConfig', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/quickAiConfig')>()),
  quickAiConfigGet: () => quickAiConfigGetMock(),
}))

vi.mock('../../src/config/aiconfig', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/aiconfig')>()),
  aiConfigGet: async () => ({ fields: [], defaultSystemPrompt: '', workTierAvailable: true, keyConfigured: false, ready: false, notReady: ['endpoint_missing'], error: null }),
}))

function candidate(id: number, over: Partial<GlossaryCandidate> = {}): GlossaryCandidate {
  return {
    id,
    source_term: `词${id}`,
    candidate_origin: 'import_scan',
    resolution: null,
    created_at: '2026-11-01T00:00:00.000Z',
    occurrence_count: 10,
    context_example: `句${id}。`,
    han_viet_suggestion: null,
    han_viet_status: 'not_chinese',
    chapter_span_count: 0,
    replaced_translation: null,
    proposed_translation: null,
    changed_count: null,
    seen_count: null,
    ...over,
  }
}

function runIdOf(mock: ReturnType<typeof vi.fn>): number {
  return (mock.mock.calls.at(-1) as unknown[])[2] as number
}

const notFound = { value: null, failed: false, error: null }
const miss = { value: { state: 'miss' }, error: null }
const done = { value: { state: 'done', model: 'm-1', usage: null, maybe_truncated: false }, error: null }

async function fresh(deps: Partial<CommandDeps> = {}) {
  vi.resetModules()
  for (const m of [pendingMock, lookupMock, contextLookupMock, translateMock, cancelMock, storeLookupMock, quickAiConfigGetMock]) m.mockReset()
  storeLookupMock.mockResolvedValue(miss)
  quickAiConfigGetMock.mockResolvedValue({
    config: { fields: [], source: 'dedicated', key_configured: null, ready: true, not_ready: [] },
    error: null,
  })
  const commands = await import('../../src/commands')
  commands.installCommands(deps as CommandDeps)
  const state = await import('../../src/glossaryQueueState')
  const i18n = await import('../../src/i18n')
  const Overlay = (await import('../../src/GlossaryQueueOverlay.vue')).default
  return { state, i18n, Overlay, commands }
}

async function openWith(state: Awaited<ReturnType<typeof fresh>>['state'], ...candidates: GlossaryCandidate[]) {
  pendingMock.mockResolvedValue({ candidates, error: null })
  await state.openGlossaryQueue()
  await flushPromises()
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('câu dịch có sẵn trong Tác phẩm', () => {
  it('hiện bản dịch của Chương và không gọi AI', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue({ value: 'Bản dịch.', failed: false, error: null })

    await openWith(state, candidate(1))

    expect(contextLookupMock).toHaveBeenCalledWith('句1。')
    expect(state.queueContextReference.value).toEqual({ kind: 'chapter', text: 'Bản dịch.' })
    expect(translateMock).not.toHaveBeenCalled()
  })

  it('không có Ngữ cảnh thì không gọi lệnh nào', async () => {
    const { state } = await fresh()

    await openWith(state, candidate(1, { context_example: null }))

    expect(contextLookupMock).not.toHaveBeenCalled()
    expect(state.queueContextReference.value).toBeNull()
    await state.translateGlossaryContextReference()
    expect(translateMock).not.toHaveBeenCalled()
  })

  it('chưa có bản dịch thì chờ cú bấm, không tự gọi AI', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)

    await openWith(state, candidate(1))

    expect(state.queueContextReference.value).toEqual({ kind: 'none' })
    expect(translateMock).not.toHaveBeenCalled()
  })
})

describe('Dịch thử', () => {
  it('chữ chảy dần rồi giữ lại khi lần gọi xong, quay lại hàng không gọi lần hai', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    let release: (v: unknown) => void = () => undefined
    translateMock.mockImplementation((_c: string, onToken: (t: string) => void) => {
      onToken('Xin ')
      onToken('chào')
      return new Promise((resolve) => {
        release = resolve
      })
    })
    await openWith(state, candidate(1), candidate(2))

    const run = state.translateGlossaryContextReference()
    await flushPromises()
    expect(state.queueContextReference.value).toEqual({ kind: 'running', text: 'Xin chào' })

    release(done)
    await run
    expect(state.queueContextReference.value).toEqual({ kind: 'ai', text: 'Xin chào', model: 'm-1', usage: null })

    state.nextGlossaryQueueCandidate()
    await flushPromises()
    state.prevGlossaryQueueCandidate()
    await flushPromises()
    expect(state.queueContextReference.value).toEqual({ kind: 'ai', text: 'Xin chào', model: 'm-1', usage: null })
    expect(translateMock).toHaveBeenCalledTimes(1)
  })

  it('chuyển hàng khi đang chạy huỷ lần gọi và chữ muộn không rơi vào hàng khác', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    let lateToken: (t: string) => void = () => undefined
    let release: (v: unknown) => void = () => undefined
    translateMock.mockImplementation((_c: string, onToken: (t: string) => void) => {
      lateToken = onToken
      return new Promise((resolve) => {
        release = resolve
      })
    })
    await openWith(state, candidate(1), candidate(2))

    const run = state.translateGlossaryContextReference()
    await flushPromises()
    state.nextGlossaryQueueCandidate()
    await flushPromises()

    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(cancelMock).toHaveBeenCalledWith(runIdOf(translateMock))
    lateToken('muộn')
    release({ value: { state: 'cancelled' }, error: null })
    await run
    expect(state.queueContextReference.value).toEqual({ kind: 'none' })
    state.prevGlossaryQueueCandidate()
    await flushPromises()
    expect(state.queueContextReference.value).toEqual({ kind: 'none' })
    expect(state.queueActionError.value).toBeNull()
  })

  it('Huỷ quay về nút và gọi lệnh huỷ riêng', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    translateMock.mockImplementation(() => new Promise(() => undefined))
    await openWith(state, candidate(1))

    void state.translateGlossaryContextReference()
    await flushPromises()
    state.cancelGlossaryContextReference()

    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(state.queueContextReference.value).toEqual({ kind: 'none' })
  })

  it('đóng lớp phủ huỷ lần gọi', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    translateMock.mockImplementation(() => new Promise(() => undefined))
    await openWith(state, candidate(1))

    void state.translateGlossaryContextReference()
    await flushPromises()
    state.closeGlossaryQueue()

    expect(cancelMock).toHaveBeenCalledTimes(1)
  })

  it('lỗi hiện ra và bấm lại được; chưa cấu hình ghi nhận trạng thái riêng', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    const error = { code: 'x', message_key: 'err.unknown', params: {}, retryable: true }
    translateMock.mockResolvedValueOnce({ value: null, error })
    await openWith(state, candidate(1))

    await state.translateGlossaryContextReference()
    expect(state.queueContextReference.value).toEqual({ kind: 'failed', error })

    translateMock.mockResolvedValueOnce({ value: { state: 'not_configured' }, error: null })
    await state.translateGlossaryContextReference()
    expect(state.queueContextReference.value).toEqual({ kind: 'not_configured' })
  })

  it('hàng đã có câu AI không chạy lại lần gọi trả phí', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    translateMock.mockImplementation((_c: string, onToken: (t: string) => void) => {
      onToken('Một')
      return Promise.resolve(done)
    })
    await openWith(state, candidate(1))
    await state.translateGlossaryContextReference()
    await state.translateGlossaryContextReference()

    expect(translateMock).toHaveBeenCalledTimes(1)
  })

  it('lớp phủ đóng thì lệnh dịch không gọi AI', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    await openWith(state, candidate(1))
    state.closeGlossaryQueue()

    await state.translateGlossaryContextReference()

    expect(translateMock).not.toHaveBeenCalled()
  })

  it('resetGlossaryQueue quên câu AI đã dịch', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    translateMock.mockImplementation((_c: string, onToken: (t: string) => void) => {
      onToken('Một')
      return Promise.resolve(done)
    })
    await openWith(state, candidate(1))
    await state.translateGlossaryContextReference()
    expect(state.queueContextReference.value?.kind).toBe('ai')

    state.resetGlossaryQueue()
    await openWith(state, candidate(1))

    expect(state.queueContextReference.value).toEqual({ kind: 'none' })
    expect(storeLookupMock).toHaveBeenCalledTimes(2)
  })
})

describe('kho dịch nhanh', () => {
  it('trúng kho thì hiện câu đã lưu cùng mô hình và không gọi AI', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    storeLookupMock.mockResolvedValue({ value: { state: 'hit', text: 'Đã lưu.', model: 'm-0' }, error: null })

    await openWith(state, candidate(1))

    expect(storeLookupMock).toHaveBeenCalledWith('句1。')
    expect(state.queueContextReference.value).toEqual({ kind: 'cached', text: 'Đã lưu.', model: 'm-0' })
    expect(translateMock).not.toHaveBeenCalled()
  })

  it('bản dịch trong Tác phẩm đứng trước kho: không tra kho', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue({ value: 'Bản dịch.', failed: false, error: null })

    await openWith(state, candidate(1))

    expect(storeLookupMock).not.toHaveBeenCalled()
  })

  it('kho không mở được thì hiện kind riêng và nút Dịch thử không chạy', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    storeLookupMock.mockResolvedValue({ value: { state: 'store_unavailable' }, error: null })

    await openWith(state, candidate(1))
    await state.translateGlossaryContextReference()

    expect(state.queueContextReference.value).toEqual({ kind: 'store_unavailable' })
    expect(translateMock).not.toHaveBeenCalled()
  })

  it('lệnh dịch báo store_unavailable hoặc cached thì đổi sang kind tương ứng', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    await openWith(state, candidate(1))

    translateMock.mockResolvedValueOnce({ value: { state: 'store_unavailable' }, error: null })
    await state.translateGlossaryContextReference()
    expect(state.queueContextReference.value).toEqual({ kind: 'store_unavailable' })

    state.resetGlossaryQueue()
    await openWith(state, candidate(1))
    translateMock.mockResolvedValueOnce({ value: { state: 'cached', text: 'Kho.', model: 'm-2' }, error: null })
    await state.translateGlossaryContextReference()
    expect(state.queueContextReference.value).toEqual({ kind: 'cached', text: 'Kho.', model: 'm-2' })
  })

  it('maybe_truncated giữ phần đã nhận ở kind truncated', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    translateMock.mockImplementation((_c: string, onToken: (t: string) => void) => {
      onToken('Nửa câu')
      return Promise.resolve({ value: { state: 'done', model: 'm-1', usage: null, maybe_truncated: true }, error: null })
    })
    await openWith(state, candidate(1))

    await state.translateGlossaryContextReference()

    expect(state.queueContextReference.value).toEqual({ kind: 'truncated', text: 'Nửa câu', model: 'm-1', usage: null })
  })

  it('run_id tăng dần theo từng lượt và đi theo lệnh huỷ', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    translateMock.mockImplementation(() => new Promise(() => undefined))
    await openWith(state, candidate(1))

    void state.translateGlossaryContextReference()
    await flushPromises()
    const first = runIdOf(translateMock)
    state.cancelGlossaryContextReference()
    void state.translateGlossaryContextReference()
    await flushPromises()
    const second = runIdOf(translateMock)

    expect(second).toBeGreaterThan(first)
    expect(cancelMock).toHaveBeenCalledWith(first)
    state.cancelGlossaryContextReference()
    expect(cancelMock).toHaveBeenLastCalledWith(second)
  })

  it('Dịch lại gọi với retranslate và huỷ nó trả về câu cũ', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    storeLookupMock.mockResolvedValue({ value: { state: 'hit', text: 'Cũ.', model: 'm-0' }, error: null })
    translateMock.mockImplementation(() => new Promise(() => undefined))
    await openWith(state, candidate(1))

    void state.retranslateGlossaryContextReference()
    await flushPromises()
    expect(translateMock.mock.calls.at(-1)?.[3]).toBe(true)
    expect(state.queueContextReference.value?.kind).toBe('running')

    state.cancelGlossaryContextReference()
    expect(state.queueContextReference.value).toEqual({ kind: 'cached', text: 'Cũ.', model: 'm-0' })
  })

  it('Dịch thử không chạy khi hàng đã có câu AI, Dịch lại không chạy khi chưa có', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    await openWith(state, candidate(1))

    await state.retranslateGlossaryContextReference()
    expect(translateMock).not.toHaveBeenCalled()
  })
})

describe('GlossaryQueueOverlay.vue — khối câu dịch', () => {
  it('bản dịch của Chương hiện kèm nhãn nguồn, không có nút', async () => {
    const { state, i18n, Overlay } = await fresh()
    contextLookupMock.mockResolvedValue({ value: 'Bản dịch.', failed: false, error: null })
    await openWith(state, candidate(1))

    const wrapper = mount(Overlay, { attachTo: document.body })
    await flushPromises()

    expect(wrapper.get('[data-gq-context-ref-text]').text()).toBe('Bản dịch.')
    expect(wrapper.get('[data-gq-context-ref-source]').text()).toBe(i18n.t('glossary.queue.context_source_chapter'))
    expect(wrapper.find('[data-gq-context-translate]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('nút Dịch thử đi qua dispatch và nhãn nguồn của AI nói rõ là tham khảo', async () => {
    const translate = vi.fn()
    const { state, i18n, Overlay } = await fresh({ translateGlossaryContextReference: translate })
    contextLookupMock.mockResolvedValue(notFound)
    await openWith(state, candidate(1))

    const wrapper = mount(Overlay, { attachTo: document.body })
    await flushPromises()
    await wrapper.get('[data-gq-context-translate]').trigger('click')
    expect(translate).toHaveBeenCalledTimes(1)

    translateMock.mockResolvedValue(done)
    translateMock.mockImplementation((_c: string, onToken: (t: string) => void) => {
      onToken('Tham khảo')
      return Promise.resolve(done)
    })
    await state.translateGlossaryContextReference()
    await flushPromises()
    expect(wrapper.get('[data-gq-context-ref-source]').text()).toBe(i18n.t('glossary.queue.context_source_ai'))
    wrapper.unmount()
  })

  it('dedicated quick readiness enables translation while the main config is incomplete', async () => {
    const { state, Overlay } = await fresh()
    const main = await import('../../src/aiConfigState')
    await main.refreshAiConfigReadiness()
    expect(main.aiConfigReadiness.value).toBe('unconfigured')
    quickAiConfigGetMock.mockResolvedValue({ config: { fields: [], source: 'dedicated', key_configured: true, ready: true, not_ready: [] }, error: null })
    contextLookupMock.mockResolvedValue(notFound)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await openWith(state, candidate(1))
    await flushPromises()
    expect(wrapper.get('[data-gq-context-translate]').attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })

  it('chưa cấu hình AI thì nút tắt kèm lý do và lối sang Cài đặt', async () => {
    const openSettings = vi.fn()
    const { state, Overlay } = await fresh({ openSettingsToAiConfig: openSettings })
    quickAiConfigGetMock.mockResolvedValue({
      config: { fields: [], source: 'incomplete', key_configured: null, ready: false, not_ready: ['endpoint_missing'] },
      error: null,
    })
    contextLookupMock.mockResolvedValue(notFound)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await openWith(state, candidate(1))
    await flushPromises()

    expect(wrapper.get('[data-gq-context-translate]').attributes('disabled')).toBeDefined()
    expect(wrapper.find('[data-gq-context-unconfigured]').exists()).toBe(true)
    await wrapper.get('[data-gq-context-open-settings]').trigger('click')
    expect(openSettings).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('chưa cấu hình AI thì Dịch lại cũng tắt, câu trong kho vẫn hiện', async () => {
    const retranslate = vi.fn()
    const { state, Overlay } = await fresh({ retranslateGlossaryContextReference: retranslate })
    quickAiConfigGetMock.mockResolvedValue({
      config: { fields: [], source: 'incomplete', key_configured: null, ready: false, not_ready: ['endpoint_missing'] },
      error: null,
    })
    contextLookupMock.mockResolvedValue(notFound)
    storeLookupMock.mockResolvedValue({ value: { state: 'hit', text: 'Đã lưu.', model: 'm-0' }, error: null })
    const wrapper = mount(Overlay, { attachTo: document.body })
    await openWith(state, candidate(1))
    await flushPromises()

    expect(wrapper.get('[data-gq-context-ref-text]').text()).toBe('Đã lưu.')
    expect(wrapper.get('[data-gq-context-retranslate]').attributes('disabled')).toBeDefined()
    await wrapper.get('[data-gq-context-retranslate]').trigger('click')
    expect(retranslate).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('câu trong kho hiện mô hình và nút Dịch lại đi qua dispatch', async () => {
    const retranslate = vi.fn()
    const { state, i18n, Overlay } = await fresh({ retranslateGlossaryContextReference: retranslate })
    contextLookupMock.mockResolvedValue(notFound)
    storeLookupMock.mockResolvedValue({ value: { state: 'hit', text: 'Đã lưu.', model: 'm-0' }, error: null })
    await openWith(state, candidate(1))

    const wrapper = mount(Overlay, { attachTo: document.body })
    await flushPromises()

    expect(wrapper.get('[data-gq-context-ref-line]').attributes('role')).toBe('status')
    expect(wrapper.get('[data-gq-context-ref-model]').text()).toBe(i18n.t('glossary.queue.context_model', { model: 'm-0' }))
    expect(wrapper.find('[data-gq-context-ref-usage]').exists()).toBe(false)
    await wrapper.get('[data-gq-context-retranslate]').trigger('click')
    expect(retranslate).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('lượt vừa chạy hiện dòng token, chuỗi bị cắt và kho hỏng có chữ riêng', async () => {
    const { state, i18n, Overlay } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    const usage = { prompt_tokens: 1, completion_tokens: 2, total_tokens: 3, cost_usd: null }
    translateMock.mockImplementation((_c: string, onToken: (t: string) => void) => {
      onToken('Câu')
      return Promise.resolve({ value: { state: 'done', model: 'm-1', usage, maybe_truncated: false }, error: null })
    })
    await openWith(state, candidate(1))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await flushPromises()

    await state.translateGlossaryContextReference()
    await flushPromises()
    expect(wrapper.get('[data-gq-context-ref-usage]').text()).toBe(
      i18n.t('ai.translate.usage_no_price', { token_count: '3' }),
    )

    state.resetGlossaryQueue()
    translateMock.mockImplementation((_c: string, onToken: (t: string) => void) => {
      onToken('Nửa')
      return Promise.resolve({ value: { state: 'done', model: 'm-1', usage: null, maybe_truncated: true }, error: null })
    })
    await openWith(state, candidate(1))
    await flushPromises()
    await state.translateGlossaryContextReference()
    await flushPromises()
    expect(wrapper.get('[data-gq-context-ref-source]').text()).toBe(i18n.t('glossary.queue.context_truncated'))
    expect(wrapper.find('[data-gq-context-retranslate]').exists()).toBe(true)

    state.resetGlossaryQueue()
    storeLookupMock.mockResolvedValue({ value: { state: 'store_unavailable' }, error: null })
    await openWith(state, candidate(1))
    await flushPromises()
    expect(wrapper.get('[data-gq-context-store-unavailable]').text()).toBe(i18n.t('glossary.queue.context_store_unavailable'))
    expect(wrapper.get('[data-gq-context-translate]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('Esc đầu huỷ lượt đang chạy và lớp phủ còn mở, Esc sau mới đóng; tiêu điểm đi theo nút', async () => {
    const { state, Overlay } = await fresh({
      cancelGlossaryContextReference: () => state.cancelGlossaryContextReference(),
      translateGlossaryContextReference: () => void state.translateGlossaryContextReference(),
      closeGlossaryQueue: () => state.closeGlossaryQueue(),
    })
    contextLookupMock.mockResolvedValue(notFound)
    translateMock.mockImplementation(() => new Promise(() => undefined))
    await openWith(state, candidate(1))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await flushPromises()

    await wrapper.get('[data-gq-context-translate]').trigger('click')
    await flushPromises()
    expect(document.activeElement).toBe(wrapper.get('[data-gq-context-cancel]').element)

    await wrapper.get('[data-gq-context-cancel]').trigger('keydown', { key: 'Escape' })
    await flushPromises()
    expect(cancelMock).toHaveBeenCalledTimes(1)
    expect(state.queueOverlayIsOpen.value).toBe(true)
    expect(wrapper.find('[data-gq-context-cancel]').exists()).toBe(false)
    expect(document.activeElement).toBe(wrapper.get('[data-gq-context-translate]').element)

    await wrapper.get('[data-gq-context-translate]').trigger('keydown', { key: 'Escape' })
    expect(state.queueOverlayIsOpen.value).toBe(false)
    wrapper.unmount()
  })
})

describe('bận giữa hai bề mặt dịch nhanh', () => {
  it('lượt của Bảng chờ khai bề mặt của nó và nhả khi xong hoặc khi huỷ', async () => {
    const { state } = await fresh()
    const run = await import('../../src/quickTranslateRun')
    contextLookupMock.mockResolvedValue(notFound)
    translateMock.mockImplementation(() => new Promise(() => undefined))
    await openWith(state, candidate(1))

    void state.translateGlossaryContextReference()
    expect(run.quickRunCurrent.value).toEqual({ surface: 'glossary_queue', runId: runIdOf(translateMock) })
    expect(run.quickRunBusyFor('import_preview').value).toBe(true)
    expect(state.queueContextBusyElsewhere.value).toBe(false)

    state.cancelGlossaryContextReference()
    expect(run.quickRunCurrent.value).toBeNull()
  })

  it('bề mặt kia đang chạy thì nút tắt kèm Đang dịch ở màn khác. và tự bật lại', async () => {
    const { state, Overlay } = await fresh()
    const run = await import('../../src/quickTranslateRun')
    contextLookupMock.mockResolvedValue(notFound)
    await openWith(state, candidate(1))
    const wrapper = mount(Overlay, { attachTo: document.body })
    await flushPromises()

    const other = run.beginQuickRun('import_preview')
    await flushPromises()
    expect(wrapper.get('[data-gq-context-translate]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-gq-context-busy]').text()).toBe('Đang dịch ở màn khác.')

    run.endQuickRun(other)
    await flushPromises()
    expect(wrapper.get('[data-gq-context-translate]').attributes('disabled')).toBeUndefined()
    expect(wrapper.find('[data-gq-context-busy]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('Rust trả busy thì hàng về trạng thái bấm lại được kèm lời giải thích, không lưu chữ nào', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    translateMock.mockResolvedValue({ value: { state: 'busy' }, error: null })
    await openWith(state, candidate(1))

    await state.translateGlossaryContextReference()

    expect(state.queueContextReference.value).toEqual({ kind: 'busy' })
    translateMock.mockResolvedValue(done)
    await state.translateGlossaryContextReference()
    expect(translateMock).toHaveBeenCalledTimes(2)
  })
})

describe('dọn kho dịch nhanh', () => {
  it('câu lấy từ kho về Chưa có, câu lấy từ Tác phẩm giữ nguyên', async () => {
    const { state } = await fresh()
    contextLookupMock.mockImplementation(async (context: string) =>
      context === '句2。' ? { value: 'Của Chương.', failed: false, error: null } : notFound,
    )
    storeLookupMock.mockResolvedValue({ value: { state: 'hit', text: 'Đã lưu.', model: 'm-0' }, error: null })
    await openWith(state, candidate(1), candidate(2))
    expect(state.queueContextReference.value?.kind).toBe('cached')
    state.nextGlossaryQueueCandidate()
    await flushPromises()
    expect(state.queueContextReference.value).toEqual({ kind: 'chapter', text: 'Của Chương.' })

    storeLookupMock.mockResolvedValue(miss)
    const run = await import('../../src/quickTranslateRun')
    run.markQuickStorePurged()
    await flushPromises()
    expect(state.queueContextReference.value).toEqual({ kind: 'chapter', text: 'Của Chương.' })
    state.prevGlossaryQueueCandidate()
    await flushPromises()
    expect(state.queueContextReference.value).toEqual({ kind: 'none' })
  })

  it('lượt đang chạy bị bỏ: chữ muộn và kết quả muộn không ghi lại, bề mặt về Chưa có', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    let lateToken: (t: string) => void = () => undefined
    let release: (v: unknown) => void = () => undefined
    translateMock.mockImplementation((_c: string, onToken: (t: string) => void) => {
      lateToken = onToken
      onToken('Xin ')
      return new Promise((resolve) => {
        release = resolve
      })
    })
    await openWith(state, candidate(1))
    const running = state.translateGlossaryContextReference()
    await flushPromises()
    expect(state.queueContextReference.value).toEqual({ kind: 'running', text: 'Xin ' })

    const run = await import('../../src/quickTranslateRun')
    run.markQuickStorePurged()
    await flushPromises()
    lateToken('chào')
    release(done)
    await running
    await flushPromises()

    expect(state.queueContextReference.value).toEqual({ kind: 'none' })
    expect(run.quickRunCurrent.value).toBeNull()
  })

  it('tra kho đang bay lúc dọn không đưa câu cũ trở lại', async () => {
    const { state } = await fresh()
    contextLookupMock.mockResolvedValue(notFound)
    let answer: (v: unknown) => void = () => undefined
    storeLookupMock.mockImplementationOnce(
      () => new Promise((resolve) => {
        answer = resolve
      }),
    )
    await openWith(state, candidate(1))

    const run = await import('../../src/quickTranslateRun')
    run.markQuickStorePurged()
    answer({ value: { state: 'hit', text: 'Cũ.', model: 'm-0' }, error: null })
    await flushPromises()

    expect(state.queueContextReference.value).toEqual({ kind: 'none' })
  })
})
