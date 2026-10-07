/**
 * IPC adapter of the export screen: scope counts and the destination-folder dialog.
 * Never throws; `{ value: null, error: null }` means the IPC bridge is unavailable.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

export type ExportScope = { kind: 'work' } | { kind: 'chapters'; chapter_ids: number[] }

export type ExportScopeCounts = {
  chapter_count: number
  segment_count: number
  unconfirmed_count: number
}

export type ExportScopeSummaryResult = { counts: ExportScopeCounts | null; error: IpcError | null }

export type ExportFolderResult =
  | { outcome: 'picked'; path: string }
  | { outcome: 'cancelled' }
  | { outcome: 'ipc_unavailable' }
  | { outcome: 'error'; error: IpcError }

const CMD_EXPORT_SCOPE_SUMMARY = 'export_scope_summary'
const CMD_EXPORT_CHOOSE_FOLDER = 'export_choose_folder'

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

function hasIpcBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function failureOf(err: unknown, command: string): IpcError | null {
  if (isIpcError(err)) return err
  if (hasIpcBridge()) {
    console.error(`[export] \`${command}\` failed with a non-IpcError value: ${String(err)}`)
    return UNKNOWN_IPC_ERROR
  }
  console.info(`[export] cannot call \`${command}\` — running outside Tauri? ${String(err)}`)
  return null
}

function isCounts(value: unknown): value is ExportScopeCounts {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ExportScopeCounts>
  return (
    typeof v.chapter_count === 'number' &&
    typeof v.segment_count === 'number' &&
    typeof v.unconfirmed_count === 'number'
  )
}

export async function exportScopeSummary(scope: ExportScope): Promise<ExportScopeSummaryResult> {
  try {
    const raw = await invoke<unknown>(CMD_EXPORT_SCOPE_SUMMARY, { scope })
    if (!isCounts(raw)) {
      console.error(`[export] \`${CMD_EXPORT_SCOPE_SUMMARY}\` returned an unexpected shape: ${String(raw)}`)
      return { counts: null, error: UNKNOWN_IPC_ERROR }
    }
    return { counts: raw, error: null }
  } catch (err) {
    return { counts: null, error: failureOf(err, CMD_EXPORT_SCOPE_SUMMARY) }
  }
}

export async function exportChooseFolder(): Promise<ExportFolderResult> {
  try {
    const raw = await invoke<unknown>(CMD_EXPORT_CHOOSE_FOLDER)
    if (raw === null) return { outcome: 'cancelled' }
    if (typeof raw !== 'string') {
      console.error(`[export] \`${CMD_EXPORT_CHOOSE_FOLDER}\` returned a non-string path: ${String(raw)}`)
      return { outcome: 'error', error: UNKNOWN_IPC_ERROR }
    }
    return { outcome: 'picked', path: raw }
  } catch (err) {
    const error = failureOf(err, CMD_EXPORT_CHOOSE_FOLDER)
    return error === null ? { outcome: 'ipc_unavailable' } : { outcome: 'error', error }
  }
}
