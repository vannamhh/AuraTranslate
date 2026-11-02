/**
 * Status bar, 12.7: permanent info (position, confirmed counter, Glossary/TM counts,
 * narrow lookup line) sits OUTSIDE the `role="status"` live region, which carries only the notice chain.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

const reviewOpen = vi.hoisted(() => ({ value: false }))
const tmCount = vi.fn<() => Promise<{ total: number; mine: number; others: number } | null>>()
const glossaryCount = vi.fn<() => Promise<number | null>>()

async function confirmOk(segmentId: number) {
  return { outcome: { segment_id: segmentId, status: 'confirmed', version_created: true, chapter_started: false }, error: null }
}

const CHAPTERS = [
  { chapter_id: 7, ord: 47, title: null, status: 'draft', segment_count: 3, translated_count: 2, unconfirmed_count: 2, in_translation_count: 3, updated_at: '2026-10-14T00:00:00Z', origin_author: null, origin_site_name: null, origin_url: null, origin_published_at: null },
]
let chaptersOnDisk: unknown[] = CHAPTERS
let omittedSegmentId: number | null = null
const setChapterStatusWire = vi.fn<(args: unknown) => Promise<unknown>>()
const omitWire = vi.fn<(...args: unknown[]) => Promise<unknown>>()
const paragraphEndWire = vi.fn<(...args: unknown[]) => Promise<unknown>>()

vi.mock('../../src/config/segment', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/segment')>()),
  readOpenChapterSegments: async () => {
    const result = await readFixture()
    if (omittedSegmentId === null || result.loaded === null) return result
    const segments = result.loaded.segments.map((s) => (s.id === omittedSegmentId ? { ...s, is_omitted: true } : s))
    return { ...result, loaded: { ...result.loaded, segments } }
  },
  saveSegmentTargets: recordSave,
  confirmSegment: confirmOk,
  setSegmentOmitted: (...args: unknown[]) => omitWire(...args),
  setSegmentParagraphEnd: (...args: unknown[]) => paragraphEndWire(...args),
}))
vi.mock('../../src/config/chapter', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/chapter')>()),
  listChapters: async () => ({ chapters: chaptersOnDisk, error: null }),
}))
vi.mock('../../src/reviewModeState', async (importOriginal) => {
  const { computed, ref } = await import('vue')
  const actual = await importOriginal<typeof import('../../src/reviewModeState')>()
  const open = ref(reviewOpen.value)
  Object.defineProperty(reviewOpen, 'value', { get: () => open.value, set: (v: boolean) => (open.value = v), configurable: true })
  return { ...actual, reviewModeIsOpen: computed(() => open.value) }
})
vi.mock('../../src/config/tm', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/tm')>()),
  tmCountPairs: () => tmCount(),
}))
vi.mock('../../src/config/glossary', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/glossary')>()),
  countGlossaryEntries: () => glossaryCount(),
}))
vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => path,
  invoke: vi.fn(async (cmd: string, args?: unknown) => {
    if (cmd === 'lookup_dictionary') {
      return {
        grouped: {
          route: 'zh',
          branch: 'exact_btree',
          groups: [
            {
              layer: 'layer-a',
              source: { code: 'src-a', display_name: 'Nguồn A' },
              entries: [{ entry_id: 1, source_code: 'src-a', lang: 'zh', headword: '測試', headword_simp: null, reading: null }],
              total_entries: null,
            },
          ],
          skipped: [],
          truncated_layers: [],
          hidden_sources: [],
          layers_loaded: true,
        },
        senses_by_layer: {
          'layer-a': [{ entry_id: 1, sense_id: 1, pos: null, pos_lang: null, pos_is_foreign: false, gloss: 'thử nghiệm', note: null, ord: 1, examples: [] }],
        },
        query_truncated: false,
        senses_failed: [],
      }
    }
    if (cmd === 'put_config') return null
    if (cmd === 'set_chapter_status') return setChapterStatusWire(args)
    throw new Error(`statusBarWorkspace.test.ts: unexpected invoke "${cmd}"`)
  }),
}))

async function freshBar(extraDeps: Record<string, unknown> = {}) {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const chapters = await import('../../src/modes/libraryChapters')
  const commands = await import('../../src/commands')
  const works = await import('../../src/modes/libraryWorks')
  if (commands.commandRegistry.list().length === 0) {
    commands.installCommands({
      isMac: true,
      setMode: () => {},
      setOpenChapterDone: () => void works.setOpenChapterStatus(),
      dismissChapterDoneSuggestion: state.dismissChapterDoneSuggestion,
      ...extraDeps,
    })
  }
  ;(await import('../../src/modes/modeState')).setMode('workspace')
  const StatusBar = (await import('../../src/StatusBar.vue')).default
  await chapters.loadChapters()
  await state.ensureSegmentsLoaded()
  const wrapper = mount(StatusBar, { attachTo: document.body })
  await flushPromises()
  return { state, wrapper, commands, works, chapters }
}

function assignChord(shortcuts: typeof import('../../src/config/shortcutsState'), id: string, code: string): void {
  const row = document.createElement('div')
  row.setAttribute('data-command-id', id)
  const cell = document.createElement('span')
  cell.setAttribute('data-key-cell', '')
  cell.tabIndex = 0
  row.appendChild(cell)
  document.body.appendChild(row)
  row.addEventListener('focusin', shortcuts.aimRowFrom)
  cell.focus()
  shortcuts.captureShortcut()
  if (!shortcuts.handleCaptureKey({ code, metaKey: true, altKey: true, shiftKey: true })) throw new Error('capture refused')
  if (shortcuts.shortcutNotice.value !== null) throw new Error(`capture notice: ${shortcuts.shortcutNotice.value.key}`)
}

const info = (w: { find: (s: string) => { exists: () => boolean; text: () => string } }, attr: string) =>
  w.find(`[${attr}]`).exists() ? w.find(`[${attr}]`).text() : null

beforeEach(() => {
  document.body.innerHTML = ''
  resetRecorder()
  tmCount.mockReset().mockResolvedValue({ total: 12, mine: 9, others: 3 })
  glossaryCount.mockReset().mockResolvedValue(7)
  chaptersOnDisk = CHAPTERS
  omittedSegmentId = null
  reviewOpen.value = false
  setChapterStatusWire.mockReset().mockResolvedValue({ status: null, status_is_override: false, derived_status: 'in_progress' })
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('thanh trạng thái — thông tin thường trực', () => {
  it('con trỏ ở câu 2 trên 3: "Chương 47 · câu 2/3" và "1/3 đã xác nhận"', async () => {
    const { state, wrapper } = await freshBar()
    state.setEditorCaret(12)
    await flushPromises()

    expect(info(wrapper, 'data-status-position')).toBe('Chương 47 · câu 2/3')
    expect(info(wrapper, 'data-status-confirmed')).toBe('1/3 đã xác nhận')
    wrapper.unmount()
  })

  it('câu đã cắt bỏ không vào tử số lẫn mẫu số của bộ đếm, nhưng vẫn là một hàng của vị trí', async () => {
    omittedSegmentId = 12
    const { state, wrapper } = await freshBar()
    state.setEditorCaret(12)
    await flushPromises()

    expect(info(wrapper, 'data-status-confirmed')).toBe('1/2 đã xác nhận')
    expect(info(wrapper, 'data-status-position')).toBe('Chương 47 · câu 2/3')
    wrapper.unmount()
  })

  it('Chương chưa có hàng trong danh sách ⇒ chỉ "Câu 2/3", không "Chương 0" hay "Chương undefined"', async () => {
    chaptersOnDisk = []
    const { state, wrapper } = await freshBar()
    state.setEditorCaret(12)
    await flushPromises()

    expect(info(wrapper, 'data-status-position')).toBe('Câu 2/3')
    wrapper.unmount()
  })

  it('chưa có con trỏ nào ⇒ ẩn vị trí, không "câu 0/3"', async () => {
    const { wrapper } = await freshBar()
    expect(info(wrapper, 'data-status-position')).toBeNull()
    wrapper.unmount()
  })

  it('đổi con trỏ đổi vị trí nhưng KHÔNG đổi vùng role="status"; vùng đó là phần tử duy nhất mang role', async () => {
    const { state, wrapper } = await freshBar()
    state.setEditorCaret(11)
    await flushPromises()
    const live = (): string => wrapper.get('[data-status-live]').text()
    const before = live()
    const positionBefore = info(wrapper, 'data-status-position')

    state.setEditorCaret(13)
    await flushPromises()

    expect(info(wrapper, 'data-status-position')).not.toBe(positionBefore)
    expect(live()).toBe(before)
    const root = wrapper.element as unknown as Element
    const roles = Array.from(root.querySelectorAll('[role="status"]'))
    expect(roles).toHaveLength(1)
    expect(roles[0].hasAttribute('data-status-live')).toBe(true)
    expect(root.getAttribute('role')).toBeNull()
    for (const attr of ['data-status-position', 'data-status-confirmed', 'data-status-glossary', 'data-status-tm']) {
      expect(roles[0].querySelector(`[${attr}]`), `${attr} nằm trong vùng sống`).toBeNull()
    }
    wrapper.unmount()
  })

  it('thanh không còn câu gợi ý chuột phải và dòng tra cứu ngoại tuyến', async () => {
    const { wrapper } = await freshBar()

    const text = wrapper.text()
    expect(text).toContain('Glossary')
    expect(text).not.toContain('Chuột phải')
    expect(text).not.toContain('Tra cứu ngoại tuyến')
    wrapper.unmount()
  })

  it('lượt đổi cờ câu bị từ chối hiện lý do ở thanh trạng thái; không có cầu IPC thì hiện câu chung', async () => {
    const { wrapper } = await freshBar()
    const state = await import('../../src/panels/editorPanelState')
    const live = (): string => wrapper.find('[data-status-live]').text()

    state.setEditorCaret(11)
    omitWire.mockResolvedValueOnce({ outcome: null, error: null })
    expect(await state.setCurrentSegmentOmitted(true)).toBe('refused')
    await flushPromises()
    expect(live()).toContain('Chưa đổi được cờ của câu này')

    paragraphEndWire.mockResolvedValueOnce({
      outcome: null,
      error: { code: 'segment.not_found', message_key: 'err.segment.not_found', params: { segment_id: '11' } },
    })
    expect(await state.setCurrentSegmentParagraphEnd(true)).toBe('refused')
    await flushPromises()
    expect(live()).toContain('Không tìm thấy câu số 11')
    wrapper.unmount()
  })

  it('Tách câu khi chưa có điểm cắt nói phải giữ phím nào, theo nền tảng', async () => {
    const { wrapper } = await freshBar()
    const state = await import('../../src/panels/editorPanelState')

    expect(await state.splitCurrentSegment()).toBe('no-cut')
    await flushPromises()

    expect(wrapper.find('[data-status-live]').text()).toMatch(/giữ (⌘|Ctrl) và bấm vào cột nguyên văn/)
    wrapper.unmount()
  })
})

describe('thanh trạng thái — số Glossary và TM', () => {
  it('hiện số từ hai lệnh đếm khi bật; lệnh trả null ⇒ ẩn mục đó, không "0"', async () => {
    tmCount.mockResolvedValue(null)
    const { wrapper } = await freshBar()

    expect(info(wrapper, 'data-status-glossary')).toBe('Glossary 7 mục')
    expect(info(wrapper, 'data-status-tm')).toBeNull()
    expect(wrapper.text()).not.toMatch(/TM: 0 cặp|Glossary 0 mục/)
    wrapper.unmount()
  })

  it('0 thật vẫn hiện là 0 (khác "chưa biết")', async () => {
    tmCount.mockResolvedValue({ total: 0, mine: 0, others: 0 })
    const { wrapper } = await freshBar()
    expect(info(wrapper, 'data-status-tm')).toBe('TM 0 · 65%')
    expect(wrapper.get('[data-status-tm]').attributes('title')).toBe('TM: 0 cặp · 0 của tôi · 0 của người khác · ngưỡng gợi ý 65%')
    wrapper.unmount()
  })

  it('xác nhận câu thành công làm tươi bộ đếm xác nhận và số TM', async () => {
    const { state, wrapper } = await freshBar()
    state.setEditorCaret(12)
    await flushPromises()
    expect(info(wrapper, 'data-status-confirmed')).toBe('1/3 đã xác nhận')
    expect(info(wrapper, 'data-status-tm')).toBe('TM 12 · 65%')
    expect(wrapper.get('[data-status-tm]').attributes('title')).toBe('TM: 12 cặp · 9 của tôi · 3 của người khác · ngưỡng gợi ý 65%')
    const callsBefore = tmCount.mock.calls.length
    tmCount.mockResolvedValue({ total: 13, mine: 10, others: 3 })

    await state.confirmCurrentSegment()
    await flushPromises()

    expect(tmCount.mock.calls.length).toBeGreaterThan(callsBefore)
    expect(info(wrapper, 'data-status-tm')).toBe('TM 13 · 65%')
    expect(wrapper.get('[data-status-tm]').attributes('title')).toBe('TM: 13 cặp · 10 của tôi · 3 của người khác · ngưỡng gợi ý 65%')
    expect(info(wrapper, 'data-status-confirmed')).toBe('2/3 đã xác nhận')
    wrapper.unmount()
  })

  it('the threshold follows a save in Settings, not the value read at startup', async () => {
    const { state, wrapper } = await freshBar()
    state.setEditorCaret(12)
    await flushPromises()
    const settings = await import('../../src/tmSettingsState')
    settings.tmSettingsThresholdInput.value = '80'
    await settings.saveTmSettings()
    await flushPromises()
    expect(info(wrapper, 'data-status-tm')).toBe('TM 12 · 80%')
    expect(wrapper.get('[data-status-tm]').attributes('title')).toBe('TM: 12 cặp · 9 của tôi · 3 của người khác · ngưỡng gợi ý 80%')
    wrapper.unmount()
  })
})

describe('thanh trạng thái — tầng bố cục', () => {
  it('mọi tầng: thanh không còn câu giải thích auto-save', async () => {
    const { wrapper } = await freshBar()
    const layout = await import('../../src/layout/lookupDrawerState')

    for (const tier of ['full', 'short', 'narrow'] as const) {
      layout.syncLayoutTier(tier)
      await flushPromises()
      expect(wrapper.find('[data-status-autosave-hint]').exists(), tier).toBe(false)
    }
    wrapper.unmount()
  })

  it('màn rất hẹp: dòng nghĩa đầu + phím mở + nút ngăn kéo; không Glossary/TM/auto-save', async () => {
    const { wrapper } = await freshBar()
    const layout = await import('../../src/layout/lookupDrawerState')
    const lookup = await import('../../src/panels/lookupPanelState')
    await lookup.runLookup('測試')

    const shortcuts = await import('../../src/config/shortcutsState')
    assignChord(shortcuts, 'layout.lookup_drawer_open', 'KeyY')
    layout.syncLayoutTier('narrow')
    await flushPromises()

    expect(info(wrapper, 'data-status-lookup-line')).toBe('thử nghiệm')
    expect(info(wrapper, 'data-status-lookup-chord')).toBe(
      shortcuts.shortcutRows.value.find((r) => r.id === 'layout.lookup_drawer_open')?.display.join(' / '),
    )
    expect(wrapper.find('[data-lookup-drawer-open]').exists()).toBe(true)
    for (const attr of ['data-status-glossary', 'data-status-tm']) {
      expect(wrapper.find(`[${attr}]`).exists(), attr).toBe(false)
    }
    wrapper.unmount()
  })

  it('màn rất hẹp mà chưa tra gì (hoặc tra trượt) ⇒ chỉ có nút, không dòng nghĩa', async () => {
    const { wrapper } = await freshBar()
    const layout = await import('../../src/layout/lookupDrawerState')
    layout.syncLayoutTier('unsupported')
    await flushPromises()

    expect(wrapper.find('[data-status-lookup-line]').exists()).toBe(false)
    expect(wrapper.find('[data-lookup-drawer-open]').exists()).toBe(true)
    wrapper.unmount()
  })
})

describe('thanh trạng thái — nút đặt Chương Đã xong', () => {
  const doneButton = (w: { find: (s: string) => { exists: () => boolean; attributes: (n: string) => string | undefined; text: () => string; trigger: (e: string) => Promise<void> } }) =>
    w.find('[data-chapter-done]')

  it('bấm nút gửi set_chapter_status done cho Chương đang mở rồi tải lại danh sách Chương', async () => {
    const { wrapper, chapters } = await freshBar()
    expect(doneButton(wrapper).attributes('disabled')).toBeUndefined()
    expect(doneButton(wrapper).text()).toBe('Đặt Chương là Đã xong')

    chaptersOnDisk = [{ ...(CHAPTERS[0] as object), status: 'done' }]
    await doneButton(wrapper).trigger('click')
    await flushPromises()

    expect(setChapterStatusWire).toHaveBeenCalledTimes(1)
    expect(setChapterStatusWire).toHaveBeenCalledWith({ chapterId: 7, status: 'done' })
    expect(chapters.libraryChapters.value[0]?.status).toBe('done')
    expect(doneButton(wrapper).attributes('disabled')).toBeDefined()
    expect(doneButton(wrapper).text()).toBe('Chương đã xong')
    wrapper.unmount()
  })

  it('Chương đã xong thì nút khoá và không gọi IPC', async () => {
    chaptersOnDisk = [{ ...(CHAPTERS[0] as object), status: 'done' }]
    const { wrapper } = await freshBar()
    expect(doneButton(wrapper).attributes('disabled')).toBeDefined()
    await doneButton(wrapper).trigger('click')
    expect(setChapterStatusWire).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('lỗi hiện ngay ở Workspace, nút bật lại và danh sách Chương không đổi', async () => {
    setChapterStatusWire.mockRejectedValue({ code: 'work.create_failed', message_key: 'err.work.create_failed', params: {}, retryable: false })
    const { wrapper, chapters } = await freshBar()

    await doneButton(wrapper).trigger('click')
    await flushPromises()

    expect(wrapper.find('[data-status-chapter-done-error]').exists()).toBe(true)
    expect(wrapper.get('[data-status-chapter-done-error]').text()).not.toBe('')
    expect(doneButton(wrapper).attributes('disabled')).toBeUndefined()
    expect(chapters.libraryChapters.value[0]?.status).toBe('draft')
    wrapper.unmount()
  })

  it('thành công sau lỗi xoá câu lỗi', async () => {
    setChapterStatusWire.mockRejectedValueOnce({ code: 'work.create_failed', message_key: 'err.work.create_failed', params: {}, retryable: false })
    const { wrapper } = await freshBar()
    await doneButton(wrapper).trigger('click')
    await flushPromises()
    expect(wrapper.find('[data-status-chapter-done-error]').exists()).toBe(true)

    await doneButton(wrapper).trigger('click')
    await flushPromises()
    expect(wrapper.find('[data-status-chapter-done-error]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('gợi ý hiện khi xác nhận câu cuối, ẩn khi Chương đã xong, và đặt xong thì tắt', async () => {
    omittedSegmentId = 13
    const { state, wrapper, chapters } = await freshBar()
    expect(chapters.libraryChapterDoneSuggestionVisible.value).toBe(false)

    state.setEditorCaret(12)
    await state.confirmCurrentSegment()
    expect(chapters.libraryChapterDoneSuggestionVisible.value).toBe(true)

    chaptersOnDisk = [{ ...(CHAPTERS[0] as object), status: 'done' }]
    await chapters.loadChapters()
    expect(chapters.libraryChapterDoneSuggestionVisible.value).toBe(false)
    wrapper.unmount()
  })

  it('đặt Đã xong từ nút tắt gợi ý đang bật', async () => {
    omittedSegmentId = 13
    const { state, wrapper } = await freshBar()
    state.setEditorCaret(12)
    await state.confirmCurrentSegment()
    expect(state.editorChapterDoneSuggested.value).toBe(true)

    await doneButton(wrapper).trigger('click')
    await flushPromises()
    expect(state.editorChapterDoneSuggested.value).toBe(false)
    wrapper.unmount()
  })

  it('dải gợi ý ở Workspace: hiện khi xác nhận câu cuối; "Để sau" đóng, nút đặt dùng cùng lệnh với nút thanh trạng thái', async () => {
    omittedSegmentId = 13
    const { state, wrapper: bar } = await freshBar()
    const WorkspaceMode = (await import('../../src/modes/WorkspaceMode.vue')).default
    const mode = mount(WorkspaceMode, { global: { stubs: { WorkspaceDock: true, ReviewDock: true } } })
    await flushPromises()
    expect(mode.find('[data-chapter-done-suggestion]').exists()).toBe(false)

    state.setEditorCaret(12)
    await state.confirmCurrentSegment()
    await flushPromises()
    expect(mode.get('[data-chapter-done-suggestion]').text()).toContain('Đánh dấu Chương Đã xong?')

    await mode.get('[data-chapter-done-suggestion-dismiss]').trigger('click')
    expect(mode.find('[data-chapter-done-suggestion]').exists()).toBe(false)
    expect(setChapterStatusWire).not.toHaveBeenCalled()

    state.setEditorCaret(12)
    await state.confirmCurrentSegment()
    await flushPromises()
    await mode.get('[data-chapter-done-suggestion-accept]').trigger('click')
    await flushPromises()
    expect(setChapterStatusWire).toHaveBeenCalledWith({ chapterId: 7, status: 'done' })
    expect(mode.find('[data-chapter-done-suggestion]').exists()).toBe(false)
    mode.unmount()
    bar.unmount()
  })
})

describe('thanh trạng thái — nút ‹ › sang Chương kề', () => {
  const nav = (w: Awaited<ReturnType<typeof freshBar>>['wrapper']) => ({
    prev: w.find('[data-chapter-prev]'),
    next: w.find('[data-chapter-next]'),
  })

  it.each([
    ['data-chapter-next', 'goToNextChapter'],
    ['data-chapter-prev', 'goToPrevChapter'],
  ] as const)('bấm %s chạy đúng dep %s', async (attr, dep) => {
    const goToNextChapter = vi.fn()
    const goToPrevChapter = vi.fn()
    const { wrapper } = await freshBar({ goToNextChapter, goToPrevChapter })
    await wrapper.get(`[${attr}]`).trigger('click')
    const called = dep === 'goToNextChapter' ? goToNextChapter : goToPrevChapter
    const other = dep === 'goToNextChapter' ? goToPrevChapter : goToNextChapter
    expect(called).toHaveBeenCalledTimes(1)
    expect(other).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('mũi tên là icon svg dùng chung, không còn ký tự chữ cỡ theo phông', async () => {
    const { wrapper } = await freshBar()
    for (const button of Object.values(nav(wrapper))) {
      expect(button.find('svg.app-icon').exists()).toBe(true)
      expect(button.text()).toBe('')
    }
    wrapper.unmount()
  })

  it('mousedown trên nút bị chặn mặc định để tiêu điểm và caret ở nguyên chỗ', async () => {
    const { wrapper } = await freshBar()
    const event = new MouseEvent('mousedown', { bubbles: true, cancelable: true })
    wrapper.get('[data-chapter-next]').element.dispatchEvent(event)
    expect(event.defaultPrevented).toBe(true)
    wrapper.unmount()
  })

  it('có aria-label và title kèm tên lệnh', async () => {
    const { wrapper } = await freshBar()
    const { next, prev } = nav(wrapper)
    expect(next.attributes('aria-label')).toBe('Chương sau')
    expect(prev.attributes('aria-label')).toBe('Chương trước')
    expect(next.attributes('title')).toContain('Chương sau')
    wrapper.unmount()
  })

  it('Chương đã nạp, Workspace, Review Mode đóng ⇒ hiện cả khi chưa có caret', async () => {
    const { wrapper } = await freshBar()
    expect(info(wrapper, 'data-status-position')).toBeNull()
    expect(nav(wrapper).next.exists()).toBe(true)
    expect(nav(wrapper).prev.exists()).toBe(true)
    wrapper.unmount()
  })

  it('Review Mode mở ⇒ ẩn cả hai nút', async () => {
    const { wrapper } = await freshBar()
    reviewOpen.value = true
    await flushPromises()
    expect(nav(wrapper).next.exists()).toBe(false)
    expect(nav(wrapper).prev.exists()).toBe(false)
    wrapper.unmount()
  })

  it('Chương đang nạp ⇒ ẩn cùng lúc với dòng vị trí', async () => {
    const { state, wrapper } = await freshBar()
    state.resetEditorPanel()
    await flushPromises()
    expect(state.editorHasLoaded()).toBe(false)
    expect(nav(wrapper).next.exists()).toBe(false)
    expect(info(wrapper, 'data-status-position')).toBeNull()
    wrapper.unmount()
  })

  it('Thư viện hoặc Chế độ đọc ⇒ không có nút', async () => {
    const { wrapper } = await freshBar()
    const modes = await import('../../src/modes/modeState')
    for (const mode of ['library', 'reading'] as const) {
      modes.setMode(mode)
      await flushPromises()
      expect(nav(wrapper).next.exists(), mode).toBe(false)
    }
    wrapper.unmount()
  })

  it('nút ngăn kéo ẩn khi panel.lookup đang ở dock và hiện lại khi nó rời dock', async () => {
    const { wrapper } = await freshBar()
    const layout = await import('../../src/layout/lookupDrawerState')
    const visibility = await import('../../src/layout/panelVisibilityState')
    layout.syncLayoutTier('narrow')
    visibility.setVisibleDockPanels(['panel.grid', 'panel.lookup'])
    await flushPromises()
    expect(wrapper.find('[data-lookup-drawer-open]').exists()).toBe(false)

    visibility.setVisibleDockPanels(['panel.grid'])
    await flushPromises()
    expect(wrapper.find('[data-lookup-drawer-open]').exists()).toBe(true)
    wrapper.unmount()
  })
})
