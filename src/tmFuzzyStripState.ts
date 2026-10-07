import { computed, readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { tmFuzzyMatches } from './config/segment'
import type { TmExactTarget, TmFuzzyMatch, TmFuzzyTier } from './config/segment'
import type { IpcError } from './i18n'

export type TmFuzzyPendingAccept = {
  segmentId: number
  tier: TmFuzzyTier
  unitId: number
  expectedTarget: string
  draft: string
  kind: 'fuzzy' | 'exact'
}

const shown = shallowRef<{
  segmentId: number
  matches: readonly TmFuzzyMatch[]
  exact: readonly TmExactTarget[]
} | null>(null)
const aimed = ref(0)
const scanError = shallowRef<IpcError | null>(null)
const acceptError = shallowRef<IpcError | null>(null)
const accepting = ref(false)
const pendingAccept = shallowRef<TmFuzzyPendingAccept | null>(null)
const focusRequest = ref(0)

let suppressed = new Set<number>()
let sequence = 0
let savedFocusEl: HTMLElement | null = null
let enteredViaChord = false

export const tmFuzzyMatchesShown = computed<readonly TmFuzzyMatch[]>(() => shown.value?.matches ?? [])
export const tmFuzzyExactShown = computed<readonly TmExactTarget[]>(() => shown.value?.exact ?? [])
export const tmFuzzySegmentId = computed<number | null>(() => shown.value?.segmentId ?? null)
export const tmFuzzyAimedIndex: DeepReadonly<Ref<number>> = readonly(aimed)
export const tmFuzzyScanError: DeepReadonly<Ref<IpcError | null>> = readonly(scanError)
export const tmFuzzyAcceptError: DeepReadonly<Ref<IpcError | null>> = readonly(acceptError)
export const tmFuzzyAccepting: DeepReadonly<Ref<boolean>> = readonly(accepting)
export const tmFuzzyPendingAccept: DeepReadonly<Ref<TmFuzzyPendingAccept | null>> = readonly(pendingAccept)
export const tmFuzzyFocusRequest: DeepReadonly<Ref<number>> = readonly(focusRequest)

/** Eligible to render: matches to show, or a scan error the user must see. */
export const tmFuzzyIsEligible = computed<boolean>(
  () =>
    (shown.value !== null && (shown.value.matches.length > 0 || shown.value.exact.length > 0)) ||
    scanError.value !== null,
)

export function tmFuzzyRowCount(): number {
  const current = shown.value
  if (current === null) return 0
  return current.exact.length > 0 ? current.exact.length : current.matches.length
}

export function tmFuzzyAimedExact(): TmExactTarget | null {
  const list = shown.value?.exact ?? []
  return list[aimed.value] ?? null
}

export function tmFuzzyAimedMatch(): TmFuzzyMatch | null {
  const list = shown.value?.matches ?? []
  return list[aimed.value] ?? null
}

function clearShown(): void {
  shown.value = null
  aimed.value = 0
  scanError.value = null
  acceptError.value = null
  pendingAccept.value = null
}

export const TM_FUZZY_SCAN_DEBOUNCE_MS = 150

let currentSegmentId: number | null = null
let scanTimer: ReturnType<typeof setTimeout> | null = null

function cancelScanTimer(): void {
  if (scanTimer !== null) clearTimeout(scanTimer)
  scanTimer = null
}

async function runScan(segmentId: number, mine: number): Promise<void> {
  const { outcome, error } = await tmFuzzyMatches(segmentId)
  if (mine !== sequence) return
  if (outcome === null) {
    clearShown()
    scanError.value = error
    return
  }
  if (outcome.segment_id !== segmentId) return
  scanError.value = null
  if (outcome.matches.length === 0 && outcome.exact.length === 0) {
    clearShown()
    return
  }
  shown.value = { segmentId, matches: outcome.matches, exact: outcome.exact }
  aimed.value = 0
}

/**
 * Scans for the segment the caret settled on, after a short debounce so holding an arrow key
 * scans only the last segment. A response is applied only when no newer sync or reset
 * happened meanwhile and it names the same segment. Re-syncing the same segment keeps the
 * strip and its focus bookkeeping.
 */
export function syncTmFuzzyStrip(segmentId: number | null): void {
  sequence += 1
  const mine = sequence
  cancelScanTimer()
  if (segmentId !== currentSegmentId) {
    currentSegmentId = segmentId
    clearShown()
    savedFocusEl = null
    enteredViaChord = false
  }
  if (segmentId === null || suppressed.has(segmentId)) return

  scanTimer = setTimeout(() => {
    scanTimer = null
    void runScan(segmentId, mine)
  }, TM_FUZZY_SCAN_DEBOUNCE_MS)
}

export function moveTmFuzzyAim(delta: number): void {
  const count = tmFuzzyRowCount()
  if (count === 0 || pendingAccept.value !== null) return
  aimed.value = Math.min(count - 1, Math.max(0, aimed.value + delta))
}

export function aimTmFuzzyRow(index: number): void {
  const count = tmFuzzyRowCount()
  if (index < 0 || index >= count || pendingAccept.value !== null) return
  aimed.value = index
}

function restoreFocus(): void {
  if (savedFocusEl !== null && savedFocusEl.isConnected) savedFocusEl.focus()
  savedFocusEl = null
  enteredViaChord = false
}

export function focusTmFuzzyStrip(isVisible: boolean): boolean {
  if (!tmFuzzyIsEligible.value) return false
  if (!isVisible) {
    console.error('[tm] tm.fuzzy.focus ran while a higher-priority strip is showing -- ignored')
    return false
  }
  if (!enteredViaChord) {
    const active = document.activeElement
    savedFocusEl = active instanceof HTMLElement ? active : null
    enteredViaChord = true
  }
  focusRequest.value += 1
  return true
}

/** Esc: answers an open overwrite question with no; otherwise hides the strip for this segment this session and returns focus. */
export function hideTmFuzzyStrip(): void {
  if (accepting.value) return
  if (pendingAccept.value !== null) {
    pendingAccept.value = null
    return
  }
  const segmentId = shown.value?.segmentId ?? null
  if (segmentId !== null) suppressed.add(segmentId)
  sequence += 1
  clearShown()
  restoreFocus()
}

/** The accepted segment's strip closes and stays closed for the session. */
export function closeTmFuzzyStripAfterAccept(segmentId: number): void {
  suppressed.add(segmentId)
  if (shown.value?.segmentId !== segmentId) return
  sequence += 1
  clearShown()
  restoreFocus()
}

export function tmFuzzyAcceptingNow(): boolean {
  return accepting.value
}

export function setTmFuzzyAccepting(value: boolean): void {
  accepting.value = value
}

export function setTmFuzzyAcceptError(error: IpcError | null): void {
  acceptError.value = error
}

export function setTmFuzzyPendingAccept(value: TmFuzzyPendingAccept | null): void {
  pendingAccept.value = value
}

/** Keep the draft: drops the overwrite question and leaves the strip as it was. */
export function cancelTmFuzzyOverwrite(): void {
  pendingAccept.value = null
}

/** Work, chapter or segment-set changed: forget everything, including per-segment Esc. */
export function resetTmFuzzyStrip(): void {
  sequence += 1
  cancelScanTimer()
  currentSegmentId = null
  suppressed = new Set()
  clearShown()
  accepting.value = false
  focusRequest.value = 0
  savedFocusEl = null
  enteredViaChord = false
}
