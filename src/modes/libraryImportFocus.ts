import { nextTick } from 'vue'
import { setMode } from './modeState'

const FIRST_FIELD = 'input:not([type="radio"]):not([type="checkbox"]):not([type="file"]):not([disabled]), textarea:not([disabled])'

export async function openLibraryImportForm(): Promise<void> {
  try {
    setMode('library')
    await nextTick()
    const form = document.querySelector<HTMLElement>('.import-form')
    if (form === null) {
      console.warn('[library] import form is not in the DOM; nothing to focus.')
      return
    }
    form.scrollIntoView({ block: 'nearest' })
    form.querySelector<HTMLElement>(FIRST_FIELD)?.focus()
  } catch (err) {
    console.error('[library] opening the import form failed:', err)
  }
}
