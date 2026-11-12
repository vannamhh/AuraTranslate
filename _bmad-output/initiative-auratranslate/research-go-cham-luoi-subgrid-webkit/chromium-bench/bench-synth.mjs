import os from 'os'
// Trang tổng hợp do-tren-windows.html (+ biến thể bọc trong cuộn riêng) đo bằng cùng hàm measure().
//   AURA_BENCH_SCROLLBARS=1 node bench-synth.mjs [runs=2]
import { createRequire } from 'module'
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import { measure } from './bench-snap.mjs'
const require = createRequire(import.meta.url)
const { chromium } = createRequire(process.env.PW_PATH + '/')('playwright')
const here = path.dirname(fileURLToPath(import.meta.url))
const TMP = process.env.AURA_BENCH_TMP ?? path.join(os.tmpdir(), 'aura-bench-pages')
const FONT = path.resolve(here, '../../../../src-tauri/resources/fonts/NotoSerifCJKtc-Regular.otf')
const SB = '::-webkit-scrollbar{width:10px;height:10px}::-webkit-scrollbar-thumb{background:#999}'
const WRAP = '#bar{position:static}#grid{height:725px;overflow:auto}.grid-block{width:auto}'
const NEST = '#bar{position:static}.outer{height:725px;width:640px}.gv{display:flex;flex-direction:column;height:100%;width:100%;overflow:hidden}.cc{flex:1 1 auto;min-height:0}.vp{height:100%;width:100%}.panel{position:relative;display:flex;flex-direction:column;min-width:0;height:100%;overflow:hidden;padding:14px 21px;box-sizing:border-box}.panel-body{display:flex;flex-direction:column;flex:1 1 0%;min-height:0}#grid{flex:1;min-height:0;overflow:auto}.grid-block{width:auto}'
const VAR = {
  'viewport-scroller (như do-tren-windows)': '',
  'outer>panel(h:100%)>body>#grid flex:1, auto, thanh 10px': NEST + SB,
  'như trên nhưng flex:1 1 0px': NEST + SB + '#grid{flex:1 1 0px}',
  'như trên nhưng overflow:scroll': NEST + SB + '#grid{overflow:scroll}',
  'như trên nhưng scrollbar-gutter:stable': NEST + SB + '#grid{scrollbar-gutter:stable}',
  'như trên nhưng panel height:px cố định': NEST + SB + '.panel{height:725px}',
  'như trên nhưng .vp height:px cố định': NEST + SB + '.vp{height:725px}',
  'như trên nhưng .cc flex:none;height:100%': NEST + SB + '.cc{flex:none;height:100%}',
  'như trên nhưng thanh ẩn': NEST + '::-webkit-scrollbar{display:none}',
  'như trên nhưng panel-body height cố định': NEST + SB + '.panel-body{flex:none;height:725px}',
}
const src = fs.readFileSync(path.join(here, 'do-tren-windows.html'), 'utf8').replace('url(NotoSerifCJKtc-Regular.otf)', `url(file://${FONT})`)
const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH || undefined, headless: true, ignoreDefaultArgs: process.env.AURA_BENCH_SCROLLBARS ? ['--hide-scrollbars'] : [] })
console.log('chrome', browser.version())
for (const n of [90, 200]) for (const [name, css] of Object.entries(VAR)) {
  const f = path.join(TMP, `synth-${n}-${Object.keys(VAR).indexOf(name)}.html`)
  fs.writeFileSync(f, src.replace('</style>', css + '</style>').replace('<div id="grid"></div>', css.includes('.panel{') ? '<div class="outer"><div class="gv"><div class="cc"><div class="vp"><section class="panel"><div class="panel-body"><div id="grid"></div></div></section></div></div></div></div>' : '<div id="grid"></div>'))
  for (let r = 1; r <= Number(process.argv[2] ?? 2); r++) {
    const m = await measure(browser, f + `?v=ruby&n=${n}`, '', [1280, 860], 40)
    console.log(`synth n=${n} ${name.padEnd(42)} run${r} tot=${m.total} dirty=${m.dirty} | wall med=${m.wallMed.toFixed(2)} | Layout n=${m.layouts} med=${m.layMed.toFixed(2)} p90=${m.layP90.toFixed(2)} ms`)
  }
}
await browser.close()
