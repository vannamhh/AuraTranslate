// Real-mouse bench: cliclick and System Events post trusted input into the app window while the
// wdio session only reads state back. cliclick `kp:esc` never reaches the page, so Escape goes
// through osascript. Run one `it` at a time: a stale Vite HMR stamp splits the fixture's modules.
import { execFileSync } from 'node:child_process'
import { mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { waitForGridRows } from '../../../e2e/support/gridWait.mjs'
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const results = {}

const cli = (...args) => execFileSync('cliclick', ['-e', '40', ...args], { encoding: 'utf8' })
const pause = (ms) => browser.pause(ms)

let offset = null

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
  cli(`c:${guess.x},${guess.y}`)
  await pause(500)
  const pt = await browser.execute(() => window.__bench.down)
  results.calibration = { win, guess, pt }
  if (pt === null) throw new Error('no press reached the page: the window is not in front')
  offset = { x: guess.x - pt[0], y: guess.y - pt[1] }
}

async function screenPoint(selector) {
  const p = await browser.execute((sel) => {
    const r = document.querySelector(sel).getBoundingClientRect()
    return [r.x + r.width / 2, r.y + r.height / 2]
  }, selector)
  return `${String(Math.round(p[0] + offset.x))},${String(Math.round(p[1] + offset.y))}`
}

async function snapshot() {
  return await browser.execute(() => ({
    log: window.__bench.log.splice(0),
    caret: document.querySelector('.cell-tgt.editing')?.getAttribute('data-segment-id') ?? null,
    notice: document.querySelector('[data-review-notice]') !== null,
    history: document.querySelector('.hist-panel') !== null,
    selection: String(window.getSelection()),
    tgtText: [...document.querySelectorAll('[data-col="tgt"]')].map((el) => el.textContent),
    activeElement: `${document.activeElement?.tagName}:${document.activeElement?.getAttribute('data-segment-id') ?? ''}`,
    selected: [...document.querySelectorAll('[data-num-segment-id].row-selected')].map((e) => e.getAttribute('data-num-segment-id')),
    status: document.querySelector('[data-status-live]')?.textContent?.trim() ?? null,
  }))
}

async function answerReviewReads(chapter, segmentIds) {
  await browser.execute((chapterId, idsOnDisk) => {
    const rows = idsOnDisk.map((_, i) => ({ id: 100 + i, kind: 'text', source_text: `n${i}`, target_text: `s${i}` }))
    const segments = idsOnDisk.map((id, i) => ({ id, ord: i + 1, role: null, source_text: `n${i}`, target_text: `d${i}` }))
    const pairs = idsOnDisk.map((id, i) => ({
      group_id: i + 1,
      decided_by: 'machine',
      segment_ids: [id],
      row_ids: [100 + i],
      spans: [{ kind: 'delete', text: `d${i}` }, { kind: 'insert', text: `s${i}` }],
      decision: null,
    }))
    const replies = {
      alignment_open: { chapter_id: chapterId, file_name: 'r.md', file_kind: 'md', rows, segments, groups: [], unmatched_row_ids: [], unmatched_segment_ids: [], is_resolved: true },
      review_diff: { chapter_id: chapterId, pairs },
    }
    const realFetch = window.fetch
    window.fetch = (url, init) => {
      const reply = replies[String(url).split('/').pop()]
      if (reply === undefined) return realFetch(url, init)
      return Promise.resolve(new Response(JSON.stringify(reply), { headers: { 'Tauri-Response': 'ok', 'Content-Type': 'application/json' } }))
    }
  }, chapter, segmentIds.map(Number))
}

async function barGeometry() {
  return await browser.execute(() => {
    const box = (sel) => {
      const r = document.querySelector(sel).getBoundingClientRect()
      return { top: r.top, bottom: r.bottom, height: r.height }
    }
    return { bar: box('.review-bar'), stats: box('[data-review-stats]'), next: box('[data-review-diff-next]') }
  })
}

const SEL = {
  tgt: (id) => `[data-col="tgt"][data-segment-id="${id}"]`,
  num: (id) => `[data-num-segment-id="${id}"]`,
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
const overlay = () => browser.execute(() => document.querySelector('.hist-panel') !== null)

function activate() {
  const pid = execFileSync('pgrep', ['-n', 'auratranslate'], { encoding: 'utf8' }).trim()
  execFileSync('osascript', ['-e', `tell application "System Events" to set frontmost of (first process whose unix id is ${pid}) to true`])
}

async function press(kind, selector) {
  for (let attempt = 0; attempt < 4; attempt += 1) {
    await browser.execute(() => { window.__bench.down = null })
    cli(`${kind}:${await screenPoint(selector)}`)
    await pause(350)
    if ((await browser.execute(() => window.__bench.down)) !== null) return
    activate()
    await pause(300)
  }
  throw new Error(`press never reached the page: ${selector}`)
}

async function fresh(name) {
  await openWorkspaceWithWork(name, '一二三。四五六。七八九。')
  await waitForGridRows(3, { what: 'bench' })
  await installProbe()
  for (let attempt = 0; attempt < 5 && offset === null; attempt += 1) {
    activate()
    await pause(1200)
    await calibrate().catch(() => undefined)
  }
  return await ids()
}

async function typeInto(id, text) {
  await press('c', SEL.tgt(id))
  cli(`t:${text}`)
  await pause(300)
  await snapshot()
}

async function viaMenu(id) {
  await press('rc', SEL.tgt(id))
  await browser.execute(() => {
    const b = [...document.querySelectorAll('[data-sentence-menu] button')].find((x) => x.textContent.includes('Lịch sử phiên bản'))
    b.setAttribute('data-bench-history', '')
  })
  await snapshot()
  await press('c', '[data-bench-history]')
  await pause(400)
}

const preset = (n) => `.layout-switch .layout-switch-btn:nth-of-type(${n})`
const CELL = preset(3)
const count = (log, type) => log.filter((l) => l.startsWith(type)).length
const dock = () => browser.execute(() => document.querySelector('[data-review-dock]') !== null)

describe('real input', () => {
  after(() => writeFileSync(join(HERE, 'ket-qua.json'), JSON.stringify(results, null, 2)))

  it('history from the menu: a key under the overlay, Escape, typing', async () => {
    const all = await fresh('Bench menu')
    await typeInto(all[1], 'abc')
    await viaMenu(all[1])
    results.menu_open = { ...(await snapshot()), sel: await selectionHost(), overlay: await overlay() }
    cli('t:Q')
    await pause(400)
    results.menu_key = { ...(await snapshot()), sel: await selectionHost(), overlay: await overlay() }
    esc()
    await pause(700)
    results.menu_esc = { ...(await snapshot()), sel: await selectionHost(), overlay: await overlay() }
    cli('t:zz')
    await pause(400)
    results.menu_typed = { ...(await snapshot()), sel: await selectionHost(), overlay: await overlay() }
  })

  it('Review Mode: open by the cell, a key, diff buttons, close by the cell, typing', async () => {
    const all = await fresh('Bench review')
    const chapterId = (await browser.execute(async () => window.__TAURI_INTERNALS__.invoke('read_open_chapter_segments', {}))).chapter_id
    await answerReviewReads(chapterId, all)
    await typeInto(all[1], 'abc')
    await press('c', CELL)
    await pause(500)
    results.review_open = { ...(await snapshot()), dock: await dock(), bar: await barGeometry() }
    cli('t:Q')
    await pause(400)
    results.review_key = { ...(await snapshot()), dock: await dock() }
    await press('c', '[data-review-diff-next]')
    await press('c', '[data-review-diff-next]')
    results.review_diff = await browser.execute(() => ({
      status: document.querySelector('[data-review-diff-status]').textContent.trim(),
      active: `${document.activeElement?.tagName}.${document.activeElement?.className}`,
    }))
    await snapshot()
    await press('c', CELL)
    await pause(800)
    results.review_closed = { ...(await snapshot()), dock: await dock(), sel: await selectionHost() }
    cli('t:xy')
    await pause(400)
    results.review_typed = { ...(await snapshot()), sel: await selectionHost() }
  })

  it('control: preset cells, no Review Mode involved', async () => {
    const all = await fresh('Bench control')
    await typeInto(all[1], 'abc')
    results.rounds = []
    for (let i = 0; i < 16; i += 1) {
      const same = i < 8
      await press('c', same ? preset(2) : preset(1 + (i % 2)))
      await pause(700)
      const { log } = await snapshot()
      results.rounds.push({ same, down: count(log, 'mousedown'), up: count(log, 'mouseup'), click: count(log, 'click') })
    }
  })
})
