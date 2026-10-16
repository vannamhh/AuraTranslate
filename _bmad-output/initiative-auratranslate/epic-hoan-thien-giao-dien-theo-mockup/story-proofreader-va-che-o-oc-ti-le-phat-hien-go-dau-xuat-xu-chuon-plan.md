---
title: 'Proofreader và Chế độ đọc: tỉ lệ phát hiện, gỡ dấu, xuất xứ Chương'
type: 'feature'
ticket: '15'
created: '2026-10-16'
status: 'built'
baseline_revision: 'a48123e798710cbc937afd453c1da9c85da40f54'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/reading-mode.html'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/proofreader.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Proofreader không lưu quyết định nào ngoài chữ ký "không phải lỗi" (đã nhận và bỏ qua không để lại gì), nên không có tỉ lệ FR150 đúng. Chế độ đọc chỉ đặt được dấu "cần sửa", không gỡ được (`reading_mark` chỉ có INSERT, FR148), không hiện xuất xứ bản dịch ở cấp Chương (FR151), và lệch mockup ở thanh công cụ, affordance, mốc cuối và danh sách chỗ đã đánh dấu (c:129–133, c:141, c:142, c:145).

**Approach:** Bậc migration 38 thêm `proof_decision` theo AD-56, đủ bốn người ghi và một lệnh đọc số đếm; Rust thêm lệnh gỡ một dấu và gỡ hết dấu, và số đếm xuất xứ theo Chương trên lượt đọc. Webview hiện tỉ lệ trên dải Proofreader và Chỗ đã bỏ qua của 12.10, "Sửa lần lượt" và gỡ dấu trong danh sách chỗ đã đánh dấu, dòng xuất xứ đầu Chương, và các dòng lệch còn lại của Chế độ đọc.

## Boundaries & Constraints

**Always:**
- AD-56: mỗi quyết định một hàng; mẫu số = accepted + not_error + skipped; tổng 0 ⇒ "chưa có quyết định", không 0%; webview chưa đọc xong thì không hiện gì (không nhầm "chưa nạp" với "chưa có quyết định"). Phần trăm do webview định dạng (AD-21).
- `ai_proofread_accept` trả kiểu riêng mang `decision_id`; `ai_proofread_revert` nhận `decision_id`, `DELETE` đúng một hàng `accepted` cùng giao dịch với lượt ghi văn bản, 0 hàng ⇒ `ai_proofread.decision_missing` và không ghi gì (kể cả văn bản). Lượt ghi `target_text` giữ nguyên luật AD-47/AD-53.
- *Bỏ qua* thành command Rust chèn một hàng `skipped`; không chạm segment, không tạo ghi nhớ. `proofread_ignore_add` chèn thêm một hàng `not_error` cùng giao dịch kể cả khi chữ ký đã có.
- Gỡ dấu chạy trong giao dịch Rust; `list_reading_marks` giữ phép đếm bất biến hiện có. Xuất xứ Chương chỉ dùng ba giá trị `self` · `other` · `bilingual_import` của `segment.translation_origin`; câu `''` (chưa có bản dịch) và câu cắt bỏ không tính; Chương không có câu nào được tính ⇒ webview hiện "chưa có bản dịch", không nhãn nào.
- Tên: không `origin` trần; wire `translation_origin_counts` (không đụng `ChapterOrigin`/`origin_author`, đó là xuất xứ tài liệu nguồn, AD-43). `Work`/`Chapter`/`ReadingMode`.
- Migration chỉ tiến, bậc 38 là bậc kế tiếp có thật (37 = `WORK_TM_ONLY_MINE_DDL`); `CHECK` tập loại của `proof_decision` giống `proof_ignore`.
- Decision (Ice, 2026-10-16): giữ trọn phạm vi một story (plan 4.725 token), không tách các dòng lệch bố cục Chế độ đọc.
- Decision (Ice, 2026-10-16): xuất xứ Chương (FR151) hiện MỘT nhãn tóm tắt: chỉ `self` ⇒ "bản dịch của tôi"; chỉ `other` ⇒ "người khác dịch"; chỉ `bilingual_import` ⇒ "nhập song ngữ"; từ hai giá trị trở lên ⇒ "hỗn hợp". Wire vẫn là ba số đếm; hàm gộp thuần ở webview, ca kiểm đủ 7 tổ hợp khác rỗng. "hỗn hợp" là nhãn hiển thị, không phải giá trị lưu: không thêm vào tập AD-47 ⑥.
  🔵 2026-10-16: Ice đổi nhãn self từ "bản dịch của bạn" sang "bản dịch của tôi" vì UX-DR47 cấm "bạn"; Kiểm D không có ngoại lệ.
- Decision (Ice, 2026-10-16, sau pha 3): "Sửa lần lượt" bước qua thật: mở chỗ đánh dấu đầu tiên trong Workspace, và Workspace có điều khiển "chỗ kế tiếp (n/m)" đi tiếp qua các dấu còn lại theo thứ tự danh sách mà không về Chế độ đọc; hết dấu thì điều khiển biến mất. Phạm vi mở rộng sang Workspace.
- Decision (Ice, 2026-10-16): "Bỏ hết đánh dấu" hỏi một bước: bấm lần đầu đổi nút thành "Bỏ hết n dấu? Xác nhận", Esc hoặc rời lớp phủ huỷ; chỉ bấm Xác nhận mới gọi `clear_reading_marks`.

**Never:**
- Không đếm từ `proof_ignore`/`proof_scan`, không bộ đếm cộng dồn, không điền ngược `proof_decision` từ `proof_ignore` (đếm thiếu mà trông hợp lệ, AD-56 Prevents 1). Hàng có trước bậc 38 không có quyết định; Tác phẩm cũ hiện "chưa có quyết định".
- Không `DELETE`/`UPDATE proof_decision` ngoài lệnh hoàn lại; `proofread_ignore_remove`/`clear` không xoá hàng quyết định.
- Không tên người dịch ("của Minh"), không sửa FR11, không sửa AD-54/55/57, không lọc TM (12.14), không cột xuất xứ phiên bản (12.19). Không dependency mới. Không đổi tỉ lệ báo động giả FR81 (`scripts/measure-proofread-false-alarm.mjs`).
- Không "mốc cuối Chương 46 · Chương 48" và không A-/A+ (loại 3, dòng đã phân loại).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Tỉ lệ sau quét | Hai phát hiện: nhận một, bỏ qua một | Dải và Chỗ đã bỏ qua: 1 · 0 · 1; mở lại Tác phẩm vẫn 1 · 0 · 1 | — |
| Hoàn lại | Hoàn lại lượt nhận | 0 · 0 · 1; hàng `accepted` biến mất | `decision_missing` ⇒ văn bản không đổi |
| Chưa quyết định | Tác phẩm không hàng nào | "chưa có quyết định" | Lệnh đọc lỗi ⇒ dòng lỗi, không số |
| Mất ngăn hoàn lại | Đổi Chương rồi tự gõ ngược | Vẫn đếm *accepted* (AD-56 ⚠️) | — |
| Bỏ qua hai lượt | Cùng phát hiện bỏ qua ở hai lượt quét | Hai hàng `skipped` | — |
| Gỡ một dấu | Dấu M trong danh sách | Hàng biến mất, `is_marked` của câu về false, số trên nút giảm 1 | Câu đã không có dấu ⇒ không lỗi (idempotent) |
| Gỡ hết | Tác phẩm có 3 dấu | Chỗ cần sửa rỗng, nút "Xem n chỗ" ẩn | Kho lỗi ⇒ `store.write_failed`, dấu còn |
| Xuất xứ Chương | Chương có câu `self`, `other`, `''` | Đếm hai giá trị, `''` bỏ; nhãn "hỗn hợp" | — |
| Xuất xứ một giá trị | Chương chỉ có câu `self` (và `''`) | Nhãn "bản dịch của tôi" | — |
| Bỏ hết, huỷ | Bấm "Bỏ hết đánh dấu" rồi Esc | Nút về như cũ, dấu còn nguyên, không gọi Rust | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/schema.rs` -- `PROOFREAD_MEMORY_DDL` :988 (CHECK tập loại), `PROJECT_MIGRATIONS` cuối bậc 37 :2400; thêm `PROOF_DECISION_DDL` bậc 38 (mẫu `WORK_TM_ONLY_MINE_DDL` :1029).
- `src-tauri/src/commands/proofread.rs` -- `proofread_ignore_add` :404 (thêm hàng `not_error`), `proofread_ignore_remove/clear` :484-500 (không chạm quyết định), `wire` :592; thêm `proofread_decision_skip`, `proofread_decision_counts`. `core/ai/proofread.rs` -- hàm đếm thuần.
- `src-tauri/src/commands/segment/proofread_accept.rs` `replace_proofread_range` :120 (ghi văn bản + chèn/xoá hàng cùng giao dịch); `wire.rs` :471-554 `ai_proofread_accept`/`revert` dùng chung `PromoteAiTranslationOutcome`, tách kiểu.
- `src-tauri/src/commands/segment/reading.rs` -- `mark_reading_segment` :83 (INSERT OR IGNORE), `select_reading_marks` :42, `ReadingChapter` :145, `read_reading_run` :282 (`select_chapter_segments` đã đọc `translation_origin`, `chapter_read.rs:225`: đếm không thêm truy vấn). `regroup.rs:343` UPDATE `reading_mark` (không đổi). `lib.rs:1223,1260,1401` đăng ký lệnh.
- `src/proofreadState.ts` -- `acceptProofreadFinding` :429 (cất `decision_id` trong `RevertItem`), `revertProofreadFinding` :481, `dismissProofreadFinding` :543 (nay gọi Rust), `ignoreProofreadFinding` :550, `resetProofread` :299. `src/config/proofread.ts` bọc IPC.
- `src/ProofreaderStrip.vue` (đầu dải :190-212), `src/ProofreadMemoryOverlay.vue` (12.10 đã thêm ngày và số chặn); `src/commands/index.ts` lệnh `ai.proofread.*`.
- `src/modes/ReadingMode.vue` -- thanh công cụ :416-481, mốc biên :660-681, lớp phủ dấu :683-730, affordance ~:921-945; `src/modes/readingState.ts` :180-302 (`markAimedReadingSegment`, `openReadingMarks`, `resetReadingMarks`); `src/config/reading.ts`; `reading.marks*` ở `commands/index.ts:2695-2706`. Danh sách Chương: `config/chapter.ts::listChapters` (đếm "Chương đã dịch").
- Mockup: `reading-mode.html:139-140,243-252`, `proofreader.html:357-359`. Dòng: `bang-lech-c.md` §6; `phan-loai-loai-5-c.md` hàng 14-16.
- Test phải đổi theo: `review_harvest_contract.rs:699`, `tm_contract.rs:778,817,823,1234,1246,1266` (phiên bản 37 → 38); `segment_contract.rs:2190` (`[Migration; 37]` → 38); `project_contract.rs:2070` (`NON_ENTITY_DETAIL_TABLES` 21 → 22, thêm `proof_decision`); `ipc_contract.rs`, `ai_proofread_contract.rs`, `proofread_write_boundary.rs`; vitest `proofreadActions/State/Memory`, `proofreaderStrip*`, `readingState`.

## Tasks & Acceptance

**Execution (bốn pha, mỗi pha một agent mới, bàn giao qua tệp `handoff-…-phases/`):**
- [x] Pha 1 -- Plan: giải xong Open Questions, ghi quyết định vào khối frozen; không mã.
- [x] `src-tauri/src/core/store/schema.rs`, `core/ai/proofread.rs`, `commands/proofread.rs`, `commands/segment/{proofread_accept,wire,reading}.rs`, `lib.rs` -- Pha 2 Rust: bậc 38; bốn người ghi AD-56 (accept trả `decision_id`, revert theo `decision_id`, `not_error`, `skipped`); `proofread_decision_counts` theo `outcome` và `kind`; `unmark_reading_segment`, `clear_reading_marks`; `translation_origin_counts` trên `ReadingChapter` -- FR148, FR150, FR151.
- [x] `src-tauri/tests/proofread_write_boundary.rs` -- cổng đọc dòng mã: `DELETE FROM proof_decision`/`UPDATE proof_decision` chỉ ở lệnh hoàn lại (AD-56 mục 2). Đối chứng đỏ: thêm thật một `DELETE` ở `proofread_ignore_remove`, chạy đúng target này.
- [x] `src/proofreadState.ts`, `src/config/proofread.ts`, `ProofreaderStrip.vue`, `ProofreadMemoryOverlay.vue`, `commands/index.ts` -- Pha 3 Webview: Bỏ qua gọi Rust; `decision_id` trong ngăn hoàn lại; tỉ lệ (số đếm + phần trăm, "chưa có quyết định"), nạp lại sau mỗi quyết định và khi mở Tác phẩm; reset trong `resetProofread` (`check:panel-refs`).
- [x] `src/modes/ReadingMode.vue`, `readingState.ts`, `config/reading.ts`, `commands/index.ts`, `vi.json` -- Pha 3: nhóm Cỡ chữ liền, chip phím, nhãn Sáng/Tối, nút "⚑ n chỗ đã đánh dấu", dòng tiến độ Chương, "↵ sửa", mốc cuối ba nút, lớp phủ riêng "Chỗ bạn đánh dấu khi đọc" có "Sửa →", gỡ từng dấu, "Bỏ hết đánh dấu" có bước xác nhận, "Sửa lần lượt", dòng xuất xứ Chương bằng hàm gộp nhãn thuần; lệnh mới có `labelKey`, `HANDLER_TABLE`, `DISPATCH_FLOOR`/`COMPONENT_FILE_FLOOR` nâng nếu cổng đòi.
- [x] Pha 4 -- Tests that move: các tệp trong Code Map (phiên bản 38, bảng 22, hình trả mới); ca mới: `proofread_decision_contract.rs` (bốn người ghi, tổng 0, hoàn lại 0 hàng, mở lại Tác phẩm còn số, hàng cũ không điền ngược), `reading_marks_contract` (gỡ một, gỡ hết, idempotent), vitest I/O matrix, hàm gộp nhãn xuất xứ đủ 7 tổ hợp, bước xác nhận Bỏ hết (Esc không gọi Rust). Mỗi guard mới đối chứng đỏ bằng phép gỡ thật (bỏ `DELETE` của revert; bỏ chèn `skipped`; gỡ lượt nạp lại tỉ lệ sau quyết định).
- [x] `e2e/specs/story-12-15-proofread-ratio-reading-marks.e2e.mjs` -- Verify trên app thật; `deferred-work.md` -- nợ lượt dùng thật `Chủ: Epic 12`, và các dòng lệch bị hoãn nếu có.

**Acceptance Criteria:**
- Given quét có hai phát hiện, when nhận một và bỏ qua một, then tỉ lệ hiện 1 · 0 · 1 và còn sau khi mở lại Tác phẩm; hoàn lại lượt nhận thì 0 · 0 · 1.
- Given Tác phẩm không có quyết định nào, when mở dải Proofreader, then hiện "chưa có quyết định", không 0%.
- Given Tác phẩm có dấu "cần sửa", when gỡ hết, then Chỗ cần sửa rỗng và `is_marked` của mọi câu về false sau khi nạp lại.

## Implementation Notes

- `proof_decision` (bậc 38) không điền ngược từ `proof_ignore`: Tác phẩm có trước bậc 38 hiện "chưa có quyết định" dù đã có chữ ký "không phải lỗi".
- Lượt hoàn lại xoá hàng `accepted` TRƯỚC khi ghi văn bản, trong cùng giao dịch; 0 hàng ⇒ `decision_missing`, văn bản không đổi. Lượt gõ tay khôi phục cụm cũ không chạm `proof_decision` (AD-56 ⚠️), nên không có chỗ gỡ để đối chứng đỏ ca đó.
- Câu vai trò (alt, caption) không bị cắt bỏ được đếm vào xuất xứ Chương; plan không nói, pha Rust chọn.
- Ice đổi nhãn self thành "bản dịch của tôi" sau pha 3; mục `VOICE_EXCEPTIONS` pha 3 đã thêm bị gỡ, Kiểm D không ngoại lệ.
- "Sửa lần lượt" bước qua bằng nút "Chỗ kế tiếp (n/m)" ở thanh trạng thái chung, chỉ hiện khi mode là `workspace`; lượt đi là bản chụp danh sách dấu, `resetReading()` không `preserveAnchor` kết thúc nó.
- Nút cũ Mục trước / Mục kế tiếp / Mở trong Workspace của lớp phủ dấu bị bỏ theo mockup; lệnh `reading.marks_next/prev/open` vẫn còn.
- Phần trăm tỉ lệ chia theo phần dư lớn nhất bằng số nguyên để luôn cộng đúng 100.
- Sàn đếm nâng theo cổng: `VUE_FLOOR` 44→48, `FILE_FLOOR` 159→170, `DISPATCH_FLOOR` 299→322, `REGISTERED_COMMAND_FLOOR` 125→136, `FRONTEND_FLOOR`/`WEBVIEW_FLOOR` 153→167, `WEBVIEW_PROOFREAD_FILE_FLOOR` 8→11.
- e2e `story-12-15-proofread-ratio-reading-marks` gọi IPC thật, chưa chạy vì webview đứng ở `tauri://localhost`.

## Plan Change Log

- 2026-10-16 sau pha 3: Ice đổi nhãn self thành "bản dịch của tôi" (bỏ mục `VOICE_EXCEPTIONS` pha 3 đã thêm) và mở rộng "Sửa lần lượt" sang điều khiển bước qua ở Workspace; pha 3b làm cả hai trước pha 4.

## Review Triage Log

- `ReadingMode.vue` chip phím trên nút "⚑ n chỗ đã đánh dấu" là phím của `reading.mark_aimed` — false: mockup `reading-mode.html:127` đặt `M` cạnh số dấu, `M` là phím làm số đó tăng.
- Chú thích tiếng Việt, kể lại tên hàm ở `reading.rs` và `segment/wire.rs` — low → patch: còn một dòng tiếng Anh nói hợp đồng `false`/số đếm, bỏ hai dòng ở vỏ wire.
- Phần trăm làm tròn độc lập cộng ra 99/101, thanh hở hoặc tràn — low → patch: `wholePercents` chia theo phần dư lớn nhất bằng số nguyên; ca mới đỏ khi gỡ thật về `Math.round`.
- "Sửa lần lượt" dùng bản chụp nên vẫn mở dấu đã bị gỡ giữa lượt — low, bỏ: chỉ xảy ra khi rời Workspace về lớp phủ để gỡ, hậu quả là mở một câu không còn dấu; chặn cần thêm nhánh.
- Lượt bước qua biến mất không báo khi `resetReading()` chạy (đổi Chương, nhập) — low, bỏ: kết thúc lượt khi người dùng tự đi chỗ khác là hợp lý, thêm thông báo là thêm bề mặt.
- Ô Tasks, Implementation Notes trống — bỏ: sửa plan của chính build này.
- e2e chưa chạy — defer: đã có mục nợ `Chủ: Epic 12` (webview đứng ở `tauri://localhost`).
- e2e chỉ gọi IPC, không chạm UI — low, bỏ: runner không có provider; các dòng UI có ca vitest DOM.
- Không thấy lượt chạy toàn bộ sau migration — maybe-false → chạy toàn bộ một lần sau lượt vá review.
- Cổng `proof_decision` bỏ lọt `REPLACE INTO` / `INSERT OR REPLACE` và chữ thường — low → patch: thêm mẫu, so chữ hoa; gài thật `INSERT OR REPLACE` vào `proofread_decision_skip` làm cổng đỏ.

## Design Notes

- "Sửa lần lượt" nằm ở Chế độ đọc (mockup `reading-mode.html:243`, C16: mở từng chỗ qua `reading.marks_next`/`marks_open` có sẵn), dù mô tả entry ghi dưới Proofreader.
- Số đếm xuất xứ Chương tính từ các hàng `select_chapter_segments` đã đọc trong cùng lượt: không thêm truy vấn, không lệnh IPC phụ.
- "Chương đã dịch" của dòng tiến độ = số Chương `done` từ `listChapters`; chưa nạp ⇒ ẩn dòng, không "0".

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test proofread_decision_contract --test proofread_write_boundary --test proofread_memory_contract --test ai_proofread_contract --test ipc_contract --test project_contract` -- xanh.
- `npm run test:story 12.15` hoặc vitest các tệp proofread*/readingState -- xanh.
- Một lượt toàn bộ một lần (migration, `lib.rs` là wiring chung) -- xanh.
- `npm run test:e2e -- --spec e2e/specs/story-12-15-proofread-ratio-reading-marks.e2e.mjs` -- xanh (webview e2e đang đứng ở `tauri://localhost`, có thể phải chờ nightly).
