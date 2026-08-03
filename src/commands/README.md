`CommandRegistry` của **giao diện** — MỌI thao tác người dùng đăng ký ở đây rồi mới bind vào chuột/phím (AD-34, FR22). Không thao tác nào chỉ tồn tại trong một handler chuột.

Command id dùng khoá chấm có tiền tố miền: `lookup.search_selection`.

**Story sở hữu nội dung: 1.6.**

---

⚠️ **Đừng nhầm với `src-tauri/src/commands/`.** Hai thư mục cùng tên, hai thứ hoàn toàn khác nhau:

| Đường dẫn | Là gì | AD |
|---|---|---|
| `src-tauri/src/commands/` | **Bề mặt IPC** — hàm `#[tauri::command]` mà frontend gọi qua. Adapter thuần, không chứa quy tắc nghiệp vụ | AD-1 |
| `src/commands/` *(thư mục này)* | **`CommandRegistry` của giao diện** — nơi đăng ký thao tác người dùng | AD-34, FR22 |

Hai thứ này **không** ánh xạ một-một và **không** được gộp.
