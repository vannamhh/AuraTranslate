/**
 * The grid splits a Chapter into blocks of at most 200 rows (`gridBlocks.ts`). vitest runs on
 * `happy-dom` and cannot see selection painting or scrolling; these cases do.
 *
 *     npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/grid-blocks.e2e.mjs
 *
 * Real OS mouse drags need `cliclick` and live in `_bmad-output/implementation-artifacts/2-16-ban-do/keo-cot-nguyen-van.e2e.mjs`.
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

const SENTENCE_COUNT = 1200
const TRANSLATED_ROWS = 1100
const BOUNDARY_ROW = 200
const TRANSLATION = 'Ban dich da co san tren dia.'

const TEXT = Array.from({ length: SENTENCE_COUNT }, (_, i) => `第${i + 1}句子内容在这里。`).join('')

let segments = []

async function invoke(command, args) {
  return browser.execute(async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a), command, args)
}

async function pressChord(chord) {
  await browser.execute((init) => {
    const target = document.activeElement ?? document.body
    target.dispatchEvent(new KeyboardEvent('keydown', { ...init, bubbles: true, cancelable: true }))
  }, chord)
}

async function scrollRowIntoView(index, col) {
  await browser.execute(
    (id, c) => {
      document.querySelector(`[data-col="${c}"][data-segment-id="${id}"]`).scrollIntoView({ block: 'center' })
    },
    segments[index].id,
    col,
  )
  await browser.pause(200)
}

async function cellOf(index, col) {
  return $(`[data-col="${col}"][data-segment-id="${segments[index].id}"]`)
}

describe('Grid blocks in a real webview', () => {
  before(async function () {
    this.timeout(300_000)
    await openWorkspaceWithWork(`e2e-grid-blocks-${Date.now()}`, TEXT)
    await waitForGridRows(SENTENCE_COUNT, { col: 'tgt', timeout: 60_000, what: 'Chapter just built' })

    const loaded = await invoke('read_open_chapter_segments', {})
    segments = loaded.segments
    expect(segments.length).toBe(SENTENCE_COUNT)
    await invoke('save_segment_targets', {
      chapterId: loaded.chapter_id,
      edits: segments.slice(0, TRANSLATED_ROWS).map((s) => ({ id: s.id, target_text: TRANSLATION })),
    })

    await browser.execute(() => {
      window.__beforeReload = true
      window.location.reload()
    })
    await browser.waitUntil(async () => browser.execute(() => window.__beforeReload !== true), {
      timeout: 60_000,
      timeoutMsg: 'the page never reloaded',
    })
    await browser.waitUntil(
      async () =>
        browser.execute(
          (n) => document.querySelectorAll('.grid-block').length === 6 && document.querySelectorAll('[data-col="tgt"]').length === n,
          SENTENCE_COUNT,
        ),
      { timeout: 60_000, interval: 500, timeoutMsg: 'the reloaded Chapter never showed six grid blocks' },
    )
  })

  it('the Chapter is split into blocks of at most 200 rows', async () => {
    const blocks = await browser.execute(() =>
      [...document.querySelectorAll('.grid-block')].map(
        (block) => block.querySelectorAll(':scope > .col-tgt > .cell-tgt').length,
      ),
    )
    expect(blocks).toEqual([200, 200, 200, 200, 200, 200])
  })

  it('next untranslated jumps from block 0 into block 5, lands in the cell and scrolls it into view', async () => {
    await realClick(await cellOf(0, 'tgt'))
    await browser.pause(300)

    await pressChord({ code: 'ArrowDown', key: 'ArrowDown', metaKey: true, altKey: true })
    await browser.pause(800)

    const landed = await browser.execute(() => {
      const active = document.activeElement
      const scroller = document.querySelector('.grid-scroll').getBoundingClientRect()
      const rect = active.getBoundingClientRect()
      const blocks = [...document.querySelectorAll('.grid-block')]
      return {
        id: active.getAttribute('data-segment-id'),
        col: active.getAttribute('data-col'),
        block: blocks.findIndex((b) => b.contains(active)),
        inView: rect.top >= scroller.top - 1 && rect.bottom <= scroller.bottom + 1,
      }
    })
    expect(landed.id).toBe(String(segments[TRANSLATED_ROWS].id))
    expect(landed.col).toBe('tgt')
    expect(landed.block).toBe(5)
    expect(landed.inView).toBe(true)
  })

  it('a selection in a source cell of block 5 reaches the Lookup panel when the mouse is released', async () => {
    await scrollRowIntoView(TRANSLATED_ROWS, 'src')
    const before = await browser.execute(async () => {
      const mod = await import('/src/panels/lookupPanelState.ts')
      return mod.currentQuery.value
    })

    const selected = await browser.execute((id) => {
      const piece = document.querySelector(`[data-col="src"][data-segment-id="${id}"] .src-piece`)
      const walker = document.createTreeWalker(piece, NodeFilter.SHOW_TEXT)
      const text = walker.nextNode()
      const range = document.createRange()
      range.setStart(text, 0)
      range.setEnd(text, text.length)
      const selection = window.getSelection()
      selection.removeAllRanges()
      selection.addRange(range)
      piece.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, cancelable: true }))
      return selection.toString().trim()
    }, segments[TRANSLATED_ROWS].id)
    expect(selected).not.toBe('')
    await browser.waitUntil(
      async () =>
        browser.execute(async () => {
          const mod = await import('/src/panels/lookupPanelState.ts')
          return mod.currentQuery.value
        }).then((now) => now !== null && now !== before),
      { timeout: 15_000, timeoutMsg: 'the Lookup panel never took the block 5 selection' },
    )
  })

  it('Mod+Shift+ArrowDown across the block boundary selects every row id and paints them in all four columns', async () => {
    const first = BOUNDARY_ROW - 2
    await scrollRowIntoView(first, 'tgt')
    await realClick(await cellOf(first, 'tgt'))
    await browser.pause(300)
    for (let i = 0; i < 4; i += 1) {
      await pressChord({ code: 'ArrowDown', key: 'ArrowDown', metaKey: true, shiftKey: true })
      await browser.pause(100)
    }
    await browser.pause(300)

    const painted = await browser.execute(() => {
      const ids = (sel) =>
        [...document.querySelectorAll(sel)].map((el) => el.getAttribute('data-segment-id'))
      return {
        tgt: ids('.cell-tgt.row-selected'),
        src: ids('.cell-src.row-selected'),
        num: document.querySelectorAll('.cell-num.row-selected').length,
        rule: document.querySelectorAll('.cell-rule.row-selected').length,
      }
    })
    const expected = segments.slice(first, first + 5).map((s) => String(s.id))
    expect(painted.tgt).toEqual(expected)
    expect(painted.src).toEqual(expected)
    expect(painted.num).toBe(5)
    expect(painted.rule).toBe(5)
  })
})
