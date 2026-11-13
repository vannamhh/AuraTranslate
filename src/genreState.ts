import { readonly, ref } from 'vue'
import { genreCreate, genreDelete, genreList, genreRename } from './config/genre'
import type { GenreWire } from './config/genre'
import type { IpcError } from './i18n'

const genreRows = ref<GenreWire[]>([])
const genresHaveLoaded = ref(false)
const genreActionError = ref<IpcError | null>(null)
const genreBusy = ref(false)
const genreAddDraft = ref('')
const genreSelectedId = ref<string | null>(null)
const genreRenameDraft = ref('')

export const genres = readonly(genreRows)
export const genreListHasLoaded = readonly(genresHaveLoaded)
export const genreError = readonly(genreActionError)
export const genreIsBusy = readonly(genreBusy)
export const genreNewName = readonly(genreAddDraft)
export const genreSelected = readonly(genreSelectedId)

export function setGenreRows(rows: readonly GenreWire[]): void {
  genreRows.value = rows.map((r) => ({ id: r.id, name: r.name }))
  genresHaveLoaded.value = true
}

/** `null` when the id is not a row of the table. */
export function genreNameFor(id: string): string | null {
  return genreRows.value.find((g) => g.id === id)?.name ?? null
}

/** Reloads the list; on failure the previous rows stay and the error is returned. */
export async function loadGenres(): Promise<IpcError | null> {
  const result = await genreList()
  if (result.genres !== null) setGenreRows(result.genres)
  return result.error
}

export async function reloadGenresForSettings(): Promise<void> {
  genreActionError.value = await loadGenres()
}

export function resetGenreEditing(): void {
  genreActionError.value = null
  genreBusy.value = false
  genreAddDraft.value = ''
  genreSelectedId.value = null
  genreRenameDraft.value = ''
}

export function setGenreNewName(value: string): void {
  genreAddDraft.value = value
}

export function selectGenreRow(id: string): void {
  if (genreSelectedId.value === id) return
  genreSelectedId.value = id
  genreRenameDraft.value = genreNameFor(id) ?? ''
}

export function genreRenameValue(id: string): string {
  return genreSelectedId.value === id ? genreRenameDraft.value : (genreNameFor(id) ?? '')
}

export function setGenreRenameValue(id: string, value: string): void {
  selectGenreRow(id)
  genreRenameDraft.value = value
}

export async function addGenre(): Promise<void> {
  if (genreBusy.value) return
  genreBusy.value = true
  genreActionError.value = null
  const result = await genreCreate(genreAddDraft.value)
  if (result.error !== null) {
    genreActionError.value = result.error
  } else if (result.genre !== null) {
    genreAddDraft.value = ''
    genreActionError.value = await loadGenres()
  }
  genreBusy.value = false
}

export async function renameSelectedGenre(): Promise<void> {
  const id = genreSelectedId.value
  if (id === null || genreBusy.value) return
  genreBusy.value = true
  genreActionError.value = null
  const error = await genreRename(id, genreRenameDraft.value)
  if (error !== null) {
    genreActionError.value = error
  } else {
    genreActionError.value = await loadGenres()
    genreRenameDraft.value = genreNameFor(id) ?? ''
  }
  genreBusy.value = false
}

export async function deleteSelectedGenre(): Promise<void> {
  const id = genreSelectedId.value
  if (id === null || genreBusy.value) return
  genreBusy.value = true
  genreActionError.value = null
  const error = await genreDelete(id)
  if (error !== null) {
    genreActionError.value = error
  } else {
    genreSelectedId.value = null
    genreActionError.value = await loadGenres()
  }
  genreBusy.value = false
}
