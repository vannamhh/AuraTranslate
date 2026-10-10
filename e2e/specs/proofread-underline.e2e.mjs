import { mkdtempSync, readFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { inflateSync } from 'node:zlib'
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

const TYPED = 'Tôi đi họcc ở trường lớp.'
const QUOTE = 'họcc'

function decodePng(path) {
  const buf = readFileSync(path)
  let pos = 8
  let width = 0
  let height = 0
  let colorType = 0
  const idat = []
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos)
    const type = buf.toString('ascii', pos + 4, pos + 8)
    const data = buf.subarray(pos + 8, pos + 8 + len)
    if (type === 'IHDR') {
      width = data.readUInt32BE(0)
      height = data.readUInt32BE(4)
      colorType = data[9]
    } else if (type === 'IDAT') idat.push(data)
    pos += 12 + len
  }
  const bpp = colorType === 6 ? 4 : 3
  const raw = inflateSync(Buffer.concat(idat))
  const stride = width * bpp
  const px = Buffer.alloc(height * stride)
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)]
    for (let i = 0; i < stride; i++) {
      const cur = raw[y * (stride + 1) + 1 + i]
      const a = i >= bpp ? px[y * stride + i - bpp] : 0
      const b = y > 0 ? px[(y - 1) * stride + i] : 0
      const c = i >= bpp && y > 0 ? px[(y - 1) * stride + i - bpp] : 0
      let v = cur
      if (filter === 1) v = cur + a
      else if (filter === 2) v = cur + b
      else if (filter === 3) v = cur + ((a + b) >> 1)
      else if (filter === 4) {
        const p = a + b - c
        const pa = Math.abs(p - a)
        const pb = Math.abs(p - b)
        const pc = Math.abs(p - c)
        v = cur + (pa <= pb && pa <= pc ? a : pb <= pc ? b : c)
      }
      px[y * stride + i] = v & 255
    }
  }
  return { width, height, bpp, px }
}

/** Pixels in a device-pixel box whose colour is within `tol` of `rgb`, plus the rows they occupy. */
function errorPixels(img, box, rgb, tol) {
  const rows = new Set()
  let count = 0
  const x0 = Math.max(0, Math.floor(box.x0))
  const x1 = Math.min(img.width, Math.ceil(box.x1))
  const y0 = Math.max(0, Math.floor(box.y0))
  const y1 = Math.min(img.height, Math.ceil(box.y1))
  for (let y = y0; y < y1; y++) {
    for (let x = x0; x < x1; x++) {
      const o = (y * img.width + x) * img.bpp
      const d = Math.abs(img.px[o] - rgb[0]) + Math.abs(img.px[o + 1] - rgb[1]) + Math.abs(img.px[o + 2] - rgb[2])
      if (d <= tol) {
        count++
        rows.add(y)
      }
    }
  }
  return { count, rows: [...rows].sort((a, b) => a - b) }
}

describe('proofread underline in a real WKWebView', () => {
  it('draws a wavy error underline under the located quote and clears it on typing', async () => {
    await openWorkspaceWithWork('proofread underline')
    await waitForGridRows(1, { col: 'tgt', what: 'proofread-underline grid' })

    const targetId = await browser.execute(
      () => document.querySelector('[data-col="tgt"]')?.getAttribute('data-segment-id') ?? null,
    )
    await realClick(await $(`[data-col="tgt"][data-segment-id="${targetId}"]`))
    await browser.pause(200)
    const typed = await browser.execute((text) => document.execCommand('insertText', false, text), TYPED)
    await expect(typed).toBe(true)

    const api = await browser.execute(() => ({
      highlights: typeof CSS !== 'undefined' && 'highlights' in CSS,
      highlightCtor: typeof Highlight === 'function',
      dpr: window.devicePixelRatio,
    }))
    await expect(api.highlights).toBe(true)
    await expect(api.highlightCtor).toBe(true)

    const dir = mkdtempSync(join(tmpdir(), 'auratranslate-e2e-proofread-'))
    const before = join(dir, 'before.png')
    const after = join(dir, 'after.png')
    await browser.saveScreenshot(before)

    // The wire cannot be stubbed (the Tauri bridge is frozen) and a real scan needs the OS keychain, so the
    // real draw helpers are called on the real cell; GridPanel's watcher wiring is guarded by vitest.
    const start = TYPED.indexOf(QUOTE)
    const painted = await browser.execute(
      async (segmentId, text, s, e) => {
        const mod = await import('/src/panels/proofreadHighlight.ts')
        const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
        const ranges = mod.rangesForScan(cell, text, [{ start: s, end: e }])
        mod.paintProofreadRanges(ranges)
        const guarded = mod.rangesForScan(cell, text + 'x', [{ start: s, end: e }])
        const h = CSS.highlights.get('proofread-error')
        const painted = h ? [...h] : []
        const rect = painted[0]?.getBoundingClientRect()
        const probe = document.createElement('canvas').getContext('2d')
        probe.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--color-error').trim()
        return {
          guardedNull: guarded === null,
          count: painted.length,
          quote: painted[0]?.toString() ?? null,
          inCell: painted[0] ? cell.contains(painted[0].startContainer) : false,
          rect: rect ? { x0: rect.left, x1: rect.right, y0: rect.top, y1: rect.bottom } : null,
          errorColor: probe.fillStyle,
        }
      },
      targetId,
      TYPED,
      start,
      start + QUOTE.length,
    )
    await expect(painted.guardedNull).toBe(true)
    await expect(painted.count).toBe(1)
    await expect(painted.quote).toBe(QUOTE)
    await expect(painted.inCell).toBe(true)

    await browser.pause(300)
    await browser.saveScreenshot(after)

    const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(painted.errorColor)
    const rgb = m ? [parseInt(m[1], 16), parseInt(m[2], 16), parseInt(m[3], 16)] : null
    await expect(rgb).not.toBe(null)

    const dpr = api.dpr
    const box = {
      x0: painted.rect.x0 * dpr,
      x1: painted.rect.x1 * dpr,
      y0: (painted.rect.y1 - 2) * dpr,
      y1: (painted.rect.y1 + 10) * dpr,
    }
    const imgBefore = decodePng(before)
    const imgAfter = decodePng(after)
    const scale = imgAfter.width / (await browser.execute(() => window.innerWidth))
    const sbox = { x0: box.x0 / dpr * scale, x1: box.x1 / dpr * scale, y0: box.y0 / dpr * scale, y1: box.y1 / dpr * scale }
    const pre = errorPixels(imgBefore, sbox, rgb, 200)
    const post = errorPixels(imgAfter, sbox, rgb, 200)
    console.log(
      `[proofread-measure] dpr=${dpr} scale=${scale} rect=${JSON.stringify(painted.rect)} before=${pre.count} after=${post.count} ` +
        `rowsAfter=${JSON.stringify(post.rows)} shots=${dir} color=${painted.errorColor}`,
    )
    await expect(post.count).toBeGreaterThan(pre.count + 3)
  })
})
