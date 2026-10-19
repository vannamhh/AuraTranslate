---
title: 'Danh sách Chương: nhóm trạng thái, chọn nhiều, thanh hành động'
type: 'feature'
ticket: '4'
created: '2026-10-14'
status: 'built'
baseline_revision: 'c521fd6a436d2bd3089a573ece56677a73295800'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Danh sách Chương là một khối phẳng trong cột Library: một con trỏ, không nhóm, không lọc/tìm/sắp, không chọn nhiều. Gộp chỉ gộp vào Chương trước, từng cặp một. Tách chỉ làm được trong Editor. Chương đã xong không nói còn bao nhiêu câu chưa xác nhận (FR138; a:102, a:103, a:105, a:107, a:108, a:110; A2–A6).

**Approach:** Mở một Tác phẩm thì Library chuyển sang trang Tác phẩm riêng theo `chapter-list.html`: đầu trang, ô trạng thái, dải lọc, danh sách nhóm theo trạng thái, chọn nhiều, thanh hành động, màn gộp, hộp tách. Rust bổ sung các số đếm theo Chương, giá trị trạng thái suy ra, và hai lệnh ghi nhiều Chương, mỗi lệnh chạy trong một giao dịch.

## Boundaries & Constraints

**Always:**
- Gộp và tách Chương chỉ đổi `chapter_id`/`ord` (AD-32). Mỗi lệnh ghi chạy `Indexer::rebuild` sau giao dịch (khuôn `lifecycle::finish_lifecycle_write`). Flush Editor trước, như `beginChapterReorg`.
- Lệnh ghi nhiều Chương là một giao dịch: hỏng thì không Chương nào đổi.
- Rust trả số, webview dựng chữ (AD-21). Chữ chỉ đi qua `vi.json`, khoá lỗi mới qua `message_keys!`.
- Định nghĩa "chưa xác nhận" và "đã dịch" dùng lại mẩu SQL của `core/export/scope.rs`. Câu `is_omitted` không tính, nên số trên danh sách khớp số của màn Xuất.
- Đóng lớp phủ trả tiêu điểm về nút đã mở nó (UX-DR17). `@click` gọi đúng một `dispatch`.
- `Nhập bản reviewer` và `Nối tay` ở lại `Công cụ ▾`. Chỉ `Xuất` rời menu đó.

**Never:**
- Không tay kéo ⠿, không đổi tên hàng loạt, không ngày đặt ghi đè, không hoàn tác gộp bằng ⌘Z. Cả bốn ghi nợ `Chủ: Epic 12`.
- Màn gộp không ghi "Hoàn tác được bằng ⌘Z" và không ghi "Ranh giới câu được tính lại ở chỗ nối": câu đầu app không làm được, câu sau trái AD-32. Màn gộp cũng không đoán trạng thái Chương sau gộp.
- Không nối hành vi phạm vi vào `Xuất…` và `Nhập bản reviewer`: hai nút dispatch lệnh có sẵn, story 17 và 16 nối phần còn lại.
- Không migration. Không gộp Chương không liền nhau.

## Decisions (Ice, 2026-10-14)

- Giữ một story, không tách 12.4b.
- Trang Tác phẩm riêng theo mockup (a:102), có lối quay về lưới Library.
- Gộp nhiều Chương bằng lệnh Rust mới `merge_chapters`, một giao dịch, chỉ nhận Chương liền nhau.
- Ô trạng thái hiện giá trị suy ra khi đang ghi đè (a:105). Ngày đặt ghi nợ.
- Dời lên và Dời xuống vào thanh hành động khi chọn đúng 1 Chương. Kéo thả ghi nợ.
- `Tách…` mở hộp liệt kê câu nguồn để chọn câu mở đầu Chương mới.
- `Đổi tên…` khi chọn đúng 1 Chương. Đổi tên hàng loạt ghi nợ.
- Sắp theo: thứ tự Chương (mặc định) · sửa gần nhất.
- Mốc "sửa gần nhất" là thời điểm muộn hơn trong hai mốc: `chapter.updated_at` và lần xác nhận câu cuối cùng (`segment_version.created_at`). Bản nháp chưa xác nhận không làm mốc đổi. Ice chọn sau review lượt 1, vì `chapter.updated_at` không đổi khi dịch câu.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Gộp ba | chọn Chương 88, 89, 90 liền nhau | màn gộp liệt kê theo thứ tự; xác nhận → còn một Chương 88 mang tên Chương 88, số câu bằng tổng; các Chương sau dồn số | — |
| Gộp không liền | chọn 3 và 7 | nút Gộp tắt; lệnh gọi thẳng vẫn từ chối | `err.chapter.merge_not_contiguous`, không đổi gì |
| Gộp hỏng giữa chừng | lỗi ở Chương thứ hai | không Chương nào đổi | thông báo lỗi trên trang |
| Đã xong còn câu | Chương `done`, 4 câu không `confirmed`, không `is_omitted` | hàng ghi "… câu · 4 câu chưa xác nhận" | — |
| Đã xong sạch | `done`, mọi câu đã xác nhận hoặc bỏ qua | không có đoạn "chưa xác nhận" | — |
| Đổi trạng thái | chọn 3, đặt Tạm ngưng | cả ba vào nhóm Tạm ngưng | một giao dịch; hỏng thì không Chương nào đổi |
| Lọc rỗng | tìm "xyz" | ghi "không có Chương khớp" sau khi `libraryChaptersHaveLoaded` | — |
| Tách | 1 Chương, chọn câu đầu | nút xác nhận tắt | `err.chapter.split_leaves_empty` nếu gọi thẳng |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/chapter.rs` -- `ChapterRow` L342 và `fetch_chapter_rows` L384: thêm `translated_count`, `unconfirmed_count`, `updated_at`, đếm bằng một truy vấn có GROUP BY. `rename_chapter` cũng trả `Vec<ChapterRow>`. `merge_chapter_into_previous` L843: tách phần thân thành hàm nhận `&Transaction` để `merge_chapters` gọi lặp trong một giao dịch. `split_chapter_at_segment` L1053 đòi Chương đang mở. `*_indexed` L1225–1285, `mod wire` từ L1290.
- `src-tauri/src/commands/lifecycle.rs` -- `WorkLifecycle` (`status`, `status_is_override`; doc L36 nói các trường khớp tên `WorkMeta`): thêm giá trị suy ra mà không phá điều doc đó khẳng định. `set_chapter_status` L146 làm khuôn cho lệnh đặt trạng thái nhiều Chương.
- `src-tauri/src/core/export/scope.rs`, `core/export/block_paragraphs.rs:14` (`UNTRANSLATED_SQL`), `core/segment/omit.rs:76` (`IN_TRANSLATION_SQL`) -- dùng lại, không sao chép.
- `src-tauri/src/lib.rs` ~L1189 -- đăng ký lệnh mới. Nâng `REGISTERED_COMMAND_FLOOR` (`tests/ipc_argument_contract.rs`) và sửa hình đã đóng băng trong `tests/ipc_contract.rs`.
- `src/modes/LibraryMode.vue` (2.040 dòng) -- khối `.chapters-block` ~L1123–1330 và khối trạng thái Tác phẩm. Tách ra component mới cho trang Tác phẩm, không nhồi thêm vào tệp này.
- `src/modes/libraryChapters.ts` -- state hiện có: con trỏ, `chapterWindow` L605, thao tác tổ chức lại, `beginChapterReorg`, `mergeCurrentChapterUp` L457 (danh sách reset sau gộp). Thêm tập chọn, lọc/tìm/sắp, nhóm. Cửa sổ ảo hoá chạy trên danh sách phẳng gồm cả hàng tiêu đề nhóm. `resetLibraryChapters` reset cả state mới (`check:panel-refs`).
- `src/config/chapter.ts` L100/138 -- `ChapterRow` và `isChapterRow`: thêm trường mới.
- `src/commands/index.ts` (~2069 rename, ~2102 merge_up, `export.open` 3435, `export.reviewer_import.open` ~3471) và `main.ts` -- lệnh mới. Nâng `DISPATCH_FLOOR` (`scripts/check-commands.mjs:247`).
- `src/TitlebarToolsMenu.vue` L177–183 -- gỡ mục `export.open`. Đích trả tiêu điểm `[data-export-open]` của `ExportOverlay` chuyển sang nút `Xuất…` của thanh hành động.
- `src/modes/ReadingMode.vue` -- tiền lệ FR120 (lớp `.unconfirmed`).
- `_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/chapter-list.html` -- nguồn nhãn: nhóm, hàng, dải lọc, thanh hành động, màn gộp. Đọc lại khi dựng.
- Test sẽ dời: `tests/frontend/libraryChapters.test.ts`, `tests/frontend/titlebarToolsMenu.test.ts`, `e2e/specs/story-5-7-open-chapter.e2e.mjs`, `e2e/specs/story-5-8-reorganise-chapters.e2e.mjs`, `e2e/specs/titlebar-tools.e2e.mjs` (L76 assert mục cuối là `export.open`), `e2e/specs/library-mode-switch-focus.e2e.mjs`. Giữ các hook `data-library-chapter-*` khi có thể.
- Test dời thêm, thấy lúc cài đặt: `src-tauri/tests/config_invariants.rs`, `lifecycle_contract.rs`, `naming_boundary.rs`, `project_contract.rs`, `reviewer_import_contract.rs`, `ipc_contract.rs`, `ipc_argument_contract.rs`; `tests/frontend/exportBlock.test.ts`, `exportDocx.test.ts`, `exportImages.test.ts`, `exportScope.test.ts`, `exportText.test.ts`, `libraryWorks.test.ts`, `readingState.test.ts`; `e2e/specs/story-5-4-*.e2e.mjs`, `story-12-2-*.e2e.mjs`, `e2e/support/pointer.mjs`. Test mới: `tests/frontend/chapterWireGuards.test.ts`, `libraryChapterBulkWrites.test.ts`, `libraryChapterSelection.test.ts`; `e2e/specs/story-12-4-work-page-chapter-list.e2e.mjs`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/commands/chapter.rs` -- ba trường đếm mới; `merge_chapters(chapter_ids)` (≥2, liền nhau theo `ord`, một giao dịch, reindex); khoá lỗi mới -- FR138, A6
- [ ] `src-tauri/src/commands/lifecycle.rs` -- trạng thái suy ra khi đang ghi đè; `set_chapters_status(chapter_ids, status)` một giao dịch -- a:105, a:110
- [ ] `src-tauri/src/lib.rs` + `src-tauri/tests/*_contract.rs` -- đăng ký lệnh; ca: gộp ba, gộp không liền bị từ chối và không đổi gì, lỗi giữa chừng không đổi gì, số đếm trên Chương có câu `is_omitted`, đặt trạng thái nhiều Chương -- canh đường ghi
- [ ] `src/config/chapter.ts`, `src/config/lifecycle.ts` -- adapter và guard cho trường và lệnh mới
- [ ] `src/modes/libraryChapters.ts` + `src/modes/libraryChapterSelection.ts` (mới) -- tập chọn (bấm, Mod+bấm, Shift+bấm, Esc bỏ chọn), lọc theo nhóm, tìm theo tên hoặc số, sắp, nhóm (Chưa bắt đầu gấp mặc định), quy tắc bật nút
- [ ] `src/modes/LibraryWorkPage.vue` (mới) + `LibraryMode.vue` -- đầu trang (a:103), ô trạng thái (a:105), dải lọc có đếm (A2, A3), nhóm (a:107), hàng (a:108: số · tên · tiến độ câu · chip · thời điểm), thanh hành động (a:110), nút về Library
- [ ] `src/ChapterMergeOverlay.vue`, `src/ChapterSplitOverlay.vue` (mới, khuôn `ExportOverlay.vue` + `exportState.ts`) -- A6, A5; mỗi cái có `reset*` và trả tiêu điểm
- [ ] `src/commands/index.ts`, `main.ts`, `src/i18n/vi.json`, `src/TitlebarToolsMenu.vue` -- lệnh, nhãn, gỡ Xuất tạm
- [ ] `tests/frontend/*` -- state chọn/lọc/nhóm/bật nút; dời các test cũ
- [ ] `e2e/specs/story-12-4-*.e2e.mjs` + dời 5-7, 5-8, titlebar-tools -- chọn ba Chương rồi gộp qua thanh hành động; Chương đã xong hiện số câu chưa xác nhận
- [ ] `deferred-work.md` -- bốn mục nợ `Chủ: Epic 12`: kéo thả, đổi tên hàng loạt, ngày đặt ghi đè, ⌘Z hoàn tác gộp

**Acceptance Criteria:**
- Given một Tác phẩm đang mở ở Library, when người dùng mở nó, then trang Tác phẩm thay lưới, với bốn nhóm theo thứ tự Đang dịch · Tạm ngưng · Đã xong · Chưa bắt đầu (nhóm cuối gấp), và nút về Library trả tiêu điểm về thẻ Tác phẩm.
- Given ba Chương liền nhau, when chọn cả ba rồi bấm `Gộp thành một Chương` và xác nhận ở màn gộp, then danh sách còn đúng một Chương thay cho ba.
- Given một Chương `done` có 4 câu chưa xác nhận, when danh sách hiện, then hàng ghi đúng số 4 mà Rust trả về.
- Given `Công cụ ▾` mở, when liệt kê mục, then không còn `Xuất`, và `Xuất…` trên thanh hành động mở màn Xuất rồi trả tiêu điểm về chính nút đó.
- Given Tác phẩm đang ghi đè trạng thái, when xem ô trạng thái, then ô ghi cả giá trị đang dùng lẫn giá trị suy ra.

## Implementation Notes

- `ChapterRow` có thêm `in_translation_count`, ngoài ba trường plan nêu. Lý do: `segment_count` tính cả câu `is_omitted`, nên mẫu số tiến độ "đã dịch / tổng" phải là số câu không bỏ qua.
- `WorkLifecycle.derived_status` tính từ `chapter.status` cả khi Tác phẩm đang ghi đè. `From<&WorkMeta>` đổi thành `from_meta`.
- Lệnh IPC `merge_chapter_into_previous` và lệnh webview `library.chapter_merge_up` bị gỡ, vì không còn nút nào gọi. Hàm thuần vẫn còn: `merge_chapters` gọi lặp phần thân `merge_into_previous_tx` trong một giao dịch.
- Mọi thao tác ghi lên một Chương (đổi tên, dời, sửa xuất xứ) giờ đòi chọn đúng một Chương; con trỏ thôi đủ. Chọn hàng bắt ở `@mousedown`, không ở `@click`.
- Hộp tách mở Chương đích vào Editor để đọc câu, và để Chương mở như thế khi bấm huỷ, vì `split_chapter_at_segment` đòi Chương đang mở.
- Ở e2e, Shift/Meta + nhấn chuột là `mousedown` phát trong trang (`modifiedMouseDown`): giữ phím thật trên WKWebView không mở rộng được tập chọn.
- Sàn dân số được nâng vì số tệp tăng (check-commands, check-i18n, check-tokens, check-layout, `naming_boundary`, `segment_boundary`). Không hạ sàn nào.
- `library-mode-switch-focus.e2e.mjs`: ca 2–4 đỏ ở hàng rào panel-reset, và đỏ y hệt khi chạy frontend của HEAD trên cùng binary. Chạy riêng từng ca thì xanh. Không do story này.
- ⌘E đổi trạng thái chưa gán; menu đổi trạng thái chỉ mở bằng nút.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick, 2026-10-14): 12 phát hiện — medium 6 · low 6 · false 0 · maybe-false 0. Patch 11; intent_gap 1, Ice quyết rồi sửa tại chỗ, không revert. Patch giao cho một agent mới, vì agent cài đặt đã vượt ~250k context. Ngoài review, trọn bộ `cargo test` bắt được sàn `WEBVIEW_FLOOR` của `segment_boundary` trôi; đã nâng lên 153.

- medium · patch — Lọc, tìm, gấp nhóm vẫn giữ Chương đã chọn mà bị ẩn, nên thanh hành động tác động lên Chương người dùng không thấy. Sửa: tỉa tập chọn theo hàng đang hiện.
- medium · patch — `moveCurrentChapter` nạp lại danh sách nhưng không gọi `syncCursorToChapter`, nên con trỏ chỉ sang Chương bên cạnh.
- low · patch — Enter mở Chương ở hàng con trỏ dù đang chọn ≥2 và nút `Mở Chương` đang tắt. Sửa: Enter theo cùng quy tắc với nút.
- medium · intent_gap — "Sửa gần nhất" và cột thời điểm đọc `chapter.updated_at`, mà cột này chỉ đổi khi đổi tên, đổi trạng thái hoặc gộp; 13 câu `UPDATE segment` không chạm `updated_at` nào. Tiền đề của phương án Ice chọn là sai, nên đưa Ice quyết. Ice chọn thêm mốc xác nhận câu. Đã sửa `fetch_chapter_rows` và thêm một test, đối chứng bằng phép gỡ thật.
- medium · patch — Màn gộp ghi "không câu nào mất" và "dùng Tách… để chia lại", trong khi `merge_into_previous_tx` xoá tên, xuất xứ, vị trí đã lưu và bản reviewer của các Chương bị gộp. Sửa: viết lại chữ cho đúng.
- medium · patch — `chapters_not_loaded` đổi thành "Đang tải…", nhưng cờ đã tải còn `false` cả khi không có cầu IPC, nên chữ báo đang tải mãi (rỗng im lặng). Sửa: dùng câu trung tính.
- low · patch — Chú thích mang mã story hoặc lịch sử sự cố ở `commands/index.ts`, các spec e2e 5-4, 5-7, 5-8, 12-4 và `pointer.mjs`.
- low · patch — `merge_into_previous_tx` chuyển chỗ kéo theo chú thích có ngày, mã story và 🔵. Đó là dòng đang đổi, nên xoá theo AGENTS.md.
- low · patch — Chú thích ở `export.open` còn nói nút thanh tiêu đề dispatch lệnh này, nhưng nút đó đã bị gỡ.
- medium · patch — AC1 (về Library trả tiêu điểm về thẻ Tác phẩm) không có test canh. Mã focus vào `[data-library-open-work]` đầu tiên, mà nút này chỉ hiện ở thẻ dưới con trỏ, sau khi danh sách Tác phẩm nạp lại.
- low · patch — Ma trận "mang tên Chương 88" không có assert nào. Một test số đếm kèm phép thử `merge_chapters` không liên quan. Nhãn tổng câu dùng `in_translation_count` mà không ca nào phân biệt được nó với `segment_count`.
- low · patch — Chuỗi SQL trong `set_chapters_status` dính một dải khoảng trắng do mất dấu nối dòng.

## Design Notes

- Bật nút: Gộp khi chọn ≥2 Chương liền nhau. Tách… khi chọn 1 Chương có ≥2 câu. Đổi tên…, Dời lên, Dời xuống khi chọn đúng 1. Đổi trạng thái và Xuất… khi chọn ≥1. Nhập bản reviewer luôn bật. Phím: ↵ mở Chương ở Workspace, F2 đổi tên, Esc bỏ chọn. ⌘E đổi trạng thái chỉ gán khi không trùng phím đã khai trong `src/commands/index.ts`.
- Thanh hành động dùng lại nhãn mockup: "N Chương đang chọn" · `Gộp thành một Chương` · `Tách…` · `Đổi tên…` · `Dời lên` · `Dời xuống` · `Đổi trạng thái` · `Xuất…` · `Nhập bản reviewer`.

## Verification

**Commands:**
- `npm run test:story 12-4` -- expected: các test được chọn xanh
- `npm run build && cargo test --manifest-path src-tauri/Cargo.toml` -- expected: xanh trọn bộ (đổi `lib.rs` là dây dùng chung)
- `npm run test:e2e -- --spec e2e/specs/story-12-4-*.e2e.mjs` -- expected: xanh

**Manual checks:**
- Ice: gộp, tách, đổi trạng thái trên một Tác phẩm 2.000 Chương ở bản đóng gói (mục `Chủ: Epic 12`).
