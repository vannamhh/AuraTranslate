import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'

const overlayOpen = ref(false)

export const libraryImportOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)

export function openLibraryImportOverlay(): void {
  overlayOpen.value = true
}

export function closeLibraryImportOverlay(): void {
  overlayOpen.value = false
}

export function resetLibraryImportOverlay(): void {
  overlayOpen.value = false
}
