---
epic: epic-hoan-thien-giao-dien-theo-mockup
date: 2026-10-19
verdict: rejected
criteria: declared
headless: false
---

# Retrospective — Epic 12: Hoàn thiện giao diện theo mockup

## Epic summary

- Epic: `epic-hoan-thien-giao-dien-theo-mockup` (id 12), 22 story, `pending_tickets` rỗng. Cả 22 plan ở `status: done`: 12.1 đã `done` từ trước, 21 story còn lại Ice lật từ `built` sang `done` trong phiên này, trước khi chạy retro. Không còn story nào ở `built`.
- Done when: có khai báo, 5 tiêu chí (`epic-hoan-thien-giao-dien-theo-mockup.md` §Done when).
- Toàn epic: `c35ef91..e93c5ff`, 38 commit, 0 merge. Mã + test (`src/`, `src-tauri/`, `e2e/`, `tests/`, `scripts/`, `.github/`, `.githooks/`): 470 tệp, +49.333/−6.215. Diff thô của range là 801 tệp; phần chênh là tài liệu, trong đó có lượt chuyển cây ticket của migration-2.
- Range theo plan, xếp theo thứ tự baseline (story cuối suy ra tới HEAD):
  12.1 `c35ef91..fe08b04` · 12.2 `fe08b04..26975a9` · 12.3 `26975a9..c521fd6` · 12.4 `c521fd6..68f609d` · 12.7 `68f609d..de7c67e` · 12.9 `de7c67e..23b2981` · 12.8 `23b2981..6bff1b8` · 12.5 `6bff1b8..b6137b0` · 12.6 `b6137b0..26a731a` · 12.10 `26a731a..b9aca37` · 12.11 `b9aca37..9e13ca1` · 12.12 `9e13ca1..78bf298` · 12.14 `78bf298..1b50f51` · 12.13 `1b50f51..a48123e` · 12.15 `a48123e..9dcd7fe` · 12.16 `9dcd7fe..3a48193` · 12.17 `3a48193..e5ac8e6` · 12.18 `e5ac8e6..2fe5328` · 12.19 `2fe5328..c2fd3f4` · 12.20 `c2fd3f4..0dc7b46` · 12.21 `0dc7b46..1a212e1` · 12.22 `1a212e1..e93c5ff` (suy ra).
- Phần cắt ra: range 12.21 chứa `8713931`, `bbe8dfa`, `1a212e1`, là migration-2 của BMad v7 cùng bản sửa cổng của nó, không thuộc việc Epic 12. Các commit cổng/định dạng xen giữa (`a8489d2`, `fe08b04`, `b498a11`, `d4ce1fb`, `bc821d7`, `9dcd7fe`, `3081a9c`) thuộc range của story có baseline đứng trước chúng.
- Bằng chứng có: epic file, 22 plan, 2 story file (12.1, 12.2), các thư mục `handoff-*-phases`, commit message, `gh run list`.
- Bằng chứng thiếu hoặc bị thu hẹp:
  - Không đọc session log, nên bài học quy trình chỉ rút từ plan, handoff và commit.
  - Review diff chỉ phủ ranh giới story, theo lựa chọn của Ice: 32 tệp sản phẩm mà ≥3 story cùng sửa. Trong đó 17 tệp được đọc diff thực chất, 11 tệp đọc một phần; 438 tệp còn lại chưa được review ở cấp epic. Lens adversarial chỉ ra 4 finding đứng được, dưới ngưỡng tự đặt của lens.
  - Đồ thị import chỉ dựng cho `src/` (script, 202 tệp, 955 cạnh). Rust chỉ grep `use crate::` trong các dòng thêm.
  - Không chạy thật app, theo lựa chọn của Ice.

## Findings

Ký hiệu nguồn: `dw:N` là dòng N của `deferred-work.md`.

### Quy trình / bằng chứng

- **P1. 34 commit của Epic 12 chưa lên remote.** `origin/master` = `fe08b04`, `master` đi trước 34 commit. Sáu lượt CI `schedule` 2026-10-13..18 đều chạy trên `fe08b04`. Vì vậy từ 12.2 trở đi, phần Windows, lượt UTC và nightly e2e chưa từng chạy, và điều kiện "đọc CI + nightly trước `done`" của AGENTS.md chưa thể đạt cho 21 story. Nguồn: `git rev-list --count origin/master..master` = 34; `gh run list --event schedule`. Xử lý: **sửa ngay** (push, đọc CI). Phòng lần sau: push sau mỗi story để CI có thể đỏ trong lúc epic còn mở.
- **P2. Bảy spec e2e ghi "chưa chạy".** Gồm 12-10 (cập nhật), 12-13, 12-14 (hai spec), 12-15, 12-16, 12-17, đều vì webview đứng ở `tauri://localhost` trong sandbox; 12.11 và 12.12 không có spec e2e. Nguồn: commit `b9aca37`, `1b50f51`, `a48123e`, `31d6087`, `3a48193`, `e5ac8e6`; `dw:13248`. Xử lý: **sửa ngay**, chạy ngoài sandbox hoặc chờ nightly sau P1.
- **P3. Lựa chọn hai phương án mà không có số đo.** B22 (12.11 ghi "Không đo hai phương án"), A10, A19 (12.3). Epic Notes đòi "story đo rồi đưa Ice chọn", AGENTS.md cũng có luật "hai phương án hợp lệ ⇒ trình cả hai kèm số đo". Nguồn: plan 12.11 dòng 28, commit `9e13ca1`; plan 12.3 dòng 54–55, commit `c521fd6`. Xử lý: **chấp nhận** (Ice đã chọn). Phòng lần sau: plan phải có ô "số đo hai phương án" thì mới được qua bước chọn.
- **P4. Thứ tự NFR15 bị đảo.** Hàng Stack của `is-wsl`/`is-docker` ghi sau khi đã vào `Cargo.lock`, commit tự khai. Nguồn: `c521fd6`. Xử lý: **chấp nhận**.

### Review ranh giới story (`bmad-review`, ba lens)

- **R1. Lô "đưa bản dịch AI sang Editor" có thể ghi vào Tác phẩm khác.** Đã xác minh ở mức mã, chưa chạy thật.
  - `promoteAiTranslationBatchToEditor` và `settlePendingPromoteBatch` lặp `await promoteAiTranslation(segmentId, …)` trên một bản sao cục bộ của `items`, không có dấu danh tính Tác phẩm hay sequence guard (`src/panels/editorPanelState.ts:432-515`).
  - Khi đổi Tác phẩm, đường reset (`editorPanelState.ts:1126-1129`) xoá `pendingPromoteBatch` và hạ `promoteBatchBusy`, nhưng vòng lặp đang chạy vẫn tiếp tục.
  - Phía Rust ghi theo `OpenWorkState` hiện tại (`src-tauri/src/commands/segment/wire.rs:363-377`). Các câu còn lại của lô vì thế ghi vào `segment.id` trùng số của Tác phẩm mới, kể cả nhánh `force=true`. Đây là lớp lỗi im lặng.
  - Ranh giới: 12.11 (`9e13ca1`) với đường đổi Tác phẩm của 12.2/12.4 (`libraryChapters.ts:370-396`).
  - Xử lý: **sửa ngay**. Phòng lần sau: mọi vòng `await` ghi theo `segment_id` mang danh tính Tác phẩm và dừng khi nó đổi.
- **R2. Thông báo "đã chốt, Hoàn tác" của dải Glossary che dải TM và Proofreader của cùng câu.** Thông báo này đứng đầu thứ tự ưu tiên, nên hai dải kia bị che cho tới khi caret rời câu; lệnh focus dải TM thành no-op mà không báo gì. Nguồn: `src/inlineStripEligibility.ts:32`, `src/glossaryConfirmStripState.ts:207-220`; ranh giới 12.20 (`0dc7b46`) với 12.9 (`a92008f`). Mức tin cậy: có khả năng. Xử lý: **hoãn**, Ice quyết.
- **R3. Menu `Công cụ ▾` mở bằng lệnh không đóng khi lớp phủ khác mở hoặc khi đổi chế độ.** `resetToolsMenu` không có chỗ gọi sản phẩm, và `isBlocked` không có `toolsMenuIsOpen`. Nguồn: `src/TitlebarToolsMenu.vue:51`, `src/titlebarToolsState.ts:16`, `src/main.ts:1119`; ranh giới 12.1 (`c8b37b1`) với 12.21 (`348e4e8`). Mức tin cậy: có khả năng. Xử lý: **hoãn**, gộp vào A3.
- **R4. `check_glossary_lifecycle` chỉ kiểm chuỗi con `pending_confirmed_at` trong SQL của trigger**, nên một trigger sai nghĩa vẫn qua cổng. Nguồn: `src-tauri/src/core/store/schema.rs:2984`, `0dc7b46`. Xử lý: **hoãn**.
- Đã kiểm, sạch: phím mặc định trùng (không có), thứ tự migration Project 35..40 và Global 14, `INSERT INTO segment_version` chỉ một chỗ và có `translation_origin` (`commands/segment/confirm.rs:283`), xoá `segment_unlinked` cùng giao dịch, mọi đường đọc TM qua `TmSidePolicy`, import khi trang Tác phẩm đang mở.

### Aggregate views

**Phình tệp**
- **A-S1. `src/commands/index.ts` 4.702 → 5.465 dòng (+763), epic thêm 62 command.** `registerAll` là một hàm khoảng 3.900 dòng (`:1346-5267`). Các story góp nhiều nhất: 12.4 +211, 12.13 +104, 12.12 +96. Đã có tiền lệ tách ra `libraryWorkPageCommandDeps.ts` và `aiPromptInspectorHandlers.ts`, nhưng các story khác không theo. Xử lý: **hoãn**, thành một story tách.
- **A-S2. Logic nằm trong component.** `TmManageOverlay.vue` 993 → 1.525 (12.14), `AiPromptInspectorOverlay.vue` 557 → 1.016 (12.12); `GlossaryManageOverlay`, `PromptLibraryOverlay`, `SettingsOverlay` cũng sát hoặc vượt 1.000. Xử lý: **hoãn**.
- **A-S3. `src-tauri/src/core/library/indexer.rs` +566 (12.5 +515).** Phần tìm kiếm khoảng 1.200 dòng nằm chung tệp với phần rebuild. Xử lý: **hoãn**.
- Sạch: `LibraryWorkPage.vue` (1.256 dòng, script 215 dòng, còn lại là template và CSS); `LibraryMode.vue` giảm 724 dòng; `library_index_contract.rs` lớn có lý do.

**Trùng lặp**
- **A-D1. Khôi phục tiêu điểm cộng `trapTab` được chép tay ở 20–21 lớp phủ.** Epic thêm 5 bản: `ChapterMergeOverlay`, `ChapterSplitOverlay`, `GlossaryHarvestOverlay`, `LibraryImportOverlay`, `LibraryUrlImportScreen`. Xử lý: **hoãn**, gộp vào A3.
- **A-D2. Danh sách `reset*` khi đổi Tác phẩm viết hai lần.** Ở `src/modes/libraryChapters.ts:393-397` và `src/modes/libraryImport.ts:464-468` (đã kiểm tay); `e93c5ff` (12.22) phải thêm vào cả hai nơi. Xử lý: **sửa ngay**, gộp vào A3.
- **A-D3. 8 hàm `reset*` mới không có chỗ gọi sản phẩm.** `layoutPresetState` (2 hàm), `titlebarToolsState`, `aiUsageVisibleState`, `glossaryHarvestState`, `chapterMergeState`, `chapterSplitState`, `libraryImportOverlayState`. Nối reset của Merge/Split vào `libraryChapters.ts` sẽ tạo vòng import value. Liên quan R3. Xử lý: **sửa ngay**, gộp vào A3.
- **A-D4. `glossaryHarvestState.ts` (12.13) là bản thứ ba của máy trạng thái nạp** (`sequence`, thang `unknown/ipc_unavailable/error/no_work/loaded`), sau `glossaryQueueState` và `glossaryManageState`. Xử lý: **hoãn**.
- **A-D5. Nhãn `translation_origin` có ba họ khoá i18n và ba bảng ánh xạ TS** (`mode.reading.origin_*` của 12.15, `history.origin_*` của 12.19, `tm.manage.origin_*`). Xử lý: **hoãn**.

**Lệch quy ước**
- **A-C1. `toLocaleString('vi-VN')` trái tiền lệ NFR16 của `AiTranslationPanel.vue:242`.** Ở `TmManageOverlay.vue:231` (12.14), `GlossaryQueueOverlay.vue:88` (12.13), `libraryRescan.ts:106` (12.3); đã kiểm tay cả ba. Xử lý: **sửa ngay**, nhỏ.
- **A-C2. `core/export/options.rs:4` import `commands::segment::ChapterSegment`** (12.17, `e5ac8e6`), là bản thứ 6 của chiều `core → commands`. Thêm hai cạnh `commands → commands` vào `aitranslate.rs`. Xử lý: **hoãn**.
- **A-C3. Ba `eslint-disable` mới có lý do** (`bootstrap.ts:315,317`, `shortcuts.ts:19`, 12.18); test `src-tauri/tests/story_22_measure_export_tm.rs` mang id story trong tên; vài chú thích mang `AC2`/`AC6`/lịch sử review (`GridPanel.vue:2148,2692`, `dict/query.rs:46`, `schema.rs:1406`). Xử lý: **chấp nhận**, dọn khi chạm tới.
- Sạch: từ vựng cấm, định danh `origin` trần, `v-html`, `@click` ngoài `dispatch`, `invoke` ngoài `config/`.

**Kiến trúc**
- 0 vòng import (value và type) ở cả `c35ef91` lẫn `e93c5ff`. Năm tệp mà cổng `import()` không có value import từ `vue`/`dockview`. Miền Library chia hai chỗ `src/` và `src/modes/`, với hai hub là `StatusBar.vue` (fan-out 6 → 17) và `libraryChapters.ts` (20 → 27). Đây là hub, không phải vòng. Xử lý: **chấp nhận**.

### Đối chiếu spec

- **S1. Done when 1 mới đạt một phần.** Thanh tiêu đề đúng hình (`src/App.vue:202-263`, `c8b37b1`). Nhưng Review Mode không có lối mở nhìn thấy: không có `dispatch('review.open')` nào trong `src/` (đã kiểm), chỉ còn `Mod+Alt+3`. Open question cuối epic Notes về chuyện này còn một nửa chưa ai nhận. Tiêu điểm trả về nút mở chưa được xác minh trên app thật (`dw:13111`). Xử lý: **sửa ngay hoặc đổi spec**, Ice quyết.
- **S2. Spine cần cập nhật cho khớp mã:**
  - AD-55 mục 7 chưa có `source_text` mà 12.19 thêm vào (`c2fd3f4`).
  - AD-54 mục 5 ghi `glossary.not_found`, mã dựng dùng `glossary.entry_missing` (`0dc7b46`).
  - Cơ chế "bản xuất có đánh số/dấu không nhập lại được" (`aura.export_annotated`, `export.annotated_copy_not_reimportable`) không có chữ nào trong PRD, spine hay UX (`e5ac8e6`).

  Xử lý: **đề xuất cho Winston**.
- **S3. PRD FR139 ghi "hướng dịch", nhưng chỉ mục không có cột ngôn ngữ đích**, nên bản dựng lọc theo `source_lang → Việt` (`b6137b0`). Xử lý: **đề xuất làm rõ PRD**.
- **S4. Mô tả entry 15 lệch với bản dựng.** Entry đặt "Sửa lần lượt" dưới Proofreader; bản dựng làm ở cả lớp phủ dấu Chế độ đọc lẫn Workspace (`31d6087`). Xử lý: **đề xuất cập nhật entry**.
- **S5. Màn gộp Chương hứa "không mất gì".** Review 12.4 bắt được rằng tên, xuất xứ và bản reviewer của Chương bị gộp bị xoá; hướng xử lý nằm trong `68f609d`, nhưng chưa ai đối chiếu với mã cuối. Xử lý: **xác minh** (A5).
- **S6. Hai mục nợ đã dựng xong nhưng sổ chưa đóng bằng lời:** `dw:671` (A20, 12.7, `de7c67e`) và `dw:6064` (B18, 12.20, `0dc7b46`). Xử lý: **sửa ngay** (sổ sách).
- Đạt, có nguồn:
  - Done when 2: 122 dòng loại 2 đều có mã dòng trong `tickets.toml`; 63 dòng loại 5 mỗi dòng đúng một đích, 13 dòng còn lại là loại 3. Việc map từng dòng về mã không được kiểm.
  - Cửa của Done when 3: AD-54..57 vào spine ở `0bb6c81`, trước 12.14/12.15/12.19/12.20. 17 FR đều có story, commit và test theo tên; chạy thật thì chưa.
  - Done when 4: 12.9 đo trên WKWebView bản debug, Chương 2.000 câu: dải mọc median 193 ms, thu 222 ms (`a92008f`, plan 12.9 dòng 108). Chưa có số trên bản đóng gói.

### Nợ `Chủ: Epic 12` còn mở hoặc 🟡: 35 mục (30 mở, 5 🟡)

- 20 mục chờ lượt dùng thật của Ice trên bản đóng gói macOS: `dw:13111`, `13119`, `13123`, `13147`, `13155`, `13163`, `13171`, `13183`, `13199`, `13203`, `13212`, `13233`, `13244`, `13252`, `13256`, `13260`, `13269`, `13274` 🟡, `13279` 🟡, `13284` 🟡.
- 7 mục thiếu chức năng chờ Ice quyết làm hay KHÔNG LÀM: `dw:13131` (tay kéo ⠿), `13135` (đổi tên hàng loạt), `13139` (ngày đặt ghi đè), `13143` (gộp không `⌘Z`), `13167` (tên ngắn nguồn), `13175` (phím mở kết quả tìm), `13179` (Esc đóng tìm).
- 4 mục code: `dw:13151` (`is_omitted` đọc thành "chưa dịch"), `13159` (`onStripFocusOut` có thể làm con trỏ thành `null`), `13289` và `13293` (tiêu điểm Attribution).
- 1 mục 🟡 code: `dw:13237` (`tm_export_counts` đổi sang `COUNT` SQL, `commands/tm.rs:644`).
- 1 mục chạy e2e: `dw:13248`.
- 2 mục sổ sách (S6): `dw:671`, `dw:6064`.

## Behavior verification

Runtime chưa được chạy thật trong retro này (Ice chọn "Không kiểm"). Cũng không có bằng chứng gián tiếp từ CI hay nightly cho 12.2–12.22 (xem P1). Bằng chứng hành vi duy nhất là các lượt test do từng story tự khai trong commit message, ví dụ full suite của 12.22: 14 cổng, vitest 2.982 ca, `cargo test` 2.650 ca, chạy trên máy Ice ở `e93c5ff`.

## Previous-retro follow-through

Epic liền trước theo thứ tự thực thi trong `tickets/tickets.toml` là Epic 9 (`epic-ai-proofreader`). Thư mục của nó không có tệp `epic-ai-proofreader-retrospective.md`, tức là không có tệp retro nào, chứ không phải "có tệp nhưng không còn việc tồn". Vì vậy không có action item nào cần theo dõi.

## Action items

Mọi mục dưới đây là **đề xuất**; retro không sửa mã, spec hay sổ nợ.

1. **Push 34 commit và đọc CI, gồm phần Windows và lượt UTC.** Chờ nightly `schedule` chạy trên `e93c5ff` rồi ghi lý do cho mọi chỗ đỏ (P1, P2). Chủ: Ice.
2. **Sửa R1:** gắn danh tính Tác phẩm (hoặc sequence) vào hai vòng lô PROMOTE, dừng lô khi Tác phẩm đổi, kèm ca canh đỏ khi gỡ guard. Hình dạng story: `bmad-build`, nhỏ. Chủ: Epic 12.
3. **Story dọn teardown cấp ứng dụng.** Gom danh sách `reset*` khi đổi Tác phẩm về một nơi (A-D2), nối hoặc gỡ 8 hàm `reset*` mồ côi mà không tạo vòng import (A-D3), đóng menu `Công cụ ▾` qua `isBlocked`/reset (R3), và cân nhắc gom `trapTab`/khôi phục tiêu điểm (A-D1). Có thể gộp hai nợ tiêu điểm `dw:13289`, `dw:13293`. Chủ: Epic 12.
4. **Sửa các mục code nhỏ trong sổ:** `dw:13151` (`state_omitted`), `dw:13237` (`COUNT` SQL), A-C1 (`toLocaleString`); đo `dw:13159` trên WKWebView. Chủ: Epic 12.
5. **Xác minh S5:** gộp Chương có giữ tên, xuất xứ và bản reviewer của Chương bị gộp như màn hứa không. Chủ: Epic 12.
6. **Quyết định của Ice:**
   - Review Mode có nút nhìn thấy, hay đổi cách đọc Done when 1 (S1).
   - Bảy mục thiếu chức năng (`dw:13131`–`13179`): mỗi mục làm hay KHÔNG LÀM.
   - R2: giữ hay đổi thứ tự ưu tiên của dải.

   Chủ: Ice.
7. **Lượt dùng thật trên bản đóng gói macOS** cho 20 mục chờ, rồi đóng từng mục bằng lời (Done when 5). Chủ: Ice.
8. **Đóng hai mục sổ sách S6** (`dw:671`, `dw:6064`). Chủ: Epic 12.
9. **Đối chiếu spec**, chờ người áp dụng:
   - Winston: AD-55 mục 7 thêm `source_text`, AD-54 mục 5 sửa mã lỗi, ghi cơ chế xuất không nhập lại được (S2).
   - PRD FR139 làm rõ "hướng dịch" (S3).
   - Sofia: entry 15 (S4).
10. **Hoãn, ghi thành nợ có chủ:** tách `registerAll`/`CommandDeps` (A-S1), đưa logic ra khỏi các lớp phủ trên ~1.000 dòng (A-S2), tách phần tìm kiếm khỏi `indexer.rs` (A-S3), gom máy trạng thái nạp (A-D4) và nhãn xuất xứ (A-D5), tầng `core → commands` (A-C2), cổng R4. Chủ: Winston (A-C2, A-S1), Epic 10 (phần còn lại) hoặc story dọn sau, do Ice chọn.
11. **Bài học quy trình:**
    - Push sau mỗi story để CI có thể đỏ trong lúc epic còn mở (P1).
    - Plan có lựa chọn hai phương án phải có số đo trước khi trình Ice (P3).
    - Story nào thêm một lớp phủ phải nối reset và tiêu điểm vào chỗ dùng chung, không chép tay (A-D1, A-D3).

    Chủ: Ice (đưa vào AGENTS.md qua `bmad-project-context` nếu Ice muốn).

## Acceptance verdict

**Phán quyết máy: rejected** (Epic 12 không được chấp nhận). Tiêu chí có khai báo. Mọi ticket đều đã xong, nên phán quyết này không đến từ ticket dở dang mà từ tiêu chí chưa đạt:

- Done when 5 chưa đạt: chưa có lượt dùng thật trên bản đóng gói macOS, và 35 mục nợ `Chủ: Epic 12` còn mở hoặc 🟡.
- Done when 1 đạt một phần: Review Mode không có lối mở nhìn thấy, và tiêu điểm trả về chưa xác minh trên app thật.
- Done when 3 mới đạt phần cửa AD; chưa có FR nào được xác minh chạy trong app thật, và CI/nightly chưa chạy trên 34 commit (P1).
- Còn một finding chặn: R1, ghi nhầm Tác phẩm một cách im lặng.
- Done when 2 và 4 đạt, có nguồn.

Chưa có quyết định của người. Ice có thể ghi đè phán quyết; khi đó cập nhật cả `verdict` ở frontmatter.

## Open questions

- Ice có muốn sửa R1 cùng các mục code trong Epic 12 trước khi đóng, hay đóng Epic 12 rồi chuyển chúng sang một chủ khác? Câu trả lời đổi phán quyết và chủ của action item 2–5.
- Review Mode có cần nút nhìn thấy để Done when 1 đạt không?
- Sau khi push, CI phần Windows và nightly e2e có xanh trên `e93c5ff` không? Retro này chưa trả lời được.
- R2, R3 và R4 mới ở mức "có khả năng"; chưa ai tái hiện trên app thật.
