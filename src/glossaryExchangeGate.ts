/**
 * Cửa loại trừ Xuất ↔ Nhập — ban đầu chỉ của Glossary (cụm D vá, vòng rà Epic 3, 2026-08-26).
 *
 * 🔵 **DÙNG CHUNG từ Story 4.5** — `promptSetImportState.ts::openPromptImportPreviewOverlay`
 * (và, gián tiếp qua chỗ gọi của nó, lượt Xuất một bộ prompt) đọc/ghi ĐÚNG cờ này thay vì một
 * bản twin riêng: spec 4.5 §Code Map nói "share the existing exchange latch (or a twin)", và
 * chia sẻ đúng nghĩa hơn ở đây — CẢ Glossary lẫn bộ prompt đều mở một hộp thoại HỆ ĐIỀU HÀNH
 * của chính Rust, và ứng dụng không có mô hình nào cho "hai hộp thoại chọn tệp cùng mở" bất
 * kể domain. Tên tệp giữ nguyên (đổi tên là một việc tách biệt, không cần cho Story 4.5).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO MỘT TỆP RIÊNG, KHÔNG PHẢI MỘT MODULE IMPORT LẪN NHAU
 * ─────────────────────────────────────────────────────────────────────────────
 * `glossaryManageState.ts` (Xuất CSV) và `glossaryImportState.ts` (mở hộp thoại Nhập CSV,
 * Story 3.10b) là hai module state ĐỘC LẬP theo chủ ý — doc-comment đầu mỗi tệp nói thẳng
 * điều đó. Đóng cửa loại trừ Xuất↔Nhập (hai lượt cùng mở hộp thoại hệ điều hành của Rust là
 * một trạng thái không ai định nghĩa hành vi, §Tasks của spec) mà không làm hai module đó
 * `import` LẪN NHAU — một phụ thuộc VÒNG — cần một ô nhớ đứng NGOÀI cả hai.
 *
 * §Ask First của spec trình hai hình dạng: "một cờ thứ ba ở tầng trên" hay "mỗi bên đọc
 * `readonly` của bên kia". Hình dạng thứ hai sinh phụ thuộc VÒNG ngay khi CẢ HAI phía cần tự
 * CHẶN ở đường vào hàm của chính mình (không chỉ đọc để hiện `:disabled` trên nút) — mỗi bên
 * phải `import` bên kia, và đồ thị import khép vòng. Hình dạng NÀY không sinh vòng: đồ thị
 * import chỉ có hai cạnh, cả hai đều đi VÀO tệp này (`glossaryManageState.ts` → đây,
 * `glossaryImportState.ts` → đây); tệp này không `import` lại module nào trong hai module đó.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'

const busy = ref(false)

/** Đúng khi MỘT TRONG HAI lượt Xuất/Nhập Glossary đang mở hộp thoại hệ điều hành của Rust. */
export const glossaryExchangeBusy: DeepReadonly<Ref<boolean>> = readonly(busy)

/**
 * Đặt cờ dùng chung — gọi từ CẢ HAI module Xuất (`glossaryManageState.ts::exportGlossaryManageTier`)
 * và Nhập (`glossaryImportState.ts::openGlossaryImportPreviewOverlay`), SÁT NGAY mỗi lượt
 * đổi cờ RIÊNG của chính chúng (`exportBusy`/`opening`) — hai cờ đó và cờ này luôn đổi CÙNG
 * NHỊP, không có đường nào một bên đổi mà bên kia không theo.
 */
export function setGlossaryExchangeBusy(value: boolean): void {
  busy.value = value
}

/**
 * `check:panel-refs` — mọi ô nhớ cấp module phải có một đường `reset*()`.
 *
 * 🔵 **SỬA (vòng rà thứ hai, #8) — "0 chỗ gọi sản phẩm hôm nay" hết đúng, và đây là bản sửa
 * TẠI CHỖ, không chỉ chú thích.** Bản trước khai hàm này là chỗ gọi của
 * `resetGlossaryManage`/`resetGlossaryImport` trong khi CẢ HAI gọi thẳng
 * `setGlossaryExchangeBusy(false)` — một export sống trên giấy, chết trên mã (`grep` = 0 chỗ
 * gọi thật). Nay CẢ HAI hàm `reset*()` đó gọi ĐÚNG hàm này, nên câu khai ở trên khớp mã: 0
 * đường nào để cờ này kẹt `true` sau khi cả hai lớp phủ sở hữu nó đã tự dọn.
 */
export function resetGlossaryExchangeGate(): void {
  busy.value = false
}
