/**
 * State of the Chapter split box (A5): lists the source sentences of the selected Chapter and
 * lets the user pick the sentence that opens the new Chapter. Rust splits only the open Chapter,
 * so opening the box opens that Chapter in the Editor first. Do NOT import this file from
 * `src/commands/index.ts`; handlers are injected through `CommandDeps`.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { editorSegments, editorSplitChapterError, editorSplitChapterNotice, openChapterById, setEditorCaret, splitChapterHere } from './panels/editorPanelState'
import type { SplitChapterNotice } from './panels/editorPanelState'
import { loadChapters, soleSelectedLibraryChapter, libraryChapterRules } from './modes/libraryChapters'
import type { ChapterRow } from './config/chapter'
import type { IpcError } from './i18n'

export type ChapterSplitLoadStatus = 'loading' | 'loaded' | 'failed'
export type ChapterSplitSentence = { id: number; ord: number; source_text: string }
export type ChapterSplitFailure = { error: IpcError | null; notice: SplitChapterNotice | null }

const overlayOpen = ref(false)
const loadStatus = ref<ChapterSplitLoadStatus>('loading')
const chapter = ref<ChapterRow | null>(null)
const sentences = ref<ChapterSplitSentence[]>([])
const pickedId = ref<number | null>(null)
const busy = ref(false)
const failure = ref<ChapterSplitFailure | null>(null)
let sequence = 0

export const chapterSplitOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const chapterSplitLoadStatus: DeepReadonly<Ref<ChapterSplitLoadStatus>> = readonly(loadStatus)
export const chapterSplitChapter: DeepReadonly<Ref<ChapterRow | null>> = readonly(chapter)
export const chapterSplitSentences: DeepReadonly<Ref<ChapterSplitSentence[]>> = readonly(sentences)
export const chapterSplitPickedId: DeepReadonly<Ref<number | null>> = readonly(pickedId)
export const chapterSplitBusy: DeepReadonly<Ref<boolean>> = readonly(busy)
export const chapterSplitFailure: DeepReadonly<Ref<ChapterSplitFailure | null>> = readonly(failure)

/** The first sentence cannot open a new Chapter: the upper half would be empty. */
export function chapterSplitCanConfirm(): boolean {
  const first = sentences.value.at(0)
  return !busy.value && pickedId.value !== null && first !== undefined && pickedId.value !== first.id
}

export async function openChapterSplit(): Promise<void> {
  const row = soleSelectedLibraryChapter.value
  if (overlayOpen.value || row === null || !libraryChapterRules.value.split) return

  sequence += 1
  const mySequence = sequence
  overlayOpen.value = true
  loadStatus.value = 'loading'
  chapter.value = { ...row }
  sentences.value = []
  pickedId.value = null
  failure.value = null

  const opened = await openChapterById(row.chapter_id)
  if (mySequence !== sequence) return
  if (!opened) {
    loadStatus.value = 'failed'
    return
  }
  sentences.value = editorSegments.value.map((s) => ({ id: s.id, ord: s.ord, source_text: s.source_text }))
  loadStatus.value = 'loaded'
}

export function pickChapterSplitSentence(segmentId: number): void {
  pickedId.value = segmentId
  failure.value = null
}

export function closeChapterSplit(): void {
  sequence += 1
  overlayOpen.value = false
  busy.value = false
}

export async function confirmChapterSplit(): Promise<void> {
  if (!overlayOpen.value || !chapterSplitCanConfirm() || pickedId.value === null) return
  busy.value = true
  failure.value = null
  setEditorCaret(pickedId.value)
  const split = await splitChapterHere()
  busy.value = false
  if (!split) {
    failure.value = { error: editorSplitChapterError.value, notice: editorSplitChapterNotice.value }
    return
  }
  overlayOpen.value = false
  await loadChapters()
}

export function resetChapterSplit(): void {
  sequence += 1
  overlayOpen.value = false
  loadStatus.value = 'loading'
  chapter.value = null
  sentences.value = []
  pickedId.value = null
  busy.value = false
  failure.value = null
}
