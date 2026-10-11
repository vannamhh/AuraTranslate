import { Channel, invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'
import {
  hasIpcBridge,
  isAiTranslateUsageWire,
  isIpcError,
  UNKNOWN_IPC_ERROR,
} from './aitranslate'
import type { AiTranslateUsageWire } from './aitranslate'

export const PROOFREAD_FINDING_KINDS = ['spelling', 'grammar', 'mistranslation', 'too_loose', 'obscure'] as const
export type ProofreadFindingKind = (typeof PROOFREAD_FINDING_KINDS)[number]

export function isMeaningKind(kind: ProofreadFindingKind): boolean {
  return kind !== 'spelling' && kind !== 'grammar'
}

/** `start`/`end` are UTF-16 offsets into `scanned_text`, end exclusive. */
export type ProofreadFindingWire = {
  kind: ProofreadFindingKind
  start: number
  end: number
  explanation: string
  suggestion: string
}

export type ProofreadSegmentResultWire = {
  segment_id: number
  scanned_text: string
  findings: ProofreadFindingWire[]
  unlocated: number
}

export type ProofreadEventWire =
  | { kind: 'planned'; segments: number; batches: number }
  | { kind: 'batch'; usage: AiTranslateUsageWire | null; segments: ProofreadSegmentResultWire[] }

export type ProofreadOutcomeWire =
  | { state: 'not_configured' }
  | { state: 'cancelled'; not_scanned: number }
  | { state: 'done'; not_scanned: number }

const CMD_PROOFREAD = 'ai_proofread_segments'
const CMD_PROOFREAD_CANCEL = 'ai_proofread_cancel'

function isFindingWire(value: unknown): value is ProofreadFindingWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ProofreadFindingWire, unknown>>
  return (
    typeof v.kind === 'string' &&
    (PROOFREAD_FINDING_KINDS as readonly string[]).includes(v.kind) &&
    typeof v.start === 'number' &&
    typeof v.end === 'number' &&
    typeof v.explanation === 'string' &&
    typeof v.suggestion === 'string'
  )
}

function isSegmentResultWire(value: unknown): value is ProofreadSegmentResultWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ProofreadSegmentResultWire, unknown>>
  return (
    typeof v.segment_id === 'number' &&
    typeof v.scanned_text === 'string' &&
    Array.isArray(v.findings) &&
    v.findings.every(isFindingWire) &&
    typeof v.unlocated === 'number'
  )
}

export function isProofreadEventWire(value: unknown): value is ProofreadEventWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as { kind?: unknown; segments?: unknown; batches?: unknown; usage?: unknown }
  if (v.kind === 'planned') return typeof v.segments === 'number' && typeof v.batches === 'number'
  if (v.kind !== 'batch') return false
  return (
    (v.usage === null || isAiTranslateUsageWire(v.usage)) &&
    Array.isArray(v.segments) &&
    v.segments.every(isSegmentResultWire)
  )
}

export function isProofreadOutcomeWire(value: unknown): value is ProofreadOutcomeWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as { state?: unknown; not_scanned?: unknown }
  if (v.state === 'not_configured') return true
  return (v.state === 'done' || v.state === 'cancelled') && typeof v.not_scanned === 'number'
}

/** Never throws; `onEvent` receives only well-shaped events and may run zero times. */
export async function runProofreadSegments(
  segmentIds: readonly number[],
  onEvent: (event: ProofreadEventWire) => void,
): Promise<{ value: ProofreadOutcomeWire | null; error: IpcError | null }> {
  const channel = new Channel<unknown>()
  channel.onmessage = (raw) => {
    if (!isProofreadEventWire(raw)) {
      console.error(`[proofread] \`${CMD_PROOFREAD}\` sent an event that is not ProofreadEventWire`)
      return
    }
    onEvent(raw)
  }

  try {
    const wire = await invoke<unknown>(CMD_PROOFREAD, { segmentIds: [...segmentIds], channel })
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
