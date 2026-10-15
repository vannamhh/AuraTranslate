/**
 * Glossary queue opener in the Workspace, the harvest review screen reached from Cong cu, and the
 * Glossary manager table.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-13-glossary-harvest.e2e.mjs
 *
 * Harvest candidates only come from a reviewer copy, which this spec does not build: the
 * three-candidate flow (accept two, leave one) is covered by `review_harvest_contract.rs` and
 * `glossaryHarvest.test.ts`. Here the screen is opened empty, so the spec checks the menu entry,
 * the empty sentence and the focus return in a real webview.
 */
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TRIGGER = '.tools-trigger'
const QUEUE_OPENER = '[data-glossary-queue-open]'

async function activeMatches(selector) {
  return browser.execute((s) => document.activeElement?.matches(s) ?? false, selector)
}

describe('Story 12-13 -- Glossary queue, harvest review and manager', () => {
  it('the Workspace opener opens the queue and Escape returns focus to it', async () => {
    await openWorkspaceWithWork('Story 12-13 -- glossary queue', '甲。\n乙。')
    const opener = await $(QUEUE_OPENER)
    await opener.waitForDisplayed({ timeout: 30_000 })
    await realClick(opener)
    await $('.gq-panel').waitForDisplayed({ timeout: 10_000 })
    await browser.keys(['Escape'])
    await $('.gq-panel').waitForExist({ reverse: true, timeout: 10_000 })
    expect(await activeMatches(QUEUE_OPENER)).toBe(true)
  })

  it('Cong cu -> Duyet thu hoach thuat ngu opens the screen, shows the empty sentence, and Escape returns focus to the menu button', async () => {
    const trigger = await $(TRIGGER)
    await trigger.waitForDisplayed({ timeout: 30_000 })
    await realClick(trigger)
    const item = await $('[data-tools-item="glossary.harvest.open"]')
    await item.waitForDisplayed({ timeout: 10_000 })
    await realClick(item)
    await $('.gh-panel').waitForDisplayed({ timeout: 10_000 })
    expect(await browser.execute(() => document.querySelectorAll('.gh-row').length)).toBe(0)
    await browser.keys(['Escape'])
    await $('.gh-panel').waitForExist({ reverse: true, timeout: 10_000 })
    expect(await activeMatches('.tools-trigger')).toBe(true)
  })

  it('the Glossary manager shows a seven-column table with a tier switch', async () => {
    const trigger = await $(TRIGGER)
    await realClick(trigger)
    const item = await $('[data-tools-item="glossary.manage.open"]')
    await item.waitForDisplayed({ timeout: 10_000 })
    await realClick(item)
    await $('.gm-panel').waitForDisplayed({ timeout: 10_000 })
    const shape = await browser.execute(() => ({
      columns: document.querySelectorAll('.gm-panel thead th').length,
      tierOptions: document.querySelectorAll('.gm-panel input[name="gm-tier"]').length,
    }))
    expect(shape).toEqual({ columns: 7, tierOptions: 3 })
    await browser.keys(['Escape'])
    await $('.gm-panel').waitForExist({ reverse: true, timeout: 10_000 })
  })
})
