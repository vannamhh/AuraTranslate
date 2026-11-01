import { beforeEach, describe, expect, it, vi } from 'vitest'
import { failNextSave, FIXTURE_SEGMENTS, recordSave, resetRecorder } from './support/segmentFixture'

const openMock = vi.fn()
const diffMock = vi.fn()
const acceptMock = vi.fn()
const openChapterMock = vi.fn()
const dockSuspendedSpy = vi.fn()

vi.mock('../../src/config/alignment', () => ({
  alignmentOpen: (...args: unknown[]) => openMock(...args),
  reviewDiff: (...args: unknown[]) => diffMock(...args),
  alignmentJoin: vi.fn(),
  alignmentSkip: vi.fn(),
  alignmentUnjoin: vi.fn(),
  reviewAcceptChange: (...args: unknown[]) => acceptMock(...args),
  reviewSkipChange: vi.fn(),
}))

vi.mock('../../src/layout/dockController', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/layout/dockController')>()
  return {
    ...actual,
    setDockSuspended: (value: boolean) => {
      dockSuspendedSpy(value)
      actual.setDockSuspended(value)
    },
  }
})

vi.mock('../../src/config/segment', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/segment')>()),
  readOpenChapterSegments: async () => ({
    loaded: {
      chapter_id: 2,
      segments: FIXTURE_SEGMENTS.map((s) => ({ ...s })),
      caret_segment_id: FIXTURE_SEGMENTS[0].id,
      assets: [],
      assets_dir: '',
      tm_filled_segment_ids: [],
      tm_prefill: { kind: 'ran', hidden_count: 0 },
    },
    error: null,
  }),
  saveSegmentTargets: async (chapterId: number, edits: { id: number; target_text: string }[]) => {
    return recordSave(chapterId, edits)
  },
  saveChapterPosition: async () => ({ error: null }),
}))

const ROW = (chapter_id: number) => ({
  chapter_id,
  ord: chapter_id,
  title: `Chuong ${chapter_id}`,
  status: 'not_started',
  segment_count: 3,
  translated_count: 0,
  unconfirmed_count: 0,
  unlinked_count: 0,
  in_translation_count: 3,
  updated_at: '2026-10-01T00:00:00Z',
  origin_author: null,
  origin_site_name: null,
  origin_url: null,
  origin_published_at: null,
})

vi.mock('../../src/config/chapter', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/chapter')>()),
  openChapter: (...args: unknown[]) => openChapterMock(...args),
  openAdjacentChapter: async () => ({
    switched: { outcome: 'moved', chapter: { chapter_id: 2, source_text: 'x', source_lang: 'en' } },
    error: null,
  }),
  listChapters: async () => ({ chapters: [ROW(1), ROW(2)], error: null }),
  readOpenChapter: async () => ({
    chapter: { chapter_id: 2, source_text: 'x', source_lang: 'en' },
    error: null,
  }),
}))

const alignmentOf = (chapterId: number) => ({
  alignment: {
    chapter_id: chapterId,
    file_name: 'r.docx',
    file_kind: 'docx',
    rows: [{ id: 100, kind: 'text', source_text: 'a', target_text: 'b' }],
    segments: [{ id: 10, ord: 1, role: null, source_text: 'a', target_text: 'c' }],
    groups: [],
    unmatched_row_ids: [],
    unmatched_segment_ids: [],
    is_resolved: true,
  },
  error: null,
})
const diff = {
  diff: {
    chapter_id: 1,
    pairs: [
      {
        group_id: 1,
        decided_by: 'machine',
        segment_ids: [10],
        row_ids: [100],
        spans: [{ kind: 'insert', text: 'b' }],
        decision: null,
      },
    ],
  },
  error: null,
}
const failure = (code: string) => ({
  alignment: null,
  error: { code, message_key: 'err.unknown', params: {}, retryable: false },
})

async function load() {
  vi.resetModules()
  const review = await import('../../src/reviewModeState')
  const editor = await import('../../src/panels/editorPanelState')
  const library = await import('../../src/modes/libraryChapters')
  const reading = await import('../../src/modes/readingState')
  const modes = await import('../../src/modes/modeState')
  return { review, editor, library, reading, modes }
}

beforeEach(() => {
  resetRecorder()
  openMock.mockReset()
  diffMock.mockReset()
  diffMock.mockResolvedValue(diff)
  acceptMock.mockReset()
  acceptMock.mockResolvedValue({ outcome: { needs_confirmation: false, unsigned_draft: null }, error: null })
  dockSuspendedSpy.mockReset()
  openChapterMock.mockReset()
  openChapterMock.mockImplementation(async (id: number) => ({
    chapter: { chapter_id: id, source_text: 'x', source_lang: 'en' },
    error: null,
  }))
})

describe('Review Mode khi mở Chương khác', () => {
  it('Library: mở Chương khác từ openCurrentChapter đóng phiên, accept sau đó không ghi', async () => {
    const { review, library } = await load()
    openMock.mockResolvedValue(alignmentOf(1))
    await review.openReviewMode(1)
    expect(review.reviewModeStatus.value).toBe('open')
    await library.loadChapters()
    library.moveLibraryChapterSelection(1, false)
    review.reviewDiffNext()
    expect(review.reviewModeCurrentPair.value?.acceptable).toBe(true)
    await library.openCurrentChapter()
    expect(openChapterMock).toHaveBeenCalledWith(2)
    expect(review.reviewModeStatus.value).toBe('idle')
    await review.reviewAcceptChange()
    expect(acceptMock).not.toHaveBeenCalled()
  })

  it('Chế độ đọc: openSearchHitInReading sang Chương khác đóng phiên', async () => {
    const { review, reading } = await load()
    openMock.mockResolvedValue(alignmentOf(1))
    await review.openReviewMode(1)
    expect(await reading.openSearchHitInReading(2, null)).toBe(true)
    expect(review.reviewModeStatus.value).toBe('idle')
  })

  it('Workspace: openChapterById sang Chương khác đóng phiên', async () => {
    const { review, editor } = await load()
    openMock.mockResolvedValue(alignmentOf(1))
    await review.openReviewMode(1)
    expect(await editor.openChapterById(2)).toBe(true)
    expect(review.reviewModeStatus.value).toBe('idle')
  })

  it('mở lại đúng Chương thì phiên còn nguyên', async () => {
    const { review, editor } = await load()
    openMock.mockResolvedValue(alignmentOf(1))
    await review.openReviewMode(1)
    review.reviewDiffNext()
    const key = review.reviewModeCurrentPairKey.value
    expect(key).not.toBeNull()
    expect(await editor.openChapterById(1)).toBe(true)
    expect(review.reviewModeStatus.value).toBe('open')
    expect(review.reviewModeCurrentPairKey.value).toBe(key)
  })

  it('flush trượt chặn lượt mở từ Library, phiên còn, người dùng ở Library và có câu báo', async () => {
    const { review, editor, library, modes } = await load()
    openMock.mockResolvedValue(alignmentOf(1))
    await review.openReviewMode(1)
    await library.loadChapters()
    library.moveLibraryChapterSelection(1, false)
    modes.setMode('library')
    await editor.ensureSegmentsLoaded()
    editor.setEditorCaret(11)
    editor.noteEditorEdit(11, 'dang go')
    failNextSave.value = true
    await library.openCurrentChapter()
    expect(modes.currentMode.value).toBe('library')
    expect(review.reviewModeStatus.value).toBe('open')
    expect(editor.editorNavNotice.value).toBe('chapter-flush-failed')
  })

  it('open_chapter lỗi từ Library thì phiên còn, vẫn ở Library và có câu chapter-switch-failed', async () => {
    const { review, editor, library, modes } = await load()
    openMock.mockResolvedValue(alignmentOf(1))
    await review.openReviewMode(1)
    await library.loadChapters()
    library.moveLibraryChapterSelection(1, false)
    modes.setMode('library')
    openChapterMock.mockResolvedValue({
      chapter: null,
      error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: false },
    })
    await library.openCurrentChapter()
    expect(modes.currentMode.value).toBe('library')
    expect(review.reviewModeStatus.value).toBe('open')
    expect(editor.editorNavNotice.value).toBe('chapter-switch-failed')
  })

  it('đang nạp ở A, mở B xong trước thì không bao giờ thành open', async () => {
    const { review, editor } = await load()
    let release: (v: unknown) => void = () => {}
    openMock.mockReturnValue(new Promise((resolve) => (release = resolve)))
    const pending = review.openReviewMode(1)
    expect(review.reviewModeStatus.value).toBe('loading')
    await editor.openChapterById(1)
    expect(review.reviewModeStatus.value).toBe('loading')
    await editor.openChapterById(2)
    expect(review.reviewModeStatus.value).toBe('idle')
    release(alignmentOf(1))
    await pending
    expect(review.reviewModeStatus.value).toBe('idle')
  })

  it('lời báo cũ của A biến mất khi mở B, còn khi mở lại A', async () => {
    const { review, editor } = await load()
    openMock.mockResolvedValue(failure('export.alignment_not_imported'))
    await review.openReviewMode(1)
    expect(review.reviewModeStatus.value).toBe('not_imported')
    await editor.openChapterById(1)
    expect(review.reviewModeStatus.value).toBe('not_imported')
    await editor.openChapterById(2)
    expect(review.reviewModeStatus.value).toBe('idle')
  })

  it('không có phiên thì mở Chương không đụng tới dock', async () => {
    const { review, editor } = await load()
    await editor.openChapterById(2)
    expect(dockSuspendedSpy).not.toHaveBeenCalled()
    expect(review.reviewModeStatus.value).toBe('idle')
  })

  it('goToNextChapter: đang nạp ở A không bao giờ thành open', async () => {
    const { review, editor } = await load()
    let release: (v: unknown) => void = () => {}
    openMock.mockReturnValue(new Promise((resolve) => (release = resolve)))
    const pending = review.openReviewMode(1)
    await editor.ensureSegmentsLoaded()
    editor.goToNextChapter()
    await vi.waitFor(() => expect(review.reviewModeStatus.value).toBe('idle'))
    release(alignmentOf(1))
    await pending
    expect(review.reviewModeStatus.value).toBe('idle')
  })

  it('goToNextChapter: lời báo not_imported của A đóng khi sang Chương khác', async () => {
    const { review, editor } = await load()
    openMock.mockResolvedValue(failure('export.alignment_not_imported'))
    await review.openReviewMode(1)
    await editor.ensureSegmentsLoaded()
    editor.goToNextChapter()
    await vi.waitFor(() => expect(review.reviewModeStatus.value).toBe('idle'))
  })
})
