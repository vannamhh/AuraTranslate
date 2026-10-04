import type { CommandDeps } from './commands'
import { cancelTmImportPreview, confirmTmImportPreview, openTmImportPreviewOverlay } from './tmImportState'
import {
  beginTmManageEdit,
  cancelTmManageEdit,
  closeTmManage,
  deleteTmManageOthers,
  deleteTmManagePair,
  exportTmManageTier,
  nextTmManageRow,
  openTmManage,
  prevTmManageRow,
  pushTmManagePair,
  saveTmManageEdit,
  tmManageExchangeTier,
} from './tmManageState'

/** The `CommandDeps` handlers of the TM management overlay; `main.ts` spreads these. */
export function tmManageCommandDeps(): Pick<
  CommandDeps,
  | 'openTmManage'
  | 'closeTmManage'
  | 'beginTmManageEdit'
  | 'saveTmManageEdit'
  | 'cancelTmManageEdit'
  | 'deleteTmManagePair'
  | 'deleteTmManageOthers'
  | 'pushTmManagePair'
  | 'nextTmManageRow'
  | 'prevTmManageRow'
  | 'exportTmManageTier'
  | 'openTmImportPreview'
  | 'confirmTmImportPreview'
  | 'cancelTmImportPreview'
> {
  return {
    openTmManage: () => {
      void openTmManage()
    },
    closeTmManage,
    beginTmManageEdit,
    saveTmManageEdit: () => {
      void saveTmManageEdit()
    },
    cancelTmManageEdit,
    deleteTmManagePair: () => {
      void deleteTmManagePair()
    },
    deleteTmManageOthers: () => {
      void deleteTmManageOthers()
    },
    pushTmManagePair: () => {
      void pushTmManagePair()
    },
    nextTmManageRow,
    prevTmManageRow,
    exportTmManageTier: () => {
      void exportTmManageTier()
    },
    openTmImportPreview: () => {
      void openTmImportPreviewOverlay(tmManageExchangeTier.value)
    },
    confirmTmImportPreview: () => {
      void confirmTmImportPreview()
    },
    cancelTmImportPreview: () => {
      void cancelTmImportPreview()
    },
  }
}
