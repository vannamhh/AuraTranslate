---
title: 'Tự dịch các khối đang thấy ở màn xem trước nhập URL'
type: 'feature'
ticket: '6'
created: '2026-11-11'
status: 'done'
baseline_revision: '01fe4f12f9ca91f47cc5222640a0421dcfd1650c'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md §Dịch nhanh']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ở tầng Giữ/Loại khối, người đã bật `Tự dịch` vẫn phải bấm `Dịch các khối đang thấy` sau mỗi lần cuộn; Bảng chờ đã tự dịch từ 15.5 còn màn xem trước thì chưa.

**Approach:** Webview lên lịch một lô `trigger: 'auto'` khi tập khối đang thấy đứng yên, dùng lại cổng Rust và lệnh lô đã có; lô chạy nối tiếp, khối lỗi/thiếu bị bỏ qua tới khi đóng màn.

## Boundaries & Constraints

**Always:** AD-64 mục 1, 7, 8, 9, 11. Cuộn không huỷ lô đang chạy; khối mới chờ lô đó xong rồi mới lên lịch. Khối đang nằm trong một lượt không được gửi lại. Khối lỗi hoặc thiếu kết quả (bất kể `trigger`) bỏ qua tới khi đóng màn; dọn kho không xoá dấu này (tiền lệ 15.5). Tự dịch bật thì không có nút `Dịch các khối đang thấy`. Thời gian chờ dùng chung `QUICK_AUTO_WAIT_MS` của 15.5. Rời Chương, đóng màn, tắt tuỳ chọn, đổi cấu hình nhanh thì xoá bộ đếm giờ và huỷ lượt tự chạy đang chạy. Trúng kho thì 0 lời gọi.

**Decisions (Ice, 2026-11-11):** (1) Đang tạm dừng vì lô thật thì nút `Dịch các khối đang thấy` hiện cạnh câu tạm dừng; tự dịch bật mà không tạm dừng thì ẩn. (2) Huỷ một lô tự chạy làm như Bảng chờ 15.5: khối bị huỷ về trống, không vào dấu bỏ qua; lần tập khối đang thấy đổi kế tiếp lên lịch lại và có thể gửi lại chúng nếu còn trên màn.

**Never:** Sửa Rust (cổng, `trigger`, `auto_off`, `paused` của lệnh lô đã có ở 15.5 kèm đối chứng (a) phần tầng khối). Dịch khối chưa thấy, khối ảnh, hoặc chạy khi lớp phủ đóng. Chạm `AiTranslateGeneration`, `J` `K` `Space`. Thêm phụ thuộc. Phím tắt mặc định.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Tự dịch | Bật, cấu hình riêng đủ, cuộn rồi dừng | Sau `QUICK_AUTO_WAIT_MS` một lô gửi các khối đang thấy mà kho chưa có | — |
| Cuộn khi đang chạy | Lô A chạy, khối mới vào màn | A không huỷ; xong A thì lô B gồm khối mới, không lặp khối của A | — |
| Quá trần | `deferred > 0` | Lô kế tiếp lấy phần còn lại nếu vẫn đang thấy | — |
| Lỗi/thiếu | Khối thiếu kết quả hoặc lỗi mạng | Khối hiện lỗi + `Dịch thử`; tự dịch không gửi lại tới khi đóng màn | Bấm tay vẫn thử lại |
| Lô thật chạy | `active_count > 0` hoặc Rust trả `paused` | `Lô dịch đang chạy nên tự dịch tạm dừng.`; lô thật xong thì lên lịch lại, không thăm dò | — |
| Tắt / cấu hình trống | Tuỳ chọn tắt hoặc Rust trả `auto_off` | 0 lời gọi tự chạy; nút `Dịch các khối đang thấy` như 15.3 | — |
| Bận | Bảng chờ đang chạy | Không gửi; tự lên lịch lại khi lượt kia xong | — |

</frozen-after-approval>

## Code Map

- Rust đã xong, chỉ đọc: `commands/aitranslate.rs` `quick_auto_gate` (:679), `quick_translate_blocks` (:2981, tham số `texts, trigger, retranslate, runId, channel`; trả `auto_off`/`paused` trước khi lấy vé). Ca IPC thật cả hai bề mặt: `tests/quick_translation_contract.rs:800-880`. Lượt mới cùng bề mặt thay lượt cũ ở Rust, nên webview không được gửi lô mới khi lô cũ còn chạy.
- `src/config/quickBatch.ts` -- `translateQuickBlocks` (:117) đang gắn cứng `trigger: 'manual'`; thêm tham số `trigger`.
- `src/importPreviewState.ts` -- khối dịch nhanh :851-1103: `visibleBlockKeys` (:906, chỉ khối chữ, theo `quickBlockKey`), `lookupVisibleBlocks` (:924), `settleQuickBatch` (:958, `auto_off`/`paused` hiện chỉ xoá `running`), `runQuickBatch` (:991, đã từ chối khi có lượt hoặc bận), `cancelImportPreviewQuickBatch` (:1069), watcher `chapterCursor`/`quickPurgeCount`, `resetImportPreviewQuickBatch` (:1094, gọi khi đóng màn ở :2728). Thêm bộ lên lịch, tập khoá đã lỗi, cờ tạm dừng.
- Khuôn lên lịch để chép: `src/glossaryQueueState.ts` `QUICK_AUTO_WAIT_MS` (:134), `clearAutoTimer` (:139), `scheduleAutomaticReference` (:212), các watcher :238-266 (`quickAiAutoEnabled`, `quickAiConfigRevision`, `aiTranslateActivity` `flush: 'sync'`), đánh dấu lỗi :318, lên lịch lại sau `paused` :319. Nguồn: `src/quickAiConfigState.ts` (`quickAiAutoEnabled`, `quickAiConfigRevision`), `src/config/aiTranslateActivity.ts` (`aiTranslateActivity`, `aiTranslateActivityError`: `null`/lỗi thì không tự chạy), `src/quickTranslateRun.ts` (`quickRunCurrent` để lên lịch lại khi bề mặt kia xong).
- `src/ImportPreviewOverlay.vue` -- khối tính trạng thái :998-1039 và đầu tầng :1800-1846; `IntersectionObserver` sẵn có :1041-1065 gọi `setImportPreviewVisibleBlocks` (đây là tín hiệu "cuộn"). Ẩn nút khi `quickAiAutoEnabled` trừ lúc tạm dừng; câu tạm dừng dùng chuỗi mới `mode.library.preview.tier2_quick_paused` (không mượn khoá `glossary.queue.context_paused`).
- `src/i18n/vi.json` :1195-1206.
- Test: `tests/frontend/importPreviewQuickBatch.test.ts`; e2e mẫu `e2e/specs/story-15-3-quick-batch-preview.e2e.mjs` (máy chủ trang + SSE đếm, `openPreview` :59) và `story-15-5-quick-auto-queue.e2e.mjs` (`enable` :21 qua `quick_ai_config_set_auto`, `quick_ai_config_save_fields` :57, dòng `AURA_AUTO_MEASUREMENT`).

## Tasks & Acceptance

Ba pha, mỗi pha một agent mới, giao qua `story-tu-dich-cac-khoi-ang-thay-o-man-xem-truoc-nhap-url-handoff.md` cạnh plan. Không có pha Rust.

**Execution (pha Webview):**
- [x] `src/config/quickBatch.ts`, `src/importPreviewState.ts` -- tham số `trigger`; bộ lên lịch theo ma trận; tập khoá đã lỗi sống tới `resetImportPreviewQuickBatch`; `paused` thành cờ tạm dừng; lên lịch lại khi lô xong, khi `active_count` về 0, khi bề mặt kia xong; xoá giờ + huỷ ở các lối ra trong Always.
- [x] `src/ImportPreviewOverlay.vue`, `src/i18n/vi.json` -- ẩn nút khi tự dịch bật, trừ lúc tạm dừng (Decision 1); dòng tạm dừng; `Esc` đầu vẫn huỷ lô tự chạy.
- [x] `tests/frontend/importPreviewQuickBatch.test.ts` -- mỗi hàng ma trận; cuộn trong lúc chạy không gửi lô chồng và không gửi lại khối của lô đang chạy; khối lỗi không tự gửi lại, mở lại màn thì được; tắt tuỳ chọn trong lúc chờ 0 lời gọi; `J` `K` `Space` không đổi. Mỗi seam gỡ thật một lần, đỏ đúng lý do.

**Execution (pha Tests that move):**
- [x] `e2e/specs/story-15-6-quick-auto-preview.e2e.mjs` (mới) -- trang đủ dài để cuộn nhiều màn; máy chủ ghi chỉ số khối của từng lời gọi và thời điểm bắt đầu/kết thúc; in một dòng số đo kèm build và quần thể.

**Acceptance Criteria:**
- Given tự dịch bật trên WKWebView với provider giả, when cuộn hết trang theo từng nấc rồi dừng, then mỗi khối chữ xuất hiện trong đúng một lời gọi và không hai lời gọi nào chồng thời gian.
- Given tuỳ chọn tắt, when mở màn xem trước và cuộn hết trang, then provider nhận 0 lời gọi.
- Given đối chứng của từng seam mới (khoá đã lỗi, chặn khi đang chạy, xoá giờ khi tắt), when gỡ seam trong mã sản phẩm, then đúng ca canh của nó đỏ vì đúng lý do.

## Implementation Notes

- Không đổi Rust: cổng `auto`, `auto_off`, `paused` của `quick_translate_blocks` và đối chứng (a) phần tầng khối đã có từ 15.5.
- `done` không còn khối hoãn mà có khối không kết quả thì khối đó vào dấu bỏ qua; `done` có khối hoãn chỉ lên lịch lại khi có ít nhất một khối xong. Không có hai chốt này thì một lô không trả gì lặp vô tận.
- Lượt của chính màn xem trước kết thúc không kích lên lịch qua `quickRunCurrent`, chỉ lượt của Bảng chờ, cũng để tránh vòng lặp.
- `paused` từ Rust chỉ dựng cờ tạm dừng khi webview còn thấy `active_count > 0`; sự kiện kết thúc tới trước phản hồi thì lên lịch lại ngay.
- Số đo e2e (debug `--features wdio`, Vite dev, WKWebView, provider SSE giả, trang 60 đoạn, màn đầu + 23 nấc 0,6 màn): 22 lời gọi, 60/60 khối gửi đúng một lần, 0 chồng, khoảng cách giữa hai lời gọi 804–825 ms; tắt: 0 lời gọi. Chi tiết ở [handoff](story-tu-dich-cac-khoi-ang-thay-o-man-xem-truoc-nhap-url-handoff.md).
- `check:panel-refs` đỏ 16 lỗi có sẵn ở baseline (15.4/15.5: `settingsQuickDataState`, `quickTranslateRun`, `bootstrap`), story này không thêm lỗi.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens `quick`): high 0 · medium 1 · low 3 · false 3 · maybe-false 0; một phát hiện low bị bác.
- medium · patch — e2e ca tắt mở lại trang `/a` mà kho đã có đủ 60 câu từ ca bật (một `$APPDATA` mỗi lượt), nên assert 0 lời gọi vẫn đúng khi tự dịch bật; đổi sang trang chữ mới và gỡ thử cho đỏ.
- low · patch — `paused` từ Rust không chặn lần cuộn sau: mỗi lần đổi tập khối lại gửi IPC `auto` (không ra mạng), và nếu sự kiện kết thúc tới trước phản hồi thì câu tạm dừng kẹt; đặt cờ chỉ khi webview còn thấy `active_count > 0`, không thì lên lịch lại. Chốt cờ trong `quickAutoMayRun` gỡ đi không ca nào đỏ (`active_count === 0` đã chặn) nên đã bỏ.
- low · reject — cùng kiểu hỏi lại cho `auto_off` khi webview tưởng bật mà Rust nói tắt: không ra mạng, chỉ xảy ra khi trạng thái cấu hình ở webview cũ, sửa phải thêm cờ.
- low · patch — `quickAutoFailed` không bỏ khoá khi bấm tay thành công, nên sau dọn kho khối đó không được tự dịch nữa; xoá khoá khi khối có kết quả.
- false — dòng `quickAutoTimer = null` "chết" trong `resetImportPreviewQuickBatch`: xoá thì `check:panel-refs` đỏ vì cổng đòi ô cấp module được dọn trực tiếp trong `reset*`; đã đặt lại.
- false — "đối chứng gỡ seam không có bằng chứng": handoff ghi ba phép gỡ thật và ca đỏ của từng phép (handoff không nằm trong diff được review).
- false — rerun e2e làm ca bật đỏ vì kho còn: `wdio.conf.mjs` cấp `$APPDATA` tạm mới mỗi lượt.

## Design Notes

Bộ lên lịch chỉ đọc `visibleBlockKeys` lúc giờ nổ, không lúc đặt giờ, nên khối đã cuộn khỏi màn không bị gửi. Lên lịch lại sau khi lô xong cũng đi qua cùng thời gian chờ; nếu đo thấy chậm thì báo ở handoff, không tự rút ngắn.

## Verification

**Commands:**
- `npx vitest run tests/frontend/importPreviewQuickBatch.test.ts tests/frontend/importPreviewBlocks.test.ts tests/frontend/glossaryQueue.test.ts` -- xanh
- `npm run check:i18n && npm run check:commands && npm run check:panel-refs && npm run check:tokens` -- xanh, `POINTER_DEBT` vẫn `[]`
- `npm run test:e2e -- --spec e2e/specs/story-15-6-quick-auto-preview.e2e.mjs` -- xanh, chạy ngoài sandbox; ghi số đo vào handoff
- Không cần chạy toàn bộ: không đổi đăng ký lệnh, `Cargo.toml` hay migration.
