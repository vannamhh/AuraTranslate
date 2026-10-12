import { mkdtempSync, readFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { inflateSync } from 'node:zlib'
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

const TYPED = 'Tôi đi học ạ ộ ợ ở trường lớp.'
const QUOTE = 'học ạ ộ ợ'
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
 * Rows of pixels in a device-pixel box that changed toward `token` and fit it better than `other`.
 * At dpr 1 a wavy stroke is about one anti-aliased pixel and never reaches the token colour itself.
 */
function tokenRows(before, after, box, token, other) {
  const rows = []
  for (let y = Math.max(0, Math.floor(box.y0)); y < Math.min(after.height, Math.ceil(box.y1)); y++) {
    for (let x = Math.max(0, Math.floor(box.x0)); x < Math.min(after.width, Math.ceil(box.x1)); x++) {
      const o = (y * after.width + x) * after.bpp
      const b = [before.px[o], before.px[o + 1], before.px[o + 2]]
      const p = [after.px[o], after.px[o + 1], after.px[o + 2]]
      const mine = blendFit(b, p, token)
      if (mine === null || mine.a < 0.15 || mine.a > 1.2 || mine.residual > 12) continue
      const theirs = blendFit(b, p, other)
      if (theirs !== null && theirs.a >= 0.15 && theirs.residual <= mine.residual) continue
      rows.push(y)
    }
  }
  return rows
}

/** Lowest device-pixel row holding text ink (anything far from the box's most common colour) in `before`. */
function lowestInkRow(before, box) {
  const counts = new Map()
  const rows = []
  for (let y = Math.floor(box.y0); y < Math.min(before.height, Math.ceil(box.y1)); y++) {
    for (let x = Math.floor(box.x0); x < Math.min(before.width, Math.ceil(box.x1)); x++) {
      const o = (y * before.width + x) * before.bpp
      const key = `${before.px[o]},${before.px[o + 1]},${before.px[o + 2]}`
      counts.set(key, (counts.get(key) ?? 0) + 1)
    }
  }
  const bg = [...counts.entries()].sort((a, b) => b[1] - a[1])[0][0].split(',').map(Number)
  let low = -1
  for (let y = Math.floor(box.y0); y < Math.min(before.height, Math.ceil(box.y1)); y++) {
    for (let x = Math.floor(box.x0); x < Math.min(before.width, Math.ceil(box.x1)); x++) {
      const o = (y * before.width + x) * before.bpp
      if (Math.hypot(before.px[o] - bg[0], before.px[o + 1] - bg[1], before.px[o + 2] - bg[2]) > 120) low = Math.max(low, y)
    }
  }
  return low
}

const toRgb = (color) => {
  const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(color)
  return m ? [parseInt(m[1], 16), parseInt(m[2], 16), parseInt(m[3], 16)] : null
}

describe('proofread underline in a real WKWebView', () => {
  for (const theme of ['light', 'dark']) {
    it(`draws wavy underlines clear of the under-dots and clears them on typing (${theme})`, async () => {
      await openWorkspaceWithWork(`proofread underline ${theme}`)
      await waitForGridRows(1, { col: 'tgt', what: 'proofread-underline grid' })
      await browser.execute(async (t) => {
        const mod = await import('/src/tokens/index.ts')
        mod.applyTheme(t)
      }, theme)

      const targetId = await browser.execute(
        () => document.querySelector('[data-col="tgt"]')?.getAttribute('data-segment-id') ?? null,
      )
      await realClick(await $(`[data-col="tgt"][data-segment-id="${targetId}"]`))
      await browser.pause(200)
      const typed = await browser.execute((text) => document.execCommand('insertText', false, text), TYPED)
      await expect(typed).toBe(true)
      const dpr = await browser.execute(() => window.devicePixelRatio)

      const dir = mkdtempSync(join(tmpdir(), 'auratranslate-e2e-proofread-'))
      const before = join(dir, 'before.png')
      const after = join(dir, 'after.png')
      const cleared = join(dir, 'cleared.png')
      await browser.saveScreenshot(before)

      // The wire cannot be stubbed (the Tauri bridge is frozen) and a real scan needs the OS keychain, so the
      // real draw helper is called on the real cell; GridPanel's watcher wiring is guarded by vitest.
      const start = TYPED.indexOf(QUOTE)
      const meaningStart = TYPED.indexOf(MEANING_QUOTE)
      const painted = await browser.execute(
        async (segmentId, text, s, e, ms, me) => {
          const mod = await import('/src/panels/proofreadHighlight.ts')
          const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
          mod.wrapProofread(cell, text, [
            { start: s, end: e, meaning: false },
            { start: ms, end: me, meaning: true },
          ])
          const errorSpans = [...cell.querySelectorAll('.proofread-error')]
          const meaningSpans = [...cell.querySelectorAll('.proofread-meaning')]
          const box = (el) => {
            const r = el?.getBoundingClientRect()
            return r ? { x0: r.left, x1: r.right, y0: r.top, y1: r.bottom } : null
          }
          const probe = document.createElement('canvas').getContext('2d')
          const tokenColor = (name) => {
            probe.fillStyle = '#000000'
            probe.fillStyle = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
            return probe.fillStyle
          }
          return {
            text: cell.textContent,
            errorQuote: errorSpans.map((x) => x.textContent),
            meaningQuote: meaningSpans.map((x) => x.textContent),
            rect: box(errorSpans[0]),
            meaningRect: box(meaningSpans[0]),
            meaningColor: tokenColor('--color-tm-text'),
            errorColor: tokenColor('--color-error'),
            offset: getComputedStyle(errorSpans[0]).textUnderlineOffset,
          }
        },
        targetId,
        TYPED,
        start,
        start + QUOTE.length,
        meaningStart,
        meaningStart + MEANING_QUOTE.length,
      )
      await expect(painted.text).toBe(TYPED)
      await expect(painted.errorQuote).toEqual([QUOTE])
      await expect(painted.meaningQuote).toEqual([MEANING_QUOTE])
      await expect(painted.meaningColor).not.toBe(painted.errorColor)

      await browser.pause(300)
      await browser.saveScreenshot(after)

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
      const errorUnderQuote = tokenRows(imgBefore, imgAfter, qbox, rgb, meaningRgb)
      const meaningUnderQuote = tokenRows(imgBefore, imgAfter, qbox, meaningRgb, rgb)
      const meaningUnderMeaning = tokenRows(imgBefore, imgAfter, mbox, meaningRgb, rgb)
      const errorUnderMeaning = tokenRows(imgBefore, imgAfter, mbox, rgb, meaningRgb)
      const ink = lowestInkRow(imgBefore, { x0: qbox.x0, x1: qbox.x1, y0: painted.rect.y0 * scale, y1: qbox.y1 })
      const waveTop = Math.min(...errorUnderQuote)
      console.log(
        `[proofread-measure] ${theme} dpr=${dpr} scale=${scale} shots=${dir} ` +
          `quote: error=${errorUnderQuote.length} meaning=${meaningUnderQuote.length} · meaningQuote: meaning=${meaningUnderMeaning.length} error=${errorUnderMeaning.length} · inkBottom=${ink} waveTop=${waveTop}`,
      )
      await expect(errorUnderQuote.length).toBeGreaterThanOrEqual(10)
      await expect(errorUnderQuote.length).toBeGreaterThan(5 * meaningUnderQuote.length)
      await expect(meaningUnderMeaning.length).toBeGreaterThanOrEqual(10)
      await expect(meaningUnderMeaning.length).toBeGreaterThan(5 * errorUnderMeaning.length)
      await expect(ink).toBeGreaterThan(0)
      await expect(waveTop - ink).toBeGreaterThan(1)
      await expect(painted.offset).toBe('4px')

      // Wrapped line: narrow the cell so the quote spans two lines; the wave must sit under both fragments.
      const narrowed = await browser.execute(async (segmentId) => {
        const mod = await import('/src/panels/proofreadHighlight.ts')
        const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
        const span = cell.querySelector('.proofread-error')
        const cellLeft = cell.getBoundingClientRect().left
        cell.style.width = `${Math.round(span.getBoundingClientRect().left - cellLeft + 60)}px`
        mod.unwrapProofread(cell, true)
        return true
      }, targetId)
      await expect(narrowed).toBe(true)
      await browser.pause(200)
      const wrapBefore = join(dir, 'wrap-before.png')
      const wrapAfter = join(dir, 'wrap-after.png')
      await browser.saveScreenshot(wrapBefore)
      const lines = await browser.execute(
        async (segmentId, text, s, e) => {
          const mod = await import('/src/panels/proofreadHighlight.ts')
          const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
          mod.wrapProofread(cell, text, [{ start: s, end: e, meaning: false }])
          return [...cell.querySelector('.proofread-error').getClientRects()].map((r) => ({ x0: r.left, x1: r.right, y0: r.top, y1: r.bottom }))
        },
        targetId,
        TYPED,
        start,
        start + QUOTE.length,
      )
      await expect(lines.length).toBeGreaterThanOrEqual(2)
      await browser.pause(300)
      await browser.saveScreenshot(wrapAfter)
      const wb = decodePng(wrapBefore)
      const wa = decodePng(wrapAfter)
      const perLine = lines.map((rect) => tokenRows(wb, wa, underBox(rect), rgb, meaningRgb).length)
      console.log(`[proofread-measure] ${theme} wrapped fragments=${lines.length} waveByFragment=${JSON.stringify(perLine)}`)
      for (const n of perLine) await expect(n).toBeGreaterThanOrEqual(5)

      // Theme flip after painting: the same spans must take the other theme's colour with no repaint.
      const other = theme === 'light' ? 'dark' : 'light'
      const flipped = await browser.execute(async (to, back) => {
        const mod = await import('/src/tokens/index.ts')
        const probe = document.createElement('canvas').getContext('2d')
        const hex = (v) => {
          probe.fillStyle = '#000000'
          probe.fillStyle = v
          return probe.fillStyle
        }
        const span = document.querySelector('.proofread-error')
        const read = () => hex(getComputedStyle(span).textDecorationColor)
        const first = read()
        mod.applyTheme(to)
        const second = read()
        const expected = hex(getComputedStyle(document.documentElement).getPropertyValue('--color-error').trim())
        mod.applyTheme(back)
        return { first, second, expected }
      }, other, theme)
      await expect(flipped.second).toBe(flipped.expected)
      await expect(flipped.second).not.toBe(flipped.first)
      await browser.execute((segmentId) => {
        document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`).style.width = ''
      }, targetId)
      await browser.pause(200)

      const rewrapped = await browser.execute(
        async (segmentId, text, s, e, ms, me) => {
          const mod = await import('/src/panels/proofreadHighlight.ts')
          const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
          mod.unwrapProofread(cell, true)
          mod.wrapProofread(cell, text, [
            { start: s, end: e, meaning: false },
            { start: ms, end: me, meaning: true },
          ])
          return { spans: cell.querySelectorAll('.proofread-error, .proofread-meaning').length, text: cell.textContent }
        },
        targetId,
        TYPED,
        start,
        start + QUOTE.length,
        meaningStart,
        meaningStart + MEANING_QUOTE.length,
      )
      await expect(rewrapped.spans).toBe(2)
      await expect(rewrapped.text).toBe(TYPED)

      const insertedX = await browser.execute((text) => document.execCommand('insertText', false, text), 'x')
      await expect(insertedX).toBe(true)
      await browser.pause(300)
      const afterTyping = await browser.execute((segmentId) => {
        const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
        return { text: cell.textContent, spans: cell.querySelectorAll('.proofread-error, .proofread-meaning').length, nodes: cell.childNodes.length }
      }, targetId)
      await expect(afterTyping.spans).toBe(0)
      await expect(afterTyping.text).toBe(`${TYPED}x`)
      await browser.saveScreenshot(cleared)
      const imgCleared = decodePng(cleared)
      await expect(tokenRows(imgBefore, imgCleared, qbox, rgb, meaningRgb).length).toBe(0)
      await expect(tokenRows(imgBefore, imgCleared, mbox, meaningRgb, rgb).length).toBe(0)

      const mid = await browser.execute(
        async (segmentId, text, s, e) => {
          const mod = await import('/src/panels/proofreadHighlight.ts')
          const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
          mod.wrapProofread(cell, text, [{ start: s, end: e, meaning: false }])
          getSelection().setPosition(cell.querySelector('.proofread-error').firstChild, 2)
          const ok = document.execCommand('insertText', false, 'y')
          return { ok, text: cell.textContent, spans: cell.querySelectorAll('.proofread-error, .proofread-meaning').length }
        },
        targetId,
        `${TYPED}x`,
        start,
        start + QUOTE.length,
      )
      await expect(mid.ok).toBe(true)
      await expect(mid.spans).toBe(0)
      await expect(mid.text).toBe(`${TYPED}x`.slice(0, start + 2) + 'y' + `${TYPED}x`.slice(start + 2))
    })
  }

  describe('with spans in the cell', () => {
    const SETUP = async () => {
      await openWorkspaceWithWork('proofread underline keys')
      await waitForGridRows(1, { col: 'tgt', what: 'proofread-underline grid' })
      const id = await browser.execute(
        () => document.querySelector('[data-col="tgt"]')?.getAttribute('data-segment-id') ?? null,
      )
      await realClick(await $(`[data-col="tgt"][data-segment-id="${id}"]`))
      await browser.pause(200)
      return id
    }

    // Types the same two runs into the real cell (spans drawn, `steps` applied) and into a bare contenteditable;
    // native undo must leave both with the same text.
    const undoOutcome = (id, steps) =>
      browser.execute(
        async (segmentId, mode) => {
          const mod = await import('/src/panels/proofreadHighlight.ts')
          const caretAt = (el, offset) => {
            const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT)
            let consumed = 0
            for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
              if (offset <= consumed + node.data.length) {
                getSelection().setPosition(node, offset - consumed)
                return
              }
              consumed += node.data.length
            }
          }
          const run = (el, withSpans) => {
            el.textContent = 'Tôi '
            el.focus()
            caretAt(el, 4)
            document.execCommand('insertText', false, 'đi họcc')
            let spansAfterWrap = 0
            if (withSpans) {
              const i = el.textContent.indexOf('họcc')
              mod.wrapProofread(el, el.textContent, [{ start: i, end: i + 4, meaning: false }])
              spansAfterWrap = el.querySelectorAll('.proofread-error').length
            }
            if (mode === 'keydown') {
              caretAt(el, el.textContent.length - 2)
              if (withSpans) el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Shift', bubbles: true }))
              document.execCommand('insertText', false, 'x')
            }
            document.execCommand('undo')
            return { text: el.textContent, spansAfterWrap }
          }
          const bare = document.createElement('div')
          bare.contentEditable = 'true'
          document.body.appendChild(bare)
          const expected = run(bare, false).text
          bare.remove()
          const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
          const before = cell.textContent
          const wrapped = run(cell, true)
          const actual = wrapped.text
          return { spansAfterWrap: wrapped.spansAfterWrap, expected, actual, before, spans: cell.querySelectorAll('.proofread-error, .proofread-meaning').length }
        },
        id,
        steps,
      )

    it('native undo after a draw gives the text it gives without spans', async () => {
      const id = await SETUP()
      const out = await undoOutcome(id, 'history')
      console.log(`[proofread-measure] undo history ${JSON.stringify(out)}`)
      await expect(out.spansAfterWrap).toBeGreaterThan(0)
      await expect(out.actual).toBe(out.expected)
      await expect(out.expected).not.toBe(out.before)
    })

    it('typing after a draw, then undo, gives the text it gives without spans', async () => {
      const id = await SETUP()
      const out = await undoOutcome(id, 'keydown')
      console.log(`[proofread-measure] undo keydown ${JSON.stringify(out)}`)
      await expect(out.spansAfterWrap).toBeGreaterThan(0)
      await expect(out.actual).toBe(out.expected)
      await expect(out.expected).not.toBe(out.before)
    })

    it('redraws after a key that writes nothing, and holds still during a composition', async () => {
      const id = await SETUP()
      const result = await browser.execute(async (segmentId) => {
        const mod = await import('/src/proofreadState.ts')
        const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${segmentId}"]`)
        const text = 'Tôi đi họcc ở trường.'
        cell.focus()
        document.execCommand('insertText', false, text)
        const at = cell.textContent.indexOf('họcc')
        const wire = { start: at, end: at + 4, kind: 'spelling' }
        mod.proofreadRunSegmentIds.value = [Number(segmentId)]
        mod.proofreadResults.value = new Map([[Number(segmentId), { scannedText: cell.textContent, findings: [wire], unlocated: 0, ambiguous: 0 }]])
        await new Promise((r) => setTimeout(r, 150))
        const count = () => cell.querySelectorAll('.proofread-error').length
        const drawn = count()
        cell.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true }))
        const duringKey = count()
        await new Promise((r) => setTimeout(r, 150))
        const afterKey = count()
        cell.dispatchEvent(new InputEvent('beforeinput', { inputType: 'insertFromPaste', data: '', bubbles: true, cancelable: true }))
        await new Promise((r) => setTimeout(r, 150))
        const afterBlocked = count()
        const span = cell.querySelector('.proofread-error')
        cell.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true }))
        cell.dispatchEvent(new KeyboardEvent('keydown', { key: 'a', bubbles: true }))
        cell.dispatchEvent(new InputEvent('beforeinput', { inputType: 'insertCompositionText', data: 'a', bubbles: true }))
        await new Promise((r) => setTimeout(r, 150))
        const sameSpanDuring = cell.querySelector('.proofread-error') === span
        cell.dispatchEvent(new CompositionEvent('compositionend', { bubbles: true }))
        await new Promise((r) => setTimeout(r, 150))
        const afterEnd = count()
        mod.resetProofread()
        return { text: cell.textContent === text, drawn, duringKey, afterKey, afterBlocked, sameSpanDuring, afterEnd }
      }, id)
      console.log(`[proofread-measure] keys ${JSON.stringify(result)}`)
      await expect(result.text).toBe(true)
      await expect(result.drawn).toBe(1)
      await expect(result.duringKey).toBe(0)
      await expect(result.afterKey).toBe(1)
      await expect(result.afterBlocked).toBe(1)
      await expect(result.sameSpanDuring).toBe(true)
      await expect(result.afterEnd).toBe(1)
    })
  })
})
