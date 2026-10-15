# Handoff Pha Rust -> Pha Webview (ticket 14)

Rust xong, chưa commit. Tauri đổi snake_case Rust thành camelCase ở ĐỐI SỐ lệnh (`onlyMine`), còn khoá JSON trả về giữ snake_case.

## Lệnh mới

| Lệnh | Đối số | Trả về |
|---|---|---|
| `tm_only_mine_get` | không | `{ work_open: bool, only_mine: bool }` (`only_mine` = false khi không mở Tác phẩm) |
| `tm_only_mine_set` | `onlyMine: bool` | cùng hình `{ work_open, only_mine }`; không mở Tác phẩm -> lỗi `work.none_open` (`no_work_open`) |
| `tm_export_counts` | `tier: 'work' \| 'global'` | `{ all: number, mine: number }` = số `<tu>` file xuất sẽ có theo bộ lọc `all` / `mine`; tầng `work` không mở Tác phẩm -> lỗi `no_work_open` |

Cả ba đã đăng ký trong `generate_handler!` (`lib.rs`). `tm_export_counts` là `(async)`.

## Lệnh đổi

- `tm_list_pairs(pairOrigin, tier, search, sort?)`: `sort` tuỳ chọn, mặc định `'newest'`; giá trị: `newest | oldest | source | target | side` (sai -> `tm.invalid_filter`, `filter: 'sort'`). Sắp NHÓM trước khi cắt 200: `side` (không phải `origin`: `naming_boundary` cấm danh từ trần) = nhóm có hàng đầu là của tôi trước; `target` = theo đích của hàng đầu. Trả thêm `tier_totals: { work: number | null, global: number }` = số cặp lưu mỗi tầng, KHÔNG theo bộ lọc nào (`work` null khi không mở Tác phẩm). `health` giữ nguyên (3 mục `self|other|bilingual_import`, theo bộ lọc tầng). Pill "Của tôi" = `self`; "Của người khác" = `other` + `bilingual_import`. Danh sách luôn dùng `browse()`: đủ cặp bất kể công tắc.
- `tm_export_tier(tier, filter?)`: `filter` tuỳ chọn, `'all'` (mặc định) hoặc `'mine'` (sai -> `tm.invalid_filter`, `filter: 'export_filter'`). Hợp đồng `tier = both` bị từ chối giữ nguyên. Tên tệp gợi ý thêm `_mine`. Không phụ thuộc công tắc.
- `glossary_tm_suggestions(sourceTerm)`: trả `{ suggestions: [{ target_text, count }], hidden_count: number }` (trước là mảng). Hình sai (mảng) phải bị guard từ chối.
- `read_open_chapter_segments` / nạp Chương: `tm_prefill` là `{ kind: 'ran', hidden_count: number } | { kind: 'not_asked' } | { kind: 'skipped', code }`. `ran` KHÔNG còn vắng `hidden_count` (guard coi vắng là hình sai). `hidden_count` cộng theo từng nguồn phân biệt của các segment nháp rỗng có xét điền sẵn, theo cặp chữ phân biệt `(source, target)` chỉ còn bản của người khác; một nguồn có cả cặp của tôi lẫn cặp người khác vẫn đếm cặp người khác ẩn (không dùng riêng cho nguồn "đáng lẽ điền").
- `tm_fuzzy_matches`: thêm `hidden_count: number` ở `TmFuzzyMatches` (cạnh `segment_id`, `matches`, `exact`). Gồm cặp khớp ĐÚNG bị ẩn + cặp khớp mờ bị ẩn. Quan trọng cho `tmFuzzyStripState.runScan`: `matches` và `exact` cùng rỗng nhưng `hidden_count > 0` thì dải KHÔNG được xoá. Khi khớp đúng bị ẩn hết, Rust vẫn chạy quét mờ trên cặp của tôi (khớp đúng bị loại khỏi quét mờ như trước), `hidden_count` là tổng cả hai.
- Xem prompt (`ai_prompt_*` ledger): `ledger.tm` thêm `hidden_count: number | null` (`null` đúng khi `kind === 'not_asked'`; `0` thật khi công tắc tắt). Guard: `kind === 'searched'` thì `hidden_count` phải là số.

Không đổi: `tm_concordance` (hình giữ nguyên, luôn đủ cặp), `tm_count_pairs` (không lọc), `accept_tm_fuzzy` / `accept_tm_exact`.

## Khác với plan

- `tm_export_counts` là `(async)` (đọc cả tầng hai lần) nên `config_invariants` ghi `tm.rs` 6 plain / 7 async.
- Tên lệnh cờ: `tm_only_mine_get` / `tm_only_mine_set` (plan chỉ nói "lệnh đọc và đặt cờ"). Lệnh webview `tm.manage.only_mine_toggle` -> gọi `tm_only_mine_set` với `!onlyMine`.
- Cờ chỉ có hiệu lực từ thao tác kế tiếp; webview phải chủ động quét lại dải sau khi đặt cờ.
- Đếm theo tầng đặt tên `tier_totals` (không phải `tier_counts`).
- Cổng `tm_boundary.rs` đọc lời gọi `TmSidePolicy::browse(`; ca dương đòi `TmSidePolicy::from_work(` có mặt trong 4 hàm bề mặt.

## Test đã sửa / thêm (cho pha Tests that move)

- Mới: `src-tauri/tests/tm_policy_contract.rs` (8 ca: cờ mặc định + CHECK, tra đúng, lọc trước trần 3, gợi ý chốt, nguồn gốc lạ vẫn lỗi, điền sẵn, sổ chèn + lô giữ chính sách, xuất `mine`) và `tests/tm_boundary.rs` (5 ca). Có thể gộp vào `tm_contract.rs` hoặc giữ nguyên.
- Đã sửa cho biên dịch / hằng 36 -> 37: `tm_contract.rs`, `tmx_contract.rs`, `tm_wire.rs`, `ai_rag_contract.rs`, `ai_prompt_contract.rs` (:1567), `ipc_contract.rs`, `config_invariants.rs`, `review_harvest_contract.rs`, `segment_contract.rs`, `pinned_contract.rs`.
- Còn nợ pha Tests: ca ipc cho `TmOnlyMineState` / `TmExportCounts` / `GlossaryTmSuggestions` (khoá JSON), ca wire `tm_only_mine_set` không mở Tác phẩm, ca sắp `ManageSort` trên >200 nhóm, ca migration 36 -> 37 giữ hàng `work` cũ bằng 0, đối chứng đỏ bốn phép gỡ ở plan.

## Còn đỏ có chủ ý

- `cargo test --test ipc_argument_contract` đỏ cho tới khi pha Webview thêm lời gọi `invoke('tm_only_mine_get')`, `invoke('tm_only_mine_set', { onlyMine })`, `invoke('tm_export_counts', { tier })` trong `src/**/*.ts` (cổng kiểm cả tên đối số camelCase, cả `sort` / `filter` mới của `tm_list_pairs` / `tm_export_tier` nếu webview gửi). Đó là bằng chứng kiểm tên đối số, không bỏ qua.
- Full `cargo test` lượt cuối (sau khi sửa hằng 37, sàn `BOUNDARY_FILE_FLOOR` 19, `config_invariants` 96/62): chỉ còn `ipc_argument_contract` đỏ; các mục tiêu đã sửa chạy lại xanh riêng lẻ, chưa chạy lại full lần hai.

## Đo

`rank_manage_listing` trên 200.000 cặp (100.000 nhóm, chỉ tầng Global), `cargo test --release`, một lượt, tải máy 3,75: đọc hàng ~155-180 ms; xếp hạng ~660-730 ms mọi trục sắp (newest 667, oldest 662, source 706, target 732, side 669). Không có bản đo trước thay đổi trên cùng quần thể nên không so được; sắp nhóm theo trục không đổi bậc đáng kể giữa các trục. `nfr_bench` chưa chạy.

# Handoff Pha Webview -> Pha Tests that move (ticket 14)

Webview xong, chưa commit. Xanh: `vue-tsc` (phần `src/`), `eslint src`, `check:commands|i18n|tokens|panel-refs|layout|gates`, `cargo test --test ipc_argument_contract`. Để nâng sàn `check-commands`: `DISPATCH_FLOOR` 280 -> 299, `HANDLER_ATTR_FLOOR` 114 -> 122 (sàn trôi dưới 80% số thật, không phải tầng quét hỏng).

## Tệp mới / đổi chính
- Mới: `src/tmPolicyState.ts` (`onlyMine`/`workOpen` là `boolean | null` đến khi Rust trả; `loadTmPolicy`, `toggleTmOnlyMine(): Promise<boolean>`, `resetTmPolicy`), `src/promptMarkers.ts` (`splitPromptBody(body, known)` -> `{text, kind: plain|known|unknown}[]`, cùng luật quét với `scan_markers`).
- Lệnh mới `tm.manage.only_mine_toggle`, `tm.manage.export_mine` (không phím). Quét lại dải sau khi đổi cờ nằm ở `tmManageCommandDeps.ts` (không ở state, để state không kéo `editorPanelState`): `syncTmFuzzyStrip(editorCaretSegmentId)` + `refreshGlossaryConfirmTmSuggestions()` (hàm mới ở `glossaryConfirmStripState.ts`).
- `config/tm.ts`: `tmListPairs(..., sort='newest')`, `tmExportTier(tier, filter='all')`, `tmExportCounts(tier)`, `tmOnlyMineGet/Set`; `TmManageListing.tier_totals` bắt buộc; `config/glossary.ts::listGlossaryTmSuggestions` trả `{suggestions, hidden_count}`; `TmPrefillStatus.ran.hidden_count`, `TmFuzzyMatches.hidden_count`, `ledger.tm.hidden_count` đều là guard chặt (vắng = hình sai).
- Thông báo sau điền sẵn: `NavNotice 'tm-prefill-hidden'` + `editorTmPrefillHiddenCount` (`editorPanelState.ts`), khoá `panel.grid.nav_tm_prefill_hidden`. Dải TM giữ lại khi chỉ có `hidden_count > 0` (`tmFuzzyIsEligible`, `tmFuzzyHiddenCount`). Dòng chung `tm.policy.hidden_note` ở dải TM (`data-tm-hidden-note`), dải chốt Glossary (cùng thuộc tính), Xem prompt (`data-aip-tm-hidden`).
- `TmManageOverlay.vue` viết lại phần template: lưới `role="grid"` (hàng `role="row"`, ô `gridcell`; KHÔNG còn `role="option"`/`listbox`), thanh tầng + pill xuất xứ là radio trong `<fieldset>` (`name="tm-tier-filter"` / `"tm-origin-filter"`), `<select>` chỉ còn ô Sắp theo, health là `.tm-health` > `.tm-legend li` (3 `li`), Sửa/Xoá mỗi hàng là hai `<form>` (`onRowEdit`/`onRowDelete` gọi `aimTmManageRow(i)` rồi `dispatch`), thanh `.tm-actions` chỉ còn Đẩy lên · Xoá của người khác · Chuyển lên/xuống, công tắc `button role="switch"` trong `.tm-policy`, hộp xuất có ô "Chỉ cặp tôi dịch", nút "Xuất N cặp ra TMX" và "Xuất riêng N cặp của tôi", TM trống có nút Nhập TMX. State mới: `tmManageSort`/`setTmManageSort`, `tmManageExportMineOnly`, `tmManageExportCounts` (`{work, global}` mỗi bên `{all, mine} | null`), `tmManageOriginCounts`, `aimTmManageRow`, `exportTmManageMine`, `refreshExportCounts` (gọi sau mỗi lần ghi và nhập).
- `PromptLibraryOverlay.vue`: thanh phạm vi (`name="pl-scope"`, Toàn cục n · Tác phẩm n · Cả hai) + `[data-prompt-in-use]` "Đang dùng: …"; nhãn `.pl-vs` cạnh ba biến (`prompt.library.var_<name>_need`); ô thân là `textarea.pl-ta-input` trên `pre.pl-ta-mirror` (`.pl-var`, `.pl-var-unknown`), KHÔNG còn nút Lưu và `.pl-body-form` là `div` (không `form`); tự lưu bằng `flushBody()` (chuỗi tuần tự, đợi `promptSetBusy` rỗi, trả `false` khi lỗi để chặn rời ô): gọi ở blur, chọn bộ (chỉ khi thân đổi), Bộ mới, Xuất, Nhân bản, Xem prompt, Esc, Đóng (nay là `<form>` `onClose`), và khi lớp phủ đóng bằng đường khác. Nút Xem prompt `[data-prompt-inspector-open]` (focus chính nó rồi `dispatch('ai.prompt_inspector.open')`), nút Nhân bản (`duplicatePromptSetName` ở `promptSetState.ts`, thử số kế khi `prompt_set.name_taken`, tối đa 20 lượt). `aiPromptInspectorHandlers.openAiPromptWorkPrompts` chỉ đóng Xem prompt khi thư viện đang mở.

## Test đang đỏ vì đổi hình (pha Tests phải sửa)
- 22 lỗi `tsc` ở `tests/**`: 14 fixture `kind: 'ran'` thiếu `hidden_count` (`support/segmentFixture.ts`, `aiBatchPanelRows`, `aiTranslateBatchResetWiring`, `chapterPosition`, `importPreviewDestination`, `libraryImport*`, `segmentSelection`...), fixture `ledger.tm` thiếu `hidden_count` (`aiPromptInspector*.test.ts`), `TmFuzzyMatches` thiếu `hidden_count` (`tmFuzzyStrip*.test.ts`), `listing()` thiếu `tier_totals` (`tmImport.test.ts`, `tmManage.test.ts` — riêng `tmManage.test.ts` tôi đã thêm `tier_totals` và ba mock `tmExportCounts/tmOnlyMineGet/Set` để thử dựng, còn 18 ca đỏ).
- `tmManage.test.ts`: tham số thứ tư `sort` (`'newest'`) của `tmListPairs`; `[role="option"]` -> `[role="row"].tm-row`; `.tm-toolbar select` hết (chỉ còn một `select` sắp); `.tm-actions` không còn Sửa/Xoá nên chỉ số nút đổi; `.tm-health li` vẫn 3.
- `tmImport.test.ts` (radio xuất `.tm-exchange-tier input[type="radio"]` vẫn có, nhãn nay kèm số cặp; nút xuất ở `.tm-exchange-actions button` nay có 3 nút), `glossaryConfirmStripTmOptions.test.ts` (mock `listGlossaryTmSuggestions` trả `{suggestions, hidden_count}`), `tmConfigGuards.test.ts`, `aiPromptConfigGuards.test.ts` (đối chứng chiều ngược cần `hidden_count`), `promptLibraryOverlayRender.test.ts` (ca "sửa thân" dùng nút Lưu đã bỏ -> blur; `.pl-body-form textarea` vẫn khớp).
- Chưa viết: ca vitest cho `splitPromptBody` + JSON fixture chung với `scan_markers`, `duplicatePromptSetName`, `tmPolicyState`, ca "bỏ lưu trước Nhân bản", e2e `story-12-14-*`, sửa e2e `story-12-10:72` cho hình trả mới của `glossary_tm_suggestions`, bốn phép gỡ đối chứng đỏ, `deferred-work.md`.

## Rủi ro chưa kiểm thật
- Chưa thử trong app/WebKit: lớp phản chiếu tô biến cần khớp điểm ảnh với `<textarea>` (cùng `padding`, font, `scrollbar-gutter: stable`, `::after` xuống dòng); `check:tokens` chỉ canh màu. Cần e2e hoặc mắt người.
- Tiêu đề dải TM khi chỉ có số ẩn vẫn đọc "0 câu gần giống".
- `refreshExportCounts` gọi `tm_export_counts` (đọc cả hai tầng hai lần, 4 phép đọc) sau mỗi lần ghi trong Quản lý TM; chưa đo trên quần thể lớn.
