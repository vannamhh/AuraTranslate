//! Hành vi của **tầng lệnh** Library — Story 5.3, hai hàng §I/O Matrix mà
//! `library_index_contract.rs` không với tới được.
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

use auratranslate_lib::commands::library::{apply_chosen_root, forget_orphan, rescan};
use auratranslate_lib::core::library::indexer::Indexer;
use auratranslate_lib::core::library::meta::{META_SCHEMA_VERSION, WorkMeta};
use auratranslate_lib::core::scope::load_global_config;
use auratranslate_lib::core::store::{Store, StoreSpec};

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
    };
    meta.write_atomic(&dir)
        .unwrap_or_else(|e| panic!("ghi meta.json ở {}: {e}", dir.display()));
    fs::write(dir.join("project.db"), b"not a real sqlite file -- AD-9")
        .unwrap_or_else(|e| panic!("ghi project.db giả: {e}"));

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
    let root = library_root(&dir);
    write_atproj(&root, "First", "id-first", "Tac pham dau");

    let indexer = open_indexer(&dir);
    let before = rescan(Some(&indexer), &root).expect("lượt quét đầu");
    assert_eq!(before.indexed, 1);
    assert!(before.orphans.is_empty());

    // Người dùng copy một thư mục `.atproj` thứ hai vào bằng Finder/Explorer.
    write_atproj(&root, "Second", "id-second", "Tac pham hai");

    let after = rescan(Some(&indexer), &root).expect("lượt quét sau khi copy");
    assert_eq!(after.indexed, 2, "Tác phẩm vừa copy vào phải có mặt sau ĐÚNG một lượt quét");
    assert_eq!(after.conflicts, 0);
    assert_eq!(after.skipped, 0);
    assert!(
        after.orphans.is_empty(),
        "thêm một Tác phẩm KHÔNG được biến Tác phẩm cũ thành mồ côi"
    );

    let works = indexer.list_works().expect("list_works");
    let ids: Vec<&str> = works.iter().map(|w| w.work_id.as_str()).collect();
    assert!(ids.contains(&"id-first") && ids.contains(&"id-second"), "cả hai phải còn: {ids:?}");
    assert!(works.iter().all(|w| !w.orphaned));

    assert_eq!(after.root, root.display().to_string(), "báo cáo phải nêu đúng gốc vừa quét");

    drop(indexer);
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
        indexer.list_works().expect("list_works").is_empty(),
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

    let rescan_err = rescan(None, &root).expect_err("không có Indexer thì không quét được");
    assert_eq!(rescan_err.code(), "library.indexer_missing");

    let forget_err =
        forget_orphan(None, "id-bat-ky", "Ten bat ky").expect_err("không có Indexer thì không gỡ được");
    assert_eq!(forget_err.code(), "library.indexer_missing");

    let choose_err = apply_chosen_root(Some(&global), None, Some(&dir))
        .expect_err("không có Indexer thì lượt quét sau khi chọn phải nói ra");
    assert_eq!(choose_err.code(), "library.indexer_missing");

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
    let root = library_root(&dir); // chưa từng tạo

    let indexer = open_indexer(&dir);
    let report = rescan(Some(&indexer), &root).expect("rescan trên gốc vắng không phải lỗi");

    assert!(report.root_missing, "gốc chưa từng tồn tại phải báo root_missing = true");
    assert_eq!(report.indexed, 0);
    assert!(report.orphans.is_empty());

    drop(indexer);
    cleanup(&dir);
}

/// Gốc TỒN TẠI nhưng không chứa `.atproj` nào ⇒ `root_missing = false`, `indexed = 0` —
/// "đã quét, thật sự rỗng", một trạng thái KHÁC hẳn ca trên dù cùng mang `indexed == 0`.
#[test]
fn rescan_on_a_root_that_exists_but_is_truly_empty_reports_root_missing_false() {
    let dir = temp_dir("rescan-root-empty");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo gốc rỗng thật");

    let indexer = open_indexer(&dir);
    let report = rescan(Some(&indexer), &root).expect("rescan trên gốc rỗng không phải lỗi");

    assert!(
        !report.root_missing,
        "gốc CÓ tồn tại (dù rỗng) không được báo root_missing = true -- hai trạng thái này \
         phải phân biệt được, không phải cùng một con số 0 nói dối"
    );
    assert_eq!(report.indexed, 0);

    drop(indexer);
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
    let root = library_root(&dir);
    write_atproj(&root, "Alive", "id-alive", "Alive");

    let indexer = open_indexer(&dir);
    rescan(Some(&indexer), &root).expect("rescan");

    let err = forget_orphan(Some(&indexer), "id-alive", "Alive")
        .expect_err("hàng đang sống phải bị từ chối");
    assert_eq!(err.code(), "library.not_orphaned");
    assert_eq!(err.params().get("work_id").map(String::as_str), Some("id-alive"));
    // P9 (vòng rà THỨ HAI) -- `name` do CHỖ GỌI truyền vào phải có mặt trong `params`, không
    // chỉ `work_id` (một UUID trần không phải thứ người dùng nhận ra).
    assert_eq!(err.params().get("name").map(String::as_str), Some("Alive"));

    drop(indexer);
    cleanup(&dir);
}

/// Cùng mệnh đề, cho ca `work_id` LẠ (không tồn tại) — CÙNG một `code`, và `params["work_id"]`
/// phải mang đúng cái tên lạ đó, không phải một chuỗi rỗng hay một giá trị cũ sót lại.
#[test]
fn forget_orphan_at_the_command_layer_carries_the_right_code_and_work_id_param_for_an_unknown_id() {
    let dir = temp_dir("commands-forget-unknown");
    let root = library_root(&dir);
    fs::create_dir_all(&root).expect("tạo gốc");

    let indexer = open_indexer(&dir);
    rescan(Some(&indexer), &root).expect("rescan");

    let err = forget_orphan(Some(&indexer), "id-la-mot-cai-ten-la", "Ten hien thi luc bam nut")
        .expect_err("work_id lạ phải bị từ chối");
    assert_eq!(err.code(), "library.not_orphaned");
    assert_eq!(err.params().get("work_id").map(String::as_str), Some("id-la-mot-cai-ten-la"));
    assert_eq!(
        err.params().get("name").map(String::as_str),
        Some("Ten hien thi luc bam nut"),
        "ca work_id LA cung phai mang dung `name` da truyen vao -- khong roi mat no"
    );

    drop(indexer);
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
