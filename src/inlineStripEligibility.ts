import { computed } from 'vue'
import { confirmStripIsOpen } from './glossaryConfirmStripState'
import { quickAddIsOpen } from './glossaryQuickAddState'
import { topmostStrip } from './panels/inlineStripPriority'
import type { InlineStripKind } from './panels/inlineStripPriority'
import { editorCaretSegmentId } from './panels/editorPanelState'
import {
  proofreadActionError,
  proofreadCurrent,
  proofreadProgress,
  proofreadRevertible,
  proofreadRunSegmentIds,
  proofreadStateValue,
  proofreadStripHidden,
} from './proofreadState'
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
  if (confirmStripIsOpen.value) list.push('glossary_confirm')
  const proofread = proofreadStateValue.value
  const caret = editorCaretSegmentId.value
  const hidden = proofreadStripHidden.value && !proofreadScanLevelPresent.value
  const hasJob =
    !hidden &&
    (proofreadScanLevelPresent.value ||
    proofreadCurrent.value !== null ||
    proofreadRevertible.value ||
    proofreadActionError.value !== null)
  if (hasJob) list.push('proofreader')
  if (tmFuzzyIsEligible.value) list.push('tm_fuzzy')
  if (!hidden && proofread !== 'idle' && (caret === null || proofreadRunSegmentIds.value.includes(caret))) {
    list.push('proofreader_summary')
  }
  return list
})

export type ProofreaderStripPlace = 'foot' | 'track'

/** Where the Proofreader strip renders: window foot for scan-level state, grid track for the caret sentence, `null` when it has no home. */
export const proofreaderStripPlace = computed<ProofreaderStripPlace | null>(() => {
  const top = topmostStrip(eligibleInlineStrips.value)
  if (top !== 'proofreader' && top !== 'proofreader_summary') return null
  if (proofreadScanLevelPresent.value) return 'foot'
  return editorCaretSegmentId.value === null ? null : 'track'
})

/** Segment whose row the winning strip grows under; `null` when no strip takes a grid track. */
export const inlineStripAnchorSegmentId = computed<number | null>(() => {
  switch (topmostStrip(eligibleInlineStrips.value)) {
    case 'tm_fuzzy':
      return tmFuzzySegmentId.value
    case 'glossary_confirm':
      return editorCaretSegmentId.value
    case 'proofreader':
    case 'proofreader_summary':
      return proofreaderStripPlace.value === 'track' ? editorCaretSegmentId.value : null
    default:
      return null
  }
})
