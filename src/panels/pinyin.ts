const NUMERIC_SYLLABLE = /^([A-Za-z]+:?[A-Za-z]*)([1-5])$/
const TONE_MARKS: Record<string, string> = { '1': '\u0304', '2': '\u0301', '3': '\u030C', '4': '\u0300' }
const TRAILING_PAREN = /\s*\([^)]*\)\s*$/

function markSyllable(letters: string, tone: string): string {
  const base = letters.replace(/u:/g, 'ü').replace(/U:/g, 'Ü')
  const mark = TONE_MARKS[tone]
  if (!mark) return base
  const lower = base.toLowerCase()
  let at = lower.search(/[ae]/)
  if (at < 0) at = lower.includes('ou') ? lower.indexOf('o') : -1
  if (at < 0) {
    for (let i = lower.length - 1; i >= 0; i--) {
      if ('aeiouü'.includes(lower[i]!)) {
        at = i
        break
      }
    }
  }
  if (at < 0) return base
  return (base.slice(0, at + 1) + mark + base.slice(at + 1)).normalize('NFC')
}

/** Numeric pinyin (`da3`, `lu:4`, `Zhong1 guo2`) to tone marks; any other form is returned as-is. */
export function toneMarkedReading(raw: string): string {
  return raw
    .split(/(\s+)/)
    .map((token) => {
      const m = NUMERIC_SYLLABLE.exec(token)
      return m ? markSyllable(m[1]!, m[2]!) : token
    })
    .join('')
}

/** Comparison key: tone-marked, trailing `(…)` dropped, whitespace dropped, lowercased. */
export function readingKey(raw: string): string {
  return toneMarkedReading(raw).replace(TRAILING_PAREN, '').replace(/\s+/g, '').toLowerCase().normalize('NFC')
}

/** The single key shared by every non-null reading, or `null` when none exists or two differ. */
export function sharedReadingKey(readings: readonly (string | null)[]): string | null {
  let shared: string | null = null
  for (const r of readings) {
    if (r === null) continue
    const key = readingKey(r)
    if (shared === null) shared = key
    else if (shared !== key) return null
  }
  return shared
}
