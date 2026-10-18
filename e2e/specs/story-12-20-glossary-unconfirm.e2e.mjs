/**
 * A term confirmed from the Glossary strip can be unconfirmed from the "confirmed" notice:
 * the entry returns to pending, and the strip does not grow back on that term.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-20-glossary-unconfirm.e2e.mjs
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TERM = '萧炎'
const TRANSLATION = 'Tieu Viem'
const STRIP = '.glossary-confirm-strip'
const NOTICE = '[data-glossary-confirm-done]'

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

async function entryOf(term) {
  const entries = await invoke('glossary_list_entries', {})
  return entries.find((e) => e.source_term === term)
}

describe('Story 12-20 -- unconfirm a Glossary translation', () => {
  it('confirm from the strip, unconfirm from the notice, the term goes back to pending', async () => {
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

    await openWorkspaceWithWork('Story 12-20 -- glossary unconfirm', `${TERM}登场。`)
    await waitForGridRows(1)
    const loaded = await invoke('read_open_chapter_segments', {})
    const segmentId = loaded.segments[0].id

    await browser.execute((id) => {
      document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`).focus()
    }, segmentId)
    await browser.waitUntil(async () => (await $(STRIP)).isExisting(), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the pending-term strip did not appear',
    })

    await browser.keys(['Meta', 'Alt', 'c'])
    await browser.waitUntil(
      async () => browser.execute((s) => document.querySelector(s)?.contains(document.activeElement) === true, STRIP),
      { timeout: 5_000, interval: 100, timeoutMsg: 'Mod+Alt+C did not move focus into the strip' },
    )
    const input = await $(`${STRIP} .gcs-input`)
    await realClick(input)
    await input.setValue(TRANSLATION)
    await realClick(await $(`${STRIP} button[type="submit"]`))

    await browser.waitUntil(async () => (await $(NOTICE)).isExisting(), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the confirmed notice did not appear after the confirm',
    })
    await expect((await entryOf(TERM)).translation).toBe(TRANSLATION)
    await expect(await $(`${NOTICE} kbd`).isExisting()).toBe(false)

    await realClick(await $(`${NOTICE} .gcs-undo`))
    await browser.waitUntil(async () => !(await $(NOTICE).isExisting()), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the notice stayed after the undo',
    })

    await expect((await entryOf(TERM)).translation).toBe(null)
    await browser.pause(500)
    await expect(await $(STRIP).isExisting()).toBe(false)
  })
})
