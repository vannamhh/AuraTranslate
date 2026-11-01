/**
 * Every radio and checkbox box sits on the centre of its first text line in a real WKWebView,
 * whatever alignment the surface's own label class asks for, and a wrapped label keeps the box
 * on the first line (DESIGN.md §Điều khiển).
 *
 *     npm run test:e2e -- --spec e2e/specs/check-box-on-text-line.e2e.mjs
 */
import { aimWork, openAimedWork, reloadWorks } from '../support/libraryGrid.mjs'
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TOLERANCE = 0.5
const WORK_NAME = `e2e-check-line-${Date.now()}`
const ROW = '[data-library-chapters-list] [data-library-chapter-row]'

/** Centre of the box minus the centre of the first glyph, for each visible `ui-check` with text. */
const OFFSETS = `
  function offsets(root) {
    return [...root.querySelectorAll('label.ui-check')].flatMap((label) => {
      const input = label.querySelector('input')
      if (input === null || label.getClientRects().length === 0) return []
      const walker = document.createTreeWalker(label, NodeFilter.SHOW_TEXT)
      let node = walker.nextNode()
      while (node !== null && node.textContent.trim() === '') node = walker.nextNode()
      if (node === null) return []
      const start = node.textContent.search(/\\S/)
      const range = document.createRange()
      range.setStart(node, start)
      range.setEnd(node, start + 1)
      const glyph = range.getClientRects()[0]
      const box = input.getBoundingClientRect()
      return [{
        label: label.className.replace(/data-v-\\S+/g, '').trim(),
        offset: (box.top + box.bottom) / 2 - (glyph.top + glyph.bottom) / 2,
      }]
    })
  }
`

async function measure(selector) {
  return browser.execute(new Function(`${OFFSETS}\nreturn offsets(document.querySelector(${JSON.stringify(selector)}))`))
}

function expectCentred(rows) {
  expect(rows.length).toBeGreaterThan(0)
  expect(rows.filter((row) => Math.abs(row.offset) > TOLERANCE)).toEqual([])
}

async function openFromTools(item, panel) {
  const trigger = await $('.tools-trigger')
  await trigger.waitForDisplayed({ timeout: 30_000 })
  await realClick(trigger)
  const entry = await $(`[data-tools-item="${item}"]`)
  await entry.waitForDisplayed({ timeout: 10_000 })
  await realClick(entry)
  await $(panel).waitForDisplayed({ timeout: 10_000 })
}

async function closeWithEscape(panel) {
  await browser.keys(['Escape'])
  await browser.waitUntil(async () => browser.execute((sel) => document.querySelector(sel) === null, panel), {
    timeout: 10_000,
    timeoutMsg: `${panel} stayed open after Escape`,
  })
}

describe('Radio and checkbox boxes on their text line', () => {
  before(async function () {
    this.timeout(300_000)
    await openWorkspaceWithWork(WORK_NAME, '甲。\n乙。')
  })

  it('Glossary quick add: category and tier radios', async () => {
    await browser.keys(['Meta', 'Alt', 'g'])
    await $('.glossary-quick-add').waitForDisplayed({ timeout: 10_000 })
    const rows = await measure('.glossary-quick-add')
    expect(rows.length).toBeGreaterThanOrEqual(6)
    expectCentred(rows)
  })

  it('Glossary quick add: a wrapped label keeps the box on its first line', async () => {
    const offset = await browser.execute(new Function(`${OFFSETS}
      const label = document.querySelector('.glossary-quick-add .gqa-radio')
      label.style.width = '42px'
      const wrapped = label.getBoundingClientRect().height
      const [row] = offsets(label.parentElement)
      label.style.width = ''
      return { wrapped, offset: row.offset }
    `))
    expect(offset.wrapped).toBeGreaterThan(28)
    expect(Math.abs(offset.offset)).toBeLessThanOrEqual(TOLERANCE)
    await browser.keys(['Escape'])
    await $('.glossary-quick-add').waitForExist({ timeout: 10_000, reverse: true })
  })

  it('Glossary manager: the sort checkbox and every option of a select', async () => {
    await openFromTools('glossary.manage.open', '.gm-panel')
    expectCentred(await measure('.gm-panel'))
    const select = await $('.gm-panel .gm-field-origin summary')
    await realClick(select)
    await $('.gm-panel .gm-field-origin .select-item').waitForDisplayed({ timeout: 10_000 })
    const options = await measure('.gm-panel .gm-field-origin')
    expect(options.length).toBeGreaterThan(1)
    expectCentred(options)
    await browser.keys(['Escape'])
    await closeWithEscape('.gm-panel')
  })

  it('TM manager: filters and the export options', async () => {
    await openFromTools('tm.manage.open', '.tm-panel')
    expectCentred(await measure('.tm-panel'))
    await closeWithEscape('.tm-panel')
  })

  it('Export: scope, format and content options', async () => {
    await browser.keys(['Meta', '1'])
    await $('.titlebar-import').waitForDisplayed({ timeout: 30_000 })
    await reloadWorks()
    await aimWork(WORK_NAME)
    await openAimedWork()
    await $(ROW).waitForExist({ timeout: 20_000 })
    await realClick(await $(ROW))
    await realClick(await $(`${ROW} [data-library-row-edit]`))
    await $('[data-export-open]:not([disabled])').waitForExist({ timeout: 10_000 })
    await realClick(await $('[data-export-open]'))
    await $('.ex-panel').waitForDisplayed({ timeout: 10_000 })
    const rows = await measure('.ex-panel')
    expect(rows.length).toBeGreaterThanOrEqual(8)
    expectCentred(rows)
  })
})
