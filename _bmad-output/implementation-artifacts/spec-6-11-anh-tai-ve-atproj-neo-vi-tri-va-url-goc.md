---
title: 'Story 6.11: Ảnh tải về `.atproj`, neo vị trí, và URL gốc'
type: 'feature'
created: '2026-09-08'
status: 'done'
baseline_commit: 'f97e97b2ea5b4ecfcb8cde0a86d210eae6579f77'
review_loop_iteration: 2
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ảnh trong bài tải từ web bị vứt trọn vẹn, và cả ba nửa của đường đi đều đã dựng
sẵn rồi **bỏ không**: `BlockBody::Image { src, alt }` có từ Story 6.9 với **0 chỗ gọi sản
phẩm**, `ResourceKind::Image` và `Allowlist::with_tier2_hosts` có từ Story 6.8 với **0 chỗ
gọi**, `assets/` được tạo ở mọi lượt tạo Tác phẩm từ Story 1.15 với **0 đường ghi**. Đích lược
đồ `project.db` là **19** và không có bảng `asset`, không cột `source_url` nào. Vì AD-4 đóng
băng `source_text` lúc nhập, một bài có hình minh hoạ mất hình **vĩnh viễn** ngay ở lượt nhập —
không có đường phục hồi sau này.

**Approach:** Ở bước **xác nhận nhập** (`create_work`), tải đúng những ảnh **đang GIỮ** qua tầng
2 của allowlist đã có, ghi thành **tệp thật** trong `assets/`, và ghi một hàng `asset` mang
**neo vị trí trong Chương** cộng `source_url` (rỗng hợp lệ). Neo tính **một lần lúc nhập** bằng
cách chạy lại hai bước biến đổi văn bản trên **tiền tố** đứng trước ảnh — đo được là chính xác,
và không đụng `PIPELINE_ORDER`.

## Boundaries & Constraints

**Always:**
- Lời gọi mạng cho ảnh xảy ra **CHỈ** trong `create_work`, tức sau thao tác xác nhận của người
  dùng. **0** lời gọi mới ở đường xem trước (AD-15: không tải nền, không prefetch).
- Tầng 2 dựng từ host của **đúng những ảnh `effective_kept_for_blocks` trả `true`** — không phải
  mọi `<img>` của trang; nạp qua `Allowlist::with_tier2_hosts`, tải qua
  `fetch(url, &allowlist, ResourceKind::Image)`. Tầng 2 **không bao giờ** tải tài liệu (AD-41).
- Mọi `Vec<DomainLogEntry>` mà `fetch` trả về phải nối vào `DomainLogState` — kể cả lượt trượt.
- Ghi **tệp** nằm NGOÀI closure `store.write` (khuôn `meta.rs:205-206`); ghi **SQL** đi qua
  `store::Writer` như mọi lệnh ghi khác.
- `asset.source_url` cho phép `NULL` — ảnh người dùng tự thêm là ca hợp lệ, không phải lỗi.
- Neo lưu xuống đĩa **một lần lúc nhập**; không đường mã nào tính lại lúc nạp (cùng luật ranh
  giới segment).
- Bước di trú **20** mới với DDL mới. Không sửa một hằng DDL đã chạy trên đĩa.
- Một ảnh trượt tải **không** làm trượt cả lượt nhập; nó không sinh hàng `asset`, và lý do trượt
  có mặt trong nhật ký domain — không biến mất im lặng.

**Ask First:**
- **Có đặt trần tổng byte ảnh cho MỘT lượt nhập không?** Đề xuất: **KHÔNG thêm ngưỡng mới** —
  trần `MAX_RESPONSE_BYTES = 20 MiB` mỗi phản hồi đã có, và phép đo trên bảy mẫu thật của bàn đo
  6.1 cho **8 ảnh GIỮ / 513 thẻ `<img>`** (≈ 1,1 ảnh mỗi trang, 5/7 trang giữ 0 ảnh). Đây đúng
  hình dạng mà Ice đã bác ở Story 6.7 (*"một hằng số ngưỡng chưa đo được"*). Nếu Ice muốn một
  trần, nó cần một số đo, không một con số chọn cho tròn.
- Nếu phép đo tiền tố (§Design Notes) **lệch** trên dữ liệu thật ngoài bảy mẫu, dừng và nêu —
  đừng đổi phép tính neo cho hết lệch.

**Never:**
- **Không đụng `src-tauri/tauri.conf.json` hay `capabilities/main.json`.** Bề mặt HIỂN THỊ ảnh
  là Story 6.14; `assetProtocol.scope` hôm nay là `$RESOURCE/fonts/**` và
  `config_invariants.rs:313` khoá đúng một phần tử đó.
- Không loại ảnh thứ hai chỉ mang link (AD-9). Không đường nào để một URL từ xa đi vào chỗ
  hiển thị (AD-16 §3).
- Không sinh `Segment` vai `alt`/`caption` — đó là Story 6.13 (AD-42).
- Không thêm phụ thuộc mới. `reqwest::Url` (phân giải URL tương đối) và `uuid` đã có.
- Không đổi `PIPELINE_ORDER`, không đổi hợp đồng của `core::cleanup` hay `normalize`.
- Không nhận `image/svg+xml` — SVG là **đánh dấu**, không phải ảnh raster; nhận nó mở lại đúng
  bề mặt mà AD-16 tồn tại để bịt.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Ảnh giữ, host tầng 2, MIME raster | `<img src>` tuyệt đối, `image/jpeg` | Tệp trong `assets/<uuid>.jpg`; một hàng `asset` mang neo + `source_url` | N/A |
| `src` tương đối hoặc rời giao thức | `/a.jpg`, `//cdn/x.jpg` | Phân giải tuyệt đối theo URL trang rồi mới hỏi allowlist | `src` không phân giải được ⇒ bỏ ảnh, ghi nhật ký |
| Ảnh cùng `source_url`, cùng Tác phẩm | Ảnh đã có hàng `asset` | **0** lời gọi mạng; dùng lại tệp đã có (AD-41) | N/A |
| Host ngoài tầng 2 | `src` trỏ host lạ | `Denied` **trước khi** mở kết nối; bỏ ảnh | Một hàng nhật ký `Denied` |
| MIME không phải ảnh raster nhận được | `text/html`, `image/svg+xml` | Bỏ ảnh, **không** ghi tệp | Một hàng nhật ký, lý do phân biệt được |
| Phản hồi vượt trần | thân > 20 MiB | Cắt giữa chừng, bỏ ảnh | `FetchError::TooLarge` vào nhật ký |
| Ảnh trượt giữa lượt nhập N Chương | 1/8 ảnh trượt | 7 hàng `asset`; Chương vẫn nhập đủ; `create_work` trả số ảnh trượt | Không ném |
| Ảnh không có `alt` | `<img>` trần | Vẫn có hàng `asset` và vẫn giữ đúng vị trí nhờ neo riêng | N/A |
| Ảnh do người dùng tự thêm | không có URL | `source_url IS NULL` — hợp lệ | N/A |
| Ghi tệp trượt giữa chừng | đĩa đầy | Lượt nhập trượt sạch, thư mục `.atproj` bị dọn (đường `remove_folder` đã có) | `IpcError` có `message_key` |
| Đường nhập tệp/dán tay | `PipelineShape::Blob` | **0** ảnh, **0** lời gọi mạng, byte ghi xuống trùng đúng trước story | N/A |

</frozen-after-approval>

## Code Map

**Máy đã dựng, chưa nối — ba chỗ có 0 lời gọi sản phẩm**
- `src-tauri/src/core/webimport/extractor.rs:120` — `BlockBody::Image { src: Option<String>, alt: Option<String> }`; `:220-225` là nơi DUY NHẤT đọc thuộc tính `<img>`; `:379` `infer_image_kept_state` suy `machine_kept` cho ảnh từ khối chữ lân cận. ⚠️ `srcset`/`<picture>` **0 hit** — chỉ `src`.
  🔴 `extractor.rs` **KHÔNG** được phân giải URL tuyệt đối tại chỗ: `webimport_boundary.rs:229` `extractor_carries_zero_lines_naming_a_network_client_or_a_literal_url_scheme` cấm mọi dòng nhắc `reqwest` hoặc một literal `http://`/`https://` trong tệp đó.
- `src-tauri/src/core/webimport/allowlist.rs:109` `with_tier2_hosts` — **0 chỗ gọi**; `:119` `decide` là ma trận 2×2 cạn; `:139` `host_of` dùng `reqwest::Url::parse`.
- `src-tauri/src/core/webimport/fetcher.rs:230` `fetch(url, allowlist, kind) -> (Result<FetchedPage, FetchError>, Vec<DomainLogEntry>)`; `FetchedPage { bytes, content_type }` `:177`. `ResourceKind::Image` **0 chỗ gọi** (chỗ gọi sản phẩm duy nhất là `commands/project.rs:2573`, luôn `Page`). Client dùng chung `shared_client()` `:153` — tái dùng được nguyên trạng. `looks_like_html:360` so MIME **bằng**, không `contains` — khuôn để chép cho vị từ MIME ảnh mới.
- `src-tauri/src/core/library/atproj.rs:24` `ASSETS_DIR`, tạo ở `:183`. `:156` ghi thẳng lý do: *"Epic 6 không phải kiểm tra sự tồn tại của nó ở mọi đường ghi ảnh"* ⇒ **không tạo lại**.

**Chỗ ảnh còn sống tới lúc xác nhận**
- `src-tauri/src/core/segment/import.rs:412` `ImportedChapter` — trường `blocks: Option<Vec<Block>>` (thêm ở 6.9) chở **mô hình khối CẢ TRANG** của Chương đó. ⇒ `create_work` **đã có** danh sách ảnh; **không phải đổi pipeline**.
- `src-tauri/src/core/segment/pipeline.rs:876-877` `BlockBody::Image { .. } => None` — nơi ảnh bị bỏ khỏi **văn bản** (đúng, không sửa); `:841` `effective_kept_for_blocks` và `:869` `join_kept_blocks` là hai hàm thuần dùng chung để biết ảnh nào GIỮ.
- `src-tauri/src/core/segment/pipeline.rs:166` `ChapterInput::RawBytes { bytes, label }` — `label` **là URL trang** trên đường URL (`:580` truyền nó thẳng vào `extract`). ⇒ URL trang và tập URL tầng 1 đọc được từ `shape` trước khi nó bị `run_import` nuốt.

**Chỗ ghi**
- `src-tauri/src/commands/project.rs:308` `create_work(...)` — trình tự đã có: `:319` tạo thư mục+`assets/` → `:320` mở `Store` → `:369` `run_pipeline` (NGOÀI closure) → `:417` `store.write` một giao dịch (`INSERT INTO work` `:418`, vòng lặp `INSERT INTO chapter` `:437-442`, `insert_segments` `:449`) → **sau commit** `:468` `rebuild_from_store` → `:497` `meta.write_atomic` → `:508-511` đường dọn khi trượt. 🔴 `:417` ghi bằng chữ *"job ghi CHỈ SQL — không `fs::write` nào bên trong closure này"*.
- `src-tauri/src/core/store/schema.rs:1547` `PROJECT_MIGRATIONS`, bước cuối `:1665-1668` `to_version: 19`; `:1929` `target_version` tính từ phần tử cuối. Khuôn bảng vệ tinh theo Chương đã có: `chapter_position` `:788`, `reading_mark` `:806`. 🔴 **0 `FOREIGN KEY`** trong toàn lược đồ, lý do ở `:1024-1027`.
- `src-tauri/src/core/library/indexer.rs:1309` `MINIMUM_HARVEST_SCHEMA_VERSION = 8` — luật ở `:1305`: nâng `PROJECT_MIGRATIONS` mà **đường đọc gõ thêm cột mới** thì phải nâng sàn này. Story này **không** đổi đường đọc của `Indexer` ⇒ sàn **giữ nguyên 8**, và đó là một mệnh đề phải khẳng định bằng test chứ không bằng im lặng.
- `src-tauri/src/commands/project.rs:3919` — chỗ duy nhất gọi `append_domain_log_entries`; `commands/project.rs:2350` `confirm_import_with_encoding` → `:2385` `create_work` là đường sản phẩm từ UI.

**Cổng sẽ nói gì**
- `src-tauri/tests/config_invariants.rs:313/335/347` — **đỏ nếu** story này chạm `assetProtocol.scope` hay `capabilities/main.json`. Chúng phải **ở nguyên màu xanh**; đó là phép đo cho §Never.
- `src-tauri/tests/webimport_contract.rs:1037` `an_image_request_to_a_tier_2_only_host_is_allowed` — ca tầng 2 đã có, **đừng dựng nguồn sự thật thứ hai**; `:986/:1012/:1057` là ba ca AD-41 còn lại.
- `src-tauri/tests/webimport_boundary.rs:270` `reqwest_is_named_only_inside_core_webimport_or_core_ai` — module mới phải nằm trong `core/webimport/`.
- `src-tauri/tests/segment_pipeline_boundary.rs::pipeline_order_matches_ad_39_step_by_step` và `::run_import_is_the_one_product_call_site` — **không được đụng**.
- 🔴 `deferred-work.md:10409` — `cargo test --locked` trên máy Ice **có lúc** đỏ 13–14 ca của `webimport_contract.rs` vì Application Firewall macOS chặn binary test, **không phải một dòng mã**. Đo bằng phép GỠ thật: baseline `83d5c88` cho 14 đỏ. Đừng đọc một lượt đỏ ở nhóm ca đó là hồi quy của story này trước khi kiểm điều kiện đó.

**Sổ nợ có chủ Story 6.11 — ba mục, phải đóng bằng chữ trong cùng lượt**
- `deferred-work.md:10156` — `Extractor` bỏ toàn bộ ảnh/caption/alt-text.
- `deferred-work.md:10296` — AC *"không tải lại ảnh đã có"* không dựng được ở 6.8 vì **0 bảng ảnh, 0 cột `source_url`**.
- `deferred-work.md:10342` — `BlockBody::Image`/`Caption` có mặt, 0 chỗ gọi sản phẩm; `infer_image_kept_state` chưa qua dữ liệu thật ngoài bảy mẫu. ⚠️ Mục này ghi nhầm vai *"6.11 (hiển thị ảnh) / 6.13 (tải ảnh về `.atproj`, FR127)"* — theo `epics.md` thì **6.11 là FR127 (tải về)** và **6.14 là hiển thị**. Ghi chỗ lệch, đừng sửa mục cũ (sổ nợ không sửa mục đã có).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store/schema.rs` — hằng `ASSET_DDL` mới + bước `Migration { to_version: 20, sql: ASSET_DDL }` nối vào CUỐI `PROJECT_MIGRATIONS:1547`. Cột: `id` (`AUTOINCREMENT`, không tái dùng — cùng lý do `segment`), `chapter_id`, `file_name` (tên tệp **tương đối** trong `assets/` — 🔴 không đường dẫn tuyệt đối nào được ghi vào trong `.atproj`, `atproj.rs:15`), `source_url TEXT` (**NULL hợp lệ**), `anchor_after_segment_ord INTEGER NOT NULL` (`0` = trước segment đầu), `byte_len`, `content_type`, `created_at`. ⚠️ **Không** cột `work_id`: `project.db` là kho của ĐÚNG một Tác phẩm (đúng một hàng `work`), nên `chapter_id` đã xác định Tác phẩm — cạnh `WORK ||--o{ ASSET` của ERD spine là quan hệ khái niệm, không phải một cột thứ hai. Rào rỗng cho `file_name` phải liệt **trọn 25 điểm mã `White_Space`** như `GLOSSARY_ENTRY_DDL` — `trim()` của SQLite chỉ cắt dấu cách ASCII (`src-tauri/AGENTS.md`).
- [x] `src-tauri/src/core/webimport/assets.rs` (**mới**) + đăng ký ở `webimport/mod.rs:49-57` — module thuần cho ba việc: ① phân giải `src` thành URL tuyệt đối theo URL trang (`reqwest::Url::join`, hợp lệ ở đây vì `webimport_boundary.rs:270` cho phép `reqwest` trong `core/webimport/`); ② vị từ MIME ảnh raster **so BẰNG**, danh mục **ĐÓNG** `image/jpeg|image/png|image/gif|image/webp` (khuôn `looks_like_html:360`, và ghi tại chỗ vì sao `image/svg+xml` bị loại); ③ ánh xạ MIME → đuôi tệp. **Không** hàm nào ở đây chạm đĩa hay quyết định neo.
- [x] `src-tauri/src/core/segment/anchor.rs` (**mới**) + đăng ký ở `core/segment/mod.rs` — hàm **thuần** tính neo: nhận `blocks` + `effective_kept` + hai phép biến đổi văn bản, trả `Vec<usize>` neo theo **số segment đứng trước** mỗi ảnh giữ. 🔴 Hàm phải **tự kiểm**: tiền tố sau biến đổi **phải** là tiền tố của văn bản đầy đủ; không phải thì trả một lỗi phân biệt được, **không** làm tròn thành `0`.
- [x] `src-tauri/src/commands/project.rs:308` `create_work` — chèn một pha ảnh **giữa** `run_pipeline:369` và `store.write:417`: đọc URL trang từ `shape`, dựng `Allowlist::from_urls(tier1).with_tier2_hosts(host của ảnh GIỮ)`, `fetch(..., ResourceKind::Image)` từng ảnh, so `source_url` để **bỏ qua ảnh đã có**, ghi byte vào `assets/` (khuôn atomic của `meta.rs:207`/`exchange_io.rs:88`), tính neo, rồi `INSERT INTO asset` **bên trong** giao dịch `:417` cùng `chapter`/`segment`. Nối `Vec<DomainLogEntry>` qua đường `:3919`. Đường trượt dùng lại `remove_folder` `:508-511`.
- [x] `src-tauri/src/commands/project.rs` — `OpenWork` (hoặc kiểu trả về của `create_work`) mang thêm **số ảnh đã lưu** và **số ảnh trượt**, `snake_case` trên dây, **không** `rename_all`.
- [x] `src-tauri/src/core/i18n/mod.rs:567-584` — khoá mới cho lý do ảnh trượt, đúng khuôn `message_keys!` (khai một chỗ, `ALL` tự sinh); `src/i18n/vi.json` khoá tương ứng, phẳng, không giá trị rỗng.
- [x] `src-tauri/tests/asset_contract.rs` (**mới**) — hàng I/O Matrix: neo đúng khi ảnh ở đầu/giữa/cuối Chương; ảnh không `alt` vẫn giữ vị trí; `source_url IS NULL` hợp lệ; trùng `source_url` ⇒ **0** lời gọi mạng; MIME sai/SVG bị từ chối; một ảnh trượt không làm trượt Chương; đường `Blob` cho **0** ảnh và byte ghi xuống **trùng đúng** trước story.
- [x] `src-tauri/tests/webimport_contract.rs` — ca tầng 2 đi **đầu-cuối** qua `create_work` (host ngoài tầng 2 bị chặn **trước khi** mở kết nối; tầng 2 **không** tải được tài liệu). Đừng chép lại bốn ca AD-41 đã có ở `:986-1057`.
- [x] `src-tauri/tests/config_invariants.rs` — **không sửa**; chạy để chứng minh `assetProtocol.scope` và `capabilities/main.json` **không đổi** (phép đo cho §Never).
- [x] `src-tauri/tests/library_index_contract.rs` (hoặc nơi đang canh) — ca khẳng định `MINIMUM_HARVEST_SCHEMA_VERSION` **vẫn là 8** sau bước 20, và một `.atproj` lược đồ 19 vẫn thu hoạch được.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — đóng bằng chữ ba mục `:10156` · `:10296` · `:10342` (🟡 nếu chỉ đóng một nửa, liệt phần còn hở); ghi nợ **MỚI có chủ** cho: vế *"copy sang máy khác ⇒ ảnh hiển thị đầy đủ"* (**Chủ: Story 6.14** — hôm nay **0** chỗ gọi `asset_protocol_scope`/`allow_directory` trong `src-tauri/src/`), `srcset`/`<picture>` chưa đọc, `image/svg+xml` bị loại, và chỗ lệch vai ở `:10342`.

- [x] `src-tauri/src/core/webimport/domain_log.rs:55-63` — **nới `DomainLogEntry` một trường KẾT QUẢ** (Ice ký 2026-09-08, §Spec Change Log vòng rà 1) để hai ô `Error Handling` của ma trận có chỗ mà sống: một lượt `Allowed(Tier::Two)` rồi trượt phải phân biệt được với một lượt tải xong. Danh mục ĐÓNG, `snake_case`, phủ đúng các lý do đã có (`FetchError` + MIME bị loại) — 🔵 sửa TẠI CHỖ doc-comment `:52-54` (*"trừ cột Kết quả"*) vì mệnh đề đó hết đúng từ story này, kèm ngày và lý do. Cập nhật `fetcher.rs` (nơi dựng bản ghi), `webimport_contract.rs`/`ipc_contract.rs` nếu chúng khoá hình dạng, và bề mặt Cài đặt › Quyền riêng tư nếu nó đọc trường mới.

**Acceptance Criteria:**
- 🔴 Given một lượt nhập URL có ảnh giữ, when xác nhận, then mỗi ảnh có **một tệp thật** trong `assets/` và **một hàng `asset`**; và ca này phải **ĐỎ** nếu ai chỉ ghi hàng SQL mà không ghi tệp.
- 🔴 Given phép **GỠ** vị từ MIME ảnh, when chạy bộ test **CŨ**, then nó phải **ĐỎ** — đối chứng là một phép **GỠ biên dịch được và chạy**, không phải một lập luận.
- 🔴 Given một lượt nhập đường tệp/dán tay và **không thao tác tay nào**, when xác nhận, then byte ghi xuống `.atproj` **trùng đúng** kết quả trước story này, và **0** lời gọi mạng.
- 🔴 Given một `src` trỏ host ngoài tầng 2, when nhập, then host đó nhận **0** kết nối, và có một hàng nhật ký nói vì sao.
- Given `cargo test`, when chạy, then đích `PROJECT_MIGRATIONS` là **20**, `MINIMUM_HARVEST_SCHEMA_VERSION` **vẫn 8**, và `assetProtocol.scope` **vẫn đúng một phần tử**.
- Given **mười một** cổng của `pre-push` cộng `cargo test --locked` cộng `npm run test`, when chạy trọn, then **0** finding và **0** ca đỏ.

## Spec Change Log

### Vòng rà 2 — 2026-09-09 (Ice ký cùng ngày)

**Phát hiện kích hoạt.** Neo `anchor_after_segment_ord` VỠ IM LẶNG ngay khi người dùng tổ chức
lại Chương. Đo 2026-09-09: `commands/chapter.rs:748` (gộp) chạy
`UPDATE segment SET chapter_id = ?1, ord = ord + ?2`, `:913` (tách) chạy
`UPDATE segment SET ord = ord - (?2 - 1)`, `commands/segment.rs:2792`/`:2802` (gom nhóm) chạy
`UPDATE segment SET ord = ?1` — và chữ `asset` xuất hiện **0 lần** trong CẢ HAI tệp. Nặng hơn:
`chapter.rs:779` xoá `chapter_position` của Chương bị gộp NGAY TRƯỚC `:780` xoá `chapter`, tức
kỷ luật chống-hàng-mồ-côi đã tồn tại và `asset` không theo nó ⇒ sau một lượt gộp, hàng `asset`
mồ côi ở lại kèm một tệp trong `assets/` không ai xoá.

**Chỗ hổng của spec.** Doc-comment `ASSET_DDL` biện hộ neo bằng câu *"nó sống như một sự kiện đã
xảy ra, giống `chapter.source_text`"* — nhưng `source_text` bị **AD-4 đóng băng**, còn
`segment.ord` thì **không**. §Code Map của spec này không nêu `commands/chapter.rs` /
`commands/segment.rs` là bề mặt bị ảnh hưởng, nên lượt thi công không có lý do nào để nhìn tới
đó. Đây là chỗ thiếu ở phần CHƯA đóng băng, không phải ở ý định.

**Đã sửa gì.** Thêm việc: duy trì hàng `asset` trong CHÍNH bốn giao dịch đó, theo đúng khuôn
`chapter_position`. Ice chốt phương án sửa-ngay (2026-09-09) thay vì ghi nợ giao Story 6.14 hay
chặn thao tác.

**Trạng thái xấu đã tránh.** Nếu ghi nợ: từ hôm nay tới lúc Story 6.14 chạy, mỗi lượt gộp/tách
Chương của người dùng để lại neo trỏ sai và hàng/tệp mồ côi **trên máy thật** — và 6.14 sẽ phải
sửa dữ liệu đã lệch chứ không chỉ viết mã hiển thị. Không cổng nào đỏ suốt quãng đó.

### Vòng rà 1 — 2026-09-08 (Ice ký cùng ngày)

**Phát hiện kích hoạt.** Hai ô `Error Handling` trong ma trận I/O (khối ĐÓNG BĂNG) đòi nhật ký
domain phân biệt được **lý do** một ảnh ĐÃ ĐƯỢC PHÉP rồi trượt (*"lý do phân biệt được"* ·
*"`FetchError::TooLarge` vào nhật ký"*). Đo được: `DomainLogEntry`
(`core/webimport/domain_log.rs:55-63`) chỉ chở `at_epoch_ms · domain · kind · decision`, và
`decision` chỉ có `Allowed(Tier)`/`Denied` — doc-comment `:52-54` khai cột *"Kết quả"* **cố ý
không phải một trường** (quyết định Story 6.8). ⇒ Một ảnh vượt trần để lại đúng một hàng *"ĐÃ
CHO PHÉP"*, và ca test phải lùi xuống assert `Allowed(Tier::Two)` trong khi TÊN nó khai
`denied`. Đây là mệnh đề *"không rỗng im lặng"*, lớp lỗi trung tâm của dự án.

**Đã sửa gì.** Ý định ĐÓNG BĂNG **không đổi một chữ** — Ice chốt nới `DomainLogEntry` thay vì
thu hẹp AC. Phần chưa đóng băng (§Code Map + §Tasks) là chỗ thiếu: nó không nêu rằng đáp ứng
hai ô đó đòi một trường KẾT QUẢ trên `DomainLogEntry`. Thêm việc đó vào §Tasks, cộng mười bảy
mục `patch` của ba lớp rà.

**Trạng thái xấu đã tránh.** Nếu để nguyên: nhật ký Quyền riêng tư khai *"đã cho phép"* cho mọi
ảnh trượt, người dùng đọc màn đó không phân biệt được *đã tải xong* với *bị chặn giữa chừng*,
và `images_failed` là con số DUY NHẤT nói có chuyện — một con số **chưa bề mặt nào đọc**. Cộng
lại: một lượt nhập mất ảnh trông y hệt một lượt nhập không có ảnh.

🔴 **KEEP — những thứ đã ĐO và phải sống sót qua mọi lượt dựng lại:**
- Neo bằng phép **chạy lại hai bước biến đổi trên TIỀN TỐ** cộng bước TỰ KIỂM tiền tố
  (đo 8/8, lệch 0, bảy mẫu bàn đo 6.1). Không thay bằng đếm khối, không thay bằng offset-tracking.
- `extension_for_mime` là điểm gác MIME **DUY NHẤT** trên đường sản phẩm — đo bằng phép GỠ thật:
  gỡ vị từ thứ hai ⇒ **0 ca đỏ** trên 46 binary, tức nó là mã chết ở đó.
- Ghi **tệp** ngoài closure `store.write`, ghi **SQL** trong đúng giao dịch cùng `chapter`/`segment`.
- Tầng 2 dựng CHỈ từ host của ảnh đang GIỮ, và `Allowlist::default()` (tầng 1 **rỗng**) — **chặt
  hơn** chữ trong §Tasks bản đầu (`from_urls(tier1)`), vì pha này không bao giờ tải Trang.
- Ca `a_disk_write_failure_mid_asset_write…` gây lỗi bằng **chướng ngại THẬT** (`chmod` mất quyền
  ghi trên chính `assets/`), không bằng cách gọi thẳng hàm dựng lỗi.
- **0** đụng chạm `tauri.conf.json` · `capabilities/main.json` · `config_invariants.rs` ·
  `PIPELINE_ORDER` — và `config_invariants.rs` xanh là phép đo cho lời tuyên bố đó.

## Design Notes

**Vì sao neo tính được CHÍNH XÁC mà không phải đụng AD-39 — và phép đo đỡ nó.** Ảnh bị bỏ khỏi
văn bản ở bước ghép (`pipeline.rs:876`), nên vị trí của nó không sống sót qua hai bước biến đổi
sau đó (làm sạch, chuẩn hoá). Ba đường đã cân nhắc: ① đếm *"khối văn bản giữ thứ k"* — **đo là
sai**: a03 có 45 khối văn bản giữ nhưng **46** đoạn, a07 có 17 khối nhưng **19** đoạn; ② làm hai
bước kia mang theo bảng ánh xạ offset — chính xác nhưng phải mở lại hợp đồng của
`core::cleanup` và `normalize`; ③ **chạy lại hai bước ấy trên TIỀN TỐ** đứng trước ảnh rồi đếm
segment lọt vào tiền tố đó. Đo 2026-09-08 trên bảy mẫu bàn đo 6.1: **8/8 neo** cho kết quả *tiền
tố sau chuẩn hoá vẫn là tiền tố của văn bản đầy đủ* (lệch **0**), và chuẩn hoá **không** đổi số
đoạn ở cả hai mẫu có ảnh (46→46 với 5 dòng bị nối, 19→19). ⇒ Chọn ③.
⚠️ **Giới hạn thật, ghi ra thay vì để người sau tự phát hiện:** tính phân tách-theo-tiền-tố là
một **giả định đo được**, không một định lý — một luật làm sạch khớp **vắt qua** ranh giới tiền
tố sẽ phá nó. Vì thế hàm neo **tự kiểm** và trả lỗi phân biệt được thay vì làm tròn: một neo sai
im lặng đặt ảnh vào giữa một câu và không cổng nào đỏ.

**Vì sao tải ở lúc XÁC NHẬN, không ở lúc xem trước.** Không phải một lựa chọn: trước khi xác
nhận **chưa có thư mục `.atproj`** nào để mà ghi vào (`create_work_folder` chạy ở `:319`, sau
xác nhận), và màn xem trước dựng lại kết quả mỗi lần đổi ứng viên bảng mã — tải ở đó là tải lại
cùng một ảnh năm lần. Nó cũng khớp AD-41 vế *"không tải lại ảnh đã có (so `source_url`, cùng
Tác phẩm)"*: câu đó chỉ có nghĩa khi đã có một Tác phẩm để mà hỏi.

**Vì sao tên tệp là `uuid` chứ không phải băm nội dung.** Băm nội dung đòi một crate băm mới
(**0** crate băm trong `Cargo.toml` hôm nay) và cửa rà giấy phép NFR15 cho một lợi ích mà cột
`source_url` đã cấp: phép *"đã có chưa"* của AD-41 hỏi **URL**, không hỏi byte. `uuid` v4 đã là
phụ thuộc (`Cargo.toml:68`), cho tên **portable** trên cả NTFS lẫn APFS (NFR14) và không bao giờ
đụng luật ký tự cấm của `atproj.rs:59`. Đuôi tệp đến từ **MIME của phản hồi**, không từ đuôi
trong URL — một URL `.jpg` trả `text/html` là ca có thật.

**Vì sao ảnh trượt không làm trượt lượt nhập.** Một link hỏng ở Story 6.7 **khoá** nút xác nhận
vì nó là **cả một Chương** biến mất. Một ảnh hỏng thì không: văn bản vẫn đủ, và người dùng đã
bấm xác nhận rồi — huỷ trọn lượt nhập vì một ảnh 404 là đánh đổi sai chiều. Nhưng nó **không
được im lặng**: lý do đi vào nhật ký domain (bề mặt đã có ở Cài đặt › Quyền riêng tư, Story 6.8)
và số ảnh trượt đi lên dây. Bề mặt HIỂN THỊ con số đó là nợ có chủ, không phải một dòng dev tự
thêm.

## Verification

**Commands:**
- `npm run build && cargo test --locked` — expected: 0 đỏ. 🔴 `dist/` phải có **TRƯỚC** `cargo test`. ⚠️ **Tự đo và ghi số thật** (ca / binary), đừng chép từ spec cũ.
- `npm run test` — expected: 0 đỏ.
- Chạy **TỪNG** cổng: `check:deps` `check:tokens` `check:i18n` `check:commands` `check:layout` `check:panel-refs` `check:dict` `check:dict-manifest` `check:lint` `check:gates` `check:debt-owner` — **mười một**, đúng `.githooks/pre-push:81`.
- 🔴 **Đối chứng đỏ ① — tệp có thật sự được ghi không.** Bỏ bước ghi byte (giữ nguyên `INSERT INTO asset`) rồi chạy bộ test mới — expected: ca *"mỗi ảnh có một tệp thật"* **ĐỎ**. Trả lại — xanh.
- 🔴 **Đối chứng đỏ ② — hàng rào tầng 2 có thật không.** GỠ lời gọi `with_tier2_hosts` (để tầng 2 rỗng) rồi chạy — expected: ca đầu-cuối **ĐỎ** vì mọi ảnh bị `Denied`. Đây phải là một phép **GỠ THẬT**.
- 🔴 **Đối chứng đỏ ③ — vị từ MIME so BẰNG hay `contains`.** Đổi vị từ sang `contains("image/")` rồi chạy — expected: ca `image/svg+xml` **ĐỎ**.
- **Đối chứng ④ — dây.** Đặt `#[serde(rename_all = "camelCase")]` lên kiểu dây mới rồi chạy `project_contract.rs` — expected: **ĐỎ**. ⚠️ Phép biến đổi **không rỗng** (`source_url`, `anchor_after_segment_ord` là từ ghép).
- **Đối chứng ⑤ — §Never đo được.** `cargo test --locked --test config_invariants` — expected: **XANH**, đặc biệt `:313` và `:347`. Một lượt đỏ ở đây nghĩa là story đã chạm thứ nó tuyên bố không chạm.
- ⚠️ **Trước khi kết luận `webimport_contract.rs` đỏ là hồi quy:** kiểm điều kiện firewall macOS của `deferred-work.md:10409` (`socketfilterfw --getglobalstate`, so thời lượng lượt chạy). Một lỗi có ở cả hai phía không phải lỗi của thứ đang bị nghi.

**Manual checks (if no CLI):**
- Nhập một danh sách link có ảnh: mở `.atproj/assets/` trên đĩa ⇒ đúng số tệp; mở `project.db` ⇒ mỗi tệp một hàng `asset` mang `source_url`.
- Ngắt mạng giữa lượt tải ảnh: Chương vẫn nhập đủ chữ, nhật ký domain có hàng nói vì sao.
- Copy `.atproj` sang thư mục khác: **0** đường dẫn tuyệt đối nào trong `project.db` (`grep` chuỗi HOME trong dump).

## Suggested Review Order

**Neo vị trí — chỗ luật thật sự sống**

- Điểm vào: cả cơ chế neo nằm trong một hàm thuần, có bước TỰ KIỂM tiền tố.
  [`anchor.rs:62`](../../src-tauri/src/core/segment/anchor.rs#L62)

- Đếm segment lọt vào tiền tố; trả lỗi phân biệt được thay vì làm tròn về `0`.
  [`anchor.rs:130`](../../src-tauri/src/core/segment/anchor.rs#L130)

**Bảng `asset` và bước di trú 20**

- Hình dạng bảng: `source_url` cho `NULL`, rào rỗng 25 điểm mã, `chapter_id` không `work_id`.
  [`schema.rs:981`](../../src-tauri/src/core/store/schema.rs#L981)

- Bước 20 nối vào cuối `PROJECT_MIGRATIONS`; sàn thu hoạch của Indexer giữ nguyên 8.
  [`schema.rs:1828`](../../src-tauri/src/core/store/schema.rs#L1828)

**Pha tải ảnh — chạy giữa pipeline và giao dịch ghi**

- Neo tính TRƯỚC mạng; tầng 2 dựng chỉ từ host của ảnh đang GIỮ.
  [`project.rs:782`](../../src-tauri/src/commands/project.rs#L782)

- Đường `fs::write` duy nhất của cả pha; `extension_for_mime` là điểm gác MIME duy nhất.
  [`project.rs:1017`](../../src-tauri/src/commands/project.rs#L1017)

- URL trang đọc từ `shape` trước khi `run_import` nuốt nó.
  [`project.rs:700`](../../src-tauri/src/commands/project.rs#L700)

**Duy trì neo khi người dùng tổ chức lại Chương**

- Gộp Chương: hàng `asset` theo đúng khuôn segment ngay trên, cùng một `shift`.
  [`chapter.rs:784`](../../src-tauri/src/commands/chapter.rs#L784)

- Tách Chương: ảnh đi theo nửa chứa neo của nó, cùng công thức segment.
  [`chapter.rs:966`](../../src-tauri/src/commands/chapter.rs#L966)

- Gom nhóm câu: ba miền `CASE`; miền giữa mang một quyết định CHƯA AI KÝ.
  [`segment.rs:2768`](../../src-tauri/src/commands/segment.rs#L2768)

**Nhật ký domain nay chở KẾT QUẢ, không chỉ quyết định**

- Danh mục đóng tám biến thể; `MimeRejected` chỉ `commands` gán, không phải Fetcher (AD-40).
  [`domain_log.rs:63`](../../src-tauri/src/core/webimport/domain_log.rs#L63)

- Ánh xạ lý do trượt sang kết quả, gắn vào bản ghi chặng cuối.
  [`fetcher.rs:418`](../../src-tauri/src/core/webimport/fetcher.rs#L418)

- Một bản ghi lạ chỉ mất chính nó, không kéo cả nhật ký xuống trạng thái lỗi.
  [`project.ts:923`](../../src/config/project.ts#L923)

**Ngoại vi**

- Ca đầu-cuối: ảnh nhập THẬT sống sót một lượt tổ chức lại Chương không liên quan.
  [`asset_contract.rs:1354`](../../src-tauri/tests/asset_contract.rs#L1354)

- Đường huỷ lượt nhập, gây lỗi bằng chướng ngại thật — chỉ nghiệm thu được trên Unix.
  [`asset_contract.rs:648`](../../src-tauri/tests/asset_contract.rs#L648)

- Ba ca cô lập hai-Chương: một lượt tổ chức lại không chạm ảnh của Chương khác.
  [`project_contract.rs:2919`](../../src-tauri/tests/project_contract.rs#L2919)
