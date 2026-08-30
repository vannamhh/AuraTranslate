/**
 * `config/reading.ts` + `modes/readingState.ts` + `modes/ReadingMode.vue` — Story 5.11 ·
 * Story 5.12 (FR120).
 *
 * ⚠️ **PHẠM VI** — `happy-dom` canh HÀNH VI, không hình học (`tests/AGENTS.md`). Mọi mệnh đề
 * về bố cục thật (bề rộng cột, vị trí lề song ngữ) thuộc bàn đo e2e
 * (`story-5-11-reading-mode.e2e.mjs`/`story-5-12-reading-frontier.e2e.mjs`), không tệp này.
 *
 * 🔴 Đối chứng trung tâm của tệp: câu đã cắt bỏ không bao giờ tới được webview vì Rust đã lọc
 * TRƯỚC khi dữ liệu rời `project.db` — Vue không có, và không cần, một đường lọc thứ hai. Ca
 * "gỡ vế lọc ở fixture" bên dưới là phép GỠ THẬT (đổi dữ liệu giả để mô phỏng một Rust ĐÃ QUÊN
 * lọc), không một lượt chèn thêm — nếu Vue có ẩn một `v-if="!s.is_omitted"` thì ca đó vẫn cho
 * ra 2 câu; nó cho ra 3 câu, chứng minh không có đường lọc nào ở tầng này.
 *
 * 🔵 SỬA TẠI CHỖ (2026-08-30, Story 5.12) — bề mặt đổi từ MỘT Chương (`ReadingChapter` trần)
 * sang MỘT LƯỢT ĐỌC (`ReadingRun`). Mọi fixture dưới đây dựng một `ReadingRun` (mảng
 * `chapters` + `frontier`) thay vì một `ReadingChapter` đơn; các ca dựa trên hình dạng cũ
 * (`chapterHasSegments`, nhánh `'empty-unknown'`, đọc thẳng `readingChapter`) đã SỬA theo —
 * xem `deferred-work.md`/Code Map của story cho lý lẽ đầy đủ.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const WORK_NONE_OPEN_ERROR = {
  code: 'work.none_open',
  message_key: 'err.work.none_open',
  params: {},
  retryable: false,
}

const OTHER_ERROR = {
  code: 'store.read_failed',
  message_key: 'err.store.read_failed',
  params: { store: 'project.db' },
  retryable: false,
}

type ReadingSegmentFixture = { id: number; source_text: string; target_text: string; is_confirmed?: boolean }

/** Một `ReadingChapter` fixture — `segment_count` mặc định đếm đúng số câu của `paragraphs`
 * (ca thường nhất: không câu nào bị cắt bỏ), ghi đè được cho ca "mọi câu đã cắt bỏ". */
function chapterFixture(
  paragraphs: ReadingSegmentFixture[][],
  overrides: Partial<{ chapter_id: number; chapter_ord: number; chapter_title: string | null; segment_count: number }> = {},
) {
  const segmentCount = paragraphs.reduce((sum, p) => sum + p.length, 0)
  return {
    chapter_id: 1,
    chapter_ord: 1,
    chapter_title: 'Chuong Mot',
    paragraphs: paragraphs.map((segments) => ({
      segments: segments.map((s) => ({ is_confirmed: true, ...s })),
    })),
    segment_count: segmentCount,
    ...overrides,
  }
}

/** Mốc biên "hết Tác phẩm" — dùng mặc định cho mọi `runFixture` không quan tâm mốc biên. */
function endOfWorkFrontier() {
  return { kind: 'end-of-work' as const, chapter: null }
}

function nextNotDoneFrontier(overrides: Partial<{ chapter_id: number; chapter_ord: number; chapter_title: string | null; status: string }> = {}) {
  return {
    kind: 'next-not-done' as const,
    chapter: { chapter_id: 2, chapter_ord: 2, chapter_title: 'Chuong Hai', status: 'in_progress', ...overrides },
  }
}

function runFixture(chapters: ReturnType<typeof chapterFixture>[], frontier: ReturnType<typeof endOfWorkFrontier | typeof nextNotDoneFrontier> = endOfWorkFrontier()) {
  return { chapters, frontier }
}

function chapterRow(overrides: Partial<{ chapter_id: number; ord: number; title: string | null; status: string; segment_count: number }> = {}) {
  return { chapter_id: 1, ord: 1, title: 'Chuong Mot', status: 'in_progress', segment_count: 4, ...overrides }
}

beforeEach(async () => {
  mockInvoke.mockReset()
  const state = await import('../../src/modes/readingState')
  state.resetReading()
  state.resetReadingToc()
  state.resetReadingPreferences()
})

// ═════════════════════════════════════════════════════════════════════════════════
// `config/reading.ts::readReadingRun` — ba trạng thái, cùng khuôn mọi adapter khác.
// ═════════════════════════════════════════════════════════════════════════════════

describe('config/reading.ts::readReadingRun', () => {
  it('đọc được ⇒ { run, error: null }', async () => {
    mockInvoke.mockResolvedValueOnce(runFixture([chapterFixture([[{ id: 1, source_text: 'a', target_text: 'b' }]])]))
    const { readReadingRun } = await import('../../src/config/reading')
    const result = await readReadingRun()
    expect(result.error).toBeNull()
    expect(result.run?.chapters[0]?.chapter_id).toBe(1)
    expect(result.run?.chapters[0]?.paragraphs).toHaveLength(1)
  })

  it('Rust trả lời bằng một lỗi ⇒ { run: null, error }', async () => {
    mockInvoke.mockRejectedValueOnce(WORK_NONE_OPEN_ERROR)
    const { readReadingRun } = await import('../../src/config/reading')
    const result = await readReadingRun()
    expect(result.run).toBeNull()
    expect(result.error?.code).toBe('work.none_open')
  })

  it('không có cầu IPC (chạy ngoài Tauri) ⇒ { run: null, error: null }', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('khong co cau IPC trong window nay'))
    const { readReadingRun } = await import('../../src/config/reading')
    const result = await readReadingRun()
    expect(result.run).toBeNull()
    expect(result.error).toBeNull()
  })

  it('hình dạng sai (thiếu `frontier`) ⇒ lỗi hồi phòng, không ném', async () => {
    mockInvoke.mockResolvedValueOnce({ chapters: [] })
    const { readReadingRun } = await import('../../src/config/reading')
    const result = await readReadingRun()
    expect(result.run).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// `ReadingMode.vue` — câu đã cắt bỏ không bao giờ tới được webview.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/ReadingMode.vue — segment đã cắt bỏ', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('paragraphs Rust gửi sao thì Vue hiện nguyên vậy — câu đã cắt bỏ vắng mặt hoàn toàn', async () => {
    // Fixture ĐÃ LỌC — giống đúng một lượt đọc thật: câu 2 (bị cắt bỏ) đã vắng mặt khỏi
    // `paragraphs` (AC5), không phải bị Vue ẩn đi.
    mockInvoke.mockResolvedValueOnce(
      runFixture([
        chapterFixture([
          [
            { id: 1, source_text: 'Mot.', target_text: 'Cau mot.' },
            { id: 3, source_text: 'Ba.', target_text: 'Cau ba.' },
          ],
        ]),
      ]),
    )
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const spans = wrapper.findAll('.column .paragraph span')
    expect(spans.map((s) => s.text())).toEqual(['Cau mot.', 'Cau ba.'])
  })

  it('🔴 GỠ VẾ LỌC ở fixture (mô phỏng Rust quên lọc) ⇒ câu hiện ra — Vue KHÔNG lọc lần hai', async () => {
    // Cùng đoạn ở trên, nhưng câu 2 (đáng lẽ vắng mặt) nay CÓ MẶT trong `paragraphs` — mô
    // phỏng một hồi quy phía Rust. Nếu `ReadingMode.vue` giấu một đường lọc riêng
    // (`v-if="!s.is_omitted"`) thì ca này vẫn cho 2 span; nó cho 3, chứng minh không có.
    mockInvoke.mockResolvedValueOnce(
      runFixture([
        chapterFixture([
          [
            { id: 1, source_text: 'Mot.', target_text: 'Cau mot.' },
            { id: 2, source_text: 'Hai.', target_text: 'Cau hai dang le da bi cat bo.' },
            { id: 3, source_text: 'Ba.', target_text: 'Cau ba.' },
          ],
        ]),
      ]),
    )
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const spans = wrapper.findAll('.column .paragraph span')
    expect(spans).toHaveLength(3)
    expect(spans.map((s) => s.text())).toContain('Cau hai dang le da bi cat bo.')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// `role="status"` — tám nhánh phân biệt được nhau qua `data-reading-status`.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/ReadingMode.vue — role="status" phân biệt được các ca', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  function status() {
    const node = wrapper?.find('p.status[role="status"]')
    return { kind: node?.attributes('data-reading-status'), text: node?.text() }
  }

  it('chưa gọi `ensureReadingLoaded()` ⇒ "chưa nạp", KHÔNG khẳng định "chưa có gì"', async () => {
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    wrapper = mount(ReadingMode)
    await wrapper.vm.$nextTick()

    expect(status().kind).toBe('not-loaded')
    expect(status().text).toBe('')
  })

  it('lượt nạp CÒN ĐANG BAY ⇒ "đang nạp", không câu "chưa có gì" nào hiện ra', async () => {
    let resolveInvoke: (value: unknown) => void = () => {}
    mockInvoke.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveInvoke = resolve
        }),
    )
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)

    const pending = state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    expect(status().kind).toBe('pending')
    expect(status().text).not.toBe('')

    resolveInvoke(runFixture([]))
    await pending
  })

  it('chưa mở Tác phẩm nào ⇒ "no-work", câu nói rõ vì sao', async () => {
    mockInvoke.mockRejectedValueOnce(WORK_NONE_OPEN_ERROR)
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    expect(status().kind).toBe('no-work')
    expect(status().text).not.toBe('')
  })

  it('một lỗi KHÁC ⇒ "error", phân biệt với "no-work"', async () => {
    mockInvoke.mockRejectedValueOnce(OTHER_ERROR)
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    expect(status().kind).toBe('error')
  })

  it('dãy đọc RỖNG (Chương đang mở chưa `done`) ⇒ "frontier-only"', async () => {
    mockInvoke.mockResolvedValueOnce(runFixture([], nextNotDoneFrontier({ chapter_id: 1, chapter_ord: 1 })))
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    expect(status().kind).toBe('frontier-only')
    expect(status().text).not.toBe('')
  })

  it('dãy có Chương nhưng KHÔNG segment nào ⇒ "empty-chapters"', async () => {
    mockInvoke.mockResolvedValueOnce(runFixture([chapterFixture([], { segment_count: 0 })]))
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    expect(status().kind).toBe('empty-chapters')
  })

  it('🔴 MỌI câu đều cắt bỏ ⇒ "all-omitted" — KHÁC "empty-chapters" dù `paragraphs` cùng rỗng', async () => {
    mockInvoke.mockResolvedValueOnce(runFixture([chapterFixture([], { segment_count: 4 })]))
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    expect(status().kind).toBe('all-omitted')
    // Câu trạng thái không được RỖNG (khác "content"), và bản thân `kind` đã khác
    // "empty-chapters" ở phép so trên — hai ca cùng `paragraphs = []` nhưng hai câu khác nhau.
    expect(status().text).not.toBe('')
  })

  it('lượt đọc có nội dung ⇒ "content", câu trạng thái RỖNG (đoạn văn tự nói đủ)', async () => {
    mockInvoke.mockResolvedValueOnce(runFixture([chapterFixture([[{ id: 1, source_text: 'a', target_text: 'b' }]])]))
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    expect(status().kind).toBe('content')
    expect(status().text).toBe('')
    expect(wrapper.find('.column').exists()).toBe(true)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Không công cụ biên tập nào trong cây — §Always của story.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/ReadingMode.vue — không bề mặt biên tập nào', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('không `[data-col]`, không `contenteditable`, không nút xác nhận', async () => {
    mockInvoke.mockResolvedValueOnce(runFixture([chapterFixture([[{ id: 1, source_text: 'Mot.', target_text: 'Cau mot.' }]])]))
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-col]').exists()).toBe(false)
    expect(wrapper.find('[contenteditable]').exists()).toBe(false)
    expect(wrapper.findAll('button').every((b) => !b.text().toLowerCase().includes('xác nhận'))).toBe(true)
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Mục lục — hàng *"Mở mục lục khi chưa mở Tác phẩm"* của I/O Matrix Story 5.11.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/ReadingMode.vue — mục lục khi chưa mở Tác phẩm', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  /**
   * 🔴 Danh sách rỗng phải nói **VÌ SAO** nó rỗng — lớp lỗi *rỗng im lặng* của dự án. Hai câu
   * KHÔNG được cùng hiện: *"Tác phẩm này chưa có Chương nào"* là một **sự thật về dữ liệu**, còn
   * một lượt `list_chapters` bị Rust từ chối là một **lỗi**. Trộn hai câu là khẳng định dứt khoát
   * một điều màn hình chưa biết.
   */
  it('`list_chapters` trả lỗi ⇒ câu lỗi hiện, câu "chưa có Chương nào" KHÔNG hiện', async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_chapters') return Promise.reject(WORK_NONE_OPEN_ERROR)
      return Promise.reject(WORK_NONE_OPEN_ERROR)
    })
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)

    await state.openTableOfContents()
    await wrapper.vm.$nextTick()

    expect(state.readingTocOpen.value).toBe(true)
    expect(state.readingTocChapters.value).toEqual([])
    expect(state.readingTocError.value).not.toBeNull()
    expect(wrapper.find('.toc-error').text()).not.toBe('')
    expect(wrapper.find('.toc-empty').text()).toBe('')
  })

  /** Không một lượt mở Chương nào được phát đi trên một danh sách rỗng. */
  it('mở Chương đang chọn trên một mục lục RỖNG ⇒ không lượt `open_chapter` nào phát đi', async () => {
    mockInvoke.mockImplementation(() => Promise.reject(WORK_NONE_OPEN_ERROR))
    const state = await import('../../src/modes/readingState')

    await state.openTableOfContents()
    mockInvoke.mockClear()
    await state.openCurrentTocChapter()

    expect(mockInvoke.mock.calls.map((c) => c[0])).not.toContain('open_chapter')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng *"Gõ phím trần trong một ô nhập"* của I/O Matrix — trên bộ command THẬT.
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ **Không chồng lên `check:commands` Kiểm D.** Kiểm D nghiệm thu *cơ chế* (`isTypingZone`)
// trên một **registry GIẢ** với một hợp âm `KeyB` bịa ra. Ca dưới đây hỏi một câu khác, và nó
// chỉ trả lời được từ story này trở đi: **bộ command THẬT** có thật sự gán `B`/`2`/`D` ở dạng
// TRẦN không, và ba hợp âm ấy có đi qua đúng luật vùng gõ không. Một lượt đổi `B` thành `Mod+B`
// làm ca này đỏ; Kiểm D thì vẫn xanh.

describe('src/commands — hợp âm TRẦN của Chế độ đọc đi qua luật vùng gõ', () => {
  const TYPING_ZONE = { tagName: 'INPUT', isContentEditable: false }
  const OUTSIDE = { tagName: 'DIV', isContentEditable: false }

  it('`B` · `2` · `D` chạy ngoài vùng gõ, và KHÔNG chạy trong một ô nhập', async () => {
    const commands = await import('../../src/commands')
    const { createKeymap } = await import('../../src/commands/keys')

    const called: string[] = []
    commands.installCommands({
      isMac: true,
      // `setMode` là dep BẮT BUỘC của `CommandDeps` (vỏ chuyển chế độ, Story 1.6) — không
      // liên quan story này, nhưng thiếu nó thì `vue-tsc` đỏ và bộ command không dựng được.
      setMode: () => {},
      toggleReadingBilingual: () => {
        called.push('bilingual')
      },
      setReadingLevelBalanced: () => {
        called.push('level_balanced')
      },
      toggleReadingTheme: () => {
        called.push('theme')
      },
    })
    const keymap = createKeymap(commands.commandRegistry, { isMac: true })

    for (const code of ['KeyB', 'Digit2', 'KeyD']) {
      expect(keymap.handle({ code, target: OUTSIDE })).toBe(true)
    }
    expect(called).toEqual(['bilingual', 'level_balanced', 'theme'])

    called.length = 0
    for (const code of ['KeyB', 'Digit2', 'KeyD']) {
      expect(keymap.handle({ code, target: TYPING_ZONE })).toBe(false)
    }
    expect(called).toEqual([])
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Nhánh THÀNH CÔNG của mục lục, và ba hàng của I/O Matrix mà lượt rà 2026-08-30 tìm ra
// là chưa ai canh.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/readingState — mở một Chương từ mục lục (nhánh THÀNH CÔNG)', () => {
  /**
   * 🔴 AC của story nói: *"chọn một Chương và xác nhận ⇒ Chế độ đọc hiển thị Chương đó, VÀ
   * tiêu điểm ở lại trong Chế độ đọc"*. Trước ca này chỉ nhánh `row === undefined` có test —
   * gỡ `reloadReading()`, gỡ `tocOpen.value = false`, hay gỡ `enterFocus('mode.reading')` đều
   * KHÔNG làm một ca nào đỏ.
   */
  it('nạp lại nội dung, đóng mục lục, và trả tiêu điểm về Chế độ đọc', async () => {
    const chapterOne = runFixture([chapterFixture([[{ id: 1, source_text: 'Mot.', target_text: 'Cau mot.' }]])])
    const chapterTwo = runFixture([
      chapterFixture([[{ id: 9, source_text: 'Chin.', target_text: 'Cau chin.' }]], { chapter_id: 2, chapter_ord: 2, chapter_title: 'Chuong Hai' }),
    ])
    let readCount = 0
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === 'list_chapters') {
        return Promise.resolve([chapterRow({ chapter_id: 1 }), chapterRow({ chapter_id: 2, ord: 2, title: 'Chuong Hai' })])
      }
      if (cmd === 'open_chapter') return Promise.resolve({ chapter_id: 2, source_text: 'x', source_lang: 'zh' })
      if (cmd === 'read_reading_run') {
        readCount += 1
        return Promise.resolve(readCount === 1 ? chapterOne : chapterTwo)
      }
      // `openChapterById` gọi thêm vài lệnh của Editor/Source — không ca nào ở đây quan tâm.
      return Promise.resolve(null)
    })

    const state = await import('../../src/modes/readingState')
    await state.ensureReadingLoaded()
    expect(readCount).toBe(1)

    await state.openTableOfContents()
    state.nextTocChapter()
    expect(state.readingTocCursor.value).toBe(1)

    await state.openCurrentTocChapter()

    // ① Nội dung được nạp LẠI — một vòng `read_reading_run` thứ hai đã đi.
    expect(readCount).toBe(2)
    expect(state.readingRun.value?.chapters[0]?.paragraphs[0]?.segments[0]?.id).toBe(9)
    // ② Lớp phủ đóng lại.
    expect(state.readingTocOpen.value).toBe(false)
  })
})

describe('modes/ReadingMode.vue — hai câu trong một đoạn không được dính nhau', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  /**
   * 🔴 Đo bằng chuỗi HIỂN THỊ, không bằng `toContain` từng câu một — `toContain` xanh y hệt
   * trên `"Cau mot.Cau hai."` lẫn `"Cau mot. Cau hai."`, và đó chính là lý do bàn đo e2e của
   * story không thấy lỗi này.
   */
  it('có ĐÚNG một dấu cách giữa hai câu, và không dấu cách thừa ở đầu/cuối đoạn', async () => {
    mockInvoke.mockResolvedValueOnce(
      runFixture([
        chapterFixture([
          [
            { id: 1, source_text: 'Mot.', target_text: 'Cau mot.' },
            { id: 2, source_text: 'Hai.', target_text: 'Cau hai.' },
          ],
        ]),
      ]),
    )
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.paragraph').text()).toBe('Cau mot. Cau hai.')
  })
})
