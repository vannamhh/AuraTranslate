---
title: 'Ẩn tiêu đề panel một-panel, bật lại bằng "Sắp xếp panel"'
type: 'feature'
ticket: ''
created: '2026-10-26'
status: 'in-progress'
baseline_revision: '233ba4ec88ac97e921e598363fda35701812f3e3'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Mỗi panel Workspace mang thanh tab dock 36px chỉ để in tên ("Đối chiếu", "Tra cứu", "Đề xuất AI"); Ice muốn bỏ để rộng chỗ cho nội dung (2026-10-26, sau story 12-26).

**Approach:** Ẩn thanh tab của mọi nhóm dock chỉ có một panel. Mục "Sắp xếp panel" trong `Công cụ ▾` bật/tắt hiện lại mọi thanh tab để kéo thả và lưu bố cục; lựa chọn được nhớ qua các phiên bằng một khoá `app_config` toàn cục, cùng khuôn `ai_usage_visible`.

## Decisions (Ice, 2026-10-26)

1. Ẩn mặc định; bật lại qua "Sắp xếp panel", nhớ qua các phiên.
2. Nhóm có từ hai panel trở lên luôn hiện thanh tab, để không panel nào bị giấu mất.

## Boundaries & Constraints

**Always:**
- Hiện/ẩn thanh tab là trạng thái dẫn xuất từ (chế độ sắp xếp, số panel trong nhóm), đặt lại sau mọi lượt `fromJSON`, `applyPreset`, gộp/tách tầng hẹp và kéo thả.
- JSON bố cục ghi đĩa và bố cục tự đặt tên không mang `hideHeader`.
- Bật/tắt chế độ không tự lên lịch ghi bố cục.
- Ghi khoá cấu hình trượt ⇒ giữ trạng thái đã chọn trong phiên, báo lỗi qua `tError` như `aiUsageVisibleSaveError`.

**Never:**
- Không đụng `ReviewDock.vue` (thanh tab là tiêu đề duy nhất của MINE/COPY).
- Không thêm lớp nổi, không hiện thanh tab khi rê chuột.
- Không dependency mới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mặc định | Lần mở đầu, preset Lưới bên trái | Ba nhóm một-panel, không thanh tab nào hiện | — |
| Sắp xếp | Bấm "Sắp xếp panel" | Mọi thanh tab hiện, mục menu `aria-checked="true"`; kéo thả được | — |
| Tắt sắp xếp | Bấm lại | Nhóm một-panel ẩn thanh tab, nhóm ≥2 panel vẫn hiện | — |
| Gộp tab | Kéo AI vào nhóm Tra cứu rồi tắt sắp xếp | Nhóm đó vẫn hiện thanh tab | — |
| Tầng hẹp | `applyMerge` gộp AI vào nhóm Tra cứu | Nhóm gộp hiện thanh tab; `undoMerge` thì ẩn lại | — |
| Khởi động lại | Đã bật sắp xếp, thoát, mở lại | Thanh tab hiện | Khoá không đọc được ⇒ mặc định ẩn |
| Ghi lỗi | `put_config` trượt | Trạng thái trong phiên vẫn đổi; câu lỗi hiện | `tError` |
| Bố cục đã lưu | Lưu bố cục khi thanh tab đang ẩn, rồi áp lại khi đang sắp xếp | Thanh tab hiện | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/scope/store.rs:168,171,326,333` -- const + accessor; twin `panel_headers_shown` theo `update_check_enabled()` (bool kiểu `"on"`/khác).
- `src-tauri/src/commands/config.rs:116,166` -- trường `BootstrapConfig` + điền giá trị.
- `src-tauri/tests/ipc_contract.rs:182,201` -- struct literal và danh sách tên trường đóng băng.
- `src/config/bootstrap.ts:98,222,237,287,330` -- interface, `KEY_*`, ref, readonly export, parse.
- `src/aiUsageVisibleState.ts` -- khuôn cho `src/layout/panelHeadersState.ts` (computed + set + reset + saveError).
- `scripts/check-panel-refs.mjs:~247,256` -- mục EXEMPT cho ref bootstrap mới, như `src/config/bootstrap.ts::aiUsageVisible`.
- `src/layout/WorkspaceDock.vue` -- `onReady` :986-1046 (api); `restore()` :935, `applyLayoutJson()` :808, `applyPreset()` :230, `applyMerge()` :640/`undoMerge()`; `jsonForPersist()` :785 (bỏ `hideHeader`); thêm một listener `onDidLayoutChange` riêng gọi hàm đồng bộ (AsapEvent, chạy cả khi `suppressPersist`), cộng `watch` chế độ. ⚠️ `onDidAddGroup/RemoveGroup` không bắn khi kéo; `onDidLayoutChange` có.
- `src/layout/dockController.ts:44` -- thêm cổng nếu lệnh cần gọi vào dock.
- `src/commands/index.ts` -- `register` lệnh `layout.toggle_panel_headers` cạnh `layout.preset_*` :1401-1412; không import Vue (chạy dưới Node).
- `src/main.ts:576` -- nối dep vào `installCommands`.
- `src/TitlebarToolsMenu.vue` -- mục `role="menuitemcheckbox"` `:aria-checked`; `menuItems()` :24 và đóng-khi-bấm :61 đang chỉ chọn `[role="menuitem"]` ⇒ mở rộng selector.
- `src/i18n/vi.json` -- `command.layout.toggle_panel_headers` ("Sắp xếp panel").
- Tests: `tests/frontend/titlebarToolsMenu.test.ts` (:43, :91, :112, :139 đếm 8), `aiUsageVisibleState.test.ts` (khuôn), `workspaceDockTier.test.ts`, `layoutUserPresets.test.ts`; `e2e/specs/titlebar-tools.e2e.mjs:49` (danh sách `data-tools-item`).

## Tasks & Acceptance

**Execution (phase Rust):**
- [ ] `store.rs`, `config.rs`, `ipc_contract.rs` -- khoá `panel_headers_shown` theo khuôn `update_check_enabled`.

**Execution (phase webview):**
- [ ] `bootstrap.ts`, `panelHeadersState.ts`, `check-panel-refs.mjs` -- đọc/ghi khoá.
- [ ] `WorkspaceDock.vue` -- hàm đồng bộ `group.header.hidden = !shown && group.panels.length < 2` cho mọi nhóm; gọi sau restore/preset/layoutJson/merge/undoMerge, trong listener `onDidLayoutChange` riêng, và khi chế độ đổi; `jsonForPersist` xoá `hideHeader`.
- [ ] `commands/index.ts`, `main.ts`, `vi.json`, `TitlebarToolsMenu.vue` -- lệnh và mục menu checkbox.

**Execution (phase test):**
- [ ] Ca cho từng hàng của Matrix (vitest mount `WorkspaceDock` hoặc test hàm đồng bộ với group giả + một ca nối dây qua dock thật); sửa test menu và e2e `titlebar-tools` cho mục mới; ca `jsonForPersist` không có `hideHeader`. Đối chứng: gỡ lời gọi đồng bộ khỏi đường `applyMerge` ⇒ ca Tầng hẹp đỏ.

**Acceptance Criteria:**
- Given Workspace vừa mở, when nhìn ba panel, then không thanh tab nào và chiều cao nội dung mỗi panel tăng đúng chiều cao thanh tab.
- Given đang sắp xếp, when kéo một panel sang chỗ khác và lưu bố cục tên X, then áp lại X sau khi tắt sắp xếp thì đúng vị trí, nhóm một-panel ẩn thanh tab.

## Implementation Notes

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo test --test ipc_contract` (sau `npm run build`) -- expected: xanh.
- `npm run check:tokens && npm run check:i18n && npm run check:commands && npm run check:layout && npm run check:panel-refs && npm run check:lint` -- expected: xanh.
- `npx vitest run` các tệp đã sửa/thêm -- expected: xanh, mã thoát 0, không dòng Errors.
- Đủ bộ một lần: chạm `scripts/check-panel-refs.mjs` và bootstrap IPC.

**Manual checks:**
- App thật: ba panel không thanh tab; bật/tắt "Sắp xếp panel"; kéo AI vào nhóm Tra cứu; khởi động lại giữ chế độ.
