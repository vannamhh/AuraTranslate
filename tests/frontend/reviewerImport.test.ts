/**
 * Reviewer-copy import overlay: state transitions around the Rust-held plan, and the
 * `vue/no-v-html` rule. `config/reviewerImport.ts` is the IPC boundary and is mocked.
 */
import { resolve } from 'node:path'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ESLint } from 'eslint'
import type { ReviewerImportPreview } from '../../src/config/reviewerImport'

const openMock = vi.fn()
const confirmMock = vi.fn()
const cancelMock = vi.fn()
const refreshMock = vi.fn()
const openAlignmentMock = vi.fn()
let alignmentListeners: Array<(event: { chapterId: number | null; flow: unknown; chainEnded: boolean }) => void> = []

vi.mock('../../src/alignmentState', () => ({
  openAlignmentOverlay: (...args: unknown[]) => openAlignmentMock(...args),
  onAlignmentClosed: (listener: (typeof alignmentListeners)[number]) => {
    alignmentListeners.push(listener)
    return () => {
      alignmentListeners = alignmentListeners.filter((l) => l !== listener)
    }
  },
}))

vi.mock('../../src/config/reviewerImport', () => ({
  reviewerHarvestRefresh: (...args: unknown[]) => refreshMock(...args),
  reviewerImportOpenPreview: (...args: unknown[]) => openMock(...args),
  reviewerImportConfirm: (...args: unknown[]) => confirmMock(...args),
  reviewerImportCancel: (...args: unknown[]) => cancelMock(...args),
}))

function preview(): ReviewerImportPreview {
  return {
    file_name: 'review.docx',
    file_kind: 'docx',
    chapters: [{ chapter_id: 1, chapter_ord: 1, title: null, row_count: 3, replaces: null }],
    skipped: [],
    image_rows_ignored: 0,
  }
}

function ipcError(code: string) {
  return { code, message_key: 'err.export.reviewer_import_no_pending', params: {}, retryable: false }
}

async function fresh() {
  vi.resetModules()
  for (const m of [openMock, confirmMock, cancelMock, refreshMock, openAlignmentMock]) m.mockReset()
  alignmentListeners = []
  cancelMock.mockResolvedValue({ ok: true, error: null })
  return await import('../../src/reviewerImportState')
}

describe('reviewer import overlay state', () => {
  let state: Awaited<ReturnType<typeof fresh>>
  beforeEach(async () => {
    state = await fresh()
  })

  it('stays closed when the file dialog is cancelled', async () => {
    openMock.mockResolvedValue({ outcome: 'cancelled' })
    await state.openReviewerImportPreviewOverlay()
    expect(state.reviewerImportOverlayIsOpen.value).toBe(false)
    expect(state.reviewerImportOpening.value).toBe(false)
  })

  it('opens with the preview and writes nothing until confirm', async () => {
    openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openReviewerImportPreviewOverlay()
    expect(state.reviewerImportOverlayIsOpen.value).toBe(true)
    expect(state.reviewerImportStatus.value).toBe('loaded')
    expect(state.reviewerImportPreview.value?.chapters).toHaveLength(1)
    expect(confirmMock).not.toHaveBeenCalled()
  })

  it('shows a load error without a preview', async () => {
    openMock.mockResolvedValue({ outcome: 'error', error: ipcError('export.reviewer_import_unreadable') })
    await state.openReviewerImportPreviewOverlay()
    expect(state.reviewerImportStatus.value).toBe('error')
    expect(state.reviewerImportPreview.value).toBeNull()
    expect(state.reviewerImportLoadError.value?.code).toBe('export.reviewer_import_unreadable')
  })

  it('cancel closes the overlay and tells Rust to drop the plan', async () => {
    openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openReviewerImportPreviewOverlay()
    await state.cancelReviewerImportPreview()
    expect(state.reviewerImportOverlayIsOpen.value).toBe(false)
    expect(cancelMock).toHaveBeenCalledTimes(1)
  })

  it('confirm moves to done with the summary, and closing afterwards does not call cancel', async () => {
    openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    confirmMock.mockResolvedValue({ summary: { chapter_count: 1, row_count: 3, replaced_count: 0, chapters: [], harvest_candidate_count: 0, harvest_error: null }, error: null })
    await state.openReviewerImportPreviewOverlay()
    await state.confirmReviewerImportPreview()
    expect(state.reviewerImportStatus.value).toBe('done')
    expect(state.reviewerImportSummary.value?.row_count).toBe(3)
    await state.cancelReviewerImportPreview()
    expect(state.reviewerImportOverlayIsOpen.value).toBe(false)
    expect(cancelMock).not.toHaveBeenCalled()
  })

  it('a harvest failure still counts as a finished import and keeps its own error', async () => {
    openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    const harvestError = ipcError('export.harvest_failed')
    confirmMock.mockResolvedValue({
      summary: { chapter_count: 1, row_count: 3, replaced_count: 0, chapters: [], harvest_candidate_count: null, harvest_error: harvestError },
      error: null,
    })
    await state.openReviewerImportPreviewOverlay()
    await state.confirmReviewerImportPreview()
    expect(state.reviewerImportStatus.value).toBe('done')
    expect(state.reviewerImportConfirmError.value).toBeNull()
    expect(state.reviewerImportSummary.value?.harvest_error?.code).toBe('export.harvest_failed')
  })

  it('a successful confirm calls the after-imported hook exactly once', async () => {
    const afterImported = vi.fn()
    state.installReviewerImportHooks({ afterImported })
    openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    confirmMock.mockResolvedValue({ summary: { chapter_count: 1, row_count: 3, replaced_count: 0, chapters: [], harvest_candidate_count: 0, harvest_error: null }, error: null })
    await state.openReviewerImportPreviewOverlay()
    await state.confirmReviewerImportPreview()
    expect(afterImported).toHaveBeenCalledTimes(1)
  })

  it('a failed confirm does not call the after-imported hook', async () => {
    const afterImported = vi.fn()
    state.installReviewerImportHooks({ afterImported })
    openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    confirmMock.mockResolvedValue({ summary: null, error: ipcError('store.write_failed') })
    await state.openReviewerImportPreviewOverlay()
    await state.confirmReviewerImportPreview()
    expect(afterImported).not.toHaveBeenCalled()
  })

  describe('closing the overlay after an import', () => {
    async function importThenClose(summary: { harvest_candidate_count: number | null; harvest_error: unknown }) {
      const afterClosedWithHarvest = vi.fn()
      state.installReviewerImportHooks({ afterClosedWithHarvest })
      openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
      confirmMock.mockResolvedValue({ summary: { chapter_count: 1, row_count: 3, replaced_count: 0, chapters: [], ...summary }, error: null })
      await state.openReviewerImportPreviewOverlay()
      await state.confirmReviewerImportPreview()
      await state.cancelReviewerImportPreview()
      return afterClosedWithHarvest
    }

    it('opens the pending queue when the harvest queued candidates', async () => {
      const hook = await importThenClose({ harvest_candidate_count: 2, harvest_error: null })
      expect(hook).toHaveBeenCalledTimes(1)
    })

    it('opens nothing when the harvest queued no candidate', async () => {
      const hook = await importThenClose({ harvest_candidate_count: 0, harvest_error: null })
      expect(hook).not.toHaveBeenCalled()
    })

    it('opens nothing when the harvest failed', async () => {
      const hook = await importThenClose({ harvest_candidate_count: null, harvest_error: ipcError('export.harvest_failed') })
      expect(hook).not.toHaveBeenCalled()
    })

    it('opens nothing when cancelled before confirming', async () => {
      const afterClosedWithHarvest = vi.fn()
      state.installReviewerImportHooks({ afterClosedWithHarvest })
      openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
      await state.openReviewerImportPreviewOverlay()
      await state.cancelReviewerImportPreview()
      expect(afterClosedWithHarvest).not.toHaveBeenCalled()
    })
  })

  it('a failed confirm keeps the preview so the user can retry', async () => {
    openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    confirmMock.mockResolvedValue({ summary: null, error: ipcError('store.write_failed') })
    await state.openReviewerImportPreviewOverlay()
    await state.confirmReviewerImportPreview()
    expect(state.reviewerImportStatus.value).toBe('loaded')
    expect(state.reviewerImportPreview.value).not.toBeNull()
    expect(state.reviewerImportConfirmError.value?.code).toBe('store.write_failed')
    expect(state.reviewerImportConfirming.value).toBe(false)
  })

  it('no_pending on confirm drops the preview and shows the error', async () => {
    openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    confirmMock.mockResolvedValue({ summary: null, error: ipcError('export.reviewer_import_no_pending') })
    await state.openReviewerImportPreviewOverlay()
    await state.confirmReviewerImportPreview()
    expect(state.reviewerImportStatus.value).toBe('error')
    expect(state.reviewerImportPreview.value).toBeNull()
  })

  it('a second open while the overlay is open does not call the dialog again', async () => {
    openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openReviewerImportPreviewOverlay()
    await state.openReviewerImportPreviewOverlay()
    expect(openMock).toHaveBeenCalledTimes(1)
  })
})

describe('import flow into hand joining', () => {
  let state: Awaited<ReturnType<typeof fresh>>
  beforeEach(async () => {
    state = await fresh()
  })

  function match(chapterId: number, ord: number, unresolved: number, matched = 2, total = 3) {
    return { chapter_id: chapterId, chapter_ord: ord, matched_rows: matched, total_rows: total, unresolved_count: unresolved }
  }

  async function confirmWith(chapters: ReturnType<typeof match>[], harvest: { count: number | null; error?: unknown } = { count: 0 }) {
    const afterClosedWithHarvest = vi.fn()
    state.installReviewerImportHooks({ afterClosedWithHarvest })
    openMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    confirmMock.mockResolvedValue({
      summary: {
        chapter_count: chapters.length,
        row_count: 3,
        replaced_count: 0,
        chapters,
        harvest_candidate_count: harvest.count,
        harvest_error: harvest.error ?? null,
      },
      error: null,
    })
    await state.openReviewerImportPreviewOverlay()
    await state.confirmReviewerImportPreview()
    await new Promise((r) => setTimeout(r, 0))
    return afterClosedWithHarvest
  }

  function closeAlignment(chapterId: number, chainEnded = false) {
    for (const listener of [...alignmentListeners]) listener({ chapterId, flow: {}, chainEnded })
  }

  async function settle() {
    await new Promise((r) => setTimeout(r, 0))
  }

  it('opens hand joining on the mismatched Chapter, carrying that Chapter\'s own match numbers', async () => {
    await confirmWith([match(10, 1, 0, 3, 3), match(30, 3, 1, 2, 3)])
    expect(state.reviewerImportOverlayIsOpen.value).toBe(false)
    expect(openAlignmentMock).toHaveBeenCalledTimes(1)
    expect(openAlignmentMock).toHaveBeenCalledWith(30, { chapterIndex: 1, chapterCount: 1, matchedRows: 2, totalRows: 3, unresolved: 1 })
  })

  it('walks the mismatched Chapters in chapter_ord order, one at a time', async () => {
    await confirmWith([match(50, 5, 1), match(30, 3, 2)])
    expect(openAlignmentMock.mock.calls.map((c) => c[0])).toEqual([30])
    expect(openAlignmentMock.mock.calls[0][1]).toMatchObject({ chapterIndex: 1, chapterCount: 2 })
    refreshMock.mockResolvedValue({ refresh: { harvest_candidate_count: 0, harvest_error: null }, error: null })
    closeAlignment(30)
    await settle()
    expect(openAlignmentMock.mock.calls.map((c) => c[0])).toEqual([30, 50])
    expect(openAlignmentMock.mock.calls[1][1]).toMatchObject({ chapterIndex: 2, chapterCount: 2 })
    expect(refreshMock).not.toHaveBeenCalled()
  })

  it('refreshes the harvest after the last Chapter and opens the harvest screen when it queued candidates', async () => {
    const hook = await confirmWith([match(30, 3, 1)], { count: 0 })
    refreshMock.mockResolvedValue({ refresh: { harvest_candidate_count: 4, harvest_error: null }, error: null })
    closeAlignment(30)
    await settle()
    expect(refreshMock).toHaveBeenCalledTimes(1)
    expect(hook).toHaveBeenCalledTimes(1)
    expect(state.reviewerImportOverlayIsOpen.value).toBe(false)
  })

  it('decides on the refreshed count, not the count at import time', async () => {
    const hook = await confirmWith([match(30, 3, 1)], { count: 7 })
    refreshMock.mockResolvedValue({ refresh: { harvest_candidate_count: 0, harvest_error: null }, error: null })
    closeAlignment(30)
    await settle()
    expect(hook).not.toHaveBeenCalled()
    expect(state.reviewerImportOverlayIsOpen.value).toBe(true)
    expect(state.reviewerImportStatus.value).toBe('done')
    expect(state.reviewerImportSummary.value?.harvest_candidate_count).toBe(0)
  })

  it('Esc mid-chain ends the chain and goes to the harvest', async () => {
    const hook = await confirmWith([match(30, 3, 1), match(50, 5, 1)])
    refreshMock.mockResolvedValue({ refresh: { harvest_candidate_count: 2, harvest_error: null }, error: null })
    closeAlignment(30, true)
    await settle()
    expect(openAlignmentMock).toHaveBeenCalledTimes(1)
    expect(refreshMock).toHaveBeenCalledTimes(1)
    expect(hook).toHaveBeenCalledTimes(1)
  })

  it('ignores a close of hand joining that is not part of the flow', async () => {
    await confirmWith([match(30, 3, 0)])
    expect(openAlignmentMock).not.toHaveBeenCalled()
    expect(alignmentListeners).toHaveLength(0)
  })

  it('everything matched with candidates opens the harvest straight away', async () => {
    const hook = await confirmWith([match(10, 1, 0)], { count: 3 })
    expect(openAlignmentMock).not.toHaveBeenCalled()
    expect(refreshMock).not.toHaveBeenCalled()
    expect(hook).toHaveBeenCalledTimes(1)
    expect(state.reviewerImportOverlayIsOpen.value).toBe(false)
  })

  it('everything matched without candidates stays on the imported screen with the match numbers', async () => {
    const hook = await confirmWith([match(10, 1, 0, 3, 3)], { count: 0 })
    expect(hook).not.toHaveBeenCalled()
    expect(state.reviewerImportOverlayIsOpen.value).toBe(true)
    expect(state.reviewerImportStatus.value).toBe('done')
    expect(state.reviewerImportMatchTotals.value).toEqual({ matched: 3, total: 3, unresolved: 0 })
  })

  it('an import-time harvest error skips the refresh and stops on the imported screen after hand joining', async () => {
    const hook = await confirmWith([match(30, 3, 1)], { count: null, error: ipcError('export.harvest_failed') })
    expect(openAlignmentMock).toHaveBeenCalledTimes(1)
    closeAlignment(30)
    await settle()
    expect(refreshMock).not.toHaveBeenCalled()
    expect(hook).not.toHaveBeenCalled()
    expect(state.reviewerImportOverlayIsOpen.value).toBe(true)
    expect(state.reviewerImportSummary.value?.harvest_error?.code).toBe('export.harvest_failed')
  })

  it('a refresh error keeps its error on the imported screen and opens no harvest', async () => {
    const hook = await confirmWith([match(30, 3, 1)], { count: 0 })
    refreshMock.mockResolvedValue({ refresh: { harvest_candidate_count: null, harvest_error: ipcError('export.harvest_failed') }, error: null })
    closeAlignment(30)
    await settle()
    expect(hook).not.toHaveBeenCalled()
    expect(state.reviewerImportOverlayIsOpen.value).toBe(true)
    expect(state.reviewerImportSummary.value?.harvest_error?.code).toBe('export.harvest_failed')
  })

  it('a refresh command failure is shown as the harvest error, not swallowed', async () => {
    const hook = await confirmWith([match(30, 3, 1)], { count: 0 })
    refreshMock.mockResolvedValue({ refresh: null, error: ipcError('work.none_open') })
    closeAlignment(30)
    await settle()
    expect(hook).not.toHaveBeenCalled()
    expect(state.reviewerImportSummary.value?.harvest_error?.code).toBe('work.none_open')
  })

  it('reset during the chain drops the listener', async () => {
    await confirmWith([match(30, 3, 1)])
    expect(alignmentListeners).toHaveLength(1)
    state.resetReviewerImport()
    expect(alignmentListeners).toHaveLength(0)
  })
})

describe('vue/no-v-html', { timeout: 120_000 }, () => {
  // The typed project service only knows files on disk, so the text is linted under a real path.
  async function lint(template: string) {
    const eslint = new ESLint({ cwd: resolve(__dirname, '../..') })
    const code = `<script setup lang="ts">\nconst html = 'x'\n</script>\n<template>\n${template}\n</template>\n`
    const [result] = await eslint.lintText(code, { filePath: resolve(__dirname, '../../src/ReviewerImportOverlay.vue') })
    return result.messages.filter((m) => m.ruleId === 'vue/no-v-html')
  }

  it('flags v-html in a .vue file', async () => {
    expect(await lint('  <div v-html="html"></div>')).toHaveLength(1)
  })

  it('accepts text interpolation', async () => {
    expect(await lint('  <div>{{ html }}</div>')).toHaveLength(0)
  })
})

describe('preview of a second import', () => {
  it('names the accepted groups that will be lost, next to the hand-made ones', async () => {
    const state = await fresh()
    const { mount } = await import('@vue/test-utils')
    const Overlay = (await import('../../src/ReviewerImportOverlay.vue')).default
    const { t } = await import('../../src/i18n')
    const withReplaced = preview()
    withReplaced.chapters = [
      {
        chapter_id: 1,
        chapter_ord: 1,
        title: null,
        row_count: 3,
        replaces: { file_name: 'old.docx', stale: false, user_group_count: 2, accepted_group_count: 4 },
      },
    ]
    openMock.mockResolvedValue({ outcome: 'loaded', preview: withReplaced })
    await state.openReviewerImportPreviewOverlay()

    const wrapper = mount(Overlay, { attachTo: document.body })
    const text = wrapper.text()
    expect(text).toContain(t('reviewer.import.replaces_user_groups', { count: '2' }))
    expect(text).toContain(t('reviewer.import.replaces_accepted_groups', { count: '4' }))
    wrapper.unmount()
  })

  it('says nothing about accepted groups when there are none', async () => {
    const state = await fresh()
    const { mount } = await import('@vue/test-utils')
    const Overlay = (await import('../../src/ReviewerImportOverlay.vue')).default
    const withReplaced = preview()
    withReplaced.chapters[0].replaces = { file_name: 'old.docx', stale: false, user_group_count: 0, accepted_group_count: 0 }
    openMock.mockResolvedValue({ outcome: 'loaded', preview: withReplaced })
    await state.openReviewerImportPreviewOverlay()

    const wrapper = mount(Overlay, { attachTo: document.body })
    expect(wrapper.text()).not.toContain('đã chấp nhận')
    wrapper.unmount()
  })
})
