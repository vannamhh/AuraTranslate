import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const mockInvoke = vi.fn()
const mockOpenChapterById = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))
vi.mock('../../src/panels/editorPanelState', () => ({
  openChapterById: (...args: unknown[]) => mockOpenChapterById(...args),
  requestCurrentEditorCaretPlacement: () => {},
}))

const mark = (segment_id: number, chapter_id: number) => ({
  segment_id,
  navigation_segment_id: segment_id,
  chapter_id,
  chapter_ord: chapter_id,
  chapter_title: null,
  source_text: '原文',
  target_text: `Bản dịch ${segment_id}`,
  is_retired: false,
  marked_at: '2026-10-16T00:00:00.000Z',
})

const THREE = [mark(11, 1), mark(12, 2), mark(13, 3)]

function counts(self_count: number, other_count: number, bilingual_import_count: number) {
  return { self_count, other_count, bilingual_import_count }
}

function runOf(originCounts: ReturnType<typeof counts>) {
  return {
    chapters: [
      {
        chapter_id: 1,
        chapter_ord: 1,
        chapter_title: 'Một',
        paragraphs: [{ segments: [{ id: 11, source_text: 'a', target_text: 'b', is_confirmed: true, is_marked: true }] }],
        segment_count: 1,
        translation_origin_counts: originCounts,
        images: [],
      },
    ],
    frontier: { kind: 'end-of-work', chapter: null },
    assets_dir: '',
  }
}

let stored = THREE

function answer(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
  if (cmd === 'list_reading_marks') return Promise.resolve(stored)
  if (cmd === 'unmark_reading_segment') {
    const before = stored.length
    stored = stored.filter((m) => m.segment_id !== args?.segmentId)
    return Promise.resolve(stored.length < before)
  }
  if (cmd === 'clear_reading_marks') {
    const removed = stored.length
    stored = []
    return Promise.resolve(removed)
  }
  if (cmd === 'list_chapters') return Promise.resolve([])
  return Promise.resolve(null)
}

const callsOf = (cmd: string) => mockInvoke.mock.calls.filter((c) => c[0] === cmd)

beforeEach(async () => {
  stored = THREE
  mockInvoke.mockReset()
  mockInvoke.mockImplementation(answer)
  mockOpenChapterById.mockReset()
  mockOpenChapterById.mockResolvedValue(true)
  const state = await import('../../src/modes/readingState')
  state.resetReading()
  state.resetReadingToc()
  state.cancelReadingMarksClear()
})

describe('unmark one', () => {
  it('removes that row, lowers the total, and clears the flag of the sentence in the run', async () => {
    const state = await import('../../src/modes/readingState')
    mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) =>
      cmd === 'read_reading_run' ? Promise.resolve(runOf(counts(1, 0, 0))) : answer(cmd, args),
    )
    await state.ensureReadingLoaded()
    await state.openReadingMarks()
    expect(state.readingMarksTotal.value).toBe(3)
    expect(state.readingRun.value?.chapters[0]?.paragraphs[0]?.segments[0]?.is_marked).toBe(true)

    state.setReadingMarksCursor(0)
    await state.unmarkCurrentReadingMark()

    expect(callsOf('unmark_reading_segment')).toEqual([['unmark_reading_segment', { segmentId: 11 }]])
    expect(state.readingMarks.value.map((m) => m.segment_id)).toEqual([12, 13])
    expect(state.readingMarksTotal.value).toBe(2)
    expect(state.readingRun.value?.chapters[0]?.paragraphs[0]?.segments[0]?.is_marked).toBe(false)
  })

  it('keeps the row and the error when Rust refuses', async () => {
    const state = await import('../../src/modes/readingState')
    await state.openReadingMarks()
    mockInvoke.mockImplementation((cmd: string) =>
      cmd === 'unmark_reading_segment'
        ? Promise.reject({ code: 'store.write_failed', message_key: 'err.store.write_failed', params: { store: 'project' }, retryable: true })
        : answer(cmd),
    )
    await state.unmarkCurrentReadingMark()
    expect(state.readingMarks.value).toHaveLength(3)
    expect(state.readingMarksError.value?.code).toBe('store.write_failed')
  })
})

describe('"Bỏ hết đánh dấu" asks once', () => {
  it('the first call only arms; Rust is not called', async () => {
    const state = await import('../../src/modes/readingState')
    await state.openReadingMarks()
    await state.clearAllReadingMarks()
    expect(state.readingMarksClearPending.value).toBe(true)
    expect(callsOf('clear_reading_marks')).toHaveLength(0)
    expect(state.readingMarks.value).toHaveLength(3)
  })

  it('cancelling disarms without a call and the marks stay', async () => {
    const state = await import('../../src/modes/readingState')
    await state.openReadingMarks()
    await state.clearAllReadingMarks()
    state.cancelReadingMarksClear()
    expect(state.readingMarksClearPending.value).toBe(false)
    expect(callsOf('clear_reading_marks')).toHaveLength(0)
    await state.clearAllReadingMarks()
    expect(state.readingMarksClearPending.value).toBe(true)
    expect(callsOf('clear_reading_marks')).toHaveLength(0)
  })

  it('closing the overlay or moving the cursor disarms', async () => {
    const state = await import('../../src/modes/readingState')
    await state.openReadingMarks()
    await state.clearAllReadingMarks()
    state.closeReadingMarks()
    expect(state.readingMarksClearPending.value).toBe(false)
    await state.openReadingMarks()
    await state.clearAllReadingMarks()
    state.setReadingMarksCursor(1)
    expect(state.readingMarksClearPending.value).toBe(false)
    expect(callsOf('clear_reading_marks')).toHaveLength(0)
  })

  it('the second call clears once: no marks, total 0, every flag in the run down', async () => {
    const state = await import('../../src/modes/readingState')
    mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) =>
      cmd === 'read_reading_run' ? Promise.resolve(runOf(counts(1, 0, 0))) : answer(cmd, args),
    )
    await state.ensureReadingLoaded()
    await state.openReadingMarks()
    await state.clearAllReadingMarks()
    await state.clearAllReadingMarks()
    expect(callsOf('clear_reading_marks')).toHaveLength(1)
    expect(state.readingMarks.value).toHaveLength(0)
    expect(state.readingMarksTotal.value).toBe(0)
    expect(state.readingRun.value?.chapters[0]?.paragraphs[0]?.segments[0]?.is_marked).toBe(false)
  })

  it('a store failure keeps the marks', async () => {
    const state = await import('../../src/modes/readingState')
    await state.openReadingMarks()
    mockInvoke.mockImplementation((cmd: string) =>
      cmd === 'clear_reading_marks'
        ? Promise.reject({ code: 'store.write_failed', message_key: 'err.store.write_failed', params: { store: 'project' }, retryable: true })
        : answer(cmd),
    )
    await state.clearAllReadingMarks()
    await state.clearAllReadingMarks()
    expect(state.readingMarks.value).toHaveLength(3)
    expect(state.readingMarksError.value?.code).toBe('store.write_failed')
  })

  it('in the overlay the button turns into "Bỏ hết 3 dấu? Xác nhận" and Esc turns it back without a call', async () => {
    const commands = await import('../../src/commands')
    const state = await import('../../src/modes/readingState')
    commands.installCommands({
      isMac: true,
      setMode: () => {},
      closeReadingMarks: state.closeReadingMarks,
      clearAllReadingMarks: () => void state.clearAllReadingMarks(),
      cancelReadingMarksClear: state.cancelReadingMarksClear,
    } as never)
    mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) =>
      cmd === 'read_reading_run' ? Promise.resolve(runOf(counts(1, 0, 0))) : answer(cmd, args),
    )
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const wrapper = mount(ReadingMode, { attachTo: document.body })
    await state.ensureReadingLoaded()
    await state.openReadingMarks()
    await wrapper.vm.$nextTick()

    const button = wrapper.get('[data-reading-marks-clear]')
    expect(button.text()).toBe('Bỏ hết đánh dấu')
    await button.trigger('click')
    await wrapper.vm.$nextTick()
    expect(button.text()).toBe('Bỏ hết 3 dấu? Xác nhận')
    expect(callsOf('clear_reading_marks')).toHaveLength(0)

    await wrapper.get('[role="dialog"]').trigger('keydown', { key: 'Escape' })
    await wrapper.vm.$nextTick()
    expect(button.text()).toBe('Bỏ hết đánh dấu')
    expect(wrapper.find('[role="dialog"]').exists()).toBe(true)
    expect(callsOf('clear_reading_marks')).toHaveLength(0)
    wrapper.unmount()
  })
})

describe('"Sửa lần lượt" walks the marks in the Workspace', () => {
  it('opens mark 1, then 2, then 3; the control disappears on the last and an extra call does nothing', async () => {
    const state = await import('../../src/modes/readingState')
    await state.openReadingMarks()
    await state.walkReadingMarks()
    expect(mockOpenChapterById).toHaveBeenLastCalledWith(1, 11)
    expect(state.readingMarksWalkProgress.value).toEqual({ next: 1, total: 3 })

    await state.walkNextReadingMark()
    expect(mockOpenChapterById).toHaveBeenLastCalledWith(2, 12)
    expect(state.readingMarksWalkProgress.value).toEqual({ next: 2, total: 3 })

    await state.walkNextReadingMark()
    expect(mockOpenChapterById).toHaveBeenLastCalledWith(3, 13)
    expect(state.readingMarksWalkProgress.value).toBeNull()

    await state.walkNextReadingMark()
    expect(mockOpenChapterById).toHaveBeenCalledTimes(3)
  })

  it('a single mark never shows the control', async () => {
    stored = [mark(11, 1)]
    const state = await import('../../src/modes/readingState')
    await state.openReadingMarks()
    await state.walkReadingMarks()
    expect(mockOpenChapterById).toHaveBeenCalledTimes(1)
    expect(state.readingMarksWalkProgress.value).toBeNull()
  })

  it('a failed first open arms nothing', async () => {
    mockOpenChapterById.mockResolvedValue(false)
    const state = await import('../../src/modes/readingState')
    await state.openReadingMarks()
    await state.walkReadingMarks()
    expect(state.readingMarksWalkProgress.value).toBeNull()
  })

  it('a failed next open keeps the position, and the retry opens the same mark', async () => {
    const state = await import('../../src/modes/readingState')
    await state.openReadingMarks()
    await state.walkReadingMarks()
    mockOpenChapterById.mockResolvedValueOnce(false)
    await state.walkNextReadingMark()
    expect(state.readingMarksWalkProgress.value).toEqual({ next: 1, total: 3 })
    await state.walkNextReadingMark()
    expect(mockOpenChapterById).toHaveBeenLastCalledWith(2, 12)
    expect(state.readingMarksWalkProgress.value).toEqual({ next: 2, total: 3 })
  })

  it('changing Work (resetReading) ends the walk, the walk own opens do not', async () => {
    const state = await import('../../src/modes/readingState')
    await state.openReadingMarks()
    await state.walkReadingMarks()
    state.resetReading({ preserveAnchor: true })
    expect(state.readingMarksWalkProgress.value).toEqual({ next: 1, total: 3 })
    state.resetReading()
    expect(state.readingMarksWalkProgress.value).toBeNull()
  })
})

describe('the Chapter origin line', () => {
  async function lineFor(originCounts: ReturnType<typeof counts>): Promise<string | null> {
    const state = await import('../../src/modes/readingState')
    mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) =>
      cmd === 'read_reading_run' ? Promise.resolve(runOf(originCounts)) : answer(cmd, args),
    )
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const wrapper = mount(ReadingMode, { attachTo: document.body })
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()
    const line = wrapper.find('[data-reading-chapter-origin]')
    const text = line.exists() ? line.text() : null
    wrapper.unmount()
    return text
  }

  it('names one origin when only one is present', async () => {
    expect(await lineFor(counts(2, 0, 0))).toContain('bản dịch của tôi')
  })

  it('says "hỗn hợp" for two or more origins', async () => {
    expect(await lineFor(counts(2, 1, 0))).toContain('hỗn hợp')
  })

  it('says "chưa có bản dịch" and names no origin when nothing is counted', async () => {
    const text = await lineFor(counts(0, 0, 0))
    expect(text).toContain('chưa có bản dịch')
    expect(text).not.toContain('của tôi')
  })
})

describe('the run wire', () => {
  it('a Chapter without translation_origin_counts rejects the whole run, never a zero stand-in', async () => {
    const { readReadingRun } = await import('../../src/config/reading')
    const run = runOf(counts(1, 0, 0))
    delete (run.chapters[0] as { translation_origin_counts?: unknown }).translation_origin_counts
    mockInvoke.mockImplementation((cmd: string) => (cmd === 'read_reading_run' ? Promise.resolve(run) : Promise.resolve(null)))
    const result = await readReadingRun()
    expect(result.run).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })
})

describe('the frontier "Xem n chỗ đã đánh dấu" button', () => {
  async function frontierButton(count: number | null, afterClear = false) {
    const state = await import('../../src/modes/readingState')
    mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === 'read_reading_run') return Promise.resolve(runOf(counts(1, 0, 0)))
      if (cmd === 'list_reading_marks' && count === null) return Promise.reject(new Error('unknown'))
      return answer(cmd, args)
    })
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const wrapper = mount(ReadingMode, { attachTo: document.body })
    await state.ensureReadingLoaded()
    await state.openReadingMarks()
    if (afterClear) {
      await state.clearAllReadingMarks()
      await state.clearAllReadingMarks()
    }
    state.closeReadingMarks()
    await wrapper.vm.$nextTick()
    const found = wrapper.find('[data-reading-frontier-marks]')
    const text = found.exists() ? found.text() : null
    wrapper.unmount()
    return { text, total: state.readingMarksTotal.value }
  }

  it('is shown while marks remain', async () => {
    const { text } = await frontierButton(3)
    expect(text).toContain('3')
  })

  it('is hidden once every mark is cleared', async () => {
    const { text, total } = await frontierButton(3, true)
    expect(total).toBe(0)
    expect(text).toBeNull()
  })

  it('is hidden while the total is unknown', async () => {
    const { text, total } = await frontierButton(null)
    expect(total).toBeNull()
    expect(text).toBeNull()
  })
})
