import type { CommandDeps } from './commands'
import { closeExport, openExport, runExport } from './exportState'

/** The `CommandDeps` handlers of the export screen; `main.ts` spreads these. */
export function exportCommandDeps(): Pick<CommandDeps, 'openExport' | 'closeExport' | 'runExport'> {
  return {
    openExport: () => {
      void openExport()
    },
    closeExport,
    runExport: () => {
      void runExport()
    },
  }
}
