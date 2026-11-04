import { waitForGridRows } from '../support/gridWait.mjs'
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TARGET = (id) => `[data-col="tgt"][data-segment-id="${id}"]`
const OVERLAYS = [
  ['ai.proofread.scope.open', '[data-proofread-scope]', '[data-proofread-scope-open]'],
  ['tm.manage.open', '.tm-panel', null],
  ['settings.open', '.set-panel', null],
]

async function firstTargetId() {
  return await browser.execute(() => document.querySelector('[data-col="tgt"]').getAttribute('data-segment-id'))
}

async function placeCaretAfterAb(id) {
  await realClick(await $(TARGET(id)))
  await browser.pause(250)
  await browser.execute((sel) => {
    document.execCommand('insertText', false, 'abc')
    const walker = document.createTreeWalker(document.querySelector(sel), NodeFilter.SHOW_TEXT)
    const range = document.createRange()
    range.setStart(walker.nextNode(), 2)
    range.collapse(true)
    window.getSelection().removeAllRanges()
    window.getSelection().addRange(range)
  }, TARGET(id))
  await browser.pause(250)
}

// The driver clears the selection on click, a real press keeps it: rebuild the Range in the cell, then click.
// addRange moves focus to the cell, so these cases prove only that the caret returns after Esc; the leak and the held-cell branch are guarded by the bench and Vitest.
async function pressWithSelection(buttonSel, id) {
  await browser.execute((btn, cellSel) => {
    const button = document.querySelector(btn)
    const walker = document.createTreeWalker(document.querySelector(cellSel), NodeFilter.SHOW_TEXT)
    const range = document.createRange()
    range.setStart(walker.nextNode(), 2)
    range.collapse(true)
    button.focus()
    window.getSelection().removeAllRanges()
    window.getSelection().addRange(range)
    button.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
  }, buttonSel, TARGET(id))
}

const cellText = (id) => browser.execute((sel) => document.querySelector(sel).textContent, TARGET(id))

describe('lớp phủ mở bằng chuột không để phím lọt xuống ô bản dịch', () => {
  for (const [item, panel, opener] of OVERLAYS) {
    it(`${item}: Esc rồi gõ chèn đúng điểm cũ của ô`, async () => {
      await openWorkspaceWithWork(`Story 14.11 — ${item}`, '一。二。')
      await waitForGridRows(2, { what: 'Tác phẩm giữ caret' })
      const id = await firstTargetId()
      await placeCaretAfterAb(id)

      if (opener === null) await realClick(await $('.tools-trigger'))
      await pressWithSelection(opener ?? `[data-tools-item="${item}"]`, id)
      await $(panel).waitForExist({ timeout: 5_000 })
      await browser.pause(250)
      await browser.execute(() => document.execCommand('insertText', false, 'Q'))
      await browser.pause(250)
      await expect(await cellText(id)).toBe('abc')

      await browser.keys(['Escape'])
      await $(panel).waitForExist({ timeout: 5_000, reverse: true })
      await browser.pause(250)
      await browser.execute(() => document.execCommand('insertText', false, 'zz'))
      await browser.pause(250)
      await expect(await cellText(id)).toBe('abzzc')
    })
  }

  it('nút thanh trạng thái: Esc rồi gõ chèn đúng điểm cũ của ô', async () => {
    await openWorkspaceWithWork('Story 14.11 — nút trạng thái', '一。二。')
    await waitForGridRows(2, { what: 'Tác phẩm giữ caret' })
    const id = await firstTargetId()
    await placeCaretAfterAb(id)

    await pressWithSelection('[data-glossary-queue-open]', id)
    await $('.gq-panel').waitForExist({ timeout: 5_000 })
    await browser.pause(250)
    await browser.execute(() => document.execCommand('insertText', false, 'Q'))
    await browser.pause(250)
    await expect(await cellText(id)).toBe('abc')
    await browser.keys(['Escape'])
    await $('.gq-panel').waitForExist({ timeout: 5_000, reverse: true })
    await browser.pause(250)
    await browser.execute(() => document.execCommand('insertText', false, 'zz'))
    await browser.pause(250)
    await expect(await cellText(id)).toBe('abzzc')
  })

  it('menu Công cụ ▾ đóng bằng cú bấm ra ngoài: gõ chèn đúng điểm cũ của ô', async () => {
    await openWorkspaceWithWork('Story 14.11 — menu', '一。二。')
    await waitForGridRows(2, { what: 'Tác phẩm giữ caret' })
    const id = await firstTargetId()
    await placeCaretAfterAb(id)

    await pressWithSelection('.tools-trigger', id)
    await $('.tools-list').waitForExist({ timeout: 5_000 })
    await browser.execute(() => document.execCommand('insertText', false, 'Q'))
    await browser.pause(250)
    await expect(await cellText(id)).toBe('abc')
    await realClick(await $('.titlebar'))
    await $('.tools-list').waitForExist({ timeout: 5_000, reverse: true })
    await browser.pause(300)
    await browser.execute(() => document.execCommand('insertText', false, 'zz'))
    await browser.pause(250)
    await expect(await cellText(id)).toBe('abzzc')
  })

  it('hợp âm mở lớp phủ: Esc rồi gõ chèn đúng điểm cũ của ô', async () => {
    await openWorkspaceWithWork('Story 14.11 — hợp âm', '一。二。')
    await waitForGridRows(2, { what: 'Tác phẩm giữ caret' })
    const id = await firstTargetId()
    await placeCaretAfterAb(id)

    await browser.keys(['Meta', 'Alt', 'm'])
    await $('.gm-panel').waitForExist({ timeout: 5_000 })
    await browser.keys(['Escape'])
    await $('.gm-panel').waitForExist({ timeout: 5_000, reverse: true })
    await browser.pause(250)
    await browser.execute(() => document.execCommand('insertText', false, 'zz'))
    await browser.pause(250)
    await expect(await cellText(id)).toBe('abzzc')
  })
})
