---
title: 'Lịch sử phiên bản: xuất xứ từng phiên bản và so với bản đang dùng'
type: 'feature'
ticket: '19'
created: '2026-10-16'
status: 'built'
baseline_revision: '2fe532806650924a93df7726a1c018d1bfea0f23'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/data-integrity.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `segment_version` chỉ có bốn cột nên lịch sử một câu không nói phiên bản nào do ai dịch, không đánh dấu hàng đang dùng, không so được với bản đang dùng; lớp phủ chỉ có danh sách, lệch hai ngăn của mockup (a:149, A17, A18, FR141).

**Approach:** Bậc migration 39 thêm `segment_version.translation_origin` theo AD-55 (`NULL` = không rõ cho hàng có trước); lượt ký và lượt khôi phục ghi/đặt xuất xứ theo một hàm chọn phiên bản thuần; lệnh đọc lịch sử trả `current_version_id` và diff dựng ở Rust cho mọi phiên bản; webview vẽ hai ngăn, nhãn xuất xứ và nhãn "đang dùng" theo `id`.

## Boundaries & Constraints

**Always:**
- AD-55 là đặc tả: ba giá trị `self` · `other` · `bilingual_import` cộng `NULL`; trigger `BEFORE INSERT` chặn `NULL` mới; một hàm thuần chọn "id lớn nhất có `target_text` bằng văn bản sau `trim` + NFC" dùng cho cả nhãn đang dùng lẫn xuất xứ lượt khôi phục; không phiên bản nào khớp ⇒ `None`, không hàng nào mang nhãn.
- Lượt chèn duy nhất ở `confirm_segment` ghi `segment_version.translation_origin` đúng giá trị nó ghi vào `segment.translation_origin`, cùng giao dịch (AD-47).
- Khôi phục (FR101, ngoại lệ AD-47): vẫn ghi `target_text` + `baseline_target_text` + `baseline_translation_origin` trong một câu, không ghi `translation_origin`; `baseline_translation_origin` = xuất xứ phiên bản mà hàm chọn trả cho văn bản vừa khôi phục, phiên bản `NULL` ⇒ `other`.
- Diff là `core::matching::diff_spans(văn bản phiên bản, target_text hiện tại, MatchLang::En)`; webview chỉ vẽ span (`<del>`/`<ins>`, không `v-html`, không thư viện diff JS).
- Mở lịch sử flush segment trước (AD-35, đã có). Sau khôi phục thành công webview đọc lại lịch sử để nhãn đang dùng và diff theo văn bản mới.
- Bốn nhãn cố định: tôi dịch · người khác dịch · nhập song ngữ · không rõ. Chuỗi mới ở `vi.json` (`history.*`), không "bạn" (UX-DR47).
- Tên: không `origin` trần; wire `translation_origin`. Một segment chưa ký, hoặc bản đang soạn không khớp phiên bản nào, phải hiện dòng nói lý do, không chỉ vắng nhãn.
- Migration chỉ tiến; bậc 39 là bậc kế tiếp có thật (38 = `PROOF_DECISION_DDL`). Phép kiểm tập lúc mở kho thêm nhánh `segment_version.translation_origin` (`NULL` được, `''` và giá trị lạ không).
- Đo trên một segment có ≥ 50 phiên bản: lệnh đọc ≤ 50 ms (AD-55 mục 5). Vượt ⇒ pha Rust dừng, ghi số vào handoff và hỏi Ice, không tự đổi sang tính theo phiên bản được mở.

**Never:**
- Không suy xuất xứ phiên bản cũ (không nối `tm_unit`, không điền `''`/`self` lúc di trú), không nhãn từ AI · từ TM · từ bản review, không cột "cơ chế ghi" (AD-55 Prevents, lệch có chủ đích).
- Không so nội dung ở webview để chọn hàng; không nhãn đang dùng cho nhiều hàng.
- Không DTO nào mang khoá `baseline_*`; `translation_origin` của phiên bản chỉ để hiển thị, không gửi ngược.
- Không đổi AD-31/35/47/50/51 (spine đã có dòng 🔵 của AD-55). Không dependency mới (`similar` đã có). Không sửa hộp xác nhận bản nháp (a:154, loại 4) và không sửa cờ `is_target_paragraph_end` khi khôi phục (nợ đỏ của 2.6, ngoài story).
- Không ghi chú TM và dòng phụ "chấp nhận thay đổi của reviewer", "khớp 100% từ Chương 12", "đưa sang từ panel Đề xuất" của mockup: chúng cần nhãn ngoài tập AD-55 hoặc nối `tm_unit` (Prevents 1, 3).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Ba phiên bản (ticket) | Một hàng có trước bậc 39 (`NULL`), một bản gõ tay ký, một bản đề xuất AI đưa sang rồi ký không sửa; văn bản đang dùng = bản AI | Nhãn không rõ · tôi dịch · người khác dịch; đúng một hàng "đang dùng"; mỗi hàng có diff với bản đang dùng (hàng đang dùng: toàn `equal`) | — |
| Hai hàng trùng chữ | Ký A, sửa B, ký A lại; đang dùng = A | Chỉ hàng A mới nhất mang nhãn đang dùng | — |
| Bản đang soạn | Văn bản hiện tại chưa ký, không trùng phiên bản nào | `current_version_id` = null, không hàng nào có nhãn, có dòng "chưa ký" | — |
| Khôi phục người khác dịch | Khôi phục hàng `other` rồi ký không sửa | `segment.translation_origin` = `other`, hàng mới `other` | — |
| Khôi phục hàng `NULL` | Khôi phục hàng có trước bậc 39 rồi ký không sửa | `other` | — |
| Khôi phục tôi dịch | Khôi phục hàng `self`, ký không sửa | `self` | — |
| Khôi phục rồi xem | Khôi phục thành công | Lịch sử đọc lại, nhãn đang dùng chuyển hàng khớp văn bản mới | Lỗi đọc lại ⇒ `history.load_failed`, bản dịch không đổi |
| Segment về hưu | `retired_at` khác `NULL` | Vẫn đọc được, có nhãn và diff | — |
| Chèn `NULL` | `INSERT segment_version` không cột xuất xứ | Trigger ABORT | Lỗi kho |
| Giá trị lạ | Hàng có xuất xứ ngoài tập (sửa tay `.atproj`) | Mở kho lỗi `store.unknown_translation_origin` | Như hàng `segment` |

## Decisions (Ice, 2026-10-16)

- Diff: ngăn chi tiết mặc định hiện toàn văn phiên bản; nút "So với bản đang dùng" (mockup `data-integrity.html:238`) bật/tắt diff qua lệnh mới `history.toggle_compare` (`labelKey` + `HANDLER_TABLE`, `@click` là `dispatch`).
- "So với phiên bản trước" mà mockup hiện mặc định là lệch có chủ đích theo AD-55 (chỉ so với bản đang dùng); không dựng.
- Nguyên văn: lệnh đọc lịch sử trả thêm `source_text` của segment (cùng câu đọc hàng `segment`, 0 truy vấn mới), đúng cả segment đã về hưu; lệch chữ AD-55 mục 7 đúng một trường, Ice chấp nhận.

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/schema.rs` -- `SEGMENT_STATUS_AND_VERSION_DDL` :1687 (doc-comment :1655-1686 khẳng định "đúng bốn cột", thay trong cùng lượt sửa), `PROJECT_MIGRATIONS` cuối bậc 38 :2416; thêm `SEGMENT_VERSION_ORIGIN_DDL` bậc 39: `ALTER TABLE … ADD COLUMN translation_origin TEXT CHECK (… IS NULL OR … IN ('self','other','bilingual_import'))` + `CREATE TRIGGER … BEFORE INSERT … WHEN NEW.translation_origin IS NULL … RAISE(ABORT, …)`. Mẫu cột + CHECK: `WORK_TM_ONLY_MINE_DDL` :1028; mẫu trigger: :328. Index `idx_segment_version_segment_created` (:1878) giữ nguyên.
- `src-tauri/src/commands/project/mod.rs` -- `reject_unknown_translation_origin` :3129: thêm nhánh `segment_version` (chỉ `IS NOT NULL AND NOT IN` ba giá trị, không dùng `TRANSLATION_ORIGINS` vì nó có `''`).
- `src-tauri/src/core/segment/translation_origin.rs` -- nhà của hàm thuần mới (so cùng phép `trim` + NFC với `arbitrate`); `core/tm/mod.rs:44` `PairOrigin` (`from_stored`/`as_str`) để kiểm giá trị.
- `src-tauri/src/commands/segment/confirm.rs` -- câu `INSERT INTO segment_version` :283 thêm cột, giá trị `confirmed_origin.as_str()` đã có trong cùng giao dịch (nhánh `Unsigned` đã bị từ chối trước đó, nên không bao giờ `''`).
- `src-tauri/src/commands/segment/history.rs` -- `SegmentVersionRow` :19 (thêm `translation_origin: Option<String>`, `spans: Vec<DiffSpan>`), `read_segment_history` :88 (nay trả struct mới `{ current_version_id: Option<i64>, source_text, versions }`; đọc `segment.target_text` và `segment.source_text` trong cùng câu đọc hàng `segment` ở :107), `restore_segment_version` :266, chỗ gọi `write_non_user_target(…, TRANSLATION_ORIGIN_OTHER, None)` :~345 đổi tham số thứ tư theo hàm chọn. `targets.rs:234` `write_non_user_target` KHÔNG đổi.
- `src-tauri/src/commands/segment/wire.rs` -- :296 kiểu trả về, :327 khôi phục; `lib.rs:1289,1295` tên lệnh giữ nguyên (không đổi đăng ký).
- `src-tauri/src/core/matching/mod.rs` -- `diff_spans` :490, `DiffSpan`/`DiffKind` :470-485 (`Serialize`, snake_case); không sửa.
- `src/config/segment.ts` -- `SegmentVersion` :~430, `isSegmentVersionArray` :510, `readSegmentHistory` :992 (kiểm kiểu lúc chạy, đổi theo dây mới; dùng lại `ReviewDiffSpan` ở `config/alignment.ts:42`).
- `src/panels/segmentHistoryState.ts` -- state cấp module (`versions`, `aimedVersionId`, `loadHistory` :~190, `restoreVersion` :~215); thêm `currentVersionId`, nạp lại sau `restored`, nhắm hàng đầu khi mở.
- `src/SegmentHistoryOverlay.vue` -- 470 dòng, `.hist-panel` max-width 1100px, danh sách :~255-310 mỗi hàng có nút khôi phục riêng; `src/panels/ReviewSpans.vue` render `<del>/<ins>` (dùng lại).
- `src/i18n/vi.json` -- `history.*` :1489-1511; nhãn `mode.reading.origin_*` :1209 thuộc Chế độ đọc, không dùng chung.
- Mockup `data-integrity.html:178-245`; dòng `bang-lech-a.md:149,151`, `phan-loai-loai-5-a.md` hàng 17-18; nợ `deferred-work.md:3565-3581` (bốn nhãn) và `:3579-3593` (diff), cả hai `Chủ: Epic 12`.
- Test phải đổi theo (phiên bản 38 → 39): `tm_contract.rs:778,786,817,823` (danh sách `IN (28…38)`), `segment_contract.rs:937,1268,2190` (`[Migration; 38]`), `pinned_contract.rs:265-273`, `review_harvest_contract.rs:701`, `chapter_origin_contract.rs:1152`. Cột/CHECK: `segment_contract.rs:1299-1308` (bốn cột → năm) và `:1596-1616` (khẳng định bảng không có `CHECK`). Chèn `segment_version` thẳng SQL (trigger chặn `NULL`): `alignment_contract.rs:1172`, `segment_contract.rs:5185`, `project_contract.rs:3418,7244`; `segment_contract.rs:1679` dựng ở bậc 10 nên không đổi. 21 chỗ gọi `read_segment_history(` ở `segment_contract.rs` đổi theo kiểu trả về. Webview: `tests/frontend/segmentHistory.test.ts` (449 dòng), e2e `e2e/specs/segment-history-restore.e2e.mjs:71` (đọc mảng trần).

## Tasks & Acceptance

**Execution (bốn pha, mỗi pha một agent mới, bàn giao qua tệp `handoff-…-phases/`):**
- [x] Pha 1 -- Plan: giải Open Questions vào khối frozen; không mã.
- [x] `src-tauri/src/core/store/schema.rs`, `commands/project/mod.rs` -- Pha 2 Rust: bậc 39 (cột, CHECK, trigger), sửa doc-comment "bốn cột", nhánh kiểm tập lúc mở kho.
- [x] `src-tauri/src/core/segment/translation_origin.rs` -- Pha 2: hàm thuần chọn phiên bản theo văn bản (id lớn nhất sau `trim` + NFC; không khớp ⇒ `None`).
- [x] `commands/segment/confirm.rs`, `history.rs`, `wire.rs` -- Pha 2: ghi xuất xứ lúc ký; kiểu trả `{ current_version_id, source_text, versions }` với `translation_origin` và `spans` theo `diff_spans`; khôi phục đặt `baseline_translation_origin` theo hàm chọn (`NULL` ⇒ `other`).
- [x] Đo lệnh đọc trên segment ≥ 50 phiên bản, ghi số (build, tải máy) vào handoff -- Pha 2.
- [x] `src/config/segment.ts`, `panels/segmentHistoryState.ts`, `SegmentHistoryOverlay.vue`, `i18n/vi.json`, `commands/index.ts` -- Pha 3 Webview: hai ngăn (danh sách trái: thời điểm, nhãn đang dùng, nhãn xuất xứ; chi tiết phải: nguyên văn từ `source_text`, toàn văn phiên bản hoặc diff theo nút "So với bản đang dùng" = lệnh `history.toggle_compare`, nút "Khôi phục phiên bản này"), ↑↓ nhắm hàng kế bằng `@keydown` cục bộ như Esc, nạp lại sau khôi phục, dòng "chưa ký" khi không hàng nào đang dùng; nút khôi phục theo hàng chuyển lên ngăn chi tiết, `history.restore` giữ nguyên.
- [x] Pha 4 -- Tests that move: các tệp trong Code Map; ca mới `segment_version_origin_contract.rs` (ma trận I/O, trigger, mở kho giá trị lạ, hàm chọn trùng chữ, khôi phục ba nhánh + `NULL`, ký sau khôi phục, diff đúng span, `ipc` ghim `translation_origin` + `spans` và không `baseline_*`); vitest hai ngăn, nhãn theo `id` (hai hàng trùng chữ), nạp lại sau khôi phục, không `v-html`; e2e `story-12-19-*` (ba phiên bản của ticket). Mỗi guard mới đối chứng đỏ bằng phép gỡ thật: bỏ ghi cột ở `confirm.rs`; đổi hàm chọn thành khớp nội dung ở webview; trả `NULL` thay vì `other` khi khôi phục hàng `NULL`.
- [x] `deferred-work.md` -- đóng hai mục `:3565`, `:3579` bằng `→ ✅ ĐÃ ĐÓNG <ngày> (Story 12-19)` (nhãn từ AI · từ TM · từ bản review đóng bằng "lệch có chủ đích AD-55"); thêm nợ lượt dùng thật `Chủ: Epic 12` nếu e2e không chạy được.

**Acceptance Criteria:**
- Given một câu có hàng có trước bậc 39, một bản gõ tay đã ký và một bản đề xuất AI đưa sang rồi ký không sửa (văn bản đang dùng), when mở lịch sử (`⌘H`), then ba hàng hiện không rõ · tôi dịch · người khác dịch, đúng một hàng mang "đang dùng", và mỗi hàng có diff với bản đang dùng.
- Given hai phiên bản trùng chữ, when mở lịch sử, then chỉ hàng mới nhất trong chúng mang "đang dùng".
- Given khôi phục hàng người khác dịch rồi ký không sửa, when đọc `segment.translation_origin`, then là `other`; hàng `NULL` cũng ra `other`; hàng tôi dịch ra `self`.
- Given khôi phục thành công, when lớp phủ còn mở, then nhãn đang dùng và các diff đã tính lại theo văn bản mới.
- Given văn bản đang soạn chưa ký, when mở lịch sử, then không hàng nào có nhãn đang dùng và có dòng nói bản đang soạn chưa ký.
- Given ngăn chi tiết đang hiện toàn văn một phiên bản, when bấm "So với bản đang dùng", then ngăn hiện diff của phiên bản đó với bản đang dùng, và bấm lần nữa trở lại toàn văn.
- Given một segment đã về hưu có lịch sử, when mở lịch sử của nó, then ngăn chi tiết hiện đúng nguyên văn của segment đó.
- Given `project.db` có hàng `segment_version` xuất xứ ngoài ba giá trị, when mở Tác phẩm, then lỗi `store.unknown_translation_origin`.

## Implementation Notes

- Bậc 39 thêm cột và trigger chặn `NULL`; `confirm_segment` là chỗ duy nhất trong mã sản phẩm chèn `segment_version`, nên trigger không chặn đường nào khác.
- Đo lệnh đọc trên segment 60 phiên bản: 4,43 ms trung bình 20 lượt (debug `test`, rustc Homebrew, máy Ice không tải), dưới trần 50 ms của AD-55.
- `tm_contract` "khôi phục bản cũ rồi ký không sửa" đổi kỳ vọng `other` → `self`: phiên bản khôi phục là tôi dịch, đúng hàng tương ứng của ma trận I/O (AD-55).
- Pha 4 sót hai test cũ phải lên bậc 39 (`proofread_decision_contract` hạ bậc 37 phải gỡ cả bậc 39, `segment_role_contract`); chỉ lượt cargo toàn bộ lộ ra.
- Bản `other` trong e2e đến từ `promote_ai_translation` + `confirm_segment` (nhận văn bản làm tham số, không cần nhà cung cấp); hàng `NULL` cần kho trước bậc 39 nên chỉ Rust phủ.
- Đối chứng đỏ bằng phép gỡ thật: `confirm.rs` ghi cứng `'self'` (3 ca đỏ); khôi phục hàng `NULL` ra `self` thay `other` (1 ca đỏ, không thể trả `NULL` vì cột baseline `NOT NULL`); lớp phủ chọn hàng đang dùng theo nội dung thay vì `id` (ca trùng chữ đỏ).
- Một lượt vitest toàn bộ in một lời hứa bị từ chối không ai bắt; hai lượt kế trên cùng cây thoát 0, không lặp lại, chưa bắt được tệp nguồn.
- Bố cục hai ngăn chưa ai xem trong app thật: nợ `Chủ: Epic 12` ở cuối `deferred-work.md`.

## Plan Change Log

## Review Triage Log

- R1 medium · patch: không mục nợ nào giữ lượt xem lớp phủ hai ngăn trong app thật (e2e chỉ gọi IPC); phần "cargo/vitest/cổng chưa chạy" của handoff pha 4 là false, đã chạy sau đó (cargo toàn bộ, vitest 2 lượt, 14 cổng, e2e 3/3).
- R2 medium · patch: e2e `story-12-19-*` chỉ ký ba bản gõ tay (`self`), không mở lớp phủ, lệch task Pha 4 "ba phiên bản của ticket"; chỉ chạm một tệp test nên vá thay vì vòng lại plan, hàng `NULL` cần kho trước bậc 39 nên chỉ Rust phủ.
- R3 low · patch: comment mới vi phạm AGENTS.md (tiếng Việt, nhắc lại mã, id story ở đầu spec e2e) ở `segmentHistoryState.ts`, `segment.ts`, `SegmentHistoryOverlay.vue`, `schema.rs`; tiêu đề `describe('Story 12-19 …')` là quy ước của 12-16/17/18, giữ.
- R4 low · reject: đọc lại hỏng sau khôi phục xoá danh sách đúng như ma trận I/O; `sourceText` cũ vẫn là của chính segment đó, `compareOn` cũ vô hại.
- R5 false: `HANDLER_TABLE` khoá theo tệp gọi trong template; `check:commands` xanh sau pha 4; lớp phủ không có ô nhập nên chặn ↑↓ không nuốt gì.
- R6 false: `wire.rs` trả thẳng `Result<SegmentHistory, IpcError>`, serde của struct chính là dây; ghim ở `segment_version_origin_contract.rs` đủ.
- R7 false: thứ tự nhánh `segment` trước `segment_version` không cho ra kết quả sai, cả hai đều trả `store.unknown_translation_origin`.
- R8 false: `tm_contract` đổi `other` → `self` đúng hàng "Khôi phục tôi dịch" của ma trận I/O (AD-55); ghi ở Implementation Notes.

## Design Notes

- `current_version_id` và nhãn xuất xứ lượt khôi phục cùng đến từ MỘT hàm, nên nhãn đang dùng luôn là hàng mà lượt ký ngay sau khôi phục sẽ ghi xuất xứ theo (AD-55 Prevents 5); khôi phục không dùng `version_id` người dùng bấm để chọn xuất xứ.
- Hàng mới không bao giờ `NULL` nhờ trigger; test cũ chèn thẳng SQL phải truyền một giá trị trong tập.
- Nhãn hiển thị chọn ở webview theo bốn giá trị, `NULL` ⇒ "không rõ"; không dùng chung khoá `mode.reading.origin_*`.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test segment_version_origin_contract --test segment_contract --test tm_contract --test pinned_contract --test review_harvest_contract --test chapter_origin_contract --test alignment_contract --test project_contract --test ipc_contract` -- xanh.
- `npm run test:story 12.19` hoặc `npx vitest run tests/frontend/segmentHistory.test.ts` -- xanh.
- Một lượt toàn bộ một lần (migration, `lib.rs` là wiring chung) -- xanh.
- `npm run test:e2e -- --spec e2e/specs/story-12-19-segment-history-origin.e2e.mjs` -- xanh hoặc ghi nợ nếu webview vẫn đứng ở `tauri://localhost`.
