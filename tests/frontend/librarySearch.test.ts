/**
 * `modes/librarySearch.ts` + `config/library.ts::searchLibrary` — Story 5.9, FR8.
 *
 * ⚠️ **PHẠM VI** — `happy-dom` canh HÀNH VI của module thuần, không hình học/engine thật
 * (`tests/AGENTS.md`). Bốn mệnh đề dưới đây là những chỗ *"trông như đúng nhưng có thể sai
 * im lặng"* mà story này đặc biệt lo: năm ca rỗng phải phân biệt được (không suy diễn từ một
 * con số một mình), một truy vấn rỗng không được phát IPC, một lượt gõ NHANH không để kết quả
 * CŨ ghi đè kết quả MỚI, và một lượt mở kết quả không bao giờ phát lệnh mở Chương trước khi
 * lượt mở Tác phẩm đã THẬT SỰ xong (không chỉ `await` xong).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { SearchHit } from '../../src/config/library'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

/** Kết quả mà lượt `flushEditorBeforeDiscreteWrite()` kế tiếp trả về — cùng khuôn
 * `libraryChapters.test.ts`. */
const ketQuaFlush: { value: 'clean' | 'failed' | 'still-dirty' } = { value: 'clean' }
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return { ...actual, flushEditorBeforeDiscreteWrite: async () => ketQuaFlush.value }
})

const SEARCH_HIT_A = {
  work_id: 'id-a',
  work_name: 'Alpha',
  chapter_id: 7,
  chapter_ord: 1,
  chapter_title: 'Chuong Mot',
  segment_id: 42,
  field: 'target' as const,
  snippet: '‹má› của tôi',
}

const OPENED_WORK_A = {
  meta: {
    meta_schema_version: 1,
    work_id: 'id-a',
    name: 'Alpha',
    source_lang: 'zh',
    genre: '',
    created_at: '2026-08-01T00:00:00.000Z',
    updated_at: '2026-08-01T00:00:00.000Z',
    chapter_count: 1,
  },
  folder: '/tmp/Alpha.atproj',
  chapter_id: 7,
}

const OPEN_CHAPTER_A = {
  chapter_id: 7,
  source_text: 'nguon',
  source_lang: 'zh',
}

beforeEach(async () => {
  mockInvoke.mockReset()
  ketQuaFlush.value = 'clean'
  const search = await import('../../src/modes/librarySearch')
  search.resetLibrarySearch()
  const chapters = await import('../../src/modes/libraryChapters')
  chapters.resetLibraryChapters()
  const works = await import('../../src/modes/libraryWorks')
  works.resetLibraryWorks()
  const editor = await import('../../src/panels/editorPanelState')
  editor.resetEditorPanel()
})

afterEach(() => {
  vi.restoreAllMocks()
})

// ═════════════════════════════════════════════════════════════════════════════════
// `librarySearchStatus()` — hàm THUẦN, năm ca rỗng + ca có kết quả (§I/O Matrix).
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/librarySearch.ts::librarySearchStatus — hàm thuần', () => {
  it('chưa nạp lần nào, không bận ⇒ not_typed', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(false, false, 0, 0, false)).toBe('not_typed')
  })

  it('chưa nạp lần nào, đang bận ⇒ searching', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(false, true, 0, 0, false)).toBe('searching')
  })

  it('đã nạp, indexedSegments = 0 ⇒ index_empty (bất kể hits/shortQuery)', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 0, 0, true)).toBe('index_empty')
    expect(librarySearchStatus(true, false, 0, 5, false)).toBe('index_empty')
  })

  it('đã nạp, indexedSegments > 0, hits rỗng, short_query ⇒ short_query', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 10, 0, true)).toBe('short_query')
  })

  it('đã nạp, indexedSegments > 0, hits rỗng, KHÔNG short_query ⇒ no_match', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 10, 0, false)).toBe('no_match')
  })

  it('🔴 short_query = true NHƯNG có hit thật (nửa bản dịch) ⇒ result, KHÔNG short_query', async () => {
    // Đúng bảng đo của §Design Notes: "ma" (2 ký tự) vẫn khớp TRỌN TỪ ở nửa unicode61.
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 10, 1, true)).toBe('result')
  })

  it('đã nạp, có hit ⇒ result', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 10, 3, false)).toBe('result')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// `runLibrarySearch()` — truy vấn rỗng KHÔNG phát IPC (§I/O Matrix).
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/librarySearch.ts::runLibrarySearch — truy vấn rỗng', () => {
  it('🔴 ô tìm rỗng (hoặc chỉ khoảng trắng) ⇒ 0 lượt `invoke`, trạng thái về "chưa gõ gì"', async () => {
    const state = await import('../../src/modes/librarySearch')
    state.librarySearchQuery.value = '   '
    await state.runLibrarySearch()

    expect(mockInvoke).not.toHaveBeenCalled()
    expect(state.librarySearchHasLoaded.value).toBe(false)
    expect(state.librarySearchBusy.value).toBe(false)
    expect(state.librarySearchHits.value).toEqual([])
  })

  it('truy vấn có chữ ⇒ gọi ĐÚNG một lượt `library_search`', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_search') {
        return Promise.resolve({ hits: [SEARCH_HIT_A], total: 1, indexed_segments: 5, short_query: false, truncated: false })
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })

    const state = await import('../../src/modes/librarySearch')
    state.librarySearchQuery.value = 'má của tôi'
    await state.runLibrarySearch()

    expect(mockInvoke.mock.calls.filter((c) => c[0] === 'library_search')).toHaveLength(1)
    expect(state.librarySearchHasLoaded.value).toBe(true)
    expect(state.librarySearchHits.value).toEqual([SEARCH_HIT_A])
    expect(state.librarySearchTotal.value).toBe(1)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Chống đua — một lượt gõ NHANH không để kết quả CŨ ghi đè kết quả MỚI.
//
// 🔵 **SỬA 2026-08-29, và mệnh đề cũ ở đây đã HẾT ĐÚNG.** Tiêu đề cũ ghi cơ chế là
// `searchSequence`. Đo trên chính mã sản phẩm: `runLibrarySearch` chặn ở `busy` (`:142-145`)
// TRƯỚC khi bump `sequence` (`:149`), nên hai lượt **không bao giờ** cùng bay và
// `mySequence !== sequence` (`:152`) không với tới được từ đường này. Cơ chế THẬT là cặp
// `busy` + `reloadPending`: lượt hai bị GHI NHỚ, rồi chạy lại bằng truy vấn MỚI sau khi lượt
// đầu hạ cánh. Ca dưới đây đo đúng cơ chế đó — bản đầu của nó khẳng định `callCount === 2` và
// **đỏ**, vì lượt hai chưa hề phát một lời gọi IPC nào.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/librarySearch.ts::runLibrarySearch — chống đua giữa hai lượt gõ', () => {
  it('🔴 lượt ĐẦU trả về SAU lượt HAI ⇒ kết quả CŨ không được ghi đè kết quả MỚI', async () => {
    // ⚠️ Bộ giải quyết lấy ra NGOÀI `mockImplementation`, không phải một `let … | null` gán
    // bên trong callback: TypeScript không theo dõi lượt gán nằm trong một hàm lồng, nên biến
    // đó bị thu hẹp về `null` ở chỗ gọi và `vue-tsc` đỏ với `Type 'never' has no call
    // signatures`. Khuôn "tạo promise trước, giữ resolver" nói đúng sự thật cho trình biên
    // dịch mà không đổi một dòng nào của mã sản phẩm (`tests/AGENTS.md::Known pitfalls`).
    let resolveFirst!: (value: unknown) => void
    const firstPending = new Promise<unknown>((resolve) => {
      resolveFirst = resolve
    })
    const queriesSeen: string[] = []
    let callCount = 0
    mockInvoke.mockImplementation((cmd: string, args?: { query?: string }) => {
      if (cmd !== 'library_search') return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
      callCount += 1
      queriesSeen.push(args?.query ?? '')
      if (callCount === 1) {
        // Lượt ĐẦU treo lại -- mô phỏng một lượt IPC chậm, hạ cánh SAU khi người dùng đã gõ tiếp.
        return firstPending
      }
      // Lượt GHI NHỚ chạy lại bằng truy vấn MỚI -- trả về ngay.
      return Promise.resolve({
        hits: [SEARCH_HIT_A],
        total: 1,
        indexed_segments: 5,
        short_query: false,
        truncated: false,
      })
    })

    const state = await import('../../src/modes/librarySearch')

    state.librarySearchQuery.value = 'ma'
    const firstRun = state.runLibrarySearch()
    // Đợi lượt đầu THỰC SỰ bắt đầu bay (đã gọi `invoke`) trước khi gõ tiếp.
    await Promise.resolve()
    expect(callCount).toBe(1)
    expect(state.librarySearchBusy.value).toBe(true)

    // Người dùng gõ tiếp trong lúc lượt đầu còn bay: lượt hai bị GHI NHỚ, KHÔNG phát IPC.
    state.librarySearchQuery.value = 'ma khac'
    await state.runLibrarySearch()
    expect(callCount).toBe(1)

    // BÂY GIỜ mới cho lượt đầu (CŨ, truy vấn `ma`) hạ cánh với kết quả RỖNG.
    resolveFirst({ hits: [], total: 0, indexed_segments: 5, short_query: false, truncated: false })
    await firstRun

    // 🔴 Lượt ghi nhớ phải đã chạy, và phải chạy bằng truy vấn MỚI — không phải truy vấn cũ.
    expect(callCount).toBe(2)
    expect(queriesSeen).toEqual(['ma', 'ma khac'])

    // Và state cuối cùng mang kết quả của truy vấn MỚI, không phải cái rỗng của truy vấn cũ.
    expect(state.librarySearchHits.value).toEqual([SEARCH_HIT_A])
    expect(state.librarySearchTotal.value).toBe(1)
    expect(state.librarySearchBusy.value).toBe(false)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// `openCurrentLibrarySearchHit()` — thứ tự HAI lượt mở, và cửa chặn khi mở Tác phẩm trượt.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/librarySearch.ts::openCurrentLibrarySearchHit', () => {
  it('danh sách rỗng (không hit nào đang chọn) ⇒ no-op, 0 lượt invoke', async () => {
    const state = await import('../../src/modes/librarySearch')
    await state.openCurrentLibrarySearchHit()
    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('🔴 `open_work` THÀNH CÔNG ⇒ gọi ĐÚNG THỨ TỰ open_work RỒI open_chapter', async () => {
    const callOrder: string[] = []
    mockInvoke.mockImplementation((cmd: string) => {
      callOrder.push(cmd)
      if (cmd === 'library_search') {
        return Promise.resolve({ hits: [SEARCH_HIT_A], total: 1, indexed_segments: 5, short_query: false, truncated: false })
      }
      if (cmd === 'open_work') return Promise.resolve(OPENED_WORK_A)
      if (cmd === 'open_chapter') return Promise.resolve(OPEN_CHAPTER_A)
      if (cmd === 'list_chapters') return Promise.resolve([])
      if (cmd === 'read_open_chapter_segments') {
        return Promise.resolve({ chapter_id: 7, segments: [], caret_segment_id: null })
      }
      if (cmd === 'read_open_chapter') return Promise.resolve(OPEN_CHAPTER_A)
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })

    const state = await import('../../src/modes/librarySearch')
    state.librarySearchQuery.value = 'má của tôi'
    await state.runLibrarySearch()
    expect(state.currentLibrarySearchHit.value).toEqual(SEARCH_HIT_A)

    await state.openCurrentLibrarySearchHit()

    const openWorkIdx = callOrder.indexOf('open_work')
    const openChapterIdx = callOrder.indexOf('open_chapter')
    expect(openWorkIdx).toBeGreaterThanOrEqual(0)
    expect(openChapterIdx).toBeGreaterThan(openWorkIdx)
  })

  it('🔴 `open_work` TRƯỢT (lỗi IPC) ⇒ `open_chapter` KHÔNG BAO GIỜ được gọi', async () => {
    const OPEN_WORK_ERROR = {
      code: 'library.work_not_indexed',
      message_key: 'err.library.work_not_indexed',
      params: { work_id: 'id-a' },
      retryable: false,
    }
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_search') {
        return Promise.resolve({ hits: [SEARCH_HIT_A], total: 1, indexed_segments: 5, short_query: false, truncated: false })
      }
      // `invoke()` giả trượt bằng cách NÉM đúng hình dạng IpcError -- cùng khuôn adapter thật.
      if (cmd === 'open_work') return Promise.reject(OPEN_WORK_ERROR)
      if (cmd === 'open_chapter') return Promise.reject(new Error('KHONG DUOC GOI o ca nay'))
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })

    const state = await import('../../src/modes/librarySearch')
    state.librarySearchQuery.value = 'má của tôi'
    await state.runLibrarySearch()

    await state.openCurrentLibrarySearchHit()

    expect(mockInvoke.mock.calls.some((c) => c[0] === 'open_chapter')).toBe(false)
  })

  it('🔴 lượt mở Tác phẩm bị CHẶN vì flush chưa sạch ⇒ `open_work` VÀ `open_chapter` đều KHÔNG chạy', async () => {
    ketQuaFlush.value = 'still-dirty'
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_search') {
        return Promise.resolve({ hits: [SEARCH_HIT_A], total: 1, indexed_segments: 5, short_query: false, truncated: false })
      }
      return Promise.reject(new Error(`KHONG duoc goi: ${cmd}`))
    })

    const state = await import('../../src/modes/librarySearch')
    state.librarySearchQuery.value = 'má của tôi'
    await state.runLibrarySearch()

    await state.openCurrentLibrarySearchHit()

    expect(mockInvoke.mock.calls.some((c) => c[0] === 'open_work')).toBe(false)
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'open_chapter')).toBe(false)
  })

  // ───────────────────────────────────────────────────────────────────────────────
  // 🔴 Ba ca CON TRỎ của §I/O Matrix. Chúng đo `editorCaretPlacement` — tín hiệu DUY NHẤT
  // mà `GridPanel.vue` đọc để đặt caret và cuộn (`GridPanel.vue:1110-1129`), nên đó là bề
  // mặt đúng để khẳng định *"mở tại đúng chỗ khớp"* ở tầng vitest. Ba ca này thiếu ở lượt
  // dựng đầu: bộ ca cũ trả `segments: []` cho MỌI ca, tức nó đi qua nhánh "segment đã về
  // hưu" mà không khẳng định gì về con trỏ — một bộ xanh không chứng minh chỗ nối được canh.
  // ───────────────────────────────────────────────────────────────────────────────

  /** Dựng bộ mock cho một lượt mở đầy đủ; `segments` và `caretSegmentId` do ca gọi quyết. */
  function mockMoDayDu(
    hit: SearchHit,
    segments: { id: number }[],
    caretSegmentId: number | null,
  ): void {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_search') {
        return Promise.resolve({ hits: [hit], total: 1, indexed_segments: 5, short_query: false, truncated: false })
      }
      if (cmd === 'open_work') return Promise.resolve(OPENED_WORK_A)
      if (cmd === 'open_chapter') return Promise.resolve(OPEN_CHAPTER_A)
      if (cmd === 'list_chapters') return Promise.resolve([])
      if (cmd === 'read_open_chapter_segments') {
        return Promise.resolve({
          chapter_id: 7,
          segments: segments.map((s) => ({
            id: s.id,
            ord: s.id,
            source_text: `nguon ${s.id}`,
            target_text: '',
            status: 'draft',
            is_paragraph_end: false,
            is_target_paragraph_end: false,
            is_omitted: false,
          })),
          caret_segment_id: caretSegmentId,
        })
      }
      if (cmd === 'read_open_chapter') return Promise.resolve(OPEN_CHAPTER_A)
      if (cmd === 'save_chapter_position') return Promise.resolve(null)
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
  }

  it('🔴 hit có `segment_id` CÓ THẬT trong Chương ⇒ con trỏ đặt vào ĐÚNG câu khớp', async () => {
    mockMoDayDu(SEARCH_HIT_A, [{ id: 41 }, { id: 42 }, { id: 43 }], 41)

    const state = await import('../../src/modes/librarySearch')
    const editor = await import('../../src/panels/editorPanelState')
    state.librarySearchQuery.value = 'má của tôi'
    await state.runLibrarySearch()
    await state.openCurrentLibrarySearchHit()

    // 42 là câu khớp; 41 là câu mà RUST đề nghị. Câu khớp phải THẮNG.
    expect(editor.editorCaretPlacement.value).toBe(42)
  })

  it('🔴 hit cấp CHƯƠNG (`segment_id = null`) ⇒ con trỏ để RUST quyết, không ép vào câu nào', async () => {
    const hitChuong = { ...SEARCH_HIT_A, segment_id: null, field: 'source' as const }
    mockMoDayDu(hitChuong, [{ id: 41 }, { id: 42 }], 41)

    const state = await import('../../src/modes/librarySearch')
    const editor = await import('../../src/panels/editorPanelState')
    state.librarySearchQuery.value = '分久必合'
    await state.runLibrarySearch()
    await state.openCurrentLibrarySearchHit()

    expect(mockInvoke.mock.calls.some((c) => c[0] === 'open_chapter')).toBe(true)
    expect(editor.editorCaretPlacement.value).toBe(41)
  })

  it('🔴 hit ĐÃ CŨ (`segment_id` không còn trong Chương) ⇒ lượt mở VẪN xong, con trỏ giữ nguyên của Rust', async () => {
    // 42 là câu khớp mà chỉ mục còn nhớ; Chương thật nay chỉ còn 41 và 43 (42 đã về hưu).
    mockMoDayDu(SEARCH_HIT_A, [{ id: 41 }, { id: 43 }], 43)

    const state = await import('../../src/modes/librarySearch')
    const editor = await import('../../src/panels/editorPanelState')
    state.librarySearchQuery.value = 'má của tôi'
    await state.runLibrarySearch()
    await state.openCurrentLibrarySearchHit()

    // Lượt mở KHÔNG bị huỷ vì chuyện này...
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'open_chapter')).toBe(true)
    // ...và con trỏ KHÔNG bị ép vào một câu không tồn tại.
    expect(editor.editorCaretPlacement.value).toBe(43)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// `resetLibrarySearch()` — mọi ô nhớ trở về giá trị ban đầu (check:panel-refs).
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/librarySearch.ts::resetLibrarySearch', () => {
  it('vứt sạch state của một lượt tìm trước đó', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_search') {
        return Promise.resolve({ hits: [SEARCH_HIT_A], total: 1, indexed_segments: 5, short_query: false, truncated: false })
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })

    const state = await import('../../src/modes/librarySearch')
    state.librarySearchQuery.value = 'má của tôi'
    await state.runLibrarySearch()
    expect(state.librarySearchHits.value).toHaveLength(1)

    state.resetLibrarySearch()

    expect(state.librarySearchQuery.value).toBe('')
    expect(state.librarySearchHits.value).toEqual([])
    expect(state.librarySearchHasLoaded.value).toBe(false)
    expect(state.librarySearchBusy.value).toBe(false)
    expect(state.librarySearchError.value).toBeNull()
    expect(state.librarySearchCursor.value).toBe(0)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Type guard LÚC CHẠY — thêm ở lượt rà 2026-08-29.
// ═════════════════════════════════════════════════════════════════════════════════

describe('config/library.ts::searchLibrary — kiểm hình dạng LÚC CHẠY', () => {
  it('🔴 hàng thứ HAI sai hình dạng ⇒ CẢ báo cáo bị từ chối, không chỉ hàng đầu được kiểm', async () => {
    // Bản đầu của `isSearchHitArray` chỉ đọc `value[0]` rồi kết luận cho cả mảng, nên một hàng
    // hỏng ở vị trí thứ hai đi thẳng vào `v-for` của `LibraryMode.vue`.
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd !== 'library_search') return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
      return Promise.resolve({
        hits: [SEARCH_HIT_A, { ...SEARCH_HIT_A, chapter_id: 'bay-gio-la-mot-chuoi' }],
        total: 2,
        indexed_segments: 5,
        short_query: false,
        truncated: false,
      })
    })

    const { searchLibrary } = await import('../../src/config/library')
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    const result = await searchLibrary('má')

    expect(result.report).toBeNull()
    expect(result.error).not.toBeNull()
  })

  it('🔴 báo cáo THIẾU hẳn `truncated` bị từ chối — một trường mới trên dây phải có cửa canh', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd !== 'library_search') return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
      return Promise.resolve({ hits: [], total: 0, indexed_segments: 5, short_query: false })
    })

    const { searchLibrary } = await import('../../src/config/library')
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    const result = await searchLibrary('má')

    expect(result.report).toBeNull()
    expect(result.error).not.toBeNull()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Trạng thái "đang tìm" phải THẮNG kết quả CŨ — thêm ở lượt rà 2026-08-29.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/librarySearch.ts::librarySearchStatus — lượt tìm THỨ HAI', () => {
  it('🔴 đã có kết quả cũ mà một lượt tìm mới đang bay ⇒ `searching`, KHÔNG `result`', async () => {
    // Bản đầu chỉ đọc `busy` trong nhánh `!hasLoaded`, nên màn hình giữ nguyên danh sách của
    // truy vấn TRƯỚC và khai nó là kết quả — cho một truy vấn khác đang nằm trong ô tìm.
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, true, 10, 3, false)).toBe('searching')
    expect(librarySearchStatus(true, true, 10, 0, false)).toBe('searching')
  })
})
