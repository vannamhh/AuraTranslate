---
title: 'Mở đầu: vá C1, C2, C4, C6 của Epic 4'
type: 'bugfix'
ticket: '9'
created: '2026-10-10'
status: 'built'
baseline_revision: 'a61ad96c0fbba6d829d679b0d8618f2cfb5739af'
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

**Problem:** Bốn lỗi của retro Epic 4 còn sống trên đường dịch AI: thử lại lô xoá mọi hàng đã `done` (C1); "đưa sang Editor" ghi kết quả dịch đơn cũ vào câu cũ thay vì kết quả lô ở câu có con trỏ (C2); lỗi lô không mang `segment_id` để hàng `running` kẹt mãi (C4); `promote_ai_translation` ghi được vào segment ngoài Chương đang mở (C6). Ice giữ Epic 4 `rejected` tới khi C1, C2 được vá.

**Approach:** Mỗi lỗi một ca đỏ viết trước, rồi bản vá nhỏ nhất tại chỗ gây lỗi; không rút hàm chung (việc của 9.12), không đổi hình dạng dây Rust ↔ webview.

**Decisions (Ice, 2026-10-10):**
- C2: giữ cả kết quả đơn lẫn kết quả lô; promote chọn kết quả của lượt BẮT ĐẦU sau cùng (một lần thử lại lô tính là một lượt lô mới). Kết quả mới hơn không dùng được (lô mới hơn nhưng con trỏ không ở hàng lô có text) thì lùi về kết quả kia. Không reset kết quả đơn khi lô bắt đầu.
- C4: vá ở webview — lỗi lô không có `segment_id` dùng được thì hàng đang `running` thành `error`, giữ text đã nhận; Rust không đổi.
- C6: khoá lỗi mới `segment.not_in_open_chapter` (params `segment_id`, `chapter_id`), không dùng lại `ai_prompt.segment_not_in_chapter`.
- Plan giữ nguyên độ dài (~2.050 token), không tách.

## Boundaries & Constraints

**Always:** ca đỏ trước bản vá và đỏ đúng lý do; IPC thử lại vẫn chỉ gửi id lỗi/chờ (`[12, 13]`), không gửi lại hàng `done`/`skipped`/`cancelled`; giữ thứ tự kiểm của promote (thiếu → retired → [mới] ngoài Chương → needs_confirmation/force) và lượt ghi một UPDATE của AD-47 ①; lỗi Rust chỉ qua `IpcError::new` với `message_key` trong `message_keys!`.

**Never:** rút hàm chung đơn/lô; sửa C3, C5, C7, C8; đổi luật 4.8 "kết quả đơn đáp vào segment nó bắt đầu" (test `tests/frontend/aiTranslate.test.ts:654`); thêm `?.` hay nới assert để xoá đỏ; comment mang id story, ngày, lịch sử review.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| C1 thử lại | Lô 11-13: 11 `done` "Ket qua 1", 12 lỗi retryable, 13 chờ; bấm thử lại | IPC nhận `[12, 13]`; danh sách còn `[11, 12, 13]`, 11 vẫn `done` với text cũ; con trỏ ở 11 + promote → ghi "Ket qua 1" vào 11 | 12, 13 về `pending` với text rỗng |
| C2 lô mới hơn | Dịch đơn 11 xong (T); rồi lô 12-13 xong; con trỏ 13; promote | Ghi text lô của 13 vào 13; T vẫn còn | — |
| C2 đơn mới hơn | Lô 12-13 xong; rồi dịch đơn 11 xong (T); con trỏ 13; promote | Ghi T vào 11 | — |
| C2 lùi về | Dịch đơn 11 xong (T); rồi lô 12-13 xong; con trỏ 14 (không có hàng lô); promote | Ghi T vào 11 | — |
| C2 không đổi | Dịch đơn 11 xong, con trỏ dời sang 14, không có lô; promote | Vẫn ghi T vào 11 (luật 4.8) | — |
| C4 lỗi không id | Lô đang chạy câu 12; kết quả lỗi `ai_translate.internal_failure` params rỗng | Hàng 12 thành `error` giữ text đã nhận, `aiTranslateBatchRunningSegmentId` = null | Không retry (non-retryable) |
| C6 ngoài Chương | Segment của Chương A; đang mở Chương B; promote, `force` false hoặc true | `Err` mã `segment.not_in_open_chapter`; `target_text` và `translation_origin` không đổi | Webview hiện lỗi qua `tError()` sẵn có |

</frozen-after-approval>

## Code Map

- `src/aiTranslateBatchState.ts:204-279` -- `runAiTranslateBatch` dựng lại `rows`/`rowIndexBySegmentId` (218-220: gốc C1); 255-262 tra `params.segment_id`, NaN thì bỏ qua (gốc C4); 182-184 `aiTranslateBatchRetryIds`.
- `src/aiTranslateHandlers.ts:40-60` -- `promoteAiTranslate`: kết quả đơn trước (41-47), lô theo con trỏ sau (gốc C2); 95-111 handler thử lại gọi `runAiTranslateBatch(name, ids)` (gốc C1); handler `runAiTranslateBatch` cùng tệp.
- `src/aiTranslateState.ts:47,109,174` -- `aiTranslateRunSegmentId`, `resetAiTranslate` (chỉ gọi ở `modes/libraryChapters.ts:316`, `modes/libraryImport.ts:436`, `panels/editorPanelState.ts:1885`).
- `src-tauri/src/commands/segment/targets.rs:276-365` -- `promote_ai_translation`, SELECT ở 293-294 không đọc `chapter_id` (gốc C6); caller khác `segment/tm_match.rs:292` `accept_tm_fuzzy` thừa hưởng phép kiểm, chấp nhận được.
- `src-tauri/src/commands/aitranslate.rs:284-287` -- phép kiểm mẫu của `prepare_translate_call` qua `read_open_chapter_segments` + `segment_not_in_chapter` (`commands/aiprompt.rs:74`, code `ai_prompt.segment_not_in_chapter`). `OpenWork.chapter_id` ở `commands/project/mod.rs`.
- `src-tauri/src/commands/aitranslate.rs:174-185,714,995` -- `batch_panicked_error()` params rỗng; release có `panic = "abort"` (`Cargo.toml:215`) nên nhánh này chủ yếu ở dev, nhưng `UNKNOWN_IPC_ERROR` của adapter (`src/config/aitranslate.ts:202,209`) đi cùng đường ở release.
- `src-tauri/src/core/i18n/mod.rs:~274` · `src/i18n/vi.json` · `src-tauri/src/commands/segment/confirm.rs` (`segment_retired`) -- chỗ thêm khoá `segment.not_in_open_chapter`; `check:i18n` canh cặp Rust ↔ vi.json.
- `tests/frontend/aiTranslateBatch.test.ts` -- ca C1 sửa assert ở 624 (đang khẳng định lỗi) + thêm promote; ca C2 ở describe 940 (`pendingSegmentRun()` :142, `pendingBatchRun()`, mẫu dispatch 857-870); ca C4 cạnh describe "lỗi giữa lô" :533. Dùng lại `freshPanel`, `selectAllThreeFixtureSegments`, `PROVIDER_UNREACHABLE_ON_12`, `promoteMock`.
- `src-tauri/tests/ai_translate_contract.rs:1598` -- mẫu ca retired; dùng lại `temp_dir`, `open_work` (:106), `first_segment_id` (:115), `read_text_and_origin` (:1634), `cleanup`; Chương thứ hai theo `insert_chapter_directly` (`tests/project_contract.rs:1465`) + `commands::chapter::open_chapter`.

## Tasks & Acceptance

**Execution:**
- [ ] `tests/frontend/aiTranslateBatch.test.ts` -- viết ca đỏ C1, C2, C4 theo ma trận, chạy thấy đỏ đúng lý do -- bằng chứng lỗi có thật trước khi vá.
- [ ] `src-tauri/tests/ai_translate_contract.rs` -- ca đỏ C6 cho `force` false và true -- như trên.
- [ ] `src/aiTranslateBatchState.ts` -- lối vào thử lại giữ `rows`, chỉ đặt lại hàng được thử (status `pending`, text rỗng, usage null), dùng chung thân Channel với `runAiTranslateBatch`; lỗi không có `segment_id` thì hàng `running` thành `error` -- C1, C4.
- [ ] `src/aiTranslateHandlers.ts` -- handler thử lại gọi lối vào mới; promote chọn kết quả của lượt bắt đầu sau cùng, lùi về kết quả kia khi không dùng được -- C1, C2.
- [ ] `src-tauri/src/commands/segment/targets.rs` + `core/i18n/mod.rs` + `src/i18n/vi.json` + helper cạnh `segment_retired` -- đọc `chapter_id`, từ chối trước mọi lượt ghi khi khác Chương đang mở -- C6.
- [ ] Đối chứng: gỡ thật từng bản vá, chạy lại đúng target của nó, thấy đỏ vì đúng lỗi đó; trả bản vá lại.

**Acceptance Criteria:**
- Given bốn ca đỏ đã viết, when áp bốn bản vá, then cả bốn xanh và mỗi ca đỏ lại khi gỡ đúng bản vá của nó.
- Given các test dịch đơn, dịch lô, huỷ, thử lại hiện có, when chạy `tests/frontend/aiTranslate*.test.ts` và target `ai_translate_contract`, then vẫn xanh (trừ assert 624 được sửa có chủ đích).
- Given Epic 4 retro, when story xong, then phán quyết C1, C2 được ghi để Ice xét ký lại Epic 4.

## Implementation Notes

- C1 và C2 đã vá (ca đỏ rồi xanh, gỡ bản vá thì đỏ lại): Ice xét ký lại Epic 4. C4, C6 cũng vá.
- C1: `retryAiTranslateBatch` giữ `rows`, chỉ đặt lại hàng được thử; dùng chung `streamBatch` với `runAiTranslateBatch`.
- C2: `src/aiTranslateRunClock.ts` cấp stamp đơn điệu cho lượt đơn và lô; promote chọn stamp lớn hơn, lùi về kết quả kia khi không dùng được. Thêm miễn trừ `check:panel-refs` có lý do.
- C4: lỗi lô không có `segment_id` dùng được thì hàng `running` thành `error`.
- C6: `segment.not_in_open_chapter` kiểm trước mọi lượt ghi trong `promote_ai_translation`.
- Đối chứng C4, C6 (lượt review 2): gỡ nhánh `else` của lỗi không id ⇒ đúng ca C4 đỏ (`expected 'running' to be 'error'`, 1 failed | 28 passed); gỡ phép kiểm `OtherChapter` ⇒ đúng ca C6 đỏ ở `.expect` đòi `Err` (68 passed; 1 failed); trả lại ⇒ `ai_translate_contract` 69, `segment_wire` 4, `tm_contract` 144 (1 ignored), `config_invariants` 33 xanh, vitest ba tệp 56/56, `check:i18n` đạt.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens `quick`): high 0 · medium 0 · low 1 · false 5 · maybe-false 0.
- low · patch — comment mới trái luật AGENTS.md (doc tiếng Việt ở `retryAiTranslateBatch` và `segment_not_in_open_chapter`, doc lặp tên ở hai stamp và `nextAiTranslateRunStamp`); sửa: một dòng tiếng Anh cho hợp đồng thử lại, xoá phần còn lại.
- false · rejected — "checkbox chưa tick, không có bằng chứng đối chứng": sửa nó là sửa plan (bị loại); đối chứng C2 chạy lại ở phía điều phối: gỡ điều kiện `batchIsNewer` ⇒ đúng ca "C2 lô mới hơn" đỏ (1 failed | 28 passed), trả lại ⇒ 29/29.
- false · rejected — ca C6 đặt `open.chapter_id` bằng tay: phép kiểm chỉ đọc `open.chapter_id`, nên ca chạm đúng seam; nhánh promote thành công trong Chương đang mở đã có các ca promote sẵn trong 69 ca `ai_translate_contract` xanh.
- false · rejected — thử lại lô đua với lượt dịch đơn: handler `retryAiTranslateBatch` chặn lượt đơn đang `generating` từ trước (`src/aiTranslateHandlers.ts:100`); lượt bị vượt mặt do `sequence` chặn, reset dọn cả `rows`.
- false · rejected — hai stamp reset độc lập: reset đơn xoá cả text nên `singleUsable` false; reset lô xoá `rows` nên `batchUsable` false; không cặp nào chọn sai kết quả.
- false · rejected — `accept_tm_fuzzy` thừa hưởng phép kiểm và `message_keys!` là wiring chung: dải TM chỉ nhắm segment của Chương đang mở; `check:i18n` đạt và target `tm_contract`, `config_invariants` chạy lại (kết quả ghi ở Implementation Notes).

Lượt 2 (lens `quick`): high 0 · medium 1 · low 1 · false 3 · maybe-false 0.
- medium · đối chứng ở phía điều phối — AC1 thiếu bằng chứng đỏ-khi-gỡ cho C4, C6 và dòng cuối lượt 1 trỏ kết quả `tm_contract`/`config_invariants` không có trong Notes; đã chạy, ghi ở Implementation Notes. Phần "checkbox chưa tick" bị loại vì sửa nó là sửa plan.
- false · rejected — thử lại lặng lẽ bỏ một phần id lạ: handler lấy id từ `aiTranslateBatchRetryIds(rows)`, mọi id đó đều có trong `rowIndexBySegmentId`.
- false · rejected — doc của `setRowAt` cũ: map vẫn chỉ được dựng ở đầu `runAiTranslateBatch`, lối thử lại dùng lại không dựng lại; dòng đó không nằm trong diff.
- false · rejected — hàng `cancelled` không được thử lại, tổng usage gồm hàng xong từ lượt trước: §Always loại `cancelled` khỏi IPC thử lại; `aiTranslateBatchUsageSummary` là tổng của cả lô (FR76), chi phí hàng 11 đã thật sự trả.
- low · rejected — ca C6 không phủ thứ tự retired → ngoài Chương → needs_confirmation: vòng `force` true chứng minh phép kiểm đứng trước lượt ghi và trước `force`; đảo với retired chỉ đổi mã lỗi; hiếm gặp, sửa là thêm ca mới. Phần đặt `open.chapter_id` bằng tay: carried từ lượt 1.

## Verification

**Commands:**
- `npx vitest run tests/frontend/aiTranslateBatch.test.ts tests/frontend/aiTranslate.test.ts tests/frontend/aiTranslationPromoteErrorAlert.test.ts` -- expected: xanh.
- `npm run build && cargo test --manifest-path src-tauri/Cargo.toml --test ai_translate_contract --test segment_wire` -- expected: xanh.
- `npm run check:i18n` -- expected: đạt.

