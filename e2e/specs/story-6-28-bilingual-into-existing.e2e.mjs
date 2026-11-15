/**
 * A .csv dropped on the import overlay with Đưa vào = an existing Work opens the bilingual preview
 * with that Work as destination, and Nhập appends the paired rows as new Chapters after the last one,
 * whether the destination is the open Work or a closed one.
 */
import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { createWorkThroughForm, dropFiles, selectImportTab } from '../support/importForm.mjs'
import { aimWork, openAimedWork } from '../support/libraryGrid.mjs'
import { realClick } from '../support/pointer.mjs'

const PANEL = '.li-panel'
const SCREEN = '.bip-screen'
const dir = mkdtempSync(join(tmpdir(), 'e2e-628-'))

async function ipcValue(command, args = {}) {
  const json = await browser.execute(
    async (name, payload) => {
      try {
        return JSON.stringify({ ok: true, value: await window.__TAURI_INTERNALS__.invoke(name, payload) })
      } catch (err) {
        return JSON.stringify({ ok: false, text: err && err.code ? String(err.code) : String(err) })
      }
    },
    command,
    args,
  )
  const out = JSON.parse(json)
  if (!out.ok) throw new Error(`${command} failed: ${out.text}`)
  return out.value
}

async function ready() {
  await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
    timeout: 30_000,
    interval: 250,
  })
  await $('[data-library-import-open]').waitForDisplayed({ timeout: 30_000 })
}

function writeCsv(file, row) {
  const path = join(dir, file)
  writeFileSync(path, `${row}\n`, 'utf8')
  return path
}

/** Thêm Chương… on the named Work's card, drop the .csv, preview, Nhập. */
async function appendCsvThroughCard(workName, csv) {
  await aimWork(workName)
  const more = await $(
    `//li[contains(@class,"work-cell")][.//span[@data-library-work-name][normalize-space()="${workName}"]]//*[@data-library-work-more]`,
  )
  await more.waitForDisplayed({ timeout: 10_000 })
  await realClick(await more.$('summary, button'))
  const item = await more.$('[data-library-work-add-chapters]')
  await item.waitForDisplayed({ timeout: 10_000 })
  await realClick(item)
  await $(PANEL).waitForDisplayed({ timeout: 10_000 })
  await selectImportTab('files')
  await dropFiles([csv])
  await browser.waitUntil(async () => browser.execute(() => document.querySelectorAll('[data-import-file-path]').length === 1), {
    timeout: 10_000,
  })
  await realClick(await $(`${PANEL} [data-import-preview-open]`))
  await $(SCREEN).waitForDisplayed({ timeout: 30_000 })

  expect(await $(`${SCREEN} [data-bip-destination]`).getText()).toContain(workName)
  expect(await browser.execute(() => document.querySelector('[data-bilingual-use-new-work]') === null)).toBe(true)
  await browser.waitUntil(async () => browser.execute(() => document.querySelector('.bip-screen .bip-act-primary')?.disabled === false), {
    timeout: 30_000,
    timeoutMsg: 'Nhập stayed disabled for an existing-Work destination',
  })
  await realClick(await $(`${SCREEN} .bip-act-primary`))
  await $(SCREEN).waitForExist({ reverse: true, timeout: 30_000 })
}

/** Opens the Work page and waits for the new Chapter's row; with `translation`, opens it in the grid and waits for the target cell. */
async function expectChapterOnScreen(workName, chapterId, translation = null) {
  await aimWork(workName)
  await openAimedWork()
  const row = `[data-library-chapter-row][data-chapter-id="${chapterId}"]`
  await $(row).waitForDisplayed({ timeout: 20_000 })
  if (translation === null) return
  await realClick(await $(`${row} [data-library-row-translate]`))
  await browser.waitUntil(
    async () =>
      browser.execute(
        (text) => Array.from(document.querySelectorAll('[data-col="tgt"]')).some((cell) => (cell.textContent || '').includes(text)),
        translation,
      ),
    { timeout: 30_000, timeoutMsg: `no grid target cell shows "${translation}"` },
  )
}

async function backToLibrary() {
  await realClick(await $('.titlebar .mode-tab:first-child'))
  await $('[data-library-import-open]').waitForDisplayed({ timeout: 30_000 })
}

async function workIdByName(name) {
  const { works } = await ipcValue('library_list_works', { filter: null, genre: null, sourceLang: null, sort: null })
  return works.find((w) => w.name === name).work_id
}

describe('Story 6-28: bilingual import into an existing Work', () => {
  it('open destination: previews against the Work without the old notice, and Nhập appends a Chapter', async () => {
    await ready()
    await createWorkThroughForm('Song ngu them 628')
    await $('[data-library-work-name]').waitForDisplayed({ timeout: 30_000 })
    const before = await ipcValue('list_chapters')
    expect(before.length).toBeGreaterThan(0)
    const lastBefore = before[before.length - 1]

    await appendCsvThroughCard('Song ngu them 628', writeCsv('them-628.csv', 'hello world,xin chao the gioi'))

    const after = await ipcValue('list_chapters')
    expect(after.length).toBe(before.length + 1)
    expect(after.slice(0, before.length).map((c) => c.chapter_id)).toEqual(before.map((c) => c.chapter_id))
    const added = after[after.length - 1]
    expect(added.ord).toBe(lastBefore.ord + 1)
    expect(added.status).toBe('in_progress')
    await ipcValue('open_chapter', { chapterId: added.chapter_id })
    const loaded = await ipcValue('read_open_chapter_segments')
    expect(JSON.stringify(loaded.segments)).toContain('xin chao the gioi')

    await expectChapterOnScreen('Song ngu them 628', added.chapter_id)
    await backToLibrary()
  })

  it('closed destination: the Work is opened from the index, gains the Chapter and becomes the open Work', async () => {
    await createWorkThroughForm('Song ngu A 628')
    await $('[data-library-work-name]').waitForDisplayed({ timeout: 30_000 })
    const aBefore = await ipcValue('list_chapters')
    await createWorkThroughForm('Song ngu B 628')
    const bBefore = await ipcValue('list_chapters')
    const aIds = aBefore.map((c) => c.chapter_id)

    await appendCsvThroughCard('Song ngu A 628', writeCsv('a-628.csv', 'good morning,chao buoi sang'))

    const open = await ipcValue('list_chapters')
    expect(open.length).toBe(aBefore.length + 1)
    expect(open.slice(0, aBefore.length).map((c) => c.chapter_id)).toEqual(aIds)
    const added = open[open.length - 1]
    expect(added.ord).toBe(aBefore[aBefore.length - 1].ord + 1)
    expect(added.status).toBe('in_progress')

    await ipcValue('open_work', { workId: await workIdByName('Song ngu B 628') })
    expect(await ipcValue('list_chapters')).toEqual(bBefore)

    await expectChapterOnScreen('Song ngu A 628', added.chapter_id, 'chao buoi sang')
  })
})
