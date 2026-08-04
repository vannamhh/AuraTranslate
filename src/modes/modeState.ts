/**
 * Chế độ đang hiện — state UI, và **frontend được phép sở hữu nó** (AD-1: *"frontend chỉ
 * render và giữ state UI (focus, cuộn, vùng chọn, bố cục panel)"*). Story 1.6 · AC3.
 *
 * ⚠️ Tệp này ĐƯỢC PHÉP `import` Vue, khác hẳn `../commands/registry.ts`. Ranh giới nằm
 * ở chỗ ai bị `scripts/check-commands.mjs` nạp bằng Node thuần: `src/commands/**` thì
 * có, tệp này thì không.
 *
 * ⛔ Không có `#[tauri::command]` nào cho việc đổi chế độ. Một vòng IPC cho một thao tác
 * phải mượt là quy tắc nghiệp vụ giả đặt sai chỗ — xem §Vì sao story này KHÔNG có phần
 * Rust trong story file.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
// ⚠️ Đường dẫn TƯƠNG ĐỐI — dự án KHÔNG khai alias `@` (`vite.config.ts` và
// `tsconfig.json` đều không có `alias`/`paths`). Lượt review Story 1.5 đã bắt một README
// viết `@/i18n` không chạy được.
//
// Hướng phụ thuộc là `modes/` → `commands/`, một chiều. ⛔ Đừng đảo lại: `src/commands/**`
// phải nạp được bằng Node thuần, và một cạnh trỏ về đây là kéo `vue` vào cổng.
import { enterFocus, MODE_IDS } from '../commands'
import type { ModeId } from '../commands'

export type { ModeId } from '../commands'
export { MODE_IDS } from '../commands'

/**
 * `library` là mặc định vì PRD §5.2 gọi Library là **điểm vào ứng dụng**.
 *
 * ⚠️ Hôm nay cả ba chế độ đều rỗng nên lựa chọn này không quan sát được ở đâu ngoài tab
 * nào đang sáng lúc mở. Ghi lý do ra đây để Story 5.x không phải đoán.
 */
const mode: Ref<ModeId> = ref('library')

/** Chỉ đọc ở nơi tiêu thụ — đường đổi chế độ duy nhất là `setMode`, và nó đi qua command. */
export const currentMode: DeepReadonly<Ref<ModeId>> = readonly(mode)

export function setMode(next: ModeId): void {
  // Chốt lúc chạy, không chỉ lúc biên dịch: Story 1.8 sẽ nạp chế độ cuối cùng TỪ ĐĨA,
  // nơi giá trị có thể là `''` hay `undefined` sau một lần sửa tay tệp cấu hình. Cùng
  // khuôn với `applyTheme()` của Story 1.4 — rơi về mặc định và kêu to hơn là chết im.
  if (!MODE_IDS.includes(next)) {
    console.warn(`[modes] chế độ không hợp lệ: ${JSON.stringify(next)} — giữ nguyên \`${mode.value}\`.`)
    return
  }
  /**
   * 🔴 BẤM `⌘1` KHI ĐANG Ở CHÍNH CHẾ ĐỘ ĐÓ PHẢI DỜI ĐƯỢC FOCUS VỀ.
   *
   * Gán một giá trị không đổi cho một `ref` không kích hoạt re-render, nên không có
   * `onActivated`, nên không có `enterFocus` — trong khi `keys.ts` đã `preventDefault()`
   * rồi. Kết quả: phím bị nuốt và **không gì dời cả**.
   *
   * Đó đúng là thao tác một người dùng sẽ thử để tự cứu khi focus đã rơi ra ngoài (hoặc
   * đang mắc ở thanh tab sau một cú bấm chuột), và NFR17 hứa bàn phím luôn đưa họ về
   * được. Ở đây phần tử của chế độ **đã có trong DOM** — nó đang hiện — nên gọi thẳng
   * `enterFocus` là an toàn, khác hẳn nhánh đổi chế độ thật (xem comment ở
   * `../commands/index.ts`: lúc đó Vue mới chỉ nhận được thay đổi state).
   */
  if (mode.value === next) {
    enterFocus(`mode.${next}`)
    return
  }
  mode.value = next
}
