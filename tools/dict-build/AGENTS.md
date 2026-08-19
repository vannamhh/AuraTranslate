<!-- bmad:context -->
<!-- Verified 2026-08-19 against 705d17a. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## tools/dict-build/ — bộ dựng từ điển

Workspace Rust ĐỘC LẬP, không phải thành viên của `src-tauri` và không có workspace cha. `rust-version 1.97.1` cố ý lệch với `src-tauri` (1.85) — đừng đồng bộ hai số. Nguồn thô ở `docs/dics/`.

## Conventions that differ from defaults

- 🔴 `dict-tran-van-chanh.db` phải ở lại một tệp `.db` RIÊNG. Trần Văn Chánh (1999) còn trong bản quyền — tác giả còn sống; giấy CC0 của người số hoá không xoá được bản quyền tác phẩm gốc. Lớp này đóng gói rời chính vì rủi ro đó: FR112 thực thi được bằng cách xoá đúng một tệp. Không gộp lớp, không "hợp nhất cho gọn", không đưa dữ liệu của nó vào `dict-core.db`. (`src-tauri/tests/dict_sources.rs::deleting_any_detachable_layer_keeps_the_whole_lookup_suite_green` canh việc gỡ một lớp vẫn xanh — nó KHÔNG canh việc ai đó gộp lớp.)
- 🔴 Đổi lược đồ ⇒ dựng lại CẢ BỐN tệp `.db` bằng `--layer all` ⇒ bốn SHA-256 mới trong `dict-manifest.toml` ⇒ một release mới. Đúng cả khi nguồn thô của một lớp không đổi một byte. Nguồn sự thật là `dict-manifest.toml` + `src/schema.rs`, KHÔNG phải `README.md` — README còn viết `--layer all` dựng *"đúng ba"* tệp và `build.rs:595` còn viết *"hai lớp gỡ rời"*, cả hai có trước khi `tran-van-chanh` được thêm.
- Ba trường bắt buộc mỗi mục manifest: `url` · `sha256` · `source_version` — `source_version` là phiên bản NGUỒN THÔ, không phải phiên bản tệp `.db`. Không điền giá trị giả để "cho có".
- `is_han` có hai bản chép CỐ Ý (`src/char_idx.rs` và `src-tauri/src/core/dict/mod.rs`) vì hai workspace không import chéo nhau được. Hai cổng canh: `dict_lookup.rs::han_ranges_are_verbatim_from_dict_build_char_idx` đọc tệp này như văn bản rồi so dải CJK, và `dict_boundary.rs::exactly_one_definition_of_is_han_exists_under_src_tauri`. Sửa một bên mà quên bên kia thì tra vào một `char_idx` chưa bao giờ lập chỉ mục ký tự đó ⇒ rỗng, không lỗi.
- Chuỗi ở đây được miễn trừ CÓ TÊN khỏi `check:i18n` Kiểm A nên giữ dấu tiếng Việt thoải mái.

## Known pitfalls

- `npm run check:dict-manifest` chỉ kiểm HÌNH DẠNG — nó phải xanh trên một runner không có byte dữ liệu từ điển nào, nên nó không bao giờ mở tệp `.db`. Nó bắt được một lớp bị rơi mất (đòi đúng 3 `[[detachable]]`, đúng tên); nó KHÔNG bắt được dữ liệu bị trộn giữa các tệp.

<!-- /bmad:context -->
