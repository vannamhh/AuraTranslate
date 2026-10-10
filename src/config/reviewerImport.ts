import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

export type ReviewerFileKind = 'docx' | 'md'

export type ReviewerImportReplaces = {
  file_name: string
  stale: boolean
  user_group_count: number
  accepted_group_count: number
}

export type ReviewerImportChapter = {
  chapter_id: number
  chapter_ord: number
  title: string | null
  row_count: number
  replaces: ReviewerImportReplaces | null
}

export type ReviewerImportSkipped = { heading: string; row_count: number }

export type ReviewerImportPreview = {
  file_name: string
  file_kind: ReviewerFileKind
  chapters: ReviewerImportChapter[]
  skipped: ReviewerImportSkipped[]
  image_rows_ignored: number
}

export type ReviewerImportSummary = {
  chapter_count: number
  row_count: number
  replaced_count: number
  /** `null` when harvesting failed; the import itself is already written then. */
  harvest_candidate_count: number | null
  harvest_error: IpcError | null
}

export type ReviewerImportPreviewResult =
  | { outcome: 'loaded'; preview: ReviewerImportPreview }
  | { outcome: 'cancelled' }
  | { outcome: 'ipc_unavailable' }
  | { outcome: 'error'; error: IpcError }

export type ReviewerImportConfirmResult =
  | { summary: ReviewerImportSummary; error: null }
  | { summary: null; error: IpcError | null }

export type ReviewerImportAckResult = { ok: boolean; error: IpcError | null }

const CMD_OPEN_PREVIEW = 'reviewer_import_open_preview'
const CMD_CONFIRM = 'reviewer_import_confirm'
const CMD_CANCEL = 'reviewer_import_cancel'

const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- wire data may carry null params
    v.params !== null
  )
}

function hasIpcBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function failureOf(err: unknown, command: string): IpcError | null {
  if (isIpcError(err)) return err
  if (hasIpcBridge()) {
    console.error(`[reviewer-import] \`${command}\` failed with a non-IpcError value: ${String(err)}`)
    return UNKNOWN_IPC_ERROR
  }
  console.info(`[reviewer-import] cannot call \`${command}\` — running outside Tauri? ${String(err)}`)
  return null
}

function malformed(wire: unknown, command: string): IpcError {
  console.error(`[reviewer-import] \`${command}\` returned an unexpected shape: ${String(wire)}`)
  return UNKNOWN_IPC_ERROR
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function isReplaces(value: unknown): value is ReviewerImportReplaces {
  return (
    isObject(value) &&
    typeof value.file_name === 'string' &&
    typeof value.stale === 'boolean' &&
    typeof value.user_group_count === 'number' &&
    typeof value.accepted_group_count === 'number'
  )
}

function isChapter(value: unknown): value is ReviewerImportChapter {
  return (
    isObject(value) &&
    typeof value.chapter_id === 'number' &&
    typeof value.chapter_ord === 'number' &&
    (value.title === null || typeof value.title === 'string') &&
    typeof value.row_count === 'number' &&
    (value.replaces === null || isReplaces(value.replaces))
  )
}

function isSkipped(value: unknown): value is ReviewerImportSkipped {
  return isObject(value) && typeof value.heading === 'string' && typeof value.row_count === 'number'
}

function isPreview(value: unknown): value is ReviewerImportPreview {
  return (
    isObject(value) &&
    typeof value.file_name === 'string' &&
    (value.file_kind === 'docx' || value.file_kind === 'md') &&
    Array.isArray(value.chapters) &&
    value.chapters.every(isChapter) &&
    Array.isArray(value.skipped) &&
    value.skipped.every(isSkipped) &&
    typeof value.image_rows_ignored === 'number'
  )
}

function isSummary(value: unknown): value is ReviewerImportSummary {
  return (
    isObject(value) &&
    typeof value.chapter_count === 'number' &&
    typeof value.row_count === 'number' &&
    typeof value.replaced_count === 'number' &&
    (value.harvest_candidate_count === null || typeof value.harvest_candidate_count === 'number') &&
    (value.harvest_error === null || isIpcError(value.harvest_error))
  )
}

/** Opens the pick dialog in Rust, parses the file and keeps the plan there. Never throws. */
export async function reviewerImportOpenPreview(): Promise<ReviewerImportPreviewResult> {
  try {
    const wire = await invoke<unknown>(CMD_OPEN_PREVIEW)
    if (wire === null) return { outcome: 'cancelled' }
    if (!isPreview(wire)) return { outcome: 'error', error: malformed(wire, CMD_OPEN_PREVIEW) }
    return { outcome: 'loaded', preview: wire }
  } catch (err) {
    const error = failureOf(err, CMD_OPEN_PREVIEW)
    return error === null ? { outcome: 'ipc_unavailable' } : { outcome: 'error', error }
  }
}

export async function reviewerImportConfirm(): Promise<ReviewerImportConfirmResult> {
  try {
    const wire = await invoke<unknown>(CMD_CONFIRM)
    if (!isSummary(wire)) return { summary: null, error: malformed(wire, CMD_CONFIRM) }
    return { summary: wire, error: null }
  } catch (err) {
    return { summary: null, error: failureOf(err, CMD_CONFIRM) }
  }
}

export async function reviewerImportCancel(): Promise<ReviewerImportAckResult> {
  try {
    await invoke<unknown>(CMD_CANCEL)
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: failureOf(err, CMD_CANCEL) }
  }
}
