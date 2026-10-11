import { computed } from 'vue'
import { confirmStripIsOpen } from './glossaryConfirmStripState'
import { quickAddIsOpen } from './glossaryQuickAddState'
import type { InlineStripKind } from './panels/inlineStripPriority'
import { editorCaretSegmentId } from './panels/editorPanelState'
import {
  proofreadActionError,
  proofreadCurrent,
  proofreadProgress,
  proofreadRevertible,
  proofreadRunSegmentIds,
  proofreadStateValue,
} from './proofreadState'
import { tmFuzzyIsEligible } from './tmFuzzyStripState'

/** Every inline strip that could render now; `topmostStrip` picks the one that does. */
export const eligibleInlineStrips = computed<InlineStripKind[]>(() => {
  const list: InlineStripKind[] = []
  if (quickAddIsOpen.value) list.push('glossary_quick_add')
  if (confirmStripIsOpen.value) list.push('glossary_confirm')
  const proofread = proofreadStateValue.value
  const caret = editorCaretSegmentId.value
  const hasJob =
    proofread === 'scanning' ||
    proofread === 'error' ||
    proofread === 'not_configured' ||
    (proofread === 'done' && proofreadProgress.value.total === 0) ||
    proofreadCurrent.value !== null ||
    proofreadRevertible.value ||
    proofreadActionError.value !== null
  if (hasJob) list.push('proofreader')
  if (tmFuzzyIsEligible.value) list.push('tm_fuzzy')
  if (proofread !== 'idle' && (caret === null || proofreadRunSegmentIds.value.includes(caret))) {
    list.push('proofreader_summary')
  }
  return list
})
