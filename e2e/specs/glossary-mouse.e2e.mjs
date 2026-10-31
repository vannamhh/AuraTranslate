import { waitForGridRows } from '../support/gridWait.mjs'
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const COLUMN_SELECTOR = {
  src: (id) => `[data-col="src"][data-segment-id="${id}"]`,
  tgt: (id) => `[data-col="tgt"][data-segment-id="${id}"]`,
  num: (id) => `[data-num-segment-id="${id}"]`,
}
const STRIP_SOURCE = '.glossary-quick-add .gqa-input'

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

async function selectFirstChar(selector) {
  return await browser.execute((sel) => {
    const walker = document.createTreeWalker(document.querySelector(sel), NodeFilter.SHOW_TEXT)
    for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
      const start = node.data.search(/\S/)
      if (start < 0) continue
      window.getSelection().setBaseAndExtent(node, start, node, start + 1)
      return window.getSelection().toString()
    }
    throw new Error(`no text in ${sel}`)
  }, selector)
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

async function addTermItem() {
  const items = await $$('[data-sentence-menu] button')
  for (const item of items) {
    if ((await item.getText()).includes('Thêm thuật ngữ')) return item
  }
  throw new Error('menu item "Thêm thuật ngữ" missing')
}

async function stripValue() {
  return await browser.execute((sel) => {
    const input = document.querySelector(sel)
    return input instanceof HTMLInputElement ? input.value : null
  }, STRIP_SOURCE)
}

async function openChapterOfThree(title) {
  await openWorkspaceWithWork(title, '一。二。三。')
  await waitForGridRows(3, { what: 'Tác phẩm Thêm thuật ngữ bằng chuột' })
  return await rowIds()
}

describe('Thêm thuật ngữ từ menu câu trong WKWebView thật', () => {
  afterEach(async () => {
    if (await $('[data-sentence-menu]').isExisting()) await browser.keys(['Escape'])
    if (!(await $(STRIP_SOURCE).isExisting())) return
    await browser.execute((sel) => document.querySelector(sel).focus(), STRIP_SOURCE)
    await browser.keys(['Escape'])
    await $(STRIP_SOURCE).waitForExist({ timeout: 5_000, reverse: true })
  })

  it('trong vùng bôi ở nguyên văn: ô thuật ngữ điền đúng từ bôi và nhận tiêu điểm', async () => {
    const ids = await openChapterOfThree('Story 14.5 — vùng bôi')
    const picked = await selectFirstChar(COLUMN_SELECTOR.src(ids[1]))

    await secondaryPress(COLUMN_SELECTOR.src(ids[1]))
    await realClick(await addTermItem())
    await $(STRIP_SOURCE).waitForExist({ timeout: 10_000 })

    await $('[data-sentence-menu]').waitForExist({ timeout: 5_000, reverse: true })
    await expect(await stripValue()).toBe(picked)
    await expect(
      await browser.execute(() => document.activeElement?.classList.contains('gqa-input') ?? false),
    ).toBe(true)
  })

  it('ô số câu khi có từ bôi ở câu khác: ô thuật ngữ trống', async () => {
    const ids = await openChapterOfThree('Story 14.5 — ô số')
    await selectFirstChar(COLUMN_SELECTOR.src(ids[0]))

    await secondaryPress(COLUMN_SELECTOR.num(ids[2]))
    await realClick(await addTermItem())
    await $(STRIP_SOURCE).waitForExist({ timeout: 10_000 })

    await expect(await stripValue()).toBe('')
  })

  for (const column of ['src', 'num', 'tgt']) {
    it(`Esc sau khi nhắm câu 3 ở cột ${column}: caret và tiêu điểm về ô của câu 1`, async () => {
      const ids = await openChapterOfThree(`Story 14.5 — Esc ${column}`)
      await realClick(await $(COLUMN_SELECTOR.tgt(ids[0])))
      await browser.pause(250)
      await expect(await caretSegmentId()).toBe(ids[0])

      await secondaryPress(COLUMN_SELECTOR[column](ids[2]))
      await realClick(await addTermItem())
      await $(STRIP_SOURCE).waitForExist({ timeout: 10_000 })
      await browser.keys(['Escape'])
      await $(STRIP_SOURCE).waitForExist({ timeout: 5_000, reverse: true })

      await expect(await caretSegmentId()).toBe(ids[0])
      await expect(
        await browser.execute(() => document.activeElement?.getAttribute('data-segment-id') ?? null),
      ).toBe(ids[0])
    })
  }

  it('dải đang mở: mục tắt ở câu khác', async () => {
    const ids = await openChapterOfThree('Story 14.5 — đang mở')
    await secondaryPress(COLUMN_SELECTOR.src(ids[0]))
    await realClick(await addTermItem())
    await $(STRIP_SOURCE).waitForExist({ timeout: 10_000 })

    await secondaryPress(COLUMN_SELECTOR.src(ids[1]))

    await expect(await (await addTermItem()).isEnabled()).toBe(false)
  })
})
