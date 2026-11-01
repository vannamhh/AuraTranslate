// Real-mouse bench: for every surface that takes focus away from a target cell, type Q while it shows, Esc, type zz,
// then read the cell and target_text from disk. BENCH_PHASE=truoc|sau names the output.
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
const WORK = `bench-1411-${String(Date.now() % 1_000_000)}`
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
  const dir = mkdtempSync(join(tmpdir(), 'aura-bench-1411-'))
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


const TOOLS = [
  'glossary.manage.open',
  'glossary.settings.open',
  'tm.manage.open',
  'settings.open',
  'ai.proofread.scope.open',
  'ai.proofread.memory.open',
  'export.reviewer_import.open',
  'export.alignment.open',
  'glossary.harvest.open',
]
const DIRECT = [
  ['status.glossary_queue', '[data-glossary-queue-open]'],
  ['status.lookup_drawer', '[data-lookup-drawer-open]'],
  ['ai.prompt_inspector', '[data-ai-prompt-inspector-open]'],
  ['ai.prompt_library_link', '[data-ai-prompt-library-link]'],
  ['ai.settings_link', '[data-ai-unconfigured-open-settings]'],
  ['lookup.attribution', '[data-attribution-open]'],
  ['strip.memory', '[data-proofread-strip-memory]'],
  ['review.notice_import', '[data-review-notice-import]'],
  ['review.align_link', '[data-review-align-link]'],
  ['lookup.sources_menu', '[data-sources-trigger]'],
]
const CHORDS = [
  ['chord.glossary_manage', 'keystroke "m" using {command down, option down}'],
  ['chord.settings', 'keystroke "," using command down'],
  ['chord.history', 'keystroke "h" using command down'],
]

describe(`14-11 bench ${PHASE}`, () => {
  let segments = []
  const ID = () => segments[4].id
  const cellSel = () => `[data-col="tgt"][data-segment-id="${String(ID())}"]`
  const cellText = () => browser.execute((sel) => document.querySelector(sel)?.textContent ?? null, cellSel())
  const caretIn = () =>
    browser.execute((sel) => {
      const el = document.querySelector(sel)
      const s = window.getSelection()
      if (el === null || s === null || s.rangeCount === 0 || !el.contains(s.anchorNode)) return null
      const r = document.createRange()
      r.selectNodeContents(el)
      r.setEnd(s.anchorNode, s.anchorOffset)
      return { at: r.toString().length, collapsed: s.isCollapsed }
    }, cellSel())
  const setCaret = (n) =>
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
    }, cellSel(), n)
  const cellPoint = (fraction) =>
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
    }, cellSel(), fraction)
  const shown = () => browser.execute(() => document.querySelectorAll('[role="dialog"], [role="menu"], [data-sentence-menu]').length)
  const visible = (sel) =>
    browser.execute((s) => {
      const el = document.querySelector(s)
      if (el === null) return false
      const r = el.getBoundingClientRect()
      return r.width > 0 && r.height > 0
    }, sel)
  const diskText = async () => {
    const loaded = await invoke('read_open_chapter_segments', { prefill: false })
    return loaded.segments.find((s) => s.id === ID())?.target_text ?? null
  }
  const menuItemCentre = (label) =>
    browser.execute((text) => {
      const el = [...document.querySelectorAll('[data-sentence-menu] button')].find((b) => b.textContent.includes(text))
      if (el === undefined) return null
      const r = el.getBoundingClientRect()
      return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
    }, label)

  const closeButtonCentre = () =>
    browser.execute(() => {
      const el = [...document.querySelectorAll('[role="dialog"] button')].find((b) => /^\s*(Đóng|Hủy|Huỷ)/.test(b.textContent ?? '') || /Đóng/.test(b.getAttribute('aria-label') ?? ''))
      if (el === undefined) return null
      const r = el.getBoundingClientRect()
      return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
    })

  async function settle() {
    for (let i = 0; i < 4 && (await shown()) > 0; i += 1) {
      const btn = i === 0 ? null : await closeButtonCentre()
      if (btn !== null) await press(btn.x, btn.y)
      else await osaKey(53)
      await pause(500)
    }
  }

  async function prepare() {
    await settle()
    await pressAt('cell', () => cellPoint(0.5))
    await setCaret(4)
    return { text: await cellText(), caret: await caretIn() }
  }

  async function finish(id, pre, extra) {
    const showing = await shown()
    const showingExtra = extra.visibleSel === undefined ? null : await visible(extra.visibleSel)
    await typeText('Q')
    await pause(500)
    const afterQ = await cellText()
    const caretQ = await caretIn()
    const active = await describeActive()
    if (extra.clickAway === true) {
      const pt = await browser.execute(() => {
        const el = document.querySelector('[data-titlebar-work-name]') ?? document.querySelector('.titlebar')
        const r = el.getBoundingClientRect()
        return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
      })
      await press(pt.x, pt.y)
    } else {
      await osaKey(53)
    }
    await pause(600)
    const escClosed = (await shown()) === 0
    const afterEsc = { escClosed, shown: await shown(), caret: await caretIn(), active: await describeActive() }
    if (!escClosed) await settle()
    await typeText('zz')
    await pause(500)
    const afterZz = await cellText()
    await pause(3500)
    const disk = await diskText()
    const leakedQ = afterQ !== pre.text
    const expected = `${pre.text.slice(0, 4)}zz${pre.text.slice(4)}`
    const rec = {
      pre, showing, showingExtra, escClosed, activeWhileShown: active, afterQ, caretQ, leakedQ,
      afterEsc, afterZz, disk, zzAtOldPoint: afterZz === expected, diskMatchesCell: disk === afterZz,
      qOnDisk: typeof disk === 'string' && disk.includes('Q'),
    }
    note(id, rec)
    // Restore a known cell text so later surfaces start clean.
    await invoke('save_segment_targets', { chapterId: results.chapterId, edits: [{ id: ID(), target_text: pre.text }] })
    await resetPanelState()
    await browser.keys(['Meta', '2'])
    await waitForGridRows(PAIRS)
    await pause(500)
  }

  async function openAndMeasure(id, openFn, extra = {}) {
    const pre = await prepare()
    let opened = null
    try {
      opened = await openFn()
    } catch (e) {
      note(id, { skipped: String(e.message).slice(0, 160) })
      return
    }
    if (opened === false) {
      note(id, { skipped: 'opener not present in this state' })
      return
    }
    await finish(id, pre, extra)
  }

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
    note('load', execFileSync('uptime', { encoding: 'utf8' }).trim())
  })

  for (const item of TOOLS) {
    it(`tools ${item}`, async function () {
      this.timeout(120_000)
      await openAndMeasure(`tools.${item}`, async () => {
        const trigger = await pressSel('trigger', '.tools-trigger')
        if (!trigger.click) return false
        const pressed = await pressSel(item, `[data-tools-item="${item}"]`)
        if (!pressed.click) return false
        await pause(700)
        return true
      })
    })
  }

  it('tools menu itself open', async function () {
    this.timeout(120_000)
    await openAndMeasure('tools.menu_open', async () => (await pressSel('trigger', '.tools-trigger')).click, { clickAway: true })
  })

  for (const [id, sel] of DIRECT) {
    it(id, async function () {
      this.timeout(120_000)
      await openAndMeasure(id, async () => {
        if (!(await visible(sel))) return false
        const pressed = await pressSel(id, sel)
        await pause(600)
        return pressed.click
      }, { clickAway: id === 'lookup.sources_menu' })
    })
  }

  for (const [id, index] of [['tab.library', 0], ['tab.reading', 2]]) {
    it(id, async function () {
      this.timeout(300_000)
      const runs = []
      for (let n = 0; n < Number(process.env.BENCH_TAB_N ?? 1); n += 1) {
        const pre = await prepare()
        const tab = await pressSel(id, '.mode-tab', index)
        await pause(900)
        await typeText('Q')
        await pause(500)
        await pressSel(`${id} back`, '.mode-tab', 1)
        await pause(1200)
        await waitForGridRows(PAIRS)
        const afterQ = await cellText()
        const caret = await caretIn()
        await typeText('zz')
        await pause(500)
        const afterZz = await cellText()
        await pause(3500)
        const disk = await diskText()
        runs.push({ pre, clicked: tab.click, afterQ, leakedQ: afterQ !== pre.text, caretAfterReturn: caret, afterZz, disk, qOnDisk: typeof disk === 'string' && /q/i.test(disk) })
        await invoke('save_segment_targets', { chapterId: results.chapterId, edits: [{ id: ID(), target_text: pre.text }] })
        await resetPanelState()
        await browser.keys(['Meta', '2'])
        await waitForGridRows(PAIRS)
      }
      note(id, { ...runs[runs.length - 1], runs: runs.length, leaks: runs.filter((r) => r.leakedQ).length })
    })
  }

  for (const [id, script] of CHORDS) {
    it(id, async function () {
      this.timeout(120_000)
      await openAndMeasure(id, async () => {
        ensureFront()
        execFileSync('osascript', ['-e', `tell application "System Events" to ${script}`])
        await pause(900)
        return (await shown()) > 0
      })
    })
  }

  async function restoreWindow() {
    await browser.setWindowSize(2600, 1800)
    await pause(700)
  }

  it('status.lookup_drawer_narrow: shrink to the tier that offers the drawer button', async function () {
    this.timeout(180_000)
    let found = null
    await openAndMeasure('status.lookup_drawer_narrow', async () => {
      for (const width of [2000, 1600, 1300, 1100, 900, 760]) {
        await browser.setWindowSize(width, 1400)
        await pause(900)
        if (await visible('[data-lookup-drawer-open]')) {
          found = width
          break
        }
      }
      note('lookup_drawer_width', found)
      if (found === null) return false
      await pause(300)
      await pressAt('cell', () => cellPoint(0.5))
      await setCaret(4)
      const pressed = await pressSel('status.lookup_drawer_narrow', '[data-lookup-drawer-open]')
      await pause(700)
      return pressed.click
    })
    await restoreWindow()
    await settle()
  })

  it('tools.menu_to_status_button: menu open, then straight to a status-bar button', async function () {
    this.timeout(120_000)
    await openAndMeasure('tools.menu_to_status_button', async () => {
      const trigger = await pressSel('trigger', '.tools-trigger')
      if (!trigger.click) return false
      if (!(await visible('[data-glossary-queue-open]'))) return false
      const pressed = await pressSel('status button', '[data-glossary-queue-open]')
      await pause(700)
      return pressed.click
    })
  })

  for (const item of ['tm.manage.open', 'glossary.manage.open', 'settings.open']) {
    it(`overlay input field ${item}: typing lands in the overlay, not the cell`, async function () {
      this.timeout(120_000)
      const pre = await prepare()
      const trigger = await pressSel('trigger', '.tools-trigger')
      const pressed = await pressSel(item, `[data-tools-item="${item}"]`)
      await pause(900)
      const before = await browser.execute(() => [...document.querySelectorAll('[role="dialog"] input:not([type="checkbox"]):not([type="radio"]), [role="dialog"] textarea')].length)
      const active = await describeActive()
      await typeText('Q')
      await pause(500)
      const afterQ = await cellText()
      const inputs = await browser.execute(() => [...document.querySelectorAll('[role="dialog"] input, [role="dialog"] textarea')].map((e) => e.value))
      note(`input.${item}`, {
        clicked: trigger.click && pressed.click, inputsInDialog: before, activeAfterOpen: active, afterQ, leakedQ: afterQ !== pre.text,
        qInInput: inputs.some((v) => v.includes('Q')), inputValues: inputs,
      })
      await settle()
      await invoke('save_segment_targets', { chapterId: results.chapterId, edits: [{ id: ID(), target_text: pre.text }] })
      await resetPanelState()
      await browser.keys(['Meta', '2'])
      await waitForGridRows(PAIRS)
      await pause(500)
    })
  }

  it('input click tm.manage.open: real click into the search input, real typing lands in the input', async function () {
    this.timeout(120_000)
    const pre = await prepare()
    const trigger = await pressSel('trigger', '.tools-trigger')
    const pressed = await pressSel('tm.manage.open', '[data-tools-item="tm.manage.open"]')
    await pause(900)
    const inputSel = '.tm-panel input[type="search"], .tm-panel input[type="text"], .tm-panel input:not([type])'
    const hasInput = await visible(inputSel)
    let rec = { skipped: 'no visible text input in .tm-panel' }
    if (hasInput) {
      const clicked = await pressSel('tm search input', inputSel)
      const active = await describeActive()
      await typeText('Qa')
      await pause(500)
      const inputValue = await browser.execute((sel) => document.querySelector(sel).value, inputSel)
      const afterQ = await cellText()
      rec = {
        clicked: trigger.click && pressed.click && clicked.click, activeAfterClick: active, inputValue, afterQ,
        inputHoldsText: /q/i.test(inputValue) && inputValue.toLowerCase().includes('qa'), cellUnchanged: afterQ === pre.text,
        cellBefore: pre.text,
      }
    }
    note('input.click.tm.manage.open', rec)
    await settle()
    await invoke('save_segment_targets', { chapterId: results.chapterId, edits: [{ id: ID(), target_text: pre.text }] })
    await resetPanelState()
    await browser.keys(['Meta', '2'])
    await waitForGridRows(PAIRS)
  })

  it('status.chapter_done_then_type: pressing Đã xong leaves the selection alone', async function () {
    this.timeout(120_000)
    const pre = await prepare()
    let rec = { skipped: 'button not visible' }
    if (await visible('[data-chapter-done]')) {
      const pressed = await pressSel('chapter done', '[data-chapter-done]')
      await pause(700)
      const caretAfter = await caretIn()
      const active = await describeActive()
      await typeText('zz')
      await pause(500)
      const afterZz = await cellText()
      await pause(3500)
      const disk = await diskText()
      rec = {
        pre, clicked: pressed.click, caretAfter, active, afterZz, disk,
        zzAtOldPoint: afterZz === `${pre.text.slice(0, 4)}zz${pre.text.slice(4)}`, diskMatchesCell: disk === afterZz,
      }
    }
    note('status.chapter_done_then_type', rec)
    await invoke('save_segment_targets', { chapterId: results.chapterId, edits: [{ id: ID(), target_text: pre.text }] })
    await resetPanelState()
    await browser.keys(['Meta', '2'])
    await waitForGridRows(PAIRS)
  })

  it('menu.history via real right click', async function () {
    this.timeout(120_000)
    await openAndMeasure('menu.history', async () => {
      for (let attempt = 0; attempt < 3 && (await shown()) === 0; attempt += 1) {
        const p = await cellPoint(0.6)
        await rightPress(p.x, p.y)
        note(`menu.history.rightPresses`, attempt + 1)
      }
      if ((await shown()) === 0) return false
      const rec = await pressAt('Lịch sử', () => menuItemCentre('Lịch sử phiên bản'))
      await pause(900)
      return rec.click
    })
  })
})
