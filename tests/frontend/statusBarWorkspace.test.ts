/**
 * Status bar, 12.7: permanent info (position, confirmed counter, key hint, Glossary/TM counts,
 * narrow lookup line) sits OUTSIDE the `role="status"` live region, which carries only the notice chain.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

const tmCount = vi.fn<() => Promise<number | null>>()
const glossaryCount = vi.fn<() => Promise<number | null>>()

async function confirmOk(segmentId: number) {
  return { outcome: { segment_id: segmentId, status: 'confirmed', version_created: true }, error: null }
}

const CHAPTERS = [
  { chapter_id: 7, ord: 47, title: null, status: 'draft', segment_count: 3, translated_count: 2, unconfirmed_count: 2, in_translation_count: 3, updated_at: '2026-10-14T00:00:00Z', origin_author: null, origin_site_name: null, origin_url: null, origin_published_at: null },
]
let chaptersOnDisk: unknown[] = CHAPTERS
let omittedSegmentId: number | null = null

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
}))
vi.mock('../../src/config/chapter', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/chapter')>()),
  listChapters: async () => ({ chapters: chaptersOnDisk, error: null }),
}))
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
  invoke: vi.fn(async (cmd: string) => {
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
    throw new Error(`statusBarWorkspace.test.ts: unexpected invoke "${cmd}"`)
  }),
}))

async function freshBar() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const chapters = await import('../../src/modes/libraryChapters')
  const commands = await import('../../src/commands')
  if (commands.commandRegistry.list().length === 0) commands.installCommands({ isMac: true, setMode: () => {} })
  const StatusBar = (await import('../../src/StatusBar.vue')).default
  await chapters.loadChapters()
  await state.ensureSegmentsLoaded()
  const wrapper = mount(StatusBar, { attachTo: document.body })
  await flushPromises()
  return { state, wrapper, commands }
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
  tmCount.mockReset().mockResolvedValue(12)
  glossaryCount.mockReset().mockResolvedValue(7)
  chaptersOnDisk = CHAPTERS
  omittedSegmentId = null
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
    for (const attr of ['data-status-position', 'data-status-confirmed', 'data-status-hint', 'data-status-glossary', 'data-status-tm']) {
      expect(roles[0].querySelector(`[${attr}]`), `${attr} nằm trong vùng sống`).toBeNull()
    }
    wrapper.unmount()
  })

  it('gợi ý phím gộp/tách theo phím ĐANG GÁN: gán lại phím gộp thì gợi ý đổi theo', async () => {
    const { wrapper } = await freshBar()
    const shortcuts = await import('../../src/config/shortcutsState')
    const rowOf = (id: string) => shortcuts.shortcutRows.value.find((r) => r.id === id)
    const hintFor = (): string =>
      `${rowOf('editor.merge_segments')?.display.join(' / ')} gộp · ${rowOf('editor.split_segment')?.display.join(' / ')} tách`

    const before = info(wrapper, 'data-status-hint')
    expect(before).toBe(hintFor())

    assignChord(shortcuts, 'editor.merge_segments', 'KeyY')
    await flushPromises()

    const after = info(wrapper, 'data-status-hint')
    expect(rowOf('editor.merge_segments')?.chords).toHaveLength(1)
    expect(rowOf('editor.merge_segments')?.chords).not.toEqual(['Mod+M'])
    expect(after).toBe(hintFor())
    expect(after).not.toBe(before)
    wrapper.unmount()
  })

  it('gỡ phím gộp ⇒ ẩn cả gợi ý, không in nửa câu', async () => {
    const { wrapper } = await freshBar()
    const shortcuts = await import('../../src/config/shortcutsState')
    const row = document.createElement('div')
    row.setAttribute('data-command-id', 'editor.merge_segments')
    document.body.appendChild(row)
    row.addEventListener('click', shortcuts.aimRowFrom)
    row.click()
    shortcuts.unassignShortcut()
    await flushPromises()

    expect(info(wrapper, 'data-status-hint')).toBeNull()
    wrapper.unmount()
  })
})

describe('thanh trạng thái — số Glossary và TM', () => {
  it('hiện số từ hai lệnh đếm khi bật; lệnh trả null ⇒ ẩn mục đó, không "0"', async () => {
    tmCount.mockResolvedValue(null)
    const { wrapper } = await freshBar()

    expect(info(wrapper, 'data-status-glossary')).toBe('Glossary 7 mục')
    expect(info(wrapper, 'data-status-tm')).toBeNull()
    expect(wrapper.text()).not.toMatch(/TM 0 cặp|Glossary 0 mục/)
    wrapper.unmount()
  })

  it('0 thật vẫn hiện là 0 (khác "chưa biết")', async () => {
    tmCount.mockResolvedValue(0)
    const { wrapper } = await freshBar()
    expect(info(wrapper, 'data-status-tm')).toBe('TM 0 cặp')
    wrapper.unmount()
  })

  it('xác nhận câu thành công làm tươi bộ đếm xác nhận và số TM', async () => {
    const { state, wrapper } = await freshBar()
    state.setEditorCaret(12)
    await flushPromises()
    expect(info(wrapper, 'data-status-confirmed')).toBe('1/3 đã xác nhận')
    expect(info(wrapper, 'data-status-tm')).toBe('TM 12 cặp')
    const callsBefore = tmCount.mock.calls.length
    tmCount.mockResolvedValue(13)

    await state.confirmCurrentSegment()
    await flushPromises()

    expect(tmCount.mock.calls.length).toBeGreaterThan(callsBefore)
    expect(info(wrapper, 'data-status-tm')).toBe('TM 13 cặp')
    expect(info(wrapper, 'data-status-confirmed')).toBe('2/3 đã xác nhận')
    wrapper.unmount()
  })
})

describe('thanh trạng thái — tầng bố cục', () => {
  it('tầng rộng: câu nhắc auto-save nằm ngoài vùng sống; tầng khác không có nó', async () => {
    const { wrapper } = await freshBar()
    const layout = await import('../../src/layout/lookupDrawerState')

    expect(wrapper.find('[data-status-autosave-hint]').exists()).toBe(false)
    layout.syncLayoutTier('full')
    await flushPromises()
    const hint = wrapper.get('[data-status-autosave-hint]')
    expect(hint.text()).toContain('~2 giây')
    expect(hint.text()).toContain('5 giây')
    expect(wrapper.get('[data-status-live]').element.contains(hint.element)).toBe(false)

    layout.syncLayoutTier('short')
    await flushPromises()
    expect(wrapper.find('[data-status-autosave-hint]').exists()).toBe(false)
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
    for (const attr of ['data-status-glossary', 'data-status-tm', 'data-status-autosave-hint', 'data-status-lookup-offline']) {
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
