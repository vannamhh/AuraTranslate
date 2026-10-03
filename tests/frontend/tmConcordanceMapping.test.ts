/** Vietnamese selection to Chinese span through confirmed Glossary spans. */
import { describe, expect, it } from 'vitest'
import { sliceByCodePoint, sourceSpanForVietnameseSelection } from '../../src/panels/concordanceSourceMapping'
import type { SegmentTermSpan } from '../../src/panels/glossaryMarksMap'

function span(patch: Partial<SegmentTermSpan>): SegmentTermSpan {
  return {
    start: 0,
    end: 1,
    isConfirmed: true,
    translation: 'x',
    id: 1,
    sourceTerm: 'x',
    tier: 'work',
    hanVietSuggestion: null,
    hanVietStatus: 'not_requested',
    occurrenceCount: null,
    ...patch,
  }
}

describe('sourceSpanForVietnameseSelection', () => {
  const first = span({ id: 1, start: 0, end: 1, translation: 'Hắn' })
  const second = span({ id: 2, start: 2, end: 4, translation: 'Sư phụ' })

  it('picks the mark whose translation equals the selection, not the first mark', () => {
    expect(sourceSpanForVietnameseSelection('sư phụ', [first, second])).toEqual({ start: 2, end: 4 })
  })

  it('matches NFC, trimmed and case-insensitively', () => {
    const composed = span({ id: 3, start: 5, end: 7, translation: 'Sư phụ'.normalize('NFC') })
    const selection = `  ${'SƯ PHỤ'.normalize('NFD')} `
    expect(sourceSpanForVietnameseSelection(selection, [composed])).toEqual({ start: 5, end: 7 })
  })

  it('takes the earliest of several marks with the same translation', () => {
    const later = span({ id: 4, start: 9, end: 11, translation: 'Sư phụ' })
    expect(sourceSpanForVietnameseSelection('Sư phụ', [later, second])).toEqual({ start: 2, end: 4 })
  })

  it('never matches an unconfirmed mark', () => {
    const pending = span({ isConfirmed: false, translation: null, start: 0, end: 2 })
    expect(sourceSpanForVietnameseSelection('Sư phụ', [pending])).toBeNull()
    expect(sourceSpanForVietnameseSelection('', [second])).toBeNull()
  })

  it('returns null when no mark translates to the selection', () => {
    expect(sourceSpanForVietnameseSelection('Thầy', [first, second])).toBeNull()
  })
})

describe('sliceByCodePoint', () => {
  it('counts code points, not UTF-16 units', () => {
    expect(sliceByCodePoint('𠀀他叫师父', 3, 5)).toBe('师父')
  })
})
