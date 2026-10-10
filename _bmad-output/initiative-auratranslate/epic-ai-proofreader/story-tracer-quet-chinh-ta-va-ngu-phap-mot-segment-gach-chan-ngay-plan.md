---
title: 'Tracer: quét chính tả và ngữ pháp một segment, gạch chân ngay trong ô'
type: 'feature'
ticket: '1'
created: '2026-10-10'
status: 'built'
baseline_revision: 'd31e95b67d9fce242c2d515fce3da0998dcf1308'
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

**Problem:** Chưa có đường nào đưa bản dịch của người dùng qua AI để bắt lỗi chính tả/ngữ pháp và vẽ kết quả ngay dưới chữ trong ô (FR80, FR85, FR86); hình dạng phát hiện trên dây chưa có nên 9.2–9.6 không có gì để xây lên.

**Approach:** Lát mỏng xuyên lớp: lệnh `ai.proofread.run` (⌘P theo mockup) flush segment có con trỏ → command Rust mới trong file seam mới `commands/proofread.rs` → hàm lắp prompt thuần trong `core/ai/rag.rs` nhận bản dịch → `TranslationProvider` → token qua Channel, kết quả cuối là danh sách phát hiện Rust đã định vị → webview vẽ gạch chân lượn sóng `error` bằng CSS Custom Highlight API. Không ghi `project.db`.

**Decision (Ice 2026-10-10):** văn bản prompt quét cố định trong `core/ai/rag.rs` (đánh dấu `aura-allow-text`), hàm thuần nhận bản dịch; không thêm biến prompt-set. Kế hoạch giữ trọn (3.108 token), không tách e2e.

## Boundaries & Constraints

**Always:** AD-13 (seam mới vào allow-list `ai_boundary.rs` và compile probe), AD-14 (prompt lắp bằng hàm thuần), AD-15 (gọi qua `TranslationProvider`), AD-21 (lỗi `IpcError`), AD-22 (Channel, huỷ được, không tự thử lại). Phát hiện có đủ bốn trường FR83: `kind` (`spelling`|`grammar`) · vị trí (offset UTF-16 trong `target_text` đã quét) · `explanation` · `suggestion`. Rust định vị cụm chữ (AD-1), webview chỉ vẽ. Chưa cấu hình AI ⇒ `not_configured` trước mọi lượt mạng, mời cấu hình như dịch đơn (FR77). Hiện số token và ước tính chi phí khi xong.

**Never:** ghi `project.db` hay đổi trạng thái xác nhận; lưu phát hiện ra đĩa; dải gợi ý, chấp nhận, bỏ qua, ghi nhớ (9.4, 9.6, AD-53); quét Chương hoặc vùng chọn (9.3); đối chiếu bản gốc (9.2); thêm phụ thuộc mới; đặt cụm chữ bằng offset do model trả về.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Có lỗi | segment có con trỏ, AI đã cấu hình, model trả hai phát hiện | `done{usage, findings[2], unlocated: 0}`; hai cụm gạch chân `error` | — |
| Chưa cấu hình | endpoint/model/khoá trống | `not_configured`, không gọi mạng; dải mời cấu hình | — |
| Cụm không có trong văn bản | model trích cụm không xuất hiện | phát hiện bị bỏ, `unlocated` đếm nó (không im lặng) | — |
| Cụm lặp | cùng cụm xuất hiện hai lần | lần xuất hiện đầu chưa bị phát hiện trước chiếm | — |
| Phản hồi hỏng | không phải JSON đúng lược đồ | không vẽ gì | `IpcError` khoá mới, `retryable: true` |
| Huỷ | người dùng huỷ giữa chừng | `cancelled`, không vẽ | — |
| Gõ sửa sau quét | người dùng gõ trong ô đã gạch chân | xoá gạch chân của segment đó | — |
| Chữ trong ô khác chữ đã quét khi kết quả về | flush xong rồi gõ tiếp trong lúc chờ | không vẽ | — |
| Không có Tác phẩm / segment ngoài Chương mở | | | `work.none_open` / `ai_prompt.segment_not_in_chapter` |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/aitranslate.rs` -- tái dùng: `resolve_call_config` :231, `read_api_key` :275, `state_missing` :265 (nâng `pub(crate)`; proofread.rs không gọi keychain trực tiếp nên `aiconfig_keychain_boundary.rs` không đổi), mẫu `AiTranslateGeneration` :495 (`next`/`is_current`), `send_prepared_translate_call` :691 (`spawn_blocking`+`block_on`), `AiTranslateUsageWire` :545, `wire::ai_translate_segment` :835, `wire::ai_translate_cancel` :1022. Không đổi hành vi dịch.
- `src-tauri/src/core/ai/rag.rs` -- `assemble_prompt` :614 chỉ nhận câu nguồn; thêm hàm thuần lắp prompt quét nhận bản dịch, văn bản prompt mang `// aura-allow-text: …` như :254. Bộ phân tích phản hồi + định vị đặt trong `core/ai/` (module mới hoặc rag.rs).
- `src-tauri/src/ports/translation_provider.rs:138` `translate(req, on_token, should_cancel)`; `core/ai/client.rs:323` `OpenAiChatClient`; `core/ai/pricing.rs:64`.
- `src-tauri/src/commands/segment/chapter_read.rs:369` `read_open_chapter_segments` → `target_text`; `aiprompt.rs:74` `segment_not_in_chapter`.
- `src-tauri/src/lib.rs` -- `generate_handler!` :1024-1324 (AI :1276-1293); state ở `open_work_slot` :1552ff (`AiTranslateGeneration` :1578).
- `src-tauri/src/core/i18n/mod.rs` `message_keys!` -- khoá lỗi mới.
- Ghim phải cập nhật: `tests/ai_boundary.rs` (hằng seam theo file chính xác, allow-list tên, vòng chính :649-660, compile probe :1773ff, marker dòng `lib.rs` :1551), `tests/config_invariants.rs:1734` `COMMAND_FILE_CENSUS` (18 → 19), `tests/ipc_contract.rs:2145` (thêm ca đăng ký wire proofread). Mẫu test: `ai_translate_contract.rs` (`FakeProvider` :979, `collecting_channel` :1045, keychain mock :162, `read_text_and_origin` :1675), `ai_translate_wire.rs` (`Harness` :103).
- Webview: `src/config/aitranslate.ts` (mẫu adapter Channel :119-139, cancel :229; `isIpcError`/`hasIpcBridge`/`UNKNOWN_IPC_ERROR` đang private), `src/aiTranslateState.ts` (union trạng thái :42, `sequence` :66, stale theo segment :85-91, reset :178), `src/aiTranslateCancel.ts`, `src/aiTranslateHandlers.ts:30-36` (`editorCaretSegmentId`), `src/panels/editorPanelState.ts:800` `flushEditorBeforeDiscreteWrite`.
- `src/commands/index.ts` -- `CommandSpec` (registry.ts:45-80); mẫu `ai.translate.run` :4124-4144; `index.ts` không import giá trị vue (src/AGENTS.md :25); handler gắn ở `main.ts:955`. `Mod+P` chưa ai dùng.
- `src/panels/GridPanel.vue` -- ô `.cell-tgt` :1900-1917 (một text node lúc nạp, nhiều text node `\n` sau Enter, không có phần tử con); `onEditInput` :1548; `restoreEditedText` :1068-1081 ghi lại `textContent`; TreeWalker offset mẫu `editorSegments.ts:295-321`.
- `src/panels/inlineStripPriority.ts` -- khe `proofreader` đã giữ chỗ; dùng cho dải trạng thái quét. Mẫu dòng usage `AiTranslationPanel.vue:243-253`, khoá `ai.translate.usage_*`.
- Token `--color-error` (`src/tokens/tokens.json:34,60`); `check:tokens` buộc `text-decoration-color` là token. Mockup `ux-auratranslate/mockups/proofreader.html:50-52`: wavy, 1px, offset 4px.
- e2e: WebdriverIO trong WKWebView thật (`e2e/AGENTS.md`), dùng để đo Highlight API.

## Tasks & Acceptance

**Execution:** ba pha, mỗi pha một agent mới (AGENTS.md §Policy), bàn giao qua `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/e16ca69f-11f9-471c-b3df-67a80c724067/scratchpad/9-1-handoff.md` (pha sau đọc nó, ghi thêm mục của mình): **Rust** = bốn task đầu; **Webview** = năm task kế (tới vitest); **Đo & đối chứng** = e2e, đối chứng, bộ đầy đủ một lần.
- [x] `src-tauri/src/core/ai/` -- hàm thuần lắp prompt quét; hàm thuần phân tích phản hồi JSON và định vị từng cụm trích (`quote`) trong `target_text`, trả `findings` + `unlocated`. Unit test các hàng ma trận định vị/phản hồi hỏng.
- [x] `src-tauri/src/commands/proofread.rs` (mới) -- lớp thuần nhận `Option<&Store>`/`Option<&OpenWork>` + provider; `wire::ai_proofread_segment(segment_id, channel)` và `wire::ai_proofread_cancel`; state `ProofreadGeneration` riêng để huỷ dịch không huỷ quét. Đăng ký ở `lib.rs`, khoá lỗi ở i18n.
- [x] `src-tauri/tests/ai_boundary.rs`, `config_invariants.rs`, `ipc_contract.rs` -- thêm seam mới, đếm file, ca đăng ký.
- [x] `src-tauri/tests/ai_proofread_contract.rs` (mới) -- FakeProvider: done/không cấu hình/huỷ/phản hồi hỏng/segment ngoài Chương; ca chụp `target_text`, `translation_origin` và trạng thái xác nhận của MỌI segment trong Chương trước và sau một lượt quét có phát hiện, bằng nhau.
- [x] `src/config/proofread.ts`, `src/proofreadState.ts` (mới) -- adapter không ném; trạng thái theo mẫu dịch đơn (sequence, stale, reset có `check:panel-refs`), flush trước khi gọi.
- [x] `src/commands/index.ts`, `src/main.ts`, i18n -- `ai.proofread.run` (`Mod+P`), `ai.proofread.cancel`.
- [x] `src/panels/GridPanel.vue` (+ helper thuần offset → Range) -- vẽ bằng `CSS.highlights` + `::highlight()` wavy token `error`, offset 4px; xoá theo segment khi gõ, khi `restoreEditedText`/thay segment/đổi Chương/reset; chỉ vẽ khi `textContent` bằng văn bản đã quét.
- [x] Dải trạng thái ở khe `proofreader`: đang quét + Huỷ, xong (số phát hiện, `unlocated` nếu > 0, token, chi phí), mời cấu hình, lỗi qua `tError`.
- [x] vitest: adapter, state (stale, huỷ, reset), helper offset → Range qua nhiều text node, wiring lệnh.
- [x] `e2e/` -- spec đo: `CSS.highlights` có trong WKWebView, vẽ gạch chân dưới đúng cụm, chụp màn hình. Không có Highlight API hoặc wavy/offset không ăn trong `::highlight` ⇒ DỪNG, trình Ice hai phương án (lớp phủ / cách khác) kèm số đo.
- [x] Đối chứng: gỡ thật lệnh vẽ, phép định vị, và phép kiểm "chữ khác thì không vẽ" lần lượt; ca canh tương ứng đỏ đúng lý do.

**Acceptance Criteria:**
- Given một segment có lỗi chính tả cố ý và AI đã cấu hình, when gọi `ai.proofread.run`, then gạch chân lượn sóng màu `error` hiện dưới đúng cụm chữ trong ô (ảnh chụp e2e hoặc Ice xác nhận trong app thật).
- Given lượt quét có phát hiện, when so `project.db` trước và sau, then không ký tự `target_text` nào và không trạng thái xác nhận nào đổi.
- Given cây nguồn, when chạy `ai_boundary`, then chỉ seam `commands/proofread.rs` chạm module quét và lời gọi đi qua `TranslationProvider`.

## Implementation Notes

- Đo WKWebView (e2e, dpr 2): `CSS.highlights` có; trong `::highlight()` chỉ `text-decoration-line/-style/-color` dạng tách có hiệu lực (shorthand không vẽ gì; `text-underline-offset`, `-position`, `text-decoration-thickness` bị bỏ qua), nên sóng chạm chấm dưới `ọ`. Ice chọn giữ Highlight API, không lớp phủ (2026-10-10); khoảng hở thành mục nợ `Chủ: Story 9.5`, đường ⌘P trong app thật thành mục nợ `Chủ: Epic 9`.
- Hình dạng trên dây: `ai_proofread_segment {segmentId, channel}` → `not_configured` · `cancelled` · `done{usage, scanned_text, findings[{kind,start,end,explanation,suggestion}], unlocated}`, offset UTF-16 vào `scanned_text`; bản dịch rỗng trả `done` rỗng, không gọi mạng.
- Seam thứ tư của `ai_boundary.rs` khoá đúng năm đường `core::ai::*` mà `commands/proofread.rs` được gọi; khoá API vẫn chỉ đọc trong `aitranslate.rs` (`read_api_key` nâng `pub(crate)`).
- Dải trạng thái dùng khe `proofreader` có sẵn; flush trượt/còn dơ thành lỗi có tên (`proofread.flush_failed`, `proofread.flush_still_dirty`), không im lặng.
- Bốn sàn cổng nâng vì tệp mới đẩy số thật vượt 80% sàn cũ: `check-commands` TS 89→97, `check-panel-refs` 89→97, `check-tokens` 125→137, `check-doc-refs` 480→512 (Ice duyệt).
- Đối chứng: gỡ `CSS.highlights.set`, `rangeForOffsets`, phép so chữ đã quét, phép tìm trong `locate_findings`, nhánh mời cấu hình của dải ⇒ mỗi ca canh đỏ đúng lý do; ca vẽ ban đầu chưa có người canh nên thêm hai ca `paintProofreadRanges`.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 4 medium · 2 low · 3 false · 1 low loại · 1 loại (sửa plan).
- medium · patch — đổi Chương trong Tác phẩm không gọi `resetProofread()` (`editorPanelState.ts:1885` chỉ reset hai state dịch): lượt quét đang chạy về thành `done` của Chương cũ.
- medium · patch — gõ trong ô đang quét: `clearProofreadFor` chỉ xoá khi `done`, kết quả về không vẽ (đúng) nhưng dải nói "đã gạch chân trong ô".
- medium · patch — `wire::ai_proofread_segment` gọi `next()` sau `prepare` (đọc Chương + keychain): huỷ đến trong khe đó bị lượt quét ghi đè. Cùng thứ tự ở `ai_translate_segment` có từ trước ⇒ defer.
- medium · patch — flush `failed`/`still-dirty` đưa về `idle` chỉ kèm console.warn: ⌘P nháy "Đang quét" rồi không còn gì (im lặng rỗng); các nơi gọi khác báo người dùng.
- low · patch — thông điệp assert `config_invariants.rs` còn "khai 78/56" sau khi đổi thành (80, 56).
- low · patch — comment đầu `ProofreaderStrip.vue` kể lại mã; các doc comment khác mang hợp đồng hoặc bất biến nên giữ.
- low · loại — `TEXT>>>` trong bản dịch phá khung prompt: văn bản của chính người dùng, hiếm, sửa phải thêm thoát chuỗi.
- false — một phát hiện `kind` lạ làm hỏng cả phản hồi: đúng hàng "Phản hồi hỏng" của ma trận đã khoá.
- false — dải proofreader che dải TM mờ trên cùng segment: đúng thứ tự UX-DR21 (`inlineStripPriority.ts`), việc TM nhường là nợ `Chủ: Story 9.4`.
- false — `browser.pause` trong e2e: chỉ chạy nightly, chưa thấy đỏ thất thường; không có hại cụ thể.
- loại — ô task còn `[ ]` khi `in-review`: cách sửa là sửa plan.

## Design Notes

Model trả `[{kind, quote, explanation, suggestion}]`, không trả offset: LLM đếm ký tự kém, nên Rust tìm `quote` trong `target_text` và tính offset UTF-16 cho Range của JS. Cụm không tìm thấy được đếm vào `unlocated` và hiện ra, vì đó là đúng lớp lỗi "im lặng rỗng". Quét đọc `target_text` từ DB sau flush (`flushEditorBeforeDiscreteWrite`) để chỉ có một nguồn sự thật. Outcome trả kèm văn bản đã quét để webview so với ô trước khi vẽ. Highlight API giữ Range sống ngoài DOM của ô, nên Vue không phải render lại. Phát hiện cũ đi thế nào là việc của AD-53; tracer chỉ xoá khi gõ.

## Verification

**Commands:**
- `cargo test --test ai_proofread_contract`, `--test ai_boundary`, `--test config_invariants`, `--test ipc_contract` -- xanh (sau `npm run build`); đây là wiring dùng chung (`lib.rs`) nên chạy cả bộ đầy đủ một lần trước review.
- `npx vitest run tests/frontend/proofread*.test.ts` -- xanh; `npm run check:tokens check:i18n check:commands check:layout check:panel-refs` -- xanh.
- `npm run test:e2e -- --spec <spec proofread>` -- đo Highlight API, ảnh chụp.

**Manual checks (if no CLI):**
- Trong app thật: gõ lỗi, ⌘P, thấy gạch chân lượn sóng; không đạt tự động thì thành mục nợ `Chủ: Epic 9`.
