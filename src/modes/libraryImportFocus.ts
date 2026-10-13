import { setMode } from './modeState'
import { openLibraryImportOverlay } from './libraryImportOverlayState'

export function openLibraryImportForm(): void {
  try {
    setMode('library')
    openLibraryImportOverlay()
  } catch (err) {
    console.error('[library] opening the import form failed:', err)
  }
}
