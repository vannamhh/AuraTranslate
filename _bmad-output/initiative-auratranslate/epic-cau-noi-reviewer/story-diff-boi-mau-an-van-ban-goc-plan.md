---
title: 'Diff bôi màu, ẩn văn bản gốc'
type: 'feature'
ticket: '12'
created: '2026-10-09'
status: 'built'
baseline_revision: '332fe527a90549d0e0f50c2f7180193324aaab20'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Review Mode (8.11) hiện hai bản dịch thành hai danh sách độc lập; người dịch phải tự so bằng mắt.

**Approach:** Ghép cặp theo nhóm alignment (8.10), tính diff bằng `diff_spans` phía Rust (AD-51), tô chỗ khác bằng token `diff-*`, cuộn hai panel theo cặp, thêm hai lệnh nhảy khác biệt.

## Boundaries & Constraints

**Always:**
- Diff chỉ tính trong `core/matching::diff_spans`, webview không tính (AD-51 ①). `old` = bản của tôi, `new` = bản reviewer; trái vẽ equal+delete, phải vẽ equal+insert.
- Màu chỉ từ họ token `diff-*` trong `contrast.pairs`; không dựa riêng vào màu (tương phản nền 1,12–1,29:1, số đo 8.1): thêm gạch chân/gạch ngang.
- Nguyên văn không vào DOM. Review Mode vẫn chỉ đọc.

**Never:** chấp nhận/bỏ qua thay đổi (8.13), thêm crate, tính diff ở webview, ghép cặp theo chỉ số mảng hay "Đoạn N".

**Decisions (Ice, 2026-10-09):**
- Phần ngoài nhóm alignment (chưa ghép): hiện thường, gắn nhãn "chưa ghép", không tô.
- "Sửa" = `delete` kề `insert` bằng token `diff-*` hiện có; không thêm họ token mới.
- Mục chưa ghép đứng đúng vị trí trong bản dịch, chen giữa các cặp (trái theo `ord` segment, phải theo thứ tự dòng), không dồn cuối.
- Phím mặc định: `review.diff_next` = `Alt+ArrowDown`, `review.diff_prev` = `Alt+ArrowUp`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Khác | cặp có từ đổi | trái tô từ cũ, phải tô từ mới | — |
| Giống | bằng nhau sau trim+NFC | không tô | — |
| Nhiều-một | nhóm 2 segment, 1 hàng | diff trên văn bản nối như 8.10 (`emit_items`) | — |
| Chưa ghép | đoạn ngoài mọi nhóm, giữa Chương | hiện thường, nhãn "chưa ghép", không `ins`/`del`, đứng đúng chỗ giữa hai cặp kề | — |
| Nhảy cuối | đang ở khác biệt cuối | đứng nguyên, báo bằng chữ | không quay vòng im lặng |
| Lỗi diff | IPC lỗi | lời báo riêng, không panel không màu | khoá `review.diff_failed` |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/matching/mod.rs:486` -- `diff_spans`; `MatchLang` chỉ `Zh`/`En`, chữ Việt đi đường `En` (8.1). Không sửa.
- `src-tauri/src/commands/export.rs:569,786` + `lib.rs:1115` -- khuôn `alignment_open`; thêm `review_diff(chapter_id)` cạnh. Chạm `lib.rs` ⇒ chạy cả bộ.
- `src/config/alignment.ts:7-39,152` -- `AlignmentGroup`, `alignmentOpen`; thêm `reviewDiff` + validator theo `isTmDiffSpan` (`config/segment.ts:1182`).
- `src/reviewModeState.ts` -- ô nhớ cấp module; thêm cặp/chỉ số khác biệt, reset trong `resetReviewMode`.
- Chen mục chưa ghép (sau vòng 1): mỗi panel một danh sách mục gồm cặp và chưa ghép. Trái: segment chưa ghép đứng sau cặp cuối (theo thứ tự cặp) có segment `ord` nhỏ hơn nó, không có thì đứng đầu. Phải: dòng chưa ghép, cùng luật với vị trí dòng trong `alignment.rows`. Cặp giữ đúng thứ tự Rust trả để khoá `data-review-pair` hai bên khớp; mục chưa ghép không mang `data-review-pair` nên `reviewScrollSync` bỏ qua chúng khi neo.
- Phím: `keys: ['Alt+ArrowDown']` / `['Alt+ArrowUp']`; `conflictFor` chạy toàn registry, grep trước đã cho 0 lần dùng.
- `src/panels/ReviewMinePanel.vue`, `ReviewCopyPanel.vue` -- đổi sang danh sách theo cặp. `TmFuzzyStrip.vue:189,327` -- khuôn `<del>/<ins>` + `--color-diff-*`.
- `src/tokens/tokens.json:96-102,294-300`, `scripts/check-tokens.mjs:1118` -- cặp AA hai theme đã canh.
- `src/commands/index.ts:3381`, `reviewModeCommandDeps.ts`, `main.ts:879` -- khuôn `review.open`. `Mod+Alt+ArrowUp/Down` đã xuất hiện 4 lần trong `index.ts`; grep xem còn trống trước khi gán mặc định.
- Cổng: `check:commands` (`COMMAND_FLOOR`, `HANDLER_TABLE` cho `@scroll`/`@keydown`), `check:panel-refs`, `check:i18n`; tệp mới làm sàn quần thể trôi 80% ⇒ nâng cùng lượt (ba cổng này từng đỏ ở `de33eea`).

## Tasks & Acceptance

**Execution:**
- [ ] `commands/export.rs`, `lib.rs` -- `review_diff`: nhóm → văn bản hai bên bằng `emit_items` → `diff_spans(.., En)`; không ghi DB
- [ ] `src-tauri/tests/alignment_contract.rs` (dùng `mine_and_reviewer`) -- ca khác/giống/nhiều-một/bất biến dựng lại; đối chứng: bỏ NFC hoặc đảo thứ tự nhóm thật ⇒ đỏ
- [ ] `config/alignment.ts`, `reviewModeState.ts` -- `reviewDiff`, trạng thái cặp, `reviewDiffNext/Prev`, reset
- [ ] hai panel -- vẽ theo cặp, `<del>/<ins>` bằng token, cuộn đồng bộ theo cặp neo, chống vòng lặp sự kiện
- [ ] `commands/index.ts`, deps, `main.ts`, `vi.json` -- `review.diff_next`, `review.diff_prev`
- [ ] `tests/frontend/reviewMode.test.ts` -- mỗi AC một ca; DOM không có `source_text`; không màu thẳng trong hai panel; đối chứng gỡ thật chỗ gán lớp `ins`/`del` ⇒ ca tô đỏ
- [ ] Cổng -- nâng sàn; chạy cả bộ một lần. Không thêm phụ thuộc (`similar` đã ở Stack; NFR15 không kích hoạt)

**Acceptance Criteria:**
- Given Review Mode mở, when hiển thị, then không chữ nguyên văn nào hiện ở hai panel.
- Given reviewer đổi một từ, when so, then trái tô từ cũ, phải tô từ mới, phần còn lại không tô.
- Given hai theme, when chạy `check:tokens`, then cặp `diff-*` đạt AA và hai panel không có màu viết thẳng.
- Given Chương dài, when cuộn một panel, then panel kia dừng ở cùng cặp.
- Given cặp giống hệt, when hiển thị, then không có `ins`/`del`.
- Given màn phím tắt, when mở, then hai lệnh nhảy có mặt, gán lại được; gọi chúng đưa khác biệt vào tầm nhìn cả hai panel, focus không rơi về `body`.

## Implementation Notes

- Một lệnh Rust `review_diff` (diff ở `core/export/alignment.rs`, `diff_spans(.., En)`); webview chỉ vẽ `ReviewSpans.vue` bằng token `diff-*` kèm gạch chân/gạch ngang.
- Mục chưa ghép chen vào danh sách theo `reviewModeMineItems`/`reviewModeCopyItems` (neo vào cặp cuối có thành viên đứng trước); chỉ cặp mang `data-review-pair`, nên cuộn đồng bộ bỏ qua chúng.
- ⚠️ `Alt+ArrowDown` bị nuốt khi caret nằm trong ô gõ (đo ở 2.5b); Review Mode chỉ đọc nên không có ô gõ, nhưng người gán lại phím cần biết.

## Plan Change Log

- Vòng review 1 (intent_gap): mục chưa ghép bị dồn cuối hai panel và hai lệnh nhảy không có phím mặc định; Ice quyết (khối đóng băng). Tránh: dồn cuối, `keys: undefined`. KEEP: áp nguyên `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/d6a1208d-9b8d-4546-b4ba-eef52a43418c/scratchpad/8-12-round1.patch` (`git apply`, đã kiểm áp được) — lệnh Rust `review_diff`, validator, `ReviewSpans.vue`, `reviewScrollSync.ts`, `diff_failed`, các test, sàn `check-tokens`/`check-layout` và `naming_boundary`/`segment_boundary` 128, `config_invariants` export 6 plain và tổng (76, 56) — rồi chỉ đổi phần dưới. Gộp luôn ba bản vá nhỏ của vòng 1: `GroupDiff` đúng thứ tự chữ cái trong `pub use` của `core/export/mod.rs`; lời assert `config_invariants.rs` "khai 75/56" thành "khai 76/56"; một mục nợ `Chủ: Epic 8` trong `deferred-work.md` (định dạng AGENTS.md, ≤5 dòng) cho lượt dùng thật WKWebView.

## Review Triage Log

Vòng 1 (lens quick):
- medium, intent_gap — mục chưa ghép dồn cuối hai panel (`reviewModeState.ts` v-for pairs rồi v-for unmatched), mất vị trí trong bản dịch; quyết định "hiện thường, gắn nhãn" không nói chỗ đặt.
- medium, intent_gap — `review.diff_next/prev` `keys: undefined`; repo không có bảng lệnh nên lệnh ngủ tới khi tự gán; Code Map ngụ ý gán mặc định, `Mod+Alt+ArrowDown` đã là `editor.next_untranslated`, `conflictFor` chạy toàn registry.
- low, patch (moot vì loopback) — `core/export/mod.rs` `GroupDiff` lệch thứ tự chữ cái trong `pub use`.
- low, patch (moot vì loopback) — `config_invariants.rs` lời assert còn ghi "khai 75/56" sau khi số thành (76, 56).
- low, patch (moot vì loopback) — thiếu mục nợ `Chủ: Epic 8` cho lượt dùng thật WKWebView.
- low, rejected — `alignmentOpen` và `reviewDiff` hai lượt đọc riêng; cửa sổ giữa hai IPC trong lúc `loading`, sửa cần gộp lệnh.
- low, rejected — `echo` có thể nuốt một lượt cuộn người dùng nếu trùng khung hình; lượt cuộn kế đồng bộ lại, sửa thêm cơ chế hết hạn.
- low, rejected — cuộn vào đuôi "chưa ghép" không kéo panel kia; không có cặp để neo, đúng thiết kế cuộn theo cặp.
- false — `review_diff` "ghi DB": chỉ là `align_if_pending` của `read_alignment`, đã chạy ở `alignment_open` ngay trước nên là no-op.
- false — `hasSegment` với segment bị loại khỏi bản dịch: `read_alignment` chỉ lấy `segments_in_translation`, nhóm không chứa chúng.
- false — Prev từ con trỏ -1 về khác biệt đầu: hành vi hợp lý, không báo sai.
- false — `@scroll` không thuộc `HANDLER_ATTR_RE` của `check-commands.mjs`; cổng xanh.
- false — `ReviewDock.vue` thiếu `aura-allow-text`: `check:i18n` xanh.
- rejected (sửa plan) — Implementation Notes trống; đối chứng đỏ không để dấu trong diff (đã khôi phục theo luật).

Vòng 2 (lens quick):
- low, patch — comment đầu tệp `reviewScrollSync.ts`, doc `scrollReviewPairIntoView` và comment trên `interleave` nhắc lại mã, trái luật comment của AGENTS.md.
- low, patch — `ReviewDock.vue` `<p diff-status>` chỉ gắn ở cú nhảy đầu nên hai panel giật xuống một dòng; giữ chỗ từ lúc mở.
- maybe-false (nếu thật thì low), rejected — `Alt+ArrowUp/Down` `preventDefault` ngoài vùng gõ khi Review đóng; có thể ăn cuộn trang Option+mũi tên của WebKit; vùng gõ đã được trả lại (`isTypingZone`+`lacksPrimaryMod`); cần đo trong WKWebView, sửa cần cổng theo chế độ.
- low, rejected — nhãn alt/caption của cặp lấy từ dòng đầu; nhóm trộn loại dòng hiếm, sửa thêm nhánh.
- carried — đối chứng đỏ không để dấu trong diff (rejected vòng 1).
- false — NFC trong `diff_spans`: pha Rust gỡ `.nfc()` khỏi `diff_spans` làm ca NFC đỏ.
- false — mục nợ `Chủ: Epic 8` đã có trong `deferred-work.md` (tệp nằm ngoài phạm vi diff lens đọc).
- false — "gán lại được" không có ca rebind riêng: lớp đè hợp âm khoá theo id (Story 1.21) áp cho mọi command đã đăng ký.

## Design Notes

- "Sửa" = `delete` kề `insert` (`DiffSpan` không có loại thứ ba). Cuộn theo cặp, không theo `scrollTop` tỉ lệ, vì hai bên khác chiều cao dòng nên tỉ lệ trôi dần.
- 8.11 cấm lệnh Rust mới chỉ cho 8.11; AD-51 ① buộc diff ở Rust nên 8.12 cần đúng một lệnh.

## Verification

**Commands:**
- `cargo test --test alignment_contract` và `npx vitest run tests/frontend/reviewMode.test.ts` -- expected: xanh; mỗi phép gỡ đối chứng làm đỏ đúng ca
- `npm run check:tokens && npm run check:commands && npm run check:panel-refs && npm run check:i18n` -- expected: xanh
- cả `cargo test` và vitest một lần (đụng `lib.rs`) -- expected: xanh

**Manual checks (if no CLI):**
- Một lượt dùng thật trong WKWebView: chữ tô đọc được hai theme, cuộn khớp trên Chương dài; ghi nợ `Chủ: Epic 8`.
