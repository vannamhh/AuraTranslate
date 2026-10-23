/**
 * Doi chung go that cho Story 2.14: chi phi doi con tro co ty le voi TONG KICH THUOC DOM cua luoi khong?
 * Mot phien, reload + lang lai truoc moi bien the. Giu N hang dau (remove() moi con theo hang o ca bon cot),
 * diem den la hang 3 va 8 (chi so 0). Do tu luc thao tac toi het hai rAF. Ban debug nap Vite dev.
 */
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { execSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { realClick } from '../../../e2e/support/pointer.mjs'

const SO_CAU = 9850
const SO_LUOT = 4
const DICH_A = 3
const DICH_B = 8
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
}

async function demNut() {
  return browser.execute(() => document.querySelector('.grid').querySelectorAll('*').length)
}

async function giuN(n) {
  return browser.execute((keep) => {
    const grid = document.querySelector('.grid')
    let removed = 0
    for (const sel of ['.col-rule', '.col-num', '.col-src', '.col-tgt']) {
      const col = grid.querySelector(`:scope > ${sel}`)
      let idx = -1
      for (const child of [...col.children]) {
        if (child.classList.contains('strip-pad')) {
          if (idx >= keep) {
            child.remove()
            removed += 1
          }
          continue
        }
        idx += 1
        if (idx >= keep) {
          child.remove()
          removed += 1
        }
      }
    }
    grid.style.gridTemplateRows = `repeat(${keep}, auto)`
    return { removed, tgt_con_lai: document.querySelectorAll('[data-col="tgt"]').length }
  }, n)
}

async function setPos() {
  return browser.execute(
    async (a, b, n) => {
      const out = []
      const cells = document.querySelectorAll('[data-col="tgt"]')
      for (let i = 0; i < n + 1; i += 1) {
        const cell = cells[i % 2 === 0 ? b : a]
        const t0 = performance.now()
        getSelection().setPosition(cell.firstChild ?? cell, 0)
        await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))
        const ms = +(performance.now() - t0).toFixed(2)
        if (i > 0) out.push(ms)
        await new Promise((r) => setTimeout(r, 700))
      }
      return out
    },
    DICH_A,
    DICH_B,
    SO_LUOT,
  )
}

async function bamThat() {
  const down = []
  const o = await $$('[data-col="tgt"]')
  for (let i = 0; i < SO_LUOT + 1; i += 1) {
    await browser.execute(() => {
      window.__b = null
      window.__h = () => {
        const t0 = performance.now()
        requestAnimationFrame(() =>
          requestAnimationFrame(() => {
            window.__b = +(performance.now() - t0).toFixed(2)
          }),
        )
      }
      document.addEventListener('mousedown', window.__h, true)
    })
    await realClick(o[i % 2 === 0 ? DICH_B : DICH_A])
    await browser.pause(1500)
    const r = await browser.execute(() => {
      document.removeEventListener('mousedown', window.__h, true)
      return window.__b
    })
    if (i > 0) down.push(r)
  }
  return down
}

function thongKe(lan) {
  const sorted = [...lan].sort((a, b) => a - b)
  return { trung_vi: sorted[Math.floor(sorted.length / 2)], max: sorted.at(-1) }
}

async function doBienThe(ten, n, nut, extra = {}) {
  const sp = await setPos()
  const ck = await bamThat()
  const mot = { ten, N: n, so_nut: nut, ...extra, setPosition: { lan: sp, ...thongKe(sp) }, click_that: { lan: ck, ...thongKe(ck) } }
  ketQua.bien_the.push(mot)
  console.log(`[2.14-kichthuoc] ${ten}:`, JSON.stringify(mot))
}

describe('Doi chung kich thuoc DOM 2.14', function () {
  this.timeout(1_700_000)
  before(async function () {
    this.timeout(600_000)
    ketQua.tai_truoc = taiMay()
    console.log('[2.14-kichthuoc] tai truoc:', ketQua.tai_truoc)
    await openWorkspaceWithWork('2.14 — kich thuoc DOM')
    await browser.setTimeout({ script: 300_000 })
    await browser.execute(async (text) => {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: '2.14-kichthuoc-' + Date.now(),
        sourceLang: 'zh',
        genre: 'general',
        text,
      })
    }, vanBan(SO_CAU))
    await nap()
    ketQua.ua = await browser.execute(() => navigator.userAgent)
    console.log('[2.14-kichthuoc] ua:', ketQua.ua)
  })

  it('1 Control A day du', async function () {
    this.timeout(900_000)
    await nap()
    await doBienThe('A', SO_CAU, await demNut())
  })

  for (const n of [500, 2000, 5000]) {
    it(`2 Giu N=${n}`, async function () {
      this.timeout(900_000)
      await nap()
      const r = await giuN(n)
      await langLai()
      await doBienThe(`N${n}`, n, await demNut(), r)
    })
  }

  it('3 Control B day du', async function () {
    this.timeout(900_000)
    await nap()
    await doBienThe('B', SO_CAU, await demNut())
  })

  after(() => {
    ketQua.tai_sau = taiMay()
    console.log('[2.14-kichthuoc] tai sau:', ketQua.tai_sau)
    const tep = path.join(THU_MUC, `ket-qua-kich-thuoc-dom-${new Date().toISOString().replace(/:/g, '-')}.json`)
    fs.writeFileSync(tep, JSON.stringify(ketQua, null, 2))
    console.log('[2.14-kichthuoc] json:', tep)
  })
})
