/**
 * Export screen content options in a real webview: open from the Work page action bar, switch the
 * three content toggles and the format, watch the preview follow, close and check focus.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-17-export-content-options.e2e.mjs
 *
 * The folder picker and the write itself start in a native dialog that WebDriver cannot drive; the
 * file contents are covered by `export_content_options_contract.rs` and the wording by
 * `exportContent.test.ts`.
 */
import { realClick } from '../support/pointer.mjs'

const WORK_NAME = `e2e-export-options-${Date.now() % 1_000_000}`
const SOURCE_TEXT = 'Cau mot。Cau hai。Cau ba。'

const LIST_WORKS_BTN = '[data-lifecycle-action="list_works"]'
const WORK_NEXT_BTN = '[data-library-work-next]'
const OPEN_WORK_BTN = '[data-library-open-work]'
const WORK_PAGE = '[data-library-work-page]'
const ROW = '[data-library-chapters-list] [data-library-chapter-row]'
const EXPORT_OPEN = '[data-export-open]'
const PANEL = '.ex-panel'

async function ipcValue(command, args = {}) {
  const json = await browser.execute(
    async (name, payload) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) return JSON.stringify({ ok: false, error: 'no IPC bridge' })
      try {
        return JSON.stringify({ ok: true, value: await internals.invoke(name, payload) })
      } catch (err) {
        return JSON.stringify({ ok: false, error: String(err && err.code ? err.code : err) })
      }
    },
    command,
    args,
  )
  const out = JSON.parse(json)
  if (!out.ok) throw new Error(`fixture: ${command} failed: ${out.error}`)
  return out.value
}

async function previewState() {
  return browser.execute(() => ({
    wordTable: document.querySelector('[data-export-word-preview]') !== null,
    textBlock: document.querySelector('[data-export-text-preview]') !== null,
    rows: document.querySelectorAll('[data-export-preview-row]').length,
    text: (document.querySelector('.ex-panel')?.textContent ?? '').replace(/\s+/g, ' '),
    fileName: (document.querySelector('[data-export-file-name]')?.textContent ?? '').trim(),
    notReimportable: document.querySelector('[data-export-not-reimportable][role="status"]') !== null,
    markDisabledNote: document.querySelector('[data-export-mark-disabled]') !== null,
  }))
}

async function checked(selector) {
  return browser.execute((sel) => document.querySelector(sel)?.checked === true, selector)
}

async function openWorkPage() {
  await realClick(await $(LIST_WORKS_BTN))
  const next = await $(WORK_NEXT_BTN)
  await browser.waitUntil(
    async () => {
      const current = await browser.execute(() => {
        const name = document.querySelector('.works-grid .work-cell[aria-current="true"] .work-name')
        return name === null ? '' : (name.textContent || '').trim()
      })
      if (current === WORK_NAME) return true
      await realClick(next)
      return false
    },
    { timeout: 30_000, timeoutMsg: `the grid cursor never reached "${WORK_NAME}"` },
  )
  await realClick(await $(OPEN_WORK_BTN))
  await $(WORK_PAGE).waitForExist({ timeout: 20_000 })
  await $(ROW).waitForExist({ timeout: 20_000 })
}

describe('Story 12-17 -- export content options', () => {
  it('opens from the action bar, shows the content fieldset with every option off, and a two-column preview', async () => {
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
      timeout: 30_000,
      interval: 250,
    })
    await ipcValue('create_work_from_text', { name: WORK_NAME, sourceLang: 'zh', genre: 'general', text: SOURCE_TEXT })
    await openWorkPage()
    await realClick(await $(ROW))
    await $(`${EXPORT_OPEN}:not([disabled])`).waitForExist({ timeout: 10_000 })
    await realClick(await $(EXPORT_OPEN))
    await $(PANEL).waitForDisplayed({ timeout: 10_000 })
    await $('[data-export-content]').waitForDisplayed({ timeout: 10_000 })
    expect(await checked('[data-export-number-toggle]')).toBe(false)
    expect(await checked('[data-export-appendix-toggle]')).toBe(false)
    expect(await checked('[data-export-mark-toggle]')).toBe(false)
    await browser.waitUntil(async () => (await previewState()).wordTable, {
      timeout: 15_000,
      timeoutMsg: 'the two-column preview never appeared',
    })
    expect((await previewState()).notReimportable).toBe(false)
  })

  it('number and mark toggles change the preview and warn that the file cannot be imported again', async () => {
    const before = (await previewState()).text
    await realClick(await $('[data-export-number-toggle]'))
    await browser.waitUntil(async () => (await previewState()).text.includes('[1]'), {
      timeout: 15_000,
      timeoutMsg: 'the preview did not gain the sentence number prefix',
    })
    expect((await previewState()).text).not.toBe(before)
    expect((await previewState()).notReimportable).toBe(true)

    await realClick(await $('[data-export-mark-toggle]'))
    await browser.waitUntil(async () => (await previewState()).text.includes('câu chưa xác nhận —'), {
      timeout: 15_000,
      timeoutMsg: 'the preview did not gain the unconfirmed mark',
    })
    await realClick(await $('[data-export-number-toggle]'))
    expect((await previewState()).notReimportable).toBe(true)
    await realClick(await $('[data-export-mark-toggle]'))
    await browser.waitUntil(async () => !(await previewState()).notReimportable, {
      timeout: 15_000,
      timeoutMsg: 'the warning stayed after both toggles were switched off',
    })
  })

  it('switching to Markdown turns the preview into a text block, disables the mark and renames the file', async () => {
    const twoColumnName = (await previewState()).fileName
    const radios = await $$('input[name="export-format"]')
    await realClick(radios[2])
    await browser.waitUntil(async () => (await previewState()).textBlock, {
      timeout: 15_000,
      timeoutMsg: 'the Markdown preview block never appeared',
    })
    const state = await previewState()
    expect(state.wordTable).toBe(false)
    expect(state.markDisabledNote).toBe(true)
    expect(state.fileName).not.toBe(twoColumnName)
    expect(state.fileName).toContain('.md')
  })

  it('Escape closes the screen and focus returns to the Export button', async () => {
    await browser.keys(['Escape'])
    await $(PANEL).waitForExist({ reverse: true, timeout: 10_000 })
    expect(await browser.execute(() => document.activeElement?.hasAttribute('data-export-open') ?? false)).toBe(true)
  })
})
