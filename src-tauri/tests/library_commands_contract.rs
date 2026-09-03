//! Hành vi của **tầng lệnh** Library — Story 5.3, hai hàng §I/O Matrix mà
//! `library_index_contract.rs` không với tới được.
//!
//! 🔵 **SỬA (2026-08-27, Story 5.4)** — `Indexer::list_works()` đổi chữ ký (nhận `filter`,
//! trả `WorksReport { total, works }`); hai ca ở tệp này gọi `.list_works(None).<...>.works`
//! để giữ NGUYÊN hành vi cũ (đọc mọi hàng, không lọc) — xem doc-comment tương ứng ở
//! `library_index_contract.rs` cho lý do đầy đủ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO MỘT TỆP RIÊNG, KHÔNG NHÉT THÊM VÀO `library_index_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! Hai tệp, hai vai — cùng cách `store_contract.rs` và `project_contract.rs` tách nhau:
//! `library_index_contract.rs` canh **`Indexer`** (đối chiếu, mồ côi, trùng `work_id`);
//! tệp này canh **`commands::library`**, tức lớp trên nó — nơi ba con số của một lượt quét
//! được gói lại, nơi "huỷ hộp thoại" được quyết, và nơi `Indexer` vắng mặt phải thành một
//! `IpcError` thay vì một panic.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 HÀNG "HUỶ HỘP THOẠI" LÀ LÝ DO `apply_chosen_root` TỒN TẠI
//! ─────────────────────────────────────────────────────────────────────────────
//! `blocking_pick_folder()` cần một cửa sổ THẬT, nên khi nhánh huỷ còn nằm inline trong
//! `wire::library_choose_root` thì nó là một nhánh **không ca nào chạy được** — trong khi
//! §I/O Matrix của story có một hàng đòi đúng nhánh đó (*"không ghi cấu hình, không quét,
//! không một biến thể lỗi"*). `Option<&Path>` là ranh giới xa nhất còn viết được một ca hợp
//! đồng, đúng cùng lý lẽ mà `partition_dir_entries` đã tách khỏi `scan_atproj_dirs`.
//!
//! Ba luật của tệp này chép `library_index_contract.rs`: một thư mục tạm mỗi ca (pid +
//! `AtomicU64`, **không thêm `tempfile`**); `Store`/`Indexer` drop TRƯỚC khi xoá thư mục
//! (Windows từ chối xoá tệp đang mở); `project.db` trong mọi fixture là RÁC, nên bất kỳ
//! đường nào lỡ mở nó sẽ panic ngay tại ca đó (AD-9).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::library::{
    apply_chosen_root, forget_orphan, list_works, rescan, search_library,
};
use auratranslate_lib::core::library::indexer::{Indexer, MINIMUM_HARVEST_SCHEMA_VERSION, WorkQuery};
use auratranslate_lib::core::library::meta::{META_SCHEMA_VERSION, WorkMeta};
use auratranslate_lib::core::scope::load_global_config;
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-library-commands-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));
    dir
}

/// ⚠️ Gọi **sau** khi mọi `Store`/`Indexer` đã drop.
fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn open_global(dir: &Path) -> Store {
    Store::open(StoreSpec::global(dir.join("global.db"))).expect("mở global.db")
}

fn open_indexer(dir: &Path) -> Indexer {
    Indexer::open(dir.join("library-index.db")).unwrap_or_else(|e| panic!("mở indexer: {e}"))
}

fn library_root(dir: &Path) -> PathBuf {
    dir.join("library")
}

/// Dựng một `<folder>.atproj/` thật: `meta.json` qua đường ghi sản phẩm, `project.db` RÁC
/// (xem doc-comment đầu tệp — AD-9 nói `Indexer` không bao giờ mở tệp đó).
fn write_atproj(root: &Path, folder: &str, work_id: &str, name: &str) -> PathBuf {
    let dir = root.join(format!("{folder}.atproj"));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));

    let meta = WorkMeta {
        meta_schema_version: META_SCHEMA_VERSION,
        work_id: work_id.to_owned(),
        name: name.to_owned(),
        source_lang: "en".to_owned(),
        genre: String::new(),
        created_at: "2026-08-01T00:00:00.000Z".to_owned(),
        updated_at: "2026-08-01T00:00:00.000Z".to_owned(),
        chapter_count: 1,
        // 🔵 THÊM (2026-08-27, Story 5.4) — tệp này kiểm tầng LỆNH (ba con số/huỷ hộp
        // thoại), không trạng thái vòng đời; giá trị trung tính.
        status: Some("not_started".to_owned()),
        status_is_override: false,
        // 🔵 THÊM (2026-08-28, Story 5.5) — cùng lý lẽ ngay trên: tệp này không kiểm tiến độ,
        // giá trị trung tính `Some(0)`.
        chapter_done_count: Some(0),
    };
    meta.write_atomic(&dir)
        .unwrap_or_else(|e| panic!("ghi meta.json ở {}: {e}", dir.display()));
    fs::write(dir.join("project.db"), b"not a real sqlite file -- AD-9")
        .unwrap_or_else(|e| panic!("ghi project.db giả: {e}"));

    dir
}

/// **THÊM (vòng rà bốn lớp, Story 5.10, mục 2)** — khuôn THU HẸP của
/// `library_index_contract.rs::write_atproj_with_real_project_db`: đúng MỘT Chương, MỘT
/// segment, `project.db` THẬT (khác mọi fixture còn lại của tệp này, cố ý RÁC — xem doc-comment
/// đầu tệp). Cần thiết vì tầng LỆNH (`commands::library::search_library`) chưa từng chạy qua
/// `From<CoreSearchHit> for SearchHit` với một hit THẬT — mọi ca khác gọi nó trên fixture 0 hàng.
fn write_atproj_with_one_real_segment(
    root: &Path,
    folder: &str,
    work_id: &str,
    name: &str,
    source_text: &str,
    target_text: &str,
) -> PathBuf {
    let dir = write_atproj(root, folder, work_id, name);

    let db_path = dir.join("project.db");
    fs::remove_file(&db_path).unwrap_or_else(|e| panic!("xoa project.db rac o {}: {e}", db_path.display()));
    let store = Store::open(StoreSpec::project(db_path.clone()))
        .unwrap_or_else(|e| panic!("mo project.db that o {}: {e}", db_path.display()));

    let source_text = source_text.to_owned();
    let target_text = target_text.to_owned();
    store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (1, 'C1', 'irrelevant', 'draft', '2026-08-29T00:00:00.000Z', '2026-08-29T00:00:00.000Z')",
                (),
            )?;
            let chapter_id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO segment \
                 (chapter_id, ord, source_text, is_paragraph_end, created_at, updated_at, \
                  target_text, status, is_omitted, translation_origin) \
                 VALUES (?1, 1, ?2, 0, '2026-08-29T00:00:00.000Z', '2026-08-29T00:00:00.000Z', \
                         ?3, 'confirmed', 0, 'self')",
                (chapter_id, &source_text, &target_text),
            )?;
            Ok(())
        })
        .unwrap_or_else(|e| panic!("ghi fixture project.db that o {}: {e}", db_path.display()));

    drop(store);
    dir
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "`.atproj` MỚI copy vào"
// ═════════════════════════════════════════════════════════════════════════════════

/// Hàng đầu tiên của §I/O Matrix, và là câu chuyện người dùng NGUYÊN VĂN của FR99: *"copy
/// một thư mục `.atproj` vào là nó xuất hiện trong Library"*.
///
/// ⚠️ Ca này KHÁC `an_orphan_that_reappears_is_restored_without_a_second_row` ở
/// `library_index_contract.rs`: ở đó `work_id` đã từng có trong chỉ mục (một hàng mồ côi
/// sống lại); ở đây `work_id` là một cái tên chỉ mục **chưa bao giờ thấy** — đường UPSERT
/// phải chèn MỚI, không chỉ cập nhật. Hai nhánh khác nhau của cùng một câu SQL, nên hai ca.
#[test]
fn a_brand_new_atproj_copied_into_the_root_appears_after_one_rescan() {
    let dir = temp_dir("new-atproj");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj(&root, "First", "id-first", "Tac pham dau");

    let indexer = open_indexer(&dir);
    let before = rescan(Some(&indexer), Some(&global), &root).expect("lượt quét đầu");
    assert_eq!(before.indexed, 1);
    assert!(before.orphans.is_empty());

    // Người dùng copy một thư mục `.atproj` thứ hai vào bằng Finder/Explorer.
    write_atproj(&root, "Second", "id-second", "Tac pham hai");

    let after = rescan(Some(&indexer), Some(&global), &root).expect("lượt quét sau khi copy");
    assert_eq!(after.indexed, 2, "Tác phẩm vừa copy vào phải có mặt sau ĐÚNG một lượt quét");
    assert!(after.conflicts.is_empty());
    assert_eq!(after.skipped, 0);
    assert!(
        after.orphans.is_empty(),
        "thêm một Tác phẩm KHÔNG được biến Tác phẩm cũ thành mồ côi"
    );

    let works = indexer.list_works(WorkQuery::default()).expect("list_works").works;
    let ids: Vec<&str> = works.iter().map(|w| w.work_id.as_str()).collect();
    assert!(ids.contains(&"id-first") && ids.contains(&"id-second"), "cả hai phải còn: {ids:?}");

    assert_eq!(after.root, root.display().to_string(), "báo cáo phải nêu đúng gốc vừa quét");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Huỷ hộp thoại" và "Đổi thư mục gốc"
// ═════════════════════════════════════════════════════════════════════════════════

/// §I/O Matrix *"Huỷ hộp thoại"*: `Ok(None)` — **không** ghi cấu hình, **không** quét,
/// **không** một biến thể lỗi.
///
/// 🔴 Ba phép khẳng định, không một: kiểu trả về `Ok(None)` một mình KHÔNG chứng minh được
/// hai vế còn lại. Ca này đối chiếu cả `library_root` trên đĩa (phải vẫn `None`) lẫn nội
/// dung chỉ mục (một `.atproj` mới đặt vào gốc **trước** lượt huỷ phải vẫn CHƯA vào chỉ mục
/// sau lượt huỷ — nếu một lượt quét lén chạy, ca này đỏ).
#[test]
fn cancelling_the_folder_dialog_writes_no_config_and_leaves_the_index_alone() {
    let dir = temp_dir("dialog-cancel");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo gốc");

    let global = open_global(&dir);
    let indexer = open_indexer(&dir);

    // Một `.atproj` nằm sẵn trên đĩa nhưng CHƯA quét lần nào — nếu nhánh huỷ lỡ quét, nó sẽ
    // xuất hiện trong chỉ mục và ca này đỏ.
    write_atproj(&root, "Never-Scanned", "id-unscanned", "Chua quet");

    let out = apply_chosen_root(Some(&global), Some(&indexer), None).expect("huỷ không phải lỗi");
    assert!(out.is_none(), "huỷ hộp thoại phải là `Ok(None)`, không một biến thể lỗi");

    let cfg = load_global_config(&global).expect("đọc cấu hình");
    assert_eq!(cfg.library_root(), None, "huỷ KHÔNG được ghi `library_root` xuống đĩa");

    assert!(
        indexer.list_works(WorkQuery::default()).expect("list_works").works.is_empty(),
        "huỷ KHÔNG được kéo theo một lượt quét"
    );

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// §I/O Matrix *"Đổi thư mục gốc"* ở tầng lệnh: ghi cấu hình **và** quét ngay trên gốc mới,
/// trong cùng một lượt. `library_index_contract.rs` canh vế chỉ mục (hàng của gốc cũ thành
/// mồ côi); ca này canh vế **cấu hình đã xuống đĩa** — thứ quyết định lần khởi động sau.
#[test]
fn choosing_a_root_persists_it_and_rescans_that_root_in_the_same_call() {
    let dir = temp_dir("dialog-choose");
    let chosen = dir.join("chosen-root");
    fs::create_dir_all(&chosen).expect("tạo gốc mới");
    write_atproj(&chosen, "In-New-Root", "id-new-root", "Trong goc moi");

    let global = open_global(&dir);
    let indexer = open_indexer(&dir);

    let report = apply_chosen_root(Some(&global), Some(&indexer), Some(&chosen))
        .expect("chọn thư mục")
        .expect("chọn thật phải trả một báo cáo, không phải `None`");

    assert_eq!(report.root, chosen.display().to_string());
    assert_eq!(report.indexed, 1, "phải quét NGAY trên gốc vừa chọn");

    let cfg = load_global_config(&global).expect("đọc cấu hình");
    assert_eq!(
        cfg.library_root(),
        Some(chosen.display().to_string()),
        "lựa chọn phải xuống đĩa — nếu không, lần khởi động sau lại về gốc cũ mà không ai hiểu vì sao"
    );

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// Đường dẫn KHÔNG phải thư mục ⇒ từ chối, và từ chối **trước** khi ghi cấu hình. Một lượt
/// ghi rồi mới lỗi để lại một `library_root` trỏ vào một tệp — lần khởi động sau sẽ đọc nó.
#[test]
fn a_chosen_path_that_is_not_a_directory_is_refused_before_any_config_write() {
    let dir = temp_dir("choose-not-a-dir");
    let file = dir.join("mot-tep.txt");
    fs::write(&file, b"khong phai thu muc").expect("ghi tệp");

    let global = open_global(&dir);
    let indexer = open_indexer(&dir);

    let err = apply_chosen_root(Some(&global), Some(&indexer), Some(&file))
        .expect_err("một tệp không phải thư mục gốc hợp lệ");
    assert_eq!(err.code(), "library.root_invalid");

    let cfg = load_global_config(&global).expect("đọc cấu hình");
    assert_eq!(cfg.library_root(), None, "từ chối rồi thì KHÔNG được để lại gì trên đĩa");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// `Indexer` vắng mặt — một `IpcError`, không một panic
// ═════════════════════════════════════════════════════════════════════════════════

/// `lib.rs::open_library_index` ghi chẩn đoán rồi **đi tiếp** khi mở chỉ mục thất bại, nên
/// `app.manage(indexer)` có thể chưa từng chạy — cả ba lệnh phải nói ra điều đó thay vì
/// panic (`panic = "abort"` giết cả tiến trình).
#[test]
fn every_library_command_reports_a_missing_indexer_instead_of_panicking() {
    let dir = temp_dir("no-indexer");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo gốc");
    let global = open_global(&dir);

    let rescan_err = rescan(None, Some(&global), &root).expect_err("không có Indexer thì không quét được");
    assert_eq!(rescan_err.code(), "library.indexer_missing");

    let forget_err = forget_orphan(None, Some(&global), "id-bat-ky", "Ten bat ky")
        .expect_err("không có Indexer thì không gỡ được");
    assert_eq!(forget_err.code(), "library.indexer_missing");

    let choose_err = apply_chosen_root(Some(&global), None, Some(&dir))
        .expect_err("không có Indexer thì lượt quét sau khi chọn phải nói ra");
    assert_eq!(choose_err.code(), "library.indexer_missing");

    // 🔵 THÊM (2026-08-29, Story 5.9) — cùng khuôn ba lệnh trên: `search_library` không được
    // panic khi `Indexer` chưa quản lý, và nó tái dùng ĐÚNG khoá `library.indexer_missing`
    // (danh mục MessageKey ĐÓNG của story — không đúc khoá thứ ba).
    // 🔵 SỬA (2026-08-29, Story 5.10) — chữ ký `search_library` thêm tham số thứ TƯ (`mode:
    // Option<&str>`); `None` ⇒ mặc định `exact`, không đổi phán quyết của ca này.
    let search_err = search_library(None, "bat ky truy van nao", None, None)
        .expect_err("không có Indexer thì không tìm được");
    assert_eq!(search_err.code(), "library.indexer_missing");

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// P1 (vòng rà bốn lớp, 2026-08-27) — `RescanReport::root_missing` phải PHÂN BIỆT được
// "gốc không còn ở đó" với "gốc rỗng thật".
// ═════════════════════════════════════════════════════════════════════════════════

/// Gốc CHƯA TỪNG tồn tại ⇒ `root_missing = true`, `indexed = 0`. Trước bản vá P1,
/// `RebuildOutcome::root_missing` (đã tính đúng ở tầng `Indexer`) bị VỨT ở tầng lệnh, nên
/// `RescanReport` không nói được câu này — đúng lớp "rỗng im lặng" mà `AGENTS.md::Known
/// pitfalls` cấm.
#[test]
fn rescan_on_a_root_that_does_not_exist_reports_root_missing_true() {
    let dir = temp_dir("rescan-root-missing");
    let global = open_global(&dir);
    let root = library_root(&dir); // chưa từng tạo

    let indexer = open_indexer(&dir);
    let report = rescan(Some(&indexer), Some(&global), &root).expect("rescan trên gốc vắng không phải lỗi");

    assert!(report.root_missing, "gốc chưa từng tồn tại phải báo root_missing = true");
    assert_eq!(report.indexed, 0);
    assert!(report.orphans.is_empty());

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// Gốc TỒN TẠI nhưng không chứa `.atproj` nào ⇒ `root_missing = false`, `indexed = 0` —
/// "đã quét, thật sự rỗng", một trạng thái KHÁC hẳn ca trên dù cùng mang `indexed == 0`.
#[test]
fn rescan_on_a_root_that_exists_but_is_truly_empty_reports_root_missing_false() {
    let dir = temp_dir("rescan-root-empty");
    let global = open_global(&dir);
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo gốc rỗng thật");

    let indexer = open_indexer(&dir);
    let report = rescan(Some(&indexer), Some(&global), &root).expect("rescan trên gốc rỗng không phải lỗi");

    assert!(
        !report.root_missing,
        "gốc CÓ tồn tại (dù rỗng) không được báo root_missing = true -- hai trạng thái này \
         phải phân biệt được, không phải cùng một con số 0 nói dối"
    );
    assert_eq!(report.indexed, 0);

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// P6 (vòng rà bốn lớp, 2026-08-27) — `impl From<IndexError> for IpcError` không ca nào đi
// qua ở TẦNG LỆNH. Mọi ca hiện có bắt thẳng biến thể `IndexError` từ `Indexer::*`; đổi
// chuỗi `code`, đổi tên tham số, hay hoán hai nhánh `match` đều KHÔNG làm ca nào đỏ.
// ═════════════════════════════════════════════════════════════════════════════════

/// `commands::library::forget_orphan` (tầng LỆNH, không phải `Indexer::forget_orphan`) trên
/// một hàng ĐANG SỐNG phải trả đúng `code = "library.not_orphaned"` VÀ `params["work_id"]`
/// mang đúng id đã truyền vào — không chỉ "một lỗi nào đó".
#[test]
fn forget_orphan_at_the_command_layer_carries_the_right_code_and_work_id_param_for_a_live_row() {
    let dir = temp_dir("commands-forget-live");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj(&root, "Alive", "id-alive", "Alive");

    let indexer = open_indexer(&dir);
    rescan(Some(&indexer), Some(&global), &root).expect("rescan");

    let err = forget_orphan(Some(&indexer), Some(&global), "id-alive", "Alive")
        .expect_err("hàng đang sống phải bị từ chối");
    assert_eq!(err.code(), "library.not_orphaned");
    assert_eq!(err.params().get("work_id").map(String::as_str), Some("id-alive"));
    // P9 (vòng rà THỨ HAI) -- `name` do CHỖ GỌI truyền vào phải có mặt trong `params`, không
    // chỉ `work_id` (một UUID trần không phải thứ người dùng nhận ra).
    assert_eq!(err.params().get("name").map(String::as_str), Some("Alive"));

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// Cùng mệnh đề, cho ca `work_id` LẠ (không tồn tại) — CÙNG một `code`, và `params["work_id"]`
/// phải mang đúng cái tên lạ đó, không phải một chuỗi rỗng hay một giá trị cũ sót lại.
#[test]
fn forget_orphan_at_the_command_layer_carries_the_right_code_and_work_id_param_for_an_unknown_id() {
    let dir = temp_dir("commands-forget-unknown");
    let global = open_global(&dir);
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo gốc");

    let indexer = open_indexer(&dir);
    rescan(Some(&indexer), Some(&global), &root).expect("rescan");

    let err = forget_orphan(Some(&indexer), Some(&global), "id-la-mot-cai-ten-la", "Ten hien thi luc bam nut")
        .expect_err("work_id lạ phải bị từ chối");
    assert_eq!(err.code(), "library.not_orphaned");
    assert_eq!(err.params().get("work_id").map(String::as_str), Some("id-la-mot-cai-ten-la"));
    assert_eq!(
        err.params().get("name").map(String::as_str),
        Some("Ten hien thi luc bam nut"),
        "ca work_id LA cung phai mang dung `name` da truyen vao -- khong roi mat no"
    );

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// P11 (vòng rà THỨ HAI, 2026-08-27) — `apply_chosen_root` với `store: None` không ca nào
// chạm, dù doc-comment của nó khai nhánh "ghi cấu hình trượt ⇒ lỗi kho".
// ═════════════════════════════════════════════════════════════════════════════════

/// `store = None` ⇒ `put_config` bên trong `apply_chosen_root` phải trả lỗi *mở kho* (đi qua
/// `commands::config::put_config` ⇒ `store_is_missing()`), KHÔNG panic và KHÔNG âm thầm bỏ
/// qua bước ghi cấu hình. Đường dẫn được chọn hợp lệ (một thư mục có thật) để ca này chỉ đo
/// đúng MỘT biến — sự vắng mặt của `Store` — không lẫn với ca "đường dẫn không phải thư mục".
#[test]
fn choosing_a_root_with_no_global_store_reports_a_store_error_instead_of_silently_skipping_the_write() {
    let dir = temp_dir("choose-root-no-store");
    let chosen = dir.join("chosen-root");
    fs::create_dir_all(&chosen).expect("tạo gốc mới");

    let indexer = open_indexer(&dir);

    let err = apply_chosen_root(None, Some(&indexer), Some(&chosen))
        .expect_err("Store vắng mặt phải là một lỗi, không phải một lượt bỏ qua im lặng");
    assert_eq!(
        err.code(),
        "store.open_failed",
        "phải đi qua `commands::config::put_config` -- cùng khoá mà mọi lệnh ghi AppConfig khác dùng khi Store vắng mặt"
    );

    drop(indexer);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
//  `commands::library::list_works` — TẦNG LỆNH, thứ mà tầng `Indexer` không với tới
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ THÊM ở lượt rà 2026-08-28. `library_index_contract.rs` canh `Indexer::list_works`, nhưng
// nó nhận `LifecycleStatus` ĐÃ PHÂN GIẢI — nên hai mệnh đề của tầng lệnh không ca nào chạm:
// (a) một giá trị lọc lạ trên dây bị TỪ CHỐI chứ không bị bỏ qua im lặng, và (b) `matched`
// đếm số hàng SAU LỌC chứ không phải `total`. `ipc_contract.rs` dựng `WorkListReport` bằng
// struct literal nên nó cũng không chạy qua đường này. Vế (b) là đúng lỗi mà doc-comment của
// chính hàm đó nêu tên: *"Story 3.9 từng bịa `totalCount` bằng chính `filteredCount`"*.

/// Một giá trị lọc ngoài danh mục bốn giá trị phải cho `err.lifecycle.unknown_status` và
/// KHÔNG chạm SQL — không phải một danh sách rỗng trông như "không có gì khớp".
#[test]
fn an_unknown_filter_value_at_the_command_layer_is_refused_not_silently_dropped() {
    let dir = temp_dir("list-works-unknown-filter");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo thư mục gốc");
    let indexer = open_indexer(&dir);
    let global = open_global(&dir);
    write_atproj(&root, "Mot", "11111111-1111-4111-8111-111111111111", "Mot");
    indexer.rebuild(&root, Some(&global)).expect("lập chỉ mục");

    let filter = vec!["finished".to_owned()];
    let err = list_works(Some(&indexer), Some(&filter), None, None, None)
        .expect_err("gia tri loc la phai bi tu choi");
    assert_eq!(
        err.code(),
        "lifecycle.unknown_status",
        "mot gia tri loc la phai noi ten no, khong duoc tra ve mot danh sach rong"
    );

    // Đối chứng: cùng lời gọi với một giá trị HỢP LỆ vẫn đi qua bình thường — chứng minh ca
    // trên đỏ vì giá trị lạ, không vì đường lọc hỏng sẵn.
    let ok = list_works(Some(&indexer), Some(&vec!["not_started".to_owned()]), None, None, None)
        .expect("gia tri hop le phai di qua");
    assert_eq!(ok.matched, 1);

    indexer.close();
    global.close();
    cleanup(&dir);
}

/// `matched` là số hàng SAU LỌC, `total` là số hàng trong chỉ mục — và ca này chỉ có nghĩa khi
/// hai con số KHÁC nhau. Đây là hàng "bộ lọc quét sạch" của §I/O Matrix đi qua tầng LỆNH.
#[test]
fn the_command_layer_reports_matched_separately_from_total() {
    let dir = temp_dir("list-works-matched-vs-total");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo thư mục gốc");
    let indexer = open_indexer(&dir);
    let global = open_global(&dir);
    write_atproj(&root, "Mot", "11111111-1111-4111-8111-111111111111", "Mot");
    write_atproj(&root, "Hai", "22222222-2222-4222-8222-222222222222", "Hai");
    write_atproj(&root, "Ba", "33333333-3333-4333-8333-333333333333", "Ba");
    indexer.rebuild(&root, Some(&global)).expect("lập chỉ mục");

    // Cả ba `.atproj` đều `not_started` (xem `write_atproj`), nên lọc `done` quét sạch.
    let swept = list_works(Some(&indexer), Some(&vec!["done".to_owned()]), None, None, None).expect("lọc done");
    assert_eq!(swept.matched, 0, "khong hang nao khop");
    assert_eq!(
        swept.total, 3,
        "total phai la so hang TRONG CHI MUC -- man hinh can no de noi 'bo loc khong khop \
         hang nao tren 3 Tac pham' thay vi 'Library trong'"
    );
    assert!(swept.works.is_empty());

    // Không lọc: hai con số bằng nhau, và `matched` bám theo `works.len()`.
    let all = list_works(Some(&indexer), None, None, None, None).expect("khong loc");
    assert_eq!(all.total, 3);
    assert_eq!(all.matched, all.works.len());
    assert_eq!(all.matched, 3);

    indexer.close();
    global.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.5 — tiến độ Tác phẩm, đi trọn xuống tầng LỆNH (`WorkRow`).
// ═════════════════════════════════════════════════════════════════════════════════

/// Dựng một `<folder>.atproj/` với `meta.json` **HÌNH DẠNG V2 THẬT** (trước Story 5.5) — mang
/// `status`/`status_is_override` (đã có từ Story 5.4) nhưng THIẾU HẲN `chapter_done_count`.
/// Khác `write_v1_atproj_missing_lifecycle_fields` của `library_index_contract.rs`: ca đó mô
/// phỏng một `meta.json` TRƯỚC CẢ Story 5.4; ca này mô phỏng đúng lát cắt của §I/O Matrix
/// story 5.5 — "`meta.json` v2 (trước story này)".
fn write_v2_atproj_missing_chapter_done_count(root: &Path, folder: &str, work_id: &str, name: &str) -> PathBuf {
    let dir = root.join(format!("{folder}.atproj"));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));

    let raw = format!(
        "{{\n  \"meta_schema_version\": 2,\n  \"work_id\": {work_id:?},\n  \"name\": {name:?},\n  \
         \"source_lang\": \"en\",\n  \"genre\": \"\",\n  \
         \"created_at\": \"2026-08-01T00:00:00.000Z\",\n  \
         \"updated_at\": \"2026-08-01T00:00:00.000Z\",\n  \"chapter_count\": 3,\n  \
         \"status\": \"done\",\n  \"status_is_override\": false\n}}"
    );
    fs::write(dir.join("meta.json"), raw).unwrap_or_else(|e| panic!("ghi meta.json v2 gia: {e}"));
    fs::write(dir.join("project.db"), b"not a real sqlite file -- AD-9")
        .unwrap_or_else(|e| panic!("ghi project.db gia: {e}"));

    dir
}

/// §I/O Matrix: *"`meta.json` v2 (trước story này): khoá `chapter_done_count` vắng mặt ⇒ đọc
/// ra `None`; Library hiện câu 'chưa biết', không hiện `0 /`"* — đi trọn từ `meta.json` trên
/// đĩa xuống `WorkRow` ở tầng LỆNH (`commands::library::list_works`), không dừng ở tầng
/// `Indexer` như `library_index_contract.rs` đã canh.
#[test]
fn a_v2_meta_json_missing_chapter_done_count_reaches_the_work_row_as_none() {
    let dir = temp_dir("list-works-v2-missing-progress");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo thư mục gốc");
    let indexer = open_indexer(&dir);
    let global = open_global(&dir);
    write_v2_atproj_missing_chapter_done_count(
        &root,
        "Old",
        "11111111-1111-4111-8111-111111111111",
        "Old Work",
    );
    let outcome = indexer.rebuild(&root, Some(&global)).expect("lập chỉ mục");
    assert_eq!(outcome.indexed, 1, "meta.json v2 phai doc duoc va vao chi muc, khong bi skip");

    let report = list_works(Some(&indexer), None, None, None, None).expect("list_works");
    assert_eq!(report.works.len(), 1);
    assert_eq!(
        report.works[0].chapter_done_count, None,
        "khoa chapter_done_count vang mat tren dia phai di THANH None xuong WorkRow, khong \
         phai Some(0) -- doc lay 'chua biet', khong phai '0 Chuong da xong'"
    );
    // `status`/`status_is_override` cua Story 5.4 (da co trong v2) van doc dung, khong bi anh
    // huong boi khoa moi vang mat.
    assert_eq!(report.works[0].status.as_deref(), Some("done"));
    assert!(!report.works[0].status_is_override);

    indexer.close();
    global.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.6 — khoá sắp ở tầng LỆNH: `sort=None` ⇒ mặc định, khoá lạ ⇒ TỪ CHỐI.
// ═════════════════════════════════════════════════════════════════════════════════

/// §Always: "một khoá lạ trên dây ⇒ `IpcError`, KHÔNG im lặng rơi về mặc định" — cùng lý lẽ
/// đã canh cho `filter`, giờ áp dụng cho `sort`.
#[test]
fn an_unknown_sort_key_at_the_command_layer_is_refused_not_silently_defaulted() {
    let dir = temp_dir("list-works-unknown-sort");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo thư mục gốc");
    let indexer = open_indexer(&dir);
    let global = open_global(&dir);
    write_atproj(&root, "Mot", "11111111-1111-4111-8111-111111111111", "Mot");
    indexer.rebuild(&root, Some(&global)).expect("lập chỉ mục");

    let err = list_works(Some(&indexer), None, None, None, Some("bua"))
        .expect_err("khoa sap la phai bi tu choi");
    assert_eq!(
        err.code(),
        "library.unknown_sort",
        "mot khoa sap la phai noi ten no, khong duoc am tham roi ve mac dinh"
    );

    indexer.close();
    global.close();
    cleanup(&dir);
}

/// §I/O Matrix "Sắp mặc định": `sort=None` ⇒ `updated_desc` — hai Tác phẩm, cái sửa GẦN ĐÂY
/// HƠN phải đứng trước khi không truyền `sort`.
#[test]
fn no_sort_key_defaults_to_updated_desc() {
    let dir = temp_dir("list-works-default-sort");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo thư mục gốc");
    let indexer = open_indexer(&dir);
    let global = open_global(&dir);
    write_atproj(&root, "Old", "11111111-1111-4111-8111-111111111111", "Old");
    write_atproj(&root, "New", "22222222-2222-4222-8222-222222222222", "New");
    indexer.rebuild(&root, Some(&global)).expect("lập chỉ mục");

    let report = list_works(Some(&indexer), None, None, None, None).expect("list_works mac dinh");
    assert_eq!(report.works.len(), 2);
    // `write_atproj` (khuôn của tệp `library_index_contract.rs`) ghi CÙNG một `updated_at` cho
    // mọi hàng -- không đối chứng được thứ tự NGÀY SỬA ở đây (đã canh trọn ở
    // `library_index_contract.rs::sorting_by_updated_at_orders_the_most_recently_touched_work_first`).
    // Ca này chỉ canh rằng KHÔNG lỗi nào ném khi `sort` vắng mặt, và tầng lệnh không tự bịa
    // một chuỗi rỗng cho tham số `sort` (thứ sẽ trượt `WorkSortKey::from_wire`).
    assert_eq!(report.matched, 2);

    indexer.close();
    global.close();
    cleanup(&dir);
}

/// **THÊM Story 5.10 (FR9)** — khuôn TRỰC TIẾP của
/// `an_unknown_sort_key_at_the_command_layer_is_refused_not_silently_defaulted` ngay trên,
/// cho tham số `mode`: một chuỗi lạ trên dây phải bị TỪ CHỐI, không âm thầm rơi về `exact`.
#[test]
fn an_unknown_search_mode_at_the_command_layer_is_refused_not_silently_defaulted() {
    let dir = temp_dir("search-unknown-mode");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo thư mục gốc");
    let indexer = open_indexer(&dir);
    let global = open_global(&dir);
    write_atproj(&root, "Mot", "11111111-1111-4111-8111-111111111111", "Mot");
    indexer.rebuild(&root, Some(&global)).expect("lập chỉ mục");

    let err = search_library(Some(&indexer), "bat ky truy van nao", None, Some("fuzzy"))
        .expect_err("mot che do la phai bi tu choi");
    assert_eq!(
        err.code(),
        "library.unknown_search_mode",
        "mot che do la phai noi ten no, khong duoc am tham roi ve mac dinh"
    );

    indexer.close();
    global.close();
    cleanup(&dir);
}

/// §I/O Matrix "`mode` vắng mặt": `mode=None` ⇒ chạy như `exact` — mặc định không bao giờ là
/// khoan dung (AD-27 · AC4). Ca này chỉ canh KHÔNG lỗi nào ném và `mode`/`effective_mode` trả
/// về đúng `"exact"`; nội dung hit thuộc `library_index_contract.rs`.
#[test]
fn no_search_mode_defaults_to_exact() {
    let dir = temp_dir("search-default-mode");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo thư mục gốc");
    let indexer = open_indexer(&dir);
    let global = open_global(&dir);
    write_atproj(&root, "Mot", "11111111-1111-4111-8111-111111111111", "Mot");
    indexer.rebuild(&root, Some(&global)).expect("lập chỉ mục");

    let report = search_library(Some(&indexer), "bat ky truy van nao", None, None).expect("search mac dinh");
    assert_eq!(report.mode, "exact", "mode=None phai duoc doc la exact, khong doan mo");
    assert_eq!(report.effective_mode, "exact");
    assert!(!report.widened);

    indexer.close();
    global.close();
    cleanup(&dir);
}

/// **THÊM (vòng rà bốn lớp, mục 2)** — `match_kind` trên STRUCT DÂY
/// (`commands::library::SearchHit`) chưa từng chạy qua `From<CoreSearchHit>` với một hit THẬT
/// trước ca này: `ipc_contract.rs` chỉ dựng struct TAY (`match_kind: "exact".to_owned()`, bỏ
/// qua hẳn phép chuyển đổi), và mọi ca khác của tệp này gọi `search_library` trên fixture 0
/// hàng (`project.db` RÁC). Một hit `khoáng sản` chỉ khớp truy vấn `khoang` qua chỉ mục `_nd`
/// (§Design Notes của `5-10-hai-che-do-dau.md`) — đúng ca cần `mode = Lenient` để phân biệt.
///
/// 🔴 Đối chứng "ca này đỏ được": hạ `impl From<CoreSearchHit> for SearchHit`
/// (`commands/library.rs`) xuống `match_kind: MatchKind::Exact.as_str().to_owned()` cứng (bỏ
/// đọc `hit.match_kind.as_str()` — một lỗi chép-dán rất thật, hai dòng bên cạnh nó đúng hình
/// dạng "đọc trường rồi `.as_str().to_owned()`") ⇒ ca này FAIL, vì hit `lenient` sẽ khai
/// `"exact"` trên dây.
#[test]
fn search_library_carries_a_lenient_match_kind_through_to_the_wire_struct() {
    let dir = temp_dir("search-match-kind-wire");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo thư mục gốc");
    let indexer = open_indexer(&dir);
    let global = open_global(&dir);
    write_atproj_with_one_real_segment(&root, "Solo", "id-solo", "Solo", "irrelevant", "khoáng sản");
    indexer.rebuild(&root, Some(&global)).expect("lập chỉ mục");

    let report = search_library(Some(&indexer), "khoang", None, Some("lenient")).expect("search lenient");
    assert_eq!(report.hits.len(), 1, "hit: {:?}", report.hits);
    assert_eq!(
        report.hits[0].match_kind, "lenient",
        "match_kind tren STRUCT DAY phai la 'lenient' -- neu day la 'exact' thi \
         `From<CoreSearchHit>` da hoa mot hang thay vi doc `hit.match_kind`"
    );
    assert_eq!(report.hits[0].field, "target");

    indexer.close();
    global.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// retro Epic 5, AI-2/AI-3 (vòng rà lần hai, mục 1) — hai `From` ở BIÊN IPC chưa từng chạy
// qua một lượt gọi THẬT của `commands::library::rescan`/`search_library` trước cụm này.
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 LỖ HỔNG ĐÃ ĐO ĐƯỢC, KHÔNG PHẢI NGHI NGỜ (vòng rà lần hai): đảo `works_total`/
// `works_with_text` trong `From<CoreSearchReport> for SearchReport` (`commands/library.rs`)
// và CHẠY TOÀN BỘ BỘ TEST RUST -- 35/35 crate test giữ nguyên `ok`, 0 lỗi. `ipc_contract.rs`
// chỉ dựng struct TAY (không đi qua `From`); `library_index_contract.rs` chỉ đọc
// `core::library::indexer::SearchReport` (TRƯỚC lượt chuyển đổi). Không tệp nào gọi
// `commands::library::search_library`/`rescan` trên một fixture có SỐ LỆCH NHAU rồi đọc lại
// hai trường này qua đúng cổng IPC. `grep "text_skipped\|works_total\|works_with_text"
// tests/library_commands_contract.rs` cho 0 trước cụm ca này.

/// Hai Tác phẩm: MỘT harvest được (`project.db` THẬT, đích mới nhất), MỘT trượt vì lược đồ
/// CŨ hơn sàn (đúng khuôn `a_project_db_older_than_the_harvest_floor_…` của
/// `library_index_contract.rs`) — dựng MỘT LẦN, dùng CHUNG cho cả hai lời khẳng định dưới
/// đây (`rescan` rồi `search_library`, cùng `indexer`/`global`, không dựng lại chỉ mục giữa
/// chừng).
#[test]
fn rescan_and_search_library_report_the_two_new_wire_fields_through_the_real_conversions() {
    let dir = temp_dir("commands-text-skipped-and-coverage");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo thư mục gốc");
    let global = open_global(&dir);
    let indexer = open_indexer(&dir);

    // Tác phẩm HARVEST ĐƯỢC — `project.db` THẬT, đích mới nhất, một segment tìm được.
    write_atproj_with_one_real_segment(
        &root,
        "Fresh",
        "id-fresh",
        "Fresh",
        "irrelevant nguon",
        "uniquefreshtargettext",
    );

    // Tác phẩm TRƯỢT THU HOẠCH — `project.db` THẬT nhưng bị hạ NGAY DƯỚI sàn, đúng khuôn
    // `a_project_db_older_than_the_harvest_floor_…`.
    let old_dir = write_atproj_with_one_real_segment(
        &root,
        "OldSchema",
        "id-old-schema",
        "OldSchema",
        "irrelevant nguon cu",
        "irrelevant dich cu",
    );
    {
        let db_path = old_dir.join("project.db");
        let conn = rusqlite::Connection::open(&db_path).expect("mo lai de ha phien ban");
        conn.pragma_update(None, "user_version", i64::from(MINIMUM_HARVEST_SCHEMA_VERSION - 1))
            .expect("ha user_version xuong ngay duoi san");
        conn.execute("ALTER TABLE segment DROP COLUMN is_omitted", [])
            .expect("go cot is_omitted -- dung hinh dang mot project.db THAT ngay duoi san");
    }

    // ── Lời khẳng định 1: `rescan()` THẬT, không hàm thuần nội bộ nào của `Indexer` ──
    let report = rescan(Some(&indexer), Some(&global), &root).expect("rescan");
    assert_eq!(report.indexed, 2, "metadata cua CA HAI Tac pham van UPSERT binh thuong");
    assert_eq!(
        report.text_skipped.len(),
        1,
        "dung MOT Tac pham bi bo qua phan van ban: {:?}",
        report.text_skipped
    );
    assert_eq!(report.text_skipped[0].work_id, "id-old-schema");
    // 🔴 Mã lý do ĐI QUA `From<TextHarvestSkipped> for TextSkippedEntry` thật -- không dựng
    // tay struct này. Nếu `From` đọc `.diagnostic()` thay vì `.code()`, chuỗi ở đây sẽ là một
    // câu chẩn đoán dài mang cả số phiên bản, không phải mã ổn định `"schema_too_old"`.
    assert_eq!(
        report.text_skipped[0].reason,
        "schema_too_old",
        "reason tren day PHAI la ma on dinh tu HarvestSkipReason::code(), khong phai \
         HarvestSkipReason::diagnostic() (mot chuoi tho, dai, mang so phien ban)"
    );

    // ── Lời khẳng định 2: `search_library()` THẬT, cùng chỉ mục vừa `rescan()` dựng ──
    let search = search_library(Some(&indexer), "uniquefreshtargettext", None, None)
        .expect("search_library");
    assert_eq!(search.hits.len(), 1, "chi Tac pham harvest duoc moi co van ban de khop: {:?}", search.hits);
    // 🔴 Hai khẳng định RIÊNG BIỆT, giá trị KHÁC NHAU (2 ≠ 1) — một phép đảo `works_total`/
    // `works_with_text` trong `From<CoreSearchReport>` sẽ làm ĐÚNG MỘT trong hai dòng dưới
    // đây đỏ (không phải cả hai cùng lúc đổi bù cho nhau, vì `assert_eq!` so một hằng cụ thể
    // cho từng vế, không so lẫn nhau).
    assert_eq!(search.works_total, 2, "works_total phai dem CA HAI hang library_work");
    assert_eq!(
        search.works_with_text, 1,
        "works_with_text chi dem Tac pham THAT SU co van ban trong chi muc"
    );

    indexer.close();
    global.close();
    cleanup(&dir);
}
