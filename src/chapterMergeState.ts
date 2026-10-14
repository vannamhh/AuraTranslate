/**
 * State of the Chapter merge screen (A6): a snapshot of the selected Chapters taken when the
 * screen opens, so the list stays stable while the write runs. The write itself lives in
 * `modes/libraryChapters.ts::mergeSelectedChapters`. Do NOT import this file from
 * `src/commands/index.ts`; handlers are injected through `CommandDeps`.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import type { ChapterRow } from './config/chapter'
import { libraryChapterRules, libraryChapterSelectedRows, mergeSelectedChapters } from './modes/libraryChapters'

const overlayOpen = ref(false)
const rows = ref<ChapterRow[]>([])

export const chapterMergeOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const chapterMergeRows: DeepReadonly<Ref<ChapterRow[]>> = readonly(rows)

export function openChapterMerge(): void {
  if (overlayOpen.value || !libraryChapterRules.value.merge) return
  rows.value = libraryChapterSelectedRows.value.map((row) => ({ ...row }))
  overlayOpen.value = true
}

export function closeChapterMerge(): void {
  overlayOpen.value = false
}

export async function confirmChapterMerge(): Promise<void> {
  if (!overlayOpen.value) return
  const outcome = await mergeSelectedChapters()
  if (outcome === 'merged') overlayOpen.value = false
}

export function resetChapterMerge(): void {
  overlayOpen.value = false
  rows.value = []
}
