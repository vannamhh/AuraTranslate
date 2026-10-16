import { Channel, invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'
import {
  hasIpcBridge,
  isAiTranslateUsageWire,
  isIpcError,
  UNKNOWN_IPC_ERROR,
} from './aitranslate'
import type { AiTranslateUsageWire } from './aitranslate'
import type { PromoteAiTranslationOutcome } from './segment'

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
  ambiguous: number
}

/** Findings an ignore entry silenced during one scan; only entries with `count > 0` are sent. */
export type ProofreadBlockedWire = { ignore_id: number; count: number }

function isBlockedWire(value: unknown): value is ProofreadBlockedWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ProofreadBlockedWire, unknown>>
  return isCount(v.ignore_id) && isCount(v.count)
}

export type ProofreadEventWire =
  | { kind: 'planned'; segments: number; batches: number }
  | { kind: 'batch'; usage: AiTranslateUsageWire | null; segments: ProofreadSegmentResultWire[] }

export type ProofreadOutcomeWire =
  | { state: 'not_configured' }
  | { state: 'cancelled'; not_scanned: number }
  | { state: 'done'; not_scanned: number; filtered: number; blocked: ProofreadBlockedWire[] }

const CMD_PROOFREAD = 'ai_proofread_segments'
const CMD_PROOFREAD_CANCEL = 'ai_proofread_cancel'
const CMD_PROOFREAD_ACCEPT = 'ai_proofread_accept'
const CMD_PROOFREAD_REVERT = 'ai_proofread_revert'
const CMD_IGNORE_ADD = 'ai_proofread_ignore_add'
const CMD_IGNORE_LIST = 'ai_proofread_ignore_list'
const CMD_IGNORE_REMOVE = 'ai_proofread_ignore_remove'
const CMD_IGNORE_CLEAR = 'ai_proofread_ignore_clear'
const CMD_SCAN_RECORD = 'ai_proofread_scan_record'
const CMD_DECISION_SKIP = 'ai_proofread_decision_skip'
const CMD_DECISION_COUNTS = 'ai_proofread_decision_counts'

function isCount(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value >= 0
}

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
    typeof v.unlocated === 'number' &&
    typeof v.ambiguous === 'number'
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
  const v = value as { state?: unknown; not_scanned?: unknown; filtered?: unknown; blocked?: unknown }
  if (v.state === 'not_configured') return true
  if (v.state === 'done') {
    return (
      typeof v.not_scanned === 'number' &&
      isCount(v.filtered) &&
      Array.isArray(v.blocked) &&
      v.blocked.every(isBlockedWire)
    )
  }
  return v.state === 'cancelled' && typeof v.not_scanned === 'number'
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

/** `start`/`end` are UTF-16 offsets into `expectedText`; `expectedPhrase` is what `[start, end)` holds there. */
export type ProofreadRangeWrite = {
  segmentId: number
  expectedText: string
  start: number
  end: number
  expectedPhrase: string
  replacement: string
  kind: ProofreadFindingKind
}

/** `decisionId` is set only by accept; revert hands it back to Rust. */
export type ProofreadWriteResult = {
  outcome: PromoteAiTranslationOutcome | null
  decisionId: number | null
  error: IpcError | null
}

type WriteWhich = 'accept' | 'revert'

function isWriteOutcome(value: unknown): value is PromoteAiTranslationOutcome {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof PromoteAiTranslationOutcome, unknown>>
  return (
    typeof v.segment_id === 'number' &&
    typeof v.target_text === 'string' &&
    typeof v.translation_origin === 'string' &&
    (v.status === 'draft' || v.status === 'confirmed') &&
    typeof v.needs_confirmation === 'boolean' &&
    (typeof v.unsigned_draft === 'string' || v.unsigned_draft === null)
  )
}

function writeFailure(which: WriteWhich, err: unknown): ProofreadWriteResult {
  const command = which === 'accept' ? CMD_PROOFREAD_ACCEPT : CMD_PROOFREAD_REVERT
  if (isIpcError(err)) return { outcome: null, decisionId: null, error: err }
  if (hasIpcBridge()) {
    console.error(`[proofread] \`${command}\` failed with a non-IpcError: ${String(err)}`)
    return { outcome: null, decisionId: null, error: UNKNOWN_IPC_ERROR }
  }
  console.info(`[proofread] cannot call \`${command}\` outside Tauri: ${String(err)}`)
  return { outcome: null, decisionId: null, error: null }
}

function checkedOutcome(which: WriteWhich, wire: unknown): ProofreadWriteResult {
  const command = which === 'accept' ? CMD_PROOFREAD_ACCEPT : CMD_PROOFREAD_REVERT
  if (!isWriteOutcome(wire)) {
    console.error(`[proofread] \`${command}\` returned a shape that is not a write outcome`)
    return { outcome: null, decisionId: null, error: UNKNOWN_IPC_ERROR }
  }
  if (which === 'revert') return { outcome: wire, decisionId: null, error: null }
  const decisionId = (wire as { decision_id?: unknown }).decision_id
  if (!isCount(decisionId)) {
    console.error(`[proofread] \`${command}\` returned no decision_id`)
    return { outcome: null, decisionId: null, error: UNKNOWN_IPC_ERROR }
  }
  return { outcome: wire, decisionId, error: null }
}

/** Replaces `[start, end)` with `replacement`; Rust refuses unless the text and phrase still match. Never throws. */
export async function acceptProofreadSuggestion(args: ProofreadRangeWrite): Promise<ProofreadWriteResult> {
  try {
    const wire = await invoke<unknown>(CMD_PROOFREAD_ACCEPT, {
      segmentId: args.segmentId,
      expectedText: args.expectedText,
      start: args.start,
      end: args.end,
      expectedPhrase: args.expectedPhrase,
      replacement: args.replacement,
      kind: args.kind,
    })
    return checkedOutcome('accept', wire)
  } catch (err) {
    return writeFailure('accept', err)
  }
}

/** Same replacement with the roles swapped: `expectedPhrase` is the suggestion now in the text. Never throws. */
export async function revertProofreadSuggestion(
  args: ProofreadRangeWrite & { decisionId: number },
): Promise<ProofreadWriteResult> {
  try {
    const wire = await invoke<unknown>(CMD_PROOFREAD_REVERT, {
      segmentId: args.segmentId,
      expectedText: args.expectedText,
      start: args.start,
      end: args.end,
      expectedPhrase: args.expectedPhrase,
      replacement: args.replacement,
      kind: args.kind,
      decisionId: args.decisionId,
    })
    return checkedOutcome('revert', wire)
  } catch (err) {
    return writeFailure('revert', err)
  }
}

/** One remembered "not an error" signature of the open Work; spelling rows carry empty contexts. */
export type ProofreadIgnoreRowWire = {
  id: number
  kind: ProofreadFindingKind
  phrase: string
  context_before: string
  context_after: string
  created_at: string
  /** `null` = the row predates the counter: show the date only, never `0`. */
  blocked_count: number | null
}

export type ProofreadScanScope = 'segment' | 'selection' | 'chapter'

export type ProofreadIgnoreAdd = Omit<ProofreadRangeWrite, 'replacement'>

function isIgnoreRowWire(value: unknown): value is ProofreadIgnoreRowWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ProofreadIgnoreRowWire, unknown>>
  return (
    typeof v.id === 'number' &&
    typeof v.kind === 'string' &&
    (PROOFREAD_FINDING_KINDS as readonly string[]).includes(v.kind) &&
    typeof v.phrase === 'string' &&
    typeof v.context_before === 'string' &&
    typeof v.context_after === 'string' &&
    typeof v.created_at === 'string' &&
    (v.blocked_count === null || isCount(v.blocked_count))
  )
}

function memoryFailure(label: string, err: unknown): IpcError | null {
  if (isIpcError(err)) return err
  if (hasIpcBridge()) {
    console.error(`[proofread] \`ai_proofread_${label}\` failed with a non-IpcError: ${String(err)}`)
    return UNKNOWN_IPC_ERROR
  }
  console.info(`[proofread] cannot call \`ai_proofread_${label}\` outside Tauri: ${String(err)}`)
  return null
}

/** Remembers "not a mistake" for the phrase at `[start, end)`. `ok` is false on any failure. Never throws. */
export async function addProofreadIgnore(args: ProofreadIgnoreAdd): Promise<{ ok: boolean; error: IpcError | null }> {
  try {
    await invoke<unknown>(CMD_IGNORE_ADD, {
      segmentId: args.segmentId,
      expectedText: args.expectedText,
      start: args.start,
      end: args.end,
      expectedPhrase: args.expectedPhrase,
      kind: args.kind,
    })
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: memoryFailure('ignore_add', err) }
  }
}

/** Newest first. `rows` is null on failure. Never throws. */
export async function listProofreadIgnores(): Promise<{ rows: ProofreadIgnoreRowWire[] | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_IGNORE_LIST)
    if (!Array.isArray(wire) || !wire.every(isIgnoreRowWire)) {
      console.error(`[proofread] \`${CMD_IGNORE_LIST}\` returned a shape that is not a list of ProofreadIgnoreRowWire`)
      return { rows: null, error: UNKNOWN_IPC_ERROR }
    }
    return { rows: wire, error: null }
  } catch (err) {
    return { rows: null, error: memoryFailure('ignore_list', err) }
  }
}

/** `removed` is false when the row was already gone. Never throws. */
export async function removeProofreadIgnore(id: number): Promise<{ removed: boolean | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_IGNORE_REMOVE, { id })
    if (typeof wire !== 'boolean') {
      console.error(`[proofread] \`${CMD_IGNORE_REMOVE}\` returned a shape that is not a boolean`)
      return { removed: null, error: UNKNOWN_IPC_ERROR }
    }
    return { removed: wire, error: null }
  } catch (err) {
    return { removed: null, error: memoryFailure('ignore_remove', err) }
  }
}

/** Rust clears without asking; the two-click guard lives in the caller. Never throws. */
export async function clearProofreadIgnores(): Promise<{ cleared: number | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_IGNORE_CLEAR)
    if (!isCount(wire)) {
      console.error(`[proofread] \`${CMD_IGNORE_CLEAR}\` returned a shape that is not a row count`)
      return { cleared: null, error: UNKNOWN_IPC_ERROR }
    }
    return { cleared: wire, error: null }
  } catch (err) {
    return { cleared: null, error: memoryFailure('ignore_clear', err) }
  }
}

/** One row per finished scan; Rust rejects a Work that is no longer open. Never throws. */
export async function recordProofreadScan(
  workId: string,
  scope: ProofreadScanScope,
  reported: number,
  filtered: number,
  blocked: readonly ProofreadBlockedWire[],
): Promise<{ ok: boolean; error: IpcError | null }> {
  try {
    await invoke<unknown>(CMD_SCAN_RECORD, { workId, scope, reported, filtered, blocked })
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: memoryFailure('scan_record', err) }
  }
}

export type ProofreadDecisionTally = { accepted: number; not_error: number; skipped: number }

export type ProofreadDecisionCountsWire = ProofreadDecisionTally & {
  by_kind: (ProofreadDecisionTally & { kind: ProofreadFindingKind })[]
}

function isTallyWire(value: unknown): value is ProofreadDecisionTally {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ProofreadDecisionTally, unknown>>
  return isCount(v.accepted) && isCount(v.not_error) && isCount(v.skipped)
}

function isKindTallyWire(value: unknown): value is ProofreadDecisionTally & { kind: ProofreadFindingKind } {
  if (!isTallyWire(value)) return false
  const kind = (value as unknown as { kind?: unknown }).kind
  return typeof kind === 'string' && (PROOFREAD_FINDING_KINDS as readonly string[]).includes(kind)
}

function isDecisionCountsWire(value: unknown): value is ProofreadDecisionCountsWire {
  if (!isTallyWire(value)) return false
  const byKind = (value as unknown as { by_kind?: unknown }).by_kind
  return Array.isArray(byKind) && byKind.every(isKindTallyWire)
}

/** Records one "Bỏ qua" decision; touches no segment and creates no memory. `ok` is false on any failure. Never throws. */
export async function skipProofreadDecision(kind: ProofreadFindingKind): Promise<{ ok: boolean; error: IpcError | null }> {
  try {
    await invoke<unknown>(CMD_DECISION_SKIP, { kind })
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: memoryFailure('decision_skip', err) }
  }
}

/** Counts of the open Work's decisions. `counts` is null on failure, never a zeroed stand-in. Never throws. */
export async function getProofreadDecisionCounts(): Promise<{
  counts: ProofreadDecisionCountsWire | null
  error: IpcError | null
}> {
  try {
    const wire = await invoke<unknown>(CMD_DECISION_COUNTS)
    if (!isDecisionCountsWire(wire)) {
      console.error(`[proofread] \`${CMD_DECISION_COUNTS}\` returned a shape that is not ProofreadDecisionCountsWire`)
      return { counts: null, error: UNKNOWN_IPC_ERROR }
    }
    return { counts: wire, error: null }
  } catch (err) {
    return { counts: null, error: memoryFailure('decision_counts', err) }
  }
}
