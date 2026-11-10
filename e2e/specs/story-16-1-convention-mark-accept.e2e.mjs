/**
 * One Global convention rule goes from the Quản lý quy ước overlay to a dotted mark under the avoided
 * phrase and to a one-spot replacement, all with the real mouse.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-16-1-convention-mark-accept.e2e.mjs
 */
import { waitForGridRows, waitForGridText } from '../support/gridWait.mjs'
import { realClick } from '../support/pointer.mjs'
import { resetPanelState } from '../support/panelReset.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const NAME = `Story 16-1 -- convention ${Date.now()}`
const AVOID = 'đảm bảo'
const REPLACEMENT = 'bảo đảm'
const BEFORE = 'Đảm bảo an toàn.'
const AFTER = 'Bảo đảm an toàn.'
const MARK = '[data-col="tgt"] .convention-mark'

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

// The WebKit driver drops a space handed to `browser.keys` and keeps focus on the field clicked
// first, so keystroke typing glues both phrases into one input; `setValue` addresses the element.
async function pickAddTier(value) {
  const trigger = await $('[data-convention-add-tier] summary')
  await trigger.waitForDisplayed({ timeout: 10_000 })
  await realClick(trigger)
  const option = await $(`//input[@data-convention-add-tier-option="${value}"]/parent::label`)
  await option.waitForDisplayed({ timeout: 10_000 })
  await realClick(option)
}

async function typeInto(selector, value) {
  const input = await $(selector)
  await realClick(input)
  await input.setValue(value)
  expect(await input.getValue()).toBe(value)
}

describe('Story 16-1 -- convention rule, mark and accept', () => {
  let segmentId

  it('an AI-translated sentence is on the grid before any rule exists, with no mark', async () => {
    await openWorkspaceWithWork(NAME, 'Bảo đảm an toàn.')
    await waitForGridRows(1, { col: 'tgt', what: 'convention grid' })
    const read = await invoke('read_open_chapter_segments', {})
    segmentId = read.segments[0].id
    await invoke('promote_ai_translation', { segmentId, targetText: BEFORE, force: false })
    const stored = (await invoke('read_open_chapter_segments', {})).segments[0]
    await expect(stored.target_text).toBe(BEFORE)
    // The bench writes `target_text` straight over IPC, so nothing mirrors it into the grid
    // the way the product does; this replays the panel's own load so the grid rereads the row.
    await resetPanelState()
    await waitForGridText(0, BEFORE, { col: 'tgt' })
    await expect(await browser.execute((sel) => document.querySelectorAll(sel).length, MARK)).toBe(0)
  })

  it('adds the rule through the overlay with the mouse and the open Chapter is rescanned', async () => {
    await realClick(await $('.tools-trigger'))
    const item = await $('[data-tools-item="convention.manage.open"]')
    await item.waitForDisplayed({ timeout: 10_000 })
    await realClick(item)
    await $('[data-convention-add-avoid]').waitForDisplayed({ timeout: 10_000 })
    await typeInto('[data-convention-add-avoid]', AVOID)
    await typeInto('[data-convention-add-replacement]', REPLACEMENT)
    await pickAddTier('global')
    await realClick(await $('[data-convention-add]'))
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelectorAll('[data-convention-rule-row]').length === 1),
      { timeout: 10_000, timeoutMsg: 'the rule row never appeared' },
    )
    await browser.keys(['Escape'])
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('[data-convention-add-avoid]') === null),
      { timeout: 10_000, timeoutMsg: 'the overlay stayed open after Escape' },
    )
    await $(MARK).waitForExist({ timeout: 15_000, timeoutMsg: 'no convention mark after the rule was added' })
    await expect(await $(MARK).getText()).toBe('Đảm bảo')
  })

  it('a real click on the mark opens the strip and Nhận thay writes the replacement, origin unchanged', async () => {
    const before = (await invoke('read_open_chapter_segments', {})).segments[0]
    await realClick(await $(MARK))
    const accept = await $('[data-convention-accept]')
    await accept.waitForDisplayed({ timeout: 10_000 })
    await realClick(accept)
    await waitForGridText(0, AFTER, { col: 'tgt' })
    const after = (await invoke('read_open_chapter_segments', {})).segments[0]
    await expect(after.target_text).toBe(AFTER)
    await expect(after.translation_origin).toBe(before.translation_origin)
    await expect(await browser.execute((sel) => document.querySelectorAll(sel).length, MARK)).toBe(0)
  })

  it('Hoàn lại puts the original text back byte for byte', async () => {
    const revert = await $('[data-convention-revert]')
    await revert.waitForDisplayed({ timeout: 10_000 })
    await realClick(revert)
    await waitForGridText(0, BEFORE, { col: 'tgt' })
    const after = (await invoke('read_open_chapter_segments', {})).segments[0]
    await expect(after.target_text).toBe(BEFORE)
  })
})
