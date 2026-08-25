<!-- bmad:context -->
<!-- Verified 2026-08-25 against 69b19a8. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## src/ — Vue 3 + TypeScript

Frontend chỉ render và giữ state UI. Không quy tắc nghiệp vụ nào ở TypeScript (AD-1); ngoại lệ tường minh duy nhất là văn bản đang gõ trong Editor.

## Conventions that differ from defaults

- `invoke()` gửi tham số dạng **camelCase** dù hàm Rust nhận `snake_case` ⇒ viết `sourceLang`. NHƯNG trường của struct TRẢ VỀ giữ nguyên `snake_case` (`meta_schema_version`, `work_id`). Hai chiều khác nhau — đây là chỗ dễ sai nhất trên dây.
- Adapter IPC ở `src/config/*.ts` KHÔNG BAO GIỜ ném: một `invoke`, một `try/catch`, trả hình dạng ba trạng thái `{ <giá trị> | null, error: IpcError | null }`. Tầng UI hiển thị lỗi bằng `tError()`, không bằng `try/catch`. (`shortcutsState.ts` không phải adapter — nó là state Vue gọi xuống `bootstrap.ts`.)
- Luôn kiểm kiểu LÚC CHẠY cho dữ liệu qua dây. `IpcError` phía TS là một lời khai về dữ liệu đã đi qua IPC, không phải bảo đảm của trình biên dịch.
- `verbatimModuleSyntax` bật ⇒ `import type` phải tường minh. vitest đặt `globals: false` ⇒ mỗi tệp test tự `import { describe, it, expect } from 'vitest'`.
- `@click` trong `.vue` phải là ĐÚNG MỘT lời gọi `dispatch('<id>')` — không hàm khác, không mã nội tuyến (`check:commands` Kiểm A). Phím tắt và Auto-Lookup phát cùng một `dispatch(...)`: một lời gọi thẳng dựng đường thứ hai mà Kiểm A không nhìn thấy.
- Command id dùng cùng văn phạm khoá chấm với khoá i18n (`review.accept_change`) — id trần sẽ bị hai giai đoạn cách nhau nhiều tháng đăng ký trùng và ghi đè nhau âm thầm.
- Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó ghi chẩn đoán nêu đích danh rồi trả `false`. Đừng "vá" bằng cách tự chuyển chế độ: đó là đoán ý người dùng.
- Màu VÀ cỡ chữ chỉ đến từ token; không bóng đổ, không gradient, không lớp nổi. `opacity` trung gian cần một miễn trừ CÓ TÊN.
- Thư mục mang một khái niệm thì có `README.md` — hôm nay thiếu ở `src/config/` và `src/selftest/`.

## Known pitfalls

- 🔴 `Ref` KHÔNG tự bóc trong khối `<script>`, chỉ trong `template`. `if (someRef)` chạy trên **đối tượng** nên luôn đúng, và vì là TypeScript hợp lệ nên `vue-tsc` im. Lỗi này đã lọt qua CHÍN trên chín cổng và là lý do cổng thứ mười (`check:lint`, có kiểu) ra đời.
- 🔴 Năm tệp phải nạp được bằng **Node thuần** vì các cổng `import()` chúng để chạy phép kiểm HÀNH VI trên chính mã sản phẩm: `src/i18n/resolve.ts` (tệp này không import gì cả), `src/commands/{index,registry,focus}.ts`, `src/layout/writeSchedule.ts`. Cấm import GIÁ TRỊ từ `vue`/`dockview`; cấm `enum`, `namespace`, parameter property (`constructor(private x)`) — ba thứ đó sinh mã nên Node từ chối. Một dòng vi phạm giết ba phép kiểm cùng lúc. `src/layout/dockController.ts` tồn tại chính vì thế: `main.ts` tiêm hàm vào, không import ngược.
- 🔴 Thứ tự khởi động trong `src/main.ts` là bắt buộc, cả ba mệnh đề: `applyTheme()` trước `mount()` (nếu không, mọi `var(--color-…)` rỗng ở lượt render đầu ⇒ một nháy trắng — và trên bản đóng gói nháy đó NGẮN HƠN máy dev, nên lỗi chỉ lộ ở máy người khác); `installCommands()` trước `mount()` (`dispatch` ném với id chưa đăng ký); `loadFonts()` khởi động trước `await loadBootstrapConfig()`.
- Đăng ký command ở `main.ts`, KHÔNG trong `App.vue` — một lượt HMR dựng lại component sẽ gọi `installCommands()` lần hai và `register()` ném vì id trùng.
- `onDidLayoutChange` của dockview bắn LIÊN TỤC trong lúc kéo sash: ghi một `putConfig` mỗi lần bắn thì một cú kéo 3 giây là hàng trăm job nối tiếp qua `store::Writer`. Không cổng nào đỏ vì chuyện đó — nó lộ ra ở Epic 2 dưới dạng *"gõ bị khựng"*. Mọi nhịp ghi đi qua `src/layout/writeSchedule.ts`.
- Hai cặp hằng nhịp ghi, chỉ MỘT mang bảo đảm AD-35: bố cục dùng `IDLE_MS 500`/`HARD_CAP_MS 5000` ở `layout/writeSchedule.ts` (không mang bảo đảm); Editor dùng `EDITOR_IDLE_MS 2000`/`EDITOR_HARD_CAP_MS 5000` ở `panels/editorFlush.ts` (có). Dùng chung hình dạng, không dùng chung bảo đảm — đừng gộp hai cặp.
- Hàm nhịp ghi không tự đọc `Date.now()`: mọi thời điểm đi vào qua tham số, để phép kiểm tất định và tức thời thay vì phải `sleep` thật.
- Không cửa sổ OS thứ hai (AD-24): `addPopoutGroup` là đường duy nhất trong dockview gọi `window.open` — cấm. `check:layout` Kiểm C là một danh sách CHO PHÉP cho mọi thành viên `window`/`document` mà `src/**` chạm tới; thêm một cái tên là một quyết định phải viết ra.
- Nội dung từ ngoài KHÔNG BAO GIỜ render thành HTML: không `v-html`, không tương đương (AD-16). Rust phân tích thành mô hình dữ liệu có cấu trúc; Vue render từ mô hình đó.
- `src/selftest/**` cố ý không đi vào bản phát hành (`#[cfg(debug_assertions)]` phía Rust + `import()` động phía frontend) — không mã sản phẩm nào được import tĩnh từ đó.

<!-- /bmad:context -->
