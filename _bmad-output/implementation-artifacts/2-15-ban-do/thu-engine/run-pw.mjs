import { createRequire } from 'node:module'
import { pathToFileURL } from 'node:url'
import path from 'node:path'
const require = createRequire(process.env.PW_PATH + '/')
const pw = require('playwright')
const page = pathToFileURL(path.join(path.dirname(new URL(import.meta.url).pathname), 'grid.html')).href
const n = process.env.N || '9850'
for (const engine of (process.env.ENGINES || 'webkit,chromium').split(',')) {
  const browser = await pw[engine].launch({ headless: true })
  for (const v of (process.env.VARIANTS || 'subgrid,chunk,rows').split(',')) {
    const p = await browser.newPage({ viewport: { width: 1200, height: 900 } })
    await p.goto(`${page}?v=${v}&n=${n}`)
    const r = await p.evaluate(() => window.__run())
    console.log(JSON.stringify({ engine, version: browser.version(), ...r, ua: undefined }))
    await p.close()
  }
  await browser.close()
}
