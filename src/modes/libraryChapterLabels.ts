import { t } from '../i18n'
import type { ChapterRow } from '../config/chapter'

export function chapterTitleLabel(row: Pick<ChapterRow, 'title' | 'ord'>): string {
  return row.title ?? t('mode.library.chapter_untitled', { ord: String(row.ord) })
}

/** One of the four lifecycle values, or the raw string when the value is unknown. */
export function lifecycleStatusLabel(status: string): string {
  if (status === 'not_started') return t('lifecycle.not_started')
  if (status === 'in_progress') return t('lifecycle.in_progress')
  if (status === 'paused') return t('lifecycle.paused')
  if (status === 'done') return t('lifecycle.done')
  return t('mode.library.works_status_invalid', { status })
}
