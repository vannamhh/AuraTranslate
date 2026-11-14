import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

describe('webview spellcheck', () => {
  it('the document root turns spellcheck off so every input inherits it', () => {
    const html = readFileSync(resolve(process.cwd(), 'index.html'), 'utf8')
    const doc = new DOMParser().parseFromString(html, 'text/html')
    expect(doc.documentElement.getAttribute('spellcheck')).toBe('false')
  })

  it('no component opts back into spellcheck', () => {
    const sources = import.meta.glob<string>('/src/**/*.vue', { query: '?raw', import: 'default', eager: true })
    const optIns = Object.entries(sources)
      .filter(([, text]) => /\bspellcheck\s*=\s*"true"|:spellcheck\s*=/.test(text))
      .map(([file]) => file)
    expect(Object.keys(sources).length).toBeGreaterThan(0)
    expect(optIns).toEqual([])
  })
})
