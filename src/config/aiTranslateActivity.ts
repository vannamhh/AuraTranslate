import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { readonly, ref } from 'vue'
import type { IpcError } from '../i18n'
import { isIpcError, UNKNOWN_IPC_ERROR } from './aitranslate'

const CMD_ACTIVITY = 'ai_translate_activity'

export type AiTranslateActivityWire = { revision: number; active_count: number }
const activity = ref<AiTranslateActivityWire | null>(null)
const activityError = ref<IpcError | null>(null)
export const aiTranslateActivity = readonly(activity)
export const aiTranslateActivityError = readonly(activityError)

export function resetAiTranslateActivity(): void {
  activity.value = null
  activityError.value = null
}

export function isAiTranslateActivityWire(value: unknown): value is AiTranslateActivityWire {
  if (typeof value !== 'object' || value === null) return false
  const wire = value as Record<string, unknown>
  return typeof wire.revision === 'number' && Number.isSafeInteger(wire.revision) && wire.revision >= 0 &&
    typeof wire.active_count === 'number' && Number.isSafeInteger(wire.active_count) && wire.active_count >= 0
}

function accept(value: unknown): void {
  if (!isAiTranslateActivityWire(value)) {
    activityError.value = UNKNOWN_IPC_ERROR
    return
  }
  if (activity.value !== null && value.revision <= activity.value.revision) return
  activity.value = value
  activityError.value = null
}

export async function subscribeAiTranslateActivity(): Promise<{ unlisten: UnlistenFn | null; error: IpcError | null }> {
  resetAiTranslateActivity()
  let unlisten: UnlistenFn | null = null
  try {
    unlisten = await listen<unknown>('ai-translate-activity-v1', event => accept(event.payload))
    accept(await invoke<unknown>(CMD_ACTIVITY))
    return { unlisten, error: activityError.value }
  } catch (error) {
    activityError.value = isIpcError(error) ? error : UNKNOWN_IPC_ERROR
    return { unlisten, error: activityError.value }
  }
}
