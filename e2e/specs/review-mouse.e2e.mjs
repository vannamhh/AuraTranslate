/**
 * Review Mode and version history reached by pointer, in a real WKWebView.
 *
 * Importing a reviewer copy starts in a native file dialog WebDriver cannot drive, so
 * `answerReviewReads` replies to the two read commands at the IPC transport. Everything past that
 * reply is the real UI.
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { aimWork, openAimedWork } from '../support/libraryGrid.mjs'
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const NUM = (id) => `[data-num-segment-id="${id}"]`
const TGT = (id) => `[data-col="tgt"][data-segment-id="${id}"]`

async function rowIds() {
  return await browser.execute(() =>
    [...document.querySelectorAll('[data-col="src"]')].map((el) => el.getAttribute('data-segment-id')),
  )
}

async function readSegmentsFromDisk() {
  return browser.execute(async () => window.__TAURI_INTERNALS__.invoke('read_open_chapter_segments', {}))
}

async function sign(chapterId, id, text) {
  await browser.execute(
    async (c, i, t) => {
      const internals = window.__TAURI_INTERNALS__
      await internals.invoke('save_segment_targets', { chapterId: c, edits: [{ id: i, target_text: t }] })
      await internals.invoke('confirm_segment', { segmentId: i })
    },
    chapterId,
    id,
    text,
  )
}

async function answerReviewReads(chapterId, ids) {
  await browser.execute(
    (chapter, segmentIds) => {
      const rows = segmentIds.map((_, i) => ({ id: 100 + i, kind: 'text', source_text: `nguồn ${i}`, target_text: `sửa ${i}` }))
      const segments = segmentIds.map((id, i) => ({ id, ord: i + 1, role: null, source_text: `nguồn ${i}`, target_text: `dịch ${i}` }))
      const pairs = segmentIds.map((id, i) => ({
        group_id: i + 1,
        decided_by: 'machine',
        segment_ids: [id],
        row_ids: [100 + i],
        spans: [
          { kind: 'delete', text: `dịch ${i}` },
          { kind: 'insert', text: `sửa ${i}` },
        ],
        decision: null,
      }))
      const replies = {
        alignment_open: {
          chapter_id: chapter,
          file_name: 'review.md',
          file_kind: 'md',
          rows,
          segments,
          groups: [],
          unmatched_row_ids: [],
          unmatched_segment_ids: [],
          is_resolved: true,
        },
        review_diff: { chapter_id: chapter, pairs },
      }
      const realFetch = window.fetch
      window.fetch = (url, init) => {
        const reply = replies[String(url).split('/').pop()]
        if (reply === undefined) return realFetch(url, init)
        return Promise.resolve(
          new Response(JSON.stringify(reply), { headers: { 'Tauri-Response': 'ok', 'Content-Type': 'application/json' } }),
        )
      }
    },
    chapterId,
    ids.map(Number),
  )
}

async function caretSegment() {
  return browser.execute(() => document.querySelector('.cell-tgt.editing')?.getAttribute('data-segment-id') ?? null)
}

async function switchCells() {
  return $$('.layout-switch .layout-switch-btn')
}

async function secondaryPress(selector) {
  await browser.execute((sel) => {
    const el = document.querySelector(sel)
    const rect = el.getBoundingClientRect()
    const init = {
      button: 2,
      bubbles: true,
      cancelable: true,
      clientX: Math.round(rect.x + rect.width * 0.3),
      clientY: Math.round(rect.y + rect.height / 2),
    }
    el.dispatchEvent(new MouseEvent('mousedown', init))
    el.dispatchEvent(new MouseEvent('contextmenu', init))
    el.dispatchEvent(new MouseEvent('mouseup', init))
  }, selector)
  await $('[data-sentence-menu]').waitForExist({ timeout: 5_000 })
}

async function menuItem(label) {
  for (const item of await $$('[data-sentence-menu] button')) {
    if ((await item.getText()).includes(label)) return item
  }
  throw new Error(`menu item "${label}" missing`)
}

describe('Rà soát và Lịch sử bằng chuột trong WKWebView thật', () => {
  it('ô Review Mode trên Chương chưa có bản Reviewer: lời báo hiện, Workspace còn nguyên', async () => {
    await openWorkspaceWithWork('Story 14.4 — ô Review Mode', '一。二。')
    await waitForGridRows(2, { what: 'Tác phẩm Review Mode' })

    const cells = await $$('.layout-switch .layout-switch-btn')
    await expect(cells.length).toBe(3)
    await expect(await cells[2].getAttribute('aria-pressed')).toBe('false')
    await realClick(cells[2])

    await $('[data-review-notice]').waitForExist({ timeout: 10_000 })
    await expect(await $('[data-review-dock]').isExisting()).toBe(false)
    await expect(await cells[2].getAttribute('aria-pressed')).toBe('false')
  })

  it('ô Review Mode vào, hai nút dời diff, và chính ô đó trả về đúng câu', async () => {
    await openWorkspaceWithWork('Story 14.4 — vào ra Review Mode', '一。二。三。')
    await waitForGridRows(3, { what: 'Tác phẩm vào ra Review Mode' })
    const ids = await rowIds()
    await answerReviewReads((await readSegmentsFromDisk()).chapter_id, ids)

    await realClick(await $(TGT(ids[1])))
    await browser.waitUntil(async () => (await caretSegment()) === ids[1], { timeout: 5_000 })

    await realClick((await switchCells())[2])
    await $('[data-review-dock]').waitForExist({ timeout: 10_000 })
    const pressed = await browser.execute(() =>
      [...document.querySelectorAll('.layout-switch .layout-switch-btn')].map((b) => b.getAttribute('aria-pressed')),
    )
    await expect(pressed).toEqual(['false', 'false', 'true'])

    const status = () => $('[data-review-diff-status]').getText()
    await realClick(await $('[data-review-diff-next]'))
    await realClick(await $('[data-review-diff-next]'))
    await browser.waitUntil(async () => (await status()) === 'Khác biệt 2/3', { timeout: 5_000 })
    await realClick(await $('[data-review-diff-prev]'))
    await browser.waitUntil(async () => (await status()) === 'Khác biệt 1/3', { timeout: 5_000 })

    await realClick((await switchCells())[2])
    await $('[data-review-dock]').waitForExist({ timeout: 10_000, reverse: true })
    await browser.waitUntil(async () => (await caretSegment()) === ids[1], {
      timeout: 5_000,
      timeoutMsg: 'caret không về câu trước khi mở Review Mode',
    })
  })

  it('Review Mode ở Chương A: mở lại A từ Thư viện giữ dải, mở B thì dải đóng', async () => {
    const workName = 'Story 14.12 — mở Chương khác'
    await openWorkspaceWithWork(workName, '一。二。三。')
    await waitForGridRows(3, { what: 'Tác phẩm trước khi tách Chương' })
    const ids = await rowIds()
    const first = await readSegmentsFromDisk()
    await browser.execute(
      async (segmentId) => window.__TAURI_INTERNALS__.invoke('split_chapter_at_segment', { segmentId }),
      Number(ids[1]),
    )
    const chapters = await browser.execute(async () => window.__TAURI_INTERNALS__.invoke('list_chapters', {}))
    const other = chapters.find((row) => row.chapter_id !== first.chapter_id).chapter_id

    const toLibrary = async () => realClick((await $$('.mode-tab'))[0])
    const openFromWorkPage = async (chapterId) => {
      const row = await $(`[data-library-chapter-row][data-chapter-id="${chapterId}"]`)
      await row.waitForExist({ timeout: 10_000 })
      await realClick(row)
      await realClick(await row.$('[data-library-open-chapter]'))
    }

    await toLibrary()
    await aimWork(workName)
    await openAimedWork()
    await openFromWorkPage(first.chapter_id)
    await waitForGridRows(1, { what: 'Chương A sau khi tách' })

    await answerReviewReads(first.chapter_id, ids.slice(0, 1))
    await realClick((await switchCells())[2])
    await $('[data-review-dock]').waitForExist({ timeout: 10_000 })

    await toLibrary()
    await openFromWorkPage(first.chapter_id)
    await $('[data-review-dock]').waitForDisplayed({ timeout: 10_000 })

    await toLibrary()
    await openFromWorkPage(other)
    await $('[data-review-dock]').waitForExist({ timeout: 10_000, reverse: true })
    await waitForGridRows(2, { what: 'Chương B sau khi đóng Review Mode' })
    await expect((await readSegmentsFromDisk()).chapter_id).toBe(other)
    await browser.waitUntil(
      async () => browser.execute(() => document.activeElement?.matches('.mode') ?? false),
      { timeout: 5_000, timeoutMsg: 'tiêu điểm không về gốc Workspace sau khi mở Chương từ Thư viện' },
    )
    await expect(await $('[data-review-dock]').isExisting()).toBe(false)
  })

  it('Review Mode ở Chương A: mở B khi đang ở Workspace đóng dải, tiêu điểm về lưới', async () => {
    await openWorkspaceWithWork('Story 14.12 — trong Workspace', '一。二。三。')
    await waitForGridRows(3, { what: 'Tác phẩm trước khi tách Chương' })
    const ids = await rowIds()
    const first = await readSegmentsFromDisk()
    await browser.execute(
      async (segmentId) => window.__TAURI_INTERNALS__.invoke('split_chapter_at_segment', { segmentId }),
      Number(ids[1]),
    )
    const chapters = await browser.execute(async () => window.__TAURI_INTERNALS__.invoke('list_chapters', {}))
    const other = chapters.find((row) => row.chapter_id !== first.chapter_id).chapter_id

    await answerReviewReads(first.chapter_id, ids.slice(0, 1))
    await realClick((await switchCells())[2])
    await $('[data-review-dock]').waitForExist({ timeout: 10_000 })

    // Read the module URL from `main.ts`: Vite may serve it with an HMR stamp, a separate instance.
    await browser.execute(async (chapterId) => {
      const main = await (await fetch('/src/main.ts')).text()
      const url = main.match(/"(\/src\/panels\/editorPanelState\.ts[^"]*)"/)[1]
      await (await import(url)).openChapterById(chapterId)
    }, other)

    await $('[data-review-dock]').waitForExist({ timeout: 10_000, reverse: true })
    await waitForGridRows(2, { what: 'Chương B sau khi đóng Review Mode' })
    await browser.waitUntil(
      async () =>
        browser.execute(() => {
          const active = document.activeElement
          return active !== null && (active.matches('[data-col]') || active.querySelector('[data-col="src"]') !== null)
        }),
      { timeout: 5_000, timeoutMsg: 'tiêu điểm không về lưới sau khi Review Mode đóng' },
    )
  })

  it('chuột phải câu B rồi Lịch sử phiên bản: lớp phủ hiện lịch sử đúng câu B', async () => {
    await openWorkspaceWithWork('Story 14.4 — lịch sử từ menu', '一。二。三。')
    await waitForGridRows(3, { what: 'Tác phẩm lịch sử' })
    const ids = await rowIds()
    const chapterId = (await readSegmentsFromDisk()).chapter_id
    await sign(chapterId, Number(ids[1]), 'Bản của B.')

    await realClick(await $(TGT(ids[0])))
    await browser.pause(250)
    for (const selector of [TGT(ids[1]), `[data-col="src"][data-segment-id="${ids[1]}"]`, NUM(ids[1])]) {
      await secondaryPress(selector)
      await realClick(await menuItem('Lịch sử phiên bản'))
      await $('.hist-panel').waitForExist({ timeout: 10_000 })
      await browser.waitUntil(async () => (await $$('[data-history-row]')).length === 1, {
        timeout: 10_000,
        timeoutMsg: 'lịch sử của câu B không hiện đúng một phiên bản',
      })
      await expect(await $('[data-history-detail]').getText()).toContain('Bản của B.')
      await browser.keys(['Escape'])
      await $('.hist-panel').waitForExist({ timeout: 5_000, reverse: true })
      await expect(await browser.execute(() => String(window.getSelection()))).toBe('')
    }
  })
})
