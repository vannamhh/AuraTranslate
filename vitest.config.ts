import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

/**
 * Bộ chạy test frontend — Story 2.3, Task 0b. Ice LẬT vế *"không bộ chạy test frontend"*
 * của NFR15 ngày 2026-08-12.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 CỬA RÀ GIẤY PHÉP CỦA NFR15 **VẪN ĐỨNG** — lượt lật này đi QUA nó, không xoá nó
 * ─────────────────────────────────────────────────────────────────────────────
 * Luật cũ chưa bao giờ nói *"không chạy được test"*; nó nói *"mở tệp giấy phép trong
 * nguồn đã tải, rồi vào bảng Stack, TRƯỚC khi thêm"*. Ba gói dưới đây đã đi qua đúng cửa
 * đó (§Completion Notes của story ghi đường dẫn và dòng đầu của cả ba tệp giấy phép thật).
 * **Gói thứ tư vẫn phải đi qua đúng cửa đó.**
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO CÂY TEST NẰM Ở `tests/frontend/**`, NGOÀI `src/**` — Quyết định #6
 * ─────────────────────────────────────────────────────────────────────────────
 * **BỐN** cổng đếm quần thể `src/**` rồi `abort()` khi số tệp **dưới sàn**, với một
 * doctrine ghi thành số: sàn phải nằm ở ~80–85 % của số thật
 * (`check-commands.mjs:200-245` · `check-tokens.mjs:85-92` · `check-layout.mjs:90-97`).
 * Sàn là **cận dưới**, nên tệp test đổ vào `src/**` KHÔNG làm cổng đỏ — nó **thổi phồng
 * mẫu số**, và mọi story sau phải nâng sàn vì một lý do **giả**. `check-tokens.mjs:85-91`
 * ghi lại đúng một lượt *"bắt kịp"* sau khi ba story để sàn tụt xuống 69,8 % — *"đúng
 * trạng thái canh không được gì mà chính nó cảnh báo"*.
 *
 * Cộng hai va chạm cụ thể: Kiểm A của `check-i18n` đỏ với **chữ tiếng Việt ở vị trí mã**
 * (mà một tệp test viết cho người Việt đọc thì đầy chuỗi tiếng Việt), và Kiểm B của
 * `check-tokens` đỏ với **màu viết thẳng trong component** (mà một fixture test màu vạch
 * cần đúng thứ đó).
 *
 * Tiền lệ miễn trừ **có tên** đã tồn tại cho đúng hình dạng này: `src-tauri/tests/**` được
 * miễn trừ khỏi Kiểm A của `check-i18n` (`core/store/mod.rs:52-59`). Cây frontend đi đúng
 * khuôn đó: một thư mục, một miễn trừ có tên, **0** tệp thêm vào quần thể của bốn cổng.
 *
 * ⚠️ Điều kiện kèm theo, và nó là mệnh đề của Task 0b.8: `tsconfig.json` phải **nhìn thấy**
 * cây này và `npm run build` (`vue-tsc --noEmit` hai lượt) phải vẫn xanh. Một cây test
 * không được kiểm kiểu là một cây test sẽ mục.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VAI CỦA BỘ CHẠY NÀY, VÀ BA VAI NÓ **KHÔNG** THAY — AC25, luật chống hai nguồn sự thật
 * ─────────────────────────────────────────────────────────────────────────────
 * Nó lo vế **hành vi** của module thuần, của mã đụng DOM, và của `.vue`. Nó **không** thay:
 *   - **cổng tĩnh `scripts/check-*.mjs`** — vế khai báo trên TOÀN CÂY (*"không màu viết
 *     thẳng ở bất kỳ đâu"*). Một cổng quét cả `src/**` tìm thứ không được phép tồn tại là
 *     một phép kiểm **khác hạng** với một test khẳng định một hàm trả đúng giá trị;
 *   - **bàn đo chạy tay** — vế thị giác và vế đo số trên engine thật;
 *   - **e2e WebdriverIO** — vế hành vi trong **WKWebView thật**. `happy-dom` là một bản mô
 *     phỏng DOM trong Node, **không phải** WebKit.
 */
export default defineConfig({
  plugins: [vue()],
  test: {
    // `happy-dom`, không `jsdom`: nó là peer TUỲ CHỌN của vitest 4 và nhẹ hơn hẳn. Vai của
    // nó dừng ở "một DOM đủ thật để mount một component" — mọi mệnh đề về engine thật đi
    // qua bàn đo hoặc e2e.
    environment: 'happy-dom',
    // ⚠️ Phạm vi quét CHỈ `tests/frontend/**` — xem khối Quyết định #6 ở trên. Đừng nới ra
    // `src/**`: mặc định của Vitest là đồng vị trí, và đó đúng là thứ bị loại bằng số đo.
    include: ['tests/frontend/**/*.test.ts'],
    // Vá những chỗ `happy-dom` thiếu so với một DOM thật. **Mỗi mục ở đó kèm một dòng nói nó
    // thiếu gì và ai đọc nó** — danh sách càng dài thì khoảng cách giữa bản mô phỏng và
    // WKWebView càng lớn, và mọi mệnh đề của cây test này càng cần bàn đo/e2e đứng sau.
    setupFiles: ['tests/frontend/support/setup.ts'],
    // Không `globals: true` — mỗi tệp test `import { describe, it, expect } from 'vitest'`
    // tường minh. Cùng lý do `verbatimModuleSyntax` bật trong `tsconfig.json`: một cái tên
    // xuất hiện từ hư không là một cái tên `vue-tsc` phải được dạy riêng để thấy.
    globals: false,
    // ─────────────────────────────────────────────────────────────────────────
    // 🔴 THÊM 2026-09-05 (Story 6.5, phán quyết Ice) — CỔNG PHẢI ĐO MÃ, KHÔNG ĐO MÁY
    // ─────────────────────────────────────────────────────────────────────────
    // Chín tệp trong cây này mang ca phụ thuộc TẢI MÁY: chúng mount component thật rồi
    // `await` một mốc, và `testTimeout` mặc định là 5 000 ms — nên phán quyết của chúng
    // là một phép đo của MÃ **cộng** MÁY. Đo được ở Story 6.5 (2026-09-05, cùng máy,
    // cùng lượt):
    //   • nền chưa có story, `npm run test` song song ......... 822/822 xanh, 27,43 s
    //   • có story, `npm run test` song song .................. 835 xanh / **5 ĐỎ**, 37,09 s
    //   • cùng cây, `--no-file-parallelism` ................... 840/840 xanh, 98,65 s
    //   • bốn tệp đỏ chạy RIÊNG ............................... 42/42 xanh
    // Cả bốn tệp đỏ (`editorClearSourceCuts` · `editorTypingZone` · `glossaryHoverSelection`
    // · `glossaryMarksRefresh`) KHÔNG nạp một tệp nào Story 6.5 chạm — `src/config/project.ts`
    // chỉ có ba chỗ import và không chỗ nào nằm trên đường của chúng. ⇒ Story 6.5 không đổi
    // hành vi của chúng; nó chỉ đẩy tải worker qua một ngưỡng VỐN ĐÃ lung lay (tệp nặng nhất
    // kho, `glossaryMarksRefresh`, đã mất 3,28 s ngay cả khi chạy tuần tự).
    //
    // Hai lối vá bị loại, mỗi cái vì một lý do đo được: nâng `testTimeout` là HẠ NGƯỠNG cho
    // hết đỏ (`src-tauri/AGENTS.md` cấm đúng hình dạng đó — cho exit 0 trên một sản phẩm
    // đang hỏng); hạ `maxWorkers` xuống một con số cố định là một con số phù thuỷ chưa đo
    // trên máy CI, tức vẫn để nguyên lớp lỗi "xanh ở máy Ice, đỏ ở CI" mà `AGENTS.md:28`
    // đã ghi thành luật.
    //
    // ⚠️ Giá phải trả, ghi ra thay vì làm nhẹ đi: mỗi lượt `npm run test` đi từ ~27 s lên
    // ~99 s, trên CẢ `pre-push` LẪN CI. Đổi lại, một lượt xanh nói về mã. Gốc rễ — bốn tệp
    // kia không được phụ thuộc wall-clock — là một món nợ CÓ CHỦ, không phải thứ story này
    // im lặng nhận.
    fileParallelism: false,
  },
})
