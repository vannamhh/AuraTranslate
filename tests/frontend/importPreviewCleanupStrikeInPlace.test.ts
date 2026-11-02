// Blocks are rendered from `BlockWire.cleanup_spans` as sent by Rust; no case computes offsets itself (AD-1).
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type {
  BlockWire,
  ChapterDetailWire,
  CleanupPreviewWire,
  CleanupRuleReportWire,
  CleanupSpanWire,
  EncodingCandidateWire,
  ImportEncodingPreview,
  UrlImportBatchWire,
} from '../../src/config/project'

const previewTextMock = vi.fn()
const cleanupSetEnabledMock = vi.fn()
const startUrlImportMock = vi.fn()
const previewChapterDetailMock = vi.fn()
const invokeMock = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invokeMock(...args) }))

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (text: string, sourceLang: string) => previewTextMock(text, sourceLang),
    cleanupSetEnabled: (tier: string, id: number, enabled: boolean) => cleanupSetEnabledMock(tier, id, enabled),
    startUrlImport: (urls: string[], sourceLang: string) => startUrlImportMock(urls, sourceLang),
    previewChapterDetail: (...args: unknown[]) => previewChapterDetailMock(...args),
  }
})

async function freshOverlay() {
  vi.resetModules()
  previewTextMock.mockReset()
  cleanupSetEnabledMock.mockReset()
  startUrlImportMock.mockReset()
  previewChapterDetailMock.mockReset()
  invokeMock.mockReset()
  const state = await import('../../src/importPreviewState')
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { state, ImportPreviewOverlay }
}

function rule(over: Partial<CleanupRuleReportWire> = {}): CleanupRuleReportWire {
  return {
    tier: 'global',
    id: 1,
    pattern: 'quang cao',
    kind: 'literal',
    enabled: true,
    count_in_chapter: 1,
    count_in_import: 1,
    ...over,
  }
}

function span(start: number, end: number, over: Partial<CleanupSpanWire> = {}): CleanupSpanWire {
  return { tier: 'global', id: 1, start, end, ...over }
}

function block(text: string, spans: CleanupSpanWire[] = [], over: Partial<BlockWire> = {}): BlockWire {
  return { body: { kind: 'paragraph', text }, kept: true, confirmed: false, cleanup_spans: spans, ...over }
}

function cleanupWire(over: Partial<CleanupPreviewWire> = {}): CleanupPreviewWire {
  return { text: '', spans: [], rules: [rule()], window_truncated: false, final_text: '', ...over }
}

function candidate(over: Partial<EncodingCandidateWire> = {}): EncodingCandidateWire {
  return {
    label: 'UTF-8',
    encoding: 'UTF-8',
    preview: 'x',
    normalized: { text: 'x', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
    cleanup: cleanupWire(),
    chapters: null,
    blocks: null,
    ...over,
  }
}

function previewOf(c: EncodingCandidateWire): ImportEncodingPreview {
  return {
    source_lang: 'en',
    source_lang_auto: false,
    confidence: 'high',
    selected_encoding: 'UTF-8',
    candidates: [c],
    self_declared_normalized: null,
    self_declared_cleanup: null,
    self_declared_chapters: null,
  }
}

function urlBatch(blocks: BlockWire[], rules: CleanupRuleReportWire[] = [rule()]): UrlImportBatchWire {
  return {
    items: [{ url: 'https://a.example/1', ok: true, error: null }],
    encoding_preview: previewOf(candidate({ cleanup: cleanupWire({ rules }), blocks: { blocks } })),
    domain_log_domain_count: 1,
    duplicate_urls_dropped: 0,
  }
}

async function openUrl(blocks: BlockWire[], rules: CleanupRuleReportWire[] = [rule()]) {
  const { state, ImportPreviewOverlay } = await freshOverlay()
  startUrlImportMock.mockResolvedValue({ batch: urlBatch(blocks, rules), error: null })
  await state.openImportPreviewFromUrls('Ten', 'en', '', ['https://a.example/1'], null)
  const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
  await wrapper.vm.$nextTick()
  return { state, wrapper }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('đường URL: mảnh gạch nằm trong khối', () => {
  it('mỗi khối vẽ mảnh gạch của CHÍNH nó, văn bản Chương chỉ hiện ở cột khối', async () => {
    const { state, wrapper } = await openUrl([
      block('dau quang cao cuoi', [span(4, 13)]),
      block('khong co gi'),
      block('hai quang cao', [span(4, 13)]),
    ])

    const texts = wrapper.findAll('.ip-block-text')
    expect(texts).toHaveLength(3)
    expect(texts[0]?.findAll('.ip-cleanup-struck')).toHaveLength(1)
    expect(texts[1]?.findAll('.ip-cleanup-struck')).toHaveLength(0)
    expect(texts[2]?.findAll('.ip-cleanup-struck')).toHaveLength(1)
    expect(texts[0]?.find('.ip-cleanup-struck').element.firstChild?.textContent).toBe('quang cao')
    expect(wrapper.findAll('.ip-cleanup-struck')).toHaveLength(2)
    expect(wrapper.find('.ip-cleanup-text').exists()).toBe(false)
    expect(wrapper.find('aside.ip-config .ip-cleanup-window-note').exists()).toBe(false)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('chỗ khớp vắt qua hai khối: hai mảnh, mỗi khối một mảnh', async () => {
    const { state, wrapper } = await openUrl([block('dau quang', [span(4, 9)]), block('cao cuoi', [span(0, 3)])])

    const texts = wrapper.findAll('.ip-block-text')
    expect(texts[0]?.find('.ip-cleanup-struck').element.firstChild?.textContent).toBe('quang')
    expect(texts[1]?.find('.ip-cleanup-struck').element.firstChild?.textContent).toBe('cao')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('khối loại và khối ảnh không span thì không gạch; khối giữ thì gạch', async () => {
    const { state, wrapper } = await openUrl([
      block('quang cao loai', [], { kept: false }),
      block('quang cao giu', [span(0, 9)]),
      { body: { kind: 'image', src: 'https://a.example/quang-cao.jpg', alt: 'quang cao' }, kept: true, confirmed: false, cleanup_spans: [] },
    ])

    expect(wrapper.findAll('.ip-cleanup-struck')).toHaveLength(1)
    expect(wrapper.find('.ip-block-text .ip-cleanup-struck').element.firstChild?.textContent).toBe('quang cao')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('hai luật chồng nhau: MỘT đoạn gạch, nhãn nêu cả hai luật', async () => {
    const { state, wrapper } = await openUrl(
      [block('dau quang cao cuoi', [span(4, 13, { id: 1 }), span(4, 13, { id: 2, tier: 'work' })])],
      [rule({ id: 1 }), rule({ id: 2, tier: 'work', pattern: 'q.*o', kind: 'regex' })],
    )

    const struck = wrapper.findAll('.ip-cleanup-struck')
    expect(struck).toHaveLength(1)
    expect(struck[0]?.find('.ip-cleanup-hint').text()).toBe('quang cao · Chuỗi con · Toàn cục; q.*o · Biểu thức chính quy · Tác phẩm')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('nhãn ẩn kiểu chỉ-đọc-màn-hình có mặt trong DOM, mảnh URL không nhận Tab', async () => {
    const { state, wrapper } = await openUrl([block('dau quang cao cuoi', [span(4, 13)])])

    const struck = wrapper.get('.ip-cleanup-struck')
    expect(struck.get('.ip-cleanup-hint').text()).toBe('quang cao · Chuỗi con · Toàn cục')
    expect(struck.attributes('tabindex')).toBeUndefined()

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('tắt luật: tải lại trả khối không span thì mảnh mất; bật lại thì mảnh hiện', async () => {
    const withSpan = [block('dau quang cao cuoi', [span(4, 13)])]
    const { state, wrapper } = await openUrl(withSpan)
    expect(wrapper.findAll('.ip-cleanup-struck')).toHaveLength(1)

    cleanupSetEnabledMock.mockResolvedValue({ ok: true, error: null })
    const off: ChapterDetailWire = {
      cleanup: cleanupWire({ rules: [rule({ enabled: false })] }),
      blocks: { blocks: [block('dau quang cao cuoi')] },
    }
    previewChapterDetailMock.mockResolvedValue({ detail: off, error: null })
    await state.toggleImportPreviewCleanupRule('global', 1, false)
    await wrapper.vm.$nextTick()
    expect(wrapper.findAll('.ip-cleanup-struck')).toHaveLength(0)

    const on: ChapterDetailWire = { cleanup: cleanupWire(), blocks: { blocks: withSpan } }
    previewChapterDetailMock.mockResolvedValue({ detail: on, error: null })
    await state.toggleImportPreviewCleanupRule('global', 1, true)
    await wrapper.vm.$nextTick()
    expect(wrapper.findAll('.ip-cleanup-struck')).toHaveLength(1)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('đổi luật lần đầu ở Chương 0: danh sách khối KHÔNG biến mất trong lúc chờ tải lại', async () => {
    const blocks = [block('mot quang cao', [span(4, 13)]), block('hai'), block('ba')]
    const { state, wrapper } = await openUrl(blocks)
    expect(wrapper.findAll('.ip-block')).toHaveLength(3)

    let release: (value: unknown) => void = () => {}
    previewChapterDetailMock.mockReturnValue(new Promise((resolve) => (release = resolve)))
    cleanupSetEnabledMock.mockResolvedValue({ ok: true, error: null })
    const pending = state.toggleImportPreviewCleanupRule('global', 1, false)
    await vi.waitFor(() => expect(previewChapterDetailMock).toHaveBeenCalled())
    await wrapper.vm.$nextTick()

    expect(state.importPreviewSelectedBlocks.value?.blocks).toHaveLength(3)
    expect(wrapper.findAll('.ip-block')).toHaveLength(3)
    expect(wrapper.findAll('.ip-cleanup-struck')).toHaveLength(1)

    release({
      detail: { cleanup: cleanupWire({ rules: [rule({ enabled: false })] }), blocks: { blocks: [block('mot quang cao'), block('hai'), block('ba')] } },
      error: null,
    })
    await pending
    await wrapper.vm.$nextTick()
    expect(wrapper.findAll('.ip-block')).toHaveLength(3)
    expect(wrapper.findAll('.ip-cleanup-struck')).toHaveLength(0)

    wrapper.unmount()
    state.resetImportPreview()
  })
})

describe('đường tệp và dán tay: văn bản Chương ở cột văn bản', () => {
  async function openText(cleanup: CleanupPreviewWire) {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: previewOf(candidate({ cleanup })), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'dau quang cao cuoi', null)
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    return { state, wrapper }
  }

  it('văn bản có gạch và câu cắt nằm ở tầng 2; tầng 3 chỉ còn danh sách luật', async () => {
    const { state, wrapper } = await openText(
      cleanupWire({ text: 'dau quang cao cuoi', spans: [span(4, 13)], window_truncated: true }),
    )

    const tier2 = wrapper.get('[aria-labelledby="ip-tier-2-title"]')
    const tier3 = wrapper.get('[aria-labelledby="ip-tier-3-title"]')
    expect(tier2.get('h4.ip-tier-title').text()).toBe('Văn bản Chương')
    expect(tier2.get('.ip-cleanup-window-note').text()).toContain('Chỉ xem trước phần đầu Chương')
    const struck = tier2.findAll('.ip-cleanup-struck')
    expect(struck).toHaveLength(1)
    expect(struck[0]?.element.firstChild?.textContent).toBe('quang cao')
    expect(struck[0]?.attributes('tabindex')).toBe('0')

    expect(tier3.find('.ip-cleanup-text').exists()).toBe(false)
    expect(tier3.find('.ip-cleanup-window-note').exists()).toBe(false)
    expect(tier3.find('.ip-cleanup-struck').exists()).toBe(false)
    expect(tier3.findAll('.ip-cleanup-rule')).toHaveLength(1)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('văn bản rỗng: câu rỗng ở tầng 2, không ở tầng 3', async () => {
    const { state, wrapper } = await openText(cleanupWire({ text: '', spans: [] }))

    const tier2 = wrapper.get('[aria-labelledby="ip-tier-2-title"]')
    const tier3 = wrapper.get('[aria-labelledby="ip-tier-3-title"]')
    expect(tier2.text()).toContain('Không có gì để hiện')
    expect(tier3.text()).not.toContain('Không có gì để hiện')

    wrapper.unmount()
    state.resetImportPreview()
  })
})

describe('hình dạng dây BlockWire', () => {
  it('khối thiếu `cleanup_spans` bị guard từ chối; có thì đi qua', async () => {
    vi.resetModules()
    invokeMock.mockReset()
    const { previewChapterDetail } = await vi.importActual<typeof import('../../src/config/project')>('../../src/config/project')
    const detail = (b: Record<string, unknown>) => ({
      cleanup: cleanupWire(),
      blocks: { blocks: [b] },
    })

    invokeMock.mockResolvedValue(detail({ body: { kind: 'paragraph', text: 'x' }, kept: true, confirmed: false }))
    const missing = await previewChapterDetail(0, 'UTF-8', 'en', null)
    expect(missing.detail).toBeNull()
    expect(missing.error).not.toBeNull()

    invokeMock.mockResolvedValue(
      detail({ body: { kind: 'paragraph', text: 'x' }, kept: true, confirmed: false, cleanup_spans: [span(0, 1)] }),
    )
    const ok = await previewChapterDetail(0, 'UTF-8', 'en', null)
    expect(ok.error).toBeNull()
    expect(ok.detail?.blocks?.blocks[0]?.cleanup_spans).toEqual([span(0, 1)])

    invokeMock.mockResolvedValue(
      detail({ body: { kind: 'paragraph', text: 'x' }, kept: true, confirmed: false, cleanup_spans: [{ tier: 'global', id: 1 }] }),
    )
    const badSpan = await previewChapterDetail(0, 'UTF-8', 'en', null)
    expect(badSpan.detail).toBeNull()
  })
})
