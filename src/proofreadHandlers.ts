import { aiTranslateBatchStateValue } from './aiTranslateBatchState'
import { aiTranslateStateValue } from './aiTranslateState'
import { editorCaretSegmentId } from './panels/editorPanelState'
import { cancelProofread, runProofread } from './proofreadState'

export const proofreadHandlers = {
  runAiProofread: (): void => {
    if (aiTranslateStateValue.value === 'generating' || aiTranslateBatchStateValue.value === 'generating') {
      console.warn('[proofread] not scanning: a translation is running')
      return
    }
    void runProofread(editorCaretSegmentId.value)
  },

  cancelAiProofread: (): void => {
    cancelProofread()
  },
}
