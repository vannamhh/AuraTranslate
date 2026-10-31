---
title: 'Bật tắt panel bằng chuột'
type: 'feature'
ticket: '8'
created: '2026-10-31'
status: 'built'
baseline_revision: 'a536818e0aa1a2dfb4638bff7db7baef1c11ca49'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-san-chuot/ux-san-chuot/EXPERIENCE.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-san-chuot/ux-san-chuot/DESIGN.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Chưa đường chuột nào ẩn được một panel: ba `layout.toggle_*` không gán phím mặc định và không có nút, nên người dùng không biết chúng tồn tại; `layout.preset_*` không cùng kết quả nên `via` không đạt.

**Approach:** Thêm ba công tắc `menuitemcheckbox` ở cuối `Công cụ ▾`, mỗi công tắc một `dispatch` literal, đúng theo mục Sắp xếp panel. Handler `togglePanel` giữ nguyên; phần việc thật là làm cho công tắc nói đúng trạng thái dock và không im lặng ở các ca `hidePanel` từ chối.

## Boundaries & Constraints

**Decision (Ice 2026-10-31):** Ở tầng `narrow`/`unsupported` mục `Tra cứu` vẫn bật được, và nút mở ngăn kéo ở `StatusBar.vue:743` ẩn khi `panel.lookup` đang ở dock. Đo bằng test tạm hai `PanelFrame` cùng owner: `declareFocus` ném ở `onMounted`, app sống, không mất dữ liệu, nhưng `enterFocus('panel.lookup')` trỏ vào bản dưới ngăn kéo. Đường này đóng cả phím tự gán và ca tắt Đối chiếu rồi thu cửa sổ, vì `SACRIFICE_ORDER` dừng ở `panel.lookup`.

**Decision (Ice 2026-10-31):** Sửa ngay trong story này việc tắt rồi bật Đối chiếu sai bố cục: `findTreeSpot` trả một lá làm neo nên panel đứng cạnh một nhánh chỉ cắt đôi lá đó. `showPanel` và `undoMerge` dùng dạng thêm vào mép dock khi neo là nhánh. Lệnh Never về `showPanel` được Ice nới đúng cho thay đổi này.

**Decision (Ice 2026-10-31):** Nới Never cho đúng một dòng `autoHiddenIds.delete(panelId)` trong `togglePanel`: bật hoặc tắt bằng tay là lựa chọn thủ công, nên tầng không được trả panel đó về khi cửa sổ nới ra.

**Always:** Ba `<button>` viết tay, không `v-for`, vì Kiểm A và Kiểm M chỉ đọc `dispatch('<id>')` literal. Nhãn lấy `panel.grid.title` · `panel.lookup.title` · `panel.ai_translation.title`, không thêm khoá. Ba mục nằm sau mục Sắp xếp panel và dòng lỗi của nó, thứ tự Đối chiếu, Tra cứu, Đề xuất AI; `v-if` chỉ ở Workspace; `:disabled` khi Review Mode mở. Mục của panel duy nhất đang hiện cũng tắt (R9), vì `hidePanel` từ chối ca này chỉ bằng `console.error`. Trạng thái công tắc đọc một tập phản ứng ghi từ dock Workspace thật, không qua `panelRing()`, vì `panelRing()` trả `REVIEW_RING` khi Review Mode mở. `ArrowUp`/`ArrowDown` của menu bỏ qua mục tắt.

**Never:** Đổi `togglePanel`, `hidePanel` hay luật tầng trong `WorkspaceDock.vue`; ở tệp đó chỉ thêm lượt nghe ghi tập phản ứng và đường đặt panel theo Decision trên. Gán phím mặc định: `keys: undefined` giữ nguyên. Đổi `POINTER_DEBT_BASELINE`. Lưu chỗ của panel ẩn qua các lần mở app: sentinel `{ reference: id }` ở `restore()` giữ nguyên.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Tắt rồi bật | Ba panel hiện, bấm một công tắc hai lần | Panel ẩn, panel còn lại lấp chỗ, bật lại về đúng chỗ cũ và cỡ cũ; `aria-checked` đúng ở mỗi lần mở menu | Không lỗi |
| Panel cuối | Chỉ còn một panel hiện | Mục của panel đó tắt, hai mục kia bật được | Không gọi `togglePanel` |
| Review Mode | Review Mode mở | Ba mục hiện, tắt, `aria-checked` vẫn nói trạng thái Workspace | `setDockSuspended` giữ nguyên |
| Ngoài Workspace | Thư viện hoặc Đọc | Ba mục không hiện | Không lỗi |
| Chữ chưa flush | Gõ vào Đối chiếu, tắt rồi bật trong 2 giây | Chữ còn nguyên qua `restoreEditedText` | Đỏ e2e là lỗi story này |
| Tầng hẹp, `Tra cứu` ở dock | Cửa sổ `narrow`, bật `Tra cứu` | Panel vào dock, nút mở ngăn kéo ẩn; tắt lại thì nút hiện lại | Không còn `declareFocus` ném |
| Tầng hẹp, `Tra cứu` tắt | Cửa sổ `narrow`, panel không ở dock | Nút mở ngăn kéo hiện như cũ | Không lỗi |
| Phím mũi tên | Menu mở bằng phím, có mục tắt | Tiêu điểm nhảy qua mục tắt, không kẹt | Không lỗi |
| Tắt hai panel, bật không theo thứ tự ngược | Tắt A, tắt B, bật A | Đi nhánh dự phòng sẵn có của `showPanel`, panel vẫn nhìn thấy; nút preset đưa về bố cục đủ | Đo và ghi nợ, không sửa |
| Mở lại app với panel đang ẩn | Khởi động lại rồi bật | Panel về bên phải panel đầu tiên, theo sentinel sẵn có | Ghi nợ, không sửa |

</frozen-after-approval>

## Code Map

- `src/TitlebarToolsMenu.vue` -- mục `layout.toggle_panel_headers` ở dòng 222–237 là khuôn; `menuItems()` dòng 24 phải lọc mục tắt; `onRootClick` đóng menu sau mỗi cú bấm mục.
- `src/layout/layoutPresetState.ts` -- khuôn tệp trạng thái module `shallowRef` + `readonly`. `dockController.ts` nạp bằng Node thuần nên không giữ được `ref`.
- `src/layout/WorkspaceDock.vue` -- `togglePanel` dòng 459, `hidePanel`, `showPanel`, `rememberSpot` chỉ đọc. Lượt nghe đăng ký cạnh `disposables.push(...)` dòng 1047. ⚠️ Dùng `api.onDidAddPanel`/`onDidRemovePanel`, không `onDidLayoutChange`, vì sự kiện đó bắn liên tục khi kéo. Kiểm `dockview-core` 7.0.4 có hai sự kiện này trước khi dùng.
- `src/StatusBar.vue:743` -- `v-if="lookupHasRetreated"` thêm vế `panel.lookup` không ở dock.
- `src/modes/modeState.ts` `currentMode` · `src/reviewModeState.ts` `reviewModeIsOpen` · `src/layout/lookupDrawerState.ts` `lookupHasRetreated`.
- `src/commands/index.ts:1432-1453` -- khối comment sai: nói "bốn command", và dòng 1442 khẳng định ẩn/hiện panel chỉ tới được bằng chuột trong khi không đường chuột nào tồn tại.
- `scripts/check-commands.mjs:2908` `POINTER_DEBT` · `:2971` `POINTER_FLOOR` tính `ceil(0.85 × live)`.
- `tests/frontend/titlebarToolsMenu.test.ts` -- `setup()` tiêm `CommandDeps`; ca Sắp xếp panel dòng 239 là khuôn; `get('[role="menuitemcheckbox"]')` lấy mục đầu nên ca cũ vẫn trỏ Sắp xếp panel. `currentMode` mặc định `library`, ca mới phải đặt `workspace`.
- `e2e/specs/review-mouse.e2e.mjs` -- khuôn spec chuột; `_bmad-output/implementation-artifacts/14-7-ban-do/chuot-that.e2e.mjs` -- khuôn bàn đo.

## Tasks & Acceptance

**Execution:**
- [x] `src/layout/panelVisibilityState.ts` -- tệp mới: tập panel đang hiện, đọc `readonly`, một hàm ghi, một hàm reset -- nguồn chung cho công tắc và nút ngăn kéo.
- [x] `src/layout/WorkspaceDock.vue` -- ghi tập sau mỗi lượt thêm/gỡ panel và lúc `onReady` -- `restore()` dựng panel qua `fromJSON`, nên giá trị đầu phải đúng cả khi bố cục đã lưu có panel ẩn.
- [x] `src/TitlebarToolsMenu.vue` -- thêm ba `menuitemcheckbox`; lọc `:disabled` ở `menuItems()`; `:disabled` theo Review Mode và panel cuối -- nối công tắc vào `dispatch`.
- [x] `src/StatusBar.vue` -- thêm vế vào `v-if` của nút ngăn kéo -- quyết định của Ice.
- [x] `src/commands/index.ts` -- thay khối comment dòng 1432–1453 bằng tối đa hai dòng tiếng Anh về `keys: undefined` -- comment đang sai và chứa story id.
- [x] `scripts/check-commands.mjs` -- xoá ba id khỏi `POINTER_DEBT`; `POINTER_FLOOR` thành 246 -- Kiểm M đọc 289/33/8.
- [x] `tests/frontend/titlebarToolsMenu.test.ts` -- ca (i) cho từng id; ca Panel cuối, Review Mode, Ngoài Workspace, Phím mũi tên -- R7.
- [x] `tests/frontend/statusBarWorkspace.test.ts`, `tests/frontend/workspaceDockTier.test.ts` -- nút ngăn kéo ẩn/hiện theo tập; tập đúng sau `restore()` có panel ẩn, sau `layout.preset_*` và sau lượt tầng -- hai bề mặt đọc cùng một tập.
- [x] `e2e/specs/panel-toggle-mouse.e2e.mjs` -- trên WKWebView: tắt rồi bật từng panel, đọc lại `visiblePanelsInLayoutOrder` và kích thước; ca Chữ chưa flush -- verify của vé.
- [x] `_bmad-output/implementation-artifacts/14-8-ban-do/` -- bàn đo chuột thật theo khuôn 14-7; đo thêm vị trí cuộn của lưới sau bật lại -- bắt lỗi `.click()` không tái hiện.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- đóng mục Story 14.8 ở dòng 13675; thêm mục `Chủ: Epic 14` cho hai hàng ghi nợ của Matrix -- sổ nợ theo đúng thực tế.

**Acceptance Criteria:**
- Given cây thật, when chạy `npm run check:commands`, then Kiểm M đọc 289 điểm chuột, 33 khai `pointer`, 8 nợ và xanh.
- Given gỡ `@click` khỏi một công tắc, when chạy ca (i) của id đó, then ca đỏ ở lần gọi `togglePanel`.
- Given tầng `narrow`, when bật `Tra cứu` qua công tắc, then thanh trạng thái không còn nút mở ngăn kéo và cả phiên không có `declareFocus` ném.
- Given app thật, when tắt rồi bật từng panel bằng chuột, then panel về đúng chỗ cũ, đúng kích thước và mọi cú bấm đều tới.

## Implementation Notes

- `findTreeSpot` trả một lá làm neo nên tắt rồi bật Đối chiếu ra `(Đối chiếu | Tra cứu) / Đề xuất AI`; `spotPosition` thêm vào mép dock khi neo là nhánh, dùng chung cho `showPanel` và `undoMerge`. Đề xuất AI và Tra cứu vốn về đúng cây; lượt e2e đầu gán nhầm AI vì các ca chạy nối tiếp trên bố cục đã hỏng.
- Bố cục preset mặc định mở đầu với một gốc một-con; sau khi bật lại Đối chiếu lớp đó mất, nhìn không khác, nên ca cây chuẩn hoá bọc ngoài trước khi so.
- `fromJSON` bắn đủ `onDidAddPanel`/`onDidRemovePanel`: `applyLayoutJson` giữ đúng tập, đo bằng test tạm, không cần lượt nghe `onDidLayoutFromJSON`.
- Tệp mới đẩy `FILE_FLOOR` 182 xuống dưới 80% của 228 tệp, `check:tokens` đỏ do chính story này; sàn lên 194.
- Đối chứng gỡ thật, mỗi phép đỏ đúng ca: ba `@click`, vế panel cuối, lọc mục tắt, vế `v-if` ở `StatusBar`, hai lượt nghe dock, nhánh mép dock.
- Selector `[aria-checked='true'] > .ui-switch__track` không tới track trong `.ui-row`, nên mục Sắp xếp panel có sẵn cũng chưa từng hiện trạng thái bật; Vitest và bàn đo chỉ đọc `aria-checked`, chỉ màu tính toán trong e2e bắt được.
- Tắt rồi bật Đối chiếu mất vị trí cuộn (600 → 0); ghi nợ Epic 14, xem [ket-qua.md](../../../../implementation-artifacts/14-8-ban-do/ket-qua.md).

## Plan Change Log

## Review Triage Log

Lượt 1, `quick`: 1 high · 2 medium · 11 low · 1 false; 13 patch, 2 reject.

| Verdict | Route | Phát hiện | Bằng chứng |
|---|---|---|---|
| high | patch | Công tắc không bao giờ hiện trạng thái bật | `[aria-checked='true'] > .ui-switch__track` là con trực tiếp, track nằm trong `.ui-row`; `.on` không có rule; dính cả Sắp xếp panel. Vá selector, e2e so màu tính toán. |
| medium | patch | Xung đột `declareFocus` vẫn tới được qua Concordance và phím `layout.lookup_drawer_open` | `tmConcordanceCommandDeps.ts:87` gọi `openLookupDrawer` không kiểm dock. Chắn ở `openLookupDrawer`. |
| medium | patch | Bật rồi tắt tay ở tầng hẹp, nới cửa sổ thì tầng trả panel về | `togglePanel` không xoá `autoHiddenIds`; `reconcileAutoHidden` thấy `hidden.has`. Ice nới Never cho một dòng. |
| low | reject | Phím `layout.toggle_lookup` khi ngăn kéo đang mở | Ngăn kéo là scrim phủ kín nên chuột không tới; chỉ còn phím tự gán lúc ngăn kéo mở, sửa cần thêm guard ở `togglePanel`. |
| low | patch | Gợi ý phím ngăn kéo còn hiện khi panel ở dock | `StatusBar.vue:696`; gộp vào computed `lookupDrawerOffered`. |
| low | patch | Không ca nào đi qua dây nối ở tầng hẹp | Ca `statusBarWorkspace` đặt state bằng tay. Thêm ca trên khung case N. |
| low | patch | e2e không phân biệt panel nào về chỗ nào | `groupBoxes()` so hình chữ nhật vô danh. Thêm định danh panel. |
| low | patch | `beside` coi nhánh một-con là nhánh thật | `root H[branch[a], b, c]`, ẩn `b` về mép phải. Thêm `besideKind`. |
| low | patch | Đĩa lệch màn hình khi gỡ gộp cạnh nhánh | `unmergeForPersist` bỏ qua `beside`; thêm nhánh mép gốc. |
| low | patch | Bấm mục tắt có thể đóng menu | `onRootClick` khớp cả mục tắt. Thêm `:not(:disabled)`, e2e bấm thật. |
| low | patch | Công tắc thiếu `title` tên đầy đủ và phím | EXPERIENCE §Voice and Tone. Thêm `commandTitle`. |
| low | patch | Vế "không gọi togglePanel" của ca panel cuối không đỏ được | VTU `trigger` bỏ qua phần tử tắt. Bỏ vế đó; `visible as never` sửa thành `PanelId[]`. |
| low | patch | Ca chữ chưa flush không chặn 2 giây | Thêm chặn thời gian dưới 2000 ms. |
| low | patch | Comment `§QĐ #3 phía trên` trỏ vào khối đã xoá | Bỏ "phía trên" ở `index.ts:1515`. |
| false | reject | Comment ở `index.ts:1452` và `:3047` trỏ khối đã xoá | Dòng 1452 nói vòng `layout.toggle_*` vẫn nằm ngay trên; dòng 3047 trỏ Story 1.6, không trỏ khối. |

## Verification

**Commands:**
- `npm run check:commands`, `npm run check:i18n`, `npm run check:tokens`, `npm run check:debt-owner` -- exit 0.
- `npx vitest run tests/frontend/titlebarToolsMenu.test.ts tests/frontend/workspaceDockTier.test.ts tests/frontend/statusBarWorkspace.test.ts`, `npx vue-tsc --noEmit`, `npm run check:lint` -- exit 0.
- `npm run build`, `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/panel-toggle-mouse.e2e.mjs` -- xanh, chạy ngoài sandbox.
- Chạy đủ bộ một lần: story chạm `scripts/check-*` (AGENTS.md §Tests).

**Manual checks (if no CLI):**
- Lượt dùng thật của Ice trước khi Epic 14 `done`: tắt, bật, đổi preset, đọc lại bố cục.
