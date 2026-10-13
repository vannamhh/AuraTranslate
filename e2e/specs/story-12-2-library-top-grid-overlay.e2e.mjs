/**
 * Library top bar, four-column grid, import overlay and first-run empty state in
 * the real webview. Each spec file gets a fresh Library root, so the first case sees a truly
 * empty Library. Layout reads the container-query tier from computed grid columns, not from
 * pixel guesses, and requires 4, 3 and 2 columns to each be observed.
 */
import { createWorkThroughForm } from '../support/importForm.mjs'
import { realClick } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const FORM_WORK = `e2e-122-form-${RUN_TAG}`
const IPC_WORKS = [1, 2, 3, 4].map((n) => `e2e-122-ipc${n}-${RUN_TAG}`)

const OPEN_BTN = '[data-library-import-open]'
const PANEL = '.li-panel'
const LIST_BTN = '[data-lifecycle-action="list_works"]'

async function waitForLibrary() {
  await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
    timeout: 30_000,
    interval: 250,
  })
  await $(OPEN_BTN).waitForDisplayed({ timeout: 30_000 })
}

async function createViaIpc(name) {
  const result = await browser.execute(async (workName) => {
    try {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: workName,
        sourceLang: 'zh',
        genre: 'Tien hiep',
        text: 'Một câu nguồn để bộ nhập có việc mà làm.',
      })
      return { ok: true, detail: '' }
    } catch (err) {
      return { ok: false, detail: String(err && err.code ? err.code : err) }
    }
  }, name)
  expect(result.detail).toBe('')
  expect(result.ok).toBe(true)
}

async function cardNames() {
  return browser.execute(() =>
    Array.from(document.querySelectorAll('.works-grid .work-cell .work-name')).map((n) => (n.textContent || '').trim()),
  )
}

async function activeIsOpenButton() {
  return browser.execute(() => document.activeElement?.hasAttribute('data-library-import-open') ?? false)
}

/** `setWindowSize` units differ by driver and dpr, so accept whichever factor lands the logical width. */
async function resizeToLogicalWidth(width) {
  const dpr = await browser.execute(() => window.devicePixelRatio)
  let inner = 0
  for (const factor of [dpr, 1]) {
    await browser.setWindowSize(Math.round(width * factor), 790 * Math.round(factor))
    await browser.pause(300)
    inner = await browser.execute(() => window.innerWidth)
    if (Math.abs(inner - width) <= 12) return inner
  }
  throw new Error(`window could not reach logical width ${width} (last innerWidth ${inner}, dpr ${dpr})`)
}

async function gridShape() {
  return browser.execute(() => {
    const grid = document.querySelector('.works-grid')
    const mode = document.querySelector('.mode')
    const cs = getComputedStyle(mode)
    const content = mode.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight)
    const cols = getComputedStyle(grid).gridTemplateColumns.split(' ').filter((c) => c !== '').length
    const cover = document.querySelector('.works-grid .work-cover')
    const rect = cover.getBoundingClientRect()
    const gridRect = grid.getBoundingClientRect()
    return { cols, content, ratio: rect.width / rect.height, gridWidth: gridRect.width }
  })
}

describe('Story 12-2: Library top bar, grid and import overlay', () => {
  it('empty Library shows the invitation, no import form in the page, and the overlay opens and closes with focus back on the button', async () => {
    await waitForLibrary()
    await $('[data-library-empty]').waitForDisplayed({ timeout: 30_000 })
    expect(await browser.execute(() => document.querySelectorAll('[data-library-empty-invite]').length)).toBe(2)
    expect(await browser.execute(() => document.querySelector('.import-form') === null)).toBe(true)
    expect(await browser.execute(() => document.querySelector('[data-library-search-button]') === null)).toBe(true)
    expect(await browser.execute(() => document.querySelector('.works-grid .work-cell') === null)).toBe(true)

    await realClick(await $('[data-library-empty-invite="file"]'))
    await $(`${PANEL} .import-form`).waitForDisplayed({ timeout: 10_000 })
    await browser.keys(['Escape'])
    await $(PANEL).waitForExist({ reverse: true, timeout: 10_000 })

    await realClick(await $(OPEN_BTN))
    await $(`${PANEL} .import-form`).waitForDisplayed({ timeout: 10_000 })
    expect(await browser.execute(() => document.activeElement?.closest('.import-form') !== null)).toBe(true)
    await browser.keys(['Escape'])
    await $(PANEL).waitForExist({ reverse: true, timeout: 10_000 })
    expect(await activeIsOpenButton()).toBe(true)
    expect(await browser.execute(() => document.querySelector('.import-form') === null)).toBe(true)
  })

  it('creating a Work through the overlay closes it and puts a card with the source language in the grid', async () => {
    await createWorkThroughForm(FORM_WORK)
    await browser.waitUntil(async () => (await cardNames()).includes(FORM_WORK), {
      timeout: 30_000,
      timeoutMsg: `card "${FORM_WORK}" did not appear after the overlay closed`,
    })
    expect(await browser.execute(() => document.querySelector('.li-panel') === null)).toBe(true)
    expect(await browser.execute(() => document.querySelector('[data-library-empty]') === null)).toBe(true)
    const card = await browser.execute((name) => {
      const cell = Array.from(document.querySelectorAll('.works-grid .work-cell')).find(
        (c) => (c.querySelector('.work-name')?.textContent || '').trim() === name,
      )
      return {
        lang: (cell?.querySelector('[data-library-work-lang]')?.textContent || '').trim(),
        meta: (cell?.querySelector('[data-library-work-meta]')?.textContent || '').trim(),
        text: cell?.textContent || '',
      }
    }, FORM_WORK)
    expect(card.lang).not.toBe('')
    expect(card.meta).toMatch(/^(vừa sửa|sửa )/)
    expect(card.text).not.toMatch(/Invalid|undefined|NaN/)
  })

  it('grid is 4 columns at 1440 and narrows to 3 then 2; covers keep their ratio', async () => {
    for (const name of IPC_WORKS) await createViaIpc(name)
    await realClick(await $(LIST_BTN))
    await browser.waitUntil(async () => (await cardNames()).length >= 5, {
      timeout: 30_000,
      timeoutMsg: 'fewer than five cards after reload',
    })

    const seen = []
    for (const [width, expected] of [
      [1440, 4],
      [1000, 3],
      [800, 2],
    ]) {
      await resizeToLogicalWidth(width)
      let shape = null
      try {
        await browser.waitUntil(
          async () => {
            shape = await gridShape()
            return shape.cols === expected
          },
          { timeout: 10_000 },
        )
      } catch {
        throw new Error(`at logical width ${width} expected ${expected} columns, last read ${JSON.stringify(shape)}`)
      }
      const tier = shape.content >= 1000 ? 4 : shape.content >= 800 ? 3 : 2
      expect(shape.cols).toBe(tier)
      expect(Math.abs(shape.ratio - 1)).toBeLessThan(0.05)
      expect(shape.gridWidth).toBeGreaterThan(shape.content - 2)
      seen.push(shape.cols)
    }
    expect(seen).toEqual([4, 3, 2])
  })

  it('the old blocks sit below the grid in the old order', async () => {
    const order = await browser.execute(() => {
      const pick = (sel) => {
        const el = document.querySelector(sel)
        return el === null ? null : el.getBoundingClientRect().top + window.scrollY
      }
      return {
        grid: pick('.works-grid'),
        root: pick('.root-block'),
        open: pick('.open-work-wrap'),
      }
    })
    expect(order.grid).not.toBeNull()
    expect(order.root).toBeGreaterThan(order.grid)
    expect(order.open).toBeGreaterThan(order.root)
  })
})
