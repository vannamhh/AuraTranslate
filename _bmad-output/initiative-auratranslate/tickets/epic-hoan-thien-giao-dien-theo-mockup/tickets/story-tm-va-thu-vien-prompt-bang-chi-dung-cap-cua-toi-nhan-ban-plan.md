---
title: 'TM và Thư viện prompt: bảng, chỉ dùng cặp của tôi, Nhân bản'
type: 'feature'
ticket: '14'
created: '2026-10-15'
status: 'built'
baseline_revision: '78bf298c0a023e30dedc8bbca6c555cf02728903'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/tm-manage.html'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/prompt-library.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Quản lý TM (`TmManageOverlay.vue`) là danh sách flex-wrap, không bảng, không sắp, không đếm theo tầng / xuất xứ, không giải thích; không có công tắc "chỉ dùng cặp của tôi" của Tác phẩm (FR149) nên cặp người khác dịch hoặc nhập song ngữ vẫn đi vào điền sẵn, dải TM, dải chốt Glossary, Smart RAG và dịch lô, và không xuất riêng được các cặp của mình. Thư viện prompt không có phạm vi tầng có số, dòng "Đang dùng", tô biến, nhãn bắt buộc / nên có, lối sang Xem prompt (FR71), Nhân bản, tự lưu khi rời ô; câu TM rỗng không mời Nhập TMX (c:61, c:63, c:65–c:67, c:84, c:85, c:89, c:90, c:93, c:96, a:169; C7–C11, C13).

**Approach:** Cột `work.tm_only_mine` ở bậc migration 37; kiểu `TmSidePolicy` bắt buộc ở hàm đọc `read_tier_rows` / `pairs_for_source` của `core/tm`, một chỗ lọc sau khi chấm điểm, mỗi bề mặt áp mang số cặp bị ẩn (AD-57); `tm_export_tier` nhận bộ lọc xuất độc lập với cờ. Webview dựng bảng TM theo mockup, công tắc, xuất riêng, và sửa Thư viện prompt theo mockup.

**Đã chốt (nguồn):**
- Giữ nguyên một story, không tách (Ice, 2026-10-13, epic Notes).
- AD-57 đã ký (Ice, 2026-10-13), kể cả điều mất: người khác dịch và nhập song ngữ bị LOẠI, không chỉ xếp sau (AD-57 mục 10). Nhãn công tắc nói rõ không gồm bản nhập song ngữ.
- Giữ một plan dù 9.646 token o200k (Ice, 2026-10-15).
- Sắp bảng TM (C7): Rust nhận tham số sắp và sắp NHÓM trước khi cắt `MANAGE_GROUP_LIMIT = 200`, nên mọi trục sắp đúng trên cả kho; đo lại `tm_list_pairs` sau khi thêm (Ice, 2026-10-15).
- Lọc xuất xứ (c:63): ba pill có số theo mockup, VÀ giữ hai bộ lọc đúng xuất xứ `other` / `bilingual_import` làm điều khiển phụ; không mất khả năng FR62 (Ice, 2026-10-15).
- Hộp xuất TMX (C11): giữ chọn một tầng mỗi lần xuất (hợp đồng `tm_export_tier` và ca từ chối `both` không đổi), thêm ô "Chỉ cặp tôi dịch" và số cặp từng tầng (Ice, 2026-10-15).
- "Kèm mọi bản dịch" (C11): không dựng ô; câu giải thích hộp xuất nói mọi bản dịch được giữ (FR63) (Ice, 2026-10-15).
- Tô biến (c:89): lớp phản chiếu `<pre>` dưới `<textarea>` chữ trong suốt, đồng bộ cuộn, cùng font; quy tắc tách `{{…}}` dùng một tệp JSON fixture chung cho Rust `scan_markers` và hàm webview (Ice, 2026-10-15).
- Xem prompt từ Thư viện (c:93): mở chồng lên, thư viện vẫn mở dưới, đóng Xem prompt thì tiêu điểm về nút đã mở nó và bản nháp còn nguyên (Ice, 2026-10-15).
- Nhân bản (C13): webview sinh tên "<tên> (bản sao)", "(bản sao 2)"… từ `promptSets`, gọi `prompt_set_create` cùng tầng, gặp `name_taken` thì thử số kế; không lệnh Rust mới (Ice, 2026-10-15).

## Boundaries & Constraints

**Always:**
- Mọi đọc TM phía đi qua `read_tier_rows` / `pairs_for_source` với `TmSidePolicy` làm tham số bắt buộc, không mặc định. `TmSidePolicy` chỉ dựng được bằng hai hàm trong `core/tm`: đọc cờ của Tác phẩm đang mở (không Tác phẩm nào mở thì ưu tiên của tôi) và `browse()`. `browse()` chỉ được gọi trong hàm `prepare_tm_concordance` và `prepare_tm_list`; một cổng đọc dòng mã canh điều đó.
- Chính sách đọc đúng một lần ở đầu mỗi thao tác (một lượt nạp Chương, một lượt chuẩn bị dải, một lô dịch) và đi xuống cùng ảnh chụp; đổi cờ có hiệu lực từ thao tác kế tiếp.
- Lọc ở đúng một chỗ trong `core/tm`, bằng phép chiếu `PairOrigin::side()` (AD-47 mục 6), SAU khi chấm điểm và TRƯỚC khi gộp trùng và cắt số lượng (trần 3 của dải TM, trần 3 của prompt); trả kiểu `PolicedMatches { kept, hidden_count }` mà chỉ hàm lọc dựng được.
- `hidden_count` đếm cặp chữ phân biệt `(source_text, target_text)` đáng lẽ khớp bề mặt đó mà chỉ còn bản của người khác; một cặp chữ còn bản của tôi vẫn hiện nên không tính. Cờ tắt hoặc chính sách `browse()` thì 0 thật (không phải "chưa biết").
- Bề mặt áp mang `hidden_count` ra webview: điền sẵn, dải TM, đề xuất TM của dải chốt Glossary, sổ chèn TM của prompt. Bề mặt rỗng vì công tắc nói điều đó, không nói "không có".
- Concordance và danh sách quản lý TM hiện đủ cặp bất kể cờ. Ghi không đổi: xác nhận segment vẫn ghi cặp TM với xuất xứ phân xử (AD-31, AD-50); bật cờ không viết lại chữ đã điền sẵn; tắt cờ thì cặp người khác về ngay.
- Cột: `ALTER TABLE work ADD COLUMN tm_only_mine INTEGER NOT NULL DEFAULT 0 CHECK (tm_only_mine IN (0, 1))`, một bậc mới của `PROJECT_MIGRATIONS` (số 37), không sửa `WORK_DDL`. Lệnh đặt cờ không nhận work id, chỉ ghi cột này (không `updated_at`, không `meta.json`, không dựng lại `library-index.db`).
- Xuất riêng: `tm_export_tier` nhận bộ lọc xuất `all | mine`, kiểu riêng, độc lập với cờ; cùng vị từ chiếu `side()`; thuộc tính TMX `x-aura-origin` không đổi. Con số N trên nút xuất do Rust đếm bằng chính hàm chọn cặp xuất.
- Giá trị `translation_origin` lạ vẫn là lỗi đọc (`UnknownPairOrigin`), không bị coi là "người khác" hay bị đếm vào cặp ẩn. Đếm theo tầng trả `Option` (tầng Tác phẩm là `None` khi không mở Tác phẩm), không `0`.
- Thư viện prompt: nhãn và chữ qua `vi.json`; màu qua token; mỗi `@click` là đúng một `dispatch('<id>')`, thao tác có tham số đi qua `<form>` + `@submit.prevent` như hiện nay; ô nhớ module mới có `reset*`.

**Never:**
- Không lọc ở webview, không bên tiêu thụ nào tự lọc theo `side`, không gọi `browse()` ở nơi khác, không tham số chính sách có mặc định.
- Không loại thứ 10 trong bảng AD-18, không cột công tắc ở `global.db` / `config_value` / `meta.json`; không đổi `scope_kinds!`.
- Không "Chương · câu" ở meta cặp TM (AD-6); meta chỉ xuất xứ và ngày. Không câu khẳng định "bạn đang ở vai biên tập" (vai là persona, không phải trạng thái app; C10).
- Không thêm biến `{{chapter_context}}` vào bảng biến (ngoài từ vựng đã chuẩn y, Epic 4); không đổi hợp đồng ba biến của `PromptVariable`.
- Không dependency mới, không màu / cỡ chữ literal, không `v-html`.
- Không sửa spine, PRD, epic, `tickets.toml`; AD-18 đã có dòng cập nhật 2026-10-13.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Cờ tắt (mặc định) | Mọi Tác phẩm cũ sau migration | Hành vi hôm nay; `hidden_count` 0; không dòng "bị ẩn" nào hiện | — |
| Điền sẵn, cờ bật | Segment nháp rỗng có nguồn khớp đúng một cặp người khác, không cặp của tôi | Không điền; `tm_prefill` báo `hidden_count` ≥ 1; webview hiện một thông báo nêu số cặp | — |
| Dải TM, cờ bật | 3 cặp người khác điểm cao hơn + 1 cặp của tôi điểm thấp hơn (đều ≥ ngưỡng) | Dải hiện đúng cặp của tôi, `hidden_count` 3 (lọc trước khi cắt trần 3) | — |
| Dải TM rỗng vì cờ | Mọi cặp khớp là của người khác | Dải vẫn hiện, một dòng "N cặp của người khác không dùng vì đang bật chỉ dùng cặp của tôi"; không tự biến mất | — |
| Dải chốt Glossary | Thuật ngữ T có TM nguồn = T chỉ của người khác | Không đề xuất TM; Hán Việt và ô gõ giữ; dòng nêu số bị ẩn | Lệnh TM lỗi: như 12.10 |
| Smart RAG / dịch lô | Cờ bật; prompt có `{{tm_similar_segments}}` | Prompt không mang cặp người khác; sổ chèn TM mang `hidden_count`; Xem prompt hiện dòng đó | Thân không có marker TM: `not_asked`, không đọc cờ cho TM |
| Đổi cờ giữa lô | Lô N câu đang chuẩn bị, người dùng bật cờ lúc đó | Cả lô dùng giá trị đọc ở đầu lô; lô sau dùng giá trị mới | — |
| Concordance và danh sách TM | Cờ bật | Vẫn hiện cả cặp người khác và cặp nhập song ngữ | — |
| Xuất riêng | Tầng có 3 cặp chữ: tự dịch, người khác, nhập song ngữ; thêm một cặp chữ có cả bản tự dịch và bản người khác | File có đúng 2 `<tu>` (cặp tự dịch + cặp chữ có bản của tôi, mang `x-aura-origin = self`); nút ghi "2 cặp"; cờ không ảnh hưởng | Không cặp nào của tôi: nút khoá, không tạo file |
| Không mở Tác phẩm | Chỉ tầng Global | Công tắc khoá kèm ghi chú sẵn có; xuất riêng tầng Global dùng được; mọi bề mặt chạy chính sách ưu tiên của tôi | — |
| Nhân bản prompt | Bộ "A" tầng Tác phẩm, đã có "A (bản sao)" | Tạo bản sao cùng thân, tên không trùng trong tầng đó; bộ gốc nguyên vẹn | `prompt_set.name_taken` ngoài dự kiến: hiện qua `tError`, không tạo gì |
| Tự lưu khi rời ô | Gõ vào thân rồi Esc / Đóng / chọn bộ khác / Nhân bản / Xem prompt | Thân được lưu trước khi rời; thân không đổi thì không ghi | Lưu lỗi: `tError`, bản nháp còn nguyên trong ô |
| TM trống | Cả hai tầng không cặp | Câu giải thích cơ chế + dòng "Đã có sẵn TM từ công cụ khác? Nhập TMX" mở `tm.manage.import_tmx` | — |

</frozen-after-approval>

## Code Map

Số dòng tính trên `78bf298`.

- `src-tauri/src/core/store/schema.rs` -- `PROJECT_MIGRATIONS` kết thúc ở `to_version: 36` (`PROOF_IGNORE_BLOCKED_COUNT_DDL`, :1023, :2393). Thêm bước 37 `WORK_TM_ONLY_MINE_DDL` theo khuôn `WORK_STATUS_OVERRIDE_DDL` (:887, `ALTER TABLE` riêng, không sửa `WORK_DDL` :848). Story 12.3 chỉ nâng `LIBRARY_INDEX_MIGRATIONS` 8 → 9 (không bậc project); luật `indexer.rs:2146` (nâng project khi đường đọc gõ cột mới) không kích hoạt vì chỉ mục không đọc cột này. Bậc kế (12.15 `proof_decision`) lấy số 38.
- `src-tauri/src/core/tm/mod.rs` (972 dòng) -- `PairOrigin::side()` :68 là phép chiếu duy nhất (giữ); `pairs_for_source` :217, `merge_tagged_tiers` :243, `first_per_text_pair` :288, `read_tier_rows` :328 + `TierRows` :323, `count_tier_pairs` :352 (SQL `translation_origin = 'self'` là bản định nghĩa "của tôi" thứ hai; giữ cho đếm, thêm ca so với `side()`), `target_suggestions_for_term` :376, `fuzzy_pairs_for_source` :398 (không ai gọi, xoá theo AD-57 mục 4), `rank_fuzzy_candidates` :410 (cắt `FUZZY_MATCH_LIMIT` ở :424), `fuzzy_pairs_in_candidates` :429, `rank_concordance` :480, `rank_manage_listing` :640 (cắt 200 nhóm :660), `PairOriginFilter` :574. Thêm `TmSidePolicy`, `PolicedMatches<T>`, `TmExportFilter`.
- `src-tauri/src/core/tm/tmx.rs` -- `distinct_tier_pairs` :664 nhận thêm `TmExportFilter`; `render_tmx` :180 không đổi; `ORIGIN_PROP = "x-aura-origin"` :12.
- `src-tauri/src/commands/segment/chapter_read.rs` -- `fill_exact_tm_matches` :520 (một `pairs_for_source` mỗi nguồn phân biệt; đọc chính sách một lần đầu hàm), `TmPrefillStatus` :129 (`Ran` hôm nay là biến thể đơn vị), gán ở :508-514.
- `src-tauri/src/commands/segment/tm_match.rs` -- `TmFuzzyMatches` :20, `prepare_tm_fuzzy` :73 (nhánh khớp đúng :107, `read_tier_rows` :132), `TmFuzzyScan`, `score_tm_fuzzy` :145, `prepare_tm_concordance` :228 (`browse()` ở đây), `accept_tm_fuzzy` (đọc lại cặp bằng `pair_by_id`, KHÔNG nhận chính sách).
- `src-tauri/src/commands/glossary.rs` -- `glossary_tm_suggestions` :547 (trả mảng; vỏ wire :1348); AD-57 liệt kê nó là bề mặt áp; đổi thành `{ suggestions, hidden_count }`. 12.10 để sẵn chỗ gắn chính sách ở `target_suggestions_for_term`.
- `src-tauri/src/core/ai/rag.rs` -- `TmRows` :288 + `load_tm_rows` :295 (đọc một lần, `TmRowsCache` ở `aiprompt.rs:472` giữ cả lô), `gather_tm_context` :314, `select_tm_pairs` :376 (trần `TM_PAIR_CAP = 3`, đếm `omitted_count`), `TmInjectionStatus::Searched { pairs, omitted_count }` :142, `assemble_prompt` :688 (23 chỗ gọi trong `tests/ai_rag_contract.rs`).
- `src-tauri/src/commands/aiprompt.rs` -- `TmInjectionStatusWire` :~250-285, `assemble_and_record_prompt_with_tm` :475; `src-tauri/src/commands/aitranslate.rs` :552 (lô chuẩn bị toàn bộ prompt trước khi gửi, nên đổi cờ giữa lô không đổi prompt đã lắp).
- `src-tauri/src/commands/tm.rs` (1195 dòng) -- `TmPairList` :93, `prepare_tm_list` :199 (`browse()` ở đây), `score_tm_list` :227, `tm_count_pairs` :296 (cho thanh trạng thái; không lọc), `tm_read_tier_pairs` :537, `tm_export_tier` :594 / `_after_dialog` :604, vỏ `wire::tm_export_tier` :~1075, `wire::tm_list_pairs` :837. Lệnh mới đặt ở đây.
- `src-tauri/src/lib.rs` -- `generate_handler!` ~:1412-1422 (đăng ký lệnh mới; đây là wiring chung ⇒ chạy full suite tay); bench `nfr_bench` ~:893-930 gọi `tm::wire::tm_list_pairs` với chữ ký hiện tại.
- `src-tauri/tests/support/boundary_scan.rs` -- `code_lines`, `without_test_modules`, `matching_close_brace`, `assert_population_floor` cho cổng mới `tests/tm_boundary.rs`.
- Test Rust phải đổi: `tm_contract.rs` (:778-823 và :1215-1229 cứng số 36; helper :918; :2090-2128 `rank_fuzzy_candidates` / `read_tier_rows`), `review_harvest_contract.rs:699` (`schema_version() == 36`), `tmx_contract.rs` (~10 lời gọi `tm_export_tier`), `ai_rag_contract.rs`, `ai_prompt_contract.rs` (:1567, :1727, :1766 hình JSON `tm`), `tm_wire.rs`, `glossary_commands_contract.rs`, `ipc_contract.rs`, `config_invariants.rs` (:1387-1460 bảng lệnh async theo tệp), `ai_boundary.rs:232`.
- `src/TmManageOverlay.vue` (997 dòng) -- `tm-health` badge :374, `tm-toolbar` (tìm, tầng `<select>`, xuất xứ `<select>`) :386, danh sách `tm-list` :461, thanh Sửa / Xoá chung `tm-actions` :595, khối xuất radio `tm-exchange` :642; bẫy Tab, `useSelectionSurface`, trả tiêu điểm về `[data-tm-manage-open]` giữ nguyên. `src/tmManageState.ts` (543) -- `loadRows` :173, `exportTmManageTier` :240, `setTmManage*` :326-353. `src/tmManageCommandDeps.ts`, `src/config/tm.ts` (`TmManageListing` :31, `tmExportTier` :344, `tmCountPairs` :241).
- `src/TmFuzzyStrip.vue` + `src/tmFuzzyStripState.ts` -- `runScan` :101 xoá dải khi `matches` và `exact` cùng rỗng và `tmFuzzyIsEligible` :60: phải giữ dải khi `hidden_count > 0`. `src/config/segment.ts` -- `TmPrefillStatus` :216 + guard :221 (36 fixture `kind: 'ran'` ở 26 tệp vitest), `TmFuzzyMatches` :1178 + guard :1220.
- `src/GlossaryConfirmStrip.vue`, `src/glossaryConfirmStripState.ts` (:229 `listGlossaryTmSuggestions`), `src/config/glossary.ts` (:588-620).
- `src/panels/editorPanelState.ts` :223-229 -- chỗ đọc `tm_prefill`; mẫu thông báo `datThongBao({ nav: 'tm-prefill-skipped' })` (`StatusBar.vue:440`).
- `src/AiPromptInspectorOverlay.vue` (khối TM của sổ chèn), `src/config/aiprompt.ts` :82-83 / :234 (guard `omitted_count`).
- `src/PromptLibraryOverlay.vue` (1094) -- nhóm tầng `pl-group-h` :517 / :528, bảng biến `pl-vars` :674, ô thân :685, nút Lưu :700, forms thao tác; `src/promptSetState.ts` (`createPromptSet` :180, `isPromptSetNameValid` :130, `busy` chặn lượt thứ hai); `src/promptLibraryState.ts`. `ai.prompt_inspector.open` (`src/commands/index.ts:4530`) đã có, không cần lệnh mới cho nút Xem prompt.
- `src/commands/index.ts` -- `tm.manage.*` :4006-4105, `prompt.library.*` :4446-4457; `scripts/check-commands.mjs` (`HANDLER_TABLE`, sàn `CLICK_FLOOR` / `COMMAND_FLOOR`).
- e2e: `e2e/specs/story-12-10-glossary-tm-suggestions.e2e.mjs:72` gọi `glossary_tm_suggestions` (hình trả về đổi). Mockup: `tm-manage.html` (`.bar`, `.health`, `.pair`, mục 2 `.btnrow`, mục 3 hộp xuất), `prompt-library.html`, `empty-states.html:126`.

## Tasks & Acceptance

**Execution:**

Pha Rust:
- [ ] `src-tauri/src/core/store/schema.rs` -- bậc 37 thêm cột `work.tm_only_mine` (DDL ở Always), đăng ký vào `PROJECT_MIGRATIONS` -- AD-57 mục 1, 9.
- [ ] `src-tauri/src/core/tm/mod.rs`, `tmx.rs` -- `TmSidePolicy` (hai hàm dựng; trường riêng), `PolicedMatches<T>` (trường riêng, một hàm lọc), `TmExportFilter`; `read_tier_rows` và `pairs_for_source` nhận chính sách, `TierRows` mang nó; lọc trong `fuzzy_pairs_in_candidates` / `pairs_for_source` sau chấm điểm và trước gộp trùng và cắt; `target_suggestions_for_term` trả kèm `hidden_count`; xoá `fuzzy_pairs_for_source`; `distinct_tier_pairs` nhận bộ lọc xuất; `rank_manage_listing` trả đếm theo tầng (`Option` cho tầng Tác phẩm) và nhận tham số sắp, sắp nhóm trước khi cắt 200 -- AD-57 mục 3-5, 8; c:61, C7.
- [ ] `src-tauri/src/commands/segment/chapter_read.rs`, `tm_match.rs` -- điền sẵn và dải TM đọc chính sách một lần; `TmPrefillStatus::Ran { hidden_count }`; `TmFuzzyMatches.hidden_count` (gồm cặp khớp đúng bị ẩn); `prepare_tm_concordance` dùng `browse()` -- AD-57 mục 5-6.
- [ ] `src-tauri/src/commands/glossary.rs` -- `glossary_tm_suggestions` trả `{ suggestions, hidden_count }` -- FR146 + AD-57.
- [ ] `src-tauri/src/core/ai/rag.rs`, `src-tauri/src/commands/aiprompt.rs` -- `TmRows` mang chính sách đọc lúc nạp; `gather_tm_context` trả kèm số ẩn; `TmInjectionStatus::Searched` thêm `hidden_count`; wire `hidden_count: Option` (`None` ⇔ `not_asked`) -- Smart RAG và dịch lô.
- [ ] `src-tauri/src/commands/tm.rs`, `src-tauri/src/lib.rs` -- `prepare_tm_list` dùng `browse()`; lệnh đọc và đặt cờ (không work id; trả `work_open` + `only_mine`); `tm_export_counts(tier)` (số cặp xuất `all` / `mine`, qua `distinct_tier_pairs`); `tm_export_tier` nhận bộ lọc; `TmPairList` thêm đếm theo tầng; đăng ký và cập nhật `config_invariants.rs`, `ipc_contract.rs` -- C9, C11.
- [ ] `src-tauri/tests/tm_boundary.rs` (mới) -- cổng đọc dòng mã: `browse()` chỉ trong `prepare_tm_concordance` và `prepare_tm_list`; sàn quần thể; nhận cả lời gọi mới thêm.

Pha Webview:
- [ ] `src/config/tm.ts`, `segment.ts`, `glossary.ts`, `aiprompt.ts` -- kiểu và guard cho trường mới (`hidden_count`, đếm theo tầng, bộ lọc xuất, hình trả của lệnh cờ); trường vắng là hình sai.
- [ ] `src/tmPolicyState.ts` (mới) + `src/commands/index.ts` + `src/main.ts` + `scripts/check-commands.mjs` -- ô nhớ `onlyMine` (`Option` đến khi đọc), lệnh `tm.manage.only_mine_toggle` và `tm.manage.export_mine` (không phím), `HANDLER_TABLE`, `reset*`; đổi cờ làm các dải quét lại.
- [ ] `src/TmManageOverlay.vue`, `src/tmManageState.ts`, `src/i18n/vi.json` -- bảng lưới 4 cột (nguồn · đích · meta xuất xứ + ngày · Sửa / Xoá mỗi hàng bằng `<form>` đặt con trỏ rồi `dispatch`), vạch lề theo xuất xứ (token: tự dịch `primary`, người khác `tm-rule`, nhập song ngữ `outline`; cần nhãn chữ kèm để màu không là kênh duy nhất), thanh tầng có số, pill xuất xứ có số kèm điều khiển phụ cho `other` / `bilingual_import`, điều khiển sắp gửi tham số sắp xuống Rust, health bar thanh chồng + chú giải + hai hộp giải thích (C8), dải bối cảnh tỉ lệ người khác khi người khác > của tôi (C10, không câu "vai biên tập"), công tắc "Chỉ dùng cặp của tôi cho Tác phẩm này" (nhãn nói rõ không gồm bản nhập song ngữ; nêu "có hiệu lực từ thao tác kế tiếp"), nút "Xuất riêng N cặp của tôi", hộp xuất chọn một tầng + ô "Chỉ cặp tôi dịch" + số cặp từng tầng + câu "mọi bản dịch được giữ", kèm minh hoạ `x-aura-origin`, dòng "Nhập TMX" ở trạng thái TM trống (a:169) -- c:61, c:63, c:65–c:67, C7–C11, a:169.
- [ ] `src/TmFuzzyStrip.vue`, `tmFuzzyStripState.ts`, `GlossaryConfirmStrip.vue`, `glossaryConfirmStripState.ts`, `panels/editorPanelState.ts`, `AiPromptInspectorOverlay.vue` -- dải TM không biến mất khi `hidden_count > 0`; dòng "N cặp của người khác không dùng" ở dải TM, dải chốt, thông báo sau điền sẵn, khối TM của Xem prompt -- AD-57 mục 6.
- [ ] `src/PromptLibraryOverlay.vue`, `src/promptSetState.ts`, `src/promptLibraryState.ts`, `src/i18n/vi.json` -- phạm vi tầng có số (Toàn cục n · Tác phẩm n · Cả hai), dòng "Đang dùng: …", nhãn bắt buộc có / nên có cạnh ba biến (không hàng thứ tư), tô biến bằng lớp phản chiếu (fixture JSON chung với `scan_markers`), nút Xem prompt cuối cùng đã gửi (`dispatch('ai.prompt_inspector.open')`, mở chồng lên thư viện; khi thư viện đang mở, nút "Mở prompt Tác phẩm" của Xem prompt chỉ đóng Xem prompt để về thư viện), Nhân bản (sinh tên ở webview, thử số kế khi `name_taken`), tự lưu khi rời ô: bỏ nút Lưu, lưu khi `blur` nếu thân đổi, và đợi lượt lưu đang bay trước Nhân bản / chọn bộ / Đóng / Xem prompt (`busy` đang chặn lượt thứ hai) -- c:84, c:85, c:89, c:90, c:93, c:96, C13.

Pha Tests that move:
- [x] Rust -- `tm_contract.rs` (chính sách: cặp người khác và nhập song ngữ bị ẩn ở điền sẵn, dải, gợi ý chốt, RAG; lọc trước trần 3; `hidden_count` theo cặp chữ; ca cờ tắt = hôm nay; giá trị xuất xứ lạ vẫn lỗi), `tmx_contract.rs` (xuất `mine`: đúng cặp, ca cặp chữ có hai bản), `ai_rag_contract.rs` + `ai_prompt_contract.rs` (sổ chèn `hidden_count`, đổi cờ giữa lô), `project_contract.rs` / migration (hàng `work` có sẵn lên bậc 37 giữ 0; CHECK từ chối 2), `tm_boundary.rs`, hằng 36 → 37 ở `tm_contract.rs` và `review_harvest_contract.rs`, `tm_wire.rs`, `ipc_contract.rs`, `config_invariants.rs`.
- [x] Vitest -- `tmManage.test.ts` (đếm tầng, pill, sắp, công tắc, xuất riêng, TM trống mời nhập), `tmFuzzyStrip*.test.ts` (dải còn khi chỉ có số ẩn), `glossaryConfirmStripTmOptions.test.ts`, `tmConfigGuards.test.ts`, `promptLibraryOverlayRender.test.ts` (phạm vi, Đang dùng, nhãn biến, Nhân bản, tự lưu và nhả bản nháp), `aiPromptInspectorScreen.test.ts`, `aiPromptConfigGuards.test.ts`, 26 tệp fixture `kind: 'ran'`.
- [x] e2e -- `story-12-14-tm-only-mine.e2e.mjs` (bật công tắc trên app thật, dải còn dòng "bị ẩn", concordance vẫn thấy cặp người khác) và sửa `story-12-10-glossary-tm-suggestions.e2e.mjs:72` cho hình trả mới; chưa chạy xanh trên máy Ice (webview ở `tauri://localhost`), chờ nightly.
- [x] Tô biến và Nhân bản -- tệp fixture JSON chung (ví dụ `tests/fixtures/prompt-variable-markers.json`) đọc bởi một ca Rust gọi `scan_markers` và một ca vitest gọi hàm tách của webview; vitest cho chuỗi tên bản sao và nhánh thử lại `name_taken`; e2e `story-12-14-prompt-library.e2e.mjs` đo lớp phản chiếu khớp vị trí với `<textarea>` sau khi cuộn (chờ nightly như trên).
- [x] Đối chứng đỏ (gỡ THẬT, giữ chữ ký, chạy đúng mục tiêu test): đổi lời gọi của `fill_exact_tm_matches` sang `browse()` ⇒ `tm_boundary` đỏ; đưa phép lọc xuống sau `truncate(FUZZY_MATCH_LIMIT)` ⇒ ca "3 người khác + 1 của tôi" đỏ; bỏ chuyển `hidden_count` vào `TmInjectionStatus` ⇒ ca sổ chèn đỏ; bỏ lưu trước Nhân bản ⇒ ca nháp đỏ.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- sau khi dựng: nợ "`{{chapter_context}}` ngoài từ vựng, mockup vẽ hàng thứ tư" (`Chủ: Winston`); đóng các mục TM liên quan; lượt dùng thật bật công tắc trên Tác phẩm có cặp người khác (`Chủ: Epic 12`).

**Acceptance Criteria:**
- Given một Tác phẩm có cặp TM do người khác dịch và cặp nhập song ngữ cho cùng một câu, when bật "chỉ dùng cặp của tôi" rồi mở Chương, then segment đó không được điền sẵn và thông báo nêu số cặp bị ẩn.
- Given công tắc bật và câu có cặp khớp mờ chỉ của người khác, when con trỏ vào câu, then dải TM vẫn hiện, không liệt kê cặp nào, và ghi số cặp bị ẩn vì công tắc.
- Given công tắc bật, when dải chốt Glossary mọc cho thuật ngữ có TM chỉ của người khác, then không có đề xuất TM, Hán Việt vẫn có, và dòng số bị ẩn hiện.
- Given công tắc bật, when dịch AI hoặc dịch lô với prompt có `{{tm_similar_segments}}`, then prompt không chứa cặp người khác và màn Xem prompt ghi số cặp bị ẩn.
- Given công tắc bật, when mở Concordance hoặc Quản lý TM, then vẫn thấy cặp người khác và cặp nhập song ngữ.
- Given công tắc bật rồi tắt, when mở Chương kế, then cặp người khác trở lại điền sẵn và dải; chữ đã điền sẵn trước đó không đổi.
- Given tầng có cặp của tôi và của người khác, when bấm "Xuất riêng N cặp của tôi", then file TMX chỉ có cặp của tôi, N bằng số `<tu>` trong file, và công tắc không ảnh hưởng kết quả.
- Given Quản lý TM, when mở, then thấy bảng nguồn · đích · meta xuất xứ + ngày · Sửa / Xoá mỗi hàng, số theo tầng và theo xuất xứ, health bar có tỉ lệ; TM trống thì có dòng mời Nhập TMX.
- Given Thư viện prompt, when mở, then thấy phạm vi tầng có số, dòng "Đang dùng", nhãn bắt buộc / nên có cạnh ba biến, và nút mở Xem prompt cuối cùng đã gửi.
- Given một bộ prompt, when bấm Nhân bản, then có bộ mới cùng thân, tên không trùng trong tầng, bộ gốc nguyên vẹn.
- Given đang gõ dở thân prompt, when rời ô, đóng lớp phủ, chọn bộ khác hoặc bấm Nhân bản, then thân đã đổi được lưu; thân không đổi thì không ghi.
- Given Tác phẩm cũ sau nâng bậc 37, when mở, then công tắc tắt và mọi bề mặt hành xử như trước.

## Implementation Notes

- Rust: bậc 37 `work.tm_only_mine`; `TmSidePolicy` / `PolicedMatches` chỉ dựng được trong `core/tm`, lọc một chỗ (`police`) sau chấm điểm và trước gộp trùng và cắt; `tm_boundary.rs` canh `browse()` chỉ ở `prepare_tm_concordance` và `prepare_tm_list`.
- Lệnh mới `tm_only_mine_get` / `tm_only_mine_set` (không work id, lỗi `work.none_open` khi không mở Tác phẩm) và `tm_export_counts`; `tm_list_pairs` nhận `sort` và trả `tier_totals` (`work` null khi không mở Tác phẩm); `glossary_tm_suggestions` trả `{ suggestions, hidden_count }`. Cờ đổi chỉ có hiệu lực từ thao tác kế tiếp nên webview quét lại dải sau khi Rust nhận (`tmManageCommandDeps.ts`).
- Đối chứng đỏ bằng phép gỡ thật, mỗi lần đúng mục tiêu: `fill_exact_tm_matches` đổi sang `browse()` làm `tm_boundary` (2 ca) và ca điền sẵn đỏ; đẩy `police` xuống sau `truncate(FUZZY_MATCH_LIMIT)` làm ca "3 người khác + 1 của tôi" đỏ (`left: []`, `right: ["mine"]`); gán `hidden_count: 0` thay giá trị của `TmContext` ở `assemble_prompt` chỉ làm ca sổ chèn trong `tm_policy_contract` đỏ (`ai_rag_contract` và `ai_prompt_contract` vẫn xanh, nên chỉ một ca canh nối này); bỏ `flushBody()` trước Nhân bản làm ca thứ tự `update` rồi `create` đỏ.
- Đo `tm_list_pairs` (`prepare_tm_list` + `score_tm_list`, 200.000 cặp, 100.000 nhóm, chỉ tầng Global, chia ba xuất xứ đều nhau), profile `bench-release` (`opt-level = "s"`, lto), macOS máy Ice, load average 1 phút 6,2–6,5, xen kẽ nền và sau thay đổi, mỗi bên 3 tiến trình × 5 lượt: nền `78bf298` đọc 189–267 ms (trung vị 198), xếp hạng 837–1019 ms (trung vị 877); sau thay đổi đọc 189–205 ms (trung vị 195), xếp hạng 865–1088 ms (trung vị 884). Chênh trung vị xếp hạng +0,8%, trong nhiễu; sắp nhóm và đếm theo tầng không làm đổi bậc. Con số 660–730 ms của pha Rust là build `release` khác (load 3,75), không so được với bảng này.
- Bốn phép đọc của `refreshExportCounts` (hai tầng, hai bộ lọc) sau mỗi lần ghi trong Quản lý TM chưa đo trên quần thể lớn.
- Tô biến: fixture JSON dùng chung `tests/fixtures/prompt-variable-markers.json` đọc bởi `scan_markers_agrees_with_the_shared_marker_fixture` (Rust) và ca vitest `khớp fixture dùng chung` gọi `splitPromptBody`; việc lớp phản chiếu khớp điểm ảnh với `<textarea>` chỉ đo được trong WKWebView nên thuộc `story-12-14-prompt-library.e2e.mjs`.
- Dải TM chỉ còn số ẩn thì tiêu đề là "Translation Memory" (`tm.fuzzy.title_hidden_only`), không còn "0 câu gần giống".
- Hai spec e2e `story-12-14-tm-only-mine` và `story-12-14-prompt-library` cùng sửa `story-12-10` cho hình trả mới chưa chạy được (webview đứng ở `tauri://localhost`); cặp "của người khác" của spec đầu đến từ nhập song ngữ qua IPC. Nợ lượt dùng thật và hàng biến thứ tư nằm ở `deferred-work.md`.
- `check:debt-owner` đỏ ngay trên cây này vì `PLANNED_TICKET_FLOOR` 150 trôi dưới 80% số thật (188); nâng lên 160 theo lời cổng.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 4 · false 4 · maybe-false 1; thêm 4 bị loại (low hiếm gặp, sửa plan, đã có nợ).
- medium · patch — Nhân bản / Xoá với thân đang sửa dở: blur-save bật `promptSetBusy`, nút thành `disabled` trước mouseup nên cú bấm đầu mất; sửa không khoá hai nút theo busy, `onDuplicate` chờ rảnh, thêm ca vitest.
- low · patch — `tm.manage.others_outweigh`, `tm.policy.hidden_note`, `panel.grid.nav_tm_prefill_hidden` gọi là "người khác dịch" trong khi số gồm cả nhập song ngữ; sửa câu.
- low · patch — pill "Cả hai tầng" không có số (plan: thanh tầng có số); hiện tổng từ `tier_totals`.
- low · patch — dải TM chỉ có số ẩn vẫn ghi gợi ý "1 2 3 nhận"; bỏ gợi ý khi không có hàng.
- low · patch — `.pl-ta-wrap` khai hai lần; gộp.
- maybe-false · defer — `refreshExportCounts` đọc trọn hai tầng bốn lượt sau mỗi lần ghi; cần đo ở 200.000 cặp (mục nợ `Chủ: Epic 12`).
- false — `selectRow` dùng hàng cũ sau flush: chỉ đọc `tier`/`id`/`name`/`body` của hàng đích, mà lượt lưu bộ khác không đổi bốn trường đó.
- false — `syncTmFuzzyStrip` không quét lại cùng câu: hàm luôn hẹn `runScan`, chỉ xoá `shown` khi đổi câu (`tmFuzzyStripState.ts:133`).
- false — `prepare_tm_fuzzy` trả số ẩn ở nhánh khớp đúng < 2 bản: không bật công tắc thì dải sẽ hiện danh sách hai bản, nên báo số ẩn là đúng AD-57.
- false — lớp phản chiếu lệch `.pl-textarea`: padding, family, cỡ, line-height và viền 1px trùng nhau (`PromptLibraryOverlay.vue:1150-1180`); điểm ảnh thật nằm ở mục nợ Epic 12.
- low · reject — `tm.manage.export_mine` chạy theo id khi 0 cặp: lệnh không phím, repo không có bảng lệnh, cả hai nút đã khoá ở 0; chặn thêm là thêm nhánh.
- low · reject — cổng `tm_boundary` bỏ lọt `from_work(None)` / `Self::browse(`: cần một người cố ý viết sai; đóng kín là thêm phân tích cú pháp.
- reject — hộp việc pha Rust / Webview chưa tích, nâng sàn `check-commands` chưa ghi: sửa là sửa plan.
- reject — e2e `story-12-14-*` chưa chạy: đã có mục nợ `Chủ: Epic 12`.

## Design Notes

- Phân pha (AGENTS.md): plan · Rust · Webview · Tests that move, mỗi pha một agent mới, bàn giao qua tệp ở `handoff-tm-va-thu-vien-prompt-bang-chi-dung-cap-cua-toi-nhan-ban-phases/`. Pha Rust nên đo thêm thời gian `tm_list_pairs` sau khi thêm đếm theo tầng và phép sắp nhóm, trên cùng quần thể 200.000 cặp của `nfr_bench`, cùng tải máy.
- Vì sao lọc trước khi cắt: `rank_fuzzy_candidates` cắt ba hàng sau khi sắp theo điểm; lọc sau cắt có thể bỏ hết ba hàng đầu (đều của người khác) trong khi cặp của tôi điểm thấp hơn vẫn tồn tại, tức dải rỗng sai. Cùng lý do với `select_tm_pairs`.
- `hidden_count` theo cặp chữ phân biệt, không theo hàng: cùng quy ước `first_per_text_pair` và `omitted_count` (ca `omitted_count_ignores_duplicates...`, `ai_rag_contract.rs:1845`).
- `TmPrefillStatus::Ran` thành biến thể có `hidden_count` để kiểu nói thật (`NotAsked` ≠ `Ran` 0 ẩn); giá phải trả là 36 fixture ở 26 tệp vitest.
- Hàng Sửa / Xoá theo `<form>` + `@submit.prevent` vì `@click` chỉ được là `dispatch('<id>')` không tham số; cùng khuôn `PromptLibraryOverlay.vue`.
- Nút Xem prompt không cần lệnh mới: `ai.prompt_inspector.open` chỉ ĐỌC bản ghi (Decision 2 spec 4.7), không lắp lại.
- Cổng `browse()` đọc `src/**`, không đọc `tests/**`; helper test gọi `pairs_for_source` với `browse()` được.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test tm_contract --test tmx_contract --test tm_wire --test tm_boundary --test ai_rag_contract --test ai_prompt_contract --test glossary_commands_contract --test ipc_contract --test config_invariants --test review_harvest_contract` -- xanh; chạm migration và `lib.rs` nên chạy full `cargo test` và full vitest một lần bằng tay.
- `npx vitest run tests/frontend/tmManage.test.ts tests/frontend/tmFuzzyStrip.test.ts tests/frontend/tmFuzzyStripContent.test.ts tests/frontend/glossaryConfirmStripTmOptions.test.ts tests/frontend/tmConfigGuards.test.ts tests/frontend/promptLibraryOverlayRender.test.ts tests/frontend/aiPromptInspectorScreen.test.ts tests/frontend/aiPromptConfigGuards.test.ts` -- xanh; `npm run test:story 12-14 -- --list` để xem tập chọn.
- `npm run check:commands`, `check:panel-refs`, `check:i18n`, `check:tokens`, `check:debt-owner` -- xanh.

**Manual checks (if no CLI):**
- `npm run tauri dev`: Tác phẩm có cặp tự dịch và cặp người khác; bật công tắc, mở Chương, đối chiếu dải TM, dải chốt, Xem prompt; xuất riêng và mở file TMX xem `x-aura-origin`; đối chiếu `tm-manage.html` và `prompt-library.html`. Webview e2e đứng ở `tauri://localhost` nên đây là nợ `Chủ: Epic 12`.
