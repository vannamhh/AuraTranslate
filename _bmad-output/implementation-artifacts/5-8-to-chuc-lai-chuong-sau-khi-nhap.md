---
title: 'Story 5.8: Tổ chức lại Chương sau khi nhập'
type: 'feature'
created: '2026-08-29'
status: 'done'
baseline_revision: '1bc8c400ac8c8d74efbae6aa6a23c2e67926a125'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
warnings: ['multiple-goals', 'oversized']
deferred:
  - summary: >-
      Gộp Chương XOÁ VĨNH VIỄN một hàng `chapter` chỉ bằng một lượt bấm — không xác nhận,
      không hoàn tác, và ranh giới Chương cùng tên Chương không dựng lại được.
    evidence: |-
      `merge_chapter_into_previous` chạy `DELETE FROM chapter WHERE id = ?1` trong cùng giao
      dịch với lượt dời segment. Nội dung KHÔNG mất (AD-32 giữ trọn `segment.id`, văn bản,
      lịch sử phiên bản, trạng thái xác nhận, và `source_text` nối thô), nhưng ba thứ thì mất
      và không có đường về: tên Chương bị gộp, ranh giới giữa hai Chương, và hàng
      `chapter_position` của nó. Lối "sửa" duy nhất là tách lại bằng tay tại đúng câu — và
      lượt tách đó lại dựng lại `source_text` từ segment, tức KHÔNG khôi phục được bản thô.
      `epics.md` §Story 5.8 không đòi một bước xác nhận, nên đây KHÔNG phải lệch spec; nhưng
      nút "Gộp vào Chương trước" nằm ngay cạnh ba nút không phá huỷ gì, cùng hình dạng, cùng
      kích thước. AD-48 (mô hình hoàn tác) đã có hồ sơ ở `planning-artifacts/`
      (`ad-brief-2026-08-17-mo-hinh-hoan-tac.md`) nhưng chưa soạn thành AD, nên hôm nay không
      có cơ chế chung nào để cắm vào.
    location: >-
      src-tauri/src/commands/chapter.rs — merge_chapter_into_previous ↔ src/modes/LibraryMode.vue
    severity: medium
---

<intent-contract>

## Intent

**Problem:** FR15 hứa sửa lại được **tên · thứ tự · ranh giới Chương** sau khi nhập mà không
mất công đã dịch, và AD-32 đã viết sẵn luật cho nó — nhưng hôm nay **0 đường mã nào** đổi
`chapter.title`, `chapter.ord`, hay `segment.chapter_id`: `INSERT INTO chapter` xuất hiện
đúng **một** lần trong toàn kho (`commands/project.rs:271`, trong `create_work`), và không có
`DELETE FROM chapter` nào. Mọi Tác phẩm vì thế có đúng một Chương, tên `NULL`, `ord = 1`.

**Approach:** Bốn thao tác **tổ chức** trên bề mặt IPC đã có (`commands/chapter.rs`, khuôn
hai lớp), mỗi thao tác đi qua **khuôn bốn bước** mà `commands/lifecycle.rs` đã dựng và đã có
đối chứng (ghi SQL → `WorkMeta::rebuild_from_store` → `write_atomic` → `reindex`): đổi tên ·
dời lên/xuống · gộp vào Chương liền trước · tách tại câu đang có caret. Ba thao tác đầu vào
khối Chương của `LibraryMode.vue` (Story 5.7 đã dựng); thao tác tách vào Editor, nơi duy
nhất một câu được chọn — cùng chỗ và cùng khuôn `editor.split_segment` (Story 2.8).

## Boundaries & Constraints

**Always:**
- **AD-32 theo nghĩa đen, và nó là mệnh đề nghiệm thu chính:** gộp/tách Chương đổi **đúng
  hai cột** trên các hàng `segment` liên quan — `chapter_id` và `ord`. Mọi cột khác
  (`id`, `source_text`, `target_text`, `status`, `retired_at`, `is_paragraph_end`,
  `is_target_paragraph_end`, `is_omitted`, `translation_origin`, `created_at`, `updated_at`)
  và **mọi hàng `segment_version`** giữ nguyên từng byte. **Không** `retired_at` nào được
  đặt — đây là điểm khác biệt cố ý với AD-5 (gộp/tách *segment*).
- **`ord` dời bằng một PHÉP TỊNH TIẾN HẰNG SỐ, không đánh số lại từ đầu.** Mọi hàng được dời
  — **kể cả hàng đã về hưu** — cộng/trừ cùng một số, nên thứ tự tương đối và điểm neo của
  hàng về hưu (AD-5: *"vẫn mở được về ĐÚNG VỊ TRÍ trong Chương"*) sống sót nguyên vẹn, và
  `ord` của mỗi Chương vẫn liên tục **từ 1** — tiền đề mà Story 2.10 (*"segment kế tiếp"*)
  đứng trên.
- **Chuẩn hoá `chapter.ord` về `1..N` theo `(ord, id)` TRƯỚC mỗi thao tác**, chỉ `UPDATE`
  những hàng thật sự đổi (`WHERE ord <> ?`). `chapter.ord` cố ý **không** `UNIQUE` và không
  hứa liên tục (`schema.rs` §`CHAPTER_DDL`), nên một phép `ord ± 1` trần sẽ trỏ sai hàng
  hoặc thành no-op im lặng trên `ord` thưa/trùng — đúng lớp lỗi mà `open_adjacent_chapter`
  đã phải viết so sánh bộ đôi để tránh.
- **Cả bốn thao tác là thao tác RỜI RẠC:** phía webview `flushEditorBeforeDiscreteWrite()`
  chạy TRƯỚC, đọc **cả ba** giá trị, và `'failed'`/`'still-dirty'` ⇒ **CHẶN kèm câu báo**
  (khuôn `libraryChapters.ts::openWorkById`). Phía Rust cả bốn đi qua **khuôn bốn bước**:
  một lượt tổ chức đổi `chapter_count`/`chapter_done_count`/`updated_at`, nên `meta.json` và
  `library-index.db` phải theo kịp trong cùng thao tác logic (AD-33/AD-8).
- **Một lượt ghi không đi đâu phải NÓI VÌ SAO.** Dời lên ở Chương đầu · dời xuống ở Chương
  cuối · gộp ở Chương đầu · tách ngay tại câu đầu Chương ⇒ `IpcError` **có tên**, không một
  `Ok` im lặng và không một hàng nào bị chạm.
- **Con trỏ `OpenWork::chapter_id` đổi SAU khi giao dịch commit, không trước** (luật đã ghi ở
  `open_adjacent_chapter`). Gộp xoá hàng Chương B ⇒ nếu B đang mở, con trỏ dời sang A.
- **Tên rỗng cắt bằng `str::trim()` của RUST, không `trim()` của SQLite.** `trim()` của
  SQLite chỉ cắt dấu cách ASCII (`src-tauri/AGENTS.md`, đo 2026-08-19); một tên chỉ gồm
  U+3000 sẽ lọt xuống đĩa thành một nhan đề "có chữ" mà màn hình hiện ra trống.

**Block If:**
- Một lượt đo cho thấy hợp âm `Mod+Shift+Slash` **đã bị chiếm** (hôm nay: `grep` trên
  `src/commands/index.ts` cho **0** hợp âm `Mod+Shift+*`) — chọn hợp âm mặc định khác là một
  quyết định về bảng phím người dùng, thuộc Ice.
- Một AC đòi đổi `segment.id` hoặc cho một segment về hưu để thi hành được — đó là AD-32 bị
  lật, tức một `AD` mới, không một dòng mã.

**Never:**
- **Không bước di trú mới.** Bốn thao tác chạy trọn trên lược đồ hiện có; bước cuối của
  `PROJECT_MIGRATIONS` giữ nguyên **17**.
- **Không tính lại ranh giới segment** (AD-4). `split_source_text` **không** được gọi ở bất
  kỳ đường nào của story này.
- **Không đụng `work.status_override`** và **không tự suy trạng thái Chương từ segment** —
  `chapter.status` vẫn chỉ đổi qua `commands::lifecycle::set_chapter_status`, với đúng một
  ngoại lệ có tên ở §I/O Matrix (hạ `done` khi gộp).
- **Không xoá Chương ngoài đường gộp.** Không có lệnh *"xoá Chương"* ở story này; xoá một
  Chương là xoá văn bản người dùng, một quyết định chưa AC nào đòi.
- **Không nút "tách Chương" trong `LibraryMode.vue`.** Điểm tách là một **câu**, và câu chỉ
  chọn được ở Editor — một ô nhập số thứ tự câu ở Library là mời người dùng đếm mù.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Đổi tên | `rename_chapter(id, "Hồi 1")` | `chapter.title = 'Hồi 1'` + `updated_at` mới; **0 hàng `segment` bị chạm**; `meta.json` + chỉ mục dựng lại | No error expected |
| Đổi tên về không tên | `title` rỗng sau `str::trim()` (kể cả U+3000) | `chapter.title = NULL` — *chưa đặt tên*, một trạng thái hợp lệ; danh sách hiện lại nhãn `chapter_untitled` | No error expected |
| Dời lên/xuống | Chương ở giữa | Chuẩn hoá `1..N` rồi **hoán vị `ord`** với hàng liền kề; **chỉ cột `ord` đổi** trên đúng hai hàng | No error expected |
| Dời quá biên | dời lên ở Chương **đầu** | **0 hàng bị chạm** | `err.chapter.at_first` `{chapter_id}` |
| Dời quá biên | dời xuống ở Chương **cuối** | **0 hàng bị chạm** | `err.chapter.at_last` `{chapter_id}` |
| Gộp | Chương B gộp vào A liền trước | mọi hàng `segment` của B (**sống VÀ về hưu**) đổi `chapter_id → A` và `ord += MAX(ord) của A`; `A.source_text = A.source_text ‖ "\n\n" ‖ B.source_text`; hàng B xoá; `chapter_position` của B xoá; Chương sau B tịnh tiến `ord −1` | No error expected |
| Gộp ở Chương đầu | không có Chương liền trước | **0 hàng bị chạm** | `err.chapter.at_first` `{chapter_id}` |
| Gộp `done` + chưa xong | `A.status='done'`, `B.status≠'done'` | Chương gộp mang `in_progress` — **không bao giờ khai `done` cho văn bản chưa ai xác nhận**; `done` chỉ giữ khi **cả hai** là `done` | No error expected |
| Gộp Chương đang mở | `OpenWork::chapter_id == B` | sau commit, con trỏ dời sang **A**; webview nạp lại Editor + Source | No error expected |
| Tách | caret ở câu `s` (không phải câu đầu) | Chương mới B chèn ngay sau A (`ord` các Chương sau tịnh tiến `+1`); mọi hàng `segment` **tại và sau `(ord, id)` của `s`** (sống VÀ về hưu) đổi `chapter_id → B`, `ord −= (ord của s) − 1`; `status`/`title` của B chép từ A (`title` thành `NULL`); `source_text` của A và B **dựng lại bằng phép nối `source_text` của segment còn sống, phân tách bằng `\n`** | No error expected |
| Tách tại câu đầu | `s` là câu đầu Chương | **0 hàng bị chạm** — một Chương rỗng không phải kết quả có nghĩa | `err.chapter.split_leaves_empty` `{chapter_id}` |
| Tách khi caret trống | `editorCaretSegmentId === null` | lệnh **KÊU** rồi trả `false`, không lượt IPC nào | chẩn đoán nêu đích danh (luật hợp âm bàn phím, `src/AGENTS.md`) |
| Tách trên `segment_id` lạ | `segment_id` không thuộc Chương đang mở | **0 hàng bị chạm** | `segment.segment_not_found` `{segment_id}` (tái dùng) |
| Vị trí làm việc theo câu qua lượt tách | `chapter_position(A) → s` và `s` dời sang B | hàng vị trí **dời theo** sang B — cùng câu, đúng Chương | No error expected |
| Ghi vị trí lệch cặp | `save_chapter_position(c, s)` với `s` **không** thuộc `c` | **0 hàng ghi** — đóng món nợ 5.7 | `segment.segment_not_found` `{segment_id}` |
| Chưa mở Tác phẩm | bất kỳ lệnh nào trong bốn | không lượt SQL nào chạy | `work.none_open` (tái dùng) |

</intent-contract>

## Code Map

**Rust — bề mặt IPC và luật**

- `src-tauri/src/commands/chapter.rs` (493 dòng) — **tệp nhận cả bốn hàm thuần mới**.
  `no_work_open` (~72, `pub(crate)`) · `chapter_not_found` (~101, `pub(crate)`) — **tái dùng,
  không đúc khoá mới**. `open_adjacent_chapter` (~228+) là **khuôn so sánh bộ đôi `(ord, id)`**
  và chở luật *"con trỏ đổi SAU khi truy vấn thành công"*. `list_chapters` (~340) và
  `open_chapter` (~380) là khuôn hàm-thuần-rồi-vỏ mới nhất. `mod wire` (~410+) —
  `try_state`, `MutexGuard` giữ xuyên suốt cho mọi hàm nhận `&mut`.
- `src-tauri/src/commands/lifecycle.rs` — 🔴 **KHUÔN BỐN BƯỚC, chép nguyên**: doc-comment đầu
  tệp (~7–19) phát biểu nó; `write_lifecycle_after_change` (~79, **hôm nay private — nâng lên
  `pub(crate)`**, giữ nguyên TÊN kèm một khối 🔵 + ngày, vì đổi tên làm mồ côi mọi tham chiếu
  trong ba tệp story); `finish_lifecycle_write` (~240, `pub`, generic trên `T` — **dùng
  thẳng**); `set_chapter_status` (~130–160) là khuôn *"cưỡng chế ở tầng Rust TRƯỚC khi chạm
  SQL"* + *"`UPDATE … WHERE id` khớp 0 hàng ⇒ lỗi có tên"*; `set_chapter_status_indexed`
  (~262) là khuôn hàm-thuần-cộng-bước-4 mà `tests/**` gọi được; `wire::finish_with_reindex`
  (~380) cho thấy cách lấy `Indexer`/`Store`/`root` ra khỏi `AppHandle`. ⚠️ `#[tauri::command(async)]`
  là **bắt buộc** cho mọi vỏ có bước 4 — cổng `config_invariants.rs::the_blocking_wires_run_off_the_main_thread`.
- `src-tauri/src/commands/segment.rs` (2996 dòng) — `save_chapter_position` (~950–1000):
  **thêm phép kiểm cặp `(chapter_id, segment_id)`** vào chính giao dịch đã có (câu
  `SELECT COUNT(*) FROM chapter WHERE id = ?1` ở ~968 đổi thành phép kiểm `segment` thuộc
  Chương — khuôn có sẵn ở `:2088`). `segment_not_found` (~1445, **hôm nay private — nâng
  `pub(crate)`**, cùng tiền lệ `no_work_open`/`chapter_not_found`). `write_regroup` (~2325) là
  **đối chứng ngược**: đọc nó để thấy gộp/tách *segment* về hưu + tạo mới, còn story này thì
  **không** — hai thao tác không được dùng chung một hàm.
- `src-tauri/src/core/store/schema.rs` — `CHAPTER_DDL` (~799) và doc-comment `~774–799`
  (`AUTOINCREMENT`, `ord` cố ý không `UNIQUE`); `SEGMENT_DDL` (~897) + `idx_segment_chapter_ord`;
  `CHAPTER_POSITION_DDL` (~768) — 🔴 khối ⚠️ ở `~763–767` ghi đích danh *"Một Chương bị xoá
  (Story 5.8) để lại một hàng vị trí mồ côi … chủ dọn là Story 5.8"*: **sửa tại chỗ kèm 🔵 +
  ngày** khi story này dọn nó. `PROJECT_MIGRATIONS` (~1403–1499) — **không thêm bước**.
- `src-tauri/src/core/library/meta.rs` — `rebuild_from_store` (~170+): `chapter_count`,
  `chapter_done_count`, và `updated_at` dẫn xuất từ `MAX(chapter.updated_at)`/`MAX(segment.updated_at)`.
  Đây là lý do **cả bốn** thao tác phải chạy bước 2+3, kể cả đổi tên.
- `src-tauri/src/core/i18n/mod.rs` — `message_keys!` (~100+); cụm Story 5.4 (~424–432) và
  Story 5.7 (~434+) là **chỗ nối thêm ba khoá mới** kèm một khối chú thích khai *"danh mục
  ĐÓNG, ca X tái dùng khoá Y"* đúng khuôn hai cụm đó.
- `src-tauri/src/lib.rs:324+` — `generate_handler![…]`: **năm vỏ mới phải vào danh sách này**.

**Frontend**

- `src/config/chapter.ts` (266 dòng) — adapter ba trạng thái đầy đủ: `readOpenChapter` (~110),
  `listChapters` (~?), `openChapter`, `isIpcError` (~33), `hasIpcBridge` (~93). **Bốn adapter
  mới vào chính tệp này, cùng khuôn.** ⚠️ `invoke()` gửi tham số **camelCase**
  (`chapterId`, `segmentId`, `title`) trong khi trường TRẢ VỀ giữ `snake_case`.
- `src/modes/libraryChapters.ts` (319 dòng) — `chapters`/`chaptersHaveLoaded`/`chaptersBusy`
  (~57–60), `chapterCursor` (~63), `loadChapters` (~105, cơ chế `chaptersReloadPending` +
  `chaptersSequence`), `currentLibraryChapter` (~93), `openWorkById` (~155: **khuôn flush →
  IPC → vứt state → nạp lại → nhả cờ bận SAU CÙNG**, đọc khối 🔴 *"CỬA SỔ MỞ NHẦM CHƯƠNG"*),
  `resetLibraryChapters` (~285). Nơi nhận `chapterRenameDraft` + bốn thao tác mới.
- `src/modes/libraryImport.ts:67` — `export const pastedText = ref('')` là **khuôn ref ghi
  được cho `v-model`** mà `LibraryMode.vue` đọc/ghi và `CommandDeps` nộp.
- `src/modes/LibraryMode.vue` (1456 dòng) — khối `.chapters-block` (~767–880): `role="status"`
  ba câu phân biệt được (~775–782), `.grid-nav` (~801–835), `.chapters-list` cuộn cửa sổ
  (~845–878), hàng `.chapter-row` (~854–872). `<style scoped>` `.chapters-list`/`.chapter-row`
  (~1395–1435). Form nhập (~913–960) là khuôn `<input v-model>` + nút `:disabled`.
- `src/panels/editorPanelState.ts` — `editorCaretSegmentId` (~101), `editorChapterId` (~90),
  `setEditorCaret` (~190, **định nghĩa duy nhất của "rời segment"**),
  `flushEditorBeforeDiscreteWrite`, `flushChapterPositionNow`, `openChapterById`,
  `resetEditorPanel` (~518), `ensureSegmentsLoaded` (~126). `switchChapter` (~1456–1580) là
  khuôn đầy đủ cho một lượt đổi Chương sau khi tách.
- `src/commands/index.ts` (2560 dòng) — `CommandDeps` cụm Chương (~288–302); đăng ký
  `library.open_work`/`list_chapters`/`chapter_next`/`chapter_prev`/`open_chapter`
  (~1086–1137) là **khuôn chép**; bảng ba lệnh Editor có hợp âm (~1879–1892:
  `editor.merge_segments` `Mod+M`, `editor.split_segment` `Mod+Slash`) là **khuôn cho
  `editor.split_chapter`**. Khối doc-comment ~1830–1868 chở luật *"command id nằm trong bảng
  keybinding người dùng, đổi tên là mồ côi phím IM LẶNG"*.
- `src/commands/keys.ts` — thứ tự phần tử hợp âm **CỐ ĐỊNH** `Mod → Meta → Ctrl → Alt → Shift
  → phím` (~241, `:269`); `Shift` hợp lệ (~196). Đo 2026-08-29: `Mod+Shift+*` xuất hiện
  **0 lần** trong `src/commands/index.ts` ⇒ `Mod+Shift+Slash` còn trống.
- `src/main.ts:340–360` — `installCommands({…})`, chỗ tiêm dep mới.
- `src/i18n/vi.json` (479 dòng) — cụm `mode.library.*`, `command.library.*`, `command.editor.*`,
  `err.*`. **Phẳng, khoá chấm, không giá trị rỗng**, placeholder khớp `[a-z_][a-z0-9_]*`.

**Cổng và bàn đo**

- `src-tauri/tests/project_contract.rs` (2075 dòng) — `insert_chapter_directly` (~848) là hàm
  dựng Chương thứ hai cho test; `list_chapters_*` (~1978–2028) và `opening_a_named_chapter_*`
  (~2030–2075) là khuôn ca gần nhất. Đây là tệp nhận bộ ca của story này.
- `src-tauri/tests/segment_contract.rs` (6803 dòng) —
  `a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else` **phải giữ xanh**;
  bộ ca `save_chapter_position` sống ở đây và nhận ca cặp lệch mới.
- `src-tauri/tests/ipc_contract.rs` (650 dòng) — `:472–535` đóng băng khoá `snake_case` của
  `WorkRow`/`WorkListReport`; **mọi struct mới trên dây phải có ca ở đây**.
- `src-tauri/tests/lifecycle_contract.rs` (598 dòng) — khuôn ca *"ghi thành công ⇒ hàng
  `library_work` mang giá trị mới"* qua `*_indexed`.
- `src-tauri/tests/meta_write_boundary.rs` — ranh giới ba tệp ghi / ba tệp đọc `meta.json`.
  Story này **không** thêm chỗ đọc/ghi mới (nó gọi lại `write_lifecycle_after_change` của
  `commands/lifecycle.rs`, một tệp đã có trong danh sách) ⇒ **cổng phải giữ xanh không đổi
  con số**; nếu nó đỏ thì lượt cài đã đi sai đường.
- `tests/frontend/libraryChapters.test.ts` · `chapterPosition.test.ts` · `editorChapterSwitch.test.ts`
  — `invoke` giả + khuôn mount `LibraryMode.vue`.
- `e2e/specs/story-5-7-open-chapter.e2e.mjs` — `createWork()` qua IPC trần, `realClick`,
  `e2e/support/panelReset.mjs`. ⚠️ **`focusViaJs` + `browser.keys` KHÔNG kích hoạt được một
  `<button>`** (đo 2026-08-29, `deferred-work.md` §"Deferred from: 5-7…") ⇒ bàn đo e2e chỉ phủ
  đường CHUỘT; vế bàn phím **không** được khai là đạt.
- `scripts/check-commands.mjs` — `COMMAND_FLOOR` 52 (:319) · `CLICK_FLOOR` 27 (:352) ·
  `DISPATCH_FLOOR` 40 (:363), đều là **cận DƯỚI**. Kiểm A: mỗi `@click` là đúng một
  `dispatch('<id>')` **id literal**.
- `scripts/check-panel-refs.mjs` — mọi ô nhớ cấp module phải có một đường `reset*()`; mỗi
  khai báo state nằm **trên một dòng**.

## Tasks & Acceptance

**Execution:**

1. `src-tauri/src/core/i18n/mod.rs` -- thêm cụm *"Story 5.8 (FR15)"* với **đúng ba** khoá
   mới: `ChapterAtFirst => "err.chapter.at_first" ["chapter_id"]`,
   `ChapterAtLast => "err.chapter.at_last" ["chapter_id"]`,
   `ChapterSplitLeavesEmpty => "err.chapter.split_leaves_empty" ["chapter_id"]`, kèm một khối
   chú thích khai **danh mục ĐÓNG** đúng khuôn cụm 5.4/5.7: ca *"chưa mở Tác phẩm"* tái dùng
   `WorkNoneOpen`, ca *"chapter_id lạ"* tái dùng `SegmentChapterNotFound`, ca *"segment lạ"*
   tái dùng `SegmentNotFound` -- không đúc khoá thứ tư cho một câu đã có chủ. -- Rationale:
   `message_key` là danh mục đóng khai bằng macro; một khoá trùng nghĩa là hai chuỗi phải giữ
   khớp bằng kỷ luật.
2. `src/i18n/vi.json` -- ba khoá `err.chapter.*`, năm khoá `command.library.chapter_rename` ·
   `command.library.chapter_move_up` · `command.library.chapter_move_down` ·
   `command.library.chapter_merge_up` · `command.editor.split_chapter`, và cụm
   `mode.library.chapter_rename_label` · `chapter_rename_button` · `chapter_move_up` ·
   `chapter_move_down` · `chapter_merge_up` · `chapter_reorg_flush_failed` ·
   `chapter_reorg_still_dirty`. Phẳng, không giá trị rỗng, placeholder `{chapter_id}` /
   `{segment_id}` đúng dải. -- Rationale: `check:i18n` chạy trên `ALL` của macro; một khoá
   thiếu ở `vi.json` là cổng đỏ, một khoá thừa cũng vậy.
3. `src-tauri/src/commands/lifecycle.rs` -- nâng `write_lifecycle_after_change` lên
   `pub(crate)`, **giữ nguyên tên**, thêm một khối 🔵 + ngày nói vì sao (Story 5.8 tái dùng
   đúng khuôn bốn bước; đổi tên làm mồ côi mọi tham chiếu trong ba tệp story đã ký). --
   Rationale: hai bản chép của khuôn bốn bước là hai thứ phải giữ khớp bằng kỷ luật; đo
   2026-08-27 đã cho thấy chỗ nối bước 4 **chưa có ai canh** khi nó bị chép.
4. `src-tauri/src/commands/segment.rs` -- nâng `segment_not_found` lên `pub(crate)` kèm lý do
   tại chỗ; và **thêm phép kiểm cặp** vào giao dịch của `save_chapter_position`: đổi câu
   `SELECT COUNT(*) FROM chapter WHERE id = ?1` thành phép kiểm `segment` thuộc đúng Chương
   (khuôn có sẵn ở `:2088`), trả `segment_not_found(segment_id)` qua đúng ô `Arc<Mutex<bool>>`
   đã có, và giữ nguyên `chapter_not_found` cho ca Chương lạ. -- Rationale: đóng mục
   `deferred` #1 của Story 5.7 -- một cặp lệch đọc lên **giống hệt** ca "segment đã về hưu",
   hai nguyên nhân khác hẳn đội chung một biểu hiện; story này là story đầu tiên sinh ra được
   cặp lệch đó (segment ĐỔI `chapter_id`).
5. `src-tauri/src/commands/chapter.rs` -- hàm dùng chung `normalize_chapter_ord(tx) -> SqlResult<()>`:
   đọc `SELECT id FROM chapter ORDER BY ord, id`, `UPDATE chapter SET ord = ?1 WHERE id = ?2
   AND ord <> ?1`. Chạy **đầu** mọi giao dịch tổ chức. -- Rationale: `ord` không `UNIQUE` và
   không hứa liên tục; chuẩn hoá một lần làm mọi phép còn lại là số học an toàn, và mệnh đề
   `WHERE ord <> ?1` giữ `updated_at` của hàng không đổi khỏi bị nhảy oan.
6. `src-tauri/src/commands/chapter.rs` -- `rename_chapter(open: Option<&mut OpenWork>, chapter_id, title: &str)`:
   `str::trim()` ở **Rust**, rỗng ⇒ `NULL`; `UPDATE chapter SET title = ?1, updated_at = …
   WHERE id = ?2`; `touched == 0` ⇒ `chapter_not_found`; rồi `write_lifecycle_after_change`.
   Trả `Vec<ChapterRow>` (danh sách đã dựng lại) để webview không phải đoán. -- Rationale:
   `trim()` của SQLite chỉ cắt dấu cách ASCII -- đo 2026-08-19, rào rỗng của Story 3.1 đã
   thủng bảy đường vì đúng chỗ này.
7. `src-tauri/src/commands/chapter.rs` -- `move_chapter(open, chapter_id, direction: ChapterDirection)`:
   chuẩn hoá; tìm hàng kề theo `(ord, id)` **bằng đúng câu SQL của `open_adjacent_chapter`**;
   không có hàng kề ⇒ `ChapterAtFirst`/`ChapterAtLast` và **0 hàng bị chạm**; có ⇒ hoán vị
   `ord` hai hàng trong cùng giao dịch. Tái dùng `enum ChapterDirection` đã có. -- Rationale:
   một `ord ± 1` trần biên dịch sạch, đi qua mọi cổng, và báo *"đã ở Chương cuối"* trên một
   Tác phẩm còn nguyên Chương phía sau -- lớp lỗi đã có tên ở `open_adjacent_chapter`.
8. `src-tauri/src/commands/chapter.rs` -- `merge_chapter_into_previous(open, chapter_id)`: chuẩn
   hoá; tìm A liền trước (không có ⇒ `ChapterAtFirst`, 0 hàng bị chạm); trong **một** giao
   dịch: `shift = SELECT MAX(ord) FROM segment WHERE chapter_id = A` (`NULL ⇒ 0`);
   `UPDATE segment SET chapter_id = A, ord = ord + shift WHERE chapter_id = B` -- **không mệnh
   đề `retired_at`**, mọi hàng đi cùng; `UPDATE chapter SET source_text = source_text || char(10)
   || char(10) || (SELECT source_text FROM chapter WHERE id = B), status = <luật hạ done>,
   updated_at = … WHERE id = A`; `DELETE FROM chapter_position WHERE chapter_id = B`;
   `DELETE FROM chapter WHERE id = B`; `UPDATE chapter SET ord = ord - 1 WHERE ord > <ord của B>`.
   Sau commit: nếu `open.chapter_id == B` thì đặt `= A`. Rồi `write_lifecycle_after_change`. --
   Rationale: nối `source_text` **thô** là lượt duy nhất không mất byte nào, và nó đúng cả với
   Chương chưa từng tách segment (25 Chương của Epic 1) -- một phép nối theo segment sẽ **xoá
   sạch** nguyên văn của chúng.
9. `src-tauri/src/commands/chapter.rs` -- `split_chapter_at_segment(open, segment_id)`: đọc
   `(chapter_id, ord)` của `segment_id` (không có / khác Chương đang mở ⇒ `segment_not_found`);
   `ord == 1` (hoặc không hàng sống nào đứng trước) ⇒ `ChapterSplitLeavesEmpty`, 0 hàng bị
   chạm. Trong **một** giao dịch: chuẩn hoá; `INSERT INTO chapter (ord, title, source_text,
   status, …) VALUES (<ordA + 1>, NULL, '', <status của A>, …)` lấy `B = last_insert_rowid()`;
   `UPDATE chapter SET ord = ord + 1 WHERE ord > <ordA> AND id <> B`;
   `UPDATE segment SET chapter_id = B, ord = ord - (<ord của s> - 1) WHERE chapter_id = A AND
   (ord > ?s OR (ord = ?s AND id >= ?sid))` -- **không mệnh đề `retired_at`**;
   `UPDATE chapter_position SET chapter_id = B WHERE chapter_id = A AND segment_id IN (…đã dời)`;
   rồi dựng lại `source_text` của **cả A và B** bằng phép nối `segment.source_text` của hàng
   **còn sống** theo `ord`, phân tách bằng `\n`. Sau commit: `open.chapter_id` giữ nguyên A.
   Rồi `write_lifecycle_after_change`. -- Rationale: xem §Design Notes *"Vì sao `source_text`
   của lượt TÁCH dựng lại từ segment"*.
10. `src-tauri/src/commands/chapter.rs` -- bốn hàm `*_indexed` (khuôn `set_chapter_status_indexed`)
    nhận `Option<&Indexer>`/`Option<&Store>`/`&Path` và gọi `lifecycle::finish_lifecycle_write`.
    -- Rationale: đo 2026-08-27 -- gỡ hẳn hai lời gọi bước 4 cho **0 failed** trên toàn bộ 34
    binary khi bước 4 sống trong `mod wire`; `tests/**` không dựng được `AppHandle`.
11. `src-tauri/src/commands/chapter.rs` -- năm vỏ `#[tauri::command(async)]` trong `mod wire`
    (`rename_chapter` · `move_chapter` · `merge_chapter_into_previous` · `split_chapter_at_segment`,
    cộng `finish_with_reindex` riêng của tệp hoặc tái dùng đường của `lifecycle::wire`).
    `try_state`, khoá chạy phần ghi rồi **NHẢ trước bước 4**. -- Rationale: bước 4 quét toàn
    bộ thư mục gốc Library; `config_invariants.rs::the_blocking_wires_run_off_the_main_thread`
    canh `(async)`, và giữ khoá qua một lượt quét đĩa chặn mọi lệnh khác.
12. `src-tauri/src/lib.rs` -- thêm bốn vỏ vào `generate_handler![…]` (~:324) kèm chú thích
    ngắn không dấu. -- Rationale: một vỏ không có trong danh sách là một lệnh `invoke` trả lỗi
    "command not found" mà không cổng nào đỏ.
13. `src-tauri/src/core/store/schema.rs` -- sửa **tại chỗ** khối ⚠️ của `CHAPTER_POSITION_DDL`
    (~763–767) kèm 🔵 + ngày: mệnh đề *"Một Chương bị xoá (Story 5.8) để lại một hàng vị trí
    mồ côi"* hết đúng -- đường gộp xoá hàng đó trong cùng giao dịch, và đường tách **dời** nó
    theo câu. -- Rationale: luật của kho là sửa tại chỗ kèm ngày, không để một mệnh đề lặng lẽ
    sai.
14. `src/config/chapter.ts` -- bốn adapter ba trạng thái: `renameChapter(chapterId, title)` ·
    `moveChapter(chapterId, direction)` · `mergeChapterIntoPrevious(chapterId)` ·
    `splitChapterAtSegment(segmentId)`. Tham số **camelCase**; kiểm kiểu **lúc chạy** cho hàng
    `ChapterRow` trả về (tái dùng vị từ đã có). -- Rationale: adapter không bao giờ ném; hình
    dạng ba trạng thái là hợp đồng của cả thư mục.
15. `src/modes/libraryChapters.ts` -- `chapterRenameDraft = ref('')` (ghi được cho `v-model`),
    `chapterReorgBusy`/`chapterReorgNotice`/`chapterReorgError` (mỗi khai báo **một dòng**), và
    bốn hàm `renameCurrentChapter` · `moveCurrentChapterUp` · `moveCurrentChapterDown` ·
    `mergeCurrentChapterUp`: mỗi hàm chạy `flushEditorBeforeDiscreteWrite()` đọc **cả ba** giá
    trị, CHẶN khi khác `'clean'`, gọi IPC, rồi `await loadChapters()`; nhả cờ bận **sau cùng**.
    Gộp/tách khi Chương đang mở bị ảnh hưởng ⇒ `resetEditorPanel()`/`resetSourcePanel()` rồi
    `ensureChapterLoaded()` + `ensureSegmentsLoaded()`. Mở rộng `resetLibraryChapters()` cho
    mọi ô nhớ mới. -- Rationale: khuôn `openWorkById` đã trả giá cho cửa sổ "mở nhầm Chương";
    `check:panel-refs` đòi mọi ô nhớ cấp module có một đường `reset*()`.
16. `src/panels/editorPanelState.ts` -- `splitChapterHere(): Promise<boolean>`: đọc
    `editorCaretSegmentId`; `null` ⇒ **ghi chẩn đoán nêu đích danh rồi trả `false`**, không
    lượt IPC nào; ngược lại flush → `splitChapterAtSegment` → `resetEditorPanel()` +
    `ensureChapterLoaded()` + `ensureSegmentsLoaded()`. -- Rationale: hàm chạy từ một hợp âm
    bàn phím KHÔNG BAO GIỜ ném -- nó kêu; và không được tự chuyển chế độ.
17. `src/commands/index.ts` -- năm dep mới trong `CommandDeps` cụm Chương; đăng ký
    `library.chapter_rename` · `library.chapter_move_up` · `library.chapter_move_down` ·
    `library.chapter_merge_up` (**`keys: undefined`**, khuôn `library.work_next`) và
    `editor.split_chapter` (**`keys: ['Mod+Shift+Slash']`**, thêm vào bảng ba lệnh ~1879 cùng
    một khối chú thích chở phép đo hợp âm còn trống). Mọi handler thiếu dep ⇒ `portMissing`.
    -- Rationale: AD-34 §1 -- mọi thao tác đi qua `CommandRegistry`; command id nằm trong bảng
    keybinding người dùng nên đặt tên là quyết định một lần.
18. `src/main.ts` -- tiêm năm dep mới vào `installCommands({…})` (~340–360). -- Rationale:
    đăng ký ở `main.ts`, không trong `App.vue` -- một lượt HMR gọi `installCommands()` lần hai
    và `register()` ném vì id trùng.
19. `src/modes/LibraryMode.vue` -- trong `.chapters-block`: một `<input v-model="chapterRenameDraft">`
    + nút `@click="dispatch('library.chapter_rename')"`, ba nút
    `library.chapter_move_up`/`chapter_move_down`/`chapter_merge_up` cạnh `.grid-nav`, và một
    `<p class="error" role="status">` cho `chapterReorgError`/`chapterReorgNotice` (khuôn khối
    `libraryOpenWorkError` ~786–800). Mọi nút `:disabled` theo `chapterReorgBusy ||
    libraryChaptersBusy || libraryOpenWorkBusy || currentLibraryChapter === null`. Màu/cỡ chữ
    **chỉ từ token**. -- Rationale: `check:commands` Kiểm A -- mỗi `@click` là đúng một
    `dispatch('<id>')` id literal; `check:tokens` Kiểm B/B2 cấm màu và cỡ chữ viết thẳng.
20. `src-tauri/tests/project_contract.rs` -- bộ ca hành vi, tên hàm là **một câu khẳng định**.
    Bắt buộc có: `merging_two_chapters_changes_only_chapter_id_and_ord_on_every_segment_column`
    và bản song sinh cho lượt tách -- **chụp TOÀN BỘ mọi cột của mọi hàng `segment` cộng mọi
    hàng `segment_version` trước và sau, rồi khẳng định đúng hai cột đổi**;
    `no_segment_is_ever_retired_by_a_chapter_reorganisation`;
    `a_merged_chapter_never_claims_done_when_one_half_was_not_done`;
    `moving_the_first_chapter_up_touches_no_row_and_is_a_named_error`; bản song sinh cho Chương
    cuối và cho gộp ở Chương đầu; `splitting_at_the_first_segment_is_refused_before_any_write`;
    `renaming_to_an_ideographic_space_stores_null_not_a_blank_title`;
    `chapter_ord_stays_dense_from_one_after_a_merge_on_a_sparse_ord_sequence` (dựng `ord`
    thưa/trùng bằng `insert_chapter_directly`);
    `a_reorganisation_moves_retired_segments_with_their_living_neighbours`;
    `the_open_chapter_cursor_follows_the_surviving_chapter_after_a_merge`. -- Rationale: AC3–AC8
    là mệnh đề **về những cột KHÔNG đổi**; chỉ một phép chụp toàn cột mới nghiệm thu được nó.
21. `src-tauri/tests/segment_contract.rs` -- `saving_a_position_whose_segment_belongs_to_another_chapter_writes_nothing`
    và `a_split_moves_the_remembered_position_row_to_the_new_chapter`; giữ
    `a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else` xanh. -- Rationale:
    đóng mục `deferred` #1 của Story 5.7 bằng một ca, không bằng một lời khai.
22. `src-tauri/tests/ipc_contract.rs` -- đóng băng khoá `snake_case` của mọi struct mới trên
    dây và tên bốn lệnh mới. -- Rationale: bốn tên trường là **dây**; `#[serde(rename_all)]`
    là thói quen viết Tauri phải cưỡng lại.
23. `tests/frontend/libraryChapters.test.ts` (mở rộng) -- ca *"flush trả `'still-dirty'` ⇒
    KHÔNG lượt IPC nào và câu báo hiện ra"*, ca *"đổi tên xong danh sách nạp lại"*, ca *"nút
    tắt suốt cửa sổ bận"*, và ca `splitChapterHere` với caret `null` ⇒ **0 lượt `invoke`**. --
    Rationale: `AGENTS.md::Known pitfalls` -- một bộ test xanh không chứng minh chỗ nối được
    canh; ba ca này là ba chỗ nối mới.
24. `e2e/specs/story-5-8-reorganise-chapters.e2e.mjs` -- dựng một Tác phẩm qua IPC trần, tách
    nó thành hai Chương, đổi tên, dời, gộp lại, và khẳng định danh sách Chương trên màn hình
    khớp từng bước. **Chỉ đường CHUỘT (`realClick`)**; ghi ra bằng chữ rằng vế bàn phím không
    được nghiệm thu ở đây. -- Rationale: `focusViaJs` + `browser.keys` cho `window.__logs`
    RỖNG (đo 2026-08-29) -- khai nó thành đạt là *"đánh dấu đạt bằng suy luận"*.
25. `_bmad-output/implementation-artifacts/deferred-work.md` -- mục *"Deferred from: 5-8…"*
    cho: (a) mất khoảng trắng/dòng trống của `chapter.source_text` ở đường TÁCH (§Design
    Notes), (b) `work.last_chapter_id` -- *"mở Tác phẩm ở Chương nào"* nay **quan sát được**
    vì Chương thứ hai tồn tại thật, mà `open_work` vẫn mở Chương đầu; và **đóng** mục
    `deferred` #1 của Story 5.7 bằng `→ ✅ ĐÃ ĐÓNG 2026-08-29 (Story 5.8)` kèm cách đóng. --
    Rationale: mọi thứ không nghiệm thu được ở story hiện tại đi vào sổ nợ **kèm một chủ**;
    không mục nào mồ côi.
26. `src/AGENTS.md` -- nối một dòng vào mục *"Ba cặp hằng nhịp ghi"* hoặc mục Chương nếu lượt
    cài đổi một mệnh đề đang có; nếu không đổi mệnh đề nào thì **không sửa**. -- Rationale:
    chỉ sửa khi một mệnh đề hết đúng; thêm chữ vào một tệp nạp mỗi phiên là chi phí thật.

**Acceptance Criteria:**

- Given một Chương tên `NULL`, when đổi tên thành `"Hồi 1"`, then `chapter.title = 'Hồi 1'`,
  `chapter.updated_at` mới, **0 hàng `segment` và 0 hàng `segment_version` đổi một byte**, và
  `meta.json` + hàng `library_work` mang `updated_at` mới.
- Given ba Chương `ord` thưa `(5, 5, 9)` dựng bằng `insert_chapter_directly`, when dời Chương
  giữa lên, then thứ tự đọc ra theo `(ord, id)` đổi đúng một bậc và `ord` của cả ba là
  `1, 2, 3` liên tục -- **không** một hàng `segment` nào bị chạm.
- Given Chương A (3 câu, một câu đã về hưu) và Chương B (2 câu) liền nhau, when gộp B vào A,
  then A có đủ 5 hàng theo đúng thứ tự cũ, hàng về hưu vẫn ở đúng chỗ, `segment.id` của cả
  năm **không đổi**, `chapter_count` giảm 1, và hàng `chapter` của B **không còn**.
- Given B đang mở (`OpenWork::chapter_id == B`), when gộp B vào A, then con trỏ trỏ A và một
  lượt `read_open_chapter` **không** trả `segment.chapter_not_found`.
- Given A `status = 'done'` và B `status = 'not_started'`, when gộp, then Chương gộp mang
  `in_progress`; given cả hai `done`, then Chương gộp mang `done`.
- Given một Chương 4 câu với lịch sử phiên bản trên câu 3, when tách tại câu 3, then Chương
  mới mang câu 3 và 4 với **cùng `segment.id`**, `ord` của chúng là `1, 2`, mọi hàng
  `segment_version` của câu 3 **còn nguyên và vẫn tra được**, và **không** câu nào mang
  `retired_at`.
- Given `chapter_position` của Chương A trỏ câu 3, when tách tại câu 3, then hàng vị trí đó
  thuộc Chương mới và `save_chapter_position` trên cặp cũ `(A, câu 3)` bị từ chối bằng
  `segment.segment_not_found` **không ghi một hàng nào**.
- Given caret trong Editor đang ở `null`, when phát `editor.split_chapter`, then **0 lượt
  `invoke`**, một dòng chẩn đoán nêu đích danh, và lệnh trả `false` -- không ném, không tự
  chuyển chế độ.
- Given tập chờ Editor còn bẩn, when phát bất kỳ lệnh nào trong bốn, then **0 lượt IPC tổ
  chức nào chạy** và màn hình hiện đúng một trong hai câu `chapter_reorg_flush_failed` /
  `chapter_reorg_still_dirty`.
- Given lượt sửa đã xong, when gỡ mệnh đề `AND (ord > ?s OR (ord = ?s AND id >= ?sid))` khỏi
  câu `UPDATE segment` của đường tách và chạy lại bộ ca Task 20, then bộ ca **phải ĐỎ**.

## Spec Change Log

## Review Triage Log

### 2026-08-29 — Review pass

- intent_gap: 0
- bad_spec: 0
- patch: 7: (high 1, medium 4, low 2)
- defer: 1: (high 0, medium 1, low 0)
- reject: 6: (high 0, medium 2, low 4)
- addressed_findings:
  - `[high]` `[patch]` `move_chapter`/`merge_chapter_into_previous` với một `chapter_id` không tồn tại trả `store.write_failed` — một câu SAI VỀ LOẠI (không tệp nào hỏng), đúng lớp lỗi Story 2.11 đã sửa một lần khi dựng `chapter_not_found`. **Đo trước khi vá** bằng cách chạy thật hai lời gọi: cả hai in ra `store.write_failed`. Thêm phép kiểm hàng tồn tại vào đầu cả hai giao dịch, trả `segment.chapter_not_found` với 0 hàng bị chạm; ca canh `moving_or_merging_an_unknown_chapter_id_reuses_the_named_error_not_a_store_error`.
  - `[medium]` `[patch]` `chapterRenameDraft` không đồng bộ theo con trỏ danh sách — dời con trỏ sang Chương khác rồi bấm "Đổi tên" áp chữ còn sót của Chương TRƯỚC lên Chương mới, im lặng. Thêm `watch` trên `chapter_id` (KHÔNG trên đối tượng hàng — `loadChapters()` thay cả mảng mỗi lượt và sẽ giẫm lên chữ đang gõ dở). Hai ca canh, đối chứng ĐỎ đã chạy.
  - `[medium]` `[patch]` `editor.split_chapter` báo mọi đường trượt chỉ bằng `console.error` — kể cả ca thường nhất (`err.chapter.split_leaves_empty`), trong khi hai lệnh ANH EM của nó hiện câu từ chối trên `StatusBar`. Thêm ô nhớ câu chữ **thứ TƯ** qua `datThongBao` (đúng đường mở rộng mà doc-comment của hàm đó đã hẹn), cộng bảng khoá ĐÓNG thứ tư ở `StatusBar.vue`. Đối chứng ĐỎ đã chạy trên thứ tự ghi-sau-reset.
  - `[medium]` `[patch]` `chapters.value.at(chapterCursor.value - 1)` với con trỏ ở 0 cho `.at(-1)`, tức phần tử CUỐI mảng, không `undefined`; và nó đọc TRƯỚC một `await` có thể kéo tới trần cứng 5 s của AD-35. Kẹp biên dưới và dời lượt đọc xuống sau `beginChapterReorg()`; ca canh con trỏ ở hàng đầu.
  - `[medium]` `[patch]` thân `splitChapterHere` và nhánh nạp lại Editor của `mergeCurrentChapterUp` không có một ca nào chạm tới (chỉ vị từ caret rỗng có lưới). Thêm ba ca cho `splitChapterHere` (caret rỗng · Rust từ chối · thành công) và một ca cho con trỏ hàng đầu.
  - `[low]` `[patch]` ba câu lỗi mới chở `chapter_id` thô — một `AUTOINCREMENT` cục bộ không khớp con số nào người dùng thấy (danh sách hiện `ord`). Bỏ tham số khỏi cả `message_keys!` lẫn `vi.json`, theo tiền lệ `MessageKey::LibraryRootInvalid`; tham số giữ ở chữ ký hàm cho chuỗi chẩn đoán.
  - `[low]` `[patch]` ca `chapter_ord_stays_dense_from_one_after_a_merge_on_a_sparse_ord_sequence` mang chữ *"after_a_merge"* nhưng chỉ gọi `move_chapter` — một cái tên khai nhiều hơn thứ nó đo. Đổi tên thành `..._after_a_move_...` và thêm ca thật cho đường gộp.


## Design Notes

### Vì sao `source_text` của lượt GỘP nối thô, còn lượt TÁCH dựng lại từ segment

Hai đường, hai luật -- và sự bất đối xứng đó **bị ép**, không phải một sở thích.

**Gộp** có sẵn cả hai chuỗi thô trên đĩa, nên `A ‖ "\n\n" ‖ B` không mất một byte nào. Và nó
là đường **duy nhất** đúng cho một Chương chưa từng tách segment: đo 2026-08-29, GridPanel có
nhánh `showNoSegments` cho *"25 Chương của Epic 1"* -- những Chương có `source_text` và **0**
hàng `segment`. Một phép nối theo segment sẽ ghi chuỗi rỗng đè lên nguyên văn của chúng.

**Tách** thì không có đường nào giữ nguyên byte: không cột nào lưu vị trí của một segment
trong `chapter.source_text`. Hai phương án, và một phương án **đo được là hỏng**:

- **(a) Cắt chuỗi thô tại offset tìm bằng cách dò `segment.source_text` trong đó.** Hỏng ngay
  khi Chương đã đi qua một lượt gộp/tách *segment* của Story 2.8: `write_regroup` **tạo hàng
  mới** với văn bản ghép, và văn bản ghép đó không còn là một chuỗi con của bản thô. ⇒ Phương
  án này bắt buộc phải kèm một nhánh dự phòng, tức **hai bản cài đặt của cùng một quy tắc** --
  đúng hình dạng mà `AGENTS.md::Known pitfalls` gọi tên.
- **(b) Nối `segment.source_text` của hàng còn sống bằng `\n`.** Luôn chạy được, một nhánh.

Chọn **(b)**, và cái giá của nó **đo được là bằng không cho mọi chỗ đọc đang tồn tại**. Đo
2026-08-29, `chapter.source_text` sau khi Chương đã có segment chỉ còn ba chỗ đọc sản phẩm:
`sourcePanelState.ts::hanCharOccurrenceCount` (**đếm** ký tự Hán), `ensureHanVietLoaded` (**tra**
âm theo ký tự), và `GridPanel.vue::isEmptyChapter` (`.trim() === ''`). Cả ba **không đọc
khoảng trắng**. Chỗ đọc thứ tư -- `split_chapter_into_segments` -- có rào `already_split` nên
nó không bao giờ chạy trên một Chương đã tách. Đường quét glossary lúc nhập thì đã đọc
**segment** chứ không đọc `chapter.source_text` (`project.rs::read_chapter_segment_texts`).

⚠️ **Giới hạn thật, ghi ra thay vì để người sau tưởng đã xét:** lượt tách **mất** dòng trống,
thụt đầu dòng và mọi khoảng trắng ngoài câu của bản thô, ở **cả hai** nửa. Hôm nay không chỗ
đọc nào thấy được điều đó; ngày một chỗ đọc thứ tư cần bản thô đúng từng byte (ví dụ một
đường xuất `.docx` giữ định dạng), món nợ này là thứ phải trả trước. Ghi vào `deferred-work.md`
kèm chủ.

### Vì sao tịnh tiến `ord` một hằng số, không đánh số lại 1..N

`write_regroup` (Story 2.8) đánh số lại từ đầu, và nó **đúng ở đó**: gộp/tách segment tạo và
huỷ hàng, nên không có phép tịnh tiến nào giữ được thứ tự. Story này thì **không tạo, không
huỷ** một hàng `segment` nào -- nó chỉ dời một khối liền mạch sang một `chapter_id` khác. Với
một khối liền mạch, `ord ± hằng số` giữ đúng ba thứ cùng lúc: thứ tự tương đối, tính liên tục
từ 1 (tiền đề của Story 2.10), và **điểm neo của hàng đã về hưu** -- thứ mà một lượt đánh số
lại chỉ-hàng-sống sẽ đẩy lệch, làm một chỗ đánh dấu FR119 trỏ tới nó đập xuống sai chỗ (AD-5).

### Vì sao gộp `done` + chưa xong ra `in_progress`

`chapter.status` là dữ liệu người dùng, và story này không có AC nào nói về nó. Nhưng phép
gộp **sinh ra** một hàng mới về mặt nội dung, và giữ nguyên `done` của A cho một Chương nay
chứa cả những câu chưa ai xác nhận là một **lời khai sai trông như bình thường**: nó chảy
tiếp qua `derive_work_status` thành một Tác phẩm khai *Đã xong*, và Story 5.5 đếm nó vào
`chapter_done_count`. Luật ở đây là dạng hẹp nhất nói được sự thật: `done` chỉ sống sót khi
**cả hai** nửa là `done`; mọi ca khác giữ nguyên `status` của A, hạ `done` xuống `in_progress`.
Trùng đúng triết lý FR58 -- *hệ thống không bao giờ tự coi một segment là đã xong*.

### Điểm tách sống ở Editor, không ở Library

Ba thao tác kia làm việc trên một **Chương**, và Library có sẵn con trỏ Chương (Story 5.7).
Lượt tách làm việc trên một **câu**, và `editorCaretSegmentId` là chỗ duy nhất trong kho biết
câu nào đang được chọn. Đường thứ hai -- một ô nhập số thứ tự câu ở Library -- mời người dùng
đếm mù trên một Chương 9.850 câu. `editor.split_segment` (`Mod+Slash`) là tiền lệ đầy đủ: một
hợp âm, không nút, vì nó chỉ có nghĩa ở đúng chỗ caret đang đứng. `Mod+Shift+Slash` đọc ra
thành *"cùng thao tác, một tầng lớn hơn"*, và nó mang `Mod` nên đi qua được luật vùng gõ
(`keys.ts:415`).

## Verification

**Commands:**
- `npm run check:i18n` -- expected: exit 0. Ba `MessageKey` mới đồng bộ `vi.json`; không chuỗi tiếng Việt có dấu ở vị trí mã trong `src-tauri/src/**`.
- `npm run check:commands` -- expected: exit 0. Mỗi `@click` mới là đúng một `dispatch('<id>')` id literal; năm lệnh mới đăng ký đủ; ba sàn quần thể không thủng.
- `npm run check:tokens` -- expected: exit 0. Màu và cỡ chữ của các nút mới đến từ token.
- `npm run check:panel-refs` -- expected: exit 0. Mọi ô nhớ mới của `libraryChapters.ts` nằm trong `resetLibraryChapters()`, mỗi khai báo trên một dòng.
- `npm run check:gates` -- expected: exit 0. Không cổng mới nào được thêm ⇒ ba danh sách không đổi.
- `npm run test` -- expected: mọi tệp xanh, kể cả `libraryChapters.test.ts` mở rộng.
- `npm run build` -- expected: `vue-tsc` 0 lỗi, `vite build` xong (chạy **TRƯỚC** `cargo test`).
- `cargo test --locked` (trong `src-tauri/`) -- expected: 0 đỏ; nêu đích danh `project_contract` (bộ ca mới), `segment_contract::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`, `meta_write_boundary` (**giữ nguyên con số, không nới**), `ipc_contract`, `config_invariants::the_blocking_wires_run_off_the_main_thread`.
- `npm run test:e2e` -- expected: `story-5-8-reorganise-chapters.e2e.mjs` xanh; `story-5-6-library-grid.e2e.mjs` **đỏ từ trước** (`deferred-work.md`, chủ Story 5.6) và không được đọc thành hồi quy.

**Manual checks (if no CLI):**
- Gỡ mệnh đề `AND (ord > ?s OR (ord = ?s AND id >= ?sid))` khỏi câu `UPDATE segment` của đường tách rồi chạy lại bộ ca Task 20: **phải ĐỎ**. Một bộ xanh không chứng minh chỗ nối mới được canh.
- Gỡ lời gọi `finish_lifecycle_write` khỏi bốn hàm `*_indexed` rồi chạy `cargo test --locked`: **phải ĐỎ** ở ca chỉ mục. Đo 2026-08-27 cho thấy đúng phép gỡ này từng cho **0 failed**.
- `grep -rn "split_source_text" src-tauri/src/commands/chapter.rs` sau lượt sửa: **0** kết quả -- bằng chứng của §Never *"không tính lại ranh giới segment"* (AD-4).
- `grep -rn "retired_at" src-tauri/src/commands/chapter.rs` sau lượt sửa: **0** kết quả trong bốn câu `UPDATE segment` -- bằng chứng hàng về hưu đi cùng khối, không bị bỏ lại.
- Đọc lượt CI (macOS **và** Windows) trước khi kết luận xanh -- `pre-push` chỉ chạy trên macOS của Ice.


## Auto Run Result

Status: done
Blocking condition: (không có)

### Đã dựng

Bốn thao tác **tổ chức** trên Chương (FR15 · AD-32), mỗi thao tác đi qua **khuôn bốn bước** đã
có của `commands/lifecycle.rs`: đổi tên · dời lên/xuống · gộp vào Chương liền trước · tách tại
câu đang có caret. **Không** bước di trú mới — bước cuối của `PROJECT_MIGRATIONS` vẫn là **17**.
Bốn vỏ IPC mới, năm id lệnh mới (bốn `library.chapter_*` không hợp âm, `editor.split_chapter`
mang `Mod+Shift+Slash`), ba `MessageKey` mới.

Story này cũng đóng một món nợ Story 5.7 giao đích danh cho nó (`save_chapter_position` không
kiểm cặp `(chapter_id, segment_id)`) và dọn hàng `chapter_position` mồ côi mà
`CHAPTER_POSITION_DDL` đã hẹn.

### Số đo nghiệm thu

- 11 cổng tĩnh + `check:lint` — **xanh**.
- `npm run build` (`vue-tsc` 0 lỗi) — **xanh**.
- `npm run test` — **47/47 tệp, 651/651 ca**.
- `cargo test --locked` — **36 binary, 929 ca, 0 đỏ**.
- `npm run test:e2e -- --spec e2e/specs/story-5-8-reorganise-chapters.e2e.mjs` — **1 passing
  (1m 33,8s)** trong WKWebView thật.

### Bốn đối chứng ĐỎ — một bộ xanh không chứng minh chỗ nối được canh

| Gỡ chỗ nối | Kết quả bắt buộc | Đo được |
|---|---|---|
| bỏ `AND (ord > ?2 OR (ord = ?2 AND id >= ?4))` khỏi `UPDATE segment` của lượt tách | ĐỎ | **2 ca đỏ** (`splitting_a_chapter_changes_only_…`, `splitting_rebuilds_source_text_…`) |
| trung hoà `finish_lifecycle_write` ở bốn hàm `*_indexed` | ĐỎ | **1 ca đỏ** (`merging_two_chapters_leaves_the_smaller_chapter_count_in_the_library_index`) |
| bỏ `await loadChapters()` khỏi `renameCurrentChapter` | ĐỎ | **1 ca vitest đỏ** |
| trung hoà phép tịnh tiến `ord` của lượt gộp (`ord + ?2` → `ord + 0`) | ĐỎ | **bàn đo e2e ĐỎ** |

Cả bốn đã khôi phục nguyên trạng sau phép đo.

### Hai phép grep của §Verification

- `split_source_text` trong `commands/chapter.rs`: **0** — AD-4 không bị đụng.
- Hai câu `UPDATE segment` của lượt gộp/tách: **0** mệnh đề `retired_at` — hàng về hưu đi cùng
  khối, đúng điểm khác AD-5 cố ý.

### Một chỗ sửa thêm, có đo

`commands/chapter.rs` bản đầu viết thẳng `"done"`/`"in_progress"` ở luật hạ trạng thái khi gộp.
⚠️ **Đo 2026-08-29:** đó là **hai chỗ DUY NHẤT** trong `src-tauri/src/**` ngoài
`core/lifecycle/mod.rs` còn viết thẳng bốn giá trị vòng đời ở vị trí mã, và **không cổng nào
canh** — §Verification của Story 5.4 đã cấm hình dạng này bằng chữ. Nay cả hai đi qua
`LifecycleStatus::Done/InProgress::as_str()`, tức một danh mục ĐÓNG sinh từ một khai báo; đo
lại sau lượt sửa: **0** kết quả.

### Giới hạn — ghi ra thay vì để người sau tưởng đã phủ

- 🔴 **Vế "làm được bằng BÀN PHÍM" chưa có đường nghiệm thu tự động nào**, và lượt TÁCH trong
  bàn đo e2e đi qua cầu IPC trần chứ không qua hợp âm `Mod+Shift+Slash`. Cùng khuyết tật bàn
  đo đã đo được ở Story 5.7 (`focusViaJs` + `browser.keys` cho `window.__logs` RỖNG). Ghi vào
  `deferred-work.md`, chủ Ice.
- 🔴 **Lượt TÁCH làm `chapter.source_text` mất khoảng trắng và dòng trống của bản thô** — cái
  giá đo được là **bằng không cho cả ba chỗ đọc đang tồn tại**, nhưng nó là một món nợ thật
  ngày có chỗ đọc thứ tư cần bản thô đúng byte. `deferred-work.md`, chủ Ice.
- ⚠️ **Chưa đọc lượt CI** — `pre-push` và mọi số đo trên đây chạy trên macOS. Nửa Windows chưa
  nói gì.
- ⚠️ `story-5-6-library-grid.e2e.mjs` **đỏ từ baseline** (chủ Story 5.6) — không đọc thành hồi
  quy của story này.

### Tệp đã đổi

**Rust — lõi và bề mặt IPC**
- `src-tauri/src/commands/chapter.rs` — bốn hàm thuần tổ chức + bốn `*_indexed` + bốn vỏ `(async)`, `normalize_chapter_ord`, ba hàm dựng `IpcError`.
- `src-tauri/src/commands/lifecycle.rs` — `write_lifecycle_after_change` nâng `pub(crate)`, giữ nguyên tên kèm lý do tại chỗ.
- `src-tauri/src/commands/segment.rs` — `save_chapter_position` kiểm cặp `(chapter_id, segment_id)`; `segment_not_found` nâng `pub(crate)`.
- `src-tauri/src/core/i18n/mod.rs` — ba `MessageKey` mới, không tham số.
- `src-tauri/src/core/store/schema.rs` — sửa tại chỗ khối ⚠️ của `CHAPTER_POSITION_DDL` (mệnh đề "hàng vị trí mồ côi" hết đúng).
- `src-tauri/src/lib.rs` — bốn vỏ vào `generate_handler!`.

**Frontend**
- `src/config/chapter.ts` — bốn adapter ba trạng thái.
- `src/modes/libraryChapters.ts` — `chapterRenameDraft` + `watch` đồng bộ, ba cờ tổ chức, bốn hàm thao tác.
- `src/modes/LibraryMode.vue` — khối `.chapter-reorg` (ô nhập + bốn nút) và câu báo, cộng `<style scoped>`.
- `src/panels/editorPanelState.ts` — `splitChapterHere` + ô nhớ câu chữ **thứ tư** qua `datThongBao`.
- `src/StatusBar.vue` — bảng khoá ĐÓNG thứ tư và nhánh render.
- `src/commands/index.ts` · `src/main.ts` — năm lệnh, năm dep.
- `src/i18n/vi.json` — 18 khoá mới.

**Test và bàn đo**
- `src-tauri/tests/project_contract.rs` — 20 ca hành vi, cộng sáu hàm dựng/chụp dùng chung.
- `src-tauri/tests/segment_contract.rs` · `ipc_contract.rs` · `config_invariants.rs` — bốn ca.
- `tests/frontend/libraryChapters.test.ts` — 12 ca.
- `e2e/specs/story-5-8-reorganise-chapters.e2e.mjs` — bàn đo mới.

**Tài liệu**
- `src-tauri/AGENTS.md` — sửa tại chỗ mệnh đề "`INSERT`/`DELETE FROM chapter` chỉ có đúng một chỗ".
- `deferred-work.md` — ba mục mới có chủ, một mục của Story 5.7 đóng bằng chữ.

### Kết quả review — 7 vá · 1 hoãn · 6 bác

Bốn lớp review chạy song song trên diff 3.942 dòng. **0 `intent_gap`, 0 `bad_spec`**: mọi mục hoặc là một lượt trượt cục bộ so với một luật spec ĐÃ viết ra, hoặc là một khuyết tật máy móc, hoặc là thiếu ca — không mục nào đòi dựng lại thiết kế.

Chi tiết bảy bản vá ở §Review Triage Log. Mục hoãn: gộp Chương xoá vĩnh viễn một hàng `chapter` bằng một lượt bấm, không xác nhận, không hoàn tác. Sáu mục bác: bốn mục đã có chủ trong sổ nợ, một ngoài phạm vi theo chính intent (gộp-XUỐNG), một quá mỏng.

**Follow-up review khuyến nghị: `true`** — đếm theo luật của workflow trên đúng bảy mục `patch` của lượt này: high **1**, medium **4**, low **2**; có một mục `high` ⇒ `true` (điểm `3 × 4 + 1 × 2 = 14`, cũng vượt ngưỡng 5).

### Nghiệm thu lại SAU lượt vá

- 11 cổng tĩnh + `check:lint` — **xanh**.
- `npm run build` — **0 lỗi**.
- `npm run test` — **47/47 tệp, 657/657 ca** (+6).
- `cargo test --locked` — **36 binary, 931 ca, 0 đỏ** (+2).
- e2e `story-5-8-reorganise-chapters` — **1 passing (1m 26,8s)**.

Ba đối chứng ĐỎ mới cho chính các bản vá: gỡ `watch` đồng bộ ô nhập ⇒ ca đổi tên **đỏ**; dời câu báo lên TRƯỚC `resetEditorPanel()` ⇒ ca tách **đỏ**. ⚠️ Lượt đầu của đối chứng thứ hai **không hợp lệ** và điều đó được ghi ra thay vì giấu: nó CHÈN THÊM một lượt ghi thay vì DỜI, nên dòng gốc vẫn chạy sau và ca vẫn xanh — một "đối chứng" xanh không chứng minh gì. Làm lại bằng phép dời thật thì ca đỏ đúng chỗ.

### Rủi ro còn lại

- ⚠️ **Chưa đọc lượt CI.** Mọi số đo trên đây chạy trên macOS của Ice; nửa Windows chưa nói gì.
- ⚠️ **Bộ e2e chỉ phủ đường CHUỘT cho ba thao tác, và phủ lượt tách bằng IPC trần.** Hợp âm `Mod+Shift+Slash` không có ca nào lái nó bằng phím thật — cùng khuyết tật bàn đo đã đo được ở Story 5.7, chủ Ice.
- ⚠️ **Lượt TÁCH làm `chapter.source_text` mất khoảng trắng và dòng trống của bản thô** — giá đo được là bằng không cho cả ba chỗ đọc hôm nay, nhưng nó là một món nợ thật, chủ Ice.
- ⚠️ `story-5-6-library-grid.e2e.mjs` đỏ từ baseline (chủ Story 5.6) — không đọc thành hồi quy của story này.
