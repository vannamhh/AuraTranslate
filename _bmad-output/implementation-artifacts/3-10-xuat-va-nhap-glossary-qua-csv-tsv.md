---
title: 'Story 3.10 — Xuất và nhập Glossary qua CSV/TSV (nửa định dạng + đường ghi)'
type: 'feature'
created: '2026-08-24'
status: 'done'
review_loop_iteration: 1
baseline_commit: '3a1d8295a395639cb2c05a731328e1c55c83ee8a'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-9-quan-ly-glossary.md'
  - '{project-root}/_bmad-output/planning-artifacts/ad-brief-2026-08-24-hop-thoai-chon-tep.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Glossary hôm nay **không mang đi được**. `grep -rn "csv\|tsv" src-tauri/src/**` trả **rỗng** — không đường nào sinh ra một tệp, không đường nào đọc một tệp vào. Hệ quả nặng nhất: một bộ thuật ngữ dựng suốt một Tác phẩm dài chết theo máy của người dựng, và cách chia sẻ mà FR49/NFR9 hứa (*"không cần server hay tài khoản"*) chưa tồn tại ở bất kỳ hình dạng nào.

**Approach:** Một module định dạng **thuần** (`&str` vào, `String` ra) cộng **một** đường ghi nguyên tử xuống kho. Xuất đọc **một tầng** qua `load_tier`; nhập phân tích trọn văn bản, phân loại từng hàng là *mới* / *giống* / *bất đồng*, rồi ghi theo quyết định của người dùng trong **một** giao dịch. Mục vào từ tệp mang một xuất xứ **thứ tư**, `TermOrigin::FileImport` — thứ đòi một bước di trú dựng lại bảng trên **cả hai** kho.

## Boundaries & Constraints

**Always:**
- 🔴 **Vào `&str`, ra `String`. Module này KHÔNG chạm hệ thống tệp.** Đọc/ghi byte và lấy đường dẫn thuộc nửa chọn tệp, đang chờ một `AD` (`ad-brief-2026-08-24-hop-thoai-chon-tep.md`). Chỗ nối để lại là **một hàm** trả `PathBuf` — đừng dựng sẵn một `#[tauri::command]` nhận đường dẫn mà không lối vào nào gọi tới; một bề mặt IPC không ai gọi là bề mặt không ai canh.
- 🔴 **Bước di trú là một hằng MỚI, KHÔNG sửa `GLOSSARY_ENTRY_DDL` tại chỗ** — `schema.rs:438-447` ghi nguyên lý và lý do: một kho đã di trú không bao giờ chạy lại hằng cũ, nên sửa tại chỗ làm kho cũ và kho mới lệch lược đồ trong khi cùng báo một `user_version`. Hằng mới vào **cả hai** danh sách: `GLOBAL_MIGRATIONS` bước **5**, `PROJECT_MIGRATIONS` bước **15** — cùng khuôn `GLOSSARY_ENTRY_DDL` đã dùng cho hai thang.
- 🔴 **`DROP TABLE` cuốn theo index và trigger.** Bước dựng lại phải tạo lại `idx_glossary_entry_source_term` (`UNIQUE`) **và** `glossary_entry_lifecycle_is_one_way`. Thiếu trigger là vòng đời một chiều của AD-36 chết trong im lặng; thiếu index là `source_term` trùng được.
- 🔴 **`id` KHÔNG được tái dùng sau di trú.** Bảng dùng `AUTOINCREMENT`; dựng lại bảng vứt mốc `sqlite_sequence`, nên nếu đã có hàng bị xoá trước lượt di trú thì `id` cao nhất từng cấp bị hạ xuống và một `id` cũ được cấp lại. Mốc phải được mang theo trong chính bước di trú.
- 🔴 **Đường ghi nhập tự đặt `TermOrigin::FileImport`, KHÔNG nhận qua tham số.** `glossary_boundary.rs:232` cấm mọi tệp ngoài `core/glossary/**` **gõ** một biến thể `TermOrigin::` phi-manual, và `insert_manual_entry` đã CỐ Ý mất tham số `term_origin` từ Story 3.2 (vế cấu trúc của FR55). Giữ nguyên hình dạng đó: hàm mới suy xuất xứ từ **việc nó là đường nhập tệp**, đúng cách `approve_candidate` suy từ `candidate_origin`.
- 🔴 **Không ghi một phần.** Toàn bộ lượt nhập đi trong **một** `store.write` — `Store::write` khai *"mỗi job là một giao dịch: `Ok` ⇒ commit, `Err` ⇒ rollback"*. Phân tích xong TRỌN văn bản rồi mới mở giao dịch: một lỗi ở dòng 604 phải để dòng 1 không được ghi.
- 🔴 **Không im lặng ghi đè.** Hàng bất đồng với mục đang có **không** được ghi trừ khi quyết định của người dùng nói ghi. Mặc định là **giữ của tôi** — một tệp lạ không được phép lật một quyết định biên tập chỉ vì nó tới sau.
- **Tên cột trong tệp là định danh máy, KHÔNG DẤU** (`source_term` · `translation` · `note` · `category` · `term_origin` · `created_at`). Kiểm A của `check:i18n` cấm chữ tiếng Việt có dấu ở vị trí mã, và AD-21/NFR16 đã đặt cùng luật cho `Category::as_str()`. Xem §Design Notes — mockup vẽ tên cột tiếng Việt và đó là chỗ spec này cố ý lệch mockup.
- **Không thêm phụ thuộc npm/crate nào** (NFR15 + luật cổng: parser trong kho là **tập con nghiêm ngặt tự viết**, cú pháp ngoài tập con ⇒ FAIL chứ không bỏ qua). Không `csv`, không `serde_csv`.
- Chuỗi literal trong `src-tauri/src/**` viết **KHÔNG DẤU**; `impl Display` của lỗi mới là chẩn đoán cho log. Khoá lỗi mới khai qua `message_keys!` để vào `ALL`. `params` của `IpcError` mang **DỮ LIỆU** (`{line}`, `{column}`), không mang câu.

**Ask First:**
- Nếu hoá ra bước di trú **không** giữ được mốc `sqlite_sequence` bằng SQL tĩnh — đó là đổi một bất biến, không phải một chi tiết cài đặt.
- Bất kỳ phụ thuộc mới nào (NFR15).
- Nếu vòng đời một chiều (AD-36) hoá ra chặn một ca nhập hợp lệ mà §I/O Matrix chưa liệt.

**Never:**
- **Không đọc tệp, không ghi tệp, không hộp thoại, không `#[tauri::command]` mới.** Cả bốn thuộc nửa đang chờ `AD`.
- **Không sinh `id` vào tệp xuất** — `id` chỉ duy nhất TRONG một kho (`entry.rs:135`), nên một `id` trong tệp của người khác là một con số vô nghĩa và một cái bẫy va khoá.
- **Không hợp nhất hai tầng khi xuất.** Xuất đọc đúng một tầng; `list_all_entries` phát cả hàng bị che thành hàng thứ hai, nên dùng nó sẽ sinh `source_term` trùng trong tệp và lượt nhập lại va `UNIQUE`.
- **Không sửa `source_term` của một mục đang có** để giải một bất đồng — `deferred-work.md:6664` giao đúng chỗ này cho story: nhánh va `UNIQUE` phải được **viết ra**, không phải mở rộng chữ ký `update_manual_term`.
- Không nới `GLOSSARY_ONLY_SURFACE`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Xuất một tầng | Tầng có N mục | Một `String`: hàng tiêu đề + N hàng, **6 cột**, không cột `id` | Lỗi đọc kho ⇒ `StoreError` |
| Xuất tầng rỗng | 0 mục | **Chỉ** hàng tiêu đề — một tệp có tiêu đề nói *"rỗng"*, một tệp trống không nói gì | N/A |
| Trường chứa dấu phân cách / nháy kép / xuống dòng | `note` = `a,b"c` | Bọc nháy kép, nháy kép nhân đôi (RFC 4180) — áp cho **cả** CSV và TSV | N/A |
| Vòng tròn xuất→nhập | Tệp vừa xuất, nhập vào một tầng RỖNG | N mục, **5 trường giữ nguyên** (`source_term` · `translation` · `note` · `category` · `created_at`); `term_origin` thành `file_import` — xem §Design Notes | N/A |
| Đoán dấu phân cách | Hàng tiêu đề chứa `,` **hoặc** `\t` | Chọn đúng cái có mặt | Có **cả hai** hoặc **không cái nào** ⇒ lỗi có tên, **0** lượt ghi |
| Thiếu một cột bắt buộc | Tiêu đề không có `source_term` | Lỗi nêu **tên cột** thiếu | 0 lượt ghi |
| Thừa cột lạ | Tiêu đề có `usage_count` | **Bỏ qua cột đó và NÓI RA** — không im lặng vứt | N/A |
| Vắng cột tuỳ chọn | Chỉ 4 cột như mockup | Nhập được; `created_at` = hôm nay, `term_origin` = `file_import` | N/A |
| Số ô lệch hàng tiêu đề | Dòng 87 có 5 ô / 6 cột | Lỗi mang **số dòng** và số ô đếm được | 0 lượt ghi |
| `category` lạ | `category` = `weapon` | Lỗi mang **số dòng** và giá trị đọc được | 0 lượt ghi |
| `source_term` rỗng/toàn khoảng trắng | Ô trống hoặc `U+3000` | Lỗi mang số dòng — cùng mệnh đề `CHECK` của DDL, bắt ở Rust **trước** khi SQL bắt | 0 lượt ghi |
| Mục không có bản dịch | Ô `translation` rỗng | Vào **chờ chốt** (`translation = None`), **không** phải đã chốt với chuỗi rỗng | N/A |
| Hàng mới | `source_term` chưa có ở tầng đích | Phân loại **mới** | N/A |
| Hàng giống hệt | Cùng `source_term`, cùng `translation` | Phân loại **giống** — không đề nghị gì, không ghi | N/A |
| Hàng bất đồng | Cùng `source_term`, khác `translation` | Phân loại **bất đồng**, mang **cả hai** bản dịch; mặc định **giữ của tôi** | N/A |
| Bất đồng, người dùng lấy của file | Quyết định `TakeTheirs` | 🔴 `UPDATE` **chỉ cột `translation`** của hàng đó. `note` và `category` **không bị chạm, dù tệp có nói gì về chúng hay không** | Trigger AD-36 chặn lượt lùi về rỗng ⇒ `store.write_failed`, cả lô rollback |
| Bất đồng, tệp thiếu cột `note`/`category` | Tệp hai cột + `TakeTheirs` | Ghi chú và phân loại của mục đang có giữ nguyên **từng byte** | N/A |
| Bất đồng, tệp CÓ cột `note` mang giá trị khác | `TakeTheirs` | Vẫn **không** ghi `note` — đường nhập không sửa ghi chú của một mục đã có | N/A |
| `source_term` trùng **trong chính tệp** | Hai dòng cùng `source_term` | Lỗi mang **cả hai** số dòng — không "dòng sau thắng" im lặng | 0 lượt ghi |
| Nhập vào tầng Tác phẩm khi chưa mở | `work` là `None` | `GlossaryError::WorkTierUnavailable` | 0 lượt ghi |
| Va `UNIQUE` giữa chừng | Một mục được thêm ở nơi khác sau lượt phân loại | Giao dịch rollback; **0** hàng ghi, lỗi nói va ở `source_term` nào | Nạp lại rồi phân loại lại |
| Văn bản rỗng / chỉ có tiêu đề | 0 hàng dữ liệu | **0 mục, không lỗi** — phân biệt được với *"tệp hỏng"* | N/A |
| Tệp kết thúc bằng dòng trống | `…\n\n` (trình soạn thảo tự thêm) | Dòng trống **không phải một hàng dữ liệu** — bỏ qua, **không lỗi** | N/A |
| `created_at` sai định dạng | Ô mang `hom qua` | Lỗi mang **số dòng** và giá trị đọc được — cột này là ISO-8601 UTC, không phải văn bản tự do | 0 lượt ghi |
| BOM ở đầu tệp | `EF BB BF` | Cắt trước khi phân tích — cùng khuôn `import.rs::strip_bom` | N/A |
| `\r\n` và `\n` lẫn lộn | Tệp từ Windows | Cả hai đọc được; `\r` không lọt vào giá trị ô cuối | N/A |

</frozen-after-approval>

## Code Map

**Định dạng — module MỚI, thuần**
- `src-tauri/src/core/glossary/exchange.rs` (**mới**) -- render + parse + phân loại bất đồng. Không `rusqlite`, không `tauri`. Khai vào `mod.rs` cạnh `entry`/`store`.
- `src-tauri/src/core/segment/import.rs:241` `import_file` · `:60` `BOM` · `strip_bom` -- **khuôn chép** cho việc cắt BOM và cho luật *"từ chối tường minh thay vì đoán"*. ⚠️ **Không** chép trần `MAX_IMPORT_BYTES` (`:57`): nó canh `std::fs::read`, mà module này không đọc tệp.
- `src-tauri/src/core/glossary/entry.rs:27-69` `Category` (4 giá trị, `as_str`/`from_wire`) · `:82-111` `TermOrigin` (**3 → 4**, `as_str`/`from_wire` đều là `match` toàn phần) · `:195-211` `GlossaryEntry` (7 trường) · `:215` `is_confirmed()` = `translation.is_some()` -- **trạng thái chốt không phải một cột**, nên ô `translation` rỗng LÀ *chờ chốt*.

**Di trú — lần đầu kho dựng lại một bảng**
- `src-tauri/src/core/store/schema.rs:300-331` `GLOSSARY_ENTRY_DDL` -- `CHECK (term_origin IN (…))` ở `:325`, `UNIQUE INDEX` `:327`, trigger `:328-331`. 🔴 **Đọc, đừng sửa.**
- `…/schema.rs:438-464` `GLOSSARY_CANDIDATE_OCCURRENCE_CONTEXT_DDL` -- **khuôn chép** cho một bước thêm-vào-sau: doc-comment `:441-447` viết thẳng vì sao hằng cũ không được sửa tại chỗ. ⚠️ Nó là `ALTER TABLE`; bước của story này là **dựng lại**, nên chỉ chép *nguyên lý*, không chép *hình dạng SQL*.
- `…/schema.rs:480-499` `GLOBAL_MIGRATIONS` (4 bước, `GLOSSARY_ENTRY_DDL` ở bước 4) · `:1175+` `PROJECT_MIGRATIONS` (14 bước, `GLOSSARY_ENTRY_DDL` ở bước 12). Cùng một hằng, hai danh sách — bước mới cũng vậy.
- `src-tauri/src/core/store/mod.rs:624` `Store::write` -- *"mỗi job là một giao dịch"*; `:655` `Store::read` đặt `query_only = 1`.

**Đường ghi**
- `src-tauri/src/core/glossary/store.rs:237` `load_tier` -- nguồn của lượt XUẤT (`BTreeMap<String, GlossaryEntry>` một tầng). Bị `GLOSSARY_ONLY_SURFACE` cấm gọi ngoài module — module này Ở TRONG, hợp lệ.
- `…/store.rs:117-171` `insert_manual_entry` -- khuôn `INSERT`; `:126-131` giải thích vì sao `source_term` được `trim` — và nói thẳng `str::trim()` của Rust cắt theo **thuộc tính Unicode**, khác tập 25 điểm mã mà `CHECK` liệt. **Chữ ký không có `term_origin`, và giữ thế.**
- `…/store.rs:626` `add_manual_term` -- khuôn định tuyến `&Store` theo `tier`. `:777` `delete_manual_term` · `:828` `promote_to_global` -- khuôn `GlossaryError` + *"0 lượt ghi khi từ chối"*.
- `…/store.rs:385-415` `GlossaryError` (5 biến thể) · `:456` `impl From<GlossaryError> for IpcError` -- 🔴 đi **qua `IpcError::new`**, không struct literal.
- `src-tauri/src/core/i18n/mod.rs:283-304` -- bốn khoá `err.glossary.*` hôm nay; khoá mới khai trong `message_keys!` để vào `ALL`. `:293` là khuôn khoá **không tham số**, `:304` là khuôn khoá mới nhất.
- `src/i18n/vi.json:25-28` -- bốn câu tương ứng. 🔴 Cặp `message_keys!` ↔ `vi.json` do **`cargo test`** canh, không `check:i18n`: `src-tauri/tests/ipc_contract.rs:232` (khoá nào cũng phải có câu) và `:325` (câu phải dùng đúng tham số đã khai).
- `src-tauri/src/core/store/schema.rs` §`CHECK` rào rỗng -- 🔴 liệt **25 điểm mã `White_Space`**; `trim()` của SQLite chỉ cắt dấu cách ASCII. Phép kiểm `source_term` rỗng ở Rust phải phủ đúng tập đó, không chỉ `str::trim()`.

**Cổng và test sẽ phán**
- `src-tauri/tests/glossary_boundary.rs:232` `NON_MANUAL_ORIGIN_TOKENS` **2 → 3** (thêm `TermOrigin::FileImport`); `:142` `GLOSSARY_ONLY_SURFACE` **giữ nguyên 4**; `:194` `QUICK_ADD_SURFACE` **giữ nguyên 12** — story này không dựng vỏ IPC nào.
- `src-tauri/tests/glossary_contract.rs:338` `the_blank_form_list_covers_every_unicode_whitespace_code_point` -- bằng chứng CHẠY ĐƯỢC cho bảng 25 ký tự khoảng trắng của `GLOSSARY_ENTRY_DDL`; `:395` `an_empty_or_whitespace_only_source_term_is_refused_and_writes_nothing`. 🔴 Bảng ký tự trong DDL dựng lại phải **trùng từng byte** — `schema.rs:389-393` ghi rằng mệnh đề đó được khoá bằng một phép so chuỗi, không bằng mắt.
- `check:i18n` Kiểm A (chữ có dấu ở vị trí mã) · A2 (khoá khớp `vi.json`). `check:deps` (không phụ thuộc mới).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store/schema.rs` -- hằng di trú **mới** dựng lại `glossary_entry` với `CHECK` bốn giá trị: tạo bảng tạm ⇒ chép hàng **giữ nguyên `id`** ⇒ mang mốc `sqlite_sequence` ⇒ `DROP` ⇒ `RENAME` ⇒ tạo lại `UNIQUE INDEX` **và** trigger một chiều. Vào `GLOBAL_MIGRATIONS` (bước 5) **và** `PROJECT_MIGRATIONS` (bước 15) -- một `CHECK` không `ALTER` được, nên đây là đường duy nhất; và `DROP` cuốn theo index + trigger nên bỏ sót một trong hai là mất một bất biến trong im lặng.
- [x] `src-tauri/src/core/glossary/entry.rs` -- `TermOrigin::FileImport`, `as_str()` = `"file_import"`, `from_wire` nhận nó -- ba nhánh `match` đều là `match` toàn phần nên trình biên dịch chỉ ra đủ chỗ phải sửa.
- [x] `src-tauri/src/core/glossary/exchange.rs` (**mới**) -- `render_tier(&BTreeMap<String, GlossaryEntry>, Delimiter) -> String` và `parse(&str) -> Result<ParsedImport, Vec<ParseIssue>>`; bọc/gỡ nháy kép RFC 4180 cho **cả hai** dấu phân cách; đoán dấu phân cách từ hàng tiêu đề; cắt BOM; nhận `\r\n` lẫn `\n` -- một đường bọc dùng chung là chỗ duy nhất bảo đảm vòng tròn xuất→nhập khép kín; hai đường sẽ lệch ở đúng ô có dấu phẩy. 🔵 **Chữ ký `parse` LỆCH bản nháp của Tasks có chủ ý** — trả `ParsedImport { rows, ignored_columns }` thay vì `Vec<ImportRow>` trần: I/O Matrix đòi "cột lạ ⇒ bỏ qua và NÓI RA" là ca **KHÔNG lỗi** (`Error Handling: N/A`), nên tin đó phải đi trên nhánh `Ok`, không nhét được vào `Vec<ParseIssue>` (nhánh `Err`). `Vec<ImportRow>` trần không có chỗ chở tin đó.
- [x] `src-tauri/src/core/glossary/exchange.rs` -- `classify(rows, &BTreeMap<String, GlossaryEntry>) -> Vec<RowPlan>` với ba nhánh *mới* / *giống* / *bất đồng (mang cả hai bản dịch)* -- phân loại là hàm THUẦN để nghiệm thu được không cần kho, và để mặc định *giữ của tôi* nằm ở một chỗ đọc được.
- [x] `src-tauri/src/core/glossary/store.rs` -- `import_into_tier(global, work, tier, plans, decisions)` -- **một** `store.write`; tự đặt `TermOrigin::FileImport`; `WorkTierUnavailable` khi `tier == Work` mà `work` là `None`; va `UNIQUE` giữa chừng ⇒ rollback trọn lô -- xuất xứ không đi qua tham số vì `glossary_boundary.rs:232` cấm chỗ gọi ngoài gõ token đó. `GlossaryError::ImportUniqueConflict{source_term}` được phát hiện **SAU** khi giao dịch đã rollback (nạp lại tầng, tìm hàng `New` nay đã tồn tại), không phải một khoá đọc-trước-khi-ghi mở cửa sổ đua thứ hai.
- [x] `src-tauri/src/core/glossary/store.rs` -- `export_tier(store, Delimiter) -> Result<String, GlossaryError>` gọi `load_tier` **một tầng** -- không đi qua `list_all_entries`: nó phát hàng bị che thành hàng thứ hai, tức `source_term` trùng trong tệp.
- [x] `src-tauri/src/core/i18n/mod.rs` -- khoá `err.glossary.import_*` cho từng ca §I/O Matrix, khai trong `message_keys!` kèm **bảng tham số** (`{line}` · `{column}` · `{value}`) -- tham số chở DỮ LIỆU, không chở CÂU. Bảy khoá: `import_delimiter_unresolved` · `import_missing_column` · `import_cell_count_mismatch` · `import_unknown_category` · `import_blank_source_term` · `import_duplicate_source_term` · `import_unique_conflict`.
- [x] `src/i18n/vi.json` -- một câu tiếng Việt cho **mỗi** khoá mới, và mỗi câu phải **dùng đúng** những tham số đã khai -- `ipc_contract.rs:232` `every_message_key_exists_in_vi_json` và `:325` `every_message_key_declares_the_params_its_string_needs` cho ĐỎ ở `cargo test`, không ở `check:i18n`; một khoá quên câu là một lỗi chỉ lộ ra ở khâu cuối.
- [x] `src-tauri/src/core/glossary/mod.rs` -- khai `exchange`, cập nhật doc-comment §HÌNH DẠNG ĐÃ DỰNG kèm ngày -- module đó là bản đồ đầu tiên người sau đọc.
- [x] `src-tauri/tests/glossary_boundary.rs` -- `NON_MANUAL_ORIGIN_TOKENS` 2 → 3.
- [x] `src-tauri/tests/glossary_exchange_contract.rs` (**mới**) -- phủ trọn §I/O Matrix (28 ca); gồm ca **vòng tròn** trên một tầng có ô mang dấu phẩy, nháy kép và xuống dòng, và ca **đối chứng**: gỡ lớp bọc nháy kép ⇒ ca vòng tròn phải ĐỎ.
- [x] `src-tauri/tests/glossary_contract.rs` -- ca di trú, dựng bằng khuôn `store_contract.rs:859` `spec_with_migrations`: một kho ở phiên bản CŨ có hàng dữ liệu **và một hàng đã bị xoá**, di trú lên, rồi khẳng định bốn mệnh đề — hàng còn đủ, `id` **không đổi**, `UNIQUE INDEX` và trigger một chiều còn **đỏ được**, và `id` cấp tiếp theo **lớn hơn** mọi `id` từng cấp -- hàng đã xoá là thứ làm mệnh đề `sqlite_sequence` kiểm được; thiếu nó thì `max(id)` bằng mốc và ca xanh mà không canh gì. 🔴 **Hàng đã xoá phải là hàng `id` CAO NHẤT** — bản đầu của ca này xoá một hàng GIỮA và đối chứng gỡ chỗ nối (③ dưới) không đỏ, vì `INSERT … SELECT id, …` tường minh tự nâng mốc `sqlite_sequence` của bảng mới lên đúng `id` lớn nhất còn lại trong dữ liệu chép, che mất việc thiếu bước mang mốc riêng. Sửa fixture để xoá đúng hàng cao nhất mới thật sự bắt buộc bước mang mốc chạy.

**Acceptance Criteria:**
- Given một tầng có mục mang dấu phẩy, nháy kép và xuống dòng trong `note`, when xuất rồi nhập vào một tầng rỗng, then năm trường người dùng nhìn thấy giữ nguyên từng byte, và phép đối chứng gỡ lớp bọc cho ca test ĐỎ.
- Given một kho ở phiên bản trước story này có mục Glossary, when mở bằng bản mới, then di trú chạy, **0** hàng mất, `id` không đổi, và một `id` đã cấp không bao giờ được cấp lại.
- Given một tệp nhập hỏng ở bất kỳ dòng nào, when nhập, then **0** hàng được ghi và lỗi nêu đúng số dòng.
- Given `cargo test --locked`, when chạy, then xanh — gồm `every_message_key_exists_in_vi_json` và `every_message_key_declares_the_params_its_string_needs`; và `check:i18n` · `check:deps` xanh **không** một miễn trừ mới nào.
- Given `grep -rn "std::fs\|PathBuf\|tauri::" src-tauri/src/core/glossary/exchange.rs`, when chạy, then **rỗng** — module định dạng không chạm tệp và không chạm Tauri.

## Spec Change Log

### 2026-08-25 — vòng rà ba lớp, một `intent_gap` và mười bản vá

**`intent_gap` — §I/O Matrix sửa, Ice chốt 2026-08-25.** HAI lớp rà độc lập (`blind-hunter`,
`edge-case-hunter`) hội tụ vào cùng một chỗ, và lượt đối chứng tận mã xác nhận: nhánh
`TakeTheirs` ghi `SET translation = ?1, note = ?2, category = ?3` **vô điều kiện**, trong khi
`exchange.rs:475`/`:504` điền `Category::Other` và `""` cho cột VẮNG. ⇒ Nhập một tệp hai cột
(đúng hình dạng mockup vẽ, và đúng hình dạng tệp người khác gửi) rồi bấm *lấy của file* **xoá
sạch ghi chú người dùng tự viết** và **hạ `person` xuống `other`** — trong khi người dùng chỉ
đồng ý đổi **bản dịch**. Phá dữ liệu người dùng trong im lặng, ngược chính §Always *"Không im
lặng ghi đè"*.

Chỗ hỏng nằm TRONG `<frozen-after-approval>`: §I/O Matrix chỉ viết *"`UPDATE` đúng hàng đó"*,
chưa bao giờ nói **đúng những cột nào**. Ice chốt: *lấy của file* ghi **chỉ `translation`**,
không bao giờ chạm `note`/`category` — khớp đúng thứ màn hình xung đột CHO NGƯỜI DÙNG THẤY
(mockup chỉ hiện *"đang có X · file ghi Y"* về bản dịch). Ba hàng Matrix mới ghi cả ba ca.
Ice cũng cho phép **vá tại chỗ thay vì hoàn tác và dựng lại** — lệch workflow một bước, có
chữ ký.

**Hai hàng Matrix nữa, từ hai phát hiện đo được:** tệp kết thúc bằng dòng trống (đo bằng một
ca tạm rồi gỡ: `parse` trả `CellCountMismatch { line: 3, expected: 6, found: 1 }` — một tệp
hợp lệ bị từ chối TRỌN VẸN, kèm câu lỗi trỏ vào một dòng người dùng thấy là rỗng); và
`created_at` sai định dạng đi thẳng vào một cột ISO-8601 mà không ai kiểm.

**Ba phát hiện bị BÁC, ghi ra thay vì bỏ im:** *"bước dựng lại bảng có thể làm gãy khoá
ngoại"* — lược đồ này không có `FOREIGN KEY` nào, cả kho (`schema.rs:693` · `:857` · `:1069`);
*"không có vỏ `#[tauri::command]` nên tính năng không có lối vào"* — đúng §Never, có chủ ý;
*"ô chỉ có khoảng trắng phá vòng tròn"* — không tới được, mọi giá trị đã bị `trim` trước khi
vào kho.

🔴 **KEEP — đã đúng, phải sống sót mọi lượt sửa sau:**
- **Bước di trú**, nguyên vẹn. Ba mệnh đề SQLite của nó đã được đo lại độc lập (`sqlite3`
  3.43.2): chèn `id` tường minh KHÔNG nâng mốc lên giá trị lịch sử (mốc 3 → còn 2, nên việc
  mang mốc bằng tay là cần THẬT); `DROP TABLE` cuốn theo hàng `sqlite_sequence` của bảng cũ
  (nên thứ tự *`UPDATE` trước `DROP`* là bắt buộc, và mã đang đúng); `RENAME TO` tự đổi tên
  hàng `sqlite_sequence`; `id` cấp tiếp theo sau di trú là **4**, không phải 3.
- **`export_tier` đọc một tầng qua `load_tier`**, không đi qua `list_all_entries` — giữ cho
  tệp không có `source_term` trùng.
- **Một đường bọc nháy kép dùng chung cho cả CSV lẫn TSV**, và phép đối chứng gỡ lớp bọc.
- **Ba chỗ gọi `insert_entry_row` cũ truyền `None`** — đã đối chứng: đúng 4 chỗ gọi, ba chỗ cũ
  chỉ mọc thêm một dòng `None`, rơi vào nhánh `strftime('now')` y như trước.
- **Phân tích TRỌN văn bản trước khi mở giao dịch**, để người dùng thấy MỌI lỗi trong một lượt.

### 2026-08-25 (muộn hơn) — vá tám phát hiện của vòng rà, ba đối chứng gỡ chỗ nối mới

Tám phát hiện (P1-P8) của vòng rà ba lớp được vá trong một lượt:

- **P1 (`intent_gap`, Ice chốt).** `TakeTheirs` trong `import_into_tier` nay `UPDATE
  glossary_entry SET translation = ?1 WHERE id = ?2` — CHỈ một cột. `note`/`category` không
  bao giờ đi vào câu lệnh, kể cả khi tệp mang giá trị THẬT cho chúng. Ba hàng Matrix mới phủ
  bằng ba ca test (`take_theirs_updates_only_translation_and_never_touches_note_or_category`
  · `take_theirs_ignores_a_note_and_category_the_file_actually_provides` — dựng `RowPlan`
  trực tiếp với `note`/`category` KHÁC hẳn giá trị đang có, xác nhận cả hai giữ nguyên TỪNG
  BYTE sau `TakeTheirs`).
- **P2.** Một dòng logic RỖNG (`raw.trim().is_empty()`) không còn sinh `ParseIssue` — áp
  CÙNG một luật cho dòng trống CUỐI tệp và dòng trống Ở GIỮA tệp, viết ra tường minh trong
  doc-comment (không để tình cờ). Hai ca mới:
  `a_trailing_blank_logical_line_is_not_a_data_row_and_is_not_an_error` và
  `a_blank_logical_line_in_the_middle_of_the_file_is_also_skipped_by_the_same_rule` (ca sau
  còn khẳng định số dòng của hàng SAU dòng trống KHÔNG bị lệch).
- **P3.** `ParseIssue::InvalidCreatedAt { line, value }` mới — `created_at` có mặt, không
  rỗng, phải khớp hình dạng `YYYY-MM-DDTHH:MM:SS.mmmZ` (kiểm HÌNH DẠNG bằng
  `looks_like_iso8601_utc`, không kiểm lịch, không crate ngày-giờ mới — NFR15). Khoá
  `err.glossary.import_invalid_created_at` + câu `vi.json` mới.
- **P4.** Khoảng hở phép canh đã đóng: `import_into_tier_preserves_a_supplied_created_at_
  for_a_new_row` dựng `RowPlan { kind: New, created_at: Some(mốc cố định), .. }` trực tiếp
  (helper `row()` ghim `None`, không phủ được nhánh này qua `classify`), nạp lại và khẳng
  định `created_at` trên đĩa khớp NGUYÊN VĂN mốc đã cung cấp.
- **P5.** `import_into_tier` viết lại: kiểm `is_unique_constraint_violation` (mã mới trong
  `core::store`, so bằng `extended_code == SQLITE_CONSTRAINT_UNIQUE` — KHÔNG so chuỗi
  `Display`, vì thông điệp trigger AD-36 là văn bản tự do có thể đổi) NGAY tại chỗ một
  `INSERT` hàng `New` thất bại, không còn đoán ngược từ việc nạp lại tầng sau rollback. Một
  lỗi không phải `UNIQUE` (kể cả trigger AD-36) abort ngay với nguyên nhân GỐC. Hàm mới sống
  trong `core::store` (không phải `core::glossary`) vì `store_boundary.rs::only_core_store_
  may_name_rusqlite` cấm `core::glossary` gõ tên `rusqlite` — bắt bởi chính cổng đó ở lượt
  vá đầu, sửa bằng cách tái xuất một hàm thay vì một kiểu.
- **P6.** `GlossaryError::ImportUniqueConflict` đổi từ `source_term: String` sang
  `source_terms: Vec<String>` — vòng lặp GOM mọi va chạm `UNIQUE` của hàng `New` trong CÙNG
  một lượt (không dừng ở va đầu tiên) rồi mới rollback trọn qua một lỗi ĐÁNH DẤU
  (`unique_conflict_marker_error`, nội dung không bao giờ đọc lại — danh sách thật đi qua
  `Arc<Mutex<Vec<String>>>`). `IpcError.params["value"]` nối các thuật ngữ bằng `", "`.
- **P7.** `glossary_exchange_contract.rs::every_parse_issue_variant_maps_to_the_message_
  key_it_actually_produces` — một hàm `match` EXHAUSTIVE viết tay (không nhánh `_`) đối
  chiếu MỖI biến thể `ParseIssue` với `MessageKey` nó THẬT SỰ tạo ra qua `.into()`. Thêm một
  biến thể `ParseIssue` mà quên cập nhật hàm này là lỗi biên dịch ở tệp test, không phải một
  lỗ hổng im lặng.
- **P8.** Ca di trú nay chụp ảnh `sqlite_master` (lọc `tbl_name = 'glossary_entry'`) TRƯỚC
  lượt di trú thật, khẳng định đúng hai đối tượng (`idx_glossary_entry_source_term` +
  `glossary_entry_lifecycle_is_one_way`) — lời khai "hai đối tượng DUY NHẤT" nay là một mệnh
  đề kiểm được, không chỉ một câu trong doc-comment.

**Ba đối chứng GỠ CHỖ NỐI mới, mỗi ca gỡ rồi khôi phục, chạy trên cây thật:**

| Gỡ chỗ nối | Kết quả |
|---|---|
| P1 — trả `UPDATE` về ghi cả ba cột (`translation, note, category`) | ĐỎ — đúng **2 ca**: `take_theirs_updates_only_translation_and_never_touches_note_or_category` và `take_theirs_ignores_a_note_and_category_the_file_actually_provides`. Khôi phục ⇒ xanh. |
| P2 — bỏ nhánh `if raw.trim().is_empty() { continue; }` | ĐỎ — đúng **2 ca**: `a_trailing_blank_logical_line_is_not_a_data_row_and_is_not_an_error` và `a_blank_logical_line_in_the_middle_of_the_file_is_also_skipped_by_the_same_rule`, cả hai trượt với `CellCountMismatch` — đúng lỗi mà P2 tồn tại để đóng. Khôi phục ⇒ xanh. |
| P5 — bỏ vế `if is_unique_constraint_violation(&e)`, coi MỌI lỗi `New` là `UNIQUE` | ⚠️ **Ca đầu KHÔNG đỏ** (`a_batch_with_both_an_unrelated_pre_existing_term_and_a_trigger_violation_reports_the_trigger_as_the_real_cause`) — lượt `TakeTheirs` luôn abort qua `?` TRƯỚC khi `local_conflicts` kịp ảnh hưởng kết quả, dù cơ chế phân biệt có mặt hay không; ca đó đo đúng HÀNH VI cuối nhưng không đo đúng CHỖ NỐI. Dựng ca cô lập thứ hai — `a_new_row_failing_for_a_reason_other_than_unique_reports_the_real_error`, một hàng `New` vi phạm `CHECK` (không phải `UNIQUE`, dựng `RowPlan` trực tiếp) — chạy lại ⇒ ĐỎ đúng ca (`Nhan: Err(ImportUniqueConflict { source_terms: ["hang-loi"] })`, sai nguyên nhân). Khôi phục ⇒ xanh. |

⚠️ **Bài học từ đối chứng P5, ghi lại vì nó áp cho mọi ca "báo đúng nguyên nhân" tương lai:**
một ca test có thể đo đúng KẾT QUẢ CUỐI mà không đo đúng CHỖ NỐI đang được tuyên bố là đã
canh — `a_batch_with_both_...` đúng về hành vi (kết quả là `Store`, không phải
`ImportUniqueConflict`) nhưng SAI về việc nó có phụ thuộc vào `is_unique_constraint_
violation` hay không, vì một đường tắt khác (`?` trên lỗi trigger) đã quyết định kết quả
trước khi chỗ nối đó kịp chạy. Đối chứng gỡ chỗ nối bắt được đúng khoảng lệch này; một bộ
test xanh một mình thì không.

**Số đo sau vá:** `cargo test --locked` 26 bộ, **0 failed** — đọc MÃ THOÁT thật của `cargo`
(`$?` ngay sau lệnh), không suy từ `tail`. `glossary_exchange_contract.rs` từ 28 lên **39**
ca. `npm run check:i18n`/`check:deps` xanh, 0 miễn trừ mới (346 khoá `vi.json`, tăng từ 345).
`.githooks/pre-push` mười một cổng → vitest 449/449 → build → cargo test, xanh trong ~90-100s.
`grep -rn "std::fs\|PathBuf\|tauri::" exchange.rs` vẫn rỗng.

### 2026-08-24/25 — lượt dựng đầu, ba đối chứng gỡ chỗ nối chạy trên cây thật

Ba đối chứng bắt buộc của §Verification, mỗi ca gỡ rồi khôi phục:

| Gỡ chỗ nối | Kết quả |
|---|---|
| ① Vô hiệu `field_needs_quoting` (luôn `false`) trong `exchange.rs` | ĐỎ — ca vòng tròn (`round_trip_preserves_five_user_visible_fields_and_marks_origin_as_file_import`) trượt vì `parse` đọc lại tệp thấy `CellCountMismatch` (dấu phẩy trong `note`/`translation` bị hiểu nhầm là ranh giới cột). Khôi phục ⇒ xanh. |
| ② Bỏ hai dòng `CREATE UNIQUE INDEX`/`CREATE TRIGGER` khỏi `GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL` | ĐỎ — ca di trú (`migrating_past_the_old_three_value_check_keeps_ids_and_carries_the_watermark_forward`) trượt đúng ở mệnh đề (3): trigger AD-36 không còn chặn lượt lùi `translation` về `NULL`. Khôi phục ⇒ xanh. |
| ③ Bỏ câu `UPDATE sqlite_sequence SET seq = MAX(…)` khỏi cùng hằng | **Lần đầu KHÔNG đỏ** — fixture ban đầu xoá một hàng Ở GIỮA (`id=2` trong ba hàng 1/2/3), và `INSERT … SELECT id, …` tường minh tự nâng mốc `sqlite_sequence` của bảng mới lên đúng `id=3` (lớn nhất còn lại trong dữ liệu chép) dù không có câu `UPDATE` mang mốc riêng — che mất đúng lỗ hổng mà ca này dựng ra để bắt. Sửa fixture: xoá hàng **`id` CAO NHẤT** (`id=3` trong ba hàng) thay vì hàng giữa. Chạy lại ⇒ ĐỎ đúng ca (`id` cấp tiếp theo là 3, tái cấp phát đúng id đã xoá). Khôi phục ⇒ xanh. |

⚠️ **Bài học từ ③, ghi lại vì nó áp cho mọi ca tương lai kiểm mốc `AUTOINCREMENT`:** một `INSERT` tường minh giữ nguyên `id` tự động nâng mốc `sqlite_sequence` của bảng ĐÍCH lên đúng `id` lớn nhất **trong dữ liệu đang chép** — cơ chế này chỉ hụt đúng khi hàng đã xoá là hàng mang `id` CAO NHẤT từng cấp. Một fixture xoá một hàng ở giữa không kiểm được gì về bước "mang mốc riêng"; nó tình cờ đúng nhờ cơ chế phụ, không nhờ bước đang được kiểm.

**Số đo sau vá:** `cargo test --locked` 26 bộ test, **0 failed**; `npm run check:i18n`/`check:deps` xanh, 0 miễn trừ mới; `npm run build` sạch; `npm run test` (vitest) 449/449 không đổi; `.githooks/pre-push` mười một cổng → vitest → build → cargo test xanh trong ~100–190s.

**Chỗ nối chưa có phép kiểm tất định, ghi nợ không làm tròn lên:** nhánh `UPDATE … WHERE id = ?4` trả `changed == 0` trong `import_into_tier` (hàng `Conflict` bị một lượt Xoá khác xoá GIỮA lúc `classify()` và lúc ghi) rơi về `store.write_failed` chung — không có ca test tất định nào dựng được cửa sổ đua đó (cùng lớp khó với `promote_to_global`'s `changed == 0` mà spec 3.9 đã ghi nợ tương tự). Đường mã có (xem doc-comment tại chỗ), chỉ chưa có bằng chứng chạy được.

## Design Notes

**Vòng tròn giữ NĂM trường, không sáu — và đó là một quyết định, không một chỗ hụt.** AC2 của epic đòi *"round-trip đầy đủ, không mất trường nào"*; AC6 đòi *"mục nhập vào từ file mang xuất xứ phân biệt được với mục người dùng tự nhập tay"*. Hai câu đó **không cùng đúng được** cho cột `term_origin`: giữ nguyên nó khi nhập thì AC6 chết. Cách đọc chọn ở đây: `term_origin` trả lời *"thuật ngữ này vào kho CỦA TÔI bằng đường nào"*, nên với một mục tới từ tệp, câu trung thực là **`file_import`**, bất kể nó vào kho người gửi bằng đường nào. Cột vẫn được **xuất** (người nhận đọc được xuất xứ ở phía người gửi như thông tin) nhưng **không** được dùng làm giá trị ghi. ⚠️ **Giới hạn thật, ghi ra thay vì để người sau tự phát hiện:** nhập lại chính tệp mình vừa xuất sẽ đổi `term_origin` của những mục vốn là `manual` thành `file_import`. Đó là hệ quả có chủ của cách đọc trên, không phải một lỗi.

**Vì sao tên cột KHÔNG DẤU, ngược mockup.** `mockups/glossary-manage.html:238` vẽ tiêu đề `thuật_ngữ,bản_dịch,phân_loại,ghi_chú`. Ba lý do cùng chỉ một hướng: Kiểm A của `check:i18n` cho **ĐỎ** với chữ tiếng Việt có dấu ở vị trí mã trong `src-tauri/src/**`; AD-21/NFR16 đã đặt đúng luật đó cho `Category::as_str()` (*"định danh máy đọc, không phải nhãn hiển thị"*); và một tệp trao đổi giữa hai máy là **dây**, cùng hạng với tên trường IPC. Nhãn tiếng Việt là việc của lớp giao diện ở story sau, đọc từ `vi.json`.

**Vì sao bọc nháy kép cho CẢ TSV.** TSV theo lệ không bọc. Nhưng `note` là ô văn bản tự do và một ký tự Tab dán vào đó sẽ phá hàng trong im lặng — đúng lớp lỗi *"rỗng/sai mà không ai báo"*. Một đường bọc dùng chung cho hai dấu phân cách là chỗ duy nhất bảo đảm vòng tròn khép kín; hai đường riêng sẽ lệch nhau ở đúng ô ít ai thử.

**Vì sao phân tích TRỌN rồi mới mở giao dịch.** `Store::write` cho rollback, nên về lý một lỗi giữa chừng vẫn không để lại hàng nào. Nhưng *"không ghi một phần"* của AC5 còn có vế thứ hai: người dùng phải thấy **mọi** lỗi của tệp trong một lượt, không phải sửa dòng 87 rồi mới biết dòng 412 cũng hỏng. Phân tích trọn trước là thứ trả về được **một danh sách** lỗi.

## Verification

**Commands:**
- `cargo test --locked` (trong `src-tauri/`) -- hợp đồng định dạng + hợp đồng di trú + ranh giới; 0 failed.
- `npm run check:i18n` -- Kiểm A: **0** chữ tiếng Việt có dấu ở vị trí mã trong `src-tauri/src/**` (tên cột và giá trị `as_str()` mới đều không dấu).
- `npm run check:deps` -- 0 phụ thuộc mới.
- `.githooks/pre-push` -- mười một cổng → vitest → build → cargo test; exit 0.

**Manual checks:**
- Đối chứng **GỠ chỗ nối** (`AGENTS.md`: một bộ test xanh không chứng minh chỗ nối mới được canh), ba lượt, mỗi lượt khôi phục lại: ① gỡ lớp bọc nháy kép ⇒ ca vòng tròn phải ĐỎ; ② bỏ dòng tạo lại trigger trong bước di trú ⇒ ca *"trigger còn đỏ được"* phải ĐỎ; ③ bỏ dòng mang mốc `sqlite_sequence` ⇒ ca *"`id` không cấp lại"* phải ĐỎ. Ghi số ca vào §Spec Change Log.
- `grep -rn "std::fs\|PathBuf\|tauri::" src-tauri/src/core/glossary/exchange.rs` -- phải rỗng.

## Suggested Review Order

**Ghi vào dữ liệu người dùng — chỗ vòng rà tìm ra lỗi nặng nhất**

- Điểm vào: *lấy của file* ghi ĐÚNG một cột; `note`/`category` không bao giờ bị chạm.
  [`store.rs:1416`](../../src-tauri/src/core/glossary/store.rs#L1416)

- Cả lô đi trong MỘT giao dịch, và phân tích đã xong trọn trước khi nó mở.
  [`store.rs:1357`](../../src-tauri/src/core/glossary/store.rs#L1357)

- Va `UNIQUE` nhận diện bằng `extended_code` của SQLite, không bằng so chuỗi.
  [`mod.rs:139`](../../src-tauri/src/core/store/mod.rs#L139)

**Bước di trú — lần ĐẦU kho này dựng lại một bảng**

- Hằng MỚI, không sửa DDL cũ tại chỗ; mốc `sqlite_sequence` mang theo trước `DROP`.
  [`schema.rs:521`](../../src-tauri/src/core/store/schema.rs#L521)

- Cùng một hằng vào hai thang: `global.db` bước 5…
  [`schema.rs:606`](../../src-tauri/src/core/store/schema.rs#L606)

- …và `project.db` bước 15.
  [`schema.rs:1374`](../../src-tauri/src/core/store/schema.rs#L1374)

- Xuất xứ thứ tư; đường ghi tự đặt nó, không nhận qua tham số (`FR55`).
  [`entry.rs:98`](../../src-tauri/src/core/glossary/entry.rs#L98)

**Định dạng thuần — `&str` vào, `String` ra, 0 phụ thuộc mới**

- Một đường bọc nháy kép dùng chung cho CSV lẫn TSV — chỗ giữ vòng tròn khép kín.
  [`exchange.rs:110`](../../src-tauri/src/core/glossary/exchange.rs#L110)

- Phân tích TRỌN rồi mới trả lỗi, để người dùng thấy mọi lỗi trong một lượt.
  [`exchange.rs:470`](../../src-tauri/src/core/glossary/exchange.rs#L470)

- Dòng logic rỗng không phải một hàng dữ liệu — cùng luật cho cuối tệp và giữa tệp.
  [`exchange.rs:402`](../../src-tauri/src/core/glossary/exchange.rs#L402)

- Ba nhánh *mới* / *giống* / *bất đồng*; mặc định là giữ của tôi.
  [`exchange.rs:679`](../../src-tauri/src/core/glossary/exchange.rs#L679)

- Xuất đọc ĐÚNG một tầng, không qua `list_all_entries` — tránh `source_term` trùng.
  [`store.rs:1283`](../../src-tauri/src/core/glossary/store.rs#L1283)

**Phép kiểm — bốn ca mang nhiều sức nặng nhất**

- Ca canh lỗi phá dữ liệu; gỡ bản vá ra thì nó và một ca nữa ĐỎ, ca cũ vẫn xanh.
  [`glossary_exchange_contract.rs:619`](../../src-tauri/tests/glossary_exchange_contract.rs#L619)

- Di trú: `id` không đổi, index và trigger dựng lại, `id` kế tiếp không tái dùng.
  [`glossary_contract.rs:2802`](../../src-tauri/tests/glossary_contract.rs#L2802)

- `match` toàn phần: một `ParseIssue` mới không biên dịch được cho tới khi ánh xạ khoá.
  [`glossary_exchange_contract.rs:444`](../../src-tauri/tests/glossary_exchange_contract.rs#L444)

- Khoảng hở vòng rà tìm ra: `created_at` đi trọn đường ghi rồi đọc lại từ kho.
  [`glossary_exchange_contract.rs:916`](../../src-tauri/tests/glossary_exchange_contract.rs#L916)
