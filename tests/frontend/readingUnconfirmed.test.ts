/**
 * `ReadingMode.vue` — gạch chấm nhẹ `.unconfirmed` cho câu CHƯA xác nhận. Story 5.12, AC6.
 *
 * 🔴 AC6 nói thẳng: màn hình KHÔNG được nói dối về trạng thái công việc — và cách rẻ nhất
 * để nó nói dối là một `is_confirmed` THIẾU đọc thành *falsy* rồi rơi vào nhánh "đã xác
 * nhận". Ca cuối của tệp này là đối chứng ĐỎ bắt buộc #3 của story: gỡ `is_confirmed` khỏi
 * `isReadingSegment` (mô phỏng ở đây bằng cách gỡ trường khỏi FIXTURE, không khỏi mã sản
 * phẩm) phải làm adapter TỪ CHỐI cả `run`, không âm thầm vẽ mọi câu như đã hoàn chỉnh.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

type SegmentFixture = { id: number; source_text: string; target_text: string; is_confirmed?: boolean }

function chapterFixture(chapterId: number, ord: number, paragraphs: SegmentFixture[][]) {
  return {
    chapter_id: chapterId,
    chapter_ord: ord,
    chapter_title: `Chuong ${ord}`,
    paragraphs: paragraphs.map((segments) => ({ segments })),
    segment_count: paragraphs.reduce((sum, p) => sum + p.length, 0),
  }
}

beforeEach(async () => {
  mockInvoke.mockReset()
  const state = await import('../../src/modes/readingState')
  state.resetReading()
  state.resetReadingToc()
})

describe('modes/ReadingMode.vue — câu chưa xác nhận mang lớp `unconfirmed`', () => {
  let wrapper: ReturnType<typeof mount> | null = null

  afterEach(() => {
    wrapper?.unmount()
    wrapper = null
  })

  it('dãy HAI Chương: đúng những câu `is_confirmed = false` mang lớp, các câu còn lại thì không', async () => {
    // Chương 1: câu 1 đã ký, câu 2 CHƯA. Chương 2: câu 3 CHƯA ký, câu 4 đã ký. Ba câu chưa
    // xác nhận nằm rải giữa hai Chương — không phải một khối liền để loại trừ khả năng
    // phép đếm chỉ đúng vì tình cờ liền kề.
    mockInvoke.mockResolvedValueOnce({
      chapters: [
        chapterFixture(1, 1, [
          [
            { id: 1, source_text: 'Mot.', target_text: 'Cau mot.', is_confirmed: true },
            { id: 2, source_text: 'Hai.', target_text: 'Cau hai.', is_confirmed: false },
          ],
        ]),
        chapterFixture(2, 2, [
          [
            { id: 3, source_text: 'Ba.', target_text: 'Cau ba.', is_confirmed: false },
            { id: 4, source_text: 'Bon.', target_text: 'Cau bon.', is_confirmed: true },
          ],
        ]),
      ],
      frontier: { kind: 'end-of-work', chapter: null },
    })

    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const spans = wrapper.findAll('.column .paragraph span')
    expect(spans).toHaveLength(4)

    const unconfirmedTexts = spans.filter((s) => s.classes('unconfirmed')).map((s) => s.text())
    const confirmedTexts = spans.filter((s) => !s.classes('unconfirmed')).map((s) => s.text())

    // Đúng số câu chưa xác nhận trong fixture (hai), không hơn không kém.
    expect(unconfirmedTexts).toHaveLength(2)
    expect(unconfirmedTexts.every((t) => t.includes('Cau hai') || t.includes('Cau ba'))).toBe(true)
    expect(confirmedTexts).toHaveLength(2)
    expect(confirmedTexts.every((t) => !t.includes('Cau hai') && !t.includes('Cau ba'))).toBe(true)
  })

  it('mọi câu đã xác nhận ⇒ KHÔNG span nào mang lớp `unconfirmed`', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [
        chapterFixture(1, 1, [[{ id: 1, source_text: 'a', target_text: 'Cau mot.', is_confirmed: true }]]),
      ],
      frontier: { kind: 'end-of-work', chapter: null },
    })
    const { default: ReadingMode } = await import('../../src/modes/ReadingMode.vue')
    const state = await import('../../src/modes/readingState')
    wrapper = mount(ReadingMode)
    await state.ensureReadingLoaded()
    await wrapper.vm.$nextTick()

    const spans = wrapper.findAll('.column .paragraph span')
    expect(spans).toHaveLength(1)
    expect(spans.filter((s) => s.classes('unconfirmed'))).toHaveLength(0)
  })
})

describe('config/reading.ts::readReadingRun — `is_confirmed` THIẾU làm adapter từ chối cả run', () => {
  /**
   * 🔴 Đối chứng ĐỎ bắt buộc #3 của story (`§Verification`). `is_confirmed` vắng mặt (một
   * hình dạng dây trượt khỏi một lượt đổi kiểu phía Rust, hoặc một fixture soạn sai) KHÔNG
   * được lọt qua thành `undefined` rồi hiển thị như đã xác nhận — nó phải làm cả `run` bị
   * TỪ CHỐI, để lỗi lộ ra ngay ở màn hình trạng thái thay vì im lặng vẽ sai.
   */
  it('gỡ `is_confirmed` khỏi một segment ⇒ `{ run: null, error: ipc.unknown }`, không ném', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [
        {
          chapter_id: 1,
          chapter_ord: 1,
          chapter_title: null,
          // Segment KHÔNG mang `is_confirmed` -- mô phỏng một hình dạng dây đã trượt.
          paragraphs: [{ segments: [{ id: 1, source_text: 'a', target_text: 'b' }] }],
          segment_count: 1,
        },
      ],
      frontier: { kind: 'end-of-work', chapter: null },
    })

    const { readReadingRun } = await import('../../src/config/reading')
    const result = await readReadingRun()

    expect(result.run).toBeNull()
    expect(result.error?.code).toBe('ipc.unknown')
  })

  it('cùng fixture trượt đó đi qua `ensureReadingLoaded()` ⇒ `readingStatusKind` là "error", KHÔNG "content"', async () => {
    mockInvoke.mockResolvedValueOnce({
      chapters: [
        {
          chapter_id: 1,
          chapter_ord: 1,
          chapter_title: null,
          paragraphs: [{ segments: [{ id: 1, source_text: 'a', target_text: 'b' }] }],
          segment_count: 1,
        },
      ],
      frontier: { kind: 'end-of-work', chapter: null },
    })

    const state = await import('../../src/modes/readingState')
    await state.ensureReadingLoaded()

    expect(state.readingStatusKind.value).toBe('error')
    expect(state.readingRun.value).toBeNull()
  })
})
