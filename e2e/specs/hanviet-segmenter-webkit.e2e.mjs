/**
 * `wordBoundary.ts`/`SourceHanViet.vue` measured `Intl.Segmenter`, `U+2060` and
 * `Selection.modify()` only on Chromium/`happy-dom` (`hanVietCutAnchors.test.ts`). This spec
 * re-measures the same three propositions on real WKWebView.
 */
import { realClick } from '../support/pointer.mjs'
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

/**
 * A driver-synthesized double-click here never sets `event.detail` above 0, so WebKit's native
 * double-click word-selection never engages. Both propositions below drive `Selection.modify()`
 * directly instead — a pure DOM API with no such gap, and what AC11's keyboard selection uses.
 */

/** Mirrors `SourceHanViet.vue`'s private `READING_PLACEHOLDER` — a syllable equal to this
 * carries no dictionary reading, and the cohesion test below needs real Latin text. */
const READING_PLACEHOLDER = '·'
const WORD_JOINER = '⁠'

/** '山东' (Shandong) is ICU's own dictionary word, isolated from '很美' by no self-spacing
 * boundary — same fixture used for all three propositions below. */
const SOURCE_TEXT = '山东很美。'

const ICU_WORD_CASES = [
  { text: SOURCE_TEXT, expected: ['山东', '很美', '。'] },
  { text: '你好世界', expected: ['你好', '世界'] },
  { text: '文化', expected: ['文化'] },
]

async function openHanVietSwitch(name) {
  await openWorkspaceWithWork(name, SOURCE_TEXT)
  await waitForGridRows(1)
  await realClick(await $('#grid-tab-han-viet'))
  await (await $('.hv-word')).waitForDisplayed({ timeout: 10_000 })
}

async function openHanVietParallel(name) {
  await openHanVietSwitch(name)
  await realClick(await $('.view-toggle'))
  await (await $('.hv-unit')).waitForDisplayed({ timeout: 10_000 })
}

function segmentWords(text) {
  return [...new Intl.Segmenter('zh', { granularity: 'word' }).segment(text)].map((p) => p.segment)
}

function readFirstWordSyllables() {
  return [...document.querySelectorAll('.hv-word')[0].querySelectorAll('.hv-syl')].map((s) => s.textContent)
}

/** `browser.execute` ships a function standalone (no shared module scope), so the two probes
 * below duplicate the same collapse-then-extend-by-word sequence rather than sharing a helper. */
function extendFirstSyllableByWord(steps) {
  const node = document.querySelectorAll('.hv-word')[0].querySelectorAll('.hv-syl')[0].firstChild
  const selection = window.getSelection()
  selection.removeAllRanges()
  const range = document.createRange()
  range.setStart(node, 0)
  range.collapse(true)
  selection.addRange(range)
  const texts = []
  for (let i = 0; i < steps; i += 1) {
    selection.modify('extend', 'right', 'word')
    texts.push(selection.toString())
  }
  return texts
}

function extendFirstUnitByWord(steps) {
  const unit = document.querySelectorAll('.hv-unit')[0]
  const ruby = unit.querySelector('ruby')
  const node = [...ruby.childNodes].find((n) => n.nodeType === Node.TEXT_NODE)
  const selection = window.getSelection()
  selection.removeAllRanges()
  const range = document.createRange()
  range.setStart(node, 0)
  range.collapse(true)
  selection.addRange(range)
  const texts = []
  for (let i = 0; i < steps; i += 1) {
    selection.modify('extend', 'right', 'word')
    texts.push(selection.toString())
  }
  return texts
}

describe('Hán Việt · Intl.Segmenter/U+2060/Selection.modify trên WKWebView thật', () => {
  it('① Intl.Segmenter cắt cùng ranh giới đã đo trên Chromium/happy-dom', async () => {
    await openHanVietSwitch('e2e-hv-segmenter-icu')
    for (const { text, expected } of ICU_WORD_CASES) {
      const got = await browser.execute(segmentWords, text)
      expect(got).toEqual(expected)
    }
  })

  it('② U+2060 giữ hai âm của một từ dính nhau qua Selection.modify(word); một dấu cách thật thì không', async () => {
    await openHanVietSwitch('e2e-hv-segmenter-joiner')
    await (await $('.hv-word')).waitForDisplayed({ timeout: 10_000 })

    const syllables = await browser.execute(readFirstWordSyllables)
    expect(syllables).toHaveLength(2)
    const plain = syllables.map((s) => s.replace(WORD_JOINER, ''))
    for (const s of plain) {
      expect(s).not.toBe('')
      expect(s).not.toBe(READING_PLACEHOLDER)
    }

    const joined = await browser.execute(extendFirstSyllableByWord, 1)
    expect(joined[0]).toContain(plain[0])
    expect(joined[0]).toContain(plain[1])

    // Counter-check: swap the invisible joiner for a real space, the exact difference the
    // production code measured (`SourceHanViet.vue`'s `WORD_JOINER` doc-comment).
    await browser.execute((joiner) => {
      const second = document.querySelectorAll('.hv-word')[0].querySelectorAll('.hv-syl')[1]
      second.firstChild.textContent = second.firstChild.textContent.replace(joiner, ' ')
    }, WORD_JOINER)
    const split = await browser.execute(extendFirstSyllableByWord, 1)
    expect(split[0]).not.toContain(plain[1])
  })

  it('③ Selection.modify(word) tiến qua ranh giới `.hv-unit` sang từ ICU kế tiếp', async () => {
    await openHanVietParallel('e2e-hv-segmenter-modify')
    await (await $('.hv-unit')).waitForDisplayed({ timeout: 10_000 })

    const texts = await browser.execute(extendFirstUnitByWord, 10)
    // '很'/'美' are the second ICU word's own characters — their presence, not a bare length
    // count, proves the selection crossed into the next `.hv-unit`.
    expect(texts.some((t) => t.includes('很') || t.includes('美'))).toBe(true)
  })
})
