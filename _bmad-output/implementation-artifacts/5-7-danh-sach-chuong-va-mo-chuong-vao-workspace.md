---
title: 'Story 5.7: Danh sách Chương và mở Chương vào Workspace'
type: 'feature'
created: '2026-08-29'
status: 'done'
baseline_revision: '6b2cb24829cc5be8857ea5f19bdff03d15f6c20e'
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
      `save_chapter_position` chỉ kiểm `chapter_id` tồn tại, KHÔNG kiểm `segment_id` thuộc
      đúng Chương đó — một cặp lệch được ghi nguyên vào `chapter_position`.
    evidence: |-
      Câu `UPSERT` chỉ đứng sau một `SELECT COUNT(*) FROM chapter WHERE id = ?1`. Một
      `segment_id` của Chương khác (hoặc chưa từng tồn tại) ghi xuống được, và đường ĐỌC chỉ
      bắt nó GIÁN TIẾP: `read_open_chapter_segments` thấy id đó không có trong danh sách vừa
      đọc rồi rơi về segment đầu kèm chẩn đoán — tức nó đọc lên GIỐNG HỆT ca "segment đã về
      hưu", hai nguyên nhân khác hẳn nhau đội chung một biểu hiện.
      Hôm nay chưa đường sản phẩm nào sinh được cặp lệch: chỗ gọi DUY NHẤT là
      `setEditorCaret`, và nó truyền `chapterId.value` cùng một `segment.id` đến từ chính
      Chương đang nạp. ⇒ Thật, nhưng chưa với tới được; đóng cùng lượt Story 5.8 mở đường
      gộp/tách Chương (lúc đó segment ĐỔI `chapter_id`).
    location: >-
      src-tauri/src/commands/segment.rs — save_chapter_position
    severity: medium
  - summary: >-
      Mở một Chương tự nó ARM một lượt ghi `chapter_position` thừa, ghi lại đúng giá trị vừa
      đọc ra.
    evidence: |-
      Chuỗi đo được: `ensureSegmentsLoaded` đặt `caretPlacement` → watcher của
      `GridPanel.vue:1110` gọi `target.focus()` + `setCaret(...)` → sự kiện `selectionchange`
      (listener đăng ký ở `GridPanel.vue:1090`) → `onSelectionChange` (`:998`) →
      `setEditorCaret(id)` (`:1004`) → `positionFlush.markMoved(...)`. ⇒ Chỉ MỞ một Chương,
      không chạm gì, vẫn tốn một lượt `save_chapter_position` sau ~500 ms.
      Vô hại về dữ liệu (ghi đúng giá trị vừa đọc) nên KHÔNG vá ở lượt này; đáng đóng vì nó
      là một lượt ghi đĩa cho mỗi lần mở Chương, và vì không ca nào canh mệnh đề "mở Chương
      thôi thì nhịp ghi vị trí phải SẠCH".
    location: >-
      src/panels/editorPanelState.ts — ensureSegmentsLoaded ↔ setEditorCaret
    severity: low
  - summary: >-
      Nhánh cuối của chuỗi ba ngôi hiển thị trạng thái Chương trả về `chapter.status` THÔ,
      không qua `t()`.
    evidence: |-
      `LibraryMode.vue` kết thúc chuỗi bằng `: chapter.status`. Nó chỉ an toàn nhờ một giả
      định LÚC CHẠY (`chapter.status` là `NOT NULL` và luôn nằm trong bốn giá trị), không
      nhờ kiểu; và chú thích ngay cạnh đã tự khai rằng Kiểm A2 của `check:i18n` **không đọc
      tĩnh được** một toán tử ba ngôi. ⇒ Nếu bất biến đó vỡ, một chuỗi thô rò lên giao diện
      mà không cổng nào đỏ. Khác hàng `work.status` ở ngay trên (đã có nhánh
      `works_status_invalid` qua `t()`) — hai bề mặt cạnh nhau, hai cách xử lý.
    location: >-
      src/modes/LibraryMode.vue — hàng Chương, nhãn trạng thái
    severity: low
  - summary: >-
      Nút "Mở Tác phẩm" vẫn bấm được trong lúc một lượt đổi Chương đang bay.
    evidence: |-
      `openChapterById` giữ cờ `dangChuyenChuong` ở phạm vi module của `editorPanelState.ts`
      và KHÔNG phơi ra, nên `LibraryMode.vue` không có tín hiệu để tắt nút. Lượt vá ở review
      này đã đóng chiều NGƯỢC LẠI (nút "Mở Chương" nay tắt theo `libraryOpenWorkBusy`), nên
      cửa sổ còn lại hẹp hơn hẳn; đóng nốt cần phơi một vị từ bận của đường đổi Chương —
      một quyết định về bề mặt state, không phải một lượt vá tiện tay.
    location: >-
      src/modes/LibraryMode.vue:691 ↔ src/panels/editorPanelState.ts (dangChuyenChuong)
    severity: low
  - summary: >-
      Vế "làm được bằng bàn phím" của AC7 chưa có đường nghiệm thu tự động nào.
    evidence: |-
      Đo 2026-08-29, cùng nút cùng phiên: `focus()` bằng JS + `browser.keys(['Enter'])` cho
      `window.__logs` RỖNG (handler chưa chạy), `realClick()` ngay sau đó thì chạy. Đây là
      khuyết tật của BỘ ĐO, không của sản phẩm. Cùng gốc với lượt đỏ của
      `story-5-6-library-grid.e2e.mjs`. Đã ghi đầy đủ kèm ba đường ra vào `deferred-work.md`
      §"Deferred from: 5-7…", chủ Ice.
    location: >-
      e2e/specs/story-5-7-open-chapter.e2e.mjs
    severity: medium
---

<intent-contract>

## Intent

**Problem:** FR12 đòi *mở đúng Chương đang dở và thấy con trỏ ở đúng câu bỏ dở*, nhưng ba
mảnh của nó đều **chưa tồn tại**, và mảnh dưới cùng là món nợ kiến trúc trung tâm của cả
Epic 5: ① **không đường nào mở lại một `.atproj` đã có trên đĩa** — `OpenWorkState` khởi tạo
`None` mỗi lượt chạy và chỗ DUY NHẤT đặt giá trị vào nó là `create_work_from_text`/
`_from_file` (`commands/project.rs:1784`/`:1819`), nên cách duy nhất mở một Tác phẩm là
**tạo mới** nó; ② **không lệnh nào liệt kê Chương** — `commands/chapter.rs` chỉ có
`read_open_chapter` (Chương ĐANG mở) và `open_adjacent_chapter` (kề theo hướng), không có
đường chọn một Chương đích danh; ③ **0 mẩu hạ tầng vị trí làm việc** — `grep "scroll"
src-tauri/src` = 0, không cột/bảng nào trong `project.db` giữ *"câu đang làm"*, nên mở lại
một Chương luôn bắt đầu từ hư không.

**Approach:** Ba lớp, đi qua đúng những khuôn đã có, không dựng cơ chế mới:
① **`open_work(work_id)`** — phân giải `atproj_path` từ `library-index.db`, `WorkMeta::read`
(kèm bề mặt hiển thị của cơ chế từ chối `meta.json` mới hơn mà Story 1.15 cố ý gỡ),
`Store::open`, dựng lại `ScopeResolver::with_work`, rồi `replace_open_work` — đúng thứ tự
`create_work` đã dựng, chỉ thay bước *tạo* bằng bước *đọc*;
② **`list_chapters()` + `open_chapter(chapterId)`** — hàng Chương KHÔNG mang `source_text`,
cộng một lưới cuộn **có cửa sổ** ở webview (AC2);
③ **bảng `chapter_position`** (bước di trú 17 của `project.db`) giữ `segment.id` — **không**
pixel (AD-3, Ice ký 2026-08-18) — và `read_open_chapter_segments` chở `caret_segment_id` về
để đường `editorCaretPlacement` **đã có** (`GridPanel.vue:1110`) đặt caret **và** cuộn.

## Boundaries & Constraints

**Always:**
- **Khuôn hai lớp cho mọi bề mặt IPC mới**: hàm thuần nhận `Option<&OpenWork>` / `&Path`, vỏ
  `#[tauri::command]` mỏng trong `mod wire` lấy `State` qua **`try_state`** (`src-tauri/AGENTS.md`).
- **Mọi lượt ghi vị trí đi qua `store::Writer` của Tác phẩm đang mở.** Không module nào tự
  mở kết nối ghi.
- **Đổi Tác phẩm hoặc đổi Chương phải FLUSH TRƯỚC lượt IPC dời con trỏ**, và đọc kết quả:
  `'failed'`/`'still-dirty'` ⇒ **CHẶN** kèm câu báo, không đi tiếp (khuôn
  `libraryImport.ts::beginSubmit` và `editorPanelState.ts::switchChapter`). Sau lượt IPC mới
  `resetEditorPanel()` + `resetSourcePanel()` rồi **nạp lại ngay** — vứt state mà không nạp lại
  để lại màn hình đứng ở Chương cũ vì ba chế độ sống trong `<KeepAlive>`.
- **Đổi Tác phẩm** (không phải đổi Chương) còn phải vứt `resetLookupPanel()` +
  `resetSegmentHistory()` — chúng thuộc TẦNG TÁC PHẨM, và `finishSubmit` là tiền lệ đầy đủ.
- **Danh sách rỗng phải nói VÌ SAO nó rỗng**: mỗi khối mới mang một vị từ `…HasLoaded` và
  màn hình hỏi nó TRƯỚC khi kết luận *"không có"*.
- **`chapter_position` vắng hàng nghĩa là *CHƯA TỪNG MỞ*** — phân biệt được với *"đã mở, đang
  ở câu đầu"*. Rust quyết `caret_segment_id` (AC5), webview không suy ra.
- Mọi nhãn qua `t()` (AD-21); mọi thao tác duyệt/mở Chương làm được bằng bàn phím (NFR17);
  màu và cỡ chữ chỉ đến từ token, WCAG AA ở cả hai theme.
- `@click` trong `.vue` là **đúng một** `dispatch('<id>')` với id **literal** (`check:commands`
  Kiểm A). Con trỏ danh sách Chương đi bằng hai lệnh thật, chép khuôn
  `library.work_next`/`work_prev`.

**Block If:**
- Vị từ *"đúng vị trí cuộn"* của AC4 **không** đạt được bằng `segment.id` + `focus()` và đòi
  một `scrollTop` pixel ⇒ HALT với blocking condition `AD moi: vi tri cuon theo pixel`. AD-3
  cấm tường minh, đường pixel đã trình cho Ice và **bị loại** (`deferred-work.md:5004`);
  chọn lại nó là một AD MỚI, giao Winston, KHÔNG tự soạn.
- Cuộn có cửa sổ (AC2) đòi **thêm một phụ thuộc npm** ⇒ HALT với blocking condition
  `phu thuoc moi can cua ra giay phep NFR15` — cửa NFR15 (mở tệp giấy phép trong nguồn ĐÃ
  TẢI, ghi vào bảng Stack của spine TRƯỚC khi thêm) là quyết định của Ice, không của lượt chạy này.
- Xuất hiện nhu cầu ghi `meta.json` ở một chỗ **thứ tư**, hoặc một đường ghi **tăng dần** cho
  `library_work`, để `open_work` chạy đúng ⇒ HALT với blocking condition
  `AD moi: duong ghi tang dan cho library_work` (kế thừa nguyên văn §Block If của Story 5.6).
- Bơm bất cứ gì vào giao dịch flush làm cổng đang xanh
  `segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
  hoặc `meta_write_boundary.rs` **đỏ** ⇒ HALT, KHÔNG sửa hai cổng đó (cả hai là AC đã ký của
  Story 2.3 và 5.5).

**Never:**
- **Không** cho webview truyền một **đường dẫn hệ tệp** vào lệnh mở kho. Tham số là `work_id`;
  `atproj_path` phân giải ở Rust từ `library-index.db`. `WorkRow.atproj_path` trên dây hôm nay
  là **dữ liệu hiển thị**, và đọc nó thành một tham số mở kho là mở một bề mặt IPC mới ngoài
  phạm vi `capabilities/main.json` đã rà.
- **Không** thêm cột `work.last_chapter_id` / *"Chương mở gần nhất của Tác phẩm"* ở lượt này —
  không AC nào đòi nó, và hôm nay **0** đường sản phẩm tạo Chương thứ hai (FR14 · Epic 6;
  `INSERT INTO chapter` chỉ ở `commands/project.rs:271`) nên khác biệt đó **không quan sát
  được**. Xem §Design Notes.
- **Không** sửa `CHAPTER_DDL` tại chỗ và **không** thêm cột vào `chapter`/`segment` — vết sẹo
  số 4 của `PROJECT_MIGRATIONS`. Bảng mới đi bằng **bước 17 MỚI**.
- **Không** đưa `chapter.source_text` vào hàng của danh sách Chương (Chương lớn nhất có thật
  **48.640 ký tự**; 2.000 hàng như thế là một lượt IPC vô nghĩa).
- **Không** gộp cặp hằng nhịp ghi vị trí với `EDITOR_IDLE_MS`/`IDLE_MS` — cùng *hình dạng*,
  không cùng *bảo đảm* (`src/AGENTS.md`).
- **Không** dựng tìm kiếm full-text (5.9), không Chế độ đọc (5.11–5.13), không tổ chức lại
  Chương (5.8), không đổi trạng thái một Tác phẩm **không phải** Tác phẩm đang mở.
- **Không** `v-html`, không đường mã nào tính lại ranh giới segment lúc nạp (AD-4/AD-39).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Mở một Tác phẩm chưa mở | `work_id` có trong chỉ mục, `.atproj` còn trên đĩa | `OpenWorkState` trỏ Tác phẩm đó; `chapter_id` = Chương đầu theo `(ord, id)`; `scope` là `with_work` | Không lỗi |
| `work_id` không có trong chỉ mục | UUID lạ | — | `IpcError` `library.work_not_indexed` `{work_id}`; `OpenWorkState` **không đổi** |
| Thư mục `.atproj` đã biến mất | chỉ mục còn hàng, đĩa thì không | — | `IpcError` `work.open_failed` `{name}`; `OpenWorkState` **không đổi** |
| `meta.json` phiên bản MỚI HƠN | `meta_schema_version` = 99 | — | `IpcError` `work.meta_too_new` `{found}`,`{supported}`; **không một byte nào bị ghi** |
| `project.db` phiên bản MỚI HƠN | `user_version` vượt đích | — | `StoreError::SchemaTooNew` ⇒ `store.schema_too_new`; kho **không** bị ghi (AD-30, đã có) |
| Mở lại **chính** Tác phẩm đang mở | `work_id` trùng | Mở lại bình thường (kho cũ đóng qua `Drop`), màn hình nạp lại | Không lỗi |
| Liệt kê Chương | Tác phẩm đang mở có `n` Chương | `n` hàng `(chapter_id, ord, title, status, segment_count)` theo `ORDER BY ord, id`; **không** `source_text` | Không lỗi |
| Liệt kê khi chưa mở Tác phẩm | `OpenWorkState = None` | — | `IpcError` `work.none_open` (khoá đã có, không đúc khoá thứ hai) |
| Chương không có tiêu đề | `chapter.title IS NULL` | Nhãn dựng từ `ord` qua `t('mode.library.chapter_untitled', { ord })` | Không lỗi |
| Mở một Chương đích danh | `chapterId` thuộc Tác phẩm đang mở | `OpenWork::chapter_id` đổi SAU khi truy vấn thành công; trả `OpenChapter` mới | Không lỗi |
| `chapterId` không tồn tại | id lạ | Con trỏ Chương **không đổi** | `IpcError` `segment.chapter_not_found` `{chapter_id}` (tái dùng khoá đã có) |
| Chương đã từng làm việc | `chapter_position` có hàng, `segment_id` còn sống | `caret_segment_id = segment_id`; lưới đặt caret vào đúng câu đó và cuộn tới | Không lỗi |
| Vị trí trỏ vào segment ĐÃ VỀ HƯU | `segment.retired_at IS NOT NULL` (gộp/tách, AD-5) | Rơi về **segment đầu** của Chương — không trả một id mà lưới không dựng ô cho | Không lỗi — rơi về CÓ LÝ DO, ghi chẩn đoán |
| Chương chưa từng mở | không hàng `chapter_position` | `caret_segment_id` = `segment.id` **đầu tiên** theo `(ord, id)` (AC5) | Không lỗi |
| Chương không có segment nào | `segment` rỗng | `caret_segment_id = null`; lưới rỗng nói *"chưa tách câu nào"* | Không lỗi |
| Ghi vị trí | caret dời từ câu A sang câu B | Một lượt `save_chapter_position` sau nhịp idle 500 ms / trần cứng 5 s | Lỗi ghi ⇒ chẩn đoán, KHÔNG hộp thoại |
| Ghi vị trí cho Chương không thuộc kho | `chapterId` lạ | Không hàng nào được ghi | `IpcError` `segment.chapter_not_found`; chẩn đoán, không chặn người dùng |
| Danh sách 2.000 Chương | `n = 2000` | Số phần tử `<li>` trong DOM **≤ 60**, và ô đang chọn luôn nằm trong cửa sổ | Không lỗi |
| Con trỏ Chương ra ngoài sau lượt tải | con trỏ ở hàng 7, lượt tải còn 3 hàng | Kẹp về hàng cuối còn lại; danh sách rỗng ⇒ `next`/`prev` là no-op | Không lỗi |
| Flush trượt lúc mở Tác phẩm/Chương khác | tập chờ còn chữ | **CHẶN**, câu báo trên thanh trạng thái, `OpenWorkState` **không đổi** | Không mất chữ im lặng |

</intent-contract>

## Code Map

**Tầng mở lại `.atproj` — món nợ kiến trúc trung tâm**

- `src-tauri/src/commands/project.rs` — `struct OpenWork` (§struct, ~dòng 43–96: bốn trường
  `dir`/`store`/`scope`/`meta` cộng `chapter_id`), `create_work` (~228–359: **khuôn thứ tự
  chuẩn** dựng thư mục → mở kho → ghi → `rebuild_from_store` → `write_atomic` →
  `ScopeResolver::with_work` ~348), `resolve_library_root` (~149–215), `replace_open_work`
  (~821–839: `swap_locked` + drop **ngoài** vùng khoá, AC10 Story 1.16 — **tái dùng, không
  dựng lại**), `OpenWorkState` (~799), `mod wire::reindex_library` (~1737), và hai vỏ
  `create_work_from_*` (~1763/~1796) làm **khuôn nguyên vẹn** cho vỏ `open_work`.
- `src-tauri/src/core/library/meta.rs` — `WorkMeta::read` (~170–190) là đường đọc `meta.json`
  **đã có test** (`project_contract.rs::a_newer_meta_schema_is_refused_without_touching_a_single_byte`)
  nhưng **0 chỗ gọi sản phẩm**; story này là chỗ gọi đầu tiên. `MetaError::SchemaTooNew` ~59.
  ⚠️ `meta_write_boundary.rs` khoá `WorkMeta::read` **chỉ** ở `core/library/meta.rs` +
  `core/library/indexer.rs` — thêm một chỗ đọc thứ ba **làm cổng đó ĐỎ**, xem §Design Notes.
- `src-tauri/src/core/library/mod.rs:50–59` — khối comment ghi **bằng chữ** rằng biến thể
  `MetaTooNew` + `MessageKey` + khoá `vi.json` bị gỡ ở Story 1.15 và *"Story nào dựng đường mở
  lại `.atproj` sở hữu việc thêm lại CÙNG MỘT LƯỢT"*. `enum WorkError` ~41–48,
  `From<MetaError> for WorkError` ~74–86, `From<WorkError> for IpcError` ~90–100.
- `src-tauri/src/core/library/indexer.rs` — `list_works` (~497–576) là **khuôn câu đọc**;
  hằng `COLUMNS` ~508. Cần một đường đọc **một hàng theo `work_id`** ở đây (không ở
  `commands/`: `library_index_boundary.rs` cấm module lệnh mang từ vựng chỉ mục).
- `src-tauri/src/lib.rs:324+` — `generate_handler![…]`; `:674` `app.manage(OpenWorkState)`;
  `:774` `close_open_work`. Mọi vỏ mới phải vào danh sách `:324`.

**Tầng Chương**

- `src-tauri/src/commands/chapter.rs` — `OpenChapter` (~34–46), `no_work_open` (~63),
  `chapter_not_found` (~86, `pub(crate)`, tái dùng), `read_open_chapter` (~107–141),
  `open_adjacent_chapter` (~230–306: **khuôn so sánh bộ đôi `(ord, id)`** và luật *"con trỏ
  đổi SAU khi truy vấn thành công"*), `mod wire` (~309–354). Đây là tệp nhận `list_chapters`
  và `open_chapter`. ⚠️ `warm_jieba_for_source_lang` phải gọi ở **mọi** đường mở Chương mới.
- `src-tauri/src/core/store/schema.rs` — `CHAPTER_DDL` (~765, `id`/`ord`/`title`/`source_text`/
  `status`/`created_at`/`updated_at`; `ord` cố ý **không** `UNIQUE`), `PROJECT_MIGRATIONS`
  (~1403–1499, bước cuối **16**; bước mới là **17**), `WORK_STATUS_OVERRIDE_DDL` (~738) là
  **khuôn doc-comment** cho một hằng DDL mới.
- `src-tauri/src/commands/segment.rs` — `ChapterSegment` (~211–221), `ChapterSegments`
  (~229–233: **thêm `caret_segment_id` ở đây**), `read_open_chapter_segments` (~841+, câu
  `SELECT … WHERE chapter_id = ?1 AND retired_at IS NULL ORDER BY ord, id`), `save_segment_targets`
  (~1186, **không sửa** — cổng `a_flush_touches_…` canh).
- `src-tauri/src/core/i18n/mod.rs:100+` — `message_keys! { … }`; cụm Story 5.3/5.6 ~392–421 là
  chỗ nối thêm. `WorkNoneOpen` ~159, `SegmentChapterNotFound` (tái dùng, không đúc khoá mới).

**Tầng frontend**

- `src/config/chapter.ts` — adapter ba trạng thái đầy đủ (`readOpenChapter` ~110,
  `openAdjacentChapter` ~146, `isIpcError` ~33, `hasIpcBridge` ~93). Thêm `listChapters`/
  `openChapter` **vào chính tệp này**, cùng khuôn.
- `src/config/library.ts` — `WorkRow` (~140–152), `isWorkRowArray` (~184), `isStringArray`
  (~212). Thêm adapter `openWork(workId)` và vị từ kiểm kiểu lúc chạy cho hàng Chương.
- `src/panels/editorPanelState.ts` — `switchChapter` (~1456–1580) là **KHUÔN ĐẦY ĐỦ để chép**
  cho `openChapterById`: ① `flushEditorBeforeDiscreteWrite()` (~1484) đọc **cả ba** giá trị →
  ② IPC → ③ `resetEditorPanel()` (~1529) + `resetSourcePanel()` (~1553) → ④
  `ensureSegmentsLoaded()` + `ensureChapterLoaded()` (~1556–1557), cộng cờ `dangChuyenChuong`
  (~1448) chống hai lượt cùng bay. `setEditorCaret` (~177–182) là **định nghĩa duy nhất của
  "rời segment"** — chỗ móc nhịp ghi vị trí. `ensureSegmentsLoaded` (~126–150),
  `editorCaretPlacement`/`clearEditorCaretPlacement` (~800–806), `editorHasLoaded` (~118),
  `resetEditorPanel` (~518), `wireExitFlush` (~687).
- `src/panels/GridPanel.vue:1110–1130` — watcher `editorCaretPlacement`: `target.focus()` +
  `setCaret`. Khối §AC8 (~1132–1165) ghi **ba phép đo** chứng minh `focus()` **là** vế cuộn và
  cuộn khéo hơn công thức tự cài. ⇒ AC4 nối vào đây, **không** viết một hàm cuộn thứ hai.
- `src/panels/editorFlush.ts:48,61` — `EDITOR_IDLE_MS`/`EDITOR_HARD_CAP_MS`, khuôn cho cặp hằng
  vị trí. `src/layout/writeSchedule.ts::createWriteSchedule` (~85) nhận `(idleMs, hardCapMs)`.
- `src/modes/libraryRescan.ts` — `orphanCursor` (:34), `currentLibraryOrphan` (:69), kẹp biên
  (:83–84), `next`/`prev` (:185/:190), reset (:203). `src/modes/libraryWorks.ts` — `workCursor`
  (:61), `clampWorkCursor` (:120), `loadWorks` (:134, cơ chế `worksReloadPending`),
  `currentLibraryWork` (:100), `resetLibraryWorks` (:355). **Hai khuôn con trỏ, chép cái thứ hai.**
- `src/modes/libraryImport.ts` — `beginSubmit` (:108–152, flush + đọc kết quả),
  `finishSubmit` (:154+, `resetSourcePanel`/`resetLookupPanel`/`resetEditorPanel`/
  `resetSegmentHistory` rồi **nạp lại**). Đây là khuôn cho `openWorkById`.
- `src/modes/LibraryMode.vue` — `.works-grid` (~532–604, ô `work-cell` + `data-library-work-cell`),
  `.grid-nav` (~510–530), khối `.open-work-block` (~606–669) là chỗ **gắn danh sách Chương**,
  khối `.empty` (~678–690), `<style scoped>` từ ~613.
- `src/commands/index.ts` — cụm `library.*` (~877–970), `library.work_next`/`work_prev`
  (~276–278 khai `CommandDeps`), khuôn `portMissing`. `src/main.ts:340–360` `installCommands({…})`
  là chỗ tiêm; `setMode` đã có sẵn trong `deps` (`:341`).
- `src/modes/modeState.ts:38` `setMode` — đường chuyển sang Workspace sau khi mở Chương.
- `src/i18n/vi.json` — cụm `mode.library.*`, `command.library.*`, `err.*`.

**Cổng và bàn đo**

- `src-tauri/tests/meta_write_boundary.rs` — (a) `.write_atomic(` đúng ba tệp; (c)
  `WorkMeta::read` đúng **hai** tệp. Story này đọc `meta.json` ở một chỗ **thứ ba** ⇒ hoặc đi
  vòng qua một hàm của `core/library/`, hoặc mở rộng danh sách **kèm lý do tại chỗ**. Xem §Design Notes.
- `src-tauri/tests/library_index_boundary.rs` — cấm module ngoài `core/library/**` mang từ vựng
  chỉ mục. `src-tauri/tests/library_index_contract.rs` — hàm dựng `.atproj` **thật** (~92–125),
  hai chuỗi `meta.json` viết tay (~438, ~1325).
- `src-tauri/tests/project_contract.rs` — `a_newer_meta_schema_is_refused_without_touching_a_single_byte`
  (ca đã có cho cơ chế; story này thêm ca cho **bề mặt**).
- `src-tauri/tests/ipc_contract.rs:472–535` — đóng băng khoá `snake_case` của `WorkRow`/
  `WorkListReport`; mọi struct mới trên dây phải có ca ở đây.
- `src-tauri/tests/segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
  — **phải giữ xanh** (§Block If).
- `tests/frontend/libraryWorks.test.ts` — `invoke` giả, khuôn ca mount `LibraryMode.vue`.
- `e2e/specs/story-5-6-library-grid.e2e.mjs` — `createWork()` qua IPC trần (~79–101),
  `gridProbe()` một lệnh WebDriver (~107), `realClick`, focus-bằng-JS-rồi-gửi-phím cho ca
  "bằng bàn phím". `e2e/support/pointer.mjs::realClick`, `e2e/support/panelReset.mjs`.
- `scripts/check-commands.mjs` — `COMMAND_FLOOR` 52 · `CLICK_FLOOR` 27 · `DISPATCH_FLOOR` 40
  (đều là **cận DƯỚI**; thêm lệnh không làm đỏ). Kiểm A chỉ canh `@click`.
- `scripts/check-panel-refs.mjs` — mọi ô nhớ cấp module phải có một đường `reset*()`.

## Tasks & Acceptance

**Execution:**

1. [x] `src-tauri/src/core/store/schema.rs` -- thêm hằng `CHAPTER_POSITION_DDL` (bảng
   `chapter_position(chapter_id INTEGER PRIMARY KEY, segment_id INTEGER NOT NULL, updated_at
   TEXT NOT NULL)`) kèm doc-comment nêu: vì sao **một bảng riêng** chứ không một cột trên
   `chapter`, vì sao **vắng hàng = chưa từng mở**, và vì sao **không** `FOREIGN KEY` (cùng khuôn
   cả lược đồ, `PRAGMA foreign_keys` mặc định tắt); nối vào `PROJECT_MIGRATIONS` làm bước
   **17** -- một bảng mới đi bằng một bước MỚI, không sửa `CHAPTER_DDL` tại chỗ (vết sẹo số 4).
2. [x] `src-tauri/src/core/library/indexer.rs` -- thêm `Indexer::find_work(work_id) ->
   Result<Option<IndexedWork>, StoreError>`: một câu `SELECT {COLUMNS} FROM library_work WHERE
   work_id = ?1`, dùng lại `COLUMNS`/`map_row` của `list_works` (tách `map_row` ra thành một hàm
   tự do trong module nếu closure không chia sẻ được) -- `atproj_path` phải phân giải ở tầng
   chỉ mục, không ở tầng lệnh (`library_index_boundary.rs`).
3. [x] `src-tauri/src/core/library/mod.rs` -- thêm lại biến thể `WorkError::MetaTooNew { found,
   supported }` và `WorkError::OpenFailed { name, detail }`; sửa **tại chỗ** khối comment
   `:50–59` kèm 🔵 + ngày (mệnh đề *"story này không dựng màn hình mở lại"* hết đúng ở lượt
   này); `From<MetaError>` ánh xạ `SchemaTooNew` sang biến thể mới thay vì gộp vào
   `CreateFailed`; `From<WorkError> for IpcError` đi qua `IpcError::new` cho cả ba.
4. [x] `src-tauri/src/core/i18n/mod.rs` -- ba khoá mới trong `message_keys!`, cụm Story 5.7 kèm
   comment nói rõ **những khoá nào được TÁI DÙNG** (`WorkNoneOpen`, `SegmentChapterNotFound`)
   để không ai đúc khoá thứ hai cho cùng một câu: `WorkMetaTooNew => "err.work.meta_too_new"
   ["found", "supported"]` · `WorkOpenFailed => "err.work.open_failed" ["name"]` ·
   `LibraryWorkNotIndexed => "err.library.work_not_indexed" ["work_id"]`.
5. [x] `src-tauri/src/commands/project.rs` -- hàm thuần `open_work(root: &Path, indexed:
   &IndexedWork) -> Result<OpenWork, IpcError>`: `WorkMeta::read` → `Store::open(StoreSpec::
   project(dir/project.db))` → `ScopeResolver::with_work` → chọn `chapter_id` bằng
   `SELECT id FROM chapter ORDER BY ord, id LIMIT 1`; **không** `remove_folder` ở bất kỳ nhánh
   lỗi nào (khác hẳn `create_work`: thư mục này là **dữ liệu có sẵn của người dùng**, không phải
   thư mục lượt gọi vừa tạo — ghi mệnh đề đó thành doc-comment 🔴). Vỏ `wire::open_work(app,
   work_id)` lấy `Indexer` + `Store` toàn cục qua `try_state`, `resolve_library_root`, gọi
   `Indexer::find_work`, `None` ⇒ `library.work_not_indexed`, rồi `replace_open_work`; trả
   `CreatedWork`-shape mở rộng (`meta` · `folder` · `chapter_id`) dưới tên `OpenedWork`.
6. [x] `src-tauri/src/commands/chapter.rs` -- ① `struct ChapterRow { chapter_id, ord, title:
   Option<String>, status, segment_count }` + hàm thuần `list_chapters(open: Option<&OpenWork>)`
   (`SELECT c.id, c.ord, c.title, c.status, (SELECT COUNT(*) FROM segment s WHERE s.chapter_id
   = c.id AND s.retired_at IS NULL) FROM chapter c ORDER BY c.ord, c.id`, **không**
   `source_text`); ② hàm thuần `open_chapter(open: Option<&mut OpenWork>, chapter_id)` -- kiểm
   hàng tồn tại TRƯỚC, dời `OpenWork::chapter_id` SAU (đúng luật đã ghi ở
   `open_adjacent_chapter:294`), gọi `warm_jieba_for_source_lang`, trả `OpenChapter`; hai vỏ
   trong `mod wire` (`try_state`, vỏ ghi giữ khoá `Mutex` qua lời gọi).
7. [x] `src-tauri/src/commands/segment.rs` -- thêm trường `caret_segment_id: Option<i64>` vào
   `ChapterSegments`; trong `read_open_chapter_segments`, sau lượt đọc segment, tính nó **trong
   cùng một lượt `Store::read`**: hàng `chapter_position` của `chapter_id` nếu `segment_id` đó
   **có mặt trong danh sách vừa đọc** (tức còn sống), ngược lại `segments.first().id`, và
   `None` khi danh sách rỗng -- ghi chẩn đoán KHÔNG DẤU ở nhánh rơi về (một vị trí trỏ vào
   segment đã về hưu là *rỗng CÓ LÝ DO*, không được im lặng). Thêm hàm thuần
   `save_chapter_position(open, chapter_id, segment_id)` (`INSERT … ON CONFLICT(chapter_id) DO
   UPDATE`) + vỏ; câu `UPDATE` của `save_segment_targets` **không đổi một dòng**.
8. [x] `src-tauri/src/lib.rs` -- đăng ký bốn vỏ mới vào `generate_handler![…]` (`open_work` ·
   `list_chapters` · `open_chapter` · `save_chapter_position`) kèm comment một dòng mỗi lệnh
   nói vai của nó, đúng khuôn các cụm đã có.
9. [x] `src-tauri/src/core/library/meta.rs` -- **không sửa hành vi**; chỉ mở rộng doc-comment của
   `WorkMeta::read` bằng 🔵 + ngày: từ lượt này nó **có** một chỗ gọi sản phẩm
   (`commands::project::open_work`), nên mệnh đề *"0 chỗ gọi sản phẩm"* mà ba tệp khác đang
   viện dẫn đã hết đúng.
10. [x] `src-tauri/tests/meta_write_boundary.rs` -- mở rộng danh sách CHO PHÉP của phép kiểm (c)
    (`WorkMeta::read`) thêm `commands/project.rs`, **kèm một comment tại chỗ nói vì sao** (đây
    là đường mở lại `.atproj` mà chính story 5.5 đã ghi là *"chưa tồn tại"*); giữ nguyên (a),
    (b), (d) và hai ca tự kiểm.
11. [x] `src-tauri/tests/project_contract.rs` (nhà của MỌI ca lệnh Chương hôm nay -- 13 lần nhắc
    `open_adjacent_chapter`, cộng helper `insert_chapter_directly` ~dòng 848 dựng Chương thứ
    hai; **không** dựng một tệp `chapter_contract.rs` thứ hai cho cùng một bề mặt) +
    `src-tauri/tests/library_index_contract.rs` (riêng ca `Indexer::find_work`) -- ca cho
    **mọi** hàng §I/O Matrix thuộc tầng Rust: mở được một `.atproj` dựng sẵn trên đĩa (và
    `ScopeResolver` là `with_work`, không `global_only`); `work_id` lạ ⇒
    `library.work_not_indexed`; thư mục biến mất ⇒ `work.open_failed`; `meta.json` v99 ⇒
    `work.meta_too_new` và **`meta.json` không đổi một byte**; `list_chapters` trả đúng thứ tự
    `(ord, id)` và **không** trường `source_text`; `open_chapter` với id lạ **không dời con
    trỏ**; vị trí lưu rồi đọc lại đúng; vị trí trỏ vào segment đã về hưu ⇒ rơi về segment đầu;
    Chương chưa từng mở ⇒ segment đầu; Chương rỗng ⇒ `None`.
12. [x] `src-tauri/tests/ipc_contract.rs` -- đóng băng khoá `snake_case` của `ChapterRow`,
    `OpenedWork` và trường mới `caret_segment_id` của `ChapterSegments`.
13. [x] `src/config/chapter.ts` + `src/config/library.ts` -- adapter ba trạng thái cho bốn lệnh
    mới, mỗi cái **một** `invoke` + **một** `try/catch`, không ném; kiểu `ChapterRow`/
    `OpenedWork` viết `snake_case` đúng như trên dây; vị từ kiểm kiểu **lúc chạy** cho mảng
    Chương và cho `caret_segment_id` (`number | null`, từ chối `undefined`).
14. [x] `src/panels/positionFlush.ts` (MỚI) -- cặp hằng RIÊNG `POSITION_IDLE_MS = 500` /
    `POSITION_HARD_CAP_MS = 5000` + một `createWriteSchedule(POSITION_IDLE_MS,
    POSITION_HARD_CAP_MS)`, doc-comment nói thẳng nó **KHÔNG** mang bảo đảm AD-35 (mất một
    lượt ghi vị trí là mất một lời nhắc, không mất công việc) và **không** đọc `Date.now()`
    bên trong -- mọi thời điểm đi vào qua tham số, đúng luật đã có.
15. [x] `src/panels/editorPanelState.ts` -- ① móc nhịp ghi vị trí vào `setEditorCaret` (chỗ DUY
    NHẤT định nghĩa *"rời segment"*), ghi khi tới hạn qua `saveChapterPosition`; ② hàm
    `flushChapterPositionNow()` gọi ở ba biên: trước lượt đổi Chương, trước lượt đổi Tác phẩm,
    và trong `wireExitFlush`; ③ `openChapterById(chapterId)` chép **trọn** khuôn `switchChapter`
    (cờ chống hai lượt cùng bay · flush đọc cả ba giá trị · IPC · reset · nạp lại · kiểm
    `chapterId` khớp giữa hai lượt IPC); ④ sau `ensureSegmentsLoaded()`, đặt
    `caretPlacement.value = caret_segment_id` để watcher của `GridPanel.vue` đặt caret **và**
    cuộn (AC4/AC5) -- không một hàm cuộn thứ hai; ⑤ `resetEditorPanel` dọn cả nhịp ghi vị trí.
16. [x] `src/modes/libraryChapters.ts` (MỚI) -- state + thao tác của khối "Chương": `chapters` ·
    `chaptersHaveLoaded` · `chaptersBusy` · `chaptersError` · `chapterCursor` (chép khuôn
    `workCursor` của `libraryWorks.ts`, **kể cả lượt kẹp biên sau mỗi lượt tải**) ·
    `currentLibraryChapter` · `loadChapters` (cơ chế chống nuốt lượt tải như `loadWorks`) ·
    `nextChapter`/`prevChapter` · `openWorkById` (flush → `open_work` → vứt state tầng Tác
    phẩm → `loadChapters` → nạp lại panel) · `openCurrentChapter` (gọi `openChapterById` rồi
    `setMode('workspace')`) · `resetLibraryChapters`; cộng **một hàm THUẦN** `chapterWindow(
    scrollTop, viewportHeight, rowHeight, total, overscan)` trả `{ start, end, padTop, padBottom }`
    -- tách thuần để vitest kiểm được cửa sổ mà không cần bố cục thật (`happy-dom` không có
    hình học).
17. [x] `src/commands/index.ts` -- năm lệnh mới với `labelKey` + cổng `portMissing` theo đúng khuôn
    cụm `library.*`: `library.open_work` · `library.chapter_next` · `library.chapter_prev` ·
    `library.open_chapter` · `library.list_chapters`; khai năm `CommandDeps` tương ứng.
18. [x] `src/main.ts` -- tiêm năm hàm mới vào `installCommands({…})`, cạnh cụm Story 5.4/5.6.
19. [x] `src/modes/LibraryMode.vue` -- ① nút "Mở Tác phẩm" trong mỗi ô lưới (`@click=
    "dispatch('library.open_work')"`, id **literal**, thao tác trên `currentLibraryWork` --
    cùng khuôn `library.forget_orphan`); ② khối `.chapters-block` mới: nhãn, dải `role="status"`
    hỏi `chaptersHaveLoaded` **trước** khi nói *"chưa có Chương nào"*, hai nút con trỏ `‹`/`›`,
    một nút "Mở Chương", và một `<ul class="chapters-list">` **cuộn có cửa sổ** (chiều cao hàng
    cố định từ token, hai `<li>` đệm trên/dưới, `@scroll` cập nhật `scrollTop`) hiện
    `ord`/tiêu đề (`title` `null` ⇒ `t('mode.library.chapter_untitled', { ord })`)/trạng thái/
    số câu, `aria-current` trên hàng đang chọn; ③ móc `data-library-chapter-*` cho e2e; CSS chỉ
    dùng token.
20. [x] `src/i18n/vi.json` -- khoá cho: ba thông điệp lỗi mới, nhãn khối Chương, `chapter_untitled`,
    câu "chưa tải Chương"/"Tác phẩm này chưa có Chương nào", nhãn năm lệnh mới, `aria-label`
    hai nút con trỏ, và câu báo khi lượt mở bị **chặn** vì flush chưa sạch.
21. [x] `tests/frontend/libraryChapters.test.ts` (MỚI) + `tests/frontend/libraryWorks.test.ts` --
    ca cho: `chapterWindow` (danh sách 2.000 hàng ⇒ `end - start` ≤ 60; cuộn tới cuối ⇒ cửa sổ
    kẹp đúng biên; `total = 0` ⇒ cửa sổ rỗng, không `NaN`); con trỏ Chương kẹp biên sau một
    lượt tải ngắn hơn; `next`/`prev` trên danh sách rỗng là no-op; mount `LibraryMode.vue` với
    `invoke` giả trả 2.000 Chương ⇒ **số `<li>` thật trong DOM ≤ 60**; nhánh
    `chaptersHaveLoaded === false` **không** nói *"chưa có Chương nào"*; `title: null` ⇒ nhãn
    dựng từ `ord`.
22. [x] `e2e/specs/story-5-7-open-chapter.e2e.mjs` (MỚI) -- một kịch bản đi trọn đường trong
    WKWebView thật.
    🔵 **VIẾT LẠI 2026-08-29 sau hai lượt đo, và cả hai vế của bản đầu đều đã bị BÁC.** Bản
    đầu viết *"`library.open_work` trên **chính Tác phẩm đó**"* và *"mở Chương **bằng bàn
    phím**… ô có `[data-caret]`"*. ① Mở lại **chính** Tác phẩm đang mở làm mọi phép khẳng định
    phía sau **đúng dù `open_work` có chạy hay không** (Tác phẩm vốn đã mở, và
    `LibraryMode.vue` tự gọi `loadChapters()` ở `onActivated`) — spec phải tạo một Tác phẩm
    **THỨ HAI** để đẩy Tác phẩm A ra khỏi `OpenWorkState` trước đã; ② `focus()` bằng JS +
    `browser.keys(['Enter'])` **không tới được phần tử** (đo: `window.__logs` rỗng, trong khi
    `realClick` trên cùng nút cùng phiên thì chạy), nên mọi lượt kích hoạt đi bằng
    `realClick`; ③ **không** móc `[data-caret]` nào tồn tại trong `GridPanel.vue` — vị trí
    khôi phục nghiệm thu bằng `caret_segment_id` đọc qua đúng lệnh IPC sản phẩm, cộng hai ca
    vitest mới (`chapterPosition.test.ts`) canh nửa frontend.
    ⇒ Hình dạng ĐÃ GIAO: tạo Tác phẩm A (hai câu) → lưu vị trí câu thứ hai qua IPC → tạo Tác
    phẩm B (một câu) để A rời `OpenWorkState` → làm mới lưới → con trỏ tới ô A →
    `library.open_work` → **đối chứng dương**: Chương đang mở quay về của A (phân biệt bằng
    NỘI DUNG, không bằng `chapter_id` — id là số cục bộ từng kho) → `library.open_chapter` →
    Workspace nạp lưới → vị trí khôi phục đúng câu thứ hai. Vế "bằng bàn phím" của AC7 **không**
    được spec này nghiệm thu — ghi nợ có chủ, xem §Auto Run Result.
23. [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- đóng **bằng chữ** bốn mục có
    chủ là story/epic này, mỗi mục kèm cách đóng: `:759` (bề mặt hiển thị `meta_too_new`),
    `:2632` (*"đóng app → mở lại → chữ còn đó"*), `:5004` (AC5 của Story 2.11 -- vị trí làm
    việc), `:5449` (Glossary tầng Tác phẩm sau một lượt mở lại). Mục `:8307` (ba lệnh vòng đời
    chỉ chạy trên Tác phẩm đang mở) đóng ở mức 🟡 kèm phần còn hở nếu bề mặt đổi trạng thái
    cho một Tác phẩm **không đang mở** vẫn chưa dựng. Không xoá mục nào.
24. [x] `src-tauri/AGENTS.md` + `src/AGENTS.md` -- sửa **tại chỗ** kèm 🔵 + ngày: mệnh đề
    *"`library-index.db`/`meta.json` là dẫn xuất… đường mở lại `.atproj` chưa tồn tại"* và
    (nếu có) mọi câu khác trong hai tệp đó nói *"không đường mở lại"*; ghi cặp hằng nhịp ghi
    **thứ ba** vào khối *"Hai cặp hằng nhịp ghi"* của `src/AGENTS.md` (nay là **ba**, và chỉ
    một mang bảo đảm AD-35).

**Acceptance Criteria:**

- **AC1** — Given một `.atproj` đã có trên đĩa và ứng dụng vừa khởi động (`OpenWorkState =
  None`), when người dùng mở nó từ lưới Library, then `OpenWorkState` trỏ vào Tác phẩm đó,
  `ScopeResolver` của nó là `with_work` (**không** `global_only`), và mọi mục Glossary tầng
  Tác phẩm ghi ở phiên trước đọc lại được — nghiệm thu bằng một ca Rust, không bằng suy luận.
- **AC2** — Given Tác phẩm đang mở, when danh sách Chương hiện, then mỗi hàng mang `ord`, tiêu
  đề (hoặc nhãn dựng từ `ord` khi `title IS NULL`), **trạng thái vòng đời** và số câu; và
  given một danh sách **2.000** Chương, when nó hiện, then số phần tử `<li>` thật trong DOM
  **≤ 60** và hàng đang chọn luôn nằm trong cửa sổ.
- **AC3** — Given danh sách Chương, when người dùng mở một Chương, then chế độ đang hoạt động
  đổi sang **Workspace** và cả Panel Editor lẫn Panel Source hiện nội dung của **đúng Chương
  đó** (hai panel không bao giờ nói về hai Chương khác nhau).
- **AC4** — Given một Chương đã từng làm việc (có hàng `chapter_position`), when mở lại nó,
  then caret nằm ở **đúng `segment.id`** đã lưu và ô đó **nằm trong vùng nhìn** — nghiệm thu
  qua đường `editorCaretPlacement` đã có, và **0** dòng tính `scrollTop` bằng pixel được thêm
  vào `src/**`.
- **AC5** — Given một Chương **chưa từng mở** (không hàng `chapter_position`), when mở nó,
  then caret ở segment **đầu tiên** theo `(ord, id)`; và given hàng vị trí trỏ vào một segment
  đã **về hưu**, when mở nó, then caret rơi về segment đầu **kèm một dòng chẩn đoán**, không
  rơi vào một id mà lưới không dựng ô cho.
- **AC6** — Given người dùng dời caret rồi đóng ứng dụng bình thường, when mở lại ứng dụng và
  mở lại chính Chương đó, then vị trí khôi phục đúng — tức nó đã nằm trong `project.db`, không
  trong state tạm của frontend; và `grep localStorage src/**` cho **0** kết quả cho vị trí này.
- **AC7** — Given chỉ dùng bàn phím, when người dùng mở một Tác phẩm, chạy con trỏ qua danh
  sách Chương và mở một Chương, then làm được trọn vẹn cả ba không cần chuột, và hàng đang
  chọn luôn nhìn thấy được.
- **AC8** — Given một `.atproj` có `meta.json` phiên bản **mới hơn** bản ứng dụng hiểu, when
  người dùng mở nó, then màn hình hiện một câu **nói đúng loại lỗi** (qua `err.work.meta_too_new`,
  có tham số `found`/`supported`), `OpenWorkState` **không đổi**, và **không một byte nào**
  trong `.atproj` bị ghi.
- **AC9** — Given tập chờ của Editor còn chữ chưa lưu, when người dùng mở một Tác phẩm khác
  hoặc một Chương khác và lượt flush trả `'failed'`/`'still-dirty'`, then thao tác bị **CHẶN**
  kèm câu báo và `OpenWorkState` không đổi — không một ký tự nào mất im lặng.
- **AC10** — Given toàn bộ cây, when `cargo test --locked` và `npm run test` chạy, then hai
  cổng `segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
  và `meta_write_boundary.rs` (bốn phép kiểm + hai ca tự kiểm) **vẫn xanh**, và
  `library_index_boundary.rs` vẫn xanh với đường phân giải `atproj_path` mới.

## Spec Change Log

## Review Triage Log

### 2026-08-29 — Review pass

- intent_gap: 0
- bad_spec: 0
- patch: 9: (high 2, medium 4, low 3)
- defer: 5: (high 0, medium 2, low 3)
- reject: 1: (high 0, medium 0, low 1)
- addressed_findings:
  - `[high]` `[patch]` `Store::open` mang `SQLITE_OPEN_CREATE`, nên `open_work` trên một
    `.atproj` còn `meta.json` nhưng MẤT `project.db` **âm thầm tạo** một kho rỗng trong thư
    mục người dùng rồi mới trượt bằng một lỗi kho chung chung — ghi vào dữ liệu người dùng ở
    đúng nhánh mà doc-comment của chính hàm đó tuyên bố *"chỉ được phép TỪ CHỐI MỞ"*. Vá:
    kiểm `db_path.exists()` TRƯỚC `Store::open`, trả `work.open_failed`. Canh bằng
    `project_contract.rs::opening_a_work_whose_project_db_is_missing_is_refused_and_creates_no_file`
    (khẳng định cả câu báo lẫn "đĩa không bị chạm"); đối chứng: gỡ cửa ⇒ ca đỏ.
  - `[high]` `[patch]` Cửa sổ **mở NHẦM Chương**: `openWorkById` nhả `openWorkBusy` ngay sau
    lượt IPC, trước bốn lượt vứt state và `loadChapters()`. Trong cửa sổ đó `OpenWorkState`
    đã sang Tác phẩm MỚI còn `.chapters-list` vẫn là của Tác phẩm CŨ, và vì `chapter.id` là
    `AUTOINCREMENT` **cục bộ từng `project.db`** (đo được ở chính bàn đo e2e của story này:
    hai Tác phẩm đều có `chapter_id = 1`), một cú bấm "Mở Chương" mở một Chương **khác hẳn**
    thứ người dùng vừa bấm — `Ok`, không lỗi, không dấu hiệu. Vá ba vế: giữ `openWorkBusy`
    tới cuối hàm, vứt `chapters`/`chaptersHaveLoaded`/`chapterCursor` ngay khi kho đổi, và
    tắt nút "Mở Chương" theo `libraryOpenWorkBusy`.
  - `[medium]` `[patch]` `open_work` trên một `project.db` **không hàng `chapter` nào** trả
    một lỗi KHO (`query_row` biến 0 hàng thành `QueryReturnedNoRows`) — đúng lớp *"một câu
    SAI VỀ LOẠI"* mà Story 2.11 đã sửa một lần. Vá bằng khuôn `query_map().next()` của
    `read_open_chapter`, trả `work.open_failed` mang TÊN Tác phẩm; **không** đúc khoá thứ tư
    (0 đường sản phẩm nào tới được ca này trước Story 5.8). Canh bằng
    `opening_a_work_with_no_chapter_rows_is_a_named_error_not_a_store_error`; đối chứng đỏ.
  - `[medium]` `[patch]` Cửa chặn AC9 của `openWorkById` (mở Tác phẩm KHÁC khi flush chưa
    sạch) **không có ca nào** ở bất kỳ tầng nào — gỡ trọn khối `if (flushed !== 'clean')` đi
    qua sạch cả bộ. Thêm hai ca ở `libraryChapters.test.ts` (`'failed'` và `'still-dirty'`,
    hai câu báo phân biệt được), khẳng định `open_work` **không** lên dây; đối chứng đỏ.
  - `[medium]` `[patch]` Nửa ĐỌC của AC4/AC5 ở frontend không được canh: xoá dòng
    `caretPlacement.value = loaded?.caret_segment_id ?? null` không làm ca nào đỏ (ca Rust
    chỉ thấy giá trị trên dây). Hồi quy sẽ làm caret **luôn** rơi về câu đầu — một lỗi đội
    lốt giá trị hồi phòng hợp lệ của AC5. Thêm ca ở `chapterPosition.test.ts`; đối chứng đỏ.
  - `[medium]` `[patch]` Nửa GHI của AC4/AC6 không được canh: `setEditorCaret` → 
    `positionFlush.markMoved` là đường một người dùng THẬT đi qua, mà bàn đo e2e cố ý ghi vị
    trí bằng IPC trần còn các ca `createPositionFlush` gọi `markMoved` bằng tay. Gỡ hai dòng
    móc đi qua sạch cả bộ, và `chapter_position` sẽ không bao giờ được ghi trong dùng thật.
    Thêm ca đi qua `setEditorCaret` + `flushChapterPositionNow`; đối chứng đỏ.
  - `[low]` `[patch]` Bốn doc-comment ở `commands/chapter.rs` trỏ mã lỗi
    `project.no_work_open` — **không tồn tại trong kho** (mã thật: `work.none_open`). Hai
    trong bốn là bản chép của story này, hai kia có từ trước; sửa cả bốn thay vì để một mệnh
    đề sai nằm cạnh bản vá của chính nó.
  - `[low]` `[patch]` Doc-comment đầu `commands/chapter.rs` khai *"`read_open_chapter`/
    `open_adjacent_chapter` là hai điểm sản phẩm DUY NHẤT đưa một `source_lang` mới lên
    webview"* — story này thêm điểm thứ ba (`open_chapter`) trong chính tệp đó. Sửa tại chỗ
    kèm 🔵 + ngày, và nói rõ vì sao `open_work` KHÔNG thuộc danh sách.
  - `[low]` `[patch]` Pager của danh sách Chương mượn khoá `mode.library.work_position` của
    lưới Tác phẩm — một khoá, hai bề mặt, nên một lượt sửa cho bề mặt này lặng lẽ đổi bề mặt
    kia. Tách `mode.library.chapter_position`.
  - `[low]` `[patch]` Task 22 trong chính spec này mô tả một bàn đo **không phải** thứ đã
    giao (mở Chương "bằng bàn phím", móc `[data-caret]` không tồn tại, mở lại chính Tác phẩm
    đang mở). Viết lại kèm 🔵 và hai phép đo đã bác bản đầu.

## Design Notes

### Vì sao một BẢNG riêng cho vị trí, không một cột trên `chapter`

Ba lý do đo được, không một sở thích:

1. **Vắng hàng là một trạng thái phân biệt được.** AC5 đòi *"Chương chưa từng mở ⇒ segment
   đầu"*, và một cột `chapter.last_segment_id NULL` cũng nói được điều đó — nhưng nó nói
   **cùng chỗ** với dữ liệu nội dung Chương, nên mọi lượt đọc `chapter` (danh sách, tách
   segment, vòng đời) kéo theo một cột không liên quan tới vai của lượt đọc đó.
2. **`chapter` đang bị ba đường đọc/ghi khác chạm.** `WorkMeta::rebuild_from_store` đọc
   `SELECT status FROM chapter`, `commands/lifecycle.rs:143` `UPDATE chapter SET status`,
   `create_work` `INSERT INTO chapter`. Thêm một cột **đổi theo mỗi lượt rê caret** vào chính
   bảng ấy đặt một giá trị nhịp-cao cạnh những giá trị nhịp-thấp, và `chapter.updated_at` —
   thứ `rebuild_from_store` nay dùng để tính `updated_at` của Tác phẩm (Story 5.6) — sẽ nhảy
   theo mỗi lượt **đọc** của người dùng. Đó là một hồi quy im lặng cho AC4 của Story 5.6.
3. **Bảng riêng không đòi `chapter.updated_at` phải đổi.** `chapter_position.updated_at` là
   mốc của **chính hàng vị trí**, tách hẳn.

⚠️ **Giới hạn thật, ghi ra thay vì để người sau tưởng đã xét:** bảng này không có `FOREIGN KEY`
tới `chapter` (cùng khuôn cả lược đồ — `PRAGMA foreign_keys` mặc định TẮT trong SQLite, một
khoá ngoại khai ra mà không bật pragma là một lời hứa không ai giữ). Một Chương bị xoá (Story
5.8) để lại một hàng vị trí mồ côi. Nó **vô hại** (`chapter_id` không tái dùng —
`AUTOINCREMENT`) nhưng nó là rác; chủ dọn là **Story 5.8**, ghi vào `deferred-work.md`.

### Vì sao KHÔNG có "Chương mở gần nhất của Tác phẩm" ở lượt này

Đo 2026-08-29: `INSERT INTO chapter` xuất hiện **đúng một lần** trong toàn kho
(`commands/project.rs:271`, bên trong `create_work`), và `epics.md` giao đường sinh Chương thứ
hai cho **FR14 (Epic 6)** và **FR15 (Story 5.8)**. ⇒ Hôm nay mọi Tác phẩm có **đúng một**
Chương, nên *"mở Tác phẩm ở Chương nào"* là một câu hỏi **không quan sát được**. Thêm một cột
`work.last_chapter_id` bây giờ là đoán trước một quyết định UX mà không AC nào đòi và không
phép đo nào đỡ — đúng thứ *"một khoá cho tính năng chưa tồn tại"* mà Story 1.7 §Completion
Notes #3 cấm. `open_work` mở **Chương đầu theo `(ord, id)`**, và danh sách Chương là bề mặt
chọn. Món nợ ghi có chủ: **Story 5.8** (khi Chương thứ hai tồn tại thật).

### `focus()` LÀ vế cuộn — không viết hàm cuộn thứ hai

`GridPanel.vue:1110–1165` chở **ba phép đo** của Story 2.10 (Ice ký 2026-08-18) chứng minh
`target.focus()` tự cuộn ô vào vùng nhìn **và** cuộn khéo hơn công thức tự cài (WebKit căn
giữa khi đích ở xa — đo `scrollTop` 1569 so với 1242 của công thức nearest). ⇒ AC4 nối vào
đúng đường ấy bằng cách đặt `caretPlacement`, và bất kỳ dòng `scrollTop`/`scrollIntoView` nào
thêm vào `src/**` ở story này là mã chết theo đúng nghĩa §Miễn trừ của kho.

### `meta_write_boundary.rs` — vì sao NỚI danh sách chứ không đi vòng

Cổng AC4 của Story 5.5 khoá `WorkMeta::read` ở **đúng hai** tệp. Story này thêm chỗ đọc thứ
ba, và có hai đường:

- **(a) nới danh sách CHO PHÉP** thêm `commands/project.rs`, kèm lý do tại chỗ.
- **(b) đi vòng**: bọc `WorkMeta::read` bằng một hàm mới trong `core/library/` rồi gọi hàm bọc.

Chọn **(a)**. Đường (b) làm cổng xanh mà **không** đổi sự thật nó đang canh — đúng hình dạng
*"bọc một hàm thuần chỉ để test không phải một bản vá"*: vỏ chỉ chuyển tiếp, nên mệnh đề
*"đúng hai tệp đọc `meta.json`"* trở thành sai trong khi cổng vẫn xanh. Cổng tồn tại để **đếm
chỗ đọc**, và số chỗ đọc thật sự tăng từ 2 lên 3 — nên con số trong cổng phải tăng theo, kèm
tên tệp và lý do. Ba phép kiểm còn lại và hai ca tự kiểm giữ nguyên.

### Cửa "chưa tải xong" của danh sách Chương

Cùng lớp lỗi mà `AGENTS.md::Known pitfalls` đã đếm **ba** lần (Story 1.16 · 2.10 · 3.9): một
danh sách rỗng không tự nói vì sao nó rỗng. `chaptersHaveLoaded` là vị từ, và **tham số đi vào
nó cũng phải kiểm** — bài học Story 3.9 là vị từ thì đúng còn chỗ gọi bịa mất con số. ⇒ Khối
Chương mang **ba** câu phân biệt được: *chưa tải* · *đã tải, Tác phẩm này không có Chương nào*
· *đã tải, có `n` Chương*. Ca vitest ở Task 21 khẳng định nhánh thứ nhất **không** nói câu của
nhánh thứ hai.

### Cặp hằng thứ BA, và nó không mang bảo đảm AD-35

`src/AGENTS.md` ghi *"Hai cặp hằng nhịp ghi, chỉ MỘT mang bảo đảm AD-35 … dùng chung hình
dạng, không dùng chung bảo đảm — đừng gộp hai cặp."* Story này thêm cặp thứ ba
(`POSITION_IDLE_MS`/`POSITION_HARD_CAP_MS`), và nó thuộc nhóm **không** mang bảo đảm: mất một
lượt ghi vị trí là mất **một lời nhắc**, không mất công việc — cùng hạng với nhịp ghi bố cục,
khác hạng với `EDITOR_IDLE_MS`. Ghi thẳng vào doc-comment của tệp mới **và** cập nhật bảng
trong `src/AGENTS.md` cùng lượt, để mệnh đề "hai cặp" không lặng lẽ sai.

Vì sao **phải** có nhịp ghi chứ không ghi thẳng ở mỗi lượt dời caret: `setEditorCaret` chạy
theo **mỗi phím mũi tên** của `segmentNavigation`, và `flushEditorNow()` ở đó gần như luôn
trả `'clean'` **không tốn một lượt IPC nào** khi tập chờ rỗng. Ghi thẳng biến một lượt rê
caret thành một job qua `store::Writer` nối tiếp mỗi phím — đúng bẫy `onDidLayoutChange` mà
`src/layout/writeSchedule.ts` tồn tại để chặn, và **không cổng nào đỏ vì nó**.

### Cuộn có cửa sổ tự viết, không một phụ thuộc

`chapterWindow()` là một **hàm thuần** nhận `(scrollTop, viewportHeight, rowHeight, total,
overscan)` và trả `{ start, end, padTop, padBottom }` — không đọc DOM, không đọc đồng hồ, nên
vitest kiểm được nó **tất định và tức thời** trên `happy-dom` (không có hình học). Phần `.vue`
chỉ nối `@scroll` → `scrollTop` và render `segments.slice(start, end)` cộng hai `<li>` đệm
mang `height` tính sẵn. Chiều cao hàng **cố định**, đến từ token — chiều cao thay đổi theo nội
dung sẽ đòi đo hình học thật, tức một mệnh đề thuộc bàn đo/e2e chứ không thuộc vitest.

## Verification

**Commands:**
- `npm run check:i18n` -- expected: exit 0. Ba khoá `MessageKey` mới đồng bộ với `vi.json`; không chuỗi tiếng Việt có dấu ở vị trí mã trong `src-tauri/src/**`.
- `npm run check:commands` -- expected: exit 0. Mỗi `@click` mới là đúng một `dispatch('<id>')` id literal; năm lệnh mới đăng ký đủ; sàn quần thể không thủng.
- `npm run check:tokens` -- expected: exit 0. Mọi màu/cỡ chữ của khối Chương đến từ token.
- `npm run check:panel-refs` -- expected: exit 0. `libraryChapters.ts` và `positionFlush.ts` mỗi tệp có một đường `reset*()`.
- `npm run check:layout` -- expected: exit 0. `writeSchedule.ts` giữ nguyên hành vi (Kiểm B đếm số lượt ghi).
- `npm run test` -- expected: mọi tệp xanh, kể cả `libraryChapters.test.ts` mới và `libraryWorks.test.ts` đã có.
- `npm run build` -- expected: `vue-tsc` 0 lỗi, `vite build` xong (chạy TRƯỚC `cargo test`).
- `cargo test --locked` (trong `src-tauri/`) -- expected: 0 đỏ; nêu đích danh `meta_write_boundary` (6 ca), `segment_contract::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`, `library_index_boundary`, `ipc_contract`.
- `npm run test:e2e` -- expected: `story-5-7-open-chapter.e2e.mjs` xanh, và **mọi** spec cũ giữ nguyên số ca xanh (một lượt đỏ do đổi bố cục Library phải sửa CÙNG LƯỢT, không để bị đọc thành hồi quy).

**Manual checks (if no CLI):**
- Gỡ hàng `chapter_position` khỏi truy vấn ở Task 7 rồi chạy lại bộ ca của Task 11: **phải ĐỎ**. Một bộ xanh không chứng minh chỗ nối mới được canh (`AGENTS.md::Known pitfalls` — Epic 3 dính năm lần trong bảy ngày).
- `grep -rn "scrollTop\|scrollIntoView" src/` sau lượt sửa: **0** kết quả mới so với baseline `6b2cb24` — bằng chứng của AC4 vế *"không dựng đường cuộn thứ hai"*.
- Đọc lượt CI (macOS **và** Windows) trước khi kết luận xanh — `pre-push` chỉ chạy trên macOS của Ice.

## Auto Run Result

Status: done
Blocking condition: (không có)

### Đã dựng

Đường **mở lại một `.atproj` đã có trên đĩa** — món nợ kiến trúc trung tâm của Epic 5 — cộng
danh sách Chương có cuộn cửa sổ và vị trí làm việc bền trong `project.db`. Bốn vỏ IPC mới
(`open_work` · `list_chapters` · `open_chapter` · `save_chapter_position`), bước di trú **17**
(`chapter_position`, khoá theo `chapter_id`, giữ `segment.id` — **không** pixel, AD-3), năm id
lệnh mới, và một cặp hằng nhịp ghi **thứ ba** không mang bảo đảm AD-35.

### Tệp đã đổi

**Rust — lõi và bề mặt IPC**
- `core/store/schema.rs` — hằng `CHAPTER_POSITION_DDL` + bước 17 của `PROJECT_MIGRATIONS`.
- `core/library/indexer.rs` — `Indexer::find_work(work_id)`, tách `map_indexed_work_row`.
- `core/library/mod.rs` — trả lại `WorkError::MetaTooNew` (Story 1.15 cố ý gỡ) + `OpenFailed`.
- `core/library/meta.rs` — `WorkMeta::read` nay CÓ một chỗ gọi sản phẩm; sửa mệnh đề tại chỗ.
- `core/i18n/mod.rs` — ba khoá: `WorkMetaTooNew` · `WorkOpenFailed` · `LibraryWorkNotIndexed`.
- `commands/project.rs` — hàm thuần `open_work` + vỏ; **không** `remove_folder` ở nhánh lỗi nào.
- `commands/chapter.rs` — `ChapterRow`/`list_chapters`/`open_chapter` + hai vỏ.
- `commands/segment.rs` — `ChapterSegments.caret_segment_id` + `save_chapter_position`.
- `lib.rs` — đăng ký bốn vỏ mới.

**Frontend**
- `config/{chapter,library,segment}.ts` — adapter ba trạng thái + vị từ kiểm kiểu lúc chạy.
- `panels/positionFlush.ts` (MỚI) — cặp hằng thứ ba, doc-comment nói rõ nó KHÔNG mang AD-35.
- `panels/editorPanelState.ts` — móc nhịp ghi vị trí, `openChapterById`, nối `caretPlacement`.
- `modes/libraryChapters.ts` (MỚI) — state khối Chương + hàm thuần `chapterWindow()`.
- `modes/LibraryMode.vue` — nút "Mở Tác phẩm"/"Mở Chương", danh sách Chương cuộn cửa sổ.
- `commands/index.ts` · `main.ts` · `i18n/vi.json` — năm lệnh, chỗ tiêm, nhãn.

**Lưới canh**
- `tests/project_contract.rs` (+8 ca) · `segment_contract.rs` · `ipc_contract.rs` ·
  `meta_write_boundary.rs` (nới danh sách + một ca đối chứng dương) · `pinned_contract.rs` (neo
  số bước di trú).
- `tests/frontend/{libraryChapters,chapterPosition}.test.ts` (MỚI) · `libraryWorks.test.ts`.
- `e2e/specs/story-5-7-open-chapter.e2e.mjs` (MỚI).

**Tài liệu** — `deferred-work.md` (đóng 4 mục ✅ + 1 mục 🟡, mở 3 mục mới có chủ) ·
`src-tauri/AGENTS.md` · `src/AGENTS.md` · `sprint-status.yaml`.

### Review

- **patch: 9** (high 2 · medium 4 · low 3) — xem §Review Triage Log.
- **defer: 5** (medium 2 · low 3) — ghi ở frontmatter `deferred`.
- **reject: 1** — *"`open_work` thiếu `warm_jieba_for_source_lang`"*: bác bằng đo. `open_work`
  không đưa `source_lang` nào lên webview; `openWorkById` luôn theo sau bằng
  `ensureChapterLoaded()` → `read_open_chapter`, và **lượt đó** hâm. Thêm một lời gọi nữa chỉ
  là mã thừa.
- **Follow-up review: `true`** — hai phát hiện đã vá ở mức `high`.

### Nghiệm thu ĐÃ CHẠY (tôi tự chạy, không lấy báo cáo của agent)

| Lệnh | Kết quả |
|---|---|
| 10 cổng `check:*` (gồm `check:lint`) | PASS |
| `npm run build` | `vue-tsc` 0 lỗi |
| `npm run test` | **47 tệp / 644 ca**, 0 đỏ |
| `cargo test --locked` | **906 xanh, 0 đỏ** |
| `npm run test:e2e` (trọn bộ, trước lượt vá) | 16/18 spec xanh |
| `story-5-7-open-chapter.e2e.mjs` (sau lượt vá) | xanh |

Hai cổng §Block If giữ xanh: `segment_contract::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
và `meta_write_boundary` (15 ca).

**Mọi mệnh đề mới đều được đối chứng bằng một lượt ĐỎ**, không bằng một lượt xanh: gỡ cửa
chặn flush · gỡ dòng nối `caretPlacement` · gỡ móc nhịp ghi vị trí · gỡ `replace_open_work` ·
gỡ hai cửa mới của `open_work` — mỗi lượt làm đúng ca tương ứng đỏ, rồi khôi phục byte-chính-xác.

20/20 hàng §I/O Matrix có ca phủ, và mọi ca đều đã chạy thật trong lượt xanh ở trên.

### Rủi ro còn lại

1. 🔴 **Vế "bằng bàn phím" của AC7 CHƯA được nghiệm thu.** Đo được: bộ e2e không kích hoạt
   được một `<button>` bằng phím (`focus()` bằng JS + `browser.keys` ⇒ handler không chạy;
   `realClick` trên cùng nút thì chạy). Sản phẩm dùng `<button>` thật mang `@click`, tức hành
   vi gốc HTML — nhưng *"không có mã để hỏng"* **không phải** một phép đo. Nợ có chủ (Ice).
2. ⚠️ **`story-5-6-library-grid.e2e.mjs` ĐỎ, và KHÔNG phải hồi quy của story này** — tái tạo
   ba lượt, gồm một lượt trên cây đã `git stash` sạch tại `6b2cb24`. Cùng gốc với mục 1. Chủ:
   Story 5.6, đang còn mở.
3. ⚠️ **`story-3-5-review.e2e.mjs` đỏ MỘT lần rồi xanh khi chạy riêng** — chập chờn, ca đó
   khẳng định một cuộc đua theo thời gian. Diff của story này chạm **0 dòng** đường quét khi
   nhập. Chưa có bước tái tạo ⇒ không đọc thành "đã sửa". Chủ: Ice.
4. ⚠️ **AC4 vế "đúng vị trí cuộn" nghiệm thu GIÁN TIẾP.** Vị trí lưu là `segment.id`; phần
   "cuộn" dựa vào `focus()` của watcher `GridPanel.vue` — đường đã có ba phép đo từ Story 2.10
   nhưng **không** được đo lại ở story này (bộ e2e không đặt được tiêu điểm DOM trên một
   `<span contenteditable>`, cùng giới hạn đã ghi ở `deferred-work.md:2557`, chủ Story 2.3).
5. ⚠️ **AC2 vế "cuộn mượt" chỉ nghiệm thu ở mặt CẤU TRÚC** (`<li>` trong DOM ≤ 60), không đo
   khung hình. Phép đo hiệu năng đủ điều kiện là Story 6.18, sau khi Epic 6 có đường tạo 5.000
   Chương thật.
6. ⚠️ **Chỉ chạy trên macOS.** `pre-push` và mọi lượt trên là máy của Ice; nửa Windows chưa
   chạy. Đọc lượt CI trước khi kết luận xanh.
7. ⚠️ **Story 5.6 vẫn `blocked`/`in-progress`** (chờ Ice chốt A/B), trong khi mã của nó đã ở
   trên `master` từ `6b2cb24`. Story 5.7 dựng trên nền đó.
