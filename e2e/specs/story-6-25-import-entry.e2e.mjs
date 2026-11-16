/**
 * Import entry (Story 6.25) in the real webview: one common block, three source tabs, one
 * primary button, a classified file list, and the last-used tab for the session. Drops go
 * through the `aura://file-dropped` listener that Rust feeds (`dropFiles`), so the
 * classification behind them is the real one. A Finder drag onto the overlay frame is the
 * debt item `Chủ: Epic 6`.
 */
import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { dropFiles, selectImportTab, waitLibraryReady } from '../support/importForm.mjs'
import { realClick } from '../support/pointer.mjs'

const OPEN_BTN = '[data-library-import-open]'
const PANEL = '.li-panel'
const PRIMARY = `${PANEL} [data-import-preview-open]`

async function rowProblems() {
  return browser.execute(() =>
    Array.from(document.querySelectorAll('[data-import-file-path]')).map((row) => ({
      name: (row.querySelector('.li-file-name')?.textContent || '').trim(),
      kind: row.getAttribute('data-import-file-kind'),
      problem: row.getAttribute('data-import-file-problem'),
    })),
  )
}

async function primaryState() {
  return browser.execute((sel) => {
    const button = document.querySelector(sel)
    return { disabled: button.disabled, label: (button.textContent || '').trim() }
  }, PRIMARY)
}

describe('Story 6-25: import entry', () => {
  const dir = mkdtempSync(join(tmpdir(), 'e2e-625-'))
  const files = {}
  for (const name of ['a.txt', 'b.txt', 'c.csv', 'd.pdf']) {
    files[name] = join(dir, name)
    writeFileSync(files[name], name.endsWith('.csv') ? 'a,b\n' : 'x\n', 'utf8')
  }

  it('opens on Dán with no website button, and the URL tab shows the counter, the promise and "Tải 3 link"', async () => {
    await waitLibraryReady()
    await realClick(await $(OPEN_BTN))
    await $(PANEL).waitForDisplayed({ timeout: 10_000 })
    expect(await $(`${PANEL} [data-import-tab="paste"]`).getAttribute('aria-selected')).toBe('true')
    expect(await $('[data-library-url-screen-open]').isExisting()).toBe(false)

    await (await $(`${PANEL} .import-form input[type="text"]`)).setValue('e2e-625-work')
    const panel = await selectImportTab('urls')
    await (await panel.$('textarea')).setValue('https://a.example/1\nhttps://a.example/2\nhttps://b.example/3')
    const text = await browser.execute(() => ({
      count: (document.querySelector('[data-import-url-count]')?.textContent || '').trim(),
      promise: (document.querySelector('[data-import-tabpanel="urls"] .li-note:last-child')?.textContent || '').trim(),
    }))
    expect(text.count).toBe('3 link · sẽ tạo 3 Chương · 2 domain')
    expect(text.promise).toContain('Chỉ tải đúng 3 link này.')
    expect((await primaryState()).label).toContain('Tải 3 link')
  })

  it('2 .txt + 1 .csv: the .csv row is rejected, the button is locked, and removing it unlocks the button', async () => {
    await selectImportTab('files')
    await dropFiles([files['a.txt'], files['b.txt'], files['c.csv']])
    await browser.waitUntil(async () => (await rowProblems()).length === 3, { timeout: 10_000 })

    const rows = await rowProblems()
    expect(rows.map((r) => r.kind)).toEqual(['text', 'text', 'bilingual'])
    expect(rows[2].problem).toBe('kind_mismatch')
    expect((await primaryState()).disabled).toBe(true)
    expect(await $('[data-import-files-lock]').isDisplayed()).toBe(true)

    const remove = await $(`[data-import-file-path$="c.csv"] [data-import-file-remove]`)
    await realClick(remove)
    await browser.waitUntil(async () => (await rowProblems()).length === 2, { timeout: 10_000 })
    expect((await primaryState()).disabled).toBe(false)
  })

  it('a dropped .pdf is rejected with a reason until it is removed', async () => {
    await dropFiles([files['d.pdf']])
    await browser.waitUntil(async () => (await rowProblems()).length === 3, { timeout: 10_000 })
    expect((await rowProblems())[2].problem).toBe('unsupported')
    expect((await primaryState()).disabled).toBe(true)

    await realClick(await $(`[data-import-file-path$="d.pdf"] [data-import-file-remove]`))
    await browser.waitUntil(async () => (await rowProblems()).length === 2, { timeout: 10_000 })
    expect((await primaryState()).disabled).toBe(false)
  })

  it('switching tabs keeps the other tab, and the last-used tab survives a close and a reopen in the session', async () => {
    await selectImportTab('urls')
    expect(await browser.execute(() => document.querySelector('[data-import-tabpanel="urls"] textarea').value)).toContain(
      'https://a.example/1',
    )
    await selectImportTab('files')
    expect((await rowProblems()).length).toBe(2)

    await realClick(await $(`${PANEL} .li-close`))
    await $(PANEL).waitForExist({ reverse: true, timeout: 10_000 })
    await realClick(await $(OPEN_BTN))
    await $(PANEL).waitForDisplayed({ timeout: 10_000 })
    expect(await $(`${PANEL} [data-import-tab="files"]`).getAttribute('aria-selected')).toBe('true')
  })

  it('the primary button opens the text preview of the two remaining .txt files', async () => {
    await realClick(await $(PRIMARY))
    await $('.ip-screen').waitForDisplayed({ timeout: 30_000 })
    await realClick(await $('.ip-screen .ip-actions .ip-act:not(.ip-act-primary)'))
    await $('.ip-screen').waitForExist({ reverse: true, timeout: 10_000 })
  })
})
