/**
 * Glossary manager, TM manager and "Replace the whole Chapter" run on the shared overlay
 * frame: 1120px at a 1280px window and exactly one vertical scroller inside the frame.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-17-3-glossary-tm-quy-uoc.e2e.mjs
 */
import { waitForGridRows, waitForGridText } from '../support/gridWait.mjs'
import { realClick } from '../support/pointer.mjs'
import { resetPanelState } from '../support/panelReset.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { isInViewport, measureOverlayFrame, resizeLogical } from '../support/overlayFrame.mjs'

const NAME = `Story 17-3 -- frame ${Date.now()}`
const AVOID = 'đảm bảo'
const REPLACEMENT = 'bảo đảm'
const MARK = '[data-col="tgt"] .convention-mark'
const WIDTH_TOLERANCE_PX = 1
const TARGETS = ['Đảm bảo an toàn.', 'Hãy đảm bảo nhé.', 'Phải đảm bảo.']

async function invoke(command, args) {
  return browser.execute(async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a), command, args)
}

const expectedWidth = async () => Math.min(1120, (await browser.execute(() => window.innerWidth)) - 28)

async function expectOneRegion(selector) {
  const frame = await measureOverlayFrame(selector)
  await expect(Math.abs(frame.width - (await expectedWidth()))).toBeLessThanOrEqual(WIDTH_TOLERANCE_PX)
  await expect(frame.nested).toBe(0)
  await expect(frame.scrollers.length).toBe(1)
  await expect(frame.scrollers[0]).toContain('ui-overlay__body')
}

async function openFromTools(item) {
  await realClick(await $('.tools-trigger'))
  const entry = await $(`[data-tools-item="${item}"]`)
  await entry.waitForDisplayed({ timeout: 10_000 })
  await realClick(entry)
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

describe('Story 17-3 -- Glossary, TM and convention overlays on the shared frame', () => {
  it('sets up a Work with three AI sentences and a window at 1280px', async () => {
    await openWorkspaceWithWork(NAME, '一。二。三。')
    await waitForGridRows(3, { col: 'tgt', what: 'frame grid' })
    const ids = (await invoke('read_open_chapter_segments', {})).segments.map((s) => s.id)
    for (let i = 0; i < 3; i += 1) {
      await invoke('promote_ai_translation', { segmentId: ids[i], targetText: TARGETS[i], force: false })
    }
    await resetPanelState()
    await waitForGridText(2, TARGETS[2], { col: 'tgt' })
    await resizeLogical(1280, 720)
  })

  it('Glossary manager is one width with one scroll region', async () => {
    await openFromTools('glossary.manage.open')
    await $('.gm-panel').waitForDisplayed({ timeout: 10_000 })
    await expectOneRegion('.gm-panel')
    await browser.keys(['Escape'])
    await $('.gm-panel').waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('TM manager is one width with one scroll region', async () => {
    await browser.keys(['Meta', 'Alt', 'b'])
    await $('.tm-panel').waitForDisplayed({ timeout: 15_000 })
    await expectOneRegion('.tm-panel')
    await browser.keys(['Escape'])
    await $('.tm-panel').waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('Replace the whole Chapter is one width with one scroll region and its foot in view', async () => {
    await openFromTools('convention.manage.open')
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
    await $(MARK).waitForExist({ timeout: 15_000, timeoutMsg: 'no convention mark after the rule was added' })

    await realClick(await $(MARK))
    const button = await $('[data-convention-replace-chapter]')
    await button.waitForDisplayed({ timeout: 10_000 })
    await realClick(button)
    await $('[data-convention-replace]').waitForDisplayed({ timeout: 10_000 })
    await $('[data-convention-replace-spot]').waitForExist({ timeout: 15_000 })
    await expectOneRegion('.crv-panel')
    await expect(await isInViewport(await $('[data-convention-replace-confirm]'))).toBe(true)

    await browser.execute(() => {
      document.querySelector('.crv-panel').style.maxHeight = '280px'
    })
    const overflow = await browser.execute(() => {
      const body = document.querySelector('.crv-panel .ui-overlay__body')
      return { overflowing: body.scrollHeight > body.clientHeight + 1, top: body.scrollTop }
    })
    await expect(overflow.overflowing).toBe(true)
    await expect(overflow.top).toBe(0)
    await expect(await isInViewport(await $('[data-convention-replace-confirm]'))).toBe(true)
    await expectOneRegion('.crv-panel')

    const spots = await $$('[data-convention-replace-spot]')
    await browser.execute(() => document.querySelector('[data-convention-replace-list]').focus())
    for (let i = 0; i < spots.length; i += 1) await browser.keys(['ArrowDown'])
    const after = await browser.execute(() => {
      const body = document.querySelector('.crv-panel .ui-overlay__body')
      const rows = document.querySelectorAll('[data-convention-replace-spot]')
      const last = rows[rows.length - 1].getBoundingClientRect()
      const box = body.getBoundingClientRect()
      return { scrolled: body.scrollTop > 0, lastInBody: last.top >= box.top - 1 && last.bottom <= box.bottom + 1 }
    })
    await expect(after.scrolled).toBe(true)
    await expect(after.lastInBody).toBe(true)
    await expect(await isInViewport(await $('[data-convention-replace-confirm]'))).toBe(true)
    await browser.execute(() => {
      document.querySelector('.crv-panel').style.maxHeight = ''
    })
    await browser.keys(['Escape'])
    await $('[data-convention-replace]').waitForExist({ reverse: true, timeout: 10_000 })
  })
})
