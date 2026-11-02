---
title: 'Story 6.23 — Màn xem trước nhập: chân dính có nút Nhập, cột cấu hình dính, khối loại gộp thành dải'
type: 'feature'
ticket: 23
created: '2026-11-02'
status: 'built'
baseline_revision: 'b3ab13b1461b11c97491874b1c1ced99509f9a18'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/import-preview-gon.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Trên màn xem trước nhập, nút xác nhận nằm dưới cùng danh sách khối: một bài epochtimes.com bóc ra 301 khối thì phải cuộn 18 955 px mới tới nút. Đường URL còn bị ép một cột, và ô mẫu phân tách Chương hiện ra ở đường URL dù gõ vào không đổi gì (trái AD-39).

**Approach:** Mọi đường nhập dùng hai cột: cột cấu hình dính bên trái, cột văn bản bên phải, cùng một chân dính mang `Huỷ` · `Nhập N Chương ⌥⌘↵`. Khối loại liền nhau gộp thành dải mở được. Bảng mã tin cậy cao thu còn một dòng. Lượt một Chương ẩn khối Tách Chương. Ô mẫu phân tách hiện theo cờ `already_chaptered` mà Rust gửi trong `ChapterSplitPreviewWire`.

## Boundaries & Constraints

**Always:** Giữ thứ tự pipeline AD-39 và việc không ghi gì xuống đĩa trước khi xác nhận. Cùng đầu vào và cùng thao tác sửa thì Tác phẩm tạo ra giống hệt trước story này. Cờ hình dạng suy ra từ `PipelineShape` và còn đúng cả khi `run_pipeline` trả `Err`. Phím giữ nguyên nghĩa (AC5). Màn song ngữ (FR115) chỉ nhận thêm cờ (luôn `false`), bố cục giữ nguyên.

**Decisions (Ice, 2026-11-02):**
- Tên Tác phẩm sửa được ngay trên màn xem trước. Ô `textarea` tự giãn nằm trong cột cấu hình, chỉ hiện khi đích là Tác phẩm mới. Khi xác nhận, ứng dụng dùng tên đã sửa. Tên rỗng sau `trim()` thì nút tắt, cùng luật mà màn trước đang áp (`LibraryImportOverlay.vue:256`).
- Số ký tự ở chân là tổng `chapters[].length` do webview cộng. Rust không thêm khoá.
- Tầng luật làm sạch chuyển sang cột cấu hình ngay trong story này, kể cả văn bản in lại; story 24 bỏ phần văn bản đó.
- Plan giữ một khối, không tách dải và phím ra story khác.
- Khối vừa loại bằng `Space` vẫn hiện thành một hàng riêng, con trỏ đứng trên nó, nên `Space` lần nữa đảo lại như hôm nay. Khối chỉ gộp vào dải khi con trỏ rời đi, và dải thu bên cạnh vẫn thu (Ice, vòng review 1).

**Never:** Webview không quyết theo `importPreviewLastSubmittedFrom === 'urls'` để hiện ô mẫu. Story này không bỏ phần văn bản in lại của tầng luật và không gạch ngang tại chỗ (đó là story 24). Không đổi bốn trường xuất xứ, nhật ký domain, danh sách link với `Bỏ`/tải lại, hay việc chọn Tác phẩm đã có. Không đổi `ChapterOrigin.vue` theo cách làm hỏng trang Tác phẩm.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected |
|---|---|---|
| URL nhiều khối loại | 1 link, ≥ 200 khối loại, có khối giữ xen giữa | Mỗi dãy khối loại liền nhau thành 1 dải (kể cả dãy chỉ 1 khối); khối giữ hiện đủ; chân thấy được mà không cần cuộn |
| Giữ khối trong dải | Mở dải 98 khối, `Giữ khối này` ở khối 40 | Dải tách thành 39 + 58, khối 40 hiện đủ |
| J/K qua dải | Con trỏ ở khối giữ ngay trước một dải đang thu | `J` dừng ở dải; `J` lần nữa qua tới khối sau dải; `Space` mở/thu dải |
| Một Chương | `.txt` không khớp mẫu, hoặc 1 link | Không có khối Tách Chương, bộ đếm hay `⌥W`; chân ghi "Nhập 1 Chương" |
| Nhiều Chương | `.txt` khớp mẫu 3 Chương | Con trỏ Chương, bộ đếm *cần xem · sạch* và `⌥W` ở đầu cột văn bản |
| Ô mẫu | URL (`already_chaptered=true`) / `.txt`, dán tay, nhiều tệp, song ngữ (`false`) | URL: không có ô · các đường còn lại: có ô, gõ mẫu thì số Chương đổi như hôm nay |
| Bảng mã | `high` / `low` / `self_declared` | `high`: một dòng "tên · tin cậy cao · Đổi E" · `low`: dải năm ứng viên mở sẵn · `self_declared`: như hôm nay |
| Nút tắt | danh sách link có link hỏng, hoặc đang xác nhận/tải | Nút trong chân tắt theo đúng điều kiện hôm nay; `⌥⌘↵` không làm gì |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/segment/pipeline.rs:553-568,755-816` -- `Flow.already_chaptered` chỉ `true` với `PipelineShape::Chapters` (chỉ đường URL dựng ra shape này: `url_import.rs:381,418`). Thêm `PipelineShape::is_already_chaptered()` và dùng lại nó trong match của `run_pipeline`.
- `src-tauri/src/commands/project/mod.rs:1189-1291` -- `ChapterSplitPreviewWire` (serde snake_case) và `build_chapter_split_preview_wire` có 4 chỗ gọi: `:1826` và `:2047` truyền `shape.is_already_chaptered()`; `:2360` (dán tay, Blob) và `bilingual.rs:402` truyền `false`.
- `src/config/project.ts:283-300,508-520` -- bản TS viết tay của wire cùng guard `isChapterSplitPreviewWire`. Guard kiểm mọi trường, nên mọi fixture TS dựng wire phải có thêm trường mới.
- `src/ImportPreviewOverlay.vue` (2687 dòng; đọc theo khoảng). Các vùng:
  - `:1052`: `.ip-two-stacked`, nơi duy nhất ép một cột.
  - `:1053-1143`: `aside.ip-config`. Ô mẫu `v-if` ở `:1100`.
  - `:1145-1197`: tầng bảng mã.
  - `:1210-1384`: tầng 2 (khối). `ol.ip-blocks` listbox ở `:1295-1373`, con trỏ Chương ở `:1236-1277`.
  - `:1386-1556`: tầng luật.
  - `:1558-1701`: Tách Chương cùng bộ đếm và `⌥W`.
  - `:1703-1721`: Xuất xứ. Tiêu đề in hai lần vì `ChapterOrigin.vue:60` cũng in `<h4>` "Xuất xứ tài liệu".
  - `:1741-1824`: lý do khoá, lỗi, tiến độ ảnh, nhật ký domain, `ip-hint` "chưa ghi gì", `.ip-actions`.
  - Phím: `onTier2Keydown :662-715`, `onChapterCursorKeydown :727`, `onChapterFilterKeydown :761`. `confirmLabel :847`.
- `src/importPreviewState.ts` -- `blockFocusedIndex :294` và `blockRangeStart :295` là chỉ số vào `importPreviewSelectedBlocks.value.blocks` (`:792`; `null` với tệp và văn bản). Các hàm liên quan:
  - `next/prev/setImportPreviewBlockFocus :1302-1317`
  - `toggleImportPreviewBlockKept :1329`
  - `importPreviewStripIsOpen :611` và `openImportPreviewCandidatePicker :619`
  - `importPreviewCanConfirm :531`
  - `pendingName :141` (chỉ đọc, export ở `:442`)
  - đổi mẫu: `setImportPreviewChapterPattern :1999`
- `src/commands/index.ts:1640-1666` -- `import.preview.open_picker` (`E`) và `import.preview.confirm` (`Mod+Alt+Enter`, đã có sẵn). Lệnh khối và Chương ở `:1870-1980`.
- `src/i18n/vi.json` -- `confirm_with_count` ("Xác nhận và nhập {count} Chương"), `hint_no_write_before_confirm`, `tier_origin_title`.
- `src/modes/LibraryMode.vue:382` -- chỗ gắn overlay. Phải kiểm container nào cuộn để `position: sticky` bám được.
- Không đụng: `BilingualImportPreviewOverlay.vue` / `bilingualImportPreviewState.ts` (tách riêng, `bip-*`). Chỉ fixture của chúng nhận cờ.
- Test sẽ đổi:
  - Rust: `src-tauri/tests/segment_contract.rs` (`:10914` ghim bộ 6 khoá, `:10524`), `src-tauri/tests/webimport_contract.rs`.
  - Rust (thêm khi làm): `src-tauri/tests/bilingual_import_contract.rs` (cờ song ngữ là `false`).
  - Vitest mới: `tests/frontend/importPreviewBands.test.ts` (hàm gộp dải), `tests/frontend/importPreviewFooterLayout.test.ts` (các dòng ma trận I/O).
  - Vitest có fixture dựng wire: `tests/frontend/importPreviewChapters.test.ts`, `tests/frontend/importPreviewTwoColumn.test.ts`, `tests/frontend/importPreviewChapterOrigin.test.ts`, `tests/frontend/importPreviewBilingual.test.ts`, `tests/frontend/importPreviewEncoding.test.ts`, `tests/frontend/importPreviewBlocks.test.ts`, `tests/frontend/importPreviewEncodingWireShape.test.ts`, `tests/frontend/importPreviewUrls.test.ts`, `tests/frontend/importPreviewBilingualWireShape.test.ts`, `tests/frontend/importPreviewFiles.test.ts`, `tests/frontend/importPreviewOverlayRender.test.ts`.
  - Vitest đọc cấu trúc DOM: `tests/frontend/importPreviewBlocks.test.ts` (`.ip-block` theo chỉ số), `tests/frontend/importPreviewTwoColumn.test.ts` (`.ip-two-stacked`, thứ tự tầng, nhãn nút), `tests/frontend/importPreviewCleanup.test.ts`.
  - e2e: `e2e/specs/import-preview-mouse.e2e.mjs` (`blockStates()` map mọi `.ip-block`) và `e2e/specs/story-12-6-import-preview.e2e.mjs`.

## Tasks & Acceptance

Theo AGENTS.md, công việc chia bốn pha, mỗi pha một agent mới, bàn giao qua tệp: Rust · Webview cấu trúc · Webview dải và phím · Tests.

**Execution:**
- [ ] `src-tauri/src/core/segment/pipeline.rs`, `src-tauri/src/commands/project/mod.rs`, `src-tauri/src/commands/project/bilingual.rs` -- thêm `already_chaptered: bool` vào wire qua `PipelineShape::is_already_chaptered()`, có giá trị ở cả 4 chỗ gọi -- AC8, cách A.
- [ ] `src-tauri/tests/segment_contract.rs`, `src-tauri/tests/webimport_contract.rs` -- sửa bộ khoá đã ghim; thêm ca: đường URL cho `true` (cả nhánh `Err`), `.txt`/dán tay/nhiều tệp/song ngữ cho `false`.
- [ ] `src/config/project.ts` -- thêm trường vào type và guard.
- [ ] `src/importPreviewState.ts` -- hàm thuần gộp dãy khối loại liền nhau thành dải (`{ kind: 'block', index } | { kind: 'band', start, end }`); trạng thái mở dải; J/K đi qua một dải như một hàng; `Space` trên dải thì mở/thu; giữ một khối thì dải tách và số trên các dải tính lại; computed `importPreviewSplitPatternApplies` đọc cờ từ wire; setter tên cho `pendingName` (`:141`), mà nhánh xác nhận đã đọc sẵn (`:2180`); computed tổng số ký tự; thêm điều kiện "tên rỗng" vào `importPreviewCanConfirm`.
- [ ] `src/ImportPreviewOverlay.vue` -- bỏ `.ip-two-stacked`. Cột cấu hình theo thứ tự trong EXPERIENCE §Bố cục: nguồn, đích, tên, ngôn ngữ, bảng mã, luật làm sạch, xuất xứ; `sticky` và tự cuộn riêng. Cột văn bản: URL hiện tầng 2 với dải; tệp/dán tay hiện Tách Chương và văn bản xem trước. Các thay đổi còn lại:
  - Từ 2 Chương: con trỏ Chương, bộ đếm và `⌥W` ở đầu cột văn bản. 1 Chương: ẩn khối Tách Chương.
  - Bảng mã `high`: một dòng, `Đổi` dispatch `import.preview.open_picker`.
  - Xuất xứ chỉ in một tiêu đề: bỏ `tier_origin_title`, giữ `<h4>` của `ChapterOrigin`.
  - Chân dính chứa lý do khoá, lỗi, tiến độ ảnh, số Chương, số ký tự, ước lượng Glossary, trạng thái ban đầu, "chưa ghi gì xuống đĩa", nhật ký domain, `Huỷ`, `Nhập N Chương ⌥⌘↵`.
  - Ô tên Tác phẩm: `textarea` tự giãn, ghi qua setter mới của `pendingName` trong `importPreviewState.ts`.
  - CSS theo `check:tokens`: `z-index` phải có `aura-allow-z-index`, line-height là bội của 7 px.
- [ ] `src/i18n/vi.json` -- nhãn `Nhập {count} Chương`, nhãn dải, dòng bảng mã. Xoá các khoá không còn dùng.
- [ ] Tests vitest liệt kê ở Code Map -- thêm cờ vào fixture; viết lại các assert DOM theo cấu trúc mới; ca thuần cho hàm gộp dải (dãy 1 khối, tách khi giữ, J/K, Space) và cho `importPreviewSplitPatternApplies`.
- [ ] `e2e/specs/import-preview-mouse.e2e.mjs`, `e2e/specs/story-12-6-import-preview.e2e.mjs`, và spec mới `e2e/specs/story-6-23-import-preview-footer.e2e.mjs` -- máy chủ cục bộ phục vụ một trang có ≥ 200 khối loại, cộng một `.txt` nhiều Chương trong thư mục tạm. Spec mới kiểm: nút trong khung ở đầu và ở cuối trang cuộn, cột cấu hình đứng yên, dải mở được, không có ô mẫu trên đường URL. Dùng `realClick`.

**Acceptance Criteria:** AC 1–9 của story là tiêu chí nghiệm thu, không chép lại ở đây. Thêm:
- Given cờ đã bị gỡ thật ở Rust (luôn `false`), when chạy spec e2e mới, then ca "URL không có ô mẫu" đỏ đúng vì ô hiện lại.
- Given cùng đầu vào và cùng thao tác sửa, when xác nhận trước và sau story này, then các Chương và khối tạo ra giống hệt.

## Implementation Notes

- Chia bốn pha, mỗi pha một agent mới: Rust · cấu trúc webview · dải và phím · tests. Ghi chú bàn giao nằm ngoài repo, trong scratchpad của phiên.
- Cờ được đặt ở `cleanup_and_chapters_preview_for` trước khi `shape` bị move, nên nhánh `run_pipeline` trả `Err` vẫn mang đúng giá trị. Đường dán tay luôn dựng `Blob`, nên `self_declared_chapters.already_chaptered` luôn là `false`.
- Bảng mã: chỉ `high` thu một dòng. Pha 2 từng thu cả `self_declared`; lỗi đó đã sửa (`encodingIsOneLine`), và ô `strip_open_hint` của `self_declared`/`low` khi dải đóng được giữ lại.
- `bandOpenIndices` giữ mọi chỉ số của một dải đang mở, nên khi giữ một khối thì hai nửa của dải vẫn mở. Tập này đặt lại khi chuyển Chương, vì chỉ số khối là riêng từng Chương. Ca đối chứng đỏ đã kiểm: gỡ `watch(chapterCursor)` thì ca đó đỏ.
- `check-commands.mjs`: `HANDLER_ATTR_FLOOR` nâng 168 → 180 (số đếm thật là 211), và thêm `setImportPreviewBandFocus` vào bảng trình xử lý.
- Đối chứng e2e: ép `already_chaptered: false` ở `build_chapter_split_preview_wire` thì ca URL trong `story-6-23-import-preview-footer` đỏ vì ô mẫu hiện lại.

## Plan Change Log

- Review vòng 1 (Ice chọn giữ code và sửa tại chỗ thay vì xoá rồi làm lại):
  - Intent gap: loại một khối bằng `Space` thì khối chui vào dải thu, mất kiểu hoàn tác một phím. Đã thêm quyết định vào §Boundaries.
  - Bad plan: cột cấu hình lệch 43 px ở cuối trang cuộn, vì plan không nói cột dính tính chiều cao chân thế nào. Sửa tại chỗ, và e2e phải đo cả ở cuối trang chứ không chỉ giữa trang.
  - KEEP: cờ Rust và guard TS; bố cục hai cột và chân; `encodingIsOneLine`; đặt lại dải khi chuyển Chương; hàm gộp dải thuần.

## Review Triage Log

Vòng 1 (lens `quick`, cộng 2 phát hiện của chính build). Kết quả: high 0 · medium 2 · low 4 · false 4 · maybe-false 1.

- medium · intent_gap · `importPreviewState.ts` `toggleImportPreviewBlockKept` → `groupImportPreviewBlockRows`. Bấm `Space` để loại một khối thì khối đó gộp vào một dải thu, vì chỉ số đầu dải mới không có trong `bandOpenIndices`. `Space` lần nữa sẽ mở dải chứ không đảo lại khối. Cùng gốc: khi gộp, dải mới lấy trạng thái mở theo chỉ số đầu dải, nên nửa phải đang mở bị thu. Đúng chữ AC5, nhưng "loại xong thì khối hiện ra sao" là lựa chọn plan chưa chốt.
- medium · bad_plan · `ImportPreviewOverlay.vue` `.ip-config`/`.ip-footer`. Ở cuối trang cuộn, cột cấu hình bị đẩy lên 43 px (e2e pha 4 đo top 56 → 13), trái AC2 "đứng yên". Nguyên nhân: chân dính nằm ngoài lưới `.ip-two`, và `max-height` của cột đoán theo `100vh` mà không trừ chiều cao chân. Plan không nói cột dính tính chân thế nào.
- low · patch · `ImportPreviewOverlay.vue`, chú thích CSS trên `.ip-band`: viết tiếng Việt và chỉ nói lại tên class, trái luật chú thích của AGENTS.md. Sửa: xoá.
- low · patch · `vi.json` `tier2_current_chapter_note` vẫn ghi "con trỏ Chương bên dưới", trong khi con trỏ nay ở đầu cột văn bản. Sửa câu.
- low · reject · `bandOpenIndices` không đặt lại khi đổi ứng viên bảng mã. Hiếm gặp, hậu quả chỉ là một vài dải tự mở, và cách sửa là thêm một guard.
- low · reject · Chưa có ca Rust cho URL nhiều Chương kèm link hỏng với `already_chaptered`. Cờ suy thuần từ `PipelineShape`, và nhánh `Err` đã có ca riêng.
- maybe-false · reject · Dòng bảng mã một dòng hiện nhãn rỗng khi `importPreviewSelectedCandidate` là `null`. Cần bằng chứng Rust gửi `selected_encoding` nằm ngoài `candidates`; nếu có thật thì chỉ là `low`.
- false · Tên không `trim()` khi gửi: màn trước cũng gửi `effectiveName` không cắt (`src/modes/libraryImport.ts:617`), đúng luật hôm nay.
- false · Đường URL còn danh sách Tách Chương khi có từ 2 Chương: AC7 vẫn đạt (con trỏ, bộ đếm và `⌥W` ở đầu cột văn bản), danh sách có từ trước và plan không cấm.
- false · `importPreviewCleanup.test.ts` và `webimport_contract.rs` không có trong diff: Cleanup nằm trong 14 tệp của `test:story 6-23` và xanh; `webimport_contract.rs` không dựng wire literal nào.
- false · `HANDLER_ATTR_FLOOR` 180 thấp hơn số thật 211: đây là nâng sàn, cổng chỉ đòi sàn ≥ mức cũ; không hạ ngưỡng nào.
- Kết quả vòng 1 (Ice chọn sửa tại chỗ):
  - intent_gap `Space`: thêm `lingeringDroppedIndex`; ca test mới trong `importPreviewFooterLayout.test.ts`.
  - bad_plan 43 px: `max-height` của cột cấu hình tính từ chiều cao thật của scroller và chân (dùng `ResizeObserver`); e2e đo đầu cột ở 30 %, 60 % và cuối trang.
  - Hai patch nhỏ đã sửa.
  - Kiểm lại: `test:story 6-23` xanh, ba spec e2e xanh (9 ca), các cổng tĩnh sạch.

## Design Notes

- Dải chỉ có trên tầng 2 (URL), vì `importPreviewSelectedBlocks` là `null` với tệp và văn bản. Dải hiện số khối cùng chữ đầu của 4 khối đầu (theo mockup), cắt một dòng. Con số 4 là đề xuất; build đưa Ice xem trên bài epochtimes.com thật (Notes của story).
- Chỉ số của `blockFocusedIndex` vẫn là chỉ số khối trong wire, để các lệnh `[` `]`, `Space` và `tier2BlockSetKept` không đổi. Khi con trỏ nằm trên dải thu, nó trỏ khối đầu của dải; khối này được đánh dấu "đang trên dải".
- `happy-dom` không đo được sticky hay hình học, nên phần đó chỉ kiểm bằng e2e.

## Verification

**Commands:**
- `cargo test --test segment_contract` và `cargo test --test webimport_contract` (chạy `npm run build` trước) -- expected: xanh.
- `npm run test:story 6-23` -- expected: các tệp vitest trong Code Map xanh; thoát 0, không có dòng `Errors`.
- `npm run test:e2e` cho riêng spec mới và hai spec đã sửa, chạy ngoài sandbox, sau khi dịch `--features wdio` -- expected: xanh. Đối chứng: gỡ cờ thật ở Rust thì ca ô mẫu đỏ.

**Manual checks:**
- Ice nhập lại bài epochtimes.com trong app thật: nút Nhập hiện mà không phải cuộn, và chốt con số "vài chữ đầu" của dải.
