import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

export type AlignmentRowKind = 'text' | 'alt' | 'caption'
export type AlignmentSegmentRole = 'alt' | 'caption'

export type AlignmentRow = {
  id: number
  kind: AlignmentRowKind
  source_text: string | null
  target_text: string
}

export type AlignmentSegment = {
  id: number
  ord: number
  role: AlignmentSegmentRole | null
  source_text: string
  target_text: string
}

export type AlignmentGroup = {
  id: number
  decided_by: 'machine' | 'user'
  row_ids: number[]
  segment_ids: number[]
}

export type ChapterAlignment = {
  chapter_id: number
  file_name: string
  file_kind: 'docx' | 'md'
  rows: AlignmentRow[]
  segments: AlignmentSegment[]
  groups: AlignmentGroup[]
  unmatched_row_ids: number[]
  unmatched_segment_ids: number[]
  is_resolved: boolean
}

export type ReviewDiffSpan = { kind: 'equal' | 'delete' | 'insert'; text: string }

export type ReviewDiffPair = {
  group_id: number
  decided_by: 'machine' | 'user'
  segment_ids: number[]
  row_ids: number[]
  spans: ReviewDiffSpan[]
}

export type ReviewDiff = { chapter_id: number; pairs: ReviewDiffPair[] }

export type ReviewDiffResult =
  | { diff: ReviewDiff; error: null }
  | { diff: null; error: IpcError | null }

export type AlignmentResult =
  | { alignment: ChapterAlignment; error: null }
  | { alignment: null; error: IpcError | null }

const CMD_OPEN = 'alignment_open'
const CMD_JOIN = 'alignment_join'
const CMD_SKIP = 'alignment_skip'
const CMD_UNJOIN = 'alignment_unjoin'
const CMD_DIFF = 'review_diff'

const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function isIpcError(value: unknown): value is IpcError {
  if (!isObject(value)) return false
  return (
    typeof value.code === 'string' &&
    typeof value.message_key === 'string' &&
    typeof value.retryable === 'boolean' &&
    isObject(value.params)
  )
}

function hasIpcBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function failureOf(err: unknown, command: string): IpcError | null {
  if (isIpcError(err)) return err
  if (hasIpcBridge()) {
    console.error(`[alignment] \`${command}\` failed with a non-IpcError value: ${String(err)}`)
    return UNKNOWN_IPC_ERROR
  }
  console.info(`[alignment] cannot call \`${command}\` — running outside Tauri? ${String(err)}`)
  return null
}

function isIdList(value: unknown): value is number[] {
  return Array.isArray(value) && value.every((v) => typeof v === 'number')
}

function isRow(value: unknown): value is AlignmentRow {
  return (
    isObject(value) &&
    typeof value.id === 'number' &&
    (value.kind === 'text' || value.kind === 'alt' || value.kind === 'caption') &&
    (value.source_text === null || typeof value.source_text === 'string') &&
    typeof value.target_text === 'string'
  )
}

function isSegment(value: unknown): value is AlignmentSegment {
  return (
    isObject(value) &&
    typeof value.id === 'number' &&
    typeof value.ord === 'number' &&
    (value.role === null || value.role === 'alt' || value.role === 'caption') &&
    typeof value.source_text === 'string' &&
    typeof value.target_text === 'string'
  )
}

function isGroup(value: unknown): value is AlignmentGroup {
  return (
    isObject(value) &&
    typeof value.id === 'number' &&
    (value.decided_by === 'machine' || value.decided_by === 'user') &&
    isIdList(value.row_ids) &&
    isIdList(value.segment_ids)
  )
}

function isAlignment(value: unknown): value is ChapterAlignment {
  return (
    isObject(value) &&
    typeof value.chapter_id === 'number' &&
    typeof value.file_name === 'string' &&
    (value.file_kind === 'docx' || value.file_kind === 'md') &&
    Array.isArray(value.rows) &&
    value.rows.every(isRow) &&
    Array.isArray(value.segments) &&
    value.segments.every(isSegment) &&
    Array.isArray(value.groups) &&
    value.groups.every(isGroup) &&
    isIdList(value.unmatched_row_ids) &&
    isIdList(value.unmatched_segment_ids) &&
    typeof value.is_resolved === 'boolean'
  )
}

function isDiffSpan(value: unknown): value is ReviewDiffSpan {
  return (
    isObject(value) &&
    (value.kind === 'equal' || value.kind === 'delete' || value.kind === 'insert') &&
    typeof value.text === 'string'
  )
}

function isDiffPair(value: unknown): value is ReviewDiffPair {
  return (
    isObject(value) &&
    typeof value.group_id === 'number' &&
    (value.decided_by === 'machine' || value.decided_by === 'user') &&
    isIdList(value.segment_ids) &&
    isIdList(value.row_ids) &&
    Array.isArray(value.spans) &&
    value.spans.every(isDiffSpan)
  )
}

function isReviewDiff(value: unknown): value is ReviewDiff {
  return (
    isObject(value) &&
    typeof value.chapter_id === 'number' &&
    Array.isArray(value.pairs) &&
    value.pairs.every(isDiffPair)
  )
}

async function call(command: string, args: Record<string, unknown>): Promise<AlignmentResult> {
  try {
    const wire = await invoke<unknown>(command, args)
    if (!isAlignment(wire)) {
      console.error(`[alignment] \`${command}\` returned an unexpected shape: ${String(wire)}`)
      return { alignment: null, error: UNKNOWN_IPC_ERROR }
    }
    return { alignment: wire, error: null }
  } catch (err) {
    return { alignment: null, error: failureOf(err, command) }
  }
}

/** Reads both sides of the reviewer copy of a Chapter and how they are grouped. Never throws. */
export async function alignmentOpen(chapterId: number): Promise<AlignmentResult> {
  return call(CMD_OPEN, { chapterId })
}

/** Joins the given segments and reviewer rows into one user group. Never throws. */
export async function alignmentJoin(chapterId: number, segmentIds: number[], rowIds: number[]): Promise<AlignmentResult> {
  return call(CMD_JOIN, { chapterId, segmentIds, rowIds })
}

/** Sets exactly one segment or one reviewer row aside. Never throws. */
export async function alignmentSkip(
  chapterId: number,
  segmentId: number | null,
  rowId: number | null,
): Promise<AlignmentResult> {
  return call(CMD_SKIP, { chapterId, segmentId, rowId })
}

/** Dissolves a group; its members return to the list. Never throws. */
export async function alignmentUnjoin(chapterId: number, groupId: number): Promise<AlignmentResult> {
  return call(CMD_UNJOIN, { chapterId, groupId })
}

/** Word-level diff of every aligned pair (mine = old, reviewer = new), computed in Rust. Never throws. */
export async function reviewDiff(chapterId: number): Promise<ReviewDiffResult> {
  try {
    const wire = await invoke<unknown>(CMD_DIFF, { chapterId })
    if (!isReviewDiff(wire)) {
      console.error(`[alignment] \`${CMD_DIFF}\` returned an unexpected shape: ${String(wire)}`)
      return { diff: null, error: UNKNOWN_IPC_ERROR }
    }
    return { diff: wire, error: null }
  } catch (err) {
    return { diff: null, error: failureOf(err, CMD_DIFF) }
  }
}
