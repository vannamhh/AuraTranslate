import type { CommandDeps } from './commands'
import { closeChapterMerge, confirmChapterMerge, openChapterMerge } from './chapterMergeState'
import { closeChapterSplit, confirmChapterSplit, openChapterSplit } from './chapterSplitState'
import {
  beginChapterRename,
  cancelChapterRename,
  clearLibraryChapterSelection,
  closeLibraryWorkPage,
  setSelectedChaptersStatus,
} from './modes/libraryChapters'
import { setChapterFilter, toggleChapterStatusMenu, toggleNotStartedGroup } from './modes/libraryChapterSelection'

type WorkPageKeys =
  | 'closeLibraryWorkPage'
  | 'setLibraryChapterFilter'
  | 'toggleLibraryNotStartedGroup'
  | 'clearLibraryChapterSelection'
  | 'beginLibraryChapterRename'
  | 'cancelLibraryChapterRename'
  | 'toggleLibraryChapterStatusMenu'
  | 'setLibrarySelectedChaptersStatus'
  | 'openLibraryChapterMerge'
  | 'closeLibraryChapterMerge'
  | 'confirmLibraryChapterMerge'
  | 'openLibraryChapterSplit'
  | 'closeLibraryChapterSplit'
  | 'confirmLibraryChapterSplit'

/** The `CommandDeps` handlers of the Work page and its two screens; `main.ts` spreads these. */
export function libraryWorkPageCommandDeps(): Pick<CommandDeps, WorkPageKeys> {
  return {
    closeLibraryWorkPage,
    setLibraryChapterFilter: setChapterFilter,
    toggleLibraryNotStartedGroup: toggleNotStartedGroup,
    clearLibraryChapterSelection,
    beginLibraryChapterRename: beginChapterRename,
    cancelLibraryChapterRename: cancelChapterRename,
    toggleLibraryChapterStatusMenu: toggleChapterStatusMenu,
    setLibrarySelectedChaptersStatus: (status) => {
      void setSelectedChaptersStatus(status)
    },
    openLibraryChapterMerge: openChapterMerge,
    closeLibraryChapterMerge: closeChapterMerge,
    confirmLibraryChapterMerge: () => {
      void confirmChapterMerge()
    },
    openLibraryChapterSplit: () => {
      void openChapterSplit()
    },
    closeLibraryChapterSplit: closeChapterSplit,
    confirmLibraryChapterSplit: () => {
      void confirmChapterSplit()
    },
  }
}
