---
title: 'Thu hoạch chạy độc lập với Review Mode'
type: 'feature'
ticket: '15'
created: '2026-10-09'
status: 'built'
baseline_revision: 'c3e953798a6804639cfcb6443fa4285fd155d7a7'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Thu hoạch của 8.14 đã chạy ngay trong `reviewer_import_confirm`, nhưng FR95 còn hở hai chỗ. Thứ nhất, số ứng viên chỉ hiện trong lớp phủ nhập; đóng lớp phủ là mất, và nút bảng chờ trên thanh tiêu đề không có số. Thứ hai, chưa có gì canh giữ việc thu hoạch độc lập: `reviewerImportState.ts` import thẳng `resetReviewMode` từ `reviewModeState`, nên gỡ Review Mode sẽ làm vỡ đường nhập.

**Approach:** Đưa ứng viên thu hoạch ra một chỗ còn thấy được sau khi đóng lớp phủ nhập. Đảo chiều phụ thuộc giữa phần nhập và Review Mode. Thêm một cổng quét mã để khoá chuỗi phụ thuộc: phía thu hoạch không được nhắc tới ký hiệu nào của Review Mode.

## Boundaries & Constraints

**Always:**
- Đường thu hoạch gồm Rust (`harvest.rs`, `harvest_reviewer_copies`, `enqueue_review_harvest`), webview (`reviewerImportState.ts`, `glossaryQueueState.ts`, chỗ hiện ứng viên) và test của 8.14/8.15. Đường này không tham chiếu `review_diff`, `diff_spans`, `ReviewDock`, `reviewModeState`, `reviewScrollSync` hay panel `Review*`.
- Review Mode vẫn được làm mới sau khi nhập xong, nhưng phải qua móc nối mà `main.ts` gắn vào, theo khuôn `*CommandDeps.ts`.
- `group_texts` ở `core/export/alignment.rs` là hạ tầng alignment của 8.10, không phải Review Mode, nên giữ nguyên chỗ.

**Never:**
- Không đổi thuật toán, luật N ≥ 2 hay lược đồ của 8.14.
- Không thêm lệnh Tauri nếu lệnh `pending_glossary_candidates` đã có là đủ.
- Không dựng framework thông báo chung (`StatusBar.vue` :17-19 tự cấm điều đó).

## Decisions (Ice, 2026-10-09)

- **Chỗ hiện ứng viên (AC4):** chọn B. Đóng lớp phủ nhập khi đã nhập xong và có ứng viên thu hoạch thì bảng chờ Glossary tự mở. Lý do: khớp mockup `review-mode.html:210-260` (khung N/B, nút "Để lại trong bảng chờ"). Không thêm số trên nút thanh tiêu đề.
- **Cỡ plan:** giữ một story dù plan khoảng 2800 token; công việc vẫn chia pha agent theo AGENTS.md.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Nhập, chưa từng mở Review Mode | bản review sinh 2 ứng viên; đóng lớp phủ nhập | bảng chờ tự mở, có 2 hàng `review_harvest` | — |
| Nhập, không có ứng viên | `harvest_candidate_count = 0` | đóng lớp phủ, không mở gì thêm | — |
| Thu hoạch lỗi | `harvest_error` khác null | lời báo lỗi như 8.14; đóng lớp phủ, bảng chờ không tự mở | — |
| Huỷ trước khi xác nhận | đóng lớp phủ khi chưa `done` | không mở bảng chờ | — |
| Để lại trong bảng chờ | bảng chờ tự mở, người dùng đóng ngay | ứng viên còn nguyên; mở lại bằng nút thanh tiêu đề vẫn thấy | — |
| Bảng chờ đang mở sẵn | lời gọi mở lần hai | bỏ qua, như chốt tái mở của `openGlossaryQueue` | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/export.rs:429-481` -- `harvest_reviewer_copies` được gọi sau `store.write(confirm_import)`. Không đổi luồng.
- `src-tauri/src/core/export/harvest.rs:1-11` -- `use` chỉ gồm glossary, matching, scope, store, `alignment::group_texts` và `reviewer_copy`. Hiện chưa chạm Review Mode, và cổng mới khoá trạng thái này.
- `src-tauri/src/core/export/alignment.rs:530,581` -- `group_texts` được `harvest.rs:125` và `review_diff` gọi chung. Giữ nguyên.
- `src/reviewerImportState.ts:12,104` -- `import { resetReviewMode }` và lời gọi sau khi xác nhận thành công. Thay bằng một móc nối "sau khi nhập xong" do `main.ts` gắn với `resetReviewMode`, theo khuôn `reviewerImportCommandDeps.ts`.
- `tests/frontend/reviewerImport.test.ts:106-107` -- test đang import `reviewModeState` để dọn dẹp. Bỏ import này, dọn qua móc nối.
- `src/reviewerImportState.ts` `cancelReviewerImportPreview` (đoạn `wasDone`) -- chỗ đóng lớp phủ khi đã nhập xong; điểm phát móc nối "đóng sau khi nhập có ứng viên".
- `src/glossaryQueueState.ts:161` -- `openGlossaryQueue` (bỏ qua khi đã mở). Gắn nó trong `main.ts`, không import từ `reviewerImportState.ts`.
- `src/App.vue:264-272` -- nút `data-glossary-queue-open` là đường lui tiêu điểm của bảng chờ (UX-DR17); không đổi.
- `src/ReviewerImportOverlay.vue:114-120` -- dòng `harvest_done`/`harvest_error`, giữ nguyên.
- `src/i18n/vi.json:378` -- `reviewer.import.harvest_done`.
- Mockup `ux-auratranslate/mockups/review-mode.html:210-260` -- khung "Thêm vào Glossary Tác phẩm?", hàng N/B, nút "Để lại trong bảng chờ". `EXPERIENCE.md:406` -- "báo này xuất hiện dù Ice có mở Review Mode hay không". Không văn bản nào chỉ định bề mặt.
- Khuôn cổng: `src-tauri/tests/support/boundary_scan.rs` (`code_lines` :291, `rust_sources`, `any_sources`, `assert_population_floor`), dùng như ở `glossary_boundary.rs` (có đối chứng dương). `code_lines` không bỏ `<!-- -->`, nên chỉ quét `.ts`/`.rs`.
- Thêm tệp mới vào `src/` hay `src-tauri/src/` thì phải nâng các sàn quần thể 80% cùng lượt (`*_FLOOR` trong `tests/*_boundary.rs`, `scripts/check-*.mjs`; quyết định của epic, 2026-10-07).

## Tasks & Acceptance

**Execution:**
- [x] `src/reviewerImportState.ts` + `src/main.ts` + `tests/frontend/reviewerImport.test.ts` -- đảo chiều phụ thuộc: phần nhập phát móc nối "đã nhập xong", `main.ts` gắn `resetReviewMode` vào đó. Ca test khẳng định móc nối được gọi đúng một lần sau khi xác nhận thành công.
- [x] `src/reviewerImportState.ts` + `src/main.ts` -- khi đóng lớp phủ ở trạng thái `done`, `harvest_error` null và `harvest_candidate_count > 0`, phát móc nối; `main.ts` gắn `openGlossaryQueue` vào đó.
- [x] `tests/frontend/` -- ca theo ma trận I/O. Có một ca chạy luồng nhập mà không nạp module Review Mode nào, và vẫn thấy ứng viên.
- [x] `src-tauri/tests/harvest_boundary.rs` (mới) -- quét dòng mã (`code_lines`) của các tệp trong đường thu hoạch (Boundaries/Always) để tìm token Review Mode. Có đối chứng dương: chuỗi mẫu chứa token phải bị bắt. Có sàn: số tệp quét được phải đúng bằng số đã liệt kê, để không tệp nào mất im lặng.
- [x] Đối chứng gỡ: khôi phục `import { resetReviewMode }` trong `reviewerImportState.ts`, cổng phải đỏ đúng tệp đó. Gỡ móc nối ở `main.ts`, ca vitest phải đỏ.

**Acceptance Criteria:**
- Given người dùng nhập bản review có ứng viên mà chưa từng mở Review Mode, when đóng lớp phủ nhập, then bảng chờ Glossary mở ra với các hàng thu hoạch và tiêu điểm nằm trong bảng chờ.
- Given một token Review Mode được thêm vào một tệp trong đường thu hoạch, when chạy `cargo test --test harvest_boundary`, then cổng đỏ và nêu tệp:dòng.
- Given Review Mode đang mở khi nhập xong, when xác nhận nhập, then Review Mode vẫn được làm mới như trước.
- Given cắt Diff Viewer (R1), when đọc cổng, then đường thu hoạch không có phụ thuộc nào phải cắt theo.

## Verification

**Commands:**
- `cd src-tauri && cargo test --test harvest_boundary --test review_harvest_contract` -- xanh; đối chứng gỡ đỏ đúng ca.
- `npx vitest run tests/frontend/reviewerImport.test.ts tests/frontend/glossaryQueue.test.ts` cùng tệp test mới -- xanh.
- Story có `main.ts` và có thể thêm tệp làm trôi sàn, nên chạy các test sàn liên quan. Không cần chạy cả bộ, trừ khi chạm wiring dùng chung (AGENTS.md).

## Implementation Notes

- Móc nối là `installReviewerImportHooks({ afterImported, afterClosedWithHarvest })` trong `reviewerImportState.ts`; `main.ts` gắn `resetReviewMode` và `openGlossaryQueue`. Ô `hooks` cấp module được miễn trừ có tên trong `check:panel-refs` (dây nối lúc khởi động, cùng hạng `commands/index.ts::installedIsMac`), vì `resetReviewerImport` mà dọn nó thì khi được nối vào lượt đổi Tác phẩm sẽ cắt im lặng hai móc nối.
- `harvest_boundary.rs` quét 12 tệp nguyên văn và riêng thân `harvest_reviewer_copies`/`reviewer_import_confirm` của `commands/export.rs` (tệp đó còn chứa lệnh `review_diff`). `reviewerImportHooksWiring.test.ts` bị loại vì nó cố ý mock `reviewModeState` để ném nếu đường thu hoạch nạp nó.
- Tiêu điểm: `nextTick` của bảng chờ chạy sau cả hai watcher nên bảng chờ giữ tiêu điểm bất kể thứ tự mount; khi đóng bảng chờ, đích trả về (nút trong lớp phủ nhập đã gỡ) không còn nối nên rơi về `[data-glossary-queue-open]` (UX-DR17).
- Đối chứng gỡ, mỗi phép đỏ đúng ca của nó rồi khôi phục bằng sửa tay: khôi phục import `reviewModeState` (cổng đỏ ở `src/reviewerImportState.ts:12`); gỡ móc nối ở `main.ts` (ca nối dây đỏ); chèn `"review_diff"` vào `harvest_reviewer_copies` (cổng đỏ ở `commands/export.rs:430`); gỡ chốt tái mở của `openGlossaryQueue` (ca "mở lần hai" đỏ); gỡ `nextTick(() => panel.focus())` của bảng chờ (ca tiêu điểm đỏ).
- Không thêm tệp vào `src/` hay `src-tauri/src/`, nên không sàn quần thể nào đổi.
- Lượt chạy cả bộ bằng tay (vì sửa `scripts/check-panel-refs.mjs`), máy load 140–540: 12 cổng xanh, build xanh, `cargo test --no-fail-fast` xanh cả 87 target sau khi `harvest_boundary.rs` chuyển sang bộ duyệt và sàn dùng chung. Vitest đỏ 8 tệp không liên quan (`aiTranslateBatch`, `settingsFrame`, `tm*`, `importPreview*`, `libraryChapters`), tập đỏ đổi giữa hai lượt chạy lại; `aiTranslateBatch` + `settingsFrame` ở worktree gốc `c3e9537` cùng tải đỏ đúng 3 ca như cây đã sửa ⇒ do tải máy, chưa có lượt xanh trọn bộ.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 2 · low 2 · false 6 · maybe-false 0. 🔵 2026-10-09: một hàng false lật thành medium · patch, xem hàng "thiếu sàn".

- medium · patch -- cổng chỉ khớp bốn chuỗi cố định nên `resetReviewMode`/`openReviewMode`/`reviewModeCommandDeps` lọt nếu tệp không nhắc `reviewModeState`, và hai lớp phủ hiện kết quả thu hoạch không được quét; khớp mọi định danh chứa `reviewMode`/`ReviewMode`, thêm hai tệp `.vue`.
- medium · patch -- vế tiêu điểm của AC1 không có test; đọc mã thì `nextTick` của bảng chờ chạy sau cả hai watcher nên tiêu điểm vào bảng chờ, nay khoá bằng ca mount hai lớp phủ theo thứ tự `App.vue`.
- low · patch -- ca nối dây `main.ts` khớp cả dòng đã comment; bỏ dòng `//` trước khi khớp.
- low · patch -- ca "chạy không nạp Review Mode" để `doUnmock` ngoài `finally` và import lại `vitest`; sửa trực tiếp.
- false -- "thiếu sàn số tệp": mỗi tệp trong danh sách cố định được đọc hoặc panic, `function_text` panic khi không thấy chữ ký, nên không tệp nào mất im lặng. 🔵 2026-10-09: phán quyết sai — lượt chạy cả bộ đỏ ở `boundary_scan_contract` (mọi `*_boundary.rs` phải lấy tệp qua bộ duyệt dùng chung và gọi `assert_population_floor`); nay `load()` lấy tệp qua `paths_with_extensions` và sàn `HARVEST_UNIT_FLOOR` = 14, gỡ một tệp khỏi cây làm đỏ với 13/14.
- false -- "chưa có đối chứng gỡ": đã chạy — khôi phục import `reviewModeState` làm cổng đỏ ở `src/reviewerImportState.ts:12`, gỡ móc nối ở `main.ts` làm ca vitest đỏ, chèn `"review_diff"` vào `harvest_reviewer_copies` làm cổng đỏ ở `commands/export.rs:430`, gỡ chốt tái mở của `openGlossaryQueue` làm ca mới đỏ.
- false -- "móc nối rò giữa các ca": `fresh()` gọi `vi.resetModules()` rồi import lại `reviewerImportState`.
- false -- "miễn trừ `hooks` không cần": `resetReviewerImport` không được gọi ở mã thật; dọn móc nối trong đó thì khi nó được nối vào lượt đổi Tác phẩm sẽ cắt im lặng việc làm mới Review Mode; cùng hạng `commands/index.ts::installedIsMac`.
- false -- "comment trái luật trong `harvest_boundary.rs`": ba comment nêu bất biến không hiển nhiên (vì sao một tệp test được loại, vì sao chỉ cắt hàm của `export.rs`, vì sao đệm dòng).
- false -- "`ReviewerCopyPanel` dương tính giả" và "frontmatter chưa điền": tên giả định không tồn tại và nếu có thì cổng đỏ to; ghi chú cài đặt được điền khi trình bày.
