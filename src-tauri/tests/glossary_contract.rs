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

/// Bảy dạng "trắng" — đo 2026-08-19 trên SQLite 3.53.4: `trim(X)` MỘT tham số chỉ cắt dấu
/// cách ASCII, nên tab/xuống dòng/NBSP/dấu cách biểu ý đều LỌT qua một `CHECK` một tham số.
/// `GLOSSARY_ENTRY_DDL` dùng `trim(X, <bảng ký tự>)` HAI tham số — bảy ca dưới đây, cộng
/// một chuỗi TRỘN cả bảy loại, là bằng chứng chạy được cho bảng ký tự đó, không chỉ đọc
/// bằng mắt trong doc-comment.
fn seven_blank_forms() -> [(&'static str, &'static str); 8] {
    [
        ("rong", ""),
        ("dau cach ASCII", "   "),
        ("tab", "\t\t"),
        ("xuong dong LF", "\n\n"),
        ("xuong dong CR", "\r\r"),
        ("NBSP U+00A0", "\u{00A0}\u{00A0}"),
        ("dau cach bieu y U+3000", "\u{3000}\u{3000}"),
        ("tron ca bay loai", " \t\n\r\u{00A0}\u{3000} "),
    ]
}

#[test]
fn an_empty_or_whitespace_only_translation_is_refused_and_writes_nothing() {
    let dir = temp_dir("empty-translation");
    let store = open_global(&dir);

    for (label, blank) in seven_blank_forms() {
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
        "tam luot chen bi tu choi khong duoc de lai mot hang nao"
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

    for (label, blank) in seven_blank_forms() {
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
        "tam luot chen bi tu choi khong duoc de lai mot hang nao"
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

    for (label, blank) in seven_blank_forms() {
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
