/**
 * IPC adapter for the shortcut file dialogs (FR144, AD-48). Rust only shows the native
 * dialog and moves bytes; the webview validates the content before any write.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

const CMD_EXPORT = 'shortcuts_export'
const CMD_IMPORT = 'shortcuts_import'

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- IPC value, see `./pinned.ts`
    v.params !== null
  )
}

function hasIpcBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

function failure(err: unknown, cmd: string): IpcError | null {
  if (isIpcError(err)) return err
  if (hasIpcBridge()) {
    console.error(`[shortcuts] \`${cmd}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
    return UNKNOWN_IPC_ERROR
  }
  console.info(`[shortcuts] không gọi được \`${cmd}\` — chạy ngoài Tauri? ${String(err)}`)
  return null
}

/** Save dialog then write `content`. `path: null, error: null` == dialog cancelled or no bridge. */
export async function shortcutsExport(content: string): Promise<{ path: string | null; error: IpcError | null }> {
  try {
    const path = await invoke<string | null>(CMD_EXPORT, { content })
    return { path: typeof path === 'string' ? path : null, error: null }
  } catch (err) {
    return { path: null, error: failure(err, CMD_EXPORT) }
  }
}

/** Open dialog then read the file. `content: null, error: null` == dialog cancelled or no bridge. */
export async function shortcutsImport(): Promise<{ content: string | null; error: IpcError | null }> {
  try {
    const content = await invoke<string | null>(CMD_IMPORT)
    return { content: typeof content === 'string' ? content : null, error: null }
  } catch (err) {
    return { content: null, error: failure(err, CMD_IMPORT) }
  }
}
