---
title: 'Story 5.2: Chỉ mục Library dẫn xuất, một đường ghi duy nhất'
type: 'feature'
created: '2026-08-27'
status: 'done'
baseline_commit: '95b54e31d8ade3f35c62141d3a941688fe197cba'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-5-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `library-index.db` hôm nay chỉ có **chỗ đứng khai báo** — `StoreKind::LibraryIndex` (`core/store/mod.rs:173`) và một lời tự khai *"Không có `StoreSpec::library_index` hôm nay"* (`:292`). Không hàm dựng, không bộ di trú, không chỗ gọi `Store::open`, không `Indexer`. Hệ quả: FR98 (*"chỉ mục phải dựng lại được hoàn toàn từ các `.atproj`"*) và NFR10 chưa có một dòng mã nào chống lưng, và luật `src-tauri/AGENTS.md:29` (*"chỉ `Indexer` ghi `library-index.db`"*) là một quy ước **không cổng nào canh** — đúng lớp hỏng Story 5.1 vừa đóng cho quy ước đặt tên.

**Approach:** Dựng kho ghi được thứ ba theo AD-7/AD-11, cộng **nhánh không-di-trú** mà `core/store/mod.rs:163` giao lại cho story này: lệch phiên bản lược đồ ⇒ **xoá tệp rồi dựng lại**, không `ALTER TABLE` — ngược chiều có chủ ý với `project.db`/`global.db` vốn phải **từ chối mở** khi gặp lược đồ mới hơn (AD-30). Chỉ mục dẫn xuất **chỉ từ `meta.json`**, không mở `project.db` lần nào (AD-9). Rồi biến ba mệnh đề còn trần trụi thành phép đo: một đường ghi duy nhất, `.atproj` ghi trước, xoá chỉ mục là thao tác an toàn.

## Boundaries & Constraints

**Always:**
- `Indexer` **tự mở** kho của nó (`Indexer::open`); `lib.rs` chỉ gọi `Indexer::open(...)` + `app.manage(...)`. Lý do: giữ danh sách miễn trừ của cổng ranh giới còn **đúng hai** đường dẫn (`core/library/indexer.rs` + điểm khai `core/store/mod.rs`) thay vì ba.
- Chỉ mục chép **trung thành** những gì `meta.json` khai — không suy diễn, không vá. `meta.json` hôm nay đóng băng từ lúc tạo (§Design Notes); đó là khuyết tật của **nguồn**, ghi nợ có chủ, không sửa ở đây.
- Trùng `work_id` ⇒ **phát hiện, giữ mục đầu, trả ra trong kết quả** — không gộp, không ghi đè im lặng (AD-28). Một `.atproj` thiếu/hỏng `meta.json` cũng phải **phân biệt được** với "không có Tác phẩm nào", không rơi im lặng.
- Cổng ranh giới mới chép đủ **bốn phần** của `store_boundary.rs`: sàn quần thể → needle cấm → đối chứng dương (module thật sự dùng) → tự kiểm mỗi miễn trừ khớp ≥ 1 tệp thật.
- Thư mục tạm trong test theo khuôn `store_contract.rs:51-69` (pid + `AtomicU64`, `cleanup` sau khi `Store` đã drop). **Không thêm `tempfile`.**

**Ask First:**
- Bất kỳ thay đổi nào lên `ARCHITECTURE-SPINE.md`, hoặc lên một AC đã ký của story trước.
- Nếu nhánh không-di-trú không dựng được mà **không** đổi ngữ nghĩa từ chối `SchemaTooNew` của `Store::open` (`core/store/mod.rs:551` bước 3) — thứ `project.db`/`global.db` đang dựa vào: HALT, trình hai phương án kèm số đo, đừng tự chọn. *(Thêm `StoreSpec::library_index` và sửa ba chú thích ở cùng tệp thì **không** cần hỏi — đó là việc đã giao ở §Tasks.)*

**Never:**
- **Không FTS5, không bảng tìm kiếm.** Spine xếp *"Cấu trúc chi tiết chỉ mục FTS cho tìm kiếm Library"* vào bảng Deferred (spine:1056); Story 5.9 sở hữu. Dựng hôm nay là chốt im lặng một quyết định đang mở.
- **Không cột cho tính năng chưa tồn tại:** không `cover` (chủ: 5.6, đã ghi nợ ở 5.1), không cột trạng thái vòng đời (chủ: 5.4), không cột tiến độ (chủ: 5.5). Lược đồ = đúng các trường `WorkMeta` cộng đường dẫn `.atproj`.
- Không đụng đường flush nóng, không bơm `work.updated_at`/`chapter.updated_at`, không làm `meta.json` sống lại — **Ice chốt hẹp 2026-08-27** (§Design Notes).
- Không dựng lệnh IPC, không dựng lưới Tác phẩm, không quét-lại-tăng-dần/phát hiện mục mồ côi (FR99 = Story 5.3).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Dựng lại từ đĩa | Thư mục gốc có N `.atproj` hợp lệ | Chỉ mục có đúng N hàng, mọi trường khớp từng ký tự với `meta.json` | N/A |
| Chỉ mục vắng mặt | Xoá `library-index.db` rồi mở lại | Dựng lại đủ N hàng; không `.atproj` nào bị chạm (so byte trước/sau) | N/A |
| Lược đồ lệch | Tệp trên đĩa mang version ≠ version đích (cả hai chiều) | Xoá tệp, dựng lại từ đầu — **không** `ALTER TABLE`, **không** từ chối mở | N/A |
| Trùng `work_id` | Hai `.atproj` cùng UUID | Giữ mục đầu, mục sau trả ra trong danh sách xung đột kèm cả hai đường dẫn | Không gộp, không ghi đè |
| `.atproj` hỏng | Thư mục `.atproj` thiếu `meta.json`, hoặc JSON không phân tích được | Bỏ qua **có ghi nhận** trong kết quả, các Tác phẩm còn lại vẫn vào chỉ mục | Trả ra, không panic |
| `meta.json` mới hơn | `meta_schema_version` > `META_SCHEMA_VERSION` | Cùng nhánh "`.atproj` hỏng" — bỏ qua có ghi nhận, không đọc bừa | `MetaError::SchemaTooNew` |
| Thư mục gốc vắng | Thư mục gốc Library chưa tồn tại | Chỉ mục rỗng **có lý do**, phân biệt được với "chưa quét" | Không tạo thư mục, không lỗi |
| Xoá chỉ mục ≠ mất dữ liệu | `global.db` có Glossary/pinned; xoá `library-index.db` | Mọi hàng `global.db` nguyên vẹn | N/A |
| Thứ tự ghi | `create_work` chạy tới cùng | Hàng chỉ mục chỉ xuất hiện **sau khi** `meta.json` đã trên đĩa | Chỉ mục hỏng ⇒ `.atproj` vẫn đầy đủ |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/mod.rs:167-201` -- `StoreKind` (4 biến thể; `LibraryIndex` → `"library-index"` ở `:198`). `:278-315` -- `StoreSpec` (`kind`/`path`/`tuning`/`migrations`) và **chỉ hai** hàm dựng: `::global(:293)`, `::project(:307)`. `:163`/`:172`/`:292` -- ba chú thích tự khai kho này là Epic 5, không di trú, và *"cần một nhánh khác mà story đó phải tự quyết"* — sửa cả ba khi nhánh đó ra đời.
- `src-tauri/src/core/store/mod.rs:551` -- `Store::open`, 9 bước có đánh số; `:687-695` `close()` theo thứ tự writer → readers → checkpoint. Nhánh từ chối `SchemaTooNew` nằm ở bước 3 — đây là chỗ `library-index.db` phải rẽ khác.
- `src-tauri/src/core/store/schema.rs:592` -- `GLOBAL_MIGRATIONS` (đích v5); `:1305`-`:1403` -- `PROJECT_MIGRATIONS` (đích v15) + `target_version()` `:1403-1405`. Không có hằng `SCHEMA_VERSION` đặt tên — quy ước là `migrations.last().to_version`. `:53-57` -- DDL `schema_migration_log`.
- `src-tauri/src/core/store/writer.rs:232`/`:245` -- `Writer::write` (chặn) và `::enqueue` (trả `WriteTicket`, `:143` `wait`). Một luồng, một hàng đợi (AD-11).
- `src-tauri/src/core/store/pragmas.rs:148-155,185-188,219-221` -- `journal_mode = WAL`, `wal_autocheckpoint = 0`, `busy_timeout`, `synchronous = FULL`. Kho mới thừa hưởng nguyên bộ qua `Tuning`.
- `src-tauri/src/core/library/meta.rs:33` -- `META_SCHEMA_VERSION = 1`; `:73-92` -- `WorkMeta` **7 trường** (`work_id`, `name`, `source_lang`, `genre`, `created_at`, `updated_at`, `chapter_count`) + `meta_schema_version`, có `Serialize`/`Deserialize`, **không** `rename_all`; `:105-125` -- `WorkMeta::read` (từ chối `SchemaTooNew`) — **0 chỗ gọi sản phẩm hôm nay**, story này là chỗ gọi đầu tiên; `:131-182` -- `write_atomic` (tmp → `sync_all` → `rename` → fsync thư mục cha); `:188-227` -- `rebuild_from_store`.
- `src-tauri/src/core/library/atproj.rs:78,162,215` -- `sanitize_name`, `create_work_folder`, `remove_folder`. Doc-comment `:3-8`: thư mục do mã dựng có **ba** mục (`meta.json`, `project.db`, `assets/`); `-wal`/`-shm` là sidecar SQLite ⇒ phép quét **không được** giả định đúng ba tên (`deferred-work.md:761`).
- `src-tauri/src/commands/project.rs:106-124` -- `default_library_root()` = `~/Documents/AuraTranslate/` qua `document_dir()` (AD-23), có override e2e (`lib.rs:143/148`). Hôm nay chỉ là nơi **tạo**; story này là nơi đầu tiên **quét** nó.
- `src-tauri/src/commands/project.rs:140-263` -- `create_work`: `create_work_folder(:147)` → `Store::open(:149)` → một `store.write` (`INSERT INTO work :176`, `INSERT INTO chapter :182`, `insert_segments :197`) → `rebuild_from_store(:217)` → `write_atomic(:242)` → `ScopeResolver::with_work(:252)`. **Chỗ cắm chỉ mục là ngay sau `:242`.** Rollback thủ công ở `:152,:208,:219,:247`.
- `src-tauri/src/commands/chapter.rs:103` + `:305-325` -- khuôn IPC hai lớp (`try_state`, **không** `state()` vì `panic = "abort"`). Ghi ở đây làm **điểm chờ cho Story 5.6**, không phải việc của story này — §Never cấm dựng bề mặt IPC ở lượt này.
- `src-tauri/src/lib.rs:523-528` -- `StoreSpec::global` + `app.manage(store)` trong `setup()`; `:656` -- `app.manage(OpenWorkState::new(None))`. Đây là khuôn cho hai dòng `lib.rs` được phép thêm.
- `src-tauri/tests/store_boundary.rs:57,69,120-130,133-194,200-220` -- **khuôn để chép**: `RS_FLOOR = 43`, mảng `FORBIDDEN`, sàn quần thể, đối chứng âm, đối chứng dương, cộng tự kiểm miễn trừ `:177-180` và chuẩn hoá `\`→`/` `:75-86`.
- `src-tauri/tests/store_contract.rs:51-69` -- khuôn thư mục tạm (pid + `AtomicU64`, `cleanup` sau drop); `:1303` -- ca duy nhất chạm `LibraryIndex` hôm nay (`as_str() == "library-index"`).
- `src-tauri/tests/naming_boundary.rs:789` -- fixture chứa `enum StoreKind { … LibraryIndex … }`; nếu thêm biến thể/đổi thân enum thì chuỗi dựng tay này phải theo kịp, nếu không ca đó xanh giả.
- `src-tauri/tests/project_contract.rs:216-239` -- ca đã có cho `rebuild_from_store` + `write_atomic`; `:8` -- doctrine *"Không thêm `tempfile`"*.
- `_bmad-output/implementation-artifacts/deferred-work.md:761` (quét gặp năm mục), `:7992-7996` (nợ `work.updated_at`, chủ hiện là story này) -- hai mục phải nối tiếp, **không xoá**.
- `src-tauri/AGENTS.md:29` -- luật *"chỉ `Indexer` ghi `library-index.db`… Xoá chúng phải luôn là thao tác an toàn"*; sau story này phải trỏ được tới cổng có thật, theo đúng cách `AGENTS.md:41` đã được sửa ở Story 5.1.
- `.githooks/pre-push` -- 11 cổng `check:*` → `npm run test` → `npm run build` → `cargo test --locked`. Cổng mới là **test Rust**, không phải `check-*.mjs`, nên **không** đụng ba danh sách của `check:gates`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store/schema.rs` -- thêm `LIBRARY_INDEX_MIGRATIONS`: **đúng một** bước dựng bảng chỉ mục (khoá chính `work_id`, cộng đường dẫn `.atproj` và đúng các trường `WorkMeta`) -- lược đồ không di trú thì một bước là hình dạng đúng, và bump số phiên bản = viết lại chính bước đó.
- [x] `src-tauri/src/core/store/mod.rs` -- thêm `StoreSpec::library_index(path)`; sửa ba chú thích `:163`/`:172`/`:292` cho hết nói dối -- lời tự khai phải theo kịp mã, đúng cách Story 5.1 sửa `AGENTS.md`.
- [x] `src-tauri/src/core/library/indexer.rs` -- **tệp mới**: `Indexer::open` (nhánh không-di-trú: lệch phiên bản ⇒ xoá tệp + sidecar rồi dựng lại), `Indexer::rebuild(root)` (quét `.atproj`, đọc `meta.json`, ghi qua `store::Writer`), `Indexer::list_works` (đường đọc), và một kiểu kết quả chở **số đã lập chỉ mục + xung đột UUID + mục bỏ qua** -- rỗng phải có lý do, không rỗng im lặng.
- [x] `src-tauri/src/core/library/mod.rs` -- khai `pub mod indexer;` và sửa chú thích `:13-14` (*"Story 5.2 sở hữu `library-index.db`"*) thành mệnh đề đã hết hạn có ngày -- 🔵 tại chỗ, không để lặng lẽ sai.
- [x] `src-tauri/src/lib.rs` -- mở chỉ mục trong `setup()` và `app.manage(...)`, theo khuôn `:523-528` -- hai dòng, và **chỉ** hai dòng, để cổng ranh giới không phải nới miễn trừ.
- [x] `src-tauri/src/commands/project.rs` -- sau `write_atomic(:242)` thành công thì đưa Tác phẩm mới vào chỉ mục; chỉ mục lỗi **không** làm hỏng `.atproj` đã ghi -- đây là AD-8 "ghi trước / ghi sau" viết bằng mã.
- [x] `src-tauri/tests/library_index_boundary.rs` -- **tệp mới**: cổng "chỉ `Indexer` ghi", chép đủ bốn phần khuôn `store_boundary.rs` + đối chứng dương/âm trên chuỗi dựng tay -- biến `src-tauri/AGENTS.md:29` thành phép đo.
- [x] `src-tauri/tests/library_index_contract.rs` -- **tệp mới**: toàn bộ §I/O Matrix, cộng ca so byte `.atproj` trước/sau một lượt dựng lại -- *"không mất một byte"* phải là phép đo, không lời hứa.
- [x] `src-tauri/AGENTS.md` -- dòng 29 trỏ tới hai tệp cổng vừa dựng -- lời tự khai phải kiểm được.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- nối tiếp mục `:7992` (mở rộng: `chapter.updated_at` cũng đóng băng; đổi chủ sang Story 5.6 theo phán quyết hẹp của Ice 2026-08-27) và ghi ba nợ mới có chủ: `meta.json` đóng băng từ lúc tạo (chủ **5.5**), bề mặt đọc/lưới của AC7 (chủ **5.6** + **5.9**), hiển thị cảnh báo trùng UUID (chủ **5.6**) -- không mục nào mồ côi, không mục cũ bị xoá.

**Acceptance Criteria:**
- Given một chỉ mục dựng từ N `.atproj`, when xoá `library-index.db` rồi dựng lại, then chỉ mục cũ và mới bằng nhau từng hàng từng cột, và mọi tệp trong mọi `.atproj` giống hệt byte-với-byte.
- Given một người sau này gọi `StoreSpec::library_index` hay mở kho đó từ một module ngoài `core/library/indexer.rs`, when chạy `cargo test`, then cổng ranh giới đỏ và nêu đúng `tệp:dòng`.
- Given `src-tauri/AGENTS.md:29`, when đọc, then nó nêu tên cổng có thật, không còn là một quy ước không ai canh.
- Given bộ test cũ (`store_contract`, `project_contract`, `naming_boundary`, `scope_contract`, `segment_contract`, bộ frontend), when chạy sau story này, then xanh — kho thứ ba là **thêm vào**, không đổi hành vi kho nào đang có.

## Spec Change Log

- **Vòng rà 1 — không hoàn nguyên, mười mục vá.** Ba lớp rà độc lập; không mục nào là `intent_gap` hay `bad_spec`, nên mã không bị dựng lại. Hai phát hiện đáng ghi lại vì chúng nói về **dụng cụ đo**, không về mã:
  **① Lệnh grep của §Verification là dụng cụ hỏng, và nó đã gây hại thật.** Bản đầu ghi *"expected: chỉ hai tệp"*. `grep` không phân biệt mã với chú thích, nên mọi doc-comment giải thích ranh giới đều thành vi phạm giả — lượt dựng đầu phản ứng bằng cách viết một doc-comment **cố ý mơ hồ** ở `lib.rs` (nguyên văn: *"cố ý không đánh vần đủ hai định danh đó"*), tức làm mờ tài liệu sản phẩm để giữ một phép kiểm xanh. Cùng lỗi lặp trong test: `library_index_boundary.rs:253` quét toàn văn trong khi `:136` của **cùng tệp** viết rõ *"Dòng comment KHÔNG phải vi phạm"*. Đã sửa **nguồn** cả hai vế: ca `:253` nay lọc `//` bằng đúng vị từ Phần 2 dùng, doc-comment `lib.rs` nói thẳng trở lại, và §Verification ghi kỳ vọng đúng (ba tệp, `lib.rs` chỉ ở dòng `///`).
  **② Kho dẫn xuất không tự lành khi tệp HỎNG — khác "lệch phiên bản".** §I/O Matrix có hàng cho lệch phiên bản nhưng không có hàng cho *"tệp tồn tại nhưng không đọc được"*. `delete_if_schema_version_differs(...)?` đẩy lỗi lên ⇒ `Indexer::open` hỏng vĩnh viễn ⇒ Library rỗng mãi mãi không nói vì sao — đúng lớp lỗi trung tâm dự án cấm, và ngược AD-8. Đã vá: không đọc được phiên bản ⇒ đồng hạng với lệch ⇒ xoá và dựng lại.
  **Xếp `patch` chứ không `bad_spec`, có chủ ý** — theo đúng tiền lệ đã ghi ở §Spec Change Log của Story 5.1: bản vá cơ học, chỉ một cách đọc hợp lý (một kho **dẫn xuất** thì xoá một tệp không đọc được không mất gì theo định nghĩa), và hoàn nguyên ~850 dòng đã xanh để dựng lại một kết quả gần y hệt là nghi thức, không phải tính mạch lạc. ⚠️ Nhưng gốc rễ nằm **trong** khối `<frozen-after-approval>` (thiếu một hàng ma trận) nên **chỉ Ice** thêm hàng đó được — cờ lên ở báo cáo bàn giao.
  **KEEP — phải sống sót mọi lần dựng lại:** (1) ca P1 phải được chứng minh **ĐỎ trước khi vá**, nguyên văn `read PRAGMA user_version: file is not a database`; (2) vị từ quét dùng **CHUNG một hàm** cho cổng thật lẫn mọi đối chứng dương/âm — hai bên không được trôi khỏi nhau bằng một phép so chép lại; (3) không phép kiểm nào của story này được quét toàn văn tệp mà bỏ bước lọc comment; (4) `RebuildOutcome` phải giữ **cả ba** vế phân biệt (`indexed` · `root_missing` · `conflicts`/`skipped`) — một con số `0` một mình không nói được vì sao rỗng.

## Design Notes

**Nhánh không-di-trú, viết ra vì nó ngược chiều với phần còn lại của kho.** `project.db`/`global.db` gặp lược đồ **mới hơn** thì **từ chối mở và không bao giờ ghi vào** (AD-30) — vì chúng là nguồn sự thật. `library-index.db` là **dẫn xuất**, nên đúng phản xạ đó ở đây là một lỗi: nó phải **xoá và dựng lại**, ở **cả hai** chiều lệch. Đây là chỗ dễ chép nhầm khuôn nhất trong story, và là thứ `core/store/mod.rs:163` giao lại bằng chữ.

**Vì sao chỉ đọc `meta.json`, không mở `project.db`.** AD-9 nói `meta.json` là *"metadata Library đọc được **không cần mở SQLite**"*. Suy mốc sửa thật (`MAX(segment.updated_at)`) buộc mở N tệp SQLite mỗi lượt quét — với 5.000 Chương đó là rủi ro thẳng vào NFR4 (< 3 giây). Đo hôm nay không kết luận được (chưa có đường tạo 5.000 Chương — đó là FR14/Epic 6, và phép đo đủ điều kiện là Story 6.18), nên chọn hình dạng **không** mở SQLite và ghi lý do lại đây.

**Phạm vi hẹp — Ice chốt 2026-08-27, kèm bốn phép đo đã trình:** (1) `work.updated_at` 1 lần ghi (`project.rs:177`), **0** `UPDATE`; (2) `chapter.updated_at` cũng **0** `UPDATE` toàn cây — sổ nợ hiện chỉ nêu `work`, tức lời khả nợ hẹp hơn khuyết tật thật; (3) `meta.json` có **đúng một** chỗ gọi `write_atomic` sản phẩm (`project.rs:242`), nên mọi trường của nó đúng đúng một lần trong đời; (4) bơm `work.updated_at` trong giao dịch flush làm cổng đang xanh `segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else` **đỏ**, tức phải mở lại một AC đã ký của Story 2.3. ⇒ Chỉ mục chép trung thành `meta.json`; ba khuyết tật trên là của **nguồn**, đi vào sổ nợ có chủ.

⚠️ **Hệ quả phải nói thẳng, không để người sau tự phát hiện:** tới hết story này, cột "ngày sửa" và `chapter_count` trong chỉ mục thực chất là **giá trị lúc tạo**. Chỉ mục **đúng** (nó phản chiếu đĩa trung thực, và đó chính là điều FR98/NFR10 đòi); thứ sai là nguồn.

**AC7 chỉ đóng được một nửa, có chủ.** *"Danh sách, lọc, sắp xếp và tìm kiếm của Library đọc từ `library-index.db`"* mô tả đích đến; hôm nay `src/modes/LibraryMode.vue` là khung rỗng có chủ ý và `tests/frontend/**` có **0** tệp chạm Library. Năng lực chưa dựng ≠ lệch spec: story này dựng **đường đọc** (`Indexer::list_works`, có test) và ghi nợ vế hiển thị cho 5.6/5.9 — không sửa `epics.md`.

## Verification

**Commands:**
- `cd src-tauri && cargo test --test library_index_contract --test library_index_boundary` -- expected: xanh, và **mọi** ca đối chứng dương/âm đều chạy (đối chứng dương phải được chứng minh đỏ được bằng một lượt thử tay, ghi lại kết quả).
- `cd src-tauri && cargo test --locked` -- expected: toàn bộ xanh; đặc biệt `store_contract`, `project_contract`, `naming_boundary`, `scope_contract`, `segment_contract` không ca nào đỏ.
- `npm run test` -- expected: xanh (43 tệp / 567 ca ở baseline; story này không thêm tệp frontend nên con số phải giữ nguyên — lệch là dấu hiệu chạm nhầm cây).
- `npm run check:gates && npm run check:debt-owner && npm run check:i18n` -- expected: xanh. `check:gates` phải xanh **mà không** cần sửa ba danh sách: cổng mới là test Rust, không phải `check-*.mjs`.
- `grep -rn "StoreSpec::library_index\|StoreKind::LibraryIndex" src-tauri/src` -- expected: ba tệp — `core/library/indexer.rs`, `core/store/mod.rs` (điểm khai), và `lib.rs` **chỉ ở dòng `///`**.
  🔵 **Sửa 2026-08-27, vòng rà 1 — lệnh này ban đầu ghi *"chỉ hai tệp"* và đó là một dụng cụ hỏng.** `grep` không phân biệt được **mã** với **chú thích**, nên kỳ vọng "hai tệp" biến mọi dòng doc-comment giải thích ranh giới thành một vi phạm giả — và nó đã gây hại thật: lượt dựng đầu viết một doc-comment cố ý mơ hồ ở `lib.rs` để né đúng lệnh này. Phán quyết thuộc về `cargo test --test library_index_boundary`, thứ **có** lọc `//` (`:136`). Cách đọc đúng lệnh grep: xem mỗi dòng khớp là **mã** hay `///`; một dòng mã ở tệp thứ ba là vi phạm thật và cổng phải đỏ — nếu cổng xanh thì **cổng hỏng**.
- `grep -rn "rusqlite\|Connection::open" src-tauri/src/core/library/` -- expected: **0 dòng**. `Indexer` đi qua `Store`/`Writer`, không tự mở kết nối (AD-11, và `store_boundary.rs` đang canh).

**Manual checks:**
- Chạy ứng dụng, tạo một Tác phẩm, xoá `$APPDATA/library-index.db` bằng tay, mở lại ứng dụng: chỉ mục dựng lại, Glossary toàn cục và mục đã ghim trong `global.db` còn nguyên.

## Suggested Review Order

**Nhánh KHÔNG-DI-TRÚ — thứ đi ngược phần còn lại của kho, đọc trước tiên**

- Bắt đầu ở đây: cả hình dạng lẫn lý do nó không giống hai kho kia.
  [`indexer.rs:69`](../../src-tauri/src/core/library/indexer.rs#L69)

- Nguồn sự thật đã cũ tự sửa: kho thứ ba nay có mã, không còn là lời hứa.
  [`store/mod.rs:181`](../../src-tauri/src/core/store/mod.rs#L181)

- Nửa QUYẾT ĐỊNH: lệch phiên bản **cả hai chiều**, và không đọc được cũng tính là lệch.
  [`store/mod.rs:890`](../../src-tauri/src/core/store/mod.rs#L890)

- Nửa ĐỌC, cố ý không đi qua `Store::open` — nhánh từ chối của nó sai cho kho dẫn xuất.
  [`store/mod.rs:834`](../../src-tauri/src/core/store/mod.rs#L834)

- Mở không `CREATE`: nếu không, một tệp vừa bị xoá bị bịa lại thành phiên bản 0.
  [`pragmas.rs:69`](../../src-tauri/src/core/store/pragmas.rs#L69)

**Chỉ mục dẫn xuất — chỉ đọc `meta.json`, không mở `project.db` lần nào**

- Đường ghi duy nhất: quét, đọc `meta.json`, ghi lại toàn bảng trong một giao dịch.
  [`indexer.rs:98`](../../src-tauri/src/core/library/indexer.rs#L98)

- Rỗng phải có lý do — ba vế phân biệt, một số `0` một mình không nói được gì.
  [`indexer.rs:372`](../../src-tauri/src/core/library/indexer.rs#L372)

- Một thư mục hỏng không được huỷ cả lượt quét; tách thuần để gieo được lỗi giả.
  [`indexer.rs:298`](../../src-tauri/src/core/library/indexer.rs#L298)

- Lược đồ: đúng bảy trường `WorkMeta` cộng đường dẫn, không cột cho tính năng chưa có.
  [`schema.rs:1459`](../../src-tauri/src/core/store/schema.rs#L1459)

- Một bước di trú duy nhất — kho không di trú thì đó là hình dạng đúng.
  [`schema.rs:1479`](../../src-tauri/src/core/store/schema.rs#L1479)

**Chỗ nối — AD-8 "ghi trước / ghi sau" viết bằng mã**

- Ở lớp vỏ chứ không trong hàm thuần: `Indexer` sống trong state, hàm thuần thì không.
  [`project.rs:1498`](../../src-tauri/src/commands/project.rs#L1498)

- Lượt dựng lại đầu tiên chạy lúc khởi động — chưa có lệnh "quét lại" nào (Story 5.3).
  [`lib.rs:699`](../../src-tauri/src/lib.rs#L699)

- Phát hiện trùng UUID mà không ai quan sát thì chưa phải "cảnh báo" (AD-28).
  [`indexer.rs:402`](../../src-tauri/src/core/library/indexer.rs#L402)

**Cổng — thứ story này thật sự thêm vào kho**

- Bốn từ khoá và hai miễn trừ, cạnh nhau, là toàn bộ luật.
  [`library_index_boundary.rs:35`](../../src-tauri/tests/library_index_boundary.rs#L35)

- Đối chứng nặng nhất: cây nguồn thật, 0 vi phạm — và nó lọc dòng `//`.
  [`library_index_boundary.rs:117`](../../src-tauri/tests/library_index_boundary.rs#L117)

- Chỗ vòng rà 1 sửa: ca này từng quét toàn văn, mâu thuẫn với luật ngay trên nó.
  [`library_index_boundary.rs:284`](../../src-tauri/tests/library_index_boundary.rs#L284)

- Vị từ DÙNG CHUNG cho cổng thật và mọi đối chứng — hai bên không trôi khỏi nhau được.
  [`library_index_boundary.rs:97`](../../src-tauri/tests/library_index_boundary.rs#L97)

- Nguồn của luật; cổng chỉ thi hành nó, và nó nói cả hai giới hạn còn hở.
  [`AGENTS.md:29`](../../src-tauri/AGENTS.md#L29)

**Hợp đồng — chín hàng ma trận, cộng ca vòng rà bắt được**

- Ca vòng rà 1: tệp rác phải tự lành, không làm `Indexer::open` hỏng vĩnh viễn.
  [`library_index_contract.rs:593`](../../src-tauri/tests/library_index_contract.rs#L593)

- Lời hứa trung tâm của FR98/NFR10 thành phép đo: so byte `.atproj` trước và sau.
  [`library_index_contract.rs:168`](../../src-tauri/tests/library_index_contract.rs#L168)

- Trùng `work_id`: giữ mục đầu, trả cả hai đường dẫn — không gộp, không ghi đè.
  [`library_index_contract.rs:308`](../../src-tauri/tests/library_index_contract.rs#L308)

- Thứ tự ghi: hàng chỉ mục chỉ xuất hiện sau khi `meta.json` đã trên đĩa.
  [`library_index_contract.rs:543`](../../src-tauri/tests/library_index_contract.rs#L543)

- Mọi trường khớp từng ký tự với `meta.json` — chỉ mục chép, không suy diễn.
  [`library_index_contract.rs:124`](../../src-tauri/tests/library_index_contract.rs#L124)
