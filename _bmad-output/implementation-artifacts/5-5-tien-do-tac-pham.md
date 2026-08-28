---
title: 'Story 5.5: Tiến độ Tác phẩm'
type: 'feature'
created: '2026-08-28'
status: 'done'
baseline_commit: 'b4baa1f292d9f46a0363ca5f349db6d06611eac2'
baseline_revision: 'b4baa1f292d9f46a0363ca5f349db6d06611eac2'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-5-context.md'
warnings: ['oversized']
deferred:
  - summary: >-
      Mệnh đề "`chapter_count` không còn đóng băng" đứng trên một lượt `grep` tại một thời
      điểm, KHÔNG có cổng nào canh — Epic 6 (FR14, nhập hàng loạt) sẽ dựng đường tạo Chương
      thứ hai và làm nó sai mà không gì đỏ.
    evidence: |-
      Đo 2026-08-28 trên `b4baa1f`: `INSERT INTO chapter` đúng MỘT chỗ
      (`commands/project.rs:271`), `DELETE FROM chapter` KHÔNG chỗ nào, `write_atomic` hai chỗ
      gọi sản phẩm. Bốn con số này là thứ làm mệnh đề đúng HÔM NAY, và cả bốn đều là ảnh chụp
      một lượt quét — không ca test nào, không cổng nào cưỡng chế chúng. Khác hẳn AC4, vế được
      đóng bằng một cổng THẬT (`meta_write_boundary.rs`). Ai dựng FR14 phải đo lại bốn số này
      TRƯỚC khi thêm đường tạo Chương thứ hai, hoặc dựng cổng cho chúng.
    location: >-
      src-tauri/src/core/library/meta.rs::rebuild_from_store — chủ: Story 6.2 (pipeline nhập)
    severity: medium
  - summary: >-
      Nửa "nút đổi trạng thái Chương NẰM Ở Workspace" của AC5 chưa dựng — bàn đo e2e chuyển
      sang Workspace rồi gọi thẳng `set_chapter_status` qua cầu IPC làm vật thế chỗ.
    evidence: |-
      AC5 nguyên văn: "Chương đổi trạng thái TRONG Workspace, quay về Library". Vế "quay về
      Library" ĐÃ đóng thật (`onActivated` → `loadWorks()`, đối chứng đã chạy: gỡ lời gọi đó
      thì `story-5-5-progress.e2e.mjs` ĐỎ đúng câu "tiến độ không tăng lên 1/1"). Vế "trong
      Workspace" thì chưa: Workspace không có bề mặt nào đổi trạng thái Chương. Story 5.4 đã
      ghi đúng chỗ hở này (`libraryWorks.ts::setOpenChapterStatus` viết cứng `'done'`, ba
      chuyển đổi còn lại không lối vào) và giao chủ Story 5.7.
    location: >-
      e2e/specs/story-5-5-progress.e2e.mjs — chủ: Story 5.7
    severity: medium
  - summary: >-
      KHUYẾT TẬT BÀN ĐO, không thuộc story này — `story-5-3-rescan.e2e.mjs` đặt bãi đỗ tạm ở
      thư mục tạm HỆ THỐNG dùng chung cho MỌI lượt chạy, nên rác của một lượt gãy làm lượt sau
      đỏ và kéo theo hai spec chạy sau nó.
    evidence: |-
      `parkingLot = join(dirname(libraryRoot), 'e2e-parking-lot')`
      (`story-5-3-rescan.e2e.mjs:113`). `libraryRoot` là thư mục tạm THEO LƯỢT
      (`/T/auratranslate-e2e-library-XXXX`), nên `dirname` của nó là `/T` — dùng chung mọi
      lượt. Đo được trong phiên 2026-08-28 khi chạy bộ e2e nhiều lượt liên tiếp (điều kho chưa
      từng làm: bộ này vốn chạy nhịp đêm): hai lượt liên tiếp đỏ TRÙNG KHÍT ở
      `shortcuts-focus` · `story-5-3` · `story-5-4` · `story-5-5`, với dấu vết
      `ENOENT ... rename '/T/e2e-parking-lot/e2e-rescan-probe-*.atproj'`. Đã loại trừ story
      5.5 làm nguyên nhân bằng bốn phép đo: mã GỐC `b4baa1f` 15/15 xanh; mã CÓ story nhưng cất
      spec thứ 16 ra 15/15 xanh; `story-5-3` chạy riêng 7/7 xanh; và lượt trọn bộ 16 spec SAU
      khi rác được dọn 16/16 xanh. Spec mới của story chạy CUỐI trong một bộ tuần tự
      (`maxInstances: 1`) nên nó không thể là nguyên nhân của một spec chạy trước nó.
    location: >-
      e2e/specs/story-5-3-rescan.e2e.mjs:113 — chủ: Ice (quyết định cô lập bãi đỗ theo lượt)
    severity: medium
---

<intent-contract>

## Intent

**Problem:** Library hiện được TÊN và TRẠNG THÁI vòng đời của một Tác phẩm nhưng không hiện
được **còn bao nhiêu** — số Chương đã xong không tồn tại ở một tầng nào: `WorkMeta` chỉ mang
`chapter_count` (tổng), `library_work` không có cột nào cho nó, `WorkRow` không chở nó qua
dây, và không câu SQL nào trong kho đếm `chapter.status = 'done'`. Người dịch phải mở từng
Tác phẩm ra đếm — đúng điều FR7 tồn tại để bỏ.

**Approach:** Kéo **một** đại lượng duy nhất — số Chương đã xong — đi trọn bốn tầng đã dựng
sẵn của Story 5.2/5.4 (`project.db` → `meta.json` → `library_work` → `WorkRow` → DOM), tính
tại ĐÚNG chỗ `WorkMeta::rebuild_from_store` đang đọc `chapter.status` (không câu SQL thứ hai),
rồi hiện "đã xong / tổng" kèm thanh tiến độ ở danh sách Tác phẩm của `LibraryMode.vue`. AC4
("không thành phần nào khác ghi `meta.json`") được đóng bằng một **cổng ranh giới** chạy trên
cây nguồn, không bằng một lượt rà tay.

## Boundaries & Constraints

**Always:**

- `chapter_done_count` là `Option<u32>` ở `WorkMeta` và `NULL`-được ở `library_work` —
  `None`/`NULL` nghĩa **CHƯA BIẾT** (một `meta.json` v1/v2 chưa từng qua
  `WorkMeta::rebuild_from_store`), **KHÔNG** phải `0`. Đúng lý lẽ `status: Option<String>` của
  Story 5.4: một Tác phẩm đã dịch xong mà hiện "0 Chương đã xong" là rỗng im lặng.
- Đếm chạy **NGOÀI** nhánh `match status_override` trong `rebuild_from_store`. Hôm nay nhánh
  `Some(raw)` **không phân giải** `chapter_status_rows` một lần nào — đặt phép đếm bên trong
  `match` làm mọi Tác phẩm có ghi đè thủ công mất tiến độ.
- Đếm từ tập **ĐÃ PHÂN GIẢI** (`Vec<LifecycleStatus>`), không từ chuỗi thô: một hàng
  `chapter.status` hỏng không được tính là đã xong, và nó đã có sẵn đường ghi chẩn đoán.
- Một lượt đọc `SELECT status FROM chapter` duy nhất — tái dùng câu đang có, không thêm
  `SELECT COUNT(*) … WHERE status = 'done'`.
- `META_SCHEMA_VERSION` 2 → 3. `LIBRARY_INDEX_MIGRATIONS.to_version` 4 → 5, bằng cách **viết
  lại `LIBRARY_WORK_DDL` TẠI CHỖ** — luật riêng của kho dẫn xuất, đã ghi trong doc-comment của
  chính hằng đó; không thêm một bước di trú thứ hai.
- Thanh tiến độ chỉ dùng token (`var(--color-…)`); không màu viết thẳng, không gradient, không
  bóng đổ. Nó khai giá trị cho trợ năng: `role="progressbar"` + `aria-valuemin/valuemax/
  valuenow` + `aria-valuetext` lấy qua `t()` (NFR17, WCAG AA cả hai theme).
- Mọi chuỗi ở vị trí mã trong `src-tauri/**` viết KHÔNG DẤU; mọi văn bản hiển thị đi qua
  `vi.json`.
- Không lệnh `CommandRegistry` mới: story này đọc qua `library.list_works` đã đăng ký.

**Block If:**

- Định nghĩa "đã xong" cần đổi khỏi `chapter.status = 'done'` (ví dụ: đếm theo câu đã xác
  nhận, hay một trạng thái Chương thứ năm) — đó là quyết định của Ice, không phải một dòng mã.
- Đóng nợ `work.updated_at`/`chapter.updated_at` đòi mở lại AC đã ký của Story 2.3 (cổng
  `segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
  đỏ) — nợ đó có chủ **Story 5.6**, không mở ở lượt này.

**Never:**

- Không suy tiến độ từ `segment.status` — §Never của Story 5.4 cấm suy trạng thái Chương từ
  segment, và tiến độ đứng trên trạng thái Chương.
- Ghi đè thủ công trạng thái Tác phẩm KHÔNG bao giờ đổi tiến độ: một Tác phẩm ghi đè `paused`
  vẫn hiện đúng số Chương đã xong thật.
- Không lưới Tác phẩm, không bìa, không lọc/sắp xếp theo tiến độ (chủ Story 5.6); không danh
  sách Chương (chủ Story 5.7); không bề mặt đổi trạng thái Chương MỚI ở Workspace (chủ Story
  5.7).
- Không cột tiến độ ở `global.db`, không kho thứ ba.
- Không thêm một chỗ gọi ghi `meta.json` thứ ba — hai chỗ đang có
  (`commands/project.rs`, `commands/lifecycle.rs`) là danh mục đóng của story này.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Tác phẩm một Chương chưa bắt đầu | 1 hàng `chapter`, `status = 'not_started'` | `chapter_done_count = Some(0)`, `chapter_count = 1`; Library hiện `0 / 1`, thanh ở 0 % | Không lỗi |
| Chương chuyển sang `done` | `set_chapter_status(id, "done")` | `meta.json` ghi lại `Some(1)`; `library_work` UPSERT `1`; danh sách sau lượt tải lại hiện `1 / 1`, thanh 100 % | Lỗi ghi `meta.json` NÓI RA (`work.create_failed`), không nuốt |
| Tác phẩm có ghi đè thủ công | `work.status_override = 'paused'`, 1/2 Chương `done` | `status = "paused"`, `status_is_override = true`, **và** `chapter_done_count = Some(1)`, `chapter_count = 2` | Không lỗi |
| `meta.json` v2 (trước story này) | Khoá `chapter_done_count` vắng mặt | Đọc ra `None`; `library_work.chapter_done_count IS NULL`; Library hiện câu "chưa biết", **không** hiện `0 /` và **không** vẽ thanh | Không lỗi — vắng khoá là hợp lệ |
| `meta.json` v3 gặp bản ứng dụng cũ | `meta_schema_version = 3 > supported` | `MetaError::SchemaTooNew`, **không ghi một byte nào** | Đường từ chối đã có, không đổi |
| Hàng `chapter.status` hỏng | 2 hàng, một mang giá trị ngoài bốn | Hàng hỏng KHÔNG được đếm là đã xong; `chapter_count` vẫn `2`; chẩn đoán `eprintln!` đã có vẫn in | Không lỗi ra người dùng |
| Tác phẩm 0 Chương | `chapter_count = 0` | `chapter_done_count = Some(0)`; **không chia cho 0** — thanh vẽ ở 0 %, không `NaN` | Không lỗi |
| `library-index.db` ở `to_version` 4 | Tệp cũ trên đĩa | `Indexer::open` xoá và dựng lại như mọi lượt lệch phiên bản | Không mất dữ liệu người dùng (kho dẫn xuất) |

</intent-contract>

## Code Map

**Tầng nguồn sự thật**

- `src-tauri/src/core/library/meta.rs` -- `WorkMeta` (§struct, khoảng dòng 76–120) và
  `rebuild_from_store` (khoảng dòng 225–305). Chỗ DUY NHẤT tính giá trị dẫn xuất. Câu
  `SELECT status FROM chapter` và vòng `from_wire` để tái dùng nằm trong nhánh `None` của
  `match status_override` — **phải nâng ra ngoài `match`**. `META_SCHEMA_VERSION` ở dòng ~42.
- `src-tauri/src/core/lifecycle/mod.rs` -- `LifecycleStatus::Done` là vị từ đếm. Không sửa tệp
  này; đọc để biết `derive_work_status` KHÔNG liên quan tới tiến độ.

**Tầng chỉ mục dẫn xuất**

- `src-tauri/src/core/store/schema.rs` -- `LIBRARY_WORK_DDL` (khoảng dòng 1614) và
  `LIBRARY_INDEX_MIGRATIONS` (`to_version: 4`, ngay dưới). Doc-comment của `LIBRARY_WORK_DDL`
  mang nguyên văn câu *"không cột tiến độ (chủ Story 5.5)"* — mệnh đề đó **hết đúng** ở lượt
  này, sửa TẠI CHỖ kèm 🔵 và ngày.
- `src-tauri/src/core/library/indexer.rs` -- câu `INSERT INTO library_work … ON CONFLICT DO
  UPDATE` (khoảng dòng 296–320), hằng `COLUMNS` của đường đọc (khoảng dòng 487–530), và
  `struct IndexedWork` (khoảng dòng 850–868). Ba chỗ phải cùng thêm cột. `WorkMeta::read` ở
  dòng ~246 là chỗ đọc `meta.json` sản phẩm DUY NHẤT.

**Tầng dây IPC**

- `src-tauri/src/commands/library.rs` -- `struct WorkRow` (~dòng 275) + `impl From<IndexedWork>`
  (~dòng 288). `list_works` không đổi logic.
- `src-tauri/src/commands/lifecycle.rs` -- `write_lifecycle_after_change` (~dòng 76–92): khuôn
  bốn bước, đã gọi `rebuild_from_store` + `write_atomic`. **Không cần sửa** — tiến độ đi ké
  đúng đường này. Đây là bằng chứng cho AC5.
- `src-tauri/src/commands/project.rs` -- `create_work` (~dòng 271 `INSERT INTO chapter`, ~305
  `rebuild_from_store`, ~330 `write_atomic`). Chỗ ghi `meta.json` sản phẩm THỨ HAI và cuối.

**Tầng frontend**

- `src/config/library.ts` -- `type WorkRow` (~dòng 135) và vị từ kiểm kiểu lúc chạy
  `isWorkRowArray` (~dòng 161–180). Vị từ phải kiểm trường mới (`number | null`).
- `src/modes/LibraryMode.vue` -- khối `<li class="works-row">` (~dòng 378–406) và `<style>`
  (~dòng 840–872). `onActivated` (~dòng 82) + `watch(createdWork, …)` (~dòng 103) đã tải lại
  danh sách mỗi lần quay về Library — **dây nối của AC5 đã có sẵn**, không dựng lại.
- `src/i18n/vi.json` -- cụm khoá `mode.library.*` (~dòng 175–190).

**Cổng và bàn đo**

- `src-tauri/tests/library_index_boundary.rs` -- KHUÔN để chép cho cổng mới: `EXEMPT_FILES` +
  `FORBIDDEN` + `RS_FLOOR` + hai ca tự kiểm vị từ (bắt được / không bắt oan).
- `src-tauri/tests/library_index_contract.rs` -- hàm dựng `.atproj` thật (~dòng 92–125) và hai
  chuỗi JSON `meta.json` viết tay (~dòng 438, ~1325) phải thêm khoá mới.
- `src-tauri/tests/{project_contract.rs:221, library_commands_contract.rs:88, ipc_contract.rs:472-530}`
  -- ba chỗ khẳng định hình dạng, phải cập nhật cùng lượt.
- `tests/frontend/libraryWorks.test.ts` -- `invoke` giả, khuôn cho ca frontend mới.
- `e2e/specs/story-5-4-lifecycle.e2e.mjs` -- khuôn spec e2e; móc định danh
  `[data-lifecycle-action="…"]`, `.works-block .works-list .works-row`, `realClick`.

## Tasks & Acceptance

**Execution:**

1. [x] `src-tauri/src/core/library/meta.rs` -- thêm trường `chapter_done_count: Option<u32>` với
   `#[serde(default)]` kèm doc-comment nêu vì sao `Option` (chưa biết ≠ 0); nâng
   `META_SCHEMA_VERSION` 2 → 3 kèm khối 🔵 và ngày. Trong `rebuild_from_store`: **nâng** vòng
   phân giải `chapter_status_rows` ra NGOÀI `match status_override` để cả hai nhánh dùng chung,
   rồi đếm `LifecycleStatus::Done`. -- Nguồn sự thật của FR7; nâng ra ngoài `match` là chỗ
   duy nhất một Tác phẩm ghi đè thủ công không bị mất tiến độ.
2. [x] `src-tauri/src/core/store/schema.rs` -- thêm cột `chapter_done_count INTEGER` (cho phép
   `NULL`) vào `LIBRARY_WORK_DDL`, `to_version` 4 → 5; sửa TẠI CHỖ mệnh đề *"không cột tiến độ
   (chủ Story 5.5)"* kèm 🔵 + ngày + lý do `NULL` nghĩa chưa biết. -- Kho dẫn xuất viết lại tại
   chỗ, không thêm bước di trú.
3. [x] `src-tauri/src/core/library/indexer.rs` -- thêm cột vào câu UPSERT (cả `INSERT` lẫn nhánh
   `DO UPDATE`), vào hằng `COLUMNS` của đường đọc, và trường vào `IndexedWork`. -- Ba chỗ này
   trôi khỏi nhau thì một cột ghi vào mà không đọc ra được.
4. [x] `src-tauri/src/commands/library.rs` -- thêm trường vào `WorkRow` + `impl From<IndexedWork>`.
   -- Đưa tiến độ qua dây, `snake_case`, không `rename_all`.
5. [x] `src-tauri/tests/meta_write_boundary.rs` (MỚI) -- cổng ranh giới cưỡng chế AC4, chép khuôn
   `library_index_boundary.rs`: (a) `.write_atomic(` trên `WorkMeta` chỉ xuất hiện ở
   `core/library/meta.rs` · `commands/project.rs` · `commands/lifecycle.rs`; (b) `META_FILE` và
   chuỗi `"meta.json"` ở vị trí mã chỉ ở `core/library/meta.rs`; (c) `WorkMeta::read` chỉ ở
   `core/library/meta.rs` + `core/library/indexer.rs`; (d) sàn quần thể `.rs`; (e) hai ca **tự
   kiểm** chứng minh vị từ bắt được một dòng dựng tay và không bắt oan một dòng của module
   khác. -- AC4 nói "rà toàn bộ mã nguồn"; một lượt rà tay không sống qua story kế tiếp.
6. [x] `src-tauri/tests/library_index_contract.rs` + `library_commands_contract.rs` +
   `project_contract.rs` + `ipc_contract.rs` -- cập nhật hình dạng đã đóng băng; thêm ca cho
   ba ngưỡng của I/O Matrix: ghi đè thủ công vẫn có tiến độ, `meta.json` thiếu khoá ⇒ `None`
   đi trọn xuống `WorkRow`, hàng `chapter.status` hỏng không được đếm là đã xong. -- Đây là
   nơi ma trận I/O được nghiệm thu.
7. [x] `src/config/library.ts` -- thêm `chapter_done_count: number | null` vào `type WorkRow` và
   vào vị từ `isWorkRowArray` (chấp nhận `number` HOẶC `null`, từ chối `undefined`). -- Luật
   kho: dữ liệu qua dây luôn kiểm kiểu lúc chạy.
8. [x] `src/i18n/vi.json` -- ba khoá: `mode.library.works_progress` (`{done}` / `{total}`),
   `mode.library.works_progress_unknown`, `mode.library.works_progress_aria`. -- Không văn bản
   hiển thị nào ngoài `vi.json`.
9. [x] `src/modes/LibraryMode.vue` -- trong `<li class="works-row">` thêm một `<span>` "đã xong /
   tổng" và một thanh tiến độ (`role="progressbar"`, `aria-valuemin=0`, `aria-valuemax`=tổng,
   `aria-valuenow`=đã xong, `aria-valuetext` qua `t()`); nhánh `chapter_done_count === null` ⇒
   hiện câu "chưa biết" và KHÔNG vẽ thanh; `chapter_count === 0` ⇒ 0 %, không chia cho 0. CSS
   chỉ dùng token. -- Bề mặt của AC1.
10. [x] `tests/frontend/libraryWorks.test.ts` -- ca mount `LibraryMode.vue` với `invoke` giả trả ba
    hàng (`Some(0)`, `Some(1)`, `null`) và khẳng định ba nhánh render, cộng ca `chapter_count = 0`
    không sinh `NaN`. -- `happy-dom` đủ cho mệnh đề văn bản/thuộc tính; hình học thì không.
11. [x] `e2e/specs/story-5-5-progress.e2e.mjs` (MỚI) -- một kịch bản đi trọn đường trong WKWebView
    thật: tạo Tác phẩm qua form → tải danh sách → khẳng định `0 / 1` → bấm "Đặt Chương này là
    Đã xong" → tải lại → khẳng định `1 / 1` và `aria-valuenow="1"`. Dùng `realClick`, móc
    `[data-lifecycle-action="…"]`, không `:nth-of-type`. -- AC5 là mệnh đề ĐẦU-TỚI-CUỐI; nó
    chỉ được nghiệm thu ở webview thật.
12. [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- đóng bằng chữ mục *"`meta.json`
    (và do đó mọi cột `library_work` chép từ nó) là ẢNH CHỤP lúc tạo"* cho **vế
    `chapter_count`** kèm số đo mới (xem §Design Notes), và ghi rõ vế `updated_at` VẪN MỞ, chủ
    Story 5.6. -- Luật đóng nợ bằng chữ; không xoá mục đã đóng.
13. [x] `src-tauri/AGENTS.md` -- sửa TẠI CHỖ kèm 🔵 + ngày mệnh đề *"chỉ mục là ẢNH CHỤP lúc tạo
    (`updated_at`/`chapter_count` đóng băng theo `meta.json` — nợ ở `deferred-work.md`, chủ
    Story 5.5)"*: vế `chapter_count` đã đóng, vế `updated_at` còn mở. -- Mệnh đề hết đúng thì
    sửa tại chỗ, đừng để nó lặng lẽ sai.

**Acceptance Criteria:**

- [x] Given một Tác phẩm có `n` Chương trong đó `k` Chương ở `done`, when Library tải danh sách,
  then hàng của nó hiện `k / n` **và** một thanh tiến độ mang `aria-valuenow="k"` /
  `aria-valuemax="n"`.
- [x] Given một Tác phẩm mà `meta.json` chưa từng qua `rebuild_from_store` của story này, when
  Library tải danh sách, then hàng đó nói **"chưa biết"** và không vẽ thanh — không hiện `0 /`.
- [x] Given một Chương của Tác phẩm đang mở đổi sang `done`, when người dùng quay về Library,
  then tiến độ đã tăng — không còn số của lượt hiện trước.
- [x] Given toàn bộ cây `src-tauri/src/**`, when `cargo test` chạy, then cổng
  `meta_write_boundary.rs` khẳng định đúng ba tệp được ghi `meta.json` và đúng hai tệp được đọc
  nó, **và** hai ca tự kiểm của chính cổng đó chứng minh nó đỏ được.
- [x] Given `library-index.db` ở `to_version` 4 trên đĩa, when ứng dụng mở chỉ mục, then tệp bị xoá
  và dựng lại ở phiên bản 5, không mất một `.atproj` nào.
- [x] Given cả hai theme, when thanh tiến độ hiển thị, then mọi màu đến từ token và `check:tokens`
  xanh.

## Spec Change Log

## Review Triage Log

### 2026-08-28 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 7: (high 0, medium 3, low 4)
- defer: 3: (high 0, medium 3, low 0)
- reject: 6: (high 0, medium 0, low 6)
- addressed_findings:
  - `[medium]` `[patch]` Cổng `meta_write_boundary.rs` cắt dòng ở `//` ĐẦU TIÊN kể cả khi nó nằm trong một chuỗi ⇒ ÂM TÍNH GIẢ. Doc-comment khai "không tệp nào có `//` trong chuỗi" — đo được **5** dòng (`commands/project.rs:388`, `lib.rs:160,164,247,250`, các hằng `aura://…`). Sửa thành máy trạng thái nhận biết chuỗi, sửa mệnh đề sai thành con số đã đếm, thêm ca tự kiểm neo vào hình dạng `aura://`. Đối chứng ĐÃ CHẠY: giả lại hàm cắt thô ⇒ ca mới đỏ (`left: None, right: Some("write_atomic(")`).
  - `[medium]` `[patch]` AC "`library-index.db` ở `to_version` 4 bị xoá và dựng lại ở 5" chỉ phủ bằng hai ca CƠ CHẾ (99 và 0), không ca nào nêu đích danh số 4. Thêm ca dựng tệp ở `user_version = 4` với DDL CŨ rồi khẳng định dựng lại ở 5 kèm cột mới.
  - `[medium]` `[patch]` Bàn đo e2e của AC5 bấm nút nằm trong chính khối Library, không qua một lượt chuyển chế độ ⇒ chứng minh "ghi xong thì tính lại", không phải "quay về thì tải lại". Mở rộng: chuyển sang Workspace, đổi trạng thái ở đó, quay về bằng ⌘1 rồi mới khẳng định. Đối chứng ĐÃ CHẠY: gỡ `loadWorks()` khỏi `onActivated` ⇒ đỏ.
  - `[low]` `[patch]` Doc-comment của `path_in` khai lượt nâng `pub(crate)` phục vụ `tests/**` — sai, cây đó biên dịch thành crate riêng. Sửa cho đúng chỗ gọi thật.
  - `[low]` `[patch]` Hai `eprintln!` đổi chữ để tránh cổng, chưa ghi phép đo. Thêm vế: tên tệp VẪN tới log qua `{err}` vì `MetaError::Io` in đường dẫn đầy đủ.
  - `[low]` `[patch]` Nhánh chặn tràn `Math.min(100, …)` chưa ca nào chạm. Thêm ca `chapter_done_count > chapter_count`.
  - `[low]` `[patch]` Lập luận đo lại nợ chép vào hai tài liệu không trỏ nhau. Thêm tham chiếu chéo hai chiều.


## Design Notes

### Đo lại nợ trước khi thi hành nó — tiền đề của sổ nợ đã hết đúng

Sổ nợ giao Story 5.5 *"sở hữu cơ chế làm `chapter_count`/ngày sửa của `meta.json` sống thật"*,
với bằng chứng đo **2026-08-27**: *"`meta.json` có đúng MỘT chỗ gọi `write_atomic` sản phẩm
(`commands/project.rs:242`)"*. Đo lại hôm nay (2026-08-28, trên `b4baa1f`):

- `grep -rn "write_atomic" src-tauri/src` ⇒ **hai** chỗ gọi sản phẩm: `commands/project.rs:330`
  (sau `create_work`) và `commands/lifecycle.rs:80` (sau mỗi lượt đổi trạng thái). Chỗ thứ hai
  do **Story 5.4** dựng, tức SAU lượt đo của sổ nợ.
- `grep -rn "INSERT INTO chapter" src-tauri/src` ⇒ **một** chỗ (`commands/project.rs:271`,
  trong `create_work`). `grep -rn "DELETE FROM chapter" src-tauri/src` ⇒ **không** chỗ nào.

⇒ `chapter_count` **không** đóng băng nữa, và không phải vì story này sửa gì: số Chương chỉ đổi
được lúc tạo Tác phẩm, và `meta.json` được ghi ngay tại đó. Việc còn lại của story này là
**thêm một đại lượng** vào cùng đường đó, không phải dựng một cơ chế làm nó sống.

Vế `updated_at` thì **vẫn đóng băng** — `rebuild_from_store` đọc `work.updated_at` từ một cột
mà **0** câu `UPDATE` nào chạm, nên một lượt ghi lại `meta.json` chỉ chép lại đúng giá trị cũ.
Nợ đó có chủ Story 5.6 (đã chuyển 2026-08-27) và **không** mở ở đây: bơm nó trong giao dịch
flush làm cổng đang xanh `segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
đỏ, tức mở lại một AC đã ký của Story 2.3.

### AC3 nói "`store::Writer`", Quyết định #3 của Story 1.15 nói "NGOÀI closure ghi"

AC3 đọc nguyên văn: *"ghi bởi **chính `store::Writer` của Tác phẩm đó**, trong cùng thao tác
logic với thay đổi sinh ra nó"*. Mã hôm nay ghi `meta.json` ở **tầng thao tác**
(`commands/*`), ngay sau khi giao dịch đã commit — và doc-comment của `core/library/meta.rs`
cấm tường minh gọi `write_atomic` bên trong closure của `Store::write`.

Hai câu này không mâu thuẫn nếu đọc AC3 là mệnh đề **quyền sở hữu**, không phải mệnh đề vị trí
gọi: *đường ghi của chính Tác phẩm đó, và chỉ đường đó, mới được chạm `meta.json`; nó chạy
trong cùng thao tác logic, không phải một lượt nền tách rời.* Câu kế của FR7 (AC4: *"không
thành phần nào khác ghi vào `meta.json`"*) xác nhận đúng cách đọc này — nó nói về **chủ**, không
về tầng. Story này vì thế **không** kéo `write_atomic` vào trong giao dịch SQL (một lượt ghi hệ
tệp bên trong một giao dịch SQLite kéo dài giao dịch và không hoàn tác được cùng nó), và thay
vào đó cưỡng chế vế quyền sở hữu bằng cổng ở nhiệm vụ 5. Nếu Ice muốn đúng chữ AC3, đó là một
`AD` mới, không phải một dòng mã.

### Vì sao `Option<u32>` chứ không `u32`

Cùng bàn cân đã dùng cho `status: Option<String>` ở Story 5.4, và nó đắt hơn ở đây: `0` là một
giá trị **hợp lệ và thường gặp** (Tác phẩm chưa dịch), nên `#[serde(default)]` về `0` làm một
`meta.json` v2 chưa di trú **không phân biệt được** với một Tác phẩm thật sự chưa xong Chương
nào. Với `Option`, `None` nói *chưa biết* và màn hình nói đúng như thế.

### Cái bẫy ở `match status_override`

```rust
let (status, status_is_override) = match status_override {
    Some(raw) => (Some(raw), true),          // <-- nhánh này KHÔNG chạm chapter_status_rows
    None => { /* phân giải + derive_work_status */ }
};
```

Đặt phép đếm bên trong nhánh `None` biên dịch sạch, qua mọi cổng hiện có, và làm **mọi** Tác
phẩm có ghi đè thủ công mất tiến độ. Vòng phân giải phải nâng lên trước `match`; ca hợp đồng
"ghi đè thủ công vẫn có tiến độ" (nhiệm vụ 6) là thứ canh chỗ này.

## Verification

**Commands:**

- `npm run build` -- expected: dựng `dist/` (chạy TRƯỚC `cargo test`, thiếu nó thì gãy ở khâu
  biên dịch chứ không ở một assert).
- `cargo test --locked --manifest-path src-tauri/Cargo.toml` -- expected: toàn bộ xanh, gồm
  `meta_write_boundary.rs` mới và các ca hợp đồng đã cập nhật.
- `npm test` -- expected: toàn bộ vitest xanh, gồm ca `libraryWorks.test.ts` mới.
- `npm run check:i18n && npm run check:tokens && npm run check:commands && npm run check:panel-refs && npm run check:debt-owner`
  -- expected: exit 0 cả năm; `check:tokens` xác nhận không màu viết thẳng ở thanh tiến độ,
  `check:debt-owner` xác nhận mục nợ vừa sửa không mồ côi.
- `npm run test:e2e` -- expected: `story-5-5-progress.e2e.mjs` xanh. ⚠️ Bộ e2e nằm NGOÀI
  `pre-push` và ngoài nhịp `push` của CI — một lượt push xanh không nói gì về nó; chạy tay.

**Đối chứng bắt buộc (một bộ test xanh KHÔNG chứng minh chỗ nối được canh):**

- Gỡ phép đếm ra khỏi `rebuild_from_store` (trả cứng `Some(0)`) rồi chạy lại `cargo test` +
  `npm test` -- expected: **đỏ**. Nếu xanh, chỗ nối chưa có ai canh.
- Đặt phép đếm vào TRONG nhánh `None` của `match status_override` rồi chạy lại -- expected:
  ca "ghi đè thủ công vẫn có tiến độ" **đỏ**.
- Thêm một dòng dựng tay gọi `WorkMeta::write_atomic` vào một tệp ngoài ba tệp được miễn trừ,
  chạy `cargo test` -- expected: `meta_write_boundary.rs` **đỏ**; gỡ dòng đó ra, xanh lại.

## Auto Run Result

Status: done
Ngày: 2026-08-28 · baseline `b4baa1f292d9f46a0363ca5f349db6d06611eac2`

### Thay đổi đã dựng

Kéo MỘT đại lượng — số Chương đã xong — đi trọn bốn tầng đã có: đếm tại
`WorkMeta::rebuild_from_store` (chỗ duy nhất tính giá trị dẫn xuất), cache vào `meta.json`
(lược đồ 2 → 3), chép nguyên vẹn vào `library_work.chapter_done_count`
(`LIBRARY_INDEX_MIGRATIONS` 4 → 5, viết lại DDL TẠI CHỖ đúng luật kho dẫn xuất), qua dây ở
`WorkRow`, rồi hiện "đã xong / tổng" kèm thanh tiến độ có `role="progressbar"` ở danh sách
Tác phẩm. AC4 đóng bằng một cổng ranh giới THẬT, không bằng một lượt rà tay.

### Tệp đã đổi

- `src-tauri/src/core/library/meta.rs` — trường `chapter_done_count: Option<u32>`; nâng
  `META_SCHEMA_VERSION` 2 → 3; **nâng vòng phân giải `chapter.status` ra NGOÀI**
  `match status_override`.
- `src-tauri/src/core/store/schema.rs` — cột `chapter_done_count INTEGER` (cho `NULL`),
  `to_version` 4 → 5; sửa TẠI CHỖ mệnh đề "không cột tiến độ (chủ Story 5.5)" đã hết đúng.
- `src-tauri/src/core/library/indexer.rs` — cột vào UPSERT, vào `COLUMNS` đường đọc, vào
  `IndexedWork`.
- `src-tauri/src/commands/library.rs` — trường vào `WorkRow` + `From<IndexedWork>`.
- `src-tauri/src/commands/{project,lifecycle}.rs` — chú thích chẩn đoán kèm phép đo; không
  đổi logic.
- `src-tauri/tests/meta_write_boundary.rs` (MỚI) — cổng ranh giới của AC4, 14 ca.
- `src-tauri/tests/{library_index,library_commands,project,lifecycle,ipc}_*.rs` — ca cho từng
  hàng §I/O Matrix.
- `src/config/library.ts` · `src/i18n/vi.json` · `src/modes/LibraryMode.vue` ·
  `tests/frontend/libraryWorks.test.ts` — bề mặt và ca của AC1.
- `e2e/specs/story-5-5-progress.e2e.mjs` (MỚI) — AC5 đầu-tới-cuối trong WKWebView thật.
- `deferred-work.md` · `src-tauri/AGENTS.md` — đóng vế `chapter_count` của một món nợ bằng
  phép đo lại; vế `updated_at` để mở, chủ Story 5.6.

### Vòng rà

7 vá (3 medium · 4 low) · 3 hoãn (3 medium) · 6 bác · 0 intent_gap · 0 bad_spec.
Điểm khuyến nghị rà tiếp: 3×3 + 1×4 = **13** ≥ 5 ⇒ `followup_review_recommended: true`.

### Nghiệm thu đã chạy

- `npm run build` xanh · `cargo test --locked --no-fail-fast` **869 ca, 0 đỏ** ·
  `npm test` **45 tệp / 602 ca** · bảy cổng tĩnh xanh.
- Kiểm toán §I/O Matrix: **8/8 hàng** có ca CHẠY THẬT và xanh.
- **Bốn đối chứng gỡ-chỗ-nối đã chạy thật, không suy luận** — mỗi cái đỏ đúng ca dự đoán rồi
  khôi phục: (1) giá cứng số đếm về 0 ⇒ 2 ca đỏ ở 2 binary *(lượt đầu chỉ thấy 1 ca vì thiếu
  `--no-fail-fast`; `cargo test` dừng ở binary đỏ đầu tiên — con số 1 đó là ảo)*; (2) đặt phép
  đếm vào trong nhánh `None` ⇒ đúng ca ghi-đè đỏ; (3) dòng `write_atomic` dựng tay ở
  `commands/chapter.rs` ⇒ cổng ranh giới đỏ kèm `tệp:dòng`; (4) giả lại hàm cắt chú thích thô
  ⇒ ca tự kiểm `aura://` đỏ.
- **Bộ e2e: 16/16 xanh (21m49).**

### Rủi ro còn lại

- Bộ e2e KHÔNG nằm trong `pre-push` lẫn nhịp `push` của CI, và nửa Windows/WebView2 chưa từng
  chạy — một lượt push xanh vẫn không nói gì về nó.
- Ba món nợ ở frontmatter `deferred`, mỗi món có chủ. Đáng chú ý nhất là khuyết tật BÀN ĐO
  (bãi đỗ tạm của `story-5-3-rescan` dùng chung thư mục tạm hệ thống giữa các lượt) — nó làm
  hai lượt e2e giữa phiên này đỏ và đã được loại trừ khỏi story bằng bốn phép đo.
- Vế "trong Workspace" của AC5 chưa dựng (chủ Story 5.7); vế "quay về Library" đã đóng và có
  đối chứng.

