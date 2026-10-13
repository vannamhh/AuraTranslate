import { t } from '../i18n'
import type { WorkRow } from '../config/library'

export interface WorkCardMeta {
  langLabel: string | null
  line: string | null
}

const MINUTE_MS = 60_000
const HOUR_MS = 60 * MINUTE_MS
const DAY_MS = 24 * HOUR_MS

function sourceLangLabel(code: string): string | null {
  if (code.trim() === '') return null
  if (code === 'zh') return t('mode.library.lang_zh')
  if (code === 'en') return t('mode.library.lang_en')
  return code
}

/** `null` when the timestamp is unreadable or lies in the future. */
function updatedLabel(updatedAt: string, now: Date): string | null {
  const stamp = Date.parse(updatedAt)
  if (Number.isNaN(stamp)) return null
  const diff = now.getTime() - stamp
  if (diff < 0) return null
  if (diff < MINUTE_MS) return t('mode.library.work_updated_just_now')
  if (diff < HOUR_MS) return t('mode.library.work_updated_minutes', { count: String(Math.floor(diff / MINUTE_MS)) })
  if (diff < DAY_MS) return t('mode.library.work_updated_hours', { count: String(Math.floor(diff / HOUR_MS)) })
  if (diff < 2 * DAY_MS) return t('mode.library.work_updated_yesterday')
  if (diff < 30 * DAY_MS) return t('mode.library.work_updated_days', { count: String(Math.floor(diff / DAY_MS)) })
  return t('mode.library.work_updated_on', { date: new Date(stamp).toISOString().slice(0, 10) })
}

export function workCardMeta(work: Pick<WorkRow, 'source_lang' | 'genre' | 'updated_at'>, now: Date): WorkCardMeta {
  const parts: string[] = []
  const genre = work.genre.trim()
  if (genre !== '') parts.push(genre)
  const updated = updatedLabel(work.updated_at, now)
  if (updated !== null) parts.push(updated)
  return { langLabel: sourceLangLabel(work.source_lang), line: parts.length === 0 ? null : parts.join(' · ') }
}
