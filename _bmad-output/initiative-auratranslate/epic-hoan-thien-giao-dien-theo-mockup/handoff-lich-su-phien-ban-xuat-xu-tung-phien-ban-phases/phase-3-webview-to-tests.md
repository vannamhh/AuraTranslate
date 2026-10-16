# Handoff Pha 3 Webview -> Pha 4 Tests (ticket 19)

## Đã làm (chưa commit)
- `src/config/segment.ts`: `SegmentVersion` có `translation_origin: VersionTranslationOrigin | null` + `spans: HistorySpan[]`; kiểu mới `SegmentHistory { current_version_id, source_text, versions }`; guard `isSegmentHistory` (thay `isSegmentVersionArray`, kiểm cả enum xuất xứ và từng span). `readSegmentHistory` nay trả `{ history: SegmentHistory | null, error }` (trước là `{ versions, error }`).
- `src/panels/segmentHistoryState.ts`: thêm `historyCurrentVersionId`, `historySourceText`, `historyCompareOn`, `toggleHistoryCompare()`, `moveHistoryAim(±1)`. `loadHistory` nhắm hàng đầu khi hàng đang nhắm không còn. `restoreVersion` gọi `await loadHistory(id)` sau `'restored'` (nạp lại nhãn + diff). Mở lịch sử đặt lại `compareOn=false`.
- `src/SegmentHistoryOverlay.vue`: hai ngăn (`.hist-two`): trái danh sách (thời điểm, badge "đang dùng", nhãn xuất xứ), phải chi tiết (nguyên văn `source_text`, toàn văn hoặc diff qua `ReviewSpans`, xuất xứ, nút "Khôi phục phiên bản này" = `history.restore`, nút "So với bản đang dùng" = `history.toggle_compare`). Dòng "chưa ký" (`data-history-unsigned`) khi có phiên bản mà `current_version_id` null. `@keydown.up/.down.prevent` trên scrim.
- Hook cho test: `data-history-row`, `data-history-current="true|false"`, `data-history-origin="self|other|bilingual_import|unknown"`, `data-history-detail`, `data-history-source`, `data-history-full`, `data-history-diff`, `data-history-unsigned`.
- `src/panels/ReviewSpans.vue`: prop `spans` nay `readonly ReviewDiffSpan[]`.
- Lệnh mới `history.toggle_compare` (`commands/index.ts` mảng 0-hợp-âm + `CommandDeps.toggleHistoryCompare`, `main.ts`, `vi.json` `command.history.toggle_compare`). `scripts/check-commands.mjs` HANDLER_TABLE thêm `SegmentHistoryOverlay.vue::moveHistoryAim` (R_CURSOR_LOCAL).
- `vi.json`: thêm `history.act_restore_version|act_compare|versions_count|source_label|detail_full|detail_diff|detail_current|pick_version|badge_current|origin_{self,other,bilingual_import,unknown}|origin_line|unsigned_current|keys_hint`; XOÁ `history.act_restore` (không còn chỗ dùng).

## Kiểm đã chạy
`vue-tsc --noEmit` sạch; mọi `npm run check:*` (trừ scope/fmt/clippy/debt) thoát 0 gồm check:commands, check:i18n, check:tokens, check:lint; eslint ba tệp chính sạch. Không đổi sàn nào (COMMAND_FLOOR 267 < 268 thật).

## Việc của Pha 4 (webview)
- `tests/frontend/segmentHistory.test.ts` đỏ 4/16 + 17 lỗi: mock `readSegmentHistory` vẫn trả `{ versions, error }` và fixture thiếu `translation_origin`/`spans`; đổi sang `{ history: { current_version_id, source_text, versions }, error }`. Cũng xem `libraryImportResetsSegmentHistory.test.ts`, `editorClearSourceCuts.test.ts`, `gridPanelRowErrorPriority.test.ts`, `e2e/specs/grid-row-error-label.e2e.mjs`, `e2e/support/panelReset.mjs` (có nhắc readSegmentHistory/state) và `e2e/specs/segment-history-restore.e2e.mjs:71` (đọc mảng trần).
- Ca mới nên có: nhãn theo `id` (hai hàng trùng chữ, chỉ hàng `current_version_id` mang `data-history-current=true`); nạp lại sau khôi phục (2 lượt đọc, nhãn chuyển hàng); toggle_compare bật/tắt toàn văn <-> diff; `current_version_id` null => dòng "chưa ký"; ↑↓ đổi hàng nhắm; không `v-html`; guard `isSegmentHistory` từ chối thiếu `spans`/xuất xứ lạ.
- Đối chứng đỏ theo plan: "đổi hàm chọn thành khớp nội dung ở webview" (so `target_text` thay vì `current_version_id`).
- Chưa kiểm bằng mắt/e2e: bố cục hai ngăn (270px | 1fr) chưa chụp trong webview thật; chưa chạy vitest nào ngoài segmentHistory.test.ts.
