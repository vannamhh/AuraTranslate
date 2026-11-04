import { afterEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import SourceHanViet from '../../src/panels/SourceHanViet.vue'
import { mirrorParallelSelection, PARALLEL_SELECTION_HIGHLIGHT } from '../../src/panels/hanVietParallelSelection'

function host(html: string): HTMLElement {
  const p = document.createElement('p')
  p.innerHTML = html
  document.body.append(p)
  return p
}

function collect(): { ranges: Range[]; clear(): void; add(range: Range): void } {
  const ranges: Range[] = []
  return { ranges, clear: () => ranges.splice(0), add: (r: Range) => ranges.push(r) }
}

function select(startNode: Node, startOffset: number, endNode: Node, endOffset: number): Selection {
  const range = document.createRange()
  range.setStart(startNode, startOffset)
  range.setEnd(endNode, endOffset)
  const selection = window.getSelection()
  if (selection === null) throw new Error('no selection')
  selection.removeAllRanges()
  selection.addRange(range)
  return selection
}

afterEach(() => {
  window.getSelection()?.removeAllRanges()
  document.body.innerHTML = ''
})

describe('mirrorParallelSelection', () => {
  it('paints the selected base text clipped to the selection and leaves the readings out', () => {
    const p = host('<span><ruby>大紀元<rt>đại kỉ nguyên</rt></ruby></span><span>，</span><span><ruby>記者<rt>kí giả</rt></ruby></span>')
    const [first, last] = [...p.querySelectorAll('ruby')].map((r) => r.firstChild as Text)
    const sink = collect()
    mirrorParallelSelection([p], select(first, 1, last, 1), sink)
    expect(sink.ranges.map((r) => r.toString())).toEqual(['紀元', '，', '記'])
  })

  it('paints nothing for a selection outside every host', () => {
    const p = host('<span><ruby>大紀元<rt>đại kỉ nguyên</rt></ruby></span>')
    const outside = document.createElement('div')
    outside.textContent = 'ngoài'
    document.body.append(outside)
    const sink = collect()
    sink.add(document.createRange())
    mirrorParallelSelection([p], select(outside.firstChild as Text, 0, outside.firstChild as Text, 3), sink)
    expect(sink.ranges).toEqual([])
  })
})

describe('SourceHanViet — Song song selection goes through the CSS highlight', () => {
  it('hides native selection on the paragraph and mirrors a selection into the shared highlight', async () => {
    const w = mount(SourceHanViet, {
      props: { sourceText: '京都」，春風', viewMode: 'parallel', cuts: [], glossaryTerms: [], surfaceRole: 'cell' as const },
      attachTo: document.body,
    })
    await nextTick()
    const paragraph = w.get('.hv-parallel')
    expect(paragraph.classes()).toContain('hv-mirrored-selection')

    const base = paragraph.element.querySelector('ruby')?.firstChild
    if (!(base instanceof Text)) throw new Error('no ruby base text')
    select(base, 0, base, 2)
    document.dispatchEvent(new Event('selectionchange'))

    const painted = CSS.highlights.get(PARALLEL_SELECTION_HIGHLIGHT)
    expect(painted === undefined ? [] : [...painted].map((r) => (r as Range).toString())).toEqual(['京都'])
    w.unmount()
    expect(CSS.highlights.has(PARALLEL_SELECTION_HIGHLIGHT)).toBe(false)
  })
})
