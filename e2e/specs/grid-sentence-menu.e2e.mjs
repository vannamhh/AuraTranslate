import { waitForGridRows } from '../support/gridWait.mjs'
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const CONFIRM_TIMEOUT_MS = 30_000

async function readSegmentsFromDisk() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('read_open_chapter_segments', {})
  })
}

async function seedTranslation(chapterId, id, text) {
  return browser.execute(
    async (c, i, t) => {
      const internals = window.__TAURI_INTERNALS__
      return internals.invoke('save_segment_targets', { chapterId: c, edits: [{ id: i, target_text: t }] })
    },
    chapterId,
    id,
    text,
  )
}

const COLUMN_SELECTOR = {
  src: (id) => `[data-col="src"][data-segment-id="${id}"]`,
  tgt: (id) => `[data-col="tgt"][data-segment-id="${id}"]`,
  num: (id) => `[data-num-segment-id="${id}"]`,
}

async function rowIds() {
  return await browser.execute(() =>
    [...document.querySelectorAll('[data-col="src"]')].map((el) => el.getAttribute('data-segment-id')),
  )
}

async function caretSegmentId() {
  return await browser.execute(() => {
    const anchor = window.getSelection()?.anchorNode ?? null
    const from = anchor instanceof Element ? anchor : (anchor?.parentElement ?? null)
    return from?.closest('[data-col="tgt"]')?.getAttribute('data-segment-id') ?? null
  })
}

async function omittedFlags() {
  return await browser.execute(() =>
    [...document.querySelectorAll('[data-col="src"]')].map((el) => el.classList.contains('omitted')),
  )
}

/**
 * The driver's pointer events are untrusted and carry no default action, so the press is
 * dispatched in the page in the order macOS delivers it: mousedown, contextmenu, mouseup.
 */
async function secondaryPress(selector, edge = false) {
  await browser.execute((sel, atEdge) => {
    const el = document.querySelector(sel)
    const rect = el.getBoundingClientRect()
    const init = {
      button: 2,
      bubbles: true,
      cancelable: true,
      clientX: atEdge ? window.innerWidth - 4 : Math.round(rect.x + rect.width * 0.3),
      clientY: atEdge ? window.innerHeight - 4 : Math.round(rect.y + rect.height / 2),
    }
    el.dispatchEvent(new MouseEvent('mousedown', init))
    el.dispatchEvent(new MouseEvent('contextmenu', init))
    el.dispatchEvent(new MouseEvent('mouseup', init))
  }, selector, edge)
  await $('[data-sentence-menu]').waitForExist({ timeout: 5_000 })
}

async function menuItem(label) {
  const items = await $$('[data-sentence-menu] button')
  for (const item of items) {
    if ((await item.getText()).includes(label)) return item
  }
  throw new Error(`menu item "${label}" missing`)
}

async function targetText(id) {
  return await browser.execute((sel) => document.querySelector(sel).textContent, COLUMN_SELECTOR.tgt(id))
}

async function typeAbcCaretAfterA(id) {
  await realClick(await $(COLUMN_SELECTOR.tgt(id)))
  await browser.pause(250)
  await browser.execute((sel) => {
    document.execCommand('insertText', false, 'abc')
    const walker = document.createTreeWalker(document.querySelector(sel), NodeFilter.SHOW_TEXT)
    const range = document.createRange()
    range.setStart(walker.nextNode(), 1)
    range.collapse(true)
    window.getSelection().removeAllRanges()
    window.getSelection().addRange(range)
  }, COLUMN_SELECTOR.tgt(id))
  await browser.pause(250)
  await expect(await targetText(id)).toBe('abc')
}

async function closeHistoryByEscAndType(id) {
  await $('.hist-panel').waitForExist({ timeout: 5_000 })
  await browser.keys(['Escape'])
  await $('.hist-panel').waitForExist({ timeout: 5_000, reverse: true })
  await browser.pause(250)
  await browser.execute(() => document.execCommand('insertText', false, 'zz'))
  await browser.pause(250)
  return await targetText(id)
}

describe('menu chuột phải của Lưới trong WKWebView thật', () => {
  for (const column of ['tgt', 'src', 'num']) {
    it(`nhấn phụ ở cột ${column} của câu 3 rồi Cắt bỏ câu: đúng câu 3 bị cắt bỏ, caret không phải câu 3 lúc bắt đầu`, async () => {
      await openWorkspaceWithWork(`Story 14.2 — ${column}`, '一。二。三。')
      await waitForGridRows(3, { what: 'Tác phẩm menu câu' })
      const ids = await rowIds()

      await realClick(await $(COLUMN_SELECTOR.tgt(ids[0])))
      await browser.pause(250)
      await expect(await caretSegmentId()).toBe(ids[0])

      await secondaryPress(COLUMN_SELECTOR[column](ids[2]))
      await realClick(await menuItem('Cắt bỏ câu'))
      await browser.waitUntil(async () => (await omittedFlags())[2] === true, {
        timeout: 10_000,
        timeoutMsg: 'Câu 3 không bị cắt bỏ sau khi chọn mục từ menu chuột phải',
      })

      await expect(await omittedFlags()).toEqual([false, false, true])
    })
  }

  it('đóng bằng Esc: caret về câu cũ và menu biến mất', async () => {
    await openWorkspaceWithWork('Story 14.2 — đóng', '一。二。三。')
    await waitForGridRows(3, { what: 'Tác phẩm menu câu' })
    const ids = await rowIds()
    await realClick(await $(COLUMN_SELECTOR.tgt(ids[0])))
    await browser.pause(250)

    await secondaryPress(COLUMN_SELECTOR.src(ids[1]))
    await browser.keys(['Escape'])
    await $('[data-sentence-menu]').waitForExist({ timeout: 5_000, reverse: true })

    await expect(await caretSegmentId()).toBe(ids[0])
  })

  it('điểm cắt ở câu khác: Tách câu tắt và title nói câu nào', async () => {
    await openWorkspaceWithWork('Story 14.2 — điểm cắt', '一二三四五六七八九十甲乙丙丁戊己庚辛。二。')
    await waitForGridRows(2, { what: 'Tác phẩm menu câu' })
    const ids = await rowIds()
    await browser.execute((id) => {
      const cell = document.querySelector(`[data-col="src"][data-segment-id="${id}"]`)
      const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
      let node = walker.nextNode()
      while (node !== null && node.data.length === 0) node = walker.nextNode()
      const range = document.createRange()
      range.selectNodeContents(node)
      const r = range.getClientRects()[0]
      cell.dispatchEvent(
        new MouseEvent('mouseup', {
          clientX: Math.round(r.x + r.width * 0.3),
          clientY: Math.round(r.y + r.height / 2),
          bubbles: true,
          cancelable: true,
          metaKey: true,
          ctrlKey: true,
        }),
      )
    }, ids[0])

    await secondaryPress(COLUMN_SELECTOR.src(ids[1]))
    const split = await menuItem('Tách câu')

    await expect(await split.isEnabled()).toBe(false)
    await expect(await split.getAttribute('title')).toContain('câu 1')
  })

  it('với menu mở, ⌘M đóng menu và không gộp câu', async () => {
    await openWorkspaceWithWork('Story 14.2 — cổng phím', '一。二。三。')
    await waitForGridRows(3, { what: 'Tác phẩm menu câu' })
    const ids = await rowIds()
    await realClick(await $(COLUMN_SELECTOR.tgt(ids[1])))
    await browser.pause(250)

    await secondaryPress(COLUMN_SELECTOR.src(ids[1]))
    await browser.keys(['Meta', 'm'])
    await $('[data-sentence-menu]').waitForExist({ timeout: 5_000, reverse: true })
    await browser.pause(1_000)

    await expect((await rowIds()).length).toBe(3)
  })

  it('Xác nhận chọn từ cột nguyên văn: đúng câu được ký trên đĩa và caret sang câu kế', async () => {
    await openWorkspaceWithWork('Story 14.2 — xác nhận', '一。二。三。')
    await waitForGridRows(3, { what: 'Tác phẩm menu câu' })
    const ids = await rowIds()
    const before = await readSegmentsFromDisk()
    await seedTranslation(before.chapter_id, Number(ids[1]), 'Bản dịch đã có sẵn.')
    await realClick(await $(COLUMN_SELECTOR.tgt(ids[0])))
    await browser.pause(250)

    await secondaryPress(COLUMN_SELECTOR.src(ids[1]))
    await realClick(await menuItem('Xác nhận'))

    await browser.waitUntil(
      async () => {
        const now = await readSegmentsFromDisk()
        return now.segments.find((x) => String(x.id) === ids[1])?.status === 'confirmed'
      },
      { timeout: CONFIRM_TIMEOUT_MS, timeoutMsg: 'Câu 2 không được ký trên đĩa sau khi chọn Xác nhận từ menu' },
    )
    const after = await readSegmentsFromDisk()
    for (const row of after.segments) {
      if (String(row.id) !== ids[1]) await expect(row.status).not.toBe('confirmed')
    }
    await browser.waitUntil(async () => (await caretSegmentId()) === ids[2], {
      timeout: 10_000,
      timeoutMsg: 'Caret không sang câu kế sau khi xác nhận',
    })
  })

  it('mở sát mép phải và mép dưới cửa sổ: menu nằm trọn trong khung nhìn', async () => {
    await openWorkspaceWithWork('Story 14.2 — mép', '一。二。三。')
    await waitForGridRows(3, { what: 'Tác phẩm menu câu' })
    const ids = await rowIds()

    await secondaryPress(COLUMN_SELECTOR.src(ids[0]), true)
    const rect = await browser.execute(() => {
      const r = document.querySelector('[data-sentence-menu]').getBoundingClientRect()
      return { left: r.left, top: r.top, right: r.right, bottom: r.bottom, w: window.innerWidth, h: window.innerHeight }
    })

    await expect(rect.left).toBeGreaterThanOrEqual(0)
    await expect(rect.top).toBeGreaterThanOrEqual(0)
    await expect(rect.right).toBeLessThanOrEqual(rect.w)
    await expect(rect.bottom).toBeLessThanOrEqual(rect.h)
  })
  it('Lịch sử mở bằng ⌘H rồi Esc: tiêu điểm về ô, gõ `zz` ra `azzbc`', async () => {
    await openWorkspaceWithWork('Story 14.10 — ⌘H', '一。二。三。')
    await waitForGridRows(3, { what: 'Tác phẩm menu câu' })
    const ids = await rowIds()
    await typeAbcCaretAfterA(ids[0])

    await browser.keys(['Meta', 'h'])

    await expect(await closeHistoryByEscAndType(ids[0])).toBe('azzbc')
  })

  it('Lịch sử mở từ mục menu chuột phải rồi Esc: tiêu điểm về ô, `zz` chèn vào mà không chữ nào bị đè', async () => {
    await openWorkspaceWithWork('Story 14.10 — menu Lịch sử', '一。二。三。')
    await waitForGridRows(3, { what: 'Tác phẩm menu câu' })
    const ids = await rowIds()
    await typeAbcCaretAfterA(ids[0])

    await secondaryPress(COLUMN_SELECTOR.tgt(ids[0]))
    await realClick(await menuItem('Lịch sử phiên bản'))

    const typed = await closeHistoryByEscAndType(ids[0])
    await expect(typed).toHaveLength(5)
    await expect(typed.replace('zz', '')).toBe('abc')
    await expect(typed.startsWith('zz')).toBe(false)
  })
})
