/**
 * `src/main.ts` is the only place that attaches the reviewer-import hooks. Read as text for the
 * same reason as `mainConfirmImportPreviewWiring.test.ts`: importing it runs the whole bootstrap.
 */
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it, vi } from 'vitest'

describe('main.ts wires the reviewer-import hooks', () => {
  const source = readFileSync(resolve(process.cwd(), 'src/main.ts'), 'utf8')
  const start = source.indexOf('installReviewerImportHooks({')
  const block =
    start === -1
      ? ''
      : source
          .slice(start, source.indexOf('})', start))
          .split('\n')
          .filter((line) => !line.trim().startsWith('//'))
          .join('\n')

  it('attaches resetReviewMode to the after-imported hook', () => {
    expect(start).toBeGreaterThan(-1)
    expect(block).toMatch(/afterImported:\s*resetReviewMode/)
  })

  it('attaches openGlossaryQueue to the after-closed-with-harvest hook', () => {
    expect(block).toMatch(/afterClosedWithHarvest:[\s\S]*openGlossaryQueue\(\)/)
  })
})

describe('harvest path runs without Review Mode loaded', () => {
  it('imports, confirms and closes with the queue hook firing, never loading reviewModeState', async () => {
    try {
      vi.resetModules()
      vi.doMock('../../src/reviewModeState', () => {
        throw new Error('reviewModeState must not be loaded by the harvest path')
      })
      vi.doMock('../../src/config/reviewerImport', () => ({
        reviewerImportOpenPreview: async () => ({
          outcome: 'loaded',
          preview: { file_name: 'r.docx', file_kind: 'docx', chapters: [], skipped: [], image_rows_ignored: 0 },
        }),
        reviewerImportConfirm: async () => ({
          summary: { chapter_count: 1, row_count: 3, replaced_count: 0, harvest_candidate_count: 2, harvest_error: null },
          error: null,
        }),
        reviewerImportCancel: async () => ({ ok: true, error: null }),
      }))
      const state = await import('../../src/reviewerImportState')
      const opened = vi.fn()
      state.installReviewerImportHooks({ afterClosedWithHarvest: opened })
      await state.openReviewerImportPreviewOverlay()
      await state.confirmReviewerImportPreview()
      await state.cancelReviewerImportPreview()
      expect(opened).toHaveBeenCalledTimes(1)
    } finally {
      vi.doUnmock('../../src/reviewModeState')
      vi.doUnmock('../../src/config/reviewerImport')
    }
  })
})
