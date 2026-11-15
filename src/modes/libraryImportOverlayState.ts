import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { resetImportOpener } from '../importOpener'

const overlayOpen = ref(false)

export const libraryImportOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)

export function openLibraryImportOverlay(): void {
  overlayOpen.value = true
}

export function closeLibraryImportOverlay(): void {
  overlayOpen.value = false
}

export function resetLibraryImportOverlay(): void {
  resetImportOpener()
  overlayOpen.value = false
}
