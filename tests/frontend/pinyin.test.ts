import { describe, expect, it } from 'vitest'
import { readingKey, sharedReadingKey, sharedReadingText, toneMarkedReading } from '../../src/panels/pinyin'

describe('toneMarkedReading', () => {
  it.each([
    ['da3', 'dǎ'],
    ['lu:4', 'lǜ'],
    ['Zhong1 guo2', 'Zhōng guó'],
    ['r5', 'r'],
    ['zou3 lang2', 'zǒu láng'],
    ['hui4', 'huì'],
    ['jiu3', 'jiǔ'],
    ['gou3', 'gǒu'],
    ['shān (shan¹)', 'shān (shan¹)'],
    ['zǒuláng', 'zǒuláng'],
    ['sơn', 'sơn'],
  ])('%s -> %s', (raw, expected) => {
    expect(toneMarkedReading(raw)).toBe(expected)
  })
})

describe('readingKey', () => {
  it('matches across numeric, spaced and toned forms', () => {
    expect(readingKey('zou3 lang2')).toBe(readingKey('zǒuláng'))
    expect(readingKey('zou3 lang2')).toBe('zǒuláng')
    expect(readingKey('Shan1')).toBe(readingKey('shān (shan¹)'))
  })
  it('keeps different tones apart', () => {
    expect(readingKey('da2')).not.toBe(readingKey('da3'))
  })
})

describe('sharedReadingKey', () => {
  it('ignores null readings and returns the common key', () => {
    expect(sharedReadingKey([null, 'zou3 lang2', 'zǒuláng'])).toBe('zǒuláng')
  })
  it('is null when readings differ or none exist', () => {
    expect(sharedReadingKey(['da2', 'da3', 'dǎ'])).toBeNull()
    expect(sharedReadingKey([null, null])).toBeNull()
    expect(sharedReadingKey([])).toBeNull()
  })
})

describe('sharedReadingText', () => {
  it('shows the syllable-spaced form when any source carries one', () => {
    expect(sharedReadingText([null, 'zou3 lang2', 'zǒuláng'])).toBe('zǒu láng')
    expect(sharedReadingText(['zǒuláng', 'zou3 lang2'])).toBe('zǒu láng')
  })
  it('falls back to the key when no source spaces its syllables, and is null when readings differ', () => {
    expect(sharedReadingText(['Shan1', 'shān (shan¹)'])).toBe('shān')
    expect(sharedReadingText(['da2', 'da3'])).toBeNull()
  })
})
