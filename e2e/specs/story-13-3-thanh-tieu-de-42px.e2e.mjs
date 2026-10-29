/**
 * The macOS title bar in a real WKWebView is drawn 42px tall from the top edge of the window, with a 91px
 * left inset for the three window buttons (AD-58 Rules 5–6, DESIGN.md §Lưới 7px).
 *
 *     npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/story-13-3-thanh-tieu-de-42px.e2e.mjs
 */
const TOLERANCE = 0.5

describe('macOS title bar in a real webview', () => {
  it('the title bar is 42px tall at the top edge, with the 91px inset on macOS', async () => {
    await $('.titlebar').waitForDisplayed({ timeout: 30_000 })
    const bar = await browser.execute(() => {
      const el = document.querySelector('.titlebar')
      const box = el.getBoundingClientRect()
      return {
        platform: document.documentElement.dataset.platform ?? '',
        top: box.top,
        height: box.height,
        paddingLeft: getComputedStyle(el).paddingLeft,
      }
    })
    expect(Math.abs(bar.top)).toBeLessThanOrEqual(TOLERANCE)
    expect(Math.abs(bar.height - 42)).toBeLessThanOrEqual(TOLERANCE)
    expect(bar.platform).toBe('mac')
    expect(bar.paddingLeft).toBe('91px')
  })
})
