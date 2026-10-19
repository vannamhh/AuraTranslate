---
title: 'Review Mode — bố cục hai cửa sổ side-by-side'
type: 'feature'
ticket: '11'
created: '2026-10-09'
status: 'built'
baseline_revision: 'de33eea7172b4183e8bed97b7ff98da971420af8'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Bản reviewer đã nhập (8.9) và đã khớp (8.10) hiện chưa có chỗ nào để người dịch xem cạnh bản của mình. 8.12 (diff) và 8.13 (chấp nhận) cũng cần có sẵn bố cục này.

**Approach:** Review Mode là một trạng thái bố cục bên trong Workspace (AD-24), không phải chế độ thứ tư. Có hai panel chỉ đọc: trái "Bản dịch của tôi", phải "Bản Reviewer đã sửa". Dữ liệu lấy từ lệnh `alignment_open` có sẵn. Lệnh mở và đóng đều đăng ký trong `CommandRegistry` (AD-34). Khi rời Review Mode, bố cục Workspace trước đó được trả lại nguyên vẹn.

## Boundaries & Constraints

**Always:**
- Bố cục Review Mode không bao giờ được ghi vào `workspace_layout`. Mở lại ứng dụng thì luôn vào bố cục thường.
- Hai panel khai điểm vào focus trong `FOCUS_OWNERS`. Khi mở, focus vào panel trái; khi đóng, focus về panel đang hoạt động trước đó. Không lúc nào focus rơi về `body`.
- `NotImported` và `Stale` là hai lời báo riêng, không bao giờ hiện thành panel rỗng (AD-52 ⑥).
- Câu chưa dịch (`target_text` NULL) hiện một nhãn có chữ, không để khối trắng.
- Vue chỉ render bằng nội suy văn bản.

**Never:**
- Thêm phần tử vào `MODE_IDS` hay mở cửa sổ OS thứ hai.
- Sửa nội dung trong hai panel. Đây là bề mặt chỉ đọc, và lệnh chấp nhận là của 8.13.
- Ẩn nguyên văn, bôi màu diff, cuộn đồng bộ hay ghép cặp theo "Đoạn N". Các việc này là của 8.12.
- Thêm lệnh Rust mới, hoặc ghi `segment`/`review_row`.

## Decisions (Ice, 2026-10-09)

- **Bố cục:** dock thứ hai. `WorkspaceMode` đổi qua lại giữa `WorkspaceDock` (giữ sống, không bị gỡ) và `ReviewDock.vue` không có đường ghi. Lệnh `layout.*` từ chối khi đang Review.
- **`Stale`:** báo "Chương đã đổi sau lúc nhập", không vào bố cục Review, mời nhập lại. Vế "bản reviewer của Chương bị gộp đã bị bỏ" (AD-52 ⑤) không đọc ra được từ lược đồ; nó thành mục nợ `Chủ: Winston`.
- **Đơn vị hiển thị:** hai danh sách độc lập, segment theo `ord` và hàng reviewer theo `ord`. Ghép cặp và cuộn đồng bộ là của 8.12.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Mở | Chương đang mở đã có bản reviewer | bố cục hai panel, trái là bản dịch, phải là hàng reviewer theo `ord`; focus ở panel trái | — |
| Chưa có bản | `NotImported` | bố cục không đổi | lời báo không chặn, kèm hành động nhập tệp (`export.reviewer_import.*`) |
| Lỗi thời | `Stale` | bố cục không đổi | lời báo "Chương đã đổi sau lúc nhập", mời nhập lại |
| Không có Chương | `editorChapterId` null | bố cục không đổi | lời báo riêng |
| Đóng | đang ở Review Mode, gọi `review.close` | bố cục, panel đang hoạt động và vị trí trong lưới trả về như trước lúc mở | — |
| Đổi chế độ rồi quay lại | Library → Workspace khi đang ở Review | vẫn ở Review Mode | — |
| Đóng Tác phẩm | đang ở Review | trạng thái Review bị xoá, lần mở Workspace sau vào bố cục thường | — |

</frozen-after-approval>

## Code Map

- `src/config/alignment.ts:7-172` -- `alignmentOpen(chapterId)` trả `ChapterAlignment` với `segments` (đã lọc "thuộc bản dịch") và `rows`. Lỗi đi ra dưới khoá `export.alignment_not_imported` và `export.alignment_stale` (`commands/export.rs:538-554`). Đây là nguồn dữ liệu duy nhất, không thêm lệnh Rust. Lần đọc đầu tiên có thể chạy `align_if_pending` (`alignment.rs:440`); đó là hành vi có sẵn.
- `src/panels/editorPanelState.ts:156-158` -- `editorChapterId` là Chương để mở Review, giống `alignmentCommandDeps.ts:29-31`.
- `src/layout/WorkspaceDock.vue` -- không đổi hành vi. Đọc để chép khuôn: `addPanel` `:225`, `restoreFocusIfLost` `:449`, `onReady` `:944`. `onDeactivated`/`onActivated` `:1036-1052` gỡ và đặt lại `setDockController`; khi Review đang hiện, controller phải là `null` hoặc từ chối, không trỏ vào dock bị giấu (`src/layout/dockController.ts`). Không thêm panel Review vào `PANEL_IDS`/`PANEL_COMPONENTS`/`LAYOUT_PRESETS` (`workspaceLayout.ts:45-169`), để Kiểm A/E/F của `check:layout` không đổi.
- `src/layout/ReviewDock.vue` (mới) -- `DockviewVue` hai panel cạnh nhau, không `emit('persist')`, không `addPopoutGroup`/`localStorage` (Kiểm C quét cả tệp này).
- `src/modes/WorkspaceMode.vue` -- `declareFocus('mode.workspace')` `:42`, `onPersist` `:81`, vị trí lời báo không chặn (`tier-change` `:15,104`).
- `src/commands/index.ts` -- `FOCUS_OWNERS` `:67`; `CommandDeps` `:169`, theo khuôn deps reviewer/alignment `:895-916`; đăng ký theo khuôn `:3347-3440`; comment chừa `Mod+Alt+3` `:1212-1225`. `review.open` lấy `Mod+Alt+3`. `review.close` không có hợp âm mặc định nhưng gán được qua `shortcutsState.ts:131`.
- `src/alignmentCommandDeps.ts`, `src/main.ts:879-880` -- khuôn tệp deps, trải vào `installCommands`.
- `src/App.vue:298-325` -- nút thanh tiêu đề `data-review-open`, gồm `@mousedown="focusOnPointerDown($event)"` và đúng một `@click="dispatch('review.open')"` (Kiểm A). Comment `:40` và `AttributionOverlay.vue:8` ghi sai `Mod+4`; sửa thành `Mod+Alt+3` vì đang chạm đúng dòng đó.
- `src/libraryChapters.ts:302-324`, `src/libraryImport.ts:422-441` -- danh sách reset khi đổi Tác phẩm; thêm `resetReviewMode` vào đây.
- Cổng:
  - `check:commands`: Kiểm E đối chiếu `FOCUS_OWNERS`; `HANDLER_TABLE` `:2313` cho mọi `@keydown`; `COMMAND_FLOOR` `:237`.
  - `check:panel-refs`: mỗi ô nhớ cấp module phải có hàm `reset*`.
  - `check:i18n`, `check:tokens`.
  - Tệp mới làm sàn quần thể 80 % trôi; nâng lên `ceil(0.85×live)` cùng lượt (Decision 2026-10-07 của epic).
- Test: khuôn mock `tests/frontend/alignment.test.ts:15,195-199`; dock thật ở `workspaceDockTier.test.ts`.

## Tasks & Acceptance

**Execution:**
- [x] `src/reviewModeState.ts` (mới) -- trạng thái `idle | loading | open | not_imported | stale | no_chapter`, `openReviewMode`/`closeReviewMode`/`resetReviewMode`, đọc qua `alignmentOpen`.
- [x] `src/panels/ReviewMinePanel.vue`, `src/panels/ReviewCopyPanel.vue` (mới) -- chỉ đọc, `PanelFrame` có owner riêng, chỉ nội suy văn bản.
- [x] `src/layout/ReviewDock.vue` (mới) + `src/modes/WorkspaceMode.vue` -- đổi hai dock bằng `<KeepAlive>` hoặc `v-show`, giữ `WorkspaceDock` sống; khi đang Review, `layout.*` từ chối và controller không trỏ vào dock bị giấu.
- [x] `src/commands/index.ts` + `src/reviewModeCommandDeps.ts` (mới) + `main.ts` -- `review.open` (`Mod+Alt+3`), `review.close`, hai owner focus, nhãn `command.review.*` trong `vi.json`.
- [x] `src/App.vue` -- nút thanh tiêu đề, sửa hai comment `Mod+4`. `resetReviewMode` vào hai danh sách reset.
- [x] `tests/frontend/reviewMode.test.ts` (mới) -- cả ma trận I/O, kèm đối chứng gỡ thật: gỡ chỗ chặn ghi thì ca "không ghi" đỏ; gỡ dòng gọi `enterFocus` thì ca focus đỏ.
- [x] `deferred-work.md`
  - Đóng `:670` (preset Review Mode).
  - Đóng `:12988` (lời báo `Stale`) bằng `→ 🟡`: vế "bản bị gộp đã bị bỏ" chưa làm, `Chủ: Winston` (cần bổ sung AD-52 và một migration).
  - `:3461`: Review Mode chỉ đọc nên không phải bề mặt soạn thảo thứ hai; nối dòng `→ … Chủ: Ice`.
  - Nợ mới `Chủ: Epic 8`: một lượt dùng thật, focus không rơi về `body` trong WKWebView.

**Acceptance Criteria:**
- Given đang Review, when chạy `flush` hoặc ghi theo lịch, then không có lần ghi `workspace_layout` nào mang panel Review, và giá trị đã lưu trùng từng byte với bố cục trước lúc mở.
- Given đang Review, when gọi một lệnh `layout.preset_*`/`layout.toggle_*`, then bố cục `WorkspaceDock` không đổi.
- Given đóng Review, when quay về lưới, then component `GridPanel` là đúng instance trước lúc mở (không bị dựng lại) và `scrollTop` của nó không đổi.
- Given đang Review, when gọi `review.close`, then `document.activeElement` nằm trong panel đang hoạt động trước lúc mở, không phải `body`.
- Given `review.open` và `review.close`, when mở màn phím tắt, then cả hai có mặt và gán lại được; `review.open` mặc định là `Mod+Alt+3`.
- Given `MODE_IDS`, when Review đang mở, then hằng vẫn có ba phần tử và `currentMode` là `workspace`.

## Implementation Notes

- Hai dock nằm chồng lên nhau trong cùng một ô lưới. `WorkspaceDock` chỉ bị ẩn bằng `visibility: hidden` cộng `inert` nên kích thước giữ nguyên; dockview không đo lại về 0 và không phát lượt ghi nào. `ReviewDock` chỉ được mount khi trạng thái là `open`.
- Việc "không ghi" được bảo đảm vì `ReviewDock` không có đường `persist`. Không có chỗ chặn ghi nào để gỡ làm đối chứng. AC byte-trùng nằm chung một ca với `layout.*`; gỡ `setDockSuspended(true)` thì ca đó đỏ.
- `dockController.ts` có thêm `setDockSuspended`/`resetDockSuspended`/`activeDockPanelId`. Khi đang suspended thì `applyPreset`/`togglePanel` trả `false`, còn `panelRing` trả `[]`.
- "Chưa dịch" là `target_text === ''`, không phải NULL: `segment.target_text` khai `NOT NULL DEFAULT ''` (`schema.rs:199-206`). Chữ "NULL" trong Boundaries là chỗ plan ghi sai.
- Có thêm trạng thái `error` cho lỗi IPC khác, hiện thành lời báo riêng. `review.open` gọi `setMode('workspace')` trước, nên phím tắt chạy được cả từ Library.
- Hai danh sách reset thật nằm ở `src/modes/libraryChapters.ts` và `src/modes/libraryImport.ts`; đường dẫn trong Code Map đã cũ.
- `VUE_FLOOR` 33 → 36 ở `check-commands.mjs` và `check-i18n.mjs`.
- Lượt chạy cả bộ bắt được `naming_boundary` đỏ: `ReviewDock` đọc `e.origin`, mà bảng miễn trừ chỉ có `WorkspaceDock`. Cách sửa không phải nới danh sách miễn trừ: việc đọc trường dockview đó dồn vào `isUserActivation` trong `dockController.ts`, hai dock cùng gọi hàm này, và mục miễn trừ duy nhất chuyển sang `dockController.ts`. Không tạo tệp mới, vì chỉ thêm một tệp là các sàn quần thể trôi qua ngưỡng.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 4 · low 4 · false 3 · maybe-false 0.

- medium · patch -- đóng lời báo bằng nút "Đóng lời báo": `v-if` gỡ nút đang giữ focus, còn `closeReviewMode` thoát sớm khi `!wasOpen`, nên focus rơi về `body`. Sửa: trên đường lời báo, đưa focus về `mode.workspace`.
- medium · patch -- mở `review.open` từ Library: controller của dock còn `null` vì `<KeepAlive>` chưa kích hoạt lại, nên `previousOwner` thành owner của Library. Sửa: chụp `previousOwner` sau `await`, và chỉ lấy từ dock.
- medium · patch -- `panelRing` trả `[]` khi suspended, nên không tới được panel phải bằng bàn phím (AD-34 ②). Sửa: khi suspended, trả về vòng gồm hai panel Review.
- medium · patch -- nhập xong bản reviewer mà lời báo "chưa có bản"/"đã đổi" vẫn còn trên màn hình, và Review đang mở vẫn hiện bản cũ. Sửa: gọi `resetReviewMode()` khi xác nhận nhập thành công.
- low · patch -- ca "reset" không đỏ được: không có dock nào mount nên `absent()` cũng trả `false`. Sửa: assert `isDockSuspended()` và chạy đối chứng gỡ thật.
- low · patch -- doc comment mới ở `dockController.ts` chỉ nhắc lại chữ ký hàm, hoặc viết bằng tiếng Việt; comment CSS ở `WorkspaceMode.vue` cũng bằng tiếng Việt. Sửa: xoá, hoặc viết lại thành một dòng tiếng Anh.
- low · patch -- câu "Review Mode của Story 8.11 dựng cái thứ hai" ở `WorkspaceDock.vue` không còn đúng (`ReviewDock` không gọi `setDockController`). Sửa: xoá câu đó.
- low · reject -- lời báo `error` không nêu chi tiết lỗi IPC: hiếm gặp, và muốn sửa phải thêm một nhánh hiển thị.
- false -- `resetReviewMode` để focus nằm trên một nút đã bị gỡ: cả hai lời gọi đều chạy từ Library (`openWorkById` ở `libraryChapters.ts:361`/`librarySearch.ts:354`, `finishImportSubmission` ở `main.ts:543,558`), và `onActivated` của Workspace đặt lại focus.
- false -- dòng nối `:3461` thiếu dấu: `→ … Chủ: <mới>` đúng là dạng chuyển chủ mà AGENTS.md quy định.
- false -- plan thiếu bằng chứng đối chứng gỡ và các ô Execution chưa đánh dấu: bằng chứng đã có trong Implementation Notes và báo cáo của agent cài đặt; sửa plan thì bị luật triage bác.
- Ghi chú quy trình -- diff chạm `scripts/check-*` (nâng sàn), nên theo AGENTS.md phải chạy cả bộ một lần bằng tay; dòng Verification "không chạy cả bộ" là sai.

## Verification

**Commands:**
- `npx vitest run tests/frontend/reviewMode.test.ts` -- xanh; mỗi phép gỡ đối chứng làm đỏ đúng ca của nó
- `npm run check:commands && npm run check:layout && npm run check:panel-refs && npm run check:i18n && npm run check:tokens` -- xanh
- Không chạm dây dùng chung (`lib.rs`, migration, `Cargo.toml`, `vitest.config.ts`) ⇒ không chạy cả bộ; `pre-push` chạy một lần.
