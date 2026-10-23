/**
 * Ban sao tach app Story 2.14: khoi ~390 ms co con khi go Vue/app khoi ban sao DOM khong? Bisect CSS tren ban sao.
 * Moi bien the: reload + lang lai -> sao #app -> unmount Vue -> thay ban sao -> (tuy chon) ghi de CSS. Ban debug nap Vite dev.
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
const ketQua = { ua: null, tai_truoc: null, tai_sau: null, bien_the: [] }

const GHI_DE = {
  'a-grid-block': '.grid, .col { display: block !important; grid-template-rows: none !important; grid-template-columns: none !important; }',
  'a2-col-block-only': '.col { display: block !important; grid-template-rows: none !important; }',
  'b-hide-other-cols': '.col-rule, .col-num, .col-src { display: none !important; }',
  'c1-scroll-visible': '.grid-scroll { overflow: visible !important; }',
  'c2-scroll-hidden': '.grid-scroll { overflow: hidden !important; }',
  'd-user-select-tgt-only': '* { -webkit-user-select: none !important; user-select: none !important; } [data-col="tgt"], [data-col="tgt"] * { -webkit-user-select: text !important; user-select: text !important; }',
  'g-tgt-white-space-normal': '.cell-tgt { white-space: normal !important; }',
  'h-no-cell-borders': '.cell { border: 0 !important; }',
  'i-panel-overflow-visible': '.panel, .grid-scroll { overflow: visible !important; }',
}

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
  const info = await browser.execute(() => ({
    href: location.href,
    so_cau: document.querySelectorAll('[data-col="tgt"]').length,
  }))
  if (!info.href.includes('localhost:1420') || info.so_cau < SO_CAU) throw new Error('DANH TINH PHIEN KHONG KHOP')
  await browser.execute(() => {
    const p = { t0: null, t1: null, tSel: null, tR1: null, tR2: null, tLong: null }
    window.__p = p
    document.addEventListener('selectionchange', () => {
      if (p.t0 !== null && p.tSel === null) p.tSel = performance.now()
    })
    window.__resetP = () => {
      for (const k of Object.keys(p)) p[k] = null
    }
    window.__rafs = () => {
      requestAnimationFrame(() => {
        p.tR1 = performance.now()
        requestAnimationFrame(() => {
          p.tR2 = performance.now()
        })
      })
    }
  })
}

function tinh(m) {
  const f = (x) => (x === null || m.tR2 === null ? null : +x.toFixed(2))
  return {
    sync: m.t1 === null ? null : f(m.t1 - m.t0),
    handlers: m.tSel === null ? null : f(m.tSel - m.t0),
    toFrame1: f(m.tR1 - m.t0),
    frame1: f(m.tR2 - m.tR1),
    total: f(m.tR2 - m.t0),
    tLong: m.tLong === null ? null : f(m.tLong - m.t0),
  }
}

async function chayTrongTrang(cot, focus) {
  return browser.execute(
    async (col, doFocus, a, b, n) => {
      const out = []
      const cells = document.querySelectorAll(`[data-col="${col}"]`)
      for (let i = 0; i < n + 1; i += 1) {
        const cell = cells[i % 2 === 0 ? b : a]
        window.__resetP()
        const p = window.__p
        p.t0 = performance.now()
        window.__rafs()
        if (doFocus) cell.focus()
        getSelection().setPosition(cell.firstChild ?? cell, 0)
        p.t1 = performance.now()
        setTimeout(() => {
          p.tLong = performance.now()
        }, 0)
        await new Promise((r) => setTimeout(r, 1200))
        out.push({ ...p })
        await new Promise((r) => setTimeout(r, 300))
      }
      return out.slice(1)
    },
    cot,
    focus,
    DICH_A,
    DICH_B,
    SO_LUOT,
  )
}


async function sao(css, mode) {
  const r = await browser.execute(
    (cssText, kieu) => {
      const app = document.querySelector('#app')
      const gridTruoc = document.querySelector('.grid')
      const truoc = {
        grid_display: getComputedStyle(gridTruoc).display,
        col_display: getComputedStyle(document.querySelector('.col-tgt')).display,
        tgt_cells: document.querySelectorAll('[data-col="tgt"]').length,
        tgt_contenteditable: document.querySelectorAll('[data-col="tgt"][contenteditable]').length,
        all: document.querySelectorAll('*').length,
      }
      const clone = app.cloneNode(true)
      const vueApp = app.__vue_app__
      if (!vueApp) return { loi: 'khong co __vue_app__' }
      vueApp.unmount()
      app.replaceWith(clone)
      if (cssText) {
        const st = document.createElement('style')
        st.id = '__ghi_de'
        st.textContent = cssText
        document.head.appendChild(st)
      }
      if (kieu === 'grid-rieng') {
        const g = clone.querySelector('.grid')
        document.body.replaceChildren(g)
      } else if (kieu === 'grid-scroll-rieng') {
        const g = clone.querySelector('.grid-scroll')
        g.style.height = '100vh'
        document.body.replaceChildren(g)
      }
      void document.body.offsetHeight
      const g2 = document.querySelector('.grid')
      let dataV = 0
      for (const a of g2.attributes) if (a.name.startsWith('data-v-')) dataV += 1
      const tgt0 = document.querySelector('[data-col="tgt"]')
      let dataVCell = 0
      for (const a of tgt0.attributes) if (a.name.startsWith('data-v-')) dataVCell += 1
      window.__mut = 0
      window.__mo = new MutationObserver((l) => {
        window.__mut += l.length
      })
      window.__mo.observe(document.body, { subtree: true, childList: true, attributes: true, characterData: true })
      window.__lateSel = 0
      document.addEventListener('selectionchange', () => {
        window.__lateSel += 1
      })
      return {
        truoc,
        sau: {
          grid_display: getComputedStyle(g2).display,
          col_display: getComputedStyle(document.querySelector('.col-tgt')).display,
          tgt_cells: document.querySelectorAll('[data-col="tgt"]').length,
          tgt_contenteditable: document.querySelectorAll('[data-col="tgt"][contenteditable]').length,
          all: document.querySelectorAll('*').length,
          grid_data_v_attrs: dataV,
          tgt_cell_data_v_attrs: dataVCell,
          vue_app_con: !!document.querySelector('#app')?.__vue_app__,
          style_tags_head: document.head.querySelectorAll('style').length,
          ghi_de_tag: !!document.getElementById('__ghi_de'),
        },
      }
    },
    css,
    mode,
  )
  if (r.loi) throw new Error(r.loi)
  await langLai()
  await browser.pause(800)
  return r
}

async function chayVaDemMut(cot, focus) {
  const marks = await chayTrongTrang(cot, focus)
  const mut = await browser.execute(() => ({ mut: window.__mut, lateSel: window.__lateSel }))
  return { marks, mut }
}

async function bamThat() {
  const out = []
  const o = await $$('[data-col="tgt"]')
  for (let i = 0; i < SO_LUOT + 1; i += 1) {
    await browser.execute(() => {
      window.__resetP()
      window.__h = () => {
        const p = window.__p
        p.t0 = performance.now()
        window.__rafs()
        setTimeout(() => {
          p.tLong = performance.now()
        }, 0)
      }
      window.addEventListener('mousedown', window.__h, true)
    })
    await realClick(o[i % 2 === 0 ? DICH_B : DICH_A])
    await browser.pause(1500)
    const r = await browser.execute(() => {
      window.removeEventListener('mousedown', window.__h, true)
      return { ...window.__p }
    })
    if (i > 0) out.push(r)
  }
  return out
}

function med(xs) {
  const v = xs.filter((x) => x !== null).sort((a, b) => a - b)
  return v.length ? v[Math.floor(v.length / 2)] : null
}

async function doBienThe(ten, fn, extra = {}) {
  const got = await fn()
  const marks = Array.isArray(got) ? got : got.marks
  if (!Array.isArray(got)) extra = { ...extra, mutations: got.mut }
  const derived = marks.map(tinh)
  const medians = {}
  for (const k of ['sync', 'handlers', 'toFrame1', 'frame1', 'total', 'tLong']) medians[k] = med(derived.map((d) => d[k]))
  const anomalies = marks.map((m, i) => ({
    i,
    tSel_null: m.tSel === null,
    raf_null: m.tR1 === null || m.tR2 === null,
    raf_order_odd: m.tR1 !== null && m.tR2 !== null && !(m.tR2 >= m.tR1 && m.tR1 >= m.t0),
  }))
  const mot = { ten, n: marks.length, marks, derived, medians, anomalies, ...extra }
  ketQua.bien_the.push(mot)
  console.log(`[2.14-bansao] ${ten} medians:`, JSON.stringify(medians), 'tSel_null:', anomalies.filter((a) => a.tSel_null).length)
}


async function bienTheSao(ten, khoa, mode, cot) {
  await nap()
  const css = khoa ? GHI_DE[khoa] : ''
  const info = await sao(css, mode)
  console.log(`[2.14-bansao] ${ten} dom:`, JSON.stringify(info.sau))
  for (const c of cot) {
    await doBienThe(`${ten}-${c.cot}`, () => chayVaDemMut(c.cot, c.focus), { css_chen: css, mode, dom_truoc: info.truoc, dom_sau: info.sau, node_count: info.sau.all })
  }
}

describe('Ban sao tach app 2.14', function () {
  this.timeout(2_300_000)
  before(async function () {
    this.timeout(600_000)
    ketQua.tai_truoc = taiMay()
    console.log('[2.14-bansao] tai truoc:', ketQua.tai_truoc)
    await openWorkspaceWithWork('2.14 — ban sao')
    await browser.setTimeout({ script: 300_000 })
    await browser.execute(async (text) => {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: '2.14-bansao-' + Date.now(),
        sourceLang: 'zh',
        genre: 'general',
        text,
      })
    }, vanBan(SO_CAU))
    await nap()
    ketQua.ua = await browser.execute(() => navigator.userAgent)
    console.log('[2.14-bansao] ua:', ketQua.ua)
  })

  it('0 in-app control', async function () {
    this.timeout(900_000)
    await nap()
    const n = await browser.execute(() => document.querySelectorAll('*').length)
    await doBienThe('0-in-app-tgt', () => chayTrongTrang('tgt', true), { node_count: n })
  })
  it('1 detached clone', async function () {
    this.timeout(900_000)
    await bienTheSao('1-clone', null, null, [{ cot: 'src', focus: false }, { cot: 'tgt', focus: true }])
  })
  for (const [khoa, mode] of [
    ['a-grid-block', null],
    ['a2-col-block-only', null],
    ['b-hide-other-cols', null],
    ['c1-scroll-visible', null],
    ['c2-scroll-hidden', null],
    ['d-user-select-tgt-only', null],
    ['g-tgt-white-space-normal', null],
    ['h-no-cell-borders', null],
    ['i-panel-overflow-visible', null],
    ['e-grid-alone-in-body', 'grid-rieng'],
    ['e2-grid-scroll-alone-in-body', 'grid-scroll-rieng'],
  ]) {
    it(`2 ${khoa}`, async function () {
      this.timeout(900_000)
      await bienTheSao(`2-${khoa}`, GHI_DE[khoa] ? khoa : null, mode, [{ cot: 'tgt', focus: true }])
    })
  }
  it('3 detached clone repeat (drift)', async function () {
    this.timeout(900_000)
    await bienTheSao('3-clone-drift', null, null, [{ cot: 'src', focus: false }, { cot: 'tgt', focus: true }])
  })

  after(() => {
    ketQua.tai_sau = taiMay()
    console.log('[2.14-bansao] tai sau:', ketQua.tai_sau)
    const tep = path.join(THU_MUC, `ket-qua-ban-sao-${new Date().toISOString().replace(/:/g, '-')}.json`)
    fs.writeFileSync(tep, JSON.stringify(ketQua, null, 2))
    console.log('[2.14-bansao] json:', tep)
  })
})
