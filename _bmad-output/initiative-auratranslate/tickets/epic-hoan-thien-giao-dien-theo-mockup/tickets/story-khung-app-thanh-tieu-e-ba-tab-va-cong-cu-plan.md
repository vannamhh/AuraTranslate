---
title: 'Khung app: thanh tiêu đề ba tab và Công cụ ▾'
type: 'feature'
ticket: '1'
created: '2026-10-13'
status: done
baseline_revision: 'c35ef91fa180a09515832c32f66fafd1215ecaf1'
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

**Problem:** Thanh tiêu đề mang ba tab cùng mười nút lối vào (`App.vue` 235–374), trong khi không mockup nào đặt nút ở đó (D1). Quản lý TM không có phím. Library không có phím mở nhập tài liệu (a:19). Trạng thái quét của Proofreader chỉ nằm trong dải ở Editor.

**Approach:** Thanh tiêu đề còn ba tab bên trái. Bên phải là trạng thái Proofreader (khi có), công tắc Ⓑ-2/Ⓑ-1 (chỉ ở Workspace) và nút `Công cụ ▾` mở danh sách thả xuống trong webview, theo bảng EXPERIENCE.md §Thanh tiêu đề và lối vào. Xuất nằm tạm trong danh sách cho tới story 4.

## Decisions (Ice, 2026-10-13)

1. Quản lý TM có phím mặc định `Mod+Alt+B`.
2. `⌘N` (`Mod+N`) là một lệnh mới: chuyển sang Library, đặt tiêu điểm vào ô đầu của form nhập, và cuộn form vào tầm nhìn.
3. Mục Quét chính tả không ghi phím, vì lệnh của nó (`ai.proofread.scope.open`, mở bộ chọn phạm vi) không có phím. Cột Phím `⌘P` của IA là phím quét ngay, không phải phím của mục này.
4. Nút `Công cụ ▾` mang cả tám thuộc tính `data-*-open` của các mục; mục không mang. Nhánh dự phòng sẵn có của các lớp phủ tìm ra nút đó. `TmManageOverlay.vue` được thêm nhánh dự phòng còn thiếu.

## Boundaries & Constraints

**Always:**
- Mọi `@click` là đúng một `dispatch('<id>')`. Nút `Công cụ ▾` mở/đóng qua một lệnh mới có nhãn trong `vi.json`.
- Danh sách theo thứ tự bảng IA: Quản lý Glossary · Quản lý TM · Cài đặt · Quét chính tả · Chỗ đã bỏ qua · Nhập bản reviewer · Nối tay · Xuất. Phím hiện bằng `formatChord`. Mục không có phím thì không ghi phím.
- Mũi tên lên/xuống đi giữa các mục; `Escape` đóng danh sách và trả tiêu điểm về `Công cụ ▾`; chọn một mục thì đóng danh sách.
- Đóng mọi lớp phủ mở từ `Công cụ ▾` thì tiêu điểm về `Công cụ ▾` (UX-DR17), cả khi mở bằng chuột lẫn bằng bàn phím trong danh sách.
- Màu, chữ, lớp chồng chỉ từ token. Danh sách thả xuống mang `/* aura-allow-z-index: <lý do> */`.
- Bảng chờ Glossary (`Mod+Alt+Q`) và Review Mode (`Mod+Alt+3`) rời thanh tiêu đề, chỉ còn phím.

**Never:**
- Không dùng menu gốc hệ điều hành hay Tauri Menu (D10).
- Không đặt nút Nhập tài liệu trên thanh tiêu đề (quyết định 12.1, nút thuộc story 2).
- Không gắn phím cho bốn lệnh `library.import_*`: chúng nộp form, không mở gì.
- Không đổi Rust, schema hay lệnh Tauri.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mở mục bằng chuột | Bấm `Công cụ ▾`, bấm Cài đặt, `Escape` | Cài đặt mở; đóng thì tiêu điểm ở `Công cụ ▾` | — |
| Mở mục bằng bàn phím | Tiêu điểm ở `Công cụ ▾`, `Enter`, mũi tên tới Quản lý TM, `Enter`, `Escape` | Quản lý TM mở; đóng thì tiêu điểm ở `Công cụ ▾` | — |
| Đóng danh sách | Danh sách mở, `Escape` hoặc bấm ra ngoài | Danh sách đóng; `Escape` đặt tiêu điểm ở `Công cụ ▾` | — |
| Phím TM | Workspace, gõ `Mod+Alt+B` | Quản lý TM mở; `Mod+M` vẫn gộp segment | — |
| `⌘N` | Đang ở Workspace hoặc Chế độ đọc | Chuyển sang Library, tiêu điểm vào ô đầu của form nhập, form cuộn vào tầm nhìn | Không ném lỗi |
| Trạng thái quét | Quét xong Chương 47, 184 câu | Thanh tiêu đề ghi "Chương 47 · đã quét 184 câu" | Chưa quét hoặc đã đổi Chương: không hiện |
| Công tắc bố cục | Library hoặc Chế độ đọc | Không hiện | Review Mode: không bấm được |

</frozen-after-approval>

## Code Map

- `src/App.vue` -- 235–374 thanh tiêu đề: tab `nav.modes` 236–261 giữ nguyên; mười nút `.titlebar-act` 267–373 bỏ; `focusOnPointerDown` 191 (vì WKWebView không đặt tiêu điểm cho `<button>` khi bấm chuột, giải thích ở 166–190) dùng lại cho `Công cụ ▾`; CSS 541–581.
- `src/commands/index.ts` -- `installCommands`; mỗi `CommandSpec {id, labelKey, run, keys?}` (`registry.ts:45`). `tm.manage.open` 3571; `library.import_*` 1399–1435 (comment 1391–1397: cố ý không phím); `layout.preset_grid` `Mod+Alt+1` / `preset_columns` `Mod+Alt+2` 1278–1289. `Mod+N` còn trống. Phím `Mod+Alt+` còn trống: A, B, D, H, I, 4–9 (`Y` va fixture `tmConcordance.test.ts:316`).
- `src/commands/keys.ts:328` -- `formatChord(chord, platform)`; `createKeymap` 461 ném lỗi khi trùng phím.
- `src/commands/focus.ts:413` -- `focusReturnTargetOnOpen`: giữ phần tử đang có tiêu điểm. Khi đóng, lớp phủ dùng nút đã giữ nếu nó còn trong DOM, nếu không thì `querySelector('[data-…-open]')`. `TmManageOverlay.vue:79` thiếu nhánh dự phòng này.
- Các lớp phủ và selector của chúng: TmManage:73, GlossaryManage:108, Settings:74, Export:74, Alignment:59, ReviewerImport:25, ProofreadScope:15, ProofreadMemory:25.
- `src/layout/WorkspaceDock.vue:546` -- `currentPresetId` cục bộ, chỉ lộ qua `data-layout-preset` 1095. Công tắc cần đọc được nó. `applyPreset` (`dockController.ts:110`) không làm gì ngoài Workspace và bị từ chối trong Review Mode.
- `src/proofreadState.ts` -- `proofreadProgress.done` 82 (số câu đã quét của lượt cuối), `proofreadStateValue`, `resetProofread` 269 (gọi khi đổi Chương/Tác phẩm/nhập). Số thứ tự Chương lấy từ `editorChapterId` (`editorPanelState.ts:160`) và `libraryChapters[].ord`; nhãn `mode.library.chapter_untitled`.
- `src/modes/LibraryMode.vue:1273` -- `<form class="import-form">` luôn có trong DOM.
- `src/i18n/vi.json` -- khoá phẳng `command.<id>`; `check:i18n` cấm chữ Việt trong `.vue`.
- `scripts/check-commands.mjs` -- Check K `HANDLER_TABLE` (~2287–2380): mỗi handler `@keydown`/`@mousedown` mới cần một dòng. Sàn CLICK 162 / DISPATCH 220 / COMMAND 200.
- Test đọc thanh tiêu đề: `tests/frontend/exportScope.test.ts:223`, `tmManage.test.ts:744` (khẳng định TM không có phím) và 985, `proofreadWiring.test.ts:197` (regex trên `App.vue`); `e2e/specs/shortcuts-focus.e2e.mjs:61`, `shortcuts-capture-mouse.e2e.mjs:61`, `story-3-5-review.e2e.mjs:177` (`realClick('[data-settings-open]')`).
- Tests that move: `tests/frontend/exportScope.test.ts`, `proofreadWiring.test.ts`, `tmManage.test.ts`, `titlebarToolsMenu.test.ts`, `titlebarProofreadStatus.test.ts`, `workspaceDockTier.test.ts`; `e2e/specs/titlebar-tools.e2e.mjs`, `shortcuts-focus.e2e.mjs`, `shortcuts-capture-mouse.e2e.mjs`, `story-3-5-review.e2e.mjs`.

## Tasks & Acceptance

**Execution:**
- [ ] `src/commands/index.ts` -- thêm lệnh mở/đóng `Công cụ ▾`, lệnh mở nhập tài liệu `Mod+N`, phím mặc định cho `tm.manage.open`; nhãn vào `vi.json` -- mọi lối vào là một lệnh.
- [ ] `src/TitlebarToolsMenu.vue` (mới) và `src/App.vue` -- dựng thanh tiêu đề mới, danh sách thả xuống, trạng thái Proofreader, công tắc bố cục; bỏ mười nút -- D1, D10, D11.
- [ ] `src/layout/WorkspaceDock.vue` -- lộ preset hiện tại cho công tắc -- công tắc cần trạng thái bật.
- [ ] `src/TmManageOverlay.vue` -- thêm nhánh dự phòng `[data-tm-manage-open]` như các lớp phủ khác -- đường lui tiêu điểm.
- [ ] `scripts/check-commands.mjs` -- thêm handler mới vào `HANDLER_TABLE`.
- [ ] `tests/frontend/` -- sửa `exportScope`, `tmManage`, `proofreadWiring` theo thanh tiêu đề mới; thêm test cho thứ tự mục, phím hiện, lệnh `Mod+N`, phím TM không trùng, trạng thái Proofreader ẩn/hiện.
- [ ] `e2e/specs/` -- sửa ba spec mở Cài đặt qua `Công cụ ▾`; thêm một spec theo Verify của 12.1.

**Acceptance Criteria:**
- Given app vừa mở, when nhìn thanh tiêu đề, then chỉ có ba tab và `Công cụ ▾` (Library không có công tắc bố cục).
- Given Workspace, when nhìn thanh tiêu đề, then có công tắc Ⓑ-2/Ⓑ-1 đánh dấu đúng preset đang dùng, và bấm thì đổi bố cục.
- Given `Công cụ ▾` mở, when đọc danh sách, then có đúng tám mục theo thứ tự IA, mục có phím ghi phím theo nền tảng.
- Given lệnh nào đó có phím, when mở Cài đặt › Phím tắt, then phím TM và `Mod+N` hiện ở đó và đổi được.

## Design Notes

Đường lui tiêu điểm: chọn mục bằng bàn phím thì mục có tiêu điểm, và khi danh sách đóng nút đã giữ rời DOM. Lớp phủ khi đó rơi về `querySelector('[data-…-open]')`, và quyết định 4 làm selector đó trúng `Công cụ ▾`. Chọn mục bằng chuột thì WKWebView không đặt tiêu điểm cho mục, nên tiêu điểm vẫn ở `Công cụ ▾` (nhờ `focusOnPointerDown` lúc mở danh sách) và lớp phủ giữ chính nút đó. Ba spec e2e đang `realClick('[data-settings-open]')` sẽ trúng `Công cụ ▾`, nên phải bấm thêm mục Cài đặt; khẳng định tiêu điểm về `[data-settings-open]` vẫn đúng.

## Implementation Notes

- Hai lệnh mới: `titlebar.tools.toggle` (không phím) và `library.import.open` (`Mod+N`); `tm.manage.open` có `Mod+Alt+B`. Phím hiện trong danh sách đọc từ `shortcutRows`, nên đổi phím ở Cài đặt cũng đổi nhãn.
- `TitlebarToolsMenu.vue` đóng danh sách bằng `addEventListener('click')` trên gốc, không bằng `@click`, để mỗi `@click` vẫn là đúng một `dispatch`; lúc đó mục còn trong DOM nên lớp phủ thấy đúng `activeElement`.
- `focusout` chỉ đóng danh sách khi `relatedTarget` là một node ngoài danh sách: WKWebView báo `null` khi bấm mục, đóng ở đó sẽ nuốt cú bấm.
- Công tắc bố cục đọc `activeLayoutPreset` (`layout/layoutPresetState.ts`), `WorkspaceDock.vue` ghi vào đó mỗi lần đổi preset; bố cục khôi phục từ đĩa vẫn báo preset mặc định, như `currentPresetId` đã làm từ trước.
- Trạng thái Proofreader chỉ hiện khi lượt quét `done` và Chương đang mở nằm trong `libraryChapters`; `resetProofread` đã xoá trạng thái khi đổi Chương.
- Bảng chờ Glossary và Review Mode rời thanh tiêu đề; `GlossaryQueueOverlay` không còn nút mở nên nhánh dự phòng tiêu điểm của nó không tìm thấy gì.
- Chưa kiểm tay: danh sách bằng bàn phím trên macOS, `⌘N`, trạng thái sau một lượt quét thật (nợ `Chủ: Epic 12` trong `deferred-work.md`).
- 🔵 Đo trên happy-dom thì câu "mục còn trong DOM" ở dòng thứ hai không đúng: lúc lớp phủ TM lưu nút mở, danh sách đã vẽ lại, mục đã rời DOM và tiêu điểm ở `body`. `focusReturnTargetOnOpen` chọn `Công cụ ▾` vì `body` là tổ tiên của `[data-tm-manage-open]`. Nhánh dự phòng mới của `TmManageOverlay.vue` vì thế không nằm trên đường này. Ca `tmManage.test.ts` "an item chosen by keyboard…" đỏ khi gỡ `data-tm-manage-open` khỏi nút.
- `check:debt-owner` đỏ do chính plan này: số ticket có trạng thái plan lên 176, nên sàn `PLANNED_TICKET_FLOOR` 140 rơi dưới 80 %. Cổng đòi nâng lên 150; việc sửa ngưỡng cổng chờ Ice.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 9 phát hiện — high 0 · medium 2 · low 4 · false 2 · maybe-false 0; ngoài ra 1 phát hiện chờ Ice.

- medium · patch — `WorkspaceDock.vue` `restore()`: bố cục khôi phục bằng `fromJSON` vẫn để công tắc đánh dấu Ⓑ-2, trái AC2; comment ~538 thừa nhận bố cục khôi phục không phải preset nào. Sửa: đặt preset chung về `null` sau `fromJSON` thành công.
- medium · chờ Ice — `check:debt-owner` đỏ vì plan này đẩy số ticket có trạng thái plan lên 176; sửa là nâng `PLANNED_TICKET_FLOOR` 140 → 150, bị bộ phân loại quyền chặn vì sửa ngưỡng cổng.
  - 🔵 2026-10-19: đã vượt; `PLANNED_TICKET_FLOOR` nay 160 (`scripts/check-debt-owner.mjs:742`), ghi nhận ở Story 12-22.
- low · patch — `resetActiveLayoutPreset`, `resetToolsMenu` không ai gọi (grep `src`, `tests`). Xoá.
  - 🔵 2026-10-13: chuyển thành false. Xoá hai hàm làm `check:panel-refs` đỏ: cổng đòi mỗi ref cấp module có một hàm `reset*`. Implementer đã thêm hai mục `EXEMPT` để xanh lại; đã gỡ hai mục đó và trả hai hàm về.
- low · patch — `main.ts` 70–74: hai import mới chen giữa comment ⚠️ và import `lookupDrawerState` mà nó chú thích. Dời.
- low · patch — test "the trigger opens and closes…" chỉ đi nhánh `portMissing`; cú bật/tắt của nút không có ca nào chạm. Thay bằng ca bấm `.tools-trigger`.
- low · patch — trạng thái Proofreader chỉ test hàm thuần; đổi chỗ đối số trong `App.vue` vẫn xanh. Dời `computed` vào `titlebarProofreadStatus.ts` và test với state thật.
- low · patch — e2e ca cuối: `activeElement?.closest(...) !== null` đúng cả khi `activeElement` là null. Phần "không có bằng chứng cổng đã chạy" sai: `check:commands`/`tokens`/`i18n` chạy xanh ở bước 3.
- low · reject — `HANDLER_TABLE` gán `R_CATEGORY_NAV` cho `onKeydown` dù nó cũng đóng bằng Escape. Không gọi lệnh nào nên vế "no dispatch" vẫn đúng; tách handler hay thêm hằng lý do là thêm phức tạp cho chỗ không ai gặp.
- false — chuỗi phím `Mod+Alt+1/2` vẫn tới `applyPreset` trong Review Mode: `dockController.ts:61,105` từ chối lệnh bố cục khi Review Mode đang hiện.

## Verification

**Commands:**
- `npm run test:story 12-1 -- --list` rồi `npm run test:story 12-1` -- expected: chọn và xanh đúng các test đã sửa/thêm.
- `npm run check:commands && npm run check:tokens && npm run check:i18n` -- expected: xanh, không thêm miễn trừ ngoài `aura-allow-z-index` cho danh sách thả xuống.
- `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml && npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/<spec mới>.e2e.mjs` và ba spec đã sửa -- expected: xanh.

**Manual checks (if no CLI):**
- Mở danh sách bằng bàn phím trong app thật trên macOS (e2e không tự động hoá được kích hoạt nút bằng phím, nên ghi nợ `Chủ: Epic 12`).
