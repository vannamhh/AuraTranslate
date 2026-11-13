import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('../../src/config/dict', async (original) => ({
  ...(await original<typeof import('../../src/config/dict')>()),
  lookupDictionary: vi.fn(async () => ({ response: null, error: null })),
}))

import { resetLookupPanel, runLookup } from '../../src/panels/lookupPanelState'
import { atomicBlocksForSpan, lookupSpanAtPoint, sourceRangesForSpan } from '../../src/panels/editorSegments'
import type { LookupSpanContext, SourceSpan } from '../../src/panels/editorSegments'
import {
  attachSelectionWatcher,
  currentSelectionText,
  offerPointLookup,
  registerSelectionSurface,
  skipNextAutoLookup,
} from '../../src/panels/selectionContract'
import { wordAt } from '../../src/panels/wordBoundary'

beforeEach(() => {
  document.body.innerHTML = ''
  window.getSelection()?.removeAllRanges()
})

function mountCell(html: string): HTMLElement {
  const cell = document.createElement('div')
  cell.setAttribute('data-col', 'src')
  cell.setAttribute('data-segment-id', '1')
  cell.innerHTML = html
  document.body.append(cell)
  return cell
}

function ctx(sourceText: string, overrides: Partial<LookupSpanContext> = {}): LookupSpanContext {
  return { sourceText, glossarySpanAt: () => null, wordAt, hits: () => true, pointerTarget: null, ...overrides }
}

function textOf(node: Node | null): Text {
  const text = node?.firstChild
  if (!(text instanceof Text)) throw new Error('no text node')
  return text
}

describe('wordAt', () => {
  it('returns the word containing the character, in raw code points', () => {
    const text = '他拉小提琴。'
    const word = wordAt(text, 3)
    expect(word).not.toBeNull()
    const span = word as { start: number; end: number; wordLike: boolean }
    expect(span.wordLike).toBe(true)
    expect([...text].slice(span.start, span.end).join('')).toBe('小提琴')
    expect(wordAt(text, span.start)).toEqual(wordAt(text, span.end - 1))
  })

  it('marks punctuation and whitespace as not word-like', () => {
    expect(wordAt('你好，世界', 2)?.wordLike).toBe(false)
    expect(wordAt('hello world', 5)?.wordLike).toBe(false)
  })

  it('returns a whole Latin word', () => {
    const word = wordAt('hello world', 8)
    expect(word).toEqual({ start: 6, end: 11, wordLike: true })
  })

  it('keeps raw code-point offsets across CRLF and astral characters', () => {
    const text = '你好\r\n𠀀世界'
    const rawCp = [...text]
    const at = rawCp.indexOf('世')
    const word = wordAt(text, at)
    expect(word?.wordLike).toBe(true)
    expect(rawCp.slice(word?.start, word?.end).join('')).toContain('世')
  })

  it('is null when the offset is outside the text', () => {
    expect(wordAt('你好', 2)).toBeNull()
    expect(wordAt('你好', -1)).toBeNull()
  })

  it('falls back to one code point per word without Intl.Segmenter', async () => {
    vi.resetModules()
    const original = Intl.Segmenter
    const error = vi.spyOn(console, 'error').mockImplementation(() => undefined)
    Object.defineProperty(Intl, 'Segmenter', { value: undefined, configurable: true, writable: true })
    try {
      const fresh = await import('../../src/panels/wordBoundary')
      expect(fresh.wordAt('小提琴a', 1)).toEqual({ start: 1, end: 2, wordLike: true })
      expect(fresh.wordAt('小，', 1)).toEqual({ start: 1, end: 2, wordLike: false })
    } finally {
      Object.defineProperty(Intl, 'Segmenter', { value: original, configurable: true, writable: true })
      error.mockRestore()
      vi.resetModules()
    }
  })
})

describe('point lookup through the single Auto-Lookup path', () => {
  function mountSource(): { release: () => void } {
    const cell = mountCell('<span class="src-piece" data-src-start="0">小提琴</span>')
    return { release: registerSelectionSurface(cell, 'source') }
  }

  it('dispatches once with the offered phrase and clears it after the release', () => {
    const { release } = mountSource()
    const dispatched = vi.fn(() => {
      expect(currentSelectionText()).toBe('小提琴')
    })
    const detach = attachSelectionWatcher(document, dispatched)

    offerPointLookup('小提琴')
    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))

    expect(dispatched).toHaveBeenCalledTimes(1)
    expect(currentSelectionText()).toBe('')
    detach()
    release()
  })

  it('positive control: Mod+Alt+L after the release reads the real selection, not the old phrase', () => {
    const { release } = mountSource()
    const detach = attachSelectionWatcher(document, vi.fn())
    offerPointLookup('小提琴')
    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))

    const piece = document.querySelector('.src-piece')
    const range = document.createRange()
    range.setStart(textOf(piece), 0)
    range.setEnd(textOf(piece), 2)
    window.getSelection()?.addRange(range)
    expect(currentSelectionText()).toBe('小提')
    detach()
    release()
  })

  it('does not dispatch twice for the same phrase', () => {
    const { release } = mountSource()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)
    for (let i = 0; i < 2; i += 1) {
      offerPointLookup('小提琴')
      document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))
    }
    expect(dispatched).toHaveBeenCalledTimes(1)
    detach()
    release()
  })

  it('skips the release that follows a double-click and then forgets the request', () => {
    const { release } = mountSource()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)

    const piece = document.querySelector('.src-piece')
    const range = document.createRange()
    range.setStart(textOf(piece), 1)
    range.setEnd(textOf(piece), 3)
    window.getSelection()?.addRange(range)
    skipNextAutoLookup()
    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, detail: 2 }))
    expect(dispatched).not.toHaveBeenCalled()

    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))
    expect(dispatched).toHaveBeenCalledTimes(1)
    detach()
    release()
  })

  it('forgets the last dispatched phrase once the panel moves to another phrase by an explicit path', async () => {
    const { release } = mountSource()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)
    const click = async (phrase: string): Promise<void> => {
      offerPointLookup(phrase)
      document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))
      await runLookup(phrase)
    }

    await click('小提琴')
    await runLookup('布拉姆斯')
    await click('小提琴')

    expect(dispatched).toHaveBeenCalledTimes(2)
    detach()
    release()
    resetLookupPanel()
  })

  it('a drag selection without an offered phrase still dispatches as before', () => {
    const { release } = mountSource()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)
    const piece = document.querySelector('.src-piece')
    const range = document.createRange()
    range.setStart(textOf(piece), 0)
    range.setEnd(textOf(piece), 2)
    window.getSelection()?.addRange(range)
    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))
    expect(dispatched).toHaveBeenCalledTimes(1)
    detach()
    release()
  })

  it('an empty click with nothing offered dispatches nothing', () => {
    const { release } = mountSource()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)
    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))
    expect(dispatched).not.toHaveBeenCalled()
    detach()
    release()
  })
})

describe('lookupSpanAtPoint, plain view', () => {
  const html = '<span class="src-piece" data-src-start="0">他拉小提琴。</span>'

  it('looks up the segmenter word containing the clicked character', () => {
    const cell = mountCell(html)
    const text = textOf(cell.firstElementChild)
    const span = lookupSpanAtPoint(cell, text, 3, ctx('他拉小提琴。', { hits: (t) => t instanceof Range && t.toString() === '提' }))
    expect([...'他拉小提琴。'].slice(span?.start, span?.end).join('')).toBe('小提琴')
  })

  it('picks the character the pointer is over when the caret sits on its right half', () => {
    const cell = mountCell(html)
    const text = textOf(cell.firstElementChild)
    const hitsOnly = (c: string) => (t: Range | Element) => t instanceof Range && t.toString() === c
    const left = lookupSpanAtPoint(cell, text, 3, ctx('他拉小提琴。', { hits: hitsOnly('小') }))
    const right = lookupSpanAtPoint(cell, text, 3, ctx('他拉小提琴。', { hits: hitsOnly('提') }))
    expect(left).toEqual(right)
  })

  it('the Glossary term containing the character wins over the word', () => {
    const cell = mountCell('<span class="src-piece" data-src-start="0">布拉姆斯来了</span>')
    const text = textOf(cell.firstElementChild)
    const term: SourceSpan = { start: 0, end: 4 }
    const span = lookupSpanAtPoint(
      cell,
      text,
      3,
      ctx('布拉姆斯来了', { glossarySpanAt: (cp) => (cp >= term.start && cp < term.end ? term : null), hits: (t) => t instanceof Range && t.toString() === '姆' }),
    )
    expect(span).toEqual(term)
  })

  it('punctuation gives nothing to look up', () => {
    const cell = mountCell(html)
    const text = textOf(cell.firstElementChild)
    expect(lookupSpanAtPoint(cell, text, 6, ctx('他拉小提琴。', { hits: (t) => t instanceof Range && t.toString() === '。' }))).toBeNull()
  })

  it('blank space around the text gives nothing to look up', () => {
    const cell = mountCell(html)
    const text = textOf(cell.firstElementChild)
    expect(lookupSpanAtPoint(cell, text, 3, ctx('他拉小提琴。', { hits: () => false }))).toBeNull()
  })

  it('a caret on the cell itself gives nothing to look up', () => {
    const cell = mountCell(html)
    expect(lookupSpanAtPoint(cell, cell, 0, ctx('他拉小提琴。'))).toBeNull()
  })

  it('counts astral characters as one code point', () => {
    const source = '𠀀小提琴'
    const cell = mountCell(`<span class="src-piece" data-src-start="0">${source}</span>`)
    const text = textOf(cell.firstElementChild)
    const span = lookupSpanAtPoint(cell, text, 3, ctx(source, { hits: (t) => t instanceof Range && t.toString() === '提' }))
    expect([...source].slice(span?.start, span?.end).join('')).toContain('提')
    expect(span?.start).toBeGreaterThanOrEqual(0)
  })
})

describe('lookupSpanAtPoint, Han Viet views', () => {
  const switchHtml =
    '<p class="hv-switch">' +
    '<span class="hv-word" data-src-start="0" data-src-atomic="1"><span class="hv-syl">tieu</span><span class="hv-syl">de</span></span>' +
    '<span class="hv-text" data-src-start="2" data-src-atomic="1">，</span>' +
    '<span class="hv-word" data-src-start="3" data-src-atomic="1"><span class="hv-syl">cam</span></span>' +
    '</p>'
  const source = '小提，琴'

  it('switch: a click on a reading looks up the whole Han block under it', () => {
    const cell = mountCell(switchHtml)
    const syl = cell.querySelector('.hv-syl')
    const span = lookupSpanAtPoint(cell, textOf(syl), 1, ctx(source))
    expect(span).toEqual({ start: 0, end: 2 })
  })

  it('switch: punctuation between blocks is not looked up', () => {
    const cell = mountCell(switchHtml)
    const span = lookupSpanAtPoint(cell, textOf(cell.querySelector('.hv-text')), 0, ctx(source))
    expect(span).toBeNull()
  })

  it('switch: a Glossary term containing the block wins', () => {
    const cell = mountCell(switchHtml)
    const term = { start: 0, end: 2 }
    const span = lookupSpanAtPoint(cell, textOf(cell.querySelector('.hv-syl')), 0, ctx(source, { glossarySpanAt: (cp) => (cp < 2 ? term : null) }))
    expect(span).toEqual(term)
  })

  it('parallel: a click on the reading under the Han word resolves to the block, not the reading text', () => {
    const cell = mountCell(
      '<p class="hv-parallel"><span class="hv-unit" data-src-start="0" data-src-atomic="1"><ruby>小提琴<rt>tieu de cam</rt></ruby></span></p>',
    )
    const rt = cell.querySelector('rt')
    const span = lookupSpanAtPoint(cell, textOf(rt), 2, ctx('小提琴'))
    expect(span).toEqual({ start: 0, end: 3 })
  })

  it('a block the pointer is not over is not looked up', () => {
    const cell = mountCell(switchHtml)
    expect(lookupSpanAtPoint(cell, textOf(cell.querySelector('.hv-syl')), 0, ctx(source, { hits: () => false }))).toBeNull()
  })

  it('switch: the block under the pointer wins over a caret that snapped to the end of the previous block', () => {
    const cell = mountCell(switchHtml)
    const punctuation = textOf(cell.querySelector('.hv-text'))
    const target = cell.querySelectorAll('.hv-syl')[2]
    const span = lookupSpanAtPoint(cell, punctuation, 1, ctx(source, { pointerTarget: target }))
    expect(span).toEqual({ start: 3, end: 4 })
  })

  it('parallel: the unit under the pointer wins over a caret that snapped into the previous reading', () => {
    const cell = mountCell(
      '<p class="hv-parallel">' +
        '<span class="hv-unit" data-src-start="0" data-src-atomic="1"><ruby>他<rt>tha</rt></ruby></span>' +
        '<span class="hv-unit" data-src-start="1" data-src-atomic="1"><ruby>小提琴<rt>tieu de cam</rt></ruby></span>' +
        '</p>',
    )
    const [first, second] = cell.querySelectorAll('ruby')
    const span = lookupSpanAtPoint(cell, textOf(first.querySelector('rt')), 3, ctx('他小提琴', { pointerTarget: second }))
    expect(span).toEqual({ start: 1, end: 4 })
  })

  it('punctuation under the pointer is not looked up even when the caret snapped into a block', () => {
    const cell = mountCell(switchHtml)
    const span = lookupSpanAtPoint(cell, textOf(cell.querySelector('.hv-syl')), 0, ctx(source, { pointerTarget: cell.querySelector('.hv-text') }))
    expect(span).toBeNull()
  })
})

describe('sourceRangesForSpan', () => {
  it('covers exactly the characters of the span inside a piece', () => {
    const cell = mountCell('<span class="src-piece" data-src-start="0">他拉</span><span class="src-piece" data-src-start="2">小提琴。</span>')
    const ranges = sourceRangesForSpan(cell, 2, 5)
    expect(ranges.map((r) => r.toString()).join('')).toBe('小提琴')
    expect(sourceRangesForSpan(cell, 1, 3).map((r) => r.toString())).toEqual(['拉', '小'])
  })

  it('leaves the reading out in the parallel view', () => {
    const cell = mountCell(
      '<span class="hv-unit" data-src-start="0" data-src-atomic="1"><ruby>小提琴<rt>tieu de cam</rt></ruby></span>',
    )
    expect(sourceRangesForSpan(cell, 0, 3).map((r) => r.toString())).toEqual(['小提琴'])
  })

  it('covers whole atomic blocks that overlap the span', () => {
    const cell = mountCell(
      '<span class="hv-word" data-src-start="0" data-src-atomic="1"><span class="hv-syl">a</span><span class="hv-syl">b</span></span><span class="hv-word" data-src-start="2" data-src-atomic="1"><span class="hv-syl">c</span></span>',
    )
    expect(sourceRangesForSpan(cell, 0, 2).map((r) => r.toString())).toEqual(['ab'])
    expect(sourceRangesForSpan(cell, 1, 3).map((r) => r.toString())).toEqual(['ab', 'c'])
  })

  it('counts astral characters as one code point', () => {
    const cell = mountCell('<span class="src-piece" data-src-start="0">𠀀小提琴</span>')
    expect(sourceRangesForSpan(cell, 1, 2).map((r) => r.toString())).toEqual(['小'])
  })
})

describe('atomicBlocksForSpan and block paint', () => {
  const html =
    '<span class="hv-word" data-src-start="0" data-src-atomic="1"><span class="hv-syl">a</span><span class="hv-syl">b</span></span>' +
    '<span class="hv-text" data-src-start="2" data-src-atomic="1">，</span>' +
    '<span class="hv-unit" data-src-start="3" data-src-atomic="1"><ruby>小提<rt>x y</rt></ruby></span>'

  it('returns the atomic blocks overlapping the span, whatever their class', () => {
    const cell = mountCell(html)
    expect(atomicBlocksForSpan(cell, 1, 4).map((e) => e.className)).toEqual(['hv-word', 'hv-text', 'hv-unit'])
    expect(atomicBlocksForSpan(cell, 3, 5).map((e) => e.className)).toEqual(['hv-unit'])
    expect(atomicBlocksForSpan(cell, 5, 6)).toEqual([])
  })

  it('painting block B after block A leaves only B marked, and resetLookupPaint removes it', async () => {
    const { resetLookupPaint, paintLookupBlocks } = await import('../../src/panels/lookupPaint')
    const cell = mountCell(html)
    const [a, , b] = [...cell.querySelectorAll('[data-src-atomic]')]
    paintLookupBlocks([a!])
    paintLookupBlocks([b!])
    expect(cell.querySelectorAll('[data-lookup-active]')).toHaveLength(1)
    expect(b!.hasAttribute('data-lookup-active')).toBe(true)
    resetLookupPaint()
    expect(cell.querySelectorAll('[data-lookup-active]')).toHaveLength(0)
  })

  it('parallel: only the reading is hit, the click still resolves to the block', () => {
    const cell = mountCell(html)
    const rt = cell.querySelector('rt')
    const span = lookupSpanAtPoint(cell, textOf(rt), 1, ctx('ab，小提', { hits: (t) => t === rt }))
    expect(span).toEqual({ start: 3, end: 5 })
  })

  it('parallel: a caret in the reading with the pointer over neither block nor reading looks nothing up', () => {
    const cell = mountCell(html)
    expect(lookupSpanAtPoint(cell, textOf(cell.querySelector('rt')), 1, ctx('ab，小提', { hits: () => false }))).toBeNull()
  })
})
