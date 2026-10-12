export const PROOFREAD_ERROR_CLASS = 'proofread-error'
export const PROOFREAD_MEANING_CLASS = 'proofread-meaning'
export const PROOFREAD_SPAN_SELECTOR = `.${PROOFREAD_ERROR_CLASS}, .${PROOFREAD_MEANING_CLASS}`

export type OffsetSpan = { readonly start: number; readonly end: number }
export type ProofreadMark = OffsetSpan & { readonly meaning: boolean }

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

function offsetOf(cell: Element, node: Node, offset: number): number {
  const range = cell.ownerDocument.createRange()
  range.setStart(cell, 0)
  range.setEnd(node, offset)
  return range.toString().length
}

function pointAt(cell: Element, offset: number): { node: Node; offset: number } {
  const walker = cell.ownerDocument.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
  let consumed = 0
  let last: Node | null = null
  for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
    const length = node.textContent?.length ?? 0
    if (offset <= consumed + length) return { node, offset: offset - consumed }
    consumed += length
    last = node
  }
  return last === null ? { node: cell, offset: 0 } : { node: last, offset: last.textContent?.length ?? 0 }
}

function keepingSelection(cell: Element, change: () => void): void {
  const selection = cell.ownerDocument.defaultView?.getSelection() ?? null
  const anchor = selection?.anchorNode ?? null
  const focus = selection?.focusNode ?? null
  if (selection === null || anchor === null || focus === null || !cell.contains(anchor) || !cell.contains(focus)) {
    change()
    return
  }
  const anchorOffset = offsetOf(cell, anchor, selection.anchorOffset)
  const focusOffset = offsetOf(cell, focus, selection.focusOffset)
  change()
  const a = pointAt(cell, anchorOffset)
  const f = pointAt(cell, focusOffset)
  selection.setBaseAndExtent(a.node, a.offset, f.node, f.offset)
}

/**
 * Removes every proofread span of one cell. `mergeText` also joins the pieces into one text node; leave it off
 * from an input event, because WebKit inserts into the text node it already holds and a merge detaches it.
 */
export function unwrapProofread(cell: Element, mergeText = false): void {
  const spans = cell.querySelectorAll(PROOFREAD_SPAN_SELECTOR)
  if (spans.length === 0) return
  keepingSelection(cell, () => {
    for (const span of spans) span.replaceWith(...span.childNodes)
    if (mergeText) cell.normalize()
  })
}

/**
 * Wraps each mark in an inline span so a real `text-decoration` can take `text-underline-offset`.
 * Overlapping marks are dropped (first by start wins); nothing is drawn once the cell text differs from `scannedText`.
 */
export function wrapProofread(cell: Element, scannedText: string, marks: readonly ProofreadMark[]): void {
  if (cell.textContent !== scannedText || marks.length === 0) return
  const ordered = [...marks].sort((a, b) => a.start - b.start || b.end - a.end)
  keepingSelection(cell, () => {
    let drawnTo = 0
    for (const mark of ordered) {
      if (mark.start < drawnTo) continue
      const range = rangeForOffsets(cell, mark.start, mark.end)
      if (range === null) continue
      const span = cell.ownerDocument.createElement('span')
      span.className = mark.meaning ? PROOFREAD_MEANING_CLASS : PROOFREAD_ERROR_CLASS
      span.appendChild(range.extractContents())
      range.insertNode(span)
      drawnTo = mark.end
    }
  })
}

export function unwrapAllProofread(host: Element): void {
  const cells = new Set<Element>()
  for (const span of host.querySelectorAll(PROOFREAD_SPAN_SELECTOR)) {
    const cell = span.closest('[data-segment-id]')
    if (cell !== null) cells.add(cell)
  }
  for (const cell of cells) unwrapProofread(cell, true)
}

export type ScanEntry<K> = {
  readonly scannedText: string
  readonly findings: readonly { readonly start: number; readonly end: number; readonly kind: K }[]
}

/** Redraws every scanned cell of `host` from scratch: unwrap all, then wrap each cell that still holds its scanned text. */
export function repaintProofreadHost<K>(
  host: Element,
  results: Iterable<readonly [number, ScanEntry<K>]>,
  isMeaning: (kind: K) => boolean,
): void {
  unwrapAllProofread(host)
  for (const [id, entry] of results) {
    const cell = host.querySelector(`[data-segment-id="${String(id)}"]`)
    if (cell === null) continue
    wrapProofread(cell, entry.scannedText, entry.findings.map((f) => ({ start: f.start, end: f.end, meaning: isMeaning(f.kind) })))
  }
}
