import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'
import type { AiConfigTestOutcome, AiNotReadyReason } from './aiconfig'

export type QuickAiConfigField = 'quick_provider' | 'quick_endpoint' | 'quick_model'
export type QuickAiConfigSource = 'dedicated' | 'main_global' | 'incomplete'
export type QuickAiConfigWire = {
  fields: { field: QuickAiConfigField; value: string }[]
  key_configured: boolean | null
  source: QuickAiConfigSource
  auto_enabled: boolean
  auto_eligible: boolean
  ready: boolean | null
  not_ready: AiNotReadyReason[]
}

const CMD_SET_AUTO = 'quick_ai_config_set_auto'
const CMD_GET = 'quick_ai_config_get'
const CMD_SAVE_FIELDS = 'quick_ai_config_save_fields'
const CMD_SAVE_KEY = 'quick_ai_config_save_key'
const CMD_DELETE_KEY = 'quick_ai_config_delete_key'
const CMD_TEST_CONNECTION = 'quick_ai_config_test_connection'
const CMD_TEST_CANCEL = 'quick_ai_config_test_connection_cancel'

const UNKNOWN: IpcError = { code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false }
const FIELDS = ['quick_provider', 'quick_endpoint', 'quick_model'] as const

function ipcError(value: unknown): IpcError {
  if (typeof value !== 'object' || value === null) return UNKNOWN
  const v = value as Record<string, unknown>
  return typeof v.code === 'string' && typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' && typeof v.params === 'object' && v.params !== null ? v as IpcError : UNKNOWN
}

function isWire(value: unknown): value is QuickAiConfigWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Record<string, unknown>
  const fields = v.fields
  return Array.isArray(fields) && fields.length === FIELDS.length &&
    FIELDS.every((field) => fields.filter((f: unknown) => typeof f === 'object' && f !== null && 'field' in f && f.field === field && 'value' in f && typeof f.value === 'string').length === 1) &&
    (v.key_configured === null || typeof v.key_configured === 'boolean') &&
    typeof v.auto_enabled === 'boolean' && typeof v.auto_eligible === 'boolean' &&
    (v.ready === null || typeof v.ready === 'boolean') &&
    typeof v.source === 'string' && ['dedicated', 'main_global', 'incomplete'].includes(v.source) &&
    Array.isArray(v.not_ready) && v.not_ready.every((r: unknown) => typeof r === 'string' && ['endpoint_missing', 'model_missing', 'key_missing'].includes(r))
}

export async function quickAiConfigGet(): Promise<{ config: QuickAiConfigWire | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_GET)
    return isWire(wire) ? { config: wire, error: null } : { config: null, error: UNKNOWN }
  } catch (err) {
    return { config: null, error: ipcError(err) }
  }
}

export async function quickAiConfigSaveFields(fields: readonly { field: QuickAiConfigField; value: string }[]): Promise<IpcError | null> {
  try {
    await invoke(CMD_SAVE_FIELDS, { tier: 'global', fields })
    return null
  } catch (err) {
    return ipcError(err)
  }
}

export async function quickAiConfigSaveKey(value: string): Promise<IpcError | null> {
  try {
    await invoke(CMD_SAVE_KEY, { tier: 'global', value })
    return null
  } catch (err) {
    return ipcError(err)
  }
}

export async function quickAiConfigDeleteKey(): Promise<IpcError | null> {
  try {
    await invoke(CMD_DELETE_KEY, { tier: 'global' })
    return null
  } catch (err) {
    return ipcError(err)
  }
}

export async function quickAiConfigTestConnection(): Promise<{ outcome: AiConfigTestOutcome | null; error: IpcError | null }> {
  try {
    const value = await invoke<unknown>(CMD_TEST_CONNECTION)
    if (typeof value === 'object' && value !== null && 'kind' in value &&
      (value.kind === 'cancelled' || value.kind === 'not_configured' ||
        (value.kind === 'connected' && 'model_count' in value && typeof value.model_count === 'number'))) {
      return { outcome: value as AiConfigTestOutcome, error: null }
    }
    return { outcome: null, error: UNKNOWN }
  } catch (err) {
    return { outcome: null, error: ipcError(err) }
  }
}

export async function quickAiConfigTestConnectionCancel(): Promise<IpcError | null> {
  try {
    await invoke(CMD_TEST_CANCEL)
    return null
  } catch (err) {
    return ipcError(err)
  }
}

export async function quickAiConfigSetAuto(enabled: boolean): Promise<IpcError | null> {
  try {
    await invoke(CMD_SET_AUTO, { tier: 'global', enabled })
    return null
  } catch (err) {
    return ipcError(err)
  }
}
