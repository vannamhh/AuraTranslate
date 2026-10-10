---
title: 'Rút đường gọi AI dùng chung cho dịch đơn và dịch lô'
type: 'refactor'
ticket: '12'
created: '2026-10-10'
status: 'built'
baseline_revision: '0acd35aa57e5483b3bb24ca02e65e589e97872f6'
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

**Problem:** 4-9 dựng đường lô song song với đường đơn của 4-8 mà không rút hàm chung (retro Epic 4 §B, action #8). Proofreader (9.1) sắp cần cùng đường gọi AI; không rút trước thì nó thành bản chép thứ ba.

**Approach:** Rút phần giống nhau thành hàm chung mà cả hai đường gọi, giữ nguyên hành vi, lỗi, dây Rust ↔ webview và chữ ký công khai. Phép kiểm chưa có ca nào chạm tới thì thêm ca canh, xanh TRƯỚC khi chuyển chỗ.

**Decisions (Ice, 2026-10-10):**
- `resolve_two_tiers`, `strip_bom` ngoài phạm vi (để 9.11).
- Chỉ năm chỗ của retro; đường gửi (`send_prepared_translate_call`/`send_prepared_batch_call`, dựng `TranslateRequest`) không rút ở đây.
- Giữ plan nguyên độ dài (2.587 token), không tách.

## Boundaries & Constraints

**Always:** đơn và lô giữ đúng thứ tự kiểm hiện có (store → work → cấu hình → segment/omitted → keychain); mã lỗi, `message_key`, params, cờ retryable không đổi; nhãn `ai_translate[segment]`/`ai_translate[batch]` trên stderr giữ nguyên (truyền vào helper); các ca C1, C2, C4, C6 của 9.9 xanh suốt quá trình.

**Never:** tách `commands/aitranslate.rs` thành thư mục hay chuyển code sang tệp khác (`ai_boundary.rs:174,1316` và `aiconfig_keychain_boundary.rs:193,472` ghim đúng tệp này, cấm `aitranslate/mod.rs`; đổi là một AD); đổi tên/chữ ký lệnh Tauri, item `pub` mà test dùng, hay export TS; dời ref module-level hoặc phép gán trong `reset*()` ra khỏi tệp của nó (`check:panel-refs` chỉ theo lời gọi cùng tệp); rút `send_prepared_*` hay phần dựng `TranslateRequest`; sửa C3, C5, C7, C8; nới assert hay thêm `?.` để xoá đỏ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Thiếu state, đơn | `wire::ai_translate_segment` trên app không `manage` state ghi prompt / `AiTranslateGeneration` | `Err` mã `ai_translate.record_state_missing` / `…generation_state_missing`, `MessageKey::Unknown`, params rỗng, không retryable | stderr mang `ai_translate[segment]` |
| Thiếu state, lô | như trên với `wire::ai_translate_batch` | như trên | stderr mang `ai_translate[batch]` |
| Chưa cấu hình | endpoint/model trống, hoặc keychain không có khoá | đơn và lô cùng trả NotConfigured như hiện tại | — |
| Huỷ | `state` khác `generating` | không gọi IPC huỷ | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/aitranslate.rs:244-314` `prepare_translate_call` / `:364-452` `prepare_batch_call` -- giống: guard store+work (251-252/371-372), đọc endpoint+model+NotConfigured (257-261/377-381), parse temperature/max_tokens (271-277/383-389), match keychain (292-296/410-414). Khác thật: đơn lỗi `ai_translate.segment_omitted`, lô xếp `Omitted`; lô chỉ đọc keychain khi còn hàng không omitted; đơn `assemble_and_record_prompt`, lô `…_with_tm` + `TmRowsCache`. Hình dạng gợi ý: `resolve_call_config(...) -> Result<Option<ResolvedCallConfig>, IpcError>` + `read_api_key() -> Result<Option<String>, IpcError>` (None = NotConfigured).
- `aitranslate.rs:807-818,905-916` (`record_state_missing`) và `:844-854,942-952` (`generation_state_missing`) trong `wire::ai_translate_segment` (798-886) / `wire::ai_translate_batch` (896-997) -- chỉ khác nhãn eprintln. Một helper `state_missing(tag, state_name, code) -> IpcError`. Không test hay script nào nhắc hai mã này; `tests/ai_translate_wire.rs:182,219` gọi hai wire qua MockRuntime, chỉ đường thành công.
- `src/aiTranslateState.ts:157-160` `cancelAiTranslate` / `src/aiTranslateBatchState.ts:322-325` `cancelAiTranslateBatch` -- thân giống hệt (`if (state.value !== 'generating') return; void cancelAiTranslateCall()`), chỉ khác ref `state`. Helper nhận ref, hai export giữ nguyên tên. Caller: `src/aiTranslateHandlers.ts:38-41`, `src/commands/index.ts:1074,4140-4143`.
- Tests ghim: `src-tauri/tests/ai_translate_contract.rs` (prepare đơn 535-970, lô 1872-2259; đọc `aitranslate.rs` theo đường dẫn ở 1326), `ai_translate_wire.rs`, `ai_boundary.rs`, `aiconfig_keychain_boundary.rs`; vitest `tests/frontend/aiTranslate.test.ts`, `aiTranslateBatch.test.ts`, `aiTranslateBatchResetWiring.test.ts`, `aiTranslateHandlersWiring.test.ts`, `aiTranslationPromoteErrorAlert.test.ts`.
- Không đổi: lệnh `wire::ai_translate_{segment,batch,cancel}` (`lib.rs:1284-1293`), state `AiTranslateGeneration` (`lib.rs:1578`), `PreparedTranslateCall`, `PrepareOutcome`, `PrepareBatchOutcome`, `PreparedBatchItem`, `run_translate_call`, `run_batch_call`, `batch_stopped_error`, `batch_panicked_error`, `single_run_outcome_wire`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/tests/ai_translate_wire.rs` -- thêm ca thiếu state cho hai wire (bảng I/O hàng 1-2); chạy XANH trên mã chưa đổi -- canh nhánh chưa ca nào chạm trước khi chuyển chỗ.
- [ ] Đếm ca NotConfigured (endpoint/model trống, keychain trống/không đọc được) ở cả đơn lẫn lô trong `ai_translate_contract.rs`; thiếu bên nào thì thêm, xanh trên mã chưa đổi.
- [ ] `src-tauri/src/commands/aitranslate.rs` -- rút `resolve_call_config`, `read_api_key`, `state_missing`; hai `prepare_*` và hai wire gọi chúng.
- [ ] `src/aiTranslateState.ts` + `src/aiTranslateBatchState.ts` -- helper huỷ dùng chung nhận ref; hai export gọi nó.
- [ ] Đối chứng: với từng ca canh mới, gỡ thật phép kiểm nó canh trong helper (giữ chữ ký), chạy đúng target, thấy đỏ đúng lý do; trả lại.

**Acceptance Criteria:**
- Given đơn và lô, when đọc mã, then cả hai gọi cùng hàm chung cho cấu hình, khoá API và lỗi thiếu state, và cả hai export huỷ gọi cùng helper.
- Given các test dịch đơn, dịch lô, huỷ, thử lại hiện có cùng ca C1, C2, C4, C6 của 9.9, when chạy các target ở Verification, then xanh, không assert nào bị sửa.
- Given ca canh mới, when gỡ phép kiểm nó canh, then chính ca đó đỏ.

## Implementation Notes

- Rust: `resolve_call_config` (guard store/work + cấu hình hai tầng, `None` = chưa cấu hình), `read_api_key`, `state_missing(tag, …)` riêng trong `aitranslate.rs`; nhãn stderr truyền vào nên giữ nguyên.
- TS: helper `cancelWhenGenerating` ở tệp mới `src/aiTranslateCancel.ts`, không đặt trong `config/aitranslate.ts` vì các mock vitest thay cả module đó sẽ để export mới thành `undefined`.
- Ca canh mới, xanh trên mã chưa đổi: 4 ca thiếu state (`ai_translate_wire`), 1 ca lô `endpoint`/`model` rỗng (`ai_translate_contract`; bên đơn đã có), 2 ca huỷ khi không `generating` (vitest; hai ca `reset` sẵn có không chạm hàm huỷ).
- Đối chứng: `state_missing` bỏ qua `code` ⇒ 4 ca thiếu state đỏ vì sai mã; gỡ phép kiểm `model` rỗng ⇒ ca lô đỏ (bản đầu của ca vẫn xanh vì không có khoá thì keychain cũng trả NotConfigured, nên ca lưu khoá trước); gỡ phép kiểm `generating` trong helper ⇒ 2 ca vitest đỏ (`expected "vi.fn()" to not be called`); trả lại cả ba.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens `quick`): high 0 · medium 0 · low 4 · false 3 · maybe-false 0.
- low · patch — `resetAiTranslate`/`resetAiTranslateBatch` còn chép phép kiểm `generating` mà helper đã giữ; sửa: hai `reset*()` gọi helper.
- low · patch — bốn dòng `return Err(state_missing(…))` và lời gọi mới trong `ai_translate_wire.rs` vượt 100 cột trong khi khối cũ được ngắt; sửa: ngắt tay (tệp không sạch rustfmt từ baseline: 18/18/143 hunk).
- false · rejected — helper nhận `{ readonly value: string }` thay vì `Ref`: phép so và chuỗi `'generating'` chuyển nguyên văn, hai ref có union khác nhau nên kiểu chung là chuỗi; không doc là mặc định của AGENTS.md.
- false · rejected — "không có bằng chứng xanh-trước và đỏ-khi-gỡ": đã chạy, ghi ở Implementation Notes; phần đòi tick checkbox là sửa plan.
- low · rejected — ca lô không phủ `trim()`/chiều ngược: khoảng hở có từ trước ở cả đường đơn (ca :581 cũng chỉ đặt endpoint), nay là một phép kiểm chung; thêm ca là phạm vi mới.
- false · rejected — mất khối 🔵 QUYET DINH 6: lý do còn ở một dòng bất biến tiếng Anh trên `resolve_call_config`; lịch sử quyết định vào commit theo AGENTS.md.
- low · rejected — test không assert nhãn stderr: diff truyền đúng `"segment"`/`"batch"` ở mỗi wire; bắt stderr cần thêm hạ tầng test cho một dòng chẩn đoán trên nhánh không bao giờ chạy ở sản phẩm.

## Verification

**Commands:**
- `npm run build && cargo test --manifest-path src-tauri/Cargo.toml --test ai_translate_contract --test ai_translate_wire --test ai_boundary --test aiconfig_keychain_boundary` -- expected: xanh.
- `npx vitest run tests/frontend/aiTranslate.test.ts tests/frontend/aiTranslateBatch.test.ts tests/frontend/aiTranslateBatchResetWiring.test.ts tests/frontend/aiTranslateHandlersWiring.test.ts tests/frontend/aiTranslationPromoteErrorAlert.test.ts` -- expected: xanh.
- `npm run check:panel-refs` -- expected: đạt.
