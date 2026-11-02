import { describe, expect, it } from 'vitest'

import {
  focusedImportPreviewBlockRowPosition,
  groupImportPreviewBlockRows,
} from '../../src/importPreviewState'

const kept = (...flags: boolean[]) => flags.map((k) => ({ kept: k }))

describe('groupImportPreviewBlockRows', () => {
  it('turns every run of dropped blocks into one band, even a single block', () => {
    const rows = groupImportPreviewBlockRows(kept(false, true, false, false, true, false), new Set())
    expect(rows).toEqual([
      { kind: 'band', start: 0, end: 0, open: false },
      { kind: 'block', index: 1 },
      { kind: 'band', start: 2, end: 3, open: false },
      { kind: 'block', index: 4 },
      { kind: 'band', start: 5, end: 5, open: false },
    ])
  })

  it('lists the blocks of an open band after its header', () => {
    const rows = groupImportPreviewBlockRows(kept(true, false, false), new Set([1, 2]))
    expect(rows).toEqual([
      { kind: 'block', index: 0 },
      { kind: 'band', start: 1, end: 2, open: true },
      { kind: 'block', index: 1 },
      { kind: 'block', index: 2 },
    ])
  })

  it('splits a band into two when a block in the middle is kept', () => {
    const blocks = Array.from({ length: 98 }, () => ({ kept: false }))
    expect(groupImportPreviewBlockRows(blocks, new Set())).toEqual([{ kind: 'band', start: 0, end: 97, open: false }])
    blocks[39] = { kept: true }
    const rows = groupImportPreviewBlockRows(blocks, new Set())
    expect(rows).toEqual([
      { kind: 'band', start: 0, end: 38, open: false },
      { kind: 'block', index: 39 },
      { kind: 'band', start: 40, end: 97, open: false },
    ])
    expect(rows.filter((r) => r.kind === 'band').map((r) => r.end - r.start + 1)).toEqual([39, 58])
  })

  it('has no band when nothing is dropped', () => {
    expect(groupImportPreviewBlockRows(kept(true, true), new Set()).every((r) => r.kind === 'block')).toBe(true)
  })
})

describe('focusedImportPreviewBlockRowPosition', () => {
  const blocks = kept(true, false, false, true)

  it('owns every index inside a collapsed band', () => {
    const rows = groupImportPreviewBlockRows(blocks, new Set())
    expect(focusedImportPreviewBlockRowPosition(rows, 0, false)).toBe(0)
    expect(focusedImportPreviewBlockRowPosition(rows, 1, true)).toBe(1)
    expect(focusedImportPreviewBlockRowPosition(rows, 2, false)).toBe(1)
    expect(focusedImportPreviewBlockRowPosition(rows, 3, false)).toBe(2)
  })

  it('tells the header of an open band from its first block', () => {
    const rows = groupImportPreviewBlockRows(blocks, new Set([1, 2]))
    expect(focusedImportPreviewBlockRowPosition(rows, 1, true)).toBe(1)
    expect(focusedImportPreviewBlockRowPosition(rows, 1, false)).toBe(2)
  })
})
