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
  conflicts: 0,
  skipped: 0,
  orphans: [{ work_id: 'id-orphan', name: 'Ghost Work', atproj_path: '/tmp/library/Ghost.atproj' }],
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
    const result = await forgetLibraryOrphan('id-x')

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

describe('modes/libraryRescan.ts — con trỏ mồ côi kẹp lại sau khi một mục bị gỡ', () => {
  it('gỡ mục ĐANG CHỌN (cuối danh sách) thì con trỏ lùi về mục còn lại, không rơi ra ngoài phạm vi', async () => {
    const twoOrphans = {
      root: '/tmp/library',
      root_missing: false,
      indexed: 0,
      conflicts: 0,
      skipped: 0,
      orphans: [
        { work_id: 'id-a', name: 'A', atproj_path: '/tmp/A.atproj' },
        { work_id: 'id-b', name: 'B', atproj_path: '/tmp/B.atproj' },
      ],
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
      conflicts: 0,
      skipped: 0,
      orphans: [],
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
      conflicts: 0,
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
})
