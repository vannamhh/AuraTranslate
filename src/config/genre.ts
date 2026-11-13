import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

export type GenreWire = { id: string; name: string }

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- runtime guard on IPC data
    v.params !== null
  )
}

function isGenreWire(value: unknown): value is GenreWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<GenreWire>
  return typeof v.id === 'string' && typeof v.name === 'string'
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

const CMD_LIST = 'genre_list'
const CMD_CREATE = 'genre_create'
const CMD_RENAME = 'genre_rename'
const CMD_DELETE = 'genre_delete'

function failure(operation: string, err: unknown): IpcError | null {
  if (isIpcError(err)) return err
  if (hasIpcBridge()) {
    console.error(`[genre] \`genre_${operation}\` failed with a non-IpcError: ${String(err)}`)
    return UNKNOWN_IPC_ERROR
  }
  console.info(`[genre] cannot call \`genre_${operation}\` outside Tauri: ${String(err)}`)
  return null
}

/** `genres: null` with `error: null` means there is no IPC bridge (plain browser). */
export async function genreList(): Promise<{ genres: GenreWire[] | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_LIST)
    if (!Array.isArray(wire) || !wire.every(isGenreWire)) {
      console.error(`[genre] \`${CMD_LIST}\` returned an unexpected shape`)
      return { genres: null, error: UNKNOWN_IPC_ERROR }
    }
    return { genres: wire, error: null }
  } catch (err) {
    return { genres: null, error: failure('list', err) }
  }
}

export async function genreCreate(name: string): Promise<{ genre: GenreWire | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_CREATE, { name })
    if (!isGenreWire(wire)) {
      console.error(`[genre] \`${CMD_CREATE}\` returned an unexpected shape`)
      return { genre: null, error: UNKNOWN_IPC_ERROR }
    }
    return { genre: wire, error: null }
  } catch (err) {
    return { genre: null, error: failure('create', err) }
  }
}

export async function genreRename(id: string, newName: string): Promise<IpcError | null> {
  try {
    await invoke(CMD_RENAME, { id, newName })
    return null
  } catch (err) {
    return failure('rename', err)
  }
}

export async function genreDelete(id: string): Promise<IpcError | null> {
  try {
    await invoke(CMD_DELETE, { id })
    return null
  } catch (err) {
    return failure('delete', err)
  }
}
