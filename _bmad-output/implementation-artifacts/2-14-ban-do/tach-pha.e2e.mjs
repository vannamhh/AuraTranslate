/**
 * Tach pha Story 2.14: thoi gian doi con tro nam o JS ung dung hay o render WebKit?
 * DOM day du, mot phien, reload + lang lai truoc moi bien the. Ban debug nap Vite dev.
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

async function doBienThe(ten, fn) {
  const marks = await fn()
  const derived = marks.map(tinh)
  const medians = {}
  for (const k of ['sync', 'handlers', 'toFrame1', 'frame1', 'total', 'tLong']) medians[k] = med(derived.map((d) => d[k]))
  const anomalies = marks.map((m, i) => ({
    i,
    tSel_null: m.tSel === null,
    raf_null: m.tR1 === null || m.tR2 === null,
    raf_order_odd: m.tR1 !== null && m.tR2 !== null && !(m.tR2 >= m.tR1 && m.tR1 >= m.t0),
  }))
  const mot = { ten, n: marks.length, marks, derived, medians, anomalies }
  ketQua.bien_the.push(mot)
  console.log(`[2.14-tachpha] ${ten} medians:`, JSON.stringify(medians), 'tSel_null:', anomalies.filter((a) => a.tSel_null).length)
}

describe('Tach pha 2.14', function () {
  this.timeout(1_700_000)
  before(async function () {
    this.timeout(600_000)
    ketQua.tai_truoc = taiMay()
    console.log('[2.14-tachpha] tai truoc:', ketQua.tai_truoc)
    await openWorkspaceWithWork('2.14 — tach pha')
    await browser.setTimeout({ script: 300_000 })
    await browser.execute(async (text) => {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: '2.14-tachpha-' + Date.now(),
        sourceLang: 'zh',
        genre: 'general',
        text,
      })
    }, vanBan(SO_CAU))
    await nap()
    ketQua.ua = await browser.execute(() => navigator.userAgent)
    console.log('[2.14-tachpha] ua:', ketQua.ua)
  })

  it('1 tgt setPosition + focus', async function () {
    this.timeout(900_000)
    await nap()
    await doBienThe('1-tgt-setPosition', () => chayTrongTrang('tgt', true))
  })
  it('2 src setPosition', async function () {
    this.timeout(900_000)
    await nap()
    await doBienThe('2-src-setPosition', () => chayTrongTrang('src', false))
  })
  it('3 tgt realClick', async function () {
    this.timeout(900_000)
    await nap()
    await doBienThe('3-tgt-realClick', bamThat)
  })
  it('4 tgt setPosition + focus (drift control)', async function () {
    this.timeout(900_000)
    await nap()
    await doBienThe('4-tgt-setPosition-drift', () => chayTrongTrang('tgt', true))
  })

  after(() => {
    ketQua.tai_sau = taiMay()
    console.log('[2.14-tachpha] tai sau:', ketQua.tai_sau)
    const tep = path.join(THU_MUC, `ket-qua-tach-pha-${new Date().toISOString().replace(/:/g, '-')}.json`)
    fs.writeFileSync(tep, JSON.stringify(ketQua, null, 2))
    console.log('[2.14-tachpha] json:', tep)
  })
})
