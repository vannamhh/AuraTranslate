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
    glossary_delete_term, glossary_list_entries, glossary_lookup_term,
    glossary_pending_candidates, glossary_promote_term_to_global, glossary_reject_candidate,
    glossary_update_term,
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
    let layers = auratranslate_lib::core::dict::DictLayers::empty();
    let disabled = std::collections::BTreeSet::new();
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

    let rows = glossary_pending_candidates(Some(&opened), &layers, &disabled)
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
    let layers = auratranslate_lib::core::dict::DictLayers::empty();
    let disabled = std::collections::BTreeSet::new();
    let rows = glossary_pending_candidates(None, &layers, &disabled).expect("khong Tac pham nao van phai Ok");
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
    let layers = auratranslate_lib::core::dict::DictLayers::empty();
    let disabled = std::collections::BTreeSet::new();
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
    let candidate_id = glossary_pending_candidates(Some(&opened), &layers, &disabled)
        .expect("liet ke bang cho")
        .into_iter()
        .find(|c| c.source_term == "夜幕城")
        .expect("phai tim thay ung vien vua chen")
        .id;

    let entry_id = glossary_approve_candidate(Some(&opened), candidate_id, None, Category::Place)
        .expect("nhan ung vien qua commands::glossary");
    assert!(entry_id > 0);

    // Hang ung vien da quyet -- khong con trong bang CHO DUYET.
    let still_pending = glossary_pending_candidates(Some(&opened), &layers, &disabled).expect("liet ke lai");
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

/// Story 3.7 — Nhận một ứng viên CÓ đề xuất (`translation = Some("Bac Luong")`, đúng chữ ký
/// §I/O Matrix *"`glossary_approve_candidate(id, "Bắc Lương", category)`"*) ⇒ mục Glossary
/// MỚI mang `translation IS NOT NULL` (**đã chốt**), và hàng ứng viên cũ `resolution =
/// 'approved'` — đối chứng bằng `SELECT` lại qua `glossary_lookup_term`/`glossary_pending_
/// candidates`, không bằng suy luận (`glossary_approve_candidate` KHÔNG tự tính đề xuất — nó
/// nhận `translation` từ CHỖ GỌI, đúng §Always của story: "máy chỉ ĐỀ XUẤT", không tự ghi).
#[test]
fn glossary_approve_candidate_with_a_suggestion_creates_a_confirmed_entry() {
    let layers = auratranslate_lib::core::dict::DictLayers::empty();
    let disabled = std::collections::BTreeSet::new();
    let root = temp_dir("approve-with-suggestion");
    let global_dir = temp_dir("approve-with-suggestion-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Nhan Ung Vien Co De Xuat");

    insert_import_scan_candidates(
        &opened.store,
        &[ScanCandidate {
            source_term: "北涼".to_owned(),
            occurrence_count: 5,
            context_example: "vi du".to_owned(),
        }],
    )
    .expect("chen ung vien quet");
    let candidate_id = glossary_pending_candidates(Some(&opened), &layers, &disabled)
        .expect("liet ke bang cho")
        .into_iter()
        .find(|c| c.source_term == "北涼")
        .expect("phai tim thay ung vien vua chen")
        .id;

    let entry_id =
        glossary_approve_candidate(Some(&opened), candidate_id, Some("Bac Luong"), Category::Place)
            .expect("nhan ung vien co de xuat qua commands::glossary");
    assert!(entry_id > 0);

    // Hang ung vien: resolution = 'approved', khong con trong bang CHO DUYET.
    let still_pending = glossary_pending_candidates(Some(&opened), &layers, &disabled).expect("liet ke lai");
    assert!(
        still_pending.iter().all(|c| c.id != candidate_id),
        "ung vien da duyet (co de xuat) khong con trong bang cho DUYET"
    );

    // Muc Glossary moi: translation KHAC NULL -- DA CHOT, khong CHO CHOT nhu ca khong de xuat.
    let found = glossary_lookup_term(Some(&global), Some(&opened), "北涼")
        .expect("tra lai qua commands::glossary")
        .entry
        .expect("phai tim thay muc vua sinh");
    assert_eq!(found.id, entry_id);
    assert_eq!(
        found.translation.as_deref(),
        Some("Bac Luong"),
        "de xuat da duoc CHO GOI truyen vao -- muc phai DA CHOT ngay luc sinh"
    );
    assert_eq!(found.term_origin, "import_scan");

    // ─────────────────────────────────────────────────────────────────────────────
    // Hang I/O *"Sua mot muc da vao tu de xuat"* -- muc sinh tu de xuat KHONG duoc
    // mang mot duong khoa rieng nao. Kiem NGAY TAI DAY thay vi mot ca rieng: mot ca
    // rieng phai dung lai toan bo luot nhan o tren, va tien de no canh la *"muc VUA
    // sinh tu de xuat"* -- tach ra la danh mat chinh tien de do.
    // ⚠️ `tier = Work`: `approve_candidate` ghi vao `project.db` (bang cho ung vien
    // chi ton tai o do), nen tang cua muc vua sinh KHONG phai `Global`.
    glossary_update_term(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        entry_id,
        Some("Bac Luong (sua tay)"),
        "sua sau khi nhan tu de xuat",
        Category::Place,
    )
    .expect("sua mot muc sinh tu de xuat -- phai di duong y het moi muc khac");

    let edited = glossary_lookup_term(Some(&global), Some(&opened), "北涼")
        .expect("tra lai sau khi sua")
        .entry
        .expect("muc phai con do");
    assert_eq!(edited.id, entry_id, "sua tai cho, khong sinh hang moi");
    assert_eq!(
        edited.translation.as_deref(),
        Some("Bac Luong (sua tay)"),
        "ban dich de xuat phai bi de len -- de xuat la GIA TRI KHOI DAU, khong phai mot khoa"
    );
    assert_eq!(edited.note, "sua sau khi nhan tu de xuat");

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
    let layers = auratranslate_lib::core::dict::DictLayers::empty();
    let disabled = std::collections::BTreeSet::new();
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
    let candidate_id = glossary_pending_candidates(Some(&opened), &layers, &disabled)
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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.8 — BỎ một ứng viên (reject_candidate), qua bề mặt IPC thật
// ═════════════════════════════════════════════════════════════════════════════════

/// Bỏ một ứng viên chờ duyệt qua bề mặt IPC thật: hàng biến mất khỏi `pending_candidates`
/// (`resolution = 'rejected'`), và **0** mục Glossary nào được sinh ra — khác hẳn Nhận.
#[test]
fn glossary_reject_candidate_removes_the_row_from_the_pending_queue_and_writes_no_entry() {
    let layers = auratranslate_lib::core::dict::DictLayers::empty();
    let disabled = std::collections::BTreeSet::new();
    let root = temp_dir("reject-real-work");
    let global_dir = temp_dir("reject-real-work-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Bo Ung Vien");

    insert_import_scan_candidates(
        &opened.store,
        &[ScanCandidate {
            source_term: "落雁城".to_owned(),
            occurrence_count: 4,
            context_example: "vi du".to_owned(),
        }],
    )
    .expect("chen ung vien quet");
    let candidate_id = glossary_pending_candidates(Some(&opened), &layers, &disabled)
        .expect("liet ke bang cho")
        .into_iter()
        .find(|c| c.source_term == "落雁城")
        .expect("phai tim thay ung vien vua chen")
        .id;

    glossary_reject_candidate(Some(&opened), candidate_id).expect("bo ung vien qua commands::glossary");

    let still_pending = glossary_pending_candidates(Some(&opened), &layers, &disabled).expect("liet ke lai");
    assert!(
        still_pending.iter().all(|c| c.id != candidate_id),
        "ung vien da bo khong con trong bang cho DUYET"
    );

    // 0 muc Glossary nao duoc sinh -- Bo khac han Nhan, khong tao gi ca.
    let found = glossary_lookup_term(Some(&global), Some(&opened), "落雁城")
        .expect("tra lai qua commands::glossary")
        .entry;
    assert!(found.is_none(), "Bo khong duoc sinh mot muc Glossary nao");

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// `id` không khớp hàng ứng viên nào ⇒ lỗi mang `message_key`, không `Ok` rỗng.
#[test]
fn glossary_reject_candidate_with_an_unknown_id_fails_readably() {
    let root = temp_dir("reject-unknown-id");
    let opened = open_work(&root, "Id La Bo");

    let err = glossary_reject_candidate(Some(&opened), 999_999)
        .expect_err("id khong ton tai phai la loi, khong phai Ok rong");
    assert_eq!(err.message_key(), MessageKey::StoreWriteFailed);

    drop(opened);
    cleanup(&root);
}

/// Bỏ lại một ứng viên ĐÃ quyết (đã duyệt) ⇒ lỗi đọc được, hàng ứng viên cũ và mục Glossary
/// đã sinh KHÔNG bị đổi — vòng đời một chiều (AD-36) đứng cả hai chiều (Nhận rồi Bỏ cũng bị
/// chặn, không chỉ Bỏ rồi Bỏ lại).
#[test]
fn glossary_reject_candidate_on_an_already_approved_candidate_changes_nothing() {
    let layers = auratranslate_lib::core::dict::DictLayers::empty();
    let disabled = std::collections::BTreeSet::new();
    let root = temp_dir("reject-already-approved");
    let global_dir = temp_dir("reject-already-approved-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Da Duyet Roi Bo");

    insert_import_scan_candidates(
        &opened.store,
        &[ScanCandidate {
            source_term: "焚炎谷".to_owned(),
            occurrence_count: 6,
            context_example: "vi du".to_owned(),
        }],
    )
    .expect("chen ung vien quet");
    let candidate_id = glossary_pending_candidates(Some(&opened), &layers, &disabled)
        .expect("liet ke bang cho")
        .into_iter()
        .find(|c| c.source_term == "焚炎谷")
        .expect("phai tim thay ung vien vua chen")
        .id;

    glossary_approve_candidate(Some(&opened), candidate_id, None, Category::Place)
        .expect("duyet lan dau phai thanh cong");

    let err = glossary_reject_candidate(Some(&opened), candidate_id)
        .expect_err("ung vien da duyet khong the bo lai");
    assert_eq!(err.message_key(), MessageKey::StoreWriteFailed);

    // Muc Glossary da sinh tu luot Nhan van con nguyen.
    let found = glossary_lookup_term(Some(&global), Some(&opened), "焚炎谷")
        .expect("tra lai qua commands::glossary")
        .entry
        .expect("muc da duyet phai con nguyen");
    assert_eq!(found.translation, None);

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// Chưa có Tác phẩm nào đang mở ⇒ `project.no_work_open`, không `Ok` giả.
#[test]
fn glossary_reject_candidate_without_an_open_work_fails_readably() {
    let err = glossary_reject_candidate(None, 1)
        .expect_err("khong the bo ung vien khi chua mo Tac pham nao");
    assert_eq!(err.message_key(), MessageKey::ProjectNoWorkOpen);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.8 — THỨ TỰ của `pending_candidates`, qua bề mặt IPC thật
// ═════════════════════════════════════════════════════════════════════════════════

/// `occurrence_count DESC, id ASC` — tần suất giảm dần là tiêu chí chính, `id` tăng dần
/// là mốc phụ TẤT ĐỊNH cho ca ĐỒNG HẠNG. Chèn cố ý KHÔNG theo thứ tự mong đợi (id tăng dần
/// không khớp thứ tự tần suất), để một cổng vô tình giữ nguyên thứ tự CHÈN vẫn đỏ.
#[test]
fn glossary_pending_candidates_orders_by_occurrence_count_desc_then_id_asc_for_ties() {
    let layers = auratranslate_lib::core::dict::DictLayers::empty();
    let disabled = std::collections::BTreeSet::new();
    let root = temp_dir("pending-order");
    let opened = open_work(&root, "Thu Tu Bang Cho");

    // Ba hàng CÙNG occurrence_count (đồng hạng) chèn theo thứ tự A, B, C -- id tăng dần
    // đúng thứ tự chèn (AUTOINCREMENT), nên "id ASC" và "thứ tự chèn" trùng nhau ở NHÓM
    // này -- ca đồng hạng đo đúng mốc phụ, không đo trùng lặp với ca khác hạng ở dưới.
    insert_import_scan_candidates(
        &opened.store,
        &[
            ScanCandidate {
                source_term: "甲".to_owned(),
                occurrence_count: 10,
                context_example: "vi du".to_owned(),
            },
            ScanCandidate {
                source_term: "乙".to_owned(),
                occurrence_count: 10,
                context_example: "vi du".to_owned(),
            },
            ScanCandidate {
                source_term: "丙".to_owned(),
                occurrence_count: 10,
                context_example: "vi du".to_owned(),
            },
        ],
    )
    .expect("chen ba ung vien dong hang");

    // Chèn SAU (id lớn hơn CẢ BA hàng trên) nhưng tần suất CAO HƠN -- phải đứng ĐẦU danh
    // sách dù id lớn nhất, chứng minh occurrence_count là tiêu chí CHÍNH, không phải id.
    insert_import_scan_candidates(
        &opened.store,
        &[ScanCandidate {
            source_term: "丁".to_owned(),
            occurrence_count: 99,
            context_example: "vi du".to_owned(),
        }],
    )
    .expect("chen ung vien tan suat cao nhat, id lon nhat");

    let rows = glossary_pending_candidates(Some(&opened), &layers, &disabled)
        .expect("liet ke bang cho qua commands::glossary");
    let terms: Vec<&str> = rows.iter().map(|c| c.source_term.as_str()).collect();
    assert_eq!(
        terms,
        vec!["丁", "甲", "乙", "丙"],
        "tan suat 99 dung DAU du id lon nhat; ba hang dong hang 10 sap theo id TANG DAN"
    );

    drop(opened);
    cleanup(&root);
}

/// Hai lượt gọi liên tiếp trên cùng dữ liệu trả về **CÙNG MỘT** thứ tự — AC "mở hai lần
/// liên tiếp, thứ tự hai lượt giống hệt nhau" (đo trực tiếp bằng máy, không suy từ `ORDER
/// BY` đã đọc trong mã).
#[test]
fn glossary_pending_candidates_returns_the_same_order_across_two_consecutive_calls() {
    let layers = auratranslate_lib::core::dict::DictLayers::empty();
    let disabled = std::collections::BTreeSet::new();
    let root = temp_dir("pending-order-stable");
    let opened = open_work(&root, "On Dinh");

    insert_import_scan_candidates(
        &opened.store,
        &[
            ScanCandidate {
                source_term: "壹".to_owned(),
                occurrence_count: 3,
                context_example: "vi du".to_owned(),
            },
            ScanCandidate {
                source_term: "贰".to_owned(),
                occurrence_count: 3,
                context_example: "vi du".to_owned(),
            },
        ],
    )
    .expect("chen hai ung vien dong hang");

    let first = glossary_pending_candidates(Some(&opened), &layers, &disabled).expect("lan mo thu nhat");
    let second = glossary_pending_candidates(Some(&opened), &layers, &disabled).expect("lan mo thu hai");

    let first_ids: Vec<i64> = first.iter().map(|c| c.id).collect();
    let second_ids: Vec<i64> = second.iter().map(|c| c.id).collect();
    assert_eq!(first_ids, second_ids, "hai lan mo lien tiep phai tra ve DUNG MOT thu tu");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.9 — Quản lý Glossary: `glossary_list_entries` · `glossary_delete_term` ·
// `glossary_promote_term_to_global`, qua bề mặt IPC thật
// ═════════════════════════════════════════════════════════════════════════════════

/// AD-18 qua bề mặt liệt kê: cùng `source_term` ở CẢ hai tầng ⇒ hai hàng, hàng Work THẮNG
/// (`is_shadowed == false`) và hàng Global BỊ CHE (`is_shadowed == true`).
#[test]
fn glossary_list_entries_lists_both_tiers_and_flags_the_shadowed_row() {
    let root = temp_dir("list-entries-shadowed");
    let global_dir = temp_dir("list-entries-shadowed-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Liet Ke Ca Hai Tang");

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

    let rows =
        glossary_list_entries(Some(&global), Some(&opened)).expect("liet ke qua commands::glossary");
    assert_eq!(rows.len(), 2, "dung hai hang cho cung mot source_term. Nhan: {rows:?}");

    let winner = rows
        .iter()
        .find(|r| r.tier == "work" && !r.is_shadowed)
        .expect("phai co hang THANG o tang Work");
    assert_eq!(winner.translation.as_deref(), Some("Thanh Khau Rieng"));

    let shadowed = rows
        .iter()
        .find(|r| r.tier == "global" && r.is_shadowed)
        .expect("phai co hang BI CHE o tang Global");
    assert_eq!(shadowed.translation.as_deref(), Some("Thanh Khau Toan Cuc"));

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// Chưa mở Tác phẩm nào ⇒ chỉ mục tầng Global, không lỗi (I/O Matrix: *"Mở, chưa mở Tác
/// phẩm"*).
#[test]
fn glossary_list_entries_without_an_open_work_lists_only_the_global_tier() {
    let global_dir = temp_dir("list-entries-no-work-global");
    let global = open_global(&global_dir);

    glossary_add_term(
        Some(&global),
        None,
        GlossaryTier::Global,
        "青丘",
        Some("Thanh Khau"),
        "",
        Category::Place,
    )
    .expect("them muc o tang Global");

    let rows = glossary_list_entries(Some(&global), None).expect("liet ke khi chua mo Tac pham");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].tier, "global");
    assert!(!rows[0].is_shadowed, "mot tang duy nhat khong co gi de che no");

    drop(global);
    cleanup(&global_dir);
}

/// `glossary_delete_term` với `tier = Work` xoá ĐÚNG hàng trong `open.store` — kể cả một
/// mục ĐÃ CHỐT (§Always của spec 3.9: "Xoá một mục ĐÃ CHỐT là hợp lệ").
#[test]
fn glossary_delete_term_at_the_work_tier_removes_an_already_confirmed_row_from_a_real_open_work() {
    let root = temp_dir("delete-work-tier");
    let global_dir = temp_dir("delete-work-tier-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Xoa Tang Tac Pham");

    let id = glossary_add_term(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
    )
    .expect("them muc DA CHOT o tang Tac pham");

    glossary_delete_term(Some(&global), Some(&opened), GlossaryTier::Work, id)
        .expect("xoa mot muc DA CHOT qua commands::glossary phai THANH CONG");

    let found = glossary_lookup_term(Some(&global), Some(&opened), "慕容")
        .expect("tra lai qua commands::glossary")
        .entry;
    assert!(found.is_none(), "muc phai bien mat sau khi xoa");

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// `id` không khớp hàng nào ⇒ `glossary.entry_missing`, không `Ok` rỗng.
#[test]
fn glossary_delete_term_with_an_unknown_id_fails_readably() {
    let global_dir = temp_dir("delete-unknown-id-global");
    let global = open_global(&global_dir);

    let err = glossary_delete_term(Some(&global), None, GlossaryTier::Global, 999_999)
        .expect_err("id khong ton tai phai la loi, khong phai Ok rong");
    assert_eq!(err.message_key(), MessageKey::GlossaryEntryMissing);

    drop(global);
    cleanup(&global_dir);
}

/// Đẩy tầng, đích trống: `INSERT global` ⇒ `DELETE work`, qua bề mặt IPC thật — mục biến
/// khỏi tầng Work và xuất hiện ở tầng Global, `glossary_lookup_term` (AD-18) tra lại thấy nó
/// ở đúng tầng mới.
#[test]
fn glossary_promote_term_to_global_moves_the_row_through_a_real_open_work() {
    let root = temp_dir("promote-real-work");
    let global_dir = temp_dir("promote-real-work-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Day Tang");

    let id = glossary_add_term(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        "青丘",
        Some("Thanh Khâu"),
        "",
        Category::Place,
    )
    .expect("them muc o tang Tac pham");

    glossary_promote_term_to_global(Some(&global), Some(&opened), id)
        .expect("day tang qua commands::glossary phai THANH CONG khi dich trong");

    let found = glossary_lookup_term(Some(&global), Some(&opened), "青丘")
        .expect("tra lai qua commands::glossary")
        .entry
        .expect("muc phai con tim thay sau khi day tang");
    assert_eq!(found.tier, "global", "muc phai chuyen sang tang Global");
    assert_eq!(found.translation.as_deref(), Some("Thanh Khâu"));

    // Đối chứng: tầng Global-only (không qua `opened`) cũng thấy mục — nó không còn phụ
    // thuộc `OpenWork` nào để tồn tại.
    let global_only = glossary_lookup_term(Some(&global), None, "青丘")
        .expect("tra chi tang global")
        .entry
        .expect("muc phai ton tai doc lap voi OpenWork");
    assert_eq!(global_only.tier, "global");

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// Chưa mở Tác phẩm nào ⇒ `project.no_work_open` — bảng `glossary_entry` tầng Work chỉ tồn
/// tại trong MỘT `project.db` đang mở.
#[test]
fn glossary_promote_term_to_global_without_an_open_work_fails_readably() {
    let global_dir = temp_dir("promote-no-work-global");
    let global = open_global(&global_dir);

    let err = glossary_promote_term_to_global(Some(&global), None, 1)
        .expect_err("khong the day tang khi chua mo Tac pham nao");
    assert_eq!(err.message_key(), MessageKey::ProjectNoWorkOpen);

    drop(global);
    cleanup(&global_dir);
}

/// 🔴 "Đẩy một mục Global" (§I/O Matrix: *"Lệnh không áp dụng"*) — Rust không nhận tham số
/// `tier`, nó luôn đọc `id` từ `open.store` (`project.db`). Gọi lệnh này với một `id` chỉ
/// tồn tại ở `global.db` (không đi qua UI, thứ đã chặn nút/phím cho hàng Global) tự nhiên
/// rơi vào `glossary.entry_missing` — không có mục nào ở tầng Work mang `id` đó.
#[test]
fn glossary_promote_term_to_global_with_a_global_only_id_is_not_found_at_the_work_tier() {
    let root = temp_dir("promote-global-only-id");
    let global_dir = temp_dir("promote-global-only-id-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Day Mot Muc Global");

    let global_id = glossary_add_term(
        Some(&global),
        None,
        GlossaryTier::Global,
        "青丘",
        Some("Thanh Khau"),
        "",
        Category::Place,
    )
    .expect("them muc o tang Global");

    let err = glossary_promote_term_to_global(Some(&global), Some(&opened), global_id)
        .expect_err("mot id chi ton tai o tang Global khong duoc tim thay o tang Work");
    assert_eq!(err.message_key(), MessageKey::GlossaryEntryMissing);

    // Đối chứng: mục Global gốc không bị đụng tới.
    let found = glossary_lookup_term(Some(&global), Some(&opened), "青丘")
        .expect("tra lai")
        .entry
        .expect("muc Global goc phai con nguyen");
    assert_eq!(found.tier, "global");
    assert_eq!(found.translation.as_deref(), Some("Thanh Khau"));

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

/// Đẩy tầng, đích ĐÃ CÓ `source_term` này ⇒ `glossary.global_term_exists`, **0 lượt ghi**,
/// qua bề mặt IPC thật.
#[test]
fn glossary_promote_term_to_global_rejects_when_the_destination_already_has_the_term() {
    let root = temp_dir("promote-destination-exists-real");
    let global_dir = temp_dir("promote-destination-exists-real-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Dich Da Co");

    glossary_add_term(
        Some(&global),
        None,
        GlossaryTier::Global,
        "青丘",
        Some("Thanh Khau Cu"),
        "",
        Category::Place,
    )
    .expect("them muc o tang Global truoc");
    let work_id = glossary_add_term(
        Some(&global),
        Some(&opened),
        GlossaryTier::Work,
        "青丘",
        Some("Thanh Khâu Moi"),
        "",
        Category::Place,
    )
    .expect("them muc trung ten o tang Tac pham");

    let err = glossary_promote_term_to_global(Some(&global), Some(&opened), work_id)
        .expect_err("dich da co source_term nay phai bi TU CHOI");
    assert_eq!(err.message_key(), MessageKey::GlossaryGlobalTermExists);

    // Đối chứng 0 lượt ghi: cả hai mục giữ nguyên giá trị cũ.
    let global_entry = glossary_lookup_term(Some(&global), None, "青丘")
        .expect("tra chi tang global")
        .entry
        .expect("muc Global cu phai con nguyen");
    assert_eq!(global_entry.translation.as_deref(), Some("Thanh Khau Cu"));

    let work_entry = glossary_lookup_term(Some(&global), Some(&opened), "青丘")
        .expect("tra qua ca hai tang")
        .entry
        .expect("muc phai con tim thay");
    assert_eq!(work_entry.tier, "work", "AD-18: tang Work van thang, chua bi xoa");
    assert_eq!(work_entry.translation.as_deref(), Some("Thanh Khâu Moi"));

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}
