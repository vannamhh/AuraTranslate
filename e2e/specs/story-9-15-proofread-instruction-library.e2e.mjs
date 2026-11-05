/**
 * The Prompt library's instruction card: picking a Global set writes the choice, and a chosen set
 * that is deleted afterwards shows up as a missing name, never as an empty select.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-9-15-proofread-instruction-library.e2e.mjs
 */
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

const KEEP = 'Story 9-15 -- giu'
const GONE = 'Story 9-15 -- mat'

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

async function pickKind(kind) {
  const tab = await $(`[data-prompt-kind-option="${kind}"]`)
  await tab.waitForDisplayed({ timeout: 15_000 })
  await realClick(tab)
}

describe('Story 9-15 -- Proofreader instruction library', () => {
  it('writes the Global choice and flags the chosen name once it is deleted', async () => {
    await openWorkspaceWithWork('Story 9-15 -- instruction library')
    await invoke('prompt_set_create', { tier: 'global', kind: 'proofread', name: KEEP, body: `so sanh {{source_segment}}\n${'dong chi dan\n'.repeat(40)}` })
    const gone = await invoke('prompt_set_create', { tier: 'global', kind: 'proofread', name: GONE, body: 'chi dan tam' })

    await realClick(await $('[data-ai-prompt-inspector-open]'))
    const opener = await $('[data-prompt-library-open]')
    await opener.waitForDisplayed({ timeout: 15_000 })
    await realClick(opener)
    await browser.waitUntil(async () => (await $('.pl-panel')).isExisting(), {
      timeout: 15_000,
      interval: 250,
      timeoutMsg: 'the Prompt library did not open',
    })

    await pickKind('proofread')
    const globalTrigger = await $('[data-proofread-choice-global] summary')
    await globalTrigger.waitForDisplayed({ timeout: 15_000 })
    await realClick(globalTrigger)
    const option = await $(`//input[@data-proofread-choice-option="${GONE}"]/parent::label`)
    await option.waitForDisplayed({ timeout: 15_000 })
    await realClick(option)

    await browser.waitUntil(
      async () => (await invoke('prompt_set_proofread_choice_get', {})).global?.name === GONE,
      { timeout: 15_000, interval: 250, timeoutMsg: 'the Global choice was not written' },
    )
    await expect(await $('[data-proofread-choice-missing="global"]').isExisting()).toBe(false)

    await invoke('prompt_set_delete', { tier: 'global', kind: 'proofread', id: gone.id })
    await pickKind('translate')
    await pickKind('proofread')

    const note = await $('[data-proofread-choice-missing="global"]')
    await note.waitForDisplayed({ timeout: 15_000 })
    await expect(await note.getText()).toContain(GONE)
    const read = await invoke('prompt_set_proofread_choice_get', {})
    await expect(read.global).toEqual({ name: GONE, missing: true })

    const keepRow = await $('.pl-panel').$(`button*=${KEEP}`)
    await realClick(keepRow)
    await (await $('.pl-ta-input')).waitForDisplayed({ timeout: 15_000 })
    await expect((await $('.pl-ta-mirror').$$('.pl-var')).length).toBe(0)

    const box = await browser.execute(() => {
      const area = document.querySelector('.pl-ta-input')
      return { scroll: area.scrollHeight, client: area.clientHeight }
    })
    await expect(box.scroll).toBeLessThanOrEqual(box.client + 1)
  })
})
