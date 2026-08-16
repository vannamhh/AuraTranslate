//! Hành vi của **mục đã ghim** — Story 1.20, AC2 · AC3.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_contract.rs`/`project_contract.rs` — một tệp,
//! một mối quan tâm.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — thừa kế nguyên vẹn từ `store_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + `AtomicU64`). Không thêm `tempfile`.
//! 2. **Drop `Store`/`OpenWork` TRƯỚC khi xoá thư mục** — Windows từ chối xoá tệp đang mở.
//! 3. Không `sleep` dài.
//! 4. Không ca nào treo khi nó trượt.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 PHẠM VI: `global.db`, KHÔNG `project.db` — Ice ký lại 2026-08-11
//! ─────────────────────────────────────────────────────────────────────────────
//! Bản đầu của story đặt bảng ghim vào `project.db` (Quyết định #1, 2026-08-10). Một phép
//! đo hôm sau lật nó — không tồn tại đường mở lại một `.atproj`, nên **AC3 không nghiệm
//! thu được** ở phạm vi đó. [`the_pin_table_lives_in_the_global_store_not_the_project_one`]
//! là ca canh mệnh đề mới, và nó đỏ ngay nếu ai đó chuyển bảng ngược lại mà quên phép đo.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::pinned::{PinnedEntry, list_pinned_entries, pin_entry, unpin_entry};
use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::store::{
    GLOBAL_MIGRATIONS, PINNED_ENTRY_DDL, PROJECT_MIGRATIONS, Store, StoreSpec, Transaction,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

/// Một thư mục tạm **của riêng ca này**. Xem luật 1 ở doc-comment đầu tệp.
fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-pinned-{}-{}-{}",
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

/// Một `global.db` thật, với bộ di trú THẬT.
fn open_global(dir: &Path) -> Store {
    Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db")
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bước di trú thứ ba của `global.db`
// ═════════════════════════════════════════════════════════════════════════════════

/// Một `global.db` mới tinh kết thúc ở **phiên bản 3**, và bảng `pinned_entry` có mặt.
#[test]
fn a_fresh_global_database_ends_at_the_pinned_entry_step() {
    let dir = temp_dir("fresh-global-target");
    let store = open_global(&dir);

    assert_eq!(
        store.schema_version(),
        3,
        "`GLOBAL_MIGRATIONS` co ba buoc (1.7 so di tru, 1.8 `config_value`, 1.20 \
         `pinned_entry`), nen mot `global.db` moi phai ket thuc o phien ban 3"
    );
    assert_eq!(
        GLOBAL_MIGRATIONS.len(),
        3,
        "so buoc va so phien ban dich phai di cung nhau"
    );

    let pinned = list_pinned_entries(Some(&store)).expect("doc bo ghim");
    assert!(pinned.is_empty(), "mot kho vua tao chua ghim gi");

    drop(store);
    cleanup(&dir);
}

/// Một `global.db` ở **phiên bản 2** di trú lên 3 và **giữ nguyên dữ liệu cũ** (AC3).
///
/// 🔴 Ca thật của người dùng đã chạy bản 1.8–1.19: theme, chế độ, bố cục và tập nguồn đang
/// tắt đều nằm trong `config_value`, và một bước di trú làm mất chúng là mất **toàn bộ**
/// cấu hình của họ. Bộ cũ dựng bằng **hai bước đầu của chính `GLOBAL_MIGRATIONS`**, không
/// một bản chép — một fixture chép tay sẽ trôi khỏi sự thật đúng vào ngày một bước cũ
/// được sửa.
#[test]
fn an_older_global_database_migrates_up_and_keeps_its_rows() {
    let dir = temp_dir("older-global");
    let db = dir.join("global.db");

    let old_steps: &'static [auratranslate_lib::core::store::Migration] = &GLOBAL_MIGRATIONS[..2];
    let old_spec = StoreSpec {
        migrations: old_steps,
        ..StoreSpec::global(db.clone())
    };

    let old = Store::open(old_spec).expect("mo kho o phien ban cu");
    assert_eq!(old.schema_version(), 2, "fixture phai dung o phien ban 2");
    old.write(|tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO config_value (kind, key, value, updated_at) \
             VALUES ('app_config', 'theme', 'dark', 'x')",
            [],
        )?;
        Ok(())
    })
    .expect("ghi hang config_value cu");
    drop(old);

    let migrated = Store::open(StoreSpec::global(db)).expect("mo lai sau khi di tru");
    assert_eq!(migrated.schema_version(), 3, "buoc 3 phai da chay");

    let theme: String = migrated
        .read(|conn| {
            conn.query_row(
                "SELECT value FROM config_value WHERE kind = 'app_config' AND key = 'theme'",
                [],
                |r| r.get(0),
            )
        })
        .expect("doc lai hang cu");
    assert_eq!(theme, "dark", "di tru khong duoc dung toi cau hinh cu");

    let empty: i64 = migrated
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM pinned_entry", [], |r| r.get(0)))
        .expect("dem bang moi");
    assert_eq!(empty, 0, "bang moi phai rong, khong mot hang dung san");

    drop(migrated);
    cleanup(&dir);
}

/// 🔴 **Bảng ghim sống ở `global.db`, và `project.db` KHÔNG mang nó** — Ice ký 2026-08-11.
///
/// Ca này là lưới của lượt đổi phạm vi. Nó đỏ ở **cả hai** hướng đi sai: chuyển bảng ngược
/// về `project.db`, hoặc để lại một bản sao ở đó *"cho chắc"*.
#[test]
fn the_pin_table_lives_in_the_global_store_not_the_project_one() {
    let root = temp_dir("scope-of-the-table");
    let opened = create_work_from_text(&root, "Pham Vi", "zh", "tieu thuyet", "noi dung".to_owned())
        .expect("tao tac pham");

    // ⚠️ Cap nhat Story 2.2: `PROJECT_MIGRATIONS` nhan them buoc `SEGMENT_TARGET_TEXT_DDL`,
    // danh so **6** (so 4 la mot so DA CHAY — vet seo cua chinh lượt doi pham vi nay). Nam
    // buoc, dich la phien ban 6. Hai con so duoi day la PROXY; menh de that cua ca nay la
    // phep dem `sqlite_master` ngay ben duoi — bang `pinned_entry` KHONG duoc co mat o
    // `project.db`.
    assert!(
        PROJECT_MIGRATIONS.iter().all(|m| m.sql != PINNED_ENTRY_DDL),
        "`PINNED_ENTRY_DDL` quay lai `PROJECT_MIGRATIONS` — Ice ky 2026-08-11 chuyen no sang `global.db`"
    );
    // 🔵 CAP NHAT 2026-08-14 (Story 2.5): nam buoc → SAU, dich 6 → 7. Menh de cua ca nay
    // KHONG doi mot chu — no canh viec `pinned_entry` **khong** o trong `project.db`; hai
    // con so duoi day chi la neo de phep kiem do do lech khi bo di tru doi.
    // 🔵 CAP NHAT 2026-08-15 (Story 2.5c): sau buoc → BAY, dich 7 → 8 (cot `is_omitted`).
    // 🔵 CAP NHAT 2026-08-16 (Story 2.5d): bay buoc → TAM, dich 8 → 9 (cot
    //    `is_target_paragraph_end`, FR134/AD-46).
    // 🔵 CAP NHAT 2026-08-16 (Story 2.6): tam buoc → CHIN, dich 9 → 10 (index
    //    `idx_segment_version_segment_created`, FR101). ⚠️ Buoc 10 la buoc dau tien KHONG
    //    them mot cot nao — no dung mot cau truc dan xuat. Hai con so duoi day van chi la
    //    NEO: menh de cua ca nay khong doi mot chu, no canh viec `pinned_entry` khong nam
    //    trong `project.db`.
    assert_eq!(
        PROJECT_MIGRATIONS.len(),
        9,
        "`PROJECT_MIGRATIONS` phai co chin buoc — 1/2/3 cua Story 1.15, 5 cua Story 2.1, \
         6 cua Story 2.2, 7 cua Story 2.5, 8 cua Story 2.5c, 9 cua Story 2.5d, \
         10 cua Story 2.6"
    );
    assert_eq!(
        opened.store.schema_version(),
        10,
        "mot `project.db` moi phai dung o phien ban 10 (so 4 da chay)"
    );

    let has_table: i64 = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'pinned_entry'",
                [],
                |r| r.get(0),
            )
        })
        .expect("doc sqlite_master cua project.db");
    assert_eq!(
        has_table, 0,
        "`project.db` van mang bang `pinned_entry` — pham vi ghim da di nguoc lai quyet \
         dinh 2026-08-11, hoac mot ban sao bi bo quen o do"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hợp đồng ghim
// ═════════════════════════════════════════════════════════════════════════════════

/// Ghim hai lần cùng một mục cho **đúng một** hàng — `UNIQUE` thật sự cưỡng chế.
#[test]
fn pinning_the_same_entry_twice_leaves_exactly_one_row() {
    let dir = temp_dir("pin-twice");
    let store = open_global(&dir);

    pin_entry(Some(&store), "cvdict", 41, "聽潮閣", Some("Thinh Trieu Cac"))
        .expect("luot ghim thu nhat");
    let after = pin_entry(Some(&store), "cvdict", 41, "聽潮閣", Some("Thinh Trieu Cac"))
        .expect("luot ghim thu hai phai VO HAI, khong mot loi");

    assert_eq!(after.len(), 1, "ghim hai lan sinh ra hai hang");
    assert_eq!(after[0].source_code, "cvdict");
    assert_eq!(after[0].entry_id, 41);
    assert_eq!(after[0].headword, "聽潮閣");
    assert_eq!(after[0].gloss.as_deref(), Some("Thinh Trieu Cac"));

    drop(store);
    cleanup(&dir);
}

/// **Đối chứng âm** — bỏ ghim rồi ghim lại cho trạng thái *giống hệt* trước khi bỏ.
///
/// ⚠️ So bằng bốn trường mang nghĩa, **không** `id` và **không** `pinned_at`: `id` đi qua
/// `AUTOINCREMENT` nên nó KHÔNG bao giờ được phát lại (AD-3), và `pinned_at` là thời điểm
/// ghim mới. Một assert trên hai trường đó sẽ đỏ vì đúng những bất biến ta muốn giữ.
#[test]
fn unpinning_then_pinning_again_restores_the_same_entry() {
    let dir = temp_dir("unpin-repin");
    let store = open_global(&dir);

    let before = pin_entry(Some(&store), "thieuchuu", 7, "氣機", None).expect("ghim lan dau");
    assert_eq!(before.len(), 1);

    let emptied = unpin_entry(Some(&store), "thieuchuu", 7).expect("bo ghim");
    assert!(emptied.is_empty(), "bo ghim phai xoa han hang do");

    let again = pin_entry(Some(&store), "thieuchuu", 7, "氣機", None).expect("ghim lai");
    assert_eq!(again.len(), 1);
    assert_eq!(again[0].source_code, before[0].source_code);
    assert_eq!(again[0].entry_id, before[0].entry_id);
    assert_eq!(again[0].headword, before[0].headword);
    assert_eq!(again[0].gloss, before[0].gloss);

    // Bo ghim mot muc chua tung ghim la thao tac VO HAI, khong mot loi.
    let noop = unpin_entry(Some(&store), "thieuchuu", 999).expect("bo ghim mot muc la");
    assert_eq!(noop.len(), 1, "mot luot bo ghim khong khop khong duoc xoa gi");

    drop(store);
    cleanup(&dir);
}

/// 🔴 **AC3 — mục ghim sống sót qua một lượt ĐÓNG rồi MỞ LẠI kho.**
///
/// Đây là ca mà lượt đổi phạm vi tồn tại để làm cho có nghĩa. Ở `project.db` nó **không
/// nghiệm thu được**: không có đường mở lại một `.atproj`, nên bộ ghim không có đường nào
/// để đọc tới sau khi tiến trình khởi động lại. `global.db` mở lại ở mỗi lượt `setup()`,
/// nên vòng ghi → đóng → mở → đọc dưới đây **là** đúng vòng mà người dùng đi qua.
#[test]
fn pins_survive_closing_and_reopening_the_store() {
    let dir = temp_dir("survive-reopen");

    let first = open_global(&dir);
    pin_entry(Some(&first), "cvdict", 41, "聽潮閣", Some("Thinh Trieu Cac")).expect("ghim");
    first.close();
    drop(first);

    let second = open_global(&dir);
    let pinned = list_pinned_entries(Some(&second)).expect("doc lai sau khi mo lai");
    assert_eq!(pinned.len(), 1, "muc ghim phai song sot qua mot luot mo lai");
    assert_eq!(pinned[0].headword, "聽潮閣");
    assert_eq!(pinned[0].gloss.as_deref(), Some("Thinh Trieu Cac"));

    drop(second);
    cleanup(&dir);
}

/// Kho vắng mặt ⇒ một lỗi **CÓ TÊN** thuộc từ vựng KHO, không một danh sách rỗng.
#[test]
fn without_a_store_every_path_is_a_named_error() {
    for (label, err) in [
        (
            "list",
            list_pinned_entries(None).expect_err("doc phai la mot loi"),
        ),
        (
            "pin",
            pin_entry(None, "cvdict", 1, "x", None).expect_err("ghim phai la mot loi"),
        ),
        (
            "unpin",
            unpin_entry(None, "cvdict", 1).expect_err("bo ghim phai la mot loi"),
        ),
    ] {
        assert_eq!(
            err.code(),
            "store.open_failed",
            "{label} tra ve mot `code` khac — tu vung loi thu hai da len vao"
        );
        assert_eq!(
            err.message_key(),
            MessageKey::StoreOpenFailed,
            "{label} mang mot `message_key` ngoai nam khoa kho cua Story 1.7"
        );
        assert_eq!(
            err.params().get("store").map(String::as_str),
            Some("global"),
            "{label}: `params` phai mang ten kho, va chi DU LIEU"
        );
    }
}

/// `gloss = NULL` là hình dạng **thật**, không một ca biên: một lượt tra không lấy về nghĩa
/// nào vẫn ghim được, và nó phải đi về nguyên vẹn là `None`.
#[test]
fn an_entry_with_no_gloss_is_still_pinnable_and_comes_back_as_none() {
    let dir = temp_dir("null-gloss");
    let store = open_global(&dir);

    let rows = pin_entry(Some(&store), "vietphrase", 3, "鐵鏽", None).expect("ghim");
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].gloss, None,
        "`gloss` NULL phai di ve la `None`, khong mot chuoi rong"
    );
    assert!(
        !rows[0].pinned_at.is_empty(),
        "`pinned_at` phai duoc SQLite dien, khong de rong"
    );

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 MỐI NỐI AC3 — hai ca thêm ở lượt code review 2026-08-11
// ═════════════════════════════════════════════════════════════════════════════════
//
// [`pins_survive_closing_and_reopening_the_store`] đóng vế KHO của AC3: ghi → đóng → mở →
// đọc. Nó **không** chạm hai thứ mà AC3 cũng đứng trên, và cả hai đều đo được bằng máy:
//
// 1. **Tên trường trên dây.** `PinnedEntry` (Rust) và `PinnedEntry` (TS, `config/pinned.ts`)
//    khai cùng sáu tên `snake_case`, và cả hai phía chỉ ghi ràng buộc đó bằng **chú thích**
//    (*"Đổi một tên ở đây mà không đổi ở kia cho ra `undefined` mà TypeScript không hề
//    biết"*). `tests/ipc_contract.rs` **không** khoá hình dạng này. Một lượt đổi tên phía
//    Rust vì thế đi qua trọn `cargo test` **và** trọn chín cổng, rồi vỡ đúng ở màn hình
//    sau khi mở lại app — tức vỡ **đúng AC3** và không sớm hơn một giây nào.
// 2. **Thứ tự sau lượt mở lại.** `SELECT_PINNED` sắp `pinned_at DESC, id DESC`, nhưng
//    không ca nào đọc thứ tự đó **qua** một mối nối đóng/mở.

/// 🔴 **AC3 · mối nối dây** — sáu tên trường đi tới webview đúng như `config/pinned.ts` đọc.
///
/// Ca này đọc **mã nguồn TypeScript thật** thay vì một danh sách chép tay: một bảng tên
/// viết cứng ở đây chỉ khoá Rust vào chính nó, còn thứ cần khoá là Rust vào **phía bên
/// kia dây**. Cùng doctrine quét nguồn của `store_boundary.rs`.
#[test]
fn the_pinned_wire_shape_matches_what_the_frontend_reads() {
    let entry = PinnedEntry {
        id: 7,
        source_code: "cvdict".to_string(),
        entry_id: 41,
        headword: "聽潮閣".to_string(),
        gloss: None,
        pinned_at: "2026-08-11T09:15:00.000Z".to_string(),
    };

    let json = serde_json::to_value(&entry).expect("serialize `PinnedEntry`");
    let object = json.as_object().expect("`PinnedEntry` phai serialize thanh object");

    // `gloss = None` đi ra là `null` CÓ MẶT, không một trường bị bỏ đi: `config/pinned.ts`
    // khai `gloss: string | null`, và một trường vắng mặt đọc ra `undefined` — một giá trị
    // thứ ba mà không nhánh nào phía TypeScript xử.
    assert_eq!(
        object.get("gloss"),
        Some(&serde_json::Value::Null),
        "`gloss` NULL phai di ra la `null` co mat, khong mot truong vang mat"
    );

    let adapter_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src")
        .join("config")
        .join("pinned.ts");
    let adapter = fs::read_to_string(&adapter_path)
        .unwrap_or_else(|e| panic!("doc {}: {e}", adapter_path.display()));

    // Cắt lấy đúng khối `export type PinnedEntry = { … }` — quét cả tệp sẽ khớp nhầm với
    // các tên nhắc trong chú thích, và một ca xanh vì lý do sai còn tệ hơn một ca đỏ.
    let start = adapter
        .find("export type PinnedEntry = {")
        .expect("`config/pinned.ts` khong con khai `export type PinnedEntry`");
    let block_len = adapter[start..]
        .find("\n}")
        .expect("khoi `PinnedEntry` khong dong lai");
    let block = &adapter[start..start + block_len];

    // ⚠️ Khớp theo **ĐẦU DÒNG**, không `contains`: `"entry_id:"` chứa `"id:"`, nên một
    // phép khớp chuỗi con để lọt đúng ca gỡ mất trường `id`. Một lưới xanh vì một trường
    // KHÁC tình cờ chứa tên nó là một lưới không bắt gì cả.
    let declared: Vec<&str> = block
        .lines()
        .filter_map(|line| line.trim().split_once(':').map(|(name, _)| name.trim()))
        .collect();

    for name in object.keys() {
        assert!(
            declared.contains(&name.as_str()),
            "truong `{name}` cua `PinnedEntry` phia Rust khong duoc khai trong \
             `src/config/pinned.ts` — hai dau day da lech ten, va TypeScript doc ra \
             `undefined` ma khong bao gi. Da khai: {declared:?}"
        );
    }
    assert_eq!(
        object.len(),
        6,
        "so truong tren day doi — cap nhat `src/config/pinned.ts` cung luot"
    );
}

/// 🔴 **AC3 · thứ tự sống sót qua lượt mở lại** — gần nhất trước, đo SAU khi đóng/mở kho.
#[test]
fn pins_come_back_most_recent_first_after_a_reopen() {
    let dir = temp_dir("order-after-reopen");

    let first = open_global(&dir);
    pin_entry(Some(&first), "cvdict", 1, "春秋", None).expect("ghim mot");
    pin_entry(Some(&first), "cvdict", 2, "氣機", None).expect("ghim hai");
    pin_entry(Some(&first), "cvdict", 3, "徐驍", None).expect("ghim ba");
    first.close();
    drop(first);

    let second = open_global(&dir);
    let pinned = list_pinned_entries(Some(&second)).expect("doc lai sau khi mo lai");
    let order: Vec<i64> = pinned.iter().map(|p| p.entry_id).collect();
    assert_eq!(
        order,
        vec![3, 2, 1],
        "gan nhat truoc phai song sot qua mot luot dong/mo — `pinned_at DESC, id DESC`"
    );

    drop(second);
    cleanup(&dir);
}
