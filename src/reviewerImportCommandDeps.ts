import type { CommandDeps } from './commands'
import {
  cancelReviewerImportPreview,
  confirmReviewerImportPreview,
  openReviewerImportPreviewOverlay,
} from './reviewerImportState'

/** The `CommandDeps` handlers of the reviewer-copy import overlay; `main.ts` spreads these. */
export function reviewerImportCommandDeps(): Pick<
  CommandDeps,
  'openReviewerImportPreview' | 'confirmReviewerImportPreview' | 'cancelReviewerImportPreview'
> {
  return {
    openReviewerImportPreview: () => {
      void openReviewerImportPreviewOverlay()
    },
    confirmReviewerImportPreview: () => {
      void confirmReviewerImportPreview()
    },
    cancelReviewerImportPreview: () => {
      void cancelReviewerImportPreview()
    },
  }
}
