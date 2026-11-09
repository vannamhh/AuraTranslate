/**
 * The Prompt library body box lays a mirror layer under a transparent textarea; the mirror must
 * sit on the same glyph positions, the box must grow with the body instead of scrolling, and
 * Duplicate must save the typed body.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-14-prompt-library.e2e.mjs
 */
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

const NAME = 'Story 12-14 -- bo mau'
const TOLERANCE_PX = 1

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

function longBody() {
  const lines = []
  for (let i = 0; i < 40; i += 1) lines.push(`Dong ${i} voi {{glossary_terms}} va {{la_hoac}} de co chu de cuon.`)
  return lines.join('\n')
}

describe('Story 12-14 -- Prompt library', () => {
  it('the mirror tracks the textarea as the box grows with the body, and Duplicate keeps the typed body', async () => {
    await openWorkspaceWithWork('Story 12-14 -- prompt library')
    await invoke('prompt_set_create', { tier: 'global', kind: 'translate', name: NAME, body: longBody() })

    await realClick(await $('[data-ai-prompt-inspector-open]'))
    const opener = await $('[data-prompt-library-open]')
    await opener.waitForDisplayed({ timeout: 15_000 })
    await realClick(opener)
    await browser.waitUntil(async () => (await $('.pl-panel')).isExisting(), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the Prompt library did not open',
    })

    const row = await $('.pl-panel').$(`button*=${NAME}`)
    await realClick(row)
    const textarea = await $('.pl-ta-input')
    await textarea.waitForDisplayed({ timeout: 15_000 })

    const grown = await browser.execute(() => {
      const area = document.querySelector('.pl-ta-input')
      const mirror = document.querySelector('.pl-ta-mirror')
      const firstVar = mirror.querySelector('.pl-var')
      return {
        areaScroll: area.scrollHeight,
        areaClient: area.clientHeight,
        areaWidth: area.clientWidth,
        mirrorWidth: mirror.clientWidth,
        varCount: mirror.querySelectorAll('.pl-var').length,
        unknownCount: mirror.querySelectorAll('.pl-var-unknown').length,
        hasVar: firstVar !== null,
      }
    })
    // The box grows with the body and never scrolls; that is what lets the mirror skip
    // scroll syncing, so a box that scrolls here puts the two layers back out of step.
    await expect(grown.areaClient).toBeGreaterThan(0)
    await expect(grown.areaScroll).toBeLessThanOrEqual(grown.areaClient + TOLERANCE_PX)
    await expect(Math.abs(grown.areaWidth - grown.mirrorWidth)).toBeLessThanOrEqual(TOLERANCE_PX)
    await expect(grown.hasVar).toBe(true)
    await expect(grown.unknownCount).toBeGreaterThan(0)

    const wrap = await browser.execute(async () => {
      const area = document.querySelector('.pl-ta-input')
      const mirror = document.querySelector('.pl-ta-mirror')
      const show = async (text) => {
        area.value = text
        area.dispatchEvent(new Event('input', { bubbles: true }))
        await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))
      }
      await show('ngan')
      const unscrolled = { areaWidth: area.clientWidth, mirrorWidth: mirror.clientWidth }
      const heightGaps = []
      for (let narrow = 100; narrow < 260; narrow += 1) {
        await show(`${'dong\n'.repeat(20)}${'i'.repeat(narrow)} chu`)
        heightGaps.push(Math.abs(area.scrollHeight - mirror.scrollHeight))
      }
      return { unscrolled, worstHeightGap: Math.max(...heightGaps), overflows: area.scrollHeight > area.clientHeight }
    })
    await expect(wrap.overflows).toBe(false)
    await expect(Math.abs(wrap.unscrolled.areaWidth - wrap.unscrolled.mirrorWidth)).toBeLessThanOrEqual(TOLERANCE_PX)
    await expect(wrap.worstHeightGap).toBeLessThanOrEqual(TOLERANCE_PX)

    await textarea.setValue(`${longBody()}\nDong cuoi them tay.`)
    const duplicate = await $('.pl-dup-form button')
    await realClick(duplicate)

    await browser.waitUntil(
      async () => {
        const listed = await invoke('prompt_set_list', { kind: 'translate' })
        return listed.sets.some((s) => s.name === `${NAME} (bản sao)`)
      },
      { timeout: 15_000, interval: 250, timeoutMsg: 'Duplicate did not create the copy' },
    )
    const listed = await invoke('prompt_set_list', { kind: 'translate' })
    const original = listed.sets.find((s) => s.name === NAME)
    const copy = listed.sets.find((s) => s.name === `${NAME} (bản sao)`)
    await expect(original.body.endsWith('Dong cuoi them tay.')).toBe(true)
    await expect(copy.body).toBe(original.body)
  })
})
