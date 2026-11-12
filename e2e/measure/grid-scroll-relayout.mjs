// Layout cost of one text edit in a synthetic Parallel grid nested in the dockview height chain, with and
// without `scrollbar-gutter: stable` on the scroll box. Run outside the sandbox:
//   PW_PATH=<dir containing node_modules/playwright> [CHROME_PATH=<chrome binary>] node e2e/measure/grid-scroll-relayout.mjs [runs=2] [edits=40]
import { createRequire } from 'node:module'
import path from 'node:path'
import fs from 'node:fs'
import os from 'node:os'
import { fileURLToPath } from 'node:url'

const require = createRequire(process.env.PW_PATH + '/')
const { chromium } = require('playwright')
const here = path.dirname(fileURLToPath(import.meta.url))
const FONT = path.resolve(here, '../../src-tauri/resources/fonts/NotoSerifCJKtc-Regular.otf')
const sfc = fs.readFileSync(path.resolve(here, '../../src/panels/GridPanel.vue'), 'utf8')
const productBlock = /^\.grid-scroll \{([^}]*)\}/m.exec(sfc)?.[1]
if (!productBlock) throw new Error('.grid-scroll block not found in GridPanel.vue')
const hasGutter = /scrollbar-gutter\s*:/.test(productBlock)
if (!hasGutter) console.log('WARNING: GridPanel.vue .grid-scroll has no scrollbar-gutter; the fixed case equals the unfixed case')
const stripVars = (css) => css.replace(/\/\*[\s\S]*?\*\//g, '').replace(/calc\([^;]*\)|var\([^;]*\)/g, '0px')
const GRID_SCROLL = stripVars(productBlock)
const GRID_SCROLL_UNFIXED = GRID_SCROLL.replace(/scrollbar-gutter\s*:[^;]*;?/g, '')
const [runs = 2, edits = 40] = process.argv.slice(2).map(Number)

const CSS = `
@font-face{font-family:NotoTC;src:url(file://${FONT})}
html,body{margin:0;height:100%}body{font:16px/1.95 NotoTC,serif}
::-webkit-scrollbar{width:10px;height:10px}::-webkit-scrollbar-thumb{background:#999}
.outer{height:725px;width:640px}
.gv{display:flex;flex-direction:column;height:100%;width:100%;overflow:hidden}
.cc{flex:1 1 auto;min-height:0}.vp{height:100%;width:100%}
.panel{position:relative;display:flex;flex-direction:column;min-width:0;height:100%;overflow:hidden;padding:14px 21px;box-sizing:border-box}
.panel-body{display:flex;flex-direction:column;flex:1 1 0%;min-height:0}
.grid-block{display:grid;grid-template-columns:3px 30px minmax(0,1fr) minmax(0,1fr);align-items:stretch}
.col{display:grid;grid-template-rows:subgrid;grid-row:1/-1;min-width:0}
.c0{grid-column:1}.c1{grid-column:2}.c2{grid-column:3}.c3{grid-column:4}
.cell{padding:4px 8px;border-bottom:1px solid #ddd;min-width:0}
.cell-tgt{white-space:pre-wrap;contain:layout style}
.hv-parallel{margin:0;white-space:pre-wrap;font-size:18px;line-height:normal}
ruby{ruby-position:under}
rt{padding-inline:2px;padding-block:2px 4px;font-size:16px;line-height:1.95;font-family:serif}`

function page(n, fixed) {
  let seed = 42
  const r = () => (seed = (seed * 16807) % 2147483647)
  const src = [], tgt = []
  for (let i = 0; i < n; i++) {
    let h = '<p class=hv-parallel>'
    for (let w = 0; w < 18; w++) {
      const ch = String.fromCharCode(0x4e00 + (r() % 3000)) + String.fromCharCode(0x4e00 + (r() % 3000))
      h += `<span><ruby>${ch}<rt>ab cd</rt></ruby></span>`
    }
    src.push(h + '</p>')
    tgt.push('Bản dịch câu số ' + i + ' đây là một câu tiếng Việt.')
  }
  const grid = `<div class=grid-block style="grid-template-rows:repeat(${n},auto)">
<div class="col c0">${src.map(() => '<div class=cell></div>').join('')}</div>
<div class="col c1">${src.map((_, i) => `<div class=cell>${i + 1}</div>`).join('')}</div>
<div class="col c2">${src.map((s) => `<div class=cell>${s}</div>`).join('')}</div>
<div class="col c3">${tgt.map((t) => `<div class="cell cell-tgt" contenteditable role=textbox>${t}</div>`).join('')}</div></div>`
  const extra = `.grid-scroll{${fixed ? GRID_SCROLL : GRID_SCROLL_UNFIXED}}`
  return `<!doctype html><meta charset=utf-8><style>${CSS}${extra}</style>
<div class=outer><div class=gv><div class=cc><div class=vp><section class=panel><div class=panel-body><div class=grid-scroll>${grid}</div></div></section></div></div></div></div>`
}

const med = (a) => [...a].sort((x, y) => x - y)[a.length >> 1]
const p90 = (a) => [...a].sort((x, y) => x - y)[Math.min(a.length - 1, Math.floor(a.length * 0.9))]

async function measure(browser, file) {
  const p = await browser.newPage({ viewport: { width: 1280, height: 860 } })
  await p.goto('file://' + file)
  await p.evaluate(() => Promise.all([...document.fonts].map((f) => f.load().catch(() => {}))))
  const hasBar = await p.evaluate(() => {
    const g = document.querySelector('.grid-scroll')
    const e = [...document.querySelectorAll('.cell-tgt')]
    const t = e[e.length >> 1]
    t.focus()
    const s = getSelection()
    s.collapse(t.lastChild, t.lastChild.length)
    return g.offsetWidth - g.clientWidth > 0
  })
  const edit = () => p.evaluate(() => new Promise((res) => {
    document.execCommand('insertText', false, 'ê')
    document.body.offsetHeight
    requestAnimationFrame(() => res())
  }))
  for (let i = 0; i < 5; i++) await edit()
  const cdp = await p.context().newCDPSession(p)
  const ev = []
  cdp.on('Tracing.dataCollected', (e) => ev.push(...e.value))
  const done = new Promise((r) => cdp.once('Tracing.tracingComplete', r))
  await cdp.send('Tracing.start', { traceConfig: { includedCategories: ['devtools.timeline', 'blink'] } })
  for (let i = 0; i < edits; i++) await edit()
  await cdp.send('Tracing.end')
  await done
  await p.close()
  const lay = ev.filter((e) => e.name === 'Layout' && e.ph === 'X').map((e) => e.dur / 1000)
  return { hasBar, layouts: lay.length, med: med(lay), p90: p90(lay) }
}

const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'grid-scroll-relayout-'))
const browser = await chromium.launch({
  headless: true,
  executablePath: process.env.CHROME_PATH || undefined,
  ignoreDefaultArgs: ['--hide-scrollbars'],
})
console.log('chrome', browser.version())
for (const n of [90, 200]) {
  for (const fixed of [false, true]) {
    const f = path.join(dir, `grid-${n}-${fixed}.html`)
    fs.writeFileSync(f, page(n, fixed))
    for (let r = 1; r <= runs; r++) {
      const m = await measure(browser, f)
      console.log(`rows=${n} ${fixed ? 'product(GridPanel.vue)' : 'gutter stripped'} run${r} scrollbar=${m.hasBar} layouts=${m.layouts} median=${m.med.toFixed(2)} p90=${m.p90.toFixed(2)} ms`)
    }
  }
}
await browser.close()
fs.rmSync(dir, { recursive: true, force: true })
