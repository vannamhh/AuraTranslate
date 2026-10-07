import type { CommandDeps } from './commands'
import { chooseExportFolder, closeExport, openExport, runExport } from './exportState'

/** The `CommandDeps` handlers of the export screen; `main.ts` spreads these. */
export function exportCommandDeps(): Pick<CommandDeps, 'openExport' | 'closeExport' | 'chooseExportFolder' | 'runExport'> {
  return {
    openExport: () => {
      void openExport()
    },
    closeExport,
    chooseExportFolder: () => {
      void chooseExportFolder()
    },
    runExport: () => {
      void runExport()
    },
  }
}
