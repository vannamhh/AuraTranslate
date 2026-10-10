import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { PROOFREAD_HIGHLIGHT_NAME, paintProofreadRanges, rangeForOffsets, rangesForScan } from '../../src/panels/proofreadHighlight'

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

describe('rangesForScan', () => {
  const spans = [{ start: 0, end: 2 }, { start: 3, end: 5 }]

  it('draws one range per span when the cell still holds the scanned text', () => {
    const ranges = rangesForScan(cellOf('ab', ' cd'), 'ab cd', spans)
    expect(ranges?.map((r) => r.toString())).toEqual(['ab', 'cd'])
  })

  it('draws nothing when the cell text differs from the scanned text', () => {
    expect(rangesForScan(cellOf('ab cdx'), 'ab cd', spans)).toBeNull()
  })
})

describe('paintProofreadRanges', () => {
  const registry = new Map<string, unknown>()

  beforeEach(() => {
    registry.clear()
    vi.stubGlobal('CSS', { highlights: registry })
    vi.stubGlobal('Highlight', class { readonly ranges: Range[]; constructor(...ranges: Range[]) { this.ranges = ranges } })
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('registers the ranges under the proofread highlight name', () => {
    const ranges = rangesForScan(cellOf('ab cd'), 'ab cd', [{ start: 3, end: 5 }])
    paintProofreadRanges(ranges)
    const painted = registry.get(PROOFREAD_HIGHLIGHT_NAME) as { ranges: Range[] } | undefined
    expect(painted?.ranges.map((r) => r.toString())).toEqual(['cd'])
  })

  it('removes the highlight when there is nothing to draw', () => {
    paintProofreadRanges(rangesForScan(cellOf('ab'), 'ab', [{ start: 0, end: 2 }]))
    expect(registry.has(PROOFREAD_HIGHLIGHT_NAME)).toBe(true)
    paintProofreadRanges(null)
    expect(registry.has(PROOFREAD_HIGHLIGHT_NAME)).toBe(false)
  })
})
