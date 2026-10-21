/**
 * IPC adapter for the manual update check (FR111, AD-15 outbound point 2). Rust reads the
 * release list and decides; the webview only shows the outcome and asks Rust to open the page.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

const CMD_CHECK = 'update_check'
const CMD_OPEN = 'update_open_release_page'

export type UpdateCheckOutcome =
  | { kind: 'up_to_date'; current: string }
  | { kind: 'update_available'; latest: string }
  | { kind: 'no_app_release' }

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

const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

function failure(err: unknown, cmd: string): IpcError | null {
  if (isIpcError(err)) return err
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    console.error(`[update] \`${cmd}\` failed with a non-IpcError value: ${String(err)}`)
    return UNKNOWN_IPC_ERROR
  }
  console.info(`[update] cannot call \`${cmd}\` outside Tauri: ${String(err)}`)
  return null
}

function parseOutcome(value: unknown): UpdateCheckOutcome | null {
  if (typeof value !== 'object' || value === null) return null
  const v = value as Record<string, unknown>
  if (v.kind === 'no_app_release') return { kind: 'no_app_release' }
  if (v.kind === 'up_to_date' && typeof v.current === 'string') return { kind: 'up_to_date', current: v.current }
  if (v.kind === 'update_available' && typeof v.latest === 'string') return { kind: 'update_available', latest: v.latest }
  return null
}

/** `outcome: null, error: null` == no bridge (plain browser). */
export async function updateCheck(): Promise<{ outcome: UpdateCheckOutcome | null; error: IpcError | null }> {
  try {
    const raw = await invoke<unknown>(CMD_CHECK)
    const outcome = parseOutcome(raw)
    if (outcome === null) {
      console.error(`[update] \`${CMD_CHECK}\` returned an unknown shape`)
      return { outcome: null, error: UNKNOWN_IPC_ERROR }
    }
    return { outcome, error: null }
  } catch (err) {
    return { outcome: null, error: failure(err, CMD_CHECK) }
  }
}

/** `version` is `X.Y.Z` without the `v`; Rust validates it and builds the URL. */
export async function updateOpenReleasePage(version: string): Promise<IpcError | null> {
  try {
    await invoke<void>(CMD_OPEN, { version })
    return null
  } catch (err) {
    return failure(err, CMD_OPEN)
  }
}
