import {
  acceptConventionSpot,
  hideConventionStrip,
  ignoreConventionSpot,
  revertConventionSpot,
  skipConventionSpot,
  stepMarks,
  viewedConventionRuleKey,
} from './conventionState'
import {
  closeConventionReplace,
  confirmConventionReplace,
  openConventionReplace,
  rescanConventionReplace,
  selectConventionReplaceVisible,
} from './conventionReplaceState'
import {
  beginConventionManageEdit,
  cancelConventionManageDeleteConfirm,
  cancelConventionManageEdit,
  clearConventionManageIgnores,
  closeConventionManage,
  deleteConventionManageRule,
  disableConventionManageRule,
  enableConventionManageRule,
  exportConventionManageTier,
  forgetConventionManageIgnore,
  nextConventionManageTier,
  openConventionManage,
  saveConventionManageEdit,
  setConventionManageTab,
  stepConventionManageCursor,
  submitConventionRule,
} from './conventionManageState'
import {
  cancelConventionImportPreview,
  confirmConventionImportPreview,
  decideConventionImportAtCursor,
  openConventionImportOverlay,
} from './conventionImportState'

export const conventionHandlers = {
  nextMark: (): void => {
    stepMarks(1)
  },

  prevMark: (): void => {
    stepMarks(-1)
  },

  acceptConvention: (): void => {
    void acceptConventionSpot()
  },

  skipConvention: (): void => {
    skipConventionSpot()
  },

  ignoreConvention: (): void => {
    void ignoreConventionSpot()
  },

  revertConvention: (): void => {
    void revertConventionSpot()
  },

  hideConvention: (): void => {
    hideConventionStrip()
  },

  openConventionManage: (): void => {
    void openConventionManage()
  },

  closeConventionManage: (): void => {
    closeConventionManage()
  },

  addConventionRule: (): void => {
    void submitConventionRule()
  },

  beginConventionManageEdit: (): void => {
    beginConventionManageEdit()
  },

  saveConventionManageEdit: (): void => {
    void saveConventionManageEdit()
  },

  cancelConventionManageEdit: (): void => {
    cancelConventionManageDeleteConfirm()
    cancelConventionManageEdit()
  },

  deleteConventionManageRule: (): void => {
    void deleteConventionManageRule()
  },

  disableConventionManageRule: (): void => {
    void disableConventionManageRule()
  },

  enableConventionManageRule: (): void => {
    void enableConventionManageRule()
  },

  nextConventionManageRow: (): void => {
    stepConventionManageCursor(1)
  },

  prevConventionManageRow: (): void => {
    stepConventionManageCursor(-1)
  },

  nextConventionManageTier: (): void => {
    nextConventionManageTier()
  },

  showConventionManageRulesTab: (): void => {
    setConventionManageTab('rules')
  },

  showConventionManageIgnoredTab: (): void => {
    setConventionManageTab('ignored')
  },

  forgetConventionManageIgnore: (): void => {
    void forgetConventionManageIgnore()
  },

  clearConventionManageIgnores: (): void => {
    void clearConventionManageIgnores()
  },

  exportConventionManageTier: (): void => {
    void exportConventionManageTier()
  },

  openConventionImportPreview: (): void => {
    void openConventionImportOverlay()
  },

  confirmConventionImportPreview: (): void => {
    void confirmConventionImportPreview()
  },

  cancelConventionImportPreview: (): void => {
    void cancelConventionImportPreview()
  },

  keepMineConventionImport: (): void => {
    decideConventionImportAtCursor('keep_mine')
  },

  takeFileConventionImport: (): void => {
    decideConventionImportAtCursor('take_file')
  },

  openConventionReplace: (): void => {
    void openConventionReplace(null)
  },

  openConventionReplaceForRule: (): void => {
    void openConventionReplace(viewedConventionRuleKey())
  },

  selectAllConventionReplace: (): void => {
    selectConventionReplaceVisible(true)
  },

  selectNoneConventionReplace: (): void => {
    selectConventionReplaceVisible(false)
  },

  confirmConventionReplace: (): void => {
    void confirmConventionReplace()
  },

  rescanConventionReplace: (): void => {
    void rescanConventionReplace()
  },

  closeConventionReplace: (): void => {
    closeConventionReplace()
  },
}
