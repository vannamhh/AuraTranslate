---
title: 'Dịch lô gom câu liền nhau thành lượt gọi (AD-60)'
type: 'feature'
ticket: 15
created: '2026-10-27'
status: 'done'
baseline_revision: 'fd7fec3a7505cefd675de90d7744c88d0237c5c8'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-ai-mo-va-smart-rag-injector/tickets/story-dich-lo-gom-cau-thanh-luot.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Dịch lô gửi mỗi câu một request (không ngữ cảnh kề, trả đủ giá phần cố định), và `translate` coi "`[DONE]` có chữ" là xong nên luồng bị cắt giữa câu mất 292–525 câu không báo lỗi (AD-60 Prevents 1–3).

**Approach:** Thi hành AD-60 đúng như đã ký: Rust lập kế hoạch lượt (tối đa 100 câu cần dịch), lắp một prompt khối mỗi lượt, đọc khối theo ngữ pháp đóng, chỉ phát `Received` sau khi luồng của lượt kết thúc; câu không có khối đóng đúng thành *thiếu kết quả AI*; đưa sang Editor kiểm `source_hash`. Story gốc là nguồn chính cho 13 tiêu chí; plan này không chép lại chúng.

## Boundaries & Constraints

**Always:** Story §Boundaries + AD-60 §15. Một Channel mỗi thao tác, không tự thử lại. Mọi sự kiện mang `call_index`. Một tầng chia lượt (Rust), một tầng cộng usage (webview, theo `call_index`). Glossary đã phân giải và chính sách TM đọc một lần mỗi thao tác.

**Never:** Webview giữ hằng trần lượt. Lượt dịch lô ghi `target_text`. Hàm thứ hai viết lại khâu mở rộng thân. Đổi Proofreader, system prompt mặc định, hình `IpcError`. Thêm điểm ra mạng. Comment lịch sử trong mã.

**Decisions (Ice, 2026-10-27, checkpoint 1):**
- Plan giữ nguyên, không tách (4.749 token): story không merge rời được.
- Màn Xem prompt: bản ghi mới hơn thắng giữa bản ghi lượt và bản ghi dịch một câu; một đồng hồ chung đóng dấu cả hai, `AssembledPromptRecord` được sửa theo.
- `source_hash` dùng `std::hash::DefaultHasher`, không thêm dependency; chỉ cần đúng trong một phiên app.

## I/O & Edge-Case Matrix

Đã có ở story (tiêu chí 1–11) và AD-60 §14 (a)–(u); bảng ánh xạ ca → task → target ở §Tasks.

</frozen-after-approval>

## Code Map

Đã đối chiếu lại với mã hiện tại; `git diff 7bd1f90 HEAD -- src src-tauri tests e2e` rỗng, nên research doc còn đúng.

**Dùng lại, không viết lại**
- `src-tauri/src/core/ai/rag.rs`: `expand_prompt_body` (:557, private, khâu mở rộng thân duy nhất), `render_glossary_pairs` (:380), `select_tm_pairs`/`render_tm_pairs` (:392/:423), `gather_glossary_context` (:228), `load_tm_rows`/`gather_tm_context` (:305/:325), `InjectionLedger` (:194), `TM_PAIR_CAP` (:282). Các hằng `PROOFREAD_SEGMENT_OPEN/CLOSE` (:782/:784) là dấu `<<<SEGMENT`/`SEGMENT>>>` của AD-60 mục 4.
- `core/dict::is_han` (src/core/dict/mod.rs:166), gọi nguyên, không chép bảng dải (`dict_boundary.rs:313` đếm đúng một định nghĩa).
- `commands/aiprompt.rs:485 assemble_and_record_prompt_with_tm` (resolve hai tầng, `TmRowsCache`, `resolve_system_prompt`), `:611 mark_prompt_as_sent`.
- `commands/segment/targets.rs:295 promote_ai_translation` (`needs_confirmation`, `is_unsigned_draft`, `write_non_user_target`); `segment/wire.rs:363`.
- `commands/aitranslate.rs`: `openai_client_error_family` (:119), `insert_status_params` (:226), `AiTranslateGeneration` (:650), `read_api_key`/`resolve_call_config`, `send_prepared_*` mẫu `spawn_blocking`+`block_on` (:1098/:1147), `AiTranslateUsageWire` (:937).
- `TranslationProvider::is_empty_reply` (ports/translation_provider.rs): `EmptyReply` ở lô là lượt kết thúc không chữ, không phải lỗi dừng thao tác.

**Đổi**
- `ports/translation_provider.rs`: `TranslateOutcome::Done(Option<TranslateUsage>)` thành `Done { usage, finish_reason: Option<String> }` (bỏ `Copy`). `client.rs:575` (`SseEventOutcome::Done`) trả `last_finish_reason`; không phán. 28 chỗ khớp `Done(` ở `commands/{aitranslate,proofread}.rs`, `tests/{ai_translate_contract,ai_proofread_contract,proofread_memory_contract}.rs`.
- Mới `core/ai/batch.rs` (thuần): hằng `TRANSLATE_CALL_MAX_SEGMENTS`, `CONTEXT_GAP_MAX`, `TM_PAIRS_PER_SEGMENT` (= `TM_PAIR_CAP`), dấu khối (di chuyển hai dấu SEGMENT khỏi rag.rs, byte không đổi), `plan_calls`, `guard_marker_line`/`unguard_marker_line` (U+200B), `BlockStream` (đọc tăng dần, trả mảnh chữ theo chỉ số), `reconcile`, `has_han`. Khai trong `core/ai/mod.rs`.
- `rag.rs`: thêm `assemble_call_prompt` cạnh `assemble_prompt`, gọi lại `expand_prompt_body` với `glossary_block` đã gộp, `tm_block` là dòng dẫn cố định, `source_segment` là chuỗi khối; sổ chèn cộng theo câu.
- `commands/aiprompt.rs`: bản ghi theo lượt (`AiCallRecordsState`, khoá `(operation_id, call_index)`: prompt, system, `segment_ids` theo n, `sent_usage`, sổ chèn); `ai_prompt_read_record` nhận `segment_id` tuỳ chọn.
- `commands/aitranslate.rs`: thay `PreparedBatchItem`/`prepare_batch_call*`/`run_batch_call`/`AiTranslateBatchEventWire`/`batch_stopped_error(segment_id,..)` bằng kế hoạch, bộ chạy thao tác (`run_translate_operation<P>`), sự kiện `Planned|CallStarted|Token|Received|Discarded|CallDone` (đều `call_index`), `batch_stopped_error(call_index, first, last, err)`. `ai_translate_batch(scope, prompt_set_name, channel)`; `AiTranslateOutcomeWire::Done` thêm `maybe_truncated`; lệnh mới `ai_translate_resend_call`.
- `segment/targets.rs`+`wire.rs`: `promote_ai_translation` giữ chữ ký công khai, gọi `promote_inner(.., expected_source_hash: Option<&str>)`; wire thêm `source_hash: Option<String>`. Khớp thất bại ⇒ từ chối, không ghi (khoá lỗi dùng khóa sẵn có `segment.*`; thêm `MessageKey` + `vi.json` nếu cần).
- `lib.rs`: `:980` (đo E7 gọi `ai_translate_batch`), đăng ký lệnh (:1414), `app.manage` trạng thái bản ghi lượt, dọn khi đóng Work (:1889). Đây là dây chung.
- `src/config/aitranslate.ts` (:177 kiểu sự kiện, :208 `runAiTranslateBatchCall`), `config/aiprompt.ts:346`, `config/segment.ts:1123` (thêm `sourceHash`), `aiTranslateBatchState.ts` (viết lại: hàng `queued|streaming|received|missing(reason)|not_sent`), `aiTranslateHandlers.ts` (:111 chapter, :140 retry), `chapterTranslateScope.ts` (xoá; Rust chọn câu), `panels/AiTranslationPanel.vue` (:291 nhãn hàng, :637 thanh lô), `panels/editorPanelState.ts` (:389/:452 đưa sang kèm hash), `aiPromptInspectorState.ts`, `i18n/vi.json`, `commands/index.ts` nếu thêm id.

**Không đổi:** đường dịch một câu (chỉ thêm cờ), Proofreader (`commands/proofread.rs`, `assemble_proofread_batch_prompt` byte-for-byte), `ai_translate_cancel`, `AiTranslateGeneration`, keychain, `IpcError`.

**Gate chạm vào**
- `ai_boundary.rs`: `commands/aitranslate.rs` chỉ được gọi tên `client::{OpenAiChatClient,OpenAiClientError}` (:402). Nay cần thêm marker `crate::core::ai::batch::` và danh sách tên cho phép riêng; `aiprompt.rs` thêm `assemble_call_prompt` vào danh sách 14 tên (:229); bước gỡ seam của ca biên dịch `deleting_core_ai…` (#[ignore], :2118) phải gỡ thêm dòng mới.
- `config_invariants.rs:1489` bảng đếm lệnh theo tệp và `:1722` tổng `(70,28)`; `ipc_contract.rs:2020,2579` tham số `ai_translate_batch`, `promote_ai_translation`; `ipc_argument_contract.rs`.

## Tasks & Acceptance

Mỗi phase một agent mới, bàn giao qua tệp. Phase Rust và Webview mỗi phase xong phải biên dịch và xanh ở các target nêu.

**Phase 1 — Plan:** tệp này. 0 task.

**Phase 2 — Rust** (lệnh: `cargo test --test ai_batch_wire --test ai_translate_contract --test ai_translate_wire --test ai_boundary --test ai_prompt_contract --test ai_rag_contract --test ai_proofread_contract --test proofread_memory_contract --test ipc_contract --test ipc_argument_contract --test segment_wire --test tm_contract`; `npm run build` trước)
- [x] R1 `ports/translation_provider.rs`, `core/ai/client.rs`, `commands/{aitranslate,proofread}.rs` -- `Done { usage, finish_reason }`; sửa 28 chỗ khớp; Proofreader dùng `Done { usage, .. }`.
- [x] R2 `core/ai/batch.rs`, `core/ai/mod.rs` -- hằng, dấu khối, U+200B hai chiều, `plan_calls` (đoạn rời, `is_omitted` vô hình, trần 100, cắt đoạn đã dịch > 3, ngữ cảnh chỉ chế độ cả Chương, vùng chọn lấy câu đã dịch), `BlockStream`, `reconcile` (chỉ theo chỉ số; `stop` ⇒ `missing`, khác ⇒ `no_finish`, lỗi/huỷ ⇒ `interrupted`), `has_han`.
- [x] R3 `core/ai/rag.rs` -- `assemble_call_prompt` + phần dặn định dạng khối nối sau system đã phân giải (kể cả rỗng); dấu SEGMENT lấy từ `batch.rs`; test inline `proofread_prompt_tests` giữ nguyên.
- [x] R4 `commands/aiprompt.rs` -- `prepare` theo thao tác đọc Glossary đã phân giải, chính sách TM, system **một lần**; `prepare_call` lắp từng lượt, bỏ câu đã về hưu; trạng thái bản ghi lượt; `ai_prompt_read_record(segment_id)`.
- [x] R5 `commands/aitranslate.rs` -- `ai_translate_batch(scope: selection|chapter)`: đọc ảnh chụp Chương một lần, `Planned` một lần, `run_translate_operation` tuần tự, kiểm huỷ trước mỗi lượt và giữa khung, `Received|Discarded` ngay trước `CallDone`, `source_hash` (xem OQ2), lỗi mang `call_index/first/last`, luồng đóng thiếu `[DONE]` là lỗi nhưng giữ khối đóng đúng; `maybe_truncated` cho đường một câu.
- [x] R6 `commands/aitranslate.rs` -- `ai_translate_resend_call(operation_id, call_index, channel)` gửi nguyên prompt đã ghi, đọc qua cùng `run_translate_operation`.
- [x] R7 `segment/targets.rs`, `segment/wire.rs` -- `promote_inner` + `source_hash`.
- [x] R8 `lib.rs` -- đăng ký, manage, dọn khi đóng Work, sửa `:980`.
- [x] R9 `tests/ai_boundary.rs`, `config_invariants.rs`, `ipc_contract.rs`, `ipc_argument_contract.rs`, `src/i18n` + `MessageKey` -- cập nhật gate như §Code Map; `vi.json` cùng `{placeholder}`.
- [x] R10 `tests/ai_batch_wire.rs` (mới, máy chủ SSE cục bộ khung viết tay, nhiều kết nối kịch bản; mượn khung từ `ai_translate_wire.rs:71,144`), `ai_translate_contract.rs` (xoá/viết lại ca `prepare_batch_call`, `run_batch_call`, `PreparedBatchItem`, `batch_stopped_error`: dòng 2605–3800, 4341), `ai_translate_wire.rs` (ca lô :324–:511, :844, :983, :1159).

**Phase 3 — Webview** (lệnh: `npx vitest run` đúng các tệp dưới + `npm run check:i18n check:commands check:panel-refs check:lint` + `npm run build`)
- [x] W1 `config/aitranslate.ts` -- kiểu và kiểm kiểu sự kiện mới, `runAiTranslateBatchCall(scope, ..)`, `resendAiCall`; `isAiTranslateOutcomeWire` nhận `maybe_truncated`.
- [x] W2 `aiTranslateBatchState.ts` -- hàng theo `Planned`, `Received|Discarded` (xoá chữ đã stream), `CallDone` cộng usage một lần mỗi `call_index`, `not_sent` khi dừng, không hằng trần lượt.
- [x] W3 `aiTranslateHandlers.ts`, `chapterTranslateScope.ts` -- chapter gửi `scope: chapter`; thử lại luôn `selection` trên `missing ∪ not_sent`.
- [x] W4 `config/segment.ts`, `panels/editorPanelState.ts` -- đưa sang kèm `source_hash`; nút tắt khi hàng không `received`.
- [x] W5 `panels/AiTranslationPanel.vue`, `i18n/vi.json` -- nhãn *thiếu kết quả AI*, *chưa gửi*, cờ `has_han`/`maybe_truncated`, lỗi nêu lượt/câu đầu/câu cuối; `@click` chỉ `dispatch`.
- [x] W6 `config/aiprompt.ts`, `aiPromptInspectorState.ts` -- bản ghi theo lượt, đánh dấu khối, "chỉ là ngữ cảnh", gửi lại qua W1.
- [x] W7 vitest: `aiTranslateBatch.test.ts`, `aiBatchPanelRows.test.ts`, `aiChapterRun.test.ts`, `aiChapterRunPanel.test.ts`, `aiTranslateBatchResetWiring.test.ts`, `aiTranslateHandlersWiring.test.ts`, `aiTranslatePromoteConfirmFlow.test.ts`, `aiTranslateResendCall.test.ts`, `aiPromptInspector.test.ts`, `aiPromptInspectorScreen.test.ts`, `aiTranslationPromoteErrorAlert.test.ts`.

**Phase 4 — Tests that move:** chạy phép gỡ thật từng ca (bảng dưới) bằng đúng target của ca; chạy **một** lần đủ bộ (lib.rs và đăng ký lệnh đổi); `cargo test --test ai_boundary -- --ignored` cho ca biên dịch; mục nợ `Chủ: Epic 4` (tiêu chí 13, ≤5 dòng, grep `deferred-work.md`); dòng 🔵 vào spec 4.9; đọc CI và nightly e2e trước `done`.

**Ánh xạ tiêu chí ⇒ task:** 1 R2,R5 · 2 R2 · 3 R3,R4 · 4 R2,R5,W2 · 5 R5 · 6 R5,W2 · 7 R5,W3 · 8 R7,W4 · 9 W2 · 10 R4,R6,W6 · 11 R2,R5,W5 · 12 R10 + bảng · 13 Phase 4.

**Ca AD-60 §14 ⇒ target ⇒ phép gỡ thật** (target Rust là `ai_batch_wire`, trừ khi ghi khác; mọi phép gỡ giữ chữ ký hàm)
- (a) finish_reason vắng ⇒ `no_finish`: gỡ nhánh `finish_reason != Some("stop")` trong `reconcile`. (b) `length`: gỡ riêng so khớp chuỗi (chỉ còn kiểm `None`). (c) thiếu khối k: gỡ vòng phát `Discarded{missing}` cho chỉ số vắng.
- (d) trùng chỉ số: gỡ phép kiểm trùng. (e) rỗng: gỡ `trim().is_empty()`. (f) mở thiếu `<<<`: gỡ nhánh dạng lỏng trong `BlockStream`. (g) nguồn có `SEGMENT>>>`: gỡ `guard_marker_line` khi lắp.
- (h) huỷ giữa hai lượt: gỡ kiểm `should_cancel` trước lượt; `CallDone` khi huỷ: gỡ lệnh gửi trong nhánh huỷ. (h2): đưa `Received` về phát ngay lúc khối đóng.
- (i) đoạn rời: coi vùng chọn là một khoảng. (j) 100/100/50: gỡ chia theo trần. (k): gỡ cắt đoạn đã dịch > 3. (l): gỡ lọc `is_omitted`. (m): đưa id ngữ cảnh vào `Planned`/`Token`.
- (n) `has_han`: ép `false`; webview chỉ nhận cờ (vitest). (o): đảo thứ tự nối phần dặn. (p): gỡ kiểm marker vắng (luôn chèn Glossary). (q): gỡ `call_index` khỏi `params` trong `batch_stopped_error`. (r): vùng chọn lọc câu đã dịch (như chế độ chapter).
- (s) `ai_batch_wire`/`segment_wire`: gỡ so hash trong `promote_inner`. (s2) `Planned` đủ lượt (Rust) + vitest `aiTranslateBatch.test.ts` quét `src/**` không có hằng trần: thêm `const MAX = 100` vào state thì đỏ.
- (t) `CallDone.usage` đúng kịch bản (Rust) + vitest cộng một lần mỗi `call_index`: gỡ khử trùng thì đỏ. (u): đổi `tm_only_mine` của Work giữa hai kết nối; gỡ bộ đệm `TmRowsCache` theo thao tác thì lượt 2 đọc lại và đỏ.
- Phụ (tiêu chí 11): đường một câu không `finish_reason` ⇒ `maybe_truncated` (`ai_translate_wire`): gỡ gán cờ. Proofreader: `ai_proofread_contract` + `proofread_prompt_tests` xanh không sửa.

## Implementation Notes

- `EmptyReply` (chỉ `[DONE]`) trong một lượt cho mọi câu `missing(no_finish)`: trait cổng không trả được `finish_reason` của lỗi; hiện ra vẫn là *thiếu kết quả AI*, thao tác chạy tiếp.
- Thứ tự cắt của bộ lập kế hoạch: đoạn đã dịch > 3 thắng (lượt mới mang 3 câu ngữ cảnh), rồi mới tới trần 100 (không ngữ cảnh). Hai lý do cùng lúc thì lượt vẫn bị cắt vì đoạn đã dịch, nên mang ngữ cảnh đúng AD-60 mục 2.
- W3 không xoá `chapterTranslateScope.ts`: xoá nó làm mất con số câu trên nút dịch Chương. Tệp chỉ còn đếm để hiển thị; tập câu gửi đi vẫn do Rust lập.
- Phase Rust đầu đọc Glossary theo từng lượt, trái AD-60; đã sửa thành một ảnh chụp `GlossarySnapshot` mỗi thao tác, canh bằng ca (u) cho Glossary.
- Chữ "similar" trong câu dặn TM khớp tên crate `similar` mà `matching_boundary` (AD-17) cấm ngoài `core/matching`; câu đổi thành "close", không thêm miễn trừ.
- `batch.rs` mới đẩy số tệp `src/**` lên 148, nên sàn `SRC_RS_FLOOR` của `proofread_write_boundary` được nâng từ 118 lên 126.
- `cargo test` dừng ở binary đỏ đầu tiên: lượt đủ bộ đầu bỏ sót 47 target và doctest; đã chạy nốt bằng `--no-fail-fast` và `--doc`.
- Gửi lại một lượt không kiểm câu đã về hưu; ghi sang Editor vẫn bị `promote_inner` chặn (về hưu, rồi `source_hash`).

## Plan Change Log

## Review Triage Log

- [quick] `aitranslate.rs` `run_translate_operation`: lượt lỗi/huỷ không gọi `call_sent` — medium, patch. Huỷ chỉ kiểm giữa khung sau `.send()`, nên lượt đã đi; đã dời `call_sent` lên trước nhánh dừng, ca `a_call_that_fails_after_it_was_sent_is_still_recorded_as_sent` đỏ khi gỡ.
- [quick] `batch.rs` `complete_line`: khối CONTEXT chép lại trong khối SEGMENT lọt vào bản dịch — medium, patch (tiêu chí 4). `ContextOpen/Close` nay bật cùng cờ với TM; ca `a_context_block_copied_inside_a_segment_…` đỏ khi gỡ.
- [quick] `EmptyReply` ra `no_finish` thay vì `missing` — false: `no_finish` là một lý do của trạng thái `missing`, hàng hiện *thiếu kết quả AI* như Design Notes nói.
- [quick] `plan_chapter`: lượt đầy 100 câu vẫn mang ngữ cảnh khi đoạn đã dịch > 3 — false: đoạn > 3 tự nó bắt cắt lượt, và AD-60 mục 2 cho lượt cắt vì đoạn đã dịch mang ngữ cảnh.
- [quick] `ai_prompt_read_record`: rơi về bản ghi một câu của câu khác — false: `aiPromptRecordIsStale` gắn nhãn bản ghi lệch câu; `lib.rs:1721` luôn manage `AiCallRecordsState`.
- [quick] `AiUsageSummary.sentenceCount` nay đếm lượt — low, reject: kiểu dùng chung với Proofreader (đếm câu); khoá i18n riêng của dịch đã nói "lượt", đổi tên lan sang Proofreader và placeholder.
- [quick] `ResendCallSource` không kiểm câu về hưu — low, reject: tiêu chí 10 chỉ đòi kiểm 4–6; `promote_inner` vẫn từ chối ghi câu về hưu.
- [quick] Quy trình: chưa đọc kết quả đủ bộ, Implementation Notes rỗng, W3 lệch plan — reject (sửa plan): đủ bộ đã đọc và chạy nốt, notes đã ghi; CI và nightly e2e đọc trước `done`.
- [quick] `BlockStream` bỏ ký tự xuống dòng của dòng trống khi stream — low, reject: chỉ ảnh hưởng chữ tạm, `Received` ghi đè bằng chữ cuối.

## Design Notes

- Phép đọc khối là một bộ máy: `BlockStream` phục vụ cả mảnh chữ hiển thị (`Token`) lẫn kết quả cuối (`reconcile`), nên không có hai bộ đọc lệch nhau (Prevents 6). Chữ trong TM con, ngữ cảnh và ngoài khối không bao giờ thành `Token`.
- Prompt lắp **từng lượt, ngay trước khi gửi**: chỉ như vậy mới bỏ được câu về hưu mà vẫn đánh số n liên tục. Glossary/TM/system đọc một lần ở đầu thao tác, nên (u) đo được bằng cách đổi cấu hình giữa các lượt.
- `EmptyReply` (chỉ `[DONE]`) ở lô là lượt kết thúc không chữ: mọi câu thành `missing`, thao tác tiếp tục.
- ⚠️ `lib.rs` đổi chữ ký lệnh và danh sách lệnh nên cần một lần chạy đủ bộ; `npm run build` trước `cargo test`.

## Verification

**Commands:** `npm run build` -- xanh; các lệnh target ở Phase 2 và 3 -- xanh; `cargo test --test ai_boundary -- --ignored` -- đúng một ca chạy; `npm run check:gates` -- xanh nếu có thêm gate.
