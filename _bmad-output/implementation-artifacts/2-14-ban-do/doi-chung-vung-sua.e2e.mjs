/**
 * Doi chung go that cho Story 2.14: chi phi dời con tro co ty le voi so vung `contenteditable` khong?
 * Mot phien, reload + lang lai giua cac bien the. Do tu luc su kien toi het hai rAF.
 * Gioi han: ban debug nap Vite dev; o duoc giu sua duoc = diem den (3, 8, 5000) + K o chia deu.
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { realClick } from '../../../e2e/support/pointer.mjs'

const SO_CAU = 9850
const SO_LUOT = 4
const DICH_A = 3
const DICH_B = 5000
const CLICK_A = 3
const CLICK_B = 8

function vanBan(n) {
  const parts = []
  for (let i = 0; i < n; i += 1) parts.push(`第${i + 1}句话，用来量渲染成本。`)
  return parts.join('')
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

async function demSua() {
  return browser.execute(() => document.querySelectorAll('[data-col="tgt"][contenteditable]').length)
}

async function goSua(giu) {
  return browser.execute((keep) => {
    const cells = document.querySelectorAll('[data-col="tgt"]')
    const set = new Set(keep)
    cells.forEach((c, i) => {
      if (!set.has(i)) c.removeAttribute('contenteditable')
    })
    return document.querySelectorAll('[data-col="tgt"][contenteditable]').length
  }, giu)
}

function chiaDeu(k) {
  const out = new Set([DICH_A, DICH_B, CLICK_A, CLICK_B])
  const buoc = SO_CAU / k
  for (let j = 0; out.size < k + 4 && j < k; j += 1) out.add(Math.floor(j * buoc) + 1)
  return [...out]
}

async function setPos() {
  const lan = await browser.execute(
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
  return lan
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
    await realClick(o[i % 2 === 0 ? CLICK_B : CLICK_A])
    await browser.pause(1500)
    const r = await browser.execute(() => {
      document.removeEventListener('mousedown', window.__h, true)
      return window.__b
    })
    if (i > 0) down.push(r)
  }
  return down
}

function tomTat(ten, lan, extra = {}) {
  const sorted = [...lan].sort((a, b) => a - b)
  console.log(`[2.14-doichung] ${ten}:`, JSON.stringify({ ...extra, lan, trung_vi: sorted[Math.floor(sorted.length / 2)], max: sorted.at(-1) }))
}

async function doCaHai(ten, extra = {}, click = true) {
  tomTat(`${ten} setPosition`, await setPos(), extra)
  if (click) tomTat(`${ten} click that (mousedown)`, await bamThat(), extra)
}

describe('Doi chung vung sua 2.14', function () {
  this.timeout(1_700_000)
  before(async function () {
    this.timeout(600_000)
    await openWorkspaceWithWork('2.14 — doi chung')
    await browser.setTimeout({ script: 300_000 })
    await browser.execute(async (text) => {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: '2.14-doichung-' + Date.now(),
        sourceLang: 'zh',
        genre: 'general',
        text,
      })
    }, vanBan(SO_CAU))
    await nap()
    console.log('[2.14-doichung] ua:', await browser.execute(() => navigator.userAgent))
  })

  it('1 Control A', async function () {
    this.timeout(900_000)
    await nap()
    await doCaHai('A', { sua: await demSua() })
  })

  it('2 Go contenteditable tru diem den', async function () {
    this.timeout(900_000)
    await nap()
    const sua = await goSua([DICH_A, DICH_B, CLICK_A, CLICK_B])
    await langLai()
    await doCaHai('go', { sua })
    console.log('[2.14-doichung] sua con lai sau do:', await demSua())
  })

  for (const k of [100, 1000, 5000]) {
    it(`3 Giu K=${k}`, async function () {
      this.timeout(900_000)
      await nap()
      const sua = await goSua(chiaDeu(k))
      await langLai()
      await doCaHai(`K${k}`, { sua }, false)
    })
  }

  it('4 content-visibility auto', async function () {
    this.timeout(900_000)
    await nap()
    const h = await browser.execute(() => {
      const row = document.querySelectorAll('[data-col="tgt"]')[0]
      const r = row.getBoundingClientRect()
      const s = document.createElement('style')
      s.textContent = `[data-col="src"],[data-col="tgt"]{content-visibility:auto;contain-intrinsic-size:auto ${Math.round(r.height)}px}`
      document.head.appendChild(s)
      return Math.round(r.height)
    })
    await langLai()
    await doCaHai('cv', { sua: await demSua(), h })
  })

  it('5 Control B', async function () {
    this.timeout(900_000)
    await nap()
    await doCaHai('B', { sua: await demSua() })
  })
})
