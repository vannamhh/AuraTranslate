# Hồ sơ bàn giao cho Winston — một `AD` mới: **mốc so xuất xứ lưu phía Rust**

**Ngày:** 2026-10-01 · **Người bàn giao:** lượt `bmad-build` mở Story 7.2 · **Người nhận:** Winston (architect)
**Nguồn gốc:** AC3 của Story 7.2 (*"câu họ viết lại mang xuất xứ tôi dịch"*), spec `implementation-artifacts/spec-7-2-xuat-xu-tren-tung-cap-tm.md` §D1.
**Quyết định của Ice (2026-10-01):** phương án (A) — lưu mốc phía Rust. Hai phương án bị loại: (B) giữ mốc trong webview suốt phiên (vẫn mất khi mở lại app); (C) không sửa, ghi nợ.
**Baseline cây nguồn:** `266c155f5ab91c00337cd065b50e81a668efee13`. Số `AD` trống kế tiếp: **50** (spine dừng ở AD-49; hồ sơ `ad-brief-2026-08-17-vach-le-cau-cuoi-chuong.md` chưa viết và cố ý không nhận số).

## 1. Lỗ đo được trong mã

1. Người dùng mở một Tác phẩm nhập song ngữ (hoặc câu đã nhận đề xuất AI), viết lại một câu. Flush (AD-35) ghi văn bản xuống đĩa và **không** đụng `translation_origin` — đúng AD-47 ①.
2. Người dùng đổi Chương rồi quay lại, hoặc đóng và mở lại app. `ensureSegmentsLoaded` (`src/panels/editorPanelState.ts:186`) nạp lại `segments`, nên mốc so trong webview **bằng chính câu họ vừa viết**.
3. Xác nhận ⇒ `confirm_segment` (`src-tauri/src/commands/segment.rs:2558`) thấy văn bản trùng `text_at_load` và xuất xứ trên đĩa khác `''` ⇒ trả về `origin_at_load` (`bilingual_import`/`other`) ⇒ cặp TM (Story 7.1, `insert_pair` tại `:2577`) mang nhãn *của người khác* cho chữ của người dùng.

Hỏng im lặng, đúng lớp AD-47 §Prevents mô tả, chỉ ngược chiều: lần này chữ của người dùng bị xếp sau trong `RagInjector`, thay vì chữ người khác bị đẩy lên. Chuyện đổi Chương là thao tác thường nhật, không phải ca biên.

## 2. Vì sao cần một `AD`, không phải một dòng mã

- AD-47 ③ liệt *"Nạp Chương từ đĩa"* là một lượt ghi không-phải-người-dùng, và ② nói *"hôm nay lượt nạp là lượt ghi không-phải-người-dùng duy nhất đã cài"*. Hai câu đó hợp thức hoá việc lượt nạp đặt lại mốc.
- Quyết định #2(b) (2026-08-16, *"mốc sống ở webview, Rust tin nó"*) và mục sổ nợ `deferred-work.md:3693` (*"Rust TIN mốc do webview khai"*, KHÔNG LÀM 2026-09-28 bởi Story 11.5) ghi rõ: *"Mở lại chỉ khi một AD mới làm mốc suy được ở phía Rust (vd. lưu phía server)."*

## 3. Câu hỏi cho `AD`

1. **Hình dạng lưu.** Mốc cần hai giá trị: văn bản mốc và xuất xứ mốc. Xuất xứ mốc phải tách khỏi `translation_origin` vì lượt xác nhận ghi đè cột đó thành `self`, trong khi Quyết định #96 (`deferred-work.md:3820`) đòi chuỗi ký → sửa → ký → sửa ngược → ký lại trả về xuất xứ **lúc đặt mốc**. Hai cột trên `segment`, hay một dạng khác? Đặt tên theo quy ước "xuất xứ" của spine (không `origin` trần).
2. **Ai đặt mốc.** Đi theo bảng AD-47 ③: nhập song ngữ, đưa đề xuất AI sang Editor, chấp nhận Review Mode (FR94), điền sẵn TM 100% (Story 7.4), gộp/tách (④), khôi phục (⑤ — đặt văn bản mốc, không đặt xuất xứ). Lượt nạp Chương **thôi** đặt mốc. Flush vẫn không đặt mốc (①, giữ ca *gõ rồi hoàn tác*).
3. **Dây IPC.** `confirm_segment` có bỏ `text_at_load`/`origin_at_load` không (webview hết giữ mốc), và `ipc_contract` ghim hình dạng mới thế nào?
4. **Di trú (bước 28, sau `tm_unit` bước 27).** Backfill cho hàng cũ: mốc = `target_text` hiện tại, xuất xứ mốc = `translation_origin` hiện tại? Cách đó giữ đúng ngữ nghĩa hôm nay cho dữ liệu đã có; câu đã viết lại và flush trước bước 28 vẫn mang lỗ cũ — cùng khuôn với "không backfill" D3 của Story 7.1.
5. **Bảng ⑦ "đổi gì / không đổi gì":** AD-31 bảng xuất xứ, AD-35, AD-5 có chữ nào phải sửa không.

## 4. Thứ Story 7.2 làm được trước khi `AD` này có

Kiểu xuất xứ khép kín của TM (`''` không vào được `tm_unit`), phép chiếu *của tôi / của người khác* theo AD-47 ⑥, và guard cho mọi nhánh FR117 ở mức cặp TM. Phần D1 dừng chờ `AD`.
