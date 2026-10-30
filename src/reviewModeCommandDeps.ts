import type { CommandDeps } from './commands'
import { setMode } from './modes/modeState'
import { editorChapterId, editorLastCaretSegmentId, goToNextChapter, goToPrevChapter, setEditorCaret } from './panels/editorPanelState'
import {
  cancelPendingAccept,
  closeReviewMode,
  confirmPendingAccept,
  openReviewMode,
  refuseChapterMoveInReview,
  reviewAcceptChange,
  reviewDiffNext,
  reviewDiffPrev,
  reviewModeIsOpen,
  reviewSkipChange,
} from './reviewModeState'

type ReviewModeDepNames =
  | 'openReviewMode'
  | 'closeReviewMode'
  | 'reviewDiffNext'
  | 'reviewDiffPrev'
  | 'reviewAcceptChange'
  | 'reviewSkipChange'
  | 'reviewConfirmAccept'
  | 'reviewCancelAccept'
  | 'goToNextChapter'
  | 'goToPrevChapter'

/** After Review Mode the panel shell holds focus, and the caret only shows from inside the cell. */
export function returnToLastCaret(): void {
  const id = editorLastCaretSegmentId.value
  const active = document.activeElement
  if (id === null || active === null || active === document.body) return
  const cell = active.querySelector<HTMLElement>(`[data-col="tgt"][data-segment-id="${String(id)}"]`)
  if (cell === null) return
  cell.focus({ preventScroll: true })
  setEditorCaret(id)
}

/** The `CommandDeps` handlers of Review Mode; `main.ts` spreads these. */
export function reviewModeCommandDeps(): Pick<CommandDeps, ReviewModeDepNames> {
  return {
    openReviewMode: () => {
      setMode('workspace')
      void openReviewMode(editorChapterId.value)
    },
    closeReviewMode: () => {
      const wasOpen = reviewModeIsOpen.value
      void closeReviewMode().then(() => {
        if (wasOpen) returnToLastCaret()
      })
    },
    reviewDiffNext,
    reviewDiffPrev,
    reviewAcceptChange: () => {
      void reviewAcceptChange()
    },
    reviewSkipChange: () => {
      void reviewSkipChange()
    },
    reviewConfirmAccept: () => {
      void confirmPendingAccept()
    },
    reviewCancelAccept: cancelPendingAccept,
    goToNextChapter: () => {
      if (!refuseChapterMoveInReview()) goToNextChapter()
    },
    goToPrevChapter: () => {
      if (!refuseChapterMoveInReview()) goToPrevChapter()
    },
  }
}
