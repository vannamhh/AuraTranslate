import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import type { IpcError } from './i18n'
import { editorCaretSegmentId } from './panels/editorPanelState'
import { syncTmFuzzyStrip } from './tmFuzzyStripState'
import { KEY_TM_FUZZY_THRESHOLD, SCOPE_APP_CONFIG, bootstrapTmFuzzyThreshold, putConfig } from './config/bootstrap'

const saved = ref(false)
const thresholdInput = ref('')
const knownThreshold = ref<number | null>(null)
const saving = ref(false)
const saveError = ref<IpcError | null>(null)

export const tmSettingsSaved: DeepReadonly<Ref<boolean>> = readonly(saved)
export const tmSettingsThresholdInput: Ref<string> = thresholdInput
export const tmSettingsSaving: DeepReadonly<Ref<boolean>> = readonly(saving)
export const tmSettingsSaveError: DeepReadonly<Ref<IpcError | null>> = readonly(saveError)

export const TM_FUZZY_THRESHOLD_MIN = 50
export const TM_FUZZY_THRESHOLD_MAX = 99

/**
 * Mirrors the range `core::scope::store::parse_tm_fuzzy_threshold` accepts so the form can
 * refuse before a round trip; Rust stays the judge and falls back to 65 on a bad stored value.
 */
export function parsedTmFuzzyThreshold(raw: string): number | null {
  const trimmed = raw.trim()
  if (!/^\+?[0-9]+$/.test(trimmed)) return null
  const n = Number(trimmed)
  return Number.isInteger(n) && n >= TM_FUZZY_THRESHOLD_MIN && n <= TM_FUZZY_THRESHOLD_MAX ? n : null
}

export function loadTmSettingsForm(): void {
  if (saving.value) return
  thresholdInput.value = String(knownThreshold.value ?? bootstrapTmFuzzyThreshold.value)
  saveError.value = null
  saved.value = false
}

export function markTmSettingsEdited(): void {
  saved.value = false
}

/** Handler of `tm.settings.save`; `@submit` also reaches it, so it re-validates the input. */
export async function saveTmSettings(): Promise<void> {
  if (saving.value) return
  const parsed = parsedTmFuzzyThreshold(thresholdInput.value)
  if (parsed === null) return

  saving.value = true
  saveError.value = null
  saved.value = false

  const err = await putConfig(SCOPE_APP_CONFIG, KEY_TM_FUZZY_THRESHOLD, String(parsed))

  saving.value = false
  if (err !== null) {
    saveError.value = err
    return
  }
  knownThreshold.value = parsed
  saved.value = true
  syncTmFuzzyStrip(editorCaretSegmentId.value)
}
