# Handoff: pha Rust của 5-20

## Chữ ký cuối (`src-tauri/`)
- `core/library/indexer.rs`: `pub fn classify_query(&str) -> SearchMode` (tự `trim`; `Lenient` khi có chữ ASCII, không có dấu tổ hợp U+0300..036F và `fold_diacritics_case_preserving(q) == q`; ngoài ra `Exact`). Dấu tổ hợp rời phải kiểm riêng vì `fold_one_char_case_preserving` để nguyên dấu đứng một mình.
- `Indexer::search(&self, query, limit)` và `Indexer::search_filtered(&self, query, limit, filter)`; chế độ gọi `classify_query(&trimmed)` đúng một chỗ trong `search_filtered`.
- `SearchMode` chỉ còn `Exact`/`Lenient` + `as_str`; đã xoá `ALL`, `from_wire`, `Default`.
- Nhánh khoan dung: hàm `merge_exact_first` (hit chính xác theo thứ tự, rồi hit `_nd` có rowid ngoài tập, cắt chung ở `limit`); `_nd` lấy `limit + 1 + số hit chính xác`; `truncated` = nửa chính xác bị cắt HOẶC còn dư sau gộp. Nhãn `match_kind` vẫn theo tập rowid chính xác.
- `commands/library.rs`: `search_library(indexer, query, limit)`, `search_library_scoped(indexer, query, limit, &SearchArgs)`, `wire::library_search` không còn `mode`. Đã xoá `unknown_search_mode`; `core/i18n/mod.rs` xoá `MessageKey::LibraryUnknownSearchMode`.
- `SearchReport.mode` (wire: chuỗi `"exact"`/`"lenient"`) nay là chế độ phân loại; `effective_mode`, `widened` không đổi.

## Test đã đổi
- `library_index_contract.rs`: gỡ đối số chế độ khỏi mọi lời gọi (script cơ học) và khỏi `rebuild_and_search`; xoá ca khứ hồi `every_search_mode_variant_round_trips_through_as_str_and_from_wire`, thay bằng `query_classification_follows_the_ad_61_table` (chín dòng + `  chi  `). Viết lại bằng truy vấn có dấu: `a_zero_hit_exact_query_widens_…` (`mả của`), `a_two_diacritic_query_widens_…` và ca migration 6→7 (`nguyên huê`), `an_empty_index_never_widens_…` (`bất kỳ truy vấn nào`), `widening_that_still_finds_nothing_…` (`từ vô nghĩa không tồn tại zzqq`, thêm nhánh `zzqq` gõ không dấu 0 hit không `widened`), `widening_stays_inside_the_filter_…` (`dà nang`), bất biến `the_widened_flag_always_equals_…` (bảng 4 ca có `mode` mong đợi). Thêm `a_low_ordered_exact_hit_survives_a_tight_cap_in_the_lenient_pass` (6c).
- Bàn đo `bench_p95_of_a_library_search_over_five_thousand_chapters` (5.9, tổng hợp, `#[ignore]`) đã đổi đủ để còn đúng: `source_latin_unique` Lenient, `target_auto_widen_truncated` dùng `mả của tôi`. KHÔNG chạy.
- `bench_p95_of_a_library_search_over_the_story_6_18_library` (5 ca) CHƯA đụng: compile được nhưng `assert_case` sẽ đỏ khi chạy (`requested_mode` cũ). Việc của pha Tests + đo theo plan (`target_typed_lenient_truncated`, `mả của tôi`, `source_latin_unique` Lenient).
- `library_commands_contract.rs`: xoá `an_unknown_search_mode_…`; `no_search_mode_defaults_to_exact` thành `the_search_mode_on_the_wire_is_derived_from_the_query`; thêm `typing_chi_finds_both_with_the_exact_word_first_while_typing_chi_with_hook_finds_only_it` (6b, qua `search_library`, hai Tác phẩm "chi phí" và "chỉ đường"); đổi chữ ký các lời gọi còn lại.
- `ipc_contract.rs`: bỏ `mode: Option<String>` khỏi danh sách tham số và thêm khẳng định vắng `mode: Option<String>` trong `commands/library.rs`.
- `project_contract.rs`: hai lời gọi `.search` đổi chữ ký.

## Kiểm
- Xanh: `cargo test --test library_index_contract --test library_commands_contract --test ipc_contract --test project_contract` (105+32+48+155), `cargo clippy --all-targets`, `cargo fmt`.
- Đối chứng đỏ (gỡ thật, đã khôi phục): (1) `let mode = classify_query(..)` đổi thành `SearchMode::Exact` ⇒ ca `chi` đỏ vì chỉ ra 1 hit thay vì 2 (thiếu "chỉ"); (2) `merge_exact_first` trả danh sách `_nd` trần ⇒ ca trần đỏ (`hits[0].match_kind` Lenient, mong Exact).

## Còn lại cho pha Webview
- `src/i18n/vi.json` còn khoá `err.library.unknown_search_mode` (khoá Rust đã xoá); `check:i18n` có thể đỏ đến khi gỡ.
- Cổng `ipc_contract` ghim wire struct `SearchReport` vẫn có `mode` (chuỗi); Webview giữ `SearchReport.mode`.
- Chưa chạy bộ đủ `cargo test` (không chạm `lib.rs`/đăng ký lệnh).
