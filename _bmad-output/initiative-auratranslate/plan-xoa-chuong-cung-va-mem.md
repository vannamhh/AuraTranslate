---
title: 'Xoá Chương: xoá hẳn khi chưa dịch, xoá mềm khi đã dịch'
type: 'feature'
ticket: ''
created: '2026-11-14'
status: 'ready-for-dev'
baseline_revision: '7e6b6b1e5f25a196288123ba889f1075b3514c23'
route: 'full'
route_source: 'auto'
risk: 'high'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Không có đường nào xoá một Chương. `story-to-chuc-lai-chuong-sau-khi-nhap` cố ý gạt ra: *"xoá một Chương là xoá văn bản người dùng, một quyết định chưa AC nào đòi"*. Chương nhập trùng, nhập sai hay chỉ có rác hiện chỉ gộp được vào Chương khác, và PRD chưa có FR nào cho việc xoá.

**Approach:** Một nút xoá hai bước trong panel Chương của `LibraryWorkPage.vue`. Rust một mình quyết đường đi: Chương không có câu nào mang bản dịch thì xoá hẳn, theo đúng khuôn dọn dẹp mà `merge_into_previous_tx` đã chạy; Chương đã có bản dịch thì xoá mềm — rời danh sách, tìm kiếm, bản xuất và tiến độ, mà không mất một byte `target_text` nào. Xoá Chương cuối cùng của một Tác phẩm không còn gì để giữ thì xoá luôn thư mục `.atproj`.

**Phân lớp AD-49 (mục 2 đòi khai trong spec):** xoá hẳn một Chương và xoá một Tác phẩm thuộc lớp **(iii)** — sau lượt ghi không còn bản sao nào trong app. Xoá mềm thuộc lớp **(ii)** — không phá byte nội dung người dùng nào. Cả ba vẫn hỏi hai bước, vì người dùng không biết trước đường nào sẽ chạy.

**Đã chốt (Ice, 2026-11-14):**
- Xoá mềm là **dời hàng sang bảng `chapter_deleted`** và cho các segment của nó về hưu bằng `retired_at` có sẵn — không phải một cột `chapter.deleted_at`. Lý do chọn: `chapter` giữ nguyên nghĩa *"các Chương của Tác phẩm này"*, nên **0** trong 19 câu SQL liệt kê Chương phải sửa, `ord` không có lỗ, `chapter_count` vẫn đúng, và không bất biến nào đổi nên không cần một `AD` mới.
- **"Xoá luôn Tác phẩm" là xoá thư mục `.atproj` khỏi đĩa**, rồi dựng lại chỉ mục. Mất hẳn, không lấy lại được; câu xác nhận phải nói thẳng rằng cả Tác phẩm và mọi thứ trong `.atproj` sẽ mất.
- **Tác phẩm một Chương mà Chương đó đã dịch: xoá mềm Chương, GIỮ Tác phẩm.** Không bao giờ phá một bản dịch. ⇒ Suy ra luật xoá thư mục: chỉ xoá `.atproj` khi sau lượt xoá Tác phẩm không còn Chương nào, **cả hiện lẫn đã xoá mềm** (`chapter` rỗng VÀ `chapter_deleted` rỗng) — một Tác phẩm còn giữ Chương đã xoá mềm thì ở lại Library với 0 Chương hiện ra.
- Giữ cả ba luật trong một plan dù vượt hướng dẫn 1600 token (1472 từ lúc ký; không đếm được bằng tiktoken vì máy không có gói này và PyPI không tới được).

## Boundaries & Constraints

**Always:** Rust quyết hẳn-hay-mềm bằng `segment_has_translation_sql!()` (`schema.rs:57`), không tin cờ nào từ webview; `chapter.status` không phải căn cứ, vì Story 5.4 đã ghi lối ghi duy nhất trong app viết cứng `'done'`. Một lượt xoá là một giao dịch `store.write`: đường xoá hẳn dọn đúng các bảng `merge_into_previous_tx` dọn và theo thứ tự đó, cộng thêm `segment`; đường xoá mềm dời hàng sang `chapter_deleted` và đặt `retired_at` cho các segment của nó; cả hai rồi đánh số lại `ord` liền 1..N, và không để lại hàng mồ côi (hợp đồng đã viết trong `ASSET_DDL`/`CHAPTER_POSITION_DDL`). Xoá Chương đang mở thì con trỏ dời trong cùng lượt qua `set_open_chapter`: sang Chương liền trước, liền sau nếu nó là Chương đầu; `work.last_chapter_id` không bao giờ trỏ một Chương đã xoá. `meta.json` và `library-index.db` dựng lại qua `Indexer` sau khi `.atproj` ghi xong (AD-8). Sau khi xoá một Tác phẩm, lượt dựng lại chỉ mục ghi nó thành mục mồ côi (`indexer.rs:560`, FR99) nên cùng lượt đó phải quên nó đi (`orphan_store::forget`), và Library không được hiện một mục mồ côi nào cho Tác phẩm Ice vừa cố ý xoá. Hỏi theo khuôn hai bước có sẵn của `GlossaryManageOverlay`/`TmManageOverlay`, nút `.ui-btn--danger`, và câu xác nhận nói đúng đường sắp chạy. Lỗi đi qua `tError` + `message_key`.

**Never:** Không ngăn xếp hoàn tác, không `Mod+Z`, không thùng rác (AD-49; `change-rut-hoan-tac-cho-gop-tach` đã rút lời hứa ⌘Z cho gộp/tách). Không màn khôi phục Chương đã xoá mềm ở plan này. Không `window.confirm`. Không xoá nhiều Chương một lượt, dù panel chọn nhiều đã có cho gộp. Không tính lại ranh giới segment (AD-4), `split_source_text` không được gọi. Không đổi `chapter.status` để giả lập trạng thái đã xoá, và không thêm giá trị thứ năm vào `LifecycleStatus`. Không nút "xoá Tác phẩm" độc lập trong Library — đường duy nhất là xoá Chương cuối. Không thêm vị từ lọc vào 19 câu SQL liệt kê Chương: hình dạng đã chốt làm việc đó thành không cần thiết, và thêm vào là mời người sau tin rằng `chapter` có hàng vô hình.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Xoá Chương chưa dịch | 0 câu thoả `segment_has_translation_sql!()` | Hàng `chapter` và mọi hàng `segment`/`asset`/`chapter_position`/alignment/`review_*` của nó biến mất; `ord` còn lại liền 1..N | No error expected |
| Xoá Chương đã dịch | ≥1 câu có bản dịch | Hàng dời sang `chapter_deleted`, segment của nó mang `retired_at`; **0 byte** `target_text` bị xoá | No error expected |
| Chương đang mở | `OpenWork::chapter_id` là Chương bị xoá | Con trỏ sang Chương liền trước, liền sau nếu nó là Chương đầu; webview nạp lại Editor + Source | No error expected |
| Bấm lần đầu | chưa xác nhận | **0 hàng bị chạm**; nút đổi nhãn, kèm chữ *"không hoàn tác được"* ở hai đường phá dữ liệu | No error expected |
| Chương không còn | id đã xoá hoặc id lạ | **0 hàng bị chạm** | `err.chapter.not_found` `{chapter_id}` |
| Sau một lượt xoá mềm | tìm kiếm Library, một lượt xuất bản, bản reviewer | Chương không ra kết quả, không vào tệp xuất, không vào bản reviewer | No error expected |
| Tiến độ | sau một lượt xoá bất kỳ | `chapter_count` và tiến độ thẻ Library không đếm Chương đã xoá | No error expected |
| Chương cuối, chưa dịch | Tác phẩm còn 1 Chương, `chapter_deleted` rỗng | Xoá Chương **và** thư mục `.atproj`; Tác phẩm rời Library; app về Library mode | No error expected |
| Chương cuối, đã dịch | Tác phẩm còn 1 Chương, đã có bản dịch | Xoá mềm; Tác phẩm **ở lại** Library với 0 Chương hiện ra; thư mục còn trên đĩa | No error expected |
| Chương hiện cuối, còn Chương đã xoá mềm | `chapter` còn 1 hàng, `chapter_deleted` **không** rỗng | Xoá Chương đó theo luật riêng của nó; **thư mục KHÔNG bị xoá** | No error expected |
| Mồ côi sau khi xoá Tác phẩm | `.atproj` vừa bị xoá, chỉ mục dựng lại | Library hiện **0** mục mồ côi cho Tác phẩm đó | No error expected |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/chapter.rs:947-1038` (`merge_into_previous_tx`) -- khuôn DUY NHẤT đã xoá một hàng `chapter`: `asset`, `chapter_position`, alignment (`delete_alignment_of_chapter`), `review_row`/`review_chapter`, `DELETE FROM chapter`, renumber `ord`. Đường xoá hẳn chép đúng thứ tự này và thêm `DELETE FROM segment`. Cũng ở tệp này: `:117` `set_open_chapter`, `:344` `ChapterRow` (có `translated_count`), `:396` `fetch_chapter_rows`, `:1357`/`:1493` khuôn vỏ lệnh đồng bộ và async.
- `src-tauri/src/core/store/schema.rs:57` `segment_has_translation_sql!()` (bí danh `s`) -- phép thử "đã dịch". `:1625` `CHAPTER_DDL`, 7 cột mà `chapter_deleted` phải soi. `:2372-2613` `PROJECT_MIGRATIONS`, bước cuối `to_version: 45` ở `:2610` ⇒ bảng mới là bước **46**.
- `src-tauri/src/core/store/mod.rs:790` `Store::write` -- một job là một giao dịch.
- `segment.retired_at` (`schema.rs:1735`, AD-5) -- bia mộ có sẵn; vị từ `retired_at IS NULL` đã nằm trong mọi đường đọc segment, nên các segment của Chương đã xoá mềm tự biến mất khỏi mọi bề mặt. Khuôn tên `<verb>_at`, `NULL` là chưa xảy ra.
- **19 câu SQL liệt kê Chương, ở 9 tệp** (đo `(FROM|JOIN) chapter` trong `src-tauri/src` tại `7e6b6b1e`, bỏ 3 doc-comment và các câu tra theo `id`): `core/library/meta.rs:323,337,341` · `core/library/indexer.rs:2296` · `core/export/scope.rs:48` · `core/export/reviewer_copy.rs:331` · `commands/lifecycle.rs:67` · `commands/chapter.rs:286,291,405,579,811,816,897,969` · `commands/project/work_creation.rs:899` · `commands/segment/reading.rs:52,387` · `commands/project/mod.rs:3523`. Hình dạng đã chốt giữ cả 19 câu **không đổi**; danh sách này là chỗ để đối chứng rằng chúng vẫn đúng, không phải chỗ để sửa.
- `src-tauri/src/core/library/atproj.rs:215` `remove_folder` (`fs::remove_dir_all`) -- hôm nay chỉ lùi lại một thư mục vừa tạo lỗi; đường xoá Tác phẩm dùng lại nó.
- `src-tauri/src/core/library/indexer.rs:346,560` -- chỗ `Indexer::rebuild` so chỉ mục cũ với lượt quét và ghi mục mồ côi. `:920` đã có đường gọi `orphan_store::forget`; `core/library/orphan_store.rs:124` là hàm đó.
- `src-tauri/src/commands/project/mod.rs:3511,3523` -- `work.last_chapter_id` và lượt chọn Chương đầu làm con trỏ dự phòng.
- `src/modes/LibraryWorkPage.vue:633` hàng Chương, `:713` mở panel một Chương, `:795`/`:804` nút dời, `:947` mở gộp -- chỗ đặt nút xoá. `src/modes/libraryChapters.ts:264` `loadChapters`, `:768` `mergeSelectedChapters` -- khuôn store. `src/config/chapter.ts:389` `CMD_MERGE_CHAPTERS` -- khuôn vỏ `invoke`. `src/commands/index.ts:2418` -- khuôn đăng ký dispatch. `src/tokens/controls.css:37` `.ui-btn--danger`.
- Tiền lệ hỏi hai bước: `src/GlossaryManageOverlay.vue:765-779`, `src/TmManageOverlay.vue:740-747`. `src/i18n/index.ts:72` `tError`.
- Test ghim bậc di trú, bước 46 làm đỏ cả bốn: `src-tauri/tests/pinned_contract.rs:262` (`len() == 44`), `segment_contract.rs:932` (vector `to_version`), `chapter_origin_contract.rs:1075`, `segment_role_contract.rs:790`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/store/schema.rs` -- `CHAPTER_DELETED_DDL` soi 7 cột của `chapter` cộng `deleted_at`, và bước **46** của `PROJECT_MIGRATIONS`.
- [ ] `src-tauri/src/core/chapter_delete/` (mới) -- `delete_chapter_tx`: quyết hẳn-hay-mềm bằng `segment_has_translation_sql!()`, dọn theo khuôn `merge_into_previous_tx`, đánh số lại `ord`, dời con trỏ, và trả về việc Tác phẩm có còn Chương nào không.
- [ ] `src-tauri/src/commands/chapter.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/core/i18n/mod.rs` -- lệnh `delete_chapter`; khi không còn Chương nào ở cả hai bảng thì gọi `remove_folder`, dựng lại chỉ mục, rồi `orphan_store::forget`; các khoá lỗi mới.
- [ ] `src/config/chapter.ts`, `src/modes/libraryChapters.ts`, `src/commands/index.ts` -- vỏ `invoke`, hàm store, dispatch `library.chapter_delete.*`; về Library mode khi Tác phẩm biến mất.
- [ ] `src/modes/LibraryWorkPage.vue`, `src/i18n/vi.json` -- nút hai bước, ba câu xác nhận (xoá hẳn · xoá mềm · xoá cả Tác phẩm) chọn theo `translated_count` và số Chương còn lại.
- [ ] `src-tauri/tests/chapter_delete_contract.rs` (mới), `tests/frontend/libraryChapters.test.ts`, bốn test ghim bậc di trú -- phủ mọi hàng Matrix.
- [ ] `_bmad-output/initiative-auratranslate/deferred-work.md` -- một mục nợ: PRD chưa có FR cho xoá Chương và xoá Tác phẩm (FR160 cao nhất), cần một FR mới qua correct-course. **Chủ: Sofia.**

**Acceptance Criteria:**
- Given một Chương chưa có câu nào mang bản dịch, when xoá và xác nhận, then nó rời danh sách và không còn hàng nào của nó ở `segment`, `asset`, `chapter_position`, `review_*`, alignment.
- Given một Chương đã có bản dịch, when xoá và xác nhận, then số câu có `target_text` không rỗng của Chương đó trong kho không đổi, mà nó không còn trong danh sách, tìm kiếm và bản xuất.
- Given đã xoá một Chương ở giữa, when mở lại Tác phẩm, then `ord` các Chương còn lại liền 1..N và thẻ Library hiện `chapter_count` mới.
- Given Chương đang mở bị xoá, when lượt xoá xong, then Editor đứng ở Chương liền trước và `work.last_chapter_id` trỏ đúng nó.
- Given một Tác phẩm chỉ còn một Chương chưa dịch, when xoá và xác nhận, then thư mục `.atproj` không còn trên đĩa, Tác phẩm rời Library, và Library hiện 0 mục mồ côi.

## Implementation Notes

## Plan Change Log

## Review Triage Log

## Design Notes

Hai phát biểu của cùng luật "đã dịch" đang cùng tồn tại: `segment_has_translation_sql!()` (`schema.rs:57`) và `UNTRANSLATED_SQL` (`core/export/block_paragraphs.rs:17`). Lệnh xoá dùng bản ở `schema.rs`, vì `mark_chapter_started` đã dùng nó. `ChapterRow.translated_count` tính bằng bản export và chỉ là gợi ý cho CÂU xác nhận, không bao giờ là căn cứ quyết hẳn-hay-mềm. Plan này không hợp nhất hai bản.

Khoá API là Global-only, cố ý, vì *"xoá Tác phẩm thì mục keychain đi đâu"* không ai trả lời được (nợ ở `deferred-work.md`, spec 4.3). ⇒ Xoá một Tác phẩm không có hệ quả keychain nào.

## Verification

**Commands:**
- `npm run build`, rồi `cargo test --test chapter_delete_contract` -- xanh.
- `cargo test --test pinned_contract --test segment_contract --test chapter_origin_contract --test segment_role_contract` -- xanh sau khi sửa bậc 45 thành 46.
- `npx vitest run tests/frontend/libraryChapters.test.ts tests/frontend/libraryChapterBulkWrites.test.ts` -- xanh.
- `npm run check:i18n`, `npm run check:commands` -- xanh.
- Chạy trọn bộ một lần, vì diff chạm `PROJECT_MIGRATIONS` và `lib.rs` (AGENTS.md §Tests).
- Đối chứng đỏ: gỡ THẬT lượt `orphan_store::forget` khỏi đường xoá Tác phẩm, rồi chạy lại đúng ca mồ côi; đỏ phải vì mục mồ côi hiện ra, không vì lý do khác. Làm cùng khuôn cho lượt dời hàng sang `chapter_deleted`.

**Manual checks (if no CLI):**
- Ice, trong app thật: xoá một Chương rác, xoá một Chương đã dịch, xoá Chương cuối của một Tác phẩm thử, mở lại app, rồi xem danh sách Chương, thẻ Library, mục mồ côi và một lượt xuất bản.
