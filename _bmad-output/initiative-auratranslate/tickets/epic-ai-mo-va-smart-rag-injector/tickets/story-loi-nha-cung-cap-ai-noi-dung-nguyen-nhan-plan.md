---
title: 'Story 4.13 — Lỗi nhà cung cấp AI nói đúng nguyên nhân'
type: 'feature'
ticket: 13
created: '2026-10-27'
status: 'done'
baseline_revision: '67c46a2db49971c1cfaee372a04fe30a7b668c81'
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
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `translate()` vứt thân phản hồi non-2xx nên mọi mã (402, 429, 5xx) ra cùng câu "kiểm lại endpoint và khoá API"; preset Gemini ghi bí danh `gemini-flash-latest`; "Kiểm tra kết nối" báo thiếu mô hình vì Gemini liệt kê id dạng `models/<id>`.

**Approach:** Thi hành AD-59 nguyên văn: client đọc tối đa 16 KiB thân lỗi, trích `message` từ JSON, che khoá, mang trong `NonSuccessStatus.provider_message`; Rust tách khoá lỗi theo mã trạng thái (một bảng cho ba đường); frontend hiện lời nhà cung cấp ở dòng riêng bằng nội suy văn bản. Yêu cầu: story 4-13, tiêu chí 1–7.

## Boundaries & Constraints

**Always:** Hình dạng `IpcError` không đổi; `provider_message` là tham số KHÔNG bắt buộc, không đưa vào chuỗi `vi.json` của khoá nào (AD-59 Rule 1, `required_params` giữ nguyên nghĩa). Che khoá nằm trong `client.rs` (Rule 3). `Display`/`Debug` không in `provider_message` (Rule 4). Retry chỉ do người dùng bấm (AD-22).

**Never:** Không `v-html` (AD-16). Không rẽ nhánh theo `error.code` của nhà cung cấp. Không đụng đường 2xx. Không thêm `#[tauri::command]`. Không sửa preset tám nhà còn lại hay cấu hình đã lưu.

**Decisions:** Đường Kiểm tra kết nối dùng cùng bảng câu theo mã như đường dịch (thêm khoá `test_*` cho 402/429/5xx) — story 4-13 tiêu chí 1 đã nói "cùng bảng này cho cả ba đường".
- 2026-10-27, Ice: 5xx `retryable: true`, 429 `retryable: false` (nút thử lại vẫn do người dùng bấm, AD-22).
- 2026-10-27, Ice: chấp nhận nguyên lời các câu ở Design Notes và nhãn "Lời nhà cung cấp: {message}".
- 2026-10-27, Ice: câu 429 và 5xx bỏ chữ "bạn" vì `check:i18n` Kiểm D (UX-DR47) cấm xưng hô: "…hết hạn mức hoặc gọi quá nhanh…", "…không phải do cấu hình đã lưu."; Design Notes đọc theo dòng này.
- 2026-10-27, Ice: giữ cả plan dù vượt trần 1600 token (~8,4k ký tự, chưa đo bằng tiktoken).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 402 có lời | JSON `[{"error":{"message":"…khoá…"}}]` | khoá `…payment_required`, `provider_message` đã che | khoá sót ≥12 ký tự ⇒ không có tham số |
| 502 HTML | thân HTML | khoá 5xx, không `provider_message` | chỉ câu chính |
| Id có tiền tố | `models/gemini-3.8-flash`, cấu hình `gemini-3.8-flash` | Connected | id thật sự vắng ⇒ ModelMissing |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/ai/client.rs` -- `NonSuccessStatus { status }` (~L111, Display ~L141); `translate()` trả nó ở ~L395 (có `request.api_key: &str`); `check_connection()` ở ~L544 (`request.api_key: Option<&str>`); `check_model_list` ~L488; mẫu đọc thân có trần: vòng `response.chunk()` + `MAX_MODEL_LIST_BYTES` ~L452/L563. Thêm `MAX_ERROR_BODY_BYTES = 16 KiB`, hàm thuần che khoá và hàm thuần trích `message`, một hàm đọc thân lỗi dùng chung hai nơi. `Display` giữ nguyên chỉ `status`.
- `src-tauri/src/commands/aitranslate.rs` -- `openai_client_error_family` ~L119 trả bộ ba cố định `(code, key, retryable)`; `From<OpenAiClientError> for IpcError` ~L159 và `batch_stopped_error` ~L185 đều chèn `status` rồi gọi hàm họ (chỗ chung duy nhất để thêm `provider_message` vào `params`); `connection_check_error` ~L749 (401|403, 404, còn lại). Hai bản chép tay `status` này nên gom về một hàm trợ giúp để ba đường dùng cùng bảng.
- `src-tauri/src/core/i18n/mod.rs` -- `message_keys!` ~L62: khoá khai tham số BẮT BUỘC; tham số không khai là tham số thừa và `IpcError::new` giữ nguyên (doc ~L1090), tức `provider_message` không cần cơ chế mới. `AiConfigTestRefused` ~L751, `AiTranslateProviderRefused` ~L897. Doc-comment "`params` cũng không được mang văn bản hiển thị" ~L1039 phải nêu ngoại lệ AD-59 theo tên.
- `src/i18n/vi.json` -- L113 `provider_refused`, L78–83 nhóm `test_*`. Thêm khoá mới ở đây; `ipc_contract.rs` (`every_message_key_exists_in_vi_json`, `every_message_key_declares_the_params_its_string_needs`) đối chiếu hai chiều.
- `src/aiConfigState.ts` L71–76 preset Gemini. Bốn chỗ hiện lỗi qua `tError`: `src/panels/AiTranslationPanel.vue` L551 (một câu), L712 và L722 (lô), `src/SettingsOverlay.vue` L795 (Kiểm tra kết nối). Mỗi chỗ cần thêm dòng `provider_message`.
- Tests: `src-tauri/tests/ai_translate_wire.rs` có máy chủ giả thật (`spawn_counted_sse_server`, chỉ trả 200), `save_key`, `Harness`, và gọi `wire::ai_translate_segment` qua đường thật; `connection_check_contract.rs` có `mod fake_server::reply(status_line, body)`; `ai_translate_contract.rs` ~L1837, L1894, L3526 và `connection_check_contract.rs` ~L136–148 khẳng định 401/500 ra `provider_refused`/`test_refused`, nay phải đổi. `tests/frontend/aiConfigState.test.ts` L813–853 ghim `gemini-flash-latest`.
- Gate: `check:i18n` quét chuỗi có dấu trong `.vue`/`src/**` (dòng kết quả `tError` đã có `aura-allow-text`); `src-tauri/tests/ai_boundary.rs` canh ranh giới `ai/` (chạy lại vì `client.rs` thêm hàm). Không gate nào quét `params` ngoài hai test `ipc_contract` trên.

## Tasks & Acceptance

**Execution:**
- [x] Phase 1 Rust, `client.rs` -- thêm `provider_message: Option<String>` vào `NonSuccessStatus`; hàm che (thay khoá đầy đủ bằng `***`, còn đoạn ≥12 ký tự ⇒ `None`, khoá rỗng bỏ qua); hàm trích ba hình dạng, bỏ ký tự điều khiển, gộp khoảng trắng, cắt 300 ký tự; đọc thân ≤16 KiB ở `translate()` và `check_connection()` -- AD-59 Rule 2–4
- [x] Phase 1, `client.rs::check_model_list` -- bỏ tiền tố `models/` khỏi id liệt kê trước khi so; giữ `:latest` -- tiêu chí 6
- [x] Phase 1, `i18n/mod.rs` + `vi.json` -- khoá mới theo mã (401/403, 402, 404, 429, 5xx) cho `ai_translate.*` và `ai_config.test_*`, khoá chung giữ `status`; nhãn "lời nhà cung cấp" có `{message}`; sửa doc-comment `IpcError` -- tiêu chí 1, 2
- [x] Phase 1, `aitranslate.rs` -- một hàm chọn khoá theo mã dùng chung cho `From`, `batch_stopped_error`, `connection_check_error`; chèn `provider_message` vào `params` chỉ khi có -- tiêu chí 1
- [x] Phase 2 Webview -- một hàm thuần đọc `params.provider_message` của `IpcError` (kiểm kiểu lúc chạy, chuỗi rỗng ⇒ `null`) và dòng nhãn + lời ở bốn chỗ trên, nội suy văn bản; `aiConfigState.ts` preset `gemini-3.8-flash` -- tiêu chí 2, 5
- [x] Phase 3 Tests that move, `ai_translate_wire.rs` -- máy chủ giả trả mã/thân tuỳ ý; ca 402 + khoá trong lời đi qua `wire::ai_translate_segment`; ca 502 HTML; ca mảng Gemini; test thuần hàm che (4 ca) và hàm trích -- tiêu chí 3, 4
- [x] Phase 3, sửa các khẳng định cũ ở `ai_translate_contract.rs`, `connection_check_contract.rs` (thêm ca `models/`, 402/429/5xx), `aiConfigState.test.ts`; vitest cho hàm đọc `provider_message` và dòng lời ở bốn chỗ
- [x] Phase 3, đối chứng đỏ -- GỠ thật lời gọi hàm che trong `translate()` (giữ chữ ký), chạy lại đúng `ai_translate_wire`, ca 402 phải đỏ vì khoá lộ; hoàn lại
- [x] Phase 3, `deferred-work.md` -- dòng 🔵 cho mục preset (Notes của story, giữ `Chủ: Epic 12`) và một mục mới `Chủ: Epic 4` (≤5 dòng) -- tiêu chí 7

**Acceptance Criteria:**
- Tiêu chí 1–7 của story 4-13 đều có ít nhất một ca chạm đúng đường thật (đếm ca, không chỉ xanh).
- Không trường nào của `IpcError`, `Display`, `Debug` hay log chứa khoá hoặc đoạn khoá.

## Design Notes

Câu đã chốt (Ice, 2026-10-27): 401/403 đường dịch "Nhà cung cấp AI từ chối khoá API (mã {status}) — kiểm lại khoá đã lưu rồi thử lại."; 404 đường dịch "Không tìm thấy endpoint hoặc mô hình (mã 404) — kiểm lại địa chỉ endpoint và tên mô hình."; đường Kiểm tra kết nối giữ hai câu 401/403 và 404 hiện có; 402 "Nhà cung cấp AI báo vấn đề thanh toán hoặc số dư (mã 402) — kiểm tài khoản còn tiền rồi thử lại."; 429 "Nhà cung cấp AI báo hết hạn mức hoặc bạn gọi quá nhanh (mã 429) — chờ một lúc rồi thử lại."; 5xx "Sự cố phía nhà cung cấp AI (mã {status}), không phải cấu hình của bạn."; chung "Nhà cung cấp AI từ chối yêu cầu này (mã {status})."; nhãn "Lời nhà cung cấp: {message}".

## Verification

**Commands (từng đích, không chạy cả bộ):**
- `cd src-tauri && cargo test --test ai_translate_wire --test connection_check_contract --test ai_translate_contract --test ipc_contract --test ai_boundary` -- expected: xanh (sau `npm run build`)
- `npx vitest run tests/frontend/aiConfigState.test.ts tests/frontend/aiTranslate.test.ts tests/frontend/aiTranslateBatch.test.ts` + file test mới -- expected: xanh
- `npm run check:i18n` -- expected: xanh


## Implementation Notes

- `provider_message` không khai trong `message_keys!`: `IpcError::new` chỉ kiểm tham số bắt buộc và giữ tham số thừa, nên tham số tuỳ chọn của AD-59 Rule 1 không cần cơ chế mới; nhãn có khoá riêng `ai.error.provider_message`.
- Ba chỗ từng tự chọn khoá theo mã (`From`, `batch_stopped_error`, `connection_check_error`) nay dùng chung `insert_status_params` và một bảng mã; hai khoá 401/403 và 404 của Kiểm tra kết nối cũng mang `provider_message` khi có.
- `Debug` của `OpenAiClientError` viết tay (`finish_non_exhaustive`) để `provider_message` không lọt vào chuỗi gỡ lỗi (Rule 4).
- Phép che nằm trong `read_provider_message`, dùng chung cho `translate()` và `check_connection()`; đối chứng đỏ gỡ lời gọi ở đó làm đỏ cả `ai_translate_wire` (2 ca) lẫn `connection_check_contract` (1 ca), đều vì khoá lộ.
- Câu 429/5xx đầu tiên có chữ "bạn", `check:i18n` Kiểm D (UX-DR47) chặn; đã viết lại không xưng hô (Ice, 2026-10-27).
- `models/` được bỏ ở cả id liệt kê lẫn mô hình đã lưu: chỉ bỏ một phía làm cấu hình `models/gemini-2.5-flash` từng khớp thành ModelMissing (review).
- Máy chủ giả là bằng chứng duy nhất cho dạng mảng và tiền tố `models/` của Gemini; lời thật của Google chờ lượt dùng thật, mục nợ `Chủ: Epic 4`.
## Review Triage Log

- quick · `client.rs` `check_model_list` chỉ bỏ `models/` ở id liệt kê, cấu hình đã lưu `models/gemini-2.5-flash` từng khớp nay ra ModelMissing · low · patch: bỏ tiền tố ở cả phía cấu hình, thêm một ca.
- quick · con trỏ mục nợ `Chủ: Epic 4` trỏ `story-…-nguyen-nhan.md §Notes` không tồn tại · false: tệp story có `## Notes` (dòng 72), dòng "Lượt rà 2026-10-27" ở dòng 76.
- quick · ô Tasks chưa đánh dấu, thiếu `## Implementation Notes` · rejected: sửa chính plan của lượt dựng; Implementation Notes ghi ở bước trình bày.
- quick · đối chứng đỏ chưa cho thấy đường `check_connection()` đỏ · false: đã chạy lại — gỡ `redact_api_key` trong `read_provider_message` (giữ chữ ký), `connection_check_contract` đỏ đúng một ca `http_402_and_429_…` vì khoá lộ (`sk-live-0123456789abcdef`), hoàn lại bản sao lưu.
