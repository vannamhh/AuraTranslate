import type { ReadingTranslationOriginCounts } from '../config/reading'

export type ReadingOriginLabel = 'self' | 'other' | 'bilingual_import' | 'mixed' | 'none'

/** "mixed" is a display label only; it is never a stored translation origin. */
export function readingOriginLabel(counts: ReadingTranslationOriginCounts): ReadingOriginLabel {
  const present: ReadingOriginLabel[] = []
  if (counts.self_count > 0) present.push('self')
  if (counts.other_count > 0) present.push('other')
  if (counts.bilingual_import_count > 0) present.push('bilingual_import')
  if (present.length === 0) return 'none'
  return present.length === 1 ? (present[0] as ReadingOriginLabel) : 'mixed'
}
