import {
  acceptConventionSpot,
  hideConventionStrip,
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
import { closeConventionManage, openConventionManage, submitConventionRule } from './conventionManageState'

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
