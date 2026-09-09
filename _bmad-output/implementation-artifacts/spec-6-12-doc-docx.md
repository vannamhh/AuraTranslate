---
title: 'Story 6.12 — Đọc `.docx`'
type: 'feature'
created: '2026-09-09'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: 'd58cb771c39bfc51aa4f756717225abb2ccea7e4'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-6-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `.docx` là một trong ba định dạng FR13 hứa, nhưng hôm nay nó chỉ được nhắc để **TỪ CHỐI**: `import.rs:65` `SUPPORTED_EXTENSIONS = ["txt","md"]`, và `.docx` rơi vào `ImportError::UnsupportedFormat`. Hai mệnh đề kiến trúc đang đứng trên năng lực chưa ai đo: ① AD-39 *"`.docx` bỏ qua bước giải mã bảng mã"* tới nay nghiệm thu **bằng HÌNH DẠNG dựng tay**, chưa một tệp `.docx` thật đi qua (nợ `deferred-work.md:9170`, chủ story này); ② AD-38 — cổng chặn *"bảng một hàng, ô nhiều đoạn"* ở Story 8.8 — **không cài được** nếu không đọc ra được số hàng, số ô và **số đoạn trong từng ô**. Không có story này, Epic 8 xây trên một giả định.

**Approach:** Mở nhánh `.docx` tại đúng một chỗ (`import_file`) trả `PipelineShape::Blob(ChapterInput::AlreadyText)` — phần còn lại của chuỗi AD-39 (bỏ transcode, `SplitChapters`, xem trước, tách segment) **đã dựng sẵn và có test**. Đọc OOXML **bằng mã của ta** trên `zip` + `quick-xml`; ô bảng thành **đoạn riêng** để đóng nợ `:2214`; ảnh nhúng đi vào **đúng đường tài sản Story 6.11** với `source_url = NULL`.

**Quyết định Ice ký 2026-09-09 (ba câu hỏi mở đã đóng):**
1. **Giữ trọn spec** (~7.500 token, trên trần đề nghị 1.600) — cùng khuôn các spec Epic 6 trước.
2. **Mẫu đo do dev tự quy định**, không chờ Ice cấp. ⚠️ Kèm một mệnh đề phải ghi ra chứ không được để im: mẫu tự quy định đo được **ĐƯỜNG DÂY và LUẬT của ta**, nó **không** đo được *"Word thật sinh ra hình dạng XML gì"* — vế đó thành **nợ chủ Ice**, đúng tiền lệ FR126 (`6-1-ban-do/REPORT.md:12`), không thành một dấu tích.
3. **Tự đọc OOXML bằng `zip` + `quick-xml`**, không gọi `docx_rs::read_docx` trên đường sản phẩm. Đo trước khi chốt: `cargo tree -i` cho thấy **cả hai vào cây qua đúng `docx-rs`** (`zip 8.6.0`, `quick-xml 0.41.0`) ⇒ khai trực tiếp thêm **0 gói, 0 phiên bản**; cả hai có tệp giấy phép trên đĩa (`zip-8.6.0/LICENSE`, `quick-xml-0.41.0/LICENSE-MIT.md`). Lý do chọn: 140 điểm panic của `docx-rs/src/reader/` dưới `panic = "abort"` làm ô *"tệp hỏng"* của ma trận **không đóng được** bằng bất kỳ lớp chắn nào. `docx-rs` **giữ nguyên** vai bộ GHI cho Epic 8.

## Boundaries & Constraints

**Always:**
- `.docx` đến pipeline dưới `ChapterInput::AlreadyText` (AD-39 spine `:500`; `pipeline.rs:181` đã ghi thẳng tên story này trong doc-comment). **0 byte** đi vào vế transcode của `Step::DecodeEncoding`, nghiệm thu bằng một **tệp `.docx` đi trọn đường sản phẩm** (`import_file` → `run_pipeline` → ghi), không bằng một `ChapterInput` dựng tay trong test. ⚠️ Điều đó đóng nợ `:9170` **một nửa**: vế *"một tệp `.docx` thật sự đi qua"* xong, vế *"do Word sinh ra"* vẫn hở vì fixture là tự sinh (Quyết định 2) — ghi 🟡, đừng ghi ✅.
- Đi qua `run_pipeline` (`project.rs:269`). `segment_pipeline_boundary.rs:191` cưỡng chế literal `run_import(` xuất hiện **đúng một lần** trong `src-tauri/src/**` — thêm lời gọi thứ hai là đỏ.
- Bộ đọc `.docx` **không** đặt trong `core/webimport/`: `webimport_boundary.rs:270` cho phép `reqwest` ở đó, và một bộ đọc tệp cục bộ không được thừa kế quyền ra mạng. Module riêng, kèm cổng biên song song cấm `reqwest`/`TcpStream` trong tệp đó.
- Ảnh nhúng dùng **lại** máy của 6.11, không dựng nguồn sự thật thứ hai: `anchor::compute_anchor` (thuần), `assets::extension_for_mime` (danh mục ĐÓNG 4 MIME, SVG bị loại có chủ ý), khuôn ghi `<uuid>.<ext>` + `INSERT INTO asset` trong **cùng giao dịch** `store.write`. `source_url = NULL` đã hợp lệ theo lược đồ (`schema.rs:888-890`, test `asset_contract.rs:428`) ⇒ **không migration mới**, `PROJECT_MIGRATIONS` giữ đích **20**.
- Lỗi dựng **chỉ** qua `IpcError::new`; khoá mới khai bằng `message_keys!` **và** `src/i18n/vi.json` trong **cùng một lượt** (`ipc_contract.rs:232` canh parity, `check:i18n` thì không).
- 🔴 **Bộ đọc `.docx` mang 0 điểm panic**: không `unwrap()`/`expect()`/`panic!`/`unreachable!`/chỉ số mảng trần trên đường đọc — mọi nhánh trả `Result`. Đây là *cả lý do* chọn tự đọc, nên nó phải là một **cổng có test**, không một lời hứa trong doc-comment.
- **Thứ tự NFR15 là một phần của bản giao:** mở `~/.cargo/registry/src/…/zip-8.6.0/LICENSE` và `…/quick-xml-0.41.0/LICENSE-MIT.md` **bằng mắt** → ghi hai hàng vào bảng Stack spine `:803` (kèm đường dẫn đã mở và dòng đầu, khuôn `2-3-hop-dong-flush…:753-765`) → **rồi mới** sửa `Cargo.toml`. Ghim bằng `=`, mỗi crate một chú thích tiếng Việt nêu module sở hữu.
- Fixture của bộ test sản phẩm **sinh tại lúc chạy**, không commit blob nhị phân: `docx-rs` (bộ GHI — một cài đặt **độc lập** với bộ đọc của ta, nên không còn là vòng tròn) cho các ca thường, cộng zip dựng tay cho ca AD-38 và hai ca hỏng.
- Bàn đo AD-38 quét `src-tauri/tests/fixtures/docx/` — thư mục dành cho tệp **Word thật** nếu Ice thả vào sau. Rỗng ⇒ **báo 0 mẫu và thoát khác 0**, ghi *"chưa đo"*, **không phải** "đạt 0%" (khuôn `webimport_probe.rs:260`).

**Never:**
- **Không** gọi `docx_rs::read_docx` trên đường sản phẩm (Quyết định 3). Và **không** sinh fixture bằng chính bộ đọc của ta rồi đọc lại — đó mới là vòng tròn; dùng `docx-rs` hoặc zip dựng tay ở phía sinh.
- **Không** khai một dấu tích *"đã đo AD-38"* dựa trên fixture tự sinh: nó chứng minh **luật của ta**, không chứng minh **Word thật**.
- **Không** cài cổng AD-38 (từ chối bảng một hàng ô nhiều đoạn) ở story này — đó là **Story 8.8**. Story này chỉ **CẤP** năng lực đếm và chứng minh nó bằng số.
- **Không** đụng `PIPELINE_ORDER`, `assetProtocol.scope`, `capabilities/main.json`, `MINIMUM_HARVEST_SCHEMA_VERSION`.
- **Không** dựng đường nhập song ngữ hai cột (FR115) — đó là Story 6.16. Bảng ở đây chỉ để **đếm** và để **không cắt vắt qua ô**.
- **Không** sửa `epics.md`/spine cho khớp mã: vế nào không dựng được thì ghi nợ có chủ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Văn bản thường | `.docx` chỉ có đoạn | Text ra đúng; `Blob(AlreadyText)`; **0** lượt transcode; xem trước khai `self_declared` | N/A |
| Có bảng | Bảng N hàng × M ô | Đếm được **hàng · ô · đoạn trong từng ô**; mỗi ô là một **đoạn riêng** trong văn bản ra | N/A |
| Ô nhiều câu | Một ô chứa 2 câu | Ranh giới ô = ranh giới đoạn ⇒ **0** segment vắt qua hai ô (đóng nợ `:2214`) | N/A |
| Ảnh nhúng | `word/media/image1.png` | Một tệp thật trong `assets/`, một hàng `asset`, `source_url IS NULL`, neo đúng vị trí | MIME ngoài danh mục 4 ⇒ bỏ ảnh, ghi `images_failed`, Chương **vẫn** nhập |
| Tệp hỏng | zip cắt cụt / `document.xml` sai | Lỗi phân biệt được qua `IpcError`, **0 byte** ghi xuống, **0** thư mục `.atproj` để lại | `Result` ở mọi nhánh — ca này là lý do tồn tại của Quyết định 3 |
| Đuôi `.docx`, không phải zip | Tệp `.txt` đổi tên | Cùng lỗi như trên; nhận theo **nội dung**, không theo tên | Như trên |
| Rỗng | `.docx` 0 đoạn | Lỗi "không có văn bản", cùng khuôn tệp rỗng của `.txt` | Không ghi |
| Quá cỡ | > `MAX_IMPORT_BYTES` (100 MB, `import.rs:80`) | Từ chối **trước** khi đọc zip | Khoá lỗi đã có |

</frozen-after-approval>

## Code Map

**Đường nhập một tệp — chỗ duy nhất phải mở**
- `src-tauri/src/core/segment/import.rs:65` `SUPPORTED_EXTENSIONS = ["txt","md"]`, cưỡng chế ở `:522` `reject_unsupported_extension`; `:479` `import_file` là **nơi duy nhất tạo `ChapterInput`** (`:507` trả `Blob(RawBytes)`), `:80` `MAX_IMPORT_BYTES` 100 MB. `.docx` bị từ chối ở `:63`/`:90` (`UnsupportedFormat`) và `commands/project.rs:1563`.
- 🔴 **Không có hộp thoại native cho đường nhập** — người dùng gõ/thả vào ô text (`src/modes/LibraryMode.vue:1247`, `src/modes/libraryImport.ts:404`). ⇒ Gác phần mở rộng nằm **100% ở Rust**; không có bộ lọc UI nào phải sửa.
- `src-tauri/src/core/segment/pipeline.rs:181` `ChapterInput::AlreadyText(String)` — doc-comment **đã nêu đích danh Story 6.12**; `:794` `Unit::Decoded(t) => Ok(Unit::Decoded(strip_bom(t)))` là đường bỏ transcode **đã dựng**. Test khoá: `segment_contract.rs:8133`. Tầng lệnh cũng đã có nhánh tự khai: `project.rs:2596` (`self_declared_utf8()`, 0 ứng viên bảng mã).
- `PipelineShape::Blob` ⇒ `Step::SplitChapters` **CHẠY** (`pipeline.rs:190`) — đúng cho `.docx` (một tệp, chưa chia Chương).
- `src-tauri/src/core/segment/import.rs:412` `ImportedChapter { source_text, segments, cleanup_report, title, blocks, joined_line_count }` — trường `blocks: Option<Vec<Block>>` là chỗ ảnh `.docx` phải sống để pha ảnh nhìn thấy.

**Đường tài sản 6.11 — cắt tại đâu để tái dùng**
- `src-tauri/src/commands/project.rs:534` `prepare_chapter_images(...)` chạy **sau** `run_pipeline:450`, **trước** `store.write:560`; `INSERT INTO asset` ở `:606-618` trong cùng giao dịch; `:688` trả `images_saved`/`images_failed`.
- 🔴 **Ranh giới tái dùng: `fetch_and_write_one_asset` (`project.rs:1017-1119`).** `:1023` `webimport::fetch(...)` là điểm mạng **duy nhất** — thay nó bằng "đọc byte từ zip" và giữ nguyên `:1037` `extension_for_mime` (cổng MIME duy nhất), `:1069` `normalized_mime`, `:1079-1094` đặt tên `<uuid>.<ext>` + `fs::write` duy nhất. `:884-886` (`host_of` → `with_tier2_hosts`) là phần **thuần mạng**, đường `.docx` không đi qua.
- `src-tauri/src/core/segment/anchor.rs:62-70` `compute_anchor(blocks, effective_kept, image_index, full_source_text, segments, cleanup_rules, source_lang)` — **hoàn toàn thuần**, tái dùng nguyên vẹn nếu `.docx` dựng đúng `Vec<Block>`; `:98` là chỗ nó **tự kiểm** tiền tố và trả `Err` thay vì làm tròn về `0`.
- `src-tauri/src/core/webimport/extractor.rs:112-123` `BlockBody::{Paragraph(String), Image{src,alt}, Caption(String)}` + `Block { body, machine_kept, exact_gap_before, is_list_item }` — hình dạng bắt buộc. ⚠️ `Image.src` là `Option<String>`: ảnh `.docx` **không có URL**, đây là chỗ khác biệt phải nghĩ, không phải chỗ bịa một URL giả.
- `src-tauri/src/core/webimport/assets.rs:159` `extension_for_mime` (danh mục ĐÓNG 4 MIME, `_ => None` **là** vế từ chối), `:132` `normalized_mime`. ⚠️ `:65` `resolve_absolute_url` **không** tái dùng được — nó chặn scheme khác http/https.
- `asset` DDL: `schema.rs:981-1000` (migration `to_version: 20`, `:1828`). `source_url` là cột **duy nhất** cho NULL, doc-comment `:888-890` nói thẳng nó dành cho ảnh không đến từ mạng.

**OOXML — cái gì phải đọc, và vì sao không qua `docx-rs`**
- Đường đọc: `word/document.xml` (đoạn `w:p`, bảng `w:tbl`/`w:tr`/`w:tc`, chữ `w:t`), `word/_rels/document.xml.rels` (rId → `media/imageN.png`), `word/media/*` cho byte ảnh, `[Content_Types].xml` cho MIME. ⚠️ Đường trong zip có thể mang `\` (tệp nén trên Windows) — `docx-rs/src/reader/read_zip.rs:8-14` xử lý đúng ca đó, chép **lý do** chứ đừng chép mã.
- 🔴 **Vì sao không `docx_rs::read_docx`, đo 2026-09-09 trên nguồn đã tải:** **140 điểm `unwrap()`/`expect()`/`panic!`/`unreachable!` trong mã sản phẩm** của `src/reader/` (đã cắt `#[cfg(test)]`), trên **63/77 tệp**. Điểm nằm đúng trên ca *"tệp hỏng"*: `read_zip.rs:20` `xml.read_to_end(&mut data).unwrap()` — luồng deflate cắt cụt là panic, không `Err`. `Cargo.toml:167` `panic = "abort"` ⇒ chết tiến trình; `catch_unwind` vô dụng. ⚠️ Và `Cargo.toml:173` ghi rằng test target **luôn** dựng bằng `unwind` ⇒ **một cổng xanh ở đây không nói gì về bản phát hành**.
- `docx-rs` vẫn ở lại: chủ `core::export` (Epic 8), và là **bộ sinh fixture** cho bộ test của story này. `documents/elements/table_cell.rs:19` `TableCellContent::{Paragraph, Table, StructuredDataTag, TableOfContents}` là khuôn hình dạng bảng để đối chiếu.
- `zip 8.6.0` + `quick-xml 0.41.0`: **cả hai vào cây qua đúng `docx-rs`** (`cargo tree -i`, đo 2026-09-09) ⇒ khai trực tiếp thêm 0 gói. ⚠️ `quick-xml 0.38.4` cũng có trong cây (qua `tauri` → `plist`) — khai **0.41.0** để không thêm một phiên bản thứ ba.

**Cổng sẽ nói gì**
- `src-tauri/Cargo.toml:41` `docx-rs =0.4.22` khai chủ *"core::export (AD-38)"*, **0 lời gọi sản phẩm** hôm nay (`deferred-work.md:3777`) ⇒ story này là chỗ gọi đầu tiên; sửa chú thích chủ sở hữu tại chỗ.
- `check:deps` (`scripts/check-deps.mjs`) **không** kiểm giấy phép — nó quét tên crate cấm + `--locked`. NFR15 là quy trình người: mở tệp trong `~/.cargo/registry/src/…`, ghi bảng Stack spine `:803` **trước** khi sửa `Cargo.toml`.
- 11 cổng `pre-push:81`: `deps tokens i18n commands layout panel-refs dict dict-manifest lint gates debt-owner`. `check:debt-owner` đỏ nếu một mục `open` thiếu `Chủ:` thật.
- ⚠️ `naming_boundary.rs:444` `FORBIDDEN_WORDS_LOWER` cấm `document` **trong chuỗi `code` đầu của `IpcError::new(`** (bốn từ cấm dạng thường). Chuỗi `"word/document.xml"` trong mã thì an toàn — cổng chính `:103` so **phân biệt hoa/thường** trên `Document`. ⇒ Đặt mã lỗi là `docx.unreadable`, **không** `docx.document_unreadable`.
- `vi.json:9` đang ghi *"chỉ .txt và .md mở được"* — **hết đúng** từ story này, phải sửa cùng lượt.
- ⚠️ **Trước khi đọc một lượt đỏ ở `webimport_contract.rs`/`asset_contract.rs` là hồi quy:** kiểm điều kiện Application Firewall macOS ở `deferred-work.md:10409` (đã quan sát **ba lần** trong một phiên, lần thứ ba chặn cả `git push`). Một lỗi có ở cả hai phía không phải lỗi của thứ đang bị nghi.

## Tasks & Acceptance

**Execution:**
- [x] `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md:803` — **LÀM TRƯỚC MỌI THỨ**: hai hàng Stack mới (`zip 8.6.0`, `quick-xml 0.41.0`) sau khi mở tệp giấy phép bằng mắt, ghi đường dẫn đã mở + dòng đầu. Rồi mới `src-tauri/Cargo.toml` (ghim `=`, chú thích nêu module sở hữu).
- [x] `src-tauri/src/core/docx/mod.rs` (**mới**) — đọc OOXML → ① văn bản đã ghép theo đoạn, ② `Vec<Block>` (đoạn + ảnh), ③ cấu trúc **đếm bảng** (`rows`, `cells_per_row`, `paragraphs_per_cell`) cho AD-38. Hàm thuần, **`Result` ở mọi nhánh, 0 điểm panic**, không chạm đĩa ngoài byte được truyền vào.
- [x] `src-tauri/src/core/segment/import.rs:65/:479` — thêm `"docx"` vào `SUPPORTED_EXTENSIONS`; nhánh mới trong `import_file` trả `Blob(ChapterInput::AlreadyText(text))`; gỡ `.docx` khỏi nhánh `UnsupportedFormat` (`:63`/`:90`) và `commands/project.rs:1563`.
- [x] `src-tauri/src/commands/project.rs:1017` — tách `fetch_and_write_one_asset` thành hai nửa: "lấy byte" (mạng **hoặc** zip) và "ghi + dựng hàng"; đường `.docx` đi nửa sau với `source_url = None`. **Không** dựng `Allowlist` nào trên đường này.
- [x] `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` — khoá lỗi mới cho `.docx` không đọc được / không có văn bản, đúng khuôn `message_keys!`; **sửa `vi.json:9`** ("chỉ .txt và .md") vì mệnh đề đó hết đúng.
- [x] `src-tauri/src/core/export/mod.rs:6` + `Cargo.toml:41` — 🔵 sửa **tại chỗ** chú thích chủ sở hữu `docx-rs` (nay có cả đường ĐỌC), kèm ngày và lý do.
- [x] `src-tauri/tests/docx_boundary.rs` (**mới**) — hai cổng biên, mỗi cổng kèm **ca tự kiểm** chứng minh vị từ bắt được một token gieo (khuôn `webimport_boundary.rs:197`) và một sàn số tệp để phép quét không xanh vì quét rỗng: ① module đọc mang **0** dòng nhắc `reqwest`/`TcpStream`/`http://`/`https://`; ② module đọc mang **0** điểm `unwrap()`/`expect()`/`panic!`/`unreachable!` trong mã sản phẩm — đây là mệnh đề mà cả Quyết định 3 dựa lên.
- [x] `src-tauri/tests/docx_contract.rs` (**mới**) — mọi hàng của ma trận I/O, chạy trên fixture **thật**; cộng ca *"đường `.txt`/dán tay không đổi một byte"*.
- [x] `src-tauri/tests/fixtures_docx.rs` (**mới**, helper dùng chung) — sinh bảy fixture tại lúc chạy, **không commit blob**: `plain` · `table_two_columns` · `table_one_row_multi_paragraph` (ca AD-38) · `image_png` · `empty` (0 đoạn) — bằng `docx-rs` (bộ GHI); `truncated` (chặt nửa một tệp hợp lệ) và `not_a_zip` — dựng tay.
- [x] `src-tauri/tests/docx_probe.rs` (**mới**) + `_bmad-output/implementation-artifacts/6-12-ban-do/` — bàn đo `#[ignore]` theo khuôn `webimport_probe.rs`: quét `src-tauri/tests/fixtures/docx/*.docx` (tệp **Word thật**, gitignore), ghi TSV (tên tệp · số hàng · số ô · số đoạn từng ô · số ảnh · lỗi), **0 mẫu ⇒ thoát khác 0**. `REPORT.md` + `environment.txt` mang `baseline_commit` đầy đủ.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — đóng bằng chữ: `:9170` → **🟡** (một tệp `.docx` đi trọn đường sản phẩm; vế "do Word sinh ra" còn hở) và `:2214` → ✅ (câu bị cắt giữa ô bảng); ghi nợ **MỚI có chủ** cho mọi vế còn hở: 🔴 **0 tệp Word THẬT đi qua — fixture tự sinh chứng minh luật của ta, không chứng minh hình dạng XML Word sinh ra (Chủ: Ice)**; ảnh SVG/EMF/WMF; header/footer/footnote/ghi chú chưa đọc; `w:tbl` lồng trong ô.

**Acceptance Criteria:**
- 🔴 Given một `.docx` có ảnh nhúng đi trọn đường sản phẩm, when nhập và xác nhận, then mỗi ảnh giữ có **một tệp thật** trong `assets/` và **một hàng `asset`** với `source_url IS NULL`; ca này phải **ĐỎ** nếu ai chỉ ghi hàng SQL mà không ghi tệp.
- 🔴 Given phép **GỠ** nhánh `"docx"` khỏi `SUPPORTED_EXTENSIONS`, when chạy bộ test **MỚI**, then nó phải **ĐỎ** — đối chứng là một phép GỠ biên dịch được và chạy, không phải một lập luận.
- 🔴 Given một `.docx` có bảng, when đọc, then **0** segment nào vắt qua ranh giới hai ô, và ca này phải **ĐỎ** nếu ô bảng được nối thành một dòng phẳng.
- 🔴 Given một lượt nhập `.txt`/`.md`/dán tay và **không thao tác tay nào**, when xác nhận, then byte ghi xuống `.atproj` **trùng đúng** kết quả trước story này.
- 🔴 Given một `.docx` **cắt cụt** và một tệp text đổi đuôi thành `.docx`, when nhập, then cả hai trả một `IpcError` phân biệt được, **0 byte** ghi xuống, **0** thư mục `.atproj` để lại — và **0** lượt panic. ⚠️ Ca này chạy dưới `unwind` nên nó chứng minh *"mã của ta trả `Result`"*, **không** chứng minh gì về bản `abort`; mệnh đề còn lại được cổng `docx_boundary` §0-điểm-panic đóng.
- Given bàn đo chạy trên thư mục tệp Word thật, when đọc xong, then `REPORT.md` ghi **số mẫu THẬT**, và 0 mẫu ghi **"0 mẫu — chưa đo"** kèm một chủ ở `deferred-work.md`, **không** một phán quyết.
- Given `cargo test`, when chạy, then `PROJECT_MIGRATIONS` vẫn đích **20**, `MINIMUM_HARVEST_SCHEMA_VERSION` vẫn **8**, `assetProtocol.scope` **không đổi**, và `PIPELINE_ORDER` vẫn đúng 7 bước theo thứ tự AD-39.
- Given **mười một** cổng `pre-push` cộng `cargo test --locked` cộng `npm run test`, when chạy trọn, then **0** finding và **0** ca đỏ.

## Implementation Notes

**Vòng nghiệm thu của phiên điều phối (2026-09-09) — số ĐO, không phải số chép.**

- `cargo test --locked --no-fail-fast` **SAU vòng rà và bảy mục vá**: **50 binary · 1.331 xanh · 26 đỏ · 12 ignored** (trước vòng rà: 48 binary · 1.323 xanh · 27 đỏ).
- 26 ca đỏ vẫn nằm TRỌN trong hai binary mở kết nối thật (`asset_contract` 12, `webimport_contract` 14) — điều kiện môi trường đã chứng minh ở ma trận dưới đây, không phải hồi quy.
- `docx_contract` **22/22 xanh** (17 trước vòng rà + 5 ca vòng rà thêm), `docx_boundary` **9/9 xanh**.
- 🔴 **Đối chứng đỏ cho phép sửa `mc:Fallback`:** gỡ `"Fallback"` khỏi `OPAQUE_LOCAL_NAMES` ⇒ ca `a_word_alternate_content_wrapper_around_one_image_yields_exactly_one_image_block` **ĐỎ**; trả lại ⇒ **XANH**.
- 27 ca đỏ nằm TRỌN trong hai binary mở kết nối thật: `asset_contract` (12) và `webimport_contract` (15).
- `npm run test`: **70 tệp · 953 ca · 0 đỏ**. Mười một cổng `pre-push`: chạy TỪNG cổng, **11/11 OK** — cả hai chạy lại sau vòng vá.

🔴 **27 ca đỏ đó KHÔNG phải hồi quy của story này — và phép đo suýt nói ngược.** Ma trận đo:

| Cây | Chỗ đặt binary | Luồng | Kết quả |
|---|---|---|---|
| baseline `d58cb77` | `/private/tmp` | mặc định | 19 xanh (5,21 s) |
| **baseline `d58cb77`** | **repo** | mặc định | **7 xanh / 12 đỏ** (9,85 s) |
| story | repo | mặc định | 12 đỏ · 12 đỏ · **19 xanh** |
| story | `/private/tmp` | mặc định | 19 xanh (1,26 s) |
| story | repo | `--test-threads=1` | 19 xanh; một lượt khác 18/19 |

⚠️ **Hai bài học của lượt rà này, ghi ra vì cả hai suýt thành một kết luận sai:** ① hai lượt đỏ
giống hệt nhau **không** đủ để gọi là tất định — lượt thứ ba xanh mà không đổi một dòng mã;
② baseline chạy ở `/private/tmp` còn cây story chạy trong repo là **so hai ô chéo nhau**, và ô
còn thiếu (baseline + repo) mới là ô lật kết luận. Nguyên nhân đã có chủ ở `deferred-work.md`
(ca `a_disk_write_failure_...` là ca bập bênh cả khi một luồng).

**Một hàng ma trận I/O không có ca test nào — bù trong lượt rà, không ghi nợ.** Vế xử lý lỗi
của hàng *"Ảnh nhúng"* (*"MIME ngoài danh mục 4 ⇒ bỏ ảnh, ghi `images_failed`, Chương vẫn
nhập"*) không được một dòng test nào chạm: bảy fixture đầu đều mang PNG hợp lệ. Thêm
`fixtures_docx::image_unsupported_mime` (zip dựng TAY — `docx-rs::Pic` luôn ghi đuôi `.png`,
nên không có đường nào bảo nó sinh `.bmp`) và
`docx_contract::an_embedded_image_with_a_mime_outside_the_closed_four_is_dropped_and_the_chapter_still_imports`,
kiểm bằng trạng thái THẬT (thư mục `assets/` rỗng, bảng `asset` 0 hàng) chứ không suy từ
`images_failed`. **Đối chứng đỏ đã chạy:** cho `mime_for_media_extension` ánh xạ `bmp` sang một
MIME trong danh mục ⇒ ca **ĐỎ**; trả lại ⇒ **XANH**.


- `core/docx/mod.rs` (mới, 607 dòng) — đọc OOXML bằng `zip` (`default-features = false,
  features = ["deflate"]`, khớp đúng khai báo của `docx-rs` cho crate này — bật default
  features suýt kéo `time >=0.3.47` xung đột với `time 0.3.45` đã khoá qua `cookie`/`tauri`,
  đo được bằng `cargo check` đỏ TRƯỚC khi sửa) + `quick-xml 0.41.0`. Duyệt sự kiện theo LOCAL
  NAME (bỏ tiền tố namespace, đơn giản hoá có chủ, ghi nợ nếu cần siết bằng URI thật). Văn bản
  trả về dựng bằng CHÍNH `core::segment::pipeline::join_kept_blocks` trên chính `Vec<Block>`
  vừa dựng — một nguồn sự thật cho cả "văn bản vào pipeline" lẫn "khối để tính neo ảnh", loại
  bỏ nguy cơ hai bản trôi khỏi nhau mà không cần một test đối chiếu riêng.
- `core::segment::import::import_file` đổi chữ ký từ `Result<PipelineShape, _>` sang
  `Result<(PipelineShape, Option<DocxSidecar>), _>` — thay đổi DUY NHẤT lên chữ ký hàm thuần
  đã có; mọi chỗ gọi (`create_work_from_file`, `wire::preview_import_encoding_from_file`, ba
  test cũ trong `project_contract.rs`) cập nhật để đọc `.0` khi không cần sidecar.
- `commands::project`: `create_work`/`stash_pending_import_source`/`confirm_import_with_encoding`
  thêm tham số `docx_sidecar`; khối + ảnh `.docx` gắn vào Chương ĐẦU TIÊN sau khi `run_pipeline`
  chạy xong (`.docx` không đi qua `Step::ExtractMainContent` — đó là bóc HTML). `prepare_chapter_images`
  nhận thêm `docx_images: &[DocxImage]`; ảnh `.docx` (`src: None` + byte có sẵn) đi qua một
  nhánh `PendingImageSource::Local` MỚI, tách biệt hoàn toàn khỏi `Remote` (0 `Allowlist`, 0
  mạng, 0 `DomainLogState`). `fetch_and_write_one_asset` tách thành hai nửa
  (`fetch_asset_bytes_over_network` + `write_local_asset_bytes`) đúng Task list; `.docx` gọi
  thẳng nửa ghi. `SavedAsset::source_url` đổi từ `String` sang `Option<String>` (`NULL` cho
  ảnh `.docx`).
- Bảy fixture (`tests/fixtures_docx.rs`, helper dùng chung qua `#[path]`) sinh tại lúc chạy
  bằng `docx-rs` (bộ GHI, cài đặt độc lập) + hai fixture hỏng dựng tay — không commit blob.
  Ca "Ô nhiều câu" đặt vào ô `(0,0)` của `table_two_columns` (không phải fixture thứ tám).
- `deferred-work.md`: `:9170` → 🟡 (một tệp `.docx` đi trọn đường sản phẩm; vế "do Word sinh
  ra" còn hở), `:2214` → ✅ **cho `.docx`** (ghi rõ Markdown bảng phẳng trong `.txt`/`.md` vẫn
  hở, KHÔNG được đọc thành "mọi bảng đã đóng"); bốn mục nợ mới có chủ (Ice): SVG/EMF/WMF chưa
  đọc, header/footer/footnote chưa đọc, `w:tbl` lồng trong ô chưa đọc, Markdown-bảng-phẳng
  chưa có bộ đọc cấu trúc.
- **Phát hiện ngoài phạm vi spec, ghi nợ chứ không sửa:** `tests/asset_contract.rs` (Story
  6.11) đỏ ngẫu nhiên (tới 12/19 ca) dưới đa luồng mặc định của `cargo test` — đo được nguyên
  nhân là tranh chấp scheduler giữa nhiều `TcpListener`/luồng-đua-chmod/`thread::sleep` chạy
  ĐỒNG THỜI trong CÙNG tệp (`--test-threads=1` cho 19/19 xanh ổn định, lặp lại nhiều lượt);
  đối chứng trên baseline `d58cb77` (qua `git worktree`, không đụng cây làm việc) cho thấy
  biến thiên tương tự, không phải một hồi quy của story này. Ghi ở `deferred-work.md`, chủ
  Ice (quyết định kiến trúc bộ test của Story 6.11).

**Lượt code review 2026-09-09 — sáu mục, sửa tại chỗ:**
- `mc:AlternateContent` (Word gói MỘT ảnh thành `mc:Choice > w:drawing` + `mc:Fallback >
  w:pict` cùng rId) từng cho HAI khối `Image` vì `Fallback` chưa nhảy trọn subtree — thêm
  `"Fallback"` vào `OPAQUE_LOCAL_NAMES`; fixture dựng tay
  `fixtures_docx::image_wrapped_in_alternate_content` + ca
  `a_word_alternate_content_wrapper_around_one_image_yields_exactly_one_image_block`.
- Nhánh VML (`w:pict`/`v:imagedata`) của `find_image_rel_id` chưa từng chạy qua một ca test —
  fixture `image_vml_only` (chỉ `w:pict`, 0 `w:drawing`) + ca
  `a_vml_only_image_with_no_drawing_anywhere_still_resolves_its_rel_id_and_reads_real_bytes`.
- `read_zip_entry` dùng `.ok()?` — một mục zip lỗi BẤT KỲ làm thoát trọn hàm, báo
  `MissingEntry` oan cho `word/document.xml` lành lặn. Đổi sang bỏ qua đúng mục lỗi, quét
  tiếp.
- Rels không phải UTF-8 từng `unwrap_or_default()` im lặng — nay `eprintln!` một dòng chẩn
  đoán (không dùng tiền tố `"docx[rels]"` — khớp nhầm vị từ chỉ số mảng trần của
  `docx_boundary.rs`, đổi sang dấu hai chấm).
- `mime_for_media_extension`: tên tệp media không dấu chấm từng làm `ext` thành CẢ ĐƯỜNG DẪN
  (`rsplit('.').next()` không khớp thì trả nguyên chuỗi) — đổi sang lấy basename rồi
  `rsplit_once('.')`, cho `ext = ""` đúng nghĩa khi không có đuôi; kết quả từ chối không đổi.
- Hai mệnh đề tự khai không phép đo: nhảy `w:tbl` lồng rồi đọc tiếp phần còn lại của ô (ca
  `a_paragraph_after_a_nested_table_in_the_same_cell_is_still_read`, fixture
  `table_with_nested_table_and_trailing_paragraph`) và đường `.docx` là "0 `DomainLogState`"
  (ca `importing_a_docx_with_an_image_leaves_the_domain_log_empty`).
- Giới hạn `DocxSidecar` (chỉ Chương đầu đọc `blocks`) khi mẫu phân tách cho N > 1 Chương
  THẬT: ca mới `an_image_after_a_chapter_split_boundary_fails_distinguishably_instead_of_being_saved_to_the_wrong_chapter`
  đo được ảnh sau ranh giới TRƯỢT có thể phân biệt được (không lưu sai Chương) nhưng cũng
  không được lưu đúng Chương của nó — ghi nợ mới vào `deferred-work.md`, **Chủ: Ice**.

## Spec Change Log

## Review Triage Log

### Vòng rà 1 — 2026-09-09 (ba lớp: blind-hunter · edge-case · verification-gap)

| # | Phát hiện | Verdict | Bằng chứng phân xử | Tuyến |
|---|---|---|---|---|
| 1 | `mc:AlternateContent` của Word gói CẢ `w:drawing` (Choice) LẪN `w:pict` (Fallback) cho CÙNG một rId; `parse_paragraph` coi `mc:*` là trong suốt nên khớp cả hai ⇒ **hai** khối ảnh cho một tấm | medium | Đọc `core/docx/mod.rs:450` — nhánh `"drawing" \| "pict"` xử lý y hệt nhau, và `OPAQUE_LOCAL_NAMES` không chứa `Fallback`. Một `.docx` mang khuôn đó cho hai `ParaPiece::Image` cùng rId ⇒ hai tệp, hai hàng `asset`, neo lệch | patch |
| 2 | `.docx` chỉ có ảnh, 0 đoạn chữ ⇒ `EmptyText`, ảnh mất theo | low | Có thật (`mod.rs:192`), nhưng **đúng** hàng "Rỗng" của Ma trận I/O đã đóng băng; sửa nó là sửa spec của chính lượt build này | bác |
| 3 | Nhánh VML (`v:imagedata`) 0 ca test — khai trong doc-comment, không ai kiểm | medium | `grep` `docx_contract.rs`: 0 lần nhắc `imagedata`/`pict`. Mệnh đề "đọc VML hạn chế" ở `mod.rs` mục 7 và trong sổ nợ đều dựa trên mã chưa từng chạy | patch |
| 4 | Bỏ qua `w:tbl` lồng trong ô: 0 ca kiểm việc phân tích **tiếp tục đúng** phần còn lại của ô/hàng sau lượt nhảy | medium | `parse_cell` nhánh `"tbl"` gọi `read_to_end` rồi đi tiếp; không test nào đặt một đoạn SAU bảng lồng trong cùng ô. Giới hạn được ghi nợ, nhưng **an toàn của chính phép nhảy** thì không | patch |
| 5 | `read_zip_entry` quét tuyến tính mọi mục mỗi lần gọi — O(ảnh × mục) | low | Đúng mã, nhưng một `.docx` có vài chục mục; sửa đòi dựng chỉ mục (thêm phức tạp, không phải một phép sửa thẳng) | bác |
| 6 | `sprint-status.yaml` `in-progress` trong khi spec `in-review` | false | Đúng thiết kế của workflow: sprint lên `review` ở Bước 5, sau khi vòng rà đóng. Không phải chỗ lệch | bác |
| 7 | Hai nửa (`fetch_asset_bytes_over_network` / `write_local_asset_bytes`) cùng gọi `extension_for_mime` + lặp nhánh `Err` bất biến | low | Nguồn sự thật của BẢNG MIME vẫn đúng MỘT (`assets::extension_for_mime`); cái lặp là lời gọi và một `eprintln`. Gộp lại đòi thêm một hàm chung — hơn một phép sửa thẳng | bác |
| 8 | Byte ảnh `.docx` bị clone nhiều lần (`img.bytes.clone()`, clone `docx_sidecar` lúc xác nhận) | low | Có thật (`project.rs:3145`), nhưng trần `MAX_IMPORT_BYTES` 100 MB chặn cả tệp; sửa đòi đổi sang `Arc`/move xuyên hai tầng — thêm phức tạp | bác |
| 9 | Mệnh đề §Always *"đường `.docx`: 0 mạng, 0 Allowlist, 0 DomainLogState"* không một ca test nào canh | medium | `grep domain_log` trong `docx_contract.rs`: **0 lần**. Đây là một mệnh đề spec tự khai mà không có phép đo — đúng lớp lỗi `AGENTS.md` gọi tên | patch |
| 10 | `mime_for_media_extension` với tên tệp media không có dấu chấm: `rsplit('.')` trả nguyên đường dẫn | low | Kết quả CUỐI vẫn đúng (rơi vào nhánh từ chối), chỉ chuỗi chẩn đoán sai lệch; sửa là một phép lọc một dòng | patch |
| 11 | Fixture `image_unsupported_mime` thiếu `[Content_Types].xml` | low | Bộ đọc của ta **không mở** tệp đó (`read_docx` chỉ đọc `document.xml`/rels/media), nên thêm nó không chứng minh thêm điều gì | bác |
| 12 | **`.docx` tách thành N Chương bằng mẫu phân tách: ảnh sau ranh giới Chương 0 mất hẳn** — Chương ≥1 có `blocks: None` nên bị `continue`, còn Chương 0 lại tính neo cho chúng trên `source_text` đã cắt cụt ⇒ `images_failed` gán nhầm | high | Lớp verification-gap nộp kèm bằng chứng đã truy vết; tôi đọc lại `project.rs:857-900` xác nhận: `blocks` chỉ gắn ở `chapters.first_mut()`, `compute_anchor` nhận `&chapter.source_text`. Đường UI thật đi qua `confirm_import_with_encoding` có `chapter_pattern` | patch |
| 13 | `read_zip_entry`: `archive.by_index(i).ok()?` bỏ TRỌN phép quét nếu một mục bất kỳ mở lỗi | medium | Đọc `mod.rs:205`: `?` thoát cả hàm ⇒ `word/document.xml` lành lặn bị báo `MissingEntry`. Có thật, và `zip` khai `default-features = false` nên một mục nén bằng phương pháp khác là đủ | patch |
| 14 | rels không phải UTF-8 ⇒ `unwrap_or_default()` ⇒ map rỗng, mọi ảnh mất ánh xạ, **0 tín hiệu** | medium | Đọc `mod.rs:148`. Ảnh vẫn đếm vào `images_failed` nên không hoàn toàn câm, nhưng LÝ DO thì biến mất — đúng lớp "rỗng im lặng" mà `AGENTS.md` gọi là lớp lỗi trung tâm | patch |
| 15 | (trùng #10 — cùng dòng, cùng mệnh đề) | low | Gộp vào #10 | patch |

**Nhóm theo nguyên nhân gốc:** #1+#3 (một khuôn XML của Word thật không được xử lý, và nhánh VML chưa ai chạy) · #13+#14+#10 (`core::docx` nuốt lỗi ở ba chỗ đọc gói) · #12 (`DocxSidecar` gắn vào Chương 0 trong khi pipeline có thể tách N Chương) · #4 · #9 (hai mệnh đề tự khai không có phép đo).


## Design Notes

**Vì sao `.docx` đi vào bằng `AlreadyText` chứ không một biến thể `ChapterInput` thứ ba.** Bảng hình dạng AD-39 (spine `:486-491`) phân loại theo **hình dạng dữ liệu**, không theo đường nhập: `.docx` là *"đã là văn bản, nguồn tự khai bảng mã"* — cùng hộc với văn bản dán tay. `pipeline.rs:181` đã viết sẵn tên story này trong doc-comment từ Story 6.2, và `:794` đã có nhánh bỏ transcode. Thêm một biến thể thứ ba là nhân đôi một hình dạng đã có tên, và làm `segment_contract.rs:8133` mất nghĩa.

**Vì sao ô bảng thành một đoạn riêng.** Nợ `:2214` mô tả ca sai đã quan sát: một ô chứa hai câu bị bộ tách cấp câu cắt tại dấu chấm giữa ô, cho một segment mang `|` mồ côi. Story 6.5 đã **từ chối** vá ở tầng làm sạch với lý do đúng: `core::cleanup` chỉ biết XOÁ theo mẫu văn bản, nó không biết "hàng bảng" là gì. Tầng duy nhất biết là tầng ĐỌC — nơi cấu trúc bảng còn tồn tại trước khi mọi thứ phẳng thành chuỗi. Đặt ranh giới ô = ranh giới đoạn thì bộ tách câu **không bao giờ** có cơ hội cắt vắt qua, và không một dòng nào của `core/segment/split.rs` phải biết Markdown hay OOXML là gì.

**Vì sao một lớp chắn trước `docx-rs` bị loại, dù nó rẻ hơn.** Phương án đó kiểm chữ ký zip rồi giải nén thử trước khi trao byte — nó chặn được ca *"không phải zip"*, nhưng 140 điểm panic không nằm ở khâu mở zip; chúng nằm rải trên **63 tệp** của đường phân tích XML, nơi một `w:tc` thiếu con hay một rId trỏ hụt cũng đủ. Một lớp chắn đóng được ca dễ nhất và để nguyên lớp ca mà ma trận I/O gọi tên. ⚠️ Và nó tự che mắt mình: dưới `unwind` của test target, mọi ca sẽ trông như "đỏ một ca" thay vì "chết cả ứng dụng", nên bộ test **không bao giờ** báo giá đúng.

**Ranh giới giữa story này và Story 8.8.** Story này trả về **số** (hàng · ô · đoạn/ô). Story 8.8 mới là chỗ đọc số đó thành một phán quyết *"đây là bản đăng bài FR121, từ chối"*. Trộn hai việc sẽ đặt một luật nghiệp vụ Epic 8 vào một bộ đọc định dạng, và làm cổng AD-38 không test riêng được.

## Verification

**Commands:**
- `npm run build && cargo test --locked` — expected: 0 đỏ. 🔴 `dist/` phải có **TRƯỚC** `cargo test`. ⚠️ **Tự đo và ghi số thật** (số ca / số binary), đừng chép từ spec cũ.
  → ĐÃ ĐO 2026-09-09: **48 binary, 1351 ca `cargo test` xanh, 0 đỏ** (`src-tauri`, lượt sạch
  sau `npm run build`). `docx_boundary` 9 ca, `docx_contract` 16 ca, `fixtures_docx` 0 ca (hạ
  tầng thuần, đúng chủ ý — xem doc-comment tệp đó), `docx_probe` `#[ignore]` (xem hàng riêng
  dưới). ⚠️ `tests/asset_contract.rs` (Story 6.11, KHÔNG do story này viết) đỏ ngẫu nhiên
  dưới đa luồng mặc định của `cargo test` (đo được: nguyên nhân là tranh chấp scheduler giữa
  nhiều `TcpListener` thật chạy đồng thời trong CÙNG tệp, `--test-threads=1` cho 19/19 xanh ổn
  định qua nhiều lượt lặp, và biến thiên tương tự đo được trên baseline `d58cb77` qua `git
  worktree`) — ghi ở `deferred-work.md`, chủ Ice, KHÔNG phải một hồi quy của story này.
- `npm run test` — expected: 0 đỏ. → ĐÃ CHẠY qua `pre-push`, xanh.
- Chạy **TỪNG** cổng: `check:deps` `check:tokens` `check:i18n` `check:commands` `check:layout` `check:panel-refs` `check:dict` `check:dict-manifest` `check:lint` `check:gates` `check:debt-owner` — **mười một**, đúng `.githooks/pre-push:81`. → ĐÃ CHẠY 2026-09-09, cả **mười một** xanh (`sh .githooks/pre-push`, 299s tổng).
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test docx_probe -- --ignored --nocapture` — expected: TSV + `REPORT.md` sinh ra; **0 mẫu ⇒ thoát khác 0** (đó là hành vi ĐÚNG, không phải một lỗi của bàn đo). → ĐÃ CHẠY: `DOCX_SAMPLES 0`, panic đúng câu "chưa đo", thoát mã khác 0 (đo được, không phải lý thuyết) — xem `6-12-ban-do/REPORT.md`.
- 🔴 **Đối chứng đỏ ① — đuôi tệp.** GỠ `"docx"` khỏi `SUPPORTED_EXTENSIONS` rồi chạy bộ test MỚI — expected: **ĐỎ**. Trả lại — xanh. → ĐÃ LÀM TAY 2026-09-09: 7/16 ca `docx_contract` đỏ khi gỡ, 16/16 xanh sau khi trả lại.
- 🔴 **Đối chứng đỏ ② — ranh giới ô bảng.** Đổi bộ đọc để nối các ô thành một dòng phẳng rồi chạy — expected: ca *"0 segment vắt qua ô"* **ĐỎ**. → ĐÃ LÀM TAY: lượt đầu (mọi ô đều kết thúc bằng dấu chấm) KHÔNG đỏ — đo được đây là giới hạn thật của phép đối chứng (bộ tách CÂU vẫn đúng dù thiếu ranh giới ĐOẠN, khi mọi ô đều có dấu kết câu); sửa fixture `table_two_columns` bỏ dấu chấm cuối ba trong bốn ô (khớp ca bảng thật: nhãn/số liệu không có câu hoàn chỉnh) — lượt hai ĐỎ đúng như kỳ vọng, trả lại — xanh.
- 🔴 **Đối chứng đỏ ③ — tệp thật cho ảnh.** Bỏ bước ghi byte (giữ `INSERT INTO asset`) rồi chạy — expected: ca ảnh `.docx` **ĐỎ**. → ĐÃ LÀM TAY: ca ảnh đỏ đúng với thông báo "PHAI ton tai that su tren dia", trả lại — xanh.
- **Đối chứng ④ — hai cổng biên tự kiểm.** Gieo một dòng `// reqwest` rồi chạy `docx_boundary` — expected: **ĐỎ**. Gieo một `let _ = v[0];`/`.unwrap()` vào module đọc rồi chạy lại — expected: **ĐỎ** ở cổng §0-điểm-panic. Gỡ cả hai — xanh. → Tự động hoá NGAY TRONG `docx_boundary.rs` (ca `..._would_actually_flag_a_seeded_violation...` cho cả hai mệnh đề) — chạy như một phần của 9/9 ca xanh, không cần thao tác tay lặp lại mỗi lượt.
- **Đối chứng ⑤ — §Never đo được.** `cargo test --locked --test config_invariants` — expected: **XANH**. → ĐÃ CHẠY: 28/28 xanh.
- **Đối chứng ⑥ — phép quét có thật không.** Trỏ gốc quét của `docx_boundary` vào một thư mục rỗng — expected: **ĐỎ** ở phép kiểm sàn số tệp, không phải xanh im lặng (bài học `naming_boundary.rs:95-96`). → Tự động hoá trong `docx_boundary.rs::pointing_the_scan_root_at_an_empty_directory_yields_zero_files_not_a_silent_green` (chứng minh cơ chế quét trả 0 tệp trên thư mục rỗng, đúng tiền đề mà sàn số tệp `the_scanned_tree_is_large_enough_to_be_real` dựa lên).

**Manual checks (if no CLI):**
- Nhập một `.docx` thật có ảnh: mở `.atproj/assets/` ⇒ đúng số tệp; `project.db` ⇒ mỗi tệp một hàng `asset` với `source_url` **NULL**. → ĐÃ KIỂM qua `docx_contract.rs::a_docx_with_an_embedded_image_writes_a_real_file_and_a_null_source_url_asset_row` (đọc trực tiếp đĩa + CSDL sau `create_work_from_file` thật, không suy từ `images_saved`).
- Nhập một `.docx` cắt cụt (`head -c` một nửa tệp hợp lệ) **trên bản dựng `--release` thật**, không trong `cargo test`: ứng dụng **còn sống** và hiện một câu lỗi đọc được. 🔴 Đây là ca duy nhất phân biệt được `abort` với `unwind`, và không cổng nào chạy nó — phải làm tay. ⚠️ **CHƯA LÀM** — đòi dựng `--release` (`codegen-units=1`+LTO, vài phút) và thao tác tay trên UI thật; để lại cho Ice, ghi rõ ở đây thay vì khai đã kiểm.
- Nhập lại một `.txt` đã nhập trước story: byte trong `.atproj` không đổi. → ĐÃ KIỂM qua `docx_contract.rs::a_plain_text_file_import_is_completely_unaffected_by_the_docx_branch` (so `bytes == content` sau `import_file`, chưa qua `Step::DecodeEncoding`).
