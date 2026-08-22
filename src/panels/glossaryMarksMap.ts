/**
 * 🔴 **CHIA MARK TUYỆT ĐỐI VỀ TỪNG SEGMENT** — Story 3.4b, FR50/FR51.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO CHẤT NỐI LÀ `\n`, VÀ VÌ SAO ĐÂY LÀ MỘT ỨNG VIÊN CHỨ CHƯA PHẢI KẾT LUẬN
 * ─────────────────────────────────────────────────────────────────────────────
 * `ChapterSegment` (`src/config/segment.ts`) không mang một trường offset cấp Chương nào, nên
 * đường nối DUY NHẤT còn lại để dựng một chuỗi có phép cộng dồn NGHỊCH được (offset tuyệt đối
 * → `(segmentId, offset cục bộ)`) là nối `segment.source_text` bằng một ký tự phân tách.
 *
 * `\n` được chọn vì CẢ HAI nhánh khớp của `core::matching` đã có lý do từ chối bắc cầu qua nó
 * TỪ TRƯỚC story này: nhánh `En` từ chối tường minh một dãy bắc cầu qua `.`/`!`/`?`/`\n`; nhánh
 * `Zh` phụ thuộc `jieba-rs` cắt `\n` thành một token riêng. Nhưng đây là một GIẢ ĐỊNH có lý do,
 * KHÔNG một số đo — `tests/frontend/glossaryMarksMap.test.ts` §"không dấu bắc cầu qua chất nối"
 * là AC đo lại đúng mệnh đề này trên dữ liệu thật (`3-4b-…md` §Verification).
 *
 * ⚠️ **KHÔNG nối bằng `chapter.source_text` thô.** `core/segment/import.rs::push_segment`
 * `trim()` mỗi câu và bỏ câu rỗng, `skip_gap` nuốt trọn khe trắng giữa hai câu — chuỗi thô
 * KHÔNG cho một phép cộng dồn nghịch được sau khi đã tách segment, và càng không sau một lượt
 * gộp/tách (Story 2.8). Chuỗi phải GỬI CHO Rust (`glossaryMarksForChapter`, `config/glossary.ts`)
 * và chuỗi DÙNG ĐỂ chia mark ở đây phải là ĐÚNG MỘT chuỗi — xem `glossaryMarksState.ts`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 BA TẬP ĐIỂM TRÊN CÙNG MỘT TRỤC, ĐỪNG GỘP (xem `3-4b-…md` §Design Notes)
 * ═════════════════════════════════════════════════════════════════════════════════
 * ```text
 * props.cuts        -> điểm ngắt đoạn của người dùng   -> CẮT + vẽ `cut-here`
 * termBoundaries     -> biên thuật ngữ Glossary          -> CẮT, không vẽ gì
 * srcStart của node  -> neo ánh xạ ngược về source_text  -> KHÔNG cắt, chỉ báo cáo
 * ```
 * Hàm này chỉ sinh ra tập THỨ HAI (`boundaries`) — nó không biết gì về `pendingCuts`, và chỗ
 * gọi (`GridPanel.vue`) là nơi HAI tập được giữ RIÊNG cho tới lúc hợp lại thành điểm cắt DOM.
 *
 * ⚠️ **Hàm THUẦN** — không `import` Vue, không DOM, test được bằng dữ liệu bịa
 * (`tests/frontend/glossaryMarksMap.test.ts`).
 */
import type { GlossaryMark, GlossaryTierWire } from '../config/glossary'

/** Đúng bốn trường mà [`glossaryMarksBySegment`] cần từ một hàng segment. */
export type GlossarySegmentSource = { id: number; source_text: string }

/**
 * Một mark [`GlossaryMark`] đã CẮT (clip) về hệ toạ độ CỤC BỘ của một segment — điểm mã, tính
 * từ đầu `segment.source_text` của CHÍNH nó.
 */
export type SegmentTermSpan = {
  /** Điểm mã bắt đầu (bao gồm), cục bộ trong `segment.source_text`. */
  start: number
  /** Điểm mã kết thúc (không bao gồm), cục bộ trong `segment.source_text`. */
  end: number
  isConfirmed: boolean
  translation: string | null
  /**
   * 🔵 THÊM 2026-08-22 (Story 3.6) — `glossary_entry.id`, chép thẳng từ `GlossaryMark.id`.
   * Dải "Chờ chốt" đọc trường này để biết hỏi/ghi ĐÚNG hàng nào; cắt mark về toạ độ cục bộ
   * không được đánh rơi khoá ghi.
   */
  id: number
  /**
   * 🔵 THÊM 2026-08-22 (Story 3.6) — khoá ghi thật, chép thẳng từ `GlossaryMark.source_term`.
   * Có thể KHÁC bề mặt đã khớp trên màn hình (nhánh tiếng Anh khớp theo hình thái).
   */
  sourceTerm: string
  /**
   * 🔵 THÊM 2026-08-22 (Story 3.6) — chép thẳng từ `GlossaryMark.tier`. Dải "Chờ chốt" cần
   * biết tầng để gọi `confirmPendingGlossaryTranslation(tier, id, ..)` — ghi vào ĐÚNG kho.
   */
  tier: GlossaryTierWire
}

/** Dấu thuật ngữ của MỘT segment — biên cắt (không nhãn) cộng span (có nhãn). */
export type SegmentGlossaryMarks = {
  /**
   * Điểm mã CỤC BỘ nơi phải cắt thêm vì một biên thuật ngữ đi qua — `0 < x < length`, đã sắp
   * tăng dần, KHÔNG trùng lặp. Đây LÀ tập `termBoundaries` mà `buildSegments`
   * (`SourceHanViet.vue`) và `sourcePiecesOf`/`sourcePieceStartsOf` (`GridPanel.vue`) phải hợp
   * (KHÔNG gộp) với `pendingCuts`.
   */
  boundaries: readonly number[]
  /**
   * Span đã cắt, sắp theo `start` tăng dần — nguồn nhãn (đã chốt/chờ chốt) cho việc tô màu.
   *
   * 🔴 **KHÔNG CHỒNG LẤN — một bảo đảm ĐI VÀO từ Rust, không phải một mệnh đề tự thân của
   * mảng này.** `marks_for_source_text` (`core/glossary/store.rs:722`, `resolve_overlaps`,
   * gọi ở `:810`) đã phân xử mọi lượt chồng nhau TRƯỚC khi lên dây — dài nhất thắng, hoà thì
   * trái nhất — nên `marks` mà hàm này nhận vào ĐÃ rời rạc. `glossarySpanAt`/`glossarySpanAtPoint`
   * (`GridPanel.vue`) và `glossarySpanFor` (`SourceHanViet.vue`) đều tra span bằng
   * `Array.prototype.find()` — hàm đó chỉ đúng nếu ĐÚNG NHIỀU NHẤT MỘT span thoả điều kiện.
   * Nếu một nguồn mark thứ hai (không đi qua `marks_for_source_text`) từng được thêm vào đây
   * mà KHÔNG mang cùng bảo đảm đó, `.find()` sẽ lặng lẽ trả span ĐẦU TIÊN khớp và bỏ sót các
   * span chồng lấn còn lại — không lỗi nào ném, dấu chỉ "biến mất" không rõ vì sao.
   */
  spans: readonly SegmentTermSpan[]
}

/** `SegmentGlossaryMarks` rỗng dùng chung — tránh một object mới mỗi lượt tra không tìm thấy. */
export const EMPTY_SEGMENT_GLOSSARY_MARKS: SegmentGlossaryMarks = { boundaries: [], spans: [] }

/**
 * Chia `marks` (offset TUYỆT ĐỐI vào chuỗi `segments` nối bằng `\n`) về từng segment.
 *
 * 🔴 **Đếm bằng ĐIỂM MÃ (`Array.from`/`[...str]`), không `string.length` (UTF-16)** — cùng đơn
 * vị mà `marks_for_source_text` phía Rust dùng, và cùng đơn vị mà `sourcePieceStartsOf`/
 * `sourceCutOffsetOf` của lưới đã dùng từ Story 2.9. Một ký tự ngoài BMP (CJK Extension B) đếm
 * SAI ở đây làm mọi segment SAU nó lệch offset — im lặng, không cổng nào bắt được ngoài test
 * kèm ký tự astral tường minh.
 *
 * ⚠️ **Không lọc `marks.length === 0` sớm rồi trả một Map rỗng**: chỗ gọi (`GridPanel.vue`)
 * tra `Map.get(id)` cho MỌI `id`, và một Map rỗng cho `.get()` trả `undefined` giống hệt một
 * Map ĐẦY nhưng segment đó không có mark nào — cả hai đường đều an toàn nhờ
 * [`EMPTY_SEGMENT_GLOSSARY_MARKS`] ở phía gọi, nên hàm này không cần tự dựng đủ N mục cho N
 * segment khi `marks` rỗng — bỏ qua bước đó là một phép tối ưu ĐÚNG, không một lượt thiếu sót.
 */
export function glossaryMarksBySegment(
  segments: readonly GlossarySegmentSource[],
  marks: readonly GlossaryMark[],
): ReadonlyMap<number, SegmentGlossaryMarks> {
  const result = new Map<number, SegmentGlossaryMarks>()
  if (marks.length === 0 || segments.length === 0) return result

  let offset = 0
  for (const seg of segments) {
    const codepoints = Array.from(seg.source_text)
    const len = codepoints.length
    const segStart = offset
    const segEnd = offset + len
    // 🔴 `\n` phân tách — cộng SAU khi tính `segEnd`, TRƯỚC lượt segment kế tiếp. Đây LÀ chỗ
    // duy nhất trong kho phép cộng dồn `\n`-join này tồn tại; xem doc-comment đầu tệp.
    offset = segEnd + 1

    const spans: SegmentTermSpan[] = []
    const boundarySet = new Set<number>()
    for (const mark of marks) {
      // Từ chối nhanh: mark không chạm khoảng [segStart, segEnd) của segment này.
      if (mark.end <= segStart || mark.start >= segEnd) continue

      const localStart = Math.max(mark.start, segStart) - segStart
      const localEnd = Math.min(mark.end, segEnd) - segStart
      if (localEnd <= localStart) continue // biên chạm nhau, không phủ ký tự nào — bỏ qua

      spans.push({
        start: localStart,
        end: localEnd,
        isConfirmed: mark.is_confirmed,
        translation: mark.translation,
        id: mark.id,
        sourceTerm: mark.source_term,
        tier: mark.tier,
      })
      if (localStart > 0) boundarySet.add(localStart)
      if (localEnd < len) boundarySet.add(localEnd)
    }

    if (spans.length === 0) continue // giữ Map thưa — segment này dùng [`EMPTY_SEGMENT_GLOSSARY_MARKS`]

    spans.sort((a, b) => a.start - b.start)
    result.set(seg.id, {
      boundaries: Array.from(boundarySet).sort((a, b) => a - b),
      spans,
    })
  }
  return result
}

/**
 * Nối `segments` bằng `\n` — ĐÚNG chuỗi mà [`glossaryMarksBySegment`] giả định khi chia mark
 * tuyệt đối về từng segment, và ĐÚNG chuỗi phải gửi cho `glossaryMarksForChapter`
 * (`src/config/glossary.ts`). Một chỗ, để hai đầu (chuỗi GỬI ĐI, chuỗi DÙNG ĐỂ chia) không
 * bao giờ lệch nhau — xem `glossaryMarksState.ts`.
 */
export function joinSegmentSourceText(segments: readonly GlossarySegmentSource[]): string {
  return segments.map((s) => s.source_text).join('\n')
}
