---
title: 'Chấp nhận từng thay đổi'
type: 'feature'
ticket: '13'
created: '2026-10-09'
status: 'in-review'
baseline_revision: '77d4f38f5a7f17b6ef97e2ef6bc244773dd4e377'
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

**Problem:** Review Mode (8.11, 8.12) chỉ cho xem diff; người dịch không đưa được sửa đổi của reviewer vào bản của mình, cũng không đánh dấu được chỗ đã cân nhắc và bỏ qua (FR94).

**Approach:** Thêm hai command `review.accept_change` và `review.skip_change`, áp lên nhóm alignment đang trỏ. Chấp nhận nhóm 1:1 là một lượt ghi không-phải-người-dùng qua `write_non_user_target`. Quyết định của từng hàng reviewer được lưu trong một bảng mới, bảng này bị xoá cùng bản reviewer.

## Boundaries & Constraints

**Always:**
- Một thay đổi là một nhóm alignment (`alignment_group.id`, đơn vị của `review_diff`). Chỉ nhóm 1:1 mới chấp nhận được.
- Lượt chấp nhận làm trong một giao dịch:
  - Đọc lại nhóm qua `read_alignment`.
  - Kiểm nhóm vẫn là 1:1 với đúng segment và hàng đã hiện.
  - Kiểm `target_text` hiện tại vẫn bằng văn bản bản-của-tôi đã hiện. Nếu khác thì từ chối bằng lỗi có kiểu và tải lại diff (đây là mặc định của story).
  - Kiểm điều kiện hỏi lại theo AD-49 (iii).
  - Ghi qua `write_non_user_target`: status `draft`, origin `other`, đặt cả hai mốc so. Không tạo `SegmentVersion` (AD-31, AD-47 ③, AD-50).
  - Ghi quyết định.
- Ghi ngay, không qua bộ đệm gõ: webview gọi `flushEditorBeforeDiscreteWrite()` trước, sau đó gọi `replaceEditorSegment` (AD-35).
- Nhánh `needs_confirmation` không ghi cột nào (AD-50 ②). Gọi lại với `force` thì mới ghi.
- Quyết định được khoá theo `review_row.id` cùng `review_chapter_id`. Mọi nơi xoá bản reviewer cũng xoá bảng này trong cùng giao dịch, tức trong `delete_alignment_of_chapter` (AD-52 ⑥ ⑦).
- Bản reviewer (`review_row`) không bao giờ bị sửa (AD-52).
- Màn hình xem trước của lần nhập lại nêu phần đã chấp nhận sẽ mất (AD-52 ④).

**Never:**
- Không có lệnh nhận tất cả.
- Không chia văn bản của nhóm n:m.
- Không tính diff ở webview.
- Không thêm crate.
- Không đặt token Review Mode vào đường thu hoạch (`harvest_boundary.rs`).
- Không dùng tên `skip` trần, vì `alignment_skip` của 8.10 đã có nghĩa là "để chưa ghép".

**Decisions (Ice, 2026-10-10):**
- Phím mặc định: `review.accept_change` là `Alt+Enter`, `review.skip_change` là `Alt+Backspace`.
- Alignment của một nhóm bị đổi (nối, tách, chuyển thành viên của segment về hưu) ⇒ xoá quyết định của mọi hàng reviewer trong nhóm đó, cùng giao dịch. Văn bản đã ghi vào segment không bị hoàn lại.
- Bỏ qua được mọi nhóm có ít nhất một hàng reviewer. Nhóm chỉ có segment của tôi chỉ mang lời báo "sửa tay" và không tính là thay đổi chờ xử lý.
- Xem trước lần nhập lại hiện số nhóm đã chấp nhận sẽ mất, đặt cạnh `user_group_count`.
- Segment đã đổi sau khi diff được tính ⇒ áp mặc định của story, như ở Always.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error Handling |
|---|---|---|---|
| Chấp nhận 1:1 | Nhóm đang trỏ là 1:1. Segment có bản sao trong `segment_version` hoặc đang trống. | Segment mang văn bản reviewer, status `draft`, origin `other`. Không có hàng `segment_version` mới. Nhóm hiện "đã chấp nhận". | — |
| Đè bản nháp chưa có bản sao | `target_text` khác rỗng và không có trong `segment_version`. | Trả `needs_confirmation`, không đổi gì. Người dùng đồng ý thì gọi lại với `force` và ghi. | Huỷ thì giữ nguyên. |
| Bỏ qua | Một nhóm bất kỳ | Segment giữ nguyên. Nhóm không còn là "chưa xử lý", kể cả sau khi đóng rồi mở lại Review Mode. | — |
| Nhóm không phải 1:1 | n:1, 1:m, n:m, một phía | Không có nút chấp nhận, chỉ có lời báo "sửa tay trong Editor". Gọi lệnh thì Rust từ chối. | Lỗi có kiểu, i18n |
| Văn bản đã đổi sau khi diff | `target_text` khác văn bản đã hiện | Không ghi gì. Có lời báo, diff tải lại. | Lỗi có kiểu |
| Bản reviewer Stale hoặc NotImported | — | Từ chối. | Lỗi có kiểu của `read_alignment` |
| Nhập lại | Chương có nhóm đã chấp nhận | Xem trước nêu phần chấp nhận sẽ mất. Xác nhận nhập thì xoá các quyết định cùng giao dịch. | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/export/alignment.rs`:
  - `read_alignment` :437 là đường đọc duy nhất.
  - `review_diff` :581 cho ra `GroupDiff{group_id, segment_ids, row_ids, spans}`.
  - `delete_alignment_of_chapter` :373 được gọi từ `reviewer_copy.rs:514 confirm_import` và `commands/chapter.rs:930` (khi gộp Chương). Đây là chỗ xoá quyết định.
  - `user_group_count` :387 là khuôn cho phép đếm của phần xem trước.
- `src-tauri/src/core/store/schema.rs`: thêm v33 vào `PROJECT_MIGRATIONS` (:2138). Không có `REFERENCES`, nên mọi lượt xoá phải viết tay.
- `src-tauri/src/commands/segment/targets.rs`:
  - `write_non_user_target` :234 (`pub(super)`) là hàm duy nhất của AD-50 ③.
  - Điều kiện hỏi lại nằm trong `promote_ai_translation` :276.
  - Kiểu trả về theo khuôn `PromoteAiTranslationOutcome` :211.
- `src-tauri/src/commands/segment/tm_match.rs:272 accept_tm_fuzzy` là khuôn cho "đọc lại, từ chối nếu khác, rồi ủy thác ghi".
- `src-tauri/src/commands/export.rs`:
  - `review_diff` :635, `ReviewDiffPairWire` :617, `alignment_error` :580, `pub mod wire` :698.
  - Phần xem trước: `reviewer_preview_wire` :361, dùng `ReplacedCopy` của `reviewer_copy.rs:366`.
- `src-tauri/src/lib.rs:1112-1119` là `generate_handler!`.
- `src-tauri/src/core/i18n/mod.rs` chứa `message_keys!` cho các key lỗi Rust phát.
- Webview:
  - `src/reviewModeState.ts`: `diffCursor` :41, `reviewModePairs` :54 (đang bỏ mất độ dài `segment_ids`/`row_ids`), `jumpDiff` :209, `resetReviewMode`.
  - `src/reviewModeCommandDeps.ts`.
  - `src/commands/index.ts:3384-3419` chứa các lệnh review. Chord không được trùng toàn cục.
  - `src/config/alignment.ts` là IPC.
  - `src/panels/ReviewMinePanel.vue`.
  - `src/layout/ReviewDock.vue:89` là dòng trạng thái `data-review-diff-status`.
  - `src/panels/editorPanelState.ts`: `flushEditorBeforeDiscreteWrite` và `replaceEditorSegment` :301. `promoteAiTranslationToEditor` :356 là khuôn hỏi lại.
  - `src/i18n/vi.json`: các key `review.*` và `command.review.*`.
  - Phần xem trước: `src/ReviewerImportOverlay.vue` và `src/config/reviewerImport.ts`. Hai tệp này nằm trong đường quét của `harvest_boundary`.
- Không đổi: `review_row`, `alignment_member`, `diff_spans`, `harvest_reviewer_copies`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/store/schema.rs`: thêm v33 `review_decision(review_row_id PK, review_chapter_id, decision accepted|skipped CHECK, decided_at)`. Cập nhật các test ghim phiên bản theo danh sách trong Verification.
- [ ] `src-tauri/src/core/export/review_decision.rs` (mới):
  - Đọc quyết định theo Chương.
  - Ghi quyết định cho các hàng của một nhóm.
  - Đếm số nhóm đã chấp nhận.
  - Xoá theo `review_chapter_id`, gọi từ `delete_alignment_of_chapter`.
  - Xoá theo hàng khi alignment đổi: gọi từ các đường join, unjoin, `alignment_skip` và `write_regroup`/`move_members_of_retired` của `alignment.rs`, trong cùng giao dịch.
- [ ] `src-tauri/src/commands/segment/review_accept.rs` (mới): `review_accept_change(open, chapter_id, group_id, expected_target, force)` thực hiện giao dịch nêu ở Boundaries, dùng `write_non_user_target`. Thêm wire và đăng ký ở `lib.rs`.
- [ ] `src-tauri/src/commands/export.rs`:
  - Thêm `review_skip_change(open, chapter_id, group_id)`.
  - Thêm `decision` vào `ReviewDiffPairWire`.
  - Thêm số chấp nhận sẽ mất vào wire xem trước.
- [ ] `src/reviewModeState.ts` và `src/reviewModeCommandDeps.ts`:
  - Giữ độ dài các vế của nhóm và `decision` trong view.
  - Thêm `reviewAcceptChange`, `reviewSkipChange` và trạng thái chờ đồng ý.
  - Sau khi xử lý xong một nhóm, nhảy tới nhóm chưa xử lý kế tiếp.
  - Khi hết nhóm chưa xử lý thì báo bằng chữ.
- [ ] `src/commands/index.ts`: đăng ký hai lệnh và gán phím mặc định.
- [ ] `src/panels/ReviewMinePanel.vue` và `src/layout/ReviewDock.vue`:
  - Nhóm đang trỏ có nút Chấp nhận và Bỏ qua.
  - Có nhãn đã chấp nhận, đã bỏ qua, không chấp nhận được.
  - Khối hỏi lại nằm tại chỗ, không mở lớp phủ thứ hai.
  - Dòng trạng thái ghi "N thay đổi · đã xử lý M".
- [ ] `src/ReviewerImportOverlay.vue`: hiện phần chấp nhận sẽ mất.
- [ ] Tests:
  - `src-tauri/tests/alignment_contract.rs` cho các hàng của ma trận, gồm đối chứng `segment_version` không có hàng mới.
  - `src-tauri/tests/reviewer_import_contract.rs` cho xem trước và xoá cùng giao dịch.
  - `tests/frontend/reviewMode.test.ts` cho lệnh, phím, nhảy và khối hỏi lại.

**Acceptance Criteria:**
- Given Review Mode đã xử lý hết, when rời Review Mode, then các segment đã nhận thay đổi hiện trạng thái chưa xác nhận trong Editor mà không cần tải lại Chương.
- Given hai lệnh mới, when mở cài đặt phím, then gán lại được và không trùng chord nào.

## Implementation Notes

## Plan Change Log

## Review Triage Log

Vòng 1 (quick): medium 2 · low 3 · false 1 · maybe-false 1.

- medium · patch: `reviewModeChangeStats` đếm nhóm chỉ có segment của tôi vào N, mà nhóm đó không quyết định được, nên M = N không bao giờ tới; Decision của Ice nói nhóm này không tính là chờ xử lý.
- medium · patch: hàng ma trận "Stale hoặc NotImported" chỉ có ca NotImported, thiếu ca Stale cho cả chấp nhận lẫn bỏ qua.
- low · patch: banner `// ───── … (FR94) ─────` trong `alignment_contract.rs` trái luật comment của AGENTS.md; `three_chapter_vec()` chỉ bọc `three_chapter()`.
- low · reject: `skip_change` ở Rust ghi đè được quyết định "accepted" qua lời gọi trực tiếp; webview chặn (`decision === 'accepted'`), cờ `busy` chặn bấm đúp, và sửa thì phải thêm nhánh canh.
- low · reject: AC1 chỉ kiểm qua mock `replaceEditorSegment`; bản thân `replaceEditorSegment` đã có test riêng, còn nhìn tận mắt trong app thật là lượt real-use của Ice cho Epic 8.
- low · reject: AC2 "gán lại được" không có ca gán lại; đó là cơ chế chung của registry (`createKeymap`), lệnh mới đi qua đúng đường đó.
- false: hai lệnh thêm `review.accept_overwrite`/`review.accept_keep` không phải lỗi, vì `check:commands` (AD-34) bắt mọi `@click` dispatch một lệnh đã đăng ký; không có phím mặc định.
- maybe-false · reject: với bản `.md`, `acceptable` của webview không xét `group.mine == segment.target_text` như Rust, nên có thể hiện nút rồi Rust từ chối. Ca này cần một đoạn một-segment mà `emit_items` đổi văn bản, và nếu có thật thì chỉ là low vì lời từ chối đã nói "sửa tay trong Editor". Muốn chốt thì chạy một ca `.md` có đoạn như vậy.

## Design Notes

- "Thay đổi" là nhóm có `changed` hoặc đã có `decision`. Nhờ vậy nhóm đã chấp nhận (nay bằng nhau) vẫn đứng trong danh sách với nhãn của nó, khớp mockup "11 thay đổi · đã xử lý 1".
- `diff_next` và `diff_prev` vẫn đi qua mọi thay đổi. Chấp nhận và bỏ qua thì nhảy tới thay đổi chưa xử lý kế tiếp.
- Thêm hai tệp `.rs` nằm trong khoảng còn trống của sàn (132/141). Không thêm `.vue`, vì sàn `.vue` còn trống 2 tệp.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test alignment_contract --test reviewer_import_contract --test segment_baseline_guard --test config_invariants --test pinned_contract --test segment_contract --test tm_contract --test review_harvest_contract --test project_contract --test chapter_origin_contract --test segment_role_contract --test harvest_boundary`: kỳ vọng xanh.
- Bộ test đầy đủ phải chạy một lần, vì change chạm migration và đăng ký lệnh (AGENTS.md).
- `npx vitest run tests/frontend/reviewMode.test.ts tests/frontend/reviewerImport.test.ts`: kỳ vọng xanh.
- `npm run check:commands && npm run check:i18n && npm run check:panel-refs`: kỳ vọng xanh.
