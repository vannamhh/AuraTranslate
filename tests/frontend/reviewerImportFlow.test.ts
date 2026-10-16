import { describe, expect, it, vi } from 'vitest'

describe('reviewer import flow opens Nối tay per Chapter', () => {
  it('gives the second mismatched Chapter its own numbers', async () => {
    vi.resetModules()
    const opened: Array<{ id: number | null; flow: unknown }> = []
    vi.doMock('../../src/alignmentState', async () => {
      const listeners = new Set<(e: unknown) => void>()
      return {
        onAlignmentClosed: (l: (e: unknown) => void) => {
          listeners.add(l)
          return () => listeners.delete(l)
        },
        openAlignmentOverlay: async (id: number | null, flow: unknown) => {
          opened.push({ id, flow })
          if (opened.length === 1) setTimeout(() => listeners.forEach((l) => l({ chapterId: id, flow, chainEnded: false })), 0)
        },
      }
    })
    vi.doMock('../../src/config/reviewerImport', () => ({
      reviewerImportOpenPreview: async () => ({
        outcome: 'loaded',
        preview: { file_name: 'r.docx', file_kind: 'docx', chapters: [], skipped: [], image_rows_ignored: 0 },
      }),
      reviewerImportConfirm: async () => ({
        summary: {
          chapter_count: 2,
          row_count: 10,
          replaced_count: 0,
          chapters: [
            { chapter_id: 5, chapter_ord: 2, matched_rows: 3, total_rows: 4, unresolved_count: 2 },
            { chapter_id: 4, chapter_ord: 1, matched_rows: 5, total_rows: 6, unresolved_count: 1 },
          ],
          harvest_candidate_count: 0,
          harvest_error: null,
        },
        error: null,
      }),
      reviewerImportCancel: async () => ({ ok: true, error: null }),
      reviewerHarvestRefresh: async () => ({ refresh: { harvest_candidate_count: 0, harvest_error: null }, error: null }),
    }))
    try {
      const state = await import('../../src/reviewerImportState')
      await state.openReviewerImportPreviewOverlay()
      await state.confirmReviewerImportPreview()
      await vi.waitFor(() => expect(opened).toHaveLength(2))
      expect(opened[0].id).toBe(4)
      expect(opened[1]).toEqual({
        id: 5,
        flow: { chapterIndex: 2, chapterCount: 2, matchedRows: 3, totalRows: 4, unresolved: 2 },
      })
    } finally {
      vi.doUnmock('../../src/alignmentState')
      vi.doUnmock('../../src/config/reviewerImport')
    }
  })
})
