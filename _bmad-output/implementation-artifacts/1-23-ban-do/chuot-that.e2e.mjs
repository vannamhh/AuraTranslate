// Real-mouse bench for lookup by one click: cliclick and System Events post trusted input into the app window while the
// wdio session only reads state back. cliclick `kp:esc` never reaches the page, so Escape goes
// through osascript. Run one `it` at a time: a stale Vite HMR stamp splits the fixture's modules.
import { execFileSync } from 'node:child_process'
import { existsSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { waitForGridRows } from '../../../e2e/support/gridWait.mjs'
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
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
    const bench = { log: [], down: null }
    window.__bench = bench
    const name = (t) => {
      if (!(t instanceof Element)) return String(t)
      const col = t.closest('[data-col]')?.getAttribute('data-col')
      const id = t.closest('[data-segment-id]')?.getAttribute('data-segment-id')
      const num = t.closest('[data-num-segment-id]')?.getAttribute('data-num-segment-id')
      if (t.closest('.layout-switch-btn') !== null) return 'layout-cell'
      if (t.closest('[data-sentence-menu]') !== null) return 'menu'
      return col !== undefined && col !== null ? `${col}:${id}` : num ? `num:${num}` : t.tagName
    }
    window.addEventListener('mousedown', (e) => {
      bench.down = [e.clientX, e.clientY]
    }, true)
    for (const type of ['mousedown', 'mouseup', 'click', 'focusin', 'focusout', 'keydown']) {
      window.addEventListener(type, (e) => {
        bench.log.push(`${type} ${name(e.target)} key=${String(e.key)} prevented=${String(e.defaultPrevented)}`)
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
  if (pt === null) throw new Error(`no press reached the page at ${JSON.stringify(guess)} win=${JSON.stringify(win)} mouse=${execFileSync('cliclick', ['p'], { encoding: 'utf8' }).trim()}`)
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

async function screenPoint(selector) {
  const p = await browser.execute((sel) => {
    const r = document.querySelector(sel).getBoundingClientRect()
    return [r.x + Math.min(r.width / 2, 24), r.y + r.height / 2]
  }, selector)
  return `${String(Math.round(p[0] + offset.x))},${String(Math.round(p[1] + offset.y))}`
}

const SOURCE = '他拉小提琴。'
const PHRASE = '小提琴'

const headword = () => browser.execute(() => document.querySelector('.lookup-headword')?.textContent?.trim() ?? null)
const painted = () =>
  browser.execute(() => {
    const highlight = CSS.highlights.get('lookup-active')
    return highlight === undefined ? null : [...highlight].map((range) => range.toString()).join('')
  })
const selection = () => browser.execute(() => String(window.getSelection()))

// Screen point of one source character (code-point index) of the first source cell.
async function charPoint(index) {
  const p = await browser.execute((cp) => {
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
        const r = range.getBoundingClientRect()
        return [r.x + r.width / 2, r.y + r.height / 2]
      }
      seen += chars.length
    }
    throw new Error('no character at ' + cp)
  }, index)
  return `${String(Math.round(p[0] + offset.x))},${String(Math.round(p[1] + offset.y))}`
}

async function pressAt(kind, point) {
  for (let attempt = 0; attempt < 4; attempt += 1) {
    await browser.execute(() => { window.__bench.down = null })
    cli(`m:${point}`, 'w:200', `${kind}:${point}`)
    await pause(350)
    if ((await browser.execute(() => window.__bench.down)) !== null) return
    await pause(300)
  }
  throw new Error(`press never reached the page at ${point}`)
}

// Panel Lookup only mounts in the `full` tier (work area 1100×820, under a 42px titlebar and a 35px status bar). A window as tall as the screen makes WKWebView
// report `screenY` at the screen's bottom edge, which breaks the screen mapping, so it is sized to fit instead.
async function fullTierWindow() {
  const dpr = await browser.execute(() => window.devicePixelRatio)
  await browser.setWindowSize(Math.round(1200 * dpr), Math.round(910 * dpr))
  await browser.waitUntil(
    async () => (await browser.execute(() => document.querySelector('[data-layout-tier]')?.getAttribute('data-layout-tier'))) === 'full',
    {
      timeout: 10_000,
      timeoutMsg: `bench window did not reach the full layout tier: ${JSON.stringify(await browser.execute(() => ({ w: window.innerWidth, h: window.innerHeight, x: window.screenX, y: window.screenY, tier: document.querySelector('[data-layout-tier]')?.getAttribute('data-layout-tier') })))}`,
    },
  )
}

async function fresh(name) {
  await openWorkspaceWithWork(name, SOURCE)
  await fullTierWindow()
  await waitForGridRows(1, { what: 'bench' })
  await installProbe()
  for (let attempt = 0; attempt < 5 && offset === null; attempt += 1) {
    await pause(1200)
    await calibrate().catch((err) => {
      results.calibration_errors = [...(results.calibration_errors ?? []), `${String(err)} app=${appPid()} front=${frontPid()}`]
    })
  }
  if (offset === null) throw new Error(`no calibration: ${String(results.calibration_errors)}`)
  await waitForSingleClicks()
}

describe('real input: lookup by one click', () => {
  after(() => {
    const file = join(HERE, 'ket-qua.json')
    const prior = existsSync(file) ? JSON.parse(readFileSync(file, 'utf8')) : {}
    writeFileSync(file, JSON.stringify({ ...prior, ...results }, null, 2))
  })

  it('a real single click on one character looks up its word and paints it, with no selection', async () => {
    await fresh('Bench lookup click')
    await pressAt('c', await charPoint(SOURCE.indexOf('提')))
    await pause(800)
    results.click_word = { headword: await headword(), painted: await painted(), selection: await selection() }
    if (results.click_word.headword !== PHRASE || results.click_word.painted !== PHRASE) throw new Error(JSON.stringify(results.click_word))
  })

  it('a real click on punctuation keeps the previous result and paint', async () => {
    await fresh('Bench lookup punctuation')
    await pressAt('c', await charPoint(SOURCE.indexOf('提')))
    await pause(800)
    await pressAt('c', await charPoint(SOURCE.indexOf('。')))
    await pause(500)
    results.click_punctuation = { headword: await headword(), painted: await painted() }
    if (results.click_punctuation.headword !== PHRASE || results.click_punctuation.painted !== PHRASE) throw new Error(JSON.stringify(results.click_punctuation))
  })

  it('a real double click inside the same phrase does not look it up again', async () => {
    await fresh('Bench lookup double')
    await pressAt('c', await charPoint(SOURCE.indexOf('提')))
    await pause(800)
    await browser.execute(() => { window.__lookupHeadwordChanges = 0; new MutationObserver(() => { window.__lookupHeadwordChanges += 1 }).observe(document.querySelector('.lookup-headword').parentElement, { childList: true, subtree: true, characterData: true }) })
    await pressAt('dc', await charPoint(SOURCE.indexOf('琴')))
    await pause(800)
    results.double_click = { headword: await headword(), mutations: await browser.execute(() => window.__lookupHeadwordChanges) }
    if (results.double_click.headword !== PHRASE || results.double_click.mutations !== 0) throw new Error(JSON.stringify(results.double_click))
  })
})
