# Ghi chú cạnh khối `security` của `tauri.conf.json`

> **Vì sao ghi chú nằm ở tệp riêng.** Story 1.2 định đặt nó thành một field chú thích
> ngay trong `tauri.conf.json`. Không được: `tauri-build` từ chối mọi field lạ
> (*"unknown field `_comment_security`"*) và build gãy. JSON không mang được chú thích,
> nên chú thích sống ở tệp này — **nằm ngay cạnh `tauri.conf.json`** — và được neo bằng
> một phép kiểm chạy được: `tests/config_invariants.rs`. Sửa `security` mà quên đọc đây
> thì test đỏ.

## `asset:` và `http://asset.localhost` KHÔNG phải "nới CSP"

AD-15 cấm **origin từ xa** — CDN, font ngoài, ảnh ngoài. Asset protocol là **giao thức
cục bộ** cho tài nguyên đã nằm trong bản cài; không có một byte nào ra mạng. Story 1.1 đã
nạp năm tệp font qua đúng đường này trên **bản build release** và kiểm chứng được.

⛔ **Đừng gỡ `asset:` khỏi `font-src` / `img-src` để "siết CSP"** — làm vậy là làm hỏng
đường nạp font mà Story 1.4 sẽ dựng trên đó.

⚠️ **`http://asset.localhost` là dạng của Windows.** Bỏ nó đi thì macOS vẫn chạy và
Windows hỏng — đúng loại khác biệt nền tảng mà Story 1.3 tồn tại để bắt.

## `"csp": null` là TẮT CSP

Nó **không** có nghĩa *"dùng mặc định"*. Luôn khai chuỗi tường minh. Đây là chỗ AC4 của
Story 1.2 hỏng im lặng dễ nhất.

## `style-src 'self'` — đã cân, không chép máy móc

Story 1.1 chạy với `style-src 'self' 'unsafe-inline'`. Story 1.2 **hạ xuống `'self'` và
kiểm chứng trên bản build release**: cây frontend thật khác app thăm dò một tệp — Vite
tách `<style scoped>` của SFC ra tệp CSS thật lúc build, còn `:style` binding của Vue ghi
qua CSSOM nên CSP không chặn. Không cần `'unsafe-inline'`.

Nếu một story sau **buộc** phải mở lại `'unsafe-inline'`: ghi vào Completion Notes của
story đó **thứ gì đã bị chặn và ở đâu**, đừng mở vì "cho chắc".

## `assetProtocol.scope` — đúng hai mục, và vì sao không có `$APPDATA`

AD-23 khai ba vùng nhưng **hai cơ chế cưỡng chế khác nhau**:

| Vùng AD-23 | Ai chạm tới | Cưỡng chế bằng |
|---|---|---|
| `$RESOURCE/dict/**` chỉ đọc | Rust mở `.db` | `assetProtocol.scope` — framework cưỡng chế |
| `$RESOURCE/fonts/**` chỉ đọc | frontend nạp `@font-face` | `assetProtocol.scope` — framework cưỡng chế |
| `$APPDATA/**` đọc + ghi | **chỉ Rust** | kỷ luật mã Rust + AD-7, AD-11 — **nghiệm thu bằng vắng mặt** |

⛔ **Không đưa `$APPDATA` vào `assetProtocol.scope`.** Frontend không có việc gì với
`global.db` hay `library-index.db` (AD-1, AD-11).

**Ba điều phải hiểu đúng, nếu không sẽ báo cáo sai:**

1. `assetProtocol.scope` là hàng rào **thật**, do framework cưỡng chế. Đường dẫn ngoài
   scope bị webview từ chối nạp (*"asset protocol not configured to allow the path"*).
2. Asset protocol **chỉ đọc**. Không có đường ghi qua nó. Đó là lý do nó khớp trọn vẹn
   hai vùng `$RESOURCE/**` và **không** khớp `$APPDATA/**`.
3. Capabilities của Tauri canh **bề mặt IPC — tức webview**, không canh Rust. Mã Rust gọi
   `std::fs` hay `rusqlite::Connection::open` **không** đi qua capabilities. Phát biểu
   *"chỉ ba vùng này chạm tới"* đúng, nhưng nó đúng nhờ **vắng mặt bề mặt** (không plugin
   `fs`, không `dialog`, không `sql`), **không** nhờ một dòng khai báo.

⛔ **Đừng viết vào báo cáo rằng "framework đã cưỡng chế mọi truy cập file".** Viết đúng
ba dòng của bảng trên. Tiền lệ AD-41: *giấu chỗ yếu mới là chỗ nguy hiểm*.

## Vì sao không có plugin `fs`

*"Một Tauri plugin tồn tại để **phơi API ra JavaScript**, đúng thứ NFR11 cấm"* (AD-29).

Áp cho `fs`: plugin `fs` phơi **API hệ thống file** ra JavaScript. Nhưng AD-1 nói frontend
**chỉ render và giữ state UI** — nó không có việc gì với hệ thống file. Cài plugin `fs`
rồi thu hẹp scope là **tự tạo một bề mặt tấn công rồi rào lại**; không cài nó là **không
có bề mặt để rào**.

Cùng lý do đó đã loại `tauri-plugin-sql` (dùng `rusqlite` trực tiếp, AD-11),
`tauri-plugin-keyring` (dùng crate `keyring` trực tiếp, AD-29), `tauri-plugin-dialog`.
Ice chốt 2026-08-03. `scripts/check-deps.sh` cưỡng chế bằng lệnh.
