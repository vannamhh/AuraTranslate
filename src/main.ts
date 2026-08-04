import { createApp } from 'vue'
import App from './App.vue'
import './tokens/reset.css'
import { applyTheme } from './tokens'
import { loadFonts } from './tokens/fonts'
import { attachKeyboard, installCommands } from './commands'
import { setMode } from './modes/modeState'

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

// ⚠️ THỨ TỰ BẮT BUỘC #2 — `installCommands()` phải chạy TRƯỚC `mount()`.
//
// `App.vue` render ba tab chế độ với `@click="dispatch('mode.…')"`, và `dispatch` NÉM
// với một id chưa đăng ký (AC1). Nếu lượt đăng ký đi sau `mount()` thì cú bấm đầu tiên
// trong khoảng giữa hai lệnh sẽ ném — hẹp, nhưng có thật, và đúng loại lỗi chỉ lộ ra
// trên máy chậm hơn máy dev.
//
// ⛔ Đăng ký ở đây chứ KHÔNG ở trong `App.vue`: một lượt HMR dựng lại component sẽ gọi
// `installCommands()` lần thứ hai, và `register()` ném vì id trùng — đúng hành vi AC2,
// sai chỗ để gặp nó.
//
// `setMode` được TIÊM VÀO thay vì `src/commands/index.ts` tự import: tệp đó phải nạp
// được bằng Node thuần để Kiểm C/D/E của `npm run check:commands` chạy trên chính bộ
// command của sản phẩm. Một dòng `import { setMode } from '../modes/modeState'` ở đó là
// kéo `vue` vào cổng và làm ba phép kiểm hành vi chết.
// 🔴 CANH GÁC — vì "ném thì đỏ ngay ở màn hình đầu tiên" chỉ đúng khi CÓ một màn hình.
//
// `register()`, `createKeymap()` và `parseChord()` đều ném theo thiết kế, và cả ba lý lẽ
// biện minh cho việc ném (`registry.ts`, `commands/index.ts`) đều dựa trên tiền đề rằng
// lỗi sẽ hiện ra trước mắt người dùng. Nhưng hai lệnh dưới đây chạy TRƯỚC `mount()` và
// `index.html` chỉ có một `<div id="app">` rỗng — nên một lần ném ở đây cho ra **cửa sổ
// trắng hoàn toàn**, với chẩn đoán nằm trong một console mà người dùng cuối không mở.
//
// Ca thật đã biết: một xung đột hợp âm chỉ tồn tại trên MỘT nền tảng (`Mod+1` phân giải
// thành `Ctrl+Digit1` ngoài macOS) — cổng xanh trên CI, cửa sổ trắng trên máy Windows.
//
// ⛔ Không nuốt lỗi: vẫn `throw` lại sau khi đã vẽ. Mục đích của khối này là làm cho lần
// ném đó NHÌN THẤY ĐƯỢC, không phải làm cho nó biến mất.
try {
  installCommands({ setMode })

  // `void` tường minh: `attachKeyboard` trả về hàm gỡ, `noUnusedLocals` đang bật, và cửa
  // sổ này sống đúng bằng vòng đời tiến trình nên không có chỗ nào để gọi hàm gỡ cả.
  void attachKeyboard(window)
} catch (err) {
  // ⚠️ Cố ý KHÔNG đi qua `t()`: lượt cài đặt vừa gãy, nên mọi giả định về trạng thái ứng
  // dụng đều đáng ngờ — kể cả catalog chuỗi. Một câu tiếng Việt viết thẳng ở đây là đúng
  // chỗ duy nhất trong dự án mà điều đó được phép, và lý do là nó phải hiện ra kể cả khi
  // phần còn lại đã chết.
  const host = document.getElementById('app')
  if (host !== null) {
    // ⚠️ Token, KHÔNG màu viết thẳng — và điều đó khả thi vì `applyTheme('light')` ở trên
    // đã ghi xong toàn bộ custom property lên `documentElement` TRƯỚC khối `try` này.
    // Một miễn trừ `aura-allow-literal` ở đây sẽ là lách cổng cho một thứ không cần lách.
    const box = document.createElement('pre')
    box.style.cssText =
      'margin:0;padding:var(--space-panel-inline);white-space:pre-wrap;' +
      'font-family:var(--face-ui-mono);font-size:var(--font-ui-mono);' +
      'line-height:var(--leading-ui-mono);color:var(--color-error);' +
      'background:var(--color-background);height:100vh;overflow:auto;box-sizing:border-box'
    box.textContent =
      'AuraTranslate không khởi động được.\n\n' +
      'Lượt đăng ký thao tác hoặc lượt dựng bàn phím đã ném, nên giao diện chưa được ' +
      'dựng.\nĐây là một lỗi lập trình, không phải lỗi dữ liệu của bạn.\n\n' +
      String(err instanceof Error ? (err.stack ?? err.message) : err)
    host.appendChild(box)
  }
  throw err
}

createApp(App).mount('#app')
