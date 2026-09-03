/**
 * `modes/librarySearch.ts` + `config/library.ts::searchLibrary` — Story 5.9, FR8. Story 5.10
 * (FR9) thêm hai chế độ dấu.
 *
 * ⚠️ **PHẠM VI** — `happy-dom` canh HÀNH VI của module thuần, không hình học/engine thật
 * (`tests/AGENTS.md`). Bốn mệnh đề dưới đây là những chỗ *"trông như đúng nhưng có thể sai
 * im lặng"* mà story này đặc biệt lo: năm ca rỗng phải phân biệt được (không suy diễn từ một
 * con số một mình), một truy vấn rỗng không được phát IPC, một lượt gõ NHANH không để kết quả
 * CŨ ghi đè kết quả MỚI, và một lượt mở kết quả không bao giờ phát lệnh mở Chương trước khi
 * lượt mở Tác phẩm đã THẬT SỰ xong (không chỉ `await` xong). Story 5.10 thêm ba mệnh đề: đổi
 * chế độ khi ô tìm rỗng không phát IPC, đổi chế độ khi có truy vấn CÓ chạy lại đúng `mode`, và
 * hàng `lenient` mang nhãn phân biệt được trên DOM thật.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
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
  match_kind: 'exact' as const,
}

/** **THÊM Story 5.10.** Cùng khuôn `SEARCH_HIT_A`, khác đúng `match_kind` — hit chỉ khớp qua
 * chỉ mục khoan dung `_nd`. */
const SEARCH_HIT_LENIENT = {
  work_id: 'id-b',
  work_name: 'Beta',
  chapter_id: 8,
  chapter_ord: 1,
  chapter_title: 'Chuong Mot',
  segment_id: 55,
  field: 'target' as const,
  snippet: '‹khoáng› sản',
  match_kind: 'lenient' as const,
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
// `librarySearchStatus()` — hàm THUẦN, TÁM trạng thái phân biệt được (§I/O Matrix, Story 5.9 +
// 5.10 — xem khối 🔵 đầu `librarySearch.ts` cho lý do "tám" chứ không "bảy").
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/librarySearch.ts::librarySearchStatus — hàm thuần', () => {
  it('chưa nạp lần nào, không bận ⇒ not_typed', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(false, false, 0, 0, false, false)).toBe('not_typed')
  })

  it('chưa nạp lần nào, đang bận ⇒ searching', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(false, true, 0, 0, false, false)).toBe('searching')
  })

  it('đã nạp, indexedSegments = 0 ⇒ index_empty (bất kể hits/shortQuery/widened)', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 0, 0, true, false)).toBe('index_empty')
    expect(librarySearchStatus(true, false, 0, 5, false, false)).toBe('index_empty')
    expect(librarySearchStatus(true, false, 0, 0, false, true)).toBe('index_empty')
  })

  it('đã nạp, indexedSegments > 0, hits rỗng, short_query, KHÔNG widened ⇒ short_query', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 10, 0, true, false)).toBe('short_query')
  })

  it('đã nạp, indexedSegments > 0, hits rỗng, KHÔNG short_query, KHÔNG widened ⇒ no_match', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 10, 0, false, false)).toBe('no_match')
  })

  it('🔴 hits rỗng VÀ widened ⇒ no_match_widened — THẮNG short_query dù truy vấn cũng ngắn', async () => {
    // Story 5.10: một lượt đã TỰ NỚI (đã thử cả hai chế độ) là thông tin đầy đủ hơn "quá
    // ngắn" — kể cả khi cả hai điều kiện cùng đúng, `no_match_widened` phải thắng.
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 10, 0, false, true)).toBe('no_match_widened')
    expect(librarySearchStatus(true, false, 10, 0, true, true)).toBe('no_match_widened')
  })

  it('🔴 short_query = true NHƯNG có hit thật (nửa bản dịch) ⇒ result, KHÔNG short_query', async () => {
    // Đúng bảng đo của §Design Notes: "ma" (2 ký tự) vẫn khớp TRỌN TỪ ở nửa unicode61.
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 10, 1, true, false)).toBe('result')
  })

  it('đã nạp, có hit, KHÔNG widened ⇒ result', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 10, 3, false, false)).toBe('result')
  })

  it('🔴 đã nạp, có hit, widened ⇒ result_widened — tách khỏi result thường', async () => {
    const { librarySearchStatus } = await import('../../src/modes/librarySearch')
    expect(librarySearchStatus(true, false, 10, 3, false, true)).toBe('result_widened')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// `librarySearchCoverageGap()` — hàm THUẦN, retro Epic 5 AI-3. Bề mặt ĐỘC LẬP với
// `librarySearchStatus`: thủng · không thủng · `worksTotal === 0`.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/librarySearch.ts::librarySearchCoverageGap — hàm thuần', () => {
  it('thủng: worksWithText < worksTotal ⇒ { missing, total }', async () => {
    const { librarySearchCoverageGap } = await import('../../src/modes/librarySearch')
    expect(librarySearchCoverageGap(47, 12)).toEqual({ missing: 35, total: 47 })
  })

  it('không thủng: worksWithText === worksTotal ⇒ null (không một chuỗi rỗng)', async () => {
    const { librarySearchCoverageGap } = await import('../../src/modes/librarySearch')
    expect(librarySearchCoverageGap(47, 47)).toBeNull()
  })

  it('🔴 worksTotal === 0 ⇒ null — chỉ mục rỗng hẳn KHÔNG phải một chỗ thủng', async () => {
    // §I/O Matrix "Chỉ mục rỗng hẳn": `library_segment` = 0 hàng, `library_work` = 0 ⇒
    // `index_empty` đã nói đủ; dòng độ phủ không được ĐÈ lên nó bằng "0/0 Tác phẩm chưa vào
    // chỉ mục" — một câu đúng hình dạng, sai sự thật.
    const { librarySearchCoverageGap } = await import('../../src/modes/librarySearch')
    expect(librarySearchCoverageGap(0, 0)).toBeNull()
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
        return Promise.resolve({ hits: [SEARCH_HIT_A], total: 1, indexed_segments: 5, short_query: false, truncated: false, mode: 'exact', effective_mode: 'exact', widened: false, works_total: 5, works_with_text: 5 })
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
        mode: 'exact',
        effective_mode: 'exact',
        widened: false,
        works_total: 5,
        works_with_text: 5,
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
    resolveFirst({ hits: [], total: 0, indexed_segments: 5, short_query: false, truncated: false, mode: 'exact', effective_mode: 'exact', widened: false, works_total: 5, works_with_text: 5 })
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
// STORY 5.10 — "Hai chế độ dấu" (FR9): `setLibrarySearchModeExact`/`setLibrarySearchModeLenient`.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/librarySearch.ts::setLibrarySearchModeExact/Lenient', () => {
  it('🔴 ô tìm RỖNG ⇒ đổi cờ chế độ nhưng KHÔNG phát lượt IPC nào (§I/O Matrix)', async () => {
    const state = await import('../../src/modes/librarySearch')
    expect(state.librarySearchQuery.value).toBe('')
    expect(state.librarySearchMode.value).toBe('exact')

    state.setLibrarySearchModeLenient()
    // Đổi cờ chạy ĐỒNG BỘ; `runLibrarySearch()` (nếu có gọi) mới là async — chờ một macrotask
    // để chắc chắn không có lượt IPC nào lặng lẽ bay sau đó.
    await Promise.resolve()

    expect(state.librarySearchMode.value).toBe('lenient')
    expect(mockInvoke).not.toHaveBeenCalled()
  })

  it('🔴 đã có truy vấn ⇒ đổi chế độ CHẠY LẠI lượt tìm, và `mode` gửi đi đúng giá trị mới', async () => {
    const queriesAndModes: { query?: string; mode?: string }[] = []
    mockInvoke.mockImplementation((cmd: string, args?: { query?: string; mode?: string }) => {
      if (cmd !== 'library_search') return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
      queriesAndModes.push({ query: args?.query, mode: args?.mode })
      return Promise.resolve({
        hits: [SEARCH_HIT_LENIENT],
        total: 1,
        indexed_segments: 5,
        short_query: false,
        truncated: false,
        mode: 'lenient',
        effective_mode: 'lenient',
        widened: false,
        works_total: 5,
        works_with_text: 5,
      })
    })

    const state = await import('../../src/modes/librarySearch')
    state.librarySearchQuery.value = 'khoang'
    await state.runLibrarySearch()
    expect(queriesAndModes).toEqual([{ query: 'khoang', mode: 'exact' }])

    state.setLibrarySearchModeLenient()
    // `setLibrarySearchModeLenient` gọi lại `runLibrarySearch()` fire-and-forget (khuôn
    // `rescanLibraryFolder`) -- chờ một lượt microtask để lời gọi IPC kịp phát.
    await Promise.resolve()
    await Promise.resolve()

    expect(queriesAndModes).toEqual([
      { query: 'khoang', mode: 'exact' },
      { query: 'khoang', mode: 'lenient' },
    ])
    expect(state.librarySearchMode.value).toBe('lenient')
  })

  // ─────────────────────────────────────────────────────────────────────────────
  // 🔴 THÊM (vòng rà bốn lớp, mục 3) — `setLibrarySearchModeExact` chưa từng được GỌI ở tệp
  // này trước lượt vá này (`grep -rn setLibrarySearchModeExact tests/` cho hai dòng, cả hai
  // là VĂN BẢN: một chú thích và một tiêu đề `describe`). Hai hàm chỉ khác nhau đúng literal
  // truyền vào `setLibrarySearchMode(next)` — một lỗi chép-dán (`setLibrarySearchModeExact`
  // gọi `setLibrarySearchMode('lenient')`) sẽ làm nút "Phân biệt dấu" — đường về chế độ MẶC
  // ĐỊNH mà AD-27 bắt buộc — hỏng im lặng mà không ca nào đỏ. Ca đối xứng dưới đây bắt đầu từ
  // `mode = 'lenient'` rồi gọi `setLibrarySearchModeExact()`.
  // ─────────────────────────────────────────────────────────────────────────────
  it('🔴 setLibrarySearchModeExact() lật cờ về "exact" VÀ gửi đúng `mode: "exact"` trên dây', async () => {
    const queriesAndModes: { query?: string; mode?: string }[] = []
    mockInvoke.mockImplementation((cmd: string, args?: { query?: string; mode?: string }) => {
      if (cmd !== 'library_search') return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
      queriesAndModes.push({ query: args?.query, mode: args?.mode })
      return Promise.resolve({
        hits: [SEARCH_HIT_A],
        total: 1,
        indexed_segments: 5,
        short_query: false,
        truncated: false,
        mode: 'exact',
        effective_mode: 'exact',
        widened: false,
        works_total: 5,
        works_with_text: 5,
      })
    })

    const state = await import('../../src/modes/librarySearch')
    state.librarySearchQuery.value = 'khoang'
    await state.runLibrarySearch()
    expect(queriesAndModes).toEqual([{ query: 'khoang', mode: 'exact' }])

    // Chuyển sang lenient trước — cùng khuôn ca ngay trên.
    state.setLibrarySearchModeLenient()
    await Promise.resolve()
    await Promise.resolve()
    expect(state.librarySearchMode.value).toBe('lenient')
    expect(queriesAndModes.at(-1)).toEqual({ query: 'khoang', mode: 'lenient' })

    // 🔴 Mệnh đề trung tâm: gọi ĐÚNG `setLibrarySearchModeExact`, không phải `Lenient`.
    state.setLibrarySearchModeExact()
    await Promise.resolve()
    await Promise.resolve()

    expect(state.librarySearchMode.value).toBe('exact')
    expect(queriesAndModes.at(-1)).toEqual({ query: 'khoang', mode: 'exact' })
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// STORY 5.10 — hàng `lenient` mang một nhãn phân biệt được trên DOM THẬT, hàng `exact` thì
// không (§Always: "hai loại kết quả phân biệt được trên màn hình").
// ═════════════════════════════════════════════════════════════════════════════════

const WORK_NONE_OPEN_ERROR_FOR_MOUNT = {
  code: 'work.none_open',
  message_key: 'err.work.none_open',
  params: {},
  retryable: false,
}

/** 🔵 SỬA (vòng rà bốn lớp, mục 4) — nhận thêm `overrides` để dựng được CẢ TÁM nhánh của
 * `role="status"`, không chỉ ca "có hit, lenient, không cắt" ban đầu. Mặc định GIỮ NGUYÊN
 * hành vi cũ (không đổi ca đã có). */
function mockInvokeForSearchMount(
  hits: SearchHit[],
  overrides: Partial<{
    total: number
    indexed_segments: number
    short_query: boolean
    truncated: boolean
    mode: string
    effective_mode: string
    widened: boolean
    works_total: number
    works_with_text: number
  }> = {},
): void {
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === 'library_list_works') {
      return Promise.resolve({ total: 0, matched: 0, works: [], genres: [], source_langs: [] })
    }
    if (cmd === 'read_work_lifecycle') return Promise.reject(WORK_NONE_OPEN_ERROR_FOR_MOUNT)
    if (cmd === 'list_chapters') return Promise.reject(WORK_NONE_OPEN_ERROR_FOR_MOUNT)
    if (cmd === 'library_search') {
      return Promise.resolve({
        hits,
        total: overrides.total ?? hits.length,
        indexed_segments: overrides.indexed_segments ?? 5,
        short_query: overrides.short_query ?? false,
        truncated: overrides.truncated ?? false,
        mode: overrides.mode ?? 'lenient',
        effective_mode: overrides.effective_mode ?? 'lenient',
        widened: overrides.widened ?? false,
        works_total: overrides.works_total ?? 5,
        works_with_text: overrides.works_with_text ?? 5,
      })
    }
    return Promise.reject(new Error(`invoke gia khong biet lenh: ${cmd}`))
  })
}

describe('modes/LibraryMode.vue — nhãn khoan dung dấu trên DOM thật (mount thật)', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    // 🔴 BẮT BUỘC unmount ở MỌI đường ra — cùng lý do đã ghi ở `libraryChapters.test.ts`:
    // `declareFocus('mode.library', ..)` ném khi owner TRÙNG nếu lượt trước không nhả.
    wrapper?.unmount()
    wrapper = null
  })

  it('hàng `lenient` mang nhãn `data-library-search-hit-lenient`, hàng `exact` thì KHÔNG', async () => {
    mockInvokeForSearchMount([SEARCH_HIT_A, SEARCH_HIT_LENIENT])

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/librarySearch')
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()

    state.librarySearchQuery.value = 'khoang'
    await state.runLibrarySearch()
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('[data-library-search-hit]')
    expect(rows).toHaveLength(2)
    expect(rows[0]?.find('[data-library-search-hit-lenient]').exists()).toBe(false)
    expect(rows[1]?.find('[data-library-search-hit-lenient]').exists()).toBe(true)
  })

  // ─────────────────────────────────────────────────────────────────────────────
  // 🔴 THÊM (retro Epic 5, AI-3 — 2026-09-03) — dòng độ phủ ĐỘC LẬP với khối `role="status"`
  // tám nhánh: nó phải hiện được CẢ KHI có kết quả (§Always). §I/O Matrix "Tìm trên chỉ mục
  // thủng": `works_total = 47`, `works_with_text = 12` ⇒ màn hình hiện CẢ "1 kết quả" LẪN
  // "35/47 Tác phẩm chưa vào chỉ mục".
  // ─────────────────────────────────────────────────────────────────────────────

  it('🔴 chỉ mục THỦNG ⇒ dòng độ phủ hiện ĐỒNG THỜI với số kết quả, không thay thế nó', async () => {
    mockInvokeForSearchMount([SEARCH_HIT_A], { works_total: 47, works_with_text: 12 })

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/librarySearch')
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()

    state.librarySearchQuery.value = 'khoang'
    await state.runLibrarySearch()
    await wrapper.vm.$nextTick()

    const status = wrapper.find('[data-library-search-status]')
    expect(status.text()).toContain('1 kết quả')

    const coverageGap = wrapper.find('[data-library-search-coverage-gap]')
    expect(coverageGap.exists()).toBe(true)
    expect(coverageGap.text()).toContain('35/47')
  })

  it('🔴 chỉ mục ĐẦY ĐỦ (worksWithText === worksTotal) ⇒ dòng độ phủ KHÔNG render', async () => {
    mockInvokeForSearchMount([SEARCH_HIT_A], { works_total: 47, works_with_text: 47 })

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/librarySearch')
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()

    state.librarySearchQuery.value = 'khoang'
    await state.runLibrarySearch()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-library-search-coverage-gap]').exists()).toBe(false)
  })

  // ─────────────────────────────────────────────────────────────────────────────
  // 🔴 THÊM (vòng rà bốn lớp, mục 4) — tám nhánh của `role="status"` được TÍNH đúng ở
  // `librarySearchStatus` (hàm thuần, đã có lưới) nhưng chưa ca nào đọc CÂU CHỮ THẬT hiện
  // ra trên DOM cho hai nhánh widened. Một lỗi nối dây giữa khoá trạng thái và bảng ternary
  // của `LibraryMode.vue` (ví dụ đặt nhánh sai thứ tự) sẽ hiện nhầm câu mà không ca nào đỏ.
  // ─────────────────────────────────────────────────────────────────────────────

  it('🔴 hits rỗng + widened ⇒ DOM hiện đúng câu "đã thử cả chế độ khoan dung dấu"', async () => {
    mockInvokeForSearchMount([], { widened: true, mode: 'exact', effective_mode: 'lenient' })

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/librarySearch')
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()

    state.librarySearchQuery.value = 'tu vo nghia'
    await state.runLibrarySearch()
    await wrapper.vm.$nextTick()

    const status = wrapper.find('[data-library-search-status]')
    expect(status.text()).toContain('đã thử cả chế độ khoan dung dấu')
  })

  it('🔴 có hit + widened, KHÔNG cắt ⇒ DOM hiện câu "đã tự chuyển sang khoan dung dấu", không "còn nữa"', async () => {
    mockInvokeForSearchMount([SEARCH_HIT_LENIENT], { widened: true, mode: 'exact', effective_mode: 'lenient' })

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/librarySearch')
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()

    state.librarySearchQuery.value = 'khoang'
    await state.runLibrarySearch()
    await wrapper.vm.$nextTick()

    const status = wrapper.find('[data-library-search-status]')
    expect(status.text()).toContain('đã tự chuyển sang khoan dung dấu')
    expect(status.text()).not.toContain('còn nữa')
  })

  // ─────────────────────────────────────────────────────────────────────────────
  // 🔴 THÊM (vòng rà bốn lớp, mục 1) — `result_widened` PHẢI rẽ theo `truncated` NGAY BÊN
  // TRONG nhánh của nó: một lượt vừa tự nới VỪA bị trần `limit` cắt phải nói CẢ HAI, không
  // chỉ "đã tự chuyển sang khoan dung dấu" (đúng lớp lỗi mà `SearchReport::truncated` của
  // Story 5.9 tồn tại để chặn — "không trần nào được cắt trong im lặng").
  // ─────────────────────────────────────────────────────────────────────────────

  it('🔴 có hit + widened + truncated ⇒ DOM nói CẢ HAI: đã tự nới VÀ còn nữa', async () => {
    mockInvokeForSearchMount([SEARCH_HIT_LENIENT], {
      widened: true,
      truncated: true,
      mode: 'exact',
      effective_mode: 'lenient',
    })

    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const state = await import('../../src/modes/librarySearch')
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()

    state.librarySearchQuery.value = 'khoang'
    await state.runLibrarySearch()
    await wrapper.vm.$nextTick()

    const status = wrapper.find('[data-library-search-status]')
    expect(status.text()).toContain('khoan dung dấu')
    expect(status.text()).toContain('còn nữa')
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
        return Promise.resolve({ hits: [SEARCH_HIT_A], total: 1, indexed_segments: 5, short_query: false, truncated: false, mode: 'exact', effective_mode: 'exact', widened: false, works_total: 5, works_with_text: 5 })
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
        return Promise.resolve({ hits: [SEARCH_HIT_A], total: 1, indexed_segments: 5, short_query: false, truncated: false, mode: 'exact', effective_mode: 'exact', widened: false, works_total: 5, works_with_text: 5 })
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
        return Promise.resolve({ hits: [SEARCH_HIT_A], total: 1, indexed_segments: 5, short_query: false, truncated: false, mode: 'exact', effective_mode: 'exact', widened: false, works_total: 5, works_with_text: 5 })
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
        return Promise.resolve({ hits: [hit], total: 1, indexed_segments: 5, short_query: false, truncated: false, mode: 'exact', effective_mode: 'exact', widened: false, works_total: 5, works_with_text: 5 })
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
        // 🔴 SỬA (vòng rà bốn lớp, mục 10) — `mode: 'lenient'` cộng `widened: true` trong CÙNG
        // một report là một tổ hợp mà `Indexer::search` THẬT không bao giờ phát ra (bất biến
        // `widened == (mode == exact && effective_mode == lenient)`, khoá ở tầng Rust) — ở đây
        // nó được chọn CÓ CHỦ Ý, chỉ để đẩy CẢ BA ô nhớ mới (`mode`/`effectiveMode`/`widened`)
        // ra khỏi giá trị MẶC ĐỊNH trong một lượt, cho `resetLibrarySearch` có gì thật để vứt.
        return Promise.resolve({
          hits: [SEARCH_HIT_A],
          total: 1,
          indexed_segments: 5,
          short_query: false,
          truncated: false,
          mode: 'lenient',
          effective_mode: 'lenient',
          widened: true,
          works_total: 5,
          works_with_text: 5,
        })
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })

    const state = await import('../../src/modes/librarySearch')
    state.setLibrarySearchModeLenient()
    state.librarySearchQuery.value = 'má của tôi'
    await state.runLibrarySearch()
    expect(state.librarySearchHits.value).toHaveLength(1)
    // Tiền đề của ca: cả ba ô nhớ mới THẬT SỰ đã rời khỏi mặc định trước khi reset.
    expect(state.librarySearchMode.value).toBe('lenient')
    expect(state.librarySearchEffectiveMode.value).toBe('lenient')
    expect(state.librarySearchWidened.value).toBe(true)
    // **THÊM (retro Epic 5, AI-3)** — cùng tiền đề: đã rời khỏi 0 trước khi reset.
    expect(state.librarySearchWorksTotal.value).toBe(5)
    expect(state.librarySearchWorksWithText.value).toBe(5)

    state.resetLibrarySearch()

    expect(state.librarySearchQuery.value).toBe('')
    expect(state.librarySearchHits.value).toEqual([])
    expect(state.librarySearchHasLoaded.value).toBe(false)
    expect(state.librarySearchBusy.value).toBe(false)
    expect(state.librarySearchError.value).toBeNull()
    expect(state.librarySearchCursor.value).toBe(0)
    // 🔴 THÊM (vòng rà bốn lớp, mục 10) — đúng ba ô mà `check:panel-refs` là lý do chúng có
    // mặt trong `resetLibrarySearch`; khoan dung KHÔNG BAO GIỜ là mặc định (AD-27 · AC4).
    expect(state.librarySearchMode.value).toBe('exact')
    expect(state.librarySearchEffectiveMode.value).toBe('exact')
    expect(state.librarySearchWidened.value).toBe(false)
    // **THÊM (retro Epic 5, AI-3)** — độ phủ cấp Tác phẩm cũng phải về 0, không giữ số của
    // lượt tìm trước.
    expect(state.librarySearchWorksTotal.value).toBe(0)
    expect(state.librarySearchWorksWithText.value).toBe(0)
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
    expect(librarySearchStatus(true, true, 10, 3, false, false)).toBe('searching')
    expect(librarySearchStatus(true, true, 10, 0, false, false)).toBe('searching')
  })
})
