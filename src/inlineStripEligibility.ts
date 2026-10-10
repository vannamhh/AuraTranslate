import { computed } from 'vue'
import { confirmStripIsOpen } from './glossaryConfirmStripState'
import { quickAddIsOpen } from './glossaryQuickAddState'
import type { InlineStripKind } from './panels/inlineStripPriority'
import { editorCaretSegmentId } from './panels/editorPanelState'
import { proofreadRunSegmentId, proofreadStateValue } from './proofreadState'
import { tmFuzzyIsEligible } from './tmFuzzyStripState'

/** Every inline strip that could render now; `topmostStrip` picks the one that does. */
export const eligibleInlineStrips = computed<InlineStripKind[]>(() => {
  const list: InlineStripKind[] = []
  if (quickAddIsOpen.value) list.push('glossary_quick_add')
  if (confirmStripIsOpen.value) list.push('glossary_confirm')
  const proofread = proofreadStateValue.value
  if (proofread === 'scanning' || (proofread !== 'idle' && proofreadRunSegmentId.value === editorCaretSegmentId.value)) {
    list.push('proofreader')
  }
  if (tmFuzzyIsEligible.value) list.push('tm_fuzzy')
  return list
})
