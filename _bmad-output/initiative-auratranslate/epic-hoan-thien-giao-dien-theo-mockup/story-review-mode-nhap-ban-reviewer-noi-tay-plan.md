---
title: 'Review Mode, nhập bản reviewer, nối tay'
type: 'feature'
ticket: '16'
created: '2026-10-16'
status: 'built'
baseline_revision: '9dcd7fe621471ca9c2c63cf1ab0349879b2cf4ca'
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

**Problem:** Nhập bản reviewer dừng ở câu "Đã nhập n Chương (m hàng)", không nói bao nhiêu đã khớp và bao nhiêu cần nối tay; Nối tay là một danh sách "Chưa xử lý / Đã nhóm" mở tay từ `Công cụ ▾`; đóng màn nhập mở Bảng chờ thay vì màn thu hoạch; Review Mode không có nhãn đoạn, chú thích "chưa xác nhận" hay lối vào Nối tay; Nhập bản reviewer không có ở thẻ Tác phẩm (D11; dòng c:155, c:156, c:157, c:161, c:164).

**Approach:** Rust trả số khớp/chưa khớp theo Chương trong kết quả nhập. Webview nối thành một luồng: xác nhận nhập → Nối tay (hai cột theo mockup, đầu màn mang số khớp) khi còn mục chưa xử lý → màn thu hoạch của 12.13. Review Mode thêm nhãn "Đoạn N · …", chú thích chân và link Nối tay chỉ hiện khi còn mục chưa xử lý. Thẻ Tác phẩm thêm nút Nhập bản reviewer.

## Boundaries & Constraints

**Always:** Một định nghĩa "chưa xử lý" duy nhất: hàng/câu không thuộc nhóm nào, như `read_alignment` (`alignment.rs:608-618`); số trong kết quả nhập phải bằng số `AlignmentOverlay` hiện khi mở cùng Chương. Giữ nguyên phím Nối tay (↑↓ Space ↵ S U Esc) và mọi lệnh `alignment_*`. Giọng văn vô nhân xưng (UX-DR47, Kiểm D của `check:i18n`): cột trái là "Bản dịch của tôi", không "Bản của bạn"; "cần nối tay", không "cần bạn nối tay". Mọi nút mở mang `data-reviewer-import-open` / `data-alignment-open` để tiêu điểm trở về đúng nút (UX-DR17). Nhận thay đổi vẫn ghi `status = 'draft'` như AD-31; chú thích chỉ nói điều mã đã làm.

**Never:** Không đổi thuật toán khớp máy (`align_chapter`), lược đồ, hay cách nhận/bỏ (`review_accept_change`, `review_skip_change`). Không gợi ý ghép tự động cho hàng `?` (mockup vẽ một cặp gợi ý; app không có dữ liệu đó). Không thêm phím mới. Không sửa `GlossaryHarvestOverlay` ngoài việc mở nó.

## Decisions (Ice, 2026-10-16)

- Giữ cả plan (3.190 token), không tách.
- Số khớp đếm theo hàng của bản reviewer: y = số hàng, x = hàng nằm trong nhóm có cả hai phía; nhãn "Đã khớp x/y hàng · k mục cần nối tay", k = hàng + câu chưa xử lý, bằng số `AlignmentOverlay` hiện.
- Thu hoạch chạy lại khi Nối tay của luồng nhập đóng, trước khi mở màn thu hoạch: một lệnh Rust mới dùng lại `harvest_reviewer_copies`, để cặp nối tay có trong màn thu hoạch. Số thu hoạch quyết định có mở màn thu hoạch là số của lần chạy lại.
- Nhiều Chương lệch: Nối tay đi lần lượt từng Chương còn lệch theo `chapter_ord`, đầu màn ghi "Chương i/n"; đóng Chương cuối mới tới thu hoạch. Esc ở giữa chuỗi vẫn kết thúc chuỗi và đi tới thu hoạch.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Có lệch | Nhập bản có 1 hàng chưa khớp ở Chương 3 | Màn nhập chuyển thẳng sang Nối tay của Chương 3, đầu màn ghi tên tệp và số khớp; đóng Nối tay mở màn thu hoạch | — |
| Khớp hết, có thu hoạch | 0 mục chưa xử lý, `harvest_candidate_count > 0` | Mở thẳng màn thu hoạch | — |
| Khớp hết, không thu hoạch | 0 mục, số thu hoạch 0 | Ở lại màn "Đã nhập" với số khớp; không mở gì | — |
| Thu hoạch lỗi | `harvest_error` có giá trị | Nối tay vẫn mở nếu còn lệch; sau đó dừng ở màn "Đã nhập" có dòng lỗi thu hoạch, không mở màn thu hoạch | Hiện lỗi, không nuốt |
| Review Mode còn lệch | Chương mở có mục chưa xử lý | Link "Nối tay" hiện ở thanh Review; sau khi Nối tay đóng, Review Mode đọc lại diff, link mất khi hết lệch | Đọc lại lỗi → trạng thái `diff_failed` sẵn có |
| Review Mode khớp hết | 0 mục chưa xử lý | Không có link | — |
| Thẻ Tác phẩm khác | Bấm Nhập bản reviewer trên thẻ của Tác phẩm chưa mở | Mở Tác phẩm đó rồi mở hộp chọn tệp | Mở Tác phẩm lỗi → không mở hộp chọn tệp |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/export/reviewer_copy.rs` -- `confirm_import` :601 gọi `align_chapter` :660 trong giao dịch; `ImportSummary` :579. Thêm số theo Chương bằng cách đọc lại qua cùng hàm mà `read_alignment` (`alignment.rs:608`) dùng, không viết truy vấn đếm thứ hai.
- `src-tauri/src/commands/export.rs` -- `ReviewerImportSummaryWire` :382 (thêm mảng theo Chương), `reviewer_import_confirm` :589/:1063, `harvest_reviewer_copies` :557 (dùng lại cho lệnh làm mới). `alignment_*` :764–:865 không đổi.
- `src-tauri/src/core/export/harvest.rs:107` `harvest_work` đọc nhóm hai phía (`group_texts`), nên nhóm nối tay sinh thu hoạch nếu chạy lại; `enqueue_review_harvest` (`candidate_store.rs:416`) cập nhật/xoá theo cặp chữ, chạy lại an toàn.
- `src-tauri/src/lib.rs:1216-1222` -- đăng ký lệnh làm mới thu hoạch; `tests/ipc_contract.rs` (hình đóng băng), `tests/ipc_argument_contract.rs` (`REGISTERED_COMMAND_FLOOR`), `tests/reviewer_import_contract.rs`, `alignment_contract.rs`, `review_harvest_contract.rs`.
- `src/config/reviewerImport.ts` -- adapter + bộ gác kiểu cho wire mới. `src/reviewerImportState.ts` -- `afterImported` :114, `afterClosedWithHarvest` :121-129 (đang mở `openGlossaryQueue`, đổi sang `openGlossaryHarvest` ở `glossaryHarvestState.ts:43`); nối `src/main.ts:1222-1227`.
- `src/ReviewerImportOverlay.vue` -- màn `done` :103-127 (nút `export.alignment.open` :122).
- `src/AlignmentOverlay.vue` (+ `src/alignmentState.ts` `openAlignmentOverlay(chapterId)` :88, `src/alignmentCommandDeps.ts:29-31`) -- dựng lại bố cục hai cột; `pendingCount` :37 là nguồn số "n / k".
- `src/reviewModeState.ts` -- `unmatchedSegmentIds`/`unmatchedRowIds` :62-63 riêng tư; xuất một computed đếm. `src/layout/ReviewDock.vue:78-94` (thanh `review-bar`: link + chú thích). `src/panels/ReviewMinePanel.vue`, `ReviewCopyPanel.vue` -- nhãn đoạn; "có sửa" = có span khác `Equal` trong `review_diff`.
- `src/modes/LibraryMode.vue:552-656` -- thẻ Tác phẩm (khuôn: nút `library.open_work` :638); `openWorkById` ở `src/modes/libraryChapters.ts`. `src/modes/LibraryWorkPage.vue:496-504` -- nút thanh hành động, thêm `data-reviewer-import-open`.
- `src/i18n/vi.json` -- `reviewer.import.*` 524-546, `alignment.*` 547-569, `review.*` 1194-1228.
- Mockup `_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/review-mode.html` :86-133 (nhập + Nối tay), :149-203 (Review Mode). Đọc lại khi dựng.
- Test sẽ dời: `tests/frontend/reviewerImport.test.ts`, `tests/frontend/reviewerImportFocus.test.ts`, `tests/frontend/reviewerImportHooksWiring.test.ts`, `tests/frontend/alignment.test.ts`, `tests/frontend/reviewMode.test.ts`, `tests/frontend/libraryChapters.test.ts`, `tests/frontend/libraryWorks.test.ts`, `src-tauri/tests/config_invariants.rs`; `scripts/check-commands.mjs` `HANDLER_TABLE` nếu thêm handler. e2e mới: `e2e/specs/story-12-16-*.e2e.mjs`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/export/reviewer_copy.rs`, `src-tauri/src/commands/export.rs` -- kết quả nhập mang mỗi Chương: `chapter_id`, `chapter_ord`, số đã khớp, tổng, số mục chưa xử lý (Decisions) -- để webview biết đi Nối tay Chương nào.
- [ ] `src-tauri/src/commands/export.rs`, `src-tauri/src/lib.rs` -- lệnh làm mới thu hoạch trả cùng hình `harvest_candidate_count` / `harvest_error`; nâng `REGISTERED_COMMAND_FLOOR`, thêm hình vào `ipc_contract.rs`, một ca hợp đồng: nhóm nối tay sau nhập sinh ứng viên.
- [ ] `src/config/reviewerImport.ts`, `src/reviewerImportState.ts`, `src/main.ts` -- luồng nhập → Nối tay lần lượt từng Chương còn lệch → thu hoạch như ma trận.
- [ ] `src/AlignmentOverlay.vue`, `src/alignmentState.ts` -- hai cột "Bản dịch của tôi | Bản Reviewer đã sửa", hàng `→` (đã nhóm, mờ) và `?` (chưa xử lý, ô trống ghi "— không có đoạn tương ứng —"); nút "Nối … đoạn đã chọn" (nhãn theo số đang chọn), "Bỏ qua đoạn này", "Xem n chỗ còn lại" (tới mục chưa xử lý kế); dòng "i / k · ↑ ↓ chuyển · ↵ nối"; đầu màn ghi tên tệp + số khớp khi mở từ luồng nhập.
- [ ] `src/reviewModeState.ts`, `src/layout/ReviewDock.vue`, `src/panels/ReviewMinePanel.vue`, `src/panels/ReviewCopyPanel.vue` -- nhãn "Đoạn N · reviewer có sửa / không đổi / đã chấp nhận thay đổi / đã bỏ qua" (N = vị trí cặp trong `pairs`); chú thích chân; link Nối tay khi còn mục; đọc lại sau khi Nối tay đóng.
- [ ] `src/modes/LibraryMode.vue`, `src/modes/LibraryWorkPage.vue` -- nút trên thẻ Tác phẩm; `data-reviewer-import-open` trên cả hai nút.
- [ ] `src/i18n/vi.json` -- khoá mới, qua Kiểm D.
- [ ] Test: ca Rust cho số theo Chương (bằng số `alignment_open` trả); vitest cho từng hàng ma trận; một spec e2e cho luồng verify.

**Acceptance Criteria:**
- Given một bản reviewer có một hàng lệch, when xác nhận nhập, then màn Nối tay mở ngay, đầu màn ghi số khớp, và sau khi đóng nó màn thu hoạch mở.
- Given Review Mode mở trên Chương còn mục chưa xử lý, when nối hết rồi đóng Nối tay, then link Nối tay biến mất mà không mở lại Review Mode.
- Given đang ở Nối tay, when bấm ↑↓ Space ↵ S U, then mỗi phím làm đúng việc như trước story.
- Given mở Nhập bản reviewer từ thẻ Tác phẩm hoặc thanh hành động, when đóng màn nhập, then tiêu điểm về đúng nút đã bấm.

## Implementation Notes

- Số khớp theo Chương đọc lại qua `read_alignment` sau giao dịch nhập (`ChapterAlignment::match_counts`), cùng nguồn với `alignment_open`, không truy vấn đếm thứ hai; mảng `chapters` của kết quả nhập xếp theo `chapter_ord`.
- Lệnh mới `reviewer_harvest_refresh` dùng lại `harvest_reviewer_copies`; lỗi thu hoạch đi trong `harvest_error`, chỉ "không có Tác phẩm mở" là lỗi của lệnh. Khi thu hoạch lúc nhập đã lỗi, luồng không gọi làm mới.
- Đầu màn Nối tay mang số của chính Chương đang mở, không cộng dồn; màn "Đã nhập" ghi số "lúc nhập", không khẳng định phần còn lại đã nối.
- Nút đóng Nối tay giữa chuỗi đọc "Sang Chương kế"; Esc kết thúc chuỗi và đi thẳng tới làm mới thu hoạch. Nút Nối tay ở màn "Đã nhập" bị gỡ: lối vào còn ở `Công cụ ▾` và link Review Mode.
- `alignmentState.onAlignmentClosed` là chỗ nối duy nhất giữa Nối tay với luồng nhập và với Review Mode (đọc lại diff khi đóng trên Chương đang mở; đối chứng gỡ thật làm hai ca đọc lại đỏ).
- Nút trên thẻ Tác phẩm cần lệnh mới `library.open_work_reviewer_import` vì `check:commands` chỉ cho `@click` gọi `dispatch('<id>')`; mở Tác phẩm gỡ thẻ khỏi DOM nên tiêu điểm trở về nút Nhập bản reviewer của trang Tác phẩm. Nút chỉ hiện ở thẻ đang chọn, như `library.open_work`.
- Cặp thiếu một phía mang nhãn trần "Đoạn N"; nhãn "không đổi" chỉ cho cặp đủ hai phía không có span khác `equal` (đối chứng gỡ nhánh làm đúng ca mới đỏ).
- `CLICK_FLOOR` 214→228: HEAD có 266 `@click` và xanh, story thêm 2 nên cổng tự đỏ ở 268; 228 = ceil(0,85 × 268).
- `alignmentState.ts::closedListeners` là miễn trừ có tên của `check:panel-refs` (Ice chọn thay vì đổi sang ref sự kiện + `watch`): tập người nghe sống theo tiến trình, dọn nó cắt im lặng việc Review Mode đọc lại diff. `resetAlignment`/`resetReviewerImport` vẫn chưa có chỗ gọi khi đổi Tác phẩm, như trước story.
- e2e `story-12-16-*` chỉ mở Nối tay rỗng từ `Công cụ ▾`: hộp chọn tệp reviewer là hộp thoại gốc (AD-48); chuỗi nhập canh bằng vitest và `review_harvest_contract.rs`, lượt dùng thật là nợ `Chủ: Epic 12`.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 2 · low 4 · false 4 · maybe-false 0.
- medium · patch: đầu màn Nối tay của chuỗi mang số cộng mọi Chương, lệch số "Còn k mục" của chính Chương đó trên cùng màn; truyền số của Chương đang mở.
- low · patch: "Đã nhập" sau chuỗi khẳng định "phần còn lại đã qua bước nối tay" cả khi Esc giữa chuỗi; đổi thành số lúc nhập, không khẳng định.
- low · patch: cặp một phía (không hàng reviewer hoặc không câu) mang nhãn "không đổi"; dùng nhãn trần "Đoạn N".
- low · reject: `read_alignment` lỗi sau commit làm lệnh nhập báo lỗi dù đã ghi; chỉ xảy ra khi đọc lại hàng vừa ghi thất bại, lỗi báo to, sửa phải thêm trường lỗi vào wire.
- low · reject: bỏ nút Nối tay ở màn "Đã nhập"; lối vào còn ở `Công cụ ▾` và link Review Mode (D11), nút cũ mở Chương của Editor, không chắc là Chương vừa nhập.
- false: rò listener khi `resetAlignment` giữa chuỗi; `resetAlignment`/`resetReviewerImport` không có chỗ gọi nào trong `src/` ngoài định nghĩa.
- false: dòng phím trùng; `alignment.hint_keys` nói Space/S/U/Esc, dòng vị trí mới chỉ "i / k · ↑ ↓ · ↵", hai nội dung khác nhau.
- false: `CLICK_FLOOR` 214→228 nuốt trôi; chạy `check-commands` trên bản sao HEAD cho 266 `@click` (xanh, 214 ≥ 0,8 × 266), story thêm 2 thành 268 nên cổng tự đỏ, 228 = ceil(0,85 × 268).
- false: tiêu điểm đường thẻ Tác phẩm không về nút trên thẻ; ma trận đòi mở Tác phẩm trước, việc đó gỡ thẻ khỏi DOM, tiêu điểm về nút Nhập bản reviewer cùng chức năng của trang Tác phẩm.
- medium · defer: e2e không đi chuỗi nhập → Nối tay → thu hoạch vì hộp chọn tệp gốc (AD-48); mục `Chủ: Epic 12` trong `deferred-work.md`.

## Design Notes

- Chia pha theo AGENTS.md: Rust (số theo Chương, lệnh làm mới thu hoạch) · Webview (luồng nhập + Nối tay) · Webview (Review Mode + thẻ Tác phẩm) · Tests that move; mỗi pha một agent mới, bàn giao qua `handoff-review-mode-nhap-ban-reviewer-noi-tay-phases.md`.
- Nối tay hai cột dựng từ `ChapterAlignmentWire`: mỗi nhóm một hàng `→` (câu của tôi trái, hàng reviewer phải); mỗi câu chưa xử lý một hàng `?` với ô phải trống; mỗi hàng reviewer chưa xử lý một hàng `?` với ô trái trống. 🔵 2026-10-16: giữ thứ tự `alignmentEntries` có sẵn — mọi hàng `?` chưa xử lý trước, hàng `→` đã nhóm sau — để ↑↓ không đổi; không xen theo `segment.ord` như mockup (Ice chọn). Ô chọn vẫn là từng câu/hàng, không phải cả hàng lưới.

## Verification

**Commands:**
- `npm run test:story 12-16` -- expected: các ca của story xanh
- `cargo test --test reviewer_import_contract` và `--test ipc_contract` (sau `npm run build`) -- expected: xanh
- `npm run check:i18n` -- expected: Kiểm D xanh với khoá mới

**Manual checks (if no CLI):**
- Nhập một `.docx` thật có đoạn gộp: Nối tay mở ngay, rồi thu hoạch; Review Mode hiện nhãn đoạn và chú thích.
