import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { KeepAlive, defineComponent, h } from 'vue'

const mockInvoke = vi.fn()
const mockOpenChapterById = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))
vi.mock('../../src/panels/editorPanelState', () => ({
  openChapterById: (...args: unknown[]) => mockOpenChapterById(...args),
  requestCurrentEditorCaretPlacement: () => {},
}))

function chapter(chapterId: number, segmentId: number) {
  return {
    chapter_id: chapterId,
    chapter_ord: chapterId,
    chapter_title: `Chuong ${String(chapterId)}`,
    blocks: [{ kind: 'paragraph', segments: [{ id: segmentId, source_text: 'x', target_text: 'y', is_confirmed: true, is_marked: false }] }],
    segment_count: 1,
    translation_origin_counts: { self_count: 1, other_count: 0, bilingual_import_count: 0 },
    images: [],
  }
}

const RUN = { chapters: [chapter(1, 11), chapter(2, 21)], frontier: { kind: 'end-of-work', chapter: null }, assets_dir: '' }
const RUN_FROM_TWO = { chapters: [chapter(2, 21)], frontier: { kind: 'end-of-work', chapter: null }, assets_dir: '' }

function row(chapterId: number) {
  return {
    chapter_id: chapterId,
    ord: chapterId,
    title: `Chuong ${String(chapterId)}`,
    status: 'done',
    segment_count: 1,
    translated_count: 1,
    unconfirmed_count: 0,
    unlinked_count: 0,
    in_translation_count: 0,
    updated_at: '2026-10-01T00:00:00Z',
    origin_author: null,
    origin_site_name: null,
    origin_url: null,
    origin_published_at: null,
  }
}

let runReads = 0
function answerIpc(): void {
  runReads = 0
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'list_chapters') return Promise.resolve([row(1), row(2)])
    if (cmd === 'read_reading_run') {
      runReads += 1
      return Promise.resolve(runReads === 1 ? RUN : RUN_FROM_TWO)
    }
    return Promise.resolve(null)
  })
}

function rect(top: number, bottom: number): DOMRect {
  return { top, bottom, left: 0, right: 0, width: 0, height: bottom - top, x: 0, y: top, toJSON: () => ({}) }
}

let commandsInstalled = false
async function installTocCommandsOnce(): Promise<void> {
  if (commandsInstalled) return
  const commands = await import('../../src/commands')
  const state = await import('../../src/modes/readingState')
  commands.installCommands({
    isMac: true,
    setMode: () => {},
    closeReadingToc: state.closeTableOfContents,
    openCurrentReadingTocChapter: () => {
      void state.openCurrentTocChapter()
    },
  } as never)
  commandsInstalled = true
}

let wrapper: ReturnType<typeof mount> | null = null

beforeEach(async () => {
  mockInvoke.mockReset()
  mockOpenChapterById.mockReset()
  answerIpc()
  const state = await import('../../src/modes/readingState')
  state.resetReading()
  state.resetReadingToc()
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('modes/readingState — Mục lục mở ở Chương đang đọc', () => {
  it('câu đang nhắm thuộc Chương 2 ⇒ con trỏ Mục lục ở Chương 2', async () => {
    const state = await import('../../src/modes/readingState')
    await state.ensureReadingLoaded()
    state.setReadingAim(21)

    await state.openTableOfContents()

    expect(state.readingTocCursor.value).toBe(1)
  })

  it('Chương đang đọc không có trong danh sách ⇒ con trỏ về dòng đầu', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_chapters') return Promise.resolve([row(7), row(8)])
      if (cmd === 'read_reading_run') return Promise.resolve(RUN)
      return Promise.resolve(null)
    })
    const state = await import('../../src/modes/readingState')
    await state.ensureReadingLoaded()
    state.setReadingAim(21)

    await state.openTableOfContents()

    expect(state.readingTocCursor.value).toBe(0)
  })
})

describe('modes/ReadingMode.vue — Mục lục theo vị trí cuộn và bấm chuột', () => {
  async function mountLoaded() {
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    const Shell = defineComponent(() => () => h(KeepAlive, null, [h(ReadingMode)]))
    wrapper = mount(Shell, { attachTo: document.body })
    await state.ensureReadingLoaded()
    await flushPromises()
    return { state, mounted: wrapper }
  }

  it('đã cuộn qua Chương 1, không câu nào được nhắm ⇒ con trỏ Mục lục ở Chương 2', async () => {
    const { state, mounted } = await mountLoaded()
    const host = mounted.get('section.mode').element as HTMLElement
    vi.spyOn(host, 'getBoundingClientRect').mockReturnValue(rect(0, 600))
    const first = mounted.get('[data-reading-segment="11"]').element as HTMLElement
    const second = mounted.get('[data-reading-segment="21"]').element as HTMLElement
    vi.spyOn(first, 'getBoundingClientRect').mockReturnValue(rect(-400, -360))
    vi.spyOn(second, 'getBoundingClientRect').mockReturnValue(rect(20, 60))

    await state.openTableOfContents()

    expect(state.readingTocCursor.value).toBe(1)
  })

  it('câu được nhắm ở Chương 1 đã trôi khỏi màn hình ⇒ con trỏ Mục lục theo Chương đang thấy', async () => {
    const { state, mounted } = await mountLoaded()
    const host = mounted.get('section.mode').element as HTMLElement
    vi.spyOn(host, 'getBoundingClientRect').mockReturnValue(rect(0, 600))
    const first = mounted.get('[data-reading-segment="11"]').element as HTMLElement
    const second = mounted.get('[data-reading-segment="21"]').element as HTMLElement
    vi.spyOn(first, 'getBoundingClientRect').mockReturnValue(rect(-400, -360))
    vi.spyOn(second, 'getBoundingClientRect').mockReturnValue(rect(20, 60))
    state.setReadingAim(11)

    await state.openTableOfContents()

    expect(state.readingTocCursor.value).toBe(1)
  })

  it('bấm đúp một dòng ⇒ chỉ một lượt mở Chương', async () => {
    await installTocCommandsOnce()
    mockOpenChapterById.mockResolvedValue(true)
    const { state, mounted } = await mountLoaded()

    await state.openTableOfContents()
    await mounted.vm.$nextTick()
    const target = mounted.findAll('.toc-list button').at(1)
    await target?.trigger('click')
    await target?.trigger('click')
    await vi.waitFor(() => {
      expect(state.readingTocOpen.value).toBe(false)
    })

    expect(mockOpenChapterById).toHaveBeenCalledTimes(1)
  })

  it('bấm một dòng ⇒ mở đúng Chương đó, đóng Mục lục, cuộn về đầu trang', async () => {
    await installTocCommandsOnce()
    mockOpenChapterById.mockResolvedValue(true)
    const { state, mounted } = await mountLoaded()
    const host = mounted.get('section.mode').element as HTMLElement
    let scrollTop = 900
    Object.defineProperty(host, 'scrollTop', { configurable: true, get: () => scrollTop, set: (v: number) => { scrollTop = v } })

    await state.openTableOfContents()
    await mounted.vm.$nextTick()
    const rows = mounted.findAll('.toc-list button')
    expect(rows).toHaveLength(2)
    expect(scrollTop).toBe(900)
    await rows[1]?.trigger('pointerdown')
    await rows[1]?.trigger('click')
    await vi.waitFor(() => {
      expect(state.readingTocOpen.value).toBe(false)
    })
    await vi.waitFor(() => {
      expect(scrollTop).toBe(0)
    })

    expect(mockOpenChapterById).toHaveBeenCalledWith(2)
    expect(state.readingRun.value?.chapters[0]?.chapter_id).toBe(2)
  })
})
