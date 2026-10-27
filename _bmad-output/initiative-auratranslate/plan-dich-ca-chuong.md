---
title: 'Dịch cả Chương: một nút gửi mọi câu chưa dịch của Chương đang mở qua đường dịch lô'
type: 'feature'
ticket: ''
created: '2026-10-26'
status: 'done'
baseline_revision: 'a20e17a5fb489f816ecc46808b4e15fa0267ebdf'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Muốn AI dịch nốt một Chương thì hiện phải bấm ⌘⇧↓ qua từng câu để dựng vùng chọn, mà vùng chọn đó còn gồm cả những câu Ice đã dịch tay.

**Approach:** Thêm nút "Dịch cả Chương (N chưa dịch)" vào hàng nút thứ hai của panel Đề xuất AI. Nút gửi mọi câu chưa dịch và không bị cắt bỏ của Chương đang mở qua đường dịch lô có sẵn. Rust là nơi quyết câu nào được gửi.

**Đã chốt (Ice qua Sally, 2026-10-26; mục nợ cùng ngày trong `deferred-work.md`):**
- Chỉ gửi câu có `target_text` rỗng và không bị cắt bỏ (`is_omitted`), theo thứ tự tài liệu.
- N trên nút đúng bằng số câu sẽ gửi. Khoá nút khi N = 0 hoặc khi chưa mở Tác phẩm.
- Danh sách câu do Rust quyết (AD-1).
- Dùng lại đường dịch lô (`ai_translate_batch`): Huỷ, Thử lại, "Đưa N câu sang bản dịch" và hộp xác nhận chống mất bản nháp chạy như với vùng chọn.
- Không đổi "Dịch câu đã chọn", ⌘⇧↓ hay mô hình vùng chọn.
- Khoá `OpenWorkState` lúc lắp prompt cho cả lô giữ nguyên như lô theo vùng chọn, không đo thêm và không ghi nợ (Ice, 2026-10-26). Số đo đã có: lô 100 câu, TM 100.000 cặp mỗi tầng, giữ khoá p50 81,9 s (Epic 7 retro R-4).
- Giữ một plan dù vượt 1600 token (1342 từ; tiktoken không có trên máy).

## Boundaries & Constraints

**Always:**
- Rust lọc lại theo dữ liệu trong `project.db` ở lượt gửi. Một câu webview tưởng chưa dịch mà Rust thấy đã có bản dịch thì Rust bỏ qua, hàng đó hiện "Đã bỏ qua", không gọi provider.
- Số N là ảnh của luật Rust tính phía webview, theo khuôn `proofreadScope.ts::scannableIds` ("Mirrors what Rust drops"). Câu đang gõ dở thì tính theo chữ trong bộ đệm gõ, vì bộ đệm gõ là ngoại lệ duy nhất của AD-1.
- "Rỗng" nghĩa là rỗng sau khi `trim`, giống nhau ở cả hai phía (cùng luật `proofread.rs:164`).
- `@click` là đúng một `dispatch`. Lệnh mới có id dạng chấm và có nhãn `command.<id>`.

**Never:**
- Không thêm một đường dịch lô thứ hai, không đổi hình dạng sự kiện Channel.
- Không đổi luật chọn của "Dịch câu đã chọn". Lượt gọi theo vùng chọn vẫn gửi cả câu đã có bản dịch.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Chương dịch dở | 10 câu: 6 đã dịch, 1 cắt bỏ, 3 rỗng | Nút "Dịch cả Chương (3 chưa dịch)"; bấm thì đúng 3 lượt gọi provider, theo thứ tự tài liệu | — |
| Đã dịch hết | Mọi câu có bản dịch hoặc bị cắt | Nút khoá, N = 0 | — |
| Chưa mở Tác phẩm | `editorSegments` rỗng | Nút khoá | — |
| Đang gõ dở | Câu 4 rỗng trong DB, bộ đệm gõ có chữ | N không tính câu 4 | — |
| Lệch giữa hai phía | Webview gửi câu 5, DB đã có bản dịch của câu 5 | Hàng câu 5 "Đã bỏ qua", 0 lượt gọi cho câu 5 | — |
| Chỉ có khoảng trắng | `target_text = "  "` | Tính là chưa dịch ở cả hai phía | — |
| Một lượt khác đang chạy | Lượt đơn hoặc lô đang `generating` | Nút khoá | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/aitranslate.rs:492` `prepare_batch_call` -- đọc Chương một lần, sắp `segment_ids` theo thứ tự tài liệu, `is_omitted` ⇒ `PreparedBatchItem::Omitted` ⇒ sự kiện `Skipped` (`:953`). Thêm chế độ "bỏ câu đã dịch" ở đây: câu có bản dịch rơi vào cùng nhánh bỏ qua, không lắp prompt.
- `src-tauri/src/commands/aitranslate.rs:1329` `wire::ai_translate_batch(app, segment_ids, prompt_set_name, channel)` -- thêm một tham số cho chế độ trên. Không có lệnh mới nên `lib.rs` không đổi. Mọi lượt gọi trong `src-tauri/tests/ai_translate_wire.rs` (314, 390, 428, 493, 548, 680) phải thêm đối số.
- `src-tauri/src/commands/proofread.rs:164` -- luật lọc mẫu (`!is_omitted && !target_text.trim().is_empty()`).
- `src/config/aitranslate.ts:101` -- adapter `ai_translate_batch`. Đối số `invoke` là object literal (`ipc_argument_contract`, xem 3b5eca7).
- `src/aiTranslateBatchState.ts:246` `runAiTranslateBatch(promptSetName, segmentIds)` và `retryAiTranslateBatch` -- truyền chế độ xuống. Lượt Thử lại giữ chế độ của lô đang chạy.
- `src/aiTranslateHandlers.ts:102` -- mẫu handler `runAiTranslateBatch`. Handler mới gửi danh sách ảnh.
- `src/proofreadScope.ts:9` -- khuôn ảnh phía webview. Bộ đệm gõ nằm trong `src/panels/editorPanelState.ts` (snapshot `:861`).
- `src/commands/index.ts:5179` -- đăng ký `ai.translate.batch_run`. Thêm `ai.translate.chapter_run` cùng khuôn, `keys: undefined`.
- `src/panels/AiTranslationPanel.vue` -- `.ai-batch-bar` (do a20e17a dựng). Đặt nút trước "Dịch câu đã chọn". Điều kiện khoá giống `canRunAiTranslateBatch`.
- `src/i18n/vi.json` -- `command.ai.translate.chapter_run` và nhãn có số.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/commands/aitranslate.rs` -- chế độ bỏ câu đã dịch trong `prepare_batch_call` và tham số ở `ai_translate_batch` -- Rust quyết câu nào được gửi
- [ ] `src-tauri/tests/ai_translate_wire.rs` -- thêm đối số cho các lượt gọi cũ; thêm ca cho hàng "Chương dịch dở", "Lệch giữa hai phía" và "Chỉ có khoảng trắng" -- đếm lượt gọi provider thật
- [ ] `src/config/aitranslate.ts`, `src/aiTranslateBatchState.ts`, `src/aiTranslateHandlers.ts`, `src/commands/index.ts`, `src/main.ts` -- lệnh `ai.translate.chapter_run` và ảnh ids -- nối dây
- [ ] `src/panels/AiTranslationPanel.vue`, `src/i18n/vi.json` -- nút có số N -- giao diện
- [ ] `tests/frontend/` -- các hàng webview của ma trận (N, khoá, đang gõ dở, Thử lại giữ chế độ) -- canh ảnh và nút

**Acceptance Criteria:**
- Given một lô cả Chương đang chạy, when Ice bấm Huỷ, then lô dừng như lô theo vùng chọn và các hàng đã xong vẫn đưa sang bản dịch được.
- Given lô cả Chương xong, when Ice bấm "Đưa N câu sang bản dịch", then chỉ những câu vẫn còn rỗng được ghi; câu đã có bản nháp phải qua hộp xác nhận.

## Verification

**Commands:**
- `cargo test --test ai_translate_wire` (sau `npm run build`) -- expected: qua
- `npm run test:story -- --list`, rồi chạy các tệp vitest của lô và panel -- expected: qua, dòng `Errors` trống
- `npm run check:commands && npm run check:i18n && npm run check:lint && npm run check:clippy` -- expected: xanh

**Manual checks (if no CLI):**
- Ice dịch cả một Chương thật đang dở.

## Implementation Notes

- Rust keeps `prepare_batch_call` unchanged and adds `prepare_batch_call_with(..., skip_translated)`, so the existing contract tests did not need a new argument. Translated rows reuse `PreparedBatchItem::Omitted`, hence the `Skipped` event and no keychain read for them.
- Only `wire::ai_translate_batch` takes the new `skip_translated: bool`; no new command, so command registration is unchanged. `tests/ipc_contract.rs` pins its parameter list and was updated.
- `lib.rs` has a second caller of `ai_translate_batch` inside `mod nfr_bench` (feature `nfr-bench`, not covered by `check:clippy`); it passes `false`.
- 🔵 `cargo clippy --features nfr-bench --all-targets -- -D warnings` has 3 `collapsible_if` errors at `lib.rs:912/931/944` that predate this plan and were left alone.
- `src/main.ts` needed no change: the new handler lives in `aiTranslateHandlers.ts`, which `main.ts` already spreads.
- Webview N comes from `src/chapterTranslateScope.ts`, reading `editorEditedShape` (typing buffer) before `textShapeOf(target_text)`; shape 0 or 1 counts as untranslated.
- `runAiTranslateBatchCall` takes `skipTranslated` as a trailing optional parameter; four existing assertions in `aiTranslateBatch.test.ts` now expect `false`. `batchSkipsTranslated` is reset in `resetAiTranslateBatch()` (`check:panel-refs`).
- Retry keeps the mode of the running batch through that module-level flag.
- Counter-check: forcing the translated-row clause to `false` made `a_skip_translated_batch_...` red with `provider_unreachable` on segment 5 (a 4th provider call); restored, 16/16 green.
- Manual check still open: Ice translating a real half-finished Chapter.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 1 · medium 0 · low 2 · false 6 · maybe-false 0.
- high · patch — `src-tauri/tests/ipc_contract.rs:2577` ghim danh sách tham số cũ của `ai_translate_batch`; thêm `skip_translated` làm `cargo test --test ipc_contract` đỏ.
- low · patch — test Rust của lô cả Chương chỉ đếm `done`/`skipped` và thứ tự, không kiểm id nào bị bỏ; thêm assert theo id.
- low · reject — handler `runAiTranslateChapter` chưa có test khi lượt đơn đang chạy; nút đã khoá (có test) và lệnh không có phím mặc định, nên hiếm ai chạm tới.
- false — `batchSkipsTranslated` sau `resetAiTranslateBatch`: reset xoá cả `rows`, nên Thử lại không còn hàng nào để gửi.
- false — `editorSegments` chỉ nạp khi lưới dựng: lưới có mặt ở mọi tầng bố cục (`workspaceLayout.ts`, tầng `narrow` "chỉ còn lưới").
- false — nút mới dùng chung class `.ai-translate-batch-run`: không có hành vi sai; selector test dùng `data-*` riêng.
- false — dòng `invoke` dài 112 ký tự: `check:lint` xanh.
- false — AC Huỷ / Đưa sang bản dịch không có test riêng: lô cả Chương đi qua cùng `aiTranslateBatchState`, các test lô hiện có canh hai đường đó.
- false — fixture khoảng trắng phụ thuộc `save_segment_targets` có trim hay không: hàm không trim (`segment/targets.rs:94`).
- reject — ghi chú kế hoạch (`main.ts` trong task, Implementation Notes trống): sửa plan, không phải sửa mã; agent điền Implementation Notes.
