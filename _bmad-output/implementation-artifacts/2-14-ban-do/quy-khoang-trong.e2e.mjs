/**
 * Quy khoang trong Story 2.14: ~460-510 ms cua t0->rAF1 khong thuoc render/patch cua component nao.
 * Gan vao doc layout, IPC, timer, handler su kien, va khoi chan luong chinh (MessageChannel ping).
 * Bien the: (a) gan dung cu + setPosition, (b) go dung cu (doi chung), (c) gan dung cu + realClick.
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
    const now = () => performance.now()
    const orig = {
      st: window.setTimeout.bind(window),
      raf: window.requestAnimationFrame.bind(window),
    }
    window.__orig = orig
    const I = { t0: null, tR1: null, tEnd: null, tSel: null, rec: null, hook: null, on: false, undo: [] }
    window.__I = I
    window.__vm = []
    new PerformanceObserver((list) => {
      for (const e of list.getEntries()) window.__vm.push({ name: e.name, start: e.startTime, dur: e.duration })
    }).observe({ type: 'measure' })
    document.querySelector('#app').__vue_app__.config.performance = true

    const act = () => I.on && I.t0 !== null && I.tEnd === null
    const stk = (skip) => new Error().stack.split('\n').slice(skip, skip + 4)
    const newRec = () => ({ layout: [], agg: {}, ipc: [], timers: [], ev: [], gaps: [], n_ping: 0 })
    I.newRec = newRec
    const note = (label, s, d, skip) => {
      const r = I.rec
      const a = (r.agg[label] ??= { n: 0, ms: 0 })
      a.n += 1
      a.ms += d
      if (d > 1) r.layout.push({ name: label, t: s, dur: d, frames: stk(skip) })
    }
    const wrapFn = (obj, name, label) => {
      const o = obj[name]
      if (typeof o !== 'function') return
      obj[name] = function (...a) {
        if (!act()) return o.apply(this, a)
        const s = now()
        try {
          return o.apply(this, a)
        } finally {
          note(label, s, now() - s, 3)
        }
      }
      I.undo.push(() => {
        obj[name] = o
      })
    }
    const wrapGet = (obj, name, label) => {
      const d = Object.getOwnPropertyDescriptor(obj, name)
      if (!d || !d.get) return
      const g = d.get
      Object.defineProperty(obj, name, {
        ...d,
        get() {
          if (!act()) return g.call(this)
          const s = now()
          try {
            return g.call(this)
          } finally {
            note(label, s, now() - s, 3)
          }
        },
      })
      I.undo.push(() => Object.defineProperty(obj, name, d))
    }
    const wrapSched = (name, kind) => {
      const o = window[name]
      if (typeof o !== 'function') return
      window[name] = function (fn, ...rest) {
        if (!act() || typeof fn !== 'function') return o.call(window, fn, ...rest)
        const r = I.rec
        const sched = now()
        const frames = stk(2)
        const isRaf = kind === 'raf'
        return o.call(
          window,
          function (...args) {
            const s = now()
            try {
              return fn.apply(this, args)
            } finally {
              r.timers.push({ kind, delay: isRaf ? null : rest[0] ?? 0, sched, t: s, dur: now() - s, frames })
            }
          },
          ...rest,
        )
      }
      I.undo.push(() => {
        window[name] = o
      })
    }
    const evTypes = ['focusin', 'focusout', 'focus', 'blur', 'mousedown', 'mouseup']
    const listeners = []
    const addL = (target, type, fn, cap) => {
      target.addEventListener(type, fn, cap)
      listeners.push(() => target.removeEventListener(type, fn, cap))
    }
    const mark = (type, phase, e) =>
      I.rec && I.t0 !== null && I.tEnd === null && I.rec.ev.push({ type, phase, t: now(), tgt: e?.target?.nodeName ?? null })

    const install = () => {
      if (I.on) return
      I.on = true
      I.undo = []
      for (const t of evTypes) {
        addL(
          window,
          t,
          (e) => {
            if (t === 'mousedown' && I.hook) I.hook()
            mark(t, 'cap', e)
            if (!act()) return
            const fn = () => mark(t, 'tgt_last', e)
            e.target.addEventListener(t, fn, { once: true })
          },
          true,
        )
        addL(document, t, (e) => mark(t, 'doc_bub', e), false)
      }
      addL(document, 'selectionchange', (e) => {
        mark('selectionchange', 'doc_first', e)
        if (I.t0 !== null && I.tSel === null) I.tSel = now()
      }, true)
      I.selLast = (e) => mark('selectionchange', 'doc_last', e)
      I.relast = () => {
        document.removeEventListener('selectionchange', I.selLast)
        document.addEventListener('selectionchange', I.selLast)
      }
      I.relast()
      listeners.push(() => document.removeEventListener('selectionchange', I.selLast))
      wrapFn(Element.prototype, 'getBoundingClientRect', 'Element.getBoundingClientRect')
      wrapFn(Element.prototype, 'getClientRects', 'Element.getClientRects')
      wrapFn(Range.prototype, 'getBoundingClientRect', 'Range.getBoundingClientRect')
      wrapFn(Range.prototype, 'getClientRects', 'Range.getClientRects')
      wrapFn(window, 'getComputedStyle', 'getComputedStyle')
      wrapFn(CSSStyleDeclaration.prototype, 'getPropertyValue', 'CSSStyleDeclaration.getPropertyValue')
      wrapFn(Element.prototype, 'scrollIntoView', 'scrollIntoView')
      wrapFn(HTMLElement.prototype, 'focus', 'focus')
      for (const n of ['offsetTop', 'offsetLeft', 'offsetWidth', 'offsetHeight', 'offsetParent'])
        wrapGet(HTMLElement.prototype, n, n)
      for (const n of ['clientTop', 'clientLeft', 'clientWidth', 'clientHeight', 'scrollTop', 'scrollLeft', 'scrollWidth', 'scrollHeight'])
        wrapGet(Element.prototype, n, n)
      wrapSched('setTimeout', 'timeout')
      wrapSched('requestAnimationFrame', 'raf')
      wrapSched('requestIdleCallback', 'idle')
      const T = window.__TAURI_INTERNALS__
      const oi = T.invoke
      T.invoke = function (cmd, ...rest) {
        if (!act()) return oi.call(this, cmd, ...rest)
        const e = { cmd, t: now(), settle: null, after: null, err: false, frames: stk(2) }
        I.rec.ipc.push(e)
        const p = oi.call(this, cmd, ...rest)
        const fin = (err) => {
          e.settle = now()
          e.err = err
          queueMicrotask(() => {
            e.after = now()
          })
        }
        Promise.resolve(p).then(() => fin(false), () => fin(true))
        return p
      }
      I.undo.push(() => {
        T.invoke = oi
      })
    }
    const uninstall = () => {
      if (!I.on) return
      I.on = false
      for (const u of I.undo.reverse()) u()
      for (const u of listeners.splice(0)) u()
    }
    window.__inst = { install, uninstall }

    window.__reset = () => {
      I.t0 = null
      I.tR1 = null
      I.tEnd = null
      I.tSel = null
      I.rec = newRec()
      window.__vm = []
      performance.clearMeasures()
      performance.clearMarks()
    }
    window.__start = () => {
      I.t0 = now()
      const r = I.rec
      let last = I.t0
      if (I.on) {
        const mc = new MessageChannel()
        mc.port1.onmessage = () => {
          const t = now()
          r.n_ping += 1
          if (t - last > 5) r.gaps.push({ start: last, end: t })
          last = t
          if (I.tEnd === null) mc.port2.postMessage(0)
          else mc.port1.close()
        }
        mc.port2.postMessage(0)
      }
      orig.raf(() => {
        I.tR1 = now()
        orig.raf(() => {
          I.tEnd = now()
        })
      })
    }
  })
}

async function chayTrongTrang(instr) {
  return browser.execute(
    async (a, b, n, instr) => {
      if (instr) window.__inst.install()
      else window.__inst.uninstall()
      const out = []
      const I = window.__I
      const cells = document.querySelectorAll('[data-col="tgt"]')
      for (let i = 0; i < n + 1; i += 1) {
        const cell = cells[i % 2 === 0 ? b : a]
        window.__reset()
        if (instr) I.relast()
        window.__start()
        cell.focus()
        getSelection().setPosition(cell.firstChild ?? cell, 0)
        await new Promise((r) => window.__orig.st(r, 1200))
        const { t0, tR1, tEnd, tSel, rec } = I
        out.push({ t0, tR1, tEnd, tSel, rec, vm: window.__vm.slice() })
        await new Promise((r) => window.__orig.st(r, 300))
      }
      window.__inst.uninstall()
      return out.slice(1)
    },
    DICH_A,
    DICH_B,
    SO_LUOT,
    instr,
  )
}

async function bamThat() {
  await browser.execute(() => window.__inst.install())
  const out = []
  const o = await $$('[data-col="tgt"]')
  for (let i = 0; i < SO_LUOT; i += 1) {
    await browser.execute(() => {
      window.__reset()
      window.__I.relast()
      window.__I.hook = () => {
        window.__I.hook = null
        window.__start()
      }
    })
    await realClick(o[i % 2 === 0 ? DICH_B : DICH_A])
    await browser.pause(1500)
    const r = await browser.execute(() => {
      const { t0, tR1, tEnd, tSel, rec } = window.__I
      return { t0, tR1, tEnd, tSel, rec, vm: window.__vm.slice() }
    })
    if (i > 0) out.push(r)
  }
  await browser.execute(() => window.__inst.uninstall())
  return out
}

function med(xs) {
  const v = xs.filter((x) => x !== null && x !== undefined).sort((a, b) => a - b)
  return v.length ? v[Math.floor(v.length / 2)] : null
}

function hopIv(iv) {
  const s = iv.filter(([x, y]) => y > x).sort((a, b) => a[0] - b[0])
  const out = []
  for (const [x, y] of s) {
    const l = out[out.length - 1]
    if (l && x <= l[1]) l[1] = Math.max(l[1], y)
    else out.push([x, y])
  }
  return out
}
const dai = (iv) => hopIv(iv).reduce((t, [x, y]) => t + (y - x), 0)
const cat = (iv, lo, hi) => iv.map(([x, y]) => [Math.max(x, lo), Math.min(y, hi)]).filter(([x, y]) => y > x)

const SRC = /(\/src\/[^\s?:)]+)(?:\?[^:\s)]*)?:(\d+):(\d+)/
function srcFrame(frames) {
  for (const f of frames ?? []) {
    const m = SRC.exec(f)
    if (m) return `${m[1].slice(1)}:${m[2]}`
  }
  return frames?.[0]?.slice(0, 80) ?? '?'
}

function phanTich(m) {
  const { t0, rec } = m
  const r1 = m.tR1 - t0
  const total = m.tEnd - t0
  const rel = (t) => t - t0
  const vue = m.vm
    .filter((e) => e.start >= t0 && e.start <= m.tEnd)
    .map((e) => {
      const i = e.name.lastIndexOf('> ')
      return { name: e.name, comp: e.name.slice(0, i + 1), type: e.name.slice(i + 2), rel: rel(e.start), dur: e.dur }
    })
  const vuePhan = vue.filter((e) => e.type === 'render' || e.type === 'patch')
  const layout = rec.layout.map((e) => ({ ...e, rel: rel(e.t), src: srcFrame(e.frames) }))
  const timers = rec.timers.map((e) => ({ ...e, rel: rel(e.t), sched_rel: rel(e.sched), src: srcFrame(e.frames) }))
  const ipc = rec.ipc.map((e) => ({
    cmd: e.cmd,
    rel: rel(e.t),
    settle_rel: e.settle === null ? null : rel(e.settle),
    handler_ms: e.after !== null && e.settle !== null ? e.after - e.settle : null,
    err: e.err,
    src: srcFrame(e.frames),
  }))
  const byType = {}
  const evPairs = []
  for (const e of rec.ev) (byType[e.type] ??= []).push(e)
  for (const [type, l] of Object.entries(byType)) {
    const caps = l.filter((e) => e.phase === 'cap' || e.phase === 'doc_first')
    const lasts = l.filter((e) => e.phase === 'tgt_last' || e.phase === 'doc_last')
    caps.forEach((c, i) => lasts[i] && evPairs.push({ type, rel: rel(c.t), end: rel(lasts[i].t), dur: lasts[i].t - c.t }))
  }
  const ev = rec.ev.map((e) => ({ type: e.type, phase: e.phase, rel: rel(e.t), tgt: e.tgt }))
  const gaps = rec.gaps.map((g) => ({ start: rel(g.start), end: rel(g.end), dur: g.end - g.start }))
  const iVue = vuePhan.map((e) => [e.rel, e.rel + e.dur])
  const iLay = layout.map((e) => [e.rel, e.rel + e.dur])
  const iTim = timers.map((e) => [e.rel, e.rel + e.dur])
  const iEv = evPairs.map((e) => [e.rel, e.end])
  const iIpc = ipc.filter((e) => e.handler_ms !== null).map((e) => [e.settle_rel, e.settle_rel + e.handler_ms])
  const iGap = gaps.map((g) => [g.start, g.end])
  const known = [...iVue, ...iLay, ...iTim, ...iEv, ...iIpc]
  const c = (iv) => dai(cat(iv, 0, r1))
  const covKnown = c(known)
  const cov = {
    r1,
    total,
    vue: c(iVue),
    layout_union: c(iLay),
    layout_sum_hits: cat(iLay, 0, r1).reduce((t, [x, y]) => t + y - x, 0),
    layout_sum_all_calls: Object.values(rec.agg).reduce((t, a) => t + a.ms, 0),
    timers: c(iTim),
    events: c(iEv),
    ipc_handlers: c(iIpc),
    known_union: covKnown,
    uncovered: r1 - covKnown,
    blocks_union: c(iGap),
    blocks_not_known: c([...known, ...iGap]) - covKnown,
    n_ping: rec.n_ping,
  }
  const timeline = [
    ...vue.map((e) => ({ kind: 'vue', rel: e.rel, dur: e.dur, name: e.name })),
    ...layout.map((e) => ({ kind: 'layout', rel: e.rel, dur: e.dur, name: e.name, src: e.src, frames: e.frames })),
    ...ipc.map((e) => ({ kind: 'ipc', rel: e.rel, settle_rel: e.settle_rel, handler_ms: e.handler_ms, name: e.cmd, src: e.src })),
    ...timers.map((e) => ({ kind: e.kind, rel: e.rel, dur: e.dur, sched_rel: e.sched_rel, delay: e.delay, src: e.src })),
    ...ev.map((e) => ({ kind: 'event', rel: e.rel, name: `${e.type}:${e.phase}` })),
    ...gaps.map((g) => ({ kind: 'block', rel: g.start, end: g.end, dur: g.dur })),
  ].sort((a, b) => a.rel - b.rel)
  const cons = {}
  const add = (k, d) => {
    cons[k] ??= { ms: 0, n: 0 }
    cons[k].ms += d
    cons[k].n += 1
  }
  for (const e of layout) add(`layout ${e.name} @ ${e.src}`, e.dur)
  for (const e of timers) add(`${e.kind} cb scheduled @ ${e.src}`, e.dur)
  for (const e of ipc) add(`ipc ${e.cmd} handler-after-settle @ ${e.src}`, e.handler_ms ?? 0)
  return { cov, timeline, evPairs, consumers: cons, agg: rec.agg, tSel: m.tSel === null ? null : m.tSel - t0 }
}

async function doBienThe(ten, fn, instr) {
  const marks = await fn(instr)
  const entry = { ten, instr, n: marks.length }
  if (!instr) {
    entry.marks = marks.map((m) => ({ t0: m.t0, tR1: m.tR1, tEnd: m.tEnd }))
    entry.totals = marks.map((m) => m.tEnd - m.t0)
    entry.r1s = marks.map((m) => m.tR1 - m.t0)
    entry.medians = { total: med(entry.totals), r1: med(entry.r1s) }
    ketQua.bien_the.push(entry)
    console.log(`[2.14-quy] ${ten} medians:`, JSON.stringify(entry.medians))
    return
  }
  const per = marks.map(phanTich)
  entry.per_sample = per
  entry.totals = per.map((p) => p.cov.total)
  entry.r1s = per.map((p) => p.cov.r1)
  const keys = Object.keys(per[0].cov)
  entry.cov_medians = Object.fromEntries(keys.map((k) => [k, med(per.map((p) => p.cov[k]))]))
  const all = new Set(per.flatMap((p) => Object.keys(p.consumers)))
  entry.consumers = Object.fromEntries(
    [...all]
      .map((k) => [k, { ms: med(per.map((p) => p.consumers[k]?.ms ?? 0)), n: med(per.map((p) => p.consumers[k]?.n ?? 0)) }])
      .sort((x, y) => y[1].ms - x[1].ms)
      .slice(0, 25),
  )
  const names = new Set(per.flatMap((p) => Object.keys(p.agg)))
  entry.agg_medians = Object.fromEntries(
    [...names].map((k) => [k, { ms: med(per.map((p) => p.agg[k]?.ms ?? 0)), n: med(per.map((p) => p.agg[k]?.n ?? 0)) }]),
  )
  entry.medians = { total: med(entry.totals), r1: med(entry.r1s) }
  ketQua.bien_the.push(entry)
  console.log(`[2.14-quy] ${ten} medians:`, JSON.stringify(entry.medians))
  console.log(`[2.14-quy] ${ten} coverage:`, JSON.stringify(entry.cov_medians))
  console.log(`[2.14-quy] ${ten} consumers:`, JSON.stringify(Object.entries(entry.consumers).slice(0, 10)))
  console.log(`[2.14-quy] ${ten} agg:`, JSON.stringify(entry.agg_medians))
  const mid = per[Math.floor(per.length / 2)]
  console.log(`[2.14-quy] ${ten} blocks sample:`, JSON.stringify(mid.timeline.filter((e) => e.kind === 'block').map((b) => [+b.rel.toFixed(1), +b.end.toFixed(1)])))
}

describe('Quy khoang trong 2.14', function () {
  this.timeout(1_700_000)
  before(async function () {
    this.timeout(600_000)
    ketQua.tai_truoc = taiMay()
    console.log('[2.14-quy] tai truoc:', ketQua.tai_truoc)
    await openWorkspaceWithWork('2.14 — quy khoang trong')
    await browser.setTimeout({ script: 300_000 })
    await browser.execute(async (text) => {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: '2.14-quy-' + Date.now(),
        sourceLang: 'zh',
        genre: 'general',
        text,
      })
    }, vanBan(SO_CAU))
    await nap()
    ketQua.ua = await browser.execute(() => navigator.userAgent)
    console.log('[2.14-quy] ua:', ketQua.ua)
  })

  it('a gan dung cu, tgt setPosition + focus', async function () {
    this.timeout(900_000)
    await langLai()
    await doBienThe('a-instr-setPosition', chayTrongTrang, true)
  })
  it('b go dung cu (doi chung)', async function () {
    this.timeout(900_000)
    await langLai()
    await doBienThe('b-control-setPosition', chayTrongTrang, false)
  })
  it('c gan dung cu, realClick', async function () {
    this.timeout(900_000)
    await langLai()
    await doBienThe('c-instr-realClick', bamThat, true)
  })

  after(() => {
    ketQua.tai_sau = taiMay()
    console.log('[2.14-quy] tai sau:', ketQua.tai_sau)
    const tep = path.join(THU_MUC, `ket-qua-quy-khoang-trong-${new Date().toISOString().replace(/:/g, '-')}.json`)
    fs.writeFileSync(tep, JSON.stringify(ketQua, null, 2))
    console.log('[2.14-quy] json:', tep)
  })
})
