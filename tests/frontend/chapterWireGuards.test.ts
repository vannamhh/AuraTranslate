import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const FULL_ROW = {
  chapter_id: 1,
  ord: 1,
  title: 'Chuong Mot',
  status: 'done',
  segment_count: 5,
  translated_count: 4,
  unconfirmed_count: 1,
  unlinked_count: 0,
  in_translation_count: 5,
  updated_at: '2026-10-01T00:00:00Z',
  origin_author: null,
  origin_site_name: null,
  origin_url: null,
  origin_published_at: null,
}

beforeEach(() => {
  mockInvoke.mockReset()
  vi.spyOn(console, 'error').mockImplementation(() => undefined)
})

describe('config/chapter.ts ChapterRow guard', () => {
  it('accepts a row carrying the four 12-4 fields', async () => {
    mockInvoke.mockResolvedValue([FULL_ROW])
    const { listChapters } = await import('../../src/config/chapter')
    const out = await listChapters()
    expect(out.error).toBeNull()
    expect(out.chapters).toEqual([FULL_ROW])
  })

  it.each(['translated_count', 'unconfirmed_count', 'in_translation_count', 'updated_at'])(
    'rejects a row without %s instead of showing an empty list',
    async (field) => {
      const broken: Record<string, unknown> = { ...FULL_ROW }
      delete broken[field]
      mockInvoke.mockResolvedValue([broken])
      const { listChapters } = await import('../../src/config/chapter')
      const out = await listChapters()
      expect(out.chapters).toBeNull()
      expect(out.error).not.toBeNull()
    },
  )

  it('rejects a count sent as a string', async () => {
    mockInvoke.mockResolvedValue([{ ...FULL_ROW, unconfirmed_count: '1' }])
    const { listChapters } = await import('../../src/config/chapter')
    expect((await listChapters()).chapters).toBeNull()
  })

  it('mergeChapters sends chapterIds in camelCase and rejects a malformed rebuilt list', async () => {
    mockInvoke.mockResolvedValue([{ ...FULL_ROW, in_translation_count: undefined }])
    const { mergeChapters } = await import('../../src/config/chapter')
    const out = await mergeChapters([1, 2])
    expect(mockInvoke).toHaveBeenCalledWith('merge_chapters', { chapterIds: [1, 2] })
    expect(out.chapters).toBeNull()
    expect(out.error).not.toBeNull()
  })
})

describe('config/chapter.ts delete adapters', () => {
  it('deleteChapter sends chapterId in camelCase and rejects a malformed answer', async () => {
    mockInvoke.mockResolvedValue({ disposition: 'soft', work_removed: false, open_chapter_id: 'x', chapters: [] })
    const { deleteChapter } = await import('../../src/config/chapter')
    const out = await deleteChapter(7)
    expect(mockInvoke).toHaveBeenCalledWith('delete_chapter', { chapterId: 7 })
    expect(out.deleted).toBeNull()
    expect(out.error).not.toBeNull()
  })

  it('deleteChapter accepts a null open_chapter_id', async () => {
    mockInvoke.mockResolvedValue({ disposition: 'soft', work_removed: false, open_chapter_id: null, chapters: [] })
    const { deleteChapter } = await import('../../src/config/chapter')
    const out = await deleteChapter(7)
    expect(out.error).toBeNull()
    expect(out.deleted?.open_chapter_id).toBeNull()
  })

  it('previewChapterDelete sends chapterId and rejects an unknown disposition', async () => {
    mockInvoke.mockResolvedValue({ disposition: 'maybe', removes_work: false })
    const { previewChapterDelete } = await import('../../src/config/chapter')
    const out = await previewChapterDelete(7)
    expect(mockInvoke).toHaveBeenCalledWith('delete_chapter_preview', { chapterId: 7 })
    expect(out.preview).toBeNull()
    expect(out.error).not.toBeNull()
  })
})

describe('config/lifecycle.ts WorkLifecycle guard', () => {
  it('accepts status, override flag and derived status', async () => {
    mockInvoke.mockResolvedValue({ status: 'paused', status_is_override: true, derived_status: 'in_progress' })
    const { readWorkLifecycle } = await import('../../src/config/lifecycle')
    const out = await readWorkLifecycle()
    expect(out.error).toBeNull()
    expect(out.lifecycle?.derived_status).toBe('in_progress')
  })

  it('rejects a lifecycle without derived_status', async () => {
    mockInvoke.mockResolvedValue({ status: 'paused', status_is_override: true })
    const { readWorkLifecycle } = await import('../../src/config/lifecycle')
    const out = await readWorkLifecycle()
    expect(out.lifecycle).toBeNull()
    expect(out.error).not.toBeNull()
  })

  it('setChaptersStatus sends chapterIds and status', async () => {
    mockInvoke.mockResolvedValue({ status: 'paused', status_is_override: false, derived_status: 'paused' })
    const { setChaptersStatus } = await import('../../src/config/lifecycle')
    await setChaptersStatus([1, 2, 3], 'paused')
    expect(mockInvoke).toHaveBeenCalledWith('set_chapters_status', { chapterIds: [1, 2, 3], status: 'paused' })
  })
})
