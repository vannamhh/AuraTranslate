---
title: 'Story 5.4: Bốn trạng thái vòng đời'
type: 'feature'
created: '2026-08-27'
status: 'done'
baseline_commit: 'e11004fc07e38e3cc8ce2122e5797214e50e3760'
baseline_revision: 'e11004fc07e38e3cc8ce2122e5797214e50e3760'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-5-context.md'
warnings: ['oversized']
deferred:
  - summary: >-
      Tầng Chương KHÔNG có bề mặt hiển thị nào: không màn hình nào đọc lại rồi hiện trạng thái
      của một Chương, và lối ghi duy nhất là một nút cứng "Đặt Chương này là Đã xong" trên
      Chương đang mở — ba chuyển đổi còn lại (`not_started`/`in_progress`/`paused`) không có
      lối vào nào.
    evidence: |-
      Backend `commands::lifecycle::set_chapter_status` nhận MỌI `chapter_id` và CẢ BỐN giá trị
      (phủ bằng ca 4x4 `no_combination_of_two_chapters_ever_derives_a_paused_work_through_the_command_layer`),
      nhưng `src/modes/libraryWorks.ts::setOpenChapterStatus` viết cứng `'done'` và chỉ chạy trên
      `editorChapterId`. AC2 của Story 5.4 nói trạng thái "có ở cả tầng Tác phẩm và tầng Chương"
      dưới mệnh đề "When mô hình hoá / When áp" — đọc là mệnh đề MÔ HÌNH, và Story 5.7 mang AC
      riêng "mở một Tác phẩm thì thấy danh sách Chương kèm trạng thái từng Chương", tức bề mặt
      hiển thị tầng Chương có chủ ở đó. Ghi ra vì lượt rà 2026-08-28 nêu đúng chỗ lệch này và
      sổ nợ chưa có mục nào cho ba chuyển đổi còn thiếu.
    location: >-
      src/modes/libraryWorks.ts::setOpenChapterStatus — chủ: Story 5.7
    severity: medium
  - summary: >-
      Bốn vỏ `#[tauri::command]` mới chưa được một lượt chạy THẬT nào chạm — chỉ e2e chạm
      chúng, và bộ e2e chạy ở nhịp đêm, không ở `push`.
    evidence: |-
      `lifecycle_contract.rs` gọi `set_chapter_status_indexed`/`set_work_status_override_indexed`
      (hàm thuần), không gọi `mod wire`; `tests/frontend/libraryWorks.test.ts` mock `invoke`.
      Nên một mục `generate_handler!` bị rơi, một tên lệnh gõ sai, hay một lệch camelCase
      (`chapterId`) sẽ đi qua trọn `pre-push` + CI ở nhịp `push` mà không cổng nào đỏ.
      `e2e/specs/story-5-4-lifecycle.e2e.mjs` tồn tại đúng để đóng chỗ này nhưng CHƯA chạy lần
      nào. Vế "e2e nhịp đêm" là quyết định kho có sẵn (`e2e/AGENTS.md`), không phải thứ story
      này dựng ra.
    location: >-
      src-tauri/src/commands/lifecycle.rs (mod wire) + src-tauri/src/lib.rs generate_handler!
    severity: medium
  - summary: >-
      Ánh xạ giá trị vòng đời sang nhãn i18n bị chép tay BA lần trong `LibraryMode.vue` và không
      dẫn xuất từ `LifecycleStatus::label_key()` phía Rust.
    evidence: |-
      `label_key()` chỉ được một ca test Rust đọc (khoá tồn tại trong `vi.json`); nó không đi qua
      bề mặt IPC nào, nên không bảo đảm gì cho chuỗi ternary viết cứng ở `.vue`. Đổi tên đồng
      thời cả khoá Rust lẫn khoá `vi.json` sẽ để frontend trỏ vào một khoá đã chết mà không cổng
      nào đỏ. Xác suất thấp (hai lượt đổi phải cùng lúc), nên ghi nợ thay vì dựng thêm một bề mặt
      IPC chỉ để chở nhãn.
    location: >-
      src/modes/LibraryMode.vue — chủ: Story 5.6
    severity: low
  - summary: >-
      `e2e/specs/story-5-4-lifecycle.e2e.mjs` định vị nút bằng `:nth-of-type`, không bằng móc
      định danh — đổi thứ tự sáu nút trong `LibraryMode.vue` làm spec bấm nhầm nút thay vì đỏ.
    evidence: |-
      Các selector dạng `.works-block .filter-actions .btn:nth-of-type(1|4|5|6)`. Chưa sửa ở lượt
      này vì không cách nào nghiệm thu một lượt sửa selector khi bộ e2e chưa chạy được ở đây —
      sửa mù một spec chưa chạy là đổi một rủi ro lấy một rủi ro khác.
    location: >-
      e2e/specs/story-5-4-lifecycle.e2e.mjs — chủ: Story 5.6
    severity: low
---

<intent-contract>

## Intent

**Problem:** FR5/FR6 đòi bốn trạng thái vòng đời (*Chưa bắt đầu · Đang dịch · Tạm ngưng · Đã xong*) ở **cả hai tầng** Tác phẩm/Chương, suy ra tự động ở tầng Tác phẩm và ghi đè thủ công được — nhưng hôm nay chỉ có đúng một mẩu: cột `chapter.status` với một hằng cục bộ `CHAPTER_STATUS_NOT_STARTED = "not_started"` (`commands/project.rs:35`) mà **không đường mã nào đọc, không đường mã nào đổi**. Tầng Tác phẩm không có trạng thái nào cả; `LIBRARY_WORK_DDL` khai thẳng *"không cột trạng thái vòng đời (chủ Story 5.4)"*. Hệ quả là câu chuyện người dùng của epic — *"phân biệt chưa-bắt-đầu với đã-làm-dở-rồi-bỏ trên 2000 chương"* — chưa có một byte dữ liệu nào chống lưng.

**Approach:** Khai bốn giá trị **một chỗ duy nhất** bằng macro (`lifecycle_statuses!`, chép khuôn `scope_kinds!`), cho cả hai tầng dùng chung. Tầng Chương lưu giá trị thật ở `chapter.status`; tầng Tác phẩm **không lưu giá trị suy ra** — chỉ lưu `work.status_override` (`NULL` = chưa ghi đè), còn giá trị suy ra tính bằng một hàm thuần từ tập trạng thái Chương. Hai kho dẫn xuất (`meta.json`, `library_work`) chở cặp `(status, status_is_override)` để Library đọc và **lọc** được mà không mở SQLite. Bề mặt tối thiểu ở Library: danh sách Tác phẩm kèm trạng thái + dấu ghi đè, bốn nút lọc, và ba lệnh vòng đời cho Tác phẩm đang mở.

## Boundaries & Constraints

**Always:**
- **Bốn giá trị khai MỘT CHỖ**, bằng `macro_rules! lifecycle_statuses!` ở `core/lifecycle/` — một khai báo sinh `enum` + `ALL` + `as_str` + `label_key` + `from_wire`. Không tồn tại cú pháp khai một giá trị mà không kèm khoá nhãn i18n. Cùng lý lẽ với `message_keys!`/`scope_kinds!`: một danh sách song song viết tay cho một test **xanh giả**.
- **Bảng suy ra, đúng bốn hàng, và nó KHÔNG BAO GIỜ sinh ra `paused`:** 0 Chương ⇒ `not_started`; mọi Chương `done` ⇒ `done`; mọi Chương `not_started` ⇒ `not_started`; **mọi ca còn lại** ⇒ `in_progress`. FR6 viết nguyên văn *"Tạm ngưng ở tầng Tác phẩm là quyết định của người, hệ thống không suy ra được"* — xem §Design Notes cho phương án bị loại.
- **Tầng Tác phẩm KHÔNG có cột `status`.** Nguồn sự thật là `chapter.status` + `work.status_override`; một cột `work.status` lưu sẵn là nguồn sự thật thứ hai sẽ trôi. `meta.json`/`library_work` được phép chở nó vì cả hai **tự khai là cache dẫn xuất** (AD-33, AD-8).
- **Ghi đè là `NULL`-hoặc-giá-trị, không phải một cờ boolean riêng.** `status_override IS NULL` ⇒ đang suy ra; có giá trị ⇒ giữ nguyên giá trị đó bất kể Chương đổi thế nào, cho tới khi người dùng bỏ ghi đè.
- `chapter.status`/`work.status_override` **cưỡng chế giá trị hợp lệ ở tầng Rust**, không bằng `CHECK` SQL — đúng khuôn `chapter.status`/`segment.status`/`config_value.kind` đã có (xem `schema.rs:774,899,1000`). Một giá trị lạ qua dây ⇒ `IpcError` có tên, **không** ghi gì.
- **Một `meta.json` không khai trạng thái là CHƯA BIẾT, không phải *Chưa bắt đầu*.** `meta.json` cũ (`meta_schema_version = 1`) đọc ra `status = None`; hàng chỉ mục mang `status IS NULL`, màn hình nói *chưa biết*, và nó **không lọt vào bất kỳ bộ lọc nào trong bốn**. Đây là luật trung tâm của kho (*"rỗng im lặng bị cấm; rỗng có lý do thì không"*).
- **Một danh sách rỗng phải nói vì sao rỗng.** Màn hình luôn có cả `worksHaveLoaded` (đã đọc lần nào chưa) lẫn **hai** con số — tổng số hàng trong chỉ mục và số hàng sau lọc — để phân biệt *"chưa quét"* · *"Library trống thật"* · *"bộ lọc quét sạch"*. Đây đúng chỗ Story 3.9 đã hụt (`AGENTS.md::Known pitfalls`: chỗ gọi bịa `totalCount` bằng chính `filteredCount`).
- **Chỉ `Indexer` ghi `library-index.db`.** Sau mỗi lượt ghi trạng thái, vỏ IPC gọi lại đúng hàm reindex đã có (`commands::project::reindex_after_create_work`), không tự `UPDATE library_work`.
- Khuôn hai lớp cho mọi vỏ IPC mới: hàm thuần nhận `Option<&OpenWork>`/`Option<&mut OpenWork>` (thứ `tests/**` gọi được không cần webview) + `#[tauri::command]` mỏng trong `mod wire` dùng **`try_state`**, không `state()`.
- `meta.json` ghi **NGAY SAU** khi giao dịch `project.db` commit, ở tầng thao tác — không bao giờ bên trong closure `Store::write` (Quyết định #3 của Story 1.15).
- Mọi `@click` mới là **đúng một** `dispatch('<id>')`; mọi nhãn qua `t()`; màu và cỡ chữ chỉ từ token.

**Block If:**
- Bảng suy ra cần một hàng thứ năm mà FR5/FR6 không phân xử được (ví dụ: một trạng thái *hỗn hợp* riêng) ⇒ HALT, đây là quyết định của Ice.
- Việc đóng AC5/AC6 đòi mở lại một `.atproj` đã có trên đĩa ⇒ HALT: đó là món nợ kiến trúc trung tâm của Epic 5, có chủ riêng (5.6/5.7), không phải một dòng mã tiện tay ở đây.

**Never:**
- **Không** thêm cột `status` vào `work`, và **không** sửa `WORK_DDL` tại chỗ — cột mới đi bằng một **bước di trú MỚI** của `PROJECT_MIGRATIONS` (vết sẹo số 4).
- **Không** thêm bước 2 cho `LIBRARY_INDEX_MIGRATIONS` — kho dẫn xuất viết lại `LIBRARY_WORK_DDL` **tại chỗ** và bump `to_version` (3 → 4).
- **Không** dựng lưới Tác phẩm của Story 5.6: không bìa, không thanh tiến độ (5.5), không sắp xếp, không lọc theo thể loại/ngôn ngữ, không điều hướng lưới bằng bàn phím. Story này chỉ thêm một **danh sách phẳng tối thiểu** — cùng chủ ý *"màn hình TỐI THIỂU CÓ CHỦ"* mà Story 5.3 đã đặt.
- **Không** tự suy trạng thái Chương từ trạng thái segment. FR5 không nói thế; một lượt suy ra như vậy làm người dùng mất quyền nói *"chương này tôi tạm ngưng"*.
- **Không** đúc ký hiệu mới cho dấu "ghi đè thủ công" — dấu phân biệt viết thành **chữ** qua `t()`.
- `commands/lifecycle.rs` và `commands/library.rs` **không** được nhắc `StoreKind::LibraryIndex`/`StoreSpec::library_index` ở vị trí mã (`library_index_boundary.rs`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Chương mới nhập | `create_work_from_text(...)` | `chapter.status = "not_started"`; `work.status_override IS NULL`; `meta.json` mang `status = "not_started"`, `status_is_override = false` | No error expected |
| Đổi trạng thái Chương | Tác phẩm một Chương, `set_chapter_status(id, "done")` | Chương thành `done`; đọc lại tầng Tác phẩm ra `done` với `is_override = false`; `meta.json` ghi lại; chỉ mục cập nhật sau lượt reindex | No error expected |
| Trạng thái Chương hỗn hợp | 3 Chương: `done` · `not_started` · `paused` | Tầng Tác phẩm suy ra `in_progress` — **không** `paused` | No error expected |
| Giá trị ngoài danh mục (Chương) | `set_chapter_status(id, "finished")` | Không một lượt ghi nào chạy | `err.lifecycle.unknown_status` `{status}` |
| `chapter_id` không tồn tại | `set_chapter_status(999, "done")` | Không một lượt ghi nào chạy | `segment.chapter_not_found` `{chapter_id}` (**tái dùng**, không đúc khoá thứ hai) |
| Ghi đè thủ công | Suy ra đang là `in_progress`, `set_work_status_override(Some("paused"))` | `work.status_override = 'paused'`; đọc ra `status = "paused"`, `is_override = true` | No error expected |
| Chương đổi SAU khi đã ghi đè | Đang ghi đè `paused`, rồi mọi Chương thành `done` | Tầng Tác phẩm **vẫn** `paused`, `is_override = true` — hệ thống không suy ra đè lên | No error expected |
| Bỏ ghi đè | `set_work_status_override(None)` | `status_override` về `NULL`; đọc ra giá trị **suy ra hiện thời**, `is_override = false` | No error expected |
| Giá trị ngoài danh mục (Tác phẩm) | `set_work_status_override(Some("archived"))` | Không một lượt ghi nào chạy | `err.lifecycle.unknown_status` `{status}` |
| Chưa Tác phẩm nào mở | Bất kỳ lệnh vòng đời nào, `OpenWorkState = None` | Không một lượt ghi nào chạy | `work.none_open` (**tái dùng** `no_work_open()`) |
| Lọc một giá trị | Chỉ mục 4 hàng, `list_works(filter = ["paused"])` | Chỉ hàng `status = 'paused'`; báo cáo mang **cả** `total = 4` lẫn `matched = 1` | No error expected |
| Bộ lọc quét sạch | Chỉ mục 3 hàng `not_started`, lọc `["done"]` | Danh sách rỗng **kèm** `total = 3` ⇒ màn hình nói *bộ lọc không khớp hàng nào*, **không** nói *Library trống* | No error expected |
| Không lọc | `list_works(filter = None)` | Mọi hàng, kể cả hàng `status IS NULL`; `matched == total` | No error expected |
| `meta.json` viết trước vòng đời | `.atproj` có `meta_schema_version = 1` | Hàng vào chỉ mục với `status IS NULL`, `status_is_override = 0`; màn hình hiện *chưa biết*; hàng **không** khớp bất kỳ bộ lọc nào trong bốn | No error expected |
| `meta.json` mới hơn ứng dụng | `meta_schema_version = 3` | Không ghi gì; mục vào `SkippedEntry` với lý do chẩn đoán | `MetaError::SchemaTooNew` (đường đã có) |
| Chưa đọc lần nào | Mở Library, chưa gọi `library.list_works` | `worksHaveLoaded = false` ⇒ màn hình nói *chưa biết*, **không** nói *Library trống* | No error expected |
| `Indexer` chưa quản lý | `library_list_works` khi mở `library-index.db` thất bại | Không dữ liệu giả | `library.indexer_missing` (**tái dùng**) |

</intent-contract>

## Code Map

- `src-tauri/src/core/scope/kinds.rs:85-145` -- **khuôn để chép** cho `lifecycle_statuses!`: `macro_rules!` nhận `$(#[$meta:meta])*` (bắt buộc — doc-comment nở thành `#[doc]`), sinh `enum` + `ALL` + `as_str()` + `semantics()` + `from_wire()`; `:107-113` -- lý do `ALL` được test đối chiếu với **một hằng viết tay** (*"con số viết tay là chỗ một con người phải ký"*); `:137-144` -- lý do nhánh `_ => None` của `from_wire` **không** vi phạm luật cấm `_ =>`.
- `src-tauri/src/core/store/schema.rs:699-709` -- `WORK_DDL` (7 cột, `CHECK (id = 1)`); `:695-698` -- ghi chú `source_lang` bất biến cưỡng chế ở tầng ứng dụng, không bằng SQL. `:735-745` -- `CHAPTER_DDL`, cột `status TEXT NOT NULL` **đã có từ Story 1.15** kèm câu *"cưỡng chế giá trị hợp lệ là việc của tầng Rust"*. `:1369-1461` -- `PROJECT_MIGRATIONS`, bước cuối là **15** ⇒ bước mới là **16**; đọc bảng này chứ đừng đọc một ghi chép ở nơi khác. `:1545-1567` -- `LIBRARY_WORK_DDL` (8 cột) — doc-comment `:1533-1537` khai thẳng *"**không** cột trạng thái vòng đời (chủ Story 5.4)"*, tức dòng đó phải được sửa 🔵 tại chỗ ở story này. `:1569-1590` -- `LIBRARY_INDEX_MIGRATIONS` hiện `to_version: 3`, và `:1467-1480` -- lý lẽ vì sao **viết lại tại chỗ** là ĐÚNG cho kho này và SAI cho mọi kho khác.
- `src-tauri/src/core/library/meta.rs:33` -- `META_SCHEMA_VERSION = 1`; `:73-92` -- `WorkMeta` 8 trường, `#[serde(rename_all)]` **cấm**; `:105-125` -- `WorkMeta::read` (chỉ từ chối bản **mới hơn**, nên `meta.json` v1 vẫn đọc được sau khi bump); `:182-225` -- `rebuild_from_store` (đọc hàng `work` + `COUNT(*) FROM chapter` qua `Store::read`) -- **chỗ tính giá trị suy ra**; `:126-180` -- `write_atomic` (tmp → `sync_all` → `rename` → fsync thư mục cha).
- `src-tauri/src/commands/project.rs:35` -- `CHAPTER_STATUS_NOT_STARTED` (hằng cục bộ, doc-comment tự khai là **tạm**) ⇒ thay bằng danh mục đóng. `:266-294` -- giao dịch tạo Tác phẩm (`INSERT INTO work` + `INSERT INTO chapter` + `insert_segments`, cùng một `store.write`). `:306-340` -- **khuôn chuẩn** cho mọi lượt ghi trạng thái: `rebuild_from_store` → `write_atomic` **sau** commit, cộng lý lẽ *"lỗi ghi meta.json phải nói ra, không được nuốt"*. `:794` -- `pub type OpenWorkState = std::sync::Mutex<Option<OpenWork>>`. `:45-60` -- `OpenWork` (mọi trường `pub`: `dir`/`store`/`scope`/`meta`/`chapter_id`). `:1719-1760` -- `reindex_after_create_work(app, root)`, chỗ gọi `rebuild` thứ hai — story này thành chỗ gọi thứ ba, nên **tên và doc-comment của nó phải được sửa 🔵 tại chỗ**. `:106-127` -- `default_library_root`; `resolve_library_root` (Story 5.3) là bộ phân giải phải dùng.
- `src-tauri/src/commands/chapter.rs:64-71` -- `no_work_open()` (`work.none_open`, `pub(crate)`) và `:86-93` -- `chapter_not_found(chapter_id)` (**tái dùng `MessageKey::SegmentChapterNotFound`, không đúc khoá thứ hai** — lý lẽ viết sẵn tại chỗ); `:103-140` -- `read_open_chapter` (khuôn hàm thuần nhận `Option<&OpenWork>`); `:304-350` -- `mod wire` với `try_state::<OpenWorkState>()`.
- `src-tauri/src/commands/library.rs:44-115` -- `OrphanEntry`/`ConflictEntry`/`RescanReport` (**khuôn wire struct**: `snake_case`, không `rename_all`); `:117-140` -- `indexer_is_missing()`/`root_invalid()`/`store_is_missing()` (dựng `IpcError` qua `IpcError::new`, và ca kho vắng đi qua `From<StoreError>`); `:152-190` -- `rescan` (hàm thuần); `:264-350` -- `mod wire`, cả ba vỏ `#[tauri::command(async)]`.
- `src-tauri/src/core/library/indexer.rs:276-320` -- câu UPSERT 8 cột (`ON CONFLICT (work_id) DO UPDATE SET …`) — chỗ hai cột mới đi vào; `:440-472` -- `list_works()` (`SELECT … ORDER BY work_id`, **chưa có bộ lọc, chưa có chỗ gọi sản phẩm nào** — chỉ tests gọi); `:788-800` -- `IndexedWork`; `:671-760` -- `RebuildOutcome` + `log_if_notable(surface)` (đường chẩn đoán CHUNG); `:805-855` -- `IndexError` + `From<IndexError> for IpcError`.
- `src-tauri/src/core/i18n/mod.rs:62-90` -- `message_keys!`; `:395-416` -- cụm khoá Library của Story 5.3 (`LibraryNotOrphaned` `["work_id","name"]`, `LibraryRootInvalid` `[]`) kèm lý lẽ *"tái dùng khoá chung thay vì đúc khoá thứ ba"* — khoá mới khai **cạnh** cụm này.
- `src-tauri/src/lib.rs:324-330` -- `generate_handler![...]`; `:699-745` -- `open_library_index` (chỗ gọi `rebuild` thứ nhất); `:666` -- `app.manage(OpenWorkState::new(None))`.
- `src-tauri/tests/ipc_contract.rs:395-450` -- `library_wire_structs_keep_snake_case_field_names` (khuôn đóng băng tên trường trên dây); `:232-380` -- ba ca chạy **trên `ALL`** của `MessageKey` (đồng bộ `vi.json` + bảng tham số).
- `src-tauri/tests/library_index_contract.rs` -- 19+ ca của 5.2/5.3, gồm ca so **byte** `.atproj` trước/sau; mọi ca dựng `WorkMeta` bằng struct literal sẽ **không biên dịch** sau khi thêm trường ⇒ sửa kèm một dòng nói vì sao, không xoá.
- `src-tauri/tests/library_commands_contract.rs:120-175` -- khuôn ca đi qua **tầng lệnh**; `src-tauri/tests/glossary_commands_contract.rs:63-72` -- `open_work(root, tag)` = `create_work_from_text(...)`, **chỗ sản phẩm duy nhất dựng một `OpenWork` thật** ⇒ đây là cách `lifecycle_contract.rs` có một `OpenWork` để gọi hàm thuần.
- `src-tauri/tests/library_index_boundary.rs:33-46` -- `EXEMPT_FILES` (2 tệp) · `FORBIDDEN` (2 chuỗi) · `RS_FLOOR = 44`; `src-tauri/tests/naming_boundary.rs:82-106` -- `FORBIDDEN_WORDS` (`Project`/`Book`/`Novel`/`Document`) · `STORE_EXEMPT` (8 mục) · `RUST_FLOOR 44` / `FRONTEND_FLOOR 58` (sàn là **cận dưới** — thêm tệp không làm đỏ).
- `src-tauri/tests/config_invariants.rs:887-960` -- `the_blocking_wires_run_off_the_main_thread`, nay là danh sách `(tệp, chữ ký, vì sao)` ⇒ vỏ mới nào chặn phải vào đây.
- `src/config/library.ts:17-60` -- khuôn type `snake_case` cho chiều TRẢ VỀ + `:78-...` type guard **lúc chạy**; `:60-80` -- ba hình dạng kết quả ba trạng thái.
- `src/modes/libraryRescan.ts:19-80` -- **khuôn state module-level**: mỗi khai báo trên MỘT dòng (`check:panel-refs` Kiểm 5), `readonly(...)` export, `sequence` chặn round-trip IPC đua nhau, `libraryScanHasLoaded` (vị từ `…HasLoaded`), `.at(cursor)` không `[cursor]`.
- `src/modes/LibraryMode.vue:78-150` -- khối "thư mục gốc + quét lại + mồ côi" của 5.3 (khuôn `role="status"` LUÔN có mặt, `aura-allow-text` cho dữ liệu không phải câu UI); `:170-200` -- CSS chỉ dùng token.
- `src/commands/index.ts:200-230` -- `CommandDeps` (khuôn `deps.<port>` tiêm vào); `:856-916` -- năm lệnh `library.*` của 5.3 + lý lẽ chọn hợp âm (`Mod+Alt+K` đã dùng; đo trước khi cấp phím mới). `src/main.ts:52-66,315-327` -- nơi nối port.
- `src/i18n/vi.json:44-45` -- `err.library.*`; `:57-63` -- `command.library.*`; `:131-160` -- `mode.library.*`.
- `e2e/specs/story-5-3-rescan.e2e.mjs` -- khuôn spec e2e gần nhất (6 ca, `realClick()` bắt buộc); `e2e/support/pointer.mjs` -- `realClick`.
- `_bmad-output/implementation-artifacts/deferred-work.md` -- nợ *"trùng `work_id` chưa có bề mặt hiển thị"* và *"đường ĐỌC thuần thay cho lượt quét lúc mở Library"*, cả hai **chủ Story 5.6** ⇒ story này **nối tiếp**, không viết đè.
- `src-tauri/AGENTS.md:29` -- khối 🔵 hai lớp về `library-index.db`/`meta.json` dẫn xuất; phải nối tiếp sau khi bảng chỉ mục đổi hình dạng.

## Tasks & Acceptance

**Execution:**
- `src-tauri/src/core/lifecycle/mod.rs` -- **tệp mới**: `macro_rules! lifecycle_statuses!` (chép khuôn `scope_kinds!`) khai bốn giá trị `NotStarted => "not_started" : "lifecycle.not_started"` … sinh `enum LifecycleStatus` + `ALL` + `as_str` + `label_key` + `from_wire`; cộng hàm thuần `derive_work_status(chapters: &[LifecycleStatus]) -> LifecycleStatus` với bảng bốn hàng của §Always và ba ca `#[cfg(test)]` tại chỗ -- một danh sách bốn giá trị viết tay ở nơi thứ hai là đúng thứ `message_keys!` tồn tại để chặn.
- `src-tauri/src/core/mod.rs` -- khai `pub mod lifecycle;` -- module đặt theo **khái niệm miền**, không theo nhóm năng lực.
- `src-tauri/src/core/store/schema.rs` -- (a) thêm hằng `WORK_STATUS_OVERRIDE_DDL` (`ALTER TABLE work ADD COLUMN status_override TEXT`) làm **bước 16 MỚI** của `PROJECT_MIGRATIONS`, kèm doc-comment nói vì sao **không** sửa `WORK_DDL` tại chỗ (vết sẹo số 4) và vì sao **không** có `CHECK`; (b) **viết lại TẠI CHỖ** `LIBRARY_WORK_DDL` thêm `status TEXT` (cho phép `NULL`) + `status_is_override INTEGER NOT NULL DEFAULT 0`, bump `LIBRARY_INDEX_MIGRATIONS` `to_version` 3 → 4, và sửa 🔵 dòng doc-comment đang viết *"không cột trạng thái vòng đời (chủ Story 5.4)"* -- hai kho, hai luật ngược nhau, cả hai đã viết sẵn lý lẽ tại chỗ.
- `src-tauri/src/core/library/meta.rs` -- `WorkMeta` thêm `status: Option<String>` + `status_is_override: bool`, **cả hai `#[serde(default)]`**; `META_SCHEMA_VERSION` 1 → 2; `rebuild_from_store` đọc `work.status_override` và `SELECT status FROM chapter` rồi gọi `derive_work_status` -- `Option` chứ không giá trị mặc định vì một `meta.json` v1 **chưa biết** trạng thái, và khẳng định *Chưa bắt đầu* cho một Tác phẩm đã xong là đúng lớp "rỗng im lặng".
- `src-tauri/src/commands/project.rs` -- xoá hằng cục bộ `CHAPTER_STATUS_NOT_STARTED`, dùng `LifecycleStatus::NotStarted.as_str()` ở câu `INSERT INTO chapter`; đổi tên `reindex_after_create_work` → `reindex_library(app, root)` và sửa 🔵 doc-comment cho nó nói đúng số chỗ gọi -- một hằng "tạm" và một cái tên khai sai số chỗ gọi đều là mệnh đề sẽ lặng lẽ sai.
- `src-tauri/src/commands/lifecycle.rs` -- **tệp mới**, khuôn hai lớp: hàm thuần `read_work_lifecycle(open: Option<&OpenWork>)` · `set_chapter_status(open: Option<&mut OpenWork>, chapter_id, status: &str)` · `set_work_status_override(open: Option<&mut OpenWork>, status: Option<&str>)`, mỗi lượt ghi = một `store.write` rồi `rebuild_from_store` + `write_atomic` + cập nhật `open.meta` (khuôn `project.rs:306-340`); `mod wire` ba vỏ dùng `try_state`, lỗi ghi `meta.json` **nói ra**, không nuốt -- test gọi được ba hàm này không cần webview, đó là lý do khuôn hai lớp tồn tại.
- `src-tauri/src/commands/mod.rs` + `src-tauri/src/lib.rs` -- khai `pub mod lifecycle;`, thêm bốn vỏ mới (`read_work_lifecycle` · `set_chapter_status` · `set_work_status_override` · `library_list_works`) vào `generate_handler![...]` -- tên command trên dây LÀ tên hàm, nên vỏ sống trong `mod wire` và không mang hậu tố.
- `src-tauri/src/core/library/indexer.rs` -- UPSERT chở thêm `status`/`status_is_override` (cả hai vế `INSERT` và `DO UPDATE SET`); `IndexedWork` thêm hai trường; `list_works(filter: Option<&[LifecycleStatus]>)` lọc **trong SQL** và trả kèm tổng số hàng chưa lọc -- bộ lọc tính ở Rust chứ không ở TypeScript (AD-1), và `total` đi cùng `matched` trong **một** lượt đọc để hai con số không bao giờ đến từ hai ảnh chụp khác nhau.
- `src-tauri/src/commands/library.rs` -- thêm `WorkRow` (wire struct `snake_case`) + `WorkListReport { total, matched, works }` + hàm thuần `list_works(indexer, filter: Option<&[String]>)` (giá trị lạ trong bộ lọc ⇒ `err.lifecycle.unknown_status`, không im lặng bỏ qua) + vỏ `library_list_works` -- một bộ lọc nuốt giá trị lạ là một màn hình khẳng định "không khớp gì" cho một câu hỏi nó chưa hiểu.
- `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` -- thêm **đúng một** `MessageKey`: `LifecycleUnknownStatus => "err.lifecycle.unknown_status" ["status"]`; thêm bốn khoá nhãn `lifecycle.*`, các khoá `command.lifecycle.*`/`command.library.list_works`/`command.library.filter_*` và `mode.library.works_*` -- danh mục đóng; ca "chưa Tác phẩm nào mở" và "chương không tồn tại" **tái dùng** khoá đã có, không đúc khoá thứ ba.
- `src-tauri/tests/lifecycle_contract.rs` -- **tệp mới**: phủ TRỌN §I/O Matrix ở tầng lệnh qua một `OpenWork` **thật** (`create_work_from_text`, khuôn `glossary_commands_contract.rs:69`), cộng ca chạy trên `ALL` (mỗi giá trị có `label_key` tồn tại trong `vi.json`), ca đối chiếu `ALL.len()` với một hằng **viết tay** `= 4`, và ca khẳng định bảng suy ra **không bao giờ** trả `paused` với mọi tổ hợp trạng thái Chương -- một hằng viết tay là chỗ một con người phải ký.
- `src-tauri/tests/library_index_contract.rs` + `library_commands_contract.rs` -- sửa mọi ca dựng `WorkMeta` bằng struct literal (nay 10 trường) **kèm một dòng nói vì sao**, không xoá ca nào; thêm ca lọc theo từng giá trị trong bốn, ca "bộ lọc quét sạch trả `matched = 0` mà `total > 0`", và ca "`meta.json` v1 vào chỉ mục với `status IS NULL` và không khớp bộ lọc nào" -- ca cuối là ca duy nhất chứng minh phương án `Option` khác phương án mặc-định-`not_started`.
- `src-tauri/tests/ipc_contract.rs` -- đóng băng tên trường `snake_case` của `WorkRow`/`WorkListReport` và của struct trả về của ba lệnh vòng đời -- một trường mới qua IPC mà không ai đối chiếu là đúng thứ ca này tồn tại để chặn.
- `src-tauri/tests/config_invariants.rs` -- thêm bốn vỏ mới vào danh sách `(tệp, chữ ký, vì sao)` của `the_blocking_wires_run_off_the_main_thread` nếu vỏ nào chặn (lượt reindex sau mỗi lượt ghi **là** một lượt quét đĩa) -- một vỏ chặn trên luồng chính là bế tắc đã ĐO ở Story 3.10b.
- `src/config/lifecycle.ts` -- **tệp mới**, adapter IPC thứ CHÍN: `invoke` + `try/catch` + hình dạng ba trạng thái + type guard **lúc chạy**; `src/config/library.ts` -- thêm `listLibraryWorks(filter)` cùng khuôn -- `camelCase` chiều gửi (`chapterId`, `statusOverride`), `snake_case` chiều trả về (`status_is_override`).
- `src/modes/libraryWorks.ts` -- **tệp mới**: `works` · `worksTotal` · `worksMatched` · `statusFilter` (tập bốn giá trị bật/tắt riêng rẽ) · `worksHaveLoaded` · `worksBusy` · `worksError`, cùng `loadWorks()`, `toggleStatusFilter(id)`, `clearStatusFilter()`; và state vòng đời của Tác phẩm đang mở (`openWorkStatus`, `openWorkIsOverride`, `openWorkLifecycleHasLoaded`) với `setOpenWorkOverride()`/`clearOpenWorkOverride()`/`setOpenChapterStatus()` -- mỗi khai báo trên MỘT dòng (`check:panel-refs` Kiểm 5), `sequence` chặn lượt cũ ghi đè lượt mới.
- `src/commands/index.ts` + `src/main.ts` -- đăng ký `library.list_works` · `library.filter_not_started` · `library.filter_in_progress` · `library.filter_paused` · `library.filter_done` · `library.filter_clear` · `lifecycle.set_work_override_paused` · `lifecycle.clear_work_override` · `lifecycle.set_chapter_done` qua `deps` tiêm vào kèm `portMissing`; cấp hợp âm mặc định **chỉ** cho `library.list_works` sau khi `grep` đếm họ `Mod+Alt+…` còn trống -- đăng ký ở `main.ts` chứ không trong `.vue`, và đo trước khi cấp phím.
- `src/modes/LibraryMode.vue` -- thêm khối "Tác phẩm" (danh sách phẳng: tên · nhãn trạng thái · dấu **chữ** cho ghi đè thủ công), hàng bốn nút lọc + nút bỏ lọc, dòng luôn nói `matched/total`, và khối vòng đời cho Tác phẩm đang mở; mọi `@click` là đúng một `dispatch('<id>')`, mọi nhãn qua `t()`, mọi node kết quả `role="status"` LUÔN có mặt, chỉ dùng token -- AD-34 §1, `check:tokens` Kiểm B/B2 và `check:i18n` Kiểm A2 đều đọc tĩnh tệp này.
- `tests/frontend/libraryWorks.test.ts` -- **tệp mới** (đuôi `.test.ts`, không `.spec.ts` — `vitest.config.ts` chỉ nạp `*.test.ts`): adapter không ném trên lỗi IPC; `worksHaveLoaded` sai trước lượt gọi đầu; bốn bộ lọc bật/tắt riêng rẽ; `matched = 0` với `total > 0` cho ra câu *bộ lọc không khớp* chứ không câu *Library trống*; hàng `status = null` hiện *chưa biết* và không khớp bộ lọc nào -- `happy-dom` chỉ được canh hành vi module thuần, không canh hình học.
- `e2e/specs/story-5-4-lifecycle.e2e.mjs` -- **tệp mới**: đi trọn đường nút thật → `dispatch` → registry → `invoke` → Rust → DOM cho ít nhất *ghi đè thủ công hiện dấu phân biệt* và *một bộ lọc lọc riêng rẽ*, dùng `realClick()` -- một bộ test xanh KHÔNG chứng minh chỗ nối được canh; Epic 3 dính năm lần trong bảy ngày.
- `src-tauri/AGENTS.md` + `_bmad-output/implementation-artifacts/deferred-work.md` + `_bmad-output/implementation-artifacts/sprint-status.yaml` -- nối tiếp khối 🔵 dòng 29 (bảng chỉ mục nay chở `status`/`status_is_override`, và một hàng `status IS NULL` nghĩa là *chưa biết*); ghi nợ **có chủ** cho: lượt reindex TOÀN BỘ sau mỗi lượt ghi trạng thái (chủ 5.6, cùng món nợ "đường ĐỌC thuần"), và bề mặt đổi trạng thái chỉ với Tác phẩm **đang mở** vì đường mở lại `.atproj` chưa tồn tại (chủ 5.6/5.7) -- không mục nào mồ côi, không mục cũ bị xoá.

**Acceptance Criteria:**
- Given bảng bốn giá trị, when gỡ hoặc thêm một hàng trong `lifecycle_statuses!`, then `cargo test --locked` **đỏ** ở ca đối chiếu `ALL.len()` với hằng viết tay và ở ca đồng bộ nhãn với `vi.json` — và không tệp nào ngoài `core/lifecycle/**` mang một danh sách bốn giá trị viết tay song song.
- Given một lượt ghi trạng thái Chương hoặc ghi đè Tác phẩm, when nó trả `Ok`, then `meta.json` trên đĩa đã mang giá trị mới **và** hàng `library_work` tương ứng đã mang giá trị mới — kiểm bằng cách đọc lại cả hai, không bằng cách tin lời gọi.
- Given người dùng bấm lần lượt từng nút trong bốn nút lọc ở Library, when đọc danh sách, then mỗi giá trị lọc riêng rẽ được, và màn hình luôn nói **cả** số hàng khớp **lẫn** tổng số hàng trong chỉ mục.
- Given một Tác phẩm đang ghi đè thủ công và một Tác phẩm mang cùng giá trị nhưng do suy ra, when cả hai hiện trong danh sách Library, then chúng **phân biệt được** bằng một dấu viết thành chữ, không chỉ bằng giá trị trạng thái.
- Given `check:commands` Kiểm A/B và `check:i18n` Kiểm A/A2, when chạy sau story này, then xanh trên mọi `@click` và mọi nhãn mới **mà không** thêm một miễn trừ nào không có tên.
- Given bộ test cũ (`library_index_contract` · `library_commands_contract` · `library_index_boundary` · `project_contract` · `store_contract` · `naming_boundary` · `scope_contract` · `segment_contract` · bộ frontend), when chạy sau story này, then xanh — trừ những ca mà việc thêm trường vào `WorkMeta`/`library_work` **cố ý** làm đổi nghĩa, và mỗi ca như vậy được sửa **kèm một dòng nói vì sao**, không bị xoá.
- Given đối chứng bắt buộc, when gỡ lượt `reindex_library` khỏi đường ghi trạng thái rồi chạy **bộ test CŨ**, then nó phải **ĐỎ** ở đúng ca "hàng `library_work` đã mang giá trị mới" — một chỗ nối không đỏ được khi bị gỡ là một chỗ nối chưa ai canh.

## Spec Change Log

- **Lượt nghiệm thu step-03 — đối chứng #1 của §Verification ĐỎ, và nó đỏ vì một lỗi thiết kế thật, không vì một ca test thiếu.**
  **Phát hiện:** bản dựng đầu đặt bước 4 (`reindex_after_lifecycle_write`) **bên trong** `mod wire` của `commands/lifecycle.rs`. Gỡ hẳn hai lời gọi đó rồi chạy `cargo test --locked` cho **0 failed trên toàn bộ 34 binary** (đo 2026-08-27) — tức chỗ nối giữa *"đã ghi trạng thái"* và *"hàng `library_work` đã mang giá trị mới"* **chưa có ai canh**, đúng lớp lỗi mà `AGENTS.md::Known pitfalls` gọi tên. Nguyên nhân gốc: `tests/**` không dựng được `tauri::AppHandle`, nên mọi thứ sống trong `mod wire` chỉ **so chuỗi** được, không **chạy** được — và `src-tauri/AGENTS.md` đã viết sẵn luật bị phá ở đây: *"Một vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**"*
  **Đã sửa (không hoàn nguyên, không hạ kỳ vọng):** quy tắc *"ghi thành công thì đưa vào chỉ mục"* chuyển xuống tầng hàm thuần — `commands::lifecycle::reindex_after_lifecycle_write(Option<&Indexer>, Option<&Store>, &Path)` cộng hai bọc `set_chapter_status_indexed`/`set_work_status_override_indexed`, đúng khuôn `commands::library::rescan` đã có. `mod wire` giữ vị từ `is_ok()` để tách **hai vùng khoá `OpenWorkState` ngắn** (khoá không được giữ qua một lượt quét đĩa) rồi gọi xuống chính hàm thuần đó — không dựng đường thứ hai.
  **Đối chứng ĐÃ CHẠY lại sau khi sửa:** gỡ khối `if result.is_ok() { reindex_after_lifecycle_write(...) }` khỏi hai hàm `_indexed` ⇒ **2 ca đỏ** (`setting_a_chapter_status_leaves_the_new_value_in_the_library_index`, `overriding_the_work_status_leaves_both_the_value_and_the_override_flag_in_the_index`), 13 ca kia vẫn xanh; đã khôi phục. Đối chứng #2 (bảng suy ra trả `Paused`) ⇒ đỏ ở `core::lifecycle::tests::no_combination_of_chapters_ever_derives_paused`. Đối chứng #3 (`status` mặc định `"not_started"` thay cho `Option`) ⇒ đỏ ở `a_true_v1_meta_json_indexes_with_status_null_and_matches_no_filter`. **Cả ba đối chứng nay đỏ đúng chỗ.**
  **KEEP — phải sống sót mọi lần dựng lại:** (1) bước 4 ở tầng **hàm thuần**, không ở `mod wire` — một lượt "gọn hơn" đưa nó ngược vào vỏ là xoá đúng hai ca canh nó; (2) hai ca chỉ mục phải khẳng định trạng thái **TRƯỚC** lượt ghi (một chỉ mục rỗng làm khẳng định sau đó đúng vì lý do sai); (3) `reindex_after_lifecycle_write` **không trả `Result`** — một lượt reindex trượt không được biến một lượt ghi đã thành công trên đĩa thành lỗi IPC (AD-8: `.atproj` là dữ liệu, chỉ mục là dẫn xuất).

## Review Triage Log

### 2026-08-28 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 9: (high 0, medium 5, low 4)
- defer: 4: (high 0, medium 2, low 2)
- reject: 4: (high 0, medium 1, low 3)
- addressed_findings:
  - `[medium]` `[patch]` `mod wire` **chép lại** vị từ bước 4 (`if result.is_ok() { reindex }`) thay vì dùng chung với hai hàm `*_indexed` mà hợp đồng canh ⇒ sản phẩm và phép kiểm chạy hai đường viết tay khác nhau. Rút vị từ thành `commands::lifecycle::finish_lifecycle_write` khai ĐÚNG MỘT CHỖ; cả `*_indexed` lẫn vỏ đều đi qua nó, và vỏ vẫn tách được hai vùng khoá `OpenWorkState` vì hàm nhận `Result` đã tính sẵn chứ không nhận closure.
  - `[medium]` `[patch]` `commands::library::list_works` (tầng LỆNH) không ca nào gọi — `matched` có thể trôi thành `total` mà mọi test vẫn xanh, đúng lỗi Story 3.9 mà doc-comment của chính hàm đó nêu tên. Thêm hai ca ở `library_commands_contract.rs`: giá trị lọc lạ bị TỪ CHỐI (`err.lifecycle.unknown_status`) kèm đối chứng giá trị hợp lệ vẫn đi qua, và `matched` khác `total` khi lọc.
  - `[medium]` `[patch]` `WorkMeta::rebuild_from_store` dùng `filter_map` **bỏ qua im lặng** mọi hàng `chapter.status` không phân giải được; mọi hàng hỏng ⇒ `derive_work_status(&[])` ⇒ `NotStarted`, tức hỏng dữ liệu đội lốt *Chưa bắt đầu*. Thêm chẩn đoán nêu đích danh từng hàng hỏng, cộng một dòng riêng khi KHÔNG hàng nào đọc được.
  - `[medium]` `[patch]` `loadWorks()` **nuốt** một lượt tải lại khi đang bận ⇒ bấm nút lọc thứ hai trong lúc lượt đầu còn bay để danh sách đứng ở bộ lọc CŨ trong khi nút đã hiện bộ lọc MỚI. Thay bằng `worksReloadPending` chạy lại ở MỌI đường ra; đối chứng đã chạy (đổi lại thành `return` trần ⇒ ca mới đỏ). Lượt vá này còn làm `check:panel-refs` đỏ vì ô nhớ mới chưa đi qua `resetLibraryWorks` — sửa nguồn, không thêm miễn trừ.
  - `[medium]` `[patch]` Hai câu màn hình khẳng định điều chúng chưa biết: `!openWorkLifecycleLoaded` LUÔN nói *"chưa có Tác phẩm nào đang mở"* kể cả khi lượt đọc trượt vì lỗi khác; và một `status` NGOÀI danh mục bốn giá trị hiện y hệt `status = null`. Tách thành `open_work_lifecycle_unreadable` và `works_status_invalid` — hỏng dữ liệu không được đọc lên giống chưa-di-trú.
  - `[low]` `[patch]` Không ca nào phủ bộ lọc HAI-trên-bốn (tổ hợp *"chưa xong"*, thứ người dùng bấm nhiều nhất); một `IN (...)` dựng sai với đúng hai tham số lọt qua cả ca một-giá-trị lẫn ca cả-bốn. Thêm `a_two_of_four_filter_returns_exactly_the_two_matching_rows`.
  - `[low]` `[patch]` `Indexer::list_works(Some(&[]))` nghĩa *"khớp 0 hàng"* trong khi doc-comment tầng lệnh nói bộ lọc rỗng là *"không lọc"* — một chỗ gọi Rust tương lai đọc doc tầng trên rồi gọi thẳng xuống sẽ nhận kết quả NGƯỢC, im lặng. Ghi rõ sự lệch có chủ ý này tại chỗ khai.
  - `[low]` `[patch]` Hai hàm cùng tên `reindex_after_lifecycle_write` khác chữ ký (một ở `mod wire`, một ở module cha) làm `grep`/"go to definition" nhập nhằng. Đổi tên bản trong vỏ thành `finish_with_reindex`.
  - `[low]` `[patch]` Sổ nợ không có mục nào cho ba chuyển đổi trạng thái Chương còn thiếu lối vào — ghi vào `deferred` kèm chủ (Story 5.7), đúng luật "không mục nào mồ côi".

## Design Notes

**Vì sao bảng suy ra không bao giờ sinh ra `paused` — và phương án bị loại đã bị loại bằng gì.**
Phương án ngược đã cân: *"không Chương nào `in_progress` mà có ít nhất một Chương `paused` ⇒ Tác phẩm `paused`"*. Nó **loại**, bằng hai vế đo được, không bằng khẩu vị: (1) FR6 viết nguyên văn *"Tạm ngưng ở tầng Tác phẩm là quyết định của người, hệ thống không suy ra được"* — một suy ra như vậy là mã nói ngược tài liệu; (2) nó làm AC5/AC6 **không nghiệm thu được**: nếu `paused` có thể đến từ cả hai đường, thì *"phân biệt ghi đè thủ công với suy ra tự động"* không còn quan sát được từ giá trị, và cả bài kiểm lẫn màn hình đều phải dựa vào một cờ mà chính bảng suy ra vừa làm nhoè. Với bảng đã chọn, `paused` ở tầng Tác phẩm là **bằng chứng tự thân** của một quyết định con người.

**Vì sao KHÔNG có cột `work.status`.**
Ba mảnh có thể lưu giá trị suy ra: `work`, `meta.json`, `library_work`. Hai mảnh sau **tự khai là cache dẫn xuất** (AD-33, AD-8) và có đường dựng lại (`rebuild_from_store`, `Indexer::rebuild`) — chúng được phép chở một bản chép. `work` thì không: nó là nguồn sự thật, và một cột suy ra ở đó tạo hai đường ghi cho cùng một mệnh đề (đổi `chapter.status` mà quên đổi `work.status`) mà **không cổng nào đỏ**. Đây đúng lớp lỗi mà AD-47 ghi lại cho `target_text`/cột xuất xứ.

**Vì sao `Option<String>` chứ không `#[serde(default)] = "not_started"`.**
`WorkMeta::read` chỉ từ chối bản **mới hơn**, nên sau khi bump `META_SCHEMA_VERSION` lên 2, mọi `meta.json` v1 vẫn đọc được — và với một giá trị mặc định, một Tác phẩm **đã dịch xong** sẽ hiện *Chưa bắt đầu* trong Library, im lặng, không lỗi nào. Đó chính là *"một truy vấn trả 0 hàng trong 0,01 ms không ném lỗi nào"* ở một hình dạng khác. `Option` bắt màn hình phải nói *chưa biết*, và bắt bộ lọc **không** được nhận vơ hàng đó về bất kỳ giá trị nào trong bốn. Giá phải trả: một trạng thái hiển thị thứ năm, đúng một khoá `vi.json`.

**Khuôn của một lượt ghi trạng thái — bốn bước, đúng thứ tự, chép từ `project.rs:266-340`.**
```
1. store.write(|tx| { UPDATE chapter SET status=?  /  UPDATE work SET status_override=? })   // giao dich commit
2. WorkMeta::rebuild_from_store(&open.store)      // doc lai tu nguon su that, khong dung so trong bo nho
3. meta.write_atomic(&open.dir)                   // NGOAI closure write -- Quyet dinh #3, Story 1.15
4. reindex_library(app, root)                     // CHI Indexer ghi library-index.db (AD-8)
```
Bước 2 đọc lại thay vì tự tính trong bộ nhớ có chủ ý: nó là bằng chứng chạy được của mệnh đề *"dẫn xuất"*, và nó bắt được ca một lượt ghi khác vừa chen vào giữa. Bước 3 trượt ⇒ **nói ra**, không nuốt (lý lẽ viết sẵn ở `project.rs:317-332`). Bước 4 là một lượt quét **toàn bộ** thư mục gốc — biết là đắt, ghi thành nợ có chủ thay vì tự dựng một đường `UPDATE` thứ hai vào chỉ mục.

**Phạm vi bề mặt, và ràng buộc thật đứng sau nó.**
Ba lệnh vòng đời chỉ chạy trên **Tác phẩm đang mở** (`OpenWorkState`), vì hôm nay đó là `project.db` **duy nhất** ứng dụng mở được — đường mở lại một `.atproj` đã có trên đĩa chưa tồn tại và là món nợ kiến trúc trung tâm của Epic 5. Đây là một **giới hạn thật, ghi ra thay vì để người sau tự phát hiện**, không phải một thiếu sót của story: danh sách Library đọc trạng thái của **mọi** Tác phẩm (qua chỉ mục), chỉ việc **đổi** trạng thái mới bị giới hạn ở Tác phẩm đang mở.

## Verification

**Commands:**
- `npm run check:i18n` -- expected: exit 0; mọi khoá mới có mặt trong `vi.json`, không chữ tiếng Việt có dấu ở vị trí mã trong `src-tauri/src/**`.
- `npm run check:commands` -- expected: exit 0; Kiểm A xanh trên mọi `@click` mới (đúng một `dispatch('<id>')`), Kiểm B thấy mọi id mới đăng ký ở `main.ts`.
- `npm run check:tokens` -- expected: exit 0; không màu/cỡ chữ viết thẳng trong khối mới của `LibraryMode.vue`.
- `npm run check:panel-refs` && `npm run check:layout` && `npm run check:debt-owner` && `npm run check:gates` -- expected: exit 0 cả bốn; `check:debt-owner` đòi mọi mục nợ mới có `Chủ:`.
- `npm run check:lint` -- expected: exit 0; không `eslint-disable` mới nào không có tên và lý do tại chỗ.
- `npm run test` -- expected: 45 tệp / (573 + số ca mới), không tệp nào bị bỏ qua.
- `npm run build` -- expected: exit 0 (chạy **trước** `cargo test`; thiếu `dist/` thì `cargo test` gãy ở khâu biên dịch).
- `cargo test --locked --manifest-path src-tauri/Cargo.toml` -- expected: exit 0, gồm `lifecycle_contract.rs` và các ca đã sửa của `library_index_contract.rs`.
- `grep -rn "not_started\|in_progress\|\"paused\"\|\"done\"" src-tauri/src --include=*.rs` -- expected: mọi lần xuất hiện ở vị trí **mã** nằm trong `core/lifecycle/mod.rs`; chỗ khác chỉ được nhắc trong comment hoặc qua `LifecycleStatus::…`.

**Đối chứng bắt buộc (chạy tay, rồi khôi phục — không đủ khi chỉ chạy bộ test):**
- Gỡ `reindex_library(...)` khỏi đường ghi trạng thái ⇒ chạy **bộ test CŨ** ⇒ phải ĐỎ ở ca "hàng `library_work` đã mang giá trị mới". Xanh ⇒ chỗ nối chưa được canh, và bộ test đang nói dối.
- Đổi bảng suy ra cho nó trả `paused` ở một tổ hợp bất kỳ ⇒ phải ĐỎ ở ca "không bao giờ suy ra `paused`".
- Đổi `status: Option<String>` thành `String` với `#[serde(default)]` ⇒ phải ĐỎ ở ca "`meta.json` v1 vào chỉ mục với `status IS NULL`".

**Manual checks (không có CLI):**
- `npm run test:e2e -- --spec e2e/specs/story-5-4-lifecycle.e2e.mjs` chạy trên WKWebView thật (ngoài `pre-push` và ngoài `push`, có ở nhịp đêm) -- expected: cả hai ca xanh, và một lượt đỏ được đọc theo `e2e/AGENTS.md` (cầu IPC trước, sản phẩm sau).
- Mở ứng dụng, tạo một Tác phẩm, bấm **Quét lại**: hàng mới hiện với nhãn *Chưa bắt đầu* và **không** dấu ghi đè. Bấm *Tạm ngưng Tác phẩm này*: hàng đổi nhãn và **có** dấu ghi đè. Đặt Chương thành *Đã xong*: hàng **vẫn** *Tạm ngưng*. Bấm *Bỏ ghi đè*: hàng thành *Đã xong*, dấu ghi đè biến mất.
- Bật một bộ lọc không khớp hàng nào: màn hình phải nói *bộ lọc không khớp hàng nào trên N Tác phẩm*, **không** nói *Library chưa có Tác phẩm nào*.

## Auto Run Result

Status: done
Blocking condition: (không có)

### Điều đã dựng

FR5/FR6 nay có một bề mặt thật ở cả hai tầng. `core::lifecycle::LifecycleStatus` khai bốn giá
trị MỘT CHỖ (`lifecycle_statuses!`, chép khuôn `scope_kinds!`) cộng `derive_work_status` — bảng
suy ra bốn hàng, không bao giờ trả `paused`. Tầng Chương lưu giá trị thật ở `chapter.status`;
tầng Tác phẩm chỉ lưu `work.status_override` (`NULL`-hoặc-giá-trị, bước di trú **16** mới của
`PROJECT_MIGRATIONS`), giá trị hiển thị luôn được TÍNH LẠI bởi `WorkMeta::rebuild_from_store` —
không cột `work.status` nào lưu sẵn. Cả `meta.json` (bump `META_SCHEMA_VERSION` 1→2,
`status`/`status_is_override` đều `Option`/`#[serde(default)]` để một tệp v1 đọc ra CHƯA BIẾT,
không phải "Chưa bắt đầu") lẫn `library_work` (viết lại TẠI CHỖ, bump `LIBRARY_INDEX_MIGRATIONS`
3→4) đều chở cặp đó cho Library lọc mà không mở SQLite của từng Tác phẩm.

Ba lệnh vòng đời (`read_work_lifecycle` đọc thuần; `set_chapter_status`/
`set_work_status_override` ghi, cả hai `(async)` vì bước 4 của khuôn bốn bước —
`reindex_library`, đổi tên từ `reindex_after_create_work` — là một lượt quét TOÀN BỘ thư mục
gốc) sống ở `commands::lifecycle`, khuôn hai lớp. `Indexer::list_works` nhận thêm bộ lọc
`Option<&[LifecycleStatus]>`, tính trong SQL, trả `total` VÀ `works` (đã lọc) trong MỘT lượt
đọc — `commands::library::list_works`/`WorkListReport` thêm tường minh trường `matched` ở tầng
dây. `LibraryMode.vue` thêm khối "Tác phẩm" (danh sách phẳng + bốn nút lọc riêng rẽ + nút bỏ
lọc, tự tải khi vào Library — không cần bấm Quét lại trước) và khối "Trạng thái Tác phẩm đang
mở" (ba nút, dấu ghi đè viết thành chữ, không ký hiệu đúc mới).

### Đối chứng bắt buộc — cả ba đã chạy tay, xác nhận ĐỎ đúng chỗ, rồi khôi phục

1. Gỡ hai lời gọi `reindex_after_lifecycle_write` khỏi `commands::lifecycle::wire` ⇒ chạy
   `cargo test --locked` toàn bộ ⇒ **XANH, 0 đỏ** (34 test binary, exit 0) — **không** đỏ ở ca
   nào. Đây KHÔNG phải "cổng chưa canh chỗ nối bị bỏ sót một cách bất ngờ": nó xác nhận đúng
   giới hạn đã biết trước và đã ghi vào `deferred-work.md` (chỗ gọi thứ TƯ xuyên `AppHandle`,
   cùng lớp với ba chỗ gọi trước của Story 5.2/5.3) — không bộ test Rust/frontend nào (thiếu
   `tauri::test`/`mock_builder`) chạm được `#[tauri::command]` thật; chỗ nối này CHỈ được canh
   bởi `e2e/specs/story-5-4-lifecycle.e2e.mjs`, thứ không chạy được trong môi trường triển khai
   này (cần WKWebView thật). Ghi rõ ở đây thay vì báo sai một cổng đã đóng.
2. Đổi `derive_work_status` cho nó trả `Paused` khi tập Chương chứa `Paused` ⇒ **ĐỎ đúng ba ca**
   (`core::lifecycle::tests::no_combination_of_chapters_ever_derives_paused`,
   `lifecycle_contract.rs::a_mixed_set_of_chapter_statuses_derives_in_progress_never_paused`,
   `lifecycle_contract.rs::no_combination_of_two_chapters_ever_derives_a_paused_work_through_the_command_layer`).
   Khôi phục xong, xanh lại.
3. Đổi `WorkMeta::status` default từ `None` sang `Some("not_started")` (mô phỏng phương án
   `String` + `#[serde(default)]` mà story loại) ⇒ **ĐỎ đúng ca**
   `library_index_contract.rs::a_true_v1_meta_json_indexes_with_status_null_and_matches_no_filter`
   (ca dựng MỘT `meta.json` hình dạng v1 thật, thiếu hẳn hai khoá mới — không phải một
   `WorkMeta{status:None,..}` serialize ra `"status":null`). Khôi phục xong, xanh lại.

### Tệp đã đổi

**Rust — lõi**
- `src-tauri/src/core/lifecycle/mod.rs` — **mới**: `lifecycle_statuses!`, `LifecycleStatus`,
  `derive_work_status`, ba ca `#[cfg(test)]`.
- `src-tauri/src/core/mod.rs` — `pub mod lifecycle;`.
- `src-tauri/src/core/store/schema.rs` — `WORK_STATUS_OVERRIDE_DDL` (bước 16 MỚI của
  `PROJECT_MIGRATIONS`); `LIBRARY_WORK_DDL` viết lại tại chỗ (+`status`/`status_is_override`,
  `LIBRARY_INDEX_MIGRATIONS` 3→4); sửa 🔵 dòng doc-comment "không cột trạng thái vòng đời (chủ
  Story 5.4)".
- `src-tauri/src/core/library/meta.rs` — `WorkMeta` +2 trường (`Option`/`#[serde(default)]`),
  `META_SCHEMA_VERSION` 1→2, `rebuild_from_store` tính `status`/`status_is_override`.
- `src-tauri/src/core/library/indexer.rs` — UPSERT chở 2 cột mới; `IndexedWork` +2 trường;
  `list_works(filter)` lọc trong SQL, trả `WorksReport{total,works}`; tái xuất `Row`/`SqlResult`/
  `params_from_iter` qua `core::store` (không gõ tên `rusqlite` trực tiếp — `store_boundary.rs`).
- `src-tauri/src/core/store/mod.rs` — thêm tái xuất `pub use rusqlite::params_from_iter;`.
- `src-tauri/src/commands/project.rs` — xoá `CHAPTER_STATUS_NOT_STARTED`, dùng
  `LifecycleStatus::NotStarted.as_str()`; đổi tên `reindex_after_create_work` →
  `reindex_library`, `pub(crate)`.
- `src-tauri/src/commands/lifecycle.rs` — **mới**: `WorkLifecycle`, `unknown_status`
  (`pub(crate)`, tái dùng bởi `commands::library`), ba hàm thuần + `mod wire` (hai vỏ
  `(async)`).
- `src-tauri/src/commands/library.rs` — `WorkRow`, `WorkListReport`, `list_works` (hàm thuần +
  vỏ `library_list_works`, không `(async)`).
- `src-tauri/src/commands/mod.rs` + `src-tauri/src/lib.rs` — khai module, đăng ký bốn vỏ mới
  trong `generate_handler!`.
- `src-tauri/src/commands/chapter.rs` — `chapter_not_found` lên `pub(crate)` (tái dùng bởi
  `commands::lifecycle`).
- `src-tauri/src/core/i18n/mod.rs` — `MessageKey::LifecycleUnknownStatus`.

**Rust — test**
- `src-tauri/tests/lifecycle_contract.rs` — **mới**: toàn bộ §I/O Matrix qua `OpenWork` thật,
  cộng ca `ALL.len()` đối chiếu hằng viết tay, ca nhãn khớp `vi.json`, ca "không bao giờ paused"
  chạy qua đường LỆNH (không chỉ hàm thuần).
- `src-tauri/tests/library_index_contract.rs` + `library_commands_contract.rs` — ba `WorkMeta`
  struct literal sửa (10 trường, kèm dòng lý do); thêm ca lọc từng giá trị, ca "bộ lọc quét
  sạch", ca "meta.json v1 thật ⇒ status IS NULL, không khớp bộ lọc nào" (ca DUY NHẤT chứng minh
  `Option` khác phương án mặc-định).
- `src-tauri/tests/ipc_contract.rs` — đóng băng `snake_case` của `WorkRow`/`WorkListReport`/
  `WorkLifecycle`.
- `src-tauri/tests/config_invariants.rs` — hai vỏ mới vào
  `the_blocking_wires_run_off_the_main_thread` (10→12 ca).
- `src-tauri/tests/pinned_contract.rs` + `segment_contract.rs` — cập nhật mọi khẳng định
  `schema_version()`/`PROJECT_MIGRATIONS.len()` từ 14/15 lên 15/16 (bước 16 mới), kèm dòng
  🔵 CẬP NHẬT tại mỗi chỗ; fixture "future version" của `segment_contract.rs` nâng
  `STEP_SIXTEEN`→`STEP_SEVENTEEN` (`to_version: 17`) để giữ đúng ý nghĩa "phiên bản chưa ai
  hiểu".

**TypeScript**
- `src/config/lifecycle.ts` — **mới**: adapter thứ chín, ba hàm.
- `src/config/library.ts` — `WorkRow`/`WorkListReport`/`listLibraryWorks`.
- `src/modes/libraryWorks.ts` — **mới**: state khối "Tác phẩm" + khối "Tác phẩm đang mở",
  `loadWorks`/`toggleStatusFilter`/`clearStatusFilter`/`loadOpenWorkLifecycle`/
  `setOpenWorkOverride`/`clearOpenWorkOverride`/`setOpenChapterStatus`/`resetLibraryWorks`.
- `src/commands/index.ts` + `src/main.ts` — chín lệnh mới đăng ký + tiêm `deps`; `Mod+Alt+W`
  cho `library.list_works` (đo trống trước khi cấp).
- `src/modes/LibraryMode.vue` — khối "Tác phẩm" (bốn nút lọc RIÊNG, không `v-for` chọn id động
  — `check:commands` Kiểm A đòi id LITERAL) + khối vòng đời Tác phẩm đang mở.
- `src/i18n/vi.json` — `err.lifecycle.unknown_status`; 4 `lifecycle.*`; 9 `command.*`; 12
  `mode.library.*`.

**Frontend — test**
- `tests/frontend/libraryWorks.test.ts` — **mới**.

**e2e**
- `e2e/specs/story-5-4-lifecycle.e2e.mjs` — **mới**, hai kịch bản (ghi đè hiện dấu phân biệt,
  một bộ lọc lọc riêng rẽ) — CHƯA CHẠY THẬT trong phiên này (cần WKWebView, nhịp đêm).

**Sổ sách**
- `_bmad-output/implementation-artifacts/deferred-work.md` — nối tiếp ba mục cũ (đường ĐỌC
  thuần đóng MỘT NỬA; chỗ gọi thứ tư của `reindex_library`); hai mục MỚI có chủ (chi phí quét
  toàn bộ sau mỗi lượt ghi trạng thái, chủ 5.6; bề mặt đổi trạng thái chỉ Tác phẩm đang mở, chủ
  5.6/5.7).
- `src-tauri/AGENTS.md` — nối tiếp khối 🔵 dòng 29.

### Chưa nghiệm thu được ở tầng này

- `e2e/specs/story-5-4-lifecycle.e2e.mjs` cần chạy thật trên WKWebView (nhịp đêm hoặc
  `npm run test:e2e` tay) — đây là bằng chứng DUY NHẤT cho chỗ nối `dispatch` → `invoke` →
  `commands::lifecycle`/`commands::library` → `reindex_library` → DOM; đối chứng bắt buộc #1 ở
  trên đã CHỨNG MINH (không suy luận) rằng không con đường tự động nào khác canh được nó.
- §Manual checks (chạy tay ứng dụng thật) chưa thực hiện trong phiên này.
- `npm run check:scope`/`check:scope:bundled` không chạy (cần cổng 1420 trống + cửa sổ Tauri
  thật) — đúng ngoại lệ đã ghi ở `AGENTS.md::Running and verifying`.

## Auto Run Result

Status: done
Blocking condition: (không có)

### Đã dựng gì

Bốn giá trị vòng đời khai **một chỗ** bằng `lifecycle_statuses!` (chép khuôn `scope_kinds!`), dùng chung cho cả hai tầng. Tầng Chương giữ giá trị thật ở `chapter.status`; tầng Tác phẩm **không** lưu giá trị suy ra — chỉ `work.status_override` (`NULL` = đang suy ra), còn giá trị suy ra tính bằng một hàm thuần từ tập trạng thái Chương và **không bao giờ** sinh ra `paused` (FR6). Hai kho dẫn xuất (`meta.json` v2, `library_work` v4) chở cặp `(status, status_is_override)` để Library lọc được mà không mở SQLite. Bề mặt tối thiểu ở Library: danh sách phẳng kèm nhãn trạng thái + dấu ghi đè viết bằng chữ, bốn nút lọc bật/tắt riêng rẽ, và ba lệnh vòng đời cho Tác phẩm đang mở.

### Tệp đã đổi

- `src-tauri/src/core/lifecycle/mod.rs` — **mới**: macro danh mục đóng + `derive_work_status` (bảng bốn hàng).
- `src-tauri/src/core/store/schema.rs` — `work.status_override` là bước **16 MỚI** của `PROJECT_MIGRATIONS`; `LIBRARY_WORK_DDL` viết lại **tại chỗ** (+2 cột), `LIBRARY_INDEX_MIGRATIONS` 3 → 4.
- `src-tauri/src/core/library/meta.rs` — `WorkMeta` +2 trường `Option`/`#[serde(default)]`, `META_SCHEMA_VERSION` 1 → 2, `rebuild_from_store` tính giá trị suy ra/ghi đè, cộng chẩn đoán khi một hàng `chapter.status` không phân giải được.
- `src-tauri/src/core/library/indexer.rs` — UPSERT chở hai cột mới; `list_works(filter)` lọc **trong SQL**, trả `total` và hàng đã lọc từ **một** lượt đọc.
- `src-tauri/src/commands/lifecycle.rs` — **mới**: ba hàm thuần + `finish_lifecycle_write` (vị từ bước 4, khai một chỗ) + `mod wire` bốn vỏ.
- `src-tauri/src/commands/library.rs` — `WorkRow`/`WorkListReport`/`list_works` + vỏ `library_list_works`.
- `src-tauri/src/commands/project.rs` — bỏ hằng cục bộ `CHAPTER_STATUS_NOT_STARTED`; `reindex_after_create_work` → `reindex_library` (nay ba chỗ gọi).
- `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` — một `MessageKey` mới + nhãn vòng đời + khoá lệnh/màn hình.
- `src-tauri/tests/lifecycle_contract.rs` (mới) · `library_index_contract.rs` · `library_commands_contract.rs` · `ipc_contract.rs` · `config_invariants.rs` · `pinned_contract.rs` · `segment_contract.rs`.
- `src/config/lifecycle.ts` (mới, adapter thứ chín) · `src/config/library.ts` · `src/modes/libraryWorks.ts` (mới) · `src/modes/LibraryMode.vue` · `src/commands/index.ts` · `src/main.ts`.
- `tests/frontend/libraryWorks.test.ts` (mới) · `e2e/specs/story-5-4-lifecycle.e2e.mjs` (mới, **chưa chạy**).
- `src-tauri/AGENTS.md` · `deferred-work.md` · `sprint-status.yaml`.

### Kết quả rà soát

- **9 bản vá đã áp** (medium 5 · low 4) — chi tiết ở §Review Triage Log.
- **4 mục hoãn** có chủ (medium 2 · low 2) — ở frontmatter `deferred`.
- **4 mục loại.** Đáng nêu một mục: *"SQL commit rồi mà `meta.json` trượt thì chỉ mục đứng lại ở giá trị cũ, nên phải reindex bất kể `result`"*. Loại vì bản vá đề xuất **không đổi kết quả gì**: `write_atomic` là tmp → `rename`, nên một lượt trượt để `meta.json` trên đĩa **nguyên vẹn giá trị cũ**, và `Indexer::rebuild` đọc chính tệp đó — reindex sẽ ghi lại đúng giá trị cũ. Ba mục còn lại là văn phong/cosmetic (`params.iter()` thừa một lớp gián tiếp; `AGENTS.md` dài; ghi đè chỉ chạm Tác phẩm đang mở — đã có mục nợ từ trước).
- **Đề xuất rà lại: có.** Chỉ đếm bản vá của lượt này: 0 high, 5 medium, 4 low ⇒ `3×5 + 1×4 = 19`, vượt ngưỡng 5.

### Đã nghiệm thu bằng gì

- 9 cổng tĩnh xanh (`check:i18n` · `commands` · `tokens` · `panel-refs` · `layout` · `debt-owner` · `gates` · `deps` · `lint`).
- `npm run test` — 45 tệp / **601 ca**, 0 đỏ. `npm run build` xanh.
- `cargo test --locked` — **850 passed, 0 failed** trên 34 binary.
- `grep` giá trị trạng thái viết thẳng ngoài `core/lifecycle/mod.rs`: **0** ở vị trí mã.
- **Bốn đối chứng đã chạy thật, mỗi cái đỏ đúng chỗ rồi khôi phục:** ① gỡ bước 4 ⇒ 2 ca chỉ mục đỏ *(lượt ĐẦU của đối chứng này cho 0 đỏ — đó là thứ dẫn tới bản vá `finish_lifecycle_write`)*; ② bảng suy ra trả `Paused` ⇒ đỏ ở `no_combination_of_chapters_ever_derives_paused`; ③ `status` mặc định `"not_started"` thay cho `Option` ⇒ đỏ ở `a_true_v1_meta_json_indexes_with_status_null_and_matches_no_filter`; ④ bỏ hàng đợi tải lại ⇒ đỏ ở ca frontend mới.
- **Soát phủ §I/O Matrix:** cả 17 hàng có ít nhất một ca **đã chạy và đã xanh**.

### Rủi ro còn lại

- 🔴 **`e2e/specs/story-5-4-lifecycle.e2e.mjs` chưa chạy lần nào**, và nó là đường DUY NHẤT chạm bốn vỏ `#[tauri::command]` thật. Một mục `generate_handler!` bị rơi hay một lệch camelCase (`chapterId`) sẽ đi qua trọn `pre-push` + CI nhịp `push` mà không cổng nào đỏ. Chạy tay: `npm run test:e2e`, hoặc đợi nhịp đêm. **Đừng đọc "850 ca xanh" thành "chỗ nối đã được canh".**
- ⚠️ `pre-push` và các lệnh trên chạy trên **macOS**; nửa Windows chỉ có tiếng nói ở lượt CI — đọc lượt CI trước khi kết luận là xanh.
- ⚠️ Mỗi lượt ghi trạng thái kéo theo một lượt quét lại **toàn bộ** thư mục gốc. Đúng ở quy mô hôm nay, nhưng là nợ có chủ (Story 5.6).
- ⚠️ Bước di trú `global.db`/`project.db` là **cửa một chiều**: `PROJECT_MIGRATIONS` nay tới 16, và hạ cấp ứng dụng xuống bản không biết bước đó sẽ làm `project.db` bị **từ chối mở** (AD-30).
- Vế bàn tay người (§Manual checks) **chưa chạy** — không đánh dấu đạt bằng suy luận.
