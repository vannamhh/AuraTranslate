import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import { t } from '../../src/i18n'
import { isReadableChapterStatus } from '../../src/config/lifecycle'

const mockInvoke = vi.fn()
const mockOpenChapterById = vi.fn()
const mockRequestCurrentEditorCaretPlacement = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
  convertFileSrc: (path: string) => `mock-asset://${path}`,
}))
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/panels/editorPanelState')>()),
  openChapterById: (...args: unknown[]) => mockOpenChapterById(...args),
  requestCurrentEditorCaretPlacement: (...args: unknown[]) => mockRequestCurrentEditorCaretPlacement(...args),
  flushEditorBeforeDiscreteWrite: async () => 'clean' as const,
  flushChapterPositionNow: async () => {},
}))

const SOURCE_OF_TRANSLATED = 'NGUYEN-VAN-DA-DICH'

function paragraph(id: number) {
  return {
    kind: 'paragraph' as const,
    segments: [{ id, source_text: SOURCE_OF_TRANSLATED, target_text: 'Ban dich.', is_confirmed: true, is_marked: false }],
  }
}

function chapter(blocks: unknown[], overrides: Record<string, unknown> = {}) {
  return {
    chapter_id: 7,
    chapter_ord: 3,
    chapter_title: 'Chuong Ba',
    blocks,
    segment_count: 6,
    translation_origin_counts: { self_count: 0, other_count: 0, bilingual_import_count: 0 },
    images: [],
    ...overrides,
  }
}

function run(chapters: unknown[]) {
  return {
    chapters,
    frontier: { kind: 'end-of-work', chapter: null },
    assets_dir: '',
  }
}

const RUN_WITH_BLOCK = run([chapter([paragraph(1), { kind: 'untranslated', segment_ids: [21, 22, 23] }])])

let wrapper: VueWrapper | null = null
let commandsInstalled = false

async function installCommandsOnce(): Promise<void> {
  if (commandsInstalled) return
  const commands = await import('../../src/commands')
  const state = await import('../../src/modes/readingState')
  commands.installCommands({
    isMac: true,
    setMode: () => {},
    openUntranslatedReadingBlock: () => {
      void state.openUntranslatedReadingBlock()
    },
  } as never)
  commandsInstalled = true
}

async function mountReading(value: unknown) {
  mockInvoke.mockResolvedValueOnce(value)
  const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
  const state = await import('../../src/modes/readingState')
  wrapper = mount(ReadingMode, { attachTo: document.body })
  await state.ensureReadingLoaded()
  await wrapper.vm.$nextTick()
  return state
}

beforeEach(async () => {
  mockInvoke.mockReset()
  mockOpenChapterById.mockReset()
  mockRequestCurrentEditorCaretPlacement.mockReset()
  const state = await import('../../src/modes/readingState')
  state.resetReading()
  state.resetReadingToc()
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('ReadingMode — khối câu chưa dịch', () => {
  it('khối hiện đúng số câu của dãy', async () => {
    await mountReading(RUN_WITH_BLOCK)
    const button = wrapper!.get('[data-reading-untranslated]')
    expect(button.text()).toBe(t('mode.reading.untranslated_run', { count: '3' }))
    expect(button.attributes('data-reading-untranslated-count')).toBe('3')
    expect(button.attributes('data-reading-untranslated-first')).toBe('21')
  })

  it('Song ngữ bật: ghi chú nguyên văn chỉ nối câu của khối đoạn, khối chưa dịch không mang chữ nguyên văn', async () => {
    const state = await mountReading(RUN_WITH_BLOCK)
    if (!state.readingBilingual.value) state.toggleBilingual()
    await wrapper!.vm.$nextTick()
    const notes = wrapper!.findAll('[data-reading-source-note]')
    expect(notes).toHaveLength(1)
    expect(notes[0]!.text()).toBe(SOURCE_OF_TRANSLATED)
    expect(wrapper!.get('[data-reading-untranslated]').text()).not.toContain(SOURCE_OF_TRANSLATED)
    expect(wrapper!.html().split(SOURCE_OF_TRANSLATED)).toHaveLength(2)
  })

  it('bấm khối sau khi trỏ vào: mở Chương đó ở câu đầu dãy và chuyển sang Workspace', async () => {
    await installCommandsOnce()
    const mode = await import('../../src/modes/modeState')
    mode.setMode('reading')
    const state = await mountReading(RUN_WITH_BLOCK)
    mockOpenChapterById.mockResolvedValueOnce(true)

    const button = wrapper!.get('[data-reading-untranslated]')
    await button.trigger('mouseenter')
    await button.trigger('click')
    await vi.waitFor(() => expect(mockOpenChapterById).toHaveBeenCalledWith(7, 21))
    expect(mode.currentMode.value).toBe('workspace')
    expect(state.readingAnchorSegmentId.value).toBe(21)
    expect(mockRequestCurrentEditorCaretPlacement).toHaveBeenCalledOnce()
  })

  it('mousedown cũng đặt aim: bấm không có mouseenter trước (con trỏ đã nằm sẵn trên khối) vẫn mở đúng câu', async () => {
    await installCommandsOnce()
    const mode = await import('../../src/modes/modeState')
    mode.setMode('reading')
    await mountReading(RUN_WITH_BLOCK)
    mockOpenChapterById.mockResolvedValueOnce(true)

    const button = wrapper!.get('[data-reading-untranslated]')
    await button.trigger('mousedown')
    await button.trigger('click')
    await vi.waitFor(() => expect(mockOpenChapterById).toHaveBeenCalledWith(7, 21))
  })

  it('chặn flush (openChapterById false) ⇒ ở lại Chế độ đọc', async () => {
    const mode = await import('../../src/modes/modeState')
    mode.setMode('reading')
    const state = await mountReading(RUN_WITH_BLOCK)
    mockOpenChapterById.mockResolvedValueOnce(false)
    state.setReadingUntranslatedAim([21, 22])

    await state.openUntranslatedReadingBlock()
    expect(mockOpenChapterById).toHaveBeenCalledWith(7, 21)
    expect(mode.currentMode.value).toBe('reading')
    expect(mockRequestCurrentEditorCaretPlacement).not.toHaveBeenCalled()
  })

  it('không aim thì bấm là no-op', async () => {
    const state = await mountReading(RUN_WITH_BLOCK)
    await state.openUntranslatedReadingBlock()
    expect(mockOpenChapterById).not.toHaveBeenCalled()
  })

  it('reading.mark_aimed bỏ qua khối: aim khối không đặt aim câu và không phát IPC đánh dấu', async () => {
    const state = await mountReading(RUN_WITH_BLOCK)
    const button = wrapper!.get('[data-reading-untranslated]')
    await button.trigger('mouseenter')
    expect(state.readingAimedUntranslatedFirstId.value).toBe(21)
    expect(state.readingAimedSegmentId.value).toBeNull()

    mockInvoke.mockClear()
    await state.markAimedReadingSegment()
    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('ảnh neo vào một câu GIỮA dãy hiện ngay sau khối', async () => {
    const image = { asset_id: 5, file_name: 'a.jpg', source_url: null, after_segment_id: 22, alt_text: 'x', caption_text: null }
    await mountReading(run([chapter([{ kind: 'untranslated', segment_ids: [21, 22, 23] }], { images: [image] })]))
    const block = wrapper!.get('[data-reading-untranslated]').element.closest('.reading-paragraph')!
    const children = [...block.children].map((el) => el.tagName)
    expect(children).toEqual(['P', 'FIGURE'])
  })

  it('Chương chỉ có một khối chưa dịch vẫn là trạng thái "content"', async () => {
    const state = await mountReading(run([chapter([{ kind: 'untranslated', segment_ids: [1, 2] }], { segment_count: 2 })]))
    expect(state.readingStatusKind.value).toBe('content')
  })
})

describe('config/reading — validator khối', () => {
  async function readWith(blocks: unknown[]) {
    mockInvoke.mockResolvedValueOnce(run([chapter(blocks)]))
    const { readReadingRun } = await import('../../src/config/reading')
    return readReadingRun()
  }

  it.each([
    ['kind lạ', [{ kind: 'x', segments: [] }]],
    ['đoạn thiếu segments', [{ kind: 'paragraph' }]],
    ['khối chưa dịch rỗng', [{ kind: 'untranslated', segment_ids: [] }]],
    ['khối chưa dịch mang văn bản', [{ kind: 'untranslated', segment_ids: ['a'] }]],
  ])('từ chối %s', async (_name, blocks) => {
    const result = await readWith(blocks)
    expect(result.run).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })

  it('từ chối kind mốc cũ "next-not-done"', async () => {
    mockInvoke.mockResolvedValueOnce({
      ...run([chapter([paragraph(1)])]),
      frontier: { kind: 'next-not-done', chapter: { chapter_id: 9, chapter_ord: 9, chapter_title: null, status: 'not_started' } },
    })
    const { readReadingRun } = await import('../../src/config/reading')
    const result = await readReadingRun()
    expect(result.run).toBeNull()
  })

  it('nhận hai loại khối hợp lệ', async () => {
    const result = await readWith([paragraph(1), { kind: 'untranslated', segment_ids: [2] }])
    expect(result.error).toBeNull()
    expect(result.run?.chapters[0]?.blocks).toHaveLength(2)
  })
})

describe('config/lifecycle — isReadableChapterStatus', () => {
  it.each([
    ['done', true],
    ['in_progress', true],
    ['paused', true],
    ['not_started', false],
    ['finished', false],
  ])('%s ⇒ %s', (status, expected) => {
    expect(isReadableChapterStatus(status)).toBe(expected)
  })
})
