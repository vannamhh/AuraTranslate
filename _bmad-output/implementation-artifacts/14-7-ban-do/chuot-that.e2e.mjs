// Real-mouse bench: cliclick posts trusted input into the app window while the wdio session only
// reads state back. Imports a one-row bilingual CSV with a long Vietnamese target line.
import { execFileSync } from 'node:child_process'
import { writeFileSync } from 'node:fs'
import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
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

// Move first and wait, then hold the button: cliclick `c:` releases at once and loses the press.
async function press(x, y, holdMs = 90) {
  cli(`m:${screen(x, y)}`)
  await pause(200)
  if (holdMs === 0) cli(`c:${screen(x, y)}`)
  else cli(`dd:${screen(x, y)}`, `w:${String(holdMs)}`, `du:${screen(x, y)}`)
  await pause(350)
}

const SCREEN = '.bip-screen'

async function state() {
  return browser.execute(() => {
    const targets = [...document.querySelectorAll('.bip-cut-target')]
    return {
      heading: document.querySelector('.bip-mismatch-heading')?.textContent?.trim() ?? null,
      glyphs: targets.map((t) => t.textContent),
      aimed: targets.findIndex((t) => t.classList.contains('bip-cut-aimed')),
      endAimed: document.querySelectorAll('.bip-cut-end.bip-cut-aimed').length,
      active: `${document.activeElement?.tagName}.${document.activeElement?.className ?? ''}`,
      selection: String(window.getSelection()),
      pieces: [...document.querySelectorAll('.bip-mismatch-column')[1].querySelectorAll('li')].map((li) => li.textContent.trim()),
      pairButtons: [...document.querySelectorAll('.bip-mismatch-heading-row button')].map((b) => b.textContent.trim()),
    }
  })
}

async function targetPoint(index) {
  return browser.execute((i) => {
    const el = document.querySelectorAll('.bip-cut-target')[i]
    el.scrollIntoView({ block: 'center' })
    const r = el.getBoundingClientRect()
    return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
  }, index)
}

async function pressTarget(index) {
  const p = await targetPoint(index)
  for (let attempt = 1; attempt <= 4; attempt += 1) {
    await browser.execute(() => { window.__bench.log.length = 0 })
    await press(p.x, p.y)
    const log = await browser.execute(() => window.__bench.log.slice())
    ;(results.target_events ??= []).push({ index, attempt, log })
    if (log.some((line) => line.startsWith('click'))) return attempt
    cli(`m:${screen(p.x - 90, p.y + 90)}`)
    await pause(300)
  }
  throw new Error(`target ${String(index)}: no click after four presses`)
}

async function pressPairButton(label) {
  const p = await browser.execute((text) => {
    const b = [...document.querySelectorAll('.bip-mismatch-heading-row button')].find((el) => el.textContent.trim() === text)
    const r = b.getBoundingClientRect()
    return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
  }, label)
  for (let attempt = 1; attempt <= 4; attempt += 1) {
    await browser.execute(() => { window.__bench.log.length = 0 })
    await press(p.x, p.y)
    const log = await browser.execute(() => window.__bench.log.slice())
    if (log.some((line) => line.startsWith('click'))) return attempt
    cli(`m:${screen(p.x - 90, p.y + 90)}`)
    await pause(300)
  }
  throw new Error(`${label}: no click after four presses`)
}

async function arrow(code) {
  execFileSync('osascript', ['-e', `tell application "System Events" to key code ${String(code)}`])
  await pause(300)
}

async function geometry() {
  return browser.execute(() => {
    const targets = [...document.querySelectorAll('.bip-cut-target')]
    const centres = targets.map((t) => {
      const r = t.getBoundingClientRect()
      return { x: r.x + r.width / 2, y: r.y + r.height / 2, w: r.width, h: r.height }
    })
    const gaps = []
    for (let i = 1; i < centres.length; i += 1) {
      if (Math.abs(centres[i].y - centres[i - 1].y) < 2) gaps.push(Math.round((centres[i].x - centres[i - 1].x) * 10) / 10)
    }
    const line = document.querySelector('.bip-caret-line').getBoundingClientRect()
    return { count: targets.length, width: centres[0]?.w, height: centres[0]?.h, minCentreGap: Math.min(...gaps), lineHeight: line.height, lineWidth: line.width, rows: new Set(centres.map((c) => Math.round(c.y))).size }
  })
}

const LONG_TARGET =
  'Ong lao ngoi duoi goc cay da nhin ra con song dang chay xiet nghi ve nhung nam thang da qua ve nhung nguoi da di xa va nhung loi hua chua kip thuc hien roi nhe nhang noi voi dua tre ben canh rang troi sap mua'
const SOURCE = '老人坐在树下。看着河水。想起往事。'

describe('14-7 bench', () => {
  afterEach(() => {
    writeFileSync(join(HERE, OUT), `${JSON.stringify(results, null, 2)}\n`)
  })

  it('cuts, uncuts, aims and keeps arrow keys alive under a real mouse', async () => {
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), { timeout: 30_000, interval: 250 })
    await $('[data-library-import-open]').waitForDisplayed({ timeout: 30_000 })
    await installProbe()
    await calibrate()
    await waitForSingleClicks()

    const dir = mkdtempSync(join(tmpdir(), 'bench-147-'))
    const csv = join(dir, 'lech.csv')
    writeFileSync(csv, `${SOURCE},${LONG_TARGET}\n`, 'utf8')
    await realClick(await $('[data-library-import-open]'))
    const form = await $('.li-panel .import-form')
    await form.waitForDisplayed({ timeout: 10_000 })
    await (await form.$('.field input[type="text"]')).setValue(`bench-147-${String(Date.now() % 100000)}`)
    await realClick(await (await form.$('[data-import-source-lang] input[type="radio"][value="zh"]')).$('..'))
    const inputs = await form.$$('.field input[type="text"]')
    await inputs[inputs.length - 1].setValue(csv)
    await realClick(await form.$('[data-bilingual-import-preview-open]'))
    await $(SCREEN).waitForDisplayed({ timeout: 30_000 })
    await $('.bip-cut-target').waitForDisplayed({ timeout: 15_000 })

    results.geometry = await geometry()
    results.start = await state()

    // Cut at the third and sixth mark: three pieces pair with three source sentences.
    const first = await pressTarget(2)
    const afterFirst = await state()
    const geometryAfterCut = await geometry()
    const second = await pressTarget(5)
    const afterSecond = await state()
    results.cut = { first, afterFirst, geometryAfterCut, second, afterSecond }

    // Click the same mark again: the row goes back to mismatched with the caret kept.
    await pressTarget(5)
    const afterUncut = await state()
    await arrow(123)
    const afterLeft = await state()
    await arrow(124)
    const afterRight = await state()
    results.uncut = { afterUncut, afterLeft, afterRight }

    // No retry here: each press is counted once. Targets alternate so every press moves the caret
    // and re-renders on mousedown; the heading is a control press with no handler on it.
    const headingPoint = () =>
      browser.execute(() => {
        const r = document.querySelector('.bip-mismatch-heading').getBoundingClientRect()
        return { x: r.x + 40, y: r.y + r.height / 2 }
      })
    const counted = async (kind, point) => {
      await browser.execute(() => { window.__bench.log.length = 0 })
      await press(point.x, point.y)
      const log = await browser.execute(() => window.__bench.log.slice())
      const has = (type) => log.some((line) => line.startsWith(`${type} `))
      ;(results.loss ??= []).push({ kind, down: has('mousedown'), up: has('mouseup'), click: has('click') })
    }
    for (let round = 0; round < 12; round += 1) {
      await counted('target', await targetPoint(round % 2 === 0 ? 9 : 14))
      await counted('heading', await headingPoint())
    }

    await browser.saveScreenshot(join(HERE, 'man-hinh.png'))
  })
})
