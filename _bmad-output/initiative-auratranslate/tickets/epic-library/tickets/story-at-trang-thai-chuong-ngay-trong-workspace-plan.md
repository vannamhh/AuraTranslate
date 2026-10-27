---
title: 'Đặt trạng thái Chương ngay trong Workspace'
type: 'feature'
ticket: '18'
created: '2026-10-27'
status: 'ready-for-dev'
baseline_revision: ''
route: 'full'
route_source: 'auto'
risk: 'medium'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Gõ, nhận bản AI hay xác nhận câu đều không đổi `chapter.status`, nên Chương dịch trọn vẫn ở Chưa bắt đầu; Workspace không có đường đặt Đã xong, nút duy nhất nằm ở trang Tác phẩm Library và đổi Chương đang mở ở Workspace (PRD FR5 🔵 2026-10-27).

**Approach:** Rust tự chuyển Chưa bắt đầu → Đang dịch trong cùng giao dịch với lượt ghi bản dịch đầu tiên do người dùng, và một migration sửa dữ liệu cũ; Workspace có nút Đã xong dùng lệnh `lifecycle.set_chapter_done` sẵn có, và gợi ý không chặn khi xác nhận câu cuối; gỡ nút ở Library.

## Boundaries & Constraints

**Always:**
- Chuyển tiếp tự động duy nhất: `not_started → in_progress`, khi Chương có ít nhất một câu `is_omitted = 0`, chưa về hưu, bản dịch khác rỗng sau trim. `paused`/`done`/`in_progress` không bao giờ bị đổi tự động.
- Lượt ghi tính: flush gõ, nhận bản AI (gồm dịch lô, đi qua `promote_ai_translation`), nhận TM khớp mờ và khớp tuyệt đối do người bấm, nhận thay đổi reviewer (FR94), khôi phục phiên bản (FR101), nhận/hoàn sửa của AI Proofreader. Không tính: điền sẵn TM lúc mở Chương, gộp/tách câu, nhập.
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
- Test ghim phiên bản phải sửa: `tests/pinned_contract.rs:262-274`; `tests/segment_contract.rs:2200-2244` (bước giả 42 → 43, thêm `PROJECT_MIGRATIONS[40]`); `review_harvest_contract.rs:701`; `prompt_set_contract.rs:887,895`; `tm_contract.rs:778,817,823,1220,1234,1249,1269`; `proofread_decision_contract.rs:285`.
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
- Cổng: `check:commands` (`@click` đúng `dispatch('<id>')`), `check:i18n`, `check:tokens`, `check:panel-refs` (ref cấp module phải có `reset*()`).

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/commands/lifecycle.rs` -- hàm thuần `mark_chapter_started(tx, chapter_id) -> bool`: một câu `UPDATE chapter SET status='in_progress', updated_at=… WHERE id=? AND status='not_started' AND EXISTS(câu không cắt bỏ, chưa về hưu, bản dịch khác rỗng)`; vị từ "khác rỗng" khớp `str::trim` (25 code point `White_Space`, như `GLOSSARY_ENTRY_DDL`) -- một chỗ khai quy tắc.
- [ ] `src-tauri/src/commands/segment/{targets,confirm,tm_match,review_accept,history,proofread_accept}.rs` + `wire.rs` -- gọi `mark_chapter_started` trong giao dịch ghi của từng đường tính; trả cờ `chapter_started` trong kết quả IPC; thêm lệnh async `refresh_after_chapter_started` (khuôn vỏ `set_chapter_status` + `finish_with_reindex`, không UPDATE) đăng ký trong `lib.rs`, kèm hàng census ở `tests/config_invariants.rs`.
- [ ] `src-tauri/src/core/store/schema.rs` -- bước `to_version: 42` UPDATE dữ liệu như migration ở matrix; đường mở Tác phẩm viết lại `meta.json` và reindex khi bước 42 vừa chạy làm đổi hàng.
- [ ] `src-tauri/tests/*` -- sửa các test ghim phiên bản; `lifecycle_contract.rs` thêm ca cho mọi hàng matrix phía Rust, kèm một ca cho mỗi đường tính và ca điền sẵn TM.
- [ ] `src/StatusBar.vue` + `src/i18n/vi.json` -- nút "Đặt Chương là Đã xong" (`ui-btn ui-btn--sm`) ở nhánh Workspace, `dispatch('lifecycle.set_chapter_done')`, khoá khi Chương đã `done` hoặc đang bận; nhãn khoá ghi Chương đã xong.
- [ ] `src/modes/libraryWorks.ts` + các cổng IPC của sáu đường ghi trong `src/config/` -- sau khi đặt thành công gọi `loadChapters()`; lỗi đưa tới chỗ Workspace hiện được; khi kết quả ghi mang `chapter_started` thì gọi `refresh_after_chapter_started` rồi `loadChapters()`.
- [ ] `src/panels/editorPanelState.ts` + `src/modes/WorkspaceMode.vue` -- ref gợi ý bật ở nhánh `'confirmed'` khi không còn câu không cắt bỏ chưa xác nhận; dải có nút `dispatch('lifecycle.set_chapter_done')` và lệnh đóng mới; xoá trong `resetEditorPanel`, khi đổi Chương, khi đặt xong.
- [ ] `src/modes/LibraryWorkPage.vue` + `src/i18n/vi.json` -- gỡ nút và khoá `mode.library.set_chapter_done`; giữ lệnh.
- [ ] `tests/frontend/editorConfirmSegment.test.ts`, `tests/frontend/statusBarWorkspace.test.ts`, `tests/frontend/libraryWorks.test.ts` -- ca gợi ý (hiện / không hiện / đóng), nút (dispatch, khoá khi done, lỗi hiện), `loadChapters` sau đặt.
- [ ] `e2e/specs/story-5-5-progress.e2e.mjs` -- thay `setChapterStatusViaIpc` bằng bấm nút Workspace; `story-5-12-reading-frontier.e2e.mjs` -- giữ cầu IPC (Chương 1 không phải Chương đang mở), sửa chú thích đầu tệp cho đúng lý do.

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

## Plan Change Log

## Review Triage Log
