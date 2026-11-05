import { computed } from 'vue'
import { confirmStripIsOpen, confirmStripNoticeSegmentId, confirmStripNoticeShown } from './glossaryConfirmStripState'
import { quickAddIsOpen } from './glossaryQuickAddState'
import { topmostStrip } from './panels/inlineStripPriority'
import type { InlineStripKind } from './panels/inlineStripPriority'
import { editorCaretSegmentId } from './panels/editorPanelState'
import {
  proofreadActionError,
  proofreadCurrent,
  proofreadProgress,
  proofreadReportedFindings,
  proofreadResults,
  proofreadRevertible,
  proofreadRunSegmentIds,
  proofreadStateValue,
  proofreadStripHidden,
} from './proofreadState'
import { conventionStripIsEligible } from './conventionState'
import { tmFuzzyIsEligible, tmFuzzySegmentId } from './tmFuzzyStripState'

const proofreadScanLevelPresent = computed(() => {
  const proofread = proofreadStateValue.value
  return (
    proofread === 'scanning' ||
    proofread === 'error' ||
    proofread === 'not_configured' ||
    (proofread === 'done' && proofreadProgress.value.total === 0)
  )
})

/** Every inline strip that could render now; `topmostStrip` picks the one that does. */
export const eligibleInlineStrips = computed<InlineStripKind[]>(() => {
  const list: InlineStripKind[] = []
  if (quickAddIsOpen.value) list.push('glossary_quick_add')
  if (confirmStripIsOpen.value || confirmStripNoticeShown.value) list.push('glossary_confirm')
  const proofread = proofreadStateValue.value
  const caret = editorCaretSegmentId.value
  const hidden = proofreadStripHidden.value && !proofreadScanLevelPresent.value
  const ownJob =
    proofreadCurrent.value !== null || proofreadRevertible.value || proofreadActionError.value !== null
  const scanLevel = proofreadScanLevelPresent.value && !(conventionStripIsEligible.value && !ownJob)
  const hasJob = !hidden && (scanLevel || ownJob)
  if (hasJob) list.push('proofreader')
  if (conventionStripIsEligible.value) list.push('convention')
  if (tmFuzzyIsEligible.value) list.push('tm_fuzzy')
  if (!hidden && proofread !== 'idle' && (caret === null || summaryBelongsAt(caret))) {
    list.push('proofreader_summary')
  }
  return list
})

/** A multi-sentence scan that reported findings speaks only at the sentences still holding one. */
function summaryBelongsAt(caret: number): boolean {
  const run = proofreadRunSegmentIds.value
  if (!run.includes(caret)) return false
  if (run.length === 1 || proofreadReportedFindings.value === 0) return true
  return (proofreadResults.value.get(caret)?.findings.length ?? 0) > 0
}

export type ProofreaderStripPlace = 'foot' | 'track'

/** Where the Proofreader strip renders: window foot for scan-level state, grid track for the caret sentence, `null` when it has no home. */
export const proofreaderStripPlace = computed<ProofreaderStripPlace | null>(() => {
  const top = topmostStrip(eligibleInlineStrips.value)
  if (top === 'convention') return editorCaretSegmentId.value === null ? null : 'track'
  if (top !== 'proofreader' && top !== 'proofreader_summary') return null
  if (proofreadScanLevelPresent.value) return 'foot'
  return editorCaretSegmentId.value === null ? null : 'track'
})

/** The TM and Proofreader strips float over the rows below their anchor instead of pushing them down. */
export const stripTrackFloats = computed(() => {
  const top = topmostStrip(eligibleInlineStrips.value)
  return top === 'tm_fuzzy' || top === 'proofreader' || top === 'convention' || top === 'proofreader_summary'
})

/** Segment whose row the winning strip grows under; `null` when no strip takes a grid track. */
export const inlineStripAnchorSegmentId = computed<number | null>(() => {
  switch (topmostStrip(eligibleInlineStrips.value)) {
    case 'tm_fuzzy':
      return tmFuzzySegmentId.value
    case 'glossary_confirm':
      return confirmStripIsOpen.value ? editorCaretSegmentId.value : confirmStripNoticeSegmentId.value
    case 'proofreader':
    case 'convention':
    case 'proofreader_summary':
      return proofreaderStripPlace.value === 'track' ? editorCaretSegmentId.value : null
    default:
      return null
  }
})
