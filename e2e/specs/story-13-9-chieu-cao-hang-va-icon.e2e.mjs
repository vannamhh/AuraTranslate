/**
 * One-line baseline rows declare a height on the 7px grid, and icon-only buttons put the icon bottom on
 * the baseline of their empty line box with the group centred in the button (DESIGN.md §Lưới 7px).
 *
 * Every declared row is measured twice: as shipped, and with `height: auto` (its natural height). A row
 * passes when the declared height is the step and the natural height is no taller than the step, so a
 * declaration never clips a child. Screens that need data a driver cannot create (an import preview, a
 * reviewer copy) are measured on a probe carrying the overlay's own scope attribute, which measures the
 * shipped CSS rather than the overlay's data, at the panel's real width in an 800px window. Every row is measured at the 800px minimum window except the Lookup rows, which need the full tier.
 *
 *     npm run test:e2e -- --spec e2e/specs/story-13-9-chieu-cao-hang-va-icon.e2e.mjs
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { ensureFullLayoutTier } from '../support/layoutTier.mjs'
import { aimWork, openAimedWork } from '../support/libraryGrid.mjs'
import { realClick, realClickAt } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TOLERANCE = 0.5
const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_NAME = `e2e-row-heights-${RUN_TAG}`
const SOURCE = '他拉小提琴。'
const SET_NAME = `e2e-row-heights-set-${RUN_TAG}`

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
  /** Where the svg bottom must sit when the button's empty line box is centred, against where it is. */
  function lineGroupOf(btn) {
    const pseudo = getComputedStyle(btn, '::before')
    const ref = document.createElement('div')
    ref.style.cssText =
      'position:absolute;visibility:hidden;left:0;top:0;white-space:nowrap;font-family:' + pseudo.fontFamily +
      ';font-size:' + pseudo.fontSize + ';line-height:' + pseudo.lineHeight
    ref.textContent = 'x'
    document.body.appendChild(ref)
    const ascent = baselineOf(ref) - ref.getBoundingClientRect().top
    ref.remove()
    const box = btn.getBoundingClientRect()
    const cs = getComputedStyle(btn)
    const inner = box.height - parseFloat(cs.borderTopWidth) - parseFloat(cs.borderBottomWidth)
    const lineTop = box.top + parseFloat(cs.borderTopWidth) + (inner - parseFloat(pseudo.lineHeight)) / 2
    return { expected: lineTop + ascent, actual: btn.querySelector('svg').getBoundingClientRect().bottom }
  }
`

async function measure(body) {
  return browser.execute(new Function(`${PROBE}\n${body}`))
}

/**
 * Declared and natural height of every laid-out match, whether a stylesheet rule matching it declares a
 * height, and how many children leave its border box. With `html`, the rows are measured on a probe built
 * under a fixed column of `width` px whose elements carry the scope attribute of `scopeClass`.
 */
function measureRows({ selector, content, html, scopeClass, width, step }) {
  let root = document
  let probe = null
  if (html !== undefined) {
    let attr = null
    const re = new RegExp(`\\.${scopeClass}\\[(data-v-[a-z0-9]+)\\]`)
    for (const sheet of document.styleSheets) {
      let rules
      try {
        rules = sheet.cssRules
      } catch {
        continue
      }
      for (const rule of rules) {
        const m = re.exec(rule.selectorText ?? '')
        if (attr === null && m !== null) attr = m[1]
      }
    }
    if (attr === null) return { error: `no scoped rule for .${scopeClass}` }
    probe = document.createElement('div')
    probe.style.cssText = `position:fixed;left:0;top:0;width:${width}px;display:flex;flex-direction:column`
    probe.innerHTML = html
    for (const el of probe.querySelectorAll('*')) el.setAttribute(attr, '')
    document.body.appendChild(probe)
    root = probe
  }
  const read = (el) => {
    const r = el.getBoundingClientRect()
    if (!content) return r.height
    const cs = getComputedStyle(el)
    return (
      r.height -
      parseFloat(cs.paddingTop) -
      parseFloat(cs.paddingBottom) -
      parseFloat(cs.borderTopWidth) -
      parseFloat(cs.borderBottomWidth)
    )
  }
  const matches = (el, sel) => {
    try {
      return el.matches(sel)
    } catch {
      return false
    }
  }
  const sized = []
  const collect = (rules) => {
    for (const rule of rules) {
      if (rule.cssRules !== undefined) collect(rule.cssRules)
      if (rule.selectorText !== undefined && [rule.style.height, rule.style.minHeight].includes(`${step}px`)) sized.push(rule.selectorText)
    }
  }
  for (const sheet of document.styleSheets) {
    try {
      collect(sheet.cssRules)
    } catch {
      continue
    }
  }
  try {
    return {
      rows: [...root.querySelectorAll(selector)]
        .filter((el) => el.getBoundingClientRect().height > 0)
        .map((el) => {
          const box = el.getBoundingClientRect()
          const stray = [...el.children].filter((c) => {
            const r = c.getBoundingClientRect()
            return r.height > 0 && (r.top < box.top - 0.5 || r.bottom > box.bottom + 0.5)
          }).length
          const declared = read(el)
          const had = el.style.height
          el.style.height = 'auto'
          const natural = read(el)
          el.style.height = had
          const last = el.lastElementChild
          let lastOffset = null
          if (last !== null) {
            const cs = getComputedStyle(el)
            const top = box.top + parseFloat(cs.borderTopWidth) + parseFloat(cs.paddingTop)
            const inner =
              box.height -
              parseFloat(cs.borderTopWidth) -
              parseFloat(cs.borderBottomWidth) -
              parseFloat(cs.paddingTop) -
              parseFloat(cs.paddingBottom)
            const r = last.getBoundingClientRect()
            lastOffset = r.top + r.height / 2 - (top + inner / 2)
          }
          return { declared, natural, stray, lastOffset, outer: box.height, hasRule: sized.some((sel) => matches(el, sel)) }
        }),
    }
  } finally {
    if (probe !== null) probe.remove()
  }
}

const NATURALS = []

function expectRows(rows, height, label, { centred = false, rule = true } = {}) {
  expect(rows.length).toBeGreaterThan(0)
  for (const row of rows) {
    if (rule) expect(row.hasRule).toBe(true)
    expect(Math.abs(row.declared - height)).toBeLessThanOrEqual(TOLERANCE)
    expect(row.natural).toBeLessThanOrEqual(height + TOLERANCE)
    expect(row.stray).toBe(0)
    if (centred) expect(Math.abs(row.lastOffset)).toBeLessThanOrEqual(TOLERANCE)
  }
  NATURALS.push(`${label} ${height}: ${[...new Set(rows.map((r) => r.natural))].join('/')}`)
}

async function expectRowsOn(selector, height, options = {}) {
  const out = await browser.execute(measureRows, { selector, content: options.content === true, step: height })
  expectRows(out.rows, height, selector, options)
  return out.rows
}

async function expectProbe(html, scopeClass, selector, height, width, options = {}) {
  const out = await browser.execute(measureRows, { selector, content: options.content === true, html, scopeClass, width, step: height })
  expect(out.error).toBeUndefined()
  expectRows(out.rows, height, `${selector}@${width}`, options)
  return out.rows
}

/** Content width of a panel at the 800px window: the panel is `min(max, 800 - 2 * 14 scrim padding)` minus its own inset. */
const panelWidth = (max, inset = 28) => Math.min(max ?? Infinity, 772) - inset

const TITLE = 'Tiêu đề của lớp phủ'
const HEAD = (p, actions = `<button type="button" class="${p}-close ui-btn ui-btn--quiet">Đóng</button>`) =>
  `<header class="${p}-head"><h2 class="${p}-title">${TITLE}</h2>${actions}</header>`
const BARE = '<button type="button" class="ui-btn ui-btn--quiet">Đóng</button>'

/** Overlay heads with a `read-title` title: one 35 tall row, the Close button centred in it. */
const READ_TITLE_HEADS = [
  { prefix: 'aip', max: null },
  { prefix: 'al', max: 959 },
  { prefix: 'attr', max: null },
  { prefix: 'cm', max: 721 },
  { prefix: 'cvi', max: 721, html: HEAD('cvi', BARE) },
  {
    prefix: 'cvm',
    max: 1099,
    html: HEAD(
      'cvm',
      '<div class="cvm-head-actions"><button type="button" class="ui-btn">Xuất</button><button type="button" class="ui-btn">Nhập</button><button type="button" class="ui-btn ui-btn--quiet">Đóng</button></div>',
    ),
  },
  { prefix: 'gh', max: 721 },
  { prefix: 'gi', max: 721 },
  { prefix: 'gm', max: 1099 },
  { prefix: 'gq', max: 1120 },
  { prefix: 'li', max: 721 },
  { prefix: 'pi', max: 721 },
  { prefix: 'pl', max: 959, html: HEAD('pl', '<form><button type="submit" class="pl-close ui-btn ui-btn--quiet">Đóng</button></form>') },
  { prefix: 'ri', max: 721 },
  { prefix: 'hist', max: 1099 },
  { prefix: 'ti', max: 721 },
  { prefix: 'tm', max: 882 },
]

async function openFromTools(item) {
  const trigger = await $('.tools-trigger')
  await trigger.waitForDisplayed({ timeout: 30_000 })
  await realClick(trigger)
  const entry = await $(`[data-tools-item="${item}"]`)
  await entry.waitForDisplayed({ timeout: 10_000 })
  await realClick(entry)
}

/** Viewport centre of one source character of the first source cell, found by its code-point index. */
async function charCentre(index) {
  return browser.execute((cp) => {
    const cell = document.querySelector('[data-col="src"]')
    const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
    let seen = 0
    for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
      if (node.parentElement.closest('rt') !== null) continue
      const chars = [...node.data]
      if (cp < seen + chars.length) {
        const from = chars.slice(0, cp - seen).join('').length
        const range = document.createRange()
        range.setStart(node, from)
        range.setEnd(node, from + chars[cp - seen].length)
        const rect = range.getBoundingClientRect()
        return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 }
      }
      seen += chars.length
    }
    throw new Error(`no source character at ${cp}`)
  }, index)
}

async function invoke(command, args) {
  return browser.execute(async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a), command, args)
}

/** `setWindowSize` units differ by driver and dpr, so accept whichever factor lands the logical width. */
async function resizeLogical(width, height) {
  const dpr = await browser.execute(() => window.devicePixelRatio)
  let inner = 0
  for (const factor of [dpr, 1]) {
    await browser.setWindowSize(Math.round(width * factor), Math.round(height * factor))
    await browser.pause(400)
    inner = await browser.execute(() => window.innerWidth)
    if (Math.abs(inner - width) <= 12) return
  }
  throw new Error(`window could not reach logical width ${width} (last innerWidth ${inner}, dpr ${dpr})`)
}

const tierOf = () =>
  browser.execute(() => document.querySelector('[data-layout-tier]')?.getAttribute('data-layout-tier') ?? null)

/** The smallest window that keeps the full tier (1100 x 820 work area), else the largest the screen allows. */
async function resizeToSmallestFull() {
  await resizeLogical(1120, 900)
  if ((await tierOf()) !== 'full') await ensureFullLayoutTier()
}

const MINIMUM = [800, 700]

describe('Row heights and icon-only buttons in a real webview', () => {
  before(async function () {
    this.timeout(300_000)
    await openWorkspaceWithWork(WORK_NAME, SOURCE)
    await waitForGridRows(1, { col: 'tgt', timeout: 60_000, what: 'Chapter just built' })
  })

  after(() => {
    console.log(`[13-9] natural heights\n${NATURALS.join('\n')}`)
  })

  describe('Icon-only buttons at 800px', () => {
    before(async () => {
      await resizeLogical(...MINIMUM)
    })

    it('status bar: the icon button stands level with the text button, the icon bottom is on the line baseline and its line box is centred in the button', async () => {
      const m = await measure(`
        const bar = document.querySelector('footer.status')
        const icon = bar.querySelector('.ui-icon-btn')
        const text = bar.querySelector('.ui-btn')
        return {
          iconTop: icon.getBoundingClientRect().top,
          textTop: text.getBoundingClientRect().top,
          iconHeight: icon.getBoundingClientRect().height,
          svgBottom: icon.querySelector('svg').getBoundingClientRect().bottom,
          lineBaseline: baselineOf(bar.querySelector('.status-line')),
          group: lineGroupOf(icon),
        }
      `)
      expect(Math.abs(m.iconTop - m.textTop)).toBeLessThanOrEqual(TOLERANCE)
      expect(Math.abs(m.iconHeight - 28)).toBeLessThanOrEqual(TOLERANCE)
      expect(Math.abs(m.svgBottom - m.lineBaseline)).toBeLessThanOrEqual(TOLERANCE)
      expect(Math.abs(m.group.actual - m.group.expected)).toBeLessThanOrEqual(TOLERANCE)
    })

    it('layout switch: the icon bottom is on the baseline of the text button beside it and its line box is centred in the item', async () => {
      const m = await measure(`
        const item = document.querySelector('.titlebar .layout-switch .ui-seg__item')
        const btn = document.querySelector('.titlebar .tools-trigger')
        const box = item.getBoundingClientRect()
        const btnBox = btn.getBoundingClientRect()
        return {
          iconBaselineFromTop: item.querySelector('svg').getBoundingClientRect().bottom - box.top,
          textBaselineFromTop: baselineOf(btn.querySelector('.ui-row')) - btnBox.top,
          itemHeight: box.height,
          btnHeight: btnBox.height,
          group: lineGroupOf(item),
        }
      `)
      expect(Math.abs(m.itemHeight - m.btnHeight)).toBeLessThanOrEqual(TOLERANCE)
      expect(Math.abs(m.iconBaselineFromTop - m.textBaselineFromTop)).toBeLessThanOrEqual(TOLERANCE)
      expect(Math.abs(m.group.actual - m.group.expected)).toBeLessThanOrEqual(TOLERANCE)
    })
  })

  describe('Lookup rows at the smallest full window', () => {
    before(async () => {
      await resizeToSmallestFull()
    })

    it('.lookup-tools is 28 tall', async () => {
      await $('.lookup-tools').waitForDisplayed({ timeout: 30_000 })
      await expectRowsOn('.lookup-tools', 28)
    })

    it('.lookup-collapse: the chevron sits on the baseline of its source line, its line box is centred and it keeps its centre when the source collapses and expands', async function () {
      const at = await charCentre(SOURCE.indexOf('提'))
      await realClickAt(at.x, at.y)
      const present = await $('.lookup-collapse')
        .waitForExist({ timeout: 15_000 })
        .then(
          () => true,
          () => false,
        )
      if (!present) {
        console.warn('[BÀN ĐO KHÔNG DỰNG ĐƯỢC] Panel Lookup không có nguồn từ điển nào để thu: dựng từ điển rồi chạy lại.')
        this.skip()
      }
      const read = () =>
        measure(`
          const btn = document.querySelector('.lookup-collapse')
          const svg = btn.querySelector('svg').getBoundingClientRect()
          const closed = btn.classList.contains('lookup-collapse--closed')
          return {
            closed,
            cx: svg.left + svg.width / 2,
            cy: svg.top + svg.height / 2,
            bottom: svg.bottom,
            textBaseline: baselineOf(btn.closest('.lookup-source-line')),
            group: closed ? null : lineGroupOf(btn),
          }
        `)
      const first = await read()
      await realClick(await $('.lookup-collapse'))
      await browser.waitUntil(async () => (await read()).closed !== first.closed, { timeout: 10_000 })
      const second = await read()
      expect(Math.abs(first.cx - second.cx)).toBeLessThanOrEqual(TOLERANCE)
      expect(Math.abs(first.cy - second.cy)).toBeLessThanOrEqual(TOLERANCE)
      const open = first.closed ? second : first
      expect(Math.abs(open.bottom - open.textBaseline)).toBeLessThanOrEqual(TOLERANCE)
      expect(Math.abs(open.group.actual - open.group.expected)).toBeLessThanOrEqual(TOLERANCE)
      await realClick(await $('.lookup-collapse'))
      await browser.waitUntil(async () => (await read()).closed === first.closed, { timeout: 10_000 })
    })

    it('opens the Prompt library for the later cases', async () => {
      await invoke('prompt_set_create', { tier: 'global', kind: 'translate', name: SET_NAME, body: 'Dich {{glossary_terms}}.' })
      await realClick(await $('[data-ai-prompt-inspector-open]'))
      const opener = await $('[data-prompt-library-open]')
      await opener.waitForDisplayed({ timeout: 15_000 })
      await realClick(opener)
      await $('.pl-head').waitForDisplayed({ timeout: 15_000 })
    })
  })

  describe('Rows at the 800px minimum', () => {
    before(async () => {
      await resizeLogical(...MINIMUM)
    })

    describe('Prompt library', () => {
      it('.pl-head (read-title) is 35 tall, holds its children and centres the Close button', async () => {
        await expectRowsOn('.pl-head', 35, { centred: true })
        const m = await browser.execute(() => {
          const head = document.querySelector('.pl-head').getBoundingClientRect()
          const btn = document.querySelector('.pl-head .pl-close').getBoundingClientRect()
          return { offset: btn.top - head.top - (head.height - btn.height) / 2, btn: btn.height }
        })
        expect(Math.abs(m.btn - 28)).toBeLessThanOrEqual(TOLERANCE)
        expect(Math.abs(m.offset)).toBeLessThanOrEqual(TOLERANCE)
      })

      it('.pl-seg is 28 tall', async () => {
        await $('.pl-seg').waitForDisplayed({ timeout: 15_000 })
        await expectRowsOn('.pl-seg', 28)
      })

      it('.pl-rename-form .pl-field is 35 tall', async () => {
        await realClick(await $('.pl-panel').$(`button*=${SET_NAME}`))
        await $('.pl-rename-form .pl-field').waitForDisplayed({ timeout: 15_000 })
        await expectRowsOn('.pl-rename-form .pl-field', 35)
        await browser.keys(['Escape'])
        await browser.keys(['Escape'])
      })
    })

    describe('Library and Work page', () => {
      before(async () => {
        await browser.keys(['Meta', '1'])
        await $('.titlebar-import').waitForDisplayed({ timeout: 30_000 })
      })

      it('.search-field is 28 tall', async () => {
        await $('.search-field').waitForDisplayed({ timeout: 30_000 })
        await expectRowsOn('.search-field', 28)
      })

      it('.wp-sort is 28 tall', async () => {
        await aimWork(WORK_NAME)
        await openAimedWork()
        await $('.wp-sort').waitForDisplayed({ timeout: 20_000 })
        await expectRowsOn('.wp-sort', 28)
      })

      after(async () => {
        await browser.keys(['Escape'])
        await browser.keys(['Meta', '2'])
        await $('.layout-switch').waitForDisplayed({ timeout: 30_000 })
      })
    })

    describe('Settings', () => {
      it('.set-head holds the title and the Close button inside one 28 tall content box, 56 tall with the panel border', async () => {
        await openFromTools('settings.open')
        await $('.set-panel').waitForDisplayed({ timeout: 10_000 })
        const rows = await expectRowsOn('.set-head', 28, { content: true, centred: true })
        expect((rows[0].outer + 1) % 7).toBeLessThanOrEqual(TOLERANCE)
      })

      it('.sc-search is 35 tall', async () => {
        await realClick(await $('[data-settings-section="shortcuts"]'))
        await $('.sc-search').waitForDisplayed({ timeout: 10_000 })
        await expectRowsOn('.sc-search', 35)
        await browser.keys(['Escape'])
      })
    })

    describe('Overlay heads on a probe at their panel width', () => {
      for (const head of READ_TITLE_HEADS) {
        it(`.${head.prefix}-head is 35 tall, holds its children and centres its last child`, async () => {
          const rows = await expectProbe(
            head.html ?? HEAD(head.prefix),
            `${head.prefix}-head`,
            `.${head.prefix}-head`,
            35,
            panelWidth(head.max),
            { centred: true },
          )
          expect(rows.length).toBe(1)
        })
      }

      it('.ex-head is 35 tall in the content box and 63 tall with the panel border', async () => {
        const rows = await expectProbe(HEAD('ex'), 'ex-head', '.ex-head', 35, panelWidth(null, 2), { content: true, centred: true })
        expect((rows[0].outer + 1) % 7).toBeLessThanOrEqual(TOLERANCE)
      })

      it('.ld-head is 35 tall in the content box', async () => {
        await expectProbe(HEAD('ld'), 'ld-head', '.ld-head', 35, panelWidth(420, 0), { content: true, centred: true })
      })

      it('.pfmem-head is 28 tall', async () => {
        await expectProbe(HEAD('pfmem'), 'pfmem-head', '.pfmem-head', 28, panelWidth(357), { centred: true })
      })

      it('.pfscope-head is 28 tall', async () => {
        await expectProbe(HEAD('pfscope'), 'pfscope-head', '.pfscope-head', 28, panelWidth(322), { centred: true })
      })

      it('.diff-status keeps one 35 tall line when empty', async () => {
        await expectProbe('<p class="diff-status" role="status"></p>', 'diff-status', '.diff-status', 35, 772)
      })

      it('.review-bar stays 35 tall when the status message is empty', async () => {
        const icon = '<button type="button" class="ui-icon-btn"><svg width="14" height="14"></svg></button>'
        await expectProbe(
          `<div class="review-bar"><p class="diff-stats">1 / 3</p>${icon}${icon}<p class="diff-status" role="status"></p></div>`,
          'review-bar',
          '.review-bar',
          35,
          772,
          { rule: false },
        )
      })
    })
  })
})
