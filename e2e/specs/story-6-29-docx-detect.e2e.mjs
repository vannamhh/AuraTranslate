/**
 * A lone .docx dropped on the Files tab opens the preview of the shape Rust detected from its
 * content, and the Đọc như … switch flips between the text and bilingual previews without
 * choosing the file again.
 */
import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { backToLibrary, dropFiles, selectImportTab, waitLibraryReady } from '../support/importForm.mjs'
import { realClick } from '../support/pointer.mjs'
import { buildDocxWithTable, buildPlainDocx } from '../support/miniDocx.mjs'

const OPEN_BTN = '[data-library-import-open]'
const PANEL = '.li-panel'
const TEXT_SCREEN = '.ip-screen'
const BILINGUAL_SCREEN = '.bip-screen'
const dir = mkdtempSync(join(tmpdir(), 'e2e-629-'))

async function openFilesTabWith(file, bytes) {
  const path = join(dir, file)
  writeFileSync(path, bytes)
  await realClick(await $(OPEN_BTN))
  await $(PANEL).waitForDisplayed({ timeout: 10_000 })
  await selectImportTab('files')
  await dropFiles([path])
  await browser.waitUntil(async () => browser.execute(() => document.querySelectorAll('[data-import-file-path]').length === 1), {
    timeout: 10_000,
  })
}

async function pickEnglish() {
  await realClick(await $(`${PANEL} [data-import-source-lang] .lang-chip:has(input[value="en"])`))
}

async function flipSwitchTo(screenPrefix, shape) {
  await realClick(await $(`[data-${screenPrefix}-docx-shape] summary`))
  const option = await $(`[data-${screenPrefix}-docx-shape] input[data-docx-shape-option][value="${shape}"]`)
  await realClick(await option.$('..'))
}

describe('Story 6-29: a lone .docx is detected when the preview opens', () => {
  it('a plain .docx opens the text preview, the checkbox is gone, and a refused switch keeps it', async () => {
    await waitLibraryReady()
    await openFilesTabWith('van-ban.docx', buildPlainDocx())
    const row = await $('[data-import-file-path]')
    expect(await row.getText()).toContain('.docx, dò khi xem trước')
    expect(await browser.execute(() => document.querySelector('[data-import-file-path] input[type="checkbox"]') === null)).toBe(true)

    await pickEnglish()
    await realClick(await $(`${PANEL} [data-import-preview-open]`))
    await $(TEXT_SCREEN).waitForDisplayed({ timeout: 30_000 })
    await $('[data-ip-docx-shape]').waitForDisplayed({ timeout: 10_000 })

    await flipSwitchTo('ip', 'bilingual')
    await $('[data-ip-docx-shape] .ip-error').waitForDisplayed({ timeout: 10_000 })
    expect(await $(TEXT_SCREEN).isDisplayed()).toBe(true)
    expect(await $(BILINGUAL_SCREEN).isExisting()).toBe(false)

    await browser.waitUntil(async () => browser.execute(() => document.querySelector('.ip-screen .ip-act-primary')?.disabled === false), {
      timeout: 30_000,
    })
    await realClick(await $(`${TEXT_SCREEN} .ip-act-primary`))
    await $(TEXT_SCREEN).waitForExist({ reverse: true, timeout: 30_000 })
    await backToLibrary()
  })

  it('a two-column table .docx opens the bilingual preview and the switch flips there and back', async () => {
    await openFilesTabWith('song-ngu.docx', buildDocxWithTable([
      ['Hello world.', 'Xin chao the gioi.'],
      ['Good morning.', 'Chao buoi sang.'],
    ]))
    await pickEnglish()
    await realClick(await $(`${PANEL} [data-import-preview-open]`))
    await $(BILINGUAL_SCREEN).waitForDisplayed({ timeout: 30_000 })

    await flipSwitchTo('bip', 'text')
    await $(TEXT_SCREEN).waitForDisplayed({ timeout: 30_000 })
    await $(BILINGUAL_SCREEN).waitForExist({ reverse: true, timeout: 10_000 })

    await flipSwitchTo('ip', 'bilingual')
    await $(BILINGUAL_SCREEN).waitForDisplayed({ timeout: 30_000 })
    await $(TEXT_SCREEN).waitForExist({ reverse: true, timeout: 10_000 })

    await browser.waitUntil(async () => browser.execute(() => document.querySelector('.bip-screen .bip-act-primary')?.disabled === false), {
      timeout: 30_000,
    })
    await realClick(await $(`${BILINGUAL_SCREEN} .bip-act-primary`))
    await $(BILINGUAL_SCREEN).waitForExist({ reverse: true, timeout: 30_000 })
  })
})
