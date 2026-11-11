import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'
import { hasIpcBridge, isIpcError, UNKNOWN_IPC_ERROR } from './aitranslate'

const CMD_STATS = 'quick_translation_stats'
const CMD_COUNT_OLDER = 'quick_translation_count_older'
const CMD_PURGE = 'quick_translation_purge'

export type QuickTranslationModelCountWire = { model: string; count: number }

export type QuickTranslationStatsWire = {
  count: number
  bytes: number
  by_model: QuickTranslationModelCountWire[]
}

export type QuickTranslationPurgeScope = { kind: 'all' } | { kind: 'older_than'; days: number }

export type QuickTranslationPurgeWire = {
  deleted: number
  bytes_before: number
  bytes_after: number
  compact_error: string | null
}

function isCount(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value >= 0
}

export function isQuickTranslationStatsWire(value: unknown): value is QuickTranslationStatsWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Record<string, unknown>
  return (
    isCount(v.count) &&
    isCount(v.bytes) &&
    Array.isArray(v.by_model) &&
    v.by_model.every((row: unknown) => {
      if (typeof row !== 'object' || row === null) return false
      const r = row as Record<string, unknown>
      return typeof r.model === 'string' && isCount(r.count)
    })
  )
}

export function isQuickTranslationPurgeWire(value: unknown): value is QuickTranslationPurgeWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Record<string, unknown>
  return (
    isCount(v.deleted) &&
    isCount(v.bytes_before) &&
    isCount(v.bytes_after) &&
    (v.compact_error === null || typeof v.compact_error === 'string')
  )
}

type Failure = { error: IpcError | null }

function failure(err: unknown, command: string): Failure {
  if (isIpcError(err)) return { error: err }
  if (hasIpcBridge()) {
    console.error(`[quick-translation-data] \`${command}\` failed with a non-IpcError: ${String(err)}`)
    return { error: UNKNOWN_IPC_ERROR }
  }
  console.info(`[quick-translation-data] \`${command}\` not callable, running outside Tauri? ${String(err)}`)
  return { error: null }
}

export async function quickTranslationStats(): Promise<{ value: QuickTranslationStatsWire | null; error: IpcError | null }> {
  try {
    const wire: unknown = await invoke(CMD_STATS)
    if (isQuickTranslationStatsWire(wire)) return { value: wire, error: null }
    console.error(`[quick-translation-data] \`${CMD_STATS}\` returned an unexpected shape`)
    return { value: null, error: UNKNOWN_IPC_ERROR }
  } catch (err) {
    return { value: null, ...failure(err, CMD_STATS) }
  }
}

export async function quickTranslationCountOlder(days: number): Promise<{ value: number | null; error: IpcError | null }> {
  try {
    const wire: unknown = await invoke(CMD_COUNT_OLDER, { days })
    if (isCount(wire)) return { value: wire, error: null }
    console.error(`[quick-translation-data] \`${CMD_COUNT_OLDER}\` returned an unexpected shape`)
    return { value: null, error: UNKNOWN_IPC_ERROR }
  } catch (err) {
    return { value: null, ...failure(err, CMD_COUNT_OLDER) }
  }
}

export async function quickTranslationPurge(
  scope: QuickTranslationPurgeScope,
): Promise<{ value: QuickTranslationPurgeWire | null; error: IpcError | null }> {
  try {
    const wire: unknown = await invoke(CMD_PURGE, { scope })
    if (isQuickTranslationPurgeWire(wire)) return { value: wire, error: null }
    console.error(`[quick-translation-data] \`${CMD_PURGE}\` returned an unexpected shape`)
    return { value: null, error: UNKNOWN_IPC_ERROR }
  } catch (err) {
    return { value: null, ...failure(err, CMD_PURGE) }
  }
}
