/**
 * "Không phải lỗi" on a real WKWebView: the strip button with a real mouse and with the keyboard
 * alone, and the Không phải lỗi card of Quản lý quy ước, whose rows select on `pointerdown` and
 * whose list has the overlay's arrow/Space handler switched off.
 *
 * There is no by-keyboard case: `e2e/AGENTS.md` records that keyboard activation of a
 * <button> does not work in this harness. Space on the focused button and Tab into the strip were
 * both tried and neither lands.
 *
 * Persistence across closing and reopening the Work is proven by
 * `convention_contract::a_remembered_spot_stays_hidden_after_the_work_is_closed_and_opened_again`;
 * what only a real webview can show is focus, pointer and the WebKit button behaviour.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-16-5-convention-not-an-error.e2e.mjs
 */
import { waitForGridRows, waitForGridText } from '../support/gridWait.mjs'
import { realClick } from '../support/pointer.mjs'
import { resetPanelState } from '../support/panelReset.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const NAME = `Story 16-5 -- not an error ${Date.now()}`
const AVOID = 'đảm bảo'
const MATCHED = 'Đảm bảo'
const REPLACEMENT = 'bảo đảm'
const FIRST = 'Đảm bảo an toàn.'
const SECOND = 'Cần đảm bảo thêm.'
const MARK = '[data-col="tgt"] .convention-mark'
const SPACE = ''

let segmentIds = []

function cell(id) {
  return `[data-col="tgt"][data-segment-id="${id}"]`
}

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

async function markCount() {
  return browser.execute((sel) => document.querySelectorAll(sel).length, MARK)
}

async function remembered() {
  return invoke('convention_ignore_list', {})
}

async function openOverlay() {
  await realClick(await $('.tools-trigger'))
  const item = await $('[data-tools-item="convention.manage.open"]')
  await item.waitForDisplayed({ timeout: 10_000 })
  await realClick(item)
  await $('[data-convention-add-avoid]').waitForDisplayed({ timeout: 10_000 })
}

// A test that aborts mid-flow can leave the overlay or the Công cụ menu up, and the next one
// needs the grid. Clicking the trigger of an OPEN menu closes it, so the menu has to go first.
async function ensureOverlayClosed() {
  for (let i = 0; i < 3; i += 1) {
    const up = await browser.execute(
      () =>
        document.querySelector('[data-convention-add-avoid]') !== null ||
        document.querySelector('[data-tools-item="convention.manage.open"]') !== null,
    )
    if (!up) return
    await browser.keys(['Escape'])
    await browser.pause(300)
  }
  throw new Error('an overlay or the Công cụ menu would not close')
}

async function closeOverlayWithEscape() {
  await browser.keys(['Escape'])
  await browser.waitUntil(
    async () => browser.execute(() => document.querySelector('[data-convention-add-avoid]') === null),
    { timeout: 10_000, timeoutMsg: 'the overlay stayed open after Escape' },
  )
}

async function openIgnoredCard() {
  await openOverlay()
  await realClick(await $('[data-convention-tab="ignored"]'))
  await $('[data-convention-ignore-card]').waitForDisplayed({ timeout: 10_000 })
}

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

describe('Story 16-5 -- "không phải lỗi" remembered per Work', () => {
  it('two AI sentences with the same phrase both get a mark', async () => {
    // `。` and not a full stop: the splitter reads Chinese sentence marks, so a Vietnamese
    // string with dots would arrive as ONE segment.
    await openWorkspaceWithWork(NAME, '一。二。')
    await waitForGridRows(2, { col: 'tgt', what: 'not-an-error grid' })
    const read = await invoke('read_open_chapter_segments', {})
    segmentIds = read.segments.map((s) => s.id)
    await invoke('promote_ai_translation', { segmentId: read.segments[0].id, targetText: FIRST, force: false })
    await invoke('promote_ai_translation', { segmentId: read.segments[1].id, targetText: SECOND, force: false })
    await resetPanelState()
    await waitForGridText(0, FIRST, { col: 'tgt' })
    await waitForGridText(1, SECOND, { col: 'tgt' })

    await openOverlay()
    await typeInto('[data-convention-add-avoid]', AVOID)
    await typeInto('[data-convention-add-replacement]', REPLACEMENT)
    await pickAddTier('global')
    await realClick(await $('[data-convention-add]'))
    await closeOverlayWithEscape()
    await browser.waitUntil(async () => (await markCount()) === 2, {
      timeout: 15_000,
      timeoutMsg: 'both sentences should carry a mark',
    })
  })

  beforeEach(async () => {
    await ensureOverlayClosed()
  })

  it('with the real mouse: the strip button remembers the place and only that mark goes', async () => {
    await realClick(await $(MARK))
    const button = await $('[data-convention-ignore]')
    await button.waitForDisplayed({ timeout: 10_000 })
    await realClick(button)
    await browser.waitUntil(async () => (await markCount()) === 1, {
      timeout: 15_000,
      timeoutMsg: 'the clicked mark should go and the other should stay',
    })
    const rows = await remembered()
    expect(rows).toHaveLength(1)
    // The stored phrase is the MATCHED text with its own case, not the rule's lowercase key: the
    // key already carries the case-insensitive match, and the card draws this.
    expect(rows[0].avoid_phrase).toBe(MATCHED)
    expect(rows[0].rule_key).toBe(AVOID)
    // WebKit must not hand the button focus: that is why the actions wrapper prevents mousedown,
    // and without it focusout would drop the caret and take the strip away before the click landed.
    const onButton = await browser.execute(
      () => document.activeElement?.hasAttribute('data-convention-ignore') === true,
    )
    expect(onButton).toBe(false)
  })

  it('the Không phải lỗi card lists the place and Bỏ ghi nhớ brings its mark back', async () => {
    await openIgnoredCard()
    const row = await $('[data-convention-ignore-row]')
    await row.waitForDisplayed({ timeout: 10_000 })
    expect(await (await $('[data-convention-ignore-row]')).getText()).toContain(`[${MATCHED}]`)
    // Selecting happens on pointerdown and `aria-current` is how that shows, so these two pin
    // which half is at fault when the forget does not land.
    await realClick(row)
    expect(await row.getAttribute('aria-current')).toBe('true')
    const forget = await $('[data-convention-ignore-forget]')
    expect(await forget.isEnabled()).toBe(true)
    await realClick(forget)
    await browser.waitUntil(async () => (await remembered()).length === 0, {
      timeout: 15_000,
      timeoutMsg: 'Bỏ ghi nhớ should drop the selected row',
    })
    const doneShown = await browser.execute(
      () => document.querySelector('[data-convention-ignore-done]') !== null,
    )
    expect(doneShown).toBe(true)
    await closeOverlayWithEscape()
    await browser.waitUntil(async () => (await markCount()) === 2, {
      timeout: 15_000,
      timeoutMsg: 'forgetting should let the next scan mark the place again',
    })
  })
})
