/**
 * Tầng tách Chương (tầng 4) của lớp phủ **Xem trước lượt nhập** — Story 6.6, FR14, AD-39
 * bước 5.
 *
 * ⚠️ Khuôn `importPreviewCleanup.test.ts`: `config/project.ts` là biên IPC, giả lập bằng
 * `vi.mock`, không gọi `@tauri-apps/api` thật.
 *
 * ⚠️ **Thứ tự bắt buộc trong mỗi ca**: `freshState()`/`freshOverlay()` TRƯỚC, cấu hình
 * `mockResolvedValue` SAU — cả hai tự `mockReset()` mọi mock.
 *
 * 🔴 Khác `importPreviewCleanup.test.ts` ở MỘT điểm có chủ ý: wrapper mock của
 * `previewImportEncodingFromText`/`_FromFile` ở đây forward ĐỦ BA tham số (kể cả
 * `chapterPattern`) — tệp này cần khẳng định CHÍNH tham số đó, các tệp khác không cần nên
 * wrapper của chúng cắt bớt cho gọn.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type {
  ChapterDetailWire,
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

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (...args: unknown[]) => previewTextMock(...args),
    previewImportEncodingFromFile: (...args: unknown[]) => previewFileMock(...args),
    confirmImportWithEncoding: (...args: unknown[]) => confirmMock(...args),
    startUrlImport: (...args: unknown[]) => startUrlImportMock(...args),
    previewChapterDetail: (...args: unknown[]) => previewChapterDetailMock(...args),
  }
})

async function freshState() {
  vi.resetModules()
  previewTextMock.mockReset()
  previewFileMock.mockReset()
  confirmMock.mockReset()
  startUrlImportMock.mockReset()
  previewChapterDetailMock.mockReset()
  return import('../../src/importPreviewState')
}

async function freshOverlay() {
  const state = await freshState()
  const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
  return { state, ImportPreviewOverlay }
}

function chapters(over: Partial<ChapterSplitPreviewWire> = {}): ChapterSplitPreviewWire {
  return {
    chapter_count: 2,
    chapters: [
      { ord: 1, title: 'Chuong 1: Mo Dau', length: 20, cleanup_match_count: 0 },
      { ord: 2, title: 'Chuong 2: Tiep Theo', length: 25, cleanup_match_count: 0 },
    ],
    ...over,
  }
}

function candidate(over: Partial<EncodingCandidateWire> = {}): EncodingCandidateWire {
  return {
    label: 'UTF-8',
    encoding: 'UTF-8',
    preview: 'Chuong 1: Mo Dau Chuong 2: Tiep Theo',
    normalized: {
      text: 'Chuong 1: Mo Dau Chuong 2: Tiep Theo',
      joined_lines: 0,
      blank_lines_removed: 0,
      window_truncated: false,
    },
    cleanup: {
      text: 'Chuong 1: Mo Dau Chuong 2: Tiep Theo',
      spans: [],
      rules: [],
      window_truncated: false,
      final_text: 'Chuong 1: Mo Dau Chuong 2: Tiep Theo',
    },
    chapters: chapters(),
    blocks: null,
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

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('importPreviewState — importPreviewSelectedChapters', () => {
  it('mặc định là khối tách Chương của ứng viên Rust đã chọn', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    expect(state.importPreviewSelectedChapters.value).toEqual(chapters())
  })

  it('đổi ứng viên đổi NGAY khối tách Chương, 0 lời gọi IPC thêm', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({
      preview: preview({
        candidates: [
          candidate({ encoding: 'UTF-8' }),
          candidate({
            label: 'GBK',
            encoding: 'GBK',
            chapters: chapters({
              chapter_count: 1,
              chapters: [{ ord: 1, title: null, length: 5, cleanup_match_count: 0 }],
            }),
          }),
        ],
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    const ipcCallsBefore =
      previewTextMock.mock.calls.length + previewFileMock.mock.calls.length + confirmMock.mock.calls.length

    state.selectImportPreviewCandidate('GBK')

    expect(state.importPreviewSelectedChapters.value?.chapter_count).toBe(1)
    expect(state.importPreviewSelectedChapters.value?.chapters[0]?.title).toBeNull()

    const ipcCallsAfter =
      previewTextMock.mock.calls.length + previewFileMock.mock.calls.length + confirmMock.mock.calls.length
    expect(ipcCallsAfter).toBe(ipcCallsBefore)
  })

  it('ứng viên "không ra chữ" (`chapters: null`) đọc ra `null`', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({
      preview: preview({
        candidates: [candidate({ preview: null, normalized: null, cleanup: null, chapters: null })],
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    expect(state.importPreviewSelectedChapters.value).toBeNull()
  })

  it('đường DÁN VĂN BẢN TAY (0 ứng viên) đọc `self_declared_chapters`, KHÔNG rơi về null', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({
      preview: preview({
        confidence: 'self_declared',
        candidates: [],
        self_declared_chapters: chapters({ chapter_count: 1, chapters: [{ ord: 1, title: null, length: 9, cleanup_match_count: 0 }] }),
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'van ban dan tay')

    expect(state.importPreviewSelectedCandidate.value).toBeNull()
    expect(state.importPreviewSelectedChapters.value?.chapter_count).toBe(1)
  })
})

describe('importPreviewState — setImportPreviewChapterPattern', () => {
  it('mẫu MỚI ⇒ đúng MỘT vòng IPC, gửi {pattern, kind} đúng hình dạng', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    previewTextMock.mockClear()

    previewTextMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: chapters({ chapter_count: 3 }) })] }), error: null })

    await state.setImportPreviewChapterPattern('第.*章', 'regex')

    expect(previewTextMock).toHaveBeenCalledTimes(1)
    expect(previewTextMock).toHaveBeenCalledWith('x', 'en', { pattern: '第.*章', kind: 'regex' })
    expect(state.importPreviewChapterPatternText.value).toBe('第.*章')
    expect(state.importPreviewChapterPatternKind.value).toBe('regex')
    expect(state.importPreviewSelectedChapters.value?.chapter_count).toBe(3)
  })

  it('mẫu KHÔNG đổi (cùng text, cùng kind) ⇒ 0 lời gọi IPC', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    previewTextMock.mockClear()

    await state.setImportPreviewChapterPattern('', 'literal') // giá trị mặc định lúc mở, không đổi gì

    expect(previewTextMock).not.toHaveBeenCalled()
  })

  it('giữ NGUYÊN ứng viên đang CHỌN TAY sau một lượt sửa mẫu', async () => {
    const state = await freshState()
    const twoCandidatesPreview = preview({
      selected_encoding: 'UTF-8',
      candidates: [candidate({ encoding: 'UTF-8' }), candidate({ encoding: 'GBK', label: 'GBK' })],
    })
    previewTextMock.mockResolvedValue({ preview: twoCandidatesPreview, error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    state.selectImportPreviewCandidate('GBK')
    expect(state.importPreviewSelectedEncoding.value).toBe('GBK')

    previewTextMock.mockResolvedValue({ preview: twoCandidatesPreview, error: null })
    await state.setImportPreviewChapterPattern('Chuong', 'literal')

    expect(state.importPreviewSelectedEncoding.value).toBe('GBK')
  })

  it('mẫu regex hỏng ⇒ GIỮ NGUYÊN kết quả CŨ, chỉ báo lỗi RIÊNG — không lật status/preview', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    previewTextMock.mockClear()

    const err = {
      code: 'import.invalid_chapter_pattern',
      message_key: 'err.import.invalid_chapter_pattern',
      params: {},
      retryable: false,
    }
    previewTextMock.mockResolvedValue({ preview: null, error: err })

    const oldChapters = state.importPreviewSelectedChapters.value

    await state.setImportPreviewChapterPattern('[unclosed', 'regex')

    expect(state.importPreviewChapterPatternError.value).toEqual(err)
    // §I/O Matrix spec 6.6 — "xem trước GIỮ kết quả CŨ": preview/status KHÔNG bị đụng.
    expect(state.importPreviewStatus.value).toBe('loaded')
    expect(state.importPreviewSelectedChapters.value).toEqual(oldChapters)
  })

  it('một `@change` thứ hai đến trong lúc lượt đầu còn bay ĐƯỢC XẾP HÀNG, không biến mất (vòng rà đối kháng 3, mục 4)', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    previewTextMock.mockClear()

    let resolveFirst!: (value: { preview: ImportEncodingPreview; error: null }) => void
    const firstCallPromise = new Promise<{ preview: ImportEncodingPreview; error: null }>((resolve) => {
      resolveFirst = resolve
    })
    previewTextMock.mockReturnValueOnce(firstCallPromise)
    previewTextMock.mockResolvedValueOnce({
      preview: preview({ candidates: [candidate({ chapters: chapters({ chapter_count: 5 }) })] }),
      error: null,
    })

    const firstSet = state.setImportPreviewChapterPattern('mau-mot', 'literal')
    // Lượt đầu đang bay — cờ sending phải true NGAY (đồng bộ, trước await đầu tiên).
    expect(state.importPreviewChapterPatternSending.value).toBe(true)

    const secondSet = state.setImportPreviewChapterPattern('mau-hai', 'literal')
    // Lượt hai KHÔNG được gọi IPC song song — chỉ ĐÚNG một lời gọi đã xảy ra (lượt đầu).
    expect(previewTextMock).toHaveBeenCalledTimes(1)

    resolveFirst({ preview: preview(), error: null })
    await firstSet
    await secondSet

    // Lượt hai không hề biến mất — nó tự chạy NGAY sau khi lượt đầu xong, đúng MỘT lời gọi
    // IPC thêm, với ĐÚNG giá trị SAU CÙNG người dùng đã gõ.
    expect(previewTextMock).toHaveBeenCalledTimes(2)
    expect(previewTextMock).toHaveBeenNthCalledWith(1, 'x', 'en', { pattern: 'mau-mot', kind: 'literal' })
    expect(previewTextMock).toHaveBeenNthCalledWith(2, 'x', 'en', { pattern: 'mau-hai', kind: 'literal' })
    expect(state.importPreviewChapterPatternText.value).toBe('mau-hai')
    expect(state.importPreviewSelectedChapters.value?.chapter_count).toBe(5)
    expect(state.importPreviewChapterPatternSending.value).toBe(false)
  })

  it('xoá sạch ô mẫu (rỗng) ⇒ vẫn MỘT vòng IPC, gửi `chapterPattern: null`', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    await state.setImportPreviewChapterPattern('Chuong', 'literal')
    previewTextMock.mockClear()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })

    await state.setImportPreviewChapterPattern('', 'literal')

    expect(previewTextMock).toHaveBeenCalledTimes(1)
    expect(previewTextMock).toHaveBeenCalledWith('x', 'en', null)
  })

  it('ô CHỈ CÓ khoảng trắng gửi `chapterPattern: null` — cùng quy ước `.trim()` của hai ô luật làm sạch (vòng rà đối kháng 3, mục 5)', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    previewTextMock.mockClear()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })

    await state.setImportPreviewChapterPattern('   ', 'literal')

    expect(previewTextMock).toHaveBeenCalledTimes(1)
    expect(previewTextMock).toHaveBeenCalledWith('x', 'en', null)
  })

  it('mở một lượt xem trước MỚI reset mẫu về rỗng — KHÔNG nhớ mẫu giữa hai lượt nhập (§Ask First spec 6.6)', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    await state.setImportPreviewChapterPattern('Chuong', 'literal')
    expect(state.importPreviewChapterPatternText.value).toBe('Chuong')

    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten Khac', 'en', '', 'y')

    expect(state.importPreviewChapterPatternText.value).toBe('')
    expect(state.importPreviewChapterPatternKind.value).toBe('literal')
    expect(state.importPreviewChapterPatternError.value).toBeNull()
  })
})

describe('ImportPreviewOverlay.vue — tầng 4 dựng đúng danh sách, sắp xếp, và gửi mẫu', () => {
  it('hiện số Chương nhận ra, ord, title, độ dài', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    expect(wrapper.find('.ip-chapters-count').text()).toContain('2')
    const rows = wrapper.findAll('.ip-chapters-entry')
    expect(rows).toHaveLength(2)
    expect(rows[0]?.find('.ip-chapters-title').text()).toBe('Chuong 1: Mo Dau')
    expect(rows[1]?.find('.ip-chapters-title').text()).toBe('Chuong 2: Tiep Theo')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('Chương không có tiêu đề hiện nhãn thay thế, không phải chuỗi rỗng', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({
      preview: preview({
        candidates: [
          candidate({
            chapters: chapters({ chapter_count: 1, chapters: [{ ord: 1, title: null, length: 800, cleanup_match_count: 0 }] }),
          }),
        ],
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    expect(wrapper.find('.ip-chapters-title-none').exists()).toBe(true)
    expect(wrapper.find('.ip-chapters-title-none').text().length).toBeGreaterThan(0)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('mặc định KHÔNG sắp xếp; bấm sắp xếp theo độ dài đưa Chương NGẮN NHẤT lên đầu', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({
      preview: preview({
        candidates: [
          candidate({
            chapters: chapters({
              chapter_count: 3,
              chapters: [
                { ord: 1, title: 'Dai', length: 4000, cleanup_match_count: 0 },
                { ord: 2, title: 'Ngan Bat Thuong', length: 40, cleanup_match_count: 0 },
                { ord: 3, title: 'Dai Nua', length: 3800, cleanup_match_count: 0 },
              ],
            }),
          }),
        ],
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    // Mặc định — thứ tự NGUYÊN VĂN (N ≤ 6 nên hiện trọn, không cắt).
    let rows = wrapper.findAll('.ip-chapters-entry')
    expect(rows[0]?.find('.ip-chapters-title').text()).toBe('Dai')

    await wrapper.find('.ip-chapters-sort-toggle input').setValue(true)

    rows = wrapper.findAll('.ip-chapters-entry')
    expect(rows[0]?.find('.ip-chapters-title').text()).toBe('Ngan Bat Thuong')

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('N > 6 Chương ⇒ khung nhìn mặc định chỉ hiện ba đầu, `⋯`, ba cuối', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    const many = Array.from({ length: 9 }, (_, i) => ({ ord: i + 1, title: `Chuong ${i + 1}`, length: 100 + i, cleanup_match_count: 0 }))
    previewTextMock.mockResolvedValue({
      preview: preview({
        candidates: [candidate({ chapters: chapters({ chapter_count: 9, chapters: many }) })],
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    const rows = wrapper.findAll('.ip-chapters-entry')
    expect(rows).toHaveLength(6)
    expect(rows[0]?.find('.ip-chapters-title').text()).toBe('Chuong 1')
    expect(rows[2]?.find('.ip-chapters-title').text()).toBe('Chuong 3')
    expect(rows[3]?.find('.ip-chapters-title').text()).toBe('Chuong 7')
    expect(rows[5]?.find('.ip-chapters-title').text()).toBe('Chuong 9')
    expect(wrapper.find('.ip-chapters-ellipsis').exists()).toBe(true)

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('gõ mẫu rồi rời ô (`@change`) gọi lại xem trước với ĐÚNG {pattern, kind}', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    previewTextMock.mockClear()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.find('.ip-chapters-pattern-input').setValue('Chuong')
    await wrapper.find('.ip-chapters-pattern-input').trigger('change')

    expect(previewTextMock).toHaveBeenCalledWith('x', 'en', { pattern: 'Chuong', kind: 'literal' })

    wrapper.unmount()
    state.resetImportPreview()
  })

  it('mẫu hỏng hiện thông báo lỗi RIÊNG mà KHÔNG xoá danh sách Chương đang hiện', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')
    previewTextMock.mockClear()
    previewTextMock.mockResolvedValue({
      preview: null,
      error: {
        code: 'import.invalid_chapter_pattern',
        message_key: 'err.import.invalid_chapter_pattern',
        params: {},
        retryable: false,
      },
    })

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.find('.ip-chapters-pattern-input').setValue('[unclosed')
    await wrapper.find('.ip-chapters-pattern-input').trigger('change')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.ip-chapters-error').exists()).toBe(true)
    // Danh sách CŨ vẫn còn — không bị "vỡ trắng" ra một thông báo lỗi thay thế toàn bộ tầng.
    expect(wrapper.findAll('.ip-chapters-entry')).toHaveLength(2)

    wrapper.unmount()
    state.resetImportPreview()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.10a — con trỏ *Chương đang chọn* (`⌥←`/`⌥→`). Chỉ có nghĩa trên đường URL
// (`PipelineShape::Chapters`) — khuôn `importPreviewUrls.test.ts` cho fixture URL.
// ═════════════════════════════════════════════════════════════════════════════════

function urlItem(url: string): UrlImportItemWire {
  return { url, ok: true, error: null }
}

/** Ba Chương thật (N = 3), mỗi Chương một bộ số tóm tắt RIÊNG — cùng khuôn đối chứng Rust
 * `cleanup_and_chapters_preview_for_returns_the_summary_of_every_chapter_and_the_detail_of_the_chosen_one`. */
function threeChapters(): ChapterSplitPreviewWire {
  return {
    chapter_count: 3,
    chapters: [
      { ord: 1, title: 'Chuong 1', length: 100, cleanup_match_count: 1 },
      { ord: 2, title: 'Chuong 2', length: 200, cleanup_match_count: 2 },
      { ord: 3, title: 'Chuong 3', length: 300, cleanup_match_count: 3 },
    ],
  }
}

function urlBatch(): UrlImportBatchWire {
  const urls = ['https://a.example/1', 'https://a.example/2', 'https://a.example/3']
  const encodingPreview: ImportEncodingPreview = preview({
    candidates: [
      candidate({
        cleanup: {
          text: 'chuong 0',
          spans: [],
          rules: [],
          window_truncated: false,
          final_text: 'chuong 0',
        },
        blocks: { blocks: [] },
        chapters: threeChapters(),
      }),
    ],
  })
  return {
    items: urls.map(urlItem),
    encoding_preview: encodingPreview,
    domain_log_domain_count: 1,
  }
}

/** Cùng `urlBatch()` nhưng HAI ứng viên bảng mã — cho ca AC "đổi ứng viên khi con trỏ ở
 * Chương k > 0 không nhảy về Chương 0". */
function urlBatchTwoCandidates(): UrlImportBatchWire {
  const urls = ['https://a.example/1', 'https://a.example/2', 'https://a.example/3']
  const encodingPreview: ImportEncodingPreview = preview({
    candidates: [
      candidate({
        encoding: 'UTF-8',
        cleanup: { text: 'utf8 chuong 0', spans: [], rules: [], window_truncated: false, final_text: 'utf8 chuong 0' },
        blocks: { blocks: [] },
        chapters: threeChapters(),
      }),
      candidate({
        label: 'GBK',
        encoding: 'GBK',
        cleanup: { text: 'gbk chuong 0', spans: [], rules: [], window_truncated: false, final_text: 'gbk chuong 0' },
        blocks: { blocks: [] },
        chapters: threeChapters(),
      }),
    ],
  })
  return {
    items: urls.map(urlItem),
    encoding_preview: encodingPreview,
    domain_log_domain_count: 1,
  }
}

function chapterDetail(label: string): ChapterDetailWire {
  return {
    cleanup: { text: label, spans: [], rules: [], window_truncated: false, final_text: label },
    blocks: { blocks: [] },
  }
}

describe('importPreviewState — con trỏ Chương (Story 6.10a)', () => {
  it('mặc định con trỏ ở Chương 0, đọc thẳng chi tiết của ứng viên (0 lời gọi IPC lazy)', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])

    expect(state.importPreviewChapterCursor.value).toBe(0)
    expect(state.importPreviewSelectedCleanup.value?.final_text).toBe('chuong 0')
    expect(previewChapterDetailMock).not.toHaveBeenCalled()

    state.resetImportPreview()
  })

  it('⌥→ dời con trỏ sang Chương 1, dựng chi tiết LAZY qua đúng một lời gọi IPC', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])
    previewChapterDetailMock.mockResolvedValue({ detail: chapterDetail('chuong 1'), error: null })

    state.nextImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(state.importPreviewChapterCursor.value).toBe(1)
    expect(previewChapterDetailMock).toHaveBeenCalledTimes(1)
    expect(previewChapterDetailMock).toHaveBeenCalledWith(1, 'UTF-8', 'en', null)
    expect(state.importPreviewSelectedCleanup.value?.final_text).toBe('chuong 1')

    state.resetImportPreview()
  })

  it('P4 (vòng rà đối kháng bước 4) — luôn gửi chapterPattern: null, kể cả khi ô mẫu đang gõ khác rỗng', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])
    // Ô mẫu phân tách ĐANG GÕ khác rỗng — đường eager (`url_import_encoding_preview` phía
    // Rust) truyền `chapter_pattern: None` CỨNG cho MỌI ứng viên trên đường URL bất kể ô này;
    // lệnh lazy phải khớp NGUYÊN VĂN, không được gửi mẫu đang gõ.
    await state.setImportPreviewChapterPattern('Chuong', 'literal')
    previewChapterDetailMock.mockResolvedValue({ detail: chapterDetail('chuong 1'), error: null })

    state.nextImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(previewChapterDetailMock).toHaveBeenCalledWith(1, 'UTF-8', 'en', null)

    state.resetImportPreview()
  })

  it('P2 (vòng rà đối kháng bước 4) — lỗi phải DỌN chi tiết đang hiện, không giữ Chương cũ dưới nhãn Chương mới', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])
    previewChapterDetailMock.mockResolvedValue({ detail: chapterDetail('chuong 1'), error: null })
    state.nextImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(state.importPreviewSelectedCleanup.value?.final_text).toBe('chuong 1')

    // Lượt dựng chi tiết KẾ TIẾP (Chương 2) trượt — chi tiết của Chương 1 KHÔNG được phép
    // tiếp tục hiện dưới nhãn "Chương 2".
    previewChapterDetailMock.mockResolvedValue({
      detail: null,
      error: { code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false },
    })
    state.nextImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(state.importPreviewChapterCursor.value).toBe(2)
    expect(state.importPreviewSelectedCleanup.value).toBeNull()
    expect(state.importPreviewChapterDetailError.value).not.toBeNull()

    state.resetImportPreview()
  })

  it('P2 (vòng rà đối kháng bước 4) — trạng thái CŨ (detail: null) cũng phải DỌN chi tiết đang hiện', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])
    previewChapterDetailMock.mockResolvedValue({ detail: chapterDetail('chuong 1'), error: null })
    state.nextImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(state.importPreviewSelectedCleanup.value?.final_text).toBe('chuong 1')

    // Mẫu phân tách vừa đổi làm N đổi dưới chân — Rust trả trạng thái CŨ (`detail: null`,
    // `error: null`), KHÔNG được giữ lại chi tiết của Chương 1 dưới nhãn Chương 2.
    previewChapterDetailMock.mockResolvedValue({ detail: null, error: null })
    state.nextImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(state.importPreviewChapterCursor.value).toBe(2)
    expect(state.importPreviewSelectedCleanup.value).toBeNull()

    state.resetImportPreview()
  })

  it('P3 (vòng rà đối kháng bước 4) — hai lượt gọi CÙNG index, lượt ĐẦU resolve SAU lượt SAU: dữ liệu hiện ra phải là của lượt SAU', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoCandidates(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])

    let resolveFirst!: (v: { detail: ChapterDetailWire | null; error: null }) => void
    let resolveSecond!: (v: { detail: ChapterDetailWire | null; error: null }) => void
    previewChapterDetailMock
      .mockImplementationOnce(() => new Promise((resolve) => (resolveFirst = resolve)))
      .mockImplementationOnce(() => new Promise((resolve) => (resolveSecond = resolve)))

    // Hai lượt gọi liên tiếp CHO CÙNG index 1 (đổi ứng viên bảng mã hai lần liên tiếp trong
    // lúc con trỏ đứng yên ở 1 — mô phỏng bằng cách gọi thẳng `selectImportPreviewCandidate`
    // hai lần, KHÔNG dời con trỏ giữa hai lượt).
    state.nextImportPreviewChapter() // 0 -> 1, lượt gọi ĐẦU (index 1)
    await Promise.resolve()
    state.selectImportPreviewCandidate('GBK') // vẫn index 1, lượt gọi SAU (index 1)
    await Promise.resolve()

    expect(previewChapterDetailMock).toHaveBeenCalledTimes(2)

    // Lượt SAU (thứ hai) resolve TRƯỚC.
    resolveSecond({ detail: chapterDetail('tu luot sau'), error: null })
    await new Promise((resolve) => setTimeout(resolve, 0))
    // Lượt ĐẦU (thứ nhất, CŨ HƠN) resolve SAU — kết quả của nó phải bị BỎ, không được ghi đè
    // dữ liệu của lượt sau.
    resolveFirst({ detail: chapterDetail('tu luot dau, cu hon'), error: null })
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(state.importPreviewSelectedCleanup.value?.final_text).toBe('tu luot sau')

    state.resetImportPreview()
  })

  it('dừng ở Chương cuối — không kêu, không lời gọi IPC', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])
    previewChapterDetailMock.mockResolvedValue({ detail: chapterDetail('x'), error: null })

    state.nextImportPreviewChapter() // 0 -> 1
    await new Promise((resolve) => setTimeout(resolve, 0))
    state.nextImportPreviewChapter() // 1 -> 2 (Chương cuối, index 2 của 3 Chương)
    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(state.importPreviewChapterCursor.value).toBe(2)
    previewChapterDetailMock.mockClear()

    state.nextImportPreviewChapter() // đứng yên
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(state.importPreviewChapterCursor.value).toBe(2)
    expect(previewChapterDetailMock).not.toHaveBeenCalled()

    state.resetImportPreview()
  })

  it('dừng ở Chương đầu — ⌥← ở Chương 0 đứng yên, không lời gọi IPC', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])

    state.prevImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(state.importPreviewChapterCursor.value).toBe(0)
    expect(previewChapterDetailMock).not.toHaveBeenCalled()

    state.resetImportPreview()
  })

  it('lớp phủ đã đóng — dời con trỏ là no-op tuyệt đối', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])
    state.resetImportPreview() // đóng lớp phủ, cursor về 0

    state.nextImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(state.importPreviewChapterCursor.value).toBe(0)
    expect(previewChapterDetailMock).not.toHaveBeenCalled()
  })

  // 🔵 **SỬA 2026-09-08 (vòng nghiệm thu) — tên cũ viện dẫn SAI thẩm quyền.** Ca này từng tên
  // "… đúng I/O Matrix", nhưng hàng 1 của ma trận spec 6.10a lúc đó đòi ĐIỀU NGƯỢC LẠI (tầng 3
  // theo Chương đang chọn trên chính đường `Blob` + mẫu phân tách) — tức một ca XANH khoá chặt
  // một hành vi TRÁI hợp đồng đã ký, và tự xưng là hợp đồng cho phép. Thẩm quyền THẬT của hành
  // vi này là một giới hạn của pipeline, không phải một điều khoản của ma trận: `pipeline.rs`
  // §`split_chapters_step` — bước làm sạch chạy trên TOÀN blob TRƯỚC bước tách Chương, nên chỉ
  // `ord = 1` có `CleanupReport` thật. Ice đã sửa hàng 1 cho khai đúng giới hạn đó (2026-09-08);
  // tên ca nay trỏ vào NGUYÊN NHÂN, không trỏ vào một tài liệu nói ngược.
  it('đường tệp/dán tay (N > 1 do mẫu phân tách) — con trỏ KHÔNG đi đâu được: `Blob` chỉ có MỘT báo cáo làm sạch, ở `ord = 1`', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview(), error: null }) // N = 2, đường text
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    state.nextImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(state.importPreviewChapterCursor.value).toBe(0)
    expect(previewChapterDetailMock).not.toHaveBeenCalled()

    state.resetImportPreview()
  })

  it('AC — đổi ứng viên bảng mã khi con trỏ ở Chương k > 0: hiện Chương k, KHÔNG nhảy về Chương 0', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({ batch: urlBatchTwoCandidates(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])
    expect(state.importPreviewSelectedEncoding.value).toBe('UTF-8')

    // Dời con trỏ sang Chương 1 (chỉ số 1) trên ứng viên UTF-8.
    previewChapterDetailMock.mockResolvedValue({ detail: chapterDetail('chuong 1 utf8'), error: null })
    state.nextImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(state.importPreviewChapterCursor.value).toBe(1)
    expect(state.importPreviewSelectedCleanup.value?.final_text).toBe('chuong 1 utf8')

    // Đổi ứng viên sang GBK — con trỏ PHẢI giữ nguyên ở 1, và chi tiết dựng lại VỚI bảng mã MỚI.
    previewChapterDetailMock.mockClear()
    previewChapterDetailMock.mockResolvedValue({ detail: chapterDetail('chuong 1 gbk'), error: null })
    state.selectImportPreviewCandidate('GBK')
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(state.importPreviewSelectedEncoding.value).toBe('GBK')
    expect(state.importPreviewChapterCursor.value).toBe(1) // KHÔNG nhảy về 0
    expect(previewChapterDetailMock).toHaveBeenCalledTimes(1)
    expect(previewChapterDetailMock).toHaveBeenCalledWith(1, 'GBK', 'en', null)
    expect(state.importPreviewSelectedCleanup.value?.final_text).toBe('chuong 1 gbk')

    state.resetImportPreview()
  })

  it('DOM THẬT — dời con trỏ đổi `aria-selected`/`aria-activedescendant` của tầng 4, đúng khuôn `blocksList`', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    startUrlImportMock.mockResolvedValue({ batch: urlBatch(), error: null })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])
    previewChapterDetailMock.mockResolvedValue({ detail: chapterDetail('chuong 1'), error: null })

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const list = wrapper.get('.ip-chapters-list')
    expect(list.attributes('aria-activedescendant')).toBe('ip-chapter-1')
    const rows = wrapper.findAll('.ip-chapters-entry')
    expect(rows[0]?.attributes('aria-selected')).toBe('true')
    expect(rows[1]?.attributes('aria-selected')).toBe('false')

    state.nextImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()

    expect(list.attributes('aria-activedescendant')).toBe('ip-chapter-2')
    const rowsAfter = wrapper.findAll('.ip-chapters-entry')
    expect(rowsAfter[0]?.attributes('aria-selected')).toBe('false')
    expect(rowsAfter[1]?.attributes('aria-selected')).toBe('true')
    expect(document.activeElement).toBe(list.element)

    wrapper.unmount()
    state.resetImportPreview()
  })
})
