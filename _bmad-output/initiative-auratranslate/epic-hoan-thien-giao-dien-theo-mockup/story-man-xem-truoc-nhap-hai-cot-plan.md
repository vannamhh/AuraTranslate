---
title: 'Màn xem trước nhập hai cột'
type: 'feature'
ticket: '6'
created: '2026-10-14'
status: 'built'
baseline_revision: 'b6137b0ebbe82fa68c117dfc2ccb9e2a8c15c4b8'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/change-hoan-thien-giao-dien-theo-mockup/bang-lech-a.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ba màn xem trước nhập (tách Chương, URL, song ngữ) là lớp phủ modal xếp tầng một cột. Mockup vẽ chúng thành màn đầy đủ hai cột. Màn không báo trước số ứng viên Glossary sẽ vào bảng chờ (FR137). Ngoài ra một tệp song ngữ còn một hàng lệch là bị chặn nhập trọn, dù 2.000 cặp khác đã khớp.

**Approach:** Thay ba lớp phủ và bước dán link bằng các màn thay chỗ nội dung Library, theo `library-and-import.html`, `web-import.html` và `bilingual-import.html`. Thêm ước lượng ứng viên chỉ đọc cho các màn văn xuôi. Thêm lối "Nhập luôn, nối sau": hàng lệch nhập thành một segment, mang dấu "chưa nối" mà lưới và danh sách Chương hiện ra. Dòng: a:43, a:44, a:47, a:48, a:52, a:59–a:62, a:64, a:66, a:69, a:84, a:88, a:91; A1.

## Boundaries & Constraints

**Always:**
- Xem trước giữ nguyên trong bộ nhớ, đúng thứ tự `PIPELINE_ORDER` (AD-39). Ước lượng không ghi `glossary_candidate` hay `glossary_entry` (AD-20) và không gọi `ImportScanGeneration::next()`.
- Ước lượng tính theo đúng ngưỡng từng Chương như quét thật, lấy hợp nguồn qua mọi Chương, trừ Glossary toàn cục; khi nhập nối vào Tác phẩm có sẵn thì trừ cả Glossary và bảng chờ `import_scan` của Tác phẩm đó.
- `confirm_bilingual_import` mặc định vẫn từ chối hàng lệch; chỉ một cờ tường minh mới cho nhập hàng lệch. Hàng lệch ghi `target_text` cùng baseline và cột xuất xứ như cặp đã khớp (AD-47), trạng thái chưa xác nhận (FR115).
- Dấu "chưa nối" mất khi segment về hưu (gộp, tách, tái tách) hoặc được xác nhận, trong cùng giao dịch với thao tác đó.
- Mỗi lối vào cũ vẫn mở được; tiêu điểm trả về nút mở khi đóng (UX-DR17). Khi màn xem trước đang mở, phím tắt toàn cục bị chặn như lúc lớp phủ đang mở.
- `ChapterOrigin.vue` giữ API (dùng chung với `LibraryWorkPage.vue`).

**Never:** sửa AD-5 hay spine; ước lượng trên màn song ngữ; đổi `GlossaryImportOverlay`, `PromptImportOverlay`, `ReviewerImportOverlay`, `TmImportOverlay`; chữ hiển thị trong Rust.

**Decisions (Ice, 2026-10-14):**
- D1. "Nhập luôn, nối sau" là một hàng = một segment (nguồn trọn hàng, đích trọn hàng), không AD mới. Dấu nằm ở bảng mới, bước 35 của `PROJECT_MIGRATIONS`.
- D2. Dấu hiện ở lưới Workspace (vạch lề hoặc nhãn "chưa nối", có chữ cho trình đọc màn hình) và ở hàng danh sách Chương ("N câu chưa nối").
- D3. FR137: pha Rust dựng hai cách: (a) đúng như quét thật, chạy nền, có cache tra từ điển, huỷ được; (b) chỉ lọc tần suất. Đo cả hai trên 2.000 Chương, rồi DỪNG cho Ice chọn trước pha Webview.
- D4. Màn xem trước thay chỗ nội dung Library (khuôn `LibraryWorkPage`); ba tab vẫn hiện. Đổi tab không đóng màn; quay lại tab Library thì màn còn nguyên.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Tách Chương | tệp txt 2.000 Chương | hai cột; trái: mẫu, tên Tác phẩm, ngôn ngữ "cố định sau khi tạo", "Chương nhận ra 2 000", "Trạng thái ban đầu Chưa bắt đầu", "Ứng viên Glossary sẽ quét ~N"; nút "Xác nhận và nhập 2 000 Chương" | — |
| Ước lượng không đổi bảng chờ | mở xem trước, đổi mẫu, huỷ | số hàng `glossary_candidate` bằng trước khi mở | — |
| Ước lượng hỏng | probe trả `Inconclusive` | dòng ước lượng ghi "không ước lượng được", xác nhận vẫn bấm được | không chặn nhập |
| Dán link | 50 URL cùng domain | "50 link · sẽ tạo 50 Chương · 1 domain", câu cam kết, nút "Tải 50 link" | — |
| Xem trước URL | 50 Chương đã tải | một màn, tầng 1–3 rồi tầng 4 "Xuất xứ tài liệu"; không còn tầng chuẩn hoá | — |
| Song ngữ lệch, nhập luôn | 1.284 hàng, 47 lệch | "Nhập luôn, nối sau" nhập 47 hàng thành 47 segment có dấu; danh sách Chương ghi "N câu chưa nối" | — |
| Song ngữ lệch, xem chỗ lệch | như trên | "Xem 47 chỗ lệch rồi nhập" đưa tiêu điểm tới hàng lệch đầu (hành vi cũ) | — |
| Xoá dấu | segment có dấu được xác nhận, hoặc bị tách | dấu mất; số ở danh sách Chương giảm | — |
| Cờ vắng | `confirm_bilingual_import` gọi không cờ khi còn hàng lệch | từ chối, không ghi gì | `import.bilingual_mismatched_rows` |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/project/wire.rs` -- `preview_import_encoding_from_text` :638, `…_from_file` :717, `start_url_import` :1331, `confirm_bilingual_import` :1236 (doc :1205 ghi vì sao song ngữ không quét). Các màn văn xuôi trả chung `ImportEncodingPreview` (`commands/project/mod.rs:1509`), nên một trường mới phủ cả ba.
- `src-tauri/src/commands/project/mod.rs` -- `run_pipeline` theo từng ứng viên bảng mã :1779-1810 (segment bị bỏ sau đó). Quét thật: `spawn_import_scan` :646, `run_one_chapter_import_scan` :719, `scan_with_configured_threshold` :439, probe :752-765.
- `src-tauri/src/core/glossary/scan.rs:123` -- `scan_candidates_controlled`, thuần, dùng lại được. Lọc theo phạm vi: `core/glossary/store.rs:365` (cần work `Store`, phải viết hàm trừ tập thuần mới). Khử trùng bảng chờ hiện nằm trong SQL ở `candidate_store.rs:256`.
- `src-tauri/src/core/store/work_creation.rs:246-257, :771-776` -- chỗ từ chối hàng lệch, phải nới khi có cờ. `core/segment/bilingual.rs:395` `BilingualRegroupingAction`.
- `src-tauri/src/core/store/schema.rs` -- `PROJECT_MIGRATIONS` (bước cuối là 34); thêm bước 35 cho bảng dấu "chưa nối". Không sửa DDL cũ.
- `src-tauri/src/commands/segment/confirm.rs:176`, `commands/segment/regroup.rs:441/557` -- xoá dấu trong giao dịch xác nhận, gộp, tách; tái tách Chương cũng vậy. `commands/chapter.rs:344` `ChapterRow`: thêm số câu chưa nối. DTO segment của lưới: lần theo hằng `CMD_*` ở `src/config/segment.ts`.
- `src/ImportPreviewOverlay.vue` (2674 dòng) + `importPreviewState.ts` -- tầng: tầng 1 :1069, chuẩn hoá :1123-1160 (bỏ), tầng 2 :1172 (có `<ChapterOrigin>` ở :1211, chuyển xuống thành tầng 4 ở cuối), tầng 3 :1294, danh sách Chương :1453, chân :1642. Nhãn xác nhận `command.import.preview.confirm`.
- `src/BilingualImportPreviewOverlay.vue` + `bilingualImportPreviewState.ts:136` (`…CanConfirm` đòi `mismatches.length === 0`).
- `src/LibraryImportOverlay.vue:239-256` -- ô dán URL, sẽ thành màn "Nhập từ website". Lớp phủ nhập cố ý mở bên dưới màn xem trước, vì nút `[data-import-preview-open]` của nó là đích trả tiêu điểm.
- `src/App.vue:343-354` gắn ba lớp phủ; `src/modes/LibraryMode.vue:275` khuôn `LibraryWorkPage v-if`; khuôn component riêng của `LibrarySearchResults.vue`; cổng phím tắt `main.ts` ~1029.
- `src/modes/libraryImport.ts:581/605/625/642` -- các lối mở.
- `scripts/check-commands.mjs:2413-2433` `HANDLER_TABLE` -- cập nhật mọi handler đổi tên, bỏ hoặc thêm. Lệnh mới đăng ký ở `src/commands/index.ts`.
- Mockup: `ux-auratranslate/mockups/library-and-import.html:161-220`, `web-import.html:162-398`, `bilingual-import.html:98-182`; EXPERIENCE.md §Đường nhập (316-375).

## Tasks & Acceptance

**Execution:** mỗi pha một agent mới; giao việc qua `story-man-xem-truoc-nhap-hai-cot-handoff.md` cạnh plan này.
- [ ] Pha Rust -- hai bộ ước lượng sau một trường `glossary_estimate` của `ImportEncodingPreview` (`Option`, không bao giờ `0` khi chưa biết); bảng dấu ở bước 35; cờ "nhập luôn" của `confirm_bilingual_import`; xoá dấu khi xác nhận hoặc về hưu; số chưa nối ở `ChapterRow`. Đo (a) và (b) trên 2.000 Chương tổng hợp, ghi bản build và số câu vào handoff, rồi **DỪNG cho Ice chọn (D3)**.
- [ ] Pha Webview -- màn tách Chương, màn "Nhập từ website", màn xem trước URL hợp nhất và màn song ngữ, thay chỗ nội dung Library; khoá i18n mới; `HANDLER_TABLE`; dấu "chưa nối" ở lưới và danh sách Chương.
- [ ] Pha Tests -- sửa các test dựa vào `.ip-scrim`/`.ip-panel`/`.li-panel`; thêm case cho mọi hàng của ma trận I/O.

**Tests that move:**
- Rust: `bilingual_import_contract.rs`, `ipc_contract.rs`, `glossary_scan_contract.rs`, `segment_contract.rs`, `webimport_contract.rs`.
- Vitest: `tests/frontend/importPreviewOverlayRender.test.ts`, `tests/frontend/importPreviewBlocks.test.ts`, `tests/frontend/importPreviewBilingual.test.ts`, `tests/frontend/importPreviewChapters.test.ts`, `tests/frontend/importPreviewCleanup.test.ts`, `tests/frontend/importPreviewFiles.test.ts`, `tests/frontend/importPreviewUrls.test.ts`, `tests/frontend/importPreviewChapterOrigin.test.ts`, `tests/frontend/importPreviewDestination.test.ts`, `tests/frontend/libraryImportOverlay.test.ts`, `tests/frontend/libraryImportBlocksResubmitWhilePreviewOpen.test.ts`, `tests/frontend/libraryChapters.test.ts`.
- e2e: `e2e/support/importForm.mjs` (bộ chọn), cùng ba người gọi nó (`story-5-4`, `story-5-5`, `story-12-2`); thêm `e2e/specs/story-12-6-import-preview.e2e.mjs`.
- Cũng chạm qua đổi dây (`ChapterSegment.unlinked`, `ChapterRow.unlinked_count`, bước 35, `byte_len`): Rust `asset_contract` · `chapter_origin_contract` · `config_invariants` · `dict_boundary` · `pinned_contract` · `review_harvest_contract` · `segment_role_contract` · `tm_contract` · `glossary_estimate_contract`; Vitest `tests/frontend/importPreviewTwoColumn.test.ts` (mới), `importPreviewEncoding*`, `importPreviewNormalized`, `libraryImportDropMultiple`, `libraryImportRetryAfterFailedConfirm` và các tệp fixture `unlinked`/`unlinked_count` (`editorRegroup*`, `export*`, `gridPanel*`, `librarySearch`, `reading*`, `segment*`, `tmConcordance*`, `chapterWireGuards`, `configSegmentChapterSegmentsShape`).

**Acceptance Criteria:**
- Given màn xem trước nào trong bốn màn đang mở, when nhìn thanh tiêu đề, then ba tab vẫn hiện, không có scrim, và ô phải ghi nguồn theo mockup của màn đó: tệp văn xuôi ghi tên · dung lượng, song ngữ ghi tên · N hàng, URL ghi N link · D domain (Ice, 2026-10-14).
- Given màn xem trước đang mở, when bấm một phím tắt toàn cục, then không lệnh toàn cục nào chạy; when Huỷ, then tiêu điểm về nút đã mở màn.
- Given Chương có câu chưa nối, when mở Chương đó trong Workspace, then trình đọc màn hình đọc được chữ "chưa nối" ở mỗi câu đó.

## Implementation Notes

- D3 chot (a) (Ice, 2026-10-14): uoc luong dung nhu quet that, co cache tra tu dien, huy duoc; duong chi-loc-tan-suat da go. Do tren 2.000 Chuong tong hop: ~32-35 s, tra tu dien ~11 s, dem n-gram ~25 s (handoff).
- "Nhap luon, noi sau" bi khoa kem ly do khi con hang lech nguon rong (Ice): Rust van tu choi hang do; webview dem bang `source_sentences.length === 0` trong DTO co san.
- Ước lượng là lệnh riêng `estimate_import_glossary` chạy sau xem trước, không phải một trường của `ImportEncodingPreview` như Code Map ghi: trường đó sẽ luôn `None` nên đã gỡ.
- WKWebView không focus nút khi bấm chuột, và driver e2e chỉ phát `mousedown` (không `pointerdown`), nên nút mở được ghi ở `@pointerdown`/`@mousedown`/`@focus` (`src/importOpener.ts`); lần đầu spec e2e đã che lỗi này bằng cách focus trước.
- Ô phải thanh tiêu đề cần kích thước tệp mà dây không mang: thêm `FileImportItem.byte_len: Option<u64>` (`None` khi không đo được, không bao giờ `0`).
- Ba màn xem trước giữ tên tệp `*Overlay.vue` dù không còn là lớp phủ; gắn trong `LibraryMode.vue`, lớp phủ nhập vẫn mở nhưng ẩn để giữ đích trả tiêu điểm.

## Plan Change Log

## Review Triage Log

Vòng 1 (quick): high 0 · medium 3 · low 6 · false 1 · maybe-false 0.
- medium · patch — xác nhận câu không xoá `unlinked` phía client: `editorPanelState.ts:1382` chỉ vá `status`, lưới còn vạch và aria "chưa nối" tới khi nạp lại Chương.
- medium · patch — `main.ts` `isBlocked` và `titlebarImportSource.ts` đọc ba cờ màn mà không xét `currentMode`: sang tab Workspace khi màn xem trước còn mở thì mất mọi phím toàn cục và thanh tiêu đề vẫn ghi nguồn nhập.
- medium · patch — chưa test nào chạm phép trừ của lệnh `estimate_import_glossary` (Glossary, hàng `import_scan` khi nhập nối); e2e chỉ đếm bảng chờ của Work mới. Kèm: ca vitest gọi lệnh thật "Xem chỗ lệch" để tiêu điểm tới hàng lệch đầu.
- low · intent_gap → Ice sửa AC1 (2026-10-14), không hoàn tác — AC1 đòi "tên tệp và dung lượng" cho cả bốn màn; màn URL không có tệp, màn song ngữ theo mockup ghi tên · N hàng, không dung lượng. AC không định được nghĩa cho nguồn không phải tệp văn xuôi.
- low · reject — pha `run_pipeline` của ước lượng không huỷ được, chỉ phần đếm n-gram và tra từ điển huỷ được; lượt cũ chỉ tốn CPU rồi bị bỏ kết quả, không sai số. Sửa cần luồn huỷ vào pipeline.
- false — dừng cả ước lượng khi một probe `Inconclusive`: đúng hàng ma trận frozen "probe trả `Inconclusive` → không ước lượng được".
- low · patch — comment mới phạm AGENTS.md §Code comments: khối "🔴 STORY 6.7b" chép lại trong `ImportPreviewOverlay.vue`, "⚠️ … KHÔNG đặt." ở `glossary_estimate.rs`, doc tiếng Việt thuật lại chữ ký ở `wire.rs::estimate_import_glossary`, đầu tệp hai màn viết lại dài.
- low · patch — khoá i18n không còn ai dùng: `mode.library.url_count`, `submit_urls`, `submit_urls_append`, `preview.chapters_count`, `preview.tier_normalized_*`, `preview.destination_source_lang`.
- low · patch — dòng `LibraryUrlImportScreen.vue::onEscape` nằm giữa khối `ImportPreviewOverlay.vue::*` của `HANDLER_TABLE`.
- low · patch — `wire.rs::estimate_import_glossary` lấy lỗi "chưa chỉ mục" bằng `open_destination_for_append(work_id, None).err()`, một đường mở store ghi; dùng thẳng `work_not_indexed`.
- low · reject — Implementation Notes thiếu dấu: sửa là sửa plan của chính lượt build.

## Design Notes

- Song ngữ không có ước lượng: confirm song ngữ không chạy quét (`wire.rs:1205`), và `bilingual-import.html` không vẽ dòng này. Một con số không ứng với lần quét nào là lời hứa sai.
- Ước lượng chạy sau khi xem trước dựng xong và không chặn màn: dòng hiện "đang ước lượng…" rồi "~N". Mỗi lần dựng lại (đổi mẫu, đổi bảng mã) huỷ lượt cũ.
- Bước 35 đặt trước migration của 12.14/12.15/12.19/12.20. Các story đó chưa viết, nên số của chúng tự lùi.

## Verification

**Commands:**
- `npm run build && npm run test:story 12-6` -- expected: xanh.
- `cargo test --test bilingual_import_contract` -- expected: xanh, có case cờ vắng (từ chối) và cờ có (47 segment có dấu).
- Migration chạm wiring dùng chung: chạy toàn bộ `cargo test` một lần trước khi push.
- `npm run test:e2e -- e2e/story-12-6-import-preview.e2e.mjs` -- expected: xanh.

**Manual checks:**
- Ice nhập một tệp `.docx` song ngữ thật có hàng lệch trên bản đóng gói: thành mục nợ `Chủ: Epic 12`.
