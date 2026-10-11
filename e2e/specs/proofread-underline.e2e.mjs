import { mkdtempSync, readFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { inflateSync } from 'node:zlib'
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

const TYPED = 'Tôi đi họcc ở trường lớp.'
const QUOTE = 'họcc'
const MEANING_QUOTE = 'trường lớp'

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

/** How far `after` moved from `before` toward `token`, as `after ≈ before + a·(token − before)`. */
function blendFit(before, after, token) {
  const d = [0, 1, 2].map((i) => token[i] - before[i])
  const dd = d[0] ** 2 + d[1] ** 2 + d[2] ** 2
  if (dd < 900) return null
  const v = [0, 1, 2].map((i) => after[i] - before[i])
  const a = (v[0] * d[0] + v[1] * d[1] + v[2] * d[2]) / dd
  return { a, residual: Math.hypot(...v.map((x, i) => x - a * d[i])) }
}

/**
 * Pixels in a device-pixel box that changed toward `token` and fit it better than `other`.
 * At dpr 1 a wavy stroke is about one anti-aliased pixel and never reaches the token colour itself.
 */
function tokenPixels(before, after, box, token, other) {
  let count = 0
  for (let y = Math.max(0, Math.floor(box.y0)); y < Math.min(after.height, Math.ceil(box.y1)); y++) {
    for (let x = Math.max(0, Math.floor(box.x0)); x < Math.min(after.width, Math.ceil(box.x1)); x++) {
      const o = (y * after.width + x) * after.bpp
      const b = [before.px[o], before.px[o + 1], before.px[o + 2]]
      const p = [after.px[o], after.px[o + 1], after.px[o + 2]]
      const mine = blendFit(b, p, token)
      if (mine === null || mine.a < 0.15 || mine.a > 1.2 || mine.residual > 12) continue
      const theirs = blendFit(b, p, other)
      if (theirs !== null && theirs.a >= 0.15 && theirs.residual <= mine.residual) continue
      count++
    }
  }
  return count
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
    const meaningStart = TYPED.indexOf(MEANING_QUOTE)
    const painted = await browser.execute(
      async (segmentId, text, s, e, ms, me) => {
        const mod = await import('/src/panels/proofreadHighlight.ts')
        const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
        const ranges = mod.rangesForScan(cell, text, [{ start: s, end: e }])
        const meaningRanges = mod.rangesForScan(cell, text, [{ start: ms, end: me }])
        mod.paintProofreadRanges(ranges, meaningRanges)
        const guarded = mod.rangesForScan(cell, text + 'x', [{ start: s, end: e }])
        const h = CSS.highlights.get('proofread-error')
        const painted = h ? [...h] : []
        const rect = painted[0]?.getBoundingClientRect()
        const mh = CSS.highlights.get('proofread-meaning')
        const meaningPainted = mh ? [...mh] : []
        const mrect = meaningPainted[0]?.getBoundingClientRect()
        const probe = document.createElement('canvas').getContext('2d')
        const tokenColor = (name) => {
          probe.fillStyle = '#000000'
          probe.fillStyle = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
          return probe.fillStyle
        }
        return {
          meaningCount: meaningPainted.length,
          meaningQuote: meaningPainted[0]?.toString() ?? null,
          meaningRect: mrect ? { x0: mrect.left, x1: mrect.right, y0: mrect.top, y1: mrect.bottom } : null,
          meaningColor: tokenColor('--color-tm-rule'),
          guardedNull: guarded === null,
          count: painted.length,
          quote: painted[0]?.toString() ?? null,
          inCell: painted[0] ? cell.contains(painted[0].startContainer) : false,
          rect: rect ? { x0: rect.left, x1: rect.right, y0: rect.top, y1: rect.bottom } : null,
          errorColor: tokenColor('--color-error'),
        }
      },
      targetId,
      TYPED,
      start,
      start + QUOTE.length,
      meaningStart,
      meaningStart + MEANING_QUOTE.length,
    )
    await expect(painted.guardedNull).toBe(true)
    await expect(painted.count).toBe(1)
    await expect(painted.quote).toBe(QUOTE)
    await expect(painted.inCell).toBe(true)
    await expect(painted.meaningCount).toBe(1)
    await expect(painted.meaningQuote).toBe(MEANING_QUOTE)
    await expect(painted.meaningColor).not.toBe(painted.errorColor)

    await browser.pause(300)
    await browser.saveScreenshot(after)

    const toRgb = (color) => {
      const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(color)
      return m ? [parseInt(m[1], 16), parseInt(m[2], 16), parseInt(m[3], 16)] : null
    }
    const rgb = toRgb(painted.errorColor)
    const meaningRgb = toRgb(painted.meaningColor)
    await expect(rgb).not.toBe(null)
    await expect(meaningRgb).not.toBe(null)

    const imgBefore = decodePng(before)
    const imgAfter = decodePng(after)
    const scale = imgAfter.width / (await browser.execute(() => window.innerWidth))
    const underBox = (rect) => ({
      x0: rect.x0 * scale,
      x1: rect.x1 * scale,
      y0: (rect.y1 - 4) * scale,
      y1: (rect.y1 + 6) * scale,
    })
    const qbox = underBox(painted.rect)
    const mbox = underBox(painted.meaningRect)
    const errorUnderQuote = tokenPixels(imgBefore, imgAfter, qbox, rgb, meaningRgb)
    const meaningUnderQuote = tokenPixels(imgBefore, imgAfter, qbox, meaningRgb, rgb)
    const meaningUnderMeaning = tokenPixels(imgBefore, imgAfter, mbox, meaningRgb, rgb)
    const errorUnderMeaning = tokenPixels(imgBefore, imgAfter, mbox, rgb, meaningRgb)
    console.log(
      `[proofread-measure] dpr=${api.dpr} scale=${scale} rect=${JSON.stringify(painted.rect)} shots=${dir} ` +
        `quote: error=${errorUnderQuote} meaning=${meaningUnderQuote} · meaningQuote: meaning=${meaningUnderMeaning} error=${errorUnderMeaning}`,
    )
    await expect(errorUnderQuote).toBeGreaterThanOrEqual(10)
    await expect(errorUnderQuote).toBeGreaterThan(5 * meaningUnderQuote)
    await expect(meaningUnderMeaning).toBeGreaterThanOrEqual(10)
    await expect(meaningUnderMeaning).toBeGreaterThan(5 * errorUnderMeaning)
  })
})
