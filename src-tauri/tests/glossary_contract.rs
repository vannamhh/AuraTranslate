//! Hành vi Glossary hai tầng + vòng đời ba trạng thái — Story 3.1, I/O & Edge-Case Matrix.
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

use auratranslate_lib::core::glossary::{
    Category, TermOrigin, confirm_translation, entries_eligible_for_injection, insert_entry,
    load_tier,
};
use auratranslate_lib::core::scope::{ScopeError, ScopeResolver, WorkScope};
use auratranslate_lib::core::store::{
    GLOBAL_MIGRATIONS, Store, StoreError, StoreSpec, Transaction,
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

    insert_entry(
        &store,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
        TermOrigin::Manual,
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

    insert_entry(
        &global_store,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
        TermOrigin::Manual,
    )
    .expect("chen muc global");
    insert_entry(
        &work_store,
        "慕容",
        Some("Mộ Dong"),
        "",
        Category::Person,
        TermOrigin::Manual,
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

    insert_entry(
        &global_store,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
        TermOrigin::Manual,
    )
    .expect("chen muc global da chot");
    insert_entry(
        &work_store,
        "慕容",
        None,
        "",
        Category::Person,
        TermOrigin::Manual,
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

    insert_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
        TermOrigin::Manual,
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

    insert_entry(
        &store,
        "青丘",
        Some("Thanh Khâu"),
        "",
        Category::Place,
        TermOrigin::Manual,
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
        let result = insert_entry(
            &store,
            "term",
            Some(blank),
            "",
            Category::Other,
            TermOrigin::Manual,
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
/// nào ngoài `NOT NULL` — `insert_entry("", …)` chiếm vĩnh viễn ô chuỗi rỗng của chỉ mục
/// UNIQUE đó.
#[test]
fn an_empty_or_whitespace_only_source_term_is_refused_and_writes_nothing() {
    let dir = temp_dir("empty-source-term");
    let store = open_global(&dir);

    for (label, blank) in every_blank_form() {
        let result = insert_entry(
            &store,
            blank,
            Some("ban dich"),
            "",
            Category::Other,
            TermOrigin::Manual,
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

    let id = insert_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
        TermOrigin::Manual,
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

    insert_entry(
        &store,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
        TermOrigin::Manual,
    )
    .expect("chen muc dau tien");

    let second = insert_entry(
        &store,
        "慕容",
        Some("Mo Dung Khac"),
        "",
        Category::Person,
        TermOrigin::Manual,
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
/// `insert_entry`) cũng bị `CHECK` từ chối — cùng một hằng DDL, nhưng khác đường ghi.
#[test]
fn confirming_with_a_blank_translation_is_refused_by_the_same_check_insert_uses() {
    let dir = temp_dir("confirm-blank-translation");
    let store = open_global(&dir);

    let id = insert_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
        TermOrigin::Manual,
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

/// P8(c) — `confirm_translation` với `id` KHÔNG khớp hàng nào trả `Ok(())` dù 0 hàng đổi.
///
/// ⚠️ Ca này KHOÁ hành vi hiện tại bằng test — xem cảnh báo rủi ro ở doc-comment của
/// `confirm_translation`: một lượt chốt nhắm vào một `id` không tồn tại sẽ THÀNH CÔNG mà
/// không làm gì cả, đúng khuôn `delete_value`. **Chủ của rủi ro đó: Story 3.3.**
#[test]
fn confirming_an_unknown_id_succeeds_and_changes_nothing() {
    let dir = temp_dir("confirm-unknown-id");
    let store = open_global(&dir);

    insert_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
        TermOrigin::Manual,
    )
    .expect("chen mot muc that de doi chung");

    let result = confirm_translation(&store, 999_999, "Thanh Khâu");
    assert!(
        result.is_ok(),
        "chot mot id khong ton tai phai THANH CONG (0 hang doi), khong phai mot loi. \
         Nhan: {result:?}"
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

    let id = insert_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
        TermOrigin::Manual,
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

/// 🔴 `insert_entry` cắt khoảng trắng biên ở **tầng Rust**, và đây là ca DUY NHẤT đo được
/// điều đó từ ngoài.
///
/// ⚠️ Ca trùng đã có (`a_duplicate_source_term_is_refused_and_the_original_row_survives`)
/// chèn **cùng một chuỗi** hai lần, nên nó xanh y hệt dù `.trim()` ở `insert_entry` có bị
/// xoá hay không. Các ca dạng trắng cũng không đo được: chúng bị `CHECK` của SQL chặn độc
/// lập với tầng Rust. ⇒ Không có ca này, xoá `.trim()` khỏi `insert_entry` là một lượt đỏ
/// KHÔNG XẢY RA, và `" 慕容"` với `"慕容"` thành hai hàng dưới một chỉ mục tự xưng là "một
/// thuật ngữ, một mục".
#[test]
fn a_padded_source_term_collides_with_its_trimmed_twin_instead_of_becoming_a_second_row() {
    let dir = temp_dir("source-term-trim-collides");
    let store = open_global(&dir);

    insert_entry(
        &store,
        "慕容",
        Some("Mộ Dung"),
        "",
        Category::Person,
        TermOrigin::Manual,
    )
    .expect("chen muc dau tien");

    let padded = insert_entry(
        &store,
        "  慕容\t",
        Some("Mo Dung Khac"),
        "",
        Category::Person,
        TermOrigin::Manual,
    );
    assert!(
        matches!(padded, Err(StoreError::WriteFailed { .. })),
        "`  慕容\\t` phai bi cat khoang trang bien o tang Rust roi va vao DUNG mot UNIQUE. \
         Nhan: {padded:?} -- neu la Ok, `.trim()` o insert_entry da bien mat va chi muc \
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

    let id = insert_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
        TermOrigin::Manual,
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

/// 🔴 Mọi biến thể của `Category` và `TermOrigin` phải ghi xuống đĩa được và đọc lại đúng.
///
/// ⚠️ Trước ca này, bộ test chỉ từng ghi `Person` · `Place` · `Other` · `Manual`. **Ba biến
/// thể chưa từng chạm đĩa lần nào**: `DomainTerm` · `ImportScan` · `ReviewHarvest`. Một lỗi
/// gõ trong `as_str()` của bất kỳ cái nào trong ba — `"domainterm"`, `"importscan"` — đi
/// qua trình biên dịch sạch, đi qua cả bộ test sạch, và chỉ đỏ lần đầu Story 3.5 (quét khi
/// nhập) hay Epic 8 (thu hoạch từ review) ghi một hàng THẬT. Ca này đóng khoảng cách giữa
/// kiểu Rust và `CHECK … IN (…)` bằng phép chạy, không bằng lời hứa trong doc-comment.
#[test]
fn every_category_and_term_origin_variant_round_trips_through_the_store() {
    let dir = temp_dir("all-enum-variants");
    let store = open_global(&dir);

    let categories = [
        Category::Person,
        Category::Place,
        Category::DomainTerm,
        Category::Other,
    ];
    let origins = [
        TermOrigin::Manual,
        TermOrigin::ImportScan,
        TermOrigin::ReviewHarvest,
    ];

    for (i, category) in categories.iter().enumerate() {
        for (j, term_origin) in origins.iter().enumerate() {
            let term = format!("thuat-ngu-{i}-{j}");
            insert_entry(
                &store,
                &term,
                Some("ban dich"),
                "",
                *category,
                *term_origin,
            )
            .unwrap_or_else(|e| {
                panic!(
                    "category={category} term_origin={term_origin} phai ghi xuong dia duoc \
                     -- mot `WriteFailed` o day nghia la `as_str()` da troi khoi \
                     `CHECK (… IN (…))` cua GLOSSARY_ENTRY_DDL. Nhan: {e:?}"
                )
            });
        }
    }

    let global = load_tier(&store).expect("nap tang global");
    assert_eq!(global.len(), categories.len() * origins.len());

    for (i, category) in categories.iter().enumerate() {
        for (j, term_origin) in origins.iter().enumerate() {
            let entry = &global[&format!("thuat-ngu-{i}-{j}")];
            assert_eq!(
                entry.category, *category,
                "category phai doc lai DUNG bien the da ghi"
            );
            assert_eq!(
                entry.term_origin, *term_origin,
                "term_origin phai doc lai DUNG bien the da ghi"
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

    let id = insert_entry(
        &store,
        "慕容",
        Some("Mộ Dung"),
        "ho kep, khong phai ten don",
        Category::DomainTerm,
        TermOrigin::ImportScan,
    )
    .expect("chen muc day du bay cot");

    let global = load_tier(&store).expect("nap tang global");
    let entry = &global["慕容"];

    assert_eq!(entry.id, id, "id phai la rowid ma insert_entry vua tra ve");
    assert_eq!(entry.source_term, "慕容");
    assert_eq!(entry.translation.as_deref(), Some("Mộ Dung"));
    assert_eq!(
        entry.note, "ho kep, khong phai ten don",
        "note phai doc lai nguyen van -- doc nham cot se lot ra o day"
    );
    assert_eq!(entry.category, Category::DomainTerm);
    assert_eq!(entry.term_origin, TermOrigin::ImportScan);

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
    insert_entry(
        &store,
        "青丘",
        None,
        "",
        Category::Place,
        TermOrigin::Manual,
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
/// Fixture dựng bằng **ba bước THẬT đầu tiên** của `GLOBAL_MIGRATIONS` cộng một bước 4
/// KHÔNG có `CHECK` — cùng khuôn `pinned_contract.rs::an_older_global_database_…` dùng lát
/// cắt của bộ di trú thật thay vì một bản chép tay, để fixture không trôi khỏi sự thật.
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

    static DRIFTED_LADDER: [Migration; 4] = [
        GLOBAL_MIGRATIONS[0],
        GLOBAL_MIGRATIONS[1],
        GLOBAL_MIGRATIONS[2],
        Migration {
            to_version: 4,
            sql: PERMISSIVE_GLOSSARY_DDL,
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
        assert_eq!(drifted.schema_version(), 4, "fixture phai dung o dich that");

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
/// nó được.** Cả hai ca đó gọi qua `insert_entry`, thứ cắt khoảng trắng bằng `str::trim()`
/// của Rust TRƯỚC khi chạm SQL. Một chuỗi `"\u{2009}"` bị Rust cắt thành `""`, rồi `CHECK`
/// từ chối `''` — nên chúng xanh **y hệt** dù bảng ký tự trong `GLOSSARY_ENTRY_DDL` có bảy
/// ký tự hay hai mươi lăm. Tức lớp Rust CHE lớp SQL, và suốt lượt rà soát #1 không phép đo
/// nào nhìn thấy 17 điểm mã mà bảng bảy ký tự để lọt.
///
/// ⇒ Ca này ghi thẳng bằng SQL, đúng cửa mà một bản ứng dụng khác hay một lượt sửa tay
/// `.db` sẽ đi, và vì thế nó đo **chính bảng ký tự của hằng DDL**, không đo `insert_entry`.
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
