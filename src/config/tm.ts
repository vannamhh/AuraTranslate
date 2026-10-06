import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

export type TmManageTier = 'work' | 'global'
export type TmManageTierFilter = 'both' | 'work' | 'global'
export type TmManagePairOriginFilter = 'all' | 'others' | 'self' | 'other' | 'bilingual_import'
export type TmPairOrigin = 'self' | 'other' | 'bilingual_import'
export type TmPairSide = 'mine' | 'others'

export type TmCopy = { tier: TmManageTier; unit_id: number }

export type TmManageRow = {
  tier: TmManageTier
  unit_id: number
  copies: TmCopy[]
  hidden_copies: number
  target_text: string
  translation_origin: TmPairOrigin
  side: TmPairSide
  created_at: string
}

export type TmManageGroup = {
  source_text: string
  distinct_targets: number
  rows: TmManageRow[]
}

export type TmHealthEntry = { translation_origin: TmPairOrigin; count: number }

export type TmManageListing = {
  work_open: boolean
  tm_empty: boolean
  health: TmHealthEntry[]
  total_pairs: number
  total_groups: number
  groups: TmManageGroup[]
}

export type TmStoredPair = Omit<TmManageRow, 'copies' | 'hidden_copies'> & {
  source_text: string
}

export type TmDeleteOthersOutcome = {
  deleted_work: number
  deleted_global: number
}

export type TmxImportPreview = {
  file_name: string
  tier: TmManageTier
  unit_count: number
  new_count: number
  already_count: number
  skipped_count: number
}

export type TmxImportSummary = { inserted: number; already_count: number; future_dated_count: number }

export type TmExportResult =
  | { outcome: 'done'; path: string }
  | { outcome: 'cancelled' }
  | { outcome: 'ipc_unavailable' }
  | { outcome: 'error'; error: IpcError }

export type TmImportPreviewResult =
  | { outcome: 'loaded'; preview: TmxImportPreview }
  | { outcome: 'cancelled' }
  | { outcome: 'ipc_unavailable' }
  | { outcome: 'error'; error: IpcError }

export type TmConfirmImportResult = { summary: TmxImportSummary | null; error: IpcError | null }

export type TmListResult = {
  listing: TmManageListing | null
  error: IpcError | null
}
export type TmPairResult = {
  pair: TmStoredPair | null
  error: IpcError | null
}
export type TmAckResult = { ok: boolean; error: IpcError | null }
export type TmDeleteOthersResult = {
  outcome: TmDeleteOthersOutcome | null
  error: IpcError | null
}

const CMD_TM_LIST_PAIRS = 'tm_list_pairs'
const CMD_TM_UPDATE_PAIR_TARGET = 'tm_update_pair_target'
const CMD_TM_DELETE_PAIR = 'tm_delete_pair'
const CMD_TM_DELETE_OTHERS = 'tm_delete_others'
const CMD_TM_PUSH_PAIR_TO_GLOBAL = 'tm_push_pair_to_global'
const CMD_TM_EXPORT_TIER = 'tm_export_tier'
const CMD_TM_OPEN_IMPORT_PREVIEW = 'tm_open_import_preview'
const CMD_TM_CONFIRM_IMPORT = 'tm_confirm_import'
const CMD_TM_CANCEL_IMPORT = 'tm_cancel_import'

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
    console.error(`[tm] \`${command}\` failed with a non-IpcError value: ${String(err)}`)
    return UNKNOWN_IPC_ERROR
  }
  console.info(`[tm] cannot call \`${command}\` — running outside Tauri? ${String(err)}`)
  return null
}

function isTier(value: unknown): value is TmManageTier {
  return value === 'work' || value === 'global'
}

function isPairOrigin(value: unknown): value is TmPairOrigin {
  return value === 'self' || value === 'other' || value === 'bilingual_import'
}

function isSide(value: unknown): value is TmPairSide {
  return value === 'mine' || value === 'others'
}

function isCopy(value: unknown): value is TmCopy {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<TmCopy>
  return isTier(v.tier) && typeof v.unit_id === 'number'
}

function isPairFields(value: unknown): value is Omit<TmManageRow, 'copies' | 'hidden_copies'> {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<TmManageRow>
  return (
    isTier(v.tier) &&
    typeof v.unit_id === 'number' &&
    typeof v.target_text === 'string' &&
    isPairOrigin(v.translation_origin) &&
    isSide(v.side) &&
    typeof v.created_at === 'string'
  )
}

function isManageRow(value: unknown): value is TmManageRow {
  if (!isPairFields(value)) return false
  const copies = (value as Partial<TmManageRow>).copies
  const hidden = (value as Partial<TmManageRow>).hidden_copies
  return Array.isArray(copies) && copies.length > 0 && copies.every(isCopy) && isCount(hidden)
}

function isStoredPair(value: unknown): value is TmStoredPair {
  return isPairFields(value) && typeof (value as Partial<TmStoredPair>).source_text === 'string'
}

function isManageGroup(value: unknown): value is TmManageGroup {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<TmManageGroup>
  return (
    typeof v.source_text === 'string' &&
    typeof v.distinct_targets === 'number' &&
    Array.isArray(v.rows) &&
    v.rows.every(isManageRow)
  )
}

function isHealthEntry(value: unknown): value is TmHealthEntry {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<TmHealthEntry>
  return isPairOrigin(v.translation_origin) && typeof v.count === 'number'
}

function isListing(value: unknown): value is TmManageListing {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<TmManageListing>
  return (
    typeof v.work_open === 'boolean' &&
    typeof v.tm_empty === 'boolean' &&
    typeof v.total_pairs === 'number' &&
    typeof v.total_groups === 'number' &&
    Array.isArray(v.health) &&
    v.health.every(isHealthEntry) &&
    Array.isArray(v.groups) &&
    v.groups.every(isManageGroup)
  )
}

function isDeleteOthersOutcome(value: unknown): value is TmDeleteOthersOutcome {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<TmDeleteOthersOutcome>
  return typeof v.deleted_work === 'number' && typeof v.deleted_global === 'number'
}

function malformed(value: unknown, command: string): IpcError {
  console.error(`[tm] \`${command}\` returned a malformed payload: ${JSON.stringify(value)}`)
  return UNKNOWN_IPC_ERROR
}

/** Both tiers, filtered; Rust caps the answer at 200 source groups. Never throws. */
export async function tmListPairs(
  pairOrigin: TmManagePairOriginFilter,
  tier: TmManageTierFilter,
  search: string,
): Promise<TmListResult> {
  try {
    const listing = await invoke<unknown>(CMD_TM_LIST_PAIRS, {
      pairOrigin,
      tier,
      search,
    })
    if (!isListing(listing)) return { listing: null, error: malformed(listing, CMD_TM_LIST_PAIRS) }
    return { listing, error: null }
  } catch (err) {
    return { listing: null, error: failureOf(err, CMD_TM_LIST_PAIRS) }
  }
}

export async function tmUpdatePairTarget(
  copies: TmCopy[],
  sourceText: string,
  expectedTarget: string,
  targetText: string,
): Promise<TmPairResult> {
  try {
    const pair = await invoke<unknown>(CMD_TM_UPDATE_PAIR_TARGET, {
      copies,
      sourceText,
      expectedTarget,
      targetText,
    })
    if (!isStoredPair(pair)) return { pair: null, error: malformed(pair, CMD_TM_UPDATE_PAIR_TARGET) }
    return { pair, error: null }
  } catch (err) {
    return { pair: null, error: failureOf(err, CMD_TM_UPDATE_PAIR_TARGET) }
  }
}

export async function tmDeletePair(copies: TmCopy[], sourceText: string, expectedTarget: string): Promise<TmAckResult> {
  try {
    await invoke<unknown>(CMD_TM_DELETE_PAIR, {
      copies,
      sourceText,
      expectedTarget,
    })
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: failureOf(err, CMD_TM_DELETE_PAIR) }
  }
}

export async function tmDeleteOthers(tier: TmManageTierFilter): Promise<TmDeleteOthersResult> {
  try {
    const outcome = await invoke<unknown>(CMD_TM_DELETE_OTHERS, { tier })
    if (!isDeleteOthersOutcome(outcome)) return { outcome: null, error: malformed(outcome, CMD_TM_DELETE_OTHERS) }
    return { outcome, error: null }
  } catch (err) {
    return { outcome: null, error: failureOf(err, CMD_TM_DELETE_OTHERS) }
  }
}

export async function tmPushPairToGlobal(
  copies: TmCopy[],
  sourceText: string,
  expectedTarget: string,
): Promise<TmPairResult> {
  try {
    const pair = await invoke<unknown>(CMD_TM_PUSH_PAIR_TO_GLOBAL, {
      copies,
      sourceText,
      expectedTarget,
    })
    if (!isStoredPair(pair)) return { pair: null, error: malformed(pair, CMD_TM_PUSH_PAIR_TO_GLOBAL) }
    return { pair, error: null }
  } catch (err) {
    return { pair: null, error: failureOf(err, CMD_TM_PUSH_PAIR_TO_GLOBAL) }
  }
}

function isCount(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value >= 0
}

function isImportPreview(value: unknown): value is TmxImportPreview {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<TmxImportPreview>
  return (
    typeof v.file_name === 'string' &&
    isTier(v.tier) &&
    isCount(v.unit_count) &&
    isCount(v.new_count) &&
    isCount(v.already_count) &&
    isCount(v.skipped_count)
  )
}

function isImportSummary(value: unknown): value is TmxImportSummary {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<TmxImportSummary>
  return isCount(v.inserted) && isCount(v.already_count) && isCount(v.future_dated_count)
}

/** Opens the save dialog in Rust and writes one tier as TMX. `null` from Rust is a cancelled dialog. Never throws. */
export async function tmExportTier(tier: TmManageTier): Promise<TmExportResult> {
  try {
    const path = await invoke<unknown>(CMD_TM_EXPORT_TIER, { tier })
    if (path === null) return { outcome: 'cancelled' }
    if (typeof path !== 'string' || path === '') {
      return { outcome: 'error', error: malformed(path, CMD_TM_EXPORT_TIER) }
    }
    return { outcome: 'done', path }
  } catch (err) {
    const error = failureOf(err, CMD_TM_EXPORT_TIER)
    return error === null ? { outcome: 'ipc_unavailable' } : { outcome: 'error', error }
  }
}

/** Opens the pick dialog in Rust, parses the file and keeps the plan there. Never throws. */
export async function tmOpenImportPreview(tier: TmManageTier): Promise<TmImportPreviewResult> {
  try {
    const wire = await invoke<unknown>(CMD_TM_OPEN_IMPORT_PREVIEW, { tier })
    if (wire === null) return { outcome: 'cancelled' }
    if (!isImportPreview(wire)) {
      return { outcome: 'error', error: malformed(wire, CMD_TM_OPEN_IMPORT_PREVIEW) }
    }
    return { outcome: 'loaded', preview: wire }
  } catch (err) {
    const error = failureOf(err, CMD_TM_OPEN_IMPORT_PREVIEW)
    return error === null ? { outcome: 'ipc_unavailable' } : { outcome: 'error', error }
  }
}

export async function tmConfirmImport(fileIsMine: boolean): Promise<TmConfirmImportResult> {
  try {
    const wire = await invoke<unknown>(CMD_TM_CONFIRM_IMPORT, { fileIsMine })
    if (!isImportSummary(wire)) return { summary: null, error: malformed(wire, CMD_TM_CONFIRM_IMPORT) }
    return { summary: wire, error: null }
  } catch (err) {
    return { summary: null, error: failureOf(err, CMD_TM_CONFIRM_IMPORT) }
  }
}

export async function tmCancelImport(): Promise<TmAckResult> {
  try {
    await invoke<unknown>(CMD_TM_CANCEL_IMPORT)
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: failureOf(err, CMD_TM_CANCEL_IMPORT) }
  }
}
