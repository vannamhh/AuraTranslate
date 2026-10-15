import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { approveGlossaryCandidate, lookupGlossaryTerm, pendingGlossaryCandidates, rejectGlossaryCandidate } from './config/glossary'
import type { GlossaryCandidate } from './config/glossary'
import { isInconsistentHarvest } from './glossaryQueueState'
import { refreshWorkspaceCounts } from './workspaceCountsState'
import type { IpcError } from './i18n'

export type GlossaryHarvestRow = {
  candidate: GlossaryCandidate
  selected: boolean
  outcome: 'accepted' | 'rejected' | null
  /** Set by a failed write on this row only; cleared when the row is tried again. */
  error: IpcError | null
}

export type GlossaryHarvestStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'no_work' | 'loaded'

const overlayOpen = ref(false)
const status = ref<GlossaryHarvestStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const rows = ref<GlossaryHarvestRow[]>([])
const cursor = ref(0)
const busy = ref(false)

let sequence = 0

export const harvestOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const harvestStatus: DeepReadonly<Ref<GlossaryHarvestStatus>> = readonly(status)
export const harvestLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const harvestRows: DeepReadonly<Ref<readonly GlossaryHarvestRow[]>> = readonly(rows)
export const harvestCursor: DeepReadonly<Ref<number>> = readonly(cursor)
export const harvestBusy: DeepReadonly<Ref<boolean>> = readonly(busy)

/** Rows still waiting for a decision. */
export const harvestOpenRows = computed(() => rows.value.filter((r) => r.outcome === null))

/** Rows ticked and not yet decided: what "Nhận N mục đã chọn" will write. */
export const harvestSelectedCount = computed<number>(
  () => rows.value.filter((r) => r.outcome === null && r.selected).length,
)

export async function openGlossaryHarvest(): Promise<void> {
  if (overlayOpen.value) return

  sequence += 1
  const mine = sequence

  overlayOpen.value = true
  status.value = 'unknown'
  loadError.value = null
  rows.value = []
  cursor.value = 0
  busy.value = false

  const pending = await pendingGlossaryCandidates()
  if (mine !== sequence) return

  if (pending.candidates === null) {
    status.value = pending.error === null ? 'ipc_unavailable' : 'error'
    loadError.value = pending.error
    return
  }

  rows.value = pending.candidates
    .filter((c) => c.candidate_origin === 'review_harvest')
    .map((candidate) => ({
      candidate,
      selected: !isInconsistentHarvest(candidate),
      outcome: null,
      error: null,
    }))
  if (rows.value.length > 0) {
    status.value = 'loaded'
    return
  }

  const probe = await lookupGlossaryTerm('')
  if (mine !== sequence) return
  if (probe.found === 'unknown') {
    status.value = probe.error === null ? 'ipc_unavailable' : 'error'
    loadError.value = probe.error
    return
  }
  status.value = probe.workTierAvailable ? 'loaded' : 'no_work'
}

function advanceFrom(index: number): void {
  if (cursor.value !== index) return
  const next = rows.value.findIndex((r, i) => i > index && r.outcome === null)
  if (next !== -1) cursor.value = next
}

/** Closing is "leave in the queue": no write. Blocked while a write is in flight. */
export function closeGlossaryHarvest(): void {
  if (busy.value) return
  overlayOpen.value = false
}

export function setGlossaryHarvestCursor(index: number): void {
  if (rows.value.length === 0) return
  cursor.value = Math.min(Math.max(index, 0), rows.value.length - 1)
}

export function nextGlossaryHarvestRow(): void {
  setGlossaryHarvestCursor(cursor.value + 1)
}

export function prevGlossaryHarvestRow(): void {
  setGlossaryHarvestCursor(cursor.value - 1)
}

export function setGlossaryHarvestSelected(index: number, selected: boolean): void {
  const row = rows.value.at(index)
  if (row === undefined || row.outcome !== null) return
  row.selected = selected
}

async function approveRow(row: GlossaryHarvestRow): Promise<void> {
  const result = await approveGlossaryCandidate(row.candidate.id, row.candidate.proposed_translation, 'other')
  if (result.error !== null) {
    row.error = result.error
    return
  }
  row.error = null
  row.outcome = 'accepted'
  row.selected = false
  void refreshWorkspaceCounts()
}

async function rejectRow(row: GlossaryHarvestRow): Promise<void> {
  const result = await rejectGlossaryCandidate(row.candidate.id)
  if (result.error !== null) {
    row.error = result.error
    return
  }
  row.error = null
  row.outcome = 'rejected'
  row.selected = false
}

/** Accepts the row under the cursor. */
export async function acceptGlossaryHarvestRow(): Promise<void> {
  if (busy.value) return
  const row = rows.value.at(cursor.value)
  if (row === undefined || row.outcome !== null) return
  busy.value = true
  const mine = sequence
  const index = cursor.value
  await approveRow(row)
  if (mine !== sequence) return
  busy.value = false
  if (row.error === null) advanceFrom(index)
}

/** Rejects the row under the cursor. */
export async function rejectGlossaryHarvestRow(): Promise<void> {
  if (busy.value) return
  const row = rows.value.at(cursor.value)
  if (row === undefined || row.outcome !== null) return
  busy.value = true
  const mine = sequence
  const index = cursor.value
  await rejectRow(row)
  if (mine !== sequence) return
  busy.value = false
  if (row.error === null) advanceFrom(index)
}

/**
 * Accepts every ticked row, one at a time and in list order: each write is read back before the
 * next id is sent, so a `Conflict` (decided elsewhere) marks only its own row. A failed row keeps
 * its tick and its error; rows already accepted stay accepted.
 */
export async function acceptGlossaryHarvestSelected(): Promise<void> {
  if (busy.value) return
  const targets = rows.value.filter((r) => r.outcome === null && r.selected)
  if (targets.length === 0) return
  busy.value = true
  const mine = sequence
  for (const row of targets) {
    if (mine !== sequence) return
    await approveRow(row)
  }
  if (mine === sequence) busy.value = false
}

export function resetGlossaryHarvest(): void {
  sequence += 1
  overlayOpen.value = false
  status.value = 'unknown'
  loadError.value = null
  rows.value = []
  cursor.value = 0
  busy.value = false
}
