---
title: 'Thiết kế lại Chế độ đọc theo mockup'
type: 'feature'
ticket: ''
created: '2026-11-02'
status: 'built'
baseline_revision: 'e9edf4b294b15a47a39ddbbfa260132ca6e447f8'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Trang Chế độ đọc hiện là hàng nút viền rời và một dòng tiến độ thuần chữ, lệch hẳn `ux-auratranslate/mockups/reading-mode.html` (ảnh Ice gửi ngày 2026-11-02).

**Approach:** Dựng lại phần vỏ trang theo mockup: thanh công cụ phẳng, dải vị trí đọc có vạch tiến độ, đầu Chương có nhãn trên tiêu đề và viên xuất xứ, mốc biên gọn theo cột đọc. Hành vi, lệnh và dữ liệu giữ nguyên. Quyết định của Ice (2026-11-02): không ghim, cả hai thanh trôi cùng nội dung trong `section.mode`.

## Boundaries & Constraints

**Always:** Mọi nút vẫn là `dispatch('<id>')` với id cũ và giữ `data-reading-*` cũ. Phím hiện từ `chordOf()`, phím chưa gán thì không vẽ. `section.mode` vẫn là phần tử cuộn, mang `ref="root"` và tiêu điểm. Chỉ dùng token, bội 7 và control chung (`ui-switch`, `ui-seg`, `ui-kbd`, `ui-btn--quiet`).

**Never:** Không đổi Rust, không thêm lệnh, không ghi neo vị trí khi cuộn. Không lật các quyết định của Ice mới hơn mockup: `⌘,` thuộc Cài đặt (#102) nên Tinh chỉnh không vẽ `⌘,`; nguyên văn lề giữ cỡ và thẳng hàng hiện tại; chip Đánh dấu/Sửa, cỡ `ui-sm` 13px, `ui-seg` 28px giữ nguyên. Không dựng lại lớp phủ Mục lục và Danh sách đánh dấu.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Tiến độ thường | `done` 128, đang ở Chương 44 | Vạch 34%, chữ "128 Chương đã dịch · đang ở 44" | No error expected |
| Thứ tự vượt số đã dịch | `chapter_ord` 130, `done` 128 | Vạch dừng ở 100% | Kẹp, không tràn |
| Chưa biết tiến độ | `done` null hoặc không có Chương | Không có dải | Dải ẩn, trang vẫn đọc được |
| Không có `done` | `done` 0 | Vạch 0% | Không chia cho 0 |
| Chương không tên | `chapter_title` null | Dải in đậm "Chương N" một lần; đầu Chương không lặp nhãn | No error expected |

</frozen-after-approval>

## Code Map

- `src/modes/ReadingMode.vue:596` -- vỏ template, `.toolbar`, `.progress-line`, `.tuner`, `.chapter-origin`, `.frontier`, khối style. Hai thanh nằm trong `section.mode`, tràn lề bằng margin âm bù đúng `padding` của nó.
- `src/modes/ReadingMode.vue:141-205` -- logic neo đo `root`; không sửa.
- `src/modes/readingState.ts:75` -- `readingCurrentChapter` là nguồn của dải; dùng nguyên.
- `src/tokens/controls.css` -- `.ui-switch` (`[aria-checked]` xét con trực tiếp), `.ui-seg__item[aria-pressed]`, `.ui-kbd`, `.ui-btn--quiet`, `.ui-btn--primary`. Không sửa; ghi đè màu chọn nằm cục bộ.
- `src/i18n/vi.json:1339` -- `marks_button_count` tách để in đậm số.
- `scripts/check-tokens.mjs` -- Kiểm B2 (font-size/line-height chỉ `var`), cặp tương phản.
- `tests/frontend/readingMarks.test.ts`, `readingLookupPopup.test.ts` -- đọc `section.mode` làm phần tử cuộn; phải giữ xanh không sửa.
- `e2e/specs/story-5-11-reading-mode.e2e.mjs`, `story-5-13-reading-marks.e2e.mjs`, `story-13-7-ra-be-mat-library-doc.e2e.mjs` -- dùng `data-reading-*`.

## Tasks & Acceptance

**Execution:**
- [ ] `tests/frontend/readingToolbar.test.ts` -- tạo mới, viết đỏ trước -- phủ bảng I/O, `role="switch"`, chữ đầu Chương, vị trí nút đánh dấu.
- [ ] `src/modes/ReadingMode.vue` -- dựng thanh công cụ phẳng: Song ngữ `role="switch"`, nhóm Cỡ chữ với mục chọn nền `surface-accent` chữ `primary`, Tinh chỉnh, Sáng/Tối, Mục lục, nút đánh dấu sát phải màu `tm-text` -- khớp ảnh.
- [ ] `src/modes/ReadingMode.vue` -- dải vị trí: "Chương N · **tiêu đề**", vạch 2px `aria-hidden`, chữ tiến độ in đậm số -- khớp ảnh.
- [ ] `src/modes/ReadingMode.vue` -- đầu Chương: nhãn hoa, viên xuất xứ, tiêu đề; đệm `.page`; `.marked::after` cờ `⚑`; mốc biên rộng theo cột, tiêu đề `read-sm`, cụm nút giữa, nút đầu `ui-btn--primary`.
- [ ] `src/i18n/vi.json` -- tách đơn vị của nút đánh dấu, thêm khoá thiếu.
- [ ] `e2e/specs/story-5-11-reading-mode.e2e.mjs` -- thêm ca đo chiều cao hai thanh và vạch tiến độ.
- [ ] `ux-auratranslate/DESIGN.md` -- một dòng 🔵 ghi các lệch có chủ ý so với mockup.

**Acceptance Criteria:**
- Given cửa sổ 1280 px, when mở Chế độ đọc, then thanh công cụ một hàng cao 42 px, dải vị trí 28 px, đúng thứ tự trong ảnh.
- Given cửa sổ tối thiểu 800 px, when mở Chế độ đọc, then thanh xuống hàng, không tràn ngang.
- Given bấm hoặc dùng phím Song ngữ, 1 2 3, D, Mục lục, Tinh chỉnh, M, when thao tác, then kết quả như trước.
- Given nền tối, when đổi theme, then chữ và mục chọn đọc được, `check:tokens` xanh.
- Given trang dài, when cuộn xuống, then cả hai thanh trôi đi cùng nội dung.

## Implementation Notes

- `⌘,` thuộc Cài đặt nên Tinh chỉnh không có phím mặc định; nút vẽ phím từ `chordOf()` nếu Ice gán.
- Nút Sáng/Tối bỏ `aria-pressed`: nhãn đã đổi theo theme, giữ thuộc tính thì nền tối có khối nền thường trực.
- Kiểm K chỉ đọc thuộc tính vật lý, nên khoảng cách và cỡ trong tệp này viết vật lý để cổng thật sự canh.
- Nút thanh công cụ đệm 7 px, margin âm 7 px: chữ cách chữ đều 21 px, ô rê chuột vẫn rộng.
- Đo WebKit: 800 px thanh hai hàng cao 77 px, tiêu đề dài giữ dải 28 px, vạch 70 px, không tràn ngang; `scrollTop` 200 đưa cả hai thanh lên đúng 200 px.
- Lề phải 21 px là suy từ 11 px đệm cộng 10 px khe, chưa đo vì Playwright không có thanh cuộn.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens `quick`): high 1 · medium 7 · low 3 · false 0 · maybe-false 0.

- high · patch · `story-13-7` đo `titleToOrigin`, `originToFirst` theo thứ tự cũ, đỏ cả ba mức ở nightly; đã đọc `e2e/specs/story-13-7-ra-be-mat-library-doc.e2e.mjs:146-167`.
- medium · patch · Song ngữ vẽ công tắc trước chữ, ngược `reading-mode.html:119` và AC1.
- medium · patch · Nền tối giữ `aria-pressed` trên nút Sáng/Tối nên nút có khối nền thường trực; không test nào đọc thuộc tính này trên nút đó.
- medium · patch · Lề phải hai thanh 31 px: `padding-inline` 21 cộng khe 10, trang là 21.
- medium · patch · Tiêu đề dài ép vạch về 0 rồi tràn dải 28 px; `.progress-line` không có `nowrap`, `min-inline-size`.
- medium · patch · Guard e2e mới không thể đỏ: đo `documentElement` thay vì `section.mode`, vạch chỉ kẹp trong khoảng, 42 px bị điều kiện che.
- medium · patch · Test `readingToolbar` không có phím thật nên ca Tinh chỉnh vô nghĩa, nhánh "phím chưa gán" không chạy.
- medium · patch · Thiếu hàng ma trận "không có Chương"; ca `done` null không chờ `list_chapters`; ca "in đậm" không kiểm in đậm.
- low · patch · Chú thích đệm `.page` sai: thực tế 42 px vì `margin` 14 của dải và `.status` cộng với 28; nhắm 28 px.
- low · patch · `DESIGN.md` nói `ui-seg` chọn là `primary`, thực tế `on-surface`; thêm màu công tắc vào danh sách lệch.
- low · patch · `aura-allow-text` trong CSS không cổng nào đọc; `aura-allow-spacing` trên thuộc tính logic không miễn gì và Kiểm K bỏ sót chúng (`GRID_PROPS`, `SIZE_PROPS` chỉ có thuộc tính vật lý).
- low · reject · Mốc biên lệch cột khi song ngữ ở cửa sổ hẹp: có thật nhưng chỉ khi rãnh trái tối thiểu vượt nửa khoảng trống, không gặp ở cỡ mặc định, sửa phải đưa mốc vào lưới `.page`.
- low · patch · Tự phát hiện khi đọc diff: dải dùng nền `surface` và viền `outline`, mockup dùng nền trang và `outline-faint`.

## Verification

**Commands:**
- `npx vitest run tests/frontend/readingToolbar.test.ts tests/frontend/readingMarks.test.ts tests/frontend/readingMarksRemoval.test.ts tests/frontend/readingLookupPopup.test.ts tests/frontend/readingTypography.test.ts tests/frontend/readingModeImages.test.ts tests/frontend/readingFrontierDom.test.ts` -- expected: thoát 0, không `Errors`.
- `npx vue-tsc --noEmit` và `npm run -s check:tokens`, `check:i18n`, `check:commands` -- expected: thoát 0.
- Một spec e2e `story-5-11-reading-mode` -- expected: xanh (vitest không bắt TDZ vòng import).

**Manual checks (if no CLI):**
- So ảnh WebKit cạnh ảnh Ice gửi, sáng và tối.
- Cổng 1420 đang có Vite do Ice chạy: không tắt nó. Playwright WebKit (`find ~/.npm/_npx -maxdepth 4 -type d -name playwright`, nạp bằng `createRequire`) mở được `http://localhost:1420/` không cần Tauri nhưng không có dữ liệu Tác phẩm, nên muốn đo thanh phải giả `invoke` hoặc dùng e2e. Không nói "đã qua" khi chưa đo.
