/**
 * The TM strip grows as a grid track right under the anchor row, and the
 * frame cost of growing and shrinking it on a 2,000-sentence Chapter is measured (no pass
 * threshold; the numbers go to the plan).
 *
 *     npm run test:e2e -- --spec e2e/specs/story-12-9-strip-row-track.e2e.mjs
 */
import { waitForGridRows } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const SENTENCE_COUNT = 2000
const MATCH_ORDINAL = 1500
const ROUNDS = 7
const SEED = '甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未'
const NEAR_SEED = '甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午申'

function sentence(ordinal) {
  if (ordinal === 1) return `${SEED}。`
  if (ordinal === MATCH_ORDINAL) return `${NEAR_SEED}。`
  return `第${ordinal}句子内容在这里。`
}

const TEXT = Array.from({ length: SENTENCE_COUNT }, (_, i) => sentence(i + 1)).join('')

async function invoke(command, args) {
  return browser.execute(
    async (c, a) => window.__TAURI_INTERNALS__.invoke(c, a),
    command,
    args,
  )
}

function median(values) {
  const sorted = [...values].sort((a, b) => a - b)
  return sorted[Math.floor(sorted.length / 2)]
}

/**
 * Focuses `[data-col="tgt"][data-segment-id=id]` and records requestAnimationFrame gaps until
 * `.tm-fuzzy-strip` presence equals `expectStrip` for `settleFrames` frames. Returns the
 * gap of the first frame that shows the expected state, the maximum gap of the window, and
 * the number of frames sampled.
 */
async function focusAndSample(segmentId, expectStrip) {
  return browser.execute(
    (id, expect, settleFrames) =>
      new Promise((resolve) => {
        const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`)
        const gaps = []
        let last = performance.now()
        let firstExpected = null
        let settled = 0
        const started = last
        const tick = (now) => {
          const gap = now - last
          last = now
          gaps.push(gap)
          const present = document.querySelector('.tm-fuzzy-strip') !== null
          if (present === expect) {
            if (firstExpected === null) firstExpected = gap
            settled += 1
          } else {
            settled = 0
            firstExpected = null
          }
          if (settled >= settleFrames) {
            resolve({ firstExpected, max: Math.max(...gaps), frames: gaps.length })
            return
          }
          if (now - started > 15000) {
            resolve({ firstExpected: -1, max: Math.max(...gaps), frames: gaps.length })
            return
          }
          requestAnimationFrame(tick)
        }
        requestAnimationFrame(tick)
        cell.focus()
      }),
    segmentId,
    expectStrip,
    5,
  )
}

describe('Story 12-9 -- strip as a row track', () => {
  it('sits right under the anchor row, and growing/shrinking it is measured', async () => {
    await openWorkspaceWithWork('Story 12-9 -- strip track', TEXT)
    await waitForGridRows(SENTENCE_COUNT)

    const loaded = await invoke('read_open_chapter_segments', {})
    const segments = loaded.segments
    await expect(segments.length).toBe(SENTENCE_COUNT)
    const seedId = segments[0].id
    const matchId = segments[MATCH_ORDINAL - 1].id
    const otherId = segments[MATCH_ORDINAL - 2].id
    const otherId2 = segments[MATCH_ORDINAL - 3].id
    const nextId = segments[MATCH_ORDINAL].id

    await invoke('save_segment_targets', {
      chapterId: loaded.chapter_id,
      edits: [{ id: seedId, target_text: '已确认的译文' }],
    })
    await invoke('confirm_segment', { segmentId: seedId })

    await browser.execute((id) => {
      document
        .querySelector(`[data-col="tgt"][data-segment-id="${id}"]`)
        .scrollIntoView({ block: 'center' })
    }, matchId)

    const grow = await focusAndSample(matchId, true)
    await expect(grow.firstExpected).toBeGreaterThanOrEqual(0)

    const geometry = await browser.execute((id, followingId) => {
      const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`)
      const strip = document.querySelector('.tm-fuzzy-strip')
      const next = document.querySelector(`[data-col="tgt"][data-segment-id="${followingId}"]`)
      const c = cell.getBoundingClientRect()
      const s = strip.getBoundingClientRect()
      const n = next.getBoundingClientRect()
      return {
        gapBelowCell: s.top - c.bottom,
        gapAboveNext: n.top - s.bottom,
        left: s.left - c.left,
      }
    }, matchId, nextId)
    await expect(Math.abs(geometry.gapBelowCell)).toBeLessThanOrEqual(2)
    await expect(Math.abs(geometry.gapAboveNext)).toBeLessThanOrEqual(2)
    await expect(Math.abs(geometry.left)).toBeLessThanOrEqual(2)

    const scrollBeforeAfter = await browser.execute(async (id, otherCellId) => {
      document.querySelector(`[data-col="tgt"][data-segment-id="${otherCellId}"]`).focus()
      await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))
      const scroller = document.querySelector('.grid-scroll')
      const target = document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`)
      target.scrollIntoView({ block: 'end' })
      await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))
      const before = scroller.scrollTop
      target.focus({ preventScroll: true })
      for (let i = 0; i < 30; i += 1) await new Promise((r) => requestAnimationFrame(r))
      return {
        before,
        after: scroller.scrollTop,
        stripShown: document.querySelector('.tm-fuzzy-strip') !== null,
      }
    }, matchId, otherId2)
    await expect(scrollBeforeAfter.stripShown).toBe(true)
    await expect(scrollBeforeAfter.after).toBe(scrollBeforeAfter.before)

    const grows = []
    const shrinks = []
    const growMax = []
    const shrinkMax = []
    for (let round = 0; round < ROUNDS; round += 1) {
      const down = await focusAndSample(otherId, false)
      shrinks.push(down.firstExpected)
      shrinkMax.push(down.max)
      const up = await focusAndSample(matchId, true)
      grows.push(up.firstExpected)
      growMax.push(up.max)
    }

    await focusAndSample(otherId, false)
    const baseline = []
    for (let round = 0; round < ROUNDS; round += 1) {
      const move = await focusAndSample(round % 2 === 0 ? otherId2 : otherId, false)
      baseline.push(move.max)
    }

    const report =
      `[12-9 frame] ${SENTENCE_COUNT} sentences, ${ROUNDS} rounds, WKWebView debug build\n` +
      `  grow   first-frame gap median ${median(grows).toFixed(1)} ms, max ${Math.max(...grows).toFixed(1)} ms; window max median ${median(growMax).toFixed(1)} ms, max ${Math.max(...growMax).toFixed(1)} ms\n` +
      `  shrink first-frame gap median ${median(shrinks).toFixed(1)} ms, max ${Math.max(...shrinks).toFixed(1)} ms; window max median ${median(shrinkMax).toFixed(1)} ms, max ${Math.max(...shrinkMax).toFixed(1)} ms\n` +
      `  baseline (caret between two rows without a strip) window max median ${median(baseline).toFixed(1)} ms, max ${Math.max(...baseline).toFixed(1)} ms`
    console.log(report)
  })
})
