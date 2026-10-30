// Real-mouse bench for Story 14.2: cliclick posts trusted events into the app window while the
// wdio session only reads state back. Driver pointer events are untrusted and have no default
// action, so they cannot show WebKit selecting a word on right click.
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
// cliclick `kp:esc` reaches the page as no keydown at all, so Esc goes through System Events.
const esc = () => execFileSync('osascript', ['-e', 'tell application "System Events" to key code 53'], { encoding: 'utf8' })

let offset = null

async function installProbe() {
  await browser.execute(() => {
    if (window.__bench !== undefined) return
    const bench = { log: [], down: null }
    window.__bench = bench
    const name = (t) => {
      if (!(t instanceof Element)) return String(t)
      const col = t.closest('[data-col]')?.getAttribute('data-col')
      const num = t.closest('[data-num-segment-id]')?.getAttribute('data-num-segment-id')
      const id = t.closest('[data-segment-id]')?.getAttribute('data-segment-id')
      if (t.closest('[data-sentence-menu]') !== null) return 'menu'
      return col !== undefined && col !== null ? `${col}:${id}` : num ? `num:${num}` : t.tagName
    }
    window.addEventListener('mousedown', (e) => {
      bench.down = [e.clientX, e.clientY]
    }, true)
    for (const type of ['mousedown', 'contextmenu', 'mouseup', 'click', 'focusin', 'focusout', 'keydown']) {
      window.addEventListener(type, (e) => {
        const extra = type === 'mousedown' || type === 'mouseup' ? ` b${String(e.button)}` : ''
        bench.log.push(`${type}${extra} ${name(e.target)} trusted=${String(e.isTrusted)}`)
      }, true)
    }
    document.addEventListener('selectionchange', () => {
      const s = window.getSelection()
      bench.log.push(`selectionchange ${s?.type} "${s?.toString() ?? ''}"`)
    })
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
    const el = document.querySelector(sel)
    const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT)
    let node = walker.nextNode()
    while (node !== null && node.data.trim() === '') node = walker.nextNode()
    if (node === null) {
      const r = el.getBoundingClientRect()
      return [r.x + r.width / 2, r.y + r.height / 2]
    }
    const range = document.createRange()
    range.selectNodeContents(node)
    const r = range.getClientRects()[0]
    return [r.x + r.width / 2, r.y + r.height / 2]
  }, selector)
  return `${String(Math.round(p[0] + offset.x))},${String(Math.round(p[1] + offset.y))}`
}

async function snapshot() {
  return await browser.execute(() => {
    const s = window.getSelection()
    const anchor = s?.anchorNode ?? null
    const from = anchor instanceof Element ? anchor : (anchor?.parentElement ?? null)
    const log = window.__bench.log.splice(0)
    return {
      log,
      stateCaret: document.querySelector('.cell-tgt.editing')?.getAttribute('data-segment-id') ?? null,
      activeElement: `${document.activeElement?.tagName}:${document.activeElement?.getAttribute('data-segment-id') ?? ''}`,
      selectionType: s?.type,
      selectionText: s?.toString() ?? '',
      selectionCol: from?.closest('[data-col]')?.getAttribute('data-col') ?? null,
      selectionSegment: from?.closest('[data-segment-id]')?.getAttribute('data-segment-id') ?? null,
      menu: document.querySelector('[data-sentence-menu]')?.getBoundingClientRect().toJSON() ?? null,
      targets: [...document.querySelectorAll('[data-col="tgt"]')].map((c) => c.textContent),
      omitted: [...document.querySelectorAll('[data-col="src"]')].map((c) => c.classList.contains('omitted')),
    }
  })
}

const SEL = {
  tgt: (id) => `[data-col="tgt"][data-segment-id="${id}"]`,
  src: (id) => `[data-col="src"][data-segment-id="${id}"]`,
  num: (id) => `[data-num-segment-id="${id}"]`,
}

async function ids() {
  return await browser.execute(() => [...document.querySelectorAll('[data-col="src"]')].map((el) => el.getAttribute('data-segment-id')))
}

async function typeInto(id, text) {
  cli(`c:${await screenPoint(SEL.tgt(id))}`)
  await pause(300)
  cli(`t:${text}`)
  await pause(400)
}

async function itemPoint(label) {
  const p = await browser.execute((text) => {
    const b = [...document.querySelectorAll('[data-sentence-menu] button')].find((x) => x.textContent.includes(text))
    if (b === undefined) return null
    const r = b.getBoundingClientRect()
    return { x: r.x + r.width / 2, y: r.y + r.height / 2, disabled: b.disabled, title: b.title }
  }, label)
  if (p === null) throw new Error(`no item ${label}`)
  return { ...p, at: `${String(Math.round(p.x + offset.x))},${String(Math.round(p.y + offset.y))}` }
}

describe('14.2 real mouse', () => {
  after(() => {
    mkdirSync(HERE, { recursive: true })
    writeFileSync(join(HERE, 'ket-qua.json'), JSON.stringify(results, null, 2))
  })

  it('collects the matrix', async () => {
    await openWorkspaceWithWork('Bench 14.2', '一二三四五六。七八九十甲乙。丙丁戊己庚辛。壬癸子丑寅卯。辰巳午未申酉。')
    await waitForGridRows(5, { what: 'bench' })
    await installProbe()
    try {
      execFileSync('osascript', [
        '-e', 'tell application "System Events"',
        '-e', 'set p to first process whose name contains "auratranslate"',
        '-e', 'set frontmost of p to true',
        '-e', 'perform action "AXRaise" of window 1 of p',
        '-e', 'end tell',
      ])
    } catch (error) {
      results.activate = String(error)
    }
    await pause(800)
    await calibrate()
    const row = await ids()
    results.rows = row

    await typeInto(row[0], 'alpha beta gamma')
    await typeInto(row[1], 'delta epsilon zeta')
    await typeInto(row[2], 'eta theta iota')
    results.typed = await snapshot()

    for (const column of ['tgt', 'src', 'num']) {
      cli(`c:${await screenPoint(SEL.tgt(row[0]))}`)
      await pause(350)
      await browser.execute(() => window.__bench.log.splice(0))
      const before = await snapshot()

      cli(`rc:${await screenPoint(SEL[column](row[1]))}`)
      await pause(500)
      const open = await snapshot()

      esc()
      await pause(400)
      const closed = await snapshot()

      cli('t:Q')
      await pause(400)
      const typedAfter = await snapshot()
      results[`dismiss_${column}`] = { before, open, closed, typedAfter }
    }

    cli(`c:${await screenPoint(SEL.tgt(row[0]))}`)
    await pause(350)
    cli(`rc:${await screenPoint(SEL.num(row[1]))}`)
    await pause(450)
    const omit = await itemPoint('Cắt bỏ câu')
    await browser.execute(() => window.__bench.log.splice(0))
    cli(`c:${omit.at}`)
    await pause(700)
    results.choose_omit_from_num = await snapshot()

    cli('t:W')
    await pause(400)
    results.type_after_choose = await snapshot()

    cli(`dc:${await screenPoint(SEL.src(row[2]))}`)
    await pause(400)
    const selected = await snapshot()
    cli(`rc:${await screenPoint(SEL.src(row[2]))}`)
    await pause(450)
    const insideSelection = await snapshot()
    esc()
    await pause(400)
    results.right_click_inside_selection = { selected, insideSelection, closed: await snapshot() }

    cli(`c:${await screenPoint(SEL.tgt(row[3]))}`)
    await pause(350)
    cli('kd:cmd', `c:${await screenPoint(SEL.src(row[3]))}`, 'ku:cmd')
    await pause(400)
    cli(`rc:${await screenPoint(SEL.src(row[4]))}`)
    await pause(450)
    results.split_cut_elsewhere = { item: await itemPoint('Tách câu'), state: await snapshot() }
    esc()
    await pause(300)
    results.cut_after_esc = await browser.execute(() => document.querySelectorAll('.cut-mark').length)

    cli(`c:${await screenPoint(SEL.tgt(row[0]))}`)
    await pause(350)
    cli(`rc:${await screenPoint(SEL.src(row[2]))}`)
    await pause(450)
    const confirm = await itemPoint('Xác nhận')
    await browser.execute(() => window.__bench.log.splice(0))
    cli(`c:${confirm.at}`)
    await pause(1200)
    cli(`c:${await screenPoint(SEL.tgt(row[0]))}`)
    await pause(350)
    cli(`rc:${await screenPoint(SEL.src(row[3]))}`)
    await pause(450)
    const outside = await browser.execute(() => {
      const r = document.querySelector('[data-status-confirmed]').getBoundingClientRect()
      return [r.x + r.width / 2, r.y + r.height / 2]
    })
    await browser.execute(() => window.__bench.log.splice(0))
    cli(`c:${String(Math.round(outside[0] + offset.x))},${String(Math.round(outside[1] + offset.y))}`)
    await pause(500)
    results.dismiss_by_outside_press = await snapshot()

    results.confirm_from_src = {
      state: await snapshot(),
      statuses: await browser.execute(() => [...document.querySelectorAll('.col-rule .cell, .col-rule > *')].length),
      counter: await browser.execute(() => document.querySelector('[data-status-confirmed]')?.textContent ?? null),
    }
  })
})
