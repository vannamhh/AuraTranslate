/**
 * State của lớp phủ **Thư viện prompt** — Story 4.4 (FR69), lớp phủ THỨ MƯỜI MỘT
 * (`src/SettingsOverlay.vue:4-8` — "A big screen is the 11th overlay", §Never của spec 4.4).
 *
 * Chỉ giữ cờ MÀN HÌNH đang mở hay không, khuôn NGUYÊN VĂN `dictSourcesState.ts::attributionIsOpen`/
 * `settingsState.ts::overlayOpen` — dữ liệu bộ prompt (danh sách hai tầng đã phân giải) sống ở
 * `promptSetState.ts` (Phase 4a), tệp đó KHÔNG bị sửa ở đây, chỉ được TIÊU THỤ. Lựa chọn hàng
 * đang sửa/soạn bên trong màn hình là state CỤC BỘ của `PromptLibraryOverlay.vue` — nó không cần
 * lộ ra ngoài component, nên không sống ở tệp module này (khác cờ mở/đóng, thứ `main.ts` PHẢI
 * đọc được để nạp vào `isBlocked`).
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { loadPromptSets } from './promptSetState'

const overlayOpen = ref(false)

export const promptLibraryOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)

/**
 * Handler thật của `prompt.library.open` — mở lớp phủ và nạp lại danh sách hai tầng đã phân
 * giải, cùng lý do `openGlossaryManage()`: người dùng có thể đã tạo/xoá/đổi tên một bộ ở một
 * phiên trước, hoặc mở/đóng một Tác phẩm khác kể từ lần cuối màn hình này mở.
 */
export function openPromptLibrary(): void {
  overlayOpen.value = true
  void loadPromptSets()
}

/** Handler thật của `prompt.library.close`. KHÔNG dọn danh sách đã nạp — mở lại không cần nạp
 * lại NGAY, cùng khuôn `closeGlossaryManage`/`closeSettings`. */
export function closePromptLibrary(): void {
  overlayOpen.value = false
}

/** `check:panel-refs` đòi mọi ô nhớ cấp module có một đường `reset*()` của CHÍNH tệp này. */
export function resetPromptLibrary(): void {
  overlayOpen.value = false
}
