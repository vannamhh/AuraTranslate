import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'
import type { ChapterOriginWire, GlossaryEstimateWire, ImportEncodingPreview } from '../../src/config/project'

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
const startUrlMock = vi.fn()
const estimateMock = vi.fn()
const cancelEstimateMock = vi.fn()
const confirmMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (...args: unknown[]) => previewTextMock(...args),
    previewImportEncodingFromFile: (...args: unknown[]) => previewFileMock(...args),
    startUrlImport: (...args: unknown[]) => startUrlMock(...args),
    estimateImportGlossary: (...args: unknown[]) => estimateMock(...args),
    cancelImportGlossaryEstimate: () => cancelEstimateMock(),
    confirmImportWithEncoding: (...args: unknown[]) => confirmMock(...args),
  }
})

function chaptersBlock(count: number) {
  return {
    chapter_count: count,
    chapters: Array.from({ length: Math.min(count, 3) }, (_, i) => ({
      ord: i + 1,
      title: `Chuong ${i + 1}`,
      length: 10,
      cleanup_match_count: 0,
      joined_line_count_in_chapter: null,
      needs_review: false,
      review_causes: [],
      chapter_origin: ORIGIN_STUB,
      source_file: null,
    })),
    broken_item_count: 0,
    needs_review_count: 0,
    clean_count: count,
    any_signal_participated: false,
  }
}

function proseCandidate(chapterCount: number) {
  return {
    label: 'UTF-8',
    encoding: 'UTF-8',
    preview: 'x',
    normalized: { text: 'x', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
    cleanup: { text: 'x', spans: [], rules: [], window_truncated: false, final_text: 'x' },
    chapters: chaptersBlock(chapterCount),
    blocks: null,
  }
}

function proseFixture(chapterCount: number): ImportEncodingPreview {
  return {
    selected_encoding: 'UTF-8',
    confidence: 'high',
    candidates: [proseCandidate(chapterCount)],
    self_declared_normalized: null,
    self_declared_cleanup: null,
    self_declared_chapters: null,
  }
}

function urlFixture(count: number) {
  const urls = Array.from({ length: count }, (_, i) => `https://site.example/c/${i + 1}`)
  return {
    urls,
    batch: {
      items: urls.map((url) => ({ url, ok: true, error: null })),
      encoding_preview: {
        ...proseFixture(count),
        candidates: [],
        confidence: 'self_declared',
        self_declared_normalized: { text: '', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
        self_declared_cleanup: { text: '', spans: [], rules: [], window_truncated: false, final_text: 'x' },
        self_declared_chapters: chaptersBlock(count),
      },
      domain_log_domain_count: 1,
      duplicate_urls_dropped: 0,
    },
  }
}

function wire(outcome: GlossaryEstimateWire['outcome'], count: number | null): { estimate: GlossaryEstimateWire; error: null } {
  return { estimate: { outcome, count }, error: null }
}

async function freshOverlay() {
  vi.resetModules()
  for (const m of [previewTextMock, previewFileMock, startUrlMock, estimateMock, cancelEstimateMock, confirmMock]) m.mockReset()
  const commands = await import('../../src/commands')
  const stateForDeps = await import('../../src/importPreviewState')
  commands.installCommands({ cancelImportPreview: stateForDeps.cancelImportPreview } as never)
  const state = stateForDeps
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { state, ImportPreviewOverlay }
}

beforeEach(() => {
  document.body.innerHTML = ''
  vi.useFakeTimers()
})

afterEach(() => {
  vi.useRealTimers()
})

async function settle(ms = 400) {
  await vi.advanceTimersByTimeAsync(ms)
  await flushPromises()
}

describe('màn tách Chương hai cột (tệp txt 2.000 Chương)', () => {
  it('cột trái ghi tên Tác phẩm, ngôn ngữ cố định, Chương nhận ra, trạng thái ban đầu, ứng viên Glossary; nút ghi "Xác nhận và nhập 2 000 Chương"', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewFileMock.mockResolvedValue({
      batch: { items: [{ path: '/tmp/truyen.txt', byte_len: 43_200_000, ok: true, error: null }], encoding_preview: proseFixture(2000) },
      error: null,
    })
    estimateMock.mockResolvedValue(wire('estimated', 12345))
    await state.openImportPreviewFromFile('Tuyet Trung', 'zh', '', ['/tmp/truyen.txt'], null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await settle()

    expect(wrapper.find('.ip-two').exists()).toBe(true)
    const config = wrapper.get('aside.ip-config').text()
    expect(config).toContain('Tuyet Trung')
    expect(config).toContain('cố định sau khi tạo')
    expect(config).toContain('Chưa bắt đầu')
    expect(config).toMatch(/Chương nhận ra\s*2 000/)
    expect(wrapper.get('[data-ip-glossary-estimate]').text()).toContain('~12 345')
    expect(wrapper.get('.ip-act-primary').text()).toBe('Xác nhận và nhập 2 000 Chương')
    expect(wrapper.find('.ip-scrim').exists()).toBe(false)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('không có tầng chuẩn hoá trên bất kỳ nguồn nào', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: proseFixture(2), error: null })
    estimateMock.mockResolvedValue(wire('estimated', 1))
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung', null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await settle()

    expect(wrapper.find('.ip-normalized-text').exists()).toBe(false)
    expect(wrapper.find('.ip-chapters-count').exists()).toBe(false)

    wrapper.unmount()
    state.resetImportPreview()
  })
})

describe('ước lượng ứng viên Glossary (FR137)', () => {
  it('chạy sau khi xem trước dựng xong: "đang ước lượng…" rồi "~N", gọi với nguồn, bảng mã, mẫu', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: proseFixture(2), error: null })
    let resolveEstimate: (v: unknown) => void = () => {}
    estimateMock.mockImplementation(() => new Promise((resolve) => { resolveEstimate = resolve }))
    await state.openImportPreviewFromText('Ten', 'zh', '', 'noi dung', null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })

    expect(wrapper.get('[data-ip-glossary-estimate]').text()).toContain('đang ước lượng…')
    await settle()
    expect(estimateMock).toHaveBeenCalledTimes(1)
    expect(estimateMock.mock.calls[0]?.slice(0, 2)).toEqual(['zh', 'UTF-8'])
    expect(wrapper.get('[data-ip-glossary-estimate]').text()).toContain('đang ước lượng…')

    resolveEstimate(wire('estimated', 77))
    await flushPromises()
    expect(wrapper.get('[data-ip-glossary-estimate]').text()).toContain('~77')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('probe Inconclusive: "không ước lượng được" và xác nhận vẫn bấm được', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: proseFixture(2), error: null })
    estimateMock.mockResolvedValue(wire('inconclusive', null))
    await state.openImportPreviewFromText('Ten', 'zh', '', 'noi dung', null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await settle()

    expect(wrapper.get('[data-ip-glossary-estimate]').text()).toContain('không ước lượng được')
    expect(wrapper.get('.ip-act-primary').attributes('disabled')).toBeUndefined()
    expect(wrapper.get('[data-ip-glossary-estimate]').text()).not.toContain('~0')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('lỗi IPC hoặc cancelled cũng không thành "~0"', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: proseFixture(2), error: null })
    estimateMock.mockResolvedValue({ estimate: null, error: null })
    await state.openImportPreviewFromText('Ten', 'zh', '', 'noi dung', null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await settle()

    expect(wrapper.get('[data-ip-glossary-estimate]').text()).toContain('không ước lượng được')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('đổi mẫu tách Chương khi lượt cũ còn chạy: huỷ lượt cũ và bỏ kết quả muộn của nó', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: proseFixture(2), error: null })
    const resolvers: Array<(v: unknown) => void> = []
    estimateMock.mockImplementation(() => new Promise((resolve) => { resolvers.push(resolve) }))
    await state.openImportPreviewFromText('Ten', 'zh', '', 'noi dung', null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await settle()
    expect(estimateMock).toHaveBeenCalledTimes(1)

    previewTextMock.mockResolvedValue({ preview: proseFixture(5), error: null })
    await state.setImportPreviewChapterPattern('^Chuong', 'regex')
    expect(cancelEstimateMock).toHaveBeenCalled()
    await settle()
    expect(estimateMock).toHaveBeenCalledTimes(2)

    resolvers[0]?.(wire('estimated', 111))
    await flushPromises()
    expect(wrapper.get('[data-ip-glossary-estimate]').text()).not.toContain('~111')
    resolvers[1]?.(wire('estimated', 222))
    await flushPromises()
    expect(wrapper.get('[data-ip-glossary-estimate]').text()).toContain('~222')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('huỷ màn khi ước lượng đang chạy: gọi huỷ ước lượng, không bao giờ ghi bảng chờ từ phía giao diện', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: proseFixture(2), error: null })
    estimateMock.mockImplementation(() => new Promise(() => {}))
    await state.openImportPreviewFromText('Ten', 'zh', '', 'noi dung', null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await settle()

    state.cancelImportPreview()
    expect(cancelEstimateMock).toHaveBeenCalledTimes(1)
    expect(confirmMock).not.toHaveBeenCalled()

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('đường nhập song ngữ không đăng ký lệnh ước lượng nào', async () => {
    const { readFileSync } = await import('node:fs')
    const source = readFileSync('src/bilingualImportPreviewState.ts', 'utf8')
    expect(source).not.toMatch(/estimateImportGlossary/)
  })
})

describe('màn xem trước URL hợp nhất', () => {
  it('50 Chương đã tải: một màn, thứ tự tầng 1-3 rồi tầng 4 Tách Chương rồi tầng "Xuất xứ tài liệu" cuối, không tầng chuẩn hoá', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const { urls, batch } = urlFixture(50)
    startUrlMock.mockResolvedValue({ batch, error: null })
    estimateMock.mockResolvedValue(wire('estimated', 9))
    await state.openImportPreviewFromUrls('Ten', 'zh', '', urls, null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await settle()

    const titles = wrapper.findAll('.ip-tier-title').map((h) => h.text())
    const iTier1 = titles.findIndex((x) => x === 'Bảng mã')
    const iTier2 = titles.findIndex((x) => x === 'Ranh giới nội dung')
    const iTier3 = titles.findIndex((x) => x === 'Luật làm sạch')
    const iTier4 = titles.findIndex((x) => x === 'Tách Chương')
    const iOrigin = titles.findIndex((x) => x === 'Xuất xứ tài liệu')
    expect(iTier1).toBeGreaterThanOrEqual(0)
    expect(iTier1).toBeLessThan(iTier2)
    expect(iTier2).toBeLessThan(iTier3)
    expect(iTier3).toBeLessThan(iTier4)
    expect(iOrigin).toBe(titles.length - 1)
    expect(titles.some((x) => /chuẩn hoá/i.test(x))).toBe(false)
    expect(wrapper.get('.ip-act-primary').text()).toBe('Xác nhận và nhập 50 Chương')
    expect(wrapper.find('.ip-two-stacked').exists()).toBe(true)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('tầng "Xuất xứ tài liệu" chỉ có ở đường URL, vắng trên đường văn bản dán', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: proseFixture(2), error: null })
    estimateMock.mockResolvedValue(wire('estimated', 1))
    await state.openImportPreviewFromText('Ten', 'zh', '', 'noi dung', null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await settle()

    expect(wrapper.text()).not.toContain('Xuất xứ tài liệu')

    wrapper.unmount()
    state.resetImportPreview()
  })
})

describe('ô phải thanh tiêu đề ghi nguồn đang nhập', () => {
  it('một tệp: tên tệp và dung lượng, định dạng "41,2 MB"', async () => {
    const { state } = await freshOverlay()
    previewFileMock.mockResolvedValue({
      batch: { items: [{ path: '/tmp/tuyet-trung-han-dao-hanh.txt', byte_len: 43_200_000, ok: true, error: null }], encoding_preview: proseFixture(2) },
      error: null,
    })
    estimateMock.mockResolvedValue(wire('estimated', 1))
    await state.openImportPreviewFromFile('Ten', 'zh', '', ['/tmp/tuyet-trung-han-dao-hanh.txt'], null)
    const { importSourceStatusText } = await import('../../src/titlebarImportSource')

    expect(importSourceStatusText.value).toBe('tuyet-trung-han-dao-hanh.txt · 41,2 MB')

    state.resetImportPreview()
  })

  it('kích thước chưa biết (byte_len null): chỉ tên tệp, không bịa "0 B"', async () => {
    const { state } = await freshOverlay()
    previewFileMock.mockResolvedValue({
      batch: { items: [{ path: '/tmp/a.txt', byte_len: null, ok: true, error: null }], encoding_preview: proseFixture(2) },
      error: null,
    })
    estimateMock.mockResolvedValue(wire('estimated', 1))
    await state.openImportPreviewFromFile('Ten', 'zh', '', ['/tmp/a.txt'], null)
    const { importSourceStatusText } = await import('../../src/titlebarImportSource')

    expect(importSourceStatusText.value).toBe('a.txt')

    state.resetImportPreview()
  })

  it('văn bản dán: nhãn kèm dung lượng UTF-8 thật; không màn nào mở: null', async () => {
    const { state } = await freshOverlay()
    const { importSourceStatusText } = await import('../../src/titlebarImportSource')
    expect(importSourceStatusText.value).toBeNull()

    previewTextMock.mockResolvedValue({ preview: proseFixture(2), error: null })
    estimateMock.mockResolvedValue(wire('estimated', 1))
    await state.openImportPreviewFromText('Ten', 'zh', '', '中文', null)
    expect(importSourceStatusText.value).toBe('Văn bản đã dán · 6 B')

    state.resetImportPreview()
  })
})

describe('màn xem trước mở nhưng đã đổi sang tab khác', () => {
  it('chế độ Workspace ⇒ ô phải thanh tiêu đề trống; quay lại Library thì hiện lại', async () => {
    const { state } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: proseFixture(2), error: null })
    estimateMock.mockResolvedValue(wire('estimated', 1))
    await state.openImportPreviewFromText('Ten', 'zh', '', 'noi dung', null)
    const { importSourceStatusText } = await import('../../src/titlebarImportSource')
    const { setMode } = await import('../../src/modes/modeState')

    setMode('library')
    expect(importSourceStatusText.value).not.toBeNull()
    setMode('workspace')
    expect(importSourceStatusText.value).toBeNull()
    setMode('library')
    expect(importSourceStatusText.value).not.toBeNull()

    state.resetImportPreview()
  })
})

describe('cổng phím tắt toàn cục', () => {
  it('main.ts chặn phím toàn cục khi bất kỳ màn xem trước hoặc màn website đang mở', async () => {
    const { readFileSync } = await import('node:fs')
    const source = readFileSync('src/main.ts', 'utf8')
    const start = source.indexOf('isBlocked: () =>')
    expect(start).toBeGreaterThan(-1)
    const block = source.slice(start, source.indexOf('}', start + 200) + 1)
    for (const flag of ['importPreviewIsOpen', 'bilingualImportPreviewIsOpen', 'libraryUrlImportScreenIsOpen']) {
      expect(block, flag).toContain(`${flag}.value`)
    }
    expect(block).toMatch(/currentMode\.value === 'library' &&\s*\(libraryUrlImportScreenIsOpen/)
  })
})

describe('tiêu điểm khi đóng màn xem trước', () => {
  it('Huỷ trả tiêu điểm về nút đã mở màn; Esc trên màn cũng huỷ', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    document.body.innerHTML = '<button data-import-preview-open id="opener"></button>'
    const opener = document.getElementById('opener') as HTMLButtonElement
    opener.focus()
    previewTextMock.mockResolvedValue({ preview: proseFixture(2), error: null })
    estimateMock.mockResolvedValue(wire('estimated', 1))
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })

    await state.openImportPreviewFromText('Ten', 'zh', '', 'noi dung', null)
    await settle()
    expect(wrapper.get('.ip-panel').element.contains(document.activeElement) || document.activeElement === wrapper.get('.ip-panel').element).toBe(true)

    state.cancelImportPreview()
    await settle()
    expect(document.activeElement).toBe(opener)

    opener.focus()
    await state.openImportPreviewFromText('Ten', 'zh', '', 'noi dung', null)
    await settle()
    await wrapper.get('.ip-screen').trigger('keydown', { key: 'Escape' })
    await settle()
    expect(state.importPreviewIsOpen.value).toBe(false)
    expect(document.activeElement).toBe(opener)

    wrapper.unmount()
    state.resetImportPreview()
  })
})
