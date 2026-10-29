/**
 * Widths written in `ch` land on the 7px grid in a real WKWebView: `round(up, Nch, 7px)` is supported
 * and yields a multiple of 7 (DESIGN.md §Lưới 7px — Bề rộng).
 *
 *     npm run test:e2e -- --spec e2e/specs/story-13-2-be-rong-ch.e2e.mjs
 */
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TOLERANCE = 0.01
const isMultipleOf7 = (px) => {
  if (!(px > 0)) return false
  const r = px % 7
  return Math.min(r, 7 - r) <= TOLERANCE
}

async function openSettingsSection(name) {
  const opener = await $('[data-settings-open]')
  await opener.waitForExist({ timeout: 30_000 })
  await realClick(opener)
  await realClick(await $('[data-tools-item="settings.open"]'))
  await $('.set-panel').waitForDisplayed({ timeout: 10_000 })
  await realClick(await $(`[data-settings-section="${name}"]`))
}

describe('ch widths on the 7px grid in a real webview', () => {
  before(async function () {
    this.timeout(300_000)
    await openWorkspaceWithWork(`e2e-ch-grid-${Date.now()}`, '他控制了局面。我们走吧。')
  })

  it('round(up, 24ch, 7px) is supported and measures a multiple of 7', async () => {
    const probe = await browser.execute(() => {
      const supported = CSS.supports('width', 'round(up, 24ch, 7px)')
      const el = document.createElement('div')
      el.style.cssText = 'position:absolute;visibility:hidden;font:var(--font-ui-md) var(--face-ui-md);width:round(up, 24ch, 7px)'
      document.body.appendChild(el)
      const rounded = el.getBoundingClientRect().width
      el.style.width = '24ch'
      const raw = el.getBoundingClientRect().width
      el.remove()
      return { supported, rounded, raw }
    })
    console.log('[13-2 PROBE]', JSON.stringify(probe))
    expect(probe.supported).toBe(true)
    expect(isMultipleOf7(probe.rounded)).toBe(true)
    expect(probe.rounded).toBeGreaterThanOrEqual(probe.raw - TOLERANCE)
  })

  it('a real ui-field--medium in Settings is a multiple of 7 wide', async () => {
    await openSettingsSection('export')
    const width = await browser.execute(() => document.querySelector('.set-panel .ui-field--medium')?.getBoundingClientRect().width ?? null)
    expect(width).not.toBeNull()
    expect(isMultipleOf7(width)).toBe(true)
    await browser.keys(['Escape'])
    await $('.set-panel').waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('.column is a multiple of 7 wide at all three reading levels, and each holds N to N + 7/1ch characters', async () => {
    await browser.execute(async () => {
      const internals = window.__TAURI_INTERNALS__
      const chapter = await internals.invoke('read_open_chapter')
      await internals.invoke('set_chapter_status', { chapterId: chapter.chapter_id, status: 'done' })
    })
    await browser.keys(['Meta', '3'])
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('.column') !== null), {
      timeout: 15_000,
      timeoutMsg: 'reading page did not build .column',
    })
    const measure = () =>
      browser.execute(() => {
        const col = document.querySelector('.column')
        const probe = document.createElement('div')
        probe.style.cssText = 'position:absolute;visibility:hidden;width:100ch'
        col.appendChild(probe)
        const chPx = probe.getBoundingClientRect().width / 100
        probe.remove()
        return { width: col.getBoundingClientRect().width, chPx }
      })
    for (const [level, n] of [['1', 62], ['2', 68], ['3', 76]]) {
      await browser.keys([level])
      await browser.pause(300)
      const m = await measure()
      expect(isMultipleOf7(m.width)).toBe(true)
      const chars = m.width / m.chPx
      expect(chars).toBeGreaterThanOrEqual(n - TOLERANCE)
      expect(chars).toBeLessThan(n + 7 / m.chPx)
    }
  })
})
