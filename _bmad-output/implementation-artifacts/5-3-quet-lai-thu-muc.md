---
title: 'Story 5.3: Quét lại thư mục'
type: 'feature'
created: '2026-08-27'
status: 'done'
baseline_commit: 'c533c62a867331065c6bf54de2d90b1de989d058'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-5-context.md'
warnings: ['oversized']
deferred:
  - summary: >-
      🔵 SỬA (2026-08-27, vòng rà THỨ HAI P6) — mục này VIẾT TRƯỚC khi dựng
      `e2e/specs/story-5-3-rescan.e2e.mjs` và không quay lại sửa. Nay MỘT trong ba vỏ
      (`library_choose_root`, hộp thoại native) vẫn không ca tự động nào chạm; HAI vỏ kia
      (`library_rescan`, `library_forget_orphan`) đã có 6 ca e2e chạm thật qua WKWebView.
    evidence: |-
      `e2e/specs/story-5-3-rescan.e2e.mjs` (6 ca, xem §Chạy trên ỨNG DỤNG THẬT) đi trọn
      đường nút thật → `dispatch` → registry → `invoke` → `Indexer` → DOM cho
      `library_rescan`/`library_forget_orphan` — đây LÀ lần đầu hai vỏ đó được một phép kiểm
      chạm tới, không chỉ bị so chuỗi chữ ký. `library_choose_root` vẫn CHỈ bị SO CHUỖI ở
      `config_invariants.rs::the_blocking_wires_run_off_the_main_thread`: hộp thoại chọn thư
      mục là native, ngoài tầm WebDriver, và bấm nó trong một lượt tự động sẽ TREO cửa sổ
      chờ người thật. Vế THẬT của AC6 ("gõ được trong lúc quét") vẫn sống ở §Manual checks —
      dựng một thư viện đủ lớn để đo là một phép đo chập chờn theo tốc độ đĩa người chạy.
      Mệnh đề "bộ e2e nằm ngoài cả `pre-push` lẫn `ci.yml`" ĐÃ SAI nửa sau: `ci.yml` có job
      `e2e` chạy ở nhịp `schedule` (cron `0 18 * * *`) + `workflow_dispatch` — mệnh đề đúng
      là "ngoài `pre-push` và ngoài `push`, nhưng CÓ trong `ci.yml` ở nhịp đêm". Đã ghi nợ có
      chủ ở `deferred-work.md` (chủ: Story 5.6, nối tiếp — không viết đè lịch sử); mục này
      lặp lại ở frontmatter để lượt rà sau không phải tự tìm.
    location: >-
      src-tauri/src/commands/library.rs (mod wire) — riêng library_choose_root
    severity: medium
  - summary: >-
      Con trỏ danh sách mồ côi kẹp theo CHỈ SỐ, không theo danh tính — thứ tự đổi giữa hai
      lượt quét thì con trỏ trỏ sang một mục khác mà người dùng không hay.
    evidence: |-
      `src/modes/libraryRescan.ts::applyReport` gán lại `orphans.value` trọn gói mỗi lượt
      rồi chỉ kẹp con trỏ về khoảng hợp lệ. Một mục mồ côi MỚI sắp trước mục đang chọn
      (danh sách sắp theo `work_id`) làm cùng một chỉ số trỏ sang mục khác. Hậu quả chặn
      trên: thao tác duy nhất trên mục đang chọn là `library.forget_orphan`, và nó chỉ xoá
      một hàng mồ côi — không mất dữ liệu người dùng, chỉ gỡ nhầm một lời nhắc.
    location: >-
      src/modes/libraryRescan.ts
    severity: low
  - summary: >-
      Miễn trừ `library_work` thêm vào cổng `no_update_statement_anywhere_touches_source_lang`
      dùng `contains` trên TOÀN chuỗi literal, tức rộng hơn đúng bảng nó đặt tên.
    evidence: |-
      `src-tauri/tests/project_contract.rs` loại mọi chuỗi SQL có chứa `library_work`. Một
      chuỗi literal tương lai nhắc cả `library_work` LẪN một `UPDATE work SET source_lang`
      sẽ lọt. Miễn trừ CÓ TÊN, có lý do tại chỗ, và có đối chứng dương/âm
      (`the_library_work_exemption_is_named_and_does_not_swallow_the_work_table`), nên nó
      không phải một lượt hạ ngưỡng — nhưng vị từ chặt hơn (loại theo TỆP
      `core/library/indexer.rs`, đúng khuôn các mảng `EXEMPT` khác của kho) thì đúng hơn.
    location: >-
      src-tauri/tests/project_contract.rs
    severity: low
  - summary: >-
      Một lượt e2e đỏ chưa chẩn đoán được, trong đó ứng dụng ĐỌC thư mục Library thật của
      người chạy thay vì thư mục tạm của bộ e2e; không byte nào bị ghi vào đó.
    evidence: |-
      Lượt đầu của `story-5-3-rescan.e2e.mjs` 5/6 đỏ, `.root-value` ra
      `/Users/hoangnam/Documents/AuraTranslate` và `.orphan-name` ra `Epochtime` (Tác phẩm
      thật). Hai lượt sau 6/6 xanh, root đúng thư mục tạm. Đã kiểm: thư mục thật không mọc
      thêm mục nào, mtime không đổi. Không giải thích được từ mã (override gác hai lớp cfg,
      thứ tự setup đúng, có cổng quét nguồn cho thứ tự ưu tiên). Giả thuyết `core.invoke
      not available` đã bị LOẠI — cảnh báo đó có ở cả ba lượt.
    location: >-
      e2e/specs/story-5-3-rescan.e2e.mjs
    severity: medium
  - summary: >-
      AC4 nói "cảnh báo" nhưng sản phẩm cho một CON SỐ TRẦN, và spec 5.3 không nêu đây là
      một chỗ hai cách đọc — trong khi nó có nêu chỗ tương tự về cờ mồ côi.
    evidence: |-
      `RescanReport.conflicts` chỉ chở `usize`; `WorkIdConflict` có đủ hai đường dẫn nhưng
      chỉ đi ra `stderr` qua `log_if_notable`. Con số nằm cùng một câu với "đã lập chỉ mục"
      và "bỏ qua", trong một node `.status` bình thường — không `role="alert"`, không class
      `.error` mà chính tệp đó dùng cho lỗi thật. Không ca nào (unit/contract/e2e) chạy kịch
      bản trùng `work_id` qua giao diện. Bề mặt hiển thị đã có chủ (Story 5.6, ghi từ 5.2);
      thứ CHƯA có chủ là quyết định "cảnh báo" nghĩa là một con số hay một affordance riêng.
    location: >-
      src-tauri/src/commands/library.rs (RescanReport.conflicts)
    severity: medium
  - summary: >-
      `default_library_root` vẫn là `pub fn` trần — không cổng nào buộc chỗ gọi MỚI đi qua
      `resolve_library_root`, nên "bề mặt rỗng im lặng thứ hai" mà AC5 tồn tại để chặn có
      thể mọc lại mà không ai đỏ.
    evidence: |-
      Kho đã có đúng khuôn cần thiết ở `library_index_boundary.rs` (cấm mọi tệp ngoài danh
      sách miễn trừ nhắc một định danh). `default_library_root` không có đối ứng như vậy;
      kỷ luật hiện nay chỉ là một câu trong doc-comment.
    location: >-
      src-tauri/src/commands/project.rs
    severity: medium
  - summary: >-
      Sau một lượt quét TRƯỢT, màn hình hiện dải báo lỗi mới cạnh kết quả CŨ mà không đánh
      dấu kết quả đó là đã cũ.
    evidence: |-
      `rescanLibraryFolder`/`chooseLibraryRootFolder` đặt `lastError` rồi `return` — không
      chạm `libraryRoot`/`orphans`/ba con số. Giữ dữ liệu cũ là lựa chọn ĐÚNG (xoá đi là mất
      thông tin), nhưng màn hình không nói ra rằng chúng đến từ một lượt trước.
    location: >-
      src/modes/libraryRescan.ts
    severity: low
  - summary: >-
      Vòng đánh dấu mồ côi chạy một `UPDATE` mỗi hàng cũ, bên trong giao dịch đang giữ
      `rebuild_lock`, thay vì một câu lệnh theo tập.
    evidence: |-
      Với một thư viện lớn đó là O(số hàng) lượt round-trip nối tiếp dưới đúng cái khoá chặn
      mọi lượt quét/gỡ khác — ngược chiều mục tiêu AC6. Chưa đo được vì Epic 5 không có
      đường tạo 5.000 Chương thật (đó là FR14/Epic 6, phép đo đủ điều kiện là Story 6.18).
    location: >-
      src-tauri/src/core/library/indexer.rs
    severity: low
---

<intent-contract>

## Intent

**Problem:** FR99 (*copy một thư mục `.atproj` vào là nó xuất hiện trong Library*) chưa có một bề mặt nào. `Indexer::rebuild` đã tồn tại từ Story 5.2 nhưng chỉ chạy ở **hai chỗ máy tự gọi** (`lib.rs::open_library_index` lúc khởi động, `commands/project.rs::reindex_after_create_work` sau khi tạo Tác phẩm) — không lệnh IPC, không command đăng ký, không phím. Đồng thời `rebuild` hôm nay là `DELETE FROM library_work` rồi `INSERT` lại: một `.atproj` bị **xoá/di chuyển ra ngoài** thư mục gốc biến mất khỏi chỉ mục **im lặng**, đúng lớp lỗi trung tâm mà kho cấm — AC3 đòi nó phải ở lại như một **mục mồ côi nêu rõ nó trỏ tới đâu**. Và thư mục gốc Library hôm nay là một hằng suy ra từ `document_dir()`, người dùng **không đổi được**.

**Approach:** Ba việc, một mạch: ① đổi ngữ nghĩa `rebuild` từ *xoá-sạch-ghi-lại* sang **đối chiếu** (thấy trên đĩa ⇒ cập nhật đường dẫn + gỡ cờ mồ côi; không thấy và đường dẫn cũ không còn ⇒ **đánh dấu mồ côi, giữ hàng**), cộng một đường **gỡ tường minh** cho hàng mồ côi; ② một khoá `AppConfig` mới cho thư mục gốc, đọc TRƯỚC khi rơi về `~/Documents/AuraTranslate/`, đổi được qua hộp thoại chọn thư mục gọi từ Rust (AD-48); ③ bề mặt IPC + command đăng ký + màn hình tối thiểu trong `LibraryMode.vue`, chạy **ngoài luồng chính** để một lượt quét không chặn thao tác người dùng.

## Boundaries & Constraints

**Always:**
- **`Indexer` vẫn là module DUY NHẤT mở `library-index.db`.** Bề mặt IPC mới (`commands/library.rs`) gọi xuống `Indexer` qua `try_state`, không tự dựng `StoreSpec` — cổng `tests/library_index_boundary.rs` phải xanh **mà không nới danh sách miễn trừ** (vẫn đúng hai đường dẫn).
- **Đường ghi vẫn DUY NHẤT một.** Không thêm một hàm "chèn một hàng" chạy song song với `rebuild`; `forget_orphan` là đường **xoá** tường minh, có tiền điều kiện, không phải một đường ghi thứ hai vào cùng bảng (xem §Design Notes).
- **Một lượt quét chạy trọn vẹn, không xen kẽ với lượt khác.** Quét đĩa + ghi phải nằm trong **cùng một vùng khoá** — đây là món nợ `deferred-work.md` đã giao đích danh cho story này, và story này biến một khả năng lý thuyết thành một cửa sổ thật (chỗ gọi `rebuild` thứ ba, do người dùng bấm).
- **Vị từ mồ côi có ĐÚNG HAI vế, và cả hai đều cần:** một hàng thành mồ côi khi `work_id` của nó **không** đọc được ở lượt quét này **VÀ** `atproj_path` của nó **không** nằm trong tập `.atproj` mà lượt quét vừa liệt kê được trong thư mục gốc. Bỏ vế thứ hai thì một `.atproj` còn nằm nguyên đó nhưng `meta.json` hỏng (hạng `skipped`, đã có từ Story 5.2) bị gọi là mồ côi — nói *"nó không còn ở đây"* về một thư mục đang nằm đó là một câu sai. Bỏ vế thứ nhất thì một Tác phẩm **di chuyển trong gốc** bị đánh dấu mồ côi ngay trước khi hàng của nó được cập nhật đường dẫn.
- **Vỏ IPC nào chặn thì mang `#[tauri::command(async)]`**, và ca `config_invariants.rs::the_blocking_wires_run_off_the_main_thread` phải **thật sự đọc tệp mới** (hôm nay nó viết cứng `src/commands/glossary.rs` — xem §Code Map).
- **Adapter `src/config/library.ts` KHÔNG BAO GIỜ ném** — một `invoke`, một `try/catch`, hình dạng ba trạng thái. Mọi `@click` trong `.vue` là **đúng một** `dispatch('<id>')`.
- **Danh sách rỗng phải nói vì sao rỗng** — vị từ `…HasLoaded` có mặt TRƯỚC khi màn hình khẳng định *"không có mục mồ côi nào"*.
- Móc e2e `library_root_override()` giữ **thứ tự ưu tiên đầu tiên**, trước cả giá trị người dùng cấu hình.

**Block If:**
- Nhánh đối chiếu mới không dựng được mà **không** đổi ngữ nghĩa từ chối `SchemaTooNew` của `Store::open` (`core/store/mod.rs:551` bước 3) — thứ `project.db`/`global.db` đang dựa vào.
- Việc thêm trường mới lên dây buộc phải **sửa một AC đã ký** của story trước (ví dụ hợp đồng `save_segment_targets` chạm đúng hai cột của Story 2.3, hay danh sách trường `BootstrapConfig` của Story 1.8/1.14/1.19).
- Bất kỳ thay đổi nào lên `ARCHITECTURE-SPINE.md`, hoặc việc cấp một số `AD` mới.

**Never:**
- **Không lưới Tác phẩm, không lọc, không sắp xếp, không bìa, không bốn trạng thái vòng đời, không tiến độ** — 5.4/5.5/5.6 sở hữu. Màn hình ở story này chỉ có: thư mục gốc, nút quét lại, danh sách **mục mồ côi**, và ba con số kết quả.
- **Không FTS5, không bảng tìm kiếm** (5.9 sở hữu, spine xếp vào Deferred).
- **Không BỎ QUA thư mục theo `mtime`, không theo dõi hệ thống tệp (watcher).** *(Story 5.2 giao lại cho story này *"quét-lại tăng dần"* — nghĩa đã nhận ở đây là **đối chiếu thay vì xoá-sạch-ghi-lại**, tức lượt quét giữ được hàng mồ côi và cập nhật đường dẫn tại chỗ. Nghĩa **không** nhận: một lượt quét đọc `mtime` rồi bỏ qua thư mục *"chắc chưa đổi"* — đó là một bộ đệm suy đoán, và nó sai im lặng đúng vào ngày người dùng copy đè một `.atproj`.)* Quét lại vẫn đọc **mọi** `.atproj` trong gốc mỗi lượt; một watcher là một quyết định kiến trúc mới.
- **Không tự sửa/di chuyển/xoá bất kỳ `.atproj` nào.** `forget_orphan` xoá **một hàng chỉ mục**, không chạm đĩa một byte.
- **Không làm `meta.json` sống lại**, không bơm `work.updated_at`/`chapter.updated_at` — chủ là Story 5.5/5.6 (Ice chốt hẹp 2026-08-27).
- **Không mở `project.db` trong lượt quét** (AD-9) — chỉ đọc `meta.json`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| `.atproj` MỚI copy vào | Gốc có N+1 `.atproj`, chỉ mục có N | Quét lại ⇒ chỉ mục có N+1 hàng, hàng mới `orphaned = 0` | N/A |
| `.atproj` DI CHUYỂN trong gốc | Cùng `work_id`, đường dẫn khác | Đúng **một** hàng, `atproj_path` là đường mới, `orphaned = 0` | N/A |
| `.atproj` XOÁ / chuyển ra NGOÀI gốc | Hàng cũ trỏ tới đường dẫn không còn tồn tại | Hàng **ở lại**, `orphaned = 1`, `atproj_path` giữ nguyên (đó là *"nêu rõ nó trỏ tới đâu"*) | Không xoá im lặng |
| Mồ côi QUAY LẠI | `.atproj` được copy lại vào gốc | Cùng `work_id` ⇒ `orphaned` về 0, đường dẫn cập nhật, **không** tạo hàng thứ hai | N/A |
| Người dùng GỠ mục mồ côi | `forget_orphan(work_id)` trên hàng `orphaned = 1` | Hàng biến mất; trả danh sách mồ côi còn lại | N/A |
| Gỡ nhầm một hàng ĐANG SỐNG | `forget_orphan(work_id)` trên hàng `orphaned = 0` | **Từ chối**, 0 lượt xoá | `err.library.not_orphaned` |
| Gỡ một `work_id` không có | `work_id` lạ | **Từ chối**, cùng nhánh trên — không im lặng thành công | `err.library.not_orphaned` |
| `.atproj` hỏng nhưng CÒN nằm đó | Thiếu/hỏng `meta.json` | Vào `skipped` (đã có từ 5.2), **KHÔNG** thành mồ côi — thư mục vẫn tồn tại | Trả ra, không panic |
| Trùng `work_id` | Hai `.atproj` cùng UUID | Giữ mục đầu (thứ tự quét đã sắp), mục sau vào `conflicts`, có ít nhất một dòng chẩn đoán | Không gộp, không ghi đè |
| Đổi thư mục gốc | Người dùng chọn thư mục D qua hộp thoại | `AppConfig/library_root = D`, quét lại ngay trên D; hàng của gốc cũ thành **mồ côi** (đường dẫn cũ không nằm trong D) dù thư mục đó vẫn còn trên đĩa — chỉ mục nói về **thư viện**, không nói về đĩa | N/A |
| Huỷ hộp thoại | Người dùng bấm Cancel | `Ok(None)` — không ghi cấu hình, không quét, **không** một biến thể lỗi | N/A |
| Thư mục gốc vắng | Gốc (mặc định hoặc đã cấu hình) chưa tồn tại | `root_missing = true`; **mọi hàng sống thành mồ côi** (tập liệt kê được là rỗng), không xoá sạch bảng | Không tạo thư mục, không lỗi |
| Quét trên thư viện lớn | Người dùng bấm quét lại | Vỏ IPC chạy ngoài luồng chính; giao diện còn bấm/gõ được suốt lượt | N/A |
| Hai lượt quét chồng nhau | Khởi động + người dùng bấm gần như cùng lúc | Hai lượt **nối tiếp**; chỉ mục cuối cùng phản ánh lượt chạy sau, không phải một ảnh chụp trộn | N/A |

</intent-contract>

## Code Map

- `src-tauri/src/core/store/schema.rs:1459-1483` -- `LIBRARY_WORK_DDL` (bảng `library_work`, 8 cột) + `LIBRARY_INDEX_MIGRATIONS` (`to_version: 1`, **đúng một bước, mãi mãi**). Thêm cột ⇒ **viết lại chính bước đó** và bump `to_version` — kho dẫn xuất không di trú (AD-8), tuyệt đối không thêm bước 2. `:1403-1405` -- `target_version()`; `:592` -- `GLOBAL_MIGRATIONS` (đích v5) làm đối chiếu.
- `src-tauri/src/core/library/indexer.rs:60-79` -- `Indexer::open` (nhánh không-di-trú: `delete_if_schema_version_differs` rồi `Store::open`). `:99-176` -- `Indexer::rebuild`, **chỗ đổi ngữ nghĩa chính**: hôm nay `DELETE FROM library_work` + `INSERT` trong một `store.write`. `:178-192` -- `clear_for_missing_root` (xoá sạch khi gốc vắng — phải đổi thành *đánh dấu mồ côi*). `:196-219` -- `list_works` (đường đọc). `:236-283` -- `scan_atproj_dirs` + `ScanRootOutcome` (đã có nhánh `RootMissing` cho ca đua TOCTOU). `:300-330` -- `partition_dir_entries` (vị từ thuần, đã có 3 ca `#[cfg(test)]` tại chỗ). `:355-420` -- `RebuildOutcome` + `log_if_notable(surface)` (đường chẩn đoán CHUNG cho mọi chỗ gọi — thêm vế mồ côi vào đây, không dựng đường thứ hai). `:430-470` -- `WorkIdConflict`/`SkippedEntry`/`IndexedWork`. `:475-490` -- `IndexError` (`Io` · `Store`) + `Display` KHÔNG DẤU.
- `src-tauri/src/core/library/meta.rs:33` -- `META_SCHEMA_VERSION`; `:73-92` -- `WorkMeta` 7 trường; `:105-125` -- `WorkMeta::read` (từ chối `SchemaTooNew`).
- `src-tauri/src/core/library/atproj.rs:3-8` -- doc-comment: một `.atproj` sống mang **năm** mục trên đĩa (`-wal`/`-shm` là sidecar) ⇒ phép kiểm tồn tại phải hỏi **thư mục**, không đếm tên tệp.
- `src-tauri/src/commands/project.rs:106-127` -- `default_library_root(app)` = móc e2e ⇒ `document_dir()` + `DOCUMENTS_SUBFOLDER`. Doc-comment tự khai *"module này là nơi DUY NHẤT gọi hàm này"* — mệnh đề đó phải được sửa 🔵 tại chỗ khi bộ phân giải mới xuất ra ngoài. `:1498-1513` -- `reindex_after_create_work` (chỗ gọi `rebuild` thứ hai). `:308-318` -- `ImportScanGeneration` (`Arc<AtomicU64>`, `next`/`is_current`) và `:536-551` -- khuôn **thread nền + thế hệ** cho việc chạy ngoài luồng chính.
- `src-tauri/src/commands/glossary.rs:1310-1345` -- **khuôn để chép** cho vỏ hộp thoại: `#[tauri::command(async)]` + kiểm mọi tiền điều kiện **TRƯỚC** dialog + khoá state **LẦN THỨ HAI** sau khi dialog đóng + huỷ ⇒ `Ok(None)`. `:940` -- `use tauri_plugin_dialog::DialogExt as _;`.
- `~/.cargo/registry/src/*/tauri-plugin-dialog-2.7.2/src/lib.rs:723` -- `blocking_pick_folder(self) -> Option<FilePath>` (đã kiểm tồn tại 2026-08-27); crate đã có trong `Cargo.toml:60-70` (AD-48), **không thêm phụ thuộc nào**.
- `src-tauri/src/core/scope/store.rs:48-56,72-76,109-115` -- bốn khoá `AppConfig` đã có (`theme`/`mode`/`workspace_layout`/`dict_sources_disabled`/`glossary_scan_threshold`) kèm lý lẽ *"vì sao `AppConfig` chứ không một `ScopeKind` thứ mười"* — khoá thứ sáu chép đúng khuôn đó. `:118-130` -- khuôn getter-phân-giải-giá-trị-hỏng (**hàm thuần, đây là thứ test gọi**). `:295` -- `resolve_one(ScopeKind::AppConfig)`.
- `src-tauri/src/core/scope/kinds.rs:218` -- `AppConfig => "app_config" : GlobalOnly`. Không thêm `ScopeKind` mới.
- `src-tauri/src/commands/config.rs:150-166` -- `put_config(store, kind, key, value)` **đã tổng quát**: lưu thư mục gốc **không cần** một lệnh IPC ghi cấu hình mới.
- `src-tauri/src/core/i18n/mod.rs:62-90` -- `macro_rules! message_keys!` (một khai báo sinh `enum` + `ALL` + `as_str` + bảng tham số). `:99-105` -- `IoReadFailed => "err.io.read_failed" ["path"]` (tái dùng cho `IndexError::Io`). Khoá mới khai **cạnh** khoá của cùng tầng, kèm chú thích nói vì sao đúng bấy nhiêu khoá.
- `src-tauri/src/lib.rs:324-330` -- `generate_handler![...]` (nơi khai vỏ mới). `:699-745` -- `open_library_index` (chỗ gọi `rebuild` thứ nhất; nó gọi `default_library_root` — phải chuyển sang bộ phân giải mới, và nó **chưa có `Store`** tại thời điểm đó nên thứ tự `open_global_store` → `open_library_index` phải kiểm lại). `:75-99` -- `E2E_DATA_DIR_ENV`/`E2E_LIBRARY_ROOT_ENV`; `:146-152` -- `library_root_override()`.
- `src-tauri/tests/library_index_contract.rs` -- 19 ca của 5.2, gồm ca so **byte** `.atproj` trước/sau; ca *"xoá `library-index.db` rồi dựng lại ⇒ bằng nhau từng hàng"* là ca sẽ đổi nghĩa (§Design Notes).
- `src-tauri/tests/library_index_boundary.rs:136,253` -- vị từ lọc `//` dùng CHUNG cho cổng thật lẫn đối chứng (KEEP của 5.2 — không được quét toàn văn).
- `src-tauri/tests/config_invariants.rs:887-960` -- `the_blocking_wires_run_off_the_main_thread`: `:888` **viết cứng `src/commands/glossary.rs`**, mảng `cases: [(&str,&str,&str); 7]` chỉ mang tên vỏ. Một vỏ chặn ở tệp khác **không được canh** — phải đổi thành danh sách `(tệp, chữ ký, vì sao)` như ca `:1001-1014` ngay dưới đã làm.
- `src-tauri/tests/ipc_contract.rs` -- đóng băng tên trường trên dây; struct wire mới phải vào đây **cùng lượt**.
- `src/commands/index.ts:806-828` -- `library.import_text`/`library.import_file` (khuôn `deps.<port>` tiêm vào + `portMissing`). `src/commands/registry.ts` -- `COMMAND_ID_RE` (không chữ hoa). `src/main.ts:315-327` -- `installCommands({...})`, nơi nối port mới.
- `src/glossaryManageState.ts:83,144,193-197,356-366,448` -- **khuôn con trỏ danh sách**: `cursor` riêng tư + `readonly` export + `.at(cursor)` + `next`/`prev` + kẹp lại sau khi danh sách đổi. `src/GlossaryManageOverlay.vue:561-580` -- khuôn nút `@click="dispatch('<id>')"` với `:disabled="<row> === null"`.
- `src/modes/LibraryMode.vue:63-150` -- khung có sẵn: `.empty` + `form.import-form` + ba node `role="status"` LUÔN có mặt; `:170-200` -- CSS dùng token (`--space-panel-block`, …). `src/modes/libraryImport.ts:1-18` -- lý do một module thuần riêng thay vì viết trong `.vue`.
- `src/config/project.ts` -- khuôn adapter ba trạng thái (adapter thứ 8 chép đúng khuôn này). `src/i18n/vi.json:55-56,124-137` -- khoá `command.library.*` và `mode.library.*`.
- `_bmad-output/implementation-artifacts/deferred-work.md:8079-8091` -- món nợ **"hai lượt `rebuild` chồng nhau"**, chủ ghi đích danh **Story 5.3**; `:8060-8077` -- món nợ "đoạn nối không ca nào chạm", chủ Story 5.6 (**không** đóng ở đây, chỉ nối tiếp nếu có gì đổi).
- `src-tauri/AGENTS.md:29` -- luật `library-index.db`/`meta.json` là dẫn xuất + hai giới hạn đang mở; phải cập nhật 🔵 sau khi ngữ nghĩa mồ côi ra đời.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store/schema.rs` -- viết lại `LIBRARY_WORK_DDL` thêm cột `orphaned INTEGER NOT NULL DEFAULT 0` và bump `LIBRARY_INDEX_MIGRATIONS` lên `to_version: 2` (vẫn **một** bước) -- kho dẫn xuất không di trú: bump = viết lại bước đó, và `Indexer::open` tự xoá-dựng-lại.
- [x] `src-tauri/src/core/library/indexer.rs` -- đổi `rebuild` thành **đối chiếu trong một giao dịch**: upsert mọi mục đọc được (`atproj_path` mới, `orphaned = 0`), rồi đánh dấu `orphaned = 1` cho mọi hàng còn lại mà `atproj_path` **không nằm trong tập `.atproj` vừa liệt kê được** (hai vế, xem §Always); `clear_for_missing_root` đổi thành `mark_all_orphaned_for_missing_root`; thêm `RebuildOutcome::orphans` và vế mồ côi vào `log_if_notable`; thêm `forget_orphan(work_id)` (từ chối khi hàng không tồn tại hoặc `orphaned = 0`) và `list_orphans()`; bọc trọn scan+ghi bằng một `Mutex` khoá theo lệ kho (`unwrap_or_else(PoisonError::into_inner)`) -- vừa là AC3, vừa là **món nợ `deferred-work.md:8079` mà story này là chủ**.
- [x] `src-tauri/src/core/scope/store.rs` -- thêm khoá `AppConfig` thứ sáu `library_root` + getter **hàm thuần** phân giải giá trị thô (rỗng/chỉ khoảng trắng ⇒ coi như chưa cấu hình) -- chép khuôn `glossary_scan_threshold`, vì đây là chỗ DUY NHẤT biết một giá trị trên đĩa hỏng.
- [x] `src-tauri/src/commands/project.rs` -- thêm `resolve_library_root(app, store: Option<&Store>)` = móc e2e ⇒ giá trị đã cấu hình ⇒ `default_library_root`; sửa 🔵 tại chỗ mệnh đề *"module này là nơi DUY NHẤT gọi hàm này"* cho nó nói đúng; đổi `reindex_after_create_work` sang bộ phân giải mới -- một Tác phẩm mới phải sinh ra trong đúng thư mục người dùng đã chọn, nếu không AC5 mở ra một chỗ rỗng im lặng thứ hai.
- [x] `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` -- thêm **đúng hai** `MessageKey`: `LibraryNotOrphaned => "err.library.not_orphaned" ["work_id"]` và `LibraryRootInvalid => "err.library.root_invalid" []`; thêm mọi khoá `command.library.*`/`mode.library.*` mới -- danh mục đóng, và `IndexError::Io` tái dùng `IoReadFailed` thay vì đúc khoá thứ ba.
- [x] `src-tauri/src/commands/library.rs` -- **tệp mới**, khuôn hai lớp: hàm thuần (`rescan`, `forget_orphan`) nhận `Option<&Indexer>` + `&Path`, và `mod wire` với `library_rescan` · `library_choose_root` · `library_forget_orphan`, tất cả `#[tauri::command(async)]`, `try_state` (không `state()`), huỷ hộp thoại ⇒ `Ok(None)` -- vỏ chặn trên luồng chính là bế tắc đã ĐO được ở Story 3.10b, không phải một rủi ro lý thuyết.
- [x] `src-tauri/src/commands/mod.rs` + `src-tauri/src/lib.rs` -- khai `pub mod library;`, thêm ba vỏ vào `generate_handler![...]`, và đổi `open_library_index` sang `resolve_library_root` -- lượt quét lúc khởi động phải nhìn cùng một thư mục gốc mà người dùng bấm quét lại nhìn.
- [x] `src-tauri/tests/library_index_contract.rs` -- phủ TRỌN §I/O Matrix ở tầng `Indexer`: mới/di chuyển/xoá/quay lại/gỡ/gỡ nhầm/hỏng-mà-còn-đó/gốc vắng, cộng ca **hai lượt `rebuild` chạy từ hai luồng** cho ra một trạng thái nhất quán -- món nợ đóng bằng phép đo, không bằng một lời hứa trong chú thích.
- [x] `src-tauri/tests/library_commands_contract.rs` -- **tệp mới** (vòng rà, không phải bản dựng đầu): phủ §I/O Matrix ở tầng LỆNH (`commands::library`) mà `library_index_contract.rs` không với tới -- "`.atproj` MỚI copy vào" qua chính tầng lệnh, "Huỷ hộp thoại"/"Đổi thư mục gốc" qua hàm thuần MỚI `apply_chosen_root(store, indexer, picked: Option<&Path>)` (tách khỏi `wire::library_choose_root` để nhánh huỷ có một ca chạy được mà không cần cửa sổ thật), `Indexer` vắng mặt ⇒ `IpcError` không panic, `root_missing` phân biệt gốc vắng/gốc rỗng thật (P1), và `From<IndexError> for IpcError` đi qua đúng tầng lệnh với `code`/`params` đúng (P6).
- [x] `src-tauri/tests/config_invariants.rs` -- đổi `the_blocking_wires_run_off_the_main_thread` thành danh sách `(tệp, chữ ký, vì sao)` và thêm ba vỏ mới; thêm một ca khẳng định cổng này **đọc nhiều hơn một tệp** -- một cổng viết cứng một đường dẫn là một cổng mù với đúng tệp story này vừa tạo.
- [x] `src-tauri/tests/ipc_contract.rs` -- đóng băng tên trường của mọi struct wire mới (`snake_case`, **không** `rename_all`) -- một trường mới qua IPC mà không ai đối chiếu là đúng thứ ca này tồn tại để chặn.
- [x] `src/config/library.ts` -- **tệp mới**, adapter IPC thứ tám: `invoke` + `try/catch` + hình dạng ba trạng thái, type guard lúc chạy cho dữ liệu qua dây -- `snake_case` giữ nguyên ở chiều TRẢ VỀ, `camelCase` ở chiều gửi đi (`workId`).
- [x] `src/modes/libraryRescan.ts` -- **tệp mới**: `libraryRoot` · `orphans` · `orphanCursor`/`currentOrphan` · `conflictCount`/`skippedCount` · `rescanBusy` · `libraryScanHasLoaded` · `lastError`, cùng `rescan()`, `chooseRoot()`, `forgetOrphan()`, `orphanNext()`/`orphanPrev()` -- vị từ `…HasLoaded` là điều kiện để màn hình được phép nói *"không có mục mồ côi nào"*.
- [x] `src/commands/index.ts` + `src/main.ts` -- đăng ký `library.rescan` (**có phím**, AC7) · `library.choose_root` · `library.forget_orphan` · `library.orphan_next` · `library.orphan_prev` qua `deps` tiêm vào, kèm `portMissing` -- đăng ký ở `main.ts` chứ không trong `.vue`, nếu không `check:commands` Kiểm B không thấy id nào.
- [x] `src/modes/LibraryMode.vue` -- thêm khối "thư mục gốc + quét lại + mục mồ côi": mỗi `@click` là **đúng một** `dispatch('<id>')`, mọi nhãn qua `t()`, node kết quả `role="status"` LUÔN có mặt, chỉ dùng token màu/cỡ chữ -- AD-34 §1 và `check:tokens` Kiểm B/B2 đều đọc tĩnh tệp này.
- [x] `tests/frontend/libraryRescan.test.ts` -- **tệp mới** (đặt `.test.ts`, không `.spec.ts` -- xem §Spec Change Log): adapter không ném trên lỗi IPC, `libraryScanHasLoaded` sai trước lượt gọi đầu, con trỏ mồ côi kẹp lại đúng sau khi một mục bị gỡ, `forgetOrphan` không gọi IPC khi chưa chọn mục nào -- `happy-dom` chỉ được canh hành vi module thuần, không canh hình học.
- [x] `src-tauri/AGENTS.md` + `_bmad-output/implementation-artifacts/deferred-work.md` -- sửa 🔵 dòng 29 (chỉ mục nay giữ hàng mồ côi, tức nó **không còn** dẫn xuất trọn vẹn từ đĩa) và nối tiếp món nợ `:8079` bằng `→ ✅ ĐÃ ĐÓNG 2026-08-27 (Story 5.3)` kèm cách đóng; ghi nợ mới có chủ cho vế hiển thị còn thiếu (bề mặt cảnh báo trùng `work_id` ⇒ 5.6; đường ĐỌC thuần thay cho lượt quét lúc mở Library ⇒ 5.6) -- không mục nào mồ côi, không mục cũ bị xoá.

**Acceptance Criteria:**
- Given một chỉ mục có N Tác phẩm và người dùng bấm **quét lại**, when giao diện đang xử lý lượt quét, then cửa sổ vẫn nhận thao tác (vỏ IPC chạy ngoài luồng chính, có cổng canh), và khi xong màn hình nói đủ **ba** con số phân biệt được: đã lập chỉ mục · trùng `work_id` · bỏ qua.
- Given thao tác quét lại, when đọc bộ đăng ký command, then `library.rescan` là một id đã đăng ký kèm hợp âm phím mặc định, và `check:commands` Kiểm A/B xanh trên mọi `@click` mới.
- Given một `.atproj` bị xoá khỏi thư mục gốc, when quét lại rồi khởi động lại ứng dụng, then hàng đó vẫn ở đó, vẫn mang cờ mồ côi và vẫn nêu đường dẫn cũ — người dùng phải **chủ động** gỡ nó mới mất.
- Given người dùng đổi thư mục gốc qua hộp thoại, when tạo một Tác phẩm mới sau đó, then `.atproj` mới nằm trong thư mục vừa chọn, không nằm trong `~/Documents/AuraTranslate/`.
- Given bộ test cũ (`library_index_contract`, `library_index_boundary`, `store_contract`, `project_contract`, `naming_boundary`, `scope_contract`, `segment_contract`, bộ frontend), when chạy sau story này, then xanh — trừ những ca của 5.2 mà ngữ nghĩa mồ côi **cố ý** đổi nghĩa, và mỗi ca như vậy được sửa **kèm một dòng nói vì sao**, không bị xoá.
- Given cổng `library_index_boundary`, when chạy sau khi bề mặt IPC mới ra đời, then nó vẫn xanh **mà không** thêm một đường dẫn nào vào danh sách miễn trừ.

## Spec Change Log

- **Vòng dựng đầu — hai lệch tường tự sửa, không hoàn nguyên.**
  **① Tên tệp test frontend sai đuôi.** §Code Map/§Tasks đặt tên `tests/frontend/libraryRescan.spec.ts`, nhưng `vitest.config.ts:60` chỉ nạp `tests/frontend/**/*.test.ts` — mọi 43 tệp có sẵn trong cây đều mang đuôi `.test.ts`, không `.spec.ts`. Một tệp `.spec.ts` sẽ biên dịch sạch, không ai gọi nó, và `npm run test` báo "44 tệp" giả trong khi thực tế vẫn 43 tệp CHẠY — đúng lớp "một bộ test xanh không chứng minh chỗ nối được canh" mà `AGENTS.md::Known pitfalls` cấm. Đặt tên `libraryRescan.test.ts` theo cây nguồn thật (`vitest.config.ts` thắng, đúng luật "cây nguồn thắng, báo lại chỗ lệch" của chính `_bmad-output/project-context.md`). Đo sau khi sửa: `npm run test` → 44 tệp / 573 ca, đúng bằng baseline 43/567 cộng đúng 1 tệp/6 ca mới.
  **② `check:debt-owner` đỏ từ TRƯỚC baseline, không do story này.** Đối chứng: `git diff <baseline>..HEAD -- deferred-work.md` trước khi story này chạm gì vào tệp đó là RỖNG, và `npm run check:debt-owner` đã đỏ đúng ở `deferred-work.md:8017` (mục "22 cảnh báo clippy" từ Story 5.1, thiếu `Chủ:`) — một món nợ mồ côi từ trước, không phải hệ quả của story này. `§Verification` của story này đòi cổng đó xanh (nó nằm trong chuỗi lệnh bắt buộc), và sửa nó là một dòng thêm siêu nhỏ, không đổi nội dung mục cũ — thêm `**Chủ: Ice — quyết định có đưa clippy vào một cổng hay không**`, đúng khuôn "Ice là người chốt các quyết định mở". Xếp `patch` chứ không `bad_spec`: sửa TẠI CHỖ một khoảng trống trong tài liệu, không đụng một dòng mã hay một quyết định kiến trúc nào.
  **KEEP — phải sống sót mọi lần dựng lại:** (1) `Đối chứng bắt buộc` của §Verification phải THẬT SỰ chạy — cả hai (gỡ vế đánh dấu mồ côi; gỡ `(async)`) đã chạy tay, đỏ đúng chỗ, rồi khôi phục; (2) vị từ mồ côi giữ ĐÚNG hai vế (không đọc được VÀ không nằm trong tập liệt kê được) — biến thể một-vế đã bị loại tường minh ở §Design Notes, đừng đơn giản hoá lại; (3) `Indexer::rebuild_lock` khoá TOÀN BỘ scan+ghi, không chỉ giai đoạn ghi — đây là điểm mấu chốt đóng món nợ `:8079`, một Mutex chỉ bọc `store.write` sẽ không đóng được nó; (4) `commands/library.rs` KHÔNG được nhắc `StoreKind::LibraryIndex`/`StoreSpec::library_index` ở vị trí MÃ (comment thì được) — cổng ranh giới `library_index_boundary.rs` canh đúng việc đó.

- **Soát phủ §I/O Matrix sau lượt dựng — hai hàng không có ca nào, cả hai đã đóng bằng mã, không bằng cách sửa kỳ vọng.**
  **① *"Huỷ hộp thoại"* là một nhánh KHÔNG CA NÀO CHẠY ĐƯỢC.** Bản dựng đầu để nguyên nhánh huỷ inline trong `wire::library_choose_root`, mà `blocking_pick_folder()` cần một cửa sổ thật — nên hàng ma trận *"`Ok(None)`, không ghi cấu hình, không quét"* có kỳ vọng mà không có phép đo. Đã tách một **hàm thuần** `commands::library::apply_chosen_root(store, indexer, picked: Option<&Path>)` — vỏ không còn tự quyết định gì, cả nhánh huỷ lẫn nhánh chọn thật đi qua cùng một hàm. Đúng cùng lý lẽ mà `partition_dir_entries` đã tách khỏi `scan_atproj_dirs` ở Story 5.2, và đúng khuôn hai lớp mà `src-tauri/AGENTS.md` đặt ra. **Đối chứng đã CHẠY:** nhét một `put_config` vào nhánh huỷ ⇒ đúng một ca đỏ (`cancelling_the_folder_dialog_writes_no_config_and_leaves_the_index_alone`), bốn ca kia vẫn xanh; đã khôi phục.
  **② Hàng ĐẦU TIÊN của ma trận — *"`.atproj` MỚI copy vào"*, tức câu chuyện người dùng nguyên văn của FR99 — cũng không có ca nào.** `an_orphan_that_reappears_...` gần giống nhưng khác nhánh SQL: ở đó `work_id` đã có trong bảng (UPSERT đi nhánh `DO UPDATE`), còn một Tác phẩm vừa copy vào là một `work_id` bảng CHƯA từng thấy (nhánh `INSERT`). Đã thêm ca đi qua chính tầng lệnh.
  ⇒ Cả hai ca sống ở **tệp mới `src-tauri/tests/library_commands_contract.rs`** (5 ca): hai hàng trên, cộng vế *"đổi thư mục gốc"* ở tầng cấu hình (lựa chọn phải XUỐNG ĐĨA, thứ quyết định lần khởi động sau), đường dẫn không phải thư mục bị từ chối **trước** khi ghi, và `Indexer` vắng mặt ⇒ `IpcError` chứ không panic. Tệp riêng chứ không nhét vào `library_index_contract.rs`: hai vai khác nhau (`Indexer` so với `commands::library`), đúng cách `store_contract.rs`/`project_contract.rs` đã tách.
  **③ Hai con số trong §Verification của chính spec này SAI, sửa tại chỗ thay vì để chúng lặng lẽ sai.** `cargo tree --locked -e normal` cho **860** dòng, không phải 831 — đối chứng: `Cargo.toml`/`Cargo.lock` không đổi một byte ở story này (`git status`), nên 860 là hiện trạng có TRƯỚC story, và con số 831 chép từ tài liệu Story 5.2 đã lỗi thời. Mệnh đề đáng giữ vẫn đúng và đã đo: **story này thêm 0 phụ thuộc**. Lệnh `grep` cho `StoreSpec::library_index` nay khớp **năm** tệp, không ba — hai tệp mới (`core/i18n/mod.rs`, `commands/library.rs`) chỉ khớp ở dòng `//`. Đây đúng là cái bẫy mà §Spec Change Log của Story 5.2 đã gọi tên (*"`grep` không phân biệt mã với chú thích"*), và tôi chép lại kỳ vọng hỏng đó vào spec này; phán quyết thuộc về `cargo test --test library_index_boundary` (có lọc `//`), thứ đang xanh với đúng hai tệp ở vị trí MÃ.
  **KEEP:** (5) nhánh huỷ hộp thoại phải ở lại trong một hàm THUẦN nhận `Option<&Path>` — kéo nó về inline trong vỏ là làm hàng ma trận đó mất phép đo lần nữa; (6) `apply_chosen_root` ghi cấu hình TRƯỚC rồi mới quét: một lượt quét trượt vẫn phải để lại lựa chọn của người dùng trên đĩa.

## Review Triage Log

### 2026-08-27 — Review pass

- intent_gap: 0
- bad_spec: 0
- patch: 9: (high 2, medium 4, low 3)
- defer: 3: (high 0, medium 1, low 2)
- reject: 4: (high 0, medium 0, low 4)
- addressed_findings:
  - `[high]` `[patch]` `RescanReport` đánh rơi `root_missing` mà `RebuildOutcome` đã tính — màn hình nhận `indexed: 0` và không có cách nào phân biệt *"thư mục gốc không còn ở đó"* với *"gốc rỗng thật"*, đúng lớp "rỗng im lặng" mà §Always cấm. Đã nối `root_missing` xuyên Rust → `ipc_contract.rs` → `config/library.ts` (type guard lúc chạy) → `libraryRescan.ts` → một câu trạng thái RIÊNG ở `LibraryMode.vue`. Đối chứng GỠ đã chạy ở CẢ HAI tầng (ghim cứng `false` ở Rust, rồi ở TS) — mỗi lượt đúng một ca đỏ.
  - `[high]` `[patch]` `LibraryMode.vue` khẳng định *"Mặc định — ~/Documents/AuraTranslate/"* mỗi khi `currentLibraryRoot === null`, trong khi `null` chỉ nghĩa là *"chưa quét lần nào trong phiên này"* — một người dùng đã cấu hình gốc riêng từ phiên trước thấy màn hình nói sai về thư mục của họ. Chuỗi đó còn chép cứng một đường dẫn mà `default_library_root` cố ý không viết cứng (NFR14) và không biểu diễn được ca `document_dir()` trượt. Đổi sang `mode.library.root_not_scanned_yet` — một câu nói đúng trạng thái CHƯA BIẾT; đường dẫn thật chỉ đến từ `RescanReport.root` do Rust phân giải.
  - `[medium]` `[patch]` Vị từ mồ côi vế HAI hỏi *"đường dẫn có được liệt kê không"* thay vì *"có được liệt kê mà KHÔNG đọc được không"*: một `.atproj` bị xoá rồi bị một Tác phẩm KHÁC chiếm đúng tên thư mục để lại hàng cũ như một hàng ĐANG SỐNG trỏ vào nội dung của người khác. Dựng lại tập vế-hai từ nhánh `Err` của `WorkMeta::read` (`unreadable_paths`). Đối chứng GỠ đã chạy, đỏ đúng ca `a_path_reclaimed_by_a_different_work_orphans_the_old_occupants_row`.
  - `[medium]` `[patch]` Thứ tự ưu tiên của `resolve_library_root` (móc e2e ⇒ cấu hình ⇒ mặc định) được doc-comment gọi là bất biến nhưng **không gì canh** — đảo hai khối `if` vẫn xanh toàn bộ, vì `library_root_override()` trả `None` ở bản dựng test thường (feature `wdio` tắt) nên nhánh đó không chạy được. Dựng một cổng QUÉT NGUỒN ở `config_invariants.rs` khẳng định thứ tự byte của ba lời gọi, kèm đối chứng dương/âm trên chuỗi dựng tay.
  - `[medium]` `[patch]` `impl From<IndexError> for IpcError` không ca nào đi qua ở tầng lệnh: mọi ca bắt thẳng biến thể `IndexError`, còn ca frontend hardcode một `IpcError` giả. Đổi chuỗi `code` hay hoán hai nhánh `match` đều không làm ca nào đỏ, mà hậu quả tới người dùng là `err.unknown` thay cho câu đã dịch. Thêm ca gọi `commands::library::forget_orphan` khẳng định `code()` và `params()["work_id"]`.
  - `[medium]` `[patch]` Hai nút điều hướng mồ côi `‹`/`›` là ký tự trần, không `aria-label`, không `t()` — NFR17, và không cổng nào canh chỗ này. Thêm nhãn qua `t()` theo khuôn `GlossaryQueueOverlay.vue`.
  - `[low]` `[patch]` `resolve_library_root` nuốt lỗi `load_global_config` bằng `if let Ok(..)`: một `global.db` hỏng làm ứng dụng lặng lẽ về gốc mặc định, không dấu vết. Thêm chẩn đoán KHÔNG DẤU trước khi rơi về mặc định.
  - `[low]` `[patch]` `resolve_library_root_value` tự khai *"hàm thuần, đây là thứ test gọi"* và *"chép khuôn `parse_glossary_scan_threshold`"* mà không ca nào gọi. Thêm 5 ca (None · rỗng · chỉ khoảng trắng · giá trị thật · giá trị đã sạch) đặt cạnh hàm anh em.
  - `[low]` `[patch]` `wire::library_choose_root` kiểm `Indexer` trước hộp thoại theo khuôn P1 nhưng bỏ `Store` — người dùng duyệt xong thư mục rồi mới nhận lỗi, đúng thứ khuôn P1 tồn tại để tránh. Thêm `Store` vào cùng khối tiền điều kiện.


- **Vòng rà bốn lớp — chín bản vá, tất cả `patch`, không `intent_gap`/`bad_spec`.**
  **P1 [nặng] — `RescanReport` đánh rơi `root_missing`.** `commands::library::rescan` vứt `RebuildOutcome::root_missing` (đã tính đúng ở tầng `Indexer`) khi gói kết quả — đúng lớp "rỗng im lặng". Thêm `RescanReport::root_missing: bool`, đóng băng ở `ipc_contract.rs`, plumb qua `src/config/library.ts` (kèm `isRescanReport` — guard KIỂU LÚC CHẠY mới, đúng luật `src/AGENTS.md`) và `src/modes/libraryRescan.ts` (`libraryRootMissing`), `LibraryMode.vue` hiện một node `.root-missing` (role="status", LUÔN có mặt) riêng. Hai ca mới ở `library_commands_contract.rs` (gốc vắng mặt vs. gốc rỗng thật) + hai ca frontend (`libraryRescan.test.ts`) + một ca frontend cho guard kiểu. **Đối chứng GỠ đã chạy:** vứt `root_missing` ở cả hai tầng (Rust: gán `false` cứng; TS: gán `false` cứng) ⇒ đúng những ca MỚI đỏ, khôi phục.
  **P2 [nặng] — `LibraryMode.vue` khẳng định một điều chưa biết.** `currentLibraryRoot ?? t('mode.library.root_default')` với chuỗi chép cứng `~/Documents/AuraTranslate/` — sai khi người dùng đã cấu hình gốc riêng từ phiên trước, và chép cứng `$HOME` đúng thứ `default_library_root` cố ý tránh. Đổi khoá thành `mode.library.root_not_scanned_yet` ("Chưa quét lần nào trong phiên này…"), xoá hẳn đường dẫn viết cứng. Đường DUY NHẤT hiện đường dẫn thật vẫn là `RescanReport.root`.
  **P3 [vừa] — vị từ mồ côi vế HAI quá rộng: đường dẫn bị Tác phẩm KHÁC chiếm để lại hàng sống nói dối.** Xem §Design Notes (sửa tại chỗ, đánh dấu 🔵 — "bốn cách viết, ba cách sai"). Gốc rễ: tập chặn mồ côi dựng từ TOÀN BỘ `scan.dirs` (mọi `.atproj` liệt kê được), không phải từ tập ĐỌC-KHÔNG-ĐƯỢC. Sửa: `unreadable_paths` dựng từ nhánh `Err` của `WorkMeta::read` trong CHÍNH vòng lặp — một đường dẫn ĐỌC ĐƯỢC luôn xác nhận nó thuộc về ĐÚNG `work_id` vừa khai, không phải một tấm khiên chung. Ca mới `a_path_reclaimed_by_a_different_work_orphans_the_old_occupants_row`. **Đối chứng GỠ đã chạy:** khôi phục tập cũ (toàn bộ `scan.dirs`) ⇒ đúng ca mới đỏ ("hàng CŨ của A phải thành mồ côi… left: 0 right: 1"), khôi phục.
  **P4 [nhẹ] — `resolve_library_root` nuốt lỗi đọc cấu hình.** `if let Ok(config) = ..` bỏ im nhánh `Err`. Thêm `eprintln!` KHÔNG DẤU trước khi rơi về mặc định — vế "đi tiếp" không đổi, chỉ thêm vế "nói ra".
  **P5 [vừa] — thứ tự ưu tiên không có gì canh.** Móc e2e trả `None` trong `cargo test` thường nên không dựng được một ca HÀNH VI cho nhánh đó. Thêm cổng QUÉT NGUỒN ở `config_invariants.rs` (`resolve_library_root_checks_the_e2e_override_before_the_configured_value_before_the_default`), so vị trí BYTE của ba lời gọi trong đúng thân hàm, cộng đối chứng dương/âm trên chuỗi dựng tay (`the_order_predicate_would_actually_flag_a_reversed_order` / `..._does_not_flag_the_correct_order`).
  **P6 [vừa] — `From<IndexError> for IpcError` không ca nào đi qua ở TẦNG LỆNH.** Mọi ca hiện có bắt thẳng `IndexError` từ `Indexer::*`. Thêm hai ca ở `library_commands_contract.rs` gọi `commands::library::forget_orphan` (không phải `Indexer::forget_orphan`), khẳng định CẢ `err.code()` LẪN `err.params()["work_id"]`.
  **P7 [nhẹ] — `resolve_library_root_value` không ca nào.** Thêm 5 ca ở `glossary_scan_contract.rs` (cùng tệp với `parse_glossary_scan_threshold`, theo đúng chỉ dẫn — KHÔNG phải một hàng của §I/O Matrix Story 3.5 mà tệp đó sở hữu, chỉ mượn cùng mái nhà).
  **P8 [nhẹ] — `wire::library_choose_root` không kiểm `Store` trước hộp thoại.** Thêm phép kiểm `Store` vào CÙNG khối tiền điều kiện với `Indexer`, trước `blocking_pick_folder()`.
  **P9 [vừa] — hai nút điều hướng mồ côi không có nhãn cho trình đọc màn hình.** Thêm `:aria-label="t('mode.library.orphan_prev'|'orphan_next')"` cho cả hai nút, hai khoá `vi.json` mới.
  ⇒ `cargo test --locked`: 792 → **805** ca (13 ca mới, đúng số học: 1 + 4 + 3 + 5). `npm run test`: 573 → **576** ca (3 ca mới). Chín cổng `check:*` (`commands` `i18n` `tokens` `gates` `debt-owner` `deps` `layout` `panel-refs` `dict-manifest`) xanh. `npm run build` xanh.

- **Vòng rà THỨ HAI — mười một bản vá, tất cả `patch`, chạy trên diff đầy đủ `c533c62..HEAD` gồm cả bàn đo e2e.**
  **Chủ đề xuyên suốt:** bốn mục là tài liệu/phép đo nói sai về chính mã, ba trong số đó là ca "viết ghi chú VỀ chỗ sai thay vì SỬA chỗ sai" — sửa NGUỒN ở cả bốn, không thêm ghi chú thứ hai.
  **P1 [NẶNG] — huỷ hộp thoại chọn thư mục báo `err.unknown`.** `chooseLibraryRoot()` dùng chung `callRescan()` với `rescanLibrary()`; `library_choose_root` trả `Option<RescanReport>` nên huỷ ⇒ `null` trên dây ⇒ `isRescanReport(null)` (đúng vai của nó cho `library_rescan`) từ chối `null` ⇒ `err.unknown` đi tới người dùng, ngược §I/O Matrix. Tách `callChooseRoot` xử lý `null` làm ca HUỶ TRƯỚC khi chạm `isRescanReport`. Thêm ca adapter (`invoke` trả `null` ⇒ ba trạng thái rỗng) và ca state (`chooseLibraryRootFolder()` sau một lượt quét thành công giữ nguyên toàn bộ state). **Đối chứng GỠ đã chạy:** khôi phục đường dùng chung `callRescan` ⇒ đúng hai ca mới đỏ, tái hiện nguyên văn `err.unknown`; khôi phục.
  **P2 [NẶNG] — `resolve_library_root` không một phép kiểm HÀNH VI nào.** Cổng vòng một chỉ so THỨ TỰ CHUỖI trong mã nguồn, không bao giờ GỌI hàm — ba nhánh (giá trị đã cấu hình, `load_global_config` lỗi, `store = None`) chưa từng chạy, và bộ e2e cũng không với tới vì móc override luôn có mặt. Tách hai hàm THUẦN: `resolve_library_root_from(override, configured, default: closure)` và `resolve_configured_library_root(store) -> Option<String>` — cả hai không cần `AppHandle`. Thêm 8 ca hành vi (override thắng cấu hình, cấu hình thắng mặc định, mặc định chỉ chạy khi cả hai vắng, lỗi từ default được truyền nguyên vẹn, `store = None`, kho mới chưa cấu hình gì, giá trị đã lưu, đọc trượt qua `ReaderPool::close()` rơi về "chưa cấu hình"). Cổng quét nguồn CŨ vẫn giữ, sửa needle theo hình dạng vỏ mới (`resolve_configured_library_root(store)` thay `load_global_config(store)`) — nó canh THỨ TỰ TRONG VỎ, việc khác với hành vi. **Đối chứng GỠ đã chạy:** đảo nhánh `store = None` cho nó trả một giá trị giả thay vì `None` ⇒ đúng ca mới đỏ; khôi phục.
  **P3 [vừa] — `rebuild_lock` chỉ bọc `rebuild`; ba phát hiện, một gốc.** `rescan`/`forget_orphan` ở tầng lệnh gọi `list_orphans()` RIÊNG, KHÔNG khoá, sau khi `rebuild`/`forget_orphan` đã commit — một lượt quét khác chen vào giữa làm `orphans` phản ánh thế hệ khác với `indexed`/`conflicts`/`skipped`; `Indexer::forget_orphan` không lấy khoá nên một `rebuild` chen ngang có thể lật cờ giữa chừng; và một `list_orphans()` trượt sau khi ghi đã xong biến một thao tác ĐÃ THÀNH CÔNG thành một lỗi khó hiểu. Một bản vá đóng cả ba: `RebuildOutcome::current_orphans` và trị trả về mới của `Indexer::forget_orphan` (`Vec<IndexedWork>`) đều là ẢNH CHỤP lấy TRONG cùng phạm vi khoá; `forget_orphan` nay lấy `rebuild_lock`. `commands::library` dùng thẳng ảnh chụp, không gọi `list_orphans()` lần hai. Thêm ca đồng thời (`forget_orphan_running_alongside_concurrent_rebuilds_never_reports_a_false_not_orphaned`).
  **P4 [vừa] — §Verification mang hai con số mà §Spec Change Log đã đo là sai.** Dòng "ba tệp"/"831" chưa từng được sửa dù mục ③ ở trên đã đo ra năm tệp/860. Sửa TẠI CHỖ kèm 🔵, giữ nguyên Change Log làm lịch sử.
  **P5 [vừa] — doc-comment của `LIBRARY_WORK_DDL` khai một ngữ nghĩa đã bị xoá.** `PRIMARY KEY` không còn là "lưới chắn trùng" từ khi `rebuild` chuyển sang UPSERT — một work_id trùng lọt tới SQL nay ghi đè êm ái, không nổ lỗi ràng buộc. Sửa để nói thẳng VẾ ĐÃ MẤT: phát hiện trùng nay hoàn toàn ở tầng Rust (`first_seen`), trước khi chạm SQL.
  **P6 [vừa] — hai mệnh đề tự viết về độ phủ SAI vì viết trước khi dựng bàn đo e2e, không quay lại sửa.** `e2e/specs/story-5-3-rescan.e2e.mjs` (6 ca) chạm HAI trong BA vỏ; `ci.yml:712` CÓ job `e2e` ở nhịp `schedule`+`workflow_dispatch`. Sửa ba chỗ: frontmatter `deferred`, §Rủi ro còn lại mục 2, và nối tiếp (không viết đè) mục tương ứng ở `deferred-work.md`.
  **P7 [nhẹ]** — gỡ tiêu đề `## Review Triage Log` rỗng thứ hai, sót lại từ khuôn (do chính tôi tạo ra ở vòng rà trước khi nối `addition = anchor + ...` với anchor đã có sẵn heading đó).
  **P8 [nhẹ]** — thêm `role="status"` cho `.root-value`, khớp ba node hàng xóm.
  **P9 [nhẹ]** — `err.library.not_orphaned` thêm tham số `name` (đã có sẵn ở chỗ gọi — frontend đang hiển thị nó) cạnh `work_id`; `commands::library::forget_orphan` nhận `name: &str`, tự dựng `IpcError` cho ca `NotOrphaned` thay vì đi qua `From<IndexError>` chung (`Indexer` không biết "tên đang hiển thị").
  **P10 [nhẹ]** — thêm hai ca cho cửa chống tái nhập (`rescanBusy`) và cửa bỏ kết quả cũ (`mySequence !== sequence`, kích hoạt qua `resetLibraryRescan()` giữa một lượt IPC đang bay).
  **P11 [nhẹ]** — thêm ca `apply_chosen_root` với `store: None` ⇒ `store.open_failed`, không panic, không bỏ qua im lặng bước ghi cấu hình.
  ⇒ `cargo test --locked`: 805 → **815** ca. `npm run test`: 576 → **580** ca. Chín cổng `check:*` + `check:lint` xanh. `npm run build` xanh.

### 2026-08-27 — Review pass (vòng HAI, trên diff đã commit `c533c62..HEAD` gồm cả bàn đo e2e)

- intent_gap: 0
- bad_spec: 0
- patch: 11: (high 2, medium 4, low 5)
- defer: 4: (high 0, medium 2, low 2)
- reject: 5: (high 0, medium 0, low 5)
- addressed_findings:
  - `[high]` `[patch]` **Huỷ hộp thoại chọn thư mục báo `err.unknown` — một lỗi THẬT, đi tới màn hình.** `chooseLibraryRoot()` dùng chung `callRescan()` với `rescanLibrary()`, nhưng `library_choose_root` trả `Ok(None)` khi huỷ ⇒ trên dây là `null` ⇒ `isRescanReport(null)` trả `false` ⇒ nhánh "SAI HÌNH DẠNG" chạy trên một `null` HỢP LỆ. Ngược §I/O Matrix và ngược chính doc-comment của `ChooseRootResult` cách đó 60 dòng. Tách `callChooseRoot`, xử lý `null` TRƯỚC guard; `isRescanReport` giữ nguyên độ chặt cho `library_rescan` nơi `null` thật sự là câu trả lời hỏng. Đối chứng GỠ đã chạy: đúng 2 ca huỷ đỏ, 11 ca kia xanh. ⚠️ **Bài học nằm ở chỗ khác:** vòng MỘT tôi tách `apply_chosen_root` *để* hàng ma trận này có phép đo, và ghi vào Change Log rằng nó đã đóng. Phép đo đó đặt ở tầng Rust — nơi mã vốn đúng — trong khi hành vi người dùng thấy hỏng ở tầng trên. Một phép đo đặt sai TẦNG cho đúng cảm giác an toàn mà nó không có quyền cho.
  - `[high]` `[patch]` **`resolve_library_root` không có một phép kiểm HÀNH VI nào**, dù nó là hàm quyết định *ứng dụng đọc/ghi thư mục nào*. Cổng P5 của vòng một chỉ so **thứ tự chuỗi trong mã nguồn**, không bao giờ gọi hàm; bộ e2e luôn chạy với biến môi trường đặt sẵn nên nó thoát ở nhánh đầu và ba nhánh còn lại chưa từng chạy. Tách hàm thuần `resolve_library_root_from(override, configured, default)` + `resolve_configured_library_root(store)`, thêm 8 ca cho cả ba nhánh. Đối chứng GỠ đã chạy (đảo nhánh `store = None` ⇒ ca mới đỏ). Đây cũng là chỗ duy nhất trong kho có thể bắt được lượt e2e đỏ chưa giải thích được, nếu nó tái diễn.
  - `[medium]` `[patch]` `rebuild_lock` chỉ bọc `rebuild` — ba phát hiện một gốc: `rescan` đọc `list_orphans()` NGOÀI khoá nên báo cáo trộn được hai thế hệ; `forget_orphan` không lấy khoá nên đua được với một lượt quét và sinh `library.not_orphaned` sai nguyên nhân; `list_orphans()` trượt SAU khi mutation đã commit làm `?` báo lỗi cho một thao tác ĐÃ xảy ra. Vá bằng một nước: `rebuild`/`forget_orphan` tự trả ảnh chụp mồ côi lấy TRONG phạm vi đã khoá, và `forget_orphan` lấy khoá.
  - `[medium]` `[patch]` §Verification của spec vẫn mang hai con số (*"ba tệp"*, *"831 dòng"*) mà §Spec Change Log của **cùng tệp** đã đo là sai (5 tệp, 860 dòng). Tôi ghi lại bản sửa thay vì thực hiện nó. Sửa hai dòng tại chỗ, giữ mục Change Log làm lịch sử.
  - `[medium]` `[patch]` doc-comment `LIBRARY_WORK_DDL` còn khai *"mỗi lượt `rebuild` xoá sạch bảng rồi chèn lại"* và mô tả `work_id PRIMARY KEY` như lưới chắn trùng cuối. Từ story này `rebuild` là UPSERT, nên một hàng trùng lọt tới SQL **ghi đè im lặng** — lưới chắn đó đã mất. Viết lại, nói thẳng vế vừa mất.
  - `[medium]` `[patch]` Hai mệnh đề tôi tự viết về độ phủ đều sai: *"bộ e2e của story vẫn 0 spec"* (viết trước khi dựng bàn đo, không quay lại sửa) và *"bộ e2e nằm ngoài cả `pre-push` LẪN `ci.yml`"* (nửa sau sai — `ci.yml:712` có job `e2e` chạy `schedule` cron `0 18 * * *` + `workflow_dispatch`; tôi chép lại một mệnh đề của Story 5.2 vốn hết đúng từ 2026-08-20). Sửa cả ba chỗ mang chúng, nối tiếp chứ không viết đè.
  - `[low]` `[patch]` spec có **hai** tiêu đề `## Review Triage Log`, một rỗng sót lại từ khuôn — gỡ.
  - `[low]` `[patch]` `.root-value` thiếu `role="status"` trong khi ba node hàng xóm đều có: đường dẫn gốc đổi mà trình đọc màn hình không được báo (NFR17).
  - `[low]` `[patch]` `err.library.not_orphaned` chỉ nội suy `work_id` (UUID trần) trong khi `name` có sẵn ở chỗ gọi — thêm `name` vào bảng tham số bắt buộc và vào chuỗi.
  - `[low]` `[patch]` `libraryRescan.ts` có hai cửa đồng thời (chống tái nhập `rescanBusy`, bỏ kết quả cũ `mySequence !== sequence`) mà tệp test tự khai là vai của nó — không ca nào chạm. Thêm hai ca.
  - `[low]` `[patch]` `apply_chosen_root` với `store: None` không ca nào chạm dù doc-comment khai nhánh đó. Thêm một ca.

## Design Notes

**Vì sao `forget_orphan` không phải "đường ghi thứ hai".** Story 5.2 cấm một hàm `index_one(...)` chạy song song với `rebuild` vì đó là **hai chỗ cùng dựng cùng một hàng** và chúng sẽ trôi khỏi nhau ở logic phát hiện trùng `work_id`. `forget_orphan` không dựng hàng nào: nó **xoá đúng một hàng đã tồn tại**, với tiền điều kiện `orphaned = 1`, và nó không đọc đĩa. Không có logic nào để trôi. Ranh giới đúng là *"chỉ `rebuild` được quyết một hàng trông như thế nào"*, không phải *"chỉ tồn tại đúng một câu SQL"*.

🔴 **Hệ quả phải nói thẳng: chỉ mục KHÔNG còn dẫn xuất trọn vẹn từ đĩa.** Cờ mồ côi là mẩu trạng thái **duy nhất** trong `library-index.db` không suy ra được từ các `.atproj` — xoá tệp chỉ mục rồi dựng lại thì mọi hàng mồ côi biến mất. Hai phương án đã cân, và cả hai đều thoả AC:

- **(a) cờ sống trong `library-index.db`** *(đã chọn)* — không kho mới, không phụ thuộc mới, `Indexer` vẫn là module duy nhất chạm kho này. Mất mát khi xoá chỉ mục là **mất một lời nhắc**, không mất dữ liệu người dùng; FR98 (*"dựng lại hoàn toàn từ các `.atproj`"*) vẫn đúng cho mọi thứ **suy ra được**, và một chỉ mục vừa dựng lại từ số không mà không có mồ côi nào là một trạng thái **trung thực**, không phải một trạng thái sai.
- **(b) cờ sống trong `global.db`** — sống sót qua một lượt xoá chỉ mục, nhưng đưa từ vựng Library vào một kho khác, dựng một nguồn sự thật thứ hai về *"Tác phẩm nào từng tồn tại"*, và mở đúng lớp lỗi *"hai dữ kiện nói cùng một chuyện thì chúng lệch được"*.

⇒ Chọn (a) vì nó **hẹp hơn**, và ghi nguyên vế yếu ra đây thay vì để người sau tự phát hiện. Đây là chỗ đáng để Ice xác nhận lúc rà: nếu Ice muốn (b), đó là một quyết định kiến trúc, không phải một lượt sửa mã.

**Cột `orphaned` là boolean, không phải mốc thời gian.** Một cột `orphaned_at TEXT` nghe đầy đủ hơn nhưng kéo theo một phụ thuộc đồng hồ vào đúng đường mà mọi phép kiểm phải chạy tất định — và AC3 chỉ đòi *"nêu rõ nó trỏ tới đâu"*, thứ mà `atproj_path` đã lưu sẵn trả lời trọn vẹn. Thêm một cột cho một câu hỏi chưa ai hỏi là đúng thứ §Never của Story 5.2 cấm.

**Vị từ mồ côi: BỐN cách viết, BA cách sai.** ① *"`work_id` không có mặt trong tập ĐỌC ĐƯỢC"* — sai: một `.atproj` còn nằm nguyên đó nhưng `meta.json` hỏng không đọc được, nên nó bị gọi là mồ côi; đó là loại câu sai người dùng sẽ tin rồi đi tìm tệp ở chỗ khác (hạng `skipped` của Story 5.2 mới là chỗ đúng của ca đó). ② *"`atproj_path` không còn tồn tại trên đĩa"* — cũng sai, và sai ở ca **đổi thư mục gốc**: các `.atproj` của gốc cũ vẫn nằm nguyên trên đĩa, nên chúng ở lại chỉ mục như những hàng **đang sống** trỏ ra ngoài thư viện — chỉ mục khi đó khẳng định một điều nó không kiểm được.

🔵 **SỬA (2026-08-27, vòng rà bốn lớp P3) — cách viết ③ dưới đây, bản trước tuyên bố là ĐÚNG, cũng SAI, và đoạn này đứng lại làm bằng chứng thay vì bị xoá.** ③ *"`work_id` không đọc được ở lượt này **VÀ** `atproj_path` không nằm trong tập `.atproj` vừa LIỆT KÊ được trong gốc"* — sai ở ca **đường dẫn bị Tác phẩm KHÁC chiếm**: A sống ở `/gốc/Foo.atproj`; người dùng xoá A rồi copy B (work_id khác hẳn) vào một thư mục CŨNG tên `Foo.atproj`. B đọc được ⇒ `Foo.atproj` vẫn nằm trong tập "vừa liệt kê được", nên vế hai của ③ coi hàng của A là "đường dẫn còn đó" và KHÔNG đánh dấu mồ côi — một hàng SỐNG nói dối, trỏ vào thư mục nay thuộc về B. Cách viết ③ hỏi *"đường dẫn này có được liệt kê không"*; câu đó không phân biệt được "còn là của tôi" với "nay là của người khác".

④ *"`work_id` không đọc được ở lượt này **VÀ** `atproj_path` không nằm trong tập `.atproj` vừa liệt kê được **MÀ meta.json KHÔNG đọc được**"* — đúng cả bốn ca, vì nó hỏi đúng câu mà chỉ mục có thẩm quyền trả lời: *đường dẫn này, nếu còn tồn tại, có xác nhận được nó vẫn là của TÔI không* — một đường dẫn ĐỌC ĐƯỢC luôn xác nhận nó thuộc về đúng `work_id` vừa khai trong đó (dù đó là work_id nào), không phải một tấm khiên chung cho MỌI `work_id` từng đứng ở vị trí đó.

**Vì sao một `Mutex` chứ không một `AtomicU64` thế hệ.** Khuôn thế hệ (`ImportScanGeneration`) đúng cho việc **huỷ** một lượt quét đã cũ khi một lượt mới bắt đầu — ở đó bỏ kết quả cũ là hành vi mong muốn. Ở đây hai lượt `rebuild` đều **đúng** và đều phải hoàn tất; thứ phải chặn là chúng **xen kẽ** giai đoạn quét với giai đoạn ghi. Nối tiếp là ngữ nghĩa đúng, và một `Mutex` nói đúng điều đó.

## Verification

**Commands:**
- `cd src-tauri && cargo test --test library_index_contract --test library_index_boundary --test config_invariants --test ipc_contract` -- expected: xanh; cổng ranh giới xanh **mà không** nới danh sách miễn trừ.
- `cd src-tauri && cargo test --locked` -- expected: toàn bộ xanh (baseline 5.2: **774 ca**); mọi ca đỏ phải là ca của 5.2 mà story này cố ý đổi nghĩa, không một ca nào khác.
- `npm run test` -- expected: xanh; baseline 5.2 là **43 tệp / 567 ca**, story này thêm đúng một tệp frontend ⇒ 44 tệp và số ca tăng, không giảm.
- `npm run check:commands && npm run check:i18n && npm run check:tokens && npm run check:gates && npm run check:debt-owner` -- expected: xanh. `check:gates` xanh **mà không** sửa ba danh sách (cổng mới là test Rust, không phải `check-*.mjs`).
- `npm run build` -- expected: xanh (`vue-tsc` thấy mọi kiểu mới, kể cả cây `tests/frontend/**`).
- 🔵 **SỬA (2026-08-27, vòng rà THỨ HAI P4) — con số dưới đây đã sai, §Spec Change Log mục ③ đã đo lại nhưng bản thân dòng này chưa từng được sửa.** `grep -rn "StoreSpec::library_index\|StoreKind::LibraryIndex" src-tauri/src` -- expected: **năm tệp** — `core/library/indexer.rs`, `core/store/mod.rs` (điểm khai), `lib.rs`, `core/i18n/mod.rs`, `commands/library.rs`, và **chỉ ba tệp ĐẦU** được phép nhắc ở vị trí MÃ (`lib.rs` chỉ ở dòng `///`); hai tệp SAU (`core/i18n/mod.rs`, `commands/library.rs`) chỉ được nhắc ở dòng `//`. Một dòng **mã** ở `commands/library.rs` nhắc `StoreKind::LibraryIndex`/`StoreSpec::library_index` là vi phạm thật; nếu cổng vẫn xanh thì **cổng hỏng**. *(Lệnh grep không phân biệt mã với chú thích — phán quyết thuộc về `cargo test --test library_index_boundary`, thứ CÓ lọc `//`.)*
- `grep -rn "rusqlite\|Connection::open" src-tauri/src/core/library/ src-tauri/src/commands/library.rs` -- expected: **0 dòng**.
- 🔵 **SỬA (2026-08-27, vòng rà THỨ HAI P4)** — `cd src-tauri && cargo tree --locked -e normal | wc -l` -- expected: **860** dòng, không đổi — con số 831 chép từ Story 5.2 đã lỗi thời (đo lại 2026-08-27, xem §Spec Change Log mục ③: `Cargo.toml`/`Cargo.lock` không đổi một byte ở story này). Mệnh đề đáng giữ vẫn đúng và đã đo: story này thêm **0** phụ thuộc nào (`tauri-plugin-dialog` đã có sẵn từ 3.10b).

**Đối chứng bắt buộc (một bộ test xanh KHÔNG chứng minh chỗ nối được canh):**
- Gỡ vế đánh dấu mồ côi ra khỏi `Indexer::rebuild` rồi chạy lại `cargo test` -- expected: **đỏ**, và đỏ ở ca nói về mồ côi. Xanh ⇒ ca test chưa chạm bề mặt thật, sửa ca test chứ không sửa mã.
- Gỡ `(async)` khỏi một vỏ mới rồi chạy `cargo test --test config_invariants` -- expected: **đỏ**. Xanh ⇒ cổng vẫn đang viết cứng `glossary.rs`, tức việc mở rộng cổng chưa thật.

**Manual checks:**
- Chạy ứng dụng, tạo một Tác phẩm, thoát; **đổi tên** thư mục `.atproj` đó (vẫn để trong thư mục gốc), mở lại, bấm quét lại: đúng **một** hàng, `atproj_path` là tên mới, **không** mục mồ côi nào — đây là ca *di chuyển*.
- Rồi **chuyển hẳn** thư mục đó ra ngoài thư mục gốc, quét lại: nó hiện là mục mồ côi kèm đường dẫn cũ, và vẫn còn đó sau khi khởi động lại ứng dụng. Chuyển trở vào, quét lại: nó về bình thường, **không** có hàng thứ hai.
- Đổi thư mục gốc sang một thư mục trống, quét lại, tạo một Tác phẩm mới: `.atproj` nằm trong thư mục vừa chọn.
- Trong lúc quét một thư mục có nhiều `.atproj`, gõ vào một ô nhập của Library: chữ vẫn hiện ra ngay, cửa sổ không xoay bánh xe.

## Auto Run Result

Status: done
Blocking condition: (không có)

### Điều đã dựng

FR99 — *"copy một thư mục `.atproj` vào là nó xuất hiện trong Library"* — nay có một bề mặt thật, và `Indexer::rebuild` đổi ngữ nghĩa từ **xoá-sạch-ghi-lại** sang **đối chiếu**: mọi `.atproj` đọc được đi qua một UPSERT (đường dẫn cập nhật tại chỗ ⇒ *di chuyển* không sinh hàng thứ hai), còn hàng nào không đọc được ở lượt này **và** đường dẫn của nó không nằm trong tập `.atproj` liệt-kê-được-mà-không-đọc-được thì mang cờ `orphaned = 1` và **ở lại** kèm đường dẫn cũ. Thư mục gốc Library thành một khoá `AppConfig` (`library_root`), đổi được qua hộp thoại chọn thư mục gọi từ Rust (AD-48, **0 phụ thuộc mới** — `tauri-plugin-dialog` đã có từ Story 3.10b). Ba lệnh IPC, năm command đăng ký, và một khối màn hình tối thiểu trong `LibraryMode.vue`.

Cùng lượt, đóng món nợ `deferred-work.md` mà Story 5.2 giao đích danh cho story này: `Indexer::rebuild_lock` nay khoá **cả** giai đoạn quét đĩa lẫn giai đoạn ghi, nên hai lượt quét gọi gần nhau không còn xen kẽ được.

### Tệp đã đổi

**Rust — lõi**
- `src-tauri/src/core/store/schema.rs` — thêm cột `orphaned` vào `LIBRARY_WORK_DDL`, bump `LIBRARY_INDEX_MIGRATIONS` lên `to_version: 2` (vẫn **một** bước: kho dẫn xuất không di trú, nó tự xoá-và-dựng-lại).
- `src-tauri/src/core/library/indexer.rs` — `rebuild` đối chiếu thay vì xoá-sạch; `rebuild_lock: Mutex<()>` bọc trọn scan+ghi; thêm `forget_orphan`, `list_orphans`, `RebuildOutcome::orphans`, `IndexError::NotOrphaned`, `From<IndexError> for IpcError`.
- `src-tauri/src/core/scope/store.rs` — khoá `AppConfig` thứ sáu `library_root` + hàm thuần `resolve_library_root_value`.
- `src-tauri/src/core/i18n/mod.rs` — đúng **hai** `MessageKey` mới: `LibraryNotOrphaned`, `LibraryRootInvalid`.

**Rust — bề mặt**
- `src-tauri/src/commands/library.rs` (**mới**) — hàm thuần `rescan`/`forget_orphan`/`apply_chosen_root`, cộng ba vỏ `#[tauri::command(async)]`.
- `src-tauri/src/commands/project.rs` — `resolve_library_root(app, store)`: móc e2e ⇒ giá trị đã cấu hình ⇒ mặc định.
- `src-tauri/src/commands/mod.rs` · `src-tauri/src/lib.rs` — khai module, ba vỏ vào `generate_handler!`, lượt quét lúc khởi động dùng bộ phân giải mới.

**Rust — test**
- `src-tauri/tests/library_index_contract.rs` — +11 ca (di chuyển · xoá · quay lại · gỡ · gỡ nhầm · hỏng-mà-còn-đó · đổi gốc · sống sót khởi động lại · hai luồng đồng thời · đường dẫn bị chiếm).
- `src-tauri/tests/library_commands_contract.rs` (**mới**) — 11 ca ở tầng lệnh: `.atproj` mới copy vào · huỷ hộp thoại · chọn gốc · đường dẫn không phải thư mục · `root_missing` hai chiều · `Indexer` vắng mặt · hình dạng `IpcError` của `From<IndexError>`.
- `src-tauri/tests/config_invariants.rs` — cổng vỏ-chặn đổi thành danh sách `(tệp, chữ ký, vì sao)` **đọc nhiều tệp**; thêm cổng quét nguồn cho thứ tự ưu tiên của `resolve_library_root`.
- `src-tauri/tests/ipc_contract.rs` — đóng băng tên trường `RescanReport`/`OrphanEntry`.
- `src-tauri/tests/project_contract.rs` · `glossary_scan_contract.rs` — miễn trừ có tên cho `library_work` kèm đối chứng dương/âm; 5 ca cho `resolve_library_root_value`.

**Frontend**
- `src/config/library.ts` (**mới**) — adapter IPC thứ tám, hình dạng ba trạng thái, type guard lúc chạy.
- `src/modes/libraryRescan.ts` (**mới**) — state thuần + vị từ `…HasLoaded`.
- `src/modes/LibraryMode.vue` · `src/commands/index.ts` · `src/main.ts` · `src/i18n/vi.json` — màn hình tối thiểu, năm command (`library.rescan` gán `Mod+Alt+K`; `Mod+Alt+R` đã thuộc `editor.restore_segment`), chuỗi hiển thị.
- `tests/frontend/libraryRescan.test.ts` (**mới**) — 9 ca module thuần.

**Tài liệu**
- `src-tauri/AGENTS.md:29` — 🔵 sửa tại chỗ: `library-index.db` **không còn** dẫn xuất trọn vẹn từ đĩa.
- `_bmad-output/implementation-artifacts/deferred-work.md` — đóng món nợ "hai lượt `rebuild` chồng nhau"; mở rộng món nợ "đoạn nối không ca nào chạm" sang ba vỏ mới; ghi một nợ mới có chủ (đường ĐỌC THUẦN cho lúc mở Library, chủ 5.6); vá một mục mồ côi có sẵn từ Story 5.1.

### Kết quả vòng rà

Bốn lớp rà độc lập (blind · edge-case · verification-gap · intent-alignment). Phân loại: **intent_gap 0 · bad_spec 0 · patch 9 · defer 3 · reject 4** — mã không bị dựng lại, `review_loop_iteration` giữ 0. Chi tiết từng mục ở §Review Triage Log. Hai bản vá nặng nhất đều là **màn hình khẳng định một điều nó chưa biết**: một cái đánh rơi `root_missing` nên không phân biệt được "gốc mất" với "gốc rỗng"; một cái nói "Mặc định — ~/Documents/AuraTranslate/" trong khi nó chỉ biết "chưa quét".

Đề nghị rà tiếp: **có** (hai bản vá mức nặng ⇒ `true` theo đúng luật tính; patched theo mức: high 2 · medium 4 · low 3).

### Nghiệm thu đã chạy

- `cargo test --locked` — **805 xanh / 0 đỏ** (baseline Story 5.2: 774).
- `npm run test` — **44 tệp / 576 ca**, xanh. `npm run build` — xanh.
- Chín cổng tĩnh xanh: `commands` · `i18n` · `tokens` · `gates` · `debt-owner` · `deps` · `layout` · `panel-refs` · `dict-manifest`.
- `grep` ranh giới: `StoreSpec::library_index`/`StoreKind::LibraryIndex` chỉ ở **vị trí mã** trong `core/library/indexer.rs` và `core/store/mod.rs`; `rusqlite`/`Connection::open` dưới `core/library/` + `commands/library.rs` = **0 dòng**.
- `cargo tree --locked -e normal` = **860** dòng, **không đổi** so với baseline ⇒ 0 phụ thuộc mới. *(Con số 831 mà §Verification chép từ tài liệu Story 5.2 đã lỗi thời — xem §Spec Change Log.)*
- **Đối chứng GỠ đã chạy thật, không suy luận:** gỡ vế đánh dấu mồ côi ⇒ 3 ca đỏ · gỡ `(async)` ⇒ cổng vỏ-chặn đỏ · nhét `put_config` vào nhánh huỷ hộp thoại ⇒ đúng 1 ca đỏ · ghim cứng `root_missing: false` ở tầng Rust ⇒ đúng 1 ca đỏ (`tests/library_commands_contract.rs:278`) · dựng lại tập vế-hai từ `scan.dirs` ⇒ ca "đường dẫn bị chiếm" đỏ. Mọi lượt đều khôi phục nguyên trạng sau khi đo.
- Toàn bộ **15 hàng §I/O Matrix** đều có ít nhất một ca ĐÃ CHẠY và xanh.

### Chạy trên ỨNG DỤNG THẬT — bổ sung 2026-08-27

§Manual checks không còn là một mục treo: dựng `e2e/specs/story-5-3-rescan.e2e.mjs` (6 ca) chạy trong **WKWebView thật**, đi trọn đường nút thật → `dispatch` → registry → `invoke` → `Indexer` → DOM. Đây cũng là lần đầu ba vỏ `#[tauri::command(async)]` của story được một phép kiểm chạm tới thay vì chỉ bị so chuỗi chữ ký.

Phủ được: trạng thái CHƯA BIẾT trước lượt quét đầu · `.atproj` mới copy vào xuất hiện sau đúng một lượt quét · chuyển ra ngoài gốc ⇒ mục mồ côi kèm đường dẫn cũ · quay lại ⇒ hết mồ côi, không hàng thứ hai · nút "Gỡ khỏi chỉ mục" · thư mục gốc biến mất nói ra lý do.

**Vẫn KHÔNG phủ, ghi thẳng:** `library.choose_root` mở hộp thoại **native** ngoài webview — WebDriver không chạm tới được, và bấm nó trong một lượt tự động sẽ TREO cửa sổ chờ người thật. Vế "đổi thư mục gốc" ở lại chạy tay. AC6 (*gõ được trong lúc quét*) cũng ở lại chạy tay: dựng một thư viện đủ lớn để đo là dựng một phép đo chập chờn theo tốc độ đĩa người chạy.

Mẫu ba lượt: **2 xanh · 1 đỏ**. Lượt đỏ chưa chẩn đoán được và đã ghi nợ có chủ (xem §Rủi ro #6).

### Rủi ro còn lại — ghi ra thay vì để người sau tự phát hiện

1. 🔴 **Chỉ mục không còn dẫn xuất trọn vẹn từ đĩa.** Cờ `orphaned` là mẩu trạng thái duy nhất trong `library_work` không suy ra được từ các `.atproj`; xoá `library-index.db` rồi dựng lại làm mọi hàng mồ côi biến mất. §Design Notes cân hai phương án và chọn phương án hẹp hơn (cờ ở chính kho dẫn xuất). **Đây là chỗ đợi Ice chốt** — nếu Ice muốn cờ sống ở `global.db`, đó là một quyết định kiến trúc, không phải một lượt sửa mã.
2. 🔵 **SỬA (2026-08-27, vòng rà THỨ HAI P6) — mục này VIẾT TRƯỚC khi dựng bộ e2e của story, không quay lại sửa.** Nay **hai trong ba** vỏ (`library_rescan`, `library_forget_orphan`) có 6 ca e2e chạm THẬT qua WKWebView (xem §Chạy trên ỨNG DỤNG THẬT); chỉ `library_choose_root` (hộp thoại native, ngoài tầm WebDriver) còn hở. Vế THẬT của AC6 (*"gõ được trong lúc quét"*) vẫn chỉ nghiệm thu bằng tay. Và mệnh đề "bộ e2e nằm ngoài cả `pre-push` lẫn `ci.yml`" sai nửa sau: `ci.yml:712` có job `e2e` chạy ở nhịp `schedule` (cron `0 18 * * *`) + `workflow_dispatch` — đúng là *ngoài `pre-push` và ngoài `push`, nhưng CÓ trong `ci.yml` ở nhịp đêm*. Đã ghi nợ có chủ (Story 5.6, phần còn hở) và sửa lại frontmatter `deferred`.
3. 🟡 **§Manual checks đóng MỘT NỬA.** Bốn trong sáu mục nay chạy tự động trên cửa sổ thật (xem §Chạy trên ỨNG DỤNG THẬT). Hai mục còn hở, và chúng hở vì lý do kỹ thuật chứ không vì thiếu thời gian: hộp thoại native không lái được bằng WebDriver, và AC6 cần một thư viện lớn thật.
4. ⚠️ **`check:debt-owner` đã ĐỎ từ TRƯỚC baseline** — một mục nợ mồ côi của Story 5.1 ở `deferred-work.md`. Đối chứng đã chạy: khôi phục tệp về `c533c62` rồi chạy cổng ⇒ exit 1. Đã vá bằng một dòng nêu Ice là người quyết định (*"có đưa `cargo clippy --all-targets` vào một cổng hay không"*). Việc này **nằm ngoài phạm vi story** và được báo riêng.
5. ⚠️ **`pre-push` chưa chạy trọn một lượt, CI chưa đọc.** Chưa push, nên mọi số đo trên đây đến từ macOS của Ice; nửa Windows chưa nói gì về story này.

6. 🔴 **Một lượt e2e ĐỎ chưa chẩn đoán được, và nó chạm đúng vế dữ liệu thật.** Lượt chạy đầu của spec mới (5/6 đỏ) cho thấy ứng dụng **ĐỌC** `~/Documents/AuraTranslate` — thư viện thật của Ice — thay vì thư mục tạm: `.root-value` ra đường dẫn thật, `.orphan-name` ra `Epochtime`, một Tác phẩm có thật. **Không byte nào bị GHI vào đó** (đã kiểm sau cả ba lượt: 0 mục mang dấu e2e, mtime thư mục cha không đổi). Hai lượt sau xanh sạch. Tôi **không** giải thích được nó từ mã, và giả thuyết đầu tiên tôi nêu (*cầu IPC hỏng*) đã bị chính số đo bác — cảnh báo đó có mặt ở cả ba lượt. Nguyên văn log đã cất, nợ ghi có chủ (Ice). ⚠️ Đáng chú ý: hàng rào âm của `wdio.conf.mjs` so **chữ ký thư mục**, nên nó canh chiều GHI và **không** canh chiều ĐỌC — đúng chỗ lượt này lọt qua.

### Vòng rà THỨ HAI — 2026-08-27, bổ sung

Bốn lớp rà chạy lại trên diff đã commit (`c533c62..HEAD`, 3.967 dòng, gồm cả bàn đo e2e). **11 patch · 4 defer · 5 reject · 0 intent_gap · 0 bad_spec** — chi tiết ở §Review Triage Log.

Điều đáng ghi hơn từng mục: **bốn trong mười một bản vá là tài liệu hoặc phép đo nói sai về chính mã**, và ba trong số đó là ca *"ghi chú VỀ chỗ sai thay vì SỬA chỗ sai"*. Cộng bản vá nặng nhất — một phép đo đặt **sai tầng** (đóng hàng "huỷ hộp thoại" ở Rust trong khi hành vi người dùng hỏng ở TypeScript) — vòng này bắt được nhiều lỗi trong cách tôi **kiểm** hơn trong cách sản phẩm **chạy**.

Đo lại sau vá: `cargo test --locked` **815 xanh / 0 đỏ** · `npm run test` **44 tệp / 580 ca** · `build`, `check:lint` và chín cổng tĩnh xanh · bộ e2e chạy lại sau khi P9 đổi hợp đồng trên dây của `library_forget_orphan`: **6/6 xanh** trên cửa sổ thật. Hai đối chứng GỠ tự chạy: bỏ nhánh huỷ ⇒ đúng 2 ca đỏ; đảo nhánh `store = None` ⇒ ca mới đỏ. Mẫu e2e tới nay: **3 xanh · 1 đỏ** (lượt đỏ vẫn chưa chẩn đoán được, xem mục 6).

Phạm vi bỏ qua ĐÃ nói ra, không làm tròn: `library.choose_root` vẫn không có ca tự động nào — hộp thoại native nằm ngoài webview, và bấm nó trong một lượt tự động sẽ treo cửa sổ chờ người thật. Đó là 1 trong 3 vỏ IPC.
