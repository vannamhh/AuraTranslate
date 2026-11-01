import { Channel, invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'
import { hasIpcBridge, isAiTranslateOutcomeWire, isIpcError, UNKNOWN_IPC_ERROR } from './aitranslate'
import type { AiTranslateOutcomeWire } from './aitranslate'

const CMD_LOOKUP = 'glossary_context_translation'
const CMD_TRANSLATE = 'ai_context_reference_translate'
const CMD_CANCEL = 'ai_context_reference_cancel'

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

/** One AI translation of `contextExample`, streamed into `onToken`. Never throws. */
export async function translateContextReference(
  contextExample: string,
  onToken: (text: string) => void,
): Promise<{ value: AiTranslateOutcomeWire | null; error: IpcError | null }> {
  const channel = new Channel<string>()
  channel.onmessage = onToken
  try {
    const wire: unknown = await invoke(CMD_TRANSLATE, { contextExample, channel })
    if (!isAiTranslateOutcomeWire(wire)) {
      console.error(`[glossary-context] \`${CMD_TRANSLATE}\` returned an unexpected shape`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    return { value: wire, error: null }
  } catch (err) {
    return { value: null, ...failure(err, CMD_TRANSLATE) }
  }
}

export async function cancelContextReference(): Promise<void> {
  try {
    await invoke<void>(CMD_CANCEL)
  } catch (err) {
    console.error(`[glossary-context] \`${CMD_CANCEL}\` failed: ${String(err)}`)
  }
}
