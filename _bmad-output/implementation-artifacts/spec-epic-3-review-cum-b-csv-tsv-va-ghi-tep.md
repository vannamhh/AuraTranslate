---
title: 'Cụm B — chín chỗ hỏng ở đường phân tích CSV/TSV và đường ghi tệp'
type: 'bugfix'
created: '2026-08-25'
status: 'done'
review_loop_iteration: 0
baseline_commit: '3e76711f18b5a7d9ac261ec97626a705f37bf8a3'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-10-xuat-va-nhap-glossary-qua-csv-tsv.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Vòng rà Epic 3 (`a2eaf7c~1..HEAD`, ba lăng kính) trả 55 phát hiện; **mười** thuộc đường xuất/nhập CSV-TSV mà Story 3.10 dựng. Đã xác minh lại từng mục trên HEAD: **tám còn đúng**, **một bị bác vì tiền đề sai nhưng lỗ thật vẫn ở đúng dòng đó bằng một ký tự khác** (⇒ **chín** mục vá), và mục thứ mười là điều tra bối cảnh cho mục `.tmp`. Nặng nhất là ba thứ hỏng **im lặng**: một tệp xuất mở bằng bảng tính chạy được công thức người khác cài vào ô; một ngoặc kép không đóng nuốt mọi hàng đúng phía sau vào **một** thông báo lỗi trỏ sai chỗ; và một `source_term` chỉ gồm ký tự zero-width lọt **cả hai** lớp phòng thủ để thành một mục Glossary vô hình.

**Approach:** Vá tại chỗ theo đúng khuôn mà chính hai tệp đã đặt — không đổi hình dạng công khai của `parse`/`classify`/`render_tier`, không thêm phụ thuộc, không bước di trú. Hai lớp lỗi chưa có tên thì **cấp tên** (hai biến thể `ParseIssue` mới), và mỗi mục vá phải đi kèm một phép đối chứng **gỡ chỗ nối ⇒ đỏ**.

## Boundaries & Constraints

**Always:**
- 🔴 **Mỗi mục vá phải kèm một ca test mà GỠ bản vá ra thì ca đó ĐỎ.** Không đủ khi bộ test cũ vẫn xanh — đó chính là lớp lỗi Epic 3 dính năm lần trong bảy ngày. Ghi lại con số đối chứng thật, không suy.
- 🔴 **Hai biến thể `ParseIssue` mới phải đi TRỌN bộ ba trong CÙNG lượt**: biến thể ⇒ nhánh `impl From<ParseIssue> for IpcError` (`exchange.rs:232`) ⇒ khoá trong `message_keys!` (`core/i18n/mod.rs`) ⇒ câu trong `src/i18n/vi.json`. Cổng `glossary_exchange_contract.rs::every_parse_issue_variant_maps_to_a_declared_message_key` canh cặp đầu, `ipc_contract.rs` canh cặp sau — thiếu một mắt là một cổng đỏ, đừng nới cổng.
- 🔴 **`params` mang DỮ LIỆU, không mang CÂU** (AD-21): `{line}`, `{column}`. Chuỗi literal trong `src-tauri/src/**` viết **KHÔNG DẤU**; doc-comment giữ dấu.
- 🔴 **Không thêm phụ thuộc nào** (NFR15 + §Always của Story 3.10: parser là tập con tự viết). `uuid = "=1.24.0"` (feature `v4`) **đã có sẵn** trong `Cargo.toml` và đã dùng ở `commands/project.rs:24` — dùng lại nó **không** phải một phụ thuộc mới. Không `csv`, không `tempfile`.
- 🔴 **`parse` giữ nguyên hợp đồng "0 lượt ghi khi có lỗi"** và giữ nguyên khả năng gom nhiều lỗi một lượt mà ca `…two_errors…` hiện có đang khoá.
- **Bản vá zero-width chỉ đứng ở lớp Rust của ĐƯỜNG NHẬP.** Không sửa `GLOSSARY_ENTRY_DDL`, không bước di trú — bảng 25 điểm mã `White_Space` trong `CHECK` là **đúng** cho tập nó khai (đã đo 2026-08-19), lỗ nằm ở chỗ tập đó và `str::trim()` cùng **không** phủ ký tự zero-width. Vế SQL và vế `insert_manual_entry` ghi thành nợ có chủ.
- **Tên tệp tạm chỉ đổi ở `exchange_io.rs`, không đụng `meta.rs::write_atomic`** — tiền lệ đó đúng cho ngữ cảnh của nó (đường nội bộ cố định, đã nối tiếp hoá qua `Store`), và mở rộng nó là một phạm vi khác.

**Ask First:**
- Nếu bản vá `logical_lines` (đếm `\r` trần) làm **bất kỳ** ca BOM/CRLF hiện có trong `glossary_exchange_contract.rs` đỏ: **DỪNG và trình lỗi**. Đỏ ở đó nghĩa là một ca đã chốt cứng cách đếm dòng hiện tại như một hành vi mong muốn, và đó là một câu hỏi cho Ice chứ không phải một ca cần sửa.
- Nếu việc dò dấu phân cách trên **ô đã tách** hoá ra cần gọi `split_fields` hai lượt (một lượt cho mỗi ứng viên) và lượt nào cũng có thể ném lỗi trước khi biết delimiter đúng — trình hai hình dạng kèm số đo, đừng tự chọn.
- Nếu tách `seen` ra khỏi `row_ok` làm đổi **thứ tự** lỗi mà một ca hiện có đang khẳng định.

**Never:**
- Không đụng 39 phát hiện của cụm C, D, E, F — chúng có mục nợ riêng, chủ riêng.
- Không đụng `store.rs` (đó là cụm C, và sổ nợ đã chốt thứ tự B trước C).
- Không thêm `#[tauri::command]` mới, không đổi chữ ký hàm thuần nào đang được `commands/glossary.rs` gọi.
- Không hạ ngưỡng, không `#[allow]`, không chuyển một ca sang danh sách loại trừ để cổng hết đỏ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| **①** Xuất ô bắt đầu bằng công thức | `translation` = `=1+1` hoặc `+A1` `-A1` `@SUM(…)` | Ô ra tệp mang tiền tố vô hiệu hoá; đọc lại bằng chính `parse` của kho trả **đúng chuỗi gốc** | N/A |
| **②** Ngoặc kép mở không bao giờ đóng | `…\n"Mộ Dung,ten\n<còn 300 hàng đúng>` | Lỗi **có tên riêng**, mang số dòng nơi ô mở ngoặc kép | 0 lượt ghi. **Không** biến 300 hàng sau thành một `CellCountMismatch` |
| **③** `\r` trần bên trong ô đã bọc | Ô `"dong1\rdong2"` rồi một hàng lỗi ở dưới | Số dòng của lỗi phía sau **khớp số dòng người dùng đếm trong trình soạn thảo** | 0 lượt ghi |
| **④** Header TSV có dấu phẩy trong ngoặc kép | Tiêu đề TSV, một ô là `"note, extra"` | Nhận là TSV, nhập bình thường | **Không** `DelimiterUnresolved` |
| **⑤** Hai cột trùng tên đã biết | Tiêu đề có `translation` hai lần | Lỗi **có tên riêng**, mang tên cột trùng | 0 lượt ghi. Không im lặng lấy cột đầu |
| **⑥** `source_term` chỉ gồm ký tự zero-width | Ô = `U+200B` (hoặc `U+FEFF` giữa tệp) | `BlankSourceTerm` mang số dòng — cùng khoá, cùng câu như ô rỗng | 0 lượt ghi |
| **⑥b** `source_term` hợp lệ có zero-width kèm | `萧炎` + `U+200B` ở cuối | **Nhận**, và ký tự zero-width bị cắt khỏi giá trị lưu xuống | N/A |
| **⑦** Trùng `source_term` mà lần đầu đã bị bác vì lý do khác | Dòng 2: `X` + category lạ · Dòng 5: `X` hợp lệ | Báo **cả** lỗi category dòng 2 **và** lỗi trùng (2, 5) | 0 lượt ghi |
| **⑧** Tệp lớn lên sau khi đo, trước khi đọc | `metadata` nói 1 MiB, `read` gặp 20 MiB | Từ chối bằng `ImportFileTooLarge`; **bộ nhớ nạp vào không vượt trần** | 0 lượt ghi |
| **⑨** Hai lượt xuất song song cùng một đích | Hai lần ghi cùng `out.csv` | Mỗi lượt dùng một tệp tạm **riêng**; đích cuối là **một** trong hai bản trọn vẹn, không phải bản trộn | Lỗi ⇒ dọn đúng tệp tạm của **chính** lượt đó, không dọn của lượt kia |
| Vòng tròn xuất→nhập (hồi quy) | Tệp vừa xuất từ một tầng N mục | Vẫn nhập lại đủ N mục, 5 trường giữ nguyên | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/glossary/exchange.rs` (773 dòng) — module định dạng **thuần**, `&str` vào / `String` ra, không chạm hệ thống tệp. Bảy mục vá nằm ở đây:
  - `:74-99` `field_needs_quoting` / `quote_field` / `render_field` — chỉ rào RFC 4180 (delimiter, `"`, `\n`, `\r`); **0 nhánh** kiểm ký tự đầu ô ⇒ mục ①.
  - `:144-194` `enum ParseIssue` — **bảy** biến thể hôm nay, mỗi biến thể mang dữ liệu (`line`, `column`, `value`). Khuôn cho hai biến thể mới.
  - `:196-227` `impl Display` (không dấu, chẩn đoán log) · `:232` `impl From<ParseIssue> for IpcError` — `match` **exhaustive**, nên thêm biến thể làm nó đỏ ở khâu biên dịch. Đó là hàng rào, không phải phiền toái.
  - `:393-433` `split_first_logical_line` — vòng `in_quotes`; `continue` nuốt `\n`/`\r` làm nội dung, **không có** phép phát hiện EOF-khi-còn-mở ⇒ mục ②. Ở **ngoài** ngoặc kép (`:421-426`) `\r` trần **là** một ranh giới dòng.
  - `:439-452` `logical_lines` — `:447` `line_no += line.matches('\n').count() + 1` chỉ đếm `\n` ⇒ lệch với `:421-426` ⇒ mục ③.
  - `:520-529` dò delimiter — `header_text.contains(',')` / `.contains('\t')` chạy trên văn bản **thô**, trước `split_fields` ⇒ mục ④.
  - `:536-549` dựng chỉ số cột — `header.iter().position(|c| c == …)` lấy khớp **đầu**; `ignored_columns` lọc theo `!known.contains(…)` nên tên trùng **đã biết** không lọt vào đó ⇒ mục ⑤.
  - `:584-588` rào `source_term` rỗng — gọi `str::trim()` ⇒ mục ⑥ (xem §Design Notes: lý do trong sổ nợ **sai**, lỗ thì thật).
  - `:598-652` vòng đọc hàng — `:652` `seen.insert(…)` nằm **sau** `if !row_ok { continue }` ⇒ mục ⑦.
- `src-tauri/src/core/glossary/exchange_io.rs` (276 dòng) — nửa I/O, dựng ở Story 3.10b. Hai mục vá:
  - `:41-62` `read_import_file` — `metadata` ⇒ so `MAX_GLOSSARY_IMPORT_BYTES` (16 MiB) ⇒ `std::fs::read` **không chặn** ⇒ mục ⑧. Khuôn anh em cùng thứ tự: `core::segment::import` (trần 100 MiB).
  - `:64-127` `write_export_file` — doc-comment `:64` tự khai là "khuôn chép `core/library/meta.rs::write_atomic`", và `:68-70` tự nhận *"kho chưa có tiền lệ ghi ra một đường dẫn TUỲ Ý người dùng chọn"*. Tên tạm là `<tên>.tmp`, **0** hậu tố duy nhất ⇒ mục ⑨.
  - `:129-276` `#[cfg(test)] mod tests` — **6** ca nội bộ (vượt trần, phi-UTF-8, cắt BOM, ghi xong không để `.tmp`, dọn `.tmp` ở cả hai nhánh lỗi). Chỗ đặt ca mới cho ⑧/⑨.
- `src-tauri/src/core/library/meta.rs:131` `write_atomic` — tiền lệ được viện dẫn. **Đọc để đối chiếu, KHÔNG sửa.** Nó cũng đặt tên tạm không hậu tố duy nhất; khác biệt là đích **nội bộ cố định**.
- `src-tauri/src/core/i18n/mod.rs:306-372` — khối khoá Story 3.10/3.10b, kèm doc-comment nói rõ *"BẢY khoá đầu là lỗi PHÂN TÍCH… mỗi cái ứng với đúng một hàng 0 lượt ghi"*. Hai khoá mới vào đây, cạnh bảy khoá đó, và **doc-comment "TÁM khoá, và đúng tám" phải sửa TẠI CHỖ kèm 🔵 + ngày**.
- `src/i18n/vi.json:29-39` — mười một câu Glossary hiện có; hai câu mới đi cùng khuôn *"… — chưa có gì được nhập."*
- `src-tauri/tests/glossary_exchange_contract.rs` — **41** ca. Nửa thuần (`render_tier`/`parse`/`classify`) + nửa ghi (`export_tier`/`import_into_tier`). **Không** chạm hệ thống tệp. Ca mới cho ①–⑦ vào đây.
- `src-tauri/tests/glossary_import_dialog_contract.rs` — **21** ca, I/O tệp thật; tệp tạm dựng bằng `std::env::temp_dir()` + `AtomicU64` (khuôn tự viết, **không** crate `tempfile`).
- `src-tauri/src/commands/glossary.rs` — đường gọi, **chỉ đọc**: `:1274` `wire::glossary_export_tier` → `:682` hàm thuần → `store.rs::export_tier` → `exchange_io::write_export_file`; `:1348` `wire::glossary_open_import_preview` → `:751` hàm thuần → `exchange_io::read_import_file` → `exchange::parse` → `exchange::classify`.
- `src-tauri/src/core/store/schema.rs:200-240` + `:300` `GLOSSARY_ENTRY_DDL` — **chỉ đọc**. Bảng 25 điểm mã `White_Space` và lý do đủ-25; bằng chứng cho ranh giới của mục ⑥.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/glossary/exchange.rs` — thêm hai biến thể `ParseIssue` (`UnterminatedQuotedField { line }`, `DuplicateColumn { column }`) kèm nhánh `Display` và nhánh `From<…> for IpcError` — hai lớp lỗi ② và ⑤ hôm nay **không có tên**, nên chúng mượn tên của lỗi khác và trỏ người dùng đi sai chỗ.
- [x] `src-tauri/src/core/glossary/exchange.rs` — vá đường **sinh** (①): rào ô bắt đầu bằng `=` `+` `-` `@` — một tệp Glossary là dữ liệu người khác gửi tới, và bảng tính chạy ô đó như công thức.
- [x] `src-tauri/src/core/glossary/exchange.rs` — vá đường **phân tích** (②③④⑤⑥⑦) trong `split_first_logical_line`, `logical_lines`, khối dò delimiter, khối dựng chỉ số cột, rào `source_term`, và vòng đọc hàng — sáu chỗ, cùng một tệp, cùng một lượt vì chúng chia nhau `logical_lines` và thứ tự lỗi.
- [x] `src-tauri/src/core/glossary/exchange_io.rs` — vá ⑧ (đọc có chặn thật thay vì tin `metadata`) và ⑨ (tên tạm mang hậu tố duy nhất từ `std::process::id()` + `uuid::Uuid::new_v4()`), và **sửa TẠI CHỖ** doc-comment `:64-73` kèm 🔵 + ngày: mệnh đề "khuôn chép `write_atomic`" nay chỉ còn đúng một phần.
- [x] `src-tauri/src/core/i18n/mod.rs` — hai khoá mới cạnh bảy khoá phân tích; sửa TẠI CHỖ doc-comment "TÁM khoá, và đúng tám" kèm 🔵 + ngày.
- [x] `src/i18n/vi.json` — hai câu mới, cùng khuôn kết *"— chưa có gì được nhập."*
- [x] `src-tauri/tests/glossary_exchange_contract.rs` — ca cho ①②③④⑤⑥⑥b⑦, mỗi ca kèm phép đối chứng gỡ-chỗ-nối đã chạy thật.
- [x] `src-tauri/tests/glossary_import_dialog_contract.rs` và/hoặc `mod tests` nội bộ của `exchange_io.rs` — ca cho ⑧ và ⑨.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — đóng mục cụm B bằng chữ (`→ ✅` / `→ 🟡` kèm phần còn hở), và ghi **một mục nợ mới**: vế SQL + `insert_manual_entry` của lỗ zero-width, chủ là story đầu tiên chạm `GLOSSARY_ENTRY_DDL`.

**Acceptance Criteria:**
- Given một tệp xuất có ô `=1+1`, when mở bằng bảng tính, then ô hiển thị **văn bản** `=1+1` chứ không chạy công thức; và when nhập lại chính tệp đó bằng `parse` của kho, then giá trị đọc ra bằng **đúng từng byte** giá trị đã xuất.
- Given hai biến thể `ParseIssue` mới, when chạy `glossary_exchange_contract.rs::every_parse_issue_variant_maps_to_a_declared_message_key` và `ipc_contract.rs`, then cả hai xanh — và when xoá một trong hai khoá mới khỏi `message_keys!`, then một trong hai cổng đó **đỏ**.
- Given mỗi mục vá ①–⑨, when gỡ riêng bản vá đó ra và chạy bộ test, then **có ít nhất một ca đỏ trỏ đúng mục đó**; số ca đỏ của từng lượt gỡ được ghi lại thành số thật.
- Given toàn bộ bản vá, when chạy `.githooks/pre-push`, then exit 0 — và số ca `cargo test --locked` **tăng**, với con số trước/sau ghi lại.
- Given hợp đồng công khai của `exchange.rs`, when so chữ ký `parse`/`classify`/`render_tier` trước và sau, then **không đổi** — `commands/glossary.rs` và `store.rs` không phải sửa một dòng nào để biên dịch.

## Spec Change Log

### 2026-08-25 — lượt dựng: chín đối chứng gỡ chỗ nối, tất cả chạy thật

**Chín đối chứng GỠ CHỖ NỐI — mỗi lượt khôi phục lại ngay sau khi đo.**

| Gỡ chỗ nối | Kết quả |
|---|---|
| ① `needs_formula_guard` luôn trả `false` | **ĐỎ** — 1 failed (`a_cell_starting_with_a_formula_trigger_character_is_neutralized_on_export_and_recovered_verbatim_on_import`) |
| ② `split_first_logical_line` luôn trả cờ `unterminated = false` | **ĐỎ** — 2 failed (`an_unterminated_quoted_field_is_a_named_error_...`, `an_unterminated_quoted_field_in_the_header_row_itself_is_also_a_named_error`) |
| ③ `logical_lines` đếm `line.matches('\n').count()` (bản cũ, chỉ `\n`) thay vì `count_line_breaks` | **ĐỎ** — 1 failed (`a_bare_cr_inside_a_quoted_field_advances_the_line_number_for_rows_after_it`, mong dòng 4, được dòng 3) |
| ④ dò delimiter bằng `header_text.contains(target)` (văn bản thô) thay vì `unquoted_char_present` | **ĐỎ** — 1 failed (`a_quoted_comma_inside_a_tsv_header_cell_does_not_confuse_delimiter_detection`, `Err([DelimiterUnresolved])` oan) |
| ⑤ bỏ khối `known_column_counts`/`duplicate_column_issues` | **ĐỎ** — 1 failed (`two_header_columns_sharing_a_known_name_are_refused_naming_the_duplicated_column`, cột `translation` thứ hai biến mất im lặng) |
| ⑥ bỏ `strip_zero_width` khỏi đường trích `source_term` | **ĐỎ** — 2 failed (`a_source_term_containing_only_a_zero_width_character_is_refused_as_blank`, `a_valid_source_term_with_a_trailing_zero_width_character_is_accepted_with_it_stripped`) |
| ⑦ trả `seen` về đọc-trước/ghi-sau-khi-`row_ok` (bản cũ) | **ĐỎ** — 1 failed (`a_duplicate_whose_first_occurrence_was_rejected_for_an_unrelated_reason_is_still_flagged_as_a_duplicate`; ba ca trùng CŨ khác vẫn xanh — không lệch thứ tự lỗi) |
| ⑧ khôi phục `metadata` ⇒ so ⇒ `std::fs::read` không chặn | **ĐỎ** — 1 failed (`read_import_file_never_reads_more_than_the_cap_plus_one_byte_...`; `size` trả về đổi từ `LIMIT + 1` = 16.777.217 sang kích thước THẬT ~134.217.729) |
| ⑨ trả tên tạm về `<tên>.tmp` trần (bỏ hậu tố pid+uuid) | **ĐỎ** — 5/5 lượt chạy lại đều `FAILED` (`two_concurrent_exports_to_the_same_destination_...`; cả hai luồng cùng ghi/`rename` một tệp tạm CHUNG ⇒ `ExportWriteFailed` "No such file or directory", không phải một cửa sổ đua hiếm gặp — đỏ TẤT ĐỊNH, không chỉ "tái hiện được") |

Khôi phục trọn sau MỖI lượt ⇒ `cargo test --locked` **691** ca, 0 failed (đọc `$?` thật). `.githooks/
pre-push` exit 0 hai lần trong lượt này (trước và sau chín đối chứng), ~110-157s mỗi lượt.

**Số đo trước/sau (AC4):** `cargo test --locked` **680 → 691** (+11: chín ca cho ①-⑨, cộng một ca
phụ cho ② ở ngay hàng tiêu đề, cộng một biến thể U+FEFF-giữa-tệp gộp trong ca ⑥). `npm run
check:i18n` 382 → **384** khoá `vi.json` (hai khoá `import_unterminated_quoted_field` +
`import_duplicate_column`), 0 miễn trừ mới. `npm run check:deps` xanh, **0** crate mới (330 crate
Rust đã quét, `uuid` đã có sẵn từ Story 3.10b).

**Một chỗ lệch khỏi §Tasks, ghi ra:** mục ⑧/⑨ dùng `mod tests` nội bộ của `exchange_io.rs` (nhánh
"và/hoặc" mà §Tasks cho phép), không chạm `glossary_import_dialog_contract.rs` — hai ca mới ở đó
cần I/O tệp thật (một tệp thưa 8× trần cho ⑧, hai luồng ghi đồng thời cho ⑨) khớp đúng khuôn `mod
tests` hiện có của module đó hơn là khuôn 21 ca I/O tệp thật của `glossary_import_dialog_contract.rs`
(vốn đi qua lớp `commands::glossary`, không gọi thẳng `exchange_io::{read_import_file,
write_export_file}`).

**Ba mục nợ mới mở qua lượt này** (chi tiết trong `deferred-work.md`): nghiệm thu ① bằng mắt trên
một bảng tính thật (môi trường thi hành không có GUI); vế SQL (`GLOSSARY_ENTRY_DDL`'s `CHECK`) +
vế `insert_manual_entry` của cùng lỗ zero-width (§Boundaries đã giao có chủ cho story đầu tiên
chạm `GLOSSARY_ENTRY_DDL`).

## Design Notes

**Vì sao mục ⑥ ở lại spec dù sổ nợ nói sai.** Sổ nợ khai *"`str::trim()` không cắt U+00A0"*. **Đo được 2026-08-25** (`rustc -O`, chạy thật):

```
U+00A0  trim() ⇒ ""            rỗng sau trim: true    is_whitespace: true
U+3000  trim() ⇒ ""            rỗng sau trim: true    is_whitespace: true
U+200B  trim() ⇒ "\u{200b}"    rỗng sau trim: FALSE   is_whitespace: FALSE
U+FEFF  trim() ⇒ "\u{feff}"    rỗng sau trim: FALSE   is_whitespace: FALSE
```

⇒ Tiền đề **sai**: `str::trim()` cắt U+00A0 sạch, đúng như `schema.rs:208-238` đã khai và đã đo. Nhưng **kết luận đúng bằng một ký tự khác**: U+200B và U+FEFF không mang thuộc tính `White_Space`, nên chúng lọt **cả** `str::trim()` phía Rust **lẫn** bảng 25 điểm mã của `CHECK` phía SQL — `trim(source_term, <25 ký tự>) <> ''` **PASS**, hàng vào DB, và người dùng có một mục Glossary không nhìn thấy được, không xoá được bằng cách gõ lại thuật ngữ. `strip_bom` chỉ cắt U+FEFF ở **đầu tệp**, nên một U+FEFF giữa tệp (thường gặp khi ai đó nối hai tệp xuất) đi thẳng vào một ô.

⚠️ **Giới hạn có tên của bản vá này:** nó đóng tập **viết ra được** — U+200B, U+200C, U+200D, U+2060, U+FEFF — **không** phủ trọn thuộc tính Unicode `Cf`. Rust std không mang bảng category, và kéo một crate Unicode về là một cổng NFR15. Vế đó là một mục nợ, không phải một dòng vá.

**Vì sao ⑨ dùng `uuid` chứ không phải một bộ đếm.** `std::process::id()` một mình không đủ: hai lượt xuất trong **cùng** tiến trình (hai lần bấm Xuất) chia nhau cùng pid. Một `AtomicU64` một mình cũng không đủ: nó chết theo tiến trình, nên một lượt xuất của phiên trước để lại `.tmp` mồ côi vẫn va. `uuid::Uuid::new_v4()` đã có sẵn trong `Cargo.toml` (`=1.24.0`, feature `v4`) và đã dùng ở `commands/project.rs:24` — dùng lại nó tốn **một** dòng `use` và **0** cổng giấy phép.

**Vì sao không đụng `meta.rs::write_atomic` cùng lượt.** Nó **cũng** không có hậu tố duy nhất, nên thoạt nhìn cùng một lỗi. Nhưng đích của nó là `meta.json` cạnh `.atproj/` — một đường **nội bộ cố định** mà mọi lượt ghi đi qua `Store` (một writer nối tiếp). Đích của `write_export_file` là đường **người dùng vừa chọn trong hộp thoại**, không writer nào nối tiếp hoá. Tiền lệ đúng; phạm vi của nó không phủ ca mới — và chính doc-comment `exchange_io.rs:68-70` đã viết ra sự khác biệt đó rồi vẫn sao y tên tạm.

## Verification

**Commands:**
- `cd src-tauri && cargo test --locked` -- expected: exit 0; số ca **tăng** so với 584 của lượt đo 2026-08-24, ghi lại con số thật trước/sau.
- `npm run build` -- expected: exit 0. **Chạy TRƯỚC `cargo test`** — thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch, không ở một assert.
- `.githooks/pre-push` -- expected: exit 0 (11 cổng → vitest → build → `cargo test --locked`). ⚠️ Chạy trên macOS của Ice; nó **không** nói gì về nửa Windows — đọc lượt CI trước khi kết luận là xanh.

**Manual checks (if no CLI):**
- **Chín phép đối chứng gỡ-chỗ-nối, chạy thật, một mục một lượt:** gỡ riêng từng bản vá ①–⑨ ra, chạy bộ test, ghi lại **tên ca đỏ** và **số ca đỏ**. Một mục mà bộ test vẫn xanh trọn nghĩa là ca của nó chưa chạm bề mặt — sửa ca, không sửa kết luận.
- **Đối chứng cổng khoá lỗi:** xoá một khoá mới khỏi `message_keys!` ⇒ một trong hai cổng phải đỏ; khôi phục rồi xoá câu tương ứng khỏi `vi.json` ⇒ `ipc_contract.rs` phải đỏ.
- **Nghiệm thu ① bằng mắt trên một bảng tính thật** (mở tệp xuất bằng Numbers/Excel/LibreOffice) — ⚠️ vế này **không** nghiệm thu được bằng `cargo test`; nếu lượt thi hành không mở được một bảng tính thật thì ghi vào `deferred-work.md` kèm chủ, **không** đánh dấu đạt bằng suy luận.

## Suggested Review Order

**Rào công thức — mục ①, chỗ dễ đọc sai nhất**

- Vị từ CHUNG cho cả hai chiều; bất đối xứng ở đây là nguồn của mất dữ liệu.
  [`exchange.rs:122`](../../src-tauri/src/core/glossary/exchange.rs#L122)

- Xuất thêm đúng một `'`; nhập bỏ đúng một — khuôn RFC 4180 nhân đôi `"`.
  [`exchange.rs:137`](../../src-tauri/src/core/glossary/exchange.rs#L137)

- Mệnh đề "không phủ được nếu không thêm cột" đã sai; sửa tại chỗ, không xoá.
  [`exchange.rs:84`](../../src-tauri/src/core/glossary/exchange.rs#L84)

**Hai lớp lỗi được cấp tên — mục ② và ⑤**

- Hai biến thể mới; `match` exhaustive làm quên một mắt đỏ ở khâu biên dịch.
  [`exchange.rs:285`](../../src-tauri/src/core/glossary/exchange.rs#L285)

- Cột trùng bắt bằng đếm, không bằng `position()` lấy khớp đầu.
  [`exchange.rs:822`](../../src-tauri/src/core/glossary/exchange.rs#L822)

- Chín khoá phân tích, danh mục đóng; doc-comment "TÁM khoá" sửa tại chỗ.
  [`i18n/mod.rs:344`](../../src-tauri/src/core/i18n/mod.rs#L344)

**Rỗng im lặng — mục ⑥ và ⑥c, lớp lỗi trung tâm của kho**

- Ba cột văn bản tự do, không riêng `source_term` — vế Ice ký nới phạm vi.
  [`exchange.rs:992`](../../src-tauri/src/core/glossary/exchange.rs#L992)

- Năm ký tự zero-width lọt cả `str::trim()` lẫn bảng 25 điểm mã của SQL.
  [`exchange.rs:507`](../../src-tauri/src/core/glossary/exchange.rs#L507)

**Ranh giới dòng và dấu phân cách — mục ③ và ④**

- Đếm `\r`, `\r\n`, `\n` mỗi thứ một ranh giới; trước đó chỉ đếm `\n`.
  [`exchange.rs:615`](../../src-tauri/src/core/glossary/exchange.rs#L615)

- Quét nhận biết nháy kép thay vì `contains()` trên văn bản thô.
  [`exchange.rs:726`](../../src-tauri/src/core/glossary/exchange.rs#L726)

**I/O tệp — mục ⑧ và ⑨, hai bề mặt chạm đĩa duy nhất**

- Chặn thật bằng `take(LIMIT+1)`; `metadata` không còn quyết định gì.
  [`exchange_io.rs:60`](../../src-tauri/src/core/glossary/exchange_io.rs#L60)

- Hậu tố `pid`+`uuid`: khuôn `write_atomic` đúng cho đường nội bộ, không cho đường người dùng chọn.
  [`exchange_io.rs:135`](../../src-tauri/src/core/glossary/exchange_io.rs#L135)

- `size` nay là số byte đã đọc, không phải kích thước tệp — đánh đổi ghi rõ.
  [`store.rs:476`](../../src-tauri/src/core/glossary/store.rs#L476)

**Phép kiểm — đọc sau cùng, nhưng đọc kỹ ba ca này**

- Ca lật miễn trừ sai: `'=1+1` phải về nguyên vẹn.
  [`glossary_exchange_contract.rs:196`](../../src-tauri/tests/glossary_exchange_contract.rs#L196)

- Ca chứng minh `translation` toàn zero-width không thành mục đã chốt vô hình.
  [`glossary_exchange_contract.rs:1265`](../../src-tauri/tests/glossary_exchange_contract.rs#L1265)

- Ba ca biên; ca cột-lặp-ba-lần đỏ trong khi ca cột-lặp-hai-lần vẫn xanh.
  [`glossary_exchange_contract.rs:1312`](../../src-tauri/tests/glossary_exchange_contract.rs#L1312)
