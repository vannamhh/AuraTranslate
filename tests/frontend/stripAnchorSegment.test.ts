import { afterEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

const tmEligible = ref(false)
const tmSegmentId = ref<number | null>(null)
const confirmOpen = ref(false)
const quickAddOpen = ref(false)
const caret = ref<number | null>(null)
const proofreadState = ref('idle')
const runIds = ref<number[]>([])
const progress = ref({ done: 0, total: 1 })

vi.mock('../../src/tmFuzzyStripState', () => ({ tmFuzzyIsEligible: tmEligible, tmFuzzySegmentId: tmSegmentId }))
vi.mock('../../src/glossaryConfirmStripState', () => ({ confirmStripIsOpen: confirmOpen }))
vi.mock('../../src/glossaryQuickAddState', () => ({ quickAddIsOpen: quickAddOpen }))
vi.mock('../../src/panels/editorPanelState', () => ({ editorCaretSegmentId: caret }))
vi.mock('../../src/proofreadState', () => ({
  proofreadActionError: ref(null),
  proofreadCurrent: ref(null),
  proofreadProgress: progress,
  proofreadRevertible: ref(false),
  proofreadRunSegmentIds: runIds,
  proofreadStateValue: proofreadState,
  proofreadStripHidden: ref(false),
}))

const { inlineStripAnchorSegmentId, proofreaderStripPlace } = await import('../../src/inlineStripEligibility')

afterEach(() => {
  tmEligible.value = false
  tmSegmentId.value = null
  confirmOpen.value = false
  quickAddOpen.value = false
  caret.value = null
  proofreadState.value = 'idle'
  runIds.value = []
  progress.value = { done: 0, total: 1 }
})

describe('inlineStripAnchorSegmentId', () => {
  it('is null when no strip is eligible', () => {
    expect(inlineStripAnchorSegmentId.value).toBeNull()
  })

  it('anchors the TM strip at tmFuzzySegmentId, not at the caret', () => {
    tmEligible.value = true
    tmSegmentId.value = 5
    caret.value = 9
    expect(inlineStripAnchorSegmentId.value).toBe(5)
  })

  it('anchors the Glossary confirm strip at the caret and wins over TM', () => {
    tmEligible.value = true
    tmSegmentId.value = 5
    confirmOpen.value = true
    caret.value = 9
    expect(inlineStripAnchorSegmentId.value).toBe(9)
  })

  it('takes no track while Quick Add wins', () => {
    tmEligible.value = true
    tmSegmentId.value = 5
    confirmOpen.value = true
    quickAddOpen.value = true
    caret.value = 5
    expect(inlineStripAnchorSegmentId.value).toBeNull()
  })

  it('takes no track for the Proofreader summary without a caret', () => {
    proofreadState.value = 'done'
    caret.value = null
    expect(inlineStripAnchorSegmentId.value).toBeNull()
  })

  it('anchors the Proofreader summary at the caret when it sits in the run', () => {
    proofreadState.value = 'done'
    runIds.value = [3]
    caret.value = 3
    expect(inlineStripAnchorSegmentId.value).toBe(3)
  })

  it('puts a scan in progress at the window foot and takes no track, caret or not', () => {
    proofreadState.value = 'scanning'
    caret.value = 4
    expect(proofreaderStripPlace.value).toBe('foot')
    expect(inlineStripAnchorSegmentId.value).toBeNull()
    caret.value = null
    expect(proofreaderStripPlace.value).toBe('foot')
  })

  it.each(['error', 'not_configured'])('puts the %s job at the window foot without a caret', (state) => {
    proofreadState.value = state
    expect(proofreaderStripPlace.value).toBe('foot')
  })

  it('puts a finished scan with nothing to scan at the foot', () => {
    proofreadState.value = 'done'
    progress.value = { done: 0, total: 0 }
    caret.value = 2
    expect(proofreaderStripPlace.value).toBe('foot')
    expect(inlineStripAnchorSegmentId.value).toBeNull()
  })

  it('puts the finished scan under the caret sentence and nowhere without a caret', () => {
    proofreadState.value = 'done'
    runIds.value = [3]
    caret.value = 3
    expect(proofreaderStripPlace.value).toBe('track')
    caret.value = null
    expect(proofreaderStripPlace.value).toBeNull()
  })

  it('has no Proofreader place while another strip wins', () => {
    proofreadState.value = 'scanning'
    quickAddOpen.value = true
    expect(proofreaderStripPlace.value).toBeNull()
  })
})
