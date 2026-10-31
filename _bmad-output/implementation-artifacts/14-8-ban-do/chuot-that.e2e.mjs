// Real-mouse bench: cliclick posts trusted input into the app window while the wdio session only
// reads state back. Flips each panel switch in `Công cụ ▾` and reads the grid scroll position back.
import { execFileSync } from 'node:child_process'
import { writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { waitForGridRows } from '../../../e2e/support/gridWait.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const OUT = join(HERE, 'ket-qua.json')
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
  // Repeating a press on one point without moving the pointer loses the press (14-4 ket-qua.md).
  cli(`m:${screen(x, y + 60)}`)
  await pause(120)
  cli(`m:${screen(x, y)}`)
  await pause(200)
  if (holdMs === 0) cli(`c:${screen(x, y)}`)
  else cli(`dd:${screen(x, y)}`, `w:${String(holdMs)}`, `du:${screen(x, y)}`)
  await pause(350)
}

const SWITCHES = ['layout.toggle_grid', 'layout.toggle_lookup', 'layout.toggle_ai_translation']
const TEXT = Array.from({ length: 120 }, (_, i) => `第${String(i + 1)}句。`).join('')

const centreOf = (selector) =>
  browser.execute((sel) => {
    const r = document.querySelector(sel).getBoundingClientRect()
    return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
  }, selector)

async function pressCounted(label, selector) {
  const p = await centreOf(selector)
  await browser.execute(() => { window.__bench.log.length = 0 })
  await press(p.x, p.y)
  const log = await browser.execute(() => window.__bench.log.slice())
  const has = (type) => log.some((line) => line.startsWith(`${type} `))
  ;(results.presses ??= []).push({ label, down: has('mousedown'), up: has('mouseup'), click: has('click') })
}

const dockState = () =>
  browser.execute((ids) => ({
    groups: document.querySelectorAll('.dv-groupview').length,
    checked: ids.map((id) => document.querySelector(`[data-tools-item="${id}"]`)?.getAttribute('aria-checked') ?? null),
    drawerOpener: document.querySelector('[data-lookup-drawer-open]') !== null,
  }), SWITCHES)

// `aria-checked` only exists while the menu is open, so the read goes through two more real presses.
async function readDock(label) {
  await pressCounted(`read: open for ${label}`, '.tools-trigger')
  const state = await dockState()
  await pressCounted(`read: close for ${label}`, '.tools-trigger')
  return state
}

const gridScroll = () =>
  browser.execute(() => {
    const el = document.querySelector('.grid-scroll')
    return el === null ? null : { top: el.scrollTop, height: el.scrollHeight }
  })

describe('14-8 bench', () => {
  afterEach(() => {
    writeFileSync(OUT, `${JSON.stringify(results, null, 2)}\n`)
  })

  it('flips each panel switch under a real mouse and keeps the grid scroll position', async () => {
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), { timeout: 30_000, interval: 250 })
    await openWorkspaceWithWork('bench-148', TEXT)
    await waitForGridRows(120, { what: 'bench rows' })
    // Physical pixels at dpr 2: stays on the 1536×960 display, where `ensureFullLayoutTier` (5000 px) would push `Công cụ ▾` off screen.
    await browser.setWindowSize(2600, 1800)
    await pause(500)
    await installProbe()
    await calibrate()
    await waitForSingleClicks()

    const scrollGrid = () =>
      browser.execute(() => {
        const el = document.querySelector('.grid-scroll')
        if (el !== null) el.scrollTop = 600
      })
    results.start = await readDock('start')

    for (const id of SWITCHES) {
      await scrollGrid()
      results[`${id}.scrollBefore`] = await gridScroll()
      await pressCounted(`open menu for ${id}`, '.tools-trigger')
      await pressCounted(`${id} off`, `[data-tools-item="${id}"]`)
      await pause(400)
      results[`${id}.off`] = await readDock(`${id} off`)
      await pressCounted(`open menu for ${id}`, '.tools-trigger')
      await pressCounted(`${id} on`, `[data-tools-item="${id}"]`)
      await pause(600)
      results[`${id}.on`] = await readDock(`${id} on`)
      results[`${id}.scrollAfter`] = await gridScroll()
    }

    await browser.saveScreenshot(join(HERE, 'man-hinh.png'))
  })
})
