import { computed, readonly, ref, shallowRef } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import { resendAiCall, runAiTranslateBatchCall } from './config/aitranslate'
import { cancelWhenGenerating } from './aiTranslateCancel'
import type {
  AiTranslateBatchEventWire,
  AiTranslateDiscardReason,
  AiTranslateOutcomeWire,
  AiTranslateUsageWire,
} from './config/aitranslate'
import type { IpcError } from './i18n'
import { nextAiTranslateRunStamp } from './aiTranslateRunClock'

export type AiTranslateBatchRowStatus = 'queued' | 'streaming' | 'received' | 'missing' | 'not_sent'

export type AiTranslateBatchRow = {
  segmentId: number
  callIndex: number
  status: AiTranslateBatchRowStatus
  reason: AiTranslateDiscardReason | null
  text: string
  hasHan: boolean
  sourceHash: string | null
}

export type AiTranslateBatchState = 'not_configured' | 'generating' | 'done' | 'error' | 'cancelled'

const state = ref<AiTranslateBatchState>('not_configured')
const rows = ref<AiTranslateBatchRow[]>([])
const error = shallowRef<IpcError | null>(null)
const startStamp = ref(0)
const runningCallIndex = ref<number | null>(null)
const callUsages = ref<Record<string, AiTranslateUsageWire | null>>({})

let sequence = 0
let rowIndexBySegmentId = new Map<number, number>()

export const aiTranslateBatchStateValue: DeepReadonly<Ref<AiTranslateBatchState>> = readonly(state)
export const aiTranslateBatchRows: DeepReadonly<Ref<AiTranslateBatchRow[]>> = readonly(rows)
export const aiTranslateBatchStartStamp: DeepReadonly<Ref<number>> = readonly(startStamp)
export const aiTranslateBatchError: DeepReadonly<Ref<IpcError | null>> = readonly(error)
export const aiTranslateBatchRunningCall: DeepReadonly<Ref<number | null>> = readonly(runningCallIndex)

function countOf(status: AiTranslateBatchRowStatus): number {
  return rows.value.filter((r) => r.status === status).length
}

export const aiTranslateBatchTotalCount: ComputedRef<number> = computed(() => rows.value.length)
export const aiTranslateBatchDoneCount: ComputedRef<number> = computed(() => countOf('received'))
export const aiTranslateBatchRemainingCount: ComputedRef<number> = computed(
  () => countOf('queued') + countOf('streaming'),
)
export const aiTranslateBatchMissingCount: ComputedRef<number> = computed(() => countOf('missing'))
export const aiTranslateBatchNotSentCount: ComputedRef<number> = computed(() => countOf('not_sent'))

export const aiTranslateBatchUsageSummary: ComputedRef<{
  tokenCount: number
  reportedCount: number
  sentenceCount: number
  costUsd: number | null
}> = computed(() => {
  const calls = Object.values(callUsages.value)
  const reporting = calls.filter((u): u is AiTranslateUsageWire => u !== null)
  const allReportingHaveCost = reporting.length > 0 && reporting.every((u) => u.cost_usd !== null)
  return {
    tokenCount: reporting.reduce((sum, u) => sum + u.total_tokens, 0),
    reportedCount: reporting.length,
    sentenceCount: calls.length,
    costUsd: allReportingHaveCost ? reporting.reduce((sum, u) => sum + (u.cost_usd ?? 0), 0) : null,
  }
})

export function aiTranslateBatchResultForSegment(
  currentRows: readonly AiTranslateBatchRow[],
  segmentId: number,
): { text: string; sourceHash: string } | null {
  const row = currentRows.find((r) => r.segmentId === segmentId)
  if (row === undefined || row.status !== 'received' || row.sourceHash === null) return null
  return { text: row.text, sourceHash: row.sourceHash }
}

export function aiTranslateBatchTextForSegment(
  currentRows: readonly AiTranslateBatchRow[],
  segmentId: number,
): string | null {
  return aiTranslateBatchResultForSegment(currentRows, segmentId)?.text ?? null
}

export function aiTranslateBatchPromotableItems(
  currentRows: readonly AiTranslateBatchRow[],
): { segmentId: number; text: string; sourceHash: string }[] {
  const items: { segmentId: number; text: string; sourceHash: string }[] = []
  for (const r of currentRows) {
    if (r.status === 'received' && r.text !== '' && r.sourceHash !== null) {
      items.push({ segmentId: r.segmentId, text: r.text, sourceHash: r.sourceHash })
    }
  }
  return items
}

export const aiTranslateBatchPromotableCount: ComputedRef<number> = computed(
  () => aiTranslateBatchPromotableItems(rows.value).length,
)

export function aiTranslateBatchRetryIds(currentRows: readonly AiTranslateBatchRow[]): number[] {
  return currentRows.filter((r) => r.status === 'missing' || r.status === 'not_sent').map((r) => r.segmentId)
}

export function aiTranslateBatchMayRetry(
  batchState: AiTranslateBatchState,
  err: IpcError | null,
  retryIds: readonly number[],
): boolean {
  if (retryIds.length === 0) return false
  if (batchState === 'done' || batchState === 'cancelled') return true
  return batchState === 'error' && err !== null && err.retryable
}

export type AiTranslateBatchErrorSpan = { callIndex: number; first: number; last: number }

export function aiTranslateBatchErrorSpan(
  currentRows: readonly AiTranslateBatchRow[],
  err: IpcError | null,
): AiTranslateBatchErrorSpan | null {
  if (err === null) return null
  const callIndex = Number(err.params.call_index)
  const firstId = Number(err.params.first_segment_id)
  const lastId = Number(err.params.last_segment_id)
  if (!Number.isFinite(callIndex) || !Number.isFinite(firstId) || !Number.isFinite(lastId)) return null
  const first = currentRows.findIndex((r) => r.segmentId === firstId)
  const last = currentRows.findIndex((r) => r.segmentId === lastId)
  if (first === -1 || last === -1) return null
  return { callIndex, first: first + 1, last: last + 1 }
}

function setRowAt(index: number, patch: Partial<AiTranslateBatchRow>): void {
  if (index < 0 || index >= rows.value.length) return
  rows.value[index] = { ...rows.value[index], ...patch }
}

function queuedRow(segmentId: number, callIndex: number): AiTranslateBatchRow {
  return { segmentId, callIndex, status: 'queued', reason: null, text: '', hasHan: false, sourceHash: null }
}

function applyPlanned(calls: readonly { call_index: number; segment_ids: readonly number[] }[]): void {
  for (const call of calls) {
    for (const id of call.segment_ids) {
      const index = rowIndexBySegmentId.get(id)
      if (index === undefined) {
        rowIndexBySegmentId.set(id, rows.value.length)
        rows.value.push(queuedRow(id, call.call_index))
      } else {
        rows.value[index] = queuedRow(id, call.call_index)
      }
    }
  }
}

function settleUnresolvedRows(): void {
  rows.value = rows.value.map((r) => {
    if (r.status === 'queued') return { ...r, status: 'not_sent' }
    if (r.status === 'streaming') return { ...r, status: 'missing', reason: 'interrupted', text: '' }
    return r
  })
}

type OperationRunner = (
  onEvent: (event: AiTranslateBatchEventWire) => void,
) => Promise<{ value: AiTranslateOutcomeWire | null; error: IpcError | null }>

async function streamOperation(runner: OperationRunner): Promise<void> {
  const mine = ++sequence
  startStamp.value = nextAiTranslateRunStamp()
  state.value = 'generating'
  error.value = null
  runningCallIndex.value = null

  const onEvent = (event: AiTranslateBatchEventWire): void => {
    if (mine !== sequence) return
    if (event.kind === 'planned') {
      applyPlanned(event.calls)
      return
    }
    if (event.kind === 'call_started') {
      runningCallIndex.value = event.call_index
      return
    }
    if (event.kind === 'call_done') {
      const key = `${mine}:${event.call_index}`
      callUsages.value[key] = event.usage
      if (runningCallIndex.value === event.call_index) runningCallIndex.value = null
      return
    }
    const index = rowIndexBySegmentId.get(event.segment_id)
    if (index === undefined) return
    const row = rows.value[index]
    if (event.kind === 'token') {
      if (row.status !== 'queued' && row.status !== 'streaming') return
      setRowAt(index, { status: 'streaming', text: row.text + event.text })
      return
    }
    if (event.kind === 'received') {
      setRowAt(index, {
        status: 'received',
        reason: null,
        text: event.text,
        hasHan: event.has_han,
        sourceHash: event.source_hash,
      })
      return
    }
    setRowAt(index, { status: 'missing', reason: event.reason, text: '', hasHan: false, sourceHash: null })
  }

  const result = await runner(onEvent)
  if (mine !== sequence) return
  runningCallIndex.value = null
  settleUnresolvedRows()

  if (result.error !== null) {
    state.value = 'error'
    error.value = result.error
    return
  }
  state.value = result.value === null ? 'not_configured' : result.value.state
}

export async function runAiTranslateBatch(
  promptSetName: string | null,
  segmentIds: readonly number[] | null,
): Promise<void> {
  if (state.value === 'generating') {
    console.warn('[ai-translate-batch] khong dich: mot lo khac dang chay')
    return
  }
  if (segmentIds !== null && segmentIds.length === 0) {
    console.warn('[ai-translate-batch] khong dich: vung chon rong')
    return
  }
  rows.value = []
  rowIndexBySegmentId = new Map()
  callUsages.value = {}
  const scope =
    segmentIds === null
      ? ({ kind: 'chapter' } as const)
      : ({ kind: 'selection', segment_ids: segmentIds.slice() } as const)
  await streamOperation((onEvent) => runAiTranslateBatchCall(scope, promptSetName, onEvent))
}

export async function retryAiTranslateBatch(
  promptSetName: string | null,
  segmentIds: readonly number[],
): Promise<void> {
  if (state.value === 'generating') {
    console.warn('[ai-translate-batch] khong retry: mot lo khac dang chay')
    return
  }
  const ids = segmentIds.filter((id) => rowIndexBySegmentId.has(id))
  if (ids.length === 0) {
    console.warn('[ai-translate-batch] khong retry: khong co hang nao de chay lai')
    return
  }
  await streamOperation((onEvent) =>
    runAiTranslateBatchCall({ kind: 'selection', segment_ids: ids }, promptSetName, onEvent),
  )
}

export async function resendAiTranslateCall(operationId: number, callIndex: number): Promise<void> {
  if (state.value === 'generating') {
    console.warn('[ai-translate-batch] khong gui lai luot: mot lo dang chay')
    return
  }
  await streamOperation((onEvent) => resendAiCall(operationId, callIndex, onEvent))
}

export function cancelAiTranslateBatch(): void {
  cancelWhenGenerating(state)
}

export function resetAiTranslateBatch(): void {
  cancelWhenGenerating(state)

  sequence += 1
  state.value = 'not_configured'
  rows.value = []
  error.value = null
  startStamp.value = 0
  runningCallIndex.value = null
  callUsages.value = {}
  rowIndexBySegmentId = new Map()
}
