/**
 * State cấp module cho THEME đang áp — Story 5.11, FR9 (`D` đảo sáng/tối).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ VÌ SAO MỘT TỆP MỚI, KHÔNG THÊM MỘT `ref` VÀO `tokens/index.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `tokens/index.ts` phải nạp được từ nhiều nơi mà không kéo theo Vue có STATE — nó chỉ
 * xuất một hàm thuần `applyTheme()`. Trước story này KHÔNG có `ref` module-level nào giữ
 * "theme đang áp": `main.ts:331` gọi `applyTheme()` đúng MỘT lần lúc khởi động rồi không ai
 * đọc lại. `reading.toggle_theme` (`D`) là đường ĐỔI lúc chạy đầu tiên, và nó cần một nguồn
 * sự thật để ĐỌC theme hiện tại trước khi đảo — `document.documentElement.dataset.theme`
 * không phải nguồn sự thật, nó là HỆ QUẢ của `applyTheme()`; đọc ngược nó từ DOM là dựng
 * một nguồn sự thật thứ hai (AD-1: state UI thuộc về TypeScript, không suy từ DOM).
 *
 * ⚠️ **Loại chỉ-toàn-cục của AD-18** (cùng hạng `commands/index.ts::keymap`/
 * `config/bootstrap.ts::layout` ở `check-panel-refs.mjs`): theme là một lựa chọn ứng dụng,
 * không theo Tác phẩm — `resetThemeState()` KHÔNG được gọi từ `resetReading()` hay bất kỳ
 * lượt đổi Tác phẩm nào.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { KEY_THEME, putConfig, SCOPE_APP_CONFIG } from '../config/bootstrap'
import { applyTheme, DEFAULT_THEME, isTheme } from './index'
import type { Theme } from './index'

const theme = ref<Theme>(DEFAULT_THEME)

/** Theme đang áp. Chỉ đọc ở nơi tiêu thụ — đường đặt duy nhất là [`setTheme`]. */
export const currentTheme: DeepReadonly<Ref<Theme>> = readonly(theme)

/**
 * Áp một theme lên `documentElement` và cập nhật [`currentTheme`] — **KHÔNG ghi xuống đĩa**.
 *
 * 🔴 **Vì sao đường KHỞI TẠO tách khỏi [`setTheme`] — bắt ở lượt rà 2026-08-30.** Bản đầu để
 * `main.ts` gọi thẳng `setTheme()` lúc khởi động, và `setTheme()` thì luôn `putConfig`. Hai
 * hệ quả, cả hai đều sai:
 *
 * ① **Một lượt GHI `global.db` ở MỖI lần mở ứng dụng**, qua `store::Writer` nối tiếp, chỉ để
 *    chép lại đúng giá trị vừa đọc ra từ chính nó vài mili-giây trước.
 * ② Nặng hơn: nó biến *"người dùng CHƯA TỪNG chọn theme"* thành một giá trị **đã lưu tường
 *    minh**. `config/bootstrap.ts` §`deleteConfig` khai thẳng luật của cửa này — *"đường đọc
 *    phân biệt ba trạng thái bằng **sự có mặt của khoá**, không bằng giá trị"* — nên một lượt
 *    ghi mặc định lúc khởi động **xoá một trạng thái** khỏi mô hình, vĩnh viễn và im lặng.
 *
 * ⇒ Khởi động dùng hàm này; chỉ một thao tác NGƯỜI DÙNG mới đi qua [`setTheme`].
 */
export function initTheme(next: Theme): void {
  if (!isTheme(next)) {
    console.warn(`[tokens] theme không hợp lệ: ${JSON.stringify(next)} — rơi về \`${DEFAULT_THEME}\`.`)
    next = DEFAULT_THEME
  }
  applyTheme(next)
  theme.value = next
}

/**
 * Đặt theme theo một thao tác NGƯỜI DÙNG: áp lên `documentElement`, cập nhật
 * [`currentTheme`], rồi LƯU xuống `global.db`.
 *
 * ⚠️ **Đây là đường GHI đầu tiên cho khoá `theme`** — trước story này chỉ có đường ĐỌC
 * (`BootstrapConfig.theme`, Story 1.4/1.8). `void putConfig(...)`: một lượt lưu trượt chỉ
 * ghi chẩn đoán, không chặn thao tác đổi theme — AD-34 đòi thao tác này MƯỢT, và một hộp
 * thoại lỗi ở đây là quy tắc nghiệp vụ giả đặt sai chỗ (cùng lý lẽ `main.ts::watch(currentMode, …)`).
 *
 * 🔴 **Không gọi hàm này lúc khởi động** — xem [`initTheme`] ngay trên.
 */
export function setTheme(next: Theme): void {
  if (!isTheme(next)) {
    console.warn(`[tokens] theme không hợp lệ: ${JSON.stringify(next)} — giữ nguyên \`${theme.value}\`.`)
    return
  }
  applyTheme(next)
  theme.value = next
  void putConfig(SCOPE_APP_CONFIG, KEY_THEME, next).then((err) => {
    if (err !== null) console.warn(`[tokens] không lưu được theme (\`${err.code}\`).`)
  })
}

/** Đảo sáng ↔ tối. Handler của `reading.toggle_theme` (`D`). */
export function toggleTheme(): void {
  setTheme(theme.value === 'light' ? 'dark' : 'light')
}

/**
 * Vứt state — `check:panel-refs` đòi mọi ô nhớ cấp module đi qua một hàm `reset*()`. Đây
 * KHÔNG phải một miễn trừ: [`theme`] được gán lại ngay trong hàm này.
 *
 * ⚠️ Sản phẩm không có chỗ gọi — theme là tuỳ chọn ỨNG DỤNG (§doc-comment đầu tệp), không
 * theo Tác phẩm, nên nó không thuộc `resetReading()`. Hàm này tồn tại cho bàn đo/test, cùng
 * khuôn `resetLibraryChapters()`.
 */
export function resetThemeState(): void {
  theme.value = DEFAULT_THEME
}
