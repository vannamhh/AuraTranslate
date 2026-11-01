---
title: 'Câu dịch tham khảo dưới Ngữ cảnh ở Bảng chờ Glossary'
type: 'feature'
ticket: ''
created: '2026-11-01'
status: 'built'
baseline_revision: '53cd08d729bd43c7e9fb25309c47a38e72eaa49b'
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

**Problem:** Câu Ngữ cảnh ở Bảng chờ Glossary là tiếng Trung hoặc tiếng Anh nên Ice thường đọc không hiểu, khó quyết nhận hay bỏ ứng viên.

**Approach:** Dưới câu Ngữ cảnh của hàng đang mở: câu đã có bản dịch trong Tác phẩm thì hiện luôn; chưa có thì có nút `Dịch thử`, bấm mới gọi AI một lần. Ice chọn phương án này ngày 2026-11-01.

## Boundaries & Constraints

**Always:** Câu dịch ghi rõ nguồn (bản dịch trong Chương, hoặc AI dịch để tham khảo). AI chỉ chạy theo cú bấm (AD-15), truyền qua `Channel`, không tự thử lại, huỷ được (AD-22). Prompt do một hàm thuần lắp (AD-14). Nút có điểm chuột qua `dispatch` (AD-62). Dùng cho cả Tác phẩm `zh` và `en`. Lần gọi này có bộ đếm thế hệ riêng, không chạm lô dịch đang chạy.

**Never:** Không ghi `target_text`, Glossary, TM, không vào lịch sử gọi AI của panel dịch. Không tự gọi AI khi chuyển hàng. Không thêm phím tắt.

**Decisions (Ice, 2026-11-01):** Prompt cố định ngắn, chỉ gồm yêu cầu dịch và câu cần dịch, không dùng Bộ prompt, Glossary hay TM. Câu AI đã dịch giữ trong bộ nhớ tới khi đóng Tác phẩm, không migration, không ghi đĩa.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Có bản dịch | Câu khớp một segment còn sống, không bị lược, `target_text` khác rỗng | Hiện bản dịch, nhãn nguồn là Chương, không có nút | Không gọi AI |
| Ngữ cảnh bị cắt | `context_example` đúng 200 ký tự | Khớp theo tiền tố 200 ký tự | Không khớp thì coi như chưa có bản dịch |
| Chưa có bản dịch | Không khớp | Hiện nút `Dịch thử` | Không tự gọi |
| AI chạy | Bấm nút | Chữ chảy dần, có nút Huỷ | Lỗi hiện qua `tError()`, bấm lại được |
| Chưa cấu hình AI | Thiếu endpoint, model hoặc khoá | Nút tắt, kèm dòng lý do và lối sang Cài đặt | Trước mọi lời gọi mạng |
| Chuyển hàng khi đang chạy | Con trỏ đổi, hoặc đóng lớp phủ | Huỷ lần gọi, chữ muộn không rơi vào hàng khác | Không báo lỗi |
| Không có Ngữ cảnh | `context_example` là `NULL` | Không hiện gì | Không gọi lệnh |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/ai/rag.rs` -- hàm thuần lắp prompt, mẫu `assemble_proofread_batch_prompt` (:965).
- `src-tauri/src/commands/aitranslate.rs` -- `resolve_call_config` (:311), `read_api_key` (:718), `run_translate_call` (:667), `PreparedTranslateCall`; `ConnectionCheckGeneration` (:695) là mẫu bộ đếm riêng. Đừng dùng `AiTranslateGeneration` hay `ai_translate_cancel`, chúng dừng lô đang chạy.
- `src-tauri/src/commands/proofread.rs` -- `prepare_proofread_call` (:137) là mẫu `NotConfigured` trước mạng.
- `src-tauri/src/core/glossary/scan.rs` -- `CONTEXT_EXAMPLE_CHAR_LIMIT` 200, cắt không để dấu.
- `src-tauri/src/lib.rs` -- `generate_handler!` (:1120), `app.manage` (:1723).
- `src-tauri/tests/ipc_contract.rs`, `tests/config_invariants.rs` -- bảng lệnh và số đếm thuộc tính.
- `src/config/aitranslate.ts` -- mẫu adapter; `src/aiConfigState.ts` -- `aiConfigReadiness`.
- `src/glossaryQueueState.ts`, `src/GlossaryQueueOverlay.vue` -- state và khối `.gq-ctx`.
- `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json` -- command, `CommandDeps`, nhãn.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/ai/rag.rs` -- thêm hàm thuần lắp prompt tham khảo -- AD-14.
- [ ] `src-tauri/src/commands/glossary.rs` -- thêm lệnh đọc bản dịch có sẵn của một câu, chạy async, lọc `retired_at IS NULL`, `is_omitted = 0`, `target_text` khác rỗng -- `source_text` không có chỉ mục nên quét cả bảng, không được chặn luồng chính.
- [ ] `src-tauri/src/commands/aitranslate.rs` -- thêm lệnh dịch tham khảo, lệnh huỷ và bộ đếm riêng -- không chạm lô dịch.
- [ ] `src-tauri/src/lib.rs`, `src-tauri/tests/ipc_contract.rs`, `src-tauri/tests/config_invariants.rs` -- đăng ký lệnh và cập nhật bảng.
- [ ] `src-tauri/tests/glossary_context_reference_contract.rs` -- phủ ma trận phía Rust bằng `TranslationProvider` giả, gồm ca không ghi đĩa và ca không dừng lô.
- [ ] `src/config/glossaryContextReference.ts` -- adapter, không ném lỗi.
- [ ] `src/glossaryQueueState.ts` -- state theo từng ứng viên, chống chữ muộn khi đổi hàng, reset cùng `resetGlossaryQueue`.
- [ ] `src/commands/index.ts`, `src/main.ts` -- hai command dịch và huỷ, có `pointer` qua nút.
- [ ] `src/GlossaryQueueOverlay.vue`, `src/i18n/vi.json` -- khối câu dịch và nhãn nguồn.
- [ ] `tests/frontend/glossaryContextReference.test.ts` -- phủ ma trận phía webview.
- [ ] `e2e/specs/glossary-context-reference.e2e.mjs` -- nhánh có bản dịch và nhánh chưa cấu hình AI.

**Acceptance Criteria:**
- Given một hàng có bản dịch trong Tác phẩm, when mở hàng, then câu dịch hiện kèm nhãn nguồn mà không có lời gọi mạng nào.
- Given lô dịch đang chạy, when bấm và huỷ `Dịch thử`, then lô chạy tiếp.
- Given đã bấm `Dịch thử`, when lần gọi xong, then `project.db` không đổi.

## Implementation Notes

- Lệnh mới: `glossary_context_translation` (async, khớp `substr(source_text,1,200) = context_example`), `ai_context_reference_translate`, `ai_context_reference_cancel`; bộ đếm `ContextReferenceGeneration` riêng, đo bằng ca huỷ không đụng `AiTranslateGeneration`.
- Prompt lắp ở `rag.rs::assemble_context_reference_prompt`; `aitranslate.rs` không được gọi thẳng `core::ai::rag` (AD-13) nên đi qua `aiprompt.rs::context_reference_prompt`, thêm một tên vào `ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM` (18 lên 19).
- `send_prepared_translate_call` tách `send_with_cancel` nhận closure huỷ; hai đường dùng chung một thân.
- State theo `candidate.id` nên chữ muộn không rơi vào hàng khác; câu AI giữ trong `aiReplies` tới `resetGlossaryQueue`, kết quả tra bản dịch Chương xoá mỗi lần mở lớp phủ.
- `check-layout` `FILE_FLOOR` 180 lên 193 vì thêm một tệp `.ts` (số tệp `src/**` 226).
- Lượt e2e đầu ngay sau khi dịch binary `wdio` đỏ, không kịp bắt nguyên văn; ba lượt liền sau trên cùng binary xanh.
- `npm run build` đỏ ở `scripts/gen-zh-fold.mjs` từ `afdc4205`, không do lượt này; `dist/` là bản cũ khi chạy `cargo test`.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens `quick`): high 0 · medium 1 · low 2 · false 2 · maybe-false 1. Nhóm gộp theo gốc: hai phát hiện về guard cùng một gốc.

- medium · patch — Ca "không ghi đĩa" và ca "không dừng lô" không đi qua `wire::ai_context_reference_translate` tới `send_with_cancel`: app mock thiếu `Store` và `OpenWorkState` nên lệnh trả sớm, kết quả bị bỏ bằng `let _`. Vá bằng cách chạy lệnh thật tới bước gửi.
- low · patch — `translateGlossaryContextReference` không chặn hàng đã ở `ai` và không kiểm lớp phủ đang mở; command có thể gán phím ở Cài đặt nên gọi lại tốn lượt được. Vá một điều kiện.
- low · patch — Vài doc comment mới chỉ nhắc lại chữ ký, trái luật comment. Xoá.
- false — Thiếu `pointer` ở hai command: Kiểm M chấp nhận `@click` là đúng một `dispatch` literal, cả hai nút đều có, `check:commands` xanh.
- false — `trim(target_text) <> ''` chặt hơn plan: hàng chỉ có khoảng trắng đúng là chưa có bản dịch, ca test đã phủ, không có kết quả xấu.
- maybe-false · bỏ — Huỷ rồi bấm lại rất nhanh có thể làm lần gọi mới bị huỷ do thứ tự IPC. Nếu đúng chỉ là low: hàng về nút, không lỗi, bấm lại được. Cần đo thứ tự hai lệnh mới xác định được.

## Design Notes

Không lưu vào `AiCallRecords`, nên lần gọi này không xem được ở Prompt Inspector (FR71). Chấp nhận vì câu dịch không đi vào văn bản; `prepare_proofread_call` là tiền lệ.

## Verification

**Commands:**
- `cargo test --test glossary_context_reference_contract` -- expected: xanh.
- `npx vitest run tests/frontend/glossaryContextReference.test.ts tests/frontend/glossaryQueue.test.ts` -- expected: xanh.
- `cargo test` và `npx vitest run` -- expected: xanh, vì đổi đăng ký lệnh ở `lib.rs`.
- `npm run test:e2e -- --spec e2e/specs/glossary-context-reference.e2e.mjs` -- expected: xanh.

**Manual checks (if no CLI):**
- Ice bấm `Dịch thử` với khoá thật trên một Tác phẩm tiếng Trung và một Tác phẩm tiếng Anh.
