// Not covered: the native folder dialog (blocks WebDriver) and the host file manager launch.
import { cpSync, existsSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'

import { realClick } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const MOVER = `e2e-123-mover-${RUN_TAG}`
const DOOMED = `e2e-123-doomed-${RUN_TAG}`
const CLONE = `e2e-123-clone-${RUN_TAG}`

const RESCAN_BTN = '.root-block .root-actions .btn:nth-of-type(2)'
const REBUILD_BTN = '[data-library-rebuild-index]'
const CHANGES = '[data-library-rescan-changes]'
const UNCHANGED = '[data-library-rescan-unchanged]'
const ORPHAN_NAME = '.orphan-row .orphan-name'

async function createViaIpc(name) {
  const result = await browser.execute(async (workName) => {
    try {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: workName,
        sourceLang: 'zh',
        genre: 'general',
        text: 'Một câu nguồn để bộ nhập có việc mà làm.',
      })
      return ''
    } catch (err) {
      return String(err && err.code ? err.code : err)
    }
  }, name)
  expect(result).toBe('')
}

async function waitIdle(selector) {
  await browser.waitUntil(async () => (await $(selector)).isEnabled(), {
    timeout: 20_000,
    timeoutMsg: `${selector} did not become enabled within 20 s`,
  })
}

async function rescanAndWait() {
  await realClick(await $(RESCAN_BTN))
  await waitIdle(RESCAN_BTN)
}

async function changeNames(kind) {
  return browser.execute(
    (selector) => Array.from(document.querySelectorAll(selector), (node) => node.textContent.trim()),
    `li[data-library-change="${kind}"] .change-name`,
  )
}

describe('rescan report in the real window', () => {
  const libraryRoot = process.env.AURATRANSLATE_E2E_LIBRARY_ROOT
  const dirOf = (name) => join(libraryRoot, `${name}.atproj`)
  const renamedMover = () => join(libraryRoot, `${MOVER}-renamed.atproj`)

  before(async () => {
    expect(libraryRoot).toBeDefined()
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
      timeout: 30_000,
      interval: 250,
    })
    await $(RESCAN_BTN).waitForDisplayed({ timeout: 30_000 })
  })

  it('before any rescan there is no change strip but the startup rebuild stamp is shown', async () => {
    expect(await (await $(CHANGES)).isExisting()).toBe(false)
    await $('[data-library-rebuilt-at]').waitForExist({ timeout: 20_000 })
  })

  it('a rescan right after creation reports "no changes", not a list', async () => {
    await createViaIpc(MOVER)
    await createViaIpc(DOOMED)
    await rescanAndWait()

    expect(await (await $(UNCHANGED)).isExisting()).toBe(true)
    expect(await (await $('li[data-library-change]')).isExisting()).toBe(false)
  })

  it('a copied .atproj with a never-seen work_id is reported as exactly one appeared row', async () => {
    cpSync(dirOf(MOVER), dirOf(CLONE), { recursive: true })
    const metaPath = join(dirOf(CLONE), 'meta.json')
    const meta = JSON.parse(readFileSync(metaPath, 'utf8'))
    meta.work_id = `e2e-123-id-${RUN_TAG}`
    meta.name = CLONE
    writeFileSync(metaPath, JSON.stringify(meta))

    await rescanAndWait()

    expect(await changeNames('appeared')).toEqual([CLONE])
    expect(await changeNames('moved')).toEqual([])
    expect(await changeNames('orphaned')).toEqual([])
  })

  it('one .atproj renamed inside the root and one deleted give exactly one move and one new orphan', async () => {
    renameSync(dirOf(MOVER), renamedMover())
    rmSync(dirOf(DOOMED), { recursive: true, force: true })

    await rescanAndWait()

    expect(await changeNames('moved')).toEqual([MOVER])
    expect(await changeNames('orphaned')).toEqual([DOOMED])
    expect(await changeNames('appeared')).toEqual([])
    const movedPath = await (await $('li[data-library-change="moved"] .change-path')).getText()
    expect(movedPath).toContain(dirOf(MOVER))
    expect(movedPath).toContain(renamedMover())
    expect(await (await $(UNCHANGED)).isExisting()).toBe(false)
  })

  it('an immediate second rescan is back to "no changes" while the orphan stays listed', async () => {
    await rescanAndWait()

    expect(await (await $(UNCHANGED)).isExisting()).toBe(true)
    expect(await (await $(ORPHAN_NAME)).getText()).toBe(DOOMED)
  })

  it('a full rebuild stamps the time and does not make the orphan disappear', async () => {
    await realClick(await $(REBUILD_BTN))
    await waitIdle(REBUILD_BTN)

    const stamp = await $('[data-library-rebuilt-at]')
    await stamp.waitForExist({ timeout: 20_000 })
    expect(await stamp.getAttribute('data-rebuilt-at')).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/)
    expect(await (await $(ORPHAN_NAME)).getText()).toBe(DOOMED)
  })

  after(() => {
    for (const dir of [dirOf(MOVER), renamedMover(), dirOf(DOOMED), dirOf(CLONE)]) {
      if (existsSync(dir)) rmSync(dir, { recursive: true, force: true })
    }
  })
})
