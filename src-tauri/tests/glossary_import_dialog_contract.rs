//! Hợp đồng Story 3.10b — hộp thoại chọn tệp nối vào xuất/nhập Glossary (AD-48).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO TỆP RIÊNG, KHÔNG GỘP VÀO `glossary_exchange_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! `glossary_exchange_contract.rs` là hợp đồng Story 3.10 (định dạng thuần + đường ghi,
//! KHÔNG chạm hệ thống tệp). Tệp này là hợp đồng của story SAU, đóng đúng nửa còn lại: I/O
//! tệp (`core::glossary::exchange_io`) và bề mặt lô nhập TREO giữa hai nhịp
//! (`commands::glossary::{glossary_open_import_preview, glossary_confirm_import,
//! glossary_cancel_import, clear_pending_import_for_tier}`) — một tệp, một story, cùng
//! khuôn `glossary_boundary.rs`/`glossary_contract.rs`.
//!
//! Bốn luật thừa kế từ `glossary_commands_contract.rs`: mỗi ca một thư mục tạm riêng ·
//! Drop `Store`/`OpenWork` TRƯỚC khi xoá thư mục · không `sleep` dài · không ca nào treo
//! khi nó trượt.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::glossary::{
    PendingImportState, clear_pending_import_for_tier, glossary_cancel_import,
    glossary_confirm_import, glossary_export_tier, glossary_open_import_preview,
};
use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::core::glossary::{Category, ConflictDecision, GlossaryTier, add_manual_term};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::store::{Store, StoreSpec};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-glossary-import-dialog-{}-{}-{}",
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

fn write_file(dir: &Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).unwrap_or_else(|e| panic!("ghi {}: {e}", path.display()));
    path
}

// ═════════════════════════════════════════════════════════════════════════════════
// exchange_io — trần kích thước, phi-UTF-8, ghi nguyên tử
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix "Nhập, tệp vượt trần" — từ chối tường minh mang SỐ BYTE và TRẦN, kiểm bằng
/// `metadata` (0 byte đọc vào bộ nhớ cho một tệp đã vượt trần — không cách nào chứng minh
/// trực tiếp "0 byte đọc" từ ngoài API công khai, nhưng lỗi trả về đúng phải là
/// `ImportFileTooLarge`, KHÔNG phải một lỗi phân tích/UTF-8 nào khác, tức bước đo kích
/// thước đã chặn TRƯỚC khi tới bước đọc nội dung).
#[test]
fn a_file_over_the_sixteen_mib_cap_is_refused_by_size_before_content_is_touched() {
    let root = temp_dir("too-large");
    let global_dir = temp_dir("too-large-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    let path = root.join("huge.csv");
    {
        use std::io::{Seek, SeekFrom, Write as _};
        let mut file = fs::File::create(&path).unwrap_or_else(|e| panic!("tao {}: {e}", path.display()));
        // 16 MiB + 1 byte -- sparse tren dia, khong cap phat trong bo nho cua TEST.
        file.seek(SeekFrom::Start(16 * 1024 * 1024 + 1)).unwrap();
        file.write_all(b"x").unwrap();
    }

    let err = glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &path)
        .expect_err("tep vuot tran phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::ImportTooLarge);
    assert_eq!(err.params().get("limit").map(String::as_str), Some("16777216"));
    assert!(
        pending.lock().unwrap().is_none(),
        "0 lo nao duoc giu lai khi buoc doc trot -- khong lo nao ro ri tu mot lan doc that bai"
    );

    cleanup(&root);
    cleanup(&global_dir);
}

/// I/O Matrix "Nhập, tệp phi-UTF-8" — từ chối tường minh, không đoán bảng mã.
#[test]
fn a_non_utf8_file_is_refused_explicitly_not_guessed() {
    let root = temp_dir("not-utf8");
    let global_dir = temp_dir("not-utf8-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    let path = root.join("bad.csv");
    fs::write(&path, [0xff, 0xfe, 0x00, 0x01]).unwrap();

    let err = glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &path)
        .expect_err("byte khong phai UTF-8 phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::ImportNotUtf8);
    assert!(pending.lock().unwrap().is_none());

    cleanup(&root);
    cleanup(&global_dir);
}

/// Ghi nguyên tử: xuất thành công không để lại tệp `.tmp` nào, và nội dung khớp
/// `export_tier` — đi qua ĐÚNG bề mặt `commands::glossary::glossary_export_tier`
/// (hàm thuần, path đã "chọn" sẵn — hộp thoại thật chỉ nghiệm thu được bằng tay,
/// `npm run tauri dev`).
#[test]
fn exporting_writes_atomically_and_leaves_no_tmp_file_behind() {
    let root = temp_dir("export-atomic");
    let global_dir = temp_dir("export-atomic-global");
    let global = open_global(&global_dir);

    let path = root.join("out.csv");
    glossary_export_tier(Some(&global), None, GlossaryTier::Global, &path)
        .expect("xuat mot tang rong van phai thanh cong");

    assert!(path.exists());
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.starts_with("source_term,translation,note,category,term_origin,created_at\n"));
    assert!(!root.join("out.csv.tmp").exists(), "khong tep tam nao duoc de lai sau mot luot ghi thanh cong");

    cleanup(&root);
    cleanup(&global_dir);
}

/// §I/O Matrix "Xuất, tầng Tác phẩm chưa mở" — `WorkTierUnavailable`, **0** lượt ghi (kể cả
/// việc dò đuôi tệp/ghi tạm — hàm trả lỗi TRƯỚC khi chạm `exchange_io`).
#[test]
fn exporting_the_work_tier_without_an_open_work_fails_before_touching_disk() {
    let root = temp_dir("export-no-work");
    let global_dir = temp_dir("export-no-work-global");
    let global = open_global(&global_dir);

    let path = root.join("out.csv");
    let err = glossary_export_tier(Some(&global), None, GlossaryTier::Work, &path)
        .expect_err("tang Work chua mo phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::GlossaryWorkTierUnavailable);
    assert!(!path.exists(), "0 luot ghi khi tang Work chua mo");

    cleanup(&root);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// PendingImportState — nhịp một/nhịp hai/huỷ
// ═════════════════════════════════════════════════════════════════════════════════

/// §I/O Matrix "Nhập, xác nhận khi không có lô nào" — lỗi tường minh, **0** lượt ghi.
#[test]
fn confirming_with_no_pending_batch_fails_readably_and_writes_nothing() {
    let global_dir = temp_dir("confirm-no-pending-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    let err = glossary_confirm_import(Some(&global), None, &pending, &BTreeMap::new())
        .expect_err("xac nhan khi khong co lo phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::GlossaryNoPendingImport);

    cleanup(&global_dir);
}

/// §I/O Matrix "Nhập, quyết định trỏ thuật ngữ lạ" — lỗi tường minh mang thuật ngữ đó,
/// **0** lượt ghi, và LÔ GIỮ LẠI (không rơi vào hư không, không bị dọn vì lỗi).
#[test]
fn confirming_with_a_decision_pointing_at_an_unknown_term_fails_and_keeps_the_batch() {
    let root = temp_dir("confirm-unknown-term");
    let global_dir = temp_dir("confirm-unknown-term-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    let path = write_file(&root, "in.csv", "source_term,translation\nmoc,Moc Dung\n");
    let preview = glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &path)
        .expect("mo va xem truoc phai thanh cong");
    assert_eq!(preview.new_count, 1);

    let mut decisions = BTreeMap::new();
    decisions.insert("khong-co-trong-lo".to_owned(), ConflictDecision::TakeTheirs);

    let err = glossary_confirm_import(Some(&global), None, &pending, &decisions)
        .expect_err("mot khoa la trong ban do quyet dinh phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::GlossaryImportDecisionUnknownTerm);
    assert_eq!(err.params().get("value").map(String::as_str), Some("khong-co-trong-lo"));

    assert!(
        pending.lock().unwrap().is_some(),
        "lo phai GIU LAI de thu lai -- khong bi don vi mot quyet dinh sai"
    );

    cleanup(&root);
    cleanup(&global_dir);
}

/// §I/O Matrix "Nhập, mở lô thứ hai khi lô cũ còn treo" — lô MỚI thay lô CŨ; lô cũ không
/// bao giờ ghi được nữa (xác nhận SAU khi mở lô thứ hai chỉ áp dụng cho nội dung tệp THỨ
/// HAI, không còn dấu vết của tệp đầu).
#[test]
fn opening_a_second_batch_replaces_the_first_one_entirely() {
    let root = temp_dir("second-batch-replaces-first");
    let global_dir = temp_dir("second-batch-replaces-first-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    let first = write_file(&root, "first.csv", "source_term,translation\nmoc,Moc Dung\n");
    let preview_one = glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &first)
        .expect("mo lo dau phai thanh cong");
    assert_eq!(preview_one.file_name, "first.csv");

    let second = write_file(&root, "second.csv", "source_term,translation\nly,Ly\nvuong,Vuong\n");
    let preview_two = glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &second)
        .expect("mo lo thu hai phai thanh cong");
    assert_eq!(preview_two.file_name, "second.csv");
    assert_eq!(preview_two.new_count, 2);

    // Xac nhan ghi dung TAP DU LIEU CUA LO THU HAI, khong con dau vet cua lo dau.
    let summary = glossary_confirm_import(Some(&global), None, &pending, &BTreeMap::new())
        .expect("xac nhan lo thu hai phai thanh cong");
    assert_eq!(summary.inserted, 2);
    assert!(pending.lock().unwrap().is_none(), "lo don sau khi xac nhan thanh cong");

    cleanup(&root);
    cleanup(&global_dir);
}

/// Huỷ lô đang treo — **0** lượt ghi, và huỷ khi không có lô nào là vô hại (không lỗi).
#[test]
fn cancelling_clears_the_pending_batch_and_is_harmless_when_there_is_none() {
    let root = temp_dir("cancel");
    let global_dir = temp_dir("cancel-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    glossary_cancel_import(&pending); // Vo hai khi chua co lo nao.
    assert!(pending.lock().unwrap().is_none());

    let path = write_file(&root, "in.csv", "source_term,translation\nmoc,Moc Dung\n");
    glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &path)
        .expect("mo phai thanh cong");
    assert!(pending.lock().unwrap().is_some());

    glossary_cancel_import(&pending);
    assert!(pending.lock().unwrap().is_none());

    let err = glossary_confirm_import(Some(&global), None, &pending, &BTreeMap::new())
        .expect_err("xac nhan sau khi huy phai tra NoPendingImport");
    assert_eq!(err.message_key(), MessageKey::GlossaryNoPendingImport);

    cleanup(&root);
    cleanup(&global_dir);
}

/// §I/O Matrix "Đóng Tác phẩm khi còn lô nhập treo" — lô tầng Work bị DỌN;
/// `clear_pending_import_for_tier` là hàm thuần mà `lib.rs`/`commands::project` gọi ở
/// đúng hai chỗ `OpenWorkState` thay đổi (đóng Tác phẩm · mở một Tác phẩm KHÁC). Ca này
/// kiểm chính hàm đó, không cần dựng cửa sổ Tauri thật.
#[test]
fn a_work_tier_pending_batch_is_cleared_when_the_work_closes_but_a_global_one_survives() {
    let root = temp_dir("clear-on-close");
    let global_dir = temp_dir("clear-on-close-global");
    let global = open_global(&global_dir);
    let opened = create_work_from_text(&root, "Muc Cong", "zh", "", "noi dung mau".to_owned())
        .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));

    let path = write_file(&root, "in.csv", "source_term,translation\nmoc,Moc Dung\n");
    let pending_work = PendingImportState::new(None);
    glossary_open_import_preview(Some(&global), Some(&opened), &pending_work, GlossaryTier::Work, &path)
        .expect("mo lo tang Work phai thanh cong");
    assert!(pending_work.lock().unwrap().is_some());

    // Tac pham "dong" -- goi dung ham thuan ma lib.rs/commands::project goi o hai cho
    // OpenWorkState chuyen doi. Lo tang Work phai bi don.
    clear_pending_import_for_tier(&pending_work, GlossaryTier::Work);
    assert!(pending_work.lock().unwrap().is_none(), "lo tang Work phai bi don khi Tac pham dong");

    // Doi chung: mot lo tang GLOBAL khong bi cham toi boi cung mot loi goi.
    let pending_global = PendingImportState::new(None);
    let path2 = write_file(&root, "in2.csv", "source_term,translation\nly,Ly\n");
    glossary_open_import_preview(Some(&global), None, &pending_global, GlossaryTier::Global, &path2)
        .expect("mo lo tang Global phai thanh cong");
    clear_pending_import_for_tier(&pending_global, GlossaryTier::Work);
    assert!(
        pending_global.lock().unwrap().is_some(),
        "lo tang Global khong bi don boi mot loi goi 'don tang Work'"
    );

    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ma trận I/O còn lại — đuôi tệp quyết dấu phân cách, cột term_origin, cột lạ, tệp hỏng,
// tệp 0 hàng dữ liệu — đi qua ĐÚNG bề mặt `commands::glossary`, không chỉ qua `exchange::
// parse` thuần (đã đóng ở `glossary_exchange_contract.rs`, Story 3.10).
// ═════════════════════════════════════════════════════════════════════════════════

/// §I/O Matrix "Xuất, đuôi tệp quyết dấu phân cách" — đuôi THẬT của đường dẫn đã chọn,
/// không trạng thái UI trước đó. `.tsv` ⇒ TAB; đuôi lạ (kể cả không đuôi) ⇒ CSV.
#[test]
fn the_delimiter_follows_the_real_extension_of_the_chosen_path_not_prior_ui_state() {
    let root = temp_dir("delimiter-from-extension");
    let global_dir = temp_dir("delimiter-from-extension-global");
    let global = open_global(&global_dir);

    let csv_path = root.join("a.csv");
    glossary_export_tier(Some(&global), None, GlossaryTier::Global, &csv_path).expect("xuat .csv");
    assert_eq!(
        fs::read_to_string(&csv_path).unwrap().lines().next(),
        Some("source_term,translation,note,category,term_origin,created_at")
    );

    let tsv_path = root.join("b.tsv");
    glossary_export_tier(Some(&global), None, GlossaryTier::Global, &tsv_path).expect("xuat .tsv");
    assert_eq!(
        fs::read_to_string(&tsv_path).unwrap().lines().next(),
        Some("source_term\ttranslation\tnote\tcategory\tterm_origin\tcreated_at")
    );

    let odd_path = root.join("c.glossary");
    glossary_export_tier(Some(&global), None, GlossaryTier::Global, &odd_path).expect("xuat duoi la");
    assert_eq!(
        fs::read_to_string(&odd_path).unwrap().lines().next(),
        Some("source_term,translation,note,category,term_origin,created_at"),
        "duoi la phai roi ve CSV"
    );

    cleanup(&root);
    cleanup(&global_dir);
}

/// §I/O Matrix "Nhập, tệp có cột term_origin" — xem trước NÓI RA rằng cột này bị đọc rồi
/// bỏ, đóng `deferred-work.md:6763`.
#[test]
fn opening_a_file_with_a_term_origin_column_flags_it_in_the_preview() {
    let root = temp_dir("term-origin-column");
    let global_dir = temp_dir("term-origin-column-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    let path = write_file(
        &root,
        "in.csv",
        "source_term,translation,term_origin\nmoc,Moc Dung,manual\n",
    );
    let preview = glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &path)
        .expect("mo phai thanh cong");

    assert!(preview.term_origin_column_present);
    assert_eq!(preview.ignored_columns, Vec::<String>::new(), "term_origin KHONG phai cot la");

    cleanup(&root);
    cleanup(&global_dir);
}

/// §I/O Matrix "Nhập, tệp có cột lạ" — liệt ở cột bỏ qua, không im lặng vứt; đếm cột nhận
/// ra được KHÔNG tính cột lạ.
#[test]
fn opening_a_file_with_an_unknown_column_reports_it_as_ignored_and_excludes_it_from_the_recognized_count() {
    let root = temp_dir("unknown-column");
    let global_dir = temp_dir("unknown-column-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    let path = write_file(&root, "in.csv", "source_term,translation,usage_count\nmoc,Moc Dung,7\n");
    let preview = glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &path)
        .expect("mo phai thanh cong");

    assert_eq!(preview.ignored_columns, vec!["usage_count".to_owned()]);
    assert!(!preview.term_origin_column_present);
    assert_eq!(preview.recognized_column_count, 2, "usage_count khong duoc tinh vao cot nhan ra duoc");

    cleanup(&root);
    cleanup(&global_dir);
}

/// §I/O Matrix "Nhập, tệp hỏng ở một dòng" — lỗi phân tích đi tới đúng bề mặt
/// `commands::glossary`, KHÔNG kế hoạch nào được giữ lại.
#[test]
fn opening_a_file_broken_at_one_line_surfaces_the_parse_error_and_keeps_no_batch() {
    let root = temp_dir("broken-line");
    let global_dir = temp_dir("broken-line-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    let path = write_file(
        &root,
        "in.csv",
        "source_term,translation,note,category,term_origin,created_at\nmoc,Moc Dung\n",
    );
    let err = glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &path)
        .expect_err("hang thieu o phai bi tu choi");

    assert_eq!(err.message_key(), MessageKey::GlossaryImportCellCountMismatch);
    assert!(pending.lock().unwrap().is_none(), "0 lo nao duoc giu lai khi phan tich trot");

    cleanup(&root);
    cleanup(&global_dir);
}

/// §I/O Matrix "Nhập, tệp 0 hàng dữ liệu" — 0 hàng, KHÔNG lỗi; xem trước vẫn mở được với
/// đếm mọi thứ bằng 0 và danh sách bất đồng rỗng.
#[test]
fn opening_a_file_with_only_a_header_row_previews_as_zero_rows_with_no_error() {
    let root = temp_dir("header-only");
    let global_dir = temp_dir("header-only-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    let path = write_file(&root, "in.csv", "source_term,translation\n");
    let preview = glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &path)
        .expect("tep chi co hang tieu de khong phai loi");

    assert_eq!(preview.row_count, 0);
    assert_eq!(preview.new_count, 0);
    assert_eq!(preview.identical_count, 0);
    assert!(preview.conflicts.is_empty());
    assert!(pending.lock().unwrap().is_some(), "lo van duoc giu (0 hang van la mot lo hop le)");

    cleanup(&root);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// P1 (vòng rà ba lớp 2026-08-25) — Tác phẩm đóng GIỮA LÚC hộp thoại còn mở. `wire::
// glossary_export_tier`/`wire::glossary_open_import_preview` khoá `OpenWorkState` LẦN THỨ
// HAI, MỚI, sau khi `blocking_save_file`/`blocking_pick_file` trả về — không tái dùng giá
// trị đọc trước dialog. Không dựng được cửa sổ Tauri thật ở đây (`cargo test` không có
// `AppHandle`), nên ca dưới đây kiểm ĐÚNG lát cắt mà việc "khoá lại, mới" tạo ra: hàm THUẦN
// nhận `open` tại THỜI ĐIỂM GỌI — gọi nó với `open: Some(&opened)` (Tác phẩm còn mở lúc
// hộp thoại mở) rồi gọi LẠI với `open: None` (Tác phẩm đã đóng lúc hộp thoại trả về, đúng
// giá trị mà lần khoá THỨ HAI của wire sẽ đọc được) — mô phỏng đúng hai lần khoá của wire.
// ═════════════════════════════════════════════════════════════════════════════════

/// §I/O Matrix mở rộng — xuất tầng Work: Tác phẩm ĐANG mở lúc kiểm trước dialog, nhưng đã
/// ĐÓNG lúc hộp thoại trả về (giá trị `open` mà lần khoá THỨ HAI của wire đọc được) ⇒
/// `WorkTierUnavailable`, **0** lượt ghi — không ghi vào một kho đã cũ.
#[test]
fn exporting_the_work_tier_fails_if_the_work_closed_while_the_dialog_was_still_open() {
    let root = temp_dir("export-work-closed-mid-dialog");
    let global_dir = temp_dir("export-work-closed-mid-dialog-global");
    let global = open_global(&global_dir);
    let opened = create_work_from_text(&root, "Muc Cong", "zh", "", "noi dung mau".to_owned())
        .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));

    // Kiem TRUOC dialog (mo phong bang wire): Tac pham dang mo, tier Work hop le va ghi
    // duoc that.
    assert!(glossary_export_tier(Some(&global), Some(&opened), GlossaryTier::Work, &root.join("precheck.csv")).is_ok());

    // Tac pham "dong" trong luc hop thoai con mo -- lan khoa THU HAI cua wire doc duoc None.
    drop(opened);

    let path = root.join("out.csv");
    let err = glossary_export_tier(Some(&global), None, GlossaryTier::Work, &path)
        .expect_err("Tac pham da dong giua luc hop thoai mo phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::GlossaryWorkTierUnavailable);
    assert!(!path.exists(), "0 luot ghi khi Tac pham da dong");

    cleanup(&root);
    cleanup(&global_dir);
}

/// Cùng cửa sổ đua, cho nhịp MỘT của lượt nhập (`glossary_open_import_preview`).
#[test]
fn opening_an_import_preview_for_the_work_tier_fails_if_the_work_closed_while_the_dialog_was_still_open() {
    let root = temp_dir("import-work-closed-mid-dialog");
    let global_dir = temp_dir("import-work-closed-mid-dialog-global");
    let global = open_global(&global_dir);
    let opened = create_work_from_text(&root, "Muc Cong", "zh", "", "noi dung mau".to_owned())
        .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
    let pending = PendingImportState::new(None);

    let path = write_file(&root, "in.csv", "source_term,translation\nmoc,Moc Dung\n");

    // Tac pham "dong" TRUOC khi lan khoa THU HAI cua wire (sau dialog) chay.
    drop(opened);

    let err = glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Work, &path)
        .expect_err("Tac pham da dong giua luc hop thoai mo phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::GlossaryWorkTierUnavailable);
    assert!(pending.lock().unwrap().is_none(), "0 lo nao duoc giu lai khi tang Work da dong");

    cleanup(&root);
    cleanup(&global_dir);
}

/// P5 (vòng rà ba lớp 2026-08-25) — một quyết định trỏ vào hàng `New` (không phải
/// `Conflict`) PHẢI bị từ chối như một thuật ngữ lạ, KHÔNG được lặng lẽ bỏ qua. Lô mang
/// CẢ hai loại hàng trong CÙNG một lượt để phép kiểm phải thật sự phân biệt được hai loại,
/// không chỉ tình cờ đúng vì lô chỉ có một loại.
#[test]
fn confirming_with_a_decision_pointing_at_a_new_row_instead_of_a_conflict_is_rejected() {
    let root = temp_dir("decision-on-new-row");
    let global_dir = temp_dir("decision-on-new-row-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    // "ly" da co san trong kho -- hang cua no se phan loai Conflict. "moc" chua co -- New.
    add_manual_term(&global, None, GlossaryTier::Global, "ly", Some("Ly cu"), "", Category::Other)
        .expect("dung truoc mot muc de tao Conflict");

    let path = write_file(
        &root,
        "in.csv",
        "source_term,translation\nly,Ly moi\nmoc,Moc Dung\n",
    );
    glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &path)
        .expect("mo phai thanh cong");

    // Quyet dinh tro vao "moc" -- mot hang New, khong phai Conflict.
    let mut decisions = BTreeMap::new();
    decisions.insert("moc".to_owned(), ConflictDecision::TakeTheirs);

    let err = glossary_confirm_import(Some(&global), None, &pending, &decisions)
        .expect_err("quyet dinh tro vao hang New phai bi tu choi, khong duoc lang le bo qua");
    assert_eq!(err.message_key(), MessageKey::GlossaryImportDecisionUnknownTerm);
    assert_eq!(err.params().get("value").map(String::as_str), Some("moc"));
    assert!(pending.lock().unwrap().is_some(), "lo phai GIU LAI de thu lai");

    cleanup(&root);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// P6 (vòng rà ba lớp 2026-08-25) — `#[serde(rename)]` của `ConflictDecision` chưa lần nào
// GIẢI MÃ thật một chuỗi JSON qua serde ở bất kỳ ca nào trong kho: mọi ca Rust dựng biến
// thể THẲNG (`ConflictDecision::TakeTheirs`), và vitest mock ở biên adapter (không bao giờ
// sinh JSON thật qua dây). Gõ sai một chuỗi `rename` (`"takeTheirs"` thay vì
// `"take_theirs"`, hay ngược lại) biên dịch SẠCH và không ca nào đỏ cho tới khi người dùng
// thật bấm "Lấy của file" và nó không có tác dụng gì. Cùng khuôn
// `glossary_contract.rs::category_and_glossary_tier_wire_strings_agree_between_as_str_and_serde_rename`
// — khác ở chỗ `ConflictDecision` KHÔNG có `as_str()`/`from_wire()` để so chiều ĐỊA, nên ca
// này hardcode NGUYÊN VĂN hai chuỗi mà `src/config/glossary.ts::GlossaryConflictDecision`
// thật sự gửi, không suy chúng ra từ chính mã Rust (suy vậy sẽ tự đúng dù `rename` sai).
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn conflict_decision_serde_rename_decodes_the_exact_wire_strings_the_frontend_sends() {
    let decoded: ConflictDecision = serde_json::from_str("\"keep_mine\"").unwrap_or_else(|e| {
        panic!(
            "serde khong giai ma duoc \"keep_mine\" -- day la chuoi NGUYEN VAN \
             `GlossaryConflictDecision` phia TypeScript gui qua dispatch/`invoke`, khong phai \
             mot chuoi suy tu Rust. Loi serde: {e}"
        )
    });
    assert_eq!(decoded, ConflictDecision::KeepMine);

    let decoded: ConflictDecision = serde_json::from_str("\"take_theirs\"").unwrap_or_else(|e| {
        panic!("serde khong giai ma duoc \"take_theirs\". Loi serde: {e}")
    });
    assert_eq!(decoded, ConflictDecision::TakeTheirs);

    // Doi chung AM -- mot chuoi KHONG khop (loi go, hoac ten bien the Rust tran chua rename)
    // phai bi TU CHOI tuong minh, khong am tham roi ve mot bien the mac dinh nao.
    assert!(serde_json::from_str::<ConflictDecision>("\"keep-mine\"").is_err());
    assert!(serde_json::from_str::<ConflictDecision>("\"KeepMine\"").is_err());
    assert!(serde_json::from_str::<ConflictDecision>("\"takeTheirs\"").is_err());
}

/// Hình dạng THẬT của tham số `decisions` trên dây (`glossary_confirm_import`) là một object
/// phẳng `{ [source_term]: "keep_mine" | "take_theirs" }` — giải mã CẢ CẤU TRÚC đó qua
/// serde, không chỉ một chuỗi đơn lẻ.
#[test]
fn a_decisions_map_shaped_exactly_like_the_wire_payload_decodes_through_serde() {
    let json = r#"{"慕容":"take_theirs","张":"keep_mine"}"#;
    let decoded: BTreeMap<String, ConflictDecision> =
        serde_json::from_str(json).expect("hinh dang object phang phai giai ma duoc qua serde");

    assert_eq!(decoded.get("慕容"), Some(&ConflictDecision::TakeTheirs));
    assert_eq!(decoded.get("张"), Some(&ConflictDecision::KeepMine));
}

// ═════════════════════════════════════════════════════════════════════════════════
// P7 (vòng rà ba lớp 2026-08-25) — hai nhánh `From<GlossaryError> for IpcError`
// (`ImportReadFailed` · `ExportWriteFailed`) CHƯA lần nào chạy qua chính đường chuyển đổi
// đó ở bất kỳ ca nào trong kho trước bản vá này — `exchange_io.rs` chỉ `matches!` trên
// `GlossaryError` thô (không đọc `message_key()`/`params()`), còn `ipc_contract.rs` chỉ đối
// chiếu bảng tham số TĨNH của macro với `vi.json`, không chạy một dòng nào của `store.rs`.
// Một khoá `params` sai (`"file_path"` thay vì `"path"`) sẽ không ai bắt được cho tới khi
// người dùng thật gặp lỗi I/O.
// ═════════════════════════════════════════════════════════════════════════════════

/// Ghi tệp xuất thất bại (thư mục cha không tồn tại) — đi qua ĐÚNG đường
/// `GlossaryError::ExportWriteFailed` → `IpcError`, khẳng định `message_key()` VÀ
/// `params()["path"]` mang đường dẫn THẬT.
#[test]
fn export_write_failure_surfaces_export_write_failed_with_the_real_path_param() {
    let root = temp_dir("export-write-failed");
    let global_dir = temp_dir("export-write-failed-global");
    let global = open_global(&global_dir);

    // Thu muc CHA khong ton tai -- buoc tao tep tam phai truot.
    let path = root.join("khong-ton-tai").join("out.csv");
    let err = glossary_export_tier(Some(&global), None, GlossaryTier::Global, &path)
        .expect_err("ghi vao mot thu muc cha khong ton tai phai bi tu choi");

    assert_eq!(err.message_key(), MessageKey::GlossaryExportWriteFailed);
    assert_eq!(
        err.params().get("path").map(String::as_str),
        Some(path.display().to_string().as_str())
    );

    cleanup(&root);
    cleanup(&global_dir);
}

/// Đọc tệp nhập thất bại vì lý do KHÁC kích thước/UTF-8 (đường dẫn là một THƯ MỤC, không
/// phải tệp) — đi qua ĐÚNG đường `GlossaryError::ImportReadFailed` → `IpcError` (mượn khoá
/// CHUNG `MessageKey::IoReadFailed`), khẳng định `message_key()` VÀ `params()["path"]`.
#[test]
fn import_read_failure_surfaces_io_read_failed_with_the_real_path_param() {
    let root = temp_dir("import-read-failed");
    let global_dir = temp_dir("import-read-failed-global");
    let global = open_global(&global_dir);
    let pending = PendingImportState::new(None);

    // Duong dan la mot THU MUC -- metadata() thanh cong (duoi tran 16 MiB) nhung
    // std::fs::read() tren mot thu muc phai truot.
    let path = root.join("day-la-thu-muc");
    fs::create_dir_all(&path).unwrap();

    let err = glossary_open_import_preview(Some(&global), None, &pending, GlossaryTier::Global, &path)
        .expect_err("doc mot thu muc thay vi mot tep phai bi tu choi");

    assert_eq!(err.message_key(), MessageKey::IoReadFailed);
    assert_eq!(
        err.params().get("path").map(String::as_str),
        Some(path.display().to_string().as_str())
    );
    assert!(pending.lock().unwrap().is_none(), "0 lo nao duoc giu lai khi doc trot");

    cleanup(&root);
    cleanup(&global_dir);
}
