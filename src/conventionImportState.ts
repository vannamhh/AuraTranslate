import { computed, readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { UNKNOWN_IPC_ERROR } from './config/aitranslate'
import { cancelConventionImport, confirmConventionImport, openConventionImportPreview } from './config/convention'
import type { ConventionImportDecision, ConventionImportPreviewWire } from './config/convention'
import {
  conventionManageBusy,
  conventionManageExchangeBusy,
  conventionManageExchangeTier,
  reloadConventionManageAfterImport,
  setConventionExchangeBusy,
} from './conventionManageState'
import type { IpcError } from './i18n'

const overlayOpen = ref(false)
const preview = shallowRef<ConventionImportPreviewWire | null>(null)
const loadError = shallowRef<IpcError | null>(null)
const decisions = ref<Record<string, ConventionImportDecision>>({})
const cursor = ref(0)
const confirming = ref(false)
const confirmError = shallowRef<IpcError | null>(null)

let sequence = 0

export const conventionImportIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const conventionImportPreview: Readonly<Ref<ConventionImportPreviewWire | null>> = preview
export const conventionImportLoadError: Readonly<Ref<IpcError | null>> = loadError
export const conventionImportCursor: DeepReadonly<Ref<number>> = readonly(cursor)
export const conventionImportConfirming: DeepReadonly<Ref<boolean>> = readonly(confirming)
export const conventionImportConfirmError: Readonly<Ref<IpcError | null>> = confirmError

/** The plan is gone or the list moved under it; retrying the same plan can only fail again. */
export const conventionImportBlocked = computed(() => {
  const code = confirmError.value?.code
  return code === 'convention.import_stale' || code === 'convention.no_pending_import'
})

export function conventionImportDecisionFor(ruleKey: string): ConventionImportDecision {
  return decisions.value[ruleKey] ?? 'keep_mine'
}

/** `new` adds the rows the file brings and the conflicts taken from it; `kept` is what stays as it is. */
export const conventionImportSummary = computed(() => {
  const p = preview.value
  if (p === null) return { new_count: '0', kept_count: '0' }
  const taken = Object.keys(decisions.value).length
  return {
    new_count: String(p.new_count + taken),
    kept_count: String(p.identical_count + p.conflicts.length - taken),
  }
})

export function setConventionImportCursor(index: number): void {
  const last = (preview.value?.conflicts.length ?? 0) - 1
  cursor.value = Math.max(0, Math.min(index, last))
}

export function stepConventionImportCursor(delta: 1 | -1): void {
  setConventionImportCursor(cursor.value + delta)
}

export function setConventionImportDecision(ruleKey: string, decision: ConventionImportDecision): void {
  if (confirming.value) return
  const next = { ...decisions.value }
  if (decision === 'keep_mine') delete next[ruleKey]
  else next[ruleKey] = decision
  decisions.value = next
}

/** Applies a decision to the conflict the cursor is on. */
export function decideConventionImportAtCursor(decision: ConventionImportDecision): void {
  const row = preview.value?.conflicts[cursor.value]
  if (row !== undefined) setConventionImportDecision(row.rule_key, decision)
}

/** Opens the pick dialog in Rust. A cancelled dialog opens nothing and says nothing. */
export async function openConventionImportOverlay(): Promise<void> {
  if (conventionManageExchangeBusy.value || conventionManageBusy.value) return
  setConventionExchangeBusy(true)
  sequence += 1
  const mine = sequence
  const result = await openConventionImportPreview(conventionManageExchangeTier.value)
  if (mine !== sequence) return
  setConventionExchangeBusy(false)
  if (result.outcome === 'cancelled') return

  decisions.value = {}
  cursor.value = 0
  confirming.value = false
  confirmError.value = null
  preview.value = null
  loadError.value = null
  if (result.outcome === 'loaded') preview.value = result.preview
  else loadError.value = result.outcome === 'error' ? result.error : UNKNOWN_IPC_ERROR
  overlayOpen.value = true
}

/** A failed confirm keeps the overlay open and the plan pending in Rust. */
export async function confirmConventionImportPreview(): Promise<void> {
  if (confirming.value || preview.value === null || conventionImportBlocked.value) return
  confirming.value = true
  confirmError.value = null
  const mine = sequence
  const result = await confirmConventionImport(decisions.value)
  if (mine !== sequence) return
  confirming.value = false
  if (result.applied === null) {
    confirmError.value = result.error
    return
  }
  overlayOpen.value = false
  await reloadConventionManageAfterImport()
}

export async function cancelConventionImportPreview(): Promise<void> {
  if (confirming.value || !overlayOpen.value) return
  overlayOpen.value = false
  const mine = sequence
  const result = await cancelConventionImport()
  if (mine !== sequence) return
  if (result.error !== null) console.error(`[convention-import] cancel failed: ${JSON.stringify(result.error)}`)
}

export function resetConventionImport(): void {
  sequence += 1
  overlayOpen.value = false
  preview.value = null
  loadError.value = null
  decisions.value = {}
  cursor.value = 0
  confirming.value = false
  confirmError.value = null
}
