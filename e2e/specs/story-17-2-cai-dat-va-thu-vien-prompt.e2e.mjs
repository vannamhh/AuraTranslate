/**
 * Settings and the Prompt library run on the shared overlay frame: one width (`min(1120px,
 * window - 28px)`), no nested vertical scroller, text fields that fill their column, and number
 * fields that keep `ui-field--short`.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-17-2-cai-dat-va-thu-vien-prompt.e2e.mjs
 */
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'
import { measureOverlayFrame, resizeLogical } from '../support/overlayFrame.mjs'

const WIDTH_TOLERANCE_PX = 1
const SET = '.set-panel'
const LIB = '.pl-panel'

const expectedWidth = async () => Math.min(1120, (await browser.execute(() => window.innerWidth)) - 28)

async function invoke(command, args) {
  return browser.execute(async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a), command, args)
}

async function openSettingsSection(name) {
  const opener = await $('[data-settings-open]')
  await opener.waitForExist({ timeout: 30_000 })
  await realClick(opener)
  await realClick(await $('[data-tools-item="settings.open"]'))
  await $(SET).waitForDisplayed({ timeout: 10_000 })
  await realClick(await $(`[data-settings-section="${name}"]`))
}

async function closeSettings() {
  await browser.keys(['Escape'])
  await $(SET).waitForExist({ reverse: true, timeout: 10_000 })
}

describe('Story 17-2 -- Settings and Prompt library on the overlay frame', () => {
  it('the Prompt library at 1280px is 1120px with scrollers exactly .pl-list and .pl-ed, none nested', async () => {
    await openWorkspaceWithWork('Story 17-2 -- settings and library')
    await invoke('prompt_set_create', { tier: 'global', kind: 'translate', name: 'Story 17-2 -- bo mau', body: 'noi dung' })
    await realClick(await $('[data-ai-prompt-inspector-open]'))
    const opener = await $('[data-prompt-library-open]')
    await opener.waitForDisplayed({ timeout: 15_000 })
    await realClick(opener)
    await $(LIB).waitForDisplayed({ timeout: 15_000 })
    await $(`${LIB} .pl-list .pl-row-form`).waitForExist({ timeout: 10_000 })
    await realClick(await $(`${LIB} .pl-list .pl-row-form button`))

    await resizeLogical(1280, 720)
    const frame = await measureOverlayFrame(LIB)
    await expect(Math.abs(frame.width - (await expectedWidth()))).toBeLessThanOrEqual(WIDTH_TOLERANCE_PX)
    await expect(frame.nested).toBe(0)
    await expect(frame.scrollers.length).toBe(2)
    await expect(frame.scrollers.some((s) => s.includes('pl-list'))).toBe(true)
    await expect(frame.scrollers.some((s) => s.includes('pl-ed'))).toBe(true)

    await realClick(await $(`${LIB} [data-prompt-kind-option="proofread"]`))
    await $(`${LIB} [data-proofread-choice]`).waitForExist({ timeout: 10_000 })
    const proofread = await measureOverlayFrame(LIB)
    const regions = proofread.scrollers
    await expect(Math.abs(proofread.width - (await expectedWidth()))).toBeLessThanOrEqual(WIDTH_TOLERANCE_PX)
    await expect(proofread.nested).toBe(0)
    await expect(regions.length).toBe(2)
    await expect(regions.some((s) => s.includes('pl-list'))).toBe(true)
    await expect(regions.some((s) => s.includes('pl-ed'))).toBe(true)

    await browser.keys(['Escape'])
    await $(LIB).waitForExist({ reverse: true, timeout: 10_000 })
    await browser.keys(['Escape'])
  })

  it('Settings at 1280px is 1120px and its text fields fill the content column', async () => {
    await openSettingsSection('ai_and_model')
    await $('[data-ai-field="model"] input').waitForExist({ timeout: 10_000 })
    await resizeLogical(1280, 720)

    const frame = await measureOverlayFrame(SET)
    await expect(Math.abs(frame.width - (await expectedWidth()))).toBeLessThanOrEqual(WIDTH_TOLERANCE_PX)

    const widths = await browser.execute(() => {
      const main = document.querySelector('.set-main')
      const cs = getComputedStyle(main)
      const content = main.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight)
      const w = (sel) => document.querySelector(sel)?.getBoundingClientRect().width ?? null
      return {
        content,
        endpoint: w('[data-ai-field="endpoint"] input'),
        model: w('[data-ai-field="model"] input'),
        key: w('.ai-key-field input'),
      }
    })
    for (const name of ['endpoint', 'model', 'key']) {
      await expect(widths[name]).not.toBeNull()
      await expect(Math.abs(widths[name] - widths.content)).toBeLessThanOrEqual(1)
    }
  })

  it('the Temperature custom field keeps round(up, 8ch, 7px)', async () => {
    await realClick(await $('[data-ai-field="temperature"] [data-ai-chip="custom"]'))
    const input = await $('[data-ai-chip-input]')
    await input.waitForExist({ timeout: 10_000 })
    const { actual, expected } = await browser.execute(() => {
      const el = document.querySelector('[data-ai-chip-input]')
      const probe = document.createElement('span')
      probe.className = 'ui-field ui-field--lg'
      probe.style.cssText = 'width: round(up, 8ch, 7px); flex: none; position: absolute; visibility: hidden'
      el.parentElement.appendChild(probe)
      const expected = probe.getBoundingClientRect().width
      probe.remove()
      return { actual: el.getBoundingClientRect().width, expected }
    })
    await expect(Math.abs(actual - expected)).toBeLessThanOrEqual(0.5)
  })

  it('Settings scrolls only in .set-main on a long section', async () => {
    await realClick(await $('[data-settings-section="shortcuts"]'))
    await $('[data-shortcut-search]').waitForDisplayed({ timeout: 10_000 })
    const frame = await measureOverlayFrame(SET)
    await expect(frame.nested).toBe(0)
    await expect(frame.scrollers.some((s) => s.includes('set-main'))).toBe(true)
    await expect(frame.scrollers.some((s) => s.includes('set-panel'))).toBe(false)
    await expect(frame.scrollers.some((s) => s.includes('set-nav'))).toBe(true)

    const state = await browser.execute(() => {
      const main = document.querySelector('.set-main')
      const head = document.querySelector('.set-head')
      const nav = document.querySelector('.set-nav')
      const before = [head.getBoundingClientRect().top, nav.getBoundingClientRect().top]
      main.scrollTop = main.scrollHeight
      const after = [head.getBoundingClientRect().top, nav.getBoundingClientRect().top]
      return { overflowing: main.scrollHeight > main.clientHeight + 1, still: before[0] === after[0] && before[1] === after[1] }
    })
    await expect(state.overflowing).toBe(true)
    await expect(state.still).toBe(true)

    await browser.execute(() => {
      document.querySelector('.set-panel').style.maxHeight = '300px'
    })
    const low = await browser.execute(() => {
      const nav = document.querySelector('.set-nav')
      const main = document.querySelector('.set-main')
      const head = document.querySelector('.set-head')
      const tops = () => [head.getBoundingClientRect().top, main.getBoundingClientRect().top]
      const before = tops()
      const overflowing = nav.scrollHeight > nav.clientHeight + 1
      nav.scrollTop = nav.scrollHeight
      return { overflowing, moved: nav.scrollTop > 0, still: tops().every((t, i) => t === before[i]) }
    })
    const lowFrame = await measureOverlayFrame(SET)
    await browser.execute(() => {
      document.querySelector('.set-panel').style.maxHeight = ''
    })
    await expect(low.overflowing).toBe(true)
    await expect(low.moved).toBe(true)
    await expect(low.still).toBe(true)
    await expect(lowFrame.nested).toBe(0)
    await closeSettings()
  })
})
