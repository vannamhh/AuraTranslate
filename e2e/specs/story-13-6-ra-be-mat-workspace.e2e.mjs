/**
 * Workspace, Lookup and the status bar on the 7px grid in a real WKWebView: toolbar rows are 28px
 * tall and 14px below the panel edge, boxes with a 1px border or a side rule stay on a multiple
 * of 7 and keep their text edge on 14 (DESIGN.md §Lưới 7px).
 *
 *     npm run test:e2e -- --spec e2e/specs/story-13-6-ra-be-mat-workspace.e2e.mjs
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { ensureFullLayoutTier } from '../support/layoutTier.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const SEED = '甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未'
const NEAR_SEED = '甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午申'
const TEXT = `${SEED}。${NEAR_SEED}。第三句子内容在这里。`

async function invoke(command, args) {
  return browser.execute(async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a), command, args)
}

function onSeven(value) {
  const rest = Math.abs(value) % 7
  return Math.min(rest, 7 - rest) <= 0.5
}

describe('Workspace surfaces on the 7px grid', () => {
  before(async function () {
    this.timeout(300_000)
    await openWorkspaceWithWork(`e2e-surface-7px-${Date.now()}`, TEXT)
    await waitForGridRows(3, { col: 'tgt', timeout: 60_000, what: 'Chapter just built' })
    await ensureFullLayoutTier()
  })

  for (const selector of ['.ai-panel-head', '.lookup-tabrow', '.tabs']) {
    it(`${selector} is a 28px toolbar row 14px below the panel edge`, async () => {
      const box = await browser.execute((sel) => {
        const el = document.querySelector(sel)
        if (el === null) return null
        const r = el.getBoundingClientRect()
        const panel = el.closest('.panel').getBoundingClientRect()
        return { height: r.height, top: r.top - panel.top }
      }, selector)
      expect(box).not.toBeNull()
      expect(Math.abs(box.height - 28)).toBeLessThanOrEqual(0.5)
      expect(Math.abs(box.top - 14)).toBeLessThanOrEqual(0.5)
    })
  }

  it('the TM strip is a multiple of 7 tall and its text sits 14px inside', async () => {
    const loaded = await invoke('read_open_chapter_segments', {})
    const [seed, near] = loaded.segments
    await invoke('save_segment_targets', {
      chapterId: loaded.chapter_id,
      edits: [{ id: seed.id, target_text: '已确认的译文' }],
    })
    await invoke('confirm_segment', { segmentId: seed.id })
    await browser.execute((id) => {
      document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`).focus()
    }, near.id)
    const strip = await $('.tm-fuzzy-strip')
    await strip.waitForExist({ timeout: 15_000 })
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('.tmf-title') !== null), {
      timeout: 15_000,
      timeoutMsg: 'the TM strip never rendered its title',
    })
    const m = await browser.execute(() => {
      const s = document.querySelector('.tm-fuzzy-strip').getBoundingClientRect()
      const t = document.querySelector('.tmf-title').getBoundingClientRect()
      return { height: s.height, left: t.left - s.left, top: t.top - s.top }
    })
    expect(onSeven(m.height)).toBe(true)
    expect(Math.abs(m.left - 14)).toBeLessThanOrEqual(0.5)
    expect(Math.abs(m.top - 14)).toBeLessThanOrEqual(0.5)
  })

  describe('Lookup with results', () => {
    before(async function () {
      const sourceCount = await browser.execute(async () => {
        try {
          const res = await window.__TAURI_INTERNALS__.invoke('list_dict_sources')
          return Array.isArray(res?.sources) ? res.sources.length : -1
        } catch {
          return -1
        }
      })
      if (sourceCount < 0) throw new Error('[BÀN ĐO HỎNG] `list_dict_sources` ném hoặc trả hình dạng lạ; mệnh đề chưa được đo.')
      if (sourceCount === 0) {
        console.warn('[BÀN ĐO KHÔNG DỰNG ĐƯỢC] 0 nguồn từ điển: không có khối Lookup để đo. Dựng dữ liệu rồi chạy lại.')
        this.skip()
      }
      const ran = await browser.execute(async () => {
        try {
          const mod = await import('/src/panels/lookupPanelState.ts')
          await mod.runLookup('山丹')
          return true
        } catch {
          return false
        }
      })
      if (!ran) throw new Error('[BÀN ĐO HỎNG] không gọi được `runLookup`; mệnh đề chưa được đo.')
      try {
        await (await $('.lookup-source')).waitForExist({ timeout: 15_000 })
      } catch {
        console.warn('[BÀN ĐO KHÔNG DỰNG ĐƯỢC] lượt tra không cho nhóm nguồn nào.')
        this.skip()
      }
    })

    it('the lookup head is 42px and a source head is 35px', async () => {
      const m = await browser.execute(() => ({
        head: document.querySelector('.lookup-head').getBoundingClientRect().height,
        source: document.querySelector('.lookup-source-head').getBoundingClientRect().height,
      }))
      expect(Math.abs(m.head - 42)).toBeLessThanOrEqual(0.5)
      expect(Math.abs(m.source - 35)).toBeLessThanOrEqual(0.5)
    })

    it('a source block with two or more entries is a multiple of 7 tall', async function () {
      const measure = () =>
        browser.execute(() =>
          [...document.querySelectorAll('.lookup-source')]
            .filter((el) => el.querySelectorAll('.lookup-entry').length >= 2)
            .map((el) => el.getBoundingClientRect().height),
        )
      let heights = await measure()
      for (const query of ['山', '行', '长', '了', '为', '得', '重']) {
        if (heights.length > 0) break
        await browser.execute(async (q) => {
          const mod = await import('/src/panels/lookupPanelState.ts')
          await mod.runLookup(q)
        }, query)
        await browser.pause(500)
        heights = await measure()
      }
      if (heights.length === 0) {
        console.warn('[BÀN ĐO KHÔNG DỰNG ĐƯỢC] không nguồn nào có từ 2 cụm trở lên.')
        this.skip()
      }
      for (const h of heights) expect(onSeven(h)).toBe(true)
    })
  })

  it('the status bar is 35px tall and every gap in it is a multiple of 7', async () => {
    const m = await browser.execute(() => {
      const bar = document.querySelector('.status')
      const gaps = [bar, ...bar.querySelectorAll('.status-line, .status-info, .status-end')].map((el) =>
        parseFloat(getComputedStyle(el).columnGap),
      )
      return { height: bar.getBoundingClientRect().height, gaps }
    })
    expect(Math.abs(m.height - 35)).toBeLessThanOrEqual(0.5)
    for (const g of m.gaps) expect(onSeven(g)).toBe(true)
  })
})
