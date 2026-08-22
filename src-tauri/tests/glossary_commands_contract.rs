//! Bề mặt `commands::glossary` gọi với một `OpenWork` THẬT — Story 3.3 (Ice bắt, 2026-08-20).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO TỆP NÀY BẮT BUỘC PHẢI TỒN TẠI
//! ─────────────────────────────────────────────────────────────────────────────
//! `glossary_boundary.rs` chỉ so CHUỖI CON trên cây nguồn — nó chứng minh
//! `commands::glossary` GỌI ba hàm mới, không chứng minh chúng CHẠY ĐÚNG với dữ liệu tầng
//! Tác phẩm thật. `glossary_contract.rs` gọi thẳng `core::glossary::store::*` với `&Store`
//! dựng tay — nó không đi qua `commands::glossary::work_context`, tức không chạm đường mà
//! `OpenWork.scope` thật sự được đọc. Trước tệp này: `grep "OpenWork {" src-tauri/tests/
//! *.rs` = 0, và `work_context()` có thể thoái hoá thành luôn trả `None` mà TOÀN BỘ 68 ca
//! Rust còn lại (kể cả `glossary_boundary.rs`) vẫn xanh — `work_tier_available` sẽ luôn
//! `false`, mọi yêu cầu tầng Tác phẩm sẽ trượt, và AD-18 (tầng Tác phẩm thắng) sẽ không
//! bao giờ chạy qua bề mặt IPC thật. Đây đúng là mệnh đề mà Story 3.3 dùng để đóng
//! `deferred-work.md:603` — "không đánh dấu đạt bằng suy luận" (`AGENTS.md`).
//!
//! Dựng `OpenWork` qua `create_work_from_text` — đúng khuôn `project_contract.rs` (dùng lại
//! ~18 lần trong tệp đó), không phải một cách dựng riêng của tệp này.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — thừa kế nguyên vẹn từ `glossary_contract.rs`/`project_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. Mỗi ca một thư mục tạm riêng (pid + `AtomicU64`).
//! 2. Drop `OpenWork`/`Store` TRƯỚC khi xoá thư mục — Windows từ chối xoá tệp đang mở.
//! 3. Không `sleep` dài.
//! 4. Không ca nào treo khi nó trượt.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::glossary::{
    glossary_add_term, glossary_lookup_term, glossary_pending_candidates, glossary_update_term,
};
use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::core::glossary::{CandidateOrigin, Category, GlossaryTier, insert_candidate};
use auratranslate_lib::core::store::{Store, StoreSpec};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-glossary-cmds-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn open_global(dir: &Path) -> Store {
    Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db")
}

/// Đúng khuôn `project_contract.rs` — `create_work_from_text` là chỗ SẢN PHẨM DUY NHẤT
/// dựng một `OpenWork` với `ScopeResolver::with_work(...)` thật.
fn open_work(root: &Path, tag: &str) -> auratranslate_lib::commands::project::OpenWork {
    create_work_from_text(root, tag, "zh", "", "noi dung mau".to_owned())
        .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"))
}

/// `glossary_add_term` với `tier = Work` ghi vào ĐÚNG `open.store` (`project.db`), KHÔNG
/// đụng `global.db` — qua bề mặt `commands::glossary`, không qua `core::glossary::store`
/// thẳng.
#[test]
fn glossary_add_term_at_the_work_tier_writes_through_a_real_open_work() {
    let root = temp_dir("add-work-tier");
    let global_dir = temp_dir("add-work-tier-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Muc Cong");

    let id = glossary_add_term(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
    )
    .expect("them muc o tang Tac pham qua commands::glossary");

    // Tra lai qua chinh bo mat IPC -- khong doc thang SQL o day, de con duong
    // `work_context` -> `resolve_term_for_quick_add` duoc chay THAT, khong bi bo qua.
    let found = glossary_lookup_term(Some(&global), Some(&opened), "慕容")
        .expect("tra lai qua commands::glossary");
    assert!(found.work_tier_available, "co mot Tac pham dang mo qua OpenWork that");
    let entry = found.entry.expect("phai tim thay muc vua them");
    assert_eq!(entry.tier, "work", "muc phai o tang Tac pham");
    assert_eq!(entry.id, id);
    assert_eq!(entry.translation.as_deref(), Some("Mộ Dung"));

    // Doi chung: global.db KHONG bi dung toi -- tra mot cum khong ton tai o tang Global
    // (dung `ScopeResolver::global_only`, khong qua OpenWork) phai RONG.
    let global_only = glossary_lookup_term(Some(&global), None, "慕容").expect("tra chi tang global");
    assert!(
        global_only.entry.is_none(),
        "muc vua them o tang Tac pham KHONG duoc lot sang global.db"
    );

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// `glossary_lookup_term` qua một `OpenWork` thật trả `work_tier_available == true` — đúng
/// mệnh đề mà story dùng để đóng `deferred-work.md:603`.
#[test]
fn glossary_lookup_term_reports_work_tier_available_when_a_real_work_is_open() {
    let root = temp_dir("lookup-work-available");
    let global_dir = temp_dir("lookup-work-available-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Tra Cuu");

    let with_work = glossary_lookup_term(Some(&global), Some(&opened), "khong ton tai")
        .expect("tra qua commands::glossary voi OpenWork that");
    assert!(with_work.work_tier_available);
    assert!(with_work.entry.is_none());

    let without_work = glossary_lookup_term(Some(&global), None, "khong ton tai")
        .expect("tra qua commands::glossary khong co OpenWork");
    assert!(!without_work.work_tier_available);

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// AD-18 qua bề mặt IPC thật: cùng `source_term` ở CẢ hai tầng ⇒ `glossary_lookup_term`
/// trả về mục **tầng Tác phẩm** — không phải tầng Global, đúng "tầng Tác phẩm thắng theo
/// từng thuật ngữ".
#[test]
fn ad18_the_work_tier_wins_over_global_through_the_real_commands_glossary_surface() {
    let root = temp_dir("ad18-through-commands");
    let global_dir = temp_dir("ad18-through-commands-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "AD18");

    glossary_add_term(
        Some(&global),
        None,
        GlossaryTier::Global,
        "青丘",
        Some("Thanh Khau Toan Cuc"),
        "",
        Category::Place,
    )
    .expect("them muc o tang Global");
    glossary_add_term(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        "青丘",
        Some("Thanh Khau Rieng"),
        "",
        Category::Place,
    )
    .expect("them muc trung ten o tang Tac pham");

    let found = glossary_lookup_term(Some(&global), Some(&opened), "青丘")
        .expect("tra qua commands::glossary")
        .entry
        .expect("phai tim thay");
    assert_eq!(found.tier, "work", "tang Tac pham phai thang theo AD-18");
    assert_eq!(found.translation.as_deref(), Some("Thanh Khau Rieng"));

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// `glossary_update_term` với `tier = Work` sửa ĐÚNG hàng trong `open.store` qua bề mặt
/// `commands::glossary` — không phải một lượt gọi thẳng `core::glossary::store`.
#[test]
fn glossary_update_term_at_the_work_tier_rewrites_the_row_in_a_real_open_work() {
    let root = temp_dir("update-work-tier");
    let global_dir = temp_dir("update-work-tier-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Sua Tang Tac Pham");

    let id = glossary_add_term(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        "慕容",
        None,
        "",
        Category::Person,
    )
    .expect("them muc cho chot o tang Tac pham");

    glossary_update_term(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        id,
        Some("Mộ Dung Đã Chốt"),
        "ghi chu moi",
        Category::Person,
    )
    .expect("sua muc qua commands::glossary");

    let found = glossary_lookup_term(Some(&global), Some(&opened), "慕容")
        .expect("tra lai")
        .entry
        .expect("phai con tim thay");
    assert_eq!(found.translation.as_deref(), Some("Mộ Dung Đã Chốt"));
    assert_eq!(found.note, "ghi chu moi");

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// Story 3.5 — chỗ gọi sản phẩm ĐẦU TIÊN của `core::glossary::pending_candidates`, qua
/// `commands::glossary::glossary_pending_candidates`.
#[test]
fn glossary_pending_candidates_lists_the_pending_queue_of_the_real_open_work() {
    let root = temp_dir("pending-candidates-real-work");
    let opened = open_work(&root, "Bang Cho");

    insert_candidate(&opened.store, "萧炎", CandidateOrigin::ImportScan).expect("chen ung vien");

    let rows = glossary_pending_candidates(Some(&opened)).expect("liet ke bang cho qua commands::glossary");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source_term, "萧炎");
    assert_eq!(rows[0].candidate_origin, "import_scan");
    assert_eq!(rows[0].resolution, None);
    assert_eq!(rows[0].occurrence_count, 0, "insert_candidate khong dat occurrence_count");
    assert_eq!(rows[0].context_example, None);

    drop(opened);
    cleanup(&root);
}

/// Chưa mở Tác phẩm nào ⇒ `Ok(vec![])`, KHÔNG một lỗi — bảng chờ chỉ tồn tại ở `project.db`
/// (§Never/Code Map của story).
#[test]
fn glossary_pending_candidates_is_empty_without_a_lie_when_no_work_is_open() {
    let rows = glossary_pending_candidates(None).expect("khong Tac pham nao van phai Ok");
    assert!(rows.is_empty());
}
