---
epic: epic-tra-no-nen
date: 2026-09-30
verdict: accepted-with-open-items
criteria: declared
headless: false
---

# Retro Epic 11 — Trả nợ nền

## Tóm tắt epic

- Epic: 11 — *Trả nợ nền — đóng nợ đã hoãn của Epic 1–6 trước khi xây tiếp* (`initiative-auratranslate/archive-v6/epics.md` §Epic 11; nguồn quyết: `sprint-change-proposal-2026-09-24-epic-11-tra-no-nen.md`).
- Tiêu chí: **khai báo** — bốn khối "AC chung cho mọi story của Epic 11" trong `epics.md` §Epic 11; mỗi story thừa kế, AC riêng rút từ các mục `Chủ: Story 11.N`.
- Story: 7/7 `done` (`detect-epic --epic 11`: `story_count: 7`, `pending_stories: []`). Ice ký 11-4…11-7 từ `review` sang `done` ở `e390078` trước retro.
- Dải diff: `b825a58^..5897669` — 15 commit không merge, 0 merge, 338 tệp, +34 807 / −9 179 dòng (`git_evidence.py`). Commit lập kế hoạch epic (`b8f22f7`…`2abddd3`) nằm trước dải và không tính vào.
- Commit ngoài story trong dải: `90d4428` (fix(hanviet), không mang id story) · `3dd4a5e` (cập nhật sprint-status gộp 11-1/11-2/11-3).
- Trước retro: `8c18781` (chore(ai) `#[expect(async_fn_in_trait)]`) và `e390078` được commit riêng theo lựa chọn của Ice; nằm ngoài dải.

### Kho bằng chứng

| Bằng chứng | Có | Nguồn |
|---|---|---|
| Đặc tả epic + AC chung | có | `epics.md` §Epic 11; `sprint-change-proposal-2026-09-24-epic-11-tra-no-nen.md`; `epic-11-context.md` |
| Spec story | 12 tệp (11-1 bốn lô, 11-6 và 11-7 hai lô) | `implementation-artifacts/spec-11-*.md` |
| Tệp bàn giao pha | 15 tệp (`11-*-task0-*`, `11-*-phases-*`) | `implementation-artifacts/11-*.md` |
| Commit theo story | 15 commit trong dải, 13 mang id story | `git_evidence.py --range b825a58^..5897669` |
| Sổ nợ | 345 dòng mang `Chủ: Story 11.N` (11.1: 55 · 11.2: 60 · 11.3: 36 · 11.4: 41 · 11.5: 43 · 11.6: 77 · 11.7: 33 — đếm dòng, không phải mục) | `grep -c` trên `deferred-work.md` |
| Cổng sổ nợ | `check:debt-owner` xanh lúc retro | `npm run -s check:debt-owner` |
| CI | lượt push gần nhất xanh @`32d933c`; mọi lượt `schedule` 09-23…09-29 đỏ; 4 commit cục bộ chưa push (`7dbcf23`, `5897669`, `8c18781`, `e390078`) | `gh run list` |
| Retro trước | Epic 4 (`epic-4-retro-2026-09-23.md`) + 33 action item chưa `done` từ Epic 1/2/3/5/6/10 | `sprint-status.yaml` `action_items` |
| Session log | 18 tệp `.jsonl` từ 2026-09-24 | `~/.claude/projects/-Users-hoangnam-LocalSites-addon-AuraTranslate/` |

Trọng tâm Ice chọn lúc mở retro: **ranh giới giữa các lô/story** và **sổ nợ có đóng thật không**.

## Phát hiện

### CI và nhịp nghiệm thu

- **F-CI-1.** Job `e2e` chỉ chạy ở `schedule`/`workflow_dispatch`, không ở `push` (`.github/workflows/ci.yml:764`). Sau lượt push `717196d` (09-25), sáu commit `c6e5e21`…`32d933c` chỉ lên origin ở một lượt push 09-29 (run 36561059042). Ba đêm 09-26/27/28 (run 36270036228, 36349295058, 36494988256) vì thế chạy trên mã TRƯỚC bản vá 11-2 và đỏ cùng spec cũ `editor-typing-flush.e2e.mjs:218` (`"Đã lưu 0 giây trướcTra cứu"` — hồi quy của `505c8bc`). Nợ đỏ đêm là do mã chưa lên origin, không do bản vá sai. Xử lý: *hoãn* — bài học quy trình (V-3).
- **F-CI-2.** Đêm đầu tiên bản vá 11-2 thật sự chạy (run 36635570813 @`32d933c`, 09-29): `editor-typing-flush` xanh, nhưng helper mới của 11-2 `ensureFullLayoutTier()` đỏ ở `attribution-focus.e2e.mjs:64` — *"Cửa sổ không lên được tầng `full` … Tầng đọc được lần cuối: narrow"* (`e2e/support/layoutTier.mjs:32`). Nightly hiện vẫn đỏ, nguyên nhân mới, chưa ai ghi. Xử lý: *sửa ngay* (V-1).
- **F-CI-3.** Quyết định 3 của 11-2 (năm lượt `workflow_dispatch` xanh) chưa chạy: task còn `[ ]` ở `spec-11-2-e2e-and-test-runner-debt.md:74` khi spec `done`; lượt `workflow_dispatch` gần nhất là 08-21. Sổ nợ đã chuyển việc đó sang `Chủ: Epic 11` (`deferred-work.md` quanh L4604). Xử lý: gộp vào V-1.
- **F-CI-4.** Lên `review`/`done` khi đêm gần nhất đỏ mà không có dòng lý do: 11-4, 11-5, 11-6 lô A, 11-7 lô A và lô B (grep run id 36635570813 trong `_bmad-output` = 0 tệp). Có dòng lý do: 11-2 (spec dòng 85), 11-3 (phases dòng 445), 11-6 lô B (spec dòng 105). Luật AGENTS.md §Tests *"Red ⇒ write down why"* bị bỏ qua năm lần. Đây là lần lặp thứ tư của cùng lớp lỗi (retro Epic 1 B7, Epic 3 AI-8, Epic 5 AI-9, Epic 6 AI-1). Xử lý: *hoãn*, cần Ice quyết (Q-1).
- **F-CI-5.** Không có cổng nào đọc lượt `schedule`: `.githooks/pre-push:65` chỉ gọi `scripts/ci-previous-verdict.mjs || true` (cảnh báo, lọc `--event push`). Ghi nhận, không phải lỗi: memory *"cổng chỉ canh mã nguồn, không canh quy trình BMAD"* chặn hướng dựng cổng trên sprint-status.
- **F-CI-6.** Windows 09-24 (run 36059125872) đỏ một lần ở `store_contract.rs:711` `an_idle_pause_triggers_one_passive_checkpoint` (`busy=1`). Đây là flake PASSIVE-busy đã ghi trong sổ nợ, 11-5 chốt KHÔNG LÀM (tỉ lệ 7/14); năm đêm sau xanh. Xử lý: *chấp nhận* như đã ghi.

### Quy trình (session log)

Nguồn: 18 phiên `.jsonl` từ 2026-09-24 cùng các tệp subagent của chúng. Đỉnh context = max(`input_tokens + cache_read + cache_creation`) trên các lượt assistant.

- **F-P-1.** Phiên điều phối vượt ~250 k ở 11 trên 14 phiên có gọi Agent: 11-1 lô C 400 k, 11-7 lô A 371 k, 11-4 352 k, 11-5 350 k, 11-2 348 k, 11-6 lô B 338 k, 11-1 lô D 323 k, 11-6 lô A 303 k, 11-3 281 k, 11-1 lô B 259 k. Phiên PM lập kế hoạch 09-24 đạt 424 k.
- **F-P-2.** Có 57/145 subagent vượt 250 k. Agent cài đặt lớn nhất: 966 k (11-6 lô B, 1 737 lượt), 781 k (11-1 lô B, một agent làm cả lô), 744 k (11-4), 719 k (11-6 lô A), 642 k (11-5). Khuôn thật là Task 0 (3–4 agent đọc song song) → agent cài đặt → 3 reviewer → agent sửa. Đó không phải khuôn bốn pha plan · Rust · Webview · Tests của AGENTS.md. Gần khuôn nhất là 11-7 (2/18 agent vượt); xa nhất là 11-6 (17/28). Xử lý: bài học quy trình (V-4).
- **F-P-3.** `32d933c` vá fixture `glossaryMarksRefresh.test.ts` 61 phút sau `7c89d8e` vì pre-push đỏ: 11-6 lô B đổi hình dạng `ChapterSegments` (thêm `assets`/`assets_dir`) mà mock ở tệp test của story khác không đổi theo. Đây là lỗi ranh giới đã tự lộ; pre-push bắt được.
- **F-P-4.** Quy ước commit: `3dd4a5e` viết tiếng Anh, không nêu điều tìm được, gộp ba story; `90d4428` không mang id story. Mười một commit còn lại đúng khuôn (kiểm bằng mắt). Xử lý: *chấp nhận*, ghi lại để các lượt sau không gắn cờ lại.

### Ranh giới giữa story — Rust

Phạm vi đã thu hẹp: các tệp Rust bị ≥ 2 story chạm (`commands/project/{mod,wire,work_creation}.rs`, `core/store/{schema,mod}.rs`, `commands/{segment,glossary,chapter}.rs`, `core/glossary/{store,candidate_store}.rs`, `core/i18n/mod.rs`, `Cargo.toml`). Ba lăng kính (đối kháng, biên, lỗ xác minh) áp tay, không qua `bmad-review`. **Chưa kiểm**: `core/docx`, `webimport/assets.rs`, `url_import.rs`, `bilingual.rs`.

- **F-R-1 (trung, đã đọc lại mã).** `confirm_segment` ghi thẳng `origin_at_load` mà webview gửi lên vào `translation_origin`, không đối chiếu `TRANSLATION_ORIGINS` (`commands/segment.rs:2522-2530`). Cùng story 11-5 (`21a2521`) lại dựng sàn "từ chối mở `project.db` có origin lạ" ở `open_work` (`commands/project/mod.rs:3072-3081`). Kịch bản: văn bản không đổi, `translation_origin` khác rỗng, webview gửi một origin ngoài danh mục. Giá trị đó được ghi xuống đĩa, và lần mở sau cả Tác phẩm bị từ chối với `store.unknown_translation_origin`; trong app không có đường sửa. Ca `tests/segment_contract.rs:5902` dùng origin sai nhưng rơi vào nhánh đã confirmed (return sớm), không chạm tới lượt ghi. Kích hoạt cần một webview gửi sai, nên đây là suy luận. Xử lý: *sửa ngay* (V-2).
- **F-R-2 (trung, thứ tự mã đã xác minh, cuộc đua là suy luận).** Nhánh APPEND vào Tác phẩm **đang mở** (`commands/project/wire.rs:892-925`) không tạo `AppendInProgressGuard`; guard chỉ có ở nhánh chưa mở (`wire.rs:943`). Trong khi đó phép quét ảnh mồ côi của 11-6 lô B (`7c89d8e`) ở `replace_open_work` (`commands/project/mod.rs:3345-3368`) chạy trước khi lấy khoá `OpenWorkState`. Kịch bản: đang append URL có ảnh vào Tác phẩm đang mở, người dùng mở lại chính Tác phẩm đó. Phép quét xoá tệp ảnh chưa có hàng `asset`, rồi append commit hàng trỏ vào tệp đã mất: ảnh vỡ, không báo lỗi. Test (`commands/project/tests.rs:1141-1235`) gọi thẳng hàm quét và guard, không dựng đường này. Xử lý: *sửa ngay* (V-2).
- **F-R-3 (trung, suy luận — độ dài treo chưa đo).** Khoá `OpenWorkState` bị giữ qua việc dài, trong khi mọi wire segment là đồng bộ: append vào Tác phẩm đang mở giữ khoá suốt lượt tải ảnh (`wire.rs:895-925`, 11-6 lô B), và `glossary_pending_candidates` quét toàn Tác phẩm dưới guard (`commands/glossary.rs:1147-1170`, 11-4). Lượt sàng wire đồng bộ của 11-7 chỉ xét `PendingImportSourceState`. Kịch bản: auto-save `save_segment_targets` đứng chờ ở `state.lock()`, giao diện đứng theo. Xử lý: *hoãn* thành mục nợ có đo.
- **F-R-4 (thấp).** `merge_chapter_into_previous` gán thẳng `open.chapter_id = a_id` (`commands/chapter.rs:954`), bỏ qua `set_open_chapter` mà 11-6 lô A (`121c178`) vừa dựng để giữ `last_chapter_id`. Sau khi gộp rồi mở lại, Tác phẩm rơi về Chương đầu. Không mất dữ liệu. Xử lý: gộp vào V-2.
- **F-R-5 (thấp, chưa kích hoạt).** `chapter_span_count` dùng `unwrap_or(0)` (`commands/glossary.rs:414-417`), và `approve_candidate` ghi `Some(0)` từ cột `NOT NULL DEFAULT 0` (`core/glossary/candidate_store.rs:345`), đúng lúc 11-4 vừa đổi `glossary_entry.occurrence_count` sang nullable. Vi phạm luật NULL-vs-0; hôm nay chưa có đường sản phẩm chạm tới. Xử lý: *hoãn*.
- **F-R-6 (thấp, suy luận).** `save_segment_targets` không lọc `retired_at` (`commands/segment.rs:1972`); 11-7 lô B chỉ vá cùng lớp lỗi này ở `promote_ai_translation`. Xử lý: *hoãn*.
- **F-R-7 (thấp, suy luận).** `update_chapter_origin` với `apply_through_ord` dùng `ord BETWEEN` mà không chuẩn hoá `ord` (11-6 lô B), trong khi move/merge có chuẩn hoá. Chưa tìm được đường tạo `ord` trùng. Xử lý: *chấp nhận*, ghi lại.
- **F-R-8 (thấp, tài liệu).** Doc của `open_work` hứa "không một byte nào bị ghi" khi từ chối origin lạ, nhưng `Store::open` đã chạy migration và backup trước bước kiểm (`commands/project/mod.rs:3072` rồi `:3078`). Xử lý: gộp vào V-2.

Đã kiểm, sạch: hai migration mới (GLOBAL v10, PROJECT v25/v26) đúng thứ tự; `write_regroup` ghi origin cùng INSERT (AD-47); `confirm_segment` ghi status và origin trong một UPDATE; `promote_ai_translation` sau vá đủ nhánh missing/retired/nháp chưa ký; gộp Chương chỉ đổi `chapter_id`/`ord`; `Cargo.toml` có lý do và licence cho ba thay đổi; `i18n::ImportTooLarge` đổi params khớp cả ba chỗ dựng.

### Ranh giới giữa story — webview và cổng

Phạm vi đã thu hẹp: các tệp `src/**`, `scripts/check-*` và test frontend bị ≥ 2 story chạm. Ba lăng kính áp tay. Chạy ba tệp vitest (`glossaryMarksRefresh`, `libraryRescan`, `editorRegroupAssetRefresh`): 37 ca xanh. Số xanh đó không phủ các phát hiện dưới. **Chưa kiểm sâu**: `WorkspaceDock.vue`, `SettingsGlossarySection.vue`, khung nhìn tầng của `aiConfigState.ts`.

- **F-W-1 (cao, đã đọc lại mã).** `loadLibraryOrphans` (11-6 lô A, `121c178`, được gọi từ `LibraryMode.vue::onActivated`) tăng CHUNG bộ đếm `sequence` (`src/modes/libraryRescan.ts:224-227`). Ba lượt còn lại — `rescanLibraryFolder` `:143-146`, `chooseLibraryRootFolder` `:170-173`, `forgetCurrentLibraryOrphan` `:193-197` — đều `return` ở `mySequence !== sequence` TRƯỚC `rescanBusy.value = false`. Kịch bản: bấm Quét lại, sang Workspace rồi quay lại Library trước khi quét xong. Kết quả quét bị vứt, còn `rescanBusy` kẹt `true` đến khi khởi động lại app, nên mọi lần bấm Quét lại/Chọn thư mục/Gỡ mồ côi sau đó bị nuốt im lặng. Doc-comment `:218-221` khẳng định "và ngược lại" — chính chiều này sai. `libraryRescan.test.ts:113` chỉ phủ chiều kia. Xử lý: *sửa ngay* (V-2).
- **F-W-2 (trung, suy luận).** `refreshChapterAssetsAfterRegroup` (11-6 lô B, `7c89d8e`) dùng `++sequence` chung với `ensureSegmentsLoaded` (`src/panels/editorPanelState.ts:2644-2646` vs `:175-183`). Kịch bản: chuyển Chương chen giữa lúc gộp/tách đang await. Lượt nạp Chương mới bị vứt, `pending`/`requested` kẹt, lưới trống không báo lỗi — đúng lớp "rỗng im lặng". Ngoài ra lượt refresh không đối chiếu `loaded.chapter_id` trước khi gán ảnh. Cùng hình dạng với F-W-1. Xử lý: *sửa ngay* (V-2).
- **F-W-3 (thấp–trung, đã xác minh điều kiện).** `src/editorClearSourceCuts.ts:17-27` không canh `editorPendingPromote`. Câu hỏi PROMOTE do chính 11-5 (`21a2521`) thêm, và cùng commit đó đã mở guard cho history và shortcuts nhưng bỏ sót promote. `Escape` trên nút của câu hỏi PROMOTE vì thế xoá điểm cắt mà không huỷ câu hỏi. Xử lý: gộp vào V-2.
- **F-W-4 (thấp).** Nhánh `settingsOverlayIsOpen` trong `clearSourceCuts` (11-7 lô A, `7dbcf23`) là nhánh chết ở đường sản phẩm, vì `isBlocked` (`src/main.ts:969-985`) đã chặn chord khi Cài đặt mở; test canh hàm, không canh dây. Gộp Phím tắt vào Cài đặt cũng làm mất hành vi "đang xem bảng phím vẫn thử được chord", và comment `main.ts:951-966` giờ sai. Xử lý: *hoãn*, cần Sally/Ice quyết hành vi.
- **F-W-5 (thấp, cần Ice xác nhận).** `RS_FILE_FLOOR` của `scripts/check-dict-build.mjs:56` bị hạ 24 → 21 ở 11-1 lô D (`717196d`) trong khi số thật là 24. Đây là sàn duy nhất bị HẠ trong dải; các sàn khác đều nâng. Hạ sàn khớp với luật đo chung mới (`judgeFloor`, 21/24 = 87,5 %). Nhưng ghi chú của Story 1.10 bị xoá cùng lượt từng cảnh báo rằng sàn hở cho phép xoá tới ba parser mà cổng vẫn xanh. Xử lý: Q-3.
- **F-W-6 (thấp, suy luận).** Nhãn lỗi hàng ở `src/panels/GridPanel.vue:1602-1628` (11-5) dùng `t()` thay vì `tError()`. Hai nhánh mới (restore, flush) chỉ gác `!== null`, nên `message_key` rỗng cho ra nhãn trống trên một hàng mang class `refused`. Xử lý: *hoãn*.
- **F-W-7 (thấp, suy luận).** `src/modes/libraryChapters.ts:540-554` (11-6 lô B): khi ord nằm ngoài dải, giá trị bị đặt về `null` nhưng ô số vẫn hiện số đã gõ, nên lưu chỉ ghi một Chương mà người dùng tưởng đã áp cả đoạn. Ord cũng không được xoá sau khi lưu thành công (`:525`). Xử lý: *hoãn*.
- **F-W-8 (thấp, lỗ xác minh).** `tests/frontend/support/segmentFixture.ts:144` `readFixture` thiếu `assets`/`assets_dir`. `refreshChapterAssetsAfterRegroup` gán thẳng `loaded.assets` (không `?? []`), trong khi `ensureSegmentsLoaded` có `?? []`. `32d933c` chỉ vá fixture của `glossaryMarksRefresh`; khoảng 25 tệp test khác vẫn mock `readOpenChapterSegments` không có `assets`. Fixture không mang đúng hình dạng IPC nên test không bắt được lỗi hình dạng. Xử lý: gộp vào V-2.
- **F-W-9 (vệ sinh).** Chú thích còn trỏ tới `GlossarySettingsOverlay.vue`/`ShortcutsOverlay.vue` đã xoá hoặc đổi tên, và tới `shortcuts.close`/`glossary.settings.close` (`src/commands/index.ts:2386,3009`); `check:doc-refs` không bắt. `src/settingsState.ts` chứa 3 byte NUL thô nên git coi là nhị phân (có từ trước 11-7). Xử lý: *hoãn*.

Đã kiểm, sạch: spread `aiTranslateHandlers` trong `main.ts` (có `aiTranslateHandlersWiring.test.ts`); `Mod+Comma` đổi chủ sang `settings.open`; bẫy Tab và `capturing` của `SettingsOverlay.vue`/`SettingsShortcutsSection.vue`; `SourceHanViet.vue`; `GlossaryQueueOverlay.vue`; miễn trừ mới trong các cổng đều có lý do; `FILE_FLOOR` doc-refs 369 → 395 là nâng; `commands-scan.mjs` che comment và literal trước khi quét.

### Khung nhìn tổng thể

Phạm vi đo: `src/`, `src-tauri/src/`, `scripts/` (kích thước còn tính `src-tauri/tests`, `tests`, `e2e`); gốc so sánh là `b825a58^`. Đo bằng git/grep/awk/python.

- **F-A-1 (kích thước, tích cực).** `commands/project/mod.rs` giảm 5 276 → 3 486 dòng: 11-6 lô A (`121c178`) tách ra ba mô-đun `work_creation.rs` (1 865), `url_import.rs` (523), `bilingual.rs` (379). Cả thư mục `commands/project/` vẫn tăng 7 733 → 9 161. Hàm lớn: `create_work_with_progress` khoảng 534 dòng, `run_one_chapter_import_scan` khoảng 495 dòng (đếm bằng awk, xấp xỉ). Số tệp trên 2 000 dòng tăng 20 → 24. Ba tệp test trên 5 000 dòng: `segment_contract.rs` 10 624, `project_contract.rs` 5 936, `dict_sources.rs` 5 475. Không có cổng đo kích thước tệp (các "sàn" hiện có chỉ là sàn số tệp). Xử lý: *hoãn*; AI-6 retro Epic 6 đã tiến một phần (xem §Theo dõi retro trước).
- **F-A-2 (trùng lặp, gom thật).** Trong mã chỉ còn một bản `text_before_first_cfg_test_line` (lúc gốc: 84 lượt trong 8 tệp; nay còn 2 lượt, cả hai là comment). Cả 48 sàn đều đi qua `judgeFloor` (JS) hoặc `assert_population_floor` (Rust), không cổng nào còn tự tính 0,85. 11-1 lô B và lô D đóng thật điều chúng khai. Dư: literal `84` lặp ở 20 hằng Rust.
- **F-A-3 (trùng lặp, chưa gom).** `focusableWithin` còn 12 bản tự định nghĩa (gốc 14). Giảm 2 chỉ vì hai overlay bị gộp/xoá ở 11-7, không có bản dùng chung. Guard `chapterId === null || chapter === null` vẫn nằm ở 4 tệp như gốc. Trùng mới do Epic 11: `matching_close_brace` có 2 bản (`src-tauri/tests/ipc_argument_contract.rs:330`, `src-tauri/tests/support/boundary_scan.rs:325`); helper test `mountGrid`/`mountPanel`/`warmModules`/`freshCommands`/`recordConfirm` bị viết lại ở nhiều tệp. Mã sản phẩm mới: 0 tên hàm trùng ở ≥ 2 tệp. Xử lý: *hoãn* (AI-11/AI-12 retro Epic 3 vẫn mở).
- **F-A-4 (lệch khuôn).** 2 960 dòng comment mới, 97 dòng khớp mẫu story-id/ngày/🔵. Phần lớn là văn bản Epic 6 bị kéo theo khi tách tệp (53 dòng); chỉ 1 dòng "Story 11.1" viết mới (`scripts/check-tokens.mjs`). Đây là di sản bị chuyển chỗ, không phải vi phạm viết mới. `unwrap_or(0)` mới: `commands/glossary.rs:420` (trùng F-R-5) và `commands/project/bilingual.rs:261` (`.max().unwrap_or(0)`). Định danh `origin` trần: `let origin =` ở `commands/segment.rs:2522`, `entry?.origin` ở `src/importPreviewState.ts:643`, và emit của `ChapterOrigin.vue` — chưa xác minh từng chỗ. Miễn trừ mới: 1 `eslint-disable`, 27 `aura-allow-*`, tất cả có lý do. Định danh mới dùng Project/Book/Novel/Document: 0. Xử lý: định danh `origin` trần gộp vào V-2 (đổi tên khi chạm dòng); phần còn lại *chấp nhận*.
- **F-A-5 (kiến trúc, sạch).** Mô-đun Rust mới chỉ gồm ba tệp con của `commands/project`. Không có cạnh mới `core → commands` (ba cạnh `use crate::commands::segment::ChapterSegment` đã có từ gốc), 0 cạnh `ports → commands`. `invoke` chỉ nằm trong `src/config/*`. Bốn dependency mới (webdriverio 9.30.1 ở `c6e5e21`, unicode-normalization =0.1.25 ở `21a2521`, base64 =0.22.1 ở `7c89d8e`, feature `test` của tauri dev ở `5897669`) đều đã có dòng trong bảng Stack của spine, cùng commit với thay đổi manifest (NFR15 đạt).
- **F-A-6 (sàn).** 41 hằng sàn được nâng, đúng 1 hạ (F-W-5). Hằng mới: `HANDLER_ATTR_FLOOR`, `EMITTED_VAR_FLOOR`, `DOCKVIEW_VAR_FLOOR`, `REGISTERED_COMMAND_FLOOR`, và `FILE_FLOOR` 395 của doc-refs.

### Sổ nợ — đóng thật chưa (11.1 · 11.2 · 11.3)

Cách đếm: một mục là một khối bắt đầu bằng `- ` ở cột 0, thuộc story nếu mang `Chủ: Story 11.N`; trạng thái lấy từ dòng `→` cuối. `DW:n` là số dòng `deferred-work.md` ở `e390078`.

| Story | Mục | ✅ | KHÔNG LÀM | Chuyển chủ | 🟡/mở |
|---|---|---|---|---|---|
| 11.1 | 53 | 38 | 14 | 1 (sang 11.5, rồi 11.5 chốt KHÔNG LÀM) | 0 |
| 11.2 | 36 | 22 | 6 | 8 (`Epic 11`) | 0 |
| 11.3 | 36 | 12 | 16 | 3 (`Ice`) | 5 🟡 (Winston ×3: DW:452/821/7897; `Epic 11` ×2: DW:1081/1261) |

Spec bốn lô 11.1 cộng lại ra 55 mục, cách đếm trên ra 53; chưa giải thích được chênh lệch 2.

Lấy mẫu 34 mục ✅ (11.1: 12, 11.2: 10, 11.3: 12): mọi test/mã được trỏ tới đều tồn tại ở HEAD (`git grep`). Ba tệp vitest hẹp (`commandsRegistryPortMissing`, `floorJudge`, `sourceHanVietCopy`) xanh 14 ca.

- **F-D-1 (lỗ xác minh, suy luận).** Có mục ✅ mà guard không đỏ được khi gỡ seam trong mã sản phẩm:
  - DW:438 (11.3): ca EXPLAIN QUERY PLAN chạy SQL chép tay trên DDL của fixture, nên đổi WHERE/JOIN trong `query.rs` không làm nó đỏ. Phép đối chứng gỡ trên DDL fixture, không trên mã sản phẩm. Sổ tự khai 🟡.
  - DW:316/320 (11.1): các ca `boundary_scan_contract.rs` gọi thẳng helper, và không test nào ép 18 tệp `*_boundary.rs` phải đi qua nó. Nếu `starts_with(DIR)` trần quay lại, không ca nào đỏ.
  - DW:985 (11.2): spec ghi ca ③ của `hanviet-segmenter-webkit` vẫn xanh khi ép `.hv-unit{display:inline-block}`, tức ca không phân biệt hai nhánh. Sổ có mục riêng DW:12552 (`Chủ: Ice`), nhưng DW:985 vẫn ✅.
  - DW:3561 (11.2): chỉ có đối chứng dương (TZ=UTC vẫn xanh), không có phép gỡ dòng TZ.
  - Xử lý: *hoãn* thành mục nợ (V-5).
- **F-D-2 (đóng bằng lời).** DW:4236 (một dòng vào `e2e/AGENTS.md`), DW:2279 (đọc `setup.ts`), DW:461 và DW:609 (doc-comment / dẫn commit cũ `19ea24c`), DW:3456 (chú thích `main.ts`). Đây là mục tài liệu hoặc quy trình, không có mã để canh. Xử lý: *chấp nhận*.
- **F-D-3 (sổ nói khác mã, đã đọc lại mã).** Dòng đóng DW:1054 (11.3) nói `onCopy` "LUÔN dựng lại … bỏ điều kiện `text.includes(WORD_JOINER)`". Mã HEAD chỉ dựng lại ở kiểu `parallel`; kiểu `switch` vẫn `if (!text.includes(WORD_JOINER)) return` (`src/panels/SourceHanViet.vue:780-801`). Hành vi đúng (kiểu `switch` có WORD_JOINER) và test đầu `sourceHanVietCopy.test.ts` mô tả đúng mã; chỉ câu đóng trong sổ sai. Xử lý: sửa tại chỗ bằng 🔵 (V-6).
- **F-D-4 (KHÔNG LÀM mà hành vi sai còn đo được).** DW:4900 (ca WAL đỏ 7/14 lượt CI trên cả hai OS — chính F-CI-6); DW:1100 (khe hở lọc nguồn tới 4,8 %); DW:1389 (`sessionLookupCount` thấp hơn thật khi chạm trần 200 hàng); DW:1769 (mất tiêu điểm WKWebView, có e2e giảm nhẹ); DW:7502/10740 (hai lượt đỏ nightly "không tái lập"). Mỗi mục có lý do và ngày theo khuôn. Xử lý: *chấp nhận* như Ice đã quyết; ghi lại để lượt sau không gắn cờ lại.
- **F-D-5 (khuôn).** DW:12162 (11.1) ghi KHÔNG LÀM "(phiếu quyết #106)" không kèm nhãn Story. 13 mục KHÔNG LÀM vẫn kèm `Chủ: Ice` (DW:569/616/620; sáu mục 11.2; DW:397/408/810/825). DW:569 viết "một AD mới, chủ Winston" nhưng chủ cuối là `Ice`, trái AC chung thứ tư (đổi bất biến kiến trúc thì chuyển Winston). Ba mục Winston của 11.3 (họ NFC/NFD) chưa có `ad-brief-*.md`. Xử lý: V-6.
- **F-D-6 (Kiểm C bắt được, đúng thiết kế).** 11.2 sang `done` khi DW:4587/4669/5538 còn `Chủ: Story 11.2`; ngày 2026-09-27, lượt 11.4 chuyển chúng sang `Epic 11`. `spec-11-2…md:74` còn `- [ ]` trong khi status là `done` (trùng F-CI-3).

### Sổ nợ — đóng thật chưa (11.4 · 11.5 · 11.6 · 11.7)

Cách đếm: bullet cấp 0; trạng thái lấy từ dòng `→` đầu tiên sau `Chủ: Story 11.N` cuối cùng. Tổng 194 mục, khớp với 41/43/77/33 dòng. `L` là số dòng `deferred-work.md`. `check:debt-owner` xanh (0/296 ở Kiểm C). `cargo test --test ai_translate_contract promot` xanh 3/3.

| Story | ✅ | KHÔNG LÀM | Chuyển chủ | 🟡 | Mở |
|---|---|---|---|---|---|
| 11.4 | 21 | 10 | 10 (Epic 11 ×7, Winston ×2, B7) | 0 | 0 |
| 11.5 | 27 | 12 | 3 (Ice, Sally ×2) | 1 (Epic 11) | 0 |
| 11.6 | 51 | 14 | 7 (Winston, B7, Ice ×2, Sally, Story 10.9 ×2) | 5 (Epic 11 ×4, Ice) | 0 |
| 11.7 | 24 | 5 | 3 (Epic 11+B7, John, Winston) | 1 (Amelia) | 0 |

Ngoài ra có 18 mục MỚI mở trong các lượt review: Epic 11 ×7, Murat ×3, Amelia ×3, B7 ×2, Sally, Ice, và 1 mục đã đóng ngay ở 11.7.

- **F-D-7 (tỉ lệ đối chứng đỏ ghi trong lời đóng).** 11.4: 18/21 · 11.5: 12/27 · 11.6: 9/51 (phép đối chứng của 11.6 nằm ở tệp bàn giao `11-6-lo-{a,b}-phases`, không ở sổ hay spec) · 11.7: 13/24. Tất cả là lời khai; retro không chạy lại phép gỡ seam nào.
- **F-D-8 (guard chỉ canh hàm, không canh vỏ/dây — suy luận, chưa chạy).**
  - 11.4 L6731: `scan_with_configured_threshold`; chỗ gọi thật ở `commands/project/mod.rs:758`.
  - 11.4 L6333: `glossary_export_tier_after_dialog`; vỏ ở `commands/glossary.rs:1386`.
  - 11.4 L5882: ca đếm 6 lời gọi, nên dời một lời gọi sang nhánh khác vẫn đếm đủ 6.
  - 11.5 L12508 và 11.7 L12586: test gọi thẳng `commands::segment::promote_ai_translation`; vỏ `segment.rs:3865` chỉ được quét chữ tham số.
  - 11.7 L12304: `batch_panicked_error` chỉ canh hàm dựng lỗi; doc-comment tự nói điều đó, và đã sinh một mục cho Amelia.
  - Đóng mà seam không có guard đỏ, lời đóng tự khai: 11.5 L252 (backup `fs::copy`), L4065 (`ORDER BY`), L10678 (`file_len`), L140 (e2e gỡ ba dòng vẫn xanh), 11.6 L4543.
  - Chạm dây thật: 11.7 L12197/L12255 (khuôn MockRuntime, `tests/ai_translate_wire.rs`); 11.5 L3788 (#96, đi qua `read_open_chapter_segments` → `confirm_segment` → `flush_segment_targets`).
  - 11.6 L731: chỉ nút webview `disabled`; phía Rust vẫn rơi về "Untitled".
  - Xử lý: *hoãn*, gộp với F-D-1 (V-5).
- **F-D-9 (tài liệu nói "không có MockRuntime" sau khi đã có, đã đọc lại mã).** 11.7 lô B thêm `tauri = { …, features = ["test"] }` (`src-tauri/Cargo.toml:193`) và dựng khuôn MockRuntime. Doc-comment ở `commands/project/url_import.rs:57`, `commands/project/mod.rs:219` và `:2726`, `commands/project/work_creation.rs:949` vẫn nói không có nó. Lý do KHÔNG LÀM/kiểm tay của 11.6 (L7443, mục Epic 11 L12593) dựa trên cùng giả định đó. Hai story khác nhau, hai phía một ranh giới: một lý do KHÔNG LÀM đã hết đúng. Xử lý: *sửa ngay* (V-2) và mở lại xét L7443 (V-6).
- **F-D-10 (sổ nói khác mã).** L5401 (11.4) nói "hai giao dịch", nhưng mã HEAD `core/glossary/store.rs:892-900` là một giao dịch ở tầng Work; Triage của spec đã vá, sổ không có 🔵. L6781 (11.4): spec ghi vế (b) KHÔNG LÀM, sổ ghi (b) `Chủ: Ice`, (c) Winston, rồi `Chủ: Epic 11` cuối che cả hai (L6551 cùng kiểu). Doc `# Lỗi` của `promote_ai_translation` (`commands/segment.rs:2137`) thiếu `segment.retired`. Xử lý: V-6.
- **F-D-11 (KHÔNG LÀM mà người dùng vẫn gặp).**
  - 11.4 L5906: `scan_failed` không có UI nghe, không có vị từ `…HasScanned`, nên "chưa có ứng viên" không phân biệt được với lỗi — đúng lớp rỗng im lặng.
  - 11.4 L6937-6986: 128 cặp họ phồn thể không nhập.
  - 11.5 L3682/L3599: khôi phục không trả xuất xứ.
  - 11.6 L9259/L9519/L9859/L10352: bóc nội dung URL chưa đo trên trang truyện thật.
  - 11.7 L4358: next/prev segment không có phím mặc định.
  - Khuôn: 40/41 đúng; 11.6 L9576 thiếu `(Story 11.6)` và điều kiện mở lại. Xử lý: *chấp nhận* theo quyết định đã ghi, trừ L5906 — chính lớp lỗi AGENTS.md gọi là trung tâm, nên nêu lại cho Ice (Q-4).
- **F-D-12 (AC chung thứ tư).** Mục ghi "cần AD mới" nhưng chốt KHÔNG LÀM và không chuyển Winston: 11.5 L2236, L3599, L3682, L3693; 11.6 L7602; 11.4 L5418, L7077. Bốn mục đã chuyển Winston (11.4 L6551, L7149; 11.6 L7578; 11.7 L1224) chưa có `ad-brief-*.md` nào. Xử lý: V-6.
- **F-D-13 (mục kiểm tay mang `Chủ: Epic 11`).** 32 dòng `Chủ: Epic 11` trong sổ: 23 mục thuộc 11.4–11.7, 9 thuộc 11.1–11.3. Trong đó có ba mục năm lượt `workflow_dispatch` nhận từ 11.2 (L4587, L4669, L5538). Nội dung gồm: bản dựng macOS (`find_terms` chậm, ⌘⌥S StatusBar, kéo chuột Glossary, CSV injection, e2e/NFR2 Glossary, tiêu điểm `gm-list`, caret ô rỗng, NFR5 RSS, quét ảnh mồ côi / scope `asset://` / APPEND đua, giãn dòng `.status`, kích thước dock ×2, khung Cài đặt, bộ chọn tầng AI + xuất nhiều bộ prompt, thời gian mở bảng chờ và `open_work`). `epic-11` đang `in-progress` (`sprint-status.yaml:252`). Theo AGENTS.md, Ice chạy một lượt dùng thật trước khi epic lên `done`, và Kiểm C giữ epic không đóng trước lượt đó. Danh sách chuẩn là `grep -n 'Chủ: Epic 11' deferred-work.md` (sổ là nơi duy nhất giữ nó). Sau 11.7 lô B đã có khuôn MockRuntime (F-D-9), nên các mục kiểm tay L9974, L10502, L11247, L12593 (dây `AppHandle` của 11.6) có thể chuyển thành ca tự động.


## Kiểm hành vi

Đã chạy: `npm run test:e2e` trên máy Ice ở HEAD `e390078` (bản debug `--features wdio`, WKWebView thật, macOS). Đây là lượt e2e ĐẦU TIÊN trên `7dbcf23` và `5897669` (11-7): e2e không chạy lúc push, và hai commit đó chưa lên origin.

- Kết quả: **28/29 spec xanh, 1 đỏ**, 3 phút 40 giây.
- **F-B-1 (cao, đã đọc lại mã — ranh giới 11-2 ↔ 11-3).** `attribution-focus.e2e.mjs:89` đỏ với *"[BÀN ĐO HỎNG] `list_dict_sources` ném hoặc trả về một hình dạng không phải mảng"*.
  - 11-3 (`4db199c`) đổi kiểu trả về của lệnh `list_dict_sources` từ `Vec<SourceAttribution>` (`b825a58^:src-tauri/src/commands/dict.rs:299`) sang struct `SourceAttributions { sources, skipped }` (`src-tauri/src/core/dict/mod.rs:996-1003`).
  - Spec e2e mà 11-2 (`c6e5e21`, commit trước đó cùng ngày) vừa dựng lại cho chạy trên runner vẫn hỏi `Array.isArray(sources)` (`e2e/specs/attribution-focus.e2e.mjs:78-83`).
  - Trên CI, lỗi này bị che vì spec đỏ sớm hơn ở dòng 64 (F-CI-2). Vì vậy chỉ sửa V-1 thì nightly vẫn đỏ, và AC11 của Story 1.19 chưa được đo ở đâu kể từ `4db199c`.
  - Không phiên nào thấy cả hai phía: 11-3 không chạy e2e, còn e2e không chạy lúc push.
  - Xử lý: *sửa ngay*, gộp vào V-1.
- 28 spec còn lại xanh, gồm các spec chạm đường 11-4…11-7 (xác nhận segment, dấu, Library, dock). Đó là bằng chứng các luồng ấy không hồi quy theo những gì e2e đo, **không** phủ F-W-1, F-W-2, F-R-1, F-R-2 — không spec nào dựng các kịch bản đó.
- **Chưa kiểm**: bản đóng gói `--release` (từ điển không được đóng gói, AGENTS.md §This machine), Windows, và 20 mục kiểm tay `Chủ: Epic 11` (V-8). Retro không tự bấm tay trong app thật.

## Theo dõi retro trước

Nguồn: `sprint-status.yaml` `action_items` (69 mục; 37 chưa `done` — E1: 5, E2: 6, E3: 11, E5: 7, E6: 6, E10: 2), cùng 9 việc của `epic-4-retro-2026-09-23.md` (không có `id`, chưa từng vào `action_items`). Kiểm bằng mã HEAD, `git grep` trên `b825a58^`, grep sổ nợ. Cả 37 mục đều thuộc một trong các nhóm dưới đây.

Dải Epic 11 không sửa mục nào của `action_items`. Năm mục lật `done` kể từ retro Epic 4 (B6, B10, AI-1…3 và AI-7 của E3) đều lật ở phiên rà nợ 09-23/24, ngoài dải.

**`done` — Ice xác nhận, đã ghi vào `sprint-status.yaml` qua `--set-action-status` (2026-09-30):**

| Định danh | Bằng chứng |
|---|---|
| `epic-3-retro-item-38-ai-4-cho-moi-cong-doc-src-mot-phep-tu-do` | `717196d` (11-1 lô D): `scripts/lib/floor-judge.mjs` + `boundary_scan::assert_population_floor` (`src-tauri/tests/support/boundary_scan.rs:354`, 34 chỗ gọi); Kiểm G của `check:gates` bắt hằng `*_FLOOR` không đi qua bộ đo; 48 sàn = `ceil(0,85 × số thật)` (F-A-2) |
| `epic-6-retro-item-67-ai-6-tach-commands-project-rs-2-044-6-63` | Tách xong ở `cc8b5f0` và `121c178` (phiếu #83); cổng đo kích thước tệp chốt KHÔNG LÀM (phiếu #41, `deferred-work.md:11029`). `wc -l`: `mod.rs` 3 486, `work_creation.rs` 1 865, `wire.rs` 1 678, `tests.rs` 1 230, `url_import.rs` 523, `bilingual.rs` 379 |
| epic 10 · *"📌 De ngo cho Story 10.4: doi nut mo Attribution ra titlebar…"* (mục cũ, không `id`) | Phiếu #90 KHÔNG LÀM (`deferred-work.md:1794`; `sprint-change-proposal-2026-09-24b-phieu-quyet.md:296-297`) |

**Ice đã quyết, còn nửa việc — cần Ice nói "đủ" hay "chưa":**
- `epic-5-retro-item-58-ai-9-doi-nhip-doc-ci-mot-luot-gh-run-lis` và `epic-6-retro-item-60-ai-1-dung-cong-trang-thai-ma-ai-9-retro`: phiếu #37 chọn cảnh báo, không chặn (`.githooks/pre-push:65` → `ci-previous-verdict.mjs || true`, chỉ đọc lượt `push`). F-CI-4 cho thấy cảnh báo đó không ngăn được năm lần lên done khi nightly đỏ. Hai run ID 34780086488, 34898445729 vẫn chưa có nguyên văn trong `spec-ca-wal-do-tren-windows.md`.

**Tệ đi trong Epic 11:**
- `epic-3-retro-item-46-ai-12-gop-guard-lam-moi-dau-glossary-ba`: số bản chép guard tăng 3 → 4. 11-4 (`968529a`) thêm `src/panels/editorPanelState.ts:87` làm chỗ gọi `refreshGlossaryMarks` thứ tư; `main.ts:860` vẫn viết tay. Một story trả nợ đã tạo thêm nợ cùng loại mà retro trước đã gọi tên.
- `epic-3-retro-item-45-ai-11-rut-focusablewithin-traptab-thanh`: 14 → 12 tệp (chỉ nhờ gom Cài đặt), vẫn không có bản dùng chung.
- `epic-3-retro-item-44-ai-10-quyet-cho-dung-cho-398-dong-dieu-p`: khối quét Glossary vẫn ở `commands/project/mod.rs:349-680`. Lượt tách của 11-6 lô A không đụng tới và không ghi lý do.

**Có tiến triển, giữ `in-progress`:** mục E1 *"🔴 BO E2E CHAP CHON…"* (11-2 đóng 22/35 mục e2e, nhưng nightly vẫn đỏ — F-CI-2); `epic-5-retro-item-54-ai-5-chan-doan-10-spec-e2e-do-va-dua-bo` (còn chờ năm lượt `workflow_dispatch`); `epic-6-retro-item-68-ai-8-hoa-giai-quy-uoc-tieu-de-commit-sua` (14/15 commit trong dải mang id `11-N`, không có hook `commit-msg`).

**Không đổi, không thấy Epic 11 chạm:** AI-5 E3 (`TakeTheirs` vẫn ghi một cột, `core/glossary/store.rs:1937-1941`) · AI-6 E3 · AI-8 E3 (0 e2e cho dấu Glossary) · AI-9 E3 · AI-13 E3 (`src/panels/glossaryMarksMap.ts:17` vẫn trỏ `import.rs`) · AI-14 E3 · AI-15 E3 · AI-1/AI-4/AI-6/AI-7 E5 · AI-2/AI-3/AI-10 E6 · B1/B3/B5/B7/B8/B9 E2 · ba mục E1 còn lại.

**Bản trùng với mục nợ có chủ (phiên rà 09-24):** mục E10 FR107 ↔ `deferred-work.md:1676` `Chủ: Story 10.1`; B1 ↔ `:4976` (Winston rồi Story 10.9); mục E1 "Gom … Windows" ↔ B7; mục E1 "28 hang treo 1.20+1.21" ↔ B8. `action_items` và sổ nợ đang giữ hai bản cho cùng một việc.

**Chín việc của retro Epic 4** (không có trong `action_items`):
- E4-1: một nửa — quy tắc "run what the change touches" đã vào AGENTS.md (`a0ddf46`), nhưng override `_bmad/custom/bmad-build.toml` không tồn tại.
- E4-2: không tìm thấy bằng chứng.
- E4-3: đã push, nhưng 4-1…4-12 vẫn `review`.
- E4-4: một phần — `5897669` sửa C4 và gần C6; C1 (`aiTranslateBatchState.ts:220`), C2, C5 (`client.rs:315-316` `unwrap_or(0)`) còn nguyên.
- E4-5: một nửa — phiếu #77/#80 chốt kích thước.
- E4-6: quy tắc watcher có sẵn, không thấy mẫu bàn giao pha.
- E4-7: chưa làm (không có hook `commit-msg`).
- E4-8: chưa làm (`prepare_translate_call` và `prepare_batch_call` vẫn tách đôi).
- E4-9: một nửa — phiếu 108 câu xong; tiêu đề sổ nợ chưa tách.

## Việc cần làm

Mọi mục dưới đây là **đề xuất**; retro không sửa mã hay spec. Mã V-n được các phát hiện ở trên tham chiếu.

| Mã | Việc | Chủ | Loại | Nguồn |
|---|---|---|---|---|
| V-1 | Đưa nightly e2e về xanh: sửa `attribution-focus.e2e.mjs:78-83` theo hình dạng `SourceAttributions` mới (F-B-1); chẩn đoán vì sao `ensureFullLayoutTier()` chỉ đọc được tầng `narrow` trên runner `macos-26` (`attribution-focus.e2e.mjs:64`), rồi chạy năm lượt `workflow_dispatch` trên cùng một commit (Quyết định 3 của 11.2; mục L4587/L4669/L5538) | Amelia | sửa | F-B-1, F-CI-2, F-CI-3 |
| V-2 | Story mới 11.8 (Ice chọn; thêm qua `correct-course`) sửa ranh giới trước Epic 7: nhả `rescanBusy` khi lượt bị vượt, hoặc tách bộ đếm của `loadLibraryOrphans` (F-W-1, cao); tách bộ đếm hoặc đối chiếu `chapter_id` trong `refreshChapterAssetsAfterRegroup` (F-W-2); `confirm_segment` từ chối origin ngoài `TRANSLATION_ORIGINS` (F-R-1); `AppendInProgressGuard` cho nhánh append vào Tác phẩm đang mở (F-R-2); canh `editorPendingPromote` (F-W-3); `set_open_chapter` ở `merge_chapter_into_previous` (F-R-4); fixture `readFixture` mang `assets`/`assets_dir` (F-W-8); sửa bốn doc-comment "không có MockRuntime" (F-D-9) và doc `open_work` (F-R-8). Mỗi guard phải có phép đối chứng đỏ | Amelia | sửa | F-W-1…3, F-W-8, F-R-1/2/4/8, F-D-9 |
| V-3 | Push sau mỗi story, hoặc bấm một lượt `workflow_dispatch` trước khi lên `review`: e2e không chạy ở `push`, nên story chưa lên origin thì chưa được đo, và ba đêm đỏ 09-26…28 đo trên mã cũ | Ice | quy trình | F-CI-1, F-CI-4 |
| V-4 | Đo bài đọc bắt buộc của ba agent cài đặt lớn nhất (966 k ở 11-6 lô B, 781 k ở 11-1 lô B, 744 k ở 11-4) trước khi đổi khuôn pha. Theo bài học đã ghi, lever là bài đọc, không phải lát việc | Ice | quy trình | F-P-1, F-P-2 |
| V-5 | Guard canh dây thay cho guard canh hàm: ép cả 18 `*_boundary.rs` đi qua `boundary_scan` (DW:316/320); ca vỏ `#[tauri::command]` cho `promote_ai_translation` qua MockRuntime (L12508/L12586); ca EXPLAIN QUERY PLAN đọc SQL từ `query.rs` (DW:438); ca ③ `hanviet-segmenter-webkit` phân biệt được hai nhánh (DW:985) | Murat | sửa | F-D-1, F-D-8 |
| V-6 | Đối soát sổ nợ tại chỗ bằng 🔵: câu đóng DW:1054 (`onCopy`), L5401 (một giao dịch), L6781/L6551 (chủ cuối che vế KHÔNG LÀM), L9576 và DW:12162 (thiếu nhãn Story), 13 mục KHÔNG LÀM còn kèm `Chủ: Ice`; mở lại xét L7443 vì đã có MockRuntime | Amelia | đối soát spec | F-D-3, F-D-5, F-D-9, F-D-10 |
| V-7 | Các mục ghi "cần AD mới" mà chốt KHÔNG LÀM không qua Winston (11.5 L2236/L3599/L3682/L3693, 11.6 L7602, 11.4 L5418/L7077, 11.1 DW:569): Winston xác nhận không cần AD, hoặc mở `ad-brief`; thêm ad-brief cho họ NFC/NFD (11.3, ba mục) | Winston | đối soát spec | F-D-5, F-D-12 |
| V-9 | `scripts/ci-previous-verdict.mjs` đọc cả lượt `schedule` gần nhất và in ra khi nó đỏ; vẫn là cảnh báo không chặn (giữ phiếu #37), không dựng cổng trên sprint-status | Amelia | sửa | F-CI-4, F-CI-5 |
| V-10 | Ghi các phát hiện *hoãn* vào `deferred-work.md`, mỗi mục ≤ 5 dòng, có chủ cụ thể: F-R-3 (khoá `OpenWorkState` qua việc dài — đo trước), F-R-5, F-R-6, F-W-4, F-W-6, F-W-7, F-W-9, F-A-3 (`matching_close_brace` ×2, helper test chép lại) | Amelia | hoãn | các mục *hoãn* ở §Phát hiện |
| V-8 | Lượt dùng thật của Ice cho các mục `Chủ: Epic 11` (32 dòng; 20 kiểm tay trên bản đóng gói macOS) trước khi `epic-11` lên `done` | Ice | nghiệm thu | F-D-13 |

## Phán quyết nghiệm thu

**accepted-with-open-items** — Ice quyết trong phiên retro 2026-09-30. Phán quyết máy đề xuất trùng với quyết định này. Tiêu chí: **khai báo** (bốn khối AC chung, `epics.md` §Epic 11).

| AC chung | Kết quả | Bằng chứng |
|---|---|---|
| 1. Task 0 đọc lại từng mục trên HEAD | đạt | tệp `11-{2..7}-task0-*.md`; 11-1 đọc lại trong spec bốn lô |
| 2. Mỗi mục kết thúc bằng ✅ / KHÔNG LÀM / chuyển chủ có lý do; Kiểm C đỏ nếu còn mở | đạt | `check:debt-owner` xanh; hai lượt kiểm sổ đếm 0 mục mở mang `Chủ: Story 11.N` trên 319 mục; lệch khuôn nhỏ ở F-D-5 |
| 3. Mục treo điều kiện chưa xảy ra: không dựng mã phòng trước | không kiểm sâu | ngoài phạm vi hai lượt lấy mẫu |
| 4. Mục đòi đổi bất biến kiến trúc: chuyển Winston | lệch nhẹ | khoảng 9 mục ghi "cần AD" chốt KHÔNG LÀM mà không qua Winston (F-D-5, F-D-12) → V-7 |

Mục mở đi kèm phán quyết (đều có chủ):
- Hai lỗi mức cao do chính epic sinh ra: F-W-1 (nút Quét lại kẹt) → V-2 / Story 11.8; F-B-1 (nightly đỏ vì hình dạng `list_dict_sources`) → V-1.
- Nightly e2e đỏ từ 09-23 tới nay (F-CI-2).
- `epic-11` vẫn `in-progress` cho tới khi Ice chạy lượt dùng thật (V-8). Kiểm C giữ epic không đóng trước lượt đó. Retro lên `done` nghĩa là retro đã chạy, không phải epic đã đóng.

## Câu hỏi mở

- ✅ **Q-1** (Ice chọn: mở rộng cảnh báo đọc cả `schedule` → V-9). Năm lần lên `review`/`done` khi nightly đỏ mà không ghi lý do (F-CI-4). Đây là lần lặp thứ tư của cùng lớp lỗi. Cảnh báo `ci-previous-verdict.mjs` (phiếu #37) có đủ không, khi nó chỉ đọc lượt `push`?
- ✅ **Q-2** (Ice chọn: Story 11.8). V-2 là một story mới (ví dụ 11.8) trong Epic 11, hay các mục nợ giao cho Story 7.x đầu tiên chạm vào cùng tệp?
- **Q-3.** Hạ `RS_FILE_FLOOR` 24 → 21 (F-W-5) có phải chủ ý theo luật 85 % không?
- **Q-4.** L5906 (`scan_failed` không có UI nghe, "chưa có ứng viên" lẫn với lỗi) đã chốt KHÔNG LÀM; đây đúng là lớp rỗng im lặng. Giữ quyết định hay mở lại?
- **Q-5.** `action_items` và sổ nợ đang giữ hai bản cho bốn việc (FR107, B1, B7, B8). Đóng bản trong `action_items` và để sổ làm nguồn duy nhất?
