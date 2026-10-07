/**
 * State of the export screen (FR89): which Chapters go out, the counts Rust computed for that
 * scope, and the destination folder. Counting is Rust's; this file only holds UI state. Do NOT
 * import this file from `src/commands/index.ts`; handlers are injected through `CommandDeps`
 * (`exportCommandDeps.ts`).
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { listChapters } from './config/chapter'
import type { ChapterRow } from './config/chapter'
import { exportChooseFolder, exportDocxTwoColumn, exportScopeSummary } from './config/export'
import type { ExportScope, ExportScopeCounts, ExportedFile } from './config/export'
import type { IpcError } from './i18n'

export type ExportFormat = 'docx_two_column'
export type ExportRunStatus = 'idle' | 'running' | 'done' | 'error' | 'ipc_unavailable'
export type ExportScopeKind = 'chapter' | 'chapters' | 'work'
export type ExportLoadStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'loaded'
export type ExportCountsStatus = 'unknown' | 'none_selected' | 'ipc_unavailable' | 'error' | 'loaded'

const overlayOpen = ref(false)
const loadStatus = ref<ExportLoadStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const chapters = ref<ChapterRow[]>([])
const scopeKind = ref<ExportScopeKind>('work')
const singleChapterId = ref<number | null>(null)
const selectedChapterIds = ref<number[]>([])
const counts = ref<ExportScopeCounts | null>(null)
const countsStatus = ref<ExportCountsStatus>('unknown')
const countsError = ref<IpcError | null>(null)
const folder = ref<string | null>(null)
const folderError = ref<IpcError | null>(null)
const folderUnavailable = ref(false)
const choosingFolder = ref(false)
const format = ref<ExportFormat>('docx_two_column')
const runStatus = ref<ExportRunStatus>('idle')
const runResult = ref<ExportedFile | null>(null)
const runError = ref<IpcError | null>(null)
let sequence = 0

export const exportOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const exportLoadStatus: DeepReadonly<Ref<ExportLoadStatus>> = readonly(loadStatus)
export const exportLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const exportChapters: DeepReadonly<Ref<ChapterRow[]>> = readonly(chapters)
export const exportScopeKind: DeepReadonly<Ref<ExportScopeKind>> = readonly(scopeKind)
export const exportSingleChapterId: DeepReadonly<Ref<number | null>> = readonly(singleChapterId)
export const exportSelectedChapterIds: DeepReadonly<Ref<number[]>> = readonly(selectedChapterIds)
export const exportCounts: DeepReadonly<Ref<ExportScopeCounts | null>> = readonly(counts)
export const exportCountsStatus: DeepReadonly<Ref<ExportCountsStatus>> = readonly(countsStatus)
export const exportCountsError: DeepReadonly<Ref<IpcError | null>> = readonly(countsError)
export const exportFolder: DeepReadonly<Ref<string | null>> = readonly(folder)
export const exportFolderError: DeepReadonly<Ref<IpcError | null>> = readonly(folderError)
export const exportFolderUnavailable: DeepReadonly<Ref<boolean>> = readonly(folderUnavailable)
export const exportChoosingFolder: DeepReadonly<Ref<boolean>> = readonly(choosingFolder)
export const exportFormat: DeepReadonly<Ref<ExportFormat>> = readonly(format)
export const exportRunStatus: DeepReadonly<Ref<ExportRunStatus>> = readonly(runStatus)
export const exportRunResult: DeepReadonly<Ref<ExportedFile | null>> = readonly(runResult)
export const exportRunError: DeepReadonly<Ref<IpcError | null>> = readonly(runError)

function clearRun(): void {
  runStatus.value = 'idle'
  runResult.value = null
  runError.value = null
}

/** The scope a later export command sends to Rust; `null` while nothing is selected. */
export function currentExportScope(): ExportScope | null {
  if (scopeKind.value === 'work') return { kind: 'work' }
  if (scopeKind.value === 'chapter') {
    return singleChapterId.value === null ? null : { kind: 'chapters', chapter_ids: [singleChapterId.value] }
  }
  return selectedChapterIds.value.length === 0 ? null : { kind: 'chapters', chapter_ids: [...selectedChapterIds.value] }
}

async function refreshCounts(): Promise<void> {
  sequence += 1
  const mySequence = sequence
  clearRun()
  const scope = currentExportScope()
  if (scope === null) {
    counts.value = null
    countsError.value = null
    countsStatus.value = 'none_selected'
    return
  }

  const result = await exportScopeSummary(scope)
  if (mySequence !== sequence) return
  if (result.counts !== null) {
    counts.value = result.counts
    countsError.value = null
    countsStatus.value = 'loaded'
    return
  }
  counts.value = null
  countsError.value = result.error
  countsStatus.value = result.error === null ? 'ipc_unavailable' : 'error'
}

export async function openExport(): Promise<void> {
  if (overlayOpen.value) return

  sequence += 1
  const mySequence = sequence
  overlayOpen.value = true
  loadStatus.value = 'unknown'
  loadError.value = null
  counts.value = null
  countsStatus.value = 'unknown'
  countsError.value = null
  folderError.value = null
  folderUnavailable.value = false

  const result = await listChapters()
  if (mySequence !== sequence) return
  if (result.chapters === null) {
    chapters.value = []
    loadError.value = result.error
    loadStatus.value = result.error === null ? 'ipc_unavailable' : 'error'
    return
  }

  chapters.value = result.chapters
  loadStatus.value = 'loaded'
  const known = new Set(result.chapters.map((c) => c.chapter_id))
  if (singleChapterId.value === null || !known.has(singleChapterId.value)) {
    singleChapterId.value = result.chapters[0]?.chapter_id ?? null
  }
  selectedChapterIds.value = selectedChapterIds.value.filter((id) => known.has(id))
  await refreshCounts()
}

export function setExportFormat(next: ExportFormat): void {
  format.value = next
  clearRun()
}

export function closeExport(): void {
  if (!overlayOpen.value) return
  sequence += 1
  overlayOpen.value = false
  choosingFolder.value = false
}

export async function setExportScopeKind(kind: ExportScopeKind): Promise<void> {
  scopeKind.value = kind
  await refreshCounts()
}

export async function selectExportSingleChapter(chapterId: number): Promise<void> {
  singleChapterId.value = chapterId
  await refreshCounts()
}

export async function toggleExportChapter(chapterId: number): Promise<void> {
  const current = selectedChapterIds.value
  selectedChapterIds.value = current.includes(chapterId)
    ? current.filter((id) => id !== chapterId)
    : [...current, chapterId]
  await refreshCounts()
}

export async function chooseExportFolder(): Promise<void> {
  if (choosingFolder.value) return

  choosingFolder.value = true
  folderError.value = null
  folderUnavailable.value = false
  const result = await exportChooseFolder()
  choosingFolder.value = false

  if (result.outcome === 'picked') {
    folder.value = result.path
    clearRun()
  }
  else if (result.outcome === 'error') folderError.value = result.error
  else if (result.outcome === 'ipc_unavailable') folderUnavailable.value = true
}

/** Writes the selected format for the current scope into the chosen folder. */
export async function runExport(): Promise<void> {
  const scope = currentExportScope()
  const target = folder.value
  if (runStatus.value === 'running' || scope === null || target === null) return

  runStatus.value = 'running'
  runResult.value = null
  runError.value = null
  const result = await exportDocxTwoColumn(scope, target)
  if (result.file !== null) {
    runResult.value = result.file
    runStatus.value = 'done'
    return
  }
  runError.value = result.error
  runStatus.value = result.error === null ? 'ipc_unavailable' : 'error'
}

export function resetExport(): void {
  sequence += 1
  overlayOpen.value = false
  loadStatus.value = 'unknown'
  loadError.value = null
  chapters.value = []
  scopeKind.value = 'work'
  singleChapterId.value = null
  selectedChapterIds.value = []
  counts.value = null
  countsStatus.value = 'unknown'
  countsError.value = null
  folder.value = null
  folderError.value = null
  folderUnavailable.value = false
  choosingFolder.value = false
  format.value = 'docx_two_column'
  clearRun()
}
