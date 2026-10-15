import {
  aiTranslateBatchError,
  aiTranslateBatchPromotableItems,
  aiTranslateBatchRetryIds,
  aiTranslateBatchRows,
  aiTranslateBatchStartStamp,
  aiTranslateBatchStateValue,
  aiTranslateBatchTextForSegment,
  cancelAiTranslateBatch,
  resetAiTranslateBatch,
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
  resendAiPrompt,
  runAiTranslate,
} from './aiTranslateState'
import { aiPromptRecord, refreshAiPromptRecord, selectAiPromptInspectorTab } from './aiPromptInspectorState'
import {
  cancelPendingPromote,
  editorCaretSegmentId,
  editorPendingPromoteBatch,
  promoteAiTranslationBatchToEditor,
  promoteAiTranslationToEditor,
} from './panels/editorPanelState'
import { segmentSelectionIds } from './panels/segmentSelectionState'
import { selectedPromptSetName } from './promptSetState'

/**
 * The AI-translate command handlers. Single run and batch guard each other here because
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

  resendAiPrompt: (): void => {
    const record = aiPromptRecord.value
    if (record === null) {
      console.warn('[ai-translate] khong gui lai: chua co ban ghi prompt nao')
      return
    }
    if (aiTranslateBatchStateValue.value === 'generating') {
      console.warn('[ai-translate] khong gui lai: mot lo dang chay')
      return
    }
    if (aiTranslateStateValue.value === 'generating') {
      console.warn('[ai-translate] khong gui lai: mot luot don dang chay')
      return
    }
    selectAiPromptInspectorTab('result')
    void resendAiPrompt(record.segment_id).then(() => refreshAiPromptRecord())
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
  discardAiTranslateBatch: (): void => {
    if (editorPendingPromoteBatch.value !== null) cancelPendingPromote()
    resetAiTranslateBatch()
  },

  promoteAiTranslateBatch: (): void => {
    if (aiTranslateBatchStateValue.value === 'generating') {
      console.warn('[ai-translate-batch] khong dua ca lo sang Editor: lo dang chay')
      return
    }
    const items = aiTranslateBatchPromotableItems(aiTranslateBatchRows.value)
    if (items.length === 0) {
      console.warn('[ai-translate-batch] khong dua ca lo sang Editor: khong co hang nao da xong')
      return
    }
    void promoteAiTranslationBatchToEditor(items)
  },
}
