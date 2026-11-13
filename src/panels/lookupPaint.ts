const LOOKUP_HIGHLIGHT = 'lookup-active'

/** `CSS.highlights` is missing on engines without the Custom Highlight API; the DOM typings say it always exists. */
function highlightRegistry(): HighlightRegistry | null {
  const css: { highlights?: HighlightRegistry } | undefined = typeof CSS === 'undefined' ? undefined : CSS
  return css?.highlights ?? null
}

const LOOKUP_BLOCK_ATTR = 'data-lookup-active'

let paintedBlocks: Element[] = []

export function clearLookupPaint(): void {
  highlightRegistry()?.delete(LOOKUP_HIGHLIGHT)
  for (const block of paintedBlocks) block.removeAttribute(LOOKUP_BLOCK_ATTR)
  paintedBlocks = []
}

/** Marks whole atomic blocks with an attribute (colour lives in CSS); Custom Highlight does not repaint reliably over them on WebKit. */
export function paintLookupBlocks(blocks: readonly Element[]): void {
  clearLookupPaint()
  for (const block of blocks) block.setAttribute(LOOKUP_BLOCK_ATTR, '')
  paintedBlocks = [...blocks]
}

/** Colours `ranges` with the `lookup-active` highlight; without the API, selects them (black-on-highlight) instead. */
export function paintLookupRanges(ranges: readonly Range[]): void {
  clearLookupPaint()
  const registry = highlightRegistry()
  if (registry !== null && typeof Highlight === 'function') {
    registry.set(LOOKUP_HIGHLIGHT, new Highlight(...ranges))
    return
  }
  const selection = window.getSelection()
  const first = ranges.at(0)
  const last = ranges.at(-1)
  if (selection === null || first === undefined || last === undefined) return
  const whole = first.cloneRange()
  whole.setEnd(last.endContainer, last.endOffset)
  selection.removeAllRanges()
  selection.addRange(whole)
}
