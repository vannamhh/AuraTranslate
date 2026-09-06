---
title: 'Story 6.6: Tách Chương theo mẫu phân tách'
type: 'feature'
created: '2026-09-05'
status: 'done'
baseline_commit: 'b69a345bcf00a9352fcad5495c6d671251c31f9f'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Một file 40 MB chứa cả bộ truyện hôm nay vào Library thành ĐÚNG MỘT Chương khổng lồ. Bước 5 `Step::SplitChapters` có thân nhưng chết trên đường sản phẩm (`commands/project.rs:308-311` và `:1199-1200` viết cứng `chapter_pattern: None`), và thân ấy tách bằng chuỗi con literal qua `str::split` (`pipeline.rs:670`) — phép này **nuốt mất dòng tiêu đề**, tức cấu hình một mẫu TIÊU ĐỀ rồi không Chương nào còn tiêu đề. Người dịch phát hiện 14 Chương sai sau khi đã dịch 200 Chương.

**Approach:** Cho người dùng cấu hình mẫu phân tách (literal HOẶC regex) ngay trên màn xem trước nhập; nâng bước 5 để tách theo **vị trí khớp** (giữ tiêu đề ở đầu Chương mới) thay vì `str::split`; cho `PipelineOutput` chở N Chương kèm `title` ra tới màn xem trước, nơi hiện số Chương nhận ra, ba Chương đầu và ba Chương cuối, tiêu đề và **độ dài** từng Chương, sắp xếp được theo độ dài để chỗ bắt nhầm tự lộ ra; xác nhận thì ghi N Chương vào `project.db` ở `LifecycleStatus::NotStarted`.

## Boundaries & Constraints

**Always:**
- Mẫu phân tách là **tham số MỖI LƯỢT NHẬP**, đi qua `PipelineInput` như `encoding` và `cleanup_rules` — KHÔNG một bảng, KHÔNG một `ScopeKind`, KHÔNG một bước di trú.
- `PIPELINE_ORDER` bảy bước và chỗ gọi `run_import` **duy nhất** không đổi — story thêm thân, không thêm bước, không thêm seam. Chỗ gọi mới đi qua `run_pipeline` (`commands/project.rs:233`).
- Tách theo **vị trí khớp**: Chương thứ *i* là dải `[start_i, start_{i+1})`, nên dòng tiêu đề nằm ở **đầu** Chương mới và `title` lấy từ chính dòng ấy.
- Văn bản TRƯỚC khớp đầu tiên (lời tựa, mục lục) **KHÔNG BAO GIỜ bị vứt**: nó thành Chương `ord = 1` với `title = None` và **hiện ra** trong màn xem trước. Mất byte im lặng bị cấm.
- Nhánh `Unit::Undecoded` của bước 5 **ở lại literal-byte thuần**. Nó tồn tại để tái lập triệu chứng AD-39 (mẫu UTF-8 tìm trong byte GBK ⇒ 0 khớp ⇒ một Chương, không lỗi nào ném); một regex engine trên `&[u8]` phá đúng cặp đối chứng ấy. Mẫu `kind = Regex` gặp nhánh này ⇒ 0 khớp, cùng ngữ nghĩa.
- Regex biên dịch với **`multi_line(true)`** — `^第.*章` vô nghĩa nếu không có nó, và thiếu cờ này thì mẫu khớp 0 lần **trong im lặng**. Cùng cờ mà `core/cleanup/mod.rs:230-232` đã khai.
- Mẫu hỏng, mẫu rỗng, mẫu khớp **độ dài 0** đều trả `Err`/bị lọc ở NGUỒN — không `panic!` (`panic = "abort"`), không số đếm phồng, không Chương rỗng hàng loạt.
- Kết quả tách đi kèm **payload xem trước của từng ứng viên bảng mã**, như `normalized`/`cleanup` đã làm. 🔴 Ba ca "đổi ứng viên ⇒ 0 lời gọi IPC" (`importPreviewEncoding.test.ts:250`, `importPreviewNormalized.test.ts:106`, `importPreviewCleanup.test.ts:140`) giữ nguyên, **không sửa kỳ vọng**.
- Không byte nào xuống đĩa trước khi người dùng xác nhận.

**Ask First:**
- **Nhớ mẫu vừa dùng giữa hai lượt nhập.** Không AC nào đòi, và mặc định của story này là KHÔNG nhớ. Nếu lúc thi hành thấy cần, đó là một quyết định phải hỏi — và chỉ tầng Toàn cục (`app_config`) mới hợp lệ; tầng Tác phẩm là bẫy vòng rà 1 của Story 6.5: màn nhập đang TẠO một Tác phẩm chưa tồn tại.
- Bất kỳ lúc nào thấy cần **một hằng số ngưỡng** để đánh giá một Chương là bất thường — xem §Never, Ice đã chốt 2026-09-05 là không có ngưỡng nào.

**Never:**
- 🔴 KHÔNG gán cờ *"đáng ngờ"*, KHÔNG một ngưỡng nào, KHÔNG nút *"chỉ hiện N dòng đáng ngờ"* (Ice chốt 2026-09-05). Màn hình hiện **độ dài từng Chương** và **sắp xếp được theo độ dài**; người dùng tự phán xử. Mọi hằng số ở đây là một con số phù thuỷ chưa đo — đúng thứ Story 6.4 đã loại có lý do. Vế gán cờ + lọc là nợ có chủ **Story 6.10**, story vốn đã sở hữu bộ lọc *"cần xem"* và nhận 6.6 làm một trong bốn nguồn tín hiệu.
- 🔴 KHÔNG mở rộng `spawn_import_scan` (Ice chốt 2026-09-05) — quét N Chương chạm luồng nền, bộ đếm thế hệ `ImportScanGeneration` và một chi phí ở quy mô 2.000 Chương mà chưa ai đo. Nợ có chủ.
- KHÔNG nhập nhiều tệp cùng lúc (AC7 của `epics.md`) — đã tách, nợ có chủ Story 6.6b.
- KHÔNG sửa `epics.md`/`prd.md` cho khớp mã.
- KHÔNG đăng ký `⌥W`/`⌥←`/`⌥→` (đo: `Alt+` trần chưa ai chiếm, nhưng chúng thuộc tầng 2 và Story 6.10 — một hợp âm không có đối tượng là một đường chết).
- KHÔNG `box-shadow`/`gradient` để dựng danh sách Chương (`check:tokens` Kiểm F không có miễn trừ).
- KHÔNG `@click="fn()"` — mọi `@click` là đúng một `dispatch('<id>')`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Mẫu literal khớp N chỗ | Văn bản có `第一章`×3 | 3 Chương, mỗi `source_text` **bắt đầu bằng** dòng tiêu đề, `title` = dòng ấy | N/A |
| Mẫu regex theo dòng | `^Chương\s+\d+.*$`, `multi_line` | Khớp từng dòng tiêu đề, không khớp giữa câu | N/A |
| Có lời tựa trước khớp đầu | 800 chữ rồi `第一章` | N+1 Chương; Chương 1 `title = None`, hiện trong xem trước | N/A |
| Mẫu không khớp chỗ nào | Mẫu sai bảng mã / sai chữ | ĐÚNG MỘT Chương, `title = None`, xem trước nói *"nhận ra 1 Chương"* | Không lỗi — đây là AD-39, không phải sự cố |
| Mẫu rỗng | `""` | No-op, một Chương (giữ hành vi `pipeline.rs:667`) | N/A |
| Regex không biên dịch được | `[unclosed` | Từ chối, xem trước giữ kết quả CŨ, hiện thông báo | `ImportError::InvalidChapterPattern` → `MessageKey` mới |
| Regex khớp độ dài 0 | `x*` | Lọc ở nguồn, 0 chỗ khớp ⇒ một Chương | Không tách thành N ký tự |
| Mẫu chạy trên byte chưa giải mã | Thứ tự bước SAI, byte GBK | Một Chương, KHÔNG lỗi nào ném | Đối chứng AD-39, giữ nguyên |
| Hình dạng `Chapters` đã chia sẵn | `PipelineShape::Chapters(3)` + mẫu | Mẫu **bị bỏ qua**, ra đúng 3 | Rẽ theo `Flow::already_chaptered`, không theo `units.len()` |
| Xác nhận N Chương | N = 2000 | 2000 hàng `chapter`, `ord` 1..N liên tục, mọi hàng `status = not_started`, segment cùng giao dịch | Một giao dịch; `meta` lỗi ⇒ cuộn lại trọn vẹn |
| Sửa mẫu | Người dùng gõ tiếp | Xem trước chạy lại tự động (một vòng IPC), giữ ứng viên bảng mã đang chọn | Trượt ⇒ giữ kết quả cũ + lỗi hiện ra |

</frozen-after-approval>

## Code Map

**Rust — điểm tiêm và chỗ nối**
- `src-tauri/src/core/segment/pipeline.rs:609-649` `split_chapters_step` — **ĐIỂM TIÊM DUY NHẤT**; ba cửa thoát sớm (`already_chaptered` `:610` · `pattern == None` `:613` · `units` rỗng `:622`) giữ nguyên. `:638` gán `units`, `:641`/`:647` reset `segments`/`cleanup_reports` theo N mới. `:666-672` `split_on_literal` (🔴 `text.split` **nuốt** dấu phân tách — đây là khuyết tật phải sửa) · `:685-705` `split_on_literal_bytes` (**giữ nguyên**, nhánh AD-39). `:107-115` `PIPELINE_ORDER` · `:122-141` `validate_order` (chỉ nhận **hoán vị đủ bảy bước**) · `:449` chỗ gọi bước 5 · `:286-293` `PipelineOutput`.
- `pipeline.rs:207-210` `chapter_pattern: Option<String>` — **kiểu phải đổi** thành `Option<ChapterPattern>`; `:231`/`:252` hai constructor viết cứng `None`; `:262-265` `with_cleanup_rules` là **khuôn builder `#[must_use]`** để chép cho `with_chapter_pattern`.
- `src-tauri/src/core/segment/import.rs:308-324` `ImportedChapter` — **không có `title`**, phải thêm `title: Option<String>`; `:157`/`:167` hai biến thể `ImportError` liền kề là tiền lệ cho biến thể mới.
- `src-tauri/src/core/cleanup/mod.rs:230-232` `compile_cleanup_regex` — `RegexBuilder::new(p).multi_line(true).build()`, **khuôn phải chép nguyên cờ**; `:239-266` `byte_ranges_for` (`match_indices` cho literal, `find_iter` cho regex); `:259-263` lọc khớp **độ dài 0** ở NGUỒN; `:250-251` mẫu hỏng ⇒ `Err`, không `panic!`. `store.rs:144` `validate_pattern` — cùng hàm biên dịch, khác lớp.
- `src-tauri/src/commands/project.rs:268-457` `create_work` — 🔴 **vòng lặp `for (i, chapter) in chapters.iter().enumerate()` `:369-387` ĐÃ tổng quát cho N**; `:370` `ord = i + 1`; `:372` chèn `title` là **`NULL` cứng** (đây là chỗ bơm `title` vào); `:375` `LifecycleStatus::NotStarted.as_str()`; `:381` `last_insert_rowid()`; `:382` `insert_segments`; `:328`/`:336` hai rào NGOÀI closure; `:346` một `store.write` = một giao dịch; `:403` `rebuild_from_store`; `:428` `write_atomic`.
- `project.rs:308-311` và `:1199-1200` — hai chỗ viết cứng `chapter_pattern: None`, cả hai phải nhận tham số thật. `:1203` `outcome.chapters.into_iter().next()` — 🔴 **chỗ hỏng đầu tiên khi N > 1**. `:736-741` `spawn_import_scan` nhận MỘT `chapter_id`. `:359-361` `OpenWork.chapter_id` chốt Chương đầu (`open_work` `:1704` `ORDER BY ord, id LIMIT 1`). `:1357-1457` `preview_import_encoding`; `:1536-1575` `confirm_import_with_encoding` (giữ `MutexGuard` xuyên suốt `:1565-1572`); `:937-943` `PendingImportSourceState`; `:1227-1296` `build_cleanup_preview_wire` (khuôn cắt cửa sổ + span vắt biên `:1255-1265`); `:2630-2661` `resolve_cleanup_rules` (khuôn phân giải ở `mod wire`); `:1162-1178` số đo **6 lượt `run_pipeline`, ~13-17 ms/lượt trên 440 KB** — mốc so sánh bắt buộc đo lại.
- `src-tauri/src/core/lifecycle/mod.rs:83-96` `lifecycle_statuses!` — `NotStarted => "not_started"`; `:106-117` `derive_work_status` (mọi Chương `NotStarted` ⇒ Tác phẩm `NotStarted`, đúng sẵn).
- `src-tauri/src/core/store/schema.rs:896-904` `CHAPTER_DDL` — `title TEXT` **nullable, đã có sẵn** ⇒ 🔴 **KHÔNG cần bước di trú**.
- `src-tauri/src/core/i18n/` `message_keys!` — danh mục ĐÓNG; một biến thể quên thêm vào `ALL` cho một test **xanh giả**.

**Cổng — cái nào đỏ, cái nào mù**
- `src-tauri/tests/segment_pipeline_boundary.rs:146-164` khoá `PIPELINE_ORDER` từng bước · `:170-213` đếm `run_import` đúng 1 (`:200`) + `run_import_with_order` phải rỗng ngoài `core/segment/` (`:192`) · `:42` sàn 50 (thật 67, cận DƯỚI). ⚠️ **`:182` quét `code_lines` TRẦN, chưa cắt `#[cfg(test)]`** — một khối test trong `src/**` gọi `run_import(` làm cổng **đỏ OAN**; đây đúng lớp lỗi `cleanup_boundary.rs:318-324` đã vá.
- `src-tauri/tests/cleanup_boundary.rs:128-150`/`:307-346` — **khuôn cổng mới tốt nhất**: cả hai assert THẬT đều GỌI `text_before_first_cfg_test_line`; `:171-183`/`:280-301`/`:353-365` khuôn kiểm chứng dương **ca dương + ca âm**; `:80-89` loại đủ năm tiền tố chú thích.
- `src-tauri/tests/segment_normalize_boundary.rs:98-109` `text_before_first_cfg_test_line` (neo theo **đầu dòng**, không `find` chuỗi trần) · `:231-239`/`:244-247` hai ca tự-kiểm phép neo — **chép cả hàm lẫn hai ca**.
- `src-tauri/tests/segment_contract.rs:7999-8246` — **sáu ca đã khai `chapter_pattern: Some(..)`**, gồm hai đối chứng AD-39 (`:8020` byte GBK thứ tự sai ⇒ 1 Chương; `:8066` thứ tự đúng ⇒ 3 Chương) và hai ca hình dạng `Chapters` (`:8153`, `:8184`). Đổi kiểu `chapter_pattern` ⇒ cả sáu sửa **cơ học**, giữ nguyên mệnh đề. `:1141-1145` `schema_version() == 19` · `:1988` `STEP_TWENTY` nâng tay (**không chạm nếu không thêm di trú**).
- `src-tauri/tests/project_contract.rs:604-661` `create_work_writes_every_chapter_and_its_segments_when_the_pipeline_yields_more_than_one` — 🔴 **ca N > 1 ĐÃ TỒN TẠI**, nhưng dựng bằng `PipelineShape::Chapters` viết tay, chưa đi qua mẫu phân tách. Đây là ca phải mở rộng, không dựng lại.
- `src-tauri/tests/ipc_contract.rs:876-878` — 🔴 khoá **nguyên văn** danh sách tham số `confirm_import_with_encoding`; thêm `chapter_pattern` là sửa đúng dòng đó kèm lý do. `:232-264` mọi `MessageKey` phải có trong `vi.json`; `:325-374` khai đủ `params`.
- `src-tauri/tests/cleanup_contract.rs:459` `preview_and_confirm_agree_byte_for_byte…` — khuôn trực tiếp cho mệnh đề "xem trước = xác nhận" ở quy mô N Chương; `:38-49` `temp_dir`, `:62-73` `open_work_real`, `:75-89` `read_source_text`.

**Frontend — dây và bề mặt**
- `src/config/project.ts:194-202` `EncodingCandidateWire` (thêm khối tách Chương vào **mỗi** ứng viên) · `:205-215` `ImportEncodingPreview` (thêm trường tự khai) · `:227-321` khuôn **kiểm kiểu lúc chạy** (mảng phải `.every(...)` `:314`; nullable phải viết `(v.x === null || is…(v.x))` để `undefined` KHÔNG lọt `:292-299`) · `:223-225`/`:394-397` bảy hằng tên lệnh · `:323-345` `callPreviewImportEncoding` (ba nhánh `catch`) · `:384-391` luật *"thêm adapter KÈM chỗ gọi trong CÙNG một lượt"*.
- `src/importPreviewState.ts:109-110` `pendingText`/`pendingPath` · `:355-391` `reloadImportPreviewAfterRuleChange` — 🔴 **cơ chế "cập nhật ngay" ĐÃ CÓ**: `sequence` chống đua (`:364`, `:372`), giữ ứng viên đang chọn (`:386-388`), không đụng `lastSubmittedFrom`. Sửa mẫu tái dụng đúng đường này. `:211-233` hai bản khuôn "đọc ứng viên khi có, rơi về tự khai khi không" · `:243-246` `importPreviewEmptyReasonForTier` (chữ ký đã hẹp còn literal `2`) · `:583-606` `resetImportPreview()` quét **21 ô** (mọi ô mới phải vào đây) · `:401-492` khuôn CRUD (cờ riêng, `finally` hạ cờ, chỉ tải lại khi `error === null`).
- `src/ImportPreviewOverlay.vue:352` `role="dialog" aria-modal="true"` + `:297-323` bẫy Tab (danh sách selector cố định — **thêm `<input>` mẫu phải nằm trong đó**) · `:432-468` khuôn một tầng CÓ thân (bốn phần) · `:471-476` tầng 2 rỗng · `:129-168` `CleanupTextPiece`/`cleanupPiecesOf` (điểm mã, `[start,end)`) · `:488-495` template `<span>` **viết liền không khoảng trắng** · `:63-71` luật khoá i18n phải là **literal**, an toàn đến từ `switch` **cạn** · `:507-518` vì sao thao tác mang tham số đi qua `@change`/`@submit.prevent`, không `@click`.
- `src/commands/index.ts:1063-1098` ba lệnh xem trước (`E`, `Mod+Alt+Enter`, không phím) · `:1077-1080` lý lẽ hợp âm duy nhất **toàn cục** · `src/commands/keys.ts:470-491` `claimed` ném khi trùng.
- `src/i18n/vi.json:211-251` khối `mode.library.preview.*` (41 khoá) · `:194-196` `command.import.preview.*`.
- `tests/frontend/importPreviewCleanup.test.ts:43-58` `freshState()`/`freshOverlay()` — 🔴 **nạp ĐỘNG cả hai trong CÙNG lượt**, `freshState()` TRƯỚC `mockResolvedValue`; `importPreviewEncodingWireShape.test.ts` mock `@tauri-apps/api/core`, **không** mock `src/config/project`.

**Nợ liên quan** — `deferred-work.md:9414-9428` (🔴 **chủ story này**: tiêu đề bị luật gộp dòng 6.4 nối oan; 6.6 đóng vế tiêu đề đứng ĐẦU Chương, vế tiêu đề phụ TRONG thân vẫn mở) · `:9535-9552` (🔴 **chủ story này**: `count_in_import` luôn bằng `count_in_chapter` vì chưa có `Chapters(N>1)` thật) · mục mới cuối sổ (AC7 → Story 6.6b).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/segment/chapterpattern.rs` -- tạo mới, **THUẦN** (không `Store`, giữ `segment_boundary.rs::the_splitter_stays_pure`): `ChapterPattern { kind: Literal|Regex, pattern: String }`, `compile` khai **`multi_line(true)`** đúng như `cleanup/mod.rs:230-232`, `match_starts(text) -> Result<Vec<usize>, _>` lọc khớp **độ dài 0** ở nguồn. Mẫu hỏng trả `Err`, không `panic!`
- [x] `src-tauri/src/core/segment/pipeline.rs` -- đổi `chapter_pattern` sang `Option<ChapterPattern>` + builder `with_chapter_pattern` (khuôn `:262-265`); `split_chapters_step` nhánh `Unit::Decoded` tách theo **VỊ TRÍ** `[start_i, start_{i+1})` (thay `split_on_literal`), giữ phần trước khớp đầu làm Chương riêng, gán `title` từ dòng khớp; nhánh `Unit::Undecoded` **giữ nguyên literal-byte** -- một regex trên `&[u8]` phá cặp đối chứng AD-39
- [x] `src-tauri/src/core/segment/import.rs` -- `ImportedChapter.title: Option<String>`; biến thể `ImportError::InvalidChapterPattern { detail }` theo tiền lệ `:167`
- [x] `src-tauri/src/core/i18n/` -- `MessageKey` mới cho *mẫu phân tách hỏng* trong `message_keys!` -- danh mục ĐÓNG, đừng viết danh sách song song
- [x] `src-tauri/src/commands/project.rs` -- `create_work` nhận `chapter_pattern`, bơm `title` vào chỗ `NULL` cứng `:372`; `cleanup_preview_for`/đường xem trước đọc **TOÀN BỘ** `outcome.chapters` thay `.next()` `:1203`; dựng khối tách Chương (số Chương · ba đầu · ba cuối · `title` · **độ dài mỗi Chương tính bằng ĐIỂM MÃ**) cho **mỗi** ứng viên và đường tự khai; `confirm_import_with_encoding` truyền **cùng** mẫu -- xem trước và xác nhận phải trùng từng byte. 🔴 KHÔNG một ngưỡng, KHÔNG một cờ "đáng ngờ" nào trên dây
- [x] `src-tauri/tests/ipc_contract.rs` -- cập nhật danh sách tham số khoá cứng `:876-878` kèm lý do là **thêm tham số**, không phải một kỳ vọng đã nới
- [x] `src-tauri/tests/segment_contract.rs` -- sửa **cơ học** sáu ca `chapter_pattern: Some(..)` sang kiểu mới, giữ nguyên mọi mệnh đề; thêm ca "tách theo vị trí giữ tiêu đề ở đầu Chương" và ca "lời tựa trước khớp đầu không bị vứt"
- [x] `src-tauri/tests/segment_chapterpattern_boundary.rs` -- cổng MỚI: sàn quần thể; `pipeline.rs` **có gọi** `chapterpattern::`; `core/segment/**` mang **0** dòng gõ `Store`/`ScopeKind`; đếm chỗ gọi sản phẩm kèm số nêu rõ. 🔴 chép `text_before_first_cfg_test_line` + **cả hai** ca tự-kiểm, và **GỌI** nó trong MỌI assert thật (bài học `cleanup_boundary.rs:136-141`); kiểm chứng dương ca dương **và** ca âm cho mỗi vị từ
- [x] `src-tauri/tests/segment_pipeline_boundary.rs` -- vá `:182` cắt `#[cfg(test)]` theo khuôn `cleanup_boundary.rs:324` -- không vá thì một khối test mới trong `src/**` làm cổng đỏ OAN
- [x] `src-tauri/tests/project_contract.rs` + `cleanup_contract.rs` -- mở rộng ca `:604` để N Chương đến **từ mẫu phân tách** (không phải `Chapters` viết tay): `ord` 1..N, mọi hàng `not_started`, `title` đúng dòng tiêu đề, segment đủ mọi Chương; 🔴 ca **đóng nợ `:9535`**: một lần nhập N≥2 Chương với một luật làm sạch khớp **số lần KHÁC NHAU** mỗi Chương ⇒ `count_in_import == Σ count_in_chapter` **đúng bằng tổng tính tay**; ca "xem trước = xác nhận từng byte" ở quy mô N (khuôn `cleanup_contract.rs:459`)
- [x] `src/config/project.ts` -- kiểu wire khối tách Chương trên **mỗi** `EncodingCandidateWire` + trường tự khai; một vế `typeof` cho **từng** trường mới, mảng `.every(...)`, nullable viết `(x === null || is…(x))`; tham số `chapterPattern` (camelCase trên dây) cho ba lệnh
- [x] `src/importPreviewState.ts` -- ô mẫu + kind, computed dẫn xuất khối tách (khuôn `:211-233`), hành động sửa mẫu tái dụng `reloadImportPreviewAfterRuleChange` (giữ `sequence`, giữ ứng viên đang chọn), cờ "đang gửi" riêng; mọi ô mới vào `resetImportPreview()`
- [x] `src/ImportPreviewOverlay.vue` -- khối mẫu phân tách (ô nhập + chọn literal/regex qua `@change`, **không** `@click`) và danh sách Chương: số nhận ra · trạng thái ban đầu · ba Chương đầu, `⋯`, ba Chương cuối · `title` (hoặc nhãn "không có tiêu đề" cho Chương lời tựa) · độ dài mỗi Chương · **sắp xếp được theo độ dài** (đường duy nhất để chỗ bắt nhầm lộ ra, AC5). `<input>` mới phải vào danh sách bẫy Tab `:297-304`; khoá i18n qua `switch` **cạn**; AD-16 dữ liệu, không markup, không `v-html`
- [x] `src/i18n/vi.json` -- khoá mới cho khối mẫu và danh sách Chương; placeholder đúng dải `{ten_tham_so}`
- [x] `tests/frontend/importPreviewChapters.test.ts` (mới) + `importPreviewEncodingWireShape.test.ts` -- khối tách của ứng viên đang chọn; đổi ứng viên ⇒ khối đổi mà **0 lời gọi IPC**; sửa mẫu ⇒ **đúng một** vòng IPC và ứng viên đang chọn còn nguyên; hình dạng dây mới có ca. 🔴 ba ca "0 IPC" hiện có **không sửa kỳ vọng**
- [x] **ĐO, đừng khai** -- xem trước nay chạy chuỗi kèm tách Chương. Mốc trước story: 6 lượt `run_pipeline`, ~13-17 ms/lượt trên 440 KB (`project.rs:1162-1178`). Đo lại trên một nguồn **nhiều Chương thật**, ghi số + ngày vào chú thích; chậm thì ghi nợ có chủ -- một mệnh đề hiệu năng không kèm phép đo là thứ kho này cấm
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- nối dòng `→` cho `:9414` (🟡 vế tiêu đề đầu Chương đóng, vế tiêu đề phụ trong thân còn mở) và `:9535` (✅ đóng bằng ca `Σ count_in_chapter`); ghi nợ **MỚI có chủ** cho: ① cờ *"đáng ngờ"* + nút lọc, Ice chốt 2026-09-05 không làm ở đây (**Chủ: Story 6.10**) · ② `spawn_import_scan` chỉ quét Chương ĐẦU, N−1 Chương không được quét ứng viên Glossary và không cổng nào đỏ (**Chủ: Story 6.10**) · ③ `⌥W`/`⌥←`/`⌥→` chưa đăng ký (**Chủ: Story 6.10**) -- `check:debt-owner` đọc **dòng `→`**, một câu trong thân mục thì cổng không thấy

**Acceptance Criteria:**
- Given một nguồn có N dòng khớp mẫu, when nhập rồi đọc lại `project.db`, then có đúng N (hoặc N+1 khi có lời tựa) hàng `chapter`, `ord` liên tục từ 1, **mọi** hàng `status = 'not_started'`, và `source_text` của mỗi Chương **bắt đầu bằng** dòng tiêu đề của chính nó.
- Given `PIPELINE_ORDER` và `segment_pipeline_boundary.rs`, when chạy sau story, then bảy bước và chỗ gọi `run_import` duy nhất **không đổi**.
- Given hai ca đối chứng AD-39 (`segment_contract.rs:8020`, `:8066`), when chạy sau story, then cả hai **vẫn xanh** và mệnh đề của chúng không bị viết lại — chỉ kiểu tham số đổi.
- Given cổng mới `segment_chapterpattern_boundary.rs`, when **gỡ** nó ra và chạy lại bộ test **CŨ**, then bộ cũ **xanh** — chứng minh mệnh đề mới thật sự chưa ai canh.
- Given một mẫu bất kỳ, when so `source_text` mà `confirm_import_with_encoding` ghi xuống với văn bản mà xem trước vừa hiện cho **từng** Chương, then **giống nhau từng byte**, trên đường lệnh thật.
- Given bộ Rust và vitest đang xanh trước story, when chạy sau story, then vẫn xanh mà không nới một kỳ vọng nào — trừ danh sách tham số `ipc_contract.rs:876` và kiểu của sáu ca `chapter_pattern`, nơi lượt sửa phải kèm lý do là **thêm tham số / đổi kiểu**.
- Given mã của story sau khi xong, when `grep` tìm một hằng số ngưỡng phán xét độ dài Chương (một phân số của trung vị, một số chữ tối thiểu, một tỉ lệ), then **0 kết quả** — quyết định 2026-09-05 của Ice phải kiểm được, không chỉ được ghi.
- Given `npm run check:debt-owner`, when chạy sau story, then **0 mục mở mồ côi**, và `:9535` đọc là đóng bằng một dòng `→`.
- Given mọi chỗ còn khai bước 5 là *"cơ chế tối thiểu, không phải mẫu người dùng cấu hình được"* (`pipeline.rs:583-590`, `import.rs:31`, `:322`, `project.rs:300-302`) sau story, when soát từng chỗ, then mỗi chỗ là một chú thích 🔵 có ngày — không chỗ nào còn khai ngược sự thật.

## Spec Change Log

## Design Notes

**Vì sao mẫu phân tách KHÔNG là cấu hình hai tầng.** Story 6.5 dựng `import_cleanup_rule` hai tầng, và một lượt rà đã bắt được cái giá: màn xem trước nhập đang **TẠO** một Tác phẩm chưa tồn tại, nên một tầng "Tác phẩm" ở đúng màn đó hoặc trượt, hoặc đính vào `project.db` của một Tác phẩm **khác** đang mở — im lặng, không cổng nào đỏ (`deferred-work.md`, cuối mục 6.5). Mẫu phân tách sống **đúng một lượt nhập** và chết theo nó, nên nó là tham số như `encoding`, không phải một hàng trong `kinds.rs`. ⇒ 0 bảng mới, 0 bước di trú, 0 `ScopeKind`, và sáu assert độ dài bộ di trú **không bị chạm**.

**Vì sao tách theo VỊ TRÍ, không theo `str::split`.** `split` xoá dấu phân tách khỏi mọi mảnh (`pipeline.rs:670`), nên cấu hình mẫu tiêu đề `第.*章` cho ra N Chương mà **không Chương nào còn tiêu đề** — và cột `title` thì vẫn `NULL`. Với FR14 đó là một khuyết tật ngữ nghĩa, không phải một lựa chọn: người dùng cấu hình *"nhận diện đầu chương"* thì đầu chương phải ở lại. Dải nửa-mở `[start_i, start_{i+1})` là cùng quy ước mà `SegmentTermSpan` và `CleanupSpanWire` đã dùng khắp kho.

**Vì sao nhánh byte ở lại literal.** `split_on_literal_bytes` (`:685-705`) không phải một bản cài đặt thiếu sót — nó là **dụng cụ đo** của AD-39: mẫu UTF-8 tìm trong byte GBK trả 0 khớp một cách tự nhiên, nên cả file ra một Chương và không lỗi nào ném, đúng câu spine `:470`. Hai ca `segment_contract.rs:8020`/`:8066` là cặp đối chứng dựng trên chính tính chất ấy. Một regex engine chạy trên `&[u8]` (`regex::bytes`) có thể khớp được ở đó và **phá cặp đối chứng** — mất một phép đo đã ký để đổi lấy một năng lực không ai cần, vì trên đường sản phẩm bước 1 luôn chạy trước bước 5.

**Lời tựa không được biến mất.** `split_on_literal` hôm nay lọc mảnh rỗng bằng `filter(|s| !s.trim().is_empty())` (`:670`) — với `split` thì mảnh #0 là phần trước khớp đầu, và một lời tựa THẬT 800 chữ sẽ lặng lẽ thành Chương `ord = 1` không tên. Tách theo vị trí làm điều đó **tường minh**: phần trước khớp đầu là một Chương có thật, `title = None`, và màn xem trước hiện nó ra. Vứt nó đi là mất dữ liệu im lặng — lớp lỗi trung tâm của dự án.

**Vì sao KHÔNG có cờ "đáng ngờ" — và AC5 vẫn đạt.** Mockup (`library-and-import.html:196-212`) vẽ một hộp *"14 dòng khớp mẫu nhưng đáng ngờ"* kèm nút lọc, và định nghĩa nó bằng *"ngắn bất thường hoặc nằm giữa một Chương khác"*. Cả hai vế đều đòi một hằng số, và không hằng số nào ở đây đo được trước khi có một kho truyện thật — đúng lý lẽ đã dùng để loại phương án *"dòng ngắn thì đừng nối"* ở Story 6.4. Ice chốt 2026-09-05: **không ngưỡng, không cờ**. AC5 (*"chỗ bắt nhầm nhìn thấy được trong màn xem trước"*) đạt bằng đường yếu hơn nhưng **không nói dối**: mọi Chương hiện `title` và độ dài, sắp xếp theo độ dài đưa ngay 14 mảnh 38-51 chữ lên đầu danh sách giữa những mảnh 4.000 chữ. Người dùng nhìn thấy; máy không phán. ⚠️ Ghi ra thay vì làm nhẹ đi: đây **yếu hơn** mockup, và nó yếu ở đúng chỗ *"chỗ dễ hỏng nhất trong toàn sản phẩm"* theo lời chính mockup. Vế mạnh là nợ có chủ Story 6.10 — story đã sở hữu bộ lọc *"cần xem"* và một trong bốn nguồn tín hiệu của nó chính là story này.

**"Cập nhật ngay" ĐÃ có cơ chế, đừng dựng cái thứ hai.** `reloadImportPreviewAfterRuleChange` (`importPreviewState.ts:355-391`) đã giải đúng bài toán này cho luật làm sạch: một vòng IPC, `sequence` chống đua, giữ ứng viên bảng mã đang chọn. Sửa mẫu tái dụng nguyên đường ấy. ⚠️ Phân biệt hai nhịp: **đổi ứng viên** phải 0 IPC (kết quả tách đi kèm payload từng ứng viên, như `normalized`/`cleanup`); **sửa mẫu** thì một vòng IPC là đúng — ba ca "0 IPC" chỉ nói về nhịp thứ nhất.

## Verification

**Commands:**
- `npm run build && cargo test --locked` -- expected: xanh; `dist/` phải có TRƯỚC `cargo test`, thiếu nó thì gãy ở khâu biên dịch chứ không ở một assert
- `npm run test` -- expected: ≥ 842 ca xanh (số trước story, đo 2026-09-05), 0 đỏ
- `npm run check:debt-owner` -- expected: 0 mục mở thiếu `Chủ:`
- `npm run check:i18n && npm run check:tokens && npm run check:commands && npm run check:gates` -- expected: 0 findings mỗi cổng
- `cargo test --locked --test segment_contract` -- expected: hai ca AD-39 (`:8020`, `:8066`) xanh **mà mệnh đề không đổi**
- Đối chứng đỏ: `git stash` cổng mới `segment_chapterpattern_boundary.rs` rồi `cargo test --locked` -- expected: **xanh** (chứng minh mệnh đề mới chưa ai canh)
- Đối chứng đỏ thứ hai: gỡ lời gọi `chapterpattern::` khỏi `pipeline.rs` rồi chạy cổng mới -- expected: **đỏ**

**Manual checks (if no CLI):**
- Mở màn xem trước với một nguồn nhiều Chương thật: số Chương nhận ra khớp số dòng tiêu đề đếm tay; ba Chương đầu và ba Chương cuối hiện đúng; sửa mẫu thì danh sách đổi mà không bấm nút nào; `Tab` xoay vòng qua ô mẫu và không thoát khỏi lớp phủ.

## Suggested Review Order

**Cơ chế tách — đọc trước tiên**

- Điểm vào: ba cửa thoát sớm rồi rẽ hai nhánh theo hình dạng đơn vị.
  [`pipeline.rs:648`](../../src-tauri/src/core/segment/pipeline.rs#L648)

- Tách theo VỊ TRÍ `[start_i, start_{i+1})` — thay `str::split` vốn nuốt dấu phân tách.
  [`pipeline.rs:737`](../../src-tauri/src/core/segment/pipeline.rs#L737)

- Quy tắc tiêu đề dựa trên SỐ DÒNG, không một ngưỡng độ dài nào.
  [`pipeline.rs:773`](../../src-tauri/src/core/segment/pipeline.rs#L773)

- Module thuần: `multi_line(true)` bắt buộc, lọc khớp độ dài 0, mẫu hỏng trả `Err`.
  [`chapterpattern.rs:74`](../../src-tauri/src/core/segment/chapterpattern.rs#L74)

**Ranh giới lệnh — nơi mẫu được nghiệm thu và N Chương được ghi**

- Rào biên dịch thử: mẫu hỏng bị chặn TRƯỚC khi chạm chuỗi pipeline.
  [`project.rs:256`](../../src-tauri/src/commands/project.rs#L256)

- Vòng ghi N Chương: `ord` liên tục, `title` thay `NULL`, mọi hàng `not_started`.
  [`project.rs:414`](../../src-tauri/src/commands/project.rs#L414)

- Xem trước đọc TOÀN BỘ `outcome.chapters`, không còn `.next()`.
  [`project.rs:1363`](../../src-tauri/src/commands/project.rs#L1363)

- Hình dạng dây của khối tách, dựng cho MỖI ứng viên bảng mã.
  [`project.rs:1165`](../../src-tauri/src/commands/project.rs#L1165)

**Trạng thái frontend — ba chỗ dễ sai nhất**

- Lõi tải lại dùng chung: gửi mẫu hiện hành ở mọi lượt, hai tầng không trôi khỏi nhau.
  [`importPreviewState.ts:456`](../../src/importPreviewState.ts#L456)

- Lỗi mẫu hỏng KHÔNG được sụp lớp phủ — định tuyến riêng theo mã lỗi.
  [`importPreviewState.ts:180`](../../src/importPreviewState.ts#L180)

- Sửa mẫu: xếp hàng lượt gõ sau, chặn xác nhận khi đang bay.
  [`importPreviewState.ts:574`](../../src/importPreviewState.ts#L574)

- `trim` để một ô toàn khoảng trắng không thành mẫu literal thật.
  [`importPreviewState.ts:444`](../../src/importPreviewState.ts#L444)

**Giao diện**

- Tầng 4: ô mẫu qua `@change`, danh sách Chương sắp xếp được theo độ dài.
  [`ImportPreviewOverlay.vue:717`](../../src/ImportPreviewOverlay.vue#L717)

- Kiểm kiểu lúc chạy cho từng trường mới của khối tách.
  [`project.ts:330`](../../src/config/project.ts#L330)

**Cổng và đối chứng — đọc sau cùng**

- Cổng mới: mọi assert thật đều cắt `#[cfg(test)]` trước khi quét.
  [`segment_chapterpattern_boundary.rs:144`](../../src-tauri/tests/segment_chapterpattern_boundary.rs#L144)

- Ca đóng nợ: đọc `count_in_import` TỪ dây sản phẩm, không tự cộng trong ca test.
  [`cleanup_contract.rs:900`](../../src-tauri/tests/cleanup_contract.rs#L900)

- N Chương đến TỪ mẫu phân tách, không phải hình dạng viết tay.
  [`project_contract.rs:679`](../../src-tauri/tests/project_contract.rs#L679)
