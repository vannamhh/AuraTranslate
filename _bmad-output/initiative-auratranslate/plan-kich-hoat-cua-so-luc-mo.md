---
title: 'Kích hoạt cửa sổ chính lúc mở trên macOS để click đơn không bị nuốt'
type: 'bugfix'
ticket: ''
created: '2026-10-25'
status: 'dropped'
baseline_revision: 'f88530bf6b0372677f8f2226f8ea577802894df8'
route: 'full'
route_source: 'auto'
risk: 'low'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Mở bằng `npm run tauri dev` (binary trần, không qua LaunchServices), app lên với `foreground=0`, AppKit ghi `ordered front from a non-active application`, và cửa sổ không thành cửa sổ chính. Trong 41–123 s đầu, mọi click đơn trong mọi cửa sổ của tiến trình (cả Web Inspector) bị nuốt, chỉ double click lọt qua và ô tìm không nhận tiêu điểm.

**Approach:** Ice chọn chủ động kích hoạt: trong `setup()`, trên macOS, gọi `set_focus()` cho cửa sổ `main` để app active và cửa sổ thành cửa sổ chính ngay lúc mở. Phương án `acceptFirstMouse: true` đã bị bác vì không chữa bàn phím và đổi hành vi bấm cửa sổ nền ở bản phát hành.

</frozen-after-approval>

## Implementation Notes

Oneshot: một hàm `focus_main_window` nhỏ trong `src-tauri/src/lib.rs`, gọi từ `setup()`, chỉ biên dịch trên macOS, theo khuôn `let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else { eprintln!(…); return; }` của `wire_drag_drop`.

- ⇒ Không có tác dụng, đã hoàn nguyên. Lượt `tauri dev` 23:40:43 có `set_focus()`: unified log vẫn `ordered front from a non-active application`, không có `order all windows front`. `set_focus` của tao là `makeKeyAndOrderFront` + `activateIgnoringOtherApps:YES` (`tao-0.35.3/src/platform_impl/macos/util/async.rs:231`), và tao đã gọi đúng API đó lúc khởi động (`app_state.rs:293`); macOS 15.8 kích hoạt theo kiểu hợp tác nên app không tự lên được khi app đang ở trên cùng không nhường.
- Số đo năm lượt (unified log, `foreground=0` lúc `CHECKEDIN`): lần kích hoạt thật đầu tiên ở sau 41–123 s; lượt Ice 22:39 sau 112 s. Trong 33 s đầu lượt 23:10, `lsappinfo front` báo `auratranslate` dù AppKit chưa active — nghi vấn: LaunchServices coi app đã ở trên cùng nên click không gửi lệnh kích hoạt (chưa đo trực tiếp).

## Code Map

- `src-tauri/src/lib.rs` `setup()` (~1458) -- chỗ duy nhất chạy một lần lúc mở sau khi cửa sổ `main` đã tạo.
- `src-tauri/tauri.conf.json` `app.windows[0]` -- nơi khai `acceptFirstMouse` nếu chọn hướng đó (`tauri-utils-2.9.3/src/config.rs:2071`, mặc định `false`).
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tao-0.35.3/src/platform_impl/macos/app_state.rs:283-293` -- đường kích hoạt lúc khởi động của tao.

## Quyết định

- 2026-10-25, Ice: không sửa mã. `acceptFirstMouse: true` bị loại vì trái AD-58 Rule 1/7 (khoá ở `src-tauri/tests/config_invariants.rs` `main_window_block_matches_ad_58`), và Ice không mở AD-59. Triệu chứng cùng thao tác tránh được ghi một dòng trong `AGENTS.md` §This machine. Cmd-Tab đi rồi về chưa đo.

## Verification

**Commands:**
- `cargo build --manifest-path src-tauri/Cargo.toml` -- expected: biên dịch sạch.

**Manual checks (if no CLI):**
- `log show --predicate 'process == "auratranslate"'` sau một lượt `npm run tauri dev`: có `order all windows front` trong vòng ~1 s sau `CHECKEDIN … foreground=0`.
- Ice mở `npm run tauri dev` từ terminal Orca sau một lần sửa mã: click đơn vào thẻ Tác phẩm và ô tìm ăn ngay, gõ được vào ô tìm.
