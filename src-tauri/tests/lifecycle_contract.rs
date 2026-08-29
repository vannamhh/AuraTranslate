//! Bề mặt `commands::lifecycle` gọi với một `OpenWork` THẬT — Story 5.4, toàn bộ §I/O Matrix.
//!
//! Dựng `OpenWork` qua `create_work_from_text` — đúng khuôn `glossary_commands_contract.rs`
//! (Story 3.3, `open_work(root, tag)`), chỗ sản phẩm DUY NHẤT dựng một `OpenWork` thật.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — thừa kế nguyên vẹn từ `glossary_commands_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. Mỗi ca một thư mục tạm riêng (pid + `AtomicU64`).
//! 2. Drop `OpenWork`/`Store` TRƯỚC khi xoá thư mục — Windows từ chối xoá tệp đang mở.
//! 3. Không `sleep` dài.
//! 4. Không ca nào treo khi nó trượt.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::lifecycle::{
    read_work_lifecycle, set_chapter_status, set_work_status_override,
};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::lifecycle::LifecycleStatus;
use auratranslate_lib::core::library::meta::WorkMeta;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-lifecycle-cmds-{}-{}-{}",
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

/// Đúng khuôn `glossary_commands_contract.rs::open_work` — `create_work_from_text` là chỗ
/// SẢN PHẨM DUY NHẤT dựng một `OpenWork` thật.
fn open_work(root: &Path, tag: &str) -> OpenWork {
    create_work_from_text(root, tag, "en", "", "cau nguon mau".to_owned())
        .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"))
}

/// Chèn thêm một Chương THỨ HAI trực tiếp qua SQL — không có lệnh sản phẩm nào tạo Chương
/// thứ hai hôm nay (FR14, Epic 6). Trả `chapter_id` vừa chèn.
fn add_chapter(open: &OpenWork, ord: i64, status: &str) -> i64 {
    let status_owned = status.to_owned();
    open.store
        .write(move |tx: &auratranslate_lib::core::store::Transaction<'_>| {
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (?1, NULL, 'x', ?2, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                (ord, &status_owned),
            )?;
            Ok(tx.last_insert_rowid())
        })
        .unwrap_or_else(|e| panic!("chen Chuong thu hai: {e}"))
}

/// Đọc lại `meta.json` trên đĩa — bằng chứng "đã ghi ra", không tin lời gọi (§AC2 của
/// story: *"kiểm bằng cách đọc lại cả hai, không bằng cách tin lời gọi"*).
fn read_meta_from_disk(dir: &Path) -> WorkMeta {
    WorkMeta::read(dir).unwrap_or_else(|e| panic!("doc meta.json: {e:?}"))
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix
// ═════════════════════════════════════════════════════════════════════════════════

/// "Chương mới nhập": `chapter.status = "not_started"`; `work.status_override IS NULL`;
/// `meta.json` mang `status = "not_started"`, `status_is_override = false`.
#[test]
fn a_freshly_imported_chapter_starts_not_started_with_no_override() {
    let root = temp_dir("new-chapter");
    let opened = open_work(&root, "New Chapter");

    let lifecycle =
        read_work_lifecycle(Some(&opened)).unwrap_or_else(|e| panic!("read_work_lifecycle: {e:?}"));
    assert_eq!(lifecycle.status.as_deref(), Some("not_started"));
    assert!(!lifecycle.status_is_override);

    let on_disk = read_meta_from_disk(&opened.dir);
    assert_eq!(on_disk.status.as_deref(), Some("not_started"));
    assert!(!on_disk.status_is_override);

    drop(opened);
    cleanup(&root);
}

/// "Đổi trạng thái Chương": Tác phẩm MỘT Chương, `set_chapter_status(id, "done")` ⇒ Chương
/// thành `done`; đọc lại tầng Tác phẩm ra `done` với `is_override = false`; `meta.json` ghi
/// lại — kiểm bằng cách đọc lại từ đĩa.
#[test]
fn setting_the_only_chapter_to_done_derives_the_work_as_done() {
    let root = temp_dir("single-chapter-done");
    let mut opened = open_work(&root, "Single Chapter");
    let chapter_id = opened.chapter_id;

    let lifecycle = set_chapter_status(Some(&mut opened), chapter_id, "done")
        .unwrap_or_else(|e| panic!("set_chapter_status: {e:?}"));
    assert_eq!(lifecycle.status.as_deref(), Some("done"));
    assert!(!lifecycle.status_is_override);

    let on_disk = read_meta_from_disk(&opened.dir);
    assert_eq!(on_disk.status.as_deref(), Some("done"));
    assert!(!on_disk.status_is_override);

    drop(opened);
    cleanup(&root);
}

/// "Trạng thái Chương hỗn hợp": 3 Chương — `done` · `not_started` · `paused` — suy ra
/// `in_progress`, KHÔNG `paused`.
#[test]
fn a_mixed_set_of_chapter_statuses_derives_in_progress_never_paused() {
    let root = temp_dir("mixed-chapters");
    let mut opened = open_work(&root, "Mixed");
    let chapter_id = opened.chapter_id;

    // Chương thứ nhất (đã tồn tại từ `create_work_from_text`) thành `done`.
    let lifecycle = set_chapter_status(Some(&mut opened), chapter_id, "done")
        .unwrap_or_else(|e| panic!("set_chapter_status done: {e:?}"));
    assert_eq!(lifecycle.status.as_deref(), Some("done"), "mot Chuong DUY NHAT da Done -> Done");

    // Thêm Chương thứ hai (`not_started`) và thứ ba (`paused`) trực tiếp qua SQL — không có
    // lệnh sản phẩm nào tạo Chương mới hôm nay (Epic 6).
    add_chapter(&opened, 2, "not_started");
    add_chapter(&opened, 3, "paused");

    // Không có lệnh nào GHI lại `meta.json` sau khi chèn tay -- gọi lại `read_work_lifecycle`
    // sẽ đọc `open.meta` CŨ (chưa dựng lại). Dựng lại bằng đúng đường sản phẩm: một lượt ghi
    // trạng thái Chương khác, buộc `write_lifecycle_after_change` chạy lại.
    let lifecycle = set_chapter_status(Some(&mut opened), chapter_id, "done")
        .unwrap_or_else(|e| panic!("set_chapter_status (dung lai): {e:?}"));
    assert_eq!(
        lifecycle.status.as_deref(),
        Some("in_progress"),
        "done + not_started + paused phai suy ra in_progress"
    );
    assert_ne!(lifecycle.status.as_deref(), Some("paused"), "KHONG BAO GIO suy ra paused");
    assert!(!lifecycle.status_is_override);

    drop(opened);
    cleanup(&root);
}

/// "Giá trị ngoài danh mục (Chương)": `set_chapter_status(id, "finished")` ⇒ không một lượt
/// ghi nào chạy, `err.lifecycle.unknown_status` `{status}`.
#[test]
fn setting_an_unknown_chapter_status_writes_nothing_and_names_the_bad_value() {
    let root = temp_dir("unknown-chapter-status");
    let mut opened = open_work(&root, "Unknown Status");
    let chapter_id = opened.chapter_id;

    let before = read_meta_from_disk(&opened.dir);

    let err = set_chapter_status(Some(&mut opened), chapter_id, "finished")
        .expect_err("gia tri ngoai danh muc phai bi tu choi");
    let json = serde_json::to_value(&err).expect("IpcError serialize duoc");
    assert_eq!(json["code"], "lifecycle.unknown_status");
    assert_eq!(json["params"]["status"], "finished");

    let after = read_meta_from_disk(&opened.dir);
    assert_eq!(before, after, "khong mot luot ghi nao duoc chay -- meta.json phai giu nguyen");

    drop(opened);
    cleanup(&root);
}

/// "`chapter_id` không tồn tại": `set_chapter_status(999, "done")` ⇒ không một lượt ghi nào
/// chạy, `segment.chapter_not_found` `{chapter_id}` (tái dùng, không đúc khoá thứ hai).
#[test]
fn setting_status_on_a_missing_chapter_id_writes_nothing_and_reuses_chapter_not_found() {
    let root = temp_dir("missing-chapter-id");
    let mut opened = open_work(&root, "Missing Chapter");
    let before = read_meta_from_disk(&opened.dir);

    let err = set_chapter_status(Some(&mut opened), 999_999, "done")
        .expect_err("chapter_id khong ton tai phai bi tu choi");
    let json = serde_json::to_value(&err).expect("IpcError serialize duoc");
    assert_eq!(json["code"], "segment.chapter_not_found");
    assert_eq!(json["params"]["chapter_id"], "999999");

    let after = read_meta_from_disk(&opened.dir);
    assert_eq!(before, after, "khong mot luot ghi nao duoc chay");

    drop(opened);
    cleanup(&root);
}

/// "Ghi đè thủ công": suy ra đang là `in_progress` (bằng chèn thêm một Chương thứ hai),
/// `set_work_status_override(Some("paused"))` ⇒ `status_override = 'paused'`; đọc ra
/// `status = "paused"`, `is_override = true`.
#[test]
fn overriding_the_work_status_wins_over_the_derived_value() {
    let root = temp_dir("manual-override");
    let mut opened = open_work(&root, "Override");
    add_chapter(&opened, 2, "not_started"); // Chuong 1 (not_started) + Chuong 2 (not_started)

    // Dung mot Chuong thanh done -> suy ra in_progress.
    let chapter_id = opened.chapter_id;
    let before = set_chapter_status(Some(&mut opened), chapter_id, "done")
        .unwrap_or_else(|e| panic!("set_chapter_status: {e:?}"));
    assert_eq!(before.status.as_deref(), Some("in_progress"));

    let overridden = set_work_status_override(Some(&mut opened), Some("paused"))
        .unwrap_or_else(|e| panic!("set_work_status_override: {e:?}"));
    assert_eq!(overridden.status.as_deref(), Some("paused"));
    assert!(overridden.status_is_override);

    let on_disk = read_meta_from_disk(&opened.dir);
    assert_eq!(on_disk.status.as_deref(), Some("paused"));
    assert!(on_disk.status_is_override);

    drop(opened);
    cleanup(&root);
}

/// **THÊM (2026-08-28, Story 5.5)** — §I/O Matrix "Tác phẩm có ghi đè thủ công": 1/2 Chương
/// `done`, ghi đè `paused` ⇒ `status_is_override = true` **VÀ** tiến độ vẫn đúng số Chương đã
/// xong THẬT (`Some(1)`), không bị ghi đè kéo theo. Đây là ca canh chỗ nối §Design Notes "Cái
/// bẫy ở `match status_override`" — đặt phép đếm bên TRONG nhánh `None` sẽ làm ca này đỏ vì
/// `chapter_done_count` sẽ không bao giờ được tính khi có ghi đè.
#[test]
fn overriding_the_work_status_never_changes_the_real_chapter_progress() {
    let root = temp_dir("override-keeps-progress");
    let mut opened = open_work(&root, "OverrideProgress");
    add_chapter(&opened, 2, "not_started"); // Chuong 1 (not_started) + Chuong 2 (not_started)

    let chapter_id = opened.chapter_id;
    set_chapter_status(Some(&mut opened), chapter_id, "done")
        .unwrap_or_else(|e| panic!("set_chapter_status: {e:?}"));

    let overridden = set_work_status_override(Some(&mut opened), Some("paused"))
        .unwrap_or_else(|e| panic!("set_work_status_override: {e:?}"));
    assert_eq!(overridden.status.as_deref(), Some("paused"));
    assert!(overridden.status_is_override);

    let on_disk = read_meta_from_disk(&opened.dir);
    assert_eq!(on_disk.status.as_deref(), Some("paused"));
    assert!(on_disk.status_is_override);
    assert_eq!(on_disk.chapter_count, 2);
    assert_eq!(
        on_disk.chapter_done_count,
        Some(1),
        "ghi de thu cong KHONG BAO GIO duoc phep doi tien do -- 1/2 Chuong done van la 1, \
         khong phai None hay 0"
    );

    drop(opened);
    cleanup(&root);
}

/// "Chương đổi SAU khi đã ghi đè": đang ghi đè `paused`, rồi mọi Chương thành `done` ⇒ tầng
/// Tác phẩm VẪN `paused`, `is_override = true` — hệ thống không suy ra đè lên.
#[test]
fn chapters_changing_after_an_override_does_not_disturb_the_override() {
    let root = temp_dir("override-survives-chapter-change");
    let mut opened = open_work(&root, "Survives");
    let chapter_id = opened.chapter_id;

    set_work_status_override(Some(&mut opened), Some("paused"))
        .unwrap_or_else(|e| panic!("set_work_status_override: {e:?}"));

    let lifecycle = set_chapter_status(Some(&mut opened), chapter_id, "done")
        .unwrap_or_else(|e| panic!("set_chapter_status: {e:?}"));
    assert_eq!(
        lifecycle.status.as_deref(),
        Some("paused"),
        "ghi de PHAI thang, ke ca sau khi Chuong doi trang thai"
    );
    assert!(lifecycle.status_is_override);

    drop(opened);
    cleanup(&root);
}

/// "Bỏ ghi đè": `set_work_status_override(None)` ⇒ `status_override` về `NULL`, đọc ra giá
/// trị SUY RA hiện thời, `is_override = false`.
#[test]
fn clearing_the_override_falls_back_to_the_derived_value() {
    let root = temp_dir("clear-override");
    let mut opened = open_work(&root, "Clear Override");
    let chapter_id = opened.chapter_id;

    set_chapter_status(Some(&mut opened), chapter_id, "done")
        .unwrap_or_else(|e| panic!("set_chapter_status: {e:?}"));
    set_work_status_override(Some(&mut opened), Some("paused"))
        .unwrap_or_else(|e| panic!("set_work_status_override paused: {e:?}"));

    let cleared = set_work_status_override(Some(&mut opened), None)
        .unwrap_or_else(|e| panic!("set_work_status_override None: {e:?}"));
    assert_eq!(
        cleared.status.as_deref(),
        Some("done"),
        "bo ghi de phai roi ve gia tri SUY RA hien thoi (mot Chuong DUY NHAT da Done)"
    );
    assert!(!cleared.status_is_override);

    let on_disk = read_meta_from_disk(&opened.dir);
    assert_eq!(on_disk.status.as_deref(), Some("done"));
    assert!(!on_disk.status_is_override);

    drop(opened);
    cleanup(&root);
}

/// "Giá trị ngoài danh mục (Tác phẩm)": `set_work_status_override(Some("archived"))` ⇒ không
/// một lượt ghi nào chạy, `err.lifecycle.unknown_status` `{status}`.
#[test]
fn overriding_with_an_unknown_status_writes_nothing_and_names_the_bad_value() {
    let root = temp_dir("unknown-override-status");
    let mut opened = open_work(&root, "Unknown Override");
    let before = read_meta_from_disk(&opened.dir);

    let err = set_work_status_override(Some(&mut opened), Some("archived"))
        .expect_err("gia tri ngoai danh muc phai bi tu choi");
    let json = serde_json::to_value(&err).expect("IpcError serialize duoc");
    assert_eq!(json["code"], "lifecycle.unknown_status");
    assert_eq!(json["params"]["status"], "archived");

    let after = read_meta_from_disk(&opened.dir);
    assert_eq!(before, after, "khong mot luot ghi nao duoc chay");

    drop(opened);
    cleanup(&root);
}

/// "Chưa Tác phẩm nào mở": bất kỳ lệnh vòng đời nào, `OpenWorkState = None` ⇒ không một lượt
/// ghi nào chạy, `work.none_open` (tái dùng `no_work_open()`).
#[test]
fn every_lifecycle_command_rejects_no_open_work() {
    let read_err = read_work_lifecycle(None).expect_err("phai tu choi khi chua mo Tac pham");
    assert_eq!(
        serde_json::to_value(&read_err).expect("serialize")["code"],
        "work.none_open"
    );

    let set_chapter_err =
        set_chapter_status(None, 1, "done").expect_err("phai tu choi khi chua mo Tac pham");
    assert_eq!(
        serde_json::to_value(&set_chapter_err).expect("serialize")["code"],
        "work.none_open"
    );

    let set_override_err = set_work_status_override(None, Some("paused"))
        .expect_err("phai tu choi khi chua mo Tac pham");
    assert_eq!(
        serde_json::to_value(&set_override_err).expect("serialize")["code"],
        "work.none_open"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Danh mục `LifecycleStatus::ALL` — đối chiếu với `vi.json`, và một hằng viết tay.
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 Con số viết tay là chỗ một con người phải ký — cùng lý lẽ `scope_kinds!::ALL`/
/// `message_keys!::ALL`. Gỡ hoặc thêm một hàng trong `lifecycle_statuses!` mà quên sửa hằng
/// này thì ca này ĐỎ.
const EXPECTED_LIFECYCLE_STATUS_COUNT: usize = 4;

#[test]
fn lifecycle_status_all_matches_the_hand_written_count() {
    assert_eq!(
        LifecycleStatus::ALL.len(),
        EXPECTED_LIFECYCLE_STATUS_COUNT,
        "LifecycleStatus::ALL doi do dai ma hang viet tay EXPECTED_LIFECYCLE_STATUS_COUNT \
         chua duoc sua theo -- day la mot con nguoi phai ky, khong phai mot phep tinh"
    );
}

/// Mỗi giá trị của `LifecycleStatus::ALL` có `label_key()` tồn tại trong `vi.json`.
#[test]
fn every_lifecycle_status_label_key_exists_in_vi_json() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let vi_json_path = manifest_dir
        .parent()
        .unwrap_or_else(|| panic!("CARGO_MANIFEST_DIR khong co thu muc cha"))
        .join("src/i18n/vi.json");
    let raw = fs::read_to_string(&vi_json_path)
        .unwrap_or_else(|e| panic!("doc {}: {e}", vi_json_path.display()));
    let catalog: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse vi.json: {e}"));
    let object = catalog.as_object().unwrap_or_else(|| panic!("vi.json phai la mot object"));

    let missing: Vec<&str> = LifecycleStatus::ALL
        .iter()
        .map(|status| status.label_key())
        .filter(|key| !object.contains_key(*key))
        .collect();
    assert!(
        missing.is_empty(),
        "{} khoa nhan cua LifecycleStatus khong co trong vi.json: {missing:?}",
        missing.len()
    );
}

/// Bảng suy ra KHÔNG BAO GIỜ trả `paused` — đối chứng qua đường LỆNH thật (không chỉ hàm
/// thuần `derive_work_status` ở `core/lifecycle/mod.rs`), quét MỌI tổ hợp của hai Chương
/// (4 × 4 = 16 tổ hợp, đủ để phủ mọi cặp giá trị trong bốn).
#[test]
fn no_combination_of_two_chapters_ever_derives_a_paused_work_through_the_command_layer() {
    let root = temp_dir("exhaustive-never-paused");
    let mut opened = open_work(&root, "Exhaustive");
    let chapter_id = opened.chapter_id;
    // Chương thứ hai chèn ĐÚNG MỘT LẦN, trước vòng lặp -- vòng lặp chỉ UPDATE đúng hàng này
    // theo `id`, không chèn thêm hàng mới mỗi lượt (`chapter.ord` KHÔNG có ràng buộc UNIQUE,
    // xem `schema.rs::CHAPTER_DDL`, nên một `INSERT` mỗi vòng sẽ chồng chất hàng rác).
    let second_chapter_id = add_chapter(&opened, 2, LifecycleStatus::NotStarted.as_str());

    for first in LifecycleStatus::ALL {
        for second in LifecycleStatus::ALL {
            let second_status = second.as_str().to_owned();
            opened
                .store
                .write(move |tx: &auratranslate_lib::core::store::Transaction<'_>| {
                    tx.execute(
                        "UPDATE chapter SET status = ?1 WHERE id = ?2",
                        (&second_status, second_chapter_id),
                    )
                })
                .unwrap_or_else(|e| panic!("cap nhat Chuong thu hai {second:?}: {e}"));

            // Một lượt ghi Chương THỨ NHẤT dựng lại `meta.json` từ TOÀN BỘ tập Chương hiện
            // tại (cả hai hàng) -- đây là lượt đọc trạng thái Tác phẩm cho tổ hợp này.
            let lifecycle = set_chapter_status(Some(&mut opened), chapter_id, first.as_str())
                .unwrap_or_else(|e| panic!("set_chapter_status {first:?}: {e:?}"));
            assert_ne!(
                lifecycle.status.as_deref(),
                Some("paused"),
                "to hop ({first:?}, {second:?}) khong duoc suy ra paused"
            );
        }
    }

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
//  BƯỚC 4 — "ghi trạng thái xong thì chỉ mục Library đã biết"
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 VÌ SAO HAI CA NÀY TỒN TẠI, VÀ VÌ SAO CHÚNG PHẢI GỌI HÀM `_indexed`
//
// ⚠️ Đo 2026-08-27, lượt nghiệm thu Story 5.4: bản dựng đầu đặt bước 4 bên trong `mod wire`.
// Gỡ hẳn hai lời gọi đó rồi chạy `cargo test --locked` cho **0 failed** trên toàn bộ 34
// binary — tức chỗ nối giữa "đã ghi trạng thái" và "hàng `library_work` đã mang giá trị mới"
// KHÔNG có ai canh, dù bộ test xanh. Đúng lớp lỗi mà `AGENTS.md::Known pitfalls` gọi tên
// (Epic 3 dính năm lần trong bảy ngày), và đúng đối chứng mà §Verification của story đòi.
//
// ⇒ Quy tắc chuyển xuống tầng hàm thuần (`set_chapter_status_indexed` /
// `set_work_status_override_indexed`), và hai ca dưới đây gọi CHÍNH hai hàm đó. Đối chứng để
// chạy lại về sau: gỡ khối `if result.is_ok() { reindex_after_lifecycle_write(...) }` khỏi
// một trong hai hàm ⇒ đúng ca tương ứng phải ĐỎ.

/// Dựng một `Indexer` + `global.db` cho thư mục tạm của ca — cùng khuôn
/// `library_commands_contract.rs::open_indexer`/`open_global`.
fn open_indexer(dir: &Path) -> auratranslate_lib::core::library::indexer::Indexer {
    auratranslate_lib::core::library::indexer::Indexer::open(dir.join("library-index.db"))
        .unwrap_or_else(|e| panic!("mo indexer: {e}"))
}

fn open_global(dir: &Path) -> auratranslate_lib::core::store::Store {
    auratranslate_lib::core::store::Store::open(auratranslate_lib::core::store::StoreSpec::global(
        dir.join("global.db"),
    ))
    .unwrap_or_else(|e| panic!("mo global.db: {e}"))
}

/// Đọc trạng thái của ĐÚNG một `work_id` trong chỉ mục, hoặc `None` nếu chỉ mục không có hàng
/// đó. Đọc lại từ đĩa — không tin giá trị mà lời gọi vừa trả về.
fn status_in_index(
    indexer: &auratranslate_lib::core::library::indexer::Indexer,
    work_id: &str,
) -> Option<(Option<String>, bool)> {
    indexer
        .list_works(auratranslate_lib::core::library::indexer::WorkQuery::default())
        .unwrap_or_else(|e| panic!("list_works: {e}"))
        .works
        .into_iter()
        .find(|w| w.work_id == work_id)
        .map(|w| (w.status.clone(), w.status_is_override))
}

#[test]
fn setting_a_chapter_status_leaves_the_new_value_in_the_library_index() {
    let root = temp_dir("chapter-status-reaches-index");
    let side = temp_dir("chapter-status-reaches-index-side");
    let indexer = open_indexer(&side);
    let global = open_global(&side);
    let mut opened = open_work(&root, "Tac pham chi muc");
    let work_id = opened.meta.work_id.clone();
    let chapter_id = opened.chapter_id;

    // Trạng thái ban đầu phải có mặt trong chỉ mục TRƯỚC khi ca này chứng minh được gì —
    // một chỉ mục rỗng làm khẳng định dưới đây đúng vì lý do sai.
    auratranslate_lib::commands::lifecycle::reindex_after_lifecycle_write(
        Some(&indexer),
        Some(&global),
        &root,
    );
    assert_eq!(
        status_in_index(&indexer, &work_id),
        Some((Some("not_started".to_owned()), false)),
        "truoc luot ghi, chi muc phai mang trang thai suy ra ban dau"
    );

    auratranslate_lib::commands::lifecycle::set_chapter_status_indexed(
        Some(&mut opened),
        Some(&indexer),
        Some(&global),
        &root,
        chapter_id,
        "done",
    )
    .unwrap_or_else(|e| panic!("set_chapter_status_indexed: {e:?}"));

    assert_eq!(
        status_in_index(&indexer, &work_id),
        Some((Some("done".to_owned()), false)),
        "sau luot ghi, hang library_work phai mang gia tri MOI -- neu ca nay xanh trong khi \
         buoc 4 da bi go thi cho noi khong co ai canh"
    );

    drop(opened);
    indexer.close();
    global.close();
    cleanup(&root);
    cleanup(&side);
}

#[test]
fn overriding_the_work_status_leaves_both_the_value_and_the_override_flag_in_the_index() {
    let root = temp_dir("override-reaches-index");
    let side = temp_dir("override-reaches-index-side");
    let indexer = open_indexer(&side);
    let global = open_global(&side);
    let mut opened = open_work(&root, "Tac pham ghi de");
    let work_id = opened.meta.work_id.clone();

    auratranslate_lib::commands::lifecycle::reindex_after_lifecycle_write(
        Some(&indexer),
        Some(&global),
        &root,
    );
    assert_eq!(
        status_in_index(&indexer, &work_id),
        Some((Some("not_started".to_owned()), false)),
        "truoc luot ghi de, chi muc phai noi day la gia tri SUY RA"
    );

    auratranslate_lib::commands::lifecycle::set_work_status_override_indexed(
        Some(&mut opened),
        Some(&indexer),
        Some(&global),
        &root,
        Some("paused"),
    )
    .unwrap_or_else(|e| panic!("set_work_status_override_indexed: {e:?}"));

    assert_eq!(
        status_in_index(&indexer, &work_id),
        Some((Some("paused".to_owned()), true)),
        "sau luot ghi de, chi muc phai mang CA gia tri LAN co ghi de -- co ghi de la thu \
         phan biet mot gia tri nguoi dat voi mot gia tri suy ra (AC6)"
    );

    // Bỏ ghi đè phải quay lại giá trị suy ra, và cờ phải tắt — cùng một chỗ nối, chiều ngược.
    auratranslate_lib::commands::lifecycle::set_work_status_override_indexed(
        Some(&mut opened),
        Some(&indexer),
        Some(&global),
        &root,
        None,
    )
    .unwrap_or_else(|e| panic!("bo ghi de: {e:?}"));
    assert_eq!(
        status_in_index(&indexer, &work_id),
        Some((Some("not_started".to_owned()), false)),
        "bo ghi de xong, chi muc phai quay ve gia tri suy ra hien thoi"
    );

    drop(opened);
    indexer.close();
    global.close();
    cleanup(&root);
    cleanup(&side);
}
