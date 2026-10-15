import { dispatch } from './commands'
import { promptLibraryOverlayIsOpen } from './promptLibraryState'
import {
  aiPromptRecord,
  aiPromptSourceText,
  closeAiPromptInspectorThen,
  copyAiPromptToClipboard,
  selectAiPromptInspectorTab,
} from './aiPromptInspectorState'
import { runConcordance } from './panels/concordanceState'
import { selectLookupTab } from './panels/lookupHistoryState'

/** Handlers of the prompt inspector's own commands. Each logs and returns instead of throwing. */
export const aiPromptInspectorHandlers = {
  copyAiPrompt: (): void => {
    void copyAiPromptToClipboard()
  },

  selectAiPromptResultTab: (): void => {
    selectAiPromptInspectorTab('result')
  },

  selectAiPromptPromptTab: (): void => {
    selectAiPromptInspectorTab('prompt')
  },

  openAiPromptTmConcordance: (): void => {
    const source = aiPromptSourceText(aiPromptRecord.value)
    if (source === '') {
      console.warn('[ai-prompt-inspector] khong tra TM: ban ghi khong mang cau nguon')
      return
    }
    void closeAiPromptInspectorThen(() => {
      selectLookupTab('concordance')
      void runConcordance(source)
    })
  },

  confirmAiPromptTermNow: (): void => {
    void closeAiPromptInspectorThen(() => {
      dispatch('glossary.confirm.focus')
    })
  },

  openAiPromptGlossaryEntry: (): void => {
    void closeAiPromptInspectorThen(() => {
      dispatch('glossary.manage.open')
    })
  },

  openAiPromptWorkPrompts: (): void => {
    const libraryIsOpen = promptLibraryOverlayIsOpen.value
    void closeAiPromptInspectorThen(() => {
      if (!libraryIsOpen) dispatch('prompt.library.open')
    })
  },
}
