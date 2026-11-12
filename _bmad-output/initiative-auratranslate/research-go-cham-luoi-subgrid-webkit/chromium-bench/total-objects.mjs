import { createRequire } from 'module';
const require = createRequire(import.meta.url);
const { chromium } = createRequire(process.env.PW_PATH + '/')('playwright');
const browser = await chromium.launch({ executablePath: process.env.CHROME_PATH || undefined, headless: true });
for (const n of [500, 5000, 50000]) {
  const page = await browser.newPage();
  await page.setContent(`<div id=big></div><div id=ed contenteditable style="width:300px">abc</div>`);
  await page.evaluate(n => { const b = document.getElementById('big'); for (let i=0;i<n;i++){const d=document.createElement('div');d.textContent='row '+i;b.appendChild(d);} }, n);
  await page.evaluate(() => document.body.offsetHeight);
  const cdp = await page.context().newCDPSession(page);
  const events = [];
  cdp.on('Tracing.dataCollected', e => events.push(...e.value));
  const done = new Promise(r => cdp.once('Tracing.tracingComplete', r));
  await cdp.send('Tracing.start', { traceConfig: { includedCategories: ['devtools.timeline', 'disabled-by-default-devtools.timeline'] } });
  await page.focus('#ed');
  for (let i=0;i<20;i++) { await page.evaluate(() => { document.execCommand('insertText', false, 'x'); document.body.offsetHeight; }); }
  await cdp.send('Tracing.end'); await done;
  const lay = events.filter(e => e.name === 'Layout' && e.ph === 'X');
  const tot = lay.map(e => e.args?.beginData?.totalObjects), dirty = lay.map(e => e.args?.beginData?.dirtyObjects), part = lay.map(e=>e.args?.beginData?.partialLayout);
  const durs = lay.map(e => e.dur/1000).sort((a,b)=>a-b);
  console.log(`n=${n} layouts=${lay.length} totalObjects=${[...new Set(tot)].slice(0,4)} dirty=${[...new Set(dirty)].slice(0,4)} partial=${[...new Set(part)]} dur med=${durs[durs.length>>1]?.toFixed(3)}ms max=${durs.at(-1)?.toFixed(3)}`);
  await page.close();
}
await browser.close();
