/**
 * Review Mode: a read-only layout state inside Workspace (AD-24), not a fourth mode. The data is
 * whatever `alignment_open` returned; nothing here writes. The layout it shows is never persisted.
 * Do NOT import this file from `src/commands/index.ts`; the handlers are injected through
 * `CommandDeps` (`reviewModeCommandDeps.ts`).
 */
import { computed, nextTick, readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { enterFocus } from './commands'
import {
  alignmentOpen,
  reviewAcceptChange as acceptChangeIpc,
  reviewDiff,
  reviewSkipChange as skipChangeIpc,
} from './config/alignment'
import type {
  AlignmentRow,
  AlignmentRowKind,
  AlignmentSegment,
  ReviewDecision,
  ReviewDiffPair,
  ReviewDiffSpan,
} from './config/alignment'
import type { IpcError } from './i18n'
import { activeDockPanelId, resetDockSuspended, setDockSuspended } from './layout/dockController'
import { currentMode } from './modes/modeState'
import { flushEditorBeforeDiscreteWrite, replaceEditorSegment } from './panels/editorPanelState'
import { resetReviewScrollers, scrollReviewPairIntoView } from './reviewScrollSync'

export type ReviewModeStatus = 'idle' | 'loading' | 'open' | 'not_imported' | 'stale' | 'no_chapter' | 'error' | 'diff_failed'

const FALLBACK_OWNER = 'panel.grid'
const MINE_OWNER = 'panel.review_mine'

export type ReviewPairView = {
  key: string
  groupId: number
  changed: boolean
  /** A difference or a decision: the pairs the jumps walk and the status line counts. */
  isChange: boolean
  /** A change with a reviewer row and no decision yet. */
  pending: boolean
  /** One segment, one reviewer row, and a difference to take over. Rust has the last word. */
  acceptable: boolean
  decision: ReviewDecision | null
  hasSegment: boolean
  hasRow: boolean
  rowKind: AlignmentRowKind | null
  mine: ReviewDiffSpan[]
  copy: ReviewDiffSpan[]
}

export type ReviewDiffMessage = { key: string; params: Record<string, string> }

/** An accept held back because it would overwrite a draft that has no copy; nothing was written. */
export type PendingAccept = { groupId: number; expectedTarget: string; draft: string }

const status = ref<ReviewModeStatus>('idle')
const segments = ref<AlignmentSegment[]>([])
const rows = ref<AlignmentRow[]>([])
const pairs = ref<ReviewDiffPair[]>([])
const unmatchedSegmentIds = ref<number[]>([])
const unmatchedRowIds = ref<number[]>([])
const diffCursor = ref(-1)
const busy = ref(false)
const pendingAccept = shallowRef<PendingAccept | null>(null)
let openChapterId: number | null = null
const diffMessage = ref<ReviewDiffMessage | null>(null)
const loadError = ref<IpcError | null>(null)
let previousOwner: string | null = null
let sequence = 0

export const reviewModeStatus: DeepReadonly<Ref<ReviewModeStatus>> = readonly(status)
export const reviewModeIsOpen = computed(() => status.value === 'open')
export const reviewModeLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)

export const reviewModeSegments = computed<AlignmentSegment[]>(() => [...segments.value].sort((a, b) => a.ord - b.ord))
export const reviewModeRows = computed<AlignmentRow[]>(() => rows.value)

export const reviewModePairs = computed<ReviewPairView[]>(() =>
  pairs.value.map((p) => {
    const firstRow = p.row_ids.length > 0 ? rows.value.find((r) => r.id === p.row_ids[0]) : undefined
    const changed = p.spans.some((span) => span.kind !== 'equal')
    return {
      key: `g${p.group_id}`,
      groupId: p.group_id,
      changed,
      isChange: changed || p.decision !== null,
      pending: (changed || p.decision !== null) && p.row_ids.length > 0 && p.decision === null,
      acceptable: changed && p.segment_ids.length === 1 && p.row_ids.length === 1,
      decision: p.decision,
      hasSegment: p.segment_ids.length > 0,
      hasRow: p.row_ids.length > 0,
      rowKind: firstRow?.kind ?? null,
      mine: p.spans.filter((span) => span.kind !== 'insert'),
      copy: p.spans.filter((span) => span.kind !== 'delete'),
    }
  }),
)
export type ReviewMineItem =
  | { kind: 'pair'; pair: ReviewPairView }
  | { kind: 'unmatched'; segment: AlignmentSegment }
export type ReviewCopyItem =
  | { kind: 'pair'; pair: ReviewPairView }
  | { kind: 'unmatched'; row: AlignmentRow }

function interleave<U>(
  views: ReviewPairView[],
  pairPositions: number[][],
  unmatched: { value: U; position: number }[],
): ({ kind: 'pair'; pair: ReviewPairView } | { kind: 'unmatched'; value: U })[] {
  const buckets = new Map<number, U[]>()
  for (const u of [...unmatched].sort((a, b) => a.position - b.position)) {
    let anchor = -1
    pairPositions.forEach((positions, i) => {
      if (positions.some((x) => x < u.position)) anchor = i
    })
    buckets.set(anchor, [...(buckets.get(anchor) ?? []), u.value])
  }
  const out: ({ kind: 'pair'; pair: ReviewPairView } | { kind: 'unmatched'; value: U })[] = []
  for (const value of buckets.get(-1) ?? []) out.push({ kind: 'unmatched', value })
  views.forEach((pair, i) => {
    out.push({ kind: 'pair', pair })
    for (const value of buckets.get(i) ?? []) out.push({ kind: 'unmatched', value })
  })
  return out
}

export const reviewModeMineItems = computed<ReviewMineItem[]>(() => {
  const ordOf = new Map(segments.value.map((s) => [s.id, s.ord]))
  const unmatched = reviewModeSegments.value
    .filter((s) => unmatchedSegmentIds.value.includes(s.id))
    .map((s) => ({ value: s, position: s.ord }))
  return interleave(
    reviewModePairs.value,
    pairs.value.map((p) => p.segment_ids.map((id) => ordOf.get(id)).filter((x): x is number => x !== undefined)),
    unmatched,
  ).map((i) => (i.kind === 'pair' ? i : { kind: 'unmatched', segment: i.value }))
})

export const reviewModeCopyItems = computed<ReviewCopyItem[]>(() => {
  const indexOf = new Map(rows.value.map((r, i) => [r.id, i]))
  const unmatched = rows.value
    .filter((r) => unmatchedRowIds.value.includes(r.id))
    .map((r) => ({ value: r, position: indexOf.get(r.id) ?? 0 }))
  return interleave(
    reviewModePairs.value,
    pairs.value.map((p) => p.row_ids.map((id) => indexOf.get(id)).filter((x): x is number => x !== undefined)),
    unmatched,
  ).map((i) => (i.kind === 'pair' ? i : { kind: 'unmatched', row: i.value }))
})
export const reviewModeCurrentPairKey = computed<string | null>(() => {
  const changes = reviewModePairs.value.filter((p) => p.isChange)
  return changes[diffCursor.value]?.key ?? null
})
export const reviewModeCurrentPair = computed<ReviewPairView | null>(
  () => reviewModePairs.value.filter((p) => p.isChange)[diffCursor.value] ?? null,
)
export const reviewModeChangeStats = computed(() => {
  const changes = reviewModePairs.value.filter((p) => p.isChange && p.hasRow)
  return { count: changes.length, handled: changes.filter((p) => p.decision !== null).length }
})
export const reviewModePendingAccept: DeepReadonly<Ref<PendingAccept | null>> = readonly(pendingAccept)
export const reviewModeBusy: DeepReadonly<Ref<boolean>> = readonly(busy)
export const reviewModeDiffMessage: DeepReadonly<Ref<ReviewDiffMessage | null>> = readonly(diffMessage)

function clearData(): void {
  segments.value = []
  rows.value = []
  pairs.value = []
  unmatchedSegmentIds.value = []
  unmatchedRowIds.value = []
  diffCursor.value = -1
  diffMessage.value = null
  loadError.value = null
  pendingAccept.value = null
  busy.value = false
  openChapterId = null
}

export async function openReviewMode(chapterId: number | null): Promise<void> {
  if (status.value === 'open' || status.value === 'loading') return

  sequence += 1
  const mySequence = sequence
  clearData()

  if (chapterId === null) {
    status.value = 'no_chapter'
    return
  }

  status.value = 'loading'

  const result = await alignmentOpen(chapterId)
  if (mySequence !== sequence) return

  if (result.alignment !== null) {
    const diff = await reviewDiff(chapterId)
    if (mySequence !== sequence) return
    if (diff.diff === null) {
      loadError.value = diff.error
      status.value = 'diff_failed'
      return
    }
    segments.value = result.alignment.segments
    rows.value = result.alignment.rows
    unmatchedSegmentIds.value = result.alignment.unmatched_segment_ids
    unmatchedRowIds.value = result.alignment.unmatched_row_ids
    pairs.value = diff.diff.pairs
    openChapterId = chapterId
    previousOwner = activeDockPanelId()
    status.value = 'open'
    setDockSuspended(true)
    return
  }
  if (result.error?.code === 'export.alignment_not_imported') {
    status.value = 'not_imported'
    return
  }
  if (result.error?.code === 'export.alignment_stale') {
    status.value = 'stale'
    return
  }
  loadError.value = result.error
  status.value = 'error'
}

export async function closeReviewMode(): Promise<void> {
  const wasOpen = status.value === 'open'
  sequence += 1
  status.value = 'idle'
  clearData()
  resetReviewScrollers()
  setDockSuspended(false)
  const target = previousOwner ?? FALLBACK_OWNER
  previousOwner = null
  if (currentMode.value !== 'workspace') return

  await nextTick()
  if (!wasOpen) {
    const active = document.activeElement
    if (active === null || active === document.body) enterFocus('mode.workspace')
    return
  }
  if (!enterFocus(target)) enterFocus('mode.workspace')
}

function changedPairKeys(): string[] {
  return reviewModePairs.value.filter((p) => p.isChange).map((p) => p.key)
}

async function showCursor(): Promise<void> {
  const keys = changedPairKeys()
  diffMessage.value = {
    key: 'review.diff_position',
    params: { index: String(diffCursor.value + 1), count: String(keys.length) },
  }
  await nextTick()
  const key = keys.at(diffCursor.value)
  if (key !== undefined) scrollReviewPairIntoView(key)
  const active = document.activeElement
  if (active === null || active === document.body) enterFocus(MINE_OWNER)
}

function jumpDiff(step: 1 | -1): void {
  if (status.value !== 'open') return
  const count = changedPairKeys().length
  if (count === 0) {
    diffMessage.value = { key: 'review.diff_none', params: {} }
    return
  }
  const cursor = diffCursor.value
  if (step === 1 && cursor >= count - 1) {
    diffMessage.value = { key: 'review.diff_last', params: { count: String(count) } }
    return
  }
  if (step === -1 && cursor === 0) {
    diffMessage.value = { key: 'review.diff_first', params: { count: String(count) } }
    return
  }
  pendingAccept.value = null
  diffCursor.value = step === 1 ? cursor + 1 : Math.max(cursor - 1, 0)
  void showCursor()
}

export function reviewDiffNext(): void {
  jumpDiff(1)
}

export function reviewDiffPrev(): void {
  jumpDiff(-1)
}

export function resetReviewMode(): void {
  resetReviewScrollers()
  sequence += 1
  status.value = 'idle'
  clearData()
  previousOwner = null
  resetDockSuspended()
}

function say(key: string): void {
  diffMessage.value = { key, params: {} }
}

function sayError(error: IpcError): void {
  diffMessage.value = { key: error.message_key, params: error.params }
}

/** Reads both sides again after a write or a refusal; the cursor keeps its place among the changes. */
async function reload(chapterId: number, mySequence: number): Promise<boolean> {
  const aligned = await alignmentOpen(chapterId)
  if (mySequence !== sequence) return false
  const diff = aligned.alignment === null ? null : await reviewDiff(chapterId)
  if (mySequence !== sequence) return false
  if (aligned.alignment === null || diff === null || diff.diff === null) {
    loadError.value = aligned.error ?? diff?.error ?? null
    status.value = aligned.alignment === null ? 'error' : 'diff_failed'
    return false
  }
  segments.value = aligned.alignment.segments
  rows.value = aligned.alignment.rows
  unmatchedSegmentIds.value = aligned.alignment.unmatched_segment_ids
  unmatchedRowIds.value = aligned.alignment.unmatched_row_ids
  pairs.value = diff.diff.pairs
  const count = changedPairKeys().length
  diffCursor.value = Math.min(diffCursor.value, count - 1)
  return true
}

async function advanceToPending(): Promise<void> {
  const changes = reviewModePairs.value.filter((p) => p.isChange)
  const from = diffCursor.value
  const next = changes
    .map((_, i) => (from + 1 + i) % changes.length)
    .find((i) => changes[i].pending)
  if (next === undefined) {
    say('review.all_processed')
    return
  }
  diffCursor.value = next
  await showCursor()
}

async function runAccept(chapterId: number, groupId: number, expectedTarget: string, force: boolean): Promise<void> {
  const mySequence = sequence
  busy.value = true
  try {
    const flushed = await flushEditorBeforeDiscreteWrite()
    if (mySequence !== sequence) return
    if (flushed === 'failed' || flushed === 'still-dirty') {
      say('review.accept_flush_failed')
      return
    }
    const result = await acceptChangeIpc(chapterId, groupId, expectedTarget, force)
    if (mySequence !== sequence) return
    if (result.outcome === null) {
      pendingAccept.value = null
      if (result.error === null) return
      sayError(result.error)
      const message = diffMessage.value
      if (await reload(chapterId, mySequence)) diffMessage.value = message
      return
    }
    const outcome = result.outcome
    if (outcome.needs_confirmation) {
      pendingAccept.value = { groupId, expectedTarget, draft: outcome.unsigned_draft ?? '' }
      return
    }
    pendingAccept.value = null
    replaceEditorSegment(outcome.segment_id, {
      target_text: outcome.target_text,
      translation_origin: outcome.translation_origin,
      status: outcome.status,
    })
    if (await reload(chapterId, mySequence)) await advanceToPending()
  } finally {
    busy.value = false
  }
}

/** Takes over the reviewer's text of the pair the cursor is on (`review.accept_change`). */
export async function reviewAcceptChange(): Promise<void> {
  if (status.value !== 'open' || busy.value || openChapterId === null) return
  const current = reviewModeCurrentPair.value
  if (current === null) return say('review.change_none_current')
  if (current.decision === 'accepted') return say('review.change_done')
  if (!current.acceptable) return say('review.change_manual')
  await runAccept(openChapterId, current.groupId, current.mine.map((span) => span.text).join(''), false)
}

/** Writes the held-back accept over the unsigned draft the user has just been shown. */
export async function confirmPendingAccept(): Promise<void> {
  const waiting = pendingAccept.value
  if (status.value !== 'open' || busy.value || openChapterId === null || waiting === null) return
  await runAccept(openChapterId, waiting.groupId, waiting.expectedTarget, true)
}

export function cancelPendingAccept(): void {
  pendingAccept.value = null
}

/** Leaves the pair the cursor is on as it is and stops counting it as unprocessed (`review.skip_change`). */
export async function reviewSkipChange(): Promise<void> {
  if (status.value !== 'open' || busy.value || openChapterId === null) return
  const current = reviewModeCurrentPair.value
  if (current === null) return say('review.change_none_current')
  if (current.decision === 'accepted') return say('review.change_done')
  if (!current.hasRow) return say('review.change_manual')
  const chapterId = openChapterId
  const mySequence = sequence
  busy.value = true
  try {
    pendingAccept.value = null
    const result = await skipChangeIpc(chapterId, current.groupId)
    if (mySequence !== sequence) return
    if (result.error !== null) {
      sayError(result.error)
      const message = diffMessage.value
      if (await reload(chapterId, mySequence)) diffMessage.value = message
      return
    }
    if (await reload(chapterId, mySequence)) await advanceToPending()
  } finally {
    busy.value = false
  }
}
