/**
 * Local bench, macOS only: real OS mouse drags through `cliclick` (WebDriver pointer actions start
 * no text selection on non-editable text in WKWebView). Needs `/usr/local/bin/cliclick`, an
 * Accessibility grant for the terminal, an unlocked screen and the window frontmost; nightly CI
 * has none of these, so this file lives outside `e2e/specs/`.
 *
 *     TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-16-ban-do/keo-cot-nguyen-van.e2e.mjs
 *
 * `DRAG_SOURCE_COLUMN` fails on the tree without a drag-confinement change.
 */
import { waitForGridRows } from '../../../e2e/support/gridWait.mjs'
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { calibrateOsMouse, osDrag } from './osMouse.mjs'

const SENTENCE_COUNT = 1200
const TRANSLATED_ROWS = 1100
const BOUNDARY_ROW = 200
const TRANSLATION = 'Ban dich da co san tren dia.'

const TEXT = Array.from({ length: SENTENCE_COUNT }, (_, i) => `第${i + 1}句子内容在这里。`).join('')

let segments = []

async function invoke(command, args) {
  return browser.execute(async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a), command, args)
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

async function textPointOf(index, atFraction) {
  return browser.execute(
    (id, fraction) => {
      const piece = document.querySelector(`[data-col="src"][data-segment-id="${id}"] .src-piece`)
      const r = piece.getBoundingClientRect()
      return { x: r.x + Math.max(3, r.width * fraction - 3), y: r.y + r.height / 2 }
    },
    segments[index].id,
    atFraction,
  )
}

describe('Grid blocks: real OS mouse drags', () => {
  let osOffset = null

  before(async function () {
    this.timeout(300_000)
    await openWorkspaceWithWork(`e2e-grid-blocks-os-${Date.now()}`, TEXT)
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
    osOffset = await calibrateOsMouse()
  })

  it('a real OS mouse selection in a source cell of block 5 reaches the Lookup panel', async () => {
    await scrollRowIntoView(TRANSLATED_ROWS, 'src')
    const before = await browser.execute(async () => {
      const mod = await import('/src/panels/lookupPanelState.ts')
      return mod.currentQuery.value
    })

    osDrag(osOffset, await textPointOf(TRANSLATED_ROWS, 0), await textPointOf(TRANSLATED_ROWS, 1))
    await browser.pause(300)

    const selected = await browser.execute(() => window.getSelection().toString().trim())
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

  it('DRAG_SOURCE_COLUMN: a real OS drag in the source column across the boundary selects nothing outside that column', async () => {
    const from = BOUNDARY_ROW - 2
    const to = BOUNDARY_ROW + 2
    await scrollRowIntoView(from, 'src')
    await browser.execute(() => window.getSelection().removeAllRanges())
    osDrag(osOffset, await textPointOf(from, 0), await textPointOf(to, 0.6))
    await browser.pause(300)

    const result = await browser.execute((arg) => {
      const sel = window.getSelection()
      const touched = (selector) =>
        [...document.querySelectorAll(selector)].filter((el) => sel.containsNode(el, true)).length
      return {
        translationText: sel.toString().includes(arg),
        src: touched('.cell-src'),
        tgt: touched('.cell-tgt'),
        num: touched('.cell-num'),
        rule: touched('.cell-rule'),
      }
    }, TRANSLATION)
    expect(result.src).toBeGreaterThanOrEqual(3)
    expect({
      translationText: result.translationText,
      tgt: result.tgt,
      num: result.num,
      rule: result.rule,
    }).toEqual({ translationText: false, tgt: 0, num: 0, rule: 0 })
  })
})
