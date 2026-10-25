# Handoff: chip nhiệt độ và token (plan-chip-nhiet-do-va-token.md)

## Phase 1 (Rust) — xong, chưa commit
- `src-tauri/src/core/aiconfig/mod.rs`: thêm `pub const PROVIDER_DEFAULT = "default"`; `validate_temperature` / `validate_max_tokens` nhận đúng `default` (sau trim, phân biệt hoa thường: `Default` bị từ chối), trả `"default"`. Test đơn vị mới ở cuối tệp.
- `src-tauri/src/commands/aitranslate.rs` `resolve_call_config`: nhánh tường minh `default => None` cho cả hai trường.
- `src-tauri/tests/ai_translate_contract.rs`: `an_explicit_default_at_either_tier_leaves_temperature_and_max_tokens_unsent` (đối chứng số, default tầng Work đè số Toàn cục, số tầng Work đè default Toàn cục).
- Đã chạy xanh: `cargo test --lib aiconfig`, `--test ai_translate_contract --test aiconfig_contract`, `cargo clippy --all-targets`.

## Cho phase sau
- Test Rust `every_row_of_the_shared_validity_table...` đọc `tests/frontend/support/ai-config-validity.json` (yêu cầu >= 30 hàng). Thêm hàng `temperature`/`max_tokens` với input `default` (valid true), `Default` (valid false) ở phase Tests, rồi chạy lại `cargo test --test aiconfig_contract` để hai phía cùng đồng ý. Phía TS `isAiConfigValueValid` phải nhận `default` đúng luật này (trim, chữ thường chính xác).
- Không có đối chứng đỏ cho nhánh `default => None` ở `resolve_call_config`: trước đây `"default".parse()` thất bại cũng ra `None`, nên hành vi quan sát được giống nhau; nhánh chỉ làm ý định tường minh (theo plan). Test khoá hành vi, không khoá nhánh.
- Chưa làm: mọi tệp `src/`, `scripts/check-commands.mjs`, `tests/frontend/`, 🔵 trong `plan-preset-cau-hinh-ai.md`, ghi nợ `deferred-work.md`; `context:` của plan rỗng.

## Phase 2 (Webview) — xong, chưa commit
- Đã xanh: `vue-tsc`, `check:commands`, `check:lint`, `check:i18n`, `check:tokens`. Chưa sửa test nào.
- `src/aiConfigState.ts`: `AI_PROVIDER_PRESETS[p]` nay có `{endpoint, model, temperature, maxTokens, keyPlaceholder}` (giá trị `'default'` | số dạng chuỗi | `''` = không ghi; custom đều `''`). Xuất mới: `AI_CHIP_FIELDS`, `AI_CHIP_VALUES` (`temperature: 0.2/0.5/1`, `max_tokens: 4096/8192/16384`), kiểu `AiChipField`/`AiChipChoice`, `aiConfigChipIsOn(field, chip)` (chip = `'default'` | giá trị số | `'custom'`), `aiConfigChipInputIsOpen`, `aiConfigChipInputValue` (lưu `default` hiện ô trống), `selectAiConfigChip(field, chip)` (custom chỉ mở ô; chip khác ghi qua `saveAiConfigField`; đã bật thì không ghi), `aiConfigKeyPlaceholder()`.
- `isAiConfigValueValid`: `temperature`/`max_tokens` nhận đúng `default` sau trim (`Default` bị từ chối).
- `AiConfigPresetChange.field` mở rộng thêm `temperature`|`max_tokens`. `selectAiConfigProvider` so cả 4 trường; so số theo giá trị (`0.20` = `0.2`); lưu rỗng = mặc định = không hỏi; lưu `default` mà preset là số thì VẪN liệt kê (explicit). `applyAiConfigPreset` ghi thêm hai trường khi preset không rỗng.
- Hook DOM: hàng `[data-ai-field="temperature|max_tokens"]`; nút chip `[data-ai-chip="default|0.2|0.5|1|4096|8192|16384|custom"]` (`aria-pressed`, trong `form.ai-pill-form`); ô Tự nhập `[data-ai-chip-input]`; dòng xác nhận `[data-ai-preset-change="temperature|max_tokens"]` (`default` hiện "Mặc định nhà cung cấp"). Ô khoá có `placeholder` (rỗng cho mistral/ollama/lmstudio/custom).
- i18n mới: `chip_default`, `chip_custom`, `chip_temperature_{0_2,0_5,1}`, `chip_max_tokens_{4096,8192,16384}`, `preset_confirm_{temperature,max_tokens}`; sửa `preset_confirm_body` bỏ vế "nhiệt độ, số token không đổi". `check-commands.mjs`: thêm hàng `onSelectAiChip`.
- Hai tệp vitest hiện có đỏ (15 ca, kỳ vọng cũ): các ca `applyPreset` mong `saveFields` chỉ 3 trường, các ca vòng `AI_TEXT_FIELDS`/ô nhập `temperature`/`max_tokens` (nay là chip), danh sách `changes` hộp xác nhận. Phase 3 cập nhật.
- Ô Tự nhập tự đóng sau mỗi lượt tải lại (`loadAiConfigSection`); ô mở được suy ra từ giá trị lưu khi không khớp chip nào.

## Phase 3 (Tests) — xong, chưa commit
- `tests/frontend/aiConfigState.test.ts` (88 ca xanh) và `settingsOverlayAiConfigTierRender.test.ts` (24 ca xanh): sửa 15 ca cũ theo năm trường, thêm ca bảng preset, hộp xác nhận, placeholder khoá, và hai khối chip (state và DOM) phủ từng hàng Matrix + 3 AC.
- `tests/frontend/support/ai-config-validity.json`: thêm 6 hàng (`default`, ` default `, `Default` cho `temperature` và `max_tokens`); `cargo test --test aiconfig_contract` 29 xanh.
- Đã ghi 🔵 vào `plan-preset-cau-hinh-ai.md` (dòng Boundaries) và dòng `→ 🔵` cuối mục nợ preset trong `deferred-work.md` (`Chủ: Epic 12`).
- Chưa chạy: toàn bộ vitest, `cargo test` đầy đủ, e2e, `check:commands` (xanh ở phase 2).
