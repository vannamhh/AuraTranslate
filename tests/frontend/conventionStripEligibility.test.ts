import { afterEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

const runMock = vi.fn()
const conventionOpen = ref(false)

vi.mock('../../src/config/proofread', () => ({
  runProofreadSegments: (...args: unknown[]) => runMock(...args),
  cancelProofreadCall: () => Promise.resolve(),
  getProofreadDecisionCounts: () => Promise.resolve({ counts: { accepted: 0, not_error: 0, skipped: 0, by_kind: [] }, error: null }),
}))
vi.mock('../../src/panels/editorPanelState', async () => {
  const { ref: vueRef } = await import('vue')
  return {
    editorCaretSegmentId: vueRef<number | null>(1),
    editorSegments: vueRef([{ id: 1, target_text: 'ab' }]),
    flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
    moveEditorCaretTo: () => true,
    replaceEditorSegment: () => undefined,
  }
})
vi.mock('../../src/conventionState', () => ({ conventionStripIsEligible: conventionOpen }))
vi.mock('../../src/tmFuzzyStripState', () => ({ tmFuzzyIsEligible: ref(false), tmFuzzySegmentId: ref(null) }))
vi.mock('../../src/glossaryConfirmStripState', () => ({
  confirmStripIsOpen: ref(false),
  confirmStripNoticeShown: ref(false),
  confirmStripNoticeSegmentId: ref(null),
}))
vi.mock('../../src/glossaryQuickAddState', () => ({ quickAddIsOpen: ref(false) }))

const { runProofread, resetProofread } = await import('../../src/proofreadState')
const { eligibleInlineStrips } = await import('../../src/inlineStripEligibility')
const { topmostStrip } = await import('../../src/panels/inlineStripPriority')

afterEach(() => {
  resetProofread()
  runMock.mockReset()
  conventionOpen.value = false
})

describe('convention strip against a Proofreader scan-level state', () => {
  it('an opened convention strip is not pushed down by a scan state that has no finding at the caret', async () => {
    runMock.mockResolvedValue({ value: { state: 'not_configured' }, error: null })
    await runProofread([1])
    expect(topmostStrip(eligibleInlineStrips.value)).toBe('proofreader')
    conventionOpen.value = true
    expect(topmostStrip(eligibleInlineStrips.value)).toBe('convention')
  })
})
