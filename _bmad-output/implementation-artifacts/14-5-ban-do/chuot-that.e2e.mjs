// Real-mouse bench: cliclick and System Events post trusted input into the app window while the
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

async function snapshot() {
  return await browser.execute(() => ({
    log: window.__bench.log.splice(0),
    caret: document.querySelector('.cell-tgt.editing')?.getAttribute('data-segment-id') ?? null,
    menu: document.querySelector('[data-sentence-menu]') !== null,
    notice: document.querySelector('[data-review-notice]') !== null,
    history: document.querySelector('.hist-panel') !== null,
    selection: String(window.getSelection()),
    tgtText: [...document.querySelectorAll('[data-col="tgt"]')].map((el) => el.textContent),
    activeElement: `${document.activeElement?.tagName}:${document.activeElement?.getAttribute('data-segment-id') ?? ''}`,
    selected: [...document.querySelectorAll('[data-num-segment-id].row-selected')].map((e) => e.getAttribute('data-num-segment-id')),
    status: document.querySelector('[data-status-live]')?.textContent?.trim() ?? null,
  }))
}

const SEL = {
  tgt: (id) => `[data-col="tgt"][data-segment-id="${id}"]`,
}

async function ids() {
  return await browser.execute(() => [...document.querySelectorAll('[data-col="src"]')].map((el) => el.getAttribute('data-segment-id')))
}

async function selectionHost() {
  return await browser.execute(() => {
    const s = window.getSelection()
    const n = s?.anchorNode ?? null
    const el = n instanceof Element ? n : n?.parentElement ?? null
    return { type: s?.type, col: el?.closest('[data-col]')?.getAttribute('data-col') ?? null, id: el?.closest('[data-col]')?.getAttribute('data-segment-id') ?? null, offset: s?.anchorOffset }
  })
}

const esc = () => execFileSync('osascript', ['-e', 'tell application "System Events" to key code 53'])

async function press(kind, selector) {
  for (let attempt = 0; attempt < 4; attempt += 1) {
    await browser.execute(() => { window.__bench.down = null })
    const point = await screenPoint(selector)
    cli(`m:${point}`, 'w:200', `${kind}:${point}`)
    await pause(350)
    if ((await browser.execute(() => window.__bench.down)) !== null) return
    await pause(300)
  }
  throw new Error(`press never reached the page: ${selector}`)
}

async function fresh(name) {
  await openWorkspaceWithWork(name, '一二三。四五六。七八九。')
  await waitForGridRows(3, { what: 'bench' })
  await installProbe()
  for (let attempt = 0; attempt < 5 && offset === null; attempt += 1) {
    await pause(1200)
    await calibrate().catch((err) => {
      results.calibration_errors = [...(results.calibration_errors ?? []), `${String(err)} app=${appPid()} front=${frontPid()}`]
    })
  }
  if (offset === null) throw new Error(`no calibration: ${String(results.calibration_errors)}`)
  await waitForSingleClicks()
  return await ids()
}

async function typeInto(id, text) {
  await press('c', SEL.tgt(id))
  cli(`t:${text}`)
  await pause(300)
  await snapshot()
}


const STRIP_INPUT = '.glossary-quick-add .gqa-input'

const SEL2 = {
  src: (id) => `[data-col="src"][data-segment-id="${id}"]`,
  tgt: (id) => `[data-col="tgt"][data-segment-id="${id}"]`,
  num: (id) => `[data-num-segment-id="${id}"]`,
}

async function stripState() {
  return await browser.execute((sel) => {
    const input = document.querySelector(sel)
    return {
      open: input !== null,
      value: input instanceof HTMLInputElement ? input.value : null,
      focusInStrip: document.activeElement?.closest('.glossary-quick-add') !== null && document.activeElement?.closest('.glossary-quick-add') !== undefined,
    }
  }, STRIP_INPUT)
}

async function viaMenu(kind, selector) {
  await press(kind, selector)
  await browser.execute(() => {
    const b = [...document.querySelectorAll('[data-sentence-menu] button')].find((x) => x.textContent.includes('Thêm thuật ngữ'))
    b.setAttribute('data-bench-add-term', '')
  })
  await snapshot()
  for (let attempt = 0; attempt < 4; attempt += 1) {
    await press('c', '[data-bench-add-term]')
    await pause(500)
    results.menu_clicks = (results.menu_clicks ?? 0) + 1
    if ((await stripState()).open) return
  }
  throw new Error('strip never opened')
}

describe('real input', () => {
  after(() => {
    const file = join(HERE, 'ket-qua.json')
    const prior = existsSync(file) ? JSON.parse(readFileSync(file, 'utf8')) : {}
    writeFileSync(file, JSON.stringify({ ...prior, ...results }, null, 2))
  })

  for (const column of ['src', 'tgt']) {
    it(`right press inside a selected word at column ${column}: strip filled, a key goes into it`, async () => {
      const all = await fresh(`Bench glossary word ${column}`)
      if (column === 'tgt') await typeInto(all[1], 'alpha beta')
      await press('dc', SEL2[column](all[1]))
      await pause(300)
      results[`word_${column}_selected`] = await snapshot()
      await viaMenu('rc', SEL2[column](all[1]))
      results[`word_${column}_strip`] = { ...(await stripState()), ...(await snapshot()) }
      cli('t:Q')
      await pause(300)
      results[`word_${column}_typed`] = { ...(await stripState()), ...(await snapshot()) }
    })
  }

  it('right press on a number cell with a word selected elsewhere: strip empty and focused', async () => {
    const all = await fresh('Bench glossary number')
    await press('dc', SEL2.src(all[0]))
    await pause(300)
    await viaMenu('rc', SEL2.num(all[2]))
    results.number_strip = { ...(await stripState()), ...(await snapshot()) }
    cli('t:Q')
    await pause(300)
    results.number_typed = { ...(await stripState()), ...(await snapshot()) }
  })

  for (const column of ['src', 'num', 'tgt']) {
    it(`Escape after aiming sentence 3 at column ${column}: caret back, a key lands`, async () => {
      const all = await fresh(`Bench glossary esc ${column}`)
      await typeInto(all[0], 'abcd')
      cli('kp:arrow-left', 'kp:arrow-left')
      await pause(300)
      results[`esc_${column}_before`] = { ...(await snapshot()), sel: await selectionHost() }
      await viaMenu('rc', SEL2[column](all[2]))
      results[`esc_${column}_strip`] = await stripState()
      ensureFront()
      esc()
      await pause(700)
      results[`esc_${column}_after`] = { ...(await snapshot()), sel: await selectionHost(), strip: await stripState() }
      cli('t:Z')
      await pause(400)
      results[`esc_${column}_typed`] = { ...(await snapshot()), sel: await selectionHost() }
    })
  }
})
