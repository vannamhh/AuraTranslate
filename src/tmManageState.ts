/**
 * State of the TM management overlay (FR62, FR63).
 *
 * Filters and search run in Rust: every change re-lists, and Rust caps the answer at 200 source
 * groups. Do NOT import this file from `src/commands/index.ts` (the command gates load it under
 * plain Node); the handlers are injected through `CommandDeps` and wired in `src/main.ts`.
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import {
  tmDeleteOthers,
  tmDeletePair,
  tmExportTier,
  tmListPairs,
  tmPushPairToGlobal,
  tmUpdatePairTarget,
} from './config/tm'
import type {
  TmHealthEntry,
  TmManageListing,
  TmManagePairOriginFilter,
  TmManageRow,
  TmManageTier,
  TmManageTierFilter,
  TmxImportSummary,
} from './config/tm'
import { glossaryExchangeBusy, resetGlossaryExchangeGate, setGlossaryExchangeBusy } from './glossaryExchangeGate'
import type { IpcError } from './i18n'

export type TmManageStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'loaded'

export type TmManageFlatRow = {
  row: TmManageRow
  source_text: string
  /** True on the first row of a group that holds 2+ distinct targets: the view draws the header. */
  groupHeader: boolean
  /** Key of the group's first row; the header's source text is labelled by it. */
  groupKey: string
  groupDistinctTargets: number
  /** Rows of a group with a single distinct target carry their own source text. */
  showSource: boolean
}

const overlayOpen = ref(false)
const status = ref<TmManageStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const listing = ref<TmManageListing | null>(null)
const searchQuery = ref('')
const pairOriginFilterState = ref<TmManagePairOriginFilter>('all')
const tierFilterState = ref<TmManageTierFilter>('both')
const cursor = ref(0)
const editing = ref(false)
let editPin: { key: string; row: TmManageRow; source: string } | null = null
const editTargetInput = ref('')
const deletePendingKey = ref<string | null>(null)
const bulkPending = ref(false)
const saving = ref(false)
const savingAction = ref<'save' | 'delete' | 'push' | 'bulk'>('save')
const actionError = ref<IpcError | null>(null)
const actionNotice = ref<'push_not_applicable' | 'work_not_open' | 'pushed' | 'pushed_unlisted' | null>(null)
const bulkDeleted = ref<{ work: number; global: number } | null>(null)

export const TM_MANAGE_SEARCH_DEBOUNCE_MS = 150

let session = 0
const exchangeTierState = ref<TmManageTier>('global')
const exportBusy = ref(false)
const exportError = ref<IpcError | null>(null)
const exportIpcUnavailable = ref(false)
const exportedPath = ref<string | null>(null)
const importDone = ref<TmxImportSummary | null>(null)
let listToken = 0
let searchTimer: ReturnType<typeof setTimeout> | null = null

export const tmManageOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const tmManageStatus: DeepReadonly<Ref<TmManageStatus>> = readonly(status)
export const tmManageLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const tmManageSearchQuery: DeepReadonly<Ref<string>> = readonly(searchQuery)
export const tmManagePairOriginFilter: DeepReadonly<Ref<TmManagePairOriginFilter>> = readonly(pairOriginFilterState)
export const tmManageTierFilter: DeepReadonly<Ref<TmManageTierFilter>> = readonly(tierFilterState)
export const tmManageCursor: DeepReadonly<Ref<number>> = readonly(cursor)
export const tmManageEditing: DeepReadonly<Ref<boolean>> = readonly(editing)
export const tmManageEditTarget: Ref<string> = editTargetInput
export const tmManageBulkPending: DeepReadonly<Ref<boolean>> = readonly(bulkPending)
export const tmManageSaving: DeepReadonly<Ref<boolean>> = readonly(saving)
export const tmManageSavingAction: DeepReadonly<Ref<'save' | 'delete' | 'push' | 'bulk'>> = readonly(savingAction)
export const tmManageActionError: DeepReadonly<Ref<IpcError | null>> = readonly(actionError)
export const tmManageActionNotice: DeepReadonly<
  Ref<'push_not_applicable' | 'work_not_open' | 'pushed' | 'pushed_unlisted' | null>
> = readonly(actionNotice)
export const tmManageBulkDeleted: DeepReadonly<Ref<{ work: number; global: number } | null>> = readonly(bulkDeleted)

export const tmManageExchangeTier: DeepReadonly<Ref<TmManageTier>> = readonly(exchangeTierState)
export const tmManageExportBusy: DeepReadonly<Ref<boolean>> = readonly(exportBusy)
export const tmManageExportError: DeepReadonly<Ref<IpcError | null>> = readonly(exportError)
export const tmManageExportIpcUnavailable: DeepReadonly<Ref<boolean>> = readonly(exportIpcUnavailable)
export const tmManageExportedPath: DeepReadonly<Ref<string | null>> = readonly(exportedPath)

export const tmManageImportDone: DeepReadonly<Ref<TmxImportSummary | null>> = readonly(importDone)

export function noteTmImportDone(summary: TmxImportSummary | null): void {
  importDone.value = summary
}

export const tmManageWorkOpen = computed<boolean>(() => listing.value?.work_open ?? false)
export const tmManageTmEmpty = computed<boolean>(() => listing.value?.tm_empty ?? false)
export const tmManageTotalGroups = computed<number>(() => listing.value?.total_groups ?? 0)
export const tmManageTotalPairs = computed<number>(() => listing.value?.total_pairs ?? 0)
export const tmManageShownGroups = computed<number>(() => listing.value?.groups.length ?? 0)
export const tmManageHealth = computed<TmHealthEntry[]>(() => listing.value?.health ?? [])

export const tmManageHealthTotal = computed<number>(() => tmManageHealth.value.reduce((sum, h) => sum + h.count, 0))

/** Pairs a bulk delete would remove: the others side over the current tier filter. */
export const tmManageOthersCount = computed<number>(() =>
  tmManageHealth.value.filter((h) => h.translation_origin !== 'self').reduce((sum, h) => sum + h.count, 0),
)

export const tmManageFlatRows = computed<TmManageFlatRow[]>(() => {
  const out: TmManageFlatRow[] = []
  for (const group of listing.value?.groups ?? []) {
    const header = group.distinct_targets >= 2
    group.rows.forEach((row, i) => {
      out.push({
        row,
        source_text: group.source_text,
        groupHeader: header && i === 0,
        groupKey: tmManageRowKey(group.rows[0]),
        groupDistinctTargets: group.distinct_targets,
        showSource: !header,
      })
    })
  }
  return out
})

export const tmManageCurrentRow = computed<TmManageRow | null>(
  () => tmManageFlatRows.value.at(cursor.value)?.row ?? null,
)

export const tmManageCurrentCopyCount = computed<number>(() => tmManageCurrentRow.value?.copies.length ?? 0)

export function tmManageRowKey(row: Pick<TmManageRow, 'tier' | 'unit_id'>): string {
  return `${row.tier}:${row.unit_id}`
}

export const tmManageDeletePending = computed<boolean>(() => {
  const row = tmManageCurrentRow.value
  return row !== null && deletePendingKey.value === tmManageRowKey(row)
})

export function tmManageEmptyReasonFor(
  s: TmManageStatus,
  tmEmpty: boolean,
  shownRows: number,
): 'not_loaded' | 'ipc_unavailable' | 'tm_empty' | 'filter_no_match' | null {
  if (s === 'unknown') return 'not_loaded'
  if (s === 'ipc_unavailable') return 'ipc_unavailable'
  if (s !== 'loaded') return null
  if (tmEmpty) return 'tm_empty'
  if (shownRows === 0) return 'filter_no_match'
  return null
}

function clearPending(): void {
  deletePendingKey.value = null
  bulkPending.value = false
}

async function loadRows(keepKey: string | null): Promise<boolean> {
  listToken += 1
  const mine = listToken
  const mySession = session
  const result = await tmListPairs(pairOriginFilterState.value, tierFilterState.value, searchQuery.value)
  if (mine !== listToken || mySession !== session) return false

  if (result.listing === null) {
    if (result.error?.code === 'work.none_open' && tierFilterState.value === 'work') {
      tierFilterState.value = 'both'
      return loadRows(keepKey)
    }
    listing.value = null
    cursor.value = 0
    status.value = result.error === null ? 'ipc_unavailable' : 'error'
    loadError.value = result.error
    return false
  }

  listing.value = result.listing
  status.value = 'loaded'
  loadError.value = null
  const flat = tmManageFlatRows.value
  if (editing.value && editPin !== null) {
    const pinned = editPin.key
    const pinnedAt = flat.findIndex((f) => tmManageRowKey(f.row) === pinned)
    if (pinnedAt !== -1) {
      cursor.value = pinnedAt
      return true
    }
    editing.value = false
    editPin = null
  }
  const at = keepKey === null ? -1 : flat.findIndex((f) => tmManageRowKey(f.row) === keepKey)
  cursor.value = at !== -1 ? at : Math.max(0, Math.min(cursor.value, flat.length - 1))
  return true
}

export async function openTmManage(): Promise<void> {
  if (overlayOpen.value) return
  resetTmManage()
  overlayOpen.value = true
  const mySession = session
  await loadRows(null)
  if (mySession === session && listing.value !== null) {
    exchangeTierState.value = listing.value.work_open ? 'work' : 'global'
  }
}

export async function refreshTmManage(): Promise<void> {
  if (!overlayOpen.value) return
  const current = tmManageCurrentRow.value
  await loadRows(current === null ? null : tmManageRowKey(current))
  if (listing.value !== null && !listing.value.work_open) exchangeTierState.value = 'global'
}

export function setTmManageExchangeTier(value: TmManageTier): void {
  if (exportBusy.value) return
  if (value === 'work' && !tmManageWorkOpen.value) return
  exchangeTierState.value = value
  exportedPath.value = null
  importDone.value = null
  exportError.value = null
  exportIpcUnavailable.value = false
}

export async function exportTmManageTier(): Promise<void> {
  if (!overlayOpen.value || exportBusy.value || glossaryExchangeBusy.value) return

  exportBusy.value = true
  setGlossaryExchangeBusy(true)
  exportError.value = null
  exportIpcUnavailable.value = false
  exportedPath.value = null
  importDone.value = null
  const mySession = session

  const result = await tmExportTier(exchangeTierState.value)
  if (mySession !== session) return

  exportBusy.value = false
  setGlossaryExchangeBusy(false)
  if (result.outcome === 'cancelled') return
  if (result.outcome === 'ipc_unavailable') {
    exportIpcUnavailable.value = true
    return
  }
  if (result.outcome === 'error') {
    exportError.value = result.error
    return
  }
  exportedPath.value = result.path
}

export function closeTmManage(): void {
  if (saving.value) return
  cancelSearchTimer()
  overlayOpen.value = false
}

function cancelSearchTimer(): void {
  if (searchTimer !== null) clearTimeout(searchTimer)
  searchTimer = null
}

export function resetTmManage(): void {
  cancelSearchTimer()
  session += 1
  listToken += 1
  overlayOpen.value = false
  status.value = 'unknown'
  loadError.value = null
  listing.value = null
  searchQuery.value = ''
  pairOriginFilterState.value = 'all'
  tierFilterState.value = 'both'
  cursor.value = 0
  editing.value = false
  editPin = null
  editTargetInput.value = ''
  clearPending()
  saving.value = false
  savingAction.value = 'save'
  actionError.value = null
  actionNotice.value = null
  bulkDeleted.value = null
  exchangeTierState.value = 'global'
  exportBusy.value = false
  exportError.value = null
  exportIpcUnavailable.value = false
  exportedPath.value = null
  importDone.value = null
  resetGlossaryExchangeGate()
}

function filtersLocked(): boolean {
  return editing.value || saving.value
}

function refilter(): void {
  cancelSearchTimer()
  cursor.value = 0
  clearPending()
  actionError.value = null
  actionNotice.value = null
  bulkDeleted.value = null
  void loadRows(null)
}

export function setTmManageSearch(query: string): void {
  if (filtersLocked()) return
  searchQuery.value = query
  cancelSearchTimer()
  cursor.value = 0
  clearPending()
  actionError.value = null
  actionNotice.value = null
  bulkDeleted.value = null
  const mySession = session
  searchTimer = setTimeout(() => {
    searchTimer = null
    if (mySession !== session || !overlayOpen.value) return
    void loadRows(null)
  }, TM_MANAGE_SEARCH_DEBOUNCE_MS)
}

export function setTmManagePairOriginFilter(value: TmManagePairOriginFilter): void {
  if (filtersLocked()) return
  pairOriginFilterState.value = value
  refilter()
}

export function setTmManageTierFilter(value: TmManageTierFilter): void {
  if (filtersLocked()) return
  tierFilterState.value = value
  refilter()
}

function moveTo(next: number): void {
  cursor.value = next
  editing.value = false
  clearPending()
}

export function nextTmManageRow(): void {
  if (!overlayOpen.value || saving.value || editing.value) return
  if (cursor.value < tmManageFlatRows.value.length - 1) moveTo(cursor.value + 1)
}

export function prevTmManageRow(): void {
  if (!overlayOpen.value || saving.value || editing.value) return
  if (cursor.value > 0) moveTo(cursor.value - 1)
}

export function beginTmManageEdit(): void {
  if (!overlayOpen.value || saving.value || editing.value) return
  const row = tmManageCurrentRow.value
  if (row === null) return
  cancelSearchTimer()
  editPin = { key: tmManageRowKey(row), row, source: currentSource() }
  clearPending()
  editing.value = true
  editTargetInput.value = row.target_text
  actionError.value = null
  actionNotice.value = null
  bulkDeleted.value = null
}

export function cancelTmManageEdit(): void {
  if (!overlayOpen.value || saving.value) return
  editing.value = false
  editPin = null
}

export function cancelTmManageDeleteConfirm(): void {
  clearPending()
}

async function onWriteFailure(error: IpcError, mySession: number, keepKey: string | null): Promise<void> {
  actionError.value = error
  await loadRows(keepKey)
  if (mySession !== session) return
  clearPending()
}

function currentSource(): string {
  return tmManageFlatRows.value.at(cursor.value)?.source_text ?? ''
}

export async function saveTmManageEdit(): Promise<void> {
  if (!overlayOpen.value || saving.value || !editing.value || editPin === null) return
  const { row, source } = editPin

  savingAction.value = 'save'
  saving.value = true
  actionError.value = null
  const mySession = session
  const result = await tmUpdatePairTarget(row.copies, source, row.target_text, editTargetInput.value)
  if (mySession !== session) return

  saving.value = false
  if (result.pair === null) {
    if (result.error !== null) {
      await onWriteFailure(result.error, mySession, tmManageRowKey(row))
      if (result.error.code === 'tm.pair_not_found') {
        editing.value = false
        editPin = null
      }
    }
    return
  }
  editing.value = false
  editPin = null
  await loadRows(tmManageRowKey(row))
}

export async function deleteTmManagePair(): Promise<void> {
  if (!overlayOpen.value || saving.value || editing.value) return
  const row = tmManageCurrentRow.value
  if (row === null) return
  const source = currentSource()
  const key = tmManageRowKey(row)

  if (deletePendingKey.value !== key) {
    clearPending()
    deletePendingKey.value = key
    actionError.value = null
    actionNotice.value = null
    bulkDeleted.value = null
    return
  }

  clearPending()
  savingAction.value = 'delete'
  saving.value = true
  actionError.value = null
  actionNotice.value = null
  const mySession = session
  const result = await tmDeletePair(row.copies, source, row.target_text)
  if (mySession !== session) return

  saving.value = false
  if (!result.ok) {
    if (result.error !== null) await onWriteFailure(result.error, mySession, key)
    return
  }
  editing.value = false
  await loadRows(null)
}

export async function deleteTmManageOthers(): Promise<void> {
  if (!overlayOpen.value || saving.value || editing.value) return
  if (tmManageOthersCount.value === 0) return

  if (!bulkPending.value) {
    clearPending()
    bulkPending.value = true
    actionError.value = null
    actionNotice.value = null
    bulkDeleted.value = null
    editing.value = false
    return
  }

  clearPending()
  savingAction.value = 'bulk'
  saving.value = true
  actionError.value = null
  actionNotice.value = null
  const mySession = session
  const result = await tmDeleteOthers(tierFilterState.value)
  if (mySession !== session) return

  saving.value = false
  if (result.outcome === null) {
    if (result.error !== null) await onWriteFailure(result.error, mySession, null)
    return
  }
  bulkDeleted.value = {
    work: result.outcome.deleted_work,
    global: result.outcome.deleted_global,
  }
  editing.value = false
  await loadRows(null)
}

export async function pushTmManagePair(): Promise<void> {
  if (!overlayOpen.value || saving.value || editing.value) return
  const row = tmManageCurrentRow.value
  if (row === null) return
  const source = currentSource()

  clearPending()
  actionError.value = null
  actionNotice.value = null
  bulkDeleted.value = null
  if (row.copies.some((c) => c.tier === 'global')) {
    actionNotice.value = 'push_not_applicable'
    return
  }
  if (!tmManageWorkOpen.value) {
    actionNotice.value = 'work_not_open'
    return
  }

  savingAction.value = 'push'
  saving.value = true
  const mySession = session
  const result = await tmPushPairToGlobal(row.copies, source, row.target_text)
  if (mySession !== session) return

  saving.value = false
  if (result.pair === null) {
    if (result.error !== null) await onWriteFailure(result.error, mySession, tmManageRowKey(row))
    return
  }
  editing.value = false
  const movedKey = tmManageRowKey(result.pair)
  await loadRows(movedKey)
  if (mySession !== session) return
  actionNotice.value = tmManageFlatRows.value.some((f) => tmManageRowKey(f.row) === movedKey)
    ? 'pushed'
    : 'pushed_unlisted'
}
