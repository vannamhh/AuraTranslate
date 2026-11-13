const LOOKUP_BLOCK_ATTR = 'data-lookup-active'

let paintedBlocks: Element[] = []

export function clearLookupPaint(): void {
  for (const block of paintedBlocks) block.removeAttribute(LOOKUP_BLOCK_ATTR)
  paintedBlocks = []
}

/** Marks whole atomic blocks with an attribute (colour lives in CSS); a block is not a text range. */
export function paintLookupBlocks(blocks: readonly Element[]): void {
  clearLookupPaint()
  for (const block of blocks) block.setAttribute(LOOKUP_BLOCK_ATTR, '')
  paintedBlocks = [...blocks]
}
