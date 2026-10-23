/**
 * Do Vue Story 2.14: Vue dev tracing (config.performance) bao cao render/patch theo component
 * trong mot lan doi con tro. Vue xoa measure ngay sau khi tao nen thu bang PerformanceObserver.
 */
import fs from 'node:fs'
import path from 'node:path'
import { execSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { realClick } from '../../../e2e/support/pointer.mjs'

const SO_CAU = 9850
const SO_LUOT = 6
const DICH_A = 3
const DICH_B = 25
const THU_MUC = path.dirname(fileURLToPath(import.meta.url))
const ketQua = { ua: null, tai_truoc: null, tai_sau: null, vue_app: null, bien_the: [] }

function vanBan(n) {
  const parts = []
  for (let i = 0; i < n; i += 1) parts.push(`第${i + 1}句话，用来量渲染成本。`)
  return parts.join('')
}

function taiMay() {
  try {
    return execSync('uptime').toString().trim()
  } catch (e) {
    return String(e)
  }
}

async function langLai() {
  for (let i = 0; i < 40; i += 1) {
    const nhip = await browser.execute(
      async () =>
        await new Promise((r) => {
          const a = performance.now()
          requestAnimationFrame(() => r(performance.now() - a))
        }),
    )
    if (nhip < 50) return
  }
}

async function nap() {
  await browser.execute(() => {
    window.location.reload()
  })
  await $('[data-col="tgt"]').waitForExist({ timeout: 120_000 })
  await langLai()
  await browser.pause(1500)
  const info = await browser.execute(() => {
    const app = document.querySelector('#app')?.__vue_app__
    return {
      href: location.href,
      so_cau: document.querySelectorAll('[data-col="tgt"]').length,
      co_app: !!app,
    }
  })
  ketQua.vue_app = info.co_app
  if (!info.co_app) throw new Error('KHONG CO __vue_app__')
  if (!info.href.includes('localhost:1420') || info.so_cau < SO_CAU) throw new Error('DANH TINH PHIEN KHONG KHOP')
  await browser.execute(() => {
    const p = { t0: null, tSel: null, tEnd: null }
    window.__p = p
    window.__vm = []
    document.addEventListener('selectionchange', () => {
      if (p.t0 !== null && p.tSel === null) p.tSel = performance.now()
    })
    new PerformanceObserver((list) => {
      for (const e of list.getEntries()) window.__vm.push({ name: e.name, start: e.startTime, dur: e.duration })
    }).observe({ type: 'measure' })
    window.__resetP = () => {
      p.t0 = null
      p.tSel = null
      p.tEnd = null
      window.__vm = []
    }
    window.__rafs = () => {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          p.tEnd = performance.now()
        })
      })
    }
    document.querySelector('#app').__vue_app__.config.performance = true
  })
}

async function chayTrongTrang() {
  return browser.execute(
    async (a, b, n) => {
      const out = []
      const cells = document.querySelectorAll('[data-col="tgt"]')
      for (let i = 0; i < n + 1; i += 1) {
        const cell = cells[i % 2 === 0 ? b : a]
        window.__resetP()
        performance.clearMeasures()
        performance.clearMarks()
        const p = window.__p
        p.t0 = performance.now()
        window.__rafs()
        cell.focus()
        getSelection().setPosition(cell.firstChild ?? cell, 0)
        await new Promise((r) => setTimeout(r, 1200))
        out.push({ ...p, entries: window.__vm.slice() })
        await new Promise((r) => setTimeout(r, 300))
      }
      return out.slice(1)
    },
    DICH_A,
    DICH_B,
    SO_LUOT,
  )
}

async function bamThat() {
  const out = []
  const o = await $$('[data-col="tgt"]')
  for (let i = 0; i < 4; i += 1) {
    await browser.execute(() => {
      window.__resetP()
      performance.clearMeasures()
      performance.clearMarks()
      window.__h = () => {
        window.__p.t0 = performance.now()
        window.__vm = []
        window.__rafs()
      }
      window.addEventListener('mousedown', window.__h, true)
    })
    await realClick(o[i % 2 === 0 ? DICH_B : DICH_A])
    await browser.pause(1500)
    const r = await browser.execute(() => {
      window.removeEventListener('mousedown', window.__h, true)
      return { ...window.__p, entries: window.__vm.slice() }
    })
    if (i > 0) out.push(r)
  }
  return out
}

function med(xs) {
  const v = xs.filter((x) => x !== null && x !== undefined).sort((a, b) => a - b)
  return v.length ? v[Math.floor(v.length / 2)] : null
}

function hop(iv) {
  const s = iv.slice().sort((a, b) => a[0] - b[0])
  let tong = 0
  let cur = null
  for (const [x, y] of s) {
    if (cur === null) cur = [x, y]
    else if (x <= cur[1]) cur[1] = Math.max(cur[1], y)
    else {
      tong += cur[1] - cur[0]
      cur = [x, y]
    }
  }
  if (cur) tong += cur[1] - cur[0]
  return tong
}

function tinh(m) {
  const total = m.tEnd - m.t0
  const es = m.entries
    .filter((e) => e.start >= m.t0 && e.start <= m.tEnd)
    .map((e) => {
      const i = e.name.lastIndexOf('> ')
      return { ...e, comp: e.name.slice(0, i + 1), type: e.name.slice(i + 2), rel: e.start - m.t0 }
    })
  const phan = es.filter((e) => e.type === 'render' || e.type === 'patch')
  const union = hop(phan.map((e) => [e.start, e.start + e.dur]))
  const per = {}
  for (const e of phan) {
    const k = e.comp
    per[k] ??= { render_ms: 0, render_n: 0, patch_ms: 0, patch_n: 0 }
    per[k][e.type + '_ms'] += e.dur
    per[k][e.type + '_n'] += 1
  }
  const tSel = m.tSel === null ? null : m.tSel - m.t0
  const truocSel = phan.filter((e) => tSel !== null && e.rel < tSel).length
  const sauSel = phan.filter((e) => tSel !== null && e.rel >= tSel).length
  const grid = phan.filter((e) => e.comp === '<GridPanel>' && e.type === 'render').map((e) => +e.rel.toFixed(1))
  return { total, union, gap: total - union, tSel, truocSel, sauSel, grid_render_rel: grid, per, n_entries: es.length }
}

async function doBienThe(ten, fn) {
  const marks = await fn()
  const derived = marks.map(tinh)
  const medians = {}
  for (const k of ['total', 'union', 'gap', 'tSel', 'truocSel', 'sauSel']) medians[k] = med(derived.map((d) => d[k]))
  const comps = new Set(derived.flatMap((d) => Object.keys(d.per)))
  const comp_median = {}
  for (const c of comps) {
    const g = (f) => med(derived.map((d) => d.per[c]?.[f] ?? 0))
    comp_median[c] = { render_ms: g('render_ms'), render_n: g('render_n'), patch_ms: g('patch_ms'), patch_n: g('patch_n') }
  }
  const top = Object.entries(comp_median)
    .sort((x, y) => y[1].render_ms + y[1].patch_ms - (x[1].render_ms + x[1].patch_ms))
    .slice(0, 8)
  ketQua.bien_the.push({ ten, n: marks.length, marks, derived, medians, comp_median, top })
  console.log(`[2.14-dovue] ${ten} medians:`, JSON.stringify(medians))
  console.log(`[2.14-dovue] ${ten} top:`, JSON.stringify(top))
  console.log(`[2.14-dovue] ${ten} grid render rel:`, JSON.stringify(derived.map((d) => d.grid_render_rel)))
}

describe('Do Vue 2.14', function () {
  this.timeout(1_700_000)
  before(async function () {
    this.timeout(600_000)
    ketQua.tai_truoc = taiMay()
    console.log('[2.14-dovue] tai truoc:', ketQua.tai_truoc)
    await openWorkspaceWithWork('2.14 — do vue')
    await browser.setTimeout({ script: 300_000 })
    await browser.execute(async (text) => {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: '2.14-dovue-' + Date.now(),
        sourceLang: 'zh',
        genre: 'general',
        text,
      })
    }, vanBan(SO_CAU))
    await nap()
    ketQua.ua = await browser.execute(() => navigator.userAgent)
    console.log('[2.14-dovue] ua:', ketQua.ua)
  })

  it('1 tgt setPosition + focus', async function () {
    this.timeout(900_000)
    await langLai()
    await doBienThe('1-tgt-setPosition', chayTrongTrang)
  })
  it('2 tgt realClick', async function () {
    this.timeout(900_000)
    await langLai()
    await doBienThe('2-tgt-realClick', bamThat)
  })

  after(() => {
    ketQua.tai_sau = taiMay()
    console.log('[2.14-dovue] tai sau:', ketQua.tai_sau)
    const tep = path.join(THU_MUC, `ket-qua-do-vue-${new Date().toISOString().replace(/:/g, '-')}.json`)
    fs.writeFileSync(tep, JSON.stringify(ketQua, null, 2))
    console.log('[2.14-dovue] json:', tep)
  })
})
