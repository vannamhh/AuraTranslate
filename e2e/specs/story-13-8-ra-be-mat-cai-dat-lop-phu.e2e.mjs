/**
 * Settings and three representative overlays on the 7px grid in a real WKWebView: bordered rows
 * are a multiple of 7 tall, text edges sit on a multiple of 7 from the outer edge of a bordered
 * panel, gaps between buttons in one row are 7, and the paired footer buttons are at least 112px
 * wide (DESIGN.md §Lưới 7px).
 *
 *     npm run test:e2e -- --spec e2e/specs/story-13-8-ra-be-mat-cai-dat-lop-phu.e2e.mjs
 *
 * Alignment rows and the harvest footer only exist with a reviewer copy, which starts in a
 * native file dialog WebDriver cannot drive, so those two cases inject probe elements carrying
 * the panel's scoped attribute: they measure the shipped CSS, not overlay data.
 */
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TOLERANCE = 0.5

function onSeven(value) {
  const rest = Math.abs(value) % 7
  return Math.min(rest, 7 - rest) <= TOLERANCE
}

async function openFromTools(item) {
  const trigger = await $('.tools-trigger')
  await trigger.waitForDisplayed({ timeout: 30_000 })
  await realClick(trigger)
  const entry = await $(`[data-tools-item="${item}"]`)
  await entry.waitForDisplayed({ timeout: 10_000 })
  await realClick(entry)
}

async function openSettingsSection(name) {
  await openFromTools('settings.open')
  await $('.set-panel').waitForDisplayed({ timeout: 10_000 })
  await realClick(await $(`[data-settings-section="${name}"]`))
}

describe('Settings and overlays on the 7px grid', () => {
  afterEach(async () => {
    const open = await browser.execute(() => document.querySelector('.set-panel, .gh-panel, .tm-panel, .al-panel') !== null)
    if (!open) return
    await browser.keys(['Escape'])
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('.set-panel, .gh-panel, .tm-panel, .al-panel') === null),
      { timeout: 10_000, timeoutMsg: 'the overlay stayed open after Escape' },
    )
  })

  before(async function () {
    this.timeout(300_000)
    await openWorkspaceWithWork(`e2e-overlay-7px-${Date.now()}`, '甲。\n乙。')
  })

  describe('Settings', () => {
    it('Reading: each .sr-row is a multiple of 7 tall, .sr-label is a multiple of 7 wide and the main text edge sits on a multiple of 7', async () => {
      await openSettingsSection('reading')
      await $('.sr-row').waitForDisplayed({ timeout: 10_000 })
      const m = await browser.execute(() => {
        const panel = document.querySelector('.set-panel').getBoundingClientRect()
        const main = document.querySelector('.set-main')
        return {
          rows: [...document.querySelectorAll('.sr-row')].map((r) => r.getBoundingClientRect().height),
          labels: [...document.querySelectorAll('.sr-label')].map((l) => l.getBoundingClientRect().width),
          edge: main.firstElementChild.getBoundingClientRect().left - panel.left,
        }
      })
      expect(m.rows.length).toBeGreaterThan(0)
      expect(m.rows.filter((h) => !onSeven(h))).toEqual([])
      expect(m.labels.filter((w) => !onSeven(w))).toEqual([])
      expect(onSeven(m.edge) ? 0 : m.edge).toBe(0)
    })

    it('Layout: each .sl-item is a multiple of 7 tall and .sl-name is a multiple of 7 wide', async () => {
      await openSettingsSection('layout')
      await $('.sl-item').waitForDisplayed({ timeout: 10_000 })
      const m = await browser.execute(() => ({
        items: [...document.querySelectorAll('.sl-item')].map((r) => r.getBoundingClientRect().height),
        names: [...document.querySelectorAll('.sl-name')].map((l) => l.getBoundingClientRect().width),
      }))
      expect(m.items.length).toBeGreaterThan(0)
      expect(m.items.filter((h) => !onSeven(h))).toEqual([])
      expect(m.names.filter((w) => !onSeven(w))).toEqual([])
    })
  })

  describe('Overlays', () => {
    it('Glossary harvest footer: two ui-btn--pair buttons are 35 tall, at least 112 wide and 7 apart', async () => {
      await openFromTools('glossary.harvest.open')
      await $('.gh-panel').waitForDisplayed({ timeout: 10_000 })
      const m = await browser.execute(() => {
        const panel = document.querySelector('.gh-panel')
        const scope = [...panel.attributes].find((a) => a.name.startsWith('data-v-'))
        const actions = document.createElement('div')
        actions.className = 'gh-actions'
        for (const label of ['Accept', 'Leave']) {
          const b = document.createElement('button')
          b.type = 'button'
          b.className = `gh-act ui-btn ui-btn--lg ui-btn--pair${label === 'Accept' ? ' ui-btn--primary gh-act-primary' : ''}`
          b.textContent = label
          if (scope !== undefined) b.setAttribute(scope.name, '')
          actions.appendChild(b)
        }
        if (scope !== undefined) actions.setAttribute(scope.name, '')
        panel.appendChild(actions)
        const out = [...actions.querySelectorAll('.ui-btn--pair')].map((b) => {
          const r = b.getBoundingClientRect()
          return { left: r.left, right: r.right, width: r.width, height: r.height }
        })
        actions.remove()
        return out
      })
      expect(m.length).toBe(2)
      for (const b of m) {
        expect(Math.abs(b.height - 35)).toBeLessThanOrEqual(TOLERANCE)
        expect(b.width).toBeGreaterThanOrEqual(112 - TOLERANCE)
      }
      expect(Math.abs(m[1].left - m[0].right - 7)).toBeLessThanOrEqual(TOLERANCE)
    })

    it('TM manager: the first text edge is 14 from the outer edge at top and left and the .tm-actions buttons are 7 apart', async () => {
      await openFromTools('tm.manage.open')
      await $('.tm-panel').waitForDisplayed({ timeout: 10_000 })
      await $('.tm-actions').waitForDisplayed({ timeout: 10_000 })
      const m = await browser.execute(() => {
        const panel = document.querySelector('.tm-panel')
        const box = panel.getBoundingClientRect()
        const first = panel.firstElementChild.getBoundingClientRect()
        const buttons = [...document.querySelectorAll('.tm-actions > *')].map((b) => b.getBoundingClientRect())
        return {
          top: first.top - box.top,
          left: first.left - box.left,
          gaps: buttons.slice(1).map((b, i) => (b.top === buttons[i].top ? b.left - buttons[i].right : null)),
        }
      })
      expect(Math.abs(m.top - 14)).toBeLessThanOrEqual(TOLERANCE)
      expect(Math.abs(m.left - 14)).toBeLessThanOrEqual(TOLERANCE)
      const sameRow = m.gaps.filter((x) => x !== null)
      expect(sameRow.length).toBeGreaterThan(0)
      for (const g of sameRow) expect(Math.abs(g - 7)).toBeLessThanOrEqual(TOLERANCE)
    })

    it('Alignment: a bordered .al-row is a multiple of 7 tall', async () => {
      await openFromTools('export.alignment.open')
      await $('.al-panel').waitForDisplayed({ timeout: 10_000 })
      const height = await browser.execute(() => {
        const panel = document.querySelector('.al-panel')
        const scope = [...panel.attributes].find((a) => a.name.startsWith('data-v-'))
        const list = document.createElement('ul')
        const row = document.createElement('li')
        const label = document.createElement('label')
        list.style.cssText = 'list-style:none;margin:0;padding:0'
        row.className = 'al-row'
        label.className = 'al-row-label'
        label.textContent = '甲'
        for (const el of [list, row, label]) if (scope !== undefined) el.setAttribute(scope.name, '')
        row.appendChild(label)
        list.appendChild(row)
        panel.appendChild(list)
        const h = row.getBoundingClientRect().height
        list.remove()
        return h
      })
      expect(onSeven(height) ? 0 : height).toBe(0)
    })
  })
})
