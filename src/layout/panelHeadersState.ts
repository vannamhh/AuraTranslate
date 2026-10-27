/** Panel-arrange mode: a global `app_config` key; on shows every dock tab bar, off hides those of single-panel groups. */
import { computed, ref } from 'vue'
import type { ComputedRef } from 'vue'
import { KEY_PANEL_HEADERS_SHOWN, SCOPE_APP_CONFIG, bootstrapPanelHeadersShown, putConfig } from '../config/bootstrap'
import type { IpcError } from '../i18n'

const changed = ref<boolean | null>(null)
const saveError = ref<IpcError | null>(null)

export const panelHeadersShown: ComputedRef<boolean> = computed(() => changed.value ?? bootstrapPanelHeadersShown.value)

export const panelHeadersSaveError: ComputedRef<IpcError | null> = computed(() => saveError.value)

export async function setPanelHeadersShown(next: boolean): Promise<void> {
  changed.value = next
  saveError.value = null
  const err = await putConfig(SCOPE_APP_CONFIG, KEY_PANEL_HEADERS_SHOWN, next ? 'on' : 'off')
  if (err !== null) saveError.value = err
}

export function resetPanelHeadersShown(): void {
  changed.value = null
  saveError.value = null
}
