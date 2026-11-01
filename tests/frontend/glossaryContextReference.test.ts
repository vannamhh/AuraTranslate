import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { GlossaryCandidate } from '../../src/config/glossary'
import type { CommandDeps } from '../../src/commands'

const pendingMock = vi.fn()
const lookupMock = vi.fn()
const contextLookupMock = vi.fn()
const translateMock = vi.fn()
const cancelMock = vi.fn()
const aiConfigGetMock = vi.fn()

vi.mock('../../src/config/glossary', () => ({
  pendingGlossaryCandidates: () => pendingMock(),
  lookupGlossaryTerm: (term: string) => lookupMock(term),
  approveGlossaryCandidate: async () => ({ value: 1, error: null }),
  rejectGlossaryCandidate: async () => ({ value: null, error: null }),
  countGlossaryEntries: async () => null,
}))

vi.mock('../../src/config/glossaryContextReference', () => ({
  lookupContextTranslation: (context: string) => contextLookupMock(context),
  translateContextReference: (context: string, onToken: (text: string) => void) =>
    translateMock(context, onToken),
  cancelContextReference: () => cancelMock(),
}))

vi.mock('../../src/config/aiconfig', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/aiconfig')>()),
  aiConfigGet: () => aiConfigGetMock(),
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

const notFound = { value: null, failed: false, error: null }
const done = { value: { state: 'done', usage: null, maybe_truncated: false }, error: null }

async function fresh(deps: Partial<CommandDeps> = {}) {
  vi.resetModules()
  for (const m of [pendingMock, lookupMock, contextLookupMock, translateMock, cancelMock, aiConfigGetMock]) m.mockReset()
  aiConfigGetMock.mockResolvedValue({
    fields: null,
    defaultSystemPrompt: '',
    workTierAvailable: false,
    keyConfigured: null,
    ready: null,
    notReady: [],
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
    expect(state.queueContextReference.value).toEqual({ kind: 'ai', text: 'Xin chào' })

    state.nextGlossaryQueueCandidate()
    await flushPromises()
    state.prevGlossaryQueueCandidate()
    await flushPromises()
    expect(state.queueContextReference.value).toEqual({ kind: 'ai', text: 'Xin chào' })
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

  it('chưa cấu hình AI thì nút tắt kèm lý do và lối sang Cài đặt', async () => {
    const openSettings = vi.fn()
    const { state, Overlay } = await fresh({ openSettingsToAiConfig: openSettings })
    aiConfigGetMock.mockResolvedValue({
      fields: [],
      defaultSystemPrompt: '',
      workTierAvailable: false,
      keyConfigured: false,
      ready: false,
      notReady: ['endpoint_missing'],
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
})
