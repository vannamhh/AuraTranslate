import { beforeEach, describe, expect, it, vi } from 'vitest'

const pendingMock = vi.fn()

vi.mock('../../src/config/glossary', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/glossary')>()),
  pendingGlossaryCandidates: () => pendingMock(),
  lookupGlossaryTerm: async () => ({ found: 'none', workTierAvailable: true }),
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

function candidate(id: number) {
  return {
    id,
    source_term: `term-${id}`,
    candidate_origin: 'import_scan',
    resolution: null,
    created_at: '2026-10-15T00:00:00.000Z',
    occurrence_count: 3,
    context_example: null,
    han_viet_suggestion: null,
    han_viet_status: 'not_chinese',
    chapter_span_count: 1,
    replaced_translation: null,
    proposed_translation: null,
    changed_count: null,
    seen_count: null,
  }
}

beforeEach(() => {
  vi.resetModules()
  pendingMock.mockReset()
})

describe('the remembered queue cursor does not outlive the open Work', () => {
  it('importing a new Work clears the remembered candidate id, so the next open starts at row 0', async () => {
    const queue = await import('../../src/glossaryQueueState')
    const imp = await import('../../src/modes/libraryImport')
    const preview = await import('../../src/importPreviewState')
    pendingMock.mockResolvedValue({ candidates: [candidate(1), candidate(2), candidate(3)], error: null })
    await queue.openGlossaryQueue()
    queue.setGlossaryQueueCursor(2)
    queue.closeGlossaryQueue()

    imp.pastedText.value = 'Cau nguyen van.'
    await imp.submitPastedText()
    const result = await preview.confirmImportPreview()
    imp.finishImportSubmission(result.created, result.error)
    expect(imp.createdWork.value).not.toBeNull()

    await queue.openGlossaryQueue()
    expect(queue.queueCursor.value).toBe(0)
  })

  it('opening an existing Work clears the remembered candidate id, so the next open starts at row 0', async () => {
    const queue = await import('../../src/glossaryQueueState')
    const chapters = await import('../../src/modes/libraryChapters')
    pendingMock.mockResolvedValue({ candidates: [candidate(1), candidate(2), candidate(3)], error: null })
    await queue.openGlossaryQueue()
    queue.setGlossaryQueueCursor(2)
    queue.closeGlossaryQueue()

    expect(await chapters.openWorkById('w-co-san')).toBe(true)

    await queue.openGlossaryQueue()
    expect(queue.queueCursor.value).toBe(0)
  })
})
