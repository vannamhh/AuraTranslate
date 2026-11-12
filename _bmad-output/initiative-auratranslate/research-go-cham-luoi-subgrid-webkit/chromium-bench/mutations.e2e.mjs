/**
 * Ghi MOI dot bien DOM do MOT lan sua chu trong o ban dich (Nguyen van va Song song).
 *   AURA_BENCH_ROWS=90 AURA_BENCH_OUT=<dir> npx wdio run e2e/wdio.conf.mjs --spec <tep nay>
 * Ra <dir>/mut-<rows>-<view>.json. Chi DOC; dien ban dich bang ham that cua san pham.
 */
import fs from 'node:fs'
import { waitForGridRows } from '../../../../e2e/support/gridWait.mjs'
import { openWorkspaceWithWork } from '../../../../e2e/support/workspace.mjs'
import { realClick } from '../../../../e2e/support/pointer.mjs'

const ROWS = Number(process.env.AURA_BENCH_ROWS ?? 90)
const OUT = process.env.AURA_BENCH_OUT
const STATE = '/src/panels/editorPanelState.ts'
const src = () =>
  Array.from({ length: ROWS }, (_, i) => `第${i + 1}句：布拉姆斯的小提琴协奏曲是小提琴最受欢迎的演奏曲目之一。`).join('')

async function probe(view) {
  await realClick((await $$('[data-col="tgt"]'))[Math.floor(ROWS / 2)])
  await browser.pause(400)
  const res = await browser.execute(async () => {
    const out = []
    const desc = (n) => {
      if (!n) return 'null'
      if (n.nodeType === 3) return '#text<' + desc(n.parentElement) + '>'
      if (n.nodeType === 8) return '#comment'
      return (
        n.tagName.toLowerCase() +
        (n.id ? '#' + n.id : '') +
        [...(n.classList ?? [])].slice(0, 4).map((c) => '.' + c).join('') +
        (n.dataset?.segmentId ? `[seg=${n.dataset.segmentId}]` : '')
      )
    }
    for (let k = 0; k < 3; k++) {
      const recs = []
      const mo = new MutationObserver((l) => recs.push(...l))
      mo.observe(document, { subtree: true, attributes: true, attributeOldValue: true, childList: true, characterData: true })
      const t = document.activeElement
      t.dispatchEvent(new KeyboardEvent('keydown', { key: 'ê', bubbles: true }))
      document.execCommand('insertText', false, 'ê')
      await new Promise((r) => requestAnimationFrame(r))
      await new Promise((r) => setTimeout(r, 1200))
      recs.push(...mo.takeRecords())
      mo.disconnect()
      out.push(
        recs.map((r) => ({
          type: r.type,
          target: desc(r.target),
          attr: r.attributeName,
          old: r.attributeOldValue?.slice(0, 80),
          now: r.attributeName ? String(r.target.getAttribute(r.attributeName)).slice(0, 80) : undefined,
          added: r.addedNodes.length,
          removed: r.removedNodes.length,
        })),
      )
    }
    return out
  })
  fs.mkdirSync(OUT, { recursive: true })
  fs.writeFileSync(`${OUT}/mut-${ROWS}-${view}.json`, JSON.stringify(res, null, 1))
  console.log(`[dot bien] ${view}: ${res.map((r) => r.length).join(',')} ban ghi / lan sua`)
}

describe('Dot bien DOM moi lan sua', function () {
  this.timeout(20 * 60_000)
  it('ghi', async () => {
    await openWorkspaceWithWork(`mut-${ROWS}`, src())
    await waitForGridRows(ROWS, { timeout: 60_000 })
    await browser.execute(async (statePath) => {
      const mod = await import(statePath)
      const items = mod.editorSegments.value.map((s) => ({
        segmentId: s.id,
        text: `Bản dịch số ${s.id} của câu nguồn này.`,
        sourceHash: undefined,
      }))
      const r = await mod.promoteAiTranslationBatchToEditor(items)
      if (r === 'needs-confirmation') {
        mod.confirmPendingPromote()
        const t0 = performance.now()
        while (mod.editorPendingPromoteBatch.value !== null && performance.now() - t0 < 120000)
          await new Promise((res) => setTimeout(res, 100))
      }
    }, STATE)
    await browser.pause(800)
    await probe('plain')
    await realClick(await $('#grid-tab-han-viet'))
    await browser.pause(1500)
    await realClick(await $('.view-toggle'))
    await browser.pause(1500)
    await probe('parallel')
  })
})
