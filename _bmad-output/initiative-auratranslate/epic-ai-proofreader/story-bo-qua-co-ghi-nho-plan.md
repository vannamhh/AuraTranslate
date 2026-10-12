---
title: 'Bỏ qua có ghi nhớ'
type: 'feature'
ticket: '6'
created: '2026-10-12'
status: done
baseline_revision: '3e5a728e4e1d6bb17baaae8480cadd2f7b823d9b'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Phát hiện đã bỏ qua ở 9.4 chỉ sống trong bộ nhớ webview, quét lại là báo lại; proofreader lặp cùng một cảnh báo sai thì người dùng tắt hẳn nó (FR84), và 9.8 chưa có mẫu số để tính tỷ lệ báo động giả.

**Approach:** Lệnh "không phải lỗi" ghi `PROOF_IGNORE` trong `project.db` theo chữ ký AD-53 mục 7 (không `segment.id`), bước lọc thuần chạy trước khi kết quả quét ra dây, có lớp phủ danh sách để xem lại và bỏ ghi nhớ, và mỗi lượt quét xong ghi một hàng số đếm.

**Decisions (Ice, 2026-10-12):** giữ trọn plan, không tách. Danh sách là lớp phủ riêng mở từ nút thanh tiêu đề mới, liệt kê ghi nhớ của Tác phẩm đang mở, mỗi dòng bỏ ghi nhớ được. Lượt quét giữ chỉ đọc; webview gọi một lệnh ghi riêng sau `Done`: một hàng mỗi lượt quét xong (phạm vi, thời điểm, số báo, số bị lọc), lượt huỷ hoặc lỗi không ghi gì, Rust kiểm điều kiện kiểm được (Tác phẩm còn mở, phạm vi thuộc tập đóng, số không âm) vì số do webview khai. Bỏ ghi nhớ xoá hàng. Từ mockup chỉ lấy "Xoá toàn bộ ghi nhớ của Tác phẩm" (xoá hàng loạt, bấm hai lần kèm chữ "không hoàn tác được" theo AD-49 (iii)); không cột "đã chặn N lần", không "Khôi phục". N = 32 ký tự Unicode (Ice xác nhận ở checkpoint).

## Boundaries & Constraints

**Always:** AD-53 mục 4 (một hàm kiểm `check_proofread_range`) và mục 7: "bỏ qua" (9.4, bộ nhớ, 0 lời gọi Rust) và "không phải lỗi" (9.6, bền) là hai thao tác; chữ ký và phép khớp là MỘT hàm thuần trong `core/ai/proofread.rs`, dùng chung cho lệnh ghi và bước lọc; chữ ký chứa `(loại, cụm[, ngữ cảnh trước, ngữ cảnh sau])` văn bản thuần, không băm, không `segment.id`, vị trí, giải thích, đề xuất; phạm vi là Tác phẩm vì `project.db` là kho của đúng một Tác phẩm; N chỉ được tăng về sau; phát hiện bị lọc luôn được đếm (`filtered`), không im lặng như "0 phát hiện"; migration mới, forward-only.

**Never:** khoá theo `segment.id`; chạm `write_non_user_target` hay `proofread_write_boundary`; ghi `target_text`, trạng thái hay xuất xứ; để lượt quét ghi `project.db`; lan ghi nhớ sang Tác phẩm khác; AD mới; làm "bỏ qua" của 9.4 thành bền; hiển thị tỷ lệ báo động giả hay ngưỡng (9.8).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error |
|---|---|---|---|
| Đánh dấu | phát hiện còn đúng văn bản quét | một hàng `PROOF_IGNORE`, phát hiện biến khỏi dải và gạch chân | — |
| Quét lại | chữ ký khớp | không ra dây, `filtered` tăng | — |
| Gộp segment | cụm có ngữ cảnh hai phía không rỗng | vẫn khớp, vẫn không báo | — |
| Cụm sát biên, rồi gộp | ngữ cảnh một phía rỗng | báo lại (AD-53 mục 7, ⚠️ đã nhận) | — |
| Tác phẩm khác | cùng cụm | vẫn báo | — |
| Văn bản đã đổi | `target_text` hay cụm lệch | không ghi | `ai_proofread.text_changed` |
| Cụm rỗng sau `trim` | khoảng toàn khoảng trắng | không ghi | `ai_proofread.range_invalid` |
| Bỏ một mục / xoá toàn bộ | hàng có thật | hàng bị xoá, quét sau báo lại; xoá toàn bộ chỉ chạy ở lần bấm thứ hai | — |
| Ghi số lượt quét | `Done`, Tác phẩm còn mở | một hàng | Tác phẩm đã đổi hoặc phạm vi lạ: từ chối, không ghi |
| Lượt huỷ hoặc lỗi | không có `Done` | không gọi lệnh ghi | — |

</frozen-after-approval>

## Code Map

- `core/ai/proofread.rs`: `ProofreadFinding`, `locate_batch_findings`, `utf16_len`; thêm hằng `N = 32`, hàm chữ ký + khớp, hàm lọc (thuần). `core/segment/proofread_kind.rs`: `SignatureScope` đã sẵn, không sửa.
- `commands/proofread.rs`: `prepare_proofread_call` (đọc ghi nhớ bằng `open.store.read` trong bước chuẩn bị), `PreparedProofreadRun`, `run_proofread_call` (lọc sau `locate_batch_findings`, trước `Channel::send`), `AiProofreadOutcomeWire::Done { not_scanned }` (AD-53 mục 5: `Done` thêm `filtered`; 9.4 để `ambiguous` theo segment), vỏ `wire`. Chỉ `Done` mang `filtered` (lượt huỷ không ghi hàng). Tiêu đề tệp "Read-only" vẫn đúng cho đường quét; các lệnh ghi mới nằm ở `wire` riêng của tệp, ghi rõ.
- Lệnh mới (hai tầng: hàm thuần `Option<&OpenWork>` + vỏ `try_state`): `ai_proofread_ignore_add` (qua `check_proofread_range` của `commands/segment/proofread_accept.rs`, không kiểm thứ hai), `ai_proofread_ignore_list`, `ai_proofread_ignore_remove`, `ai_proofread_ignore_clear`, `ai_proofread_scan_record`. Đăng ký cạnh `lib.rs:1296`.
- `core/store/schema.rs`: `PROJECT_MIGRATIONS` kết ở 33 (`REVIEW_DECISION_DDL`, khuôn gần nhất); thêm bước 34 tạo hai bảng: `proof_ignore` (loại, cụm, ngữ cảnh trước/sau, `created_at`; `UNIQUE` bốn cột chữ ký; `CHECK` loại ∈ năm giá trị; không cột segment, không cột work vì kho một Tác phẩm) và `proof_scan` (phạm vi `CHECK` ∈ segment/selection/chapter, `scanned_at`, `reported`, `filtered`). Thời điểm do Rust đóng dấu theo `review_decision.rs`. `indexer.rs:1695` không phải nâng (không đọc hai bảng này).
- Tệp đổi theo số bước 33→34 (khuôn e8e4561): `tests/chapter_origin_contract.rs:891`, `segment_role_contract.rs:598`, `tm_contract.rs:687,693,719,725`, `review_harvest_contract.rs:476-484`, `segment_contract.rs:2172`, `pinned_contract.rs:258-265`, `project_contract.rs` `NON_ENTITY_DETAIL_TABLES` (`[&str; 18]`→20, kèm lý do từng bảng).
- `tests/ipc_contract.rs:2233` (khoá tên tham số cho các lệnh mới), `tests/config_invariants.rs:1766` hàng census `commands/proofread.rs` (2,0,0 thêm 5 lệnh plain) và tổng toàn cây, thông điệp assert khớp số. `core/i18n/mod.rs:864` + `vi.json:95`: dùng lại `range_invalid`/`text_changed`; thêm khoá lỗi chỉ cho `scan_record` bị từ chối nếu cần.
- `tests/ai_boundary.rs:509-520` `ALLOWED_AI_PATHS_IN_PROOFREAD_SEAM` (7 đường, mảng cỡ cố định): thêm đường hàm chữ ký và hàm lọc, đổi cỡ mảng. `tests/proofread_write_boundary.rs`: hai vế cấm giữ nguyên, lệnh mới không được chạm. `tests/ai_proofread_contract.rs:467-517` `full_snapshot` mở rộng sang mọi bảng: quét không đổi bảng nào (kể cả `proof_*`).
- Webview: `config/proofread.ts` (adapter không ném, mỗi `invoke` một literal riêng; `isProofreadOutcomeWire` đọc thêm `filtered`; kiểu dây của danh sách), `proofreadState.ts` (`dismissProofreadFinding` :477 là bỏ qua bộ nhớ; `acceptProofreadFinding` :371 là khuôn gọi Rust + `acting`/`actionFailure`; sau `Done` gọi `scan_record` với số báo/lọc/phạm vi), `proofreadHandlers.ts`, `commands/index.ts:4164-4280`, `ProofreaderStrip.vue:167-171` (thêm nút), `App.vue:269` (khuôn nút thanh tiêu đề `data-proofread-scope-open`, thêm `data-proofread-memory-open`; gắn lớp phủ ở :466), `main.ts:958,1007` (thêm cờ mở vào tập chặn hợp âm).
- Lớp phủ mới: khuôn `ProofreadScopeOverlay.vue` (`focusReturnTargetOnOpen`, `trapTab`, Esc đóng, `aria-modal`) cộng khuôn danh sách `TmManageOverlay.vue`/`tmManageState.ts` (`tmManageBulkPending` = bấm hai lần hàng loạt với gợi ý đếm và chữ "không hoàn tác được", khoá `tm.manage.bulk_confirm_hint_work` :1220). Đó là tiền lệ xoá hàng loạt của repo; nó hai lần bấm ở webview, Rust xoá không cờ.
- Cổng: `check:commands` (`COMMAND_FLOOR 200`, `DISPATCH_FLOOR 220`; đo 235 command, 258 dispatch ⇒ `ceil(0.85×live)` sát sàn; thêm ~6 command và ~6 `dispatch` literal có thể buộc nâng, cần Ice duyệt như 9.4), `check:i18n`, `check:tokens`, `check:panel-refs`, `check:layout` (hợp âm/lớp phủ). `deferred-work.md`: không có mục `Chủ: Story 9.6`; "loại do model gán đổi khoá" thuộc 9.8.
- Tests that move: Rust `proofread_memory_contract` (mới) · `ai_proofread_contract` · `ai_boundary` · `ipc_contract` · `config_invariants` · `project_contract` · `proofread_write_boundary` · `chapter_origin_contract` · `segment_role_contract` · `tm_contract` · `review_harvest_contract` · `segment_contract` · `pinned_contract`; frontend `tests/frontend/proofreadMemory.test.ts` (mới) · `tests/frontend/proofreadConfig.test.ts` · `tests/frontend/proofreadState.test.ts` · `tests/frontend/proofreadActions.test.ts` · `tests/frontend/proofreadWiring.test.ts` · `tests/frontend/proofreadScopeOverlay.test.ts` · `tests/frontend/proofreaderStrip.test.ts` · `tests/frontend/proofreaderStripSlots.test.ts` · `tests/frontend/segmentSelection.test.ts` · `tests/frontend/aiTranslateBatchResetWiring.test.ts` · `tests/frontend/libraryImportResetsAiPromptInspector.test.ts` · `tests/frontend/libraryImportResetsSegmentHistory.test.ts`.

## Tasks & Acceptance

**Execution:**
- [ ] `core/ai/proofread.rs` -- `N = 32`, `signature_of(kind, text, byte_start, byte_end)`, `matches`, `filter_remembered` thuần; cụm trim + NFC, ngữ cảnh NFC; luật khớp phía rỗng/đuôi/đầu theo AD-53 mục 7
- [ ] `core/store/schema.rs` -- bước 34: `proof_ignore`, `proof_scan`
- [ ] `commands/proofread.rs`, `lib.rs` -- năm lệnh trên; `prepare_proofread_call` đọc ghi nhớ; `run_proofread_call` lọc, đếm `filtered` vào `Done`; lượt quét không ghi
- [ ] Rust tests -- `tests/proofread_memory_contract.rs` mới: sống sót qua `merge_segments`/`split_segment` thật, chính tả khoá theo cụm, ngữ pháp theo ngữ cảnh, Tác phẩm thứ hai vẫn báo, bỏ và xoá toàn bộ báo lại, `UNIQUE`, schema không cột `segment_id`, mọi lỗi mục 4 không ghi gì, `scan_record` từ chối Tác phẩm đổi/phạm vi lạ, quét không đổi bảng nào; sửa các tệp số bước; `ai_boundary`, `ipc_contract`, `config_invariants` khớp số
- [ ] `config/proofread.ts`, `proofreadState.ts`, `proofreadHandlers.ts`, `commands/index.ts`, `ProofreaderStrip.vue`, `vi.json` -- nút/command "Không phải lỗi" (phím để trống), phát hiện biến ngay; `filtered` trên dải tóm tắt; gọi `scan_record` sau `Done`
- [ ] Lớp phủ mới + `App.vue` + `main.ts` -- danh sách, bỏ từng mục, xoá toàn bộ hai lần bấm, nút thanh tiêu đề, chặn hợp âm khi mở; command `ai.proofread.memory.{open,close,forget,clear}`
- [ ] Webview tests -- adapter (hình dây lạ bị từ chối), trạng thái, dải, lớp phủ (hai lần bấm, tiêu điểm trả về); counter-check theo AGENTS.md
- [ ] `deferred-work.md` -- một mục `Chủ: Epic 9`: kiểm tay trên Chương thật (đánh dấu, quét lại, gộp, mở danh sách, xoá toàn bộ)

**Acceptance Criteria:**
- Given một phát hiện, when đánh dấu rồi quét lại, then không ra dây và `filtered` đếm nó.
- Given `proof_ignore`, when xem cột, then không cột nào mang `segment.id`, vị trí, giải thích hay đề xuất.
- Given cụm có ngữ cảnh hai phía, when gộp hoặc tách segment chứa nó rồi quét lại, then vẫn không báo.
- Given Tác phẩm khác có cùng cụm, when quét, then vẫn báo.
- Given lớp phủ danh sách, when bỏ một mục, then lần quét sau báo lại; "Xoá toàn bộ" lần bấm đầu không xoá gì.
- Given một lượt quét xong, when đọc `proof_scan`, then có một hàng đúng phạm vi, số báo, số lọc; lượt huỷ hoặc lỗi không thêm hàng.
- Given gỡ thật bước lọc khỏi `run_proofread_call` (giữ chữ ký), when chạy `proofread_memory_contract`, then ca quét lại đỏ; bỏ cột ngữ cảnh khỏi khoá thì ca "hai cụm giống nhau khác ngữ cảnh" đỏ.

## Implementation Notes

- N = 32 chốt không có số đo: quần thể đọc được trên máy Ice chỉ 19 segment đã dịch; số thật về báo động giả đến ở 9.8.
- `utf16_to_byte` dời sang `core/segment/utf16.rs` vì `proofread_accept.rs` không được nêu `core::ai` (`ai_boundary`), mà hàm chữ ký cần cùng phép đổi.
- Webview trước đây không giữ id Tác phẩm đang mở: `openWorkIdentity.ts` gán ở `openWorkById` và `finishImportSubmission` (phủ cả nối Chương qua `CreatedWork::from_open`); `resetOpenWorkId` chỉ có để `check:panel-refs` xanh, chưa có lời gọi sản xuất.
- `reported` của `proof_scan` đếm phát hiện đã đến trước mọi lượt bỏ qua; `filtered` chỉ nằm trên `Done`, `ambiguous` vẫn theo segment như 9.4.
- Tiêu điểm quay về trong lớp phủ sau bỏ/xoá toàn bộ chọn luôn hàng đầu, nên một lượt "Xoá toàn bộ" đang chờ bấm lần hai bị huỷ; jsdom giữ tiêu điểm trên nút `disabled`, test tự `blur` để giống WebKit.
- Ice duyệt nâng sàn: `VUE_FLOOR` 36→40 (`check-commands`, `check-i18n`), `FILE_FLOOR` 128→139 (`check-layout`), và trong test Rust `FRONTEND_FLOOR`/`WEBVIEW_FLOOR` 128→139, `WEBVIEW_PROOFREAD_FILE_FLOOR` 6→8; `COMMAND_FLOOR`/`DISPATCH_FLOOR` không đổi.
- Tiêu chí gốc nói ghi nhớ sống qua "gộp hoặc tách"; theo AD-53 mục 7, tách trong vùng N quanh cụm thì báo lại, test chỉ khẳng định ca tách ngoài vùng N.

## Plan Change Log

## Review Triage Log

- medium · patch — `ProofreadMemoryOverlay.vue`: sau "Bỏ ghi nhớ"/"Xoá toàn bộ" nút đang giữ tiêu điểm bị `disabled` và hàng bị gỡ, tiêu điểm rơi về `<body>` ngoài `.pfmem-scrim` nên Esc và bẫy Tab chết.
- low · patch — `schema.rs` chú thích `PROOFREAD_MEMORY_DDL` ghi id story, AGENTS.md cấm.
- false — `signature_matches` làm tách segment mất ghi nhớ trái AC 3: AD-53 mục 7 ghi rõ tách hay sửa chữ trong vùng N quanh cụm thì báo lại; test chỉ khẳng định ca tách ngoài vùng N, đúng luật.
- low · reject — `ignoreProofreadFinding` chỉ gỡ đúng phát hiện đã đánh dấu, phát hiện cùng chữ ký trong cùng lượt quét còn hiện tới lượt quét sau: không AC nào đòi, và gỡ ở webview phải tính chữ ký trong TS, trái "một hàm" của AD-53 mục 7.
- false — `openWorkIdentity.ts` có thể giữ id cũ làm mất hàng `proof_scan` (người điều phối nêu): mọi đường đổi `OpenWorkState` trong webview (mở, tạo, nối Chương qua `CreatedWork::from_open`) đều đi qua `openWorkById` hoặc `finishImportSubmission`; lượt quét cũ sau khi đổi Tác phẩm bị thế hệ bỏ trước khi ghi.

## Design Notes

Khoá theo `segment.id` chết ở gộp/tách vì segment = retire + tạo mới, `id` không tái dùng (AD-3 :95, AD-4 :100); Chương gộp/tách chỉ đổi `chapter_id`/`ord`. Gộp chỉ làm ngữ cảnh dài ra và luật khớp "đã lưu là đuôi/đầu của hiện tại" nên ngữ cảnh không rỗng sống sót; ngữ cảnh rỗng (cụm sát biên) là cái mất đã ghi ở AD-53, nên ca gộp của test dùng cụm có chữ hai phía. Cụm lặp loại theo ngữ cảnh (`ambiguous`) không có vị trí nên không ghi nhớ được và không bị lọc.

N = 32: khoảng sáu âm tiết tiếng Việt mỗi phía, đủ một mệnh đề mà không trùm hết đa số câu. N quá nhỏ có nguy cơ nén nhầm một lần xuất hiện khác (lớp lỗi trung tâm, người dùng không thấy); N quá lớn chỉ tốn một lần báo lại (thấy được, sửa được). Vì phía đã lưu được coi là đuôi/đầu của phía hiện tại, tăng N sau này không làm hỏng hàng cũ, giảm thì có. Không đo trước: mẫu hiện có (19 segment đã dịch) quá nhỏ; dữ liệu báo động giả thật đến ở 9.8.

Hệ quả Ice đã nhận: (1) webview đóng giữa `Done` và lệnh ghi thì mất hàng `proof_scan` của lượt đó; (2) bỏ ghi nhớ xoá hàng nên 9.8 không đếm lại được các lần "không phải lỗi" đã bỏ ghi nhớ; (3) lượt huỷ hoặc lỗi không có hàng nên mẫu số chỉ gồm lượt quét xong; (4) số đếm do webview khai, Rust chỉ kiểm được hình dạng và Tác phẩm.

## Verification

**Commands:**
- `npm run build`, rồi `cargo test --test proofread_memory_contract --test ai_proofread_contract --test proofread_write_boundary --test ai_boundary --test ipc_contract --test config_invariants --test project_contract` -- xanh
- Sáu tệp đổi số bước (`chapter_origin_contract`, `segment_role_contract`, `tm_contract`, `review_harvest_contract`, `segment_contract`, `pinned_contract`) chạy riêng từng `--test`
- `npm run test:story 9-6`; `npm run check:commands check:i18n check:tokens check:panel-refs check:layout` -- xanh
- Bộ đầy đủ một lần ở pha cuối (đổi migration và `lib.rs`)

**Manual checks (if no CLI):**
- Trong ứng dụng thật trên một Chương: đánh dấu, quét lại, gộp hai câu, quét lại, mở danh sách, bỏ một mục, xoá toàn bộ (nợ `Chủ: Epic 9`, e2e không giả provider).
