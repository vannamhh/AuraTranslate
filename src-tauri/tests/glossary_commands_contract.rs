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
    glossary_add_term, glossary_approve_candidate, glossary_confirm_pending_translation,
    glossary_lookup_term, glossary_pending_candidates, glossary_update_term,
};
use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::core::glossary::scan::ScanCandidate;
use auratranslate_lib::core::glossary::{Category, GlossaryTier, insert_import_scan_candidates};
use auratranslate_lib::core::i18n::MessageKey;
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
    assert!(
        found.work_tier_available,
        "co mot Tac pham dang mo qua OpenWork that"
    );
    let entry = found.entry.expect("phai tim thay muc vua them");
    assert_eq!(entry.tier, "work", "muc phai o tang Tac pham");
    assert_eq!(entry.id, id);
    assert_eq!(entry.translation.as_deref(), Some("Mộ Dung"));

    // Doi chung: global.db KHONG bi dung toi -- tra mot cum khong ton tai o tang Global
    // (dung `ScopeResolver::global_only`, khong qua OpenWork) phai RONG.
    let global_only =
        glossary_lookup_term(Some(&global), None, "慕容").expect("tra chi tang global");
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

    insert_import_scan_candidates(
        &opened.store,
        &[ScanCandidate {
            source_term: "萧炎".to_owned(),
            occurrence_count: 37,
            context_example: "萧炎在乌坦城第一次登场。".to_owned(),
        }],
    )
    .expect("chen ung vien quet co count/context khac mac dinh");

    let rows = glossary_pending_candidates(Some(&opened))
        .expect("liet ke bang cho qua commands::glossary");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].source_term, "萧炎");
    assert_eq!(rows[0].candidate_origin, "import_scan");
    assert_eq!(rows[0].resolution, None);
    assert_eq!(rows[0].occurrence_count, 37);
    assert_eq!(
        rows[0].context_example.as_deref(),
        Some("萧炎在乌坦城第一次登场。")
    );

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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.6 — CHỐT trạng thái chờ chốt (FR114), qua bề mặt IPC thật
// ═════════════════════════════════════════════════════════════════════════════════

/// Chốt lần đầu qua bề mặt IPC thật: mục *chờ chốt* ở tầng Tác phẩm ghi được `translation`,
/// và tra lại qua `glossary_lookup_term` (không đọc thẳng SQL) thấy đúng bản dịch mới.
#[test]
fn glossary_confirm_pending_translation_writes_through_a_real_open_work() {
    let root = temp_dir("confirm-work-tier");
    let global_dir = temp_dir("confirm-work-tier-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Cho Chot");

    let id = glossary_add_term(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        "萧炎",
        None,
        "",
        Category::Person,
    )
    .expect("them muc cho chot o tang Tac pham");

    glossary_confirm_pending_translation(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        id,
        "Tieu Viem",
    )
    .expect("chot ban dich qua commands::glossary");

    let found = glossary_lookup_term(Some(&global), Some(&opened), "萧炎")
        .expect("tra lai qua commands::glossary")
        .entry
        .expect("phai con tim thay");
    assert_eq!(found.translation.as_deref(), Some("Tieu Viem"));

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// `tier = Work` mà chưa mở Tác phẩm nào ⇒ `glossary.work_tier_unavailable`, không panic,
/// không `Ok` giả.
#[test]
fn glossary_confirm_pending_translation_at_the_work_tier_without_an_open_work_fails() {
    let global_dir = temp_dir("confirm-no-work-global");
    let global = open_global(&global_dir);

    let err = glossary_confirm_pending_translation(
        Some(&global),
        None,
        GlossaryTier::Work,
        1,
        "ban dich",
    )
    .expect_err("khong the chot o tang Tac pham khi chua mo Tac pham nao");
    assert_eq!(err.message_key(), MessageKey::GlossaryWorkTierUnavailable);

    drop(global);
    cleanup(&global_dir);
}

/// `id` không khớp hàng nào ⇒ lỗi mang `message_key` đọc được, không `Ok` cho một lượt ghi
/// 0 hàng (§I/O Matrix nền tảng của `confirm_translation`, giờ đóng qua bề mặt IPC thật).
#[test]
fn glossary_confirm_pending_translation_with_an_unknown_id_fails_readably() {
    let global_dir = temp_dir("confirm-unknown-id-global");
    let global = open_global(&global_dir);

    let err = glossary_confirm_pending_translation(
        Some(&global),
        None,
        GlossaryTier::Global,
        999_999,
        "ban dich",
    )
    .expect_err("id khong ton tai phai la loi, khong phai Ok rong");
    assert_eq!(err.message_key(), MessageKey::StoreWriteFailed);

    drop(global);
    cleanup(&global_dir);
}

/// 🔵 THÊM 2026-08-22 (rà ba lớp) — `translation` rỗng/khoảng trắng ⇒ `store.write_failed`
/// (`CHECK` của `GLOSSARY_ENTRY_DDL`), đóng đúng nhánh doc-comment của
/// [`glossary_confirm_pending_translation`] đã KHAI ("`translation` rỗng/khoảng trắng ⇒
/// `store.write_failed` (CHECK)") nhưng chưa ca nào đi qua. Frontend chặn chuỗi rỗng TRƯỚC
/// IPC (`glossaryConfirmStripState.ts::confirmGlossaryConfirmStrip`) — đây là lớp phòng thủ
/// THỨ HAI, và nó phải có ca canh vì doc đã hứa.
#[test]
fn glossary_confirm_pending_translation_with_a_blank_translation_fails_and_leaves_the_row_pending() {
    let root = temp_dir("confirm-blank-translation");
    let global_dir = temp_dir("confirm-blank-translation-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Chuoi Trang");

    let id = glossary_add_term(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        "萧炎",
        None,
        "",
        Category::Person,
    )
    .expect("them muc cho chot o tang Tac pham");

    let err = glossary_confirm_pending_translation(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        id,
        "   ",
    )
    .expect_err("chuoi khoang trang phai la loi, khong phai Ok");
    assert_eq!(err.message_key(), MessageKey::StoreWriteFailed);

    // Doi chung BANG SELECT: hang van CHO CHOT, khong bi ghi nua vi (CHECK tu choi truoc khi
    // UPDATE ap dung).
    let found = glossary_lookup_term(Some(&global), Some(&opened), "萧炎")
        .expect("tra lai qua commands::glossary")
        .entry
        .expect("phai con tim thay muc cu");
    assert_eq!(found.id, id);
    assert_eq!(found.translation, None, "muc phai VAN cho chot -- CHECK da chan luot ghi");

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.6 — NHẬN một ứng viên (approve_candidate), qua bề mặt IPC thật
// ═════════════════════════════════════════════════════════════════════════════════

/// Nhận một ứng viên KHÔNG có đề xuất (`translation = None`) ⇒ một mục Glossary MỚI,
/// `translation IS NULL` (chờ chốt), `term_origin` suy từ `candidate_origin` của hàng ứng
/// viên (`import_scan`) — đóng đúng hàng I/O Matrix *"Nhận một ứng viên không có đề xuất"*.
#[test]
fn glossary_approve_candidate_without_a_suggestion_creates_a_pending_entry() {
    let root = temp_dir("approve-no-suggestion");
    let global_dir = temp_dir("approve-no-suggestion-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Nhan Ung Vien");

    insert_import_scan_candidates(
        &opened.store,
        &[ScanCandidate {
            source_term: "夜幕城".to_owned(),
            occurrence_count: 12,
            context_example: "vi du".to_owned(),
        }],
    )
    .expect("chen ung vien quet");
    let candidate_id = glossary_pending_candidates(Some(&opened))
        .expect("liet ke bang cho")
        .into_iter()
        .find(|c| c.source_term == "夜幕城")
        .expect("phai tim thay ung vien vua chen")
        .id;

    let entry_id = glossary_approve_candidate(Some(&opened), candidate_id, None, Category::Place)
        .expect("nhan ung vien qua commands::glossary");
    assert!(entry_id > 0);

    // Hang ung vien da quyet -- khong con trong bang CHO DUYET.
    let still_pending = glossary_pending_candidates(Some(&opened)).expect("liet ke lai");
    assert!(
        still_pending.iter().all(|c| c.id != candidate_id),
        "ung vien da duyet khong con trong bang cho DUYET"
    );

    // Muc Glossary moi phai o trang thai CHO CHOT.
    let found = glossary_lookup_term(Some(&global), Some(&opened), "夜幕城")
        .expect("tra lai qua commands::glossary")
        .entry
        .expect("phai tim thay muc vua sinh");
    assert_eq!(found.id, entry_id);
    assert_eq!(found.translation, None, "khong dua xuat -- muc phai CHO CHOT");
    assert_eq!(found.term_origin, "import_scan");

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// `id` không khớp hàng ứng viên nào ⇒ lỗi mang `message_key`, không `Ok` rỗng — đúng hàng
/// I/O Matrix.
#[test]
fn glossary_approve_candidate_with_an_unknown_id_fails_readably() {
    let root = temp_dir("approve-unknown-id");
    let opened = open_work(&root, "Id La");

    let err = glossary_approve_candidate(Some(&opened), 999_999, None, Category::Other)
        .expect_err("id khong ton tai phai la loi, khong phai Ok rong");
    assert_eq!(err.message_key(), MessageKey::StoreWriteFailed);

    drop(opened);
    cleanup(&root);
}

/// Nhận lại một ứng viên ĐÃ quyết (đã duyệt) ⇒ **0** mục Glossary mới, **0** cột đổi ở hàng
/// ứng viên cũ — hai bảng không được phép nói ngược nhau (§I/O Matrix).
#[test]
fn glossary_approve_candidate_on_an_already_decided_candidate_changes_nothing() {
    let root = temp_dir("approve-already-decided");
    let global_dir = temp_dir("approve-already-decided-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Da Quyet");

    insert_import_scan_candidates(
        &opened.store,
        &[ScanCandidate {
            source_term: "青丘".to_owned(),
            occurrence_count: 9,
            context_example: "vi du".to_owned(),
        }],
    )
    .expect("chen ung vien quet");
    let candidate_id = glossary_pending_candidates(Some(&opened))
        .expect("liet ke bang cho")
        .into_iter()
        .find(|c| c.source_term == "青丘")
        .expect("phai tim thay ung vien vua chen")
        .id;

    glossary_approve_candidate(Some(&opened), candidate_id, Some("Thanh Khau"), Category::Place)
        .expect("duyet lan dau phai thanh cong");

    // Doi chung SO LUONG muc Glossary TRUOC/SAU lan goi thu hai -- "0 muc moi" nghia la
    // dung MOT muc trong ca hai lan tra, khong phai suy tu Err.
    let before_second_call = glossary_lookup_term(Some(&global), Some(&opened), "青丘")
        .expect("tra lai lan dau")
        .entry
        .expect("phai tim thay muc da duyet");

    let err =
        glossary_approve_candidate(Some(&opened), candidate_id, Some("Ban Dich Khac"), Category::Other)
            .expect_err("ung vien da quyet khong duoc quyet lai");
    assert_eq!(err.message_key(), MessageKey::StoreWriteFailed);

    let after_second_call = glossary_lookup_term(Some(&global), Some(&opened), "青丘")
        .expect("tra lai lan hai")
        .entry
        .expect("muc cu phai con nguyen");
    assert_eq!(
        before_second_call.id, after_second_call.id,
        "khong co muc Glossary thu hai nao duoc sinh ra"
    );
    assert_eq!(
        after_second_call.translation.as_deref(),
        Some("Thanh Khau"),
        "muc cu KHONG bi doi boi lan goi thu hai bi tu choi"
    );
    assert_eq!(
        after_second_call.category, "place",
        "category cua muc cu cung KHONG bi doi"
    );

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// Chưa có Tác phẩm nào đang mở ⇒ lỗi đọc được (bảng chờ chỉ tồn tại ở `project.db`), không
/// panic, không `Ok` giả cho một `id` không có kho nào chứa nó.
#[test]
fn glossary_approve_candidate_without_an_open_work_fails_readably() {
    let err = glossary_approve_candidate(None, 1, None, Category::Other)
        .expect_err("khong the nhan ung vien khi chua mo Tac pham nao");
    assert_eq!(err.message_key(), MessageKey::ProjectNoWorkOpen);
}
