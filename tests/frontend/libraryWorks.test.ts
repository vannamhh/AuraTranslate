/**
 * `modes/libraryWorks.ts` + `config/library.ts::listLibraryWorks` + `config/lifecycle.ts` —
 * Story 5.4, FR5/FR6.
 *
 * ⚠️ **PHẠM VI** — `happy-dom` canh HÀNH VI của module thuần, không hình học/engine thật
 * (`tests/AGENTS.md`). Bốn mệnh đề dưới đây là những chỗ *"trông như đúng nhưng có thể sai
 * im lặng"* mà story này đặc biệt lo: adapter không bao giờ ném, bốn bộ lọc bật/tắt RIÊNG
 * RẼ, `matched = 0` với `total > 0` phân biệt được với "Library trống thật" (cả hai `0`), và
 * một hàng `status = null` không khớp bộ lọc nào.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

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

beforeEach(async () => {
  mockInvoke.mockReset()
  const state = await import('../../src/modes/libraryWorks')
  state.resetLibraryWorks()
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
    mockInvoke.mockResolvedValueOnce({ total: 0, matched: 0, works: [] })

    const { listLibraryWorks } = await import('../../src/config/library')
    await listLibraryWorks([])

    expect(mockInvoke).toHaveBeenCalledWith('library_list_works', { filter: null })
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
    mockInvoke.mockResolvedValueOnce({ status: 'done', status_is_override: false })

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
    mockInvoke.mockResolvedValueOnce({ total: 2, matched: 2, works: [WORK_ROW_A, WORK_ROW_B] })
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
    mockInvoke.mockResolvedValue({ total: 2, matched: 1, works: [WORK_ROW_A] })
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
    mockInvoke.mockResolvedValue({ total: 1, matched: 1, works: [WORK_ROW_B] })
    const state = await import('../../src/modes/libraryWorks')

    state.toggleStatusFilter('paused')
    await Promise.resolve()
    await Promise.resolve()

    expect(mockInvoke).toHaveBeenCalledWith('library_list_works', { filter: ['paused'] })
  })

  it('`clearStatusFilter()` bỏ mọi bộ lọc rồi tải lại KHÔNG lọc', async () => {
    mockInvoke.mockResolvedValue({ total: 2, matched: 2, works: [WORK_ROW_A, WORK_ROW_B] })
    const state = await import('../../src/modes/libraryWorks')

    state.toggleStatusFilter('paused')
    await Promise.resolve()
    await Promise.resolve()

    state.clearStatusFilter()
    await Promise.resolve()
    await Promise.resolve()

    expect(state.libraryStatusFilter.value.size).toBe(0)
    expect(mockInvoke).toHaveBeenLastCalledWith('library_list_works', { filter: null })
  })

  it('`clearStatusFilter()` là no-op (không gọi IPC thêm) khi đã không lọc gì', async () => {
    const state = await import('../../src/modes/libraryWorks')
    mockInvoke.mockClear()

    state.clearStatusFilter()
    await Promise.resolve()

    expect(mockInvoke).not.toHaveBeenCalled()
  })
})

describe('modes/libraryWorks.ts — `matched = 0` với `total > 0` phân biệt được với "trống thật"', () => {
  it('bộ lọc quét sạch: `worksMatched === 0` mà `worksTotal > 0`', async () => {
    mockInvoke.mockResolvedValueOnce({ total: 3, matched: 0, works: [] })
    const state = await import('../../src/modes/libraryWorks')

    await state.loadWorks()

    expect(state.libraryWorksHaveLoaded.value).toBe(true)
    expect(state.libraryWorksTotal.value).toBe(3)
    expect(state.libraryWorksMatched.value).toBe(0)
    expect(state.libraryWorks.value).toEqual([])
  })

  it('Library trống thật: CẢ HAI con số đều 0', async () => {
    mockInvoke.mockResolvedValueOnce({ total: 0, matched: 0, works: [] })
    const state = await import('../../src/modes/libraryWorks')

    await state.loadWorks()

    expect(state.libraryWorksTotal.value).toBe(0)
    expect(state.libraryWorksMatched.value).toBe(0)
  })
})

describe('modes/libraryWorks.ts — hàng `status = null` (chưa biết) vẫn có mặt khi KHÔNG lọc', () => {
  it('`loadWorks()` không lọc trả về cả hàng status = null', async () => {
    mockInvoke.mockResolvedValueOnce({ total: 2, matched: 2, works: [WORK_ROW_A, WORK_ROW_UNKNOWN] })
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
    mockInvoke.mockResolvedValueOnce({ status: 'paused', status_is_override: true })
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
    mockInvoke.mockResolvedValueOnce({ status: 'paused', status_is_override: true })
    // Lượt gọi kèm theo: `applyOpenWorkLifecycleResult` tự `loadWorks()` để đồng bộ danh
    // sách -- mock thêm một lượt cho lời gọi đó.
    mockInvoke.mockResolvedValueOnce({ total: 1, matched: 1, works: [WORK_ROW_B] })
    const state = await import('../../src/modes/libraryWorks')

    await state.setOpenWorkOverride()
    expect(state.openWorkLifecycleStatus.value).toBe('paused')
    expect(state.openWorkLifecycleIsOverride.value).toBe(true)
    expect(mockInvoke).toHaveBeenCalledWith('set_work_status_override', { status: 'paused' })

    mockInvoke.mockResolvedValueOnce({ status: 'done', status_is_override: false })
    mockInvoke.mockResolvedValueOnce({ total: 1, matched: 1, works: [WORK_ROW_A] })
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
      .mockResolvedValue({ total: 2, matched: 2, works: [WORK_ROW_A, WORK_ROW_B] })

    state.toggleStatusFilter('not_started')
    await Promise.resolve()
    // Lượt thứ hai rơi vào đúng cửa sổ "đang bận".
    state.toggleStatusFilter('paused')
    await Promise.resolve()
    expect(mockInvoke).toHaveBeenCalledTimes(1)

    releaseFirst({ total: 2, matched: 1, works: [WORK_ROW_A] })
    // Nhả cho chuỗi promise chạy hết: lượt đầu kết thúc rồi kéo theo lượt bị hoãn.
    for (let i = 0; i < 12; i += 1) await Promise.resolve()

    expect(mockInvoke).toHaveBeenCalledTimes(2)
    const secondCallArgs = mockInvoke.mock.calls[1]?.[1] as { filter?: string[] | null }
    expect(secondCallArgs.filter).toEqual(['not_started', 'paused'])
    // Và state cuối cùng phải là kết quả của lượt THỨ HAI, không phải lượt đầu.
    expect(state.libraryWorksMatched.value).toBe(2)
  })
})
