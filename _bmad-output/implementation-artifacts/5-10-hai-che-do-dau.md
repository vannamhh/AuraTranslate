---
title: 'Story 5.10: Hai chế độ dấu'
type: 'feature'
created: '2026-08-29'
status: 'done'
baseline_revision: 'a8a2f9c70f2b29aa27668617e3a0a4c26e45eccb'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
warnings: ['oversized']
deferred:
  - summary: >-
      Bàn đo e2e không dựng được `target_text`, nên một hit KHOAN DUNG thật ở nửa bản dịch chưa
      bao giờ đi qua WKWebView — chỉ qua fixture Rust và một `invoke` giả.
    evidence: |-
      `create_work_from_text` (đường tạo Tác phẩm DUY NHẤT mà e2e với tới được) chỉ ghi
      `source_text`; không bước dịch tự động nào dựng `target_text` trong một bàn đo không lái
      Editor thật. Vì khoan dung CHỈ đụng nửa bản dịch, hai ca e2e của story đo được đường tự
      nới và mệnh đề "khoan dung không làm mất hit nửa nguyên văn", nhưng KHÔNG đo được cú bấm
      "Khoan dung dấu" trả về một hit nửa bản dịch mang nhãn phân biệt được. §GIỚI HẠN của chính
      spec e2e ghi ra điều này.
    location: 'e2e/specs/story-5-10-diacritic-modes.e2e.mjs — §GIỚI HẠN'
    severity: medium
  - summary: >-
      Ở lượt khoan dung, trần `limit` có thể đẩy một hit vốn THẤY ĐƯỢC ở chế độ chính xác ra
      ngoài cửa sổ hiển thị.
    evidence: |-
      `search_target_text` và `search_target_text_nd` cùng `ORDER BY s.work_id, s.chapter_ord,
      s.segment_ord` và cùng cắt ở `limit`; tập `_nd` là tập CHA nên các hàng chỉ-khoan-dung xen
      vào giữa có thể đẩy một hàng khớp chính xác qua khỏi ngưỡng. `SearchReport::truncated` VẪN
      bật nên không phải một lượt cắt im lặng. ⚠️ Phép DÁN NHÃN thì KHÔNG bị ảnh hưởng: tập chính
      xác là một dãy CON của tập `_nd` dưới cùng một thứ tự, nên một hit nằm trong `limit` đầu của
      `_nd` luôn nằm trong `limit` đầu của tập chính xác — trừ khi khoá sắp có hàng TRÙNG, lúc đó
      thứ tự giữa hai truy vấn là KHÔNG XÁC ĐỊNH. Ca trùng khoá chưa được đo.
    location: 'src-tauri/src/core/library/indexer.rs — Indexer::search, nhánh Lenient'
    severity: low
  - summary: >-
      Đường khoan dung/tự nới chưa có một số đo hiệu năng nào — bàn đo p95 vẫn chỉ chạy
      `SearchMode::Exact`.
    evidence: |-
      `bench_p95_of_a_library_search_over_five_thousand_chapters` gọi `indexer.search(...,
      SearchMode::Exact)` và chỉ thế. Một lượt khoan dung chạy thêm một truy vấn FTS5 THỨ BA cộng
      một `HashSet` dựng mỗi lượt; chi phí đó chưa ai đo, kể cả sơ bộ. NFR3 vốn đã hoãn tới Story
      6.18 (cần FR14 để có 5.000 Chương thật), nên phép đo của nhánh mới đi cùng chủ đó.
    location: 'src-tauri/tests/library_index_contract.rs — bench p95 (#[ignore])'
    severity: low
---

<intent-contract>

## Intent

**Problem:** FR9 hứa người dịch gõ không dấu vẫn tìm ra thứ mình cần, nhưng hôm nay
`library-index.db` chỉ có **hai** chỉ mục và cả hai đều PHÂN BIỆT dấu (`library_target_fts`
`unicode61 remove_diacritics 0`, `library_source_fts` `trigram` — mặc định phân biệt dấu, đo lại
2026-08-29 bên dưới). Không chỉ mục `_nd` nào tồn tại, không tham số `mode` nào trên dây, và
`mode.library.search_no_match` (`vi.json:254`) đang nói thẳng ra rằng *"Chế độ khoan dung dấu chưa
có ở bản này"* — một câu sẽ HẾT ĐÚNG đúng lúc story này chạy.

**Approach:** Thêm **một** chỉ mục phụ `library_target_fts_nd`
(`unicode61 remove_diacritics 2`, hậu tố `_nd` đúng khuôn
`tools/dict-build/src/schema.rs::SENSE_FTS_ND_DDL`) trên chính cột `library_segment.target_text`
đã có — không cột mới, không đường ghi mới (AD-8 giữ nguyên). `Indexer::search` nhận thêm một
`SearchMode` (`exact` mặc định · `lenient`): chạy chính xác TRƯỚC, tự nới sang khoan dung **chỉ
khi** lượt chính xác trả 0 hàng trên một chỉ mục KHÔNG rỗng, và báo ra rằng nó đã nới; người dùng
cũng chọn thẳng chế độ khoan dung được bằng hai nút. Mỗi hit tự khai `match_kind` (`exact` /
`lenient`) nên hai loại kết quả phân biệt được trên màn hình.

## Boundaries & Constraints

**Always:**
- 🔴 **`remove_diacritics` phải là `2`, KHÔNG phải `1`, và con số đó bị ÉP bởi phép đo.** Đo
  2026-08-29 trên SQLite **3.53.2** (bản NHÚNG thật, xem §Design Notes): kho hai hàng
  `Nguyễn Huệ đại phá` / `nguyen hue dai pha`, truy vấn `"nguyen hue"` cho `remove_diacritics 1`
  ⇒ **1** hàng (trượt hàng có dấu), `remove_diacritics 2` ⇒ **2** hàng. Mức `1` không gỡ được dấu
  khỏi các ký tự tiếng Việt hai dấu (`ễ`, `ệ`, `ắ`, `ộ`…) — tức một chỉ mục "khoan dung" **im
  lặng khoan dung một nửa**. `tools/dict-build` đã chọn `2` cho `sense_fts_nd` vì đúng lý do này.
- 🔴 **Chính xác chạy TRƯỚC, mỗi lượt, không ngoại lệ** (AD-27 · AC1). Nới chỉ xảy ra khi
  `hits` RỖNG **và** `indexed_segments > 0` — nới trên một chỉ mục rỗng làm màn hình khai *"đã
  nới sang khoan dung"* cho một kho chưa có dòng nào, tức một câu đúng hình dạng và sai sự thật.
- 🔴 **Khoan dung KHÔNG BAO GIỜ là mặc định** (AD-27 · AC4): `SearchMode::default() == Exact`, và
  `mode = None` trên dây ⇒ `Exact`. Không một đường nào để một lượt gọi rơi vào `Lenient` mà
  không có người dùng bấm hoặc một lượt nới đã được BÁO RA.
- 🔴 **Nửa NGUYÊN VĂN vẫn chạy CHÍNH XÁC ngay cả trong lượt khoan dung — chuyển chế độ không được
  LÀM MẤT kết quả.** Khoan dung chỉ đổi nửa bản dịch; nếu nhánh `trigram` ngừng chạy khi bật
  khoan dung thì một người dùng bật nó lên sẽ thấy danh sách NGẮN ĐI, đúng lớp "rỗng im lặng do
  chính hàng rào chống rỗng im lặng sinh ra" mà AD-44 đã ghi tên.
- 🔴 **Nhãn `match_kind` của một hit phải đến từ một PHÉP ĐO CÙNG VỊ TỪ, không từ một phép so
  chuỗi thứ hai.** Trong lượt khoan dung do người dùng chọn, tập rowid của nhánh chính xác được
  lấy về và dùng làm tập thành viên: hit của `_nd` nằm trong tập đó ⇒ `exact`, ngoài ⇒ `lenient`.
  Một phép `contains()` trên văn bản thô sẽ nói KHÁC vị từ `unicode61` (khớp TRỌN TỪ) đang dùng —
  `ma` là chuỗi con của `mama` nhưng không phải một token của nó — nên nó sẽ dán nhãn sai.
  Trong lượt **tự nới**, phép so đó không cần chạy: nhánh chính xác vừa trả 0 hàng, nên mọi hit
  của `_nd` là `lenient` theo cấu tạo.
- 🔴 **`widened` và `effective_mode` là hai trường TƯỜNG MINH từ Rust** (AD-1, cùng lý lẽ `total`
  của Story 5.9), và một ca hợp đồng khoá bất biến
  `widened == (mode == exact && effective_mode == lenient)`.
- **`mode` trên dây là danh mục ĐÓNG hai giá trị, chép khuôn `WorkSortKey::from_wire`**: một
  chuỗi lạ ⇒ `IpcError` `err.library.unknown_search_mode` `{mode}`, **không** im lặng rơi về mặc
  định. Đây là `MessageKey` MỚI **duy nhất** của story.
- **Cả hai nút chế độ chạy lại lượt tìm nếu đã có truy vấn** — một danh sách kết quả CŨ nằm dưới
  một nhãn chế độ MỚI là chính lỗi mà `librarySearchStatus` đã phải sửa một lần ở vòng rà 5.9
  (`busy` thắng trước).
- **Bump `LIBRARY_INDEX_MIGRATIONS` `to_version` 6 → 7 bằng cách VIẾT LẠI `LIBRARY_WORK_DDL` TẠI
  CHỖ** — kho dẫn xuất không di trú (AD-8), không một bước di trú thứ hai.
- **Khuôn hai lớp cho bề mặt IPC** giữ nguyên: `search_library` là hàm thuần nhận
  `Option<&Indexer>`, vỏ `library_search` `(async)` trong `mod wire` lấy `try_state`.

**Block If:**
- Một AC chỉ nghiệm thu được bằng cách đổi một bất biến đã ADOPTED (AD-8 · AD-27 · AD-30) ⇒ HALT
  `blocked`: đó là một `AD` mới, và AD mới không do dev soạn.

**Never:**
- **Không** đụng `library_source_fts` và **không** dựng `library_source_fts_nd` ở story này — xem
  §Design Notes "Vì sao nửa nguyên văn không có bản khoan dung": nhánh `trigram` bắt buộc đi qua
  một bước xác minh chuỗi con Ở RUST (🔴 của Story 5.9), và không phép xác minh nào chạy được
  trên một phép gấp dấu mà Rust không tự cài. Món nợ có chủ, không một nhánh nửa vời.
- **Không** đúc một hàm gấp dấu (`đ→d`, bảng dấu tiếng Việt viết tay) trong Rust ở story này —
  đó là một CƠ CHẾ, nó đụng đúng món nợ chuẩn hoá Unicode NFC/NFD đang mở (chủ Ice), và giới hạn
  `đ` đo được bên dưới đi vào sổ nợ chứ không vào mã.
- **Không** thêm phụ thuộc mới (cửa NFR15 không mở cho gói nào ở story này).
- **Không** đổi ngưỡng NFR3 và không khai NFR3 đạt — vẫn là Story 6.18.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Chính xác vẫn thắng | `target_text = 'má của tôi'` cùng năm hàng `ma`/`mà`/`mả`/`mã`/`mạ`; truy vấn `má`, `mode = exact` | **đúng 1** hit, `match_kind = "exact"`, `effective_mode = "exact"`, `widened = false` | No error expected |
| Tự nới khi không khớp | cùng kho trên; truy vấn `ma cua` (cụm hai token, **0** hàng khớp CHÍNH XÁC vì hàng thật là `má của tôi rất hiền`), `mode = exact` | nhánh `_nd` chạy ⇒ **1** hit, `match_kind = "lenient"`, `effective_mode = "lenient"`, `widened = true` | No error expected |
| Tự nới, ký tự hai dấu | hàng `Nguyễn Huệ đại phá quân Thanh`; truy vấn `nguyen hue`, `mode = exact` | exact 0 ⇒ nới ⇒ **1** hit `lenient` (đây là ca `remove_diacritics 1` sẽ TRƯỢT) | No error expected |
| Không nới khi chính xác CÓ kết quả | truy vấn khớp 1 hàng chính xác | `_nd` **không** chạy; `widened = false`; `effective_mode = "exact"` | No error expected |
| Không nới trên chỉ mục rỗng | `library_segment` 0 hàng; truy vấn bất kỳ, `mode = exact` | `hits = []`, `indexed_segments = 0`, `widened = **false**`, `effective_mode = "exact"` | No error expected |
| Người dùng chọn khoan dung, có cả hai loại | hai hàng: `khoáng sản` và `khoang trong`; truy vấn `khoang`, `mode = lenient` | **2** hit — hàng `khoang trong` mang `match_kind = "exact"`, hàng `khoáng sản` mang `"lenient"`; `widened = false` | No error expected |
| Khoan dung KHÔNG làm mất nửa nguyên văn | một segment khớp ở `source_text` (chữ Hán/Latin), `mode = lenient` | hit đó VẪN có mặt, `field = "source"`, `match_kind = "exact"` | No error expected |
| Nới vẫn 0 | truy vấn vô nghĩa, `indexed_segments > 0`, `mode = exact` | `hits = []`, `widened = true`, `effective_mode = "lenient"` ⇒ giao diện nói *đã thử cả hai chế độ* | No error expected |
| Chế độ lạ trên dây | `mode = "fuzzy"` | **0** truy vấn SQL | `err.library.unknown_search_mode` `{mode}`, `retryable = false` |
| `mode` vắng mặt | `mode = null` | chạy như `exact` — mặc định không bao giờ là khoan dung | No error expected |
| Trần cắt ở nhánh `_nd` | nhánh khoan dung khớp hơn `limit` hàng | `truncated = true`, cùng phép `limit + 1` của Story 5.9 | No error expected |
| Truy vấn dưới 3 ký tự, khoan dung | truy vấn `má` (2 ký tự), `mode = lenient` | nửa nguyên văn vẫn câm (`short_query = true`); nhánh `_nd` (unicode61, không có sàn) VẪN chạy | No error expected |
| Ký tự cú pháp FTS5 ở nhánh `_nd` | truy vấn `state-of-the-art`, `a"b`, `NEAR`, `*`, `mode = lenient` | chạy bình thường trên cả ba nhánh, **0** lỗi SQLite (tái dùng `fts_phrase`) | No error expected |
| Đổi chế độ khi đã có kết quả | danh sách đang hiện, người dùng bấm "Khoan dung" | lượt tìm chạy LẠI với truy vấn hiện thời — danh sách cũ không bao giờ nằm dưới nhãn chế độ mới | No error expected |
| Đổi chế độ khi ô tìm rỗng | ô tìm rỗng, người dùng bấm "Khoan dung" | **0 lượt IPC**; chỉ cờ chế độ đổi | No error expected |
| Xoá chỉ mục rồi quét lại | người dùng xoá `library-index.db` (đang ở `to_version` 6) | `Indexer::open` xoá + dựng lại ở **7**; kết quả cả hai chế độ giống hệt một kho vừa dựng | No error expected |
| Đoạn trích của hit khoan dung | hit `lenient` | `snippet` mang chữ GỐC còn nguyên dấu (`_nd` lập chỉ mục trên chính cột `target_text`, không trên một bản chép đã gấp) | No error expected |

</intent-contract>

## Code Map

**Rust — lược đồ**

- `src-tauri/src/core/store/schema.rs:1717-1752` — `LIBRARY_WORK_DDL`; hai `CREATE VIRTUAL TABLE`
  hôm nay ở `:1747-1752`. 🔴 Thêm `library_target_fts_nd` **ngay sau** chúng và bump
  `LIBRARY_INDEX_MIGRATIONS` (`:1790-1793`) `to_version` 6 → 7. Khối doc-comment `:1686-1716` là
  khuôn ghi lý do từng lần bump — chép đúng hình dạng "🔵 **NÂNG LẦN SÁU (…)**".
  ⚠️ `:1711` và `:1714` mang chuỗi `3.43.2` và mệnh đề *"tham số `remove_diacritics` của trigram
  có từ SQLite 3.45"* — cả hai phải sửa TẠI CHỖ (xem §Design Notes "Số đo mang tên sai động cơ").
- `tools/dict-build/src/schema.rs:182-188` — `SENSE_FTS_ND_DDL`, khuôn đặt tên `_nd` và lý do
  chọn `remove_diacritics 2`. Chép **hình dạng**, không import (hai workspace tách rời).

**Rust — Indexer**

- `src-tauri/src/core/library/indexer.rs:754-805` — `Indexer::search`: nơi nhận thêm tham số
  `mode`, chạy nhánh chính xác trước, rồi quyết định nới. `:782-792` là khuôn `limit + 1` →
  `truncate(limit)` cho `truncated`.
- `:932-958` — `search_target_text` (`unicode61`, `snippet(..., 10)`), **khuôn trực tiếp** cho
  `search_target_text_nd`; chỉ đổi tên bảng FTS. `:983-1021` — `search_source_text` (`trigram` +
  `verify_substring`), story này **không đụng**.
- `:854-856` — `fts_phrase`, dùng lại nguyên; `:829` `MIN_SUBSTRING_QUERY_CHARS`; `:833`
  `DEFAULT_SEARCH_LIMIT`; `:837` `MAX_SEARCH_LIMIT`.
- `:858-897` — `SearchField` + `SearchHit`: `SearchHit` nhận thêm `match_kind`. Để dán nhãn theo
  tập thành viên, nhánh target cần đọc thêm `s.rowid` (không lên dây).
- `:899-927` — `SearchReport`: nhận thêm `mode` · `effective_mode` · `widened`.
- `:219-505` — `rebuild`: thêm đúng một câu
  `INSERT INTO library_target_fts_nd(library_target_fts_nd) VALUES('rebuild')` cạnh hai câu đã có.
  ⚠️ `:748` mang chuỗi `3.43.2` (cùng lỗi trên).

**Rust — bề mặt IPC**

- `src-tauri/src/commands/library.rs:415-444` (`SearchHit` dây) · `:446-478` (`SearchReport` dây) ·
  `:487-496` (hàm thuần `search_library`) · `:622-639` (vỏ `library_search`, `(async)`).
- `:334-339` — `unknown_sort()`: **khuôn trực tiếp** cho `unknown_search_mode()`.
- `src-tauri/src/core/i18n/mod.rs:432` — `LibraryUnknownSort => "err.library.unknown_sort" ["sort"]`,
  khuôn cho khoá mới; khối chú thích `:420-427` khai danh mục ĐÓNG của cụm tìm kiếm — sửa tại chỗ
  để nói ra khoá thứ hai và **vì sao** các trạng thái chế độ vẫn KHÔNG phải lỗi.
- `src-tauri/src/lib.rs:324+` — `generate_handler!`; vỏ đã có tên, không thêm vỏ mới.

**Frontend**

- `src/config/library.ts:466-583` — cụm Story 5.9: `SearchField`/`SearchHit`/`SearchReport`,
  `isSearchHit` (`:519`, kiểm MỌI trường), `isSearchHitArray` (`:542`, kiểm MỌI phần tử),
  `isSearchReport` (`:546`), `searchLibrary` (`:564`, gửi `{ query, limit }`).
- `src/modes/librarySearch.ts` (258 dòng) — state cấp module `:46-59`, export `:64-77`,
  `librarySearchStatus` hàm thuần `:104-125` (🔴 thứ tự nhánh LÀ hợp đồng), `runLibrarySearch`
  `:145-198`, `openCurrentLibrarySearchHit` `:224-240`, `resetLibrarySearch` `:245-258`.
  🔴 `check:panel-refs`: mỗi khai báo state **trên một dòng**, và mọi ô nhớ mới phải có mặt trong
  `resetLibrarySearch`.
- `src/modes/LibraryMode.vue:454-561` — khối `.search-block`: form `:456-476`, `role="status"` tám
  nhánh sau story này `:485-501`, `grid-nav` con trỏ `:503-529`, danh sách hit `:532-560`.
  `<style scoped>` `.search-*` ở `:1705-1766`. Khuôn nút nhóm-lựa-chọn: bốn nút lọc trạng thái
  `:590-636` (`library.filter_*`) — mỗi lựa chọn một `dispatch` id RIÊNG, không một nút "đảo".
- `src/commands/index.ts:1232-1281` — khối Story 5.9 (`library.search`, `Mod+Alt+F`;
  `search_next`/`search_prev`/`open_search_hit` không phím); `CommandDeps` cụm Library `:328-339`.
- `src/main.ts:100-106` (import) · `:406-409` (tiêm dep) — một khối một Story.
- `src/i18n/vi.json:89-92` (`command.library.search*`) · `:247-263` (`mode.library.search_*`).
  🔴 `:254` `search_no_match` khai *"Chế độ khoan dung dấu chưa có ở bản này"* — mệnh đề HẾT ĐÚNG
  ở story này, sửa tại chỗ.

**Cổng và bàn đo**

- `src-tauri/tests/library_index_contract.rs` — tệp nhận bộ ca chính; ca
  `an_index_file_at_schema_version_4_is_deleted_and_rebuilt_at_version_6_with_the_new_columns`
  (`:362`) là **khuôn trực tiếp** cho ca 6 → 7.
- `src-tauri/tests/ipc_contract.rs:540-591` — đóng băng khoá `snake_case` của
  `SearchHit`/`SearchReport`; `:761-790` — `the_library_search_wire_is_registered_and_keeps_its_parameter_names`
  kiểm `["query: String", "limit: Option<u32>"]` ⇒ **thêm** `mode: Option<String>`.
- `src-tauri/tests/config_invariants.rs:1046-1048` — số `#[tauri::command(async)]` của
  `commands/library.rs` phải ĐÚNG **4**; story này không thêm vỏ ⇒ con số **không đổi**, và nó đỏ
  tức lượt cài đã đi sai đường.
- `src-tauri/tests/library_index_boundary.rs` — danh sách **đúng hai** tệp được nhắc kho này;
  story này không làm nó dài ra.
- `tests/frontend/librarySearch.test.ts` (480 dòng) — khuôn `invoke` giả và bộ ca trạng thái.
- `e2e/specs/story-5-9-library-search.e2e.mjs` (281 dòng) — helper, selector
  `[data-library-search-*]`, `realClick`. 🔴 §GIỚI HẠN mục 2: `create_work_from_text` **chỉ** dựng
  `source_text` — không `target_text` nào tồn tại trong một bàn đo e2e, nên nửa bản dịch (nơi
  khoan dung sống) **không** đo được trực tiếp ở e2e; xem §Tasks 13 cho hai mệnh đề e2e đo được.
  ⚠️ `story-5-6-library-grid.e2e.mjs` đỏ từ baseline (chủ Story 5.6) — không đọc thành hồi quy.
- `scripts/check-commands.mjs` — `COMMAND_FLOOR` 52 · `CLICK_FLOOR` 27 · `DISPATCH_FLOOR` 40, đều
  là cận DƯỚI (thêm lệnh không làm đỏ). `scripts/check-i18n.mjs` Kiểm A cấm chữ tiếng Việt CÓ DẤU
  ở vị trí mã trong `src-tauri/src/**`.
- `_bmad-output/implementation-artifacts/deferred-work.md:8551-8562` — mục 🟡 *"Chế độ khoan dung
  dấu … chưa có cửa bấm"*, **chủ Story 5.10**: mục này đóng ở đây.

## Tasks & Acceptance

**Execution:**

1. [x] `src-tauri/src/core/store/schema.rs` — viết lại `LIBRARY_WORK_DDL` TẠI CHỖ: thêm
   `library_target_fts_nd(target_text, content='library_segment', content_rowid='rowid',
   tokenize="unicode61 remove_diacritics 2")`; bump `LIBRARY_INDEX_MIGRATIONS` `to_version` 6 → 7
   kèm khối 🔵 "NÂNG LẦN SÁU" đúng khuôn năm lần trước; và **sửa tại chỗ** hai mệnh đề sai ở
   `:1711`/`:1714` (động cơ thật là **3.53.2**, và `trigram remove_diacritics` **có** ở bản đó).
   -- Rationale: kho dẫn xuất không di trú; một số đo mang tên sai động cơ không phải một số đo.
2. [x] `src-tauri/src/core/library/indexer.rs` — ① `SearchMode` (`Exact`/`Lenient`, `as_str`,
   `from_wire`, `Default = Exact`) đúng khuôn `WorkSortKey`; ② `search_target_text_nd` chép khuôn
   `search_target_text`, đọc thêm `s.rowid`; ③ `Indexer::search(query, limit, mode)` chạy chính
   xác trước rồi nới theo §Always; ④ `SearchHit::match_kind`, `SearchReport::{mode,
   effective_mode, widened}`; ⑤ một câu `INSERT INTO library_target_fts_nd(...) VALUES('rebuild')`
   trong `rebuild`; ⑥ sửa chuỗi `3.43.2` ở `:748` kèm 🔵 + ngày.
   -- Rationale: một bộ điều phối chọn một nhánh trả 0 hàng trên kho CÓ dữ liệu; nhãn hit phải
   đến từ cùng vị từ đã tìm.
3. [x] `src-tauri/src/core/i18n/mod.rs` — `LibraryUnknownSearchMode => "err.library.unknown_search_mode" ["mode"]`,
   và sửa tại chỗ khối chú thích cụm tìm kiếm để khai danh mục ĐÓNG nay có **hai** khoá.
   -- Rationale: một giá trị lạ trên dây phải là `IpcError`, không một lượt rơi về mặc định im lặng.
4. [x] `src-tauri/src/commands/library.rs` — `unknown_search_mode()` chép khuôn `unknown_sort()`;
   `search_library(indexer, query, limit, mode: Option<&str>)`; `SearchHit`/`SearchReport` dây
   nhận bốn trường mới; vỏ `library_search` nhận `mode: Option<String>`.
   -- Rationale: khuôn hai lớp; bốn tên trường là **dây**.
5. [x] `src/config/library.ts` — `SearchMode`/`MatchKind` union đóng; `SearchHit.match_kind`,
   `SearchReport.{mode, effective_mode, widened}`; type guard LÚC CHẠY cho cả bốn; `searchLibrary(query, limit, mode)`
   gửi `{ query, limit, mode }`. -- Rationale: `IpcError` phía TS là một lời khai, không một bảo đảm.
6. [x] `src/modes/librarySearch.ts` — ô nhớ `mode` (mặc định `'exact'`), `effectiveMode`, `widened`
   (mỗi khai báo một dòng, đều vào `resetLibrarySearch`); `setLibrarySearchModeExact()` /
   `setLibrarySearchModeLenient()` đặt cờ rồi chạy lại `runLibrarySearch()` **chỉ khi** truy vấn
   không rỗng; mở rộng `librarySearchStatus` thành tám giá trị (`result_widened` và
   `no_match_widened` tách khỏi `result`/`no_match`). -- Rationale: một danh sách cũ dưới một nhãn
   chế độ mới là một câu khẳng định sai; `check:panel-refs` đòi mọi ô nhớ có đường reset.
7. [x] `src/modes/LibraryMode.vue` — hai nút chế độ (`@click="dispatch('library.search_mode_exact')"`
   / `…_lenient`) với `aria-pressed` theo `librarySearchMode`; `role="status"` mở rộng đủ tám
   nhánh; mỗi hàng hit `match_kind === 'lenient'` mang một nhãn phân biệt được; `<style scoped>`
   chỉ dùng token. -- Rationale: AD-34 §1 (`@click` là đúng một `dispatch`); AC5 + AC6.
8. [x] `src/commands/index.ts` + `src/main.ts` — hai lệnh `library.search_mode_exact` /
   `library.search_mode_lenient` (không phím mặc định, cùng chủ ý `library.filter_*`), hai
   `CommandDeps` mới tiêm ở khối Library. -- Rationale: nút và phím tắt phát CÙNG một `dispatch`.
9. [x] `src/i18n/vi.json` — `err.library.unknown_search_mode`; `command.library.search_mode_exact` ·
   `…_lenient`; `mode.library.search_mode_heading` · `search_mode_exact` · `search_mode_lenient` ·
   `search_result_widened` · `search_no_match_widened` · `search_hit_lenient`; và **sửa tại chỗ**
   `mode.library.search_no_match` (câu "chưa có ở bản này" đã hết đúng). Phẳng, không giá trị
   rỗng, placeholder khớp `[a-z_][a-z0-9_]*`. -- Rationale: NFR16; `check:i18n` đồng bộ hai chiều.
10. [x] `src-tauri/tests/library_index_contract.rs` — bộ ca cho **mọi hàng** §I/O Matrix thuộc tầng
    Rust, cộng ca bump 6 → 7 và ca bất biến
    `widened == (mode == exact && effective_mode == lenient)`. Tên hàm là **câu khẳng định**.
    -- Rationale: hợp đồng và bất biến kho thuộc đường Rust.
11. [x] `src-tauri/tests/ipc_contract.rs` — khoá `snake_case` của bốn trường mới; thêm
    `"mode: Option<String>"` vào danh sách tham số dây
    (`the_library_search_wire_is_registered_and_keeps_its_parameter_names`, `:783`). Cùng lượt:
    `src-tauri/tests/library_commands_contract.rs:277` gọi `search_library(None, …, None)` với
    **ba** tham số — chữ ký đổi thành bốn ⇒ tệp đó KHÔNG biên dịch nếu quên, và ca
    `library.indexer_missing` ở `:278` phải giữ nguyên phán quyết. -- Rationale: đổi tên tham số
    là đổi DÂY; một tệp test không biên dịch là một cổng câm.
12. [x] `tests/frontend/librarySearch.test.ts` — ca cho: tám trạng thái phân biệt được; đổi chế độ
    khi ô tìm rỗng KHÔNG phát IPC; đổi chế độ khi đã có truy vấn CÓ chạy lại và `mode` gửi đi
    đúng; hàng `lenient` mang nhãn còn hàng `exact` thì không.
    -- Rationale: hành vi module thuần và `.vue` thuộc vitest.
13. [x] `e2e/specs/story-5-10-diacritic-modes.e2e.mjs` — trong WKWebView thật, HAI mệnh đề mà ba
    đường kia không đo được: ① một truy vấn không khớp gì ⇒ trạng thái *đã thử cả hai chế độ*
    (đường tự nới chạy trọn qua IPC thật); ② bật khoan dung rồi tìm một chuỗi ở nửa NGUYÊN VĂN ⇒
    hit VẪN còn (chuyển chế độ không làm mất nửa nguyên văn). §GIỚI HẠN phải ghi rõ: nửa bản dịch
    không dựng được trong bàn đo e2e (`create_work_from_text` chỉ tạo `source_text`).
    -- Rationale: hành vi trong engine thật không có chủ ở ba đường kia; một giới hạn không viết
    ra là một giới hạn người sau tưởng đã được xét.
14. [x] `_bmad-output/implementation-artifacts/deferred-work.md` — ① đóng mục 🟡 *"Chế độ khoan dung
    dấu"* bằng `→ ✅ ĐÃ ĐÓNG 2026-08-29 (Story 5.10)` kèm cách đóng; ② mục nợ MỚI **có chủ Ice**:
    `đ`/`Đ` (U+0111/U+0110) **không** được `remove_diacritics` gấp về `d` ở BẤT KỲ mức nào —
    `duong` không tìm ra `đường` (số đo ở §Design Notes), đóng nó cần một hàm gấp dấu trong Rust,
    tức cùng cơ chế với món nợ NFC/NFD đang mở; ③ mục nợ MỚI **có chủ Ice**: nửa nguyên văn không
    có bản khoan dung vì nhánh `trigram` cần một phép xác minh chuỗi con mà Rust chưa gấp dấu được.
    -- Rationale: không mục nào mồ côi; không đánh dấu đạt bằng suy luận.
15. [x] `src-tauri/AGENTS.md` — ghi mệnh đề mới: `library-index.db` nay có **ba** chỉ mục FTS, và chỉ
    mục `_nd` **không bao giờ** chạy một mình (AD-27). -- Rationale: một bất biến chỉ sống nếu nó
    được viết ở chỗ agent nạp mỗi phiên.

**Acceptance Criteria:**

- Given toàn bộ cây nguồn, when rà, then **0** đường nào chạy `library_target_fts_nd` mà không có
  một lượt chính xác chạy TRƯỚC trong cùng lời gọi — và `library_index_boundary.rs` vẫn khai
  **đúng hai** tệp được nhắc kho này.
- Given một `library-index.db` ở `to_version` 6, when ứng dụng mở, then tệp bị xoá và dựng lại ở
  **7**, và lượt quét kế tiếp cho kết quả tìm kiếm **giống hệt** cả ở chế độ chính xác lẫn khoan
  dung — không một byte dữ liệu người dùng nào nằm ngoài các `.atproj`.
- Given bảy ca trạng thái (chưa gõ · đang tìm · chỉ mục trống · dưới 3 ký tự · không khớp · không
  khớp sau khi đã nới · có kết quả sau khi đã nới), when hiển thị, then **bảy câu khác nhau**, và
  hai ca "đã nới" nói ra rằng hệ thống đã tự chuyển chế độ.
- Given một lượt tìm bất kỳ, when hoàn tất, then người dùng đọc được chế độ đang có hiệu lực trên
  màn hình mà không phải suy ra từ số kết quả.
- Given ba đối chứng ĐỎ bắt buộc — ① đổi `remove_diacritics 2` thành `1`; ② cho nhánh `_nd` chạy
  cả khi lượt chính xác CÓ kết quả; ③ ngừng chạy nửa nguyên văn trong lượt khoan dung — when mỗi
  cái là một phép **GỠ/DỜI thật** rồi chạy bộ test CŨ, then mỗi cái làm ĐỎ ít nhất một ca có tên,
  và kết quả từng cái được ghi lại.
- Given `pre-push` và CI, when cả hai xanh, then vẫn phải ĐỌC lượt CI trước khi kết luận — nửa
  Windows không có đường nghiệm thu tại chỗ.

## Spec Change Log

- 🔵 **2026-08-29 — Task 6/AC3 "bảy giá trị" đã SỬA thành "tám" tại chỗ trong mã, không sửa
  spec này (đúng luật "năng lực chưa dựng ≠ lệch spec", nhưng chiều ngược: đây là spec đếm sai
  trên một tiền đề cũ, không phải mã lệch đích).** `LibrarySearchStatus` TRƯỚC story này thật sự
  có SÁU giá trị (`not_typed`/`searching`/`index_empty`/`short_query`/`no_match`/`result`, đếm
  trực tiếp trên khai báo `src/modes/librarySearch.ts:94` tại baseline `a8a2f9c`) — không NĂM
  như một chú thích cũ (Story 5.9) tự xưng. Task 6 viết "mở rộng thành bảy giá trị" bằng cách
  cộng `5 + 2`, kế thừa tiền đề sai đó. Cộng đúng hai giá trị mới (`no_match_widened`/
  `result_widened`, tách khỏi `no_match`/`result` khi tự nới) vào SÁU giá trị THẬT ⇒ **tám**,
  không bảy. AC3 ("bảy ca trạng thái") vẫn ĐÚNG như một danh sách bảy KỊCH BẢN DEMO — nó liệt
  kê bảy ca cụ thể cần phân biệt được, không tuyên bố "type có đúng bảy giá trị"; ca thứ tám
  (`result` — có kết quả, KHÔNG nới) đã có từ Story 5.9 và không đổi ở đây nên không cần liệt
  lại. ⇒ Implement giữ nguyên Ý ĐỊNH của AC3 (tám trạng thái phân biệt được, gồm cả bảy ca được
  nêu tên), sửa chỉ con SỐ đã tính sai trong Task 6 — ghi tại `src/modes/librarySearch.ts` khối
  🔵 đầu tệp, không sửa `epics.md`/`prd.md`. Đối chứng: `grep -c "'" src/modes/librarySearch.ts`
  trên khai báo `LibrarySearchStatus` mới cho tám chuỗi; `tests/frontend/librarySearch.test.ts`
  có ca cho cả tám tổ hợp.

## Review Triage Log

### 2026-08-30 — Review pass

- intent_gap: 0
- bad_spec: 0
- patch: 10: (high 1, medium 4, low 5)
- defer: 3: (high 0, medium 1, low 2)
- reject: 5: (high 0, medium 2, low 3)
- addressed_findings:
  - `[high]` `[patch]` **`result_widened` che mất lời cảnh báo trần cắt.** Nhánh `result_widened` được xét TRƯỚC `librarySearchTruncated`, nên một lượt vừa tự nới VỪA bị `limit` cắt hiện "{total} kết quả — đã tự chuyển sang khoan dung dấu" mà không một chữ nào nói còn nữa — đúng lớp lỗi mà vòng rà Story 5.9 đã vá một lần bằng `SearchReport::truncated`, nay mọc lại ở một nhánh mới. Khối chú thích ⚠️ nhận nó là "không đáng một câu riêng ở vòng này" đã bị GỠ: một giới hạn tự khai không miễn trừ được luật *"không trần nào được cắt trong im lặng"*. Thêm khoá `mode.library.search_result_widened_truncated`, rẽ theo `truncated` BÊN TRONG nhánh widened, ba ca mount DOM pin lại.
  - `[medium]` `[patch]` **`impl From<CoreSearchHit> for SearchHit` chưa có ca nào chạy qua với một hit THẬT.** `match_kind` trên struct dây chỉ được kiểm bằng một literal DỰNG TAY ở `ipc_contract.rs:557`, tức phép chuyển đổi bị bỏ qua hẳn; mọi ca của `library_commands_contract.rs` gọi `search_library` đều dùng fixture trả 0 hàng. Hardcode `MatchKind::Exact.as_str()` trong `From` thì nhãn khoan dung không bao giờ hiện nữa và không cổng nào đỏ. Thêm ca dựng dữ liệu có hit, khẳng định `match_kind` TRÊN struct dây; đối chứng ĐỎ đã chạy thật (`"exact" != "lenient"`) rồi hoàn nguyên.
  - `[medium]` `[patch]` **`setLibrarySearchModeExact` chưa từng được GỌI.** Nó xuất hiện đúng hai lần trong bộ test và cả hai là VĂN BẢN — một chú thích và một tiêu đề `describe` — trong khi hai ca bên trong khối chỉ gọi bản `Lenient`. Một cái tên khai thứ nó không đo. Hai hàm chỉ khác đúng literal truyền vào `setLibrarySearchMode`, nên một lỗi chép-dán làm hỏng im lặng đường về chế độ MẶC ĐỊNH mà AD-27 bắt buộc. Thêm ca đối xứng gọi thật và khẳng định `mode: 'exact'` đi trên dây.
  - `[medium]` `[patch]` **Tám nhánh `role="status"` chưa ca nào đọc ở tầng RENDER.** Hàm thuần được kiểm kỹ nhưng không ca nào mount `LibraryMode.vue` rồi đọc `[data-library-search-status]`; ca mount duy nhất rơi vào nhánh `result` thường. Một lỗi nối dây giữa khoá trạng thái (tính ĐÚNG) và câu chữ hiện ra sẽ không làm ca nào đỏ. Thêm hai ca mount cho `no_match_widened` và `result_widened`.
  - `[medium]` `[patch]` **Giới hạn `đ`/`Đ` chỉ sống trong văn xuôi, và bộ lưới NÉ đúng chữ đó.** Mọi ca khoan dung đang có tránh `đ` (truy vấn `nguyen hue` trên `Nguyễn Huệ đại phá quân Thanh`, không bao giờ `dai pha`), nên không ai biết ngày món nợ được đóng. Thêm `a_query_without_the_d_stroke_still_does_not_find_the_d_stroke_word_documented_gap` pin lại hành vi ĐANG SAI với người dùng, để ngày nợ đóng thì ca này ĐỎ và buộc người sửa đọc lại.
  - `[low]` `[patch]` §Code Map và Tasks còn viết `LibrarySearchStatus` "bảy" giá trị trong khi §Spec Change Log của chính tệp đã ghi đúng là TÁM — sửa cho khớp.
  - `[low]` `[patch]` Thiếu ca `SearchMode::Lenient` chọn tường minh trên một chỉ mục RỖNG (`an_empty_index_never_widens_…` chỉ chạy `Exact`, trong khi nhánh `Lenient` CÓ chạy truy vấn `_nd` trên kho rỗng).
  - `[low]` `[patch]` `mode.library.search_result_widened` thiếu vế "ở nửa bản dịch" mà câu song sinh `search_no_match_widened` có, dù hai câu tả cùng một cơ chế.
  - `[low]` `[patch]` Thiếu ca khứ hồi `from_wire(as_str())` cho `SearchMode`/`MatchKind` — hai chuỗi `"exact"`/`"lenient"` chép tay ở BA nơi (lõi Rust · tầng dây · union TypeScript). Thêm `ALL` cho cả hai enum và ca chạy TRÊN `ALL`, không một danh sách song song viết tay.
  - `[low]` `[patch]` Ca `resetLibrarySearch` không khẳng định ba ô nhớ MỚI (`mode`/`effectiveMode`/`widened`) — đúng ba ô mà `check:panel-refs` là lý do chúng có mặt trong hàm reset.

**Ba mục hoãn** (chi tiết + chủ ở frontmatter `deferred`): bàn đo e2e không dựng được `target_text` · trần `limit` ở lượt khoan dung đẩy một hit chính xác ra ngoài cửa sổ · nhánh khoan dung chưa có số đo hiệu năng nào.

**Năm mục bác**, hai trong số đó bác **sau khi đo** chứ không bằng phán đoán:
- *"Khoan dung dán nhãn SAI một hit chính xác thành `lenient`"* — bác: hai truy vấn dùng CHUNG `ORDER BY s.work_id, s.chapter_ord, s.segment_ord`, nên tập chính xác là một dãy CON của tập `_nd`; một hit ở vị trí `k ≤ limit` trong `_nd` có vị trí `≤ k` trong tập chính xác, tức luôn sống sót phép cắt và luôn nằm trong tập rowid dùng để dán nhãn. *(Vế CÒN LẠI của mục này — hit bị đẩy khỏi cửa sổ hiển thị — thì có thật và đã vào sổ hoãn.)*
- *"Bộ e2e chưa chạy lần nào phiên này (không có màn hình/WebDriver)"* — bác: đã chạy thật, **2 passing (42 s)**, `webkit 605.1.15 macos`, exit 0. Mệnh đề này đến từ chính báo cáo của agent cài đặt và không được kiểm; Story 5.9 đã chạy e2e trên đúng máy này hôm trước.
- *"Tự nới phải kích hoạt khi nửa BẢN DỊCH rỗng dù nửa nguyên văn có hit"* — AC viết *"chế độ chính xác **không có kết quả**"*, đọc thẳng là cả lượt tìm; và người dùng vẫn có nút bật khoan dung tay, kèm nhãn chế độ hiện hành.
- *"Thiếu phím tắt cho hai nút chế độ"* — đúng tiền lệ `library.filter_*`: cả bốn nút lọc trạng thái cũng `keys: undefined`, nút bấm và `realClick` xử lý bằng `dispatch('<id>')`.
- *"Đua giữa hai lượt đổi chế độ làm hiện kết quả cũ dưới nhãn chế độ mới"* — `busy` đã đẩy trạng thái về `searching`; danh sách cũ còn nằm dưới câu "Đang tìm…" là hành vi đã nhận từ Story 5.9, không do story này gây ra.

## Design Notes

### Số đo mang tên SAI ĐỘNG CƠ — `3.43.2` là SQLite của macOS, không phải của ứng dụng

Story 5.9 gắn mọi số đo của mình vào *"SQLite 3.43.2"*, và sáu chỗ trong cây nguồn chép lại con
số đó (`indexer.rs` ×3, `schema.rs` ×2, `e2e/specs/story-5-9-…mjs` ×1). Đo lại 2026-08-29 bằng
chính `cargo test --locked` của kho (tức đúng thư viện ứng dụng liên kết):

| | |
|---|---|
| `SELECT sqlite_version()` trong một test của `src-tauri/tests/**` | **3.53.2** |
| `sqlite3 --version` của macOS trên máy này | 3.43.2 |
| `#define SQLITE_VERSION` trong `libsqlite3-sys-0.38.1/sqlite3/sqlite3.h` | **"3.53.2"** |

`rusqlite = { version = "=0.40.1", features = ["bundled"] }` ⇒ động cơ NHÚNG, không phải động cơ
của hệ điều hành. Con số `3.43.2` là của **CLI hệ thống** — ai đó đã đo bằng `sqlite3` ở terminal
rồi gán tên đó cho một kết quả đến từ một động cơ khác. Kết quả đo của 5.9 tôi tái lập được trên
3.53.2 (trigram phân biệt dấu; `unicode61` gộp một dải Hán thành một token), nên **kết luận** của
5.9 đứng vững — thứ sai là **xuất xứ**. Một mệnh đề PHÁI SINH thì đã sai hẳn: `schema.rs:1714`
viết *"tham số `remove_diacritics` của trigram có từ SQLite 3.45"* như một lý do để không xét nó;
trên 3.53.2 tham số đó **có thật** và `CREATE VIRTUAL TABLE … tokenize="trigram remove_diacritics 1"`
chạy sạch. ⇒ Sửa cả sáu chỗ tại chỗ kèm 🔵 + ngày.

### Vì sao `remove_diacritics 2`, và vì sao `1` là một cái bẫy

Đo 2026-08-29, SQLite **3.53.2** nhúng, `cargo test --locked`. Kho mười hàng, cột đơn, cùng dữ
liệu nạp vào sáu bảng FTS5 khác tokenizer:

| Truy vấn | `unicode61 rd 0` | `unicode61 rd 1` | `unicode61 rd 2` | `trigram` | `trigram rd 1` |
|---|---|---|---|---|---|
| `nguyen hue` trên `Nguyễn Huệ đại phá` + `nguyen hue dai pha` | 1 | **1** | **2** | 1 | 2 |
| `toi an com` trên `tôi ăn cơm` + `toi an com` | 1 | 2 | 2 | 1 | 2 |
| `má` trên sáu hàng `ma/má/mà/mả/mã/mạ` | **1** | 6 | 6 | 0 | — |
| `duong phuong` trên `đường phượng bay` + `duong phuong bay` | 1 | **1** | **1** | 1 | **1** |
| `dong du` trên `đông đủ đầy đặn` + `dong du day dan` | 1 | **1** | **1** | 1 | **1** |

Hai kết luận:

- **`rd 1` khoan dung MỘT NỬA.** Nó gấp được `ô`/`ă`/`ơ` nhưng trượt `ễ`/`ệ` — ký tự hai dấu, thứ
  đầy trong tiếng Việt. Một chỉ mục "khoan dung" trượt đúng những chữ khó nhất là một chỉ mục nói
  dối: người dùng gõ `nguyen hue` và màn hình nói *"đã nới sang khoan dung"* rồi vẫn không ra.
  `rd 2` gấp cả hai lớp. Đây là cùng lý do `tools/dict-build` đã chọn `2` cho `sense_fts_nd`.
- 🔴 **`đ`/`Đ` (U+0111/U+0110) KHÔNG được gấp ở BẤT KỲ mức nào, kể cả `rd 2`, kể cả `trigram
  rd 1`.** `duong phuong` chỉ khớp hàng KHÔNG dấu; `đường phượng` chỉ khớp hàng CÓ dấu. Lý do:
  `remove_diacritics` gỡ **dấu phụ tổ hợp**, mà `đ` là một CHỮ CÁI riêng không phân rã được thành
  `d` + dấu. Hệ quả thật: một người dịch gõ `duoc`, `dau`, `di`, `duong` vẫn không tìm ra
  `được`, `đầu`, `đi`, `đường` — một phần đáng kể của FR9 vẫn hở. Đóng nó cần một **hàm gấp dấu
  trong Rust** (bảng `đ→d` cộng phần còn lại, hoặc một phép chuẩn hoá Unicode), tức đúng cơ chế
  mà món nợ NFC/NFD đang mở (chủ Ice) cũng cần. ⇒ Vào sổ nợ **kèm chính bảng số này**, không vào
  mã của story này: `AGENTS.md:15` đòi trình phương án kèm số đo cho Ice chốt, không tự chọn.

### Vì sao nửa NGUYÊN VĂN không có bản khoan dung

Nhánh `library_source_fts` là `trigram`, và 🔴 của Story 5.9 buộc mọi hàng trigram đi qua một bước
**xác minh chuỗi con ở Rust** (`source_text.to_lowercase().contains(&needle)`) vì FTS5 trigram trả
lời *"chứa các trigram này"*, không *"chứa chuỗi này"* — tỉ lệ dương tính giả đo được ở đường từ
điển là **10,3 %** (`中國` ⇒ 390 ứng viên, 40 sai). Một `library_source_fts_nd`
(`trigram remove_diacritics 1`, đã đo là chạy được) sẽ trả về những hàng mà `contains()` **luôn
luôn** loại — văn bản thô còn dấu, truy vấn thì không — nên chỉ có hai lối: bỏ phép xác minh (mở
cửa cho 10 % kết quả sai, đúng thứ *"một kết quả sai trông như bình thường"* mà `AGENTS.md` cấm),
hoặc tự cài một hàm gấp dấu trong Rust để xác minh (một CƠ CHẾ, xem mục trên).

Cộng thêm một dữ kiện: nửa nguyên văn chở `source_lang` là `zh`/`en` — gấp dấu tiếng Việt ở đó
không mua được gì đo được. ⇒ Khoan dung ở story này là chuyện của nửa **bản dịch**, và nửa nguyên
văn **vẫn chạy chính xác trong lượt khoan dung** để chuyển chế độ không bao giờ làm MẤT kết quả.

### Vì sao `_nd` lập chỉ mục trên chính cột `target_text`, không trên một cột chép đã gấp

Một cột bóng `target_text_nd` (gấp `đ→d` lúc thu hoạch) sẽ đóng được giới hạn `đ` ở trên — nhưng
`snippet()` đọc **cột được lập chỉ mục**, nên mọi đoạn trích của một hit khoan dung sẽ hiện chữ
đã bị bóp méo (`dường phượng` thay vì `đường phượng`). Đổi một vùng câm lấy một màn hình hiển thị
sai chữ của chính người dùng là một đánh đổi tệ hơn. Lập chỉ mục trên cột gốc giữ đoạn trích trung
thực, và giới hạn `đ` đi vào sổ nợ dưới dạng một mệnh đề đo được.

### Vì sao nhãn `match_kind` đến từ tập rowid, không từ một phép so chuỗi

`unicode61` khớp **trọn từ**. Một phép hậu kiểm `raw.contains(query)` là một vị từ KHÁC: nó nói
`ma` khớp `mama` trong khi chỉ mục thì không. Dùng nó để dán nhãn ⇒ một hit khoan dung bị gọi là
chính xác (hoặc ngược lại) mà không cổng nào đỏ. Tập rowid của chính nhánh chính xác là phép đo
cùng vị từ, và trong lượt **tự nới** nó còn miễn phí: nhánh chính xác vừa trả 0 hàng, nên tập rỗng
và mọi hit là `lenient` theo cấu tạo.

## Verification

**Commands:**
- `npm run check:deps && npm run check:tokens && npm run check:i18n && npm run check:commands && npm run check:layout && npm run check:panel-refs && npm run check:debt-owner && npm run check:gates && npm run check:dict && npm run check:dict-manifest && npm run check:lint` -- expected: mọi cổng exit 0
- `npm run build` -- expected: `vue-tsc` 0 lỗi, `dist/` có mặt (chạy TRƯỚC `cargo test`)
- `npm run test` -- expected: 0 ca đỏ, và bộ ca mới của `tests/frontend/librarySearch.test.ts` CÓ chạy (số ca tăng so với baseline 681)
- `cd src-tauri && cargo test --locked` -- expected: 0 ca đỏ; bộ ca mới của `library_index_contract.rs` có mặt trong đầu ra (baseline 952 ca)
- `npm run test:e2e -- --spec e2e/specs/story-5-10-diacritic-modes.e2e.mjs` -- expected: passing trong WKWebView thật
- `grep -n 'remove_diacritics' src-tauri/src/core/store/schema.rs` -- expected: đúng **hai** kết quả **ở vị trí mã** (bên trong `LIBRARY_WORK_DDL`) — một mang `0` (chỉ mục CHÍNH, AD-27) và đúng một mang `2` (chỉ mục `_nd`); **0** kết quả mang `1` ở vị trí mã. ⚠️ Baseline `a8a2f9c` cho **sáu** dòng, năm trong số đó là **chú thích** (`:1698`, `:1700`, `:1710`, `:1714`, `:1716`) — một phép đếm trần sẽ đếm cả chúng, và doc-comment mới còn NHẮC `1` để giải thích vì sao `1` là bẫy. Đếm theo vị trí, đừng đếm theo dòng (đúng bài học §Spec Change Log của Story 5.9)
- `grep -rn '3\.43\.2' src-tauri/ e2e/` -- expected: **0** kết quả (sáu chỗ đã sửa sang 3.53.2)
- `grep -rn 'library_target_fts_nd' src-tauri/src/commands/` -- expected: **0** kết quả (mọi SQL của chỉ mục sống trong `core/library/indexer.rs`)

**Manual checks (if no CLI):**
- **Ba đối chứng ĐỎ bắt buộc, mỗi cái GỠ hoặc DỜI một chỗ nối thật rồi chạy bộ test CŨ** — một bộ
  xanh không chứng minh chỗ nối được canh: ① `remove_diacritics 2` → `1` ⇒ ca `nguyen hue` phải
  ĐỎ; ② bỏ điều kiện *"chỉ nới khi `hits` rỗng"* ⇒ ca "không nới khi chính xác CÓ kết quả" phải
  ĐỎ; ③ bỏ nhánh nguyên văn khỏi lượt khoan dung ⇒ ca "khoan dung không làm mất nửa nguyên văn"
  phải ĐỎ. 🔴 Mỗi đối chứng phải là một phép GỠ/DỜI thật, không một dòng chèn thêm — một dòng chèn
  để dòng gốc chạy tiếp và ca vẫn xanh, tức "đối chứng" không chứng minh gì. Ghi lại kết quả từng
  cái; một đối chứng cho XANH thì nghi phép đối chứng trước, nghi bộ test sau.

  **KẾT QUẢ đo 2026-08-29** (`cargo test --locked --test library_index_contract`, mỗi lần GỠ/DỜI
  một chỗ nối thật rồi PHỤC HỒI nguyên vẹn trước khi đo cái kế tiếp — đối chứng bằng `diff` với
  bản gốc sau khi phục hồi, khớp byte-for-byte):
  - ① `sed` đổi CHÍNH XÁC chuỗi SQL `unicode61 remove_diacritics 2` → `…1` trong `LIBRARY_WORK_DDL`
    (`core/store/schema.rs`, vị trí MÃ, không đụng doc-comment) ⇒ **3 ca ĐỎ**:
    `a_two_diacritic_query_widens_and_the_snippet_keeps_the_original_accented_text` (1 ≠ 0),
    `an_index_file_at_schema_version_6_is_deleted_and_rebuilt_at_version_7_with_the_diacritic_index`
    (1 ≠ 0), `deleting_the_index_and_rescanning_reproduces_identical_results_in_both_modes`
    (1 ≠ 0) — đúng dự đoán: `ễ`/`ệ` (ký tự hai dấu) không gấp được ở mức `1`.
  - ② GỠ biến `exact_is_empty` khỏi công thức `widened` (`Indexer::search`,
    `core/library/indexer.rs`) — còn lại `mode == Exact && indexed_segments > 0`, không còn xét
    "chính xác có rỗng hay không" ⇒ **2 ca ĐỎ**: `an_exact_hit_wins_and_the_report_shows_exact_mode_with_no_widening`
    (6 ≠ 1 — cả sáu biến thể gần giống nhau lẫn vào), `diacritics_distinguish_six_near_identical_vietnamese_words_and_only_one_matches`
    (6 ≠ 1) — đúng dự đoán: AD-27 (phân biệt dấu) chết khi nhánh CHÍNH có kết quả vẫn bị nới.
  - ③ GỠ đúng một dòng `hits.extend(source);` khỏi NHÁNH KHOAN DUNG (dòng thứ hai trong hai lần
    xuất hiện — nhánh CHÍNH XÁC giữ nguyên) ⇒ **1 ca ĐỎ**: `lenient_mode_never_loses_a_source_half_hit`
    (0 ≠ 1) — đúng dự đoán: một hit ở nửa nguyên văn biến mất khi chuyển sang khoan dung.
  - Sau mỗi đối chứng: phục hồi nguyên văn (`diff` xác nhận khớp byte-for-byte với bản trước khi
    sửa), rồi `cargo test --locked --test library_index_contract --test library_commands_contract
    --test library_index_boundary --test ipc_contract --test config_invariants` ⇒ **0 ca đỏ**
    (exit code 0). Không đối chứng nào cho XANH giả — cả ba GỠ đúng chỗ nối mà chúng tuyên bố canh.
- **Hai lượt đo phải cùng tải máy.** Bộ vitest dựa trên timeout 5 s; một lượt `cargo test` chạy
  nền đẩy load average lên và cho tới 60 ca đỏ GIẢ (đo ở story 5.9). Chờ máy rảnh rồi đo lại
  trước khi kết luận một ca đỏ là một dòng mã. **Đo 2026-08-29:** hai lượt `cargo test --locked`
  chạy CHỒNG LÊN NHAU (do dev bất cẩn phóng hai lượt nền cùng lúc) làm cả hai bị treo I/O
  (`UN` trên `ipc_contract`) hàng phút không tiến triển — killed cả hai, chạy lại MỘT lượt sạch:
  967 ca passed / 0 failed / 1 ignored (bench p95, chạy tay có chủ ý), từ baseline 952 — đúng bài
  học "hai lượt đo phải cùng tải máy", lần này ở chiều NGƯỢC: hai lượt ĐO cùng lúc, không phải
  một lượt NỀN đá một lượt ĐO.
- **Đọc lượt CI** trước khi kết luận xanh: `pre-push` chạy trên macOS của Ice và không nói gì về
  nửa Windows. **Chưa đẩy nhánh lên CI ở lượt này** — cây vẫn ở local, Ice cần commit/push rồi
  đọc lượt CI thật (cả macOS lẫn Windows) trước khi coi story này là xanh trên CI.

**🔵 Vòng rà bốn lớp (2026-08-29) — mười mục `patch`, tất cả đã sửa, đo lại từ đầu:**
- 10 mục [high/medium/low] đã sửa: (1) `result_widened` rẽ theo `truncated` bên trong nhánh của
  nó, khoá `search_result_widened_truncated` mới, gỡ khối ⚠️ GIỚI HẠN tự-miễn-trừ; (2) một ca
  Rust mới ở `library_commands_contract.rs` cho `From<CoreSearchHit>` chạy qua một hit THẬT ở
  `mode = Lenient`, đối chứng ĐỎ xác nhận (hạ cứng `MatchKind::Exact.as_str()` ⇒ ca đỏ đúng như
  khai, đã phục hồi); (3) ca đối xứng `setLibrarySearchModeExact()` (trước đó cái tên chỉ xuất
  hiện trong văn bản, chưa từng được GỌI); (4) ba ca mount DOM mới đọc `[data-library-search-status]`
  cho `no_match_widened`/`result_widened`/`result_widened_truncated`; (5) một ca Rust mới pin lại
  GIỚI HẠN `đ`/`Đ` có chủ Ice, đặt tên `..._documented_gap`; (6) "bảy" → "tám" ở §Code Map + Task
  6/7/12 (giữ nguyên AC3 — bảy KỊCH BẢN demo, đúng như Spec Change Log đã lý giải); (7) một ca
  Rust mới cho `SearchMode::Lenient` trên chỉ mục RỖNG; (8) `search_result_widened` thêm vế "ở
  nửa bản dịch" khớp `search_no_match_widened`; (9) `SearchMode::ALL`/`MatchKind::ALL` mới +
  hai ca khứ hồi Rust; (10) ca `resetLibrarySearch` nay dựng `mode`/`effectiveMode`/`widened`
  RỜI KHỎI mặc định trước khi khẳng định chúng bị vứt.
- **Đo lại TOÀN BỘ §Verification sau khi sửa:** 11 cổng (`check:deps`…`check:dict-manifest`) +
  `check:lint` — tất cả `Tất cả phép kiểm … đạt.` / `Ba danh sách cổng khớp nhau.`, **0** dòng
  FAIL. `npm run build` — `vue-tsc` 0 lỗi, `dist/` dựng xong (1,34 s). `npm run test` — **690
  passed / 0 failed** (48 tệp), tăng từ baseline 681 VÀ từ 686 của lượt đo trước (mười mục patch
  cộng thêm 4 ca vitest: 1 cho mục 3, 3 cho mục 1+4). `cd src-tauri && cargo test --locked` —
  **972 passed / 0 failed / 1 ignored**, tăng từ baseline 952 VÀ từ 967 của lượt trước (cộng
  thêm 5 ca Rust: mục 2, 5, 7, và hai ca của mục 9). Ba phép `grep` — cùng kết quả như lượt đo
  đầu (hai dòng `remove_diacritics` ở vị trí mã, `0` dòng `3.43.2`, `0` dòng
  `library_target_fts_nd` dưới `commands/`).
- **Đối chứng ĐỎ cho mục 2** (chỗ nối mới của vòng rà này): hạ cứng
  `match_kind: MatchKind::Exact.as_str().to_owned()` trong `impl From<CoreSearchHit> for
  SearchHit` (`commands/library.rs`) ⇒ ca `search_library_carries_a_lenient_match_kind_through_to_the_wire_struct`
  ĐỎ đúng như doc-comment khai (`left: "exact", right: "lenient"`), phục hồi byte-for-byte
  (`diff` xác nhận), chạy lại XANH.

**e2e — `npm run test:e2e -- --spec e2e/specs/story-5-10-diacritic-modes.e2e.mjs`:** 🔵 **SỬA
(vòng rà bốn lớp) — mệnh đề "CHƯA CHẠY, không có màn hình/WebDriver thật" ở bản trước của mục
này SAI, gỡ tại chỗ.** Bộ này ĐÃ CHẠY THẬT và XANH: **2 passing (42 s)** trong WKWebView thật
(`webkit 605.1.15 macos`), exit 0 — cả hai mệnh đề của §Tasks 13 (truy vấn không khớp gì chạy
trọn đường tự nới qua IPC thật; bật khoan dung không làm mất hit ở nửa nguyên văn) đã nghiệm
thu qua đường ĐÚNG của nó, không phải suy luận. `eslint` sạch không đổi.

## Auto Run Result

Status: done
Blocking condition: (không có)

### Đã dựng

Hai chế độ dấu cho tìm kiếm Library (FR9). Một chỉ mục PHỤ `library_target_fts_nd`
(`unicode61 remove_diacritics 2`) trên **chính cột** `library_segment.target_text` đã có — không
cột bóng, không đường ghi mới; `LIBRARY_INDEX_MIGRATIONS` bump **6 → 7**, viết lại DDL TẠI CHỖ
đúng luật kho dẫn xuất (AD-8). `Indexer::search(query, limit, mode)` nhận `SearchMode`
(`Exact` mặc định · `Lenient`): chạy chỉ mục CHÍNH trước **mỗi lượt, không ngoại lệ**, tự nới sang
`_nd` **chỉ khi** lượt chính xác trả 0 hàng trên một chỉ mục KHÔNG rỗng, và báo ra bằng ba trường
tường minh `mode`/`effective_mode`/`widened`. Mỗi hit tự khai `match_kind`, dán bằng tập rowid của
chính nhánh chính xác — cùng vị từ, không một phép so chuỗi thứ hai. Nửa NGUYÊN VĂN vẫn chạy chính
xác kể cả trong lượt khoan dung, nên chuyển chế độ không bao giờ làm MẤT kết quả.

Trên màn hình: hai nút `library.search_mode_exact`/`_lenient` mang `aria-pressed`, một
`role="status"` **tám** nhánh phân biệt được, và một nhãn riêng trên mỗi hàng chỉ khớp qua chỉ mục
khoan dung. Đúng **một** `MessageKey` mới (`LibraryUnknownSearchMode`) — một `mode` lạ trên dây là
một `IpcError`, không một lượt im lặng rơi về mặc định.

### Số đo ép ra hình dạng này (đo 2026-08-29, SQLite **3.53.2** nhúng, `cargo test --locked`)

| Truy vấn | `unicode61 rd 1` | `unicode61 rd 2` |
|---|---|---|
| `nguyen hue` trên `Nguyễn Huệ đại phá` + `nguyen hue dai pha` | **1** (trượt hàng có dấu) | **2** |

⇒ Mức `1` khoan dung MỘT NỬA — nó trượt đúng các ký tự tiếng Việt hai dấu (`ễ`, `ệ`, `ắ`, `ộ`).

🔵 **Một mệnh đề của Story 5.9 đã hết đúng và được sửa tại chỗ ở sáu nơi:** kho ghi mọi số đo FTS5
vào *"SQLite 3.43.2"* — đó là `sqlite3` **CLI của macOS**, không phải động cơ NHÚNG mà `rusqlite`
(feature `bundled`) liên kết. `libsqlite3-sys-0.38.1/sqlite3/sqlite3.h` khai `SQLITE_VERSION
"3.53.2"`, và một test in `sqlite_version()` ra đúng số đó. Kết luận của 5.9 tái lập được nên chúng
đứng vững; thứ sai là XUẤT XỨ. Một mệnh đề PHÁI SINH thì sai hẳn: `schema.rs` viết *"tham số
`remove_diacritics` của trigram có từ SQLite 3.45"* như một lý do để KHÔNG xét nó — trên 3.53.2 nó
chạy sạch.

### Tệp đã đổi (18 tệp · +1.856 / −169)

**Rust** — `core/store/schema.rs` (chỉ mục thứ ba, bump 6 → 7, sửa số hiệu động cơ) ·
`core/library/indexer.rs` (`SearchMode`/`MatchKind` + `ALL`, `search_target_text_nd`, logic nới,
`rowid` nội bộ, một câu `rebuild` cho chỉ mục mới) · `core/i18n/mod.rs` (một khoá mới) ·
`commands/library.rs` (`unknown_search_mode`, bốn trường mới trên dây, `mode` vào vỏ).

**Frontend** — `config/library.ts` (kiểu + type guard lúc chạy) · `modes/librarySearch.ts` (ba ô nhớ
mới, hai hàm đặt chế độ, `LibrarySearchStatus` tám giá trị) · `modes/LibraryMode.vue` (hai nút, tám
nhánh, nhãn khoan dung) · `commands/index.ts` · `main.ts` · `i18n/vi.json`.

**Test và bàn đo** — `tests/library_index_contract.rs` (+696 dòng) ·
`tests/library_commands_contract.rs` · `tests/ipc_contract.rs` · `tests/frontend/librarySearch.test.ts`
(+358) · `e2e/specs/story-5-10-diacritic-modes.e2e.mjs` (mới, 229 dòng, 2 ca).

**Tài liệu** — `src-tauri/AGENTS.md` (ba chỉ mục FTS, và luật *"`_nd` không bao giờ chạy một mình"*) ·
`deferred-work.md` (đóng một mục, mở hai mục chủ Ice) · `e2e/specs/story-5-9-…mjs` (sửa số hiệu động
cơ) · `sprint-status.yaml`.

### Số đo nghiệm thu (sau lượt vá)

- 11 cổng tĩnh + `check:lint` — **xanh**.
- `npm run build` (`vue-tsc`) — **0 lỗi**.
- `npm run test` — **48/48 tệp, 690/690 ca** (baseline 681).
- `cargo test --locked` — **972 ca, 0 đỏ**, 7 `#[ignore]` có chủ, 35 binary, exit 0 (baseline 952).
- `npm run test:e2e -- --spec e2e/specs/story-5-10-diacritic-modes.e2e.mjs` — **2 passing (42 s)**
  trong WKWebView thật (`webkit 605.1.15 macos`, exit 0).
- Ba phép `grep` của §Verification — khớp: đúng **hai** `remove_diacritics` ở vị trí mã (`0` và
  `2`, **0** ca mang `1`); **0** chuỗi `3.43.2` trong `src-tauri/` + `e2e/`; **0** lần
  `library_target_fts_nd` dưới `commands/`.

⚠️ **Một lượt đo phải ghi kèm điều kiện chạy:** lượt vitest đầu chạy ở load average **6,5** và vẫn
xanh; lượt sau chạy khi load vọt lên **20,5** cũng xanh. `cargo test` được chạy RIÊNG, không song
song với vitest — ở Story 5.9 một lượt `cargo test` chạy nền đã cho **60 ca vitest đỏ GIẢ** vì
timeout 5 s.

### Ba đối chứng ĐỎ bắt buộc

| Gỡ chỗ nối | Kết quả bắt buộc | Đo được |
|---|---|---|
| `remove_diacritics 2` → `1` trong DDL | ĐỎ | **3 ca đỏ** có tên |
| gỡ điều kiện `exact_is_empty` khỏi `widened` | ĐỎ | **2 ca đỏ** có tên |
| gỡ `hits.extend(source)` khỏi RIÊNG nhánh khoan dung | ĐỎ | **1 ca đỏ** có tên |

Cộng một đối chứng thứ tư ở vòng vá: hardcode `MatchKind::Exact.as_str()` trong
`impl From<CoreSearchHit>` ⇒ ca mới ĐỎ (`"exact" != "lenient"`). Mỗi phép là một lượt **GỠ/DỜI
thật**, hoàn nguyên byte-cho-byte và đối chiếu bằng `diff` sau đó.

### Kết quả rà — 10 vá · 3 hoãn · 5 bác

Bốn lớp chạy song song trên diff 3.355 dòng. **0 `intent_gap`, 0 `bad_spec`.** Chi tiết ở
§Review Triage Log. Mục `high` duy nhất: `result_widened` che mất lời cảnh báo trần cắt — đúng lớp
lỗi vòng rà Story 5.9 đã vá một lần, mọc lại ở một nhánh mới, và một khối chú thích tự nhận
"không đáng một câu riêng ở vòng này" đã được gỡ cùng lượt.

**Follow-up review khuyến nghị: `true`** — đếm trên đúng mười mục `patch` của lượt này:
high **1**, medium **4**, low **5**; có mục `high` ⇒ `true` (điểm `3 × 4 + 1 × 5 = 17`, cũng vượt
ngưỡng 5).

### Rủi ro còn lại

1. 🔴 **`đ`/`Đ` không gấp được về `d` ở BẤT KỲ mức nào** — người dịch gõ `duong`/`duoc`/`dau` vẫn
   không tìm ra `đường`/`được`/`đầu` kể cả ở chế độ khoan dung. Tiền đề của FR9 vì thế mới đạt một
   phần. Đóng nó cần một HÀM GẤP DẤU trong Rust, tức cùng cơ chế với món nợ chuẩn hoá Unicode
   NFC/NFD đang mở. Sổ nợ, **chủ Ice**, kèm bảng số. Nay có một ca test pin lại giới hạn đó, nên
   ngày nó đóng thì ca ấy ĐỎ.
2. **Nửa NGUYÊN VĂN không có bản khoan dung** — nhánh `trigram` bắt buộc qua một bước xác minh chuỗi
   con ở Rust, và không phép xác minh nào chạy được trên một phép gấp dấu Rust chưa cài; bỏ phép
   xác minh là mở cửa cho ~10 % kết quả sai (tỉ lệ đo ở đường từ điển). Sổ nợ, **chủ Ice**.
3. **CI chưa đọc.** `pre-push` chạy trên macOS/UTC+7 của Ice và không nói gì về nửa Windows — theo
   đúng AC cuối của story, phải đọc lượt CI trước khi kết luận xanh.
4. Ba mục ở frontmatter `deferred` (bàn đo e2e không dựng được `target_text` · trần `limit` ở lượt
   khoan dung · nhánh khoan dung chưa có số đo hiệu năng).
