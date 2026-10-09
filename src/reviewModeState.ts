/**
 * Review Mode: a read-only layout state inside Workspace (AD-24), not a fourth mode. The data is
 * whatever `alignment_open` returned; nothing here writes. The layout it shows is never persisted.
 * Do NOT import this file from `src/commands/index.ts`; the handlers are injected through
 * `CommandDeps` (`reviewModeCommandDeps.ts`).
 */
import { computed, nextTick, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { enterFocus } from './commands'
import { alignmentOpen, reviewDiff } from './config/alignment'
import type { AlignmentRow, AlignmentRowKind, AlignmentSegment, ReviewDiffPair, ReviewDiffSpan } from './config/alignment'
import type { IpcError } from './i18n'
import { activeDockPanelId, resetDockSuspended, setDockSuspended } from './layout/dockController'
import { currentMode } from './modes/modeState'
import { resetReviewScrollers, scrollReviewPairIntoView } from './reviewScrollSync'

export type ReviewModeStatus = 'idle' | 'loading' | 'open' | 'not_imported' | 'stale' | 'no_chapter' | 'error' | 'diff_failed'

const FALLBACK_OWNER = 'panel.grid'
const MINE_OWNER = 'panel.review_mine'

export type ReviewPairView = {
  key: string
  changed: boolean
  hasSegment: boolean
  hasRow: boolean
  rowKind: AlignmentRowKind | null
  mine: ReviewDiffSpan[]
  copy: ReviewDiffSpan[]
}

export type ReviewDiffMessage = { key: string; params: Record<string, string> }

const status = ref<ReviewModeStatus>('idle')
const segments = ref<AlignmentSegment[]>([])
const rows = ref<AlignmentRow[]>([])
const pairs = ref<ReviewDiffPair[]>([])
const unmatchedSegmentIds = ref<number[]>([])
const unmatchedRowIds = ref<number[]>([])
const diffCursor = ref(-1)
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
    return {
      key: `g${p.group_id}`,
      changed: p.spans.some((span) => span.kind !== 'equal'),
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
  const changed = reviewModePairs.value.filter((p) => p.changed)
  return changed[diffCursor.value]?.key ?? null
})
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
  return reviewModePairs.value.filter((p) => p.changed).map((p) => p.key)
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
