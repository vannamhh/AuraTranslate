/**
 * Thêm Chương… in the real webview: the Work card's `⋯` menu opens the import overlay with
 * Đưa vào = that Work and the three fields locked to its values.
 */
import { createWorkThroughForm, waitLibraryReady } from '../support/importForm.mjs'
import { aimWork, openAimedWork } from '../support/libraryGrid.mjs'
import { realClick } from '../support/pointer.mjs'

const OPEN_BTN = '[data-library-import-open]'
const PANEL = '.li-panel'

// The paste helper creates a Work with no genre; the overlay keeps a picked genre and language across a close.
async function pickGenreThenClose() {
  await realClick(await $(OPEN_BTN))
  await $(PANEL).waitForDisplayed({ timeout: 10_000 })
  await realClick(await $(`${PANEL} [data-import-genre] summary`))
  const value = await browser.execute(
    (sel) => Array.from(document.querySelectorAll(sel)).map((input) => input.value).find((v) => v !== '') ?? null,
    `${PANEL} [data-import-genre] input[data-select-option]`,
  )
  if (value === null) throw new Error('no genre option with a value')
  const label = await $(`${PANEL} [data-import-genre] input[data-select-option][value="${value}"]`).$('..')
  const text = (await label.getText()).trim()
  await realClick(label)
  const chip = await $(`${PANEL} [data-import-source-lang] .lang-chip:has(input:not([value="auto"]))`)
  const lang = (await chip.getText()).trim()
  await realClick(chip)
  await realClick(await $(`${PANEL} .li-close`))
  await $(PANEL).waitForExist({ reverse: true, timeout: 10_000 })
  return { genre: text, lang }
}

async function lockedState() {
  return browser.execute(() => {
    const existing = document.querySelector('input[name="import-destination-mode"][value="existing"]')
    const picker = document.querySelector('[data-import-destination-picker]')
    const locked = Array.from(document.querySelectorAll('.li-panel .import-form input[type="text"][disabled]'))
    return {
      existingChecked: existing?.checked === true,
      pickerText: picker?.textContent ?? '',
      lockedValues: locked.map((f) => f.value),
      editable: ['[data-import-name]', '[data-import-source-lang]', '[data-import-genre]'].filter(
        (sel) => document.querySelector(sel) !== null,
      ),
    }
  })
}

function expectLocked(state, expected) {
  expect(state.existingChecked).toBe(true)
  expect(state.pickerText).toContain('Kiếm Lai 627')
  expect(state.editable).toEqual([])
  expect(state.lockedValues).toHaveLength(3)
  expect(state.lockedValues[0]).toBe('Kiếm Lai 627')
  expect(state.lockedValues[1]).toBe(expected.lang)
  expect(state.lockedValues[2]).toBe(expected.genre)
}

describe('Story 6-27: Thêm Chương… on the Work card', () => {
  let picked

  it('opens the overlay on Tác phẩm đã có with the Work picked and its fields disabled', async () => {
    await waitLibraryReady()
    picked = await pickGenreThenClose()
    await createWorkThroughForm('Kiếm Lai 627')
    await $('[data-library-work-name]').waitForDisplayed({ timeout: 30_000 })

    const card = await $('[data-library-work-more]')
    await card.waitForDisplayed({ timeout: 10_000 })
    await realClick(await card.$('summary, button'))
    const item = await $('[data-library-work-add-chapters]')
    await item.waitForDisplayed({ timeout: 10_000 })
    await realClick(item)
    await $(PANEL).waitForDisplayed({ timeout: 10_000 })

    expectLocked(await lockedState(), picked)
  })

  it('the Work page bar button opens the same overlay', async () => {
    await realClick(await $(`${PANEL} .li-close`))
    await $(PANEL).waitForExist({ reverse: true, timeout: 10_000 })
    await aimWork('Kiếm Lai 627')
    await openAimedWork()
    await $('[data-library-work-page]').waitForExist({ timeout: 20_000 })
    const button = await $('[data-library-chapters-add]')
    await button.waitForDisplayed({ timeout: 10_000 })
    await realClick(button)
    await $(PANEL).waitForDisplayed({ timeout: 10_000 })
    expectLocked(await lockedState(), picked)
  })
})
