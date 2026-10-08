/**
 * IPC adapter of the translator name (global tier). Never throws; `{ name: null, error: null }`
 * means "not set" on read and "bridge unavailable" only when `error` is also null on save.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

const CMD_TRANSLATOR_NAME_GET = 'translator_name_get'
const CMD_TRANSLATOR_NAME_SAVE = 'translator_name_save'

const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- wire data may carry null params
    v.params !== null
  )
}

function failureOf(err: unknown, command: string): IpcError | null {
  if (isIpcError(err)) return err
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    console.error(`[attribution] \`${command}\` failed with a non-IpcError value: ${String(err)}`)
    return UNKNOWN_IPC_ERROR
  }
  console.info(`[attribution] cannot call \`${command}\` — running outside Tauri? ${String(err)}`)
  return null
}

export type TranslatorNameResult = { name: string | null; error: IpcError | null }

export async function translatorNameGet(): Promise<TranslatorNameResult> {
  try {
    const raw = await invoke<unknown>(CMD_TRANSLATOR_NAME_GET)
    if (raw === null) return { name: null, error: null }
    if (typeof raw !== 'string') {
      console.error(`[attribution] \`${CMD_TRANSLATOR_NAME_GET}\` returned a non-string value: ${String(raw)}`)
      return { name: null, error: UNKNOWN_IPC_ERROR }
    }
    return { name: raw, error: null }
  } catch (err) {
    return { name: null, error: failureOf(err, CMD_TRANSLATOR_NAME_GET) }
  }
}

/** `ok` is false with `error` null when the IPC bridge is unavailable. */
export async function translatorNameSave(name: string): Promise<{ ok: boolean; error: IpcError | null }> {
  try {
    await invoke<unknown>(CMD_TRANSLATOR_NAME_SAVE, { name })
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: failureOf(err, CMD_TRANSLATOR_NAME_SAVE) }
  }
}
