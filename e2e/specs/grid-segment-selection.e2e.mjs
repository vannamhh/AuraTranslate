import { waitForGridRows } from '../support/gridWait.mjs'
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const NUM = (id) => `[data-num-segment-id="${id}"]`

async function rowIds() {
  return await browser.execute(() =>
    [...document.querySelectorAll('[data-col="src"]')].map((el) => el.getAttribute('data-segment-id')),
  )
}

async function selectedIds() {
  return await browser.execute(() =>
    [...document.querySelectorAll('[data-num-segment-id].row-selected')].map((el) => el.getAttribute('data-num-segment-id')),
  )
}

async function batchRunText() {
  return await browser.execute(() => document.querySelector('[data-ai-translate-batch-run]')?.textContent?.trim() ?? null)
}

async function dispatchMouse(selector, type, init = {}) {
  await browser.execute(
    (sel, kind, extra) => {
      const el = document.querySelector(sel)
      if (el === null) throw new Error(`không có phần tử ${sel}`)
      el.dispatchEvent(new MouseEvent(kind, { button: 0, buttons: 1, bubbles: true, cancelable: true, ...extra }))
    },
    selector,
    type,
    init,
  )
}

describe('chọn nhiều câu bằng chuột trên cột số của Lưới', () => {
  it('kéo qua ba số câu rồi Shift+bấm một câu xa: đúng các câu đó được tô, panel Đề xuất AI đếm đúng', async () => {
    await openWorkspaceWithWork('Story 14.3 — chọn nhiều', '一。二。三。四。五。')
    await waitForGridRows(5, { what: 'Tác phẩm chọn nhiều câu' })
    const ids = await rowIds()

    await dispatchMouse(NUM(ids[0]), 'mousedown')
    await dispatchMouse(NUM(ids[1]), 'mouseover')
    await dispatchMouse(NUM(ids[2]), 'mouseover')
    await dispatchMouse(NUM(ids[2]), 'mouseup', { buttons: 0 })
    await browser.waitUntil(async () => (await selectedIds()).length === 3, {
      timeout: 5_000,
      timeoutMsg: 'kéo qua ba số câu không tô đúng ba câu',
    })
    await expect(await selectedIds()).toEqual(ids.slice(0, 3))
    await expect(await batchRunText()).toContain('3')

    await dispatchMouse(NUM(ids[4]), 'mousedown', { shiftKey: true })
    await dispatchMouse(NUM(ids[4]), 'mouseup', { buttons: 0, shiftKey: true })
    await browser.waitUntil(async () => (await selectedIds()).length === 5, {
      timeout: 5_000,
      timeoutMsg: 'Shift+bấm câu xa không mở rộng vùng chọn tới câu 5',
    })
    await expect(await selectedIds()).toEqual(ids)
  })
})

describe('nút ‹ › chuyển Chương ở thanh trạng thái', () => {
  it('bấm › sang Chương kề rồi ‹ về lại, Lưới và thanh trạng thái đổi theo', async () => {
    await openWorkspaceWithWork('Story 14.3 — chuyển Chương', '一。二。三。四。')
    await waitForGridRows(4, { what: 'Tác phẩm chuyển Chương' })
    const ids = await rowIds()

    await browser.execute(async (sid) => {
      await window.__TAURI_INTERNALS__.invoke('split_chapter_at_segment', { segmentId: sid })
    }, Number(ids[2]))

    const waitForIds = (expected, what) =>
      browser.waitUntil(async () => JSON.stringify(await rowIds()) === JSON.stringify(expected), {
        timeout: 20_000,
        timeoutMsg: what,
      })

    await realClick(await $('[data-chapter-next]'))
    await waitForIds(ids.slice(2), 'bấm › không sang Chương sau')

    await realClick(await $('[data-chapter-prev]'))
    await waitForIds(ids.slice(0, 2), 'bấm ‹ không về Chương đầu')
  })
})
