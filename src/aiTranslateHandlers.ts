import {
  aiTranslateBatchError,
  aiTranslateBatchRetryIds,
  aiTranslateBatchRows,
  aiTranslateBatchStartStamp,
  aiTranslateBatchStateValue,
  aiTranslateBatchTextForSegment,
  cancelAiTranslateBatch,
  retryAiTranslateBatch,
  runAiTranslateBatch,
} from './aiTranslateBatchState'
import {
  aiTranslateAccumulatedText,
  aiTranslateError,
  aiTranslateRunSegmentId,
  aiTranslateStartStamp,
  aiTranslateStateValue,
  cancelAiTranslate,
  runAiTranslate,
} from './aiTranslateState'
import { editorCaretSegmentId, promoteAiTranslationToEditor } from './panels/editorPanelState'
import { segmentSelectionIds } from './panels/segmentSelectionState'
import { selectedPromptSetName } from './promptSetState'

/**
 * The six AI-translate command handlers. Single run and batch guard each other here because
 * neither state module may import the other. Each handler logs and returns instead of throwing.
 */
export const aiTranslateHandlers = {
  runAiTranslate: (): void => {
    if (aiTranslateBatchStateValue.value === 'generating') {
      console.warn('[ai-translate] khong dich: mot lo dang chay')
      return
    }
    void runAiTranslate(selectedPromptSetName.value, editorCaretSegmentId.value)
  },

  cancelAiTranslate: (): void => {
    cancelAiTranslate()
    cancelAiTranslateBatch()
  },

  promoteAiTranslate: (): void => {
    const state = aiTranslateStateValue.value
    const segmentId = aiTranslateRunSegmentId.value
    const text = aiTranslateAccumulatedText.value
    const singleUsable = (state === 'done' || state === 'cancelled') && segmentId !== null && text !== ''

    const caretId = editorCaretSegmentId.value
    const batchText = caretId === null ? null : aiTranslateBatchTextForSegment(aiTranslateBatchRows.value, caretId)
    const batchUsable = batchText !== null && caretId !== null

    const batchIsNewer = aiTranslateBatchStartStamp.value > aiTranslateStartStamp.value
    if (batchUsable && (batchIsNewer || !singleUsable)) {
      void promoteAiTranslationToEditor(caretId, batchText)
      return
    }
    if (singleUsable) {
      void promoteAiTranslationToEditor(segmentId, text)
      return
    }

    console.warn(
      `[ai-translate] khong dua sang Editor: chua co ket qua hop le (state=${state}, ` +
        `segmentId=${String(segmentId)}, caretId=${String(caretId)})`,
    )
  },

  runAiTranslateBatch: (): void => {
    if (aiTranslateStateValue.value === 'generating') {
      console.warn('[ai-translate-batch] khong dich: mot luot don dang chay')
      return
    }
    void runAiTranslateBatch(selectedPromptSetName.value, segmentSelectionIds.value)
  },

  retryAiTranslate: (): void => {
    if (aiTranslateBatchStateValue.value === 'generating') {
      console.warn('[ai-translate] khong retry: mot lo dang chay')
      return
    }
    if (aiTranslateStateValue.value === 'generating') {
      console.warn('[ai-translate] khong retry: mot luot don khac dang chay')
      return
    }
    const err = aiTranslateError.value
    if (aiTranslateStateValue.value !== 'error' || err === null || err.retryable !== true) {
      console.warn('[ai-translate] khong retry: khong co loi retryable nao dang cho')
      return
    }
    const segmentId = aiTranslateRunSegmentId.value
    if (segmentId === null) {
      console.warn('[ai-translate] khong retry: khong biet cau nao da loi')
      return
    }
    void runAiTranslate(selectedPromptSetName.value, segmentId)
  },

  retryAiTranslateBatch: (): void => {
    if (aiTranslateStateValue.value === 'generating') {
      console.warn('[ai-translate-batch] khong retry: mot luot don dang chay')
      return
    }
    if (aiTranslateBatchStateValue.value === 'generating') {
      console.warn('[ai-translate-batch] khong retry: mot lo khac dang chay')
      return
    }
    const err = aiTranslateBatchError.value
    if (aiTranslateBatchStateValue.value !== 'error' || err === null || err.retryable !== true) {
      console.warn('[ai-translate-batch] khong retry: khong co loi retryable nao dang cho')
      return
    }
    const ids = aiTranslateBatchRetryIds(aiTranslateBatchRows.value)
    if (ids.length === 0) {
      console.warn('[ai-translate-batch] khong retry: khong con cau nao chua chay')
      return
    }
    void retryAiTranslateBatch(selectedPromptSetName.value, ids)
  },
}
