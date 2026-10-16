/**
 * Hand joining opened from Cong cu in a real webview, on a Work with no reviewer copy.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-16-reviewer-import-hand-join.e2e.mjs
 *
 * The import itself starts in a native file dialog that WebDriver cannot drive, so the
 * import -> hand joining -> harvest chain, the two-column rows and the Review Mode link are covered
 * by `reviewerImport.test.ts`, `alignment.test.ts`, `reviewMode.test.ts` and
 * `reviewer_import_contract.rs`. Here the spec checks what a real webview adds: the overlay opens
 * on a Work without a copy and says why instead of showing an empty list, and Escape returns focus
 * to the button that opened it.
 */
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TRIGGER = '.tools-trigger'

describe('Story 12-16 -- hand joining from Cong cu', () => {
  it('opens without a reviewer copy, shows a reason instead of a blank list, and Escape returns focus to the menu button', async () => {
    await openWorkspaceWithWork('Story 12-16 -- hand join', '甲。\n乙。')
    const trigger = await $(TRIGGER)
    await trigger.waitForDisplayed({ timeout: 30_000 })
    await realClick(trigger)
    const item = await $('[data-tools-item="export.alignment.open"]')
    await item.waitForDisplayed({ timeout: 10_000 })
    await realClick(item)
    await $('.al-panel').waitForDisplayed({ timeout: 10_000 })
    await browser.waitUntil(
      async () => browser.execute(() => !document.querySelector('.al-panel .al-status[role="status"]')?.textContent?.includes('Đang đọc')),
      { timeout: 10_000, timeoutMsg: 'the overlay stayed on its loading line' },
    )
    const shape = await browser.execute(() => ({
      rows: document.querySelectorAll('.al-panel .al-row').length,
      reason: (document.querySelector('.al-panel .al-error, .al-panel .al-empty')?.textContent ?? '').trim().length > 0,
    }))
    expect(shape).toEqual({ rows: 0, reason: true })
    await browser.keys(['Escape'])
    await $('.al-panel').waitForExist({ reverse: true, timeout: 10_000 })
    expect(await browser.execute(() => document.activeElement?.matches('.tools-trigger') ?? false)).toBe(true)
  })
})
