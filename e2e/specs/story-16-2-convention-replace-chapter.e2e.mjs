/**
 * Thay cả Chương: one Global rule, four AI sentences and one confirmed, the spot inside
 * `nhằm bảo đảm bảo vệ` deselected with the keyboard, the batch confirmed with the real mouse.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-16-2-convention-replace-chapter.e2e.mjs
 */
import { waitForGridRows, waitForGridText } from '../support/gridWait.mjs'
import { realClick } from '../support/pointer.mjs'
import { resetPanelState } from '../support/panelReset.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const NAME = `Story 16-2 -- replace chapter ${Date.now()}`
const AVOID = 'đảm bảo'
const REPLACEMENT = 'bảo đảm'
const MARK = '[data-col="tgt"] .convention-mark'
const ARROW_DOWN = '\uE015'
const SPACE = '\uE00D'
const SPOT = '[data-convention-replace-spot]'
const TARGETS = [
  'Đảm bảo an toàn.',
  'Hãy đảm bảo nhé.',
  'nhằm bảo đảm bảo vệ',
  'Phải đảm bảo.',
]
const EXPECTED = [
  'Bảo đảm an toàn.',
  'Hãy bảo đảm nhé.',
  'nhằm bảo đảm bảo vệ',
  'Phải bảo đảm.',
]

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

async function typeInto(selector, value) {
  const input = await $(selector)
  await realClick(input)
  await input.setValue(value)
  expect(await input.getValue()).toBe(value)
}

async function readSegments() {
  return (await invoke('read_open_chapter_segments', {})).segments
}

async function selectedFlags() {
  return browser.execute((sel) =>
    [...document.querySelectorAll(sel)].map((row) => row.getAttribute('data-selected')), SPOT)
}

describe('Story 16-2 -- replace the whole Chapter from a list', () => {
  let before

  it('four sentences are on the grid with AI translations and the last one confirmed', async () => {
    await openWorkspaceWithWork(NAME, '一。二。三。四。')
    await waitForGridRows(4, { col: 'tgt', what: 'convention grid' })
    const ids = (await readSegments()).map((s) => s.id)
    for (let i = 0; i < 4; i += 1) {
      await invoke('promote_ai_translation', { segmentId: ids[i], targetText: TARGETS[i], force: false })
    }
    await invoke('confirm_segment', { segmentId: ids[3] })
    before = await readSegments()
    await expect(before.map((s) => s.target_text)).toEqual(TARGETS)
    await expect(before[3].status).toBe('confirmed')
    await resetPanelState()
    await waitForGridText(3, TARGETS[3], { col: 'tgt' })
  })

  it('adds the rule through the overlay with the mouse and marks appear', async () => {
    await realClick(await $('.tools-trigger'))
    const item = await $('[data-tools-item="convention.manage.open"]')
    await item.waitForDisplayed({ timeout: 10_000 })
    await realClick(item)
    await $('[data-convention-add-avoid]').waitForDisplayed({ timeout: 10_000 })
    await typeInto('[data-convention-add-avoid]', AVOID)
    await typeInto('[data-convention-add-replacement]', REPLACEMENT)
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
  })

  it('the strip button opens the list with every spot selected, filter on the viewed rule', async () => {
    await realClick(await $(MARK))
    const button = await $('[data-convention-replace-chapter]')
    await button.waitForDisplayed({ timeout: 10_000 })
    await realClick(button)
    await $('[data-convention-replace]').waitForDisplayed({ timeout: 10_000 })
    await browser.waitUntil(
      async () => (await selectedFlags()).length === 4,
      { timeout: 15_000, timeoutMsg: 'the list never showed four spots' },
    )
    await expect(await selectedFlags()).toEqual(['true', 'true', 'true', 'true'])
    await expect(await $('[data-convention-replace-hidden]').isExisting()).toBe(true)
  })

  it('arrow keys and Space deselect the syllable-boundary spot', async () => {
    await browser.keys([ARROW_DOWN, ARROW_DOWN, SPACE])
    await browser.waitUntil(
      async () => (await selectedFlags()).join() === 'true,true,false,true',
      { timeout: 10_000, timeoutMsg: 'the third spot was not deselected by the keyboard' },
    )
  })

  it('a real click on the confirm button writes the other three, nothing else changes', async () => {
    await realClick(await $('[data-convention-replace-confirm]'))
    await $('[data-convention-replace-armed]').waitForDisplayed({ timeout: 10_000 })
    await realClick(await $('[data-convention-replace-confirm]'))
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('[data-convention-replace]') === null),
      { timeout: 20_000, timeoutMsg: 'the overlay did not close after the batch' },
    )
    await waitForGridText(0, EXPECTED[0], { col: 'tgt' })
    const after = await readSegments()
    await expect(after.map((s) => s.target_text)).toEqual(EXPECTED)
    await expect(after[2].target_text).toBe(before[2].target_text)
    await expect(after.map((s) => s.translation_origin)).toEqual(before.map((s) => s.translation_origin))
    await expect(after[3].status).toBe('confirmed')
    await expect(after.slice(0, 3).map((s) => s.status)).toEqual(before.slice(0, 3).map((s) => s.status))
  })

  it('the deselected spot keeps its mark', async () => {
    await browser.waitUntil(
      async () => browser.execute((sel) => document.querySelectorAll(sel).length, MARK).then((n) => n === 1),
      { timeout: 15_000, timeoutMsg: 'the deselected spot lost its mark or others kept theirs' },
    )
  })
})
