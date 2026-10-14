import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { resetImportOpener } from '../importOpener'

const overlayOpen = ref(false)
const urlScreenOpen = ref(false)

export const libraryImportOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const libraryUrlImportScreenIsOpen: DeepReadonly<Ref<boolean>> = readonly(urlScreenOpen)

export function openLibraryImportOverlay(): void {
  overlayOpen.value = true
}

export function closeLibraryImportOverlay(): void {
  overlayOpen.value = false
  urlScreenOpen.value = false
}

export function openLibraryUrlImportScreen(): void {
  if (!overlayOpen.value) return
  urlScreenOpen.value = true
}

export function closeLibraryUrlImportScreen(): void {
  urlScreenOpen.value = false
}

export function resetLibraryImportOverlay(): void {
  resetImportOpener()
  overlayOpen.value = false
  urlScreenOpen.value = false
}
