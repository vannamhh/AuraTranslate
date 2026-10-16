import { describe, expect, it } from 'vitest'
import { readingOriginLabel } from '../../src/modes/readingOrigin'

const counts = (self_count: number, other_count: number, bilingual_import_count: number) => ({
  self_count,
  other_count,
  bilingual_import_count,
})

describe('readingOriginLabel (FR151)', () => {
  it.each([
    ['only self', counts(3, 0, 0), 'self'],
    ['only other', counts(0, 2, 0), 'other'],
    ['only bilingual import', counts(0, 0, 5), 'bilingual_import'],
    ['self and other', counts(1, 1, 0), 'mixed'],
    ['self and bilingual import', counts(4, 0, 1), 'mixed'],
    ['other and bilingual import', counts(0, 7, 2), 'mixed'],
    ['all three', counts(1, 1, 1), 'mixed'],
  ] as const)('%s reads %s', (_name, input, expected) => {
    expect(readingOriginLabel(input)).toBe(expected)
  })

  it('a Chapter with no counted sentence reads none, never a label', () => {
    expect(readingOriginLabel(counts(0, 0, 0))).toBe('none')
  })

  it('the count, not the proportion, decides: one sentence of a second origin is already mixed', () => {
    expect(readingOriginLabel(counts(9999, 1, 0))).toBe('mixed')
  })
})
