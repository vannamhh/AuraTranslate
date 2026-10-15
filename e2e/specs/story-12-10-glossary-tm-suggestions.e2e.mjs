/**
 * A pending Glossary term offers the translations the TM already holds for exactly that term,
 * and a digit key confirms one of them.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-10-glossary-tm-suggestions.e2e.mjs
 *
 * The Han Viet proposal needs the dictionary data, which CI does not build, so the spec reads
 * the proposals from the DOM and aims the digit at the TM translation instead of assuming it
 * is number 2.
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TERM = '萧炎'
const FIRST = '甲'
const SECOND = '乙'
const STRIP = '.glossary-confirm-strip'

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

async function optionTexts() {
  return browser.execute(
    (selector) =>
      Array.from(document.querySelectorAll(`${selector} .gcs-option .gcs-option-text`)).map(
        (el) => el.textContent,
      ),
    STRIP,
  )
}

describe('Story 12-10 -- Glossary strip proposes TM translations', () => {
  it('lists the TM translations of the exact term and a digit key confirms one', async () => {
    await browser.waitUntil(
      async () =>
        browser.execute(
          () => window.__TAURI_INTERNALS__ !== undefined && (document.querySelector('#app')?.children.length ?? 0) > 0,
        ),
      { timeout: 30_000, interval: 250 },
    )
    await invoke('glossary_add_term', {
      tier: 'global',
      sourceTerm: TERM,
      translation: null,
      note: '',
      category: 'person',
    })

    const text = `${TERM}\n${TERM}\n${TERM}\n${TERM}登场。`
    await openWorkspaceWithWork('Story 12-10 -- glossary TM', text)
    await waitForGridRows(4)

    const loaded = await invoke('read_open_chapter_segments', {})
    const segments = loaded.segments
    await expect(segments.length).toBe(4)
    await expect(segments[0].source_text).toBe(TERM)

    const targets = [SECOND, FIRST, FIRST]
    await invoke('save_segment_targets', {
      chapterId: loaded.chapter_id,
      edits: targets.map((target, i) => ({ id: segments[i].id, target_text: target })),
    })
    for (let i = 0; i < targets.length; i += 1) {
      await invoke('confirm_segment', { segmentId: segments[i].id })
    }

    const suggestions = await invoke('glossary_tm_suggestions', { sourceTerm: TERM })
    await expect(suggestions.map((s) => s.target_text)).toContain(FIRST)
    await expect(suggestions.map((s) => s.target_text)).toContain(SECOND)

    const anchorId = segments[3].id
    await browser.execute((id) => {
      document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`).focus()
    }, anchorId)

    await browser.waitUntil(async () => (await $(STRIP)).isExisting(), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the pending-term strip did not appear under the sentence that holds the term',
    })
    await browser.waitUntil(async () => (await optionTexts()).includes(FIRST), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the TM translation never showed up as a numbered proposal',
    })

    const texts = await optionTexts()
    await expect(texts).toContain(SECOND)
    const key = texts.indexOf(FIRST) + 1

    await browser.keys(['Meta', 'Alt', 'c'])
    await browser.waitUntil(
      async () => browser.execute((selector) => document.querySelector(selector)?.contains(document.activeElement) === true, STRIP),
      { timeout: 5_000, interval: 100, timeoutMsg: 'Mod+Alt+C did not move focus into the strip' },
    )
    await browser.keys([String(key)])

    await browser.waitUntil(async () => !(await $(STRIP).isExisting()), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the strip stayed after the digit key: the term is still pending',
    })

    const entries = await invoke('glossary_list_entries', {})
    const entry = entries.find((e) => e.source_term === TERM)
    await expect(entry.translation).toBe(FIRST)
  })
})
