/**
 * Work name auto-fill in the real webview: the overlay Name field follows the first
 * dropped file, the previews open with Name · Source language · Genre editable, and the created
 * Work carries the edited values. Drops go through `dropFiles`; a Finder drag onto the frame is
 * not covered here.
 */
import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { dropFiles, selectImportTab } from '../support/importForm.mjs'
import { realClick } from '../support/pointer.mjs'

const OPEN_BTN = '[data-library-import-open]'
const PANEL = '.li-panel'
const PRIMARY = `${PANEL} [data-import-preview-open]`
const NAME = `${PANEL} [data-import-name]`

async function ready() {
  await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
    timeout: 30_000,
    interval: 250,
  })
  await $(OPEN_BTN).waitForDisplayed({ timeout: 30_000 })
}

async function listWorks() {
  const result = await browser.execute(async () => {
    try {
      return {
        ok: true,
        value: await window.__TAURI_INTERNALS__.invoke('library_list_works', {
          filter: null,
          genre: null,
          sourceLang: null,
          sort: null,
        }),
      }
    } catch (err) {
      return { ok: false, detail: String(err && err.code ? err.code : err) }
    }
  })
  if (!result.ok) throw new Error(`library_list_works failed: ${result.detail}`)
  return result.value.works
}

async function workNamed(name) {
  await browser.waitUntil(async () => (await listWorks()).some((w) => w.name === name), {
    timeout: 20_000,
    interval: 500,
    timeoutMsg: `no Work named "${name}" in the index`,
  })
  return (await listWorks()).find((w) => w.name === name)
}

async function pickGenre(rootSelector, genreId) {
  const trigger = await $(`${rootSelector} summary`)
  await trigger.waitForDisplayed({ timeout: 10_000 })
  await realClick(trigger)
  const option = await $(`${rootSelector} .select-item:has(input[value="${genreId}"])`)
  await option.waitForDisplayed({ timeout: 10_000 })
  await realClick(option)
}

async function openOverlay() {
  await realClick(await $(OPEN_BTN))
  await $(PANEL).waitForDisplayed({ timeout: 10_000 })
}

async function enabled(selector) {
  await browser.waitUntil(async () => browser.execute((sel) => document.querySelector(sel)?.disabled === false, selector), {
    timeout: 30_000,
  })
}

describe('Story 6-26: Work name auto-fill', () => {
  const dir = mkdtempSync(join(tmpdir(), 'e2e-626-'))
  const txt = join(dir, 'tuyet-trung.txt')
  const csv = join(dir, 'song-ngu-626.csv')
  writeFileSync(txt, 'Chương một\nNội dung chương một.\n', 'utf8')
  writeFileSync(csv, 'hello,xin chao\n', 'utf8')

  it('a dropped file fills the Name field and the preview Name; Nhập creates the Work with the stem and a preview genre', async () => {
    await ready()
    await openOverlay()
    await selectImportTab('files')
    await dropFiles([txt])
    await browser.waitUntil(async () => (await $(NAME).getValue()) === 'tuyet-trung', { timeout: 10_000 })
    await realClick(await $(PRIMARY))
    await $('.ip-screen').waitForDisplayed({ timeout: 30_000 })
    expect(await $('.ip-screen [data-ip-work-name]').getValue()).toBe('tuyet-trung')

    await pickGenre('.ip-screen [data-ip-genre]', 'xianxia')
    await enabled('.ip-screen .ip-act-primary')
    await realClick(await $('.ip-screen .ip-act-primary'))
    await $('.ip-screen').waitForExist({ reverse: true, timeout: 30_000 })

    const work = await workNamed('tuyet-trung')
    expect(work.genre).toBe('xianxia')
  })

  it('a typed name survives dropping a file', async () => {
    await openOverlay()
    await selectImportTab('files')
    await (await $(NAME)).setValue('Tuyết Trung')
    await dropFiles([txt])
    await browser.waitUntil(async () => browser.execute(() => document.querySelectorAll('[data-import-file-path]').length === 1), {
      timeout: 10_000,
    })
    expect(await $(NAME).getValue()).toBe('Tuyết Trung')
    await realClick(await $(PRIMARY))
    await $('.ip-screen').waitForDisplayed({ timeout: 30_000 })
    expect(await $('.ip-screen [data-ip-work-name]').getValue()).toBe('Tuyết Trung')
    await realClick(await $('.ip-screen .ip-actions .ip-act:not(.ip-act-primary)'))
    await $('.ip-screen').waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('a .csv bilingual preview edits Name, Source language and Genre, and the Work carries all three', async () => {
    await selectImportTab('files')
    await browser.execute((sel) => {
      const field = document.querySelector(sel)
      field.value = ''
      field.dispatchEvent(new Event('input', { bubbles: true }))
    }, NAME)
    await realClick(await $(`${PANEL} [data-import-file-remove]`))
    await browser.waitUntil(async () => browser.execute(() => document.querySelectorAll('[data-import-file-path]').length === 0), {
      timeout: 10_000,
    })
    await dropFiles([csv])
    await browser.waitUntil(async () => (await $(NAME).getValue()) === 'song-ngu-626', {
      timeout: 10_000,
      timeoutMsg: 'Name field did not follow the dropped .csv',
    })
    await realClick(await $(`${PANEL} [data-import-source-lang] label:has(input[value="zh"])`))
    await realClick(await $(PRIMARY))
    await $('.bip-screen').waitForDisplayed({ timeout: 30_000 })
    expect(await $('.bip-screen [data-bip-work-name]').getValue()).toBe('song-ngu-626')

    await (await $('.bip-screen [data-bip-work-name]')).setValue('Song ngu 626 sua')
    await realClick(await $('.bip-screen [data-bip-source-lang] label:has(input[value="en"])'))
    await pickGenre('.bip-screen [data-bip-genre]', 'romance')
    await enabled('.bip-screen .bip-act-primary')
    await realClick(await $('.bip-screen .bip-act-primary'))
    await $('.bip-screen').waitForExist({ reverse: true, timeout: 30_000 })

    const work = await workNamed('Song ngu 626 sua')
    expect(work.source_lang).toBe('en')
    expect(work.genre).toBe('romance')
  })
})
