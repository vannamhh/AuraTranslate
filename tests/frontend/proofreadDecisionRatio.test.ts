import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import { flushPromises } from './support/flushMicrotasks'

const TEXT = 'toi nhin thay sai'
const GRAMMAR = { kind: 'grammar', start: 4, end: 8, explanation: 'e', suggestion: 'nhìn' }
const SPELLING = { kind: 'spelling', start: 14, end: 17, explanation: 'e', suggestion: 'sái' }
const COUNT_ERROR = { code: 'store.read_failed', message_key: 'err.store.read_failed', params: { store: 'project' }, retryable: true }

type Row = { id: number; kind: string; outcome: 'accepted' | 'not_error' | 'skipped' }

async function fresh() {
  vi.resetModules()
  const rows: Row[] = []
  let nextId = 1
  let countsFail = false
  const invoke = vi.fn((cmd: string, args?: Record<string, unknown>): Promise<unknown> => {
    if (cmd === 'ai_proofread_accept') {
      const row: Row = { id: nextId++, kind: String(args?.kind), outcome: 'accepted' }
      rows.push(row)
      return Promise.resolve({
        segment_id: 5,
        target_text: 'toi nhìn thay sai',
        translation_origin: 'self',
        status: 'draft',
        needs_confirmation: false,
        unsigned_draft: null,
        decision_id: row.id,
      })
    }
    if (cmd === 'ai_proofread_revert') {
      const at = rows.findIndex((r) => r.id === args?.decisionId && r.outcome === 'accepted')
      if (at < 0) return Promise.reject({ code: 'ai_proofread.decision_missing', message_key: 'err.ai_proofread.decision_missing', params: {}, retryable: false })
      rows.splice(at, 1)
      return Promise.resolve({ segment_id: 5, target_text: TEXT, translation_origin: 'self', status: 'draft', needs_confirmation: false, unsigned_draft: null })
    }
    if (cmd === 'ai_proofread_decision_skip') {
      rows.push({ id: nextId++, kind: String(args?.kind), outcome: 'skipped' })
      return Promise.resolve(null)
    }
    if (cmd === 'ai_proofread_ignore_add') {
      rows.push({ id: nextId++, kind: String(args?.kind), outcome: 'not_error' })
      return Promise.resolve(null)
    }
    if (cmd === 'ai_proofread_decision_counts') {
      if (countsFail) return Promise.reject(COUNT_ERROR)
      const n = (outcome: Row['outcome']): number => rows.filter((r) => r.outcome === outcome).length
      return Promise.resolve({ accepted: n('accepted'), not_error: n('not_error'), skipped: n('skipped'), by_kind: [] })
    }
    return Promise.resolve(null)
  })
  vi.doMock('@tauri-apps/api/core', () => ({
    invoke,
    Channel: class {
      onmessage: (v: unknown) => void = () => {}
    },
  }))
  const segments = ref([{ id: 5, target_text: TEXT, translation_origin: 'self', status: 'draft' }])
  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: ref<number | null>(5),
    editorLastCaretSegmentId: ref<number | null>(5),
    editorSegments: segments,
    flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
    replaceEditorSegment: (id: number, patch: Record<string, unknown>) => {
      segments.value = segments.value.map((x) => (x.id === id ? { ...x, ...patch } : x))
    },
    moveEditorCaretTo: () => {},
  }))
  vi.doMock('../../src/panels/segmentSelectionState', () => ({ segmentSelectionIds: ref<number[]>([]) }))
  const state = await import('../../src/proofreadState')
  const decisions = await import('../../src/proofreadDecisionState')
  const Ratio = (await import('../../src/ProofreadDecisionRatio.vue')).default
  const cfg = await import('../../src/config/proofread')
  vi.spyOn(cfg, 'runProofreadSegments').mockImplementation(((_ids: number[], onEvent: (e: unknown) => void) => {
    onEvent({ kind: 'planned', segments: 1, batches: 1 })
    onEvent({ kind: 'batch', usage: null, segments: [{ segment_id: 5, scanned_text: TEXT, findings: [GRAMMAR, SPELLING], unlocated: 0, ambiguous: 0 }] })
    return Promise.resolve({ value: { state: 'done', not_scanned: 0, filtered: 0, blocked: [] }, error: null })
  }) as never)
  return { state, decisions, Ratio, invoke, failCounts: () => (countsFail = true), healCounts: () => (countsFail = false) }
}

function tally(decisions: Awaited<ReturnType<typeof fresh>>['decisions']): [number, number, number] | string {
  const ratio = decisions.proofreadDecisionRatio.value
  if (ratio === null) return 'unread'
  if (ratio.kind === 'none') return 'none'
  return [ratio.accepted, ratio.notError, ratio.skipped]
}

afterEach(() => {
  document.body.innerHTML = ''
  vi.restoreAllMocks()
})

describe('proofreader decision ratio (FR150, AD-56)', () => {
  it('accepting one finding and skipping another reads 1 - 0 - 1, and still does after reopening the Work', async () => {
    const { state, decisions } = await fresh()
    await state.runProofread([5], 'segment')
    await state.acceptProofreadFinding()
    await state.dismissProofreadFinding()
    await flushPromises()
    expect(tally(decisions)).toEqual([1, 0, 1])
    expect(decisions.proofreadDecisionRatio.value).toMatchObject({ acceptedPercent: 50, notErrorPercent: 0, skippedPercent: 50 })

    state.resetProofread()
    expect(tally(decisions)).toBe('unread')
    decisions.ensureProofreadDecisionsLoaded()
    await flushPromises()
    expect(tally(decisions)).toEqual([1, 0, 1])
  })

  it('reverting the accepted finding hands its decision id back and the ratio reads 0 - 0 - 1', async () => {
    const { state, decisions, invoke } = await fresh()
    await state.runProofread([5], 'segment')
    await state.acceptProofreadFinding()
    await state.dismissProofreadFinding()
    await state.revertProofreadFinding()
    await flushPromises()
    const revert = invoke.mock.calls.find((c) => c[0] === 'ai_proofread_revert')
    expect(revert?.[1]).toMatchObject({ decisionId: 1, kind: 'grammar' })
    expect(tally(decisions)).toEqual([0, 0, 1])
  })

  it('each decision reloads the ratio on its own: accept alone, then "not an error" alone', async () => {
    const { state, decisions } = await fresh()
    await state.runProofread([5], 'segment')
    await state.acceptProofreadFinding()
    await flushPromises()
    expect(tally(decisions)).toEqual([1, 0, 0])
    await state.ignoreProofreadFinding()
    await flushPromises()
    expect(tally(decisions)).toEqual([1, 1, 0])
  })

  it('a Work without any decision reads "none" and shows the words, never 0%', async () => {
    const { decisions, Ratio } = await fresh()
    const wrapper = mount(Ratio, { props: { active: true } })
    await flushPromises()
    expect(tally(decisions)).toBe('none')
    expect(wrapper.get('[data-proofread-ratio]').attributes('data-proofread-ratio')).toBe('none')
    expect(wrapper.text()).toContain('Chưa có quyết định')
    expect(wrapper.text()).not.toContain('%')
  })

  it('shows nothing, and reads nothing, until the component is active', async () => {
    const { decisions, Ratio, invoke } = await fresh()
    const wrapper = mount(Ratio, { props: { active: false } })
    await flushPromises()
    expect(tally(decisions)).toBe('unread')
    expect(wrapper.find('[data-proofread-ratio]').exists()).toBe(false)
    expect(invoke.mock.calls.filter((c) => c[0] === 'ai_proofread_decision_counts')).toHaveLength(0)
  })

  it('a failed read shows the error line and no number', async () => {
    const { decisions, Ratio, failCounts } = await fresh()
    failCounts()
    const wrapper = mount(Ratio, { props: { active: true } })
    await flushPromises()
    expect(decisions.proofreadDecisionRatio.value).toBeNull()
    expect(wrapper.get('[data-proofread-ratio]').attributes('data-proofread-ratio')).toBe('error')
    expect(wrapper.text()).not.toMatch(/\d/)
  })

  it('does not retry a failed read on every render, but a decision reloads it', async () => {
    const { state, decisions, Ratio, invoke, failCounts, healCounts } = await fresh()
    failCounts()
    mount(Ratio, { props: { active: true } })
    await flushPromises()
    decisions.ensureProofreadDecisionsLoaded()
    await flushPromises()
    expect(invoke.mock.calls.filter((c) => c[0] === 'ai_proofread_decision_counts')).toHaveLength(1)
    healCounts()
    await state.runProofread([5], 'segment')
    await state.dismissProofreadFinding()
    await flushPromises()
    expect(tally(decisions)).toEqual([0, 0, 1])
  })

  it('a skip Rust refused neither removes the finding nor reloads the ratio', async () => {
    const { state, invoke } = await fresh()
    await state.runProofread([5], 'segment')
    const reads = (): number => invoke.mock.calls.filter((c) => c[0] === 'ai_proofread_decision_counts').length
    const before = reads()
    const impl = invoke.getMockImplementation()
    invoke.mockImplementation((cmd: string, args?: Record<string, unknown>) =>
      cmd === 'ai_proofread_decision_skip' ? Promise.reject(COUNT_ERROR) : (impl as (c: string, a?: Record<string, unknown>) => Promise<unknown>)(cmd, args),
    )
    await state.dismissProofreadFinding()
    await flushPromises()
    expect(state.proofreadFindingCount.value).toBe(2)
    expect(reads()).toBe(before)
  })
})

describe('ratio percentages', () => {
  it('always sum to 100: thirds read 34 · 33 · 33, not 33 · 33 · 33', async () => {
    vi.resetModules()
    const { wholePercents } = await import('../../src/proofreadDecisionState')
    expect(wholePercents([1, 1, 1], 3)).toEqual([34, 33, 33])
    expect(wholePercents([1, 1, 4], 6)).toEqual([17, 17, 66])
    expect(wholePercents([2, 0, 1], 3)).toEqual([67, 0, 33])
    for (const counts of [[1, 2, 4], [5, 5, 1], [7, 0, 0], [3, 3, 3]]) {
      const total = counts.reduce((sum, n) => sum + n, 0)
      expect(wholePercents(counts, total).reduce((sum, n) => sum + n, 0)).toBe(100)
    }
  })
})
