import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'

const runMock = vi.fn()

vi.mock('../../src/config/proofread', () => ({
  runProofreadSegments: (...args: unknown[]) => runMock(...args),
  cancelProofreadCall: () => Promise.resolve(),
}))
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/panels/editorPanelState')>()),
  flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
}))

const { setEditorCaret } = await import('../../src/panels/editorPanelState')
const { runProofread, resetProofread } = await import('../../src/proofreadState')
const { t } = await import('../../src/i18n')
const ProofreaderStrip = (await import('../../src/ProofreaderStrip.vue')).default

const FINDING = { kind: 'spelling', start: 0, end: 2, explanation: 'e', suggestion: 's' }
const result = (id: number) => ({ segment_id: id, scanned_text: 'ab', findings: [FINDING], unlocated: 0 })
const usage = (total: number, cost: number | null) => ({ prompt_tokens: total, completion_tokens: 0, total_tokens: total, cost_usd: cost })

afterEach(() => {
  resetProofread()
  runMock.mockReset()
})

function statusTexts(wrapper: ReturnType<typeof mount>): string[] {
  return wrapper.findAll('[role="status"]').map((p) => p.text())
}

describe('ProofreaderStrip', () => {
  it('invites to configure AI when the scan reports not_configured', async () => {
    setEditorCaret(5)
    runMock.mockResolvedValue({ value: { state: 'not_configured' }, error: null })
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([5])
    await flushPromises()

    expect(wrapper.find('[role="status"]').text()).toBe(t('panel.ai_translation.status'))
    wrapper.unmount()
  })

  it('a finished chapter scan shows the finding count, the sentences scanned and the full total with cost', async () => {
    setEditorCaret(1)
    runMock.mockImplementation((_ids: number[], onEvent: (e: unknown) => void) => {
      onEvent({ kind: 'planned', segments: 3, batches: 2 })
      onEvent({ kind: 'batch', usage: usage(100, 0.01), segments: [result(1), result(2)] })
      onEvent({ kind: 'batch', usage: usage(50, 0.005), segments: [result(3)] })
      return Promise.resolve({ value: { state: 'done', not_scanned: 0 }, error: null })
    })
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1, 2, 3])
    await flushPromises()

    const texts = statusTexts(wrapper)
    expect(texts).toContain(t('proofread.strip.found_many', { finding_count: '3', sentence_count: '3' }))
    expect(texts).toContain(t('proofread.usage_total_with_cost', { token_count: '150', cost_usd: '0,0150', sentence_count: '3' }))
    wrapper.unmount()
  })

  it('a cancelled scan shows the partial total, the underlined count and the sentences left unscanned', async () => {
    setEditorCaret(1)
    runMock.mockImplementation((_ids: number[], onEvent: (e: unknown) => void) => {
      onEvent({ kind: 'planned', segments: 4, batches: 4 })
      onEvent({ kind: 'batch', usage: usage(100, null), segments: [result(1)] })
      onEvent({ kind: 'batch', usage: null, segments: [result(2)] })
      return Promise.resolve({ value: { state: 'cancelled', not_scanned: 2 }, error: null })
    })
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1, 2, 3, 4])
    await flushPromises()

    const texts = statusTexts(wrapper)
    expect(texts).toContain(t('proofread.strip.cancelled'))
    expect(texts).toContain(t('proofread.strip.not_scanned', { not_scanned_count: '2' }))
    expect(texts).toContain(t('proofread.usage_partial', { token_count: '100', reported_count: '1', sentence_count: '2' }))
    wrapper.unmount()
  })

  it('keeps showing the partial result after a cancel nulls the caret', async () => {
    setEditorCaret(1)
    runMock.mockImplementation((_ids: number[], onEvent: (e: unknown) => void) => {
      onEvent({ kind: 'planned', segments: 2, batches: 2 })
      onEvent({ kind: 'batch', usage: usage(100, null), segments: [result(1)] })
      return Promise.resolve({ value: { state: 'cancelled', not_scanned: 1 }, error: null })
    })
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1, 2])
    setEditorCaret(null)
    await flushPromises()
    expect(statusTexts(wrapper)).toContain(t('proofread.strip.not_scanned', { not_scanned_count: '1' }))
    wrapper.unmount()
  })

  it('says nothing was scanned when the scope held no translated sentence', async () => {
    setEditorCaret(1)
    runMock.mockImplementation((_ids: number[], onEvent: (e: unknown) => void) => {
      onEvent({ kind: 'planned', segments: 0, batches: 0 })
      return Promise.resolve({ value: { state: 'done', not_scanned: 0 }, error: null })
    })
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1])
    await flushPromises()
    const texts = statusTexts(wrapper)
    expect(texts).toContain(t('proofread.strip.nothing_to_scan'))
    expect(texts).not.toContain(t('proofread.strip.clean'))
    wrapper.unmount()
  })

  it('shows progress while scanning', async () => {
    setEditorCaret(1)
    let emit: (e: unknown) => void = () => {}
    runMock.mockImplementation((_ids: number[], onEvent: (e: unknown) => void) => {
      emit = onEvent
      return new Promise(() => {})
    })
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    void runProofread([1, 2, 3])
    await vi.waitFor(() => {
      expect(runMock).toHaveBeenCalled()
    })
    emit({ kind: 'planned', segments: 3, batches: 2 })
    emit({ kind: 'batch', usage: null, segments: [result(1)] })
    await flushPromises()
    expect(statusTexts(wrapper)).toContain(t('proofread.strip.scanning_progress', { done_count: '1', total_count: '3' }))
    wrapper.unmount()
  })
})
