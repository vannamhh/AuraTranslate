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
  ChapterOriginWire,
  ChapterDetailWire,
  ChapterSplitPreviewWire,
  EncodingCandidateWire,
  ImportEncodingPreview,
  UrlImportBatchWire,
  UrlImportItemWire,
} from '../../src/config/project'

/** Story 6.15 — bốn trường xuất xứ RỖNG, dùng làm giá trị mặc định cho mọi fixture
 * `ChapterSplitPreviewEntryWire` ở tệp này (không ca nào trong tệp cần một giá trị khác
 * rỗng — các ca đọc `origin` sống ở `importPreviewChapterOrigin.test.ts`). */
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
      { ord: 1, title: 'Chuong 1: Mo Dau', length: 20, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
      { ord: 2, title: 'Chuong 2: Tiep Theo', length: 25, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
    ],
    broken_item_count: 0,
    needs_review_count: 0,
    clean_count: 2,
    any_signal_participated: false,
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

/**
 * **THÊM (Story 6.10)** — một ứng viên bảng mã mang N Chương với phán quyết `needs_review`
 * ĐẶT THẲNG theo `flags`, cộng hai con số tổng khớp đúng `flags`. Rust là nơi tính phán quyết
 * (AD-1) nên fixture ở đây chở KẾT QUẢ của nó, không tính lại — cùng khuôn `chapters()` trên.
 */
function candidateWithChapters(encoding: string, flags: boolean[]): EncodingCandidateWire {
  const needs = flags.filter(Boolean).length
  return candidate({
    label: encoding,
    encoding,
    chapters: {
      chapter_count: flags.length,
      chapters: flags.map((needsReview, i) => ({
        ord: i + 1,
        title: `Chuong ${i + 1}`,
        length: 20,
        cleanup_match_count: 0,
        joined_line_count_in_chapter: null,
        needs_review: needsReview,
        review_causes: needsReview ? (['short_length'] as const).slice() : [],
        origin: ORIGIN_STUB,
      })),
      broken_item_count: 0,
      needs_review_count: needs,
      clean_count: flags.length - needs,
      any_signal_participated: true,
    },
  })
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
              chapters: [{ ord: 1, title: null, length: 5, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB }],
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
        self_declared_chapters: chapters({ chapter_count: 1, chapters: [{ ord: 1, title: null, length: 9, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB }] }),
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
            chapters: chapters({ chapter_count: 1, chapters: [{ ord: 1, title: null, length: 800, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB }] }),
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
                { ord: 1, title: 'Dai', length: 4000, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
                { ord: 2, title: 'Ngan Bat Thuong', length: 40, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
                { ord: 3, title: 'Dai Nua', length: 3800, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
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
    const many = Array.from({ length: 9 }, (_, i) => ({ ord: i + 1, title: `Chuong ${i + 1}`, length: 100 + i, cleanup_match_count: 0, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB }))
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

  /**
   * **THÊM (vòng rà đối kháng bước 4, 2026-09-08)** — `reviewCauseMessageKey` (`switch` cạn,
   * bốn nhánh) chưa từng bị chạm bởi một fixture mang `high_cleanup_matches`/`high_joined_lines`
   * trước ca này — mọi fixture khác chỉ dùng `short_length` hoặc `[]`. Hoán đổi nội dung hai
   * nhánh đó (hoặc bất kỳ trong bốn) sẽ làm ca này đỏ.
   */
  it('bốn nguyên nhân review_causes đọc ĐÚNG chữ vi.json, không lẫn nhánh', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({
      preview: preview({
        candidates: [
          candidate({
            chapters: chapters({
              chapter_count: 4,
              chapters: [
                {
                  ord: 1,
                  title: 'C1',
                  length: 3,
                  cleanup_match_count: 0,
                  joined_line_count_in_chapter: 0,
                  needs_review: true,
                  review_causes: ['short_length'], origin: ORIGIN_STUB,
                },
                {
                  ord: 2,
                  title: 'C2',
                  length: 500,
                  cleanup_match_count: 99,
                  joined_line_count_in_chapter: 0,
                  needs_review: true,
                  review_causes: ['high_cleanup_matches'], origin: ORIGIN_STUB,
                },
                {
                  ord: 3,
                  title: 'C3',
                  length: 500,
                  cleanup_match_count: 0,
                  joined_line_count_in_chapter: 99,
                  needs_review: true,
                  review_causes: ['high_joined_lines'], origin: ORIGIN_STUB,
                },
                {
                  ord: 4,
                  title: 'C4',
                  length: 500,
                  cleanup_match_count: null,
                  joined_line_count_in_chapter: 0,
                  needs_review: true,
                  review_causes: ['not_measured'], origin: ORIGIN_STUB,
                },
              ],
            }),
          }),
        ],
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    const rows = wrapper.findAll('.ip-chapters-entry')
    expect(rows).toHaveLength(4)
    expect(rows[0]?.find('.ip-chapters-review-cause').text()).toBe('Ngắn bất thường')
    expect(rows[1]?.find('.ip-chapters-review-cause').text()).toBe('Xoá quá nhiều')
    expect(rows[2]?.find('.ip-chapters-review-cause').text()).toBe('Nối dòng cao')
    expect(rows[3]?.find('.ip-chapters-review-cause').text()).toBe('Không đo được')

    wrapper.unmount()
    state.resetImportPreview()
  })

  /**
   * **THÊM (vòng rà đối kháng bước 4, 2026-09-08)** — một hàng vừa là con trỏ (`⌥←`/`⌥→`,
   * Story 6.10a) VỪA `needs_review` (Story 6.10) phải giữ CẢ HAI lớp CSS, không luật nào nuốt
   * luật kia. Chương 0 là con trỏ MẶC ĐỊNH (0 lời gọi IPC cần thiết để dựng ca này) — gán
   * `needs_review: true` cho CHÍNH Chương đó là đủ để dựng cả hai điều kiện cùng lúc.
   */
  it('một hàng vừa là con trỏ VỪA cần xem giữ CẢ HAI lớp CSS', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({
      preview: preview({
        candidates: [
          candidate({
            chapters: chapters({
              chapter_count: 2,
              chapters: [
                {
                  ord: 1,
                  title: 'C1',
                  length: 3,
                  cleanup_match_count: 0,
                  joined_line_count_in_chapter: 0,
                  needs_review: true,
                  review_causes: ['short_length'], origin: ORIGIN_STUB,
                },
                { ord: 2, title: 'C2', length: 500, cleanup_match_count: 0, joined_line_count_in_chapter: 0, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
              ],
            }),
          }),
        ],
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    expect(state.importPreviewChapterCursor.value).toBe(0) // Chuong 0 la con tro mac dinh.
    const first = wrapper.findAll('.ip-chapters-entry')[0]
    expect(first.classes()).toContain('ip-chapters-entry-current')
    expect(first.classes()).toContain('ip-chapters-entry-needs-review')

    wrapper.unmount()
    state.resetImportPreview()
  })

  /**
   * **THÊM (vòng rà đối kháng bước 4, 2026-09-08)** — nhánh `v-else` "chưa đủ Chương để so"
   * (`any_signal_participated === false`) chưa từng được DỰNG trong một ca DOM nào trước đây.
   */
  it('any_signal_participated === false — hiện dòng "chưa đủ Chương để so", KHÔNG hiện chip', async () => {
    const { state, ImportPreviewOverlay } = await freshOverlay()
    previewTextMock.mockResolvedValue({
      preview: preview({
        candidates: [
          candidate({
            chapters: chapters({
              chapter_count: 3,
              chapters: [
                { ord: 1, title: 'C1', length: 10, cleanup_match_count: 0, joined_line_count_in_chapter: 0, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
                { ord: 2, title: 'C2', length: 20, cleanup_match_count: 0, joined_line_count_in_chapter: 0, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
                { ord: 3, title: 'C3', length: 30, cleanup_match_count: 0, joined_line_count_in_chapter: 0, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
              ],
              needs_review_count: 0,
              clean_count: 3,
              any_signal_participated: false,
            }),
          }),
        ],
      }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'x')

    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    expect(wrapper.find('.ip-chapter-filter-note').text()).toBe('Chưa đủ Chương để so')
    expect(wrapper.find('.ip-chapter-filter-chip-needs-review').exists()).toBe(false)
    expect(wrapper.find('.ip-chapter-filter-chip-clean').exists()).toBe(false)

    wrapper.unmount()
    state.resetImportPreview()
  })

  /**
   * **THÊM (vòng rà đối kháng bước 4, 2026-09-08)** — đóng nợ 6.10a "con trỏ vô hình khi rơi
   * vào phần bị co gọn" (`chaptersShowAll`, nhánh `cursor >= 3 && cursor < chapter_count - 3`)
   * chưa từng được DỰNG trong một ca DOM nào. Dựng 9 Chương (URL, không sắp/lọc), dời con trỏ
   * tới chỉ số 4 (nằm giữa dải bị elide mặc định — `[0,3)`/`[6,9)` được render, `[3,6)` bị
   * `⋯` thay thế khi KHÔNG có lý do để hiện đủ).
   */
  it('con trỏ dời vào vùng bị co gọn (N ≥ 7, không sắp/lọc) — danh sách tự hiện ĐỦ, `⋯` biến mất', async () => {
    const nineChapters: ChapterSplitPreviewWire = {
      chapter_count: 9,
      chapters: Array.from({ length: 9 }, (_, i) => ({
        ord: i + 1,
        title: `Chuong ${i + 1}`,
        length: 100 + i,
        cleanup_match_count: 0,
        joined_line_count_in_chapter: 0,
        needs_review: false,
        review_causes: [], origin: ORIGIN_STUB,
      })),
      broken_item_count: 0,
      needs_review_count: 0,
      clean_count: 9,
      any_signal_participated: false,
    }
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({
      batch: {
        items: ['a', 'b', 'c'].map(urlItem),
        encoding_preview: preview({ candidates: [candidate({ chapters: nineChapters })] }),
        domain_log_domain_count: 1,
      },
      error: null,
    })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])
    previewChapterDetailMock.mockResolvedValue({
      detail: { cleanup: { text: '', spans: [], rules: [], window_truncated: false, final_text: 'x' }, blocks: null },
      error: null,
    })

    const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    // Truoc khi doi con tro: khung nhin mac dinh co gon, `⋯` co mat.
    expect(wrapper.find('.ip-chapters-ellipsis').exists()).toBe(true)
    expect(wrapper.findAll('.ip-chapters-entry')).toHaveLength(6) // ba dau + ba cuoi

    // Doi con tro toi chi so 4 (Chuong 5) -- nam giua dai bi elide [3,6).
    for (let i = 0; i < 4; i += 1) {
      state.nextImportPreviewChapter()
      await new Promise((resolve) => setTimeout(resolve, 0))
    }
    expect(state.importPreviewChapterCursor.value).toBe(4)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.ip-chapters-ellipsis').exists()).toBe(false)
    expect(wrapper.findAll('.ip-chapters-entry')).toHaveLength(9)
    const list = wrapper.get('.ip-chapters-list')
    expect(list.attributes('aria-activedescendant')).toBe('ip-chapter-5')
    expect(wrapper.find('#ip-chapter-5').attributes('aria-selected')).toBe('true')

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
      { ord: 1, title: 'Chuong 1', length: 100, cleanup_match_count: 1, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
      { ord: 2, title: 'Chuong 2', length: 200, cleanup_match_count: 2, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
      { ord: 3, title: 'Chuong 3', length: 300, cleanup_match_count: 3, joined_line_count_in_chapter: null, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
    ],
    broken_item_count: 0,
    needs_review_count: 0,
    clean_count: 3,
    any_signal_participated: false,
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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.10 — bộ lọc "cần xem" (`⌥W`), tầng STATE (0 lời gọi IPC, Rust đã cấp sẵn
// `needs_review`/`review_causes` cho MỌI Chương lúc tải xem trước).
// ═════════════════════════════════════════════════════════════════════════════════

/** Ba Chương — Chương 0 "cần xem" (`ShortLength`), hai Chương 1/2 "sạch". `needs_review_count`/
 * `clean_count`/`any_signal_participated` khớp ĐÚNG dữ liệu Chương (Rust cộng, không tính lại
 * ở test). */
function mixedChapters(): ChapterSplitPreviewWire {
  return {
    chapter_count: 3,
    chapters: [
      {
        ord: 1,
        title: 'Chuong ngan',
        length: 3,
        cleanup_match_count: 0,
        joined_line_count_in_chapter: 0,
        needs_review: true,
        review_causes: ['short_length'], origin: ORIGIN_STUB,
      },
      {
        ord: 2,
        title: 'Chuong binh thuong 1',
        length: 500,
        cleanup_match_count: 0,
        joined_line_count_in_chapter: 0,
        needs_review: false,
        review_causes: [], origin: ORIGIN_STUB,
      },
      {
        ord: 3,
        title: 'Chuong binh thuong 2',
        length: 520,
        cleanup_match_count: 0,
        joined_line_count_in_chapter: 0,
        needs_review: false,
        review_causes: [], origin: ORIGIN_STUB,
      },
    ],
    broken_item_count: 0,
    needs_review_count: 1,
    clean_count: 2,
    any_signal_participated: true,
  }
}

describe('importPreviewState — bộ lọc "cần xem" (Story 6.10)', () => {
  it('mặc định bộ lọc TẮT', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: mixedChapters() })] }), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung')

    expect(state.importPreviewChapterFilterActive.value).toBe(false)

    state.resetImportPreview()
  })

  it('bật rồi tắt lại — bấm hai lần đảo trạng thái, 0 lời gọi IPC', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: mixedChapters() })] }), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung')

    state.toggleImportPreviewChapterFilter()
    expect(state.importPreviewChapterFilterActive.value).toBe(true)
    state.toggleImportPreviewChapterFilter()
    expect(state.importPreviewChapterFilterActive.value).toBe(false)
    expect(previewChapterDetailMock).not.toHaveBeenCalled()

    state.resetImportPreview()
  })

  it('0 mục cần xem — bộ lọc KHÔNG bật, không kêu, không ném', async () => {
    const state = await freshState()
    const allClean = chapters() // fixture mac dinh cua tep nay: hai Chuong, ca hai `needs_review: false`
    previewTextMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: allClean })] }), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung')
    expect(allClean.needs_review_count).toBe(0) // tien de cua fixture

    expect(() => state.toggleImportPreviewChapterFilter()).not.toThrow()
    expect(state.importPreviewChapterFilterActive.value).toBe(false)

    state.resetImportPreview()
  })

  /**
   * **THÊM (vòng rà đối kháng bước 4, 2026-09-08)** — ca THẬT đã lần ra: 4 link, 1 hỏng ⇒ 3
   * Chương thật ⇒ DƯỚI bốn giá trị đo được ⇒ KHÔNG hàng rào nào tồn tại
   * (`any_signal_participated === false`), 0 Chương `needs_review`, NHƯNG `needs_review_count
   * === 1` (CỘNG từ `broken_item_count`, §Always spec 6.10: "kể cả vế link hỏng"). Bản trước
   * chỉ chặn BẬT bằng `needs_review_count === 0` — điều kiện đó SAI ở đây (`=== 1`) nên bộ lọc
   * BẬT được, làm tầng 4 rỗng hẳn VÀ chip biến mất (thay bằng "chưa đủ Chương để so") — mất
   * trạng thái, mất lối tắt.
   */
  it('4 link 1 hỏng (any_signal_participated === false, needs_review_count === 1 từ link hỏng) — bộ lọc KHÔNG bật', async () => {
    const state = await freshState()
    const threeRealChaptersOneBrokenLink: ChapterSplitPreviewWire = {
      chapter_count: 3,
      chapters: [
        { ord: 1, title: 'C1', length: 10, cleanup_match_count: 0, joined_line_count_in_chapter: 0, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
        { ord: 2, title: 'C2', length: 20, cleanup_match_count: 0, joined_line_count_in_chapter: 0, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
        { ord: 3, title: 'C3', length: 30, cleanup_match_count: 0, joined_line_count_in_chapter: 0, needs_review: false, review_causes: [], origin: ORIGIN_STUB },
      ],
      broken_item_count: 1,
      needs_review_count: 1, // TU link hong -- KHONG Chuong nao needs_review that.
      clean_count: 3,
      any_signal_participated: false, // duoi bon gia tri do duoc.
    }
    startUrlImportMock.mockResolvedValue({
      batch: {
        items: [
          { url: 'a', ok: true, error: null },
          { url: 'b', ok: true, error: null },
          { url: 'c', ok: true, error: null },
          {
            url: 'd',
            ok: false,
            error: { code: 'import.web_item_failed', message_key: 'err.import.web_invalid_url', params: {}, retryable: false },
          },
        ],
        encoding_preview: preview({ candidates: [candidate({ chapters: threeRealChaptersOneBrokenLink })] }),
        domain_log_domain_count: 4,
      },
      error: null,
    })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c', 'd'])
    expect(state.importPreviewSelectedChapters.value?.needs_review_count).toBe(1)
    expect(state.importPreviewSelectedChapters.value?.any_signal_participated).toBe(false)

    state.toggleImportPreviewChapterFilter()

    expect(state.importPreviewChapterFilterActive.value).toBe(false)

    state.resetImportPreview()
  })

  it('bật lọc khi con trỏ đứng ở Chương SẠCH — con trỏ dời tới Chương cần xem đầu tiên', async () => {
    const state = await freshState()
    startUrlImportMock.mockResolvedValue({
      batch: {
        items: [{ url: 'a', ok: true, error: null }, { url: 'b', ok: true, error: null }, { url: 'c', ok: true, error: null }],
        encoding_preview: preview({ candidates: [candidate({ chapters: mixedChapters() })] }),
        domain_log_domain_count: 1,
      },
      error: null,
    })
    await state.openImportPreviewFromUrls('Ten', 'en', '', ['a', 'b', 'c'])
    previewChapterDetailMock.mockResolvedValue({
      detail: { cleanup: { text: '', spans: [], rules: [], window_truncated: false, final_text: 'chuong 1' }, blocks: null },
      error: null,
    })
    // Con trỏ dời sang Chương 1 (chỉ số 1, `needs_review: false` trong `mixedChapters()`).
    state.nextImportPreviewChapter()
    await new Promise((resolve) => setTimeout(resolve, 0))
    expect(state.importPreviewChapterCursor.value).toBe(1)

    state.toggleImportPreviewChapterFilter()

    // Chương 1 (chỉ số 1) SẠCH, vừa bị lọc khỏi DOM — con trỏ phải dời tới Chương CẦN XEM đầu
    // tiên (chỉ số 0, `mixedChapters()[0].needs_review === true`).
    expect(state.importPreviewChapterFilterActive.value).toBe(true)
    expect(state.importPreviewChapterCursor.value).toBe(0)

    state.resetImportPreview()
  })

  it('lớp phủ ĐÃ ĐÓNG — bật lọc không đổi gì', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: mixedChapters() })] }), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung')
    state.cancelImportPreview()

    state.toggleImportPreviewChapterFilter()
    expect(state.importPreviewChapterFilterActive.value).toBe(false)
  })

  it('một lượt mở MỚI (huỷ + mở lại) reset cờ lọc về TẮT', async () => {
    const state = await freshState()
    previewTextMock.mockResolvedValue({ preview: preview({ candidates: [candidate({ chapters: mixedChapters() })] }), error: null })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung')
    state.toggleImportPreviewChapterFilter()
    expect(state.importPreviewChapterFilterActive.value).toBe(true)

    state.cancelImportPreview()
    await state.openImportPreviewFromText('Ten2', 'en', '', 'noi dung khac')

    expect(state.importPreviewChapterFilterActive.value).toBe(false)

    state.resetImportPreview()
  })
})

/**
 * I/O Matrix spec 6.10, hàng *"Đổi ứng viên bảng mã khi bộ lọc đang bật"* — phán quyết tính
 * lại trên số của ứng viên MỚI, và bộ lọc **giữ trạng thái bật**.
 *
 * 🔴 **Vì sao hàng này cần một ca riêng.** `toggleImportPreviewChapterFilter` là chỗ DUY NHẤT
 * đặt `chapterFilterActive = true`, còn ba chỗ đặt `false` đều là lượt MỞ/RESET
 * (`openWith`, `openImportPreviewFromUrls`, `resetImportPreview`). Cờ vì thế sống sót qua một
 * lượt đổi ứng viên **do cấu trúc**, không do một dòng mã nào nói ra — tức đúng loại bất biến
 * mà lượt tới sẽ phá mà không cổng nào đỏ. Ma trận đã ký gọi tên nó (*"Không âm thầm tắt lọc"*)
 * nên nó phải có chủ ở đây.
 */
describe('importPreviewState — đổi ứng viên bảng mã KHÔNG tắt bộ lọc (Story 6.10)', () => {
  it('bật lọc rồi đổi ứng viên — lọc VẪN bật, phán quyết đọc theo ứng viên mới', async () => {
    const state = await freshState()
    // Hai ung vien mang HAI phan quyet khac nhau -- de khang dinh "doc theo ung vien moi"
    // khong the xanh nho ca hai giong het nhau.
    previewTextMock.mockResolvedValue({
      preview: {
        selected_encoding: 'UTF-8',
        confidence: 'low',
        candidates: [
          candidateWithChapters('UTF-8', [false, true, false, false]),
          candidateWithChapters('GBK', [true, true, true, false]),
        ],
        self_declared_normalized: null,
        self_declared_cleanup: null,
        self_declared_chapters: null,
      },
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'text')

    state.toggleImportPreviewChapterFilter()
    expect(state.importPreviewChapterFilterActive.value).toBe(true)
    expect(state.importPreviewSelectedChapters.value?.needs_review_count).toBe(1)

    state.selectImportPreviewCandidate('GBK')

    expect(state.importPreviewChapterFilterActive.value).toBe(true)
    expect(state.importPreviewSelectedChapters.value?.needs_review_count).toBe(3)
    expect(state.importPreviewSelectedChapters.value?.clean_count).toBe(1)

    state.resetImportPreview()
  })
})

/**
 * **Debt probe — Story 6.18 task 6.** `deferred-work.md`, cụm "Deferred from: 6-10…", Chủ
 * Story 6.18: *"Bật bộ lọc ép hiện TRỌN danh sách, không ảo hoá — chưa ai đo ở quy mô nghìn
 * Chương."* `chaptersShowAll` trả `true` khi lọc bật (bất kể số hàng còn lại), và
 * `chapterEntriesRendered` bỏ co gọn ba-đầu/`⋯`/ba-cuối khi đó — số hàng `<li>` trên DOM
 * bằng đúng số Chương còn lại sau lọc.
 *
 * ⚠️ **Đây KHÔNG phải bàn đo "render trong release app"** mà spec 6.18 task 6 đòi —
 * `happy-dom` là một mô phỏng DOM trong Node, không phải WKWebView thật (chính
 * `vitest.config.ts` ghi rõ điều này ở khối "VAI CỦA BỘ CHẠY NÀY"). Ca này đo được ĐÚNG một
 * nửa của món nợ (số hàng thật sự render KHÔNG ảo hoá ở quy mô 1.000 Chương) và một con số
 * THỜI GIAN MOUNT phía JS/happy-dom — bằng chứng, không phải verdict cuối cùng; số đo trên
 * WKWebView thật vẫn cần Story 6.18 task 7 (release app thật).
 */
describe('importPreviewState — chaptersShowAll ở quy mô 1.000 Chương (debt probe Story 6.18)', () => {
  it('lọc bật + 1.000 Chương đều needs_review — DOM giữ ĐỦ 1.000 hàng, không ảo hoá, không `⋯`', async () => {
    const CHAPTER_COUNT = 1_000
    const state = await freshState()
    previewTextMock.mockResolvedValue({
      preview: preview({ candidates: [candidateWithChapters('UTF-8', Array(CHAPTER_COUNT).fill(true))] }),
      error: null,
    })
    await state.openImportPreviewFromText('Ten', 'en', '', 'noi dung')

    state.toggleImportPreviewChapterFilter()
    expect(state.importPreviewChapterFilterActive.value).toBe(true)
    expect(state.importPreviewSelectedChapters.value?.chapter_count).toBe(CHAPTER_COUNT)

    const ImportPreviewOverlay = (await import('../../src/ImportPreviewOverlay.vue')).default
    const t0 = performance.now()
    const wrapper = mount(ImportPreviewOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const mountMs = performance.now() - t0

    expect(wrapper.find('.ip-chapters-ellipsis').exists()).toBe(false)
    const rows = wrapper.findAll('.ip-chapters-entry')
    expect(rows).toHaveLength(CHAPTER_COUNT)

    console.log(
      `[chaptersShowAll debt probe] ${CHAPTER_COUNT} Chuong, loc bat, khong ao hoa -- ` +
        `${rows.length} hang <li> that su tren DOM, mount+nextTick (happy-dom) = ${mountMs.toFixed(1)} ms`,
    )

    wrapper.unmount()
    state.resetImportPreview()
  })
})
