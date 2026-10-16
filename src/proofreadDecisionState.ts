import { computed, readonly, ref, shallowRef } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import { getProofreadDecisionCounts } from './config/proofread'
import type { ProofreadDecisionTally } from './config/proofread'
import type { IpcError } from './i18n'

const tally = shallowRef<ProofreadDecisionTally | null>(null)
const failure = shallowRef<IpcError | null>(null)
const loading = ref(false)
let session = 0

export const proofreadDecisionError: DeepReadonly<Ref<IpcError | null>> = readonly(failure)
export const proofreadDecisionLoading: DeepReadonly<Ref<boolean>> = readonly(loading)

export type ProofreadDecisionRatio =
  | { readonly kind: 'none' }
  | {
      readonly kind: 'counts'
      readonly accepted: number
      readonly notError: number
      readonly skipped: number
      readonly acceptedPercent: number
      readonly notErrorPercent: number
      readonly skippedPercent: number
    }

/** `null` until the read finished: an unread tally is never shown as "no decision yet". */
export const proofreadDecisionRatio: ComputedRef<ProofreadDecisionRatio | null> = computed(() => {
  const value = tally.value
  if (value === null) return null
  const total = value.accepted + value.not_error + value.skipped
  if (total === 0) return { kind: 'none' }
  const [acceptedPercent, notErrorPercent, skippedPercent] = wholePercents(
    [value.accepted, value.not_error, value.skipped],
    total,
  )
  return {
    kind: 'counts',
    accepted: value.accepted,
    notError: value.not_error,
    skipped: value.skipped,
    acceptedPercent,
    notErrorPercent,
    skippedPercent,
  }
})

/** Largest-remainder rounding: the parts always sum to exactly 100. */
export function wholePercents(counts: readonly number[], total: number): number[] {
  const floors = counts.map((count) => Math.floor((count * 100) / total))
  let left = 100 - floors.reduce((sum, part) => sum + part, 0)
  const byRemainder = counts.map((count, index) => ({ index, rest: (count * 100) % total })).sort((a, b) => b.rest - a.rest)
  for (const { index } of byRemainder) {
    if (left === 0) break
    floors[index] = (floors[index] ?? 0) + 1
    left -= 1
  }
  return floors
}

export async function reloadProofreadDecisions(): Promise<void> {
  const mine = ++session
  loading.value = true
  const result = await getProofreadDecisionCounts()
  if (mine !== session) return
  loading.value = false
  if (result.counts === null) {
    tally.value = null
    failure.value = result.error
    return
  }
  failure.value = null
  tally.value = {
    accepted: result.counts.accepted,
    not_error: result.counts.not_error,
    skipped: result.counts.skipped,
  }
}

/** Loads once per Work: a no-op while a read is in flight, the tally is known or the last read failed. */
export function ensureProofreadDecisionsLoaded(): void {
  if (loading.value || tally.value !== null || failure.value !== null) return
  void reloadProofreadDecisions()
}

export function resetProofreadDecisions(): void {
  session += 1
  tally.value = null
  failure.value = null
  loading.value = false
}
