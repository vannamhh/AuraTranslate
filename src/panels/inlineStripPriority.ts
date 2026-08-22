/**
 * Sổ ưu tiên cho DẢI NỘI TUYẾN ở chân Workspace — Story 3.6.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO MỘT SỔ CÓ TÊN, KHÔNG PHẢI MỘT `v-if` TỰ DO Ở MỖI COMPONENT
 * ─────────────────────────────────────────────────────────────────────────────
 * `GlossaryQuickAdd` sống ở CÙNG slot DOM (`App.vue`, ngay trên `<StatusBar />`) và mở được
 * từ bất kỳ bề mặt nào bằng `Mod+Alt+G` (`glossaryQuickAddState.ts:210-217`). Không có một
 * sổ chung, hai dải có thể cùng đủ điều kiện render một lúc và người dùng thấy hai ô nhập
 * chồng lên nhau — một va chạm HÔM NAY, không phải một dự phòng cho FR83/FR59. `EXPERIENCE.md`
 * (§75-81) chốt thứ tự ba dải TỰ ĐỘNG (`proofreader`/`tm_fuzzy`/…); `glossary_quick_add` đứng
 * trên cả ba vì nó là thao tác người dùng VỪA yêu cầu — một dải người dùng cố ý mở không bao
 * giờ nên bị một dải TỰ ĐỘNG che mất.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 DANH MỤC ĐÓNG — bốn loại, hai mục CHƯA CÓ MÃ
 * ─────────────────────────────────────────────────────────────────────────────
 * `proofreader` (FR83) và `tm_fuzzy` (FR59) không có component nào dựng hôm nay — chúng có
 * mặt ở đây CHỈ để khoá thứ tự bằng máy trước khi mã của chúng tồn tại, đúng lý do cổng này
 * đứng sẵn TRƯỚC khi Epic 4/7 có một dòng mã (cùng tinh thần `glossary_boundary.rs` đứng sẵn
 * trước Epic 4). Đăng ký một loại KHÔNG có nghĩa nó render — component tương lai tự quyết
 * định khi nào nó "đủ điều kiện" (`eligible`); module này chỉ trả lời "trong số các loại ĐANG
 * đủ điều kiện, loại nào thắng".
 *
 * Module THUẦN — 0 `import` từ `vue`/`dockview`, test được bằng dữ liệu bịa
 * (`tests/frontend/inlineStripPriority.test.ts`).
 */

/** Bốn loại dải nội tuyến — danh mục ĐÓNG. Thêm một loại mới là một quyết định phải viết ra. */
export type InlineStripKind = 'glossary_quick_add' | 'glossary_confirm' | 'proofreader' | 'tm_fuzzy'

/**
 * Thứ tự ưu tiên — số NHỎ HƠN thắng. `glossary_quick_add` (0) là thao tác người dùng VỪA yêu
 * cầu nên đứng trên hết; `glossary_confirm` (1) là dải mọc chốt lần đầu gặp (Story 3.6);
 * `proofreader` (2, FR83) và `tm_fuzzy` (3, FR59) là hai dải TỰ ĐỘNG chưa có mã, giữ chỗ theo
 * `EXPERIENCE.md:75-81`.
 */
const PRIORITY_ORDER: readonly InlineStripKind[] = [
  'glossary_quick_add',
  'glossary_confirm',
  'proofreader',
  'tm_fuzzy',
]

/**
 * Trong số các loại `eligible` (đang ĐỦ ĐIỀU KIỆN render), trả về loại THẮNG — hoặc `null`
 * nếu `eligible` rỗng. Hàm THUẦN: không đọc state nào, chỉ so `eligible` với [`PRIORITY_ORDER`].
 */
export function topmostStrip(eligible: readonly InlineStripKind[]): InlineStripKind | null {
  for (const kind of PRIORITY_ORDER) {
    if (eligible.includes(kind)) return kind
  }
  return null
}
