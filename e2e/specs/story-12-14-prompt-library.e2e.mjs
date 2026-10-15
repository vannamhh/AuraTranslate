/**
 * The Prompt library body box lays a mirror layer under a transparent textarea; the mirror must
 * sit on the same glyph positions, also after the box scrolls, and Duplicate saves the typed body.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-14-prompt-library.e2e.mjs
 *
 * Not yet run green on Ice's machine (the webview stands at `tauri://localhost`); the nightly is
 * its first measurement.
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
  it('the mirror tracks the textarea after the box scrolls, and Duplicate keeps the typed body', async () => {
    await openWorkspaceWithWork('Story 12-14 -- prompt library')
    await invoke('prompt_set_create', { tier: 'global', name: NAME, body: longBody() })

    const opener = await $('[data-prompt-library-open]')
    await realClick(opener)
    await browser.waitUntil(async () => (await $('.pl-panel')).isExisting(), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the Prompt library did not open',
    })

    const row = await $(`.pl-row-form button*=${NAME}`)
    await realClick(row)
    const textarea = await $('.pl-ta-input')
    await textarea.waitForDisplayed({ timeout: 15_000 })

    const scrolled = await browser.execute(() => {
      const area = document.querySelector('.pl-ta-input')
      area.scrollTop = 120
      area.dispatchEvent(new Event('scroll'))
      return area.scrollTop
    })
    await expect(scrolled).toBeGreaterThan(0)

    const offsets = await browser.execute(() => {
      const area = document.querySelector('.pl-ta-input')
      const mirror = document.querySelector('.pl-ta-mirror')
      const firstVar = mirror.querySelector('.pl-var')
      return {
        areaTop: area.scrollTop,
        mirrorTop: mirror.scrollTop,
        areaWidth: area.clientWidth,
        mirrorWidth: mirror.clientWidth,
        varCount: mirror.querySelectorAll('.pl-var').length,
        unknownCount: mirror.querySelectorAll('.pl-var-unknown').length,
        hasVar: firstVar !== null,
      }
    })
    await expect(Math.abs(offsets.areaTop - offsets.mirrorTop)).toBeLessThanOrEqual(TOLERANCE_PX)
    await expect(Math.abs(offsets.areaWidth - offsets.mirrorWidth)).toBeLessThanOrEqual(TOLERANCE_PX)
    await expect(offsets.hasVar).toBe(true)
    await expect(offsets.unknownCount).toBeGreaterThan(0)

    await textarea.setValue(`${longBody()}\nDong cuoi them tay.`)
    const duplicate = await $('.pl-dup-form button')
    await realClick(duplicate)

    await browser.waitUntil(
      async () => {
        const listed = await invoke('prompt_set_list', {})
        return listed.sets.some((s) => s.name === `${NAME} (bản sao)`)
      },
      { timeout: 15_000, interval: 250, timeoutMsg: 'Duplicate did not create the copy' },
    )
    const listed = await invoke('prompt_set_list', {})
    const original = listed.sets.find((s) => s.name === NAME)
    const copy = listed.sets.find((s) => s.name === `${NAME} (bản sao)`)
    await expect(original.body.endsWith('Dong cuoi them tay.')).toBe(true)
    await expect(copy.body).toBe(original.body)
  })
})
