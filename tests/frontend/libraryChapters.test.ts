/**
 * `modes/libraryChapters.ts` + `config/chapter.ts::listChapters` + khối "Chương" của
 * `modes/LibraryMode.vue` — Story 5.7, FR12.
 *
 * ⚠️ **PHẠM VI** — `happy-dom` canh HÀNH VI của module thuần, không hình học/engine thật
 * (`tests/AGENTS.md`). `chapterWindow()` là một hàm THUẦN nhận toạ độ qua tham số — nó
 * không đọc DOM, nên vitest kiểm được nó TẤT ĐỊNH mà không cần bố cục thật.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

/**
 * Kết quả mà lượt `flushEditorBeforeDiscreteWrite()` kế tiếp trả về. `'clean'` là mặc định —
 * mọi ca cũ của tệp này không nói gì về flush và phải giữ nguyên hành vi.
 */
const ketQuaFlush: { value: 'clean' | 'failed' | 'still-dirty' } = { value: 'clean' }

// ⚠️ Giả ĐÚNG MỘT export, giữ nguyên phần còn lại qua `importOriginal` — bốn lượt vứt state
// và `ensureSegmentsLoaded` mà `openWorkById` gọi phải là hàng THẬT, nếu không ca dưới đây
// nghiệm thu một đường đi không tồn tại.
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return { ...actual, flushEditorBeforeDiscreteWrite: async () => ketQuaFlush.value }
})

const CHAPTER_ROW_A = {
  chapter_id: 1,
  ord: 1,
  title: 'Chuong Mot',
  status: 'not_started',
  segment_count: 10,
}

const CHAPTER_ROW_UNTITLED = {
  chapter_id: 2,
  ord: 2,
  title: null,
  status: 'in_progress',
  segment_count: 5,
}

const WORK_NONE_OPEN_ERROR = {
  code: 'work.none_open',
  message_key: 'err.work.none_open',
  params: {},
  retryable: false,
}

beforeEach(async () => {
  mockInvoke.mockReset()
  const state = await import('../../src/modes/libraryChapters')
  state.resetLibraryChapters()
  const works = await import('../../src/modes/libraryWorks')
  works.resetLibraryWorks()
})

afterEach(() => {
  vi.restoreAllMocks()
})

// ═════════════════════════════════════════════════════════════════════════════════
// `chapterWindow()` — hàm THUẦN, AC2.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/libraryChapters.ts::chapterWindow — hàm thuần', () => {
  it('danh sách 2.000 hàng ⇒ end - start ≤ 60', async () => {
    const { chapterWindow } = await import('../../src/modes/libraryChapters')
    const slice = chapterWindow(0, 240, 40, 2000, 4)
    expect(slice.end - slice.start).toBeLessThanOrEqual(60)
    expect(slice.start).toBe(0)
  })

  it('cuộn tới cuối ⇒ cửa sổ kẹp đúng biên trên (`end === total`)', async () => {
    const { chapterWindow } = await import('../../src/modes/libraryChapters')
    // Cuộn quá xa so với nội dung thật -- `firstVisible` vượt `total`, `end` phải kẹp về
    // đúng `total`, không tràn ra ngoài mảng.
    const slice = chapterWindow(100_000, 240, 40, 2000, 4)
    expect(slice.end).toBe(2000)
    expect(slice.end - slice.start).toBeLessThanOrEqual(60)
    expect(slice.padBottom).toBe(0)
  })

  it('`total = 0` ⇒ cửa sổ rỗng, không `NaN`', async () => {
    const { chapterWindow } = await import('../../src/modes/libraryChapters')
    const slice = chapterWindow(0, 240, 40, 0, 4)
    expect(slice).toEqual({ start: 0, end: 0, padTop: 0, padBottom: 0 })
    expect(Number.isNaN(slice.padTop)).toBe(false)
    expect(Number.isNaN(slice.padBottom)).toBe(false)
  })

  it('`rowHeight = 0` (phòng thủ) ⇒ cửa sổ rỗng, không chia cho 0', async () => {
    const { chapterWindow } = await import('../../src/modes/libraryChapters')
    const slice = chapterWindow(0, 240, 0, 2000, 4)
    expect(slice).toEqual({ start: 0, end: 0, padTop: 0, padBottom: 0 })
  })

  it('đệm (`padTop`/`padBottom`) đúng bằng số hàng bị cắt nhân chiều cao hàng', async () => {
    const { chapterWindow } = await import('../../src/modes/libraryChapters')
    const slice = chapterWindow(400, 240, 40, 100, 2)
    expect(slice.padTop).toBe(slice.start * 40)
    expect(slice.padBottom).toBe((100 - slice.end) * 40)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Con trỏ Chương — chép khuôn `workCursor`.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/libraryChapters.ts — con trỏ Chương', () => {
  it('next/prev trên danh sách rỗng là no-op', async () => {
    const state = await import('../../src/modes/libraryChapters')
    expect(state.libraryChapterCursor.value).toBe(0)
    state.nextChapter()
    expect(state.libraryChapterCursor.value).toBe(0)
    state.prevChapter()
    expect(state.libraryChapterCursor.value).toBe(0)
  })

  it('con trỏ kẹp về ô cuối còn lại sau một lượt tải NGẮN HƠN vị trí hiện tại', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_chapters') return Promise.resolve([CHAPTER_ROW_A, CHAPTER_ROW_UNTITLED])
      return Promise.reject(new Error(`invoke gia khong biet lenh: ${cmd}`))
    })
    const state = await import('../../src/modes/libraryChapters')
    await state.loadChapters()
    state.nextChapter()
    expect(state.libraryChapterCursor.value).toBe(1)

    // Lượt tải kế tiếp trả về MỘT hàng duy nhất -- con trỏ đang ở 1 phải kẹp về 0.
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_chapters') return Promise.resolve([CHAPTER_ROW_A])
      return Promise.reject(new Error(`invoke gia khong biet lenh: ${cmd}`))
    })
    await state.loadChapters()
    expect(state.libraryChapterCursor.value).toBe(0)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// `loadChapters()` — adapter không bao giờ ném, và mọi lỗi đi qua `chaptersError`.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/libraryChapters.ts::loadChapters', () => {
  it('chưa Tác phẩm nào mở ⇒ lỗi có tên đi vào `libraryChaptersError`, danh sách rỗng', async () => {
    mockInvoke.mockRejectedValueOnce(WORK_NONE_OPEN_ERROR)
    const state = await import('../../src/modes/libraryChapters')
    await state.loadChapters()

    expect(state.libraryChapters.value).toEqual([])
    expect(state.libraryChaptersError.value?.code).toBe('work.none_open')
    // Lượt tải VẪN đã "xong" -- lỗi không phải "chưa tải".
    expect(state.libraryChaptersHaveLoaded.value).toBe(false)
  })

  it('tải thành công ⇒ `chaptersHaveLoaded = true` và danh sách khớp nguyên văn', async () => {
    mockInvoke.mockResolvedValueOnce([CHAPTER_ROW_A, CHAPTER_ROW_UNTITLED])
    const state = await import('../../src/modes/libraryChapters')
    await state.loadChapters()

    expect(state.libraryChaptersHaveLoaded.value).toBe(true)
    expect(state.libraryChapters.value).toEqual([CHAPTER_ROW_A, CHAPTER_ROW_UNTITLED])
    expect(state.libraryChaptersError.value).toBeNull()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// `modes/LibraryMode.vue` — mount THẬT, khối "Chương".
// ═════════════════════════════════════════════════════════════════════════════════

function mockInvokeForChaptersMount(chapters: unknown[] | 'none-open'): void {
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'library_list_works') {
      return Promise.resolve({ total: 0, matched: 0, works: [], genres: [], source_langs: [] })
    }
    if (cmd === 'read_work_lifecycle') {
      return Promise.reject(WORK_NONE_OPEN_ERROR)
    }
    if (cmd === 'list_chapters') {
      if (chapters === 'none-open') return Promise.reject(WORK_NONE_OPEN_ERROR)
      return Promise.resolve(chapters)
    }
    return Promise.reject(new Error(`invoke gia khong biet lenh: ${cmd}`))
  })
}

describe('modes/LibraryMode.vue — khối Chương (mount thật)', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    // 🔴 BẮT BUỘC unmount ở MỌI đường ra — cùng lý do đã ghi ở `libraryWorks.test.ts`:
    // `declareFocus('mode.library', ..)` ném khi owner TRÙNG nếu lượt trước không nhả.
    wrapper?.unmount()
    wrapper = null
  })

  it('nhánh `chaptersHaveLoaded === false` KHÔNG nói "Tác phẩm này chưa có Chương nào"', async () => {
    mockInvokeForChaptersMount('none-open')

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()

    // Trước lượt tải xong: câu "chưa tải Chương" phải hiện, câu "chưa có Chương nào" thì KHÔNG.
    const status = wrapper.find('.chapters-block .status')
    expect(status.exists()).toBe(true)
    expect(status.text()).not.toContain('chưa có Chương nào')
  })

  it('`title: null` ⇒ nhãn dựng từ `ord`', async () => {
    mockInvokeForChaptersMount([CHAPTER_ROW_UNTITLED])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryChapters')
    wrapper = mount(LibraryMode)
    await state.loadChapters()
    await wrapper.vm.$nextTick()

    const title = wrapper.find('.chapter-title')
    expect(title.exists()).toBe(true)
    expect(title.text()).toBe(`Chương ${CHAPTER_ROW_UNTITLED.ord}`)
  })

  it('Tác phẩm có Chương ⇒ câu kết quả nói đúng SỐ, và mỗi hàng mang ord/tiêu đề/trạng thái/số câu', async () => {
    mockInvokeForChaptersMount([CHAPTER_ROW_A])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryChapters')
    wrapper = mount(LibraryMode)
    await state.loadChapters()
    await wrapper.vm.$nextTick()

    const row = wrapper.find('.chapter-row')
    expect(row.exists()).toBe(true)
    expect(row.find('.chapter-ord').text()).toBe(String(CHAPTER_ROW_A.ord))
    expect(row.find('.chapter-title').text()).toBe(CHAPTER_ROW_A.title)
    expect(row.find('.chapter-segment-count').text()).toBe(String(CHAPTER_ROW_A.segment_count))
  })

  it('AC2 — danh sách 2.000 Chương ⇒ số `<li>` THẬT trong DOM ≤ 60', async () => {
    const many = Array.from({ length: 2000 }, (_, i) => ({
      chapter_id: i + 1,
      ord: i + 1,
      title: `Chuong ${i + 1}`,
      status: 'not_started',
      segment_count: 1,
    }))
    mockInvokeForChaptersMount(many)

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryChapters')
    wrapper = mount(LibraryMode)
    await state.loadChapters()
    await wrapper.vm.$nextTick()

    const items = wrapper.findAll('.chapters-list li')
    expect(items.length).toBeLessThanOrEqual(60)
    // Đối chứng dương: không phải toàn bộ 2.000 hàng đã render (nếu không thì phép kiểm
    // trên xanh vì lười, không vì cửa sổ THẬT SỰ chạy).
    expect(items.length).toBeLessThan(many.length)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// AC9 — mở một TÁC PHẨM khác khi bản dịch chưa xuống đĩa phải bị CHẶN
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 **Ca này ra đời từ lượt review, không từ lượt viết đầu.** `chapterPosition.test.ts` canh
// đúng mệnh đề này cho `openChapterById` (đổi CHƯƠNG); `openWorkById` (đổi TÁC PHẨM) là một
// hàm KHÁC, một cửa chặn KHÁC, và trước ca này **không đường nghiệm thu nào chạm tới nó** —
// bàn đo e2e bấm "Mở Tác phẩm" đúng một lần trên một Editor sạch, nên gỡ trọn khối `if
// (flushed !== 'clean')` khỏi `openWorkById` cũng không làm ca nào đỏ.
//
// 🔴 Và cái giá cao hơn hẳn nhánh đổi Chương: đổi Tác phẩm trỏ `Store` sang MỘT TỆP KHÁC, nên
// một lô flush tới trễ mang `segment.id` của kho cũ ghi vào một kho không có id đó ⇒ bản dịch
// người dùng vừa gõ mất **im lặng** (`libraryImport.ts:119-152` đã phân xử nguyên văn lớp lỗi
// này cho đường NHẬP).
describe('modes/libraryChapters.ts::openWorkById — cửa chặn khi flush chưa sạch (AC9)', () => {
  it('🔴 flush TRƯỢT ⇒ lượt mở Tác phẩm bị CHẶN, và `open_work` KHÔNG chạy', async () => {
    ketQuaFlush.value = 'failed'
    const state = await import('../../src/modes/libraryChapters')

    await state.openWorkById('mot-work-id-bat-ky')

    expect(mockInvoke.mock.calls.some((c) => c[0] === 'open_work')).toBe(false)
    expect(state.libraryOpenWorkNotice.value).toBe('flush-failed')
  })

  it("🔴 flush trả `'still-dirty'` ⇒ CHẶN, và câu báo PHÂN BIỆT được với `'failed'`", async () => {
    ketQuaFlush.value = 'still-dirty'
    const state = await import('../../src/modes/libraryChapters')

    await state.openWorkById('mot-work-id-bat-ky')

    expect(mockInvoke.mock.calls.some((c) => c[0] === 'open_work')).toBe(false)
    expect(state.libraryOpenWorkNotice.value).toBe('still-dirty')
  })
})
