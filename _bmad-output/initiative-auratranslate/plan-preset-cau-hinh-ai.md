---
title: 'Preset nhà cung cấp AI: một lần bấm ghi nhà cung cấp, endpoint và mô hình'
type: 'feature'
ticket: ''
created: '2026-10-25'
status: 'built'
baseline_revision: '29c5a09967adf59d03aee5f477ef84dbd10d8298'
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

**Problem:** Bấm một nút nhà cung cấp ở Cài đặt › AI chỉ ghi `provider`. Endpoint chỉ hiện làm placeholder và mô hình không có gợi ý nào, nên muốn dùng Claude hay Gemini người dùng vẫn phải tự gõ URL và mã mô hình. Ngoài ra danh sách chỉ có Anthropic, OpenAI và hai runtime chạy máy.

**Approach:** Mỗi nút nhà cung cấp trở thành một preset. Bấm nút ghi `provider` cùng `endpoint` và `model` của preset ở tầng đang xem. Nếu endpoint hoặc mô hình đang lưu khác giá trị preset, app hỏi xác nhận ngay trong trang trước khi ghi đè. Danh sách gồm chín nút: Anthropic · OpenAI · Gemini · OpenRouter · Mistral · xAI · Ollama · LM Studio · Endpoint khác.

## Boundaries & Constraints

**Always:**
- Quyết định của Ice (2026-10-25): chọn preset nghĩa là đồng ý đổi cả ba cột. Quyết định này thay dòng 29 của `tickets/epic-hoan-thien-giao-dien-theo-mockup/tickets/story-cai-at-bo-cuc-phim-tat-thu-ket-noi-khung-cap-nhat-plan.md`.
- Preset chỉ chạm `provider`, `endpoint`, `model`; không bao giờ chạm `temperature`, `max_tokens` hay khoá API.
- Một trường để trống trong preset nghĩa là không ghi trường đó (Ollama và LM Studio không có mô hình mặc định). "Endpoint khác" chỉ ghi `provider`, giữ nguyên như hiện nay.
- Ghi ở `effectiveTier()`, đúng như mọi trường khác (AD-18).
- Quyết định của Ice (2026-10-25, OQ1 → b): ba trường đi qua MỘT lệnh Rust mới `ai_config_save_fields`. Lệnh kiểm mọi giá trị trước (một giá trị hỏng ⇒ 0 lượt ghi) rồi ghi trong một transaction: được hoặc mất cả ba, không có cấu hình nửa mới nửa cũ.
- Hộp xác nhận nằm trong trang, không dùng `window.confirm`. Chuỗi tiếng Việt đặt trong `vi.json` (NFR16).

**Never:** không thêm client theo giao thức riêng của từng nhà cung cấp, vì `core/ai/client.rs` vẫn chỉ nói chat-completions. Không đổi `provider_needs_key`. Không ẩn hay tự sửa nhiệt độ theo nhà cung cấp.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Bắt đầu trống | endpoint và model trống, bấm Gemini | ghi provider `gemini`, endpoint Gemini, model `gemini-flash-latest`; không hỏi | — |
| Ghi đè | endpoint đang là URL OpenAI, bấm Anthropic | hiện hộp xác nhận nêu giá trị cũ và giá trị mới; bấm Áp dụng thì ghi cả ba, bấm Huỷ thì không ghi gì | — |
| Đã khớp | endpoint và model đã bằng giá trị preset | ghi provider, không hỏi | — |
| Preset không có mô hình | model `qwen`, bấm Ollama, endpoint trống | ghi provider và endpoint, giữ `qwen`, không hỏi | — |
| Tầng Tác phẩm | đang xem tầng Tác phẩm, bấm Mistral | ghi ba trường ghi đè ở tầng Tác phẩm | — |
| Lượt ghi trượt | `ai_config_save_fields` trả lỗi (kho, tầng Tác phẩm đã đóng) | không trường nào đổi; lỗi hiện dưới hàng nút nhà cung cấp | không thử lại tự động |
| Một giá trị hỏng | danh sách có một giá trị không hợp lệ | `ai_config.invalid_value`, 0 lượt ghi | — |

</frozen-after-approval>

## Code Map

- `src/aiConfigState.ts:27-37` -- `AI_PROVIDERS` và `PROVIDER_ENDPOINT_PLACEHOLDER`. Thay bằng `AI_PROVIDER_PRESETS: Record<AiProvider, { endpoint: string; model: string }>`; placeholder của ô endpoint đọc từ bảng này.
- `src/aiConfigState.ts:440-444` -- `selectAiConfigProvider`, hiện chỉ ghi provider.
- `src/aiConfigState.ts:283-300` -- `saveAiConfigField` gọi `loadAiConfigSection()` sau mỗi lượt ghi, và lượt tải đó đặt lại drafts. ⚠️ Gọi nó ba lần liền nhau thì draft của trường sau bị mất. Phải gọi `aiConfigSaveField` (`src/config/aiconfig.ts`) trực tiếp cho từng trường rồi tải lại một lần. `trackSave` và cờ `saving` dùng lại như cũ.
- `aiConfigValueAtViewTier(field)` -- giá trị so với preset để quyết định có hỏi xác nhận hay không.
- `src/SettingsOverlay.vue:222-249, 395-438` -- hàng nút (mockup `settings.html` dòng 166-173). Hộp xác nhận đặt ngay dưới `.ai-pills`.
- `src/i18n/vi.json:2055-2059` -- nhãn nút; thêm bốn nhãn mới và chuỗi của hộp xác nhận.
- `src-tauri/src/commands/aiconfig.rs:303-309` -- `provider_needs_key`. Không đổi: mọi preset mới đều cần khoá.
- `src-tauri/src/core/aiconfig/store.rs:98` -- `write_field` chạy một `store.write` (mỗi job là một transaction, `core/store/mod.rs:776`). Thêm `write_fields(store, &[(AiConfigField, String)])` dùng cùng câu UPSERT trong MỘT job.
- `src-tauri/src/commands/aiconfig.rs:250-261` -- hàm thuần `ai_config_save_field` (`validate_field` → `store_for_tier` → `write_field`); vỏ `wire::ai_config_save_field` ở 343-360. Lệnh mới theo đúng hai lớp này. Chuỗi trong tệp `.rs` viết KHÔNG DẤU (`check:i18n` Kiểm A).
- `src-tauri/src/lib.rs:1355` -- `generate_handler!`; thêm `crate::commands::aiconfig::wire::ai_config_save_fields`.
- `src-tauri/tests/ipc_contract.rs:2230` -- `the_aiconfig_key_wires_are_registered`; thêm wire mới vào danh sách. `src-tauri/tests/aiconfig_contract.rs` gọi thẳng hàm thuần, theo khuôn dòng 292-370.
- `src/config/aiconfig.ts` -- thêm adapter `aiConfigSaveFields(tier, fields)` theo khuôn `aiConfigSaveField`.
- Tests đang khoá quyết định cũ, phải đổi: `tests/frontend/aiConfigState.test.ts:786` và `tests/frontend/settingsOverlayAiConfigTierRender.test.ts:156`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/aiconfig/store.rs` + `src-tauri/src/commands/aiconfig.rs` + `src-tauri/src/lib.rs` -- `write_fields` và lệnh `ai_config_save_fields(tier, fields)`: danh sách rỗng hoặc trùng trường ⇒ `invalid_value`; kiểm toàn bộ trước khi ghi -- OQ1 → b.
- [x] `src-tauri/tests/aiconfig_contract.rs` + `src-tauri/tests/ipc_contract.rs` -- ghi cả ba ở hai tầng; một giá trị hỏng thì không hàng nào đổi; wire đã đăng ký.
- [x] `src/config/aiconfig.ts` -- adapter `aiConfigSaveFields`.
- [x] `src/aiConfigState.ts` -- bảng preset, `selectAiConfigProvider` mới ghi ba trường qua MỘT lời gọi `aiConfigSaveFields` rồi tải lại một lần, trạng thái hộp xác nhận (đang chờ / áp dụng / huỷ) -- giữ toàn bộ logic trong state, không đặt trong `.vue`.
- [x] `src/SettingsOverlay.vue` + `src/i18n/vi.json` -- chín nút, hộp xác nhận nằm trong trang (một `role="alertdialog"` gồm hai nút) -- theo khung mockup.
- [x] `tests/frontend/aiConfigState.test.ts` + `tests/frontend/settingsOverlayAiConfigTierRender.test.ts` -- phủ từng hàng của Matrix và AC; thay hai test của quyết định cũ.
- [x] `tickets/epic-hoan-thien-giao-dien-theo-mockup/tickets/story-cai-at-bo-cuc-phim-tat-thu-ket-noi-khung-cap-nhat-plan.md` -- thêm một dòng 🔵 2026-10-25 ở dòng 29 và dòng 118, trỏ sang plan này.
- [x] `deferred-work.md` -- thêm mục `Chủ: Epic 12`: Ice bấm Thử kết nối và dịch một câu trên từng preset có mạng.

**Acceptance Criteria:**
- Given hộp xác nhận đang mở, when bấm Huỷ, then không có lượt IPC ghi nào và ba trường giữ nguyên giá trị đã lưu.
- Given đã áp dụng một preset, when mở lại Cài đặt, then nút của preset đó đang bật và endpoint cùng mô hình hiện đúng giá trị đã lưu.
- Given `temperature` là `0.3`, when áp dụng bất kỳ preset nào, then `temperature` vẫn là `0.3`.

## Design Notes

Bảng preset (đã đối chiếu tài liệu chính thức ngày 2026-10-25; mục chưa kiểm được ghi "UNVERIFIED" và nằm trong mục nợ của Ice):

| provider | endpoint | model |
|---|---|---|
| anthropic | `https://api.anthropic.com/v1` | `claude-sonnet-5-5` |
| openai | `https://api.openai.com/v1` | `gpt-6.1-sol` |
| gemini | `https://generativelanguage.googleapis.com/v1beta/openai` | `gemini-flash-latest` |
| openrouter | `https://openrouter.ai/api/v1` | để trống: slug UNVERIFIED |
| mistral | `https://api.mistral.ai/v1` | `mistral-large-latest` (UNVERIFIED) |
| xai | `https://api.x.ai/v1` | `grok-4.7` |
| ollama / lmstudio | như hiện nay | để trống |

Các rủi ro đã biết nhưng không sửa trong plan này, đều thuộc mục nợ của Ice:
- Claude 5 trả 400 khi nhiệt độ dưới 1.
- Model suy luận của OpenAI từ chối `max_tokens`.
- Chưa rõ Mistral, xAI và OpenRouter có nhận `stream_options` hay không.
- `/v1/models` của Anthropic dùng Bearer có cần thêm `anthropic-version` hay không.

Hai giá trị cuối là mặc định của client, không phải lựa chọn của preset.

## Verification

**Commands:**
- `npm run build && cd src-tauri && cargo test --test aiconfig_contract --test ipc_contract` -- xanh
- Đụng `lib.rs` ⇒ chạy toàn bộ bộ test một lần bằng tay trước khi báo xong (AGENTS.md §Tests)
- `npx vitest run tests/frontend/aiConfigState.test.ts tests/frontend/settingsOverlayAiConfigTierRender.test.ts` -- xanh
- `npm run check:lint && npx vue-tsc --noEmit -p tsconfig.json && npm run check:i18n && npm run check:tokens` -- xanh

## Implementation Notes

- Ba pha Rust · Webview · Tests, mỗi pha một agent mới, bàn giao qua `handoff-preset-cau-hinh-ai-phases.md`.
- `config_invariants.rs`: bảng đếm lệnh tăng 106 → 107 plain vì có thêm `ai_config_save_fields`. Không có trong plan, nhưng là hệ quả bắt buộc.
- Bấm một nút làm ô đang gõ mất tiêu điểm, và lượt lưu khi rời ô còn đang chạy. Vì vậy `selectAiConfigProvider` chờ `inFlightSave` xong rồi mới so với giá trị đã lưu. Không chờ thì endpoint vừa gõ bị ghi đè mà không hỏi. Phép đối chứng: gỡ dòng chờ thì test đỏ (gọi `aiConfigSaveFields` một lần), trả dòng lại thì xanh.
- `check:commands` (AD-34 §1) đỏ vì hai nút của hộp xác nhận dùng `@click` gọi thẳng hàm. Đã đăng ký hai command `settings.ai_config.preset_apply` và `settings.ai_config.preset_cancel`, đều không có hợp âm mặc định, theo khuôn `history.confirm_restore`, và nối port ở `main.ts`. Test đọc mã nguồn `main.ts` để canh dây này; gỡ dòng nối port thì test đỏ.
- Toàn bộ bộ test sau khi sửa: vitest 3197/3197, `cargo test` 2693 đạt / 0 trượt (101 binary), build OK, 13/14 cổng xanh. `check:gates` đỏ sẵn từ trước: `RULE_LIGHTNESS_FLOOR` trong `scripts/check-commands.mjs` không đi qua `judgeFloor`. Lỗi này do `41e956f` sinh ra, và story này không chạm `scripts/`.
- Mã mô hình gắn cứng và còn UNVERIFIED (Mistral, chưa thử OpenAI và xAI): mục nợ `Chủ: Epic 12` trong `deferred-work.md`.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 6 · false 3 · maybe-false 0.
- medium · patch · bấm nút ngay sau khi gõ endpoint thì bỏ qua bước hỏi: `blur` → `commitAiConfigField` chạy trước `click`, `resolvedFields` còn giá trị cũ; sửa bằng cách chờ `inFlightSave`, có test và phép đối chứng.
- low · reject · hộp xác nhận giữ ảnh chụp cũ khi người dùng sửa ô trong lúc hộp đang mở: hiếm gặp, sửa phải thêm một chốt chặn.
- false · nút thứ hai bấm trong lúc đang ghi: `applyAiConfigPreset` đặt `saving.provider`, và nút đã có `:disabled="aiConfigIsSaving('provider')"`.
- low · patch · doc-comment `wire` vẫn ghi "Năm vỏ" → "Sáu vỏ".
- low · reject · kiểm trùng trường chạy trước kiểm giá trị nên `params.field` có thể chỉ trường trùng: webview không bao giờ gửi trùng, không có tác hại nêu được.
- false · dấu phẩy cuối trong `ipc_contract.rs` là cố ý để không khớp tiền tố `ai_config_save_field`; phép `contains` lỏng có từ trước story.
- false · plan chưa chốt sổ: bước 5 làm việc này; toàn bộ bộ test chạy sau review.
- low · patch · doc-comment mới viết tiếng Việt và dài 3-4 dòng, trái AGENTS.md §Code comments: viết lại bằng tiếng Anh, 1-2 dòng.
- low · patch · bộ chọn `.ai-field-error` không tồn tại trong test lỗi ghi: thu lại thành `[data-ai-field="provider"] [role="alert"]`.
- low · reject · AC "mở lại Cài đặt" chỉ phủ bằng lượt `ai_config_get` giả: vitest không mở lại app được; lượt dùng thật nằm trong mục nợ Epic 12.
