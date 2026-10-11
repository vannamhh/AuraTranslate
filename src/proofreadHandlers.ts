import { aiTranslateBatchStateValue } from './aiTranslateBatchState'
import { aiTranslateStateValue } from './aiTranslateState'
import { proofreadScopeIds } from './proofreadScope'
import type { ProofreadScopeKind } from './proofreadScope'
import { cancelProofread, closeProofreadScope, openProofreadScope, runProofread } from './proofreadState'

function scan(kind: ProofreadScopeKind): void {
  closeProofreadScope()
  if (aiTranslateStateValue.value === 'generating' || aiTranslateBatchStateValue.value === 'generating') {
    console.warn('[proofread] not scanning: a translation is running')
    return
  }
  void runProofread(proofreadScopeIds.value[kind])
}

export const proofreadHandlers = {
  runAiProofread: (): void => {
    scan('segment')
  },

  runAiProofreadSelection: (): void => {
    scan('selection')
  },

  runAiProofreadChapter: (): void => {
    scan('chapter')
  },

  openAiProofreadScope: (): void => {
    openProofreadScope()
  },

  closeAiProofreadScope: (): void => {
    closeProofreadScope()
  },

  cancelAiProofread: (): void => {
    cancelProofread()
  },
}
