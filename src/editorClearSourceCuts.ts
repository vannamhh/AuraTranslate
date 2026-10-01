import { confirmStripIsOpen } from './glossaryConfirmStripState'
import { quickAddIsOpen } from './glossaryQuickAddState'
import { clearEditorSourceCut, editorPendingPromote } from './panels/editorPanelState'
import { historyIsOpen } from './panels/segmentHistoryState'

/**
 * Guards the bare `Escape` chord against clearing pointer cuts while a Glossary strip, the
 * history overlay or the PROMOTE question is open: a `<button>` inside any of them isn't a
 * typing zone, so `Escape` there would otherwise both close the surface and silently wipe
 * the cut set.
 *
 * The test is "open", not `captureIsArmed`. `historyIsOpen` stays out of the global
 * `isBlocked()` in `main.ts` on purpose: an open history overlay blocks only this command,
 * not every chord.
 */
export function clearSourceCuts(): void {
  if (
    quickAddIsOpen.value ||
    confirmStripIsOpen.value ||
    historyIsOpen.value ||
    editorPendingPromote.value !== null
  ) {
    return
  }
  clearEditorSourceCut()
}
