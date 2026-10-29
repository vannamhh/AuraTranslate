import { readonly, ref, watch } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { lookupPending } from '../panels/lookupPanelState'

/** Selection rectangle in the content coordinates of the scrolling `section.mode` that holds the note. */
export type ReadingLookupAnchor = { top: number; bottom: number; left: number; right: number }

const anchor = ref<ReadingLookupAnchor | null>(null)
const awaitingAnswer = ref(false)

export const readingLookupAnchor: DeepReadonly<Ref<ReadingLookupAnchor | null>> = readonly(anchor)

/** `true` from a double-click until the lookup it started answers, so the previous word's result never shows at the new spot. */
export const readingLookupAwaitingAnswer: DeepReadonly<Ref<boolean>> = readonly(awaitingAnswer)

watch(lookupPending, (pending) => {
  if (!pending) awaitingAnswer.value = false
})

/**
 * Opens the popup at the current selection when that selection lies inside a bilingual source
 * note. Returns `false`, leaving the popup as it was, for any other selection.
 */
export function openReadingLookup(): boolean {
  const selection = window.getSelection()
  if (selection === null || selection.rangeCount === 0 || selection.isCollapsed) return false
  const node = selection.anchorNode
  // Same predicate as `selectionContract.ts::surfaceFor`: any other anchor makes the lookup read ''.
  if (node === null || node.nodeType !== Node.TEXT_NODE) return false
  if (selection.toString().trim() === '') return false
  const note = node.parentElement?.closest('[data-reading-source-note]') ?? null
  const host = note?.closest<HTMLElement>('section.mode') ?? null
  if (host === null) return false
  const rect = selection.getRangeAt(0).getBoundingClientRect()
  const box = host.getBoundingClientRect()
  const dx = host.scrollLeft - box.left
  const dy = host.scrollTop - box.top
  anchor.value = { top: rect.top + dy, bottom: rect.bottom + dy, left: rect.left + dx, right: rect.right + dx }
  awaitingAnswer.value = true
  return true
}

/** Closes the popup; also the reset that drops its anchor. */
export function resetReadingLookup(): void {
  anchor.value = null
  awaitingAnswer.value = false
}
