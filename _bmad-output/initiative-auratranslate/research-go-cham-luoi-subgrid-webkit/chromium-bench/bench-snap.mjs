import os from 'os'
// Đo chi phí MỖI lần sửa chữ trong ô bản dịch trên ảnh tĩnh của lưới thật (xem handoff-b.md).
//   node bench-snap.mjs <snap.json> <variant[,variant...]> [runs=2] [edits=40]
// Biến thể ở VARIANTS dưới đây; "base" = không đổi gì ngoài data-web-engine=chromium + bỏ script.
// Không commit font: @font-face trỏ thẳng src-tauri/resources/fonts.
import { createRequire } from 'module'
import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
const require = createRequire(import.meta.url)
const { chromium } = createRequire(process.env.PW_PATH + '/')('playwright')
const here = path.dirname(fileURLToPath(import.meta.url))
const FONTS = path.resolve(here, '../../../../src-tauri/resources/fonts')
const TMP = process.env.AURA_BENCH_TMP ?? path.join(os.tmpdir(), 'aura-bench-pages')
fs.mkdirSync(TMP, { recursive: true })

const fontCss = `
@font-face{font-family:"Source Serif 4";src:url("file://${FONTS}/SourceSerif4[opsz,wght].ttf");font-weight:200 900;font-style:normal}
@font-face{font-family:"Source Serif 4";src:url("file://${FONTS}/SourceSerif4-Italic[opsz,wght].ttf");font-weight:200 900;font-style:italic}
@font-face{font-family:"Source Sans 3";src:url("file://${FONTS}/SourceSans3[wght].ttf");font-weight:200 900}
@font-face{font-family:"Noto Serif CJK TC";src:url("file://${FONTS}/NotoSerifCJKtc-Regular.otf");font-weight:400}`

const SYL = ['bố', 'lạp', 'mỗ', 'tư', 'đích', 'tiểu', 'đề', 'cầm', 'hiệp', 'tấu', 'khúc', 'thị', 'tối', 'thụ', 'hoan', 'nghênh']
// css: nối vào cuối <head>; html: (s)=>s; js: chạy trong trang trước khi đo
export const VARIANTS = {
  base: {},
  syl: { // thay "·" bằng âm Hán Việt thật cho giống app thật
    html: (s) => { let i = 0; return s.replace(/<rt data-v-b99d2a20="">·<\/rt>/g, () => `<rt data-v-b99d2a20="">${SYL[i++ % SYL.length]}</rt>`) },
  },
  ...(await loadExtra()),
}
async function loadExtra() {
  const f = path.join(here, 'variants.mjs')
  return fs.existsSync(f) ? (await import(f)).default : {}
}

function build(snapFile, vnames) {
  const d = JSON.parse(fs.readFileSync(snapFile, 'utf8'))
  let h = d.html.replace(/<script[\s\S]*?<\/script>/g, '')
  h = h.replace('<html ', '<html data-web-engine="chromium" ')
  let extraCss = '', js = ''
  for (const n of vnames) {
    const v = VARIANTS[n]; if (!v) throw new Error('biến thể lạ: ' + n)
    if (v.html) h = v.html(h)
    if (v.css) extraCss += v.css + '\n'
    if (v.js) js += v.js + '\n'
  }
  h = h.replace('<head>', `<head><meta charset="utf-8"><style id="bench-fonts">${fontCss}</style>`)
  h = h.replace('</head>', `<style id="bench-extra">${extraCss}</style></head>`)
  return { html: h, js, vp: d.viewport }
}

const med = (a) => { const s = [...a].sort((x, y) => x - y); return s[s.length >> 1] }
const p90 = (a) => { const s = [...a].sort((x, y) => x - y); return s[Math.min(s.length - 1, Math.floor(s.length * 0.9))] }

export async function measure(browser, htmlFile, js, vp, edits = 40, targetSel) {
  const p = await browser.newPage({ viewport: { width: vp[0], height: vp[1] }, deviceScaleFactor: Number(process.env.AURA_BENCH_DPR ?? 1) })
  await p.goto('file://' + htmlFile)
  await p.evaluate(() => document.fonts.ready)
  await p.evaluate(() => Promise.all([...document.fonts].map((f) => f.load().catch(() => {}))))
  if (js) await p.evaluate(js)
  await p.evaluate(() => document.body.offsetHeight)
  const info = await p.evaluate((sel) => {
    const cells = [...document.querySelectorAll(sel ?? '.cell-tgt')]
    const e = cells[cells.length >> 1]
    e.id = 'bench-target'
    e.focus()
    const s = getSelection(); s.collapse(e.lastChild ?? e, (e.lastChild?.length) ?? 0)
    return { cells: cells.length, rt: document.querySelectorAll('rt').length, h: document.body.scrollHeight }
  }, targetSel)
  const MODE = process.env.AURA_BENCH_INPUT ?? 'exec' // exec | key | cdp | evkey
  const cdpIn = await p.context().newCDPSession(p)
  const edit = async (ch) => {
    if (MODE === 'exec') return p.evaluate((c) => new Promise((res) => {
      const t0 = performance.now(); document.execCommand('insertText', false, c); document.body.offsetHeight
      const ms = performance.now() - t0; requestAnimationFrame(() => res(ms))
    }), ch)
    const t0 = Date.now()
    if (MODE === 'key') await p.keyboard.press(ch === 'ê' ? 'e' : 'a')
    else if (MODE === 'cdp') await cdpIn.send('Input.insertText', { text: ch })
    else { await p.keyboard.press('Backspace'); await p.keyboard.press('e'); await cdpIn.send('Input.insertText', { text: 'ê' }) }
    await p.evaluate(() => new Promise((r) => requestAnimationFrame(() => r())))
    return Date.now() - t0
  }
  for (let i = 0; i < 5; i++) await edit('a')
  const cdp = await p.context().newCDPSession(p); const ev = []
  cdp.on('Tracing.dataCollected', (e) => ev.push(...e.value)); const done = new Promise((r) => cdp.once('Tracing.tracingComplete', r))
  await cdp.send('Tracing.start', { traceConfig: { includedCategories: ['devtools.timeline', 'blink'] } })
  const wall = []
  for (let i = 0; i < edits; i++) wall.push(await edit('ê'))
  await cdp.send('Tracing.end'); await done
  const layEv = ev.filter((e) => e.name === 'Layout' && e.ph === 'X')
  const lay = layEv.map((e) => e.dur / 1000)
  const b0 = layEv[layEv.length >> 1]?.args ?? {}
  const lroots = JSON.stringify((b0.endData?.layoutRoots ?? []).map((r) => r.name ?? r.nodeName ?? Object.keys(r).join('/')))
  await p.close()
  return { ...info, total: b0.beginData?.totalObjects, dirty: b0.beginData?.dirtyObjects, partial: b0.beginData?.partialLayout, lroots, wallMed: med(wall), wallP90: p90(wall), layouts: lay.length, layMed: lay.length ? med(lay) : NaN, layP90: lay.length ? p90(lay) : NaN }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [snap, vlist = 'base', runs = '2', edits = '40'] = process.argv.slice(2)
  const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH || undefined, headless: true, ignoreDefaultArgs: process.env.AURA_BENCH_SCROLLBARS ? ["--hide-scrollbars"] : [], args: (process.env.AURA_BENCH_ARGS ?? "").split(" ").filter(Boolean) })
  console.log('chrome', browser.version())
  for (const vs of vlist.split(';')) {
    const names = vs.split(',')
    const { html, js, vp } = build(snap, names)
    const f = path.join(TMP, `${path.basename(snap, '.json')}__${names.join('+')}.html`); fs.writeFileSync(f, html)
    for (let r = 1; r <= Number(runs); r++) {
      const m = await measure(browser, f, js, vp, Number(edits))
      console.log(`${path.basename(snap, '.json').padEnd(20)} ${names.join('+').padEnd(28)} run${r} cells=${m.cells} rt=${m.rt} wall med=${m.wallMed.toFixed(2)} p90=${m.wallP90.toFixed(2)} | tot=${m.total} dirty=${m.dirty} part=${m.partial} roots=${m.lroots} | Layout n=${m.layouts} med=${m.layMed.toFixed(2)} p90=${m.layP90.toFixed(2)} ms`)
    }
  }
  await browser.close()
}
