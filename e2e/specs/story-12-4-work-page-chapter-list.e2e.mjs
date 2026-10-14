/**
 * The Work page: grouped Chapter list, multi-select, action bar, merge screen, split
 * box. One spec file = one app process; the cases run in order and share one seeded Work.
 *
 * Mouse presses use `realClick` / `modifiedMouseDown` (selection is a `mousedown`). The
 * keyboard half of each button is not drivable here (e2e/AGENTS.md) and is covered by the
 * vitest command-registry cases.
 */
import { realClick, modifiedMouseDown } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_NAME = `e2e-work-page-${RUN_TAG}`
const SOURCE_TEXT = 'Cau mot。Cau hai。Cau ba。Cau bon。Cau nam。Cau sau。Cau bay。Cau tam。'

const LIST_WORKS_BTN = '[data-lifecycle-action="list_works"]'
const WORK_NEXT_BTN = '[data-library-work-next]'
const OPEN_WORK_BTN = '[data-library-open-work]'
const WORK_PAGE = '[data-library-work-page]'
const ROWS = '[data-library-chapters-list] [data-library-chapter-row]'
const FILTER = (key) => `[data-library-chapter-filter="${key}"]`
const SEARCH = '[data-library-chapter-search]'
const MERGE_OPEN = '[data-library-chapter-merge-open]'
const MERGE_OVERLAY = '[data-library-chapter-merge-overlay]'
const MERGE_CONFIRM = '[data-library-chapter-merge-confirm]'
const SPLIT_OPEN = '[data-library-chapter-split-open]'
const SPLIT_OVERLAY = '[data-library-chapter-split-overlay]'
const SPLIT_CONFIRM = '[data-library-chapter-split-confirm]'
const STATUS_MENU = '[data-library-chapters-status-menu]'
const STATUS_SET = (status) => `[data-library-chapters-status-set="${status}"]`
const EXPORT_OPEN = '[data-export-open]'
const GROUP_TOGGLE = '[data-library-chapter-group-toggle]'

async function ipc(command, args = {}) {
  const json = await browser.execute(
    async (name, payload) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) return JSON.stringify({ ok: false, error: { code: null, text: 'no IPC bridge' } })
      try {
        return JSON.stringify({ ok: true, value: await internals.invoke(name, payload) })
      } catch (err) {
        const code = err && err.code ? String(err.code) : null
        return JSON.stringify({ ok: false, error: { code, text: code ?? String(err) } })
      }
    },
    command,
    args,
  )
  return JSON.parse(json)
}

async function ipcValue(command, args = {}) {
  const out = await ipc(command, args)
  if (!out.ok) throw new Error(`fixture: ${command} failed: ${JSON.stringify(out.error)}`)
  return out.value
}

/** Five Chapters: one sentence each for the first four, four sentences in the fifth. */
async function seedWork() {
  await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
    timeout: 30_000,
    interval: 250,
  })
  await ipcValue('create_work_from_text', { name: WORK_NAME, sourceLang: 'zh', genre: 'general', text: SOURCE_TEXT })
  for (let split = 0; split < 4; split += 1) {
    const chapters = await ipcValue('list_chapters')
    await ipcValue('open_chapter', { chapterId: chapters[chapters.length - 1].chapter_id })
    const loaded = await ipcValue('read_open_chapter_segments')
    await ipcValue('split_chapter_at_segment', { segmentId: loaded.segments[1].id })
  }
  const chapters = await ipcValue('list_chapters')
  expect(chapters.map((c) => c.segment_count)).toEqual([1, 1, 1, 1, 4])
  return chapters.map((c) => c.chapter_id)
}

async function markDoneAndOmitOnly(chapterId) {
  await ipcValue('open_chapter', { chapterId })
  const loaded = await ipcValue('read_open_chapter_segments')
  for (const segment of loaded.segments) {
    await ipcValue('set_segment_omitted', { segmentId: segment.id, omitted: true })
  }
  await ipcValue('set_chapter_status', { chapterId, status: 'done' })
}

async function probe() {
  return browser.execute(() => {
    const text = (el) => (el === null ? '' : (el.textContent || '').trim())
    const rows = Array.from(document.querySelectorAll('[data-library-chapter-row]')).map((row) => ({
      id: Number(row.getAttribute('data-chapter-id')),
      ord: text(row.querySelector('.chapter-ord')),
      status: row.getAttribute('data-chapter-status'),
      count: text(row.querySelector('.chapter-segment-count')),
      unconfirmed: row.querySelector('[data-library-chapter-unconfirmed]') === null
        ? null
        : text(row.querySelector('[data-library-chapter-unconfirmed]')),
      selected: row.getAttribute('aria-selected') === 'true',
    }))
    const enabled = (selector) => {
      const node = document.querySelector(selector)
      return node !== null && node.disabled !== true
    }
    return {
      rows,
      groups: Array.from(document.querySelectorAll('[data-library-chapter-group]')).map((g) =>
        g.getAttribute('data-library-chapter-group'),
      ),
      moreNote: document.querySelector('[data-library-chapter-more]') !== null,
      toggle: document.querySelector('[data-library-chapter-group-toggle]') !== null,
      statusLine: text(document.querySelector('[data-library-chapters-status]')),
      selectedLabel: text(document.querySelector('[data-library-chapter-selected-count]')),
      mergeEnabled: enabled('[data-library-chapter-merge-open]'),
      splitEnabled: enabled('[data-library-chapter-split-open]'),
      renameEnabled: enabled('[data-library-chapter-rename-begin]'),
      statusEnabled: enabled('[data-library-chapters-status-menu]'),
    }
  })
}

async function rowEl(chapterId) {
  return $(`${ROWS}[data-chapter-id="${chapterId}"]`)
}

async function waitRows(count, message) {
  await browser.waitUntil(async () => (await probe()).rows.length === count, { timeout: 20_000, timeoutMsg: message })
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
  await browser.waitUntil(async () => (await probe()).rows.length > 0, {
    timeout: 20_000,
    timeoutMsg: 'the Work page did not list any Chapter',
  })
}

let ids = []

describe('Story 12-4 · Work page Chapter list', () => {
  it('opens the Work page with groups ordered, the not-started group folded, and unconfirmed counts', async () => {
    ids = await seedWork()
    const [, , , c4, c5] = ids
    await markDoneAndOmitOnly(c4)
    await ipcValue('set_chapter_status', { chapterId: c5, status: 'done' })

    await openWorkPage()
    const shot = await probe()
    expect(shot.groups).toEqual(['done', 'not_started'])
    // Folded: two of the three not-started rows, plus the fold note and its toggle.
    expect(shot.rows.map((r) => r.id)).toEqual([c4, c5, ids[0], ids[1]])
    expect(shot.moreNote).toBe(true)
    expect(shot.toggle).toBe(true)

    // Done with four unconfirmed sentences: the number is the one Rust returned.
    const fromRust = (await ipcValue('list_chapters')).find((c) => c.chapter_id === c5).unconfirmed_count
    expect(fromRust).toBe(4)
    const unconfirmed = shot.rows.find((r) => r.id === c5)
    expect(unconfirmed.count).toBe('4 câu')
    expect(unconfirmed.unconfirmed).toContain('4 câu chưa xác nhận')
    // Done with every sentence omitted: no unconfirmed clause at all.
    expect(shot.rows.find((r) => r.id === c4).unconfirmed).toBeNull()

    await realClick(await $(GROUP_TOGGLE))
    await waitRows(5, 'expanding the not-started group did not show all five rows')
  })

  it('filter and search narrow the list; an empty result says no Chapter matches', async () => {
    await realClick(await $(FILTER('done')))
    await waitRows(2, 'the Done filter did not leave exactly the two done rows')
    expect((await probe()).groups).toEqual(['done'])
    await realClick(await $(FILTER('all')))
    await waitRows(5, 'the All filter did not restore the list')

    await browser.execute((sel) => {
      const input = document.querySelector(sel)
      input.value = 'xyz'
      input.dispatchEvent(new Event('input', { bubbles: true }))
    }, SEARCH)
    await waitRows(0, 'a search with no match still lists rows')
    const empty = await probe()
    expect(empty.statusLine).toContain('Không có Chương khớp')

    await browser.execute((sel) => {
      const input = document.querySelector(sel)
      input.value = ''
      input.dispatchEvent(new Event('input', { bubbles: true }))
    }, SEARCH)
    await waitRows(5, 'clearing the search did not restore the list')
  })

  it('two Chapters that are not neighbours: the merge button is off and Rust refuses a direct call', async () => {
    await realClick(await rowEl(ids[0]))
    await modifiedMouseDown(await rowEl(ids[2]), 'meta')
    await browser.waitUntil(async () => (await probe()).rows.filter((r) => r.selected).length === 2, {
      timeout: 10_000,
      timeoutMsg: 'Mod+click did not add the third row to the selection',
    })
    const shot = await probe()
    expect(shot.mergeEnabled).toBe(false)
    expect(shot.selectedLabel).toContain('2')

    const before = await ipcValue('list_chapters')
    const refused = await ipc('merge_chapters', { chapterIds: [ids[0], ids[2]] })
    expect(refused.ok).toBe(false)
    expect(refused.error.code).toBe('chapter.merge_not_contiguous')
    const after = await ipcValue('list_chapters')
    expect(after).toEqual(before)
  })

  it('three neighbouring Chapters merge through the merge screen into the first one', async () => {
    await realClick(await rowEl(ids[0]))
    await modifiedMouseDown(await rowEl(ids[2]), 'shift')
    await browser.waitUntil(async () => (await probe()).rows.filter((r) => r.selected).length === 3, {
      timeout: 10_000,
      timeoutMsg: 'Shift+click did not select the range of three rows',
    })
    await $(`${MERGE_OPEN}:not([disabled])`).waitForExist({ timeout: 10_000 })
    await realClick(await $(MERGE_OPEN))
    await $(MERGE_OVERLAY).waitForDisplayed({ timeout: 10_000 })
    const listed = await browser.execute(
      () => document.querySelectorAll('[data-library-chapter-merge-list] li').length,
    )
    expect(listed).toBe(3)
    const result = await browser.execute(
      () => (document.querySelector('[data-library-chapter-merge-result]')?.textContent || '').trim(),
    )
    expect(result).toContain('3')

    await realClick(await $(MERGE_CONFIRM))
    await $(MERGE_OVERLAY).waitForExist({ reverse: true, timeout: 20_000 })
    await waitRows(3, 'merging three Chapters did not leave three rows')

    const shot = await probe()
    const survivor = shot.rows.find((r) => r.id === ids[0])
    expect(survivor).toBeDefined()
    expect(survivor.count).toBe('3 câu')
    // Later Chapters renumber: the former fourth and fifth are now second and third.
    expect(shot.rows.find((r) => r.id === ids[3]).ord).toBe('2')
    expect(shot.rows.find((r) => r.id === ids[4]).ord).toBe('3')
    const onDisk = await ipcValue('list_chapters')
    expect(onDisk.map((c) => c.chapter_id)).toEqual([ids[0], ids[3], ids[4]])
    expect(onDisk[0].segment_count).toBe(3)

    const focusInPage = await browser.execute(
      () => document.activeElement !== document.body && document.activeElement?.closest('[data-library-work-page]') !== null,
    )
    expect(focusInPage).toBe(true)
  })

  it('the split box lists the source sentences, keeps the first off, and splits at the picked one', async () => {
    await realClick(await rowEl(ids[4]))
    await $(`${SPLIT_OPEN}:not([disabled])`).waitForExist({ timeout: 10_000 })
    await realClick(await $(SPLIT_OPEN))
    await $(SPLIT_OVERLAY).waitForDisplayed({ timeout: 10_000 })
    await $('[data-library-chapter-split-list]').waitForDisplayed({ timeout: 20_000 })

    const radios = await browser.execute(() =>
      Array.from(document.querySelectorAll('[data-library-chapter-split-sentence]')).map((r) => ({
        id: r.getAttribute('data-library-chapter-split-sentence'),
        disabled: r.disabled,
      })),
    )
    expect(radios).toHaveLength(4)
    expect(radios[0].disabled).toBe(true)
    expect(await $(SPLIT_CONFIRM).isEnabled()).toBe(false)

    await realClick(await $(`[data-library-chapter-split-sentence="${radios[1].id}"]`))
    await browser.waitUntil(async () => $(SPLIT_CONFIRM).isEnabled(), {
      timeout: 10_000,
      timeoutMsg: 'picking the second sentence did not enable the confirm button',
    })
    await realClick(await $(SPLIT_CONFIRM))
    await $(SPLIT_OVERLAY).waitForExist({ reverse: true, timeout: 20_000 })
    await waitRows(4, 'the split did not add a fourth row')
    const onDisk = await ipcValue('list_chapters')
    expect(onDisk.map((c) => c.segment_count)).toEqual([3, 1, 1, 3])
  })

  it('three selected Chapters move to Paused in one action', async () => {
    const before = await probe()
    const first = before.rows[0].id
    const third = before.rows[2].id
    await realClick(await rowEl(first))
    await modifiedMouseDown(await rowEl(third), 'shift')
    await browser.waitUntil(async () => (await probe()).rows.filter((r) => r.selected).length === 3, {
      timeout: 10_000,
      timeoutMsg: 'the range of three rows was not selected',
    })
    const selected = (await probe()).rows.filter((r) => r.selected).map((r) => r.id)

    await realClick(await $(STATUS_MENU))
    await realClick(await $(STATUS_SET('paused')))
    await browser.waitUntil(
      async () => {
        const shot = await probe()
        return selected.every((id) => shot.rows.find((r) => r.id === id)?.status === 'paused')
      },
      { timeout: 20_000, timeoutMsg: 'the three selected Chapters did not all become Paused' },
    )
    const onDisk = await ipcValue('list_chapters')
    expect(onDisk.filter((c) => selected.includes(c.chapter_id)).map((c) => c.status)).toEqual([
      'paused',
      'paused',
      'paused',
    ])
    expect((await probe()).groups).toContain('paused')
  })

  it('the Export button opens the export screen and Escape returns focus to that same button', async () => {
    const shot = await probe()
    await realClick(await rowEl(shot.rows[0].id))
    await $(`${EXPORT_OPEN}:not([disabled])`).waitForExist({ timeout: 10_000 })
    await realClick(await $(EXPORT_OPEN))
    await $('.ex-panel').waitForDisplayed({ timeout: 10_000 })
    await browser.keys(['Escape'])
    await $('.ex-panel').waitForExist({ reverse: true, timeout: 10_000 })
    const onButton = await browser.execute(() => document.activeElement?.hasAttribute('data-export-open') ?? false)
    expect(onButton).toBe(true)
  })

  it('the back button returns focus to the open button of the same Work card', async () => {
    await realClick(await $('[data-library-work-page-close]'))
    await $(WORK_PAGE).waitForExist({ reverse: true, timeout: 10_000 })
    await browser.waitUntil(
      async () =>
        browser.execute((name) => {
          const active = document.activeElement
          if (active === null || !active.hasAttribute('data-library-open-work')) return false
          const card = active.closest('.work-cell')
          return (card?.querySelector('.work-name')?.textContent || '').trim() === name
        }, WORK_NAME),
      { timeout: 10_000, timeoutMsg: `focus did not return to the open button of "${WORK_NAME}"` },
    )
  })
})
