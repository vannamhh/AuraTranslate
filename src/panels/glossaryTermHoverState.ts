/**
 * State "đang rê chuột trên một dấu thuật ngữ Glossary đã đánh dấu" — cầu nối GIỮA hai cột
 * render (`GridPanel.vue` đường chữ trần, `SourceHanViet.vue` đường Hán Việt) VÀ
 * `StatusBar.vue` (nhánh `v-else-if` thứ năm) — Story 3.4b.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO MỘT MODULE RIÊNG, KHÔNG MỘT `ref` CỤC BỘ TRONG `GridPanel.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * `StatusBar.vue` và `GridPanel.vue` là hai component ANH EM (cả hai con trực tiếp của
 * `App.vue`/`WorkspaceMode.vue`), không cha-con — không có `props`/`emit` nào nối được chúng.
 * Mọi thanh trạng thái khác của kho (`editorConfirmNotice`, `editorRegroupNotice`,
 * `editorNavNotice`) đã đi đường này: state module-level, ghi ở một nơi, đọc ở nơi khác — cùng
 * khuôn, tệp này không đúc thêm một cơ chế mới.
 *
 * ⚠️ **HAI đường vào, MỘT state — chuột VÀ bàn phím, KHÔNG `tabindex` mới trên từng dấu.**
 * `setHoveredGlossaryTerm`/`clearHoveredGlossaryTerm` được gọi từ `@mouseenter`/`@mouseleave`
 * của mỗi mảnh (`onGlossaryPieceEnter`/`onGlossaryPieceLeave`, `GridPanel.vue`) **VÀ** từ một
 * listener `selectionchange` đọc caret/vùng chọn hiện có (`onSourceSelectionChange`,
 * `GridPanel.vue`) — Ice bác đề xuất ban đầu (gắn `tabindex="0"` lên MỖI mảnh mang dấu, có thể
 * hàng trăm trong một Chương) đúng lý do đã ghi: chèn thêm hàng trăm tab-stop vào một bề mặt
 * mà hợp đồng vùng chọn (`selectionContract.ts`, AC6/AC11/AC12 của Story 1.16/1.18) đã được đo
 * và ký RẤT cẩn thận là một rủi ro không cân xứng. Đường thay thế: `.hv-switch`/`.hv-parallel`
 * ĐÃ mang `tabindex="0"` từ Story 1.18 (AC11), và `Selection.modify()` di chuyển caret không
 * cần phần tử tự focus được — nên vế bàn phím đạt được bằng **0** tab-stop mới, xem doc-comment
 * của `onSourceSelectionChange`.
 */
import { readonly, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'

/** Đúng hai trường `StatusBar.vue` cần để vẽ câu — không `start`/`end`/`tier`. */
export type HoveredGlossaryTerm = {
  isConfirmed: boolean
  /** `null` khi và chỉ khi `isConfirmed === false`. */
  translation: string | null
}

const hovered = shallowRef<HoveredGlossaryTerm | null>(null)

/** Dấu đang được rê chuột tới, hoặc `null` — đọc bởi `StatusBar.vue`. */
export const hoveredGlossaryTerm: DeepReadonly<Ref<HoveredGlossaryTerm | null>> = readonly(hovered)

/** Ghi bởi `@mouseenter` của một mảnh mang dấu — `GridPanel.vue`/`SourceHanViet.vue`. */
export function setHoveredGlossaryTerm(term: HoveredGlossaryTerm): void {
  hovered.value = term
}

/** Ghi bởi `@mouseleave` — rời khỏi mảnh, hoặc rời khỏi một mảnh không mang dấu. */
export function clearHoveredGlossaryTerm(): void {
  hovered.value = null
}

/**
 * 🔴 Vứt state — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường `reset*()`.
 */
export function resetGlossaryTermHover(): void {
  hovered.value = null
}
