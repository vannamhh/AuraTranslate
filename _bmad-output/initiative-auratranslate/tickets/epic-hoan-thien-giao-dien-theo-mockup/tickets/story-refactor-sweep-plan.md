---
title: 'Refactor sweep'
type: 'refactor'
ticket: '22'
created: '2026-10-19'
status: 'built'
baseline_revision: '1a212e1996eb17fa65ab9e61fe34789358af6b95'
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

**Problem:** Epic 12 để lại hai phép đo chưa làm (`export_preview` L13261, `refreshExportCounts` L13238), mã chết của nút gộp lên đã gỡ ở 12-5, hai hàm `reset*` không ai gọi khi đổi Tác phẩm, và một phát hiện review chưa có dạng đóng. Epic không lên `done` khi còn phát hiện hoãn chưa đóng hoặc chưa có chủ.

**Approach:** Đo hai lệnh trên bản release, gỡ mã chết gộp lên, nối hai hàm `reset*` vào chỗ đổi Tác phẩm, và ghi dạng đóng hoặc chủ cho mọi phát hiện hoãn của epic.

**Decisions (Ice, 2026-10-19):**
- Gỡ cả `merge_chapter_into_previous` lẫn `_indexed`; chuyển test sang `merge_chapters`; bỏ ca `at_first` vì không còn đường tới.
- Gọi `resetAlignment` và `resetReviewerImport` ở hai chỗ đổi Tác phẩm, đúng khuôn `resetGlossaryQueue`, kèm ca canh đỏ được. Đây là hành vi mới duy nhất của sweep.
- `row.display.join(' / ')` (11 chỗ): KHÔNG LÀM. Thân chỉ một dòng, và review 12-8 đã `reject` việc gom.
- 12.22 nhận phép đo `refreshExportCounts` (L13238): đổi chủ rồi đo bằng quần thể 200.000 cặp.
- Ngưỡng `export_preview`: p95 ≤ 50 ms (một frame, NFR2), vì lệnh plain chạy trên luồng chính. Nếu vượt thì DỪNG và đưa Ice hai cách sửa kèm số: giới hạn số câu đọc, hoặc `(async)`. Không tự chọn.

## Boundaries & Constraints

**Always:** mã lỗi, `message_key`, dây IPC và chuỗi giao diện không đổi; các mục `Chủ: Epic 12` (lượt dùng thật), `Winston`, `Ice`, `B7` giữ chủ; mỗi ca lỗi của `merge_chapter_into_previous` còn đường tới qua `merge_chapters` vẫn có ca canh tương đương; số đo ghi build, máy và quần thể; full suite chạy một lần ở cuối.

**Never:** nhận tính năng hay lượt kiểm tay của Epic 12; gom chuỗi phím; sửa phát hiện `reject` đã có lý do; dọn chú thích ngoài dòng đang sửa; sửa `export_preview`/`tm_export_counts` khi Ice chưa chọn.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Đổi Tác phẩm | Review Mode / nhập bản reviewer đã mở trên A, rồi mở hoặc nhập B | trạng thái alignment và reviewer-import về rỗng (`status 'unknown'`, overlay đóng) | không lỗi |
| Đo preview | Chương 2.000 câu, bốn định dạng, `--release` | p50/p95 mỗi định dạng; so với 50 ms | vượt ⇒ dừng, hỏi Ice |
| Đo TM | 200.000 cặp, hai tầng × `all`/`mine` | thời gian `tm_export_counts` và thời gian giữ `OpenWorkState` | chỉ ghi số |
| Gộp qua `merge_chapters` | các ca cũ chuyển sang `&[a, b]` | cùng kết quả hàng/lỗi như trước | `chapter_not_found`, `none_open` giữ nguyên |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/export.rs:408` (core), `:1209` (wire plain); `src-tauri/src/core/export/preview.rs:49` `build_preview` nạp trọn Chương đầu rồi `take(PREVIEW_LIMIT)`. Ca có sẵn: `tests/export_content_options_contract.rs:623-716`.
- `src-tauri/src/commands/tm.rs:644` `tm_export_counts` (đọc trọn tầng hai lần), wire `:989` `(async)`; gọi từ `src/tmManageState.ts:280`. Quần thể: `src-tauri/tests/story_e7_r4_population.rs:111` (builder `#[ignore]`, 200k cặp).
- Khuôn đo `#[ignore]` + biến môi trường: `tests/dict_sources.rs:2244-2274`.
- `src-tauri/src/commands/chapter.rs:889,1319-1330`: gỡ. `merge_chapters` (:925, ≥ 2 id liền nhau) và `merge_into_previous_tx` (:992) giữ nguyên.
- Chỗ gọi test cần chuyển: `project_contract.rs` (~27, gồm `:4749` at_first, `:5123` `_indexed`, `:5283` none_open, `:5322`/`:5397` not_found), `alignment_contract.rs:8,783,942,1506`, `reviewer_import_contract.rs:9,912,947,1054`, `review_harvest_contract.rs:8,498`. Các doc-comment nhắc tên: `schema.rs:1134,1406,1423`, `lifecycle.rs:106`, `project/mod.rs:2846,2875`, `project_contract.rs:5073,5292,5350,5402`.
- `src/alignmentState.ts:216`, `src/reviewerImportState.ts:239`: gọi cạnh `resetGlossaryQueue()` ở `src/modes/libraryImport.ts:464` và `src/modes/libraryChapters.ts:393`. Khuôn ca canh: `tests/frontend/glossaryQueue.test.ts:372`.
- `scripts/check-debt-owner.mjs:742` `PLANNED_TICKET_FLOOR` = 160.
- Phạm vi đã gom và các mục không nhận: `../handoff-refactor-sweep-phases/01-scope.md`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/tests/` (tệp đo mới, `#[ignore]`) -- đo `export_preview` bốn định dạng ở 2.000 câu và `tm_export_counts` ở 200k cặp; in p50/p95 và thời gian giữ khoá.
- [x] `src-tauri/src/commands/chapter.rs` + 4 tệp test + doc-comment -- gỡ hai hàm, chuyển ca sang `merge_chapters`, bỏ ca `at_first`.
- [x] `src/modes/libraryImport.ts`, `src/modes/libraryChapters.ts` + vitest -- gọi hai `reset*`; ca canh đỏ khi gỡ lời gọi.
- [x] `story-khung-app-thanh-tieu-e-ba-tab-va-cong-cu-plan.md` Review Triage Log -- dòng 🔵: `PLANNED_TICKET_FLOOR` nay 160.
- [x] `deferred-work.md` -- L13238 thêm `→ Chủ: Story 12.22` rồi đóng bằng số đo; L13261 đóng bằng số đo; thêm mục đóng (`→ ✅`) cho rò `reset*`.

**Acceptance Criteria:**
- Given ca canh `reset*`, when gỡ thật một lời gọi, then đúng ca đó đỏ vì trạng thái không về rỗng.
- Given `grep -rn merge_chapter_into_previous src-tauri`, when chạy, then rỗng.
- Given `npm run check:debt-owner`, when chạy, then xanh, và L13238/L13261 mang số đo cùng dạng đóng.
- Given full suite, when chạy một lần ở cuối, then xanh.

## Implementation Notes

- Đo bằng `--profile bench-release` thay `--release`: `--release` (panic=abort) đụng tên tệp lib như `Cargo.toml` đã cảnh báo; lần build LTO đầu ~25 phút.
- `export_preview` p95 lớn nhất 23,3 ms (`.docx` một khối), dưới 50 ms nên không có câu hỏi sửa; số đầy đủ ở L13261.
- `tm_export_counts` đo 200.000 cặp MỖI tầng (trường hợp xấu nhất, builder R-4 chỉ 100k/tầng); 2,3–3,2 s giữ `OpenWorkState`. Ice chọn 2026-10-19: 🟡, phần còn lại (đếm bằng `COUNT` SQL) `Chủ: Epic 12`.
- `MergeStep::Merged { into }` thành `Merged`: trường `into` hết người đọc sau khi gỡ `merge_chapter_into_previous`.
- `merge_chapters` tự kiểm `chapter_id` lạ trước mọi câu ghi, nên nhánh `NotFound`/`NoPrevious` của `merge_into_previous_tx` chỉ còn là phòng thủ; ca sparse-ord nay canh phép kiểm ngoài.
- Hai lời gọi `reset*` đặt ngay sau `resetGlossaryQueue()`; đối chứng: gỡ thật từng lời gọi trong 4 lời gọi, mỗi lần đúng ca tương ứng đỏ.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 4 · false 2 · maybe-false 0 (8 phát hiện; một phát hiện là sửa plan).
- medium · patch — `merging_a_chapter_that_is_not_open_leaves_last_chapter_id_alone` gộp `[second, third]` trong khi second đang mở, `merge_chapters` gọi `set_open_chapter` nên ca không còn canh "Chương không mở"; giữ Chương mở ngoài tập gộp.
- low · patch — L13261 thiếu quần thể (`draft`, `ExportContent` mặc định, Glossary rỗng, gọi core); thêm vào dòng đóng.
- low · reject — ca sparse-ord không còn tới phép kiểm trong `merge_into_previous_tx`: hàm riêng tư một chỗ gọi, `merge_chapters` kiểm trước mọi câu ghi nên hành vi canh (không đánh số lại) vẫn được canh; gỡ nhánh phòng thủ là đụng bất biến không cần.
- false — L13238 "Ice không chọn": Ice chọn 🟡 `Chủ: Epic 12` trong phiên (ghi ở Implementation Notes); `check:debt-owner` xanh sau khi ghi.
- reject — Implementation Notes/Tasks trống: sửa là sửa plan này.
- low · reject — tệp đo không chặn chạy sai profile: thêm nhánh kiểm cho một tệp `#[ignore]` chạy tay.
- false — ca `reset*` qua `vi.resetModules()`: hai assert trước (`no_chapter`, `ipc_unavailable`) chứng minh trạng thái đã lệch khỏi mặc định trước lượt đổi.
- low · reject — `handoff-refactor-sweep-phases/` lặp số đo của sổ: cùng khuôn các thư mục `handoff-*-phases` khác của Epic 12; sổ là nơi giữ số chốt.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --release --test <tệp đo> -- --ignored --nocapture` -- expected: in số đo.
- `cargo test --test project_contract --test alignment_contract --test reviewer_import_contract --test review_harvest_contract --test export_content_options_contract` -- expected: xanh.
- vitest `reviewMode`, `reviewerImport` và tệp ca canh `reset*` -- expected: xanh.
- `npm run check:debt-owner` -- expected: xanh.
- Full suite một lần ở cuối -- expected: xanh.
