import { computed, readonly, ref, shallowRef } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import { replaceConventionChapter, scanConventionChapterWithProgress } from './config/convention'
import type { ConventionReplaceGroupArg, ConventionScanProgress, ConventionScanWire } from './config/convention'
import { applyConventionReplaced } from './conventionState'
import { t } from './i18n'
import type { IpcError } from './i18n'
import { editorSegments, flushEditorBeforeDiscreteWrite } from './panels/editorPanelState'

export type ReplaceSpot = {
  readonly key: string
  readonly segmentId: number
  readonly ruleKey: string
  readonly avoid: string
  readonly replacement: string
  readonly start: number
  readonly end: number
  readonly before: string
  readonly phrase: string
  readonly after: string
}

export type ReplaceGroup = {
  readonly segmentId: number
  readonly ord: number
  readonly confirmed: boolean
  readonly text: string
  readonly spots: readonly ReplaceSpot[]
}

export type ReplaceFilterOption = { readonly ruleKey: string; readonly avoid: string; readonly count: number }

export type ReplaceHiddenCounts = {
  readonly ignored: number
  readonly glossarySuppressed: number
  readonly overlapped: number
  readonly notScanned: number
}

export const REPLACE_SCAN_SLOW_MS = 250
const CONTEXT_UNITS = 16

const NO_HIDDEN: ReplaceHiddenCounts = { ignored: 0, glossarySuppressed: 0, overlapped: 0, notScanned: 0 }

const open = ref(false)
const scanning = ref(false)
const slow = ref(false)
const writing = ref(false)
const hasScanned = ref(false)
const progress = shallowRef<ConventionScanProgress | null>(null)
const failure = shallowRef<IpcError | null>(null)
const alertError = shallowRef<IpcError | null>(null)
const groups = shallowRef<readonly ReplaceGroup[]>([])
const hidden = shallowRef<ReplaceHiddenCounts>(NO_HIDDEN)
const selected = shallowRef<ReadonlySet<string>>(new Set())
const filter = ref<string | null>(null)
const cursor = ref(0)
const armed = ref(false)
const keptNote = ref<number | null>(null)

let session = 0
let scanGeneration = 1_000_000
let slowTimer: ReturnType<typeof setTimeout> | null = null

export const conventionReplaceIsOpen: DeepReadonly<Ref<boolean>> = readonly(open)
export const conventionReplaceScanning: DeepReadonly<Ref<boolean>> = readonly(scanning)
export const conventionReplaceScanSlow: DeepReadonly<Ref<boolean>> = readonly(slow)
export const conventionReplaceWriting: DeepReadonly<Ref<boolean>> = readonly(writing)
export const conventionReplaceHasScanned: DeepReadonly<Ref<boolean>> = readonly(hasScanned)
export const conventionReplaceProgress: Readonly<Ref<ConventionScanProgress | null>> = progress
export const conventionReplaceFailure: Readonly<Ref<IpcError | null>> = failure
export const conventionReplaceAlert: Readonly<Ref<IpcError | null>> = alertError
export const conventionReplaceHidden: Readonly<Ref<ReplaceHiddenCounts>> = hidden
export const conventionReplaceFilter: DeepReadonly<Ref<string | null>> = readonly(filter)
export const conventionReplaceCursor: DeepReadonly<Ref<number>> = readonly(cursor)
export const conventionReplaceArmed: DeepReadonly<Ref<boolean>> = readonly(armed)
export const conventionReplaceKeptNote: DeepReadonly<Ref<number | null>> = readonly(keptNote)

function spotKey(segmentId: number, ruleKey: string, start: number): string {
  return `${String(segmentId)}:${ruleKey}:${String(start)}`
}

function signature(segmentId: number, ruleKey: string, before: string, after: string): string {
  return [String(segmentId), ruleKey, before, after].join('\u0000')
}

function signatureOf(spot: ReplaceSpot, text: string): string {
  return signature(
    spot.segmentId,
    spot.ruleKey,
    text.slice(Math.max(0, spot.start - CONTEXT_UNITS), spot.start),
    text.slice(spot.end, spot.end + CONTEXT_UNITS),
  )
}

function stopSlowTimer(): void {
  if (slowTimer !== null) clearTimeout(slowTimer)
  slowTimer = null
}

function ipcFailure(key: string, retryable = true): IpcError {
  return { code: key, message_key: `err.${key}`, params: {}, retryable }
}

function buildGroups(scan: ConventionScanWire): { groups: ReplaceGroup[]; unjoined: number } {
  const byId = new Map(editorSegments.value.map((segment) => [segment.id, segment] as const))
  const built: ReplaceGroup[] = []
  let unjoined = 0
  for (const entry of scan.segments) {
    const spots: ReplaceSpot[] = []
    for (const span of [...entry.spans].sort((a, b) => a.start - b.start)) {
      if (span.replacement_cased === null) continue
      spots.push({
        key: spotKey(entry.segment_id, span.rule_key, span.start),
        segmentId: entry.segment_id,
        ruleKey: span.rule_key,
        avoid: span.avoid,
        replacement: span.replacement_cased,
        start: span.start,
        end: span.end,
        before: entry.scanned_text.slice(0, span.start),
        phrase: entry.scanned_text.slice(span.start, span.end),
        after: entry.scanned_text.slice(span.end),
      })
    }
    if (spots.length === 0) continue
    const segment = byId.get(entry.segment_id)
    if (segment === undefined) {
      unjoined += 1
      continue
    }
    built.push({
      segmentId: entry.segment_id,
      ord: segment.ord,
      confirmed: segment.status === 'confirmed',
      text: entry.scanned_text,
      spots,
    })
  }
  built.sort((a, b) => a.ord - b.ord)
  return { groups: built, unjoined }
}

/** Flushes the typing buffer, then scans the stored text. A deselection survives when its surrounding text is unchanged. */
async function runScan(keepDeselections: boolean): Promise<void> {
  const mine = session
  const mineScan = ++scanGeneration
  const kept = new Set<string>()
  if (keepDeselections) {
    for (const group of groups.value) {
      for (const spot of group.spots) if (!selected.value.has(spot.key)) kept.add(signatureOf(spot, group.text))
    }
  }
  scanning.value = true
  slow.value = false
  progress.value = null
  failure.value = null
  alertError.value = null
  keptNote.value = null
  armed.value = false
  stopSlowTimer()
  slowTimer = setTimeout(() => {
    slowTimer = null
    if (mine === session && scanning.value) slow.value = true
  }, REPLACE_SCAN_SLOW_MS)
  try {
    const flushed = await flushEditorBeforeDiscreteWrite()
    if (mine !== session) return
    if (flushed !== 'clean') {
      failure.value = ipcFailure('convention.flush_failed')
      return
    }
    const { scan, error } = await scanConventionChapterWithProgress(mineScan, (next) => {
      if (mine === session && mineScan === scanGeneration) progress.value = next
    })
    if (mine !== session || mineScan !== scanGeneration) return
    if (scan === null || scan.generation !== mineScan) {
      failure.value = error ?? ipcFailure('convention.scan_failed')
      return
    }
    const built = buildGroups(scan)
    groups.value = built.groups
    hidden.value = {
      ignored: scan.ignored,
      glossarySuppressed: scan.glossary_suppressed,
      overlapped: scan.overlapped,
      notScanned: scan.not_scanned + built.unjoined,
    }
    const next = new Set<string>()
    let keptCount = 0
    for (const group of built.groups) {
      for (const spot of group.spots) {
        if (kept.has(signatureOf(spot, group.text))) keptCount += 1
        else next.add(spot.key)
      }
    }
    selected.value = next
    if (filter.value !== null && !built.groups.some((g) => g.spots.some((s) => s.ruleKey === filter.value))) {
      filter.value = null
    }
    cursor.value = 0
    hasScanned.value = true
    if (keepDeselections) keptNote.value = keptCount
  } finally {
    if (mine === session && mineScan === scanGeneration) {
      scanning.value = false
      slow.value = false
      stopSlowTimer()
    }
  }
}

/** `ruleKey` opens the filter on that rule; `null` stands on every rule. */
export async function openConventionReplace(ruleKey: string | null): Promise<void> {
  if (open.value) return
  session += 1
  open.value = true
  hasScanned.value = false
  groups.value = []
  hidden.value = NO_HIDDEN
  selected.value = new Set()
  filter.value = ruleKey
  cursor.value = 0
  writing.value = false
  await runScan(false)
}

export async function rescanConventionReplace(): Promise<void> {
  if (!open.value || scanning.value || writing.value) return
  await runScan(true)
}

export function closeConventionReplace(): void {
  session += 1
  scanGeneration += 1
  open.value = false
  scanning.value = false
  slow.value = false
  writing.value = false
  armed.value = false
  failure.value = null
  alertError.value = null
  keptNote.value = null
  progress.value = null
  groups.value = []
  selected.value = new Set()
  hasScanned.value = false
  hidden.value = NO_HIDDEN
  filter.value = null
  cursor.value = 0
  stopSlowTimer()
}

/** Chapter or Work swapped under the overlay. */
export function resetConventionReplace(): void {
  if (open.value) closeConventionReplace()
  stopSlowTimer()
}

export const conventionReplaceOptions: ComputedRef<readonly ReplaceFilterOption[]> = computed(() => {
  const byRule = new Map<string, { avoid: string; count: number }>()
  for (const group of groups.value) {
    for (const spot of group.spots) {
      const entry = byRule.get(spot.ruleKey)
      if (entry === undefined) byRule.set(spot.ruleKey, { avoid: spot.avoid, count: 1 })
      else entry.count += 1
    }
  }
  return [...byRule.entries()]
    .map(([ruleKey, v]) => ({ ruleKey, avoid: v.avoid, count: v.count }))
    .sort((a, b) => a.avoid.localeCompare(b.avoid, 'vi'))
})

export const conventionReplaceVisible: ComputedRef<readonly ReplaceGroup[]> = computed(() => {
  const only = filter.value
  if (only === null) return groups.value
  const shown: ReplaceGroup[] = []
  for (const group of groups.value) {
    const spots = group.spots.filter((spot) => spot.ruleKey === only)
    if (spots.length > 0) shown.push({ ...group, spots })
  }
  return shown
})

const visibleSpots = computed<readonly ReplaceSpot[]>(() => conventionReplaceVisible.value.flatMap((g) => g.spots))

export const conventionReplaceCursorKey: ComputedRef<string | null> = computed(
  () => visibleSpots.value[Math.min(cursor.value, visibleSpots.value.length - 1)]?.key ?? null,
)

export const conventionReplaceTotals: ComputedRef<{ matches: number; sentences: number; selected: number }> = computed(
  () => ({
    matches: groups.value.reduce((n, g) => n + g.spots.length, 0),
    sentences: groups.value.length,
    selected: selected.value.size,
  }),
)

/** Selected draft sentences across every rule, hidden or not. */
export const conventionReplaceDraftSentences: ComputedRef<readonly ReplaceGroup[]> = computed(() =>
  groups.value.filter((g) => !g.confirmed && g.spots.some((s) => selected.value.has(s.key))),
)

export function isConventionReplaceSelected(key: string): boolean {
  return selected.value.has(key)
}

export function setConventionReplaceFilter(ruleKey: string | null): void {
  filter.value = ruleKey
  cursor.value = 0
}

function setSelected(next: Set<string>): void {
  selected.value = next
  armed.value = false
}

export function toggleConventionReplaceSpot(key: string): void {
  if (writing.value) return
  const index = visibleSpots.value.findIndex((s) => s.key === key)
  if (index >= 0) cursor.value = index
  const next = new Set(selected.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  setSelected(next)
}

export function toggleConventionReplaceAtCursor(): void {
  const key = conventionReplaceCursorKey.value
  if (key !== null) toggleConventionReplaceSpot(key)
}

export function moveConventionReplaceCursor(delta: number): void {
  const last = visibleSpots.value.length - 1
  if (last < 0) return
  cursor.value = Math.min(Math.max(cursor.value + delta, 0), last)
}

export function selectConventionReplaceVisible(on: boolean): void {
  if (writing.value || scanning.value) return
  const next = new Set(selected.value)
  for (const spot of visibleSpots.value) {
    if (on) next.add(spot.key)
    else next.delete(spot.key)
  }
  setSelected(next)
}

/** Esc: the first press undoes the pending second press; `false` means nothing was pending. */
export function disarmConventionReplace(): boolean {
  if (!armed.value) return false
  armed.value = false
  return true
}

function changedAlert(ids: readonly number[]): IpcError {
  const ords = new Map(editorSegments.value.map((segment) => [segment.id, segment.ord] as const))
  const list = ids.map((id) => t('convention.replace.sentence_ref', { segment_number: String(ords.get(id) ?? id) }))
  return {
    code: 'convention.chapter_changed',
    message_key: 'convention.replace.changed_alert',
    params: { changed_count: String(ids.length), segment_list: list.join(', ') },
    retryable: true,
  }
}

export function conventionReplaceDraftList(): string {
  return conventionReplaceDraftSentences.value
    .map((g) => t('convention.replace.sentence_ref', { segment_number: String(g.ord) }))
    .join(', ')
}

function pickedGroups(): ConventionReplaceGroupArg[] {
  const picked: ConventionReplaceGroupArg[] = []
  for (const group of groups.value) {
    const spots = group.spots
      .filter((spot) => selected.value.has(spot.key))
      .map((spot) => ({ start: spot.start, end: spot.end, expected_phrase: spot.phrase, rule_key: spot.ruleKey }))
    if (spots.length > 0) picked.push({ segment_id: group.segmentId, expected_text: group.text, spots })
  }
  return picked
}

/** First press arms when a selected sentence is a draft; the next press (or the only press) writes everything in one transaction. */
export async function confirmConventionReplace(): Promise<void> {
  if (!open.value || scanning.value || writing.value || failure.value !== null || alertError.value !== null) return
  if (selected.value.size === 0) return
  if (conventionReplaceDraftSentences.value.length > 0 && !armed.value) {
    armed.value = true
    return
  }
  const mine = session
  const picked = pickedGroups()
  writing.value = true
  armed.value = false
  alertError.value = null
  try {
    const { outcome, error } = await replaceConventionChapter(picked)
    if (outcome !== null) {
      applyConventionReplaced(outcome)
      if (mine === session) closeConventionReplace()
      return
    }
    if (mine !== session || error === null) return
    if (error.code === 'convention.chapter_changed') {
      const ids = error.params.segment_ids
        .split(',')
        .map((part) => Number.parseInt(part.trim(), 10))
        .filter((id) => Number.isFinite(id))
      alertError.value = changedAlert(ids)
    } else {
      alertError.value = error
    }
  } finally {
    if (mine === session) writing.value = false
  }
}
