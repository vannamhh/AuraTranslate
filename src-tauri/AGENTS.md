<!-- bmad:context -->
<!-- Verified 2026-08-24 against b290336. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## src-tauri/ — Rust + Tauri v2

Lõi ứng dụng: mọi quy tắc nghiệp vụ sống ở đây (AD-1). Workspace riêng, `rust-version 1.85` — cố ý lệch với `tools/dict-build` (1.97.1), đừng "đồng bộ" hai số đó. Đọc `SECURITY-NOTES.md` trước khi chạm `tauri.conf.json` hoặc `capabilities/`.

## Conventions that differ from defaults

- 🔴 `panic = "abort"` biến mọi `panic!` thành cái chết của tiến trình: không unwind, không `Drop`, không cơ hội flush WAL, và trên Windows release còn không in ra đâu. `catch_unwind` vô dụng ở đây. Mọi `unwrap()`/`expect()` trong `core/store/**` là một lỗi thiết kế. Mutex khoá bằng `lock().unwrap_or_else(|e| e.into_inner())`; kênh phản hồi gửi bằng `let _ = tx.send(…)`.
- 🔴 Khuôn hai lớp cho mọi bề mặt IPC: ① một hàm thuần nhận `Option<&Store>` — đây là thứ `tests/**` gọi được không cần webview; ② một `#[tauri::command]` mỏng trong module lồng `wire`, lấy `State` qua **`try_state`**. Không `state()`: mở kho có thể đã thất bại và `app.manage()` chưa từng chạy ⇒ `state()` panic ⇒ `abort` giết cả tiến trình. Tên command trên dây LÀ tên hàm, nên vỏ phải sống trong module lồng chứ không mang hậu tố.
- Dựng lỗi IPC CHỈ qua `IpcError::new(code, message_key, params, retryable)` — bốn trường là riêng tư và `new` là chỗ duy nhất `message_key` gặp `params`. Một struct literal đi vòng qua nó biên dịch sạch, qua mọi cổng, rồi đặt nguyên văn `{path}` lên màn hình người dùng. Đừng đặt `#[serde(rename_all = "camelCase")]` lên nó: bốn tên trường là dây, `tests/ipc_contract.rs` khoá lại.
- `message_key` là danh mục ĐÓNG, khai bằng `macro_rules! message_keys!` trong `core/i18n/` — một khai báo sinh `enum` + `ALL` + `as_str` + bảng tham số bắt buộc. Đừng viết tay một danh sách song song: test đồng bộ với `vi.json` chạy TRÊN `ALL`, nên một biến thể quên thêm vào `ALL` cho một test xanh giả. Cùng khuôn cho `scope_kinds!` ở `core/scope/kinds.rs`.
- Không văn bản hiển thị trong Rust, kể cả `impl Display` — các `Display` cho lỗi là chẩn đoán cho log và viết KHÔNG DẤU. Chuỗi literal trong `src-tauri/src/**` viết không dấu (`khong`, không `không`); `tests/**` được miễn trừ CÓ TÊN nên giữ dấu. Comment tiếng Việt có dấu thì hoàn toàn được.
- Module đặt theo KHÁI NIỆM MIỀN, không theo nhóm năng lực — `C1`–`C10` là từ vựng sản phẩm và không xuất hiện trong tên module.
- Test: hai họ tên tệp, hai vai — `*_contract.rs` (hình dạng dây, bảng đã khai, khoá đã đăng ký) và `*_boundary.rs` (module nào KHÔNG được mang từ vựng của module khác). Tên hàm test là một CÂU khẳng định, không `test_foo`: `capabilities_directory_holds_exactly_the_one_reviewed_file`.
- Crate ghim bằng `=`: `"2.6.3"` trong Cargo nghĩa là `^2.6.3`, và lock chỉ giữ số đúng tới lần `cargo update` đầu tiên.
- `[profile.release]` đóng băng (`codegen-units = 1` · `lto` · `opt-level = "s"` · `panic = "abort"` · `strip`) — đổi là làm mọi số đo NFR6 hết so sánh được. `[features]` KHÔNG có `default = [...]`; bộ mặc định rỗng là thứ giữ `tauri-plugin-wdio-webdriver` khỏi `cargo tree`, tức khỏi bản phát hành.

## Known pitfalls

- 🔴 `capabilities/` được phép chứa ĐÚNG MỘT tệp, `main.json`. Tauri nạp **mọi** tệp trong thư mục đó bằng glob `{capabilities}/**/*` — mọi phần mở rộng, có đệ quy — nên một `extra.json` với `"permissions": ["fs:default"]` cấp một bề mặt IPC mới trong khi test đọc `main.json` vẫn xanh. Cưỡng chế: `tests/config_invariants.rs::capabilities_directory_holds_exactly_the_one_reviewed_file`.
- Tập quyền là tối thiểu thật, đúng ba mục (`core:path:default` · `core:event:default` · `core:resources:default`), không phải bundle `core:default`. Thêm một quyền là một quyết định kiến trúc.
- CSP giữ nguyên, không nới: không CDN, không font ngoài, không ảnh ngoài — đây là lý do FR127 tải ảnh về `.atproj` thay vì giữ link. `assetProtocol.scope` hôm nay là `$RESOURCE/fonts/**` và chỉ thế.
- 🔴 Không mở cổng LẮNG NGHE nào trong bản phát hành (AD-45). Công cụ cần máy chủ phải đi qua HAI lớp cùng lúc: `optional = true` + feature ngoài `default`, **và** `#[cfg(debug_assertions)]` ở chỗ nối. Một `cfg` một mình không đủ — nó loại **mã**, không loại **phụ thuộc**.
- Streaming AI đi qua Tauri Channel API, không qua event rời, và KHÔNG có client SSE tự kết nối lại (AD-22): auto-reconnect tạo một yêu cầu mới hoàn toàn, nên với BYOK người dùng bị tính phí hai lần. Mọi lời gọi AI phải huỷ được giữa chừng.
- Khoá API không bao giờ đi qua IPC — crate `keyring` trực tiếp trong Rust; frontend chỉ biết "đã cấu hình / chưa cấu hình".
- Mọi lệnh ghi đi qua `store::Writer` nối tiếp của kho tương ứng; không module nào tự mở kết nối ghi. `PRAGMA wal_autocheckpoint = 0` — thời điểm checkpoint là quyết định của ứng dụng.
- `library-index.db` và `meta.json` là DẪN XUẤT: chỉ `Indexer` ghi `library-index.db`, và chỉ sau khi `.atproj` đã ghi xong. Xoá chúng phải luôn là thao tác an toàn.
- Lược đồ có phiên bản, di trú CHỈ TIẾN. Gặp phiên bản mới hơn ứng dụng ⇒ từ chối mở và báo rõ, KHÔNG BAO GIỜ ghi vào. Di trú chạy trong một giao dịch, sau khi đã sao lưu.
- Hợp đồng flush Editor (AD-35): idle 2 s · trần cứng 5 s KHÔNG reset bởi phím gõ · xác nhận · rời segment · đóng Tác phẩm. Một debounce thuần không bao giờ kích hoạt khi người dùng gõ liên tục — mất không giới hạn công việc trong khi vẫn "đúng đặc tả auto-save". Một flush chỉ xong sau khi đã ghi vào WAL. Thao tác RỜI RẠC (FR94, FR58) ghi NGAY, không qua bộ đệm gõ.
- Không hợp nhất nguồn từ điển, ở bất kỳ đâu: kết quả trả về theo từng nguồn, giữ nguyên bất đồng, cột `source` bắt buộc trên mọi bản ghi nghĩa. Cũng không hợp nhất `zh` với `en`.
- Không cơ chế nào tự ghi vào Glossary — quét khi nhập và thu hoạch từ bản review ghi vào bảng chờ riêng; chỉ thao tác duyệt của người dùng mới chuyển sang Glossary.
- Vị từ điều phối zh/en là HÌNH DẠNG CHUỖI TRUY VẤN, không phải ngôn ngữ của Tác phẩm: bôi đen `API` trong một truyện tiếng Trung mà lọc `lang='zh'` cho 0 hàng dù mục `API` có thật. Hạ chữ thường là THÊM một khoá, không THAY khoá gốc — 1.635 đầu mục tiếng Anh mang chữ hoa có nghĩa.
- `LIKE` bị cấm trên đường nóng tra cứu — đo 20–50 ms.
- ⚠️ Hai bất biến CHƯA có phép kiểm, đừng tin là đã được canh: AD-13 (không module nào ngoài `ai/` phụ thuộc `ai/`) hoãn tới Story 4.1 — `core/ai/` hôm nay là stub; AD-41 (phạm vi mạng) không được framework cưỡng chế vì capabilities của Tauri là khai báo tĩnh lúc build, và bộ test riêng của nó chưa tồn tại — `core/webimport/` cũng là stub.
- `trim()` của SQLite chỉ cắt **dấu cách ASCII**, nên `CHECK (trim(x) <> '')` KHÔNG chặn tab, xuống dòng, NBSP hay U+3000 — đo 2026-08-19 trên SQLite 3.53.4, và rào rỗng mà spec Story 3.1 viết sẵn đã thủng bảy đường. Rào rỗng trong DDL phải liệt TRỌN 25 điểm mã `White_Space` (`GLOSSARY_ENTRY_DDL`), đúng tập mà `str::trim()` của Rust cắt; thêm một ký tự vào một lớp thì phải thêm vào lớp kia CÙNG LƯỢT. Không cổng nào canh cặp này.

<!-- /bmad:context -->
