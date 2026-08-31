/** Story 5.13 — adapter + state marker. Hình học/hover thật thuộc e2e WebKit. */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const mockInvoke = vi.fn()
const mockOpenChapterById = vi.fn()
const mockRequestCurrentEditorCaretPlacement = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))
vi.mock('../../src/panels/editorPanelState', () => ({
  openChapterById: (...args: unknown[]) => mockOpenChapterById(...args),
  requestCurrentEditorCaretPlacement: (...args: unknown[]) =>
    mockRequestCurrentEditorCaretPlacement(...args),
}))

const SEGMENT = {
  id: 12,
  source_text: '原文',
  target_text: 'Bản dịch',
  is_confirmed: true,
  is_marked: false,
}

const RUN = {
  chapters: [
    {
      chapter_id: 4,
      chapter_ord: 2,
      chapter_title: 'Chương Hai',
      paragraphs: [{ segments: [SEGMENT] }],
      segment_count: 1,
    },
  ],
  frontier: { kind: 'end-of-work', chapter: null },
}

const MARK = {
  segment_id: 12,
  navigation_segment_id: 12,
  chapter_id: 4,
  chapter_ord: 2,
  chapter_title: 'Chương Hai',
  source_text: '原文',
  target_text: 'Bản dịch',
  is_retired: false,
  marked_at: '2026-08-31T00:00:00.000Z',
}

beforeEach(async () => {
  mockInvoke.mockReset()
  mockOpenChapterById.mockReset()
  mockRequestCurrentEditorCaretPlacement.mockReset()
  const state = await import('../../src/modes/readingState')
  state.resetReading()
  state.resetReadingToc()
})

describe('config/reading — marker runtime guards', () => {
  it('mark gửi `segmentId` camelCase và nhận ReadingMark snake_case', async () => {
    mockInvoke.mockResolvedValueOnce(MARK)
    const { markReadingSegment } = await import('../../src/config/reading')
    const result = await markReadingSegment(12)
    expect(mockInvoke).toHaveBeenCalledWith('mark_reading_segment', { segmentId: 12 })
    expect(result).toEqual({ mark: MARK, error: null })
  })

  it('marker hợp kiểu nhưng sai segment_id vẫn bị từ chối như payload hỏng', async () => {
    mockInvoke.mockResolvedValueOnce({ ...MARK, segment_id: 99 })
    const { markReadingSegment } = await import('../../src/config/reading')
    const result = await markReadingSegment(12)
    expect(result.mark).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })

  it('payload danh sách sai không bị đổi thành danh sách rỗng giả', async () => {
    mockInvoke.mockResolvedValueOnce([{ ...MARK, navigation_segment_id: null }])
    const { listReadingMarks } = await import('../../src/config/reading')
    const result = await listReadingMarks()
    expect(result.marks).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })
})

describe('modes/readingState — aim, mark và điều hướng exact segment', () => {
  it('M không aim là no-op; có aim thì chỉ đổi cờ sau khi Rust trả thành công', async () => {
    const state = await import('../../src/modes/readingState')
    mockInvoke.mockResolvedValueOnce(RUN)
    await state.ensureReadingLoaded()

    await state.markAimedReadingSegment()
    expect(mockInvoke).toHaveBeenCalledTimes(1)

    state.setReadingAim(12)
    mockInvoke.mockResolvedValueOnce(MARK)
    await state.markAimedReadingSegment()
    expect(mockInvoke).toHaveBeenLastCalledWith('mark_reading_segment', { segmentId: 12 })
    expect(state.readingRun.value?.chapters[0]?.paragraphs[0]?.segments[0]?.is_marked).toBe(true)
    expect(state.readingAimedSegmentId.value).toBe(12)
    expect(state.readingAnchorSegmentId.value).toBeNull()
  })

  it('lỗi ghi marker không tô state thành công giả và được giữ để UI báo', async () => {
    const state = await import('../../src/modes/readingState')
    mockInvoke.mockResolvedValueOnce(RUN)
    await state.ensureReadingLoaded()
    state.setReadingAim(12)
    mockInvoke.mockRejectedValueOnce({
      code: 'store.write_failed',
      message_key: 'err.store.write_failed',
      params: { store: 'project' },
      retryable: true,
    })

    await state.markAimedReadingSegment()

    expect(state.readingRun.value?.chapters[0]?.paragraphs[0]?.segments[0]?.is_marked).toBe(false)
    expect(state.readingMarkerError.value?.code).toBe('store.write_failed')
  })

  it('bỏ câu trả marker và danh sách cũ nếu Work đã reset trong lúc chờ IPC', async () => {
    const state = await import('../../src/modes/readingState')
    mockInvoke.mockResolvedValueOnce(RUN)
    await state.ensureReadingLoaded()
    state.setReadingAim(12)

    let resolveMark: (value: unknown) => void = () => {}
    mockInvoke.mockImplementationOnce(() => new Promise((resolve) => { resolveMark = resolve }))
    const staleMark = state.markAimedReadingSegment()
    state.resetReading()
    mockInvoke.mockResolvedValueOnce(RUN)
    await state.ensureReadingLoaded()
    resolveMark(MARK)
    await staleMark
    expect(state.readingRun.value?.chapters[0]?.paragraphs[0]?.segments[0]?.is_marked).toBe(false)
    expect(state.readingMarkerError.value).toBeNull()

    let resolveList: (value: unknown) => void = () => {}
    mockInvoke.mockImplementationOnce(() => new Promise((resolve) => { resolveList = resolve }))
    const staleList = state.openReadingMarks()
    state.resetReading()
    resolveList([MARK])
    await staleList
    expect(state.readingMarks.value).toEqual([])
    expect(state.readingMarksHaveLoaded.value).toBe(false)
  })

  it('neo cuộn đổi độc lập với aim và từ chối ID không thuộc lượt đọc', async () => {
    const state = await import('../../src/modes/readingState')
    mockInvoke.mockResolvedValueOnce(RUN)
    await state.ensureReadingLoaded()

    state.setReadingAnchor(12)
    expect(state.readingAnchorSegmentId.value).toBe(12)
    expect(state.readingAimedSegmentId.value).toBeNull()

    state.setReadingAnchor(999)
    expect(state.readingAnchorSegmentId.value).toBe(12)
  })

  it('Enter cục bộ mở đúng chapter/segment, chỉ đổi mode sau khi mở thành công', async () => {
    const state = await import('../../src/modes/readingState')
    const mode = await import('../../src/modes/modeState')
    mode.setMode('reading')
    mockInvoke.mockResolvedValueOnce(RUN)
    await state.ensureReadingLoaded()
    state.setReadingAim(12)

    mockOpenChapterById.mockResolvedValueOnce(false)
    await state.openAimedReadingSegment()
    expect(mode.currentMode.value).toBe('reading')
    expect(mockRequestCurrentEditorCaretPlacement).not.toHaveBeenCalled()

    mockOpenChapterById.mockResolvedValueOnce(true)
    await state.openAimedReadingSegment()
    expect(mockOpenChapterById).toHaveBeenLastCalledWith(4, 12)
    expect(mode.currentMode.value).toBe('workspace')
    expect(state.readingAnchorSegmentId.value).toBe(12)
    expect(mockRequestCurrentEditorCaretPlacement).toHaveBeenCalledOnce()
  })

  it('chọn marker retired dùng navigation_segment_id sống, không segment_id gốc', async () => {
    const state = await import('../../src/modes/readingState')
    const retired = { ...MARK, segment_id: 12, navigation_segment_id: 99, is_retired: true }
    mockInvoke.mockResolvedValueOnce([retired])
    await state.openReadingMarks()
    mockOpenChapterById.mockResolvedValueOnce(true)
    await state.openCurrentReadingMark()
    expect(mockOpenChapterById).toHaveBeenCalledWith(4, 99)
    expect(state.readingAnchorSegmentId.value).toBe(99)
    expect(mockRequestCurrentEditorCaretPlacement).toHaveBeenCalledOnce()
  })

  it('con trỏ marker đi trước/sau có biên và Open dùng đúng mục đang chọn', async () => {
    const state = await import('../../src/modes/readingState')
    const second = { ...MARK, segment_id: 13, navigation_segment_id: 31, chapter_id: 5 }
    mockInvoke.mockResolvedValueOnce([MARK, second])
    await state.openReadingMarks()

    state.prevReadingMark()
    expect(state.readingMarksCursor.value).toBe(0)
    state.nextReadingMark()
    state.nextReadingMark()
    expect(state.readingMarksCursor.value).toBe(1)
    mockOpenChapterById.mockResolvedValueOnce(true)
    await state.openCurrentReadingMark()
    expect(mockOpenChapterById).toHaveBeenCalledWith(5, 31)
  })
})

describe('modes/ReadingMode.vue — wrapper focus và affordance', () => {
  it('mỗi câu là một tab stop; nút marker không chen thêm một tab stop', async () => {
    mockInvoke.mockResolvedValueOnce(RUN)
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    const wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const segment = wrapper.get('[data-reading-segment="12"]')
    expect(segment.attributes('tabindex')).toBe('0')
    const affordance = segment.get('.mark-affordance')
    expect(affordance.attributes('tabindex')).toBe('-1')
    wrapper.unmount()
  })

  it('aim không tự cuộn và nút đang focus không bị thay node sau lượt mark', async () => {
    const scrollIntoView = vi.fn()
    Object.defineProperty(HTMLElement.prototype, 'scrollIntoView', {
      configurable: true,
      value: scrollIntoView,
    })
    mockInvoke.mockResolvedValueOnce(RUN)
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    const wrapper = mount(ReadingMode, { attachTo: document.body })
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const segment = wrapper.get('[data-reading-segment="12"]')
    await segment.trigger('mouseenter')
    await wrapper.vm.$nextTick()
    expect(state.readingAnchorSegmentId.value).toBeNull()
    expect(scrollIntoView).not.toHaveBeenCalled()

    const before = segment.get('.mark-affordance').element
    ;(before as HTMLElement).focus()
    mockInvoke.mockResolvedValueOnce(MARK)
    await state.markAimedReadingSegment()
    await wrapper.vm.$nextTick()
    const after = segment.get('.mark-affordance').element
    expect(after).toBe(before)
    expect(document.activeElement).toBe(before)
    expect(after.getAttribute('aria-disabled')).toBe('true')
    wrapper.unmount()
  })

  it('overlay marker có tên và giữ Tab ở trong chuỗi nút của nó', async () => {
    mockInvoke.mockResolvedValueOnce(RUN)
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    const wrapper = mount(ReadingMode, { attachTo: document.body })
    await state.ensureReadingLoaded()
    mockInvoke.mockResolvedValueOnce([MARK])
    await state.openReadingMarks()
    await wrapper.vm.$nextTick()

    const dialog = wrapper.get('[role="dialog"]')
    expect(dialog.attributes('aria-labelledby')).toBe('reading-marks-heading')
    const buttons = dialog.findAll('.toc-actions .btn')
    const first = buttons[0]
    const last = buttons.at(-1)
    expect(first).toBeDefined()
    expect(last).toBeDefined()
    ;(last!.element as HTMLElement).focus()
    await last!.trigger('keydown', { key: 'Tab' })
    expect(document.activeElement).toBe(first!.element)
    wrapper.unmount()
  })
})
