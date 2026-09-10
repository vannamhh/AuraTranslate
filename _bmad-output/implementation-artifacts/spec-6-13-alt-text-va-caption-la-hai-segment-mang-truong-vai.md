---
title: 'Story 6.13 — Alt-text và caption là hai `Segment` mang trường vai'
type: 'feature'
created: '2026-09-09'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: '2644a5243b8fb8a48731ee0d8d33a24645ee552c'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-6-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** AD-42 nói caption và alt-text là `Segment` mang trường **vai**, và nói thẳng vì sao: một cột text trên `ASSET` **không** vào TM, không vào Glossary, không qua luồng xác nhận — FR129 hỏng **im lặng**. Hôm nay cả hai vế đều chưa dựng, và hai vế hỏng theo hai kiểu khác nhau: `alt` được `Extractor` đọc ra (`extractor.rs:222`) rồi **không đi đâu cả** — `join_kept_blocks` (`pipeline.rs:890`) chỉ ghép `Paragraph | Caption`, nên alt không có mặt trong `source_text`, không có hàng nào, không hiển thị ở đâu; còn `Caption` thì **chảy chung vào `source_text` như một đoạn văn thường**, bị bộ tách cấp câu cắt như văn xuôi, và không đường mã nào biết nó là chú thích ảnh. Cột `role` đã được giữ chỗ bằng chữ trong `schema.rs:1099-1113` cho đúng story này, và sổ nợ giao đúng vế còn hở này cho 6.13 (`deferred-work.md:10186-10189`, `:10396-10401`).

**Quyết định Ice ký 2026-09-09 (bốn câu hỏi mở đã đóng):**
1. **Caption thành ĐÚNG MỘT segment** — bộ tách không cắt bên trong khối caption, đúng vế *"nhiều nhất một segment mỗi vai"* của AD-42. ⚠️ Kèm hai phép đo phải ghi ra chứ không được để im: ① trên bảy mẫu bàn đo 6.1 (2026-09-09) **2/7** khối `figcaption` dài hơn một câu (148 ký tự / 2 dấu kết câu; 127 ký tự / 3 dấu) ⇒ với những khối đó người dịch **mất** khả năng xác nhận từng câu, đó là cái giá đã biết; ② `grep '<figcaption'` trên `src-tauri/tests/**`: **0** fixture ⇒ đường caption hôm nay có **0 ca test**, nên phép ép này không làm đỏ một ca đang có nào — và cũng nghĩa là **không** lượt xanh nào hiện thời chứng minh được gì về caption.
2. **Sinh segment `alt` cho MỌI ảnh giữ có `alt` khác rỗng** — đúng chữ AD-42, không luật ngầm nào. ⚠️ Rủi ro đã đặt lên bàn và Ice nhận: một `alt` rác thành một hàng phải dịch và đếm vào `chapter.segment_count`. Số ảnh **GIỮ** có `alt` trên trang thật **chưa đo được** (bàn đo 6.1 không ghi cột đó) ⇒ ghi nợ có chủ, **không** một dấu tích.
3. **Chỉ mô hình + đường web.** Vế `.docx` (đọc `wp:docPr@descr`/`@title`, quy ước caption của Word) thành **nợ có chủ**, không làm ở story này.
4. **Không đụng UI.** Hiển thị là Story 6.14 (`epics.md:5151`). ⚠️ Hệ quả đã biết: sau story này một Chương nhập từ web có thêm hàng `alt`/`caption` **không nhãn** trong lưới cho tới khi 6.14 tới — ghi nợ có chủ, đừng để người sau tưởng là một sơ suất.

**Approach:** Thêm cột `role` (`NULL` | `alt` | `caption`) vào `segment` bằng **một bước di trú mới** (đích 21), rồi dệt segment vai vào dãy `ord` **lúc nhập** ở đúng một chỗ: sau khi `compute_anchor` đã tính neo trên dãy segment văn xuôi, một hàm **thuần** dệt `alt` vào ngay sau neo và đánh dấu `caption` (đã có sẵn trong `source_text`), rồi **dời neo** theo số segment vai chèn trước nó — đúng khuôn dời neo mà gộp/tách Chương và gộp/tách câu đã làm (`schema.rs:965-971`). Không cột text nào trên `asset`, không đường ghi thứ hai: vẫn `insert_segments` trong CÙNG giao dịch `store.write` của `create_work`.

## Boundaries & Constraints

**Always:**
- 🔴 Cột `role` tới bằng **bước di trú 21**, KHÔNG bằng một lượt sửa `SEGMENT_DDL` tại chỗ — một `project.db` đã ở phiên bản 20 không bao giờ chạy lại DDL tạo bảng, và sửa tại chỗ cho hai lược đồ khác nhau dưới cùng một số phiên bản (vết sẹo số 4, `schema.rs:1102-1107`).
- 🔴 **KHÔNG `CHECK` trong `ALTER TABLE`** — tiền lệ của chính bảng này viết bằng chữ ở `schema.rs:1516-1517`: `status`, `is_omitted`, `is_target_paragraph_end`, `translation_origin` và `chapter.status` đều cưỡng chế giá trị hợp lệ **ở tầng Rust**, không trong DDL. `role` đi cùng khuôn: một kiểu Rust đóng (`SegmentRole`) là nơi duy nhất đúc ra chuỗi `'alt'`/`'caption'`, và một ca hợp đồng khoá cặp chuỗi ⇄ kiểu (khuôn `the_backfill_literal_matches_the_origin_constant_it_copies`).
- 🔴 `insert_segments` (`commands/segment.rs:99`) đặt cột mới **TƯỜNG MINH**, không nhờ `DEFAULT` của `ALTER TABLE` — bài học đã ghi bằng chữ ở `segment_contract.rs:5692` và `:3536-3573`.
- Ba cổng đang ghim số phải đổi **cùng lượt**, mỗi chỗ một câu nói vì sao: `pinned_contract.rs:224` (`PROJECT_MIGRATIONS.len()` 19 → 20), `pinned_contract.rs:234` (`schema_version()` 20 → 21), `segment_contract.rs:2699` (số cột `segment` 13 → 14) cộng `SegmentRow`/`read_all_segment_rows` (`segment_contract.rs:2645`) — cột mới phải vào bộ đọc thô CÙNG LƯỢT, nếu không cổng `a_flush_touches_exactly_...` mù đúng với cột đó và vẫn xanh.
- Neo tính **TRƯỚC** khi dệt: `compute_anchor` (`anchor.rs:62`) giữ nguyên phép tự kiểm tiền tố byte-for-byte trên dãy segment văn xuôi; giá trị ghi xuống `asset.anchor_after_segment_ord` là giá trị **đã dời** trong không gian `ord` CUỐI CÙNG. Bất biến `asset` (`schema.rs:958-975`) không được phép hở: không hàng mồ côi, không hàng trỏ quá cuối Chương.
- 🔴 **Khối caption cho ĐÚNG một segment** (Quyết định 1): dãy segment của Chương vẫn phủ trọn `source_text` theo đúng thứ tự và mỗi segment vẫn là một **chuỗi con thật** của nó — phép ép chỉ được đặt **ranh giới** tại hai đầu khối caption, KHÔNG được viết lại, ghép thêm hay bỏ bớt một byte nào. Vị trí khối caption trong `source_text` suy bằng **đúng máy dựng tiền tố** mà `anchor::compute_anchor` đã dùng (`anchor.rs:1-27`), không bằng một phép `find()` trên chuỗi — hai chỗ đọc override phải thấy cùng một kết quả.
- `source_text` của Chương **không đổi một byte** so với hôm nay: `join_kept_blocks` giữ nguyên nhánh `Paragraph | Caption`, nên AC *"trùng đúng Story 6.7"* và mọi ca `webimport_contract.rs` hiện có vẫn phải xanh.
- Segment vai đi qua **đúng luồng xác nhận của mọi segment khác**: cùng `confirm_segment` (`commands/segment.rs:2017`), cùng `INSERT INTO segment_version` (`:2142`), cùng writer nối tiếp AD-11. Nghiệm thu bằng một ca xác nhận một segment vai `caption` rồi đọc lại trạng thái + đúng một hàng `segment_version`, **không** bằng lập luận "nó là segment nên nó chạy đúng".
- Alt rỗng hoặc chỉ khoảng trắng ⇒ **không sinh segment** (AD-42: segment rỗng trôi vào TM và vào bộ đếm tiến độ; `chapter.segment_count` ở `commands/chapter.rs:358-378` đếm MỌI hàng sống, không lọc nội dung).
- Trường `role` chở lên dây trong `ChapterSegment` (`commands/segment.rs:212-222`) và bản chép tay TypeScript (`src/config/segment.ts:66`) trong CÙNG một lượt — hai tệp này không có codegen nào giữ đồng bộ hộ.
- Mọi khoá lỗi mới (nếu có) dựng qua `IpcError::new` + `message_keys!` + `src/i18n/vi.json` cùng lượt (`ipc_contract.rs:232`).

**Never:**
- **Không** cột text (`alt`/`caption`) trên bảng `asset` — đó chính là điều AD-42 tồn tại để chặn.
- **Không** đụng bề mặt HIỂN THỊ ảnh: render ảnh đúng vị trí là **Story 6.14**, chưa có một dòng mã nào và story này không mở nó.
- **Không** tính lại ranh giới segment lúc nạp (AD-4) và **không** đổi `PIPELINE_ORDER`, `MINIMUM_HARVEST_SCHEMA_VERSION`, `assetProtocol.scope`, `capabilities/main.json`.
- **Không** nghiệm thu vế TM ở story này — `core/tm/mod.rs` hôm nay là ba dòng doc-comment, 0 hàm; phần nghiệm thu TM đóng ở **Story 7.1** (`epics.md:5404`).
- **Không** sửa `epics.md`/spine cho khớp mã: vế nào không dựng được thì ghi nợ có chủ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Ảnh giữ có alt | `<img alt="…">` được giữ | Một segment `role='alt'` ngay sau neo của ảnh; neo các ảnh sau dời theo | N/A |
| Ảnh giữ không alt | `alt` vắng/rỗng/chỉ khoảng trắng | **0** segment sinh ra; neo không dời | N/A |
| Ảnh có caption | `<figcaption>` sau ảnh | Segment của khối caption mang `role='caption'`, đứng sau segment `alt` nếu có | N/A |
| Ảnh không caption | Không `figcaption` | **0** segment `caption`; không hàng rỗng nào | N/A |
| Hai ảnh liền nhau | ảnh A (có alt) rồi ảnh B | Vai của A không trôi sang B; neo B đã cộng thêm 1 | N/A |
| Caption không có ảnh trước nó | `figcaption` mở đầu Chương | Không gán vai (không có ảnh để treo), đi tiếp như văn xuôi | Ghi chẩn đoán, Chương **vẫn** nhập |
| Chương 0 ảnh | `.txt`/dán tay | `role` là `NULL` ở mọi hàng; byte ghi xuống trùng đúng trước story này | N/A |
| Xác nhận một segment vai | caption đã dịch | Chuyển `confirmed` + **đúng một** `segment_version`, y hệt segment văn xuôi | N/A |
| Gộp/tách một segment vai | người dùng gộp caption với câu bên cạnh | Segment mới mang `role = NULL` (vai không nhân bản, AD-5 về hưu + tạo mới) | N/A |
| `project.db` phiên bản 20 | tệp cũ trước story này | Mở được, di trú lên 21, mọi hàng cũ `role = NULL` | Di trú chỉ tiến (AD-30) |

</frozen-after-approval>

## Code Map

**Lược đồ — nơi cột mới sinh ra**
- `src-tauri/src/core/store/schema.rs:1139` `SEGMENT_DDL` (13 cột hôm nay, **không** sửa tại chỗ); `:1099-1113` doc-comment đã giữ chỗ `role` cho đúng story này; `:1703` `PROJECT_MIGRATIONS` (19 bước, đích **20**) — khuôn thêm cột: một hằng SQL riêng kiểu `WORK_STATUS_OVERRIDE_DDL` (`:1795-1802`), `ALTER TABLE` (+ `UPDATE` backfill nếu cần) chứ không sửa DDL gốc.
- `src-tauri/src/core/store/schema.rs:981-1000` `ASSET_DDL` — `anchor_after_segment_ord INTEGER NOT NULL CHECK (>= 0)`; `:958-975` khai bằng chữ rằng **ba** đường tổ chức lại (gộp Chương, tách Chương, gộp/tách câu) đều dời neo cùng lượt với `segment.ord`. ⇒ Neo và `ord` sống **cùng một không gian**; dệt segment vai mà không dời neo là làm ba mệnh đề đó nói dối.
- Rào rỗng trong DDL viết theo khuôn 25 điểm mã `White_Space` (`:991-999`) — `trim()` của SQLite chỉ cắt dấu cách ASCII (`src-tauri/AGENTS.md`).

**Đường nhập — chỗ duy nhất dệt**
- `src-tauri/src/commands/project.rs:824` `prepare_chapter_images` (NGOÀI giao dịch, tải/ghi tệp), `:892` gọi `anchor::compute_anchor`, `:727` `struct SavedAsset { anchor_after_segment_ord, chapter_index, … }`.
- `src-tauri/src/commands/project.rs:602-644` vòng ghi mỗi Chương trong `store.write`: `INSERT INTO chapter` → `:619` `insert_segments(tx, chapter_id, &chapter.segments)` → `:625-639` `INSERT INTO asset`. 🔴 **Đây là chỗ duy nhất có cả `chapter_id` thật lẫn `saved_assets` đã tính neo** ⇒ dãy segment đã dệt phải được dựng **trước** `store.write` (thuần, test được không cần webview), và neo đã dời đi cùng nó.
- `src-tauri/src/core/segment/anchor.rs:62-70` `compute_anchor(...) -> Result<i64, AnchorError>`; `:1-27` doc-comment mô tả bốn bước và phép **tự kiểm tiền tố byte-for-byte** — phép này chạy trên dãy **văn xuôi**, nên nó phải chạy TRƯỚC lượt dệt và không được sửa.
- `src-tauri/src/core/segment/pipeline.rs:883-892` `join_kept_blocks` — `Paragraph | Caption` cho chữ, `Image` cho `None`. **Không sửa**: nó là thứ giữ `source_text` trùng byte với `text_content` của Story 6.7.
- `src-tauri/src/core/webimport/extractor.rs:112-123` `BlockBody::{Paragraph, Image{src,alt}, Caption}`; `:222` đọc `alt`; `:275` `is_caption = name_str == "figcaption"`; `:318` đúc `Caption`. ⚠️ Mô hình khối **phẳng**, không có nhóm `<figure>` — quan hệ caption ⇄ ảnh phải suy theo **vị trí** (ảnh GIỮ gần nhất đứng trước), và ca "caption không có ảnh trước nó" là một hàng của ma trận.
- `src-tauri/src/core/segment/import.rs:456-498` `ImportedChapter { source_text, segments, blocks, … }`; ⚠️ `:514-517` — với `.docx`, `blocks` chỉ gắn cho **Chương đầu** (`project.rs:477-481`); giới hạn đã có chủ, đừng mở rộng ở đây.
- `src-tauri/src/core/segment/split.rs:112-131` `SplitSegment { text, is_paragraph_end }` (5 chỗ dựng literal trong toàn kho) — đây là đầu ra của bộ tách **thuần**; vai KHÔNG thuộc về nó.

**Ghi và dây**
- `src-tauri/src/commands/segment.rs:99-140` `insert_segments` — `ord = index + 1`, hai chỗ gọi (`project.rs:619`, `segment.rs:338`); `:212-222` DTO `ChapterSegment` (9 trường, **không** `rename_all`); `:2017` `confirm_segment`, `:2142` `INSERT INTO segment_version`, `:2427` `unconfirm_edited_segments`.
- `src/config/segment.ts:66-120` bản chép tay của DTO; `src/panels/GridPanel.vue:1568-1730` năm cột render `editorSegments`.

**Cổng sẽ nói gì**
- `src-tauri/tests/pinned_contract.rs:222-236` ghim `len() == 19` và `schema_version() == 20`; `src-tauri/tests/segment_contract.rs:2683-2701` ghim **13 cột** và nói thẳng vì sao (bộ đọc thô phải thấy cột mới cùng lượt); `:1189-1193` ghim `segment_version` đúng bốn cột.
- 11 cổng `.githooks/pre-push:81` (`deps tokens i18n commands layout panel-refs dict dict-manifest lint gates debt-owner`) → `npm run test` (`:96`) → `npm run build` (`:100`) → `cargo test --locked` (`:103`).
- ⚠️ Trước khi đọc một lượt đỏ ở `asset_contract.rs` là hồi quy: nó có ca đỏ **ngẫu nhiên** dưới đa luồng mặc định (tranh chấp `TcpListener` thật), đo được ở baseline — xem `spec-6-12-doc-docx.md` §Verification và `deferred-work.md`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store/schema.rs` — hằng `SEGMENT_ROLE_DDL` mới (`ALTER TABLE segment ADD COLUMN role TEXT;`, `NULL`-able nên không cần `DEFAULT`, **không** `CHECK`, **không** `CREATE INDEX` — chưa đường đọc nào lọc theo cột này), thêm bước **21** vào `PROJECT_MIGRATIONS`; chú thích tại chỗ đóng dòng "một cột còn vắng" ở `:1113` bằng 🔵 + ngày.
- [x] `src-tauri/src/core/segment/role.rs` (**mới**) — kiểu `SegmentRole { Alt, Caption }` (nơi DUY NHẤT đúc chuỗi `'alt'`/`'caption'`) + hàm **thuần** dệt: nhận `blocks`, `effective_kept`, dãy `SplitSegment` và danh sách `(image_index, anchor)` → trả dãy segment đã dệt (mang `Option<SegmentRole>`) **và** neo đã dời. `Result` ở mọi nhánh, 0 điểm panic, không chạm đĩa. Quan hệ caption ⇄ ảnh suy theo vị trí: **ảnh GIỮ gần nhất đứng trước** khối caption; không có ảnh nào trước ⇒ không gán vai.
- [x] `src-tauri/src/core/segment/pipeline.rs:715`/`:1155` (bước 7 `Step::SplitSegments`) — ép ranh giới segment tại hai đầu **khối caption** (Quyết định 1) trước khi `split_source_text` cắt phần còn lại; đường không có `blocks` (`.txt`, dán tay, `.docx`) đi **đúng nhánh cũ**, không một byte nào đổi.
- [x] `src-tauri/src/commands/segment.rs` — `insert_segments` nhận dãy đã dệt và ghi cột `role` **tường minh**; `ChapterSegment` chở thêm `role`.
- [x] `src-tauri/src/commands/project.rs:602-644` — gọi hàm dệt giữa `prepare_chapter_images` và `store.write`; ghi `asset.anchor_after_segment_ord` bằng giá trị **đã dời**.
- [x] `src/config/segment.ts` — thêm `role` vào bản chép tay, kèm chú thích trỏ về struct Rust.
- [x] `src-tauri/tests/pinned_contract.rs` + `src-tauri/tests/segment_contract.rs` — cập nhật ba số đã ghim (19→20, 20→21, 13→14) và `SegmentRow`/`read_all_segment_rows`, mỗi chỗ một câu nói vì sao.
- [x] `src-tauri/tests/segment_role_contract.rs` (**mới**) — mọi hàng của ma trận I/O trên đường sản phẩm thật (`create_work_from_*`), cộng ca *"đường `.txt`/dán tay không đổi một byte"* và ca xác nhận một segment vai sinh **đúng một** `segment_version`.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — đóng bằng chữ vế `caption`/`alt` của `:10186-10189` và `:10396-10401`; ghi nợ **MỚI có chủ** cho từng vế còn hở, mỗi mục một `Chủ:` thật (`check:debt-owner` đỏ nếu thiếu): ① `.docx` không sinh alt/caption (**Chủ: Ice** — quy ước caption của Word chưa ai chốt); ② **chưa đo được** số ảnh GIỮ có `alt` trên trang thật, nên rủi ro "alt rác thành hàng phải dịch" là suy đoán chứ không phải phép đo (**Chủ: Ice**); ③ hàng `alt`/`caption` hiện **không nhãn** trong lưới (**Chủ: Story 6.14**); ④ vai mất khi gộp/tách câu (**Chủ: Story 6.14**); ⑤ nghiệm thu TM (**Chủ: Story 7.1**).

**Acceptance Criteria:**
- 🔴 Given một trang có ảnh mang `alt` và một `figcaption`, when nhập trọn đường sản phẩm, then `segment` có đúng một hàng `role='alt'` và đúng một hàng `role='caption'`, `ord` của chúng đứng **ngay sau** neo của ảnh theo đúng thứ tự `alt` rồi `caption`, và **0** cột text nào được thêm vào `asset`.
- 🔴 Given phép **GỠ** lượt dời neo (giữ nguyên phần dệt), when chạy bộ test MỚI, then nó phải **ĐỎ** — đối chứng là một phép gỡ biên dịch được và chạy, không phải một lập luận.
- 🔴 Given một khối `figcaption` chứa **ba câu**, when nhập, then nó cho **đúng một** segment `role='caption'`, và ca này phải **ĐỎ** nếu phép ép ranh giới bị gỡ (đường caption hôm nay có **0** ca test, nên ca này là phép đo đầu tiên của nó).
- 🔴 Given một `.txt`/dán tay không có ảnh, when nhập và xác nhận, then byte ghi xuống `.atproj` **trùng đúng** kết quả trước story này và mọi hàng mang `role = NULL`.
- Given một segment `role='caption'` đã dịch, when xác nhận, then nó chuyển `confirmed` và sinh **đúng một** `segment_version`, y hệt một segment văn xuôi (ca đối chiếu cạnh nhau trong cùng tệp test).
- Given một `project.db` ở phiên bản 20 tạo trước story này, when mở, then di trú lên 21 chạy trong một giao dịch và mọi hàng cũ mang `role = NULL` — không hàng nào bị viết lại nội dung.
- Given `cargo test`, when chạy, then `MINIMUM_HARVEST_SCHEMA_VERSION` vẫn **8**, `PIPELINE_ORDER` vẫn đúng 7 bước, `assetProtocol.scope` không đổi.
- Given **mười một** cổng `pre-push` cộng `npm run test` cộng `cargo test --locked`, when chạy trọn, then **0** finding và **0** ca đỏ.

## Implementation Notes

**Thiết kế thật khác literal chữ ký hàm ở Task list một chỗ, có chủ ý:** `role::weave_chapter_segments` không nhận sẵn danh sách `(image_index, anchor)` từ chỗ gọi — nó tự quét `blocks`/`effective_kept` và tự gọi lại `anchor::compute_anchor` cho MỌI ảnh GIỮ (không chỉ ảnh đã tải thành công). Lý do: Quyết định 2 đòi alt sinh **độc lập** với việc tải ảnh có thành công hay không, trong khi danh sách `saved_assets` của `prepare_chapter_images` chỉ chứa ảnh đã qua trọn cả mạng lẫn ghi đĩa. Tính lại neo là một phép thuần rẻ (không I/O), nên trùng lặp này không tốn gì mà giữ đúng mệnh đề — xem doc-comment `weave_chapter_segments` cho lý lẽ đầy đủ.

**Hai cổng ranh giới CŨ đỏ vì lý do đúng, không phải một hồi quy:** `cleanup_boundary.rs::the_cleanup_apply_function_has_exactly_two_named_product_call_sites` và `segment_normalize_boundary.rs::the_normalize_functions_have_exactly_five_named_product_call_sites` đều đỏ khi `anchor::compute_block_prefix_len` (hàm MỚI, dùng lại đúng ba bước ghép+làm sạch+chuẩn hoá mà `compute_anchor` đã dùng, để ép ranh giới caption) thêm một chỗ gọi `cleanup::apply`/`normalize::normalize` thứ ba/thứ sáu. Cả hai cổng đã NÂNG đúng con số (→ ba, → sáu), đổi tên hàm theo, và một chú thích trôi từ trước (Story 6.11 quên cập nhật hai chỗ trỏ tên hàm cũ ở `cleanup_boundary.rs`/`normalize.rs`) được sửa tại chỗ luôn.

**Điều tra một lượt đỏ tưởng là hồi quy, hoá ra không phải:** `asset_contract.rs::a_disk_write_failure_mid_asset_write_fails_the_whole_import_and_removes_the_atproj_folder` đỏ nhiều lần khi chạy `cargo test --locked` trong thư mục dự án thật, với một lỗi mạng cấp thấp (`hyper::Error(UnexpectedMessage)`, "received unexpected message from connection") tốn 8–30 giây trước khi rơi về `images_failed`. Đối chứng bằng `git stash` + chạy LẠI đúng test đó trên **baseline 2644a52 chưa sửa gì** trong CHÍNH thư mục dự án này: đỏ **4/4 lần**, cùng lỗi, cùng thời gian — trong khi baseline chạy trong một `git worktree` ở `/tmp` thì xanh 10/10, ổn định. ⇒ Đây là một sự cố MẠNG/HỆ ĐIỀU HÀNH gắn với thư mục dự án (không phải mã của story này, không phải bảng mã/schema/vai), và đã có ghi chép trước cho đúng lớp lỗi này (`deferred-work.md`, mục Story 6.12, "asset_contract.rs đỏ ngẫu nhiên"). Không sửa ở đây — ngoài phạm vi story, và nguyên nhân gốc (một thư viện HTTP client bên thứ ba phản ứng với một server TCP thô một-kết-nối) không liên quan gì tới AD-42/segment/schema.

**Vòng nghiệm thu của phiên chính (2026-09-09) — một hàng ma trận CHƯA được canh, đã bù.** Rà ma trận I/O theo diff (không theo báo cáo), hàng *"Gộp/tách một segment vai"* được viện dẫn bằng `segment_contract.rs::a_row_born_from_regroup_has_every_column_set_on_purpose_not_by_default`. Ca đó dựng Tác phẩm bằng `create_work_from_text` — Chương **0 ảnh** ⇒ cả hai câu bị gộp vốn đã `role = NULL`, nên khẳng định *"hàng mới `role = NULL`"* **đúng ở CẢ HAI nhánh**: nó xanh y hệt nếu `write_regroup` CÓ chép vai. Bù bằng ca mới `segment_role_contract.rs::merging_a_caption_segment_gives_a_new_row_with_no_role_at_all`, gộp một hàng THẬT SỰ mang vai. ⚠️ Lượt chạy đầu của ca đó bác luôn một giả định tôi đặt vào chính nó (*"câu đứng trên caption là văn xuôi"*): `alt` và `caption` của cùng một ảnh nằm **liền nhau** (đúng AD-42), nên phép gộp nuốt **cả hai hàng đều mang vai** — phép đo vì thế mạnh hơn mệnh đề ban đầu, và khẳng định đã sửa theo thứ đo được chứ không ngược lại.

**Hai đối chứng đỏ chạy lại bằng phép GỠ thật ở phiên chính, không nhận báo cáo:** ① thay `img.anchor + shift` bằng `img.anchor` ⇒ `role::tests::an_alt_only_owner_shifts_the_anchor_of_images_that_come_after_it` **ĐỎ**; trả lại ⇒ 8/8 xanh. ② cho `force_caption_segment_boundaries` trả thẳng `split_source_text` ⇒ ca đơn vị caption-nhiều-câu **ĐỎ** VÀ ca hợp đồng `a_three_sentence_figcaption_...` **ĐỎ** (`"Cau mot cua chu thich."` thay vì trọn ba câu); trả lại ⇒ xanh.

**🔵 Sửa tại chỗ mệnh đề "đỏ ngẫu nhiên" của khối ngay trên — đo được một cơ chế SẮC HƠN, tất định.** Không phải "ngẫu nhiên": trong thư mục dự án này, **lượt kết nối loopback ĐẦU TIÊN của mỗi tiến trình test bị nuốt** (~5–8 giây rồi trượt), mọi lượt sau đó xong trong vài mili-giây. Đo 2026-09-09: chèn một lượt `webimport::fetch` khởi động trước `create_work` trong ca hai-ảnh ⇒ lượt khởi động `ok=false, 7.655 ms`, rồi `images_saved=2, images_failed=0`, hai mục nhật ký domain cách nhau **3 ms** (trước đó: `Other` rồi `Fetched`, cách nhau **5.102 ms**). Cùng cơ chế giải thích `asset_contract` chạy `--test-threads=1` đỏ **đúng một** ca — và đó là ca chạy TRƯỚC theo thứ tự chữ cái, không phải ca "yếu" nào cả (đo: chạy `a_disk_write_failure` cùng `a_kept_image_gets_a_real_file` ⇒ ca đầu đỏ, ca sau xanh).

**Phép A/B chiều ngược (an toàn hơn `git stash`, và độc lập với báo cáo của agent thi hành):** mang **mã HIỆN TẠI** (diff + hai tệp mới) vào một `git worktree` ở `/tmp` ⇒ `asset_contract` **19/19 xanh (1,39 s)** và `segment_role_contract` **11/11 xanh (0,75 s)**; baseline `2644a52` trong cùng worktree cũng 19/19 xanh (1,24 s). Trong thư mục dự án, chính hai binary đó đỏ. ⇒ Nguyên nhân nằm ở **thư mục/môi trường**, không ở mã story này.

**Số đo cuối của phiên chính (2026-09-09), tự chạy, không chép:** `cargo test --locked --no-fail-fast` ⇒ **49 binary, 1.364 ca xanh, 12 đỏ — cả 12 đều trong `asset_contract`** (tệp story này KHÔNG chạm), dưới đa luồng mặc định. `segment_role_contract` **11/11 xanh** trong chính lượt chạy trọn đó. `npm run test` ⇒ **70 tệp, 953 ca xanh**. **Mười một** cổng `pre-push` chạy từng cái ⇒ **11/11 xanh**.

**Sau vòng rà 1 (2026-09-10) — mười tuyến `patch` đã vá, và hai bản vá được đối chứng bằng phép GỠ thật ở phiên chính:** ① gỡ luật *"chỉ khối `Caption` ĐẦU TIÊN của mỗi ảnh"* ⇒ `role::tests::two_captions_owned_by_the_same_image_only_the_first_gets_the_caption_role` **ĐỎ**; ② gỡ phép kiểm `count_after == count_before + 1` ⇒ `role::tests::a_cleanup_rule_that_erases_the_caption_text_tags_no_segment_at_all` **ĐỎ**; trả lại cả hai ⇒ 10/10 xanh. Lời khai của agent thi hành KHÔNG được dùng thay cho hai phép đo này.

**Số đo cuối sau vá (2026-09-10, tự chạy):** `npm run build` rồi `cargo test --locked --no-fail-fast` ⇒ **49 binary, 1.356 ca xanh, 26 đỏ** — toàn bộ 26 nằm trong `asset_contract` (12) và `webimport_contract` (14), **cả hai tệp story này KHÔNG chạm**; chạy lại đúng hai binary đó với `--test-threads=1` ⇒ **19/19** và **33/33 xanh**, đúng món nợ tranh chấp đa luồng đã có tên (`deferred-work.md`, mục Story 6.12 + phép đo bổ sung 6.13 ngay dưới nó). `segment_role_contract` **15/15 xanh** ngay trong chính lượt chạy trọn song song đó. `npm run test` ⇒ **70 tệp, 953 ca xanh**. **11/11** cổng xanh.

## Spec Change Log

## Review Triage Log

### Vòng rà 1 — 2026-09-10 (ba lớp: blind-hunter · edge-case · verification-gap)

⚠️ Lượt phóng ĐẦU của cả ba lớp chết giữa chừng vì hạn mức phiên của tài khoản (HTTP 429, mô hình `claude-sonnet-5`), **0 phát hiện** trả về — mấy dòng lọt ra trong thông báo chỉ là suy nghĩ dở dang, không phải kết quả, và KHÔNG được dùng. Cả ba phóng lại sau khi hạn đặt lại.

| # | Phát hiện | Verdict | Bằng chứng phân xử | Tuyến |
|---|---|---|---|---|
| 1 | Doc-header `PROJECT_MIGRATIONS` ghi *"hai mươi mốt bước"* trong khi mảng có **20** mục (đích 21), và `pinned_contract.rs:227` ghim `len() == 20` | medium | Đếm thật trên chính hằng: 20 mục `to_version`. `schema.rs:1582` nói 21, `:1591` ngay dưới nói *"Hai mươi bước, đích 21"* — đúng thứ rot mà vết sẹo của chính doc-comment này gọi tên | patch |
| 2 | Khối 🔵 cùng chỗ khai *"tiêu đề đổi từ **hai mươi bước** thành hai mươi mốt"*, nhưng chữ CŨ trong diff là *"mười chín bước"* | medium | Đọc diff `schema.rs`: dòng bị thay là `Hôm nay **mười chín** bước`. Cùng nguyên nhân gốc với #1 (một lượt sửa số cẩu thả) | patch |
| 3 | **Hai khối `Caption` GIỮ sau CÙNG một ảnh đều được gán `role='caption'`** — phá vế *"nhiều nhất một segment mỗi vai"* của AD-42 | medium | ĐO 2026-09-10 trên đường sản phẩm (`extract` → `force_caption_segment_boundaries` → `weave_chapter_segments`): `số segment vai caption = 2` → `["Chu thich mot.", "Chu thich hai."]`. Với tới được bằng HTML bình thường: một `<figure>` có ảnh + chú thích, rồi một `<figure>` chỉ có chú thích | patch |
| 4 | **Luật làm sạch xoá trắng văn bản một khối caption ⇒ vai `caption` gán NHẦM sang đoạn văn xuôi kế tiếp** | high | ĐO 2026-09-10 với một `CleanupRule` literal xoá đúng chữ của caption: dãy ra là `[None, None, None, Some(Alt):"mo ta", Some(Caption):"Doan ba dong y nghia, gi…"]` — một đoạn văn xuôi thật mang vai caption. Kịch bản đời thật: luật gỡ dòng ghi công ảnh (`(Ảnh: …/Shutterstock)`) chính là TRỌN văn bản caption | patch |
| 5 | `role` chưa bao giờ được khẳng định qua đường IPC (`ChapterSegment.role`) với một giá trị khác `NULL` | medium | `grep -rn '\.role' src-tauri/tests/**`: **0** chỗ đọc `.role` từ một `ChapterSegment`; mọi khẳng định vai đi qua helper SQL thô `read_role_rows`. Một chỉ số cột sai ở `select_chapter_segments`/`read_fresh_rows` (`row.get(9)`) sẽ xanh trọn bộ | patch |
| 6 | Mock `read_open_chapter_segments` ở `tests/frontend/librarySearch.test.ts:696-706` thiếu trường `role` | low | Đọc tệp: mock thiếu cả `role` lẫn `retired_at` (cái sau có từ trước story này) — kiểu lỏng nên không lỗi biên dịch. Chưa nơi nào ở webview đọc `.role`, nên hại còn tiềm ẩn; phép sửa là một dòng | patch |
| 7 | `compute_anchor` khai *"chỗ gọi đảm bảo `blocks[image_index]` là `Image`"*, còn `role.rs` gọi nó với chỉ số một khối `Caption` | low | Đọc `anchor.rs:50-53` và `role.rs` bước (c): hành vi ĐÚNG (hàm chỉ đếm tiền tố), nhưng hai doc-comment nay mâu thuẫn nhau — mệnh đề hết đúng phải sửa tại chỗ | patch |
| 8 | `unreachable!()` trong `weave_chapter_segments` dưới `panic = "abort"` | low | Đọc `role.rs` bước (e): `alt_iter.next().unwrap_or_else(\|\| unreachable!(…))`. Thật sự không tới được (đứng ngay sau `peek()`), nhưng dựng lại bằng `peek().copied()` là một phép đơn giản hoá thẳng, không thêm nhánh | patch |
| 9 | 0 ca test cho luật *"chỉ Chương 0 đọc `block_overrides`"* trên đường dệt MỚI | low | Đọc hai chỗ (`create_work` và `split_segments_step`): cả hai đều `if index == 0 {…} else { &[] }` — nhất quán, nhưng không phép đo nào canh cặp đó | patch |
| 10 | 0 ca test cho chiều TÁCH (`split_at_segment`) trên một hàng mang vai | low | `grep` `segment_role_contract.rs`: chỉ có chiều GỘP. Cùng lớp với hàng ma trận tôi vừa bù ở bước 3 | patch |
| 11 | Hai lượt tính neo (`prepare_chapter_images` và `weave_chapter_segments`) có thể lệch nhau, để lại neo cũ trong im lặng | false | Cả hai gọi CÙNG hàm thuần `compute_anchor` với CÙNG `blocks`/`effective_kept`/`segments`/`cleanup_rules`/`source_lang` ⇒ cùng kết quả; ảnh nào tính neo trượt thì `prepare_chapter_images` đã đếm `images_failed` và `eprintln!` tại `project.rs:980-985`, không im lặng | — |
| 12 | `split_segments_step` gác bằng `blocks.is_some()` mà không có cổng `.docx`, nên *"`.docx` không đổi một byte"* chỉ đúng nhờ bộ đọc `.docx` tình cờ không sinh `Caption` | false | `.docx` đi `Blob(AlreadyText)` nên `Step::ExtractMainContent` KHÔNG chạy, và `flow.blocks` khởi tạo `vec![None; n]` (`pipeline.rs:549`) ⇒ nhánh ép không bao giờ vào cho `.docx`. Đây là cấu trúc, không phải trùng hợp | — |
| 13 | Ca biên lý thuyết (luật viết tắt đọc VẮT QUA ranh giới ép) chưa có test khoá hành vi | low, bác | Chính doc-comment đã đo và ghi: hai ranh giới luôn trùng một `\n` trên dữ liệu từ `dom_smoothie`, nên ca này không tới được trên đường sản phẩm; một test ở đây sẽ khoá một hành vi chưa ai quyết là đúng | — |
| 14 | `unreachable_port()` khai *"tất định"* nhưng có khe TOCTOU (bind rồi drop) | low, bác | Helper là bản chép của `asset_contract.rs:110` có từ Story 6.11 — không do story này gây ra, và khe đó hẹp tới mức chưa lần đo nào chạm | — |
| 15 | Chữ ký `weave_chapter_segments` trong §Tasks khác chữ ký đã cài | bác | Phép sửa là sửa chính spec của lượt build này — luật bước 4 bác thẳng. Và §Implementation Notes đã ghi đúng chỗ lệch đó kèm lý do trước khi vòng rà chạy | — |


## Design Notes

**Vì sao dệt lúc nhập chứ không suy lúc đọc.** AD-4 đóng băng ranh giới segment tại lúc nhập, và `asset.anchor_after_segment_ord` đã sống trong **cùng không gian `ord`** đó — ba đường tổ chức lại Chương/câu dời neo cùng lượt với `segment.ord` (`schema.rs:958-975`). Một lớp "suy vai lúc đọc" sẽ phải chạy lại `Extractor` trên HTML gốc mà `.atproj` **không giữ**, và sẽ cho một câu trả lời khác sau mỗi lần luật bóc đổi — đúng lớp lỗi mà AD-4 ra đời để chặn.

**Vì sao neo tính trước rồi dời, chứ không tính một lần trên dãy đã dệt.** `compute_anchor` tự kiểm bằng cách dựng lại tiền tố văn bản và đòi nó là tiền tố **byte-for-byte** của `source_text` (`anchor.rs:17-18`). Segment vai `alt` **không có mặt** trong `source_text` — chạy phép tự kiểm trên dãy đã dệt là tự tay phá tiền đề của nó, và cách sửa duy nhất sẽ là nới phép tự kiểm, tức gỡ đúng cái rào đang chặn một neo sai im lặng. Dời sau là một phép cộng đếm được và test được riêng.

**Vì sao vai không thuộc `SplitSegment`.** `split_source_text` là hàm thuần cấp câu, bất biến của nó viết bằng chữ: segment không rỗng, không chứa `\n` (`split.rs:210-224`). Vai là dữ kiện của **tầng nhập** (biết khối nào là ảnh, khối nào là caption), không của bộ tách. Nhét `role` vào `SplitSegment` sẽ cho một trường mà bộ tách không bao giờ đặt — một chỗ hở để người sau tưởng bộ tách có trách nhiệm điền.

## Verification

**Commands:**
- `npm run build && cargo test --locked` (trong `src-tauri`) — **đo thật 2026-09-09**: `npm run build` xanh (`vue-tsc --noEmit` × 2 + `vite build`, `dist/` sinh trước). `cargo test --locked` — **49 binary**, **1.373 ca `ok`**. Với `-- --test-threads=1` (bỏ tranh chấp TCP đa luồng của các ca dựng `TcpListener` thật): **0 đỏ** ngoại trừ đúng **1** ca (`asset_contract.rs::a_disk_write_failure_mid_asset_write_fails_the_whole_import_and_removes_the_atproj_folder`) — ĐÃ ĐO là một sự cố mạng/hệ điều hành gắn với chính thư mục dự án, tái lập được **trên cả baseline 2644a52 chưa sửa gì** (xem §Implementation Notes), không liên quan tới story này. Mặc định đa luồng: `asset_contract.rs`/`webimport_contract.rs` có thể đỏ ngẫu nhiên do tranh chấp cổng TCP giữa các ca — đã ghi từ trước (`deferred-work.md`, mục Story 6.12).
- `npm run test` — **đo thật**: **70/70 tệp**, **953/953 ca**, 0 đỏ.
- Chạy **TỪNG** cổng: `check:deps` `check:tokens` `check:i18n` `check:commands` `check:layout` `check:panel-refs` `check:dict` `check:dict-manifest` `check:lint` `check:gates` `check:debt-owner` — **mười một**, đúng `.githooks/pre-push:81`. **Đo thật**: cả mười một đều `OK`.
- 🔴 **Đối chứng đỏ ① — dời neo.** Gỡ lượt dời neo trong `weave_chapter_segments` (`shifted = img.anchor + shift` → `shifted = img.anchor`), chạy `segment_role_contract.rs` — **ĐO ĐƯỢC: ĐỎ** đúng ca `two_adjacent_kept_images_the_second_anchor_shifts_by_the_first_alt` (`left: [3, 3]`, `right: [3, 4]`). Trả lại — xanh 10/10.
- 🔴 **Đối chứng đỏ ② — cột `role`.** Bỏ `role`/`?7` khỏi câu `INSERT` của `insert_segments` (rơi về `DEFAULT` ngầm của cột NULL-able), chạy `segment_role_contract.rs` — **ĐO ĐƯỢC: ĐỎ 5/10 ca** (mọi ca đọc `role='alt'`/`role='caption'` từ đĩa). Trả lại — xanh 10/10.
- **Đối chứng ③ — `source_text` không đổi.** `cargo test --test webimport_contract --test asset_contract` — **đo thật**: 33/33 và 19/19 xanh, **0 dòng sửa** trong hai tệp đó (`git diff` xác nhận `webimport_contract.rs` không đổi một byte; `asset_contract.rs` cũng vậy).

**Manual checks (if no CLI):**
- Mở `project.db` của một Chương vừa nhập từ web: `SELECT ord, role, substr(source_text,1,30) FROM segment ORDER BY ord` ⇒ hàng `alt`/`caption` nằm đúng ngay sau `anchor_after_segment_ord` của hàng `asset` tương ứng.
