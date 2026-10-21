/** Manual update check (FR111): switch, last outcome, and the busy flag. Nothing here runs on startup. */
import { computed, ref } from 'vue'
import type { ComputedRef } from 'vue'
import {
  KEY_UPDATE_CHECK_ENABLED,
  SCOPE_APP_CONFIG,
  bootstrapAppVersion,
  bootstrapUpdateCheckEnabled,
  putConfig,
} from './config/bootstrap'
import { updateCheck, updateOpenReleasePage } from './config/update'
import type { UpdateCheckOutcome } from './config/update'
import type { IpcError } from './i18n'

const changed = ref<boolean | null>(null)
const outcome = ref<UpdateCheckOutcome | null>(null)
const checking = ref(false)
const checkError = ref<IpcError | null>(null)
const saveError = ref<IpcError | null>(null)
const openError = ref<IpcError | null>(null)

export const updateCheckEnabled: ComputedRef<boolean> = computed(() => changed.value ?? bootstrapUpdateCheckEnabled.value)
export const updateAppVersion: ComputedRef<string> = computed(() => bootstrapAppVersion.value)
export const updateOutcome: ComputedRef<UpdateCheckOutcome | null> = computed(() => outcome.value)
export const updateChecking: ComputedRef<boolean> = computed(() => checking.value)
export const updateCheckError: ComputedRef<IpcError | null> = computed(() => checkError.value)
export const updateSaveError: ComputedRef<IpcError | null> = computed(() => saveError.value)
export const updateOpenError: ComputedRef<IpcError | null> = computed(() => openError.value)

export async function setUpdateCheckEnabled(next: boolean): Promise<void> {
  changed.value = next
  saveError.value = null
  const err = await putConfig(SCOPE_APP_CONFIG, KEY_UPDATE_CHECK_ENABLED, next ? 'on' : 'off')
  if (err !== null) saveError.value = err
}

/** A failed check keeps the previous outcome on screen and only sets the error. */
export async function runUpdateCheck(): Promise<void> {
  if (checking.value || !updateCheckEnabled.value) return
  checking.value = true
  checkError.value = null
  openError.value = null
  const result = await updateCheck()
  checking.value = false
  if (result.error !== null) {
    checkError.value = result.error
    return
  }
  if (result.outcome !== null) outcome.value = result.outcome
}

export async function openUpdateReleasePage(): Promise<void> {
  const current = outcome.value
  if (current?.kind !== 'update_available') return
  openError.value = null
  const err = await updateOpenReleasePage(current.latest)
  if (err !== null) openError.value = err
}
