---
title: 'Story 6.7: Nhập từ URL bằng danh sách link'
type: 'feature'
created: '2026-09-06'
status: 'done'
baseline_commit: 'd990e1c4f11d86facbc00468e47b4ed1b0ef9ece'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Hai lối vào đường nhập hôm nay (dán văn bản, đường dẫn tệp) đều dựng `PipelineShape::Blob`; không lối nào tải được một danh sách URL. `core/webimport/` — nhà của điểm ra mạng thứ ba (AD-15) — còn **0 dòng mã** và chưa cả được khai vào cây `mod`, nên `cargo test` chạm nó ở **0 ca chạy được**. FR122 hứa người dùng **nhìn thấy** ứng dụng chỉ tải đúng những link mình dán, chứ không phải đọc lời hứa đó trong tài liệu.

**Approach:** Dựng **cả hai nửa** của AD-40 trong `core/webimport/`: `Fetcher` (URL → byte + `content-type`, cài đặt duy nhất mãi mãi của điểm ra mạng thứ ba) và `Extractor` (byte HTML → **văn bản thuần**, không bao giờ chạm mạng). N link ⇒ `PipelineShape::Chapters(N)` — hình dạng mà `Step::SplitChapters` đã biết bỏ qua từ Story 6.2. Hai con số *N link · sẽ tạo N Chương* là **computed cục bộ** trên bề mặt dán, nên AC *"chưa bấm ⇒ 0 lời gọi mạng"* đúng theo **cấu tạo**: chưa có gì để gọi.

## Boundaries & Constraints

**Always:**
- `Fetcher` **không bao giờ** phân tích nội dung; `Extractor` **không bao giờ** chạm mạng (AD-40). Mọi lời gọi mạng của đường này sống trong `core/webimport/`, không đi qua plugin JS nào.
- 🔴 **Thứ rời `core/webimport/` là văn bản thuần — 0 chuỗi đánh dấu** (AD-16 §Rule mục 1 và mục 2). Byte HTML thô không đi qua IPC, không xuống `project.db`.
- `reqwest::blocking` gọi từ một `#[tauri::command(async)]` — khuôn đã có **17** tiền lệ. **0** crate mới, **0** plugin mới, **0** `async fn`.
- Một link hỏng thành **một mục** trong xem trước mang đúng lý do, **giữ nguyên VỊ TRÍ** (Ice chốt 2026-09-06). Hai con số vì thế vẫn bằng nhau.
- Mọi thứ xảy ra **trước bước ghi** (AD-39). Danh sách còn một mục hỏng thì nút xác nhận **khoá** — người dùng bỏ mục đó (hai số cùng giảm) hoặc tải lại riêng nó.
- `PIPELINE_ORDER` bảy bước, `validate_order`, và chỗ gọi `run_import` **duy nhất** (`project.rs:237`) không đổi. Bóc nội dung nằm **đúng ở bước 2**, không chạy trước khi vào pipeline — giải mã bảng mã (bước 1) phải đi trước.
- Một bảng mã cho **cả** danh sách — giữ hình dạng hôm nay, và **viết quyết định đó thành chữ** trong doc-comment `PipelineInput::encoding` (Ice chốt 2026-09-06; đường ② của nợ `:9262`).

**Ask First:**
- 🔴 Phép đo Task 0 cho thấy `reqwest::blocking` **không** sống được trong `sync_threadpool` của Tauri ⇒ **DỪNG và báo Ice**. Không tự chuyển sang `async fn`.
- 🔴 Phép đo Task 1 cho thấy `TextMode::Formatted` **không** giữ được ranh giới đoạn ⇒ **DỪNG**. Bước 4 (luật gộp dòng của Story 6.4) và bước 5 đều tính trên cấu trúc dòng; một `text_content` dính liền làm hai bước đó sai mà không cổng nào đỏ.
- Bất kỳ thay đổi nào lên `PipelineShape` hoặc `PIPELINE_ORDER`. (Thêm **một** trường vào `PipelineInput` kèm builder `#[must_use]` là khuôn Story 6.6 đã dùng cho `chapter_pattern` — không cần hỏi lại.)
- Mọi hằng số ngưỡng (trần số link, trần byte một phản hồi, trần thời gian) phải kèm một **phép đo**; không đo được thì hỏi trước khi chọn một con số.

**Never:**
- Không quét trang mục lục, không lần theo *"chương sau"*, không suy ra một URL nào không có trong ô dán. Một chuyển hướng sang **host khác** bị chặn tại chặng.
- Không dựng đường **sửa ranh giới bằng bàn phím**, ba trạng thái khối ở vạch lề, hay các phím `J`/`K`/`Space`/`[`/`]`/`R` — trọn phần đó ở lại **Story 6.9**. 6.7 dựng thuật toán; 6.9 dựng đường người dùng sửa nó.
- Không ảnh, không caption, không alt-text (Story 6.11/6.13) — `Extractor` của 6.7 chỉ trả văn bản.
- Không cột xuất xứ, không bước di trú. `schema_version` ở nguyên **19**.
- Không allowlist hai tầng, không nhật ký domain (Story 6.8) — 6.7 chỉ nợ AC *"chưa bấm ⇒ 0 lời gọi"*.
- Không *"thêm Chương vào Tác phẩm sẵn có"* (nợ đã ghi, **Chủ: Story 6.7b**).
- Không `@tauri-apps/plugin-http`. Không `TextMode::Markdown` — nó đưa chuỗi đánh dấu trở lại, đúng thứ AD-16 mục 2 cấm.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Dán N link, chưa bấm | N dòng non-empty trong ô | Hai số hiện *N link · sẽ tạo N Chương* + câu cam kết | **0** lời gọi mạng, **0** lời gọi IPC |
| Bấm tải, N link tốt | N URL trả HTML | N mục xem trước đúng thứ tự đã dán, mỗi mục là **văn bản đã bóc** | N/A |
| Đúng MỘT link | 1 dòng | 1 Chương; `already_chaptered = true` ⇒ bước 5 **vẫn** bỏ qua | N/A |
| Trang bóc ra rỗng | `Readability` trả nội dung rỗng/quá ngắn | Mục giữ vị trí, mang lý do *"không bóc được nội dung chính"* | 🔴 **Không** rơi về HTML thô — đó là rỗng im lặng đổi hình |
| `content-type` không phải HTML | PDF, ảnh, `text/plain` | Mục giữ vị trí, mang lý do *"không phải trang HTML"* | Không chạy `Extractor` trên byte không phải HTML |
| Link thứ k hỏng | 404 · timeout · không kết nối được | Mục thứ k giữ **vị trí k**, mang lý do; hai số vẫn bằng nhau; xác nhận **khoá** | `ImportError` biến thể mới + `MessageKey` mới |
| Tải lại riêng mục k | Người dùng bấm tải lại một mục hỏng | Đúng **1** lời gọi mạng, chỉ tới URL của mục k | Trượt lần nữa ⇒ mục k mang lý do mới |
| Bỏ mục hỏng thứ k | Người dùng xoá mục k | N−1 link · N−1 Chương — hai số cùng giảm | N/A |
| Chuyển hướng sang host khác | 301 tới host ngoài link đã dán | Chặn **tại chặng**; host đích nhận **0** kết nối | Mục hỏng, lý do *"chuyển hướng bị chặn"* |
| Phản hồi vượt trần byte | `Content-Length` lớn hoặc stream dài | Đọc theo dòng chảy, dừng ngay sau trần, `drop` | Mục hỏng, lý do *"vượt trần kích thước"* |
| Dòng không phải URL hợp lệ | Rác, khoảng trắng thừa | Dòng rỗng bỏ khi đếm; dòng rác thành mục hỏng | Lý do *"URL không hợp lệ"* — **0** lời gọi mạng cho dòng đó |
| Danh sách rỗng | Ô rỗng hoặc toàn dòng trắng | Nút tải `:disabled`; **0** lời gọi | Đóng vế còn mở của nợ `:9067` |

</frozen-after-approval>

## Code Map

**Rust — điểm tiêm**
- `src-tauri/src/core/mod.rs` — 🔴 `webimport` **chưa được khai** (`grep webimport src-tauri/src` chỉ khớp chính tệp stub). Một dòng `mod webimport;` là điều kiện để mọi thứ dưới đây tồn tại; thiếu nó thì trình liên kết loại trọn module — đúng cơ chế bàn đo 6.1 đo được **0** ký hiệu `dom_smoothie` trong nhị phân.
- `src-tauri/src/core/webimport/mod.rs:1-24` — stub thuần doc-comment, **0 dòng mã**. Tự khai *"`Extractor`/`Fetcher` thật là Story 6.9/6.7"* — 🔵 câu đó nay hết đúng, cả hai vào story này.
- `dom_smoothie 0.18.0` (`Cargo.toml:92`, ghim `=`): `Readability::new(html, Some(url), Some(config))` → `is_probably_readable()` → `parse()` → `Article`. `Article` (`readability.rs:22-52`) mang **`content: StrTendril` là HTML** và **`text_content: StrTendril` là văn bản thuần**, cộng `title`/`byline`/`length`/`excerpt`. 🔴 `text_content` sinh theo `config.text_mode` (`readability.rs:488-492`): `Raw` = `root_node.text()` (**mất ranh giới đoạn**) · `Formatted` = `root_node.formatted_text()` · `Markdown` (**cấm**). Mặc định là `Raw` — phải khai `Formatted` tường minh.
- `src-tauri/src/core/segment/pipeline.rs:408-411` `Step::ExtractMainContent` — 🔴 hôm nay là **no-op thuần** (`trace.push(step); flow`). Đây là **điểm tiêm duy nhất**. Khuôn thân thật để chép: `Step::CleanByRules` `:412-430` — destructure `Flow`, lặp `units`, **gọi xuống** `core::cleanup::apply` chứ không viết lại nội tuyến, `trace.push` **ở lại bên trong** nhánh.
- `pipeline.rs:180-189` `PipelineShape::Chapters(Vec<ChapterInput>)` — doc `:186-187` viết sẵn *"mỗi link… KỂ CẢ khi danh sách chỉ có ĐÚNG MỘT link"*. `:382-385` gán `already_chaptered`; `:648-651` bước 5 rẽ theo TRƯỜNG đó. `:193-226` `PipelineInput` (5 trường); `:207` `encoding` là **một** `&'static Encoding` — dòng phải nhận doc-comment mới. `:267-271`/`:276-280` hai builder `#[must_use]` — **khuôn để chép** cho cờ bóc nội dung. `:398-406` vòng `decode_unit(u, encoding)`; `:557-577` `decode_unit`. `:111-119` `PIPELINE_ORDER`; `:126-145` `validate_order` (hoán vị đủ 7).
- `src-tauri/src/core/segment/import.rs:89-190` `ImportError` — **9** biến thể; tiền lệ gần nhất cho lỗi mang chi tiết là `ReadFailed { path, detail }` `:118` và `InvalidChapterPattern { detail }` `:183`. `:346-370` `ImportedChapter` (4 trường). `:376-378`/`:395` hai hàm nhập, **cả hai** trả `Blob`.
- `src-tauri/src/core/i18n/mod.rs:62-91` `message_keys!` — cú pháp `Variant => "khoa.cham" [param]`; danh mục **ĐÓNG** qua `ALL` `:72`. **61** khoá, họ `err.import.*` có 8, **0** khoá về mạng/URL/tải.
- `src-tauri/src/commands/project.rs:978-984` `PendingImportSource { shape }` / `PendingImportSourceState` — 🔴 **chỗ hẹp nhất**: đúng một trường, không path, không URL. Danh sách URL và trạng thái từng mục sống ở đây trong bộ nhớ suốt lượt nhập rồi chết theo nó. `:1587` `preview_import_encoding`; `:1626-1638` nhánh `Chapters` **đã có sẵn**, comment `:1632` ghi thẳng *"danh sách URL là Story 6.7"*. `:1778-1826` `confirm_import_with_encoding` (giữ `MutexGuard` xuyên suốt `:1793-1810`; dọn ô chỉ khi `create_work` THÀNH CÔNG `:1826`). `:299` `create_work` (vòng N Chương `:412`, `insert_segments` `:429`). `:1363` `cleanup_and_chapters_preview_for`; `:1144` `ChapterSplitPreviewEntryWire` (`ord`/`title`/`length`). `:776` `spawn_import_scan` nhận đúng MỘT `chapter_id`.
- **Khuôn chạy nền** `project.rs:776-925`: `std::thread::Builder::new().name(...).spawn(...)` trả `io::Result`; `ImportScanGeneration` (`:550-566`, `Arc<AtomicU64>`) kiểm ở **4** điểm chốt; `app.emit`; seam `keep_committed_import_when_scan_spawn_fails` `:630`. 🔴 Luật `:773-775`: **không `unwrap()`/`expect()`** trên đường nền — `panic = "abort"`.
- `src-tauri/Cargo.toml:54-62` `reqwest = { version = "=0.13.4", features = ["blocking"] }`; chú thích `:60-61` để MỞ quyền quyết định cho story này. `src-tauri/src/lib.rs:634-802` `generate_handler!` (**65** lệnh phát hành); `.manage(...)` **8** kiểu, bốn ô của lượt nhập ở `:1030-1042`.

**Rust — cổng: cái nào canh, cái nào mù**
- `src-tauri/tests/webimport_probe.rs` — bàn đo 6.1, **635 dòng**, 3 ca, cả ba `#[ignore]` ở **CẤP HÀM** (`:104`, `:259`, `:358`) nên gỡ từng ca được. `spawn_once` `:618-634` là server HTTP thô `TcpListener::bind("127.0.0.1:0")`, không framework — **khuôn thẳng nhất để chép**. `redirect_case` `:385`, `size_cap_case` `:460` (⚠️ bất biến là `CAP ≤ đọc ≪ quảng cáo`, **không** một con số), `network_failure_case` `:554` (bind rồi thả cổng; thừa nhận TOCTOU `:566-568`, retry 5 lần). `:152` khuôn gọi `Readability::new`.
- `src-tauri/tests/cleanup_boundary.rs` — **khuôn cổng ranh giới tốt nhất**: `text_before_first_cfg_test_line` `:91-105` (neo theo ĐẦU DÒNG), `code_lines` `:78-89`, sàn quần thể `:27-30` + `:107-120`, hai ca tự-kiểm `:369-388`, kiểm chứng dương **ca dương + ca âm** `:171-183` và `:280-331`, `walk` `:40-59` dùng `symlink_metadata`.
- `src-tauri/tests/ipc_contract.rs:805-891` — 🔴 **khuôn khoá tham số ĐÚNG** (bản Story 6.3): neo vào `pub mod wire {` `:853-857`, bóc bằng `fn_param_list` `:894+`, `assert_eq!` **toàn bộ** danh sách. Bản 5.8 `:752-803` yếu hơn (chỉ hỏi chuỗi có xuất hiện đâu đó trong tệp 2.400 dòng) — **đừng chép bản đó**. `:836-839` khuôn assert `app.manage(...)` kèm phép đo *"xoá dòng này ⇒ 0 ca đỏ"*. `:226-257` mọi `MessageKey` phải có trong `vi.json`; `:310-372` params khai đủ, kiểm **hai chiều**.
- ⚠️ **`generate_handler!` không có ca nào đếm tổng** — 13 kết quả `grep`, tất cả là `contains(<một tên cụ thể>)`.
- ⚠️ **`@tauri-apps/plugin-http` không bị chặn ở đâu cả** — `grep plugin-http` toàn kho: **0**. `BANNED_CRATES` (`check-deps.mjs:172-176`) 4 mục, `BANNED_NPM` (`:222-227`) 4 mục.
- `scripts/check-deps.mjs:296-302` — dòng cuối in *"ba điểm ra mạng của AD-15 mở ở Story 4.x, 6.7, 10.7"* và khai *"vẫn đúng cho ĐƯỜNG SẢN PHẨM"*. 🔵 Story này làm mệnh đề đó hết đúng.
- `.github/workflows/ci.yml:628` — hàng *"Epic 6 · bốn test allowlist mạng (AD-41)"* **chưa có ✅**; chỗ cắm của **6.8**, không phải story này.
- `src-tauri/tests/glossary_han_viet_suggestion_contract.rs:479-497` — ca "không mạng" **duy nhất** của kho: quét TĨNH 7 token trên **một** tệp. ⚠️ Áp được cho `Extractor`, **không** áp được cho `Fetcher`.

**Frontend — dây và bề mặt**
- `src/modes/libraryImport.ts:87` `pastedText` / `:90` `filePath` / `:284` `submitPastedText` / `:314` `submitFilePath` — 🔴 cả hai chốt `if (busy.value || importPreviewIsOpen.value) return` **trong hàm**, không chỉ ở `:disabled`; `beginSubmit()` `:134` phải chạy trước. `:196` `finishImportSubmission` đọc `lastSubmittedFrom`.
- `src/modes/LibraryMode.vue:1216-1252` — hai khối form; nút đi qua `@click="dispatch('library.import_text')"` **trần**. `src/commands/index.ts:1029-1045` — hai lệnh **cố ý không gán phím** (`:1024-1027` giải thích). ⇒ `library.import_urls` chép đúng khuôn này; **không** hợp âm mới (`keys.ts:483-490` `claimed` ném lúc khởi động khi trùng, giết cả app).
- `src/importPreviewState.ts:102` `lastSubmittedFrom: Ref<'text'|'file'|null>` — 🔴 thêm biến thể thứ ba làm TypeScript đỏ ở **đúng bốn chỗ**: `:102`, tham số `from` của `openWith` `:318`, hai nhánh `if/else` `:465-467`, và `libraryImport.ts:275`/`:277`. `:314-376` `openWith` (bump `sequence` `:326-327` TRƯỚC `await`, so `mySequence !== sequence` sau); `:378-412` hai hàm mở (khuôn để chép); `:449-469` `runImportPreviewReload`; `:471-518` `reloadImportPreviewAfterRuleChange`. `:113-114` `pendingText`/`pendingPath` — cần ô thứ ba. `:606-705` khuôn CRUD (cờ riêng, `finally` hạ cờ, chỉ tải lại khi `error === null`). 🔴 `:814-842` `resetImportPreview()` quét **27** ô — mọi ô mới phải vào đây (`check:panel-refs` Kiểm A canh).
- `src/ImportPreviewOverlay.vue` (1414 dòng) — tầng 1 `:456`, tầng chuẩn hoá `:510`, **tầng 2 rỗng có chủ `:548-554`** (🔵 nay có thân: nội dung đã bóc; ba trạng thái khối ở vạch lề **vẫn** của 6.9), tầng 3 `:557`, tầng 4 `:716`. `focusableWithin` `:375-382` đã bắt `textarea` sẵn. `:72-80` khoá i18n phải là literal qua `switch` **cạn**. `:585-596` vì sao thao tác mang tham số đi qua `@change`/`@submit`, không `@click` — cả **4** `@click` trong tệp đều là `dispatch(...)` trần.
- `src/config/project.ts:222-250` `EncodingCandidateWire` (6 trường) / `ImportEncodingPreview` (6 trường); `:258-260` ba hằng tên lệnh nhập (7 hằng cả tệp); `:306-318` khuôn kiểm kiểu lúc chạy (mảng `.every`, nullable viết `(v.x === null || is…(v.x))` — thiếu vế này thì `undefined` lọt lên `.vue` và vỡ trắng màn hình); `:382-404` `callPreviewImportEncoding` **ba** nhánh `catch` + nhánh thứ tư ngoài `catch`; `:453-460` luật *"thêm adapter KÈM chỗ gọi trong CÙNG một lượt"*.
- `src/i18n/vi.json:212-261` — **50** khoá `mode.library.preview.*`, placeholder `{ten_tham_so}` snake_case. **0** khoá về url/link/mạng.
- `tests/frontend/` — **11** tệp / **94** ca cho màn nhập. Ba ca "0 IPC" đo bằng **HIỆU SỐ** ba mock: `importPreviewNormalized.test.ts:103`, `importPreviewCleanup.test.ts:122`, `importPreviewChapters.test.ts:111`. ⚠️ `freshState()` (`importPreviewCleanup.test.ts:43-59`) phải `vi.resetModules()` TRƯỚC, `import()` động SAU. ⚠️ Mock ở `importPreviewCleanup.test.ts` **cắt bớt tham số thứ ba** — thêm tham số thì sửa mock ở **cả hai** tệp.

**Nợ liên quan** — `deferred-work.md:9262-9296` (🔴 **chủ story này**, đóng bằng đường ②) · `:9188-9193` (🔴 **chủ story này** — `SelfDeclared` từ `charset` HTTP; story này **không** dùng, cần chủ mới) · `:9067-9093` (vế *"danh sách URL rỗng"*) · `:9566-9599` (đường sản phẩm thật đưa `Chapters(N>1)` vào xem trước) · `:9049-9065` (chi phí byte NFR6 của `dom_smoothie` — 🔴 nay **đến hạn ở story này**, không phải 6.9).

## Tasks & Acceptance

**Execution:**
- [x] **Task 0 — ĐO TRƯỚC KHI VIẾT.** Một `#[tauri::command(async)]` tối thiểu gọi `reqwest::blocking` tới `spawn_once` cục bộ, chạy trong cửa sổ Tauri thật (khuôn `check:scope`). 🔴 Không sống được ⇒ **DỪNG và báo Ice**. Ghi kết quả + ngày vào chú thích `core/webimport/mod.rs`
- [x] **Task 1 — ĐO TRƯỚC KHI VIẾT.** `Readability` với `TextMode::Formatted` trên vài mẫu HTML đã cache ở `6-1-ban-do/fixtures/html/`: `text_content` có giữ ranh giới đoạn không. 🔴 Không giữ ⇒ **DỪNG** — bước 4 và bước 5 tính trên cấu trúc dòng. Ghi số + ngày
- [x] `src-tauri/src/core/mod.rs` -- khai `mod webimport;`
- [x] `src-tauri/src/core/webimport/mod.rs` -- **`Fetcher`**: URL → byte + `content-type`; `redirect::Policy::custom` chặn **tại chặng** mọi host khác host của chính URL đó; đọc qua `Read` (**không** `.bytes()`/`.text()`) dừng ngay sau trần; `is_connect()`/`is_timeout()` phân loại lỗi; 🔴 **0 dòng phân tích nội dung**. **`Extractor`**: byte HTML → văn bản thuần qua `text_content` với `TextMode::Formatted`; 🔴 **0 dòng chạm mạng**, và `Article::content` (HTML) **không được rời hàm**. Trần byte/thời gian chép từ bàn đo 6.1, ghi rõ nguồn số
- [x] `src-tauri/src/core/segment/pipeline.rs` -- thân thật cho `Step::ExtractMainContent` `:408-411` theo khuôn `Step::CleanByRules` (**gọi xuống** `webimport::extract`, `trace.push` ở lại trong nhánh); một trường cờ trên `PipelineInput` + builder `#[must_use]` theo khuôn `:276-280` để đường tệp/dán tay **không** bị bóc; 🔵 doc-comment MỚI trên `:207` -- **một** bảng mã cho **cả** danh sách, chốt từ đơn vị ĐẦU, kèm ngày và kèm ca hở (link A GBK / link B UTF-8), đường ② của nợ `:9262`
- [x] `src-tauri/src/core/segment/import.rs` -- biến thể `ImportError` cho mục hỏng, theo tiền lệ `ReadFailed { path, detail }` `:118`; lý do phân biệt được 404 · timeout · không kết nối · chuyển hướng bị chặn · vượt trần · không phải HTML · **bóc ra rỗng** · URL không hợp lệ
- [x] `src-tauri/src/core/i18n/` -- `MessageKey` mới trong `message_keys!` -- danh mục ĐÓNG, đừng viết danh sách song song; khai đủ `params` khớp `{ten_tham_so}` trong `vi.json` (cổng kiểm **hai chiều**)
- [x] `src-tauri/src/commands/project.rs` -- lệnh `#[tauri::command(async)]` mới: nhận `Vec<String>` URL, tải **tuần tự đúng thứ tự**, dựng `PipelineShape::Chapters(N)` rồi `stash_pending_import_source`; mục hỏng giữ **vị trí**. Lệnh tải lại **một** mục. Trạng thái từng mục sống cạnh `PendingImportSource` trong bộ nhớ. 🔴 Không `unwrap()`/`expect()`; giữ khuôn `MutexGuard` xuyên suốt của `:1793-1810`
- [x] `src-tauri/src/lib.rs` -- đăng ký lệnh mới trong `generate_handler!`; state mới (nếu cần) `.manage(...)` cạnh bốn ô ở `:1030-1042`
- [x] `src-tauri/tests/webimport_boundary.rs` -- cổng MỚI canh **cả hai nửa** AD-40: `Fetcher` mang **0** dòng gõ `dom_smoothie`/`Readability`; `Extractor` mang **0** dòng gõ `reqwest`/`TcpStream`/`http://`/`https://` (khuôn `glossary_han_viet_suggestion_contract.rs:479-497`); `reqwest` xuất hiện trong `src-tauri/src/**` **chỉ** ở `core/webimport/` và `core/ai/`; **0** dòng nào để `Article::content` rời module. 🔴 Chép `text_before_first_cfg_test_line` + **cả hai** ca tự-kiểm và **GỌI** nó trong MỌI assert thật; sàn quần thể; kiểm chứng dương **ca dương và ca âm** cho mỗi vị từ
- [x] `scripts/check-deps.mjs` -- thêm `@tauri-apps/plugin-http` vào `BANNED_NPM` và `tauri-plugin-http` vào `BANNED_CRATES` (đo 2026-09-06: **0** kết quả toàn kho, tức không cổng nào chặn); 🔵 sửa dòng cuối `:296-302` -- mệnh đề *"vẫn đúng cho ĐƯỜNG SẢN PHẨM"* hết đúng từ story này
- [x] `src-tauri/tests/webimport_contract.rs` -- chép `spawn_once` từ bàn đo; ca thật (**không** `#[ignore]`) cho: chặn chuyển hướng khác host (server bị chặn nhận **0** kết nối) · cắt theo dòng chảy (`CAP ≤ đọc ≪ quảng cáo`, không một con số) · lỗi kết nối phân loại đúng · bóc ra văn bản **không chứa** `<` của thẻ · trang bóc rỗng thành mục hỏng · N link giữ đúng thứ tự · mục hỏng giữ đúng **vị trí**
- [x] `src-tauri/tests/ipc_contract.rs` -- khoá tham số lệnh mới theo khuôn **`:805-891`**, cộng assert có mặt trong `generate_handler!`
- [x] `src-tauri/tests/project_contract.rs` -- ca N Chương đến **từ danh sách URL**: `ord` 1..N đúng thứ tự đã dán, mọi hàng `not_started`, `source_text` **không chứa chuỗi đánh dấu**, segment đủ mọi Chương; và ca **xác nhận bị TỪ CHỐI** khi còn một mục hỏng -- **0** hàng ghi xuống
- [x] `src-tauri/tests/segment_pipeline_boundary.rs` -- bước 2 nay có thân: cập nhật mệnh đề "gọi xuống, đừng chép lại" cho `webimport::extract` theo đúng khuôn đã áp cho bước 3 và bước 4
- [x] `src/config/project.ts` -- kiểu wire mục-theo-link (vị trí · URL · trạng thái · lý do) + adapter + hằng tên lệnh, KÈM chỗ gọi trong cùng lượt; một vế `typeof` cho **từng** trường mới, mảng `.every(...)`, nullable viết `(x === null || is…(x))`
- [x] `src/importPreviewState.ts` -- `openImportPreviewFromUrls` theo khuôn `:378-412`; biến thể thứ ba cho `lastSubmittedFrom` (sửa đủ **bốn** chỗ TypeScript đỏ); ô `pendingUrls`; hành động bỏ mục / tải lại một mục theo khuôn CRUD (cờ riêng, `finally`); 🔴 mọi ô mới vào `resetImportPreview()`
- [x] `src/modes/libraryImport.ts` + `src/modes/LibraryMode.vue` + `src/commands/index.ts` -- ô `pastedUrls`, `submitPastedUrls()` (chốt `busy || importPreviewIsOpen` **trong hàm**, `beginSubmit()` trước), lệnh `library.import_urls` **không gán phím**, nút `@click="dispatch('library.import_urls')"` trần. 🔴 Hai con số và câu cam kết là **computed CỤC BỘ** trên `pastedUrls` -- **0** IPC, **0** trường trên dây
- [x] `src/ImportPreviewOverlay.vue` -- tầng 2 nay có thân: văn bản đã bóc của mục đang chọn; danh sách mục-theo-link (vị trí · URL · lý do hỏng · nút bỏ / tải lại); nút xác nhận khoá khi còn mục hỏng, kèm câu nói **vì sao**. 🔴 **Không** vạch lề ba trạng thái, **không** `J`/`K`/`Space`/`[`/`]`/`R` (6.9). Khoá i18n qua `switch` **cạn**; thao tác mang tham số đi qua `@change`/`@submit`; AD-16 dữ liệu, **không** `v-html`
- [x] `src/i18n/vi.json` -- khoá mới cho ô dán, hai con số, câu cam kết, mọi lý do hỏng, tiêu đề tầng 2 có thân, và câu giải thích nút khoá; placeholder đúng dải `{ten_tham_so}`
- [x] `tests/frontend/importPreviewUrls.test.ts` (mới) -- 🔴 ca AC7 chép khuôn **HIỆU SỐ ba mock** (`importPreviewNormalized.test.ts:107-120`), **không** `not.toHaveBeenCalled()`: dán N link ⇒ hai số hiện đúng ⇒ hiệu số = 0. Cộng: thứ tự giữ nguyên; bỏ một mục ⇒ hai số cùng giảm; tải lại một mục ⇒ **đúng một** vòng IPC. Ba ca "0 IPC" hiện có **không sửa kỳ vọng**
- [x] **ĐO NFR6, đừng khai** -- 🔴 con số `−16 byte` của bàn đo 6.1 **hết đúng ở story này**: nó đo *"đã ghim, chưa gọi"*, và `dom_smoothie` nay có mã sản phẩm gọi tới. Dựng lại hai bản `--release` theo đúng khuôn 6.1 (`REPORT.md`, cùng `dist/`, `git worktree` cho baseline), ghi delta thật + ngày vào spine §Deferred và đóng nợ `:9049`
- [x] **ĐO hiệu năng** -- mốc trước story: 5 ứng viên × 2.000 Chương ~242-286 ms (`project.rs:1288-1300`, debug). Bóc nội dung nay chạy trong mỗi lượt `run_pipeline`. Đo lại trên danh sách N link thật (server cục bộ), ghi số + ngày. Suy tuyến tính bị CẤM (Ice 2026-09-05)
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- dòng `→` cho `:9262` (✅ đường ②), `:9049` (✅ đo lại NFR6), `:9067` (🟡 vế danh sách rỗng); nợ MỚI có chủ cho: ① `:9188` `SelfDeclared` từ `charset` HTTP -- `Fetcher` nay tồn tại nhưng story này không dùng charset khai báo, **cần chủ mới** · ② tỉ lệ bóc sai mới đo trên **một** site (`epochtimes.com`, 7 mẫu, bàn đo 6.1) -- chưa nói gì về trang đọc truyện chữ (**Chủ: Story 6.10**) · ③ `spawn_import_scan` vẫn chỉ quét Chương ĐẦU (**Chủ: Story 6.10**) · ④ ảnh/caption/alt-text bị `Extractor` bỏ (**Chủ: Story 6.11/6.13**). `check:debt-owner` đọc **dòng `→`**

**Acceptance Criteria:**
- Given người dùng dán N link và **chưa bấm nút**, when đo lưu lượng, then **0** lời gọi mạng và **0** lời gọi IPC — chứng minh bằng hiệu số ba mock, không bằng một phép quét token.
- Given một lượt nhập URL đã xác nhận, when đọc `source_text` của mọi Chương vừa ghi, then **không chuỗi đánh dấu HTML nào** — AD-16 §Rule mục 1 và mục 2 kiểm được, không chỉ được ghi.
- Given cổng mới `webimport_boundary.rs`, when **gỡ** nó ra và chạy lại bộ test **CŨ**, then bộ cũ **xanh** — chứng minh mệnh đề mới thật sự chưa ai canh.
- Given `@tauri-apps/plugin-http` được thêm vào `package.json`, when chạy `npm run check:deps`, then **đỏ** — hôm nay nó xanh (đo 2026-09-06), và đó là lỗ cổng story này bịt.
- Given bộ Rust (**1171** xanh) và vitest (**867** xanh) trước story, when chạy sau story, then vẫn xanh mà **không nới một kỳ vọng nào**; `schema_version` vẫn **19**; `PIPELINE_ORDER` và chỗ gọi `run_import` duy nhất không đổi.
- Given `npm run check:deps && check:i18n && check:tokens && check:commands && check:gates && check:panel-refs && check:debt-owner`, when chạy sau story, then **0** finding mỗi cổng.
- Given spine §Deferred sau story, when đọc hàng chi phí byte của `dom_smoothie`, then nó mang một **delta đo được** kèm ngày — không còn con số `−16` của trạng thái *"đã ghim, chưa gọi"*.
- Given `scripts/check-deps.mjs:296-302` sau story, when đọc, then mệnh đề *"vẫn đúng cho ĐƯỜNG SẢN PHẨM"* đã được sửa tại chỗ kèm 🔵 và ngày.

## Spec Change Log

## Design Notes

**Vì sao thuật toán bóc vào 6.7 dù `epics.md` giao nó cho 6.9.** AD-16 §Rule mục 1 (`ARCHITECTURE-SPINE.md:226`, **Binds: C1**) cấm *"byte HTML thô đi qua IPC"* và nói thẳng HTML từ internet là *"ca nặng nhất và được siết thêm một bậc"*. Màn xem trước bắt buộc của epic lại đòi người dùng **thấy** thứ sắp ghi. Ở trạng thái *"6.7 xong, 6.9 chưa"*, hai ràng buộc đã ký loại trừ nhau — không phải một đánh đổi phạm vi, mà là một cửa chặn. Ice chốt 2026-09-06: kéo **thuật toán** bóc vào 6.7 để thứ qua IPC là **mô hình đã bóc**, giữ AD-16 nguyên văn mà không phải soạn một `AD` mới hay đảo kế hoạch. ⚠️ Ghi rõ ranh giới còn lại: **đường sửa ranh giới bằng bàn phím và ba trạng thái khối ở vạch lề vẫn là Story 6.9** — 6.7 không được chạm, và 6.9 vì thế **thu hẹp** chứ không biến mất. Epic vẫn giữ nguyên câu *"một bản chỉ có thuật toán mà không có đường sửa tay thì chưa đạt nghiệm thu"*: nó nói về nghiệm thu của **epic**, và epic chưa đóng.

**Vì sao `text_content` chứ không `content`.** `dom_smoothie::Article` mang cả hai (`readability.rs:34`/`:44`): `content` là **HTML**, `text_content` là văn bản. AD-16 mục 2 đòi *"mô hình nội dung không có nhánh nào mang chuỗi đánh dấu"* — nên `content` không được rời module, và cổng ranh giới canh đúng mệnh đề đó. ⚠️ `text_content` sinh theo `config.text_mode`, **mặc định `Raw`** = `root_node.text()`, thứ nối liền mọi đoạn. Bước 4 (luật gộp dòng của Story 6.4) và bước 5 (mẫu phân tách) đều tính trên **cấu trúc dòng**; một `text_content` dính liền làm cả hai sai mà **không cổng nào đỏ** — đúng lớp lỗi Story 6.6 đã dính một lần (mẫu neo dòng chết trên nguồn không có dòng trống). ⇒ `TextMode::Formatted` khai tường minh, và Task 1 đo nó **trước** khi viết dòng nào khác. `TextMode::Markdown` bị cấm: nó đưa chuỗi đánh dấu trở lại.

**Vì sao hai con số phải là computed CỤC BỘ.** Đếm dòng non-empty của chuỗi vừa dán là JS thuần, 0 IPC. Một trường `link_count` trên `ImportEncodingPreview` chỉ tồn tại **sau** khi đã gọi Rust, tức đã ra mạng — nó biến AC *"chưa bấm ⇒ 0 lời gọi"* từ một mệnh đề **kiểm được bằng cấu tạo** thành một lời hứa. Mockup (`web-import.html:219`) gọi đúng chỗ này là *"màn hình tự tố cáo"*: ai đó thêm tính năng tự quét mục lục thì hai con số lệch nhau, và không ai phải nhớ đi kiểm.

**Vì sao xác nhận KHOÁ khi còn mục hỏng — và vì sao đó không phải một luật mới.** Ice chốt 2026-09-06: link hỏng **giữ chỗ, đánh dấu**, hai con số vẫn bằng nhau. Ba khả năng còn lại đều phá một bất biến đã ký: ghi một Chương rỗng cho mục hỏng là **rỗng im lặng** — lớp lỗi trung tâm của dự án; bỏ qua mục hỏng lúc ghi cho `N link · N−1 Chương`, đúng thứ AC4 dựng một test để bắt; ghi một phần rồi dừng phá AD-39 *"mọi thứ trước bước ghi"*. ⇒ Khoá nút là hệ quả **duy nhất** còn lại, và nó trả quyền quyết cho người dùng đúng chỗ Ice đã đặt. Cùng lý lẽ áp cho ca *"bóc ra rỗng"*: rơi về HTML thô là rỗng im lặng đổi hình dạng, không phải một đường lui.

**Vì sao `blocking`, không phải `async`.** Kho có **0** `async fn` tự viết và **0** `tauri::async_runtime`, nhưng **17** lệnh dùng `#[tauri::command(async)]` trên hàm ĐỒNG BỘ để đẩy thân hàm sang `sync_threadpool` — và `library.rs:640` chở một phép đo thật: thiếu nó là *"Not Responding"* trên macOS (Story 3.10b). Feature `blocking` đã bật sẵn, bàn đo 6.1 đã đo đủ ba năng lực bằng chính nó, **0** crate mới. ⚠️ Chỗ **chưa** đo, ghi ra thay vì giấu: bàn đo chạy từ một `#[test]` thuần, **ngoài** Tauri — chưa ai chứng minh `reqwest::blocking` sống được bên trong `sync_threadpool`. Đó là lý do Task 0 đứng trước mọi việc khác, và là lý do nó có quyền DỪNG cả story.

**Vì sao một bảng mã cho cả danh sách — và điều đó để hở cái gì.** Ice chốt 2026-09-06 đường ② của nợ `:9262`: giữ hình dạng hôm nay (`PipelineInput.encoding` là một `&'static Encoding`, chốt từ `chapters.first()` tại `project.rs:1635`) và **viết quyết định đó thành chữ**. ⚠️ Ghi thẳng chỗ yếu: link A trả GBK còn link B trả UTF-8 thì B bị giải mã bằng bảng mã của A — hoặc ra rác lặng lẽ, hoặc trượt `UndecodableBytes` với một lý do **không** nói đúng nguyên nhân. Đổi lại: **0** dòng mã pipeline đổi cho vế bảng mã, đường `Blob` không bị chạm, sáu ca test của 6.6 không bị chạm, và một dải năm ứng viên duy nhất đúng như mockup vẽ. Đường ① (dò độc lập từng đơn vị) vẫn mở cho một story sau.

## Verification

**Commands:**
- `npm run build && cargo test --locked` -- expected: **1171** ca xanh, 0 đỏ; `dist/` phải có TRƯỚC `cargo test`
- `npm run test` -- expected: ≥ **867** ca xanh (số trước story, đo 2026-09-06), 0 đỏ
- `npm run check:deps && npm run check:i18n && npm run check:tokens && npm run check:commands && npm run check:layout && npm run check:panel-refs && npm run check:gates && npm run check:debt-owner` -- expected: 0 finding mỗi cổng
- Đối chứng đỏ ①: `git stash` cổng mới `webimport_boundary.rs` rồi `cargo test --locked` -- expected: **xanh**
- Đối chứng đỏ ②: gieo `@tauri-apps/plugin-http` vào `package.json` rồi `npm run check:deps` -- expected: **đỏ**; gỡ ra -- expected: **xanh**
- Đối chứng đỏ ③: gỡ lời gọi `webimport::extract` khỏi `Step::ExtractMainContent` rồi chạy `webimport_contract.rs` -- expected: **đỏ** (ca "văn bản không chứa thẻ")
- `cargo test --locked --test segment_contract` -- expected: `schema_version() == 19` (`:1144`) và hai ca AD-39 (`:8020`, `:8066`) xanh **mà mệnh đề không đổi**
- `cargo build --release --locked --manifest-path src-tauri/Cargo.toml` × 2 (baseline qua `git worktree`, cùng `dist/`) -- expected: một delta byte ĐO ĐƯỢC, ghi vào spine §Deferred

**Manual checks (if no CLI):**
- Dán 3 link vào ô mới: hai con số hiện *3 link · sẽ tạo 3 Chương* ngay khi gõ, chưa bấm nút nào; DevTools Network trống.
- Bấm tải: tầng 2 hiện **văn bản** bài viết, không một thẻ HTML nào trên màn hình.
- Một link cố tình hỏng ở giữa: mục đó giữ đúng vị trí thứ hai, mang lý do đọc được; nút xác nhận mờ kèm câu giải thích; bỏ mục đó thì hai số cùng về 2.
- `Tab` xoay vòng qua ô dán và mọi nút mới, không thoát khỏi lớp phủ.

## Suggested Review Order

**Điểm ra mạng thứ ba — đọc trước tiên (AD-15 · AD-40)**

- Điểm vào: module tự khai hai nửa, kèm phép đo Task 0 cho `blocking` trong Tauri.
  [`webimport/mod.rs:18`](../../src-tauri/src/core/webimport/mod.rs#L18)

- Tám lý do hỏng — danh mục đóng, mỗi lý do một khoá i18n riêng.
  [`webimport/mod.rs:58`](../../src-tauri/src/core/webimport/mod.rs#L58)

- `Fetcher`: byte + `content-type`, 0 dòng phân tích nội dung.
  [`fetcher.rs:97`](../../src-tauri/src/core/webimport/fetcher.rs#L97)

- Trần chặng chuyển hướng — số 10 trích nguồn từ reqwest, không đúc mới.
  [`fetcher.rs:111`](../../src-tauri/src/core/webimport/fetcher.rs#L111)

- `content-type` so BẰNG sau khi cắt tham số, không khớp chuỗi con.
  [`fetcher.rs:198`](../../src-tauri/src/core/webimport/fetcher.rs#L198)

- `Extractor`: chỉ đọc `text_content` với `TextMode::Formatted`; `content` (HTML) không rời hàm.
  [`extractor.rs:47`](../../src-tauri/src/core/webimport/extractor.rs#L47)

**Chuỗi AD-39 — bước 2 lần đầu có thân thật**

- Bước 2 gọi XUỐNG `webimport::extract`, không viết lại nội tuyến.
  [`pipeline.rs:491`](../../src-tauri/src/core/segment/pipeline.rs#L491)

- Một bảng mã cho CẢ danh sách — quyết định của Ice, kèm ca còn hở ghi thẳng ra.
  [`pipeline.rs:226`](../../src-tauri/src/core/segment/pipeline.rs#L226)

**Đường lệnh — N link ⇒ N Chương, và 0 hàng ghi khi còn mục hỏng**

- Phép cắt dòng khớp ĐÚNG `trim()` của JS — chỗ lệch `U+FEFF` đo được.
  [`project.rs:2021`](../../src-tauri/src/commands/project.rs#L2021)

- Còn một mục hỏng, hoặc một mục "ok" thiếu byte ⇒ `None`, không Chương rỗng.
  [`project.rs:2042`](../../src-tauri/src/commands/project.rs#L2042)

- Dọn danh sách URL chỉ khi `create_work` thành công, cùng kỷ luật ô đang chờ.
  [`project.rs:1938`](../../src-tauri/src/commands/project.rs#L1938)

- Ba lệnh mới trong `generate_handler!` — thiếu một dòng là "command not found" lúc chạy.
  [`lib.rs:649`](../../src-tauri/src/lib.rs#L649)

**Hai con số — bằng chứng CẤU TẠO cho "chưa bấm ⇒ 0 lời gọi"**

- Đếm bằng computed cục bộ, JS thuần: không đường mã nào ở đây gọi được `invoke`.
  [`libraryImport.ts:127`](../../src/modes/libraryImport.ts#L127)

- Hai con số và câu cam kết hiện ngay dưới ô dán.
  [`LibraryMode.vue:1273`](../../src/modes/LibraryMode.vue#L1273)

- Mở lớp phủ từ danh sách URL — biến thể thứ ba của `lastSubmittedFrom`.
  [`importPreviewState.ts:467`](../../src/importPreviewState.ts#L467)

- Nút xác nhận khoá khi còn mục hỏng HOẶC đang có lượt sửa danh sách bay dở.
  [`ImportPreviewOverlay.vue:940`](../../src/ImportPreviewOverlay.vue#L940)

**Cổng và nghiệm thu — đọc sau cùng**

- Cổng ranh giới AD-40: gỡ nó ra thì bộ test cũ vẫn xanh (đã đo).
  [`webimport_boundary.rs:167`](../../src-tauri/tests/webimport_boundary.rs#L167)

- Văn bản đã bóc không chứa một dấu `<` nào của thẻ nguồn.
  [`webimport_contract.rs:191`](../../src-tauri/tests/webimport_contract.rs#L191)

- AC4 thành test: một BOM vô hình không được làm hai con số lệch nhau.
  [`webimport_contract.rs:679`](../../src-tauri/tests/webimport_contract.rs#L679)

- Cổng chặn plugin HTTP — trước story này không cổng nào chặn nó.
  [`check-deps.mjs:182`](../../scripts/check-deps.mjs#L182)
