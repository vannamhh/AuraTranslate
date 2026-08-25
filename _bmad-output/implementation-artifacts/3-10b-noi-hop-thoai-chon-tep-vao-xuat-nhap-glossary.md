---
title: 'Story 3.10b — Nối hộp thoại chọn tệp vào xuất/nhập Glossary'
type: 'feature'
created: '2026-08-25'
status: 'done'
review_loop_iteration: 0
baseline_commit: 'ce5d2760c23444d3aaa8547919fc91015fc658bc'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-10-xuat-va-nhap-glossary-qua-csv-tsv.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `export_tier`/`import_into_tier` chạy được, nghiệm thu bằng 39 ca `cargo test`, và **không một lối vào nào** — `grep -rn "export_tier\|import_into_tier" src-tauri/src/commands/` trả **rỗng**. FR49/NFR9 hứa *"chia sẻ không cần server hay tài khoản"*; một hàm không ai gọi tới không thực hiện được lời hứa đó, và bộ thuật ngữ dựng suốt một Tác phẩm vẫn chết theo máy của người dựng.

**Approach:** Hộp thoại chọn tệp gọi **từ Rust** (AD-48 nhánh (a)) — JavaScript không cầm một quyền plugin nào. Một module I/O **mới** giữ `std::fs` khỏi `exchange.rs` (module đó phải ở lại thuần). Lượt **nhập đi hai nhịp**: nhịp một mở hộp thoại, đọc, phân tích, phân loại rồi **giữ kế hoạch lại trong Rust**; webview chỉ nhận một **mô hình đã kiểm** để vẽ màn hình xem trước, và chỉ gửi lại **bản đồ quyết định**. Lượt **xuất đi một nhịp**: mở hộp thoại lưu, kết xuất, ghi nguyên tử, trả đường dẫn.

## Boundaries & Constraints

**Always:**
- 🔴 **`.plugin(tauri_plugin_dialog::init())` là BẮT BUỘC, kể cả khi chỉ gọi từ Rust.** `DialogExt::dialog()` gọi `self.state::<Dialog<R>>()` — `state()`, không `try_state()`. Thiếu `init()` ⇒ panic ⇒ `panic = "abort"` giết cả tiến trình: không unwind, không `Drop`, không cơ hội flush WAL, và trên Windows release không in ra đâu. Đây là chỗ nối phải có một ca test canh, không phải một dòng tin tưởng.
- 🔴 **`tauri_plugin_fs::init()` KHÔNG được đăng ký** (AD-48 §Rule ③). Crate ở trong cây phụ thuộc **không** bằng lệnh trên dây: `try_fs_scope()` trả `None`, hộp thoại vẫn chạy, **0** lệnh `fs:*` phơi ra.
- 🔴 **`capabilities/main.json` giữ ĐÚNG ba quyền, và `config_invariants.rs::main_capability_grants_the_minimum_and_no_plugin_permission` phải xanh KHÔNG sửa một chữ** (AD-48 §Rule ②). Nếu nó đỏ, nghĩa là ai đó đã cấp một quyền plugin — đó là tín hiệu đúng, không phải một ca cần vá.
- 🔴 **`exchange.rs` ở lại THUẦN.** `grep -rn "std::fs\|PathBuf\|tauri::" src-tauri/src/core/glossary/exchange.rs` phải **vẫn rỗng** — đó là một AC của Story 3.10 đã `done`, và một story mới không được làm một mệnh đề đã đạt hết đúng. Mọi `std::fs` đi vào một module I/O **mới**.
- 🔴 **Nội dung tệp KHÔNG đi ra webview dưới dạng văn bản thô** (AD-48 §Rule ①). Thứ qua dây là **mô hình đã phân tích và đã kiểm** — đúng AD-16 (*"Rust phân tích thành mô hình dữ liệu có cấu trúc; Vue render từ mô hình đó"*). Đường dẫn ĐƯỢC đi ra, và lý do viết ra là AC *"đường dẫn đã ghi hiện ra"*.
- 🔴 **Kế hoạch đã phân tích Ở LẠI RUST giữa hai nhịp nhập.** Webview không bao giờ cầm rồi trả lại `RowPlan`. Nó gửi lại đúng bản đồ quyết định, và một quyết định trỏ tới `source_term` **không có trong lô** là một lỗi tường minh.
- 🔴 **Trần kích thước kiểm bằng `metadata` TRƯỚC khi đọc byte** — khuôn `core/segment/import.rs:250-257`. Đọc trọn rồi mới đo là đúng thứ trần này tồn tại để chặn.
- 🔴 **Phi-UTF-8 từ chối tường minh** qua `String::from_utf8` (**không** `from_utf8_lossy`) — khuôn `import.rs:269`. Không đoán bảng mã; dò bảng mã là Epic 6.
- **Huỷ hộp thoại là `Ok(None)`, không một biến thể lỗi** — huỷ là một lựa chọn, không một thất bại.
- **Tệp xuất ghi NGUYÊN TỬ** (tạm cạnh đích ⇒ `sync_all` ⇒ `rename`), khuôn `core/library/meta.rs:131-166`. Một tệp cụt sau lượt ghi dở là một bản sao lưu người dùng tưởng mình đang có.
- **Khuôn hai lớp cho mọi bề mặt IPC**: hàm thuần nhận `Option<&Store>` + vỏ `#[tauri::command]` mỏng trong `wire`, lấy `State` qua **`try_state`**. Lỗi IPC dựng **chỉ** qua `IpcError::new`. Khoá lỗi mới khai trong `message_keys!` để vào `ALL`; `params` mang **dữ liệu**, không mang câu.
- **Chuỗi literal trong `src-tauri/src/**` viết KHÔNG DẤU.** `tests/**` giữ dấu (miễn trừ có tên).
- **`@click` trong `.vue` là ĐÚNG MỘT `dispatch('<id>')`** (`check:commands` Kiểm A). Quyết định từng hàng bất đồng đi bằng `<input type="radio">` + `@change` — ngoài phạm vi Kiểm A, và bàn phím dùng được sẵn theo ngữ nghĩa `radiogroup`.
- **Mọi danh sách rỗng của overlay mới phải nói được VÌ SAO nó rỗng** — một hàm vị từ theo khuôn `manageEmptyReasonFor` (`glossaryManageState.ts:168`), mỗi lý do một nhánh riêng. Rỗng im lặng là lớp lỗi trung tâm của kho, đã hụt bốn lần.

**Ask First:**
- Nếu `blocking_save_file`/`blocking_pick_file` hoá ra khoá vòng lặp sự kiện trên một trong hai nền tảng — đổi sang hình dạng callback là đổi hình dạng lệnh, không phải một chi tiết cài đặt.
- Nếu delta nhị phân vượt **1 MB** — AD-48 đặt đó làm ngưỡng xét lại, và đường quay lui (`rfd` thẳng) không chạm một chữ nào của ba mệnh đề `Rule`.
- Bất kỳ phụ thuộc nào **ngoài** chín crate đã rà NFR15 ở `ARCHITECTURE-SPINE.md:873-883` (NFR15).
- Nếu ghi nguyên tử bị hệ điều hành từ chối ở một thư mục người dùng chọn (không tạo được tệp tạm cạnh đích).

**Never:**
- Không cấp `dialog:*` hay `fs:*` cho JavaScript; không cài gói npm `@tauri-apps/plugin-*`; không sửa `capabilities/main.json`; không sửa `config_invariants.rs`.
- Không nhân bản bước phân tích định dạng — **một** đường `exchange::parse`, không đường thứ hai.
- Không đọc tệp, ghi tệp, hay giữ quy tắc nghiệp vụ trong `wire`.
- Không đo NFR6 bằng nhị phân sẵn có ở `target/release/` — nó dựng **21-08 16:50**, bốn ngày và một story (`2308f52`) trước HEAD.
- Không thêm cổng `check:*` mới ở story này.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Xuất, chọn đích | Tầng có N mục, người dùng chọn `a.csv` | Tệp ghi đúng nơi chọn; lệnh trả **đường dẫn**; overlay hiện câu *"đã ghi vào …"* | Lỗi ghi ⇒ `ExportWriteFailed`, **0** tệp cụt để lại |
| Xuất, huỷ hộp thoại | Đóng không chọn gì | **Không tệp nào ghi, không lỗi nào hiện** — trả `None` | N/A |
| Xuất, tầng Tác phẩm chưa mở | `tier = Work`, `work` là `None` | `WorkTierUnavailable` **trước** khi mở hộp thoại | 0 lượt ghi |
| Xuất, đuôi tệp quyết dấu phân cách | Người dùng gõ tên `a.tsv` | Dấu phân cách theo **đuôi thật của đường dẫn đã chọn**, không theo trạng thái UI trước đó | Đuôi lạ ⇒ CSV, và nói ra trong câu trạng thái |
| Nhập, chọn tệp | Tệp 604 dòng, 4 cột | Trả xem trước: tên tệp, số hàng, cột nhận ra, cột bỏ qua, đếm *mới*/*giống*/*bất đồng* | N/A |
| Nhập, huỷ hộp thoại | Đóng không chọn gì | Không đọc gì, không lỗi, `None`; **không** kế hoạch nào để lại trong `State` | N/A |
| Nhập, tệp vượt trần | Tệp > **16 MiB** | Từ chối tường minh mang **số byte và trần**; kiểm bằng `metadata`, **0** byte đọc vào bộ nhớ | `ImportFileTooLarge` |
| Nhập, tệp phi-UTF-8 | Byte GBK/Latin-1 | Từ chối tường minh; **không** đoán bảng mã, **không** `_lossy` | `ImportNotUtf8` |
| Nhập, tệp hỏng ở một dòng | Dòng 87 lệch số ô | Danh sách **mọi** lỗi trong một lượt (khuôn `parse` đã có); **0** hàng ghi, không kế hoạch nào giữ lại | `Vec<ParseIssue>` → khoá i18n từng ca |
| Nhập, tệp có cột `term_origin` | Cột có mặt, mang giá trị | Xem trước **NÓI RA** rằng cột này bị đọc rồi bỏ, và mọi mục vào đều mang `file_import` | N/A — đóng `deferred-work.md:6763` |
| Nhập, tệp có cột lạ | Tiêu đề có `usage_count` | Xem trước liệt nó ở *cột bỏ qua* — không im lặng vứt | N/A |
| Nhập, hàng bất đồng | Cùng `source_term`, khác bản dịch | Hiện **cả hai** bản dịch; mặc định **giữ của tôi** | N/A |
| Nhập, xác nhận | Bản đồ quyết định gửi lên | Ghi trong **một** giao dịch; trả `{inserted, updated, identical}`; kế hoạch dọn khỏi `State` | Lỗi giữa chừng ⇒ rollback trọn, kế hoạch **giữ lại** để thử lại |
| Nhập, quyết định trỏ thuật ngữ lạ | Khoá không có trong lô | **Lỗi tường minh** mang thuật ngữ đó — không rơi vào hư không | `ImportDecisionUnknownTerm`, 0 lượt ghi |
| Nhập, xác nhận khi không có lô nào | Gọi nhịp hai mà chưa qua nhịp một | Lỗi tường minh, **0** lượt ghi | `NoPendingImport` |
| Nhập, huỷ ở màn hình xem trước | Bấm Huỷ | Kế hoạch dọn khỏi `State`; **0** lượt ghi; không lỗi | N/A |
| Nhập, mở lô thứ hai khi lô cũ còn treo | Chọn tệp mới trước khi xác nhận lô cũ | Lô mới **thay** lô cũ; lô cũ không bao giờ ghi được nữa | N/A |
| Nhập, tệp 0 hàng dữ liệu | Chỉ hàng tiêu đề | Xem trước hiện **0 hàng, không lỗi** — và nói rõ lý do rỗng là *"tệp không có hàng nào"*, phân biệt với *"tệp hỏng"* | N/A |
| Nhập, mọi hàng đều *giống hệt* | 0 mới, 0 bất đồng | Danh sách bất đồng rỗng, và vị từ nói lý do là *"không mục nào bất đồng"* — không phải *"chưa nạp"* | N/A |
| Nháy kép đặt sai chỗ | `a"b,c` giữa một ô không bọc | Một `"` **không đứng đầu ô** là ký tự thường ở **cả** bước cắt dòng lẫn bước cắt ô — luật của `split_fields` thắng | Đóng `deferred-work.md:6776` |
| Hàng trùng `source_term` **và** `category` lạ | Dòng thứ hai sai cả hai | Báo **cả hai** lỗi cho hàng đó | Đóng `deferred-work.md:6787` |
| Đóng Tác phẩm khi còn lô nhập treo | Lô của tầng `Work`, `work` thành `None` | Lô bị **dọn**; nhịp hai sau đó trả `NoPendingImport` | 0 lượt ghi — không bao giờ ghi vào một kho đã đóng |
| Bàn phím | Không chuột | Mọi thao tác của story làm được bằng bàn phím, gồm chọn giữ-của-tôi / lấy-của-file từng hàng | N/A |

</frozen-after-approval>

## Code Map

**Hộp thoại — API phía Rust, `tauri-plugin-dialog` 2.7.2**
- `~/.cargo/registry/src/index.crates.io-*/tauri-plugin-dialog-2.7.2/src/lib.rs:90` `DialogExt::dialog()` -- gọi `state::<Dialog<R>>()`, **panic nếu chưa `init()`**; `:182` `.file()` → `FileDialogBuilder`.
- `…/lib.rs:439` `add_filter(name, &[&str])` · `:449` `set_directory` · `:456` `set_file_name` · `:483` `set_title`.
- `…/lib.rs:769` `blocking_save_file() -> Option<FilePath>` · `:678` `blocking_pick_file() -> Option<FilePath>`. 🔴 Doc-comment `:662-663` viết nguyên văn *"This is a blocking operation, and should \*NOT\* be used when running on the main thread."* -- bằng chứng dùng được: chính plugin gọi `blocking_pick_file` trong lệnh `open` của nó (`…/src/commands.rs:158,173,190,205`).
- `…/lib.rs:26` `pub use tauri_plugin_fs::FilePath` · `tauri-plugin-fs-2.5.1/src/file_path.rs:54` `into_path() -> Result<PathBuf>` -- **có thể thất bại** (`InvalidPathUrl`); `as_path()` chỉ khớp biến thể `Path`. Đừng `unwrap`.
- `…/src/commands.rs:162` `window.try_fs_scope()` → `None` khi plugin fs chưa đăng ký, và mã đã bọc `if let Some(...)` nên **không** panic -- đây là cơ chế làm §Rule ③ chạy được.

**Chỗ nối phía Rust**
- `src-tauri/src/lib.rs:287` `.plugin(tauri_plugin_wdio_webdriver::init())` -- **khuôn chép** cho `.plugin(tauri_plugin_dialog::init())`, nhưng KHÔNG gác `cfg`.
- `…/lib.rs:302-397` `invoke_handler(generate_handler![…])` -- mười một vỏ glossary ở `:369-396`; lệnh mới nối tiếp sau `:396`.
- `…/lib.rs:335-337` -- ghi rằng command do CHÍNH ứng dụng khai **không** cần ACL trong `capabilities/`; chỉ command của **plugin** mới cần. Đây là lý do `main.json` không phải sửa.
- `…/lib.rs:500` `.manage(Store)` · `:628` `.manage(OpenWorkState)` -- trong `.setup()` từ `:401`. Kho kế hoạch nhập treo cũng `manage` ở đây.
- `src-tauri/src/commands/glossary.rs:861-871` `glossary_list_entries` + vỏ `:879-889` -- **khuôn chép** trọn vẹn: `try_state::<Store>()` ⇒ `else` trả sớm ⇒ `lock().unwrap_or_else(PoisonError::into_inner)` ⇒ gọi hàm thuần. `pub mod wire` mở `:621`.
- `src-tauri/src/core/glossary/entry.rs:158-165` `GlossaryTier` -- **chỉ `Deserialize`**, `#[serde(rename)]` `"global"`/`"work"`.
- `src-tauri/src/core/glossary/store.rs:1283` `export_tier(store: &Store, Delimiter)` -- ⚠️ nhận `&Store` **đã phân giải**, KHÔNG nhận `tier`; hàm thuần mới phải tự làm bước `match tier {…}` theo khuôn `:700-701`.
- `…/store.rs:1357` `import_into_tier(global, work, tier, plans, decisions)` -- nhận `tier`, tự phân giải. Hai chữ ký **lệch nhau**, đừng giả định giống.
- `…/store.rs:411` `GlossaryError` · `:505-530` `impl From<GlossaryError> for IpcError` -- 🔴 đi **qua `IpcError::new`**, không struct literal. Biến thể mới vào cả hai chỗ.

**I/O tệp**
- `src-tauri/src/core/segment/import.rs:57` `MAX_IMPORT_BYTES = 100 MiB` · `:250-257` `metadata` ⇒ so trần ⇒ **rồi mới** `:264` `std::fs::read` · `:269` `String::from_utf8` (không `_lossy`) · `:228-233` `strip_bom` -- **khuôn chép trọn** cho lượt đọc tệp Glossary. ⚠️ Trần 100 MiB là của tài liệu dịch; tệp Glossary nên có trần riêng, nhỏ hơn, và con số đó là một quyết định phải viết ra.
- `src-tauri/src/core/library/meta.rs:131-166` `write_atomic` -- **khuôn chép** ghi nguyên tử: `File::create` tạm ⇒ `write_all` ⇒ `sync_all` ⇒ `rename` ⇒ dọn `.tmp` ở **cả hai** nhánh lỗi ⇒ fsync thư mục cha (`:167+`). ⚠️ Nó ghi vào đường dẫn **nội bộ cố định**; kho **không có tiền lệ** ghi ra đường dẫn tuỳ ý người dùng chọn (`grep -rn "std::fs::write" src-tauri/src/` = **0**).
- `src-tauri/src/core/glossary/exchange.rs:110` `render_tier` · `:470` `parse` · `:679` `classify` · `:312` `ImportRow` · `:331` `ParsedImport{rows, ignored_columns}` · `:619` `ConflictDecision` · `:628` `RowPlanKind` · `:645` `RowPlan` · `:664` `ImportSummary` -- 🔴 **đọc, giữ thuần**. ⚠️ Doc-comment của `ConflictDecision::TakeTheirs` (`:621`) còn viết *"Lấy bản dịch/ghi chú/phân loại từ tệp"* — **hết đúng** từ bản vá P1 (chỉ `translation`); sửa tại chỗ kèm 🔵 và ngày.

**Cổng sẽ phán**
- `src-tauri/tests/config_invariants.rs:347-379` `main_capability_grants_the_minimum_and_no_plugin_permission` -- `assert_eq!` khoá cứng ba quyền. **Không sửa.** Cùng tệp: `capabilities_directory_holds_exactly_the_one_reviewed_file`.
- `src-tauri/tests/glossary_boundary.rs:142` `GLOSSARY_ONLY_SURFACE [&str; 4]` (`insert_manual_entry`·`confirm_translation`·`load_tier`·`insert_candidate`) -- module I/O mới nằm **trong** `core/glossary/**` nên được gọi `load_tier`; `commands/glossary.rs` gọi `export_tier`/`import_into_tier` **không** nằm trong bốn tên này ⇒ không đỏ. `:194` `QUICK_ADD_SURFACE [&str; 12]` -- cổng chỉ đòi 12 tên **có mặt**, không cấm tên khác.
- `src-tauri/tests/ipc_contract.rs:232` `every_message_key_exists_in_vi_json` · `:325` `every_message_key_declares_the_params_its_string_needs` -- cặp `message_keys!` ↔ `vi.json` do **`cargo test`** canh, không `check:i18n`.
- `scripts/check-deps.mjs:163,165` -- hai hàng `BANNED_CRATES` phải gỡ; `:150-156` chú thích lý do phải viết lại (canh **mã trong nhị phân**, không phải bề mặt IPC). Bốn tên còn lại đứng nguyên. `:62` `RUST_TREE_FLOOR = 200` là cận **dưới**, chín crate mới không chạm.
- `scripts/check-commands.mjs:759` Kiểm A -- `@click` = đúng một `dispatch`; `@change`/`@submit`/`@keydown` **ngoài** phạm vi (`:33-36`). Kiểm B (văn phạm id + id tồn tại), Kiểm E (nhãn `command.<id>` có trong `vi.json`).
- `scripts/check-i18n.mjs:866` Kiểm A (không chữ có dấu ở vị trí mã) · `:927` Kiểm A2 (mọi text node template đi qua `t()`) · `:1045` Kiểm B (`vi.json` **phẳng**, khoá chấm có tiền tố).
- `ARCHITECTURE-SPINE.md:873-883` -- chín hàng bảng Stack **đã có** kèm giấy phép (`notify` là **CC0-1.0**, không MIT); `:887` ghi `trash` là `dev-dependencies` chỉ Windows, không vào cây phát hành; `:895` xác nhận hai tên đã rời danh sách *"đã loại"*.
- `src-tauri/SECURITY-NOTES.md:81` -- còn liệt `tauri-plugin-dialog` là đã loại, **mâu thuẫn AD-48**; sửa tại chỗ kèm 🔵 và ngày, không xoá.
- `src-tauri/Cargo.toml:29-96` `[dependencies]` 12 mục, `tauri = "=2.11.5"` (`:33`), `[features]` `:97-100` **không** có `default` -- ghim `=2.7.2` theo lệ `=` của kho.

**Nửa frontend**
- `src/GlossaryManageOverlay.vue` (649 dòng) -- **khuôn chép** overlay: `focusableWithin` `:96-103` · `trapTab` `:106-122` · `focusReturnTargetOnOpen` `:69-94` · `role="dialog" aria-modal` `:227` · `@keydown.esc` `:223` · `role="status"` `:280,377` / `role="alert"` `:287,366` · `.btnrow` nút `:390-404`. Hai nút *Xuất CSV*/*Nhập CSV* vào hàng nút này. Tiền tố lớp riêng cho overlay mới (`gm-`/`gq-` đã dùng).
- `src/glossaryManageState.ts` -- `ref` một-dòng-một (`check:panel-refs` Kiểm 5) `:70-71` · chống đua `mySequence` `:102,186-199` · `reset*()` `:445-465` bắt buộc · 🔴 `manageEmptyReasonFor` `:168-179` và bài học `:122-133` (`totalCount` phải là export RIÊNG, không suy từ `filteredCount`).
- `src/config/glossary.ts` -- **adapter IPC**, chỗ DUY NHẤT gọi `invoke()`; không bao giờ ném, trả `{ value | null, error: IpcError | null }`. Hàm mới cho ba lệnh vào đây. ⚠️ `invoke()` gửi tham số **camelCase**, nhưng trường struct TRẢ VỀ giữ **snake_case**.
- `src/commands/index.ts:589-617` `CommandDeps` · `:1780-1895` khối `glossary.manage.*` · `:636` `portMissing` -- lệnh mới chèn sau `:1895`, field optional sau `:617`. 🔴 `index.ts` nạp bằng **Node thuần**: không import giá trị từ `vue`, không `enum`, handler **tiêm** qua `CommandDeps` từ `src/main.ts`.
- `src/i18n/vi.json` -- **phẳng**, 346 khoá; `command.glossary.manage.*` `:104-112`; `glossary.manage.*` `:310-347`. `t()` `src/i18n/index.ts:42` · `tError()` `:72`.
- `src/StatusBar.vue:329-392` -- năm hạng ưu tiên cố định, thanh 34px một mệnh đề, đều gắn luồng TRANG SOẠN. ⇒ Câu *"đã ghi vào …"* **không** mượn chỗ ở đây; nó hiện trong chính overlay, khuôn `.gm-status` `:366-387`.
- `tests/frontend/glossaryManage.test.ts` (536 dòng, 25 ca) -- khuôn: `mount` của `@vue/test-utils` trên `happy-dom`; mock **module adapter** (`vi.mock('../../src/config/glossary')`), KHÔNG mock `@tauri-apps/api`; `freshState()` `:56-68` gọi **trước** `mockResolvedValue`.
- `mockups/glossary-manage.html:210-216` (hai nút) · `:219-268` (màn hình nhập: tên tệp, *"604 dòng · 4 cột nhận ra được"*, cột thiếu tự điền, hai `pill` chọn tầng, hàng `.dupe` *"đang có X · file ghi Y"* + hai chip, biến thể *giống nhau*/*thêm vào*, nút *"Nhập 597 mục mới · giữ 7 mục đang có"* + *Huỷ*, câu hint *"Không ghi xuống đĩa trước khi bấm"*).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/Cargo.toml` -- thêm `tauri-plugin-dialog = "=2.7.2"` cuối `[dependencies]` -- ghim `=` vì `"2.7.2"` nghĩa là `^2.7.2` và lock chỉ giữ số đúng tới lần `cargo update` đầu tiên. Chín crate đã rà NFR15 xong ở spine, **không** rà lại, và **không** thêm tên thứ mười.
- [x] 🔴 **PHÉP ĐO NFR6 CHẠY NGAY SAU HAI TASK ĐẦU, KHÔNG ĐỂ CUỐI STORY** -- dựng release ở HEAD (baseline), rồi dựng lại khi cây **chỉ khác đúng** dòng `Cargo.toml` trên cộng `.plugin(tauri_plugin_dialog::init())`, **cùng một `dist/` không dựng lại giữa hai lượt**; ghi hiệu số byte -- Tauri **nhúng `dist/` vào nhị phân**, nên đo ở cuối story sẽ cộng phần overlay Vue mới vào phần chín crate rồi đem tổng đó so ngưỡng 1 MB của AD-48. Hai điểm đo phải khác nhau đúng một phụ thuộc, nếu không con số không trả lời được câu hỏi AD-48 hỏi.
- [x] `scripts/check-deps.mjs` -- gỡ `tauri-plugin-fs` (`:163`) và `tauri-plugin-dialog` (`:165`) khỏi `BANNED_CRATES`; viết lại chú thích `:150-156` kèm ngày và lý do lật -- 🔴 cổng canh **mã trong nhị phân** (NFR6 + bề mặt tấn công), còn **bề mặt IPC** (NFR11) do `config_invariants.rs` canh; hai cổng, hai mệnh đề, không cái nào thay được cái kia. Bốn tên còn lại và lý do của chúng **không đổi một chữ**.
- [x] `src-tauri/SECURITY-NOTES.md` -- sửa tại chỗ dòng liệt `tauri-plugin-dialog` là đã loại, kèm 🔵 và ngày, trỏ AD-48; bổ sung vào §*"Vì sao không có plugin `fs`"* rằng crate fs **có trong cây** nhưng **không `init()`** -- mệnh đề hết đúng thì sửa tại chỗ, đừng xoá và đừng để nó lặng lẽ sai.
- [x] `src-tauri/src/lib.rs` -- `.plugin(tauri_plugin_dialog::init())` (không gác `cfg`), và đăng ký các vỏ lệnh mới trong `generate_handler!` -- thiếu `init()` thì `DialogExt::dialog()` panic, mà `panic = "abort"` biến nó thành cái chết của tiến trình. 🔴 **Không** gọi `tauri_plugin_fs::init()`.
- [x] `src-tauri/src/core/glossary/exchange_io.rs` (**mới**) -- đọc tệp (`metadata` ⇒ trần ⇒ `read` ⇒ `from_utf8` ⇒ `strip_bom`) và ghi nguyên tử (tạm ⇒ `sync_all` ⇒ `rename` ⇒ dọn `.tmp` hai nhánh) -- đặt `std::fs` ở đây là thứ giữ `exchange.rs` thuần, tức giữ một AC của Story 3.10 còn đúng. Khai vào `mod.rs` cạnh `exchange`.
- [x] `src-tauri/src/core/glossary/store.rs` -- biến thể `GlossaryError` mới cho các ca §I/O Matrix (tệp quá lớn · phi-UTF-8 · ghi thất bại · quyết định trỏ thuật ngữ lạ · không có lô treo) và nhánh `From<GlossaryError> for IpcError` tương ứng -- đi **qua `IpcError::new`**; `params` mang số byte và tên thuật ngữ, không mang câu.
- [x] `src-tauri/src/core/glossary/exchange.rs` -- ① bước cắt DÒNG áp **đúng** luật của bước cắt Ô: một `"` chỉ mở ô bọc khi đứng **ngay đầu ô**; ② ô nhớ `seen` điền cho mọi hàng đã qua bước cắt ô, nên một hàng trùng **và** sai `category` góp **cả hai** lỗi; ③ sửa doc-comment `ConflictDecision::TakeTheirs` đã hết đúng, kèm 🔵 và ngày -- ① và ② là hai món nợ ghi đích danh story này (`deferred-work.md:6776`, `:6787`), và ① kích hoạt đúng lúc này vì đây là lần ĐẦU một tệp do người khác sinh ra đi vào hệ thống. Module vẫn **không** được chạm `std::fs`/`PathBuf`/`tauri::`.
- [x] `src-tauri/src/commands/glossary.rs` -- ba hàm thuần + ba vỏ `wire`: xuất (một nhịp), mở-và-xem-trước lượt nhập (nhịp một), xác nhận lượt nhập (nhịp hai), cộng một đường huỷ lô treo -- hàm thuần tự phân giải `tier → &Store` cho đường xuất (chữ ký `export_tier` **không** nhận `tier`, khác `import_into_tier`); vỏ lấy `State` qua **`try_state`**, mở hộp thoại, và **không giữ một quy tắc nào**.
- [x] `src-tauri/src/lib.rs` hoặc module lồng -- kho **kế hoạch nhập đang treo** dưới `.manage(...)`: đường dẫn + tầng + `Vec<RowPlan>`; lô của tầng `Work` bị **dọn khi đóng Tác phẩm** (nối vào đúng chỗ `OpenWorkState` chuyển về `None`) -- kế hoạch ở lại Rust là thứ giữ nội dung tệp khỏi webview (AD-48 §Rule ①) **và** làm một quyết định trỏ thuật ngữ lạ thành lỗi bắt được (`deferred-work.md:6798`); còn bước dọn là thứ giữ một lô cũ khỏi ghi vào một kho đã đóng.
- [x] `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` -- khoá mới cho từng lỗi mới, khai trong `message_keys!` kèm bảng tham số, và **một câu tiếng Việt cho mỗi khoá dùng đúng tham số đã khai** -- `ipc_contract.rs:232` và `:325` cho đỏ ở `cargo test`, không ở `check:i18n`; một khoá quên câu chỉ lộ ra ở khâu cuối.
- [x] `src/config/glossary.ts` -- adapter cho ba lệnh mới -- **không bao giờ ném**: một `invoke`, một `try/catch`, trả hình dạng ba trạng thái. Tham số đi **camelCase**, trường trả về giữ **snake_case**.
- [x] `src/glossaryImportState.ts` (**mới**) -- state của overlay xem trước: lô đang treo, bản đồ quyết định, cờ đang ghi, một vị từ **lý do rỗng** phân biệt *chưa nạp* / *IPC không có* / *tệp không có hàng nào* / *không mục nào bất đồng* -- khuôn `manageEmptyReasonFor`; `ref` một-dòng-một; `mySequence` chống đua; `reset*()` bắt buộc.
- [x] `src/GlossaryImportOverlay.vue` (**mới**) -- màn hình xem trước theo mockup `:219-268`, trừ khối bốn dòng đầu tệp (xem §Design Notes) -- mỗi hàng bất đồng là một `radiogroup` hai lựa chọn, mặc định **giữ của tôi**; `@change`, không `@click`, nên Kiểm A không phải nới và bàn phím có sẵn ngữ nghĩa.
- [x] `src/GlossaryManageOverlay.vue` -- hai nút *Xuất CSV* / *Nhập CSV* vào `.btnrow`, cộng một `<p role="status">` cho câu *"đã ghi vào …"* -- mỗi `@click` là **đúng một** `dispatch('<id>')`; `StatusBar` không nhận câu này (§Code Map).
- [x] `src/commands/index.ts` + `src/main.ts` -- lệnh mới (mở xuất · mở nhập · xác nhận nhập · huỷ nhập) cộng field `CommandDeps` và chỗ tiêm ở `main.ts` -- `index.ts` nạp bằng Node thuần nên handler phải **tiêm**, không import; thiếu nhãn `command.<id>` trong `vi.json` thì Kiểm E đỏ.
- [x] `src-tauri/tests/glossary_exchange_contract.rs` + tệp hợp đồng mới cho `exchange_io` -- phủ trọn §I/O Matrix: trần kích thước (kiểm **trước** khi đọc), phi-UTF-8, ghi nguyên tử không để lại `.tmp`, quyết định trỏ thuật ngữ lạ, xác nhận khi không có lô, lô thứ hai thay lô thứ nhất, nháy kép đặt sai chỗ, trùng + `category` lạ.
- [x] `src-tauri/tests/config_invariants.rs` -- ca **mới** khẳng định `tauri_plugin_dialog::init()` **có** đăng ký và `tauri_plugin_fs::init()` **không** xuất hiện ở bất kỳ đâu trong `src-tauri/src/**` -- 🔴 `main_capability_grants_the_minimum_and_no_plugin_permission` **không sửa một chữ**; ca mới đứng cạnh nó, không thay nó.
- [x] `tests/frontend/glossaryImportPreview.test.ts` (**mới**) -- state + overlay: mặc định *giữ của tôi*, đổi quyết định, bốn nhánh lý-do-rỗng, huỷ dọn lô -- mock **module adapter**, `freshState()` trước `mockResolvedValue`.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- ghi **số byte thật** của delta nhị phân kèm ngày và phiên bản toolchain; đóng bằng chữ các mục `:6752` (cả hai vế) · `:6763` · `:6776` · `:6787` · `:6798` -- 🔴 baseline phải dựng **lại ở HEAD trước khi thêm phụ thuộc**; nhị phân sẵn có dựng 21-08, bốn ngày và một story trước HEAD.

**Acceptance Criteria:**
- Given hai bản dựng release **chỉ khác nhau đúng một phụ thuộc** (cùng `dist/`, không dựng lại giữa hai lượt), when so kích thước nhị phân, then delta ghi vào `deferred-work.md` bằng **byte** kèm ngày và phiên bản toolchain; và nếu vượt **1 MB** thì dừng và trình số cho Ice (AD-48 đặt sẵn đường quay lui `rfd` thẳng). 🔴 Một delta đo ở CUỐI story gộp cả overlay Vue mới — nó không trả lời được câu hỏi AD-48 hỏi và **không** dùng thay được.
- Given `cargo test --locked`, when chạy, then xanh — gồm `main_capability_grants_the_minimum_and_no_plugin_permission` **không sửa một chữ**, `every_message_key_exists_in_vi_json`, và `every_message_key_declares_the_params_its_string_needs`.
- Given `grep -rn "tauri_plugin_fs::init" src-tauri/src/`, when chạy, then **rỗng**; và `capabilities/main.json` vẫn mang **đúng ba** quyền.
- Given `npm run check:deps`, when chạy, then xanh với `BANNED_CRATES` còn **bốn** tên, và chú thích của Kiểm 1 nói đúng thứ nó canh.
- Given `grep -rn "std::fs\|PathBuf\|tauri::" src-tauri/src/core/glossary/exchange.rs`, when chạy, then **vẫn rỗng**.
- Given người dùng chỉ dùng bàn phím, when xuất và nhập trọn một vòng gồm quyết một hàng bất đồng, then làm được hết, không cần chuột.
- Given `.githooks/pre-push`, when chạy, then exit 0; và lượt CI **cả hai nền tảng** đọc xanh trước khi kết luận — `pre-push` chỉ nói về macOS của Ice.

## Design Notes

**Vì sao lượt nhập đi HAI NHỊP với kế hoạch ở lại Rust.** AD-48 §Rule ① cấm nội dung tệp đi ra webview. Một thiết kế một-nhịp buộc phải gửi 604 `RowPlan` ra JS rồi nhận ngược lại — tức chở nguyên nội dung tệp qua dây hai lần, và tin vào thứ quay về. Giữ kế hoạch trong `State` của Rust giải cả ba chuyện cùng lúc: nội dung tệp không rời Rust; webview chỉ cầm một **mô hình đã kiểm** để vẽ (đúng AD-16); và một quyết định trỏ tới `source_term` không có trong lô trở thành **lỗi bắt được** thay vì rơi vào hư không — đúng món nợ `deferred-work.md:6798`.

**Chỗ CỐ Ý lệch mockup, và vì sao.** `glossary-manage.html:235-239` vẽ một khối xem trước **bốn dòng đầu tệp nguyên văn**. Đó là văn bản thô của một tệp lạ đi thẳng ra webview — đúng thứ AD-48 §Rule ① cấm. Thay bằng: **tên tệp** + *"N dòng · M cột nhận ra được"* + danh sách **cột bỏ qua** + danh sách **cột đọc rồi bỏ giá trị** (`term_origin`). Người dùng vẫn xác nhận được *"đúng tệp mình chọn"* mà không cần một byte thô nào qua dây. Cùng hạng quyết định với việc Story 3.10 cố ý lệch mockup ở tên cột.

**`term_origin` bị đọc rồi bỏ — chỗ NÓI RA nằm ở đây.** `exchange.rs` liệt `term_origin` trong `COLUMNS` nên nó **không** rơi vào `ignored_columns`; giá trị thì bị bỏ có chủ ý (mọi mục vào đều mang `file_import`, §Design Notes của Story 3.10). Cho tới story này, không màn hình nào tồn tại để nói điều đó ra ⇒ một người sửa tay cột đó rồi nhập lại không nhận được một câu nào. Overlay xem trước là chỗ **duy nhất** hiển thị được nó, nên nó là một hạng riêng, không gộp vào *cột bỏ qua*.

**Trần 16 MiB, và đây là phép tính đứng sau nó.** `import.rs:57` đặt 100 MiB cho **tài liệu dịch**; một tệp Glossary 100 MiB là một tệp sai, không phải một tệp lớn. Một hàng Glossary thật cỡ ~200 byte (sáu cột, `note` là chỗ dài nhất), nên 16 MiB ≈ **80.000 hàng** — xa trên mọi bộ thuật ngữ người thật dựng được, và mockup lấy ví dụ 604 dòng. Vế còn lại là bộ nhớ: văn bản đọc vào `String`, rồi `Vec<ImportRow>`, rồi `Vec<RowPlan>` — hệ số ~3-5 lần, tức đỉnh ~80 MiB cho một tệp chạm trần. Chấp nhận được, và trần cao hơn thì không. Kiểm bằng `metadata` **trước** khi đọc: kiểm sau `read` là đã nạp trọn tệp vào bộ nhớ rồi mới nói *"quá lớn"*.

**Hai món nợ của `exchange.rs` được chốt thế nào, và vì sao chốt thế.** ① *Nháy kép đặt sai chỗ*: luật của `split_fields` thắng — một `"` chỉ mở ô bọc khi đứng **ngay đầu ô**. Lý do: đó là luật RFC 4180 thật, **và** là luật mà chính `render_tier` sinh ra, nên chọn nó giữ vòng tròn xuất→nhập khép kín trong khi luật lỏng hơn của bước cắt dòng không khớp thứ kho tự sinh. ② *Một hàng báo mấy lỗi*: báo **cả hai**. §Design Notes của Story 3.10 đã chốt nguyên tắc *"người dùng phải thấy MỌI lỗi của tệp trong một lượt, không phải sửa dòng 87 rồi mới biết dòng 412 cũng hỏng"* — sửa `category` rồi nhập lại mới biết hàng đó còn trùng là **đúng lãng phí ấy, lùi xuống một cấp**. Hôm nay hành vi ngược lại, và nó là hệ quả của thứ tự viết mã chứ không của một lựa chọn (`deferred-work.md:6787` nói đúng thế).

**Ghi nguyên tử cho một đường dẫn người dùng chọn — chỗ kho chưa có tiền lệ.** `grep -rn "std::fs::write" src-tauri/src/` = **0**; khuôn duy nhất (`meta.rs:131-166`) ghi vào đường dẫn nội bộ cố định. Rủi ro mới: tệp `.tmp` cạnh đích ở một thư mục do người dùng chọn có thể bị hệ điều hành từ chối. Đó là một mục §Ask First, không phải một giả định. Lý do vẫn chọn nguyên tử: một tệp cụt sau lượt ghi dở là một **bản sao lưu người dùng tưởng mình đang có** — cùng lớp *"hỏng trong im lặng"* mà kho đã hụt bốn lần.

**`blocking_*` và luồng chính.** Doc-comment của crate nói thẳng *"should \*NOT\* be used when running on the main thread"*. Bằng chứng dùng được: chính plugin gọi `blocking_pick_file` bên trong lệnh `open` của nó (`commands.rs:158,173,190,205`), và Tauri chạy `#[tauri::command]` đồng bộ ngoài luồng UI. Nhưng đó là một **suy luận từ hai dấu hiệu**, không phải một phép đo — nên nó vào §Ask First, và lượt nghiệm thu phải mở hộp thoại **thật** trên cửa sổ thật, không chỉ chạy `cargo test`.

**Quyết định từng hàng KHÔNG đi qua `dispatch`.** `check:commands` Kiểm A đòi mọi `@click` là đúng một `dispatch('<id>')`, và `dispatch` **không nhận tham số** — nên một chip *lấy của file* cho hàng thứ 43 không diễn đạt được bằng một lệnh. Dùng `<input type="radio">` trong một `radiogroup` với `@change`: nằm ngoài phạm vi Kiểm A (`check-commands.mjs:33-36`), có tiền lệ trong chính `GlossaryManageOverlay.vue` (các `<select>` dùng `@change`), và mang sẵn ngữ nghĩa bàn phím mà AC cuối của story đòi. Bốn lệnh registry là bốn thao tác **cả-màn-hình**: mở xuất · mở nhập · xác nhận · huỷ.

## Spec Change Log

### 2026-08-25 — lượt dựng đầu, đo NFR6 và bốn đối chứng gỡ chỗ nối bắt buộc

**Số đo NFR6 (nhị phân, byte), như §Verification đòi:**

| Điểm đo | Cây | Byte | Ngày/giờ | Toolchain |
|---|---|---|---|---|
| Baseline (HEAD `ce5d276`, chỉ mã) | `dist/` 10:07 | **7.555.496** | 2026-08-25 10:09 | `rustc 1.97.1 (8bab26f4f)` · `cargo 1.97.1` |
| Sau ĐÚNG một phụ thuộc (`Cargo.toml` + `.plugin(tauri_plugin_dialog::init())`) | CÙNG `dist/` | **7.711.888** | 2026-08-25 10:26 | như trên |

**Delta = 156.392 byte (≈152,7 KiB)** — dưới xa ngưỡng xét lại 1 MB của AD-48. `cargo build --release` (không `--locked`, để khoá `Cargo.lock` lần đầu) kéo thêm `rfd 0.16.0` cùng lượt — phụ thuộc nội bộ của chính `tauri-plugin-dialog`, đã tính trong delta trên, không phải một crate thứ mười tự thêm.

**Bốn đối chứng GỠ chỗ nối bắt buộc của §Verification, mỗi ca gỡ rồi khôi phục, chạy trên cây thật:**

| Gỡ chỗ nối | Kết quả |
|---|---|
| ① Gỡ `.plugin(tauri_plugin_dialog::init())` khỏi `lib.rs` | ĐỎ — `config_invariants.rs::the_dialog_plugin_is_registered_and_the_fs_plugin_is_never_initialized` báo đúng câu *"khong tim thay `.plugin(tauri_plugin_dialog::init())`"*. Khôi phục ⇒ xanh. |
| ② Bỏ bước kiểm trần bằng `metadata` (`exchange_io::read_import_file`) | ĐỎ — `glossary_import_dialog_contract.rs::a_file_over_the_sixteen_mib_cap_is_refused_by_size_before_content_is_touched` trượt (`assert_eq!` mong `ImportTooLarge`, tệp giờ đi tiếp tới bước phân tích và trượt với `GlossaryImportDelimiterUnresolved`). Khôi phục ⇒ xanh. |
| ③ Bỏ dòng `remove_file(&tmp)` ở nhánh `rename` trượt (`exchange_io::write_export_file`) | ĐỎ — ca MỚI `write_export_file_cleans_up_the_tmp_file_when_rename_onto_an_existing_directory_fails` (dựng bằng cách nhắm đích vào một thư mục đang có — nhánh `File::create` một mình không đủ, phải là nhánh `rename` mới thật sự kiểm được dòng dọn này) báo đúng *"tep tam .tmp khong duoc de lai"*. Khôi phục ⇒ xanh. |
| ④ Bỏ bước kiểm `decisions` khớp lô đang treo (`commands::glossary::glossary_confirm_import`) | ĐỎ — `glossary_import_dialog_contract.rs::confirming_with_a_decision_pointing_at_an_unknown_term_fails_and_keeps_the_batch` trượt: một quyết định trỏ thuật ngữ lạ bị ÂM THẦM bỏ qua và giao dịch vẫn ghi thành công (`ImportSummaryWire { inserted: 1, .. }`) thay vì bị từ chối. Khôi phục ⇒ xanh. |

**Hai đối chứng thêm, không bắt buộc bởi §Verification nhưng đóng đúng hai món nợ mà spec giao cho story này (`deferred-work.md:6776`, `:6787`):**

| Gỡ chỗ nối | Kết quả |
|---|---|
| Trả `split_first_logical_line` về luật cũ (đảo `in_quotes` ở MỌI `"`, không chỉ ở đầu ô) | ĐỎ — `a_stray_quote_not_at_the_start_of_a_field_is_literal_in_both_the_line_and_field_splitter` trượt với `CellCountMismatch { line: 2, expected: 2, found: 3 }` (dòng 2 và 3 bị nuốt vào cùng một "dòng logic"). Khôi phục ⇒ xanh. |
| Trả nhánh `category`/`created_at` của `parse()` về `continue` sớm (bỏ `row_ok`) | Không đo lại bằng gỡ-chỗ-nối riêng (thay đổi cấu trúc lớn hơn một dòng); đối chứng gián tiếp qua chính ca `a_row_that_is_both_a_duplicate_and_has_an_unknown_category_reports_both_issues` — ca này KHÔNG tồn tại được trên mã cũ (một hàng trùng+category lạ chỉ báo được MỘT lỗi ở mã cũ), nên bản thân việc ca này XANH trên mã mới đã là bằng chứng của chỗ nối. |

**Số đo sau vá:** `cargo test --locked` (`src-tauri/`) — 0 failed, đọc mã thoát thật. `npm run check:deps` xanh, `BANNED_CRATES` còn bốn tên. `npm run check:commands`/`check:i18n`/`check:tokens`/`check:layout`/`check:panel-refs`/`check:lint` xanh, 0 miễn trừ mới (379 khoá `vi.json`, tăng từ 350; 79 command đăng ký, tăng từ 75). `npm run test` (vitest) 458/458 (449 cũ + 9 mới). `npm run build` sạch. `.githooks/pre-push` **mười một** cổng → vitest → build → cargo test xanh trong 131s (macOS của Ice). 🔵 *(Sửa 2026-08-25 ở lượt đối chứng: bản ghi đầu viết "mười bốn cổng" — đếm nhầm ba bước `test`/`build`/`cargo test` thành cổng. Dòng tiêu đề của chính hook và `AGENTS.md` đều nói mười một. Lượt chạy lại độc lập: exit 0, **162s**.)*

**Chưa nghiệm thu được, ghi ra thay vì đánh dấu đạt:** hộp thoại **thật** trên một cửa sổ Tauri thật (`npm run tauri dev`) chưa mở tay được trong phiên này — `cargo test` không dựng cửa sổ nên không nói gì về việc `blocking_save_file`/`blocking_pick_file` có thật sự không khoá luồng UI hay không (§Ask First của spec). CI hai nền tảng chưa đọc được vì chưa có lượt push nào — `pre-push` chỉ nói về macOS.

**Một quyết định KHÔNG hiển nhiên, ghi ra thay vì để người sau tự phát hiện — vị trí của "hai pill chọn tầng".** Mockup (`glossary-manage.html:219-268`, dẫn ở Code Map) vẽ hai pill chọn tầng NGAY TRONG màn hình xem trước lượt nhập. Cài đặt ở đây đặt bộ chọn tầng (một `radiogroup` hai lựa chọn) NGOÀI, trên `.gm-actions` của `GlossaryManageOverlay.vue`, dùng CHUNG cho cả Xuất lẫn Nhập — chốt TRƯỚC khi hộp thoại chọn tệp mở, không đổi được sau khi đã xem trước. Lý do: `classify()` (nhịp một) cần biết tầng ĐÍCH để so `existing`, và nếu pill nằm trong màn xem trước thì đổi tầng phải kích một round-trip Rust THỨ HAI để phân loại lại — một lệnh thứ năm không nằm trong "bốn lệnh registry" mà chính §Design Notes của spec chốt. Đặt tầng trước khi mở hộp thoại giữ đúng bốn lệnh, đúng thiết kế hai-nhịp (kế hoạch khoá vào MỘT tầng ngay từ nhịp một), và không mất khả năng chọn tầng — chỉ đổi THỜI ĐIỂM chọn. Trong màn xem trước, tầng hiện ra như một DÒNG DỮ LIỆU (`glossary.import.rows_label`/tệp) chứ không phải một điều khiển tương tác thứ hai. Đây là một quyết định thực thi trong phạm vi được giao, không phải một lần renegotiate `<frozen-after-approval>` — cờ lên để Ice/QA đối chiếu nếu hình dạng này không đúng ý.

### 2026-08-25 (muộn hơn) — vòng rà ba lớp, chín bản vá, ba đối chứng gỡ chỗ nối bắt buộc

Không `intent_gap`, không `bad_spec` — spec không sửa. Chín bản vá dưới đây, mỗi mục đóng
đúng một chỗ hỏng của lượt dựng đầu.

**P1 (HẠNG CAO NHẤT).** `wire::glossary_export_tier`/`wire::glossary_open_import_preview`
giữ `MutexGuard` của `OpenWorkState` xuyên suốt `blocking_save_file()`/`blocking_pick_file()`
— hộp thoại hệ điều hành mở bao lâu thì khoá giữ bấy lâu, chặn MỌI lệnh khác cần
`OpenWorkState` (gồm cả đường flush AD-35, trần cứng 5s). Sửa: `work_tier_is_open()` (khoá
đọc-nhả TRONG MỘT biểu thức, gọi TRƯỚC dialog) thay cho giữ `guard` qua dialog; sau khi
dialog trả về, `OpenWorkState` khoá LẦN THỨ HAI, MỚI — Tác phẩm đóng giữa lúc hộp thoại mở
thì lần khoá thứ hai đọc được `None`, hàm thuần tự trả `WorkTierUnavailable`, 0 lượt ghi.
Cùng lượt: `try_state::<Store>()`/`try_state::<PendingImportState>()` dời lên TRƯỚC dialog.

**P2.** Adapter (`config/glossary.ts`) trả `{ preview: null, error: null }` cho CẢ "huỷ hộp
thoại" LẪN "không có cầu IPC" — nhánh `'ipc_unavailable'` của `glossaryImportState.ts`
không bao giờ tới được, và ngoài Tauri bấm "Nhập CSV" không xảy ra gì, không nói gì. Sửa:
`GlossaryExportResult`/`GlossaryImportPreviewResult` đổi sang `outcome`-tagged union bốn
nhánh (`'done'|'cancelled'|'ipc_unavailable'|'error'` và
`'loaded'|'cancelled'|'ipc_unavailable'|'error'`); `'ipc_unavailable'` nay MỞ lớp phủ và
nói ra, `'cancelled'` vẫn im lặng có chủ. Áp CẢ cho `exportGlossaryManageTier` (cùng lớp
lỗi, không nằm trong phạm vi P2 nêu gốc nhưng để lại sẽ lệch đúng một nửa của cùng một cặp).

**P3.** `exportGlossaryManageTier` không xoá `exportedPath` khi lượt xuất MỚI bắt đầu —
xuất xong vào X, mở lại rồi HUỶ, overlay vẫn đọc "Đã ghi vào X". Sửa: `exportedPath.value =
null` ngay khi lượt xuất mới bắt đầu (trước khi hộp thoại mở), không chỉ khi lượt mới
thành công.

**P4.** `glossary_export_tier` đăng ký bộ lọc TSV trong khi tên mặc định luôn `….csv` — chọn
bộ lọc TSV vẫn nhận tên `.csv`. `rfd`/`blocking_save_file` không phát sự kiện đổi bộ lọc để
phản ứng động, nên chọn MỘT: gỡ bộ lọc TSV, giữ CSV. Dấu phân cách vẫn suy từ đuôi THẬT của
đường dẫn đã chọn (không đổi) — người dùng muốn TSV vẫn gõ `.tsv` được, chỉ không còn một bộ
lọc gợi ý sai.

**P5.** `glossary_confirm_import` dựng `known_terms` từ MỌI hàng của lô (`New`/`Identical`/
`Conflict`), nên một quyết định trỏ vào hàng `New`/`Identical` qua được phép kiểm rồi KHÔNG
có tác dụng gì — `import_into_tier` chỉ tra `decisions` cho nhánh `Conflict`. Sửa: siết
`known_terms` xuống đúng `source_term` của các hàng `RowPlanKind::Conflict`.

**P6.** Không ca nào trong kho giải mã `"keep_mine"`/`"take_theirs"` qua serde thật — mọi ca
Rust dựng biến thể thẳng, vitest mock ở biên adapter. Thêm hai ca: giải mã trực tiếp hai
chuỗi NGUYÊN VĂN mà `src/config/glossary.ts` gửi (không suy từ Rust — suy vậy sẽ tự đúng dù
`rename` sai) cộng đối chứng ÂM (`"keep-mine"`/`"KeepMine"`/`"takeTheirs"` đều bị từ chối);
và giải mã cả HÌNH DẠNG object phẳng `{ [source_term]: decision }` đúng như trên dây.

**P7.** `GlossaryError::ImportReadFailed`/`ExportWriteFailed` → `IpcError` chưa lần nào chạy
qua chính đường chuyển đổi đó ở bất kỳ ca nào — `exchange_io.rs` chỉ `matches!` trên
`GlossaryError` thô. Thêm hai ca đẩy `glossary_export_tier`/`glossary_open_import_preview`
qua lỗi I/O thật (thư mục cha không tồn tại; đường dẫn là một thư mục), khẳng định
`message_key()` VÀ `params()["path"]`.

**P8.** `write_export_file` dùng `path.file_name().unwrap_or_default()` — một đường dẫn
KHÔNG có thành phần tên tệp (gốc `/`) biến thành một tệp tạm TRẦN `.tmp` ở thư mục cha, một
tệp KHÁC thứ được yêu cầu, không nói ra. Sửa: từ chối tường minh (`ExportWriteFailed`) khi
`path.file_name()` là `None`, TRƯỚC khi chạm đĩa.

**P9.** `openGlossaryImportPreviewOverlay` thiếu cửa chống bấm chồng mà
`exportGlossaryManageTier` đã có (`if (exportBusy.value) return`) — bấm nhanh hai lần "Nhập
CSV" xếp chồng hai hộp thoại hệ điều hành. Thêm `opening` ref + cửa đối xứng, và khoá nút
"Nhập CSV" (`:disabled="importOpening"`) trong lúc hộp thoại đang mở.

**Ba đối chứng GỠ CHỖ NỐI bắt buộc (P1 · P2 · P5), mỗi ca gỡ rồi khôi phục, chạy trên cây
thật:**

| Gỡ chỗ nối | Kết quả |
|---|---|
| P1 — trả `wire::glossary_export_tier`/`glossary_open_import_preview` về khoá `OpenWorkState` MỘT LẦN, giữ `guard` xuyên qua dialog (khôi phục nguyên bản trước vá) | ĐỎ — ca CẤU TRÚC MỚI `config_invariants.rs::the_open_work_mutex_guard_in_the_dialog_wires_is_acquired_after_the_blocking_call_not_before` báo đúng: `.lock()` (offset 216) đứng TRƯỚC `.blocking_save_file()` (offset 809). Khôi phục ⇒ xanh. |
| P2 — gộp `'ipc_unavailable'` vào cùng nhánh sớm với `'cancelled'` trong `openGlossaryImportPreviewOverlay` | ĐỎ — đúng **1 ca**: `🔴 P2 (vòng rà ba lớp 2026-08-25) — không có cầu IPC MỞ lớp phủ và NÓI RA, KHÁC hẳn huỷ` (`tests/frontend/glossaryImportPreview.test.ts`), báo `importOverlayIsOpen` là `false` thay vì `true`. Khôi phục ⇒ xanh, 11/11. |
| P5 — trả `known_conflict_terms` về gom MỌI hàng của lô (bỏ `.filter(RowPlanKind::Conflict)`) | ĐỎ — đúng **1 ca**: `confirming_with_a_decision_pointing_at_a_new_row_instead_of_a_conflict_is_rejected` (`glossary_import_dialog_contract.rs`), giao dịch ghi thành công (`ImportSummaryWire { inserted: 1, .. }`) thay vì bị từ chối. Khôi phục ⇒ xanh. |

**Số đo sau vá:** `cargo test --locked` (`src-tauri/`) — 0 failed (thêm 5 ca `config_invariants.rs`
lên 20, thêm 12 ca `glossary_import_dialog_contract.rs` lên 21, thêm 1 ca
`exchange_io::tests` lên 7). `npm run test` (vitest) 464/464 (458 trước vá + 6 ca mới: P2×2,
P3×1(gộp trong 3 ca export mới)/P9×1/mount không đổi — xem `tests/frontend/glossaryManage.test.ts`
§"Xuất CSV — P2/P3" và `glossaryImportPreview.test.ts`). `npm run check:i18n` 380 khoá (từ
379, thêm `glossary.manage.export_ipc_unavailable`). `npm run check:commands` 79 command
không đổi (P9 không thêm command mới — `opening` là state nội bộ, không phải một
`dispatch()` mới). `npm run build` sạch. `.githooks/pre-push` **mười một** cổng → vitest → build → cargo test, xanh trong 110s. ⚠️ *(Con số "mười bốn" bị viết lại lần thứ hai ở bản ghi đầu của mục này và sửa lần thứ hai ở lượt đối chứng — nó đếm ba bước `test`/`build`/`cargo test` thành cổng. Hook tự khai **mười một** ngay dòng đầu output của nó; đừng chép lại con số kia.)*

## Verification

**Commands:**
- ✅ **BASELINE NFR6 ĐÃ ĐO, đừng dựng lại** -- `npm run build` rồi `cargo build --release --locked` ở `ce5d2760c23444d3aaa8547919fc91015fc658bc`, cây sạch phía mã: `src-tauri/target/release/auratranslate` = **7.555.496 byte** (2026-08-25 10:09, macOS, `rustc 1.97.1 (8bab26f4f 2026-07-14)` · `cargo 1.97.1`). ⚠️ Nhị phân cũ nằm sẵn ở đó trước lượt này là **7.420.072** byte, dựng 21-08 — lệch **135.424** byte so với baseline thật, tức 13% ngưỡng 1 MB của AD-48 tiêu vào một cây khác. 🔴 **Lượt đo thứ hai phải dùng ĐÚNG `dist/` này** — đừng chạy lại `npm run build` giữa hai lượt.
- `cargo test --locked` (trong `src-tauri/`) -- 0 failed; đọc **mã thoát thật** (`$?` ngay sau lệnh), không suy từ `tail`.
- `npm run build` **trước** `cargo test` -- thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch, không ở một assert.
- `npm run check:deps` -- `BANNED_CRATES` bốn tên, 0 miễn trừ mới.
- `npm run check:commands` · `check:i18n` · `npm run test` (vitest) -- xanh, 0 miễn trừ mới.
- `.githooks/pre-push` -- mười một cổng → vitest → build → cargo test; exit 0.
- `npm run check:scope` và `check:scope:bundled` -- chạy tay, cần cổng 1420 trống (trượt khi đang mở `npm run tauri dev`).

**Manual checks:**
- 🔴 Đối chứng **GỠ chỗ nối**, mỗi lượt khôi phục lại, và ghi số ca vào §Spec Change Log: ① gỡ `.plugin(tauri_plugin_dialog::init())` ⇒ ca canh chỗ đăng ký phải ĐỎ; ② bỏ bước kiểm trần bằng `metadata` ⇒ ca *"tệp quá lớn"* phải ĐỎ; ③ bỏ bước dọn `.tmp` ở nhánh lỗi ⇒ ca *"không để lại tệp tạm"* phải ĐỎ; ④ cho nhịp hai nhận kế hoạch từ webview thay vì từ `State` ⇒ ca *"quyết định trỏ thuật ngữ lạ"* phải ĐỎ. Một bộ test xanh **không** chứng minh chỗ nối mới được canh — Epic 3 đã dính năm lần trong bảy ngày.
- Mở hộp thoại **thật** trên cửa sổ thật (`npm run tauri dev`): xuất → chọn đích → tệp có ở đó và mở được bằng bảng tính; huỷ → không tệp nào sinh ra, không lỗi nào hiện; nhập → lọc đúng `.csv`/`.tsv`. `cargo test` không dựng được cửa sổ nên không nói gì về vế này.
- Đọc lượt **CI cả hai nền tảng** trước khi kết luận xanh — `pre-push` chạy trên macOS của Ice và không nói gì về nửa Windows, nơi hộp thoại đi qua một cài đặt hệ điều hành khác hẳn.

## Suggested Review Order

**Hộp thoại gọi TỪ RUST — chỗ AD-48 sống hay chết**

- Điểm vào: kiểm đủ ba điều kiện TRƯỚC khi mở hộp thoại, khoá lại MỚI sau khi nó đóng.
  [`glossary.rs:1230`](../../src-tauri/src/commands/glossary.rs#L1230)

- Vì sao có helper này: một lượt khoá NGẮN, không giữ `MutexGuard` qua lượt chờ người dùng.
  [`glossary.rs:1206`](../../src-tauri/src/commands/glossary.rs#L1206)

- Đăng ký plugin là bắt buộc — `DialogExt::dialog()` gọi `state()`, thiếu nó là `abort`.
  [`lib.rs:308`](../../src-tauri/src/lib.rs#L308)

**Lượt nhập HAI NHỊP — nội dung tệp không rời Rust**

- Kế hoạch đã phân tích ở lại đây giữa hai nhịp; webview chỉ gửi lại quyết định.
  [`glossary.rs:639`](../../src-tauri/src/commands/glossary.rs#L639)

- Mô hình đã kiểm đi ra dây: số liệu và hai bản dịch, không một byte thô nào.
  [`glossary.rs:714`](../../src-tauri/src/commands/glossary.rs#L714)

- Quyết định chỉ khớp hàng BẤT ĐỒNG — khớp hàng khác là im lặng vô tác dụng.
  [`glossary.rs:863`](../../src-tauri/src/commands/glossary.rs#L863)

**I/O tệp — module mới giữ `exchange.rs` ở lại thuần**

- Trần kích thước kiểm bằng `metadata` trước khi chạm một byte nào.
  [`exchange_io.rs:41`](../../src-tauri/src/core/glossary/exchange_io.rs#L41)

- Ghi nguyên tử ra một đường dẫn NGƯỜI DÙNG chọn — kho chưa có tiền lệ này.
  [`exchange_io.rs:74`](../../src-tauri/src/core/glossary/exchange_io.rs#L74)

- Nháy kép chỉ mở ô bọc khi đứng đầu ô — nay đúng ở CẢ hai bước cắt.
  [`exchange.rs:393`](../../src-tauri/src/core/glossary/exchange.rs#L393)

**Cổng — hai mệnh đề khác nhau, đừng đọc cái này thay cái kia**

- Sáu tên xuống bốn, và chú thích nay nói đúng thứ nó canh: mã trong nhị phân.
  [`check-deps.mjs:172`](../../scripts/check-deps.mjs#L172)

**Frontend — huỷ và "không có cầu IPC" phải phân biệt được**

- Liên hợp có nhãn `outcome`: gộp hai ca này lại là dựng một đường rỗng im lặng.
  [`glossary.ts:729`](../../src/config/glossary.ts#L729)

- Vị từ lý-do-rỗng bốn nhánh, mỗi nhánh tới được — khuôn `manageEmptyReasonFor`.
  [`glossaryImportState.ts:85`](../../src/glossaryImportState.ts#L85)

- Huỷ xuất xoá đường dẫn cũ; giữ lại là để một câu cũ nói dối về lượt vừa rồi.
  [`glossaryManageState.ts:117`](../../src/glossaryManageState.ts#L117)

- Quyết định từng hàng là `radiogroup`, không `dispatch` — bàn phím có sẵn ngữ nghĩa.
  [`GlossaryImportOverlay.vue:193`](../../src/GlossaryImportOverlay.vue#L193)

**Phép kiểm — bốn ca mang nhiều sức nặng nhất**

- Chỗ đăng ký plugin và lệnh cấm `fs::init()` nay là mệnh đề kiểm được, không lời dặn.
  [`config_invariants.rs:723`](../../src-tauri/tests/config_invariants.rs#L723)

- Ca cấu trúc: khoá `OpenWorkState` phải lấy SAU lượt gọi chặn, không trước.
  [`config_invariants.rs:783`](../../src-tauri/tests/config_invariants.rs#L783)

- Giải mã chuỗi dây thật của `ConflictDecision` — gõ sai `rename` nay đỏ được.
  [`glossary_import_dialog_contract.rs:552`](../../src-tauri/tests/glossary_import_dialog_contract.rs#L552)
