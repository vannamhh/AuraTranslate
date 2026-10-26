---
title: 'Panel Đề xuất AI nêu đúng lý do chưa sẵn sàng và ghi đè ở tầng Tác phẩm'
type: 'feature'
ticket: ''
created: '2026-10-26'
status: 'built'
baseline_revision: 'f75f7be215c51ab3995d5b7d525ca4bbfdbe4e42'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Khi lượt dịch chưa sẵn sàng, panel Đề xuất AI luôn hiện "Chưa cấu hình AI" kèm lời khuyên thêm nhà cung cấp, endpoint và mô hình, kể cả khi ba trường đó đã có. Ca thật 2026-10-26: tầng Toàn cục dùng Ollama đầy đủ, nhưng Tác phẩm ghi đè `provider=anthropic`. Cấu hình hiệu lực vì thế cần khoá API, keychain không có khoá, và không gì trên màn hình chỉ ra điều đó.

**Approach:**
- Rust trả danh sách điều kiện chưa đạt (thiếu endpoint, thiếu mô hình, thiếu khoá), tính từ cùng vị từ với phán quyết `ready`.
- Panel nêu từng điều kiện bằng một câu cụ thể. Nếu Tác phẩm đang ghi đè trường nào thì panel nói tên các trường đó và cho một nút mở thẳng Cài đặt › AI ở phạm vi Tác phẩm.

## Boundaries & Constraints

**Always:**
- Quyết định của Ice (2026-10-26), phương án b: lý do cụ thể, ví dụ "Nhà cung cấp Anthropic cần khoá API". Khi có ghi đè, panel nói rõ trường nào đang ghi đè ở Tác phẩm này và có hành động mở Cài đặt ở phạm vi Tác phẩm.
- Lý do và `ready` đi ra từ MỘT hàm Rust, nên không bao giờ có cảnh `ready = false` mà danh sách lý do rỗng, hoặc ngược lại.
- Keychain không trả lời (`key_configured = None`) vẫn là trạng thái riêng `key_unchecked`, không được ghi thành "thiếu khoá".
- Nút mở Cài đặt là command đăng ký (AD-34 §1). Không chuỗi tiếng Việt nào nằm trong `.vue` (NFR16).

**Never:** không đổi luật sẵn sàng (`endpoint_and_model_set`, `provider_needs_key`). Không tự sửa cấu hình thay người dùng. Không đổi luật một trường ghi đè không che các trường kia (Ice ký 2026-08-04).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Ghi đè cần khoá | Toàn cục ollama + endpoint + model; Tác phẩm `provider=anthropic`; không có khoá | câu "Nhà cung cấp Anthropic cần khoá API"; dòng "Tác phẩm này đang ghi đè: Nhà cung cấp"; nút "Mở Cài đặt ở phạm vi Tác phẩm này" mở mục AI với "Tác phẩm này" đang chọn | — |
| Thiếu endpoint và mô hình | chưa có gì | hai câu "Chưa có endpoint" và "Chưa chọn mô hình"; nút "Mở Cài đặt" mở mục AI ở tầng Toàn cục | — |
| Chỉ thiếu mô hình | provider ollama, endpoint có | chỉ câu "Chưa chọn mô hình", không có câu về khoá | — |
| Nhà cung cấp trống hoặc lạ | `provider` trống hoặc `groq`, endpoint + model có, không khoá | "Chưa chọn nhà cung cấp — cần khoá API" / "Nhà cung cấp groq cần khoá API" | — |
| Keychain không trả lời | endpoint + model có, `key_configured = None` | giữ gợi ý `key_unchecked` hiện có, không có câu "cần khoá" | — |
| Sẵn sàng | đủ điều kiện | không hiện khối chưa sẵn sàng | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/aiconfig.rs:281-309` -- `endpoint_and_model_set`, `call_ready`, `provider_needs_key`. Thêm một hàm trả `Vec<AiNotReadyReason>` (`endpoint_missing` · `model_missing` · `key_missing`, theo thứ tự đó) cùng phán quyết, rồi viết `call_ready` lại thành dẫn xuất của hàm này. `call_ready` chỉ có một chỗ gọi (`ai_config_get`, dòng 194).
- `src-tauri/src/commands/aiconfig.rs:120-150` -- `AiConfigGetWire`: thêm `not_ready: Vec<AiNotReadyReason>` (serde `snake_case`). Rỗng khi `ready` là `Some(true)` hoặc `None`.
- `src-tauri/tests/aiconfig_contract.rs` -- test hàm thuần `ai_config_get` theo khuôn hiện có (`Some(&global)`, `OpenWork` giả). Dùng khoá giả hoặc ca `key_configured` thì xem cách các test hiện có né keychain thật.
- `src/config/aiconfig.ts` -- `AiConfigGetWire` và kiểu trả về của `aiConfigGet` thêm `notReady`.
- `src/aiConfigState.ts:243-260, 355-370` -- `applyReadiness` / `refreshAiConfigReadiness` giữ thêm `notReady` và danh sách trường có `tier === 'work'`. Xuất hai readonly mới cho panel.
- `src/panels/AiTranslationPanel.vue:423-435` -- khối `data-ai-unconfigured`. Thay câu `unconfigured_local` bằng các câu lý do cùng dòng ghi đè. Nút hiện gọi `dispatch('settings.open')`, đổi thành `settings.ai_config.open`, và thêm nút cho phạm vi Tác phẩm.
- `src/settingsState.ts:251-300` + `src/aiConfigState.ts:202-205` -- `settings.ai_config.open` gọi `resetAiConfigViewTier()`, đưa `tierChoice` về `null`. Khi có Tác phẩm đang mở, `effectiveTier()` trả `tierChoice ?? 'work'`, nên mục AI đã mở ở phạm vi Tác phẩm. Không cần command mới: nút của panel dùng `settings.ai_config.open`, và chỉ đổi nhãn khi có ghi đè.
- `src/i18n/vi.json:1422-1425` -- các khoá `panel.ai_translation.unconfigured_*`.
- Tests: `tests/frontend/aiBatchPanelRows.test.ts` đang chọn `data-ai-unconfigured`. Thêm test panel và test state.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/aiconfig.rs` + `src-tauri/tests/aiconfig_contract.rs` -- hàm lý do dùng chung với `ready`, thêm trường wire, test từng hàng Matrix ở tầng Rust.
- [x] `src/config/aiconfig.ts` + `src/aiConfigState.ts` -- mang `notReady` và danh sách ghi đè tới panel.
- [x] `src/panels/AiTranslationPanel.vue` + `src/i18n/vi.json` -- câu lý do, dòng ghi đè, MỘT nút `settings.ai_config.open`; khi có ghi đè, nhãn nút là "Mở Cài đặt ở phạm vi Tác phẩm này".
- [x] `tests/frontend/` -- phủ từng hàng Matrix và AC; test bấm nút khi có Tác phẩm đang mở thì mục AI mở ở tầng `work`, đi qua `settingsState` thật.

**Acceptance Criteria:**
- Given panel đang hiện lý do "cần khoá API" do ghi đè, when bấm "Trả về kế thừa" cho nhà cung cấp trong Cài đặt, then khối chưa sẵn sàng của panel biến mất mà không phải mở lại panel.
- Given không có Tác phẩm nào đang mở, when panel chưa sẵn sàng, then không có dòng ghi đè và không có nút phạm vi Tác phẩm.

## Design Notes

Tên nhà cung cấp trong câu lý do dùng nhãn nút của Cài đặt (`settings.ai_config.provider_*`) khi giá trị nằm trong `AI_PROVIDERS`, còn lại dùng nguyên văn giá trị. Tên trường trong dòng ghi đè dùng `settings.ai_config.field_*`.

## Verification

**Commands:**
- `npm run build && cd src-tauri && cargo test --test aiconfig_contract` -- xanh
- `npx vitest run tests/frontend/aiBatchPanelRows.test.ts` cùng các tệp test mới hoặc đã sửa -- xanh
- `npm run check:lint && npm run check:commands && npm run check:i18n && npm run check:tokens && npx vue-tsc --noEmit -p tsconfig.json` -- xanh

## Implementation Notes

- Rust `call_readiness` trả cả `ready` lẫn `not_ready`, và chặn bằng `endpoint_and_model_set`, cùng vị từ với lượt dịch. `key_missing` chỉ xuất hiện khi endpoint và mô hình đã có, keychain trả `Some(false)`, và nhà cung cấp cần khoá.
- Khi có Tác phẩm đang mở, Cài đặt vốn đã mở ở phạm vi Tác phẩm (`tierChoice ?? 'work'`), nên panel chỉ có một nút `settings.ai_config.open` và đổi nhãn khi có ghi đè (xem Plan Change Log).
- Dòng ghi đè liệt kê mọi trường `tier === 'work'`, kể cả trường không liên quan tới lý do (ví dụ nhiệt độ), đúng chữ "trường nào đang ghi đè" của Intent.
- Đối chứng đỏ bằng phép gỡ thật: bỏ `notReady` khỏi `applyReadiness` (5 test Matrix đỏ), bỏ `applyReadiness` khỏi `loadAiConfigSection` (AC1 đỏ), bỏ chốt "có Tác phẩm đang mở" của dòng ghi đè (AC2 đỏ), bỏ `resetAiConfigViewTier()` (test tầng `work` đỏ).

- Kiểm cuối: `aiconfig_contract` 31/31; `aiNotReadyReasons` cùng `aiBatchPanelRows` 28/28; lint, commands, i18n, tokens, fmt, vue-tsc đều xanh. `check:clippy` đỏ ở `src/core/tm/tmx.rs:248` (`chunks_exact` với cỡ khối hằng). Tệp đó story này không chạm. Lint mới có từ khi Homebrew cập nhật rustc lên 1.99.1 lúc 13:17 ngày 2026-10-26.

## Plan Change Log

- 2026-10-26, review lượt 1. Phát hiện: `settings.ai_config.open_work` làm đúng việc `settings.ai_config.open` đã làm. `resetAiConfigViewTier()` đưa `tierChoice` về `null`, nên `effectiveTier()` mặc định trả `work` khi có Tác phẩm đang mở. Gỡ lệnh `selectAiConfigViewTier('work')` cuối cùng thì test vẫn xanh. Bẫy ⚠️ trong Code Map cũ sai tiền đề.
  - Đã sửa: Code Map, Tasks, Verification. Bỏ command mới cùng port, dây `main.ts` và `openSettingsToAiConfigWork`. Panel còn một nút, đổi nhãn khi có ghi đè.
  - Trạng thái tránh được: hai nút cùng tác dụng nằm cạnh nhau, và một command thừa không có chốt canh nào bắt được.
  - KEEP: phần Rust (`call_readiness`, `not_ready`, test hợp đồng), `aiConfigNotReady` / `aiConfigWorkOverrides` / `aiConfigReadinessWorkIsOpen`, các câu lý do và dòng ghi đè, bộ test Matrix trong `aiNotReadyReasons.test.ts`, bản sửa `aiBatchPanelRows.test.ts`.

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 3 · false 6 · maybe-false 0.
- medium · bad_plan · `settings.ai_config.open_work` trùng tác dụng với `settings.ai_config.open` (`effectiveTier()` mặc định trả `work`), và lệnh chọn tầng cuối không chốt canh nào bắt được: sửa plan, gỡ command (Plan Change Log).
- low · patch · `call_readiness` tự viết lại phép kiểm endpoint/mô hình thay vì gọi `endpoint_and_model_set`: đã chặn bằng vị từ dùng chung.
- low · patch · doc của `ready` trỏ tới `call_ready` đã xoá: đã sửa.
- false · lượt nạp bị vượt mặt để tầng ở Toàn cục: `tierChoice` là `null` sau reset nên tầng mặc định là `work`; command đã bị gỡ.
- false · `openSettingsToAiConfigWork` chép lại `openSettingsToSection`: đã gỡ.
- false · command mới không có trong `isSettingsOwnedCommand`: đã gỡ.
- low · reject · `readinessWorkOpen` trùng `workIsOpen`: hai nguồn được đặt ở hai đường nạp khác nhau; gộp lại sẽ làm Cài đặt đổi tầng giữa chừng.
- false · nhãn nút chung không nói "AI": nhãn được giữ đúng như Intent.
- false · plan chưa chốt sổ: bước 5 làm việc này.
- false · nhánh lỗi giữ `effectiveProvider` cũ: khi `readiness = 'unknown'`, khối chưa sẵn sàng không hiện.

Lượt 2 (quick): high 0 · medium 0 · low 4 · false 2 · maybe-false 0.
- low · patch · `rustfmt` chạy trần đảo thứ tự `use` theo kiểu 2021: chạy lại `cargo fmt` (edition 2024), hunk đó biến mất.
- low · patch · chú thích đầu `aiNotReadyReasons.test.ts` nêu tên plan: đã xoá.
- low · reject · dòng ghi đè liệt kê cả trường không liên quan tới lý do: đúng chữ của Intent, nút vẫn mở đúng tầng.
- low · reject · thân tệp handoff còn tả `open_work`: tệp ghi chú làm việc, đã thêm dòng cuối đánh dấu phần không còn đúng.
- false · plan chưa chốt sổ: bước 5 làm việc này.
- false · test tầng `work` dựa vào reset: đúng hành vi đã kiểm (`settingsState.ts:251-254`, `aiConfigState.ts:202-205`).
