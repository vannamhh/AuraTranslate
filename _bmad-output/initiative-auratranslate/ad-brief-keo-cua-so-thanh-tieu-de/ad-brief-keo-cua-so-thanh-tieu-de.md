---
type: ad-brief
title: "Hồ sơ bàn giao cho Winston — một `AD` mới về **kéo cửa sổ từ thanh tiêu đề của webview** (macOS `titleBarStyle: Overlay`)"
status: done
created: 2026-10-25
skill: bmad-architecture
---
# Hồ sơ bàn giao cho Winston — kéo cửa sổ từ thanh tiêu đề của webview

✅ ĐÃ ĐÓNG 2026-10-25 — `AD-58` đã viết (spine §AD-58). Review thêm vào AD ba điều hồ sơ chưa có: vùng kéo chỉ trên macOS, listener bấm-ra-ngoài ở pha capture, lề trên-trái của lớp phủ toàn cửa sổ.

**Ngày:** 2026-10-25 · **Người bàn giao:** Amelia · **Người nhận:** Winston (architect)
**Quyết định của Ice (2026-10-25):** ba tab chế độ nằm cùng hàng với ba nút cửa sổ của macOS, như mockup (`ux-auratranslate/mockups/web-import.html:22-23,168-172`, thanh `.tb` 38px có ba chấm). Ice chọn phương án A (thanh tiêu đề macOS phủ lên webview) thay vì B (`titleBarStyle: Transparent`, tab vẫn ở hàng hai).
**Baseline:** `41e956f` · spine dừng ở AD-57 (AD-58 chỉ là ứng viên bị loại trong `.memlog.md` 2026-10-23, chưa viết) · `capabilities/main.json` mang **3** quyền.

## 1. Vì sao cần một AD

`titleBarStyle: Overlay` bỏ dải tiêu đề 28pt của macOS; webview chiếm cả chỗ đó, nên không còn vùng nào kéo được cửa sổ. Phải dựng vùng kéo trong webview, và mọi đường dựng đều thêm một lệnh IPC mới. Bộ quyền tối thiểu bị khoá bởi `config_invariants.rs::main_capability_grants_the_minimum_and_no_plugin_permission` và `src-tauri/SECURITY-NOTES.md` §`capabilities/main.json`: *"Thêm một tập nữa là mở một bề mặt IPC mới — phải là một AD mới trước đã"*.

## 2. Phần không cần AD (ghi để Winston khỏi đọc lại)

- `tauri.conf.json`, cửa sổ `main`: `"titleBarStyle": "Overlay"`, `"hiddenTitle": true`, `"trafficLightPosition": {x, y}` — có ở `tauri-utils-2.9.3/src/config.rs:2060-2068`. `trafficLightPosition` chỉ có tác dụng khi `decorations: true` (mặc định). Chỉ macOS; Windows giữ thanh tiêu đề hệ điều hành.
- `src/App.vue`: lề trái cho ba nút chỉ trên macOS (dò nền tảng đã có ở `src/commands/index.ts:120-134`).
- AD-24 (một cửa sổ, label `main`) không đổi.

## 3. Hai nhánh — Ice chọn

Cả hai nhánh mở cùng một năng lực: webview yêu cầu bắt đầu kéo **chính cửa sổ của nó** (và tuỳ chọn phóng to/thu lại khi bấm đúp). Không đọc dữ liệu, không chạm tệp. AD-24 giữ một cửa sổ, nên tham số `label` của lệnh plugin không chỉ được sang cửa sổ khác.

| | (a) Quyền core của Tauri | (b) Lệnh riêng của app, gọi từ Rust |
|---|---|---|
| Quyền thêm vào `main.json` | `core:window:allow-start-dragging` + `core:window:allow-internal-toggle-maximize` (bấm đúp), 3 → 5 | 0 |
| Lệnh IPC mới | 2 lệnh plugin `window` có sẵn | 1–2 lệnh `#[tauri::command]` đăng ký trong `lib.rs::generate_handler!` (`lib.rs:1118`) |
| Mã webview | `data-tauri-drag-region="deep"` trên `<header class="titlebar">`; Tauri tự tiêm `tauri-2.11.5/src/window/scripts/drag.js` | Tự viết lại logic của `drag.js`: loại phần tử bấm được (`BUTTON`, `role=tab`…), bấm đúp trên macOS xử lý ở `mouseup` và huỷ nếu chuột đã di (tauri issue #8306) |
| Test phải đổi | `config_invariants.rs` (danh sách quyền) · `SECURITY-NOTES.md` | Test cho lệnh mới + test cho handler `mousedown` |
| Tiền lệ | — | AD-48 nhánh (a): gọi từ Rust, 0 quyền JS. Nhưng lý do ở đó là phạm vi đường dẫn của quyền `dialog` phía JS; ở đây hai nhánh mở cùng một năng lực, nên tiền lệ chỉ là sở thích nơi đặt cổng |

Tên quyền đã đối chiếu với `src-tauri/gen/schemas/desktop-schema.json`. Với `deep`, nút tab và mọi phần tử bấm được trong header vẫn chặn kéo (`drag.js` `isDragRegion`: phần tử bấm được không mang thuộc tính ⇒ `false` trước khi lên tới header). Với thuộc tính trần thì chỉ bấm đúng vào `<header>` mới kéo; khoảng trống trong `<nav class="modes">` không kéo được.

## 4. Giới hạn đã biết, nhánh nào cũng có

- Cửa sổ không được chọn thì lần bấm đầu chỉ chọn cửa sổ, chưa kéo (tauri issue #4316, ghi trong doc của `TitleBarStyle::Overlay`, `tauri-utils-2.9.3/src/lib.rs:174-178`). `acceptFirstMouse` có thể chữa, nhưng nó làm cú bấm vào cửa sổ đang ở nền kích hoạt luôn nút bên dưới; đề nghị không bật. Chưa đo.
- Toàn màn hình: ba nút ẩn, lề trái ~80px vẫn còn. Bỏ lề cần nghe sự kiện cửa sổ, tức là thêm quyền; đề nghị giữ lề.
- Chiều cao dải tiêu đề đổi theo phiên bản macOS (doc `Overlay`), nên `trafficLightPosition.y` cho thanh 40px (`--space-titlebar-height`) phải đo trên máy thật. Mục đó thành nợ `Chủ:` của story thi hành, Ice kiểm một lượt trên app thật.

## 5. Lợi đo được

Dải tiêu đề macOS 28pt trên MacBook của Ice (1536×960pt, thanh menu 25pt — `deferred-work.md`, grep `28pt title bar`). Bỏ nó trả 28pt cho nội dung; Windows không đổi.

## 6. Đã chốt và còn lại cho Winston

- Ice chọn (2026-10-25): **nhánh (a)**, **có** bấm đúp để phóng to ⇒ thêm đúng hai quyền `core:window:allow-start-dragging` và `core:window:allow-internal-toggle-maximize`, bộ quyền 3 → 5.
- Winston viết: AD-58 (số kế tiếp lúc viết), câu cập nhật cho `SECURITY-NOTES.md` §`capabilities/main.json` và cho thông điệp assert của `config_invariants.rs`.
