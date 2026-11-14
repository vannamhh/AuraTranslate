import { computed, readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { lookupDictionary, quickLookup } from './config/dict'
import type { LookupResponse, QuickGroup, QuickGroupId, QuickLookupResponse, QuickRow } from './config/dict'
import type { IpcError } from './i18n'
import { everySourceOffForRoute } from './panels/dictSourcesState'
import { notFoundOf, queryTooShortOf } from './panels/lookupPanelState'
import { lookupBodyOf } from './panels/lookupView'
import type { LookupBody } from './panels/lookupView'

export type PaletteLookupRow = { key: string; group: QuickGroupId; row: QuickRow }

/** `unavailable` is a reply that never came (no IPC bridge), never read as "no results". */
export type PaletteLookupStatus = 'idle' | 'loading' | 'error' | 'unavailable' | 'no_layers' | 'no_results' | 'ready'

const active = ref(false)
const response = shallowRef<QuickLookupResponse | null>(null)
const error = shallowRef<IpcError | null>(null)
const pending = ref(false)
const asked = ref(false)
const cursor = ref(0)

const previewHeadword = ref<string | null>(null)
const previewResponse = shallowRef<LookupResponse | null>(null)
const previewError = shallowRef<IpcError | null>(null)
const previewPending = ref(false)

let sequence = 0
let previewSequence = 0

export const paletteLookupActive: DeepReadonly<Ref<boolean>> = readonly(active)
export const paletteLookupPending: DeepReadonly<Ref<boolean>> = readonly(pending)
export const paletteLookupError: DeepReadonly<Ref<IpcError | null>> = readonly(error)
export const paletteLookupCursor: DeepReadonly<Ref<number>> = readonly(cursor)
export const paletteLookupPreviewHeadword: DeepReadonly<Ref<string | null>> = readonly(previewHeadword)
export const paletteLookupPreviewResponse: Readonly<Ref<LookupResponse | null>> = previewResponse
export const paletteLookupPreviewError: DeepReadonly<Ref<IpcError | null>> = readonly(previewError)
export const paletteLookupPreviewPending: DeepReadonly<Ref<boolean>> = readonly(previewPending)

export const paletteLookupQueryTruncated = computed(() => response.value?.query_truncated ?? false)
export const paletteLookupSkipped = computed<readonly string[]>(() => response.value?.skipped ?? [])

/** Every group except a genuinely empty one; an outdated or failed group stays visible by name. */
export const paletteLookupGroups = computed<readonly QuickGroup[]>(() =>
  (response.value?.groups ?? []).filter((group) => group.status !== 'empty'),
)

export const paletteLookupRows = computed<readonly PaletteLookupRow[]>(() =>
  paletteLookupGroups.value.flatMap((group) =>
    group.rows.map((row, i) => ({ key: `${group.id}:${String(i)}`, group: group.id, row })),
  ),
)

export const paletteLookupStatus = computed<PaletteLookupStatus>(() => {
  if (pending.value) return 'loading'
  const res = response.value
  if (res === null) {
    if (error.value !== null) return 'error'
    return asked.value ? 'unavailable' : 'idle'
  }
  if (!res.layers_loaded) return 'no_layers'
  if (paletteLookupGroups.value.length === 0) return 'no_results'
  return 'ready'
})

export const paletteLookupPreviewBody = computed<LookupBody>(() => {
  const grouped = previewResponse.value?.grouped ?? null
  const resolved = !previewPending.value && previewError.value === null && previewResponse.value !== null
  const route = grouped?.route ?? null
  return lookupBodyOf({
    error: previewError.value !== null,
    never: previewHeadword.value === null,
    resolved,
    layersLoaded: grouped?.layers_loaded ?? true,
    tooShort: resolved && grouped !== null && queryTooShortOf(grouped),
    everySourceOff: route !== null && everySourceOffForRoute(route),
    notFound: resolved && grouped !== null && notFoundOf(grouped),
    displayable: previewError.value === null && previewResponse.value !== null,
  })
})

function clearPreview(): void {
  previewSequence += 1
  previewHeadword.value = null
  previewResponse.value = null
  previewError.value = null
  previewPending.value = false
}

function clearResults(): void {
  sequence += 1
  response.value = null
  error.value = null
  pending.value = false
  asked.value = false
  cursor.value = 0
  clearPreview()
}

/** Previews the row under the cursor with the Lookup panel's own command; never records history. */
async function loadPreview(): Promise<void> {
  const mine = ++previewSequence
  const current = paletteLookupRows.value.at(cursor.value)
  previewResponse.value = null
  previewError.value = null
  if (current === undefined) {
    previewHeadword.value = null
    previewPending.value = false
    return
  }
  previewHeadword.value = current.row.headword
  previewPending.value = true
  const { response: result, error: err } = await lookupDictionary(current.row.headword)
  if (mine !== previewSequence) return
  previewPending.value = false
  previewResponse.value = result
  previewError.value = err
}

export function enterPaletteLookup(): void {
  clearResults()
  active.value = true
}

export function leavePaletteLookup(): void {
  clearResults()
  active.value = false
}

/** Runs `raw` (trimmed) as a quick lookup; only the newest reply is applied. An empty query clears. */
export async function queryPaletteLookup(raw: string): Promise<void> {
  const trimmed = raw.trim()
  if (trimmed === '') {
    clearResults()
    return
  }
  const mine = ++sequence
  pending.value = true
  asked.value = true
  const { response: result, error: err } = await quickLookup(trimmed)
  if (mine !== sequence) return
  pending.value = false
  response.value = result
  error.value = err
  cursor.value = 0
  await loadPreview()
}

export function aimPaletteLookupRow(index: number): void {
  if (index < 0 || index >= paletteLookupRows.value.length || index === cursor.value) return
  cursor.value = index
  void loadPreview()
}

export function selectNextPaletteLookupRow(): void {
  const count = paletteLookupRows.value.length
  if (count > 0) aimPaletteLookupRow((cursor.value + 1) % count)
}

export function selectPrevPaletteLookupRow(): void {
  const count = paletteLookupRows.value.length
  if (count > 0) aimPaletteLookupRow((cursor.value - 1 + count) % count)
}

/** The row Enter acts on; `null` while a newer query is in flight, so Enter never opens a stale row. */
export function selectedPaletteLookupHeadword(): string | null {
  if (pending.value) return null
  return paletteLookupRows.value.at(cursor.value)?.row.headword ?? null
}
