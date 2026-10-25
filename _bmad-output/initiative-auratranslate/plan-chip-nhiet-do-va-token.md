---
title: 'Nhiệt độ và số token tối đa: chip, kèm mặc định theo từng preset'
type: 'feature'
ticket: ''
created: '2026-10-25'
status: 'built'
baseline_revision: '376f1cfe86fb556412d9b292a463383d225e4cf1'
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

**Problem:** Nhiệt độ và số token tối đa hiện là ô nhập trống, không có gợi ý nào. Preset không đặt hai trường này, nên một giá trị cũ có thể đi theo sang nhà cung cấp mới và làm hỏng lượt dịch (Claude 5 trả 400 khi nhiệt độ dưới 1). Ở tầng Toàn cục cũng không có cách nào đưa hai trường về mặc định của nhà cung cấp.

**Approach:**
- Hai trường đổi thành hàng chip:
  - Nhiệt độ: `Mặc định nhà cung cấp · 0.2 Sát nghĩa · 0.5 Cân bằng · 1 Tự nhiên · Tự nhập`.
  - Số token tối đa: `Mặc định nhà cung cấp · 4096 · 8192 · 16384 · Tự nhập`.
- Mỗi preset mang thêm nhiệt độ và số token của riêng nó, và áp preset ghi cả hai.
- "Mặc định nhà cung cấp" được lưu thành giá trị tường minh `default`, có nghĩa ở cả hai tầng. Lượt gọi gặp `default` thì không gửi trường đó.
- Ô khoá API có placeholder theo nhà cung cấp.

## Boundaries & Constraints

**Always:**
- Quyết định của Ice (2026-10-25): phương án C, mặc định theo từng preset. Quyết định này thay dòng "Preset chỉ chạm `provider`, `endpoint`, `model`" trong `plan-preset-cau-hinh-ai.md` (commit f88530b).
- Áp preset ghi cả năm trường. Hộp xác nhận ghi đè liệt kê mọi trường mà giá trị đang lưu không rỗng và khác preset.
- Giá trị đang lưu là `default` hoặc không có hàng thì coi như "Mặc định nhà cung cấp": chip đó bật, và không hỏi xác nhận khi preset cũng là `default`.
- Một giá trị lưu khớp đúng một chip số thì chip đó bật. Mọi giá trị hợp lệ khác bật chip "Tự nhập", kèm ô nhập hiện giá trị đó. Ô nhập lưu khi rời ô, như các trường khác.
- `default` đi qua đúng luật kiểm ở Rust và webview cho hai trường này. Rust nhận diện `default` một cách tường minh, không dựa vào việc `parse()` thất bại.
- "Endpoint khác" vẫn chỉ ghi `provider`.

**Never:** không xoá hàng để biểu diễn "Mặc định" (ở tầng Tác phẩm, xoá hàng nghĩa là kế thừa Toàn cục). Không đổi khoảng hợp lệ `[0, 2]` và `> 0`. Không đặt giá trị mặc định cho khoá API.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Chọn chip số | bấm `0.5` | lưu `temperature = 0.5` ở tầng đang xem; chip 0.5 bật | — |
| Mặc định ở tầng Tác phẩm | Toàn cục `0.3`, tầng Tác phẩm bấm Mặc định | hàng của Tác phẩm là `default`; lượt dịch không gửi `temperature` | — |
| Tự nhập | bấm Tự nhập, gõ `0.35`, rời ô | lưu `0.35`; chip Tự nhập bật, ô nhập hiện `0.35` | `3` hiện câu từ chối hiện có, 0 lượt ghi |
| Áp preset đè nhiệt độ | `temperature = 0.3`, bấm Anthropic | hộp xác nhận có dòng "Nhiệt độ: 0.3 → Mặc định nhà cung cấp" | Huỷ thì không ghi gì |
| Áp preset từ trống | chưa có gì, bấm Mistral | ghi `0.2` và `default` cùng ba trường kia; không hỏi | — |
| Lượt gọi | `max_tokens = default` | thân request không có `max_tokens` | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/aiconfig/mod.rs:150-167` -- `validate_temperature` / `validate_max_tokens`: nhận thêm `default`. Có test đơn vị ở cuối tệp.
- `src-tauri/src/commands/aitranslate.rs:271-281` -- `resolve_call_config`: thêm nhánh tường minh `default ⇒ None` cho cả hai trường. Test hợp đồng nằm trong `tests/ai_translate_contract.rs` (dùng `build_request_body` để kiểm trường bị bỏ khỏi thân request).
- `src/aiConfigState.ts` -- `isAiConfigValueValid` (nhánh `temperature` / `max_tokens`) nhận `default`. `AI_PROVIDER_PRESETS` thêm `temperature`, `maxTokens`, `keyPlaceholder`. `selectAiConfigProvider` so cả năm trường. `applyAiConfigPreset` đã ghi qua `aiConfigSaveFields` (một transaction), nên chỉ cần thêm hai trường vào danh sách. Hàm chọn chip mới ghi qua `saveAiConfigField`.
- `src/SettingsOverlay.vue:479-527` -- vòng `AI_TEXT_FIELDS` vẽ bốn ô nhập. Tách `temperature` / `max_tokens` thành hàng chip theo khuôn `.ai-pills` của provider (dòng 401-416). Ô khoá API ở khoảng dòng 530 nhận `:placeholder`.
- `scripts/check-commands.mjs:2509-2511` -- bảng Kiểm K (`@submit` đông cứng hai chiều). Mỗi handler `@submit` mới phải có một hàng `nonCommand: R_SUBMIT_DIRECT` như `onSelectAiProvider`.
- `src/i18n/vi.json` -- nhãn chip và dòng "Mặc định nhà cung cấp" của hộp xác nhận (`preset_confirm_temperature` / `preset_confirm_max_tokens`).
- Tests: `tests/frontend/aiConfigState.test.ts`, `tests/frontend/settingsOverlayAiConfigTierRender.test.ts`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/aiconfig/mod.rs` + `src-tauri/src/commands/aitranslate.rs` -- nhận `default`, rồi ánh xạ tường minh `default ⇒ None` -- để biểu diễn mặc định ở cả hai tầng.
- [x] `src-tauri/tests/ai_translate_contract.rs` (+ test đơn vị validator) -- `default` cho ra thân request không có hai trường; `default` lưu được ở cả hai tầng.
- [x] `src/aiConfigState.ts` -- bảng preset theo Design Notes, so năm trường, trạng thái chip, placeholder khoá -- giữ toàn bộ logic trong state.
- [x] `src/SettingsOverlay.vue` + `src/i18n/vi.json` + `scripts/check-commands.mjs` -- hai hàng chip, ô Tự nhập, placeholder khoá, các hàng mới ở bảng Kiểm K.
- [x] `tests/frontend/aiConfigState.test.ts` + `tests/frontend/settingsOverlayAiConfigTierRender.test.ts` -- phủ từng hàng Matrix và AC.
- [x] `plan-preset-cau-hinh-ai.md` -- thêm 🔵 2026-10-25 vào dòng Boundaries bị thay. `deferred-work.md` -- nối thêm vào mục nợ preset (`Chủ: Epic 12`): kiểm nhiệt độ và số token từng preset trên nhà cung cấp thật.

**Acceptance Criteria:**
- Given giá trị lưu là `default` hoặc không có hàng, when mở Cài đặt, then chip "Mặc định nhà cung cấp" bật.
- Given đang xem tầng Tác phẩm, when bấm một chip, then chỉ hàng của tầng Tác phẩm đổi và nút "Trả về kế thừa" hiện như với các trường khác.
- Given preset là OpenAI, when đọc ô khoá API trống, then placeholder là `sk-…`.

## Design Notes

Mặc định theo từng preset. Nguồn: tài liệu chính thức hoặc nghiên cứu ngày 2026-10-25; số nào là lựa chọn của mình thì ghi rõ.

| provider | temperature | max_tokens | key placeholder | lý do |
|---|---|---|---|---|
| anthropic | default | 8192 | `sk-ant-…` | Claude 5 trả 400 khi nhiệt độ < 1. API gốc bắt buộc `max_tokens`, còn lớp tương thích khi thiếu trường này: UNVERIFIED |
| openai | default | default | `sk-…` | model suy luận từ chối cả hai (nguồn thứ cấp) |
| gemini | default | default | `AIza…` | Google: "strongly recommend keeping the temperature at 1.0" (ai.google.dev/gemini-api/docs/gemini-3) |
| openrouter | default | default | `sk-or-…` | chưa có model; token suy luận tính vào `max_tokens` |
| mistral | 0.2 | default | (trống) | Mistral khuyên 0.0–0.7; chọn 0.2 cho dịch sát nghĩa (lựa chọn của mình) |
| xai | default | default | `xai-…` | lấy mẫu tối đa ở 1 |
| ollama / lmstudio | 0.2 | default | (trống) | model chạy máy mặc định khoảng 0.7–0.8, dịch cần ổn định (lựa chọn của mình) |
| custom | không ghi | không ghi | (trống) | — |

Chọn `default` thay vì xoá hàng: Toàn cục là `0.2` (Mistral), áp Anthropic ở tầng Tác phẩm. Nếu xoá hàng thì tầng Tác phẩm kế thừa `0.2` và Claude 5 trả 400. Lưu `default` thì lượt gọi không gửi nhiệt độ. Cách này cũng không cần lệnh Rust xoá hàng ở tầng Toàn cục.

## Verification

**Commands:**
- `npx vitest run tests/frontend/aiConfigState.test.ts tests/frontend/settingsOverlayAiConfigTierRender.test.ts` -- xanh
- `npm run build && cd src-tauri && cargo test --test ai_translate_contract --test aiconfig_contract && cargo test --lib aiconfig` -- xanh
- `npm run check:lint && npm run check:commands && npm run check:i18n && npm run check:tokens && npx vue-tsc --noEmit -p tsconfig.json` -- xanh

## Implementation Notes

- Ba pha Rust · Webview · Tests, mỗi pha một agent mới, bàn giao qua `handoff-chip-nhiet-do-va-token-phases.md`.
- So chip theo giá trị số (`0.20` = `0.2`, `+4096` = `4096`). Giá trị lưu `default` mà preset là một số thì hộp xác nhận vẫn liệt kê, vì `default` là lựa chọn tường minh của người dùng.
- Bảng hợp lệ dùng chung `tests/frontend/support/ai-config-validity.json` (Rust và TS cùng đọc) thêm `default` hợp lệ, `Default` không hợp lệ cho hai trường.
- Nhánh `default ⇒ None` ở `resolve_call_config` không đối chứng đỏ được: trước khi sửa, `"default".parse()` cũng ra `None`. Test hợp đồng canh hành vi chứ không canh riêng nhánh.
- Chip không bị khoá theo `saving[field]`: cú bấm chip làm ô Tự nhập mất tiêu điểm, lượt lưu khi rời ô bật `saving`, và nút bị khoá trước `mouseup` làm mất cú bấm. `selectAiConfigChip` đã chờ `inFlightSave`. Có test, và đối chứng đỏ khi trả `:disabled` lại.

- Toàn bộ bộ test: vitest 3231/3231, `cargo test` 2695 đạt / 0 trượt (101 binary), build OK. `check:fmt` đỏ ở một test mới, đã chạy `cargo fmt` cho riêng tệp đó. `check:gates` vẫn đỏ sẵn từ trước (`RULE_LIGHTNESS_FLOOR`, commit `41e956f`).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 5 · false 3 · maybe-false 0.
- medium · patch · chip có `:disabled="aiConfigIsSaving(field)"` nuốt cú bấm khi ô Tự nhập lưu lúc rời ô: bỏ `:disabled` trên chip; test và đối chứng đỏ.
- low · patch · đóng ô Tự nhập bằng chip đang bật thì draft gõ dở còn lại và câu từ chối treo: đặt lại draft về giá trị đã lưu; test và đối chứng đỏ.
- low · patch · `aiPresetChangeValue` đổi `default` thành nhãn cho cả endpoint và model: chỉ áp cho hai trường chip.
- low · reject · nhánh `default ⇒ None` không có test bắt được khi gỡ: hành vi giống hệt đường `parse()` thất bại, muốn canh phải dựng một seam giả.
- low · patch · doc-comment nhắc lại tên hàm (`aiConfigChipIsOn`, `aiConfigChipInputIsOpen`, `AI_CHIP_VALUES`) và doc tiếng Việt trên test Rust: đã xoá.
- low · patch · `AiChipChoice` có `(string & {})` nên thực chất là `string`: `AI_CHIP_VALUES` đổi thành `as const` và thêm `aiConfigChipChoices`.
- false · plan chưa chốt sổ: bước 5 làm việc này.
- false · giá trị lưu không hợp lệ thì không chip nào bật: Rust kiểm mọi lượt ghi (`validate_field`), nên app không tạo được trạng thái này.
- false · thiếu test cho rời ô rồi bấm chip: trùng với phát hiện đầu, nay đã có test.
