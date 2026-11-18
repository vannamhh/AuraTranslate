/**
 * The prompt-set import preview runs on the shared overlay frame: one width (`min(1120px,
 * window - 28px)`), exactly one vertical scroller, and a sticky foot that keeps the confirm
 * button in view at both ends of the body scroll. The file comes from the e2e pick-file hook
 * (`AURATRANSLATE_E2E_PICK_FILE`), so the real component opens without an OS dialog.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-17-1-khung-lop-phu.e2e.mjs
 */
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'
import { isInViewport, measureOverlayFrame, resizeLogical } from '../support/overlayFrame.mjs'

const FRAME = '.pi-panel'
const WIDTH_TOLERANCE_PX = 1

const expectedWidth = async () => Math.min(1120, (await browser.execute(() => window.innerWidth)) - 28)

async function openImportPreview() {
  if (!(await $('[data-prompt-import-open]').isDisplayed())) {
    await realClick(await $('[data-ai-prompt-inspector-open]'))
    const library = await $('[data-prompt-library-open]')
    await library.waitForDisplayed({ timeout: 15_000 })
    await realClick(library)
  }
  const importButton = await $('[data-prompt-import-open]')
  await importButton.waitForDisplayed({ timeout: 15_000 })
  await realClick(importButton)
  await browser.waitUntil(async () => (await $(FRAME)).isExisting(), {
    timeout: 15_000,
    interval: 250,
    timeoutMsg: 'the import preview did not open',
  })
}

async function closeImportPreview() {
  await realClick(await $('.pi-close'))
  await browser.waitUntil(async () => !(await $(FRAME).isExisting()), {
    timeout: 5_000,
    timeoutMsg: 'the import preview did not close',
  })
}

describe('Story 17-1 -- overlay frame on the prompt import preview', () => {
  it('is 1120px at a 1280px window, has one vertical scroller, and keeps the confirm button in view', async () => {
    await openWorkspaceWithWork('Story 17-1 -- overlay frame')
    await openImportPreview()
    await resizeLogical(1280, 720)

    const frame = await measureOverlayFrame(FRAME)
    await expect(Math.abs(frame.width - (await expectedWidth()))).toBeLessThanOrEqual(WIDTH_TOLERANCE_PX)
    await expect(frame.scrollers.length).toBe(1)
    await expect(frame.nested).toBe(0)

    const body = await $(FRAME).$('.ui-overlay__body')
    const confirm = await $(FRAME).$('.ui-overlay__foot .pi-act-primary')
    await expect(await isInViewport(confirm)).toBe(true)

    const overflowing = await browser.execute((el) => el.scrollHeight > el.clientHeight + 1, body)
    await expect(overflowing).toBe(true)

    await browser.execute((el) => {
      el.scrollTop = 0
    }, body)
    await expect(await isInViewport(confirm)).toBe(true)
    await browser.execute((el) => {
      el.scrollTop = el.scrollHeight
    }, body)
    await expect(await isInViewport(confirm)).toBe(true)

    await closeImportPreview()
  })

  it('shrinks to 772px at an 800px window', async () => {
    await resizeLogical(800, 720)
    await openImportPreview()

    const frame = await measureOverlayFrame(FRAME)
    await expect(Math.abs(frame.width - (await expectedWidth()))).toBeLessThanOrEqual(WIDTH_TOLERANCE_PX)
    await expect(frame.scrollers.length).toBe(1)

    await closeImportPreview()
  })
})
