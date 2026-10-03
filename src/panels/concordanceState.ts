import { computed, readonly, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { tmConcordance } from '../config/segment'
import type { TmConcordance } from '../config/segment'
import { UNKNOWN_IPC_ERROR } from '../config/segment'
import type { IpcError } from '../i18n'

export type ConcordanceView =
  | 'not_searched'
  | 'pending'
  | 'error'
  | 'no_glossary_term'
  | 'tm_empty'
  | 'no_hit'
  | 'hits'

export type ConcordanceSourceHighlight = { segmentId: number; start: number; end: number }

const response = shallowRef<TmConcordance | null>(null)
const error = shallowRef<IpcError | null>(null)
const pending = shallowRef(false)
const noGlossarySelection = shallowRef<string | null>(null)
const sourceHighlight = shallowRef<ConcordanceSourceHighlight | null>(null)
const probe = shallowRef<{ query: string; total: number } | null>(null)

let sequence = 0
let probeSequence = 0

export const concordanceResponse: DeepReadonly<Ref<TmConcordance | null>> = readonly(response)
export const concordanceError: DeepReadonly<Ref<IpcError | null>> = readonly(error)
export const concordanceNoGlossarySelection: DeepReadonly<Ref<string | null>> = readonly(noGlossarySelection)
export const concordanceSourceHighlight: DeepReadonly<Ref<ConcordanceSourceHighlight | null>> =
  readonly(sourceHighlight)

/** Which of the seven renderings the tab shows; each is a separate state, never one fallthrough. */
export const concordanceView = computed<ConcordanceView>(() => {
  if (error.value !== null) return 'error'
  if (noGlossarySelection.value !== null) return 'no_glossary_term'
  const r = response.value
  if (r === null) return pending.value ? 'pending' : 'not_searched'
  if (r.tm_empty) return 'tm_empty'
  return r.total === 0 ? 'no_hit' : 'hits'
})

export function setConcordanceSourceHighlight(next: ConcordanceSourceHighlight | null): void {
  sourceHighlight.value = next
}

export async function runConcordance(rawQuery: string): Promise<void> {
  const query = rawQuery.trim()
  if (query === '') {
    showConcordanceNotSearched()
    return
  }
  const mine = ++sequence
  noGlossarySelection.value = null
  response.value = null
  error.value = null
  pending.value = true
  const { outcome, error: err } = await tmConcordance(query)
  if (mine !== sequence) return

  pending.value = false
  if (err !== null) {
    response.value = null
    error.value = err
    return
  }
  if (outcome === null || outcome.query !== query) {
    response.value = null
    error.value = UNKNOWN_IPC_ERROR
    return
  }
  error.value = null
  response.value = outcome
}

export function showConcordanceNoGlossaryTerm(selection: string): void {
  sequence += 1
  pending.value = false
  response.value = null
  error.value = null
  noGlossarySelection.value = selection
}

export function showConcordanceNotSearched(): void {
  sequence += 1
  pending.value = false
  response.value = null
  error.value = null
  noGlossarySelection.value = null
}

/** Background count for the dictionary not-found pointer; never touches the tab's own state. */
export async function probeConcordanceCount(rawQuery: string): Promise<void> {
  const query = rawQuery.trim()
  const mine = ++probeSequence
  probe.value = null
  if (query === '') return
  const { outcome, error: err } = await tmConcordance(query)
  if (mine !== probeSequence || err !== null || outcome === null || outcome.query !== query) return
  probe.value = { query, total: outcome.total }
}

/** Count of hits for `query`, or `null` when unknown or computed for another query. */
export function concordanceProbeTotalFor(query: string | null): number | null {
  const p = probe.value
  if (p === null || query === null || p.query !== query.trim()) return null
  return p.total
}

export function resetConcordance(): void {
  probeSequence += 1
  probe.value = null
  sourceHighlight.value = null
  showConcordanceNotSearched()
}
