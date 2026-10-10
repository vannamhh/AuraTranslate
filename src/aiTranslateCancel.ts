import { cancelAiTranslateCall } from './config/aitranslate'

export function cancelWhenGenerating(state: { readonly value: string }): void {
  if (state.value !== 'generating') return
  void cancelAiTranslateCall()
}
