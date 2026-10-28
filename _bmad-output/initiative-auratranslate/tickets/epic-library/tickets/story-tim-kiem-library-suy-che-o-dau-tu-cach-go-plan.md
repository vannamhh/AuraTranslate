---
title: 'Story 5.20: Tìm kiếm Library suy chế độ dấu từ cách gõ'
type: 'feature'
ticket: 20
created: '2026-10-28'
status: built
baseline_revision: 'aeaeaa1779bf6ea1ed6a8774b35c917b89e2bd6d'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Người dùng phải biết có hai chế độ dấu và bấm đúng nút trước khi tìm; gõ "chi" không ra "chỉ/chí" mà không gì báo còn kết quả khác (AD-61 Prevents 1).

**Approach:** Thi hành AD-61 mục 1–9, không chép lại: Rust phân loại truy vấn và bỏ tham số `mode` khỏi `Indexer::search`, `search_library`, `search_library_scoped`, `library_search`; lượt khoan dung đặt hit đúng chữ trước (mục 3); Webview bỏ nút chế độ và hai lệnh `library.search_mode_*`, thêm hai trạng thái "khoan dung vì gõ không dấu"; đo thêm hai ca NFR3 (mục 7).

## Boundaries & Constraints

**Always:** `SearchReport.mode` mang chế độ đã phân loại; bất biến `widened == (mode == Exact && effective_mode == Lenient)` giữ nguyên; lượt chính xác luôn chạy trước (mục 2). Webview không phân loại, không gửi chế độ (mục 1). Một lệnh IPC giữ tên `library_search`.

**Never:** đổi hai chỉ mục `_nd`, các cột `*_text_fold`, phép gấp dấu, sàn 3 ký tự của nửa nguyên văn, AD-27 ở C3 (tra từ điển); sửa `run.sh` hay `summarize.mjs` của `6-18-ban-do` (bằng chứng đã đóng); sửa PRD, epic, UX (John và Sally đã xong, AD-61 mục 8); nới ngưỡng NFR3 khi đỏ; thêm dependency.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Gõ không dấu, có cả "chi" và "chỉ" | `chi` | Cả hai hit; hit "chi" mang `Exact` và đứng trước; trạng thái nói "gõ không dấu nên đã tìm cả các biến thể dấu" | Không lỗi |
| Gõ có dấu | `chỉ` | Chỉ hit "chỉ"; không biến thể khác; ghi chú chính xác | 0 hit chính xác trên chỉ mục có dữ liệu ⇒ tự nới như hôm nay |
| Chữ Hán | `分久必合` | Chính xác, không lượt `_nd` thêm | Không lỗi |
| Trần nhỏ, hit đúng chữ hạng thấp trong `_nd` | `ma`, limit 2 | Hit đúng chữ vẫn có mặt | `truncated = true` khi còn nữa |
| Gõ không dấu, 0 hit | `zzqq` | Trạng thái riêng "khoan dung vì gõ không dấu, không kết quả"; không dùng câu tự nới | Không lỗi |
| Phím tắt đã gán cho id đã gỡ | Cài đặt lưu `library.search_mode_exact` | Cài đặt nạp bình thường, không `diskRejection` | Id lạ bị bỏ qua |

**Decisions (Ice, 2026-10-28):**
- Câu trạng thái mới theo phương án A: dòng đếm `search_counter` giữ nguyên cả `{ms}`; câu "Gõ không dấu nên đã tìm cả các biến thể dấu; kết quả khớp đúng chữ đứng trước." là khoá ghi chú `search_note_lenient_typed`, ca 0 hit là `search_no_match_lenient_typed`, không dùng `{total}`. Phương án B (câu có `{total}` thay dòng đếm, mất `{ms}`) bị bác.
- Giữ nguyên plan dài khoảng 6.500 token (ước lượng), không tách phần đo NFR3.
- Tìm chính xác không hiện câu ghi chú nào về dấu (`ux-auratranslate/library-nghiem-thu-thi-giac.md` L5 🔵 2026-10-28): gỡ cả `search_note_exact`. Ca tự nới giữ `search_note_widened`.

</frozen-after-approval>

## Code Map

Rust (`src-tauri/`)
- `src/core/library/indexer.rs` -- `search`/`search_filtered` (~968–977), khối doc-comment 5.10 (~931–966), lượt chính xác ~1068–1082, `widened`/`effective_mode` ~1084, nhánh khoan dung ~1111–1145, `count_per_work` ~2018, `SearchMode` ~1402–1446, `fold_diacritics_case_preserving` ~1251 (tái dùng cho hàm phân loại, không đổi). `search_*_nd` nhận `limit`; nửa nguyên văn cắt ứng viên ở `search_candidate_ceiling(limit)`.
- `src/commands/library.rs` -- `unknown_search_mode` ~591, `search_library` ~828, `search_library_scoped` ~848 (`parsed_mode` ~857), `wire::library_search` ~1122 (`mode: Option<String>` ~1130, lời gọi ~1142).
- `src/core/i18n/mod.rs` -- `MessageKey::LibraryUnknownSearchMode` ~532 và comment ~512.
- `tests/ipc_contract.rs` ~1672 -- ghim `"mode: Option<String>"` của `library_search`; phải đổi thành canh tham số vắng.
- `tests/library_commands_contract.rs` ~795–880 -- ca `unknown_search_mode`, ca `Some("lenient")`.
- `tests/library_index_contract.rs` -- ~110 lời gọi `.search(.., mode)` / `search_filtered`; helper `rebuild_and_search` ~2851; bàn đo `bench_p95_of_a_library_search_over_the_story_6_18_library` ~5350–5530 (5 ca); ca khứ hồi `SearchMode::ALL` ~5072.
- `tests/project_contract.rs` ~6476, ~6607 -- hai lời gọi `.search(.., Exact)`.

Webview (`src/`)
- `config/library.ts` -- `SearchMode`/`SearchReport`/`isSearchMode` (giữ: báo cáo vẫn mang `mode`), `searchLibrary(query, limit, mode, options)` ~822: bỏ tham số `mode` và `mode: mode ?? null` ở `invoke`.
- `modes/librarySearch.ts` -- ref `mode` ~80 và `librarySearchMode` ~122, `setLibrarySearchMode*` ~494–508, `librarySearchStatus` ~288 (nhận `widenedValue`, cần thêm chế độ đã phân loại), `LibrarySearchStatus` ~264, lời gọi ~441, reset ~410/~625, khối doc-comment đầu tệp (nói "tám trạng thái", sửa thành mười).
- `modes/LibraryMode.vue` ~405–423 khối `ui-seg` "Chế độ dấu" và import `librarySearchMode` ~94.
- `modes/LibrarySearchResults.vue` -- `showsEmpty`/`showsModeNote`/`statusText()`/`modeNoteText()` ~53–102.
- `commands/index.ts` -- deps ~509–513, hai spec ~2579–2597. `main.ts` ~187–188 và ~727–728.
- `i18n/vi.json` -- gỡ `err.library.unknown_search_mode` (381), `command.library.search_mode_*` (462–463), `mode.library.search_mode_*` (1220–1222), `search_note_exact` (1244), `search_note_lenient` (1245); thêm `search_note_lenient_typed` và `search_no_match_lenient_typed` (Decisions trong khối frozen).
- `config/shortcutsState.ts` -- không đổi. `createKeymap` (`commands/keys.ts`) chỉ duyệt spec đã đăng ký nên override của id lạ bị bỏ qua, không ném; `parseShortcutsFile` đếm `unknownCount`. Lệnh Rust `shortcuts` không kiểm id.

Tests / đo
- `tests/frontend/librarySearch.test.ts` (nhóm `setLibrarySearchModeExact/Lenient` ~305–415, ~875–894, DOM widened ~554–615), `tests/frontend/libraryTopBar.test.ts` ~130–143, `tests/frontend/commandsRegistry.test.ts` (chỗ thêm ca phím tắt mồ côi), `tests/frontend/librarySearchScope.test.ts` (kiểm đối số `invoke`).
- `e2e/specs/story-5-10-diacritic-modes.e2e.mjs` -- bấm nút chế độ ở ca ②, đọc `aria-pressed` ở ca ①.
- `_bmad-output/implementation-artifacts/6-18-ban-do/` -- fixture 50 Tác phẩm × 100 Chương × 10 segment, nguồn `zh` mỗi segment dạng `天下大势{gc}分久必合{s}, the quick brown fox number {gc}-{s}` (`story_6_18_library.rs:32,134`): phần Latin tiếng Anh đã nằm trong cột nguồn nên không cần Tác phẩm nguồn tiếng Anh riêng. `run.sh` đòi đúng 5 ca/phiên (dòng 195) và `summarize.mjs` đòi 5 ca/phiên, 15 tổng.
- Hồ sơ: `story-hai-che-do-dau-plan.md` mục `## Spec Change Log` (~339); `deferred-work.md` (grep `Chủ: Epic 5`, cuối tệp có mẫu tiêu đề `## Deferred from: … (2026-10-28, ticket N của Epic M)`).

## Tasks & Acceptance

**Execution — Phase Rust** (cuối pha: `cargo test --test library_index_contract`, `--test library_commands_contract`, `--test ipc_contract`, `--test project_contract`; `cargo clippy` đã có trong gate)
- [ ] `indexer.rs` -- thêm hàm thuần `classify_query(&str) -> SearchMode` theo AD-61 mục 1 (đã `trim`); bỏ tham số `mode` khỏi `search`/`search_filtered`, gọi hàm này trong `search_filtered`; gỡ `from_wire`, `ALL`, `Default`, giữ `as_str` nếu dây cần -- một chỗ phân loại duy nhất
- [ ] `indexer.rs` -- nhánh khoan dung theo mục 3: mỗi nửa = hit chính xác theo thứ tự rồi hit `_nd` có rowid ngoài tập đó, cắt chung ở `limit`; lấy `_nd` với `limit + 1 + số hit chính xác` để hit trùng không ăn trần; `truncated` đúng khi còn nữa; nhãn `match_kind` vẫn từ tập rowid chính xác. Cập nhật doc-comment 5.10 (xoá câu "người dùng chọn tường minh bằng nút")
- [ ] `commands/library.rs`, `core/i18n/mod.rs` -- bỏ `mode` khỏi `search_library`, `search_library_scoped`, `wire::library_search`; xoá `unknown_search_mode` và `LibraryUnknownSearchMode`
- [ ] `tests/library_index_contract.rs` -- chuyển mọi lời gọi sang chữ ký mới; truy vấn không dấu nay đi nhánh khoan dung nên mọi ca khẳng định `effective_mode == Exact` hoặc `widened` trên truy vấn không dấu phải viết lại bằng truy vấn có dấu (ví dụ `mả` cho ca tự nới); xoá ca khứ hồi `from_wire`; thêm: bảng phân loại AD-61 mục 6a (chín dòng); ca `chi`/`chỉ` qua `search_library` (6b, fixture có cả segment "chi" đúng chữ lẫn "chỉ"); ca trần (6c)
- [ ] `tests/library_commands_contract.rs`, `tests/project_contract.rs`, `tests/ipc_contract.rs` -- bỏ ca `unknown_search_mode`; chuyển chữ ký; ghim `library_search` không còn `mode: Option<String>`
- [ ] Bàn giao -- ghi `…-handoff-rust.md` cạnh plan: chữ ký cuối, danh sách ca đã xoá/viết lại, kết quả đối chứng đỏ

**Execution — Phase Webview** (cuối pha: `npm run check:i18n`, `check:commands`, `check:panel-refs`, `check:tokens`, `npx tsc`/`vue-tsc` như `package.json`; vitest chỉ các tệp bên dưới)
- [ ] `config/library.ts` -- bỏ tham số `mode` của `searchLibrary` và khoá `mode` trong `invoke`
- [ ] `modes/librarySearch.ts` -- bỏ `mode`, `librarySearchMode`, `setLibrarySearchMode*`; `librarySearchStatus` nhận chế độ đã phân loại từ báo cáo, thêm `no_match_lenient_typed` và `result_lenient_typed` (tên đề xuất, đổi nếu có lý do); `widened` vẫn là cờ riêng của lượt tự nới; sửa doc-comment đầu tệp
- [ ] `LibraryMode.vue`, `LibrarySearchResults.vue` -- bỏ khối `ui-seg`; `statusText`/`modeNoteText`/`showsModeNote` tách ba trường hợp theo AD-61 mục 4 (chính xác · khoan dung vì gõ không dấu · khoan dung vì tự nới)
- [ ] `commands/index.ts`, `main.ts` -- gỡ hai dep, hai spec, hai dòng nối
- [ ] `i18n/vi.json` -- gỡ năm khoá ở Code Map, thêm `search_note_lenient_typed` và `search_no_match_lenient_typed` (Decisions); tìm chính xác không vẽ dòng ghi chú; không dọn các khoá `search_result_*` chết có sẵn
- [ ] Bàn giao -- `…-handoff-webview.md`: tên trạng thái, khoá i18n, selector DOM cho e2e

**Execution — Phase Tests + đo** (chạy `npm run test:story 5-20 -- --list` trước; không chạy bộ đủ trừ khi đỏ có dáng phụ thuộc thứ tự)
- [ ] `tests/frontend/librarySearch.test.ts`, `libraryTopBar.test.ts` -- bỏ ca nút chế độ; thêm: không còn `[data-library-search-mode]`; `searchLibrary` không gửi `mode`; hai trạng thái mới có câu riêng, khác câu tự nới
- [ ] `tests/frontend/librarySearchScope.test.ts:428` -- ca "chế độ chính xác ⇒ dòng chế độ không có lớp nới" đổi thành: tìm chính xác thì `[data-library-search-mode-note]` vắng
- [ ] `tests/frontend/commandsRegistry.test.ts` -- `installCommands({ …, bindings: { 'library.search_mode_exact': ['Mod+Shift+E'] } })` không ném, `shortcutsDiskRejection()` là `null`, id không còn trong `commandRegistry.list()`
- [ ] `e2e/specs/story-5-10-diacritic-modes.e2e.mjs` -- ca ① đọc câu "khoan dung vì gõ không dấu" thay `aria-pressed`; ca ② bỏ bước bấm nút, giữ khẳng định hit nửa nguyên văn; thêm khẳng định nút chế độ vắng. Chạy một lần ngoài sandbox (`npm run test:e2e -- --spec …`)
- [ ] `library_index_contract.rs` (bàn đo) -- giữ năm ca của `bench_p95_of_a_library_search_over_the_story_6_18_library` để `run.sh` còn đúng: đổi `target_explicit_lenient_truncated` thành `target_typed_lenient_truncated` (`ma cua toi`, `Lenient`, `widened = false`), `target_auto_widen_truncated` dùng `mả của tôi` (chính xác 0 hit, tự nới), `source_latin_unique` nay `Lenient`; thêm hàm `#[ignore]` thứ hai gồm hai ca mới của mục 7 (`quick brown fox` làm cắt trần nửa nguyên văn; `分久必合 the quick` trộn Hán và Latin), in `NFR3_CASE` như cũ
- [ ] Đo -- dựng thư viện bằng `story_6_18_library` (đặt `AURA_6_18_EXPORT_LIBRARY_ROOT`, tên thư mục chứa `auratranslate-nfr-bench-`, khoảng 210 s) rồi chạy hàm đo mới và hàm năm ca bằng `cargo test --profile bench-release … -- --ignored --nocapture` (lệnh trong comment đầu bàn đo), ba phiên, máy rảnh; ghi build, quần thể, tải máy vào `…/6-18-ban-do/nfr3-ad61-raw.tsv`. Không chạy `run.sh` (nó dựng cả app và mười phiên). p95 < 500 ms mỗi ca; đỏ thì dừng và quay lại AD-61, không nới ngưỡng
- [ ] Hồ sơ -- thêm một dòng 🔵 vào `## Spec Change Log` của `story-hai-che-do-dau-plan.md` trỏ về 5.20 (nút chọn và "người dùng chọn tường minh" không còn); thêm một mục nợ `Chủ: Epic 5` vào `deferred-work.md` (kiểm tay trong app thật: gõ có dấu, không dấu, chữ Hán, đọc dòng trạng thái; ≤5 dòng)
- [ ] Đối chứng đỏ (gỡ thật rồi chạy đúng target đó, không comment): (1) bỏ lời gọi `classify_query` trong `search_filtered`, cố định `Exact` ⇒ ca `chi` đỏ vì thiếu "chỉ"; (2) trả lại danh sách `_nd` trần thay cho "chính xác trước" ⇒ ca trần đỏ; (3) gỡ lời gọi `searchLibrary` bỏ `mode`, hoặc thêm lại `mode` vào `invoke` ⇒ ca vitest đối số đỏ; (4) phím tắt mồ côi: ca vitest phải đỏ khi đổi vòng lặp của `createKeymap` sang `Object.keys(overrides)` (id lạ ném) -- chỉ làm nếu tạm sửa được mà không giữ lại

**Acceptance Criteria:**
- Given thư viện có cả segment "chi" và "chỉ", when gõ `chi`, then kết quả có cả hai, "chi" đứng trước mang nhãn khớp đúng chữ, và dòng ghi chú nói đã tìm cả biến thể dấu.
- Given cùng thư viện, when gõ `chỉ`, then chỉ "chỉ" xuất hiện và ghi chú nói chế độ chính xác.
- Given Library mở, when nhìn thanh điều khiển, then không có nút "Chính xác dấu | Không dấu" và bảng phím tắt không liệt kê hai lệnh `library.search_mode_*`.
- Given Cài đặt lưu phím tắt cho `library.search_mode_exact`, when khởi động, then app mở bình thường và các phím tắt khác còn hiệu lực.
- Given thư viện 5.000 Chương, when chạy hai ca mới, then p95 mỗi ca < 500 ms, ghi build và quần thể.

## Implementation Notes

- Ca `quick brown fox` (nguyên văn chạm trần, gõ không dấu) trượt NFR3 trước khi sửa: p95 625–878 ms. Chậm có từ trước AD-61: nút "Không dấu" của bản `aeaeaa17` ra p95 772–993 ms trong cùng phiên. Ice chọn (2026-10-28) bỏ lượt `_nd` của một nửa khi lượt chính xác của nửa đó đã vượt trần, vì khi đó danh sách hit và cờ `truncated` không đổi. Đo sau khi sửa bằng hai hàm bench đã commit (`bench-release`, thư viện 6.18 gồm 50.000 segment, ba phiên R1–R3, máy tải 5–9): `quick brown fox` p95 331–374 ms, Hán + Latin 142–194 ms, năm ca cũ ≤ 226 ms. Số đo ở `6-18-ban-do/nfr3-ad61-raw.tsv` (probe A/B và R1–R3). Ca chưa chạm trần mà nhiều biến thể dấu chưa đo, đã ghi nợ `Chủ: Epic 5`.
- Ca Hán + Latin dùng truy vấn `分久必合5, the quick`, không dùng `分久必合 the quick`: fixture 6.18 có `{s}, ` giữa phần Hán và phần Latin, nên truy vấn trong plan ra 0 hit.

## Plan Change Log

## Review Triage Log

- quick · medium · patch — AC NFR3 chưa có số từ hàm bench đã commit cho code đã giao (bỏ `_nd` chỉ đo bằng probe A/B); chạy lại `bench_p95_of_the_ad_61_typed_lenient_cases…` trên cây thật.
- quick · medium · patch — `lenient_mode_never_loses_a_source_half_hit` tìm `分久必合`, nay phân loại Exact nên không vào lượt khoan dung; đổi sang truy vấn Hán + Latin.
- quick · low · patch — tên `an_explicit_lenient_search…`, `an_empty_index_with_mode_lenient…` và câu panic `(mode=Lenient)` còn tả nút chọn chế độ đã gỡ.
- quick · false — `*` và `(:^-)` ở `fts5_syntax_characters_run_clean_on_the_nd_branch_too` vẫn tới `_nd`: fixture có một segment, lượt chính xác ra 0 nên tự nới chạy `_nd`.
- quick · low · patch — `src/main.ts:724` còn banner `Story 5.10` không đứng trên dòng nào.
- quick · low · patch — dòng comment diff thêm hoặc sửa vi phạm AGENTS.md §Code comments (tiếng Việt, id story).
- quick · medium · reject — ca ít hit đúng chữ mà nhiều biến thể dấu chưa đo: đã ghi nợ `Chủ: Epic 5` ở `deferred-work.md` trong lượt này.
- quick · false — truy vấn ngắn 0 hit báo `short_query` trước `no_match_lenient_typed` là thứ tự ưu tiên đã có (④ trước ⑤); câu truy vấn ngắn nói đúng lý do 0 hit.
- quick · low · reject — 0 hit sau khi tự nới hiện cả câu trạng thái lẫn ghi chú "đã nới": `showsModeNote` ở `aeaeaa17` cũng bật ở `no_match_widened`, nên không do thay đổi này gây ra.
- triage · low · patch — `widened` không được xoá ở nhánh lỗi của `runLibrarySearch`, nên ghi chú "đã nới" cũ hiện cạnh thông báo lỗi; cổng nhánh `widened` theo `showsResults` hoặc `no_match_widened`.

## Design Notes

- Bỏ hẳn tham số `mode` khỏi `Indexer::search` thay vì giữ một seam cho test: AD-61 đặt lời gọi phân loại bên trong `search`, đối chứng 6b đòi gỡ chính lời gọi đó, và một seam ép chế độ sẽ cho test đi vòng qua nó. Giá là ~110 lời gọi test phải đổi chữ ký, phần lớn cơ học.
- Bàn đo giữ năm ca để `run.sh` (5 ca/phiên) và `summarize.mjs` (15 dòng) còn dùng được; hai ca mới ở hàm riêng và tệp `nfr3-ad61-raw.tsv` riêng thay vì sửa bằng chứng 6.18. Số p95 cũ của ba ca bị đổi đường đi (`source_latin_unique`, hai ca target) là số mới, không so thẳng với `nfr3-raw.tsv`.
- Hai ca target cũ (tự nới, khoan dung chọn tay) vốn cùng truy vấn `ma cua toi`; sau phân loại cả hai thành khoan dung gõ không dấu. Đường tự nới còn sống (mục 2) nên giữ một ca đo nó bằng truy vấn có dấu.

## Verification

**Commands:**
- `npm run test:story 5-20 -- --list` -- expected: liệt kê đủ tệp ở Code Map, không báo "plan không khai test"
- `npm run build` rồi `cargo test --manifest-path src-tauri/Cargo.toml --test library_index_contract --test library_commands_contract --test ipc_contract --test project_contract` -- expected: xanh
- `npm run check:i18n && npm run check:commands && npm run check:panel-refs` -- expected: xanh, không khoá mồ côi
- Chạy đủ bộ một lần nếu `lib.rs`/đăng ký lệnh bị chạm hoặc đỏ có dáng phụ thuộc thứ tự (AGENTS.md); `pre-push` chạy đủ khi đẩy

**Manual checks (if no CLI):**
- App thật: gõ `chi`, `chỉ`, `分久必合`; đọc dòng trạng thái và ghi chú (mục nợ `Chủ: Epic 5`, Ice chạy trước khi Epic 5 `done`)
