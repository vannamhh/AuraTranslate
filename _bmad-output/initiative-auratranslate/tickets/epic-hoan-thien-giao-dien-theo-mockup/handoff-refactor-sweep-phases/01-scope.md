# 12-22 Refactor sweep: scope thu thập (HEAD 1a212e1, 2026-10-19)

Nguồn: 21 plan (Review Triage Log / Implementation Notes / Design Notes), `deferred-work.md` L13108-13291 (45 mục `summary:`, 1 là Epic 9), `git log --grep='12-[0-9]'` (36 commit) quét `TODO|FIXME|XXX|HACK` trong `src`, `src-tauri/src`: 0 kết quả. Mọi plan 12.x trừ khung-app đang `built`.
Chủ trong sổ L13108-13291: Epic 12 ×33 · Winston ×5 · Ice ×1 · B7 ×1 · Story 12.18 ×1 (đã đóng 2026-10-16) · Story 12.22 ×1 · Epic 9 ×1. Chưa kiểm: các mục không có dòng `Chủ:` riêng (nếu có).

## A. Dọn / đo thuộc 12.22

- A1 Đo `export_preview` (nợ L13262, `Chủ: Story 12.22`; plan xuat-tuy-chon Notes #12 maybe-false). Code: `src-tauri/src/commands/export.rs:408` (core) + wire `:1209`, giữ `OpenWorkState`, đọc trọn Chương đầu (`build_preview`) mỗi lần đổi công tắc/định dạng. Đo Chương 2.000 câu; chỉ sửa nếu số đo đỏ (sửa có hai hướng thì thành B). Cỡ: S (đo) . Test: `cargo test --test export_content_options_contract` (tệp tồn tại; chưa kiểm tệp nào gọi `export_preview`) + bench một lần.
- A2 Mã chết sau khi 12-5 gỡ nút "gộp lên": `merge_chapter_into_previous` (`src-tauri/src/commands/chapter.rs:889`) và `merge_chapter_into_previous_indexed` (`:1320`) không còn chỗ gọi production (lệnh IPC + `chapter_merge_up` đã gỡ, grep `src/` và `lib.rs` rỗng; chỉ còn test). Plan danh-sach-chuong Notes #4 ("hàm thuần vẫn còn"). Gỡ cần chuyển 4 chỗ test sang `merge_chapters`: `src-tauri/tests/alignment_contract.rs:783,942,1506`, `src-tauri/tests/project_contract.rs:5123`; và sửa 5 doc-comment nhắc tên (`schema.rs:1134,1406,1423`, `lifecycle.rs:106`, `project/mod.rs:2846,2875`). Cỡ: M. Test: `alignment_contract`, `project_contract`, `chapter` liên quan. Lưu ý: `merge_into_previous_tx` (`:992`) vẫn dùng bởi `merge_chapters` (giữ).

## B. Cần Ice chọn

- B1 Biểu thức hiển thị phím lặp 11 chỗ `row.display.join(' / ')`: `GlossaryConfirmStrip.vue:127`, `ProofreaderStrip.vue:44`, `TitlebarToolsMenu.vue:14`, `StatusBar.vue:189,250`, `LibrarySearchResults.vue:33`, `LibraryMode.vue:120`, `ReadingMode.vue:369`, `panels/PanelTab.vue:63`, `panels/LookupPanel.vue:90`; đã có `config/shortcutsState.ts:634` nhưng nối bằng ` · ` (khác ` / `). Plan panel-tra-cuu đã `reject` "gom là thêm hàm dùng chung" (không phải defer). (1) Thêm một hàm `shortcutChordText(id)` nhận dấu nối rồi thay 10 chỗ: cỡ M, đụng 8 tệp webview, rủi ro đổi chuỗi hiển thị; (2) KHÔNG LÀM (thân một dòng, mỗi chỗ đọc `shortcutRows` riêng). Test nếu làm: vitest các tệp đó + `check:i18n`.
- B2 `resetAlignment` (`src/alignmentState.ts:216`) và `resetReviewerImport` (`src/reviewerImportState.ts:239`) không có chỗ gọi production (chỉ test: `reviewMode.test.ts:578-599`, `reviewerImport.test.ts:349`); plan review-mode Notes #10: "vẫn chưa có chỗ gọi khi đổi Tác phẩm, như trước story". Không có mục sổ. (1) Gọi chúng ở chỗ đổi Tác phẩm (`libraryImport.ts::finishImportSubmission`, `libraryChapters.ts::openWorkById`, như `resetGlossaryQueue`): đổi hành vi, cần ca canh, nhưng là sửa rò trạng thái; (2) Gỡ hàm chết. Cũng là D1.
- B3 Đo `refreshExportCounts` (nợ L13238, `Chủ: Epic 12`, plan tm-va-thu-vien Notes #6): `src/tmManageState.ts:280`, gọi 7 chỗ (`:123,266,520,556,594,630` + định nghĩa). Là phép đo như A1 nhưng sổ đang để Epic 12 (lượt dùng thật) và quần thể 200.000 cặp cần bench; Ice chọn: (1) 12.22 nhận đo bằng `nfr_bench`, đổi chủ bằng dòng `→ Chủ: Story 12.22`; (2) giữ Epic 12.

## C. Không phải sweep (giữ nguyên chủ), sổ `deferred-work.md`

- Lượt dùng thật `Chủ: Epic 12` ×33: L13111-13265 và L13266-13291, gồm khung-app (13112), library bốn cột (13120), mở thư mục macOS (13124), danh sách Chương 2.000 (13148), workspace a11y/vạch lề (13152 nghe VoiceOver "chưa dịch" là lỗi a11y có từ trước 12-7, 13156), dải gợi ý (13160 maybe-false focus `onStripFocusOut`, 13164), popup nguồn (13172), ba dải TM/Glossary/Proofreader (13200), panel AI (13204), Xem prompt (13213), TM/prompt (13234), glossary (13242, 13246), proofreader (13250), review mode (13254), xuất (13258), cài đặt/phím/kết nối (13266), lịch sử phiên bản (13271), hoàn tác chốt (13276), Attribution (13281, 13286, 13290).
- Tính năng/quyết định `Chủ: Winston` ×5: 13116 (`updated_at` Library), 13188 (chi phí ước tính trước quét), 13217 (hai loại "bị loại" trong Xem prompt), 13221 (tách prompt hệ thống/thể loại), 13230 (`{{chapter_context}}`). Các mục 13132, 13136, 13140, 13144, 13168, 13176, 13180, 13192, 13196 là khoảng cách tính năng so với mockup (kéo ⠿, đổi tên hàng loạt, ngày ghi đè, hoàn tác gộp, tên ngắn nguồn, phím `library.open_search_hit_reading`, đóng tìm kiếm Library, "Chương N · câu M" dải TM, lý do ở Chỗ đã bỏ qua): `Chủ: Epic 12`, quyết định thuộc Ice/Winston, không dọn.
- `Chủ: Ice` ×1: 13225 ("lắp bởi RagInjector"). `Chủ: B7` ×1 (Windows): 13128.
- Đã đóng: 13210 (`Chủ: Story 12.18`, ✅ 2026-10-16). 13108 là Epic 9 (ngoài phạm vi).

## D. Phát hiện review hoãn chưa có mục sổ / chưa có dạng đóng

Mọi dòng `defer` trong Triage Log đối chiếu được với sổ: attribution #29 → 13290, #24 → 13286, #26 → 13281; dai-goi-y maybe-false → 13160; workspace "chưa dịch" → 13152; xem-prompt → 13221; tm maybe-false → 13238; xuat maybe-false → 13262; review-mode e2e → 13254; proofreader e2e → 13250. Ngoài ra không có `defer` nào mồ côi. Còn lại:

- D1 `resetAlignment` / `resetReviewerImport` không ai gọi khi đổi Tác phẩm (review-mode Notes #10, "như trước story"): không có mục sổ, không `Chủ`. Cần dạng đóng hoặc chủ; xem B2.
- D2 khung-app Triage "medium · chờ Ice": `PLANNED_TICKET_FLOOR` 140 → 150. Hiện `scripts/check-debt-owner.mjs:742` là 160 nên coi như đã nâng; plan chưa ghi dạng đóng (cần 🔵 một dòng trong Triage Log, ngoài code). Không kiểm được lúc nào đổi.
- D3 Phát hiện `reject` kiểu "người dùng gần như không gặp" (khoảng 40 dòng, đã có lý do ngay trong plan) coi là đã đóng; không liệt kê lại. Ngoại lệ đáng ghi lại nếu Ice muốn đóng chữ nghĩa: attribution #31 (ca định vị nguồn nằm trong vitest, cần luật `check-layout` mới), tm "cổng `tm_boundary` bỏ lọt `from_work(None)`", review-mode "`read_alignment` lỗi sau commit", xuat "`is_unconfirmed` lặp SQL" (`core/export/scope.rs:17` và `:80`; có ca buộc hai chỗ khớp). Không có `Chủ`; lý do reject đủ làm dạng đóng.

## Ghi chú kiểm chứng

- Không chạy test nào. Số dòng lấy từ grep tại HEAD.
- Ghi chú: `.attr-scrim` còn 2 ca âm tính (`e2e/specs/attribution-focus.e2e.mjs:94`, `tests/frontend/attributionOverlayActions.test.ts:130`): cố ý, không dọn.
