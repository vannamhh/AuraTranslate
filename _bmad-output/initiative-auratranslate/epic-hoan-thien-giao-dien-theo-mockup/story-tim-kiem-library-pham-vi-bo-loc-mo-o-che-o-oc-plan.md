---
title: 'Tìm kiếm Library: phạm vi, bộ lọc, mở ở Chế độ đọc'
type: 'feature'
ticket: '5'
created: '2026-10-14'
status: 'built'
baseline_revision: '6bff1b84752165939b6aa1c055733efd9121f0ef'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Tìm xuyên Library luôn tìm cả hai phía, không lọc được, kết quả là danh sách phẳng nằm dưới lưới Tác phẩm, không có số câu hay trạng thái xác nhận, và chỉ mở được vào Workspace (FR139; a:121, 122, 125, 127, 128, 129, 132; A7, A8, A9).

**Approach:** Lõi `Indexer::search` nhận phạm vi và bộ lọc ngay trong SQL, trước `LIMIT`. Kết quả thay lưới, nhóm theo Tác phẩm. Mỗi hit có số Chương, số câu, trạng thái câu và vạch lề theo phía. Màu tô khớp đổi theo chế độ. Một hàm mới trong `readingState.ts` mở hit ở Chế độ đọc tại đúng câu.

## Boundaries & Constraints

**Always:** Phạm vi, trạng thái và hướng dịch lọc ở SQL của chỉ mục (AD-8, AD-27). Tô khớp dựng từ dữ liệu có cấu trúc, không `v-html` (AD-16). Rỗng phải nói lý do. Nhãn chế độ dấu giữ `Chính xác dấu | Không dấu`. Tự nới vẫn chỉ chạy khi chính xác ra rỗng.

**Never:** Sửa `src/modes/ReadingMode.vue`. Lọc `hit.field` ở webview sau `limit`. Làm chân "Chỉ mục: … dựng lại lần cuối" (A10 thuộc story 3). Sửa lỗi trần ứng viên trigram (nợ của 10.9). Đặt phím cho `library.open_search_hit`, vì nó cố ý không có phím.

## Decisions (Ice, 2026-10-14)

- Giữ trọn kế hoạch, không tách phần giao diện.
- Mở ở Chế độ đọc là lệnh mới `library.open_search_hit_reading`, không phím mặc định. Hàng hit có nút `Mở ở Chế độ đọc`; gợi ý phím không ghi `⌘↵`; một mục nợ `Chủ: Epic 12` theo dõi phím. `editor.confirm_segment` không đổi.
- Hit ở Chương chưa xong vẫn mở ở Chế độ đọc; Chế độ đọc hiện trạng thái biên `frontier-only` có sẵn.
- Số kết quả theo Tác phẩm là số đếm thật cho mỗi phía, không phải số hit đã trả về. Phía bản dịch đếm bằng `GROUP BY work_id`. Phía nguyên văn đếm sau khi Rust xác minh từng ứng viên trigram; vượt trần ứng viên thì ghi `N+`, không bao giờ hiện số ứng viên như số thật.
- Trạng thái câu nằm trong chỉ mục: cột mới, chỉ mục v10, dựng lại toàn bộ ở lần mở đầu.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Chỉ bản dịch | Chuỗi chỉ có ở `target_text`, phạm vi Nguyên văn | 0 kết quả; câu rỗng nêu phạm vi | — |
| Cùng chuỗi | Phạm vi Bản dịch hoặc Cả hai | Hit ở phía bản dịch, có số câu | — |
| Lọc chặn | Kết quả chỉ ở Tác phẩm `in_progress`, lọc `Đã xong` | 0 kết quả, kèm "X trong Y Chương không được tìm tới" và bước "Bỏ bộ lọc" | — |
| Tự nới + lọc | Chính xác rỗng sau lọc | Nới trong cùng phạm vi và bộ lọc; dòng chế độ nền `--tm-bg` | — |
| Đếm nguyên văn | Ứng viên trigram có khớp giả | Số theo Tác phẩm chỉ tính ứng viên đã xác minh | — |
| Vượt trần ứng viên | Ứng viên phía nguyên văn vượt trần | Số ghi `N+` (cận dưới) | — |
| Hit cấp Chương | `segment_id` NULL | Hàng ghi Chương, không ghi câu | — |
| Giá trị lạ | `scope`/`status`/`source_lang` không hợp lệ | Không chạy SQL | `library.unknown_search_scope` hoặc khoá lỗi lọc có sẵn |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/library/indexer.rs` -- `search` L967–1108: ba đếm dân số ở L982–994 (thêm tổng Chương, và số Chương bị lọc chặn = `SUM(chapter_count)` của Tác phẩm không qua lọc). Phạm vi chặn nhánh nào chạy (L1022–1032, L1064–1090). Bốn SQL `search_target_text` L1504, `_nd` L1549, `search_source_text` L1614, `_nd` L1681 đã JOIN `library_work w`, nên thêm `AND w.status IN … AND w.source_lang = ?` và `ORDER BY` theo khoá sắp. Dùng lại bộ dựng mệnh đề của `list_works` L733–809, `WorkQuery` L2395, `WorkSortKey` L2354 (`updated_desc`, `name_asc`). Đếm theo Tác phẩm: phía bản dịch `GROUP BY w.work_id` trên cùng `MATCH` và cùng bộ lọc; phía nguyên văn lấy ứng viên tới trần (`search_candidate_ceiling` L1149, `MAX_SEARCH_LIMIT` L1142), xác minh chuỗi con như `search_source_text` L1614 rồi đếm, kèm cờ đã chạm trần. Struct hit L1443 thêm `segment_ord`, `segment_status`, `work_status`, `source_lang`. `harvest_work_text` L1877 (query L1932) và `HarvestedRow` L1849, INSERT ~L476: đọc `segment.status`.
- `src-tauri/src/core/store/schema.rs` -- `LIBRARY_WORK_DDL` L2556: `library_segment` thêm `segment_status` (NULL ở hàng cấp Chương). `LIBRARY_INDEX_MIGRATIONS` L2666: sửa DDL tại chỗ, `to_version` 9→10. `Indexer::open` xoá và dựng lại chỉ mục; không thêm bước migration.
- `src-tauri/src/commands/library.rs` -- `search_library` L779, `wire::library_search` L1005 (thêm `scope`, `status`, `source_lang`, `sort`, đều `Option`). Wire `SearchHit` L675, `SearchReport` L716 (thêm `chapters_total`, `chapters_filtered_out`, và số theo Tác phẩm cho mỗi phía kèm cờ cận dưới). Tham số lọc thêm `work_id` (`Option`) cho lượt xem cả một nhóm. Mẫu lỗi: `unknown_search_mode` L594, `unknown_sort` L574–642.
- Chỉ mục chỉ dựng lại lúc khởi động, khi Quét lại, khi nhập và khi đổi trạng thái (`project/wire.rs::reindex_library` L457; `lifecycle.rs:317`). Trạng thái câu trong chỉ mục cũng mới như snippet: xác nhận một câu không làm chỉ mục mới lên.
- `src/config/library.ts` -- `SearchHit` L645, `SearchReport` L662, guard L710/L738, `searchLibrary` L763.
- `src/modes/librarySearch.ts` -- ref mỗi dòng một cái L58–87 (`check:panel-refs`); `runLibrarySearch` L228 (đo ms quanh lời gọi IPC); `openCurrentLibrarySearchHit` L350; `resetLibrarySearch` L370 reset cả state mới.
- `src/modes/LibraryMode.vue` (1.560 dòng) -- dải trên L264–426 (ô tìm L266, chế độ dấu L280, bộ lọc Tác phẩm L318–425); `.search-results-block` L428–553; `.works-block` L555–725. Kết quả tìm thay `.works-block`. Đưa khối kết quả ra component mới `LibrarySearchResults.vue`, không nhồi thêm vào tệp này.
- `src/modes/readingState.ts` -- khuôn `openAimedReadingSegment` L208 và `openCurrentTocChapter` L393. Hàm mới: `openWorkById` → `openChapterById(chapterId, segId)` → đặt `anchorSegmentId` → `resetReading({preserveAnchor:true})` → `resetReadingToc()` → `setMode('reading')`. `ReadingMode.vue` `onActivated` tự cuộn tới neo. Chương chưa xong cho trạng thái `frontier-only` (L96).
- `src/panels/editorSegments.ts` -- `SEGMENT_RULE_VALUES` L81. Vạch lề dùng token `--color-primary`/`--ink-2`, không chép lớp `.rule-*` (Kiểm I của check-commands so hai chiều với `GridPanel.vue`).
- `src/commands/index.ts` -- khối 5.9 L2322 (`library.search` L2334, `open_search_hit` L2361). `Mod+Enter` thuộc `editor.confirm_segment` L3077; `Escape` thuộc `editor.clear_source_cuts` L3428; `Mod+Shift+Enter`, `Mod+Alt+Enter`, `Alt+Enter` cũng đã có chủ. Listener `keys.ts:604` chạy ở pha capture, `preventDefault` nhưng không `stopPropagation`. `DISPATCH_FLOOR` ở `scripts/check-commands.mjs:247`.
- `src/i18n/vi.json` -- `mode.library.search_*` L956–978.
- Mockup `_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/library-search-results.html` -- nguồn nhãn: bộ đếm, `.scopes`, chip lọc, `.mode-note`, `mark`/`mark.loose`, `.wk`, `.more`, `.hitrow`, gợi ý phím, `.empty`.
- Test: `src-tauri/tests/library_index_contract.rs` (~45 ca tìm), `library_commands_contract.rs` L795–908, `ipc_contract.rs:712` (đóng băng khoá wire), `config_invariants.rs:1530`; `tests/frontend/librarySearch.test.ts`, `libraryTopBar.test.ts`; `e2e/specs/story-5-9-library-search.e2e.mjs`, `story-5-10-diacritic-modes.e2e.mjs`, `story-12-2-library-top-grid-overlay.e2e.mjs`.

## Tasks & Acceptance

**Execution:**
- [x] `schema.rs`, `indexer.rs` -- cột `segment_status`, chỉ mục v10; phạm vi, lọc, sắp trong SQL; các đếm mới -- A7, A8
- [x] `commands/library.rs` + `tests/library_index_contract.rs`, `library_commands_contract.rs`, `ipc_contract.rs` -- tham số và trường wire; ca: chuỗi chỉ bản dịch ở từng phạm vi, lọc trước `LIMIT` (Tác phẩm bị lọc chiếm trọn trần mà Tác phẩm qua lọc vẫn ra), tự nới tôn trọng lọc, số Chương bị chặn, giá trị lạ bị từ chối; số nguyên văn loại khớp giả trigram (ứng viên có chuỗi con không liền mà vẫn qua `MATCH`), số vượt trần mang cờ cận dưới
- [x] `src/config/library.ts`, `librarySearch.ts` -- adapter, guard, state phạm vi, lọc, sắp, nhóm, ms, nhóm mở rộng
- [x] `LibrarySearchResults.vue` (mới) + `LibraryMode.vue` -- thay lưới; bộ đếm `N kết quả · X ms` và `N Chương · M Tác phẩm`; `.scopes`; chip lọc; dòng chế độ; tô khớp; nhóm; hàng hit; gợi ý phím; trạng thái rỗng ba bước -- a:121–132
- [x] `readingState.ts`, `commands/index.ts`, `main.ts`, `vi.json` -- hàm mở ở Chế độ đọc, lệnh `library.open_search_hit_reading`, nhãn
- [x] `tests/frontend/*` -- state phạm vi, lọc, nhóm, ms; hàm mở Chế độ đọc đặt neo đúng câu
- [x] Đo p95 của lượt tìm có đếm trên Library thật (ghi bản build và dân số) và ghi vào Implementation Notes; vượt NFR3 (500 ms) thì đưa Ice trước khi đi tiếp
- [x] `deferred-work.md` -- mục nợ `Chủ: Epic 12`: phím mở ở Chế độ đọc
- [x] `e2e/specs/story-12-5-*.e2e.mjs` + dời 5-9, 5-10 -- theo Verify của ticket

**Acceptance Criteria:**
- Given một chuỗi chỉ có trong bản dịch, when tìm ở phạm vi Nguyên văn, then 0 kết quả; when đổi sang Bản dịch, then hit hiện ra.
- Given một hit ở Chương đã xong, when mở ở Chế độ đọc, then Chế độ đọc hiện và câu của hit nằm trong khung nhìn.
- Given đang có kết quả, when xem Library, then lưới Tác phẩm không hiện, và kết quả nhóm theo Tác phẩm với đầu nhóm `tên · Trung → Việt · trạng thái · N kết quả`.
- Given chế độ khoan dung, when tô khớp, then màu khác chế độ chính xác.
- Given một Tác phẩm có nhiều kết quả hơn số hit đã trả về, when xem đầu nhóm, then số ghi bằng số đếm thật (hoặc `N+` khi chạm trần), và `Xem cả N kết quả trong Tác phẩm này` chạy lại lượt tìm chỉ trên Tác phẩm đó.

## Implementation Notes

- Trần đếm phía nguyên văn là `SOURCE_COUNT_CEILING = 5.000` ứng viên, không phải 25.000 (Ice chọn 2026-10-14). Lý do: ở trần 25.000, `the` khoan dung (khớp mọi câu nguồn) có p95 dao động 359–569 ms, vượt NFR3. Ở trần 5.000, p95 là 139–173 ms. Đo trên bản dev `opt-level=2`, Library tổng hợp 20 Tác phẩm và 40.000 câu, hai lượt liền nhau; bảng đầy đủ ở tệp bàn giao. Số chốt cho bản đóng gói thuộc Story 10.9.
- Bộ lọc của lượt tìm dùng chung state trạng thái, ngôn ngữ nguồn và sắp với dải trên Library (`libraryWorks.ts`). Một `watch` ở `LibraryMode.vue` chạy lại lượt tìm khi các state đó đổi. Lĩnh vực không gửi xuống, vì lõi không có bộ lọc lĩnh vực.
- `work_counts` do Rust sắp theo khoá sắp của bộ lọc, và webview xếp nhóm theo đúng thứ tự đó. Bản đầu xếp theo hit xuất hiện đầu tiên (phía bản dịch trước), nên một Tác phẩm chỉ khớp ở nguyên văn luôn đứng sau.
- Dòng `N Chương · M Tác phẩm` là tổng của chỉ mục (`chapters_total`, `works_total`), cùng con số mà trạng thái rỗng dùng.
- Thêm hai lệnh không phím ngoài kế hoạch: `library.search_group_toggle` và `library.search_group_see_all`. Lý do: mỗi `@click` chỉ được gọi một `dispatch()`. Nút trên hàng đặt con trỏ ở `@pointerdown`/`@focus`, vì WKWebView không focus nút khi bấm.
- `library_search` mang `#[expect(clippy::too_many_arguments)]`, cùng khuôn các lệnh IPC khác có nhiều tham số. `FILE_FLOOR` của `check-tokens` nâng từ 148 lên 159 vì có thêm một tệp `.vue`.
- Hàng hit ở phía nguyên văn không ghi trạng thái xác nhận, theo mockup.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick, 2026-10-14): 8 phát hiện, gồm medium 2 · low 3 · false 3 (một phát hiện tách hai nửa: một nửa low, một nửa false). Patch 5, reject 1, không loopback. Agent cài đặt đã vượt ~200k context nên các patch do phiên điều phối tự vá, riêng việc dọn chú thích giao cho một agent mới.

- medium · patch — `chapters_filtered_out` dùng `NOT (1 AND w.status IN …)`: Tác phẩm có `status` NULL bị bộ lọc loại khỏi lượt tìm nhưng không được tính vào số Chương bị chặn. Sửa: `NOT COALESCE((…), 0)`. Thêm ca `a_work_with_unknown_status_hidden_by_a_status_filter_counts_as_filtered_out`; ca này đỏ (0 ≠ 1) trước khi sửa.
- medium · patch — `⋯ N kết quả nữa` lấy số đếm thật trừ số hit đang hiện, trong khi mở rộng chỉ hiện thêm hit đã trả về, nên nhãn hứa nhiều hơn thứ nút đưa ra. Sửa: `remaining` = hit đã trả về mà chưa hiện. Kỳ vọng `37` của test cũ khẳng định đúng lỗi này, nên đổi thành `2`.
- low · patch — `rerunLibrarySearch` trả về sớm khi `!hasLoaded`, nên đổi bộ lọc trong lúc lượt đầu còn chạy thì bị bỏ, và kết quả hiện ra không khớp chip. Sửa: chạy tiếp cả khi `busy`, đi qua `reloadPending`. Thêm một ca vitest, đối chứng bằng phép gỡ thật: đỏ `expected [] to have a length of 1`.
- low · patch — hàng hit ghi tiêu đề Chương thay vì số Chương (Approach: "số Chương, số câu"). Sửa: `Chương {ord}` đậm, tiêu đề đi kèm. Thêm một ca DOM.
- false — nửa còn lại của phát hiện trên: hit phía nguyên văn không ghi trạng thái xác nhận. Mockup `library-search-results.html` ghi `câu 12 · nguyên văn` cho hit nguồn, cũng không có trạng thái.
- low · patch — chú thích tiếng Việt mới thêm trên các mục mà tên đã tự nói, trái §Code comments của AGENTS.md. Sửa: xoá, hoặc viết lại bằng tiếng Anh chỉ khi giữ một bất biến không hiển nhiên.
- low · reject — `From<CoreSearchReport>` đặt `scope` rỗng. Chỉ một chỗ gọi (`search_library_scoped`) và chỗ đó ghi đè, nên người dùng không gặp; cách sửa phải thêm trường vào lõi.
- false — e2e 5-10 vẫn chờ `.works-block`. `wdio.conf` chạy mỗi tệp spec trên một phiên mới (`maxInstances: 1`), nên `before` luôn bắt đầu không có lượt tìm nào; 5-10 chạy xanh 2/2.
- false — hai lệnh `library.search_group_toggle` và `library.search_group_see_all` không có mục nợ phím. Lệnh Library không phím là thông lệ (`deferred-work.md` L8089), và cả hai có nút trên màn hình.

## Design Notes

- "Hướng dịch" = `source_lang` → Việt: repo không có cột ngôn ngữ đích nào. Chip hiện `Trung → Việt` / `Anh → Việt`, lọc theo `source_lang`.
- Sắp sắp các nhóm bằng `WorkSortKey` (`Sửa gần đây` = `updated_desc`, `Tên`). Trong một nhóm, hit theo Chương rồi câu.
- Mỗi nhóm hiện 3 hit đầu. Bấm `⋯ N kết quả nữa` mở rộng nhóm tới hết số hit đã trả về; con trỏ `‹ k/n ›` chỉ đi qua hit đang hiện. Khi số đếm của nhóm lớn hơn số hit đã trả về, cuối nhóm mở rộng có `Xem cả N kết quả trong Tác phẩm này`, chạy lại lượt tìm với lọc `work_id`.
- `ms` đo ở webview quanh lời gọi IPC, vì đó là khoảng người dùng thực sự chờ; wire không mang thời gian.
- Phía nguồn và phía bản dịch của cùng một câu là hai hàng riêng, theo mockup.

## Verification

**Commands:**
- `npm run build && cargo test --test library_index_contract --test library_commands_contract --test ipc_contract` -- xanh
- `npx vitest run tests/frontend/librarySearch.test.ts` -- xanh
- `npm run test:story 12-5` -- các ca được chọn xanh (không thêm lệnh IPC, `lib.rs` không đổi)

**Manual checks (if no CLI):**
- e2e `story-12-5` trên WKWebView: chuỗi chỉ có trong bản dịch không ra ở Nguyên văn và ra ở Bản dịch; mở ở Chế độ đọc cuộn tới đúng câu.
