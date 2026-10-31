// Real-mouse bench: cliclick posts trusted input into the app window while the wdio session only
// reads state back. Serves one long article from 127.0.0.1 through the real URL import path.
import { execFileSync } from 'node:child_process'
import { writeFileSync } from 'node:fs'
import { createServer } from 'node:http'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { realClick } from '../../../e2e/support/pointer.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const OUT = process.env.BENCH_OUT ?? 'ket-qua.json'
const results = {}

const appPid = () => execFileSync('pgrep', ['-n', 'auratranslate'], { encoding: 'utf8' }).trim()
const frontPid = () =>
  execFileSync('osascript', ['-e', 'tell application "System Events" to get unix id of first process whose frontmost is true'], { encoding: 'utf8' }).trim()

// cliclick posts to whatever is in front, so it must never run while another app holds the screen.
function ensureFront() {
  const pid = appPid()
  for (let attempt = 0; attempt < 5; attempt += 1) {
    if (frontPid() === pid) return
    execFileSync('osascript', ['-e', `tell application "System Events" to set frontmost of (first process whose unix id is ${pid}) to true`])
    execFileSync('sleep', ['0.4'])
  }
  throw new Error('bench app is not frontmost: refusing to send real input')
}

const cli = (...args) => {
  ensureFront()
  return execFileSync('cliclick', ['-e', '40', ...args], { encoding: 'utf8' })
}
const pause = (ms) => browser.pause(ms)

let offset = null
let calibrated = null

async function installProbe() {
  await browser.execute(() => {
    if (window.__bench !== undefined) return
    window.__bench = { down: null, log: [] }
    window.addEventListener('mousedown', (e) => {
      window.__bench.down = [e.clientX, e.clientY]
    }, true)
    const name = (t) => `${t.tagName}.${String(t.className).split(' ')[0]}:${(t.textContent ?? '').trim().slice(0, 14)}`
    for (const type of ['mousedown', 'mouseup', 'click', 'focusin', 'focusout']) {
      window.addEventListener(type, (e) => {
        window.__bench.log.push(`${type} ${name(e.target)} @${String(e.clientX)},${String(e.clientY)} btn=${String(e.button)}`)
      }, true)
    }
  })
}

async function calibrate() {
  const win = await browser.execute(() => ({ x: window.screenX, y: window.screenY, oh: window.outerHeight, ih: window.innerHeight }))
  const guess = { x: win.x + 300, y: win.y + (win.oh - win.ih) + 200 }
  // A freshly started app swallows single clicks until it is activated; a double click passes.
  execFileSync('osascript', ['-e', 'tell application "Finder" to activate'])
  execFileSync('sleep', ['0.4'])
  cli(`dc:${guess.x},${guess.y}`)
  await pause(500)
  const pt = await browser.execute(() => window.__bench.down)
  results.calibration = { win, guess, pt }
  if (pt === null) throw new Error(`no press reached the page at ${JSON.stringify(guess)}`)
  offset = { x: guess.x - pt[0], y: guess.y - pt[1] }
  calibrated = guess
}

function cmdTab() {
  execFileSync('osascript', ['-e', 'tell application "System Events" to key code 48 using command down'])
  execFileSync('sleep', ['0.6'])
}

async function waitForSingleClicks() {
  cmdTab()
  cmdTab()
  const started = Date.now()
  while (Date.now() - started < 30_000) {
    await browser.execute(() => { window.__bench.down = null })
    cli(`c:${calibrated.x},${calibrated.y}`)
    await pause(500)
    if ((await browser.execute(() => window.__bench.down)) !== null) {
      results.activation_ms = Date.now() - started
      return
    }
  }
  throw new Error('single clicks are still swallowed after 30 s')
}

const screen = (x, y) => `${String(Math.round(x + offset.x))},${String(Math.round(y + offset.y))}`

// Move first and wait: without this hover beat a press can be lost (14-5 bench). cliclick `c:`
// releases at once, which loses presses a person's hand would not (see ket-qua.md), so hold.
async function press(x, y, holdMs = 90) {
  cli(`m:${screen(x, y)}`)
  await pause(200)
  if (holdMs === 0) cli(`c:${screen(x, y)}`)
  else cli(`dd:${screen(x, y)}`, `w:${String(holdMs)}`, `du:${screen(x, y)}`)
  await pause(350)
}

async function state() {
  return browser.execute(() => {
    const blocks = [...document.querySelectorAll('.ip-block')]
    const scroller = (() => {
      let el = document.querySelector('.ip-blocks')
      while (el !== null) {
        const o = getComputedStyle(el).overflowY
        if ((o === 'auto' || o === 'scroll') && el.scrollHeight > el.clientHeight) return el
        el = el.parentElement
      }
      return document.scrollingElement
    })()
    return {
      aimed: blocks.findIndex((b) => b.classList.contains('ip-block-focused')),
      dropped: blocks.map((b, i) => (b.classList.contains('ip-block-dropped') ? i : -1)).filter((i) => i >= 0),
      active: `${document.activeElement?.tagName}.${document.activeElement?.className ?? ''}`,
      scrollTop: scroller.scrollTop,
      scroller: `${scroller.tagName}.${scroller.className}`,
      selection: String(window.getSelection()),
      clusters: document.querySelectorAll('.ip-block-actions').length,
      labels: [...document.querySelectorAll('.ip-block-actions button')].map((b) => b.textContent.trim()),
      note: document.querySelector('.ip-chapter-cursor-note')?.textContent?.trim() ?? null,
      marker: document.querySelector('.ip-block-range-marker') !== null,
    }
  })
}

const blockCount = () => browser.execute(() => document.querySelectorAll('.ip-block').length)

async function scrollToBlock(index) {
  await browser.execute((i) => {
    document.getElementById(`ip-block-${String(i)}`).scrollIntoView({ block: 'center' })
  }, index)
  await pause(200)
}

async function blockTextPoint(index) {
  return browser.execute((i) => {
    const r = document.querySelector(`#ip-block-${String(i)} .ip-block-text`).getBoundingClientRect()
    return { x: r.x + 60, y: r.y + 10, top: document.getElementById(`ip-block-${String(i)}`).getBoundingClientRect().top }
  }, index)
}

async function buttonPoint(label) {
  return browser.execute((text) => {
    const b = [...document.querySelectorAll('.ip-block-actions button, .ip-chapter-cursor-row button')].find((el) => el.textContent.trim() === text)
    const r = b.getBoundingClientRect()
    return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
  }, label)
}

async function pressButton(label) {
  const p = await buttonPoint(label)
  for (let attempt = 1; attempt <= 4; attempt += 1) {
    await browser.execute(() => { window.__bench.log.length = 0 })
    await press(p.x, p.y)
    const log = await browser.execute(() => window.__bench.log.slice())
    ;(results.button_events ??= []).push({ label, attempt, log })
    if (log.some((line) => line.startsWith('click'))) return attempt
    cli(`m:${screen(p.x - 90, p.y + 90)}`)
    await pause(300)
  }
  throw new Error(`${label}: no click after four presses`)
}

// A press lost after a typed key must not derail the run: re-press after a real mouse move.
async function aimBlock(index) {
  const p = await blockTextPoint(index)
  for (let attempt = 1; attempt <= 4; attempt += 1) {
    await press(p.x, p.y)
    if ((await state()).aimed === index) {
      ;(results.aim_attempts ??= []).push({ index, attempt })
      return p
    }
    cli(`m:${screen(p.x + 90, p.y + 60)}`)
    await pause(300)
  }
  throw new Error(`block ${String(index)}: not aimed after four presses`)
}

async function pointOfSortToggle() {
  return browser.execute(() => {
    const el = document.querySelector('.ip-chapters-sort-toggle')
    el.scrollIntoView({ block: 'center' })
    const r = el.getBoundingClientRect()
    return { x: r.x + 20, y: r.y + r.height / 2 }
  })
}

async function clickLanded(p, holdMs) {
  await browser.execute(() => { window.__bench.log.length = 0 })
  await press(p.x, p.y, holdMs)
  await pause(200)
  return browser.execute(() => window.__bench.log.some((line) => line.startsWith('click')))
}

// Press right after a typed key, alternating the hold inside one run so order cannot explain a
// difference. The control's focus is handed back to the list, or the next J would not move.
async function lostAfterKey(rounds) {
  const presses = []
  for (let i = 0; i < rounds; i += 1) {
    const holdMs = i % 2 === 0 ? 90 : 0
    const target = 2 + (i % 8)
    await scrollToBlock(target)
    await aimBlock(target)
    await typeKey('j')
    await scrollToBlock(target + 1)
    const p = await buttonPoint('Đầu vùng giữ')
    const visible = await browser.execute((y) => y > 50 && y < window.innerHeight - 40, p.y)
    await browser.execute(() => { window.__bench.log.length = 0 })
    await press(p.x, p.y, holdMs)
    await pause(200)
    const cluster = await browser.execute(() => window.__bench.log.map((line) => line.split(' ')[0]))
    const control = await pointOfSortToggle()
    await pause(150)
    await browser.execute(() => { window.__bench.log.length = 0 })
    await press(control.x, control.y, holdMs)
    await pause(200)
    const controlEvents = await browser.execute(() => window.__bench.log.map((line) => line.split(' ')[0]))
    await browser.execute(() => document.querySelector('.ip-blocks').focus({ preventScroll: true }))
    presses.push({ holdMs, y: Math.round(p.y), visible, cluster, control: controlEvents })
  }
  return presses
}

async function typeKey(key) {
  cli(`t:${key}`)
  await pause(300)
}

const article = () => {
  const paragraphs = Array.from({ length: 30 }, (_, i) =>
    `<p>Doan ${String(i + 1)} cua bai viet co du chu de Readability chon lam noi dung chinh, tiep tuc mot cau dai de khoi nay cao hon mot dong tren man hinh xem truoc, khong phai quang cao hay menu dieu huong.</p>`)
  return `<html><head><title>Bai dai</title></head><body><article><h1>Bai dai</h1>${paragraphs.join('\n')}</article></body></html>`
}

let server
let port

describe('14-6 bench', () => {
  before(async () => {
    server = createServer((req, res) => {
      res.setHeader('Content-Type', 'text/html; charset=utf-8')
      res.end(article())
    })
    await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve))
    port = server.address().port
  })

  afterEach(() => {
    writeFileSync(join(HERE, OUT), `${JSON.stringify(results, null, 2)}\n`)
  })

  after(async () => {
    writeFileSync(join(HERE, OUT), `${JSON.stringify(results, null, 2)}\n`)
    await new Promise((resolve) => server.close(resolve))
  })

  it('aims, edits, selects and keeps keys alive under a real mouse', async () => {
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), { timeout: 30_000, interval: 250 })
    await $('[data-library-import-open]').waitForDisplayed({ timeout: 30_000 })
    await installProbe()
    await calibrate()
    await waitForSingleClicks()

    await realClick(await $('[data-library-import-open]'))
    await $('[data-library-url-screen-open]').waitForDisplayed({ timeout: 10_000 })
    await realClick(await $('[data-library-url-screen-open]'))
    const form = await $('[data-library-url-import-screen]')
    await form.waitForDisplayed({ timeout: 10_000 })
    await (await form.$('textarea')).setValue(`http://127.0.0.1:${String(port)}/a\nhttp://127.0.0.1:${String(port)}/b`)
    await (await form.$('input[type="text"]')).setValue(`bench-146-${String(Date.now() % 100000)}`)
    await realClick(await form.$('[data-import-preview-open]'))
    await $('.ip-screen').waitForDisplayed({ timeout: 60_000 })
    await browser.waitUntil(async () => (await blockCount()) >= 20, { timeout: 30_000, timeoutMsg: 'tier 2 stayed short' })
    results.blocks = await blockCount()

    // Same block as the opening aim, with focus still on the panel.
    await scrollToBlock(0)
    let p = await blockTextPoint(0)
    const beforeSame = await state()
    await press(p.x, p.y)
    const afterSame = await state()
    await typeKey('j')
    results.same_block = { before: beforeSame, after: afterSame, afterJ: await state() }

    // A block far down the list: the list must not jump and the row must not move.
    await scrollToBlock(14)
    const beforeFar = await state()
    p = await aimBlock(14)
    const afterFar = await state()
    const movedBy = (await blockTextPoint(14)).top - p.top
    await typeKey('j')
    results.far_block = { before: beforeFar, after: afterFar, movedBy, afterJ: await state() }

    // Drop through the cluster, then keep walking by key.
    await pressButton('Loại khối này')
    await pause(500)
    const dropped = await state()
    await typeKey('j')
    results.drop = { dropped, afterJ: await state() }

    // Range: mark here, aim two blocks up (reversed), confirm; the End button disappears.
    await pressButton('Đầu vùng giữ')
    const marked = await state()
    await scrollToBlock(marked.aimed - 2)
    await aimBlock(marked.aimed - 2)
    const aimedEnd = await state()
    await pressButton('Cuối vùng giữ')
    await pause(700)
    const confirmed = await state()
    await typeKey('j')
    results.range = { marked, aimedEnd, confirmed, afterJ: await state() }

    // Dragging over text aims the block and keeps the selection.
    await scrollToBlock(5)
    p = await blockTextPoint(5)
    cli(`m:${screen(p.x, p.y)}`)
    await pause(200)
    cli(`dd:${screen(p.x, p.y)}`, `dm:${screen(p.x + 80, p.y)}`, `du:${screen(p.x + 160, p.y)}`)
    await pause(400)
    results.select = await state()

    // Chapter buttons: focus must not land on a button that then goes disabled.
    await browser.execute(() => document.querySelector('.ip-chapter-cursor-row').scrollIntoView({ block: 'center' }))
    await pause(200)
    const beforeNext = await state()
    await pressButton('›')
    await pause(900)
    const chapter2 = await state()
    await typeKey('j')
    const afterK = await state()
    await pressButton('‹')
    await pause(900)
    results.chapter = { beforeNext, chapter2, afterK, back: await state() }

    results.layout = await browser.execute(() => {
      const head = document.querySelector('.ip-block-focused .ip-block-head')?.getBoundingClientRect()
      const actions = document.querySelector('.ip-block-actions')?.getBoundingClientRect()
      return { innerWidth: window.innerWidth, head: head?.toJSON(), actions: actions?.toJSON() }
    })
    await browser.saveScreenshot(join(HERE, 'man-hinh.png'))
  })

  it('counts presses lost right after a key, hold alternating 90 and 0 ms', async () => {
    results.lost_after_key = await lostAfterKey(12)
  })
})
