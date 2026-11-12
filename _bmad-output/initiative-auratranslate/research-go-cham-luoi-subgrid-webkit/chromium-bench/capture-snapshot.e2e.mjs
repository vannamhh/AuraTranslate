/**
 * Chụp ảnh tĩnh lưới thật (Song song / Nguyên văn) để dựng lại trong Chrome.
 *   AURA_BENCH_ROWS=90 AURA_BENCH_OUT=<dir> npm run test:e2e -- --spec <đường dẫn tệp này>
 * Ra: <dir>/snap-<rows>-<view>.json  { html, probes, viewport }
 */
import fs from 'node:fs'
import { waitForGridRows } from '../../../../e2e/support/gridWait.mjs'
import { openWorkspaceWithWork } from '../../../../e2e/support/workspace.mjs'
import { realClick } from '../../../../e2e/support/pointer.mjs'

const ROWS = Number(process.env.AURA_BENCH_ROWS ?? 90)
const OUT = process.env.AURA_BENCH_OUT
const STATE = '/src/panels/editorPanelState.ts'

function sourceText() {
  const lines = []
  for (let i = 1; i <= ROWS; i += 1) {
    lines.push(`第${i}句：布拉姆斯的小提琴协奏曲是小提琴最受欢迎的演奏曲目之一。`)
  }
  return lines.join('')
}

async function snap(view) {
  const data = await browser.execute(() => {
    const sels = ['.hv-parallel', 'rt', '.cell-tgt', '.hv-unit', '.grid-block']
    const props = ['fontFamily', 'fontSize', 'lineHeight', 'rubyPosition', 'contain', 'display', 'whiteSpace', 'padding', 'textDecorationLine', 'userSelect']
    const probes = {}
    for (const s of sels) {
      const el = document.querySelector(s)
      if (!el) { probes[s] = null; continue }
      const cs = getComputedStyle(el)
      probes[s] = Object.fromEntries(props.map((p) => [p, cs[p]]))
    }
    const css = []
    for (const sh of document.styleSheets) {
      try { css.push([...sh.cssRules].map((r) => r.cssText).join('\n')) } catch { css.push('/*blocked*/') }
    }
    return {
      html: document.documentElement.outerHTML,
      cssRules: css,
      rootStyle: document.documentElement.getAttribute('style'),
      engine: document.documentElement.getAttribute('data-web-engine'),
      probes,
      viewport: [window.innerWidth, window.innerHeight, window.devicePixelRatio],
      counts: { rt: document.querySelectorAll('rt').length, tgt: document.querySelectorAll('[data-col="tgt"]').length,
        links: [...document.querySelectorAll('link[rel=stylesheet]')].map((l) => l.href) },
    }
  })
  fs.mkdirSync(OUT, { recursive: true })
  fs.writeFileSync(`${OUT}/snap-${ROWS}-${view}.json`, JSON.stringify(data))
  console.log(`[chụp] ${view}: ${data.html.length} bytes, ${JSON.stringify(data.counts)}, vp ${data.viewport}`)
}

describe('Chụp lưới', function () {
  this.timeout(20 * 60_000)
  it('chụp', async () => {
    await openWorkspaceWithWork(`chup-${ROWS}`, sourceText())
    await waitForGridRows(ROWS, { timeout: 60_000 })
    await browser.execute(
      async (statePath) => {
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
          while (mod.editorPendingPromoteBatch.value !== null && performance.now() - t0 < 120000) {
            await new Promise((res) => setTimeout(res, 100))
          }
        }
      },
      STATE,
    )
    await browser.pause(800)
    await snap('plain')
    await realClick((await $$('[data-col="tgt"]'))[Math.floor(ROWS / 2)])
    await browser.pause(800)
    await snap('plain-ed')
    await realClick(await $('#grid-tab-han-viet'))
    await browser.pause(1500)
    await snap('hv-tab')
    await realClick(await $('.view-toggle'))
    await browser.pause(1500)
    await snap('parallel')
    await realClick((await $$('[data-col="tgt"]'))[Math.floor(ROWS / 2)])
    await browser.pause(800)
    await snap('parallel-ed')
  })
})
