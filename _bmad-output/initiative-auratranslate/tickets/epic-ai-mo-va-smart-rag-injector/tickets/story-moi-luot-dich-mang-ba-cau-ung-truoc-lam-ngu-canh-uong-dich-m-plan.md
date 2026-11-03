---
title: 'Story 4.16 — Mọi lượt dịch mang ba câu đứng trước làm ngữ cảnh, đường dịch một câu đọc theo khối'
type: 'feature'
ticket: 16
created: '2026-11-03'
status: 'built'
baseline_revision: '4d0973350e74b6257f315498e546c758cdd11254'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Chỉ dịch cả Chương mới có câu kề, và chỉ ở khe. Dịch một câu, vùng chọn và thử lại không mang câu kề nào. Đường một câu đọc chữ thường, nên chữ ngữ cảnh mà mô hình chép lại sẽ lọt vào panel.

**Approach:** Thi hành AD-65. `plan_calls` thêm tối đa `CONTEXT_BEFORE = 3` câu đứng trước vào đầu mọi lượt. Đường một câu lập kế hoạch như vùng chọn một câu, lắp bằng `assemble_call`, đọc bằng `BlockStream`, và chỉ chuyển chữ của khối 1 qua Channel. Story 4.16, tiêu chí 1–10.

## Boundaries & Constraints

**Always:**
- Ghép mọi mảnh Channel lại thì ra đúng chữ của khối 1 khi khối đóng: không có dấu khối, không có chữ ngoài khối.
- `maybe_truncated` = khối 1 chưa đóng, hoặc `finish_reason` khác `Some("stop")`.
- *Thiếu kết quả AI* trên đường một câu là `Err(IpcError)` với mã mới `ai_translate.result_missing`. Khoá `err.ai_translate.result_missing` = "Thiếu kết quả AI: mô hình có trả lời nhưng không có khối dịch dùng được — chưa có bản dịch." Thử lại do người dùng bấm. Mã này làm webview xoá chữ đã hiện.
- Xem trước (`ai_prompt_assemble`), dịch một câu và gửi lại dùng cùng một hàm lắp, nên prompt xem trước trùng byte với prompt gửi đi.

**Never:**
- Đổi lệnh, các trạng thái của panel, `LastAssembledPromptState`/`AssembledPromptRecord` hay `AiTranslateOutcomeWire`.
- Đổi cách cắt theo khe (AD-60 mục 2, ca (k)), luật đối chiếu của lô, bộ ba biến prompt, system prompt mặc định, Proofreader, dịch nhanh hay `client.rs` (`EmptyReply` giữ nguyên).
- Lùi về đọc chữ thường, tự gửi lại, hay ghi `target_text`.

## I/O & Edge-Case Matrix (đường một câu)

| Luồng trả về | Kết quả |
|---|---|
| Có khối ngữ cảnh chép lại + khối 1 đóng, `stop` | `Done`, chữ khối 1, `maybe_truncated` tắt |
| Khối 1 đóng, không có `finish_reason` | `Done`, `maybe_truncated` bật |
| `[DONE]` khi khối 1 đang mở và đã có chữ | `Done`, chữ đã nhận, `maybe_truncated` bật |
| Khối 1 mở mà chưa có chữ / không có khối 1 / khối 1 hai lần / khối 1 rỗng | `Err(result_missing)`, panel không còn chữ |
| Không có chữ nào | `Err(reply_empty…)` như bug 4.14 |
| Luồng đóng mà không có `[DONE]`, hoặc non-2xx | lỗi nhà cung cấp như hôm nay |
| Huỷ | `Cancelled`, chữ khối 1 đã hiện vẫn còn |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/ai/batch.rs` -- `plan_calls` :183, `PlanSegment` :140 (không có chữ), `PlanScope` :148 (doc "never with context" sai từ nay), `plan_selection` :199 (chia theo đoạn rồi `chunks(100)`), `plan_chapter` :221 (khe ≤3 ⇒ Context trong lượt, >3 ⇒ lượt mới với 3 câu cuối khe :234-241; trần :242 không câu kề). `CallEntry::{Context,Translate}` :156. `guard_text` :128. `BlockStream::push → Vec<TextPiece{index,text}>` :320 (tự giấu dòng có thể là dấu, bỏ chữ trong CONTEXT/TM), `finish → Vec<ParsedBlock{end: Closed|Superseded|StreamEnd}>` :340. `reconcile` :504 dùng cho lô, đừng ép đường một câu đi qua nó.
- `src-tauri/src/core/ai/rag.rs` -- `CallPromptEntry::Context{source_text,target_text}` :805; `assemble_call_prompt` :839-845 luôn in `nguồn → đích`. `assemble_prompt` :741 sau story không còn ai gọi trên đường dịch.
- `src-tauri/src/commands/aiprompt.rs` -- `assemble_and_record_prompt(_with_tm)` :494/:517 là hàm lắp chung của `prepare_translate_call` (aitranslate.rs:426) và `ai_prompt_assemble` (:1008); hôm nay gọi `assemble_prompt` :561, system không có hậu tố khối :562. `CallSpec`/`assemble_call` :740-785 (Context đi thẳng, không quét). `mark_prompt_as_sent` :644 so `segment_id`+prompt+system.
- `src-tauri/src/commands/aitranslate.rs` -- `SnapshotRow` :505; `prepare_translate_operation` :557 (plan :588-603) và `OperationCallSource::prepare_call` :1405-1454 dựng Context từ ảnh chụp, là khuôn cho đường một câu. `run_translate_call` :667 chuyển token thô :682-686. Lô lọc mảnh theo index ở :1144-1161. `single_run_outcome_wire` :1635. Gửi lại một câu: `prepare_resend_call` :452 + `wire::ai_translate_resend` :1777 (token thô). Họ lỗi `reply_empty` :174-182.
- `src-tauri/src/core/i18n/mod.rs` :917 -- thêm `AiTranslateResultMissing`; `src/i18n/vi.json` :133.
- `src/aiTranslateState.ts` :139-170 -- `error` vẫn để `accumulatedText` hiện ở thẻ kết quả (`AiTranslationPanel.vue` :619).
- `src/aiPromptInspectorState.ts` :447-480 -- `aiPromptCallNote` cho `none` khi `call === null`; `aiPromptHighlightParts` chỉ tô khối SEGMENT, không nhận ra CONTEXT. `AiPromptInspectorOverlay.vue` :497.
- Test: `ai_batch_wire.rs` (`Harness` :297, `Reply` :65, `echo_responder` :450, `translate_ids` :427, AD-60 (i) :749, (k) :777, (m) :831, một câu không `finish_reason` :1166). `ai_translate_wire.rs` (`spawn_counted_sse_server` :71 trả "Xin chao" thô, `sse_body` :798). `ai_translate_contract.rs` dùng `FakeProvider`, không có cổng nghe. `core/promptset/vars.rs:195` ca (l).

## Tasks & Acceptance

**Execution:** mỗi phase một agent mới, bàn giao qua `## Implementation Notes`.
- [x] Phase 1 Rust, `batch.rs` -- `CONTEXT_BEFORE = 3`. Sau khi chia lượt, mỗi lượt có ở đầu đúng tối đa 3 câu không `is_omitted` đứng ngay trước Translate đầu tiên của nó. Kết quả cắt theo khe phải giữ nguyên. Sửa doc `PlanScope`. -- tiêu chí 1, 3, 4
- [x] Phase 1, `rag.rs` -- Context có `target_text` rỗng sau trim thì in nguồn đã `guard_text`, không có `→`. -- tiêu chí 2
- [x] Phase 1, `aiprompt.rs` -- `assemble_and_record_prompt_with_tm` đọc ảnh chụp Chương, `plan_calls(Selection{id})`, rồi `assemble_call` (Context lấy từ ảnh chụp). `system` = `system_with_block_format(resolved)`. Bản ghi giữ đúng các trường. -- tiêu chí 1, 7
- [x] Phase 1, `aitranslate.rs` -- một bộ đọc khối dùng chung cho `run_translate_call` và `ai_translate_resend`: dựng `BlockStream`, chuyển mảnh `index == 1` của lần mở đầu tiên của khối 1, phán theo bảng I/O, trả `result_missing`. `single_run_outcome_wire` nhận cờ chưa đóng. Thêm mã và khoá i18n. -- tiêu chí 5, 6, 7
- [x] Phase 2 Webview, `aiTranslateState.ts` -- lỗi `ai_translate.result_missing` ⇒ `accumulatedText = ''`. `aiPromptInspectorState.ts` + overlay -- `call === null` thì tô khối 1. Khối `<<<CONTEXT` … `CONTEXT>>>` được đánh dấu riêng ở mọi bản ghi, kèm một khoá i18n "Câu kề (chỉ đọc)". -- tiêu chí 6, 7
- [x] Phase 3 Tests -- thêm ca (a)–(k) của AD-65 mục 8 vào `ai_batch_wire.rs` (đường một câu, vùng chọn, cả Chương, thử lại) và `ai_translate_wire.rs` (gửi lại), cùng một ca cho tiêu chí 8 (`{{chapter_context}}` vẫn là marker lạ, câu kề vẫn đi). Ca (i) của AD-60 viết lại thành ca (f). Các ca một câu hôm nay trả chữ thô thì đổi sang khung có khối, trừ ba ca trả lời rỗng :873/:900/:921. Đổi `ai_translate_contract.rs` cho khớp. Vitest: `aiTranslate.test.ts` (xoá chữ khi `result_missing`, giữ chữ khi lỗi mạng) và `aiPromptInspector.test.ts`. -- tiêu chí 9
- [x] Phase 3, đối chứng đỏ -- phép GỠ ghi một dòng mỗi ca trong Implementation Notes:
  - (a) gỡ bước thêm câu kề
  - (b) gỡ lọc khối, chuyển token thô
  - (c) gỡ nhánh khối chưa đóng ⇒ partial
  - (d) gỡ trần 3, chọn Chương có 5 câu; nửa ranh Chương: gỡ lọc `chapter_id` trong `select_chapter_segments` (giữ chữ ký)
  - (e) gỡ nhánh chưa dịch
  - (f) gỡ phép tách đoạn trong `plan_selection`
  - (g) gỡ câu kề riêng của `Selection`
  - (h) gỡ câu kề ở chỗ cắt vì trần
  - (i) cho Context đi qua quét Glossary
  - (j) gỡ `guard_text` trên dòng câu kề
  - (k) gỡ bộ đọc khối trên đường gửi lại
  - vitest: gỡ lệnh xoá chữ
  - (l) là test có sẵn, không đổi.
  -- tiêu chí 9
- [x] Phase 3, `deferred-work.md` -- một mục `Chủ: Epic 4` (tiêu chí 10): Ice dịch thật một câu bằng Ollama/LM Studio và bằng preset, mở Xem prompt thấy 3 câu kề và kết quả không có chữ câu kề; đối chiếu token vào trước/sau. -- tiêu chí 10

**Acceptance Criteria:**
- Given tiêu chí 1–10 của story, when bộ test của Verification chạy, then mọi ca xanh, và mỗi phép gỡ ở Phase 3 làm đúng ca của nó đỏ vì lý do được đo.

## Design Notes

- Đánh dấu câu kề làm ở webview, không thêm `context_segment_ids` vào bản ghi một câu. Lý do: AD-65 mục 5 giữ nguyên `LastAssembledPromptState`, và dấu `<<<CONTEXT` đã nằm sẵn trong prompt đã ghi.
- Khối 1 xuất hiện lần hai thì ngừng chuyển chữ của nó. Kết quả cuối là `result_missing`, và webview xoá chữ đã hiện.

## Verification

**Commands:**
- `npm run build` rồi `npm run test:story 4-16` -- chọn `ai_batch_wire.rs`, `ai_translate_wire.rs`, `ai_translate_contract.rs`, `ai_rag_contract.rs`, `ai_prompt_contract.rs`, `tests/frontend/aiTranslate.test.ts`, `tests/frontend/aiPromptInspector.test.ts`, `tests/frontend/aiPromptInspectorScreen.test.ts` -- expected: xanh
- `cargo test --lib promptset::vars` -- expected: ca (l) xanh

## Implementation Notes

- `plan_calls` giữ cắt theo khe: `with_leading_context` bỏ Context đầu lượt rồi thêm tối đa 3 câu hiển thị đứng trước Translate đầu tiên. Câu `is_omitted` không tính.
- `assemble_and_record_prompt_with_tm` dựng `CallPromptEntry` trực tiếp thay vì `assemble_call`, để giữ `TmRowsCache` và quét Glossary từng câu; kết quả cùng byte. Hệ quả: câu kề được quét ở hai chỗ (`aiprompt.rs` đường một câu, `assemble_call` đường lô) và test (i) khoá cả hai.
- Đường một câu: `run_translate_call` trả `SingleRun`/`SingleRunError`; chỉ chuyển mảnh `index == 1` của lần mở đầu tiên. Khối 1 `Superseded`, trùng, rỗng hoặc vắng ⇒ `ai_translate.result_missing`; chỉ `StreamEnd` có chữ ⇒ `Done` với `maybe_truncated`. Hai kiểu này nay `derive(Debug, PartialEq)` để test so sánh được.
- Gửi lại dùng chung `send_prepared_translate_call` với dịch một câu, nên một bộ đọc khối phục vụ cả hai; ca (k) khoá riêng nhánh gửi lại.
- Test cũ trả chữ thô đã đổi sang khung khối (`ai_translate_contract`, `ai_translate_wire`, `ai_prompt_contract`: system có hậu tố khối, prompt bọc `SEGMENT 1`). Ba ca trả lời rỗng giữ nguyên. Ca AD-60 (i) viết lại thành (f). Nửa ranh Chương dùng chương thứ hai dựng bằng SQL trong test và `OpenWork.chapter_id`.
- Phần tô câu kề ở webview đã có trước Phase 3; vitest thêm `aiPromptHighlightBlockNumber` (call null ⇒ khối 1) và khối CONTEXT có `context: true` kể cả khi không có số khối.
- Đối chứng đỏ (mỗi phép GỠ làm đúng ca của nó đỏ, khôi phục rồi chạy lại xanh):
  - (a) gỡ `.map(with_leading_context)` ⇒ 9 ca `ai_batch_wire` + ca gửi lại đỏ vì không có khối CONTEXT.
  - (b) gỡ lọc khối, chuyển token thô ⇒ `a_context_block_the_model_copies…`, `a_single_sentence_stream_cut…`, ca maybe_truncated đỏ vì marker lọt kênh.
  - (c) bỏ nhánh khối chưa đóng ⇒ chỉ `a_single_sentence_stream_cut_inside_block_one…` đỏ; bỏ luôn vế `Superseded` ⇒ chỉ ca `…without_a_usable_block_one…` đỏ.
  - (d) `from = 0` (gỡ trần 3) ⇒ 4 ca đỏ, kể cả Chương 5 câu dịch; `select_chapter_segments` bỏ `chapter_id = ?1` (giữ `?1`) ⇒ `a_neighbour_never_comes_from_another_chapter` đỏ với 3 câu kề từ Chương 1.
  - (e) luôn in `nguồn → đích` ⇒ `an_untranslated_neighbour…` và các ca đếm khối kề đỏ; `ai_rag_contract` vẫn xanh nên chỉ `ai_batch_wire` canh nhánh này.
  - (f) gỡ phép tách đoạn trong `plan_selection` ⇒ `a_selection_of_two_separate_ranges…` đỏ (một lượt 4 khối).
  - (g) dời `with_leading_context` vào nhánh `Chapter` của `plan_calls` ⇒ `retrying_a_missing_sentence…` đỏ, lượt thử lại không còn câu kề nào.
  - (h) bỏ câu kề ở lượt cắt vì trần ⇒ `a_chapter_run_gives_the_first_call…` đỏ ở lượt 2 và 3.
  - (i) cho Context đi qua quét Glossary, một lần ở `aiprompt.rs` và một lần ở `assemble_call` ⇒ `a_context_neighbour_is_never_scanned…` đỏ cả hai lần (ca ban đầu chỉ chạy đường lô nên không bắt được đường một câu, đã thêm `run_single`).
  - (j) bỏ `guard_text` trên dòng câu kề ⇒ `a_neighbour_line_that_equals_a_block_marker…` đỏ.
  - (k) gửi lại gọi nhà cung cấp trực tiếp, không qua bộ đọc khối ⇒ hai ca `resending_a_single_sentence…` đỏ, ca dịch một câu vẫn xanh.
  - vitest: gỡ dòng `accumulatedText.value = ''` ⇒ ca `result_missing` đỏ (`'Chu cua khoi bi bo'` còn lại); ca (l) là test có sẵn, chạy xanh.
- Nợ: một mục `Chủ: Epic 4` trong `deferred-work.md` cho lượt dùng thật một câu bằng Ollama/LM Studio và preset (tiêu chí 10). `tm_contract.rs` và `plan-dien-cau-trung-ngay-sau-khi-xac-nhan.md` thuộc việc khác, không đụng.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 6 phát hiện. Kết quả: 1 medium, 1 maybe-false, 3 low, 1 false.
- medium · patch · Xem trước câu `is_omitted` ghi một prompt không có câu đó, vì `plan_calls` bỏ câu đã cắt; đã vá bằng một khối `Segment` đứng một mình, kèm ca `previewing_an_omitted_segment_still_records_its_sentence_alone` và phép gỡ đỏ.
- maybe-false · defer · Lượt ra `result_missing` không đóng dấu `sent_at`/usage. Mọi `Err` sau khi đã gửi (`reply_empty`, đứt luồng) đều như vậy từ trước; cần Ice quyết lượt lỗi đã tốn tiền có phải hiện là "đã gửi" hay không.
- low · reject · Ô task chưa tick, Implementation Notes dài; sửa ở plan nên không thuộc review. Đã tick khi đóng bước.
- low · reject · `aiprompt.rs` dựng `CallPromptEntry` bằng tay thay vì gọi `assemble_call`; đã ghi trong Implementation Notes, test (i) khoá cả hai đường. Sửa cần tái cấu trúc.
- false · reject · Gọi `resolve_two_tiers` hai lần: hai hàm khác nhau (`core::promptset` và `core::aiconfig`).
- low · reject · Thân bộ prompt có dòng `<<<CONTEXT` do người dùng gõ sẽ bị đánh dấu là câu kề; người dùng khó gặp, và sửa phải thêm tham số.
