import { confirmStripIsOpen } from './glossaryConfirmStripState'
import { quickAddIsOpen } from './glossaryQuickAddState'
import { clearEditorSourceCut } from './panels/editorPanelState'

/**
 * Guards the bare `Escape` chord against clearing pointer cuts while a Glossary strip is
 * open: a `<button>` inside a strip isn't a typing zone, so `Escape` there would otherwise
 * both close the strip and silently wipe the cut set.
 */
export function clearSourceCuts(): void {
  if (quickAddIsOpen.value || confirmStripIsOpen.value) return
  clearEditorSourceCut()
}
