import { createApp } from 'vue'
import App from './App.vue'
import './tokens/reset.css'
import { applyTheme } from './tokens'
import { loadFonts } from './tokens/fonts'

// ⚠️ THỨ TỰ BẮT BUỘC — `applyTheme()` phải chạy TRƯỚC `mount()`.
//
// Nó ghi toàn bộ token thành CSS custom properties lên `document.documentElement`. Nếu
// `mount()` đi trước, lượt render đầu tiên chạy khi `--color-*` chưa tồn tại: mọi
// `var(--color-…)` rơi về giá trị rỗng và người dùng thấy một nháy trắng trước khi giao
// diện lên màu. Trên bản đã đóng gói nháy đó ngắn hơn hẳn so với máy dev, nên đây là
// loại lỗi chỉ lộ ra ở máy người khác.
//
// Theme mặc định là `light` — nền giấy, đúng hướng "Bàn viết". Giao diện chọn theme và
// việc lưu lựa chọn xuống đĩa thuộc Story 1.8; ở đây chỉ có công tắc ở tầng hàm.
applyTheme('light')

// Nạp font KHÔNG chặn `mount()`: bốn tệp là ~26 MiB, và chờ chúng xong mới dựng cửa sổ
// là tự tay thêm một khoảng trắng vào lúc khởi động. Chữ hiện bằng font hệ thống trong
// vài trăm mili-giây đầu rồi đổi — `display: 'swap'` ở `fonts.ts` là thứ quyết định đó.
//
// ⛔ Không `await` ở đây, và cũng không nuốt lỗi im lặng: `loadFonts()` đã tự bắt mọi
// lỗi và trả về báo cáo, nên `catch` dưới đây chỉ còn bắt được lỗi của chính đường dẫn.
void loadFonts()
  .then((results) => {
    const failed = results.filter((r) => r.status === 'failed')
    for (const r of failed) console.warn(`[fonts] ${r.family} — ${r.file}\n  ${r.detail}`)
  })
  .catch((err) => console.warn(`[fonts] đường nạp font gãy trước khi chạy: ${String(err)}`))

createApp(App).mount('#app')
