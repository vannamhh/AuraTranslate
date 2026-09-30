import { confirmStripIsOpen } from './glossaryConfirmStripState'
import { quickAddIsOpen } from './glossaryQuickAddState'
import { clearEditorSourceCut } from './panels/editorPanelState'
import { historyIsOpen } from './panels/segmentHistoryState'
import { settingsOverlayIsOpen } from './settingsState'

/**
 * Guards the bare `Escape` chord against clearing pointer cuts while a Glossary strip is
 * open: a `<button>` inside a strip isn't a typing zone, so `Escape` there would otherwise
 * both close the strip and silently wipe the cut set.
 *
 * The same holds for `SegmentHistoryOverlay` and Settings, and the test is "open", not
 * `captureIsArmed`: `Escape` on any button inside an open overlay must not wipe the cuts.
 * `historyIsOpen` stays out of the global `isBlocked()` in `main.ts` on purpose: an open
 * history overlay blocks only this command, not every chord.
 */
export function clearSourceCuts(): void {
  if (
    quickAddIsOpen.value ||
    confirmStripIsOpen.value ||
    historyIsOpen.value ||
    settingsOverlayIsOpen.value
  ) {
    return
  }
  clearEditorSourceCut()
}
