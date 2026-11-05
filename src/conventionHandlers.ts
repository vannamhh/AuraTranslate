import {
  acceptConventionSpot,
  hideConventionStrip,
  revertConventionSpot,
  skipConventionSpot,
  stepMarks,
} from './conventionState'
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
}
