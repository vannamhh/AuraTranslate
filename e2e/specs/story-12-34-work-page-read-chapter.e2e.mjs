/**
 * Work page, per-row Read button: a Done Chapter row has Dịch and Đọc, any other row has
 * no Đọc; pressing Đọc on Chapter N opens Reading with N first; Mod+1 returns to the list
 * with the cursor still on N. One spec file = one app process; cases share one seeded Work
 * and run at 1280x860 logical.
 */
import { realClick } from '../support/pointer.mjs'
import { aimWork, openAimedWork, reloadWorks } from '../support/libraryGrid.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_NAME = `e2e-read-chapter-${RUN_TAG}`
const SOURCE_TEXT = 'Cau mot。Cau hai。Cau ba。Cau bon。Cau nam。'

const WORK_PAGE = '[data-library-work-page]'
const ROW = (id) => `[data-library-chapter-row][data-chapter-id="${id}"]`

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

/** Width must land; height is capped by the screen's available area, so the inner height is reported, not enforced. */
async function resizeLogical(width, height) {
  const dpr = await browser.execute(() => window.devicePixelRatio)
  for (const factor of [dpr, 1]) {
    await browser.setWindowSize(Math.round(width * factor), Math.round(height * factor))
    await browser.pause(400)
    const inner = await browser.execute(() => ({ w: window.innerWidth, h: window.innerHeight }))
    if (Math.abs(inner.w - width) <= 12) return inner
  }
  const last = await browser.execute(() => ({ w: window.innerWidth, h: window.innerHeight, sw: screen.availWidth, sh: screen.availHeight }))
  throw new Error(`window could not reach logical width ${width}: ${JSON.stringify(last)}`)
}

async function firstReadingTitle() {
  return browser.execute(() => (document.querySelector('h2.chapter-title')?.textContent || '').trim())
}

let ids = []

describe('Story 12-34 · Work page row Read button', () => {
  it('a Done row has Đọc, a row in progress has none', async () => {
    ids = await seedWork()
    await ipcValue('set_chapter_status', { chapterId: ids[1], status: 'done' })
    await ipcValue('set_chapter_status', { chapterId: ids[3], status: 'done' })
    await ipcValue('set_chapter_status', { chapterId: ids[2], status: 'in_progress' })
    await reloadWorks()
    await aimWork(WORK_NAME)
    await openAimedWork()
    await $(WORK_PAGE).waitForExist({ timeout: 20_000 })
    const inner = await resizeLogical(1280, 860)
    expect(Math.abs(inner.w - 1280)).toBeLessThanOrEqual(12)
    await browser.waitUntil(async () => (await $$('[data-library-chapter-row]')).length >= 4, {
      timeout: 20_000,
      timeoutMsg: 'the Work page did not list the Chapters',
    })

    const read = (id) => browser.execute((sel) => document.querySelector(`${sel} [data-library-row-read]`) !== null, ROW(id))
    const translate = (id) => browser.execute((sel) => document.querySelector(`${sel} [data-library-row-translate]`) !== null, ROW(id))
    expect(await read(ids[1])).toBe(true)
    expect(await read(ids[3])).toBe(true)
    expect(await read(ids[2])).toBe(false)
    expect(await translate(ids[2])).toBe(true)
  })

  it('Đọc on Chapter 4 opens Reading with Chapter 4 first; Mod+1 returns with the cursor on 4', async () => {
    await realClick(await $(`${ROW(ids[3])} [data-library-row-read]`))
    await browser.waitUntil(async () => (await firstReadingTitle()) === 'Chương 4', {
      timeout: 20_000,
      interval: 300,
      timeoutMsg: 'Reading did not start at Chapter 4',
    })

    await browser.keys(['Meta', '1'])
    await $(WORK_PAGE).waitForDisplayed({ timeout: 20_000 })
    await browser.waitUntil(
      async () =>
        browser.execute((sel) => document.querySelector(sel)?.getAttribute('aria-selected') === 'true', ROW(ids[3])),
      { timeout: 10_000, timeoutMsg: 'the list cursor is not on Chapter 4 after returning' },
    )
  })

  it('Đọc on Chapter 2 starts Reading at Chapter 2', async () => {
    await realClick(await $(`${ROW(ids[1])} [data-library-row-read]`))
    await browser.waitUntil(async () => (await firstReadingTitle()) === 'Chương 2', {
      timeout: 20_000,
      interval: 300,
      timeoutMsg: 'Reading did not start at Chapter 2',
    })
  })
})
