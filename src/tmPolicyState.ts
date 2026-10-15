/**
 * The open Work's "only my pairs" switch (FR149, AD-57). Rust owns the value and applies it on
 * every read; this file only mirrors it for the overlay. The strips re-scan from the command handler.
 * Do NOT import this file from `src/commands/index.ts`; handlers arrive through `CommandDeps`.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { tmOnlyMineGet, tmOnlyMineSet } from './config/tm'
import type { TmOnlyMineResult } from './config/tm'
import type { IpcError } from './i18n'

const onlyMine = ref<boolean | null>(null)
const workOpen = ref<boolean | null>(null)
const busy = ref(false)
const failure = ref<IpcError | null>(null)
let session = 0

/** `null` until Rust has answered. */
export const tmPolicyOnlyMine: DeepReadonly<Ref<boolean | null>> = readonly(onlyMine)
export const tmPolicyWorkOpen: DeepReadonly<Ref<boolean | null>> = readonly(workOpen)
export const tmPolicyBusy: DeepReadonly<Ref<boolean>> = readonly(busy)
export const tmPolicyError: DeepReadonly<Ref<IpcError | null>> = readonly(failure)

function apply(result: TmOnlyMineResult): void {
  if (result.state === null) {
    failure.value = result.error
    return
  }
  failure.value = null
  onlyMine.value = result.state.only_mine
  workOpen.value = result.state.work_open
}

export async function loadTmPolicy(): Promise<void> {
  const mine = session
  const result = await tmOnlyMineGet()
  if (mine !== session) return
  apply(result)
}

/** `true` when Rust accepted the new value, so the caller re-scans the strips. */
export async function toggleTmOnlyMine(): Promise<boolean> {
  if (busy.value || onlyMine.value === null || workOpen.value !== true) return false
  const mine = session
  busy.value = true
  const result = await tmOnlyMineSet(!onlyMine.value)
  if (mine !== session) return false
  busy.value = false
  apply(result)
  return result.state !== null
}

export function resetTmPolicy(): void {
  session += 1
  onlyMine.value = null
  workOpen.value = null
  busy.value = false
  failure.value = null
}
