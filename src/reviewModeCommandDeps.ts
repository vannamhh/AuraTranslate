import type { CommandDeps } from './commands'
import { setMode } from './modes/modeState'
import { editorChapterId } from './panels/editorPanelState'
import { closeReviewMode, openReviewMode, reviewDiffNext, reviewDiffPrev } from './reviewModeState'

type ReviewModeDepNames = 'openReviewMode' | 'closeReviewMode' | 'reviewDiffNext' | 'reviewDiffPrev'

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
  }
}
