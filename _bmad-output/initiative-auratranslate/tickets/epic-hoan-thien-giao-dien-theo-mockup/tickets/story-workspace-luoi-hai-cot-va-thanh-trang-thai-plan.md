---
title: 'Workspace: lưới hai cột và thanh trạng thái'
type: 'feature'
ticket: '7'
created: '2026-10-14'
status: 'built'
baseline_revision: '68f609d192b08bb737cec8bc3c1fa191ef12e2b0'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Lưới Workspace còn cột ⑤ in chữ trạng thái trên mỗi hàng, vạch lề chỉ khác nhau bằng màu, mỗi ô nguyên văn Hán Việt còn dòng "Nguồn: …"; thanh trạng thái chỉ nói một câu báo, chưa có vị trí câu, bộ đếm, gợi ý phím, số Glossary/TM, dòng tra cứu ở màn hẹp như mockup (D3, D4, D9; dòng a:157, a:182, c:113; B1, B2, B29, B30, A20).

**Approach:** Chữ trạng thái ở lại trong hàng nhưng ẩn khỏi mắt và thành tên truy cập của ô bản dịch; vạch lề mang trạng thái bằng kiểu nét; lưới còn bốn cột; bỏ dòng nguồn Hán Việt; thanh trạng thái tách phần câu báo (vùng sống) khỏi phần thông tin thường trực.

## Decisions (Ice, 2026-10-14)

1. Một story, không tách lưới và thanh trạng thái; nhận kế hoạch quá 1.600 token.
2. Hình vạch: `confirmed` liền · `draft` đứt · `tm-rule` chấm · `primary` liền 3px; các vạch khác 2px.
3. Nhãn lỗi hàng (AC14) hiện trong ô bản dịch, dưới chữ; lưới bốn cột như mockup.
4. Bộ đếm "12/34 đã xác nhận" và gợi ý "⌘M gộp · ⌘/ tách" nằm trên thanh trạng thái (theo mô tả entry, không theo đầu panel của B1, B2).
5. Số Glossary và TM lấy từ hai lệnh Rust mới chỉ đếm; làm tươi sau mỗi lần xác nhận câu và sau mỗi thay đổi Glossary/TM.
6. Câu nhắc auto-save nằm phía phải thanh trạng thái, chỉ ở tầng rộng.

## Boundaries & Constraints

**Always:**
- WCAG 1.4.1: mỗi giá trị hiện được của `SEGMENT_RULE_VALUES` khác nhau ở kiểu nét hoặc độ dày; `none` không vẽ vạch.
- Vạch vẫn `aria-hidden`; trình đọc màn hình nghe trạng thái và lỗi qua tên truy cập của ô bản dịch.
- Hình nét bằng `border-style`/độ dày qua token, một khối `.rule-<x>` cho mỗi giá trị (Kiểm I của `check:commands`); không gradient (Kiểm F của `check:tokens`), không màu viết thẳng.
- Chữ mới qua `vi.json` (`check:i18n`); phím hiển thị qua `formatChord` theo phím đang gán, không viết tay `⌘`.
- Thông tin thường trực của thanh trạng thái nằm ngoài vùng `role="status"`: đổi câu đang sửa không làm trình đọc màn hình đọc lại.
- Giá trị chưa biết (Chương chưa có `ord`, số chưa đếm xong, lệnh đếm lỗi) không hiện thành `0`: kiểu `Option`/`null` và ẩn mục đó.
- Lệnh đếm đếm đúng tập mà `tm_list_pairs`/`glossary_list_entries` liệt kê cho cùng Tác phẩm.

**Never:**
- Không gỡ tab Hán Việt, `SourceHanViet` hay `sourcePanelState.ts` (D4 chỉ bỏ dòng "Nguồn: …"; D9).
- Không sửa `ReadingMode.vue`, nội dung dải Proofreader (12.10), Attribution (12.21).
- Không thêm tên lớp `.rule-*` mới; không thêm dependency.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Ba trạng thái | Hàng chưa dịch, đã dịch, đã xác nhận | Không chữ trạng thái nhìn thấy; tên truy cập ô bản dịch chứa chữ trạng thái; vạch: không · đứt · liền | — |
| Lỗi hàng | Xác nhận bị từ chối | Nhãn lỗi nhìn thấy dưới chữ trong ô bản dịch, có trong tên truy cập | `tError` như hiện có |
| Câu đang sửa | Con trỏ ở câu 6 trên 184 câu sống | "Chương 47 · câu 6/184 · 12/34 đã xác nhận" | 🔵 2026-10-14 (Ice duyệt): Chương không có hàng trong `libraryChapters` ⇒ chỉ hiện "Câu n/total", không "Chương 0"; bản cũ ghi "Chương chưa có `ord` ⇒ nhãn `chapter_untitled`", tự mâu thuẫn vì khoá đó là "Chương {ord}" và mọi Chương đều có `ord` |
| Xác nhận câu | ⌘Enter thành công | Bộ đếm xác nhận và số TM làm tươi | Lệnh đếm lỗi ⇒ ẩn số đó, không `0` |
| Màn rất hẹp | `lookupHasRetreated` | Dòng nghĩa đầu của nguồn đầu + phím mở + nút ngăn kéo; không Glossary/TM, không câu auto-save | Tra cứu trượt ⇒ chỉ nút |
| Hán Việt | Tab Hán Việt mở | Không còn dòng "Nguồn: …" | — |

</frozen-after-approval>

## Code Map

- `src/panels/GridPanel.vue` (2633 dòng, đọc theo offset) -- lưới `subgrid` năm cột `3px 30px 1fr 1fr 96px` :2120; vạch `.col-rule` :1782, `.rule` :1789, CSS `.rule*` :2192–2240; cột ⑤ `.col-state` :1977–1994 (chữ `STATE_LABEL_KEYS` :332, lỗi `rowErrorLabelById`); ô bản dịch `.cell-tgt` :1956–1973 chưa có `role`/`aria-*`; chú thích :326–331 giải thích vì sao vạch `aria-hidden`.
- `src/panels/editorSegments.ts` -- `SEGMENT_RULE_VALUES` :81, `resolveSegmentRule` :162.
- `src/panels/SourceHanViet.vue` -- dòng `.hv-sources` :842–846, `sourcesLine` :473, CSS :1002; khoá `panel.source.han_viet_sources_prefix` (`vi.json:1090`); `StatusBar.vue:337` nhắc khoá này trong chú thích.
- `src/StatusBar.vue` -- `<footer role="status">` :383, chuỗi `v-if` câu báo :397–459; nút `.lookup-entry` :473 (`lookupHasRetreated`). Mốc "Đã lưu N giây trước" là nhánh cuối của chuỗi câu báo và giữ nguyên chỗ, ở mọi tầng. Câu nhắc auto-save (Quyết định 6, a:157) là chữ khác: `mockups/data-integrity.html:246` "Auto-save khi ngừng gõ ~2 giây · trần 5 giây buộc ghi dù đang gõ liên tục", con số lấy theo hằng số flush thật.
- Dữ liệu webview: `editorSegments` :158, `editorChapterId` :160, `editorCaretSegmentId` :171 (`src/panels/editorPanelState.ts`); `libraryChapters[].ord`; `groupedLookup` :272, `computeSpine` :83 (`src/panels/lookupPanelState.ts`); `formatChord` (`src/commands/keys.ts:328`, mẫu ở `TitlebarToolsMenu.vue:106`); tầng bố cục `LayoutTier` (`src/layout/workspaceLayout.ts:270`); số phát hiện theo loại và mốc quét trong `src/proofreadState.ts`.
- Rust: `tm_list_pairs` (`src-tauri/src/commands/tm.rs:284`, wrapper :813, phép đếm `health` :87–98, :189); `glossary_list_entries` (`src-tauri/src/commands/glossary.rs:638`). Lệnh mới đăng ký theo đúng các chỗ `tm_list_pairs` đăng ký (`lib.rs`, capabilities, danh sách lệnh của `scripts/check-*`); wrapper TS cạnh `src/config/tm.ts`, `src/config/glossary.ts`.
- Lớp ẩn khỏi mắt: chưa có lớp dùng chung; ba bản scoped (`App.vue:414`, `AlignmentOverlay.vue:399`, `GlossaryQueueOverlay.vue:521`) cần `aura-allow-spacing` cho `margin:-1px`.
- Gates: `scripts/check-commands.mjs` Kiểm I :2047, `HANDLER_TABLE` :2365 (lưới :2469); `scripts/check-tokens.mjs` Kiểm B/F; `scripts/check-i18n.mjs` Kiểm A2.
- Tests that move: `tests/frontend/{gridPanelRowErrorPriority,gridPanelTmFilled,editorTmFilled,gridPanelImages,statusBar,editorNavNotice,editorRegroupNotice,editorChapterSwitch,glossaryMarksRefresh,workspaceDockTier,lookupSensesFailed,editorSourceCut}.test.ts`; `e2e/specs/{grid-image-row-alignment,grid-row-error-label,segment-navigation,editor-typing-flush,hanviet-parallel-row-height}.e2e.mjs`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/commands/{tm,glossary}.rs`, `lib.rs` -- hai lệnh đếm trả `u64`, cùng tập với lệnh liệt kê; test Rust so số đếm với độ dài danh sách -- Quyết định 5.
- [ ] `src/panels/GridPanel.vue` -- bỏ cột ⑤ khỏi lưới (bốn cột); chữ trạng thái ẩn khỏi mắt trong hàng; ô bản dịch có `role="textbox"`, `aria-multiline`, tên truy cập gồm số câu, trạng thái, lỗi; nhãn lỗi dưới chữ trong ô; khối `.rule-*` theo Quyết định 2 -- D3, Quyết định 2–3.
- [ ] `src/panels/SourceHanViet.vue`, `src/i18n/vi.json`, `src/StatusBar.vue:337` -- bỏ dòng nguồn, `sourcesLine`, CSS, khoá -- D4.
- [ ] `src/StatusBar.vue` -- `role="status"` chỉ bọc câu báo; trái: vị trí câu, bộ đếm xác nhận, gợi ý phím gộp/tách, "N chính tả · M nghi nghĩa" khi Chương có lượt quét; phải: "Tra cứu ngoại tuyến · Glossary N mục · TM N cặp" và câu auto-save ở tầng rộng, dòng tra cứu khi `lookupHasRetreated` -- B1, B2, B29, B30, A20, a:157, c:113.
- [ ] `DESIGN.md` §Vạch lề segment -- một dòng 🔵 ghi hình đã chọn (12.13 dùng lại).
- [ ] Tests -- sửa các tệp ở Code Map; thêm test theo Verify của 12.7 và theo bảng I/O; đối chứng đỏ: gỡ thật `border-style` khỏi một khối `.rule-*`, và gỡ lời gọi làm tươi sau xác nhận.

**Acceptance Criteria:**
- Given lưới có hàng chưa dịch, đã dịch, đã xác nhận, when đọc DOM nhìn thấy, then không hàng nào hiện chữ trạng thái, tên truy cập ô bản dịch chứa chữ trạng thái, và `getComputedStyle` của vạch khác nhau ở `border-style` hoặc bề rộng.
- Given con trỏ đổi câu, when thanh trạng thái cập nhật vị trí, then nội dung vùng `role="status"` không đổi.
- Given đổi phím gộp trong Cài đặt › Phím tắt, when nhìn thanh trạng thái, then gợi ý hiện phím mới.

## Implementation Notes

- Lệnh đếm TM và Glossary là `COUNT(*)` từng tầng cộng lại: `resolve_merge` chỉ nối và sắp xếp, không gộp cặp trùng, nên tổng bằng đúng `total_pairs` (đo 12.409 cặp, bản debug: 0,15 ms so với 123,9 ms của đường liệt kê). Nếu sau này lệnh liệt kê gộp bản trùng thì test so số đếm với danh sách sẽ đỏ.
- Nhãn lỗi hàng vẽ bằng `::after` + `attr(data-row-error)` chứ không phải phần tử con: đường ghi đọc `cell.textContent` của ô `contenteditable`. Kiểm I của `check:commands` nay nhận màu vạch từ `background-color`, `border-color` hoặc `border-left`, vì nét đứt/chấm cần `border`.
- Vạch khác nhau ngoài màu: vitest/happy-dom không áp `<style scoped>` của `.vue` vào `getComputedStyle` (đo: 0 thẻ `<style>`, `borderLeftStyle` rỗng). Giữ bằng hai chỗ: Kiểm I của `check:commands` nay đòi mỗi khối `.rule-*` khai độ dày và kiểu nét trong `border-left` và từng cặp khác nhau; e2e `story-12-7-workspace-grid-status-bar` đọc computed style thật của ba vạch trong WKWebView.
- Thanh trạng thái chỉ có hàng trong `libraryChapters` mới ra "Chương N · câu n/total"; Chương chưa có hàng (Tác phẩm mở qua IPC, danh sách chưa tải) ra "Câu n/total", không bao giờ "Chương 0".
- `resetEditorPanel()` gọi `resetWorkspaceCounts()` (số đếm về chưa biết khi đổi Tác phẩm); `check:panel-refs` đòi hàm reset cho `workspaceCountsState.ts` và một miễn trừ có tên cho `lookupDrawerState.ts::currentTier`.
- e2e dùng lại vite đang nghe cổng 1420 nếu có; một vite cũ đã HMR làm `import()` của `panelReset` trả bản sao module và đỏ cả spec không liên quan. Tắt vite cũ trước khi chạy e2e.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens quick): high 0 · medium 1 · low 2 · false 2 · maybe-false 0 · defer 1.

- medium · patch — `StatusBar.vue::confirmedCounterText` đếm cả câu `is_omitted` nên không lên N/N; lệch `in_translation_count` (`chapter.rs:365`). Loại câu bỏ qua khỏi cả tử và mẫu.
- false — `positionText` có tổng gồm câu bỏ qua: B30 định nghĩa tổng là mọi câu còn sống, trùng `segment_count`.
- false — `positionInChapter` vượt tổng: chỉ số và tổng cùng đếm trên `editorSegments`, nên n ≤ tổng.
- low · patch — vế TM của "đếm không làm tươi" sai (chỉ `confirm.rs:283` và `tmx.rs` ghi `tm_unit`, cả hai đã làm tươi); vế Glossary đúng: Nhận ứng viên ở Bảng chờ (`glossaryQueueState.ts:287`) tạo mục mà không làm tươi.
- low · defer — câu bỏ qua được đọc là "chưa dịch": `resolveSegmentRule` không có nhánh bỏ qua, có từ trước 12-7 (ghi `deferred-work.md`, Chủ: Epic 12).
- low · patch — chú thích `SourceHanViet.vue:482` và `editorSegments.ts:366` còn tả `.hv-sources` như phần tử đang render.

## Design Notes

Vạch là kênh thị giác duy nhất của trạng thái, nên đối chứng đỏ phải gỡ thật thuộc tính ngoài màu khỏi CSS. Lưới không có phần tử hàng (mỗi cột lặp `v-for` riêng), và ô bản dịch là chỗ tiêu điểm đứng khi sửa, nên tên truy cập đặt trên ô đó. Thanh 34px chỉ đủ một dòng: mục thường trực cắt bằng dấu ba chấm trước khi đẩy câu báo ra ngoài.

## Verification

**Commands:**
- `npm run build && cargo test --test <tệp test lệnh đếm>` -- expected: xanh.
- `npm run test:story 12-7` -- expected: test lưới, thanh trạng thái, Hán Việt xanh.
- Chạy cả bộ một lần trước push (đổi `lib.rs` là wiring dùng chung).

**Manual checks (if no CLI):**
- Bản đóng gói macOS, sáng và tối: ba vạch phân biệt được không cần màu (nợ `Chủ: Epic 12`).
