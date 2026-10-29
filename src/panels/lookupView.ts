import { computed } from 'vue'
import type { SenseRecord, SourceGroup } from '../config/dict'
import { everySourceOffForRoute } from './dictSourcesState'
import { entryIsPinned, pinLabelKey } from './lookupHistoryState'
import {
  currentQuery,
  groupedLookup,
  layersLoaded,
  lookupDisplayable,
  lookupError,
  lookupResolved,
  lookupRoute,
  neverLookedUp,
  notFound,
  queryTooShort,
  sensesByLayer,
  singleCharsOf,
} from './lookupPanelState'
import { sharedReadingText } from './pinyin'

export const lookupGroups = computed<readonly SourceGroup[]>(() => groupedLookup.value?.groups ?? [])

export const lookupHeadReading = computed(() =>
  sharedReadingText(lookupGroups.value.flatMap((g) => g.entries.map((hit) => hit.reading))),
)

export const lookupHeadPinTarget = computed(() => {
  const groups = lookupGroups.value
  return groups.length === 1 && groups[0].entries.length === 1 ? groups[0].entries[0] : null
})

export const lookupHeadPinned = computed(() => {
  const only = lookupHeadPinTarget.value
  return only !== null && entryIsPinned(only.source_code, only.entry_id)
})

export const lookupHeadPinKey = computed(() => pinLabelKey(lookupHeadPinned.value))

export const lookupNotFoundChars = computed(() => singleCharsOf(currentQuery.value ?? ''))

/** Hydrated senses of ONE group, filtered by `entry_id` out of its layer's shared list. */
export function lookupSensesFor(group: SourceGroup): SenseRecord[] {
  const entryIds = new Set(group.entries.map((hit) => hit.entry_id))
  return (sensesByLayer.value[group.layer] ?? []).filter((sense) => entryIds.has(sense.entry_id))
}

/** What the record body shows; the Lookup panel and the reading popup both switch on it. */
export type LookupBody =
  | 'error'
  | 'never'
  | 'no_layers'
  | 'too_short'
  | 'all_sources_off'
  | 'not_found'
  | 'results'
  | 'waiting'

/** `all_sources_off` must win over `not_found`: with every source off the empty result is not a miss. */
export const lookupBody = computed<LookupBody>(() => {
  if (lookupError.value !== null) return 'error'
  if (neverLookedUp.value) return 'never'
  const resolved = lookupResolved.value
  const route = lookupRoute.value
  if (resolved && !layersLoaded.value) return 'no_layers'
  if (resolved && queryTooShort.value) return 'too_short'
  if (resolved && route !== null && everySourceOffForRoute(route)) return 'all_sources_off'
  if (resolved && notFound.value) return 'not_found'
  if (lookupDisplayable.value) return 'results'
  return 'waiting'
})
