// Real-key bench: delete at the start of a target cell must leave grid and disk unchanged; ⌘M and the sentence menu still merge.
// BENCH_PHASE=truoc|sau names the output.
import { execFileSync } from 'node:child_process'
import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { waitForGridRows } from '../../../e2e/support/gridWait.mjs'
import { resetPanelState } from '../../../e2e/support/panelReset.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const PHASE = process.env.BENCH_PHASE ?? 'sau'
const OUT = join(HERE, `ket-qua-${PHASE}.json`)
const results = { presses: [] }
const WORK = `bench-backspace-${String(Date.now() % 1_000_000)}`
const PAIRS = 9

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

const raw = (...args) => execFileSync('cliclick', ['-e', '40', ...args], { encoding: 'utf8' })
const cli = (...args) => {
  ensureFront()
  return raw(...args)
}
const pause = (ms) => browser.pause(ms)

let offset = null
let calibrated = null

async function installProbe() {
  await browser.execute(() => {
    if (window.__bench !== undefined) return
    window.__bench = { down: null, log: [], watchSel: null, scrollAt: {} }
    window.addEventListener('mousedown', (e) => {
      window.__bench.down = [e.clientX, e.clientY]
    }, true)
    window.addEventListener('keydown', (e) => {
      window.__bench.log.push(`keydown ${e.key} target=${e.target.tagName}.${String(e.target.className).split(' ')[0]}`)
    }, true)
    const name = (t) => `${t.tagName}.${String(t.className).split(' ')[0]}:${(t.textContent ?? '').trim().slice(0, 14)}`
    for (const type of ['pointerdown', 'pointerup', 'pointercancel', 'mousedown', 'mouseup', 'click', 'focusin', 'focusout', 'blur', 'dragstart']) {
      window.addEventListener(type, (e) => {
        window.__bench.log.push(`${type} ${name(e.target)} @${String(e.clientX)},${String(e.clientY)} btn=${String(e.button)}`)
        if (window.__bench.watchSel !== null && (type === 'mousedown' || type === 'mouseup' || type === 'click')) {
          window.__bench.scrollAt[type] = document.querySelector(window.__bench.watchSel)?.scrollTop ?? null
        }
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
  let pt = null
  for (let attempt = 0; attempt < 4 && pt === null; attempt += 1) {
    cli(`dc:${guess.x},${guess.y}`)
    await pause(800)
    pt = await browser.execute(() => window.__bench.down)
    results.calibration_attempts = attempt + 1
  }
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
  ensureFront()
  raw(`m:${screen(x, y + 60)}`)
  await pause(120)
  raw(`m:${screen(x, y)}`)
  await pause(200)
  raw(`dd:${screen(x, y)}`, `w:${String(holdMs)}`, `du:${screen(x, y)}`)
  await pause(350)
}

async function key(name) {
  await browser.execute(() => { window.__bench.log.length = 0 })
  ensureFront()
  const code = { esc: 53, 'arrow-down': 125, 'arrow-up': 126, return: 36 }[name]
  execFileSync('osascript', ['-e', `tell application "System Events" to key code ${String(code)}`])
  await pause(350)
  const keyLog = await browser.execute(() => window.__bench.log.filter((l) => l.startsWith('keydown')))
  ;(results.keys ??= []).push({ name, keyLog, win: await winState() })
}

async function typeText(text) {
  cli(`t:${text}`)
  await pause(350)
}

const invoke = (command, args = {}) =>
  browser.execute(async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a), command, args)

const centreOf = (selector, index = 0) =>
  browser.execute((sel, i) => {
    const el = document.querySelectorAll(sel)[i]
    if (el === undefined) return null
    const r = el.getBoundingClientRect()
    return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
  }, selector, index)

// A press whose click never arrived did nothing (no `click`, no submit): it is repeated at most twice
// and every physical press is recorded, so the lost ones are counted instead of hidden.
async function pressAt(label, pointOrFn, { prepare = null, max = 3 } = {}) {
  const attempts = []
  let rec = null
  for (let n = 0; n < max; n += 1) {
    if (prepare !== null) await prepare()
    const p = typeof pointOrFn === 'function' ? await pointOrFn() : pointOrFn
    if (p === null) throw new Error(`${label}: no target to press`)
    await browser.execute(() => { window.__bench.log.length = 0; window.__bench.scrollAt = {} })
    await press(p.x, p.y)
    const log = await browser.execute(() => window.__bench.log.slice())
    const scrollAt = await browser.execute(() => ({ ...window.__bench.scrollAt }))
    const has = (type) => log.some((line) => line.startsWith(`${type} `))
    rec = { down: has('mousedown'), up: has('mouseup'), click: has('click'), log: log.slice(0, 16), scrollAt }
    attempts.push({ down: rec.down, up: rec.up, click: rec.click, ...(rec.down ? {} : { win: await winState(), point: p }) })
    if (rec.click) break
  }
  const entry = { label, attempts, ...rec }
  results.presses.push(entry)
  return entry
}

const pressSel = (label, selector, index = 0, opts = {}) => pressAt(label, () => centreOf(selector, index), opts)

const winState = () =>
  browser.execute(() => ({ sx: window.screenX, sy: window.screenY, iw: window.innerWidth, ih: window.innerHeight, focus: document.hasFocus(), visible: document.visibilityState }))

const describeActive = () =>
  browser.execute(() => {
    const el = document.activeElement
    if (el === null) return 'null'
    const data = [...el.attributes].filter((a) => a.name.startsWith('data-')).map((a) => a.name)
    return `${el.tagName}.${String(el.className).split(' ').filter(Boolean).slice(0, 2).join('.')}${el.id === '' ? '' : `#${el.id}`}[${data.join(',')}]`
  })

const exists = (sel) => browser.execute((s) => document.querySelector(s) !== null, sel)
const waitFor = (fn, msg, timeout = 10_000) => browser.waitUntil(fn, { timeout, interval: 150, timeoutMsg: msg })

const note = (key2, value) => { results[key2] = value }


async function seedBilingual() {
  const dir = mkdtempSync(join(tmpdir(), 'aura-bench-backspace-'))
  const path = join(dir, 'song-ngu.csv')
  const rows = Array.from({ length: PAIRS }, (_, i) => `第${String(i + 1)}句。,Ban dich ${String(i + 1)}.`)
  writeFileSync(path, `${rows.join('\n')}\n`)
  const bilingual = {
    sourceLang: 'zh', chapterPattern: null, sourceColumn: 0, targetColumn: 1, hasHeader: false, tableIndex: null,
  }
  await invoke('preview_bilingual_import_from_file', { path, ...bilingual })
  await invoke('confirm_bilingual_import', {
    name: WORK, genre: 'general', encoding: 'UTF-8', regroupings: [], importMismatchedRows: false, ...bilingual,
  })
  await resetPanelState()
  await browser.keys(['Meta', '2'])
  await waitForGridRows(PAIRS)
  const loaded = await invoke('read_open_chapter_segments', { prefill: false })
  results.chapterId = loaded.chapter_id
  return loaded.segments
}

async function rightPress(x, y) {
  ensureFront()
  raw(`m:${screen(x, y + 60)}`)
  await pause(120)
  raw(`m:${screen(x, y)}`)
  await pause(200)
  raw(`rc:${screen(x, y)}`)
  await pause(450)
}

const pressDelete = (times) => {
  ensureFront()
  for (let n = 0; n < times; n += 1) {
    execFileSync('osascript', ['-e', 'tell application "System Events" to key code 51'])
    execFileSync('sleep', ['0.25'])
  }
}

const readGrid = () =>
  browser.execute(() => ({
    ids: [...document.querySelectorAll('[data-col="tgt"]')].map((el) => el.getAttribute('data-segment-id')),
    texts: [...document.querySelectorAll('[data-col="tgt"]')].map((el) => el.textContent),
    notice: document.querySelector('.status .notice')?.textContent?.trim() ?? null,
    active: document.activeElement?.getAttribute('data-segment-id') ?? null,
  }))

const readDisk = async () => {
  const loaded = await invoke('read_open_chapter_segments', { prefill: false })
  return loaded.segments.map((s) => ({ id: s.id, target: s.target_text, status: s.status }))
}

const caretAt = (id, offset) =>
  browser.execute((segmentId, at) => {
    const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
    cell.focus()
    const range = document.createRange()
    range.setStart(cell.firstChild ?? cell, at)
    range.collapse(true)
    const sel = window.getSelection()
    sel.removeAllRanges()
    sel.addRange(range)
    return document.activeElement === cell
  }, id, offset)

const clickCell = (id) => pressSel(`cell ${id}`, `[data-col="tgt"][data-segment-id="${id}"]`)

describe(`backspace bench ${PHASE}`, () => {
  let ids = []

  afterEach(() => {
    writeFileSync(OUT, `${JSON.stringify(results, null, 2)}\n`)
  })

  it('0: opens the Work, calibrates real input', async function () {
    this.timeout(480_000)
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), { timeout: 30_000, interval: 250 })
    const segments = await seedBilingual()
    ids = segments.map((s) => s.id)
    await browser.setWindowSize(2600, 1800)
    await pause(500)
    await installProbe()
    await calibrate()
    await waitForSingleClicks()
    note('load', execFileSync('uptime', { encoding: 'utf8' }).trim())
  })

  it('text kept after the caret: six deletes from offset 3', async function () {
    this.timeout(120_000)
    const id = ids[4]
    await clickCell(id)
    note('text_focus', await caretAt(id, 3))
    await browser.execute(() => { window.__bench.log.length = 0 })
    pressDelete(6)
    await pause(800)
    const log = await browser.execute(() => window.__bench.log.filter((l) => l.startsWith('keydown')))
    await pause(3500)
    note('text_after', { keydowns: log.length, grid: await readGrid(), disk: await readDisk() })
  })

  it('emptied cell: select all, delete, then five more', async function () {
    this.timeout(120_000)
    const id = ids[7]
    await clickCell(id)
    await browser.execute((segmentId) => {
      const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
      cell.focus()
      window.getSelection().selectAllChildren(cell)
    }, id)
    await browser.execute(() => { window.__bench.log.length = 0 })
    pressDelete(6)
    await pause(800)
    const log = await browser.execute(() => window.__bench.log.filter((l) => l.startsWith('keydown')))
    await pause(3500)
    note('empty_after', { keydowns: log.length, grid: await readGrid(), disk: await readDisk() })
  })

  it('⌘M still merges at a chosen caret', async function () {
    this.timeout(120_000)
    const before = await readGrid()
    const id = before.ids[before.ids.length - 1]
    await clickCell(id)
    ensureFront()
    execFileSync('osascript', ['-e', 'tell application "System Events" to keystroke "m" using command down'])
    await waitFor(async () => (await readGrid()).ids.length === before.ids.length - 1, 'real ⌘M did not merge')
    note('chord_after', { rowsBefore: before.ids.length, grid: await readGrid() })
  })

  it('sentence menu still merges', async function () {
    this.timeout(120_000)
    const before = await readGrid()
    const id = before.ids[2]
    await clickCell(id)
    const cell = await centreOf(`[data-col="tgt"][data-segment-id="${id}"]`)
    await rightPress(cell.x, cell.y)
    await pressSel('menu merge', '.ui-menu-item[title^="Gộp câu này"]')
    await waitFor(async () => (await readGrid()).ids.length === before.ids.length - 1, 'menu item did not merge')
    note('menu_after', { rowsBefore: before.ids.length, grid: await readGrid() })
  })
})
