/**
 * Multi-column screens keep a single scroll region: Attribution has no inner scroller around the
 * licence text; Export and the import preview each allow exactly one named nested scroller (the
 * Chapter list, the sticky config column); the prompt inspector and the bilingual preview have none.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-17-5-man-nhieu-cot.e2e.mjs
 */
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { ensureFullLayoutTier } from '../support/layoutTier.mjs'
import { isInViewport, measureOverlayFrame, resizeLogical } from '../support/overlayFrame.mjs'
import { aimWork, openAimedWork, reloadWorks } from '../support/libraryGrid.mjs'
import { backToLibrary, dropFiles, selectImportTab, waitLibraryReady } from '../support/importForm.mjs'
import { mkdtempSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_NAME = `Story 17-5 -- export ${RUN_TAG}`
const CHAPTERS = 40
const LICENSE_LINES = 200

async function invoke(command, args = {}) {
  return browser.execute(async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a), command, args)
}

async function pickScope(index) {
  const labels = await $$('.ex-seg-item')
  await realClick(labels[index])
  await browser.waitUntil(
    async () => browser.execute((i) => document.querySelectorAll('input[name="export-scope"]')[i].checked, index),
    { timeout: 10_000, timeoutMsg: 'the export scope did not change' },
  )
}

async function scrollToEnd(selector) {
  await browser.execute((sel) => {
    const el = document.querySelector(sel)
    el.scrollTop = el.scrollHeight
  }, selector)
}

const MEASURE_ATTR = 'data-e2e-measure'

/** Tags the nearest vertical scroller around `selector` (the screens scroll in their host), so the frame measure sees the host region too. */
async function tagHostScroller(selector) {
  await browser.execute(
    (sel, attr) => {
      for (let node = document.querySelector(sel).parentElement; node !== null; node = node.parentElement) {
        const y = getComputedStyle(node).overflowY
        if (y === 'auto' || y === 'scroll') {
          node.setAttribute(attr, '')
          return
        }
      }
      throw new Error(`no scrolling ancestor around ${sel}`)
    },
    selector,
    MEASURE_ATTR,
  )
  return `[${MEASURE_ATTR}]`
}

async function untagHostScroller() {
  await browser.execute((attr) => document.querySelectorAll(`[${attr}]`).forEach((el) => el.removeAttribute(attr)), MEASURE_ATTR)
}

async function openImportOverlay() {
  await waitLibraryReady()
  await realClick(await $('[data-library-import-open]'))
  await $('.li-panel .import-form').waitForDisplayed({ timeout: 10_000 })
}

async function closeImportScreens(screen) {
  await browser.keys(['Escape'])
  await $(screen).waitForExist({ reverse: true, timeout: 10_000 })
  if (await $('.li-panel').isExisting()) {
    await browser.keys(['Escape'])
    await $('.li-panel').waitForExist({ reverse: true, timeout: 10_000 })
  }
}

describe('Story 17-5 -- multi-column screens without nested scroll', () => {
  it('Attribution: a 200-line licence text sits in one scroll region', async () => {
    await openWorkspaceWithWork(`e2e-17-5-attribution-${RUN_TAG}`)
    await ensureFullLayoutTier()
    const opener = await $('[data-attribution-open]')
    await opener.waitForExist({ timeout: 30_000 })
    await realClick(opener)
    await $('.attr-panel').waitForDisplayed({ timeout: 10_000 })

    await browser.execute(async (lines) => {
      const mod = await import('/src/panels/dictSourcesState.ts')
      const text = Array.from({ length: lines }, (_, i) => `Licence line ${i + 1} `.repeat(6)).join('\n')
      mod.attributionLicenseView.__v_raw.value = { code: 'e2e', text, error: null }
    }, LICENSE_LINES)
    await $('.attr-license-text').waitForDisplayed({ timeout: 10_000 })

    try {
      const frame = await measureOverlayFrame('.attr-screen')
      await expect(frame.nested).toBe(0)
      await expect(frame.scrollers.length).toBe(1)
      const overflow = await browser.execute(() => {
        const screen = document.querySelector('.attr-screen')
        return screen.scrollHeight > screen.clientHeight + 1
      })
      await expect(overflow).toBe(true)
    } finally {
      await browser.keys(['Escape'])
      await browser.keys(['Escape'])
    }
    await $('.attr-panel').waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('Export: scope Work has two sibling scrollers and none nested', async () => {
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
      timeout: 30_000,
      interval: 250,
    })
    const text = Array.from({ length: CHAPTERS }, (_, i) => `第${i + 1}章\nCau ${i + 1}。`).join('\n')
    const pattern = { pattern: '第', kind: 'literal' }
    await invoke('preview_import_encoding_from_text', { text, sourceLang: 'zh', chapterPattern: pattern, destination: null })
    await invoke('confirm_import_with_encoding', { name: WORK_NAME, sourceLang: 'zh', genre: 'general', encoding: 'UTF-8', chapterPattern: pattern })
    await resizeLogical(1280, 800)
    await browser.keys(['Meta', '1'])
    await $('.titlebar-import').waitForDisplayed({ timeout: 30_000 })
    await reloadWorks()
    await aimWork(WORK_NAME)
    await openAimedWork()
    await $('[data-library-work-page]').waitForExist({ timeout: 20_000 })
    const row = await $('[data-library-chapters-list] [data-library-chapter-row]')
    await row.waitForDisplayed({ timeout: 20_000 })
    await realClick(row)
    await realClick(await row.$('[data-library-row-edit]'))
    await $('[data-export-open]:not([disabled])').waitForExist({ timeout: 10_000 })
    await realClick(await $('[data-export-open]'))
    await $('.ex-panel').waitForDisplayed({ timeout: 10_000 })

    await pickScope(2)
    const frame = await measureOverlayFrame('.ex-panel')
    await expect(frame.nested).toBe(0)
    await expect(frame.scrollers.length).toBe(2)
  })

  it('Export: scope Chapters nests only the Chapter list and keeps the run button in view', async () => {
    await pickScope(1)
    await $('[data-export-chapter-list]').waitForDisplayed({ timeout: 10_000 })
    const overflow = await browser.execute(() => {
      const list = document.querySelector('[data-export-chapter-list]')
      return list.scrollHeight > list.clientHeight + 1
    })
    await expect(overflow).toBe(true)

    const frame = await measureOverlayFrame('.ex-panel')
    await expect(frame.nested).toBe(1)
    await expect(frame.scrollers.filter((s) => s.includes('ex-chapters')).length).toBe(1)

    await scrollToEnd('.ex-config')
    await scrollToEnd('.ex-preview')
    await expect(await isInViewport(await $('[data-export-run]'))).toBe(true)

    await browser.keys(['Escape'])
    await $('.ex-panel').waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('Prompt inspector: one scroll region, none nested', async () => {
    await openWorkspaceWithWork(`e2e-17-5-prompt-${RUN_TAG}`)
    await ensureFullLayoutTier()
    const opener = await $('[data-ai-prompt-inspector-open]')
    await opener.waitForDisplayed({ timeout: 30_000 })
    await realClick(opener)
    await $('.aip-panel').waitForDisplayed({ timeout: 10_000 })
    try {
      const frame = await measureOverlayFrame('.aip-scrim')
      await expect(frame.nested).toBe(0)
      await expect(frame.scrollers.length).toBe(1)
    } finally {
      await browser.keys(['Escape'])
    }
    await $('.aip-panel').waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('Bilingual import preview: one scroll region, none nested', async () => {
    await backToLibrary()
    const csv = join(mkdtempSync(join(tmpdir(), 'e2e-17-5-')), 'song-ngu.csv')
    writeFileSync(csv, Array.from({ length: 40 }, (_, i) => `甲${i}。,Mot ${i}`).join('\n') + '\n', 'utf8')
    await openImportOverlay()
    const form = await $('.li-panel .import-form')
    await (await form.$('input[type="text"]')).setValue(`e2e-17-5-bip-${RUN_TAG}`)
    await realClick(await (await form.$('[data-import-source-lang] input[type="radio"][value="zh"]')).$('..'))
    await dropFiles([csv])
    await $('.li-panel [data-import-tabpanel="files"]').waitForDisplayed({ timeout: 10_000 })
    await browser.waitUntil(async () => browser.execute(() => document.querySelectorAll('[data-import-file-path]').length === 1), { timeout: 10_000 })
    await realClick(await $('.li-panel [data-import-preview-open]'))
    await $('.bip-screen').waitForDisplayed({ timeout: 30_000 })
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('.bip-screen .bip-act-primary')?.disabled === false), {
      timeout: 30_000,
      timeoutMsg: 'the bilingual preview did not load its rows',
    })
    try {
      const host = await tagHostScroller('.bip-screen')
      const frame = await measureOverlayFrame(host)
      await expect(frame.nested).toBe(0)
      await expect(frame.scrollers.length).toBe(1)
    } finally {
      await untagHostScroller()
      await closeImportScreens('.bip-screen')
    }
  })

  it('Import preview: the sticky config column is the one named nested scroller', async () => {
    await backToLibrary()
    await openImportOverlay()
    await selectImportTab('paste')
    const form = await $('.li-panel .import-form')
    await (await form.$('input[type="text"]')).setValue(`e2e-17-5-ip-${RUN_TAG}`)
    await (await form.$('textarea')).setValue(Array.from({ length: 60 }, (_, i) => `Cau ${i + 1}。`).join('\n'))
    await realClick(await $('.li-panel [data-import-preview-open]'))
    await $('.ip-screen').waitForDisplayed({ timeout: 30_000 })
    await $('.ip-config').waitForDisplayed({ timeout: 10_000 })
    try {
      const host = await tagHostScroller('.ip-screen')
      const frame = await measureOverlayFrame(host)
      await expect(frame.nested).toBe(1)
      await expect(frame.scrollers.filter((s) => s.includes('ip-config')).length).toBe(1)
    } finally {
      await untagHostScroller()
      await closeImportScreens('.ip-screen')
    }
  })
})
