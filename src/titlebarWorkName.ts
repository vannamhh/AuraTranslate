import { computed } from 'vue'
import { currentMode } from './modes/modeState'
import { openWorkName } from './openWorkIdentity'

/** The open Work's name, beside the mode tabs; Library already shows the Work on its own page. */
export const titlebarWorkName = computed<string | null>(() => {
  const name = openWorkName.value
  if (currentMode.value === 'library' || name === null || name.trim() === '') return null
  return name
})
