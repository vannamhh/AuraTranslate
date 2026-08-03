//! DANH MỤC `message_key` mà Rust được phép phát ra (AD-21).
//!
//! **KHÔNG chứa văn bản hiển thị.** AD-21 nguyên văn: *"Rust không bao giờ trả về
//! văn bản hiển thị"*. Toàn bộ chuỗi giao diện sống ở `src/i18n/vi.json` và chỉ ở đó
//! (NFR16). Ở đây chỉ có khoá.
//!
//! Vì sao cần một danh mục tập trung: hình dạng lỗi qua IPC là
//! `{ code, message_key, params, retryable }`. Không có danh mục thì mỗi module tự
//! gõ khoá của mình, và một khoá gõ sai chỉ lộ ra khi người dùng gặp đúng lỗi đó —
//! frontend không phân giải được, hiện ra khoá trần hoặc chuỗi rỗng. Cùng hình dạng
//! hỏng im lặng mà `CommandRegistry` (AD-34) tồn tại để chặn.
//!
//! Hình dạng thật của danh mục (enum? hằng? sinh mã từ `vi.json`?) là quyết định của
//! **Story 1.5** — chủ sở hữu NFR16 và AD-21.
