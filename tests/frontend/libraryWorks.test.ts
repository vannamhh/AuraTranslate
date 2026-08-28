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
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

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
      return Promise.resolve({ total: works.length, matched: works.length, works })
    }
    if (cmd === 'read_work_lifecycle') {
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

    const rows = wrapper.findAll('.works-row')
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
    expect(rowUnknown.find('.work-progress').text()).toBe('Chưa biết tiến độ')
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
