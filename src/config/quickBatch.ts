import { Channel, invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'
import { hasIpcBridge, isAiTranslateUsageWire, isIpcError, UNKNOWN_IPC_ERROR } from './aitranslate'
import type { AiTranslateUsageWire } from './aitranslate'
import { isQuickLookupWire } from './glossaryContextReference'
import type { QuickLookupWire } from './glossaryContextReference'

const CMD_BLOCKS = 'quick_translate_blocks'
const CMD_LOOKUP_MANY = 'quick_translation_lookup_many'

export type QuickBlockEventWire =
  | { kind: 'cached'; index: number; text: string; model: string }
  | { kind: 'started'; indices: number[] }
  | { kind: 'token'; index: number; text: string }
  | { kind: 'received'; index: number; text: string; model: string; saved: boolean }

export type QuickBlockMissingWire = { index: number; reason: string }

export type QuickBatchOutcomeWire =
  | { state: 'not_configured' }
  | { state: 'store_unavailable' }
  | { state: 'busy' }
  | { state: 'auto_off' }
  | { state: 'paused' }
  | { state: 'cancelled' }
  | {
      state: 'done'
      model: string | null
      usage: AiTranslateUsageWire | null
      missing: QuickBlockMissingWire[]
      deferred: number
    }

function isIndex(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value >= 0
}

export function isQuickBlockEventWire(value: unknown): value is QuickBlockEventWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Record<string, unknown>
  switch (v.kind) {
    case 'cached':
      return isIndex(v.index) && typeof v.text === 'string' && typeof v.model === 'string'
    case 'started':
      return Array.isArray(v.indices) && v.indices.every(isIndex)
    case 'token':
      return isIndex(v.index) && typeof v.text === 'string'
    case 'received':
      return (
        isIndex(v.index) && typeof v.text === 'string' && typeof v.model === 'string' && typeof v.saved === 'boolean'
      )
    default:
      return false
  }
}

export function isQuickBatchOutcomeWire(value: unknown): value is QuickBatchOutcomeWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Record<string, unknown>
  switch (v.state) {
    case 'not_configured':
    case 'store_unavailable':
    case 'auto_off':
    case 'paused':
    case 'busy':
    case 'cancelled':
      return true
    case 'done':
      return (
        (v.model === null || typeof v.model === 'string') &&
        (v.usage === null || isAiTranslateUsageWire(v.usage)) &&
        Array.isArray(v.missing) &&
        v.missing.every((m: unknown) => {
          const item = m as Record<string, unknown> | null
          return typeof item === 'object' && item !== null && isIndex(item.index) && typeof item.reason === 'string'
        }) &&
        isIndex(v.deferred)
      )
    default:
      return false
  }
}

function failure(err: unknown, command: string): IpcError | null {
  if (isIpcError(err)) return err
  if (hasIpcBridge()) {
    console.error(`[quick-batch] \`${command}\` failed with a non-IpcError: ${String(err)}`)
    return UNKNOWN_IPC_ERROR
  }
  console.info(`[quick-batch] \`${command}\` not callable, running outside Tauri? ${String(err)}`)
  return null
}

/** The shared store's translation of each text, in the order sent. Never throws. */
export async function quickTranslationLookupMany(
  texts: readonly string[],
): Promise<{ value: QuickLookupWire[] | null; error: IpcError | null }> {
  try {
    const wire: unknown = await invoke(CMD_LOOKUP_MANY, { texts })
    if (!Array.isArray(wire) || wire.length !== texts.length || !wire.every(isQuickLookupWire)) {
      console.error(`[quick-batch] \`${CMD_LOOKUP_MANY}\` returned an unexpected shape`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    return { value: wire, error: null }
  } catch (err) {
    return { value: null, error: failure(err, CMD_LOOKUP_MANY) }
  }
}

/** One batch of page blocks; events stream into `onEvent`, indices are positions in `texts`.
 * `retranslate` skips the store and replaces the saved rows. Never throws. */
export async function translateQuickBlocks(
  texts: readonly string[],
  runId: number,
  onEvent: (event: QuickBlockEventWire) => void,
  retranslate = false,
  trigger: 'auto' | 'manual' = 'manual',
): Promise<{ value: QuickBatchOutcomeWire | null; error: IpcError | null }> {
  const channel = new Channel<unknown>()
  channel.onmessage = (event) => {
    if (isQuickBlockEventWire(event)) onEvent(event)
    else console.error(`[quick-batch] \`${CMD_BLOCKS}\` sent an unexpected event`)
  }
  try {
    const wire: unknown = await invoke(CMD_BLOCKS, { texts, trigger, retranslate, runId, channel })
    if (!isQuickBatchOutcomeWire(wire)) {
      console.error(`[quick-batch] \`${CMD_BLOCKS}\` returned an unexpected shape`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    return { value: wire, error: null }
  } catch (err) {
    return { value: null, error: failure(err, CMD_BLOCKS) }
  }
}
