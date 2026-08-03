/**
 * Tên event self-check, tách thành module riêng vì một lý do rất cụ thể.
 *
 * `App.vue` phải phát event này ở nhánh `catch` — nhánh chạy khi `scopeCheck.ts` gãy
 * TRƯỚC khi kịp phát gì. Nó không thể `import` hằng từ `scopeCheck.ts`: một import
 * TĨNH sẽ kéo cả module self-check vào bundle chính, phá đúng bất biến mà
 * `#[cfg(debug_assertions)]` phía Rust và `import()` động phía frontend cùng giữ —
 * *"mã self-check không vào bundle release"* (`src-tauri/src/lib.rs`).
 *
 * Bản trước giải bằng cách viết cứng chuỗi ở `App.vue`, và chỗ viết cứng đó nằm đúng
 * trong nhánh xử lý lỗi — nhánh chỉ chạy khi đã có sự cố, tức chỗ ít được kiểm nhất.
 * Đổi tên event một lần là nhánh đó phát vào hư không và "báo FAIL tường minh" thoái
 * hoá về đúng cái treo mà nó tồn tại để chặn.
 *
 * Module này rỗng ngoài một hằng, nên kéo nó vào bundle chính không tốn gì.
 *
 * ⛔ Ba chỗ phải khớp nhau, đổi một là đổi cả ba:
 *   - `SCOPE_SELFTEST_EVENT` ở `src-tauri/src/lib.rs`
 *   - `SELFTEST_EVENT` ở đây
 *   - `scripts/check-scope*.mjs` đọc dòng `VERDICT:` (không phụ thuộc tên event)
 */
export const SELFTEST_EVENT = 'selftest:scope-check'
