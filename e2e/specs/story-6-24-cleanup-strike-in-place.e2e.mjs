/**
 * Cleanup rule strike-through in place, in a real WKWebView against real Rust.
 *
 * A local `node:http` server serves one article of 40+ paragraphs; the rule pattern matches
 * the first paragraph, the second, and a paragraph that starts after 4 096 bytes of Chapter
 * text. A second case imports a `.txt` longer than 4 096 bytes on the file path.
 */
import { mkdtempSync, writeFileSync } from 'node:fs'
import { createServer } from 'node:http'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { realClick } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const URL_WORK = `e2e-624-url-${RUN_TAG}`
const TXT_WORK = `e2e-624-txt-${RUN_TAG}`
const SCREEN = '.ip-screen'
const PATTERN = 'ALPHAMARK'
const TICK = `.ip-screen .ip-cleanup-tick[aria-label*="${PATTERN}"]`
const TAIL_TOKEN = 'ZETATAILONLY'
const HEAD_TOKEN = 'OMEGAHEADONLY'

function filler(n) {
  return `Doan so ${n} cua bai viet la phan noi dung that, du dai de Readability giu lai trong ban chinh va du chu de nhieu khoi cung xep thanh mot cot dai can cuon.`
}

function page() {
  const paragraphs = []
  paragraphs.push(`Doan dau tien co ${PATTERN} nam o day va ${HEAD_TOKEN} chi xuat hien mot lan trong khoi nay, cung voi nhieu chu de vuot nguong do dai.`)
  paragraphs.push(`Doan thu hai cung chua ${PATTERN} de luat khop o khoi thu hai, them chu de vuot nguong do dai toi thieu cua Readability.`)
  for (let n = 3; n <= 44; n += 1) paragraphs.push(filler(n))
  paragraphs.push(`Doan cuoi nam sau cua so bon nghin chin muoi sau byte co ${PATTERN} va ${TAIL_TOKEN} chi mot lan, them chu de vuot nguong do dai.`)
  return `<html><head><title>bai dai</title></head><body><article><h1>bai dai</h1>\n${paragraphs
    .map((p) => `<p>${p}</p>`)
    .join('\n')}\n</article></body></html>`
}

let server
let port

async function ready() {
  await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
    timeout: 30_000,
    interval: 250,
  })
  await $('[data-library-import-open]').waitForDisplayed({ timeout: 30_000 })
}

async function struckCount(selector) {
  return browser.execute((sel) => document.querySelectorAll(sel).length, selector)
}

async function hintShown(piece) {
  return browser.execute((el) => {
    const hint = el.querySelector('.ip-cleanup-hint')
    if (hint === null) return null
    const style = getComputedStyle(hint)
    const rect = hint.getBoundingClientRect()
    return { shown: style.clipPath === 'none' && rect.width > 2 && rect.height > 2, text: hint.textContent }
  }, piece)
}

async function hoverRuleReveals() {
  return browser.execute(() => {
    for (const sheet of document.styleSheets) {
      for (const rule of sheet.cssRules) {
        if (!(rule instanceof CSSStyleRule)) continue
        if (rule.selectorText.includes('.ip-cleanup-struck:hover') && rule.selectorText.includes('.ip-cleanup-hint')) {
          return rule.style.getPropertyValue('clip-path') === 'none' && rule.style.getPropertyValue('width') === 'auto'
        }
      }
    }
    return false
  })
}

async function scroller() {
  return browser.execute(() => {
    let node = document.querySelector('.ip-screen .ip-main')
    while (node !== null) {
      const overflow = getComputedStyle(node).overflowY
      if ((overflow === 'auto' || overflow === 'scroll') && node.scrollHeight > node.clientHeight) {
        node.dataset.e2eScroller = '1'
        return true
      }
      node = node.parentElement
    }
    return false
  })
}

async function scrollTop() {
  return browser.execute(() => document.querySelector('[data-e2e-scroller="1"]')?.scrollTop ?? -1)
}

async function setScrollTop(value) {
  await browser.execute((v) => {
    document.querySelector('[data-e2e-scroller="1"]').scrollTop = v
  }, value)
  await browser.pause(400)
}

async function occurrences(token) {
  return browser.execute(
    (needle) => {
      const clone = document.querySelector('.ip-screen .ip-main').cloneNode(true)
      clone.querySelectorAll('.ip-cleanup-hint').forEach((el) => el.remove())
      return (clone.textContent ?? '').split(needle).length - 1
    },
    token,
  )
}

async function toggleRule() {
  const point = await browser.execute((sel) => {
    const rect = document.querySelector(sel).getBoundingClientRect()
    return { x: Math.round(rect.left + rect.width / 2), y: Math.round(rect.top + rect.height / 2) }
  }, TICK)
  await browser.action('pointer').move({ x: point.x, y: point.y }).down().pause(30).up().pause(100).perform()
}

async function addRule() {
  const input = await $(`${SCREEN} .ip-cleanup-add-pattern`)
  await input.waitForDisplayed({ timeout: 15_000 })
  await input.setValue(PATTERN)
  await realClick(await $(`${SCREEN} .ip-cleanup-add-submit`))
}

describe('cleanup rule strike-through in place', () => {
  before(async () => {
    server = createServer((_req, res) => {
      res.setHeader('Content-Type', 'text/html; charset=utf-8')
      res.end(page())
    })
    await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve))
    port = server.address().port
  })

  after(async () => {
    await new Promise((resolve) => server.close(resolve))
  })

  it('URL path: a match after 4 096 bytes is struck in its block, the label follows hover and the J/K cursor, text appears once, scrollTop survives a rule toggle', async () => {
    await ready()
    await realClick(await $('[data-library-import-open]'))
    await $('[data-library-url-screen-open]').waitForDisplayed({ timeout: 10_000 })
    await realClick(await $('[data-library-url-screen-open]'))
    const screen = await $('[data-library-url-import-screen]')
    await screen.waitForDisplayed({ timeout: 10_000 })
    await (await screen.$('textarea')).setValue(`http://127.0.0.1:${port}/a`)
    await (await screen.$('input[type="text"]')).setValue(URL_WORK)
    await realClick(await screen.$('[data-import-preview-open]'))
    await $(SCREEN).waitForDisplayed({ timeout: 60_000 })
    await browser.waitUntil(async () => (await $$(`${SCREEN} .ip-block`)).length >= 40, {
      timeout: 30_000,
      timeoutMsg: 'tier 2 never listed the article blocks',
    })

    const STRUCK = `${SCREEN} .ip-block-text .ip-cleanup-struck`
    expect(await struckCount(STRUCK)).toBe(0)
    await addRule()
    await browser.waitUntil(async () => (await struckCount(STRUCK)) === 3, {
      timeout: 30_000,
      timeoutMsg: 'the rule did not strike three pieces in the blocks (0 means the Rust mapping is missing)',
    })

    expect(await struckCount(`${SCREEN} .ip-cleanup-text`)).toBe(0)
    expect(await occurrences(TAIL_TOKEN)).toBe(1)
    expect(await occurrences(HEAD_TOKEN)).toBe(1)

    const lastPiece = (await $$(STRUCK))[2]
    const owner = await browser.execute((el) => el.closest('.ip-block').id, lastPiece)
    const ownerIndex = Number(owner.replace('ip-block-', ''))
    expect(ownerIndex).toBeGreaterThan(30)

    expect(await scroller()).toBe(true)
    await lastPiece.scrollIntoView({ block: 'center' })
    await browser.pause(400)

    const pieces = await $$(STRUCK)
    const firstPiece = pieces[0]
    await browser.action('pointer').move({ x: 5, y: 5 }).perform()
    await browser.pause(200)
    for (const piece of pieces) expect((await hintShown(piece)).shown).toBe(false)

    expect(await hoverRuleReveals()).toBe(true)

    await realClick(await $(`${SCREEN} #ip-block-${ownerIndex} .ip-block-text`))
    await browser.action('pointer').move({ x: 5, y: 5 }).perform()
    await browser.pause(200)
    expect((await hintShown(lastPiece)).shown).toBe(true)
    expect((await hintShown(firstPiece)).shown).toBe(false)

    await browser.keys('k')
    await browser.pause(300)
    const focusedNow = await browser.execute(() => document.querySelector('.ip-screen .ip-block-focused')?.id ?? '')
    expect(focusedNow).toBe(`ip-block-${ownerIndex - 1}`)
    expect((await hintShown(lastPiece)).shown).toBe(false)

    await setScrollTop(2000)
    const middleBlock = await browser.execute(() => {
      const mid = window.innerHeight / 2
      const hit = [...document.querySelectorAll('.ip-screen .ip-block')].find((el) => {
        const r = el.getBoundingClientRect()
        return r.top <= mid && r.bottom >= mid
      })
      return hit?.id ?? ''
    })
    expect(middleBlock).not.toBe('')
    await realClick(await $(`${SCREEN} #${middleBlock} .ip-block-text`))
    await browser.pause(800)
    const before = await scrollTop()
    expect(before).toBeGreaterThan(0)
    await toggleRule()
    await browser.waitUntil(async () => (await struckCount(STRUCK)) === 0, {
      timeout: 30_000,
      timeoutMsg: 'disabling the rule left struck pieces',
    })
    expect(await scrollTop()).toBe(before)
    expect(await $$(`${SCREEN} .ip-block`)).not.toHaveLength(0)
    await toggleRule()
    await browser.waitUntil(async () => (await struckCount(STRUCK)) === 3, {
      timeout: 30_000,
      timeoutMsg: 'enabling the rule did not bring the pieces back',
    })
    expect(await scrollTop()).toBe(before)

    await realClick(await $(`${SCREEN} footer.ip-footer .ip-act:not(.ip-act-primary)`))
    await $(SCREEN).waitForExist({ reverse: true, timeout: 10_000 })
    await realClick(await screen.$('.us-actions .us-btn:not(.us-btn-primary)'))
    await screen.waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('.txt path: the Chapter text sits in the text column with the struck piece focusable, the config column holds no text, and the text appears once', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'e2e-624-'))
    const txt = join(dir, 'truyen.txt')
    const line = 'The text of this file is long enough to run past the four thousand and ninety six byte preview window. '
    const head = `The opening paragraph has ${PATTERN} and ${HEAD_TOKEN} only once.\n`
    const tail = `The closing paragraph past the window has ${PATTERN} and ${TAIL_TOKEN}.\n`
    writeFileSync(txt, `${head}${Array.from({ length: 80 }, () => line).join('\n')}\n${tail}`, 'utf8')

    if (!(await $('.li-panel .import-form').isDisplayed())) await realClick(await $('[data-library-import-open]'))
    const form = await $('.li-panel .import-form')
    await form.waitForDisplayed({ timeout: 10_000 })
    const inputs = await form.$$('.field input[type="text"]')
    await inputs[0].setValue(TXT_WORK)
    await inputs[inputs.length - 2].setValue(txt)
    await realClick(await (await form.$('input[type="radio"][value="en"]')).$('..'))
    const buttons = await form.$$('[data-import-preview-open]')
    await realClick(buttons[1])
    await $(SCREEN).waitForDisplayed({ timeout: 60_000 })

    const piece = await $(`${SCREEN} .ip-main .ip-cleanup-text .ip-cleanup-struck`)
    await piece.waitForDisplayed({ timeout: 30_000, timeoutMsg: 'the .txt text column shows no struck piece' })
    expect(await piece.getAttribute('tabindex')).toBe('0')
    expect(await struckCount(`${SCREEN} aside.ip-config .ip-cleanup-text`)).toBe(0)
    expect(await struckCount(`${SCREEN} .ip-cleanup-window-note`)).toBe(1)
    expect(await occurrences(HEAD_TOKEN)).toBe(1)
    expect(await occurrences(TAIL_TOKEN)).toBe(0)

    await browser.action('pointer').move({ x: 5, y: 5 }).perform()
    await browser.pause(200)
    expect((await hintShown(piece)).shown).toBe(false)
    expect(await hoverRuleReveals()).toBe(true)

    await browser.action('pointer').move({ x: 5, y: 5 }).perform()
    await browser.keys('Tab')
    await browser.execute((el) => el.focus(), piece)
    await browser.pause(200)
    expect(await browser.execute((el) => el.matches(':focus-visible'), piece)).toBe(true)
    expect((await hintShown(piece)).shown).toBe(true)

    await realClick(await $(`${SCREEN} footer.ip-footer .ip-act:not(.ip-act-primary)`))
    await $(SCREEN).waitForExist({ reverse: true, timeout: 10_000 })
  })
})
