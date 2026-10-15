/**
 * Tìm kiếm Library: phạm vi, bộ lọc, nhóm theo Tác phẩm, ms, mở ở Chế độ đọc (FR139).
 * Phần cũ của `modes/librarySearch.ts` nằm ở `librarySearch.test.ts`.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { SearchHit } from '../../src/config/library'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const ketQuaFlush: { value: 'clean' | 'failed' | 'still-dirty' } = { value: 'clean' }
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/panels/editorPanelState')>()
  return { ...actual, flushEditorBeforeDiscreteWrite: async () => ketQuaFlush.value }
})

function hit(over: Partial<SearchHit> = {}): SearchHit {
  return {
    work_id: 'id-a',
    work_name: 'Alpha',
    chapter_id: 7,
    chapter_ord: 1,
    chapter_title: 'Chuong Mot',
    segment_id: 42,
    field: 'target',
    snippet: 'cau ‹khoang› san',
    match_kind: 'exact',
    segment_ord: 3,
    segment_status: 'confirmed',
    work_status: 'in_progress',
    source_lang: 'zh',
    ...over,
  }
}

type Count = { work_id: string; target_count: number; source_count: number; source_count_is_lower_bound: boolean }
const count = (work_id: string, target_count: number, source_count = 0, lower = false): Count => ({
  work_id,
  target_count,
  source_count,
  source_count_is_lower_bound: lower,
})

function report(hits: SearchHit[], over: Record<string, unknown> = {}) {
  return {
    hits,
    total: hits.length,
    indexed_segments: 5,
    short_query: false,
    truncated: false,
    mode: 'exact',
    effective_mode: 'exact',
    widened: false,
    works_total: 5,
    works_with_text: 5,
    scope: 'both',
    chapters_total: 20,
    chapters_filtered_out: 0,
    work_counts: [] as Count[],
    source_counts_capped: false,
    ...over,
  }
}

const NONE_OPEN = { code: 'work.none_open', message_key: 'err.work.none_open', params: {}, retryable: false }

/** Mọi lời gọi `library_search` được ghi lại; `build` quyết báo cáo trả về. */
function mockSearch(build: (args: Record<string, unknown>) => unknown = () => report([hit()])) {
  const calls: Record<string, unknown>[] = []
  mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === 'library_list_works') {
      return Promise.resolve({ total: 0, matched: 0, works: [], genres: [], source_langs: [] })
    }
    if (cmd === 'read_work_lifecycle' || cmd === 'list_chapters') return Promise.reject(NONE_OPEN)
    if (cmd === 'library_search') {
      calls.push(args ?? {})
      return Promise.resolve(build(args ?? {}))
    }
    return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
  })
  return calls
}

beforeEach(async () => {
  mockInvoke.mockReset()
  ketQuaFlush.value = 'clean'
  vi.resetModules()
})

afterEach(() => {
  vi.restoreAllMocks()
})

async function fresh() {
  const search = await import('../../src/modes/librarySearch')
  const works = await import('../../src/modes/libraryWorks')
  search.resetLibrarySearch()
  works.resetLibraryWorks()
  return { search, works }
}

describe('phạm vi và bộ lọc gửi xuống lõi', () => {
  it('mặc định gửi scope both, không lọc, sắp updated_desc, không workId', async () => {
    const calls = mockSearch()
    const { search } = await fresh()
    search.librarySearchQuery.value = 'khoang'
    await search.runLibrarySearch()
    expect(calls).toHaveLength(1)
    expect(calls[0]).toMatchObject({ scope: 'both', status: [], sourceLang: null, sort: 'updated_desc', workId: null })
  })

  it('đổi phạm vi khi đã có truy vấn chạy lại đúng phạm vi mới', async () => {
    const calls = mockSearch()
    const { search } = await fresh()
    search.librarySearchQuery.value = 'khoang'
    await search.runLibrarySearch()
    search.setLibrarySearchScope('target')
    await vi.waitFor(() => expect(calls).toHaveLength(2))
    expect(calls[1]).toMatchObject({ scope: 'target' })
    expect(search.librarySearchScope.value).toBe('target')
  })

  it('đổi phạm vi khi ô tìm rỗng chỉ đổi cờ, 0 lượt IPC', async () => {
    const calls = mockSearch()
    const { search } = await fresh()
    search.setLibrarySearchScope('source')
    await Promise.resolve()
    expect(search.librarySearchScope.value).toBe('source')
    expect(calls).toHaveLength(0)
  })

  it('bộ lọc trạng thái, hướng dịch và khoá sắp của Library đi theo lượt tìm', async () => {
    const calls = mockSearch()
    const { search, works } = await fresh()
    works.toggleStatusFilter('done')
    works.setSourceLangFilter('zh')
    works.setSortKey('name_asc')
    search.librarySearchQuery.value = 'khoang'
    await search.runLibrarySearch()
    const last = calls[calls.length - 1]
    expect(last).toMatchObject({ status: ['done'], sourceLang: 'zh', sort: 'name_asc' })
  })

  it('"Xem cả N kết quả" chạy lại với workId của hit đang chọn; Enter bỏ workId', async () => {
    const calls = mockSearch(() => report([hit({ work_id: 'id-a' })], { work_counts: [count('id-a', 9)] }))
    const { search } = await fresh()
    search.librarySearchQuery.value = 'khoang'
    await search.runLibrarySearch()
    await search.seeAllLibrarySearchGroup()
    expect(calls[calls.length - 1]).toMatchObject({ workId: 'id-a' })
    expect(search.librarySearchWorkId.value).toBe('id-a')
    await search.runLibrarySearch()
    expect(calls[calls.length - 1]).toMatchObject({ workId: null })
    expect(search.librarySearchWorkId.value).toBeNull()
  })
})

describe('ms đo quanh lời gọi IPC', () => {
  it('librarySearchElapsedMs là hiệu performance.now() quanh lời gọi, làm tròn', async () => {
    mockSearch()
    const { search } = await fresh()
    const now = vi.spyOn(performance, 'now')
    now.mockReturnValueOnce(1000).mockReturnValueOnce(1012.6)
    search.librarySearchQuery.value = 'khoang'
    await search.runLibrarySearch()
    expect(search.librarySearchElapsedMs.value).toBe(13)
  })
})

describe('groupSearchHits — nhóm và số đếm thật', () => {
  it('đầu nhóm lấy số đếm thật, không phải số hit trả về; "N kết quả nữa" chỉ đếm hit đã trả về mà nhóm thu gọn chưa hiện', async () => {
    const { search } = await fresh()
    const hits = [1, 2, 3, 4, 5].map((n) => hit({ chapter_id: n, chapter_ord: n, segment_id: n, segment_ord: n }))
    const [group] = search.groupSearchHits(hits, [count('id-a', 40)], [])
    expect(group.count).toBe(40)
    expect(group.visible).toHaveLength(3)
    expect(group.remaining).toBe(2)
    expect(group.hasUnreturned).toBe(true)
    expect(group.countIsLowerBound).toBe(false)
  })

  it('mở rộng nhóm hiện hết hit đã trả về; con trỏ chỉ đi qua hit đang hiện', async () => {
    mockSearch(() =>
      report(
        [1, 2, 3, 4, 5].map((n) => hit({ chapter_id: n, chapter_ord: n, segment_id: n, segment_ord: n })),
        { work_counts: [count('id-a', 5)] },
      ),
    )
    const { search } = await fresh()
    search.librarySearchQuery.value = 'khoang'
    await search.runLibrarySearch()
    expect(search.librarySearchVisibleHits.value).toHaveLength(3)
    search.nextLibrarySearchHit()
    search.nextLibrarySearchHit()
    search.nextLibrarySearchHit()
    expect(search.librarySearchCursor.value).toBe(2)
    search.toggleLibrarySearchGroup('id-a')
    expect(search.librarySearchVisibleHits.value).toHaveLength(5)
    search.toggleLibrarySearchGroup('id-a')
    expect(search.librarySearchVisibleHits.value).toHaveLength(3)
  })

  it('số nguyên văn cận dưới ghi N+ ở nhóm và ở bộ đếm tổng', async () => {
    const { search } = await fresh()
    const [group] = search.groupSearchHits([hit({ field: 'source' })], [count('id-a', 0, 5000, true)], [])
    expect(group.count).toBe(5000)
    expect(group.countIsLowerBound).toBe(true)
    expect(group.hasUnreturned).toBe(true)
  })
})

describe('LibrarySearchResults.vue trên DOM mount thật', () => {
  let wrapper: ReturnType<typeof mount> | null = null
  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  async function mountWith(build: () => unknown, query = 'khoang') {
    mockSearch(build)
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const { search, works } = await fresh()
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()
    search.librarySearchQuery.value = query
    await search.runLibrarySearch()
    await wrapper.vm.$nextTick()
    return { search, works, view: wrapper }
  }

  it('có kết quả ⇒ lưới Tác phẩm không hiện; đầu nhóm ghi tên · hướng dịch · trạng thái · N kết quả', async () => {
    const { view } = await mountWith(() => report([hit()], { work_counts: [count('id-a', 1)] }))
    expect(view.find('.works-block').exists()).toBe(false)
    const group = view.find('[data-library-search-group]')
    expect(group.find('[data-library-search-group-name]').text()).toBe('Alpha')
    expect(group.find('.wk-meta').text()).toBe('Trung → Việt · Đang dịch')
    expect(group.find('[data-library-search-group-count]').text()).toBe('1 kết quả')
  })

  it('hàng hit ghi số Chương đậm, tiêu đề Chương đi kèm', async () => {
    const { view } = await mountWith(() =>
      report([hit({ chapter_ord: 47, chapter_title: 'Hoi thu nhat' })], { work_counts: [count('id-a', 1)] }),
    )
    const row = view.find('[data-library-search-hit]')
    expect(row.find('[data-library-search-hit-chapter]').text()).toBe('Chương 47')
    expect(row.find('.hit-chapter-title').text()).toBe('Hoi thu nhat')
  })

  it('đổi bộ lọc khi lượt tìm đầu còn đang chạy ⇒ chạy lại với bộ lọc mới', async () => {
    let releaseFirst: () => void = () => {}
    const calls = mockSearch()
    const answered: Record<string, unknown>[] = []
    mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === 'library_list_works') {
        return Promise.resolve({ total: 0, matched: 0, works: [], genres: [], source_langs: [] })
      }
      if (cmd === 'read_work_lifecycle' || cmd === 'list_chapters') return Promise.reject(NONE_OPEN)
      if (cmd === 'library_search') {
        calls.push(args ?? {})
        if (calls.length === 1) {
          return new Promise((resolve) => {
            releaseFirst = () => resolve(report([hit()]))
          })
        }
        answered.push(args ?? {})
        return Promise.resolve(report([hit()]))
      }
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const { search, works } = await fresh()
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()
    search.librarySearchQuery.value = 'khoang'
    const first = search.runLibrarySearch()
    works.setSourceLangFilter('en')
    await wrapper.vm.$nextTick()
    releaseFirst()
    await first
    await vi.waitFor(() => expect(answered).toHaveLength(1))
    expect(calls[0]).toMatchObject({ sourceLang: null })
    expect(answered[0]).toMatchObject({ sourceLang: 'en' })
  })

  it('chưa tìm gì ⇒ lưới Tác phẩm hiện', async () => {
    mockSearch()
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    await fresh()
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.works-block').exists()).toBe(true)
    expect(wrapper.find('[data-library-search-group]').exists()).toBe(false)
  })

  it('bộ đếm ghi "N kết quả · X ms" từ số đếm thật, không từ số hit', async () => {
    const hits = [1, 2].map((n) => hit({ segment_id: n, segment_ord: n }))
    const { view } = await mountWith(() => report(hits, { work_counts: [count('id-a', 40, 2)] }))
    expect(view.find('[data-library-search-status]').text()).toMatch(/^42 kết quả · \d+ ms$/)
  })

  it('đếm nguyên văn cận dưới ⇒ bộ đếm và đầu nhóm ghi N+', async () => {
    const { view } = await mountWith(() =>
      report([hit({ field: 'source' })], { work_counts: [count('id-a', 0, 5000, true)], source_counts_capped: true }),
    )
    expect(view.find('[data-library-search-status]').text()).toMatch(/^5000\+ kết quả/)
    expect(view.find('[data-library-search-group-count]').text()).toBe('5000+ kết quả')
  })

  it('nhóm có nhiều kết quả hơn hit trả về ⇒ có "Xem cả N kết quả trong Tác phẩm này"', async () => {
    const { view } = await mountWith(() => report([hit()], { work_counts: [count('id-a', 9)] }))
    expect(view.find('[data-library-search-group-see-all]').text()).toBe('Xem cả 9 kết quả trong Tác phẩm này')
  })

  it('nhóm có đủ hit ⇒ không có nút "Xem cả"', async () => {
    const { view } = await mountWith(() => report([hit()], { work_counts: [count('id-a', 1)] }))
    expect(view.find('[data-library-search-group-see-all]').exists()).toBe(false)
  })

  it('chế độ khoan dung ⇒ phần khớp mang lớp mark--loose, chính xác thì không; văn bản thuần, không HTML', async () => {
    const { view } = await mountWith(() =>
      report([hit(), hit({ segment_id: 43, segment_ord: 4, match_kind: 'lenient', snippet: '‹<b>x</b>›' })], {
        work_counts: [count('id-a', 2)],
      }),
    )
    const marks = view.findAll('[data-library-search-mark]')
    expect(marks[0]?.classes()).not.toContain('mark--loose')
    expect(marks[1]?.classes()).toContain('mark--loose')
    expect(marks[1]?.text()).toBe('<b>x</b>')
    expect(marks[1]?.find('b').exists()).toBe(false)
  })

  it('vạch lề theo phía: hàng bản dịch và hàng nguyên văn mang data-field và lớp riêng', async () => {
    const { view } = await mountWith(() =>
      report([hit(), hit({ field: 'source', segment_id: 43, segment_ord: 4 })], { work_counts: [count('id-a', 1, 1)] }),
    )
    const rows = view.findAll('[data-library-search-hit]')
    expect(rows.map((row) => row.attributes('data-field'))).toEqual(['target', 'source'])
    expect(rows[0]?.classes()).toContain('hitrow--target')
    expect(rows[1]?.classes()).toContain('hitrow--source')
  })

  it('hàng ghi số câu và trạng thái xác nhận; hit cấp Chương ghi Chương, không ghi câu', async () => {
    const { view } = await mountWith(() =>
      report(
        [
          hit(),
          hit({ segment_id: 44, segment_ord: 5, segment_status: 'draft' }),
          hit({ chapter_id: 9, chapter_ord: 2, segment_id: null, segment_ord: null, segment_status: null, field: 'source' }),
        ],
        { work_counts: [count('id-a', 2, 1)] },
      ),
    )
    const rows = view.findAll('[data-library-search-hit]')
    expect(rows[0]?.find('.hit-locus').text()).toContain('câu 3 · đã xác nhận')
    expect(rows[1]?.find('.hit-locus').text()).toContain('câu 5 · chưa xác nhận')
    expect(rows[2]?.find('.hit-locus').text()).toContain('cấp Chương')
    expect(rows[2]?.find('.hit-locus').text()).not.toMatch(/câu \d/)
  })

  it('mỗi hàng có nút "Mở ở Chế độ đọc"; gợi ý phím không nhắc ⌘↵', async () => {
    const { view } = await mountWith(() => report([hit()], { work_counts: [count('id-a', 1)] }))
    expect(view.find('[data-library-search-open-reading]').text()).toBe('Mở ở Chế độ đọc')
    const hint = view.find('[data-library-search-key-hint]')
    if (hint.exists()) expect(hint.text()).not.toContain('⌘↵')
  })

  it('Chỉ bản dịch: phạm vi Nguyên văn trả 0 ⇒ câu rỗng nêu phạm vi đã tìm', async () => {
    const { view } = await mountWith(() => report([], { scope: 'source' }))
    expect(view.find('[data-library-search-empty-scope]').text()).toBe('Đã tìm trong nguyên văn; chưa tìm ở bản dịch.')
  })

  it('đổi sang phạm vi Bản dịch ⇒ hit hiện ra (cùng truy vấn)', async () => {
    const calls = mockSearch((args) =>
      args.scope === 'target'
        ? report([hit()], { scope: 'target', work_counts: [count('id-a', 1)] })
        : report([], { scope: String(args.scope) }),
    )
    const { default: LibraryMode } = await import('../../src/modes/LibraryMode.vue')
    const { search } = await fresh()
    wrapper = mount(LibraryMode)
    await wrapper.vm.$nextTick()
    search.librarySearchQuery.value = 'khoang'
    await search.runLibrarySearch()
    search.setLibrarySearchScope('source')
    await vi.waitFor(() => expect(calls[calls.length - 1]).toMatchObject({ scope: 'source' }))
    await vi.waitFor(() => expect(search.librarySearchBusy.value).toBe(false))
    await wrapper.vm.$nextTick()
    expect(search.librarySearchHits.value).toHaveLength(0)
    expect(wrapper.find('[data-library-search-hit]').exists()).toBe(false)
    expect(wrapper.find('[data-library-search-empty]').exists()).toBe(true)
    search.setLibrarySearchScope('target')
    await vi.waitFor(() => expect(search.librarySearchHits.value).toHaveLength(1))
    await wrapper.vm.$nextTick()
    expect(wrapper.find('[data-library-search-hit]').exists()).toBe(true)
  })

  it('lọc chặn: 0 kết quả kèm "X trong Y Chương không được tìm tới" và bước "Bỏ lọc"', async () => {
    const { works, view } = await mountWith(() => report([], { chapters_filtered_out: 12, chapters_total: 30 }))
    works.toggleStatusFilter('done')
    await vi.waitFor(() => expect(mockInvoke.mock.calls.filter((c) => c[0] === 'library_search').length).toBeGreaterThan(1))
    await view.vm.$nextTick()
    await view.vm.$nextTick()
    expect(view.find('[data-library-search-empty-filtered]').text()).toBe(
      'Bộ lọc đang thu hẹp phạm vi — 12 trong 30 Chương không được tìm tới.',
    )
    expect(view.find('[data-library-search-clear-filters]').text()).toBe('Bỏ lọc')
  })

  it('không có bộ lọc ⇒ không có bước "Bỏ lọc" và không có dòng chặn', async () => {
    const { view } = await mountWith(() => report([]))
    expect(view.find('[data-library-search-empty]').exists()).toBe(true)
    expect(view.find('[data-library-search-empty-filtered]').exists()).toBe(false)
    expect(view.find('[data-library-search-clear-filters]').exists()).toBe(false)
  })

  it('tự nới trong bộ lọc: dòng chế độ mang lớp mode-note--relax', async () => {
    const { view } = await mountWith(() =>
      report([hit({ match_kind: 'lenient' })], {
        widened: true,
        effective_mode: 'lenient',
        work_counts: [count('id-a', 1)],
      }),
    )
    expect(view.find('[data-library-search-mode-note]').classes()).toContain('mode-note--relax')
  })

  it('chế độ chính xác ⇒ dòng chế độ không có lớp nới', async () => {
    const { view } = await mountWith(() => report([hit()], { work_counts: [count('id-a', 1)] }))
    expect(view.find('[data-library-search-mode-note]').classes()).not.toContain('mode-note--relax')
  })
})

describe('config/library.ts::searchLibrary — guard của trường wire mới', () => {
  const required = [
    'scope',
    'chapters_total',
    'chapters_filtered_out',
    'work_counts',
    'source_counts_capped',
  ]
  for (const field of required) {
    it(`báo cáo thiếu \`${field}\` bị từ chối`, async () => {
      const bad: Record<string, unknown> = report([hit()])
      delete bad[field]
      mockInvoke.mockResolvedValue(bad)
      const { searchLibrary } = await import('../../src/config/library')
      const result = await searchLibrary('x')
      expect(result.report).toBeNull()
      expect(result.error).not.toBeNull()
    })
  }

  for (const field of ['segment_ord', 'segment_status', 'work_status', 'source_lang']) {
    it(`hit thiếu \`${field}\` bị từ chối`, async () => {
      const badHit: Record<string, unknown> = { ...hit() }
      delete badHit[field]
      mockInvoke.mockResolvedValue(report([badHit as unknown as SearchHit]))
      const { searchLibrary } = await import('../../src/config/library')
      const result = await searchLibrary('x')
      expect(result.report).toBeNull()
    })
  }

  it('gửi đủ scope/status/sourceLang/sort/workId, null khi không đặt', async () => {
    mockInvoke.mockResolvedValue(report([hit()]))
    const { searchLibrary } = await import('../../src/config/library')
    await searchLibrary('x', undefined, 'exact', { scope: 'source', status: ['done'], sourceLang: 'en', sort: 'name_asc', workId: 'w' })
    expect(mockInvoke.mock.calls[0]?.[1]).toMatchObject({
      scope: 'source',
      status: ['done'],
      sourceLang: 'en',
      sort: 'name_asc',
      workId: 'w',
    })
  })
})

describe('openSearchHitInReading — mở hit ở Chế độ đọc', () => {
  const OPENED_WORK = {
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
  const OPEN_CHAPTER = { chapter_id: 7, source_text: 'nguon', source_lang: 'zh' }

  function mockOpen(h: SearchHit, openChapterFails: boolean) {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'library_search') return Promise.resolve(report([h], { work_counts: [count('id-a', 1)] }))
      if (cmd === 'open_work') return Promise.resolve(OPENED_WORK)
      if (cmd === 'open_chapter') {
        return openChapterFails
          ? Promise.reject({ code: 'chapter.not_found', message_key: 'err.chapter.not_found', params: {}, retryable: false })
          : Promise.resolve(OPEN_CHAPTER)
      }
      if (cmd === 'list_chapters') return Promise.resolve([])
      if (cmd === 'read_open_chapter_segments') {
        return Promise.resolve({
          chapter_id: 7,
          segments: [41, 42, 43].map((id) => ({
            id,
            ord: id,
            source_text: `nguon ${id}`,
            target_text: '',
            status: 'draft',
            is_paragraph_end: false,
            is_target_paragraph_end: false,
            is_omitted: false,
            role: null,
            retired_at: null,
            translation_origin: '',
          })),
          caret_segment_id: 41,
          assets: [],
          assets_dir: '',
          tm_filled_segment_ids: [],
          tm_prefill: { kind: 'ran', hidden_count: 0 },
        })
      }
      if (cmd === 'read_open_chapter') return Promise.resolve(OPEN_CHAPTER)
      if (cmd === 'save_chapter_position') return Promise.resolve(null)
      return Promise.reject(new Error(`lenh khong mong doi: ${cmd}`))
    })
  }

  async function run(h: SearchHit, openChapterFails = false) {
    mockOpen(h, openChapterFails)
    const { search } = await fresh()
    const reading = await import('../../src/modes/readingState')
    const modeState = await import('../../src/modes/modeState')
    reading.resetReading()
    modeState.setMode('library')
    search.librarySearchQuery.value = 'khoang'
    await search.runLibrarySearch()
    await search.openCurrentLibrarySearchHitInReading()
    return { reading, modeState }
  }

  it('hit ở câu ⇒ Chế độ đọc hiện và neo đặt đúng câu của hit', async () => {
    const { reading, modeState } = await run(hit({ segment_id: 42 }))
    expect(modeState.currentMode.value).toBe('reading')
    expect(reading.readingAnchorSegmentId.value).toBe(42)
  })

  it('hit cấp Chương (segment_id null) ⇒ vào Chế độ đọc, không neo vào câu nào', async () => {
    const { reading, modeState } = await run(hit({ segment_id: null, segment_ord: null, segment_status: null, field: 'source' }))
    expect(modeState.currentMode.value).toBe('reading')
    expect(reading.readingAnchorSegmentId.value).toBeNull()
  })

  it('Chương mở trượt ⇒ ở lại Library, neo không đổi', async () => {
    const { reading, modeState } = await run(hit({ segment_id: 42 }), true)
    expect(modeState.currentMode.value).toBe('library')
    expect(reading.readingAnchorSegmentId.value).toBeNull()
  })

  it('mở Tác phẩm bị chặn vì flush chưa sạch ⇒ không mở Chương, ở lại Library', async () => {
    ketQuaFlush.value = 'still-dirty'
    const { modeState } = await run(hit({ segment_id: 42 }))
    expect(mockInvoke.mock.calls.some((c) => c[0] === 'open_chapter')).toBe(false)
    expect(modeState.currentMode.value).toBe('library')
  })

  it('không có hit nào đang chọn ⇒ no-op', async () => {
    const { search } = await fresh()
    await search.openCurrentLibrarySearchHitInReading()
    expect(mockInvoke).not.toHaveBeenCalled()
  })
})
