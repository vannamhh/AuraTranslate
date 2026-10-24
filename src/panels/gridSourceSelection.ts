const SOURCE_CELL = '[data-col="src"]'

function elementOf(node: Node | null): Element | null {
  return node instanceof Element ? node : (node?.parentElement ?? null)
}

export function isSourceColumnSelecting(selection: Selection | null, root: Element | null): boolean {
  if (selection === null || root === null || selection.isCollapsed) return false
  const from = elementOf(selection.anchorNode)
  return from !== null && root.contains(from) && from.closest(SOURCE_CELL) !== null
}

/**
 * Text of the source cells a selection covers, row order, one per line. A range that crosses a
 * grid-block boundary also spans the translation, number and rule cells of the first block in
 * DOM order; those are not part of what the user selected.
 */
export function sourceColumnText(selection: Selection, root: Element): string {
  if (selection.rangeCount === 0) return ''
  const range = selection.getRangeAt(0)
  const cells = [...root.querySelectorAll(SOURCE_CELL)]
  const indexOfCell = (node: Node, fallback: number): number => {
    const from = elementOf(node)
    let cell = from?.closest(SOURCE_CELL) ?? null
    if (cell === null) {
      const id = from?.closest('[data-segment-id]')?.getAttribute('data-segment-id')
      if (id != null) cell = root.querySelector(`${SOURCE_CELL}[data-segment-id="${CSS.escape(id)}"]`)
    }
    const at = cell === null ? -1 : cells.indexOf(cell)
    return at === -1 ? fallback : at
  }
  const first = indexOfCell(range.startContainer, 0)
  const last = indexOfCell(range.endContainer, cells.length - 1)
  const lines: string[] = []
  for (const cell of cells.slice(first, last + 1)) {
    if (!range.intersectsNode(cell)) continue
    const part = range.cloneRange()
    if (!cell.contains(range.startContainer)) part.setStart(cell, 0)
    if (!cell.contains(range.endContainer)) part.setEnd(cell, cell.childNodes.length)
    const text = part.toString()
    if (text !== '') lines.push(text)
  }
  return lines.join('\n')
}
