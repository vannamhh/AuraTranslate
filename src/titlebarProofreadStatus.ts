import { computed } from 'vue'
import { t } from './i18n'
import { libraryChapters } from './modes/libraryChapters'
import { editorChapterId } from './panels/editorPanelState'
import { proofreadProgress, proofreadStateValue } from './proofreadState'

export const proofreadStatusText = computed<string | null>(() => {
  if (proofreadStateValue.value !== 'done' || editorChapterId.value === null) return null
  const chapterId = editorChapterId.value
  const chapter = libraryChapters.value.find((row) => row.chapter_id === chapterId)
  if (chapter === undefined) return null
  return t('titlebar.proofread_status', {
    chapter: t('mode.library.chapter_untitled', { ord: String(chapter.ord) }),
    sentence_count: String(proofreadProgress.value.done),
  })
})
