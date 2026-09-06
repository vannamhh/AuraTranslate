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
  ChapterSplitPreviewWire,
  EncodingCandidateWire,
  ImportEncodingPreview,
} from '../../src/config/project'

const previewTextMock = vi.fn()
const previewFileMock = vi.fn()
const confirmMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: (...args: unknown[]) => previewTextMock(...args),
    previewImportEncodingFromFile: (...args: unknown[]) => previewFileMock(...args),
    confirmImportWithEncoding: (...args: unknown[]) => confirmMock(...args),
  }
})

async function freshState() {
  vi.resetModules()
  previewTextMock.mockReset()
  previewFileMock.mockReset()
  confirmMock.mockReset()
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
      { ord: 1, title: 'Chuong 1: Mo Dau', length: 20 },
      { ord: 2, title: 'Chuong 2: Tiep Theo', length: 25 },
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
              chapters: [{ ord: 1, title: null, length: 5 }],
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
        self_declared_chapters: chapters({ chapter_count: 1, chapters: [{ ord: 1, title: null, length: 9 }] }),
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
            chapters: chapters({ chapter_count: 1, chapters: [{ ord: 1, title: null, length: 800 }] }),
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
                { ord: 1, title: 'Dai', length: 4000 },
                { ord: 2, title: 'Ngan Bat Thuong', length: 40 },
                { ord: 3, title: 'Dai Nua', length: 3800 },
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
    const many = Array.from({ length: 9 }, (_, i) => ({ ord: i + 1, title: `Chuong ${i + 1}`, length: 100 + i }))
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
