/**
 * Vá những chỗ `happy-dom` **thiếu so với một DOM thật**. Story 2.3, Task 0b.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MỖI MỤC Ở ĐÂY LÀ MỘT KHOẢNG THIẾU CỦA BẢN MÔ PHỎNG, KHÔNG MỘT KHUYẾT TẬT SẢN PHẨM
 * ─────────────────────────────────────────────────────────────────────────────
 * Ranh giới này phải đọc được, vì hai thứ đó đòi hai hành động ngược nhau: một khoảng thiếu của
 * bản mô phỏng được vá **ở đây**; một khuyết tật sản phẩm được vá **trong `src/`**.
 *
 * ⚠️ Và đường sai rất rẻ: thêm một `?.` vào mã sản phẩm cho **hết đỏ**. Đó là một nhánh mà
 * **kiểu nói không bao giờ chạy** — `@typescript-eslint/no-unnecessary-condition` bắt đúng nó
 * ở `npm run check:lint`, và `GridPanel.vue` *(kế thừa từ `EditorPanel.vue`, gỡ ở Story 2.5b)*
 * đã ghi lại chính lần bắt đó ngày
 * 2026-08-12. Một nhánh phòng hờ cho một bản mô phỏng là mã chết vĩnh viễn trong sản phẩm.
 *
 * ⚠️ Mỗi mục PHẢI kèm một dòng nói nó thiếu gì và ai đọc nó. Danh sách này là một món **nợ đo
 * được**: nó càng dài thì khoảng cách giữa `happy-dom` và WKWebView càng lớn, và mọi mệnh đề
 * của cây test này càng cần bàn đo/e2e đứng sau.
 */

/**
 * `document.fonts` — `happy-dom@20.11.2` **không cài** FontFaceSet API.
 *
 * Ai đọc nó: 🔵 **KHÔNG AI, kể từ Story 2.5b.** Mệnh đề cũ — *`EditorPanel.vue::onMounted` chờ
 * `document.fonts.ready` để đo lại chiều cao vạch* — hết đúng: vạch nay lấy chiều cao từ track
 * hàng của `subgrid`, nên không còn lượt đo hình học nào để hẹn. Vá vẫn ở lại vì `happy-dom`
 * thiếu API này và một tệp test khác có thể chạm tới nó *(một lớp giả thiếu là một
 * `TypeError` lúc mount, không một ca đỏ đọc được)*.
 *
 * ⚠️ Đây là vế mà cây test này **không** nghiệm thu được, và nó không được đọc thành đã nghiệm
 * thu: `happy-dom` không có bố cục thật, nên `getClientRects()` ở đây trả 0 — mọi mệnh đề về
 * **hình học** vạch lề thuộc **bàn đo** (Story 2.2 đã đo trên hai engine thật).
 */
if (!('fonts' in document)) {
  Object.defineProperty(document, 'fonts', {
    configurable: true,
    value: { ready: Promise.resolve(), status: 'loaded' },
  })
}

/**
 * `ResizeObserver` — `happy-dom` cài nó, nhưng nó không bao giờ **bắn**, vì không có bố cục
 * thật để đổi kích thước.
 *
 * Ai đọc nó: 🔵 **KHÔNG AI, kể từ Story 2.5b** — `ResizeObserver` tồn tại trong `EditorPanel.vue`
 * để đo lại vạch khi panel đổi kích thước; lưới không cần lượt đo đó. Vá ở lại vì `happy-dom`
 * thiếu API này. Một lớp giả tối
 * thiểu ở đây chỉ để `new ResizeObserver(...)` không ném; vế *"đo lại khi panel đổi kích
 * thước"* thuộc bàn đo, cùng lý do trên.
 */
if (!('ResizeObserver' in globalThis)) {
  Object.defineProperty(globalThis, 'ResizeObserver', {
    configurable: true,
    value: class {
      observe(): void {}
      unobserve(): void {}
      disconnect(): void {}
    },
  })
}
