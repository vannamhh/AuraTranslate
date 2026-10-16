/** "Show tokens and cost estimate" switch (A25): a global `app_config` key that only hides the usage lines of the AI panel. */
import { computed, ref } from 'vue'
import type { ComputedRef } from 'vue'
import { KEY_AI_USAGE_VISIBLE, SCOPE_APP_CONFIG, bootstrapAiUsageVisible, putConfig } from './config/bootstrap'
import type { IpcError } from './i18n'

const changed = ref<boolean | null>(null)
const saveError = ref<IpcError | null>(null)

export const aiUsageVisible: ComputedRef<boolean> = computed(() => changed.value ?? bootstrapAiUsageVisible.value !== 'off')

export const aiUsageVisibleSaveError: ComputedRef<IpcError | null> = computed(() => saveError.value)

export async function setAiUsageVisible(next: boolean): Promise<void> {
  changed.value = next
  saveError.value = null
  const err = await putConfig(SCOPE_APP_CONFIG, KEY_AI_USAGE_VISIBLE, next ? 'on' : 'off')
  if (err !== null) saveError.value = err
}

export function resetAiUsageVisible(): void {
  changed.value = null
  saveError.value = null
}
