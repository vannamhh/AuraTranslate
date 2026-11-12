/**
 * Bàn đo: độ trễ phím gõ theo SỐ CÂU ĐÃ XÁC NHẬN (Mod+Enter), kiểu Song song Hán Việt so với
 * kiểu thường. Triệu chứng của Ice: gõ chậm dần sau hơn 100 lượt xác nhận, chỉ ở Song song.
 *
 * Nằm NGOÀI `e2e/specs/` nên nightly không gom nó. Mỗi lượt chạy một kiểu xem (một cửa sổ,
 * một tiến trình app mới):
 *
 *     AURA_BENCH_VIEW=parallel npm run test:e2e -- --spec e2e/measure/confirm-typing-parallel.e2e.mjs
 *     AURA_BENCH_VIEW=plain    npm run test:e2e -- --spec e2e/measure/confirm-typing-parallel.e2e.mjs
 *
 * 🔴 IN số đo, chỉ khẳng định CẤU TRÚC (`e2e/AGENTS.md`). Bản dịch được đưa vào Editor bằng
 * chính `promoteAiTranslationBatchToEditor` của sản phẩm (đường "AI dịch → đưa sang Editor"),
 * chỉ thiếu lượt gọi mạng. Phím gõ là phím thật của driver; `Mod+Enter` là sự kiện tổng hợp
 * vì `browser.keys` đánh rơi `Meta` ở `Enter` (xem `specs/editor-confirm-segment.e2e.mjs`).
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

const VIEW = process.env.AURA_BENCH_VIEW === 'plain' ? 'plain' : 'parallel'
const SENTENCES = Number(process.env.AURA_BENCH_SENTENCES ?? 320)
const CHECKPOINTS = (process.env.AURA_BENCH_CHECKPOINTS ?? '0,25,50,100,150,200,250')
  .split(',')
  .map(Number)
  .filter((n) => n < SENTENCES)
const PROBE_KEYS = 20
const WALK_KEYS = 'qwe'
const STATE = '/src/panels/editorPanelState.ts'

function sourceText() {
  const lines = []
  for (let i = 1; i <= SENTENCES; i += 1) {
    lines.push(`第${i}句：布拉姆斯的小提琴协奏曲是小提琴最受欢迎的演奏曲目之一。`)
  }
  return lines.join('')
}

function median(xs) {
  const s = [...xs].sort((a, b) => a - b)
  return s.length === 0 ? NaN : s[Math.floor(s.length / 2)]
}
function p90(xs) {
  const s = [...xs].sort((a, b) => a - b)
  return s.length === 0 ? NaN : s[Math.min(s.length - 1, Math.floor(s.length * 0.9))]
}
const fmt = (n) => (Number.isNaN(n) ? 'n/a' : n.toFixed(1))

/**
 * `browser.keys()` chỉ phát `keydown`, không `beforeinput`/`input` (xem
 * `specs/editor-typing-flush.e2e.mjs`), nên mỗi phím ở đây là: `keydown` tổng hợp (cho các
 * handler phím của sản phẩm) rồi `execCommand('insertText')` (cùng đường `beforeinput` → `input`
 * mà phím thật sinh ra). Độ trễ = từ lúc bắt đầu `keydown` tới khung hình kế sau `input`.
 */
async function typeKeysInPage(text) {
  return browser.execute(async (chars) => {
    const out = []
    for (const ch of chars) {
      const target = document.activeElement
      const t0 = performance.now()
      target.dispatchEvent(
        new KeyboardEvent('keydown', { key: ch, code: `Key${ch.toUpperCase()}`, bubbles: true }),
      )
      const ok = document.execCommand('insertText', false, ch)
      await new Promise((resolve) => requestAnimationFrame(resolve))
      out.push(ok ? performance.now() - t0 : -1)
    }
    return out
  }, text)
}

async function confirmAndWaitForNextRow() {
  return browser.execute(async () => {
    const idOf = () => document.activeElement?.getAttribute?.('data-segment-id') ?? null
    const before = idOf()
    const t0 = performance.now()
    document.activeElement.dispatchEvent(
      new KeyboardEvent('keydown', { code: 'Enter', key: 'Enter', metaKey: true, bubbles: true }),
    )
    for (;;) {
      await new Promise((resolve) => requestAnimationFrame(resolve))
      const now = idOf()
      if (now !== null && now !== before) {
        return { ms: performance.now() - t0, before, after: now }
      }
      if (performance.now() - t0 > 20_000) return { ms: -1, before, after: now }
    }
  })
}

async function probeKeys() {
  const textOf = () => browser.execute(() => document.activeElement?.textContent?.length ?? -1)
  const len0 = await textOf()
  const w0 = Date.now()
  const keys = await typeKeysInPage('abcdefghijklmnopqrst'.slice(0, PROBE_KEYS))
  const wallMs = Date.now() - w0
  const landed = (await textOf()) - len0
  return { keys: keys.filter((k) => k >= 0), wallMs, landed }
}

describe(`Bàn đo gõ phím theo số câu đã xác nhận — kiểu ${VIEW}`, function () {
  this.timeout(40 * 60_000)
  it('đo độ trễ phím và Mod+Enter ở các mốc', async () => {
    await openWorkspaceWithWork(`do-gõ-${VIEW}`, sourceText())
    await waitForGridRows(SENTENCES, { timeout: 60_000 })

    const filled = await browser.execute(
      async (statePath) => {
        const mod = await import(statePath)
        const items = mod.editorSegments.value.map((s) => ({
          segmentId: s.id,
          text: `Bản dịch số ${s.id} của câu nguồn này.`,
          sourceHash: undefined,
        }))
        let result = await mod.promoteAiTranslationBatchToEditor(items)
        const asked = mod.editorPendingPromoteBatch.value?.items.length ?? 0
        if (result === 'needs-confirmation') {
          mod.confirmPendingPromote()
          const t0 = performance.now()
          while (mod.editorPendingPromoteBatch.value !== null && performance.now() - t0 < 120000) {
            await new Promise((resolve) => setTimeout(resolve, 100))
          }
          result = mod.editorPendingPromoteBatch.value === null ? 'done' : 'stuck'
        }
        return { result, asked, count: items.length, report: mod.editorPromoteBatchReport.value }
      },
      STATE,
    )
    console.log(`[đo gõ] fill: ${JSON.stringify(filled)}`)
    expect(filled.result).toBe('done')

    if (VIEW === 'parallel') {
      await realClick(await $('#grid-tab-han-viet'))
      await browser.pause(500)
      await realClick(await $('.view-toggle'))
      await browser.pause(800)
    }

    const view = await browser.execute(() => ({
      tgtCells: document.querySelectorAll('[data-col="tgt"]').length,
      hvSurfaces: document.querySelectorAll('.hv-surface').length,
      rt: document.querySelectorAll('rt').length,
    }))
    console.log(`[đo gõ] kiểu ${VIEW}: ${JSON.stringify(view)}`)

    await realClick(await $('[data-col="tgt"]'))
    await browser.pause(300)

    const confirmMs = []
    const rows = []
    for (let n = 0; n < SENTENCES - 1; n += 1) {
      const isCheckpoint = CHECKPOINTS.includes(n)
      let probe = null
      if (isCheckpoint) probe = await probeKeys()
      await typeKeysInPage(WALK_KEYS)
      const c = await confirmAndWaitForNextRow()
      if (c.ms < 0) {
        console.log(`[đo gõ] dừng: Mod+Enter không đưa con trỏ sang hàng kế ở n=${n}`)
        break
      }
      confirmMs.push(c.ms)
      if (isCheckpoint) {
        const confirmed = await browser.execute(
          async (statePath) => {
            const mod = await import(statePath)
            return mod.editorSegments.value.filter((s) => s.status === 'confirmed').length
          },
          STATE,
        )
        const recent = confirmMs.slice(-10)
        const row = {
          confirmedTarget: n,
          confirmedActual: confirmed,
          keyMedian: median(probe.keys),
          keyP90: p90(probe.keys),
          keyMax: Math.max(...probe.keys),
          keysRecorded: probe.keys.length,
          batchWallMs: probe.wallMs,
          confirmMs: c.ms,
          confirmMedianLast10: median(recent),
        }
        rows.push(row)
        const load = await browser.execute(() => navigator.hardwareConcurrency)
        console.log(
          `[đo gõ] kiểu=${VIEW} câu=${SENTENCES} đã xác nhận=${n} (thực ${confirmed}) · ` +
            `phím ms trung vị ${fmt(row.keyMedian)} p90 ${fmt(row.keyP90)} max ${fmt(row.keyMax)} ` +
            `(${row.keysRecorded}/${PROBE_KEYS} phím vào chữ) · lô ${PROBE_KEYS} phím ${row.batchWallMs} ms · ` +
            `Mod+Enter ${fmt(c.ms)} ms (trung vị 10 lượt gần nhất ${fmt(row.confirmMedianLast10)}) · cores ${load}`,
        )
      }
    }
    console.log(`[đo gõ] JSON ${VIEW}: ${JSON.stringify(rows)}`)

    expect(rows.length).toBeGreaterThan(0)
    expect(rows[0].keysRecorded).toBeGreaterThan(0)
  })
})
