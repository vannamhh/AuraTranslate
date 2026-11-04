/** Name of the CSS highlight that paints the selection inside the Song song view. */
export const PARALLEL_SELECTION_HIGHLIGHT = 'hv-parallel-selection'

type RangeSink = { clear(): void; add(range: Range): unknown }

/** True when the webview can paint a CSS Custom Highlight, so native selection may be hidden. */
export function parallelSelectionMirrorAvailable(): boolean {
  return typeof CSS !== 'undefined' && 'highlights' in CSS && typeof Highlight === 'function'
}

/**
 * Copies the parts of `selection` that fall on base text of `hosts` into `sink`, one range per
 * text node, leaving out `<rt>` readings the way native selection does.
 */
export function mirrorParallelSelection(hosts: Iterable<HTMLElement>, selection: Selection | null, sink: RangeSink): void {
  sink.clear()
  if (selection === null) return
  for (let i = 0; i < selection.rangeCount; i++) {
    const range = selection.getRangeAt(i)
    if (range.collapsed) continue
    for (const host of hosts) {
      if (!range.intersectsNode(host)) continue
      const walker = document.createTreeWalker(host, NodeFilter.SHOW_TEXT)
      for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
        if (node.parentElement?.closest('rt') != null || !range.intersectsNode(node)) continue
        const part = document.createRange()
        part.selectNodeContents(node)
        if (range.compareBoundaryPoints(Range.START_TO_START, part) > 0) part.setStart(range.startContainer, range.startOffset)
        if (range.compareBoundaryPoints(Range.END_TO_END, part) < 0) part.setEnd(range.endContainer, range.endOffset)
        if (!part.collapsed) sink.add(part)
      }
    }
  }
}

const hosts = new Set<HTMLElement>()
let highlight: Highlight | null = null

function sync(): void {
  if (highlight !== null) mirrorParallelSelection(hosts, window.getSelection(), highlight)
}

/** Paints selection on `host` through the shared highlight; returns the release. */
export function registerParallelSelectionHost(host: HTMLElement): () => void {
  if (!parallelSelectionMirrorAvailable()) return () => {}
  if (highlight === null) {
    highlight = new Highlight()
    CSS.highlights.set(PARALLEL_SELECTION_HIGHLIGHT, highlight)
    document.addEventListener('selectionchange', sync)
  }
  hosts.add(host)
  sync()
  return () => {
    hosts.delete(host)
    if (hosts.size > 0) sync()
    else resetParallelSelectionMirror()
  }
}

/** Drops every host and unregisters the shared highlight and its listener. */
export function resetParallelSelectionMirror(): void {
  hosts.clear()
  if (highlight === null) return
  document.removeEventListener('selectionchange', sync)
  CSS.highlights.delete(PARALLEL_SELECTION_HIGHLIGHT)
  highlight = null
}
