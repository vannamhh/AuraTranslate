// Two sample cards need an old `meta.json`: the spec overwrites one with a v1 copy (progress
// unknown) and one without a genre, then rescans.
import { mkdtempSync, readdirSync, readFileSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { tmpdir } from 'node:os'
import { aimWork, currentWorkName, reloadWorks, workCellNames, WORK_CELL } from '../support/libraryGrid.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const tag = (name) => `${name} ${RUN_TAG}`

const SAMPLE = [
  { name: 'Đấu Phá Thương Khung', lang: 'zh', genre: 'Huyền huyễn', chapters: 6, done: 2, doing: 1 },
  { name: 'Phàm Nhân Tu Tiên', lang: 'zh', genre: 'Tiên hiệp', chapters: 5, done: 5, doing: 0 },
  { name: 'Vạn Cổ Chí Tôn', lang: 'zh', genre: 'Tiên hiệp', chapters: 4, done: 1, doing: 1 },
  { name: 'The Hollow Crown', lang: 'en', genre: 'Fantasy', chapters: 3, done: 0, doing: 0, noGenre: true },
  { name: 'Quỷ Bí Chi Chủ', lang: 'zh', genre: 'Huyền huyễn', chapters: 5, done: 0, doing: 0, unknownProgress: true },
  { name: 'Toàn Chức Pháp Sư', lang: 'zh', genre: 'Đô thị', chapters: 4, done: 1, doing: 0 },
  { name: 'Ngã Dục Phong Thiên', lang: 'zh', genre: 'Tiên hiệp', chapters: 3, done: 2, doing: 1 },
  { name: 'Silent Archive', lang: 'en', genre: 'Khoa học viễn tưởng', chapters: 2, done: 0, doing: 0 },
]

async function waitForBridge() {
  await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
    timeout: 30_000,
    interval: 250,
  })
}

async function ipc(command, args = {}) {
  const result = await browser.execute(
    async (cmd, a) => {
      try {
        return { ok: true, value: await window.__TAURI_INTERNALS__.invoke(cmd, a) }
      } catch (err) {
        return { ok: false, detail: String(err && err.code ? err.code : err) }
      }
    },
    command,
    args,
  )
  if (!result.ok) throw new Error(`IPC ${command} failed: ${result.detail}`)
  return result.value
}

function sourceText(lang, sentences) {
  return Array.from({ length: sentences }, (_, i) => (lang === 'zh' ? `Cau ${i + 1}。` : `Sentence ${i + 1}. `)).join('')
}

async function createSampleWork(spec) {
  const created = await ipc('create_work_from_text', {
    name: tag(spec.name),
    sourceLang: spec.lang,
    genre: spec.genre,
    text: sourceText(spec.lang, spec.chapters * 2),
  })
  for (let i = 1; i < spec.chapters; i += 1) {
    const rows = await ipc('list_chapters')
    const last = rows[rows.length - 1]
    await ipc('open_chapter', { chapterId: last.chapter_id })
    const { segments } = await ipc('read_open_chapter_segments', {})
    if (segments.length > 2) await ipc('split_chapter_at_segment', { segmentId: segments[2].id })
  }
  const rows = await ipc('list_chapters')
  expect(rows.length).toBe(spec.chapters)
  for (let i = 0; i < rows.length; i += 1) {
    const status = i < spec.done ? 'done' : i < spec.done + spec.doing ? 'in_progress' : null
    if (status !== null) await ipc('set_chapter_status', { chapterId: rows[i].chapter_id, status })
  }
  return created.meta.work_id
}

function rewriteMeta(libraryRoot, workId, edit) {
  for (const entry of readdirSync(libraryRoot)) {
    if (!entry.endsWith('.atproj')) continue
    const path = join(libraryRoot, entry, 'meta.json')
    let meta
    try {
      meta = JSON.parse(readFileSync(path, 'utf8'))
    } catch {
      continue
    }
    if (meta.work_id !== workId) continue
    edit(meta)
    writeFileSync(path, JSON.stringify(meta))
    return
  }
  throw new Error(`no .atproj with work_id ${workId} under ${libraryRoot}`)
}

async function workIdByName(name) {
  const report = await ipc('library_list_works', { filter: null, genre: null, sourceLang: null, sort: null })
  const row = report.works.find((w) => w.name === name)
  if (row === undefined) throw new Error(`work "${name}" is not in the index`)
  return row.work_id
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

async function gridShape() {
  return browser.execute(() => {
    const grid = document.querySelector('.works-grid')
    const cols = getComputedStyle(grid).gridTemplateColumns.split(' ').filter((c) => c !== '').length
    const widths = Array.from(document.querySelectorAll('.works-grid .work-cell')).map((c) => c.getBoundingClientRect().width)
    return { cols, widths }
  })
}

async function inLibrary() {
  return browser.execute(() => document.querySelector('[data-library-grid]') !== null && document.querySelector('[data-col="src"]') === null)
}

async function activeIsAimedCard() {
  return browser.execute(
    (sel) => {
      const active = document.activeElement
      const aimed = document.querySelector(`${sel}[aria-current="true"]`)
      return {
        same: active !== null && active === aimed,
        name: (aimed?.querySelector('.work-name')?.textContent || '').trim(),
        activeTag: active?.tagName ?? '',
      }
    },
    WORK_CELL,
  )
}

/** `browser.keys(['Shift','Enter'])` reaches the page as a bare Enter (shiftKey is lost), so the chord is dispatched on the focused card. */
async function pressShiftEnterOnFocusedElement() {
  await browser.execute(() => {
    const target = document.activeElement ?? document.body
    target.dispatchEvent(new KeyboardEvent('keydown', { code: 'Enter', key: 'Enter', shiftKey: true, bubbles: true, cancelable: true }))
  })
}

async function applyTheme(theme) {
  await browser.execute(async (t) => {
    const mod = await import('/src/tokens/index.ts')
    mod.applyTheme(t)
  }, theme)
}

describe('Story 12-24: Library theo mockup lưới rộng', () => {
  let names = []

  before(async function () {
    this.timeout(600_000)
    const libraryRoot = process.env.AURATRANSLATE_E2E_LIBRARY_ROOT
    expect(libraryRoot).toBeDefined()
    await waitForBridge()

    const ids = new Map()
    for (const spec of SAMPLE) ids.set(spec.name, await createSampleWork(spec))

    for (const spec of SAMPLE) {
      const id = ids.get(spec.name)
      if (spec.noGenre) rewriteMeta(libraryRoot, id, (meta) => { meta.genre = '' })
      if (spec.unknownProgress) {
        rewriteMeta(libraryRoot, id, (meta) => {
          meta.meta_schema_version = 1
          delete meta.status
          delete meta.status_is_override
          delete meta.chapter_done_count
        })
      }
    }
    await ipc('library_rescan')
    await reloadWorks()
    await browser.waitUntil(async () => (await workCellNames()).length >= SAMPLE.length, {
      timeout: 30_000,
      timeoutMsg: 'the sample Library never filled the grid',
    })
    names = await workCellNames()
  })

  it('the sample Library shows all four card kinds without any "unknown" text', async () => {
    const cards = await browser.execute(() =>
      Array.from(document.querySelectorAll('.works-grid .work-cell')).map((cell) => ({
        name: (cell.querySelector('.work-name')?.textContent || '').trim(),
        hasBar: cell.querySelector('[role="progressbar"]') !== null,
        progress: (cell.querySelector('[data-library-work-progress]')?.textContent || '').trim(),
        badge: (cell.querySelector('[data-library-work-status]')?.textContent || '').trim(),
        meta: (cell.querySelector('[data-library-work-meta]')?.textContent || '').trim(),
        goLabel: (cell.querySelector('[data-library-continue-translation]')?.textContent || '').trim(),
      })),
    )
    const byName = (n) => cards.find((c) => c.name === tag(n))
    expect(byName('Quỷ Bí Chi Chủ').hasBar).toBe(false)
    expect(byName('Quỷ Bí Chi Chủ').badge).toBe('')
    expect(byName('Quỷ Bí Chi Chủ').progress).toContain('5')
    expect(byName('Phàm Nhân Tu Tiên').hasBar).toBe(true)
    expect(byName('Phàm Nhân Tu Tiên').badge).not.toBe('')
    expect(byName('Silent Archive').goLabel).not.toBe(byName('Vạn Cổ Chí Tôn').goLabel)
    expect(byName('The Hollow Crown').meta).not.toContain('Fantasy')
    expect(byName('Vạn Cổ Chí Tôn').meta).toContain('Tiên hiệp')
    expect(cards.some((c) => /chưa biết/i.test(c.progress) || /chưa biết/i.test(c.badge))).toBe(false)
  })

  it('Cmd+1, arrows to a later card, Shift+Enter opens the saved sentence, Cmd+1 returns focus to that card', async () => {
    const target = names[2]
    const targetId = await workIdByName(target)

    await ipc('open_work', { workId: targetId })
    const opened = await ipc('read_open_chapter_segments', {})
    expect(opened.segments.length).toBeGreaterThanOrEqual(2)
    const savedSegment = opened.segments[1].id
    await ipc('save_chapter_position', { chapterId: opened.chapter_id, segmentId: savedSegment })
    await reloadWorks()
    names = await workCellNames()
    const targetIndex = names.indexOf(target)
    expect(targetIndex).toBeGreaterThanOrEqual(1)
    const first = names[0]
    await ipc('open_work', { workId: await workIdByName(first) })

    await aimWork(first)
    await browser.keys(['Meta', '2'])
    await $('[data-col="src"]').waitForExist({ timeout: 30_000 })
    await browser.keys(['Meta', '1'])
    await browser.waitUntil(inLibrary, { timeout: 30_000, timeoutMsg: 'Cmd+1 did not return to Library' })
    await browser.waitUntil(async () => (await activeIsAimedCard()).same, {
      timeout: 10_000,
      timeoutMsg: 'focus did not return to the aimed card on Cmd+1',
    })

    for (let i = 0; i < targetIndex; i += 1) await browser.keys(['ArrowRight'])
    await browser.waitUntil(async () => (await currentWorkName()) === target, {
      timeout: 10_000,
      timeoutMsg: `${targetIndex} ArrowRight presses did not reach "${target}" (at "${await currentWorkName()}")`,
    })
    expect((await activeIsAimedCard()).same).toBe(true)

    await pressShiftEnterOnFocusedElement()
    await $('[data-col="src"]').waitForExist({ timeout: 30_000 })
    const restored = await ipc('read_open_chapter_segments', {})
    expect(restored.chapter_id).toBe(opened.chapter_id)
    expect(restored.caret_segment_id).toBe(savedSegment)

    await browser.keys(['Meta', '1'])
    await browser.waitUntil(inLibrary, { timeout: 30_000, timeoutMsg: 'Cmd+1 did not return to Library' })
    await browser.waitUntil(async () => (await activeIsAimedCard()).same && (await currentWorkName()) === target, {
      timeout: 10_000,
      timeoutMsg: `focus is not on the target card after returning (${JSON.stringify(await activeIsAimedCard())})`,
    })
  })

  it('arrows clamp at both ends of the grid', async () => {
    await aimWork(names[0])
    await browser.keys(['ArrowLeft'])
    await browser.keys(['ArrowUp'])
    expect(await currentWorkName()).toBe(names[0])
    await aimWork(names[names.length - 1])
    await browser.keys(['ArrowRight'])
    await browser.keys(['ArrowDown'])
    expect(await currentWorkName()).toBe(names[names.length - 1])
  })

  it('at 1150 px and 2560 px the column count grows and every card stays 240-305 px wide', async () => {
    await resizeLogical(1150, 790)
    const narrow = await gridShape()
    await resizeLogical(2560, 790)
    const wide = await gridShape()
    await resizeLogical(1490, 935)
    expect(narrow.cols).toBeGreaterThanOrEqual(4)
    expect(wide.cols).toBeGreaterThan(narrow.cols)
    for (const shape of [narrow, wide]) {
      for (const w of shape.widths) {
        expect(w).toBeGreaterThanOrEqual(239)
        expect(w).toBeLessThanOrEqual(306)
      }
    }
  })

  it('a three-line name on the cover uses on-cover at 1.66 leading, clamps at three lines and clears the bottom buttons', async () => {
    await resizeLogical(1150, 935)
    await aimWork(names[1])
    const LONG = 'Thượng Thiên Tục Ngã Đáo Thương Khung Chi Hậu Người Đọc Muốn Biết Đường Về Của Kiếm Khách Lãng Tử'
    const m = await browser.execute((long) => {
      const cell = document.querySelector('.works-grid .work-cell--current')
      const name = cell?.querySelector('.work-cover .work-name')
      const go = cell?.querySelector('.work-cover .work-go')
      const cover = cell?.querySelector('.work-cover')
      if (!(name instanceof HTMLElement) || !(go instanceof HTMLElement) || !(cover instanceof HTMLElement)) return null
      const before = name.textContent
      name.textContent = long
      const style = getComputedStyle(name)
      const fontSize = parseFloat(style.fontSize)
      const lineHeight = parseFloat(style.lineHeight)
      const n = name.getBoundingClientRect()
      const g = go.getBoundingClientRect()
      const c = cover.getBoundingClientRect()
      const out = {
        color: style.color,
        fontSize,
        ratio: lineHeight / fontSize,
        lines: Math.round(n.height / lineHeight),
        clipped: name.scrollHeight > name.clientHeight + 1,
        nameBottom: n.bottom,
        goTop: g.top,
        goBottom: g.bottom,
        coverBottom: c.bottom,
        coverHeight: c.height,
      }
      name.textContent = before
      return out
    }, LONG)
    expect(m).not.toBeNull()
    expect(m.color).toBe('rgb(251, 250, 246)')
    expect(m.fontSize).toBe(16)
    expect(m.ratio).toBeGreaterThanOrEqual(1.66 - 0.01)
    expect(m.lines).toBe(3)
    expect(m.clipped).toBe(true)
    expect(m.coverHeight).toBe(136)
    expect(m.nameBottom).toBeLessThanOrEqual(m.goTop)
    expect(m.goBottom).toBeLessThanOrEqual(m.coverBottom)
  })

  it('saves 1490x935 light and dark screenshots of the sample Library', async () => {
    await resizeLogical(1490, 935)
    await aimWork(names[1])
    const dir = mkdtempSync(join(tmpdir(), 'auratranslate-e2e-12-24-'))
    for (const theme of ['light', 'dark']) {
      await applyTheme(theme)
      await browser.pause(300)
      const path = join(dir, `library-${theme}.png`)
      await browser.saveScreenshot(path)
      const dpr = await browser.execute(() => window.devicePixelRatio)
      console.log(`[library-wide-grid] ${theme} dpr=${dpr} screenshot=${path}`)
    }
    await applyTheme('light')
  })
})
