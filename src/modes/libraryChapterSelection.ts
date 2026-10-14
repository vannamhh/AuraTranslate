/**
 * View state of the Work page's Chapter list: selection, status filter, search, sort, grouping,
 * and the rules that enable the action-bar buttons. Pure functions take rows as arguments; the
 * refs below are the only module state. Counts and the derived status come from Rust (AD-21);
 * nothing here recomputes a business rule.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import type { ChapterRow } from '../config/chapter'

export type ChapterGroupKey = 'in_progress' | 'paused' | 'done' | 'not_started' | 'unknown'
export type ChapterFilter = 'all' | Exclude<ChapterGroupKey, 'unknown'>
export type ChapterSortKey = 'order' | 'recent'

export const CHAPTER_GROUP_ORDER: readonly ChapterGroupKey[] = ['in_progress', 'paused', 'done', 'not_started', 'unknown']
export const NOT_STARTED_PREVIEW = 2

export type ChapterViewItem =
  | { kind: 'group'; key: ChapterGroupKey; total: number }
  | { kind: 'chapter'; row: ChapterRow }
  | { kind: 'more'; hidden: number }

export type ChapterView = {
  items: ChapterViewItem[]
  matched: number
}

export type ChapterActionRules = {
  merge: boolean
  split: boolean
  rename: boolean
  moveUp: boolean
  moveDown: boolean
  open: boolean
  status: boolean
  export: boolean
}

const selectedIds = ref<ReadonlySet<number>>(new Set())
const anchorId = ref<number | null>(null)
const statusFilter = ref<ChapterFilter>('all')
const sortKey = ref<ChapterSortKey>('order')
const notStartedExpanded = ref(false)
const statusMenuOpen = ref(false)

/** Writable for `v-model`, like `chapterRenameDraft`. */
export const libraryChapterSearch = ref('')

export const libraryChapterSelectedIds: DeepReadonly<Ref<ReadonlySet<number>>> = readonly(selectedIds)
export const libraryChapterAnchorId: DeepReadonly<Ref<number | null>> = readonly(anchorId)
export const libraryChapterFilter: DeepReadonly<Ref<ChapterFilter>> = readonly(statusFilter)
export const libraryChapterSort: DeepReadonly<Ref<ChapterSortKey>> = readonly(sortKey)
export const libraryChapterNotStartedExpanded: DeepReadonly<Ref<boolean>> = readonly(notStartedExpanded)
export const libraryChapterStatusMenuOpen: DeepReadonly<Ref<boolean>> = readonly(statusMenuOpen)

export function chapterGroupOf(status: string): ChapterGroupKey {
  if (status === 'in_progress' || status === 'paused' || status === 'done' || status === 'not_started') return status
  return 'unknown'
}

export function countChaptersByGroup(rows: readonly ChapterRow[]): Record<ChapterGroupKey, number> {
  const counts: Record<ChapterGroupKey, number> = { in_progress: 0, paused: 0, done: 0, not_started: 0, unknown: 0 }
  for (const row of rows) counts[chapterGroupOf(row.status)] += 1
  return counts
}

/** Name (case-insensitive substring) or number (substring of `ord`). Blank query matches all. */
export function chapterMatchesQuery(row: ChapterRow, query: string): boolean {
  const needle = query.trim().toLowerCase()
  if (needle === '') return true
  if (String(row.ord).includes(needle)) return true
  return (row.title ?? '').toLowerCase().includes(needle)
}

function compareRows(sort: ChapterSortKey): (a: ChapterRow, b: ChapterRow) => number {
  if (sort === 'recent') {
    return (a, b) => {
      const byTime = Date.parse(b.updated_at) - Date.parse(a.updated_at)
      if (Number.isFinite(byTime) && byTime !== 0) return byTime
      return a.ord - b.ord || a.chapter_id - b.chapter_id
    }
  }
  return (a, b) => a.ord - b.ord || a.chapter_id - b.chapter_id
}

export function buildChapterView(
  rows: readonly ChapterRow[],
  filter: ChapterFilter,
  query: string,
  sort: ChapterSortKey,
  expandNotStarted: boolean,
): ChapterView {
  const compare = compareRows(sort)
  const matched = rows
    .filter((row) => (filter === 'all' || chapterGroupOf(row.status) === filter) && chapterMatchesQuery(row, query))
    .sort(compare)
  const collapsible = filter === 'all' && query.trim() === '' && !expandNotStarted
  const items: ChapterViewItem[] = []
  for (const key of CHAPTER_GROUP_ORDER) {
    const inGroup = matched.filter((row) => chapterGroupOf(row.status) === key)
    if (inGroup.length === 0) continue
    items.push({ kind: 'group', key, total: inGroup.length })
    const folded = collapsible && key === 'not_started' && inGroup.length > NOT_STARTED_PREVIEW
    const shown = folded ? inGroup.slice(0, NOT_STARTED_PREVIEW) : inGroup
    for (const row of shown) items.push({ kind: 'chapter', row })
    if (folded) items.push({ kind: 'more', hidden: inGroup.length - NOT_STARTED_PREVIEW })
  }
  return { items, matched: matched.length }
}

export function visibleChapterIds(items: readonly ChapterViewItem[]): number[] {
  const ids: number[] = []
  for (const item of items) if (item.kind === 'chapter') ids.push(item.row.chapter_id)
  return ids
}

/** Index of a chapter row inside the flat item list, or -1. */
export function flatIndexOfChapter(items: readonly ChapterViewItem[], chapterId: number): number {
  return items.findIndex((item) => item.kind === 'chapter' && item.row.chapter_id === chapterId)
}

export type ChapterClickModifiers = { toggle: boolean; range: boolean }

export function applyChapterClick(
  current: ReadonlySet<number>,
  anchor: number | null,
  visible: readonly number[],
  targetId: number,
  mods: ChapterClickModifiers,
): { selected: Set<number>; anchor: number } {
  if (mods.range && anchor !== null) {
    const from = visible.indexOf(anchor)
    const to = visible.indexOf(targetId)
    if (from !== -1 && to !== -1) {
      const span = visible.slice(Math.min(from, to), Math.max(from, to) + 1)
      const selected = mods.toggle ? new Set(current) : new Set<number>()
      for (const id of span) selected.add(id)
      return { selected, anchor }
    }
  }
  if (mods.toggle) {
    const selected = new Set(current)
    if (selected.has(targetId)) selected.delete(targetId)
    else selected.add(targetId)
    return { selected, anchor: targetId }
  }
  return { selected: new Set([targetId]), anchor: targetId }
}

/** Selected rows in list order. */
export function selectedRowsInOrder(rows: readonly ChapterRow[], selected: ReadonlySet<number>): ChapterRow[] {
  return rows.filter((row) => selected.has(row.chapter_id))
}

/** Button enablement; Rust stays the authority and refuses what these let through. */
export function chapterActionRules(rows: readonly ChapterRow[], selected: ReadonlySet<number>): ChapterActionRules {
  const picked = selectedRowsInOrder(rows, selected)
  const count = picked.length
  const only = count === 1 ? picked[0] : null
  let contiguous = count >= 2
  if (contiguous) {
    const indices = picked.map((row) => rows.indexOf(row))
    contiguous = indices.every((index, at) => at === 0 || index === indices[at - 1] + 1)
  }
  return {
    merge: contiguous,
    split: only !== null && only.segment_count >= 2,
    rename: only !== null,
    moveUp: only !== null,
    moveDown: only !== null,
    open: only !== null,
    status: count >= 1,
    export: count >= 1,
  }
}

export function setChapterSelection(ids: ReadonlySet<number>, anchor: number | null): void {
  selectedIds.value = ids
  anchorId.value = anchor
}

export function clearChapterSelection(): void {
  selectedIds.value = new Set()
  anchorId.value = null
  statusMenuOpen.value = false
}

/** Drops ids that are no longer in the list after a reload. */
export function pruneChapterSelection(rows: readonly ChapterRow[]): void {
  const known = new Set(rows.map((row) => row.chapter_id))
  const kept = new Set([...selectedIds.value].filter((id) => known.has(id)))
  if (kept.size !== selectedIds.value.size) selectedIds.value = kept
  if (anchorId.value !== null && !known.has(anchorId.value)) anchorId.value = null
}

export function setChapterFilter(filter: ChapterFilter): void {
  statusFilter.value = filter
}

export function setChapterSort(sort: ChapterSortKey): void {
  sortKey.value = sort
}

export function toggleNotStartedGroup(): void {
  notStartedExpanded.value = !notStartedExpanded.value
}

export function toggleChapterStatusMenu(): void {
  statusMenuOpen.value = !statusMenuOpen.value
}

export function closeChapterStatusMenu(): void {
  statusMenuOpen.value = false
}

export function resetLibraryChapterSelection(): void {
  selectedIds.value = new Set()
  anchorId.value = null
  statusFilter.value = 'all'
  sortKey.value = 'order'
  notStartedExpanded.value = false
  statusMenuOpen.value = false
  libraryChapterSearch.value = ''
}
