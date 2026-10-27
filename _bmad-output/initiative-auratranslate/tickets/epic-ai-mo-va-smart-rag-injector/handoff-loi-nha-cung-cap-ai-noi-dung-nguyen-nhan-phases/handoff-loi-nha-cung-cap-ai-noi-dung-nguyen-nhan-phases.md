# Handoff story 4-13 — Phase 1 (Rust) xong

## Đã đổi
- `src-tauri/src/core/ai/client.rs`: `NonSuccessStatus { status, provider_message: Option<String> }`; `Debug` của `OpenAiClientError` viết tay (không in `provider_message`), `Display` chỉ `status`. Hàm `pub`: `redact_api_key(text, api_key) -> Option<String>`, `extract_provider_message(body: &[u8]) -> Option<String>`; private `read_provider_message(&mut Response, api_key)` (trần `MAX_ERROR_BODY_BYTES` 16 KiB, vượt/đọc lỗi ⇒ None) gọi ở `translate()` và `check_connection()`. Thứ tự: trích (bỏ điều khiển, gộp khoảng trắng, cắt 300) rồi che khoá. `check_model_list` bỏ tiền tố `models/`.
- `src-tauri/src/commands/aitranslate.rs`: `openai_client_error_family` rẽ theo mã (401|403, 402, 404, 429, 5xx retryable=true, còn lại chung); hàm chung `insert_status_params` (status + `provider_message` khi có) cho `From`, `batch_stopped_error`; `connection_check_error` viết lại: 401/403 và 404 giữ khoá cũ không `status` nhưng nay mang `provider_message` nếu có; 402/429/5xx/khác mang `status` (+`provider_message`).
- `core/i18n/mod.rs`: khoá mới `AiTranslateProviderKeyRejected|PaymentRequired|EndpointNotFound|RateLimited|ServerError` (`err.ai_translate.provider_key_rejected|payment_required|endpoint_not_found|rate_limited|server_error`, param `status`), `AiConfigTestPaymentRequired|RateLimited|ServerError` (`err.ai_config.test_payment_required|rate_limited|server_error`, param `status`). Doc-comment `IpcError` nêu ngoại lệ AD-59.
- `src/i18n/vi.json`: các khoá `err.*` trên; `err.ai_translate.provider_refused` thành câu chung; nhãn Webview: `ai.error.provider_message` = "Lời nhà cung cấp: {message}" (tham số `{message}`).
- Tests: chỉ thêm `, provider_message: None` vào các `NonSuccessStatus { status: N }` trong `tests/ai_translate_contract.rs`, `tests/connection_check_contract.rs` để biên dịch.

## Phase 2 cần
- Đọc `error.params.provider_message` (có thể vắng); dòng nhãn qua `ai.error.provider_message` với `{message}`; bốn chỗ: AiTranslationPanel.vue (L551, L712, L722), SettingsOverlay.vue L795. Preset Gemini `gemini-3.8-flash` trong `src/aiConfigState.ts`.
- Lưu ý: ở đường Kiểm tra kết nối, 401/403 và 404 cũng có thể mang `provider_message`.

## Test đã chạy (cargo, sau npm build có sẵn dist/)
- Xanh: `ai_boundary` (29), `ipc_contract` (48), `ai_translate_wire` (16).
- Đỏ ĐÚNG DỰ KIẾN, Phase 3 sửa khẳng định cũ: `ai_translate_contract` (3 ca: `batch_stopped_error_names_the_sentence_...`, `every_openai_client_error_variant_maps_to_its_own_family_key_...`, `provider_returns_a_non_2xx_maps_to_a_non_retryable_...` — 401/500 nay ra khoá theo mã, 500 retryable); `connection_check_contract` (2 ca: `every_failure_family_has_its_own_message_key`, `http_404_500_and_html_each_get_their_own_family` — 500 nay ra `test_server_error`).
- Chưa viết: test thuần hàm che/trích, ca máy chủ giả 402/502/mảng Gemini, đối chứng đỏ, `deferred-work.md` (Phase 3).

# Phase 2 (Webview) xong
## Đã đổi
- `src/i18n/index.ts`: `providerMessageOf(err: IpcError | null): string | null` (kiểm kiểu lúc chạy, rỗng/vắng/không phải chuỗi ⇒ null).
- Dòng lời nhà cung cấp `t('ai.error.provider_message', { message })`: `AiTranslationPanel.vue` (một câu: sau `<br />` trong cùng `<p>`; lô: `<p data-ai-batch-error-provider-message>` sau khối nguyên nhân, hiện ở cả nhánh stopped lẫn nhánh đơn), `SettingsOverlay.vue` (Kiểm tra kết nối, sau `<br />`). Chuỗi chèn bằng nội suy văn bản, không v-html.
- `src/aiConfigState.ts` preset Gemini `gemini-3.8-flash`.
## Phase 3 cần
- `tests/frontend/aiConfigState.test.ts` hai ca (~L813-853) còn ghim `gemini-flash-latest` ⇒ đỏ đúng dự kiến; sửa.
- Vitest mới cho `providerMessageOf` và dòng lời ở bốn chỗ (selector `data-ai-batch-error-provider-message`; panel một câu/Settings dùng `<br />` không có data-attr riêng, kiểm bằng text).
- 🔴 `npm run check:i18n` Kiểm D ĐỎ 4 khoá do Phase 1 (câu Ice đã chốt chứa "bạn", UX-DR47 đòi vô nhân xưng): `err.ai_config.test_rate_limited`, `err.ai_config.test_server_error`, `err.ai_translate.provider_rate_limited`, `err.ai_translate.provider_server_error`. Chưa sửa vì là chữ Ice duyệt nguyên lời; cần Ice chọn: đổi chữ ("gọi quá nhanh" / "không phải cấu hình đã lưu") hoặc đổi cổng.
- vue-tsc sạch, eslint 3 tệp sạch.

# Phase 3 (Tests) xong
## Đã đổi
- `tests/ai_translate_wire.rs`: `spawn_counted_sse_server` nhận `fixed_reply`, `Harness::with_error_reply`; 6 ca đường thật (402 + khoá trong lời ⇒ `***`; mảnh khoá ≥12 ký tự ⇒ không `provider_message`; 502 HTML; mảng Gemini; thân >16 KiB; lô 429) + `mod provider_message_pure` (7 ca che/trích).
- `ai_translate_contract.rs`: bảng mã→khoá (7 hàng), `provider_message` chỉ khi có ở đường đơn và lô, hai hàng 500 đổi sang `server_error` retryable. `connection_check_contract.rs`: 402/429/5xx/418, ca HTTP 402 che khoá, 401 mang lời, id `models/`.
- Vitest: `aiProviderMessageLine.test.ts` mới; `aiConfigState.test.ts`, `settingsOverlayAiConfigTierRender.test.ts` sang `gemini-3.8-flash`; `aiTranslate.test.ts` fixture 418 + câu chung.
- `deferred-work.md`: dòng 🔵 vào mục preset (Chủ: Epic 12) + mục mới Chủ: Epic 4.
## Đối chứng đỏ
- Gỡ `redact_api_key(...)` trong `read_provider_message` (giữ chữ ký): `ai_translate_wire` đỏ 2 ca (402 lộ khoá nguyên văn; mảnh khoá không bị bỏ). Đã hoàn lại, xanh.
