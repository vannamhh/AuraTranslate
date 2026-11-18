---
title: 'Menu Công cụ theo nhóm dữ liệu'
type: 'feature'
ticket: '6'
created: '2026-11-18'
status: 'built'
baseline_revision: '2fa5dcdfee48049f79b2dac47b6235f235bba055'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `Công cụ ▾` (`src/TitlebarToolsMenu.vue`) là 16 mục phẳng, chưa theo thứ tự nhóm dữ liệu của R5 (EXPERIENCE.md §Thứ tự trong `Công cụ ▾`); còn Ngưỡng quét Glossary và Thêm Chương… mà R6 cho rời menu, và thiếu Quét chính tả.

**Approach:** Xếp lại các mục có sẵn theo R5: Glossary (`glossary.manage.open` · `glossary.harvest.open`) · TM (`tm.manage.open`) · Quy ước (`ai.proofread.scope.open` mới · `ai.proofread.memory.open` · `convention.manage.open` · `convention.replace_chapter_open`) · Reviewer (`export.reviewer_import.open` · `export.alignment.open`) · Bố cục (`layout.toggle_panel_headers` + dòng lỗi lưu của nó, ba công tắc panel chỉ ở Workspace, giữ nguyên) · cuối (`palette.open` · `settings.open`); năm `<li class="ui-menu-sep" role="separator">` giữa sáu nhóm, không nhãn nhóm. Bỏ hai mục `glossary.settings.open` và `library.add_chapters`. `glossary.settings.open` khai `pointer: { kind: 'via', … via: 'settings.open' }`; `library.add_chapters` vẫn còn nút ở Library và trang Tác phẩm. Mỗi mục vẫn là `@click="dispatch('<id>')"` literal (Kiểm A, Kiểm M). Nút `Công cụ ▾` mang thêm móc `data-proofread-scope-open` như mọi lớp phủ khác mở từ menu, để Esc trả tiêu điểm về nút (UX-DR17).

</frozen-after-approval>

## Implementation Notes

- Oneshot: đổi chủ yếu là dời khối `<li>` có sẵn; phần viết mới (một mục, năm phân cách, một khai `pointer`, test) dưới 100 dòng.
- Thêm `data-proofread-scope-open` lên `.tools-trigger` làm `document.querySelector('[data-proofread-scope-open]')` trúng nút thanh tiêu đề trước nút ở GridPanel (thanh tiêu đề đứng trước trong DOM): spec `overlay-caret-hold.e2e.mjs` phải chọn riêng nút GridPanel.
- `e2e/specs/titlebar-tools.e2e.mjs` đã lệch menu từ 0c8f8d43 (thiếu `palette.open`, `library.add_chapters`); cập nhật theo thứ tự mới, ca Ngưỡng quét Glossary chuyển sang `Mod+Alt+T`.
- `tests/frontend/proofreadWiring.test.ts` khoá quyết định cũ của 3937fb24 (Quét chính tả rời menu); đảo thành menu có mục `dispatch` literal và móc trên nút, theo quyết định 2026-11-17.
- `check:commands` đã đỏ ở master 2fa5dcdf (`HANDLER_TABLE` còn `moveHistoryAim` sau khi 17-4 bọc nó trong `aimAndReveal`); sửa ở commit riêng 8de6ca9536cfcb13d89ab23a8942aaafd9c8e6a4 trước story.
- Đối chứng gỡ thật: bỏ khai `pointer` của `glossary.settings.open` ⇒ Kiểm M đỏ đúng id; bỏ một `.ui-menu-sep` ⇒ chỉ ca R5 của vitest đỏ.
- e2e trên WKWebView: `titlebar-tools` 9/9 xanh (thêm ca Quét chính tả mở bằng chuột, Esc về `Công cụ ▾`); `overlay-caret-hold` ca `ai.proofread.scope.open` xanh, năm ca sau đỏ ở bước reset vì Vite của phiên `tauri dev` đang chạy phục vụ module mang tem HMR `?t=` (bẫy đã biết), chưa chạy lại trên Vite mới.

## Review Triage Log

- false — lối lùi ở `ProofreadScopeOverlay.vue:29` trúng `Công cụ ▾`: `focusReturnTargetOnOpen` (`src/commands/focus.ts:416`) trả phần tử đang có tiêu điểm khi nó không phải nút đầu tiên khớp, nên mở từ nút GridPanel thì quay về nút đó; lối lùi chỉ chạy khi nút mở đã rời DOM (mục menu sau khi đóng), và khi đó `Công cụ ▾` đúng là đích của UX-DR17.
- low, bác — mục Quét chính tả hiện cả ở Library, nơi ba ô có thể đều tắt: mỗi ô in số câu (`proofread.scope.count`) nên không phải rỗng im lặng; spine chỉ giới hạn công tắc panel ở Workspace, thêm `v-if` là lệch spine (báo Ice).
- low, ghi nhận — `overlay-caret-hold` năm ca chưa chạy lại trên Vite mới: đỏ ở bước reset do tem HMR `?t=` của phiên `tauri dev` (đã đo bằng `curl` từ Vite), không do mã; cần chạy lại khi Vite khởi động lại.
- false — vòng mũi tên ở vitest: `items[11]` là phần tử cuối của 12 mục ở Library, tức `settings.open`; e2e khẳng định đúng id.
- false — cột phím: `rows.map((r) => r[1])` chính là cột phím; `glossary.harvest.open` có thêm phím thì ca đỏ.
- false — frontmatter: bước oneshot đặt `built` lúc chốt plan, sau review.

## Verification

**Commands:**
- `npx vitest run tests/frontend/titlebarToolsMenu.test.ts` -- expected: xanh; ca thứ tự R5 + năm phân cách đúng chỗ, không có `glossary.settings.open`/`library.add_chapters`, Quét chính tả gọi `openAiProofreadScope`.
- `npx vitest run tests/frontend/settingsFrame.test.ts tests/frontend/tmManage.test.ts tests/frontend/proofreadWiring.test.ts tests/frontend/exportScope.test.ts` -- expected: xanh (các tệp chạm menu).
- `npm run check:commands` -- expected: xanh, `POINTER_DEBT` rỗng.
- `npm run check:i18n && npm run check:lint` -- expected: xanh.
- e2e `titlebar-tools.e2e.mjs` và `overlay-caret-hold.e2e.mjs` (ngoài sandbox, binary `--features wdio`) -- expected: xanh.
