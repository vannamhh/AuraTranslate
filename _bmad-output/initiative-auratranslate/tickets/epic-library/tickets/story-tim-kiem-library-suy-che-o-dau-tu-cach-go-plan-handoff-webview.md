# Handoff: pha Webview của 5-20

## Đã đổi (`src/`)
- `config/library.ts`: `searchLibrary(query, limit?, options)`, không còn tham số `mode` và khoá `mode` trong `invoke`. `SearchMode`/`SearchReport.mode` giữ (báo cáo vẫn mang chế độ phân loại).
- `modes/librarySearch.ts`: gỡ ref `mode`, export `librarySearchMode`, `setLibrarySearchModeExact/Lenient`. Thêm ref `classifiedMode` (chép `report.mode`, reset về `'exact'`) và export `librarySearchClassifiedMode`. `librarySearchStatus(hasLoaded, busy, indexedSegments, hitCount, shortQuery, widened, classifiedMode)` nhận đối số thứ 7.
- Mười trạng thái; thêm `no_match_lenient_typed` và `result_lenient_typed`. Thứ tự: busy · not_typed · index_empty · no_match_widened · short_query · no_match_lenient_typed · no_match · result_widened · result_lenient_typed · result. `short_query` đứng TRƯỚC `no_match_lenient_typed` (truy vấn `ab` không dấu 0 hit nói "dưới 3 ký tự", không nói "đã tìm cả biến thể").
- `modes/LibrarySearchResults.vue`: `modeNoteKind` = `widened` (cờ `librarySearchWidened`) | `lenient_typed` (chế độ phân loại lenient VÀ có hit) | null. Tìm chính xác không vẽ ghi chú. Ghi chú luôn mang lớp `mode-note--relax`. `no_match_lenient_typed` tự mang câu ở dòng trạng thái, không vẽ ghi chú.
- `modes/LibraryMode.vue`: gỡ khối `ui-seg` "Chế độ dấu". `commands/index.ts`, `main.ts`: gỡ 2 dep, 2 spec, 2 dòng nối.

## i18n (`vi.json`)
- Gỡ: `err.library.unknown_search_mode`, `command.library.search_mode_{exact,lenient}`, `mode.library.search_mode_{heading,exact,lenient}`, `search_note_exact`, `search_note_lenient`.
- Thêm: `mode.library.search_note_lenient_typed` (câu của Decisions), `mode.library.search_no_match_lenient_typed` ("Không tìm thấy — gõ không dấu nên đã tìm cả các biến thể dấu, vẫn không ra."). Giữ `search_note_widened`, `search_no_match_widened`, `search_counter`.

## Selector DOM cho e2e
- Vắng: `[data-library-search-mode]`.
- Dòng trạng thái: `[data-library-search-status]` (có hit: câu `search_counter`; 0 hit không dấu: câu `search_no_match_lenient_typed`).
- Ghi chú: `[data-library-search-mode-note]` hiện khi gõ không dấu có hit (câu "Gõ không dấu nên đã tìm cả các biến thể dấu…") hoặc tự nới (`search_note_widened`); vắng khi tìm chính xác.

## Kiểm
- Xanh: `check:i18n`, `check:commands`, `check:panel-refs`, `check:tokens`, eslint trên tệp đã đổi, `vue-tsc -p tsconfig.json` không lỗi ở `src/`.
- vue-tsc còn lỗi CHỈ ở `tests/frontend/` (việc của pha Tests): `librarySearch.test.ts` (gọi `librarySearchStatus` 6 đối số, dùng `librarySearchMode`/`setLibrarySearchMode*`), `librarySearchScope.test.ts:468` (`searchLibrary` 4 đối số), `libraryTopBar.test.ts:133` (dep `setLibrarySearchModeExact`).
- Chưa chạy vitest, chưa đối chứng đỏ (thuộc pha Tests). Chưa đụng `config/shortcutsState.ts`.
