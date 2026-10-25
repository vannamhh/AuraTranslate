---
type: ux-brief
title: 'Library: đưa bản dựng về mockup, nghiệm thu bằng mắt'
created: 2026-10-25
author: Sally
status: Ice duyệt 2026-10-25
---

# Library: đưa bản dựng về mockup, nghiệm thu bằng mắt

Ice chọn (2026-10-25): mockup `mockups/library-and-import.html` đạt, sửa app theo nó. Làm Library trước, các màn khác sau. Phần mockup gốc không vẽ (chọn thẻ, nút Tiếp tục dịch, lưới màn rộng, sáu tông bìa) vẽ ở `mockups/library-luoi-rong.html`.

**Vì sao lệch.** Plan story 12.2 mô tả đúng mockup. Retro Epic 12 ghi "Không chạy thật app", nên không ai đặt ảnh app cạnh mockup trước khi đóng epic. Phiếu này không đổi `EXPERIENCE.md`; L7 thêm token vào `DESIGN.md`.

**Đầu vào:** ảnh Library Ice chụp 2026-10-25 (cửa sổ 1490×935, 47 Tác phẩm tên `test`) và mã ở HEAD `89f37fd`.

## Quyết định của Ice (2026-10-25)

1. **Bấm một lần là chọn thẻ.** Hiện bấm không chọn được; chỉ hai nút `‹ ›` dời được con trỏ. Bấm đúp hoặc `↵` mở trang Tác phẩm (danh sách Chương).
2. **Thêm nút "Tiếp tục dịch"** để vào thẳng phần dịch.
3. **Màu bìa theo mockup**, Sally đề xuất thêm tông.
4. **Màn rộng thì thêm cột; bề rộng thẻ giữ như mockup.** Thay quyết định 1 của plan 12.2 (container query 4 / 3 / 2 cột).

## Lệch và cách sửa

| # | App thật | Cách sửa | Mã |
|---|---|---|---|
| L1 | Bìa vuông `aspect-ratio: 1/1`, cao khoảng 330 px, chữ cái đầu ở giữa | Bìa cao 112 px như mockup; bỏ chữ cái đầu (tên vẫn ở `aria-label`); nhãn ngôn ngữ chữ hoa nhỏ góc phải. 🔵 2026-10-25: bìa cao 136 px, mang tên Tác phẩm (L2); nhãn ngôn ngữ đã bỏ ở `640225b` | `LibraryMode.vue:1271` |
| L2 | Tên Tác phẩm chữ sans đậm nhỏ | Tên dùng chữ nội dung (serif); dòng phụ `lĩnh vực · sửa … trước`. 🔵 2026-10-25: tên in TRÊN bìa, serif 16 px màu `on-cover`, cắt 3 dòng, góc trên trái; phần dưới thẻ chỉ còn dòng phụ và tiến độ (`DESIGN.md` §Tông bìa Tác phẩm) | `.work-name` |
| L3 | "chưa biết", "Chưa biết tiến độ" in ra thẻ | `status = null` thì không có huy hiệu; `chapter_done_count = null` thì không có thanh, dòng tiến độ chỉ ghi `N chương`. Trạng thái ngoài danh mục (dữ liệu hỏng) vẫn hiện chữ | `LibraryMode.vue:595,609` |
| L4 | Bấm thẻ không làm gì; "Mở Tác phẩm", "Mở thư mục" xếp dọc trong thẻ | Bấm thẻ ngắm con trỏ bằng `@mousedown`/`@focusin`, theo khuôn `SegmentHistoryOverlay.vue:293` (AD-34 giữ); `@dblclick` và `↵` gọi `library.open_work`. Thẻ đang chọn: viền `primary` 2 px. Bỏ hai nút trong thẻ | `LibraryMode.vue:552+` |
| L5 | Bốn tầng điều khiển trên lưới | Một hàng: ô tìm · `Chính xác dấu \| Không dấu` · chip `Trạng thái ▾` · `Lĩnh vực ▾` · `Ngôn ngữ nguồn ▾` · `Sắp xếp ▾`. Chip đang lọc viền `primary` và mang tên giá trị (`Đang dịch ▾`); "Bỏ lọc" nằm trong menu của chip | — |
| L6 | Radio `Cả hai · Nguyên văn · Bản dịch` luôn hiện | Chỉ hiện khi ô tìm có chữ (FR139) | — |
| L7 | Bìa một màu nền chung | Sáu tông, mỗi Tác phẩm một tông cố định chọn theo `work_id` (đổi tên không đổi màu). Tông 1–3 lấy từ mockup gốc, 4–6 thêm: rêu, gạch trầm, lam đêm. Nền tối dùng cùng sắc, sáng hơn một bậc. Giá trị ở `DESIGN.md` §Tông bìa Tác phẩm (12 token mỗi theme); `check-tokens.mjs` chép thêm vào bảng đóng băng. Nhãn ngôn ngữ dùng nền `surface` đặc, không `rgba`, nên chỉ dùng cặp tương phản đã kiểm | `DESIGN.md:171` |
| L8 | Nút "Tải danh sách" | Bỏ nút; `onActivated` đã tự tải. Lệnh vẫn ở sổ lệnh | `LibraryMode.vue:151` |
| L9 | Dòng hướng dẫn tìm, tiêu đề "Tác phẩm", "47 / 47 Tác phẩm" | Hướng dẫn thành placeholder của ô tìm. Bỏ tiêu đề. Số đếm chỉ hiện khi có lọc (`5 / 47`), cuối hàng chip | — |
| L10 | Con trỏ lưới `‹ 1/47 ›` | Bỏ hai nút. Mũi tên trái/phải/lên/xuống dời lựa chọn khi lưới có tiêu điểm; lên/xuống nhảy đúng số cột đang hiện | `LibraryMode.vue:530` |
| L11 | "Nhập tài liệu" là nút tối trong hàng lọc | Lên thanh tiêu đề bên phải, cạnh `Công cụ ▾`, kiểu chữ trầm kèm `⌘N`, chỉ ở Library | `App.vue` |
| L12 | Thanh trạng thái ở Library hiện `⌘M gộp · ⌘/ tách`, TM, Glossary, câu auto-save dài | Ở Library chỉ hiện việc của Library (số Tác phẩm, trạng thái quét, lỗi). Gợi ý phím, TM, Glossary, auto-save chỉ ở Workspace. Viết lại thanh trạng thái Workspace là phiếu sau | `StatusBar.vue:518-611` |
| L13 | Không có đường vào thẳng phần dịch | Nút **Tiếp tục dịch →** ở góc dưới trái bìa, trên thẻ đang chọn và thẻ có chuột rê qua; Tác phẩm `not_started` thì ghi **Bắt đầu dịch →**. Lệnh mới `library.continue_translation` = `openWorkById` rồi chuyển sang Workspace; Rust đã mở lại đúng Chương (`work.last_chapter_id`, `commands/chapter.rs:117`) và Workspace đã khôi phục câu (`chapter_position`), nên không cần lệnh Rust mới. Cạnh đó nút `⋯`: Mở thư mục · Nhập bản reviewer | `libraryChapters.ts:430` |
| L14 | Lưới cố định 4 / 3 / 2 cột | `repeat(auto-fill, minmax(240px, 1fr))`, gap 20 px, đệm hai bên 22 px. Từ 4 cột trở lên thẻ rộng 240–305 px: cửa sổ 1150 px ra 4 cột 262 px (đúng mockup), 1490 px ra 5 cột 273 px, 2560 px ra 9 cột 262 px. Cửa sổ hẹp ít cột hơn thì thẻ rộng tới 370 px | `LibraryMode.vue` `.works-grid` |

## Câu hỏi cho người dựng

- Cả 47 Tác phẩm trong ảnh đều "Chưa biết tiến độ", tức `chapter_done_count = null` ở mọi hàng. L3 giấu chữ đó, nhưng phải tìm ra vì sao **không hàng nào** có tiến độ trước khi giấu (lớp lỗi rỗng im lặng).
- Phím mặc định cho `library.continue_translation` (gợi ý `Mod+↵`) và cho mũi tên lưới phải không đụng phím đã gán; xem sổ lệnh.
- `Tiếp tục dịch` khi Tác phẩm không mở được (khoá, lỗi) phải dừng ở Library và báo lỗi, không chuyển chế độ; cùng luật `library.open_chapter`.

## Nghiệm thu

- Ảnh app thật, cửa sổ 1490×935, dpr=2, nền sáng và nền tối, đặt cạnh `library-luoi-rong.html` cùng khổ. Mỗi dòng L1–L14 đánh dấu khớp hoặc ghi lý do.
- Dữ liệu chụp là một Library mẫu có tên, lĩnh vực, tiến độ thật, không phải 47 Tác phẩm `test`; có đủ thẻ thiếu lĩnh vực, thẻ chưa biết tiến độ, thẻ chưa bắt đầu và thẻ đã xong.
- Đi trọn bằng bàn phím: `⌘1` → mũi tên tới thẻ thứ ba → `Mod+↵` vào Workspace đúng câu đang dở; quay lại Library thì tiêu điểm về đúng thẻ đó.
- Kéo cửa sổ từ 1150 lên 2560 px: số cột tăng, thẻ luôn rộng 240–305 px.
- Ice tự chạy app một lượt trước khi story `done`.
