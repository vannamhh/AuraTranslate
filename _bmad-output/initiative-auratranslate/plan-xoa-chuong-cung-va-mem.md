---
title: 'Xoá Chương: xoá hẳn khi chưa dịch, xoá mềm khi đã dịch'
type: 'feature'
ticket: ''
created: '2026-11-14'
status: 'built'
baseline_revision: '7e6b6b1e5f25a196288123ba889f1075b3514c23'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Không có đường nào xoá một Chương. `story-to-chuc-lai-chuong-sau-khi-nhap` cố ý gạt ra: *"xoá một Chương là xoá văn bản người dùng, một quyết định chưa AC nào đòi"*. Chương nhập trùng, nhập sai hay chỉ có rác hiện chỉ gộp được vào Chương khác, và PRD chưa có FR nào cho việc xoá.

**Approach:** Một nút xoá hai bước trong panel Chương của `LibraryWorkPage.vue`. Rust một mình quyết đường đi: Chương không còn gì để giữ thì xoá hẳn, theo khuôn dọn dẹp mà `merge_into_previous_tx` đã chạy; Chương còn gì để giữ thì xoá mềm — rời danh sách, tìm kiếm, bản xuất và tiến độ, mà không mất một byte `target_text` hay `segment_version` nào. Xoá Chương cuối cùng của một Tác phẩm không còn gì để giữ thì xoá luôn thư mục `.atproj`.

**Phân lớp AD-49 (mục 2 đòi khai trong spec):** xoá hẳn một Chương và xoá một Tác phẩm thuộc lớp **(iii)** — sau lượt ghi không còn bản sao nào trong app. Xoá mềm giữ mọi byte chữ nguồn, bản dịch và lịch sử bản dịch, nhưng phá bản reviewer đã nhập, alignment và dấu đọc của Chương đó (D2). Cả ba vẫn hỏi hai bước, vì người dùng không biết trước đường nào sẽ chạy.

**Đã chốt (Ice, 2026-11-14):**
- Xoá mềm là **dời hàng sang bảng `chapter_deleted`** và cho các segment của nó về hưu bằng `retired_at` có sẵn — không phải một cột `chapter.deleted_at`. `chapter` giữ nguyên nghĩa *"các Chương của Tác phẩm này"*, nên **0** trong 19 câu SQL liệt kê Chương phải sửa, `ord` không có lỗ, `chapter_count` vẫn đúng, không cần `AD` mới.
- **"Xoá luôn Tác phẩm" là xoá thư mục `.atproj` khỏi đĩa**, rồi dựng lại chỉ mục. Mất hẳn; câu xác nhận nói thẳng rằng cả Tác phẩm và mọi thứ trong `.atproj` sẽ mất.
- **Chương hiện cuối còn gì để giữ: xoá mềm Chương, GIỮ Tác phẩm.** Chỉ xoá `.atproj` khi sau lượt xoá `chapter` rỗng VÀ `chapter_deleted` rỗng.
- **D1 — Tác phẩm 0 Chương hiện ra vẫn mở được**, với danh sách Chương rỗng và không Chương nào đang mở (`OpenWork::chapter_id` thành `Option<i64>`).
- **D2 — Xoá mềm dọn đúng các bảng phụ mà xoá hẳn dọn** (`chapter_position`, alignment cùng `review_decision`, `review_row`/`review_chapter`, `reading_mark`, `segment_unlinked`); chỉ giữ hàng Chương (dời sang `chapter_deleted`), các segment (về hưu, chữ nguyên vẹn), `segment_version` và `asset`.
- **D3 — "Còn gì để giữ"** = có ít nhất một segment của Chương mang `target_text` không trắng (kể cả câu `is_omitted` và câu đã về hưu), HOẶC có ít nhất một hàng `segment_version` của các segment đó. Không thoả ⇒ xoá hẳn.
- **D4 — Câu xác nhận lấy từ lệnh chỉ-đọc `delete_chapter_preview(chapter_id)`**, trả `{ disposition, removes_work }` tính bằng đúng phép thử D3 và số hàng `chapter_deleted` mà `delete_chapter` dùng; `ChapterRow` không đổi.
- Giữ cả D1 và lệnh xoá trong một plan dù vượt hướng dẫn 1600 token (thân plan 2.045 từ; số token chưa đếm được vì máy không có `tiktoken`).

## Boundaries & Constraints

**Always:** Rust quyết hẳn-hay-mềm bằng phép thử D3, không tin cờ nào từ webview; `chapter.status` không phải căn cứ. Một lượt xoá là một giao dịch `store.write`; cả hai đường đánh số lại `ord` liền 1..N. Con trỏ Chương đang mở chỉ đổi qua `set_open_chapter` (cùng lượt cho `OpenWork` và `work.last_chapter_id`): sang Chương liền trước, liền sau nếu nó là Chương đầu, không Chương nào nếu hết Chương hiện. Đóng Tác phẩm do xoá chạy cùng phần dọn theo Tác phẩm như hai đường đóng có sẵn, và `Store` cũ đóng NGOÀI khoá `OpenWorkState` (🔴 AC10). Xoá thư mục mà hỏng thì báo lỗi, không bao giờ trả `work_removed: true`. `meta.json` và `library-index.db` dựng lại qua `Indexer` sau khi `.atproj` ghi xong (AD-8); sau khi xoá một Tác phẩm, cùng lượt phải `orphan_store::forget` để Library không hiện mục mồ côi nào cho nó (FR99). Câu xác nhận lấy từ đúng phép thử mà lượt xoá dùng. Hỏi theo khuôn hai bước của `GlossaryManageOverlay`/`TmManageOverlay`, nút `.ui-btn--danger`. Lỗi đi qua `tError` + `message_key`.

**Never:** Không ngăn xếp hoàn tác, không `Mod+Z`, không thùng rác (AD-49). Không màn khôi phục Chương đã xoá mềm. Không `window.confirm`. Không xoá nhiều Chương một lượt. Không tính lại ranh giới segment (AD-4). Không đổi `chapter.status` để giả lập đã xoá, không thêm giá trị vào `LifecycleStatus`. Không nút "xoá Tác phẩm" độc lập — đường duy nhất là xoá Chương cuối. Không thêm vị từ lọc vào 19 câu SQL liệt kê Chương. Không sửa hợp đồng của `remove_folder`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Xoá Chương không còn gì để giữ | D3 không thoả | Hàng `chapter` và mọi hàng `segment`/`segment_version`/`asset`/`chapter_position`/alignment/`review_*`/`reading_mark`/`segment_unlinked` của nó biến mất; `ord` liền 1..N | No error expected |
| Xoá Chương có bản dịch | ≥1 câu sống có bản dịch | Hàng dời sang `chapter_deleted`, segment mang `retired_at`, bảng phụ D2 dọn; **0 byte** `target_text`/`segment_version` mất; danh sách dấu đọc của Tác phẩm vẫn đọc được | No error expected |
| Bản dịch chỉ ở chỗ khuất | bản dịch chỉ ở câu `is_omitted`/đã về hưu, hoặc chỉ có `segment_version` | Xoá mềm | No error expected |
| Chương đang mở | `OpenWork::chapter_id` là Chương bị xoá | Con trỏ sang Chương liền trước, liền sau nếu nó là Chương đầu; webview nạp lại Editor + Source | No error expected |
| Bấm lần đầu | chưa xác nhận | **0 hàng bị chạm**; nút đổi nhãn; câu nói đúng đường Rust sẽ chạy, có *"không hoàn tác được"* ở hai đường phá dữ liệu, và câu mềm kể ra những gì mất (D2) | No error expected |
| Chương không còn | id đã xoá hoặc id lạ | **0 hàng bị chạm** | `err.chapter.not_found` `{chapter_id}` |
| Sau một lượt xoá mềm | tìm kiếm Library, bản xuất, bản reviewer | Chương không ra kết quả, không vào tệp xuất, không khớp bản reviewer | No error expected |
| Tiến độ | sau một lượt xoá bất kỳ, mở lại Tác phẩm | `chapter_count`, `chapter_done_count` và trạng thái thẻ không đếm Chương đã xoá | No error expected |
| Chương cuối, không còn gì để giữ | 1 Chương, `chapter_deleted` rỗng | Xoá Chương **và** thư mục `.atproj`; Tác phẩm rời Library; app về Library mode, danh tính Tác phẩm đang mở bị xoá | No error expected |
| Chương hiện cuối, còn gì để giữ | 1 Chương, D3 thoả | Xoá mềm; Tác phẩm **ở lại** Library; mở lại được với 0 Chương, Editor không nạp gì | No error expected |
| Chương hiện cuối, còn Chương đã xoá mềm | `chapter` 1 hàng, `chapter_deleted` không rỗng | Xoá theo luật riêng; **thư mục KHÔNG bị xoá**; câu xác nhận không nói xoá cả Tác phẩm | No error expected |
| Xoá thư mục hỏng | `.atproj` không xoá được | Tác phẩm vẫn đóng, không báo `work_removed` | khoá lỗi xoá Tác phẩm `{name}` |
| Mồ côi sau khi xoá Tác phẩm | `.atproj` vừa bị xoá | Library hiện **0** mục mồ côi cho Tác phẩm đó | No error expected |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/chapter.rs` -- `:947-1038` `merge_into_previous_tx`: khuôn dọn (`chapter_position`, `delete_alignment_of_chapter`, `review_row`/`review_chapter`, `DELETE FROM chapter`). `:577` `normalize_chapter_ord`: DÙNG LẠI, đừng chép. `:117` `set_open_chapter(open, i64)` ⇒ đổi tham số thành `Option<i64>` (`NULL` khi `None`). `:344` `ChapterRow`, dựng ở đúng một chỗ `:421`. `:1539` `finish_with_reindex`. `:144` `read_open_chapter`.
- `src-tauri/src/core/store/schema.rs:57` `segment_has_translation_sql!()` -- đứng cạnh nó thêm `segment_has_keepable_sql!()` (bí danh `s`, dùng `unicode_white_space_sql!()` `:40`, không lọc `is_omitted`/`retired_at`, cộng `EXISTS segment_version v WHERE v.segment_id = s.id`). `:1852` `segment_version`. `PROJECT_MIGRATIONS` bước cuối `45` ⇒ `chapter_deleted` là bước **46**, soi đủ 11 cột của `chapter` (`CHAPTER_DDL` + `CHAPTER_ORIGIN_DDL`) cộng `deleted_at`; tiêu đề doc "bốn mươi bốn bước" phải đổi.
- `src-tauri/src/commands/project/mod.rs` -- `:102` `OpenWork::chapter_id` ⇒ `Option<i64>`. `:3509-3541` `open_work`: bỏ nhánh `OpenFailed` "khong co hang chapter nao", `chapter_id: found`. `:~330` `OpenedWork.chapter_id` ⇒ `Option`. `:3721` 🔴 AC10, `swap_locked` `:~3838`. `:3735-3838` `replace_open_work` và `lib.rs:1910` `close_open_work` chép cùng sáu lượt dọn theo Tác phẩm ⇒ rút thành `clear_per_work_state(app)` dùng chung (`close_open_work` còn `clear_call_records_on_work_close` — đường xoá cũng gọi nó). `:3786` lượt quét tệp ảnh mồ côi: lý do `asset` phải ở lại trong đường mềm.
- Chỗ đọc `OpenWork::chapter_id` cần quyết khi `None`: lọc/so sánh coi `None` là "không phải Chương đang mở" — `convention.rs:283,347,613,640,713` (`:713` đang `.unwrap_or_default()` ra 0: bẫy AGENTS.md), `segment/convention_write.rs:130,458`, `segment/proofread_accept.rs:174`, `segment/targets.rs:382`, `proofread.rs:483`, `chapter.rs:929`; cần Chương thì trả `chapter_not_found`/`no_work_open` có sẵn — `chapter.rs:144,265,1074`, `segment/chapter_read.rs:393`, `segment/reading.rs:376`; `segment/chapter_read.rs:538` `fill_exact_tm_matches` trả rỗng. `project/wire.rs:412` `OpenedWork::from_open`. Test: `aiconfig_contract.rs:720` dựng `OpenWork {` trần; ~25 tệp test đọc `.chapter_id` cần `.expect()`.
- `src-tauri/src/core/library/atproj.rs:215` `remove_folder` -- KHÔNG gọi, KHÔNG sửa. Thêm hàm anh em `delete_work_folder(dir) -> Result<(), WorkError>` + biến thể/khoá lỗi riêng cho xoá hỏng. `indexer.rs:560` ghi mồ côi; `orphan_store.rs:124` `forget`.
- `src-tauri/src/commands/segment/reading.rs:42-77` `select_reading_marks` -- so số đếm, nên một `reading_mark` trỏ câu đã về hưu làm hỏng cả danh sách: lý do D2 dọn `reading_mark`.
- Webview: `src/modes/libraryChapters.ts:~330-445` `openWorkById` gọi `ensureChapterLoaded` (`panels/sourcePanelState.ts:278`) và `ensureSegmentsLoaded` (`panels/editorPanelState.ts:250`) ở `:428-430` ⇒ bỏ qua khi `opened.chapter_id === null`. `src/config/library.ts:314,333` `OpenedWork` + `isOpenedWork` ⇒ `number | null`. `src/openWorkIdentity.ts:17` `resetOpenWorkId` hiện 0 chỗ gọi; đường Tác phẩm biến mất phải gọi. Khuôn: `libraryChapters.ts:768` `mergeSelectedChapters`, `src/config/chapter.ts:389` vỏ `invoke`, `src/commands/index.ts:2418` dispatch, `src/libraryWorkPageCommandDeps.ts`, `src/tokens/controls.css:37`. Tiền lệ hai bước: `src/GlossaryManageOverlay.vue:765-779`.
- Test: mở lại Tác phẩm theo khuôn `tests/project_contract.rs:2667` `indexed_work_from` + `open_work(&work_id, Some(&indexed))` (`:2714`); `WorkMeta::rebuild_from_store` cho `chapter_count`/`chapter_done_count`. Bốn test ghim bậc di trú: `pinned_contract.rs:262`, `segment_contract.rs:932`, `chapter_origin_contract.rs:1075`, `segment_role_contract.rs:790`; `config_invariants.rs` đếm lệnh theo tệp.
- Bản vá lượt 1 (đã revert, có khuyết R1–R13) còn ở `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/e943f3ea-9a1f-4a8c-af2d-2484bba5a1c2/scratchpad/xoa-chuong-loop1.patch` nếu thư mục tạm chưa bị dọn: dùng tham khảo cho DDL bước 46, vỏ lệnh async, nút hai bước, ca test reviewer và ca component.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/store/schema.rs` -- `segment_has_keepable_sql!()`; `CHAPTER_DELETED_DDL` bước 46.
- [ ] `src-tauri/src/commands/project/mod.rs`, `lib.rs`, `project/wire.rs` và các chỗ đọc ở Code Map -- `OpenWork::chapter_id: Option<i64>`; `open_work` mở được Tác phẩm 0 Chương; `clear_per_work_state` dùng chung.
- [ ] `src-tauri/src/commands/chapter.rs` -- `set_open_chapter(Option<i64>)`.
- [ ] `src-tauri/src/core/chapter_delete/` (mới) -- `delete_chapter_tx`: phép thử D3, dọn bảng phụ chung cho cả hai đường, xoá `asset`/`segment_version`/`segment` chỉ ở đường hẳn, dời hàng ở đường mềm, `normalize_chapter_ord`, trả Chương kế tiếp và cờ "Tác phẩm rỗng".
- [ ] `src-tauri/src/commands/chapter.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/core/i18n/mod.rs`, `src-tauri/src/core/library/atproj.rs` -- lệnh `delete_chapter` (async) và lệnh chỉ-đọc `delete_chapter_preview` (D4); khi Tác phẩm rỗng: lấy `OpenWork` ra trong khoá, dọn theo Tác phẩm, đóng `Store` ngoài khoá, `delete_work_folder`, dựng lại chỉ mục, `orphan_store::forget`; khoá lỗi mới.
- [ ] `src/config/chapter.ts`, `src/config/library.ts`, `src/modes/libraryChapters.ts`, `src/commands/index.ts`, `src/libraryWorkPageCommandDeps.ts` -- vỏ `invoke`, mở Tác phẩm 0 Chương, hàm store hai bước, dispatch `library.chapter_delete.*`; Tác phẩm biến mất thì `resetOpenWorkId()` và về Library mode.
- [ ] `src/modes/LibraryWorkPage.vue`, `src/i18n/vi.json` -- nút hai bước; lần bấm đầu gọi `delete_chapter_preview` và chọn một trong ba câu (hẳn · mềm · cả Tác phẩm) theo kết quả.
- [ ] `src-tauri/tests/chapter_delete_contract.rs` (mới), `tests/frontend/libraryChapterBulkWrites.test.ts`, `tests/frontend/libraryChapters.test.ts` (ca component), test ghim bậc di trú và các test đọc `.chapter_id` -- phủ mọi hàng Matrix; ca mở lại qua `open_work`.

**Acceptance Criteria:**
- Given một Chương có bản dịch, when xoá và xác nhận, then số câu có `target_text` không trắng và số hàng `segment_version` của Chương đó trong kho không đổi.
- Given đã xoá một Chương ở giữa, when mở lại Tác phẩm qua `open_work`, then `ord` các Chương còn lại liền 1..N và thẻ Library hiện `chapter_count` mới.
- Given Chương đang mở bị xoá, when lượt xoá xong, then Editor đứng ở Chương liền trước và `work.last_chapter_id` trỏ đúng nó.
- Given một Tác phẩm chỉ còn một Chương không còn gì để giữ, when xoá và xác nhận, then thư mục `.atproj` không còn trên đĩa, Tác phẩm rời Library, và Library hiện 0 mục mồ côi.

## Implementation Notes

- `work.delete_failed` đi cùng đường dọn với `work_removed` ở webview (`leaveVanishedWork`), vì Rust đã lấy `OpenWork` ra và đóng `Store` trước khi xoá thư mục; lỗi hiện ở lưới Library (`data-library-open-work-error`), không ở trang Tác phẩm đã rời.
- `core/chapter_delete` đọc qua `ReadHandle<'_>`, không `rusqlite::Connection`: `store_boundary` (AD-11) chỉ lộ ở lượt chạy trọn bộ.
- Hai tệp `.rs` mới đẩy số tệp lên 279, nên `SRC_TAURI_RS_FLOOR` của `dict_boundary.rs` nâng 223 → 238 theo đúng công thức của nó.
- Đối chứng đỏ bằng gỡ thật cả bốn seam ở §Verification: mỗi ca đỏ đúng mệnh đề (mục mồ côi hiện ra; 0 hàng `chapter_deleted`; `Hard` thay `Soft`; danh sách dấu đọc trả `store.read_failed`).
- Tác phẩm giữ lại với 0 Chương hiện ra không còn đường xoá trong UI: nợ ở `deferred-work.md`, Chủ: Sofia (Q5).

## Plan Change Log

- Lượt review 1 (2026-11-14): R1–R4 là intent_gap; Ice chốt D1–D3, code đã revert. Plan viết lại: phép thử D3 thay `segment_has_translation_sql!()`; đường mềm dọn bảng phụ (D2); `OpenWork::chapter_id` thành `Option`; khắc phục R5–R13 bằng các luật Always/Matrix mới (con trỏ qua `set_open_chapter`, Store đóng ngoài khoá, dọn theo Tác phẩm, `delete_work_folder` báo lỗi, câu xác nhận từ Rust, ca mở lại Tác phẩm). KEEP: DDL đủ 11 cột; ca test reviewer qua `plan_import`; ca component bấm nút thật; đối chứng đỏ bằng gỡ thật.

## Review Triage Log

Lượt 1 (lens `quick`):
- R1 high · intent_gap — Xoá mềm Chương hiện cuối để lại Tác phẩm 0 Chương mà `open_work` trả `OpenFailed` (`commands/project/mod.rs:3538`); không mở được, cũng không xoá được vì đường xoá duy nhất cần mở Tác phẩm. Intent chưa nói Tác phẩm 0 Chương hiện ra thì mở thế nào.
- R2 high · intent_gap — Xoá mềm vẫn chạy phần dọn chung: xoá `review_row`/`review_chapter`, alignment và `review_decision` (văn bản reviewer đã nhập), trái lớp AD-49 (ii) "không phá byte nội dung người dùng nào"; intent không nói đường mềm giữ hay dọn các bảng này.
- R3 high · intent_gap — Xoá mềm để `reading_mark.navigation_segment_id` trỏ câu đã về hưu; `select_reading_marks` (`commands/segment/reading.rs:72`) so số đếm và trả `InvalidQuery` cho cả Tác phẩm. Cùng gốc R2: hợp đồng "không mồ côi" của intent đụng "không phá byte" ở đường mềm (cả `segment_unlinked` và `asset`).
- R4 high · intent_gap — Phép thử bắt buộc `segment_has_translation_sql!()` bỏ qua câu `is_omitted`, câu đã về hưu và `segment_version`; đường hẳn xoá bản dịch nằm ở đó, trái "Không bao giờ phá một bản dịch". Mâu thuẫn bên trong khối đóng băng.
- R5 medium · bad_plan — Câu xác nhận nói sai đường: `'work'` khi `chapter_deleted` không rỗng (webview không thấy bảng này), và `translated_count` dùng `UNTRANSLATED_SQL` (6 ký tự trắng ASCII) trong khi Rust dùng 25 điểm Unicode, nên bản dịch toàn U+3000 nhận câu "mềm" rồi bị xoá hẳn.
- R6 medium · bad_plan — Con trỏ không đi qua `set_open_chapter`; sau xoá mềm Chương hiện cuối `OpenWork::chapter_id` vẫn trỏ hàng đã dời trong khi `last_chapter_id` là NULL.
- R7 medium · bad_plan — `drop(work)` chạy `Store::close` (TRUNCATE) trong khi còn giữ khoá `OpenWorkState`, trái 🔴 AC10 (`commands/project/mod.rs:3721`).
- R8 medium · bad_plan — Đóng Tác phẩm qua lượt xoá bỏ qua phần dọn theo Tác phẩm mà `close_open_work`/`replace_open_work` chạy (lô nhập treo, prompt tầng Work); webview không gọi `resetOpenWorkId()`.
- R9 medium · bad_plan — `remove_folder` nuốt lỗi và khi `resolve_library_root` lỗi thì `RemovedWork` bị bỏ, nhưng câu trả lời vẫn `work_removed: true`; doc 🔴 của `remove_folder` nay tự mâu thuẫn.
- R10 low · patch — `ASSET_DDL` vẫn viết "không còn đường nào tạo hàng `asset` mồ côi" trong khi đường mềm giữ hàng trỏ Chương đã dời (giữ là đúng: lượt quét `mod.rs:3786` sẽ xoá tệp ảnh nếu mất hàng).
- R11 low · patch — `renumber_chapters` chép nguyên `normalize_chapter_ord` (`commands/chapter.rs:577`).
- R12 low · patch — Ba comment mới viết tiếng Việt hoặc chỉ lặp chữ ký (`atproj.rs`, `core/i18n/mod.rs`, `wire::delete_chapter`).
- R13 medium · bad_plan — Chưa ca nào mở lại Tác phẩm qua `open_work` (AC3) hay kiểm tiến độ thẻ sau xoá (hàng Matrix "Tiến độ"); chính khe này che R1.

Lượt 2 (lens `quick`):
- Q1 medium · patch — `work.delete_failed`: Rust đã `state.take()` + đóng `Store` (`commands/chapter.rs:1382,1434`, ca `a_work_folder_that_cannot_be_deleted…`), nhưng nhánh lỗi của `confirmSelectedChapterDelete` chỉ đặt `chapterReorgError`, nên trang Tác phẩm và Editor vẫn hiện một Tác phẩm đã đóng, mọi lệnh sau trả `work.none_open`.
- Q2 low · rejected — `resolve_library_root` lỗi thì bỏ dựng lại chỉ mục mà vẫn `work_removed: true`; chỉ xảy ra khi không có override, không có gốc đã cấu hình và `document_dir()` cũng hỏng, nên không gặp trong dùng thường, và sửa phải thêm nhánh.
- Q3 low · patch — Comment chuyển chỗ còn mang story id, ngày, ký hiệu và lý lẽ "trước khi swap" nay sai chỗ: `clear_per_work_state` (`commands/project/mod.rs:3688,3710,3715`) và `resetWorkScopedState` (`src/modes/libraryChapters.ts`); AGENTS.md §Code comments bảo xoá ở dòng đang đổi.
- Q4 false — Tác phẩm 0 Chương hiện ra có trạng thái suy ra `not_started`, 0/0: đúng hàng Matrix "Tiến độ" (không đếm Chương đã xoá) và D1.
- Q5 medium · defer — Tác phẩm giữ lại với 0 Chương hiện ra (còn hàng `chapter_deleted`) không còn đường xoá nào trong UI; intent cấm nút xoá Tác phẩm độc lập và màn khôi phục, nên ghi nợ có chủ thay vì sửa.
- Q6 low · patch — Câu hỏi xác nhận dùng `role="alert"`; tiền lệ hai bước `GlossaryManageOverlay.vue:764` dùng `role="status"`.
- Q7 low · rejected — Bấm "Xoá Chương…" gỡ nút khỏi DOM nên mất tiêu điểm bàn phím; sửa cần thêm ref và lượt `focus()`, không phải sửa thẳng.

## Design Notes

Hai phát biểu của cùng luật "đã dịch" đang cùng tồn tại: `segment_has_translation_sql!()` (`schema.rs:57`) và `UNTRANSLATED_SQL` (`core/export/block_paragraphs.rs:17`). Lệnh xoá dùng cả hai cho việc khác: phép thử D3 mới (đặt cạnh bản ở `schema.rs`, cùng tập 25 ký tự trắng Unicode) quyết đường đi; `ChapterRow.translated_count` (bản export, 6 ký tự trắng ASCII) KHÔNG được dùng cho câu xác nhận, vì lệch hướng nguy hiểm nhất: bản dịch toàn U+3000 sẽ nhận câu "mềm" rồi bị xoá hẳn. Plan này không hợp nhất hai bản.

Khoá API là Global-only, cố ý, vì *"xoá Tác phẩm thì mục keychain đi đâu"* không ai trả lời được (nợ ở `deferred-work.md`, spec 4.3). ⇒ Xoá một Tác phẩm không có hệ quả keychain nào.

## Verification

**Commands:**
- `npm run build`, rồi `cargo test --test chapter_delete_contract` -- xanh.
- `cargo test --test pinned_contract --test segment_contract --test chapter_origin_contract --test segment_role_contract --test config_invariants --test ipc_contract --test project_contract` -- xanh.
- `npx vitest run tests/frontend/libraryChapters.test.ts tests/frontend/libraryChapterBulkWrites.test.ts tests/frontend/chapterWireGuards.test.ts` -- xanh.
- `npm run check:i18n`, `npm run check:commands` -- xanh.
- Chạy trọn bộ một lần, vì diff chạm `PROJECT_MIGRATIONS` và `lib.rs` (AGENTS.md §Tests).
- Đối chứng đỏ: gỡ THẬT lượt `orphan_store::forget` khỏi đường xoá Tác phẩm, rồi chạy lại đúng ca mồ côi; đỏ phải vì mục mồ côi hiện ra, không vì lý do khác. Làm cùng khuôn cho lượt dời hàng sang `chapter_deleted`, cho nhánh `segment_version` của phép thử D3, và cho lượt dọn `reading_mark` ở đường mềm.

**Manual checks (if no CLI):**
- Ice, trong app thật: xoá một Chương rác, xoá một Chương đã dịch, xoá Chương cuối của một Tác phẩm thử, mở lại app, rồi xem danh sách Chương, thẻ Library, mục mồ côi và một lượt xuất bản.
