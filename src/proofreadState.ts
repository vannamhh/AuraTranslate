import { computed, readonly, ref, shallowRef } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import { cancelProofreadCall, runProofreadSegments } from './config/proofread'
import type { ProofreadEventWire, ProofreadFindingWire, ProofreadSegmentResultWire } from './config/proofread'
import type { AiUsageSummary } from './aiUsageLine'
import type { IpcError } from './i18n'
import { flushEditorBeforeDiscreteWrite } from './panels/editorPanelState'
import { resetProofreadScopeCaret } from './proofreadScope'

export type ProofreadState = 'idle' | 'scanning' | 'done' | 'error' | 'cancelled' | 'not_configured'

export type ProofreadSegmentResult = {
  readonly scannedText: string
  readonly findings: readonly ProofreadFindingWire[]
  readonly unlocated: number
}

const state = ref<ProofreadState>('idle')
const runSegmentIds = shallowRef<readonly number[]>([])
const results = shallowRef<ReadonlyMap<number, ProofreadSegmentResult>>(new Map())
const planned = ref(0)
const arrived = ref(0)
const notScanned = ref(0)
const tokenCount = ref(0)
const reportedCount = ref(0)
const costUsdSum = ref(0)
const costMissing = ref(false)
const error = shallowRef<IpcError | null>(null)
const scopeOpen = ref(false)

let sequence = 0
let flushingRun = 0
let editedWhileScanning = new Set<number>()

export const proofreadStateValue: DeepReadonly<Ref<ProofreadState>> = readonly(state)
export const proofreadRunSegmentIds: Readonly<Ref<readonly number[]>> = runSegmentIds
export const proofreadResults: Readonly<Ref<ReadonlyMap<number, ProofreadSegmentResult>>> = results
export const proofreadError: DeepReadonly<Ref<IpcError | null>> = readonly(error)
export const proofreadNotScanned: DeepReadonly<Ref<number>> = readonly(notScanned)
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
  tokenCount.value = 0
  reportedCount.value = 0
  costUsdSum.value = 0
  costMissing.value = false
  error.value = null
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
    if (editedWhileScanning.has(segment.segment_id)) continue
    next.set(segment.segment_id, {
      scannedText: segment.scanned_text,
      findings: segment.findings,
      unlocated: segment.unlocated,
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
export async function runProofread(segmentIds: readonly number[]): Promise<void> {
  if (state.value === 'scanning') {
    console.warn('[proofread] not scanning: a scan is already running')
    return
  }
  if (segmentIds.length === 0) {
    console.warn('[proofread] not scanning: the scope holds no segment')
    return
  }

  const mine = ++sequence
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
  state.value = result.value.state
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
  resetProofreadScopeCaret()
  clearResult()
}
