import { afterEach, describe, expect, it } from 'vitest'
import { isSourceColumnSelecting, sourceColumnText } from '../../src/panels/gridSourceSelection'

let nextId = 0

function block(rows: readonly (readonly [string, string])[]): string {
  const base = nextId
  nextId += rows.length
  const src = rows.map(([s], i) => `<div class="cell-src" data-col="src" data-segment-id="${base + i}"><span class="src-piece">${s}</span></div>`)
  const tgt = rows.map(([, t], i) => `<div class="cell-tgt" data-col="tgt" data-segment-id="${base + i}">${t}</div>`)
  return `<div class="grid-block"><div class="col col-num">12</div><div class="col col-src">${src.join('')}</div><div class="col col-tgt">${tgt.join('')}</div></div>`
}

function mountGrid(): HTMLElement {
  nextId = 0
  const root = document.createElement('div')
  root.className = 'grid'
  root.innerHTML = block([['AAA', 'tr-A'], ['BBB', 'tr-B']]) + block([['CCC', 'tr-C'], ['DDD', 'tr-D']])
  document.body.appendChild(root)
  return root
}

function pieceText(root: HTMLElement, src: string): Text {
  const piece = [...root.querySelectorAll('.src-piece')].find((p) => p.textContent === src)
  return piece!.firstChild as Text
}

function select(from: Text, fromOffset: number, to: Text, toOffset: number): Selection {
  const selection = window.getSelection()!
  const range = document.createRange()
  range.setStart(from, fromOffset)
  range.setEnd(to, toOffset)
  selection.removeAllRanges()
  selection.addRange(range)
  return selection
}

afterEach(() => {
  window.getSelection()?.removeAllRanges()
  document.body.replaceChildren()
})

describe('sourceColumnText', () => {
  it('a range from a source cell of block 0 into block 1 returns source text only', () => {
    const root = mountGrid()
    const selection = select(pieceText(root, 'BBB'), 1, pieceText(root, 'CCC'), 2)

    expect(selection.toString()).toContain('tr-B')
    expect(sourceColumnText(selection, root)).toBe('BB\nCC')
  })

  it('a range over every row returns every source cell in row order', () => {
    const root = mountGrid()
    const selection = select(pieceText(root, 'AAA'), 0, pieceText(root, 'DDD'), 3)

    expect(sourceColumnText(selection, root)).toBe('AAA\nBBB\nCCC\nDDD')
  })

  it('a range inside one cell returns the slice of that cell', () => {
    const root = mountGrid()
    const selection = select(pieceText(root, 'CCC'), 1, pieceText(root, 'CCC'), 3)

    expect(sourceColumnText(selection, root)).toBe('CC')
  })
})

describe('sourceColumnText with an end in the translation column', () => {
  it('resolves an end that sits in a translation cell to the source cell of the same row', () => {
    const root = mountGrid()
    const tgt = [...root.querySelectorAll('.cell-tgt')].find((c) => c.textContent === 'tr-B')!.firstChild as Text
    const selection = select(pieceText(root, 'AAA'), 1, tgt, 2)

    expect(sourceColumnText(selection, root)).toBe('AA\nBBB')
  })

  it('a start that sits in a translation cell keeps only the source cells after it', () => {
    const root = mountGrid()
    const tgt = [...root.querySelectorAll('.cell-tgt')].find((c) => c.textContent === 'tr-B')!.firstChild as Text
    const selection = select(tgt, 1, pieceText(root, 'DDD'), 2)

    expect(sourceColumnText(selection, root)).toBe('CCC\nDD')
  })
})

describe('isSourceColumnSelecting', () => {
  it('is true for an expanded selection anchored in a source cell', () => {
    const root = mountGrid()
    const selection = select(pieceText(root, 'AAA'), 0, pieceText(root, 'CCC'), 1)

    expect(isSourceColumnSelecting(selection, root)).toBe(true)
  })

  it('is false for a collapsed selection in a source cell', () => {
    const root = mountGrid()
    const selection = select(pieceText(root, 'AAA'), 1, pieceText(root, 'AAA'), 1)

    expect(isSourceColumnSelecting(selection, root)).toBe(false)
  })

  it('is false when the anchor is in the translation column', () => {
    const root = mountGrid()
    const tgt = root.querySelector('.cell-tgt')!.firstChild as Text
    const selection = select(tgt, 0, pieceText(root, 'CCC'), 1)

    expect(isSourceColumnSelecting(selection, root)).toBe(false)
  })

  it('is false when the anchor is outside the grid', () => {
    const root = mountGrid()
    const outside = document.createElement('p')
    outside.textContent = 'outside'
    document.body.appendChild(outside)
    const selection = select(outside.firstChild as Text, 0, pieceText(root, 'CCC'), 1)

    expect(isSourceColumnSelecting(selection, root)).toBe(false)
  })
})
