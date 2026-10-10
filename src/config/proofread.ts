import { Channel, invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'
import {
  hasIpcBridge,
  isAiTranslateUsageWire,
  isIpcError,
  UNKNOWN_IPC_ERROR,
} from './aitranslate'
import type { AiTranslateUsageWire } from './aitranslate'

export type ProofreadFindingKind = 'spelling' | 'grammar'

/** `start`/`end` are UTF-16 offsets into `scanned_text`, end exclusive. */
export type ProofreadFindingWire = {
  kind: ProofreadFindingKind
  start: number
  end: number
  explanation: string
  suggestion: string
}

export type ProofreadOutcomeWire =
  | { state: 'not_configured' }
  | { state: 'cancelled' }
  | {
      state: 'done'
      usage: AiTranslateUsageWire | null
      scanned_text: string
      findings: ProofreadFindingWire[]
      unlocated: number
    }

const CMD_PROOFREAD = 'ai_proofread_segment'
const CMD_PROOFREAD_CANCEL = 'ai_proofread_cancel'

function isFindingWire(value: unknown): value is ProofreadFindingWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ProofreadFindingWire, unknown>>
  return (
    (v.kind === 'spelling' || v.kind === 'grammar') &&
    typeof v.start === 'number' &&
    typeof v.end === 'number' &&
    typeof v.explanation === 'string' &&
    typeof v.suggestion === 'string'
  )
}

export function isProofreadOutcomeWire(value: unknown): value is ProofreadOutcomeWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as {
    state?: unknown
    usage?: unknown
    scanned_text?: unknown
    findings?: unknown
    unlocated?: unknown
  }
  if (v.state === 'not_configured' || v.state === 'cancelled') return true
  if (v.state !== 'done') return false
  return (
    (v.usage === null || isAiTranslateUsageWire(v.usage)) &&
    typeof v.scanned_text === 'string' &&
    Array.isArray(v.findings) &&
    v.findings.every(isFindingWire) &&
    typeof v.unlocated === 'number'
  )
}

/** Never throws; `onToken` may run zero times. */
export async function runProofreadSegment(
  segmentId: number,
  onToken: (text: string) => void,
): Promise<{ value: ProofreadOutcomeWire | null; error: IpcError | null }> {
  const channel = new Channel<string>()
  channel.onmessage = onToken

  try {
    const wire = await invoke<unknown>(CMD_PROOFREAD, { segmentId, channel })
    if (!isProofreadOutcomeWire(wire)) {
      console.error(`[proofread] \`${CMD_PROOFREAD}\` returned a shape that is not ProofreadOutcomeWire`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    return { value: wire, error: null }
  } catch (err) {
    if (isIpcError(err)) return { value: null, error: err }
    if (hasIpcBridge()) {
      console.error(`[proofread] \`${CMD_PROOFREAD}\` failed with a non-IpcError: ${String(err)}`)
      return { value: null, error: UNKNOWN_IPC_ERROR }
    }
    console.info(`[proofread] cannot call \`${CMD_PROOFREAD}\` outside Tauri: ${String(err)}`)
    return { value: null, error: null }
  }
}

export async function cancelProofreadCall(): Promise<void> {
  try {
    await invoke<void>(CMD_PROOFREAD_CANCEL)
  } catch (err) {
    console.error(`[proofread] \`${CMD_PROOFREAD_CANCEL}\` failed: ${String(err)}`)
  }
}
