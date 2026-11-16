import type { CommandDeps } from './commands'
import { openExportForChapter } from './exportState'
import { readingCurrentChapter } from './modes/readingState'

/** The `CommandDeps` handler of the Reading toolbar export button; `main.ts` spreads it. */
export function readingExportCommandDeps(): Pick<CommandDeps, 'openExportForReadingChapter'> {
  return {
    openExportForReadingChapter: () => {
      const chapter = readingCurrentChapter.value
      if (chapter === null) return
      void openExportForChapter(chapter.chapter_id)
    },
  }
}
