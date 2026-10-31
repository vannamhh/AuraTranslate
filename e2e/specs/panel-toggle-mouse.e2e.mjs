/**
 * The three panel switches in `Công cụ ▾`, driven by the pointer in a real WKWebView.
 *
 * The dock is read back from the menu (`aria-checked`) and from the `.dv-groupview` boxes, never from
 * product internals.
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { ensureFullLayoutTier } from '../support/layoutTier.mjs'
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const SWITCHES = ['layout.toggle_grid', 'layout.toggle_lookup', 'layout.toggle_ai_translation']
const ITEM = (id) => `[data-tools-item="${id}"]`

async function openMenu() {
  const trigger = await $('.tools-trigger')
  await realClick(trigger)
  await $(ITEM('layout.toggle_grid')).waitForDisplayed({ timeout: 10_000 })
}

async function checkedState() {
  return browser.execute((ids) => {
    const out = {}
    for (const id of ids) {
      const el = document.querySelector(`[data-tools-item="${id}"]`)
      out[id] = { checked: el?.getAttribute('aria-checked') === 'true', disabled: el?.disabled === true }
    }
    return out
  }, SWITCHES)
}

async function flipSwitch(id) {
  await openMenu()
  await realClick(await $(ITEM(id)))
  await $(ITEM(id)).waitForExist({ timeout: 5_000, reverse: true })
}

async function readMenu() {
  await openMenu()
  const state = await checkedState()
  await browser.keys(['Escape'])
  return state
}

async function groupBoxes() {
  return browser.execute(() =>
    [...document.querySelectorAll('.dv-groupview')]
      .map((el) => ({ el, r: el.getBoundingClientRect() }))
      .filter(({ r }) => r.width > 0)
      .map(({ el, r }) => ({
        panel: el.querySelector('.cell-tgt') ? 'grid' : el.querySelector('.ai-panel-head') ? 'ai' : el.querySelector('.lookup-body') ? 'lookup' : '?',
        x: Math.round(r.x),
        y: Math.round(r.y),
        w: Math.round(r.width),
        h: Math.round(r.height),
      }))
      .sort((a, b) => a.x - b.x || a.y - b.y),
  )
}

async function trackColors() {
  return browser.execute((ids) => {
    const out = {}
    for (const id of ids) {
      const track = document.querySelector(`[data-tools-item="${id}"] .ui-switch__track`)
      out[id] = track === null ? null : getComputedStyle(track).backgroundColor
    }
    return out
  }, SWITCHES)
}

async function waitForGroups(count) {
  await browser.waitUntil(async () => (await groupBoxes()).length === count, {
    timeout: 5_000,
    timeoutMsg: `dock did not settle on ${count} groups`,
  })
}

function near(a, b, tolerance) {
  return Math.abs(a - b) <= tolerance
}

describe('Công tắc panel bằng chuột trong WKWebView thật', () => {
  it('ba mục hiện ở Workspace, đều bật và chưa mục nào tắt', async () => {
    await openWorkspaceWithWork('Panel toggle — bật tắt', '一。二。')
    await waitForGridRows(2, { what: 'Tác phẩm công tắc panel' })
    await ensureFullLayoutTier()
    await waitForGroups(3)

    const state = await readMenu()
    for (const id of SWITCHES) expect(state[id]).toEqual({ checked: true, disabled: false })
  })

  for (const id of SWITCHES) {
    it(`${id}: tắt rồi bật, panel về đúng chỗ cũ và cỡ cũ`, async () => {
      const before = await groupBoxes()
      expect(before).toHaveLength(3)

      await flipSwitch(id)
      await waitForGroups(2)
      const hiddenState = await readMenu()
      expect(hiddenState[id].checked).toBe(false)
      await openMenu()
      const colors = await trackColors()
      await browser.keys(['Escape'])
      const other = SWITCHES.find((s) => s !== id)
      expect(colors[id]).not.toBeNull()
      expect(colors[id]).not.toEqual(colors[other])
      for (const other of SWITCHES.filter((s) => s !== id)) expect(hiddenState[other].checked).toBe(true)

      await flipSwitch(id)
      await waitForGroups(3)
      const after = await groupBoxes()
      const drift = []
      for (let i = 0; i < before.length; i += 1) {
        if (after[i].panel !== before[i].panel) drift.push(`panel[${String(i)}] ${before[i].panel} -> ${after[i].panel}`)
        for (const key of ['x', 'y', 'w', 'h']) {
          if (!near(after[i][key], before[i][key], 2)) drift.push(`${key}[${String(i)}] ${String(before[i][key])} -> ${String(after[i][key])}`)
        }
      }
      expect(drift).toEqual([])
      const state = await readMenu()
      for (const sw of SWITCHES) expect(state[sw].checked).toBe(true)
    })
  }

  it('panel cuối cùng: mục của nó tắt, hai mục kia bật được', async () => {
    await flipSwitch('layout.toggle_lookup')
    await flipSwitch('layout.toggle_ai_translation')
    await waitForGroups(1)
    const state = await readMenu()
    expect(state['layout.toggle_grid']).toEqual({ checked: true, disabled: true })
    expect(state['layout.toggle_lookup'].disabled).toBe(false)
    expect(state['layout.toggle_ai_translation'].disabled).toBe(false)

    await openMenu()
    await realClick(await $(ITEM('layout.toggle_grid')))
    await $(ITEM('layout.toggle_grid')).waitForDisplayed({ timeout: 2_000 })
    await browser.keys(['Escape'])

    await flipSwitch('layout.toggle_lookup')
    await flipSwitch('layout.toggle_ai_translation')
    await waitForGroups(3)
  })

  it('chữ chưa flush trong Đối chiếu còn nguyên sau khi tắt rồi bật trong 2 giây', async () => {
    const cell = await $('[data-col="tgt"]')
    const segmentId = await cell.getAttribute('data-segment-id')
    await realClick(cell)
    const typed = 'Chữ chưa kịp ghi.'
    const inserted = await browser.execute((text) => document.execCommand('insertText', false, text), typed)
    expect(inserted).toBe(true)

    const typedAt = Date.now()
    await flipSwitch('layout.toggle_grid')
    await waitForGroups(2)
    await flipSwitch('layout.toggle_grid')
    await waitForGroups(3)
    expect(Date.now() - typedAt).toBeLessThan(2000)

    await browser.waitUntil(
      async () =>
        (await browser.execute(
          (id) => document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`)?.textContent ?? null,
          segmentId,
        )) === typed,
      { timeout: 5_000, timeoutMsg: 'unflushed text was lost across the switch' },
    )
  })
})
