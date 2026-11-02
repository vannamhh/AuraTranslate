/**
 * Import preview frame in a real WKWebView: the Nhập button stays inside the viewport at the
 * top and at the bottom of a long text column, the config column does not move while the text
 * scrolls, a dropped-block band opens, and the split-pattern box follows the `already_chaptered`
 * flag (absent on the URL path, present on a .txt).
 *
 * A local `node:http` server serves one page with 210 navigation list items (all dropped by
 * the extractor, one run) around a three-paragraph article.
 */
import { mkdtempSync, writeFileSync } from 'node:fs'
import { createServer } from 'node:http'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { realClick } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const URL_WORK = `e2e-623-url-${RUN_TAG}`
const TXT_WORK = `e2e-623-txt-${RUN_TAG}`
const SCREEN = '.ip-screen'
const FOOTER_BUTTON = `${SCREEN} footer.ip-footer .ip-act-primary`

function page() {
  const nav = Array.from({ length: 210 }, (_, i) => `<li>Muc dieu huong so ${i + 1}</li>`).join('\n')
  const body = [1, 2, 3]
    .map(
      (n) =>
        `<p>Doan ${n} cua bai viet co du chu de duoc Readability chon lam noi dung chinh cua trang, nhieu chu hon de vuot nguong do dai toi thieu va giu lai.</p>`,
    )
    .join('\n')
  return `<html><head><title>trang dai</title></head><body><nav><ul>${nav}</ul></nav><article><h1>trang dai</h1>${body}</article></body></html>`
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

async function footerButtonInView() {
  return browser.execute((sel) => {
    const rect = document.querySelector(sel).getBoundingClientRect()
    return rect.top >= 0 && rect.bottom <= window.innerHeight + 1 && rect.height > 0
  }, FOOTER_BUTTON)
}

async function configTop() {
  return browser.execute(() => Math.round(document.querySelector('.ip-screen aside.ip-config').getBoundingClientRect().top))
}

async function scrollTextColumn(fraction) {
  const result = await browser.execute((fraction) => {
    let node = document.querySelector('.ip-screen .ip-main')
    while (node !== null) {
      const overflow = getComputedStyle(node).overflowY
      if ((overflow === 'auto' || overflow === 'scroll') && node.scrollHeight > node.clientHeight) break
      node = node.parentElement
    }
    if (node === null) return { scrolled: false, top: 0, max: 0 }
    node.scrollTop = Math.round((node.scrollHeight - node.clientHeight) * fraction)
    return { scrolled: true, top: node.scrollTop, max: node.scrollHeight - node.clientHeight }
  }, fraction)
  await browser.pause(600)
  return result
}

describe('Story 6-23: import preview frame', () => {
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

  it('URL path: one band of 200+ dropped blocks, footer button in view at top and bottom, config column steady, no pattern box', async () => {
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
    await $(`${SCREEN} .ip-band`).waitForDisplayed({ timeout: 30_000 })

    const bandCount = await browser.execute(() => {
      const title = document.querySelector('.ip-screen .ip-band-title')?.textContent ?? ''
      return Number((title.match(/\d+/) ?? ['0'])[0])
    })
    expect(bandCount).toBeGreaterThanOrEqual(200)
    expect(await $$(`${SCREEN} .ip-band`)).toHaveLength(1)

    expect(await $(`${SCREEN} .ip-chapters-pattern-input`).isExisting()).toBe(false)

    expect(await footerButtonInView()).toBe(true)
    const blocksClosed = (await $$(`${SCREEN} .ip-block`)).length
    await realClick(await $(`${SCREEN} .ip-band`))
    await browser.waitUntil(async () => (await $$(`${SCREEN} .ip-block`)).length >= blocksClosed + 200, {
      timeout: 15_000,
      timeoutMsg: 'opening the band did not list its dropped blocks',
    })
    expect(await $(`${SCREEN} .ip-band.ip-band-open`).isExisting()).toBe(true)

    const down = await scrollTextColumn(1)
    expect(down.scrolled).toBe(true)
    expect(down.top).toBeGreaterThan(0)
    expect(await footerButtonInView()).toBe(true)
    const pinnedAtEnd = await configTop()
    expect(pinnedAtEnd).toBeGreaterThanOrEqual(0)
    expect(pinnedAtEnd).toBeLessThan(200)
    const configBottomAtEnd = await browser.execute(() =>
      Math.round(document.querySelector('.ip-screen aside.ip-config').getBoundingClientRect().bottom),
    )
    const footerTopAtEnd = await browser.execute(() =>
      Math.round(document.querySelector('.ip-screen footer.ip-footer').getBoundingClientRect().top),
    )
    expect(configBottomAtEnd).toBeLessThanOrEqual(footerTopAtEnd)

    await scrollTextColumn(0.3)
    expect(await configTop()).toBe(pinnedAtEnd)
    await scrollTextColumn(0.6)
    expect(await footerButtonInView()).toBe(true)
    expect(await configTop()).toBe(pinnedAtEnd)

    await scrollTextColumn(0)
    expect(await footerButtonInView()).toBe(true)
    expect(await configTop()).toBeGreaterThanOrEqual(pinnedAtEnd)

    await realClick(await $(`${SCREEN} .ip-band`))
    await browser.waitUntil(async () => (await $$(`${SCREEN} .ip-block`)).length === blocksClosed, { timeout: 15_000 })

    await realClick(await $(`${SCREEN} footer.ip-footer .ip-act:not(.ip-act-primary)`))
    await $(SCREEN).waitForExist({ reverse: true, timeout: 10_000 })
    await realClick(await screen.$('.us-actions .us-btn:not(.us-btn-primary)'))
    await screen.waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('.txt path: pattern box present, typing a pattern changes the Chapter count, strip with the shortcut chip appears from 2 Chapters', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'e2e-623-'))
    const txt = join(dir, 'truyen.txt')
    const chapter = (n) => `Chuong ${n}\n${'Dong van ban cua chuong nay du dai de duoc tach rieng. '.repeat(6)}\n`
    writeFileSync(txt, `${chapter(1)}\n${chapter(2)}\n${chapter(3)}`, 'utf8')

    if (!(await $('.li-panel .import-form').isDisplayed())) await realClick(await $('[data-library-import-open]'))
    const form = await $('.li-panel .import-form')
    await form.waitForDisplayed({ timeout: 10_000 })
    const inputs = await form.$$('.field input[type="text"]')
    await inputs[0].setValue(TXT_WORK)
    await inputs[inputs.length - 2].setValue(txt)
    const buttons = await form.$$('[data-import-preview-open]')
    await realClick(buttons[1])
    await $(SCREEN).waitForDisplayed({ timeout: 60_000 })

    const patternInput = await $(`${SCREEN} .ip-chapters-pattern-input`)
    await patternInput.waitForDisplayed({ timeout: 15_000 })
    expect(await footerButtonInView()).toBe(true)

    await patternInput.setValue('Chuong')
    await browser.keys('Tab')
    await browser.waitUntil(async () => (await $(FOOTER_BUTTON).getText()).includes('Nhập 3 Chương'), {
      timeout: 30_000,
      timeoutMsg: 'footer button never read "Nhập 3 Chương" after typing the pattern',
    })
    await $(`${SCREEN} .ip-main .ip-chapter-strip`).waitForDisplayed({ timeout: 10_000 })
    expect((await $(`${SCREEN} .ip-chapter-filter-key`).getText()).trim()).toBe('⌥W')

    await realClick(await $(`${SCREEN} footer.ip-footer .ip-act:not(.ip-act-primary)`))
    await $(SCREEN).waitForExist({ reverse: true, timeout: 10_000 })
  })
})
