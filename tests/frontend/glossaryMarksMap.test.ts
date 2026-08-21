/**
 * Story 3.4b · FR50/FR51 — **chia mark tuyệt đối về từng segment**, hàm THUẦN của
 * `src/panels/glossaryMarksMap.ts`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 ĐÂY LÀ CỔNG TỰ ĐỘNG DUY NHẤT CANH PHÉP CỘNG DỒN `\n`-JOIN — ĐỌC TRƯỚC KHI SỬA
 * ═════════════════════════════════════════════════════════════════════════════════
 * `glossaryMarksBySegment` là chỗ DUY NHẤT trong kho phép cộng dồn `segment.source_text` nối
 * bằng `\n` tồn tại (`3-4b-…md` §Design Notes). Một lượt sửa sai ở đây không ném lỗi, không
 * làm cổng nào khác đỏ — nó vẽ dấu Glossary lên SAI ký tự, im lặng, đúng lớp lỗi mà
 * `editorOriginalOffsets.test.ts` đã ghi cho phép ánh xạ chị em của nó.
 *
 * ⚠️ Ca *"một mark BẮC CẦU qua `\n`"* dưới đây kiểm **hành vi của hàm THUẦN này** trên MỘT
 * input bất kỳ (nó phải cắt, không tràn, không panic) — nó KHÔNG chứng minh rằng
 * `marks_for_source_text` phía Rust không BAO GIỜ sinh ra một mark như vậy trên dữ liệu thật.
 * Mệnh đề đó là một AC riêng của story, đo trên Rust thật (`3-4b-…md` §Verification).
 */
import { describe, expect, it } from 'vitest'
import { glossaryMarksBySegment, joinSegmentSourceText } from '../../src/panels/glossaryMarksMap'
import type { GlossarySegmentSource } from '../../src/panels/glossaryMarksMap'
import type { GlossaryMark } from '../../src/config/glossary'

function mark(
  start: number,
  end: number,
  overrides: Partial<GlossaryMark> = {},
): GlossaryMark {
  return {
    start,
    end,
    tier: 'global',
    is_confirmed: true,
    translation: 'bản dịch',
    ...overrides,
  }
}

describe('joinSegmentSourceText — nối segment.source_text bằng `\\n`', () => {
  it('nối ĐÚNG một `\\n` giữa mỗi cặp segment liền kề, kể cả segment rỗng', () => {
    const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '中國' }, { id: 2, source_text: '' }, { id: 3, source_text: '人' }]
    expect(joinSegmentSourceText(segs)).toBe('中國\n\n人')
  })

  it('một segment duy nhất ⇒ không `\\n` nào được thêm', () => {
    expect(joinSegmentSourceText([{ id: 1, source_text: '中國' }])).toBe('中國')
  })

  it('mảng rỗng ⇒ chuỗi rỗng', () => {
    expect(joinSegmentSourceText([])).toBe('')
  })
})

describe('glossaryMarksBySegment — chia mark tuyệt đối về từng segment', () => {
  it('Chương không có thuật ngữ nào ⇒ Map rỗng, 0 mảnh bị cắt thêm ở BẤT KỲ segment nào', () => {
    const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '中國人' }, { id: 2, source_text: '很好' }]
    const result = glossaryMarksBySegment(segs, [])
    expect(result.size).toBe(0)
    expect(result.get(1)).toBeUndefined()
    expect(result.get(2)).toBeUndefined()
  })

  it('mark nằm gọn trong segment ĐẦU — offset cục bộ bằng đúng offset tuyệt đối', () => {
    // '中國人' (0-2) + '\n'(3) + '很好' (4-5). Mark '中國' = [0, 2).
    const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '中國人' }, { id: 2, source_text: '很好' }]
    const result = glossaryMarksBySegment(segs, [mark(0, 2)])

    expect(result.get(1)).toEqual({ boundaries: [2], spans: [{ start: 0, end: 2, isConfirmed: true, translation: 'bản dịch' }] })
    expect(result.get(2)).toBeUndefined()
  })

  it('CỘNG DỒN offset qua nhiều segment — không phải một phép `slice` trần trên chuỗi Chương', () => {
    // '中國人'(0-2) '\n'(3) '很好'(4-5). Mark '很好' = [4, 6) tuyệt đối ⇒ [0, 2) cục bộ ở segment 2.
    const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '中國人' }, { id: 2, source_text: '很好' }]
    const result = glossaryMarksBySegment(segs, [mark(4, 6)])

    expect(result.get(1)).toBeUndefined()
    expect(result.get(2)).toEqual({ boundaries: [], spans: [{ start: 0, end: 2, isConfirmed: true, translation: 'bản dịch' }] })
  })

  it('thuật ngữ phủ MỘT PHẦN một segment ⇒ đúng MỘT biên cắt cục bộ, không phủ trọn', () => {
    // '文化的' (0-2). Mark '文' = [0, 1) — chỉ ký tự đầu.
    const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '文化的' }]
    const result = glossaryMarksBySegment(segs, [mark(0, 1, { is_confirmed: false, translation: null })])

    expect(result.get(1)).toEqual({
      boundaries: [1],
      spans: [{ start: 0, end: 1, isConfirmed: false, translation: null }],
    })
  })

  it('hai mark KHÔNG liền nhau trong CÙNG một segment ⇒ hai span, bốn biên gộp lại còn 4 điểm', () => {
    // '甲乙丙丁' (0-3). Mark1 '甲' [0,1); Mark2 '丙丁' [2,4).
    const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '甲乙丙丁' }]
    const result = glossaryMarksBySegment(segs, [mark(0, 1), mark(2, 4)])

    const entry = result.get(1)
    expect(entry?.spans).toEqual([
      { start: 0, end: 1, isConfirmed: true, translation: 'bản dịch' },
      { start: 2, end: 4, isConfirmed: true, translation: 'bản dịch' },
    ])
    // biên [1] của mark1 (end < len) cộng [2, 4)→ start=2 (>0) — end=4 CHẠM đúng cuối segment
    // (len=4) nên KHÔNG sinh biên ở đó (không có gì để cắt sau ký tự cuối).
    expect(entry?.boundaries).toEqual([1, 2])
  })

  it('mark BẮC CẦU qua chất nối `\\n` bị CẮT thành hai span cục bộ, một ở MỖI segment', () => {
    // '中國人'(0-2) '\n'(3) '很好'(4-5). Mark tuyệt đối [2, 5) = '人' + '\n' + '很' — bắc cầu.
    const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '中國人' }, { id: 2, source_text: '很好' }]
    const result = glossaryMarksBySegment(segs, [mark(2, 5)])

    // Segment 1 chỉ nhận phần TRƯỚC `\n`: '人' cục bộ [2, 3).
    expect(result.get(1)).toEqual({ boundaries: [2], spans: [{ start: 2, end: 3, isConfirmed: true, translation: 'bản dịch' }] })
    // Segment 2 chỉ nhận phần SAU `\n`: '很' cục bộ [0, 1) — KHÔNG mang ký tự `\n` nào.
    expect(result.get(2)).toEqual({ boundaries: [1], spans: [{ start: 0, end: 1, isConfirmed: true, translation: 'bản dịch' }] })
  })

  it('mark PHỦ ĐÚNG ký tự phân tách `\\n` ⇒ không chạm ký tự NGUỒN nào của segment nào cả', () => {
    // '中國'(0-1) '\n'(2) '人'(3). Mark [2, 3) phủ ĐÚNG `\n` — segment 1 kết thúc ở offset 2
    // (không bao gồm), segment 2 bắt đầu ở offset 3, nên `\n` không thuộc segment nào.
    const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '中國' }, { id: 2, source_text: '人' }]
    const result = glossaryMarksBySegment(segs, [mark(2, 3)])

    expect(result.size).toBe(0)
  })

  it('chờ chốt (`is_confirmed=false`, `translation=null`) đi qua NGUYÊN VẸN, không bị chuẩn hoá', () => {
    const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '甲乙' }]
    const result = glossaryMarksBySegment(segs, [mark(0, 2, { is_confirmed: false, translation: null, tier: 'work' })])

    expect(result.get(1)?.spans[0]).toEqual({ start: 0, end: 2, isConfirmed: false, translation: null })
  })

  it('ký tự NGOÀI BMP trong một segment TRƯỚC không làm lệch offset cục bộ của segment SAU', () => {
    // 𠀀 = U+20000 (CJK Extension B) — MỘT điểm mã, HAI đơn vị mã UTF-16.
    const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '𠀀國' }, { id: 2, source_text: '很好' }]
    // Segment 1 dài ĐÚNG 2 điểm mã ('𠀀', '國'); segment 2 bắt đầu ở tuyệt đối 3 (2 + 1 cho `\n`).
    const result = glossaryMarksBySegment(segs, [mark(3, 5)])

    expect(result.get(1)).toBeUndefined()
    expect(result.get(2)).toEqual({ boundaries: [], spans: [{ start: 0, end: 2, isConfirmed: true, translation: 'bản dịch' }] })
  })

  it('segment KHÔNG có mark nào ⇒ không có mục trong Map (thưa, không object rỗng)', () => {
    const segs: GlossarySegmentSource[] = [{ id: 1, source_text: '甲' }, { id: 2, source_text: '乙' }, { id: 3, source_text: '丙' }]
    const result = glossaryMarksBySegment(segs, [mark(4, 5)]) // chỉ phủ segment 3 ('丙' ở tuyệt đối 4)

    expect(result.has(1)).toBe(false)
    expect(result.has(2)).toBe(false)
    expect(result.get(3)?.spans).toHaveLength(1)
  })
})
