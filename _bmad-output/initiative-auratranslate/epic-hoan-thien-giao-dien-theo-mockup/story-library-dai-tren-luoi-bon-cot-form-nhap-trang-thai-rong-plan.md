---
title: 'Library: dải trên, lưới bốn cột, form nhập thành lớp phủ, trạng thái rỗng'
type: 'feature'
ticket: '2'
created: '2026-10-13'
status: 'ready-for-dev'
baseline_revision: 'fe08b04e2cb70d9d3805a4a6e013131b1e8e5249'
route: 'full'
route_source: 'auto'
risk: 'high'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Library xếp tìm kiếm, lọc, lưới 110 px, form nhập luôn hiện và bốn khối quản trị thành một cột dài; mockup đặt một dải trên, lưới bốn cột có thẻ giàu thông tin, form nhập ngoài trang, và một trạng thái rỗng có lời mời (a:16–34, 167, 186).

**Approach:** Dải trên (ô tìm ⌕, `Chính xác dấu | Không dấu`, bộ lọc, nút `Nhập tài liệu`) rồi lưới 4 → 3 → 2 cột. Form nhập chuyển nguyên vẹn sang một lớp phủ mở bằng nút và `⌘N`. Bốn khối cũ nằm dưới lưới, rộng cả khung.

## Decisions (Ice, 2026-10-13)

1. Ngưỡng cột đo theo bề ngang khung `.mode` bằng container query: khung ≥ 1000 px bốn cột, 800–999 ba cột, < 800 hai cột.
2. "sửa … trước" đọc `updated_at` của hàng chỉ mục như hiện có, kể cả khi mốc cũ sau một buổi sửa câu; không đổi Rust. Khoảng hở ghi `meta.json` thành nợ `Chủ: Winston`.
3. Giữ plan một mục tiêu dù 4703 token.

## Boundaries & Constraints

**Always:**
- Mọi `@click` là đúng một `dispatch('<id>')`; `library.import.open` đổi từ cuộn+focus sang mở lớp phủ; đóng lớp phủ trả tiêu điểm theo UX-DR17 (nút mở còn trong DOM thì về nút đó, không thì `[data-library-import-open]`).
- Trạng thái rỗng lần đầu chỉ khi `libraryWorksHaveLoaded && libraryWorksTotal === 0 && libraryWorksError === null` và không có bộ lọc.
- Thẻ không bao giờ in `0`, `undefined`, `Invalid Date`: thiếu lĩnh vực hoặc `updated_at` không đọc được thì bỏ vế đó; thiếu cả hai thì không có dòng.
- Chữ chỉ qua `vi.json`; màu, cỡ, lớp chồng chỉ từ token.

**Never:**
- Không đổi Rust, schema, lệnh `library.import_*`, nội dung form, `ImportPreviewOverlay`/`BilingualImportPreviewOverlay` và đường tới chúng (story 6 sửa màn xem trước).
- Không dời bốn khối cũ đi nơi khác (story 3 và 4).
- Không giữ nút "Tìm" hay nhãn của ô tìm kiếm.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Có Tác phẩm | `total > 0` | Lưới, không có form trong trang | — |
| Rỗng thật | đã tải, `total = 0`, không lọc | Trạng thái rỗng thay lưới, hai lời mời + `⌘N` mở lớp phủ | — |
| Lọc ra 0 | `total = 0` do bộ lọc | Lưới rỗng có câu "không khớp bộ lọc", KHÔNG hiện lời mời lần đầu | — |
| Chưa tải / tải lỗi | `HaveLoaded = false` hoặc `libraryWorksError` | Không rỗng lần đầu; hiện lỗi sẵn có | Lỗi qua `tError` |
| Thẻ thiếu dữ liệu | `genre = ''` / `updated_at` hỏng | Chỉ "sửa …" / chỉ lĩnh vực / không dòng | Không chữ lạ |
| Nộp thành công | xác nhận ở màn xem trước | Lớp phủ nhập đóng, thẻ mới trong lưới | Lỗi nộp hiện trong lớp phủ, lớp phủ ở lại |

</frozen-after-approval>

## Code Map

- `src/modes/LibraryMode.vue` (2181 dòng) -- thứ tự DOM hiện tại: `.root-block` 345, mồ côi 436–498, `.search-block` 500–683, `.works-block` 685–1011 (bộ lọc 700–810, `.works-grid` 853, thẻ `.work-cell` 854–940), chương 943–1250, `.empty`+`.import-form` 1259–1500. CSS: `.search-block` 2091 (`max-width: 420px`), `.works-grid` 1874 (auto-fill 110 px), `.import-form` 1594, `.empty` 1566. Con trỏ lưới theo chỉ số, không tính cột, nên đổi số cột an toàn. Form chuyển ra tệp mới; tệp này chỉ co lại.
- `src/modes/libraryImport.ts` -- toàn bộ state ô form là `ref` cấp module (không ở `.vue`), nên chuyển template không đụng state; `finishImportSubmission` (gọi ở `main.ts` 556/571) là chỗ đóng lớp phủ khi thành công.
- `src/modes/libraryImportFocus.ts` + `src/main.ts:543` -- `openLibraryImportForm` hiện `setMode('library')`, cuộn, focus ô đầu; đổi thành `setMode('library')` rồi mở lớp phủ.
- `src/commands/index.ts:225,1443` -- dep `openLibraryImportForm` và lệnh `library.import.open` (`Mod+N`); thêm `library.import.close` theo mẫu `export.open/close` 3385.
- `src/ExportOverlay.vue` + `src/exportState.ts` -- mẫu lớp phủ: `v-if` trên `ref` mở, `focusReturnTargetOnOpen('[data-export-open]')` (`commands/focus.ts:413`), `trapTab`, `resetExport`; gắn ở `App.vue:353`. Cùng mẫu cho `LibraryImportOverlay.vue` (mới) + `libraryImportOverlayState.ts` (mới, cần `reset*` vì `check:panel-refs`).
- `src/config/library.ts:179` `WorkRow` đã có `source_lang`, `genre`, `updated_at`. `lookupHistoryState.ts:250` có `relativeTimeKey` nhưng chỉ tới giờ ("ph", "g"), không có ngày; cần helper thuần mới. `vi.json`: `mode.library.lang_zh/en` 631, `search_*` 853–877, `status`/`empty_body` 627–628, `search_mode_exact/lenient` 868–869.
- `updated_at` của hàng chỉ mục: `core/library/meta.rs::rebuild_from_store` (294) tính MAX(work, chương, segment) nhưng chỉ chạy khi tạo Tác phẩm (`work_creation.rs:574`), đổi trạng thái Chương (`lifecycle.rs:96`) và quét lại. Sửa chữ thuần không ghi lại `meta.json`; doc-comment 285–293 đã ghi đây là dư địa chờ một AD mới.
- Test phụ thuộc form luôn có: `tests/frontend/libraryWorks.test.ts:825,840` (cũng khẳng định `.empty .big/.small`), `importPreviewDestination.test.ts` (`.import-form` ở 152–695), `titlebarToolsMenu.test.ts:144–172` (focus ô đầu), `e2e/specs/titlebar-tools.e2e.mjs:92`, `e2e/support/importForm.mjs` (`createWorkThroughForm`, dùng bởi `story-5-4-lifecycle`, `story-5-5-progress`), `story-5-9`/`5-10` (`[data-library-search-button]`).
- Cổng: `check:i18n` (khoá mới, bỏ `search_label`/`search_button` nếu không còn dùng), `check:commands` (`HANDLER_TABLE` ở `scripts/check-commands.mjs` cần dòng cho handler `@keydown` của lớp phủ mới, cạnh `ExportOverlay.vue::trapTab` 2446; sàn CLICK/DISPATCH), `check:tokens` (`/* aura-allow-z-index */` cho scrim), `check:panel-refs`, `check:layout`. `main.ts:1002` liệt kê lớp phủ đang mở: thêm lớp phủ mới.

## Tasks & Acceptance

**Execution:**
- [ ] `src/modes/libraryImportOverlayState.ts` (mới) -- `ref` mở/đóng, `openLibraryImportOverlay`, `closeLibraryImportOverlay`, `resetLibraryImportOverlay` -- mẫu `exportState.ts`
- [ ] `src/LibraryImportOverlay.vue` (mới) + `src/App.vue` -- chuyển nguyên `<form class="import-form">` và các dòng báo lỗi/ảnh hỏng/notice từ `LibraryMode.vue` sang đây (nội dung không đổi), focus ô đầu khi mở, `trapTab`, Escape đóng, lưu/trả tiêu điểm; gắn cạnh `ExportOverlay` -- UX-DR17, tách tệp khổng lồ
- [ ] `src/modes/libraryImport.ts` + `src/main.ts` -- `finishImportSubmission` thành công thì đóng lớp phủ nhập; thất bại thì để mở; `openLibraryImportForm` mở lớp phủ -- đóng vòng nộp
- [ ] `src/commands/index.ts`, `src/i18n/vi.json` -- `library.import.open` mở lớp phủ; thêm `library.import.close`; nhãn mới -- AD-34
- [ ] `src/modes/libraryWorkCard.ts` (mới) -- hàm thuần `workCardMeta(work, now)`: nhãn ngôn ngữ nguồn (mã lạ ⇒ in mã) và dòng "lĩnh vực · sửa …" bằng bậc theo hiệu số (<1 ph, <1 g, <24 g, <48 g "hôm qua", <30 ngày, còn lại ngày UTC cắt từ chuỗi ISO) -- không phụ thuộc múi giờ, CI chạy UTC
- [ ] `src/modes/LibraryMode.vue` -- dải trên (ô ⌕ không nhãn, bỏ nút Tìm, Enter giữ `library.search`; hai đoạn `Chính xác dấu | Không dấu` thay chữ, giữ `data-library-search-mode`; bộ lọc; nút `Nhập tài liệu` mang `data-library-import-open`), lưới 4/3/2 cột, thẻ có ngôn ngữ trên bìa và dòng meta, trạng thái rỗng với hai lời mời + `⌘N`, bốn khối cũ xuống dưới lưới rộng cả khung (bỏ `max-width: 420px`) -- giao diện
- [ ] `tests/frontend/` -- sửa `libraryWorks`, `importPreviewDestination` (mount lớp phủ), `titlebarToolsMenu`; thêm test `workCardMeta` đồng hồ cố định (thiếu lĩnh vực, ngày hỏng, tương lai, ranh giới bậc), vị từ rỗng (đã tải/0, lọc 0, chưa tải, lỗi), `Mod+N` mở lớp phủ, Escape đóng và trả tiêu điểm
- [ ] `e2e/support/importForm.mjs`, `e2e/specs/titlebar-tools.e2e.mjs`, `story-5-9`, `story-5-10` -- `createWorkThroughForm` bấm `Nhập tài liệu` trước rồi tìm `.import-form` trong lớp phủ (một chỗ sửa); thay nút Tìm bằng Enter; `⌘N` kiểm lớp phủ; thêm một spec theo Verify (1440×790 bốn cột, thu hẹp 3 rồi 2, thẻ, form vắng khỏi trang, mở/đóng/tiêu điểm, tạo qua lớp phủ ra thẻ, rỗng)

**Acceptance Criteria:**
- Given 1440×790 và ≥5 Tác phẩm, when nhìn lưới, then bốn cột rộng cả khung; thu hẹp cửa sổ thì ba rồi hai cột, thẻ giữ tỷ lệ.
- Given một thẻ, when đọc, then có ngôn ngữ nguồn ở bìa và "lĩnh vực · sửa … trước"; thiếu dữ liệu thì không có chữ rác hay `0`.
- Given ô tìm, when gõ rồi Enter, then ra đúng kết quả như nút cũ và trang không có nút "Tìm" hay nhãn; bấm `Không dấu` thì khớp khoan dung dấu, `Chính xác dấu` thì khớp chặt.
- Given Library có Tác phẩm, when nhìn trang, then không có form nhập; bấm `Nhập tài liệu` hoặc `⌘N` (kể cả từ Workspace) thì mở lớp phủ; đóng thì tiêu điểm về nút mở.
- Given lớp phủ mở, when nộp qua màn xem trước và xác nhận, then lớp phủ đóng và có thẻ mới.
- Given đã tải và 0 Tác phẩm, when nhìn Library, then thấy trạng thái rỗng, mỗi lời mời mở lớp phủ; lọc ra 0, chưa tải hoặc tải lỗi thì không thấy nó.
- Given Library, when cuộn xuống dưới lưới, then đủ Thư mục gốc, Mục mồ côi, Trạng thái Tác phẩm đang mở, Chương theo thứ tự cũ.

## Implementation Notes

## Plan Change Log

## Review Triage Log

## Design Notes

- `createWorkThroughForm` chỉ có hai spec gọi (`story-5-4`, `story-5-5`); 20/31 spec dùng `openWorkspaceWithWork` đi qua IPC (`create_work_from_text`), không chạm form. Rủi ro high vẫn giữ: các spec 5-x đọc `[data-library-*]` và nightly phải chạy đủ.
- Lớp phủ nhập đứng NGUYÊN khi màn xem trước mở (nút `[data-import-preview-open]` nằm trong form là đích trả tiêu điểm của màn xem trước); chỉ đóng sau khi `finishImportSubmission` thành công. State ô form là `ref` cấp module nên đóng/mở không mất bản nháp.
- Dòng "Đã tạo …" (`createdWork`) ở lại trang Library vì lớp phủ đã đóng; lỗi nộp ở trong lớp phủ.
- Hai lời mời trạng thái rỗng cùng mở lớp phủ (form đã có cả tệp lẫn dán); không chọn sẵn ô.
- Cột lưới: `grid-template-columns: repeat(N, 1fr)` đổi theo ngưỡng (Decision 1). Khung nội dung ở 1440 là 1440 − 2×16 (`--space-panel-inline`) = 1408 px; cửa sổ mặc định 1280, `minWidth` 800 (`tauri.conf.json`), nên khung hẹp nhất 768 px.

## Verification

**Commands:**
- `npm run test:story 12-2 -- --list` rồi `npm run test:story 12-2` -- expected: chọn và xanh đúng các test sửa/thêm.
- `npm run check:i18n && npm run check:commands && npm run check:tokens && npm run check:panel-refs && npm run check:layout` -- expected: xanh, không thêm miễn trừ ngoài `aura-allow-z-index` cho scrim.
- `npm run build`, rồi một spec e2e mới và `story-5-4`, `story-5-5`, `story-5-9`, `story-5-10`, `titlebar-tools` -- expected: xanh.

**Manual checks (if no CLI):**
- `⌘N` và trả tiêu điểm trong app thật trên macOS (nợ `Chủ: Epic 12` nếu không tự động hoá được).
