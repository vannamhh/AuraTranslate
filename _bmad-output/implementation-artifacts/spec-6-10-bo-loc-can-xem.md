---
title: 'Story 6.10: Bộ lọc "cần xem"'
type: 'feature'
created: '2026-09-08'
status: 'done'
baseline_commit: '83d5c88eae7a88ef5efc2c0cf4133faf9c175822'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Màn xem trước nay chở **số của từng Chương** (Story 6.10a dựng trục tóm tắt eager đúng cho lượt này), nhưng **không ai đọc chúng**: người dịch dán 50 link vẫn phải tự duyệt 50 hàng và sẽ bấm xác nhận mù ở hàng thứ mười. Ba con số đang có hoặc sắp có đều đủ để chỉ đích danh Chương đáng nghi — `length` đã có trên dây; `cleanup_match_count` đã có nhưng `0` của nó **chồng hai nghĩa** (`project.rs:1194-1199` tự khai `0` vừa là *"không đo được"* vừa là *"luật không khớp gì"*); `joined_lines` **đã được tính trên toàn văn từng đơn vị rồi vứt đi tại `pipeline.rs:644`**, nơi chỉ `.text` được giữ. Không cổng nào đỏ vì chuyện đó, và trung vị của N−1 số `0` mặc định hoá thành `0` sẽ làm màn hình khai *"M Chương sạch"* cho những Chương **chưa ai đo**.

**Approach:** Rust phân loại, TypeScript render (AD-1). Mỗi Chương mang một phán quyết **cần xem / sạch** kèm **nguyên nhân đã gọi tên**, tính bằng **hàng rào Tukey** trên chính các số tóm tắt đã có; `cleanup_match_count` sửa thành `Option<usize>` và số FR125 mới đi cùng khuôn, để *không đo được* phân biệt được với *đã đo và sạch*. Đầu màn hiện hai con số; `⌥W` co **cả hai** danh sách cùng lúc — danh sách link về mục hỏng, tầng 4 về Chương cần xem.

## Boundaries & Constraints

**Always:**
- 🔴 **`1,5` là hằng ngưỡng DUY NHẤT được phép.** Hàng rào Tukey: *ngắn bất thường* = dưới `Q1 − 1,5×IQR`; *xoá quá nhiều* và *nối dòng cao* = trên `Q3 + 1,5×IQR`. Một cơ chế cho ba tín hiệu so-tương-đối. Ice chốt 2026-09-08 (`sprint-change-proposal-2026-09-08b:207`); `1,5` là quy ước thống kê **có tên** (Tukey 1977), không phải số tự đúc, nên nó đi qua lệnh cấm hằng phù thuỷ 2026-09-05 theo đúng lý do lệnh cấm ấy tồn tại.
- 🔴 **"Không đo được" KHÔNG BAO GIỜ rơi vào nhánh sạch.** Hai tầng khác nhau, đừng trộn: ① một tín hiệu **không đủ dữ liệu cho cả lượt nhập** (dưới bốn giá trị đo được, hoặc `IQR = 0`) thì tín hiệu ấy **không tham gia** — không Chương nào bị nó phán, và màn hình nói ra điều đó; ② một tín hiệu **có hàng rào** nhưng giá trị của **Chương này** là `null` ⇒ Chương đó là **cần xem** với nguyên nhân *không đo được*, không bao giờ là sạch. Ca ② mới là thứ AC 2026-09-08 sinh ra để chặn.
- 🔴 **`cleanup_match_count` thành `Option<usize>`** (Ice chốt 2026-09-08). `null` = *không đo được cho Chương này*; `Some(0)` = *luật thật sự không khớp gì*. Đây là phép **sửa nguồn cho nó nói thật**, và doc-comment `project.rs:1194-1199` — đang lập luận rằng `0` là *"một SỐ THẬT (không đo được cho Chương này)"* — phải được **viết lại tại chỗ kèm 🔵 và ngày**, vì chính nó là chỗ hai nghĩa bị hàn vào nhau.
- 🔴 **Đừng đặt tên trần `joined_lines`** cho trường mới. Tên ấy đã thuộc `NormalizedPreviewWire` (`project.rs:1023`) với nghĩa **theo ứng viên bảng mã, có cửa sổ `EVIDENCE_WINDOW_BYTES`** — hai đại lượng khác nhau. Đi theo quy ước `count_in_chapter` đã có trong kho.
- **`ord` GIỮ NGUYÊN nghĩa cột `ord` bảng `chapter`** (Ice chốt phương án C 2026-09-08). Danh sách Chương chỉ chở **Chương thật**; link hỏng ở lại danh sách mục URL, nơi tám lý do đã hiển thị sẵn (`webimport_contract.rs:489-734`). **0 trường phân biệt mới trên `ChapterSplitPreviewEntryWire` cho link hỏng.**
- 🔴 **Hai con số do RUST cộng**, kể cả vế link hỏng — đó là một phán quyết, không phải một phép cộng ở tầng hiển thị (AD-1). Số mục hỏng đi **vào** hàm dựng tóm tắt qua tham số; đường tệp/dán tay truyền `0`.
- 🔴 **`⌥W` phải so `event.code === 'KeyW'`, KHÔNG `event.key`.** Trên macOS `⌥W` gõ ra `∑` nên `event.key` **không bao giờ** là `'w'`; `keys.ts:406` đã ghi đích danh một khuyết tật cùng lớp (`Alt+M` → `µ`), và `keys.ts:509` làm đúng cách đó. Cặp `⌥←`/`⌥→` của 6.10a né được vì phím mũi tên không bị `⌥` biến đổi ⇒ **khuôn của nó không chép thẳng sang được**.
- 🔴 **Cần một handler scrim THỨ BA.** `onTier2Keydown:570` `return` với `altKey`; `onChapterCursorKeydown:635` `return` khi **không** phải mũi tên. Nới cái nào cũng làm `⌥`+một phím khác rơi nhầm nhánh. `onScrimKeydown:663-666` là hàm tổng hợp thuần tuý — thêm lời gọi thứ ba vào đó, mỗi handler tự gác vị từ của mình.
- **`keys: undefined` + handler DOM cục bộ**, đúng khuôn Story 6.9/6.10a. `⌥` không phải phím bổ trợ chính (`keys.ts:415`) nên một hợp âm trần toàn cục vẫn nuốt phím **khi lớp phủ đã đóng**. ⚠️ `check:commands` **không** canh chuyện này (`grep lacksPrimaryMod` trên `check-commands.mjs` cho **0** kết quả) — kỷ luật phải tự giữ, đừng đọc lượt xanh của cổng thành *"hợp âm đã được duyệt"*.
- **Ô state mới: mỗi ô MỘT DÒNG, và phải được GÁN trong `resetImportPreview`** (`importPreviewState.ts:1464-1512`; hôm nay 41 `ref` + 3 `let`). `check-panel-refs.mjs:531` chỉ nhận `x = …`/`x.value = …`/`x.clear()`; `const a = ref(0), b = ref(0)` là **FAIL cứng**.
- **Bốn nhãn nguyên nhân là bốn khoá literal riêng qua một `switch` cạn** — không nội suy khoá, để `check:i18n` thấy literal.
- **Adapter `src/config/*.ts` KHÔNG BAO GIỜ ném** — hình dạng ba trạng thái, kèm vị từ kiểm kiểu lúc chạy cho **mọi** trường mới; trường `Option` kiểm bằng `x === null || <vị từ>`, `undefined` **không** hợp lệ.
- **Nhãn trạng thái phân biệt bằng sắc độ + viền**, không bóng đổ, không gradient (`check-tokens.mjs` Kiểm F, không lối miễn trừ).

**Ask First:**
- Cần **bất kỳ hằng ngưỡng nào ngoài `1,5`** ⇒ **DỪNG**. *(Số bốn giá trị tối thiểu KHÔNG phải một ngưỡng được chỉnh: nó là điều kiện số học để `Q1` và `Q3` rơi vào hai nửa khác nhau. Ghi lý do đó tại chỗ; nếu đường thi công thấy nó thật sự là một lựa chọn chứ không phải một hệ quả, DỪNG và nêu.)*
- Phép đối chứng đỏ cho `⌥W` **không chạy được dưới `happy-dom`** (bản mô phỏng không dựng `event.code`) ⇒ **DỪNG và nêu**. Đừng vá `src/` cho hết đỏ, đừng viết một ca xanh giả trên `event.key`: khoảng thiếu của bản mô phỏng vá ở `tests/frontend/support/setup.ts`, và nếu không vá được thì đây là một món nợ **có chủ** cộng một mục bàn đo tay — không phải một ca xanh.
- Hợp nhất ba nhánh `v-for` của tầng 4 (`ImportPreviewOverlay.vue:1216`, `:1239`, `:1263`) hoá ra **đổi thứ hai chế độ đang có render ra** ⇒ **DỪNG**. Hợp nhất là để thêm chiều thứ ba khỏi thành ba chỗ phải sửa, không phải để sửa hành vi đã ký của 6.6/6.10a.
- Đòi đổi hợp âm `⌘↵` / `editor.confirm_segment` ⇒ **DỪNG**. **Chủ: Ice** (`deferred-work.md:9943`); `import.preview.confirm` giữ `Mod+Alt+Enter` (`commands/index.ts:1144`).
- Cần một bất biến kiến trúc mới (một `AD`) ⇒ **DỪNG**, không tự cấp số. *(Lượt `correct-course` 2026-09-08b đã đo và kết luận Architecture **không bị chạm** — nếu đường thi công thấy khác thì đó là một phát hiện, phải nêu.)*
- Hàng rào Tukey hoá ra gắn cờ oan trên một lượt nhập thật ⇒ **DỪNG và nêu kèm số đo**, đừng tự chỉnh `1,5`.

**Never:**
- **Không đổi `schema_version()`** (đích hiện tại **19**, `schema.rs:1664-1667`), **0** bước di trú. Phán quyết *cần xem* là kết quả tính lúc chạy, không lưu xuống đĩa.
- **Không thêm lệnh IPC mới.** Trục tóm tắt eager đã đi ra dây; bộ lọc đọc dữ liệu đã có. ⇒ `generate_handler!` (`lib.rs`) và `ipc_contract.rs` **không bị chạm** ở vế đăng ký.
- **Không đụng `chapters_shape_if_all_ok` (`project.rs:2362`) và không đụng vị từ khoá nút xác nhận.** Bất biến 6.7 *"còn mục hỏng ⇒ nút xác nhận KHOÁ"* đứng nguyên; *xem được* và *ghi được* vẫn là hai mệnh đề.
- **Không đổi byte ghi xuống `.atproj`.** Story này đổi thứ **xem được** và **thứ tự chú ý**, không đổi phạm vi nhập — *"bộ lọc không bỏ qua Chương nào"* (FR132).
- **Không nhân cờ bảng mã tin cậy thấp thành N Chương *cần xem***. Nó là **cờ cấp lượt nhập**, nằm **ngoài** hai con số (Ice chốt 2026-09-08) — gắn cờ cả N làm hai số thành 50/0, tức bộ lọc mất tác dụng đúng lúc cần nhất.
- **Không dựng một Chương RỖNG giữ chỗ cho link hỏng**, và không cho mục hỏng vào phép tính trung vị. Một Chương độ dài `0` bịa sẽ kéo lệch chính hàng rào.
- **Không sửa `epics.md`/`prd.md`/`EXPERIENCE.md`** cho khớp mã — năng lực chưa dựng ghi nợ **có chủ**.
- **Không đăng ký hợp âm trần toàn cục**, không đụng tầng 1/tầng chuẩn hoá, không đụng bàn phím Epic 2.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| 50 link, cả 50 OK, vài Chương ngắn hẳn | `Chapters` ×50, `length` tản rộng | Chip hiện `N cần xem · M sạch`, `N + M = 50`; mỗi Chương cần xem nêu **tên nguyên nhân** | N/A |
| 50 link, #3 và #7 hỏng | 48 mục OK + 2 mục `error.is_some()` | Chip đếm **48 Chương + 2 link hỏng**, kèm một dòng tách hai vế; `ord` của 48 Chương vẫn `1..48` liên tục 🔴 **nút xác nhận vẫn KHOÁ** | Tám lý do đã có, `webimport_contract.rs:489-734` |
| Bấm `⌥W` | Có ít nhất một mục cần xem | **Cả hai** danh sách co: mục URL về mục hỏng, tầng 4 về Chương cần xem; tầng 4 **bỏ co gọn**, hiện đủ | N/A |
| Bấm `⌥W` khi 0 mục cần xem | `N = 0` | Bộ lọc **không bật**, một dòng nói rõ *"không có Chương nào cần xem"* | Không kêu, không ném |
| Bật lọc khi con trỏ đứng ở Chương sạch | Chương đang chọn bị lọc khỏi DOM | Con trỏ **dời tới Chương cần xem đầu tiên**; `aria-activedescendant` không bao giờ trỏ vào hàng đã bị lọc | N/A |
| `⌥W` khi lớp phủ đã đóng | `overlayOpen === false` | **Không thao tác nào** của màn nhập xảy ra | N/A |
| Nhập 3 Chương (dưới bốn giá trị đo được) | N = 3 | **Không tín hiệu so-tương-đối nào tham gia**; màn hình nói *"chưa đủ Chương để so"* thay vì khai 0 cần xem | Không phán quyết oan |
| 20 Chương dài **bằng nhau** | `IQR = 0` cho `length` | Tín hiệu `length` **không tham gia** (hàng rào suy biến); hai tín hiệu kia vẫn xét | N/A |
| Tệp một khối + mẫu phân tách, 10 Chương | `Blob`, chỉ `ord = 1` có `cleanup`; `joined` `null` cả 10 | Hai tín hiệu ấy **không đủ dữ liệu ⇒ không tham gia**; `length` vẫn xét — 🔴 và **không** Chương nào bị khai *"sạch"* dựa trên một tín hiệu chưa đo | N/A |
| Một tín hiệu CÓ hàng rào nhưng Chương k là `null` | Hàng rào tồn tại, `chapters[k].x == null` | Chương k là **cần xem**, nguyên nhân *không đo được* — không bao giờ sạch | N/A |
| Đổi ứng viên bảng mã khi bộ lọc đang bật | Bấm `E`, chọn ứng viên khác | Phán quyết **tính lại** trên số mới; bộ lọc **giữ trạng thái bật** | Không âm thầm tắt lọc |
| Bảng mã tin cậy thấp | `ConfidenceWire::Low` | Một dòng cảnh báo **cấp lượt nhập**, **ngoài** hai con số | N/A |

</frozen-after-approval>

## Code Map

**Rust — nơi con số FR125 bị vứt**
- `src-tauri/src/core/segment/normalize.rs:58-67` — `struct Normalized { text, joined_lines, blank_lines_removed }`. `joined_lines` = *"số LẦN hai dòng bị nối làm một"* (`:64-65`). Con số cần lấy nằm ở đây, đã tính.
- `src-tauri/src/core/segment/pipeline.rs:644` — `Unit::Decoded(normalize::normalize(&text, &source_lang).text)`. 🔴 **Chỗ vứt**: chỉ `.text` được giữ. `Flow` được huỷ cấu trúc ngay trên (`:638-639`) và **đã có** `cleanup_reports` làm khuôn cho một `Vec` song song theo đơn vị.
- `src-tauri/src/core/segment/pipeline.rs:121-126` `PIPELINE_ORDER` — chuẩn hoá (bước 4) **TRƯỚC** tách Chương (bước 5), khoá bởi `tests/segment_pipeline_boundary.rs::pipeline_order_matches_ad_39_step_by_step`. ⇒ Trên đường `Blob`, lúc `normalize` chạy chỉ có **MỘT** đơn vị ⇒ số đếm **không quy về Chương nào được**, kể cả `ord = 1` *(khác `cleanup_match_count`, nơi `ord = 1` giữ được số thật)*. Trên `PipelineShape::Chapters` (URL, `already_chaptered`), `normalize` chạy **trên từng đơn vị thật** ⇒ số mỗi Chương là thật.

**Rust — kiểu dây, chỗ nới**
- `src-tauri/src/commands/project.rs:1181-1200` `ChapterSplitPreviewEntryWire` — bốn trường hôm nay: `ord: i64` (`:1183`, doc khai *"cùng quy ước với cột `ord` bảng `chapter`"*), `title: Option<String>`, `length: usize`, `cleanup_match_count: usize` (`:1200`). 🔴 Doc-comment `:1194-1199` là chỗ hai nghĩa bị hàn vào nhau — **viết lại tại chỗ**.
- `project.rs:1213-1216` `ChapterSplitPreviewWire { chapter_count, chapters }` — chỗ treo tóm tắt phán quyết.
- `project.rs:1222-1237` — chỗ dựng từng mục; `ord: i as i64 + 1` (`:1226`), `cleanup_match_count` cộng dồn `per_rule_counts` với `.unwrap_or(0)` (`:1230-1233`) 🔴 **chính `unwrap_or(0)` này là chỗ `None` bị nuốt thành `0`**.
- `project.rs:1021-1026` `NormalizedPreviewWire` — **đã có trường `joined_lines`** (`:1023`) với nghĩa *theo ứng viên bảng mã, có cửa sổ*; `NormalizedCandidate`/`From` ở `:1030-1037`. ⚠️ Đây là tên bị trùng phải né.
- `project.rs:1610-1621` `cleanup_and_chapters_preview_for(..., detail_chapter_index: usize)` — hàm thuần dựng cả tóm tắt lẫn chi tiết; **chỗ nhận thêm tham số số mục hỏng**.
- `project.rs:2112` `chapter_detail_for_index` — hàm thuần của đường lazy (6.10a). Không đụng vế phán quyết.
- `project.rs:1415-1420` `ConfidenceWire` ba nhánh · `:1436` `ImportEncodingPreview::confidence` — nguyên nhân ① **đã sẵn trên dây**, 0 trường mới.
- `project.rs:2564-2570` `chapters_shape_for_view` — `filter_map` bỏ mục hỏng; doc-comment `:2548-2562` khai `items` **không hề bị đụng**, mục hỏng đứng nguyên vị trí. ⇒ Số mục hỏng đọc từ `items`, không từ danh sách Chương.
- `project.rs:2371` `UrlImportItemWire::error: Option<IpcError>` — mục hỏng lộ ra dây ở đây. Tám lý do gộp về **một** `code` `"import.web_item_failed"` (`core/segment/import.rs:376-400`), phân biệt bằng `message_key` (`i18n/mod.rs:567-584`) — bộ lọc **chỉ cần `error.is_some()`**.
- `src-tauri/src/core/webimport/mod.rs:66` `WebImportItemFailureReason` tám nhánh — đọc để biết phạm vi, **không** nới.

**Rust — thứ phải viết mới**
- 🔴 **0 tiện ích trung vị/tứ phân vị/IQR trong toàn `src-tauri/src/**`** (đo 2026-09-08: `grep -ri "median|quantile|quartile|percentile|iqr|tukey"` cho **0** kết quả, kể cả doc-comment). Bản duy nhất trong kho là `percentile()` ở `src/panels/lookupTiming.ts:90` — **TypeScript**, thuộc panel đo tra cứu. ⇒ Bản Rust phải **khai tường minh** nó khớp hay cố ý lệch quy ước bản kia; hai định nghĩa âm thầm khác nhau là chỗ hỏng.

**Frontend**
- `src/config/project.ts:190-201` kiểu `ChapterSplitPreviewEntryWire` + guard `isChapterSplitPreviewEntryWire` `:351-359` (khuôn `Option` vô hướng: `v.title === null || typeof v.title === 'string'` `:355`; khuôn `Option` lồng: `v.chapters === null || isChapterSplitPreviewWire(...)` `:413`). Hằng `CMD_*` theo cụm (`:289-291`, `:744` `CMD_PREVIEW_CHAPTER_DETAIL`).
- `src/importPreviewState.ts` — 41 `ref` (`:75-268`) + 3 `let` (`:272`, `:932`, `:1200`); `resetImportPreview` `:1464-1512`. Con trỏ Chương: `chapterCursor` `:213` → export `importPreviewChapterCursor` `:307`; `moveImportPreviewChapterCursor` `:1020`, `nextImportPreviewChapter` `:1034`, `prevImportPreviewChapter` `:1039`; `importPreviewSelectedChapters` `:436-442`. ⚠️ `runImportPreviewReload:1085` còn nhánh `if (from === 'urls') return null` `:1098` — món nợ `deferred-work.md:10037`.
- `src/ImportPreviewOverlay.vue` — tầng 4 `:1150-1291`; `<ul role="listbox" :aria-activedescendant="currentChapterDomId ?? undefined">` `:1202-1207`; **ba nhánh `v-for` lặp cùng khối `<li>`** `:1216` / `:1239` / `:1263`. `chapterSortByLength` `:333` · `chapterEntriesSortedByLength` `:343-347` (không co gọn) · `chapterEntriesDefaultWindow` `:351-367` (`first`/`last`/`showEllipsis`) · `currentChapterDomId` `:373-386` — 🔴 doc-comment `:373-376` **tự thừa nhận** con trỏ có thể vô hình khi Chương bị co gọn. Handler: `onTier2Keydown` `:569` (chặn `:570`), `onChapterCursorKeydown` `:634` (chặn `:635`), `onScrimKeydown` `:663-666`, gắn ở `:708`. `grep "@click"` cho `:717`, `:804`, `:1318`, `:1330`, `:1338` — **0** dòng trên hàng Chương.
- `src/commands/index.ts` — 11 command `import.preview.*` (`:1133` `open_picker` `keys: ['E']` · `:1144` `confirm` `Mod+Alt+Enter` · `:1261`/`:1272` `chapter_next`/`chapter_prev` `keys: undefined`); doc-comment `:1252-1256` giải thích vì sao `⌥` đi đường DOM cục bộ. `CommandDeps` `:162`, khuôn dep tuỳ chọn `:267`/`:270`. **`Alt+W` trần chưa ai chiếm** (`Mod+Alt+W` = `library.list_works` `:1365` là hợp âm **khác**).
- `src/i18n/vi.json` — khoá tầng 4 `:284-292`; khoá con trỏ Chương `:253-255`. **0 khoá nào nói về *cần xem* / *sạch* / bộ lọc.**
- Mockup: `planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/mockups/web-import.html:242-247` — chip hai số nằm **cạnh** thanh con trỏ Chương, kèm gợi ý phím `⌥W`.

**Cổng và test — ai đang canh mệnh đề nào**
- `src-tauri/tests/segment_contract.rs:9324-9377` `the_chapter_split_preview_wire_shape_carries_real_per_chapter_summary_numbers` — ca **DUY NHẤT** đọc `chapters[]` với N = 3 Chương thật; fixture đặt `cleanup_match_count: 0` làm *"số thật"* ⇒ 🔴 **phải sửa cùng lượt kèm 🔵**, cộng vị từ TS `config/project.ts:355`.
- `src-tauri/tests/cleanup_contract.rs:1241-1243` — đọc `chapters[0/1/2].cleanup_match_count` (1, 2, 0) trên N = 3 thật. **Chỗ hình dạng `Option` sẽ đỏ trước tiên.**
- `src-tauri/tests/segment_pipeline_boundary.rs::pipeline_order_matches_ad_39_step_by_step` — khoá thứ tự AD-39; **không được đụng**.
- `webimport_contract.rs:307` giữ chỗ mục hỏng · `:489-734` tám lý do — đã có chủ, **đừng dựng nguồn sự thật thứ hai**.
- `ipc_contract.rs` — **0** tham chiếu `ChapterSplitPreviewEntryWire`/`cleanup_match_count` (đã grep). Không có lệnh mới ⇒ không bị chạm.
- `tests/frontend/` **69** tệp: `importPreviewChapters.test.ts` 30 ca (4 `describe`, `:561` là *"con trỏ Chương (Story 6.10a)"*) · `importPreviewOverlayRender.test.ts` 10 ca (`:278` con trỏ DOM thật) · `importPreviewBlocks.test.ts` 14 ca.
- ⚠️ **`FILE_FLOOR = 39` của `check-panel-refs.mjs:555` đã lỗi thời** — `find src -name '*.ts'` đếm **63** ⇒ 61,9 %, dưới hẳn dải 80–85 % mà `:551` tự đặt và `:552` tự cảnh báo. **Món nợ này CHƯA CÓ CHỦ và không do story này gây ra** — ghi vào sổ kèm chủ, đừng lặng lẽ sửa sàn.
- **Số nền phải TỰ ĐO, đừng chép:** `spec-6-10a:154` ghi *"1.255 ca / 44 binary"* nhưng `ls src-tauri/tests/*.rs | wc -l` đếm **40** (đo 2026-09-08). Cổng của `pre-push` là **11**, không phải *"tám"*.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/segment/pipeline.rs:638-655` — giữ `joined_lines` của từng đơn vị vào một `Vec<Option<usize>>` song song trên `Flow`, đúng khuôn `cleanup_reports`; bước tách Chương đặt lại `vec![None; n]` — 🔴 **kể cả `ord = 1`**, vì trên đường `Blob` con số đo trên **toàn tài liệu**, gán nó cho Chương 1 là một lời nói dối
- [x] `src-tauri/src/core/segment/review.rs` (**mới**) + đăng ký trong `core/segment/mod.rs` — module **thuần** cho hàng rào Tukey: tứ phân vị + `IQR` + hai chiều hàng rào, cộng phép phân loại một Chương thành *cần xem*/*sạch* kèm danh mục nguyên nhân. **Khai tường minh quy ước tứ phân vị** và nói rõ nó khớp hay cố ý lệch `src/panels/lookupTiming.ts:90`; `1,5` là hằng **có tên duy nhất**, và điều kiện *"đủ bốn giá trị đo được"* ghi kèm **lý do số học**, không đặt như một ngưỡng chỉnh được. *(Tên module theo KHÁI NIỆM MIỀN — không mang `C1`–`C10`, không mang tên nhóm năng lực.)*
- [x] `src-tauri/src/commands/project.rs:1181-1200` — `cleanup_match_count: usize` → `Option<usize>`; thêm số FR125 tên `joined_line_count_in_chapter: Option<usize>` (theo quy ước `count_in_chapter` đã có ở `CleanupRuleReportWire` — 🔴 **không** `joined_lines` trần, tên ấy đã thuộc `NormalizedPreviewWire:1023` với nghĩa khác); thêm phán quyết `needs_review` + danh mục nguyên nhân **đóng** (`snake_case` trên dây, **không** `rename_all`); 🔵 viết lại doc-comment `:1194-1199` — chính nó đang hàn hai nghĩa vào một số `0`
- [x] `src-tauri/src/commands/project.rs:1222-1237` — gỡ `.unwrap_or(0)` ở `:1233` (chỗ `None` bị nuốt) và nuôi số FR125 từ `Flow` vào từng mục; `ord` **không đổi công thức**
- [x] `src-tauri/src/commands/project.rs:1213-1216,1610-1621` — `ChapterSplitPreviewWire` mang hai con số **do Rust cộng**, gồm cả vế link hỏng; `cleanup_and_chapters_preview_for` nhận thêm số mục hỏng qua tham số (đường tệp/dán tay truyền `0`) — đây là một phán quyết, không phải phép cộng ở tầng hiển thị (AD-1)
- [x] `src-tauri/src/commands/project.rs:2594` `url_import_encoding_preview` — chỗ gọi đường URL đếm mục hỏng từ `items` (`error.is_some()`) rồi truyền xuống; 🔴 **`chapters_shape_if_all_ok:2362` và vị từ khoá nút xác nhận KHÔNG đụng**
- [x] `src-tauri/tests/` — ca mới cho hàng rào Tukey: hai chiều · `IQR = 0` · dưới bốn giá trị · một tín hiệu có hàng rào nhưng Chương k `null` ⇒ **cần xem**, không sạch. 🔴 Kèm ca khẳng định **một Chương không đo được KHÔNG được đếm vào `M sạch`**
- [x] `src-tauri/tests/segment_contract.rs:9324-9377` + `cleanup_contract.rs:1241-1243` — sửa fixture sang `Option`, thêm khẳng định tên khoá cho các trường mới; 🔵 ghi tại chỗ vì sao `0` cũ hết đúng
- [x] `src/config/project.ts:190-201,351-359` — nới kiểu + **vị từ kiểm kiểu lúc chạy** cho mọi trường mới (`Option` kiểm bằng `x === null || …`, `undefined` không hợp lệ); adapter vẫn không bao giờ ném
- [x] `src/importPreviewState.ts` — ô bộ lọc (**một ô một dòng**) + **GÁN trong `resetImportPreview:1464-1512`**; dời con trỏ khi Chương đang chọn bị lọc khỏi tập hiện ra; bộ lọc **không bật** khi 0 mục cần xem
- [x] `src/ImportPreviewOverlay.vue` — chip hai số ở đầu lớp phủ (khuôn mockup `web-import.html:242-247`) kèm dòng tách *"X Chương + Y link hỏng"*; **hợp nhất ba nhánh `v-for`** `:1216`/`:1239`/`:1263` trước khi thêm chiều thứ ba; bộ lọc bật ⇒ tầng 4 **bỏ co gọn** (giết luôn ca `currentChapterDomId` treo) và danh sách mục URL co về mục hỏng; đóng hai nợ 6.10a trên chính danh sách này — `@click` chọn hàng (lời khai `role="option"`) và cuộn tới con trỏ khi cờ sắp xếp/lọc đổi
  🟡 **Vế `@click` chọn hàng KHÔNG đóng được — xung đột kiến trúc, ghi vào `deferred-work.md`
  (mục mới, Chủ: Ice).** `check-commands.mjs` Kiểm A (AD-34 §1) cấm MỌI `@click` không phải
  `dispatch('<id trần, 0 tham số>')` trên TOÀN `src/**/*.vue`; chọn một hàng cụ thể đòi truyền
  `ord` (đã sắp/lọc, không còn trùng chỉ số mảng) — không đường `@click` nào diễn đạt được mà
  không phá Kiểm A. Vế "cuộn tới con trỏ khi cờ sắp xếp/lọc đổi" ĐÃ đóng trọn.
- [x] `src/ImportPreviewOverlay.vue` — handler scrim **THỨ BA** cho `⌥W`, so **`event.code === 'KeyW'`** (macOS gõ ra `∑`), guard `event.repeat`, nối vào `onScrimKeydown:663-666`; 🔴 **không nới** `:570` hay `:635`
- [x] `src/commands/index.ts` + `src/main.ts` — một command bật/tắt bộ lọc, `keys: undefined`, dep **tuỳ chọn** theo khuôn `:267`/`:270`; 🔵 cập nhật doc-comment `:1252-1256` (vế `⌥W` hết nợ)
- [x] `src/i18n/vi.json` — khoá chip hai số, dòng tách hai vế, **bốn khoá nguyên nhân literal riêng** qua `switch` cạn, dòng *"chưa đủ Chương để so"*, dòng cờ bảng mã tin cậy thấp, dòng *"không có Chương nào cần xem"*; giọng **vô nhân xưng**, không giá trị rỗng
- [x] `tests/frontend/` — mở rộng `importPreviewChapters.test.ts` (bật/tắt lọc, dời con trỏ, ca 0-cần-xem) và `importPreviewOverlayRender.test.ts` (`⌥W` ở DOM thật qua `event.code`, và **không** thao tác khi lớp phủ đóng)
- [x] `src-tauri/src/commands/project.rs:1194-1200` doc-comment §Never spec 6.6 — 🔵 sửa tại chỗ: lệnh cấm cờ *"đáng ngờ"* là một lượt **HOÃN** (`deferred-work.md:9825`), và lý do hoãn (*"đòi một hằng số ngưỡng chưa đo được"*) đã được phép so **tương đối** trả lời
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — đóng `:9825` (cờ + nút lọc), nửa `⌥W` của `:9844`, `:9948` (phép so trung vị), vế thi công của `:9478`, và hai nợ tầng 4 của 6.10a (`role="option"` không chọn được bằng chuột · con trỏ vô hình khi co gọn); ghi nợ **MỚI có chủ** cho `FILE_FLOOR = 39` và mọi vế còn hở — đóng bằng **chữ**, không xoá; đóng nửa thì 🟡 kèm phần còn hở

**Acceptance Criteria:**
- 🔴 Given một lượt nhập N Chương có hàng rào tồn tại và một Chương mang giá trị `null`, when đọc hai con số, then Chương đó nằm trong **`N cần xem`** và **KHÔNG** trong `M sạch` — và ca này phải **ĐỎ** nếu ai trả `cleanup_match_count` về `usize` với mặc định `0`
- 🔴 Given phép **GỠ** điều kiện `IQR > 0`, when chạy bộ test **CŨ**, then nó phải **ĐỎ** — đối chứng là một phép **GỠ biên dịch được và chạy**, không phải một lập luận
- 🔴 Given một lượt nhập và **không thao tác tay nào**, when xác nhận, then byte ghi xuống `.atproj` **trùng đúng** kết quả trước story này
- 🔴 Given một danh sách có mục hỏng, when đọc trạng thái nút xác nhận, then nó **KHOÁ** — bất biến 6.7 không được nới theo bộ lọc
- Given `⌥W` trên bàn phím macOS (`event.key === '∑'`, `event.code === 'KeyW'`), when bấm, then bộ lọc bật — và ca phải **ĐỎ** nếu ai đổi vị từ sang `event.key`
- Given `cargo test`, when chạy, then `schema_version()` **không đổi** và **0** bước di trú mới
- Given **mười một** cổng của `pre-push` cộng `cargo test --locked` cộng `npm run test`, when chạy trọn, then **0** finding và **0** ca đỏ

## Design Notes

**Vì sao "không đủ dữ liệu cho cả lượt" khác "null cho Chương này", và vì sao phải tách.** Trộn hai ca này lại là cách bộ lọc tự huỷ. Ca ① — trên đường `Blob` + mẫu phân tách, chỉ `ord = 1` có `cleanup_report` (`project.rs:1196-1199`, giới hạn pipeline đã ký ở 6.10a), tức **một** giá trị đo được; một hàng rào dựng trên một giá trị là vô nghĩa, nên tín hiệu ấy **không tham gia** và không Chương nào bị nó phán. Ca ② — hàng rào **có thật** (đủ giá trị từ các Chương khác) nhưng Chương k là `null`: nếu để `null` mặc định hoá thành `0`, `0` sẽ nằm gọn trong hàng rào và Chương k được khai **sạch** trong khi **chưa ai đo nó**. Đó đúng là lớp rỗng-im-lặng mà `AGENTS.md` §Known pitfalls dẫn ra ba lần (Story 1.16 · 2.10 · 3.9), và là lý do AC *"không đo được khác sạch"* được thêm 2026-09-08.

**Vì sao `Blob` không cho `ord = 1` con số FR125, trong khi `cleanup` thì có.** Hai giới hạn nghe giống nhau nhưng khác hẳn về chất. `cleanup_report` được tính **sau** khi có văn bản của Chương 1 nên nó là một phép đo **thật của Chương 1**. `joined_lines` thì được tính ở bước 4, lúc `PIPELINE_ORDER` (`pipeline.rs:121-126`) mới chỉ có **một** đơn vị là **toàn tài liệu** — con số ấy thuộc về tài liệu, không thuộc về Chương nào. Gán nó cho Chương 1 sẽ cho Chương 1 một số cao vô lý (nó gánh cả 49 Chương kia) và đẩy chính nó qua hàng rào trên. ⇒ `None` cho **cả N**, và tín hiệu không tham gia trên đường đó.

**Vì sao bộ lọc bật thì bỏ co gọn, chứ không lọc rồi mới co.** `chapterEntriesDefaultWindow` (`:351-367`) cắt `slice(0,3)`/`slice(-3)` trên mảng **gốc**, còn `currentChapterDomId` (`:373-386`) dựng lại tập rendered từ hai nhánh. Thêm một chiều lọc mà vẫn giữ co gọn thì biểu thức đó trả một `id` cho hàng **đã bị lọc khỏi DOM** ⇒ `aria-activedescendant` treo — một khuyết tật a11y không cổng nào đỏ. Bỏ co gọn khi lọc làm ca ấy **không tồn tại**, và nó khớp đúng thứ `chapterSortByLength` (`:343-347`) đã làm hôm nay: chế độ nào cần nhìn thấy đủ thì hiện đủ. Đây cũng là chỗ **đóng luôn** món nợ *"con trỏ vô hình khi Chương bị co gọn"* của 6.10a trên đúng danh sách ấy, thay vì dựng một bề mặt thứ hai.

**Vì sao `event.code`, và vì sao khuôn 6.10a không chép sang được.** `⌥←`/`⌥→` an toàn với `event.key` vì `⌥` không biến đổi phím mũi tên. `⌥W` trên macOS gõ ra `∑`, nên một vị từ `event.key === 'w'` sẽ **không bao giờ đúng** — và nó sẽ **xanh** trên `happy-dom` nếu bản mô phỏng tự điền `key` từ `code`. `keys.ts:406` đã ghi đích danh một ca cùng lớp (`Alt+M` → `µ`) và `:509` so `code`. ⇒ Đối chứng đỏ cho vế này là bắt buộc; nếu `happy-dom` không dựng được `code` thì **dừng và nêu**, đừng đổi vị từ sản phẩm cho hợp bản mô phỏng.

**Vì sao link hỏng KHÔNG thành một hàng Chương** (Ice chốt phương án C, 2026-09-08). `ord` được doc-comment `:1183` khai là *"cùng quy ước với cột `ord` bảng `chapter`"*, và một link hỏng **không bao giờ được nhập** nên nó không có `ord` trong DB. Cho nó một hàng trong danh sách Chương là phá quy ước ấy và buộc mọi chỗ đọc `ord` phải xét thêm một nhánh. Đắt hơn: một Chương giữ chỗ độ dài `0` đi vào phép tính trung vị sẽ **kéo lệch chính hàng rào** mà story này dựng. ⇒ Hai danh sách, một thao tác: `⌥W` co cả hai, Rust cộng cả hai thành `N`, và chip nói ra vế nào bao nhiêu.

## Verification

**Commands:**
- `npm run build && cargo test --locked` — expected: 0 đỏ; `dist/` phải có **TRƯỚC** `cargo test`. ⚠️ **Tự đo và ghi số thật** (ca / binary), đừng chép `spec-6-10a:154` — nó ghi *"44 binary"* trong khi `ls src-tauri/tests/*.rs | wc -l` đếm **40**.
- `npm run test` — expected: 0 đỏ. ⚠️ `vitest.config.ts:96` `fileParallelism: false` ⇒ một lượt chậm.
- Chạy **TỪNG** cổng: `check:deps` `check:tokens` `check:i18n` `check:commands` `check:layout` `check:panel-refs` `check:dict` `check:dict-manifest` `check:lint` `check:gates` `check:debt-owner` — **mười một**, đúng `.githooks/pre-push:81`.
- 🔴 **Đối chứng đỏ ① — `null` có thật sự khác `0` không.** Trả `cleanup_match_count` về `usize` với `.unwrap_or(0)` như hôm nay rồi chạy bộ test mới — expected: ca *"không đo được không được đếm là sạch"* **ĐỎ**. Trả lại — xanh.
- 🔴 **Đối chứng đỏ ② — hàng rào có tự bảo vệ không.** **GỠ** điều kiện `IQR > 0` rồi chạy — expected: ca *"20 Chương dài bằng nhau"* **ĐỎ** (mọi Chương bị gắn cờ). Đây phải là một phép **GỠ THẬT**, không phải chèn thêm một nhánh.
- 🔴 **Đối chứng đỏ ③ — `⌥W` có so đúng trường không.** Đổi vị từ sang `event.key === 'w'` rồi chạy `npm run test` — expected: ca DOM **ĐỎ**. ⚠️ Nếu nó **XANH**, nghi bản mô phỏng trước, nghi bộ test sau: `happy-dom` có thể đang tự điền `key` từ `code`. Ca đó khi ấy **không** là một phép kiểm — dừng và nêu.
- **Đối chứng đỏ ④ — dây lồng.** Đặt `#[serde(rename_all = "camelCase")]` lên `ChapterSplitPreviewEntryWire` rồi chạy `segment_contract.rs` — expected: **ĐỎ**. ⚠️ Lần này phép biến đổi **không rỗng** (`needs_review`, `review_causes` là từ ghép), khác ghi chú `spec-6-9:167` nơi mọi trường là từ đơn.
- **Đối chứng ⑤ — giới hạn đã biết, kỳ vọng XANH.** Gieo `keys: ['Alt+w']` vào command mới rồi `npm run check:commands` — expected: **XANH** (cổng bắt va chạm, **không** bắt hợp âm có an toàn không; `grep lacksPrimaryMod` cho 0 kết quả) ⇒ ghi thành nợ **có chủ**, đừng đọc lượt xanh thành *"hợp âm đã được duyệt"*.
- `cargo test --locked --test segment_contract` — expected: `schema_version()` **không đổi**

**Manual checks (if no CLI):**
- Dán 50 link (hai link hỏng): chip hiện hai số, dòng tách *"X Chương + 2 link hỏng"*; bấm `⌥W` ⇒ **cả hai** danh sách co lại; **nút xác nhận xám**.
- Nhập một tệp `.txt` + mẫu phân tách cho 10 Chương: màn hình nói *"chưa đủ dữ liệu"* cho hai tín hiệu chuẩn hoá/làm sạch, **không** khai 10 Chương sạch.
- Nhập 3 Chương: không tín hiệu so-tương-đối nào tham gia, màn hình nói rõ vì sao.
- Bật lọc rồi bấm `E` đổi ứng viên bảng mã: phán quyết tính lại, bộ lọc **vẫn bật**.
- Đóng lớp phủ rồi bấm `⌥W`: màn nhập **không đổi gì**.
- Chụp màn ở **cả hai theme**; nhãn cần xem/sạch phân biệt được bằng **sắc độ + viền**, không bóng đổ.

## Suggested Review Order

**Phán quyết "cần xem" — nơi luật thật sự sống**

- Điểm vào: toàn bộ luật phân loại nằm trong một hàm thuần duy nhất.
  [`review.rs:193`](../../src-tauri/src/core/segment/review.rs#L193)

- Hàng rào Tukey; `1,5` là hằng có tên duy nhất, `IQR = 0` trả `None`.
  [`review.rs:161`](../../src-tauri/src/core/segment/review.rs#L161)

- Sàn bốn giá trị là HỆ QUẢ số học, không phải ngưỡng chỉnh được.
  [`review.rs:125`](../../src-tauri/src/core/segment/review.rs#L125)

- Vòng rà bắt: thiếu CẢ HAI tín hiệu từng cho hai nhãn trùng khoá.
  [`review.rs:217`](../../src-tauri/src/core/segment/review.rs#L217)

**Con số FR125 từng được tính rồi vứt đi**

- `Flow` nay chở số dòng bị nối theo từng đơn vị, khuôn `cleanup_reports`.
  [`pipeline.rs:476`](../../src-tauri/src/core/segment/pipeline.rs#L476)

- Đường `Blob`: `None` cho CẢ N — kể cả `ord = 1`, vì số đo thuộc toàn tài liệu.
  [`pipeline.rs:536`](../../src-tauri/src/core/segment/pipeline.rs#L536)

**Dây — chỗ hai nghĩa được tách ra**

- `cleanup_match_count` thành `Option`; `0` thôi mang hai nghĩa.
  [`project.rs:1208`](../../src-tauri/src/commands/project.rs#L1208)

- Hai con số do Rust cộng, gồm cả vế link hỏng (AD-1).
  [`project.rs:1306`](../../src-tauri/src/commands/project.rs#L1306)

- Số mục hỏng đi vào qua tham số; đường tệp/dán tay truyền `0`.
  [`project.rs:1732`](../../src-tauri/src/commands/project.rs#L1732)

**Bàn phím và trạng thái bộ lọc**

- `⌥W` so `event.code`; trên macOS `event.key` là `∑`, không bao giờ `'w'`.
  [`ImportPreviewOverlay.vue:731`](../../src/ImportPreviewOverlay.vue#L731)

- Chặn BẬT khi không hàng rào nào tồn tại — vòng rà bắt được lỗ này.
  [`importPreviewState.ts:1076`](../../src/importPreviewState.ts#L1076)

**Màn hình**

- Lọc thì bỏ co gọn, nên `aria-activedescendant` không thể trỏ hàng đã lọc.
  [`ImportPreviewOverlay.vue:389`](../../src/ImportPreviewOverlay.vue#L389)

- Một `v-for` duy nhất thay ba nhánh lặp — điều kiện để thêm chiều thứ ba.
  [`ImportPreviewOverlay.vue:403`](../../src/ImportPreviewOverlay.vue#L403)

- Bốn nhãn nguyên nhân qua `switch` cạn, để `check:i18n` thấy literal.
  [`ImportPreviewOverlay.vue:355`](../../src/ImportPreviewOverlay.vue#L355)

**Ngoại vi**

- Bốn ca đầu-cuối qua dây thật, gồm ca `Blob` của ma trận.
  [`review_contract.rs:20`](../../src-tauri/tests/review_contract.rs#L20)
