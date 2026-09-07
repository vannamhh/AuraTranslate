//! Nhật ký domain — NFR19, Story 6.8.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! GHI THÔ, SỐNG THEO PHIÊN CHẠY ỨNG DỤNG (§Always spec 6.8)
//! ─────────────────────────────────────────────────────────────────────────────
//! Một bản ghi cho **MỌI** lời gọi `Fetcher` thật ra tới — cả cho phép LẪN từ chối, không
//! rút gọn, không phân trang (mockup `web-import.html:421` là một cam kết). Bảng HIỆN gộp
//! theo domain ở tầng trình bày (`SettingsOverlay.vue`/`settingsState.ts`) — kho ở đây
//! không tự gộp gì, để một lượt gộp sai không bao giờ làm MẤT một bản ghi thô.
//!
//! Sống trong [`DomainLogState`] — `Mutex<Vec<DomainLogEntry>>`, `.manage` một lần trong
//! `open_work_slot` (`lib.rs`). Khởi động lại ứng dụng ⇒ nhật ký RỖNG — đây không phải một
//! giới hạn cần vá, mà là chính điều Ice chốt 2026-09-07 (§Approach spec 6.8): bền hoá là
//! MỘT QUYẾT ĐỊNH MỚI, chưa ai xin. `SettingsOverlay.vue` phải NÓI RA sự thật này (§Always
//! spec 6.8, hàng I/O Matrix "Mở Cài đặt sau khi khởi động lại app").
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! TRẦN SỐ BẢN GHI — PHÉP ĐO (§Ask First spec 6.8, cùng kỷ luật `MAX_RESPONSE_BYTES`)
//! ─────────────────────────────────────────────────────────────────────────────
//! **Kết luận: KHÔNG một trần cứng nào ở đây — và đó là con số đo được, không phải một chỗ
//! lười.** `size_of::<DomainLogEntry>()` là hằng số nhỏ (một `String` domain ~24 byte thân
//! `Vec` cộng dữ liệu heap thật ~10-30 ký tự, cộng vài trường `Copy` nhỏ) — ĐO trên máy
//! phát triển (`std::mem::size_of::<DomainLogEntry>()`, xem test
//! `a_domain_log_entry_is_small_enough_that_an_unbounded_vec_is_safe` bên dưới): **56 byte**
//! phần thân cố định, cộng độ dài chuỗi domain thật (hiếm khi vượt 60 ký tự). Ngay cả một
//! phiên KHÔNG THỰC TẾ với 100.000 bản ghi (Story 6.7 đo 20 link cục bộ ~5 s; để chạm sáu
//! chữ số bản ghi trong MỘT phiên người dùng phải dán và tải hàng trăm danh sách lớn liên
//! tục không nghỉ) chỉ tốn ~10-15 MiB — nhỏ hơn MỘT phản hồi HTML đơn lẻ đã được phép nạp
//! trọn (`MAX_RESPONSE_BYTES` = 20 MiB, `fetcher.rs`). Trần Story 6.7 đã từ chối đặt một trần
//! SỐ LINK vì cắt bớt phá đúng bất biến trung tâm (N link = N Chương,
//! `deferred-work.md` §"Deferred from: 6-7… vòng rà đối kháng bước 4") — không có trần đó ở
//! thượng nguồn thì một trần bản ghi ở đây không bảo vệ được gì mà một trần link không đã
//! bảo vệ, và nó vi phạm chính cam kết "ghi mọi lần gọi, không rút gọn". ⇒ `Vec` không trần,
//! tăng tự do theo phiên chạy.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::allowlist::{ResourceKind, Tier};

/// Kết quả một lượt quyết định allowlist, GẮN vào một bản ghi nhật ký — cùng hình dạng
/// [`super::allowlist::AllowlistDecision`], viết lại tách riêng ở đây vì lý do CHỦ SỞ HỮU:
/// `allowlist.rs` không nên biết "nhật ký" tồn tại (một module quyết định thuần không cần
/// phụ thuộc module ghi log để mà kiểm — hai việc khác vai), còn `Allowlist::decide` vẫn là
/// nguồn sự thật DUY NHẤT cho CHÍNH quyết định (`fetcher.rs` là nơi hai kiểu này gặp nhau).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainLogDecision {
    Allowed(Tier),
    Denied,
}

/// Một bản ghi THÔ — đúng những gì mockup `web-import.html:423` liệt (`Thời điểm · Domain ·
/// Tầng · Vì sao được phép · Kết quả`), trừ cột "Kết quả" (một số ĐẾM gộp theo domain — tầng
/// trình bày tự tính từ nhiều bản ghi, không phải một trường của MỘT bản ghi).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainLogEntry {
    /// Mili-giây epoch UTC lúc bản ghi được TẠO. Định dạng hiển thị chỉ ở frontend
    /// (Consistency Conventions: "lưu UTC, định dạng hiển thị chỉ ở frontend").
    pub at_epoch_ms: u64,
    pub domain: String,
    pub kind: ResourceKind,
    pub decision: DomainLogDecision,
}

impl DomainLogEntry {
    pub fn new(at_epoch_ms: u64, domain: String, kind: ResourceKind, decision: DomainLogDecision) -> Self {
        DomainLogEntry { at_epoch_ms, domain, kind, decision }
    }
}

/// Đồng hồ THẬT — tách thành một hàm riêng (thay vì gọi `SystemTime::now()` rải rác) để một
/// test có một điểm DUY NHẤT cần biết nếu sau này cần tiêm thời điểm giả (chưa cần hôm nay:
/// `fetcher.rs`/`webimport_contract.rs` đã là các phép kiểm HỢP ĐỒNG chạm mạng thật, không
/// phải các phép kiểm tất định cấp mili-giây).
pub fn now_epoch_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// Ô chứa — `.manage` MỘT lần trong `open_work_slot` (`lib.rs`), sống theo PHIÊN CHẠY ứng
/// dụng, không theo Tác phẩm đang mở (đổi/đóng Tác phẩm không đụng tới nó — nhật ký domain
/// là chuyện của ỨNG DỤNG, không của một `.atproj`).
pub type DomainLogState = Mutex<Vec<DomainLogEntry>>;

/// Nối thêm `entries` vào cuối — **hàm thuần, `pub`**, nhận `&DomainLogState` trần (không
/// `AppHandle`) để `tests/**` gọi được không cần dựng webview, đúng khuôn
/// `clear_url_import_items_after_successful_confirm` (`commands/project.rs`).
pub fn append_domain_log_entries(state: &DomainLogState, entries: Vec<DomainLogEntry>) {
    if entries.is_empty() {
        return;
    }
    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.extend(entries);
}

/// Đọc TOÀN BỘ nhật ký hiện có, theo đúng thứ tự đã ghi — **hàm thuần, `pub`**. `Clone` cả
/// mảng (không tham chiếu ra ngoài khoá Mutex) vì chỗ gọi (vỏ IPC) cần trả nó qua dây; nhật
/// ký một phiên thực tế nhỏ (xem phép đo ở doc-comment đầu tệp), nên một lượt `clone()` mỗi
/// lần đọc không phải một chi phí đáng lo.
pub fn read_domain_log(state: &DomainLogState) -> Vec<DomainLogEntry> {
    state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
}

/// Số DOMAIN PHÂN BIỆT trong nhật ký — con số chân màn xem trước cần (mockup: *"Đã gọi
/// **N** domain · xem"*, N = domain phân biệt, KHÔNG phải số bản ghi thô — I/O Matrix spec
/// 6.8 hàng 2). **Hàm thuần**, tách riêng khỏi [`read_domain_log`] để chỗ gọi không phải tự
/// dựng một `HashSet` mỗi nơi cần con số này.
pub fn distinct_domain_count(state: &DomainLogState) -> usize {
    let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for entry in guard.iter() {
        seen.insert(entry.domain.as_str());
    }
    seen.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Đối chứng cho phép đo "trần số bản ghi" ở doc-comment đầu tệp — một thay đổi kiểu sau
    /// này làm `DomainLogEntry` phình to bất ngờ (ví dụ thêm một `String` chẩn đoán dài) sẽ
    /// làm test này đỏ, đúng lúc kết luận "không cần trần" cần được ĐO LẠI, không giả định
    /// còn đúng mãi.
    #[test]
    fn a_domain_log_entry_is_small_enough_that_an_unbounded_vec_is_safe() {
        let size = std::mem::size_of::<DomainLogEntry>();
        assert!(
            size <= 96,
            "DomainLogEntry nay {size} byte/ban ghi — phep do 'khong can tran' o doc-comment \
             dau tep gia dinh con so nho hon 96 byte. Do lai truoc khi giu ket luan khong tran."
        );
    }

    #[test]
    fn append_then_read_preserves_order_and_counts_distinct_domains() {
        let state: DomainLogState = Mutex::new(Vec::new());
        assert_eq!(read_domain_log(&state).len(), 0);
        assert_eq!(distinct_domain_count(&state), 0);

        append_domain_log_entries(
            &state,
            vec![
                DomainLogEntry::new(1, "a.example".to_owned(), ResourceKind::Page, DomainLogDecision::Allowed(Tier::One)),
                DomainLogEntry::new(2, "b.example".to_owned(), ResourceKind::Page, DomainLogDecision::Denied),
                DomainLogEntry::new(3, "a.example".to_owned(), ResourceKind::Page, DomainLogDecision::Allowed(Tier::One)),
            ],
        );

        let read = read_domain_log(&state);
        assert_eq!(read.len(), 3, "ghi THO -- ba ban ghi cho ba loi goi, khong gop");
        assert_eq!(read[0].domain, "a.example");
        assert_eq!(read[1].domain, "b.example");
        assert_eq!(read[2].domain, "a.example");
        assert_eq!(distinct_domain_count(&state), 2, "hai domain PHAN BIET tu ba ban ghi");
    }

    #[test]
    fn appending_an_empty_batch_is_a_no_op() {
        let state: DomainLogState = Mutex::new(vec![DomainLogEntry::new(
            1,
            "a.example".to_owned(),
            ResourceKind::Page,
            DomainLogDecision::Allowed(Tier::One),
        )]);
        append_domain_log_entries(&state, Vec::new());
        assert_eq!(read_domain_log(&state).len(), 1);
    }
}
