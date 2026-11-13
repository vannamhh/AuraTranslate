---
title: 'Chế độ đọc đọc cả Chương đang dịch và tạm ngưng'
type: 'feature'
ticket: '19'
created: '2026-11-13'
status: 'built'
baseline_revision: '0f2d173b36e3886227a03e7f7a7e4bb70c2883af'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: ['{project-root}/src-tauri/AGENTS.md', '{project-root}/src/AGENTS.md', '{project-root}/tests/AGENTS.md', '{project-root}/e2e/AGENTS.md']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Chế độ đọc dừng ở Chương chưa `done` đầu tiên, nên Ice không đọc kiểm tra liền mạch được Chương *Đang dịch*, mà phải đọc xong mới đặt *Đã xong* (PRD FR120 🔵 2026-10-27).

**Approach:** `read_reading_run` đi qua mọi Chương *Đã xong*/*Đang dịch*/*Tạm ngưng* liên tiếp và dừng ở Chương *Chưa bắt đầu* hoặc trạng thái lạ đầu tiên. Mỗi dãy câu liền nhau chưa có bản dịch thành một khối "… N câu chưa dịch" không mang nguyên văn; bấm vào khối thì mở Workspace ở câu đầu dãy.

## Boundaries & Constraints

**Always:**
- Đọc được = `LifecycleStatus` ∈ {Done, InProgress, Paused}. Một vị từ ở Rust (`core/lifecycle`) và một bản sao ở webview dùng cho cả nút "Đọc" của Library; trạng thái lạ vẫn dừng dãy.
- Câu chưa dịch = câu sống, không `is_omitted`, `target_text.trim()` rỗng. Câu `is_omitted` không có mặt nên không tính và không cắt dãy.
- Dãy liền nhau gộp qua biên đoạn. Một đoạn bị dãy chen giữa thì tách thành hai khối đoạn.
- `source_text` của câu chưa dịch không đi qua dây, kể cả khi bật Song ngữ.
- Dấu câu chưa xác nhận giữ nguyên ở mọi Chương được đọc.
- Ảnh neo sau một câu chưa dịch vẫn hiện, đặt ngay sau khối của dãy đó.
- Quyết định (Ice 2026-11-13): dòng mờ áp ở mọi Chương được đọc, kể cả *Đã xong*. Nút "Đọc" ở trang Tác phẩm Library mở theo cùng vị từ, nằm trong story này. Giữ một plan, chia pha Rust · Webview · Tests, mỗi pha một agent mới.

**Never:** tự đổi trạng thái Chương; hiện nguyên văn để lấp chỗ trống; để `reading.mark_aimed` đánh dấu một câu chưa dịch; sửa `EXPERIENCE.md` (thuộc Sally, đã ghi Source conflict trong epic).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error |
|---|---|---|---|
| Đọc qua | mở Ch1 done; Ch2 in_progress, Ch3 paused, Ch4 not_started | dãy Ch1–Ch3, mốc biên chỉ Ch4 | — |
| Mở Chương chưa bắt đầu | Chương đang mở not_started | dãy rỗng, mốc biên trên chính nó | — |
| Trạng thái lạ | Ch2 `finished` | dừng trước Ch2, mốc ghi nguyên chuỗi | — |
| Đuôi chưa dịch | 10 câu, 6 cuối rỗng, trải hai đoạn | 1 khối, 6 id, câu đầu là câu thứ 5 | — |
| Cắt bỏ giữa dãy | U, O(omitted), U | 1 khối, 2 id | — |
| Khoảng trắng | target "  \n" | tính là chưa dịch | — |
| Mất hết chữ | Chương in_progress, mọi câu rỗng | chỉ một khối, trạng thái trang `content` | — |
| Bấm khối | click hoặc Enter | Workspace mở Chương đó ở câu đầu dãy | chặn flush ⇒ ở lại Chế độ đọc như `open_aimed` |

</frozen-after-approval>

## Code Map

Rust (`src-tauri/`):
- `src/commands/segment/reading.rs` -- `read_reading_run` :327. Luật dừng :358-369 (`from_wire == Some(Done)`). Dựng câu :396-412 trên `paragraphs_in_translation` (`core/segment/reading.rs:53`) và `strip_role_segments`. Kiểu: `ReadingSegment` :15, `ReadingParagraph` :155, `ReadingChapter` :165, `ReadingFrontierKind` :250 (`NextNotDone` "next-not-done", `EndOfWork`), `ReadingFrontierChapter` :270, `ReadingFrontier` :284. Doc-comment nêu luật cũ: :8-11, :251-261, :280, :314-321, :358-360, cùng `wire.rs:54`, `core/segment/omit.rs:34`, `chapter_read.rs:222,232,272`. Ảnh: `resolve_chapter_images` :418, ràng buộc `after_segment_id` :222.
- `src/core/lifecycle/mod.rs` -- `LifecycleStatus` :40, `from_wire` :73; vị từ đọc được đặt ở đây.
- Vị từ "khác rỗng" trong Rust hiện là `!target_text.trim().is_empty()` rời rạc (`proofread.rs:208`). Không dùng được macro SQL `segment_has_translation_sql!` vì câu đã nạp sẵn.
- Ghim: `tests/ipc_contract.rs:1700` (bộ khoá, `next-not-done` :1886, `end-of-work` :1918). `tests/segment_contract.rs` mục :8060, helper :8077-8162. Ba ca đổi kỳ vọng :8355, :8431, :8529. Bench bị ignore :8921 khẳng định `NextNotDone` :8997/:9012. Còn `segment_image_contract.rs` :146-457 và `reading_marks_contract.rs` :11/:55/:150.
- Cổng: `segment_boundary.rs:382` (dưới vỏ chỉ được `&Transaction`/`ReadHandle`), `config_invariants.rs:2442` (đếm vỏ, không đổi nếu không thêm vỏ).

Webview (`src/`):
- `config/reading.ts` -- các kiểu :34-…; validator chặt từ `isReadingSegment` ~182 đến `isReadingRun` ~319, kind ~281, frontier ~302. `readReadingRun` ~345 trả `ipc.unknown` khi lệch hình.
- `modes/readingState.ts` -- `readingStatusKind` :105-127; `loadReadingChapterProgress` :85 chỉ đếm `done`; `findReadingSegment` :189; `openAimedReadingSegment` :250 là khuôn mở Workspace; `openFrontierInWorkspace` :513.
- `modes/ReadingMode.vue` -- vòng đoạn :823-903, `source-note` :826 nối `source_text` cả đoạn, `.unconfirmed` :854/:1206, frontier :914-955. `progress` :507 tính `min(ord, done)/done`. Dòng mờ dùng khuôn `.chapter-note` :1268 (`--color-on-surface-variant`; Kiểm D cấm `opacity`).
- `commands/index.ts` -- `reading.open_aimed` :2907; khuôn `pointer: {kind:'aim', …}` ~2945. `check:commands` buộc `@click` là một `dispatch('<id>')` và mỗi lệnh có nhãn.
- `modes/LibraryWorkPage.vue:693` và `modes/libraryChapterSelection.ts:173` chỉ cho "Đọc" khi `status === 'done'`.
- `i18n/vi.json:1505-1535` -- `status_frontier_only`, `frontier_range_empty`, `frontier_end_of_work` đang nói "chưa dịch xong"/"đã dịch xong".
- Vitest dựng `ReadingRun`: `tests/frontend/reading{Frontier,FrontierDom,Unconfirmed,State,MarksRemoval,ModeImages,Toc,Toolbar,Marks,LookupPopup}.test.ts`.
- e2e: `e2e/specs/story-5-12-reading-frontier.e2e.mjs` (Ch2 in_progress chưa dịch là mốc, các khẳng định ③④ sẽ sai). `story-12-34-work-page-read-chapter.e2e.mjs:81-99` khẳng định hàng in_progress không có "Đọc".

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/lifecycle/mod.rs` -- thêm `LifecycleStatus::is_readable()` -- để luật đọc khai một chỗ.
- [ ] `src-tauri/src/commands/segment/reading.rs` -- đổi luật dừng sang `is_readable`. `ReadingChapter.paragraphs` thành `blocks: Vec<ReadingBlock>` với tag `kind`: `paragraph {segments}` | `untranslated {segment_ids}`. `NextNotDone` thành `NextUnreadable` "next-unreadable". Sửa mọi doc-comment nêu luật cũ.
- [ ] `src-tauri/tests/segment_contract.rs`, `ipc_contract.rs`, `segment_image_contract.rs`, `reading_marks_contract.rs` -- sửa theo hình mới và thêm ca cho mỗi hàng matrix phía Rust, gồm một ca serialize xác nhận chuỗi nguyên văn của câu chưa dịch vắng trong JSON. Gỡ thật từng nhánh (vị từ đọc được, gộp dãy, lọc cắt bỏ, chặn nguyên văn) thì đúng ca của nhánh đó đỏ.
- [ ] `src/config/reading.ts` -- kiểu và validator theo hình mới; kind lạ vẫn bị từ chối.
- [ ] `src/config/lifecycle.ts` (hoặc chỗ đặt vị từ trạng thái hiện có) -- `isReadableChapterStatus`. Dùng nó ở `LibraryWorkPage.vue:693`, `libraryChapterSelection.ts:173` và `loadReadingChapterProgress`.
- [ ] `src/modes/readingState.ts` + `src/commands/index.ts` -- lệnh mới `reading.open_untranslated` (pointer `aim`) mở Workspace ở `segment_ids[0]` theo khuôn `openAimedReadingSegment`. Aim của khối tách khỏi aim câu, để `reading.mark_aimed` không chạm câu chưa dịch. `readingStatusKind` coi khối chưa dịch là nội dung.
- [ ] `src/modes/ReadingMode.vue` + `src/i18n/vi.json` -- khối là một `<button>` mờ "… {count} câu chưa dịch" (`mode.reading.untranslated_run`), `@click="dispatch('reading.open_untranslated')"`. `source-note` chỉ nối câu của khối đoạn. Ảnh neo trong dãy hiện sau khối. Sửa chữ ba khoá luật cũ cho đúng luật mới.
- [ ] `tests/frontend/*` -- sửa fixture các tệp reading* theo `blocks`, thêm ca: khối hiện đúng số câu, không có nguyên văn khi bật Song ngữ, bấm khối mở đúng câu, `mark_aimed` bỏ qua khối, nút "Đọc" của Library hiện cho in_progress/paused và ẩn cho not_started.
- [ ] `e2e/specs/story-5-12-reading-frontier.e2e.mjs`, `story-12-34-work-page-read-chapter.e2e.mjs` -- sửa theo luật mới: Ch2 in_progress được đọc và hiện khối, Ch3 not_started làm mốc, bấm khối vào Workspace đúng câu.

**Acceptance Criteria:**
- Given một Tác phẩm có Ch1 Đang dịch đã dịch một phần, when Ice bấm tab Chế độ đọc, then thấy bản dịch của Ch1 và một dòng mờ cho phần còn lại, không thấy mốc "chưa đọc được" ở Ch1.
- Given trang Tác phẩm Library, when một Chương Đang dịch hoặc Tạm ngưng được chọn, then nút "Đọc" có mặt.
- Given bật Song ngữ, when trang có khối chưa dịch, then không ký tự nào của nguyên văn câu chưa dịch nằm trong DOM.

## Design Notes

- Khối mang `segment_ids` chứ không mang `count` + `first_segment_id`: số câu và câu đầu suy ra được từ danh sách, và webview đặt ảnh neo trong dãy mà không cần dây biết gì về ảnh.
- Tên kind mới là `next-unreadable`, không phải `next-not-started`, vì trạng thái lạ cũng dừng dãy ở cùng mốc; kiểu phải nói thật.
- Thanh tiến độ đếm Chương đọc được. Nhãn "{done_count} Chương đã dịch" vẫn đúng nghĩa với FR120 "phần đã dịch".

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test segment_contract --test ipc_contract --test segment_image_contract --test reading_marks_contract --test segment_boundary` -- xanh.
- `npx vitest run tests/frontend/reading` cộng các tệp library chạm nút "Đọc" -- xanh, thoát 0, không có dòng Errors.
- `npm run check:commands && npm run check:i18n && npm run check:tokens` -- xanh (`check:i18n` từng đỏ sẵn ở `AiTranslationPanel.vue:655`, đối chiếu với HEAD sạch).
- `npm run test:e2e -- --spec e2e/specs/story-5-12-reading-frontier.e2e.mjs --spec e2e/specs/story-12-34-work-page-read-chapter.e2e.mjs` -- xanh, chạy ngoài sandbox.

**Manual checks:**
- Ice mở Tác phẩm "test" như ảnh chụp, đọc qua Chương Đang dịch, bấm dòng mờ vào Workspace. Ghi mục nợ `Chủ: Epic 5` trước khi epic đóng.

## Implementation Notes

- Dây: `ReadingChapter.blocks` thay `paragraphs`, tag `kind` = `paragraph {segments}` | `untranslated {segment_ids}`. Frontier `next-not-done` thành `next-unreadable`.
- `make_done` trong `segment_contract.rs` nay dịch mọi câu, vì câu rỗng đã thành khối. Ca cần câu chưa dịch phải xoá chữ sau `make_done`.
- Pha 3 phát hiện `libraryChapters.ts::openCurrentChapterInReading` còn chặn ở `done`, nên nút "Đọc" của Library thành no-op im lặng với Chương Đang dịch. Đã sửa theo cùng vị từ.
- Aim của khối đặt cả ở `@mousedown`: cú bấm thật trong WKWebView qua driver không phát `mouseenter`/`focusin` trên nút. `check-commands.mjs` thêm hàng `HANDLER_TABLE` cho handler đó (Kiểm K).
- `reading.open_untranslated` không khai `pointer`: `@click` là một `dispatch` trần nên Kiểm M đã thoả.
- e2e 12-34 xanh trên app thật. e2e 5-12 đỏ ở bước gõ ("flush … không hoàn tất sau 30000 ms") vì Vite của `tauri dev` đang chạy trên :1420 mang tem `?t=` cũ. `editor-typing-flush` và bản HEAD của spec cũng đỏ y như vậy. Bản sao tạm của spec dùng lối chờ đọc đĩa thì xanh, nhưng tệp thật CHƯA được chạy xanh.

## Plan Change Log

## Review Triage Log

- Lượt quick 1: 0 high · 0 medium · 4 low · 3 false.
- Thiếu ca Rust cho đích toàn khoảng trắng — false: `a_paragraph_interrupted_by_an_untranslated_stretch_splits_into_two_paragraph_blocks` đặt `" \n "` qua `set_target_directly`.
- Gỡ riêng `InProgress`/`Paused` khỏi `is_readable` không có ca đỏ — false: ca đọc-qua chứa cả hai, pha 1 gỡ `InProgress` thì ca đỏ, vitest gỡ `paused` thì đỏ.
- `reading.open_untranslated` thiếu `pointer: aim` — false: `check:commands` Kiểm M thoả bằng `@click` dispatch, thoát 0.
- `?? 0` trong handler aim của khối — low, patch: hàm aim nhận `segment_ids`, id không biết là `null` thay vì 0.
- Thanh tiến độ so `ord` với số đếm — low, reject: lệch thang đo có từ trước story này, sửa cần thiết kế lại thanh.
- Doc-comment mới bằng tiếng Việt, id story trong e2e — low, patch: ba doc-comment mới đổi sang một dòng tiếng Anh, bỏ "+ 5-19". Dòng sửa trong các khối doc cũ giữ nguyên (viết lại cả khối là dọn hàng loạt).
- Chữ `status_frontier_only`/`frontier_range_empty` nói "chưa bắt đầu dịch" cả khi trạng thái lạ — low, patch: đổi thành "chưa đọc được".
