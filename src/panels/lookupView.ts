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

export const lookupHeadReading = computed(() => headReadingOf(lookupGroups.value))

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
  return sensesOfGroup(group, sensesByLayer.value)
}

export function sensesOfGroup(
  group: SourceGroup,
  byLayer: Readonly<Record<string, readonly SenseRecord[]>>,
): SenseRecord[] {
  const entryIds = new Set(group.entries.map((hit) => hit.entry_id))
  return (byLayer[group.layer] ?? []).filter((sense) => entryIds.has(sense.entry_id))
}

export function headReadingOf(groups: readonly SourceGroup[]): string | null {
  return sharedReadingText(groups.flatMap((g) => g.entries.map((hit) => hit.reading)))
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

/** Facts `lookupBodyOf` switches on; `resolved` means answered without error. */
export type LookupBodyFacts = {
  error: boolean
  never: boolean
  resolved: boolean
  layersLoaded: boolean
  tooShort: boolean
  everySourceOff: boolean
  notFound: boolean
  displayable: boolean
}

/** `all_sources_off` must win over `not_found`: with every source off the empty result is not a miss. */
export function lookupBodyOf(f: LookupBodyFacts): LookupBody {
  if (f.error) return 'error'
  if (f.never) return 'never'
  if (f.resolved && !f.layersLoaded) return 'no_layers'
  if (f.resolved && f.tooShort) return 'too_short'
  if (f.resolved && f.everySourceOff) return 'all_sources_off'
  if (f.resolved && f.notFound) return 'not_found'
  if (f.displayable) return 'results'
  return 'waiting'
}

export const lookupBody = computed<LookupBody>(() => {
  const route = lookupRoute.value
  return lookupBodyOf({
    error: lookupError.value !== null,
    never: neverLookedUp.value,
    resolved: lookupResolved.value,
    layersLoaded: layersLoaded.value,
    tooShort: queryTooShort.value,
    everySourceOff: route !== null && everySourceOffForRoute(route),
    notFound: notFound.value,
    displayable: lookupDisplayable.value,
  })
})
