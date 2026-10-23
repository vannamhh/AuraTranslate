/**
 * Phat lai dot bien Story 2.14: ghi cac mutation Vue lam moi lan doi con tro (trong app), roi phat lai dung chung bang
 * DOM thuan tren ban sao da go Vue. Ban debug nap Vite dev.
 */
import fs from 'node:fs'
import path from 'node:path'
import { execSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'

const SO_CAU = 9850
const SO_LUOT = 6
const DICH_A = 3
const DICH_B = 25
const TOI_DA_NHOM = 10
const THU_MUC = path.dirname(fileURLToPath(import.meta.url))
const ketQua = { ua: null, tai_truoc: null, tai_sau: null, pha1: null, bien_the: [] }
let nhatKy = null

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

function med(xs) {
  const v = xs.filter((x) => x !== null).sort((a, b) => a - b)
  return v.length ? v[Math.floor(v.length / 2)] : null
}

async function saoGoVue() {
  const r = await browser.execute(() => {
    const app = document.querySelector('#app')
    const clone = app.cloneNode(true)
    const vueApp = app.__vue_app__
    if (!vueApp) return { loi: 'khong co __vue_app__' }
    vueApp.unmount()
    app.replaceWith(clone)
    void document.body.offsetHeight
    return {
      all: document.querySelectorAll('*').length,
      tgt: document.querySelectorAll('[data-col="tgt"]').length,
      tgt_ce: document.querySelectorAll('[data-col="tgt"][contenteditable]').length,
      vue_con: !!document.querySelector('#app')?.__vue_app__,
    }
  })
  if (r.loi) throw new Error(r.loi)
  await langLai()
  await browser.pause(800)
  return r
}

// Pha 1: ghi mutation trong app
async function pha1() {
  return browser.execute(
    async (a, b) => {
      const app = document.querySelector('#app')
      const tomTat = (n) => {
        if (n.nodeType === 3) return { tag: '#text', text: n.data.slice(0, 60) }
        if (n.nodeType === 8) return { tag: '#comment' }
        return { tag: n.tagName.toLowerCase(), cls: n.getAttribute('class') ?? '', html: n.outerHTML.slice(0, 300) }
      }
      const colName = (c) => (c ? [...c.classList].find((x) => x.startsWith('col-')) ?? 'col' : null)
      const colOf = (n) => (n.nodeType === 1 ? n : n.parentNode)?.closest?.('.col') ?? null
      const ordinal = (r) => {
        let k = 0
        for (let e = r.previousElementSibling; e; e = e.previousElementSibling) if (!e.classList.contains('strip-pad')) k += 1
        return k
      }
      const idx = (x) => Array.prototype.indexOf.call(x.parentNode.childNodes, x)
      const locate = (n) => {
        if (!app.contains(n)) return { detached: true }
        const col = colOf(n)
        if (!col) {
          const path = []
          for (let x = n; x !== app; x = x.parentNode) path.unshift(idx(x))
          return { kind: 'other', path }
        }
        if (n === col) return { kind: 'col', col: colName(col), row: null, path: [] }
        let r = n
        while (r.parentNode !== col) r = r.parentNode
        if (r.classList?.contains('strip-pad')) return { kind: 'strip', col: colName(col), unsupported: true }
        const path = []
        for (let x = n; x !== r; x = x.parentNode) path.unshift(idx(x))
        return { kind: 'col', col: colName(col), row: ordinal(r), path, seg: r.getAttribute('data-segment-id') }
      }
      const toks = (s) => new Set((s ?? '').split(/\s+/).filter(Boolean))
      const diff = (o, nw) => {
        const O = toks(o)
        const N = toks(nw)
        return [...N].filter((x) => !O.has(x)).map((x) => '+' + x).concat([...O].filter((x) => !N.has(x)).map((x) => '-' + x)).sort().join(' ')
      }
      const cells = document.querySelectorAll('[data-col="tgt"]')
      const buf = []
      const mo = new MutationObserver((l) => { buf.push(...l) })
      const drain = () => { const r = buf.splice(0).concat(mo.takeRecords()); return r }
      mo.observe(document, { subtree: true, attributes: true, attributeOldValue: true, childList: true, characterData: true, characterDataOldValue: true })
      const moves = []
      const caps = [b, a, b]
      for (let i = 0; i < caps.length; i += 1) {
        const cell = cells[caps[i]]
        drain()
        window.__resetP()
        const p = window.__p
        p.t0 = performance.now()
        window.__rafs()
        cell.focus()
        getSelection().setPosition(cell.firstChild ?? cell, 0)
        p.t1 = performance.now()
        await new Promise((r) => setTimeout(r, 1200))
        const recs = drain()
        const out = []
        recs.forEach((rec, k) => {
          const base = { order: k, type: rec.type, loc: locate(rec.target), tag: rec.target.nodeType === 1 ? rec.target.tagName.toLowerCase() : '#' + rec.target.nodeName }
          base.col = base.loc.col ?? (base.loc.detached ? 'detached' : 'other')
          if (rec.type === 'attributes') {
            let nv = null
            let found = false
            for (let j = k + 1; j < recs.length; j += 1) {
              if (recs[j].type === 'attributes' && recs[j].target === rec.target && recs[j].attributeName === rec.attributeName) {
                nv = recs[j].oldValue
                found = true
                break
              }
            }
            if (!found) nv = rec.target.getAttribute(rec.attributeName)
            base.attributeName = rec.attributeName
            base.oldValue = rec.oldValue
            base.newValue = nv
            base.group = `${base.col}|attr:${rec.attributeName}` + (rec.attributeName === 'class' ? '|' + diff(rec.oldValue, nv) : '')
            base.cls = rec.target.getAttribute?.('class') ?? null
          } else if (rec.type === 'characterData') {
            let nv = null
            let found = false
            for (let j = k + 1; j < recs.length; j += 1) {
              if (recs[j].type === 'characterData' && recs[j].target === rec.target) {
                nv = recs[j].oldValue
                found = true
                break
              }
            }
            if (!found) nv = rec.target.data
            base.oldValue = rec.oldValue
            base.newValue = nv
            base.group = `${base.col}|characterData`
          } else {
            base.added = [...rec.addedNodes].map(tomTat)
            base.removed = [...rec.removedNodes].map(tomTat)
            const a0 = rec.addedNodes[0]
            const prev = a0 && a0.parentNode === rec.target ? a0.previousElementSibling : null
            base.anchorRow = prev && !prev.classList.contains('strip-pad') ? ordinal(prev) : null
            const nm = (xs) => xs.map((x) => x.tag + (x.cls ? '.' + x.cls.split(/\s+/).join('.') : '')).join(',')
            base.group = `${base.col}|childList|+${nm(base.added)}|-${nm(base.removed)}`
          }
          out.push(base)
        })
        moves.push({ cell: caps[i], marks: { ...p }, mutations: out })
        await new Promise((r) => setTimeout(r, 300))
      }
      drain()
      cells[a].setAttribute('data-canary', '1')
      cells[a].removeAttribute('data-canary')
      await new Promise((r) => setTimeout(r, 50))
      const canary = drain().length
      mo.disconnect()
      return { moves, canary }
    },
    DICH_A,
    DICH_B,
  )
}

function resumePha1(moves) {
  const demKieu = {}
  const demNhom = {}
  const dong = []
  moves.forEach((mv, i) => {
    const rows = {}
    for (const m of mv.mutations) {
      const kieu = m.type === 'attributes' ? `attributes:${m.attributeName}` : m.type
      demKieu[kieu] = (demKieu[kieu] ?? 0) + 1
      demNhom[m.group] = (demNhom[m.group] ?? 0) + 1
      const key = `${m.col}#${m.loc.row ?? 'x'}${m.loc.seg ? '(' + m.loc.seg.slice(0, 8) + ')' : ''}`
      rows[key] = (rows[key] ?? 0) + 1
    }
    dong.push({ move: i + 1, cell: mv.cell, tong_mutation: mv.mutations.length, so_hang_khac_nhau: new Set(Object.keys(rows).map((k) => k.split('#')[1].split('(')[0])).size, theo_cot_hang: rows, total_ms: tinh(mv.marks).total })
  })
  return { dem_theo_kieu_ca_ba_luot: demKieu, dem_theo_nhom: demNhom, theo_luot: dong }
}

// Pha 2: phat lai tren ban sao
async function caiOps(moves) {
  const mapped = moves.map((m) => m.mutations)
  await browser.execute((muts) => {
    window.__muts = { m1: muts[0], m2: muts[1], m3: muts[2] }
    window.__prep = (key, filter) => {
      const ops = []
      const cache = {}
      const rowsOf = (cn) => {
        if (!cache[cn]) {
          const col = document.querySelector('.' + cn)
          if (!col) throw new Error('khong thay cot ' + cn)
          cache[cn] = { col, rows: [...col.children].filter((c) => !c.classList.contains('strip-pad')) }
        }
        return cache[cn]
      }
      const resolve = (m) => {
        let node
        if (m.loc.kind === 'other') {
          node = document.querySelector('#app')
          for (const i of m.loc.path) node = node?.childNodes[i]
        } else {
          const { col, rows } = rowsOf(m.loc.col)
          node = m.loc.row === null ? col : rows[m.loc.row]
          for (const i of m.loc.path) node = node?.childNodes[i]
        }
        if (!node) throw new Error('khong anh xa duoc ' + JSON.stringify(m.loc) + ' ' + m.group)
        return node
      }
      for (const m of window.__muts[key]) {
        if (m.loc.detached) continue
        if (m.loc.unsupported) throw new Error('mutation tren strip-pad khong ho tro ' + m.group)
        if (filter && m.group !== filter) continue
        const node = resolve(m)
        if (m.type === 'attributes') {
          const nv = m.newValue
          ops.push(() => (nv === null ? node.removeAttribute(m.attributeName) : node.setAttribute(m.attributeName, nv)))
        } else if (m.type === 'characterData') {
          ops.push(() => {
            node.data = m.newValue
          })
        } else {
          const anchor = m.anchorRow === null ? null : rowsOf(m.col).rows[m.anchorRow]
          ops.push(() => {
            for (const r of m.removed) {
              if (r.tag.startsWith('#')) continue
              const hit = [...node.children].find((c) => c.tagName.toLowerCase() === r.tag && (c.getAttribute('class') ?? '') === r.cls)
              if (hit) hit.remove()
            }
            for (const a of m.added) {
              if (a.tag.startsWith('#')) continue
              const t = document.createElement('template')
              t.innerHTML = a.html
              const el = t.content.firstElementChild
              node.insertBefore(el, anchor ? anchor.nextSibling : node.firstChild)
            }
          })
        }
      }
      return ops
    }
  }, mapped)
}

async function chayReplay(sel, filtro, replay) {
  return browser.execute(
    async (selMode, filter, doReplay, a, b, n) => {
      const out = []
      const tgt = document.querySelectorAll('[data-col="tgt"]')
      const src = document.querySelectorAll('[data-col="src"]')
      for (let i = 0; i < n + 1; i += 1) {
        const k = i % 2 === 0 ? b : a
        const key = i === 0 ? 'm1' : i % 2 === 1 ? 'm2' : 'm3'
        const ops = doReplay ? window.__prep(key, filter) : []
        if (selMode === 'none') {
          getSelection().removeAllRanges()
          document.activeElement?.blur?.()
        } else if (selMode === 'src') {
          document.activeElement?.blur?.()
        }
        await new Promise((r) => setTimeout(r, 50))
        window.__resetP()
        const p = window.__p
        p.t0 = performance.now()
        window.__rafs()
        if (selMode === 'tgt') {
          const cell = tgt[k]
          cell.focus()
          getSelection().setPosition(cell.firstChild ?? cell, 0)
        } else if (selMode === 'src') {
          const cell = src[k]
          getSelection().setPosition(cell.firstChild ?? cell, 0)
        }
        for (const op of ops) op()
        p.t1 = performance.now()
        setTimeout(() => {
          p.tLong = performance.now()
        }, 0)
        await new Promise((r) => setTimeout(r, 1200))
        out.push({ ...p, n_ops: ops.length })
        await new Promise((r) => setTimeout(r, 300))
      }
      return out.slice(1)
    },
    sel,
    filtro,
    replay,
    DICH_A,
    DICH_B,
    SO_LUOT,
  )
}

async function bienThe(ten, desc, sel, filtro, replay) {
  await nap()
  const dom = await saoGoVue()
  await caiOps(nhatKy)
  const marks = await chayReplay(sel, filtro, replay)
  const derived = marks.map(tinh)
  const medians = {}
  for (const k of ['sync', 'handlers', 'toFrame1', 'frame1', 'total', 'tLong']) medians[k] = med(derived.map((d) => d[k]))
  ketQua.bien_the.push({ ten, replayed: desc, selection: sel, filtro, n: marks.length, n_ops: marks.map((m) => m.n_ops), dom, marks, derived, medians })
  console.log(`[2.14-phatlai] ${ten} (${desc}; sel=${sel}) medians:`, JSON.stringify(medians), 'n_ops:', marks.map((m) => m.n_ops).join(','))
}

describe('Phat lai dot bien 2.14', function () {
  this.timeout(2_350_000)
  before(async function () {
    this.timeout(600_000)
    ketQua.tai_truoc = taiMay()
    console.log('[2.14-phatlai] tai truoc:', ketQua.tai_truoc)
    await openWorkspaceWithWork('2.14 — phat lai')
    await browser.setTimeout({ script: 300_000 })
    await browser.execute(async (text) => {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: '2.14-phatlai-' + Date.now(),
        sourceLang: 'zh',
        genre: 'general',
        text,
      })
    }, vanBan(SO_CAU))
    await nap()
    ketQua.ua = await browser.execute(() => navigator.userAgent)
    console.log('[2.14-phatlai] ua:', ketQua.ua)
  })

  it('Pha 1 ghi mutation trong app', async function () {
    this.timeout(300_000)
    await nap()
    const r1 = await pha1()
    nhatKy = r1.moves
    ketQua.pha1 = { canary_mutations_expected_2: r1.canary, nhat_ky: nhatKy, tom_tat: resumePha1(nhatKy) }
    console.log('[2.14-phatlai] canary MO (2 mong doi):', r1.canary)
    console.log('[2.14-phatlai] pha1 tom tat:', JSON.stringify(ketQua.pha1.tom_tat))
  })

  it('Pha 2 R0..R3', async function () {
    this.timeout(1_200_000)
    await bienThe('R0', 'khong phat lai', 'tgt', null, false)
    await bienThe('R1', 'tron bo', 'tgt', null, true)
    await bienThe('R2', 'tron bo', 'src', null, true)
    await bienThe('R3', 'tron bo', 'none', null, true)
  })

  it('Pha 2 R4.. tung nhom', async function () {
    this.timeout(1_800_000)
    const dem = ketQua.pha1.tom_tat.dem_theo_nhom
    const nhom = Object.keys(dem).filter((g) => !g.startsWith('detached')).sort((x, y) => dem[y] - dem[x])
    ketQua.nhom_bi_bo = nhom.slice(TOI_DA_NHOM)
    let i = 4
    for (const g of nhom.slice(0, TOI_DA_NHOM)) {
      await bienThe(`R${i}`, g, 'tgt', g, true)
      i += 1
    }
  })

  it('Pha 2 R-last lap R0', async function () {
    this.timeout(600_000)
    await bienThe('R-last', 'khong phat lai (troi)', 'tgt', null, false)
  })

  after(() => {
    ketQua.tai_sau = taiMay()
    console.log('[2.14-phatlai] tai sau:', ketQua.tai_sau)
    const tep = path.join(THU_MUC, `ket-qua-phat-lai-${new Date().toISOString().replace(/:/g, '-')}.json`)
    fs.writeFileSync(tep, JSON.stringify(ketQua, null, 2))
    console.log('[2.14-phatlai] json:', tep)
  })
})
