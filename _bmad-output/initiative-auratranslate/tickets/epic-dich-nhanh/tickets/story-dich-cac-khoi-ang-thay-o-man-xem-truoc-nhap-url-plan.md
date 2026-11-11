---
title: 'Dịch các khối đang thấy ở màn xem trước nhập URL'
type: 'feature'
ticket: '3'
created: '2026-11-11'
status: 'done'
baseline_revision: '7d6e51867a2220b4b41d8a9185ae9d8ff3877a79'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-64', '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md §Dịch nhanh']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ở tầng Giữ/Loại khối của màn xem trước nhập URL, người dùng phải hiểu từng khối tiếng Trung/Anh mới quyết được giữ hay loại, mà lúc đó chưa có Tác phẩm nên `Dịch thử` của Bảng chờ không dùng được.

**Approach:** Lệnh lô mới dịch các khối đang thấy mà kho chưa có, bằng cấu hình nhanh, ngữ pháp khối AD-60 và kho `quick-translation.db`; lưu từng khối ngay khi đóng đúng. `QuickTranslateGeneration` biết bề mặt để trả `busy`. Webview hiện câu dịch dưới khối theo khuôn EXPERIENCE §Dịch nhanh.

## Boundaries & Constraints

**Always:** AD-64 mục 4–8, 11. Khoá kho là `quick_key` của chữ gốc khối, tính trước `guard_text`; một câu qua hai bề mặt ra một hàng. Chữ trang luôn qua `guard_text`; prompt lô do hàm thuần ở `rag.rs` lắp (AD-14). Một lượt nhận tối đa `QUICK_BATCH_MAX_BLOCKS` khối và `QUICK_BATCH_MAX_CHARS` ký tự, chốt bằng số đo ở pha Rust trên bảy trang `6-1-ban-do` sẵn có (Ice chọn 2026-11-11; trang tin, không phải Chương truyện, nên số đi kèm tên quần thể). Khối ảnh không dịch; khối đã loại vẫn dịch. Rust tra lại kho trước khi gửi, nên khối đã có không bao giờ bị gửi. Đóng màn xem trước huỷ lượt của nó. Mọi nút dịch ở tầng khối có `mousedown.prevent`.

**Never:** Dùng `run_translate_operation`, `plan_calls`, `AiTranslateGeneration`, `AiCallRecords`, `LastAssembledPromptState`. Ghi `target_text`, TM, Glossary, hay tạo Tác phẩm. Tự dịch, `auto_off`, trạng thái `Lô dịch đang chạy` (Story 15.5). Dịch khối ảnh, dịch trước khối chưa thấy. Đổi hành vi `J` `K` `Space`. Thêm phụ thuộc.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Lô đủ | 6 khối đang thấy, 2 đã có trong kho | 2 hiện ngay từ kho; bấm nút gửi 4, mỗi khối hiện và được lưu khi đóng | — |
| Trùng chữ | 2 khối cùng `quick_key` | Gửi một lần, một hàng, cả hai khối hiện câu dịch | — |
| Quá trần | Khối chưa có vượt trần | Lấy theo thứ tự trang từ trên xuống tới trần; nút còn đó cho phần còn lại. Một khối một mình quá `MAX_CHARS` đi một lượt riêng | — |
| Huỷ giữa lô | Khối 1–2 đã đóng, `Huỷ` hoặc `Esc` | Khối 1–2 còn trong kho và trên màn; khối còn lại về nút | `Esc` sau mới đóng màn |
| Thiếu khối | Mô hình bỏ hoặc không đóng khối 3 | Khối 3 `Thiếu kết quả ở khối này.` + `Dịch thử`; không lưu | Không tự gửi lại |
| Bị cắt | `finish_reason` = `length` | Khối đã đóng được lưu; khối dở thành thiếu | — |
| Bận | Bảng chờ đang chạy `Dịch thử` | Lệnh lô trả `busy`, 0 lời gọi; nút tắt với `Đang dịch ở màn khác.`, tự bật khi lượt kia xong. Chiều ngược lại cũng vậy | — |
| Cấu hình / kho | Chưa cấu hình; kho không mở được | `Chưa cấu hình AI…` + `Mở Cài đặt AI`; `Không mở được kho dịch nhanh.` | Không gọi mạng |
| Lỗi mạng | Provider lỗi giữa lô | Khối đã đóng giữ; khối còn lại lỗi + `Dịch thử` | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/ai/batch.rs` -- `guard_text` (:170), `BlockStream` (:363, `push` :386 trả `TextPiece`, khối đóng chỉ ra ở `finish` :411), `reconcile` (:575), `StreamEnd` (:566). Cần thêm một cách lấy khối vừa `Closed` trong lúc đọc (vd. `take_closed`) để lưu khi đọc được; `finish`/`reconcile` giữ nguyên ngữ nghĩa cho lô thật. `TRANSLATE_CALL_MAX_SEGMENTS = 100` (:7) là trần duy nhất hiện có.
- `src-tauri/src/core/ai/rag.rs` -- khuôn `assemble_call_prompt` (:819, khối `<<<SEGMENT n` + `guard_text` :864-878) và `assemble_context_reference_prompt` (:1006). Thêm hàm thuần lắp prompt lô khối, nhận chữ gốc, tự `guard_text`, kèm `system_with_block_format` (batch.rs:44).
- `src-tauri/src/commands/aitranslate.rs` -- `QuickTranslateGeneration` (:774, `begin` :784, `cancel` :798): thêm bề mặt (`glossary_queue` | `import_preview`) để khác bề mặt thì `busy`, cùng bề mặt thì thay. `ai_context_reference_translate` (:2311) truyền bề mặt Bảng chờ; `QuickTranslateOutcomeWire` (:943) thêm `busy`. Khuôn gửi-và-lưu `quick_translate<P>` (:1002), `resolve_quick_call_config` (:831), `send_with_cancel` (:1592). Khuôn kênh JSON có tag: `AiTranslateBatchEventWire` (:1337). Lệnh lô mới nhận danh sách chữ khối, `run_id`, và `Channel` sự kiện (token theo chỉ số, khối đã lưu, kết thúc có `missing` và `usage`); thân là hàm generic nhận `&P: TranslationProvider` để test chạy `Fake`. Thêm lệnh tra nhiều chữ một lượt cho khối vừa hiện.
- `src-tauri/src/core/quick_translation/mod.rs` -- `quick_key` (:17), `lookup` (:50), `save(.., still_current)` (:74). Không có tra lô; thêm nếu lệnh tra nhiều cần.
- `src-tauri/src/lib.rs` -- `invoke_handler` (:1122), lệnh nhanh ở :1450-1452, `QuickTranslateGeneration` manage ở :1764.
- `src-tauri/tests/ai_boundary.rs:521` -- `ALLOWED_AI_BATCH_NAMES_IN_COMMAND_SEAM`; `ParsedBlock`/`guard_text` chưa có. Tên mới cần ở commands thì thêm kèm lý do vào chính danh sách này, không né bằng đường khác.
- Ghim lệnh: `tests/ipc_contract.rs:3066`, `tests/ipc_argument_contract.rs:561` (tham số camelCase phải khớp một `invoke()`), `tests/config_invariants.rs:2320` (số `#[tauri::command]` của `aitranslate.rs` = 12).
- Khuôn test: `tests/quick_translation_contract.rs` (`Fake` :55-111, ca wire :580-662), bắt sự kiện kênh JSON ở `tests/ai_batch_wire.rs:377-388`.
- Số đo trần: `_bmad-output/implementation-artifacts/6-1-ban-do/` (`fixtures/html/a01-a07.html`, `extraction-raw.tsv`, trang tin `epochtimes` tiếng Trung) và probe `src-tauri/tests/webimport_probe.rs`. Khối đang thấy tối đa phụ thuộc chiều cao cửa sổ; đo ở app thật.
- Khối: `core/webimport/extractor.rs:113` `BlockBody`; wire `commands/project/mod.rs:1305/1344`, không có id, danh tính là vị trí trong Chương đang xem.
- `src/ImportPreviewOverlay.vue` (3037 dòng, tầng :1647-1806) -- đầu tầng `.ip-tier2-head` :1663; khối `li.ip-block` :1737, chữ `p.ip-block-text` :1783; nút khối `@mousedown.prevent` :1750-1780; `onTier2Keydown` :737 bỏ qua phím khi đích là nút; `onEscapeCancel` :1003/:595 đóng màn ngay; scroller là `section.mode` của `LibraryMode.vue:942` qua `findScroller()` :910; khối con trỏ cuộn vào ở :883. Không có IntersectionObserver hay ảo hoá.
- `src/importPreviewState.ts` (2540 dòng) -- `importPreviewSelectedBlocks` :813, `blockFocusedIndex` :459, `resetImportPreview`, `cancelImportPreview` :2419. Đổi Chương thì khối khác: câu dịch gắn theo chữ khối, không theo chỉ số.
- `src/glossaryQueueState.ts` -- `lastRunId` :123 (mục nợ `check:panel-refs` đỏ, `Chủ: Epic 15`), `activeRunId` :125, `runContextReference` :195. Chuyển bộ cấp `run_id` và lượt đang chạy (bề mặt, `run_id`) sang một module dùng chung, miễn trừ có tên theo tiền lệ `src/aiTranslateRunClock.ts::stamp` (`scripts/check-panel-refs.mjs:130`).
- `src/config/glossaryContextReference.ts` -- khuôn adapter IPC không ném, `Channel` :104. `src/aiUsageLine.ts`, `src/quickAiConfigState.ts` (`quickAiConfigReadiness`).
- `src/GlossaryQueueOverlay.vue` -- tham chiếu :481-560, tiêu điểm `Huỷ` :146-163, `onEscape` :165. Bảng chờ mở chồng lên màn xem trước qua `StatusBar.vue:694`.
- `src/i18n/vi.json` -- `mode.library.preview.tier2_*` :1177-1200, `glossary.queue.context_*` :1959-1970. `src/commands/index.ts` khối :1916-1985, `glossary.context.*` :4563; `scripts/check-commands.mjs` `HANDLER_TABLE` (`onEscapeCancel` :2518), `POINTER_DEBT` = `[]`.
- Test: `tests/frontend/importPreviewBlocks.test.ts` (`freshOverlay` :91), `glossaryContextReference.test.ts`, `glossaryQueue.test.ts`. e2e: `e2e/specs/import-preview-mouse.e2e.mjs` (máy chủ trang :99, mở màn :105-121), `story-15-1-quick-translation-cache.e2e.mjs` (SSE giả có đếm, `ollama` tầng Global).

## Tasks & Acceptance

Bốn pha theo AGENTS.md, mỗi pha một agent mới; giao lại qua `story-dich-cac-khoi-ang-thay-o-man-xem-truoc-nhap-url-handoff.md` cạnh plan.

**Execution (pha Rust):**
- [x] Đo -- số khối đoạn văn/chú thích mỗi trang và phân bố ký tự mỗi khối trên `6-1-ban-do/fixtures/html/a01-a07.html`; chốt hai hằng, ghi số và quần thể vào handoff, một dòng vào Implementation Notes -- AD-64 §Chưa chốt.
- [x] `src-tauri/src/core/ai/{batch.rs,rag.rs}` -- đường lấy khối vừa đóng; hàm thuần lắp prompt lô khối -- mục 7.
- [x] `src-tauri/src/commands/aitranslate.rs`, `src-tauri/src/lib.rs` -- bề mặt + `busy` trong `QuickTranslateGeneration`; lệnh lô; lệnh tra nhiều; đăng ký -- mục 5, 7, 8.
- [x] `src-tauri/tests/quick_batch_contract.rs` (mới), `quick_translation_contract.rs`, `ai_boundary.rs`, `ipc_contract.rs`, `config_invariants.rs` -- ma trận I/O với `Fake`; đối chứng (c) một câu qua `Dịch thử` rồi lô (và ngược lại) ra một hàng, 0 lời gọi; (d) phần lô: huỷ sau khối 2 giữ khối 1–2, khối dở không lưu, không có hàng `library_work`/tệp `project.db` mới; (f) phần `busy`: bề mặt thứ hai nhận `busy` và 0 lời gọi, huỷ sai `run_id` không dừng lô. Mỗi đối chứng gỡ seam thật, đỏ đúng lý do.

**Execution (pha Webview):**
- [x] `src/quickTranslateRun.ts` (mới), `src/glossaryQueueState.ts`, `scripts/check-panel-refs.mjs` -- bộ cấp `run_id` và lượt đang chạy dùng chung, phản ứng được; Bảng chờ dùng nó, hiện `busy`; đóng mục nợ `lastRunId` trong `deferred-work.md`.
- [x] `src/config/quickBatch.ts` (mới), `src/importPreviewState.ts` -- adapter không ném; khối đang thấy, tra kho khi hiện, lô, huỷ, thiếu, lỗi, `usage`; huỷ khi đóng màn, đổi Chương.
- [x] `src/ImportPreviewOverlay.vue`, `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json`, `scripts/check-commands.mjs` -- dòng đầu tầng và nút; câu dịch + dòng nguồn `role="status"` `<mô hình> · Dịch lại`; bù cuộn cho khối con trỏ; `Esc` đầu huỷ lượt; tiêu điểm `Huỷ` không rơi xuống `body`.
- [x] `tests/frontend/importPreviewQuickBatch.test.ts` (mới), `glossaryQueue.test.ts` -- từng trạng thái của ma trận, `busy` hai chiều, `Esc` hai bước, `Space` sau cú bấm chuột không gửi lượt nữa, `J` `K` không đổi.

**Execution (pha Tests that move):**
- [x] `e2e/specs/story-15-3-quick-batch-preview.e2e.mjs` (mới) -- một máy chủ `node:http` phục vụ trang nhiều khối và `chat/completions` SSE theo dấu khối, đếm lời gọi, chảy chậm có điểm dừng để huỷ giữa chừng.

**Acceptance Criteria:**
- Given một trang nhiều khối trên WKWebView với provider giả, when bấm `Dịch các khối đang thấy`, huỷ sau khi một phần khối đã đóng, rồi đóng và mở lại màn xem trước, then các khối đã đóng hiện câu dịch với 0 lời gọi mới và không có Tác phẩm nào được tạo.
- Given khối có con trỏ đứng yên, when câu dịch mọc ở khối phía trên, then vị trí trên màn của khối đó lệch không quá 1 px.
- Given mỗi đối chứng (c)(d)(f) phần của story này, when gỡ seam trong mã sản phẩm, then đúng test canh của nó đỏ vì đúng lý do.

## Implementation Notes

- Hai hằng chốt trên bảy trang tin `6-1-ban-do` (epochtimes, tiếng Trung): `QUICK_BATCH_MAX_BLOCKS = 40`, `QUICK_BATCH_MAX_CHARS = 4000`; 40 khối chữ liền nhau tối đa 3 827 ký tự, một khối tối đa 246. Số liệu đầy đủ ở handoff.
- `Dịch lại` của khối dùng chính lệnh lô với cờ `retranslate`, không mượn lệnh của Bảng chờ, để `busy` vẫn tính theo bề mặt xem trước. Dịch lại bị huỷ hoặc thiếu kết quả thì câu cũ hiện lại, vì kho vẫn giữ nó.
- Lệnh mới không vào `POINTER_DEBT_BASELINE`: danh sách đó chỉ ghi các id đỏ lúc cổng ra đời.
- Hai tệp `.ts` mới làm sàn `FRONTEND_FLOOR` (`naming_boundary.rs`) và `FILE_FLOOR` (`check-layout.mjs`) trôi dưới 80% số thật; nâng lên 207 theo chính chỉ dẫn của cổng.
- Lượt chạy đủ: build xanh, vitest 4 278/4 278 ca xanh nhưng thoát 1 vì `EnvironmentTeardownError` ở `editorConfirmSegment.test.ts` (mục nợ có sẵn, `Chủ: Ice`); `cargo test` chỉ đỏ `naming_boundary` (sàn trên, đã sửa). `check:clippy` đỏ sẵn ở `convention_exchange_contract.rs:115` (mục nợ có sẵn).

## Plan Change Log

## Review Triage Log

Lượt 1 (lens `quick`): high 1 · medium 4 · low 6 · false 1 · maybe-false 0; ngoài lens, đọc diff thêm 3 vá (comment số đo, chuỗi `v-if`, hàm reset chết) và lượt chạy đủ thêm 1 (`FRONTEND_FLOOR`).
- high · patch — nút `Dịch thử`/`Dịch lại` của khối không có con trỏ vẫn nhận Tab nên Enter gửi khối con trỏ; nút khối khác nay `tabindex=-1`, ca vitest đỏ khi gỡ.
- medium · patch — `not_configured` từ Rust khoá nút tới khi đóng màn; nút chỉ khoá theo readiness, lý do vẫn hiện; ca đỏ khi gỡ.
- medium · patch — `Đang dịch N khối` đếm cả khối quá trần; đếm theo `started`; ca đỏ khi gỡ.
- medium · patch — lỗi mạng đánh `failed` cả khối chưa gửi; chỉ khối trong `started`; cùng ca trên.
- medium · patch — e2e đo lệch ≤ 1 px sau bước nhảy đầu; agent pha Tests đo lại từ trước cú bấm.
- low · patch — xong lượt tiêu điểm về danh sách, không về nút mới (EXPERIENCE §Phím); ca đỏ khi gỡ.
- low · patch — Bảng chờ không nói `Đang dịch ở màn khác.` cạnh `Dịch lại` bị tắt; thêm dòng đó.
- low · patch — `Dịch lại` ở Bảng chờ bị Rust trả `busy` xoá câu đang hiện; trả lại `fallback`.
- low · patch — `Received.saved` là `true` khi kho bỏ qua; nay là giá trị `save` trả.
- low · patch — `ai_boundary` thêm tên không kèm lý do; thêm lý do, bỏ số đếm đã sai từ trước.
- low · patch — assert "không có `project.db`" của (d) không thể đỏ vì lệnh không nhận handle nào khác; xoá, e2e `library_list_works` là ca canh thật.
- low · reject — thông báo `busy` còn lại sau khi lượt kia xong chỉ xảy ra khi bấm trong vài ms sau lúc huỷ ở lớp phủ kia; nút vẫn bấm được.
- false — nửa `Space` của ca vitest không đỏ được, nhưng ca vẫn đỏ khi gỡ `mousedown.prevent` nhờ assert `defaultPrevented` (đã gỡ thật ở pha Webview).
- reject — ô `[ ]` của pha Tests và lượt chạy đủ là sổ sách plan; lượt chạy đủ đã chạy (xem Verification).

## Design Notes

Lưu khi đóng, không đợi `reconcile`: lô thật chỉ biết khối nhận được sau `finish`, nên nếu dùng nguyên thì huỷ giữa lô mất trắng các khối đã trả tiền (AD-64 Prevents 7). Khối lưu ở lần đóng đầu; một lần đóng trùng chỉ số sau đó không ghi đè. `reconcile` vẫn quyết khối nào thiếu khi lượt kết thúc.

Bề mặt nằm ở Rust, không chỉ ở webview, vì hai cửa sổ webview hỏng nhau vẫn không được chạy hai lượt hay huỷ chéo. Module `run_id` dùng chung ở webview chỉ để hiện `Đang dịch ở màn khác.` trước khi bấm và tự bật lại.

## Verification

**Commands:**
- `cargo test --test quick_batch_contract --test quick_translation_contract --test ai_boundary` -- xanh
- `npx vitest run tests/frontend/importPreviewQuickBatch.test.ts tests/frontend/glossaryQueue.test.ts tests/frontend/importPreviewBlocks.test.ts` -- xanh
- `npm run check:panel-refs && npm run check:commands && npm run check:i18n && npm run check:tokens` -- xanh, `POINTER_DEBT` vẫn `[]`
- Chạy toàn bộ một lần ở pha cuối vì đổi đăng ký lệnh trong `lib.rs`.

