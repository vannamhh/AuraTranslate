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

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.8 — BỐN CHỖ NỐI MỚI CỦA "TỔ CHỨC LẠI CHƯƠNG" (FR15)
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 **Vì sao đúng bốn ca này, không nhiều hơn.** Hành vi SQL của bốn thao tác thuộc
// `src-tauri/tests/project_contract.rs` — đó là đường nghiệm thu của AD-32 (`tests/AGENTS.md`:
// bốn đường, bốn vai, chọn sai đường là dựng một nguồn sự thật thứ hai). Thứ CHỈ tầng này
// canh được là bốn mệnh đề về **webview**: cửa chặn flush, lượt nạp lại sau khi ghi, cờ bận
// khoá nút, và vị từ caret rỗng.

describe('modes/libraryChapters.ts — cửa chặn flush của bốn thao tác tổ chức (Story 5.8)', () => {
  it("🔴 flush trả `'still-dirty'` ⇒ KHÔNG lệnh tổ chức nào chạy, và câu báo PHÂN BIỆT được", async () => {
    mockInvoke.mockResolvedValue([CHAPTER_ROW_A, CHAPTER_ROW_UNTITLED])
    const state = await import('../../src/modes/libraryChapters')
    await state.loadChapters()
    mockInvoke.mockClear()

    ketQuaFlush.value = 'still-dirty'
    await state.renameCurrentChapter()

    const daGhi = mockInvoke.mock.calls.some((c) =>
      ['rename_chapter', 'move_chapter', 'merge_chapter_into_previous'].includes(String(c[0])),
    )
    expect(daGhi).toBe(false)
    expect(state.libraryChapterReorgNotice.value).toBe('still-dirty')
  })

  it("🔴 flush trả `'failed'` ⇒ lượt GỘP bị CHẶN, `merge_chapter_into_previous` KHÔNG chạy", async () => {
    mockInvoke.mockResolvedValue([CHAPTER_ROW_A, CHAPTER_ROW_UNTITLED])
    const state = await import('../../src/modes/libraryChapters')
    await state.loadChapters()
    state.nextChapter() // Con trỏ về hàng thứ hai -- hàng đầu không có Chương nào phía trước.
    mockInvoke.mockClear()

    ketQuaFlush.value = 'failed'
    await state.mergeCurrentChapterUp()

    expect(mockInvoke.mock.calls.some((c) => c[0] === 'merge_chapter_into_previous')).toBe(false)
    expect(state.libraryChapterReorgNotice.value).toBe('flush-failed')
  })
})

describe('modes/libraryChapters.ts::renameCurrentChapter — Story 5.8', () => {
  it('đổi tên xong ⇒ danh sách được NẠP LẠI, không đứng ở tên cũ', async () => {
    ketQuaFlush.value = 'clean'
    mockInvoke.mockResolvedValue([CHAPTER_ROW_A])
    const state = await import('../../src/modes/libraryChapters')
    await state.loadChapters()
    expect(state.libraryChapters.value[0]?.title).toBe('Chuong Mot')

    // Từ đây trở đi cả `rename_chapter` lẫn `list_chapters` đều trả TÊN MỚI.
    const doiTen = { ...CHAPTER_ROW_A, title: 'Hoi Mot' }
    mockInvoke.mockReset()
    mockInvoke.mockResolvedValue([doiTen])
    state.chapterRenameDraft.value = 'Hoi Mot'

    await state.renameCurrentChapter()

    expect(mockInvoke.mock.calls.some((c) => c[0] === 'rename_chapter')).toBe(true)
    // Đối chứng chỗ nối: một lượt `list_chapters` THẬT SỰ chạy SAU lượt ghi. Bỏ
    // `await loadChapters()` khỏi `renameCurrentChapter` làm đúng phép kiểm này đỏ.
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'list_chapters')).toBe(true)
    expect(state.libraryChapters.value[0]?.title).toBe('Hoi Mot')
    expect(state.libraryChapterReorgBusy.value).toBe(false)
  })

  it('🔴 lỗi IPC ⇒ câu lỗi hiện ra và cờ bận được NHẢ (nút không kẹt vĩnh viễn)', async () => {
    ketQuaFlush.value = 'clean'
    mockInvoke.mockResolvedValue([CHAPTER_ROW_A])
    const state = await import('../../src/modes/libraryChapters')
    await state.loadChapters()

    mockInvoke.mockReset()
    mockInvoke.mockRejectedValue(WORK_NONE_OPEN_ERROR)
    await state.renameCurrentChapter()

    expect(state.libraryChapterReorgError.value?.code).toBe('work.none_open')
    expect(state.libraryChapterReorgBusy.value).toBe(false)
  })
})

describe('modes/LibraryMode.vue — nút tổ chức tắt khi KHÔNG có Chương nào đang chọn (Story 5.8)', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('danh sách chưa tải ⇒ khối tổ chức KHÔNG render (không có nút nào để bấm nhầm)', async () => {
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-library-chapter-rename]').exists()).toBe(false)
    expect(wrapper.find('[data-library-chapter-merge-up]').exists()).toBe(false)
  })

  it('đã tải, có Chương ⇒ bốn nút tổ chức có mặt và BẤM ĐƯỢC', async () => {
    ketQuaFlush.value = 'clean'
    mockInvoke.mockResolvedValue([CHAPTER_ROW_A, CHAPTER_ROW_UNTITLED])
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryChapters')
    wrapper = mount(LibraryMode)
    await state.loadChapters()
    await wrapper.vm.$nextTick()

    for (const attr of [
      'data-library-chapter-rename',
      'data-library-chapter-move-up',
      'data-library-chapter-move-down',
      'data-library-chapter-merge-up',
    ]) {
      const nut = wrapper.find(`[${attr}]`)
      expect(nut.exists()).toBe(true)
      expect(nut.attributes('disabled')).toBeUndefined()
    }
  })
})

// 🔴 `splitChapterHere` sống ở `editorPanelState.ts`, không ở `libraryChapters.ts` — điểm
// tách là một CÂU, và `caretSegmentId` là chỗ duy nhất biết câu nào đang được chọn. Ca dưới
// đây gọi hàng THẬT (`importOriginal` giữ nguyên mọi export trừ `flushEditorBeforeDiscreteWrite`).
describe('panels/editorPanelState.ts::splitChapterHere — caret rỗng (Story 5.8)', () => {
  it('🔴 caret `null` ⇒ 0 lượt `invoke`, trả `false`, KHÔNG ném', async () => {
    const editor = await import('../../src/panels/editorPanelState')
    editor.resetEditorPanel()
    mockInvoke.mockClear()

    const ketQua = await editor.splitChapterHere()

    expect(ketQua).toBe(false)
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'split_chapter_at_segment')).toBe(false)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.8 — LƯỢT RÀ 2026-08-29: BỐN LỖ ĐO ĐƯỢC, BỐN CA
// ═════════════════════════════════════════════════════════════════════════════════
//
// Cả bốn ca dưới đây ra đời từ lượt rà, không từ lượt viết đầu — và mỗi ca đứng trên một phép
// đo, không một lo xa. Ghi ra ở đây để lượt đọc sau biết chúng canh cái gì.

describe('modes/libraryChapters.ts — ô nhập tên đi THEO Chương đang chọn (Story 5.8, lượt rà)', () => {
  it('🔴 dời con trỏ sang Chương khác ⇒ ô nhập MỒI lại theo tên Chương mới, không giữ chữ cũ', async () => {
    ketQuaFlush.value = 'clean'
    mockInvoke.mockResolvedValue([CHAPTER_ROW_A, CHAPTER_ROW_UNTITLED])
    const state = await import('../../src/modes/libraryChapters')
    await state.loadChapters()
    await Promise.resolve()

    // Con trỏ ở hàng 0 (`Chuong Mot`) -- ô nhập mồi bằng chính tên đó.
    expect(state.chapterRenameDraft.value).toBe('Chuong Mot')

    // Người dùng gõ dở một cái tên rồi ĐỔI Ý, dời con trỏ sang Chương khác.
    state.chapterRenameDraft.value = 'Ten go do cho Chuong Mot'
    state.nextChapter()
    await Promise.resolve()

    // 🔴 Không có `watch`, ô nhập giữ nguyên "Ten go do cho Chuong Mot" và một lượt bấm "Đổi
    // tên" sẽ áp nó lên Chương THỨ HAI — một lượt ghi dữ liệu người dùng, im lặng.
    // `CHAPTER_ROW_UNTITLED.title` là `null` ⇒ ô nhập về rỗng, không giữ chữ của hàng trước.
    expect(state.chapterRenameDraft.value).toBe('')
  })

  it('lượt nạp lại danh sách KHÔNG giẫm lên chữ đang gõ dở (watch nghe `chapter_id`, không nghe đối tượng hàng)', async () => {
    ketQuaFlush.value = 'clean'
    mockInvoke.mockResolvedValue([CHAPTER_ROW_A, CHAPTER_ROW_UNTITLED])
    const state = await import('../../src/modes/libraryChapters')
    await state.loadChapters()
    await Promise.resolve()

    state.chapterRenameDraft.value = 'Dang go do'
    // `loadChapters()` thay CẢ MẢNG -- một `watch` trên `currentLibraryChapter` sẽ bắn ở đây.
    await state.loadChapters()
    await Promise.resolve()

    expect(state.chapterRenameDraft.value).toBe('Dang go do')
  })
})

describe('modes/libraryChapters.ts::mergeCurrentChapterUp — con trỏ ở hàng ĐẦU (Story 5.8, lượt rà)', () => {
  it('🔴 con trỏ ở 0 ⇒ KHÔNG đọc nhầm hàng CUỐI làm "Chương liền trước" (`.at(-1)` vòng)', async () => {
    ketQuaFlush.value = 'clean'
    const CHUONG_CUOI = { chapter_id: 9, ord: 3, title: 'Chuong Cuoi', status: 'done', segment_count: 2 }
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_chapters') return Promise.resolve([CHAPTER_ROW_A, CHAPTER_ROW_UNTITLED, CHUONG_CUOI])
      if (cmd === 'merge_chapter_into_previous') return Promise.resolve(null)
      return Promise.resolve(null)
    })
    const state = await import('../../src/modes/libraryChapters')
    await state.loadChapters()
    // Con trỏ ở 0 -- `chapters.value.at(-1)` sẽ trả `CHUONG_CUOI`, KHÔNG `undefined`.
    expect(state.libraryChapterCursor.value).toBe(0)

    await state.mergeCurrentChapterUp()

    // Mệnh đề đáng đo: lượt gọi vẫn đi tới IPC với ĐÚNG hàng đang chọn, và cờ bận được nhả.
    // (Rust là nơi từ chối bằng `chapter.at_first`; webview không được tự đoán biên.)
    const goiMerge = mockInvoke.mock.calls.filter((c) => c[0] === 'merge_chapter_into_previous')
    expect(goiMerge).toHaveLength(1)
    expect(goiMerge[0]?.[1]).toEqual({ chapterId: CHAPTER_ROW_A.chapter_id })
    expect(state.libraryChapterReorgBusy.value).toBe(false)
  })
})

describe('panels/editorPanelState.ts::splitChapterHere — thân hàm, không chỉ vị từ caret (Story 5.8, lượt rà)', () => {
  it('🔴 caret `null` ⇒ 0 lượt invoke VÀ một câu hiện ra trên thanh trạng thái', async () => {
    const editor = await import('../../src/panels/editorPanelState')
    editor.resetEditorPanel()
    mockInvoke.mockClear()

    const ketQua = await editor.splitChapterHere()

    expect(ketQua).toBe(false)
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'split_chapter_at_segment')).toBe(false)
    // 🔴 Vế THỨ HAI, và nó là vế lượt rà thêm vào: một `console.error` không phải một câu trả
    // lời cho người dùng. Bản đầu để `editorSplitChapterNotice` là `null` ở đây.
    expect(editor.editorSplitChapterNotice.value).toBe('no-caret')
  })

  it('🔴 Rust TỪ CHỐI ⇒ câu `refused` mang đúng `IpcError`, không rơi về một câu chung', async () => {
    const LOI_TACH = {
      code: 'chapter.split_leaves_empty',
      message_key: 'err.chapter.split_leaves_empty',
      params: {},
      retryable: false,
    }
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'split_chapter_at_segment') return Promise.reject(LOI_TACH)
      return Promise.resolve(null)
    })
    const editor = await import('../../src/panels/editorPanelState')
    editor.resetEditorPanel()
    editor.setEditorCaret(11)

    const ketQua = await editor.splitChapterHere()

    expect(ketQua).toBe(false)
    expect(editor.editorSplitChapterNotice.value).toBe('refused')
    expect(editor.editorSplitChapterError.value?.code).toBe('chapter.split_leaves_empty')
  })

  it('tách THÀNH CÔNG ⇒ nạp lại segment/Chương rồi mới báo, và câu báo SỐNG SÓT lượt reset', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'split_chapter_at_segment') return Promise.resolve(null)
      if (cmd === 'read_open_chapter_segments') {
        return Promise.resolve({ chapter_id: 1, segments: [], caret_segment_id: null })
      }
      return Promise.resolve({ chapter_id: 1, source_text: '', source_lang: 'zh' })
    })
    const editor = await import('../../src/panels/editorPanelState')
    editor.resetEditorPanel()
    editor.setEditorCaret(11)

    const ketQua = await editor.splitChapterHere()

    expect(ketQua).toBe(true)
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'split_chapter_at_segment')).toBe(true)
    // 🔴 Đối chứng của một BẪY THẬT: `resetEditorPanel()` chạy TRONG hàm này và chính nó gọi
    // `datThongBao({})`. Ghi câu báo TRƯỚC lượt đó thì nó bị dọn ngay và người dùng không thấy
    // gì — ca này đỏ nếu thứ tự hai dòng ấy bị đảo.
    expect(editor.editorSplitChapterNotice.value).toBe('split')
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'read_open_chapter_segments')).toBe(true)
  })
})
