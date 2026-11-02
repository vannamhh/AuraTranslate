import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import type { CommandDeps } from '../../src/commands'
import type {
  BlockWire,
  ChapterOriginWire,
  ChapterSplitPreviewWire,
  ImportEncodingPreview,
  UrlImportBatchWire,
} from '../../src/config/project'

const ORIGIN_STUB: ChapterOriginWire = {
  author: null,
  site_name: null,
  url: null,
  published_at: null,
  author_confirmed: false,
  site_name_confirmed: false,
  url_confirmed: false,
  published_at_confirmed: false,
}

const previewTextMock = vi.fn()
const previewFileMock = vi.fn()
const startUrlImportMock = vi.fn()
const confirmMock = vi.fn()
const tier2BlockSetKeptMock = vi.fn()
const previewChapterDetailMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (...a: unknown[]) => previewTextMock(...a),
    previewImportEncodingFromFile: (...a: unknown[]) => previewFileMock(...a),
    startUrlImport: (...a: unknown[]) => startUrlImportMock(...a),
    confirmImportWithEncoding: (...a: unknown[]) => confirmMock(...a),
    tier2BlockSetKept: (...a: unknown[]) => tier2BlockSetKeptMock(...a),
    previewChapterDetail: (...a: unknown[]) => previewChapterDetailMock(...a),
  }
})

function resetMocks(): void {
  for (const m of [previewTextMock, previewFileMock, startUrlImportMock, confirmMock, tier2BlockSetKeptMock, previewChapterDetailMock]) m.mockReset()
}

async function fresh() {
  vi.resetModules()
  resetMocks()
  const commands = await import('../../src/commands')
  const state = await import('../../src/importPreviewState')
  commands.installCommands({
    nextImportPreviewBlock: state.nextImportPreviewBlock,
    prevImportPreviewBlock: state.prevImportPreviewBlock,
    toggleImportPreviewBlockKept: state.toggleImportPreviewBlockKept,
    confirmImportPreview: state.confirmImportPreview,
    openImportPreviewCandidatePicker: state.openImportPreviewCandidatePicker,
  } as unknown as CommandDeps)
  const Overlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { state, Overlay, commands }
}

function chaptersWire(count: number, over: Partial<ChapterSplitPreviewWire> = {}): ChapterSplitPreviewWire {
  return {
    chapter_count: count,
    chapters: Array.from({ length: count }, (_, i) => ({
      ord: i + 1,
      title: `Chuong ${i + 1}`,
      length: 100,
      cleanup_match_count: 0,
      joined_line_count_in_chapter: null,
      needs_review: i === 0,
      review_causes: [],
      chapter_origin: ORIGIN_STUB,
      source_file: null,
    })),
    broken_item_count: 0,
    needs_review_count: count > 0 ? 1 : 0,
    clean_count: Math.max(count - 1, 0),
    any_signal_participated: true,
    already_chaptered: false,
    ...over,
  }
}

function blockOf(kept: boolean, text = 'noi dung'): BlockWire {
  return { body: { kind: 'paragraph', text }, kept, confirmed: false }
}

function previewOf(
  chapterCount: number,
  over: { alreadyChaptered?: boolean; blocks?: BlockWire[] | null; confidence?: 'high' | 'low' | 'self_declared' } = {},
): ImportEncodingPreview {
  return {
    source_lang: 'en',
    source_lang_auto: false,
    confidence: over.confidence ?? 'self_declared',
    selected_encoding: 'UTF-8',
    candidates: [
      {
        label: 'UTF-8',
        encoding: 'UTF-8',
        preview: 'x',
        normalized: { text: 'x', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        cleanup: { text: 'x', spans: [], rules: [], window_truncated: false, final_text: 'x' },
        chapters: chaptersWire(chapterCount, { already_chaptered: over.alreadyChaptered ?? false }),
        blocks: over.blocks === undefined || over.blocks === null ? null : { blocks: over.blocks },
      },
    ],
    self_declared_normalized: null,
    self_declared_cleanup: null,
    self_declared_chapters: null,
  }
}

function urlBatch(blocks: BlockWire[], chapterCount = 1): UrlImportBatchWire {
  return {
    items: [{ url: 'https://a.example/1', ok: true, error: null }],
    encoding_preview: previewOf(chapterCount, { alreadyChaptered: true, blocks }),
    domain_log_domain_count: 1,
    duplicate_urls_dropped: 0,
  }
}

const URLS = ['https://a.example/1']

beforeEach(() => {
  document.body.innerHTML = ''
})
afterEach(() => {
  document.body.innerHTML = ''
})

describe('importPreviewSplitPatternApplies (cờ already_chaptered của Rust, không phải đường nhập)', () => {
  it('false khi chưa có Chương, false khi already_chaptered, true khi không', async () => {
    const { state } = await fresh()
    expect(state.importPreviewSplitPatternApplies.value).toBe(false)

    previewTextMock.mockResolvedValue({ preview: previewOf(2, { alreadyChaptered: false }), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x', null)
    expect(state.importPreviewSplitPatternApplies.value).toBe(true)

    previewTextMock.mockResolvedValue({ preview: previewOf(2, { alreadyChaptered: true }), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x', null)
    expect(state.importPreviewSplitPatternApplies.value).toBe(false)
  })
})

describe('importPreviewTotalChars', () => {
  it('null khi chưa có Chương, tổng chapters[].length khi có', async () => {
    const { state } = await fresh()
    expect(state.importPreviewTotalChars.value).toBeNull()
    previewTextMock.mockResolvedValue({ preview: previewOf(3), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x', null)
    expect(state.importPreviewTotalChars.value).toBe(300)
  })
})

describe('tên Tác phẩm sửa được trên màn xem trước', () => {
  it('Tác phẩm mới: setter đổi tên, tên rỗng sau trim tắt xác nhận, xác nhận dùng tên đã sửa', async () => {
    const { state } = await fresh()
    previewTextMock.mockResolvedValue({ preview: previewOf(1), error: null })
    await state.openImportPreviewFromText('Ten cu', 'en', '', 'x', null)
    expect(state.importPreviewCanConfirm.value).toBe(true)

    state.setImportPreviewPendingName('   ')
    expect(state.importPreviewPendingName.value).toBe('   ')
    expect(state.importPreviewCanConfirm.value).toBe(false)
    confirmMock.mockResolvedValue({ created: null, error: null })
    await state.confirmImportPreview()
    expect(confirmMock).not.toHaveBeenCalled()

    state.setImportPreviewPendingName('Ten moi')
    expect(state.importPreviewCanConfirm.value).toBe(true)
    await state.confirmImportPreview()
    expect(confirmMock).toHaveBeenCalledTimes(1)
    expect(confirmMock.mock.calls[0]?.[0]).toBe('Ten moi')
  })

  it('đích là Tác phẩm đã có: setter không đổi tên, nút không phụ thuộc tên', async () => {
    const { state } = await fresh()
    previewTextMock.mockResolvedValue({ preview: previewOf(1), error: null })
    await state.openImportPreviewFromText('Ten co san', 'en', '', 'x', 'work-1')
    state.setImportPreviewPendingName('')
    expect(state.importPreviewPendingName.value).toBe('Ten co san')
    expect(state.importPreviewCanConfirm.value).toBe(true)
  })

  it('ô tên là textarea ghi qua setter; ô rỗng tắt nút Nhập trong chân', async () => {
    const { state, Overlay } = await fresh()
    previewTextMock.mockResolvedValue({ preview: previewOf(1), error: null })
    await state.openImportPreviewFromText('Ten cu', 'en', '', 'x', null)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    const box = wrapper.get('aside.ip-config textarea[data-ip-work-name]')
    expect((box.element as HTMLTextAreaElement).value).toBe('Ten cu')
    await box.setValue('  ')
    expect(state.importPreviewCanConfirm.value).toBe(false)
    expect(wrapper.get('footer.ip-footer .ip-act-primary').attributes('disabled')).toBeDefined()
    await box.setValue('Ten moi')
    expect(wrapper.get('footer.ip-footer .ip-act-primary').attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })

  it('đích là Tác phẩm đã có: không dựng ô tên', async () => {
    const { state, Overlay } = await fresh()
    previewTextMock.mockResolvedValue({ preview: previewOf(1), error: null })
    await state.openImportPreviewFromText('Ten co san', 'en', '', 'x', 'work-1')
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    expect(wrapper.find('[data-ip-work-name]').exists()).toBe(false)
    wrapper.unmount()
  })
})

describe('ô mẫu phân tách theo cờ already_chaptered', () => {
  it('URL (already_chaptered=true): không có ô mẫu', async () => {
    const { state, Overlay } = await fresh()
    startUrlImportMock.mockResolvedValue({ batch: urlBatch([blockOf(true)]), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', URLS, null)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    expect(wrapper.find('.ip-chapters-pattern-row').exists()).toBe(false)
    expect(wrapper.find('.ip-chapters-pattern-input').exists()).toBe(false)
    wrapper.unmount()
  })

  it('dán tay (false): có ô mẫu, kể cả khi chỉ 1 Chương', async () => {
    const { state, Overlay } = await fresh()
    previewTextMock.mockResolvedValue({ preview: previewOf(1), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x', null)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    expect(wrapper.find('.ip-chapters-pattern-input').exists()).toBe(true)
    wrapper.unmount()
  })

  it('nhiều tệp (false): có ô mẫu, gõ mẫu gửi lại xem trước và số Chương đổi', async () => {
    const { state, Overlay } = await fresh()
    previewFileMock.mockResolvedValue({
      batch: {
        items: [
          { path: '/tmp/a.txt', byte_len: 10, ok: true, error: null },
          { path: '/tmp/b.txt', byte_len: 10, ok: true, error: null },
        ],
        encoding_preview: previewOf(2),
      },
      error: null,
    })
    await state.openImportPreviewFromFile('Ten', 'en', '', ['/tmp/a.txt', '/tmp/b.txt'], null)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    expect(wrapper.find('.ip-chapters-pattern-input').exists()).toBe(true)

    previewFileMock.mockResolvedValue({
      batch: {
        items: [
          { path: '/tmp/a.txt', byte_len: 10, ok: true, error: null },
          { path: '/tmp/b.txt', byte_len: 10, ok: true, error: null },
        ],
        encoding_preview: previewOf(5),
      },
      error: null,
    })
    await wrapper.get('.ip-chapters-pattern-input').setValue('Chuong')
    await wrapper.get('.ip-chapters-pattern-input').trigger('change')
    await new Promise((r) => setTimeout(r))
    await nextTick()
    expect(wrapper.get('[data-ip-footer-chapters]').text()).toMatch(/Chương nhận ra\s*5/)
    expect(wrapper.get('footer.ip-footer .ip-act-primary').text()).toContain('Nhập 5 Chương')
    wrapper.unmount()
  })
})

describe('một Chương và nhiều Chương', () => {
  it('1 Chương: không có dải Chương, bộ đếm, ⌥W hay khối Tách Chương; chân ghi "Nhập 1 Chương"', async () => {
    const { state, Overlay } = await fresh()
    previewTextMock.mockResolvedValue({ preview: previewOf(1), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x', null)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    expect(wrapper.find('.ip-chapter-strip').exists()).toBe(false)
    expect(wrapper.find('.ip-chapter-filter-bar').exists()).toBe(false)
    expect(wrapper.find('.ip-chapter-filter-key').exists()).toBe(false)
    expect(wrapper.find('.ip-tier-chapters').exists()).toBe(false)
    expect(wrapper.get('footer.ip-footer .ip-act-primary').text()).toContain('Nhập 1 Chương')
    wrapper.unmount()
  })

  it('3 Chương: dải Chương ở đầu cột văn bản với bộ đếm cần xem · sạch và ⌥W', async () => {
    const { state, Overlay } = await fresh()
    previewTextMock.mockResolvedValue({ preview: previewOf(3), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x', null)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    const strip = wrapper.get('.ip-main .ip-chapter-strip')
    expect(strip.get('.ip-chapter-filter-chip-needs-review').text()).toContain('1')
    expect(strip.get('.ip-chapter-filter-chip-clean').text()).toContain('2')
    expect(strip.get('.ip-chapter-filter-key').text()).toBe('⌥W')
    expect(wrapper.find('aside.ip-config .ip-chapter-strip').exists()).toBe(false)
    expect(wrapper.get('footer.ip-footer .ip-act-primary').text()).toContain('Nhập 3 Chương')
    wrapper.unmount()
  })
})

describe('bảng mã tin cậy', () => {
  it('low: dải ứng viên mở sẵn; high: dòng gọn, bấm Đổi mở dải', async () => {
    const { state, Overlay } = await fresh()
    previewTextMock.mockResolvedValue({ preview: previewOf(1, { confidence: 'low' }), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x', null)
    let wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    expect(wrapper.find('.ip-strip').exists()).toBe(true)
    expect(wrapper.find('[data-ip-encoding-line]').exists()).toBe(false)
    wrapper.unmount()

    previewTextMock.mockResolvedValue({ preview: previewOf(1, { confidence: 'high' }), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x', null)
    wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    expect(wrapper.find('.ip-strip').exists()).toBe(false)
    await wrapper.get('[data-ip-encoding-line] button').trigger('click')
    await nextTick()
    expect(wrapper.find('.ip-strip').exists()).toBe(true)
    wrapper.unmount()
  })
})

describe('khối loại gộp thành dải (đường URL)', () => {
  it('mỗi dãy loại liền nhau là một dải, kể cả dãy một khối; khối giữ hiện đủ', async () => {
    const { state, Overlay } = await fresh()
    const blocks = [
      ...Array.from({ length: 200 }, (_, i) => blockOf(false, `loai ${i}`)),
      blockOf(true, 'giu A'),
      blockOf(false, 'loai le'),
      blockOf(true, 'giu B'),
    ]
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(blocks), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', URLS, null)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    const bands = wrapper.findAll('.ip-band')
    expect(bands).toHaveLength(2)
    expect(bands[0]?.get('.ip-band-title').text()).toBe('200 khối đã loại')
    expect(bands[1]?.get('.ip-band-title').text()).toBe('1 khối đã loại')
    expect(wrapper.findAll('.ip-block')).toHaveLength(2)
    wrapper.unmount()
  })

  it('giữ khối 40 trong dải 98 khối đang mở: dải tách 39 + 58, khối 40 hiện đủ', async () => {
    const { state, Overlay } = await fresh()
    const all = Array.from({ length: 98 }, () => blockOf(false))
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(all), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', URLS, null)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    await state.toggleImportPreviewBlockKept() // opens the band
    state.nextImportPreviewBlock() // header -> block 0
    state.setImportPreviewBlockFocus(39)
    const after = all.map((b, i) => (i === 39 ? { ...b, kept: true, confirmed: true } : b))
    tier2BlockSetKeptMock.mockResolvedValue({ batch: urlBatch(after), error: null })
    await state.toggleImportPreviewBlockKept()
    await nextTick()
    expect(tier2BlockSetKeptMock).toHaveBeenCalledWith(39, true, 'en', 0)
    const titles = wrapper.findAll('.ip-band-title').map((e) => e.text())
    expect(titles).toEqual(['39 khối đã loại', '58 khối đã loại'])
    expect(wrapper.findAll('.ip-block')).toHaveLength(98)
    expect(wrapper.findAll('.ip-block-kept-confirmed')).toHaveLength(1)
    wrapper.unmount()
  })

  it('J dừng ở dải thu, J nữa qua tới khối sau dải; Space mở/thu dải mà không gọi IPC', async () => {
    const { state, Overlay } = await fresh()
    const blocks = [blockOf(true), blockOf(false), blockOf(false), blockOf(false), blockOf(true)]
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(blocks), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', URLS, null)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    const scrim = wrapper.get('.ip-screen')
    expect(state.importPreviewBlockFocusedIndex.value).toBe(0)

    await scrim.trigger('keydown', { key: 'j' })
    expect(state.importPreviewFocusedBand.value).toMatchObject({ start: 1, end: 3, open: false })
    await scrim.trigger('keydown', { key: ' ' })
    expect(state.importPreviewFocusedBand.value?.open).toBe(true)
    expect(wrapper.findAll('.ip-block')).toHaveLength(5)
    await scrim.trigger('keydown', { key: ' ' })
    expect(state.importPreviewFocusedBand.value?.open).toBe(false)
    expect(wrapper.findAll('.ip-block')).toHaveLength(2)
    expect(tier2BlockSetKeptMock).not.toHaveBeenCalled()

    await scrim.trigger('keydown', { key: 'j' })
    expect(state.importPreviewFocusedBand.value).toBeNull()
    expect(state.importPreviewBlockFocusedIndex.value).toBe(4)
    await scrim.trigger('keydown', { key: 'k' })
    expect(state.importPreviewFocusedBand.value).toMatchObject({ start: 1, end: 3, open: false })
    wrapper.unmount()
  })

  it('khối vừa loại bằng Space vẫn là một hàng riêng có con trỏ, Space lần nữa giữ lại; J đi thì nó gộp vào dải', async () => {
    const { state, Overlay } = await fresh()
    const blocks = [blockOf(false), blockOf(true), blockOf(false), blockOf(true)]
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(blocks), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', URLS, null)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    const scrim = wrapper.get('.ip-screen')
    await scrim.trigger('keydown', { key: 'j' })
    expect(state.importPreviewBlockFocusedIndex.value).toBe(1)
    expect(state.importPreviewFocusedBand.value).toBeNull()

    const dropped = blocks.map((b, i) => (i === 1 ? { ...b, kept: false } : b))
    tier2BlockSetKeptMock.mockResolvedValue({ batch: urlBatch(dropped), error: null })
    await scrim.trigger('keydown', { key: ' ' })
    await vi.waitFor(() => expect(tier2BlockSetKeptMock).toHaveBeenCalledWith(1, false, 'en', 0))
    await nextTick()
    expect(wrapper.findAll('.ip-band')).toHaveLength(2)
    expect(wrapper.findAll('.ip-block')).toHaveLength(2)
    expect(wrapper.get('#ip-block-1').classes()).toContain('ip-block-focused')
    expect(wrapper.get('#ip-block-1').classes()).toContain('ip-block-dropped')

    tier2BlockSetKeptMock.mockResolvedValue({ batch: urlBatch(blocks), error: null })
    await scrim.trigger('keydown', { key: ' ' })
    await vi.waitFor(() => expect(tier2BlockSetKeptMock).toHaveBeenLastCalledWith(1, true, 'en', 0))

    tier2BlockSetKeptMock.mockResolvedValue({ batch: urlBatch(dropped), error: null })
    await scrim.trigger('keydown', { key: ' ' })
    await vi.waitFor(() => expect(tier2BlockSetKeptMock).toHaveBeenCalledTimes(3))
    await nextTick()
    await scrim.trigger('keydown', { key: 'j' })
    await nextTick()
    expect(wrapper.findAll('.ip-band-title').map((e) => e.text())).toEqual(['3 khối đã loại'])
    wrapper.unmount()
  })

  it('chuyển Chương: dải đang mở ở Chương trước không mở sẵn ở Chương sau', async () => {
    const { state } = await fresh()
    const blocks = [blockOf(false), blockOf(false), blockOf(true)]
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(blocks, 2), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', URLS, null)
    await state.toggleImportPreviewBlockKept()
    expect(state.importPreviewFocusedBand.value).toMatchObject({ start: 0, end: 1, open: true })

    previewChapterDetailMock.mockResolvedValue({
      detail: {
        cleanup: { text: 'x', spans: [], rules: [], window_truncated: false, final_text: 'x' },
        blocks: { blocks },
      },
      error: null,
    })
    state.nextImportPreviewChapter()
    await vi.waitFor(() => expect(previewChapterDetailMock).toHaveBeenCalledWith(1, 'UTF-8', 'en', null))
    await nextTick()
    expect(state.importPreviewBlockRows.value[0]).toMatchObject({ kind: 'band', start: 0, end: 1, open: false })
  })
})

describe('nút Nhập trong chân', () => {
  it('đang xác nhận: nút tắt và confirmImportPreview không gửi lượt thứ hai', async () => {
    const { state, Overlay } = await fresh()
    previewTextMock.mockResolvedValue({ preview: previewOf(2), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x', null)
    const wrapper = mount(Overlay, { attachTo: document.body })
    await nextTick()
    let release: (v: unknown) => void = () => undefined
    confirmMock.mockReturnValue(new Promise((r) => (release = r)))
    void state.confirmImportPreview()
    await nextTick()
    expect(wrapper.get('footer.ip-footer .ip-act-primary').attributes('disabled')).toBeDefined()
    await state.confirmImportPreview()
    expect(confirmMock).toHaveBeenCalledTimes(1)
    release({ created: null, error: null })
    await new Promise((r) => setTimeout(r))
    wrapper.unmount()
  })
})
