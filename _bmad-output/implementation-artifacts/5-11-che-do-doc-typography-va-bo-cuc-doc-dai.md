---
title: 'Story 5.11: Chế độ đọc — typography và bố cục đọc dài'
type: 'feature'
created: '2026-08-30'
status: 'done'
baseline_revision: 'fc7ca8949c550af5c65badbd027ed3ef9e842eb9'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
warnings: ['oversized']
deferred:
  - summary: >-
      Lớp phủ mục lục của Chế độ đọc chưa từng đi qua một webview thật — `Mod+L`, phím
      điều hướng, tiêu điểm và lượt mở Chương đều chỉ được đo bằng `invoke` giả.
    evidence: |-
      `e2e/specs/story-5-11-reading-mode.e2e.mjs` không có một lần nhắc `toc` hay `Mod+L`
      nào (grep 2026-08-30 = 0). Hành vi tiêu điểm sau khi đóng lớp phủ là mệnh đề DOM
      trong engine thật, đúng hạng mà `tests/AGENTS.md` giao cho e2e chứ không cho
      `happy-dom`. Bàn đo dựng được: `split_chapter_at_segment` (Story 5.8) là đường tạo
      Chương thứ hai mà e2e với tới được, nên một Tác phẩm hai Chương là khả thi.
    location: >-
      e2e/specs/story-5-11-reading-mode.e2e.mjs
    severity: medium
  - summary: >-
      Trang đọc không mang thuộc tính `lang` nào, nên trình đọc màn hình phát âm nguyên văn
      tiếng Trung/tiếng Anh bằng giọng tiếng Việt.
    evidence: |-
      `ReadingChapter` không chở `source_lang` trên dây, nên webview không có dữ kiện để
      dựng `lang` cho cột lề song ngữ. `OpenChapter` (`config/chapter.ts`) thì CÓ
      `source_lang` — dữ kiện tồn tại ở Rust, chỉ chưa đi qua lệnh này. Cùng lượt đó đóng
      luôn câu hỏi dấu cách giữa hai câu nguyên văn: tiếng Trung không cần, tiếng Anh cần.
    location: >-
      src-tauri/src/commands/segment.rs (ReadingChapter) · src/modes/ReadingMode.vue
    severity: low
  - summary: >-
      `read_reading_chapter` chép lại nguyên câu `SELECT` chín cột của
      `read_open_chapter_segments`, giữ đồng bộ bằng một dòng chú thích.
    evidence: |-
      Hai câu SQL mang cùng bộ lọc `WHERE chapter_id = ?1 AND retired_at IS NULL` và cùng
      `ORDER BY ord, id`. Sửa bộ lọc ở MỘT nơi làm Chế độ đọc và Workspace nói khác nhau về
      cùng một Chương, và không cổng nào đỏ — cùng hình dạng lỗi mà `omit.rs` đã ghi cho
      một vị từ bị chép hai lần. Ngoài ra hàm dựng trọn `ChapterSegment` (chín trường) rồi
      bỏ đi năm trường khi dựng `ReadingSegment`.
    location: >-
      src-tauri/src/commands/segment.rs
    severity: medium
---

<intent-contract>

## Intent

**Problem:** `src/modes/ReadingMode.vue` là **khung rỗng 50 dòng** mang đúng một câu trạng thái
(`t('mode.reading.status')`), nên FR11 chưa có một pixel nào. Cùng lúc, chốt lọc
`core::segment::omit::segments_in_translation` — Ice ký Quyết định #2 đường (b) ngày 2026-08-15,
dựng sẵn **cái chốt** cho một bề mặt chưa tồn tại — vẫn **0 nơi gọi** trong mã sản phẩm; món nợ
đó ghi đích danh *"Chế độ đọc → Epic 5 (Story 5.11 · 5.12 · 5.13)"*. Story này là bề mặt tiêu thụ
đầu tiên, nên nó vừa dựng trang đọc vừa cắm vào chốt ấy.

**Approach:** Một lệnh IPC **đọc-thuần mới** (`read_reading_chapter`) trả về Chương đang mở đã
**gom sẵn thành đoạn** ở Rust — đi qua `omit::segments_in_translation` rồi qua một hàm thuần mới
`core::segment::reading::paragraphs_in_translation`. Webview chỉ render. Ba mức typography lấy
nguyên sáu token đã có (`read-lg`/`read-md`/`read-sm` × `read-measure-lg/md/sm`), thước đo bằng
`ch`. Thanh công cụ mang công tắc song ngữ, ba preset, tinh chỉnh, sáng/tối, mục lục — mỗi thứ một
`dispatch` id riêng, phím tắt trần `B` `1` `2` `3` `D` và `Mod+L`.

## Boundaries & Constraints

**Always:**

- 🔴 **Lọc câu đã cắt bỏ ở RUST, không một `v-if="!s.is_omitted"` ở Vue** — `core/segment/omit.rs`
  nói thẳng lệnh cấm này và nêu lý do (AD-1; bề mặt xuất bản của Epic 8 phải đi qua **cùng** chốt).
- 🔴 **Cấu trúc đoạn của bản dịch ĐỌC `segment.is_target_paragraph_end`** — không suy từ
  `is_paragraph_end`, không suy từ nội dung, không suy từ vị trí `\n` trong `target_text` (AD-46,
  AC4 của Story 2.5d).
- 🔴 **Ranh giới đoạn không được mất vì một câu bị cắt bỏ.** Nếu câu mang cờ kết đoạn chính là câu
  bị cắt bỏ, cờ ấy **chuyển** cho câu còn sống liền trước trong cùng đoạn. Đây là một quy tắc, nên
  nó sống ở Rust (§Design Notes).
- Chiều rộng cột đo bằng **`ch`**, đặt trên **chính phần tử mang cỡ chữ đọc**; cỡ chữ · giãn dòng ·
  họ chữ · giãn ký tự chỉ đến từ token (`--font-read-*`, `--leading-read-*`, `--face-read-*`,
  `--space-read-measure-*`).
- 🔴 **Giãn dòng KHÔNG BAO GIỜ dưới 1.66**, kể cả giá trị đến từ thanh trượt tinh chỉnh. `check:tokens`
  Kiểm E chỉ soi `tokens.json` — một giá trị lúc chạy **không đi qua cổng nào**, nên sàn phải cưỡng
  chế trong mã sản phẩm và có test vitest riêng.
- Chế độ đọc **không hiển thị công cụ biên tập nào**: không vạch lề segment, không nút xác nhận,
  không panel, không `contenteditable`, không lưới hai cột.
- Khuôn hai lớp cho bề mặt IPC mới: hàm thuần nhận `Option<&OpenWork>` + một `#[tauri::command]`
  mỏng trong module lồng `wire` lấy `State` qua **`try_state`**. Adapter TS **không bao giờ ném**,
  trả hình dạng ba trạng thái `{ <giá trị> | null, error: IpcError | null }`, kiểm kiểu **lúc chạy**.
- `@click` trong `.vue` là **đúng một** `dispatch('<id>')` với id literal (`check:commands` Kiểm A).
- Mọi ô nhớ cấp module mới đi qua một hàm `reset*` của chính tệp đó, **hoặc** một miễn trừ CÓ TÊN
  kèm lý do trong `scripts/check-panel-refs.mjs`; mỗi khai báo trên **một dòng**.
- Mọi chuỗi hiển thị qua `t()` và `vi.json` (phẳng, khoá chấm có tiền tố miền, câu **vô nhân xưng**).
- Chỉ token: không màu viết thẳng, không `box-shadow`, không gradient, không `opacity` để lùi chữ;
  `outline: none` chỉ trên gốc `tabindex="-1"`.

**Block If:**

- Đường đi buộc phải **gỡ hoặc đổi một hợp âm ĐANG CÓ CHỦ** — cụ thể `Mod+Comma`, hiện thuộc
  `shortcuts.open` (Story 1.21). HALT với blocking condition `hop am Mod+Comma da co chu`.
  Xem §Design Notes: story này **không** giành hợp âm đó, và đó là một quyết định của Ice.
- Cần thêm **bất kỳ phụ thuộc mới** nào (NFR15 đòi rà giấy phép trong nguồn đã tải trước).
- Cần một **`AD` mới** hoặc phải đổi một bất biến kiến trúc đang đứng.
- Cần đổi hình dạng dây của `BootstrapConfig` (bảy trường, `tests/ipc_contract.rs` đóng băng) để
  lưu tuỳ chọn đọc xuống đĩa.

**Never:**

- Không dựng đánh dấu *"chỗ cần sửa"* (`M`, FR119, danh sách theo Tác phẩm) — **Story 5.13**.
- Không dựng biên *"chỉ đọc phần đã xong"*, mốc biên tường minh, đọc liên tục xuyên Chương, hay
  gạch chấm cho câu chưa xác nhận (FR120) — **Story 5.12**.
- Không dựng ảnh nhúng / `caption` / alt-text (FR43 · FR129 · FR44).
- Không thanh tiến độ đọc dính trên và không byline xuất xứ ở đầu Chương (mockup có, AC **không**).
- Không lưu tuỳ chọn đọc xuống đĩa ở story này (xem §Block If) — ghi nợ có chủ.
- Không sửa `epics.md`/`prd.md`/`EXPERIENCE.md` cho khớp mã đã viết.
- Không đụng lược đồ `project.db`, `global.db`, `library-index.db`; không bước di trú nào.
- Không tự chuyển chế độ thay người dùng từ một handler phím.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Đọc Chương đang mở | Có Tác phẩm mở, Chương có 5 câu, câu 2 và 4 mang `is_target_paragraph_end = 1` | `read_reading_chapter` trả 3 đoạn: `[1,2]`, `[3,4]`, `[5]` | Không lỗi |
| Câu đã cắt bỏ | Câu 3 có `is_omitted = 1` | Câu 3 **vắng mặt hoàn toàn** — không chỗ trống, không `[…]`, không dấu vết | Không lỗi |
| Câu cắt bỏ ĐANG mang cờ kết đoạn | Câu 2 `is_omitted = 1` **và** `is_target_paragraph_end = 1` | Đoạn vẫn kết sau câu 1 — cờ chuyển cho câu còn sống liền trước; **không** gộp hai đoạn làm một | Không lỗi |
| Cả một đoạn bị cắt bỏ | Đoạn giữa gồm 2 câu, cả hai `is_omitted = 1` | Đoạn ấy biến mất; **không** đoạn rỗng nào lọt ra dây | Không lỗi |
| Câu chưa dịch | `target_text = ''`, không cắt bỏ | Câu vẫn ở trong đoạn với chuỗi rỗng — Chế độ đọc **không** tự bịa nội dung | Không lỗi |
| Chương rỗng | Chương không có `segment` nào | `paragraphs = []`, `chapter_id` vẫn đúng | Không lỗi; màn hình hiện câu *"Chương này chưa có câu nào"* |
| Mọi câu đều cắt bỏ | 4 câu, cả 4 `is_omitted = 1` | `paragraphs = []` — và câu trạng thái phải **khác** ca Chương rỗng | Không lỗi |
| Chưa mở Tác phẩm nào | `OpenWorkState = None` | `IpcError` `err.project.no_work_open` (khoá đã có) | Màn hình hiện lỗi qua `tError()`, **không** `try/catch` ở tầng UI |
| Chạy ngoài Tauri | `npm run dev` trong trình duyệt thường | `{ chapter: null, error: null }` — trạng thái thứ ba, **không** dựng `IpcError` giả | Không dải lỗi nào hiện |
| Thanh trượt giãn dòng kéo xuống đáy | Người dùng đặt `1.2` | Giá trị bị **ghìm ở 1.66**; ô số hiển thị 1.66 | Không lỗi |
| Đổi cỡ chữ khi đang ở mức Cân | 17,5px → 22px | Chiều rộng cột (px) tăng **cùng tỉ lệ**; số ký tự mỗi dòng **không đổi** | Không lỗi |
| Bật song ngữ | `B` | Nguyên văn của **từng đoạn** hiện ở **lề trái**, cỡ nhỏ, `on-surface-variant`; cột đọc giữ nguyên bề rộng `ch` và **không** bị chen dòng | Không lỗi |
| Mở mục lục khi chưa mở Tác phẩm | `Mod+L`, `OpenWorkState = None` | Danh sách rỗng kèm câu nói **vì sao** rỗng; không lượt mở Chương nào phát đi | `IpcError` hiện qua `tError()` |
| Gõ phím trần trong một ô nhập | Con trỏ trong `<input>` của mục lục, bấm `2` | Ký tự `2` hạ cánh bình thường — luật vùng gõ của `keys.ts` chặn hợp âm không mang phím bổ trợ chính | Không lỗi |

</intent-contract>

## Code Map

**Rust — chốt lọc đã có, chưa ai gọi**

- `src-tauri/src/core/segment/omit.rs` — `segments_in_translation(&[ChapterSegment]) -> Vec<&ChapterSegment>`
  (`:58`) và `count_in_translation` (`:71`). 🔴 Doc-comment đầu tệp cấm thẳng một `v-if` ở Vue và
  ghi *"hai bề mặt kia chỉ việc gọi"*. Story này là lượt gọi **đầu tiên** — ⚠️ mệnh đề *"chưa bề
  mặt nào gọi nó"* ở `:1-10` HẾT ĐÚNG sau story này, sửa **tại chỗ** kèm 🔵 và ngày.
- `src-tauri/src/core/segment/paragraph.rs` — **khuôn trực tiếp** cho module thuần mới: doc-comment
  §*"VÌ SAO MODULE NÀY TỒN TẠI"*, `ParagraphFlags` (`struct` chứ không `(bool, bool)`), `merged`
  trả `Option` cho nhóm rỗng (*"không bịa"*). Chép **hình dạng và kỷ luật**, không chép nội dung.
- `src-tauri/src/core/segment/mod.rs:31-35` — danh sách `pub mod`; thêm `pub mod reading;` kèm một
  khối `//!` mô tả vai, đúng khuôn năm khối đã có ở `:6-29`.

**Rust — bề mặt IPC**

- `src-tauri/src/commands/segment.rs` §`ChapterSegment` (`:219`, chín trường, `is_omitted` ở `:219`);
  §`read_open_chapter_segments` (`:852`), **khuôn trực tiếp** cho hàm thuần mới: `open.chapter_id`
  (KHÔNG suy ra bằng một câu SQL thứ hai — 🔵 Quyết định #2(a) của Story 2.11 ghi tại chỗ),
  `WHERE chapter_id = ?1 AND retired_at IS NULL ORDER BY ord, id`, và phép đổi `INTEGER 0/1 → bool`
  cho ba cột cờ.
- ⚠️ `commands/segment.rs` **không** nằm trong bảng `count_async_attrs` của
  `src-tauri/tests/config_invariants.rs:1037-1058` (chỉ `glossary.rs` 7 · `library.rs` 4 ·
  `chapter.rs` 4). Một vỏ `#[tauri::command]` **không** `(async)` ở đây là một `SELECT` nhẹ, cùng
  hạng `read_open_chapter_segments` — không thêm tên vào bảng đó.
- `src-tauri/src/commands/chapter.rs` — `no_work_open()` (dùng lại nguyên, khoá `err.project.no_work_open`
  đã có), và `ChapterRow`/`list_chapters` cho mục lục (không đụng).
- `src-tauri/src/lib.rs:382` — `crate::commands::segment::wire::read_open_chapter_segments` trong
  `generate_handler!`; thêm vỏ mới **ngay cạnh**.

**Rust — cổng và bàn đo**

- `src-tauri/tests/ipc_contract.rs:540-591` — khuôn đóng băng khoá `snake_case` của một struct dây
  (`SearchHit`/`SearchReport`); `:761-790` — khuôn ca *"vỏ đã đăng ký và giữ nguyên tên tham số"*.
  Hai khuôn này áp cho `ReadingParagraph`/`ReadingChapter` và vỏ `read_reading_chapter`.
- `src-tauri/tests/segment_contract.rs` — tệp nhận bộ ca hành vi (lọc cắt bỏ, chuyển cờ kết đoạn,
  Chương rỗng, mọi câu cắt bỏ). Ca `..._carries_the_is_omitted_column_over_the_wire`
  (`commands/segment.rs:196` trỏ tới) là khuôn đặt tên: **một CÂU khẳng định**, không `test_foo`.
- `src-tauri/tests/segment_boundary.rs` — ranh giới *"chỉ `split` biết bảng chữ cái kết câu"*;
  module mới **không** được mang từ vựng đó.

**Frontend — nơi story cắm vào**

- `src/modes/ReadingMode.vue` (**50 dòng, toàn bộ tệp**) — khung rỗng; `:1-10` doc-comment tự khai
  *"KHUNG RỖNG có chủ ý … toàn bộ thuộc Epic 5 (UX-DR46)"* và ⚠️ *"ngày Epic 5 đổ chữ THẬT vào đây,
  bề mặt này phải khai token `read-*` của chính nó — mặc định của `body` là `ui-md` ở giãn dòng
  **1.5**, DƯỚI sàn 1.66, và không phép kiểm nào canh được chỗ đó"* — **sửa tại chỗ** kèm 🔵 khi viết
  lại. `declareFocus('mode.reading', …)` (`:18`) · `releaseFocus` (`:21`) · `onActivated →
  enterFocus` (`:23-24`) giữ nguyên khuôn.
- `src/modes/LibraryMode.vue` §`.filter-actions` (`:637`, bốn nút `data-lifecycle-filter` mang
  `aria-pressed` — mỗi lựa chọn một `dispatch` id RIÊNG, **không** một nút "đảo"; chú thích 🔴 ở
  `:634-636` nói vì sao **không** dùng `v-for` chọn id động) và các node `role="status"` LUÔN có mặt
  (`:323` · `:355` · `:367`, khuôn *"không `v-if` trên chính node trạng thái"*).
- `src/modes/modeState.ts` — `currentMode`/`setMode`; `App.vue:302-306` `<KeepAlive>` giữ ba chế độ,
  nên **`onActivated`** chứ không `onMounted` là điểm vào của lượt hiện thứ hai trở đi.
- `src/panels/editorPanelState.ts:1766-1848` — `openChapterById(chapterId, segmentId?)`: flush →
  `openChapter` → `resetEditorPanel`/`resetSourcePanel` → `ensureSegmentsLoaded`. ⚠️ Bước cuối
  (`:1843-1844`) gọi `enterFocus('panel.grid')` — trong Chế độ đọc panel đó **không có trong DOM**,
  nên mục lục phải `enterFocus('mode.reading')` **sau** khi hàm này trả `true`.
- `src/config/chapter.ts:224` `listChapters()` · `:100-108` `ChapterRow` (`title: string | null`) ·
  `:251` `openChapter(chapterId)`. Mục lục dùng lại **nguyên**, không lệnh IPC mới.
- `src/config/library.ts` §`isSearchHit` (`:552`, kiểm **MỌI** trường) · §`isSearchHitArray` (`:576`,
  kiểm **MỌI** phần tử) · §`isSearchReport` (`:580`) · §`searchLibrary` (`:603`) — khuôn adapter mới.
- `src/config/bootstrap.ts:156` `SCOPE_APP_CONFIG` · `:273` `putConfig(kind, key, value)`.
  ⚠️ **Không** có hằng `KEY_THEME` phía TS hôm nay — thêm một hằng, cùng lý do bốn hằng đã có
  (`:156-185`): `put_config` nhận `key` là **chuỗi trên dây**, một lỗi gõ không kiểu nào bắt được.
  Phía Rust `core/scope/store.rs:49` khai `KEY_THEME: &str = "theme"`, và
  `save_value` (`:383`) **không** có danh sách khoá trắng — chỉ đòi `ScopeKind` là `GlobalOnly`.
- `src/tokens/index.ts:73` `applyTheme(theme, root?)` — ghi trọn bộ biến rồi đặt `root.dataset.theme`;
  gọi lại được nhiều lần có chủ ý (`:57-62`). `src/main.ts:331` là lượt gọi duy nhất hôm nay.
- `src/commands/index.ts` — khối *"STORY 5.10 — HAI CHẾ ĐỘ DẤU"* (`:1294` trở đi) là chỗ chèn khối
  Story 5.11 ngay sau; `CommandDeps` cụm Library (`:322-345`) là khuôn khai dep;
  §`shortcuts.open` (`:2509`) là **chỗ `Mod+Comma` đang có chủ**.
- `src/main.ts:100-106` (import) · `:406-409` (tiêm dep) — **một khối một Story**, chép khuôn.
- `src/i18n/vi.json:277` — `mode.reading.status`, **khoá duy nhất** của chế độ này hôm nay; khối
  `mode.library.*` (`:240-273`) là khuôn đặt khoá.

**Cổng — số đo tại baseline `fc7ca89`**

- `scripts/check-commands.mjs:319` `COMMAND_FLOOR = 52` · `:352` `CLICK_FLOOR = 27` · `:363`
  `DISPATCH_FLOOR = 40` — đều là cận **DƯỚI**, thêm lệnh/nút không làm đỏ.
  Kiểm A: `@click` = đúng một `dispatch('<id>')` literal. Kiểm E: mọi `owner` được dùng phải có một
  `declareFocus()` literal ở đâu đó (`:1900-1945`).
- `src/commands/keys.ts:401-415` `lacksPrimaryMod` + `:418-437` `isTypingZone` — 🔴 doc-comment
  **gọi đích danh** `M`, `B`, `1 2 3` trần của UX-DR46 là lý do luật vùng gõ tồn tại. `createKeymap`
  (`:466-497`) **ném** khi hai command giành một hợp âm.
- Hợp âm đã chiếm (đo 2026-08-30, `grep -o "'Mod[^']*'" src/commands/index.ts | sort -u`):
  `Mod+Comma` · `Mod+D` · `Mod+Enter` · `Mod+H` · `Mod+M` · `Mod+Slash` · `Mod+Shift+Slash` ·
  `Mod+1..3` (chế độ) · họ `Mod+Alt+…`. **`Mod+L` trống**; **không hợp âm TRẦN nào** ngoài
  `Shift+Arrow*`/`Alt+Shift+Arrow*` ⇒ `B` · `D` · `1` · `2` · `3` đều trống.
- `scripts/check-tokens.mjs:849` Kiểm B (màu viết thẳng) · `:1089` Kiểm C (tương phản, **mọi** tổ hợp
  `roles.text × roles.surface` đã khai sẵn ⇒ dùng token có sẵn thì **0 cặp mới**) · `:1361` Kiểm D
  (không `opacity` lùi chữ) · `:1415` **Kiểm E chỉ đọc `tokens.json`** ⇒ sàn 1.66 lúc chạy
  **không có cổng nào canh** · `:1459` Kiểm F (không elevation) · `:1502` Kiểm H (`outline: none`).
- `scripts/check-i18n.mjs:866` Kiểm A (không chữ tiếng Việt có dấu ở vị trí mã trong `.rs`/`.vue`) ·
  `:927` Kiểm A2 (mọi text node qua `t()`, thoát bằng `<!-- aura-allow-text: … -->`) · `:1187`
  Kiểm D (cấm *"bạn"*, *"chúng tôi"*). `RS_FLOOR = 44` (`:288`) · `VUE_FLOOR = 16` (`:310`) — cận dưới.
- `scripts/check-panel-refs.mjs:125` `EXEMPT` — khuôn miễn trừ CÓ TÊN; ba luật ở `:118-123`
  (có tên · có lý do tại chỗ · **chết được**: một miễn trừ trỏ vào ô không còn tồn tại làm cổng ĐỎ).
  `RE_REACTIVE` (`:352`) neo đầu dòng ⇒ **mỗi khai báo một dòng**.
- `scripts/check-layout.mjs:417-491` `ALLOWED_GLOBAL_MEMBERS` — thêm một thành viên `window.`/
  `document.` là một quyết định phải viết ra. Cuộn bằng `el.scrollIntoView()` trên một `ref` **không**
  đi qua danh sách này.
- `e2e/specs/editor-typing-flush.e2e.mjs:38-56` — 🔴 **GIỚI HẠN ĐÃ ĐO: `browser.keys()` không gõ được
  chữ**; đường duy nhất đưa `target_text` thật vào WKWebView là `document.execCommand('insertText')`
  trên ô `contenteditable` (`:162-182`, `:270`). Đây là thứ **mở khoá** phép đo mà §GIỚI HẠN của
  Story 5.10 nói bàn đo e2e không làm được.
- `e2e/support/workspace.mjs:57` `openWorkspaceWithWork(...)` · `:128` `browser.keys(['Meta','2'])`;
  `e2e/support/pointer.mjs` `realClick` (🔴 cấm `.click()` của driver).
  ⚠️ `story-5-6-library-grid.e2e.mjs` đỏ từ baseline (chủ Story 5.6) — **không** đọc thành hồi quy.
- `_bmad-output/implementation-artifacts/deferred-work.md:3743` — mục 🟡 *"Chế độ đọc → Epic 5
  (Story 5.11 · 5.12 · 5.13)"*, vế Chế độ đọc của AC5 Story 2.5c: story này đóng nó.
  `:5030-5032` — món nợ *"đổi CHẾ ĐỘ Workspace ↔ Chế độ đọc giữ đúng Chương/câu/vị trí cuộn"*,
  **chủ: story dựng Chế độ đọc (5.11–5.13)**: story này đóng vế *đúng Chương*, thu hẹp phần còn lại.

## Tasks & Acceptance

**Execution:**

1. `src-tauri/src/core/segment/reading.rs` (**mới**) — hàm thuần
   `paragraphs_in_translation(&[ChapterSegment]) -> Vec<Vec<&ChapterSegment>>`: gọi
   `omit::segments_in_translation` (**không** chép lại vị từ `!is_omitted`), rồi cắt đoạn theo
   `is_target_paragraph_end` **của câu còn sống**; một cờ kết đoạn nằm trên câu bị cắt bỏ **chuyển**
   cho câu còn sống liền trước; **không** trả đoạn rỗng nào. Doc-comment đầu tệp chép kỷ luật của
   `paragraph.rs`: vì sao hàm này ở Rust chứ không một `computed` ở Vue, và phép đo *"cắt bỏ nuốt
   mất một ranh giới đoạn"*. — Rationale: một quy tắc có ca biên thật thì không được sống ở webview
   (AD-1), và chốt lọc phải có **đúng một** vị từ.
2. `src-tauri/src/core/segment/mod.rs` — `pub mod reading;` + một khối `//!` khai vai, đúng khuôn năm
   khối đã có. — Rationale: `mod.rs` của module này là mục lục có lý do, không một danh sách tên.
3. `src-tauri/src/core/segment/omit.rs` — **sửa tại chỗ** kèm 🔵 + ngày: mệnh đề *"chưa bề mặt nào
   gọi nó"* và bảng *"cả hai bề mặt đó là khung rỗng"* hết đúng cho vế Chế độ đọc (vế bản xuất
   **vẫn** đúng, Epic 8). — Rationale: mệnh đề hết đúng thì sửa, đừng để nó lặng lẽ sai.
4. `src-tauri/src/commands/segment.rs` — ① `ReadingSegment` (`id`, `source_text`, `target_text`) ·
   `ReadingParagraph { segments }` · `ReadingChapter { chapter_id, chapter_ord, chapter_title,
   paragraphs }`, `snake_case` trên dây, **không** `#[serde(rename_all = "camelCase")]`;
   ② hàm thuần `read_reading_chapter(open: Option<&OpenWork>) -> Result<ReadingChapter, IpcError>` —
   dùng lại `no_work_open()`, đọc `segment` **và** `chapter.ord`/`chapter.title` trong **cùng một**
   `Store::read` (cùng lý do §Story 5.7 đã ghi cho `caret_segment_id`), rồi gọi Task 1;
   ③ vỏ `wire::read_reading_chapter` (`#[tauri::command]`, **không** `(async)`) lấy `State` qua
   `try_state`. — Rationale: khuôn hai lớp là điều kiện để `tests/**` gọi được không cần webview.
5. `src-tauri/src/lib.rs` — thêm `crate::commands::segment::wire::read_reading_chapter` vào
   `generate_handler!`, ngay cạnh `read_open_chapter_segments`. — Rationale: tên trên dây LÀ tên hàm.
6. `src-tauri/tests/segment_contract.rs` — bộ ca hành vi, mỗi tên là một CÂU: đoạn cắt đúng theo cờ
   đích · câu cắt bỏ vắng mặt hoàn toàn · **cờ kết đoạn trên một câu cắt bỏ vẫn giữ được ranh giới** ·
   cả một đoạn cắt bỏ ⇒ không đoạn rỗng · Chương rỗng ⇒ `paragraphs` rỗng · mọi câu cắt bỏ ⇒
   `paragraphs` rỗng · chưa mở Tác phẩm ⇒ `err.project.no_work_open`. — Rationale: bảng I/O phải có
   chủ ở tầng gọi được, không ở màn hình.
7. `src-tauri/tests/ipc_contract.rs` — đóng băng khoá `snake_case` của ba struct mới và khẳng định vỏ
   `read_reading_chapter` đã đăng ký, chép khuôn `:540-591` và `:761-790`. — Rationale: hai nửa của
   một hợp đồng dây phải có một chỗ giữ chúng khớp nhau.
8. `src/config/reading.ts` (**mới**) — adapter IPC: `ReadingSegment`/`ReadingParagraph`/`ReadingChapter`
   + `isReadingChapter` kiểm **MỌI** trường và **MỌI** phần tử (khuôn `config/library.ts:519-563`) +
   `readReadingChapter(): Promise<{ chapter: ReadingChapter | null; error: IpcError | null }>`,
   một `invoke`, một `try/catch`, **không ném**, phân biệt đủ **ba** trạng thái. — Rationale: dữ liệu
   qua dây là một lời khai, không một bảo đảm của trình biên dịch.
9. `src/config/bootstrap.ts` — thêm hằng `KEY_THEME = 'theme'` kèm doc-comment cùng khuôn bốn hằng đã
   có. — Rationale: `put_config` nhận khoá là chuỗi trên dây; một lỗi gõ chỉ im lặng biến mất.
10. `src/tokens/themeState.ts` (**mới**) — `currentTheme` (`ref`, khởi tạo `DEFAULT_THEME`),
    `setTheme(theme)` (chốt bằng `isTheme`, gọi `applyTheme`, `void putConfig(SCOPE_APP_CONFIG,
    KEY_THEME, theme)`), `toggleTheme()`, `resetThemeState()`. Miễn trừ `check:panel-refs` **không**
    cần nếu `resetThemeState()` gán lại `currentTheme`; nếu chọn miễn trừ thì phải CÓ TÊN kèm lý do
    *"lựa chọn ứng dụng, loại chỉ-toàn-cục của AD-18"* — cùng hạng ba miễn trừ ở
    `check-panel-refs.mjs` §*"BA loại chỉ-toàn-cục của AD-18"* (`:164-186`). — Rationale: hôm nay **0** đường đổi theme lúc chạy;
    `main.ts:331` chỉ áp một lần lúc khởi động.
11. `src/main.ts` — gọi `setTheme(...)`/khởi tạo `currentTheme` từ `config.theme` **thay cho**
    `applyTheme(...)` trực tiếp, giữ nguyên 🔴 thứ tự *`applyTheme()` trước `mount()`*; thêm một
    khối import + một khối tiêm dep cho Story 5.11. — Rationale: thứ tự khởi động là bắt buộc, và
    một nháy trắng chỉ lộ ra trên máy người khác.
12. `src/modes/readingState.ts` (**mới**) — state cấp module, **mỗi khai báo một dòng**:
    ① nội dung — `chapter`, `hasLoaded`, `pending`, `loadError`, `sequence`, `requested`,
    `ensureReadingLoaded()` (idempotent, có số thứ tự lượt nạp đúng khuôn `ensureSegmentsLoaded`),
    `reloadReading()`, `resetReading()`;
    ② mục lục — `tocOpen`, `tocChapters`, `tocHaveLoaded`, `tocBusy`, `tocError`, `tocCursor`,
    `openTableOfContents()`, `closeTableOfContents()`, `nextTocChapter()`, `prevTocChapter()`,
    `openCurrentTocChapter()` (gọi `openChapterById` rồi `reloadReading()` rồi
    `enterFocus('mode.reading')`), gom trong `resetReadingToc()`;
    ③ typography — `readingLevel` (`'lg' | 'md' | 'sm'`, mặc định `'md'`), `bilingual`,
    `tunerOpen`, `fontSizeOverride`, `lineHeightOverride`, `setReadingLevel()`,
    `toggleBilingual()`, `toggleTuner()`, `setFontSize()`, `setLineHeight()` (🔴 **ghìm ≥ 1.66**),
    `readingStyle` (computed → `font-size` px, `line-height` không đơn vị, `max-width` **`ch`**).
    Ba ô của nhóm ③ là **tuỳ chọn ứng dụng**, không theo Tác phẩm ⇒ hoặc đi qua một
    `resetReadingPreferences()` riêng, hoặc EXEMPT CÓ TÊN — **không** đưa vào `resetReading()`.
    — Rationale: một vị từ `…HasLoaded` phải tồn tại trước khi một danh sách rỗng được phép nói
    *"không có gì"* (lớp lỗi *rỗng im lặng*, đã hụt ba lần).
13. `src/modes/libraryChapters.ts` · `src/modes/libraryImport.ts` — gọi `resetReading()` ở **đúng**
    những chỗ đang gọi `resetEditorPanel()`/`resetSourcePanel()` khi đổi Tác phẩm/Chương.
    — Rationale: một ô nhớ của Chương cũ sống sót sang Tác phẩm mới là cùng lớp lỗi mà cổng thứ mười
    tồn tại để chặn.
14. `src/modes/ReadingMode.vue` — viết lại: thanh công cụ (công tắc song ngữ · ba preset
    `aria-pressed` · nút tinh chỉnh · nút sáng/tối · nút mục lục), khối tinh chỉnh (hai
    `<input type="range">`, `min="1.66"` cho giãn dòng), trang đọc (một `<p>` mỗi đoạn, một
    `<span>` mỗi câu), lề trái song ngữ (nguyên văn **theo đoạn**, cỡ nhỏ,
    `var(--color-on-surface-variant)`, không chen dòng đọc), lớp phủ mục lục, và một `role="status"`
    phân biệt được các ca: chưa nạp · đang nạp · chưa mở Tác phẩm · Chương rỗng · mọi câu đã cắt bỏ ·
    có nội dung · lỗi. **Sửa tại chỗ** kèm 🔵 khối doc-comment `:1-10` (*"khung rỗng có chủ ý"* và
    ⚠️ *"`body` chạy ở 1.5"* — bề mặt này nay khai token `read-*` của chính nó).
    — Rationale: một danh sách rỗng không tự nói vì sao nó rỗng.
15. `src/modes/ReadingMode.vue` `<style scoped>` — chỉ token; `max-width: var(--space-read-measure-*)`
    đặt trên **chính** phần tử mang `font-size` đọc; giãn dòng không đơn vị; không `box-shadow`,
    không gradient, không `opacity` lùi chữ; `outline: none` chỉ trên gốc `tabindex="-1"`.
    — Rationale: `ch` tính theo cỡ chữ của **chính phần tử đó** — đặt hai thứ ở hai nơi là phá đúng
    điều ba mức tồn tại để làm.
16. `src/commands/index.ts` — khối Story 5.11 sau khối 5.10: `reading.toggle_bilingual` (`B`) ·
    `reading.level_airy` (`1`) · `reading.level_balanced` (`2`) · `reading.level_dense` (`3`) ·
    `reading.toggle_tuner` (**không hợp âm mặc định** — xem §Design Notes) · `reading.toggle_theme`
    (`D`) · `reading.toc` (`Mod+L`) · `reading.toc_next`/`toc_prev`/`toc_open`/`toc_close`
    (không hợp âm). Mỗi id một dep tuỳ chọn trong `CommandDeps` + `portMissing` khi vắng.
    🔴 **Không** đụng `:2509` `['shortcuts.open', 'openShortcuts', 'Mod+Comma']`. — Rationale: hai
    command giành một hợp âm thì `createKeymap` ném, và ứng dụng không khởi động được.
17. `src/i18n/vi.json` — khoá mới `mode.reading.*` và `command.reading.*`; **sửa tại chỗ**
    `mode.reading.status` (`:277`) nếu câu cũ hết đúng. Câu vô nhân xưng, không *"bạn"*/*"chúng tôi"*.
    — Rationale: Kiểm A2 đòi mọi text node qua `t()`, Kiểm D chấm giọng văn bằng máy.
18. `tests/frontend/readingTypography.test.ts` (**mới**) — ba mức cho đúng bộ ba
    (62ch/19px/1.95 · 68ch/17,5px/1.8 · 76ch/16px/1.66) đọc **từ `tokens.json`**, không từ số viết
    thẳng trong test; 🔴 `setLineHeight(1.2)` ⇒ `1.66`; `setFontSize` **không** đổi giá trị `ch`;
    `readingStyle.maxWidth` luôn kết thúc bằng `ch` và **không** chứa `px`; công tắc song ngữ đổi
    đúng một ô; `resetReading()` dọn **mọi** ô nội dung và **không** dọn ba ô tuỳ chọn.
    — Rationale: sàn 1.66 lúc chạy không đi qua cổng nào; đây là lưới duy nhất của nó.
19. `tests/frontend/readingState.test.ts` (**mới**) — với `invoke` giả: câu `is_omitted` **không bao
    giờ** tới được webview (bàn đo trả dữ liệu đã lọc, và test khẳng định component **không** tự lọc
    lần hai — gỡ vế lọc ở fixture ⇒ câu ấy hiện ra ⇒ chứng minh Vue **không** có đường lọc riêng);
    ba trạng thái của adapter; các nhánh `role="status"` phân biệt được nhau.
    — Rationale: đối chứng phải là một phép GỠ thật, không một lượt chèn thêm.
20. `e2e/specs/story-5-11-reading-mode.e2e.mjs` (**mới**) — trong WKWebView thật: mở Tác phẩm, mở
    Chương vào Workspace, **gõ bản dịch thật bằng `document.execCommand('insertText')`** vào ít nhất
    hai câu (khuôn `editor-typing-flush.e2e.mjs:162-182`, và ⚠️ giới hạn `browser.keys()` ghi ở `:38-56`), flush, `⌘3`; rồi đo:
    ① chữ vừa gõ hiện trên trang đọc; ② **không** phần tử biên tập nào trong cây
    (`[data-col]` · `contenteditable` · nút xác nhận) = 0; ③ bấm `2` rồi `1`: `getBoundingClientRect().width`
    và `font-size` đổi **cùng tỉ lệ** ⇒ số ký tự mỗi dòng giữ nguyên; ④ `B` bật song ngữ, nguyên văn
    nằm **bên trái** cột đọc (`rect.left` nhỏ hơn) và cột đọc **giữ nguyên** bề rộng; ⑤ `D` đổi
    `document.documentElement.dataset.theme`. — Rationale: mọi mệnh đề về **hình học** thuộc engine
    thật; `happy-dom` không bố cục.
21. `_bmad-output/implementation-artifacts/deferred-work.md` — đóng bằng chữ mục *"Chế độ đọc →
    Epic 5"* (`:3743`) theo khuôn `→ ✅ ĐÃ ĐÓNG <ngày> (Story 5.11)`; thu hẹp mục vị trí đọc
    (`:5030-5032`) thành phần còn hở thật (**vị trí cuộn** khi đổi chế độ — story này đóng vế *đúng
    Chương*); mở hai mục mới có chủ: **⌘, chưa gán cho tinh chỉnh** (chủ **Ice**) và **tuỳ chọn đọc
    không lưu xuống đĩa** (chủ: story mở lại hình dạng `BootstrapConfig`). — Rationale: không bao giờ
    xoá một mục đã đóng, và một năng lực chưa dựng là một món nợ có chủ, không một lượt sửa spec.
22. `src/modes/README.md` — bảng sở hữu (*"Nội dung Chế độ đọc … **Epic 5** ⬜"*) **sửa tại chỗ**
    kèm 🔵: phần typography/song ngữ/ba mức nay ✅ ở 5.11; đánh dấu và biên *"đã xong"* vẫn ⬜.
    — Rationale: một bảng sở hữu nói sai là chỗ story sau chép sai.

**Acceptance Criteria:**

- **Given** một Chương có câu đã cắt bỏ, **when** mở Chế độ đọc, **then** những câu ấy không để lại
  một dấu vết nào trên trang (không `[…]`, không chỗ trống, không phần tử rỗng), **and** phép lọc
  chạy ở Rust — gỡ lời gọi `omit::segments_in_translation` làm bộ ca Rust ĐỎ, còn `src/**` không có
  một đường lọc `is_omitted` nào.
- **Given** Chế độ đọc đang mở, **when** soi cây DOM, **then** không có `contenteditable`, không vạch
  lề segment, không nút xác nhận, không panel nào.
- **Given** ba nút preset, **when** chọn lần lượt Thoáng · Cân · Đặc, **then** cột đọc mang đúng
  (62ch, 19px, 1.95) · (68ch, 17,5px, 1.8) · (76ch, 16px, 1.66), lấy từ token chứ không từ số viết
  thẳng trong component, **and** Cân là mức mặc định lúc mở lần đầu.
- **Given** thanh trượt tinh chỉnh, **when** đặt giãn dòng dưới 1.66 bằng bất kỳ đường nào (kéo,
  gõ số, gọi hàm), **then** giá trị áp lên trang là 1.66.
- **Given** mức Cân, **when** đổi cỡ chữ bằng thanh trượt, **then** bề rộng cột (px) đổi **cùng tỉ lệ**
  với cỡ chữ ⇒ số ký tự mỗi dòng không đổi (đo trong WKWebView thật).
- **Given** mặc định, **when** mở Chế độ đọc, **then** chỉ bản dịch tiếng Việt hiện ra.
- **Given** công tắc song ngữ bật, **when** trang render, **then** nguyên văn của mỗi đoạn nằm ở lề
  **trái** cột đọc, cỡ nhỏ hơn, màu `on-surface-variant`, **and** cột đọc giữ nguyên bề rộng — nguyên
  văn không chen vào giữa dòng đọc.
- **Given** bàn phím, **when** bấm `B` · `1` · `2` · `3` · `D` · `⌘L` ngoài một vùng gõ, **then** lần
  lượt bật/tắt song ngữ · ba mức chữ · đổi sáng/tối · mở mục lục; **and** cùng những phím ấy gõ **trong**
  một vùng gõ hạ cánh thành ký tự bình thường.
- **Given** mục lục đang mở, **when** chọn một Chương và xác nhận, **then** Chế độ đọc hiển thị Chương
  đó, **and** tiêu điểm ở lại trong Chế độ đọc (không rơi về `body`, không nhảy sang panel lưới).
- **Given** thao tác tinh chỉnh, **when** người dùng chỉ dùng bàn phím, **then** tới được bằng `Tab` +
  `Enter`/`Space` (NFR17) — hợp âm mặc định của nó **cố ý** để trống, xem §Design Notes.
- **Given** cả hai theme, **when** `check:tokens` chạy, **then** Kiểm B/C/D/F/H xanh và **không** cặp
  màu mới nào được thêm — bề mặt này chỉ dùng token đã kiểm.
- **Given** một Chương rỗng và một Chương mà mọi câu đều đã cắt bỏ, **when** mở Chế độ đọc, **then**
  hai câu trạng thái **khác nhau** hiện ra, và không câu nào khẳng định *"chưa có gì"* trong lúc lượt
  nạp còn đang bay.

## Spec Change Log

- 🔵 **2026-08-30 — mệnh đề ③ của bàn đo e2e trong §Verification/§Tasks 20 ĐO SAI THỨ, sửa cách
  đo chứ không sửa AC.** Spec viết *"bấm `2` rồi `1`: `getBoundingClientRect().width` và
  `font-size` đổi **cùng tỉ lệ**"*. Câu đó **không đo được điều AC đòi**: đổi preset đổi **cả hai**
  — số `ch` (68 → 62) **và** cỡ chữ (17,5px → 19px) — nên hai tỉ lệ không có lý do gì bằng nhau.
  Đo tại lượt cài: tỉ lệ bề rộng ≈ **0,990** trong khi tỉ lệ cỡ chữ ≈ **1,086**. AC thì nói
  *"số ký tự mỗi dòng giữ nguyên **khi đổi cỡ chữ**"* — tức phép đo đúng là **giữ nguyên mức** và
  kéo thanh trượt cỡ chữ. ⇒ Bàn đo e2e đổi sang đo bằng thanh trượt trong **cùng một mức**; AC
  **không đổi một chữ**, và đối chứng ĐỎ #4 (`ch` → `px`) vẫn phá đúng phép đo mới. Cùng khuôn
  *"spec đếm sai trên một tiền đề cũ"* đã ghi ở Story 5.10 — sửa con số/phép đo trong Task, không
  sửa `epics.md`.
- 🔵 **2026-08-30 — hai hàng của §I/O Matrix ban đầu KHÔNG có ca phủ; đã bù, không hạ kỳ vọng.**
  Lượt cài đầu để trống *"câu chưa dịch vẫn ở trong đoạn"* và *"mở mục lục khi chưa mở Tác phẩm"*.
  Đã thêm `segment_contract.rs::an_untranslated_sentence_stays_in_its_paragraph_with_an_empty_string`
  và hai ca mục lục ở `readingState.test.ts`, mỗi ca kèm một đối chứng ĐỎ đã chạy.
  Hàng *"gõ phím trần trong một ô nhập"* nói *"ô nhập **của mục lục**"* — mục lục đã dựng **không
  có** `<input>` nào, nên chữ ấy không có nơi cắm. Mệnh đề THẬT của hàng đó (một hợp âm TRẦN của
  story này không bắn trong vùng gõ) nay có ca riêng trên **bộ command THẬT**
  (`readingState.test.ts` §*"hợp âm TRẦN của Chế độ đọc đi qua luật vùng gõ"*) — khác `check:commands`
  Kiểm D, vốn chỉ lái cơ chế trên một registry GIẢ.
- 🔵 **2026-08-30 — bàn đo e2e ĐÃ CHẠY THẬT, và ba lượt ĐỎ liên tiếp trả về ba kết luận KHÁC
  nhau; ghi cả ba vì hai trong số đó là bài học, không một chi tiết.**
  **① Lượt một — lỗi BÀN ĐO.** Trang đọc chỉ có câu MỘT
  (`"Chương 1Ban dich cau mot cho Story 5.11."`). `typeInto(cells[1])` dời caret, mà *"rời
  segment"* là một trong bốn cửa flush NGAY của AD-35 — nên `waitForFlushAfter` xanh ở lượt flush
  của câu MỘT trong khi câu HAI còn trong bộ đệm. Vá bằng **hai lượt chờ nối tiếp** (mốc trả về
  của lượt một là baseline của lượt hai); **không** một `pause()` nào, **không** hằng số nào bị nới.
  **② Lượt hai và ba — mệnh đề ③ của bàn đo ĐO SAI BẤT BIẾN, lần thứ hai trong cùng một story.**
  Spec đòi *"bề rộng và cỡ chữ đổi CÙNG TỈ LỆ"*; đo trong WKWebView cho lệch 3,72 % rồi 3,33 %.
  Số đo lấy thẳng từ engine nói vì sao: `1ch` **không tuyến tính** theo `font-size` — tỉ lệ
  `ch`/`px` là **0,51208** ở 17,5px và **0,49849** ở 22px (WebKit làm tròn bề ngang glyph theo
  từng cỡ chữ). ⇒ *"hai tỉ lệ bằng nhau"* là một mệnh đề sai về ENGINE, không về sản phẩm. Bất
  biến ĐÚNG là chữ của AC: **bề rộng / `1ch` giữ nguyên**. Bàn đo đổi sang đo đúng thứ đó, cộng
  một mệnh đề thứ hai — con số ấy phải LÀ **68**, không một con số tình cờ.
  **③ Và chính mệnh đề thứ hai ấy bắt một khuyết tật SẢN PHẨM thật.** Thước đọc buộc vào
  `max-width`, còn `.column` mang `flex: 0 0 auto` ⇒ cột co về bề rộng NỘI DUNG. Đo bằng một phép
  GỠ thật (hoàn nguyên đúng một dòng rồi chạy lại): **57,27 ch** ở 17,5px và **57,09 ch** ở 22px —
  đọc ở 57 ký tự mỗi dòng, không phải 68 mà `tokens.json` và `epics.md` khai. `check:tokens` không
  thấy (nó soi `tokens.json`, không soi bố cục), `happy-dom` cũng không (nó không bố cục).
  ⇒ Thước chuyển sang `width`, `max-width: 100%` ở lại làm rào cửa sổ hẹp; trường
  `readingStyle.maxWidth` đổi tên thành `.measure` vì tên cũ nói sai thứ nó là. Sau vá:
  **68,00 ch** ở cả hai cỡ chữ.
- 🔵 **2026-08-30 — §I/O Matrix gọi SAI TÊN khoá lỗi, và mã thì gọi ĐÚNG.** Hàng *"Chưa mở Tác
  phẩm nào"* viết `err.project.no_work_open`. Khoá thật, đọc thẳng từ
  `src-tauri/src/commands/chapter.rs::no_work_open()`: `code` là **`work.none_open`** và
  `message_key` là **`MessageKey::WorkNoneOpen`** (`err.work.none_open`) — sai ở cả hai nửa của
  cái tên. Cài đặt dùng lại đúng `no_work_open()` như §Tasks 4 đòi, nên **hành vi không lệch một
  ly**; chỉ chữ trong bảng sai. Ghi tại đây thay vì sửa bảng: §I/O Matrix nằm trong
  `<intent-contract>`, và hợp đồng đó là chỉ-đọc từ bước cài trở đi.

## Review Triage Log

### 2026-08-30 — Review pass

- intent_gap: 0
- bad_spec: 0
- patch: 13: (high 2, medium 6, low 5)
- defer: 3: (high 0, medium 2, low 1)
- reject: 2: (high 0, medium 0, low 2)
- addressed_findings:
  - `[high]` `[patch]` Hai câu liền nhau trong một đoạn render DÍNH vào nhau (`<span>` cạnh
    `<span>` không gì ngăn) — thêm một dấu cách ở đầu mọi câu trừ câu đầu, kèm ca vitest so
    chuỗi HIỂN THỊ (`toContain` mù với lỗi này) và một đối chứng ĐỎ đã chạy.
  - `[high]` `[patch]` `resetReadingToc()` được khai mà **không một chỗ gọi nào** — mục lục
    sống sót qua lượt đổi Tác phẩm và cú "Mở" phát một `chapter_id` của kho khác (`chapter.id`
    là số nguyên CỤC BỘ theo AD-3 nên nó trùng một Chương có thật). Nối vào cả ba chỗ gọi
    `resetReading()`, cộng một ca kiểm KHAI BÁO trên cây nguồn + đối chứng ĐỎ.
  - `[medium]` `[patch]` `main.ts` gọi `setTheme()` lúc khởi động ⇒ một lượt ghi `global.db` ở
    MỖI lần mở app, và *"chưa ai chọn theme"* bị biến thành một giá trị đã lưu tường minh —
    xoá một trong ba trạng thái mà `bootstrap.ts` phân biệt bằng SỰ CÓ MẶT CỦA KHOÁ. Tách
    `initTheme()` (áp, không ghi) khỏi `setTheme()` (áp và ghi).
  - `[medium]` `[patch]` Đường ghi khoá `theme` có 0 ca kiểm (`grep -rln themeState tests/ e2e/`
    = 0); một lỗi gõ `KEY_THEME`/scope biến mất im lặng, đúng thứ doc-comment của chính hằng đó
    cảnh báo. Thêm `tests/frontend/readingTheme.test.ts` theo khuôn `glossarySettings.test.ts`.
  - `[medium]` `[patch]` Nhánh THÀNH CÔNG của `openCurrentTocChapter()` — nguyên một dòng AC —
    không ca nào chạm. Thêm ca khẳng định nạp lại nội dung, đóng lớp phủ, giữ tiêu điểm.
  - `[medium]` `[patch]` Lượt `listChapters()` phụ trượt ⇒ trang rỗng bị dán nhãn
    *"Chương này chưa có câu nào"* cho một Chương đã cắt bỏ hết. Thêm trạng thái thứ TÁM
    `empty-unknown` + khoá `vi.json` + ca kiểm.
  - `[medium]` `[patch]` Lớp phủ mục lục: không nhận tiêu điểm lúc mở, không `Escape` để đóng,
    không trả tiêu điểm lúc đóng (ba lỗ NFR17). Thêm watcher `focus()`/`enterFocus` và
    `@keydown.esc` TẠI CHỖ — không một hợp âm `Escape` TRẦN toàn ứng dụng, vì registry không
    có phạm vi theo chế độ.
  - `[medium]` `[patch]` Lề song ngữ mang `aria-hidden="true"` — một cột chữ người dùng bật lên
    có chủ ý mà trình đọc màn hình không với tới. Gỡ.
  - `[low]` `[patch]` Hàng đang chọn của mục lục chỉ báo bằng MÀU NỀN — thêm `aria-current`.
  - `[low]` `[patch]` `setFontSize` không ghìm dải, khác hẳn sàn 1.66 của `setLineHeight`; hai
    hằng `READING_FONT_SIZE_MIN/MAX` dùng chung cho cả thanh trượt lẫn phép ghìm.
  - `[low]` `[patch]` Mục lục mượn ba khoá `mode.library.*` — dựng khoá `mode.reading.*` riêng.
  - `[low]` `[patch]` Chú thích trong template còn nói `max-width` sau khi thước đã chuyển sang
    `width`; và lề song ngữ `join('')` dán hai câu nguyên văn — đổi thành `join(' ')`.
  - `[low]` `[patch]` Bàn đo e2e đảo theme ở mệnh đề ⑤ mà không trả về — mọi spec dùng chung
    một phiên app, nên nó đẩy một biến môi trường không ai khai vào spec kế tiếp.

## Design Notes

### 🔴 `⌘,` đã có chủ, và story này KHÔNG giành nó — hai đường, kèm số đo, chờ Ice chốt

**Phép đo (2026-08-30, baseline `fc7ca89`):** `grep -n "'Mod+Comma'" src/commands/index.ts` cho **một**
kết quả — dòng `:2509`, `['shortcuts.open', 'openShortcuts', 'Mod+Comma']`. `createKeymap`
(`keys.ts:486`) **ném** khi hai command giành một hợp âm: *"hai chỗ giành một phím thì cái sau
chết im lặng"*. `installCommands()` chạy trước `mount()`, nên một lượt đăng ký trùng làm ứng dụng
**không khởi động được** — đây không phải một lỗi âm thầm, nó là một cái chết ồn ào.

**Và hai bên có sức nặng khác nhau, đo được:**

| | Nguồn của hợp âm | Đo ở đâu |
|---|---|---|
| `shortcuts.open` | **Lựa chọn của dev**, không phải đặc tả. Chú thích tại chỗ (`:2507-2508`) tự khai lý do là *"`⌘,` là quy ước Preferences của macOS … và hợp âm đó **chưa ai chiếm**"* | `epics.md` Story 1.21 (`:1952-2010`): **0** lần nhắc `⌘` hay `Comma` trong toàn bộ AC |
| `reading.toggle_tuner` | **Đặc tả**: AC cuối của Story 5.11 liệt `⌘,` đích danh, và mockup `reading-mode.html` vẽ nó trên thanh công cụ | `epics.md:4293-4296` (AC) và `epics.md:629` (UX-DR46, bảng phím đầy đủ) |

⇒ Đọc theo sức nặng đặc tả, `⌘,` **nên** về Chế độ đọc và `shortcuts.open` nên đổi sang một hợp âm
khác. **Nhưng đó là một quyết định của Ice, không của dev:** nó gỡ một phím tắt **đang chạy trên máy
người dùng**, và `AGENTS.md` §Policy nói thẳng *"hai phương án đều hợp lệ ⇒ nêu cả hai kèm số đo cho
Ice chốt, đừng tự chọn rồi đi tiếp"*.

**Story này thi hành đường KHÔNG PHÁ:** `reading.toggle_tuner` đăng ký với **0 hợp âm mặc định**,
đúng khuôn `library.open_search_hit`/`library.search_mode_*` — nút trên thanh công cụ là đường vào
chính, và `Tab` + `Enter` phủ NFR17. 🟡 Vế `⌘,` của AC ghi thành **món nợ có chủ Ice**, không sửa
`epics.md`.

⚠️ **Và món nợ ấy nhẹ hơn nó trông:** Story 1.21 đã dựng lớp `ChordOverrides`
(§`ChordOverrides` của `keys.ts`, `ScopeKind::Shortcut`), nên Ice **tự gán được** `⌘,` cho `reading.toggle_tuner`
ngay hôm nay — chỉ cần gỡ nó khỏi `shortcuts.open` trước. Cái còn thiếu là **mặc định**, không phải
**năng lực**.

### Vì sao gom đoạn ở RUST — một ca biên mà một `computed` ở Vue sẽ nuốt mất

Cắt đoạn *trông như* việc render: duyệt danh sách, gặp `is_target_paragraph_end` thì xuống đoạn. Ca
biên phá cách đọc đó:

```
câu 1  is_omitted=0  is_target_paragraph_end=0
câu 2  is_omitted=1  is_target_paragraph_end=1   ← người dùng đã CẮT BỎ đúng câu kết đoạn
câu 3  is_omitted=0  is_target_paragraph_end=0
```

Lọc trước rồi cắt sau ⇒ `[1, 3]` **một đoạn** — hai đoạn của người dùng nhập làm một, im lặng.
Cắt trước rồi lọc sau ⇒ đúng hai đoạn nhưng đoạn đầu phải mang cờ của một câu **không còn tồn tại**.
⇒ Quy tắc thật là *"cờ kết đoạn chuyển cho câu còn sống liền trước trong cùng đoạn"*, và một quy tắc
có ca biên thì thuộc Rust (AD-1), cùng lý lẽ mà `core/segment/paragraph.rs` đã viết ra cho ba ca biên
của AD-37.

### Vì sao `ch` phải nằm trên CHÍNH phần tử mang cỡ chữ

`1ch` là bề rộng ký tự `0` của font **đang áp cho phần tử đó**. Đặt `max-width: 68ch` trên một khung
ngoài mang `ui-md` rồi để chữ đọc ở `read-md` bên trong thì con số 68 tính theo **font sai**, và bề
rộng sẽ **không** co theo cỡ chữ đọc — đúng thứ AC *"số ký tự mỗi dòng giữ nguyên khi đổi cỡ chữ"*
tồn tại để bảo đảm, và đúng thứ `tokens.json` §spacing (`:478`) cảnh báo bằng chữ.

### Phím trần là TOÀN ỨNG DỤNG, và đó là hình dạng đã chốt chứ không một thiếu sót

`src/commands/registry.ts` không có khái niệm *"hợp âm chỉ sống trong một chế độ"*. `B`/`1`/`2`/`3`/`D`
vì thế bắn cả khi đang ở Library — nhưng `keys.ts:418-437` (luật vùng gõ) chặn chúng trong mọi
`INPUT`/`TEXTAREA`/`SELECT`/`contenteditable`, và doc-comment của chính luật ấy **gọi đích danh**
`M`, `B`, `1 2 3` của UX-DR46 làm lý do nó tồn tại. Hệ quả duy nhất còn lại: bấm `2` ở Library đổi mức
chữ của một trang chưa hiện — không quan sát được, không mất dữ liệu. Một cơ chế phạm vi theo chế độ
là một thay đổi kiến trúc, tức một `AD` mới, tức §Block If.

### Bàn đo e2e ĐỦ SỨC đo story này — khác Story 5.10

§GIỚI HẠN của `5-10-hai-che-do-dau.md` ghi *"`create_work_from_text` chỉ dựng `source_text`; không
`target_text` nào tồn tại trong một bàn đo e2e"*. Câu đó đúng **cho một bàn đo không lái Editor**.
`editor-typing-flush.e2e.mjs` §*"gõ bằng `execCommand`"* (`:162-182`) đã có công thức: `document.execCommand('insertText')` trên ô
`contenteditable` (🔴 **không** `browser.keys()` — nó chỉ bắn `keydown`, chữ không hạ cánh; giới hạn
đã đo và ghi ở `:38-56`), rồi chờ flush. ⇒ Story này **lái Editor thật** để dựng `target_text`, nên
mọi mệnh đề hình học của nó đo được trong WKWebView.

### Ba số của mỗi mức đọc từ `tokens.json`, không viết thẳng vào test

Sáu token (`read-lg`/`read-md`/`read-sm` × `read-measure-lg/md/sm`) đã tồn tại từ Story 1.4 và
`check:tokens` Kiểm A cưỡng chế giá trị của chúng khớp bảng `DESIGN.md`. Một test viết thẳng `19px`
là **bản chép thứ hai** của cùng một sự thật, và nó sẽ đứng yên xanh vào đúng ngày token đổi.

## Verification

**Commands:**

- `npm run check:tokens` — expected: Kiểm A–H xanh, **0** cặp màu mới, **0** màu viết thẳng.
- `npm run check:i18n` — expected: Kiểm A/A2/B/C/D/E xanh; mọi khoá `mode.reading.*` mới có mặt.
- `npm run check:commands` — expected: Kiểm A xanh (mọi `@click` mới là một `dispatch` literal),
  Kiểm B (id mới tồn tại trong bộ đăng ký), Kiểm D (hai nền tảng), Kiểm E (`mode.reading` có
  `declareFocus`), ba sàn nội dung không giảm.
- `npm run check:panel-refs` — expected: mọi ô mới của `readingState.ts`/`themeState.ts` đi qua một
  `reset*` hoặc một miễn trừ CÓ TÊN; **0** miễn trừ chết.
- `npm run check:layout` — expected: xanh, **không** thành viên `window.`/`document.` mới.
- `npm run test` (vitest) — expected: hai tệp test mới xanh, gồm ca ghìm 1.66.
- `npm run build && cargo test --locked --manifest-path src-tauri/Cargo.toml` — expected:
  `segment_contract.rs` bộ ca mới xanh · `ipc_contract.rs` khoá dây mới xanh ·
  `config_invariants.rs` **không đổi** (segment.rs không nằm trong bảng `count_async_attrs`) ·
  `segment_boundary.rs` xanh.
- `npm run test:e2e -- --spec e2e/specs/story-5-11-reading-mode.e2e.mjs` — expected: năm mệnh đề
  hình học xanh trong WKWebView thật. ⚠️ `story-5-6-library-grid.e2e.mjs` đỏ từ baseline — không đọc
  thành hồi quy.

**Đối chứng ĐỎ bắt buộc (làm rồi hoàn nguyên, ghi kết quả vào §Auto Run Result):**

1. **GỠ** lời gọi `omit::segments_in_translation` trong `core/segment/reading.rs` (đọc thẳng
   `segments`) ⇒ ca *"câu đã cắt bỏ vắng mặt"* của `segment_contract.rs` phải **ĐỎ**.
2. **GỠ** phép chuyển cờ kết đoạn khi câu mang cờ bị cắt bỏ ⇒ ca *"cờ kết đoạn trên một câu cắt bỏ vẫn
   giữ được ranh giới"* phải **ĐỎ**, và **chỉ** ca đó.
3. **GỠ** phép ghìm 1.66 trong `setLineHeight` ⇒ ca vitest tương ứng phải **ĐỎ**;
   `npm run check:tokens` vẫn **XANH** — đó chính là phép đo chứng minh cổng không phủ đường này.
4. **ĐỔI** `max-width` của cột đọc từ `ch` sang `px` ⇒ mệnh đề ③ của e2e phải **ĐỎ**.

## Auto Run Result

Status: done
Blocking condition: (không có)

### Đã dựng

Chế độ đọc có nội dung thật (FR11) — chế độ thứ ba thôi là một khung rỗng 50 dòng. Một lệnh IPC
đọc-thuần mới `read_reading_chapter` trả Chương đang mở **đã gom sẵn thành đoạn ở Rust**: nó đi qua
`core::segment::omit::segments_in_translation` — chốt lọc Ice ký dựng sẵn 2026-08-15 cho một bề mặt
chưa tồn tại, tới nay **0 nơi gọi** — rồi qua hàm thuần mới `core::segment::reading::paragraphs_in_translation`.
Webview chỉ render: không một `v-if="!s.is_omitted"` nào ở Vue, vì trên dây **không có** `is_omitted`
để mà lọc (AD-1).

Trên màn hình: ba mức typography lấy nguyên sáu token đã có (`read-lg/md/sm` × `read-measure-*`),
thước đo bằng `ch` đặt trên **chính** phần tử mang cỡ chữ; thanh công cụ (song ngữ · ba preset ·
tinh chỉnh · sáng/tối · mục lục), khối tinh chỉnh hai thanh trượt, lề song ngữ ở **trái** không chen
dòng đọc, lớp phủ mục lục, và một `role="status"` **tám** nhánh phân biệt được. Phím trần `B` ·
`1` `2` `3` · `D` và `Mod+L`.

### Tệp đã đổi (24 tệp · +1.291 / −21 trên phần theo dõi, cộng 9 tệp mới)

**Rust** — `core/segment/reading.rs` (mới, hàm thuần gom đoạn) · `core/segment/mod.rs` (khai module) ·
`core/segment/omit.rs` (🔵 sửa tại chỗ hai mệnh đề đã hết đúng cho vế Chế độ đọc) ·
`commands/segment.rs` (`ReadingSegment`/`ReadingParagraph`/`ReadingChapter` + hàm thuần + vỏ `wire`) ·
`lib.rs` (đăng ký vỏ) · `tests/segment_contract.rs` (+8 ca) · `tests/ipc_contract.rs` (+2 ca đóng băng dây).

**Frontend** — `config/reading.ts` (mới, adapter ba trạng thái) · `config/bootstrap.ts` (hằng `KEY_THEME`) ·
`tokens/themeState.ts` (mới, `initTheme`/`setTheme`/`toggleTheme`) · `modes/readingState.ts` (mới, ba
nhóm state) · `modes/ReadingMode.vue` (viết lại) · `commands/index.ts` (11 dep + khối lệnh 5.11) ·
`main.ts` (đường khởi động theme + tiêm dep) · `modes/libraryChapters.ts` · `modes/libraryImport.ts`
(dọn state đọc khi đổi Tác phẩm) · `i18n/vi.json` (khoá mới).

**Bàn đo** — `tests/frontend/readingState.test.ts` · `readingTypography.test.ts` ·
`readingTheme.test.ts` (cả ba mới) · `e2e/specs/story-5-11-reading-mode.e2e.mjs` (mới).

**Tài liệu** — `deferred-work.md` (đóng một mục, thu hẹp một mục, mở ba mục có chủ) ·
`src/modes/README.md` (bảng sở hữu).

### Nghiệm thu — số ĐO ĐƯỢC, chạy sau lượt vá của vòng rà

| Đường | Kết quả |
|---|---|
| `check:tokens` · `i18n` · `commands` · `panel-refs` · `layout` · `gates` · `debt-owner` · `lint` | **8/8 xanh** |
| `vue-tsc --noEmit` | sạch |
| `npm run build` | xanh |
| vitest | **730 passed · 0 failed** (51 tệp) |
| `cargo test --locked` | **982 passed · 0 failed** (35 binary) |
| `npm run test:e2e` (WKWebView thật) | **1 passing** — cả năm mệnh đề hình học |

⚠️ **Một phép đo suýt bị đọc sai, ghi ra vì nó là bài học.** Lượt vitest đầu tiên cho **82 ca ĐỎ**,
toàn bộ `Test timed out in 5000ms` ở những tệp story này KHÔNG chạm. Nguyên nhân là một
`cargo test` chạy song song: load average 11,35, thời lượng phồng từ 8,37 s lên 71,78 s. Chạy lại
trên máy rảnh: xanh trọn. Con số báo cáo là lượt sau.

### Ba lượt e2e ĐỎ, ba kết luận khác nhau — và lượt thứ ba là một lỗi SẢN PHẨM

Bàn đo e2e chạy được trên máy này (ngược với lời khai *"sandbox không có GUI"* của lượt cài đầu), và
nó phải đỏ **ba** lần mới xanh:

1. **Lỗi bàn đo.** Trang đọc chỉ có câu MỘT. `typeInto(cells[1])` dời caret, mà *"rời segment"* là
   một trong bốn cửa flush NGAY của AD-35 ⇒ `waitForFlushAfter` xanh ở lượt flush của câu MỘT.
   Vá bằng **hai lượt chờ nối tiếp**; không một `pause()`, không hằng số nào bị nới.
2. **Bàn đo đo SAI BẤT BIẾN.** Mệnh đề *"bề rộng và cỡ chữ đổi CÙNG TỈ LỆ"* lệch 3,72 % rồi 3,33 %.
   Số lấy thẳng từ engine: tỉ lệ `ch`/`px` là **0,51208** ở 17,5px và **0,49849** ở 22px — WebKit làm
   tròn bề ngang glyph theo TỪNG cỡ chữ, nên `1ch` không tuyến tính theo `font-size`. Bất biến ĐÚNG
   là chữ của AC: **bề rộng / `1ch` giữ nguyên**, và con số ấy phải LÀ 68.
3. **Và chính mệnh đề "phải LÀ 68" bắt lỗi sản phẩm.** Thước buộc vào `max-width` trên một
   `flex: 0 0 auto` ⇒ cột co về bề rộng NỘI DUNG. Đo bằng phép GỠ thật: **57,27 ch** / **57,09 ch**,
   không phải 68. `check:tokens` không thấy (soi `tokens.json`, không soi bố cục); `happy-dom` cũng
   không (không bố cục). Thước chuyển sang `width`; sau vá: **68,00 ch** ở cả hai cỡ chữ.

### Sáu đối chứng ĐỎ đã chạy thật (làm rồi hoàn nguyên, đối chiếu bằng `diff`)

| Phép GỠ | Kết quả đo |
|---|---|
| gỡ `omit::segments_in_translation` khỏi `reading.rs` | ca *"câu cắt bỏ không để lại dấu vết"* ĐỎ |
| gỡ phép chuyển cờ kết đoạn | đúng **một** ca ĐỎ, 137 ca kia xanh |
| gỡ ghìm 1.66 khỏi `setLineHeight` | hai ca vitest ĐỎ, `check:tokens` vẫn XANH |
| `width` → `max-width` | **57,27 ch** thay vì **68,00 ch** |
| bỏ dấu cách giữa hai câu | ca *"đúng một dấu cách"* ĐỎ |
| bỏ một lời gọi `resetReadingToc()` | ca kiểm khai báo ĐỎ |
| đổi `B` thành `Mod+B` | ca hợp âm trần ĐỎ; `check:commands` Kiểm D vẫn xanh |

### Kết quả rà — 13 vá · 3 hoãn · 2 bác

Bốn lớp rà (blind · edge-case · verification-gap · intent-alignment). Chi tiết ở §Review Triage Log.
Hai mục nặng nhất đều là lỗi **im lặng**: hai câu liền nhau render dính vào nhau, và
`resetReadingToc()` được khai mà **không một chỗ gọi nào** (mục lục sống sót qua lượt đổi Tác phẩm ⇒
cú "Mở" phát một `chapter_id` của kho khác, và `chapter.id` là số nguyên CỤC BỘ theo AD-3 nên nó
trùng một Chương có thật).

**Hai mục BÁC, kèm lý do:** ① *"`deferred-work.md` không được đụng"* — **sai**, tệp có đổi 68 dòng;
lớp rà không thấy vì **tôi** đã loại tệp đó khỏi gói gửi rà để giữ kích thước. Lỗi quy trình của
lượt rà, không của mã. ② *"nền lớp phủ mục lục đục hoàn toàn thay vì làm mờ"* — đó là hệ quả TRỰC
TIẾP của hai luật đang đứng (`check:tokens` Kiểm D cấm `opacity` lùi chữ; không token `scrim` nào
tồn tại), đã ghi lý do tại chỗ; đổi nó là một quyết định token, không một bản vá.

### Rủi ro còn lại

- 🔴 **Vế `⌘,` của AC cuối KHÔNG được thi hành**, có chủ ý và có phép đo — hợp âm đã thuộc
  `shortcuts.open` từ Story 1.21, và `createKeymap` ném khi hai lệnh giành một phím (ứng dụng không
  khởi động được). Chủ: **Ice**, hai đường đã trình đủ số đo ở `deferred-work.md`.
- ⚠️ Mục lục chưa từng đi qua một webview thật (hoãn, chủ ghi rõ) — hành vi tiêu điểm sau khi đóng
  lớp phủ là mệnh đề DOM mà `happy-dom` không đảm được.
- ⚠️ Mệnh đề *"cả hai theme đạt WCAG AA"* của AC được thoả **theo thừa kế** từ `check:tokens`
  (0 cặp màu mới), không bằng một phép đo riêng cho bề mặt này.
- ⚠️ `read_reading_chapter` chép lại câu `SELECT` của `read_open_chapter_segments`; sửa bộ lọc ở một
  nơi làm Chế độ đọc và Workspace nói khác nhau về cùng một Chương, không cổng nào đỏ (hoãn, có chủ).

