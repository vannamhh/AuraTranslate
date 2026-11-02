/**
 * Library card, Work page and Reading mode on the 7px grid in a real WKWebView: bordered boxes
 * stay a multiple of 7 with their text edge on a multiple of 7, the Chapter list virtualises on
 * the same 42px the CSS draws, and reading paragraphs are spaced by one drawn line box
 * (DESIGN.md §Lưới 7px).
 *
 *     npm run test:e2e -- --spec e2e/specs/story-13-7-ra-be-mat-library-doc.e2e.mjs
 */
import { aimWork, openAimedWork, reloadWorks } from '../support/libraryGrid.mjs'
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_NAME = `e2e-surface-7px-${RUN_TAG}`
const SENTENCES = 27
const SOURCE_TEXT = Array.from({ length: SENTENCES }, (_, i) => `Cau ${i + 1}。`).join('')
const WORK_PAGE = '[data-library-work-page]'
const READING_TEXT = '句一。句二。\n\n句三。句四。\n\n句五。句六。'

function onSeven(value) {
  const rest = Math.abs(value) % 7
  return Math.min(rest, 7 - rest) <= 0.5
}

async function ipcValue(command, args = {}) {
  const out = await browser.execute(
    async (name, payload) => {
      try {
        return { ok: true, value: await window.__TAURI_INTERNALS__.invoke(name, payload) }
      } catch (err) {
        return { ok: false, text: String(err?.code ?? err) }
      }
    },
    command,
    args,
  )
  if (!out.ok) throw new Error(`fixture: ${command} failed: ${out.text}`)
  return out.value
}

async function seedWork() {
  await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
    timeout: 30_000,
    interval: 250,
  })
  await ipcValue('create_work_from_text', { name: WORK_NAME, sourceLang: 'zh', genre: 'general', text: SOURCE_TEXT })
  for (let split = 0; split < SENTENCES - 1; split += 1) {
    const chapters = await ipcValue('list_chapters')
    await ipcValue('open_chapter', { chapterId: chapters[chapters.length - 1].chapter_id })
    const loaded = await ipcValue('read_open_chapter_segments')
    await ipcValue('split_chapter_at_segment', { segmentId: loaded.segments[1].id })
  }
  for (const chapter of await ipcValue('list_chapters')) {
    await ipcValue('set_chapter_status', { chapterId: chapter.chapter_id, status: 'done' })
  }
}

describe('Library, Work page and Reading mode on the 7px grid', () => {
  describe('Library card and Work page', () => {
    before(async function () {
      this.timeout(300_000)
      await seedWork()
      await reloadWorks()
    })

    it('a Library card is a multiple of 7 tall, its text edge sits on a multiple of 7 and meta-to-progress is 14', async () => {
      const m = await browser.execute((name) => {
        const cell = [...document.querySelectorAll('.works-grid .work-cell')].find(
          (c) => (c.querySelector('.work-name')?.textContent ?? '').trim() === name,
        )
        if (cell === undefined) return null
        const box = cell.getBoundingClientRect()
        const meta = cell.querySelector('.work-meta').getBoundingClientRect()
        const track = cell.querySelector('.work-progress-track')?.getBoundingClientRect() ?? null
        return {
          height: box.height,
          top: meta.top - box.top,
          left: meta.left - box.left,
          metaToTrack: track === null ? null : track.top - meta.bottom,
        }
      }, WORK_NAME)
      expect(m).not.toBeNull()
      expect(onSeven(m.height)).toBe(true)
      expect(onSeven(m.top)).toBe(true)
      expect(onSeven(m.left)).toBe(true)
      expect(m.metaToTrack).not.toBeNull()
      expect(Math.abs(m.metaToTrack - 14)).toBeLessThanOrEqual(0.5)
    })

    it('a Work page row is 42px, its text edge is 14px inside the rule and the list scrolls 42px per item', async () => {
      await aimWork(WORK_NAME)
      await openAimedWork()
      await $(WORK_PAGE).waitForExist({ timeout: 20_000 })
      await browser.waitUntil(
        async () => browser.execute(() => document.querySelectorAll('[data-library-chapter-row]').length >= 5),
        { timeout: 20_000, timeoutMsg: 'the Work page did not list the Chapters' },
      )
      const m = await browser.execute(() => {
        const list = document.querySelector('[data-library-chapters-list]')
        const row = document.querySelector('[data-library-chapter-row]')
        const box = row.getBoundingClientRect()
        const first = row.firstElementChild.getBoundingClientRect()
        return {
          itemHeight: document.querySelector('.wp-item').getBoundingClientRect().height,
          rowEdge: first.left - box.left,
          scrollHeight: list.scrollHeight,
          clientHeight: list.clientHeight,
        }
      })
      expect(Math.abs(m.itemHeight - 42)).toBeLessThanOrEqual(0.5)
      expect(Math.abs(m.rowEdge - 14)).toBeLessThanOrEqual(0.5)
      const items = Math.round(m.scrollHeight / 42)
      expect(items).toBeGreaterThanOrEqual(SENTENCES)
      expect(Math.abs(m.scrollHeight - items * 42)).toBeLessThanOrEqual(1)
      expect(m.clientHeight).toBeLessThan(m.scrollHeight)
    })
  })

  describe('Reading mode', () => {
    before(async function () {
      this.timeout(300_000)
      await openWorkspaceWithWork(`e2e-reading-7px-${RUN_TAG}`, READING_TEXT)
      const loaded = await ipcValue('read_open_chapter_segments')
      await ipcValue('save_segment_targets', {
        chapterId: loaded.chapter_id,
        edits: loaded.segments.map((s) => ({ id: s.id, target_text: `Ban dich ${s.id}.` })),
      })
      await ipcValue('set_chapter_status', { chapterId: loaded.chapter_id, status: 'done' })
      await browser.keys(['Meta', '3'])
      await browser.waitUntil(async () => browser.execute(() => document.querySelector('.column .paragraph') !== null), {
        timeout: 15_000,
        timeoutMsg: 'the reading page never drew a paragraph',
      })
    })

    const measure = () =>
      browser.execute(() => {
        const col = document.querySelector('.column')
        const probe = document.createElement('div')
        probe.style.cssText = 'position:absolute;visibility:hidden;line-height:inherit;font-size:inherit'
        probe.textContent = 'x'
        col.appendChild(probe)
        const lineBox = probe.getBoundingClientRect().height
        probe.remove()
        const title = col.querySelector('.chapter-title').getBoundingClientRect()
        const head = col.querySelector('.chapter-head').getBoundingClientRect()
        const paragraphs = [...col.querySelectorAll('.paragraph')].map((p) => p.getBoundingClientRect())
        return {
          lineBox,
          supportsLh: CSS.supports('margin-bottom', '1lh'),
          headToTitle: title.top - head.bottom,
          titleToFirst: paragraphs[0].top - title.bottom,
          paragraphGap: paragraphs.length > 1 ? paragraphs[1].top - paragraphs[0].bottom : null,
        }
      })

    for (const [key, name] of [['1', 'airy'], ['2', 'balanced'], ['3', 'dense']]) {
      it(`${name}: the line box is a multiple of 7 and one line separates title, paragraphs`, async () => {
        await browser.keys([key])
        const m = await measure()
        expect(m.supportsLh).toBe(true)
        expect(onSeven(m.lineBox)).toBe(true)
        expect(m.paragraphGap).not.toBeNull()
        expect(Math.abs(m.paragraphGap - m.lineBox)).toBeLessThanOrEqual(0.5)
        expect(Math.abs(m.headToTitle - 7)).toBeLessThanOrEqual(0.5)
        expect(Math.abs(m.titleToFirst - m.lineBox)).toBeLessThanOrEqual(0.5)
      })
    }

    it('at 21px the slider ratios 2.34, 2.37 and 2.4 draw a 56px line box', async () => {
      await browser.keys(['2'])
      await realClick(await $('[data-settings-open]'))
      await realClick(await $('[data-tools-item="settings.open"]'))
      await realClick(await $('[data-settings-section="reading"]'))
      await $('[data-settings-reading-font-size]').waitForExist({ timeout: 10_000 })
      const set = (selector, value) =>
        browser.execute(
          (sel, v) => {
            const input = document.querySelector(sel)
            input.value = String(v)
            input.dispatchEvent(new Event('input', { bubbles: true }))
          },
          selector,
          value,
        )
      await set('[data-settings-reading-font-size]', 21)
      for (const ratio of [2.34, 2.37, 2.4]) {
        await set('[data-settings-reading-line-height]', ratio)
        await browser.waitUntil(
          async () =>
            browser.execute(
              (r) =>
                getComputedStyle(document.querySelector('.column')).fontSize === '21px' &&
                Math.abs(Number(document.querySelector('[data-settings-reading-line-height]').value) - r) <= 0.005,
              ratio,
            ),
          { timeout: 5_000, timeoutMsg: `the tuner did not apply 21px and ratio ${ratio}` },
        )
        const m = await measure()
        expect(Math.abs(m.lineBox - 56)).toBeLessThanOrEqual(0.01)
      }
      await browser.keys(['Escape'])
    })
  })
})
