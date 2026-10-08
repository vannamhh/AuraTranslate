import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { translatorNameGet, translatorNameSave } from './config/attribution'
import type { IpcError } from './i18n'

const nameInput = ref('')
const saved = ref(false)
const saving = ref(false)
const saveError = ref<IpcError | null>(null)

export const exportSettingsNameInput: Ref<string> = nameInput
export const exportSettingsSaved: DeepReadonly<Ref<boolean>> = readonly(saved)
export const exportSettingsSaving: DeepReadonly<Ref<boolean>> = readonly(saving)
export const exportSettingsSaveError: DeepReadonly<Ref<IpcError | null>> = readonly(saveError)

export async function loadExportSettingsForm(): Promise<void> {
  if (saving.value) return
  saveError.value = null
  saved.value = false
  const result = await translatorNameGet()
  if (result.error !== null) saveError.value = result.error
  nameInput.value = result.name ?? ''
}

export function markExportSettingsEdited(): void {
  saved.value = false
}

/** An empty name after trim clears the stored value (Rust decides). */
export async function saveExportSettings(): Promise<void> {
  if (saving.value) return
  saving.value = true
  saveError.value = null
  saved.value = false
  const result = await translatorNameSave(nameInput.value)
  saving.value = false
  if (result.error !== null) {
    saveError.value = result.error
    return
  }
  saved.value = result.ok
}
