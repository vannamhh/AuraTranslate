import type { CommandDeps } from './commands'
import { setMode } from './modes/modeState'
import { editorChapterId } from './panels/editorPanelState'
import {
  cancelPendingAccept,
  closeReviewMode,
  confirmPendingAccept,
  openReviewMode,
  reviewAcceptChange,
  reviewDiffNext,
  reviewDiffPrev,
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

/** The `CommandDeps` handlers of Review Mode; `main.ts` spreads these. */
export function reviewModeCommandDeps(): Pick<CommandDeps, ReviewModeDepNames> {
  return {
    openReviewMode: () => {
      setMode('workspace')
      void openReviewMode(editorChapterId.value)
    },
    closeReviewMode: () => {
      void closeReviewMode()
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
  }
}
