import { createRequire } from 'module'; import fs from 'fs';
const require = createRequire(import.meta.url);
const { chromium } = createRequire(process.env.PW_PATH + '/')('playwright');
const dir = process.cwd();
const css = (v) => `
@font-face{font-family:NotoTC;src:url(NotoSerifCJKtc-Regular.otf)}
body{margin:0;width:1200px;font:16px/1.95 ${v.font?'NotoTC':'serif'}}
.grid-block{display:grid;grid-template-columns:3px 30px minmax(0,1fr) minmax(0,1fr);align-items:stretch}
${v.layout==='col' ? `.col{display:grid;grid-template-rows:subgrid;grid-row:1/-1;min-width:0}
.c0{grid-column:1}.c1{grid-column:2}.c2{grid-column:3}.c3{grid-column:4}` : ''}
${v.layout==='row' ? `.row{display:grid;grid-template-columns:subgrid;grid-column:1/-1}` : ''}
.cell{padding:4px 8px;border-bottom:1px solid #ddd;min-width:0}
.cell-tgt{white-space:pre-wrap;${v.contain?'contain:layout style;':''}}
.hv-parallel{margin:0;white-space:pre-wrap;font-size:18px;line-height:${v.lhNormal?'normal':'2.05'}}
${v.under?'ruby{ruby-position:under}':''}
${v.pct?'.slot{height:100%;border-left:3px solid red}':''}
${v.inlineBlock?'.hv-unit{display:inline-block}':''}
rt{padding-inline:2px;padding-block:2px 4px;font-size:16px;line-height:1.95;font-family:serif}`;
const rnd = (s) => () => (s = (s*16807)%2147483647);
function srcCell(i, ruby, r) {
  let h = '<p class=hv-parallel>';
  for (let w=0; w<18; w++) { const ch = String.fromCharCode(0x4e00 + r()%3000) + String.fromCharCode(0x4e00 + r()%3000);
    h += ruby ? `<span class=hv-unit><ruby>${ch}<rt>ab cd</rt></ruby></span>` : ch; }
  return h + '</p>';
}
function page(v, n) {
  const r = rnd(42); const src = [], tgt = [];
  for (let i=0;i<n;i++){ src.push(srcCell(i, v.ruby, r)); tgt.push('Bản dịch câu số '+i+' đây là một câu tiếng Việt.'); }
  let body = '';
  if (v.layout==='col') {
    body = `<div class=grid-block style="grid-template-rows:repeat(${n},auto)">
<div class="col c0">${src.map(()=>v.pct?'<div class=cell><div class=slot></div></div>':'<div class=cell></div>').join('')}</div>
<div class="col c1">${src.map((_, i)=>`<div class=cell>${i+1}</div>`).join('')}</div>
<div class="col c2">${src.map(s=>v.wrap?`<div class=cell><div class=hv-surface>${s}</div></div>`:`<div class=cell>${s}</div>`).join('')}</div>
<div class="col c3">${tgt.map((t,i)=>`<div class="cell cell-tgt" contenteditable role=textbox aria-label="c${i}" id=t${i}>${t}</div>`).join('')}</div></div>`;
  } else {
    body = `<div class=grid-block>${src.map((s,i)=>`<div class=row><div class=cell></div><div class=cell>${i+1}</div><div class=cell>${s}</div><div class="cell cell-tgt" contenteditable role=textbox aria-label="c${i}" id=t${i}>${tgt[i]}</div></div>`).join('')}</div>`;
  }
  return `<!doctype html><meta charset=utf-8><style>${css(v)}</style>${body}`;
}
const base = { layout:'col', ruby:true, font:true, under:true, lhNormal:true, contain:true };
const variants = Object.entries(JSON.parse(process.argv[2] || '{"app":{}}'));
const sizes = (process.argv[3] || '20,200').split(',').map(Number);
const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH || undefined, headless: true });
for (const [name, over] of variants) for (const n of sizes) {
  const v = { ...base, ...over }; const f = `${dir}/p-${name}-${n}.html`; fs.writeFileSync(f, page(v, n));
  const p = await browser.newPage({ viewport: { width: 1200, height: 860 } });
  await p.goto('file://' + f); await p.evaluate(() => document.fonts.ready); await p.evaluate(() => document.body.offsetHeight);
  const id = '#t' + Math.floor(n/2); await p.focus(id);
  await p.evaluate(id => { const e = document.querySelector(id); const s = getSelection(); s.collapse(e.firstChild, e.firstChild.length); }, id);
  for (let i=0;i<5;i++) await p.evaluate(() => { document.execCommand('insertText', false, 'a'); document.body.offsetHeight; });
  const cdp = await p.context().newCDPSession(p); const ev = [];
  cdp.on('Tracing.dataCollected', e => ev.push(...e.value)); const done = new Promise(r => cdp.once('Tracing.tracingComplete', r));
  await cdp.send('Tracing.start', { traceConfig: { includedCategories: ['devtools.timeline','blink'] } });
  for (let i=0;i<30;i++) await p.evaluate(() => { document.execCommand('insertText', false, 'ê'); document.body.offsetHeight; });
  await cdp.send('Tracing.end'); await done;
  const lay = ev.filter(e => e.name==='Layout' && e.ph==='X'); const d = lay.map(e=>e.dur/1000).sort((a,b)=>a-b);
  const shape = ev.filter(e => e.name==='InlineNode::ShapeTextIncludingFirstLine').length;
  console.log(`${name.padEnd(12)} n=${String(n).padStart(4)} total=${lay[0]?.args.beginData.totalObjects} layouts=${lay.length} med=${d[d.length>>1].toFixed(3)} p90=${d[Math.floor(d.length*.9)].toFixed(3)} ms shapes/edit=${(shape/30).toFixed(1)}`);
  await p.close();
}
await browser.close();
