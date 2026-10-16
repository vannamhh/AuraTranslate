/**
 * Settings frame in a real webview: open from `Công cụ ▾`, walk the four new sections, reverse-look-up
 * a chord in the shortcut search box (⌘M here), and open the AI section with ⌘T.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-18-settings.e2e.mjs
 *
 * The provider connection check needs a real server and key, and the shortcut export/import dialogs
 * are native: those are covered by `connection_check_contract.rs` (local fake server), the vitest
 * files for the file logic, and Ice's real-use pass.
 */
import { realClick } from '../support/pointer.mjs'

const OPENER = '[data-settings-open]'
const SETTINGS_ITEM = '[data-tools-item="settings.open"]'
const PANEL = '.set-panel'

const section = (name) => `[data-settings-section="${name}"]`

async function openSettings() {
  const opener = await $(OPENER)
  await opener.waitForExist({ timeout: 30_000 })
  await realClick(opener)
  await realClick(await $(SETTINGS_ITEM))
  await $(PANEL).waitForDisplayed({ timeout: 10_000 })
}

async function closeSettings() {
  await browser.keys(['Escape'])
  await $(PANEL).waitForExist({ reverse: true, timeout: 10_000 })
}

async function activeSection() {
  return browser.execute(
    () => document.querySelector('[data-settings-section][aria-current="true"]')?.getAttribute('data-settings-section') ?? null,
  )
}

async function counts(selector) {
  return browser.execute((sel) => document.querySelectorAll(sel).length, selector)
}

describe('Story 12-18 · Settings frame', () => {
  it('opens from Công cụ ▾ and shows the scope bar at the head of the window', async () => {
    await openSettings()
    await realClick(await $(section('ai_and_model')))
    await $('[data-settings-scope]').waitForExist({ timeout: 10_000 })
    expect(await counts('[data-settings-scope]')).toBe(1)
    expect(await counts('.ai-tier input[type="radio"]')).toBe(0)
  })

  it('walks Bố cục, Chế độ đọc, Dữ liệu and Cập nhật; each has a body, Cập nhật has no button', async () => {
    const bodies = [
      ['layout', '[data-settings-layout]'],
      ['reading', '[data-settings-reading-level]'],
      ['data', '[data-settings-data-root]'],
      ['update', '[data-settings-update]'],
    ]
    for (const [name, marker] of bodies) {
      await realClick(await $(section(name)))
      await $(marker).waitForExist({ timeout: 10_000 })
      const scopeLocked = await browser.execute(
        () => document.querySelector('[data-settings-scope-global]')?.disabled === true,
      )
      if (!scopeLocked) throw new Error(`${name}: the scope bar is not locked on Global`)
    }
    expect(
      await counts('[data-settings-update] button, [data-settings-update] input, [data-settings-update] a'),
    ).toBe(0)
  })

  it('typing ⌘M in the shortcut search box leaves only the merge command', async () => {
    await realClick(await $(section('shortcuts')))
    const search = await $('[data-shortcut-search]')
    await search.waitForDisplayed({ timeout: 10_000 })
    const total = await counts('[data-command-id]')
    expect(total).toBeGreaterThan(1)

    // WebDriver pointer events do not focus any text input in this webview (an AI field behaves the
    // same), so the box is focused by call; the chord path is what this case measures.
    await browser.execute(() => document.querySelector('[data-shortcut-search]').focus())
    await browser.keys(['Meta', 'm'])
    try {
      await browser.waitUntil(async () => (await counts('[data-command-id]')) === 1, { timeout: 5_000 })
    } catch {
      const seen = await browser.execute(() => ({
        value: document.querySelector('[data-shortcut-search]')?.value ?? null,
        rows: document.querySelectorAll('[data-command-id]').length,
        chord: document.querySelector('[data-shortcut-search-chord]')?.textContent ?? null,
        focused: document.activeElement?.getAttribute('data-shortcut-search') !== null,
        panelOpen: document.querySelector('.set-panel') !== null,
      }))
      throw new Error(`the ⌘M reverse lookup did not narrow the table to one row: ${JSON.stringify(seen)}`)
    }
    const ids = await browser.execute(() =>
      Array.from(document.querySelectorAll('[data-command-id]')).map((row) => row.getAttribute('data-command-id')),
    )
    expect(ids).toEqual(['editor.merge_segments'])
    expect(await counts('[data-shortcut-search-chord]')).toBe(1)
  })

  it('Escape clears the lookup then closes, and ⌘T opens Settings on the AI section', async () => {
    // The first Escape clears the chord in the search box and keeps Settings open; the second closes it.
    await browser.keys(['Escape'])
    expect(await counts('[data-shortcut-search-chord]')).toBe(0)
    expect(await counts(PANEL)).toBe(1)
    await closeSettings()
    await browser.keys(['Meta', 't'])
    await $(PANEL).waitForDisplayed({ timeout: 10_000 })
    await $('[data-ai-test-connection]').waitForExist({ timeout: 10_000 })
    expect(await activeSection()).toBe('ai_and_model')
    expect(await counts('[data-ai-provider]')).toBe(5)
    await closeSettings()
  })
})
