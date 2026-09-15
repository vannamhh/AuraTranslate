/**
 * Xuất xứ tài liệu ở màn xem trước nhập — Story 6.15, FR128/AD-43.
 *
 * ⚠️ Khuôn `importPreviewChapters.test.ts`/`importPreviewUrls.test.ts`: `config/project.ts` là
 * biên IPC, giả lập bằng `vi.mock`, không gọi `@tauri-apps/api` thật.
 *
 * ⚠️ **Thứ tự bắt buộc trong mỗi ca**: `freshState()` TRƯỚC, cấu hình `mockResolvedValue` SAU.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type {
  ChapterDetailWire,
  ChapterOriginWire,
  ChapterSplitPreviewWire,
  EncodingCandidateWire,
  ImportEncodingPreview,
  UrlImportBatchWire,
  UrlImportItemWire,
} from '../../src/config/project'

const previewTextMock = vi.fn()
const previewFileMock = vi.fn()
const confirmMock = vi.fn()
const startUrlImportMock = vi.fn()
const previewChapterDetailMock = vi.fn()
const setChapterOriginOverrideMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (...args: unknown[]) => previewTextMock(...args),
    previewImportEncodingFromFile: (...args: unknown[]) => previewFileMock(...args),
    confirmImportWithEncoding: (...args: unknown[]) => confirmMock(...args),
    startUrlImport: (...args: unknown[]) => startUrlImportMock(...args),
    previewChapterDetail: (...args: unknown[]) => previewChapterDetailMock(...args),
    setChapterOriginOverride: (...args: unknown[]) => setChapterOriginOverrideMock(...args),
  }
})

async function freshState() {
  vi.resetModules()
  previewTextMock.mockReset()
  previewFileMock.mockReset()
  confirmMock.mockReset()
  startUrlImportMock.mockReset()
  previewChapterDetailMock.mockReset()
  setChapterOriginOverrideMock.mockReset()
  setChapterOriginOverrideMock.mockResolvedValue({ ok: true, error: null })
  return import('../../src/importPreviewState')
}

async function freshOverlay() {
  const state = await freshState()
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { state, ImportPreviewOverlay }
}

function origin(over: Partial<ChapterOriginWire> = {}): ChapterOriginWire {
  return {
    author: null,
    site_name: null,
    url: null,
    published_at: null,
    author_confirmed: false,
    site_name_confirmed: false,
    url_confirmed: false,
    published_at_confirmed: false,
    ...over,
  }
}

function chaptersWithOrigin(origins: ChapterOriginWire[]): ChapterSplitPreviewWire {
  return {
    chapter_count: origins.length,
    chapters: origins.map((o, i) => ({
      ord: i + 1,
      title: null,
      length: 10,
      cleanup_match_count: 0,
      joined_line_count_in_chapter: null,
      needs_review: false,
      review_causes: [],
      origin: o,
      source_file: null,
    })),
    broken_item_count: 0,
    needs_review_count: 0,
    clean_count: origins.length,
    any_signal_participated: false,
  }
}

function candidate(over: Partial<EncodingCandidateWire> = {}): EncodingCandidateWire {
  return {
    label: 'UTF-8',
    encoding: 'UTF-8',
    preview: 'noi dung',
    normalized: { text: 'noi dung', joined_lines: 0, blank_lines_removed: 0, window_truncated: false },
    cleanup: { text: 'noi dung', spans: [], rules: [], window_truncated: false, final_text: 'noi dung' },
    chapters: chaptersWithOrigin([origin(), origin()]),
    blocks: { blocks: [] },
    ...over,
  }
}

function preview(over: Partial<ImportEncodingPreview> = {}): ImportEncodingPreview {
  return {
    confidence: 'high',
    selected_encoding: 'UTF-8',
    candidates: [candidate()],
    self_declared_normalized: null,
    self_declared_cleanup: null,
    self_declared_chapters: null,
    ...over,
  }
}

function urlItem(url: string): UrlImportItemWire {
  return { url, ok: true, error: null }
}

/** Hai link ⇒ hai Chương, Chương 0 đã có xuất xứ MÁY bóc, Chương 1 khai thiếu (chỉ author). */
function urlBatchTwoChapters(): UrlImportBatchWire {
  const urls = ['https://a.example/1', 'https://a.example/2']
  const encodingPreview = preview({
    candidates: [
      candidate({
        chapters: chaptersWithOrigin([
          origin({ author: 'Nguyen Van A', site_name: 'Bao Thi Du', url: urls[0], published_at: '2026-09-10' }),
          origin({ url: urls[1] }),
        ]),
      }),
    ],
  })
  return { items: urls.map(urlItem), encoding_preview: encodingPreview, domain_log_domain_count: 2 }
}

function chapterDetail(label: string): ChapterDetailWire {
  return {
    cleanup: { text: label, spans: [], rules: [], window_truncated: false, final_text: label },
    blocks: { blocks: [] },
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('importPreviewState — importPreviewCurrentChapterOrigin (Story 6.15)', () => {
  it('mặc định phản ánh xuất xứ MÁY đã bóc của Chương con trỏ đang chọn (Chương 0)', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoChapters(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['https://a.example/1', 'https://a.example/2'])

    expect(state.importPreviewCurrentChapterOrigin.value).toEqual(
      origin({ author: 'Nguyen Van A', site_name: 'Bao Thi Du', url: 'https://a.example/1', published_at: '2026-09-10' }),
    )
  })

  it('Chương khai thiếu ⇒ trường vắng là `null` ("không tìm thấy" phía hiển thị)', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoChapters(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['https://a.example/1', 'https://a.example/2'])

    previewChapterDetailMock.mockResolvedValue({ detail: chapterDetail('chuong 1'), error: null })
    state.nextImportPreviewChapter()
    await Promise.resolve()
    await Promise.resolve()

    expect(state.importPreviewCurrentChapterOrigin.value.author).toBeNull()
    expect(state.importPreviewCurrentChapterOrigin.value.url).toBe('https://a.example/2')
  })

  it('gõ đè ⇒ draft THẮNG giá trị máy NGAY, và ghi xuống `ChapterOriginOverridesState` qua IPC', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoChapters(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['https://a.example/1', 'https://a.example/2'])

    await state.commitImportPreviewChapterOrigin({
      author: 'Nguoi Dung Go Tay',
      siteName: 'Bao Thi Du',
      url: 'https://a.example/1',
      publishedAt: '2026-09-10',
    })

    expect(state.importPreviewCurrentChapterOrigin.value.author).toBe('Nguoi Dung Go Tay')
    expect(setChapterOriginOverrideMock).toHaveBeenCalledWith(0, {
      author: 'Nguoi Dung Go Tay',
      siteName: 'Bao Thi Du',
      url: 'https://a.example/1',
      publishedAt: '2026-09-10',
    })
  })

  // AI-7 — bảng đầy đủ của §I/O Matrix spec AI-7: `.trim()` GỐC của JS (không mock, không tái
  // hiện) là THAM CHIẾU cho luật cắt phía Rust — mỗi hàng ở đây phải khớp ĐÚNG hàng cùng tên ở
  // `chapter_origin_contract.rs::an_override_cleared_to_whitespace_only_values_matches_the_io_matrix`.
  const FEFF = String.fromCodePoint(0xfeff)
  const NEL = String.fromCodePoint(0x0085)
  const NBSP = String.fromCodePoint(0x00a0)
  const LINE_SEPARATOR = String.fromCodePoint(0x2028)

  it.each([
    ['chuoi rong', '', true],
    ['chi dau cach ASCII', '   ', true],
    ['chi BOM -- seam do duoc 2026-09-07', FEFF, true],
    ['BOM cong chu that -- chi hai dau bi cat', FEFF + 'Tấn Giang', false],
    ['BOM o giua la NOI DUNG, khong bi cat', 'Tấn' + FEFF + 'Giang', false],
    ['khoang trang hon hop hai dau', '\t' + NBSP + 'x' + LINE_SEPARATOR + ' ', false],
    ['chi NEL -- seam NGUOC, D1: JS .trim() khong cat NEL', NEL, false],
    ['NEL dau + chu that -- NEL la NOI DUNG duoi tap cua JS', NEL + 'Tấn Giang', false],
  ] as const)('%s ⇒ author null=%s (khớp `.trim()` gốc JS)', async (_label, input, expectNull) => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoChapters(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['https://a.example/1', 'https://a.example/2'])

    await state.commitImportPreviewChapterOrigin({ author: input, siteName: '', url: '', publishedAt: '' })

    if (expectNull) {
      expect(state.importPreviewCurrentChapterOrigin.value.author).toBeNull()
    } else {
      expect(state.importPreviewCurrentChapterOrigin.value.author).not.toBeNull()
    }
  })

  it('draft SỐNG QUA lượt đổi bảng mã (đổi ứng viên KHÔNG thổi bay chữ đã gõ)', async () => {
    const state = await freshState()
    const batch = urlBatchTwoChapters()
    const withSecondCandidate: UrlImportBatchWire = {
      ...batch,
      encoding_preview: preview({
        candidates: [
          ...(batch.encoding_preview?.candidates ?? []),
          candidate({ encoding: 'GBK', label: 'GBK' }),
        ],
      }),
    }
    startUrlImportMock.mockResolvedValue({ batch: withSecondCandidate, error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['https://a.example/1', 'https://a.example/2'])

    await state.commitImportPreviewChapterOrigin({
      author: 'Chu Da Go',
      siteName: '',
      url: '',
      publishedAt: '',
    })
    expect(state.importPreviewCurrentChapterOrigin.value.author).toBe('Chu Da Go')

    state.selectImportPreviewCandidate('GBK')

    expect(state.importPreviewCurrentChapterOrigin.value.author).toBe('Chu Da Go')
  })

  it('draft theo TỪNG Chương — dời con trỏ sang Chương khác rồi quay lại vẫn giữ đúng chữ đã gõ ở mỗi Chương', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoChapters(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['https://a.example/1', 'https://a.example/2'])

    await state.commitImportPreviewChapterOrigin({ author: 'Chuong Khong', siteName: '', url: '', publishedAt: '' })

    previewChapterDetailMock.mockResolvedValue({ detail: chapterDetail('chuong 1'), error: null })
    state.nextImportPreviewChapter()
    await Promise.resolve()
    await Promise.resolve()
    expect(state.importPreviewCurrentChapterOrigin.value.author).toBeNull()

    await state.commitImportPreviewChapterOrigin({ author: 'Chuong Mot', siteName: '', url: '', publishedAt: '' })
    expect(state.importPreviewCurrentChapterOrigin.value.author).toBe('Chuong Mot')

    state.prevImportPreviewChapter()
    expect(state.importPreviewCurrentChapterOrigin.value.author).toBe('Chuong Khong')

    expect(setChapterOriginOverrideMock).toHaveBeenCalledWith(0, expect.objectContaining({ author: 'Chuong Khong' }))
    expect(setChapterOriginOverrideMock).toHaveBeenCalledWith(1, expect.objectContaining({ author: 'Chuong Mot' }))
  })

  it('huỷ lớp phủ ⇒ draft bị dọn sạch (0 Tác phẩm được tạo, cùng hàng I/O Matrix)', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoChapters(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['https://a.example/1', 'https://a.example/2'])
    await state.commitImportPreviewChapterOrigin({ author: 'Se Bi Huy', siteName: '', url: '', publishedAt: '' })

    state.cancelImportPreview()

    expect(state.importPreviewCurrentChapterOrigin.value).toEqual(origin())
  })

  it('mở lượt xem trước MỚI (URL khác) ⇒ draft của lượt CŨ không rò sang', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoChapters(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['https://a.example/1', 'https://a.example/2'])
    await state.commitImportPreviewChapterOrigin({ author: 'Cua Lot Cu', siteName: '', url: '', publishedAt: '' })

    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoChapters(), error: null })
    await state.openImportPreviewFromUrls('Ten Khac', 'en', '', ['https://b.example/1', 'https://b.example/2'])

    expect(state.importPreviewCurrentChapterOrigin.value.author).toBe('Nguyen Van A')
  })
})

describe('ImportPreviewOverlay.vue — khối xuất xứ (mount thật, Story 6.15)', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  beforeEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('hiện ĐÚNG bốn ô, giá trị khớp Chương con trỏ đang chọn, chỉ trên đường URL', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoChapters(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['https://a.example/1', 'https://a.example/2'])

    wrapper = mount(ImportPreviewOverlay)
    await wrapper.vm.$nextTick()

    const inputs = wrapper.findAll('.chapter-origin-input')
    expect(inputs).toHaveLength(4)
    expect((inputs[0]?.element as HTMLInputElement).value).toBe('Nguyen Van A')
  })

  it('gõ đè rồi `change` ⇒ ghi vào draft VÀ gọi `setChapterOriginOverride`', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoChapters(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['https://a.example/1', 'https://a.example/2'])

    wrapper = mount(ImportPreviewOverlay)
    await wrapper.vm.$nextTick()

    const authorInput = wrapper.find('.chapter-origin-input')
    await authorInput.setValue('Go De Tren Man Xem Truoc')
    await wrapper.vm.$nextTick()

    expect(setChapterOriginOverrideMock).toHaveBeenCalledWith(
      0,
      expect.objectContaining({ author: 'Go De Tren Man Xem Truoc' }),
    )
    expect(state.importPreviewCurrentChapterOrigin.value.author).toBe('Go De Tren Man Xem Truoc')
  })
})
