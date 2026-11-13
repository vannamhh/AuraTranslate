/**
 * One plain click on the source column looks up the phrase under it and paints it, in a real
 * WKWebView, in all three views (Original, Han Viet switch, Han Viet parallel).
 *
 *     npm run test:e2e -- --spec e2e/specs/lookup-by-click.e2e.mjs
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { realClick, realClickAt } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { ensureFullLayoutTier } from '../support/layoutTier.mjs'

const SOURCE = '他拉小提琴。'
const PHRASE = '小提琴'
const MULTI = '一。二。三。四。'
const HEADWORD = '.lookup-headword'

async function headword() {
  return await browser.execute((sel) => document.querySelector(sel)?.textContent?.trim() ?? null, HEADWORD)
}

async function paintedText() {
  return await browser.execute(() => {
    const marked = [...document.querySelectorAll('[data-lookup-active]')]
    if (marked.length === 0) return null
    return marked
      .map((el) => {
        const copy = el.cloneNode(true)
        for (const rt of copy.querySelectorAll('rt')) rt.remove()
        return copy.textContent
      })
      .join('')
  })
}

async function waitForPhrase(expected, what) {
  await browser.waitUntil(async () => (await headword()) === expected, {
    timeout: 15_000,
    timeoutMsg:
      `${what}: Panel Lookup did not show ${expected} (last: ${await headword()}, painted: ${await paintedText()}, ` +
      `lookup panel mounted: ${await browser.execute(() => document.querySelector('.lookup-headword, [data-panel="lookup"]') !== null)})`,
  })
  await browser.waitUntil(async () => (await paintedText()) === expected, {
    timeout: 5_000,
    timeoutMsg: `${what}: painted text is ${await paintedText()}, expected ${expected}`,
  })
}

/** Viewport centre of one source character of the first source cell, found by its code-point index. */
async function charCentre(index) {
  return await browser.execute((cp) => {
    const cell = document.querySelector('[data-col="src"]')
    const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
    let seen = 0
    for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
      if (node.parentElement.closest('rt') !== null) continue
      const chars = [...node.data]
      if (cp < seen + chars.length) {
        const from = chars.slice(0, cp - seen).join('').length
        const range = document.createRange()
        range.setStart(node, from)
        range.setEnd(node, from + chars[cp - seen].length)
        const rect = range.getBoundingClientRect()
        return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 }
      }
      seen += chars.length
    }
    throw new Error(`no source character at ${cp}`)
  }, index)
}

async function openGrid(source, rows) {
  await openWorkspaceWithWork(`e2e-lookup-click-${Date.now()}`, source)
  await ensureFullLayoutTier()
  await waitForGridRows(rows, { what: 'Work for lookup by click' })
}

async function clickChar(index) {
  const at = await charCentre(index)
  await realClickAt(at.x, at.y)
}

async function clickPhrase() {
  await clickChar(SOURCE.indexOf('提'))
  await waitForPhrase(PHRASE, 'original view')
}

describe('lookup by one click', () => {
  it('Original view: a click on one character looks up its word and paints exactly it', async () => {
    await openGrid(SOURCE, 1)
    await clickPhrase()
    expect(await browser.execute(() => window.getSelection()?.toString() ?? '')).toBe('')
  })

  it('a click on punctuation keeps the previous result and paint', async () => {
    await openGrid(SOURCE, 1)
    await clickPhrase()
    await clickChar(SOURCE.indexOf('。'))
    await browser.pause(500)
    expect(await headword()).toBe(PHRASE)
    expect(await paintedText()).toBe(PHRASE)
  })

  it('Original view: a click on a second word paints only that word', async () => {
    await openGrid(SOURCE, 1)
    await clickPhrase()
    await clickChar(SOURCE.indexOf('拉'))
    await browser.waitUntil(async () => (await headword()) !== PHRASE, { timeout: 15_000, timeoutMsg: 'second word was not looked up' })
    const word = await headword()
    await browser.waitUntil(async () => (await paintedText()) === word, {
      timeout: 5_000,
      timeoutMsg: `painted text is ${await paintedText()}, expected only ${word}`,
    })
  })

  it('a second click inside the same phrase does not look it up again', async () => {
    await openGrid(SOURCE, 1)
    await clickPhrase()
    await browser.execute(() => {
      window.__headwordChanges = 0
      new MutationObserver(() => { window.__headwordChanges += 1 }).observe(
        document.querySelector('.lookup-headword').parentElement,
        { childList: true, subtree: true, characterData: true },
      )
    })
    await clickChar(SOURCE.indexOf('琴'))
    await browser.pause(500)
    expect(await headword()).toBe(PHRASE)
    expect(await paintedText()).toBe(PHRASE)
    expect(await browser.execute(() => window.__headwordChanges)).toBe(0)
  })

  it('Han Viet switch view: a click on a block paints exactly its readings and looks up its Han characters', async () => {
    await openGrid(SOURCE, 1)
    await realClick(await $('#grid-tab-han-viet'))
    await $('[data-col="src"] .hv-word').waitForDisplayed({ timeout: 15_000 })
    const block = await browser.execute(() => {
      const words = [...document.querySelectorAll('[data-col="src"] .hv-word')]
      const index = Math.max(0, words.findIndex((el) => el.querySelectorAll('.hv-syl').length > 1))
      const el = words[index]
      const rect = el.getBoundingClientRect()
      return {
        start: Number(el.getAttribute('data-src-start')),
        length: el.querySelectorAll('.hv-syl').length,
        readings: el.textContent,
        x: rect.left + rect.width / 2,
        y: rect.top + rect.height / 2,
      }
    })
    const han = [...SOURCE].slice(block.start, block.start + block.length).join('')
    await realClickAt(block.x, block.y)
    await browser.waitUntil(async () => (await headword()) === han, {
      timeout: 15_000,
      timeoutMsg: `switch view: headword ${await headword()}, expected ${han}`,
    })
    await browser.waitUntil(async () => (await paintedText()) !== null, { timeout: 5_000 })
    expect(await headword()).toBe(han)
    expect(await paintedText()).toBe(block.readings)
  })

  it('Han Viet parallel view: a click on a block paints exactly its Han base, without the reading', async () => {
    await openGrid(SOURCE, 1)
    await realClick(await $('#grid-tab-han-viet'))
    await realClick(await $('.view-toggle'))
    await $('[data-col="src"] .hv-unit').waitForDisplayed({ timeout: 15_000 })
    const block = await browser.execute(() => {
      const el = document.querySelector('[data-col="src"] .hv-unit')
      const base = [...el.querySelector('ruby').childNodes].filter((n) => n instanceof Text).map((n) => n.data).join('')
      const rect = el.getBoundingClientRect()
      return { base, reading: el.querySelector('rt').textContent, x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 }
    })
    await realClickAt(block.x, block.y)
    await browser.waitUntil(async () => (await paintedText()) !== null, { timeout: 15_000 })
    expect(await paintedText()).toBe(block.base)
    expect(await paintedText()).not.toContain(block.reading)
    expect(await headword()).toBe(block.base)
  })

  it('Han Viet switch view: clicking block A then block B marks only B', async () => {
    await openGrid(SOURCE, 1)
    await realClick(await $('#grid-tab-han-viet'))
    await $('[data-col="src"] .hv-word').waitForDisplayed({ timeout: 15_000 })
    const centres = await browser.execute(() =>
      [...document.querySelectorAll('[data-col="src"] .hv-word')].slice(0, 2).map((el) => {
        const rect = el.getBoundingClientRect()
        return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 }
      }),
    )
    expect(centres.length).toBe(2)
    await realClickAt(centres[0].x, centres[0].y)
    await browser.waitUntil(async () => (await browser.execute(() => document.querySelectorAll('[data-lookup-active]').length)) > 0, { timeout: 15_000 })
    await realClickAt(centres[1].x, centres[1].y)
    await browser.waitUntil(
      async () => (await browser.execute(() => {
        const marked = [...document.querySelectorAll('[data-lookup-active]')]
        const words = [...document.querySelectorAll('[data-col="src"] .hv-word')]
        return marked.length === 1 && marked[0] === words[1]
      })),
      { timeout: 15_000, timeoutMsg: 'block A kept its mark after clicking block B' },
    )
  })

  it('Han Viet parallel view: a click in the middle of the reading looks up and colours the Han base only', async () => {
    await openGrid(SOURCE, 1)
    await realClick(await $('#grid-tab-han-viet'))
    await realClick(await $('.view-toggle'))
    await $('[data-col="src"] .hv-unit rt').waitForDisplayed({ timeout: 15_000 })
    const block = await browser.execute(() => {
      const el = document.querySelector('[data-col="src"] .hv-unit')
      const base = [...el.querySelector('ruby').childNodes].filter((n) => n instanceof Text).map((n) => n.data).join('')
      const rect = el.querySelector('rt').getBoundingClientRect()
      return { base, x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 }
    })
    await realClickAt(block.x, block.y)
    await browser.waitUntil(async () => (await headword()) === block.base, { timeout: 15_000, timeoutMsg: `the click on the reading did not look up ${block.base}` })
    const colours = await browser.execute(() => {
      const el = document.querySelector('[data-col="src"] [data-lookup-active]')
      return { unit: getComputedStyle(el).color, rt: getComputedStyle(el.querySelector('rt')).color }
    })
    expect(colours.unit).not.toBe(colours.rt)
  })

  it('Han Viet parallel view: a click on the left part of a Han character looks up that block, not the previous one', async () => {
    await openGrid(SOURCE, 1)
    await realClick(await $('#grid-tab-han-viet'))
    await realClick(await $('.view-toggle'))
    await $('[data-col="src"] .hv-unit rt').waitForDisplayed({ timeout: 15_000 })
    const block = await browser.execute(() => {
      const el = document.querySelectorAll('[data-col="src"] .hv-unit')[1]
      const text = el.querySelector('ruby').firstChild
      const glyph = document.createRange()
      glyph.setStart(text, 0)
      glyph.setEnd(text, 1)
      const rect = glyph.getBoundingClientRect()
      return { base: text.data, x: rect.left + rect.width / 4, y: rect.top + rect.height / 2 }
    })
    await realClickAt(block.x, block.y)
    await browser.waitUntil(async () => (await headword()) === block.base, { timeout: 15_000, timeoutMsg: 'the click on the left part of the glyph did not look up its block' })
    expect(await paintedText()).toBe(block.base)
  })

  it('Han Viet switch view: a click just inside the left edge of a block looks up that block', async () => {
    await openGrid(SOURCE, 1)
    await realClick(await $('#grid-tab-han-viet'))
    await $('[data-col="src"] .hv-word').waitForDisplayed({ timeout: 15_000 })
    const block = await browser.execute(() => {
      const el = document.querySelectorAll('[data-col="src"] .hv-word')[1]
      const rect = el.getBoundingClientRect()
      return { readings: el.textContent, x: rect.left + 2, y: rect.top + rect.height / 2 }
    })
    await realClickAt(block.x, block.y)
    await browser.waitUntil(async () => (await paintedText()) === block.readings, { timeout: 15_000, timeoutMsg: 'the click on the left edge of the block did not paint it' })
  })

  it('toggling the Han Viet view after a click leaves no lookup highlight', async () => {
    await openGrid(SOURCE, 1)
    await clickPhrase()
    await realClick(await $('#grid-tab-han-viet'))
    await $('[data-col="src"] .hv-word').waitForDisplayed({ timeout: 15_000 })
    await browser.waitUntil(async () => (await paintedText()) === null, {
      timeout: 5_000,
      timeoutMsg: `highlight survived the view change: ${await paintedText()}`,
    })
  })

  it('changing Chapter after a click leaves no lookup highlight', async () => {
    await openGrid(MULTI, 4)
    const ids = await browser.execute(() =>
      [...document.querySelectorAll('[data-col="src"]')].map((el) => el.getAttribute('data-segment-id')),
    )
    await browser.execute(async (sid) => {
      await window.__TAURI_INTERNALS__.invoke('split_chapter_at_segment', { segmentId: sid })
    }, Number(ids[2]))
    await clickChar(0)
    await browser.waitUntil(async () => (await paintedText()) !== null, { timeout: 15_000 })
    await realClick(await $('[data-chapter-next]'))
    await browser.waitUntil(async () => (await paintedText()) === null, {
      timeout: 15_000,
      timeoutMsg: `highlight survived the chapter change: ${await paintedText()}`,
    })
  })

  it('a drag selection inside a source cell looks up the dragged text and leaves no lookup highlight', async () => {
    await openGrid(SOURCE, 1)
    // The driver's synthetic pointer events never move the selection, so the dragged range is built directly.
    const dragged = await browser.execute((start, end) => {
      const cell = document.querySelector('[data-col="src"]')
      const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
      for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
        const at = node.data.indexOf(start)
        if (at === -1) continue
        const range = document.createRange()
        range.setStart(node, at)
        range.setEnd(node, node.data.indexOf(end) + end.length)
        const selection = window.getSelection()
        selection.removeAllRanges()
        selection.addRange(range)
        return selection.toString()
      }
      return ''
    }, '小', '琴')
    expect(dragged).toBe('小提琴')
    const to = await charCentre(SOURCE.indexOf('琴'))
    await realClickAt(to.x, to.y)
    await browser.waitUntil(async () => (await headword()) === dragged.trim(), {
      timeout: 15_000,
      timeoutMsg: `Panel Lookup shows ${await headword()}, dragged ${dragged}`,
    })
    expect(await paintedText()).toBeNull()
  })
})
