---
title: 'System prompt cố định, sửa được, gửi trước thân bộ prompt'
type: 'feature'
ticket: ''
created: '2026-10-26'
status: 'built'
baseline_revision: 'd4c3c71a80b1eebeb5f232e8310b571d4200ff25'
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

**Problem:** Mọi lời gọi dịch AI hiện chỉ gửi một message `user` duy nhất, nội dung là thân bộ prompt đã điền biến (`core/ai/client.rs:213`). Vì vậy lệnh chung kiểu "dịch sang tiếng Việt, chỉ trả về bản dịch" phải lặp lại trong mọi bộ prompt. Bộ prompt nào quên lệnh đó thì AI trả về cả lời giải thích, và bản dịch thành sai.

**Approach:** Có một system prompt dùng chung, đứng trước thân bộ prompt trong mọi lời gọi dịch. App mang sẵn một văn bản mặc định bằng tiếng Việt và Ice sửa được. Hàm lắp prompt vẫn là hàm thuần và vẫn trả về đầy đủ những gì sẽ gửi đi (AD-14). Màn Xem prompt (FR71) hiện cả system prompt.

**Đã chốt (Ice, 2026-10-26):**
- Làm trước plan `plan-thu-vien-prompt-mau-theo-the-loai.md`. Plan mẫu sẽ bỏ khỏi thân mẫu những phần system prompt đã nói.
- Gửi bằng một message `role: "system"` riêng, đứng trước message `user`. `TranslateRequest`, `PreparedTranslateCall` và `AssembledPromptRecord` mang thêm system prompt.
- Lưu ở trường mới `AiConfigField::SystemPrompt`. Trường này có Toàn cục và ghi đè theo Tác phẩm (AD-18), không cần migration. Sửa bằng textarea nhiều dòng trong phần tinh chỉnh AI của Cài đặt.
- Không có dòng nào thì dùng văn bản mặc định của app. Lưu ô rỗng nghĩa là không gửi system prompt. Nút "Khôi phục mặc định" xoá dòng ở tầng đang sửa: ở Toàn cục thì quay về văn bản của app, ở Tác phẩm thì kế thừa Toàn cục.
- Chỉ áp cho dịch: dịch một đoạn, dịch lô, gửi lại. Proofread không gửi system prompt.
- Giữ một plan dù vượt 1600 token. Plan 1418 từ, chưa đếm được bằng tiktoken vì máy không có gói này.

## Boundaries & Constraints

**Always:**
- Lời gọi dịch thực tế gửi đi đúng system prompt và đúng thân bộ prompt mà màn Xem prompt hiện, từng byte. `mark_prompt_as_sent` so khớp cả hai.
- Gửi lại (`prepare_resend_call`) dùng system prompt đã ghi trong bản ghi của lần gửi trước, không đọc lại cấu hình hiện tại.
- Văn bản system prompt được giữ nguyên như Ice gõ, kể cả xuống dòng và khoảng trắng; không trim bên trong.
- Một ô system prompt rỗng không bao giờ sinh ra một message `system` rỗng.

**Never:**
- Không đổi schema: bảng `ai_config` không có CHECK trên `key`.
- Không đổi định dạng đầu ra cố định của proofread (`rag.rs:767-791`).
- Không thêm thử lại tự động (AD-14 · bảng §Gọi AI).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mặc định | Chưa tầng nào có dòng system prompt | Gửi văn bản mặc định của app | — |
| Ice sửa | Toàn cục có văn bản riêng | Gửi văn bản của Ice | — |
| Ghi đè Tác phẩm | Toàn cục là A, Tác phẩm là B | Gửi B; xoá dòng của Tác phẩm thì gửi lại A | — |
| Tắt | Ice lưu ô rỗng | Không có message `system` | — |
| Gửi lại | Ice sửa system prompt sau lần dịch đầu, rồi bấm Gửi lại | Gửi system prompt cũ, khớp với Xem prompt | — |
| Tắt ở một Tác phẩm | Toàn cục có văn bản, Tác phẩm lưu ô rỗng | Tác phẩm đó không gửi `system`; Tác phẩm khác vẫn gửi | — |
| Proofread | Có system prompt, chạy proofread | Request proofread chỉ có message `user` như cũ | — |
| Lỗi lưu | Ghi `ai_config` lỗi IO | Báo `tError`, bản nháp giữ nguyên trong ô | Không ghi một phần |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/ai/rag.rs:704` -- `assemble_prompt` trả về `(String, InjectionLedger)`. Kết quả mang thêm system prompt; `pieces` (`:167`) vẫn chỉ mô tả message `user`.
- `src-tauri/src/commands/aiprompt.rs:377,400,458,525,599,633` -- `AssembledPromptRecord`, trạng thái trong bộ nhớ, đường lắp chung cho Xem prompt, dịch và dịch lô; `mark_prompt_as_sent` so khớp từng byte.
- `src-tauri/src/commands/aitranslate.rs:215,255,333,395,489,602,952` -- `PreparedTranslateCall`, `resolve_call_config` (proofread cũng dùng), chuẩn bị dịch / gửi lại / dịch lô, các chỗ dựng `TranslateRequest`.
- `src-tauri/src/ports/translation_provider.rs:73-87` -- `TranslateRequest`.
- `src-tauri/src/core/ai/client.rs:191,213` -- `messages: [ChatMessage; 1]` cố định một phần tử.
- `src-tauri/src/core/aiconfig/mod.rs:48-81,148,227` -- enum, `ALL`, `as_str`, `PROVIDER_DEFAULT`, `validate_field`. Mọi validator hiện đều trim, nên system prompt cần validator riêng giữ nguyên văn.
- `src-tauri/src/core/aiconfig/store.rs:58` -- `resolve_two_tiers`. `commands/aiconfig.rs:262,283,321` -- lưu, lưu nhiều trường, xoá ghi đè.
- `src/config/aiconfig.ts:14`, `src/aiConfigState.ts:21,306,630` -- union TS, `AI_CONFIG_FIELDS`, `isAiConfigValueValid`, `applyAiConfigPreset` (preset không ghi system prompt).
- `src/SettingsOverlay.vue:251,503-515` -- `AI_TEXT_FIELDS`, đang render `<input>` một dòng. Lấy textarea của `PromptLibraryOverlay.vue` làm tiền lệ.
- `src/config/aiprompt.ts:91,120`, `src/AiPromptInspectorOverlay.vue`, `src/aiPromptInspectorState.ts` -- wire và màn Xem prompt.
- Không đổi: `assemble_proofread_batch_prompt` (`rag.rs:805`), schema `ai_config`, lệnh `prompt_set_*`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/ai/default_system_prompt.txt` (mới) + `include_str!` trong `core/ai/` -- văn bản mặc định tiếng Việt (Design Notes) -- `check:i18n` kiểm A cấm chữ có dấu trong chuỗi `.rs`; đây là dữ liệu gửi AI, không phải chuỗi giao diện.
- [x] `src-tauri/src/core/aiconfig/mod.rs` -- variant `SystemPrompt` (`"system_prompt"`), thêm vào `ALL`/`as_str`; validator riêng giữ nguyên văn và nhận chuỗi rỗng -- mọi validator khác đều trim.
- [x] `src-tauri/src/commands/aiconfig.rs` -- `ai_config_get` mang thêm văn bản mặc định để webview hiện và so; một đường xoá dòng Toàn cục cho "Khôi phục mặc định" (dùng `store::clear_field`), Tác phẩm dùng `ai_config_clear_override` sẵn có.
- [x] `src-tauri/src/ports/translation_provider.rs`, `core/ai/client.rs` -- `TranslateRequest.system: Option<&str>`; `build_request_body` gửi `[system?, user]` -- không có `system` khi `None` hoặc rỗng.
- [x] `src-tauri/src/core/ai/rag.rs`, `commands/aiprompt.rs` -- phân giải system prompt (dòng Tác phẩm → dòng Toàn cục → mặc định; rỗng ⇒ `None`), đưa vào kết quả lắp và `AssembledPromptRecord`; `mark_prompt_as_sent` so cả hai; wire `AssembledPromptWire` thêm `system`.
- [x] `src-tauri/src/commands/aitranslate.rs` -- dịch và dịch lô lấy system từ kết quả lắp; gửi lại lấy từ bản ghi; `proofread.rs:287` đặt `system: None`.
- [x] `src/config/aiconfig.ts`, `src/aiConfigState.ts`, `src/SettingsOverlay.vue`, `src/i18n/vi.json` -- union + `AI_CONFIG_FIELDS`; loại khỏi `AI_TEXT_FIELDS`, render textarea riêng (lưu khi rời ô, như thân bộ prompt), dòng trạng thái "Mặc định của app" / "Đã sửa" / "Tắt", nút Khôi phục mặc định; preset không chạm trường này.
- [x] `src/config/aiprompt.ts`, `src/AiPromptInspectorOverlay.vue`, `src/aiPromptInspectorState.ts` -- khối "System prompt" trên các mảnh, hoặc dòng "Không gửi system prompt".
- [x] `src-tauri/tests/ai_translate_contract.rs` -- viết lại ca `…verbatim_as_the_user_message_content` thành hai ca: có system ⇒ `messages` = `[system, user]`, không có ⇒ `[user]`; ca gửi lại dùng system của bản ghi; ca byte-exact gồm cả system.
- [x] `src-tauri/tests/ai_prompt_contract.rs`, `ai_proofread_contract.rs`, `tests/frontend/aiConfigState.test.ts`, `settingsOverlayAiConfigTierRender.test.ts`, `aiPromptInspectorScreen.test.ts` -- ma trận I/O: mặc định, sửa, ghi đè, tắt ở Tác phẩm, proofread không có `system`, giữ nguyên văn xuống dòng.
- [x] Đối chứng đỏ: gỡ thật trường `system` khỏi `build_request_body` ⇒ ca `[system, user]` đỏ; cho gửi lại đọc cấu hình hiện tại thay bản ghi ⇒ ca gửi lại đỏ.

**Acceptance Criteria:**
- Given cài mới, when dịch một đoạn bằng AI rồi mở Xem prompt, then thấy khối System prompt là văn bản mặc định, trên thân bộ prompt.
- Given Ice sửa system prompt ở Cài đặt, when dịch đoạn kế tiếp, then request gửi văn bản mới và Xem prompt hiện đúng văn bản đó.
- Given Ice bấm Khôi phục mặc định ở Toàn cục, when mở lại Cài đặt, then ô hiện văn bản của app với trạng thái "Mặc định của app".

## Design Notes

Văn bản mặc định (bản đề xuất, Ice sửa được sau khi cài):

```
Bạn là dịch giả chuyên nghiệp, dịch sang tiếng Việt.
Chỉ trả về bản dịch của đoạn được yêu cầu: không giải thích, không ghi chú, không chép lại nguyên văn.
Giữ số dòng và dấu câu tương ứng với đoạn nguồn.
Khi có danh sách thuật ngữ, dùng đúng bản dịch đã cho.
```

## Implementation Notes

- Văn bản mặc định nằm ở `core/aiconfig/default_system_prompt.txt`, không ở `core/ai/`. Lý do: `commands/aiconfig` mà gọi tới `core::ai` thì guard ranh giới AD-13 (`ai_boundary`) đỏ.
- Chữ ký `assemble_prompt` giữ nguyên, vì có 24 ca test gọi hàm này. System prompt được phân giải bằng hàm thuần `resolve_system_prompt`, đặt ngay cạnh lời gọi lắp trong `assemble_and_record_prompt_with_tm`. Đó là lối vào lắp duy nhất của Xem prompt, dịch và dịch lô, nên bản ghi vẫn là đúng thứ sẽ gửi đi.
- "Khôi phục mặc định" không dùng lệnh mới. `ai_config_clear_override` nhận thêm `tier` tuỳ chọn (mặc định `work`), nên phần đăng ký lệnh ở `lib.rs` không đổi.
- Chưa có dòng nào thì `ai_config_get` trả văn bản mặc định kèm `tier: "global"`, và wire mang thêm `default_system_prompt`. Webview suy trạng thái Mặc định / Đã sửa / Tắt bằng cách so với văn bản đó.
- Văn bản chỉ có khoảng trắng được coi là tắt; văn bản khác giữ nguyên văn.
- `HANDLER_ATTR_FLOOR` của `check:commands` được nâng từ 146 lên 157.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 2 · low 2 · false 4 · rejected-low 2.
- low, rejected — `assemble_prompt` không mang system prompt: hàm thuần `resolve_system_prompt` nằm cạnh lời gọi lắp, ở lối vào lắp duy nhất, nên bản ghi = thứ gửi đi. Đổi chữ ký kéo theo 24 ca gọi, người dùng không gặp lỗi nào.
- false — tệp mặc định nằm ở `core/aiconfig/`: lệch có lý do (guard AD-13), đã ghi ở Implementation Notes. Sửa nó nghĩa là sửa plan.
- low, patch — hàm thuần `ai_config_clear_override` không còn được lệnh gọi, nên ca test của nó không canh đường lệnh thật ⇒ cho nó gọi `ai_config_clear_field`.
- low, patch — doc-comment ghi "Năm trường" ở `commands/aiconfig.rs:132` và `core/aiconfig/mod.rs:45`.
- false — sổ plan chưa cập nhật: phần này do người điều phối ghi, đã ghi ở đây.
- medium, patch — system prompt chỉ có khoảng trắng vẫn gửi thành message `system` trống và hiện "Đã sửa" ⇒ coi `trim()` rỗng là tắt.
- medium, patch — đường dịch lô không có ca test nào có system prompt, nên thay seam bằng `None` vẫn xanh ⇒ thêm ca dịch lô và đối chứng đỏ. Riêng "Lỗi lưu, không ghi một phần": false, vì lượt lưu chỉ ghi một dòng.
- false — sàn `HANDLER_ATTR_FLOOR` tăng 11: sàn là mức tối thiểu, nâng lên là chặt hơn; `check:commands` xanh.
- false — dòng dài trong `SettingsOverlay.vue`: `eslint src` xanh ở phase 2.
- low, rejected — thứ tự import `restoreAiConfigSystemPrompt`: chỉ là thẩm mỹ, lint không có luật sắp xếp.

## Verification

**Commands:**
- `cargo test --test ai_translate_contract --test ai_rag_contract --test ai_prompt_contract --test ai_proofread_contract` -- expected: xanh (sau `npm run build`).
- `npx vitest run tests/frontend/aiConfigState.test.ts tests/frontend/settingsOverlayAiConfigTierRender.test.ts tests/frontend/aiPromptInspectorScreen.test.ts tests/frontend/aiTranslateResendCall.test.ts` -- expected: xanh.
