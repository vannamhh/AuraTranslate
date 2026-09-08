---
title: 'Story 6.10a: Xem trước theo từng Chương và điều hướng Chương'
type: 'feature'
created: '2026-09-08'
status: 'done'
baseline_commit: 'fcc7b3ef4d97841c84c5b47e68e9f833b0f7a28a'
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

**Problem:** Màn xem trước **chưa bao giờ đa-Chương**, và triệu chứng giống nhau che **hai** nguyên nhân khác nhau. ① Đường **tệp/dán tay + mẫu phân tách**: `run_pipeline` trả N `ImportedChapter` thật, `chapters_wire` dùng cả N, `import_totals` cộng cả N — nhưng `blocks_wire` và `chapter0_report` chỉ lấy `chapters.first()` (`project.rs:1600-1604`) ⇒ **dữ liệu đã tính rồi bị vứt**. ② Đường **danh sách URL**: hình dạng bị gói về một `Blob` đơn vị trước khi gọi pipeline (`:1892-1893` → `:1750-1753`) ⇒ Chương 2..N **chưa bao giờ đi qua pipeline** lúc xem trước. Người dùng bật một luật làm sạch rồi chỉ kiểm chứng được nó trên một phần N của thứ nó sắp xoá — đúng lớp rủi ro FR124 tồn tại để chặn. Không cổng nào đỏ vì chuyện này.

**Approach:** Một **con trỏ *Chương đang chọn*** làm đối tượng cho cả hai tầng. Dây tách hai nhịp (Ice chốt 2026-09-08): **tóm tắt eager** — mỗi mục của `ChapterSplitPreviewEntryWire` mang thêm số của **chính nó**; **chi tiết lazy** — văn bản/khối của tầng 2 và tầng 3 chỉ tính cho Chương đang chọn, qua một lệnh IPC mới khi con trỏ dời. `⌥←`/`⌥→` dời con trỏ. Rust vẫn là nơi duy nhất tính.

## Boundaries & Constraints

**Always:**
- **Dò bảng mã GIỮ NGUYÊN ở `chapters.first()`** — *"một bảng mã cho cả danh sách"* (§Always spec 6.7, doc-comment `PipelineInput::encoding`). Thứ đổi là cái gì được **nuôi vào pipeline**, không phải cái gì quyết định bảng mã. Trộn hai vế này là mở lại một quyết định đã chốt.
- 🔴 **Vị từ XEM và vị từ GHI phải TÁCH.** Hôm nay `url_import_encoding_preview` (`project.rs:2395`) dùng **chính** `chapters_shape_if_all_ok` — nên còn một mục hỏng là **không có màn xem trước nào cả**. Story này nới **đường xem**, và để `chapters_shape_if_all_ok` (`:2362`) **y nguyên** cho đường ghi. *Xem được* và *ghi được* là hai mệnh đề khác nhau; gộp chúng lại là cách bất biến 6.7 chết trong im lặng.
- **AD-1 — Rust tính, TypeScript render.** Con trỏ là state UI; mọi số và mọi văn bản của Chương đang chọn đến từ Rust.
- **Khuôn hai lớp cho lệnh IPC mới** (Story 1.8): hàm thuần nhận `Option<&Store>` + vỏ `#[tauri::command]` **mỏng** trong `mod wire` lấy `State` qua **`try_state`**, 0 quy tắc trong vỏ.
- 🔴 **Lệnh mới phải vào `generate_handler!` (`lib.rs:634`+) VÀ có một ca test mang tên nó.** Đo 2026-09-08: mọi assert `generate_handler!` trong `ipc_contract.rs` là **danh sách CÓ TÊN theo từng story** (`:783`, `:824`, `:910`, `:970`, `:1087`, `:1277`); **không** cổng nào quét chung *"mọi `#[tauri::command]` đều đã đăng ký"*. Thiếu một dòng ⇒ `invoke()` trả `"command not found"` **chỉ khi người dùng bấm**, mọi cổng vẫn xanh.
- **`⌥←`/`⌥→` theo khuôn Story 6.9**: vẫn là command đăng ký (AD-34) nhưng `keys: undefined`, phím tới qua handler DOM cục bộ. `⌥` không phải phím bổ trợ chính (`keys.ts:415`) nên hợp âm toàn cục vẫn nuốt phím **khi lớp phủ đã đóng** (`:512-513`).
- 🔴 **Cần một handler THỨ HAI, không được nới `onTier2Keydown`.** `ImportPreviewOverlay.vue:517` `return` với `ctrlKey || metaKey || altKey`; nới vị từ đó làm `⌥`+`j` rơi vào nhánh `j`.
- **Ô state mới: mỗi ô MỘT DÒNG, và phải được GÁN trong `resetImportPreview`** (`importPreviewState.ts:1237`, hôm nay 36 ô `ref` + 2 `let`, 38 phép gán). `check-panel-refs.mjs:531` chỉ nhận `x = …`/`x.value = …`/`x.clear()`; `const a = ref(0), b = ref(0)` là **FAIL cứng**, không phải bỏ qua.
- **Adapter `src/config/*.ts` KHÔNG BAO GIỜ ném** — hình dạng ba trạng thái, kèm vị từ kiểm kiểu lúc chạy (trường tuỳ chọn kiểm bằng `x === null || isXxx(x)`; `undefined` **không** hợp lệ).

**Ask First:**
- Cần **bất kỳ hằng ngưỡng nào** (bao nhiêu là "ngắn bất thường", bao nhiêu là "xoá quá nhiều") ⇒ **DỪNG**. Phép so trung vị và bốn nguyên nhân *"cần xem"* thuộc **Story 6.10**, và Ice đã cấm hằng phù thuỷ ở đây 2026-09-05.
- Nới tầng 2 cho Chương k > 1 đòi **tải lại byte từ mạng** ⇒ **DỪNG**. AD-41: không lời gọi mạng nào khi người dùng không bấm; byte đã tải sống trong `UrlImportItemsState`.
- Đòi đổi hợp âm `⌘↵` / `editor.confirm_segment` (`commands/index.ts:2157`) ⇒ **DỪNG**. **Chủ: Ice** (`deferred-work.md:9864`), và nó thuộc bề mặt bàn phím của 6.10.
- Cần một bất biến kiến trúc mới (một `AD`) ⇒ **DỪNG**, không tự cấp số; quét cả spine lẫn mọi `ad-brief-*.md`.
- **Tách `Tier2BlockOverridesState` theo Chương hoá ra không đủ** (ví dụ: số khối đổi khi đổi ứng viên bảng mã, nên override theo INDEX mất neo) ⇒ **DỪNG và nêu kèm số đo**. Đây là dữ liệu người dùng đã sửa tay; áp nhầm nó sang một Chương khác là một lượt ghi sai **không lần ngược được** — xem §Design Notes.

**Never:**
- **Không dựng bộ lọc *"cần xem"***: không hai con số `N cần xem · M sạch`, không `⌥W`, không phân loại, không trung vị. Cả bốn thuộc Story 6.10 — story này chỉ dựng **nền**.
- **Không cuộn vòng** ở hai đầu danh sách — mất chỗ đứng trên một danh sách 50 Chương.
- **Không đổi byte ghi xuống.** Story này đổi thứ **xem được**, không đổi thứ **ghi xuống**.
- **Không đổi `schema_version()`**, 0 bước di trú.
- **Không đăng ký hợp âm trần toàn cục**, không đụng tầng 1 và tầng chuẩn hoá.
- **Không sửa `epics.md`/`prd.md`** cho khớp mã — năng lực chưa dựng ghi nợ **có chủ**.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Tệp một khối + mẫu phân tách cho 3 Chương | `Blob(RawBytes)` + `ChapterPattern` | Ba mục tách Chương mang `ord`/`title`/`length` **của chính nó**. 🔵 **SỬA 2026-09-08 (Ice chốt, sau bước thi công):** vế *"tầng 3 hiện luật khớp ở đúng Chương đang chọn"* **hết đòi được trên đường này** — bước làm sạch chạy trên **toàn blob TRƯỚC** bước tách Chương (`pipeline.rs:960-976`, luật có từ Story 6.6), nên chỉ `ord = 1` có `CleanupReport` thật và `cleanup_match_count` của Chương 2..N là `0` **có lý do**, không phải một số bịa. Con trỏ vì thế **không dời** trên đường này | N/A |
| Danh sách 3 URL, cả ba OK | `Chapters(RawBytes)` ×3 | Cả ba đi qua pipeline; tầng 2 hiện khối của **Chương đang chọn**, không phải Chương 1 | N/A |
| `⌥→` ở Chương cuối | Con trỏ = N−1 | **Dừng**, con trỏ không đổi, không lời gọi IPC | Không kêu, không ném |
| `⌥←`/`⌥→` khi lớp phủ đã đóng | `overlayOpen === false` | **Không thao tác nào** của màn nhập xảy ra | N/A |
| Giữ `⌥→` cho auto-repeat | `event.repeat === true` | Không bắn một tràng IPC — khuôn `event.repeat` của `ImportPreviewOverlay.vue:547` | N/A |
| 5 link, link #3 hỏng | Một mục `error.is_some()` | 4 Chương tải được hiện **đầy đủ**; mục #3 **giữ chỗ tại vị trí 3** kèm một trong tám lý do 🔴 **nút xác nhận vẫn KHOÁ** | Lý do đã có, `webimport_contract.rs:489-734` |
| Mọi mục đều hỏng | Không mục OK nào | Xem trước **rỗng có lý do**, phân biệt được với *"chưa nạp"* | Không khẳng định "không có nội dung" khi chưa biết |
| Đổi ứng viên bảng mã khi con trỏ ở Chương k | Bấm `E`, chọn ứng viên khác | Tầng 2/3 dựng lại **cho Chương k**, con trỏ **giữ nguyên** | Không âm thầm nhảy về Chương 1 |
| Đường tệp/dán tay, không mẫu phân tách | N = 1 | Con trỏ tồn tại nhưng **không đi đâu được**; hành vi trùng đúng hôm nay | N/A |

</frozen-after-approval>

## Code Map

**Rust — hai chỗ ghim, hai nguyên nhân khác nhau**
- `src-tauri/src/commands/project.rs:1892-1893` — `PipelineShape::Chapters(chapters) => match chapters.first()`. 🔴 Đây là nguyên nhân ② (URL). Doc-comment `:1886-1891` giải thích vì sao **dò bảng mã** chốt từ đơn vị đầu — vế đó **giữ**, chỉ vế "nuôi vào pipeline" đổi.
- `src-tauri/src/commands/project.rs:1733` `encoding_candidate_wire` · `:1750-1753` gói `full_bytes` về `PipelineShape::Blob` một đơn vị. Chỗ này quyết định pipeline chỉ thấy 1 đơn vị.
- `src-tauri/src/commands/project.rs:1574` `cleanup_and_chapters_preview_for` — `:1599` `build_chapter_split_preview_wire(&chapters)` (dùng **cả N**) · `:1600` `build_chapter_blocks_preview_wire(chapters.first(), …)` · `:1602-1604` `chapter0_report` · `:1607-1615` `import_totals` cộng cả N. 🔴 Nguyên nhân ① sống ở `:1600` và `:1602-1604`.
- `src-tauri/src/commands/project.rs:2389` `url_import_encoding_preview` · `:2395` `chapters_shape_if_all_ok(items)?` — 🔴 **chỗ trộn vị từ xem với vị từ ghi**. `:2362` `chapters_shape_if_all_ok` **không được đụng** (đường ghi). `:2379` `sync_pending_from_url_items` · `:2417` chỗ dựng `UrlImportBatchWire`.

**Rust — kiểu dây**
- `project.rs:1045` `EncodingCandidateWire` **7 trường** (`label`/`encoding`/`preview`/`normalized`/`cleanup`/`chapters`/`blocks`). Không `rename_all` — tên trường đi thẳng ra dây.
- `project.rs:1181` `ChapterSplitPreviewEntryWire` (`ord`/`title`/`length`) · `:1202` `ChapterSplitPreviewWire` (`chapter_count`/`chapters`) — **đây là trục tóm tắt eager**, nới ở đây.
- `project.rs:1150` `CleanupPreviewWire` · `:1237` `BlockBodyWire` (`#[serde(tag = "kind", rename_all = "snake_case")]`) · `:1261` `BlockWire` · `:1275` `ChapterBlocksPreviewWire` — **đây là chi tiết lazy**.
- `project.rs:1007-1014` `PendingImportSource`/`PendingImportSourceState` · `:2159` `Tier2BlockOverridesState` (`Mutex<Vec<Option<bool>>>`, hôm nay chỉ cho đơn vị 0 — con trỏ Chương làm nó thành một câu hỏi, xem §Design Notes). `app.manage` ở `lib.rs:1055` và `:1071`.
- `core/segment/pipeline.rs:166` `ChapterInput` · `:190` `PipelineShape` · `:203` `PipelineInput` · `:323`/`:332`/`:343`/`:352` bốn builder · `:710` `run_import` · `:499-518` chỗ `Chapters` seed **cả N** unit. `project.rs:235-238` `run_pipeline` là **chỗ gọi sản phẩm duy nhất**.
- `core/segment/encoding.rs:64` `EVIDENCE_WINDOW_BYTES = 4096` — trần cửa sổ chứng cứ.
- `project.rs:3471`+ `mod wire` — 12 vỏ hiện có; lệnh mới đặt cạnh `:4144` `tier2_block_set_kept`/`:4203` `tier2_block_confirm_range` (khuôn gần nhất).

**Frontend**
- `src/ImportPreviewOverlay.vue` — tầng 2 `:791-867` (cả section bọc trong `importPreviewLastSubmittedFrom === 'urls'` `:801`; doc-comment `:787-789` khai *"Chương đầu tiên"*, **sửa tại chỗ kèm 🔵**) · tầng 3 `:870-1026` · **tầng 4 `:1029-1132`**, danh sách `:1079-1128` (`chapterEntriesSortedByLength` `:1081` / `chapterEntriesDefaultWindow` `:1097-1123`) — 🔴 **hôm nay 0 khái niệm Chương đang chọn**. Scrim `:609-611` (`@keydown.esc`/`@keydown.tab`/`@keydown="onTier2Keydown"`). `onTier2Keydown` `:516-569`, vị từ chặn `:517`.
- **Khuôn để chép, đã chạy trong chính tệp này:** `blocksList` template ref `:579` · watcher `.focus()` + `scrollIntoView` theo index `:584-589` · `:aria-selected` mỗi hàng `:838` · `event.repeat` guard `:547` (dẫn tiền lệ `GlossaryManageOverlay.vue:336`). `GlossaryManageOverlay.vue:461` `aria-activedescendant` + `:465-468` `role="option"`; `:95-108` khuôn "list ref riêng khỏi panel ref".
- `src/importPreviewState.ts:1237-1276` `resetImportPreview` · `:1226` `cancelImportPreview` · `:331-336` `importPreviewSelectedCandidate` (trục **ứng viên bảng mã**, không phải Chương) · `:400-403` `importPreviewSelectedBlocks` · `:369-375` `importPreviewSelectedCleanup` · `:189-204` bảy ô của 6.9 (khuôn đặt tên) · `:655-669` `applyUrlImportBatch`. ⚠️ `runImportPreviewReload` có nhánh `if (from === 'urls') return null` — **món nợ `deferred-work.md:10037` có chủ 6.10a**, đóng hoặc ghi lại phần còn hở.
- `src/config/project.ts:190-197` `ChapterSplitPreviewEntryWire` + guard `:347-355` · `:225-228`/`:383-391` container + guard · `:202-220`/`:357-381` ba kiểu khối + ba guard · `:246-261`/`:393-412` `EncodingCandidateWire` + guard · `:280-283` hình dạng ba trạng thái · `:285-287` khuôn hằng `CMD_*`.
- `src/commands/index.ts:1114-1220` chín command `import.preview.*` (bảy `keys: undefined`) — 🔵 doc-comment `:1110-1112` khai *"`⌥←`/`⌥→`/`⌥W` VẪN CHƯA dựng — ngoài phạm vi story này"*, **hết đúng một nửa** ở lượt này, sửa tại chỗ. `Alt+ArrowLeft`/`Alt+ArrowRight` trần **chưa ai chiếm** (đã chiếm: `Mod+Alt+Arrow*` = `focus.prev/next_panel` `:1033-1044`). `:162` `CommandDeps` — dep mới **tuỳ chọn**.
- `src/i18n/vi.json:251` `tier2_url_first_note` = *"Dãy khối của Chương ĐẦU TIÊN…"* — 🔴 **mệnh đề này hết đúng ở lượt này, viết lại**. `:280-288` chín khoá tầng 4 — **không khoá nào** nói về Chương đang chọn, cần khoá mới.

**Cổng và test — ai đang canh mệnh đề nào**
- `src-tauri/tests/cleanup_contract.rs:468` (`Blob(AlreadyText)`) · `:539` (`Chapters(RawBytes)`, **1** Chương, có override) · `:1203` (`Blob(AlreadyText)` + pattern → **2** Chương, so `run_import` với hàng đã ghi, **không** gọi `preview_import_encoding`) — ba ca "xem trước = xác nhận từng byte". `:1109` `count_in_import_equals_the_hand_counted_sum_…` gọi `cleanup_and_chapters_preview_for` trên 3 Chương nhưng **chỉ** kiểm hai số vô hướng.
- ⚠️ `cleanup_contract.rs:1293`/`:1342` bàn đo 2000 Chương **chỉ khẳng định `chapter_count`**, không bao giờ đánh chỉ số vào `chapters.chapters[i]`, và mỗi Chương tổng hợp **~70 byte** ⇒ nó **không** là đối chứng cho payload hay cho nội dung per-Chương.
- 🔴 **Đo 2026-09-08: KHÔNG ca nào đọc `ChapterSplitPreviewWire.chapters[]` cho N > 1 Chương thật.** Trục tóm tắt của story này là bề mặt chưa ai canh.
- `src-tauri/tests/segment_contract.rs:9075` `the_import_encoding_preview_wire_shape_keeps_snake_case_field_names` — serialize `ImportEncodingPreview`/`EncodingCandidateWire`/`NormalizedPreviewWire`, và cả hai fixture đặt `chapters: None` ⇒ **hình dạng `ChapterSplitPreviewEntryWire` chưa từng đi qua serde ở ca này**.
- `src-tauri/tests/ipc_contract.rs:812` (ba vỏ xem trước bảng mã + `app.manage(PendingImportSourceState)`) · `:898` (ba vỏ URL của 6.7) · `:957` — ba danh sách **có tên**, khuôn cho ca mới.
- `src-tauri/tests/webimport_contract.rs:307` giữ chỗ mục hỏng · `:489`/`:515`/`:545`/`:612`/`:652`/`:678`/`:714`/`:734` tám lý do.
- `scripts/check-panel-refs.mjs:531` vị từ "đã GÁN" · `:331-349` `outOfSubset()`. `scripts/check-commands.mjs:1673-1683` dựng keymap thật trên **cả hai** nhánh `isMac` — nó bắt va chạm hợp âm, **không** bắt hợp âm có an toàn không.
- `tests/frontend/importPreviewOverlayRender.test.ts` — **ca mount DOM thật DUY NHẤT** (5 ca), doc-comment `:1-19` tự khai phạm vi hẹp. `tests/frontend/importPreviewChapters.test.ts` (18 ca, `describe` `importPreviewSelectedChapters` `:103`) là chủ gần nhất của khái niệm Chương.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/project.rs` — tách **vị từ XEM** khỏi `chapters_shape_if_all_ok`: một hàm thuần dựng `PipelineShape::Chapters` từ **các mục OK**, giữ vị trí mục hỏng; `url_import_encoding_preview:2395` gọi hàm mới, `sync_pending_from_url_items:2380` và đường ghi **giữ nguyên** — *xem được* và *ghi được* là hai mệnh đề, gộp lại là cách bất biến 6.7 chết im lặng
- [x] `src-tauri/src/commands/project.rs:1892-1893` — nuôi **trọn** hình dạng `Chapters` vào pipeline; **giữ** `chapters.first()` cho vế **dò bảng mã** và sửa doc-comment `:1886-1891` cho nói đúng hai vế đã tách (🔵 + ngày)
- [x] `src-tauri/src/commands/project.rs:1733,1750-1753` — `encoding_candidate_wire` nhận hình dạng thật thay vì gói `full_bytes` về `Blob` một đơn vị — đây là chỗ Chương 2..N biến mất trên đường URL
- [x] `src-tauri/src/commands/project.rs:1574-1625` — `cleanup_and_chapters_preview_for` trả **tóm tắt cho mọi Chương** cộng **chi tiết cho một Chương chỉ định**; thay `chapters.first()` ở `:1600` và `:1602-1604` bằng chỉ số con trỏ — nguyên nhân ① là hai dòng này, dữ liệu đã có sẵn
- [x] `src-tauri/src/commands/project.rs:1181-1205` — `ChapterSplitPreviewEntryWire` mang thêm số tóm tắt **của chính Chương đó**; giữ `snake_case`, **không** `rename_all`
- [x] `src-tauri/src/commands/project.rs` `mod wire` — một lệnh mới "dựng lại tầng 2/3 cho Chương thứ k", khuôn hai lớp (hàm thuần + vỏ mỏng `try_state`), đặt cạnh `:4144`/`:4203`
- [x] `src-tauri/src/lib.rs:634`+ — thêm lệnh mới vào `generate_handler!` — không cổng chung nào canh việc này; thiếu nó thì lỗi **chỉ lộ khi người dùng bấm**
- [x] `src-tauri/tests/ipc_contract.rs` — ca mới **mang tên lệnh mới**, theo khuôn `:812`/`:898`: khẳng định có mặt trong `generate_handler!` + tên tham số khớp `src/config/project.ts`
- [x] `src-tauri/tests/segment_contract.rs:9075` — serialize `ChapterSplitPreviewWire` **có dữ liệu thật** (≥ 2 mục, đủ trường tóm tắt) và khẳng định tên khoá — hôm nay cả hai fixture đặt `chapters: None`
- [x] `src-tauri/tests/cleanup_contract.rs` — ca mới: `cleanup_and_chapters_preview_for` trên **3 Chương thật** khẳng định tầng 3 của Chương k **khác** tầng 3 của Chương 0 và khớp phép đếm tay; nối bất biến "xem trước = xác nhận từng byte" sang một lượt **có dời con trỏ**
- [x] `src-tauri/tests/webimport_contract.rs` — ca mới: 5 link với #3 hỏng ⇒ xem trước dựng được với 4 Chương, mục #3 **giữ vị trí 3**, 🔴 **và `chapters_shape_if_all_ok` vẫn trả `None`** (hai khẳng định trong một ca, vì đây là chỗ dễ trộn nhất)
- [x] `src/config/project.ts` — kiểu tóm tắt mới + kiểu tham số lệnh mới + hằng `CMD_*` + adapter ba trạng thái + **vị từ kiểm kiểu lúc chạy** cho mọi trường mới — adapter không bao giờ ném, `undefined` không hợp lệ
- [x] `src/importPreviewState.ts` — ô con trỏ Chương (**một ô một dòng**) + **GÁN trong `resetImportPreview`**; đóng hoặc ghi lại phần còn hở của `runImportPreviewReload` (`deferred-work.md:10037`) — `check:panel-refs` đòi một phép GÁN, một lượt đọc không tính
- [x] `src/ImportPreviewOverlay.vue` — handler `@keydown` **THỨ HAI** trên scrim cho `⌥←`/`⌥→` (không nới `:517`); con trỏ ở danh sách tầng 4 theo khuôn `blocksList` `:579-589` + `aria-activedescendant`; tầng 2/3 đọc theo Chương đang chọn; 🔵 sửa doc-comment `:787-789`
- [x] `src/commands/index.ts` + `src/main.ts` — hai command `import.preview.chapter_next`/`chapter_prev`, `keys: undefined`, dep tuỳ chọn; 🔵 sửa doc-comment `:1110-1112` (`⌥W` vẫn còn nợ, `⌥←`/`⌥→` hết nợ)
- [x] `src/i18n/vi.json` — viết lại `:251` `tier2_url_first_note` (mệnh đề "Chương ĐẦU TIÊN" hết đúng); khoá mới cho con trỏ, giọng **vô nhân xưng**
- [x] `tests/frontend/` — mở rộng `importPreviewChapters.test.ts` (trạng thái con trỏ, dừng ở hai đầu) và `importPreviewOverlayRender.test.ts` (`⌥←`/`⌥→` ở tầng DOM thật, và **không** thao tác khi lớp phủ đóng)
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — đóng `:9890` (tầng 3), `:10028` (tầng 2), nửa `⌥←`/`⌥→` của `:9844`, và xử lý `:10037`; nợ **MỚI có chủ** cho mọi vế còn hở — đóng bằng chữ, không xoá; đóng nửa thì 🟡 kèm phần còn hở

**Acceptance Criteria:**
- Given một lượt nhập N Chương, when **GỠ** phép truyền chỉ số con trỏ xuống `cleanup_and_chapters_preview_for`, then bộ test **CŨ** phải **ĐỎ** — đối chứng là một phép **GỠ**, không phải một phép chèn
- 🔴 Given một lượt nhập và **không thao tác tay nào**, when xác nhận, then byte ghi xuống `.atproj` **trùng đúng** kết quả trước story này
- 🔴 Given một danh sách có mục hỏng, when đọc trạng thái nút xác nhận, then nó **KHOÁ** — và ca này phải **đỏ** nếu ai cho `chapters_shape_if_all_ok` bỏ qua mục hỏng
- Given con trỏ ở Chương k > 0 và người dùng đổi ứng viên bảng mã, when tầng 2/3 dựng lại, then chúng hiện Chương **k**, không nhảy về Chương 0
- Given `cargo test`, when chạy, then `schema_version()` **không đổi** và **0** bước di trú mới
- Given **mười một** cổng của `pre-push` cộng `cargo test --locked` cộng `npm run test`, when chạy trọn, then **0** finding và **0** ca đỏ

## Spec Change Log

- 2026-09-08 (sau thi công, Ice chốt) — **Hàng 1 của I/O Matrix đòi một năng lực pipeline KHÔNG tồn tại; sửa hàng, không sửa mã.** Lượt quy hoạch khái quát món nợ tầng 3 (`deferred-work.md` §*"Tầng 3 … ghim vĩnh viễn vào Chương ĐẦU"*) ra **cả hai** đường mà không kiểm đường `Blob` có báo cáo làm sạch theo từng Chương hay không. Nó **không** có: `pipeline.rs:960-976` (Story 6.6) ghi rõ bước 3 chạy trên toàn blob **trước** bước 5 tách Chương, nên `flow.cleanup_reports` bị reset thành `vec![None; n]` và báo cáo nguyên vẹn gắn vào Chương đầu; phân bổ đúng theo ranh giới Chương đòi một cơ chế theo dõi vị trí xuyên bước chuẩn hoá **chưa dựng**. ⇒ Đây là **lỗi quy hoạch**, không phải một lượt thi công thiếu. Hàng 1 nay khai đúng thứ đạt được và **gọi tên giới hạn**; nửa còn lại thành một món nợ **có chủ**.
  **Trạng thái xấu đã tránh được:** lượt thi công đã dựng một ca test tên *"đường tệp/dán tay (N > 1 do mẫu phân tách) — con trỏ KHÔNG đi đâu được, **đúng I/O Matrix**"* — một ca **XANH khoá chặt một hành vi trái hợp đồng đã ký**, viện dẫn chính hợp đồng đó làm thẩm quyền. Nhận lượt này mà không đọc lại ma trận thì kho có một mệnh đề sai được một ca xanh bảo kê — đúng lớp *"kiểm chứng dương có thể khoá chỗ mù"*.
  **Ba chỗ khác sửa cùng lượt:** ① mệnh đề *"hàng N > 1 đường tệp KHÔNG có trong ma trận đã ký"* trong `deferred-work.md` là **sai** (nó LÀ hàng 1) — sửa tại chỗ; ② món nợ tầng 3 bị đánh `✅ ĐÓNG` trong khi chính ca nó mô tả (`Blob` + mẫu) còn hở ⇒ hạ xuống **🟡**; ③ ba món nợ mới ghi `Chủ: chưa ai nhận` — cách nói nằm ngoài `NEGATIVE_OWNER_RE` (`check-debt-owner.mjs:174`) nên cổng chống-nợ-mồ-côi đọc nó thành một chủ **thật** và báo `mở KHÔNG có Chủ: 0`. Vá **cổng** cho nó nói thật, và gán chủ thật cho ba mục.
  **Một vế nữa của §Intent cũng sai, ghi ra thay vì để nó đứng im.** Nguyên nhân ① viết đường
  tệp/dán tay là *"dữ liệu đã tính rồi bị vứt"*. Đo lại 2026-09-08: **không** có dữ liệu nào bị
  vứt trên đường đó — `build_chapter_split_preview_wire(&chapters)` đã dùng CẢ N; `blocks_wire`
  luôn `None` vì `split_chapters_step` đặt `flow.blocks = vec![None; n]` và khối chỉ tồn tại khi
  `extract_main_content`, thứ đi cùng hình dạng `Chapters`; còn `chapter0_report` là báo cáo DUY
  NHẤT tồn tại. ⇒ Thực tế chỉ có **MỘT** nguyên nhân (đường URL bị gói về một đơn vị) **cộng**
  một giới hạn pipeline có sẵn trên đường `Blob`. Tôi đã trình "hai nguyên nhân" như một phát
  hiện, và một nửa của nó là suy luận từ hình dạng lời gọi (`chapters.first()` ở `:1600`) chứ
  không phải từ một phép đo xem `chapters[k]` có mang gì. Đúng lớp lỗi *"triệu chứng có ở cả hai
  phía không phải nguyên nhân"*. §Intent nằm trong khối đã ký nên không sửa tại chỗ; bản đính
  chính sống ở đây.
  **KEEP — thứ lượt thi công làm đúng, phải sống sót qua mọi lượt dựng lại:** ① tách `chapters_shape_for_view` khỏi `chapters_shape_if_all_ok` đúng như §Always đòi, và **tự tìm thêm** một hồi quy từ chiều ngược lại (nút xác nhận vẫn gác trên `preview === null`, một vị từ **hết trùng** với *"có mục hỏng"* sau khi đường xem được nới); ② `onScrimKeydown` là hàm **tổng hợp thuần tuý** gọi hai handler độc lập, mỗi handler tự gác vị từ của mình — giữ đúng lệnh cấm *"không nới `onTier2Keydown`"* thay vì lách nó; ③ `cleanup_match_count` khai thẳng trong doc-comment rằng `0` nghĩa là *"không đo được cho Chương này"*, không phải một giá trị giữ chỗ né `Option`; ④ bốn mục nợ mới đều có thân đầy đủ, chỉ thiếu đúng cái chủ.

- 2026-09-08 (vòng rà bước 4 — ba lớp, KHÔNG quay vòng) — **Tám phép vá, không một `bad_spec`.** Hai lớp rà độc lập hội tụ vào cùng bốn chỗ, và cả bốn đã tự kiểm bằng mã trước khi giao vá, không nhận theo lời khai reviewer. Nặng nhất: `build_chapter_blocks_preview_wire` áp `block_overrides` **theo chỉ số** lên bất kỳ Chương nào được truyền vào — độc lập với việc pipeline chỉ áp cho `units[0]` — nên con trỏ ở Chương k > 0 làm màn hình báo `confirmed = true` cho một khối **chưa ai từng chạm**. Doc-comment `chapter_detail_for_index` lập luận đúng về việc KHÔNG cắt `shape`, nhưng phép áp lần thứ hai ở tầng dựng dây thì không ai chặn — một lý do đúng chỉ phủ được phạm vi của chính nó. Ba chỗ còn lại: nhánh lỗi không dọn chi tiết đang hiện (tầng 2/3 hiện Chương k−1 dưới nhãn Chương k), guard "vượt mặt" không phân biệt hai lượt gọi cùng index, và đường lazy gửi mẫu phân tách trong khi đường eager hard-code `None`.
  **Vì sao KHÔNG phải `bad_spec` (luật workflow bảo nghi ngờ thì chọn `bad_spec`):** spec đã **gọi tên trước** từng rủi ro — §Ask First cho `Tier2BlockOverridesState`, §Always cho vị từ xem/ghi — nên đây không phải khuyết tật của spec; và mọi phép sửa đều cục bộ, không đòi dựng lại 1.674 dòng. Không ở trong tình trạng nghi ngờ, nên không chọn theo mặc định phòng thủ.
  **Hai phát hiện nhắm vào chính lượt nghiệm thu của tôi, ghi ra thay vì lặng lẽ sửa:** ① phép vá `NEGATIVE_OWNER_RE` của tôi thêm `chưa story`/`chưa xác định` trong khi chỉ đo được `chưa ai` — tái diễn đúng lỗi mà doc-comment tôi vừa viết đang chẩn đoán ("một phép ĐẾM bị đọc thành một phép liệt kê ĐẦY ĐỦ"); đã gỡ hai cụm đoán. ② đối chứng hai chiều của tôi cho phép vá đó mới chỉ là **TƯỜNG THUẬT** trong doc-comment, không chạy lại được — nay là hai ca trong **Kiểm B có sẵn** (13 → 15 ca), không phải một tệp test mới, vì mệnh đề đó đã có chủ.
  **Số đo sau vá, tự chạy chứ không chép:** 11/11 cổng + build xanh · `cargo test --locked` exit 0, **1.255** ca / 44 binary · `npm run test` exit 0, **928** ca / 69 tệp.
  **Ba mục HOÃN có chủ** (không do story này gây ra): danh sách tầng 4 khai `role="option"` mà không chọn được bằng chuột · con trỏ vô hình khi Chương rơi vào phần co gọn "ba đầu ⋯ ba cuối" (kèm vế không cuộn khi bật sắp-theo-độ-dài) · mỗi lượt dời con trỏ clone byte của N Chương hai lần. **Hai mục BÁC:** rút chung một helper cho hai vị từ xem/ghi — spec **cố ý** muốn hai vị từ tách rời, ba dòng trùng lặp rẻ hơn một chỗ nối làm mờ đúng ranh giới story này dựng lên.

## Design Notes

**Vì sao tóm tắt eager mà chi tiết lazy — Ice chốt 2026-09-08 trên một phép tính, không một sở thích.** Hôm nay mỗi lượt dựng chở, cho **5 ứng viên bảng mã**: cửa sổ chứng cứ ≤ 4096 byte (`encoding.rs:64`) **cộng** `final_text` đầy đủ của Chương 0 **cộng** toàn bộ khối của Chương 0 — tức văn bản Chương 0 đi qua dây **hai lần**, nhân 5. Cho mọi Chương eager thì con số ấy thành **O(cả tài liệu) × 5 × 2**; bảy mẫu bàn đo 6.1 là HTML 154–295 KB và `epics.md` mô tả người dùng dán **50 link**. ⚠️ Và bàn đo 2000 Chương đang có (`cleanup_contract.rs:1293`) dùng Chương tổng hợp **~70 byte**, nên nó **vẫn xanh** trên phương án nặng — đúng lớp "cổng xanh không phủ đường nằm ngoài cổng". Nửa eager giữ lại **chỉ các số**, và đó chính là thứ Story 6.10 cần cho phép so trung vị: 6.10 cần **số** trên mọi Chương, **không** cần văn bản.

**Vì sao một handler thứ hai chứ không nới vị từ đang có.** `onTier2Keydown:517` `return` với `ctrlKey || metaKey || altKey`. Nới nó ra để cho `⌥` lọt thì `⌥`+`j` rơi vào nhánh `j` (`:531`) và bật/tắt một khối trong khi người dùng đang định đi Chương — một thao tác **phá huỷ nhìn thấy được**, trên một bề mặt sắp ghi xuống đĩa. Hai handler tách bạch giữ mỗi vị từ nói đúng một chuyện. Cùng lý do, `GlossaryManageOverlay.vue:299` và `GlossaryQueueOverlay.vue:151` cũng chặn `altKey` — khuôn này **không** dùng lại nguyên si được, đó là dữ kiện chứ không phải một chỗ để lách.

**Chỗ chưa có câu trả lời hiển nhiên, ghi ra thay vì giấu:** `Tier2BlockOverridesState` (`project.rs:2159`) hôm nay là **một** `Vec<Option<bool>>` cho đơn vị 0. Con trỏ Chương biến nó thành một câu hỏi thật — override của Chương 1 và Chương 2 có dùng chung một vector không. Spec 6.9 đã ghi một giản lược cùng hình dạng (*"dùng chung một vector cho cả năm ứng viên bảng mã, áp theo INDEX khối"*) và lý do ở đó là cấu trúc DOM bất biến qua năm bảng mã — **lý do ấy KHÔNG bắc cầu sang các Chương khác nhau**, vì hai Chương là hai tài liệu HTML khác nhau. ⇒ Đường thi công phải tách override **theo Chương**, hoặc dừng và nêu. Đừng đọc giản lược của 6.9 thành một tiền lệ cho ca này.

## Verification

**Commands:**
- `npm run build && cargo test --locked` — expected: 0 đỏ; `dist/` phải có **TRƯỚC** `cargo test`. ⚠️ Số nền phải **đo lại**: `spec-6-9` ghi "1230 ca / 44 binary" nhưng `ls src-tauri/tests/*.rs` đếm **40** tệp hôm nay — đừng chép tiếp một con số không phải mình đo.
- `npm run test` — expected: 0 đỏ. ⚠️ `vitest.config.ts:96` `fileParallelism: false` ⇒ một lượt chậm.
- Chạy **TỪNG** cổng: `check:deps` `check:tokens` `check:i18n` `check:commands` `check:layout` `check:panel-refs` `check:dict` `check:dict-manifest` `check:lint` `check:gates` `check:debt-owner` — **mười một**, đúng danh sách `.githooks/pre-push:81`. 🔵 Mệnh đề *"tám cổng"* của `spec-6-9` đã **hết đúng**; có 12 tệp `check-*.mjs`, hai trong đó (`check:scope`, `check:scope:bundled`) cố ý ngoài pre-push.
- 🔴 **Đối chứng đỏ ① — con trỏ có thật sự chảy xuống Rust không.** **GỠ** phép truyền chỉ số con trỏ vào `cleanup_and_chapters_preview_for` (để nó lại lấy `chapters.first()`) rồi chạy `cleanup_contract.rs` — expected: ca 3-Chương **ĐỎ**. Trả lại — xanh. Đây phải là một phép **GỠ THẬT biên dịch được và chạy**; một "đối chứng logic" không tính.
- 🔴 **Đối chứng đỏ ② — vị từ xem có thật sự tách khỏi vị từ ghi không.** Cho hàm dựng-shape-để-XEM bỏ qua mục hỏng **và** cho `chapters_shape_if_all_ok` cũng bỏ qua ⇒ chạy `webimport_contract.rs` — expected: ca "nút xác nhận vẫn KHOÁ" **ĐỎ**. Nếu nó XANH thì hai vị từ vẫn đang bị trộn.
- **Đối chứng đỏ ③ — dây lồng.** Đổi tên **một** trường mới của `ChapterSplitPreviewEntryWire` bằng `#[serde(rename = …)]` rồi chạy `segment_contract.rs` — expected: **ĐỎ**. ⚠️ Đừng dùng `rename_all = "camelCase"` làm phép gieo: `spec-6-9:167` đã đo và nó **XANH** khi mọi trường là từ đơn, tức phép biến đổi rỗng.
- **Đối chứng ④ — giới hạn đã biết, kỳ vọng XANH.** Gieo `keys: ['Alt+ArrowLeft']` vào lệnh mới rồi `npm run check:commands` — expected: **XANH** (nó bắt va chạm hợp âm, không bắt hợp âm có an toàn không) ⇒ ghi thành nợ **có chủ**, đừng đọc lượt xanh thành "hợp âm đã được duyệt".
- `cargo test --locked --test segment_contract` — expected: `schema_version()` **không đổi**

**Manual checks (if no CLI):**
- Dán 5 link (một link hỏng), bấm tải: xem trước **dựng được**, mục hỏng giữ đúng vị trí kèm lý do, **nút xác nhận xám**.
- `⌥→` vài lần: tầng 2 và tầng 3 đổi theo Chương; ở Chương cuối bấm tiếp thì **đứng yên**, không kêu.
- Ở Chương 3 bấm `E` chọn ứng viên bảng mã khác: vẫn ở Chương 3.
- Giữ `⌥→`: không thấy một tràng lời gọi trong nhật ký; chụp màn ở **cả hai theme**.
- Đóng lớp phủ rồi bấm `⌥←`/`⌥→`: màn nhập **không đổi gì**.
- Mở một tệp `.txt` **không** mẫu phân tách: màn hình trùng đúng hôm nay.

## Suggested Review Order

**Nguyên nhân gốc — đọc ba chỗ này là đủ hiểu cả story**

- Chỗ Chương 2..N từng biến mất: nay nạp thẳng `shape` đủ N, còn dò bảng mã vẫn chốt từ đơn vị đầu.
  [`project.rs:1944`](../../src-tauri/src/commands/project.rs#L1944)

- Tóm tắt cho MỌI Chương, chi tiết cho MỘT Chương — hai nhịp của dây, gói gọn trong một tham số.
  [`project.rs:1649`](../../src-tauri/src/commands/project.rs#L1649)

- Vị từ XEM, tách hẳn khỏi vị từ GHI; một mục hỏng không còn giết cả màn xem trước.
  [`project.rs:2564`](../../src-tauri/src/commands/project.rs#L2564)

**Chỗ vòng rà tìm ra, đọc kỹ nhất ở đây**

- Chương k > 0 nhận lát rỗng: override của Chương 0 từng làm màn hình khai "người dùng đã xác nhận".
  [`project.rs:1649`](../../src-tauri/src/commands/project.rs#L1649)

- Nút xác nhận nay gác trên vị từ RIÊNG — `preview === null` đã hết trùng với "có mục hỏng".
  [`importPreviewState.ts:353`](../../src/importPreviewState.ts#L353)

- Lỗi và trạng thái cũ đều DỌN chi tiết; rỗng có lý do thay vì Chương k−1 dưới nhãn Chương k.
  [`importPreviewState.ts:963`](../../src/importPreviewState.ts#L963)

- Token riêng từng lượt gọi — `chapterCursor !== index` không phân biệt được hai lượt cùng index.
  [`importPreviewState.ts:932`](../../src/importPreviewState.ts#L932)

**Chi tiết lazy — một lệnh IPC mới, và cái giá của nó**

- Chạy trên TOÀN `shape` chứ không cắt: cắt sẽ đặt Chương đang xem vào vị trí 0 của override.
  [`project.rs:2112`](../../src-tauri/src/commands/project.rs#L2112)

- Trả `None` thay vì đoán khi `chapter_index` không còn khớp, và khi nhánh `Blob` gặp k > 0.
  [`project.rs:2049`](../../src-tauri/src/commands/project.rs#L2049)

- Vỏ mỏng `try_state`; thiếu một dòng ở `generate_handler!` thì lỗi chỉ lộ khi người dùng bấm.
  [`project.rs:4479`](../../src-tauri/src/commands/project.rs#L4479)

- Dòng đăng ký đó — không cổng chung nào canh nó, nên có một ca test mang tên chính lệnh.
  [`lib.rs:664`](../../src-tauri/src/lib.rs#L664)

**Bàn phím — hai handler độc lập, không nới cái nào**

- `⌥` đi đường riêng; nới vị từ của tầng 2 sẽ làm `⌥`+`j` bật/tắt một khối ngoài ý muốn.
  [`ImportPreviewOverlay.vue:634`](../../src/ImportPreviewOverlay.vue#L634)

- Hàm tổng hợp thuần tuý — Vue chỉ cho một `@keydown` trần trên một phần tử.
  [`ImportPreviewOverlay.vue:663`](../../src/ImportPreviewOverlay.vue#L663)

- Dừng ở hai đầu, no-op ngoài đường URL, và chặn chồng lệnh — ba vị từ, mỗi cái một lý do.
  [`importPreviewState.ts:1020`](../../src/importPreviewState.ts#L1020)

**Trục tóm tắt — thứ Story 6.10 sẽ bám**

- `0` ở đây nghĩa là "không đo được cho Chương này", không phải một giá trị giữ chỗ né `Option`.
  [`project.rs:1200`](../../src-tauri/src/commands/project.rs#L1200)

**Cổng và phép kiểm**

- Đối chứng P1: gỡ lát rỗng ra thì ca này đỏ vì `confirmed` hoá `true`.
  [`cleanup_contract.rs:1341`](../../src-tauri/tests/cleanup_contract.rs#L1341)

- Cụm phủ định của cổng chống-nợ-mồ-côi — chỉ `chưa ai` là đo được, hai cụm đoán đã bị gỡ.
  [`check-debt-owner.mjs:192`](../../scripts/check-debt-owner.mjs#L192)
