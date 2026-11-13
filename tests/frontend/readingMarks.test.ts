/** Story 5.13 — adapter + state marker. Hình học/hover thật thuộc e2e WebKit. */
import { paragraphSegments } from './support/readingBlocks'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const mockInvoke = vi.fn()
const mockOpenChapterById = vi.fn()
const mockRequestCurrentEditorCaretPlacement = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/panels/editorPanelState')>()),
  openChapterById: (...args: unknown[]) => mockOpenChapterById(...args),
  requestCurrentEditorCaretPlacement: (...args: unknown[]) =>
    mockRequestCurrentEditorCaretPlacement(...args),
  flushEditorBeforeDiscreteWrite: async () => 'clean' as const,
  flushChapterPositionNow: async () => {},
}))
vi.mock('../../src/config/library', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/library')>()),
  openWork: async () => ({
    opened: { meta: { work_id: 'w-khac', genre: '', name: 'Tac pham khac', chapter_count: 1, source_lang: 'zh', target_lang: 'vi' }, folder: '/tmp/w-khac.atproj', chapter_id: 1 },
    error: null,
  }),
}))

const SEGMENT = {
  id: 12,
  source_text: '原文',
  target_text: 'Bản dịch',
  is_confirmed: true,
  is_marked: false,
}

// 🔵 THÊM Story 6.14 — `images`/`assets_dir`: `isReadingChapter`/`isReadingRun` đòi các
// trường này CÓ MẶT, nếu không CẢ RUN bị từ chối.
const RUN = {
  chapters: [
    {
      chapter_id: 4,
      chapter_ord: 2,
      chapter_title: 'Chương Hai',
      blocks: [{ kind: 'paragraph', segments: [SEGMENT] }],
      segment_count: 1,
      translation_origin_counts: { self_count: 0, other_count: 0, bilingual_import_count: 0 },
      images: [],
    },
  ],
  frontier: { kind: 'end-of-work', chapter: null },
  assets_dir: '',
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
    expect(mockInvoke.mock.calls.filter((c) => c[0] === 'mark_reading_segment')).toHaveLength(0)

    state.setReadingAim(12)
    mockInvoke.mockResolvedValueOnce(MARK)
    await state.markAimedReadingSegment()
    expect(mockInvoke).toHaveBeenCalledWith('mark_reading_segment', { segmentId: 12 })
    expect(paragraphSegments(state.readingRun.value?.chapters[0]?.blocks[0])[0]?.is_marked).toBe(true)
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

    expect(paragraphSegments(state.readingRun.value?.chapters[0]?.blocks[0])[0]?.is_marked).toBe(false)
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
    expect(paragraphSegments(state.readingRun.value?.chapters[0]?.blocks[0])[0]?.is_marked).toBe(false)
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

let readingCommandsInstalled = false
async function installReadingCommandsOnce(): Promise<void> {
  if (readingCommandsInstalled) return
  const commands = await import('../../src/commands')
  const state = await import('../../src/modes/readingState')
  commands.installCommands({
    isMac: true,
    setMode: () => {},
    closeReadingToc: state.closeTableOfContents,
    closeReadingMarks: state.closeReadingMarks,
    cancelReadingMarksClear: state.cancelReadingMarksClear,
  } as never)
  readingCommandsInstalled = true
}

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

  it('B giữ câu đầu vùng nhìn đứng yên khi ghi chú nguyên văn đổi chiều cao đoạn', async () => {
    mockInvoke.mockResolvedValue(undefined)
    mockInvoke.mockResolvedValueOnce(RUN)
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    const wrapper = mount(ReadingMode, { attachTo: document.body })
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const host = wrapper.get('section.mode').element as HTMLElement
    let scrollTop = 0
    Object.defineProperty(host, 'scrollTop', { configurable: true, get: () => scrollTop, set: (v: number) => { scrollTop = v } })
    // jsdom has no layout: the sentence sits 60px lower once a source note is rendered.
    const segment = wrapper.get('[data-reading-segment="12"]').element as HTMLElement
    vi.spyOn(segment, 'getBoundingClientRect').mockImplementation(() => {
      const top = document.querySelector('.source-note') === null ? 100 : 160
      return { top, bottom: top + 20, left: 0, right: 0, width: 0, height: 20, x: 0, y: top, toJSON: () => ({}) }
    })
    const wasBilingual = state.readingBilingual.value

    state.toggleBilingual()
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.source-note').exists()).toBe(!wasBilingual)
    expect(scrollTop).toBe(wasBilingual ? -60 : 60)
    state.toggleBilingual()
    await wrapper.vm.$nextTick()
    expect(scrollTop).toBe(0)
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

  it('chuột đặt aim khi chưa câu nào có focus; câu đang focus giữ aim khi chuột đi qua câu khác', async () => {
    const twoSegments = structuredClone(RUN)
    twoSegments.chapters[0].blocks = [{ kind: 'paragraph' as const, segments: [SEGMENT, { ...SEGMENT, id: 13 }] }]
    twoSegments.chapters[0].segment_count = 2
    mockInvoke.mockResolvedValueOnce(twoSegments)
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    const wrapper = mount(ReadingMode, { attachTo: document.body })
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const first = wrapper.get('[data-reading-segment="12"]')
    const second = wrapper.get('[data-reading-segment="13"]')
    await first.trigger('mouseenter')
    expect(state.readingAimedSegmentId.value).toBe(12)
    await first.trigger('mouseleave')
    expect(state.readingAimedSegmentId.value).toBeNull()

    ;(second.element as HTMLElement).focus()
    await wrapper.vm.$nextTick()
    expect(state.readingAimedSegmentId.value).toBe(13)
    await first.trigger('mouseenter')
    await first.trigger('mouseleave')
    expect(state.readingAimedSegmentId.value).toBe(13)
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
    const buttons = dialog.findAll('button').filter((b) => b.attributes('disabled') === undefined)
    const first = buttons[0]
    const last = buttons.at(-1)
    expect(first).toBeDefined()
    expect(last).toBeDefined()
    ;(last!.element as HTMLElement).focus()
    await last!.trigger('keydown', { key: 'Tab' })
    expect(document.activeElement).toBe(first!.element)
    wrapper.unmount()
  })

  it('the TOC and marks drawers close on a click outside the panel, not on a click inside it', async () => {
    mockInvoke.mockResolvedValueOnce(RUN)
    const state = await import('../../src/modes/readingState')
    await installReadingCommandsOnce()
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const wrapper = mount(ReadingMode, { attachTo: document.body })
    await state.ensureReadingLoaded()

    mockInvoke.mockResolvedValueOnce([MARK])
    await state.openReadingMarks()
    await wrapper.vm.$nextTick()
    await wrapper.get('.marks-panel').trigger('click')
    expect(state.readingMarksOpen.value).toBe(true)
    await wrapper.get('.marks-overlay').trigger('click')
    expect(state.readingMarksOpen.value).toBe(false)

    mockInvoke.mockResolvedValueOnce([])
    await state.openTableOfContents()
    await wrapper.vm.$nextTick()
    await wrapper.get('.toc-panel').trigger('click')
    expect(state.readingTocOpen.value).toBe(true)
    await wrapper.get('.toc-overlay').trigger('click')
    expect(state.readingTocOpen.value).toBe(false)
    wrapper.unmount()
  })

  async function mountWithMarks() {
    mockInvoke.mockResolvedValueOnce(RUN)
    const state = await import('../../src/modes/readingState')
    await installReadingCommandsOnce()
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const wrapper = mount(ReadingMode, { attachTo: document.body })
    await state.ensureReadingLoaded()
    mockInvoke.mockResolvedValueOnce([MARK])
    await state.openReadingMarks()
    await wrapper.vm.$nextTick()
    return { wrapper, state }
  }

  it('the Huỷ button shows only while clearing is armed and dispatches reading.marks_clear_cancel', async () => {
    const { wrapper, state } = await mountWithMarks()
    expect(wrapper.find('[data-reading-marks-clear-cancel]').exists()).toBe(false)
    await state.clearAllReadingMarks()
    await wrapper.vm.$nextTick()
    await wrapper.get('[data-reading-marks-clear-cancel]').trigger('click')
    expect(state.readingMarksClearPending.value).toBe(false)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-reading-marks-clear-cancel]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('focus returns to the marks panel when the focused Huỷ button disappears', async () => {
    const { wrapper, state } = await mountWithMarks()
    await state.clearAllReadingMarks()
    await wrapper.vm.$nextTick()
    const cancel = wrapper.get('[data-reading-marks-clear-cancel]').element as HTMLElement
    cancel.focus()
    expect(document.activeElement).toBe(cancel)
    await wrapper.get('[data-reading-marks-clear-cancel]').trigger('click')
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()
    expect(document.activeElement).toBe(wrapper.get('.marks-body').element)
    wrapper.unmount()
  })
})

describe('modes/readingState — the marks list follows the user into the Workspace', () => {
  const SECOND = { ...MARK, segment_id: 13, navigation_segment_id: 31, chapter_id: 5, chapter_ord: 3 }

  async function openList() {
    const state = await import('../../src/modes/readingState')
    const mode = await import('../../src/modes/modeState')
    mode.setMode('reading')
    mockInvoke.mockResolvedValueOnce([MARK, SECOND])
    await state.openReadingMarks()
    return { state, mode }
  }

  it('Sửa → keeps both rows, marks the opened row current and shows the panel in the Workspace', async () => {
    const { state, mode } = await openList()
    mockOpenChapterById.mockResolvedValueOnce(true)
    await state.openCurrentReadingMark()

    expect(mode.currentMode.value).toBe('workspace')
    expect(state.readingMarksPanelShown.value).toBe(true)
    expect(state.readingMarksOpen.value).toBe(false)
    expect(state.readingMarks.value.map((row) => row.segment_id)).toEqual([12, 13])
    expect(state.readingMarksCursor.value).toBe(0)
  })

  it('a second Sửa → from the Workspace opens the other Chapter, stays in the Workspace and keeps the list', async () => {
    const { state, mode } = await openList()
    mockOpenChapterById.mockResolvedValue(true)
    await state.openCurrentReadingMark()
    state.setReadingMarksCursor(1)
    await state.openCurrentReadingMark()

    expect(mockOpenChapterById).toHaveBeenLastCalledWith(5, 31)
    expect(mode.currentMode.value).toBe('workspace')
    expect(state.readingMarks.value).toHaveLength(2)
    expect(state.readingMarksCursor.value).toBe(1)
    expect(state.readingMarksPanelShown.value).toBe(true)
  })

  it('a failed open leaves the Workspace panel off and the modal list as it was', async () => {
    const { state, mode } = await openList()
    mockOpenChapterById.mockResolvedValueOnce(false)
    await state.openCurrentReadingMark()

    expect(mode.currentMode.value).toBe('reading')
    expect(state.readingMarksPanelShown.value).toBe(false)
    expect(state.readingMarksOpen.value).toBe(true)
    expect(state.readingMarks.value).toHaveLength(2)
  })

  it('Bỏ dấu in the Workspace drops the row and an empty list says so through marks_empty state', async () => {
    const { state } = await openList()
    mockOpenChapterById.mockResolvedValue(true)
    await state.openCurrentReadingMark()

    mockInvoke.mockResolvedValueOnce(true)
    await state.unmarkCurrentReadingMark()
    expect(state.readingMarks.value.map((row) => row.segment_id)).toEqual([13])
    mockInvoke.mockResolvedValueOnce(true)
    await state.unmarkCurrentReadingMark()
    expect(state.readingMarks.value).toEqual([])
    expect(state.readingMarksHaveLoaded.value).toBe(true)
    expect(state.readingMarksError.value).toBeNull()
  })

  it('the walk keeps the list and the current row follows the step', async () => {
    const { state } = await openList()
    mockOpenChapterById.mockResolvedValue(true)
    await state.walkReadingMarks()
    expect(state.readingMarksPanelShown.value).toBe(true)
    expect(state.readingMarksCursor.value).toBe(0)
    await state.walkNextReadingMark()
    expect(state.readingMarks.value).toHaveLength(2)
    expect(state.readingMarksCursor.value).toBe(1)
  })

  it('closing the panel and opening another Work each drop it', async () => {
    const { state, mode } = await openList()
    mockOpenChapterById.mockResolvedValue(true)

    await state.openCurrentReadingMark()
    state.closeReadingMarksPanel()
    expect(state.readingMarksPanelShown.value).toBe(false)
    expect(state.readingMarks.value).toEqual([])

    mode.setMode('reading')
    mockInvoke.mockResolvedValueOnce([MARK, SECOND])
    await state.openReadingMarks()
    await state.openCurrentReadingMark()
    expect(state.readingMarksPanelShown.value).toBe(true)
    const { openWorkById } = await import('../../src/modes/libraryChapters')
    mockInvoke.mockResolvedValue(null)
    await openWorkById('w-khac')
    expect(state.readingMarksPanelShown.value).toBe(false)
    expect(state.readingMarks.value).toEqual([])
  })

  it('the reading-mode deactivation after Sửa → keeps the list, and coming back to the reading mode drops the panel', async () => {
    const state = await import('../../src/modes/readingState')
    const mode = await import('../../src/modes/modeState')
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const { h, KeepAlive, defineComponent, nextTick } = await import('vue')
    const Empty = defineComponent({ render: () => h('div') })
    const Host = defineComponent({
      render: () => h(KeepAlive, null, [mode.currentMode.value === 'reading' ? h(ReadingMode) : h(Empty)]),
    })
    mode.setMode('reading')
    mockInvoke.mockResolvedValueOnce(RUN)
    const wrapper = mount(Host, { attachTo: document.body })
    await nextTick()
    await state.ensureReadingLoaded()
    mockInvoke.mockResolvedValueOnce([MARK, SECOND])
    await state.openReadingMarks()
    mockOpenChapterById.mockResolvedValue(true)
    await state.openCurrentReadingMark()
    await nextTick()
    await nextTick()

    expect(mode.currentMode.value).toBe('workspace')
    expect(state.readingMarks.value).toHaveLength(2)
    expect(state.readingMarksPanelShown.value).toBe(true)

    mode.setMode('reading')
    await nextTick()
    await nextTick()
    expect(state.readingMarksPanelShown.value).toBe(false)
    expect(state.readingMarks.value).toEqual([])
    wrapper.unmount()
  })
})
