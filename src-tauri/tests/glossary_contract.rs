//! Hành vi Glossary hai tầng + vòng đời ba trạng thái (Story 3.1) + bảng chờ ứng viên
//! tách hẳn (Story 3.2) — I/O & Edge-Case Matrix của cả hai story.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `scope_contract.rs`/`store_contract.rs` — một tệp,
//! một mối quan tâm. Phép kiểm **tĩnh trên cây nguồn** sống ở `glossary_boundary.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — thừa kế nguyên vẹn từ `scope_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + `AtomicU64`). Không thêm `tempfile`.
//! 2. **Drop `Store` TRƯỚC khi xoá thư mục** — Windows từ chối xoá tệp đang mở.
//! 3. Không `sleep` dài.
//! 4. Không ca nào treo khi nó trượt.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! MỖI HÀNG CỦA MA TRẬN I/O LÀ ĐÚNG MỘT CA, VÀ TÊN HÀM LÀ CÂU KHẲNG ĐỊNH
//! ─────────────────────────────────────────────────────────────────────────────
//! Ca *"tầng Tác phẩm chờ chốt che tầng Global đã chốt"* là ca dễ cài ngược nhất của cả
//! story — một cài đặt lọc TRƯỚC khi phân giải cho ra kết quả sai mà trông rất hợp lý.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::core::glossary::scan::ScanCandidate;
use auratranslate_lib::core::glossary::{
    CandidateOrigin, Category, GlossaryError, GlossaryTier, TermOrigin, add_manual_term,
    approve_candidate, confirm_translation, delete_manual_term, entries_eligible_for_injection,
    insert_candidate, insert_import_scan_candidates, insert_manual_entry, list_all_entries,
    load_tier, pending_candidates, promote_to_global, reject_candidate,
    resolve_term_for_quick_add, update_manual_term,
};
use auratranslate_lib::core::scope::{ScopeError, ScopeResolver, WorkScope};
use auratranslate_lib::core::store::{
    GLOBAL_MIGRATIONS, GLOSSARY_CANDIDATE_DDL, GLOSSARY_ENTRY_DDL, Store, StoreError, StoreSpec,
    Transaction,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

/// Một thư mục tạm **của riêng ca này**. Xem luật 1 ở doc-comment đầu tệp.
fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-glossary-{}-{}-{}",
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

fn open_project(dir: &Path) -> Store {
    Store::open(StoreSpec::project(dir.join("project.db"))).expect("mo project.db")
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 1 — chỉ tầng Global, đã chốt
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_confirmed_global_only_entry_is_eligible_for_injection() {
    let dir = temp_dir("global-only-confirmed");
    let store = open_global(&dir);

    insert_manual_entry(
        &store,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
    )
    .expect("chen muc da chot");

    let resolver = ScopeResolver::global_only();

    let eligible = entries_eligible_for_injection(&resolver, &store, None)
        .expect("entries_eligible_for_injection khong loi voi kind hop le");

    assert_eq!(eligible.len(), 1, "muc da chot phai du dieu kien chen");
    assert_eq!(eligible[0].source_term, "慕容");
    assert_eq!(eligible[0].translation.as_deref(), Some("Mộ Dung"));

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 2 — cả hai tầng, cả hai đã chốt: tầng Tác phẩm thắng theo từng thuật ngữ
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn when_both_tiers_confirm_the_same_term_the_work_tier_wins() {
    let dir = temp_dir("both-tiers-confirmed");
    let global_store = open_global(&dir);
    let work_store = open_project(&dir);

    insert_manual_entry(
        &global_store,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
    )
    .expect("chen muc global");
    insert_manual_entry(
        &work_store,
        "慕容",
        Some("Mộ Dong"),
        "",
        Category::Person,
    )
    .expect("chen muc work");

    let resolver = ScopeResolver::with_work(WorkScope {
        work_id: "0192f3c4-5678-4abc-8def-0123456789ab".to_owned(),
    });

    let eligible = entries_eligible_for_injection(&resolver, &global_store, Some(&work_store))
        .expect("entries_eligible_for_injection khong loi voi kind hop le");

    assert_eq!(
        eligible.len(),
        1,
        "cung mot thuat ngu o hai tang phai gop thanh dung MOT muc du dieu kien chen"
    );
    assert_eq!(
        eligible[0].translation.as_deref(),
        Some("Mộ Dong"),
        "AD-18: tang Tac pham thang theo TUNG thuat ngu"
    );

    drop(global_store);
    drop(work_store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 3 — tầng Tác phẩm CHỜ CHỐT che tầng Global ĐÃ CHỐT ⇒ KHÔNG đủ điều kiện chèn
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **Ca dễ cài ngược nhất của cả story.** Một cài đặt lọc TRƯỚC khi phân giải (bỏ mục
/// chưa chốt ở MỖI tầng rồi mới hợp hai tầng) sẽ để lộ mục Global bên dưới ra ngoài —
/// chèn bản dịch toàn cục cho đúng thuật ngữ người dùng vừa cố ý để ngỏ ở Tác phẩm này.
/// Ca này chỉ xanh nếu điều kiện chèn được áp SAU khi `ScopeResolver::apply_override` đã
/// quyết mục nào thắng.
#[test]
fn a_pending_work_tier_entry_shadows_and_disqualifies_a_confirmed_global_entry() {
    let dir = temp_dir("work-pending-shadows-global");
    let global_store = open_global(&dir);
    let work_store = open_project(&dir);

    insert_manual_entry(
        &global_store,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
    )
    .expect("chen muc global da chot");
    insert_manual_entry(
        &work_store,
        "慕容",
        None,
        "",
        Category::Person,
    )
    .expect("chen muc work cho chot");

    let resolver = ScopeResolver::with_work(WorkScope {
        work_id: "0192f3c4-5678-4abc-8def-0123456789ab".to_owned(),
    });

    // ⚠️ Gọi qua đúng chữ ký PHƠI RA — `entries_eligible_for_injection` tự `load_tier` cả
    // hai `Store` bên trong rồi mới phân giải. Ca này vẫn là bằng chứng cho "lọc SAU khi
    // phân giải": nếu cài đặt bên trong lỡ lọc TRƯỚC (bỏ mục chưa chốt ở mỗi tầng rồi mới
    // hợp hai tầng), mục Global đã chốt sẽ lộ ra và `eligible` sẽ KHÔNG rỗng — đúng lỗi mà
    // ca này tồn tại để bắt, đo được từ NGOÀI mà không cần nhìn vào cài đặt bên trong.
    let eligible = entries_eligible_for_injection(&resolver, &global_store, Some(&work_store))
        .expect("entries_eligible_for_injection khong loi voi kind hop le");

    assert!(
        eligible.is_empty(),
        "muc cho chot o tang Tac pham CHE muc da chot o tang Global -- thuat ngu nay \
         KHONG duoc du dieu kien chen. Nhan: {eligible:?}"
    );

    drop(global_store);
    drop(work_store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 4 — chỉ tầng Global, chờ chốt: có mặt khi liệt kê, KHÔNG đủ điều kiện chèn
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_pending_global_only_entry_is_listed_but_not_eligible_for_injection() {
    let dir = temp_dir("global-only-pending");
    let store = open_global(&dir);

    insert_manual_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
    )
    .expect("chen muc cho chot");

    // `load_tier` trực tiếp — CA NÀY canh mệnh đề "có mặt khi liệt kê", thứ
    // `entries_eligible_for_injection` không trả lời được (nó chỉ trả mục ĐỦ điều kiện).
    let global = load_tier(&store).expect("nap tang global");
    assert!(
        global.contains_key("青丘"),
        "muc cho chot phai co mat khi liet ke"
    );
    assert!(!global["青丘"].is_confirmed());

    let resolver = ScopeResolver::global_only();
    let eligible = entries_eligible_for_injection(&resolver, &store, None)
        .expect("entries_eligible_for_injection khong loi voi kind hop le");

    assert!(
        eligible.is_empty(),
        "mot muc cho chot khong duoc du dieu kien chen"
    );

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 5 — không mở Tác phẩm nào: phân giải bằng nguyên tầng Global
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn with_no_work_open_resolution_is_the_whole_global_tier() {
    let dir = temp_dir("no-work-open");
    let store = open_global(&dir);

    insert_manual_entry(
        &store,
        "青丘",
        Some("Thanh Khâu"),
        "",
        Category::Place,
    )
    .expect("chen muc da chot");

    let resolver = ScopeResolver::global_only();
    assert!(
        !resolver.has_work_tier(),
        "global_only() khong duoc mang tang Tac pham"
    );

    let eligible = entries_eligible_for_injection(&resolver, &store, None)
        .expect("entries_eligible_for_injection khong loi voi kind hop le");

    assert_eq!(eligible.len(), 1);
    assert_eq!(eligible[0].source_term, "青丘");

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 6 — bản dịch rỗng: CHECK từ chối
// ═════════════════════════════════════════════════════════════════════════════════

/// **MỌI** dạng "trắng" — chuỗi rỗng, cả 25 điểm mã mang thuộc tính Unicode `White_Space`,
/// và một chuỗi TRỘN.
///
/// 🔵 **CẬP NHẬT 2026-08-19 (vòng rà soát #2) — từ BẢY lên 25, và đó là lượt sửa THỨ HAI
/// của cùng một lỗ hổng.** Lượt #1 phát hiện `trim(X)` MỘT tham số chỉ cắt dấu cách ASCII
/// và thay bằng `trim(X, <bảng ký tự>)` HAI tham số với **bảy** ký tự. Đo lại: bảng bảy ký
/// tự vẫn để **17** điểm mã đi lọt — U+0085 · U+1680 · U+2000‥U+200A (gồm U+2009 THIN
/// SPACE) · U+2028 · U+2029 · U+202F · U+205F. Tức lượt #1 THU HẸP lỗ hổng chứ không đóng
/// nó, và một hàm tên `seven_blank_forms` khoá đúng bảy ca mà nó liệt — không phải cái LỚP
/// mà nó tự xưng là canh.
///
/// ⇒ Danh sách dưới đây là bằng chứng CHẠY ĐƯỢC cho bảng ký tự của `GLOSSARY_ENTRY_DDL`,
/// một hàng một điểm mã. Đừng rút ngắn nó về một mẫu đại diện: một mẫu đại diện là đúng
/// hình dạng đã để lọt 17 điểm mã suốt lượt #1.
fn every_blank_form() -> [(&'static str, &'static str); 27] {
    [
        ("rong", ""),
        ("SPACE U+0020", "   "),
        ("TAB U+0009", "\t\t"),
        ("LF U+000A", "\n\n"),
        ("VT U+000B", "\u{000B}\u{000B}"),
        ("FF U+000C", "\u{000C}\u{000C}"),
        ("CR U+000D", "\r\r"),
        ("NEL U+0085", "\u{0085}\u{0085}"),
        ("NBSP U+00A0", "\u{00A0}\u{00A0}"),
        ("OGHAM SPACE U+1680", "\u{1680}\u{1680}"),
        ("EN QUAD U+2000", "\u{2000}\u{2000}"),
        ("EM QUAD U+2001", "\u{2001}\u{2001}"),
        ("EN SPACE U+2002", "\u{2002}\u{2002}"),
        ("EM SPACE U+2003", "\u{2003}\u{2003}"),
        ("THREE-PER-EM U+2004", "\u{2004}\u{2004}"),
        ("FOUR-PER-EM U+2005", "\u{2005}\u{2005}"),
        ("SIX-PER-EM U+2006", "\u{2006}\u{2006}"),
        ("FIGURE SPACE U+2007", "\u{2007}\u{2007}"),
        ("PUNCTUATION SPACE U+2008", "\u{2008}\u{2008}"),
        ("THIN SPACE U+2009", "\u{2009}\u{2009}"),
        ("HAIR SPACE U+200A", "\u{200A}\u{200A}"),
        ("LINE SEPARATOR U+2028", "\u{2028}\u{2028}"),
        ("PARAGRAPH SEPARATOR U+2029", "\u{2029}\u{2029}"),
        ("NARROW NBSP U+202F", "\u{202F}\u{202F}"),
        ("MEDIUM MATH SPACE U+205F", "\u{205F}\u{205F}"),
        ("IDEOGRAPHIC SPACE U+3000", "\u{3000}\u{3000}"),
        (
            "tron ca 25 loai",
            " \t\n\u{000B}\u{000C}\r\u{0085}\u{00A0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\
             \u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200A}\u{2028}\u{2029}\u{202F}\
             \u{205F}\u{3000} ",
        ),
    ]
}

/// Đối chứng dương cho [`every_blank_form`]: danh sách phải khớp **đúng** tập mà
/// `char::is_whitespace` của Rust nhận, cộng chuỗi rỗng và chuỗi trộn.
///
/// 🔴 Không có ca này thì `every_blank_form` là một danh sách chép tay, và một điểm mã bị
/// gõ sót lại đọc thành "đã canh hết" — đúng cách bảy ký tự của lượt #1 đọc thành đã đủ.
#[test]
fn the_blank_form_list_covers_every_unicode_whitespace_code_point() {
    let listed: std::collections::BTreeSet<char> = every_blank_form()
        .iter()
        .flat_map(|(_, s)| s.chars())
        .collect();

    let expected: std::collections::BTreeSet<char> = (0u32..=0x10FFFF)
        .filter_map(char::from_u32)
        .filter(|c| c.is_whitespace())
        .collect();

    assert_eq!(
        listed, expected,
        "danh sach dang trang phai khop DUNG tap `char::is_whitespace` cua Rust -- \
         thieu: {:?} | thua: {:?}",
        expected.difference(&listed).collect::<Vec<_>>(),
        listed.difference(&expected).collect::<Vec<_>>()
    );
}

#[test]
fn an_empty_or_whitespace_only_translation_is_refused_and_writes_nothing() {
    let dir = temp_dir("empty-translation");
    let store = open_global(&dir);

    for (label, blank) in every_blank_form() {
        let result = insert_manual_entry(
            &store,
            "term",
            Some(blank),
            "",
            Category::Other,
        );
        assert!(
            matches!(result, Err(StoreError::WriteFailed { .. })),
            "translation trang dang '{label}' phai bi CHECK tu choi qua StoreError::WriteFailed. \
             Nhan: {result:?}"
        );
    }

    let rows: i64 = store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM glossary_entry", [], |r| r.get(0)))
        .expect("dem hang");
    assert_eq!(
        rows, 0,
        "moi luot chen bi tu choi khong duoc de lai mot hang nao"
    );

    drop(store);
    cleanup(&dir);
}

/// Ca tương đương cho `source_term` — P2 của lượt rà soát ba lớp: cột này vừa là khoá tra
/// cứu vừa là khoá của `idx_glossary_entry_source_term`, và trước bản vá không có rào rỗng
/// nào ngoài `NOT NULL` — `insert_manual_entry("", …)` chiếm vĩnh viễn ô chuỗi rỗng của chỉ mục
/// UNIQUE đó.
#[test]
fn an_empty_or_whitespace_only_source_term_is_refused_and_writes_nothing() {
    let dir = temp_dir("empty-source-term");
    let store = open_global(&dir);

    for (label, blank) in every_blank_form() {
        let result = insert_manual_entry(
            &store,
            blank,
            Some("ban dich"),
            "",
            Category::Other,
        );
        assert!(
            matches!(result, Err(StoreError::WriteFailed { .. })),
            "source_term trang dang '{label}' phai bi CHECK tu choi qua StoreError::WriteFailed. \
             Nhan: {result:?}"
        );
    }

    let rows: i64 = store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM glossary_entry", [], |r| r.get(0)))
        .expect("dem hang");
    assert_eq!(
        rows, 0,
        "moi luot chen bi tu choi khong duoc de lai mot hang nao"
    );

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 7 — lùi vòng đời: trigger RAISE(ABORT) từ chối
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn undoing_a_confirmed_translation_back_to_pending_is_refused_by_the_trigger() {
    let dir = temp_dir("lifecycle-is-one-way");
    let store = open_global(&dir);

    let id = insert_manual_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
    )
    .expect("chen muc cho chot");
    confirm_translation(&store, id, "Thanh Khâu").expect("chot ban dich");

    let regressed = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE glossary_entry SET translation = NULL WHERE id = ?1",
            [id],
        )?;
        Ok(())
    });
    assert!(
        matches!(regressed, Err(StoreError::WriteFailed { .. })),
        "trigger glossary_entry_lifecycle_is_one_way phai chan luot lui ve NULL. \
         Nhan: {regressed:?}"
    );

    let translation: Option<String> = store
        .read(|conn| {
            conn.query_row(
                "SELECT translation FROM glossary_entry WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
        })
        .expect("doc lai hang");
    assert_eq!(
        translation.as_deref(),
        Some("Thanh Khâu"),
        "giao dich bi tu choi phai rollback -- ban dich da chot phai o lai nguyen ven"
    );

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 8 — gõ sai tên loại: ScopeError::UnknownKind, không phân giải gì
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn resolving_glossary_data_with_a_misspelled_wire_kind_is_refused() {
    use std::collections::BTreeMap;

    let resolver = ScopeResolver::global_only();
    let empty: BTreeMap<String, auratranslate_lib::core::glossary::GlossaryEntry> =
        BTreeMap::new();

    let err = resolver
        .apply_override("glosary", &empty, None)
        .expect_err("mot khoa go sai khong duoc phan giai am tham");
    assert_eq!(
        err,
        ScopeError::UnknownKind {
            wire: "glosary".to_owned()
        }
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 9 — gọi sai ngữ nghĩa: ScopeError::WrongSemantics (đã có ở Story 1.8)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn resolving_glossary_data_through_merge_semantics_is_refused() {
    use auratranslate_lib::core::glossary::GlossaryEntry;
    use auratranslate_lib::core::scope::kinds::Semantics;

    let resolver = ScopeResolver::global_only();
    let empty: Vec<GlossaryEntry> = Vec::new();

    let err = resolver
        .apply_merge("glossary", &empty, None, None)
        .expect_err("Glossary khai Override, khong duoc phan giai nhu Merge");
    assert!(matches!(
        err,
        ScopeError::WrongSemantics {
            declared: Semantics::Override,
            called: Semantics::Merge,
            ..
        }
    ));
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 10 — kho phiên bản mới hơn: từ chối mở, không chạm một byte
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_global_database_newer_than_the_app_is_refused_without_touching_a_single_byte() {
    let dir = temp_dir("global-schema-too-new");
    let db = dir.join("global.db");
    let target = GLOBAL_MIGRATIONS
        .last()
        .map(|m| m.to_version)
        .unwrap_or(0);

    {
        let conn = rusqlite::Connection::open(&db).expect("dung fixture");
        conn.execute_batch(&format!(
            "PRAGMA journal_mode = delete;\n\
             CREATE TABLE from_the_future (id INTEGER PRIMARY KEY);\n\
             PRAGMA user_version = {};",
            target + 1
        ))
        .expect("dat fixture o phien ban tuong lai");
    }

    let before = fs::metadata(&db).expect("doc metadata truoc").len();

    let refused = Store::open(StoreSpec::global(db.clone()));
    assert!(
        matches!(refused, Err(StoreError::SchemaTooNew { .. })),
        "mot global.db moi hon ung dung phai bi tu choi mo. Nhan: {refused:?}"
    );

    assert_eq!(
        fs::metadata(&db).expect("doc metadata sau").len(),
        before,
        "mot lan tu choi mo KHONG duoc dung toi mot byte nao cua tep"
    );

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Lượt rà soát ba lớp (2026-08-19) — P8 · P9
// ═════════════════════════════════════════════════════════════════════════════════

/// P8(a) — `source_term` trùng ⇒ `StoreError::WriteFailed`, không một hàng thứ hai.
///
/// `idx_glossary_entry_source_term` là `UNIQUE`; ca này khoá hành vi đó bằng test thay vì
/// chỉ đọc bằng mắt trong doc-comment của `GLOSSARY_ENTRY_DDL`.
#[test]
fn a_duplicate_source_term_is_refused_and_the_original_row_survives() {
    let dir = temp_dir("duplicate-source-term");
    let store = open_global(&dir);

    insert_manual_entry(
        &store,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
    )
    .expect("chen muc dau tien");

    let second = insert_manual_entry(
        &store,
        "慕容",
        Some("Mo Dung Khac"),
        "",
        Category::Person,
    );
    assert!(
        matches!(second, Err(StoreError::WriteFailed { .. })),
        "source_term trung phai bi UNIQUE tu choi qua StoreError::WriteFailed. Nhan: {second:?}"
    );

    let global = load_tier(&store).expect("nap tang global");
    assert_eq!(global.len(), 1, "luot chen thu hai bi tu choi khong duoc them mot hang nao");
    assert_eq!(
        global["慕容"].translation.as_deref(),
        Some("Mộ Dung"),
        "hang goc phai o lai NGUYEN VEN sau luot chen thu hai bi tu choi"
    );

    drop(store);
    cleanup(&dir);
}

/// P8(b) — bản dịch rỗng/khoảng trắng đi qua `confirm_translation` (không chỉ
/// `insert_manual_entry`) cũng bị `CHECK` từ chối — cùng một hằng DDL, nhưng khác đường ghi.
#[test]
fn confirming_with_a_blank_translation_is_refused_by_the_same_check_insert_uses() {
    let dir = temp_dir("confirm-blank-translation");
    let store = open_global(&dir);

    let id = insert_manual_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
    )
    .expect("chen muc cho chot");

    for (label, blank) in every_blank_form() {
        let result = confirm_translation(&store, id, blank);
        assert!(
            matches!(result, Err(StoreError::WriteFailed { .. })),
            "confirm_translation voi ban dich trang dang '{label}' phai bi CHECK tu choi. \
             Nhan: {result:?}"
        );
    }

    let global = load_tier(&store).expect("nap tang global");
    assert!(
        !global["青丘"].is_confirmed(),
        "moi luot chot bi tu choi khong duoc lam muc chuyen sang da chot"
    );

    drop(store);
    cleanup(&dir);
}

/// P8(c) — 🔵 **CẬP NHẬT 2026-08-20 (Story 3.3)** — `confirm_translation` với `id` KHÔNG
/// khớp hàng nào NAY TRẢ VỀ LỖI, không còn `Ok(())` cho 0 hàng đổi.
///
/// Bản trước của test này (P8(c), Story 3.1) khoá đúng rủi ro *"rỗng im lặng"* mà chính
/// doc-comment của `confirm_translation` cảnh báo, và ghi rõ "Chủ của rủi ro đó: Story
/// 3.3" — đây là lượt đóng nợ đó. `tx.execute` trả về SỐ HÀNG bị đổi; `0` giờ là
/// `StoreError::WriteFailed` với một câu chẩn đoán đọc được.
#[test]
fn confirming_an_unknown_id_is_rejected_and_changes_nothing() {
    let dir = temp_dir("confirm-unknown-id");
    let store = open_global(&dir);

    insert_manual_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
    )
    .expect("chen mot muc that de doi chung");

    let result = confirm_translation(&store, 999_999, "Thanh Khâu");
    assert!(
        matches!(result, Err(StoreError::WriteFailed { .. })),
        "chot mot id khong ton tai phai bi TU CHOI (0 hang doi la mot loi, khong con la mot \
         thanh cong im lang). Nhan: {result:?}"
    );

    let global = load_tier(&store).expect("nap tang global");
    assert_eq!(global.len(), 1, "khong hang moi nao duoc tao ra");
    assert!(
        !global["青丘"].is_confirmed(),
        "muc that duy nhat trong kho khong duoc dung toi"
    );

    drop(store);
    cleanup(&dir);
}

/// P9 — `confirm_translation` sửa được một mục ĐÃ chốt sang một bản dịch KHÁC (Story 3.9,
/// "sửa có hiệu lực ngay"). Đây là hành vi ĐÚNG, không phải một lỗ hổng — ca này khoá nó
/// lại để người sau không "sửa" thành chỉ-nhận-chờ-chốt.
#[test]
fn confirming_an_already_confirmed_entry_again_overwrites_the_translation() {
    let dir = temp_dir("reconfirm-changes-translation");
    let store = open_global(&dir);

    let id = insert_manual_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
    )
    .expect("chen muc cho chot");
    confirm_translation(&store, id, "Thanh Khau Cu").expect("chot lan dau");

    confirm_translation(&store, id, "Thanh Khâu")
        .expect("sua mot muc DA chot sang ban dich khac phai THANH CONG (Story 3.9)");

    let global = load_tier(&store).expect("nap tang global");
    assert_eq!(
        global["青丘"].translation.as_deref(),
        Some("Thanh Khâu"),
        "ban dich phai la gia tri MOI NHAT, khong phai gia tri chot lan dau"
    );
    assert!(global["青丘"].is_confirmed());

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Vòng rà soát #2 (2026-08-19) — năm hành vi trước đó XOÁ ĐI MÀ BỘ TEST VẪN XANH
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 `insert_manual_entry` cắt khoảng trắng biên ở **tầng Rust**, và đây là ca DUY NHẤT đo được
/// điều đó từ ngoài.
///
/// ⚠️ Ca trùng đã có (`a_duplicate_source_term_is_refused_and_the_original_row_survives`)
/// chèn **cùng một chuỗi** hai lần, nên nó xanh y hệt dù `.trim()` ở `insert_manual_entry` có bị
/// xoá hay không. Các ca dạng trắng cũng không đo được: chúng bị `CHECK` của SQL chặn độc
/// lập với tầng Rust. ⇒ Không có ca này, xoá `.trim()` khỏi `insert_manual_entry` là một lượt đỏ
/// KHÔNG XẢY RA, và `" 慕容"` với `"慕容"` thành hai hàng dưới một chỉ mục tự xưng là "một
/// thuật ngữ, một mục".
#[test]
fn a_padded_source_term_collides_with_its_trimmed_twin_instead_of_becoming_a_second_row() {
    let dir = temp_dir("source-term-trim-collides");
    let store = open_global(&dir);

    insert_manual_entry(
        &store,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
    )
    .expect("chen muc dau tien");

    let padded = insert_manual_entry(
        &store,
        "  慕容\t",
        Some("Mo Dung Khac"),
        "",
        Category::Person,
    );
    assert!(
        matches!(padded, Err(StoreError::WriteFailed { .. })),
        "`  慕容\\t` phai bi cat khoang trang bien o tang Rust roi va vao DUNG mot UNIQUE. \
         Nhan: {padded:?} -- neu la Ok, `.trim()` o insert_manual_entry da bien mat va chi muc \
         `mot thuat ngu, mot muc` nay giu hai hang cho cung mot thuat ngu."
    );

    let global = load_tier(&store).expect("nap tang global");
    assert_eq!(global.len(), 1, "khong duoc co hang thu hai");
    assert!(
        global.contains_key("慕容"),
        "khoa phai la dang DA CAT, khong phai `  慕容\\t`. Nhan: {:?}",
        global.keys().collect::<Vec<_>>()
    );

    drop(store);
    cleanup(&dir);
}

/// 🔴 `confirm_translation` cũng cắt khoảng trắng biên — cùng lý do, khác đường ghi.
///
/// ⚠️ Ca `confirming_an_already_confirmed_entry_again_overwrites_the_translation` truyền một
/// chuỗi vốn đã sạch nên nó không đo được `.trim()`. Và `CHECK` của SQL **không** thay thế
/// được tầng Rust ở đây: nó chỉ cấm chuỗi TRẮNG HOÀN TOÀN, không cấm nội dung thật mang
/// khoảng trắng thừa hai đầu.
#[test]
fn confirming_a_padded_translation_stores_it_trimmed() {
    let dir = temp_dir("confirm-trims-padding");
    let store = open_global(&dir);

    let id = insert_manual_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
    )
    .expect("chen muc cho chot");

    confirm_translation(&store, id, "  Thanh Khâu\u{3000}").expect("chot ban dich co dem");

    let global = load_tier(&store).expect("nap tang global");
    assert_eq!(
        global["青丘"].translation.as_deref(),
        Some("Thanh Khâu"),
        "ban dich phai duoc cat khoang trang bien TRUOC khi ghi -- neu nhan lai chuoi co \
         dem thi `.trim()` o confirm_translation da bien mat"
    );

    drop(store);
    cleanup(&dir);
}

/// 🔴 Mọi biến thể của `Category` phải ghi xuống đĩa được và đọc lại đúng, đi qua **CẢ HAI**
/// cửa của `term_origin` mà Story 3.2 để lại.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.2) — viết lại cho hợp chữ ký mới.** Trước lượt này, ca
/// này chèn cả ba biến thể `TermOrigin` (`Manual` · `ImportScan` · `ReviewHarvest`) qua MỘT
/// hàm `insert_entry` duy nhất. `insert_entry` mất tham số `term_origin` (đổi tên
/// `insert_manual_entry`, luôn `manual`), nên ba biến thể nay đi qua HAI cửa: cửa 1 —
/// `insert_manual_entry` — chỉ còn sinh được `manual`; cửa 2 — `insert_candidate` rồi
/// `approve_candidate` — sinh `import_scan`/`review_harvest`, suy từ `CandidateOrigin` của
/// chính hàng ứng viên. Bảng chờ chỉ tồn tại ở `project.db` (§Never: "Bảng ứng viên ở
/// `global.db`"), nên ca này chuyển sang `open_project` để cả hai cửa cùng dùng được một
/// `Store`.
///
/// ⚠️ Trước Story 3.1, bộ test chỉ từng ghi `Person` · `Place` · `Other` · `Manual`. **Ba
/// biến thể chưa từng chạm đĩa lần nào**: `DomainTerm` · `ImportScan` · `ReviewHarvest`. Một
/// lỗi gõ trong `as_str()`/`to_term_origin()` của bất kỳ cái nào trong ba — `"domainterm"`,
/// `"importscan"` — đi qua trình biên dịch sạch, đi qua cả bộ test sạch, và chỉ đỏ lần đầu
/// Story 3.5 (quét khi nhập) hay Epic 8 (thu hoạch từ review) ghi một hàng THẬT. Ca này đóng
/// khoảng cách giữa kiểu Rust và `CHECK … IN (…)` bằng phép chạy, không bằng lời hứa trong
/// doc-comment.
#[test]
fn every_category_and_term_origin_variant_round_trips_through_the_store() {
    let dir = temp_dir("all-enum-variants");
    let store = open_project(&dir);

    let categories = [
        Category::Person,
        Category::Place,
        Category::DomainTerm,
        Category::Other,
    ];

    // Cửa 1 -- `insert_manual_entry`, luôn `manual`.
    for (i, category) in categories.iter().enumerate() {
        let term = format!("thu-cong-{i}");
        insert_manual_entry(&store, &term, Some("ban dich"), "", *category).unwrap_or_else(
            |e| {
                panic!(
                    "category={category} qua insert_manual_entry phai ghi xuong dia duoc -- \
                     mot `WriteFailed` o day nghia la `as_str()` da troi khoi \
                     `CHECK (… IN (…))` cua GLOSSARY_ENTRY_DDL. Nhan: {e:?}"
                )
            },
        );
    }

    // Cửa 2 -- `insert_candidate` + `approve_candidate`, suy `term_origin` từ
    // `CandidateOrigin::to_term_origin()`.
    let candidate_origins = [CandidateOrigin::ImportScan, CandidateOrigin::ReviewHarvest];
    for (i, category) in categories.iter().enumerate() {
        for (j, candidate_origin) in candidate_origins.iter().enumerate() {
            let term = format!("ung-vien-{i}-{j}");
            let id = insert_candidate(&store, &term, *candidate_origin)
                .unwrap_or_else(|e| panic!("insert_candidate({term}) phai ghi duoc. Nhan: {e:?}"));
            approve_candidate(&store, id, Some("ban dich"), *category).unwrap_or_else(|e| {
                panic!(
                    "approve_candidate voi candidate_origin={candidate_origin} \
                     category={category} phai ghi xuong dia duoc -- mot `WriteFailed` o day \
                     nghia la `as_str()`/`to_term_origin()` da troi khoi `CHECK (… IN (…))` \
                     cua GLOSSARY_ENTRY_DDL. Nhan: {e:?}"
                )
            });
        }
    }

    let all = load_tier(&store).expect("nap tang project");
    assert_eq!(
        all.len(),
        categories.len() + categories.len() * candidate_origins.len(),
        "moi luot ghi qua ca hai cua phai de lai dung mot hang -- khong luot nao duoc \
         tu choi ngam"
    );

    for (i, category) in categories.iter().enumerate() {
        let entry = &all[&format!("thu-cong-{i}")];
        assert_eq!(entry.category, *category, "category phai doc lai DUNG bien the da ghi");
        assert_eq!(
            entry.term_origin,
            TermOrigin::Manual,
            "insert_manual_entry phai luon sinh term_origin = manual"
        );
    }
    for (i, category) in categories.iter().enumerate() {
        for (j, candidate_origin) in candidate_origins.iter().enumerate() {
            let entry = &all[&format!("ung-vien-{i}-{j}")];
            assert_eq!(entry.category, *category, "category phai doc lai DUNG bien the da ghi");
            assert_eq!(
                entry.term_origin,
                candidate_origin.to_term_origin(),
                "term_origin phai la anh xa TOAN PHAN cua candidate_origin -- khong roi ve \
                 mot gia tri khac"
            );
        }
    }

    drop(store);
    cleanup(&dir);
}

/// 🔴 **CẢ BẢY cột** phải đi trọn vòng ghi rồi đọc.
///
/// ⚠️ Trước ca này bộ test chỉ khẳng định `source_term` và `translation`. Năm cột còn lại —
/// `id` · `note` · `category` · `term_origin` · `created_at` — không một dòng `assert` nào
/// chạm tới, nên **đảo hai chỉ số cột** trong `load_tier` (`row.get(3)` là `note`,
/// `row.get(4)` là `category`, cả hai đều `TEXT`) là một thay đổi mà không ca nào nhìn thấy;
/// một `strftime` hỏng làm `created_at` rỗng cũng vậy.
#[test]
fn every_column_round_trips_through_load_tier() {
    let dir = temp_dir("all-columns-round-trip");
    let store = open_global(&dir);

    let id = insert_manual_entry(
        &store,
        "慕容",
        Some("Mộ Dung"),
        "ho kep, khong phai ten don",
        Category::DomainTerm,
    )
    .expect("chen muc day du bay cot");

    let global = load_tier(&store).expect("nap tang global");
    let entry = &global["慕容"];

    assert_eq!(entry.id, id, "id phai la rowid ma insert_manual_entry vua tra ve");
    assert_eq!(entry.source_term, "慕容");
    assert_eq!(entry.translation.as_deref(), Some("Mộ Dung"));
    assert_eq!(
        entry.note, "ho kep, khong phai ten don",
        "note phai doc lai nguyen van -- doc nham cot se lot ra o day"
    );
    assert_eq!(entry.category, Category::DomainTerm);
    assert_eq!(
        entry.term_origin,
        TermOrigin::Manual,
        "insert_manual_entry (Story 3.2) luon sinh manual -- vong qua ImportScan/ReviewHarvest \
         da chuyen sang every_category_and_term_origin_variant_round_trips_through_the_store"
    );

    // `created_at` sinh o tang SQL bang strftime('%Y-%m-%dT%H:%M:%fZ', 'now') -- ISO-8601
    // UTC, dung 24 ky tu. Khong so sanh gia tri (no la thoi diem chay), so sanh HINH DANG.
    assert_eq!(
        entry.created_at.len(),
        24,
        "created_at phai la ISO-8601 UTC 24 ky tu. Nhan: {:?}",
        entry.created_at
    );
    assert!(
        entry.created_at.ends_with('Z') && entry.created_at.contains('T'),
        "created_at phai mang `T` va ket thuc bang `Z`. Nhan: {:?}",
        entry.created_at
    );

    // `note` mac dinh: vang mat va rong la CUNG mot dieu (doc-comment GLOSSARY_ENTRY_DDL).
    insert_manual_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
    )
    .expect("chen muc khong ghi chu");
    let global = load_tier(&store).expect("nap lai");
    assert_eq!(global["青丘"].note, "", "note vang mat phai doc lai la chuoi rong");

    drop(store);
    cleanup(&dir);
}

/// 🔴 Một hàng trên đĩa mang `term_origin` KHÔNG khớp `CHECK` làm `load_tier` **trả lỗi**,
/// không rơi về một giá trị mặc định.
///
/// ⚠️ Đây là ca đóng khoảng trống lớn nhất của lượt rà soát #2. `decode_category` /
/// `decode_term_origin` mang một doc-comment dài giải thích vì sao chúng TRẢ LỖI thay vì
/// rơi về `TermOrigin::Manual` — *"xuất xứ đáng tin nhất trong ba giá trị, nên một hàng
/// hỏng sẽ trông Y HỆT một mục người dùng tự nhập tay"*, đúng lớp lỗi AD-47. Nhưng **không
/// một ca nào từng chạy nhánh `Err` đó**: `CHECK` chặn mọi đường ghi của chính module, nên
/// nhánh chỉ tới được từ một đĩa đã trôi. ⇒ Trước ca này, quay `ok_or_else(..)` về
/// `unwrap_or(TermOrigin::Manual)` là một lượt đỏ KHÔNG XẢY RA.
///
/// Fixture dựng bằng **ba bước THẬT đầu tiên** của `GLOBAL_MIGRATIONS` cộng hai bước GIẢ
/// KHÔNG có `CHECK` — cùng khuôn `pinned_contract.rs::an_older_global_database_…` dùng lát
/// cắt của bộ di trú thật thay vì một bản chép tay, để fixture không trôi khỏi sự thật.
///
/// 🔵 **SỬA 2026-08-24 (Story 3.10) — fixture nâng từ BỐN bước lên NĂM.** Bản trước dừng ở
/// bước 4 (đích THẬT của story trước). Từ Story 3.10, đích thật là **5** — bước dựng lại
/// `glossary_entry` với `CHECK` bốn giá trị (xem `GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL`).
/// Nếu fixture vẫn dừng ở 4, `Store::open(StoreSpec::global(db))` cuối bài sẽ tự chạy bước 5
/// THẬT — và bước đó CHÉP MỌI HÀNG qua một `CHECK` mới, làm chính hàng "đã trôi" mà ca này
/// cố ý tạo ra bị `CHECK` chặn NGAY LÚC MỞ KHO (`OpenFailed`), trước khi `load_tier` có cơ
/// hội chạy — ca test đổi ý nghĩa mà không ai chủ định. Bước 5 GIẢ ở đây KHÔNG chạm
/// `glossary_entry` (dựng một bảng đánh dấu vô hại) — đích đạt ĐÚNG 5 mà bảng `glossary_entry`
/// vẫn giữ nguyên hình dạng không `CHECK`, đúng ý định gốc của ca: "một hàng đã trôi tồn tại
/// TRÊN ĐĨA, ở phiên bản HIỆN HÀNH, và `load_tier` (không phải `Store::open`) phải là chỗ
/// bắt được nó".
#[test]
fn a_row_whose_term_origin_drifted_from_the_check_makes_load_tier_refuse_the_whole_tier() {
    use auratranslate_lib::core::store::Migration;

    /// Bước 4 KHÔNG `CHECK` — mô phỏng "một bản ứng dụng cũ/hỏng đã ghi ra hàng này".
    /// Cùng bảy cột, cùng thứ tự, để `load_tier` đọc được tới đúng chỗ nó phải trượt.
    const PERMISSIVE_GLOSSARY_DDL: &str = "\
CREATE TABLE glossary_entry (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  source_term  TEXT    NOT NULL,
  translation  TEXT,
  note         TEXT    NOT NULL DEFAULT '',
  category     TEXT    NOT NULL,
  term_origin  TEXT    NOT NULL,
  created_at   TEXT    NOT NULL
);";

    /// Bước 5 GIẢ — chỉ để `schema_version()` chạm đích THẬT (5) mà không đưa
    /// `glossary_entry` qua `CHECK` nào. Một bảng đánh dấu vô hại, không ai đọc nó.
    const HARMLESS_MARKER_DDL: &str = "\
CREATE TABLE step_five_marker (id INTEGER PRIMARY KEY);";

    static DRIFTED_LADDER: [Migration; 5] = [
        GLOBAL_MIGRATIONS[0],
        GLOBAL_MIGRATIONS[1],
        GLOBAL_MIGRATIONS[2],
        Migration {
            to_version: 4,
            sql: PERMISSIVE_GLOSSARY_DDL,
        },
        Migration {
            to_version: 5,
            sql: HARMLESS_MARKER_DDL,
        },
    ];

    let dir = temp_dir("term-origin-drifted");
    let db = dir.join("global.db");

    {
        let drifted = Store::open(StoreSpec {
            migrations: &DRIFTED_LADDER,
            ..StoreSpec::global(db.clone())
        })
        .expect("dung fixture khong CHECK");
        assert_eq!(drifted.schema_version(), 5, "fixture phai dung o dich THAT (5)");

        drifted
            .write(|tx: &Transaction<'_>| {
                tx.execute(
                    "INSERT INTO glossary_entry \
                     (source_term, translation, note, category, term_origin, created_at) \
                     VALUES ('慕容', 'Mộ Dung', '', 'person', 'khong-phai-mot-xuat-xu', 'x')",
                    [],
                )?;
                Ok(())
            })
            .expect("mot bang khong CHECK phai nhan hang hong nay");
        drop(drifted);
    }

    // Mo lai bang bo di tru THAT: `user_version` da la 4 nen khong buoc nao chay, va luoc
    // do tren dia giu nguyen hinh dang khong CHECK. Day dung la ca "dia da troi".
    let store = Store::open(StoreSpec::global(db)).expect("mo lai bang bo di tru that");

    let refused = load_tier(&store);
    assert!(
        matches!(refused, Err(StoreError::ReadFailed { .. })),
        "mot `term_origin` la tren dia phai lam CA TANG tu choi nap, khong duoc am tham \
         roi ve TermOrigin::Manual -- mot hang hong khong duoc trong giong mot muc nguoi \
         dung tu go (AD-47). Nhan: {refused:?}"
    );

    drop(store);
    cleanup(&dir);
}

/// 🔴 **`CHECK` MỘT MÌNH** — không có `.trim()` của Rust đứng trước — phải từ chối mọi dạng
/// trắng, cho **cả hai** cột.
///
/// ⚠️ **Vì sao ca này bắt buộc phải tồn tại, và vì sao hai ca dạng trắng ở trên KHÔNG thay
/// nó được.** Cả hai ca đó gọi qua `insert_manual_entry`, thứ cắt khoảng trắng bằng `str::trim()`
/// của Rust TRƯỚC khi chạm SQL. Một chuỗi `"\u{2009}"` bị Rust cắt thành `""`, rồi `CHECK`
/// từ chối `''` — nên chúng xanh **y hệt** dù bảng ký tự trong `GLOSSARY_ENTRY_DDL` có bảy
/// ký tự hay hai mươi lăm. Tức lớp Rust CHE lớp SQL, và suốt lượt rà soát #1 không phép đo
/// nào nhìn thấy 17 điểm mã mà bảng bảy ký tự để lọt.
///
/// ⇒ Ca này ghi thẳng bằng SQL, đúng cửa mà một bản ứng dụng khác hay một lượt sửa tay
/// `.db` sẽ đi, và vì thế nó đo **chính bảng ký tự của hằng DDL**, không đo `insert_manual_entry`.
#[test]
fn the_check_constraint_alone_refuses_every_blank_form_on_both_columns() {
    let dir = temp_dir("check-alone-refuses-blanks");
    let store = open_global(&dir);

    for (label, blank) in every_blank_form() {
        let value = blank.to_owned();
        let refused_translation = store.write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO glossary_entry \
                 (source_term, translation, note, category, term_origin, created_at) \
                 VALUES ('thuat ngu that', ?1, '', 'other', 'manual', 'x')",
                [&value],
            )?;
            Ok(())
        });
        assert!(
            matches!(refused_translation, Err(StoreError::WriteFailed { .. })),
            "`CHECK` mot minh phai tu choi translation dang trang '{label}' -- bang ky tu \
             cua GLOSSARY_ENTRY_DDL con thieu diem ma nay. Nhan: {refused_translation:?}"
        );

        let value = blank.to_owned();
        let refused_source = store.write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO glossary_entry \
                 (source_term, translation, note, category, term_origin, created_at) \
                 VALUES (?1, 'ban dich that', '', 'other', 'manual', 'x')",
                [&value],
            )?;
            Ok(())
        });
        assert!(
            matches!(refused_source, Err(StoreError::WriteFailed { .. })),
            "`CHECK` mot minh phai tu choi source_term dang trang '{label}'. \
             Nhan: {refused_source:?}"
        );
    }

    let rows: i64 = store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM glossary_entry", [], |r| r.get(0)))
        .expect("dem hang");
    assert_eq!(rows, 0, "khong luot ghi bi tu choi nao duoc de lai mot hang");

    // Doi chung duong: noi dung THAT bao quanh boi khoang trang bien van di qua duoc, tuc
    // bang ky tu khong rong tay den muc cam ca du lieu hop le.
    let accepted = store.write(|tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO glossary_entry \
             (source_term, translation, note, category, term_origin, created_at) \
             VALUES (' 慕容 ', ' Mộ Dung ', '', 'person', 'manual', 'x')",
            [],
        )?;
        Ok(())
    });
    assert!(
        accepted.is_ok(),
        "noi dung that bao quanh boi khoang trang bien phai DI QUA -- `CHECK` chi cam \
         chuoi TRANG HOAN TOAN. Nhan: {accepted:?}"
    );

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.2 — Bảng chờ ứng viên tách hẳn khỏi Glossary, I/O & Edge-Case Matrix
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ `glossary_candidate` chỉ tồn tại ở `project.db` (§Never: "Bảng ứng viên ở
// `global.db`"), nên MỌI ca dưới đây dùng `open_project`, không `open_global`.

/// Hàng 1 — quét sinh ứng viên mới: hàng mới, `resolution` NULL, có mặt trong
/// `pending_candidates`.
#[test]
fn inserting_a_new_candidate_makes_it_visible_in_pending_candidates() {
    let dir = temp_dir("candidate-insert-visible");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien moi");

    let pending = pending_candidates(&store).expect("nap bang cho");
    assert_eq!(pending.len(), 1, "ung vien vua chen phai co mat trong bang cho");
    assert_eq!(pending[0].id, id);
    assert_eq!(pending[0].source_term, "慕容");
    assert_eq!(pending[0].candidate_origin, CandidateOrigin::ImportScan);
    assert_eq!(pending[0].resolution, None, "ung vien moi phai o resolution = NULL");
    assert!(pending[0].is_pending());

    drop(store);
    cleanup(&dir);
}

/// 🔴 **THÊM 2026-08-20 (lượt rà soát ba lớp).** `pending_candidates` chưa từng chạy với
/// NHIỀU HƠN một hàng chờ trước ca này — mọi ca khác để lại 0 hoặc 1 hàng, nên vòng lặp
/// `while let Some(row) = rows.next()?` bên trong nó chưa bao giờ LẶP. Một lỗi chỉ lộ ra ở
/// lần lặp thứ hai trở đi (ví dụ tái sử dụng một biến ngoài vòng lặp) sẽ không cổng nào bắt
/// được nếu thiếu ca này.
///
/// 🔵 **SỬA 2026-08-24 (Story 3.8, vòng rà ba lớp) — thông điệp `assert_eq!` khai SAI thứ
/// tự.** Bản trước viết `"thu tu khai la ORDER BY source_term"` — mệnh đề đó hết đúng từ khi
/// `pending_candidates` đổi sang `ORDER BY occurrence_count DESC, id ASC` (cùng story). Ca
/// này VẪN xanh sau lượt đổi vì `insert_candidate` (số ít, dùng ở đây) không nhận
/// `occurrence_count` — cột đó rơi về `DEFAULT 0` cho mọi hàng, nên phép sắp suy biến về
/// đúng mốc phụ `id ASC`, và `id` tăng dần trùng khớp thứ tự CHÈN (`alpha` chèn trước
/// `gamma`, hàng `beta` bị bỏ ở giữa không đổi thứ tự `id` của hai hàng còn lại). Ca này vì
/// thế đo đúng NHÁNH ĐỒNG HẠNG của `ORDER BY` (mọi hàng cùng `occurrence_count = 0`), không
/// đo tiêu chí chính — nhánh tiêu chí chính (tần suất khác nhau thắng cả `id` lớn hơn) đã có
/// ca riêng ở `glossary_commands_contract.rs::
/// glossary_pending_candidates_orders_by_occurrence_count_desc_then_id_asc_for_ties`. Sửa
/// thông điệp cho đúng thay vì đổi fixture — mục đích GỐC của ca này (vòng lặp NHIỀU hàng,
/// xem doc-comment trên) không cần tần suất khác nhau để đúng mục đích đó.
#[test]
fn pending_candidates_lists_every_still_pending_row_in_the_declared_order() {
    let dir = temp_dir("candidate-pending-multiple-rows");
    let store = open_project(&dir);

    let id_alpha =
        insert_candidate(&store, "alpha", CandidateOrigin::ImportScan).expect("chen alpha");
    let id_beta =
        insert_candidate(&store, "beta", CandidateOrigin::ReviewHarvest).expect("chen beta");
    let id_gamma =
        insert_candidate(&store, "gamma", CandidateOrigin::ImportScan).expect("chen gamma");

    reject_candidate(&store, id_beta).expect("bo beta");

    let pending = pending_candidates(&store).expect("nap bang cho");
    assert_eq!(
        pending.len(),
        2,
        "hang da bo (beta) phai roi danh sach, hai hang con lai (alpha, gamma) phai co mat. \
         Nhan: {pending:?}"
    );
    assert_eq!(
        pending[0].id, id_alpha,
        "thu tu khai la ORDER BY occurrence_count DESC, id ASC -- ca hang deu occurrence_count \
         = 0 (mac dinh cua insert_candidate) nen suy bien ve id ASC, va id_alpha < id_gamma \
         dung thu tu chen"
    );
    assert_eq!(pending[0].source_term, "alpha");
    assert_eq!(pending[0].candidate_origin, CandidateOrigin::ImportScan);
    assert_eq!(
        pending[1].id, id_gamma,
        "thu tu khai la ORDER BY occurrence_count DESC, id ASC -- cung nhanh dong hang, xem \
         chu thich o assert ngay tren"
    );
    assert_eq!(pending[1].source_term, "gamma");
    assert_eq!(pending[1].candidate_origin, CandidateOrigin::ImportScan);

    drop(store);
    cleanup(&dir);
}

/// Hàng 2 — quét lại một chuỗi **đã bỏ**: bị từ chối, không quay lại bảng chờ.
///
/// `UNIQUE (source_term)` là cơ chế chặn — hàng cũ Ở LẠI trên đĩa (không `DELETE`), nên
/// lượt `insert_candidate` thứ hai va vào đúng chỉ mục đó.
#[test]
fn rescanning_a_rejected_source_term_is_refused_and_does_not_return_to_the_pending_queue() {
    let dir = temp_dir("candidate-rescan-rejected");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien lan dau");
    reject_candidate(&store, id).expect("bo ung vien");

    let rescanned = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan);
    assert!(
        matches!(rescanned, Err(StoreError::WriteFailed { .. })),
        "quet lai mot chuoi da BO phai bi UNIQUE tu choi qua StoreError::WriteFailed. \
         Nhan: {rescanned:?}"
    );

    let pending = pending_candidates(&store).expect("nap bang cho");
    assert!(
        pending.is_empty(),
        "ung vien da bo khong duoc quay lai bang cho, va luot quet lai bi tu choi nen \
         cung khong them hang nao. Nhan: {pending:?}"
    );

    drop(store);
    cleanup(&dir);
}

/// Hàng 3 — quét lại một chuỗi **đã duyệt**: từ chối, cùng một đường với hàng 2.
#[test]
fn rescanning_an_approved_source_term_is_refused_the_same_way() {
    let dir = temp_dir("candidate-rescan-approved");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien lan dau");
    approve_candidate(&store, id, Some("Mộ Dung"), Category::Person).expect("duyet ung vien");

    let rescanned = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan);
    assert!(
        matches!(rescanned, Err(StoreError::WriteFailed { .. })),
        "quet lai mot chuoi da DUYET phai bi UNIQUE tu choi qua StoreError::WriteFailed. \
         Nhan: {rescanned:?}"
    );

    let global = load_tier(&store).expect("nap glossary_entry");
    assert_eq!(global.len(), 1, "luot quet lai bi tu choi khong duoc them mot muc Glossary nao");

    drop(store);
    cleanup(&dir);
}

/// 🔴 **THÊM 2026-08-20 (lượt rà soát ba lớp).** Va chạm `source_term` GIỮA HAI XUẤT XỨ
/// KHÁC NHAU chưa từng được kiểm — cả hai ca `rescanning_*` ở trên dùng `ImportScan` cho
/// CẢ HAI lượt chèn. `idx_glossary_candidate_source_term` là `UNIQUE` trên `source_term`
/// MỘT CỘT, không phải trên cặp `(source_term, candidate_origin)` — nên nó phải chặn đúng
/// ca này: một chuỗi đã có ứng viên `import_scan`, rồi bản thu hoạch review (Epic 8) phát
/// hiện CÙNG chuỗi đó. Đây chính là ca chỉ mục một cột tồn tại để xử.
#[test]
fn a_source_term_already_pending_from_import_scan_collides_with_the_same_term_from_review_harvest()
{
    let dir = temp_dir("candidate-cross-origin-collision");
    let store = open_project(&dir);

    insert_candidate(&store, "慕容", CandidateOrigin::ImportScan).expect("chen ung vien quet");

    let harvested = insert_candidate(&store, "慕容", CandidateOrigin::ReviewHarvest);
    assert!(
        matches!(harvested, Err(StoreError::WriteFailed { .. })),
        "cung mot source_term tu HAI xuat xu khac nhau van phai va vao UNIQUE mot cot. \
         Nhan: {harvested:?}"
    );

    let pending = pending_candidates(&store).expect("nap bang cho");
    assert_eq!(pending.len(), 1, "luot chen thu hai bi tu choi khong duoc them hang nao");
    assert_eq!(
        pending[0].candidate_origin,
        CandidateOrigin::ImportScan,
        "hang dau tien (import_scan) phai o lai NGUYEN VEN, khong bi ghi de boi luot chen \
         thu hai (review_harvest) da bi tu choi"
    );

    drop(store);
    cleanup(&dir);
}

/// Hàng 4 — duyệt một ứng viên `import_scan`: `resolution='approved'` **và**
/// `glossary_entry` mang `term_origin='import_scan'`.
#[test]
fn approving_an_import_scan_candidate_marks_it_approved_and_creates_a_glossary_entry_with_import_scan_origin()
{
    let dir = temp_dir("candidate-approve-import-scan");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien");
    let entry_id = approve_candidate(&store, id, Some("Mộ Dung"), Category::Person)
        .expect("duyet ung vien");

    let resolution: String = store
        .read(|conn| {
            conn.query_row(
                "SELECT resolution FROM glossary_candidate WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
        })
        .expect("doc resolution");
    assert_eq!(resolution, "approved");

    let global = load_tier(&store).expect("nap glossary_entry");
    let entry = &global["慕容"];
    assert_eq!(entry.id, entry_id);
    assert_eq!(entry.translation.as_deref(), Some("Mộ Dung"));
    assert_eq!(entry.category, Category::Person);
    assert_eq!(
        entry.term_origin,
        TermOrigin::ImportScan,
        "term_origin phai suy TU candidate_origin, khong phai mot gia tri mac dinh"
    );

    drop(store);
    cleanup(&dir);
}

/// Hàng 5 — duyệt một ứng viên `review_harvest`: cùng bảng chờ, `glossary_entry.term_origin
/// = 'review_harvest'` — không bảng thứ hai cho xuất xứ này.
#[test]
fn approving_a_review_harvest_candidate_creates_a_glossary_entry_with_review_harvest_origin() {
    let dir = temp_dir("candidate-approve-review-harvest");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "青丘", CandidateOrigin::ReviewHarvest)
        .expect("chen ung vien");
    approve_candidate(&store, id, Some("Thanh Khâu"), Category::Place).expect("duyet ung vien");

    let global = load_tier(&store).expect("nap glossary_entry");
    assert_eq!(global["青丘"].term_origin, TermOrigin::ReviewHarvest);

    drop(store);
    cleanup(&dir);
}

/// Hàng 6 — duyệt để ngỏ bản dịch (`translation = None`): mục Glossary ở *chờ chốt*,
/// không đủ điều kiện chèn (FR114, Story 3.1).
#[test]
fn approving_a_candidate_with_no_translation_leaves_the_glossary_entry_pending_confirmation() {
    let dir = temp_dir("candidate-approve-no-translation");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien");
    approve_candidate(&store, id, None, Category::Person).expect("duyet ma khong chot ban dich");

    let global = load_tier(&store).expect("nap glossary_entry");
    let entry = &global["慕容"];
    assert_eq!(entry.translation, None);
    assert!(!entry.is_confirmed(), "muc vua sinh phai o trang thai cho chot");

    let resolver = ScopeResolver::global_only();
    let eligible = entries_eligible_for_injection(&resolver, &store, None)
        .expect("entries_eligible_for_injection khong loi voi kind hop le");
    assert!(
        eligible.is_empty(),
        "mot muc cho chot khong duoc du dieu kien chen, ke ca khi no vua sinh tu mot \
         ung vien vua duyet"
    );

    drop(store);
    cleanup(&dir);
}

/// Hàng 7 — bỏ một ứng viên: rời `pending_candidates`, hàng còn nguyên trên đĩa.
#[test]
fn rejecting_a_candidate_removes_it_from_the_pending_queue_but_the_row_survives_on_disk() {
    let dir = temp_dir("candidate-reject-survives");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien");
    reject_candidate(&store, id).expect("bo ung vien");

    let pending = pending_candidates(&store).expect("nap bang cho");
    assert!(pending.is_empty(), "ung vien da bo phai roi danh sach cho duyet");

    let row_count: i64 = store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM glossary_candidate", [], |r| r.get(0)))
        .expect("dem hang");
    assert_eq!(row_count, 1, "hang ung vien KHONG bi xoa -- chi doi resolution");

    let resolution: String = store
        .read(|conn| {
            conn.query_row(
                "SELECT resolution FROM glossary_candidate WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
        })
        .expect("doc resolution");
    assert_eq!(resolution, "rejected");

    drop(store);
    cleanup(&dir);
}

/// Hàng 8 — lùi vòng đời (`UPDATE` đưa `resolution` về `NULL`): trigger `RAISE(ABORT)` từ
/// chối, giá trị cũ ở lại nguyên vẹn.
#[test]
fn the_resolution_trigger_refuses_regressing_a_candidate_back_to_pending() {
    let dir = temp_dir("candidate-trigger-no-regress");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien");
    approve_candidate(&store, id, Some("Mộ Dung"), Category::Person).expect("duyet ung vien");

    let regressed = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE glossary_candidate SET resolution = NULL WHERE id = ?1",
            [id],
        )?;
        Ok(())
    });
    assert!(
        matches!(regressed, Err(StoreError::WriteFailed { .. })),
        "trigger glossary_candidate_resolution_is_one_way phai chan luot lui ve NULL. \
         Nhan: {regressed:?}"
    );

    let resolution: String = store
        .read(|conn| {
            conn.query_row(
                "SELECT resolution FROM glossary_candidate WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
        })
        .expect("doc lai hang");
    assert_eq!(
        resolution, "approved",
        "giao dich bi tu choi phai rollback -- resolution da quyet phai o lai nguyen ven"
    );

    drop(store);
    cleanup(&dir);
}

/// 🔴 Vế mà lượt rà soát #1 của story này bắt được: trigger phải chặn **MỌI** hướng đi sau
/// khi đã quyết, không riêng chiều lùi về `NULL` — kể cả đặt lại CÙNG một giá trị.
///
/// `WHEN OLD.resolution IS NOT NULL AND NEW.resolution IS NULL` (khuôn đầu tiên, sao y
/// `glossary_entry_lifecycle_is_one_way`) chỉ chặn đúng ca ở hàng 8. Nó BỎ LỌT chiều
/// NGANG: một `approve_candidate` chạy SAU một `reject_candidate` trên cùng `id` (hoặc
/// ngược lại) đi qua trigger đó sạch sẽ và sinh một hàng `glossary_entry` MỚI — đúng AC
/// trung tâm *"ứng viên bị bỏ không quay lại"* chết trong im lặng. Ca này khoá lớp BẢO
/// ĐẢM (trigger, `WHEN OLD.resolution IS NOT NULL`) bằng SQL thô, bỏ qua lớp Rust đọc
/// trước — nếu chỉ kiểm qua `approve_candidate`/`reject_candidate` (đã có lớp Rust chặn ở
/// trên) thì cổng ở lớp trigger vẫn có thể hỏng mà không ca nào thấy.
#[test]
fn the_resolution_trigger_refuses_every_sideways_move_after_a_decision_not_only_the_regression_to_pending()
{
    let dir = temp_dir("candidate-trigger-no-sideways");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien");
    approve_candidate(&store, id, Some("Mộ Dung"), Category::Person).expect("duyet ung vien");

    // Chiều NGANG: approved -> rejected, thẳng bằng SQL, không qua `reject_candidate`.
    let sideways = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE glossary_candidate SET resolution = 'rejected' WHERE id = ?1",
            [id],
        )?;
        Ok(())
    });
    assert!(
        matches!(sideways, Err(StoreError::WriteFailed { .. })),
        "trigger phai chan chieu NGANG approved -> rejected, khong chi chieu lui ve NULL. \
         Nhan: {sideways:?}"
    );

    // Đặt lại CHÍNH giá trị cũ: approved -> approved. Design Notes noi ro "ke ca sang
    // chinh gia tri cu" -- WHEN chi xet OLD.resolution IS NOT NULL, khong so sanh NEW.
    let same_value = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE glossary_candidate SET resolution = 'approved' WHERE id = ?1",
            [id],
        )?;
        Ok(())
    });
    assert!(
        matches!(same_value, Err(StoreError::WriteFailed { .. })),
        "trigger phai chan CA luot dat lai dung gia tri cu -- 'da quyet thi khong quyet \
         lai', ke ca quyet lai y het. Nhan: {same_value:?}"
    );

    let resolution: String = store
        .read(|conn| {
            conn.query_row(
                "SELECT resolution FROM glossary_candidate WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
        })
        .expect("doc lai hang");
    assert_eq!(resolution, "approved", "ca hai luot bi tu choi phai rollback het");

    drop(store);
    cleanup(&dir);
}

/// Hàng 9(a) — duyệt một `id` không có: không ghi gì, **không** im lặng báo thành công.
#[test]
fn approving_an_unknown_candidate_id_is_refused_and_writes_nothing() {
    let dir = temp_dir("candidate-approve-unknown-id");
    let store = open_project(&dir);

    // Mot ung vien THAT de doi chung "khong hang nao bi dung toi".
    insert_candidate(&store, "青丘", CandidateOrigin::ImportScan).expect("chen ung vien that");

    let result = approve_candidate(&store, 999_999, Some("Thanh Khâu"), Category::Place);
    assert!(
        matches!(result, Err(StoreError::WriteFailed { .. })),
        "duyet mot id khong ton tai phai bi tu choi qua StoreError::WriteFailed, KHONG \
         duoc bao thanh cong. Nhan: {result:?}"
    );

    let global = load_tier(&store).expect("nap glossary_entry");
    assert!(global.is_empty(), "khong duoc co mot muc Glossary nao sinh ra");

    let pending = pending_candidates(&store).expect("nap bang cho");
    assert_eq!(pending.len(), 1, "ung vien that duy nhat khong duoc dung toi");

    drop(store);
    cleanup(&dir);
}

/// Hàng 9(b) — cùng luật cho `reject_candidate`.
#[test]
fn rejecting_an_unknown_candidate_id_is_refused_and_writes_nothing() {
    let dir = temp_dir("candidate-reject-unknown-id");
    let store = open_project(&dir);

    insert_candidate(&store, "青丘", CandidateOrigin::ImportScan).expect("chen ung vien that");

    let result = reject_candidate(&store, 999_999);
    assert!(
        matches!(result, Err(StoreError::WriteFailed { .. })),
        "bo mot id khong ton tai phai bi tu choi qua StoreError::WriteFailed, KHONG duoc \
         bao thanh cong. Nhan: {result:?}"
    );

    let pending = pending_candidates(&store).expect("nap bang cho");
    assert_eq!(pending.len(), 1, "ung vien that duy nhat khong duoc dung toi");

    drop(store);
    cleanup(&dir);
}

/// Hàng 10 — ứng viên rỗng: `CHECK` từ chối, không hàng nào được ghi. Tái dùng
/// `every_blank_form()` — cùng bảng ký tự với `glossary_entry`.
#[test]
fn an_empty_or_whitespace_only_candidate_source_term_is_refused_and_writes_nothing() {
    let dir = temp_dir("candidate-empty-source-term");
    let store = open_project(&dir);

    for (label, blank) in every_blank_form() {
        let result = insert_candidate(&store, blank, CandidateOrigin::ImportScan);
        assert!(
            matches!(result, Err(StoreError::WriteFailed { .. })),
            "source_term trang dang '{label}' phai bi CHECK tu choi qua \
             StoreError::WriteFailed. Nhan: {result:?}"
        );
    }

    let rows: i64 = store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM glossary_candidate", [], |r| r.get(0)))
        .expect("dem hang");
    assert_eq!(rows, 0, "moi luot chen bi tu choi khong duoc de lai mot hang nao");

    drop(store);
    cleanup(&dir);
}

/// AC — ứng viên **đã bỏ**: `approve_candidate` trên đúng `id` đó bị từ chối và **không**
/// hàng `glossary_entry` nào ra đời. Đây là vế THỨ HAI của "ứng viên bị bỏ không quay
/// lại" — vế mà `UNIQUE(source_term)` KHÔNG canh được vì nó chỉ chặn đường
/// `insert_candidate`, không canh đường duyệt lại.
#[test]
fn approving_a_rejected_candidate_is_refused_and_no_glossary_entry_is_born() {
    let dir = temp_dir("candidate-approve-after-reject");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien");
    reject_candidate(&store, id).expect("bo ung vien");

    let approved = approve_candidate(&store, id, Some("Mộ Dung"), Category::Person);
    assert!(
        matches!(approved, Err(StoreError::WriteFailed { .. })),
        "duyet mot ung vien DA BO phai bi tu choi. Nhan: {approved:?}"
    );

    let global = load_tier(&store).expect("nap glossary_entry");
    assert!(
        global.is_empty(),
        "khong duoc co mot muc Glossary nao sinh ra tu mot ung vien da bo. Nhan: {global:?}"
    );

    let resolution: String = store
        .read(|conn| {
            conn.query_row(
                "SELECT resolution FROM glossary_candidate WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
        })
        .expect("doc lai hang");
    assert_eq!(resolution, "rejected", "resolution phai o lai 'rejected', khong doi");

    drop(store);
    cleanup(&dir);
}

/// AC — ứng viên **đã duyệt**: `reject_candidate` trên đúng `id` đó bị từ chối và mục
/// Glossary đã sinh ra vẫn nguyên. Hai bảng không bao giờ được nói ngược nhau.
#[test]
fn rejecting_an_approved_candidate_is_refused_and_the_glossary_entry_survives() {
    let dir = temp_dir("candidate-reject-after-approve");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien");
    approve_candidate(&store, id, Some("Mộ Dung"), Category::Person).expect("duyet ung vien");

    let rejected = reject_candidate(&store, id);
    assert!(
        matches!(rejected, Err(StoreError::WriteFailed { .. })),
        "bo mot ung vien DA DUYET phai bi tu choi. Nhan: {rejected:?}"
    );

    let global = load_tier(&store).expect("nap glossary_entry");
    assert_eq!(
        global["慕容"].translation.as_deref(),
        Some("Mộ Dung"),
        "muc Glossary da sinh ra tu luot duyet truoc phai o lai NGUYEN VEN"
    );

    let resolution: String = store
        .read(|conn| {
            conn.query_row(
                "SELECT resolution FROM glossary_candidate WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
        })
        .expect("doc lai hang");
    assert_eq!(resolution, "approved", "resolution phai o lai 'approved', khong doi");

    drop(store);
    cleanup(&dir);
}

/// 🔴 **GHIM HÀNH VI ĐANG KẸT 2026-08-20 (lượt rà soát ba lớp) — KHÔNG PHẢI HÀNH VI MONG
/// MUỐN.** `deferred-work.md` ghi: một ứng viên trùng `source_term` với một mục
/// `glossary_entry` ĐÃ CÓ SẴN (ví dụ mục đó đến từ `insert_manual_entry` trước khi ứng
/// viên được quét ra) thì `approve_candidate` LUÔN va vào
/// `UNIQUE INDEX idx_glossary_entry_source_term` — ứng viên nằm lại bảng chờ VĨNH VIỄN,
/// không đường nào tự thoát. Chỗ chặn ĐÚNG là lượt quét (`epics.md` §Story 3.5, Story 3.5:
/// quét không được sinh ứng viên cho một chuỗi đã có mục Glossary); `insert_candidate` là
/// API thuần, cố ý KHÔNG tự tra `glossary_entry` trước khi chèn.
///
/// 🔵 **CẬP NHẬT 2026-08-22 (Story 3.5) — MÓN NỢ ĐÃ ĐÓNG Ở ĐƯỜNG SẢN PHẨM, CA NÀY VẪN ĐÚNG
/// Ở TẦNG API THUẦN.** Story 3.5 đóng chỗ chặn ĐÚNG mà doc-comment trên nêu tên: hàm ghi lô
/// mới `insert_import_scan_candidates` lọc `glossary_entry` NGAY trong câu `INSERT`
/// (`WHERE NOT EXISTS`) — một chuỗi ứng viên đã có mục Glossary không bao giờ được CHÈN,
/// nên nó không bao giờ tới được `approve_candidate` để va `UNIQUE`. Xem
/// `an_import_scan_candidate_colliding_with_an_existing_glossary_entry_is_never_inserted`
/// ngay dưới. Ca NÀY vẫn đứng đúng vì `insert_candidate` (singular, gọi trực tiếp ở trên)
/// **không đổi** — nó vẫn là API thuần không tự tra `glossary_entry`, và nay bị
/// `GLOSSARY_ONLY_SURFACE` khoá lại (`glossary_boundary.rs`) đúng vì lý do đó: 0 chỗ gọi
/// sản phẩm nào còn dùng nó theo hình dạng hở này nữa.
#[test]
fn a_candidate_colliding_with_an_existing_manual_glossary_entry_is_stuck_pending_forever_known_gap()
{
    let dir = temp_dir("candidate-stuck-behind-manual-entry");
    let store = open_project(&dir);

    insert_manual_entry(&store, "慕容", Some("Mộ Dung"), "", Category::Person)
        .expect("chen muc nhap tay truoc");

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan).expect(
        "insert_candidate KHONG tra glossary_entry truoc khi chen -- luot chen nay phai \
         THANH CONG",
    );

    let approved = approve_candidate(&store, id, Some("Mo Dung Khac"), Category::Person);
    assert!(
        matches!(approved, Err(StoreError::WriteFailed { .. })),
        "approve_candidate va vao UNIQUE INDEX idx_glossary_entry_source_term cua muc nhap \
         tay -- day la hanh vi DANG KET, co chu Story 3.5 (deferred-work.md), KHONG phai \
         hanh vi mong muon. Nhan: {approved:?}"
    );

    let pending = pending_candidates(&store).expect("nap bang cho");
    assert_eq!(
        pending.len(),
        1,
        "ung vien van nam lai trong bang cho VINH VIEN -- khong duong nao tu thoat"
    );
    assert_eq!(pending[0].id, id);

    let global = load_tier(&store).expect("nap glossary_entry");
    assert_eq!(
        global["慕容"].translation.as_deref(),
        Some("Mộ Dung"),
        "muc nhap tay ban dau phai o lai NGUYEN VEN, khong bi luot duyet that bai dung toi"
    );

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.5 — `insert_import_scan_candidates`, hàm ghi LÔ
// ═════════════════════════════════════════════════════════════════════════════════

fn scan_candidate(source_term: &str, occurrence_count: i64, context_example: &str) -> ScanCandidate {
    ScanCandidate {
        source_term: source_term.to_owned(),
        occurrence_count,
        context_example: context_example.to_owned(),
    }
}

/// I/O Matrix *"Đã có trong Glossary"* — đối chứng dương cho lời hứa của
/// `a_candidate_colliding_with_an_existing_manual_glossary_entry_is_stuck_pending_forever_known_gap`
/// ở trên: đường ghi LÔ không bao giờ tạo ra ca kẹt đó, vì nó không bao giờ CHÈN.
#[test]
fn an_import_scan_candidate_colliding_with_an_existing_glossary_entry_is_never_inserted() {
    let dir = temp_dir("scan-batch-collides-with-entry");
    let store = open_project(&dir);

    insert_manual_entry(&store, "慕容", Some("Mộ Dung"), "", Category::Person)
        .expect("chen muc nhap tay truoc");

    let (inserted, skipped) =
        insert_import_scan_candidates(&store, &[scan_candidate("慕容", 12, "慕容出现了。")])
            .expect("ghi lo");
    assert_eq!((inserted, skipped), (0, 1), "chuoi da co trong glossary_entry phai bi BO QUA, khong chen");

    let pending = pending_candidates(&store).expect("nap bang cho");
    assert!(
        pending.is_empty(),
        "khong hang moi nao duoc chen vao glossary_candidate: {pending:?}"
    );

    drop(store);
    cleanup(&dir);
}

/// I/O Matrix *"Đã từng bị bỏ"* — `ON CONFLICT (source_term) DO NOTHING` không đổi MỘT cột
/// nào của hàng cũ, kể cả khi lượt quét sau mang một `occurrence_count`/`context_example`
/// khác hẳn.
#[test]
fn rescanning_a_rejected_source_term_via_the_batch_writer_changes_nothing_on_the_old_row() {
    let dir = temp_dir("scan-batch-rejected-unchanged");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "萧炎", CandidateOrigin::ImportScan).expect("chen truoc");
    reject_candidate(&store, id).expect("bo di");

    let (inserted, skipped) =
        insert_import_scan_candidates(&store, &[scan_candidate("萧炎", 999, "cau khac han.")])
            .expect("ghi lo");
    assert_eq!((inserted, skipped), (0, 1), "chuoi da bo phai bi BO QUA, khong chen hang thu hai");

    // Hang CU khong doi mot cot nao -- doi chung bang SELECT truoc/sau, dung so hang.
    let (resolution, occurrence_count, context_example) = store
        .read(move |conn| {
            conn.query_row(
                "SELECT resolution, occurrence_count, context_example FROM glossary_candidate WHERE id = ?1",
                [id],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, i64>(1)?,
                        r.get::<_, Option<String>>(2)?,
                    ))
                },
            )
        })
        .expect("doc lai hang cu");
    assert_eq!(resolution, "rejected", "resolution phai o lai 'rejected'");
    assert_eq!(occurrence_count, 0, "occurrence_count cu (0, tu insert_candidate) khong doi");
    assert_eq!(context_example, None, "context_example cu (NULL) khong doi");

    drop(store);
    cleanup(&dir);
}

/// Cặp số `(đã chèn, đã bỏ qua)` phải phân biệt được TỪNG NGUYÊN NHÂN bỏ qua trong CÙNG một
/// lô — một chuỗi trùng `glossary_entry`, một chuỗi trùng `glossary_candidate` đã có, một
/// chuỗi hoàn toàn mới.
#[test]
fn a_batch_reports_inserted_and_skipped_counts_across_mixed_rows() {
    let dir = temp_dir("scan-batch-mixed-counts");
    let store = open_project(&dir);

    insert_manual_entry(&store, "慕容", Some("Mộ Dung"), "", Category::Person)
        .expect("chen muc nhap tay");
    insert_candidate(&store, "旧词", CandidateOrigin::ImportScan).expect("chen ung vien cu");

    let (inserted, skipped) = insert_import_scan_candidates(
        &store,
        &[
            scan_candidate("慕容", 10, "cau mot。"),       // trung glossary_entry
            scan_candidate("旧词", 10, "cau hai。"),       // trung glossary_candidate da co
            scan_candidate("新词", 10, "cau ba。"),        // hoan toan moi
        ],
    )
    .expect("ghi lo");

    assert_eq!(inserted, 1, "chi `新词` la hang moi that su");
    assert_eq!(skipped, 2, "hai hang con lai bi bo qua vi hai ly do khac nhau");

    let pending = pending_candidates(&store).expect("nap bang cho");
    assert_eq!(pending.len(), 2, "'旧词' (cu) va '新词' (moi) phai co mat: {pending:?}");
    let new_row = pending
        .iter()
        .find(|c| c.source_term == "新词")
        .unwrap_or_else(|| panic!("khong thay '新词': {pending:?}"));
    assert_eq!(new_row.occurrence_count, 10);
    assert_eq!(new_row.context_example.as_deref(), Some("cau ba。"));

    drop(store);
    cleanup(&dir);
}

/// I/O Matrix *"Chương rỗng"* ở tầng ghi lô — một lô RỖNG là `(0, 0)`, không phải một lỗi.
#[test]
fn writing_an_empty_batch_returns_zero_inserted_and_zero_skipped() {
    let dir = temp_dir("scan-batch-empty");
    let store = open_project(&dir);

    let (inserted, skipped) = insert_import_scan_candidates(&store, &[]).expect("ghi lo rong");
    assert_eq!((inserted, skipped), (0, 0));

    drop(store);
    cleanup(&dir);
}

/// AC — `approve_candidate` chạy nửa chừng thất bại (`CHECK` bản dịch trắng): cả
/// `resolution` lẫn `glossary_entry` đều KHÔNG đổi — một giao dịch `store.write`.
///
/// ⚠️ Doc-comment của `GLOSSARY_CANDIDATE_DDL`/`approve_candidate` NÊU ca này, nhưng
/// trước ca test này chỉ ca đụng `UNIQUE` được kiểm chạy được (hàng 2/3) -- CHECK bản
/// dịch trắng chưa từng được đo qua đường `approve_candidate`.
#[test]
fn approve_candidate_failing_on_a_blank_translation_leaves_both_tables_unchanged() {
    let dir = temp_dir("candidate-approve-blank-translation-atomic");
    let store = open_project(&dir);

    let id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien");

    for (label, blank) in every_blank_form() {
        // Chi thu cac dang KHONG rong -- `Some("")` va `Some(blank)` deu phai bi CHECK
        // tu choi (rong hoan toan cung nam trong `every_blank_form`, gom ca truong hop
        // dau tien "rong").
        let result = approve_candidate(&store, id, Some(blank), Category::Person);
        assert!(
            matches!(result, Err(StoreError::WriteFailed { .. })),
            "approve_candidate voi translation trang dang '{label}' phai bi CHECK cua \
             GLOSSARY_ENTRY_DDL tu choi. Nhan: {result:?}"
        );
    }

    // Ca vế: resolution KHONG doi (van NULL, tuc con trong pending_candidates)...
    let pending = pending_candidates(&store).expect("nap bang cho");
    assert_eq!(
        pending.len(),
        1,
        "moi luot duyet bi tu choi phai de resolution o lai NULL -- ung vien phai con \
         trong bang cho"
    );
    assert_eq!(pending[0].id, id);

    // ...VA khong mot hang glossary_entry nao duoc chen -- ca hai ve cung mot giao dich.
    let global = load_tier(&store).expect("nap glossary_entry");
    assert!(
        global.is_empty(),
        "khong duoc co mot muc glossary_entry nao sinh ra tu mot luot duyet da bi \
         rollback. Nhan: {global:?}"
    );

    drop(store);
    cleanup(&dir);
}

/// Ca dễ cài ngược nhất của `insert_candidate` — song song ca
/// `a_padded_source_term_collides_with_its_trimmed_twin_instead_of_becoming_a_second_row`
/// mà Story 3.1 dựng cho `insert_manual_entry`.
#[test]
fn a_padded_candidate_source_term_collides_with_its_trimmed_twin_instead_of_becoming_a_second_row()
{
    let dir = temp_dir("candidate-source-term-trim-collides");
    let store = open_project(&dir);

    insert_candidate(&store, "慕容", CandidateOrigin::ImportScan).expect("chen ung vien dau tien");

    let padded = insert_candidate(&store, "  慕容\t", CandidateOrigin::ImportScan);
    assert!(
        matches!(padded, Err(StoreError::WriteFailed { .. })),
        "`  慕容\\t` phai bi cat khoang trang bien o tang Rust roi va vao DUNG mot UNIQUE. \
         Nhan: {padded:?}"
    );

    let pending = pending_candidates(&store).expect("nap bang cho");
    assert_eq!(pending.len(), 1, "khong duoc co hang thu hai");
    assert_eq!(
        pending[0].source_term, "慕容",
        "khoa phai la dang DA CAT, khong phai `  慕容\\t`. Nhan: {:?}",
        pending[0].source_term
    );

    drop(store);
    cleanup(&dir);
}

/// Task list — "mỗi biến thể `Resolution` đi vòng qua `decode_row`". `pending_candidates`
/// chỉ trả hàng `resolution IS NULL`, nên không đường sản phẩm nào khác đọc lại một
/// `resolution` NON-NULL đã ghi — cách duy nhất đo được từ ngoài là qua thông điệp lỗi
/// "đã quyết" của `approve_candidate`/`reject_candidate`, thứ PHẢI đi qua
/// `decode_resolution` để nói đúng giá trị đang có trên đĩa (`approved`/`rejected`) thay
/// vì chỉ lặp lại chuỗi thô không qua kiểm tra.
#[test]
fn each_resolution_variant_round_trips_through_the_already_decided_decode() {
    let dir = temp_dir("candidate-resolution-decode-round-trip");
    let store = open_project(&dir);

    // Bien the Approved: duyet roi duyet lai.
    let approved_id = insert_candidate(&store, "慕容", CandidateOrigin::ImportScan)
        .expect("chen ung vien 1");
    approve_candidate(&store, approved_id, Some("Mộ Dung"), Category::Person)
        .expect("duyet lan dau");
    let reapproved = approve_candidate(&store, approved_id, Some("Mo Dung Khac"), Category::Person);
    let err = format!("{reapproved:?}");
    assert!(
        matches!(reapproved, Err(StoreError::WriteFailed { .. })) && err.contains("approved"),
        "loi 'da quyet' phai giai ma DUNG bien the Approved tu dia, khong chi lap lai \
         chuoi tho khong qua kiem tra. Nhan: {reapproved:?}"
    );

    // Bien the Rejected: bo roi bo lai.
    let rejected_id = insert_candidate(&store, "青丘", CandidateOrigin::ReviewHarvest)
        .expect("chen ung vien 2");
    reject_candidate(&store, rejected_id).expect("bo lan dau");
    let rerejected = reject_candidate(&store, rejected_id);
    let err = format!("{rerejected:?}");
    assert!(
        matches!(rerejected, Err(StoreError::WriteFailed { .. })) && err.contains("rejected"),
        "loi 'da quyet' phai giai ma DUNG bien the Rejected tu dia. Nhan: {rerejected:?}"
    );

    drop(store);
    cleanup(&dir);
}

/// Phép kiểm chéo — bảng ký tự khoảng trắng của `GLOSSARY_ENTRY_DDL` (`source_term`) và
/// `GLOSSARY_CANDIDATE_DDL` phải **trùng từng byte**. Cùng khuôn
/// `dict_lookup.rs::han_ranges_are_verbatim_from_dict_build_char_idx`: so trên VĂN BẢN
/// NGUỒN của hằng, không phải hành vi SQLite — hai bản chép không có cổng là hai bản chép
/// sẽ lệch, và một phép so hành vi không bắt được một `char(...)` gõ sai khi 24 điểm mã
/// còn lại vẫn đúng.
#[test]
fn the_whitespace_char_table_is_byte_identical_between_glossary_entry_and_glossary_candidate_ddl()
{
    // Cắt CHÍNH XÁC đoạn `' ' || char(9) || … || char(12288)` từ CHECK đầu tiên của
    // GLOSSARY_ENTRY_DDL (cột source_term) -- không chép tay lại bảy dòng đó, để phép so
    // này không tự trôi cùng lượt với chính hằng nó đang canh.
    let start_marker = "CHECK (trim(source_term, ";
    let end_marker = ") <> ''),\n  CHECK (";

    let start = GLOSSARY_ENTRY_DDL
        .find(start_marker)
        .expect("GLOSSARY_ENTRY_DDL phai co CHECK cho source_term")
        + start_marker.len();
    let end = GLOSSARY_ENTRY_DDL[start..]
        .find(end_marker)
        .expect("khong tim thay ranh gioi cuoi bang ky tu trang")
        + start;
    let ws_table = &GLOSSARY_ENTRY_DDL[start..end];

    assert!(
        ws_table.len() > 200,
        "doan cat ra qua ngan ({} byte) -- marker da lech khoi hang thuc, phep so nay \
         dang kiem mot chuoi rong",
        ws_table.len()
    );
    assert!(
        ws_table.contains("char(12288)"),
        "doan cat ra thieu diem ma cuoi -- marker da lech"
    );

    assert!(
        GLOSSARY_CANDIDATE_DDL.contains(ws_table),
        "bang ky tu khoang trang cua GLOSSARY_CANDIDATE_DDL da LECH khoi GLOSSARY_ENTRY_DDL \
         -- hai ban chep khong co cong la hai ban chep se lech. Doan can khop:\n{ws_table}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.3 — ba hàm phơi ra mới: `resolve_term_for_quick_add` · `add_manual_term` ·
// `update_manual_term`. I/O & Edge-Case Matrix của story này.
// ═════════════════════════════════════════════════════════════════════════════════

/// Không có vùng chọn khớp ở tầng nào ⇒ `resolve_term_for_quick_add` trả `None` — dải mở
/// chế độ THÊM.
#[test]
fn resolve_term_for_quick_add_returns_none_when_the_term_exists_nowhere() {
    let dir = temp_dir("quick-add-lookup-not-found");
    let store = open_global(&dir);
    let resolver = ScopeResolver::global_only();

    let found = resolve_term_for_quick_add(&resolver, &store, None, "慕容")
        .expect("resolve_term_for_quick_add khong loi voi kind hop le");
    assert!(found.is_none(), "chua tung chen gi, phai la None");

    drop(store);
    cleanup(&dir);
}

/// Cụm có ở tầng Global ⇒ trả về `(GlossaryTier::Global, entry)` — dải mở chế độ SỬA, tầng
/// ghim vào Global.
#[test]
fn resolve_term_for_quick_add_finds_a_global_only_term() {
    let dir = temp_dir("quick-add-lookup-global-only");
    let store = open_global(&dir);
    insert_manual_entry(&store, "慕容", Some("Mộ Dung"), "", Category::Person)
        .expect("chen muc global");
    let resolver = ScopeResolver::global_only();

    let (tier, entry) = resolve_term_for_quick_add(&resolver, &store, None, "慕容")
        .expect("resolve khong loi")
        .expect("phai tim thay");
    assert_eq!(tier, GlossaryTier::Global);
    assert_eq!(entry.translation.as_deref(), Some("Mộ Dung"));

    drop(store);
    cleanup(&dir);
}

/// 🔴 **Ca dễ cài ngược nhất của story** — cụm có ở CẢ hai tầng ⇒ trả về mục **tầng Tác
/// phẩm** (AD-18 ghi đè theo từng thuật ngữ), không phải tầng Global.
#[test]
fn resolve_term_for_quick_add_prefers_the_work_tier_when_both_tiers_have_the_term() {
    let dir = temp_dir("quick-add-lookup-both-tiers");
    let global_store = open_global(&dir);
    let work_store = open_project(&dir);
    insert_manual_entry(&global_store, "慕容", Some("Mộ Dung"), "", Category::Person)
        .expect("chen muc global");
    insert_manual_entry(&work_store, "慕容", Some("Mộ Dong Rieng"), "", Category::Person)
        .expect("chen muc work");
    let resolver = ScopeResolver::with_work(WorkScope {
        work_id: "wk-1".to_owned(),
    });

    let (tier, entry) =
        resolve_term_for_quick_add(&resolver, &global_store, Some(&work_store), "慕容")
            .expect("resolve khong loi")
            .expect("phai tim thay");
    assert_eq!(tier, GlossaryTier::Work, "tang Tac pham phai thang");
    assert_eq!(entry.translation.as_deref(), Some("Mộ Dong Rieng"));

    drop(global_store);
    drop(work_store);
    cleanup(&dir);
}

/// 🔴 **Ca thứ hai dễ cài ngược** — lượt tra KHÔNG lọc `is_confirmed`: một mục *chờ chốt*
/// (`translation IS NULL`) vẫn mở được ở chế độ SỬA, khác hẳn `entries_eligible_for_injection`
/// (vốn LỌC nó ra).
#[test]
fn resolve_term_for_quick_add_finds_a_pending_term_without_filtering_it_out() {
    let dir = temp_dir("quick-add-lookup-pending");
    let store = open_global(&dir);
    insert_manual_entry(&store, "青丘", None, "", Category::Place).expect("chen muc cho chot");
    let resolver = ScopeResolver::global_only();

    let (tier, entry) = resolve_term_for_quick_add(&resolver, &store, None, "青丘")
        .expect("resolve khong loi")
        .expect("mot muc CHO CHOT van phai tim thay duoc");
    assert_eq!(tier, GlossaryTier::Global);
    assert!(!entry.is_confirmed(), "muc phai o nguyen trang thai cho chot");

    // Doi chung: `entries_eligible_for_injection` LOC muc nay ra — hai ham tra loi hai cau
    // hoi khac nhau tren cung du lieu.
    let eligible = entries_eligible_for_injection(&resolver, &store, None).expect("eligible");
    assert!(
        eligible.is_empty(),
        "muc cho chot khong duoc du dieu kien chen prompt"
    );

    drop(store);
    cleanup(&dir);
}

/// Ô nguồn của dải mang khoảng trắng biên (dán nguyên vùng chọn, người dùng chưa gõ thêm gì)
/// vẫn khớp đúng khoá đã trim trên đĩa.
#[test]
fn resolve_term_for_quick_add_trims_the_query_before_looking_up() {
    let dir = temp_dir("quick-add-lookup-trims-query");
    let store = open_global(&dir);
    insert_manual_entry(&store, "慕容", Some("Mộ Dung"), "", Category::Person)
        .expect("chen muc");
    let resolver = ScopeResolver::global_only();

    let found = resolve_term_for_quick_add(&resolver, &store, None, "  慕容\t")
        .expect("resolve khong loi");
    assert!(found.is_some(), "truy van co dem bien phai van khop hang da trim");

    drop(store);
    cleanup(&dir);
}

/// `add_manual_term` ở tầng Global ghi vào đúng `&Store` được truyền làm `global`.
#[test]
fn add_manual_term_at_the_global_tier_writes_to_the_global_store() {
    let dir = temp_dir("quick-add-write-global");
    let store = open_global(&dir);

    add_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
    )
    .expect("them muc o tang Global");

    let global = load_tier(&store).expect("nap tang global");
    assert_eq!(global.len(), 1);
    assert_eq!(global["慕容"].translation.as_deref(), Some("Mộ Dung"));

    drop(store);
    cleanup(&dir);
}

/// `add_manual_term` ở tầng Tác phẩm ghi vào `&Store` của `project.db`, KHÔNG vào
/// `global.db`.
#[test]
fn add_manual_term_at_the_work_tier_writes_to_the_project_store_only() {
    let dir = temp_dir("quick-add-write-work");
    let global_store = open_global(&dir);
    let work_store = open_project(&dir);

    add_manual_term(
        &global_store,
        Some(&work_store),
        GlossaryTier::Work,
        "青丘",
        Some("Thanh Khâu"),
        "",
        Category::Place,
    )
    .expect("them muc o tang Tac pham");

    let global = load_tier(&global_store).expect("nap tang global");
    let work = load_tier(&work_store).expect("nap tang work");
    assert!(global.is_empty(), "global.db khong duoc dung toi");
    assert_eq!(work.len(), 1);
    assert_eq!(work["青丘"].translation.as_deref(), Some("Thanh Khâu"));

    drop(global_store);
    drop(work_store);
    cleanup(&dir);
}

/// Chọn tầng Tác phẩm khi chưa mở Tác phẩm nào (`work = None`) ⇒ từ chối với
/// `GlossaryError::WorkTierUnavailable` — I/O Matrix *"Chọn tầng Tác phẩm khi chưa mở Tác
/// phẩm"*.
#[test]
fn add_manual_term_at_the_work_tier_without_a_work_store_is_refused() {
    let dir = temp_dir("quick-add-write-work-unavailable");
    let store = open_global(&dir);

    let result = add_manual_term(
        &store,
        None,
        GlossaryTier::Work,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
    );
    assert!(
        matches!(result, Err(GlossaryError::WorkTierUnavailable)),
        "phai tu choi voi WorkTierUnavailable. Nhan: {result:?}"
    );

    let global = load_tier(&store).expect("nap tang global");
    assert!(global.is_empty(), "khong duoc ghi gi ca");

    drop(store);
    cleanup(&dir);
}

/// `note` toàn khoảng trắng ghi xuống chuỗi rỗng — một cách biểu diễn duy nhất cho "không có
/// ghi chú" (đóng `deferred-work.md:5380-5385`).
#[test]
fn add_manual_term_trims_a_whitespace_only_note_down_to_the_empty_string() {
    let dir = temp_dir("quick-add-write-note-trim");
    let store = open_global(&dir);

    add_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        "慕容",
        Some("Mộ Dung"),
        "   \u{3000}  ",
        Category::Person,
    )
    .expect("them muc voi ghi chu toan khoang trang");

    let global = load_tier(&store).expect("nap tang global");
    assert_eq!(
        global["慕容"].note, "",
        "ghi chu toan khoang trang phai gon ve chuoi RONG, khong dung nguyen"
    );

    drop(store);
    cleanup(&dir);
}

/// Nguồn hoặc bản dịch toàn khoảng trắng ⇒ `CHECK` từ chối qua đường `add_manual_term`.
#[test]
fn add_manual_term_rejects_a_blank_source_term() {
    let dir = temp_dir("quick-add-write-blank-source");
    let store = open_global(&dir);

    let result = add_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        "\u{3000}",
        Some("Mộ Dung"),
        "",
        Category::Person,
    );
    assert!(
        matches!(result, Err(GlossaryError::Store(StoreError::WriteFailed { .. }))),
        "nguon toan khoang trang phai bi CHECK tu choi. Nhan: {result:?}"
    );

    drop(store);
    cleanup(&dir);
}

/// `update_manual_term` sửa cả ba cột trong một lượt — "sửa có hiệu lực ngay" (Story 3.9
/// dùng lại đúng khuôn này).
#[test]
fn update_manual_term_rewrites_translation_note_and_category_together() {
    let dir = temp_dir("quick-add-update-success");
    let store = open_global(&dir);
    let id = add_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        "慕容",
        Some("Mộ Dung"),
        "ghi chu cu",
        Category::Person,
    )
    .expect("them muc");

    update_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        id,
        Some("Mộ Dung Mới"),
        "ghi chu moi",
        Category::DomainTerm,
    )
    .expect("sua muc");

    let global = load_tier(&store).expect("nap tang global");
    let entry = &global["慕容"];
    assert_eq!(entry.translation.as_deref(), Some("Mộ Dung Mới"));
    assert_eq!(entry.note, "ghi chu moi");
    assert_eq!(entry.category, Category::DomainTerm);

    drop(store);
    cleanup(&dir);
}

/// Sửa một `id` đã biến mất (mục bị xoá giữa chừng — mô phỏng bằng một `id` chưa từng tồn
/// tại) ⇒ không ghi gì, báo lỗi đọc được `GlossaryError::EntryMissing` — I/O Matrix.
#[test]
fn update_manual_term_on_a_vanished_id_is_rejected_and_changes_nothing() {
    let dir = temp_dir("quick-add-update-missing-id");
    let store = open_global(&dir);
    add_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        "青丘",
        None,
        "",
        Category::Place,
    )
    .expect("chen mot muc that de doi chung");

    let result = update_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        999_999,
        Some("Thanh Khâu"),
        "",
        Category::Place,
    );
    assert!(
        matches!(result, Err(GlossaryError::EntryMissing)),
        "id khong ton tai phai tra EntryMissing. Nhan: {result:?}"
    );

    let global = load_tier(&store).expect("nap tang global");
    assert_eq!(global.len(), 1, "khong hang moi nao duoc tao ra");
    assert!(
        !global["青丘"].is_confirmed(),
        "muc that duy nhat trong kho khong duoc dung toi"
    );

    drop(store);
    cleanup(&dir);
}

/// `update_manual_term` ở tầng Tác phẩm khi chưa mở Tác phẩm nào ⇒ `WorkTierUnavailable`,
/// cùng luật `add_manual_term`.
#[test]
fn update_manual_term_at_the_work_tier_without_a_work_store_is_refused() {
    let dir = temp_dir("quick-add-update-work-unavailable");
    let store = open_global(&dir);
    let id = add_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
    )
    .expect("them muc o Global de co mot id that");

    // `id` nay chi ton tai trong `global.db` -- goi voi `tier = Work` phai bi tu choi truoc
    // ca khi cham toi cau UPDATE, vi khong co `&Store` nao cua `project.db`.
    let result = update_manual_term(
        &store,
        None,
        GlossaryTier::Work,
        id,
        Some("Mo Dung Khac"),
        "",
        Category::Person,
    );
    assert!(
        matches!(result, Err(GlossaryError::WorkTierUnavailable)),
        "phai tu choi voi WorkTierUnavailable. Nhan: {result:?}"
    );

    drop(store);
    cleanup(&dir);
}

/// `update_manual_term` cũng cắt khoảng trắng biên của `note`/`translation` — cùng lý do
/// `add_manual_term`.
#[test]
fn update_manual_term_trims_translation_and_note() {
    let dir = temp_dir("quick-add-update-trims");
    let store = open_global(&dir);
    let id = add_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        "慕容",
        None,
        "",
        Category::Person,
    )
    .expect("them muc cho chot");

    update_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        id,
        Some("  Mộ Dung  "),
        "   ",
        Category::Person,
    )
    .expect("sua muc");

    let global = load_tier(&store).expect("nap tang global");
    let entry = &global["慕容"];
    assert_eq!(entry.translation.as_deref(), Some("Mộ Dung"));
    assert_eq!(entry.note, "");

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.3 (Ice bắt, 2026-08-20) — hai bản chép của cùng một hợp đồng dây không được lệch
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 `Category`/`GlossaryTier` mỗi kiểu có HAI cách viết ra dây: `as_str()`/`from_wire()`
/// (đường ĐĨA — `INSERT`/`UPDATE`/`load_tier`) và `#[serde(rename = …)]` (đường THAM SỐ IPC
/// của `commands::glossary::wire`, `serde` giải mã trực tiếp, không đi qua `from_wire`).
/// Không cổng nào từng đối chiếu hai bản chép đó — đúng câu mà chính doc-comment của
/// `the_whitespace_char_table_is_byte_identical_between_glossary_entry_and_glossary_candidate_ddl`
/// đã nói: *"hai bản chép không có cổng là hai bản chép sẽ lệch"*.
///
/// Kiểm cả hai chiều cho MỖI biến thể, cùng khuôn `scope_contract.rs::the_kind_table_has_
/// every_variant`:
/// ① đường ĐĨA là nghịch đảo của chính nó — `from_wire(as_str(v)) == Some(v)`;
/// ② đường DÂY giải mã ĐÚNG chuỗi mà `as_str()` sinh ra thành ĐÚNG biến thể —
///    `serde_json::from_str::<T>(quote(as_str(v))) == Ok(v)`. Nếu `#[serde(rename)]` gõ sai
///    một chữ, ca này đỏ dù ① vẫn xanh (① không hề chạm `serde`).
#[test]
fn category_and_glossary_tier_wire_strings_agree_between_as_str_and_serde_rename() {
    for category in [
        Category::Person,
        Category::Place,
        Category::DomainTerm,
        Category::Other,
    ] {
        assert_eq!(
            Category::from_wire(category.as_str()),
            Some(category),
            "duong DIA: from_wire(as_str({category})) phai la chinh no"
        );

        let quoted = format!("\"{}\"", category.as_str());
        let decoded: Category = serde_json::from_str(&quoted).unwrap_or_else(|e| {
            panic!(
                "duong DAY: serde khong giai ma duoc chuoi {quoted} ma as_str({category}) sinh \
                 ra -- #[serde(rename)] va as_str()/from_wire() da LECH nhau. Loi serde: {e}"
            )
        });
        assert_eq!(
            decoded, category,
            "duong DAY giai ma dung chuoi {quoted} nhung ra SAI bien the ({decoded} thay vi \
             {category}) -- #[serde(rename)] tro nham sang mot ten khac"
        );
    }

    for tier in [GlossaryTier::Global, GlossaryTier::Work] {
        assert_eq!(
            GlossaryTier::from_wire(tier.as_str()),
            Some(tier),
            "duong DIA: from_wire(as_str({tier})) phai la chinh no"
        );

        let quoted = format!("\"{}\"", tier.as_str());
        let decoded: GlossaryTier = serde_json::from_str(&quoted).unwrap_or_else(|e| {
            panic!(
                "duong DAY: serde khong giai ma duoc chuoi {quoted} ma as_str({tier}) sinh ra \
                 -- #[serde(rename)] va as_str()/from_wire() da LECH nhau. Loi serde: {e}"
            )
        });
        assert_eq!(
            decoded, tier,
            "duong DAY giai ma dung chuoi {quoted} nhung ra SAI bien the ({decoded} thay vi \
             {tier}) -- #[serde(rename)] tro nham sang mot ten khac"
        );
    }
}

/// Đối chứng: một chuỗi KHÔNG khớp bất kỳ biến thể nào phải bị serde TỪ CHỐI (không đoán),
/// đúng khuôn `ScopeKind::from_wire("khong_ton_tai") == None` của `scope_contract.rs`.
#[test]
fn serde_rejects_an_unknown_category_or_tier_string_instead_of_guessing() {
    assert!(
        serde_json::from_str::<Category>("\"khong_ton_tai\"").is_err(),
        "mot chuoi la phai bi serde TU CHOI, khong duoc doan ve bien the nao"
    );
    assert!(
        serde_json::from_str::<GlossaryTier>("\"khong_ton_tai\"").is_err(),
        "mot chuoi la phai bi serde TU CHOI, khong duoc doan ve bien the nao"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.9 — Quản lý Glossary: `list_all_entries` · `delete_manual_term` ·
// `promote_to_global`
// ═════════════════════════════════════════════════════════════════════════════════

fn work_scope() -> WorkScope {
    WorkScope {
        work_id: "0192f3c4-5678-4abc-8def-0123456789ab".to_owned(),
    }
}

/// 🔴 **Ca trung tâm của story** — mục Global bị một mục Work cùng `source_term` che vẫn
/// PHẢI có mặt trong `list_all_entries`, mang cờ `is_shadowed = true`. §Intent của spec:
/// bỏ hàng này đi là làm một mục Global CÓ THẬT biến mất khỏi màn hình quản lý mà không
/// dòng nào giải thích.
#[test]
fn list_all_entries_includes_a_shadowed_global_row_flagged_true() {
    let dir = temp_dir("list-shadowed");
    let global_store = open_global(&dir);
    let work_store = open_project(&dir);

    insert_manual_entry(&global_store, "慕容", Some("Mộ Dung"), "", Category::Person)
        .expect("chen muc global");
    insert_manual_entry(&work_store, "慕容", Some("Mộ Dong"), "", Category::Person)
        .expect("chen muc work");

    let resolver = ScopeResolver::with_work(work_scope());
    let rows = list_all_entries(&resolver, &global_store, Some(&work_store))
        .expect("list_all_entries khong loi voi kind hop le");

    assert_eq!(
        rows.len(),
        2,
        "dung HAI hang cho cung mot source_term: hang THANG (Work) va hang BI CHE (Global). \
         Nhan: {rows:?}"
    );

    let winner = rows
        .iter()
        .find(|(tier, _, shadowed)| *tier == GlossaryTier::Work && !shadowed)
        .expect("phai co hang THANG o tang Work");
    assert_eq!(winner.1.translation.as_deref(), Some("Mộ Dong"));

    let shadowed_row = rows
        .iter()
        .find(|(tier, _, shadowed)| *tier == GlossaryTier::Global && *shadowed)
        .expect("phai co hang BI CHE o tang Global, mang co is_shadowed=true");
    assert_eq!(shadowed_row.1.translation.as_deref(), Some("Mộ Dung"));

    // Đối chứng: hàng thắng KHÔNG mang cờ, hàng Global KHÔNG bị che (không tồn tại) khi
    // không có gì che nó — kiểm cả hai chiều để cổng không xanh giả trên "mọi hàng đều
    // mang is_shadowed=true" hay ngược lại.
    assert!(!winner.2, "hang THANG khong duoc mang co is_shadowed");

    drop(global_store);
    drop(work_store);
    cleanup(&dir);
}

/// `list_all_entries` **không lọc** `is_confirmed` — khác `entries_eligible_for_injection`.
/// Một mục chờ chốt phải hiện ra để người dùng SỬA/XOÁ nó.
#[test]
fn list_all_entries_includes_pending_entries_unlike_entries_eligible_for_injection() {
    let dir = temp_dir("list-includes-pending");
    let store = open_global(&dir);

    insert_manual_entry(&store, "青丘", None, "", Category::Place).expect("chen muc cho chot");

    let resolver = ScopeResolver::global_only();
    let rows =
        list_all_entries(&resolver, &store, None).expect("list_all_entries khong loi voi kind hop le");

    assert_eq!(rows.len(), 1, "muc cho chot phai co mat khi liet ke ca hai tang");
    assert!(!rows[0].1.is_confirmed(), "muc van o trang thai cho chot");
    assert!(!rows[0].2, "mot tang duy nhat khong co gi de che no");

    // Đối chứng: `entries_eligible_for_injection` KHÔNG trả mục này — hai hàm phục vụ hai
    // câu hỏi khác nhau (§Design Notes của Story 3.3).
    let eligible = entries_eligible_for_injection(&resolver, &store, None)
        .expect("entries_eligible_for_injection khong loi voi kind hop le");
    assert!(eligible.is_empty(), "mot muc cho chot khong duoc du dieu kien chen");

    drop(store);
    cleanup(&dir);
}

/// 🔴 **Xoá một mục ĐÃ CHỐT là hợp lệ** *(Ice chốt 2026-08-24)* — trigger một chiều chỉ
/// khớp `UPDATE OF translation`, không bao giờ khớp `DELETE`.
#[test]
fn delete_manual_term_removes_an_already_confirmed_entry() {
    let dir = temp_dir("delete-confirmed");
    let store = open_global(&dir);

    let id = insert_manual_entry(&store, "青丘", Some("Thanh Khâu"), "", Category::Place)
        .expect("chen muc da chot");

    delete_manual_term(&store, None, GlossaryTier::Global, id)
        .expect("xoa mot muc DA CHOT phai THANH CONG");

    let global = load_tier(&store).expect("nap lai tang global");
    assert!(!global.contains_key("青丘"), "muc phai bien mat sau khi xoa");

    drop(store);
    cleanup(&dir);
}

/// Xoá một `id` không khớp hàng nào ⇒ `GlossaryError::EntryMissing`, không phải một
/// `Ok(())` rỗng im lặng cho một lượt ghi 0 hàng.
#[test]
fn delete_manual_term_rejects_an_id_that_does_not_exist() {
    let dir = temp_dir("delete-missing-id");
    let store = open_global(&dir);

    let real_id =
        insert_manual_entry(&store, "青丘", Some("Thanh Khâu"), "", Category::Place).expect("chen muc that");

    let result = delete_manual_term(&store, None, GlossaryTier::Global, real_id + 1000);
    assert_eq!(
        result,
        Err(GlossaryError::EntryMissing),
        "id la khong ton tai phai bi TU CHOI. Nhan: {result:?}"
    );

    let global = load_tier(&store).expect("nap lai tang global");
    assert_eq!(global.len(), 1, "muc that khong bi dung toi boi mot lenh xoa id sai");

    drop(store);
    cleanup(&dir);
}

/// Chọn tầng Tác phẩm khi chưa mở Tác phẩm nào ⇒ `GlossaryError::WorkTierUnavailable` —
/// cùng khuôn `add_manual_term`/`update_manual_term`.
#[test]
fn delete_manual_term_at_the_work_tier_without_a_work_store_is_rejected() {
    let dir = temp_dir("delete-work-tier-unavailable");
    let store = open_global(&dir);

    let result = delete_manual_term(&store, None, GlossaryTier::Work, 1);
    assert_eq!(result, Err(GlossaryError::WorkTierUnavailable));

    drop(store);
    cleanup(&dir);
}

/// Đẩy tầng, đích trống: `INSERT global` rồi `DELETE work`; hàng đổi tầng và giữ nguyên
/// `translation`/`note`/`category`/`term_origin`.
#[test]
fn promote_to_global_moves_an_entry_when_the_destination_is_empty() {
    let dir = temp_dir("promote-empty-destination");
    let global_store = open_global(&dir);
    let work_store = open_project(&dir);

    let id = insert_manual_entry(&work_store, "青丘", Some("Thanh Khâu"), "ghi chu", Category::Place)
        .expect("chen muc work");

    promote_to_global(&global_store, &work_store, id).expect("day tang phai THANH CONG khi dich trong");

    let global = load_tier(&global_store).expect("nap tang global");
    let promoted = global.get("青丘").expect("muc phai xuat hien o tang Global sau khi day");
    assert_eq!(promoted.translation.as_deref(), Some("Thanh Khâu"));
    assert_eq!(promoted.note, "ghi chu");
    assert_eq!(promoted.category, Category::Place);
    assert_eq!(
        promoted.term_origin,
        TermOrigin::Manual,
        "muc nhap tay o tang Work phai giu nguyen term_origin sau khi day"
    );

    let work = load_tier(&work_store).expect("nap lai tang work");
    assert!(!work.contains_key("青丘"), "muc phai bien khoi tang Work sau khi day thanh cong");

    drop(global_store);
    drop(work_store);
    cleanup(&dir);
}

/// 🔴 Đẩy tầng, đích ĐÃ CÓ `source_term` này ⇒ `GlossaryError::GlobalTermExists`, **0 lượt
/// ghi**: cả hai mục giữ nguyên giá trị cũ, không ghi đè.
#[test]
fn promote_to_global_rejects_and_writes_nothing_when_the_destination_already_has_the_term() {
    let dir = temp_dir("promote-destination-exists");
    let global_store = open_global(&dir);
    let work_store = open_project(&dir);

    insert_manual_entry(&global_store, "青丘", Some("Thanh Khau Cu"), "", Category::Place)
        .expect("chen muc global truoc");
    let work_id = insert_manual_entry(&work_store, "青丘", Some("Thanh Khâu"), "", Category::Place)
        .expect("chen muc work");

    let result = promote_to_global(&global_store, &work_store, work_id);
    assert_eq!(
        result,
        Err(GlossaryError::GlobalTermExists),
        "dich da co source_term nay phai bi TU CHOI bang mot loi CO TEN. Nhan: {result:?}"
    );

    let global = load_tier(&global_store).expect("nap lai tang global");
    assert_eq!(
        global["青丘"].translation.as_deref(),
        Some("Thanh Khau Cu"),
        "muc Global cu KHONG duoc ghi de"
    );

    let work = load_tier(&work_store).expect("nap lai tang work");
    assert_eq!(
        work["青丘"].translation.as_deref(),
        Some("Thanh Khâu"),
        "muc Work van con nguyen -- 0 luot ghi nghia la KHONG bi xoa"
    );

    drop(global_store);
    drop(work_store);
    cleanup(&dir);
}

/// `id` không khớp hàng nào ở tầng Work ⇒ `GlossaryError::EntryMissing`, và `global.db`
/// KHÔNG bị chạm — bước đọc trượt trước khi bất kỳ lượt ghi nào chạy.
#[test]
fn promote_to_global_rejects_an_id_that_does_not_exist_at_the_work_tier() {
    let dir = temp_dir("promote-missing-id");
    let global_store = open_global(&dir);
    let work_store = open_project(&dir);

    let result = promote_to_global(&global_store, &work_store, 999_999);
    assert_eq!(result, Err(GlossaryError::EntryMissing), "id la khong ton tai o tang Work");

    let global = load_tier(&global_store).expect("nap tang global");
    assert!(global.is_empty(), "global.db khong duoc dung toi khi id da khong ton tai");

    drop(global_store);
    drop(work_store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.10 — bước di trú dựng lại `glossary_entry` (giá trị `term_origin` thứ tư)
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **Ca di trú trung tâm của Story 3.10** — một kho ở phiên bản CŨ (`CHECK` ba giá trị,
/// bước 4) có hàng dữ liệu VÀ một hàng ĐÃ BỊ XOÁ, di trú lên bước 5, rồi bốn mệnh đề:
/// 1. Hàng còn sống vẫn đủ.
/// 2. `id` của chúng KHÔNG đổi.
/// 3. `UNIQUE INDEX` và trigger một chiều (AD-36) còn đỏ được — không phải chỉ "còn tồn
///    tại", mà THẬT SỰ còn từ chối đúng ca chúng sinh ra để chặn.
/// 4. `id` cấp TIẾP THEO lớn hơn MỌI `id` từng cấp — id đã xoá (ở giữa) không được phát lại.
///
/// Hàng đã xoá là thứ làm mệnh đề (4) kiểm được — VÀ nó phải là hàng **`id` CAO NHẤT**,
/// không phải một hàng ở giữa. 🔴 **Đo được (đối chứng gỡ chỗ nối, xem §Verification):
/// xoá một hàng GIỮA không kiểm được gì.** `INSERT … SELECT id, …` tường minh vào bảng mới
/// tự nâng mốc `sqlite_sequence` của NÓ lên đúng `id` LỚN NHẤT còn lại trong dữ liệu chép —
/// nếu hàng bị xoá không phải hàng `id` cao nhất, mốc tự nhiên đó đã đúng sẵn mà không cần
/// bước mang mốc riêng, và ca này xanh dù dòng `UPDATE sqlite_sequence …` bị gỡ khỏi
/// migration. Chỉ xoá đúng hàng `id` cao nhất mới buộc bước mang mốc phải chạy thật.
///
/// Ladder CŨ dùng **bốn bước ĐẦU THẬT** của `GLOBAL_MIGRATIONS` (không chép tay DDL — cùng
/// khuôn `store_contract.rs::spec_with_migrations`), nên fixture không trôi khỏi sự thật
/// khi một bước sớm hơn đổi hình dạng.
#[test]
fn migrating_past_the_old_three_value_check_keeps_ids_and_carries_the_watermark_forward() {
    use auratranslate_lib::core::store::Migration;

    let dir = temp_dir("term-origin-migration-watermark");
    let db = dir.join("global.db");

    let old_ladder: &'static [Migration] = &GLOBAL_MIGRATIONS[..4];

    {
        let old = Store::open(StoreSpec {
            migrations: old_ladder,
            ..StoreSpec::global(db.clone())
        })
        .expect("mo kho o dich CU (CHECK ba gia tri)");
        assert_eq!(old.schema_version(), 4, "fixture phai dung o dich CU");

        old.write(|tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO glossary_entry \
                 (source_term, translation, note, category, term_origin, created_at) \
                 VALUES ('a', NULL, '', 'person', 'manual', 'x')",
                [],
            )?;
            tx.execute(
                "INSERT INTO glossary_entry \
                 (source_term, translation, note, category, term_origin, created_at) \
                 VALUES ('b', NULL, '', 'person', 'manual', 'x')",
                [],
            )?;
            tx.execute(
                "INSERT INTO glossary_entry \
                 (source_term, translation, note, category, term_origin, created_at) \
                 VALUES ('c', NULL, '', 'person', 'manual', 'x')",
                [],
            )?;
            Ok(())
        })
        .expect("chen ba hang (id 1, 2, 3)");

        old.write(|tx: &Transaction<'_>| {
            tx.execute("DELETE FROM glossary_entry WHERE source_term = 'c'", [])
        })
        .expect(
            "xoa hang CO ID CAO NHAT (3) -- id nay phai bi 'dot', khong bao gio duoc phat lai. \
             Xem doc-comment cua ca nay: PHAI la hang cao nhat, khong phai mot hang giua",
        );

        // 🔴 P8 (vòng rà ba lớp, 2026-08-25) — bước dựng lại KHAI (bằng doc-comment) rằng
        // `idx_glossary_entry_source_term` + `glossary_entry_lifecycle_is_one_way` là HAI
        // đối tượng DUY NHẤT gắn vào `glossary_entry`. Không gì kiểm lời khai đó trước ca
        // này — một index/trigger THỨ BA do một story sau thêm vào (mà bước dựng lại quên
        // tạo lại) sẽ bị `DROP TABLE` nuốt mất trong im lặng, và không cổng nào đỏ. Chụp ảnh
        // TRƯỚC lượt di trú thật để chính lời khai này trở thành một mệnh đề kiểm được: nếu
        // số đo ở đây một ngày nào đó không còn là hai, ca này phải ĐỎ trước khi ai tin
        // tưởng bước dựng lại vẫn tạo lại "đúng những gì cần tạo lại".
        let before_rebuild: Vec<(String, String)> = old
            .read(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT type, name FROM sqlite_master \
                     WHERE tbl_name = 'glossary_entry' AND type IN ('index', 'trigger') \
                     ORDER BY type, name",
                )?;
                stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect()
            })
            .expect("doc sqlite_master TRUOC luot di tru");
        assert_eq!(
            before_rebuild,
            vec![
                ("index".to_owned(), "idx_glossary_entry_source_term".to_owned()),
                ("trigger".to_owned(), "glossary_entry_lifecycle_is_one_way".to_owned()),
            ],
            "loi khai 'hai doi tuong DUY NHAT gan vao glossary_entry' sai TRUOC ca khi buoc di \
             tru chay -- fixture da troi khoi GLOSSARY_ENTRY_DDL that, sua fixture chu khong \
             sua menh de nay"
        );

        drop(old);
    }

    // Mo lai bang bo di tru THAT (GLOBAL_MIGRATIONS day du) -- buoc 5 phai chay.
    //
    // 🔵 SUA (2026-08-27, phan quyet Ice #1) -- dich chuyen tu 5 len 6: buoc 6 them bang
    // `library_orphan` (khong cham `glossary_entry`), nen no chay THEM sau buoc 5 ma khong
    // anh huong menh de cua ca nay.
    let migrated = Store::open(StoreSpec::global(db)).expect("mo lai sau khi di tru");
    assert_eq!(migrated.schema_version(), 6, "buoc 5 VA 6 phai da chay");

    // (1) + (2) hang con song du, va id KHONG doi ('a' van la 1, 'b' van la 2 -- khong bi
    // don lai).
    let remaining: Vec<(i64, String)> = migrated
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id, source_term FROM glossary_entry ORDER BY id")?;
            stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect()
        })
        .expect("doc hang con lai");
    assert_eq!(
        remaining,
        vec![(1, "a".to_owned()), (2, "b".to_owned())],
        "di tru phai giu DU hang con song VA giu NGUYEN id cua chung -- khong don lai"
    );

    // (3) UNIQUE INDEX con do duoc.
    let dup = migrated.write(|tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO glossary_entry \
             (source_term, translation, note, category, term_origin, created_at) \
             VALUES ('a', NULL, '', 'person', 'manual', 'x')",
            [],
        )
    });
    assert!(
        dup.is_err(),
        "UNIQUE INDEX idx_glossary_entry_source_term phai con chan source_term trung sau di tru"
    );

    // (3) trigger mot chieu (AD-36) con do duoc.
    migrated
        .write(|tx: &Transaction<'_>| {
            tx.execute("UPDATE glossary_entry SET translation = 'x' WHERE id = 1", [])
        })
        .expect("chot ban dich cho id=1");
    let regress = migrated.write(|tx: &Transaction<'_>| {
        tx.execute("UPDATE glossary_entry SET translation = NULL WHERE id = 1", [])
    });
    assert!(
        regress.is_err(),
        "trigger glossary_entry_lifecycle_is_one_way (AD-36) phai con chan luot lui ve NULL \
         sau di tru"
    );

    // (4) id cap TIEP THEO lon hon MOI id tung cap: id=3 ('c', da xoa TRUOC di tru, va la id
    // CAO NHAT tung cap) KHONG duoc phat lai -- hang moi phai nhan id=4, khong phai id=3.
    let new_id: i64 = migrated
        .write(|tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO glossary_entry \
                 (source_term, translation, note, category, term_origin, created_at) \
                 VALUES ('d', NULL, '', 'person', 'manual', 'x')",
                [],
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("chen hang moi");
    assert_eq!(
        new_id, 4,
        "id cap tiep theo phai la 4 (lon hon moi id tung cap: 1, 2, 3-da-xoa) -- id 3 KHONG \
         duoc phat lai. Nhan id={new_id} -- neu la 3 thi moc sqlite_sequence da KHONG duoc \
         mang theo qua luot di tru"
    );

    // Doi chung: CHECK bon gia tri that su nhan 'file_import' sau di tru.
    migrated
        .write(|tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO glossary_entry \
                 (source_term, translation, note, category, term_origin, created_at) \
                 VALUES ('e', NULL, '', 'person', 'file_import', 'x')",
                [],
            )
        })
        .expect("CHECK sau di tru phai nhan 'file_import' -- gia tri thu tu");

    drop(migrated);
    cleanup(&dir);
}
