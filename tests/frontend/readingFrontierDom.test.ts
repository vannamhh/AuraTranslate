/**
 * `ReadingMode.vue` — khối `.frontier` và `.chapter-note` RENDER ĐÚNG CHỮ trên DOM. Story
 * 5.12, FR120 (lượt rà 2026-08-30, Bản vá 1–3 của vòng rà bốn lớp).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO TỆP NÀY TỒN TẠI RIÊNG, KHÔNG GỘP VÀO `readingFrontier.test.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `readingFrontier.test.ts` canh STATE (`readingRun.value.frontier.*`,
 * `openFrontierInWorkspace()`) — không mount component nào. Trước tệp này, KHÔNG ca vitest
 * nào đọc `document.querySelector('.frontier')?.textContent`; ca duy nhất làm việc đó là
 * bàn đo e2e (`story-5-12-reading-frontier.e2e.mjs`), thứ chạy NGOÀI `push` (nhịp đêm,
 * `ci.yml:68`). ⇒ đảo `v-if`/`v-else` giữa hai nhánh của `.frontier`, hay đảo ternary của
 * `chapterEmptyNote()`, đi qua sạch MỌI cổng ở một lượt `push` — đúng khoảng hở mà vòng rà
 * bốn lớp bắt được. Mỗi ca dưới đây kèm một đối chứng ĐỎ THẬT đã chạy (xem lịch sử review).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { t } from '../../src/i18n'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

type SegmentFixture = { id: number; source_text: string; target_text: string; is_confirmed?: boolean }

function chapterFixture(
  chapterId: number,
  ord: number,
  paragraphs: SegmentFixture[][],
  overrides: Partial<{ chapter_title: string | null; segment_count: number }> = {},
) {
  const segmentCount = paragraphs.reduce((sum, p) => sum + p.length, 0)
  return {
    chapter_id: chapterId,
    chapter_ord: ord,
    chapter_title: null,
    paragraphs: paragraphs.map((segments) => ({
      segments: segments.map((s) => ({ is_confirmed: true, is_marked: false, ...s })),
    })),
    segment_count: segmentCount,
    ...overrides,
  }
}

function nextNotDoneFrontier(overrides: Partial<{ chapter_id: number; chapter_ord: number; chapter_title: string | null; status: string }> = {}) {
  return {
    kind: 'next-not-done' as const,
    chapter: { chapter_id: 9, chapter_ord: 9, chapter_title: null, status: 'in_progress', ...overrides },
  }
}

function endOfWorkFrontier() {
  return { kind: 'end-of-work' as const, chapter: null }
}

beforeEach(async () => {
  mockInvoke.mockReset()
  const state = await import('../../src/modes/readingState')
  state.resetReading()
  state.resetReadingToc()
})

// ═════════════════════════════════════════════════════════════════════════════════
// BẢN VÁ 1 — khối `.frontier` render ĐÚNG chữ theo `kind`, và nút "Dịch tiếp" CÓ/KHÔNG
// có mặt đúng theo `kind`.
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/ReadingMode.vue — khối .frontier render đúng chữ theo `kind`', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('kind = "next-not-done": nêu đích danh Chương chặn + nhãn trạng thái, nút "Dịch tiếp" CÓ mặt', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [chapterFixture(1, 1, [[{ id: 1, source_text: 'a', target_text: 'b' }]])],
      frontier: nextNotDoneFrontier({ chapter_ord: 2 }),
    })
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const blocked = wrapper.find('[data-reading-frontier-kind="next-not-done"]')
    expect(blocked.exists()).toBe(true)
    expect(blocked.text()).toContain(t('mode.reading.chapter_untitled', { ord: '2' }))
    expect(blocked.text()).toContain(t('lifecycle.in_progress'))
    expect(wrapper.find('[data-reading-frontier-kind="end-of-work"]').exists()).toBe(false)
    expect(wrapper.find('[data-reading-frontier-continue]').exists()).toBe(true)
  })

  it('kind = "end-of-work": câu KHÁC câu "next-not-done", nút "Dịch tiếp" KHÔNG có mặt', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [chapterFixture(1, 1, [[{ id: 1, source_text: 'a', target_text: 'b' }]])],
      frontier: endOfWorkFrontier(),
    })
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const endOfWork = wrapper.find('[data-reading-frontier-kind="end-of-work"]')
    expect(endOfWork.exists()).toBe(true)
    expect(endOfWork.text()).toBe(t('mode.reading.frontier_end_of_work'))
    expect(wrapper.find('[data-reading-frontier-kind="next-not-done"]').exists()).toBe(false)
    expect(wrapper.find('[data-reading-frontier-continue]').exists()).toBe(false)
  })

  it('`status = "finished"` (giá trị lạ) hiện CHUỖI THÔ, không một nhãn bốn trạng thái đã tra', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [chapterFixture(1, 1, [[{ id: 1, source_text: 'a', target_text: 'b' }]])],
      frontier: nextNotDoneFrontier({ status: 'finished' }),
    })
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const blocked = wrapper.find('[data-reading-frontier-kind="next-not-done"]')
    expect(blocked.text()).toContain('finished')
    for (const status of ['not_started', 'in_progress', 'paused', 'done']) {
      expect(blocked.text()).not.toContain(t(`lifecycle.${status}`))
    }
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// BẢN VÁ 2 — `.chapter-note` của TỪNG Chương nói ĐÚNG vì sao nó rỗng, trong CÙNG một dãy
// mang cả hai ca (Chương rỗng thật VÀ Chương mọi câu đã cắt bỏ).
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/ReadingMode.vue — .chapter-note của mỗi Chương nói đúng vì sao nó rỗng', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('Chương 1 rỗng thật (segment_count=0), Chương 2 mọi câu đã cắt bỏ (segment_count>0) — hai câu KHÁC nhau, đúng chỗ của nó', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [
        chapterFixture(1, 1, [], { segment_count: 0 }),
        chapterFixture(2, 2, [], { segment_count: 4 }),
      ],
      frontier: endOfWorkFrontier(),
    })
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const notes = wrapper.findAll('.chapter-note')
    expect(notes).toHaveLength(2)
    expect(notes[0]?.text()).toBe(t('mode.reading.chapter_empty'))
    expect(notes[1]?.text()).toBe(t('mode.reading.chapter_all_omitted'))
    expect(notes[0]?.text()).not.toBe(notes[1]?.text())
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// BẢN VÁ 3 — nút "Dịch tiếp" nằm trong thứ tự Tab TỰ NHIÊN (NFR17: "bằng chuột HOẶC bằng
// Tab + Enter"). Mệnh đề kiểm được ở tầng component là HÌNH DẠNG phần tử, không một lượt
// Tab mô phỏng trong `happy-dom` (không phải engine thật).
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/ReadingMode.vue — nút "Dịch tiếp" là một <button> thật trong thứ tự Tab', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('là <button type="button">, không disabled, không tabindex="-1"', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [chapterFixture(1, 1, [[{ id: 1, source_text: 'a', target_text: 'b' }]])],
      frontier: nextNotDoneFrontier(),
    })
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const btn = wrapper.find('[data-reading-frontier-continue]')
    expect(btn.exists()).toBe(true)
    expect(btn.element.tagName).toBe('BUTTON')
    expect(btn.attributes('type')).toBe('button')
    expect(btn.attributes('disabled')).toBeUndefined()
    expect(btn.attributes('tabindex')).not.toBe('-1')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// BẢN VÁ 6 — dãy ĐÚNG MỘT Chương (ca THƯỜNG NHẤT hôm nay) dùng câu RIÊNG cho
// `.frontier-range`, không "từ Chương 1 đến Chương 1".
// ═════════════════════════════════════════════════════════════════════════════════

describe('modes/ReadingMode.vue — .frontier-range phân biệt dãy MỘT Chương với dãy NHIỀU Chương', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('dãy đúng MỘT Chương ⇒ khoá `frontier_range_single`, không lặp tên Chương hai lần', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [chapterFixture(1, 1, [[{ id: 1, source_text: 'a', target_text: 'b' }]])],
      frontier: endOfWorkFrontier(),
    })
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const range = wrapper.find('.frontier-range')
    const chapterOneLabel = t('mode.reading.chapter_untitled', { ord: '1' })
    expect(range.text()).toBe(t('mode.reading.frontier_range_single', { ten: chapterOneLabel }))
    expect(range.text()).not.toBe(t('mode.reading.frontier_range', { tu: chapterOneLabel, den: chapterOneLabel }))
  })

  it('dãy HAI Chương ⇒ khoá `frontier_range`, "từ Chương 1 đến Chương 2"', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [
        chapterFixture(1, 1, [[{ id: 1, source_text: 'a', target_text: 'b' }]]),
        chapterFixture(2, 2, [[{ id: 2, source_text: 'c', target_text: 'd' }]]),
      ],
      frontier: endOfWorkFrontier(),
    })
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const range = wrapper.find('.frontier-range')
    expect(range.text()).toBe(
      t('mode.reading.frontier_range', {
        tu: t('mode.reading.chapter_untitled', { ord: '1' }),
        den: t('mode.reading.chapter_untitled', { ord: '2' }),
      }),
    )
  })
})
