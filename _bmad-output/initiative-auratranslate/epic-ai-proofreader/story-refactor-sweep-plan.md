---
title: 'Refactor sweep'
type: 'refactor'
ticket: '11'
created: '2026-10-12'
status: 'built'
baseline_revision: '6ec352f47c47cb0fb8d53f5a37fe1629c0dac286'
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

**Problem:** `wire::ai_translate_segment` và `wire::ai_translate_batch` lấy `generation_state.next()` SAU `prepare_*` (đọc segment + keychain), nên `ai_translate_cancel` đến trong khe đó bị lượt dịch ghi đè, lượt dịch chạy tới cùng. Đây là mục dọn duy nhất có `Chủ: Story 9.11` trong `deferred-work.md` (nguồn: review 9.1).

**Approach:** Dời lượt lấy thế hệ lên trước `prepare_*` ở cả hai đường dịch, đúng khuôn `commands/proofread.rs::ai_proofread_segments`, với ca canh khe đỏ được trước khi dời.

**Decisions (Ice, 2026-10-12):**
- `strip_bom` (5 bản): KHÔNG LÀM. Thân một dòng gọi std; spec 3.10 cố ý chép để glossary/promptset không phụ thuộc `core::segment`; "chỉ cắt ở đầu" đã có ca canh ở `glossary_exchange_contract.rs` và `segment_contract.rs`.
- `resolve_two_tiers` (3 bản): KHÔNG LÀM. Giống hình dạng, khác ngữ nghĩa (`apply_override` ×2, `apply_merge` ×1), khác kiểu trả và kiểu lỗi, mỗi kho một chỗ gọi; generic đẩy phép kiểm "tầng lạ thì lỗi" ra xa chỗ dùng.
- Giữ một plan.

## Boundaries & Constraints

**Always:** mã lỗi, `message_key`, params, cờ retryable, dây `AiTranslateOutcomeWire` và nhãn stderr `ai_translate[segment]`/`ai_translate[batch]` không đổi; `aitranslate.rs` vẫn là một tệp (`ai_boundary.rs`, `aiconfig_keychain_boundary.rs` ghim đường dẫn); ca C1, C2, C4, C6 của 9.9 và mọi test Epic 9 xanh suốt.

**Never:** nhận tính năng hay nợ kiểm tay của Epic 9 (các mục `Chủ: Epic 9` giữ nguyên chủ); sửa `strip_bom`/`resolve_two_tiers`; dọn chú thích ngoài dòng đang sửa.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Huỷ giữa prepare (đơn) | `ai_translate_cancel` đến khi `ai_translate_segment` còn kẹt trong `prepare_translate_call` | kết quả `Cancelled`, provider không được gọi | không lỗi |
| Huỷ giữa prepare (lô) | như trên với `ai_translate_batch` | `Cancelled`, không segment nào được dịch | không lỗi |
| Không huỷ | gọi bình thường | hành vi y như trước | không đổi |
| Prepare lỗi / NotConfigured | lỗi hoặc chưa cấu hình | trả đúng lỗi/`NotConfigured` như trước; thế hệ đã tăng (giống proofread) | không đổi |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/aitranslate.rs:870-883` -- đơn: `match prepared` rồi `try_state::<AiTranslateGeneration>()` + `next()`; dời khối `try_state`+`next()` lên trước khối `let prepared = { … }`.
- `src-tauri/src/commands/aitranslate.rs:959-972` -- lô: cùng thứ tự với `prepare_batch_call`; dời như trên.
- `src-tauri/src/commands/aitranslate.rs:495-505` -- `AiTranslateGeneration::{next, is_current}`; không đổi.
- `src-tauri/src/commands/proofread.rs:486-495` -- khuôn đúng (`next()` trước prepare, chú thích một dòng tiếng Anh); chép cách đặt, không dùng chung kiểu.
- `src-tauri/tests/ai_translate_wire.rs:120-145` -- app giả có `OpenWorkState`; giữ khoá `OpenWorkState` ở luồng test để ghim lượt gọi trong prepare, gọi `ai_translate_cancel`, nhả khoá. Theo `KEYCHAIN_KEY_TEST_LOCK` (:172) như các ca sẵn có.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/tests/ai_translate_wire.rs` -- thêm hai ca huỷ-giữa-prepare (đơn, lô); chạy trên mã cũ phải đỏ đúng lý do "chạy tới cùng thay vì Cancelled" -- ca canh trước, dời sau.
- [ ] `src-tauri/src/commands/aitranslate.rs` -- dời `try_state`+`next()` lên trước prepare ở hai wire; chú thích một dòng như proofread.
- [ ] `_bmad-output/initiative-auratranslate/deferred-work.md` -- đóng mục `Chủ: Story 9.11` bằng `→ ✅ ĐÃ ĐÓNG <date> (Story 9.11)`.

**Acceptance Criteria:**
- Given hai ca mới, when gỡ thật lượt dời (đặt `next()` về sau prepare), then đúng hai ca đó đỏ và chỉ vì kết quả không phải `Cancelled`.
- Given các target ở Verification, when chạy, then xanh.
- Given `deferred-work.md`, when `npm run check:debt-owner`, then xanh và không còn mục mở có `Chủ: Story 9.11` cuối.

## Implementation Notes

- `next()` đứng SAU phép kiểm `LastAssembledPromptState`, không ở đầu hàm: các ca `*_without_prompt_record_state_*` dựng app thiếu cả hai state và chờ `record_state_missing` trước.
- `ai_translate_cancel` thành generic `<R: tauri::Runtime>` (ngoài Code Map) để ca `MockRuntime` gọi được; đăng ký trong `lib.rs` và hành vi không đổi, cùng dạng với `ai_translate_segment`.
- Ca huỷ ghim prepare bằng khoá `OpenWorkState` và chờ cố định 500 ms vì `next()`/`is_current()` riêng tư; máy quá tải có thể làm ca đỏ giả (chỉ đỏ giả, không xanh giả).
- Đối chứng: đặt lại khối `next()` sau `match prepared` ở cả hai wire ⇒ đúng hai ca huỷ đỏ với `Done { usage: None }`, sáu ca còn lại xanh; khôi phục ⇒ xanh.
- Thêm hai ca `*_without_a_saved_key_returns_not_configured` cho hàng NotConfigured của ma trận ở tầng wire.
- Lượt vá review: kiểm `should_cancel()` trước gửi đặt trong `send_prepared_translate_call`, không trong `run_translate_call`, vì hai ca `ai_translate_contract` dùng `calls.fetch_add(1) >= 1` làm `should_cancel` và đỏ khi có thêm một lần hỏi ở đầu; ghim `ai_translate_cancel` trong `ipc_contract.rs` đổi sang `AppHandle<R>`; máy chủ SSE đếm kết nối, hai ca huỷ khẳng định 0 kết nối.
- Đối chứng sau vá: xoá kiểm trước gửi ⇒ ca huỷ đơn đỏ `left: 1, right: 0` kết nối, ca lô xanh nhờ kiểm từng câu sẵn có của `run_batch_call`; khôi phục ⇒ `ai_translate_wire` 10/10.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 2 · medium 2 · low 2 · false 0 · maybe-false 0 (6 phát hiện; hai hàng high cùng một gốc).
- high · patch — `ai_translate_cancel` generic làm đỏ `ipc_contract.rs:2191` (ghim `app: tauri::AppHandle`, đã chạy thấy đỏ); sửa chuỗi ghim thành `AppHandle<R>` như ghim `ai_translate_segment`.
- high · patch (gộp với trên) — đổi chữ ký ngoài Code Map không đối chiếu ghim nguồn trong `ipc_contract.rs`; cùng gốc.
- medium · patch — `run_translate_call` không hỏi `should_cancel()` trước `provider.translate`; huỷ giữa prepare vẫn POST kèm khoá (`client.rs:353-364`, kiểm đầu tiên ở :381), trái hàng ma trận "provider không được gọi"; thêm kiểm trước gửi như `run_batch_call:646`.
- medium · patch — hai ca huỷ chỉ khẳng định `Cancelled`, xanh dù yêu cầu đã đi; đếm kết nối máy chủ SSE vòng lặp và khẳng định 0.
- low · reject — hai ca NotConfigured xanh ở cả hai thứ tự: chúng phủ đầu ra của hàng NotConfigured ở tầng wire (trước đây không có), không canh việc dời; canh "thế hệ đã tăng" là ghim một hệ quả đã chấp nhận, thêm độ phức tạp.
- low · reject — "sáu ca còn lại" đúng lúc đo (tệp có 8 ca), sửa là sửa plan; số đối chứng sẽ ghi lại sau lượt vá.

## Design Notes

Lấy thế hệ trước prepare làm một lượt gọi lỗi/`NotConfigured` cũng vô hiệu lượt dịch đang chạy trước nó; proofread đã chấp nhận đúng hệ quả này, và webview không chạy hai lượt dịch cùng lúc.

## Verification

**Commands:**
- `npm run build` (nếu chưa có `dist/`) rồi `cargo test --test ai_translate_wire --test ai_translate_contract --test ai_boundary --test aiconfig_keychain_boundary` -- expected: xanh.
- `npm run check:debt-owner` -- expected: xanh.
