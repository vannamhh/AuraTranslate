/**
 * `config/reading.ts::isReadingRun` + `modes/readingState.ts::openFrontierInWorkspace` —
 * Story 5.12, FR120.
 *
 * 🔴 Đối chứng trung tâm: một hợp đồng *"`chapter` là `Some` khi và chỉ khi `kind ===
 * 'next-not-done'"* chỉ có nghĩa nếu có ai TỪ CHỐI cả hai chiều của trường hợp ngược —
 * không chỉ chấp nhận ca đúng. `isReadingRun` (qua `readReadingRun`) là chỗ đó.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const mockOpenChapterById = vi.fn()
vi.mock('../../src/panels/editorPanelState', () => ({
  openChapterById: (...args: unknown[]) => mockOpenChapterById(...args),
  requestCurrentEditorCaretPlacement: vi.fn(),
}))

const mockSetMode = vi.fn()
vi.mock('../../src/modes/modeState', () => ({
  setMode: (...args: unknown[]) => mockSetMode(...args),
}))

const WORK_NONE_OPEN_ERROR = {
  code: 'work.none_open',
  message_key: 'err.work.none_open',
  params: {},
  retryable: false,
}

function segment(overrides: Partial<{ id: number; source_text: string; target_text: string; is_confirmed: boolean; is_marked: boolean }> = {}) {
  return { id: 1, source_text: 'a', target_text: 'b', is_confirmed: true, is_marked: false, ...overrides }
}

function chapter(overrides: Partial<{ chapter_id: number; chapter_ord: number; chapter_title: string | null; segment_count: number }> = {}) {
  return {
    chapter_id: 1,
    chapter_ord: 1,
    chapter_title: null,
    paragraphs: [{ segments: [segment()] }],
    segment_count: 1,
    ...overrides,
  }
}

function frontierChapter(overrides: Partial<{ chapter_id: number; chapter_ord: number; chapter_title: string | null; status: string }> = {}) {
  return { chapter_id: 9, chapter_ord: 9, chapter_title: null, status: 'in_progress', ...overrides }
}

beforeEach(async () => {
  mockInvoke.mockReset()
  mockOpenChapterById.mockReset()
  mockSetMode.mockReset()
  const state = await import('../../src/modes/readingState')
  state.resetReading()
  state.resetReadingToc()
})

// ═════════════════════════════════════════════════════════════════════════════════
// Bốn hình dạng `ReadingRun` ⇒ bốn `readingStatusKind` — I/O Matrix của story.
// ═════════════════════════════════════════════════════════════════════════════════

describe('readReadingRun → readingStatusKind — bốn hình dạng ReadingRun', () => {
  it('dãy BA Chương + mốc "next-not-done" ⇒ "content"', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [chapter({ chapter_id: 1 }), chapter({ chapter_id: 2, chapter_ord: 2 }), chapter({ chapter_id: 3, chapter_ord: 3 })],
      frontier: { kind: 'next-not-done', chapter: frontierChapter({ chapter_id: 4, chapter_ord: 4 }) },
    })
    const state = await import('../../src/modes/readingState')
    await state.ensureReadingLoaded()

    expect(state.readingStatusKind.value).toBe('content')
    expect(state.readingRun.value?.chapters).toHaveLength(3)
    expect(state.readingRun.value?.frontier.chapter?.chapter_id).toBe(4)
  })

  it('dãy RỖNG + mốc "next-not-done" (Chạm biên ngay) ⇒ "frontier-only"', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [],
      frontier: { kind: 'next-not-done', chapter: frontierChapter({ chapter_id: 1, chapter_ord: 1 }) },
    })
    const state = await import('../../src/modes/readingState')
    await state.ensureReadingLoaded()

    expect(state.readingStatusKind.value).toBe('frontier-only')
    expect(state.readingRun.value?.chapters).toHaveLength(0)
  })

  it('mốc "end-of-work" (đã đọc hết Tác phẩm) ⇒ `chapter = null`, trạng thái vẫn đọc được từ nội dung', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [chapter()],
      frontier: { kind: 'end-of-work', chapter: null },
    })
    const state = await import('../../src/modes/readingState')
    await state.ensureReadingLoaded()

    expect(state.readingStatusKind.value).toBe('content')
    expect(state.readingRun.value?.frontier.kind).toBe('end-of-work')
    expect(state.readingRun.value?.frontier.chapter).toBeNull()
  })

  it('lỗi `work.none_open` ⇒ "no-work"', async () => {
    mockInvoke.mockRejectedValueOnce(WORK_NONE_OPEN_ERROR)
    const state = await import('../../src/modes/readingState')
    await state.ensureReadingLoaded()

    expect(state.readingStatusKind.value).toBe('no-work')
    expect(state.readingRun.value).toBeNull()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// `openFrontierInWorkspace()` — `frontier.chapter = null` KHÔNG ném, KHÔNG đổi chế độ.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/readingState::openFrontierInWorkspace', () => {
  it('`frontier.chapter = null` (end-of-work) ⇒ không ném, ghi chẩn đoán, không gọi `openChapterById`/`setMode`', async () => {
    mockInvoke.mockResolvedValueOnce({ chapters: [chapter()], frontier: { kind: 'end-of-work', chapter: null } })
    const state = await import('../../src/modes/readingState')
    await state.ensureReadingLoaded()

    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    await expect(state.openFrontierInWorkspace()).resolves.toBeUndefined()
    expect(errorSpy).toHaveBeenCalledOnce()
    errorSpy.mockRestore()

    expect(mockOpenChapterById).not.toHaveBeenCalled()
    expect(mockSetMode).not.toHaveBeenCalled()
  })

  it('`openChapterById` trả `false` (flush chặn) ⇒ không đổi chế độ, state đọc GIỮ NGUYÊN', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [],
      frontier: { kind: 'next-not-done', chapter: frontierChapter({ chapter_id: 9 }) },
    })
    mockOpenChapterById.mockResolvedValueOnce(false)
    const state = await import('../../src/modes/readingState')
    await state.ensureReadingLoaded()

    await state.openFrontierInWorkspace()

    expect(mockOpenChapterById).toHaveBeenCalledWith(9)
    expect(mockSetMode).not.toHaveBeenCalled()
    expect(state.readingRun.value).not.toBeNull()
  })

  it('`openChapterById` trả `true` ⇒ vứt TOÀN BỘ state đọc rồi chuyển Workspace', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [],
      frontier: { kind: 'next-not-done', chapter: frontierChapter({ chapter_id: 9 }) },
    })
    mockOpenChapterById.mockResolvedValueOnce(true)
    const state = await import('../../src/modes/readingState')
    await state.ensureReadingLoaded()

    await state.openFrontierInWorkspace()

    expect(mockOpenChapterById).toHaveBeenCalledWith(9)
    // State đọc bị vứt — lượt vào Chế độ đọc kế tiếp phải nạp lại từ con trỏ Chương mới.
    expect(state.readingRun.value).toBeNull()
    expect(state.readingHasLoaded.value).toBe(false)
    expect(mockSetMode).toHaveBeenCalledWith('workspace')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// `isReadingRun` (qua `readReadingRun`) — TỪ CHỐI cả hai chiều của bất biến
// *"`chapter` là `Some` khi và chỉ khi `kind === 'next-not-done'`"*.
// ═════════════════════════════════════════════════════════════════════════════════

describe('config/reading.ts::readReadingRun — bất biến kind ↔ chapter của ReadingFrontier', () => {
  it('`kind = "end-of-work"` kèm `chapter` NON-NULL ⇒ từ chối CẢ lượt trả về', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [],
      frontier: { kind: 'end-of-work', chapter: frontierChapter() },
    })
    const { readReadingRun } = await import('../../src/config/reading')
    const result = await readReadingRun()

    expect(result.run).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })

  it('`kind = "next-not-done"` kèm `chapter = null` ⇒ từ chối CẢ lượt trả về', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [],
      frontier: { kind: 'next-not-done', chapter: null },
    })
    const { readReadingRun } = await import('../../src/config/reading')
    const result = await readReadingRun()

    expect(result.run).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })

  it('một biến thể `kind` thứ BA (chưa ai ký) ⇒ từ chối', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [],
      frontier: { kind: 'somehow-else', chapter: null },
    })
    const { readReadingRun } = await import('../../src/config/reading')
    const result = await readReadingRun()

    expect(result.run).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })
})
