import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

const CSS = readFileSync(join(__dirname, '../../src/tokens/controls.css'), 'utf8').replace(/\/\*[\s\S]*?\*\//g, '')

function rulesOf(css: string): { selectors: string[]; body: string }[] {
  return [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map((m) => ({
    selectors: m[1]!.split(',').map((s) => s.trim()),
    body: m[2]!,
  }))
}

function forcedColorsBlock(): string {
  const start = CSS.indexOf('@media (forced-colors: active)')
  expect(start, 'controls.css has no forced-colors block').toBeGreaterThanOrEqual(0)
  return CSS.slice(CSS.indexOf('{', start) + 1)
}

const TOP_LEVEL = rulesOf(CSS.slice(0, CSS.indexOf('@media (forced-colors: active)')))

function declarations(selector: string): string {
  const hits = TOP_LEVEL.filter((rule) => rule.selectors.includes(selector))
  expect(hits.length, `no top-level rule for ${selector}`).toBeGreaterThan(0)
  return hits.map((rule) => rule.body).join(';')
}

describe('controls.css follows DESIGN.md §Điều khiển', () => {
  it('keyboard focus recolours the border of bordered controls and keeps the ring on Primary', () => {
    for (const selector of ['.ui-btn:focus-visible', '.ui-icon-btn:focus-visible', '.ui-field:focus-visible', '.ui-chip:focus-visible', '.ui-check input:focus-visible']) {
      const body = declarations(selector)
      expect(body, selector).toMatch(/outline:\s*none/)
      expect(body, selector).toMatch(/border-color:\s*var\(--color-primary\)/)
    }
    expect(declarations('.ui-btn--primary:focus-visible')).toMatch(/outline:\s*2px solid var\(--color-primary\)/)
  })

  it('Danger hover recolours only the border, to on-surface-variant, and keeps the error text', () => {
    expect(declarations('.ui-btn:hover')).toMatch(/border-color:\s*var\(--color-on-surface-variant\)/)
    const hoverDanger = TOP_LEVEL.filter((rule) =>
      rule.selectors.some((sel) => /\.ui-btn--danger/.test(sel) && /:hover/.test(sel)),
    )
    for (const rule of hoverDanger) expect(rule.body).not.toMatch(/(^|[;\s])(border-color|border|color)\s*:/)
  })

  it('focus rules come after hover rules, so focus wins', () => {
    for (const control of ['.ui-btn', '.ui-field', '.ui-chip', '.ui-icon-btn']) {
      const hover = CSS.indexOf(`${control}:hover`)
      const focus = CSS.indexOf(`${control}:focus-visible`)
      expect(hover, `${control}:hover`).toBeGreaterThanOrEqual(0)
      expect(focus, `${control}:focus-visible`).toBeGreaterThan(hover)
    }
  })

  it('forced-colors mode draws a Highlight ring on every focusable control', () => {
    const block = forcedColorsBlock()
    expect(block).toMatch(/outline:\s*2px solid Highlight/)
    for (const selector of ['.ui-btn:focus-visible', '.ui-icon-btn:focus-visible', '.ui-field:focus-visible', '.ui-chip:focus-visible', '.ui-chip:has(input:focus-visible)', '.ui-check input:focus-visible', '.ui-menu-item:focus-visible']) {
      expect(block, selector).toContain(selector)
    }
  })

  it('a bare checkbox keeps a 24×24px hit area around its 14px box', () => {
    const bare = declarations('.ui-check--bare')
    expect(bare).toMatch(/width:\s*24px/)
    expect(bare).toMatch(/height:\s*24px/)
    const box = declarations('.ui-check input')
    expect(box).toMatch(/width:\s*14px/)
    expect(box).toMatch(/height:\s*14px/)
  })

  it('locked filters wrapped in a label take the dashed locked shape', () => {
    expect(declarations('.ui-chip:has(input:disabled)')).toMatch(/border-style:\s*dashed/)
    expect(declarations('.ui-seg__item:has(input:disabled)')).toMatch(/border-left-style:\s*dashed/)
    expect(declarations('.ui-field:disabled')).toMatch(/border-style:\s*dashed/)
  })
})
