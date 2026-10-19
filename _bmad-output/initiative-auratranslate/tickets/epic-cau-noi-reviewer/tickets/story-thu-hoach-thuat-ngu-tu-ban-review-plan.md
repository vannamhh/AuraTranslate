---
title: 'Thu hoạch thuật ngữ từ bản review'
type: 'feature'
ticket: '14'
created: '2026-10-09'
status: 'built'
baseline_revision: '35cc84af889e7a37a512d1d08aec95b560221d65'
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

**Problem:** Reviewer sửa cách gọi một thuật ngữ (ví dụ «Bắc Lương vương» thành «vương Bắc Lương») nhưng bài học đó nằm chết trong `review_row`. Chương sau AI vẫn dùng cách gọi cũ, vì Glossary không biết gì (FR54).

**Approach:** Sau khi xác nhận nhập, Rust đọc các cặp alignment đã khớp (AD-52 mục 7) và tìm các thay đổi X→Y có một thuật ngữ nguồn S đứng sau. Mỗi cặp được ghi thành một ứng viên `review_harvest` trong bảng chờ của Epic 3 (AD-20), mang số lần đổi N trên tổng số lần xuất hiện M. Khi người dùng duyệt, mục vào Glossary ở trạng thái đã chốt S→Y và mang xuất xứ `review_harvest`. RAG tự chèn mục này vì `confirmed_terms_for_injection` chỉ lọc theo `is_confirmed`.

## Boundaries & Constraints

**Always:**
- Thu hoạch chỉ đọc bản reviewer qua `read_alignment`, không sửa `review_row`, `segment` hay alignment.
- Thu hoạch chạy sau giao dịch xác nhận nhập, trong một giao dịch riêng. Nếu thu hoạch lỗi thì bản nhập vẫn được giữ, và lỗi hiện thành một lời báo có kiểu riêng.
- Mọi SQL chạm bảng Glossary nằm trong `core/glossary/` (`glossary_boundary.rs`).
- Đếm lại trên mọi bản reviewer không `Stale` của Tác phẩm, không cộng dồn qua các lần nhập (AD-52 bác "giữ lịch sử" vì đếm trùng).

**Never:**
- Ghi thẳng vào `glossary_entry` mà không qua lượt duyệt.
- Đề xuất lại một cặp (S, Y) đã bị bỏ trong cùng Tác phẩm.
- Tự quyết thay người dùng mức nhất quán. Tỉ lệ N/M luôn hiện ra.
- Làm lời báo "chắc chắn nhìn thấy" và ca "gỡ Review Mode vẫn xanh". Hai việc đó thuộc 8.15.

## Decisions (Ice, 2026-10-09)

- **Phạm vi:** chỉ thuật ngữ S đã chốt trong Glossary (cả hai tầng sau `ScopeResolver`), với X là bản dịch đã chốt của S. Thuật ngữ chưa có trong Glossary không được thu hoạch ở story này và được ghi nợ.
- **Luật đề xuất:** mọi cặp có N ≥ 2 đều được đề xuất, ở bất kỳ tỉ lệ nào. Hàng có N/M ≤ 50 % mang nhãn "không nhất quán" (`review-mode.html`, FR54 "để người dùng tự phán xét").
- **Cỡ plan:** giữ một story dù plan dài khoảng 3000 token; công việc vẫn chia pha agent theo AGENTS.md.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Đổi nhất quán | Glossary có S→X; trong 4 cặp đã khớp, nguồn chứa S và bản tôi chứa X; reviewer đổi X→Y ở cả 4 | một ứng viên S: X→Y, 4/4 | — |
| Đổi một phần | 3/5 lần đổi sang Y, 2 lần giữ X | ứng viên 3/5; hàng 2/5 mang nhãn "không nhất quán" | — |
| Reviewer giữ nguyên | X còn nguyên ở mọi lần | không có ứng viên | — |
| Đổi đúng một lần | N = 1 | không có ứng viên | — |
| Cặp đã bỏ | (S, Y) có `resolution='rejected'`; nhập bản review khác | không sinh lại | — |
| Ứng viên đang chờ | (S, Y) còn chờ duyệt; nhập thêm Chương | cập nhật N/M, không thêm hàng | — |
| Duyệt, S ở tầng Work | mục Work S→X | mục đó thành S→Y, `term_origin='review_harvest'`, đã chốt | — |
| Duyệt, S chỉ ở tầng Global | mục Global S→X | thêm mục Work S→Y che mục Global (AD-18) | — |
| Bản `Stale` / chưa khớp xong | Chương lỗi thời, hoặc nhóm chỉ có một phía | bỏ qua Chương hay nhóm đó, không lỗi | — |

</frozen-after-approval>

## Code Map

Đường dẫn Rust tính từ `src-tauri/src/`.

- `commands/export.rs:425-445` (`reviewer_import_confirm`) -- gọi thu hoạch ở đây sau `store.write(confirm_import)`. Thêm số ứng viên vào `ReviewerImportSummaryWire` `:292`. `read_alignment` cần `&Store` và tự ghi, nên không gọi được bên trong giao dịch xác nhận.
- `core/export/alignment.rs:39-76,437` -- `ChapterAlignment` (rows, segments, groups). Ghép một nhóm thành chuỗi "của tôi"/"của reviewer" theo đúng cách `review_diff` `:516-545` đã làm. Tách phần ghép đó thành hàm dùng chung, không chép.
- `core/matching/mod.rs:654` -- `find_terms(text, terms, lang)` tìm S trong `source_text`. `match_lang_for_source_lang` ở `core/glossary/store.rs:1265`. ⚠️ `tokenize` En cắt theo ASCII nên làm vỡ chữ có dấu tiếng Việt: tìm X/Y trên chuỗi NFC theo ranh giới khoảng trắng, không dùng `tokenize`.
- `core/glossary/store.rs` -- `load_tier` `:268`, `update_manual_term` `:960` (khuôn UPDATE một câu), `confirmed_terms_for_injection` `:1605` (không đổi).
- `core/glossary/candidate_store.rs` -- `enqueue_import_scan_candidates` `:245-296` là khuôn ghi theo lô. Hiện `ON CONFLICT(source_term)` của nó phải đổi theo chỉ mục mới. `approve_candidate` `:311-360` và `reject_candidate` `:371`.
- `core/glossary/candidate.rs:30,58-63,114-133` -- `CandidateOrigin::ReviewHarvest` và phép ánh xạ sang `TermOrigin` đã có. Thêm trường vào `GlossaryCandidate`.
- `core/store/schema.rs:410-463` -- DDL `glossary_candidate`, và `UNIQUE idx_glossary_candidate_source_term` `:432`. Migration mới là v32, đặt sau `ALIGNMENT_DDL` `:2296`. Các assert phiên bản sẽ trượt: xem danh sách đã sửa ở 8.10 (`pinned_contract.rs`, `segment_contract.rs`, `chapter_origin_contract.rs`, `segment_role_contract.rs`, `tm_contract.rs`).
- `commands/glossary.rs:322-396` -- `GlossaryCandidateWire` / `build_pending_candidates`. Thêm X, Y, N, M.
- Webview -- `src/config/glossary.ts:428-455` (kiểu + validator), `src/glossaryQueueState.ts:281` (khi duyệt ứng viên `review_harvest`, gửi Y thay cho gợi ý Hán Việt), `src/GlossaryQueueOverlay.vue:255-273` (hàng "X → Y · đổi ở N/M lần"), `src/config/reviewerImport.ts:26,119` và `src/ReviewerImportOverlay.vue:104-113` (thêm một dòng số ứng viên), `src/i18n/vi.json`. Mockup: `ux-auratranslate/mockups/review-mode.html:208-260`.
- Cổng: `tests/glossary_boundary.rs` (`QUICK_ADD_SURFACE` `:165` cho hàm ghi mới gọi từ `commands/`; `TermOrigin::ReviewHarvest` chỉ được viết trong `core/glossary`). Thêm tệp `.rs` thì 18 sàn `*_RS_FLOOR = 105` đang đúng ở mép (131 tệp) và `dict_boundary.rs:269` sẽ đỏ, nên nâng tất cả lên `ceil(0.85×live)`. Tương tự cho các sàn webview nếu thêm tệp `.ts`/`.vue`.

## Tasks & Acceptance

**Execution:**
- [x] `core/store/schema.rs` + các assert phiên bản -- v32: thêm `replaced_translation`, `proposed_translation`, `changed_count`, `seen_count`. `CHECK`: cả bốn cột khác NULL khi và chỉ khi `candidate_origin='review_harvest'`. Thay `UNIQUE(source_term)` bằng hai chỉ mục một phần: `(source_term) WHERE candidate_origin='import_scan'` và `(source_term, proposed_translation) WHERE candidate_origin='review_harvest'`.
- [x] `core/export/harvest.rs` (mới) -- `harvest_work(global, work, source_lang) -> Vec<HarvestFinding>`: thuần đọc, trả về cặp và số đếm theo Design Notes.
- [x] `core/glossary/candidate_store.rs` -- `enqueue_review_harvest(store, findings)`: bỏ qua (S, Y) đã quyết; ứng viên đang chờ thì cập nhật N/M. Sửa `enqueue_import_scan_candidates` theo chỉ mục mới. `approve_candidate` với ứng viên `review_harvest`: nếu tầng Work đã có S thì UPDATE `translation` và `term_origin`, nếu chưa thì INSERT với Y.
- [x] `commands/export.rs` + `commands/glossary.rs` + webview + `vi.json` -- nối lời gọi, thêm wire, hàng bảng chờ, dòng ở màn đã nhập. Lỗi thu hoạch dùng khoá `err.export.harvest_failed` riêng.
- [x] `tests/review_harvest_contract.rs` (mới) -- cả ma trận I/O. Bản reviewer dựng bằng hàm xuất thật của 8.3 rồi sửa bằng mã. Thêm một ca nối tới `confirmed_terms_for_injection` để khoá AC RAG. Đối chứng gỡ: bỏ lọc `rejected`, bỏ lời gọi ở `reviewer_import_confirm`, bỏ nhánh UPDATE khi duyệt.
- [x] `tests/frontend/` -- hàng `review_harvest` hiện X → Y, N/M; phím N gửi Y.
- [x] `deferred-work.md` -- đóng mục `UNIQUE (source_term)` (`Chủ: Story 8.14`, quanh dòng 5332). Thêm một mục mới: thu hoạch thuật ngữ chưa có trong Glossary (dò S theo đồng xuất hiện), `Chủ: Ice`.

**Acceptance Criteria:**
- Given xác nhận nhập xong, when đọc `glossary_entry`, then không có hàng nào mới hoặc bị đổi.
- Given một ứng viên `review_harvest` được duyệt, when gọi `confirmed_terms_for_injection` cho đoạn nguồn chứa S, then trả về S→Y.
- Given thu hoạch lỗi SQL, when nhập xong, then bản reviewer vẫn đọc được và webview hiện lời báo thu hoạch, không phải "tệp không đọc được".

## Design Notes

- **Thuật toán, cho từng nhóm có cả hai phía:**
  - S là các thuật ngữ đã chốt của Glossary (cả hai tầng, sau `ScopeResolver`) tìm thấy trong nguồn của nhóm.
  - X là bản dịch đã chốt của S. M cộng số lần X xuất hiện trong chuỗi của tôi, ở những nhóm mà nguồn có S.
  - Nhóm có số lần X trong chuỗi reviewer ít hơn trong chuỗi của tôi là nhóm "đổi". Phần chênh cộng vào N.
  - Y là n-gram từ (1..|X|+2 từ) xuất hiện trong chuỗi reviewer, không có trong chuỗi của tôi, xếp hạng theo: số nhóm "đổi" chứa nó (giảm dần), rồi số từ chung với X (giảm dần), rồi độ dài (tăng dần). Cách này bắt được cả đảo trật tự: «Bắc Lương vương» → «vương Bắc Lương» thắng «vương Bắc» nhờ chung ba từ với X, là ca mà diff Delete/Insert tách vụn. Một đổi tên hẳn («đả khai» → «mở») không chung từ nào nên ngắn nhất thắng, loại n-gram dính ngữ cảnh.
  - Các ví dụ này là ca test bắt buộc.

## Verification

**Commands:**
- `cd src-tauri && cargo test --test review_harvest_contract` -- xanh; mỗi phép gỡ đối chứng làm đỏ đúng ca của nó.
- `cargo test --test glossary_contract --test glossary_boundary --test reviewer_import_contract` -- vẫn xanh.
- Có migration nên chạy cả bộ một lần bằng tay trước khi báo xong.

## Implementation Notes

- `CHECK` của v32 chặt hơn chữ của plan: bốn cột đều có giá trị với `review_harvest` và đều `NULL` với `import_scan`. Đọc theo chữ "có giá trị khi và chỉ khi" thì một cột lẻ vẫn lọt vào hàng `import_scan`. Vì vậy `insert_candidate` không còn tạo được hàng `review_harvest`, và các ca cũ dùng helper `insert_harvest_candidate`.
- `confirmed_term_translations` là cửa mới của `core/glossary`, vì `load_tier` bị cấm ngoài module. Phần ghép văn bản nhóm trong `review_diff` được tách thành `group_texts` để thu hoạch dùng chung.
- `reviewer_import_confirm` nhận thêm `global: Option<&Store>`. Thiếu kho global là lỗi thu hoạch có kiểu, không phải 0 ứng viên im lặng.
- `QUICK_ADD_SURFACE` của Code Map là danh sách hàm mà `commands/glossary.rs` PHẢI gọi, không phải danh sách cho phép. Hàm mới được gọi từ `commands/export.rs` nên không thuộc danh sách đó. Các cổng thật (tên bảng, token `TermOrigin`, `GLOSSARY_ONLY_SURFACE`) vẫn xanh.
- Đối chứng gỡ: bỏ phép lọc `rejected` làm đỏ đúng ca cặp đã bỏ; bỏ lời gọi thu hoạch ở `reviewer_import_confirm` làm đỏ 8 ca; bỏ nhánh UPDATE khi duyệt làm đỏ 2 ca.
- Giới hạn đã biết: hai thuật ngữ cùng đổi kiểu đổi tên hẳn, nằm chung đúng các nhóm, có thể hoà hạng nên Y của thuật ngữ này đề xuất cho thuật ngữ kia. Đề xuất sai vẫn hiện ra để người dùng bỏ, không ghi gì.
- Lượt chạy cả bộ (máy load ~60–228): build xanh, `cargo test` xanh cả 86 target. Vitest có 1 ca đỏ, `settingsFrame.test.ts` quá hạn 5000 ms; tệp này story không chạm, chạy lại hai lần đều xanh 10/10.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 6 · false 1 · maybe-false 0.

- medium · patch -- ứng viên `review_harvest` đang chờ mà lượt đếm lại không còn thấy giữ N/M cũ mãi, dưới luật N ≥ 2; xoá hàng đang chờ không còn trong lô, cùng giao dịch.
- low · patch -- `reviewer.import.harvest_done` nói "đang chờ duyệt" nhưng đếm `inserted + updated`; sau patch trên, số này bằng số hàng đang chờ, có ca khoá.
- low · patch -- giới hạn độ dài Y lấy theo X dài nhất của cả lượt, không theo |X|+2 của từng thuật ngữ như Design Notes.
- low · patch -- ba banner `// ─────` và tham số `name` bị bỏ qua trong `review_harvest_contract.rs`.
- low · patch -- hai dòng doc vừa sửa còn tiếng Việt (`candidate.rs` `source_term`, `i18n` `ExportHarvestFailed`); header "Ba mươi mốt bước" là comment máy đọc nên giữ.
- low · reject -- duyệt khi S chỉ ở tầng Global thì mục Work che nó mang `category` hàng chờ gửi (mặc định `other`), còn nhánh UPDATE giữ category cũ; ca hiếm (tên nhân vật thường ở tầng Work), sửa cần mang category của mục đang có qua wire.
- low · reject -- tập n-gram mới dùng chung cho mọi thuật ngữ của một nhóm nên Y của thuật ngữ này có thể thắng cho thuật ngữ kia khi hai bên hoà hạng; cần cả hai đổi tên hẳn và trùng đúng tập nhóm, đề xuất sai vẫn hiện ra để bỏ, sửa cần logic gán vị trí; ghi ở Implementation Notes.
- false -- "`QUICK_ADD_SURFACE` thiếu hai hàm mới": hằng đó là danh sách `commands/glossary.rs` phải gọi (`glossary_boundary.rs:584`), thêm hàm gọi từ `commands/export.rs` vào sẽ làm đỏ cổng.
- reject -- Implementation Notes trống, checkbox chưa tick, chưa có bằng chứng chạy cả bộ và đối chứng gỡ: sửa bằng chính plan; đã ghi ở Implementation Notes.
