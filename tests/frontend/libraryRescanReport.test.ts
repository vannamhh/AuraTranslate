import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const ORPHAN = { work_id: 'id-lost', name: 'Lost', atproj_path: '/lib/Lost.atproj' }

const BASE_REPORT = {
  root: '/lib',
  root_missing: false,
  indexed: 2,
  conflicts: [],
  skipped: 0,
  orphans: [],
  text_skipped: [],
  appeared: [],
  moved: [],
  newly_orphaned: [],
  rebuilt_at: '2026-10-13T08:00:00Z',
}

const CHANGED_REPORT = {
  ...BASE_REPORT,
  orphans: [ORPHAN],
  appeared: [{ work_id: 'id-new', name: 'New', atproj_path: '/lib/New.atproj' }],
  moved: [{ work_id: 'id-mv', name: 'Mv', from_path: '/lib/A.atproj', to_path: '/lib/B.atproj' }],
  newly_orphaned: [ORPHAN],
}

const WORK_ROW = {
  work_id: 'id-open',
  atproj_path: '/lib/Open.atproj',
  name: 'Open',
  source_lang: 'zh',
  genre: '',
  created_at: '2026-08-01T00:00:00.000Z',
  updated_at: '2026-08-01T00:00:00.000Z',
  chapter_count: 1,
  status: 'not_started',
  status_is_override: false,
  chapter_done_count: 0,
}

beforeEach(async () => {
  mockInvoke.mockReset()
  const state = await import('../../src/modes/libraryRescan')
  state.resetLibraryRescan()
  const works = await import('../../src/modes/libraryWorks')
  works.resetLibraryWorks()
})

describe('config/library.ts::isRescanReport — hình dạng báo cáo mới', () => {
  it.each(['appeared', 'moved', 'newly_orphaned', 'rebuilt_at'])(
    'báo cáo thiếu `%s` (hình dạng cũ) bị từ chối, không chuyển tiếp im lặng',
    async (field) => {
      const { [field]: _dropped, ...legacy } = CHANGED_REPORT as Record<string, unknown>
      mockInvoke.mockResolvedValueOnce(legacy)
      Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true })
      const { rescanLibrary } = await import('../../src/config/library')
      const result = await rescanLibrary()
      Reflect.deleteProperty(window, '__TAURI_INTERNALS__')

      expect(result.report).toBeNull()
      expect(result.error).not.toBeNull()
    },
  )

  it('một mục `moved` thiếu `to_path` làm hỏng cả báo cáo', async () => {
    const broken = { ...CHANGED_REPORT, moved: [{ work_id: 'x', name: 'x', from_path: '/a' }] }
    mockInvoke.mockResolvedValueOnce(broken)
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true })
    const { rescanLibrary } = await import('../../src/config/library')
    const result = await rescanLibrary()
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')

    expect(result.report).toBeNull()
  })

  it('`rebuilt_at: null` hợp lệ và được giữ là null', async () => {
    mockInvoke.mockResolvedValueOnce({ ...BASE_REPORT, rebuilt_at: null })
    const { rescanLibrary } = await import('../../src/config/library')
    const result = await rescanLibrary()

    expect(result.error).toBeNull()
    expect(result.report?.rebuilt_at).toBeNull()
  })
})

describe('modes/libraryRescan.ts — ba danh sách và mốc dựng', () => {
  it('trước mọi lượt quét: ba danh sách rỗng, mốc dựng null, vị từ kết quả sai', async () => {
    const state = await import('../../src/modes/libraryRescan')

    expect(state.libraryRescanResultHasLoadedState.value).toBe(false)
    expect(state.libraryAppeared.value).toEqual([])
    expect(state.libraryMoved.value).toEqual([])
    expect(state.libraryNewlyOrphaned.value).toEqual([])
    expect(state.libraryRebuiltAt.value).toBeNull()
    expect(state.libraryRebuiltAtLabel.value).toBeNull()
  })

  it('sau một lượt quét có thay đổi, state mang đúng ba danh sách và mốc', async () => {
    mockInvoke.mockResolvedValueOnce(CHANGED_REPORT)
    const state = await import('../../src/modes/libraryRescan')
    await state.rescanLibraryFolder()

    expect(state.libraryAppeared.value.map((e) => e.work_id)).toEqual(['id-new'])
    expect(state.libraryMoved.value.map((e) => [e.from_path, e.to_path])).toEqual([['/lib/A.atproj', '/lib/B.atproj']])
    expect(state.libraryNewlyOrphaned.value.map((e) => e.work_id)).toEqual(['id-lost'])
    expect(state.libraryRebuiltAt.value).toBe('2026-10-13T08:00:00Z')
    expect(state.libraryRebuiltAtLabel.value).not.toBeNull()
    expect(state.libraryRebuiltAtLabel.value).not.toMatch(/Invalid/)
  })

  it('báo cáo rỗng chỉ có nghĩa "không đổi" khi vị từ đã bật sau lượt quét thật', async () => {
    mockInvoke.mockResolvedValueOnce(BASE_REPORT)
    const state = await import('../../src/modes/libraryRescan')

    expect(state.libraryRescanResultHasLoadedState.value).toBe(false)
    await state.rescanLibraryFolder()

    expect(state.libraryRescanResultHasLoadedState.value).toBe(true)
    expect(state.libraryAppeared.value).toEqual([])
    expect(state.libraryMoved.value).toEqual([])
    expect(state.libraryNewlyOrphaned.value).toEqual([])
  })

  it('`loadLibraryOrphans()` không bật vị từ kết quả, nên không thể đọc thành "không đổi"', async () => {
    mockInvoke.mockResolvedValueOnce([])
    const state = await import('../../src/modes/libraryRescan')
    await state.loadLibraryOrphans()

    expect(state.libraryRescanResultHasLoadedState.value).toBe(false)
  })

  it('`loadLibraryRebuiltAt()` đặt mốc từ lệnh đọc khi chưa có lượt quét nào', async () => {
    mockInvoke.mockResolvedValueOnce('2026-10-13T07:00:00Z')
    Object.defineProperty(window, '__TAURI_INTERNALS__', { value: {}, configurable: true })
    const state = await import('../../src/modes/libraryRescan')
    await state.loadLibraryRebuiltAt()
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')

    expect(mockInvoke).toHaveBeenCalledWith('library_index_rebuilt_at')
    expect(state.libraryRebuiltAt.value).toBe('2026-10-13T07:00:00Z')
    expect(state.libraryRescanResultHasLoadedState.value).toBe(false)
  })

  it('`loadLibraryRebuiltAt()` nhận null thì giữ mốc là null', async () => {
    mockInvoke.mockResolvedValueOnce(null)
    const state = await import('../../src/modes/libraryRescan')
    await state.loadLibraryRebuiltAt()

    expect(state.libraryRebuiltAt.value).toBeNull()
  })

  it('`rebuilt_at: null` giữ mốc là null, không "Invalid Date"', async () => {
    mockInvoke.mockResolvedValueOnce({ ...BASE_REPORT, rebuilt_at: null })
    const state = await import('../../src/modes/libraryRescan')
    await state.rescanLibraryFolder()

    expect(state.libraryRebuiltAt.value).toBeNull()
    expect(state.libraryRebuiltAtLabel.value).toBeNull()
  })

  it('mốc không phân tích được hiện nguyên chuỗi thô, không "Invalid Date"', async () => {
    mockInvoke.mockResolvedValueOnce({ ...BASE_REPORT, rebuilt_at: 'khong-phai-ngay' })
    const state = await import('../../src/modes/libraryRescan')
    await state.rescanLibraryFolder()

    expect(state.libraryRebuiltAtLabel.value).toBe('khong-phai-ngay')
  })

  it('`resetLibraryRescan()` xoá ba danh sách, mốc và vị từ', async () => {
    mockInvoke.mockResolvedValueOnce(CHANGED_REPORT)
    const state = await import('../../src/modes/libraryRescan')
    await state.rescanLibraryFolder()
    state.resetLibraryRescan()

    expect(state.libraryAppeared.value).toEqual([])
    expect(state.libraryMoved.value).toEqual([])
    expect(state.libraryNewlyOrphaned.value).toEqual([])
    expect(state.libraryRebuiltAt.value).toBeNull()
    expect(state.libraryRescanResultHasLoadedState.value).toBe(false)
  })

  it('mỗi báo cáo áp vào state tăng `libraryIndexRevision`', async () => {
    mockInvoke.mockResolvedValue(BASE_REPORT)
    const state = await import('../../src/modes/libraryRescan')
    const before = state.libraryIndexRevision.value
    await state.rescanLibraryFolder()
    await state.rebuildLibraryIndexFolder()

    expect(state.libraryIndexRevision.value).toBe(before + 2)
  })
})

describe('modes/libraryRescan.ts::relocateCurrentLibraryOrphan', () => {
  it('gọi `library_relocate_orphan` với `workId` của mục đang chọn và áp báo cáo', async () => {
    const state = await import('../../src/modes/libraryRescan')
    mockInvoke.mockResolvedValueOnce({ ...CHANGED_REPORT, orphans: [ORPHAN] })
    await state.rescanLibraryFolder()
    mockInvoke.mockResolvedValueOnce({ ...BASE_REPORT, orphans: [], moved: [{ work_id: 'id-lost', name: 'Lost', from_path: '/x', to_path: '/lib/Lost.atproj' }] })

    await state.relocateCurrentLibraryOrphan()

    expect(mockInvoke).toHaveBeenLastCalledWith('library_relocate_orphan', { workId: 'id-lost' })
    expect(state.libraryOrphans.value).toEqual([])
    expect(state.libraryMoved.value).toHaveLength(1)
  })

  it('huỷ hộp thoại (`null`) giữ nguyên toàn bộ state và không báo lỗi', async () => {
    const state = await import('../../src/modes/libraryRescan')
    mockInvoke.mockResolvedValueOnce(CHANGED_REPORT)
    await state.rescanLibraryFolder()
    const revision = state.libraryIndexRevision.value
    mockInvoke.mockResolvedValueOnce(null)

    await state.relocateCurrentLibraryOrphan()

    expect(state.libraryRescanError.value).toBeNull()
    expect(state.libraryOrphans.value).toHaveLength(1)
    expect(state.libraryAppeared.value).toHaveLength(1)
    expect(state.libraryMoved.value).toHaveLength(1)
    expect(state.libraryIndexRevision.value).toBe(revision)
    expect(state.libraryRescanBusy.value).toBe(false)
  })

  it('lỗi `library.relocate_mismatch` được giữ để hiển thị, mục mồ côi còn nguyên', async () => {
    const state = await import('../../src/modes/libraryRescan')
    mockInvoke.mockResolvedValueOnce(CHANGED_REPORT)
    await state.rescanLibraryFolder()
    const rustError = {
      code: 'library.relocate_mismatch',
      message_key: 'err.library.relocate_mismatch',
      params: { name: 'Lost' },
      retryable: false,
    }
    mockInvoke.mockRejectedValueOnce(rustError)

    await state.relocateCurrentLibraryOrphan()

    expect(state.libraryRescanError.value).toEqual(rustError)
    expect(state.libraryOrphans.value).toHaveLength(1)
  })

  it('không có mục mồ côi nào thì không gọi lệnh', async () => {
    const state = await import('../../src/modes/libraryRescan')
    await state.relocateCurrentLibraryOrphan()

    expect(mockInvoke).not.toHaveBeenCalled()
  })
})

describe('modes/libraryRescan.ts::rebuildLibraryIndexFolder', () => {
  it('gọi `library_rebuild_index` không tham số và áp báo cáo kèm mốc mới', async () => {
    mockInvoke.mockResolvedValueOnce({ ...BASE_REPORT, rebuilt_at: '2026-10-13T09:30:00Z', skipped: 1 })
    const state = await import('../../src/modes/libraryRescan')

    await state.rebuildLibraryIndexFolder()

    expect(mockInvoke).toHaveBeenCalledWith('library_rebuild_index', undefined)
    expect(state.libraryRebuiltAt.value).toBe('2026-10-13T09:30:00Z')
    expect(state.libraryRescanResultHasLoadedState.value).toBe(true)
  })

  it('lỗi từ Rust được giữ và state không đổi', async () => {
    const rustError = { code: 'store.io', message_key: 'err.store.io', params: {}, retryable: true }
    mockInvoke.mockRejectedValueOnce(rustError)
    const state = await import('../../src/modes/libraryRescan')

    await state.rebuildLibraryIndexFolder()

    expect(state.libraryRescanError.value).toEqual(rustError)
    expect(state.libraryRescanResultHasLoadedState.value).toBe(false)
  })
})

describe('modes/libraryRescan.ts::openCurrentLibraryWorkFolder', () => {
  it('chỉ mục chưa tải ⇒ không gọi lệnh', async () => {
    const state = await import('../../src/modes/libraryRescan')
    await state.openCurrentLibraryWorkFolder()

    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('đã tải ⇒ gọi `library_open_work_folder` với `workId` của Tác phẩm đang chọn', async () => {
    const works = await import('../../src/modes/libraryWorks')
    mockInvoke.mockResolvedValueOnce({ total: 1, matched: 1, works: [WORK_ROW], genres: [], source_langs: [] })
    await works.loadWorks()
    mockInvoke.mockResolvedValueOnce(null)
    const state = await import('../../src/modes/libraryRescan')

    await state.openCurrentLibraryWorkFolder()

    expect(mockInvoke).toHaveBeenLastCalledWith('library_open_work_folder', { workId: 'id-open' })
    expect(state.libraryRescanError.value).toBeNull()
  })

  it('lỗi `library.open_folder_failed` đi vào `libraryRescanError`', async () => {
    const works = await import('../../src/modes/libraryWorks')
    mockInvoke.mockResolvedValueOnce({ total: 1, matched: 1, works: [WORK_ROW], genres: [], source_langs: [] })
    await works.loadWorks()
    const rustError = {
      code: 'library.open_folder_failed',
      message_key: 'err.library.open_folder_failed',
      params: { name: 'Open' },
      retryable: false,
    }
    mockInvoke.mockRejectedValueOnce(rustError)
    const state = await import('../../src/modes/libraryRescan')

    await state.openCurrentLibraryWorkFolder()

    expect(state.libraryRescanError.value).toEqual(rustError)
  })
})
