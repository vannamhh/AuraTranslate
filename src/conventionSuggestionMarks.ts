import { shallowRef, watch } from 'vue'
import type { Ref } from 'vue'
import { aiTranslateAccumulatedText, aiTranslateRunSegmentId, aiTranslateStateValue } from './aiTranslateState'
import { aiTranslateBatchRows, aiTranslateBatchStateValue } from './aiTranslateBatchState'
import { scanConventionSuggestions } from './config/convention'
import type { ConventionSpanWire, ConventionSuggestionItem } from './config/convention'
import { conventionGeneration } from './conventionState'
import { t } from './i18n'

export type ConventionSuggestionEntry = {
  readonly scannedText: string
  readonly spans: readonly ConventionSpanWire[]
}

export type ConventionSuggestionPart = { readonly text: string; readonly title: string | null }

const single = shallowRef<(ConventionSuggestionEntry & { readonly segmentId: number }) | null>(null)
const batch = shallowRef<ReadonlyMap<number, ConventionSuggestionEntry>>(new Map())

let singleToken = 0
let batchToken = 0

export const conventionSuggestionSingle: Readonly<
  Ref<(ConventionSuggestionEntry & { readonly segmentId: number }) | null>
> = single
export const conventionSuggestionBatch: Readonly<Ref<ReadonlyMap<number, ConventionSuggestionEntry>>> = batch

function entryOf(segment: { segment_id: number; scanned_text: string; spans: ConventionSpanWire[] }): ConventionSuggestionEntry {
  return { scannedText: segment.scanned_text, spans: [...segment.spans].sort((a, b) => a.start - b.start) }
}

function dropSingle(): void {
  singleToken += 1
  if (single.value !== null) single.value = null
}

function dropBatch(): void {
  batchToken += 1
  if (batch.value.size > 0) batch.value = new Map()
}

/** Drops every suggestion mark and invalidates scans still in flight. */
export function resetConventionSuggestions(): void {
  dropSingle()
  dropBatch()
}

async function scanSingle(): Promise<void> {
  const segmentId = aiTranslateRunSegmentId.value
  const text = aiTranslateAccumulatedText.value
  if (segmentId === null || text.trim() === '') return
  const mine = ++singleToken
  const generation = conventionGeneration.value
  const { scan } = await scanConventionSuggestions(generation, [{ segment_id: segmentId, text }])
  if (mine !== singleToken || generation !== conventionGeneration.value) return
  if (scan === null || scan.generation !== generation) return
  const hit = scan.items.find((item) => item.segment_id === segmentId)
  single.value = hit === undefined ? null : { segmentId, ...entryOf(hit) }
}

async function scanBatch(): Promise<void> {
  const items: ConventionSuggestionItem[] = []
  for (const row of aiTranslateBatchRows.value) {
    if (row.status === 'received' && row.text !== '') items.push({ segment_id: row.segmentId, text: row.text })
  }
  if (items.length === 0) return
  const mine = ++batchToken
  const generation = conventionGeneration.value
  const { scan } = await scanConventionSuggestions(generation, items)
  if (mine !== batchToken || generation !== conventionGeneration.value) return
  if (scan === null || scan.generation !== generation) return
  const next = new Map<number, ConventionSuggestionEntry>()
  for (const item of scan.items) next.set(item.segment_id, entryOf(item))
  batch.value = next
}

// Sync flush: the scan must start in the same tick the state turns `done`, and a new run must drop marks before it renders.
watch(
  aiTranslateStateValue,
  (state) => {
    dropSingle()
    if (state === 'done') void scanSingle()
  },
  { flush: 'sync' },
)

watch(
  aiTranslateBatchStateValue,
  (state) => {
    dropBatch()
    if (state === 'done') void scanBatch()
  },
  { flush: 'sync' },
)

watch(conventionGeneration, resetConventionSuggestions, { flush: 'sync' })

function titleOf(span: ConventionSpanWire): string {
  return span.replacement_cased === null
    ? t('convention.mark.title_no_replacement', { avoid_phrase: span.avoid })
    : t('convention.mark.title_with_replacement', {
        avoid_phrase: span.avoid,
        replacement_phrase: span.replacement_cased,
      })
}

/** Splits `text` at the spans of `entry`; no marks unless `entry` was scanned from exactly this text. */
export function conventionSuggestionParts(
  text: string,
  entry: ConventionSuggestionEntry | null | undefined,
): ConventionSuggestionPart[] {
  if (entry === null || entry === undefined || entry.scannedText !== text) return [{ text, title: null }]
  const parts: ConventionSuggestionPart[] = []
  let cursor = 0
  for (const span of entry.spans) {
    if (span.start < cursor || span.end <= span.start || span.end > text.length) continue
    if (span.start > cursor) parts.push({ text: text.slice(cursor, span.start), title: null })
    parts.push({ text: text.slice(span.start, span.end), title: titleOf(span) })
    cursor = span.end
  }
  if (cursor < text.length) parts.push({ text: text.slice(cursor), title: null })
  return parts
}
