import { currentMode, setMode } from './modeState'
import {
  destinationWorks,
  destinationWorksError,
  destinationWorksHaveLoaded,
  loadDestinationWorks,
  presetImportDestination,
  resetPresetImportDestination,
  selectImportTab,
} from './libraryImport'
import { openLibraryImportOverlay } from './libraryImportOverlayState'
import { libraryWorkPageWork } from './libraryChapters'
import { currentLibraryWork } from './libraryWorks'
import { openWorkId } from '../openWorkIdentity'

export function openLibraryImportForm(): void {
  try {
    resetPresetImportDestination()
    setMode('library')
    openLibraryImportOverlay()
  } catch (err) {
    console.error('[library] opening the import form failed:', err)
  }
}

export function openLibraryImportOnTab(tab: 'paste' | 'files'): void {
  selectImportTab(tab)
  openLibraryImportForm()
}

export function resolveAddChaptersTarget(): string | null {
  if (currentMode.value === 'library') {
    return libraryWorkPageWork.value?.work_id ?? currentLibraryWork.value?.work_id ?? null
  }
  return openWorkId.value
}

export async function openLibraryImportToAddChapters(): Promise<void> {
  try {
    const target = resolveAddChaptersTarget()
    presetImportDestination(target)
    const listIsStale =
      !destinationWorksHaveLoaded.value ||
      destinationWorksError.value !== null ||
      (target !== null && !destinationWorks.value.some((w) => w.work_id === target))
    if (listIsStale) await loadDestinationWorks()
    setMode('library')
    openLibraryImportOverlay()
  } catch (err) {
    console.error('[library] opening the import form to add chapters failed:', err)
  }
}
