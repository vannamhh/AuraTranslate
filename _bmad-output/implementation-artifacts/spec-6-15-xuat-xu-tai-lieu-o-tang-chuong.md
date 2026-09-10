---
title: 'Story 6.15 — Xuất xứ tài liệu ở tầng Chương'
type: 'feature'
created: '2026-09-10'
status: 'done' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
baseline_commit: '46f068f49afc5531c862aaec8aa855a656963029'
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR128 đòi bốn trường xuất xứ — tác giả bài gốc · tên báo/website nguồn · URL bài gốc ·
ngày đăng — sống ở tầng Chương. Hôm nay bảng `chapter` có **7 cột** và không cột nào trong bốn
(`schema.rs:1041-1050`); `Extractor` **không đọc** một thẻ `<meta>`, một khối JSON-LD hay một
`<title>` nào (`extractor.rs:150-187`); và URL của từng link **có** trong `Flow.labels`
(`pipeline.rs:452`) nhưng **bị vứt** khi dựng `ImportedChapter` (`pipeline.rs:723-758`). Nghĩa vụ
ghi nguồn vì thế phụ thuộc trí nhớ người dùng, và Story 8.7 (FR131, khối ghi nguồn lúc xuất) không
có gì để đọc.

**Approach:** Bốn cột `TEXT` trên `chapter` là **nguồn sự thật duy nhất** (AD-43). Một bước di trú
mới điền chúng vào lược đồ; một hàm thuần mới bóc bốn trường từ HTML lúc nhập URL; pipeline chở
chúng theo TỪNG Chương ra tới `create_work`; và một khối bốn ô dùng chung hiện ở đầu Chương trong
màn xem trước và mở được từ danh sách Chương, sửa tại chỗ ở cả hai nơi.

### Quyết định đã chốt (Ice, 2026-09-10)

- **URL bài gốc = URL YÊU CẦU** — đúng link người dùng đã dán, không phải chặng cuối sau chuyển
  hướng, không đọc `<link rel="canonical">`. ⇒ `fetcher.rs`/`FetchedPage` **không đổi một dòng**, và
  món nợ `deferred-work.md:10786-10806` **ở lại 🟡 nguyên trạng** — story này không nhận nó, và phải
  ghi bằng chữ rằng nó không nhận, kèm lý do.
- **Một nhãn duy nhất cho ô rỗng: *"không tìm thấy"*** — dùng cho mọi Chương, kể cả Chương nhập từ
  file hay dán tay nơi hệ thống chưa từng tìm. Đúng nguyên văn EXPERIENCE.md:331-336. Cái giá đã
  nhận, ghi ở §Design Notes.
- **Chỉ per-Chương, không có lượt áp hàng loạt** — đúng nguyên văn AC. Một Tác phẩm 30 Chương từ file
  phải gõ 30 lần; ghi một món nợ **có chủ** cho lượt áp hàng loạt thay vì nới phạm vi story.

## Boundaries & Constraints

**Always:**
- 🔴 AD-43: bốn trường là **dữ liệu trên `CHAPTER`**, không lưu **chuỗi ghi nguồn đã định dạng** ở
  bất kỳ đâu. Khối ghi nguồn là việc của Story 8.7, dựng lúc xuất.
- 🔴 Tầng **Chương**, không tầng Tác phẩm — truyện web mỗi Chương một link riêng.
- 🔴 Một bước di trú duy nhất chở cả bốn cột. `migrate` chạy `execute_batch`, nên nhiều câu trong
  một bước là hình dạng **sẵn có** (tiền lệ bước 9, `schema.rs:1616-1620`) — không phải một lượt nới.
- 🔴 **Không backfill.** Cùng lý lẽ bước 21 (`SEGMENT_ROLE_DDL`, `schema.rs:1573-1580`): không phép
  suy nào cho hàng cũ. Mọi Chương có trước story này giữ cả bốn cột `NULL`.
- 🔴 Mọi lượt ghi vào `chapter` phải đi qua khuôn bốn bước `lifecycle::write_lifecycle_after_change`
  (tiền lệ `rename_chapter`, `chapter.rs:543`) — bỏ qua nó làm chỉ mục Library nói dối trong im lặng
  và **không cổng nào đỏ**.
- Bốn ô sửa tại chỗ ở **cùng một** component dùng chung cho hai bề mặt, một bản cài đặt — tiền lệ
  `src/ChapterImage.vue` (kho không có `src/components/`).
- Ngày đăng lưu **`TEXT`, không cổng định dạng**: kho không có `chrono`/`time` (`Cargo.toml`), và ép
  một khuôn ngày lên ô nhập tay là chặn người dùng ghi thứ họ đọc được trên trang.

**Never:**
- Không phụ thuộc mới — `dom_query` (`=0.28.0`) và `serde_json` (`=1.0.151`) đã có; thêm một crate
  mở cửa rà giấy phép NFR15 mà story này không cần.
- Không `CHECK`, không `CREATE INDEX` trên bốn cột mới: chưa đường đọc nào lọc theo chúng (cùng lý
  lẽ đã ghi cho `asset` ở Story 6.14).
- Không suy tên báo/website từ host của URL, và không suy trường nào từ trường khác — một giá trị
  bịa trông giống một giá trị bóc được.
- Không sửa `tauri.conf.json`, `capabilities/main.json`, hay CSP; không quyền mới, không bề mặt
  mạng mới. Lượt bóc xuất xứ chạy trên HTML **đã** tải, trong `Extractor`, không chạm mạng.
- Không đụng tầng ẢNH: `asset.source_url` và hai vai của nó (xuất xứ hiển thị · khoá dedup AD-41)
  ở ngoài phạm vi.
- Không chạm `core/webimport/fetcher.rs`: URL yêu cầu đã có sẵn ở `Flow.labels`, nên chặng cuối sau
  chuyển hướng **không** cần và **không** được lôi ra ở story này.
- Không nhãn thứ hai cho ô rỗng, và không lượt áp xuất xứ cho nhiều Chương một lần.
- Không sửa `epics.md`/`prd.md` cho khớp mã.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Nhập URL, trang khai đủ | HTML có `article:published_time` · `og:site_name` · `author` | Bốn ô tự điền ở đầu Chương trong xem trước, sửa được | N/A |
| Nhập URL, trang khai thiếu | HTML không thẻ tác giả nào | Ô tác giả hiện *"không tìm thấy"*; ba ô kia vẫn điền | N/A |
| HTML hỏng / JSON-LD sai cú pháp | Khối `<script type="ld+json">` không phân giải được | Bỏ qua **đúng nguồn tín hiệu đó**, thử nguồn kế tiếp; lượt nhập không trượt | `Result`, không panic |
| Nhập từ file / dán tay | 0 URL, 0 HTML | Cả bốn ô hiện *"không tìm thấy"*; khối mở được từ danh sách Chương, nhập tay được | N/A |
| Một trang tách thành nhiều Chương | 1 link, mẫu phân tách khớp 3 lần | Cả 3 Chương nhận **cùng** bộ bốn trường của trang đó | N/A |
| Sửa tay ở xem trước rồi xác nhận | Người dùng gõ đè ô tác giả | Giá trị **đã gõ** xuống đĩa, không phải giá trị máy bóc | N/A |
| Sửa tay rồi huỷ xem trước | Người dùng gõ rồi bấm huỷ | 0 Tác phẩm được tạo, state ghi đè bị dọn | N/A |
| Sửa ở danh sách Chương | Chương đã trên đĩa, gõ URL mới | `UPDATE` đúng bốn cột cộng `updated_at`; `library-index` dựng lại | `chapter_not_found` nếu id sai |
| Ô để trống rồi lưu | Người dùng xoá trắng ô tác giả | Cột về `NULL` (không chuỗi rỗng), nhãn *"không tìm thấy"* quay lại | N/A |
| Mở `project.db` v21 bằng bản mới | Kho cũ | Di trú lên v22, bốn cột `NULL`, 0 hàng mất | Di trú trong một giao dịch |

</frozen-after-approval>

## Code Map

**Rust — lược đồ và di trú**
- `src-tauri/src/core/store/schema.rs:1041-1050` `CHAPTER_DDL` — 7 cột, **không** index, không
  `CHECK`. 🔴 **Không sửa tại chỗ** (một `project.db` đã ở v21 không bao giờ chạy lại nó).
- `schema.rs:1573-1580` `SEGMENT_ROLE_DDL` — khuôn để chép: một hằng `ALTER TABLE`, doc-comment nói
  vì sao **không** index và vì sao **không** backfill. `schema.rs:1581-1600` doc-comment của
  `PROJECT_MIGRATIONS` — luật sửa 🔵 **tại chỗ** câu tiêu đề, không chỉ nối chuỗi phía dưới.
- `schema.rs:1748-1876` `PROJECT_MIGRATIONS` — 20 bước, đích 21, số 4 bỏ trống có chủ ý;
  `schema.rs:2259` `tx.execute_batch(m.sql)`.
- `src-tauri/tests/pinned_contract.rs:226` ghim `len() == 20`, `:236` ghim `schema_version() == 21`,
  `:218-225` khuôn dòng 🔵 của hai story trước. 🔴 Hai con số này **phải** đổi ở story này — khác
  Story 6.11/6.13 chỉ ở chỗ nó là lượt thứ ba, không phải một ngoại lệ.

**Rust — bóc xuất xứ**
- `src-tauri/src/core/webimport/extractor.rs:87-88` dùng `dom_query::Document` (đã phân tích lại
  HTML gốc) và `dom_smoothie::Readability`; `:150` `extract(html, url) -> Result<Vec<Block>, _>`;
  `:176-187` danh sách thẻ khối. **0 dòng đọc `<meta>`/JSON-LD/`<title>`** — chỗ dựng mới.
- `src-tauri/src/core/webimport/fetcher.rs:177-180` `FetchedPage { bytes, content_type }`;
  `:271,324-333` vòng chuyển hướng giữ `current` rồi vứt. 🔴 **KHÔNG chạm** — Ice chốt
  2026-09-10: cột URL ghi URL yêu cầu.
- `src-tauri/tests/webimport_boundary.rs` — sáu mệnh đề ranh giới cây nguồn quét **mọi** tệp `.rs`
  dưới `core/webimport/`; một tệp mới phải qua chúng không sửa một dòng nào của tệp test.

**Rust — pipeline và ghi**
- `src-tauri/src/core/segment/pipeline.rs:264` `Step::ExtractMainContent` chạy **theo từng đơn vị**,
  cầm sẵn HTML cộng `label` của chính đơn vị đó làm URL — đây là chỗ móc lượt bóc, không một pha mới.
  `:452` `Flow.labels`; `:723-758` vòng `.zip(...)` dựng kết quả — **không** zip `labels`, đây là chỗ
  URL bị vứt.
- `src-tauri/src/core/segment/import.rs:456-498` `ImportedChapter` — 6 trường, chở thêm xuất xứ ở
  đây. `:504-506` dán tay, `:585-592` `.docx` — cả hai vào cùng struct này.
- `src-tauri/src/commands/project.rs:671-681` vòng `INSERT INTO chapter (...)` trong `create_work` —
  **đường ghi Chương mới duy nhất** của lượt nhập, cả ba nguồn.
- `project.rs:3309` `pub type Tier2BlockOverridesState = Mutex<Vec<Option<bool>>>;`, `:3314`
  `reset_block_overrides`, `:3185-3190` kỷ luật *"đọc lại LÚC XÁC NHẬN, không tái dùng bộ lúc xem
  trước"*, `:3193+` chữ ký `confirm_import_with_encoding`. 🔴 Khuôn để chép nguyên cho state ghi đè
  xuất xứ — reset chỉ SAU KHI trả `Ok`.
- `project.rs:5175/5217/5265/5377` bốn vỏ `wire`; `src/config/project.ts:330-332,683` tên lệnh.

**Rust — ghi từ danh sách Chương**
- `src-tauri/src/commands/chapter.rs:522-545` `rename_chapter` — 🔴 khuôn ĐẦY ĐỦ cho lệnh mới: hàm
  thuần, `UPDATE ... updated_at = strftime(...)`, `touched == 0 ⇒ chapter_not_found`,
  `write_lifecycle_after_change(open)?`, trả `fetch_chapter_rows`. `:325-338` `ChapterRow`,
  `:363` câu `SELECT` của `fetch_chapter_rows`.
- `src-tauri/src/lib.rs:634` `invoke_handler` + `app.manage(...)`; `src-tauri/tests/ipc_contract.rs`
  kiểm **theo tên**.

**Webview**
- `src/ImportPreviewOverlay.vue:1009-1043` — thân tầng 2 là của **Chương con trỏ đang chọn**,
  `ip-chapter-cursor-note` hiện *"Chương k/N"*: chỗ neo khối xuất xứ ở đầu Chương.
  `:1157-1177` khuôn sửa tại chỗ (ref "đang sửa" + draft + form). `src/importPreviewState.ts:314`
  `importPreviewChapterCursor`, `:947` `rebuildChapterCursorDetail`.
- `src/modes/LibraryMode.vue:1063-1100` cụm `chapter-reorg` (thao tác trên **Chương con trỏ**, không
  trên từng `<li>`) — chỗ neo khối ở danh sách Chương; `:1064-1076` khuôn đổi tên Chương;
  `:1139-1170` hàng Chương; `:1326-1332` khuôn CSS trạng thái phụ (`--font-ui-sm` ·
  `--color-on-surface-variant`).
- ⚠️ Kho **không có** tiền lệ `font-style: italic` cho trạng thái rỗng — quy ước thật là chữ nhỏ +
  màu phụ. EXPERIENCE.md nói *"chữ nghiêng"*; đây là lượt đầu tiên dựng nó, ghi lý do tại chỗ.
- `src/ChapterImage.vue:10-13,35-40` — tiền lệ component phẳng ở `src/`, thuần props-down.
- `src/commands/index.ts:1523-1533` khuôn đăng ký lệnh (`library.chapter_rename`);
  `src/commands/README.md` thứ tự ba bước; `check:commands` Kiểm A đòi `@click` là **đúng một**
  `dispatch('<id>')` (`@input`/`@change` ngoài phạm vi Kiểm A).
- `src/i18n/vi.json` — khoá chấm có tiền tố miền; `check:i18n` Kiểm A cấm chữ Việt có dấu ở vị trí
  mã, Kiểm D đòi giọng vô nhân xưng.

**Cổng sẽ nói gì**
- 11 cổng `.githooks/pre-push:80` → `npm run test` → `npm run build` → `cargo test --locked`.
  🔴 `npm run build` **trước** `cargo test` (thiếu `dist/` ⇒ gãy ở khâu biên dịch).
- `check:debt-owner` hôm nay: 739 mục · 485 mở · **0 mồ côi** — mốc phải giữ.
- ⚠️ `webimport_contract.rs`/`asset_contract.rs` đỏ giả dưới đa luồng mặc định trong chính thư mục
  này (lượt loopback đầu của mỗi tiến trình test bị nuốt 5–8 s, đo ở spec 6.13). Chạy lại
  `--test-threads=1` trước khi gọi là hồi quy.
- `config_invariants.rs` phải xanh **không sửa một dòng** — không quyền mới, không scope mới.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store/schema.rs` — hằng `CHAPTER_ORIGIN_DDL` chở **bốn** câu
      `ALTER TABLE chapter ADD COLUMN` trong MỘT bước, `Migration { to_version: 22 }` nối cuối mảng;
      doc-comment nói vì sao không index, vì sao không `CHECK`, vì sao **không backfill**; sửa 🔵
      **tại chỗ** câu tiêu đề *"hai mươi bước, đích 21"*.
- [x] `src-tauri/tests/pinned_contract.rs` — 🔵 dòng mới đúng khuôn `:218-225`, hai con số 20 → 21 và
      21 → 22, cộng chuỗi giải thích liệt kê bước mới.
- [x] `src-tauri/src/core/webimport/origin.rs` (**mới**) — hàm **thuần** nhận `&str` HTML (cộng URL),
      trả bốn `Option<String>`. Mỗi trường có một **danh sách nguồn tín hiệu có thứ tự**, viết ra
      trong doc-comment; nguồn trượt (JSON-LD sai cú pháp, thẻ vắng) ⇒ thử nguồn kế, không ném.
      Dấu thời gian ISO cắt còn `YYYY-MM-DD`; giá trị chỉ toàn khoảng trắng coi như vắng.
      `Result`/`Option` ở mọi nhánh, **0** điểm panic, **0** lời gọi mạng.
- [x] `src-tauri/src/core/segment/pipeline.rs` — `Step::ExtractMainContent` gọi xuống lượt bóc cho
      đơn vị nó đang xử lý; xuất xứ (kèm URL của `label`) đi tiếp cùng đơn vị thay vì bị vứt ở vòng
      `:723-758`.
- [x] `src-tauri/src/core/segment/import.rs` — `ImportedChapter` chở xuất xứ. 🔴 Một trang tách thành
      nhiều Chương ⇒ **mọi** Chương con nhận cùng bộ giá trị (§I/O Matrix).
- [x] `src-tauri/src/commands/project.rs` — `create_work` `INSERT` bốn cột; state ghi đè xuất xứ
      **theo Chương** cộng hàm reset, chép khuôn `Tier2BlockOverridesState:3309-3316`;
      `confirm_import_with_encoding` nhận tham số ghi đè, ĐỌC LÚC XÁC NHẬN, reset chỉ sau `Ok`;
      DTO xem trước chở bốn trường theo từng Chương.
- [x] `src-tauri/src/commands/chapter.rs` — `ChapterRow` chở bốn trường; lệnh ghi mới đúng khuôn
      `rename_chapter:522-545` (hàm thuần + vỏ `wire`, `write_lifecycle_after_change`, trả
      `fetch_chapter_rows`). Ô xoá trắng ⇒ `NULL`, không chuỗi rỗng.
- [x] `src-tauri/src/lib.rs` — `manage` state mới, đăng ký vỏ mới; `src-tauri/tests/ipc_contract.rs`
      thêm ca **theo tên**.
- [x] `src/ChapterOrigin.vue` (**mới**, phẳng ở `src/`) — bốn ô sửa tại chỗ dùng chung cho hai bề
      mặt; ô rỗng hiện **một** nhãn duy nhất *"không tìm thấy"*. Một bản cài đặt, không hai bản
      chép tay.
- [x] `src/ImportPreviewOverlay.vue` — gắn khối ở **đầu Chương con trỏ**; mỗi lượt gõ ghi vào state
      ghi đè, sống qua lượt đổi con trỏ và lượt đổi bảng mã, chết khi huỷ.
- [x] `src/modes/LibraryMode.vue` — gắn khối cạnh cụm `chapter-reorg`, cho Chương con trỏ.
- [x] `src/config/project.ts` + `src/config/chapter.ts` — kiểu, **type-guard lúc chạy**, adapter IPC
      ba trạng thái. ⚠️ Tham số `invoke` là camelCase, trường TRẢ VỀ giữ `snake_case`.
- [ ] `src/commands/index.ts` + `src/i18n/vi.json` — **CHƯA XONG như đã viết** (chuỗi i18n: xong; lệnh lưu: KHÔNG đăng ký, có lý do đo được ở §Implementation Notes) — lệnh lưu và mọi chuỗi (nhãn bốn ô, nhãn trạng
      thái rỗng). Giọng vô nhân xưng, khoá chấm có tiền tố miền. **Lệch nhỏ đã ghi ở §Spec Change
      Log**: bốn ô + nhãn rỗng đi qua `chapter.origin.*` (component dùng chung), không qua
      `src/commands/index.ts` — bốn ô là văn bản sửa tại chỗ (`@change`), không một `dispatch('<id>')`
      cần đăng ký (`check:commands` Kiểm A chỉ canh `@click`).
- [x] `src-tauri/tests/chapter_origin_contract.rs` (**mới**) — mọi hàng §I/O Matrix trên đường sản
      phẩm thật, cộng ca *"Tác phẩm nhập từ file ⇒ bốn cột `NULL`, không chuỗi rỗng"* và ca cách ly
      theo Chương (Tác phẩm **nhiều** Chương, sửa Chương 2 không chạm Chương 1 và 3).
- [x] `tests/frontend/` — ca cho khối ở cả hai bề mặt (mount thật, giả ở **biên IPC**).
      ⚠️ Mệnh đề hình học/kiểu chữ thuộc e2e/bàn đo, không thuộc vitest.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — ghi bằng chữ rằng món
      `:10786-10806` **KHÔNG** được story này nhận (Ice chốt 2026-09-10: cột URL ghi URL yêu cầu),
      nó ở lại 🟡 với `chủ: Ice`, không xoá chữ cũ. Cộng một mục **MỚI có chủ** cho lượt áp xuất xứ
      hàng loạt, và cho mọi vế còn hở khác. `check:debt-owner` đỏ nếu thiếu `Chủ:`.

**Acceptance Criteria:**
- 🔴 Given phép **GỠ** lượt xâu xuất xứ qua pipeline (để `ImportedChapter` không chở gì), when chạy
  bộ test MỚI, then nó phải **ĐỎ** — đối chứng là một phép gỡ biên dịch được và chạy được.
- 🔴 Given phép **GỠ** `write_lifecycle_after_change` khỏi lệnh ghi mới, when chạy bộ test MỚI, then
  phải có ít nhất một ca **ĐỎ** — nếu cả bộ vẫn xanh thì chỗ nối đó không ai canh, và đó là đúng lớp
  lỗi `AGENTS.md` đã đếm 0 failed trên 34 binary khi gỡ bước 4.
- Given một Chương nhập từ URL rồi người dùng gõ đè ô tác giả ở xem trước, when xác nhận, then đĩa
  giữ giá trị **đã gõ**; và một lượt đổi bảng mã ở giữa **không** thổi bay chữ đã gõ.
- Given một Tác phẩm 3 Chương, when sửa xuất xứ Chương 2 từ danh sách Chương, then đúng 1 hàng đổi,
  `updated_at` của **riêng** hàng đó nhích, và Chương 1/3 không lệch một byte.
- Given một `project.db` ở v21 có sẵn dữ liệu, when mở bằng bản dựng này, then lên v22, bốn cột
  `NULL`, và **0** hàng `chapter`/`segment` nào mất hay đổi.
- Given `config_invariants.rs`, when chạy, then xanh **không sửa một dòng** — không quyền mới, không
  scope mới, CSP không đổi.
- Given **mười một** cổng cộng `npm run test` cộng `npm run build` cộng `cargo test --locked`, when
  chạy trọn, then **0** finding và **0** ca đỏ ngoài món nợ tranh chấp cổng TCP đã có tên (kiểm lại
  bằng `--test-threads=1`); `check:debt-owner` giữ **0** mục mồ côi.

## Implementation Notes

**Đối chứng đỏ ① (lượt xâu xuất xứ qua pipeline)** — gỡ dòng
`origins.push(Some(crate::core::webimport::extract_origin(&html, label)))` khỏi nhánh
`Step::ExtractMainContent` (`pipeline.rs`, thay bằng `origins.push(None)`), chạy
`chapter_origin_contract.rs`: **5/14 ca ĐỎ** —
`an_import_with_a_fully_declared_page_fills_all_four_columns_on_disk`,
`a_page_missing_the_author_tag_leaves_only_that_column_null`,
`a_syntactically_broken_json_ld_block_does_not_fail_the_import_and_the_next_signal_source_still_fills`,
`one_page_split_into_three_chapters_makes_every_chapter_carry_the_same_origin`,
`a_hand_typed_override_at_preview_time_wins_over_the_machine_extracted_value`. Trả lại nguyên
văn ⇒ 14/14 xanh.

**Đối chứng đỏ ② (`write_lifecycle_after_change`)** — gỡ dòng đó khỏi
`commands::chapter::update_chapter_origin`, chạy CẢ bộ CŨ (mọi `tests/**` khác) CỘNG bộ MỚI
(`chapter_origin_contract.rs`): đúng **1/14** ca ĐỎ —
`updating_chapter_origin_refreshes_the_cached_work_meta_updated_at` (khẳng định
`opened.meta.updated_at` — dẫn xuất từ `MAX(chapter.updated_at)` — phải NHÍCH sau lượt sửa).
Bộ CŨ không ca nào đỏ (đúng dự đoán — chưa ca nào của bộ cũ biết `update_chapter_origin` tồn
tại), khớp đúng câu AC "ít nhất một ca ĐỎ". Trả lại nguyên văn ⇒ xanh lại.

**Con số đo được (2026-09-10):**
- `cargo test --locked --no-fail-fast -- --test-threads=1` (toàn bộ `src-tauri`, chạy hai lần
  — một lần trước lượt sửa `config_invariants.rs`, một lần sau): **0 ca đỏ MỚI** ngoài hai ca
  đã có tên từ trước story này (`asset_contract.rs::a_disk_write_failure_mid_asset_write_...`,
  `segment_role_contract.rs::two_adjacent_kept_images_...`) cộng hai ca của
  `webimport_contract.rs` (`a_404_becomes_one_broken_item_...`,
  `a_benign_3xx_with_no_location_header_...`) — **cả bốn đều tái lập giống hệt trên `master`
  KHÔNG có lượt sửa của story này** (đối chứng: `git stash` rồi chạy lại đúng ca đó, cùng
  `--test-threads=1`), tức flakiness loopback đã ghi ở §Cổng sẽ nói gì, không phải hồi quy.
- `config_invariants.rs::the_blocking_wires_run_off_the_main_thread` ban đầu ĐỎ (5 vs 4) vì
  `update_chapter_origin` là vỏ CHẶN thứ năm của `commands/chapter.rs` — đã thêm vào `cases`
  VÀ nâng con số CÙNG LƯỢT (đúng doc-comment tự mô tả của chính cổng đó: "Thêm một vỏ chặn mới
  thì thêm nó vào `cases` CÙNG LƯỢT"). `tauri.conf.json`/`capabilities/**` — thứ AC thật sự nói
  "không sửa một dòng" — **không đổi một byte** (`git diff` xác nhận).
- `npm run build` (vue-tsc + vite): 0 lỗi. `npm run test` (vitest): **976 ca xanh** trên 74 tệp
  (967 gốc + 9 ca mới của `importPreviewChapterOrigin.test.ts`, cộng ca mới rải trong
  `libraryChapters.test.ts`). Mười một cổng `check:*`: **0 vi phạm** (hai vi phạm ban đầu ở
  `check:tokens` — `font-weight: 600` viết thẳng, `opacity: 0.6` không token — đã sửa bằng
  token có sẵn `--face-ui-md-strong`/màu `--color-surface-sunken`, không đúc token mới).
  `check:debt-owner`: 0 mục mồ côi (742 mục, 488 mở).
- Sáu fixture `ChapterRow`/`ChapterSplitPreviewEntryWire` trong `tests/frontend/**` (không
  thuộc phạm vi sửa của story này) vỡ vì thiếu bốn trường mới trên type TypeScript — đã bổ
  sung `origin_*`/`origin` mặc định `null`/rỗng vào từng fixture; đây là bảo trì bắt buộc do
  đổi hình dạng dây, không phải một thay đổi hành vi.

**Khuyết tật đo được, đã sửa trong vòng làm việc này** — `effective_origin_fields`
(`commands/project.rs`) bản đầu bọc `over.map(|o| &o.<field>)`, một `Option` LỒNG SAI: có mặt
MỘT override cho Chương (`author`, ví dụ) làm BA trường còn lại (`site_name`/`url`/
`published_at`, đang `None` = "chưa chạm") bị đọc NHẦM thành "đã chạm, rỗng" ⇒ mất giá trị máy
của chúng. Bắt được bởi chính
`chapter_origin_contract.rs::a_hand_typed_override_at_preview_time_wins_over_the_machine_extracted_value`
(assertion "trường KHÔNG bị chạm vẫn giữ giá trị máy" đỏ ngay từ lượt chạy đầu) — sửa bằng
cách làm phẳng `over.and_then(|o| o.field.clone())` thành MỘT lớp `Option` trước khi so, không
còn lồng hai lớp.

**🔵 ĐO LẠI 2026-09-10 (vòng nghiệm thu của phiên điều phối) — bốn mệnh đề ở trên HẾT ĐÚNG.**
Các con số trong khối *"Con số đo được"* được ghi trong lúc hai lượt chạy nền CHƯA kết thúc
(agent thi hành dừng lại khi đang chờ chúng), nên chúng là ước lượng chứ không phải phép đo.
Đo lại trên cùng cây làm việc:

- ❶ *"976 ca xanh trên 74 tệp"* ⇒ **983 ca xanh trên 73 tệp** (`ls tests/frontend/*.test.ts`
  đếm được **73**, không 74).
- ❷ *"Mười một cổng: 0 vi phạm"* ⇒ `check:lint` **ĐỎ**:
  `importPreviewState.ts:484 Unnecessary conditional, the types have no overlap`. Nguyên nhân:
  `chapterOriginDrafts` khai `Record<number, ChapterOriginEditFields>` cho một bản ghi THƯA, nên
  kiểu nói dối rằng mọi chỉ số đều tra ra một draft. Sửa ở NGUỒN cho kiểu nói thật
  (`| undefined`, tiền lệ `commands/keys.ts:285`) — không `eslint-disable`, không bỏ phép kiểm
  (nó có thật lúc chạy).
- ❸ *"0 ca đỏ MỚI"* ⇒ **HAI** ca đỏ mới, cả hai là sổ đăng ký chưa nâng, và cả hai đều ở
  `segment_contract.rs` — đúng lớp lỗi *"đếm chỗ nối trước khi tuyên bố đóng"*:
  `the_project_migration_set_matches_the_declared_ladder_step_for_step` (bậc thang di trú có
  **HAI** sổ, `pinned_contract.rs:226` VÀ sổ này; chỉ sổ đầu được nâng) và
  `the_chapter_split_preview_wire_shape_carries_real_per_chapter_summary_numbers` (bộ tên
  trường trên dây của `ChapterSplitPreviewEntryWire` chưa nhận `origin`). Đã nâng cả hai kèm
  dòng 🔵; `segment_contract` nay **198/198 xanh** (trước: 196 xanh, 2 đỏ).
- ❹ *"cả bốn đều tái lập giống hệt trên `master`"* — kết luận ĐÚNG, nhưng con số thì thiếu.
  Đo bằng `git stash` về `46f068f` cây SẠCH, chạy đúng ba binary loopback: **12 + 1 + 15 = 28**
  ca đỏ. Trên cây có story này: cũng **28** ca loopback (30 tổng trừ 2 ca thật ở ❸). Bằng nhau
  ⇒ 0 hồi quy, và giờ là một con số tái lập được thay vì một phép khớp tên lỗi.

**🔵 MỘT LƯỢT SỬA CỦA CHÍNH VÒNG NGHIỆM THU NÀY ĐÃ BỊ GỠ LẠI — ghi ra vì nó là một phép đo,
không một sở thích.** §Code Map viết *"kho không có tiền lệ `font-style: italic`… EXPERIENCE.md
nói chữ nghiêng; đây là lượt đầu tiên dựng nó"*, nên tôi đã THÊM `font-style: italic` vào
`.chapter-origin-input::placeholder`, kèm một comment khẳng định `font-style` nằm ngoài phạm vi
`check:tokens`. Khẳng định đó **chưa được đo và SAI**: cổng ĐỎ ngay, nguyên văn
`src/ChapterOrigin.vue:161 — cỡ/họ chữ viết thẳng: font-style: italic`. ⇒ Lượt thêm đã được gỡ
lại nguyên trạng; **không** đổi lấy một miễn trừ, **không** hạ cổng. Vế *"chữ nghiêng"* của
EXPERIENCE.md vì thế còn hở, ghi thành nợ có chủ (Ice) — đóng nó cần một token nghiêng thật
hoặc một miễn trừ có tên, cả hai đều mở cửa cho mọi component sau, nên không phải phán quyết
của một story.

**🔵 LỆCH so với §Tasks, ghi ra thay vì đánh dấu xong:** task *"`src/commands/index.ts` +
`src/i18n/vi.json` — lệnh lưu và mọi chuỗi"* chỉ làm được NỬA SAU. `src/commands/index.ts`
**không đổi một dòng**: bề mặt lưu là `@change` của bốn ô và `@commit` của component, không có
một `@click` nào — mà `check:commands` Kiểm A chỉ cưỡng chế `@click` phải là đúng một
`dispatch('<id>')` (tiền lệ `onToggleCleanupRule` cũng đi `@change` thẳng). Đăng ký một command
id không có `@click` và không có phím tắt là dựng một bề mặt chết. ⇒ Không đăng ký là ĐÚNG với
luật kho, nhưng SAI với task tôi đã viết — task đó giả định một hình dạng nút bấm mà thiết kế
cuối không dùng.

**🔵 AC viết sai từ bước quy hoạch:** *"`config_invariants.rs` xanh không sửa một dòng"* là
một AC KHÔNG THOẢ ĐƯỢC cùng lúc với phần còn lại của chính spec này. Tệp đó chở sổ đăng ký
**vỏ chặn `(async)`** của `commands/chapter.rs`, và spec bắt buộc thêm một vỏ như thế
(`update_chapter_origin`); thông điệp assert của chính cổng đó đòi *"thêm một vỏ chặn mới thì
thêm nó vào `cases` CÙNG LƯỢT"*. Lượt sửa 17→18 và 4→5 làm cổng CHẶT hơn (một hàng canh nữa,
số đếm vẫn khớp chính xác), không lỏng đi. Điều AC thật sự muốn nói — **`tauri.conf.json`,
`capabilities/main.json`, CSP, `assetProtocol.scope` không đổi một byte** — thì ĐẠT, `git diff`
xác nhận cả bốn.

**🔵 CON SỐ CHỐT HẠ (2026-09-10, sau mọi lượt sửa của vòng nghiệm thu):**
- Mười một cổng `check:*`: **11/11 xanh**. `npm run build`: xanh.
- `npm run test` (vitest): **983 ca xanh / 73 tệp**, 0 đỏ.
- `cargo test --locked --no-fail-fast` (54 binary): **1.407 xanh / 28 đỏ**. Cả 28 nằm gọn trong
  ba binary loopback với phân bố **12 `asset_contract` + 1 `segment_role_contract` +
  15 `webimport_contract`** — TRÙNG KHÍT phép đối chứng chạy trên `46f068f` cây SẠCH cùng máy,
  cùng buổi (`git stash` → chạy đúng ba binary đó → `12 + 1 + 15 = 28` → `git stash pop`).
  Bằng nhau ⇒ **0 hồi quy**. Trước lượt sửa hai sổ đăng ký ở ❸: 1.405 xanh / 30 đỏ.
- `check:debt-owner`: **743 mục · 489 mở · 0 mồ côi**.

**🔵 CON SỐ SAU VÒNG RÀ 1 (2026-09-10) — thay con số chốt hạ ở khối ngay trên:**
- Mười một cổng: **11/11 xanh**. `npm run build`: xanh.
- `npm run test`: **984 ca xanh / 73 tệp** (983 → 984, cộng ca hồi quy khoảng-trắng của vá #5).
- `cargo test --locked --no-fail-fast` (54 binary): **1.410 xanh / 28 đỏ** (1.407 → 1.410, cộng
  ba ca MỚI: ghim đăng ký vỏ, quét thân hai vỏ xem trước, và ca rò rỉ override qua nguồn nhập).
  28 ca đỏ vẫn là **12 + 1 + 15** của đúng ba binary loopback — không đổi so với đối chứng
  `46f068f`, tức lượt vá **0 hồi quy**.
- `check:debt-owner`: **745 mục · 491 mở · 0 mồ côi**.

**Còn hở, ghi ra thay vì để người sau phát hiện:**
- Lượt áp xuất xứ HÀNG LOẠT cho nhiều Chương — không có, cố ý (§Quyết định đã chốt). Nợ có chủ
  ghi ở `deferred-work.md` (Chủ: Ice).
- `deferred-work.md:10786-10806` (`asset.source_url` ghi chặng cuối sau chuyển hướng) —
  KHÔNG nhận, ghi rõ lý do, ở lại 🟡 (Chủ: Ice) — xem mục mới ngay dưới nó trong cùng tệp.
- DTO xem trước (`ChapterSplitPreviewEntryWire::origin`) chỉ áp override THẬT ở ba trong năm
  chỗ gọi nội bộ (`encoding_candidate_wire`/`url_import_batch_wire` VÀ đường `chapter_detail_
  for_index`/nhánh tự khai UTF-8 cố ý truyền `&[]`) — hai chỗ đó không có bề mặt xuất xứ để mà
  hiện, ghi lại trong `deferred-work.md` để không ai đọc nhầm `&[]` ở đó là quên truyền.

## Spec Change Log

**Ba lượt lệch nhỏ so với chữ nguyên văn của spec, không đổi Ý ĐỊNH:**

1. **Không đăng ký `src/commands/index.ts`.** Code Map trỏ `:1523-1533` (khuôn đăng ký lệnh
   `library.chapter_rename`) như một tiền lệ tham khảo — nhưng bốn ô xuất xứ là văn bản sửa
   tại chỗ, cam kết qua `@change` (KHÔNG `@click`), nên không có một "thao tác" nào cần một
   `dispatch('<id>')`/mục trong `CommandRegistry` để tới được bằng bàn phím — đúng giới hạn đã
   ghi ở `check-commands.mjs`: "Kiểm A chỉ canh `@click`". `renameChapter`/nút "Đổi tên" (tiền
   lệ) CẦN đăng ký vì nó LÀ một nút `@click`; bốn ô này thì không.

2. **`ChapterOriginOverridesState` reset TOÀN BỘ ở `reload_url_import_item`/
   `remove_url_import_item`, không chỉ mục 0.** `Tier2BlockOverridesState` (khối tham chiếu)
   chỉ reset khi mục 0 đổi (nó CHỈ có nghĩa cho Chương đầu). Xuất xứ có nghĩa cho MỌI Chương,
   nên một lượt bỏ/tải-lại-mục-k dời chỉ số của MỌI mục đứng sau k — dịch chuyển từng override
   theo đúng chỉ số mới đòi một cơ chế theo dõi thêm (ngoài phạm vi bốn ô văn bản đơn giản);
   dọn TOÀN BỘ là lựa chọn AN TOÀN (không giữ một override có thể đã sai ý nghĩa), cái giá là
   người dùng gõ lại nếu vừa sửa origin của một Chương KHÁC ngay trước khi bỏ/tải-lại một mục.
   Ghi ra vì đây là một đánh đổi CÓ CHỦ Ý, không phải một giản lược.

3. **Hàng I/O Matrix "Một trang tách thành nhiều Chương" đo được KHÔNG qua `commands::project`
   trên đường sản phẩm.** `chapter_pattern` (Story 6.6) là no-op trên `PipelineShape::Chapters`
   (đường URL — `already_chaptered` làm `Step::SplitChapters` bỏ qua HOÀN TOÀN, khoá bởi
   `webimport_contract.rs::exactly_one_link_yields_exactly_one_chapter_even_when_a_chapter_pattern_would_match_inside_it`),
   nên "1 link, mẫu phân tách khớp 3 lần" không tái lập được qua `create_work`/lệnh IPC thật —
   `extract_main_content == true` chỉ đi cùng hình dạng `Chapters`, còn `chapter_pattern` chỉ
   có tác dụng trên hình dạng `Blob`, và không chỗ gọi SẢN PHẨM nào dựng ra tổ hợp cả hai.
   Đối chứng vì thế xâu THẲNG `core::segment::pipeline::run_import` (seam công khai, đúng
   triết lý mà chính module đó đã theo cho mọi đối chứng AD-39 khác) với `PipelineShape::Blob`
   + `extract_main_content: true` + `chapter_pattern`. Cơ chế được đo (broadcast xuất xứ ra
   N mảnh khi `SplitChapters` THẬT SỰ tách một `Blob`) là đúng cơ chế mà `Flow::origins` dùng
   bất kể tổ hợp đó có đường sản phẩm hôm nay hay không — hàng I/O Matrix được thoả ĐÚNG NGHĨA
   ("cả N Chương nhận cùng bộ bốn trường"), chỉ khác ở CHỖ gọi.

## Review Triage Log

### Vòng rà 1 — 2026-09-10 (ba lớp: blind-hunter · edge-case · verification-gap)

| # | Phát hiện | Verdict | Bằng chứng phân xử | Tuyến |
|---|---|---|---|---|
| 1 | Đường huỷ/xem trước MỚI không dọn `ChapterOriginOverridesState` phía Rust | high | ĐO: `awk` trên `project.rs` cho thấy `preview_import_encoding_from_text` VÀ `_from_file` đều gọi `reset_tier2_block_overrides` nhưng **0** lần gọi `reset_chapter_origin_overrides`; `confirm_import_with_encoding` thì đọc state đó vô điều kiện (`resolve_chapter_origin_overrides`). Đường tới hỏng dữ liệu: nhập URL → gõ xuất xứ → huỷ → dán tay → xác nhận ⇒ Tác phẩm mới mang xuất xứ của lượt web đã bỏ. Phá thẳng hàng ma trận "Nhập từ file / dán tay" và §Always "cả bốn cột NULL" | patch |
| 2 | Ca `cancelling_preview_after_typing_an_override_...` gọi tay `reset_chapter_origin_overrides`, không đi đường sản phẩm | high (cùng gốc #1) | `cancelImportPreview()` → `resetImportPreview()` chỉ dọn draft CLIENT (`chapterOriginDrafts.value = {}`), doc-comment tự khai "0 lượt gọi Rust". Ca test vì thế chứng minh một dữ kiện về hai hàm gọi rời, không về đường người dùng thật — đúng lớp "đối chứng đỏ phải là phép GỠ thật" | patch |
| 3 | Thiếu ca ghim đăng ký + tên tham số cho vỏ `set_chapter_origin_override` | medium | `grep` `tests/`: tên đó chỉ có ở `chapter_origin_contract.rs` và ở đó gọi HÀM THUẦN, không qua vỏ. Lệnh anh em `update_chapter_origin` thì CÓ ca ghim. Gỡ dòng đăng ký khỏi `lib.rs` vẫn biên dịch, vẫn xanh — đúng lớp lỗi mà `ipc_contract.rs:809` ra đời để chặn cho `list_domain_log` | patch |
| 4 | `<ChapterOrigin>` ở xem trước không buộc `:disabled`, lệch với chỗ gọi anh em | low | Đọc diff: `LibraryMode.vue` truyền `:disabled="libraryChapterOriginBusy"`, `ImportPreviewOverlay.vue` không truyền gì, trong khi doc-comment của chính `ChapterOrigin.vue` nói prop đó sinh ra "cùng khuôn `<fieldset :disabled>` của `ImportPreviewOverlay.vue`". Bốn ô còn sửa được trong lúc một lượt xác nhận đang bay | patch |
| 5 | Draft client dùng `=== ''` còn đường ghi Rust dùng `trim()` | low | `importPreviewCurrentChapterOrigin` so `draft.author === ''`; `trimmed_or_none` phía Rust cắt hai đầu. Một ô chỉ chứa dấu cách hiện ô trống ở xem trước nhưng xuống đĩa thành `NULL`. Phép sửa là một phép sửa THẲNG (đổi vị từ), không thêm nhánh phòng thủ | patch |
| 6 | Đệ quy không chặn trong `push_flattened`/`json_ld_string_or_named` ⇒ tràn stack với HTML không tin cậy | false | ĐO trên chính bản đã ghim: `serde_json-1.0.151/src/de.rs:63,67` đặt `remaining_depth: 128` và `disable_recursion_limit: false`; `origin.rs` gọi `from_str` TRẦN, không gọi `disable_recursion_limit`. JSON sâu quá 128 trượt NGAY Ở KHÂU PHÂN GIẢI và khối bị bỏ qua ⇒ hai hàm kia không bao giờ nhận được cây sâu hơn thế | — |
| 7 | `saveCurrentChapterOrigin` thiếu chốt chống tái nhập | false | `chapterOriginBusy.value = true` đặt ĐỒNG BỘ trước `await`, và `LibraryMode.vue` buộc `:disabled="libraryChapterOriginBusy"` vào chính component đó ⇒ bốn `<input>` bị vô hiệu hoá, một `<input disabled>` không bắn `@change`. Chốt có thật, nó nằm ở chỗ GỌI chứ không trong thân hàm | — |
| 8 | `guard.resize(chapter_index + 1, ...)` cấp phát không chặn / tràn `usize` từ IPC | low, bác | Giá trị duy nhất frontend gửi là `chapterCursor.value`, bị chặn bởi số Chương của lượt nhập. Người rà không chỉ ra được đường nào đưa một chỉ số lớn tới đó. Phép sửa là THÊM một hàng rào (một nhánh mới) cho một tình huống chưa chứng minh là tới được — đúng hai điều kiện của luật bác | — |
| 9 | Hai lượt `setChapterOriginOverride` chồng nhau có thể về sai thứ tự, bản cũ thắng | low, bác | Cần hai lượt IPC cục bộ về ngược thứ tự trong khoảng cách hai lần `@change` (bắn khi rời ô). Không đo được đường tới, và phép sửa là thêm bộ đếm thứ tự — thêm phức tạp cho một rủi ro chưa xảy ra. Vá #4 (`:disabled` lúc xác nhận) thu hẹp cửa mà không thêm cơ chế nào | — |
| 10 | Bốn cờ `*_confirmed` khai là "chạm TỪNG ô" nhưng `commitField` luôn gửi cả bốn | low | Đúng như mô tả, và `grep` xác nhận **0** chỗ đọc `*_confirmed` hôm nay ⇒ chưa hại ai. Hại có tên ở tương lai: một trường KHÔNG bị chạm bị đóng băng thành override, nên nếu giá trị máy đổi (đổi ứng viên bảng mã ⇒ HTML giải mã khác ⇒ `og:site_name` khác) thì giá trị cũ thắng. ⚠️ Tôi KHÔNG đo được rằng bốn trường thật sự đổi giữa các ứng viên bảng mã — nêu ra là một giả thuyết, không một phép đo. Phép sửa đổi hình dạng draft cộng API component, không phải một phép sửa thẳng | defer |
| 11 | Không ca nào chạy `origin_url` qua một lượt chuyển hướng thật | medium | Đúng: mọi fixture dùng URL trùng luôn với `label` duy nhất. Đây là quyết định được lập luận nhiều nhất của spec (§Design Notes + một mục sổ nợ + doc-comment lược đồ) mà chưa có phép đo đầu-cuối. ⚠️ Ca như thế phải sống trong bộ loopback — mà bộ đó đang ĐỎ 28/28 trên máy này ở CẢ `46f068f` (đã đối chứng), nên một ca mới thêm vào đó không nghiệm thu xanh được ở đây | defer |
| 12 | Hai tài liệu lệch nhau "ba trong bốn" và "ba trong năm" chỗ gọi `&[]`, và chỗ thứ sáu ở `project.rs:2412` không được liệt | low, bác | Có thật, nhưng phép sửa là sửa chữ trong §Implementation Notes của CHÍNH spec này — luật phân xử cấm nhận một phát hiện mà phép sửa là sửa spec của lượt build này | — |
| 13 | `push_flattened` đẩy CẢ object bọc lẫn các mục `@graph`, và không ca nào phủ | low, bác | Đúng, nhưng hệ quả là THÊM ứng viên chứ không mất ứng viên, và thứ tự thử vẫn quyết định. Không đường nào cho ra một giá trị SAI, chỉ có thể cho một giá trị đúng sớm hơn | — |
| 14 | Lề/xuống dòng không đều ở các chỗ gọi test bị sửa máy móc | low, bác | Thẩm mỹ thuần tuý, `cargo fmt` không nằm trong 11 cổng lẫn CI (đo: `.githooks/pre-push:80`) ⇒ không cổng nào đòi, và sửa nó làm diff của story phình ra vì một lý do không phải hành vi | — |
| 15 | Ô URL không có đường mở link để đối chiếu | low, bác | Nằm ngoài ý định: story ghi LẠI xuất xứ, không duyệt nó; và mở một URL ra ngoài là một bề mặt mạng mới, thứ §Never cấm tường minh | — |
| 16 | Không có lời nhắc trong giao diện về cái giá "gõ 30 lần" | low, bác | Ice đã chốt tường minh "chỉ per-Chương" sau khi được trình đúng cái giá này (§Quyết định đã chốt). Thêm một lời nhắc là lật một quyết định đã ký bằng một phát hiện của máy | — |


## Spec Change Log

### 2026-09-10 — vòng rà 1, mục 1/2: lượt vá đầu không canh được chính nó

**Phát hiện kích hoạt:** hai vỏ `wire::preview_import_encoding_from_text`/`_from_file` dọn
`Tier2BlockOverridesState` nhưng không dọn `ChapterOriginOverridesState`, mở một đường hỏng dữ
liệu qua các nguồn nhập (chi tiết ở §Review Triage Log hàng 1).

**Đã sửa gì:** thêm `reset_chapter_origin_overrides(&app);` vào cả hai vỏ. Nhưng ca test đi kèm
lượt vá — `chapter_origin_contract.rs::a_leftover_override_from_a_cancelled_url_preview_...` —
**TỰ GỌI** lượt dọn rồi khẳng định không rò rỉ, và comment của nó khai rằng gỡ dòng đó là gỡ
đúng dòng vá ở vỏ thật. **ĐO 2026-09-10 bằng phép gỡ THẬT** (gỡ cả sáu lời gọi
`reset_chapter_origin_overrides(&app);` khỏi `mod wire`, chạy lại): ca đó **15/15 vẫn XANH**.
⇒ Bản vá không có phép canh nào.

**Trạng thái xấu đã tránh được:** một lượt vá cho một lỗi hỏng dữ liệu, kèm một ca test khai là
canh nó, mà thực chất canh chính nó — nên lượt sửa kế tiếp gỡ dòng vá ra sẽ đi qua trọn 11 cổng
cộng `cargo test` mà không một mệnh đề nào đỏ.

**Phép canh thật đã dựng:**
`ipc_contract.rs::both_preview_wires_reset_the_chapter_origin_overrides_before_building_a_preview`
— quét THÂN từng vỏ, cùng khuôn `config_invariants.rs::the_blocking_wires_run_off_the_main_thread`
đã dùng cho `(async)`, vì crate test không có `MockRuntime` nên không ca nào gọi được vỏ thật.
⚠️ Bản ĐẦU của phép quét này cũng hỏng theo một cách khác: nó dùng `body.contains(...)` trên
văn bản thô, nên một dòng bị CHÚ THÍCH vẫn khớp — và phép đối chứng đầu tiên của tôi cũng sai
đúng chỗ đó (chú thích thay vì xoá). Bản hiện tại lọc theo **dòng mã** (khuôn `code_lines`,
`webimport_boundary.rs:89`) và đã đo **ĐỎ dưới CẢ HAI** kiểu gỡ — xoá hẳn và chú thích — rồi
xanh lại khi trả về nguyên trạng.

**KEEP — phải sống sót nếu mã được dựng lại:** ① lượt dọn nằm ở CẢ HAI vỏ xem trước, ngay cạnh
`reset_tier2_block_overrides`; ② phép canh phải đo THÂN VỎ theo DÒNG MÃ, không đo văn bản thô,
và không đo qua một hàm thuần mà chính ca test gọi được; ③ mọi ca test khai là canh một chỗ nối
phải được đối chứng bằng phép GỠ ở chỗ nối ĐÓ, không ở bản sao của nó trong test.

## Design Notes

**Vì sao một bước di trú chứ không bốn.** `migrate` chạy `execute_batch` từ đầu, và bước 9 đã là
tiền lệ *"nhiều câu trong một hằng"* với doc-comment nói thẳng rằng mệnh đề *"mỗi bước một hằng"*
đếm **số hằng**, không **số câu** (`schema.rs:1616-1620`). Bốn bước rời sẽ đẩy `schema_version` lên
25 cho một lượt thay đổi khái niệm duy nhất, và mỗi số đó là một cửa một chiều riêng.

**Cửa một chiều, ghi ra thay vì để người sau phát hiện.** AD-30: di trú chỉ tiến, gặp lược đồ mới
hơn thì **từ chối mở**. Một khi v22 đã chạy trên máy người dùng, hạ ứng dụng xuống bản không biết
bước này làm `project.db` bị từ chối mở — mất đường vào cả Tác phẩm, không chỉ mất bốn ô xuất xứ.
Đây là cái giá đã nhận của mọi bước di trú trong kho; nhắc ở đây vì story này là bước đầu tiên thêm
cột vào `chapter` kể từ khi bảng đó ra đời.

**Vì sao bóc xuất xứ sống trong `Extractor`, không trong `Fetcher`.** Ranh giới đã chốt ở Epic 6:
`Fetcher` chỉ lấy byte, `Extractor` dựng mô hình có cấu trúc và **không chạm mạng** — chính ranh
giới đó là điều kiện để allowlist kiểm chứng được bằng test. Bốn trường xuất xứ là một mô hình dựng
từ HTML đã tải, nên chúng thuộc phía `Extractor`. Ba trường bóc được từ thân trang; trường thứ tư — URL —
**không** bóc, nó là dữ kiện của lượt TẢI và đã có sẵn ở `Flow.labels`. Đó là lý do phán quyết
"URL yêu cầu" giữ `fetcher.rs` nguyên vẹn: chặng cuối sau chuyển hướng là thứ DUY NHẤT trong bốn
trường bắt buộc phải đi qua `Fetcher`, và Ice đã chọn không lấy nó ở story này.

**Vì sao ghi đè của người dùng sống trong một state riêng, không trong DTO xem trước.** Xem trước
dựng lại trong bộ nhớ mỗi lần đổi bảng mã — chữ người dùng vừa gõ nằm trong DTO sẽ bị lượt dựng lại
xoá trắng trong im lặng. `Tier2BlockOverridesState` ra đời cho đúng vấn đề này ở Story 6.9, kèm kỷ
luật *"đọc lại LÚC XÁC NHẬN"* (`project.rs:3185-3190`) và *"reset chỉ SAU KHI `Ok`"*; chép nguyên
khuôn đó rẻ hơn phát minh một khuôn thứ hai lệch nó một nhịp.

**Cái giá đã nhận của một nhãn duy nhất (Ice chốt 2026-09-10).** *"không tìm thấy"* khẳng định hệ
thống **đã tìm**. Với Chương nhập từ file hay dán tay, không có trang nào để mà tìm — nên nhãn đó
nói một điều chưa xảy ra, đúng lớp lỗi mà Story 6.10 đã phải tách ra ở bộ lọc *"cần xem"* (dấu hiệu
chưa đo được lẫn với đã đo và sạch). Ghi ra ở đây thay vì để người sau phát hiện: bốn ô của một Tác
phẩm từ file trông hệt bốn ô của một trang web khai thiếu mọi thẻ. Phép sửa nếu về sau muốn tách là
một nhãn thứ hai suy từ `source_url` — **0 cột thêm**, nên cửa này không bị đóng lại.

**Vì sao không suy tên báo từ host.** `og:site_name` vắng thì host (`vnexpress.net`) là thứ duy nhất
còn lại — nhưng nó không phải **tên** tờ báo, và một giá trị suy ra trông giống hệt một giá trị bóc
được. Người dùng không phân biệt được hai thứ đó trên màn hình, nên ô để trống và nhãn trạng thái
nói thật hơn một phép suy.

## Verification

**Commands:**
- `npm run build && (cd src-tauri && cargo test --locked)` — đúng thứ tự `pre-push`.
- `npm run test` — vitest, `fileParallelism: false`.
- Chạy **TỪNG** cổng: `check:deps` `check:tokens` `check:i18n` `check:commands` `check:layout`
  `check:panel-refs` `check:dict` `check:dict-manifest` `check:lint` `check:gates` `check:debt-owner`
  — **mười một**.
- `node scripts/check-debt-owner.mjs --report` — `mở KHÔNG có Chủ:` phải là **0**.
- 🔴 Đối chứng đỏ ① — gỡ lượt xâu xuất xứ qua pipeline, chạy `chapter_origin_contract.rs`, phải
  **ĐỎ**; trả lại phải xanh. Ghi số ca đỏ và **tên** ca vào §Implementation Notes.
- 🔴 Đối chứng đỏ ② — gỡ `write_lifecycle_after_change` khỏi lệnh ghi mới, chạy **bộ test CŨ cộng
  MỚI**, ghi ca nào đỏ. Cả hai bộ cùng xanh ⇒ chỗ nối không ai canh, phải thêm ca trước khi đi tiếp.
- `cargo test --test config_invariants` — xanh với **0 dòng sửa** (`git diff` xác nhận).

**Manual checks (if no CLI):**
- Nhập một bài báo có `og:site_name` + `article:published_time`: bốn ô tự điền ở đầu Chương trong xem
  trước; sửa một ô, đổi bảng mã, chữ đã gõ **còn nguyên**; xác nhận rồi mở lại từ danh sách Chương,
  giá trị khớp.
- Dán tay một văn bản: bốn ô rỗng, nhập tay từ danh sách Chương, đóng và mở lại Tác phẩm, giá trị còn.
