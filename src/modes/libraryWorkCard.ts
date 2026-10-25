import { t } from '../i18n'
import type { WorkRow } from '../config/library'

export interface WorkCardMeta {
  langLabel: string | null
  line: string | null
}

const MINUTE_MS = 60_000
const HOUR_MS = 60 * MINUTE_MS
const DAY_MS = 24 * HOUR_MS

export function sourceLangLabel(code: string): string | null {
  if (code.trim() === '') return null
  if (code === 'zh') return t('mode.library.lang_zh')
  if (code === 'en') return t('mode.library.lang_en')
  return code
}

export function sourceLangChoices(): { value: string; label: string }[] {
  return [
    { value: 'zh', label: t('mode.library.lang_zh') },
    { value: 'en', label: t('mode.library.lang_en') },
  ]
}

const GENRE_IDS = [
  'news',
  'commentary',
  'tcm',
  'health',
  'cuisine',
  'family',
  'travel',
  'culture',
  'history_figures',
  'spiritual',
  'arts',
  'business',
  'science',
  'legal',
  'nonfiction',
  'xianxia',
  'xuanhuan',
  'wuxia',
  'urban',
  'romance',
  'historical',
  'scifi',
  'game',
  'horror',
  'mystery',
  'fanfic',
  'literary',
  'children',
  'other',
] as const
type GenreId = (typeof GENRE_IDS)[number]

function isGenreId(raw: string): raw is GenreId {
  return (GENRE_IDS as readonly string[]).includes(raw)
}

/** Empty is `null`; an id outside the list (older free-text value) is shown verbatim. */
export function genreLabel(raw: string): string | null {
  const genre = raw.trim()
  if (genre === '') return null
  return isGenreId(genre) ? t(`mode.library.genre_${genre}`) : genre
}

export function genreChoices(): { value: string; label: string }[] {
  return [
    { value: '', label: t('mode.library.genre_unset') },
    ...GENRE_IDS.map((id) => ({ value: id, label: t(`mode.library.genre_${id}`) })),
  ]
}

/** `null` when the timestamp is unreadable or lies in the future. */
export function updatedLabel(updatedAt: string, now: Date): string | null {
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
  const genre = genreLabel(work.genre)
  if (genre !== null) parts.push(genre)
  const updated = updatedLabel(work.updated_at, now)
  if (updated !== null) parts.push(updated)
  return { langLabel: sourceLangLabel(work.source_lang), line: parts.length === 0 ? null : parts.join(' · ') }
}

export const COVER_TONE_COUNT = 6

/** Stable tone `1..6` for a Work: renaming never recolours its cover. */
export function workCoverTone(workId: string): number {
  let hash = 0x811c9dc5
  for (let index = 0; index < workId.length; index += 1) {
    hash = Math.imul(hash ^ workId.charCodeAt(index), 0x01000193) >>> 0
  }
  return (hash % COVER_TONE_COUNT) + 1
}
