import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('../../src/config/reviewerImport', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/reviewerImport')>()),
  reviewerImportOpenPreview: async () => ({ outcome: 'ipc_unavailable' }),
}))

const NEW_WORK = { meta: { work_id: 'w-moi-nhap', name: 'Tac pham nhap moi', chapter_count: 1 } }

vi.mock('../../src/config/project', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/project')>()),
  previewImportEncodingFromText: async () => ({
    preview: { confidence: 'self_declared', selected_encoding: 'UTF-8', candidates: [] },
    error: null,
  }),
  confirmImportWithEncoding: async () => ({ created: NEW_WORK, error: null }),
}))

vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))

const OPENED_WORK = { meta: { work_id: 'w-co-san' }, folder: '/tmp/w-co-san.atproj', chapter_id: 1 }

vi.mock('../../src/config/library', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/library')>()),
  openWork: async () => ({ opened: OPENED_WORK, error: null }),
}))

vi.mock('../../src/config/chapter', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/chapter')>()),
  listChapters: async () => ({ chapters: [], error: null }),
}))

vi.mock('../../src/panels/editorPanelState', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/panels/editorPanelState')>()),
  flushEditorBeforeDiscreteWrite: async () => 'clean' as const,
  flushChapterPositionNow: async () => {},
}))

async function dirtyBothStates() {
  const alignment = await import('../../src/alignmentState')
  const reviewer = await import('../../src/reviewerImportState')
  await alignment.openAlignmentOverlay(null)
  await reviewer.openReviewerImportPreviewOverlay()
  expect(alignment.alignmentOverlayIsOpen.value).toBe(true)
  expect(alignment.alignmentStatus.value).toBe('no_chapter')
  expect(reviewer.reviewerImportOverlayIsOpen.value).toBe(true)
  expect(reviewer.reviewerImportStatus.value).toBe('ipc_unavailable')
  return { alignment, reviewer }
}

beforeEach(() => {
  vi.resetModules()
})

describe('alignment and reviewer-import state do not outlive the open Work', () => {
  it('importing a new Work resets both', async () => {
    const { alignment, reviewer } = await dirtyBothStates()
    const imp = await import('../../src/modes/libraryImport')
    const preview = await import('../../src/importPreviewState')

    imp.pastedText.value = 'Cau nguyen van.'
    await imp.submitPastedText()
    const result = await preview.confirmImportPreview()
    imp.finishImportSubmission(result.created, result.error)
    expect(imp.createdWork.value).not.toBeNull()

    expect(alignment.alignmentOverlayIsOpen.value).toBe(false)
    expect(alignment.alignmentStatus.value).toBe('unknown')
    expect(reviewer.reviewerImportOverlayIsOpen.value).toBe(false)
    expect(reviewer.reviewerImportStatus.value).toBe('unknown')
  })

  it('opening an existing Work resets both', async () => {
    const { alignment, reviewer } = await dirtyBothStates()
    const chapters = await import('../../src/modes/libraryChapters')

    expect(await chapters.openWorkById('w-co-san')).toBe(true)

    expect(alignment.alignmentOverlayIsOpen.value).toBe(false)
    expect(alignment.alignmentStatus.value).toBe('unknown')
    expect(reviewer.reviewerImportOverlayIsOpen.value).toBe(false)
    expect(reviewer.reviewerImportStatus.value).toBe('unknown')
  })
})
