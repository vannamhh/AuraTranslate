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

vi.mock('../../src/config/reviewerImport', () => ({
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
  for (const m of [openMock, confirmMock, cancelMock]) m.mockReset()
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
    confirmMock.mockResolvedValue({ summary: { chapter_count: 1, row_count: 3, replaced_count: 0 }, error: null })
    await state.openReviewerImportPreviewOverlay()
    await state.confirmReviewerImportPreview()
    expect(state.reviewerImportStatus.value).toBe('done')
    expect(state.reviewerImportSummary.value?.row_count).toBe(3)
    await state.cancelReviewerImportPreview()
    expect(state.reviewerImportOverlayIsOpen.value).toBe(false)
    expect(cancelMock).not.toHaveBeenCalled()
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
