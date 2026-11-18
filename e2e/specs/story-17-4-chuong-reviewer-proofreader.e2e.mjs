/**
 * Split Chapter, hand joining and the Proofreader memory run on the shared overlay frame:
 * 1120px at a 1280px window and exactly one vertical scroller inside the frame.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-17-4-chuong-reviewer-proofreader.e2e.mjs
 */
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { isInViewport, measureOverlayFrame, resizeLogical } from '../support/overlayFrame.mjs'
import { aimWork, openAimedWork, reloadWorks } from '../support/libraryGrid.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const SPLIT_WORK = `Story 17-4 -- split ${RUN_TAG}`
const JOIN_WORK = `Story 17-4 -- join ${RUN_TAG}`
const WIDTH_TOLERANCE_PX = 1
const SENTENCES = 200
const UNMATCHED = 40

async function invoke(command, args = {}) {
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

async function squeeze(selector, height) {
  await browser.execute((sel, h) => {
    document.querySelector(sel).style.maxHeight = h
  }, selector, height)
}

describe('Story 17-4 -- Chapter, reviewer and Proofreader overlays on the shared frame', () => {
  it('sets up a Work of 200 sentences in the Library at 1280px', async () => {
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
      timeout: 30_000,
      interval: 250,
    })
    const text = Array.from({ length: SENTENCES }, (_, i) => `Cau ${i + 1}。`).join('')
    await invoke('create_work_from_text', { name: SPLIT_WORK, sourceLang: 'zh', genre: 'general', text })
    await resizeLogical(1280, 720)
  })

  it('Split Chapter is one width, one scroll region, with its foot in view', async () => {
    await reloadWorks()
    await aimWork(SPLIT_WORK)
    await openAimedWork()
    await $('[data-library-work-page]').waitForExist({ timeout: 20_000 })
    const row = await $('[data-library-chapters-list] [data-library-chapter-row]')
    await row.waitForDisplayed({ timeout: 20_000 })
    await realClick(row)
    await realClick(await row.$('[data-library-row-edit]'))
    await $('[data-library-chapter-split-open]:not([disabled])').waitForExist({ timeout: 10_000 })
    await realClick(await $('[data-library-chapter-split-open]'))
    await $('[data-library-chapter-split-overlay]').waitForDisplayed({ timeout: 10_000 })
    await $('[data-library-chapter-split-list]').waitForDisplayed({ timeout: 20_000 })

    await expectOneRegion('.cs-panel')
    const overflow = await browser.execute(() => {
      const body = document.querySelector('.cs-panel .ui-overlay__body')
      return body.scrollHeight > body.clientHeight + 1
    })
    await expect(overflow).toBe(true)
    await expect(await isInViewport(await $('[data-library-chapter-split-confirm]'))).toBe(true)

    await browser.execute(() => {
      const body = document.querySelector('.cs-panel .ui-overlay__body')
      body.scrollTop = body.scrollHeight
    })
    await expect(await isInViewport(await $('[data-library-chapter-split-confirm]'))).toBe(true)
    await browser.keys(['Escape'])
    await $('[data-library-chapter-split-overlay]').waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('sets up a Work in the workspace and a reviewer copy with many unprocessed rows', async () => {
    await openWorkspaceWithWork(JOIN_WORK, '甲。\n乙。')
    await resizeLogical(1280, 720)
    await browser.execute((count) => {
      const rows = Array.from({ length: count }, (_, i) => ({ id: 100 + i, kind: 'text', source_text: null, target_text: `sửa ${i}` }))
      const segments = Array.from({ length: count }, (_, i) => ({
        id: 500 + i,
        ord: i + 1,
        role: null,
        source_text: `nguồn ${i}`,
        target_text: `dịch ${i}`,
      }))
      const reply = {
        chapter_id: 1,
        file_name: 'review.md',
        file_kind: 'md',
        rows,
        segments,
        groups: [],
        unmatched_row_ids: rows.map((r) => r.id),
        unmatched_segment_ids: segments.map((s) => s.id),
        is_resolved: false,
      }
      const realFetch = window.fetch
      window.fetch = (url, init) => {
        if (String(url).split('/').pop() !== 'alignment_open') return realFetch(url, init)
        return Promise.resolve(
          new Response(JSON.stringify(reply), { headers: { 'Tauri-Response': 'ok', 'Content-Type': 'application/json' } }),
        )
      }
    }, UNMATCHED)
  })

  it('hand joining is one width, one scroll region, and its last row stays below the column labels', async () => {
    await openFromTools('export.alignment.open')
    await $('.al-panel').waitForDisplayed({ timeout: 10_000 })
    await browser.waitUntil(async () => (await $$('.al-panel .al-row')).length === UNMATCHED * 2, {
      timeout: 10_000,
      timeoutMsg: 'the unprocessed rows never rendered',
    })
    await expectOneRegion('.al-panel')
    await expect(await isInViewport(await $('.al-act-primary'))).toBe(true)

    await squeeze('.al-panel', '280px')
    await expectOneRegion('.al-panel')
    await expect(await isInViewport(await $('.al-act-primary'))).toBe(true)

    const probeCurrent = () =>
      browser.execute(() => {
        const body = document.querySelector('.al-panel .ui-overlay__body')
        const box = body.getBoundingClientRect()
        const row = document.querySelector('.al-panel .al-row-current').getBoundingClientRect()
        const cols = document.querySelector('.al-panel .al-cols').getBoundingClientRect()
        return {
          scrolled: body.scrollTop > 0,
          inBody: row.bottom <= box.bottom + 1,
          belowCols: row.top >= cols.bottom - 1,
        }
      })
    await browser.execute(() => document.querySelector('.al-panel').focus())
    for (let i = 0; i < UNMATCHED * 2 - 1; i += 1) await browser.keys(['ArrowDown'])
    const last = await probeCurrent()
    await expect(last.scrolled).toBe(true)
    await expect(last.inBody).toBe(true)
    await expect(last.belowCols).toBe(true)
    for (let i = 0; i < UNMATCHED * 2 - 1; i += 1) await browser.keys(['ArrowUp'])
    const first = await probeCurrent()
    await expect(first.belowCols).toBe(true)
    await expect(first.inBody).toBe(true)

    await squeeze('.al-panel', '')
    await browser.keys(['Escape'])
    await $('.al-panel').waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('Ignored spots open as a centred 1120px screen on a solid scrim', async () => {
    await openFromTools('ai.proofread.memory.open')
    await $('.pfmem-panel').waitForDisplayed({ timeout: 10_000 })
    await expectOneRegion('.pfmem-panel')
    const place = await browser.execute(() => {
      const box = document.querySelector('.pfmem-panel').getBoundingClientRect()
      const scrim = getComputedStyle(document.querySelector('.pfmem-scrim'))
      return {
        offCentre: Math.abs(box.left + box.width / 2 - window.innerWidth / 2),
        background: scrim.backgroundColor,
      }
    })
    await expect(place.offCentre).toBeLessThanOrEqual(WIDTH_TOLERANCE_PX)
    await expect(place.background).not.toBe('rgba(0, 0, 0, 0)')
    await browser.keys(['Escape'])
    await $('.pfmem-panel').waitForExist({ reverse: true, timeout: 10_000 })
  })
})
