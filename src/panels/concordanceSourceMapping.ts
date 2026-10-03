import type { SegmentTermSpan } from './glossaryMarksMap'

export type ConcordanceSourceSpan = { start: number; end: number }

function fold(text: string): string {
  return text.normalize('NFC').trim().toLowerCase()
}

/**
 * Maps a Vietnamese selection back to a Chinese span of the active segment: the earliest
 * confirmed Glossary span whose translation equals the selection (NFC, trimmed, case-insensitive).
 */
export function sourceSpanForVietnameseSelection(
  selection: string,
  spans: readonly SegmentTermSpan[],
): ConcordanceSourceSpan | null {
  const wanted = fold(selection)
  if (wanted === '') return null
  let best: SegmentTermSpan | null = null
  for (const span of spans) {
    if (!span.isConfirmed || span.translation === null) continue
    if (fold(span.translation) !== wanted) continue
    if (best === null || span.start < best.start) best = span
  }
  return best === null ? null : { start: best.start, end: best.end }
}

/** Slice by code point, the unit of every Glossary span offset. */
export function sliceByCodePoint(text: string, start: number, end: number): string {
  return Array.from(text).slice(start, end).join('')
}
