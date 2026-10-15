import { aiTranslateBatchStateValue } from './aiTranslateBatchState'
import { aiTranslateStateValue } from './aiTranslateState'
import {
  clearProofreadMemory,
  closeProofreadMemory,
  forgetProofreadMemoryRow,
  openProofreadMemory,
} from './proofreadMemoryState'
import { proofreadScopeIds } from './proofreadScope'
import type { ProofreadScopeKind } from './proofreadScope'
import {
  acceptProofreadFinding,
  cancelProofread,
  closeProofreadScope,
  dismissProofreadFinding,
  hideProofreadStrip,
  ignoreProofreadFinding,
  nextProofreadFinding,
  openProofreadScope,
  prevProofreadFinding,
  revertProofreadFinding,
  runProofread,
} from './proofreadState'

function scan(kind: ProofreadScopeKind): void {
  closeProofreadScope()
  if (aiTranslateStateValue.value === 'generating' || aiTranslateBatchStateValue.value === 'generating') {
    console.warn('[proofread] not scanning: a translation is running')
    return
  }
  void runProofread(proofreadScopeIds.value[kind], kind)
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

  acceptAiProofread: (): void => {
    void acceptProofreadFinding()
  },

  dismissAiProofread: (): void => {
    dismissProofreadFinding()
  },

  ignoreAiProofread: (): void => {
    void ignoreProofreadFinding()
  },

  openAiProofreadMemory: (): void => {
    void openProofreadMemory()
  },

  closeAiProofreadMemory: (): void => {
    closeProofreadMemory()
  },

  forgetAiProofreadMemory: (): void => {
    void forgetProofreadMemoryRow()
  },

  clearAiProofreadMemory: (): void => {
    void clearProofreadMemory()
  },

  hideAiProofread: (): void => {
    hideProofreadStrip()
  },

  nextAiProofread: (): void => {
    nextProofreadFinding()
  },

  prevAiProofread: (): void => {
    prevProofreadFinding()
  },

  revertAiProofread: (): void => {
    void revertProofreadFinding()
  },
}
