---
title: 'Cụm C — mất cập nhật im lặng ở nhịp hai của lượt nhập Glossary'
type: 'bugfix'
created: '2026-08-25'
status: 'done'
review_loop_iteration: 0
baseline_commit: 'db3833b36bf7b96c9c4de762f2b7486a23fce085'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/_bmad-output/implementation-artifacts/3-10-xuat-va-nhap-glossary-qua-csv-tsv.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Cụm C của vòng rà Epic 3 mang sáu phát hiện về đồng thời; Ice chốt `[S]` — **C4** (bốn hàm đọc hai tầng qua hai kết nối) đã ghi thành nợ có chủ vì nó đòi một `AD` mới, còn lượt này đóng **C1 · C2 · C3 · C5**. Đo lại từng mục trên `db3833b`: **C1 và C3 còn đúng**, **C2 bị bác bằng ba phép đo** (xem §Design Notes), **C5 là một lưới còn thiếu chứ không phải một lỗi đang sống** (0 chỗ thả vé trong toàn kho). Nặng nhất là C1: người dùng đối chiếu *của tôi* ↔ *của tệp* ở nhịp preview rồi chọn "lấy bản của tệp", nhưng câu `UPDATE … SET translation WHERE id` ở nhịp hai **không so lại** giá trị họ vừa nhìn — một lượt ghi chen vào giữa hai nhịp bị đè mất, không lỗi, không rollback, đúng lớp "rỗng im lặng" mà `AGENTS.md` gọi là lỗi trung tâm của dự án.

**Approach:** Cấp một phép **so lạc quan** cho nhánh `TakeTheirs` — giá trị người dùng đã thấy trở thành một phần điều kiện ghi — và cấp **tên riêng** cho lớp lỗi đó (một biến thể `GlossaryError` mới, đi trọn bộ bốn mắt). Dời luật C3 từ `commands/glossary.rs` xuống lõi theo AD-1. Đánh dấu vé ghi bằng `#[must_use]`. Không đổi chữ ký công khai nào, không thêm phụ thuộc, không bước di trú.

## Boundaries & Constraints

**Always:**
- 🔴 **Mỗi mục vá kèm một ca test mà GỠ bản vá ra thì ca đó ĐỎ**, và ghi lại **số ca đỏ thật** cùng tên ca. Bộ test cũ vẫn xanh **không đủ** — đó đúng là lớp lỗi Epic 3 dính năm lần trong bảy ngày.
- 🔴 **Biến thể `GlossaryError` mới đi TRỌN bộ bốn trong CÙNG lượt**: biến thể (`store.rs`) ⇒ nhánh `impl From<GlossaryError> for IpcError` (`store.rs:665` là khuôn) ⇒ khoá trong `message_keys!` (`core/i18n/mod.rs:378` là khuôn) ⇒ câu trong `src/i18n/vi.json`. Thiếu một mắt là một cổng đỏ — `ipc_contract.rs` và `check:i18n` canh; **đừng nới cổng**.
- 🔴 **`params` mang DỮ LIỆU, không mang CÂU** (AD-21). Chuỗi literal trong `src-tauri/src/**` viết **KHÔNG DẤU**; doc-comment giữ dấu.
- 🔴 **`import_into_tier` giữ nguyên hợp đồng "0 lượt ghi khi có lỗi"** và giữ nguyên khả năng gom nhiều va chạm `UNIQUE` một lượt mà ca `…two_errors…` đang khoá. Phép kiểm C3 chạy **TRƯỚC** khi mở giao dịch.
- 🔴 **So `translation` phải NULL-an-toàn.** `WHERE translation = ?` **không bao giờ** khớp khi giá trị là `NULL` — một mục chờ chốt (`translation IS NULL`) sẽ biến mọi lượt `TakeTheirs` hợp lệ thành lỗi giả. Dùng toán tử so NULL-an-toàn của SQLite.
- 🔴 **Không thêm phụ thuộc nào** (NFR15).
- **C3 dời luật xuống lõi, không nhân đôi nó.** Nếu gỡ phép kiểm ở `commands/glossary.rs:863-873` làm hai ca hiện có đỏ, **đo trước rồi mới quyết** — giữ lại một vỏ mỏng chuyển tiếp là hợp lệ, giữ lại hai bản sao cùng một luật thì không.
- **`#[must_use]` đặt trên KIỂU, không trên hàm** — `Result` đã `#[must_use]`, nên đánh dấu `Store::write_ticket` là dư; ca thật sự lọt là `store.write_ticket(job)?;` viết thành một câu lệnh trần.

**Ask First:**
- Nếu bản vá C1 làm **bất kỳ** ca nào trong sáu ca `TakeTheirs` hiện có đỏ: **DỪNG và trình lỗi**. Đỏ ở đó nghĩa là một ca đã chốt cứng hành vi đè-vô-điều-kiện như một hành vi mong muốn — đó là câu hỏi cho Ice, không phải một ca cần sửa.
- Nếu để C5 **đỏ được** hoá ra đòi nâng `unused_must_use` thành `deny` ở tầng crate: đó là một quyết định phạm vi. Trình **số cảnh báo hiện có** khi chạy với `-D warnings` cho Ice chốt, đừng tự nâng.
- Nếu cả danh sách va `UNIQUE` **và** danh sách so lạc quan cùng không rỗng trong một lô, mà thứ tự báo không suy ra được từ mã hiện có.

**Never:**
- Không đụng **C4** (`store.rs:715 · 787 · 922 · 1299`) — đã có mục nợ riêng, chủ riêng, và nó đòi một `AD`.
- Không đụng 39 phát hiện của cụm D, E, F.
- Không đổi chữ ký công khai của `import_into_tier` / `classify` / `RowPlan` / `RowPlanKind` — `commands/glossary.rs` không được phải sửa một dòng nào để biên dịch.
- Không sửa `GLOSSARY_ENTRY_DDL`, không bước di trú, không đụng trigger AD-36.
- Không hạ ngưỡng, không `#[allow]`, không chuyển một ca sang danh sách loại trừ để cổng hết đỏ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| **①** `TakeTheirs`, giá trị đã đổi giữa hai nhịp | preview thấy `existing_translation = "Tiêu Viêm"`; trước khi xác nhận, một lượt ghi khác đổi hàng đó thành `"Tiêu Viễn"` | Lỗi **có tên riêng** mang `source_term` | **0 lượt ghi**, cả lô rollback. **Không** báo `Ok` |
| **②** `TakeTheirs`, không ai chen vào (hồi quy) | preview thấy `"Tiêu Viêm"`, vẫn `"Tiêu Viêm"` lúc xác nhận | Ghi đè bình thường, `summary.updated` tăng | N/A |
| **③** `TakeTheirs` trên một mục **chờ chốt** | `existing_translation = None` và vẫn `None` lúc xác nhận | Ghi bình thường — `NULL` so `NULL` phải **khớp** | N/A |
| **③b** mục chờ chốt vừa được ai đó chốt | `existing_translation = None`, nay là `Some("…")` | Cùng lỗi ① | 0 lượt ghi |
| **④** hàng biến mất giữa hai nhịp | `existing_id` đã bị xoá | Giữ **nguyên** hành vi hiện có (`row_missing_error` ⇒ `store.write_failed`) — không đổi nhãn, không gộp vào ① | 0 lượt ghi |
| **⑤** quyết định trỏ thuật ngữ lạ, gọi **thẳng** hàm lõi | `import_into_tier(…, decisions = {"khong-co-trong-lo": TakeTheirs})` | `ImportDecisionUnknownTerm` mang thuật ngữ đó | **0 lượt ghi** — kiểm trước khi mở giao dịch |
| **⑥** quyết định trỏ một hàng `New` (không phải `Conflict`), gọi thẳng lõi | `decisions` mang khoá của một hàng `RowPlanKind::New` | Cùng lỗi ⑤ | 0 lượt ghi |
| **⑦** lô có **cả** va `UNIQUE` **và** một hàng ①  | một hàng `New` va `UNIQUE` + một hàng `TakeTheirs` đã cũ | Thứ tự báo **tất định**, và viết ra bằng chữ tại chỗ | 0 lượt ghi |
| **⑧** vé ghi bị thả không `.wait()` | `store.write_ticket(job)?;` viết thành câu lệnh trần | Cảnh báo lúc biên dịch nêu đúng vé | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/glossary/store.rs` (1611 dòng) — **trọng tâm**, cả ba mục C1/C2/C3 nằm ở đây:
  - `:1512-1611` `import_into_tier` — hàm lõi hai nhịp. `:1524` `plans.to_vec()` · `:1563-1566` `decisions_owned.get(…).unwrap_or(KeepMine)` ⇒ **lỗ C3** · `:1571` câu `UPDATE glossary_entry SET translation = ?1 WHERE id = ?2` ⇒ **lỗ C1** · `:1583-1585` `if changed == 0 { row_missing_error }` — hành vi ④, **giữ nguyên**, nhưng sau bản vá `changed == 0` mang **hai** nghĩa nên phải phân biệt được · `:1594` `unique_conflict_marker_error()` và `:1599-1607` khối `match result` — **khuôn có sẵn** để mang một danh sách ra khỏi closure qua `Arc<Mutex<Vec<String>>>`; mục C1 dùng lại đúng khuôn này, không phát minh khuôn mới.
  - `:1521` `work.ok_or(GlossaryError::WorkTierUnavailable)?` — **tiền lệ ngay trong chính hàm này**: một tham số được kiểm tường minh và trả lỗi riêng. C3 là cùng hình dạng cho tham số `decisions`.
  - `:390-397` `row_missing_error` — mượn `FromSqlConversionFailure` để chở một câu chẩn đoán; doc-comment giải thích vì sao "0 hàng đổi" không phải một lỗi SQL.
  - `:411-…` `enum GlossaryError` — biến thể mới vào đây. `:514-517` `ImportDecisionUnknownTerm { term }` là khuôn gần nhất.
  - `:665-674` nhánh `impl From<GlossaryError> for IpcError` cho `ImportDecisionUnknownTerm` — khuôn cho nhánh mới. `:1599-1607` cho thấy `ImportUniqueConflict { source_terms }` gom **danh sách**.
  - `:249` và `:880` — hai câu `UPDATE` còn lại của bảng. **Chỉ đọc**, là bằng chứng cho §Design Notes (không câu nào chạm `source_term`).
- `src-tauri/src/core/glossary/exchange.rs:1054-1059` — `RowPlanKind::Conflict { existing_id: i64, existing_translation: Option<String> }`. `:1099-1109` `classify()` dựng nó. ⇒ **giá trị người dùng đã thấy có sẵn nguyên vẹn** ở nhịp hai; không cần đọc lại kho, không cần đổi kiểu.
- `src-tauri/src/commands/glossary.rs` — lớp adapter:
  - `:639-646` `PendingImport` giữ `plans` giữa hai nhịp · `:773` gán · `:875` chỗ gọi sản phẩm **duy nhất** của `import_into_tier`.
  - `:863-873` phép kiểm `known_conflict_terms` ⇒ **đây là luật nghiệp vụ đặt sai tầng** (C3).
  - `:701-707` `ImportPreviewConflictWire { source_term, existing_translation, file_translation }` — **chỉ đọc**, bằng chứng người dùng ĐÃ được cho xem cả hai vế.
- `src-tauri/src/commands/mod.rs:2` — *"Adapter thuần, KHÔNG chứa quy tắc nghiệp vụ (AD-1)."* · `:22-33` khuôn "hàm thuần trước, `#[tauri::command]` là vỏ". **Đây là mệnh đề đã ký làm C3 thành một lỗi, không phải một sở thích.**
- `src-tauri/src/core/i18n/mod.rs:370-381` — khối khoá Glossary, `:378` `GlossaryImportDecisionUnknownTerm => "…" ["value"]` là khuôn. Khoá mới vào cạnh đó.
- `src/i18n/vi.json:25-42` — mười tám câu lỗi Glossary; câu mới đi cùng khuôn kết *"— chưa có gì được ghi."* (`:41`) hoặc *"Nạp lại rồi thử lại."* (`:38`).
- `src-tauri/src/core/store/writer.rs:48-63` `struct WriteTicket<T>` + `:53-64` `wait()` — **mục C5**. `:212` chỗ sinh vé.
- `src-tauri/src/core/store/mod.rs:654-663` `Store::write_ticket` — trả `Result<WriteTicket<T>, StoreError>`.
- `src-tauri/src/core/glossary/candidate_store.rs:168-179` `ImportScanWriteTicket` + `wait()` — vỏ bọc miền Glossary, **cũng thiếu dấu**, cùng lớp lỗi.
- `src-tauri/src/core/segment/paragraph.rs:78,97,115,127` và `regroup.rs:99,143,162,239` — **tám** tiền lệ `#[must_use]` trong kho, **tất cả trên hàm, không có tiền lệ nào trên kiểu, và không tiền lệ nào dùng dạng có chuỗi lý do**. Đọc trước khi chọn hình dạng.
- `src-tauri/src/core/store/schema.rs:302` `id INTEGER PRIMARY KEY AUTOINCREMENT` + `:147-148` doc-comment (AD-3) · `:548-551` bước di trú chép `id` **nguyên vẹn**. **Chỉ đọc** — ba bằng chứng bác C2.
- `src-tauri/tests/glossary_exchange_contract.rs` — **13** lời gọi thẳng `import_into_tier`, bỏ qua lớp `commands`. Sáu ca chạm `TakeTheirs`: `:853` `…keeps_the_existing_translation_by_default_on_conflict` · `:875` `…take_theirs_updates_the_existing_row` · `:904` `…only_translation_and_never_touches_note_or_category` · `:948` `…ignores_a_note_and_category_the_file_actually_provides` · `:1000` `…regressing_a_confirmed_row_to_pending_is_refused…` · `:1124` `…reports_the_trigger_as_the_real_cause`. **Ca mới của C1/C3 vào đây** — đây là tệp duy nhất chạm được lớp lõi.
- `src-tauri/tests/glossary_import_dialog_contract.rs:180` và `:505` — **hai** ca duy nhất chạm `ImportDecisionUnknownTerm`, **cả hai đi qua `commands::glossary`**. Chúng phải **vẫn xanh** sau khi C3 dời luật.
- `src-tauri/tests/config_invariants.rs:848-960` `the_blocking_wires_run_off_the_main_thread` — tiền lệ **cổng quét văn bản nguồn** (đọc `fs::read_to_string`, định vị bằng chuỗi mốc, `panic!` không dấu). Khuôn dự phòng cho C5 nếu không nâng lint.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/glossary/store.rs` — thêm **một** biến thể `GlossaryError` cho lớp lỗi "giá trị đã đổi dưới chân người dùng giữa hai nhịp" kèm nhánh `Display` và nhánh `From<GlossaryError> for IpcError` — lớp lỗi này hôm nay **không có tên**, nên nó đang rơi vào `store.write_failed` chung và người dùng đọc một câu không nói được điều gì đã xảy ra.
- [x] `src-tauri/src/core/glossary/store.rs` — vá **C1** ở nhánh `TakeTheirs`: đưa `existing_translation` (giá trị người dùng đã nhìn) vào **điều kiện ghi** bằng phép so NULL-an-toàn, dùng lại khuôn kênh `Arc<Mutex<…>>` + lỗi đánh dấu mà `:1594` đã dựng; và giữ **phân biệt được** ca ④ (hàng biến mất) khỏi ca ① (giá trị đã đổi) — sau bản vá `changed == 0` mang hai nghĩa.
- [x] `src-tauri/src/core/glossary/store.rs` — vá **C3**: kiểm `decisions` ngay đầu `import_into_tier`, **trước** khi mở giao dịch, cùng hình dạng với `:1521`; rồi thu phép kiểm ở `commands/glossary.rs:863-873` về đúng tầng của nó theo AD-1.
- [x] `src-tauri/src/core/store/writer.rs` và `src-tauri/src/core/glossary/candidate_store.rs` — vá **C5**: đánh dấu `WriteTicket<T>` và `ImportScanWriteTicket`; thả một vé nghĩa là thả trọn kết quả commit/rollback và mọi `StoreError` đi cùng nó.
- [x] `src-tauri/src/core/i18n/mod.rs` và `src/i18n/vi.json` — khoá + câu cho biến thể mới, cạnh khối khoá Glossary hiện có.
- [x] `src-tauri/tests/glossary_exchange_contract.rs` — ca cho ①②③③b④⑤⑥⑦, mỗi ca kèm phép đối chứng gỡ-chỗ-nối **đã chạy thật**. Ca ①/③b phải mô phỏng một lượt ghi **chen vào giữa** `classify()` và `import_into_tier()`.
- [x] `src-tauri/tests/config_invariants.rs` — cổng quét văn bản nguồn canh CẢ BA vế của bản vá C5 (hai `#[must_use]` + dòng `deny` trong `Cargo.toml`) — thêm sau khi đo được rằng lint một mình không làm bản vá đỏ được; xem §Spec Change Log mục cuối.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — đóng mục **Cụm C** bằng chữ: `→ 🟡` kèm phần còn hở (C4 đã có mục riêng), và ghi rõ **C2 đóng bằng `→ KHÔNG LÀM` kèm ba phép đo** đã bác nó. Không xoá mục cũ.

**Acceptance Criteria:**
- Given mỗi mục vá C1/C3/C5, when gỡ riêng bản vá đó ra và chạy bộ test, then **có ít nhất một ca đỏ trỏ đúng mục đó**; tên ca và số ca đỏ được ghi lại thành số thật, không suy.
- Given sáu ca `TakeTheirs` hiện có, when chạy sau bản vá, then **cả sáu vẫn xanh** — bản vá thu hẹp điều kiện ghi, không đổi hành vi của lượt nhập không có ai chen vào.
- Given hai ca `ImportDecisionUnknownTerm` hiện có (đi qua `commands::glossary`), when chạy sau khi C3 dời luật xuống lõi, then **cả hai vẫn xanh** và **vẫn nhận đúng biến thể lỗi cũ** — người dùng không thấy một câu lỗi khác đi.
- Given biến thể lỗi mới, when xoá khoá của nó khỏi `message_keys!`, then một cổng **đỏ**; khôi phục rồi xoá câu tương ứng khỏi `vi.json`, then `ipc_contract.rs` hoặc `check:i18n` **đỏ**.
- Given hợp đồng công khai, when so chữ ký `import_into_tier`/`classify`/`RowPlan` trước và sau, then **không đổi**.
- Given toàn bộ bản vá, when chạy `.githooks/pre-push`, then exit 0 — và số ca `cargo test --locked` **tăng**, với con số trước/sau ghi lại thật.

## Spec Change Log

### 2026-08-25 — vá C1/C3/C5, C2 đóng bằng KHÔNG LÀM, ba đối chứng gỡ chỗ nối chạy thật

**C1 — so lạc quan, NULL-an-toàn.** `import_into_tier` nay chạy `UPDATE glossary_entry SET
translation = ?1 WHERE id = ?2 AND translation IS ?3`, `?3` là `existing_translation` mà
`classify()` chụp ở nhịp preview. `changed == 0` tách hai nghĩa: hàng biến mất (④, hành vi
GIỮ NGUYÊN — `row_missing_error`) và giá trị đã đổi (①/③b, `GlossaryError::ImportStaleConflict`
mới, đi trọn bộ bốn mắt: biến thể · `From<…> for IpcError` · `message_keys!` ·
`err.glossary.import_stale_conflict` trong `vi.json`). Ca ⑦ (lô có cả va `UNIQUE` lẫn va lạc
quan): thứ tự báo tất định — `ImportUniqueConflict` được ưu tiên vì khối kiểm của nó đã đứng
trước trong mã (P6), không phải một lựa chọn mới; viết vào doc-comment tại chỗ.

**C2 — `→ KHÔNG LÀM`, ba phép đo bác cơ chế đã nêu** (xem §Design Notes gốc + mục đóng trong
`deferred-work.md`). Không sửa mã.

**C3 — dời xuống lõi.** Phép kiểm "khoá của `decisions` phải khớp một `source_term` mang
`RowPlanKind::Conflict`" dời từ `commands/glossary.rs:863-873` xuống đầu `import_into_tier`,
trước khi mở giao dịch. `commands/glossary.rs::glossary_confirm_import` không còn tự kiểm.

**C5 — `#[must_use]` trên KIỂU.** `WriteTicket<T>` (`core/store/writer.rs`) và
`ImportScanWriteTicket` (`core/glossary/candidate_store.rs`).

**Ba đối chứng GỠ CHỖ NỐI, chạy thật, khôi phục sau mỗi lượt:**

| Gỡ chỗ nối | Kết quả |
|---|---|
| C1 — trả `UPDATE` về `WHERE id = ?2` trần (bỏ `AND translation IS ?3`) | ĐỎ — đúng **2 ca**: `take_theirs_is_refused_when_the_translation_changed_under_the_users_feet_between_preview_and_confirm` và `take_theirs_is_refused_when_a_pending_row_was_confirmed_by_someone_else_between_preview_and_confirm` (`glossary_exchange_contract.rs`). Khôi phục ⇒ xanh. |
| C3 — xoá khối kiểm `known_conflict_terms`/`ImportDecisionUnknownTerm` khỏi đầu `import_into_tier`, không thêm gì thay thế | ĐỎ — đúng **4 ca**: hai ca MỚI gọi thẳng lõi (`import_into_tier_rejects_a_decision_pointing_at_a_term_absent_from_the_batch_entirely`, `import_into_tier_rejects_a_decision_pointing_at_a_new_row_instead_of_a_conflict`) VÀ hai ca CŨ đi qua `commands::glossary` (`confirming_with_a_decision_pointing_at_an_unknown_term_fails_and_keeps_the_batch`, `confirming_with_a_decision_pointing_at_a_new_row_instead_of_a_conflict_is_rejected`) — chứng minh luật chỉ còn sống ở MỘT tầng. Khôi phục ⇒ cả bốn xanh. |
| C5 — gỡ `#[must_use]` khỏi `WriteTicket<T>`, giữ nguyên một câu dò thả vé trần (`self.write_ticket(\|_\| Ok(())).unwrap();`) | Có thuộc tính ⇒ `cargo build` in đúng 1 cảnh báo `unused `WriteTicket` that must be used` trỏ đúng dòng; gỡ thuộc tính ⇒ **0 cảnh báo** cho cùng dòng mã. Câu dò chỉ tồn tại trong lượt đối chứng, không nằm lại trong cây nguồn. Khôi phục ⇒ build sạch. |

**Đối chứng chiều ngược cho C1 (§Verification):** sáu ca `TakeTheirs` hiện có
(`glossary_exchange_contract.rs:853,875,904,948,1000,1124` theo Code Map) chạy lại SAU bản vá —
cả sáu vẫn xanh, không ca nào đỏ oan.

**Cổng biên dịch/`ipc_contract` cho biến thể lỗi mới:** xoá `GlossaryImportStaleConflict` khỏi
`message_keys!` ⇒ **lỗi biên dịch** ngay (`store.rs` gọi thẳng biến thể đó) — mạnh hơn một ca
test đỏ. Khôi phục, rồi xoá câu `err.glossary.import_stale_conflict` khỏi `vi.json` ⇒
`cargo test --test ipc_contract` **đỏ đúng 2 ca**
(`every_message_key_exists_in_vi_json`, `every_message_key_declares_the_params_its_string_needs`).
Khôi phục ⇒ xanh.

**Số đo trước/sau (đo TRÊN CÙNG máy, cùng lượt, bằng `git stash` để tách hai trạng thái):**
- `cargo test --locked` (tổng số ca PASS cộng dồn mọi test binary): **695 → 702** (+7 ca mới:
  ①③③b④⑤⑥⑦; ② đã có sẵn hồi quy). `glossary_exchange_contract.rs`: 54 → 61. 0 failed cả hai
  lượt. ⚠️ Con số 695 là đo TRỰC TIẾP trên `db3833b` + cụm B tại thời điểm vá này. 🔵 **SỬA 2026-08-25 — mệnh đề gán sai người ở bản trước.** Con số
  691 ở §Verification KHÔNG phải của Ice: spec này do agent soạn ở bước lập kế hoạch, và
  691 là một con số chép từ §Spec Change Log của cụm B mà **không ai đếm lại**. Ice không
  viết nó và không xác nhận nó; Ice chỉ duyệt spec. §Verification giữ nguyên không sửa.
  Truy nguồn chênh lệch, đo được: `#[test]` ở `db3833b` = **700**, ở cây làm việc = **707**
  (+7 đúng bằng bảy ca mới); trừ **5** ca `ignored` ⇒ gốc thật **695**. Cụm B ghi 691 ở đúng
  commit mang tên nó, tức con số ấy đã cũ so với chính commit đó. Bài học đứng nguyên, và
  nó vừa tự minh hoạ: đếm, hoặc đừng viết con số.
- Sau khi C5 nâng `unused_must_use = "deny"` (xem mục riêng bên dưới): **703** (702 + 1
  doctest `compile_fail` mới), 0 failed.
- `npm run check:i18n`: khoá `vi.json` **382 → 383** (đúng +1), Kiểm D bắt được một lượt câu
  đầu tiên dùng "bạn" (vi phạm UX-DR47) — sửa lại vô nhân xưng trước khi đóng.
- `RUSTFLAGS="-D warnings" cargo check --locked`: **exit 0 — 0 cảnh báo** trong toàn crate hôm
  nay (đo cho §Ask First của C5, KHÔNG dùng để tự nâng `unused_must_use` thành `deny`).
- `.githooks/pre-push`: mười một cổng → vitest → build → cargo test, xanh trong **249s**.

**Ghi chú dọn dẹp:** hai lượt gỡ-chỗ-nối thủ công (C1, C3) sửa trực tiếp `store.rs` rồi khôi
phục bằng bản sao lưu byte-for-byte trước khi chạy `cargo build` xác nhận sạch; C5 dùng một
hàm dò tạm ở `core/store/mod.rs`, không commit.

### 2026-08-25 (muộn hơn) — Ice chốt §Ask First của C5: nâng `unused_must_use` thành `deny`

Bước rà đối kháng của workflow đòi §I/O Matrix ca ⑧ phải có ít nhất một ca test đã CHẠY và
ĐẠT — bản vá trước đóng ca ⑧ bằng một lượt bấm tay (`cargo build` + đọc cảnh báo), bước rà
tính đó là **thiếu**. Ice chốt nâng đúng MỘT lint, `unused_must_use = "deny"`, trong
`src-tauri/Cargo.toml` (`[lints.rust]`), không nâng `-D warnings` toàn bộ và không thêm lint
nào khác — phép đo cho phép nâng: `RUSTFLAGS="-D warnings" cargo check --locked` → exit 0, 0
cảnh báo trong toàn crate 2026-08-25.

**Giới hạn kỹ thuật đo được khi dựng doctest, viết ra thay vì lách qua:** `WriteTicket` và
mọi hàm sinh ra nó (`Writer::enqueue`/`Store::write_ticket`) là `pub(crate)` CÓ CHỦ (chính
doc-comment của `WriteTicket` khoá: *"Không mở sender, connection hay transaction ra ngoài
`core/store/**`"*). Một doctest LUÔN biên dịch như một crate NGOÀI — ba thực nghiệm độc lập
xác nhận không đường nào gọi được kiểu thật từ một doctest: gọi thẳng tên ⇒ `E0425`; nhập qua
đường dẫn đủ (`auratranslate_lib::core::store::writer::WriteTicket`) ⇒ `E0603 module is
private`; một hàm bọc đánh dấu `#[cfg(doctest)]` cũng KHÔNG lọt vào rlib mà doctest liên kết
(`cfg(doctest)` chỉ áp cho đoạn doctest, không áp cho crate thư viện nó liên kết tới) ⇒ vẫn
`E0603`. Mở tầm nhìn `WriteTicket` ra `pub` để một doctest gọi được là ĐỔI một bất biến kiến
trúc — việc của một `AD` mới do Ice ký, không phải một dòng mã tự quyết ở lượt vá này.

Doctest `compile_fail` được viết TẠI doc-comment của `WriteTicket` để nó nằm đúng chỗ người
đọc gặp trước, nhưng dựng trên một kiểu THAY THẾ CÙNG HÌNH DẠNG (`SameShapeAsWriteTicket`,
`#[must_use]` + `#![deny(unused_must_use)]` khai lại tại chỗ vì `[lints.rust]` của
`Cargo.toml` không lan sang chương trình riêng mà `rustdoc` biên dịch cho một doctest — đo
được: bỏ dòng `#![deny(...)]` thì khối chỉ cảnh báo, không còn lỗi, và `compile_fail` tự đỏ).
Doctest này chứng minh ĐÚNG cơ chế Rust (must_use + deny chặn câu lệnh trần, không chặn `let`
rồi rơi khỏi phạm vi — một khối `ignore` thứ hai minh hoạ đúng giới hạn đó), KHÔNG phải một
ca ràng buộc trực tiếp với kiểu `WriteTicket` thật. Ràng buộc thật với kiểu thật được đo bằng
một hàm dò tạm trên đĩa thật (`core/store/mod.rs`, không commit) — xem ba số đo A/B/C trong
mục Cụm C của `deferred-work.md`.

**Bốn đối chứng gỡ chỗ nối, chạy thật, khôi phục sau mỗi lượt:**

| Gỡ chỗ nối | Kết quả |
|---|---|
| Hàm dò tạm thả vé trần, CÓ `#[must_use]` + CÓ `[lints.rust]` | `error: unused `WriteTicket` that must be used` — biên dịch THẤT BẠI (trước đây chỉ là `warning:`, exit 0). |
| Cùng hàm dò, gỡ `#[must_use]` khỏi `WriteTicket<T>` (giữ `[lints.rust]`) | Biên dịch SẠCH, 0 chẩn đoán — chứng minh riêng `deny` không đủ, phải có `#[must_use]` trên kiểu. |
| Cùng hàm dò, khôi phục `#[must_use]`, gỡ `[lints.rust]` khỏi `Cargo.toml` | Trở lại `warning:`, exit 0 — đúng khoảng hở mà lượt vá này đóng. |
| Doctest: gỡ `#[must_use]` khỏi `SameShapeAsWriteTicket`, rồi (khôi phục) gỡ `#![deny(unused_must_use)]` | Cả hai lượt ⇒ ca `compile_fail` **ĐỎ** (`Test compiled successfully, but it's marked compile_fail`). |

**Số đo sau vá:** `cargo test --locked` (gồm doctest) **703** ca PASS (702 + 1 doctest mới), 0
failed. `.githooks/pre-push` mười một cổng → vitest → build → cargo test, xanh trong **109s**.
`cargo build --locked` mặc định (không `RUSTFLAGS`) vẫn 0 cảnh báo.

### 2026-08-25 (muộn nhất) — lint một mình KHÔNG làm bản vá C5 đỏ được; thêm cổng quét văn bản nguồn

**Phát hiện, đo chứ không suy.** Sau khi nâng `unused_must_use = "deny"`, agent chạy lại hai
phép gỡ chỗ nối trên cây nguồn THẬT, mỗi lượt khôi phục ngay:

| Gỡ chỗ nối | Kết quả |
|---|---|
| Gỡ dòng `unused_must_use = "deny"` khỏi `Cargo.toml` | `cargo test --locked` **703 xanh, 0 đỏ** |
| Gỡ `#[must_use]` khỏi `WriteTicket<T>` thật (giữ lint) | **703 xanh, 0 đỏ**, exit 0 |

⇒ **Cả hai vế của bản vá C5 có thể biến mất mà không một dòng đỏ nào.** Lý do: hôm nay 0 chỗ
trong kho thả một vé, nên `deny` không có chỗ nào để nổ; và doctest `compile_fail` canh kiểu
THẾ THÂN nên nó chỉ đỏ khi ai đó sửa chính kiểu thế thân. Bốn đối chứng ở mục trên đo đúng
**cơ chế Rust** và đo qua một **hàm dò không commit** — chúng không canh chỗ nối tới mã sản
phẩm sau khi hàm dò biến mất. ⇒ C5 **không đạt** luật 🔴 §Boundaries (*"mỗi mục vá kèm một ca
test mà GỠ bản vá ra thì ca đó ĐỎ"*) lẫn AC1, theo cả hai chiều.

🔴 **Đây là đúng lớp lỗi `khoi-phuc-trung-thanh-khong-phai-dung`: đối chứng KẾT QUẢ với ĐIỀU
NÓ KHAI.** Mục C5 KHAI là đã đóng ca ⑧; kết quả đo nói bản vá tự nó không được canh.

**Ice chốt (lượt thứ hai của cùng cửa §Ask First):** giữ nguyên lint — **bổ sung**, không
thay — một cổng quét văn bản nguồn theo khuôn `config_invariants.rs` đã có sẵn.

**Bản vá:** `src-tauri/tests/config_invariants.rs` —
`the_write_tickets_are_must_use_and_the_lint_that_gives_it_teeth_is_denied`. **MỘT ca, BA
vế**: `#[must_use]` trên `WriteTicket<T>`, trên `ImportScanWriteTicket`, và dòng
`unused_must_use = "deny"` trong `Cargo.toml`. Vế thứ ba **bỏ dòng chú thích trước khi so** —
khối chú thích ngay trên mục `[lints.rust]` có nhắc lại chính chuỗi đó, nên một phép
`contains` trần sẽ xanh oan khi dòng thật bị comment ra.

**Ba đối chứng gỡ chỗ nối, chạy thật, khôi phục sau mỗi lượt:**

| Gỡ chỗ nối | Kết quả |
|---|---|
| Gỡ `#[must_use]` khỏi `WriteTicket<T>` | **ĐỎ** — `the_write_tickets_are_must_use_and_the_lint_that_gives_it_teeth_is_denied` |
| Gỡ `#[must_use]` khỏi `ImportScanWriteTicket` | **ĐỎ** — cùng ca |
| Comment dòng `unused_must_use = "deny"` trong `Cargo.toml` | **ĐỎ** — cùng ca (đúng vế mà phép bỏ chú thích tồn tại để bắt) |

Khôi phục trọn ⇒ cổng xanh lại.

⚠️ **Giới hạn có tên, ghi ra thay vì làm tròn lên:** cổng này đọc **văn bản nguồn** — nó
khẳng định thuộc tính và dòng lint CÓ MẶT, nó không chứng minh một vé bị thả sẽ đỏ. Vế đó do
chính trình biên dịch giữ, và chỉ nổ khi có một chỗ thả thật. Nó cũng KHÔNG bắt ca
`let ticket = …?;` rồi để biến rơi khỏi phạm vi — không lint nào trong Rust hôm nay bắt được
ca đó. Cả hai giới hạn đã viết tại chỗ trong doc-comment của cổng.

### 2026-08-25 — vòng rà ba lăng kính: sáu mục vá, một mục ghi nợ, năm mục bác

**Mục vá nặng nhất, và nó kèm một phép CẮT THỬ thật (lăng kính verification-gap).** Mắt nối
`GlossaryError::ImportStaleConflict` ⇒ `MessageKey::GlossaryImportStaleConflict` ở nhánh
`impl From<GlossaryError> for IpcError` **không cổng nào canh**. Đo: đổi khoá đó sang
`GlossaryImportUniqueConflict` (hai nhánh nằm sát nhau, cùng hình dạng — một lượt trượt tay
chép-dán rất dễ xảy ra) ⇒ `glossary_exchange_contract` **61/61** · `glossary_import_dialog_contract`
**21/21** · `ipc_contract` **5/5** · `config_invariants` **22/22** — **XANH TRỌN**. Bảy ca C1 mới
assert trên `GlossaryError` **thô**, không đi qua `IpcError::from`; `ipc_contract.rs` chỉ đối chiếu
danh mục `message_keys!` với `vi.json`, nó không bao giờ dựng một `GlossaryError` rồi hỏi nó phân
giải ra khoá nào. Hậu quả thật cho người dùng: một lượt sửa đồng thời trong lúc xem trước cho họ
đọc **đúng câu sai**.
⇒ Vá: `confirming_take_theirs_after_a_concurrent_write_surfaces_import_stale_conflict_with_the_real_term`
(`glossary_import_dialog_contract.rs`), theo đúng khuôn `…export_write_failed_with_the_real_path_param`
mà chính tệp đó ghi ở `:597` là khuôn chuẩn cho một biến thể mới. **Đối chứng: lặp lại phép cắt thử
⇒ ĐỎ đúng 1 ca (ca mới), ba bộ kia vẫn xanh** — nghĩa là không bề mặt nào khác chạm mắt nối này.

**Rot số đếm — và lượt vá này tự tái sản xuất nó.** Ba chú thích THƯỜNG TRÚ (`store.rs` doc-comment
của `import_into_tier`, `store.rs` chú thích trong thân hàm, `commands/glossary.rs`) đóng đinh
*"13 lời gọi thẳng `import_into_tier`"* làm bằng chứng cho việc lớp adapter bị đi vòng. Con số 13
đúng lúc lập spec, và **chính bảy ca C1 của bản vá này** làm nó sai. Đếm lại 2026-08-25:
`grep -c "import_into_tier(" glossary_exchange_contract.rs` = **21**, trừ **1** dòng chú thích
(`:1349`) ⇒ **20**. 🔴 Đây đúng lớp rot mà commit `3e76711` đã đặt tên (*"story giết rot số đếm đã
tái sản xuất đúng nó"*) — lần này là một story dời luật khỏi tầng sai, tự đóng đinh một con số sai
vào ba chỗ vĩnh viễn. ⇒ Cả ba chỗ nay ghi **20** kèm **lệnh đếm** và **ngày**, cộng một câu dặn
đừng tin lại con số cũ. Cùng khuôn, một con số `703` đóng đinh trong câu `panic!` của cổng
`#[must_use]` đã được thay bằng chữ — một tổng số ca là mục tiêu di động, nó không thuộc về một câu
báo lỗi.

**Số dòng rot ngay trong commit sinh ra nó.** Mục nợ C4 trỏ `store.rs:715-719 · 787-791 · 922-926
· 1299-1303`, và §Design Notes của chính spec này trỏ ba câu `UPDATE` ở `249 · 880 · 1571`. Bản vá
chèn mã phía trên ⇒ bốn số của C4 lệch **38-61 dòng**, và **hai trong ba** số `UPDATE` sai (nay
`249 · 930 · 1694`). ⇒ Cả hai chỗ nay trỏ bằng **TÊN HÀM** (`entries_eligible_for_injection` ·
`resolve_term_for_quick_add` · `list_all_entries` · `marks_for_source_text`; và
`confirm_translation` · `update_manual_term` · nhánh `TakeTheirs` của `import_into_tier`).

**Mất nội dung ở bước biên soạn ngữ cảnh, và nó rơi đúng vào vùng cụm này chạm.** Lượt
`compile-epic-context` ở bước 1 của workflow **xoá hẳn** câu *"Lỗi IPC dùng mã, khoá thông báo,
tham số và khả năng thử lại; văn bản hiển thị thuộc tài nguyên giao diện"* khỏi
`epic-3-context.md` trong một lượt "cắt cho gọn" (kiểm: `grep "khoá thông báo"` trên bản mới = **0**).
Đó là ràng buộc kiến trúc về đúng thứ lượt vá này vừa dựng — biến thể lỗi mới ⇒ khoá thông báo ⇒
tham số. ⇒ Khôi phục nguyên văn.

**Cổng `#[must_use]` dễ đỏ oan.** Nó so khớp chuỗi nhiều dòng nguyên văn, và kho **không có cổng
`cargo fmt`**. ⇒ Câu `assert!` nay nói rõ hai chiều: định dạng đổi thì cập nhật chuỗi mốc (không
phải bug); `#[must_use]` biến mất thì ĐÓ LÀ bug, đừng sửa ca test cho hết đỏ.

**Ghi nợ (1):** một hàng bị XOÁ giữa hai nhịp làm biến mất mọi va chạm đã gom được của những hàng
KHÁC trong cùng lô — nhánh `still_present == None` `return` ngay, không xả hai kênh
`Arc<Mutex<…>>`. **0 lượt ghi ở mọi nhánh**, nên đây là chất lượng CHẨN ĐOÁN, không phải mất dữ
liệu. Không vá trong lượt này vì bản vá đúng sẽ ĐỔI lỗi người dùng thấy ở hàng ④, mà hàng ④ nằm
trong khối `<frozen-after-approval>` Ice đã ký (*"Giữ NGUYÊN hành vi hiện có… không đổi nhãn"*) —
đổi nó là renegotiate. Chủ: Ice. Chi tiết trong `deferred-work.md`.

**Bác (5), ghi ra để lượt sau khỏi bới lại:** ① *"`epic-3-context.md` là tệp sinh ra mà lại sửa
tay"* — tiền đề sai, nó do `compile-epic-context` sinh lại ở bước 1 workflow này, đúng thứ header
của nó dặn (vế MẤT NỘI DUNG trong cùng phát hiện thì đúng, đã vá ở trên). ② *"thứ tự ⑦ né cửa hỏi
Ice"* — cửa §Ask First có ĐIỀU KIỆN (*"mà thứ tự báo không suy ra được từ mã hiện có"*); khối kiểm
`UNIQUE` đã đứng đó từ P6 cụm B nên điều kiện không nổ. ③ câu chữ story-ID không nhất quán trong
một tệp sinh lại — nhiễu. ④ thiếu ca test cho tổ hợp C3+C1 cùng lô — phép kiểm C3 chạy TRƯỚC mọi
giao dịch nên ⑤/⑥ đã khoá vế "0 lượt ghi". ⑤ hai câu lỗi `import_stale_conflict` /
`import_unique_conflict` "khó phân biệt" — chúng khác nhau ở đúng vế cần khác (*"vừa được thêm"* ↔
*"đã bị đổi… trong lúc xem trước"*).

**Số đo sau cùng, đọc `$?` thật:** `cargo test --locked` **705** ca PASS, 0 failed (695 gốc → 705:
+7 ca C1/C3, +1 doctest, +1 cổng `#[must_use]`, +1 ca P1 của vòng rà). `.githooks/pre-push` exit 0,
mười một cổng xanh trong **161s** — chạy LẠI sau cả phần sửa tài liệu, vì cổng `debt-owner` canh
`deferred-work.md`. 🔵 **SỬA:** con số **703** ở hai mục trên hết đúng ngay khi ghi (cổng
`#[must_use]` thêm sau khi nó được đo, rồi ca P1 thêm nữa) — đúng lớp rot mà chính mục này vừa vá
ở ba chú thích. Ghi lại: **705**, đo sau cùng.

## Design Notes

**C2 bị bác — ba phép đo, không một suy luận.** Sổ nợ khai *"rowid tái dùng trỏ nhầm DANH TÍNH hàng"*. Đo trên `db3833b`:

1. `glossary_entry` khai `id INTEGER PRIMARY KEY AUTOINCREMENT` (`schema.rs:302`), và doc-comment ngay trên hằng anh em (`schema.rs:147-148`) nói rõ vì sao: *"AD-3 nói id đã về hưu không bao giờ được phát lại, và `INTEGER PRIMARY KEY` trần tái dùng rowid lớn nhất vừa xoá"*. ⇒ **cơ chế mà C2 nêu tên đã bị chặn ở tầng lược đồ.**
2. Tiền đề thứ hai, tự kiểm vì một tiền đề sai không làm kết luận sai: `source_term` có bao giờ đổi dưới một `id` còn sống không? `grep "SET source_term" src/ tests/` = **0 khớp**. Toàn kho có **ba** câu `UPDATE glossary_entry` — `confirm_translation` · `update_manual_term` · nhánh `TakeTheirs` của `import_into_tier` — cả ba chỉ chạm `translation`/`note`/`category`. 🔵 **SỬA 2026-08-25 (vòng rà):** bản đầu trỏ `store.rs:249 · 880 · 1571`; hai trong ba số đó sai **ngay trong commit tạo ra chúng** (nay là `249 · 930 · 1694`) vì chính bản vá chèn mã phía trên. Trỏ bằng TÊN, đừng trỏ bằng số dòng. ⇒ **cặp `(id, source_term)` bất biến suốt vòng đời một hàng.**
3. Bước di trú duy nhất chép `id` nguyên vẹn (`schema.rs:548-551`, `SELECT id, source_term, …`) và nâng `sqlite_sequence` theo — nó không đánh số lại.

⇒ `WHERE id` **đủ** để trỏ đúng danh tính, và ca "hàng biến mất" đã có `row_missing_error` bắt (`:1583`). C2 đóng bằng `→ KHÔNG LÀM` kèm ba phép đo trên. ⚠️ Ba phép đo này đọc **trạng thái hôm nay**: story đầu tiên thêm một đường **đổi tên thuật ngữ** làm chúng hết đúng ngay lập tức — ghi mệnh đề đó vào sổ nợ cùng lượt.

**Vì sao phép so phải NULL-an-toàn.** Vòng đời ba trạng thái cho phép `translation IS NULL` (mục *chờ chốt*), và `TakeTheirs` trên một mục chờ chốt là ca thường, không phải ca hiếm. `WHERE translation = ?` với tham số `NULL` cho **UNKNOWN**, không cho `TRUE` — mọi lượt như thế sẽ khớp 0 hàng và bật một lỗi giả. SQLite có toán tử so NULL-an-toàn; hàng ③ của §I/O Matrix tồn tại đúng để khoá vế này, và nó phải có một ca riêng.

**Vì sao C3 dời xuống lõi thay vì chép thêm một bản.** `commands/mod.rs:2` viết bằng chữ: *"Adapter thuần, KHÔNG chứa quy tắc nghiệp vụ (AD-1)."* Phép kiểm ở `glossary.rs:863-873` **là** một quy tắc nghiệp vụ, và nó là mắt duy nhất — 13 lời gọi thẳng `import_into_tier` trong `glossary_exchange_contract.rs` chứng minh đường vòng qua lớp `commands` không phải lý thuyết. Nhân đôi luật thành hai bản là đúng lớp lỗi mà cụm A vừa ghi thành nợ (*"một cổng có một BẢN SAO, và không cổng nào canh cho hai bản khớp nhau"*) — đừng dựng thêm một bản sao nữa.

**C5 là một lưới, và phải nói đúng nó là gì.** Đã rà mọi chỗ sinh/tiêu thụ vé: **0 chỗ đang thả vé**. Nên `#[must_use]` không sửa một lỗi đang sống; nó chặn lỗi kế tiếp. ⚠️ **Giới hạn có tên, còn nguyên sau lượt nâng lint:** `#[must_use]` (và `deny` của nó) chỉ nổ khi giá trị bị thả **ngay tại một câu lệnh**. `let ticket = store.write_ticket(job)?;` rồi để `ticket` rơi ra khỏi phạm vi mà không `.wait()` thì **không** lint nào bắt — viết giới hạn đó vào doc-comment tại chỗ để cổng này không nói quá về thứ nó canh.

🔵 **SỬA 2026-08-25 (muộn hơn) — Ice chốt §Ask First: đã NÂNG.** Câu trên ban đầu dừng ở "không tự đỏ được, xem §Ask First" vì hôm đó không cổng nào chạy clippy và `unused_must_use` là warn-by-default. Ice đọc số đo (`RUSTFLAGS="-D warnings" cargo check --locked` → 0 cảnh báo) rồi chốt nâng đúng lint này thành `deny` trong `[lints.rust]` của `Cargo.toml` — xem §Spec Change Log mục "Ice chốt §Ask First của C5" cho toàn bộ phép đo và bốn đối chứng gỡ chỗ nối.

## Verification

**Commands:**
- `npm run build` — expected: exit 0. **Chạy TRƯỚC `cargo test`** — thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch, không ở một assert.
- `cd src-tauri && cargo test --locked` — expected: exit 0; số ca **tăng** so với **691** của lượt đo 2026-08-25 (cụm B). Ghi con số thật trước/sau.
- `npm run check:i18n` — expected: exit 0, số khoá `vi.json` tăng đúng **1**, 0 miễn trừ mới.
- `.githooks/pre-push` — expected: exit 0. ⚠️ Chạy trên macOS của Ice; nó **không** nói gì về nửa Windows — đọc lượt CI trước khi kết luận là xanh.
- `cd src-tauri && RUSTFLAGS="-D warnings" cargo check --locked` — chỉ để **đo** cho §Ask First của C5 (bao nhiêu cảnh báo hiện có), **không** để chốt cấu hình.

**Manual checks (if no CLI):**
- **Ba phép đối chứng gỡ-chỗ-nối, chạy thật, một mục một lượt:** gỡ riêng bản vá C1, rồi C3, rồi C5; mỗi lượt chạy bộ test, ghi **tên ca đỏ** và **số ca đỏ**, rồi khôi phục. Một mục mà bộ test vẫn xanh trọn nghĩa là ca của nó chưa chạm bề mặt — **sửa ca, không sửa kết luận**.
- **Đối chứng chiều ngược cho C1:** chạy riêng sáu ca `TakeTheirs` hiện có **trước** khi thêm ca mới. Nếu một ca đỏ ⇒ dừng theo §Ask First.

## Suggested Review Order

**So lạc quan — chỗ đọc sai dễ nhất của cả lượt vá**

- Điều kiện ghi mang theo giá trị người dùng đã nhìn; `IS` chứ không `=`.
  [`store.rs:1701`](../../src-tauri/src/core/glossary/store.rs#L1701)

- `changed == 0` nay mang HAI nghĩa; đọc lại trong cùng giao dịch để tách.
  [`store.rs:1718`](../../src-tauri/src/core/glossary/store.rs#L1718)

- Thứ tự báo khi lô có cả hai loại va chạm, và vì sao nó suy ra được.
  [`store.rs:1772`](../../src-tauri/src/core/glossary/store.rs#L1772)

**Lớp lỗi mới được cấp tên — bốn mắt, thiếu một là một cổng đỏ**

- Biến thể mang danh sách, không mang một thuật ngữ lẻ.
  [`store.rs:491`](../../src-tauri/src/core/glossary/store.rs#L491)

- Mắt duy nhất không cổng nào canh trước vòng rà — nay có ca riêng.
  [`store.rs:663`](../../src-tauri/src/core/glossary/store.rs#L663)

- Khoá và tham số; `params` mang dữ liệu, không mang câu.
  [`i18n/mod.rs:390`](../../src-tauri/src/core/i18n/mod.rs#L390)

- Câu phải khác `import_unique_conflict` ở đúng vế người dùng cần phân biệt.
  [`vi.json:43`](../../src/i18n/vi.json#L43)

**Luật về đúng tầng — AD-1**

- Phép kiểm nay ở lõi, trước khi mở giao dịch; giữ "0 lượt ghi".
  [`store.rs:1633`](../../src-tauri/src/core/glossary/store.rs#L1633)

- Adapter thôi tự kiểm, chỉ để lỗi đi xuyên qua — không giữ hai bản.
  [`glossary.rs:861`](../../src-tauri/src/commands/glossary.rs#L861)

**Vé ghi — một lưới, không phải một lỗi đang sống**

- Dấu trên KIỂU, kèm giới hạn có tên viết tại chỗ.
  [`writer.rs:135`](../../src-tauri/src/core/store/writer.rs#L135)

- Vỏ bọc không thừa hưởng dấu của kiểu bên trong nó.
  [`candidate_store.rs:174`](../../src-tauri/src/core/glossary/candidate_store.rs#L174)

- Đúng một lint được nâng, kèm phép đo cho phép nâng.
  [`Cargo.toml:137`](../../src-tauri/Cargo.toml#L137)

**Cổng và ca test**

- Ca sinh ra vì đo được rằng lint một mình không làm bản vá đỏ được.
  [`config_invariants.rs:997`](../../src-tauri/tests/config_invariants.rs#L997)

- Ca của vòng rà: cắt thử đổi khoá thông báo ⇒ đúng ca này đỏ.
  [`glossary_import_dialog_contract.rs:673`](../../src-tauri/tests/glossary_import_dialog_contract.rs#L673)

- Lượt ghi chen ngang giữa hai nhịp, dựng thật chứ không giả lập.
  [`glossary_exchange_contract.rs:1354`](../../src-tauri/tests/glossary_exchange_contract.rs#L1354)

- Bẫy `NULL`: mục chờ chốt phải KHỚP, không được thành lỗi giả.
  [`glossary_exchange_contract.rs:1404`](../../src-tauri/tests/glossary_exchange_contract.rs#L1404)

- Gọi thẳng hàm lõi, bỏ qua adapter — đường mà C3 để hở.
  [`glossary_exchange_contract.rs:1524`](../../src-tauri/tests/glossary_exchange_contract.rs#L1524)

- Lô hỗn hợp: thứ tự báo tất định, khoá bằng một ca riêng.
  [`glossary_exchange_contract.rs:1586`](../../src-tauri/tests/glossary_exchange_contract.rs#L1586)
