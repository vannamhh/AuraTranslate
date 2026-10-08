/**
 * State of the segment-alignment overlay. Matching, validation and "resolved" live in Rust; the
 * webview holds the last state Rust returned plus the cursor and the marks. Do NOT import this file
 * from `src/commands/index.ts`; the handlers are injected through `CommandDeps`
 * (`alignmentCommandDeps.ts`).
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { alignmentJoin, alignmentOpen, alignmentSkip, alignmentUnjoin } from './config/alignment'
import type { AlignmentResult, ChapterAlignment } from './config/alignment'
import type { IpcError } from './i18n'

export type AlignmentStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'no_chapter' | 'loaded'

export type AlignmentEntry =
  | { kind: 'segment'; id: number }
  | { kind: 'row'; id: number }
  | { kind: 'group'; id: number }

const overlayOpen = ref(false)
const status = ref<AlignmentStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const alignment = ref<ChapterAlignment | null>(null)
const cursor = ref(0)
const markedSegments = ref<number[]>([])
const markedRows = ref<number[]>([])
const busy = ref(false)
const actionError = ref<IpcError | null>(null)
const actionUnavailable = ref(false)
let sequence = 0

export const alignmentOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const alignmentStatus: DeepReadonly<Ref<AlignmentStatus>> = readonly(status)
export const alignmentLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const alignmentData: DeepReadonly<Ref<ChapterAlignment | null>> = readonly(alignment)
export const alignmentCursor: DeepReadonly<Ref<number>> = readonly(cursor)
export const alignmentMarkedSegments: DeepReadonly<Ref<number[]>> = readonly(markedSegments)
export const alignmentMarkedRows: DeepReadonly<Ref<number[]>> = readonly(markedRows)
export const alignmentBusy: DeepReadonly<Ref<boolean>> = readonly(busy)
export const alignmentActionError: DeepReadonly<Ref<IpcError | null>> = readonly(actionError)
export const alignmentActionUnavailable: DeepReadonly<Ref<boolean>> = readonly(actionUnavailable)

/** Items still to decide first (segments, then reviewer rows), then the existing groups. */
export const alignmentEntries = computed<AlignmentEntry[]>(() => {
  const a = alignment.value
  if (a === null) return []
  return [
    ...a.unmatched_segment_ids.map((id): AlignmentEntry => ({ kind: 'segment', id })),
    ...a.unmatched_row_ids.map((id): AlignmentEntry => ({ kind: 'row', id })),
    ...a.groups.map((g): AlignmentEntry => ({ kind: 'group', id: g.id })),
  ]
})

export const alignmentCurrentEntry = computed<AlignmentEntry | null>(() => alignmentEntries.value.at(cursor.value) ?? null)

export function setAlignmentCursor(index: number): void {
  const last = alignmentEntries.value.length - 1
  cursor.value = Math.max(0, Math.min(index, Math.max(last, 0)))
}

export function nextAlignmentEntry(): void {
  setAlignmentCursor(cursor.value + 1)
}

export function prevAlignmentEntry(): void {
  setAlignmentCursor(cursor.value - 1)
}

export function toggleAlignmentMark(): void {
  const entry = alignmentCurrentEntry.value
  if (entry === null || entry.kind === 'group') return
  const list = entry.kind === 'segment' ? markedSegments : markedRows
  list.value = list.value.includes(entry.id) ? list.value.filter((id) => id !== entry.id) : [...list.value, entry.id]
}

function clearMarks(): void {
  markedSegments.value = []
  markedRows.value = []
}

function adopt(result: AlignmentResult): void {
  if (result.alignment === null) return
  alignment.value = result.alignment
  clearMarks()
  setAlignmentCursor(cursor.value)
}

export async function openAlignmentOverlay(chapterId: number | null): Promise<void> {
  if (overlayOpen.value) return

  sequence += 1
  const mySequence = sequence
  overlayOpen.value = true
  status.value = 'unknown'
  loadError.value = null
  alignment.value = null
  cursor.value = 0
  clearMarks()
  busy.value = false
  actionError.value = null
  actionUnavailable.value = false

  if (chapterId === null) {
    status.value = 'no_chapter'
    return
  }

  const result = await alignmentOpen(chapterId)
  if (mySequence !== sequence) return
  if (result.alignment !== null) {
    alignment.value = result.alignment
    status.value = 'loaded'
    return
  }
  if (result.error === null) {
    status.value = 'ipc_unavailable'
    return
  }
  status.value = 'error'
  loadError.value = result.error
}

async function mutate(run: (chapterId: number) => Promise<AlignmentResult>): Promise<void> {
  const current = alignment.value
  if (busy.value || status.value !== 'loaded' || current === null) return

  busy.value = true
  actionError.value = null
  actionUnavailable.value = false
  const mySequence = sequence

  const result = await run(current.chapter_id)
  if (mySequence !== sequence) return

  busy.value = false
  if (result.alignment === null) {
    if (result.error === null) actionUnavailable.value = true
    else actionError.value = result.error
    return
  }
  adopt(result)
}

export async function joinAlignmentMarks(): Promise<void> {
  const segmentIds = [...markedSegments.value]
  const rowIds = [...markedRows.value]
  await mutate((chapterId) => alignmentJoin(chapterId, segmentIds, rowIds))
}

export async function skipAlignmentEntry(): Promise<void> {
  const entry = alignmentCurrentEntry.value
  if (entry === null || entry.kind === 'group') return
  const segmentId = entry.kind === 'segment' ? entry.id : null
  const rowId = entry.kind === 'row' ? entry.id : null
  await mutate((chapterId) => alignmentSkip(chapterId, segmentId, rowId))
}

export async function unjoinAlignmentEntry(): Promise<void> {
  const entry = alignmentCurrentEntry.value
  if (entry === null || entry.kind !== 'group') return
  const groupId = entry.id
  await mutate((chapterId) => alignmentUnjoin(chapterId, groupId))
}

export function closeAlignmentOverlay(): void {
  sequence += 1
  overlayOpen.value = false
  busy.value = false
}

export function resetAlignment(): void {
  sequence += 1
  overlayOpen.value = false
  status.value = 'unknown'
  loadError.value = null
  alignment.value = null
  cursor.value = 0
  clearMarks()
  busy.value = false
  actionError.value = null
  actionUnavailable.value = false
}
