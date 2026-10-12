import { readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import {
  clearProofreadIgnores,
  listProofreadIgnores,
  removeProofreadIgnore,
} from './config/proofread'
import type { ProofreadIgnoreRowWire } from './config/proofread'
import type { IpcError } from './i18n'

const open = ref(false)
const rows = shallowRef<readonly ProofreadIgnoreRowWire[]>([])
const hasLoaded = ref(false)
const busy = ref(false)
const failure = shallowRef<IpcError | null>(null)
const selectedId = ref<number | null>(null)
const clearPending = ref(false)
const cleared = ref<number | null>(null)

let session = 0

export const proofreadMemoryIsOpen: DeepReadonly<Ref<boolean>> = readonly(open)
export const proofreadMemoryRows: Readonly<Ref<readonly ProofreadIgnoreRowWire[]>> = rows
export const proofreadMemoryHasLoaded: DeepReadonly<Ref<boolean>> = readonly(hasLoaded)
export const proofreadMemoryBusy: DeepReadonly<Ref<boolean>> = readonly(busy)
export const proofreadMemoryError: DeepReadonly<Ref<IpcError | null>> = readonly(failure)
export const proofreadMemorySelectedId: DeepReadonly<Ref<number | null>> = readonly(selectedId)
export const proofreadMemoryClearPending: DeepReadonly<Ref<boolean>> = readonly(clearPending)
export const proofreadMemoryCleared: DeepReadonly<Ref<number | null>> = readonly(cleared)

function disarm(): void {
  clearPending.value = false
}

async function load(mine: number): Promise<void> {
  const result = await listProofreadIgnores()
  if (mine !== session) return
  if (result.rows === null) {
    failure.value = result.error
    return
  }
  rows.value = result.rows
  hasLoaded.value = true
  if (selectedId.value !== null && !result.rows.some((row) => row.id === selectedId.value)) selectedId.value = null
}

export async function openProofreadMemory(): Promise<void> {
  if (open.value) return
  session += 1
  const mine = session
  open.value = true
  rows.value = []
  hasLoaded.value = false
  failure.value = null
  selectedId.value = null
  cleared.value = null
  disarm()
  busy.value = true
  await load(mine)
  if (mine === session) busy.value = false
}

export function closeProofreadMemory(): void {
  session += 1
  open.value = false
  busy.value = false
  disarm()
}

export function resetProofreadMemory(): void {
  closeProofreadMemory()
  clearPending.value = false
  rows.value = []
  hasLoaded.value = false
  failure.value = null
  selectedId.value = null
  cleared.value = null
}

export function selectProofreadMemoryRow(id: number): void {
  selectedId.value = id
  disarm()
}

export async function forgetProofreadMemoryRow(): Promise<void> {
  const id = selectedId.value
  if (!open.value || busy.value || id === null) return
  const mine = session
  disarm()
  cleared.value = null
  failure.value = null
  busy.value = true
  const result = await removeProofreadIgnore(id)
  if (mine !== session) return
  if (result.removed === null) {
    failure.value = result.error
    busy.value = false
    return
  }
  await load(mine)
  if (mine === session) busy.value = false
}

/** First call arms, second call clears; selecting a row or forgetting one disarms. */
export async function clearProofreadMemory(): Promise<void> {
  if (!open.value || busy.value || rows.value.length === 0) return
  if (!clearPending.value) {
    clearPending.value = true
    failure.value = null
    cleared.value = null
    return
  }
  const mine = session
  disarm()
  failure.value = null
  busy.value = true
  const result = await clearProofreadIgnores()
  if (mine !== session) return
  if (result.cleared === null) {
    failure.value = result.error
    busy.value = false
    return
  }
  cleared.value = result.cleared
  selectedId.value = null
  await load(mine)
  if (mine === session) busy.value = false
}
