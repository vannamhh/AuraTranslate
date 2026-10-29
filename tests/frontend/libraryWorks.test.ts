/**
 * `modes/libraryWorks.ts` + `config/library.ts::listLibraryWorks` + `config/lifecycle.ts` —
 * Story 5.4, FR5/FR6. Cộng `modes/LibraryMode.vue` (khối "Tác phẩm") — Story 5.5, FR7.
 *
 * ⚠️ **PHẠM VI** — `happy-dom` canh HÀNH VI của module thuần, không hình học/engine thật
 * (`tests/AGENTS.md`). Bốn mệnh đề dưới đây là những chỗ *"trông như đúng nhưng có thể sai
 * im lặng"* mà story này đặc biệt lo: adapter không bao giờ ném, bốn bộ lọc bật/tắt RIÊNG
 * RẼ, `matched = 0` với `total > 0` phân biệt được với "Library trống thật" (cả hai `0`), và
 * một hàng `status = null` không khớp bộ lọc nào.
 *
 * Story 5.5 thêm khối cuối tệp: mount THẬT `LibraryMode.vue` (không gọi thẳng một hàm nội
 * bộ) và khẳng định ba nhánh hiển thị tiến độ — `happy-dom` đủ cho mệnh đề văn bản/thuộc
 * tính (`aria-value*`), KHÔNG đủ cho hình học (bề rộng thanh THẬT trên màn hình) — đúng ranh
 * giới `tests/AGENTS.md` đã vạch.
 */
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { CommandDeps } from '../../src/commands'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const WORK_ROW_A = {
  work_id: 'id-a',
  atproj_path: '/tmp/A.atproj',
  name: 'A',
  source_lang: 'zh',
  genre: '',
  created_at: '2026-08-01T00:00:00.000Z',
  updated_at: '2026-08-01T00:00:00.000Z',
  chapter_count: 1,
  status: 'not_started',
  status_is_override: false,
  // 🔵 THÊM (2026-08-28, Story 5.5) — `isWorkRowArray` nay đòi trường này (`number | null`,
  // KHÔNG `undefined`); thiếu nó làm MỌI ca ở tệp này (kể cả những ca không liên quan tiến
  // độ) đọc lên "hình dạng sai" và `listLibraryWorks` hồi phòng về lỗi -- đúng lớp lỗi mà
  // kiểm kiểu lúc chạy tồn tại để bắt.
  chapter_done_count: 0,
}

const WORK_ROW_B = {
  ...WORK_ROW_A,
  work_id: 'id-b',
  name: 'B',
  status: 'paused',
  status_is_override: true,
}

const WORK_ROW_UNKNOWN = {
  ...WORK_ROW_A,
  work_id: 'id-unknown',
  name: 'Unknown',
  status: null,
  status_is_override: false,
}

// Cùng lý do `libraryChapters.test.ts` — MỘT lượt `import('.../LibraryMode.vue')` đã kéo theo
// `libraryWorks`/`libraryChapters`/`commands`/`config/library`/`config/lifecycle` ở tầng
// module (đọc `LibraryMode.vue:15-89`, `libraryWorks.ts:29-31`); nạp đúng NÓ ở `beforeAll`
// thay vì sáu lượt `await` nối tiếp.
beforeAll(async () => {
  await import('../../src/modes/LibraryMode.vue')
})

beforeEach(async () => {
  mockInvoke.mockReset()
  const state = await import('../../src/modes/libraryWorks')
  state.resetLibraryWorks()
  // 🔵 THÊM Story 5.7 — `LibraryMode.vue` nay mount cả khối "Chương"; vứt state của nó cùng
  // lượt để một ca của tệp này không đọc lại kết quả của ca trước.
  const chapters = await import('../../src/modes/libraryChapters')
  chapters.resetLibraryChapters()
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('config/library.ts::listLibraryWorks — adapter không bao giờ ném', () => {
  it('trên một lỗi KHÔNG phải IpcError trả `{ report: null, error: <hồi phòng> }`, không ném', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('panic gia lap phia Rust'))
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true })

    const { listLibraryWorks } = await import('../../src/config/library')
    const outcome = await listLibraryWorks()

    expect(outcome.report).toBeNull()
    expect(outcome.error).not.toBeNull()
    expect(outcome.error?.code).toBe('ipc.unknown')

    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  })

  it('bộ lọc RỖNG gửi `filter: null` (không lọc), không một mảng rỗng', async () => {
    mockInvoke.mockResolvedValueOnce({ total: 0, matched: 0, works: [], genres: [], source_langs: [] })

    const { listLibraryWorks } = await import('../../src/config/library')
    // Gọi thẳng adapter, KHÔNG qua `genre`/`sourceLang`/`sort` -- cả ba đều `undefined` ⇒
    // adapter gửi `null` trên dây (`config/library.ts::listLibraryWorks`), khác lượt gọi qua
    // `libraryWorks.ts::loadWorks()` LUÔN gửi `sortKey.value` tường minh (xem describe dưới).
    await listLibraryWorks([])

    expect(mockInvoke).toHaveBeenCalledWith('library_list_works', {
      filter: null,
      genre: null,
      sourceLang: null,
      sort: null,
    })
  })

  /** 🔵 THÊM (2026-08-28, Story 5.6) — `genre`/`sourceLang`/`sort` đi trên dây đúng camelCase,
   * và một giá trị KHÔNG rỗng đi qua nguyên vẹn (không bị chuẩn hoá thành `null`). */
  it('genre/sourceLang/sort không rỗng đi qua nguyên vẹn, camelCase trên dây', async () => {
    mockInvoke.mockResolvedValueOnce({ total: 1, matched: 1, works: [WORK_ROW_A], genres: [], source_langs: [] })

    const { listLibraryWorks } = await import('../../src/config/library')
    await listLibraryWorks(undefined, 'Tien hiep', 'zh', 'name_asc')

    expect(mockInvoke).toHaveBeenCalledWith('library_list_works', {
      filter: null,
      genre: 'Tien hiep',
      sourceLang: 'zh',
      sort: 'name_asc',
    })
  })
})

describe('config/lifecycle.ts — adapter không bao giờ ném', () => {
  it('`readWorkLifecycle()` trên `err.work.none_open` trả đúng IpcError của Rust', async () => {
    const rustError = {
      code: 'work.none_open',
      message_key: 'err.work.none_open',
      params: {},
      retryable: false,
    }
    mockInvoke.mockRejectedValueOnce(rustError)

    const { readWorkLifecycle } = await import('../../src/config/lifecycle')
    const result = await readWorkLifecycle()

    expect(result.lifecycle).toBeNull()
    expect(result.error).toEqual(rustError)
  })

  it('`setChapterStatus()` gửi `chapterId`/`status` camelCase đúng chỗ gọi', async () => {
    mockInvoke.mockResolvedValueOnce({ status: 'done', status_is_override: false, derived_status: 'done' })

    const { setChapterStatus } = await import('../../src/config/lifecycle')
    await setChapterStatus(42, 'done')

    expect(mockInvoke).toHaveBeenCalledWith('set_chapter_status', { chapterId: 42, status: 'done' })
  })
})

describe('modes/libraryWorks.ts — `libraryWorksHaveLoaded` phải SAI trước lượt gọi đầu', () => {
  it('trạng thái ban đầu là "chưa biết", không phải "Library trống thật"', async () => {
    const state = await import('../../src/modes/libraryWorks')
    expect(state.libraryWorksHaveLoaded.value).toBe(false)
    expect(state.libraryWorks.value).toEqual([])
    expect(state.libraryWorksTotal.value).toBe(0)
    expect(state.libraryWorksMatched.value).toBe(0)
  })

  it('sau một lượt `loadWorks()` thành công, vị từ chuyển thành true và state cập nhật', async () => {
    mockInvoke.mockResolvedValueOnce({ total: 2, matched: 2, works: [WORK_ROW_A, WORK_ROW_B], genres: [], source_langs: [] })
    const state = await import('../../src/modes/libraryWorks')

    await state.loadWorks()

    expect(state.libraryWorksHaveLoaded.value).toBe(true)
    expect(state.libraryWorksTotal.value).toBe(2)
    expect(state.libraryWorksMatched.value).toBe(2)
    expect(state.libraryWorks.value).toHaveLength(2)
  })
})

describe('modes/libraryWorks.ts — bốn bộ lọc bật/tắt RIÊNG RẼ', () => {
  it('`toggleStatusFilter` thêm/bớt đúng MỘT giá trị, không đụng ba giá trị còn lại', async () => {
    mockInvoke.mockResolvedValue({ total: 2, matched: 1, works: [WORK_ROW_A], genres: [], source_langs: [] })
    const state = await import('../../src/modes/libraryWorks')

    state.toggleStatusFilter('not_started')
    await Promise.resolve()
    expect(state.libraryStatusFilter.value.has('not_started')).toBe(true)
    expect(state.libraryStatusFilter.value.size).toBe(1)

    state.toggleStatusFilter('paused')
    await Promise.resolve()
    expect(state.libraryStatusFilter.value.has('not_started')).toBe(true)
    expect(state.libraryStatusFilter.value.has('paused')).toBe(true)
    expect(state.libraryStatusFilter.value.size).toBe(2)

    // Tắt lại "not_started" -- "paused" phải CÒN NGUYÊN.
    state.toggleStatusFilter('not_started')
    await Promise.resolve()
    expect(state.libraryStatusFilter.value.has('not_started')).toBe(false)
    expect(state.libraryStatusFilter.value.has('paused')).toBe(true)
    expect(state.libraryStatusFilter.value.size).toBe(1)
  })

  it('mỗi lượt bật/tắt gọi lại `loadWorks()` với đúng danh sách lọc hiện thời', async () => {
    mockInvoke.mockResolvedValue({ total: 1, matched: 1, works: [WORK_ROW_B], genres: [], source_langs: [] })
    const state = await import('../../src/modes/libraryWorks')

    state.toggleStatusFilter('paused')
    await Promise.resolve()
    await Promise.resolve()

    expect(mockInvoke).toHaveBeenCalledWith(
      'library_list_works',
      { filter: ['paused'], genre: null, sourceLang: null, sort: 'updated_desc' },
    )
  })

  it('`clearStatusFilter()` bỏ mọi bộ lọc rồi tải lại KHÔNG lọc', async () => {
    mockInvoke.mockResolvedValue({ total: 2, matched: 2, works: [WORK_ROW_A, WORK_ROW_B], genres: [], source_langs: [] })
    const state = await import('../../src/modes/libraryWorks')

    state.toggleStatusFilter('paused')
    await Promise.resolve()
    await Promise.resolve()

    state.clearStatusFilter()
    await Promise.resolve()
    await Promise.resolve()

    expect(state.libraryStatusFilter.value.size).toBe(0)
    expect(mockInvoke).toHaveBeenLastCalledWith(
      'library_list_works',
      { filter: null, genre: null, sourceLang: null, sort: 'updated_desc' },
    )
  })

  it('`clearStatusFilter()` là no-op (không gọi IPC thêm) khi đã không lọc gì', async () => {
    const state = await import('../../src/modes/libraryWorks')
    mockInvoke.mockClear()

    state.clearStatusFilter()
    await Promise.resolve()

    expect(mockInvoke).not.toHaveBeenCalled()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.6 — "Lưới Tác phẩm, lọc và sắp xếp" (lĩnh vực · ngôn ngữ nguồn · sắp xếp · con trỏ).
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/libraryWorks.ts — bộ lọc lĩnh vực/ngôn ngữ nguồn + khoá sắp', () => {
  it('`setGenreFilter`/`setSourceLangFilter` gọi lại `loadWorks()` với tham số đúng', async () => {
    mockInvoke.mockResolvedValue({ total: 1, matched: 1, works: [WORK_ROW_A], genres: [], source_langs: [] })
    const state = await import('../../src/modes/libraryWorks')

    state.setGenreFilter('Tien hiep')
    await Promise.resolve()
    await Promise.resolve()
    expect(state.libraryGenreFilter.value).toBe('Tien hiep')
    expect(mockInvoke).toHaveBeenLastCalledWith('library_list_works', {
      filter: null,
      genre: 'Tien hiep',
      sourceLang: null,
      sort: 'updated_desc',
    })

    state.setSourceLangFilter('zh')
    await Promise.resolve()
    await Promise.resolve()
    expect(state.librarySourceLangFilter.value).toBe('zh')
    expect(mockInvoke).toHaveBeenLastCalledWith('library_list_works', {
      filter: null,
      genre: 'Tien hiep',
      sourceLang: 'zh',
      sort: 'updated_desc',
    })

    // Bỏ lọc lĩnh vực (giá trị `null`) -- tham số phải quay về `null` trên dây, KHÔNG một
    // chuỗi rỗng.
    state.setGenreFilter(null)
    await Promise.resolve()
    await Promise.resolve()
    expect(state.libraryGenreFilter.value).toBeNull()
    expect(mockInvoke).toHaveBeenLastCalledWith('library_list_works', {
      filter: null,
      genre: null,
      sourceLang: 'zh',
      sort: 'updated_desc',
    })
  })

  it('`setSortKey` đổi khoá sắp rồi tải lại (AC4)', async () => {
    mockInvoke.mockResolvedValue({ total: 1, matched: 1, works: [WORK_ROW_A], genres: [], source_langs: [] })
    const state = await import('../../src/modes/libraryWorks')

    state.setSortKey('name_asc')
    await Promise.resolve()
    await Promise.resolve()

    expect(state.librarySortKey.value).toBe('name_asc')
    expect(mockInvoke).toHaveBeenLastCalledWith('library_list_works', {
      filter: null,
      genre: null,
      sourceLang: null,
      sort: 'name_asc',
    })
  })

  it('đặt lại CÙNG giá trị đang có là no-op (không gọi IPC thêm)', async () => {
    const state = await import('../../src/modes/libraryWorks')
    mockInvoke.mockClear()

    state.setGenreFilter(null) // đã là `null` -- no-op.
    state.setSortKey('updated_desc') // đã là mặc định -- no-op.
    await Promise.resolve()

    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('`clearStatusFilter()` bỏ CẢ BA bộ lọc (trạng thái, lĩnh vực, ngôn ngữ nguồn)', async () => {
    mockInvoke.mockResolvedValue({ total: 2, matched: 2, works: [WORK_ROW_A, WORK_ROW_B], genres: [], source_langs: [] })
    const state = await import('../../src/modes/libraryWorks')

    state.toggleStatusFilter('paused')
    state.setGenreFilter('Tien hiep')
    state.setSourceLangFilter('zh')
    await Promise.resolve()
    await Promise.resolve()
    await Promise.resolve()
    await Promise.resolve()
    expect(state.libraryFilterIsEmpty.value).toBe(false)

    state.clearStatusFilter()
    await Promise.resolve()
    await Promise.resolve()

    expect(state.libraryStatusFilter.value.size).toBe(0)
    expect(state.libraryGenreFilter.value).toBeNull()
    expect(state.librarySourceLangFilter.value).toBeNull()
    expect(state.libraryFilterIsEmpty.value).toBe(true)
    expect(mockInvoke).toHaveBeenLastCalledWith('library_list_works', {
      filter: null,
      genre: null,
      sourceLang: null,
      sort: 'updated_desc',
    })
  })

  it('hai mảng lựa chọn (`libraryGenres`/`librarySourceLangs`) chép NGUYÊN VẸN từ Rust, KHÔNG suy từ `works` đã lọc', async () => {
    // Rust trả `genres`/`source_langs` mang giá trị KHÔNG có mặt trong `works` đã lọc — nếu
    // state suy chúng từ `works.value` (đúng lỗi AD-1 cấm), hai mảng dưới sẽ chỉ còn "Tien
    // hiep"/"zh" thay vì cả bốn giá trị Rust thật sự trả về.
    mockInvoke.mockResolvedValueOnce({
      total: 3,
      matched: 1,
      works: [WORK_ROW_A],
      genres: ['Tien hiep', 'Do thi'],
      source_langs: ['zh', 'en'],
    })
    const state = await import('../../src/modes/libraryWorks')

    await state.loadWorks()

    expect(state.libraryGenres.value).toEqual(['Tien hiep', 'Do thi'])
    expect(state.librarySourceLangs.value).toEqual(['zh', 'en'])
  })
})

describe('modes/libraryWorks.ts — con trỏ lưới (`workCursor`), AC7', () => {
  it('`nextWork`/`prevWork` di chuyển không vòng, và là no-op khi danh sách rỗng', async () => {
    const state = await import('../../src/modes/libraryWorks')

    // Danh sách RỖNG (chưa tải gì) -- cả hai phải là no-op, không ném.
    state.nextWork()
    state.prevWork()
    expect(state.libraryWorkCursor.value).toBe(0)

    mockInvoke.mockResolvedValueOnce({
      total: 3,
      matched: 3,
      works: [WORK_ROW_A, WORK_ROW_B, WORK_ROW_UNKNOWN],
      genres: [],
      source_langs: [],
    })
    await state.loadWorks()

    expect(state.libraryWorkCursor.value).toBe(0)
    expect(state.currentLibraryWork.value?.work_id).toBe('id-a')

    state.nextWork()
    expect(state.libraryWorkCursor.value).toBe(1)
    expect(state.currentLibraryWork.value?.work_id).toBe('id-b')

    state.nextWork()
    state.nextWork() // Đã ở ô cuối -- KHÔNG vòng về 0.
    expect(state.libraryWorkCursor.value).toBe(2)

    state.prevWork()
    expect(state.libraryWorkCursor.value).toBe(1)
  })

  it('con trỏ KẸP BIÊN về ô cuối cùng còn lại khi một lượt lọc làm danh sách ngắn đi', async () => {
    mockInvoke.mockResolvedValueOnce({
      total: 3,
      matched: 3,
      works: [WORK_ROW_A, WORK_ROW_B, WORK_ROW_UNKNOWN],
      genres: [],
      source_langs: [],
    })
    const state = await import('../../src/modes/libraryWorks')
    await state.loadWorks()

    state.nextWork()
    state.nextWork()
    expect(state.libraryWorkCursor.value).toBe(2) // Ô cuối, "Unknown".

    // Lượt lọc kế tiếp chỉ còn MỘT hàng -- con trỏ (2) giờ NGOÀI phạm vi.
    mockInvoke.mockResolvedValueOnce({ total: 3, matched: 1, works: [WORK_ROW_A], genres: [], source_langs: [] })
    state.setGenreFilter('Tien hiep')
    await Promise.resolve()
    await Promise.resolve()

    expect(state.libraryWorkCursor.value).toBe(0) // Kẹp về ô cuối cùng còn lại (chỉ số 0).
    expect(state.currentLibraryWork.value?.work_id).toBe('id-a') // LUÔN có đúng một ô đang chọn.
  })

  it('con trỏ về 0 và không ô nào đang chọn khi lượt lọc quét sạch danh sách', async () => {
    mockInvoke.mockResolvedValueOnce({
      total: 2,
      matched: 2,
      works: [WORK_ROW_A, WORK_ROW_B],
      genres: [],
      source_langs: [],
    })
    const state = await import('../../src/modes/libraryWorks')
    await state.loadWorks()
    state.nextWork()

    mockInvoke.mockResolvedValueOnce({ total: 2, matched: 0, works: [], genres: [], source_langs: [] })
    state.setGenreFilter('Khong co')
    await Promise.resolve()
    await Promise.resolve()

    expect(state.libraryWorkCursor.value).toBe(0)
    expect(state.currentLibraryWork.value).toBeNull()
    // `nextWork`/`prevWork` trên danh sách rỗng: no-op, không ném.
    expect(() => state.nextWork()).not.toThrow()
    expect(() => state.prevWork()).not.toThrow()
    expect(state.libraryWorkCursor.value).toBe(0)
  })
})

describe('modes/libraryWorks.ts — `matched = 0` với `total > 0` phân biệt được với "trống thật"', () => {
  it('bộ lọc quét sạch: `worksMatched === 0` mà `worksTotal > 0`', async () => {
    mockInvoke.mockResolvedValueOnce({ total: 3, matched: 0, works: [], genres: [], source_langs: [] })
    const state = await import('../../src/modes/libraryWorks')

    await state.loadWorks()

    expect(state.libraryWorksHaveLoaded.value).toBe(true)
    expect(state.libraryWorksTotal.value).toBe(3)
    expect(state.libraryWorksMatched.value).toBe(0)
    expect(state.libraryWorks.value).toEqual([])
  })

  it('Library trống thật: CẢ HAI con số đều 0', async () => {
    mockInvoke.mockResolvedValueOnce({ total: 0, matched: 0, works: [], genres: [], source_langs: [] })
    const state = await import('../../src/modes/libraryWorks')

    await state.loadWorks()

    expect(state.libraryWorksTotal.value).toBe(0)
    expect(state.libraryWorksMatched.value).toBe(0)
  })
})

describe('modes/libraryWorks.ts — hàng `status = null` (chưa biết) vẫn có mặt khi KHÔNG lọc', () => {
  it('`loadWorks()` không lọc trả về cả hàng status = null', async () => {
    mockInvoke.mockResolvedValueOnce({ total: 2, matched: 2, works: [WORK_ROW_A, WORK_ROW_UNKNOWN], genres: [], source_langs: [] })
    const state = await import('../../src/modes/libraryWorks')

    await state.loadWorks()

    expect(state.libraryWorks.value.some((w) => w.status === null)).toBe(true)
  })
})

describe('modes/libraryWorks.ts — trạng thái Tác phẩm đang mở', () => {
  it('`openWorkLifecycleLoaded` phải SAI trước lượt gọi đầu', async () => {
    const state = await import('../../src/modes/libraryWorks')
    expect(state.openWorkLifecycleLoaded.value).toBe(false)
  })

  it('`loadOpenWorkLifecycle()` thành công cập nhật status/is_override', async () => {
    mockInvoke.mockResolvedValueOnce({ status: 'paused', status_is_override: true, derived_status: 'in_progress' })
    const state = await import('../../src/modes/libraryWorks')

    await state.loadOpenWorkLifecycle()

    expect(state.openWorkLifecycleLoaded.value).toBe(true)
    expect(state.openWorkLifecycleStatus.value).toBe('paused')
    expect(state.openWorkLifecycleIsOverride.value).toBe(true)
  })

  it('`loadOpenWorkLifecycle()` trên `work.none_open` để lỗi vào `openWorkLifecycleError`, không ném', async () => {
    mockInvoke.mockRejectedValueOnce({
      code: 'work.none_open',
      message_key: 'err.work.none_open',
      params: {},
      retryable: false,
    })
    const state = await import('../../src/modes/libraryWorks')

    await expect(state.loadOpenWorkLifecycle()).resolves.toBeUndefined()
    expect(state.openWorkLifecycleError.value?.code).toBe('work.none_open')
    expect(state.openWorkLifecycleLoaded.value).toBe(false)
  })

  it('`setOpenWorkOverride()` rồi `clearOpenWorkOverride()` cập nhật đúng dấu ghi đè', async () => {
    mockInvoke.mockResolvedValueOnce({ status: 'paused', status_is_override: true, derived_status: 'in_progress' })
    // Lượt gọi kèm theo: `applyOpenWorkLifecycleResult` tự `loadWorks()` để đồng bộ danh
    // sách -- mock thêm một lượt cho lời gọi đó.
    mockInvoke.mockResolvedValueOnce({ total: 1, matched: 1, works: [WORK_ROW_B], genres: [], source_langs: [] })
    const state = await import('../../src/modes/libraryWorks')

    await state.setOpenWorkOverride()
    expect(state.openWorkLifecycleStatus.value).toBe('paused')
    expect(state.openWorkLifecycleIsOverride.value).toBe(true)
    expect(mockInvoke).toHaveBeenCalledWith('set_work_status_override', { status: 'paused' })

    mockInvoke.mockResolvedValueOnce({ status: 'done', status_is_override: false, derived_status: 'done' })
    mockInvoke.mockResolvedValueOnce({ total: 1, matched: 1, works: [WORK_ROW_A], genres: [], source_langs: [] })
    await state.clearOpenWorkOverride()
    expect(state.openWorkLifecycleStatus.value).toBe('done')
    expect(state.openWorkLifecycleIsOverride.value).toBe(false)
    expect(mockInvoke).toHaveBeenCalledWith('set_work_status_override', { status: null })
  })
})

describe('modes/libraryWorks.ts — một lượt tải lại bị chặn KHÔNG được nuốt', () => {
  // ⚠️ THÊM ở lượt rà 2026-08-28. `loadWorks()` từng `return` thẳng khi đang bận, nên bấm nút
  // lọc thứ hai trong lúc lượt đầu còn bay làm danh sách đứng lại ở BỘ LỌC CŨ trong khi các
  // nút đã hiện bộ lọc MỚI — màn hình khẳng định một kết quả không thuộc câu hỏi đang hiển
  // thị. Đối chứng để chạy lại: đổi nhánh `worksReloadPending = true` thành `return` trần ⇒
  // ca này phải ĐỎ ở lượt `invoke` thứ hai.
  it('bật bộ lọc thứ hai khi lượt đầu còn bay vẫn dẫn tới một lượt đọc với bộ lọc MỚI', async () => {
    const state = await import('../../src/modes/libraryWorks')

    // Lượt `invoke` đầu bị giữ lại cho tới khi ta thả ra — mô phỏng một round-trip IPC chậm.
    let releaseFirst: (value: unknown) => void = () => {}
    const firstInFlight = new Promise((resolve) => {
      releaseFirst = resolve
    })
    mockInvoke
      .mockImplementationOnce(() => firstInFlight)
      .mockResolvedValue({ total: 2, matched: 2, works: [WORK_ROW_A, WORK_ROW_B], genres: [], source_langs: [] })

    state.toggleStatusFilter('not_started')
    await Promise.resolve()
    // Lượt thứ hai rơi vào đúng cửa sổ "đang bận".
    state.toggleStatusFilter('paused')
    await Promise.resolve()
    expect(mockInvoke).toHaveBeenCalledTimes(1)

    releaseFirst({ total: 2, matched: 1, works: [WORK_ROW_A], genres: [], source_langs: [] })
    // Nhả cho chuỗi promise chạy hết: lượt đầu kết thúc rồi kéo theo lượt bị hoãn.
    for (let i = 0; i < 12; i += 1) await Promise.resolve()

    expect(mockInvoke).toHaveBeenCalledTimes(2)
    const secondCallArgs = mockInvoke.mock.calls[1]?.[1] as { filter?: string[] | null }
    expect(secondCallArgs.filter).toEqual(['not_started', 'paused'])
    // Và state cuối cùng phải là kết quả của lượt THỨ HAI, không phải lượt đầu.
    expect(state.libraryWorksMatched.value).toBe(2)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.5 — "Tiến độ Tác phẩm" (FR7), mount THẬT `LibraryMode.vue`.
// ═════════════════════════════════════════════════════════════════════════════════

const WORK_PROGRESS_ZERO_OF_TWO = {
  ...WORK_ROW_A,
  work_id: 'id-progress-0',
  name: 'Chua Xong Chuong Nao',
  chapter_count: 2,
  chapter_done_count: 0,
}

const WORK_PROGRESS_ONE_OF_TWO = {
  ...WORK_ROW_A,
  work_id: 'id-progress-1',
  name: 'Mot Tren Hai',
  chapter_count: 2,
  chapter_done_count: 1,
}

const WORK_PROGRESS_UNKNOWN = {
  ...WORK_ROW_A,
  work_id: 'id-progress-unknown',
  name: 'Chua Biet Tien Do',
  chapter_count: 3,
  chapter_done_count: null,
}

const WORK_PROGRESS_ZERO_CHAPTERS = {
  ...WORK_ROW_A,
  work_id: 'id-progress-zero-chapters',
  name: 'Khong Chuong Nao',
  chapter_count: 0,
  chapter_done_count: 0,
}

/**
 * 🔵 THÊM (2026-08-28, vòng rà thứ hai) — `chapter_done_count > chapter_count`. Dữ liệu chỉ
 * mục CŨ/HỎNG có thể mang tổ hợp này (vd. `chapter_count` giảm sau một lượt gộp Chương thủ
 * công trên đĩa mà `library-index.db` chưa kịp quét lại) — `progressPercent` phải kẹp về
 * `100%`, không tự nhân lên `250%`. Không có ca này, gỡ `Math.min(100, …)` khỏi
 * `LibraryMode.vue::progressPercent` vẫn xanh.
 */
const WORK_PROGRESS_OVERFLOW = {
  ...WORK_ROW_A,
  work_id: 'id-progress-overflow',
  name: 'Du So Da Xong',
  chapter_count: 2,
  chapter_done_count: 5,
}

/**
 * `invoke` giả CHUNG cho mount thật — `LibraryMode.vue::onActivated` gọi CẢ HAI
 * `library_list_works` (khối "Tác phẩm") VÀ `read_work_lifecycle` (khối "Tác phẩm đang mở"),
 * khác các `describe` trên chỉ gọi thẳng một hàm composable. `work.none_open` là câu trả lời
 * TRUNG TÍNH cho khối thứ hai — không Tác phẩm nào đang mở trong bàn đo này, và khối đó
 * không thuộc phạm vi của ca này.
 */
function mockInvokeForMount(works: unknown[]): void {
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'library_list_works') {
      return Promise.resolve({ total: works.length, matched: works.length, works, genres: [], source_langs: [] })
    }
    if (cmd === 'read_work_lifecycle') {
      return Promise.reject({
        code: 'work.none_open',
        message_key: 'err.work.none_open',
        params: {},
        retryable: false,
      })
    }
    // 🔵 THÊM Story 5.7 — `LibraryMode.vue::onActivated` nay cũng gọi `list_chapters` (khối
    // "Chương"). Đây không phải phạm vi của các ca ở tệp này (xem `libraryChapters.test.ts`);
    // trả cùng lỗi "chưa mở Tác phẩm nào" để không làm tràn console với một reject vô danh.
    if (cmd === 'list_chapters') {
      return Promise.reject({
        code: 'work.none_open',
        message_key: 'err.work.none_open',
        params: {},
        retryable: false,
      })
    }
    return Promise.reject(new Error(`invoke gia khong biet lenh: ${cmd}`))
  })
}

describe('modes/LibraryMode.vue — ba nhánh hiển thị tiến độ (mount thật)', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    // 🔴 BẮT BUỘC unmount ở MỌI đường ra, kể cả khi một `expect` phía trên ném — thiếu bước
    // này, `LibraryMode.vue::onBeforeUnmount` (chỗ gọi `releaseFocus('mode.library')`) không
    // chạy, và mọi ca mount SAU trong cùng tiến trình `vitest` sẽ ném vì `declareFocus` NÉM
    // khi owner TRÙNG (`src/commands/focus.ts`).
    wrapper?.unmount()
    wrapper = null
  })

  it('`Some(0)`/`Some(1)`/`null` render đúng ba nhánh, và `chapter_count = 0` không sinh NaN', async () => {
    mockInvokeForMount([
      WORK_PROGRESS_ZERO_OF_TWO,
      WORK_PROGRESS_ONE_OF_TWO,
      WORK_PROGRESS_UNKNOWN,
      WORK_PROGRESS_ZERO_CHAPTERS,
      WORK_PROGRESS_OVERFLOW,
    ])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    wrapper = mount(LibraryMode)

    // `onActivated` đã tự gọi `loadWorks()`/`loadOpenWorkLifecycle()` ngay ở LƯỢT MOUNT ĐẦU
    // (Vue gọi `activated` ngay sau `mounted`, kể cả khi không nằm trong một `<KeepAlive>`
    // thật — comment ở `LibraryMode.vue::onActivated` đã ghi rõ điều này) -- nhưng ca này
    // KHÔNG phụ thuộc vào đó: gọi `loadWorks()` tường minh để tất định, và `mockInvokeForMount`
    // chấp nhận bất kỳ số lượt gọi lặp lại nào một cách an toàn.
    await state.loadWorks()
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('.work-cell')
    expect(rows).toHaveLength(5)

    // Nhánh 1 — `Some(0)`: "0 / 2 Chương đã xong", thanh có mặt, `aria-valuenow="0"`.
    const rowZero = rows[0]!
    expect(rowZero.find('.work-progress').text()).toContain('0')
    expect(rowZero.find('.work-progress').text()).toContain('2')
    const barZero = rowZero.find('.work-progress-track')
    expect(barZero.exists()).toBe(true)
    expect(barZero.attributes('aria-valuemin')).toBe('0')
    expect(barZero.attributes('aria-valuemax')).toBe('2')
    expect(barZero.attributes('aria-valuenow')).toBe('0')
    expect(barZero.attributes('role')).toBe('progressbar')

    // Nhánh 2 — `Some(1)`: "1 / 2 Chương đã xong", `aria-valuenow="1"`.
    const rowOne = rows[1]!
    const barOne = rowOne.find('.work-progress-track')
    expect(barOne.exists()).toBe(true)
    expect(barOne.attributes('aria-valuenow')).toBe('1')
    expect(barOne.attributes('aria-valuemax')).toBe('2')

    // Nhánh 3 — `null`: câu "chưa biết", VÀ không vẽ thanh (§I/O Matrix: "không hiện 0 /").
    const rowUnknown = rows[2]!
    expect(rowUnknown.find('.work-progress').text()).toBe(`${WORK_PROGRESS_UNKNOWN.chapter_count} chương`)
    expect(rowUnknown.text()).not.toContain('Chưa biết')
    expect(rowUnknown.find('.work-progress-track').exists()).toBe(false)

    // `chapter_count = 0`: thanh VẪN vẽ (chapter_done_count = Some(0), không phải None), ở
    // 0% -- KHÔNG `NaN%` (chia cho 0 phải được chặn tường minh ở `progressPercent`).
    const rowZeroChapters = rows[3]!
    const barZeroChapters = rowZeroChapters.find('.work-progress-track')
    expect(barZeroChapters.exists()).toBe(true)
    expect(barZeroChapters.attributes('aria-valuemax')).toBe('0')
    const fill = rowZeroChapters.find('.work-progress-fill')
    expect(fill.exists()).toBe(true)
    const width = fill.attributes('style') ?? ''
    expect(width).not.toContain('NaN')
    expect(width).toContain('0%')

    // `chapter_done_count = 5 > chapter_count = 2` (dữ liệu chỉ mục cũ/hỏng): bề rộng phải bị
    // KẸP về `100%`, không tự nhân lên `250%`.
    const rowOverflow = rows[4]!
    const barOverflow = rowOverflow.find('.work-progress-track')
    expect(barOverflow.exists()).toBe(true)
    expect(barOverflow.attributes('aria-valuenow')).toBe('5')
    expect(barOverflow.attributes('aria-valuemax')).toBe('2')
    const fillOverflow = rowOverflow.find('.work-progress-fill')
    expect(fillOverflow.exists()).toBe(true)
    const widthOverflow = fillOverflow.attributes('style') ?? ''
    expect(widthOverflow).toContain('100%')
    expect(widthOverflow).not.toContain('250%')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.6 — lưới (AC2/AC5/AC6/AC7), mount THẬT `LibraryMode.vue`.
// ═════════════════════════════════════════════════════════════════════════════════

const WORK_EMPTY_NAME = {
  ...WORK_ROW_A,
  work_id: 'id-empty-name',
  name: '   ',
}

describe('modes/LibraryMode.vue — lưới Tác phẩm (mount thật), Story 5.6', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('AC2 — mỗi ô mang bốn thứ cùng lúc: khung bìa, tên, tiến độ, trạng thái', async () => {
    mockInvokeForMount([WORK_ROW_A])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    wrapper = mount(LibraryMode)
    await state.loadWorks()
    await wrapper.vm.$nextTick()

    const cell = wrapper.find('.work-cell')
    expect(cell.exists()).toBe(true)
    expect(cell.find('.work-cover').exists()).toBe(true)
    expect(cell.find('.work-cover .work-name').text()).toBe('A')
    expect(cell.find('.work-body .work-name').exists()).toBe(false)
    expect(cell.findAll('.work-name')).toHaveLength(1)
    expect(cell.find('.work-progress').exists()).toBe(true)
    expect(cell.find('.work-status').exists()).toBe(true)
  })

  it('nút trên bìa theo mockup: ô đang chọn mang ui-btn--cover-on, ô khác ui-btn--cover, "⋯" cùng bậc md', async () => {
    mockInvokeForMount([WORK_ROW_A, { ...WORK_ROW_A, work_id: 'w-b', name: 'B' }])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    wrapper = mount(LibraryMode)
    await state.loadWorks()
    await wrapper.vm.$nextTick()

    const cells = wrapper.findAll('.work-cell')
    expect(cells).toHaveLength(2)
    const current = state.libraryWorkCursor.value
    cells.forEach((cell, i) => {
      const go = cell.get('[data-library-continue-translation]').classes()
      expect(go).not.toContain('ui-btn--sm')
      expect(go).not.toContain('ui-btn--primary')
      expect(go).toContain(i === current ? 'ui-btn--cover-on' : 'ui-btn--cover')
      expect(go).not.toContain(i === current ? 'ui-btn--cover' : 'ui-btn--cover-on')
      const more = cell.get('[data-library-work-more] summary').classes()
      expect(more).toEqual(expect.arrayContaining(['ui-btn', 'ui-btn--cover-plain']))
      expect(more).not.toContain('ui-btn--quiet')
    })
  })

  it('bìa không còn chữ cái đầu; mỗi bìa mang một tông 1..6 ổn định theo work_id', async () => {
    mockInvokeForMount([WORK_EMPTY_NAME, { ...WORK_ROW_A, name: 'zebra' }])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    wrapper = mount(LibraryMode)
    await state.loadWorks()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.work-initial').exists()).toBe(false)
    const covers = wrapper.findAll('[data-library-work-cover]')
    expect(covers).toHaveLength(2)
    for (const cover of covers) {
      expect(cover.attributes('data-cover-tone')).toMatch(/^[1-6]$/)
      expect(cover.classes()).toContain(`work-cover--tone-${cover.attributes('data-cover-tone')}`)
    }
  })

  it('AC5 — khối giải thích hiện KHI VÀ CHỈ KHI đã tải xong VÀ total === 0', async () => {
    mockInvokeForMount([])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    wrapper = mount(LibraryMode)

    // TRƯỚC lượt tải đầu: "chưa biết", câu "Library chưa có Tác phẩm nào" KHÔNG được nói.
    expect(wrapper.find('.empty .big').exists()).toBe(false)

    await state.loadWorks()
    await wrapper.vm.$nextTick()

    // SAU lượt tải, total === 0: khối giải thích PHẢI hiện cùng hai lời mời mở lớp phủ nhập.
    expect(wrapper.find('.empty .big').exists()).toBe(true)
    expect(wrapper.find('.empty .small').exists()).toBe(true)
    expect(wrapper.findAll('[data-library-empty-invite]')).toHaveLength(2)
    expect(wrapper.find('.import-form').exists()).toBe(false)
  })

  it('AC5 — câu "Library chưa có Tác phẩm nào" KHÔNG xuất hiện khi ĐÃ có Tác phẩm', async () => {
    mockInvokeForMount([WORK_ROW_A])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    wrapper = mount(LibraryMode)
    await state.loadWorks()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.empty .big').exists()).toBe(false)
    expect(wrapper.find('.empty .small').exists()).toBe(false)
    expect(wrapper.find('[data-library-empty]').exists()).toBe(false)
    expect(wrapper.find('.import-form').exists()).toBe(false)
  })

  it('lời mời lần đầu: lọc ra 0, chưa tải và tải lỗi đều KHÔNG hiện; bỏ lọc thì hiện', async () => {
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    const invite = () => wrapper?.find('[data-library-empty]').exists() ?? false

    // Đã tải, total 3 nhưng bộ lọc cho 0 khớp: lưới rỗng, không có lời mời.
    mockInvoke.mockImplementation((cmd: string) =>
      cmd === 'library_list_works'
        ? Promise.resolve({ total: 3, matched: 0, works: [], genres: ['Tien hiep'], source_langs: [] })
        : Promise.reject({ code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false }),
    )
    wrapper = mount(LibraryMode)
    expect(invite()).toBe(false)
    const settle = async () => {
      await new Promise((resolve) => setTimeout(resolve, 0))
      await wrapper?.vm.$nextTick()
    }
    state.setGenreFilter('Tien hiep')
    await settle()
    expect(state.libraryShowsFirstRunInvitation.value).toBe(false)
    expect(invite()).toBe(false)
    expect(wrapper.find('.works-block .status').text()).toContain('3')

    // total === 0 nhưng đang có bộ lọc: vẫn không phải lần đầu.
    mockInvokeForMount([])
    await state.loadWorks()
    await settle()
    expect(state.libraryShowsFirstRunInvitation.value).toBe(false)
    expect(invite()).toBe(false)

    // Bỏ lọc ⇒ lời mời hiện.
    state.setGenreFilter(null)
    await settle()
    expect(invite()).toBe(true)

    // Đã tải rỗng rồi lượt tải sau lỗi: lời mời biến mất, lỗi sẵn có hiện.
    mockInvoke.mockImplementation(() => Promise.reject(new Error('panic gia lap')))
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true })
    await state.loadWorks()
    await settle()
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
    expect(state.libraryShowsFirstRunInvitation.value).toBe(false)
    expect(invite()).toBe(false)
    expect(wrapper.find('.works-block .error').text()).not.toBe('')
  })

  it('AC7 — ô đang chọn mang `aria-current`, và lệnh `library.work_next` di chuyển con trỏ qua `dispatch()`', async () => {
    mockInvokeForMount([WORK_ROW_A, WORK_ROW_B])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    const { dispatch, installCommands } = await import('../../src/commands')
    installCommands({
      nextLibraryWork: state.nextWork,
      prevLibraryWork: state.prevWork,
    } as CommandDeps)
    wrapper = mount(LibraryMode)
    await state.loadWorks()
    await wrapper.vm.$nextTick()

    const cells = wrapper.findAll('.work-cell')
    expect(cells[0]!.attributes('aria-current')).toBe('true')
    expect(cells[1]!.attributes('aria-current')).toBeUndefined()

    dispatch('library.work_next')
    await wrapper.vm.$nextTick()

    const cellsAfter = wrapper.findAll('.work-cell')
    expect(cellsAfter[0]!.attributes('aria-current')).toBeUndefined()
    expect(cellsAfter[1]!.attributes('aria-current')).toBe('true')
  })

  it('chip Lĩnh vực/Ngôn ngữ/Sắp xếp dựng radio từ `WorkListReport.genres`/`source_langs`, KHÔNG suy từ `works`', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_list_works') {
        return Promise.resolve({
          total: 1,
          matched: 1,
          works: [WORK_ROW_A],
          genres: ['Tien hiep', 'Do thi'],
          source_langs: ['zh', 'en'],
        })
      }
      return Promise.reject({ code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false })
    })

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    wrapper = mount(LibraryMode)
    await state.loadWorks()
    await wrapper.vm.$nextTick()

    const optionValues = (chip: string): string[] =>
      wrapper!
        .findAll(`[data-library-chip="${chip}"] [data-library-chip-option]`)
        .map((o) => o.attributes('data-library-chip-option') ?? '')
    expect(optionValues('genre')).toEqual(['', 'Tien hiep', 'Do thi'])
    expect(optionValues('source_lang')).toEqual(['', 'zh', 'en'])
    expect(optionValues('sort')).toEqual(['updated_desc', 'name_asc'])
    expect(wrapper.find('select').exists()).toBe(false)
  })

  it('chip Lĩnh vực hiện nhãn cho định danh trong danh sách và nguyên văn cho giá trị lạ, cả ở lựa chọn lẫn nút chip', async () => {
    vi.resetModules()
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_list_works') {
        return Promise.resolve({
          total: 1,
          matched: 1,
          works: [WORK_ROW_A],
          genres: ['xianxia', 'Huyền huyễn'],
          source_langs: ['zh'],
        })
      }
      return Promise.reject({ code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false })
    })
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    wrapper = mount(LibraryMode)
    await state.loadWorks()
    await wrapper.vm.$nextTick()

    const chip = '[data-library-chip="genre"]'
    const labels = wrapper.findAll(`${chip} .select-item`).map((o) => o.text())
    expect(labels).toEqual(['Mọi lĩnh vực', 'Tiên hiệp', 'Huyền huyễn'])
    const trigger = () => wrapper!.get(`${chip} .menu-label`).text()

    await wrapper.get(`${chip} [data-library-chip-option="xianxia"]`).setValue(true)
    await wrapper.vm.$nextTick()
    expect(trigger()).toBe('Tiên hiệp')

    await wrapper.get(`${chip} [data-library-chip-option="Huyền huyễn"]`).setValue(true)
    await wrapper.vm.$nextTick()
    expect(trigger()).toBe('Huyền huyễn')
  })

  it('bấm một thẻ ngắm nó (con trỏ + aria-current + tabindex 0), bấm đúp dispatch library.open_work', async () => {
    vi.resetModules()
    mockInvokeForMount([WORK_ROW_A, WORK_ROW_B])
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    const { installCommands } = await import('../../src/commands')
    const openLibraryWork = vi.fn()
    installCommands({ openCurrentLibraryWork: openLibraryWork } as unknown as CommandDeps)
    wrapper = mount(LibraryMode)
    await state.loadWorks()
    await wrapper.vm.$nextTick()

    const cells = wrapper.findAll('.work-cell')
    await cells[1]!.trigger('mousedown')
    expect(state.libraryWorkCursor.value).toBe(1)
    const after = wrapper.findAll('.work-cell')
    expect(after[1]!.attributes('aria-current')).toBe('true')
    expect(after[1]!.attributes('tabindex')).toBe('0')
    expect(after[0]!.attributes('tabindex')).toBe('-1')

    await after[1]!.trigger('dblclick')
    expect(openLibraryWork).toHaveBeenCalledTimes(1)
  })

  it('mũi tên ±1 và Enter trên thẻ đi qua lệnh, kẹp ở hai đầu; Shift+Enter không bị lưới nuốt', async () => {
    vi.resetModules()
    mockInvokeForMount([WORK_ROW_A, WORK_ROW_B])
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    const { installCommands } = await import('../../src/commands')
    const openLibraryWork = vi.fn()
    installCommands({
      nextLibraryWork: state.nextWork,
      prevLibraryWork: state.prevWork,
      openCurrentLibraryWork: openLibraryWork,
    } as unknown as CommandDeps)
    wrapper = mount(LibraryMode, { attachTo: document.body })
    await state.loadWorks()
    await wrapper.vm.$nextTick()

    const cellAt = (index: number) => wrapper!.findAll('.work-cell')[index]!
    await cellAt(0).trigger('keydown', { key: 'ArrowRight' })
    expect(state.libraryWorkCursor.value).toBe(1)
    await cellAt(1).trigger('keydown', { key: 'ArrowRight' })
    expect(state.libraryWorkCursor.value).toBe(1)
    await cellAt(1).trigger('keydown', { key: 'ArrowLeft' })
    expect(state.libraryWorkCursor.value).toBe(0)
    await cellAt(0).trigger('keydown', { key: 'ArrowUp' })
    expect(state.libraryWorkCursor.value).toBe(0)

    await cellAt(0).trigger('keydown', { key: 'Enter' })
    expect(openLibraryWork).toHaveBeenCalledTimes(1)
    await cellAt(0).trigger('keydown', { key: 'Enter', shiftKey: true })
    expect(openLibraryWork).toHaveBeenCalledTimes(1)
  })
})

describe('modes/LibraryMode.vue — lưới rộng: mũi tên dọc, nhãn trạng thái, nút tiếp tục dịch', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  async function mountGrid(rows: unknown[]) {
    vi.resetModules()
    mockInvokeForMount(rows)
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    const { installCommands } = await import('../../src/commands')
    installCommands({
      nextLibraryWork: state.nextWork,
      prevLibraryWork: state.prevWork,
    } as unknown as CommandDeps)
    wrapper = mount(LibraryMode, { attachTo: document.body })
    await state.loadWorks()
    await wrapper.vm.$nextTick()
    return state
  }

  it('mũi tên lên/xuống đi một hàng theo số cột của lưới, kẹp ở hai đầu', async () => {
    const rows = Array.from({ length: 11 }, (_, index) => ({
      ...WORK_ROW_A,
      work_id: `id-${index}`,
      name: `W${index}`,
    }))
    const realGetComputedStyle = window.getComputedStyle.bind(window)
    vi.spyOn(window, 'getComputedStyle').mockImplementation((element: Element, pseudo?: string | null) => {
      if (element.hasAttribute('data-library-grid')) {
        return { gridTemplateColumns: '262px 262px 262px 262px 262px' } as CSSStyleDeclaration
      }
      return realGetComputedStyle(element, pseudo)
    })
    const state = await mountGrid(rows)
    const cellAt = (index: number) => wrapper!.findAll('.work-cell')[index]!

    await cellAt(0).trigger('keydown', { key: 'ArrowDown' })
    expect(state.libraryWorkCursor.value).toBe(5)
    await cellAt(5).trigger('keydown', { key: 'ArrowUp' })
    expect(state.libraryWorkCursor.value).toBe(0)
    await cellAt(0).trigger('keydown', { key: 'ArrowDown' })
    await cellAt(5).trigger('keydown', { key: 'ArrowDown' })
    expect(state.libraryWorkCursor.value).toBe(10)
    await cellAt(10).trigger('keydown', { key: 'ArrowDown' })
    expect(state.libraryWorkCursor.value).toBe(10)
  })

  it('một status ngoài bốn giá trị vòng đời vẫn hiện chữ `works_status_invalid`, status null không hiện nhãn', async () => {
    const { t } = await import('../../src/i18n')
    await mountGrid([
      { ...WORK_ROW_A, work_id: 'id-bogus', name: 'Bogus', status: 'bogus' },
      { ...WORK_ROW_A, work_id: 'id-null', name: 'Null', status: null },
    ])
    const cells = wrapper!.findAll('.work-cell')
    const badge = (index: number) => cells[index]!.find('[data-library-work-status]')

    expect(badge(0).exists()).toBe(true)
    expect(badge(0).text()).toBe(t('mode.library.works_status_invalid', { status: 'bogus' }))
    expect(badge(1).exists()).toBe(false)
  })

  it('thẻ đang chọn của Tác phẩm chưa bắt đầu ghi "Bắt đầu dịch", đang dịch ghi "Tiếp tục dịch"', async () => {
    const { t } = await import('../../src/i18n')
    const state = await mountGrid([
      { ...WORK_ROW_A, work_id: 'id-new', name: 'New', status: 'not_started' },
      { ...WORK_ROW_A, work_id: 'id-going', name: 'Going', status: 'in_progress' },
    ])
    const labelOfCurrent = (): string =>
      wrapper!
        .findAll('.work-cell')
        [state.libraryWorkCursor.value]!.find('[data-library-continue-translation]')
        .text()

    expect(labelOfCurrent()).toBe(t('mode.library.start_translation'))
    await wrapper!.findAll('.work-cell')[1]!.trigger('mousedown')
    expect(state.libraryWorkCursor.value).toBe(1)
    expect(labelOfCurrent()).toBe(t('mode.library.continue_translation'))
  })
})

describe('modes/libraryWorks.ts::loadWorks — con trỏ theo work_id khi tải lại', () => {
  const rowsReply = (works: unknown[]) => ({
    total: works.length,
    matched: works.length,
    works,
    genres: [],
    source_langs: [],
  })

  it('đổi thứ tự danh sách thì con trỏ đi theo Tác phẩm đã ngắm', async () => {
    const state = await import('../../src/modes/libraryWorks')
    mockInvokeForMount([WORK_ROW_A, WORK_ROW_B])
    await state.loadWorks()
    state.aimLibraryWork(1)
    expect(state.currentLibraryWork.value?.work_id).toBe('id-b')

    mockInvoke.mockImplementation((cmd: string) =>
      cmd === 'library_list_works' ? Promise.resolve(rowsReply([WORK_ROW_B, WORK_ROW_A])) : Promise.reject(new Error(cmd)),
    )
    await state.loadWorks()

    expect(state.libraryWorkCursor.value).toBe(0)
    expect(state.currentLibraryWork.value?.work_id).toBe('id-b')
  })

  it('Tác phẩm đã ngắm biến mất thì con trỏ kẹp về ô cuối', async () => {
    const state = await import('../../src/modes/libraryWorks')
    mockInvokeForMount([WORK_ROW_A, WORK_ROW_B])
    await state.loadWorks()
    state.aimLibraryWork(1)

    mockInvoke.mockImplementation((cmd: string) =>
      cmd === 'library_list_works' ? Promise.resolve(rowsReply([WORK_ROW_A])) : Promise.reject(new Error(cmd)),
    )
    await state.loadWorks()

    expect(state.libraryWorkCursor.value).toBe(0)
    expect(state.currentLibraryWork.value?.work_id).toBe('id-a')
  })
})

describe('modes/LibraryMode.vue — nút Nhập bản reviewer trên ô Tác phẩm', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('mỗi thẻ có nút trong menu ⋯, và nút mang dấu mở để tiêu điểm về đúng nó', async () => {
    mockInvokeForMount([WORK_ROW_A, WORK_ROW_B])
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryWorks')
    wrapper = mount(LibraryMode)
    await state.loadWorks()
    await wrapper.vm.$nextTick()

    const buttons = wrapper.findAll('[data-library-work-reviewer-import]')
    expect(buttons).toHaveLength(2)
    expect(buttons[0]!.attributes('data-reviewer-import-open')).toBeDefined()
    expect(buttons[0]!.element.closest('.work-cell')?.querySelector('.work-name')?.textContent).toBe('A')
    expect(buttons[0]!.attributes('disabled')).toBeUndefined()
  })
})

describe('config adapters: chapter_started refresh', () => {
  const LIFECYCLE = { status: 'in_progress', status_is_override: false, derived_status: 'in_progress' }
  const PROMOTE = {
    segment_id: 5,
    target_text: 'x',
    translation_origin: 'self',
    status: 'draft',
    needs_confirmation: false,
    unsigned_draft: null,
  }
  const RANGE = { segmentId: 5, expectedText: 'a', start: 0, end: 1, expectedPhrase: 'a', replacement: 'b', kind: 'spelling' as const }

  type Case = { name: string; wire: Record<string, unknown>; run: () => Promise<unknown> }
  const CASES: Case[] = [
    {
      name: 'save_segment_targets',
      wire: { chapter_id: 7, saved: 1 },
      run: async () => (await import('../../src/config/segment')).saveSegmentTargets(7, []),
    },
    {
      name: 'restore_segment_version',
      wire: { segment_id: 5, status: 'draft', restored: true, needs_confirmation: false, unsigned_draft: null },
      run: async () => (await import('../../src/config/segment')).restoreSegmentVersion(5, 1, false),
    },
    {
      name: 'promote_ai_translation',
      wire: PROMOTE,
      run: async () => (await import('../../src/config/segment')).promoteAiTranslation(5, 'x', false),
    },
    {
      name: 'confirm_segment',
      wire: { segment_id: 5, status: 'confirmed', version_created: true },
      run: async () => (await import('../../src/config/segment')).confirmSegment(5),
    },
    {
      name: 'accept_tm_fuzzy',
      wire: PROMOTE,
      run: async () => (await import('../../src/config/segment')).acceptTmFuzzy(5, 'global', 1, 'x', false),
    },
    {
      name: 'accept_tm_exact',
      wire: PROMOTE,
      run: async () => (await import('../../src/config/segment')).acceptTmExact(5, 'global', 1, 'x', false),
    },
    {
      name: 'review_accept_change',
      wire: PROMOTE,
      run: async () => (await import('../../src/config/alignment')).reviewAcceptChange(7, 1, 'x', false),
    },
    {
      name: 'ai_proofread_accept',
      wire: { ...PROMOTE, decision_id: 3 },
      run: async () => (await import('../../src/config/proofread')).acceptProofreadSuggestion(RANGE),
    },
    {
      name: 'ai_proofread_revert',
      wire: PROMOTE,
      run: async () => (await import('../../src/config/proofread')).revertProofreadSuggestion({ ...RANGE, decisionId: 3 }),
    },
  ]

  function wireInvoke(cmd: string, wire: Record<string, unknown>, started: boolean, refresh: () => unknown) {
    mockInvoke.mockImplementation(async (name: string) => {
      if (name === cmd) return { ...wire, chapter_started: started }
      if (name === 'refresh_after_chapter_started') return refresh()
      if (name === 'list_chapters') return []
      throw new Error(`unexpected invoke ${name}`)
    })
  }
  const calls = (): string[] => mockInvoke.mock.calls.map((c) => String(c[0]))

  for (const c of CASES) {
    it(`${c.name}: chapter_started true refreshes once, then reloads the chapter list`, async () => {
      wireInvoke(c.name, c.wire, true, () => LIFECYCLE)
      await c.run()
      await vi.waitFor(() => expect(calls()).toEqual([c.name, 'refresh_after_chapter_started', 'list_chapters']))
    })

    it(`${c.name}: chapter_started false never refreshes`, async () => {
      wireInvoke(c.name, c.wire, false, () => LIFECYCLE)
      await c.run()
      await new Promise((resolve) => setTimeout(resolve, 20))
      expect(calls()).toEqual([c.name])
    })
  }

  it('a failed refresh skips the reload and never throws into the writer', async () => {
    const error = { code: 'work.create_failed', message_key: 'err.work.create_failed', params: {}, retryable: false }
    vi.spyOn(console, 'error').mockImplementation(() => {})
    wireInvoke('save_segment_targets', { chapter_id: 7, saved: 1 }, true, () => {
      throw error
    })
    const result = (await CASES[0]!.run()) as { outcome: unknown; error: unknown }
    expect(result.error).toBeNull()
    await vi.waitFor(() => expect(calls()).toEqual(['save_segment_targets', 'refresh_after_chapter_started']))
    await new Promise((resolve) => setTimeout(resolve, 20))
    expect(calls()).toEqual(['save_segment_targets', 'refresh_after_chapter_started'])
  })

  it('an outcome without the flag is a malformed shape, not a silent false', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {})
    mockInvoke.mockResolvedValue({ chapter_id: 7, saved: 1 })
    const result = (await CASES[0]!.run()) as { outcome: unknown; error: { code: string } | null }
    expect(result.outcome).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })
})
