import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { ProofreadFindingWire } from '../../src/config/proofread'
import type { IpcError } from '../../src/i18n'
import { flushPromises } from './support/flushMicrotasks'

const runMock = vi.fn()
const acceptMock = vi.fn()
const flushMock = vi.fn()
const dispatchMock = vi.fn()
const tmEligible = ref(false)
const confirmOpen = ref(false)
const quickAddOpen = ref(false)

vi.mock('../../src/config/proofread', () => ({
  runProofreadSegments: (...args: unknown[]) => runMock(...args),
  cancelProofreadCall: () => Promise.resolve(),
  acceptProofreadSuggestion: (...args: unknown[]) => acceptMock(...args),
  revertProofreadSuggestion: () => Promise.resolve({ outcome: null, error: null }),
}))
vi.mock('../../src/panels/editorPanelState', async () => {
  const { ref: vueRef } = await import('vue')
  const editorSegments = vueRef<{ id: number; target_text: string }[]>([])
  return {
    editorCaretSegmentId: vueRef<number | null>(null),
    editorSegments,
    flushEditorBeforeDiscreteWrite: () => flushMock(),
    moveEditorCaretTo: () => true,
    replaceEditorSegment: (id: number, patch: Record<string, unknown>) => {
      editorSegments.value = editorSegments.value.map((s) => (s.id === id ? { ...s, ...patch } : s))
    },
  }
})
vi.mock('../../src/commands', () => ({ dispatch: (...args: unknown[]) => dispatchMock(...args) }))
vi.mock('../../src/tmFuzzyStripState', () => ({ tmFuzzyIsEligible: tmEligible }))
vi.mock('../../src/glossaryConfirmStripState', () => ({ confirmStripIsOpen: confirmOpen }))
vi.mock('../../src/glossaryQuickAddState', () => ({ quickAddIsOpen: quickAddOpen }))

const editor = await import('../../src/panels/editorPanelState')
const { runProofread, resetProofread, acceptProofreadFinding } = await import('../../src/proofreadState')
const { eligibleInlineStrips } = await import('../../src/inlineStripEligibility')
const { topmostStrip } = await import('../../src/panels/inlineStripPriority')
const { t, tError } = await import('../../src/i18n')
const caretRef = editor.editorCaretSegmentId as unknown as { value: number | null }
const segmentsRef = editor.editorSegments as unknown as { value: unknown[] }
const ProofreaderStrip = (await import('../../src/ProofreaderStrip.vue')).default

const TEXT = 'ab cd'
const FINDING: ProofreadFindingWire = { kind: 'spelling', start: 0, end: 2, explanation: 'Cụm này viết sai chính tả.', suggestion: 'AB' }
const usage = { prompt_tokens: 10, completion_tokens: 0, total_tokens: 10, cost_usd: 0.001 }
const seg = (findings: ProofreadFindingWire[]) => ({ segment_id: 1, scanned_text: TEXT, findings, unlocated: 0, ambiguous: 0 })

function scanResult(findings: ProofreadFindingWire[], outcome: unknown = { state: 'done', not_scanned: 0, filtered: 0 }): void {
  runMock.mockImplementation((_ids: number[], onEvent: (e: unknown) => void) => {
    onEvent({ kind: 'planned', segments: 1, batches: 1 })
    onEvent({ kind: 'batch', usage, segments: [seg(findings)] })
    return Promise.resolve({ value: outcome, error: null })
  })
}

const strips = () => eligibleInlineStrips.value
const top = () => topmostStrip(eligibleInlineStrips.value)
const texts = (w: ReturnType<typeof mount>) => w.findAll('p').map((p) => p.text())

beforeEach(() => {
  flushMock.mockResolvedValue('clean')
  segmentsRef.value = [{ id: 1, target_text: TEXT }] as never
  caretRef.value = 1
})

afterEach(() => {
  resetProofread()
  runMock.mockReset()
  acceptMock.mockReset()
  flushMock.mockReset()
  dispatchMock.mockReset()
  tmEligible.value = false
  confirmOpen.value = false
  quickAddOpen.value = false
})

describe('strip content', () => {
  it('shows kind, phrase, explanation, suggestion and "n / m" without blaming the user', async () => {
    scanResult([FINDING, { ...FINDING, start: 3, end: 5, suggestion: 'CD' }])
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1])
    await flushPromises()
    const text = wrapper.text()
    expect(text).toContain(t('proofread.kind.spelling'))
    expect(text).toContain('ab')
    expect(text).toContain(FINDING.explanation)
    expect(text).toContain(t('proofread.strip.suggestion', { suggestion_text: 'AB' }))
    expect(text).toContain(t('proofread.strip.position', { finding_index: '1', finding_count: '2' }))
    wrapper.unmount()
  })

  it('offers no accept-all control or command', async () => {
    scanResult([FINDING, { ...FINDING, start: 3, end: 5 }])
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1])
    await flushPromises()
    for (const button of wrapper.findAll('button')) await button.trigger('click')
    const ids = dispatchMock.mock.calls.map((call) => String(call[0]))
    expect(ids.length).toBeGreaterThan(0)
    expect(ids.filter((id) => /all/i.test(id))).toEqual([])
    expect(wrapper.text().toLowerCase()).not.toContain('tất cả')
    wrapper.unmount()
  })

  it('dismiss and accept buttons dispatch their registered commands', async () => {
    scanResult([FINDING])
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1])
    await flushPromises()
    const labels = new Map(wrapper.findAll('button').map((b) => [b.text(), b]))
    await labels.get(t('command.ai.proofread.accept'))?.trigger('click')
    await labels.get(t('command.ai.proofread.dismiss'))?.trigger('click')
    expect(dispatchMock.mock.calls.map((c) => c[0])).toEqual(['ai.proofread.accept', 'ai.proofread.dismiss'])
    wrapper.unmount()
  })

  it('dismiss writes nothing and the card goes away at once', async () => {
    scanResult([FINDING])
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1])
    await flushPromises()
    const { dismissProofreadFinding, proofreadFindingCount } = await import('../../src/proofreadState')
    dismissProofreadFinding()
    await flushPromises()
    expect(proofreadFindingCount.value).toBe(0)
    expect(wrapper.find('.pfs-card').exists()).toBe(false)
    expect(acceptMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('a flush failure on accept reports its own message, never "not scanned"', async () => {
    scanResult([FINDING])
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1])
    flushMock.mockResolvedValue('still-dirty')
    await acceptProofreadFinding()
    await flushPromises()
    const alert = wrapper.find('[role="alert"]').text()
    expect(alert).toBe(tError({ code: '', message_key: 'proofread.accept_flush_still_dirty', params: {}, retryable: true } as IpcError))
    expect(alert).not.toBe(t('proofread.flush_still_dirty'))
    expect(alert.toLowerCase()).not.toContain('chưa quét')
    wrapper.unmount()
  })
})

describe('slot: Proofreader around TM', () => {
  it('a clean sentence at the caret with nothing from TM shows the clean summary', async () => {
    scanResult([])
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1])
    await flushPromises()
    expect(top()).toBe('proofreader_summary')
    expect(texts(wrapper)).toContain(t('proofread.strip.clean'))
    wrapper.unmount()
  })

  it('a cancel with nothing from TM keeps "cancelled" and the usage total', async () => {
    scanResult([], { state: 'cancelled', not_scanned: 0 })
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1])
    await flushPromises()
    expect(top()).toBe('proofreader_summary')
    const shown = texts(wrapper)
    expect(shown).toContain(t('proofread.strip.cancelled'))
    expect(shown.some((s) => s.includes('10'))).toBe(true)
    wrapper.unmount()
  })

  it('a cancel with a pending finding at the caret still shows "cancelled" beside the card', async () => {
    scanResult([FINDING], { state: 'cancelled', not_scanned: 0 })
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread([1])
    await flushPromises()
    expect(top()).toBe('proofreader')
    expect(texts(wrapper)).toContain(t('proofread.strip.cancelled'))
    wrapper.unmount()
  })

  it('TM beats the summary when the caret sentence has no pending finding', async () => {
    scanResult([])
    tmEligible.value = true
    await runProofread([1])
    expect(strips()).toContain('proofreader_summary')
    expect(top()).toBe('tm_fuzzy')
  })

  it('a pending finding at the caret beats TM', async () => {
    scanResult([FINDING])
    tmEligible.value = true
    await runProofread([1])
    expect(top()).toBe('proofreader')
  })

  it('Glossary beats Proofreader, Proofreader beats TM, TM is hidden', async () => {
    scanResult([FINDING])
    tmEligible.value = true
    confirmOpen.value = true
    await runProofread([1])
    expect(strips()).toEqual(expect.arrayContaining(['glossary_confirm', 'proofreader', 'tm_fuzzy']))
    expect(top()).toBe('glossary_confirm')
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    expect(wrapper.find('.proofreader-strip').exists()).toBe(false)
    confirmOpen.value = false
    await flushPromises()
    expect(top()).toBe('proofreader')
    wrapper.unmount()
  })

  it('with TM absent the summary is the only eligible strip', async () => {
    scanResult([])
    await runProofread([1])
    expect(strips()).toEqual(['proofreader_summary'])
  })
})

describe('slot: job strip keeps its place ahead of TM', () => {
  it('a failed scan', async () => {
    runMock.mockResolvedValue({ value: null, error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: false } })
    tmEligible.value = true
    await runProofread([1])
    expect(top()).toBe('proofreader')
  })

  it('AI not configured', async () => {
    runMock.mockResolvedValue({ value: { state: 'not_configured' }, error: null })
    tmEligible.value = true
    await runProofread([1])
    expect(top()).toBe('proofreader')
  })

  it('a scan of zero sentences', async () => {
    runMock.mockImplementation((_ids: number[], onEvent: (e: unknown) => void) => {
      onEvent({ kind: 'planned', segments: 0, batches: 0 })
      return Promise.resolve({ value: { state: 'done', not_scanned: 0, filtered: 0 }, error: null })
    })
    tmEligible.value = true
    await runProofread([1])
    expect(top()).toBe('proofreader')
  })

  it('a scan still running', async () => {
    runMock.mockReturnValue(new Promise(() => {}))
    tmEligible.value = true
    void runProofread([1])
    await vi.waitFor(() => {
      expect(runMock).toHaveBeenCalled()
    })
    expect(top()).toBe('proofreader')
  })

  it('an accepted suggestion that can still be reverted', async () => {
    scanResult([FINDING])
    await runProofread([1])
    acceptMock.mockResolvedValue({
      outcome: { segment_id: 1, target_text: 'AB cd', translation_origin: 'ai_proofread', status: 'draft', needs_confirmation: false, unsigned_draft: null },
      error: null,
    })
    await acceptProofreadFinding()
    tmEligible.value = true
    expect(top()).toBe('proofreader')
  })

  it('an action that just failed', async () => {
    scanResult([FINDING])
    await runProofread([1])
    acceptMock.mockResolvedValue({ outcome: null, error: { code: 'ai_proofread.text_changed', message_key: 'err.ai_proofread.text_changed', params: { segment_id: '1' }, retryable: false } })
    await acceptProofreadFinding()
    editor.replaceEditorSegment(1, { target_text: 'changed' })
    tmEligible.value = true
    expect(top()).toBe('proofreader')
  })
})

describe('placement', () => {
  it('renders a running scan only in the window-foot instance, with Cancel, even without a caret', async () => {
    runMock.mockImplementation((_ids: number[], onEvent: (e: unknown) => void) => {
      onEvent({ kind: 'planned', segments: 1, batches: 1 })
      return new Promise(() => {})
    })
    const foot = mount(ProofreaderStrip, { props: { place: 'foot' }, attachTo: document.body })
    const track = mount(ProofreaderStrip, { props: { place: 'track' }, attachTo: document.body })
    void runProofread([1])
    await flushPromises()
    caretRef.value = null
    await flushPromises()
    expect(foot.find('.proofreader-strip').exists()).toBe(true)
    expect(track.find('.proofreader-strip').exists()).toBe(false)
    for (const button of foot.findAll('button')) await button.trigger('click')
    expect(dispatchMock.mock.calls.map((c) => c[0])).toContain('ai.proofread.cancel')
    foot.unmount()
    track.unmount()
  })

  it('renders the caret sentence finding only in the track instance once the scan is done', async () => {
    scanResult([FINDING])
    const foot = mount(ProofreaderStrip, { props: { place: 'foot' }, attachTo: document.body })
    const track = mount(ProofreaderStrip, { props: { place: 'track' }, attachTo: document.body })
    await runProofread([1])
    await flushPromises()
    expect(track.text()).toContain(FINDING.explanation)
    expect(foot.find('.proofreader-strip').exists()).toBe(false)
    foot.unmount()
    track.unmount()
  })
})
