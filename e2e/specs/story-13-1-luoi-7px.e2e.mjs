/**
 * The 7px grid in a real WKWebView: the UI line is 21px, the panel bar is 35px, and a grid cell keeps
 * its pre-7px padding (DESIGN.md §Lưới 7px).
 *
 *     npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/story-13-1-luoi-7px.e2e.mjs
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TEXT = Array.from({ length: 3 }, (_, i) => `第${i + 1}句子内容在这里。`).join('')

describe('7px grid in a real webview', () => {
  before(async function () {
    this.timeout(300_000)
    await openWorkspaceWithWork(`e2e-grid-7px-${Date.now()}`, TEXT)
    await waitForGridRows(3, { col: 'tgt', timeout: 60_000, what: 'Chapter just built' })
  })

  it('a ui-md line is 21px tall', async () => {
    const lineHeight = await browser.execute(() => {
      const probe = document.createElement('div')
      probe.style.cssText = 'font-family: var(--face-ui-md); font-size: var(--font-ui-md); line-height: var(--leading-ui-md);'
      probe.textContent = 'x'
      document.body.appendChild(probe)
      const box = probe.getBoundingClientRect().height
      probe.remove()
      return box
    })
    expect(Math.abs(lineHeight - 21)).toBeLessThanOrEqual(0.5)
  })

  it('the panel bar is 35px tall', async () => {
    const heights = await browser.execute(() =>
      [...document.querySelectorAll('.dv-tabs-and-actions-container')]
        .map((el) => el.getBoundingClientRect().height)
        .filter((h) => h > 0),
    )
    expect(heights.length).toBeGreaterThan(0)
    for (const h of heights) expect(Math.abs(h - 35)).toBeLessThanOrEqual(0.5)
  })

  it('a grid cell keeps its pre-7px padding', async () => {
    const padding = await browser.execute(() => {
      const style = getComputedStyle(document.querySelector('.cell-tgt'))
      return { top: style.paddingTop, left: style.paddingLeft, right: style.paddingRight }
    })
    expect(padding).toEqual({ top: '4px', left: '8px', right: '8px' })
  })
})
