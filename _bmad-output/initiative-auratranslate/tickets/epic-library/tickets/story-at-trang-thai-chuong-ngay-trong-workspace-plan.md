---
title: 'Đặt trạng thái Chương ngay trong Workspace'
type: 'feature'
ticket: '18'
created: '2026-10-27'
status: 'built'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
baseline_revision: '888423553f6e85a26ae73357f3fba377d6cf1b0d'
route: 'full'
route_source: 'auto'
risk: 'medium'
review_loop_iteration: 1
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Gõ, nhận bản AI hay xác nhận câu đều không đổi `chapter.status`, nên Chương dịch trọn vẫn ở Chưa bắt đầu; Workspace không có đường đặt Đã xong, nút duy nhất nằm ở trang Tác phẩm Library và đổi Chương đang mở ở Workspace (PRD FR5 🔵 2026-10-27).

**Approach:** Rust tự chuyển Chưa bắt đầu → Đang dịch trong cùng giao dịch với lượt ghi bản dịch đầu tiên do người dùng, và một migration sửa dữ liệu cũ; Workspace có nút Đã xong dùng lệnh `lifecycle.set_chapter_done` sẵn có, và gợi ý không chặn khi xác nhận câu cuối; gỡ nút ở Library.

## Boundaries & Constraints

**Always:**
- Chuyển tiếp tự động duy nhất: `not_started → in_progress`, khi Chương có ít nhất một câu `is_omitted = 0`, chưa về hưu, bản dịch khác rỗng sau trim. `paused`/`done`/`in_progress` không bao giờ bị đổi tự động.
- Lượt ghi tính: flush gõ, nhận bản AI (gồm dịch lô, đi qua `promote_ai_translation`), nhận TM khớp mờ và khớp tuyệt đối do người bấm, nhận thay đổi reviewer (FR94), khôi phục phiên bản (FR101), nhận/hoàn sửa của AI Proofreader, xác nhận câu (Ice 2026-10-28, vòng review 1). Không tính: điền sẵn TM lúc mở Chương, gộp/tách câu, nhập.
- Chuyển tiếp đi cùng những gì `set_chapter_status` làm: bump `chapter.updated_at`, viết lại `meta.json`, rồi `Indexer::rebuild` — chỉ khi đã thực sự chuyển, không mỗi lượt flush.
- Đã xong chỉ do người bấm; gợi ý chỉ mở đường tới lệnh.
- Quyết định Q1 (Ice 2026-10-27): vỏ của các đường ghi giữ nguyên kiểu (flush vẫn đồng bộ, không đổi khoá); kết quả mang cờ `chapter_started`, và khi cờ bật webview gọi một lệnh async mới theo khuôn `set_chapter_status` (viết lại meta, reindex) nhưng không UPDATE trạng thái. App sập giữa hai lượt thì `project.db` vẫn đúng, Library theo kịp ở lượt ghi trạng thái kế tiếp.
- Quyết định độ dài (Ice 2026-10-27): giữ một plan, chia pha Rust · Webview · Tests, mỗi pha một agent mới.

**Never:** tự đặt `done`; đổi thứ tự hay khoá của flush (AD-35); quét Library trên luồng chính; hiện trạng thái Chương mới trong Workspace ngoài trạng thái của nút; sửa Chế độ đọc (entry 19).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error |
|---|---|---|---|
| Lượt gõ đầu | Chương `not_started`, flush 1 câu "Xin chào" | `in_progress`, meta và chỉ mục theo | — |
| Lượt gõ sau | Chương đã `in_progress` | không UPDATE chapter, không reindex | — |
| Gõ rồi xoá | flush câu thành "" hoặc chỉ khoảng trắng | giữ `not_started` | — |
| Tạm ngưng / Đã xong | flush vào Chương `paused`/`done` | giữ nguyên | — |
| Điền sẵn TM | mở Chương `not_started` có TM khớp 100% | giữ `not_started` | — |
| Migration | Chương `not_started` có câu đã dịch; Chương chỉ có câu `is_omitted` mang chữ; Chương rỗng | chỉ Chương đầu sang `in_progress` | backup trước migrate như mọi bước |
| Câu cuối | xác nhận câu chưa xác nhận cuối cùng (bỏ qua câu cắt bỏ) | gợi ý "Đánh dấu Chương Đã xong?" hiện | — |
| Còn câu | xác nhận một câu khi còn câu chưa xác nhận | không gợi ý | — |
| Đặt lỗi | `set_chapter_done` trả lỗi trong Workspace | lỗi hiện ở Workspace, nút bật lại | `tError` |

</frozen-after-approval>

## Code Map

Rust (`src-tauri/`):
- `src/commands/segment/targets.rs` -- `save_segment_targets` :94 (giao dịch :129-171; chèn chuyển tiếp trước `Ok` :170), `write_non_user_target` :253, `promote_inner` :323. `src/commands/segment/confirm.rs:515` `flush_segment_targets` là nơi quyết định của flush — vỏ `wire.rs:144` không quyết gì.
- `src/commands/segment/tm_match.rs` `accept_tm_fuzzy` :316, `accept_tm_exact` :345 · `review_accept.rs:21` · `history.rs:292` restore · `proofread_accept.rs:143` -- các đường tính.
- `src/commands/segment/chapter_read.rs:586` `fill_exact_tm_matches` · `regroup.rs` · `core/segment/import.rs:150` -- KHÔNG gọi chuyển tiếp.
- `src/commands/lifecycle.rs` -- `write_lifecycle_after_change` :113 (meta), `finish_lifecycle_write` :340, `set_chapter_status_indexed` :357 (khuôn hàm thuần có reindex), vỏ async :435 và `finish_with_reindex` :516. Trạng thái: `core/lifecycle/mod.rs` `LifecycleStatus::{NotStarted,InProgress}.as_str()`.
- `src/core/store/schema.rs:2262` `PROJECT_MIGRATIONS`, bước cuối `to_version: 41` :2482; mẫu UPDATE dữ liệu `SEGMENT_TRANSLATION_ORIGIN_DDL` :2019. Literal trong `concat!` không tham chiếu const.
- Test ghim phiên bản phải sửa: `tests/pinned_contract.rs:262-274`; `tests/segment_contract.rs:2200-2244` (bước giả 42 → 43, thêm `PROJECT_MIGRATIONS[40]`); `review_harvest_contract.rs:701`; `prompt_set_contract.rs:887,895`; `tm_contract.rs:778,817,823,1220,1234,1249,1269`; `proofread_decision_contract.rs:285`; `chapter_origin_contract.rs`, `segment_role_contract.rs`, `glossary_unconfirm_contract.rs`.
- Ca mới ngoài `lifecycle_contract.rs`: `tm_contract.rs` (chọn khớp đúng, nhận khớp gần, điền sẵn giữ `not_started`), `alignment_contract.rs` (nhận đổi của Reviewer), `segment_contract.rs` (khôi phục), `proofread_decision_contract.rs` (nhận và hoàn tác).
- `tests/lifecycle_contract.rs` -- helper `open_work` :46, `add_chapter` :53, `read_meta_from_disk` :72, `open_indexer` :485, `status_in_index` :499, `chapter_statuses` :722; mẫu reindex :513.
- `tests/config_invariants.rs:2280` (census vỏ của `segment/wire.rs`), `:1449` `blocking_wire_cases` -- đổi khi thêm/đổi vỏ async.

Webview (`src/`):
- `panels/editorPanelState.ts` -- `confirmCurrentSegment` :1400, nhánh `result === 'confirmed'` :1449 (segments đã cập nhật); `datThongBao` :2698; `resetEditorPanel` :993. `'confirmed-last'` :2548 là hàng cuối theo vị trí, không phải "cả Chương".
- `config/segment.ts` -- `isSegmentConfirmed` :166, `is_omitted` :111.
- `StatusBar.vue` -- nhánh Workspace :557-700, nút `ui-btn ui-btn--sm` + `dispatch` :664-691; tra Chương đang mở trong `libraryChapters` :164-167.
- `modes/WorkspaceMode.vue:120-133` -- dải thông báo có nút (mẫu cho gợi ý).
- `modes/libraryWorks.ts:353` `setOpenChapterStatus` (lỗi vào `openWorkError`, chỉ Library hiện); `modes/libraryChapters.ts` `loadChapters` -- phải gọi lại sau khi đặt, không thì `ChapterRow.status` cũ.
- `commands/index.ts:2813` `lifecycle.set_chapter_done`; `tokens/controls.css` hệ nút ba bậc.
- `modes/LibraryWorkPage.vue:336-343` nút gỡ; `i18n/vi.json:1122` khoá gỡ (không có cổng khoá-thừa).
- Vitest khuôn dây phải thêm `chapter_started: false`: `tests/frontend/aiBatchPanelRows.test.ts`, `aiTranslatePromoteConfirmFlow.test.ts`, `proofreadDecisionRatio.test.ts`, `tmFuzzyAdapters.test.ts`.
- Cổng: `check:commands` (`@click` đúng `dispatch('<id>')`), `check:i18n`, `check:tokens`, `check:panel-refs` (ref cấp module phải có `reset*()`).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/lifecycle.rs` -- hàm thuần `mark_chapter_started(tx, chapter_id) -> bool`: một câu `UPDATE chapter SET status='in_progress', updated_at=… WHERE id=? AND status='not_started' AND EXISTS(câu không cắt bỏ, chưa về hưu, bản dịch khác rỗng)`; vị từ "khác rỗng" khớp `str::trim` (25 code point `White_Space`, như `GLOSSARY_ENTRY_DDL`) -- một chỗ khai quy tắc.
- [x] `src-tauri/src/commands/segment/{targets,confirm,tm_match,review_accept,history,proofread_accept}.rs` + `wire.rs` -- gọi `mark_chapter_started` trong giao dịch ghi của từng đường tính; trả cờ `chapter_started` trong kết quả IPC; thêm lệnh async `refresh_after_chapter_started` (khuôn vỏ `set_chapter_status` + `finish_with_reindex`, không UPDATE) đăng ký trong `lib.rs`, kèm hàng census ở `tests/config_invariants.rs`.
- [x] `src-tauri/src/core/store/schema.rs` -- bước `to_version: 42` UPDATE dữ liệu như migration ở matrix; đường mở Tác phẩm viết lại `meta.json` và reindex khi bước 42 vừa chạy làm đổi hàng.
- [x] `src-tauri/tests/*` -- sửa các test ghim phiên bản; `lifecycle_contract.rs` thêm ca cho mọi hàng matrix phía Rust, kèm một ca cho mỗi đường tính và ca điền sẵn TM.
- [x] `src/StatusBar.vue` + `src/i18n/vi.json` -- nút "Đặt Chương là Đã xong" (`ui-btn ui-btn--sm`) ở nhánh Workspace, `dispatch('lifecycle.set_chapter_done')`, khoá khi Chương đã `done` hoặc đang bận; nhãn khoá ghi Chương đã xong.
- [x] `src/modes/libraryWorks.ts` + các cổng IPC của sáu đường ghi trong `src/config/` -- sau khi đặt thành công gọi `loadChapters()`; lỗi đưa tới chỗ Workspace hiện được; khi kết quả ghi mang `chapter_started` thì gọi `refresh_after_chapter_started` rồi `loadChapters()`.
- [x] `src/panels/editorPanelState.ts` + `src/modes/WorkspaceMode.vue` -- ref gợi ý bật ở nhánh `'confirmed'` khi không còn câu không cắt bỏ chưa xác nhận; dải có nút `dispatch('lifecycle.set_chapter_done')` và lệnh đóng mới; xoá trong `resetEditorPanel`, khi đổi Chương, khi đặt xong.
- [x] `src/modes/LibraryWorkPage.vue` + `src/i18n/vi.json` -- gỡ nút và khoá `mode.library.set_chapter_done`; giữ lệnh.
- [x] `tests/frontend/editorConfirmSegment.test.ts`, `tests/frontend/statusBarWorkspace.test.ts`, `tests/frontend/libraryWorks.test.ts` -- ca gợi ý (hiện / không hiện / đóng), nút (dispatch, khoá khi done, lỗi hiện), `loadChapters` sau đặt.
- [x] `e2e/specs/story-5-5-progress.e2e.mjs` -- giữ `setChapterStatusViaIpc` (nút Workspace đi qua `dispatch` nên tải lại `works` ngay, che mất phép đo `onActivated`), sửa chú thích bỏ món nợ Story 5.7; `story-5-12-reading-frontier.e2e.mjs` -- giữ cầu IPC (Chương 1 không phải Chương đang mở), sửa chú thích đầu tệp cho đúng lý do.

**Acceptance Criteria:**
- Given Chương `not_started` vừa nhận bản dịch đầu qua flush, when quay về Library, then thẻ Tác phẩm và danh sách Chương ghi Đang dịch mà không cần quét lại.
- Given Workspace mở một Chương, when bấm nút Đã xong, then danh sách Chương ở Library ghi Đã xong và tiến độ tăng.
- Given mỗi ca mới, when gỡ thật câu UPDATE hoặc lời gọi `mark_chapter_started` ở đường đó, then ca đó đỏ vì đúng lý do.

## Design Notes

- "Câu cuối" chỉ kích hoạt từ xác nhận, không từ cắt bỏ câu (`setCurrentSegmentOmitted`): PRD nói gợi ý khi xác nhận; Chương đủ điều kiện do cắt bỏ vẫn có nút.
- Khôi phục phiên bản và AI Proofreader chỉ chạm câu đã có chữ, nhưng người dùng có thể đặt tay Chương về Chưa bắt đầu; gọi chuyển tiếp ở mọi đường do người dùng giữ một quy tắc thay vì liệt kê ngoại lệ.

## Verification

**Commands:**
- `npm run build && cargo test --manifest-path src-tauri/Cargo.toml` -- toàn bộ: migration đổi wiring dùng chung, AGENTS.md buộc chạy đủ.
- `npm run test:story 5-18` -- vitest và Rust khai trong plan xanh.
- `npm run test:e2e -- --spec e2e/specs/story-5-5-progress.e2e.mjs --spec e2e/specs/story-5-12-reading-frontier.e2e.mjs` -- 2 spec xanh, chạy ngoài sandbox.
- `npm run check:commands && npm run check:i18n && npm run check:tokens && npm run check:panel-refs` -- xanh.

**Manual checks:**
- Ice: mở một Tác phẩm cũ có Chương đã dịch mà ghi Chưa bắt đầu, thấy nó thành Đang dịch; dịch Chương mới, xác nhận câu cuối, bấm gợi ý, về Library thấy tiến độ tăng (mục nợ `Chủ: Epic 5`).

## Implementation Notes

- Cờ `chapter_started` nằm trên chín lệnh, không phải sáu: nhận TM khớp đúng/khớp gần và nhận/hoàn tác của AI Proofreader là các lệnh riêng, và `confirm_segment` thêm ở vòng review 1 (chỉ ở lượt chuyển tiếp thật); mọi lệnh đi qua `mark_chapter_started` (hoặc biến thể theo `segment_id`) trong giao dịch ghi.
- Vị từ "câu có bản dịch" là macro `segment_has_translation_sql!` (`core/store/schema.rs`), dùng chung cho migration 42 và lượt ghi, nên hai đường không lệch nhau.
- Mở Tác phẩm vừa vượt bước 42: `open_work` viết lại `meta.json` qua `lifecycle::refresh_meta_after_backfill` (`meta_write_boundary` chỉ cho ba chỗ `write_atomic`), vỏ IPC reindex khi hàng chỉ mục lệch meta. Thêm `Store::opened_from_version()`.
- Webview: một helper `noteChapterStarted` trong `config/lifecycle.ts`, không await, nên flush không thêm lượt khứ hồi; Library đăng ký `loadChapters` vào đó vì `libraryChapters` đã import `libraryWorks` (tránh vòng import).
- Guard kiểu của chín kết quả nay bắt buộc `chapter_started`; kết quả thiếu cờ thành `ipc.unknown` thay vì `false` im lặng.
- Lệnh mới `lifecycle.dismiss_chapter_done_suggestion` vì `check:commands` buộc mọi `@click` là `dispatch`. Lỗi đặt Đã xong vào `openChapterDoneError` để Workspace hiện được.
- `check:i18n` đỏ sẵn ở `src/panels/AiTranslationPanel.vue:655` từ 4f290f75 (đo trên bản HEAD sạch), không do story này.
- e2e trên máy Ice: cổng 4445 bị Google Drive giữ; chạy với `TAURI_WEBDRIVER_PORT=4545`.

## Plan Change Log

- Vòng review 1 (2026-10-28): spec e2e 5-5 bấm `[data-chapter-done]` thì xanh cả khi `onActivated` hỏng, vì nút gọi `setOpenChapterStatus` → `applyOpenWorkLifecycleResult` → `loadWorks()`. Sửa dòng Tasks e2e: giữ lời gọi IPC trần ở 5-5, chỉ sửa chú thích. Tránh: ca e2e mất cơ chế nó đo. KEEP: mọi mã Rust, webview, vitest và sửa chú thích 5-12 giữ nguyên.

## Review Triage Log

- e2e 5-5 bấm nút Workspace không còn đo `onActivated` — medium, bad_plan: nút đi `dispatch` → `applyOpenWorkLifecycleResult` → `loadWorks()`; trả spec về lời gọi IPC, sửa chú thích, sửa Tasks.
- Gợi ý Đã xong còn hiện sau khi một câu quay lại chưa ký — low, patch: `editorChapterDoneSuggested` thành computed kiểm lại mọi câu; ca vitest mới, gỡ thử thì đỏ.
- Ca câu về hưu trong `lifecycle_contract.rs` không canh vế `retired_at IS NULL` — low, patch: câu về hưu mang chữ + flush rỗng; gỡ vế đó thì ca đỏ.
- Banner mới trong `lifecycle_contract.rs` trái AGENTS.md — low, patch: xoá.
- Khoá trùng ở frontmatter plan — low, sửa (lỗi ghi sổ của bước review, không phải mã).
- `openChapterDoneError` không xoá khi đổi Chương — low, reject: chỉ gặp sau một lượt đặt lỗi hiếm rồi đổi Chương; lượt bấm kế tiếp xoá nó.
- Lệnh `refresh_after_chapter_started` không mang Work, có thể chạy sau `open_work` khi đổi Tác phẩm — low, reject: invoke làm mới được gửi trước, `open_work` còn tìm chỉ mục và mở store trước khi giữ khoá; lệch thì lượt ghi trạng thái kế tiếp sửa (quyết định Q1).
- Xác nhận câu không gọi `mark_chapter_started`, nên Chương điền sẵn TM rồi xác nhận từng câu vẫn Chưa bắt đầu — intent_gap, Ice 2026-10-28 chọn tính xác nhận; §Always và PRD FR5 sửa kèm 🔵.
