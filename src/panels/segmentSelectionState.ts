/**
 * State của **vùng chọn nhiều segment** trong lưới — Story 4.9, Quyết định #1 (Ice ký
 * 2026-09-21): *"batch input là một range selection THẬT trong lưới, không một 'N câu kế
 * tiếp' suy từ caret"*. Đây là mô hình chọn nhiều đầu tiên của kho (đo hai lần, độc lập —
 * §Code Map của spec 4-9): `editorCaretSegmentId` là một `number | null` DUY NHẤT,
 * `selectionContract.ts` chọn CHỮ trong một ô chứ không CHỌN HÀNG, và `readingState.ts`'s
 * `anchorSegmentId` là một neo cuộn của Chế độ đọc, không một vùng chọn.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MÔ HÌNH: NEO (`anchor`) + TIÊU ĐIỂM (`focus`) — cùng khuôn `Selection` của DOM
 * ─────────────────────────────────────────────────────────────────────────────
 * `anchor` đứng yên tại chỗ hợp âm mở rộng ĐẦU TIÊN bắt đầu; `focus` là đầu di động. Vùng
 * chọn là khoảng `[min(anchor, focus), max(anchor, focus)]` theo CHỈ SỐ trong
 * `editorSegments` (tức theo `ord`, thứ tự Rust quyết — §Always spec 4.9: "dịch theo THỨ TỰ
 * TÀI LIỆU"). Bấm hợp âm ngược hướng THU HẸP vùng chọn thay vì mở một vùng thứ hai — đúng
 * hành vi `Shift+↓`/`Shift+↑` của mọi trình soạn thảo văn bản.
 *
 * ⚠️ **CARET KHÔNG BỊ ĐỤNG.** `editorCaretSegmentId` chỉ được ĐỌC ở đây, đúng một lần, để
 * gieo `anchor` khi vùng chọn CHƯA tồn tại — không một lượt gán ngược. Quyết định #3 của
 * spec (Ice ký 2026-09-21) đã chốt: FR20 (hai panel cùng một câu) đã đóng nhờ CARET, không
 * nhờ kích thước vùng chọn — đụng caret ở đây sẽ phá đúng bất biến đó, và không nằm trong
 * phạm vi Phase 1 (§Tasks & Acceptance chỉ liệt tệp này, `editorSegments.ts`,
 * `GridPanel.vue`, `commands/index.ts`, hai cụm reset Tác phẩm, và tệp test — không đụng
 * `editorPanelState.ts::setEditorCaret`).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO KHÔNG CẦN NỐI VÀO `resetEditorPanel()`/lượt đổi CHƯƠNG — TỰ LÀNH, ĐO ĐƯỢC
 * ─────────────────────────────────────────────────────────────────────────────
 * [`segmentSelectionIds`] tra `anchor`/`focus` bằng `indexOf` trên **`editorSegments` HIỆN
 * TẠI** ở mỗi lượt tính lại — không một bản chép cache. `segment.id` là
 * `INTEGER PRIMARY KEY AUTOINCREMENT` trong CHÍNH `project.db` của Tác phẩm đang mở
 * (`schema.rs:337`) và đếm chung cho MỌI Chương của Tác phẩm đó — nên khi người dùng đổi
 * Chương trong CÙNG một Tác phẩm, hai id cũ hầu như chắc chắn KHÔNG tồn tại trong danh sách
 * segment của Chương mới ⇒ `indexOf` trả `-1` ⇒ vùng chọn tự rỗng, KHÔNG cần một lượt reset
 * tường minh nào ở đường Chương-đổi-trong-cùng-Tác-phẩm.
 *
 * 🔴 **Điều đó KHÔNG đúng qua một lượt đổi TÁC PHẨM** — và đây là lý do hai cụm reset dưới
 * vẫn bắt buộc. Mỗi `project.db` tự đếm `AUTOINCREMENT` từ 1, nên một `anchor`/`focus` id của
 * Tác phẩm CŨ tồn tại THẬT trong Tác phẩm MỚI và trỏ vào một câu khác hẳn — đúng lớp lỗi mà
 * `editorPanelState.ts::resetEditorPanel` đã ghi cho `confirmError`/`caretPlacement`/
 * `sourceCut`. ⇒ [`resetSegmentSelection`] phải nối vào CẢ HAI cụm đổi Tác phẩm
 * (`modes/libraryChapters.ts`, `modes/libraryImport.ts`), đúng luật của
 * `scripts/check-panel-refs.mjs`.
 *
 * ⚠️ Segment đã **cắt bỏ** (`is_omitted`) KHÔNG bị lọc khỏi vùng chọn ở tầng này —
 * `editorSegments` vẫn chở nó (cùng khuôn `ruleById` ở `GridPanel.vue`, đọc TRỌN mảng). Lượt
 * cắt bỏ được xử lý ở tầng BATCH AI (Phase 2, `prepare_batch_call`, §Always: *"segment
 * `is_omitted` bị bỏ TRƯỚC lượt gọi provider"*) — phạm vi Phase 1 là hiển thị/chọn, không phải
 * quyết định câu nào được dịch.
 *
 * 🔵 **SỬA — segment đã VỀ HƯU (`retired_at`) thì KHÔNG có gì để lọc ở tầng này, và đó là một
 * mệnh đề đo được chứ không một giả định.** `read_open_chapter_segments`
 * (`src-tauri/src/commands/segment.rs:908`) lọc `retired_at IS NULL` ngay trong câu SQL, nên
 * `editorSegments` KHÔNG BAO GIỜ chở một hàng đã về hưu — khác đoạn `is_omitted` ngay trên.
 * `:236` của cùng tệp đó ghi thêm: chưa một đường nào trong sản phẩm hôm nay thật sự cho một
 * segment về hưu (Story 2.8/2.9 chỉ khai cột, chưa đường ghi). Nếu một đường về hưu thật ra
 * đời sau này, mệnh đề *"`editorSegments` không chở segment về hưu"* đi theo NGUỒN đó — không
 * phải một điều kiện cần vá thêm ở tệp này.
 */
import { computed, readonly, ref } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import { editorCaretSegmentId, editorSegments } from './editorPanelState'

const anchorSegmentId = ref<number | null>(null)
const focusSegmentId = ref<number | null>(null)

/** Đầu NEO của vùng chọn — đứng yên từ lượt mở rộng đầu tiên. `null` == chưa có vùng chọn. */
export const segmentSelectionAnchorId: DeepReadonly<Ref<number | null>> = readonly(anchorSegmentId)
/** Đầu TIÊU ĐIỂM — đầu di động mà hai lệnh mở rộng dời tới. `null` == chưa có vùng chọn. */
export const segmentSelectionFocusId: DeepReadonly<Ref<number | null>> = readonly(focusSegmentId)

/**
 * `segment.id` đang được chọn, theo THỨ TỰ TÀI LIỆU (`ord`) — §Always spec 4.9. Rỗng khi
 * chưa có vùng chọn, khi Chương vừa đổi dưới một vùng chọn cũ (§tự lành ở đầu tệp), hoặc khi
 * Chương đang mở không còn segment nào.
 *
 * 🔴 Tính từ `editorSegments.value.map(...)` **mỗi lượt gọi**, không cache riêng — mảng đó
 * đã là nguồn sự thật duy nhất cho thứ tự, và một bản chép ở đây là một cơ hội thứ hai để
 * trôi khỏi nó.
 */
export const segmentSelectionIds: ComputedRef<readonly number[]> = computed(() => {
  const anchor = anchorSegmentId.value
  const focus = focusSegmentId.value
  if (anchor === null || focus === null) return []

  const ids = editorSegments.value.map((s) => s.id)
  const anchorIndex = ids.indexOf(anchor)
  const focusIndex = ids.indexOf(focus)
  if (anchorIndex === -1 || focusIndex === -1) return []

  const lo = Math.min(anchorIndex, focusIndex)
  const hi = Math.max(anchorIndex, focusIndex)
  return ids.slice(lo, hi + 1)
})

/** Số segment đang được chọn — thứ Panel AI Translation sẽ đọc để báo đếm (AC2, Phase 3). */
export const segmentSelectionCount: ComputedRef<number> = computed(() => segmentSelectionIds.value.length)

/**
 * Dời đầu TIÊU ĐIỂM một hàng theo `direction` (`1` == xuống, `-1` == lên), gieo `anchor` tại
 * CARET nếu vùng chọn chưa tồn tại. Dùng chung cho cả hai handler mở rộng — hai lệnh chỉ khác
 * nhau ở `direction`, không ở luật.
 *
 * 🔴 **Gieo VÀ dời trong CÙNG một lượt bấm** — đúng câu AC1 của spec: *"bấm hợp âm mở rộng
 * `n` lần thì `n`+1 hàng liên tiếp được chọn"*. Lượt bấm ĐẦU TIÊN vừa đặt `anchor = focus =
 * caret` vừa thử dời `focus` một bước — nếu caret đã ở biên Chương theo đúng hướng đó, bước
 * dời thất bại và vùng chọn dừng lại ở ĐÚNG MỘT hàng (ca *"chọn đúng một"* của §I/O Matrix,
 * đạt được qua bàn phím mà không cần một lệnh riêng).
 *
 * ⚠️ **KHÔNG quay vòng qua biên Chương** — cùng luật `nextSegmentId`/`prevSegmentId` của
 * `segmentNavigation.ts`: hết Chương thì `focus` đứng yên, không nhảy về đầu/cuối.
 */
function moveSegmentSelectionFocus(direction: 1 | -1): void {
  const ids = editorSegments.value.map((s) => s.id)
  if (ids.length === 0) return

  const anchorIndex = anchorSegmentId.value === null ? -1 : ids.indexOf(anchorSegmentId.value)
  let focusIndex = focusSegmentId.value === null ? -1 : ids.indexOf(focusSegmentId.value)

  // Gieo lại từ CARET khi: chưa có vùng chọn, HOẶC `anchor` KHÔNG CÒN tồn tại trong
  // `editorSegments` HIỆN TẠI, HOẶC `focus` thì vậy — ba nhánh, không hai. Bỏ nhánh `anchor`
  // là đúng lỗ mà một lượt GỘP/TÁCH segment (Story 2.8/2.9) mở ra: nó về hưu đúng các hàng cũ
  // rồi chèn hàng MỚI, nên `anchor` có thể biến mất khỏi `editorSegments` trong khi `focus`
  // (một hàng khác) còn sống — hai vế của [`segmentSelectionIds`] đòi CẢ HAI chỉ số hợp lệ,
  // nên chỉ kiểm `focusIndex` để đấy `anchor` chết là hợp âm mở rộng dời `focus` ÂM THẦM trong
  // khi vùng chọn đọc RỖNG mãi mãi — không một pixel nào báo.
  //
  // Cùng lý do, vế thứ hai/ba là chỗ nối với luật "tự lành" ở đầu tệp — một Chương đã đổi dưới
  // một vùng chọn cũ để lại `anchor`/`focus` KHÁC `null` nhưng KHÔNG còn tìm thấy; coi nó như
  // "chưa có vùng chọn" thay vì im lặng không làm gì, để hợp âm mở rộng vẫn hữu dụng ngay sau
  // một lượt đổi Chương.
  if (anchorIndex === -1 || focusIndex === -1) {
    const caret = editorCaretSegmentId.value
    if (caret === null) return
    const caretIndex = ids.indexOf(caret)
    if (caretIndex === -1) return
    anchorSegmentId.value = caret
    focusSegmentId.value = caret
    focusIndex = caretIndex
  }

  const nextIndex = focusIndex + direction
  if (nextIndex < 0 || nextIndex >= ids.length) return
  focusSegmentId.value = ids[nextIndex]
}

/** Mở rộng vùng chọn XUỐNG một hàng. Handler của `segment.selection.extend_down`. */
export function extendSegmentSelectionDown(): void {
  moveSegmentSelectionFocus(1)
}

/** Mở rộng vùng chọn LÊN một hàng. Handler của `segment.selection.extend_up`. */
export function extendSegmentSelectionUp(): void {
  moveSegmentSelectionFocus(-1)
}

/** Xoá vùng chọn — handler của `segment.selection.clear`. Cùng thân với [`resetSegmentSelection`]. */
export function clearSegmentSelection(): void {
  resetSegmentSelection()
}

/**
 * Vứt vùng chọn khi Tác phẩm đang mở bị thay — nối vào CẢ HAI cụm đổi Tác phẩm
 * (`modes/libraryChapters.ts`, `modes/libraryImport.ts`). Xem §Vì sao không cần nối vào
 * lượt đổi CHƯƠNG ở đầu tệp cho lý do phạm vi này DỪNG ở biên Tác phẩm.
 */
export function resetSegmentSelection(): void {
  anchorSegmentId.value = null
  focusSegmentId.value = null
}
