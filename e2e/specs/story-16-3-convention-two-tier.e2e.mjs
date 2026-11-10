/**
 * Two rule tiers in Quản lý quy ước: a Global rule switched off for the open Work loses its mark,
 * switched back on gets it back, and a Work rule with the same phrase replaces the Global one.
 * The off and on switches run once with the real mouse and once with the keyboard alone.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-16-3-convention-two-tier.e2e.mjs
 */
import { waitForGridRows, waitForGridText } from '../support/gridWait.mjs'
import { realClick } from '../support/pointer.mjs'
import { resetPanelState } from '../support/panelReset.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const NAME = `Story 16-3 -- two tiers ${Date.now()}`
const AVOID = 'đảm bảo'
const GLOBAL_REPLACEMENT = 'bảo đảm'
const WORK_REPLACEMENT = 'chắc chắn'
const BEFORE = 'Đảm bảo an toàn.'
const MARK = '[data-col="tgt"] .convention-mark'
const ROW = '[data-convention-rule-row]'
const PHRASE_KB = 'thập kỷ phím'
const PHRASE_MOUSE = 'thập kỷ chuột'
const SPACE = ''

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
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

async function workRules() {
  return invoke('convention_list_rules', { tier: 'work' })
}

async function markCount() {
  return browser.execute((sel) => document.querySelectorAll(sel).length, MARK)
}

async function openOverlay() {
  await realClick(await $('.tools-trigger'))
  const item = await $('[data-tools-item="convention.manage.open"]')
  await item.waitForDisplayed({ timeout: 10_000 })
  await realClick(item)
  await $('[data-convention-add-avoid]').waitForDisplayed({ timeout: 10_000 })
  await browser.waitUntil(
    async () => browser.execute((sel) => document.querySelectorAll(sel).length > 0, ROW),
    { timeout: 10_000, timeoutMsg: 'the rule rows never loaded' },
  )
}

async function closeOverlayWithEscape() {
  await browser.keys(['Escape'])
  await browser.waitUntil(
    async () => browser.execute(() => document.querySelector('[data-convention-add-avoid]') === null),
    { timeout: 10_000, timeoutMsg: 'the overlay stayed open after Escape' },
  )
}

async function focusGrid() {
  for (let i = 0; i < 12; i += 1) {
    const onGrid = await browser.execute(() => document.activeElement?.getAttribute('role') === 'grid')
    if (onGrid) return
    await browser.keys(['Tab'])
  }
  throw new Error('Tab never reached the rule list')
}

async function globalRow() {
  return $(`${ROW}[data-convention-rule-tier="global"]`)
}

describe('Story 16-3 -- Global and Work rule tiers', () => {
  it('a Global rule added through the overlay marks the AI sentence', async () => {
    await openWorkspaceWithWork(NAME, 'Bảo đảm an toàn.')
    await waitForGridRows(1, { col: 'tgt', what: 'two-tier grid' })
    const read = await invoke('read_open_chapter_segments', {})
    await invoke('promote_ai_translation', { segmentId: read.segments[0].id, targetText: BEFORE, force: false })
    await resetPanelState()
    await waitForGridText(0, BEFORE, { col: 'tgt' })

    await realClick(await $('.tools-trigger'))
    const item = await $('[data-tools-item="convention.manage.open"]')
    await item.waitForDisplayed({ timeout: 10_000 })
    await realClick(item)
    await $('[data-convention-add-avoid]').waitForDisplayed({ timeout: 10_000 })
    await typeInto('[data-convention-add-avoid]', AVOID)
    await typeInto('[data-convention-add-replacement]', GLOBAL_REPLACEMENT)
    await pickAddTier('global')
    await realClick(await $('[data-convention-add]'))
    await browser.waitUntil(
      async () => browser.execute((sel) => document.querySelectorAll(sel).length === 1, ROW),
      { timeout: 10_000, timeoutMsg: 'the rule row never appeared' },
    )
    await closeOverlayWithEscape()
    await $(MARK).waitForExist({ timeout: 15_000, timeoutMsg: 'no mark after the Global rule was added' })
  })

  it('with the mouse: Tắt ở Tác phẩm này removes the mark and Bật lại brings it back', async () => {
    await openOverlay()
    await realClick(await globalRow())
    await realClick(await $('[data-convention-disable]'))
    await $('[data-convention-enable]').waitForDisplayed({ timeout: 10_000 })
    await closeOverlayWithEscape()
    await browser.waitUntil(async () => (await markCount()) === 0, {
      timeout: 15_000,
      timeoutMsg: 'the mark stayed after the rule was switched off',
    })

    await openOverlay()
    await realClick(await globalRow())
    await realClick(await $('[data-convention-enable]'))
    await $('[data-convention-disable]').waitForDisplayed({ timeout: 10_000 })
    await closeOverlayWithEscape()
    await $(MARK).waitForExist({ timeout: 15_000, timeoutMsg: 'the mark did not come back' })
  })

  it('with the keyboard alone: Space switches the rule off and on again', async () => {
    await openOverlay()
    await focusGrid()
    await browser.keys([SPACE])
    await $('[data-convention-enable]').waitForDisplayed({ timeout: 10_000 })
    await browser.keys([SPACE])
    await $('[data-convention-disable]').waitForDisplayed({ timeout: 10_000 })
    await browser.keys([SPACE])
    await $('[data-convention-enable]').waitForDisplayed({ timeout: 10_000 })
    await closeOverlayWithEscape()
    await browser.waitUntil(async () => (await markCount()) === 0, {
      timeout: 15_000,
      timeoutMsg: 'the mark stayed after the keyboard switch-off',
    })
    await openOverlay()
    await focusGrid()
    await browser.keys([SPACE])
    await $('[data-convention-disable]').waitForDisplayed({ timeout: 10_000 })
    await closeOverlayWithEscape()
    await $(MARK).waitForExist({ timeout: 15_000, timeoutMsg: 'the mark did not come back from the keyboard' })
  })

  it('a Work rule with the same phrase replaces the Global one, and Esc twice closes a pending delete then the overlay', async () => {
    await openOverlay()
    await typeInto('[data-convention-add-avoid]', AVOID)
    await typeInto('[data-convention-add-replacement]', WORK_REPLACEMENT)
    await pickAddTier('work')
    await realClick(await $('[data-convention-add]'))
    await browser.waitUntil(
      async () => browser.execute((sel) => document.querySelectorAll(sel).length === 2, ROW),
      { timeout: 10_000, timeoutMsg: 'the Work row never appeared' },
    )
    const shadowed = await browser.execute(
      () => document.querySelectorAll('[data-convention-rule-status="shadowed"]').length,
    )
    await expect(shadowed).toBe(1)
    const scan = await invoke('convention_scan_chapter', { generation: 900 })
    await expect(scan.segments[0].spans[0].tier).toBe('work')
    await expect(scan.segments[0].spans[0].replacement_cased).toBe('Chắc chắn')

    await realClick(await $(`${ROW}[data-convention-rule-tier="work"]`))
    await realClick(await $('[data-convention-delete]'))
    await expect(await $('.cvm-row-delete-pending').isExisting()).toBe(true)
    await browser.keys(['Escape'])
    await browser.waitUntil(async () => !(await $('.cvm-row-delete-pending').isExisting()), {
      timeout: 5_000,
      timeoutMsg: 'Escape did not cancel the pending delete',
    })
    await expect(await $('[data-convention-add-avoid]').isExisting()).toBe(true)
    await closeOverlayWithEscape()
  })

  it('Esc closes the overlay and focus returns to the opener', async () => {
    await openOverlay()
    await closeOverlayWithEscape()
    const onOpener = await browser.execute(
      () => document.activeElement?.matches('[data-convention-manage-open]') === true,
    )
    await expect(onOpener).toBe(true)
  })

  it('with the keyboard alone: add, edit and delete a Work rule', async () => {
    await openOverlay()
    await typeInto('[data-convention-add-avoid]', PHRASE_KB)
    await $('[data-convention-add-replacement]').setValue('thập niên')
    await browser.keys(['Enter'])
    await browser.waitUntil(async () => (await workRules()).some((r) => r.avoid_phrase === PHRASE_KB), {
      timeout: 10_000,
      timeoutMsg: 'Enter in the add field did not add the rule',
    })
    await typeInto('[data-convention-search]', PHRASE_KB)
    await focusGrid()
    await browser.keys(['Enter'])
    const edit = await $('[data-convention-edit-replacement]')
    await edit.waitForDisplayed({ timeout: 10_000 })
    await edit.setValue('mười năm')
    await browser.keys(['Enter'])
    await browser.waitUntil(
      async () => (await workRules()).some((r) => r.avoid_phrase === PHRASE_KB && r.replacement_phrase === 'mười năm'),
      { timeout: 10_000, timeoutMsg: 'Enter in the edit field did not save' },
    )
    await focusGrid()
    await browser.keys(['Delete'])
    await $('.cvm-row-delete-pending').waitForExist({ timeout: 5_000 })
    await expect((await workRules()).some((r) => r.avoid_phrase === PHRASE_KB)).toBe(true)
    await browser.keys(['Delete'])
    await browser.waitUntil(async () => !(await workRules()).some((r) => r.avoid_phrase === PHRASE_KB), {
      timeout: 10_000,
      timeoutMsg: 'the second Delete did not remove the rule',
    })
    await closeOverlayWithEscape()
  })

  it('with the mouse alone: add, edit and delete a Work rule', async () => {
    await openOverlay()
    await typeInto('[data-convention-add-avoid]', PHRASE_MOUSE)
    await typeInto('[data-convention-add-replacement]', 'thế kỷ')
    await realClick(await $('[data-convention-add]'))
    await browser.waitUntil(async () => (await workRules()).some((r) => r.avoid_phrase === PHRASE_MOUSE), {
      timeout: 10_000,
      timeoutMsg: 'the Add button did not add the rule',
    })
    await typeInto('[data-convention-search]', PHRASE_MOUSE)
    const row = await $(ROW)
    await realClick(row)
    await realClick(await row.$('[data-convention-edit]'))
    const edit = await $('[data-convention-edit-replacement]')
    await edit.waitForDisplayed({ timeout: 10_000 })
    await edit.setValue('thiên niên kỷ')
    await realClick(await $('[data-convention-save]'))
    await browser.waitUntil(
      async () =>
        (await workRules()).some((r) => r.avoid_phrase === PHRASE_MOUSE && r.replacement_phrase === 'thiên niên kỷ'),
      { timeout: 10_000, timeoutMsg: 'the Save button did not save' },
    )
    const target = await $(ROW)
    await realClick(await target.$('[data-convention-delete]'))
    await $('.cvm-row-delete-pending').waitForExist({ timeout: 5_000 })
    await realClick(await (await $(ROW)).$('[data-convention-delete]'))
    await browser.waitUntil(async () => !(await workRules()).some((r) => r.avoid_phrase === PHRASE_MOUSE), {
      timeout: 10_000,
      timeoutMsg: 'the second click did not remove the rule',
    })
    await closeOverlayWithEscape()
  })
})
