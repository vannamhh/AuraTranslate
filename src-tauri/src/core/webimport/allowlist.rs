//! `Allowlist` mạng hai tầng — AD-41 (spine `:523-544`), Story 6.8.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! KHÔNG TRẠNG THÁI, BẰNG CẤU TẠO — Ice chốt 2026-09-07 (§Design Notes spec 6.8)
//! ─────────────────────────────────────────────────────────────────────────────
//! Dựng TẠI CHỖ từ chính danh sách URL của MỘT lần nhập (`Allowlist::from_urls`), sống
//! đúng thời gian của biến giữ nó (một lời gọi `commands::project::fetch_url_import_item`
//! hoặc `fetch_url_import_items`). Không `.manage`, không `AppHandle`, không vòng đời nào để
//! rò rỉ — "sống đúng một lần nhập" là hệ quả của CẤU TẠO, không một quy tắc phải nhớ giữ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HAI TẦNG, PHÂN BIỆT BẰNG **KIỂU** — không một `if` rời ở chỗ gọi
//! ─────────────────────────────────────────────────────────────────────────────
//! AD-41 spine (bảng `:531-535`) khai đúng một ma trận 2×2 giữa [`ResourceKind`] và tầng:
//!
//! | Kiểu \ Tầng | Tầng 1 (người dùng cấp) | Tầng 2 (dẫn xuất) |
//! |---|---|---|
//! | `Page` (bài viết/tài liệu) | Cho phép | **Từ chối** |
//! | `Image`    | **Từ chối** | Cho phép |
//!
//! Đọc kỹ hàng `Image`/Tầng 1: bảng AD-41 khai tầng 1 *"cho phép tải gì: Tài liệu"* — KHÔNG
//! phải "Tài liệu + Ảnh". Một ảnh nằm CÙNG host với trang tầng 1 vẫn phải đi qua tầng 2 (tầng
//! 2 định nghĩa là *"host của tài nguyên được THAM CHIẾU từ trang đã tải ở tầng 1"* — định
//! nghĩa đó KHÔNG loại trừ trường hợp host trùng với chính tầng 1), không được một nhánh tắt
//! nào ở tầng 1 cấp cho nó. [`Allowlist::decide`] vì thế là một `match` CẠN trên
//! [`ResourceKind`] — thêm một biến thể thứ ba vào kiểu đó BẮT BUỘC trình biên dịch bắt bạn
//! quay lại đúng hàm này, đúng thứ §Always spec 6.8 đòi ("luật này sống trong KIỂU").
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! TẦNG 2 CHƯA CÓ CHỖ GỌI SẢN PHẨM NÀO — dựng máy móc trước, có chủ ý (§Design Notes spec 6.8)
//! ─────────────────────────────────────────────────────────────────────────────
//! [`Allowlist::with_tier2_hosts`] tồn tại để **Story 6.11** (Extractor trả host ảnh tham
//! chiếu) có một chỗ để cắm vào mà không phải sửa ngược chữ ký `fetch`/`Allowlist`. **0** chỗ
//! gọi sản phẩm nào truyền gì khác `[]` vào nó hôm nay — `webimport_boundary.rs` không canh
//! được mệnh đề "tầng 2 chưa ai gọi" bằng máy (đó là một câu về SỐ CHỖ GỌI, không một mẫu chữ
//! tĩnh), nên nó chỉ sống ở đây, kèm phép đo: `grep -rn "with_tier2_hosts" src-tauri/src` cho
//! đúng MỘT chỗ định nghĩa, 0 chỗ gọi ngoài `#[cfg(test)]`/`tests/**` (đo 2026-09-07).

use std::collections::BTreeSet;

/// Loại tài nguyên một lượt `fetch` đang xin — trục THỨ HAI của ma trận AD-41, cạnh [`Tier`].
///
/// ⚠️ **Tên `Page`, không `Document`** — `tests/naming_boundary.rs` (Story 5.1, AGENTS.md:41)
/// cấm từ `Document` làm tên cho một khái niệm bất kỳ trong `src-tauri/src/**`, vì thuật ngữ
/// đó dành RIÊNG cho `Work` (Tác phẩm) và cấm dùng lẫn. Ý nghĩa ở đây — "trang/bài viết vừa
/// tải, đối lập với một ảnh" — không liên quan gì tới `Work`, nên đây là một xung đột TỪ
/// VỰNG tình cờ, không một vi phạm ngữ nghĩa; `Page` diễn đạt đúng ý và không đụng từ cấm.
///
/// 🔴 **`Page` là kiểu DUY NHẤT sản phẩm dùng hôm nay** — chỗ gọi thật
/// (`commands::project::fetch_url_import_item`) luôn truyền `ResourceKind::Page`.
/// `Image` tồn tại để bốn mệnh đề bắt buộc của AD-41 (spine `:542`, *"bộ test riêng"*) phát
/// biểu được, và để Story 6.11 có một biến thể sẵn thay vì phải sửa kiểu này lúc đó.
///
/// `Serialize` (`rename_all = "snake_case")` — trường DUY NHẤT của kiểu này đi qua dây là
/// [`crate::commands::project::DomainLogEntryWire::kind`] (Story 6.8), làm dữ liệu THÔ,
/// không một câu (AD-21) — TS ánh xạ `"page"`/`"image"` sang khoá `vi.json` bằng một hàm
/// THUẦN có nhánh mặc định, cùng khuôn `cleanupTierLabelKey`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Page,
    Image,
}

/// Tầng đã CẤP PHÉP một lời gọi — trả về CÙNG quyết định `Allowed`, để nhật ký domain
/// (`domain_log.rs`) ghi đúng "vì sao được phép" mà không phải suy lại từ tập host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Host của chính một link trong danh sách vừa dán.
    One,
    /// Host dẫn xuất — tham chiếu từ MỘT trang tầng 1 đã tải, trong CÙNG lần nhập.
    Two,
}

/// Kết quả một lượt quyết định — [`Allowlist::decide`] trả kiểu này thay vì `bool`, để chỗ
/// gọi (cả `fetcher.rs` lẫn `domain_log.rs`) không phải hỏi lại "tầng nào" bằng một lượt tra
/// cứu thứ hai.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowlistDecision {
    Allowed(Tier),
    Denied,
}

/// Allowlist một-lần-nhập — xem doc-comment đầu tệp. `Clone` để `fetcher.rs` mang được một
/// bản vào closure của `redirect::Policy::custom` (chạy trên MỘT luồng riêng của
/// `reqwest::blocking`, không mượn được tham chiếu qua biên đó gọn gàng bằng `Arc` một
/// `BTreeSet` nhỏ — hai tầng của một lượt nhập THẬT hiếm khi vượt vài chục host).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Allowlist {
    tier1: BTreeSet<String>,
    tier2: BTreeSet<String>,
}

impl Allowlist {
    /// Dựng tầng 1 từ CHÍNH danh sách URL của một lượt nhập — một dòng không phải URL tuyệt
    /// đối hợp lệ (hoặc không có host, ví dụ `mailto:`) bị BỎ QUA lặng lẽ ở đây: dòng đó đã
    /// tự thành một mục hỏng `InvalidUrl` ở tầng gọi (`commands::project::fetch_url_import_item`)
    /// TRƯỚC khi allowlist này có cơ hội được hỏi tới, nên không có "URL rác nào lọt vào tầng
    /// 1" — chỉ đơn giản là URL đó không đóng góp một host nào cho tập.
    pub fn from_urls<'a>(urls: impl IntoIterator<Item = &'a str>) -> Self {
        let tier1: BTreeSet<String> = urls.into_iter().filter_map(host_of).collect();
        Allowlist { tier1, tier2: BTreeSet::new() }
    }

    /// Thêm host tầng 2 — xem "TẦNG 2 CHƯA CÓ CHỖ GỌI SẢN PHẨM NÀO" ở doc-comment đầu tệp.
    /// Nhận `impl IntoIterator<Item = String>` (không `&str`) để chỗ gọi tương lai (6.11) có
    /// thể truyền thẳng kết quả `Extractor::extract` mà không phải giữ một tham chiếu sống.
    #[must_use]
    pub fn with_tier2_hosts(mut self, hosts: impl IntoIterator<Item = String>) -> Self {
        self.tier2.extend(hosts);
        self
    }

    /// Quyết định DUY NHẤT của cả module — ma trận 2×2, xem doc-comment đầu tệp. `host` so
    /// bằng CHUỖI, không phân biệt hoa/thường (`Url::host_str()` của `url`/`reqwest` đã hạ
    /// chữ thường IDNA-hoá host lúc `parse`, nên không cần một lượt `to_ascii_lowercase()`
    /// thứ hai ở đây — trùng lặp đó chỉ có ý nghĩa nếu một trong hai nguồn KHÔNG đi qua
    /// `Url::parse`, và cả `from_urls` lẫn `fetcher.rs` đều đi qua).
    pub fn decide(&self, host: &str, kind: ResourceKind) -> AllowlistDecision {
        match kind {
            ResourceKind::Page => {
                if self.tier1.contains(host) {
                    AllowlistDecision::Allowed(Tier::One)
                } else {
                    AllowlistDecision::Denied
                }
            }
            ResourceKind::Image => {
                if self.tier2.contains(host) {
                    AllowlistDecision::Allowed(Tier::Two)
                } else {
                    AllowlistDecision::Denied
                }
            }
        }
    }
}

fn host_of(url: &str) -> Option<String> {
    reqwest::Url::parse(url).ok().and_then(|u| u.host_str().map(str::to_owned))
}
