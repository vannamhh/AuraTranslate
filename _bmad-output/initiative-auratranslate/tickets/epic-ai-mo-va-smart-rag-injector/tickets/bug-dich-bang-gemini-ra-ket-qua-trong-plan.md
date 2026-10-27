---
title: 'Bug 4.14 — Dịch bằng Gemini ra kết quả trống, không báo lỗi'
type: 'bugfix'
ticket: 14
created: '2026-10-27'
status: 'built'
baseline_revision: '356340e10c83f86a7aada168334b681ab278a4f0'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `interpret_sse_event` trả đúng một kết quả cho mỗi khung và đọc `usage` trước `delta.content`, nên khung Gemini mang cả hai chỉ cho ra số token, chữ bị bỏ; hết luồng mà không có chữ nào thì không đường nào báo.

**Approach:** Một khung trả về cả chữ, `usage` và `finish_reason`; `translate()` giữ chữ và số token, và khi gặp `[DONE]` mà chưa có chữ nào thì trả lỗi `EmptyReply` đi qua họ lỗi sẵn có (AD-21). Yêu cầu: ticket 14, tiêu chí 1–3.

## Boundaries & Constraints

**Always:** Luồng OpenAI (`usage` chỉ ở khung cuối, `choices: []`) và luồng không `usage` cho kết quả như trước. Số token sau lượt gọi là của khung `usage` cuối. Không tự thử lại (AD-22). Một lỗi một `message_key` (AD-21); lỗi rỗng đi cùng đường `From<OpenAiClientError>` và `batch_stopped_error`, nên lô dừng và nêu tên câu không cần mã mới.

**Never:** Không đổi `TranslateOutcome`, không thêm `#[tauri::command]`, không đổi hình dạng `IpcError`. Không đụng đường lỗi non-2xx của 4.13. Không lọc theo tên nhà cung cấp.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Khung Gemini | mỗi khung có `delta.content` + `usage` | từng đoạn chữ qua `on_token` đúng thứ tự; usage cuối thành `Done(Some)` | không lỗi |
| Kiểu OpenAI | `usage` ở khung cuối, `choices: []` | như trước | không lỗi |
| Rỗng | chỉ khung `role`/`thought_signature` rồi `[DONE]`, `finish_reason` `content_filter` | `Err(EmptyReply)`, tham số `finish_reason` | một câu: lô dừng tại câu đó; không ghi Editor |
| Rỗng, không lý do | `[DONE]` ngay | `Err(EmptyReply)` không tham số | khoá không-lý-do |
| Chỉ khoảng trắng | các token toàn `\n` | tính là rỗng | như trên |

**Decisions (Ice, 2026-10-27):**
- Hai câu báo rỗng: không lý do "Mô hình đã trả lời xong nhưng không có chữ nào — chưa có bản dịch."; có lý do "Mô hình đã trả lời xong nhưng không có chữ nào (lý do dừng: {finish_reason}) — chưa có bản dịch."; `finish_reason` hiện nguyên văn nhà cung cấp gửi (`stop`, `length`, `content_filter`), không dịch.
- Lỗi rỗng `retryable: false`.
- Giữ cả plan dù vượt trần 1600 token (~7,7 KB, chưa đo bằng tiktoken).

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/ai/client.rs` -- `ChatCompletionsChunk`/`ChunkChoice`/`ChunkDelta` ~L367–381 (thêm `finish_reason` ở `ChunkChoice`); `ChunkUsage::has_any_token_count` ~L420 (giữ); `translate()` vòng đọc ~L525–567; `SseEventOutcome` ~L724 và `interpret_sse_event` ~L742 (sửa); `OpenAiClientError` ~L98 (thêm `EmptyReply { finish_reason: Option<String> }`, kiểm `Debug`/`Display` viết tay ~L134–165). Doc-comment "usage thắng" ở ~L734 và ~L361 sai sau sửa: xoá ở dòng đang sửa.
- `src-tauri/src/commands/aitranslate.rs` -- `openai_client_error_family` ~L119 (thêm họ, `retryable: false`: thử lại không chắc đổi kết quả, cùng `reply_unreadable`; nút thử lại vẫn do người dùng bấm); `insert_status_params` ~L217 (thêm `finish_reason` khi có). `From`/`batch_stopped_error` dùng chung, không sửa. `proofread.rs` ~L617 cũng qua `IpcError::from`, tự có câu mới.
- `src-tauri/src/core/i18n/mod.rs` ~L900–930 -- `message_keys!`: thêm `AiTranslateReplyEmpty` (`[]`) và `AiTranslateReplyEmptyWithReason` (`["finish_reason"]`); `src/i18n/vi.json` ~L121–131 cùng nhóm `err.ai_translate.*`.
- `src-tauri/tests/ai_translate_contract.rs` -- import L69; ca `a_frame_carrying_both_usage_and_delta_content_reads_as_usage_and_discards_the_content` ~L489 GHIM ĐÚNG HÀNH VI SAI, phải viết lại; ca Token/Usage ~L395–515 đổi theo hình mới; ca `StreamEndedWithoutDone` ~L1924 là mẫu cho ca khoá lỗi.
- `src-tauri/tests/ai_translate_wire.rs` -- máy chủ giả `spawn_counted_sse_server` L71; `Harness::with_error_reply` L160 gửi `fixed_reply` nguyên văn cho mọi mã, kể cả `200 OK` với thân SSE, nên KHÔNG cần sửa Harness; `single_run_error` ~L867, ca lô ~L983 làm mẫu.
- Webview: `tError` hiện mọi `IpcError` qua `message_key`, nên không cần đổi `src/`; `retryable` chỉ chặn lượt thử lại trong `src/aiTranslateHandlers.ts` (L129, L155), không đổi cách hiện lỗi. Không có Phase Webview.

## Tasks & Acceptance

**Execution:**
- [x] Phase 1 Rust, `client.rs` -- `interpret_sse_event` trả `SseEventOutcome::Chunk(SseChunk { text, usage, finish_reason })` khi khung mang ít nhất một thứ, `Ignore` khi không; `usage` rỗng vẫn bị bỏ; `translate()` gọi `on_token` trước rồi ghi `last_usage`, `last_finish_reason`, cờ đã-có-chữ (chữ không toàn khoảng trắng); ở `Done` mà cờ tắt ⇒ `Err(EmptyReply)` -- tiêu chí 1, 2
- [x] Phase 1, `aitranslate.rs` + `i18n/mod.rs` + `vi.json` -- họ lỗi, hai khoá, tham số `finish_reason`; câu đã chốt ở Decisions -- tiêu chí 2
- [x] Phase 2 Tests, `ai_translate_contract.rs` -- viết lại ca "cả hai", thêm ca nhiều khung chữ+usage đúng thứ tự, ca `finish_reason` ở khung không chữ, ca họ lỗi của `EmptyReply` (code, key, `retryable`, `finish_reason` chỉ khi có) -- tiêu chí 1, 2
- [x] Phase 2, `ai_translate_wire.rs` -- ca Gemini qua `wire::ai_translate_segment` với `with_error_reply("200 OK", <ba khung Reproduction + [DONE]>)`: kênh nhận đủ chữ, `target_text`/usage đúng; ca `[DONE]` không chữ ⇒ `Err` đúng khoá và `finish_reason`; ca lô: dừng ở câu đầu, tham số `segment_id` -- tiêu chí 3
- [x] Phase 2, đối chứng đỏ -- GỠ thật: đưa lại "có `usage` thì bỏ qua `content`" trong `interpret_sse_event` (giữ chữ ký), chạy lại đúng `ai_translate_wire`, ca Gemini phải đỏ vì thiếu chữ; hoàn lại; ghi một dòng vào Implementation Notes -- tiêu chí 3

**Acceptance Criteria:**
- Mỗi tiêu chí 1–3 có ít nhất một ca chạm đường thật (đếm ca, không chỉ xanh).
- `ipc_contract` xanh: hai khoá có trong `vi.json`, tham số khớp.

## Design Notes

Gemini gửi `usage` kèm mọi khung; số token cuối thắng vì `last_usage` bị ghi đè. `finish_reason` có thể nằm ở khung không chữ nên giữ khung cuối có giá trị.

## Verification

**Commands (từng đích, không chạy cả bộ):**
- `npm run build` rồi `cd src-tauri && cargo test --test ai_translate_contract --test ai_translate_wire --test ipc_contract --test ai_boundary --test ai_proofread_contract` -- expected: xanh
- `npm run check:i18n` -- expected: xanh (Kiểm D cấm "bạn")
- `npx vitest run tests/frontend/aiTranslate.test.ts tests/frontend/aiTranslateBatch.test.ts` -- expected: xanh, không sửa (xác nhận frontend không đổi)

## Implementation Notes

- Nguyên nhân đo bằng khoá thật: Gemini gửi `usage` kèm mọi khung có chữ, và khung cuối chỉ mang `extra_content.google.thought_signature` cùng `finish_reason`; một ca test cũ của Story 4.11 khoá đúng hành vi bỏ chữ, đã viết lại.
- `SseEventOutcome` còn `Ignore | Done | Chunk(SseChunk)`; một khung trả cả chữ, số token và `finish_reason`, `translate()` gửi chữ trước rồi mới ghi số token.
- Rỗng = không có chữ nào khác khoảng trắng tới `[DONE]` ⇒ `EmptyReply`, `retryable: false`, hai khoá có/không `finish_reason` (nguyên văn, Ice).
- Proofreader giữ cách cũ (Ice chọn b): `TranslationProvider::is_empty_reply` mặc định `false`, `OpenAiChatClient` ghi đè; lô rỗng vào `not_scanned` rồi quét tiếp. Gỡ nhánh đó thì ca `an_empty_reply_error_loses_only_its_own_batch_and_the_run_continues` đỏ vì lượt rà dừng.
- Commit 4.13 chưa qua rustfmt; diff này định dạng lại hai hàm của 4.13 để `check:fmt` xanh.
- Đối chứng đỏ: đưa lại "có `usage` thì `text = None`" trong `interpret_sse_event` làm đỏ 2 ca contract (giữ chữ + usage; nhiều khung đúng thứ tự) và ca wire Gemini (luồng có chữ trả `EmptyReply` thay vì Ok); đã hoàn lại từ bản sao.

## Plan Change Log

## Review Triage Log

- quick · hàng "Chỉ khoảng trắng" của ma trận chưa có ca nào · low · patch: thêm `a_stream_of_only_whitespace_text_fails_with_empty_reply`; bỏ `.trim()` thì ca đó đỏ, đã hoàn lại.
- quick · định dạng lại `extract_provider_message`/`read_provider_message` của 4.13 · false: `check:fmt` (cổng pre-push) đòi đúng định dạng này; commit 4.13 chưa qua rustfmt.
- quick · Proofreader: lô trả rỗng trước kia bị bỏ qua (`not_scanned`, quét tiếp), nay `EmptyReply` làm dừng cả lượt rà · medium · patch theo Ice (b, 2026-10-27): thêm `TranslationProvider::is_empty_reply`, lô rỗng vào `not_scanned` rồi quét tiếp; ca mới trong `ai_proofread_contract`, đối chứng đỏ khi gỡ nhánh.
- quick · `finish_reason` hiện nguyên văn, không che, không cắt · rejected: giá trị dạng mã ngắn (`stop`, `length`, `content_filter`), Ice chọn nguyên văn; thêm che/cắt là thêm guard cho trạng thái chưa thấy.
- quick · `Display` in `finish_reason` · rejected: không đường nào ghi `OpenAiClientError` ra log.
- quick · ô Tasks chưa đánh dấu, tệp handoff được commit · rejected: sửa plan ở bước trình bày; handoff commit theo lệ của các story Epic 4.
- quick · đối chứng đỏ ca wire đỏ vì `EmptyReply` chứ không vì thiếu chữ · false: `EmptyReply` chính là hệ quả của thiếu chữ sau bản sửa.
