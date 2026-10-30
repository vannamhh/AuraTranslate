// Real-mouse bench: cliclick posts trusted events into the app window while the
// wdio session only reads state back. Driver events are untrusted and carry whatever `buttons`
// the caller wrote, so they cannot show what WKWebView reports during a real drag.
import { execFileSync } from 'node:child_process'
import { mkdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { waitForGridRows } from '../../../e2e/support/gridWait.mjs'
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const LONG_ROWS = 9850
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
      if (t.closest('[data-chapter-next]') !== null) return 'next'
      return col !== undefined && col !== null ? `${col}:${id}` : num ? `num:${num}` : t.tagName
    }
    window.addEventListener('mousedown', (e) => {
      bench.down = [e.clientX, e.clientY]
    }, true)
    for (const type of ['mousedown', 'mouseover', 'mouseup', 'click', 'focusin', 'focusout']) {
      window.addEventListener(type, (e) => {
        bench.log.push(`${type} ${name(e.target)} buttons=${String(e.buttons)} trusted=${String(e.isTrusted)}`)
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
    activeElement: `${document.activeElement?.tagName}:${document.activeElement?.getAttribute('data-segment-id') ?? ''}`,
    selected: [...document.querySelectorAll('[data-num-segment-id].row-selected')].map((e) => e.getAttribute('data-num-segment-id')),
    status: document.querySelector('[data-status-live]')?.textContent?.trim() ?? null,
  }))
}

const SEL = {
  tgt: (id) => `[data-col="tgt"][data-segment-id="${id}"]`,
  num: (id) => `[data-num-segment-id="${id}"]`,
}

async function ids() {
  return await browser.execute(() => [...document.querySelectorAll('[data-col="src"]')].map((el) => el.getAttribute('data-segment-id')))
}

describe('real mouse', () => {
  after(() => {
    mkdirSync(HERE, { recursive: true })
    writeFileSync(join(HERE, 'ket-qua.json'), JSON.stringify(results, null, 2))
  })

  it('drag and the chapter button with a trusted mouse', async () => {
    await openWorkspaceWithWork('Bench 14.3', '一二三四六。七八九十甲乙。丙丁戊己庚辛。壬癸子丑寅卯。辰巳午未申酉。')
    await waitForGridRows(5, { what: 'bench' })
    await installProbe()
    await calibrate()
    const all = await ids()
    cli(`c:${await screenPoint(SEL.tgt(all[0]))}`)
    await pause(300)
    await snapshot()

    const from = await screenPoint(SEL.num(all[1]))
    const mid = await screenPoint(SEL.num(all[2]))
    const to = await screenPoint(SEL.tgt(all[3]))
    cli(`dd:${from}`, `m:${mid}`, `m:${to}`, `du:${to}`)
    await pause(400)
    results.dragReleaseOnTarget = await snapshot()

    cli(`c:${await screenPoint(SEL.tgt(all[0]))}`)
    await pause(300)
    cli('t:x')
    await pause(300)
    await snapshot()
    cli(`c:${await screenPoint('[data-chapter-next]')}`)
    await pause(500)
    results.nextFromTyping = await snapshot()
  })

  it('frame cost of one focus move on a long Chapter', async () => {
    await openWorkspaceWithWork('Bench 14.3 dài', '句子。'.repeat(LONG_ROWS))
    await waitForGridRows(LONG_ROWS, { what: 'long bench' })
    results.focusSweep = await browser.execute(
      (DRAG_MOVES) =>
        new Promise((resolve) => {
          const rows = [...document.querySelectorAll('[data-num-segment-id]')]
          const frame = () => new Promise((done) => requestAnimationFrame(done))
          const opts = { buttons: 1, bubbles: true }
          rows[0].dispatchEvent(new MouseEvent('mousedown', { ...opts, button: 0, cancelable: true }))
          void (async () => {
            const cost = []
            await frame()
            for (let i = 1; i <= DRAG_MOVES; i += 1) {
              const t0 = performance.now()
              rows[i].dispatchEvent(new MouseEvent('mouseover', opts))
              await frame()
              cost.push(performance.now() - t0)
            }
            document.dispatchEvent(new MouseEvent('mouseup', { button: 0, buttons: 0, bubbles: true }))
            cost.sort((a, b) => a - b)
            resolve({
              rows: rows.length,
              moves: cost.length,
              medianMs: cost[Math.floor(cost.length / 2)],
              maxMs: cost[cost.length - 1],
              over50Ms: cost.filter((ms) => ms > 50).length,
            })
          })()
        }),
      60,
    )
  })
})
