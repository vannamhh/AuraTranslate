import { computed, readonly, ref, shallowRef } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import {
  acceptConventionSpan,
  revertConventionSpan,
  scanConventionChapter,
  scanConventionSegments as scanConventionSegmentsCall,
} from './config/convention'
import type { ConventionReplaceOutcome, ConventionSpanWire, ConventionWriteResult } from './config/convention'
import type { IpcError } from './i18n'
import {
  editorCaretSegmentId,
  editorSegmentIsPending,
  editorSegments,
  flushEditorBeforeDiscreteWrite,
  moveEditorCaretTo,
  onEditorFlushed,
  replaceEditorSegment,
} from './panels/editorPanelState'
import {
  focusProofreadFinding,
  proofreadCurrent,
  proofreadMarkPositions,
  suppressProofreadStripFor,
} from './proofreadState'

export type ConventionSegmentResult = {
  readonly scannedText: string
  readonly spans: readonly ConventionSpanWire[]
}

type Span = { readonly start: number; readonly end: number }

type RevertItem = {
  readonly span: ConventionSpanWire
  readonly phrase: string
  readonly original: string
  start: number
  end: number
  text: string
}

export type ConventionNotice = { readonly key: string; readonly params: Readonly<Record<string, string>> }

export type ConventionCounts = {
  readonly ignored: number
  readonly glossarySuppressed: number
  readonly overlapped: number
  readonly notScanned: number
}

export const CONVENTION_NOTICE_MS = 5000

const EMPTY_COUNTS: ConventionCounts = { ignored: 0, glossarySuppressed: 0, overlapped: 0, notScanned: 0 }

const results = shallowRef<ReadonlyMap<number, ConventionSegmentResult>>(new Map())
const counts = shallowRef<ConventionCounts>(EMPTY_COUNTS)
const scanning = ref(false)
const scanFailed = ref(false)
const opened = shallowRef<{ segmentId: number; index: number } | null>(null)
const reverts = shallowRef<ReadonlyMap<number, readonly RevertItem[]>>(new Map())
const acting = ref(false)
const actionFailure = shallowRef<{ segmentId: number; error: IpcError } | null>(null)
const notice = shallowRef<ConventionNotice | null>(null)

let generation = 0

// Ids a flush rescan wrote while the newest whole-Chapter scan was still in flight. That scan read
// the disk BEFORE the flush, so its entry for such an id is older and must not replace the fresher one.
const mergedSinceScan = new Set<number>()
const generationNow = ref(0)
let noticeTimer: ReturnType<typeof setTimeout> | null = null

export const conventionResults: Readonly<Ref<ReadonlyMap<number, ConventionSegmentResult>>> = results
export const conventionCounts: Readonly<Ref<ConventionCounts>> = counts
export const conventionScanning: DeepReadonly<Ref<boolean>> = readonly(scanning)
export const conventionScanFailed: DeepReadonly<Ref<boolean>> = readonly(scanFailed)
export const conventionBusy: DeepReadonly<Ref<boolean>> = readonly(acting)
/** Bumps on every Chapter, Work or rule-list change; a result carrying an older value is dropped. */
export const conventionGeneration: DeepReadonly<Ref<number>> = readonly(generationNow)
export const conventionNotice: Readonly<Ref<ConventionNotice | null>> = notice

function say(key: string, params: Record<string, string> = {}): void {
  if (noticeTimer !== null) clearTimeout(noticeTimer)
  notice.value = { key, params }
  noticeTimer = setTimeout(() => {
    noticeTimer = null
    notice.value = null
  }, CONVENTION_NOTICE_MS)
}

function stopSaying(): void {
  if (noticeTimer !== null) clearTimeout(noticeTimer)
  noticeTimer = null
  notice.value = null
}

/** Drops every mark, tray and pending result; the next scan carries a newer generation. */
export function resetConvention(): void {
  generation += 1
  generationNow.value = generation
  results.value = new Map()
  counts.value = EMPTY_COUNTS
  scanning.value = false
  scanFailed.value = false
  opened.value = null
  reverts.value = new Map()
  acting.value = false
  actionFailure.value = null
  mergedSinceScan.clear()
  stopSaying()
}

/** Scans the stored text of the open Chapter. A result older than the newest request is dropped. */
export async function scanConventionNow(): Promise<void> {
  const mine = ++generation
  generationNow.value = mine
  mergedSinceScan.clear()
  scanning.value = true
  const { scan, error } = await scanConventionChapter(mine)
  if (mine !== generation) return
  scanning.value = false
  if (error !== null) {
    scanFailed.value = true
    say('convention.status.scan_failed')
    return
  }
  if (scan === null || scan.generation !== mine) return
  scanFailed.value = false
  const next = new Map<number, ConventionSegmentResult>()
  for (const segment of scan.segments) {
    next.set(segment.segment_id, {
      scannedText: segment.scanned_text,
      spans: [...segment.spans].sort((a, b) => a.start - b.start),
    })
  }
  for (const id of mergedSinceScan) {
    const fresher = results.value.get(id)
    if (fresher === undefined) next.delete(id)
    else next.set(id, fresher)
  }
  results.value = next
  counts.value = {
    ignored: scan.ignored,
    glossarySuppressed: scan.glossary_suppressed,
    overlapped: scan.overlapped,
    notScanned: scan.not_scanned,
  }
  const current = opened.value
  if (current !== null && !next.has(current.segmentId)) opened.value = null
}

/** Rescans exactly `segmentIds` after a flush and merges the outcome into the current results; other sentences keep their entries. */
export async function scanConventionSegments(segmentIds: readonly number[]): Promise<void> {
  const wanted = [...new Set(segmentIds)]
  if (wanted.length === 0) return
  const mine = generation
  const { scan, error } = await scanConventionSegmentsCall(mine, wanted)
  if (mine !== generation) return
  if (error !== null) {
    say('convention.status.scan_failed')
    return
  }
  if (scan === null || scan.generation !== mine) return
  const next = new Map(results.value)
  for (const id of wanted) next.delete(id)
  for (const segment of scan.segments) {
    next.set(segment.segment_id, {
      scannedText: segment.scanned_text,
      spans: [...segment.spans].sort((a, b) => a.start - b.start),
    })
  }
  results.value = next
  // `counts` is a whole-Chapter total; a scan of a few sentences cannot restate it, so it stays put.
  for (const id of wanted) mergedSinceScan.add(id)
  const current = opened.value
  if (current !== null && wanted.includes(current.segmentId) && !next.has(current.segmentId)) opened.value = null
}

// Registered at module load, not inside a scan: a flush can land before the first Chapter scan,
// and a follower attached later would miss it with no sign that it did.
onEditorFlushed((segmentIds) => {
  void scanConventionSegments(segmentIds)
})

/** A typed edit in a sentence kills its marks and its undo tray; other sentences are untouched. */
export function clearConventionFor(segmentId: number): void {
  if (reverts.value.has(segmentId)) {
    const nextReverts = new Map(reverts.value)
    nextReverts.delete(segmentId)
    reverts.value = nextReverts
  }
  if (opened.value?.segmentId === segmentId) opened.value = null
  if (!results.value.has(segmentId)) return
  const next = new Map(results.value)
  next.delete(segmentId)
  results.value = next
}

function editorTextOf(segmentId: number): string | null {
  return editorSegments.value.find((segment) => segment.id === segmentId)?.target_text ?? null
}

function liveEntry(segmentId: number): ConventionSegmentResult | null {
  const entry = results.value.get(segmentId)
  if (entry === undefined) return null
  return editorTextOf(segmentId) === entry.scannedText ? entry : null
}

export type ConventionCurrent = {
  readonly segmentId: number
  readonly index: number
  readonly count: number
  readonly span: ConventionSpanWire
  readonly scannedText: string
}

/** The spot the strip shows, only while the caret sentence still holds exactly the scanned text. */
export const conventionCurrent: ComputedRef<ConventionCurrent | null> = computed(() => {
  const at = opened.value
  const caret = editorCaretSegmentId.value
  if (at === null || caret === null || at.segmentId !== caret) return null
  const entry = liveEntry(caret)
  if (entry === null || entry.spans.length === 0) return null
  const index = Math.min(Math.max(at.index, 0), entry.spans.length - 1)
  return {
    segmentId: caret,
    index,
    count: entry.spans.length,
    span: entry.spans[index] as ConventionSpanWire,
    scannedText: entry.scannedText,
  }
})

/** Spots with a replacement that carry the viewed spot's rule across the scanned Chapter; `null` when the rule cannot replace. */
export const conventionViewedRuleCount: ComputedRef<number | null> = computed(() => {
  const current = conventionCurrent.value
  if (current === null || current.span.replacement_cased === null) return null
  let count = 0
  for (const entry of results.value.values()) {
    for (const span of entry.spans) {
      if (span.rule_key === current.span.rule_key && span.replacement_cased !== null) count += 1
    }
  }
  return count
})

export function viewedConventionRuleKey(): string | null {
  return conventionCurrent.value?.span.rule_key ?? null
}

function topRevert(segmentId: number): RevertItem | null {
  const stack = reverts.value.get(segmentId)
  const latest = stack?.[stack.length - 1]
  if (latest === undefined) return null
  return editorTextOf(segmentId) === latest.text ? latest : null
}

export const conventionRevertible: ComputedRef<boolean> = computed(() => {
  const at = opened.value
  const caret = editorCaretSegmentId.value
  return at !== null && caret !== null && at.segmentId === caret && topRevert(caret) !== null
})

/** The revert note is about a confirmed sentence when the tray's text sits on one. */
export const conventionRevertConfirmed: ComputedRef<boolean> = computed(() => {
  const caret = editorCaretSegmentId.value
  if (caret === null || topRevert(caret) === null) return false
  return editorSegments.value.find((segment) => segment.id === caret)?.status === 'confirmed'
})

export const conventionActionError: ComputedRef<IpcError | null> = computed(() => {
  const failure = actionFailure.value
  return failure !== null && failure.segmentId === editorCaretSegmentId.value ? failure.error : null
})

/** The strip is open for the caret sentence and has a spot, a revert note or an error to show. */
export const conventionStripIsEligible: ComputedRef<boolean> = computed(() => {
  const at = opened.value
  const caret = editorCaretSegmentId.value
  if (at === null || caret === null || at.segmentId !== caret) return false
  return conventionCurrent.value !== null || conventionRevertible.value || conventionActionError.value !== null
})

/** Closes the strip when the caret moved to another sentence. */
export function syncConventionStrip(caretSegmentId: number | null): void {
  const at = opened.value
  if (at !== null && caretSegmentId !== null && at.segmentId !== caretSegmentId) opened.value = null
}

export function openConventionAt(segmentId: number, start: number): void {
  const entry = liveEntry(segmentId)
  if (entry === null) return
  const index = entry.spans.findIndex((span) => span.start === start)
  if (index < 0) return
  if (segmentId !== editorCaretSegmentId.value && !moveEditorCaretTo(segmentId)) return
  suppressProofreadStripFor(segmentId)
  actionFailure.value = null
  stopSaying()
  opened.value = { segmentId, index }
}

/** Esc and Ẩn: focus goes back to the caret cell first so the caret survives the strip leaving. */
export function hideConventionStrip(): void {
  const at = opened.value
  if (at === null) return
  const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${String(at.segmentId)}"]`)
  if (cell instanceof HTMLElement && document.activeElement?.closest('.proofreader-strip')) cell.focus()
  opened.value = null
}

function flushFailure(flushed: 'failed' | 'still-dirty'): IpcError {
  const key = flushed === 'failed' ? 'proofread.accept_flush_failed' : 'proofread.accept_flush_still_dirty'
  return { code: key, message_key: key, params: {}, retryable: true }
}

/** `replaced` is the swapped span of the old text; `null` means a span overlaps it and must be dropped. */
function shiftSpan(span: Span, replaced: Span, delta: number): Span | null {
  if (span.end <= replaced.start) return span
  if (span.start >= replaced.end) return { start: span.start + delta, end: span.end + delta }
  return null
}

function setEntry(segmentId: number, entry: ConventionSegmentResult): void {
  const next = new Map(results.value)
  next.set(segmentId, entry)
  results.value = next
}

function shiftTray(segmentId: number, replaced: Span, delta: number, newText: string, drop: RevertItem | null): void {
  const stack = reverts.value.get(segmentId)
  if (stack === undefined) return
  const kept: RevertItem[] = []
  for (const item of stack) {
    if (item === drop) continue
    const moved = shiftSpan(item, replaced, delta)
    if (moved !== null) kept.push({ ...item, start: moved.start, end: moved.end, text: newText })
  }
  const next = new Map(reverts.value)
  if (kept.length === 0) next.delete(segmentId)
  else next.set(segmentId, kept)
  reverts.value = next
}

function mirror(outcome: NonNullable<ConventionWriteResult['outcome']>): void {
  replaceEditorSegment(outcome.segment_id, {
    target_text: outcome.target_text,
    translation_origin: outcome.translation_origin,
    status: outcome.status,
  })
}

/** Mirrors a committed Chapter replace into the Editor, drops marks and trays of those sentences, rescans the rest. */
export function applyConventionReplaced(summary: ConventionReplaceOutcome): void {
  for (const outcome of summary.outcomes) {
    replaceEditorSegment(outcome.segment_id, {
      target_text: outcome.target_text,
      translation_origin: outcome.translation_origin,
      status: outcome.status,
    })
    clearConventionFor(outcome.segment_id)
  }
  say('convention.status.replaced', {
    replaced_count: String(summary.replaced),
    sentence_count: String(summary.sentences),
    confirmed_count: String(summary.confirmed),
    draft_count: String(summary.drafts),
  })
  void scanConventionNow()
}

export async function acceptConventionSpot(): Promise<void> {
  if (acting.value) return
  const current = conventionCurrent.value
  if (current === null || current.span.replacement_cased === null) return
  const mine = generation
  const { segmentId, span, scannedText } = current
  acting.value = true
  actionFailure.value = null
  try {
    const flushed = await flushEditorBeforeDiscreteWrite()
    if (flushed !== 'clean') {
      actionFailure.value = { segmentId, error: flushFailure(flushed) }
      return
    }
    if (mine !== generation || !(results.value.get(segmentId)?.spans.includes(span) ?? false)) return
    const result = await acceptConventionSpan({
      segmentId,
      expectedText: scannedText,
      start: span.start,
      end: span.end,
      expectedPhrase: scannedText.slice(span.start, span.end),
      ruleKey: span.rule_key,
    })
    if (result.outcome === null) {
      if (result.error !== null) actionFailure.value = { segmentId, error: result.error }
      return
    }
    const outcome = result.outcome
    mirror(outcome)
    if (mine !== generation) return
    const replaced = { start: span.start, end: span.end }
    const delta = outcome.end - outcome.start - (span.end - span.start)
    const entry = results.value.get(segmentId)
    const spans: ConventionSpanWire[] = []
    for (const other of entry?.spans ?? []) {
      if (other === span) continue
      const moved = shiftSpan(other, replaced, delta)
      if (moved !== null) spans.push({ ...other, start: moved.start, end: moved.end })
    }
    setEntry(segmentId, { scannedText: outcome.target_text, spans })
    shiftTray(segmentId, replaced, delta, outcome.target_text, null)
    const next = new Map(reverts.value)
    next.set(segmentId, [
      ...(next.get(segmentId) ?? []),
      {
        span,
        phrase: outcome.phrase,
        original: scannedText.slice(span.start, span.end),
        start: outcome.start,
        end: outcome.end,
        text: outcome.target_text,
      },
    ])
    reverts.value = next
    const at = opened.value
    if (at !== null && at.segmentId === segmentId) {
      opened.value = { segmentId, index: Math.max(0, Math.min(at.index, spans.length - 1)) }
    }
    say('convention.status.accepted', {
      from_phrase: scannedText.slice(span.start, span.end),
      to_phrase: outcome.phrase,
    })
  } finally {
    acting.value = false
  }
}

export async function revertConventionSpot(): Promise<void> {
  if (acting.value) return
  const caret = editorCaretSegmentId.value
  if (caret === null) return
  const item = topRevert(caret)
  if (item === null) return
  const mine = generation
  acting.value = true
  actionFailure.value = null
  try {
    const flushed = await flushEditorBeforeDiscreteWrite()
    if (flushed !== 'clean') {
      actionFailure.value = { segmentId: caret, error: flushFailure(flushed) }
      return
    }
    const live = topRevert(caret)
    if (live === null) return
    const result = await revertConventionSpan({
      segmentId: caret,
      expectedText: live.text,
      start: live.start,
      end: live.end,
      expectedPhrase: live.phrase,
      originalPhrase: live.original,
    })
    if (result.outcome === null) {
      if (result.error !== null) actionFailure.value = { segmentId: caret, error: result.error }
      return
    }
    const outcome = result.outcome
    mirror(outcome)
    if (mine !== generation) return
    const replaced = { start: live.start, end: live.end }
    const delta = outcome.end - outcome.start - (live.end - live.start)
    const entry = results.value.get(caret)
    const spans: ConventionSpanWire[] = []
    for (const other of entry?.spans ?? []) {
      const moved = shiftSpan(other, replaced, delta)
      if (moved !== null) spans.push({ ...other, start: moved.start, end: moved.end })
    }
    const restored: ConventionSpanWire = { ...live.span, start: outcome.start, end: outcome.end }
    spans.push(restored)
    spans.sort((a, b) => a.start - b.start)
    setEntry(caret, { scannedText: outcome.target_text, spans })
    shiftTray(caret, replaced, delta, outcome.target_text, live)
    opened.value = { segmentId: caret, index: spans.indexOf(restored) }
  } finally {
    acting.value = false
  }
}

export type MarkFamily = 'proofread' | 'convention'

type MarkPosition = {
  readonly family: MarkFamily
  readonly segmentId: number
  readonly index: number
  readonly start: number
  readonly end: number
}

/** UTF-16 offset of the caret inside its translation cell, or `null` when the selection is not there. */
function caretOffsetIn(segmentId: number): number | null {
  const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${String(segmentId)}"]`)
  const selection = window.getSelection()
  if (cell === null || selection === null || selection.focusNode === null || !cell.contains(selection.focusNode)) return null
  const range = document.createRange()
  range.setStart(cell, 0)
  range.setEnd(selection.focusNode, selection.focusOffset)
  return range.toString().length
}

/** Proofreader findings and convention spots of the Chapter in text order; a tie goes to the Proofreader. */
function markPositions(): MarkPosition[] {
  const list: MarkPosition[] = proofreadMarkPositions().map((p) => ({ ...p, family: 'proofread' as const }))
  for (const segment of editorSegments.value) {
    const entry = liveEntry(segment.id)
    if (entry === null) continue
    entry.spans.forEach((span, index) =>
      list.push({ family: 'convention', segmentId: segment.id, index, start: span.start, end: span.end }),
    )
  }
  const order = new Map(editorSegments.value.map((segment, at) => [segment.id, at] as const))
  return list.sort(
    (a, b) =>
      (order.get(a.segmentId) ?? 0) - (order.get(b.segmentId) ?? 0) ||
      a.start - b.start ||
      (a.family === b.family ? 0 : a.family === 'proofread' ? -1 : 1),
  )
}

function currentMark(): { family: MarkFamily; segmentId: number; index: number } | null {
  const spot = conventionCurrent.value
  if (spot !== null) return { family: 'convention', segmentId: spot.segmentId, index: spot.index }
  const finding = proofreadCurrent.value
  if (finding !== null) return { family: 'proofread', segmentId: finding.segmentId, index: finding.index }
  return null
}

function land(target: MarkPosition): void {
  if (target.segmentId !== editorCaretSegmentId.value && !moveEditorCaretTo(target.segmentId)) return
  stopSaying()
  if (target.family === 'proofread') {
    opened.value = null
    focusProofreadFinding(target.segmentId, target.index)
    return
  }
  suppressProofreadStripFor(target.segmentId)
  actionFailure.value = null
  opened.value = { segmentId: target.segmentId, index: target.index }
}

function caretSentenceIsPending(): boolean {
  const caret = editorCaretSegmentId.value
  return caret !== null && editorSegmentIsPending(caret)
}

/** F8 / Shift+F8: the next or previous mark of either family; the end stops with a status message, no wrap. */
export function stepMarks(direction: 1 | -1): void {
  const list = markPositions()
  if (list.length === 0) {
    say(caretSentenceIsPending() ? 'convention.status.sentence_pending' : 'convention.status.no_marks')
    return
  }
  const order = editorSegments.value.map((segment) => segment.id)
  const current = currentMark()
  let target: MarkPosition | undefined
  const at =
    current === null
      ? -1
      : list.findIndex((m) => m.family === current.family && m.segmentId === current.segmentId && m.index === current.index)
  if (at >= 0) {
    target = list[at + direction]
  } else {
    const caret = editorCaretSegmentId.value
    const caretAt = caret === null ? -1 : order.indexOf(caret)
    const inCaret = list.filter((m) => m.segmentId === caret)
    const offset = caret === null ? null : caretOffsetIn(caret)
    if (inCaret.length > 0 && offset === null) target = direction === 1 ? inCaret[0] : inCaret[inCaret.length - 1]
    else if (direction === 1) {
      target = inCaret.find((m) => m.start >= (offset ?? 0)) ?? list.find((m) => order.indexOf(m.segmentId) > caretAt)
    } else {
      target =
        [...inCaret].reverse().find((m) => m.end <= (offset ?? 0)) ??
        [...list].reverse().find((m) => order.indexOf(m.segmentId) < caretAt)
    }
  }
  if (target === undefined) {
    say(
      caretSentenceIsPending()
        ? 'convention.status.sentence_pending'
        : direction === 1
          ? 'convention.status.no_next'
          : 'convention.status.no_prev',
    )
    return
  }
  land(target)
}

/** "Bỏ qua": the webview only moves to the next mark; nothing is stored and the mark stays drawn. */
export function skipConventionSpot(): void {
  if (conventionCurrent.value === null) return
  stepMarks(1)
}
