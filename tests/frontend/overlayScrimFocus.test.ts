import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

function rootTag(source: string): string {
  const template = source.slice(source.indexOf('<template>')).replace(/<!--[\s\S]*?-->/g, '')
  return /<div\b[^>]*>/.exec(template)?.[0] ?? ''
}

function keepsFocusOnScrimPress(source: string): boolean {
  return /\s@mousedown\.self\.prevent[\s>]/.test(rootTag(source))
}

describe('every overlay root keeps focus when its scrim is pressed', () => {
  const dir = join(process.cwd(), 'src')
  const overlays = readdirSync(dir).filter((f) => f.endsWith('Overlay.vue'))

  it('has overlays to check', () => {
    expect(overlays.length).toBeGreaterThan(20)
  })

  it('reads the root element only, and ignores the modifier inside a comment or a child', () => {
    expect(keepsFocusOnScrimPress('<template>\n  <div class="x" @mousedown.self.prevent @keydown.esc="a()">')).toBe(true)
    expect(keepsFocusOnScrimPress('<template>\n  <div\n    class="x"\n    @mousedown.self.prevent\n  >')).toBe(true)
    expect(keepsFocusOnScrimPress('<template>\n  <!-- <div @mousedown.self.prevent> -->\n  <div class="x">')).toBe(false)
    expect(keepsFocusOnScrimPress('<template>\n  <div class="x">\n    <div @mousedown.self.prevent>')).toBe(false)
  })

  it('each *Overlay.vue root carries @mousedown.self.prevent', () => {
    const missing = overlays.filter((f) => !keepsFocusOnScrimPress(readFileSync(join(dir, f), 'utf8')))
    expect(missing).toEqual([])
  })
})
