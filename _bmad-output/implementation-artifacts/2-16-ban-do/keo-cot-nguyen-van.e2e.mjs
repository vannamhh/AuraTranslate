/**
 * Local bench, macOS only: real OS mouse drags through `cliclick` (WebDriver pointer actions start
 * no text selection on non-editable text in WKWebView). Needs `/usr/local/bin/cliclick`, an
 * Accessibility grant for the terminal, an unlocked screen and the window frontmost; nightly CI
 * has none of these, so this file lives outside `e2e/specs/`.
 *
 *     TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-16-ban-do/keo-cot-nguyen-van.e2e.mjs
 *
 * `DRAG_SOURCE_COLUMN` fails on the tree without `gridSourceSelection.ts` and the `[data-sel-src]`
 * rule in `GridPanel.vue`. Paint is measured on two window screenshots (before and after the drag;
 * `takeElementScreenshot` returns the whole window on this driver), compared inside the cell rects
 * scaled by png width / innerWidth, so the device pixel ratio of the machine does not enter.
 */
import { waitForGridRows } from '../../../e2e/support/gridWait.mjs'
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { calibrateOsMouse, osDrag } from './osMouse.mjs'
import { decodePng, differingPixels } from './pngRegion.mjs'

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

  it('DRAG_SOURCE_COLUMN: a real OS drag in the source column across the boundary paints and reads only that column', async function () {
    this.timeout(180_000)
    const from = BOUNDARY_ROW - 2
    const to = BOUNDARY_ROW + 2
    await scrollRowIntoView(from, 'src')
    await browser.execute(() => window.getSelection().removeAllRanges())
    await browser.pause(300)

    // Probes between the two dragged source cells: block 0 rows 198-199 (translation, number, rule)
    // and block 1 rows 200-201 (number, rule). Pixel equality with the same cell before the drag
    // means nothing was painted there; the source cells are the positive control.
    await browser.execute(
      (ids, boundary) => {
        const mark = (el, name) => el.setAttribute('data-probe', name)
        const blocks = document.querySelectorAll('.grid-block')
        for (const [k, id] of ids.entries()) {
          mark(document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`), `tgt${k}`)
          mark(document.querySelector(`[data-col="src"][data-segment-id="${id}"]`), `src${k}`)
        }
        for (const k of [0, 1]) {
          mark(blocks[0].querySelectorAll('.cell-num')[boundary - 2 + k], `num${k}`)
          mark(blocks[0].querySelectorAll('.cell-rule')[boundary - 2 + k], `rule${k}`)
          mark(blocks[1].querySelectorAll('.cell-num')[k], `num${k + 2}`)
          mark(blocks[1].querySelectorAll('.cell-rule')[k], `rule${k + 2}`)
        }
      },
      [segments[from].id, segments[from + 1].id],
      BOUNDARY_ROW,
    )
    const names = ['tgt0', 'tgt1', 'num0', 'num1', 'num2', 'num3', 'rule0', 'rule1', 'rule2', 'rule3', 'src0', 'src1']
    const rects = () =>
      browser.execute(
        (list) =>
          Object.fromEntries(
            list.map((n) => {
              const r = document.querySelector(`[data-probe="${n}"]`).getBoundingClientRect()
              return [n, { x: r.x, y: r.y, width: r.width, height: r.height }]
            }),
          ),
        names,
      )
    const window1 = await browser.execute(() => window.innerWidth)
    const quietRects = await rects()
    const quiet = decodePng(await browser.takeScreenshot())
    const scale = quiet.width / window1

    osDrag(osOffset, await textPointOf(from, 0), await textPointOf(to, 0.6))
    await browser.pause(400)

    const draggedRects = await rects()
    expect(draggedRects).toEqual(quietRects)
    const dragged = decodePng(await browser.takeScreenshot())
    const painted = {}
    for (const name of names) painted[name] = differingPixels(quiet, dragged, quietRects[name], scale)
    console.log(`PIXELS dpr=${scale} ${JSON.stringify(painted)}`)
    const read = await browser.execute(async (arg) => {
      const contract = await import('/src/panels/selectionContract.ts')
      const text = contract.currentSelectionText()
      const sel = window.getSelection()
      const copied = []
      const onCopy = (e) => copied.push({ text: e.clipboardData.getData('text/plain'), prevented: e.defaultPrevented })
      document.addEventListener('copy', onCopy)
      const ok = document.execCommand('copy')
      document.removeEventListener('copy', onCopy)
      return {
        lookupHasTranslation: text.includes(arg),
        lookupText: text,
        rangeHasTranslation: sel.toString().includes(arg),
        copyOk: ok,
        copied,
      }
    }, TRANSLATION)

    expect({
      outsideSourcePainted: names.filter((n) => !n.startsWith('src') && painted[n] > 0),
      sourcePainted: [painted.src0 > 0, painted.src1 > 0],
      lookupHasTranslation: read.lookupHasTranslation,
      lookupHasSource: read.lookupText.includes('句子'),
      copied: read.copied.map((c) => ({ prevented: c.prevented, hasTranslation: c.text.includes(TRANSLATION), hasSource: c.text.includes('句子') })),
    }).toEqual({
      outsideSourcePainted: [],
      sourcePainted: [true, true],
      lookupHasTranslation: false,
      lookupHasSource: true,
      copied: [{ prevented: true, hasTranslation: false, hasSource: true }],
    })
    // The Range itself still spans the other cells: only paint and readers are confined.
    expect(read.rangeHasTranslation).toBe(true)
  })
})
