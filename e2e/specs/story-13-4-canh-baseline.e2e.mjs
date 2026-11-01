/**
 * Baseline alignment of horizontal rows in a real WKWebView: text, mono chips and 14px icons share one
 * baseline, and icon-only buttons stand in the row through an empty line box (DESIGN.md §Lưới 7px).
 *
 *     npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/story-13-4-canh-baseline.e2e.mjs
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TEXT = Array.from({ length: 3 }, (_, i) => `第${i + 1}句子内容在这里。`).join('')
const TOLERANCE = 0.5
const CHEVRON_DROP = 3

/** Bottom edge of a zero-size inline-block probe placed first inside `host`: the host's text baseline. */
const PROBE = `
  function baselineOf(host) {
    const probe = document.createElement('span')
    probe.style.cssText = 'display:inline-block;width:0;height:0;vertical-align:baseline'
    host.insertBefore(probe, host.firstChild)
    const y = probe.getBoundingClientRect().bottom
    probe.remove()
    return y
  }
`

async function measure(body) {
  return browser.execute(new Function(`${PROBE}\n${body}`))
}

describe('Baseline alignment in a real webview', () => {
  before(async function () {
    this.timeout(300_000)
    await openWorkspaceWithWork(`e2e-baseline-${Date.now()}`, TEXT)
    await waitForGridRows(3, { col: 'tgt', timeout: 60_000, what: 'Chapter just built' })
  })

  it('title bar: mode tab, Work name and Tools label share a baseline, the chevron bottom sits 3 px below it', async () => {
    const m = await measure(`
      const bar = document.querySelector('.titlebar')
      const tab = bar.querySelector('.mode-tab')
      const work = bar.querySelector('.titlebar-work')
      const tools = bar.querySelector('.tools-trigger .ui-row')
      const chevron = tools.querySelector('.app-icon')
      return {
        tab: baselineOf(tab),
        work: work ? baselineOf(work) : null,
        tools: baselineOf(tools),
        chevronBottom: chevron.getBoundingClientRect().bottom,
      }
    `)
    expect(Math.abs(m.tab - m.tools)).toBeLessThanOrEqual(TOLERANCE)
    if (m.work !== null) expect(Math.abs(m.tab - m.work)).toBeLessThanOrEqual(TOLERANCE)
    expect(Math.abs(m.chevronBottom - (m.tools + CHEVRON_DROP))).toBeLessThanOrEqual(TOLERANCE)
  })

  it('an icon-only segment item stands level with a text button', async () => {
    const m = await measure(`
      const bar = document.querySelector('.titlebar')
      const item = bar.querySelector('.layout-switch .ui-seg__item')
      const btn = bar.querySelector('.tools-trigger')
      return {
        itemTop: item.getBoundingClientRect().top,
        btnTop: btn.getBoundingClientRect().top,
        itemHeight: item.getBoundingClientRect().height,
        before: getComputedStyle(item, '::before').content,
      }
    `)
    expect(m.before).not.toBe('none')
    expect(Math.abs(m.itemTop - m.btnTop)).toBeLessThanOrEqual(TOLERANCE)
    expect(Math.abs(m.itemHeight - 28)).toBeLessThanOrEqual(TOLERANCE)
  })

  it('panel bar: tab text and key chip share a baseline', async () => {
    const pairs = await measure(`
      return [...document.querySelectorAll('.tab')]
        .filter((tab) => tab.querySelector('.tab-chord'))
        .map((tab) => {
          const row = tab.querySelector('.ui-row')
          return { text: baselineOf(row), chip: baselineOf(tab.querySelector('.tab-chord')) }
        })
    `)
    expect(pairs.length).toBeGreaterThan(0)
    for (const p of pairs) expect(Math.abs(p.text - p.chip)).toBeLessThanOrEqual(TOLERANCE)
  })

  it('status bar: text, the button row and the line share a baseline and the group is centred in the bar', async () => {
    const m = await measure(`
      const bar = document.querySelector('footer.status')
      const lineEl = bar.querySelector('.status-line')
      const info = bar.querySelector('.status-info .info-item')
      const end = bar.querySelector('.status-end')
      const line = lineEl.getBoundingClientRect()
      const box = bar.getBoundingClientRect()
      const border = parseFloat(getComputedStyle(bar).borderTopWidth)
      return {
        text: baselineOf(info),
        end: baselineOf(end),
        line: baselineOf(lineEl),
        lineMid: (line.top + line.bottom) / 2,
        barMid: (box.top + border + box.bottom) / 2,
        buttons: end.querySelectorAll('.ui-btn').length,
      }
    `)
    expect(m.buttons).toBeGreaterThan(0)
    expect(Math.abs(m.text - m.line)).toBeLessThanOrEqual(TOLERANCE)
    expect(Math.abs(m.end - m.line)).toBeLessThanOrEqual(TOLERANCE)
    expect(Math.abs(m.lineMid - m.barMid)).toBeLessThanOrEqual(TOLERANCE)
  })

  it('title bar in Library: Import text and its key chip share the mode tab baseline', async () => {
    await browser.keys(['Meta', '1'])
    await $('.titlebar-import').waitForDisplayed({ timeout: 30_000 })
    const m = await measure(`
      const bar = document.querySelector('.titlebar')
      const row = bar.querySelector('.titlebar-import .ui-row')
      const chip = bar.querySelector('.titlebar-import-chord')
      return {
        tab: baselineOf(bar.querySelector('.mode-tab')),
        text: baselineOf(row),
        chip: chip ? baselineOf(chip) : null,
      }
    `)
    expect(m.chip).not.toBeNull()
    expect(Math.abs(m.tab - m.text)).toBeLessThanOrEqual(TOLERANCE)
    expect(Math.abs(m.tab - m.chip)).toBeLessThanOrEqual(TOLERANCE)
    await browser.keys(['Meta', '2'])
    await $('.layout-switch').waitForDisplayed({ timeout: 30_000 })
  })
})
