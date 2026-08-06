//! Hành vi của tầng Tác phẩm trên đĩa — Story 1.15, AC1 tới AC9.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_contract.rs` — một tệp, một mối quan tâm.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — thừa kế nguyên vẹn từ `store_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + `AtomicU64`). ⛔ Không thêm `tempfile`.
//! 2. **Drop `Store`/`OpenWork` TRƯỚC khi xoá thư mục** — Windows từ chối xoá tệp đang mở.
//! 3. ⛔ Không `sleep` dài.
//! 4. Không ca nào treo khi nó trượt.
//!
//! ⚠️ `Store::write` nhận một closure lấy `&Transaction` — kiểu tái xuất từ `core::store` —
//! nên các ca dưới đây thao tác thẳng `work`/`chapter` mà không gõ tên crate SQLite.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::chapter::read_open_chapter;
use auratranslate_lib::commands::project::{create_work_from_file, create_work_from_text};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::library::{META_SCHEMA_VERSION, WorkMeta};
use auratranslate_lib::core::segment::import::{import_file, import_text};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};
use uuid::Uuid;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

/// Một thư mục tạm **của riêng ca này**. Xem luật 1 ở doc-comment đầu tệp.
fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-project-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

/// Tệp tạm mang nội dung `bytes`, trả về đường dẫn đầy đủ.
fn write_file(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, bytes).unwrap_or_else(|e| panic!("ghi {}: {e}", path.display()));
    path
}

#[test]
fn creating_a_work_lays_down_exactly_three_things_on_disk() {
    let root = temp_dir("three-things");

    let opened = create_work_from_text(&root, "Ba Thu", "zh", "tieu thuyet", "noi dung mau".to_owned())
        .expect("tao tac pham that bai");
    let dir = opened.dir.clone();
    drop(opened); // đóng Store trước khi TRUNCATE có cơ hội chạy và trước khi liệt kê thư mục.

    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("doc {}: {e}", dir.display()))
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        // ⚠️ `project.db-wal` / `project.db-shm` là sidecar CỦA CHÍNH `project.db` do chế độ
        // WAL sinh ra — chúng KHÔNG phải một trong ba "thành phần" AC2 đếm, nhưng sự có mặt
        // của chúng là hành vi SQLite bình thường, không phải một tệp lạc.
        .filter(|n| n != "project.db-wal" && n != "project.db-shm")
        .collect();
    names.sort();

    assert_eq!(
        names,
        vec!["assets".to_owned(), "meta.json".to_owned(), "project.db".to_owned()],
        "thu muc .atproj phai chua dung ba thanh phan: meta.json, project.db, assets/"
    );
    assert!(dir.join("assets").is_dir(), "assets/ phai la mot thu muc");

    cleanup(&root);
}

#[test]
fn meta_json_is_readable_without_ever_touching_the_database() {
    let root = temp_dir("meta-no-db");

    let opened = create_work_from_text(&root, "Doc Rieng", "en", "", "text".to_owned())
        .expect("tao tac pham that bai");
    let dir = opened.dir.clone();
    drop(opened); // ⚠️ Đóng Store trước khi xoá tệp — Windows từ chối xoá tệp đang mở.

    // Xoa han project.db khoi dia. Neu WorkMeta::read con can mo SQLite, buoc doc duoi day
    // se that bai vi tep khong con ton tai.
    fs::remove_file(dir.join("project.db")).expect("xoa project.db that bai");
    let _ = fs::remove_file(dir.join("project.db-wal"));
    let _ = fs::remove_file(dir.join("project.db-shm"));

    let meta = WorkMeta::read(&dir).expect("doc meta.json phai thanh cong ke ca khi project.db da bi xoa");
    assert_eq!(meta.name, "Doc Rieng");

    cleanup(&root);
}

#[test]
fn the_work_id_is_a_v4_uuid_with_the_right_version_and_variant_bits() {
    let root = temp_dir("uuid-v4");

    let opened = create_work_from_text(&root, "Uuid", "zh", "", "text".to_owned())
        .expect("tao tac pham that bai");

    let parsed = Uuid::parse_str(&opened.meta.work_id).expect("work_id phai la mot UUID hop le");
    assert_eq!(parsed.get_version_num(), 4, "work_id phai la UUID phien ban 4");
    assert_eq!(
        parsed.get_variant(),
        uuid::Variant::RFC4122,
        "work_id phai mang bit variant RFC 4122"
    );

    drop(opened);
    cleanup(&root);
}

#[test]
fn a_retired_chapter_id_is_never_handed_out_again() {
    let root = temp_dir("retired-id");

    let opened = create_work_from_text(&root, "Chuong", "zh", "", "chuong mot".to_owned())
        .expect("tao tac pham that bai");
    let store = opened.store;

    let first_id: i64 = store
        .read(|conn| conn.query_row("SELECT id FROM chapter WHERE ord = 1", [], |row| row.get(0)))
        .expect("doc id chuong dau tien that bai");
    assert_eq!(first_id, 1);

    // Xoa hang duy nhat roi chen hang moi — AUTOINCREMENT khong duoc phep phat lai id=1.
    store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("DELETE FROM chapter WHERE id = 1", [])?;
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (2, NULL, 'chuong hai', 'not_started', \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [],
            )?;
            Ok(())
        })
        .expect("job ghi that bai");

    let second_id: i64 = store
        .read(|conn| conn.query_row("SELECT id FROM chapter WHERE ord = 2", [], |row| row.get(0)))
        .expect("doc id chuong thu hai that bai");

    assert_ne!(second_id, 1, "id da ve huu (1) khong duoc phat lai");
    assert_eq!(second_id, 2, "AUTOINCREMENT phai cap id tang dan nghiem ngat, khong tai dung rowid");

    drop(store);
    cleanup(&root);
}

#[test]
fn a_copied_project_folder_opens_at_a_different_path() {
    let root = temp_dir("copy-open-a");
    let copy_root = temp_dir("copy-open-b");

    let opened = create_work_from_text(&root, "Sao Chep", "zh", "", "noi dung".to_owned())
        .expect("tao tac pham that bai");
    let original_dir = opened.dir.clone();
    let original_work_id = opened.meta.work_id.clone();
    drop(opened);

    let new_dir = copy_root.join(original_dir.file_name().unwrap());
    copy_dir_recursive(&original_dir, &new_dir);

    // Mo lai project.db o duong dan MOI, hoan toan doc lap voi duong dan cu.
    let reopened = Store::open(StoreSpec::project(new_dir.join("project.db")))
        .expect("mo lai project.db o thu muc da copy phai thanh cong");
    let reopened_work_id: String = reopened
        .read(|conn| conn.query_row("SELECT work_id FROM work WHERE id = 1", [], |row| row.get(0)))
        .expect("doc work_id sau khi mo lai that bai");
    assert_eq!(reopened_work_id, original_work_id);
    drop(reopened);

    let reread_meta = WorkMeta::read(&new_dir).expect("doc meta.json o thu muc da copy phai thanh cong");
    assert_eq!(reread_meta.work_id, original_work_id);

    cleanup(&root);
    cleanup(&copy_root);
}

fn copy_dir_recursive(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|e| panic!("tao {}: {e}", to.display()));
    for entry in fs::read_dir(from).unwrap_or_else(|e| panic!("doc {}: {e}", from.display())) {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_recursive(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap_or_else(|e| panic!("copy {}: {e}", entry.path().display()));
        }
    }
}

#[test]
fn meta_json_can_be_rebuilt_from_the_database_alone() {
    let root = temp_dir("rebuild-meta");

    let opened = create_work_from_text(&root, "Dung Lai", "en", "tho", "van ban".to_owned())
        .expect("tao tac pham that bai");
    let dir = opened.dir.clone();
    let store = opened.store;
    let original_meta = WorkMeta::read(&dir).expect("doc meta.json ban dau that bai");

    fs::remove_file(dir.join("meta.json")).expect("xoa meta.json that bai");

    let rebuilt = WorkMeta::rebuild_from_store(&store).expect("dung lai meta.json tu project.db that bai");
    assert_eq!(rebuilt.work_id, original_meta.work_id);
    assert_eq!(rebuilt.name, original_meta.name);
    assert_eq!(rebuilt.source_lang, original_meta.source_lang);
    assert_eq!(rebuilt.genre, original_meta.genre);
    assert_eq!(rebuilt.chapter_count, 1);

    drop(store);
    cleanup(&root);
}

#[test]
fn a_newer_meta_schema_is_refused_without_touching_a_single_byte() {
    let root = temp_dir("meta-schema-new");

    let opened = create_work_from_text(&root, "Phien Ban Moi", "zh", "", "text".to_owned())
        .expect("tao tac pham that bai");
    let dir = opened.dir.clone();
    drop(opened);

    let mut future_meta = WorkMeta::read(&dir).expect("doc meta.json that bai");
    future_meta.meta_schema_version = META_SCHEMA_VERSION + 1;
    future_meta
        .write_atomic(&dir)
        .expect("ghi meta.json phien ban tuong lai that bai");

    let result = WorkMeta::read(&dir);
    assert!(
        result.is_err(),
        "doc mot meta.json phien ban moi hon phai bi tu choi"
    );

    cleanup(&root);
}

#[test]
fn a_docx_is_refused_before_a_single_byte_is_written() {
    let root = temp_dir("docx-refused");
    let source_dir = temp_dir("docx-refused-src");

    // ⚠️ Tep .docx nay KHONG TON TAI tren dia. Neu duong tu choi lo mo tep truoc khi kiem
    // dinh dang, ca nay se that bai voi mot loi I/O khac thay vi UnsupportedFormat — do
    // chinh la bang chung "tu choi truoc khi mo tep, khong doc mot byte nao".
    let fake_docx = source_dir.join("khong-ton-tai.docx");

    let result = create_work_from_file(&root, "Tu Choi Docx", "zh", "", &fake_docx);
    assert!(result.is_err(), "mot tep .docx phai bi tu choi");

    let entries: Vec<_> = fs::read_dir(&root).unwrap().collect();
    assert!(
        entries.is_empty(),
        "khong thu muc .atproj nao duoc tao khi .docx bi tu choi"
    );

    cleanup(&root);
    cleanup(&source_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 CA DƯỚI ĐÂY THAY THẾ MỘT CA CŨ ĐÃ HỢP THỨC HOÁ ĐÚNG LỖI NÓ ĐÁNG LẼ PHẢI CANH
// ═════════════════════════════════════════════════════════════════════════════════
//
// Bản trước của `a_failed_import_leaves_no_half_built_folder_behind` **tự tay tạo**
// `Nua Voi.atproj/` trước khi gọi `create_work`, rồi assert `!expected_dir.exists()` —
// tức là nó **ĐÒI HỎI** `create_work` xoá một thư mục ⛔ không phải do nó tạo. Ca đó xanh,
// và nó khoá lại thành hợp đồng đúng đường mất dữ liệu mà lượt code review 2026-08-06 tìm
// ra: tạo Tác phẩm trùng tên ⇒ `INSERT ... VALUES (1, …)` đụng `CHECK (id = 1)` ⇒ nhánh
// dọn dẹp `remove_dir_all` cả `.atproj` của người dùng.
//
// ⚠️ **Vì sao ⛔ không còn một ca "trượt giữa chừng" bơm lỗi từ bên ngoài:** sau khi
// `create_work_folder` chuyển sang **tạo độc quyền** (`fs::create_dir`, ⛔ không `_all`),
// ⛔ không còn cách nào từ ngoài ép một lượt gọi đi vào một thư mục đã có — mọi thư mục
// đã tồn tại đều bị bỏ qua và lượt gọi nhận một tên mới. Đó chính là điều làm đường xoá
// nhầm ⛔ **không tới được nữa**, và nó là một bất biến **theo cấu trúc**, ⛔ không phải
// theo một phép kiểm lúc chạy. Vế "⛔ không để lại thư mục nửa vời" của AC8 vì thế được
// canh bằng ba ca **từ chối trước khi ghi** ở trên/dưới (`.docx`, ⛔ không UTF-8, quá
// nặng) — cả ba assert thư mục gốc **rỗng tuyệt đối**.
#[test]
fn creating_a_work_over_an_existing_folder_never_touches_it() {
    let root = temp_dir("name-collision");

    // Tác phẩm thứ nhất — coi như đây là nhiều tháng công việc của người dùng.
    let first = create_work_from_text(&root, "Truyen Kieu", "zh", "tho", "CHUONG MOT".to_owned())
        .expect("tao tac pham dau tien that bai");
    let first_dir = first.dir.clone();
    let first_id = first.meta.work_id.clone();
    drop(first); // đóng Store trước khi mở kho thứ hai.

    assert_eq!(first_dir, root.join("Truyen Kieu.atproj"));

    // Một dấu vết KHÔNG do mã sản phẩm tạo: nếu thư mục bị xoá rồi dựng lại, nó biến mất.
    let sentinel = first_dir.join("assets").join("bia.txt");
    fs::write(&sentinel, b"anh bia cua nguoi dung").expect("ghi sentinel that bai");

    // Người dùng gõ **đúng cái tên ấy** lần nữa.
    let second = create_work_from_text(&root, "Truyen Kieu", "zh", "tho", "CHUONG HAI".to_owned())
        .expect("tao tac pham thu hai phai THANH CONG, khong phai that bai");
    let second_dir = second.dir.clone();
    let second_id = second.meta.work_id.clone();
    drop(second);

    // ① Tác phẩm thứ nhất còn NGUYÊN — đây là mệnh đề sống chết của ca này.
    assert!(first_dir.is_dir(), "thu muc cua Tac pham dau tien phai con nguyen");
    assert!(
        sentinel.is_file(),
        "tep trong assets/ cua Tac pham dau tien phai con nguyen — thu muc KHONG duoc dung lai"
    );
    assert!(first_dir.join("project.db").is_file(), "project.db cu phai con nguyen");
    let first_meta = WorkMeta::read(&first_dir).expect("meta.json cu phai con doc duoc");
    assert_eq!(first_meta.work_id, first_id, "Tac pham dau tien phai giu nguyen work_id");

    // ② Tác phẩm thứ hai đi vào một thư mục ĐƯỢC ĐÁNH SỐ (Ice chốt 2026-08-06).
    assert_eq!(
        second_dir,
        root.join("Truyen Kieu (2).atproj"),
        "tac pham trung ten phai nhan hau to danh so"
    );
    assert_ne!(first_id, second_id, "hai Tac pham phai la hai work_id khac nhau");

    // ③ ⚠️ Hệ quả đã biết, khoá lại bằng một assert để ⛔ không ai đọc nhầm: `meta.name`
    //    giữ **nguyên tên người dùng gõ** ở CẢ HAI — hai Tác phẩm hiển thị giống hệt nhau,
    //    chỉ tên thư mục khác. Đó là cái giá của tự-đánh-số so với từ-chối.
    let second_meta = WorkMeta::read(&second_dir).expect("doc meta.json moi that bai");
    assert_eq!(first_meta.name, second_meta.name);

    cleanup(&root);
}

/// Ba lần liên tiếp cùng một tên ⇒ ba thư mục, ⛔ không hai.
#[test]
fn repeated_names_keep_climbing_the_suffix_instead_of_colliding() {
    let root = temp_dir("collision-climb");

    let mut dirs = Vec::new();
    for _ in 0..3 {
        let opened = create_work_from_text(&root, "Untitled", "en", "", "x".to_owned())
            .expect("tao tac pham that bai");
        dirs.push(opened.dir.clone());
        drop(opened);
    }

    assert_eq!(dirs[0], root.join("Untitled.atproj"));
    assert_eq!(dirs[1], root.join("Untitled (2).atproj"));
    assert_eq!(dirs[2], root.join("Untitled (3).atproj"));
    for dir in &dirs {
        assert!(dir.join("project.db").is_file(), "{} phai con project.db", dir.display());
    }

    cleanup(&root);
}

#[test]
fn text_that_is_not_utf8_is_refused_the_same_way_a_docx_is() {
    let root = temp_dir("not-utf8");
    let source_dir = temp_dir("not-utf8-src");

    // 0x80 mot minh khong phai mot chuoi UTF-8 hop le o bat ky vi tri nao.
    let bad_path = write_file(&source_dir, "bad.txt", &[0x66, 0x69, 0x6c, 0x65, 0x80, 0x81]);

    let result = create_work_from_file(&root, "Khong Phai Utf8", "zh", "", &bad_path);
    assert!(result.is_err(), "noi dung khong phai UTF-8 phai bi tu choi");

    let entries: Vec<_> = fs::read_dir(&root).unwrap().collect();
    assert!(
        entries.is_empty(),
        "khong thu muc .atproj nao duoc tao khi noi dung khong phai UTF-8"
    );

    cleanup(&root);
    cleanup(&source_dir);
}

#[test]
fn pasted_text_and_a_read_file_travel_the_same_import_path() {
    let source_dir = temp_dir("same-path");
    let content = "mot doan van ban giong het nhau\nqua ca hai duong";

    let from_paste = import_text(content.to_owned());

    let path = write_file(&source_dir, "sample.txt", content.as_bytes());
    let from_file = import_file(&path).expect("doc tep .txt hop le that bai");

    assert_eq!(
        from_paste.source_text, from_file.source_text,
        "dan van ban va doc tep phai di qua CUNG mot ham thuan va cho ra cung mot ket qua"
    );

    cleanup(&source_dir);
}

/// Đối chứng dương AC8 — `.docx` bị từ chối bằng đúng khoá `MessageKey`, ⛔ không bằng
/// một lỗi kho chung chung.
#[test]
fn a_docx_rejection_carries_the_dedicated_message_key() {
    let root = temp_dir("docx-key");
    let source_dir = temp_dir("docx-key-src");
    let fake_docx = source_dir.join("tai-lieu.docx");

    let err = create_work_from_file(&root, "Khoa Loi", "zh", "", &fake_docx)
        .expect_err(".docx phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::ImportUnsupportedFormat);
    assert!(!err.retryable(), "tu choi dinh dang khong the sua bang cach bam lai");

    cleanup(&root);
    cleanup(&source_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bổ sung ở lượt code review 2026-08-06
// ═════════════════════════════════════════════════════════════════════════════════

/// AC5 vế hai — *"⛔ **không** đường dẫn tuyệt đối nào của máy cũ nằm trong `meta.json`
/// hay `project.db`"*.
///
/// 🔴 AC5 viết cùng khuôn AC3: *"test chứng minh, ⛔ không phải một lời khẳng định"*. Vế
/// "mở lại được ở đường dẫn khác" đã có ca riêng; vế này trước lượt review ⛔ **không có
/// ca nào** — mệnh đề đúng về cấu trúc (⛔ không trường/cột nào chứa đường dẫn) nhưng ⛔
/// không gì canh việc một story sau thêm một cột như thế.
#[test]
fn no_absolute_path_of_this_machine_is_written_inside_the_project() {
    let root = temp_dir("no-abs-path");

    let opened = create_work_from_text(&root, "Khong Duong Dan", "zh", "", "noi dung".to_owned())
        .expect("tao tac pham that bai");
    let dir = opened.dir.clone();
    drop(opened);

    // Mảnh đường dẫn đặc trưng của máy này — nếu nó lọt vào tệp, bản copy sang máy khác
    // mang theo một mệnh đề sai về nơi nó từng nằm.
    let needle = dir.to_string_lossy().into_owned();
    let needle_bytes = needle.as_bytes();

    let meta_raw = fs::read(dir.join("meta.json")).expect("doc meta.json that bai");
    assert!(
        !contains_bytes(&meta_raw, needle_bytes),
        "meta.json chua duong dan tuyet doi cua may nay"
    );

    let db_raw = fs::read(dir.join("project.db")).expect("doc project.db that bai");
    assert!(
        !contains_bytes(&db_raw, needle_bytes),
        "project.db chua duong dan tuyet doi cua may nay"
    );

    cleanup(&root);
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Quyết định của Ice ở lượt code review 2026-08-06 — BOM là tạo tác **giải mã**, cắt ở
/// story này; CRLF là **chuẩn hoá**, ⛔ không đụng (Epic 6).
///
/// 🔴 `EF BB BF` là UTF-8 **hợp lệ**, nên nó đi lọt `String::from_utf8` mà ⛔ không cổng
/// nào kêu — và AD-4 đóng băng ranh giới segment tính lúc nhập, nên một `U+FEFF` nằm lại
/// sẽ thành ký tự đầu của segment #1 **vĩnh viễn**.
#[test]
fn a_utf8_bom_is_stripped_but_line_endings_are_left_alone() {
    let root = temp_dir("bom");
    let source_dir = temp_dir("bom-src");

    let path = write_file(&source_dir, "notepad.txt", b"\xEF\xBB\xBFCHUONG MOT\r\nCau hai");
    let imported = import_file(&path).expect("tep UTF-8 co BOM phai nhap duoc");

    assert!(
        !imported.source_text.starts_with('\u{feff}'),
        "BOM phai bi cat khoi dau van ban"
    );
    assert!(
        imported.source_text.starts_with("CHUONG MOT"),
        "van ban phai bat dau bang ky tu that dau tien"
    );
    assert!(
        imported.source_text.contains("\r\n"),
        "CRLF phai duoc GIU NGUYEN — chuan hoa xuong dong la FR124/125, Epic 6"
    );

    // ⛔ Chỉ cắt ở ĐẦU: một U+FEFF giữa văn bản là zero-width no-break space, nội dung thật.
    let inner = write_file(&source_dir, "giua.txt", "AB\u{feff}CD".as_bytes());
    let imported_inner = import_file(&inner).expect("nhap that bai");
    assert_eq!(imported_inner.source_text, "AB\u{feff}CD");

    cleanup(&root);
    cleanup(&source_dir);
}

/// Trần 100 MB (Ice chốt 2026-08-06) — và trần đó phải chặn **trước khi** đọc.
#[test]
fn a_file_past_the_size_ceiling_is_refused_before_a_single_byte_is_written() {
    let root = temp_dir("too-large");
    let source_dir = temp_dir("too-large-src");

    // ⚠️ ⛔ Không ghi 100 MB thật ra đĩa trong một test — dựng một tệp THƯA (sparse):
    // `set_len` khai kích thước mà ⛔ không cấp phát khối nào.
    let path = source_dir.join("khong lo.txt");
    let file = fs::File::create(&path).expect("tao tep that bai");
    file.set_len(100 * 1024 * 1024 + 1).expect("set_len that bai");
    drop(file);

    let err = create_work_from_file(&root, "Khong Lo", "zh", "", &path)
        .expect_err("tep vuot tran phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::ImportTooLarge);
    assert!(!err.retryable(), "bam lai cung tep do cho cung ket qua do");

    let entries: Vec<_> = fs::read_dir(&root).expect("doc root that bai").collect();
    assert!(entries.is_empty(), "khong thu muc .atproj nao duoc tao khi tep vuot tran");

    cleanup(&root);
    cleanup(&source_dir);
}

/// Tệp ⛔ không có phần mở rộng ⇒ một hạng lỗi RIÊNG, ⛔ không phải `unsupported_format`
/// với `format` rỗng (nó cho ra câu vỡ *"Định dạng . chưa được nhận"*).
#[test]
fn a_file_with_no_extension_gets_its_own_message_instead_of_a_broken_sentence() {
    let root = temp_dir("no-ext");
    let source_dir = temp_dir("no-ext-src");

    let path = write_file(&source_dir, "README", b"noi dung");
    let err = create_work_from_file(&root, "Khong Duoi", "zh", "", &path)
        .expect_err("tep khong co phan mo rong phai bi tu choi");

    assert_eq!(err.message_key(), MessageKey::ImportMissingExtension);
    assert!(!err.retryable());

    let entries: Vec<_> = fs::read_dir(&root).expect("doc root that bai").collect();
    assert!(entries.is_empty(), "khong thu muc .atproj nao duoc tao");

    cleanup(&root);
    cleanup(&source_dir);
}

/// `sanitize_name` — ba ca mà bản trước lượt review để lọt (NFR14).
#[test]
fn a_folder_name_survives_both_platforms_rules() {
    use auratranslate_lib::core::library::sanitize_name;

    // ① Tên thiết bị Windows **có đuôi** — bản trước chỉ so nguyên chuỗi nên `CON.txt` lọt.
    assert!(sanitize_name("CON.txt").ends_with('_'), "CON.txt phai duoc them hau to");
    assert!(sanitize_name("nul.md").ends_with('_'), "so sanh khong phan biet hoa thuong");
    assert!(sanitize_name("COM1").ends_with('_'));
    // ⛔ Không phải tên thiết bị thì ⛔ không đụng.
    assert_eq!(sanitize_name("CONtent"), "CONtent");

    // ② Trần theo BYTE, cắt ở biên ký tự — ⛔ không panic giữa một ký tự nhiều byte.
    let long = "Đ".repeat(300); // 600 byte
    let cut = sanitize_name(&long);
    assert!(cut.len() <= 180, "ten phai bi cat theo tran byte, dai that: {}", cut.len());
    assert!(cut.chars().all(|c| c == 'Đ'), "cat phai roi vao bien ky tu");

    // ③ Ký tự cấm và tên rỗng.
    assert_eq!(sanitize_name("Tap 1: Khoi dau"), "Tap 1_ Khoi dau");
    assert_eq!(sanitize_name("   "), "Untitled");
    assert_eq!(sanitize_name("ten."), "ten");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.16, AC8 — đường IPC đọc Chương đang mở
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC8** — chưa Tác phẩm nào mở ⇒ một lỗi CÓ TÊN RIÊNG, ⛔ không phải một lỗi kho
/// (`store.*`): `OpenWorkState` rỗng là một trạng thái sản phẩm bình thường, ⛔ không một
/// tệp nào hỏng.
#[test]
fn reading_the_open_chapter_without_a_work_open_is_a_named_error() {
    let err = read_open_chapter(None).expect_err("chua Tac pham nao mo phai la mot loi");

    assert_eq!(err.code(), "project.no_work_open");
    assert_eq!(err.message_key(), MessageKey::ProjectNoWorkOpen);
    assert!(!err.retryable(), "khong Tac pham nao mo khong phai mot loi tam thoi");
    assert!(
        err.params().is_empty(),
        "khoa nay khong doi tham so nao: {:?}",
        err.params()
    );
}

/// 🔴 **AC8** — Chương đang mở trả đúng `source_text` + `source_lang` của chính Tác phẩm
/// vừa tạo (Story 1.15 luôn ghi đúng MỘT Chương, `ord = 1`).
#[test]
fn reading_the_open_chapter_reflects_the_single_chapter_just_created() {
    let root = temp_dir("read-chapter");

    let opened = create_work_from_text(&root, "Doc Chuong", "zh", "tieu thuyet", "noi dung mau 你好".to_owned())
        .expect("tao tac pham that bai");

    let chapter = read_open_chapter(Some(&opened)).expect("doc Chuong dang mo that bai");

    assert_eq!(chapter.source_text, "noi dung mau 你好");
    assert_eq!(chapter.source_lang, "zh");
    assert!(chapter.chapter_id > 0);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// `source_lang` đọc từ Tác phẩm, ⛔ không đoán từ nội dung — một Chương tiếng Anh với
/// nội dung có chữ Hán bên trong vẫn phải mang `source_lang = "en"`.
#[test]
fn the_source_lang_is_read_from_the_work_never_guessed_from_the_text() {
    let root = temp_dir("read-chapter-lang");

    let opened = create_work_from_text(&root, "Ngon Ngu That", "en", "", "a quote: 你好".to_owned())
        .expect("tao tac pham that bai");

    let chapter = read_open_chapter(Some(&opened)).expect("doc Chuong dang mo that bai");
    assert_eq!(
        chapter.source_lang, "en",
        "source_lang phai la truong bat bien cua Tac pham, khong doan tu noi dung"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}
