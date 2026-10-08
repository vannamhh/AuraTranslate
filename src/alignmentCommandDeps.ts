import type { CommandDeps } from './commands'
import {
  closeAlignmentOverlay,
  joinAlignmentMarks,
  nextAlignmentEntry,
  openAlignmentOverlay,
  prevAlignmentEntry,
  skipAlignmentEntry,
  toggleAlignmentMark,
  unjoinAlignmentEntry,
} from './alignmentState'
import { editorChapterId } from './panels/editorPanelState'
import { cancelReviewerImportPreview } from './reviewerImportState'

type AlignmentDepNames =
  | 'openAlignment'
  | 'closeAlignment'
  | 'nextAlignmentEntry'
  | 'prevAlignmentEntry'
  | 'toggleAlignmentMark'
  | 'joinAlignmentMarks'
  | 'skipAlignmentEntry'
  | 'unjoinAlignmentEntry'

/** The `CommandDeps` handlers of the alignment overlay; `main.ts` spreads these. */
export function alignmentCommandDeps(): Pick<CommandDeps, AlignmentDepNames> {
  return {
    openAlignment: () => {
      void cancelReviewerImportPreview()
      void openAlignmentOverlay(editorChapterId.value)
    },
    closeAlignment: closeAlignmentOverlay,
    nextAlignmentEntry,
    prevAlignmentEntry,
    toggleAlignmentMark,
    joinAlignmentMarks: () => {
      void joinAlignmentMarks()
    },
    skipAlignmentEntry: () => {
      void skipAlignmentEntry()
    },
    unjoinAlignmentEntry: () => {
      void unjoinAlignmentEntry()
    },
  }
}
