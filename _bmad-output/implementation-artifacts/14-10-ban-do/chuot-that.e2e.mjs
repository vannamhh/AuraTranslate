// Real-mouse bench: cliclick and osascript post trusted input into the app window while the wdio
// session only reads state back. Measures the sentence menu across a Cmd-Tab and the Esc caret after the History overlay.
import { execFileSync } from 'node:child_process'
import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { waitForGridRows } from '../../../e2e/support/gridWait.mjs'
import { resetPanelState } from '../../../e2e/support/panelReset.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const OUT = join(HERE, 'ket-qua.json')
const results = { presses: [] }
const WORK = `bench-1410-${String(Date.now() % 1_000_000)}`
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
  const dir = mkdtempSync(join(tmpdir(), 'aura-bench-1410-'))
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

async function osaKey(code, using = '') {
  ensureFront()
  execFileSync('osascript', ['-e', `tell application "System Events" to key code ${String(code)}${using}`])
  await pause(400)
}

async function waitFront() {
  const started = Date.now()
  while (Date.now() - started < 15_000) {
    if (frontPid() === appPid()) return Date.now() - started
    await pause(100)
  }
  return null
}

describe('14-10 bench', () => {
  let segments = []
  const cellSel = (i) => `[data-col="tgt"][data-segment-id="${String(segments[i].id)}"]`
  const cellText = (i) => browser.execute((sel) => document.querySelector(sel).textContent, cellSel(i))
  const caretIn = (i) =>
    browser.execute((sel) => {
      const el = document.querySelector(sel)
      const s = window.getSelection()
      if (el === null || s === null || s.rangeCount === 0 || !el.contains(s.anchorNode)) return null
      const r = document.createRange()
      r.selectNodeContents(el)
      r.setEnd(s.anchorNode, s.anchorOffset)
      return { at: r.toString().length, collapsed: s.isCollapsed, selected: s.toString() }
    }, cellSel(i))
  const setCaret = (i, n) =>
    browser.execute((sel, at) => {
      const el = document.querySelector(sel)
      el.focus()
      const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT)
      let left = at
      let node = walker.nextNode()
      while (node !== null && left > node.length) { left -= node.length; node = walker.nextNode() }
      const r = document.createRange()
      r.setStart(node, left)
      r.collapse(true)
      const s = window.getSelection()
      s.removeAllRanges()
      s.addRange(r)
    }, cellSel(i), n)
  const menuOpen = () => exists('[data-sentence-menu]')
  const itemCentre = (label) =>
    browser.execute((text) => {
      const el = [...document.querySelectorAll('[data-sentence-menu] button')].find((b) => b.textContent.includes(text))
      if (el === undefined) return null
      const r = el.getBoundingClientRect()
      return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
    }, label)
  const omitted = () => browser.execute(() => [...document.querySelectorAll('[data-col="src"]')].map((e) => e.classList.contains('omitted')))
  const cellPoint = (i, fraction) =>
    browser.execute((sel, f) => {
      const el = document.querySelector(sel)
      const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT)
      const node = walker.nextNode()
      const r = document.createRange()
      r.setStart(node, 0)
      r.setEnd(node, Math.max(1, Math.floor(node.length * f)))
      const rects = r.getClientRects()
      const last = rects[rects.length - 1]
      return { x: last.right - 2, y: last.y + last.height / 2 }
    }, cellSel(i), fraction)
  const log = () => browser.execute(() => window.__bench.log.slice())

  afterEach(() => {
    writeFileSync(OUT, `${JSON.stringify(results, null, 2)}\n`)
  })

  it('0: opens the Work, calibrates real input', async function () {
    this.timeout(480_000)
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), { timeout: 30_000, interval: 250 })
    segments = await seedBilingual()
    await browser.setWindowSize(2600, 1800)
    await pause(500)
    await installProbe()
    await calibrate()
    await waitForSingleClicks()
    note('segments', segments.length)
  })

  for (const round of [1, 2, 3]) {
    it(`A${String(round)}: sentence menu open, Cmd-Tab away and back, then press an item`, async function () {
      this.timeout(240_000)
      await pressAt(`A${String(round)} cell`, () => cellPoint(0, 0.5))
      await setCaret(0, 4)
      const before = { omitted: await omitted(), caret: await caretIn(0) }
      await browser.execute(() => { window.__bench.log.length = 0 })
      const target = 5 + round
      const p = await cellPoint(target, 0.5)
      await rightPress(p.x, p.y)
      const opened = await menuOpen()
      cmdTab()
      const away = { front: frontPid() === appPid(), hasFocus: await browser.execute(() => document.hasFocus()) }
      cmdTab()
      const backMs = await waitFront()
      await pause(1500)
      const afterBack = { menu: await menuOpen(), caret: await browser.execute(() => {
        const s = window.getSelection()
        const a = s?.anchorNode
        const from = a instanceof Element ? a : a?.parentElement ?? null
        return from?.closest('[data-col="tgt"]')?.getAttribute('data-segment-id') ?? null
      }), active: await describeActive() }
      const events = (await log()).filter((l) => /^(focusout|blur|focusin) /.test(l))
      let pressRec = null
      let item = null
      if (afterBack.menu) {
        item = await itemCentre('Cắt bỏ câu')
        pressRec = await pressAt(`A${String(round)} Cắt bỏ câu`, () => itemCentre('Cắt bỏ câu'))
        await pause(600)
      }
      const final = { menu: await menuOpen(), omitted: await omitted() }
      note(`menuCmdTab${String(round)}`, { target, opened, away, backMs, afterBack, events, item: item !== null, pressed: pressRec === null ? null : { down: pressRec.down, up: pressRec.up, click: pressRec.click }, before, final })
      if (final.menu) await osaKey(53)
    })
  }

  it('B: no menu, Cmd-Tab away and back, typing continues at the caret', async function () {
    this.timeout(240_000)
    await pressAt('B cell', () => cellPoint(1, 0.5))
    await setCaret(1, 4)
    const pre = { text: await cellText(1), caret: await caretIn(1) }
    cmdTab()
    cmdTab()
    const backMs = await waitFront()
    await pause(1500)
    const mid = { caret: await caretIn(1), active: await describeActive() }
    await typeText('q')
    await pause(700)
    const text = await cellText(1)
    note('plainCmdTab', { pre, backMs, mid, text, ok: text === `${pre.text.slice(0, 4)}q${pre.text.slice(4)}` })
  })

  it('C: Mod+H real chord, Esc, type zz', async function () {
    this.timeout(240_000)
    await pressAt('C cell', () => cellPoint(3, 0.5))
    await setCaret(3, 4)
    const pre = { text: await cellText(3), caret: await caretIn(3) }
    ensureFront()
    execFileSync('osascript', ['-e', 'tell application "System Events" to keystroke "h" using command down'])
    await pause(800)
    const opened = await exists('.hist-panel')
    await osaKey(53)
    await pause(500)
    const afterEsc = { overlay: await exists('.hist-panel'), caret: await caretIn(3), active: await describeActive() }
    await typeText('zz')
    await pause(700)
    const text = await cellText(3)
    note('modH', { pre, opened, afterEsc, text, ok: text === `${pre.text.slice(0, 4)}zz${pre.text.slice(4)}` })
  })

  it('D: menu item Lịch sử, Esc, type zz (WebKit word selection on right click)', async function () {
    this.timeout(240_000)
    await pressAt('D cell', () => cellPoint(4, 0.5))
    await setCaret(4, 4)
    const pre = { text: await cellText(4), caret: await caretIn(4) }
    const p = await cellPoint(4, 0.6)
    await rightPress(p.x, p.y)
    const opened = await menuOpen()
    const atOpen = await caretIn(4)
    const rec = await pressAt('D Lịch sử', () => itemCentre('Lịch sử phiên bản'))
    await pause(800)
    const overlay = await exists('.hist-panel')
    await osaKey(53)
    await pause(500)
    const afterEsc = { overlay: await exists('.hist-panel'), caret: await caretIn(4), active: await describeActive() }
    await typeText('zz')
    await pause(700)
    const text = await cellText(4)
    const lost = pre.text.length + 2 !== text.length
    note('menuHistory', { pre, opened, atOpen, rec: { down: rec.down, up: rec.up, click: rec.click }, overlay, afterEsc, text, noCharLost: !lost, zzPresent: text.includes('zz') })
  })
})
