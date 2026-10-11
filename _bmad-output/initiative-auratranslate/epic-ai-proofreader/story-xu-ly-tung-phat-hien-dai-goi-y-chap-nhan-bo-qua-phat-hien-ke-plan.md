---
title: 'Xử lý từng phát hiện: dải gợi ý, chấp nhận, bỏ qua, phát hiện kế'
type: 'feature'
ticket: '4'
created: '2026-10-11'
status: 'built'
baseline_revision: 'c2ca4be1e9f9a885d0302330ddf637f0c3f078ac'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Phát hiện của 9.1–9.3 mới chỉ gạch chân; chưa đọc được giải thích, chưa nhận được đề xuất, chưa có đường ghi nào từ proofreader tới `target_text` (FR83, FR85).

**Approach:** Dải ở khe chung trên StatusBar hiện phát hiện ở con trỏ; chấp nhận đi qua lượt ghi không-phải-người-dùng của AD-53, bỏ qua chỉ đóng trong bộ nhớ, có lệnh phát hiện kế; một cổng nguồn canh đường ghi duy nhất.

**Decisions (Ice, 2026-10-11):** bỏ qua ⇒ gạch chân và thẻ mất ngay, "n / m" giảm (quét lại báo lại; ghi nhớ bền là 9.6). Khe dải xếp Glossary > Proofreader việc (đang quét, phát hiện chờ ở câu con trỏ, lỗi lượt quét, chưa cấu hình AI, lượt quét 0 câu, còn lượt chấp nhận hoàn lại được, lỗi thao tác) > TM > Proofreader tóm tắt (sạch, đã huỷ, tổng token/chi phí, chưa quét, mơ hồ, không tìm thấy); câu có gợi ý TM thì TM thắng tóm tắt, câu không có TM thì tóm tắt hiện như 9.3 — ⌘P câu sạch và Huỷ không bao giờ im lặng (🔵 Ice 2026-10-11, sau review lần 1). Next/prev chỉ F8/Shift+F8 dù `keys.ts` bỏ phím không ⌘/Ctrl trong ô gõ. Plan giữ nguyên cỡ (2.143 token o200k_base) dù vượt 1.600.

## Boundaries & Constraints

**Always:** AD-53 mục 1–7, AD-47 ③; chấp nhận từng cái, văn bản đề xuất nguyên văn; mọi thao tác là command đăng ký, gán phím được; giải thích nói việc, không đổ lỗi người dùng; thứ tự dải Glossary > Proofreader > TM.

**Never:** chấp nhận tất cả; quét lại sau chấp nhận; "Không phải lỗi" và `PROOF_IGNORE` (9.6); AD mới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error |
|---|---|---|---|
| Chấp nhận | văn bản còn bằng văn bản quét | chỉ `[start,end)` đổi, `draft`, xuất xứ theo loại | — |
| Văn bản đã đổi | `target_text` hay cụm lệch | không ghi | `ai_proofread.text_changed` |
| Khoảng hỏng | ngoài chuỗi, rỗng, cắt surrogate | không ghi | `ai_proofread.range_invalid` |
| Kết quả rỗng | rỗng sau `trim` | không ghi | `ai_proofread.text_would_be_empty` |
| Cụm lặp, loại theo ngữ cảnh | nhiều lần xuất hiện | không đặt vị trí, đếm `ambiguous` | — |

</frozen-after-approval>

## Code Map

- `commands/segment/targets.rs:234` `write_non_user_target` (`pub(super)`): dùng lại, không sửa. Khuôn: `review_accept.rs`; lỗi `confirm.rs:31-50`, `:262`; vỏ `wire.rs:412`; đăng ký `lib.rs:~1121`. `core/segment/translation_origin.rs:35` `arbitrate` chạy trên văn bản CŨ. `proofread_kind.rs` đã có `acceptance_origin()`.
- `core/ai/proofread.rs:100`: cụm lặp hiện đặt ở lần đầu cho MỌI loại; AD-53 mục 5 chỉ cho chính tả. Sửa ca :188, :212. `commands/proofread.rs:61,243` đếm `ambiguous` TỪNG segment trên dây; không thêm trường `ambiguous` cấp kết quả (webview cộng từ segment). Không chạm DB.
- Hình đã chứng minh (diff lần 1, tư liệu): `proofread_accept.rs` có `check_proofread_range` (`pub(crate)`, đổi UTF-16→byte qua `utf16_to_byte`), `replace_proofread_range` (lời gọi `write_non_user_target` duy nhất), vỏ `ai_proofread_accept`/`ai_proofread_revert` (`segment_id, expected_text, start, end, expected_phrase, replacement, kind: ProofreadKind`); hoàn lại = đảo `expectedPhrase`/`replacement`. Cổng `proofread_write_boundary.rs`: quét mọi tệp `src/**` có "proofread" trong tên, cấm `save_segment_targets`, sàn số tệp, ca vị từ bắt từng dạng cấm và bỏ qua dòng chú thích. Webview: `config/proofread.ts` (`acceptProofreadSuggestion`, `revertProofreadSuggestion`), `editorPanelState.ts` export `moveEditorCaretTo(id): boolean` (vỏ mỏng của `doiConTroToi` :1631), `proofreadState.ts` (`proofreadCurrent`, `proofreadRevertible`, `proofreadActionError`, `shiftProofreadFindings`, `acceptProofreadFinding`, `revertProofreadFinding`, `dismissProofreadFinding`, `nextProofreadFinding`, `prevProofreadFinding`).
- Webview: `proofreadState.ts` (`results`, `clearProofreadFor`), `ProofreaderStrip.vue` (`isVisible` :19), `inlineStripEligibility.ts`, `panels/inlineStripPriority.ts` (`InlineStripKind`, `PRIORITY_ORDER`; test `tests/frontend/inlineStripPriority.test.ts`), `commands/index.ts:4159-4222`, `vi.json`, `core/i18n/mod.rs:864`. `replaceEditorSegment` (`editorPanelState.ts:306`) là đường duy nhất áp văn bản Rust (khuôn `reviewModeState.ts:355-376`).
- Khe: hiện Proofreader đủ điều kiện theo `state`; lần 1 thu hẹp còn "đang quét hoặc phát hiện chờ" nên làm mất ⌘P sạch và "Đã huỷ". Nay tách hai loại: `proofreader` (việc) và `proofreader_summary` (tóm tắt).
- Ràng buộc đã học: `ipc_argument_contract` chỉ đọc lời gọi `invoke` có tham số thứ hai là object literal ⇒ mỗi adapter accept/revert viết literal riêng. `config_invariants` census `wire.rs` 18→20 plain, toàn cây 80→82 (thông điệp assert phải khớp số). `check:commands` COMMAND_FLOOR 184→200, DISPATCH_FLOOR 204→220 (Ice duyệt). Test dải phải mock `editorPanelState` ĐẦY ĐỦ, không `importOriginal` (vòng import qua `proofreadState`; spread ghi đè `editorSegments` vô hiệu). `keys.ts` bỏ phím không ⌘/Ctrl trong ô gõ ⇒ F8 chỉ hoạt động ngoài ô (Ice giữ). Không thêm `watch`/ô nhớ cấp module vào `proofread*.ts` (f117472).

## Tasks & Acceptance

**Execution:**
- [ ] Rust: `core/ai/proofread.rs`, `commands/proofread.rs` -- loại theo ngữ cảnh lặp ⇒ `ambiguous` từng segment; KHÔNG có `Done.ambiguous`
- [ ] Rust: `commands/segment/proofread_accept.rs` (mới) -- MỘT hàm kiểm `pub(crate)` (thứ tự lỗi mục 4), MỘT hàm thay khoảng có tham số đảo, hai command, chỉ qua `write_non_user_target`
- [ ] Rust: `wire.rs`, `lib.rs`, `core/i18n/mod.rs`, `vi.json`, `ipc_contract.rs`, `config_invariants.rs` -- vỏ, đăng ký, ba khoá lỗi; thông điệp assert khớp số `(82, 56)`
- [ ] Webview: `config/proofread.ts`, `proofreadState.ts`, `editorPanelState.ts` -- adapter; phát hiện theo con trỏ, chỉ khi `target_text` bằng `scannedText`; chấp nhận = flush, gọi, `replaceEditorSegment`, dời phát hiện (mục 6), hoàn lại LIFO mỗi segment; bỏ qua = xoá trong bộ nhớ. Flush lỗi khi chấp nhận/hoàn lại dùng khoá i18n RIÊNG (không dùng `proofread.flush_failed`/`flush_still_dirty` của quét, nói "chưa quét"). `step` chỉ đặt `focus` SAU khi `moveEditorCaretTo` trả true
- [ ] Webview: `panels/inlineStripPriority.ts` -- thêm `proofreader_summary`; `PRIORITY_ORDER` = quick_add, confirm, `proofreader`, `tm_fuzzy`, `proofreader_summary`
- [ ] Webview: `inlineStripEligibility.ts` -- `proofreader` khi đang quét, phát hiện chờ ở câu con trỏ, lỗi quét, chưa cấu hình AI, quét 0 câu, `proofreadRevertible`, `proofreadActionError`; `proofreader_summary` khi `state !== 'idle'` và (con trỏ null hoặc trong lượt quét) — điều kiện 9.3 cũ
- [ ] Webview: `ProofreaderStrip.vue`, `proofreadHandlers.ts`, `commands/index.ts` -- hiện khi topmost là một trong hai loại; việc: thẻ loại · cụm · giải thích · đề xuất, `n / m`; tóm tắt như 9.3; lệnh accept, dismiss, next (F8), prev (Shift+F8), revert; phím chấp nhận để trống. Không comment `Handler của …` lặp tên trường
- [ ] Tests Rust: bảng xuất xứ mục 2 (4 hàng × 2 cột), hoàn lại, mọi lỗi mục 4 không ghi gì, không `SegmentVersion`; `tests/proofread_write_boundary.rs` mới
- [ ] Tests Webview: dải, dời vị trí, ngăn hoàn lại, bốn ca khe bên dưới, ba dải đủ điều kiện, bỏ qua 0 lời gọi ghi, `inlineStripPriority.test.ts` thêm thứ tự năm loại
- [ ] `deferred-work.md` -- đóng mục "The 7.5 fuzzy strip does not yet yield to the Proofreader" bằng `→ ✅ ĐÃ ĐÓNG 2026-10-11 (Story 9-4)`; thêm mục `Chủ: Epic 9` cho chấp nhận, hoàn lại, F8 trên Chương thật

**Acceptance Criteria:**
- Given phát hiện ở con trỏ, when dải hiện, then có loại · cụm · giải thích · đề xuất, giải thích không đổ lỗi người dùng.
- Given chấp nhận, when xong, then chỉ khoảng đó đổi thành đề xuất nguyên văn, `draft`, không `segment_version` mới, xuất xứ theo AD-53 mục 2.
- Given bỏ qua, when xong, then văn bản, trạng thái, xuất xứ không đổi, 0 lời gọi IPC ghi, gạch chân và thẻ mất ngay, "n / m" giảm một.
- Given ⌘P trên câu sạch ở con trỏ và TM không có gì, when quét xong, then dải tóm tắt "sạch" hiện (không im lặng).
- Given Huỷ giữa lượt quét và TM không có gì, when huỷ xong, then "Đã huỷ" cùng tổng token/chi phí còn hiện.
- Given câu ở con trỏ có gợi ý TM và không có phát hiện chờ, when lượt quét xong, then dải TM thắng tóm tắt.
- Given câu ở con trỏ có phát hiện chờ và có gợi ý TM, when hiển thị, then Proofreader (việc) thắng TM.
- Given Glossary, Proofreader, TM cùng đủ điều kiện, when hiển thị, then Glossary thắng, Proofreader thứ hai, TM không hiện.
- Given lượt quét lỗi, chưa cấu hình AI, quét 0 câu, còn lượt hoàn lại được hoặc thao tác báo lỗi, when hiển thị, then dải việc giữ khe trước TM.
- Given flush thất bại khi chấp nhận hoặc hoàn lại, when báo lỗi, then thông báo không nói "chưa quét".
- Given Chương nhiều phát hiện, when duyệt, then không có lệnh hay nút chấp nhận tất cả.
- Given hoàn lại ngay sau chấp nhận, when ký, then chính tả/ngữ pháp ra như trước, `target_text` đúng byte cũ.
- Given văn bản đổi giữa quét và chấp nhận, when chấp nhận, then `ai_proofread.text_changed`, không ghi gì.
- Given thêm đường ghi thứ hai (lời gọi `write_non_user_target` trong `core/ai/proofread.rs`; hoặc lời gọi thứ hai trong `proofread_accept.rs`; hoặc `invoke('save_segment_targets')` trong `proofreadState.ts`), when chạy cổng, then đỏ đúng cổng đó, gỡ ra thì xanh; cổng có sàn số tệp quét.

## Implementation Notes

- `ai_proofread_accept`/`ai_proofread_revert` share one check and one range-replace; revert replaces the whole current text with the text from before that accept (review 2: a deletion suggestion left an empty range that the check refuses); the only `write_non_user_target` call in the proofread code is in `proofread_accept.rs`, and `proofread_write_boundary.rs` guards it plus `core/ai/proofread.rs` and `proofread*` webview files.
- `ambiguous` is per segment on the wire only; the webview sums it. Spelling repeats are still placed at the first unclaimed occurrence; every other kind is not placed.
- Revert re-adds the original finding at its original start so the underline returns; the revert stack survives a rescan and is cleared only by typing in the segment or reset. Revertible needs the editor text to equal the accepted text.
- Dismiss keeps the entry with empty findings so `unlocated`/`ambiguous` counts survive.
- Next/prev do not wrap; cross-segment moves follow chapter order over segments whose text still equals the scanned text. The first webview cut used `Array.at(-1)`, which wrapped prev at the first finding to the last; fixed and covered by `proofreadActions.test.ts`.
- Accept/revert flush failures use their own keys `proofread.accept_flush_*` (bound to the caret segment), not the scan keys that say "not scanned".
- `F8` had to be added to `NAMED_CODES` in `commands/keys.ts`; it threw otherwise.
- Strip tests mock `editorPanelState` completely and `commands` with a `dispatch` spy; the slot tests use `proofreaderStripSlots.test.ts`, which fails four cases when `proofreader_summary` is removed from eligibility.
- `proofreadBusy` disables the buttons during a write (not in the plan).

## Plan Change Log

- 2026-10-11 loopback 1 (intent_gap): review quick thấy dải chỉ giữ khe khi câu ở con trỏ có phát hiện chờ ⇒ ⌘P câu sạch không hiện gì, Huỷ mất "Đã huỷ" và tổng token/chi phí (verify 9.3). Ice sửa ý định thành hai hạng Proofreader quanh TM và chọn đúng quy trình: toàn bộ mã lần 1 đã hoàn tác, plan lại từ bước 2. Diff lần 1 (đã xanh trừ luật khe dải) là tư liệu, không phải bản để chép.

- 2026-10-11 (Ice, sau pha Webview): dải Proofreader giữ khe thêm khi lượt quét lỗi, chưa cấu hình AI, lượt quét 0 câu, còn lượt chấp nhận hoàn lại được, hoặc thao tác vừa báo lỗi — bỏ đi thì các thông báo đó và nút hoàn lại thành vô hình.
- 2026-10-11 (Ice): next/prev chỉ F8/Shift+F8 như plan dù `keys.ts` bỏ phím không ⌘/Ctrl trong ô gõ; trong ô dùng nút dải hoặc tự gán phím.
- 2026-10-11 (Ice duyệt): `check:commands` COMMAND_FLOOR 184→200, DISPATCH_FLOOR 204→220 theo đề nghị của chính cổng.

## Review Triage Log

- high · intent_gap — `inlineStripEligibility.ts`: dải chỉ giữ khe khi câu ở con trỏ có phát hiện chờ, nên ⌘P một câu sạch không hiện gì (im lặng rỗng), sau Huỷ mất "Đã huỷ" và tổng token/chi phí (verify 9.3), `unlocated`/`ambiguous` chỉ thấy khi còn phát hiện khác; quyết định A của Ice chỉ nêu mất số chưa quét/mơ hồ/không tìm thấy.
- false — `proofreadState.ts` `flushOrFail` khoá i18n không tồn tại: `proofread.flush_failed`/`flush_still_dirty` có ở `vi.json:105-106`.
- low · patch — `proofreadState.ts` `flushOrFail`: hai khoá đó nói "chưa quét", sai ngữ cảnh khi chấp nhận/hoàn lại (phát hiện khi kiểm #1).
- low · reject — `proofread_accept.rs` 8 tham số không allow clippy: clippy không chạy ở CI/pre-push, `wire.rs`/`work_creation.rs` đã có tiền lệ; struct tham số đổi dây Tauri.
- low · patch — `commands/index.ts`: năm comment `Handler của …` lặp lại tên trường mới; phần `check-commands.mjs` là false (diff chỉ đổi số).
- low · patch — `config_invariants.rs:1984`: thông điệp còn "khai 80/56" trong khi assert `(82, 56)`.
- low · patch — `commands/proofread.rs` `Done.ambiguous`: không ai đọc, webview cộng từ từng segment; trường thừa làm guard đòi thêm khoá và có thể lệch.
- low · patch — `proofreadState.ts` `step`: đặt `focus` trước khi `moveEditorCaretTo` thành công; dời bị từ chối thì lần sau vào câu đó bắt đầu sai chỉ số.
- low · reject — `proofread_accept.rs` nhánh `Arbitrated::Unsigned` báo `text_changed`: chỉ xảy ra với câu toàn khoảng trắng mà model không gắn phát hiện; sửa cần khoá mới.
- review 2 · medium · patch (Ice chọn patch thay vì loopback) — `proofreadState.ts` hoàn lại gợi ý xoá chữ gửi khoảng rỗng ⇒ `range_invalid` trong khi dải mời Hoàn lại; sửa: hoàn lại thay cả câu hiện tại bằng câu trước lượt chấp nhận, cùng hàm Rust, ma trận giữ nguyên.
- review 2 · false — `ProofreaderStrip.vue` `findingsText` làm mất "Đã huỷ" cạnh thẻ: dòng huỷ đến từ computed `summary` riêng; ca mới ở `proofreaderStripSlots.test.ts` xanh không cần sửa mã.
- review 2 · low · patch — `proofread_write_boundary.rs`: comment banner `// ─── …` mới, AGENTS.md cấm.
- review 2 · low · reject — `proofreadState.ts` `focus` cũ khi quay lại câu: quét lại đã đặt lại `focus`; quay lại tiếp ở chỉ số cũ không gây hại.
- review 2 · false — `shiftProofreadFindings` bỏ phát hiện chồng lấn không đếm: AD-53 mục 6 quy định "còn lại là chồng lên, bỏ".
- review 2 · low · patch — `segment/mod.rs`: `proofread_accept` chèn sai thứ tự chữ cái của danh sách `mod`/`pub use`.

## Design Notes

AD-53 mục 3 buộc có lệnh hoàn lại (không thì chấp nhận phải hỏi theo AD-49 iii) dù tiêu chí gốc không nêu. "Không phải lỗi" của mockup thuộc 9.6. Hai hạng quanh TM là cách duy nhất để TM thắng tóm tắt mà thông báo việc (lỗi, hoàn lại) vẫn thắng TM; một loại duy nhất buộc chọn một phía. Hai loại dùng chung một component, `isVisible` đúng khi topmost là một trong hai.

## Verification

**Commands:**
- `npm run build`, rồi `cargo test --test ai_proofread_contract --test proofread_write_boundary --test segment_baseline_guard --test ai_boundary --test ipc_contract --test config_invariants` -- xanh
- `npm run test:story 9-4`; `npm run check:commands check:i18n check:tokens check:panel-refs` -- xanh
- Counter-check: gỡ thật lời gọi/seam rồi chạy lại đúng `proofread_write_boundary`; gỡ `proofreader_summary` khỏi eligibility ⇒ ca ⌘P sạch đỏ
- Một spec e2e `proofread-underline` (vòng import); bộ đầy đủ một lần ở pha cuối vì đổi `lib.rs`
