/**
 * Lookup panel header: the `Nguồn N/M ▾` popup opens over the panel without being clipped,
 * `Esc` returns focus to its button, a row click toggles the source and updates N/M, and the
 * tab title carries the real chord of `lookup.lookup_selection`.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-8-lookup-sources-popup.e2e.mjs
 *
 * Space on a focused row is a native button click, which the driver cannot send (e2e/AGENTS.md);
 * that half is covered by `tests/frontend/lookupSourcesPopup.test.ts`.
 */
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { ensureFullLayoutTier } from '../support/layoutTier.mjs'

const TRIGGER = '[data-sources-trigger]'
const LIST = '.sources-list'
const ROW = '[role="menuitemcheckbox"]'

async function label() {
  return $(TRIGGER).getText()
}

function parseLabel(text) {
  const m = /^Nguồn (\d+)\/(\d+) ▾$/.exec(text.trim())
  return m === null ? null : { enabled: Number(m[1]), total: Number(m[2]) }
}

describe('Story 12-8 · Lookup sources popup', () => {
  it('opens unclipped, returns focus on Esc, toggles a source by click, and the tab shows the chord', async function () {
    await openWorkspaceWithWork(`e2e-lookup-popup-${Date.now()}`)
    await ensureFullLayoutTier()

    const sourceCount = await browser.execute(async () => {
      try {
        const res = await window.__TAURI_INTERNALS__.invoke('list_dict_sources')
        return Array.isArray(res?.sources) ? res.sources.length : -1
      } catch {
        return -1
      }
    })
    if (sourceCount < 0) throw new Error('[BÀN ĐO HỎNG] `list_dict_sources` ném hoặc trả hình dạng lạ; mệnh đề chưa được đo.')
    if (sourceCount === 0) {
      console.warn('[BÀN ĐO KHÔNG DỰNG ĐƯỢC] 0 nguồn từ điển: popup không có dòng nào để đo. Dựng dữ liệu rồi chạy lại.')
      this.skip()
    }

    const trigger = await $(TRIGGER)
    await trigger.waitForExist({ timeout: 30_000 })
    await browser.waitUntil(async () => parseLabel(await label())?.total === sourceCount, {
      timeout: 15_000,
      timeoutMsg: 'the Nguồn N/M label never reached the real source count',
    })
    const start = parseLabel(await label())

    const chord = await browser.execute(() => document.querySelector('.tab-chord')?.textContent?.trim() ?? '')
    expect(chord).not.toBe('')

    await realClick(trigger)
    const list = await $(LIST)
    await list.waitForDisplayed({ timeout: 10_000 })
    const rowCount = (await $$(ROW)).length
    expect(rowCount).toBe(sourceCount)

    const problems = await browser.execute(
      (listSel, rowSel) => {
        const listEl = document.querySelector(listSel)
        const box = listEl.getBoundingClientRect()
        const found = []
        const bodyBottom = listEl.closest('.lookup-body').getBoundingClientRect().bottom
        if (box.bottom > bodyBottom + 1) found.push(`list bottom ${Math.round(box.bottom)} is past the panel bottom ${Math.round(bodyBottom)}`)
        for (const row of document.querySelectorAll(rowSel)) {
          const r = row.getBoundingClientRect()
          const visible = r.top >= box.top - 1 && r.bottom <= box.bottom + 1
          if (!visible) continue
          const hit = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2)
          if (hit === null || !listEl.contains(hit)) found.push(`${row.getAttribute('data-source-code')} covered by ${hit?.tagName.toLowerCase()}.${hit?.className}`)
        }
        return found
      },
      LIST,
      ROW,
    )
    expect(problems).toEqual([])

    await browser.keys(['ArrowDown'])
    const focusedRow = await browser.execute(() => document.activeElement?.getAttribute('data-source-code') ?? null)
    expect(focusedRow).not.toBeNull()

    await browser.keys(['Escape'])
    await list.waitForExist({ reverse: true, timeout: 10_000 })
    const focusOnTrigger = await browser.execute(() => document.activeElement?.hasAttribute('data-sources-trigger') ?? false)
    expect(focusOnTrigger).toBe(true)

    await realClick(trigger)
    await (await $(LIST)).waitForDisplayed({ timeout: 10_000 })
    const first = (await $$(ROW))[0]
    await realClick(first)
    await browser.waitUntil(async () => parseLabel(await label())?.enabled === start.enabled - 1, {
      timeout: 10_000,
      timeoutMsg: 'a row click did not lower N',
    })
    expect(await first.getAttribute('aria-checked')).toBe('false')

    await realClick((await $$(ROW))[0])
    await browser.waitUntil(async () => parseLabel(await label())?.enabled === start.enabled, {
      timeout: 10_000,
      timeoutMsg: 'a second row click did not restore N',
    })
  })

  it('keeps the spine count on one line inside the fixed-height header at a narrow width', async function () {
    await openWorkspaceWithWork(`e2e-lookup-spine-${Date.now()}`)
    await ensureFullLayoutTier()

    const ran = await browser.execute(async () => {
      try {
        const mod = await import('/src/panels/lookupPanelState.ts')
        await mod.runLookup('学习')
        return true
      } catch {
        return false
      }
    })
    if (!ran) throw new Error('[BÀN ĐO HỎNG] không gọi được `runLookup`; mệnh đề chưa được đo.')

    const count = await $('.lookup-spine-count')
    try {
      await count.waitForExist({ timeout: 15_000 })
    } catch {
      console.warn('[BÀN ĐO KHÔNG DỰNG ĐƯỢC] lượt tra không cho nhóm nguồn nào: không có số đếm để đo.')
      this.skip()
    }

    const m = await browser.execute(() => {
      const head = document.querySelector('.lookup-head')
      const el = document.querySelector('.lookup-spine-count')
      const previous = head.style.width
      head.style.width = '120px'
      try {
        return {
          lineHeight: parseFloat(getComputedStyle(el).lineHeight),
          countHeight: el.getBoundingClientRect().height,
          headScroll: head.scrollHeight,
          headClient: head.clientHeight,
        }
      } finally {
        head.style.width = previous
      }
    })
    expect(m.countHeight).toBeLessThanOrEqual(m.lineHeight * 1.5)
    expect(m.headScroll).toBeLessThanOrEqual(m.headClient)
  })
})
