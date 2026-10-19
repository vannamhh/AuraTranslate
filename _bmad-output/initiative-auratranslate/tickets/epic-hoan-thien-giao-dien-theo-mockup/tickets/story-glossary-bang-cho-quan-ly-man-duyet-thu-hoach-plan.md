---
title: 'Glossary: bảng chờ, quản lý, màn duyệt thu hoạch'
type: 'feature'
ticket: '13'
created: '2026-10-15'
status: 'built'
baseline_revision: '1b50f511ba7cc9c9ee10150ed95d62201b4d396f'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/glossary-queue.html'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/glossary-manage.html'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/review-mode.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Bảng chờ Glossary (`GlossaryQueueOverlay.vue`) là danh sách flex-wrap không có tiến độ, không thẻ nở, không phím `Space` "để lại"; Quản lý Glossary (`GlossaryManageOverlay.vue`) là `<ul role=listbox>` không đầu cột, không công tắc tầng, không hiện Xuất xứ / Phân loại / Thêm ngày, không vạch lề trạng thái; ứng viên thu hoạch từ bản reviewer (FR54, FR95) chỉ duyệt được lẫn trong bảng chờ, không có màn riêng (D13b). Dòng: c:14, c:15, c:18–c:20, c:23, c:34, c:36, c:37, c:39–c:42, c:44, c:45, c:48, c:50; C1, C3–C6.

**Approach:** Webview dựng lại hai lớp phủ theo `glossary-queue.html` và `glossary-manage.html`, và một lớp phủ mới "Duyệt thu hoạch thuật ngữ" theo `review-mode.html` §Thu hoạch thuật ngữ, mở từ mục mới trong `Công cụ ▾` (không phím, EXPERIENCE.md §Information Architecture). Dữ liệu đã có trên wire (`candidate_origin`, `proposed_translation`, `changed_count`, `seen_count`, `term_origin`, `tier`, `shadowed`); ghi chỉ qua `glossary_approve_candidate` / `glossary_reject_candidate` có sẵn.

**Đã chốt (nguồn):**
- Bộ lọc chip phân loại máy đoán, "Bỏ hàng loạt", nhiều ví dụ ngữ cảnh và phân loại do máy đoán của bảng chờ KHÔNG làm (phiếu quyết #56, Story 11.4; hàng loại 5 #2 không thuộc entry này). "Bộ lọc" của entry là bộ lọc Quản lý Glossary c:36–c:39.
- Hàng thu hoạch vẫn hiện trong bảng chờ (`gq-harvest`, lệch loại 4 có chủ đích); màn duyệt thu hoạch là chỗ duyệt thứ hai, không đổi AD-20.
- Giữ một plan dù 4.018 token o200k và ba bề mặt (Ice, 2026-10-15).
- "Nhận N mục đã chọn": webview lặp `glossary_approve_candidate` từng id theo khuôn 12.11; không lệnh Rust mới, không đổi wire. Lỗi giữa chừng giữ phần đã nhận, hàng lỗi hiện lỗi và giữ chọn để thử lại (Ice, 2026-10-15).
- 12.13 nhận lối mở nhìn thấy cho Bảng chờ Glossary ở Workspace (epic Notes, Done when 1), mang `data-glossary-queue-open` thật; chỗ đặt do agent pha webview đóng vai Sally chọn theo mockup và ghi lý do vào Implementation Notes (Ice, 2026-10-15).

## Boundaries & Constraints

**Always:**
- AD-20, AD-36: chỉ một hành động của người dùng đưa ứng viên vào Glossary; "Để lại trong bảng chờ" và `Space` không gọi lệnh ghi nào, `resolution` giữ `NULL`.
- Mọi số đếm nói rõ phạm vi nó đo; giá trị không biết thì ẩn, không hiện `0` (tổng Chương, nhịp khi chưa có quyết định nào, ngưỡng quét khi đọc cấu hình lỗi).
- Ngưỡng ở dòng phụ bảng chờ đọc `glossary_scan_threshold` thật từ cấu hình, không chép số 5.
- Vạch lề dùng đúng hình của 12.7 (độ dày + kiểu `border-left`, màu từ token), dưới tiền tố lớp riêng, không thêm tên `.rule-*`; trạng thái luôn có chữ (hiện hoặc sr-only), vạch `aria-hidden`.
- Lối mở mới giữ đường lui tiêu điểm `data-*-open` trên nút `Công cụ ▾` như 12.1; mỗi `@click` là đúng một `dispatch('<id>')`.

**Never:**
- Không đổi hình dạng wire của `glossary_pending_candidates`, `glossary_approve_candidate`, `glossary_reject_candidate`, `glossary_list_entries` (`ipc_contract.rs`), không đổi giao dịch `approve_candidate`.
- Không gắn phím tắt cho Duyệt thu hoạch; không nối màn này vào luồng nhập bản reviewer (Story 12.16).
- Không phím `Space` toàn cục: chỉ handler cục bộ trên gốc lớp phủ bảng chờ (src/AGENTS.md, `isTypingZone`).
- Không thêm migration, không thêm lệnh Rust.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Duyệt thu hoạch, nhận một phần | 3 ứng viên `review_harvest` đang chờ; chọn 2, bấm "Nhận 2 mục đã chọn" | 2 mục vào Glossary Tác phẩm với `term_origin = 'review_harvest'`, bản dịch = `proposed_translation`; bảng chờ còn 1 | Một lần nhận lỗi ⇒ hàng đó hiện lỗi, vẫn được chọn, các hàng đã nhận rời danh sách |
| Để lại | 3 ứng viên, bấm "Để lại trong bảng chờ" | Lớp phủ đóng, không lệnh ghi nào được gọi, bảng chờ còn 3 | — |
| Chọn sẵn | Hàng nhất quán (`changed*2 > seen`) và hàng không nhất quán | Hàng nhất quán được chọn sẵn; hàng không nhất quán không chọn sẵn, mang chữ "không nhất quán" | — |
| Không có thu hoạch | Không ứng viên `review_harvest` nào chờ | Trạng thái rỗng nói chưa có thu hoạch nào chờ duyệt | Đọc lỗi ⇒ trạng thái lỗi, không phải rỗng |
| Bảng chờ `Space` | Con trỏ ở hàng i | Hàng i đánh dấu "để lại" trong phiên, con trỏ sang hàng kế, không ghi | — |
| Mở lại bảng chờ | Đóng bằng Esc ở hàng ứng viên X, X còn chờ | Mở lại thì con trỏ ở X | X đã được quyết ⇒ hàng đầu tiên còn chờ |
| Quản lý, tầng "Cả hai" | Mục Toàn cục bị mục Tác phẩm cùng thuật ngữ che | Hàng Toàn cục thụt vào ngay dưới hàng Tác phẩm che nó | — |

</frozen-after-approval>

## Code Map

- `src/GlossaryQueueOverlay.vue` (606 dòng) -- bảng chờ: `onKeydown` ~150–197 (phím 1–4, ↑↓, N, B; thêm `Space`), hàng `gq-row` flex ~420; `gq-harvest`/`gq-inconsistent` 258–272 giữ; `trapTab` và `focusReturnTargetOnOpen('[data-glossary-queue-open]')` 69/84 giữ. Khung lớp phủ để chép cho màn thu hoạch.
- `src/glossaryQueueState.ts` (366) -- `queueRows`, `queueCursor`, `queueUnprocessedCount` (112), `openGlossaryQueue` (162), nhận (268), bỏ (313), `isInconsistentHarvest` (362); khuôn chặn đua bằng số thứ tự — dùng lại. Thêm: số đã nhận / đã bỏ / để lại trong phiên, mốc giờ từng quyết định (C1), id con trỏ nhớ qua lần mở. Review 12.7: nhận ứng viên không làm mới số đếm Glossary — sửa cùng lúc.
- `src/GlossaryManageOverlay.vue` (1017) + `src/glossaryManageState.ts` (672) -- thanh lọc 381–432 (`<select>` phân loại / xuất xứ / trạng thái, ô sắp), danh sách 485–528 thành `<table>`; `ORIGIN_OPTIONS` 240; form sửa, xoá hai bước, nâng lên Toàn cục, khối xuất/nhập giữ hành vi.
- `src/GlossaryConfirmStrip.vue:105` -- ánh xạ `term_origin` → `glossary.manage.origin_*`; dùng lại cho cột Xuất xứ.
- `src/config/glossary.ts` -- `GlossaryCandidate` 437–461, `pendingGlossaryCandidates` 520, `approveGlossaryCandidate` 659, `rejectGlossaryCandidate` 707, `listGlossaryEntries` 812 (trả mọi hàng, đủ để đếm theo tầng ở webview), `GlossaryTierWire` 20. Adapter không ném lỗi.
- `src/TitlebarToolsMenu.vue` (235) -- mục `tm.manage.open` là khuôn; nút kích hoạt mang mọi `data-*-open`, thêm `data-glossary-harvest-open`. Thứ tự mục theo IA: Duyệt thu hoạch thuật ngữ đứng sau Nối tay, trước Xuất.
- `src/commands/index.ts` -- `glossary.queue.open` 4172, `tm.manage.open` 4008 là khuôn đăng ký; cổng deps ~1037. `src/main.ts` ~1040–1060 `isBlocked`: thêm ref mở của lớp phủ mới. `src/tmManageCommandDeps.ts` là khuôn deps.
- `src/panels/GridPanel.vue` ~2249–2330, `src/panels/editorSegments.ts` -- hình vạch 12.7: `confirmed` 2px solid, `primary` 3px solid, `tm-rule` 2px dotted, `draft` 2px dashed. Không sửa.
- `scripts/check-commands.mjs` -- `HANDLER_TABLE` cần hàng cho mỗi `@keydown`/`@mousedown` mới; Kiểm I chỉ đọc `.rule-*`; sàn CLICK/DISPATCH/COMMAND có thể phải nâng.
- `src/i18n/vi.json` -- khoá phẳng; có sẵn `glossary.manage.origin_review_harvest`, `glossary.queue.harvest_change`, `glossary.queue.harvest_inconsistent`. Chữ dữ liệu cần `<!-- aura-allow-text: … -->`.
- `src/StatusBar.vue`, `src/workspaceCountsState.ts` -- thanh trạng thái Workspace, đã có `status.glossary_count`; một ứng viên cho chỗ đặt lối mở Bảng chờ. Hiện `data-glossary-queue-open` chỉ có ở mã đường lui và `tests/frontend/reviewerImportFocus.test.ts:74`, chưa có nút thật.
- `src-tauri/src/core/glossary/candidate_store.rs` -- `approve_candidate` 322 (một giao dịch; `source_term` đã có thì UPDATE bản dịch + `term_origin`), `reject_candidate` 529. Không sửa.
- `src-tauri/src/commands/config.rs:103` -- `glossary_scan_threshold` có trong cấu hình gửi webview.

## Tasks & Acceptance

**Execution:**
- [ ] `src/glossaryQueueState.ts` -- đếm trong phiên đã nhận / đã bỏ / để lại / còn lại, mốc giờ quyết định, hàm `leave` không ghi, nhớ id con trỏ, làm mới số Glossary sau khi nhận -- c:15, c:23, C1, C3.
- [ ] `src/GlossaryQueueOverlay.vue` -- tiêu đề + dòng phụ ngưỡng (c:14), thanh tiến độ + chú giải + dòng nhịp (c:15, C1), hàng: thuật ngữ · phân loại phiên · số lần · nhìn trước (c:18), thẻ nở của hàng con trỏ: thuật ngữ lớn, "xuất hiện N lần · trong K Chương", vị trí "i / n", chip phân loại có chữ số (c:20), một ngữ cảnh, khối FR114 khi không có bản dịch đề xuất (c:19), dải phím N · B · Space · ↑↓ và câu Esc (c:23, C3), chân trang hai câu (C4).
- [ ] `src/GlossaryManageOverlay.vue`, `src/glossaryManageState.ts` -- công tắc tầng có số Toàn cục · Tác phẩm · Cả hai (c:34), pill phân loại (c:36), pill "Chờ chốt · N" (c:37), chữ "Sắp theo" (c:39), `<table>` bảy cột Thuật ngữ nguồn · Bản dịch · Phân loại · Xuất xứ · Dùng · Ghi chú · Thêm ngày (c:40–c:42), vạch lề trạng thái (c:44), hàng bị che thụt dưới hàng che (c:45), nút "Thêm thuật ngữ" ở chân mở thêm nhanh có sẵn (c:48), nhấp đúp sửa tại chỗ và `⌫` vào bước một của xoá hai bước (c:50), hộp giải thích chờ chốt (C5), khối "Đọc được gì trong file" ở phần nhập (C6).
- [ ] `src/GlossaryHarvestOverlay.vue`, `src/glossaryHarvestState.ts` (mới) -- lọc `candidate_origin === 'review_harvest'` từ `pendingGlossaryCandidates()`; hàng "X → Y — đổi ở n / m lần", ô chọn, chọn sẵn theo `isInconsistentHarvest`, nút từng hàng Nhận / Bỏ; "Nhận N mục đã chọn", "Để lại trong bảng chờ", câu gợi ý tỉ lệ; bẫy tiêu điểm và đường lui như bảng chờ.
- [ ] `src/commands/index.ts`, `src/main.ts`, `src/TitlebarToolsMenu.vue`, `src/i18n/vi.json`, `scripts/check-commands.mjs` -- lệnh `glossary.harvest.open` không phím, mục menu, `data-glossary-harvest-open`, `isBlocked`, khoá i18n, hàng `HANDLER_TABLE`.
- [ ] Workspace (chỗ do pha webview chọn, ví dụ `src/StatusBar.vue`) -- lối mở nhìn thấy cho Bảng chờ, `@click="dispatch('glossary.queue.open')"`, mang `data-glossary-queue-open`; vẫn hiện khi bảng chờ rỗng; số ứng viên nếu có thì ẩn khi chưa đọc được.
- [ ] Tests that move: `tests/frontend/glossaryQueue.test.ts`, `tests/frontend/glossaryManage.test.ts`, `tests/frontend/glossaryHarvest.test.ts` (mới), `tests/frontend/titlebarToolsMenu.test.ts` (đếm mục), `tests/frontend/glossaryQueueOpener.test.ts` + `tests/frontend/glossaryQueueResetWiring.test.ts` (mới), `review_harvest_contract.rs` (ca Rust: ba ứng viên thu hoạch, nhận hai qua `approve_candidate` từng id, `pending_candidates` còn đúng một), e2e `e2e/specs/story-12-13-glossary-harvest.e2e.mjs`.

**Acceptance Criteria:**
- Given ba ứng viên thu hoạch đang chờ, when mở Công cụ ▾ → Duyệt thu hoạch thuật ngữ, then màn liệt kê đúng ba hàng; when nhận hai, then bảng chờ còn một; when bấm "Để lại trong bảng chờ", then không lệnh ghi nào chạy và tiêu điểm về `Công cụ ▾`.
- Given Quản lý Glossary có mục cả hai tầng, when chọn từng nấc công tắc tầng, then số trên mỗi nấc bằng số hàng bảng hiện ở nấc đó, và mỗi hàng hiện Xuất xứ bằng chữ.
- Given hàng chờ chốt và hàng đã chốt Tác phẩm, then vạch lề của chúng khác nhau ở `border-left-style`, và trạng thái đọc được bằng chữ.
- Given bảng chờ đang mở, when nhấn `Space`, then con trỏ sang hàng kế và không lệnh ghi nào chạy.
- Given Workspace đang mở, when bấm lối mở Bảng chờ rồi đóng bằng Esc, then bảng chờ mở và tiêu điểm về đúng nút đã mở nó.

## Implementation Notes

- Lối mở Bảng chờ (vai Sally): nút `data-glossary-queue-open` ở đầu phải `StatusBar.vue`, cạnh số Glossary mà nó mô tả, ngoài nhánh `!lookupHasRetreated` nên còn với thanh hẹp; luôn hiện vì bảng chờ tự nói khi rỗng. Không mang số ứng viên: wire không có số đếm rẻ và `0` sẽ là lời nói dối.
- Nhận lô ở màn thu hoạch là vòng tuần tự `approveGlossaryCandidate` từng id (không lệnh Rust mới); lỗi một hàng giữ ô chọn và hàng đó, vòng vẫn đi tiếp. Bốn đối chứng gỡ thật (chọn sẵn, dừng khi lỗi, song song, vòng tuần tự) đều đỏ đúng ca.
- Con trỏ nhớ qua lần mở của bảng chờ khoá theo id ứng viên, id là theo kho Tác phẩm nên `resetGlossaryQueue()` nay được gọi ở cả hai chỗ đổi Tác phẩm: `libraryImport.ts::finishImportSubmission` (nhập mới) và `libraryChapters.ts::openWorkById` (mở Tác phẩm có sẵn), mỗi chỗ một ca trong `glossaryQueueResetWiring.test.ts`; trước đó hàm không có chỗ gọi.
- Số trên công tắc tầng Quản lý tính theo mọi bộ lọc khác trừ tầng, nên bằng số hàng bảng ở nấc đó; mặc định tầng là "Cả hai" (giữ thứ tự backend), không phải Tác phẩm như mockup.
- Kiểu `GlossaryManageConfirmedFilter` chỉ còn `'all' | 'pending'`: giá trị `'confirmed'` không còn đường nào từ UI sau khi pill thay `<select>`.
- Vạch lề: chờ chốt = 2px dashed (token cảnh báo), đã chốt Tác phẩm = 2px solid, đã chốt Toàn cục = không vạch; mỗi hàng luôn có chữ trạng thái. Kiểu nét chỉ canh qua tên lớp trong happy-dom; độ phân biệt sáng/tối thành nợ Epic 12.
- `COMPONENT_FILE_FLOOR` trong `scripts/check-tokens.mjs` nâng 152 lên 164 vì hai tệp `src` mới của story (`GlossaryHarvestOverlay.vue`, `glossaryHarvestState.ts`); HEAD có 190 tệp và xanh, nên đây là hệ quả của story chứ không phải nợ có sẵn.
- Không đưa dòng đọc Hán Việt dưới thuật ngữ và "reviewer đổi ở n/m lần" vào bảng Quản lý: không có trên wire. Cột "Dùng" giữ số trần, không `N×`.
- e2e `story-12-13-*` không dựng bản reviewer nên chỉ mở màn thu hoạch rỗng; luồng ba ứng viên nhận hai canh bằng `review_harvest_contract.rs` và `glossaryHarvest.test.ts`. Hai mục nợ `Chủ: Epic 12` ghi chỗ chưa chạy được.
- Chạm `scripts/check-*` (HANDLER_TABLE, sàn) thuộc điều kiện chạy cả bộ vitest tay: 192 tệp, 2734 ca xanh.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens quick, 2026-10-15): high 0 · medium 5 · low 5 · false 2 · maybe-false 0 · loại 1 (sửa plan).
- medium · patch — Màn thu hoạch khi chưa mở Tác phẩm báo "chưa có thu hoạch" (rỗng im lặng); bảng chờ đã có probe `lookupGlossaryTerm('')` mà màn này thiếu.
- medium · patch — Bấm Nhận/Bỏ từng hàng: nút bị `disabled` rồi gỡ khỏi DOM khi đang giữ tiêu điểm, tiêu điểm rơi về `body` ngoài scrim nên Esc/Tab/N/B chết.
- medium · patch — Sau N/B con trỏ thu hoạch đứng yên trên hàng đã quyết, lần N/B kế tiếp lặng lẽ không làm gì.
- medium · patch — `.gm-cell-term { display: flex }` trên `<td>` đưa ô ra khỏi bố cục bảng; vạch lề và padding áp lên con của ô vô danh.
- medium · patch — `setGlossaryManageCursor` (mousedown hàng khác) bỏ bản sửa tại chỗ đang gõ mà không hỏi; các setter lọc đã chặn bằng `isEditingBlocked()`.
- low · patch — `leaveGlossaryQueueCandidate` thiếu chặn `saving`: Space giữa lượt ghi rồi ghi trượt thì hàng hiện "Để lại" dù người dùng không chọn.
- low · patch — Doc comment ở `glossaryManageState.ts` lệch symbol (bộ lọc phân loại, xuất xứ) và mất doc `manageCurrentRow`.
- low · patch — Chú thích mới trái AGENTS.md §Code comments: khối HTML lý do đặt nút ở `StatusBar.vue`, đầu tệp thu hoạch chép lại plan, doc `resetGlossaryQueue` kể tên chỗ gọi.
- low · patch — Năm khoá `glossary.manage.confirmed_filter_*`/`category_filter_all` không còn ai dùng (grep `src tests scripts`).
- low · patch — Ca Rust không khẳng định bản dịch = `proposed_translation`; `onRowEscape` không có test; nhánh Space và `onRowEscape` chưa đối chứng gỡ.
- false — Phút ước tính lệch "Còn lại": dòng nhịp không hiện số đếm nào, nó ước cho mọi hàng chưa quyết kể cả hàng để lại (vẫn chờ trong kho), đúng phạm vi.
- false — `resetGlossaryHarvest` không có chỗ gọi: `openGlossaryHarvest` dựng lại toàn bộ state mỗi lần mở và lớp phủ là modal nên Tác phẩm không đổi lúc nó mở; không có kết quả sai.
- loại — Checkbox Tasks chưa tick / Triage Log trống: cách sửa là sửa plan của chính build này.

## Design Notes

- Hình vạch Quản lý Glossary (c:44, theo mockup `tr.g` và `.wait`): chờ chốt = hình `draft` (2px dashed, màu token cảnh báo); đã chốt tầng Tác phẩm = hình `confirmed` (2px solid); đã chốt Toàn cục = không vạch. Tầng còn có chữ ở hàng, nên màu không là kênh duy nhất.
- Chia pha theo AGENTS.md: Webview (bảng chờ + thu hoạch + menu) · Webview (Quản lý) · Tests that move; mỗi pha một agent mới, bàn giao qua `handoff-glossary-bang-cho-quan-ly-man-duyet-thu-hoach-phases.md`. Không có pha Rust (nhận lô lặp ở webview).
- Nhận lô tuần tự, không song song: mỗi lần nhận đọc lại kết quả trước khi gọi id kế, để lỗi `Conflict` (đã quyết ở nơi khác) chỉ đánh dấu đúng hàng đó.

## Verification

**Commands:**
- `npm run test:story 12-13 -- --list` rồi `npm run test:story 12-13` -- expected: xanh, không THIẾU KHAI.
- `npm run check:commands && npm run check:tokens && npm run check:i18n && npm run check:panel-refs` -- expected: xanh.
- `npm run build && cargo test --manifest-path src-tauri/Cargo.toml --test review_harvest_contract --test glossary_commands_contract` -- expected: xanh.

**Manual checks (if no CLI):**
- e2e `story-12-13-glossary-harvest` không chạy được trên máy Ice (webview đứng ở `tauri://localhost`): đọc lượt nightly; bàn phím trong menu và độ phân biệt vạch sáng/tối thành mục nợ `Chủ: Epic 12`.
