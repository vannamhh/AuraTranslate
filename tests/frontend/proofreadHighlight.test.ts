import { describe, expect, it } from 'vitest'
import { PROOFREAD_ERROR_CLASS, PROOFREAD_MEANING_CLASS, rangeForOffsets, repaintProofreadHost, unwrapProofread, wrapProofread } from '../../src/panels/proofreadHighlight'

function cellOf(...texts: string[]): HTMLElement {
  const cell = document.createElement('div')
  for (const text of texts) cell.appendChild(document.createTextNode(text))
  return cell
}

describe('rangeForOffsets', () => {
  it('maps a span inside one text node', () => {
    const cell = cellOf('xin chaof bạn')
    expect(rangeForOffsets(cell, 8, 9)?.toString()).toBe('f')
  })

  it('maps a span crossing several text nodes', () => {
    const cell = cellOf('ab\n', 'cd\n', 'ef')
    const range = rangeForOffsets(cell, 1, 7)
    expect(range?.toString()).toBe('b\ncd\ne')
    expect(range?.startContainer).toBe(cell.childNodes[0])
    expect(range?.endContainer).toBe(cell.childNodes[2])
  })

  it('counts a character outside the BMP as two UTF-16 units', () => {
    const cell = cellOf('𠀀 lỗi')
    expect(rangeForOffsets(cell, 3, 6)?.toString()).toBe('lỗi')
  })

  it('returns null for an empty or out-of-range span', () => {
    const cell = cellOf('abc')
    expect(rangeForOffsets(cell, 2, 2)).toBeNull()
    expect(rangeForOffsets(cell, 1, 9)).toBeNull()
    expect(rangeForOffsets(cell, -1, 2)).toBeNull()
  })
})


describe('wrapProofread', () => {
  const spansOf = (cell: Element, cls: string) => [...cell.querySelectorAll(`.${cls}`)].map((e) => e.textContent)

  it('gives each kind its own class', () => {
    const cell = cellOf('ab cd')
    wrapProofread(cell, 'ab cd', [{ start: 0, end: 2, meaning: false }, { start: 3, end: 5, meaning: true }])
    expect(spansOf(cell, PROOFREAD_ERROR_CLASS)).toEqual(['ab'])
    expect(spansOf(cell, PROOFREAD_MEANING_CLASS)).toEqual(['cd'])
    expect(cell.textContent).toBe('ab cd')
  })

  it('wraps a quote that crosses several text nodes in one span', () => {
    const cell = cellOf('ab\n', 'cd\n', 'ef')
    wrapProofread(cell, 'ab\ncd\nef', [{ start: 1, end: 7, meaning: false }])
    expect(spansOf(cell, PROOFREAD_ERROR_CLASS)).toEqual(['b\ncd\ne'])
    expect(cell.textContent).toBe('ab\ncd\nef')
  })

  it('draws nothing when the cell text differs from the scanned text', () => {
    const cell = cellOf('ab cdx')
    wrapProofread(cell, 'ab cd', [{ start: 0, end: 2, meaning: false }])
    expect(cell.querySelector('span')).toBeNull()
  })

  it('does not nest: touching marks stay siblings and an overlapping mark is dropped', () => {
    const cell = cellOf('abcdef')
    wrapProofread(cell, 'abcdef', [
      { start: 0, end: 2, meaning: false },
      { start: 2, end: 4, meaning: true },
      { start: 3, end: 6, meaning: false },
    ])
    expect(cell.querySelectorAll('span span')).toHaveLength(0)
    expect(spansOf(cell, PROOFREAD_ERROR_CLASS)).toEqual(['ab'])
    expect(spansOf(cell, PROOFREAD_MEANING_CLASS)).toEqual(['cd'])
    expect(cell.textContent).toBe('abcdef')
  })

  it('unwrap leaves the text split unless mergeText is set, then joins it into one node', () => {
    const cell = cellOf('ab cd')
    wrapProofread(cell, 'ab cd', [{ start: 0, end: 2, meaning: false }, { start: 3, end: 5, meaning: true }])
    unwrapProofread(cell)
    expect(cell.querySelector('span')).toBeNull()
    expect(cell.childNodes.length).toBeGreaterThan(1)
    wrapProofread(cell, 'ab cd', [{ start: 0, end: 2, meaning: false }])
    unwrapProofread(cell, true)
    expect(cell.childNodes).toHaveLength(1)
    expect(cell.firstChild?.nodeType).toBe(Node.TEXT_NODE)
    expect(cell.textContent).toBe('ab cd')
  })

  it('keeps the selection at the same text offset across wrap and unwrap', () => {
    const cell = cellOf('ab cd ef')
    document.body.appendChild(cell)
    try {
      const selection = window.getSelection()
      selection?.setPosition(cell.firstChild, 4)
      const caret = () => {
        const r = document.createRange()
        r.setStart(cell, 0)
        r.setEnd(selection?.anchorNode as Node, selection?.anchorOffset ?? 0)
        return r.toString().length
      }
      wrapProofread(cell, 'ab cd ef', [{ start: 3, end: 5, meaning: false }])
      expect(caret()).toBe(4)
      unwrapProofread(cell, true)
      expect(caret()).toBe(4)
    } finally {
      cell.remove()
    }
  })
})

describe('repaintProofreadHost after a typed edit in one segment', () => {
  const isMeaning = (kind: string) => kind === 'meaning'
  const hostWithTwoCells = () => {
    const host = document.createElement('div')
    for (const [id, text] of [[1, 'ab cd'], [2, 'ef gh']] as const) {
      const cell = cellOf(text)
      cell.setAttribute('data-segment-id', String(id))
      host.appendChild(cell)
    }
    return host
  }
  const scan = (text: string) => ({ scannedText: text, findings: [{ start: 0, end: 2, kind: 'spelling' }, { start: 3, end: 5, kind: 'meaning' }] })

  it('keeps the other segment underlined and drops only the edited one', () => {
    const host = hostWithTwoCells()
    const results = new Map([[1, scan('ab cd')], [2, scan('ef gh')]])
    repaintProofreadHost(host, results, isMeaning)
    expect(host.querySelectorAll('span')).toHaveLength(4)

    const edited = host.querySelector('[data-segment-id="1"]') as HTMLElement
    unwrapProofread(edited, true)
    edited.firstChild!.textContent = 'ab cdx'
    results.delete(1)
    repaintProofreadHost(host, results, isMeaning)

    expect(edited.querySelectorAll('span')).toHaveLength(0)
    const other = host.querySelector('[data-segment-id="2"]') as HTMLElement
    expect([...other.querySelectorAll('span')].map((e) => e.textContent)).toEqual(['ef', 'gh'])
    expect(other.textContent).toBe('ef gh')
  })

  it('draws nothing in a cell whose text moved on, even if its entry is still there', () => {
    const host = hostWithTwoCells()
    repaintProofreadHost(host, new Map([[1, scan('ab cdx')]]), isMeaning)
    expect(host.querySelectorAll('span')).toHaveLength(0)
  })

  it('repaint merges the text back into one node before wrapping', () => {
    const host = hostWithTwoCells()
    const results = new Map([[1, scan('ab cd')]])
    repaintProofreadHost(host, results, isMeaning)
    repaintProofreadHost(host, results, isMeaning)
    expect(host.querySelectorAll('[data-segment-id="1"] span span')).toHaveLength(0)
    expect(host.querySelectorAll('[data-segment-id="1"] span')).toHaveLength(2)
  })
})
