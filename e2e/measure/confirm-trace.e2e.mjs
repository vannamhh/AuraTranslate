/**
 * Bàn đo pha 2: tách chi phí MỖI lượt Mod+Enter (Rust vs webview), nhóm thay đổi DOM theo hàng/cột,
 * và đo độ trễ phím quanh lượt flush hẹn giờ của Editor. Nằm ngoài `e2e/specs/`.
 *
 *     AURA_BENCH_VIEW=parallel npm run test:e2e -- --spec e2e/measure/confirm-trace.e2e.mjs
 *
 * 🔴 IN số đo, không khẳng định số tuyệt đối. Bọc `__TAURI_INTERNALS__.invoke` từ phía bàn đo,
 * không sửa mã sản phẩm; điền bản dịch bằng `promoteAiTranslationBatchToEditor` thật.
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

const VIEW = process.env.AURA_BENCH_VIEW === 'plain' ? 'plain' : 'parallel'
const SENTENCES = Number(process.env.AURA_BENCH_SENTENCES ?? 320)
const CHECKPOINTS = (process.env.AURA_BENCH_CHECKPOINTS ?? '0,100,299').split(',').map(Number)
const SAMPLES = Number(process.env.AURA_BENCH_SAMPLES ?? 4)
const TYPE_MS = Number(process.env.AURA_BENCH_TYPE_MS ?? 15000)
const DO_FLUSH = process.env.AURA_BENCH_FLUSH !== '0'
const STATE = '/src/panels/editorPanelState.ts'

function sourceText() {
  const lines = []
  for (let i = 1; i <= SENTENCES; i += 1) {
    lines.push(`第${i}句：布拉姆斯的小提琴协奏曲是小提琴最受欢迎的演奏曲目之一。`)
  }
  return lines.join('')
}

async function install() {
  await browser.execute(() => {
    const B = (window.__bench = { log: [], recs: [], frames: [], rafOn: false })
    const ti = window.__TAURI_INTERNALS__
    const origRaw = ti.invoke
    const orig = origRaw.bind(ti)
    ti.invoke = (cmd, args, opts) => {
      const e = { cmd, req: performance.now(), resp: null }
      B.log.push(e)
      const p = orig(cmd, args, opts)
      p.then(
        () => { e.resp = performance.now() },
        () => { e.resp = performance.now(); e.err = true },
      )
      return p
    }
    B.invokeSticks = ti.invoke !== origRaw
    if (!B.invokeSticks) {
      const origFetch = window.fetch.bind(window)
      window.fetch = (input, init) => {
        const url = typeof input === 'string' ? input : input?.url ?? String(input)
        const m = /^(?:ipc:\/\/localhost|https?:\/\/ipc\.localhost)\/([^?#]+)/.exec(url)
        if (!m) return origFetch(input, init)
        const e = { cmd: m[1], req: performance.now(), resp: null }
        B.log.push(e)
        const p = origFetch(input, init)
        p.then(
          () => { e.resp = performance.now() },
          () => { e.resp = performance.now(); e.err = true },
        )
        return p
      }
    }
    const tgts = document.querySelectorAll('[data-col="tgt"]')
    let root = tgts[0].parentElement
    while (root && !root.contains(tgts[tgts.length - 1])) root = root.parentElement
    B.root = root
    B.obs = new MutationObserver((records) => {
      B.recs.push({ t: performance.now(), records })
    })
    B.obs.observe(root, { subtree: true, childList: true, attributes: true, characterData: true })
    const loop = () => {
      B.frames.push(performance.now())
      requestAnimationFrame(loop)
    }
    requestAnimationFrame(loop)
    B.summarize = (batches, confirmedBefore, currentId) => {
      const rows = { src: new Map(), tgt: new Map(), none: new Map() }
      let hv = 0, ruby = 0, added = 0, removed = 0, total = 0, attr = 0, chars = 0, child = 0
      const hvRows = new Set()
      const noneT = new Map()
      for (const b of batches) {
        for (const r of b.records) {
          total += 1
          if (r.type === 'attributes') attr += 1
          else if (r.type === 'characterData') chars += 1
          else child += 1
          added += r.addedNodes.length
          removed += r.removedNodes.length
          const el = r.target.nodeType === 1 ? r.target : r.target.parentElement
          const row = el?.closest('[data-segment-id]')
          const id = row?.getAttribute('data-segment-id') ?? null
          const col = row?.getAttribute('data-col') ?? 'none'
          if (!row) { const k = `${el?.tagName?.toLowerCase()}.${String(el?.className ?? '').slice(0, 40)}<${el?.parentElement?.className?.toString().slice(0, 30)}`; noneT.set(k, (noneT.get(k) ?? 0) + 1) }
          const key = id ?? '-'
          const m = rows[col] ?? rows.none
          m.set(key, (m.get(key) ?? 0) + 1)
          if (el?.closest('.hv-surface')) { hv += 1; if (id) hvRows.add(id) }
          if (el?.closest('ruby')) ruby += 1
        }
      }
      const cls = (m) => {
        let prevConfirmed = 0, current = 0, other = 0
        const exConfirmed = []
        for (const id of m.keys()) {
          if (id === '-') continue
          if (String(id) === String(currentId)) current += 1
          else if (confirmedBefore.has(String(id))) { prevConfirmed += 1; if (exConfirmed.length < 4) exConfirmed.push(id) }
          else other += 1
        }
        return { rows: m.size, prevConfirmed, current, other, exConfirmed, records: [...m.values()].reduce((a, b) => a + b, 0) }
      }
      return { total, attr, chars, child, added, removed, hv, ruby, hvRows: hvRows.size, src: cls(rows.src), tgt: cls(rows.tgt), none: cls(rows.none), noneT: Object.fromEntries([...noneT].slice(0, 6)) }
    }
  })
}

async function confirmedIds() {
  return browser.execute(async (statePath) => {
    const mod = await import(statePath)
    return mod.editorSegments.value.filter((s) => s.status === 'confirmed').map((s) => String(s.id))
  }, STATE)
}

async function typeKeys(text) {
  return browser.execute(async (chars) => {
    for (const ch of chars) {
      const target = document.activeElement
      target.dispatchEvent(new KeyboardEvent('keydown', { key: ch, code: `Key${ch.toUpperCase()}`, bubbles: true }))
      document.execCommand('insertText', false, ch)
      await new Promise((resolve) => requestAnimationFrame(resolve))
    }
  }, text)
}

async function tracedConfirm(confirmedBefore) {
  return browser.execute(async (before) => {
    const B = window.__bench
    const beforeSet = new Set(before)
    await new Promise((r) => setTimeout(r, 150))
    B.obs.takeRecords()
    B.recs.length = 0
    B.log.length = 0
    B.frames.length = 0
    const idOf = () => document.activeElement?.getAttribute?.('data-segment-id') ?? null
    const cur = idOf()
    const ticks = []
    const tick = setInterval(() => ticks.push(performance.now()), 1)
    const t0 = performance.now()
    document.activeElement.dispatchEvent(new KeyboardEvent('keydown', { code: 'Enter', key: 'Enter', metaKey: true, bubbles: true }))
    const syncMs = performance.now() - t0
    let tRow = null
    for (;;) {
      await new Promise((resolve) => requestAnimationFrame(resolve))
      if (tRow === null && idOf() !== cur) tRow = performance.now()
      if (tRow !== null && performance.now() - tRow > 250) break
      if (performance.now() - t0 > 20000) break
    }
    clearInterval(tick)
    const busy = []
    let prev = t0
    for (const t of ticks) { if (t - prev > 8) busy.push([+(prev - t0).toFixed(0), +(t - prev).toFixed(0)]); prev = t }
    const gaps = B.frames.map((f, i) => (i === 0 ? 0 : f - B.frames[i - 1])).filter((g) => g > 30).map((g) => +g.toFixed(0))
    const rest = B.obs.takeRecords()
    if (rest.length) B.recs.push({ t: performance.now(), records: rest })
    const confirmEntry = B.log.find((e) => e.cmd === 'confirm_segment')
    const nextFrame = confirmEntry?.resp != null ? B.frames.find((f) => f >= confirmEntry.resp) : null
    const sum = B.summarize(B.recs, beforeSet, cur)
    const rel = (x) => (x == null ? null : +(x - t0).toFixed(1))
    return {
      cur,
      syncMs: +syncMs.toFixed(1),
      busy,
      gaps,
      rowChangeMs: rel(tRow),
      confirmReq: rel(confirmEntry?.req),
      confirmResp: rel(confirmEntry?.resp),
      rustMs: confirmEntry?.resp != null ? +(confirmEntry.resp - confirmEntry.req).toFixed(1) : null,
      respToFrameMs: confirmEntry?.resp != null && nextFrame != null ? +(nextFrame - confirmEntry.resp).toFixed(1) : null,
      cmds: B.log.map((e) => `${e.cmd}@${rel(e.req)}..${rel(e.resp)}`),
      firstMutBatchMs: B.recs.length ? rel(B.recs[0].t) : null,
      batches: B.recs.map((b) => `${rel(b.t)}:${b.records.length}`),
      sum,
    }
  }, confirmedBefore)
}

async function typeTimed(ms) {
  return browser.execute(async (durationMs) => {
    const B = window.__bench
    B.obs.takeRecords()
    B.recs.length = 0
    B.log.length = 0
    const alphabet = 'abcdefghijklmnopqrstuvwxyz'
    const keys = []
    const start = performance.now()
    let i = 0
    while (performance.now() - start < durationMs) {
      const ch = alphabet[i % 26]
      i += 1
      const t0 = performance.now()
      document.activeElement.dispatchEvent(new KeyboardEvent('keydown', { key: ch, code: `Key${ch.toUpperCase()}`, bubbles: true }))
      document.execCommand('insertText', false, ch)
      await new Promise((resolve) => requestAnimationFrame(resolve))
      keys.push([+(t0 - start).toFixed(0), +(performance.now() - t0).toFixed(1)])
      const wait = t0 + 125 - performance.now()
      if (wait > 0) await new Promise((resolve) => setTimeout(resolve, wait))
    }
    await new Promise((resolve) => setTimeout(resolve, 300))
    const rest = B.obs.takeRecords()
    if (rest.length) B.recs.push({ t: performance.now(), records: rest })
    const flushes = B.log.filter((e) => e.cmd !== 'noop').map((e) => ({ cmd: e.cmd, req: +(e.req - start).toFixed(0), resp: e.resp == null ? null : +(e.resp - start).toFixed(0) }))
    const batches = B.recs.map((b) => ({ t: +(b.t - start).toFixed(0), n: b.records.length, s: B.summarize([b], new Set(), null) }))
    return { keys, flushes, batches: batches.map((b) => ({ t: b.t, n: b.n, src: b.s.src.rows, tgt: b.s.tgt.rows, hv: b.s.hv, ruby: b.s.ruby, hvRows: b.s.hvRows })) }
  }, ms)
}

function median(xs) {
  const s = [...xs].sort((a, b) => a - b)
  return s.length === 0 ? NaN : s[Math.floor(s.length / 2)]
}

function reportFlush(label, r) {
  const lat = r.keys.map((k) => k[1])
  const med = median(lat)
  console.log(`[đo vết] ${label}: ${r.keys.length} phím, trung vị ${med.toFixed(1)} ms, max ${Math.max(...lat).toFixed(1)}`)
  console.log(`[đo vết] ${label}: lời gọi IPC (ms từ đầu): ${JSON.stringify(r.flushes)}`)
  const spikes = r.keys.filter((k) => k[1] > Math.max(2 * med, 45))
  console.log(`[đo vết] ${label}: phím chậm (>max(2x trung vị,45)): ${JSON.stringify(spikes)}`)
  console.log(`[đo vết] ${label}: lô DOM {t,n,hàng src,hàng tgt,hv,ruby,hvRows} (chỉ lô > 5 bản ghi): ${JSON.stringify(r.batches.filter((b) => b.n > 5))}`)
  const flushResp = r.flushes.filter((f) => f.cmd === 'save_segment_targets' && f.resp != null).map((f) => f.resp)
  for (const fr of flushResp) {
    const near = r.keys.filter((k) => k[0] + k[1] >= fr - 150 && k[0] <= fr + 300)
    console.log(`[đo vết] ${label}: quanh phản hồi flush @${fr}: ${JSON.stringify(near)}`)
  }
}

describe(`Bàn đo vết Mod+Enter / flush — kiểu ${VIEW}`, function () {
  this.timeout(40 * 60_000)
  it('đo vết', async () => {
    await openWorkspaceWithWork(`do-vet-${VIEW}`, sourceText())
    await waitForGridRows(SENTENCES, { timeout: 60_000 })
    const filled = await browser.execute(async (statePath) => {
      const mod = await import(statePath)
      const items = mod.editorSegments.value.map((s) => ({ segmentId: s.id, text: `Bản dịch số ${s.id} của câu nguồn này.`, sourceHash: undefined }))
      let result = await mod.promoteAiTranslationBatchToEditor(items)
      if (result === 'needs-confirmation') {
        mod.confirmPendingPromote()
        const t0 = performance.now()
        while (mod.editorPendingPromoteBatch.value !== null && performance.now() - t0 < 120000) {
          await new Promise((resolve) => setTimeout(resolve, 100))
        }
        result = mod.editorPendingPromoteBatch.value === null ? 'done' : 'stuck'
      }
      return result
    }, STATE)
    expect(filled).toBe('done')
    if (VIEW === 'parallel') {
      await realClick(await $('#grid-tab-han-viet'))
      await browser.pause(500)
      await realClick(await $('.view-toggle'))
      await browser.pause(800)
    }
    console.log(`[đo vết] kiểu ${VIEW}: ${JSON.stringify(await browser.execute(() => ({ tgt: document.querySelectorAll('[data-col="tgt"]').length, src: document.querySelectorAll('[data-col="src"]').length, hv: document.querySelectorAll('.hv-surface').length, rt: document.querySelectorAll('rt').length, cores: navigator.hardwareConcurrency })))}`)
    await install()
    console.log(`[đo vết] invokeSticks=${await browser.execute(() => window.__bench?.invokeSticks)}`)
    await realClick(await $('[data-col="tgt"]'))
    await browser.pause(300)

    const last = Math.max(...CHECKPOINTS) + SAMPLES
    for (let n = 0; n < Math.min(last, SENTENCES - 2); n += 1) {
      const sampling = CHECKPOINTS.some((c) => n >= c && n < c + SAMPLES)
      await typeKeys('qwe')
      if (!sampling) {
        await browser.execute(async () => {
          const idOf = () => document.activeElement?.getAttribute?.('data-segment-id') ?? null
          const before = idOf()
          document.activeElement.dispatchEvent(new KeyboardEvent('keydown', { code: 'Enter', key: 'Enter', metaKey: true, bubbles: true }))
          const t0 = performance.now()
          while (idOf() === before && performance.now() - t0 < 20000) await new Promise((r) => requestAnimationFrame(r))
        })
        continue
      }
      const before = await confirmedIds()
      const r = await tracedConfirm(before)
      console.log(`[đo vết] A kiểu=${VIEW} n=${n} đã xác nhận trước=${before.length} · sync ${r.syncMs} ms · busy ${JSON.stringify(r.busy)} · rust ${r.rustMs} ms · phản hồi→khung ${r.respToFrameMs} ms · đổi hàng @${r.rowChangeMs} · lô DOM đầu @${r.firstMutBatchMs} · ${JSON.stringify(r.sum)} · lô ${JSON.stringify(r.batches)} · ${JSON.stringify(r.cmds)}`)
    }

    if (!DO_FLUSH) return
    const confirmed = await confirmedIds()
    console.log(`[đo vết] B bắt đầu: đã xác nhận ${confirmed.length}`)
    const unconfirmedId = await browser.execute(() => document.activeElement?.getAttribute('data-segment-id'))
    const r1 = await typeTimed(TYPE_MS)
    reportFlush(`B1 chưa xác nhận (hàng ${unconfirmedId}) kiểu=${VIEW}`, r1)
    const target = confirmed[Math.floor(confirmed.length / 2)]
    await realClick(await $(`[data-col="tgt"][data-segment-id="${target}"]`))
    await browser.pause(500)
    const r2 = await typeTimed(TYPE_MS)
    reportFlush(`B2 đã xác nhận (hàng ${target}) kiểu=${VIEW}`, r2)
  })
})
