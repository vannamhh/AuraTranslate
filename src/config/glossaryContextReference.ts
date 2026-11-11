import { Channel, invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'
import { hasIpcBridge, isAiTranslateUsageWire, isIpcError, UNKNOWN_IPC_ERROR } from './aitranslate'
import type { AiTranslateUsageWire } from './aitranslate'

const CMD_LOOKUP = 'glossary_context_translation'
const CMD_TRANSLATE = 'ai_context_reference_translate'
const CMD_CANCEL = 'ai_context_reference_cancel'
const CMD_QUICK_LOOKUP = 'quick_translation_lookup'

export type QuickTranslateOutcomeWire =
  | { state: 'not_configured' }
  | { state: 'store_unavailable' }
  | { state: 'cached'; text: string; model: string }
  | { state: 'done'; model: string; usage: AiTranslateUsageWire | null; maybe_truncated: boolean }
  | { state: 'cancelled' }

export type QuickLookupWire =
  | { state: 'miss' }
  | { state: 'hit'; text: string; model: string }
  | { state: 'store_unavailable' }

export function isQuickTranslateOutcomeWire(value: unknown): value is QuickTranslateOutcomeWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Record<string, unknown>
  switch (v.state) {
    case 'not_configured':
    case 'store_unavailable':
    case 'cancelled':
      return true
    case 'cached':
      return typeof v.text === 'string' && typeof v.model === 'string'
    case 'done':
      return (
        typeof v.model === 'string' &&
        (v.usage === null || isAiTranslateUsageWire(v.usage)) &&
        typeof v.maybe_truncated === 'boolean'
      )
    default:
      return false
  }
}

export function isQuickLookupWire(value: unknown): value is QuickLookupWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Record<string, unknown>
  if (v.state === 'miss' || v.state === 'store_unavailable') return true
  return v.state === 'hit' && typeof v.text === 'string' && typeof v.model === 'string'
}

type Failure = { error: IpcError | null }

function failure(err: unknown, command: string): Failure {
  if (isIpcError(err)) return { error: err }
  if (hasIpcBridge()) {
    console.error(`[glossary-context] \`${command}\` failed with a non-IpcError: ${String(err)}`)
    return { error: UNKNOWN_IPC_ERROR }
  }
  console.info(`[glossary-context] \`${command}\` not callable, running outside Tauri? ${String(err)}`)
  return { error: null }
}

/**
 * The Work's own translation of the sentence `contextExample`, or `null` for none. `error` is
 * set only when the lookup failed, so `null` with no error is a real "no translation".
 * Never throws.
 */
export async function lookupContextTranslation(
  contextExample: string,
): Promise<{ value: string | null; failed: boolean; error: IpcError | null }> {
  try {
    const wire: unknown = await invoke(CMD_LOOKUP, { contextExample })
    if (wire === null || typeof wire === 'string') return { value: wire, failed: false, error: null }
    console.error(`[glossary-context] \`${CMD_LOOKUP}\` returned an unexpected shape`)
    return { value: null, failed: true, error: UNKNOWN_IPC_ERROR }
  } catch (err) {
    return { value: null, failed: true, ...failure(err, CMD_LOOKUP) }
  }
}

/** The shared store's translation of `text`. Never throws. */
export async function quickTranslationLookup(
  text: string,
): Promise<{ value: QuickLookupWire | null; error: IpcError | null }> {
  try {
    const wire: unknown = await invoke(CMD_QUICK_LOOKUP, { text })
    if (!isQuickLookupWire(wire)) {
      console.error(`[glossary-context] \`${CMD_QUICK_LOOKUP}\` returned an unexpected shape`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    return { value: wire, error: null }
  } catch (err) {
    return { value: null, ...failure(err, CMD_QUICK_LOOKUP) }
  }
}

/** One AI translation of `contextExample`, streamed into `onToken`. Never throws. */
export async function translateContextReference(
  contextExample: string,
  runId: number,
  retranslate: boolean,
  onToken: (text: string) => void,
): Promise<{ value: QuickTranslateOutcomeWire | null; error: IpcError | null }> {
  const channel = new Channel<string>()
  channel.onmessage = onToken
  try {
    const wire: unknown = await invoke(CMD_TRANSLATE, { contextExample, runId, retranslate, channel })
    if (!isQuickTranslateOutcomeWire(wire)) {
      console.error(`[glossary-context] \`${CMD_TRANSLATE}\` returned an unexpected shape`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    return { value: wire, error: null }
  } catch (err) {
    return { value: null, ...failure(err, CMD_TRANSLATE) }
  }
}

export async function cancelContextReference(runId: number): Promise<void> {
  try {
    await invoke<void>(CMD_CANCEL, { runId })
  } catch (err) {
    console.error(`[glossary-context] \`${CMD_CANCEL}\` failed: ${String(err)}`)
  }
}
