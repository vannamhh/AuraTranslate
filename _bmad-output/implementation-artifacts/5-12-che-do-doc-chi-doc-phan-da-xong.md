---
title: 'Story 5.12: Chế độ đọc chỉ đọc phần đã xong'
type: 'feature'
created: '2026-08-30'
status: 'done'
baseline_revision: 'e36599e82c2a3f89e2f75ecc0c7aafa400ea4499'
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
deferred: []
---

<intent-contract>

## Intent

**Problem:** Chế độ đọc (Story 5.11) đọc **đúng một** Chương — Chương mà `OpenWork.chapter_id` đang
trỏ — và **không nhìn `chapter.status` một lần nào**. Hệ quả đo được: mở một Chương đang dịch dở rồi
bấm `⌘3` là ném một trang nửa dịch nửa trống ra làm "bản đọc", và bật song ngữ trên đó là ném nguyên
văn tiếng Trung vào giữa trang đọc tiếng Việt — đúng thứ FR120 tồn tại để chặn. Không có mốc biên,
không có đường đọc liên tục, và một câu **chưa xác nhận** trong một Chương đã đánh dấu *Đã xong* hiện
ra y hệt một câu đã chốt, tức màn hình **nói dối về trạng thái công việc**.

**Approach:** Đổi bề mặt đọc từ *một Chương* thành **một LƯỢT ĐỌC** (`ReadingRun`): Rust chọn dãy
Chương liên tiếp ở trạng thái `done` bắt đầu **tại Chương đang mở**, trả kèm một **mốc biên giới**
nói vì sao dãy dừng ở đó. Chương chưa `done` **không bao giờ rời `project.db`** — cùng kỷ luật mà
`is_omitted` đã dùng: webview không có gì để lọc vì không có gì được gửi. Mỗi câu chở thêm
`is_confirmed` để trang đọc gạch chấm nhẹ câu chưa xác nhận mà không phải đoán.

## Boundaries & Constraints

**Always:**
- **Vị từ *Đã xong* đi qua `LifecycleStatus::from_wire(...) == Some(Done)`**, không một phép so chuỗi
  `== "done"` viết tay. Một giá trị `chapter.status` **ngoài bốn giá trị** (cột là chuỗi tự do, không
  `CHECK`) đọc thành **KHÔNG `done`** ⇒ dãy dừng tại đó. Không đoán, không bỏ qua im lặng.
- **Chọn dãy và cắt đoạn ở RUST** (AD-1). `src/**` không được có một đường nào hỏi `status` để quyết
  hiển thị hay không; nó chỉ render thứ đã tới.
- **Một lượt `Store::read` duy nhất** cho cả danh sách Chương lẫn segment của mọi Chương trong dãy —
  cùng lý lẽ `read_reading_chapter` đã ghi: hai lượt đọc rời nhau là hai ảnh chụp lệch được.
- **Câu SQL đọc segment của một Chương có ĐÚNG MỘT bản** — lượt này rút nó thành một hàm dùng chung
  cho `read_open_chapter_segments` và đường đọc, đóng món nợ 🟡 mà Story 5.11 tự ghi.
- **Không nhánh rỗng nào im lặng.** Một Chương `done` trong dãy mà không sinh đoạn nào vẫn hiện tiêu
  đề **kèm một câu nói vì sao** (rỗng · mọi câu đã cắt bỏ). Dãy rỗng cũng có câu của riêng nó.
- Chuỗi ở vị trí mã trong `src-tauri/**` viết KHÔNG DẤU; mọi text node ở `.vue` qua `t()`.
- Màu và cỡ chữ chỉ từ token. Gạch chấm dùng `var(--color-ornament)` — token vai `stroke` đã có, **0**
  cặp tương phản mới.

**Block If:**
- Lời giải đòi một giá trị **thứ năm** của `LifecycleStatus` (ví dụ một trạng thái *"đọc được"*
  riêng) — §Block If của Story 5.4 giao dứt khoát cho Ice.
- Lời giải đòi một **hợp âm TRẦN `Enter`** đăng ký toàn ứng dụng, hoặc một cơ chế **phạm vi hợp âm
  theo chế độ** — cái sau là một `AD` mới, không một dòng mã.
- Lời giải đòi đổi ngữ nghĩa `chapter.status` hoặc thêm `CHECK` vào cột đó.

**Never:**
- **Không** gửi một byte nào của Chương chưa `done` ra dây — kể cả `chapter_title`, trừ đúng một tên
  Chương trên **mốc biên** (đó là nhãn của một cái mốc, không phải nội dung để đọc).
- **Không** nhớ vị trí cuộn của Chế độ đọc — món nợ `deferred-work.md` mang chủ *"Story 5.12 hoặc
  5.13"*; lượt này **thu hẹp chủ về 5.13** bằng chữ, không tự làm.
- **Không** dựng danh sách chỗ đánh dấu, không phím `M`, không `↵` nhảy sang Workspace bằng hợp âm —
  FR119 là Story 5.13.
- **Không** thêm nút *"Về đầu Tác phẩm"* / *"Xem N chỗ đã đánh dấu"* của mockup: không AC nào đòi.
- **Không** đổi `epics.md`/`prd.md` cho khớp mã.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Đọc liên tục | Chương đang mở `done`; hai Chương kế `done`; Chương thứ tư `in_progress` | `chapters` = **ba** Chương theo `(ord, id)`; `frontier.kind = "next-not-done"`, `frontier.chapter` = Chương thứ tư | No error expected |
| Chạm biên ngay | Chương đang mở `in_progress` | `chapters` = **rỗng**; `frontier.chapter` = **chính Chương đang mở** | No error expected |
| Hết Tác phẩm | Mọi Chương từ Chương đang mở tới cuối đều `done` | `chapters` = tất cả; `frontier.kind = "end-of-work"`, `frontier.chapter = null` | No error expected |
| Trạng thái lạ | Chương kế mang `status = "finished"` (ngoài bốn giá trị) | Dãy **dừng trước** nó; `frontier.chapter.status = "finished"` nguyên văn | Không ném — một giá trị lạ là *không `done`* |
| Câu chưa xác nhận | Chương `done`, một câu `status = 'draft'` | Câu **vẫn ra dây**, `is_confirmed = false` | No error expected |
| Xong bằng tay | Chương đặt `done` thủ công, còn ba câu `draft` | Cả ba ra dây với `is_confirmed = false` | No error expected |
| Chương `done` rỗng | Chương `done`, 0 segment còn sống | `paragraphs = []`, `segment_count = 0` | No error expected |
| Mọi câu cắt bỏ | Chương `done`, mọi segment `is_omitted = 1` | `paragraphs = []`, `segment_count > 0` | No error expected |
| Chưa mở Tác phẩm | `open = None` | — | `Err` code `work.none_open` |
| Hàng Chương biến mất | `OpenWork.chapter_id` không còn trong bảng `chapter` | — | `Err` code `segment.chapter_not_found` |
| Đi tiếp từ mốc | Bấm *Dịch tiếp Chương N* trên mốc biên | `openChapterById(N)` rồi `setMode('workspace')`; state đọc bị vứt | `openChapterById` trả `false` ⇒ không đổi chế độ, ghi chẩn đoán |

</intent-contract>

## Code Map

**Rust — vị từ trạng thái (dùng lại, KHÔNG chép)**

- `src-tauri/src/core/lifecycle/mod.rs` — `lifecycle_statuses!` khai bốn giá trị; `LifecycleStatus::from_wire`
  (`:76`) là **cửa duy nhất** phân giải một giá trị đến từ cột trên đĩa, và nó trả `None` cho giá trị lạ
  chứ không đoán. `derive_work_status` (`:106`) là chuyện tầng Tác phẩm — **không** liên quan story này.
  🔴 Doc-comment của module cấm thẳng việc thêm giá trị thứ năm.
- `src-tauri/src/commands/lifecycle.rs:138` `set_chapter_status` — đường DUY NHẤT ghi `chapter.status`;
  test và bàn đo e2e dựng fixture qua đây, không `UPDATE` tay ở tầng SQL.

**Rust — bề mặt đọc hôm nay (thứ story này viết lại)**

- `src-tauri/src/commands/segment.rs:949` `ReadingSegment` (ba trường) · `:958` `ReadingParagraph` ·
  `:967` `ReadingChapter` (bốn trường) · `:987` `read_reading_chapter` · `:2878-2894` vỏ `wire`.
  ⚠️ Doc-comment `:962-966` khai *"hình dạng dây đã CHỐT ở bốn trường"* và `readingState.ts:16-25` dựa
  vào mệnh đề đó để đi vòng một lượt IPC phụ — **cả hai hết đúng sau story này, sửa TẠI CHỖ kèm 🔵**.
- `src-tauri/src/commands/segment.rs:852` `read_open_chapter_segments` — mang **bản gốc** của câu
  `SELECT` chín cột; `read_reading_chapter:1000-1020` mang **bản chép thứ hai** của chính nó
  (món nợ 🟡 Story 5.11 tự ghi: *"sửa bộ lọc ở một nơi làm Chế độ đọc và Workspace nói khác nhau"*).
  ⇒ Task 1 rút thành một hàm dùng chung.
- `src-tauri/src/commands/segment.rs:1516` `SEGMENT_STATUS_CONFIRMED` — hằng cho `is_confirmed`,
  không một chuỗi `"confirmed"` viết thẳng.
- `src-tauri/src/core/segment/reading.rs` `paragraphs_in_translation` — **không đụng**: nó nhận
  `&[ChapterSegment]` của MỘT Chương và đúng vai đó không đổi. Đường đọc gọi nó một lần mỗi Chương.
- `src-tauri/src/core/segment/omit.rs` — chốt lọc cắt bỏ; **không** thêm một vị từ thứ hai ở đâu.

**Rust — khuôn cho hình dạng dây mới**

- `src-tauri/src/commands/chapter.rs:184` `ChapterSwitchOutcome` — **khuôn trực tiếp** cho
  `ReadingFrontierKind`: một `enum` `Serialize` với `#[serde(rename = "…")]` từng biến thể, và
  doc-comment `:180-182` nói rõ *"không biến thể nào cho lỗi — lỗi đi bằng `Err(IpcError)`"*.
- `src-tauri/src/commands/chapter.rs:201` `ChapterSwitch` — **khuôn trực tiếp** cho `ReadingFrontier`:
  một trường `kind` + một `Option<T>` mang dữ liệu, kèm mệnh đề *"`Some` khi và chỉ khi `kind == …`"*
  viết ra thành doc-comment.
- `src-tauri/src/commands/chapter.rs:325` `ChapterRow` (`chapter_id`/`ord`/`title`/`status`/`segment_count`)
  · `:356` `fn fetch_chapter_rows` — câu `SELECT` danh sách Chương kèm `status` và số segment còn sống.
  Đường đọc cần **cùng dữ kiện** nhưng bên trong `Store::read` của chính nó ⇒ không gọi lại
  `list_chapters()` (nó tự mở một lượt `read` thứ hai — đúng thứ §Always cấm).
- `src-tauri/src/commands/chapter.rs:74` `no_work_open()` · `:97` `chapter_not_found(id)` — dùng lại
  nguyên, khoá `err.work.none_open` / `err.segment.chapter_not_found` đã có trong `vi.json`.
- `src-tauri/src/lib.rs:385` `crate::commands::segment::wire::read_reading_chapter` trong
  `generate_handler!` — **đổi tên tại chỗ** thành `read_reading_run`.
- ⚠️ `commands/segment.rs` **không** nằm trong bảng `count_async_attrs`
  (`src-tauri/tests/config_invariants.rs:1037-1058`, chỉ `glossary.rs` 7 · `library.rs` 4 ·
  `chapter.rs` 4). Vỏ mới **không** `(async)` — cùng hạng đọc-thuần với vỏ nó thay thế; xem
  §Design Notes cho phép đo và món nợ quy mô.

**Rust — cổng và bàn đo**

- `src-tauri/tests/segment_contract.rs:6916` khối *"STORY 5.11"* — chỗ nối cho khối 5.12. Helper
  đã có, dùng lại nguyên: `ordered_ids` (`:6925`) · `set_omitted` (`:6936`) · `paragraph_shapes`
  (`:6948`). `split_chapter_at_segment` **đã được import** ở `:18` ⇒ dựng Chương thứ hai không cần
  import mới.
- `src-tauri/tests/project_contract.rs:2341` — khuôn dựng nhiều Chương bằng `split_chapter_at_segment`,
  kèm `insert_segment_directly`/`set_chapter_status_directly`.
- `src-tauri/tests/ipc_contract.rs:810-873` — ca đóng băng khoá `snake_case` của ba struct đọc;
  `:875-890` — ca *"vỏ đã đăng ký"*. Cả hai **sửa tại chỗ** cho hình dạng mới, không thêm ca song song.
- `src-tauri/tests/segment_boundary.rs` — ranh giới *"chỉ `split` biết bảng chữ cái kết câu"*; mã mới
  không mang từ vựng đó.

**Frontend — nơi story cắm vào**

- `src/config/reading.ts` (**146 dòng, toàn bộ tệp**) — adapter ba trạng thái + vị từ kiểm kiểu
  MỌI trường / MỌI phần tử (`isReadingSegment` `:66` · `isReadingSegmentArray` `:74` ·
  `isReadingParagraph` `:78` · `isReadingChapter` `:88`). Khuôn cho các vị từ mới; `hasIpcBridge()`
  (`:99`) và `UNKNOWN_IPC_ERROR` (`:104`) giữ nguyên.
- `src/modes/readingState.ts` — ① nội dung (`chapter` `:39` · `chapterHasSegments` `:48` ·
  `readingStatusKind` `:69` · `ensureReadingLoaded` `:95` · `resetReading` `:141`) · ② mục lục
  (`resetReadingToc` `:239`) · ③ typography (`:250` trở đi, **không đụng**).
  🔴 `chapterHasSegments` + lượt `listChapters()` phụ trong `ensureReadingLoaded` **biến mất** — dữ
  kiện đó nay đến thẳng qua `ReadingChapter.segment_count`, tức nhánh `'empty-unknown'` **hết lý do
  tồn tại** (nó ra đời chỉ vì lượt hỏi phụ có thể trượt).
- `src/modes/ReadingMode.vue` — `statusMessage` (`:86`, một `switch`, mỗi nhánh đúng một `t()`) ·
  khối `.page` (`:226`) render một Chương · lề song ngữ (`:236`) · `.column` mang `width: readingStyle.measure`
  (`:249-257`) · lớp phủ mục lục (`:290`). Vòng lặp Chương bọc ngoài khối `.page` hiện tại.
  ⚠️ Dấu cách giữa hai câu (`:270-272`) là **nội dung**, giữ nguyên nguyên tắc đó khi bọc thêm.
- `src/modes/libraryChapters.ts:304` `openCurrentChapter()` — `openChapterById(id)` rồi
  `setMode('workspace')`. **Khuôn nguyên văn** cho đường sang Workspace của mốc biên.
- `src/panels/editorPanelState.ts` `openChapterById(chapterId, segmentId?)` — flush → `openChapter`
  → `resetEditorPanel`/`resetSourcePanel` → `ensureSegmentsLoaded`; kết thúc bằng
  `enterFocus('panel.grid')`, hợp lệ vì đích đến là Workspace.
- `src/commands/index.ts:785-808` khai `CommandDeps` cụm đọc · `:1362-1460` khối lệnh Story 5.11 —
  chỗ chèn khối 5.12 ngay sau. `src/main.ts:100-106` (import) · `:406-409` (tiêm dep) — một khối một
  Story.
- `src/i18n/vi.json:288-308` khối `mode.reading.*` · `:96-106` khối `command.reading.*` ·
  `:208-211` `lifecycle.*` (nhãn bốn trạng thái, dùng lại cho nhãn trạng thái Chương chặn).
- `src/tokens/tokens.json:29,47` `ornament` (`#a9a196` sáng / `#6a6459` tối) — **đúng giá trị
  `--orn`** mà `mockups/reading-mode.html:9,18,75` dùng cho `.s.unconf`. Vai `stroke`
  (`roles.stroke` `:80`) ⇒ Kiểm C không sinh cặp mới. Đã dùng ở `src/modes/LibraryMode.vue:1294`.

**Cổng — số đo tại baseline `e36599e`**

- `scripts/check-commands.mjs:319` `COMMAND_FLOOR = 52` · `:352` `CLICK_FLOOR = 27` · `:363`
  `DISPATCH_FLOOR = 40` — cận **DƯỚI**, thêm lệnh/nút không làm đỏ. Kiểm A: `@click` = đúng một
  `dispatch('<id>')` literal. Kiểm B: id trong template phải có trong bộ đăng ký.
- `src/commands/keys.ts:401-437` `lacksPrimaryMod` + `isTypingZone`; `createKeymap` (`:466-497`) **ném**
  khi hai lệnh giành một hợp âm. Hợp âm đã chiếm (đo 2026-08-30): `Mod+Comma` · `Mod+D` · `Mod+Enter`
  · `Mod+H` · `Mod+L` · `Mod+M` · `Mod+Slash` · `Mod+Shift+Slash` · `Mod+1..3` · họ `Mod+Alt+…`;
  trần: `B` · `D` · `1` `2` `3` (Story 5.11). ⇒ Lệnh mới của story này đăng ký **không hợp âm**.
- `scripts/check-tokens.mjs:849` Kiểm B (màu viết thẳng) · `:1089` Kiểm C (tương phản text × surface)
  · `:1361` Kiểm D (không `opacity` lùi chữ) · `:1459` Kiểm F (không elevation) · `:1502` Kiểm H.
- `scripts/check-i18n.mjs:866` Kiểm A · `:927` Kiểm A2 · `:1187` Kiểm D (cấm *"bạn"*/*"chúng tôi"*).
- `scripts/check-panel-refs.mjs:125` `EXEMPT` — mọi ô cấp module mới phải qua một `reset*` hoặc một
  miễn trừ CÓ TÊN; `RE_REACTIVE` (`:352`) neo đầu dòng ⇒ **mỗi khai báo một dòng**.
- `e2e/specs/story-5-8-reorganise-chapters.e2e.mjs:63,93,103` — công thức dựng fixture bằng
  `internals.invoke('create_work_from_text' | 'read_open_chapter_segments' | 'split_chapter_at_segment')`;
  ⚠️ tên tham số đi **camelCase** (`segmentId`). `e2e/specs/story-5-4-lifecycle.e2e.mjs:50-57` — chọn
  Tác phẩm và các nút vòng đời. `e2e/support/workspace.mjs:57` `openWorkspaceWithWork(...)`.
  ⚠️ `story-5-6-library-grid.e2e.mjs` đỏ từ baseline (chủ Story 5.6) — không đọc thành hồi quy.
- `_bmad-output/implementation-artifacts/deferred-work.md:3743-3749` — mục Chế độ đọc, dòng
  *"đọc liên tục xuyên Chương (FR120) … chủ riêng: Story 5.12 · 5.13"*: story này đóng vế FR120.
  `:5047` và `:8746` — món nợ **vị trí cuộn**, chủ *"Story 5.12 hoặc 5.13"*: story này **thu hẹp chủ
  về 5.13**, không đóng.

## Tasks & Acceptance

**Execution:**

1. `src-tauri/src/commands/segment.rs` — rút câu `SELECT` chín cột thành **một** hàm dùng chung
   (`fn select_chapter_segments(conn: &Connection, chapter_id: i64) -> SqlResult<Vec<ChapterSegment>>`),
   rồi cho `read_open_chapter_segments` (`:852`) **và** đường đọc mới cùng gọi nó. Doc-comment ghi
   bằng chữ rằng đây là bản DUY NHẤT và vì sao. — Rationale: đóng món nợ 🟡 Story 5.11 tự ghi — hai
   bản chép của một bộ lọc lệch nhau làm Chế độ đọc và Workspace nói khác nhau về cùng một Chương,
   và **không cổng nào đỏ**.
2. `src-tauri/src/commands/segment.rs` — hình dạng dây mới, `snake_case`, **không**
   `#[serde(rename_all)]`: ① `ReadingSegment` **thêm** `is_confirmed: bool` (nguồn: `status ==
   SEGMENT_STATUS_CONFIRMED`, không một chuỗi viết thẳng); ② `ReadingChapter` **thêm**
   `segment_count: i64` (số hàng còn sống của Chương đó, đếm **trong cùng lượt đọc**, không một lượt
   IPC phụ); ③ `enum ReadingFrontierKind { NextNotDone, EndOfWork }` với
   `#[serde(rename = "next-not-done" | "end-of-work")]`, khuôn `ChapterSwitchOutcome`; ④
   `ReadingFrontierChapter { chapter_id, chapter_ord, chapter_title, status }`; ⑤
   `ReadingFrontier { kind, chapter: Option<ReadingFrontierChapter> }` kèm doc-comment *"`Some` khi
   và chỉ khi `kind == NextNotDone`"*; ⑥ `ReadingRun { chapters: Vec<ReadingChapter>, frontier }`.
   — Rationale: một `Option` có điều kiện phải phát biểu điều kiện ra thành chữ, đúng khuôn
   `ChapterSwitch::chapter` đã ký.
3. `src-tauri/src/commands/segment.rs` — hàm thuần `read_reading_run(open: Option<&OpenWork>) ->
   Result<ReadingRun, IpcError>` **thay** `read_reading_chapter`: trong **một** `Store::read` —
   ① đọc `id, ord, title, status` mọi Chương `ORDER BY ord, id`; ② tìm vị trí `open.chapter_id`,
   vắng mặt ⇒ `Err(chapter_not_found(...))`; ③ từ vị trí đó đi tới, **lấy tiền tố** các Chương mà
   `LifecycleStatus::from_wire(&status) == Some(Done)`; ④ với mỗi Chương của tiền tố gọi Task 1 rồi
   `reading::paragraphs_in_translation`, và đếm `segment_count` từ chính dãy đã đọc (**không** một
   `COUNT(*)` thứ hai); ⑤ dựng `frontier`: hàng ngay sau tiền tố ⇒ `NextNotDone` + dữ liệu hàng đó
   (kể cả khi tiền tố rỗng — hàng đó là chính Chương đang mở), hết bảng ⇒ `EndOfWork` + `None`.
   — Rationale: *"chỉ đọc phần đã xong"* là một quy tắc nghiệp vụ có ca biên, nên nó thuộc Rust
   (AD-1); và một mốc biên không nói được vì sao nó ở đó là một trang cụt.
4. `src-tauri/src/commands/segment.rs` (module `wire`) + `src-tauri/src/lib.rs:385` — đổi vỏ
   `read_reading_chapter` thành `read_reading_run` (`#[tauri::command]`, **không** `(async)`,
   `State` qua `try_state`) và đổi tên trong `generate_handler!`. **Xoá** vỏ cũ, không để lại hai
   đường. — Rationale: tên trên dây LÀ tên hàm; hai lệnh đọc cùng một thứ là hai nguồn sự thật.
5. `src-tauri/src/commands/segment.rs:962-966` và `src/modes/readingState.ts:16-25` — **sửa TẠI CHỖ**
   kèm 🔵 + ngày hai mệnh đề nay hết đúng: *"hình dạng dây đã CHỐT ở bốn trường"* và cả khối
   *"vì sao Chương rỗng và mọi câu đã cắt bỏ là hai trạng thái phải phân biệt được"* (lượt IPC phụ
   biến mất). — Rationale: một mệnh đề hết đúng để lại nguyên là chỗ story sau chép sai.
6. `src-tauri/tests/segment_contract.rs` — khối ca mới, mỗi tên là một CÂU, phủ **mọi** hàng của
   §I/O Matrix: dãy ba Chương dừng trước Chương `in_progress` · Chương đang mở chưa `done` ⇒ dãy rỗng
   và mốc trỏ chính nó · mọi Chương còn lại `done` ⇒ `end-of-work` và `chapter = None` ·
   `status = "finished"` chặn dãy và đi ra dây nguyên văn · câu `draft` mang `is_confirmed = false` ·
   Chương đặt `done` thủ công còn ba câu `draft` ⇒ cả ba mang dấu · Chương `done` rỗng ⇒
   `segment_count = 0` · mọi câu cắt bỏ ⇒ `paragraphs` rỗng mà `segment_count > 0` · chưa mở Tác phẩm
   ⇒ `work.none_open` · hàng Chương biến mất ⇒ `segment.chapter_not_found`. Dựng Chương thứ hai bằng
   `split_chapter_at_segment`, đặt trạng thái bằng `set_chapter_status`. — Rationale: bảng I/O phải
   có chủ ở tầng gọi được **không cần webview**, không ở màn hình.
7. `src-tauri/tests/ipc_contract.rs:810-890` — **sửa tại chỗ**: đóng băng khoá `snake_case` của
   `ReadingRun`/`ReadingFrontier`/`ReadingFrontierChapter`/`ReadingChapter`/`ReadingSegment`, đóng
   băng **hai chuỗi biến thể** trên dây (`"next-not-done"` · `"end-of-work"`), và
   khẳng định vỏ `read_reading_run` có mặt trong `generate_handler!` (và tên cũ **không** còn).
   — Rationale: hai nửa của một hợp đồng dây phải có một chỗ giữ chúng khớp nhau.
8. `src/config/reading.ts` — kiểu mới khớp Task 2 + vị từ kiểm kiểu **MỌI trường, MỌI phần tử**
   (`isReadingFrontierKind` chốt đúng hai chuỗi · `isReadingFrontierChapter` · `isReadingFrontier`
   kiểm cả bất biến *"`chapter` non-null khi và chỉ khi `kind === 'next-not-done'`"* ·
   `isReadingRun`), và `readReadingRun(): Promise<{ run: ReadingRun | null; error: IpcError | null }>`
   — một `invoke`, một `try/catch`, **không ném**, đủ **ba** trạng thái. — Rationale: `is_confirmed`
   quyết định một dấu hiển thị; một `undefined` lọt qua sẽ vẽ mọi câu như đã xác nhận, im lặng.
9. `src/modes/readingState.ts` nhóm ① — thay `chapter` bằng `run`; **gỡ** `chapterHasSegments`, lượt
   `listChapters()` phụ trong `ensureReadingLoaded`, và nhánh `'empty-unknown'`. `readingStatusKind`
   nay: `'not-loaded'` · `'pending'` · `'no-work'` · `'error'` · `'content'` (có ít nhất một đoạn) ·
   `'frontier-only'` (dãy rỗng) · `'all-omitted'` (dãy có Chương, **0** đoạn, tổng `segment_count > 0`)
   · `'empty-chapters'` (dãy có Chương, **0** đoạn, tổng `segment_count = 0`). Thêm
   `openFrontierInWorkspace()`: đọc `frontier.chapter`, `null` ⇒ ghi chẩn đoán và trả (**không ném** —
   nó chạy từ một `dispatch`), ngược lại `openChapterById(id)` → nếu `true` thì `resetReading()` +
   `resetReadingToc()` + `setMode('workspace')`. `resetReading()` dọn **mọi** ô mới. — Rationale: một
   danh sách rỗng không tự nói vì sao nó rỗng, và nhánh `'empty-unknown'` chỉ tồn tại vì lượt hỏi phụ
   có thể trượt — gỡ nguyên nhân thì gỡ luôn nhánh.
10. `src/modes/ReadingMode.vue` — vòng lặp **Chương** bọc ngoài khối trang: mỗi Chương một tiêu đề
    (`chapter_title ?? untitled(ord)`), các đoạn của nó, và — khi `paragraphs.length === 0` — một câu
    nói vì sao (`segment_count > 0` ⇒ *mọi câu đã cắt bỏ*, ngược lại ⇒ *chưa có câu nào*). Câu **chưa
    xác nhận** mang `:class="{ unconfirmed: !segment.is_confirmed }"`. Lề song ngữ lặp theo **cùng**
    cấu trúc Chương → đoạn, giữ nguyên luật một dấu cách giữa hai câu. Cuối trang: khối **mốc biên**
    (`.frontier`) — tiêu đề, một câu nói dãy vừa đọc từ Chương nào tới Chương nào (hoặc dãy rỗng),
    một câu nói Chương nào chặn kèm **nhãn trạng thái** của nó qua `t('lifecycle.…')`, và một
    `<button>` `@click="dispatch('reading.continue_in_workspace')"` chỉ hiện khi
    `kind === 'next-not-done'`. **Sửa tại chỗ** kèm 🔵 khối doc-comment đầu tệp. — Rationale: mốc biên
    thay hai nút trước/sau chính là để *dừng có lý do* thay vì cụt lủn (UX §memlog quyết định 2026-08-02).
11. `src/modes/ReadingMode.vue` `<style scoped>` — `.unconfirmed { border-bottom: 1px dotted
    var(--color-ornament); }` và khối `.frontier` chỉ dùng token đã có (`--color-outline` cho vạch
    ngăn, `--color-on-surface-variant` cho chữ phụ). Không `box-shadow`, không gradient, không
    `opacity` lùi chữ. — Rationale: `ornament` là **đúng giá trị** `--orn` của mockup và đã ở vai
    `stroke`, nên dấu này là một cái tên đã kiểm chứ không một màu mới chưa ai đo.
12. `src/commands/index.ts` — khối Story 5.12 ngay sau khối 5.11: `reading.continue_in_workspace`,
    **0 hợp âm mặc định**, một dep tuỳ chọn trong `CommandDeps` + `portMissing` khi vắng. 🔴 **Không**
    đăng ký một hợp âm TRẦN `Enter`. — Rationale: xem §Design Notes; một `Enter` trần toàn ứng dụng
    bắn cả trong Library và Workspace vì `registry` không có phạm vi theo chế độ.
13. `src/main.ts` — một khối import + một dòng tiêm dep cho Story 5.12, đúng khuôn 5.11.
    — Rationale: đăng ký ở `main.ts`, không trong `App.vue` — một lượt HMR gọi `installCommands()`
    lần hai và `register()` ném vì id trùng.
14. `src/i18n/vi.json` — khoá mới `mode.reading.frontier_*`, `mode.reading.status_frontier_only`,
    `mode.reading.status_empty_chapters`, `mode.reading.chapter_empty`, `mode.reading.chapter_all_omitted`,
    `command.reading.continue_in_workspace`; **gỡ** `mode.reading.status_empty_unknown` (nhánh đã chết)
    và **sửa tại chỗ** `status_empty`/`status_all_omitted` nếu câu cũ nói về "Chương này" trong khi
    màn hình nay nói về cả một dãy. Câu vô nhân xưng, placeholder đúng dải `{ten_tham_so}`.
    — Rationale: Kiểm A2 đòi mọi text node qua `t()`; Kiểm B đòi khoá không giá trị rỗng, và một khoá
    mồ côi là một khoá sẽ được ai đó dùng lại sai.
15. `tests/frontend/readingFrontier.test.ts` (**mới**) — với `invoke` giả: bốn hình dạng `ReadingRun`
    (dãy ba Chương + mốc · dãy rỗng + mốc · `end-of-work` · lỗi `work.none_open`) cho đúng bốn
    `readingStatusKind`; `openFrontierInWorkspace()` với `frontier.chapter = null` **không ném** và
    **không** đổi chế độ; vị từ `isReadingRun` **từ chối** một `run` có `kind = 'end-of-work'` kèm
    `chapter` non-null (và ngược lại). — Rationale: đối chứng phải là một phép GỠ thật, và một hợp
    đồng *"`Some` khi và chỉ khi"* chỉ có nghĩa nếu có ai từ chối trường hợp ngược.
16. `tests/frontend/readingUnconfirmed.test.ts` (**mới**) — mount `ReadingMode.vue` với một dãy hai
    Chương: câu `is_confirmed = false` mang lớp `unconfirmed`, câu `true` **không**; số phần tử mang
    lớp đó bằng đúng số câu chưa xác nhận trong fixture; **gỡ** `is_confirmed` khỏi fixture (thành
    `undefined`) ⇒ adapter từ chối cả `run` chứ không vẽ mọi câu như đã xác nhận. — Rationale: AC6
    nói màn hình **không được nói dối về trạng thái**, và cách nó nói dối rẻ nhất là một trường
    thiếu đọc thành *falsy* rồi rơi vào nhánh sai.
17. `tests/frontend/readingState.test.ts` — **sửa tại chỗ** các ca dựa trên hình dạng cũ
    (`chapterHasSegments`, `'empty-unknown'`, `readingChapter`), giữ nguyên ca *"câu `is_omitted`
    không bao giờ tới được webview"* và ca hợp âm trần. — Rationale: một cây test không sửa theo kiểu
    là một cây test vẫn xanh trong khi thứ nó kiểm đã đổi dưới chân.
18. `e2e/specs/story-5-12-reading-frontier.e2e.mjs` (**mới**) — trong WKWebView thật: dựng một Tác
    phẩm, `split_chapter_at_segment` để có **hai** Chương, gõ bản dịch thật bằng
    `document.execCommand('insertText')` + chờ flush **nối tiếp** (khuôn
    `story-5-11-reading-mode.e2e.mjs:57-80`), đặt Chương 1 `done` và để Chương 2 `in_progress`, `⌘3`;
    rồi đo: ① chữ của Chương 1 có trên trang; ② **không** một mảnh `source_text` nào của Chương 2 có
    trong `document.body.textContent` (kể cả khi bật `B`); ③ khối `.frontier` có mặt và nêu đích danh
    Chương 2; ④ bấm nút *Dịch tiếp* ⇒ `mode.workspace` hiện và lưới mang segment của Chương 2.
    — Rationale: mệnh đề *"nguyên văn của Chương chưa xong KHÔNG xuất hiện"* là một mệnh đề về **cây
    DOM trong engine thật**; `happy-dom` chạy trên `invoke` giả nên nó chỉ chứng minh được rằng dữ
    liệu giả không có mặt.
19. `_bmad-output/implementation-artifacts/deferred-work.md` — **đóng bằng chữ** vế FR120 của mục
    *"Chế độ đọc → Epic 5"* (`:3743-3749`) theo khuôn `→ ✅ ĐÃ ĐÓNG <ngày> (Story 5.12)`; **thu hẹp**
    món nợ vị trí cuộn (`:5047` · `:8746`) từ *"Story 5.12 hoặc 5.13"* về **chủ duy nhất: Story 5.13**,
    kèm lý do; **mở** hai mục có chủ: `↵` của mốc biên chưa có hợp âm (**chủ: Ice**, cùng hạng món nợ
    `⌘,` của 5.11) và **dãy đọc nạp trọn vào bộ nhớ trong một lượt** (**chủ: Story 5.14**, cùng lượt
    đo NFR3/NFR4/NFR5). — Rationale: không bao giờ xoá một mục đã đóng, và một mục hai chủ là một mục
    không ai làm.
20. `src/modes/README.md` — bảng sở hữu **sửa tại chỗ** kèm 🔵: vế *"đọc liên tục qua Chương đã xong
    và mốc biên"* nay ✅ ở 5.12; đánh dấu chỗ cần sửa (FR119) vẫn ⬜, chủ 5.13.
    — Rationale: một bảng sở hữu nói sai là chỗ story sau chép sai.

**Acceptance Criteria:**

- **Given** một Tác phẩm có Chương 1 và Chương 2 đều *Đã xong* và Chương 3 *Đang dịch*, **when** mở
  Chương 1 rồi vào Chế độ đọc, **then** trang đọc chở nội dung của **cả hai** Chương 1 và 2 theo thứ
  tự, mỗi Chương có tiêu đề riêng, **and** không thao tác nào phải đi qua Library.
- **Given** cùng Tác phẩm đó, **when** trang đọc render, **then** **không** một ký tự nào của
  `source_text` **hoặc** `target_text` thuộc Chương 3 có mặt trong cây DOM — kể cả khi công tắc song
  ngữ bật — **and** phép chặn chạy ở Rust: gỡ vị từ `from_wire(...) == Some(Done)` khỏi
  `read_reading_run` làm bộ ca Rust ĐỎ, còn `src/**` không có một đường lọc `status` nào.
- **Given** dãy đọc dừng vì Chương 3 chưa xong, **when** cuộn tới cuối trang, **then** một **mốc biên
  giới** hiện ra nói **đã hết phần đã dịch**, nêu đích danh Chương 3 và **nhãn trạng thái hiện thời**
  của nó, **and** mang đúng một nút sang Workspace.
- **Given** mốc biên giới, **when** bấm nút sang Workspace (bằng chuột **hoặc** bằng `Tab` + `Enter`),
  **then** ứng dụng chuyển sang Workspace với Chương 3 đã mở, **and** state đọc bị vứt nên lượt vào
  Chế độ đọc kế tiếp nạp lại từ con trỏ Chương mới.
- **Given** mọi Chương từ Chương đang mở tới cuối Tác phẩm đều *Đã xong*, **when** đọc hết, **then**
  mốc biên nói **hết Tác phẩm** — một câu **khác** câu *"Chương sau chưa xong"* — **and** không nút
  sang Workspace nào hiện ra.
- **Given** Chương đang mở **chưa** ở trạng thái *Đã xong*, **when** vào Chế độ đọc, **then** không
  một câu nào của nó hiện ra, **and** màn hình nói rõ **vì sao** trang trống (Chương đang mở chưa
  xong) chứ không im lặng, **and** mốc biên trỏ vào chính Chương đó.
- **Given** một Chương *Đã xong* có ba câu **chưa xác nhận**, **when** trang đọc render, **then**
  đúng ba câu ấy mang gạch chấm nhẹ và những câu còn lại **không** mang, **and** dấu ấy đến từ
  `is_confirmed` trên dây chứ không từ một phép đoán ở webview — gỡ trường đó khỏi dữ liệu làm
  adapter **từ chối** cả lượt trả về thay vì vẽ mọi câu như đã hoàn chỉnh.
- **Given** một Chương được đặt *Đã xong* **bằng tay** trong khi còn câu chưa xác nhận, **when** đọc,
  **then** hành vi y hệt gạch trên — không đường mã nào phân biệt *đã xong bằng tay* với *đã xong*.
- **Given** một Chương *Đã xong* mà mọi câu đều đã cắt bỏ, và một Chương *Đã xong* rỗng, **when**
  chúng nằm trong dãy đọc, **then** mỗi Chương hiện tiêu đề kèm **hai câu khác nhau** nói vì sao nó
  không có nội dung, **and** không câu nào khẳng định *"chưa có gì"* trong lúc lượt nạp còn đang bay.
- **Given** một `chapter.status` mang giá trị ngoài bốn giá trị hợp lệ, **when** dãy đọc được dựng,
  **then** dãy **dừng trước** Chương đó — một giá trị không đọc được **không bao giờ** được coi là
  *Đã xong*.
- **Given** cả hai theme, **when** `check:tokens` chạy, **then** Kiểm B/C/D/F/H xanh và **0** cặp màu
  mới — gạch chấm dùng token `ornament` đã kiểm.
- **Given** toàn bộ story, **when** `check:commands` chạy, **then** Kiểm A xanh (mọi `@click` mới là
  đúng một `dispatch` literal), Kiểm B xanh, ba sàn nội dung không giảm, **and** `createKeymap`
  không ném — không hợp âm nào bị hai lệnh giành.

## Spec Change Log

## Review Triage Log

### 2026-08-30 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 10: (high 0, medium 6, low 4)
- defer: 0
- reject: 3: (high 0, medium 1, low 2)
- addressed_findings:
  - `[medium]` `[patch]` Khối `.frontier` không có phép kiểm nào CHẠY TỰ ĐỘNG đọc chữ đã render — mọi ca vitest chỉ đọc `state.readingRun.frontier.*`, ca duy nhất đọc `textContent` là bàn đo e2e, mà `ci.yml:68` cho `test:e2e` chạy ở `schedule` chứ không ở `push`. Đảo `v-if`/`v-else` của hai câu mốc biên thì mọi lượt push vẫn xanh. Vá: `tests/frontend/readingFrontierDom.test.ts` (mới) mount `ReadingMode.vue` thật, khẳng định câu nào render theo `kind`, nút *Dịch tiếp* có/không mặt, và `status = "finished"` cho ra chuỗi thô.
  - `[medium]` `[patch]` `chapterEmptyNote()` chưa được khẳng định ở tầng DOM — hai ca cũ chỉ đọc `role="status"` cấp TOÀN DÃY, nên đảo ternary làm một Chương bị cắt bỏ hết câu hiện *"chưa có câu nào"* mà không ca nào đỏ. Vá: ca dãy HAI Chương (`segment_count: 0` và `segment_count > 0` + `paragraphs: []`), khẳng định `.chapter-note` của từng Chương.
  - `[medium]` `[patch]` Vế bàn phím của AC *"bấm nút sang Workspace bằng chuột HOẶC bằng `Tab` + `Enter`"* không có phép kiểm nào — bàn đo e2e chỉ dùng `realClick()`. Vá: khẳng định ở tầng component rằng phần tử là `<button type="button">` thật, không `disabled`, không `tabindex="-1"`.
  - `[medium]` `[patch]` Vòng lặp Chương sinh MỘT `<h1>` mỗi Chương, nên một dãy bốn Chương cho bốn `<h1>` ngang hàng rồi `<h2>` của mốc biên treo dưới cái cuối — cây tiêu đề nói sai cấu trúc trang, và không cổng nào canh. Story 5.11 có đúng MỘT `<h1>`. Vá: mỗi Chương bọc `<section>`, tiêu đề Chương hạ xuống `<h2>`, mốc biên giữ `<h2>` ngang hàng.
  - `[medium]` `[patch]` `KNOWN_LIFECYCLE_STATUSES` phía TS là bản chép tay của `LifecycleStatus` mà không cổng nào nối hai đầu — `lifecycle_contract.rs::every_lifecycle_status_label_key_exists_in_vi_json` đã nối Rust → `vi.json`, nhưng mắt xích `vi.json` → bảng TS thì đứt, nên một giá trị HỢP LỆ đổi ở Rust sẽ âm thầm rơi vào nhánh *"giá trị lạ"*. Vá: GỠ bản chép, thay bằng `hasMessageKey()` (`src/i18n/index.ts`) đọc thẳng `vi.json`, cộng `tests/frontend/i18nHasMessageKey.test.ts`.
  - `[medium]` `[patch]` Chưa fixture nào trộn `is_omitted` với một câu `draft` trong CÙNG một Chương `done` — hai bộ lọc chạy trên cùng dãy segment nhưng chỉ được kiểm rời nhau. Vá: ca Rust `omitted_and_unconfirmed_sentences_coexist_correctly_in_the_same_done_chapter`.
  - `[low]` `[patch]` `chapterLabel()` bị chép hai bản — template `<h1>` viết thẳng lại chính biểu thức `chapter_title ?? untitled(chapter_ord)` vài dòng dưới helper. Vá: template gọi helper.
  - `[low]` `[patch]` `mode.reading.frontier_range` đọc thành *"từ Chương 1 đến Chương 1"* với dãy MỘT Chương — và đó là ca thường nhất hôm nay (chưa đường sản phẩm nào tạo Chương thứ hai ngoài `split_chapter_at_segment`). Vá: khoá `frontier_range_single` + nhánh riêng, kèm ca kiểm.
  - `[low]` `[patch]` Mục nợ quy mô (chủ Story 5.14) mới kể MỘT nửa chi phí — nó bỏ qua rằng truy vấn đầu tiên của `read_reading_run` quét MỌI hàng Chương của Tác phẩm bất kể dãy dừng ở đâu, một chi phí O(tổng số Chương) thứ hai độc lập. Vá: `deferred-work.md` sửa tại chỗ, cùng mục cùng chủ, không mở mục thứ hai.
  - `[low]` `[patch]` Tên lớp `.frontier-blocked` dùng cho CẢ nhánh `end-of-work` — nói sai một nửa số ca. Vá: đổi thành `.frontier-note`, thêm `data-reading-frontier-kind`.

**Ba mục BÁC, kèm lý do — ghi ra vì một mục bị bác đáng đọc lại bằng tiền đề khác:**
- `[medium]` *"Bàn đo e2e chưa từng chạy nên AC1–AC4 chỉ được nghiệm thu bằng `happy-dom` trên `invoke` giả."* — **tiền đề hết đúng**. Mệnh đề ấy đọc từ §Rủi ro còn lại của lượt cài; bàn đo ĐÃ chạy sau đó và xanh (`webkit 605.1.15 macos`, `1 passing`, ba lượt độc lập). Chỗ cần sửa là câu trong spec, không phải mã — đã gạch ngang tại chỗ.
- `[low]` *"`openFrontierInWorkspace` chỉ `console.error` mà không báo lên giao diện."* — đó ĐÚNG luật `src/AGENTS.md` (*"hàm chạy từ một hợp âm/lệnh KHÔNG BAO GIỜ ném — nó ghi chẩn đoán rồi trả"*), và nhánh ấy không tới được từ sản phẩm: nút chỉ render khi `kind === 'next-not-done'`, và lệnh không có hợp âm mặc định.
- `[low]` *"Thêm một trần `MAX_RUN_CHAPTERS` cho dãy đọc."* — bác trên **thẩm quyền của đặc tả**, không trên spec: FR120 nói *"đọc liên tục qua các Chương Đã xong"* không kèm trần, và kho cấm cắt ngầm (*"không cổng nào được cắt bớt trong im lặng"*). Nửa ĐÚNG của mục này KHÔNG bị bỏ: nó thành bản vá số 8 — mục nợ quy mô nay kể đủ hai chi phí.


## Design Notes

### 🔴 Vì sao *"Chương đang mở chưa xong ⇒ không hiện gì"* là đường ĐÚNG, không phải một lượt gỡ năng lực

Story 5.11 đọc Chương đang mở **bất kể trạng thái** — không phải vì nó ký điều đó, mà vì `status`
chưa được nhìn tới lần nào (`read_reading_chapter` không `SELECT` cột ấy). AC của 5.11 **không** có
một dòng nào về trạng thái Chương. Vậy lượt này không gỡ một mệnh đề đã ký; nó điền vào chỗ trống.

Và hai vế của FR120 chọn giúp:

- *"Đọc liên tục qua các Chương ở trạng thái **Đã xong**"* — điều kiện đứng trên **mọi** Chương của
  dãy, không riêng Chương thứ hai trở đi.
- Câu mở đầu của chính story: *"Chế độ đọc **không bao giờ** ném nguyên văn tiếng Trung vào giữa
  trang đọc tiếng Việt"*. Một Chương đang dịch dở là **đúng** ca đó, và nó là ca **thường nhất** —
  người dùng vừa rời Editor.

⇒ Đọc *"trừ Chương đang mở ra"* biến chữ *"không bao giờ"* thành *"trừ lúc bạn hay gặp nhất"*.

⚠️ **Cái giá, ghi ra thay vì giấu:** bấm `⌘3` khi đang dịch dở nay cho một trang chỉ có mốc biên.
Đó là lý do nhánh `'frontier-only'` phải có **câu chữ của riêng nó** — nếu nó dùng lại câu *"Chương
này chưa có câu nào"* thì người dùng đọc thành một lỗi dữ liệu.

### Vì sao mốc biên chở NHÃN TRẠNG THÁI, không chỉ tên Chương

*"Chương 48 chưa dịch xong nên không hiện ở đây"* (mockup) đúng cho ca `in_progress`. Nhưng bốn
trạng thái thì có **ba** cách một Chương không phải *Đã xong*: *Chưa bắt đầu* · *Đang dịch* ·
*Tạm ngưng*. Ba câu trả lời khác nhau cho câu hỏi *"tôi nên làm gì tiếp"*. Nhãn đã có sẵn trong
`vi.json:208-211` và `LifecycleStatus::label_key()` đã bắt buộc mỗi giá trị mang một khoá — dùng lại,
đừng đúc một câu chung nói ít hơn dữ kiện đang có.

⚠️ Và vì `status` là chuỗi tự do ở tầng SQL, mốc biên chở **chuỗi thô**; webview tra nhãn qua bảng
`lifecycle.*`, **không khớp** thì hiện chuỗi thô. Đó là ca *"giá trị lạ"* của §I/O Matrix, và nó phải
đọc lên như một điều bất thường chứ không rơi vào một nhãn nào đó.

### Vì sao `segment_count` lên dây, và vì sao nó GỠ được một nhánh

`readingState.ts:16-25` hôm nay mô tả một lượt IPC **phụ** (`listChapters()`) chỉ để phân biệt
*Chương rỗng* với *mọi câu đã cắt bỏ*, cộng một nhánh thứ tám `'empty-unknown'` cho ca lượt phụ ấy
**trượt**. Với một dãy Chương, cách đó nhân lên: mỗi Chương rỗng lại là một câu hỏi, và
`ChapterRow` phải được ghép theo `chapter_id` ở webview.

Dữ kiện ấy đã nằm ngay trong lượt đọc đang chạy — dãy `ChapterSegment` **trước** khi lọc cắt bỏ có
đúng độ dài cần đếm. Đưa nó lên dây tốn một `i64` mỗi Chương và **xoá**: một vòng IPC, một phép ghép
ở webview, và cả một nhánh trạng thái. Một nhánh biến mất vì **nguyên nhân** của nó biến mất là cách
đúng để một nhánh chết — khác hẳn việc gộp nó vào nhánh khác cho gọn.

### `↵` của mốc biên: cùng hạng món nợ `⌘,`, và story này thi hành đường KHÔNG PHÁ

Mockup vẽ `↵` trên nút *Dịch tiếp*. **AC không đòi một hợp âm nào** — nó đòi *"một đường sang
Workspace"*, và một `<button>` với `Tab` + `Enter` là một đường đầy đủ theo NFR17.

Phép đo: `src/commands/registry.ts` **không có** khái niệm *"hợp âm chỉ sống trong một chế độ"*
(§Design Notes Story 5.11 đã ghi và đo). Một `Enter` **trần** đăng ký toàn ứng dụng vì thế bắn cả ở
Library (nơi `Enter` là cử chỉ tự nhiên để mở Tác phẩm đang chọn) và ở Workspace. `keys.ts:418-437`
chỉ chặn trong vùng gõ — nó **không** cứu hai ca kia.

⇒ Lệnh đăng ký **0 hợp âm**, đúng khuôn `reading.toggle_tuner` (5.11) và `library.open_search_hit`.
Vế `↵` thành **món nợ có chủ Ice**, và như món `⌘,`: `ChordOverrides` (Story 1.21) cho phép Ice tự
gán ngay hôm nay — cái thiếu là **mặc định**, không phải **năng lực**.

### Nạp trọn dãy trong một lượt — phép đo, và món nợ có chủ

Đường đọc nạp **mọi** segment của **mọi** Chương `done` trong dãy vào bộ nhớ một lần.

**Đo tại hôm nay (2026-08-30):** chưa đường sản phẩm nào tạo Chương thứ hai ngoài
`split_chapter_at_segment` (Story 5.8) — `epic-5-context.md` §Cross-Story Dependencies nói thẳng
điều đó, và FR14 (nhập hàng loạt) là **Epic 6**. Một Tác phẩm thật hôm nay có 1–2 Chương, tức cùng
hạng chi phí với `read_open_chapter_segments` đang chạy. Đó là lý do vỏ IPC **không** `(async)`:
bảng `count_async_attrs` dành cho **vỏ CHẶN**, và thêm `segment.rs` vào bảng đó hôm nay là khai một
mệnh đề chưa đo được.

⚠️ **Ngưỡng mà nó hỏng, ghi ra trước:** ở quy mô Epic 6 (5.000 Chương giả định của NFR3/NFR4), một
Tác phẩm dịch xong hoàn toàn cho một dãy **toàn bộ Tác phẩm** — vượt trần bộ nhớ nhàn rỗi 300 MB
trước khi vượt bất cứ thứ gì khác. ⇒ Món nợ **có chủ: Story 5.14**, đóng cùng lượt đo ba ngưỡng ấy;
đây không phải một lời nhắc trôi nổi.

### Hai bản chép của một câu `SELECT` — đóng nó ở đây, không hoãn thêm

Story 5.11 tự ghi vào §Rủi ro còn lại: *"`read_reading_chapter` chép lại câu `SELECT` của
`read_open_chapter_segments`; sửa bộ lọc ở một nơi làm Chế độ đọc và Workspace nói khác nhau về cùng
một Chương, không cổng nào đỏ"*. Lượt này **thêm** một chỗ gọi thứ ba (mỗi Chương trong dãy), nên để
nguyên là nhân bản chép lên ba. Task 1 rút thành một hàm trước khi thêm chỗ gọi — cùng kỷ luật mà
`omit::count_in_translation` đã ghi cho vị từ `!is_omitted`.

## Verification

**Commands:**

- `npm run check:tokens` — expected: Kiểm A–H xanh, **0** cặp màu mới, **0** màu viết thẳng.
- `npm run check:i18n` — expected: Kiểm A/A2/B/C/D/E xanh; **0** khoá mồ côi sau khi gỡ
  `mode.reading.status_empty_unknown`.
- `npm run check:commands` — expected: Kiểm A/B/D/E xanh, ba sàn nội dung không giảm.
- `npm run check:panel-refs` — expected: mọi ô mới của `readingState.ts` đi qua một `reset*` hoặc một
  miễn trừ CÓ TÊN; **0** miễn trừ chết (`chapterHasSegments` bị gỡ ⇒ mọi miễn trừ trỏ vào nó cũng phải
  đi cùng lượt).
- `npm run check:layout` — expected: xanh, **không** thành viên `window.`/`document.` mới.
- `npm run test` (vitest) — expected: hai tệp mới xanh; `readingState.test.ts` đã sửa xanh.
- `npm run build && cargo test --locked --manifest-path src-tauri/Cargo.toml` — expected:
  `segment_contract.rs` bộ ca mới xanh · `ipc_contract.rs` hình dạng dây mới xanh ·
  `config_invariants.rs` **không đổi** · `segment_boundary.rs` xanh.
- `npm run test:e2e -- --spec e2e/specs/story-5-12-reading-frontier.e2e.mjs` — expected: bốn mệnh đề
  xanh trong WKWebView thật. ⚠️ `story-5-6-library-grid.e2e.mjs` đỏ từ baseline — không đọc thành hồi quy.

**Đối chứng ĐỎ bắt buộc (làm rồi hoàn nguyên, đối chiếu bằng `diff`, ghi kết quả vào §Auto Run Result):**

1. **GỠ** vị từ `LifecycleStatus::from_wire(...) == Some(Done)` khỏi `read_reading_run` (nhận mọi
   Chương) ⇒ ca *"Chương chưa xong không ra dây"* phải **ĐỎ**, và **chỉ** nhóm ca ấy.
2. **ĐỔI** vị từ đó thành một phép so chuỗi `status != "done"` (tức coi giá trị lạ là `done`) ⇒ ca
   *"`status = \"finished\"` chặn dãy"* phải **ĐỎ**.
3. **GỠ** `is_confirmed` khỏi `isReadingSegment` (để trường lọt qua như `undefined`) ⇒ ca vitest
   *"adapter từ chối cả lượt trả về"* phải **ĐỎ**; `vue-tsc` vẫn **sạch** — đó chính là phép đo chứng
   minh trình biên dịch không canh được đường này.
4. **ĐỔI** `frontier` sang luôn `EndOfWork` ⇒ ca *"mốc biên nêu đích danh Chương chặn"* phải **ĐỎ**,
   và mệnh đề ③ của bàn đo e2e phải **ĐỎ**.
5. **GỠ** lời gọi `resetReading()` trong `openFrontierInWorkspace()` ⇒ ca vitest *"state đọc bị vứt"*
   phải **ĐỎ** (nếu không đỏ thì ca ấy đang kiểm một thứ khác — nghi ca trước, nghi bộ test sau).

## Auto Run Result

### Đã dựng

- **Rust** (`src-tauri/src/commands/segment.rs`): `select_chapter_segments` (Task 1, bản DUY NHẤT
  của câu `SELECT` chín cột, dùng chung bởi `read_open_chapter_segments` và `read_reading_run`);
  năm struct dây mới/sửa (`ReadingSegment` +`is_confirmed`, `ReadingChapter` +`segment_count`,
  `ReadingFrontierKind`, `ReadingFrontierChapter`, `ReadingFrontier`, `ReadingRun`); hàm thuần
  `read_reading_run` thay `read_reading_chapter` (Task 2–3); vỏ `wire::read_reading_run` đổi tên
  tại chỗ (Task 4); doc-comment cũ đã hết đúng sửa tại chỗ kèm 🔵 (Task 5, cả ở `segment.rs` lẫn
  `core/segment/omit.rs`). `lib.rs` đổi tên trong `generate_handler!`.
- **Rust — cổng**: khối test mới trong `segment_contract.rs` (12 ca, mỗi tên một câu, phủ trọn
  §I/O Matrix — 5 ca kế thừa từ Story 5.11 sửa tại chỗ theo hình dạng `ReadingRun` + 10 ca mới của
  Story 5.12); `ipc_contract.rs` sửa tại chỗ hai test đóng băng khoá dây + đăng ký vỏ, đóng băng
  thêm hai chuỗi biến thể `ReadingFrontierKind` và bất biến `chapter` Some/None theo `kind`.
- **Frontend**: `src/config/reading.ts` viết lại hoàn toàn theo hình dạng `ReadingRun` (Task 8, đủ
  vị từ kiểm kiểu MỌI trường/MỌI phần tử, kể cả bất biến frontier); `src/modes/readingState.ts`
  nhóm ① viết lại (Task 9: gỡ `chapterHasSegments`/lượt `listChapters()` phụ/nhánh
  `'empty-unknown'`, thêm `openFrontierInWorkspace`); `src/modes/ReadingMode.vue` (Task 10–11: vòng
  lặp Chương ngoài `.page`, khối `.frontier`, lớp `.unconfirmed`); lệnh
  `reading.continue_in_workspace` (0 hợp âm mặc định, Task 12) tiêm ở `main.ts` (Task 13); khoá
  `vi.json` mới/sửa (Task 14 — xem §Quyết định bên dưới cho cách xử lý `status_empty`).
- **Test frontend mới**: `tests/frontend/readingFrontier.test.ts` (Task 15, 10 ca — bốn hình dạng
  `ReadingRun`, ba ca `openFrontierInWorkspace`, ba ca từ chối bất biến `kind ↔ chapter`) và
  `tests/frontend/readingUnconfirmed.test.ts` (Task 16, 4 ca — gạch chấm đúng câu, không câu nào
  sai, và đối chứng ĐỎ bắt buộc #3). `tests/frontend/readingState.test.ts` sửa tại chỗ (Task 17):
  fixture đổi sang `ReadingRun`, xoá describe *"empty-unknown"* (nhánh đã chết), giữ nguyên ca
  *"is_omitted không bao giờ tới webview"* và ca hợp âm trần. `tests/frontend/readingTypography.test.ts`
  sửa một dòng (`readingChapter` → `readingRun`).
- **e2e mới**: `e2e/specs/story-5-12-reading-frontier.e2e.mjs` (Task 18) — dựng Tác phẩm, gõ bản
  dịch thật, tách Chương, đặt trạng thái qua IPC trần, `⌘3`, đo bốn mệnh đề của AC2/AC3/AC4. 🔵
  **ĐÃ CHẠY THẬT (review vòng bốn lớp)** — xem §Nghiệm thu; không còn "chưa chạy".
- **Review vòng bốn lớp (2026-08-30)**: mười bản vá — xem §Vòng rà bốn lớp bên dưới cho danh sách
  đầy đủ. Tóm tắt mã nguồn: `src/i18n/index.ts` thêm `hasMessageKey()`; `ReadingMode.vue` gỡ
  `KNOWN_LIFECYCLE_STATUSES`, đổi mỗi Chương từ `<div class="page"><h1>` sang
  `<section class="page"><h2>`, dùng `chapterLabel()` thay vì viết tay lần hai,
  `.frontier-blocked` → `.frontier-note`, thêm khoá `frontier_range_single`; `vi.json` thêm khoá
  đó; `segment_contract.rs` thêm một ca trộn `is_omitted`+`draft`; `deferred-work.md` sửa tại chỗ
  mục nợ Story 5.14 (thêm chi phí ②). Hai tệp test mới: `readingFrontierDom.test.ts`,
  `i18nHasMessageKey.test.ts`.
- **Sổ nợ** (Task 19): đóng vế FR120 của mục "Chế độ đọc → Epic 5"; thu hẹp cả hai mục vị trí
  cuộn (trước "5.12 hoặc 5.13") về **Story 5.13 duy nhất**; mở hai mục mới có chủ (`↵` mốc biên —
  Ice; dãy đọc nạp trọn bộ nhớ — Story 5.14). `src/modes/README.md` (Task 20) sửa tại chỗ.

### Quyết định của người thực thi (chưa qua Ice) — nêu ra để Ice soát lại

Task 14 nói *"sửa tại chỗ `status_empty`/`status_all_omitted` nếu câu cũ nói về 'Chương này'"*.
Vì nhánh `readingStatusKind` cấp Chương-đơn (`'empty-chapter'`) không còn tồn tại — thay bằng
`'empty-chapters'` cấp CẢ DÃY — hai khoá được xử lý KHÁC nhau, và đây là một lựa chọn cần Ice
soát:
- `status_all_omitted` **giữ nguyên tên khoá**, sửa CÂU để nói về cả dãy (branch name không đổi).
- `status_empty` (cũ, "Chương này chưa có câu nào") **bị XOÁ** — vai của nó tách làm hai: khoá
  MỚI `status_empty_chapters` (cấp dãy) và khoá MỚI `chapter_empty` (cấp từng Chương trong vòng
  lặp `.page`, dùng nguyên câu cũ). Xoá thay vì "sửa tại chỗ" vì giữ `status_empty` lại sẽ là một
  khoá mồ côi — không nhánh code nào còn gọi nó.
⇒ Nếu Ice muốn giữ tên khoá `status_empty` cho một trong hai vai trên, đó là một sửa nhỏ, cục bộ.

### Nghiệm thu — lệnh đã chạy, kết quả

- `npm run check:tokens` — **OK**, Kiểm A–H xanh, 0 cặp màu mới (`.unconfirmed`/`.frontier` chỉ
  dùng token đã kiểm: `--color-ornament`, `--color-outline`, `--color-on-surface-variant`, các
  token `--font-ui-*`).
- `npm run check:i18n` — **OK**, Kiểm A/A2/B/C/D/E xanh, 566 khoá (từ 557 tại baseline `e36599e`
  — đo bằng `python3 -c "import json; print(len(json.load(open('src/i18n/vi.json'))))"` trên cả
  hai đầu, net **+9**: 11 khoá mới của Task 14 trừ 2 khoá gỡ, `status_empty` +
  `status_empty_unknown`).
- `npm run check:commands` — **OK**, Kiểm A–I xanh; `reading.continue_in_workspace` xuất hiện đúng
  một lần trong danh sách `unbound()` (0 hợp âm mặc định, đúng §Design Notes).
- `npm run check:panel-refs` — **OK**, 61 tệp `.ts`, 253 ô nhớ cấp module, 31 miễn trừ có tên, tự
  kiểm xanh.
- `npm run check:layout` — **OK**, 17 thành viên `window`/`document` — không tăng.
- `npm run check:debt-owner` — **OK**, 0/402 mục mở thiếu `Chủ:` (kể cả hai mục mới của Task 19).
- `npm run check:gates` / `check:deps` — **OK**, không đổi (story không thêm cổng/phụ thuộc).
- `npm run test` (vitest) — **OK**, 55 tệp / 753 ca xanh. 🔵 **SỬA (review vòng bốn lớp)** — thêm
  hai tệp mới ở lượt vá Bản vá 1–3/6/7: `readingFrontierDom.test.ts` (9 ca — khối `.frontier`
  render đúng chữ, `.chapter-note` từng Chương, hình dạng nút *Dịch tiếp*, dãy một/hai Chương) và
  `i18nHasMessageKey.test.ts` (2 ca). Từ 51 tệp / 730 ca trước story ban đầu.
- `npx vue-tsc --noEmit` — **OK**, sạch.
- `npm run check:lint` — **OK sau một lượt sửa**: `ReadingMode.vue::frontierRangeText` đọc
  `run.chapters[0]` (chỉ số mảng trần) bị `@typescript-eslint/no-unnecessary-condition` chấm là
  "không thể `undefined`" (thiếu `noUncheckedIndexedAccess`) — cùng đúng cái bẫy mà
  `readingState.ts::openCurrentTocChapter` đã ghi chú cho `.at(...)`. Sửa `[0]` → `.at(0)`, cổng
  xanh lại.
- `npm run build` — **OK**, `vite build` thành công.
- `cargo test --locked` (toàn workspace) — **OK sau một lượt sửa**: lượt đầu `store_boundary.rs`
  đỏ (`only_core_store_may_name_rusqlite`) vì `commands/segment.rs` import thẳng
  `rusqlite::Connection` thay vì `core::store::ReadHandle` đã tái xuất (AD-11) — sửa tại chỗ, lượt
  chạy lại sau đó xanh toàn bộ. `segment_contract.rs` 147 ca (từ 121 trước — +26 ca ròng, kể cả 5
  ca kế thừa sửa tại chỗ + 10 ca mới của lượt đầu + 1 ca của Bản vá 10 ở review vòng bốn lớp),
  `ipc_contract.rs` 16 ca, `segment_boundary.rs` 8 ca, `config_invariants.rs` không đổi,
  `store_boundary.rs` 4 ca — tất cả xanh.
  ⚠️ **Máy chạy lượt này bận** (nhiều tiến trình agent khác cùng chạy song song) — một lượt
  `cargo test --locked` giữa chừng cho một test KHÔNG liên quan (`matching_boundary.rs`) TREO
  do tranh chấp tài nguyên; chạy CÔ LẬP lại đúng tệp đó xong trong 0,05s (10/10 ca xanh), và một
  lượt `cargo test --locked` CÔ LẬP thứ hai (không tiến trình nặng nào khác chạy cùng) đi hết
  toàn bộ ~40 test binary + 3 doctest KHÔNG một chữ `FAILED`/`panicked` nào trong log — kể cả
  `store_contract.rs` (17 ca) và ba doctest của `core::store`/`core::scope`. Ghi ra đúng luật
  *"hai lượt đo phải cùng tải máy"* — số đo timeout không phải một mệnh đề về code một mình.
- `npm run test:e2e -- --spec e2e/specs/story-5-12-reading-frontier.e2e.mjs` — 🔵 **SỬA (lượt rà
  2026-08-30, review vòng bốn lớp) — mệnh đề "CHƯA CHẠY" của bản đầu ĐÃ HẾT ĐÚNG.** ĐÃ CHẠY THẬT
  trên `webkit 605.1.15 macos`: `1 passing (41,9s)` ở lượt đầu, `Spec Files: 1 passed, 1 total`.
  Chạy lại LẦN THỨ HAI sau đối chứng ĐỎ #4 mở rộng (xem bên dưới) vẫn xanh: `1 passing (45,8s)`.
  Bốn mệnh đề của AC2/AC3/AC4 đã nghiệm thu bằng máy ở tầng engine thật, không chỉ ở `happy-dom`.

### Đối chứng ĐỎ đã chạy thật (làm rồi hoàn nguyên, đối chiếu bằng `diff`)

Cả năm đối chứng chạy trên baseline ĐÃ XANH (không chạy trên một cây đang hỏng), mỗi lượt sửa một
dòng, chạy cổng liên quan, ghi kết quả, rồi hoàn nguyên đúng dòng đó — `git diff` rỗng sau mỗi lượt.

1. **GỠ vị từ `Done`** (`is_done = true` không điều kiện) — `cargo test --test segment_contract`:
   **ĐỎ đúng 3 ca** (`a_continuous_run_stops_before_a_chapter_that_is_not_yet_done`,
   `an_unknown_status_value_blocks_the_run_and_travels_verbatim_on_the_frontier`,
   `opening_a_not_done_chapter_yields_an_empty_run_with_the_frontier_on_itself`) — đúng nhóm ca
   "Chương chưa xong không ra dây", 143 ca còn lại vẫn xanh.
2. **ĐỔI vị từ thành so chuỗi coi giá trị lạ là done** (`is_done = status != "in_progress"`, một
   hình dạng bug cụ thể hoá cho mô tả "coi giá trị lạ là done" — so sánh trực tiếp
   `status != "done"` không đổi hành vi vì `LifecycleStatus::from_wire` vốn đã so khớp CHÍNH XÁC
   chuỗi `"done"`, nên một phép so chuỗi tương đương không tạo khác biệt quan sát được trên bất kỳ
   input nào; hình dạng bug thật — liệt kê các giá trị KHÔNG-done thay vì so khớp giá trị done —
   mới tạo ra đúng lớp lỗi §Design Notes mô tả) — `cargo test --test segment_contract`: **ĐỎ đúng
   1 ca** (`an_unknown_status_value_blocks_the_run_and_travels_verbatim_on_the_frontier`), 145 ca
   còn lại xanh.
3. **GỠ `is_confirmed` khỏi `isReadingSegment`** — `npx vitest run tests/frontend/readingUnconfirmed.test.ts`:
   **ĐỎ đúng 2 ca** (ca "gỡ `is_confirmed`… ⇒ `{ run: null, error: ipc.unknown }`" và ca
   `readingStatusKind` "error"), 2 ca còn lại trong tệp xanh; `npx vue-tsc --noEmit`: **sạch** —
   xác nhận đúng mệnh đề "trình biên dịch không canh được đường này".
4. **ĐỔI `frontier` thành luôn `EndOfWork`** — `cargo test --test segment_contract`: **ĐỎ đúng 3
   ca**, cùng ba ca của đối chứng #1 (lý do khác: `left: EndOfWork, right: NextNotDone`) — đúng
   nhóm "mốc biên nêu đích danh Chương chặn". 🔵 **SỬA (review vòng bốn lớp) — câu "mệnh đề ③ của
   bàn đo e2e không chạy được" HẾT ĐÚNG; lý do thật là NÓ CHƯA CHẠY, không phải KHÔNG CHẠY ĐƯỢC.**
   Chạy lại đúng mutation này qua chính bàn đo e2e (rebuild + `npm run test:e2e -- --spec
   e2e/specs/story-5-12-reading-frontier.e2e.mjs`): **ĐỎ đúng mệnh đề ③**
   (`expect(received).toContain(expected)` — `Expected substring: "Chương 2"`, `Received string`
   không chứa nó, `1 failing (16,7s)`), rồi hoàn nguyên và chạy lại xanh (`1 passing (45,8s)`).
   Đối chứng #4 nay chạy CẢ HAI tầng, không chỉ tầng Rust.
5. **GỠ `resetReading()` trong `openFrontierInWorkspace()`** —
   `npx vitest run tests/frontend/readingFrontier.test.ts`: **ĐỎ đúng 1 ca** (*"`openChapterById`
   trả `true` ⇒ vứt TOÀN BỘ state đọc rồi chuyển Workspace"*), 9 ca còn lại trong tệp xanh.

Sau cả năm lượt: `git diff` trên `src-tauri/src/commands/segment.rs` và `src/config/reading.ts` và
`src/modes/readingState.ts` xác nhận không còn dấu vết mutation nào; `cargo build` và `npm run
test` chạy lại lần cuối đều xanh.

### Vòng rà bốn lớp (2026-08-30) — mười bản vá, ba mục bị bác

Toàn bộ mục `patch` đã sửa trong cùng một lượt, mỗi bản vá kèm một đối chứng ĐỎ THẬT (làm rồi
hoàn nguyên). §Verification chạy lại TRỌN sau khi vá xong — xem các mục Nghiệm thu ở trên (đã sửa
tại chỗ) cho kết quả cuối.

- **Bản vá 1** — thêm `tests/frontend/readingFrontierDom.test.ts`: mount `ReadingMode.vue` thật,
  khẳng định `.frontier-note[data-reading-frontier-kind="…"]` render đúng câu theo `kind`, nút
  *Dịch tiếp* CÓ/KHÔNG mặt đúng theo `kind`, và `status = "finished"` hiện CHUỖI THÔ. Đối chứng
  ĐỎ: đảo nội dung hai nhánh `v-if`/`v-else` của `.frontier` ⇒ ĐÚNG 3 ca liên quan đỏ, 2 ca còn
  lại (button/DOM-shape) không đụng.
- **Bản vá 2** — cùng tệp: dãy HAI Chương (một `segment_count = 0`, một `segment_count > 0` với
  `paragraphs = []`) trong CÙNG một `ReadingRun`, khẳng định `.chapter-note` của MỖI Chương nói
  ĐÚNG câu của nó. Đối chứng ĐỎ: đảo ternary trong `chapterEmptyNote()` ⇒ đúng 1 ca đỏ.
- **Bản vá 3** — cùng tệp: khẳng định nút *Dịch tiếp* là `<button type="button">` thật, không
  `disabled`, không `tabindex="-1"` (hình dạng phần tử, không mô phỏng Tab trong `happy-dom`).
  Đối chứng ĐỎ: thêm `tabindex="-1"` vào nút ⇒ đúng 1 ca đỏ.
- **Bản vá 4** — `ReadingMode.vue`: mỗi Chương bọc trong `<section>`, tiêu đề Chương hạ từ `<h1>`
  xuống `<h2>` (dùng `chapterLabel()`, xem Bản vá 5), mốc biên giữ `<h2>` ngang hàng — cây tiêu đề
  nay đọc tuyến tính "Chương 1, …, Mốc biên" thay vì N `<h1>` rồi một `<h2>` treo dưới Chương cuối.
- **Bản vá 5** — `chapter.chapter_title ?? untitled(chapter.chapter_ord)` viết tay lần thứ hai ở
  template đã gỡ; template gọi thẳng `chapterLabel(chapter)`.
- **Bản vá 6** — thêm khoá `mode.reading.frontier_range_single` + nhánh `run.chapters.length ===
  1` trong `frontierRangeText` — dãy ĐÚNG MỘT Chương (ca thường nhất hôm nay) không còn đọc "từ
  Chương 1 đến Chương 1". Hai ca mới trong `readingFrontierDom.test.ts` (dãy một Chương / dãy hai
  Chương). Đối chứng ĐỎ: gỡ nhánh `length === 1` ⇒ đúng 1 ca đỏ, tái hiện ĐÚNG câu bug mà review
  mô tả ("Lượt đọc này gồm từ Chương 1 đến Chương 1").
- **Bản vá 7** — gỡ hẳn `KNOWN_LIFECYCLE_STATUSES` (bản chép tay thứ ba của bốn giá trị
  `LifecycleStatus`); thêm `hasMessageKey()` xuất từ `src/i18n/index.ts`, đọc THẲNG `vi.json`.
  `frontierStatusLabel()` gọi hàm đó thay vì tra `Set`. Thêm `tests/frontend/i18nHasMessageKey.test.ts`
  (2 ca: bốn khoá thật có mặt, giá trị lạ trả `false`). Đối chứng ĐỎ: `hasMessageKey` luôn trả
  `false` ⇒ 2 ca đỏ trên HAI tệp khác nhau (`i18nHasMessageKey.test.ts` +
  `readingFrontierDom.test.ts`), đúng cả hai chỗ tiêu thụ.
- **Bản vá 8** — `deferred-work.md`: mục nợ quy mô Story 5.14 sửa tại chỗ kèm 🔵, kể THÊM chi phí
  ② (câu `SELECT` đầu tiên của `read_reading_run` quét MỌI hàng `chapter` của Tác phẩm, độc lập
  với chi phí ① đã ghi) — cùng mục, cùng chủ, không mở mục thứ hai.
- **Bản vá 9** — CSS/template: `.frontier-blocked` (dùng cho CẢ hai nhánh, kể cả `end-of-work` —
  "blocked" nói sai một nửa số ca) đổi tên `.frontier-note`; thêm `data-reading-frontier-kind`
  để test/bàn đo phân biệt nhánh mà không phụ thuộc câu chữ.
- **Bản vá 10** — `segment_contract.rs`: ca mới
  `omitted_and_unconfirmed_sentences_coexist_correctly_in_the_same_done_chapter` — một Chương
  `done` vừa có câu cắt bỏ vừa có câu chưa xác nhận, khẳng định câu cắt bỏ vắng mặt HOÀN TOÀN và
  câu chưa xác nhận còn lại mang `is_confirmed = false` ĐÚNG vị trí. Đối chứng ĐỎ: `is_confirmed:
  true` không điều kiện ⇒ đúng ca này đỏ.

**Ba mục bị bác — ĐỒNG Ý cả ba, không sửa gì:**
① e2e chưa từng chạy nên AC1–AC4 chưa nghiệm thu — mệnh đề đã hết đúng (xem mục ✅ ở dưới), review
tự sửa lại khi báo bản vá xong.
② `openFrontierInWorkspace` chỉ `console.error` — đúng `src/AGENTS.md`, đồng ý nguyên văn, không
đụng.
③ trần `MAX_RUN_CHAPTERS` — bị bác đúng thẩm quyền của FR120 ("không kèm trần"); Bản vá 8 đã làm
mục nợ Story 5.14 kể đủ hai chi phí thay vì thêm một trần ngầm.

### Rủi ro còn lại, ghi ra thay vì để người sau tưởng đã phủ

- ✅ ~~`e2e/specs/story-5-12-reading-frontier.e2e.mjs` CHƯA từng chạy.~~ **HẾT ĐÚNG (lượt rà
  2026-08-30, review vòng bốn lớp).** Mục này khai một giới hạn của MÔI TRƯỜNG thực thi lượt
  đầu, không một giới hạn của chính bàn đo — `npm run test:e2e -- --spec
  e2e/specs/story-5-12-reading-frontier.e2e.mjs` chạy THẬT được ở đây: `webkit 605.1.15 macos`,
  `1 passing` hai lần liên tiếp (41,9s và 45,8s, lần hai sau khi hoàn nguyên đối chứng ĐỎ #4).
  AC1–AC4 đã nghiệm thu bằng máy ở tầng engine thật, không chỉ suy luận từ `happy-dom`. Gạch
  ngang thay vì xoá, đúng luật *"mệnh đề đã hết đúng thì sửa tại chỗ, không xoá"*.
- ⚠️ Quyết định gỡ khoá `mode.reading.status_empty` (thay vì sửa tại chỗ) là một lựa chọn của
  người thực thi, chưa qua Ice — xem §Quyết định ở trên.
- ⚠️ `deferred-work.md` mở hai món nợ mới (hợp âm `↵` của mốc biên — chủ Ice; nạp trọn dãy vào bộ
  nhớ một lượt — chủ Story 5.14) đúng theo §Design Notes của chính spec này — không phải nợ phát
  sinh ngoài dự kiến.
- Không đổi `config_invariants.rs`/`count_async_attrs` — vỏ `read_reading_run` vẫn đọc-thuần,
  đúng §Design Notes; số đo quy mô (1–2 Chương/Tác phẩm hôm nay) không tự kiểm chứng được ở lượt
  này, chỉ trích dẫn lại phép đo đã có tại baseline `e36599e`.

### Lượt nghiệm thu ĐỘC LẬP của người điều phối — chạy lại từ đầu, không tin lời khai

🔴 Mọi con số dưới đây do người điều phối tự chạy trên cây làm việc cuối cùng, **tuần tự**, không
lượt nào chạy song song với lượt nào — luật *"hai lượt đo phải cùng tải máy"*, và chính lượt cài
này đã có một lần `cargo test` chạy chen làm một test KHÔNG liên quan treo.

| Đường | Trước vòng rà | Sau mười bản vá |
|---|---|---|
| 9 cổng tĩnh (`tokens` · `i18n` · `commands` · `panel-refs` · `layout` · `gates` · `debt-owner` · `deps` · `lint`) | 9/9 xanh | **9/9 xanh** |
| `npx vue-tsc --noEmit` | sạch | **sạch** |
| `npm run test` (vitest) | 744 ca · 53 tệp | **753 ca · 55 tệp** |
| `npm run build` | xanh | **xanh** |
| `cargo test --locked` | 989 ca · 35 binary | **990 ca · 35 binary** |
| `npm run test:e2e -- --spec e2e/specs/story-5-12-reading-frontier.e2e.mjs` | `1 passing (43s)` | **`1 passing (43,6s)`** |

Bàn đo e2e chạy trên `webkit 605.1.15 macos` — engine thật, không `happy-dom`.

⚠️ **Một phép đối chứng riêng, vì "989 ca xanh" KHÔNG chứng minh mười ca mới đã chạy.** Lượt
`cargo test` toàn bộ được lọc còn dòng `test result:`, nên nó không nói tên ca nào. Chạy riêng
`cargo test --locked --test segment_contract` cho hiện tên: **mười ca mới đều có mặt và `ok`**,
`146 passed; 0 failed; 0 filtered out`. Một ca tồn tại mà không chạy thì tính là thiếu — đây là
phép đo loại trừ chính điều đó.

**Hai đối chứng ĐỎ người điều phối TỰ chạy lại** (không nhận báo cáo), mỗi lượt sửa đúng một dòng
rồi hoàn nguyên, đối chiếu bằng `diff -q` với một bản sao lưu:

| Phép GỠ | Đo được | Hoàn nguyên |
|---|---|---|
| `is_done = LifecycleStatus::from_wire(…) == Some(Done)` → `true` | **đúng 3 ca ĐỎ**, 143 ca kia xanh; ba ca ấy đúng là ba ca canh cổng `Done` | byte-for-byte, `146 passed` trở lại |
| gỡ `typeof v.is_confirmed === 'boolean'` khỏi `isReadingSegment` | **đúng 2 ca ĐỎ**; ca đọc được nhất là `expected 'content' to be 'error'` — thiếu trường thì màn hình vẽ MỌI câu như đã xác nhận, đúng lớp nói dối mà AC6 tồn tại để chặn | byte-for-byte, `14 passed` trở lại |

### Kết quả vòng rà — 10 vá · 0 hoãn · 3 bác

Bốn lớp rà độc lập (blind · edge-case · verification-gap · intent-alignment), gói gửi rà mang
**trọn** diff kể cả `deferred-work.md` và chính tệp story — không tệp nào bị loại để giữ kích
thước *(lỗi quy trình của lượt rà Story 5.11, đã làm một lớp bác oan một mệnh đề đúng)*.

- **intent_gap: 0** · **bad_spec: 0** — không mục nào bắt nguồn từ `<intent-contract>`, và không
  mục nào cần dựng lại mã. Chín trên mười bản vá là **thêm phép kiểm** hoặc sửa chữ; đường Rust
  chọn dãy `Done` không đổi một dòng qua cả vòng rà.
- **patch: 10** — 6 `medium`, 4 `low`, 0 `high`. Chi tiết từng mục ở §Review Triage Log.
- **defer: 0** — không mục nào là lỗi có sẵn; món nợ quy mô đã có chủ (Story 5.14) từ lúc soạn spec,
  và nửa đúng của mục edge-case về nó đi vào **bản vá 8** thay vì thành một mục hoãn thứ hai.
- **reject: 3** — lý do từng mục ghi ở §Review Triage Log, không bác mục nào bằng một câu phán quyết.

🔴 **Phát hiện nặng nhất của cả vòng rà không phải một lỗi mã, mà là một khoảng hở CỔNG.** Toàn bộ
chữ nghĩa của mốc biên — thứ DUY NHẤT nói cho người dùng biết vì sao trang dừng — chỉ có bàn đo
e2e chạm tới, và `ci.yml:68` cho `test:e2e` chạy ở **`schedule`** (nhịp đêm) chứ không ở `push`.
Phép GỠ chứng minh: đảo `v-if`/`v-else` của hai câu ấy thì **mọi lượt push vẫn xanh**. Bốn bản vá
đầu kéo các mệnh đề đó về đường vitest, tức về `pre-push`. Đây đúng mệnh đề *"cổng xanh không phủ
đường nằm ngoài cổng"* mà `AGENTS.md` đã ghi bằng chữ.

**Khuyến nghị rà tiếp: `true`.** Đếm theo luật — chỉ tính mục triage `patch` của lượt này, không
tính `defer`/`reject`: 0 `high` · 6 `medium` · 4 `low` ⇒ điểm `3×6 + 1×4 = 22`, vượt ngưỡng 5.
Không mục `high` nào, nên `true` đến từ **số lượng**, không từ mức độ: mười bản vá trong một lượt
là dấu hiệu bề mặt này còn đáng soi thêm một vòng, không phải dấu hiệu có một lỗi nặng đang mở.
