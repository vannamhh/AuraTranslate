import { computed, readonly, ref, shallowRef } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import {
  acceptProofreadSuggestion,
  addProofreadIgnore,
  cancelProofreadCall,
  recordProofreadScan,
  revertProofreadSuggestion,
  runProofreadSegments,
  skipProofreadDecision,
} from './config/proofread'
import type {
  ProofreadBlockedWire,
  ProofreadEventWire,
  ProofreadFindingWire,
  ProofreadScanScope,
  ProofreadSegmentResultWire,
  ProofreadWriteResult,
} from './config/proofread'
import { openWorkId } from './openWorkIdentity'
import { reloadProofreadDecisions, resetProofreadDecisions } from './proofreadDecisionState'
import { resetProofreadMemory } from './proofreadMemoryState'
import type { AiUsageSummary } from './aiUsageLine'
import type { IpcError } from './i18n'
import {
  editorCaretSegmentId,
  editorSegments,
  flushEditorBeforeDiscreteWrite,
  moveEditorCaretTo,
  replaceEditorSegment,
} from './panels/editorPanelState'

export type ProofreadState = 'idle' | 'scanning' | 'done' | 'error' | 'cancelled' | 'not_configured'

export type ProofreadSegmentResult = {
  readonly scannedText: string
  readonly findings: readonly ProofreadFindingWire[]
  readonly unlocated: number
  readonly ambiguous: number
}

type RevertItem = {
  readonly decisionId: number
  readonly finding: ProofreadFindingWire
  readonly phrase: string
  readonly before: string
  start: number
  end: number
  text: string
}

type Span = { readonly start: number; readonly end: number }

const state = ref<ProofreadState>('idle')
const runSegmentIds = shallowRef<readonly number[]>([])
const results = shallowRef<ReadonlyMap<number, ProofreadSegmentResult>>(new Map())
const planned = ref(0)
const arrived = ref(0)
const notScanned = ref(0)
const filteredCount = ref(0)
const reportedFindings = ref(0)
const tokenCount = ref(0)
const reportedCount = ref(0)
const costUsdSum = ref(0)
const costMissing = ref(false)
const error = shallowRef<IpcError | null>(null)
const scopeOpen = ref(false)
const reverts = shallowRef<ReadonlyMap<number, readonly RevertItem[]>>(new Map())
const focus = shallowRef<{ segmentId: number; index: number }>({ segmentId: -1, index: 0 })
const acting = ref(false)
const actionFailure = shallowRef<{ segmentId: number; error: IpcError } | null>(null)

let sequence = 0
let flushingRun = 0
let editedWhileScanning = new Set<number>()

export const proofreadStateValue: DeepReadonly<Ref<ProofreadState>> = readonly(state)
export const proofreadRunSegmentIds: Readonly<Ref<readonly number[]>> = runSegmentIds
export const proofreadResults: Readonly<Ref<ReadonlyMap<number, ProofreadSegmentResult>>> = results
export const proofreadError: DeepReadonly<Ref<IpcError | null>> = readonly(error)
export const proofreadNotScanned: DeepReadonly<Ref<number>> = readonly(notScanned)
/** Findings Rust silenced from the remembered "not a mistake" list; zero while a scan runs. */
export const proofreadFiltered: DeepReadonly<Ref<number>> = readonly(filteredCount)
export const proofreadScopeIsOpen: DeepReadonly<Ref<boolean>> = readonly(scopeOpen)

export const proofreadProgress: ComputedRef<{ done: number; total: number }> = computed(() => ({
  done: arrived.value,
  total: planned.value,
}))

export const proofreadFindingCount: ComputedRef<number> = computed(() => {
  let count = 0
  for (const entry of results.value.values()) count += entry.findings.length
  return count
})

export const proofreadAmbiguous: ComputedRef<number> = computed(() => {
  let count = 0
  for (const entry of results.value.values()) count += entry.ambiguous
  return count
})

export const proofreadUnlocated: ComputedRef<number> = computed(() => {
  let count = 0
  for (const entry of results.value.values()) count += entry.unlocated
  return count
})

/** `sentenceCount` counts the segments of batches that arrived, `reportedCount` those whose batch carried usage. */
export const proofreadUsageSummary: ComputedRef<AiUsageSummary> = computed(() => ({
  tokenCount: tokenCount.value,
  reportedCount: reportedCount.value,
  sentenceCount: arrived.value,
  costUsd: reportedCount.value > 0 && !costMissing.value ? costUsdSum.value : null,
}))

function clearResult(): void {
  results.value = new Map()
  planned.value = 0
  arrived.value = 0
  notScanned.value = 0
  filteredCount.value = 0
  reportedFindings.value = 0
  tokenCount.value = 0
  reportedCount.value = 0
  costUsdSum.value = 0
  costMissing.value = false
  error.value = null
  actionFailure.value = null
  editedWhileScanning = new Set()
}

function applyEvent(event: ProofreadEventWire): void {
  if (event.kind === 'planned') {
    planned.value = event.segments
    return
  }
  const next = new Map(results.value)
  for (const segment of event.segments as readonly ProofreadSegmentResultWire[]) {
    arrived.value += 1
    reportedFindings.value += segment.findings.length
    if (editedWhileScanning.has(segment.segment_id)) continue
    next.set(segment.segment_id, {
      scannedText: segment.scanned_text,
      findings: [...segment.findings].sort((a, b) => a.start - b.start),
      unlocated: segment.unlocated,
      ambiguous: segment.ambiguous,
    })
  }
  results.value = next
  if (event.usage !== null) {
    reportedCount.value += event.segments.length
    tokenCount.value += event.usage.total_tokens
    if (event.usage.cost_usd === null) costMissing.value = true
    else costUsdSum.value += event.usage.cost_usd
  }
}

function notScannedFromError(err: IpcError): number {
  const parsed = Number.parseInt(String(err.params.not_scanned), 10)
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : 0
}

/** Flushes the Editor first so the scan reads the same `target_text` the user sees. */
export async function runProofread(segmentIds: readonly number[], scope: ProofreadScanScope | null = null): Promise<void> {
  if (state.value === 'scanning') {
    console.warn('[proofread] not scanning: a scan is already running')
    return
  }
  if (segmentIds.length === 0) {
    console.warn('[proofread] not scanning: the scope holds no segment')
    return
  }

  const mine = ++sequence
  hiddenForSegmentId.value = null
  runSegmentIds.value = [...segmentIds]
  clearResult()
  state.value = 'scanning'

  flushingRun = mine
  const flushed = await flushEditorBeforeDiscreteWrite()
  if (flushingRun === mine) flushingRun = 0
  if (mine !== sequence) return
  if (flushed !== 'clean') {
    state.value = 'error'
    error.value = {
      code: 'proofread.flush_failed',
      message_key: flushed === 'failed' ? 'proofread.flush_failed' : 'proofread.flush_still_dirty',
      params: {},
      retryable: true,
    }
    return
  }

  const result = await runProofreadSegments(segmentIds, (event) => {
    if (mine !== sequence) return
    applyEvent(event)
  })
  if (mine !== sequence) return

  if (result.error !== null) {
    state.value = 'error'
    error.value = result.error
    notScanned.value = notScannedFromError(result.error)
    return
  }
  if (result.value === null) {
    state.value = 'idle'
    return
  }
  if (result.value.state !== 'not_configured') notScanned.value = result.value.not_scanned
  if (result.value.state === 'done') {
    filteredCount.value = result.value.filtered
    if (scope !== null) {
      void recordFinishedScan(scope, reportedFindings.value, result.value.filtered, result.value.blocked)
    }
  }
  state.value = result.value.state
}

async function recordFinishedScan(
  scope: ProofreadScanScope,
  reported: number,
  filtered: number,
  blocked: readonly ProofreadBlockedWire[],
): Promise<void> {
  const workId = openWorkId.value
  if (workId === null) {
    console.warn('[proofread] scan not recorded: no open Work id known')
    return
  }
  const result = await recordProofreadScan(workId, scope, reported, filtered, blocked)
  if (!result.ok) console.warn(`[proofread] scan not recorded: ${result.error?.code ?? 'no IPC bridge'}`)
}

export function cancelProofread(): void {
  if (state.value !== 'scanning') return
  if (flushingRun === sequence) {
    // No Rust call is in flight yet; a cancel sent now would be lost to the next generation.
    sequence += 1
    state.value = 'cancelled'
    return
  }
  void cancelProofreadCall()
}

/** A typed edit in a scanned cell invalidates its underlines; other segments are untouched. */
export function clearProofreadFor(segmentId: number): void {
  if (!runSegmentIds.value.includes(segmentId)) return
  dropReverts(segmentId)
  if (state.value === 'scanning') {
    if (runSegmentIds.value.length === 1) {
      resetProofread()
      return
    }
    editedWhileScanning.add(segmentId)
    if (!results.value.has(segmentId)) return
  } else if (!results.value.has(segmentId)) {
    return
  }
  const next = new Map(results.value)
  next.delete(segmentId)
  results.value = next
  if (state.value === 'done' && runSegmentIds.value.length === 1) {
    sequence += 1
    state.value = 'idle'
    runSegmentIds.value = []
    clearResult()
  }
}

/** Segment whose Proofreader strip Esc hid; it shows again once the caret reaches another segment. */
const hiddenForSegmentId = ref<number | null>(null)

export const proofreadStripHidden: ComputedRef<boolean> = computed(
  () => hiddenForSegmentId.value !== null && hiddenForSegmentId.value === editorCaretSegmentId.value,
)

/** Focus returns to the caret's translation cell first, so leaving the strip keeps the caret. */
export function hideProofreadStrip(): void {
  if (state.value === 'scanning' || state.value === 'error' || state.value === 'not_configured') return
  const segmentId = editorCaretSegmentId.value
  if (segmentId === null) return
  const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${String(segmentId)}"]`)
  if (cell instanceof HTMLElement && document.activeElement?.closest('.proofreader-strip')) cell.focus()
  hiddenForSegmentId.value = segmentId
}

export function syncProofreadStripHidden(caretSegmentId: number | null): void {
  if (caretSegmentId !== null && hiddenForSegmentId.value !== caretSegmentId) hiddenForSegmentId.value = null
}

export function openProofreadScope(): void {
  scopeOpen.value = true
}

export function closeProofreadScope(): void {
  scopeOpen.value = false
}

export function resetProofread(): void {
  if (state.value === 'scanning' && flushingRun !== sequence) void cancelProofreadCall()
  sequence += 1
  flushingRun = 0
  state.value = 'idle'
  runSegmentIds.value = []
  scopeOpen.value = false
  hiddenForSegmentId.value = null
  reverts.value = new Map()
  focus.value = { segmentId: -1, index: 0 }
  acting.value = false
  clearResult()
  resetProofreadMemory()
  resetProofreadDecisions()
}

function dropReverts(segmentId: number): void {
  if (!reverts.value.has(segmentId)) return
  const next = new Map(reverts.value)
  next.delete(segmentId)
  reverts.value = next
}

function editorTextOf(segmentId: number): string | null {
  return editorSegments.value.find((segment) => segment.id === segmentId)?.target_text ?? null
}

function flushFailure(flushed: 'failed' | 'still-dirty'): IpcError {
  const key = flushed === 'failed' ? 'proofread.accept_flush_failed' : 'proofread.accept_flush_still_dirty'
  return { code: key, message_key: key, params: {}, retryable: true }
}

/** `[s, e)` is the replaced span of the old text; `null` means the span overlaps it and must be dropped. */
function shiftSpan(span: Span, replaced: Span, delta: number): Span | null {
  if (span.end <= replaced.start) return span
  if (span.start >= replaced.end) return { start: span.start + delta, end: span.end + delta }
  return null
}

export type ProofreadCurrent = {
  readonly segmentId: number
  readonly index: number
  readonly count: number
  readonly finding: ProofreadFindingWire
  readonly scannedText: string
}

/** The finding at the caret, only while the segment still holds exactly the scanned text. */
export const proofreadCurrent: ComputedRef<ProofreadCurrent | null> = computed(() => {
  const caret = editorCaretSegmentId.value
  if (caret === null) return null
  const entry = results.value.get(caret)
  if (entry === undefined || entry.findings.length === 0) return null
  if (editorTextOf(caret) !== entry.scannedText) return null
  const wanted = focus.value.segmentId === caret ? focus.value.index : 0
  const index = Math.min(Math.max(wanted, 0), entry.findings.length - 1)
  return {
    segmentId: caret,
    index,
    count: entry.findings.length,
    finding: entry.findings[index] as ProofreadFindingWire,
    scannedText: entry.scannedText,
  }
})

function topRevert(segmentId: number): RevertItem | null {
  const stack = reverts.value.get(segmentId)
  const latest = stack?.[stack.length - 1]
  if (latest === undefined) return null
  return editorTextOf(segmentId) === latest.text ? latest : null
}

/** An accepted suggestion at the caret that can still be undone. */
export const proofreadRevertible: ComputedRef<boolean> = computed(() => {
  const caret = editorCaretSegmentId.value
  return caret !== null && topRevert(caret) !== null
})

export const proofreadActionError: ComputedRef<IpcError | null> = computed(() => {
  const failure = actionFailure.value
  return failure !== null && failure.segmentId === editorCaretSegmentId.value ? failure.error : null
})

export const proofreadBusy: DeepReadonly<Ref<boolean>> = readonly(acting)

function setEntry(segmentId: number, entry: ProofreadSegmentResult): void {
  const next = new Map(results.value)
  next.set(segmentId, entry)
  results.value = next
}

/** Moves the remaining findings and revert items of a segment after `[replaced)` of the old text was swapped. */
export function shiftProofreadFindings(
  segmentId: number,
  replaced: Span,
  delta: number,
  newText: string,
  skip: ProofreadFindingWire | null,
): void {
  const entry = results.value.get(segmentId)
  if (entry !== undefined) {
    const findings: ProofreadFindingWire[] = []
    for (const finding of entry.findings) {
      if (finding === skip) continue
      const moved = shiftSpan(finding, replaced, delta)
      if (moved !== null) findings.push({ ...finding, start: moved.start, end: moved.end })
    }
    setEntry(segmentId, { ...entry, scannedText: newText, findings })
  }
  const stack = reverts.value.get(segmentId)
  if (stack !== undefined) {
    const kept: RevertItem[] = []
    for (const item of stack) {
      const moved = shiftSpan(item, replaced, delta)
      if (moved !== null) kept.push({ ...item, start: moved.start, end: moved.end, text: newText })
    }
    const next = new Map(reverts.value)
    if (kept.length === 0) next.delete(segmentId)
    else next.set(segmentId, kept)
    reverts.value = next
  }
}

function mirror(outcome: NonNullable<ProofreadWriteResult['outcome']>): void {
  replaceEditorSegment(outcome.segment_id, {
    target_text: outcome.target_text,
    translation_origin: outcome.translation_origin,
    status: outcome.status,
  })
}

export async function acceptProofreadFinding(): Promise<void> {
  if (acting.value) return
  const current = proofreadCurrent.value
  if (current === null) return
  const mine = sequence
  const { segmentId, finding, scannedText } = current
  acting.value = true
  actionFailure.value = null
  try {
    const flushed = await flushEditorBeforeDiscreteWrite()
    if (flushed !== 'clean') {
      actionFailure.value = { segmentId, error: flushFailure(flushed) }
      return
    }
    if (mine !== sequence || !(results.value.get(segmentId)?.findings.includes(finding) ?? false)) return
    const result = await acceptProofreadSuggestion({
      segmentId,
      expectedText: scannedText,
      start: finding.start,
      end: finding.end,
      expectedPhrase: scannedText.slice(finding.start, finding.end),
      replacement: finding.suggestion,
      kind: finding.kind,
    })
    if (result.outcome === null) {
      if (result.error !== null) actionFailure.value = { segmentId, error: result.error }
      return
    }
    mirror(result.outcome)
    if (mine !== sequence) return
    const replaced = { start: finding.start, end: finding.end }
    const delta = finding.suggestion.length - (finding.end - finding.start)
    shiftProofreadFindings(segmentId, replaced, delta, result.outcome.target_text, finding)
    const stack = reverts.value.get(segmentId) ?? []
    const next = new Map(reverts.value)
    next.set(segmentId, [
      ...stack,
      {
        decisionId: result.decisionId as number,
        finding,
        phrase: scannedText.slice(finding.start, finding.end),
        before: scannedText,
        start: finding.start,
        end: finding.start + finding.suggestion.length,
        text: result.outcome.target_text,
      },
    ])
    reverts.value = next
    void reloadProofreadDecisions()
  } finally {
    acting.value = false
  }
}

export async function revertProofreadFinding(): Promise<void> {
  if (acting.value) return
  const caret = editorCaretSegmentId.value
  if (caret === null) return
  const latest = topRevert(caret)
  if (latest === null) return
  const mine = sequence
  acting.value = true
  actionFailure.value = null
  try {
    const flushed = await flushEditorBeforeDiscreteWrite()
    if (flushed !== 'clean') {
      actionFailure.value = { segmentId: caret, error: flushFailure(flushed) }
      return
    }
    const live = topRevert(caret)
    if (live === null) return
    const result = await revertProofreadSuggestion({
      segmentId: caret,
      expectedText: live.text,
      start: 0,
      end: live.text.length,
      expectedPhrase: live.text,
      replacement: live.before,
      kind: live.finding.kind,
      decisionId: live.decisionId,
    })
    if (result.outcome === null) {
      if (result.error !== null) actionFailure.value = { segmentId: caret, error: result.error }
      return
    }
    mirror(result.outcome)
    const stack = reverts.value.get(caret) ?? []
    const rest = stack.filter((item) => item !== live)
    const nextReverts = new Map(reverts.value)
    if (rest.length === 0) nextReverts.delete(caret)
    else nextReverts.set(caret, rest)
    reverts.value = nextReverts
    void reloadProofreadDecisions()
    if (mine !== sequence) return
    const delta = live.phrase.length - (live.end - live.start)
    shiftProofreadFindings(caret, { start: live.start, end: live.end }, delta, result.outcome.target_text, null)
    const entry = results.value.get(caret)
    if (entry !== undefined) {
      const restored: ProofreadFindingWire = { ...live.finding, start: live.start, end: live.start + live.phrase.length }
      setEntry(caret, {
        ...entry,
        findings: [...entry.findings, restored].sort((a, b) => a.start - b.start),
      })
    }
  } finally {
    acting.value = false
  }
}

function removeFinding(current: ProofreadCurrent): void {
  const entry = results.value.get(current.segmentId)
  if (entry === undefined) return
  actionFailure.value = null
  focus.value = { segmentId: current.segmentId, index: current.index }
  setEntry(current.segmentId, { ...entry, findings: entry.findings.filter((f) => f !== current.finding) })
}

/** "Bỏ qua": Rust records one `skipped` decision (no segment, no memory), then the finding leaves the strip. */
export async function dismissProofreadFinding(): Promise<void> {
  if (acting.value) return
  const current = proofreadCurrent.value
  if (current === null) return
  const mine = sequence
  const { segmentId, finding } = current
  acting.value = true
  actionFailure.value = null
  try {
    const result = await skipProofreadDecision(finding.kind)
    if (!result.ok) {
      if (result.error !== null) actionFailure.value = { segmentId, error: result.error }
      return
    }
    void reloadProofreadDecisions()
    if (mine !== sequence) return
    const live = proofreadCurrent.value
    if (live !== null && live.finding === finding) removeFinding(live)
    else {
      const entry = results.value.get(segmentId)
      if (entry !== undefined) setEntry(segmentId, { ...entry, findings: entry.findings.filter((f) => f !== finding) })
    }
  } finally {
    acting.value = false
  }
}

/** "Not a mistake": Rust remembers the signature for the open Work, then the finding leaves the strip. */
export async function ignoreProofreadFinding(): Promise<void> {
  if (acting.value) return
  const current = proofreadCurrent.value
  if (current === null) return
  const mine = sequence
  const { segmentId, finding, scannedText } = current
  acting.value = true
  actionFailure.value = null
  try {
    const flushed = await flushEditorBeforeDiscreteWrite()
    if (flushed !== 'clean') {
      actionFailure.value = { segmentId, error: flushFailure(flushed) }
      return
    }
    if (mine !== sequence || !(results.value.get(segmentId)?.findings.includes(finding) ?? false)) return
    const result = await addProofreadIgnore({
      segmentId,
      expectedText: scannedText,
      start: finding.start,
      end: finding.end,
      expectedPhrase: scannedText.slice(finding.start, finding.end),
      kind: finding.kind,
    })
    if (!result.ok) {
      if (result.error !== null) actionFailure.value = { segmentId, error: result.error }
      return
    }
    void reloadProofreadDecisions()
    if (mine !== sequence) return
    const live = proofreadCurrent.value
    if (live !== null && live.finding === finding) removeFinding(live)
    else {
      const entry = results.value.get(segmentId)
      if (entry !== undefined) setEntry(segmentId, { ...entry, findings: entry.findings.filter((f) => f !== finding) })
    }
  } finally {
    acting.value = false
  }
}

type Position = { segmentId: number; index: number }

function findingPositions(): Position[] {
  const list: Position[] = []
  for (const segment of editorSegments.value) {
    const entry = results.value.get(segment.id)
    if (entry === undefined || entry.scannedText !== segment.target_text) continue
    entry.findings.forEach((_, index) => list.push({ segmentId: segment.id, index }))
  }
  return list
}

function step(direction: 1 | -1): void {
  const positions = findingPositions()
  const current = proofreadCurrent.value
  let target: Position | undefined
  if (current !== null) {
    const at = positions.findIndex((p) => p.segmentId === current.segmentId && p.index === current.index)
    target = at + direction < 0 ? undefined : positions[at + direction]
  } else {
    const caret = editorCaretSegmentId.value
    const order = editorSegments.value.map((segment) => segment.id)
    const caretAt = caret === null ? -1 : order.indexOf(caret)
    const after = positions.filter((p) => {
      const at = order.indexOf(p.segmentId)
      return direction === 1 ? at > caretAt : at < caretAt
    })
    target = direction === 1 ? after.at(0) : after.at(-1)
  }
  if (target === undefined) return
  actionFailure.value = null
  if (target.segmentId !== editorCaretSegmentId.value && !moveEditorCaretTo(target.segmentId)) return
  focus.value = target
}

export function nextProofreadFinding(): void {
  step(1)
}

export function prevProofreadFinding(): void {
  step(-1)
}
