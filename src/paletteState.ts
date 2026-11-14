import { computed, nextTick, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { commandRegistry, dispatch } from './commands'
import type { CommandDeps } from './commands'
import { shortcutRows } from './config/shortcutsState'
import { t, tError } from './i18n'
import type { IpcError } from './i18n'
import { openWorkById } from './modes/libraryChapters'
import {
  libraryWorks,
  libraryWorksBusy,
  libraryWorksError,
  libraryWorksHaveLoaded,
  libraryWorksMatched,
  libraryWorksTotal,
  loadWorks,
} from './modes/libraryWorks'
import { setMode } from './modes/modeState'

export type PaletteItem =
  | { kind: 'work'; key: string; workId: string; label: string; doneCount: number | null; chapterCount: number }
  | { kind: 'command'; key: string; commandId: string; label: string; chords: readonly string[] }

export type PaletteWorksState = 'loading' | 'error' | 'ready'

const open = ref(false)
const query = ref('')
const cursor = ref(0)

export const paletteIsOpen: DeepReadonly<Ref<boolean>> = readonly(open)
export const paletteQuery: DeepReadonly<Ref<string>> = readonly(query)
export const paletteCursor: DeepReadonly<Ref<number>> = readonly(cursor)

/** Lower-case, Vietnamese diacritics removed, `đ` as `d`. Filters names and labels only; never stored. */
export function foldVietnamese(text: string): string {
  return text
    .normalize('NFD')
    .replace(/\p{M}+/gu, '')
    .replace(/[đĐ]/g, 'd')
    .toLowerCase()
}

export const paletteWorksState = computed<PaletteWorksState>(() => {
  if (libraryWorksError.value !== null) return 'error'
  return libraryWorksHaveLoaded.value ? 'ready' : 'loading'
})

export const paletteWorksErrorText = computed<string | null>(() => {
  const err = libraryWorksError.value
  return err === null ? null : tError(err as IpcError)
})

/** The Library list is filtered by its own status/genre filters; the palette says so rather than hide Works silently. */
export const paletteWorksAreFiltered = computed<boolean>(
  () => libraryWorksHaveLoaded.value && libraryWorksMatched.value < libraryWorksTotal.value,
)

export const paletteWorkItems = computed<readonly PaletteItem[]>(() => {
  if (paletteWorksState.value !== 'ready') return []
  const needle = foldVietnamese(query.value.trim())
  return libraryWorks.value
    .filter((work) => needle === '' || foldVietnamese(work.name).includes(needle))
    .map((work) => ({
      kind: 'work',
      key: `work:${work.work_id}`,
      workId: work.work_id,
      label: work.name,
      doneCount: work.chapter_done_count,
      chapterCount: work.chapter_count,
    }))
})

export const paletteCommandItems = computed<readonly PaletteItem[]>(() => {
  const needle = foldVietnamese(query.value.trim())
  const marked = new Set(
    commandRegistry
      .list()
      .filter((spec) => spec.palette === true)
      .map((spec) => spec.id),
  )
  return shortcutRows.value
    .filter((row) => marked.has(row.id))
    .map((row): PaletteItem => ({
      kind: 'command',
      key: `command:${row.id}`,
      commandId: row.id,
      label: t(row.labelKey),
      chords: row.display,
    }))
    .filter((item) => needle === '' || foldVietnamese(item.label).includes(needle))
})

export const paletteItems = computed<readonly PaletteItem[]>(() => [...paletteWorkItems.value, ...paletteCommandItems.value])

export function openPalette(): void {
  if (open.value) return
  query.value = ''
  cursor.value = 0
  open.value = true
  if (!libraryWorksHaveLoaded.value && !libraryWorksBusy.value) void loadWorks()
}

export function closePalette(): void {
  open.value = false
}

export function setPaletteQuery(value: string): void {
  query.value = value
  cursor.value = 0
}

export function aimPaletteItem(index: number): void {
  if (index >= 0 && index < paletteItems.value.length) cursor.value = index
}

export function selectNextPaletteItem(): void {
  const count = paletteItems.value.length
  if (count > 0) cursor.value = (cursor.value + 1) % count
}

export function selectPrevPaletteItem(): void {
  const count = paletteItems.value.length
  if (count > 0) cursor.value = (cursor.value - 1 + count) % count
}

/** Closes first so focus returns to where the user stood, then acts on the next tick. */
export async function runSelectedPaletteItem(): Promise<void> {
  if (!open.value) return
  const item = paletteItems.value.at(cursor.value)
  if (item === undefined) return
  closePalette()
  await nextTick()
  if (item.kind === 'command') {
    dispatch(item.commandId)
    return
  }
  if (await openWorkById(item.workId)) setMode('workspace')
}

export function paletteCommandDeps(): Pick<
  CommandDeps,
  'openPalette' | 'closePalette' | 'selectNextPaletteItem' | 'selectPrevPaletteItem' | 'runSelectedPaletteItem'
> {
  return {
    openPalette,
    closePalette,
    selectNextPaletteItem,
    selectPrevPaletteItem,
    runSelectedPaletteItem: () => {
      void runSelectedPaletteItem()
    },
  }
}
