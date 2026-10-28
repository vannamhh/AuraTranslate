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
    for (const selector of ['.ui-btn:focus-visible', '.ui-icon-btn:focus-visible', '.ui-field:focus-visible', '.ui-chip:focus-visible', '.ui-chip:has(input:focus-visible)', '.ui-check input:focus-visible', '.ui-tab:focus-visible', '.ui-menu-item:focus-visible']) {
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

  it('a segmented control is 28px tall including its border, so it lines up with md buttons and fields', () => {
    expect(declarations('.ui-seg')).toMatch(/(^|;)\s*height:\s*28px/)
    expect(declarations('.ui-seg__item')).not.toMatch(/(^|;)\s*height:/)
    expect(declarations('.ui-btn')).toMatch(/(^|;)\s*height:\s*28px/)
    expect(declarations('.ui-field')).toMatch(/(^|;)\s*height:\s*28px/)
  })

  it('a segment zeroes its border width, so the locked rule\'s dashed style cannot draw a medium-width left border on the first segment', () => {
    expect(declarations('.ui-seg__item')).toMatch(/(^|;)\s*border:\s*0\s*(;|$)/)
  })

  it('an sm segmented control is 24px tall including its border, the same tier as sm buttons', () => {
    expect(declarations('.ui-seg--sm')).toMatch(/(^|;)\s*height:\s*24px/)
    expect(declarations('.ui-btn--sm')).toMatch(/(^|;)\s*height:\s*24px/)
  })

  it('cover buttons take the Library mockup fills: surface with primary text, or primary with on-primary text', () => {
    const cover = declarations('.ui-btn--cover')
    expect(cover).toMatch(/background:\s*var\(--color-surface\)/)
    expect(cover).toMatch(/(^|;)\s*color:\s*var\(--color-primary\)/)
    const plain = declarations('.ui-btn--cover-plain')
    expect(plain).toMatch(/background:\s*var\(--color-surface\)/)
    expect(plain).toMatch(/(^|;)\s*color:\s*var\(--color-on-surface-variant\)/)
    const on = declarations('.ui-btn--cover-on')
    expect(on).toMatch(/background:\s*var\(--color-primary\)/)
    expect(on).toMatch(/(^|;)\s*color:\s*var\(--color-on-primary\)/)
  })

  it('cover buttons keep their fill when pressed or locked, so the label never sinks into the cover', () => {
    for (const cls of ['.ui-btn--cover', '.ui-btn--cover-plain', '.ui-btn--cover-on']) {
      for (const state of [':active', ':disabled']) {
        const body = declarations(cls + state)
        expect(body, cls + state).toMatch(/background:\s*var\(--color-(surface|primary)\)/)
        expect(body, cls + state).not.toMatch(/dashed/)
      }
      expect(CSS.indexOf(`${cls}:disabled`), `${cls}:disabled after .ui-btn:disabled`).toBeGreaterThan(CSS.indexOf('.ui-btn:disabled'))
    }
    expect(declarations('.ui-btn--cover-on:focus-visible')).toMatch(/outline:\s*2px solid var\(--color-on-cover\)/)
  })

  it('an md chip is 28px tall, the same tier as md buttons and fields', () => {
    expect(declarations('.ui-chip--md')).toMatch(/(^|;)\s*height:\s*28px/)
  })

  it('locked filters wrapped in a label take the dashed locked shape', () => {
    expect(declarations('.ui-chip:has(input:disabled)')).toMatch(/border-style:\s*dashed/)
    expect(declarations('.ui-seg__item:has(input:disabled)')).toMatch(/border-left-style:\s*dashed/)
    expect(declarations('.ui-field:disabled')).toMatch(/border-style:\s*dashed/)
    expect(declarations('.ui-btn:disabled')).toMatch(/border:\s*1px dashed/)
  })

  it('a label wrapping a checked hidden radio takes the chosen fill', () => {
    expect(declarations('.ui-chip:has(input:checked)')).toMatch(/background:\s*var\(--color-on-surface\)/)
    expect(declarations('.ui-seg__item:has(input:checked)')).toMatch(/background:\s*var\(--color-on-surface\)/)
  })

  it('a locked switch takes the dashed track and ignores hover', () => {
    expect(declarations('.ui-switch:disabled > .ui-switch__track')).toMatch(/border:\s*1px dashed/)
    expect(declarations('.ui-switch:disabled:hover > .ui-switch__track')).toMatch(/border:\s*1px dashed/)
    expect(declarations('.ui-switch:disabled')).toMatch(/cursor:\s*default/)
  })

  it('a selected tab takes the on-surface underline and strong weight, and focus recolours the underline to primary after hover', () => {
    const selected = declarations(".ui-tab[aria-selected='true']")
    expect(selected).toMatch(/border-bottom-color:\s*var\(--color-on-surface\)/)
    expect(selected).toMatch(/font-weight:\s*var\(--weight-ui-md-strong\)/)
    const focus = declarations('.ui-tab:focus-visible')
    expect(focus).toMatch(/outline:\s*none/)
    expect(focus).toMatch(/border-bottom-color:\s*var\(--color-primary\)/)
    expect(CSS.indexOf('.ui-tab:focus-visible')).toBeGreaterThan(CSS.indexOf('.ui-tab:hover'))
    expect(CSS.indexOf('.ui-tab:focus-visible')).toBeGreaterThan(CSS.indexOf(".ui-tab[aria-selected='true']"))
  })

  it('the prompt editor mirror declares the same box and font as textarea.ui-field', () => {
    const overlay = readFileSync(join(__dirname, '../../src/PromptLibraryOverlay.vue'), 'utf8')
    const mirror = overlay.match(/\.pl-ta-mirror\s*\{([^{}]*)\}/)?.[1] ?? ''
    expect(mirror, '.pl-ta-mirror rule').not.toBe('')
    const both = `${declarations('.ui-field')};${declarations('textarea.ui-field')}`
    const value = (body: string, prop: string): string | undefined =>
      [...body.matchAll(new RegExp(`(?:^|[;\\s])${prop}\\s*:\\s*([^;]+)`, 'g'))].map((m) => m[1]!.trim().replace(/\s+/g, ' ')).at(-1)
    for (const prop of ['padding', 'font-family', 'font-size', 'line-height']) {
      const expected = value(both, prop)
      expect(expected, `ui-field ${prop}`).toBeDefined()
      const actual = value(mirror, prop)
      expect(actual, `.pl-ta-mirror ${prop}`).toBe(expected)
    }
    expect(value(both, 'border')).toMatch(/^1px /)
    expect(value(mirror, 'border')).toMatch(/^1px /)
  })

  it('the prompt editor stays transparent over its mirror even when locked, outranking .ui-field:disabled', () => {
    const overlay = readFileSync(join(__dirname, '../../src/PromptLibraryOverlay.vue'), 'utf8').replace(/\/\*[\s\S]*?\*\//g, '')
    const transparent = rulesOf(overlay).filter(
      (rule) => rule.selectors.some((sel) => sel.includes('.pl-ta-input')) && /(^|[;\s])color:\s*transparent/.test(rule.body),
    )
    expect(transparent.length, 'no rule makes .pl-ta-input text transparent').toBeGreaterThan(0)
    const lockedClasses = ('.ui-field:disabled'.match(/[.:][\w-]+/g) ?? []).length
    for (const rule of transparent) {
      for (const sel of rule.selectors.filter((s) => s.includes('.pl-ta-input'))) {
        expect((sel.match(/[.:][\w-]+/g) ?? []).length + 1, `${sel} must outrank .ui-field:disabled with the scoped attribute`).toBeGreaterThan(lockedClasses)
        expect(sel, sel).not.toMatch(/:(disabled|hover|focus)/)
      }
    }
    expect(declarations('.ui-field:disabled')).toMatch(/(^|[;\s])color:/)
  })
})
