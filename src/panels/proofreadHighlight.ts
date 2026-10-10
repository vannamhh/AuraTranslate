export const PROOFREAD_HIGHLIGHT_NAME = 'proofread-error'

export type OffsetSpan = { readonly start: number; readonly end: number }

/** Maps UTF-16 offsets of `cell.textContent` onto a Range, across any number of text nodes. */
export function rangeForOffsets(cell: Element, start: number, end: number): Range | null {
  if (!(start >= 0 && end > start)) return null
  const doc = cell.ownerDocument
  const walker = doc.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
  const range = doc.createRange()
  let consumed = 0
  let startSet = false
  for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
    const length = node.textContent?.length ?? 0
    if (!startSet && start < consumed + length) {
      range.setStart(node, start - consumed)
      startSet = true
    }
    if (startSet && end <= consumed + length) {
      range.setEnd(node, end - consumed)
      return range
    }
    consumed += length
  }
  return null
}

/** `null` ⇔ the cell no longer holds the scanned text, so nothing may be drawn. */
export function rangesForScan(cell: Element, scannedText: string, spans: readonly OffsetSpan[]): Range[] | null {
  if (cell.textContent !== scannedText) return null
  const ranges: Range[] = []
  for (const span of spans) {
    const range = rangeForOffsets(cell, span.start, span.end)
    if (range !== null) ranges.push(range)
  }
  return ranges
}

export function paintProofreadRanges(ranges: readonly Range[] | null): void {
  if (typeof CSS === 'undefined' || !('highlights' in CSS)) return
  if (ranges === null || ranges.length === 0) {
    CSS.highlights.delete(PROOFREAD_HIGHLIGHT_NAME)
    return
  }
  CSS.highlights.set(PROOFREAD_HIGHLIGHT_NAME, new Highlight(...ranges))
}
