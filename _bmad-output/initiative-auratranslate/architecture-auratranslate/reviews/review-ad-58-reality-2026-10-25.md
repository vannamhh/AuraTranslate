# Review thực tế AD-58 (và dòng 🔵 của AD-48 Rule 2) — 2026-10-25

Ống kính: mỗi quyết định đã cam kết có được đối chiếu với nguồn tải về, không phải với trí nhớ. Phạm vi đọc: spine dòng 770-776 và 1139-1162; nguồn `tauri-utils-2.9.3`, `tauri-2.11.5`, `tauri-runtime-wry-2.11.4`, `node_modules/@tauri-apps/api`, `src-tauri/`, `src/`.

Kết luận: các khẳng định về nguồn Tauri đều đúng. Không có lỗi nghiêm trọng. Có hai chỗ hở về thi hành (cổng cấm chưa kín, tài liệu lệch sau khi đổi "ba"→"năm") và vài sắc thái thiếu.

## Khẳng định đã xác nhận (kèm file:line)

| Khẳng định | Nguồn nói gì | Kết quả |
|---|---|---|
| `trafficLightPosition` cần Overlay + decorations true | `tauri-utils-2.9.3/src/config.rs:2063-2065` "Requires titleBarStyle: Overlay and decorations: true." | Đúng |
| `titleBarStyle`, `hiddenTitle` có trong cấu hình cửa sổ | `config.rs:2058-2067` | Đúng |
| Ba khoá chỉ có tác dụng trên macOS | `tauri-runtime-wry-2.11.4/src/lib.rs:864-878` bọc trong `#[cfg(target_os = "macos")]` | Đúng |
| Overlay: phải tự dựng vùng kéo, không kéo được khi cửa sổ không focus (#4316) | `tauri-utils-2.9.3/src/lib.rs:172-178` | Đúng, đúng chữ |
| Transparent "0 quyền mới, tab vẫn hàng thứ hai" | `lib.rs:164-165` mô tả Transparent: nền cửa sổ hiện thay thanh tiêu đề, không phủ nội dung | Đúng (phần "không đạt mockup" là phán đoán sản phẩm, không kiểm được từ nguồn) |
| `acceptFirstMouse` là khoá thật, mặc định false | `config.rs:2068-2071` (`pub accept_first_mouse: bool`, `#[serde(default, alias="accept-first-mouse")]`); mặc định `false` tại `config.rs:2334` | Đúng |
| `drag.js`: nút/`role=tab`/`tabindex` chặn kéo; `"deep"` = cả cây con; `"false"` chặn | `tauri-2.11.5/src/window/scripts/drag.js:12-37` (CLICKABLE_TAGS, INTERACTIVE_ROLES, isClickableElement) và `:39-61` (isDragRegion) | Đúng |
| Gọi `plugin:window\|start_dragging` / `internal_toggle_maximize` | `drag.js:96-97` | Đúng |
| macOS: nhấp đúp xử lý ở `mouseup`, Windows/Linux ở `mousedown` | `drag.js:81-87` (macOS return sớm ở mousedown) và `:104-121` (mouseup) | Đúng |
| `drag.js` tiêm bất kể capability | `tauri-2.11.5/src/window/plugin.rs:241-262` (`Builder::new("window").js_init_script(init_script)`, không điều kiện ACL) | Đúng; ACL chỉ chặn lúc `invoke` |
| `internal_toggle_maximize` luôn toggle (không theo cài đặt macOS) | `plugin.rs:219-233`: nếu `is_resizable` thì maximize/unmaximize, không đọc cài đặt hệ thống | Đúng. Thêm sắc thái: no-op nếu cửa sổ không resizable/maximizable (`plugin.rs:225-231`); `tauri.conf.json:21` đặt `resizable: true` nên không vướng |
| Lệnh cửa sổ nhận `label` | `plugin.rs:13-21` `get_window(window, label)`; `internal_toggle_maximize` ở `plugin.rs:220-223` nhận `label: Option<String>`; `start_dragging` qua macro `setter!` `plugin.rs:165` | Đúng; AD-24 một cửa sổ nên `label` chỉ trỏ về `main` |
| Định danh quyền | `src-tauri/gen/schemas/desktop-schema.json:1502` `core:window:allow-internal-toggle-maximize`, `:1856` `core:window:allow-start-dragging` | Đúng chuỗi |
| `core:window:default` phơi hàng chục lệnh | `desktop-schema.json:1422` liệt kê ~28 `allow-*` | Đúng. Lưu ý: `internal_toggle_maximize` đã nằm sẵn trong `core:window:default` (cũng `tauri-2.11.5/build.rs:125`, cờ `true`), còn `start_dragging` thì không (`build.rs:113`, `false`) |
| `@tauri-apps/api` khớp dòng crate | `package.json:35` pin `2.11.1`; `node_modules/@tauri-apps/api/package.json:3` = 2.11.1; `Cargo.lock` tauri = 2.11.5 | Cùng dòng 2.11, khớp; `@tauri-apps/api/window` là đường nhập đúng để cấm |
| Token `--space-titlebar-height` có thật | `src/App.vue:428`, `src/layout/WorkspaceDock.vue:565` | Đúng |
| Bộ dò nền tảng có sẵn | `src/commands/index.ts:115-138` `detectIsMac()` (Chỗ DUY NHẤT đọc nền tảng) | Đúng |
| Cổng hiện tại khoá ba chuỗi | `src-tauri/tests/config_invariants.rs:~370-388` `assert_eq!` ba chuỗi; thông điệp đã có câu "phải là một AD mới trước đã" | Đúng; AD-58 là thay đổi có chủ đích của cổng này |
| `capabilities/main.json` hiện ba quyền | `src-tauri/capabilities/main.json:6` | Đúng (trạng thái trước khi thi hành) |

## Phát hiện

### F1 — medium: cổng cấm ở Rule 3 hẹp hơn lời cấm
- Khẳng định: "mã của app không gọi `plugin:window|*`, không import `@tauri-apps/api/window`"; test "đỏ khi có import `@tauri-apps/api/window`".
- Thực tế: cùng năng lực `startDragging()` còn qua `@tauri-apps/api/webviewWindow` (`WebviewWindow`/`getCurrentWebviewWindow`) và qua `invoke('plugin:window|…')` trực tiếp; `drag.js` chính nó gọi `window.__TAURI_INTERNALS__.invoke('plugin:window|'+cmd)` (`drag.js:96-97`). Test chỉ đếm một đường nhập sẽ xanh khi có đường khác. Lệnh `label` (`plugin.rs:13-21`) cũng không bị cổng này canh.
- Đề xuất: test quét ba mẫu: `@tauri-apps/api/window`, `@tauri-apps/api/webviewWindow`, chuỗi `plugin:window|`; hoặc ghi rõ trong AD rằng ACL (đúng năm quyền) mới là cơ chế cưỡng chế thật, test import chỉ là lưới phụ. (Bám nguyên tắc "một guard phải có khả năng đỏ").

### F2 — medium/low: tài liệu cận kề sẽ lệch sau khi đổi ba→năm
- AD-58 Rule 2 nêu cập nhật `SECURITY-NOTES.md` "thêm hai hàng" và thông điệp assert, nhưng `src-tauri/SECURITY-NOTES.md:134` ("đúng BA quyền") và `:157` (tiêu đề "ba tập quyền, không phải `core:default`") cùng `capabilities/main.json:4` (mô tả "Ba tập quyền dưới đây…") và chú thích assert (`config_invariants.rs` "Tối thiểu THẬT, ba tập") đều còn chữ "ba". AD-58 không liệt kê các chỗ này trong Binds/Rule 2 (chỉ nói "thêm hai hàng"). Rủi ro: "một sự thật, một chỗ" bị vi phạm, đọc sau ra mâu thuẫn.
- Đề xuất: liệt kê các chỗ "ba" cần sửa cùng commit.

### F3 — low: "tabindex ≥ 0" không đúng chính xác với drag.js
- `drag.js:32` coi mọi `tabindex` khác `"-1"` là clickable (cả `tabindex="-2"` hay giá trị rác), không phải "≥ 0". Hơn nữa `contenteditable` (khác `"false"`), `LABEL`, `SUMMARY`, `A`, `INPUT`, `SELECT`, `TEXTAREA` cũng chặn (`drag.js:13-34`), và vai trò `link/menuitem/checkbox/radio/switch/option` (`drag.js:22-31`). Mô tả trong AD là tập con đúng cho trường hợp thực tế; nên nói "theo `isClickableElement` của drag.js" thay vì liệt kê.
- Hệ quả thiết kế đáng ghi: phần tử `tabindex="-1"` (ví dụ chữ trạng thái focus được bằng script) KHÔNG chặn kéo; nếu có nút đặt `tabindex="-1"` trong header thì vẫn bị chặn nhờ thẻ `BUTTON`, nhưng `<div role="presentation" tabindex="-1" @click>` sẽ kích hoạt kéo thay vì bấm.

### F4 — low: hành vi bấm đúp macOS trên mousedown không được nêu
- `drag.js:81-87` ở macOS, `detail===2` return sớm, KHÔNG `preventDefault` — nên nhấp đúp trên chữ trong header có thể chọn từ (hành vi mặc định). Prevents 2 nói đúng chuyện mousedown/mouseup nhưng Rule 5 không ghi giới hạn này; nếu `user-select:none` đã có trên `.titlebar` thì không vấn đề (chưa kiểm, ngoài phạm vi đọc). Cũng: nhấp đúp phải đứng yên (`clientX/Y` bằng đúng, `drag.js:111-113`) mới phóng to.

### F5 — low: Rule 4 thiếu cách App.vue lấy cờ macOS
- `detectIsMac()` tồn tại (`src/commands/index.ts:130-138`) nhưng comment tại `:115-123` nói nó phải TIÊM ĐƯỢC và chỉ dùng ở đúng một chỗ, không gọi từ `keys.ts`. AD-58 nói "theo bộ dò nền tảng đã có" mà không nói gọi từ đâu (App.vue? một composable?), nên có thể phát sinh chỗ đọc thứ hai. Đề xuất nêu điểm gọi duy nhất.
- Ngoài ra `trafficLightPosition` là cấu hình tĩnh (`config.rs:2065`); lề `--space-…` phía CSS và toạ độ trong `tauri.conf.json` là hai nguồn số phải khớp; AD nói "đo trên máy thật" nhưng chưa có cổng nào canh hai số khớp nhau. Low.

### F6 — low: Overlay + chiều cao thanh theo phiên bản OS
- Chính doc Tauri (`tauri-utils-2.9.3/src/lib.rs:169-171`) cảnh báo chiều cao thanh khác nhau theo phiên bản macOS, nên toạ độ đo trên một máy có thể lệch trên bản OS khác. AD-58 ghi "đo trên máy thật" nhưng không ghi giới hạn này trong Rule 5 (Giới hạn chấp nhận). Nên thêm một dòng.

## Không tìm thấy lỗi ở
- Tên quyền, tên lệnh, giá trị `"deep"`, mặc định `acceptFirstMouse=false`, phụ thuộc phiên bản (api 2.11.1 / tauri 2.11.5 / wry runtime 2.11.4 cùng dòng 2.11).
- Dòng 🔵 của AD-48 Rule 2 (spine dòng 770-776): nhất quán với AD-58 Rule 2 và Rule 6; nhánh Rust-only của hộp thoại không bị đụng. Không phải sửa gì ngoài ghi chú ở F2.
- Các con trỏ "mã đọc" cuối AD-58: `config.rs:2060-2068`, `lib.rs:160-180`, `plugin.rs:220-233` đều trúng đoạn nói; riêng `config.rs` nên mở rộng tới `:2071` để bao `acceptFirstMouse`, vì Rule 5 dựa vào nó.

## Ghi chú phương pháp
Chỉ đọc nguồn trong `~/.cargo/registry/src/*/` và repo; không chạy ứng dụng, không đo hành vi thật trên macOS (ví dụ cú bấm đầu khi cửa sổ không focus, chiều cao thanh). Những điểm đó vẫn cần Ice kiểm trong app thật theo AGENTS.md.
