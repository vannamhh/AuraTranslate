//! Hình dạng `.atproj/` trên đĩa — AD-9, Story 1.15, AC2/AC5/AC6.
//!
//! ```text
//! <Tên>.atproj/
//! ├── meta.json      # xem super::meta
//! ├── project.db     # xem crate::core::store — StoreSpec::project
//! └── assets/        # ảnh là TỆP THẬT (Epic 6) — tồn tại kể cả khi rỗng
//! ```
//!
//! ⚠️ [`crate::core::store::Store::open`] **không tự tạo thư mục cha** — cờ
//! `SQLITE_OPEN_CREATE` tạo được một *tệp*, không phải một *thư mục*
//! (`core/store/pragmas.rs::open_connection`). Dựng thư mục là việc của module này, và nó
//! phải chạy **trước** [`crate::core::store::Store::open`].
//!
//! 🔴 **Không đường dẫn tuyệt đối nào được ghi vào bên trong** `.atproj/` (AC5) — module
//! này chỉ trả về đường dẫn đã tạo, không ghi nó vào đâu cả; chỗ gọi (`commands::project`)
//! chịu trách nhiệm không rò rỉ đường dẫn máy cũ vào `meta.json`/`project.db`.

use std::path::{Path, PathBuf};

use super::ProjectError;

/// Thư mục con chứa ảnh — Epic 6 ghi vào đây; story này chỉ đảm bảo nó tồn tại.
const ASSETS_DIR: &str = "assets";

/// Đuôi thư mục của một Tác phẩm — **ngoại lệ lịch sử**, không kéo theo tên thực thể
/// (`ARCHITECTURE-SPINE.md:642` — thực thể là `Work`, không phải `Project`).
const WORK_FOLDER_SUFFIX: &str = ".atproj";

/// Tên hồi phòng khi tên Tác phẩm rút gọn thành rỗng (mọi ký tự đều bị cấm/khoảng trắng).
const FALLBACK_NAME: &str = "Untitled";

/// Trần độ dài phần tên (chưa gồm `.atproj`) tính bằng **BYTE**, không phải ký tự.
///
/// 🔴 Vì sao byte: giới hạn của hệ tệp là byte, không phải ký tự — `NAME_MAX` trên
/// ext4/APFS là **255 byte**, và một ký tự tiếng Việt/tiếng Trung ăn 3 byte trong UTF-8.
/// Một tên 100 ký tự tiếng Việt = 300 byte ⇒ `ENAMETOOLONG`, và người dùng chỉ thấy
/// "khong tao duoc" mà không hiểu vì sao. 180 chừa chỗ cho `.atproj` (7) + hậu tố
/// ` (999)` (6) + biên an toàn cho các hệ tệp keo kiệt hơn.
///
/// ⚠️ Đây là một con số **TẠM, chưa được đo** — chưa story nào khảo sát giới hạn thật
/// trên NTFS/APFS/ext4 với đường dẫn lồng sâu (Windows còn có trần `MAX_PATH` 260 ký tự
/// cho **cả đường dẫn**, không riêng tên thư mục). Story nào chạm giới hạn thật sở hữu
/// việc đo lại.
const MAX_FOLDER_NAME_BYTES: usize = 180;

/// Số hậu tố tối đa thử khi tên đã có thư mục chiếm chỗ — `Tên (2)` … `Tên (999)`.
///
/// ⚠️ Có trần vì vòng lặp này chạm đĩa mỗi vòng: không để một thư mục bệnh hoạn (999
/// Tác phẩm trùng tên) biến một thao tác tạo thành một vòng quét vô hạn.
const MAX_NAME_ATTEMPTS: u32 = 999;

/// Ký tự cấm trên **ít nhất một** trong hai nền tảng — hợp của tập cấm Windows
/// (`< > : " / \ | ? *`) với `/` của Unix. Thay bằng `_`.
///
/// ⚠️ `pragmas.rs:44-53` mở kết nối **không** cờ `URI`, nhưng `?` trong tên thư mục vẫn là
/// một ký tự hợp lệ trên macOS/Linux — vấn đề chỉ nảy sinh nếu ai đó BẬT cờ URI sau này.
/// Cấm nó ở đây thêm một lớp an toàn không tốn gì, và giữ tên thư mục portable qua cả hai
/// nền tảng (NFR14) — một Tác phẩm tạo trên macOS phải copy mở được trên Windows.
const FORBIDDEN_CHARS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Tên thiết bị dành riêng của Windows — tạo một thư mục trùng tên (kể cả có đuôi) thất
/// bại trên NTFS. So sánh không phân biệt hoa/thường.
const WINDOWS_RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Chuẩn hoá tên Tác phẩm thành một tên thư mục hợp lệ trên **cả hai** nền tảng (NFR14).
///
/// Không cắt ký tự Unicode hợp lệ (tên tiếng Việt/tiếng Trung phải giữ nguyên) — chỉ thay
/// đúng tập ký tự cấm, cắt khoảng trắng/dấu chấm cuối (Windows từ chối chúng ở cuối tên),
/// cắt bớt theo trần byte, và thêm hậu tố nếu tên trùng một thiết bị dành riêng của Windows.
///
/// 🔴 Hàm này **không** đảm bảo tên trả về là DUY NHẤT trên đĩa — hai tên khác nhau
/// vẫn có thể rút gọn về cùng một chuỗi (`A/B` và `A_B` đều ra `A_B`). Việc tránh giẫm
/// lên một thư mục đã có là của [`create_work_folder`], không phải của hàm này.
pub fn sanitize_name(name: &str) -> String {
    let mut out: String = name
        .chars()
        .map(|c| {
            if FORBIDDEN_CHARS.contains(&c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();

    // Cắt theo trần BYTE, ở đúng biên ký tự — không `truncate()` trần (nó panic khi
    // cắt giữa một ký tự nhiều byte, và `panic = "abort"` giết cả tiến trình).
    if out.len() > MAX_FOLDER_NAME_BYTES {
        let cut = (0..=MAX_FOLDER_NAME_BYTES)
            .rev()
            .find(|&i| out.is_char_boundary(i))
            .unwrap_or(0);
        out.truncate(cut);
    }

    // Sau khi cắt, đuôi có thể lại là `.`/` ` — nên bước này phải đứng SAU bước cắt.
    while matches!(out.chars().last(), Some('.') | Some(' ')) {
        out.pop();
    }

    if out.trim().is_empty() {
        out = FALLBACK_NAME.to_owned();
    }

    // 🔴 So với **PHẦN GỐC** (trước dấu chấm đầu tiên), không phải nguyên chuỗi:
    // Windows từ chối `CON.txt`, `NUL.md`, `COM1.bat` y hệt như `CON` trần — tên thiết bị
    // được nhận diện theo phần gốc, đuôi không cứu được nó.
    let stem = out.split('.').next().unwrap_or("");
    if WINDOWS_RESERVED
        .iter()
        .any(|reserved| reserved.eq_ignore_ascii_case(stem))
    {
        out.push('_');
    }

    out
}

/// Dựng `<Tên>.atproj/` + `assets/` dưới `root`. Trả về đường dẫn thư mục vừa tạo.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 TẠO ĐỘC QUYỀN, KHÔNG `create_dir_all` TRÊN THƯ MỤC GỐC — VÌ SAO
/// ─────────────────────────────────────────────────────────────────────────────
/// `create_dir_all` **thành công im lặng** khi thư mục đã tồn tại. Kết hợp với đường
/// dọn dẹp của chỗ gọi (`remove_folder` khi một bước sau trượt), nó dựng nên một đường
/// **XOÁ TRẮNG TÁC PHẨM CỦA NGƯỜI DÙNG**: tạo trùng tên ⇒ `Store::open` mở lại
/// `project.db` cũ ⇒ `INSERT ... VALUES (1, …)` đụng `CHECK (id = 1)` ⇒ dọn dẹp
/// `remove_dir_all` cả thư mục không phải do lượt gọi này tạo. Lỗi tìm ra ở lượt code
/// review 2026-08-06 và **đã được chứng minh bằng một lượt chạy thật**, không phải suy
/// luận.
///
/// ⇒ `std::fs::create_dir` (**không** `_all`) trên thư mục gốc là bước tạo độc quyền:
/// nó trả `AlreadyExists` thay vì giẫm lên. Nhờ đó hàm này có một hậu điều kiện mạnh mà
/// chỗ gọi dựa vào được: **đường dẫn trả về LUÔN là một thư mục lượt gọi này vừa tạo**,
/// nên `remove_folder` trên nó không bao giờ xoá dữ liệu có sẵn.
///
/// 🔴 Đây cũng là lý do bước kiểm phải là **tạo độc quyền**, không phải `dir.exists()`:
/// một phép kiểm rồi mới tạo để hở một cửa sổ đua (TOCTOU). `create_dir` hỏi hệ điều hành
/// **một câu duy nhất**, và câu trả lời là nguyên tử.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// TRÙNG TÊN ⇒ TỰ ĐÁNH SỐ — Ice chốt ở lượt code review 2026-08-06
/// ─────────────────────────────────────────────────────────────────────────────
/// `Tên.atproj` → `Tên (2).atproj` → … → `Tên (999).atproj`.
///
/// ⚠️ **Hệ quả đã biết, ghi ra thay vì giấu:** hai Tác phẩm cùng tên hiển thị **giống hệt
/// nhau** trong `meta.json` (`name` giữ nguyên tên người dùng gõ, không mang hậu tố) —
/// chỉ tên thư mục khác nhau. Người dùng **không** được cảnh báo là mình vừa tạo trùng.
/// Đường thay thế *(từ chối bằng một lỗi riêng)* ồn hơn nhưng nói thật hơn; Ice chọn tự
/// đánh số. Epic 5 (lưới Tác phẩm) là chỗ khác biệt này lộ ra với người dùng.
///
/// `assets/` tồn tại **kể cả khi rỗng** — Epic 6 không phải kiểm tra sự tồn tại của nó
/// ở mọi đường ghi ảnh.
///
/// # Lỗi
/// [`ProjectError::CreateFailed`] nếu I/O trượt (quyền, đĩa đầy, tên quá dài) hoặc nếu cả
/// [`MAX_NAME_ATTEMPTS`] hậu tố đều đã bị chiếm.
pub fn create_work_folder(root: &Path, name: &str) -> Result<PathBuf, ProjectError> {
    let base = sanitize_name(name);

    // Thư mục gốc chứa các `.atproj` có thể chưa tồn tại (lần chạy đầu tiên). Bước này
    // KHÔNG phải bước tạo độc quyền — nó tạo thư mục CHA, không phải Tác phẩm.
    std::fs::create_dir_all(root).map_err(|e| ProjectError::CreateFailed {
        detail: format!("create root {}: {e}", root.display()),
    })?;

    for attempt in 1..=MAX_NAME_ATTEMPTS {
        let folder_name = if attempt == 1 {
            format!("{base}{WORK_FOLDER_SUFFIX}")
        } else {
            format!("{base} ({attempt}){WORK_FOLDER_SUFFIX}")
        };
        let dir = root.join(folder_name);

        match std::fs::create_dir(&dir) {
            Ok(()) => {
                // Từ đây trở đi `dir` là THƯ MỤC CỦA LƯỢT GỌI NÀY — dọn nó khi bước
                // `assets/` trượt là đúng, không đụng dữ liệu của ai khác.
                if let Err(e) = std::fs::create_dir(dir.join(ASSETS_DIR)) {
                    remove_folder(&dir);
                    return Err(ProjectError::CreateFailed {
                        detail: format!("create {}/{ASSETS_DIR}: {e}", dir.display()),
                    });
                }
                return Ok(dir);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                return Err(ProjectError::CreateFailed {
                    detail: format!("create {}: {e}", dir.display()),
                });
            }
        }
    }

    Err(ProjectError::CreateFailed {
        detail: format!("all {MAX_NAME_ATTEMPTS} name attempts for {base:?} are taken"),
    })
}

/// Dọn một `.atproj/` nửa vời — AC8: một lỗi giữa chừng ⇒ không thư mục nào còn lại.
///
/// Idempotent và im lặng trên lỗi: đây là đường dọn dẹp sau một lỗi KHÁC, nên một lỗi thứ
/// hai ở chính bước dọn dẹp không có gì thêm để báo — chẩn đoán gốc đã được giữ ở lỗi đầu.
///
/// 🔴 **HỢP ĐỒNG GỌI — KHÔNG PHẢI MỘT LỜI KHUYÊN:** chỉ được gọi trên một thư mục do
/// [`create_work_folder`] **trả về ở chính lượt thao tác đang chạy**. Đây là `remove_dir_all`
/// đệ quy; gọi nó trên một đường dẫn có sẵn là **xoá dữ liệu người dùng**. Hậu điều kiện
/// tạo-độc-quyền của [`create_work_folder`] là thứ làm hợp đồng này giữ được — xem lý do
/// đầy đủ ở doc-comment của hàm đó.
pub fn remove_folder(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}
