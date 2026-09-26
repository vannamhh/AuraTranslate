/**
 * `gridPanelImages.test.ts` (`happy-dom`) only proves the STRUCTURE of the five `subgrid`
 * columns (`GridPanel.vue`) — `tests/AGENTS.md` puts real-engine geometry out of vitest's
 * scope. This spec measures the claim itself: a real image, in a real WKWebView, still lets
 * `subgrid` keep all five columns' rows aligned when one row grows taller than the rest.
 */
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { unlinkSync, writeFileSync } from 'node:fs'

import { waitForGridRows } from '../support/gridWait.mjs'
import { resetPanelState } from '../support/panelReset.mjs'
import { buildDocxWithImageAfterFirstParagraph } from '../support/miniDocx.mjs'

const TOLERANCE_PX = 1

function writeFixtureDocx() {
  const path = join(tmpdir(), `aura-e2e-grid-image-${Date.now()}-${Math.random().toString(36).slice(2)}.docx`)
  writeFileSync(path, buildDocxWithImageAfterFirstParagraph())
  return path
}

async function openWorkFromDocx(name, path) {
  await browser.waitUntil(
    async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined),
    { timeout: 30_000, interval: 250 },
  )

  const created = await browser.execute(
    async (workName, filePath) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) return { ok: false, detail: 'không có cầu IPC' }
      try {
        await internals.invoke('create_work_from_file', {
          name: workName,
          sourceLang: 'en',
          genre: 'general',
          path: filePath,
        })
        return { ok: true, detail: '' }
      } catch (err) {
        return { ok: false, detail: String(err && err.code ? err.code : err) }
      }
    },
    name,
    path,
  )
  if (!created.ok) {
    throw new Error(`Fixture không tạo được Tác phẩm "${name}" từ .docx: ${created.detail}`)
  }

  await resetPanelState()
  await browser.keys(['Meta', '2'])
  await waitForGridRows(2)
}

/** Bounding rects of every row's cell, one array per column, in DOM (== row) order.
 * `browser.execute` ships this function body to the page standalone, with no access to a
 * module-level constant, so the column selector list is inlined here. */
function measureGrid() {
  const selectors = ['.col-rule', '.col-num', '.col-src', '.col-tgt', '.col-state']
  const columns = selectors.map((sel) => [...document.querySelectorAll(`${sel} > *`)])
  const columnCounts = columns.map((cells) => cells.length)
  const rowCount = columnCounts[0]
  const rows = []
  for (let i = 0; i < rowCount; i += 1) {
    rows.push(columns.map((cells) => cells[i].getBoundingClientRect()))
  }
  return {
    columnCounts,
    rowCount,
    tops: rows.map((row) => row.map((r) => r.top)),
    bottoms: rows.map((row) => row.map((r) => r.bottom)),
    srcHeights: rows.map((row) => row[selectors.indexOf('.col-src')].height),
    images: document.querySelectorAll('[data-col="src"] .grid-image').length,
  }
}

describe('Lưới 5 cột giữ hàng thẳng khi một ảnh làm một hàng cao hơn (FR42/FR43, ticket #68)', () => {
  it('mọi cột cùng hàng có top khớp nhau, và hai hàng không đè lên nhau', async () => {
    const path = writeFixtureDocx()
    try {
      await openWorkFromDocx(`e2e-grid-image-${Date.now()}`, path)

      const grid = await browser.execute(measureGrid)

      expect(grid.images).toBe(1)
      expect(new Set(grid.columnCounts).size).toBe(1)
      expect(grid.rowCount).toBe(2)

      for (const rowTops of grid.tops) {
        expect(Math.max(...rowTops) - Math.min(...rowTops)).toBeLessThanOrEqual(TOLERANCE_PX)
      }

      // Hàng mang ảnh (hàng 0) phải THẬT SỰ cao hơn hàng không mang ảnh — nếu không, phép so
      // top ở trên chỉ đúng vì không có gì để lệch.
      expect(grid.srcHeights[0]).toBeGreaterThan(grid.srcHeights[1])

      const row0BottomMax = Math.max(...grid.bottoms[0])
      const row1TopMin = Math.min(...grid.tops[1])
      expect(row0BottomMax).toBeLessThanOrEqual(row1TopMin + TOLERANCE_PX)
    } finally {
      unlinkSync(path)
    }
  })
})
