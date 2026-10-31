// Real-mouse bench for Story 14.9: cliclick posts trusted input into the app window while the wdio
// session only reads state back. Presses the eight new or rewired buttons and reads their effect.
import { execFileSync } from 'node:child_process'
import { existsSync, mkdtempSync, readdirSync, statSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { waitForGridRows } from '../../../e2e/support/gridWait.mjs'
import { resetPanelState } from '../../../e2e/support/panelReset.mjs'
import { realClick } from '../../../e2e/support/pointer.mjs'
import { aimWork, openAimedWork, reloadWorks } from '../../../e2e/support/libraryGrid.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const OUT = join(HERE, 'ket-qua.json')
const results = { presses: [] }
const WORK = `bench-149-${String(Date.now() % 1_000_000)}`
const PAIRS = 26

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

function appWindows() {
  const script = `tell application "System Events" to tell (first process whose unix id is ${appPid()}) to return (count of windows) & "," & (count of (windows whose subrole is "AXDialog" or subrole is "AXSystemDialog" or role description is "dialog"))`
  try {
    return execFileSync('osascript', ['-e', script], { encoding: 'utf8' }).trim()
  } catch (error) {
    return `err:${String(error.message).slice(0, 80)}`
  }
}
const sheetCount = () => {
  const script = `tell application "System Events" to tell (first process whose unix id is ${appPid()}) to return (count of sheets of window 1)`
  try {
    return execFileSync('osascript', ['-e', script], { encoding: 'utf8' }).trim()
  } catch (error) {
    return `err:${String(error.message).slice(0, 80)}`
  }
}

async function seedBilingual() {
  const dir = mkdtempSync(join(tmpdir(), 'aura-bench-149-'))
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
  // Segment 0 stays unconfirmed so it can be typed into; the other 25 become TM pairs.
  for (const segment of loaded.segments.slice(1)) await invoke('confirm_segment', { segmentId: segment.id })
  return loaded.segments
}

async function recover(label) {
  let n = 0
  while (n < 4 && (await exists('.tm-panel, .pl-panel, .aip-panel, .al-panel, .marks-panel'))) {
    await key('esc')
    n += 1
  }
  if (n > 0) results[`recover.${label}`] = n
}

describe('14-9 bench', () => {
  let segments = []

  afterEach(() => {
    writeFileSync(OUT, `${JSON.stringify(results, null, 2)}\n`)
  })

  it('0: opens the Work, calibrates real input', async function () {
    this.timeout(480_000)
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), { timeout: 30_000, interval: 250 })
    segments = await seedBilingual()
    // Physical pixels at dpr 2: stays on the 1536×960 display.
    await browser.setWindowSize(2600, 1800)
    await pause(500)
    await installProbe()
    await calibrate()
    await waitForSingleClicks()
    note('segments', segments.length)
  })

  it('5: Ẩn then type lands the character at the caret offset', async function () {
    this.timeout(480_000)
    const id = segments[0].id
    const cell = `[data-col="tgt"][data-segment-id="${String(id)}"]`
    const caret = () =>
      browser.execute((sel) => {
        const el = document.querySelector(sel)
        const s = window.getSelection()
        if (el === null || s === null || s.rangeCount === 0 || !el.contains(s.anchorNode)) return null
        const r = document.createRange()
        r.selectNodeContents(el)
        r.setEnd(s.anchorNode, s.anchorOffset)
        return r.toString().length
      }, cell)
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
      }, cell, n)

    await pressSel('cell: focus translation cell', cell)
    await setCaret(4)
    const before = await browser.execute((sel) => document.querySelector(sel).textContent, cell)
    const injected = await browser.execute(async (segmentId, text) => {
      const urls = performance.getEntriesByType('resource').map((e) => e.name)
      const found = urls.find((n) => /\/src\/proofreadState\.ts(\?|$)/.test(n)) ?? null
      const mod = await import(found ?? '/src/proofreadState.ts')
      mod.proofreadResults.value = new Map([[segmentId, {
        scannedText: text,
        findings: [{ kind: 'spelling', start: 0, end: 3, explanation: 'bench', suggestion: 'Bản' }],
        unlocated: 0,
        ambiguous: 0,
      }]])
      return found
    }, id, before)
    note('proofreadStateUrl', injected)
    await waitFor(async () => exists('[data-proofread-strip-hide]'), 'the Proofreader strip with Ẩn did not render after injecting a result')
    await pause(400)
    await setCaret(4)
    await pause(300)
    const pre = {
      text: await browser.execute((sel) => document.querySelector(sel).textContent, cell),
      caret: await caret(),
      active: await describeActive(),
      hideDisabled: await browser.execute(() => document.querySelector('[data-proofread-strip-hide]').disabled),
      hideTitle: await browser.execute(() => document.querySelector('[data-proofread-strip-hide]').title),
    }
    const rec = await pressSel('Ẩn', '[data-proofread-strip-hide]')
    await pause(300)
    const after = {
      stripGone: !(await exists('.proofreader-strip')),
      caret: await caret(),
      active: await describeActive(),
    }
    await typeText('q')
    await pause(500)
    const typed = await browser.execute((sel) => document.querySelector(sel).textContent, cell)
    const expected = `${pre.text.slice(0, 4)}q${pre.text.slice(4)}`
    const neighbour = await browser.execute((nid) => document.querySelector(`[data-col="tgt"][data-segment-id="${nid}"]`)?.textContent ?? null, segments[1].id)
    note('hide', { pre, rec: { down: rec.down, up: rec.up, click: rec.click }, after, typed, expected, ok: typed === expected, neighbour })
  })

  it('1-3: TM Sửa and Xoá, row aimed by the pressed row, clipped row', async function () {
    this.timeout(480_000)
    await browser.keys(['Meta', 'Alt', 'b'])
    await waitFor(async () => exists('.tm-panel .tm-row'), 'the TM manager did not open on Mod+Alt+B', 15_000)
    await pause(500)
    const rows = () =>
      browser.execute(() =>
        [...document.querySelectorAll('.tm-row')].map((r) => ({
          target: r.querySelector('.tm-target')?.textContent ?? '',
          current: r.classList.contains('tm-row-current'),
          pending: r.classList.contains('tm-row-delete-pending'),
        })),
      )
    const editValue = () => browser.execute(() => document.querySelector('.tm-edit-form .tm-input')?.value ?? null)
    const start = await rows()
    note('tm.start', { rows: start.length, current: start.findIndex((r) => r.current) })
    const A = 0
    const B = 3

    const aimA = async () => {
      await browser.execute(() => { document.querySelector('.tm-panel').scrollTop = 0 })
      await pause(200)
      return pressSel('TM: aim row A', '.tm-row .tm-target', A)
    }
    const btnPoint = (i, which) =>
      browser.execute((idx, w) => {
        const b = document.querySelectorAll('.tm-row')[idx].querySelectorAll('button')[w]
        const r = b.getBoundingClientRect()
        return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
      }, i, which)

    // 2a: cursor on A (pressed), then Sửa on B.
    let cursorBefore = null
    const suaRec = await pressAt('TM: Sửa on row B (cursor on A)', () => btnPoint(B, 0), {
      prepare: async () => {
        if (await exists('.tm-edit-form')) await key('esc')
        await aimA()
        cursorBefore = (await rows()).findIndex((r) => r.current)
      },
    })
    await pause(300)
    const edit = await editValue()
    const cursorAfter = (await rows()).findIndex((r) => r.current)
    note('tm.edit', { cursorBefore, cursorAfter, editValue: edit, expected: start[B].target, ok: edit === start[B].target, attempts: suaRec.attempts })
    await key('esc')
    note('tm.editClosedByEsc', !(await exists('.tm-edit-form')))

    // 2b: cursor on A, Xoá on row D twice.
    const D = 2
    await browser.execute(() => { document.querySelector('.tm-panel').scrollTop = 0 })
    await pause(300)
    const before = await rows()
    let currentBeforeFirst = null
    const del1 = await pressAt('TM: Xoá row D, first press', () => btnPoint(D, 1), {
      prepare: async () => {
        await aimA()
        currentBeforeFirst = (await rows()).findIndex((r) => r.current)
      },
    })
    await pause(300)
    const afterFirst = await rows()
    const del2 = await pressAt('TM: Xoá row D, second press', () => btnPoint(D, 1))
    await pause(600)
    const afterSecond = await rows()
    const targets = afterSecond.map((r) => r.target)
    const activeAfterDelete = await describeActive()
    const cursorBeforeKey = afterSecond.findIndex((r) => r.current)
    await key('arrow-down')
    await pause(300)
    const cursorAfterKey = (await rows()).findIndex((r) => r.current)
    note('tm.delete', {
      activeAfterDelete,
      cursorBeforeKey,
      cursorAfterKey,
      count: [before.length, afterFirst.length, afterSecond.length],
      currentBeforeFirst,
      pendingAfterFirst: afterFirst.findIndex((r) => r.pending),
      currentAfterFirst: afterFirst.findIndex((r) => r.current),
      deletedTarget: before[D].target,
      deletedGone: !targets.includes(before[D].target),
      aSurvives: targets.includes(before[A].target),
      attempts: { first: del1.attempts, second: del2.attempts },
    })

    // 3: clipped row, button half cut at the bottom edge of the scrolling panel.
    const K = 9
    let geometry = null
    let targetsBefore = null
    let scrollBefore = null
    const clipRec = await pressAt('TM: Sửa on clipped row', async () => geometry.p, {
      prepare: async () => {
        if (await exists('.tm-edit-form')) await key('esc')
        geometry = await browser.execute((idx) => {
          const panel = document.querySelector('.tm-panel')
          const btn = document.querySelectorAll('.tm-row')[idx].querySelectorAll('button')[0]
          const cb = panel.getBoundingClientRect().bottom
          const br0 = btn.getBoundingClientRect()
          panel.scrollTop += br0.top + br0.height / 2 - (cb - 4)
          window.__bench.watchSel = '.tm-panel'
          const br = btn.getBoundingClientRect()
          const visibleBottom = Math.min(br.bottom, cb)
          const p = { x: br.x + br.width / 2, y: (br.top + visibleBottom) / 2 }
          const hit = document.elementFromPoint(p.x, p.y)
          return {
            p,
            scrollable: panel.scrollHeight > panel.clientHeight,
            scrollTop: panel.scrollTop,
            panelBottom: cb,
            btnTop: br.top,
            btnBottom: br.bottom,
            visiblePx: visibleBottom - br.top,
            cutPx: br.bottom - cb,
            hitIsButton: hit === btn || btn.contains(hit),
            windowInnerHeight: window.innerHeight,
          }
        }, K)
        await pause(300)
        targetsBefore = await rows()
        scrollBefore = await browser.execute(() => document.querySelector('.tm-panel').scrollTop)
      },
    })
    await pause(300)
    const scrollAfter = await browser.execute(() => document.querySelector('.tm-panel').scrollTop)
    const editClipped = await editValue()
    note('tm.clipped', {
      geometry,
      scrollBefore,
      scrollAfter,
      scrollAtMousedown: clipRec.scrollAt.mousedown,
      scrollAtMouseup: clipRec.scrollAt.mouseup,
      scrollAtClick: clipRec.scrollAt.click,
      attempts: clipRec.attempts,
      editValue: editClipped,
      expected: targetsBefore[K].target,
      editOk: editClipped === targetsBefore[K].target,
    })
    await browser.execute(() => { window.__bench.watchSel = null })
    for (let n = 0; n < 4 && (await exists('.tm-panel')); n += 1) await key('esc')
    note('tm.closed', !(await exists('.tm-panel')))
  })

  it('4: Prompt library, Nhập từ file and Đóng after typing', async function () {
    this.timeout(480_000)
    await recover('4_Pro')
    const name = `bench-149 prompt ${String(Date.now() % 1000)}`
    const body = 'Dong mot {{glossary_terms}}\nDong hai'
    await invoke('prompt_set_create', { tier: 'global', name, body })
    await realClick(await $('[data-ai-prompt-inspector-open]'))
    const opener = await $('[data-prompt-library-open]')
    await opener.waitForDisplayed({ timeout: 15_000 })
    await realClick(opener)
    await waitFor(async () => exists('.pl-panel'), 'the Prompt library did not open', 15_000)
    await pause(500)

    const baseWindows = appWindows()
    const baseSheets = sheetCount()
    const imp = await pressSel('Nhập từ file', '[data-prompt-import-open]')
    await pause(900)
    const dialog = { windows: appWindows(), sheets: sheetCount() }
    const activeDuringDialog = await describeActive()
    // Dismiss the native picker: Esc goes to the frontmost process, which is the app that owns it.
    let dismissed = false
    for (let attempt = 0; attempt < 4 && !dismissed; attempt += 1) {
      execFileSync('osascript', ['-e', 'tell application "System Events" to key code 53'])
      await pause(800)
      dismissed = appWindows() === baseWindows && sheetCount() === baseSheets
    }
    note('prompt.import', {
      events: { down: imp.down, up: imp.up, click: imp.click },
      base: { windows: baseWindows, sheets: baseSheets },
      dialog,
      activeDuringDialog,
      dismissed,
      after: { windows: appWindows(), sheets: sheetCount() },
      overlayStillOpen: await exists('.pl-panel'),
      importPreviewOpen: await exists('[data-prompt-import-overlay], .pi-panel'),
    })

    const rowButton = await $('.pl-panel').$(`button*=${name}`)
    await realClick(rowButton)
    await (await $('.pl-ta-input')).waitForDisplayed({ timeout: 15_000 })
    await pause(300)
    await pressSel('prompt textarea: focus', '.pl-ta-input')
    await typeText('QXZ')
    const typedValue = await browser.execute(() => document.querySelector('.pl-ta-input').value)
    const closeRec = await pressSel('Đóng Prompt library right after typing', '.pl-close')
    await pause(900)
    const closed = !(await exists('.pl-panel'))
    const stored = (await invoke('prompt_set_list', {})).sets.find((s) => s.name === name)
    note('prompt.close', {
      typedValue,
      events: { down: closeRec.down, up: closeRec.up, click: closeRec.click },
      overlayClosed: closed,
      storedBody: stored?.body ?? null,
      storedHasTyped: (stored?.body ?? '').includes('QXZ'),
    })

    const opener2 = await $('[data-prompt-library-open]')
    await opener2.waitForDisplayed({ timeout: 15_000 })
    await realClick(opener2)
    await waitFor(async () => exists('.pl-panel'), 'the Prompt library did not reopen', 15_000)
    await realClick(await $('.pl-panel').$(`button*=${name}`))
    await (await $('.pl-ta-input')).waitForDisplayed({ timeout: 15_000 })
    await pause(300)
    const reopened = await browser.execute(() => document.querySelector('.pl-ta-input').value)
    const close2 = await pressSel('Đóng Prompt library (no edit)', '.pl-close')
    await pause(700)
    note('prompt.reopen', {
      reopenedValue: reopened,
      showsSaved: reopened.includes('QXZ'),
      secondClose: { down: close2.down, up: close2.up, click: close2.click },
      closedAgain: !(await exists('.pl-panel')),
    })
    await key('esc')
    note('prompt.inspectorClosed', !(await exists('[data-prompt-library-open]')))
  })

  it('1+6: alignment Mục trước, focus after the last step back, arrow key', async function () {
    this.timeout(480_000)
    await recover('1+6_a')
    const chapters = await invoke('list_chapters')
    const chapterId = chapters[0].chapter_id
    const root = process.env.AURATRANSLATE_E2E_LIBRARY_ROOT
    if (root === undefined || !existsSync(root)) throw new Error('AURATRANSLATE_E2E_LIBRARY_ROOT is not set: refusing to write a fixture')
    const works = readdirSync(root).filter((n) => n.endsWith('.atproj') && existsSync(join(root, n, 'project.db')))
    works.sort((a, b) => statSync(join(root, b, 'project.db')).mtimeMs - statSync(join(root, a, 'project.db')).mtimeMs)
    const db = join(root, works[0], 'project.db')
    const sql = [
      `INSERT INTO review_chapter (chapter_id, file_name, file_kind, imported_at, stale_at, aligned_at) VALUES (${String(chapterId)}, 'bench.md', 'md', '2026-10-31T00:00:00Z', NULL, '2026-10-31T00:00:00Z');`,
      ...[1, 2, 3, 4].map((i) => `INSERT INTO review_row (review_chapter_id, ord, kind, source_text, target_text) VALUES ((SELECT id FROM review_chapter WHERE chapter_id = ${String(chapterId)}), ${String(i)}, 'text', NULL, 'Reviewer ${String(i)}');`),
    ].join('\n')
    execFileSync('sqlite3', [db, sql])
    note('alignment.fixture', { db: works[0], via: 'sqlite3 INSERT (the reviewer import goes through a native file dialog)' })

    const trigger = await $('.tools-trigger')
    await realClick(trigger)
    const item = await $('[data-tools-item="export.alignment.open"]')
    await item.waitForDisplayed({ timeout: 10_000 })
    await realClick(item)
    await $('.al-panel').waitForDisplayed({ timeout: 10_000 })
    await waitFor(async () => (await browser.execute(() => document.querySelectorAll('.al-panel .al-row').length)) > 2, 'alignment rows did not load')
    await pause(500)
    const cursor = () => browser.execute(() => [...document.querySelectorAll('.al-panel .al-row')].findIndex((r) => r.classList.contains('al-row-current')))
    const labels = () => browser.execute(() => [...document.querySelectorAll('.al-actions .al-act')].map((b) => (b.textContent ?? '').trim()))
    const idx = async (text) => (await labels()).findIndex((l) => l.startsWith(text))
    note('alignment.start', { cursor: await cursor(), rows: await browser.execute(() => document.querySelectorAll('.al-panel .al-row').length), labels: await labels() })

    const actPoint = (i) =>
      browser.execute((n) => {
        const r = document.querySelectorAll('.al-actions .al-act')[n].getBoundingClientRect()
        return { x: r.x + r.width / 2, y: r.y + r.height / 2 }
      }, i)
    await pressAt('alignment: Xem tiếp (1)', async () => actPoint(await idx('Xem')))
    await pressAt('alignment: Xem tiếp (2)', async () => actPoint(await idx('Xem')))
    const atTwo = await cursor()
    const prev1 = await pressAt('alignment: Mục trước (2 to 1)', async () => actPoint(await idx('Mục trước')))
    const atOne = await cursor()
    const prevStill = await idx('Mục trước')
    const prev2 = await pressAt('alignment: Mục trước (1 to 0)', async () => actPoint(await idx('Mục trước')))
    await pause(300)
    const atZero = await cursor()
    const activeAfter = await describeActive()
    const prevGone = (await idx('Mục trước')) === -1
    await key('arrow-down')
    const afterArrow = await cursor()
    note('alignment.prev', {
      cursors: { afterTwoNext: atTwo, afterPrev1: atOne, afterPrev2: atZero, afterArrowDown: afterArrow },
      prevButtonStillThereAtOne: prevStill !== -1,
      prevButtonGoneAtZero: prevGone,
      activeElementAfterLastStep: activeAfter,
      events: { prev1: { down: prev1.down, up: prev1.up, click: prev1.click }, prev2: { down: prev2.down, up: prev2.up, click: prev2.click } },
      keyboardWorks: afterArrow === 1,
    })
    await key('esc')
    await waitFor(async () => !(await exists('.al-panel')), 'the alignment overlay stayed open after Esc')
  })

  it('1+6: Library Bỏ chọn', async function () {
    this.timeout(480_000)
    await recover('1+6_L')
    // Four Chapters, so a selection and the Work page exist.
    for (let split = 0; split < 3; split += 1) {
      const chapters = await invoke('list_chapters')
      await invoke('open_chapter', { chapterId: chapters[chapters.length - 1].chapter_id })
      const loaded = await invoke('read_open_chapter_segments', { prefill: false })
      await invoke('split_chapter_at_segment', { segmentId: loaded.segments[1].id })
    }
    await browser.keys(['Meta', '1'])
    await reloadWorks()
    await aimWork(WORK)
    await openAimedWork()
    await $('[data-library-work-page]').waitForExist({ timeout: 20_000 })
    await waitFor(async () => (await browser.execute(() => document.querySelectorAll('[data-library-chapter-row]').length)) >= 3, 'the Work page listed no Chapters', 20_000)
    await pause(600)
    const selected = () => browser.execute(() => [...document.querySelectorAll('[data-library-chapter-row]')].map((r) => r.getAttribute('aria-selected') === 'true'))
    await pressSel('library: select Chapter row', '[data-library-chapter-row]', 1)
    await waitFor(async () => exists('[data-library-chapter-clear-selection]'), 'Bỏ chọn did not appear after selecting a Chapter')
    await pause(500)
    const sel1 = await selected()
    const rec = await pressSel('Bỏ chọn', '[data-library-chapter-clear-selection]')
    await pause(400)
    const sel2 = await selected()
    const activeAfter = await describeActive()
    const buttonGone = !(await exists('[data-library-chapter-clear-selection]'))
    await key('arrow-down')
    const sel3 = await selected()
    // Control: with focus put back on the list by a real row press, the same key must move the selection.
    await pressSel('library: control, select Chapter row', '[data-library-chapter-row]', 1)
    await pause(400)
    const controlBefore = (await selected()).findIndex(Boolean)
    const controlActive = await describeActive()
    await key('arrow-down')
    const controlAfter = (await selected()).findIndex(Boolean)
    note('library.control', { selectedIndexBefore: controlBefore, selectedIndexAfterArrowDown: controlAfter, activeElement: controlActive })
    note('library.clear', {
      selectedBefore: sel1.filter(Boolean).length,
      selectedAfter: sel2.filter(Boolean).length,
      buttonGone,
      activeElementAfter: activeAfter,
      events: { down: rec.down, up: rec.up, click: rec.click },
      selectedAfterArrowDown: sel3.filter(Boolean).length,
      keyboardWorks: sel3.filter(Boolean).length === 1,
    })
  })

  it('1+6: Reading marks Huỷ', async function () {
    this.timeout(480_000)
    await recover('1+6_R')
    const chapters = await invoke('list_chapters')
    await invoke('open_chapter', { chapterId: chapters[0].chapter_id })
    const loaded = await invoke('read_open_chapter_segments', { prefill: false })
    await invoke('mark_reading_segment', { segmentId: loaded.segments[0].id })
    for (const chapter of await invoke('list_chapters')) await invoke('set_chapter_status', { chapterId: chapter.chapter_id, status: 'done' })
    await browser.keys(['Meta', '3'])
    await waitFor(async () => exists('[data-reading-segment]'), 'Reading mode rendered no segment', 15_000)
    await realClick(await $('[data-reading-toggle-tuner]'))
    await waitFor(async () => exists('[data-reading-tuner-font-size]'), 'the tuner did not open')
    await realClick(await $('[data-reading-marks]'))
    await waitFor(async () => exists('.marks-panel'), 'the marks overlay did not open')
    await pause(500)
    const arm = await pressSel('marks: Bỏ hết đánh dấu (arm)', '[data-reading-marks-clear]')
    await pause(300)
    const armed = await exists('[data-reading-marks-clear-cancel]')
    const huy = await pressSel('Huỷ', '[data-reading-marks-clear-cancel]')
    await pause(400)
    const activeAfterHuy = await describeActive()
    const cancelGone = !(await exists('[data-reading-marks-clear-cancel]'))
    const marksStill = await browser.execute(() => document.querySelectorAll('.marks-list li').length)
    // Keyboard on the same surface: arm again, Esc cancels the pending clear, Esc again closes.
    await pressSel('marks: Bỏ hết đánh dấu (arm again)', '[data-reading-marks-clear]')
    await pause(300)
    const armed2 = await exists('[data-reading-marks-clear-cancel]')
    await key('esc')
    const pendingAfterEsc = await exists('[data-reading-marks-clear-cancel]')
    const stillOpenAfterEsc1 = await exists('.marks-panel')
    await key('esc')
    const closedAfterEsc2 = !(await exists('.marks-panel'))
    note('marks', {
      armed,
      events: { arm: { down: arm.down, up: arm.up, click: arm.click }, huy: { down: huy.down, up: huy.up, click: huy.click } },
      cancelGone,
      marksStillListed: marksStill,
      activeElementAfterHuy: activeAfterHuy,
      keyboard: { armedAgain: armed2, pendingAfterFirstEsc: pendingAfterEsc, overlayOpenAfterFirstEsc: stillOpenAfterEsc1, closedAfterSecondEsc: closedAfterEsc2 },
    })
    await browser.saveScreenshot(join(HERE, 'man-hinh.png'))
  })
})
