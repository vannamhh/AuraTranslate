/**
 * `modes/libraryRescan.ts` + `config/library.ts` — Story 5.3, FR99.
 *
 * ⚠️ **PHẠM VI** — `happy-dom` canh HÀNH VI của module thuần, không hình học/engine thật
 * (`tests/AGENTS.md`). Bốn mệnh đề dưới đây là những chỗ *"trông như đúng nhưng có thể sai
 * im lặng"* mà story này đặc biệt lo: adapter không bao giờ ném, vị từ `…HasLoaded` đúng
 * TRƯỚC lượt gọi đầu (AGENTS.md::Known pitfalls — danh sách rỗng phải nói vì sao rỗng), con
 * trỏ mồ côi kẹp lại sau khi danh sách co lại, và `forgetOrphan` không gọi IPC khi chưa có
 * mục nào để gỡ.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const RESCAN_REPORT_ONE_ORPHAN = {
  root: '/tmp/library',
  root_missing: false,
  indexed: 2,
  conflicts: [],
  skipped: 0,
  orphans: [{ work_id: 'id-orphan', name: 'Ghost Work', atproj_path: '/tmp/library/Ghost.atproj' }],
  // 🔵 THÊM (retro Epic 5, AI-2/AI-3 — 2026-09-03) -- trường mới, bắt buộc trên dây.
  text_skipped: [],
}

beforeEach(async () => {
  mockInvoke.mockReset()
  const state = await import('../../src/modes/libraryRescan')
  state.resetLibraryRescan()
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('config/library.ts — adapter không bao giờ ném', () => {
  it('`rescanLibrary()` trên một lỗi KHÔNG phải IpcError trả `{ report: null, error: <hồi phòng> }`, không ném', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('panic gia lap phia Rust'))
    // Có cầu IPC: giả lập `window.__TAURI_INTERNALS__` để đi vào nhánh "lỗi thật", không
    // nhánh "chạy ngoài Tauri".
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true });

    const { rescanLibrary } = await import('../../src/config/library')
    const outcome = await rescanLibrary()

    expect(outcome.report).toBeNull()
    expect(outcome.error).not.toBeNull()
    expect(outcome.error?.code).toBe('ipc.unknown')

    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  })

  it('`forgetLibraryOrphan()` mang đúng một IpcError của Rust khi Rust trả `library.not_orphaned`', async () => {
    const rustError = {
      code: 'library.not_orphaned',
      message_key: 'err.library.not_orphaned',
      params: { work_id: 'id-x' },
      retryable: false,
    }
    mockInvoke.mockRejectedValueOnce(rustError)

    const { forgetLibraryOrphan } = await import('../../src/config/library')
    const result = await forgetLibraryOrphan('id-x', 'Ten hien thi')

    expect(result.orphans).toBeNull()
    expect(result.error).toEqual(rustError)
  })
})

describe('modes/libraryRescan.ts — `libraryScanHasLoadedState` phải SAI trước lượt gọi đầu', () => {
  it('trạng thái ban đầu là "chưa biết", không phải "không có mục mồ côi nào"', async () => {
    const state = await import('../../src/modes/libraryRescan')
    // 🔴 Đây chính là vị từ mà LibraryMode.vue phải hỏi TRƯỚC khi kết luận "không có mục mồ
    // côi nào" — danh sách rỗng ở đây KHÔNG được đọc thành "đã quét, thật sự không có gì".
    expect(state.libraryScanHasLoadedState.value).toBe(false)
    expect(state.libraryOrphans.value).toEqual([])
  })

  it('sau một lượt `rescanLibraryFolder()` thành công, vị từ chuyển thành true và state cập nhật', async () => {
    mockInvoke.mockResolvedValueOnce(RESCAN_REPORT_ONE_ORPHAN)
    const state = await import('../../src/modes/libraryRescan')

    await state.rescanLibraryFolder()

    expect(state.libraryScanHasLoadedState.value).toBe(true)
    expect(state.currentLibraryRoot.value).toBe('/tmp/library')
    expect(state.libraryIndexedCount.value).toBe(2)
    expect(state.libraryOrphans.value).toHaveLength(1)
    expect(state.currentLibraryOrphan.value?.work_id).toBe('id-orphan')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// `loadLibraryOrphans()` đọc mồ côi KHÔNG cần một lượt Quét lại
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/libraryRescan.ts::loadLibraryOrphans — đọc mồ côi hiện có, không gọi `library_rescan`', () => {
  it('gọi ĐÚNG `library_list_orphans`, không tham số, và bật `libraryScanHasLoadedState`', async () => {
    mockInvoke.mockResolvedValueOnce([{ work_id: 'id-orphan', name: 'Ghost Work', atproj_path: '/tmp/library/Ghost.atproj' }])
    const state = await import('../../src/modes/libraryRescan')

    expect(state.libraryScanHasLoadedState.value).toBe(false)
    await state.loadLibraryOrphans()

    expect(mockInvoke).toHaveBeenCalledTimes(1)
    expect(mockInvoke).toHaveBeenCalledWith('library_list_orphans')
    expect(state.libraryScanHasLoadedState.value).toBe(true)
    expect(state.libraryOrphans.value).toHaveLength(1)
    expect(state.currentLibraryOrphan.value?.work_id).toBe('id-orphan')
  })

  it('một lượt `rescanLibraryFolder()` MỚI HƠN bắt đầu trong lúc `loadLibraryOrphans()` còn bay: kết quả CŨ không ghi đè', async () => {
    const state = await import('../../src/modes/libraryRescan')
    let resolveOrphans: (value: unknown) => void = () => {}
    mockInvoke.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveOrphans = resolve
        }),
    )

    const loading = state.loadLibraryOrphans()
    mockInvoke.mockResolvedValueOnce(RESCAN_REPORT_ONE_ORPHAN)
    await state.rescanLibraryFolder()
    expect(state.libraryOrphans.value).toHaveLength(1)
    expect(state.currentLibraryRoot.value).toBe('/tmp/library')

    // Lượt `loadLibraryOrphans()` CŨ hơn giờ mới về -- danh sách rỗng của nó không được ghi
    // đè lên state mà `rescanLibraryFolder()` (MỚI hơn) vừa đặt.
    resolveOrphans([])
    await loading

    expect(state.libraryOrphans.value).toHaveLength(1)
    expect(state.currentLibraryRoot.value).toBe('/tmp/library')
  })

  it('KHÔNG bật `libraryRescanResultHasLoadedState` — chỉ mồ côi được đọc, ba-con-số Quét lại/`rootMissing` chưa từng được đo', async () => {
    mockInvoke.mockResolvedValueOnce([])
    const state = await import('../../src/modes/libraryRescan')

    expect(state.libraryRescanResultHasLoadedState.value).toBe(false)
    await state.loadLibraryOrphans()

    // `libraryScanHasLoadedState` (mồ côi) lên, nhưng cờ ba-con-số Quét lại đứng RIÊNG vẫn
    // `false` -- `loadLibraryOrphans()` không chạm `indexedCount`/`conflictCount`/
    // `skippedCount`/`textSkippedCount`/`rootMissing`, nên LibraryMode.vue không được phép
    // đọc chúng như một kết quả THẬT chỉ vì mồ côi vừa được đọc.
    expect(state.libraryScanHasLoadedState.value).toBe(true)
    expect(state.libraryRescanResultHasLoadedState.value).toBe(false)
  })
})

describe('modes/libraryRescan.ts — con trỏ mồ côi kẹp lại sau khi một mục bị gỡ', () => {
  it('gỡ mục ĐANG CHỌN (cuối danh sách) thì con trỏ lùi về mục còn lại, không rơi ra ngoài phạm vi', async () => {
    const twoOrphans = {
      root: '/tmp/library',
      root_missing: false,
      indexed: 0,
      conflicts: [],
      skipped: 0,
      orphans: [
        { work_id: 'id-a', name: 'A', atproj_path: '/tmp/A.atproj' },
        { work_id: 'id-b', name: 'B', atproj_path: '/tmp/B.atproj' },
      ],
      text_skipped: [],
    }
    mockInvoke.mockResolvedValueOnce(twoOrphans)
    const state = await import('../../src/modes/libraryRescan')
    await state.rescanLibraryFolder()

    // Con trỏ vào mục CUỐI ("B").
    state.nextLibraryOrphan()
    expect(state.libraryOrphanCursor.value).toBe(1)
    expect(state.currentLibraryOrphan.value?.work_id).toBe('id-b')

    // Gỡ "B" -- Rust trả danh sách mồ côi CÒN LẠI (chỉ "A").
    mockInvoke.mockResolvedValueOnce([{ work_id: 'id-a', name: 'A', atproj_path: '/tmp/A.atproj' }])
    await state.forgetCurrentLibraryOrphan()

    expect(state.libraryOrphans.value).toHaveLength(1)
    // 🔴 Con trỏ PHẢI kẹp lại về chỉ số hợp lệ cuối cùng (0), không giữ nguyên 1 (ngoài phạm vi).
    expect(state.libraryOrphanCursor.value).toBe(0)
    expect(state.currentLibraryOrphan.value?.work_id).toBe('id-a')
  })
})

describe('modes/libraryRescan.ts — `forgetCurrentLibraryOrphan` không gọi IPC khi chưa chọn mục nào', () => {
  it('danh sách mồ côi rỗng ⇒ 0 lượt gọi `invoke`', async () => {
    const state = await import('../../src/modes/libraryRescan')
    // Chưa quét lần nào -- `currentLibraryOrphan` là `null`.
    expect(state.currentLibraryOrphan.value).toBeNull()

    await state.forgetCurrentLibraryOrphan()

    expect(mockInvoke).not.toHaveBeenCalled()
  })
})

describe('P1 (vòng rà bốn lớp, 2026-08-27) — `root_missing` phải phân biệt "gốc vắng mặt" với "gốc rỗng thật"', () => {
  it('`rescanLibraryFolder()` với `root_missing: true` cập nhật `libraryRootMissing`, giữ nguyên `currentLibraryRoot`', async () => {
    mockInvoke.mockResolvedValueOnce({
      root: '/gốc-đã-mất',
      root_missing: true,
      indexed: 0,
      conflicts: [],
      skipped: 0,
      orphans: [],
      text_skipped: [],
    })
    const state = await import('../../src/modes/libraryRescan')

    await state.rescanLibraryFolder()

    expect(state.libraryRootMissing.value).toBe(true)
    expect(state.currentLibraryRoot.value).toBe('/gốc-đã-mất')
    expect(state.libraryScanHasLoadedState.value).toBe(true)
  })

  it('một lượt quét THÀNH CÔNG trên gốc CÒN tồn tại phải đặt `libraryRootMissing` về `false`', async () => {
    mockInvoke.mockResolvedValueOnce(RESCAN_REPORT_ONE_ORPHAN) // root_missing: false
    const state = await import('../../src/modes/libraryRescan')

    await state.rescanLibraryFolder()

    expect(state.libraryRootMissing.value).toBe(false)
  })
})

describe('config/library.ts — kiểm KIỂU LÚC CHẠY cho `RescanReport` (P1)', () => {
  it('một `RescanReport` THIẾU `root_missing` (hình dạng cũ) bị từ chối, không chuyển tiếp im lặng', async () => {
    // Mô phỏng đúng ca "Rust đổi lược đồ mà TS chưa cập nhật kiểu" -- hình dạng CŨ, trước P1.
    mockInvoke.mockResolvedValueOnce({
      root: '/tmp/library',
      indexed: 1,
      conflicts: [],
      skipped: 0,
      orphans: [],
    })
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true })

    const { rescanLibrary } = await import('../../src/config/library')
    const result = await rescanLibrary()

    expect(result.report).toBeNull()
    expect(result.error).not.toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')

    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  })

  /**
   * **THÊM (2026-08-27, phán quyết Ice #3)** — đối chứng GỠ bắt buộc: nén dữ liệu xung đột
   * trở lại thành một con số (hình dạng TRƯỚC phán quyết #3, khi `conflicts` còn là
   * `number`) phải làm guard từ chối, không chuyển tiếp im lặng. Không có ca này, một lượt
   * Rust lùi về hình dạng cũ (hoặc một chỗ gọi TS quên cập nhật) sẽ đi qua `isRescanReport`
   * và đổ một con số trần vào chỗ đang mong một mảng — đúng lớp lỗi "bộ test xanh không
   * chứng minh chỗ nối được canh" mà `AGENTS.md::Known pitfalls` gọi tên.
   */
  it('một `RescanReport` với `conflicts` là SỐ (hình dạng cũ, trước phán quyết Ice #3) bị từ chối', async () => {
    mockInvoke.mockResolvedValueOnce({
      root: '/tmp/library',
      root_missing: false,
      indexed: 1,
      conflicts: 2, // hình dạng CŨ -- một `usize` trần, không còn hợp lệ.
      skipped: 0,
      orphans: [],
    })
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true })

    const { rescanLibrary } = await import('../../src/config/library')
    const result = await rescanLibrary()

    expect(result.report).toBeNull()
    expect(result.error).not.toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')

    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  })

  /**
   * **THÊM (retro Epic 5, vòng rà lần hai, mục 2 — 2026-09-03)** — đối chứng GỠ cho
   * `isTextSkippedEntryArray`: bản đầu chỉ đọc `value[0]` rồi kết luận cho cả mảng (đúng lỗ
   * hổng mà `isSearchHitArray` đã ghi lại bằng chữ ở `config/library.ts`). Mục ĐẦU hợp lệ,
   * mục THỨ HAI sai hình dạng (`reason` là số thay vì chuỗi) -- nếu guard chỉ soi mục đầu, ca
   * này sẽ đi qua trót lọt.
   */
  it('`RescanReport.text_skipped` có mục THỨ HAI sai hình dạng ⇒ cả báo cáo bị từ chối', async () => {
    mockInvoke.mockResolvedValueOnce({
      root: '/tmp/library',
      root_missing: false,
      indexed: 2,
      conflicts: [],
      skipped: 0,
      orphans: [],
      text_skipped: [
        { work_id: 'id-a', reason: 'schema_too_old' },
        { work_id: 'id-b', reason: 42 }, // hình dạng SAI -- `reason` phải là chuỗi.
      ],
    })
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true })

    const { rescanLibrary } = await import('../../src/config/library')
    const result = await rescanLibrary()

    expect(result.report).toBeNull()
    expect(result.error).not.toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')

    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  })
})

describe('phán quyết Ice #3 (2026-08-27) — dữ liệu xung đột có cấu trúc, không chỉ một con số', () => {
  it('`rescanLibraryFolder()` với `conflicts` chở hai mục cập nhật `libraryConflicts`/`firstLibraryConflict`/`libraryConflictCount`', async () => {
    mockInvoke.mockResolvedValueOnce({
      root: '/tmp/library',
      root_missing: false,
      indexed: 1,
      conflicts: [
        { work_id: 'id-dup', kept_path: '/tmp/library/A.atproj', duplicate_path: '/tmp/library/B.atproj' },
        { work_id: 'id-dup-2', kept_path: '/tmp/library/C.atproj', duplicate_path: '/tmp/library/D.atproj' },
      ],
      skipped: 0,
      orphans: [],
      text_skipped: [],
    })
    const state = await import('../../src/modes/libraryRescan')

    await state.rescanLibraryFolder()

    expect(state.libraryConflictCount.value).toBe(2)
    expect(state.libraryConflicts.value).toHaveLength(2)
    expect(state.firstLibraryConflict.value).toEqual({
      work_id: 'id-dup',
      kept_path: '/tmp/library/A.atproj',
      duplicate_path: '/tmp/library/B.atproj',
    })
  })

  it('không có xung đột nào ⇒ `firstLibraryConflict` là `null`, `libraryConflicts` rỗng', async () => {
    mockInvoke.mockResolvedValueOnce(RESCAN_REPORT_ONE_ORPHAN) // conflicts: []
    const state = await import('../../src/modes/libraryRescan')

    await state.rescanLibraryFolder()

    expect(state.firstLibraryConflict.value).toBeNull()
    expect(state.libraryConflicts.value).toEqual([])
  })
})

describe('P1 vòng rà THỨ HAI (2026-08-27) — huỷ hộp thoại chọn thư mục KHÔNG được là một lỗi', () => {
  it('config/library.ts: `chooseLibraryRoot()` với `invoke` trả `null` ⇒ ba trạng thái rỗng, không lỗi', async () => {
    mockInvoke.mockResolvedValueOnce(null)
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true })

    const { chooseLibraryRoot } = await import('../../src/config/library')
    const result = await chooseLibraryRoot()

    // 🔴 Đúng bug đã ĐO: bản trước gọi CHUNG `callRescan`, và `isRescanReport(null)` trả
    // `false` khiến nhánh này từng trả `{ report: null, error: UNKNOWN_IPC_ERROR }`.
    expect(result.report).toBeNull()
    expect(result.error).toBeNull()

    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  })

  it('modes/libraryRescan.ts: `chooseLibraryRootFolder()` sau một lượt HUỶ giữ nguyên toàn bộ state đã có từ lượt quét trước', async () => {
    // Lượt quét ĐẦU -- thành công, dựng state có thật để đối chứng "giữ nguyên".
    mockInvoke.mockResolvedValueOnce(RESCAN_REPORT_ONE_ORPHAN)
    const state = await import('../../src/modes/libraryRescan')
    await state.rescanLibraryFolder()
    expect(state.libraryRescanError.value).toBeNull()
    const rootBefore = state.currentLibraryRoot.value
    const orphansBefore = state.libraryOrphans.value
    const indexedBefore = state.libraryIndexedCount.value
    const conflictsBefore = state.libraryConflictCount.value
    const skippedBefore = state.librarySkippedCount.value

    // Lượt "Đổi thư mục gốc" thứ hai -- người dùng HUỶ hộp thoại.
    mockInvoke.mockResolvedValueOnce(null)
    await state.chooseLibraryRootFolder()

    expect(state.libraryRescanError.value).toBeNull()
    expect(state.currentLibraryRoot.value).toBe(rootBefore)
    expect(state.libraryOrphans.value).toBe(orphansBefore)
    expect(state.libraryIndexedCount.value).toBe(indexedBefore)
    expect(state.libraryConflictCount.value).toBe(conflictsBefore)
    expect(state.librarySkippedCount.value).toBe(skippedBefore)
  })
})

describe('retro Epic 5, AI-2/AI-3 (2026-09-03) — text_skipped không còn bị VỨT', () => {
  it('`rescanLibraryFolder()` với `text_skipped` chở hai mục cập nhật `libraryTextSkippedCount`', async () => {
    mockInvoke.mockResolvedValueOnce({
      root: '/tmp/library',
      root_missing: false,
      indexed: 47,
      conflicts: [],
      skipped: 0,
      orphans: [],
      text_skipped: [
        { work_id: 'id-old-1', reason: 'schema_too_old' },
        { work_id: 'id-old-2', reason: 'schema_too_old' },
      ],
    })
    const state = await import('../../src/modes/libraryRescan')

    await state.rescanLibraryFolder()

    expect(state.libraryTextSkippedCount.value).toBe(2)
  })

  it('`resetLibraryRescan()` đưa `libraryTextSkippedCount` về 0', async () => {
    mockInvoke.mockResolvedValueOnce({
      root: '/tmp/library',
      root_missing: false,
      indexed: 1,
      conflicts: [],
      skipped: 0,
      orphans: [],
      text_skipped: [{ work_id: 'id-old-1', reason: 'schema_too_old' }],
    })
    const state = await import('../../src/modes/libraryRescan')
    await state.rescanLibraryFolder()
    expect(state.libraryTextSkippedCount.value).toBe(1)

    state.resetLibraryRescan()

    expect(state.libraryTextSkippedCount.value).toBe(0)
  })
})

describe('P10 (vòng rà THỨ HAI, 2026-08-27) — hai cửa đồng thời của libraryRescan.ts', () => {
  it('cửa CHỐNG TÁI NHẬP: `rescanLibraryFolder()` gọi lần hai trong khi lần đầu còn đang bay bị bỏ qua ngay, không thêm một lượt `invoke` nào', async () => {
    const state = await import('../../src/modes/libraryRescan')

    let resolveFirst: (value: unknown) => void = () => {}
    const pending = new Promise((resolve) => {
      resolveFirst = resolve
    })
    mockInvoke.mockReturnValueOnce(pending)

    const first = state.rescanLibraryFolder() // chưa resolve -- `rescanBusy` đang `true`.
    expect(state.libraryRescanBusy.value).toBe(true)

    // Lượt gọi THỨ HAI trong khi lượt đầu còn đang bay -- `if (rescanBusy.value) return`
    // phải chặn nó NGAY, không gọi `invoke` thêm lần nào.
    await state.rescanLibraryFolder()
    expect(mockInvoke).toHaveBeenCalledTimes(1)

    resolveFirst(RESCAN_REPORT_ONE_ORPHAN)
    await first
    expect(state.libraryRescanBusy.value).toBe(false)
  })

  it('cửa BỎ KẾT QUẢ CŨ: `resetLibraryRescan()` giữa một lượt IPC đang bay chặn kết quả CŨ ghi đè lên state đã reset', async () => {
    const state = await import('../../src/modes/libraryRescan')

    let resolveFirst: (value: unknown) => void = () => {}
    const pending = new Promise((resolve) => {
      resolveFirst = resolve
    })
    mockInvoke.mockReturnValueOnce(pending)

    const inFlight = state.rescanLibraryFolder() // chưa resolve.

    // Đặt lại TOÀN BỘ state GIỮA CHỪNG -- `resetLibraryRescan()` không đi qua cửa `rescanBusy`
    // (nó không phải một trong ba thao tác IPC), và nó tăng `sequence` -- đúng cơ chế mà
    // `mySequence !== sequence` tồn tại để bắt.
    state.resetLibraryRescan()
    expect(state.libraryScanHasLoadedState.value).toBe(false)
    expect(state.currentLibraryRoot.value).toBeNull()

    // Lượt IPC CŨ giờ mới trả về -- kết quả của nó KHÔNG được phép ghi đè state đã reset.
    resolveFirst(RESCAN_REPORT_ONE_ORPHAN)
    await inFlight

    expect(state.libraryScanHasLoadedState.value).toBe(false)
    expect(state.currentLibraryRoot.value).toBeNull()
    expect(state.libraryOrphans.value).toEqual([])
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// AC — "Given the Library root has an orphan, when Library mode is activated, then the
// orphan shows without a Rescan."
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/LibraryMode.vue — khối "Mồ côi" hiện đúng ngay khi vào Library, không cần bấm Quét lại (mount thật)', () => {
  it('sau `loadLibraryOrphans()` (đúng lời gọi `onActivated` phát ra), khối "Mồ côi" hiện mục — không một lượt `library_rescan` nào chạy', async () => {
    const { mount } = await import('@vue/test-utils')
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_list_orphans') {
        return Promise.resolve([{ work_id: 'id-orphan', name: 'Ghost Work', atproj_path: '/tmp/library/Ghost.atproj' }])
      }
      if (cmd === 'library_list_works') {
        return Promise.resolve({ total: 0, matched: 0, works: [], genres: [], source_langs: [] })
      }
      if (cmd === 'read_work_lifecycle' || cmd === 'list_chapters') {
        return Promise.reject({ code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false })
      }
      return Promise.reject(new Error(`invoke gia khong biet lenh: ${cmd}`))
    })

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/libraryRescan')
    const wrapper = mount(LibraryMode)

    // ⚠️ `mount()` của `@vue/test-utils` không chạy tin cậy hook `onActivated` trong môi
    // trường `happy-dom` này (không một `<KeepAlive>` thật bọc ngoài) — cùng bẫy mà
    // `libraryWorks.test.ts` đã ghi lại bằng chữ ("ca này KHÔNG phụ thuộc vào đó: gọi
    // `loadWorks()` tường minh để tất định"). Gọi thẳng [`loadLibraryOrphans`] — ĐÚNG hàm mà
    // `LibraryMode.vue::onActivated` gọi — để ca này tất định, không phụ thuộc cơ chế kích
    // hoạt của test harness. Việc `onActivated` THẬT SỰ gọi hàm này là một dòng mã một dòng,
    // xác nhận bằng đọc mã (`LibraryMode.vue`), không bằng mount ở đây.
    await state.loadLibraryOrphans()
    await wrapper.vm.$nextTick()

    // 🔴 Chưa lượt `library_rescan` nào -- khối "Mồ côi" hiện được là nhờ lượt ĐỌC thuần
    // `library_list_orphans`, không nhờ một lượt Quét lại ngầm.
    expect(mockInvoke.mock.calls.some((call) => call[0] === 'library_rescan')).toBe(false)
    expect(mockInvoke.mock.calls.some((call) => call[0] === 'library_list_orphans')).toBe(true)
    expect(wrapper.get('.orphan-name').text()).toBe('Ghost Work')

    wrapper.unmount()
  })
})
