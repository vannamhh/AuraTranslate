import type { CommandDeps } from './commands'
import {
  beginTmManageEdit,
  cancelTmManageEdit,
  closeTmManage,
  deleteTmManageOthers,
  deleteTmManagePair,
  nextTmManageRow,
  openTmManage,
  prevTmManageRow,
  pushTmManagePair,
  saveTmManageEdit,
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
  }
}
