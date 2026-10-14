/**
 * Library search scope, grouped results and "open in Reading" in a real WKWebView (FR139).
 *
 * Limits:
 * - The query field is set with `value = …` plus a dispatched `input` event (same as 5-9).
 * - The Library root is one temp dir shared by the whole run; queries carry a run tag.
 * - Target text is typed with `execCommand('insertText')` (editor-typing-flush.e2e.mjs) and reaches
 *   the search index when the chapter status change rebuilds it.
 */

import { realClick } from '../support/pointer.mjs'
import { markFlushBaseline, waitForFlushAfter } from '../support/flushWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_DONE = `e2e-12-5-done-${RUN_TAG}`
const WORK_OPEN = `e2e-12-5-open-${RUN_TAG}`
const TARGET_MARKER = `khoáng${RUN_TAG}tq`
const TARGET_MARKER_PLAIN = `khoang${RUN_TAG}tq`
const SOURCE_MARKER = `zzqqunfinished${RUN_TAG}`

const SEARCH_INPUT = '[data-library-search-input]'
const HITS = '[data-library-search-hit]'
const OPEN_READING = '[data-library-search-open-reading]'

async function typeSearchQuery(text) {
  await browser.execute(
    (sel, value) => {
      const input = document.querySelector(sel)
      if (input === null) throw new Error('search input missing')
      input.value = value
      input.dispatchEvent(new Event('input', { bubbles: true }))
    },
    SEARCH_INPUT,
    text,
  )
}

async function submitSearch() {
  await browser.execute((el) => el.focus(), await $(SEARCH_INPUT))
  await browser.keys(['Enter'])
}

async function hitCount() {
  return browser.execute((sel) => document.querySelectorAll(sel).length, HITS)
}

async function pickScope(scope) {
  await realClick(await $(`[data-library-search-scope="${scope}"]`))
}

async function waitForHits(expected, what) {
  await browser.waitUntil(async () => (await hitCount()) === expected, {
    timeout: 20_000,
    interval: 300,
    timeoutMsg: `${what}: expected ${expected} hit rows`,
  })
}

async function backToLibrary() {
  await browser.keys(['Meta', '1'])
  await browser.waitUntil(async () => browser.execute((sel) => document.querySelector(sel) !== null, SEARCH_INPUT), {
    timeout: 30_000,
    timeoutMsg: 'Library did not show the search field',
  })
}

describe('Story 12-5 — Library search: scope, groups, open in Reading', () => {
  let targetSegmentId = 0
  let chapterId = 0

  before(async () => {
    const sentences = Array.from({ length: 60 }, (_, n) => `Cau so ${n + 1}。`).join('')
    await openWorkspaceWithWork(WORK_DONE, sentences)

    const cells = await $$('[data-col="tgt"]')
    expect(cells.length).toBeGreaterThan(5)
    const cell = cells[cells.length - 1]
    await browser.execute((el) => el.scrollIntoView({ block: 'center' }), cell)
    const baseline = await markFlushBaseline()
    await realClick(cell)
    await browser.pause(200)
    const inserted = await browser.execute((t) => document.execCommand('insertText', false, t), `ban dich ${TARGET_MARKER} het`)
    expect(inserted).toBe(true)
    await waitForFlushAfter(baseline, { what: 'target text typed into a late segment' })

    const snapshot = await browser.execute(async () => {
      const internals = window.__TAURI_INTERNALS__
      const read = await internals.invoke('read_open_chapter_segments', {})
      const chapter = await internals.invoke('read_open_chapter')
      return { segments: read.segments, chapterId: chapter.chapter_id }
    })
    const typed = snapshot.segments.find((s) => s.target_text.includes(TARGET_MARKER))
    if (typed === undefined) throw new Error('typed target text did not reach disk')
    targetSegmentId = typed.id
    chapterId = snapshot.chapterId

    // The status change rebuilds the index, so the typed target text becomes searchable.
    await browser.execute(async (id) => {
      await window.__TAURI_INTERNALS__.invoke('set_chapter_status', { chapterId: id, status: 'done' })
    }, chapterId)

    await backToLibrary()
  })

  it('a string only in the translation: no hit at scope Source, a hit at scope Target', async () => {
    await typeSearchQuery(TARGET_MARKER)
    await pickScope('source')
    await browser.waitUntil(
      async () => {
        await submitSearch()
        return browser.execute(() => document.querySelector('[data-library-search-empty]') !== null)
      },
      { timeout: 20_000, interval: 500, timeoutMsg: 'scope Source never produced the empty state' },
    )
    expect(await hitCount()).toBe(0)
    const emptyScope = await browser.execute(
      () => document.querySelector('[data-library-search-empty-scope]')?.textContent ?? '',
    )
    expect(emptyScope).toContain('nguyên văn')

    await pickScope('target')
    await waitForHits(1, 'scope Target')
    const row = await browser.execute(() => document.querySelector('[data-library-search-hit]')?.getAttribute('data-field'))
    expect(row).toBe('target')
  })

  it('results replace the Works grid and group under the Work header', async () => {
    const shape = await browser.execute(() => ({
      grid: document.querySelector('.works-block') !== null,
      header: document.querySelector('[data-library-search-group] .wk')?.textContent ?? '',
      counter: document.querySelector('[data-library-search-status]')?.textContent ?? '',
      extent: document.querySelector('[data-library-search-extent]')?.textContent ?? '',
    }))
    expect(shape.grid).toBe(false)
    expect(shape.header).toContain(WORK_DONE)
    expect(shape.header).toContain('Trung → Việt')
    expect(shape.header).toContain('1 kết quả')
    expect(shape.counter).toMatch(/^1 kết quả · \d+ ms$/)
    expect(shape.extent).toMatch(/Chương · \d+ Tác phẩm/)
  })

  it('a lenient hit paints its match with a different background than an exact hit', async () => {
    const markBackground = () =>
      browser.execute(() => {
        const mark = document.querySelector('[data-library-search-mark]')
        return mark === null ? null : getComputedStyle(mark).backgroundColor
      })
    const exact = await markBackground()
    expect(exact).not.toBeNull()

    // The plain spelling matches only through the accent-insensitive index, so the hit is lenient.
    await typeSearchQuery(TARGET_MARKER_PLAIN)
    await browser.waitUntil(
      async () => {
        await submitSearch()
        return browser.execute(() => document.querySelector('[data-library-search-mark].mark--loose') !== null)
      },
      { timeout: 20_000, interval: 500, timeoutMsg: 'lenient mark class never appeared' },
    )
    const loose = await markBackground()
    expect(loose).not.toBeNull()
    expect(loose).not.toBe(exact)
  })

  it('a status filter that excludes the Work blocks the hit and says how many Chapters were not searched', async () => {
    await realClick(await $('[data-lifecycle-filter="not_started"]'))
    await waitForHits(0, 'filter not_started')
    const blocked = await browser.execute(
      () => document.querySelector('[data-library-search-empty-filtered]')?.textContent ?? '',
    )
    expect(blocked).toMatch(/\d+ trong \d+ Chương không được tìm tới/)
    await realClick(await $('[data-library-search-clear-filters]'))
    await waitForHits(1, 'filter cleared')
  })

  it('"Open in Reading" shows Reading with the hit segment inside the viewport', async () => {
    await realClick(await $(OPEN_READING))
    await browser.waitUntil(
      async () =>
        browser.execute((id) => document.querySelector(`[data-reading-segment="${id}"]`) !== null, targetSegmentId),
      { timeout: 20_000, interval: 300, timeoutMsg: 'Reading did not render the hit segment' },
    )
    await browser.waitUntil(
      async () =>
        browser.execute((id) => {
          const rect = document.querySelector(`[data-reading-segment="${id}"]`)?.getBoundingClientRect()
          return rect !== undefined && rect.top >= 0 && rect.bottom <= window.innerHeight
        }, targetSegmentId),
      { timeout: 10_000, interval: 300, timeoutMsg: 'hit segment is not inside the viewport after opening Reading' },
    )
  })

  it('a hit in a Chapter that is not finished still opens Reading, showing the frontier-only state', async () => {
    await browser.execute(
      async (name, text) => {
        await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
          name,
          sourceLang: 'zh',
          genre: 'general',
          text,
        })
      },
      WORK_OPEN,
      `Cau mot。Cau chua ${SOURCE_MARKER} xong。`,
    )
    await backToLibrary()
    await typeSearchQuery(SOURCE_MARKER)
    await pickScope('both')
    await browser.waitUntil(
      async () => {
        if ((await hitCount()) === 1) return true
        await submitSearch()
        return false
      },
      { timeout: 20_000, interval: 500, timeoutMsg: 'unfinished Work hit never appeared' },
    )
    await realClick(await $(OPEN_READING))
    await browser.waitUntil(
      async () =>
        browser.execute(
          () => document.querySelector('[data-reading-status]')?.getAttribute('data-reading-status') === 'frontier-only',
        ),
      { timeout: 20_000, interval: 300, timeoutMsg: 'Reading did not show frontier-only for an unfinished Chapter' },
    )
  })
})
