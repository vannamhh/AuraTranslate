/**
 * The Work's "only my pairs" switch hides the others side from the using surfaces and tells how
 * much it hid, while Concordance and the TM manager still show every pair.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-14-tm-only-mine.e2e.mjs
 *
 * The others-side pair comes from confirming a bilingual-imported segment (origin
 * `bilingual_import`), which the switch treats like a pair translated by someone else.
 */
import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

import { waitForGridRows } from '../support/gridWait.mjs'
import { resetPanelState } from '../support/panelReset.mjs'
import { realClick } from '../support/pointer.mjs'

const SOURCE = '你好。'
const TARGET = 'Xin chao.'

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

async function waitForApp() {
  await browser.waitUntil(
    async () =>
      browser.execute(
        () => window.__TAURI_INTERNALS__ !== undefined && (document.querySelector('#app')?.children.length ?? 0) > 0,
      ),
    { timeout: 30_000, interval: 250 },
  )
}

async function openBilingualWork() {
  const dir = mkdtempSync(join(tmpdir(), 'aura-e2e-only-mine-'))
  const path = join(dir, 'song-ngu.csv')
  writeFileSync(path, `${SOURCE},${TARGET}\n`)
  await invoke('preview_bilingual_import_from_file', {
    path,
    sourceLang: 'zh',
    chapterPattern: null,
    sourceColumn: 0,
    targetColumn: 1,
    hasHeader: false,
    tableIndex: null,
  })
  await invoke('confirm_bilingual_import', {
    name: `Story 12-14 -- only mine ${Date.now()}`,
    sourceLang: 'zh',
    genre: 'general',
    encoding: 'UTF-8',
    chapterPattern: null,
    sourceColumn: 0,
    targetColumn: 1,
    hasHeader: false,
    regroupings: [],
    tableIndex: null,
    importMismatchedRows: false,
  })
  await resetPanelState()
  await browser.keys(['Meta', '2'])
  await waitForGridRows(1)
  const loaded = await invoke('read_open_chapter_segments', { prefill: false })
  await invoke('confirm_segment', { segmentId: loaded.segments[0].id })
}

describe('Story 12-14 -- only my pairs', () => {
  it('starts off, hides the others side with a count when switched on, and leaves Concordance whole', async () => {
    await waitForApp()
    await openBilingualWork()

    const initial = await invoke('tm_only_mine_get', {})
    await expect(initial).toEqual({ work_open: true, only_mine: false })

    const loaded = await invoke('read_open_chapter_segments', { prefill: false })
    const segmentId = loaded.segments[0].id

    const visible = await invoke('tm_fuzzy_matches', { segmentId })
    await expect(visible.hidden_count).toBe(0)

    const flipped = await invoke('tm_only_mine_set', { onlyMine: true })
    await expect(flipped).toEqual({ work_open: true, only_mine: true })

    const hidden = await invoke('tm_fuzzy_matches', { segmentId })
    await expect(hidden.hidden_count).toBe(1)
    await expect(hidden.exact).toEqual([])
    await expect(hidden.matches).toEqual([])

    const suggestions = await invoke('glossary_tm_suggestions', { sourceTerm: SOURCE })
    await expect(suggestions.suggestions).toEqual([])
    await expect(suggestions.hidden_count).toBe(1)

    const concordance = JSON.stringify(await invoke('tm_concordance', { query: SOURCE }))
    await expect(concordance).toContain(TARGET)
  })

  it('the manager still lists the pair, shows the switch on, and flips it with a real click', async () => {
    await invoke('tm_only_mine_set', { onlyMine: true })
    await browser.keys(['Meta', 'Alt', 'b'])
    await browser.waitUntil(async () => (await $('.tm-panel')).isExisting(), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the TM manager did not open on Mod+Alt+B',
    })

    const rows = await $$('.tm-row')
    await expect(rows.length).toBeGreaterThan(0)

    const toggle = await $('.tm-policy [role="switch"]')
    await browser.waitUntil(async () => (await toggle.getAttribute('aria-checked')) === 'true', {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the switch did not read "on" for a Work whose flag is on',
    })

    await realClick(toggle)
    await browser.waitUntil(async () => (await toggle.getAttribute('aria-checked')) === 'false', {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'a real click on the switch did not turn it off',
    })
    await expect(await invoke('tm_only_mine_get', {})).toEqual({ work_open: true, only_mine: false })
  })
})
