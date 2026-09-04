//! Hành vi của tầng Tác phẩm trên đĩa — Story 1.15, AC1 tới AC9.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_contract.rs` — một tệp, một mối quan tâm.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — thừa kế nguyên vẹn từ `store_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + `AtomicU64`). Không thêm `tempfile`.
//! 2. **Drop `Store`/`OpenWork` TRƯỚC khi xoá thư mục** — Windows từ chối xoá tệp đang mở.
//! 3. Không `sleep` dài.
//! 4. Không ca nào treo khi nó trượt.
//!
//! ⚠️ `Store::write` nhận một closure lấy `&Transaction` — kiểu tái xuất từ `core::store` —
//! nên các ca dưới đây thao tác thẳng `work`/`chapter` mà không gõ tên crate SQLite.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::chapter::{
    ChapterDirection, ChapterSwitchOutcome, merge_chapter_into_previous, move_chapter,
    open_adjacent_chapter, read_open_chapter, rename_chapter, split_chapter_at_segment,
};
use auratranslate_lib::commands::project::{
    OpenWork, create_work, create_work_from_file, create_work_from_text,
};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::library::{META_SCHEMA_VERSION, WorkMeta};
use auratranslate_lib::core::scope::{ScopeResolver, Tier, WorkScope};
use auratranslate_lib::core::segment::import::{import_file, import_text};
use auratranslate_lib::core::segment::pipeline::{
    ChapterInput, PipelineInput, PipelineShape, run_import,
};
use auratranslate_lib::core::store::{SqlResult, Store, StoreSpec, Transaction};
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
    // 🔵 THÊM (2026-08-28, Story 5.5) — Chương duy nhất vừa tạo ở `not_started`, nên tiến độ
    // PHẢI là `Some(0)`, KHÔNG `None`: `rebuild_from_store` LUÔN đặt giá trị này.
    assert_eq!(
        rebuilt.chapter_done_count,
        Some(0),
        "Chuong vua tao o not_started -- tien do phai la Some(0), khong phai None (chua biet)"
    );

    drop(store);
    cleanup(&root);
}

/// **THÊM (2026-08-28, Story 5.5)** — đếm THẬT trên nhiều Chương với trạng thái trộn lẫn,
/// bao gồm một hàng HỎNG (`status` ngoài danh mục bốn giá trị). Chương thứ hai/ba được chèn
/// bằng SQL thẳng (không qua một đường sản phẩm nào — hôm nay chưa đường nào tạo Chương thứ
/// hai, xem `epic-5-context.md`), CHỈ để dựng fixture cho phép đếm.
#[test]
fn rebuild_counts_done_chapters_from_the_resolved_set_ignoring_a_corrupt_row() {
    let root = temp_dir("rebuild-progress-count");

    let opened = create_work_from_text(&root, "Dem Tien Do", "en", "", "van ban".to_owned())
        .expect("tao tac pham that bai");
    let store = opened.store;

    // Chuong 1 (co san) -> done; them Chuong 2 (done), Chuong 3 (not_started), Chuong 4 (HONG).
    store
        .write(|tx: &Transaction<'_>| {
            tx.execute("UPDATE chapter SET status = 'done' WHERE ord = 1", [])?;
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (2, NULL, 'c2', 'done', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [],
            )?;
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (3, NULL, 'c3', 'not_started', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [],
            )?;
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (4, NULL, 'c4', 'this_is_not_a_real_status', \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [],
            )?;
            Ok(())
        })
        .expect("dung fixture bon Chuong that bai");

    let rebuilt = WorkMeta::rebuild_from_store(&store).expect("dung lai meta.json that bai");
    assert_eq!(rebuilt.chapter_count, 4, "sam ca hang HONG cung duoc DEM vao tong so Chuong");
    assert_eq!(
        rebuilt.chapter_done_count,
        Some(2),
        "chi hai Chuong 'done' THAT (1 va 2) duoc dem -- hang hong (4) khong duoc tinh la da xong"
    );

    drop(store);
    cleanup(&root);
}

/// **THÊM (2026-08-28, Story 5.5)** — §I/O Matrix "Tác phẩm 0 Chương": `chapter_count = 0`
/// ⇒ `chapter_done_count = Some(0)`, KHÔNG chia cho 0 (hàm đếm chạy trên một tập RỖNG, không
/// một nhánh `if chapter_count == 0` đặc biệt nào ở tầng Rust — phép chia chỉ tồn tại ở tầng
/// hiển thị, `LibraryMode.vue::progressPercent`). Không đường sản phẩm nào tạo một Tác phẩm 0
/// Chương hôm nay (`create_work` luôn chèn đúng một Chương) — xoá Chương bằng SQL thẳng để
/// dựng fixture, cùng khuôn `rebuild_counts_done_chapters_from_the_resolved_set_ignoring_a_corrupt_row`.
#[test]
fn rebuild_on_a_work_with_zero_chapters_reports_some_zero_not_none() {
    let root = temp_dir("rebuild-zero-chapters");

    let opened = create_work_from_text(&root, "Khong Chuong", "en", "", "van ban".to_owned())
        .expect("tao tac pham that bai");
    let store = opened.store;

    store
        .write(|tx: &Transaction<'_>| {
            tx.execute("DELETE FROM chapter", [])?;
            Ok(())
        })
        .expect("xoa het Chuong that bai");

    let rebuilt = WorkMeta::rebuild_from_store(&store).expect("dung lai meta.json that bai");
    assert_eq!(rebuilt.chapter_count, 0);
    assert_eq!(
        rebuilt.chapter_done_count,
        Some(0),
        "0 Chuong -- tien do phai la Some(0), khong phai None va khong panic vi chia cho 0"
    );

    drop(store);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.6 — `updated_at` DẪN XUẤT (AC8), MAX(work.created_at, chapter.updated_at,
// segment.updated_at). Trước story này, cột này được CHÉP từ `work.updated_at` -- một cột
// với đúng MỘT lượt `INSERT` và 0 lượt `UPDATE` toàn kho, nên nó đứng yên vĩnh viễn ở mốc
// TẠO. Đối chứng bắt buộc của §Verification (gỡ phép tính `MAX`, xác nhận ca đỏ) chạy TAY,
// không phải một ca ở đây -- ghi lại trong báo cáo cuối story.
// ═════════════════════════════════════════════════════════════════════════════════

/// Một Tác phẩm vừa tạo, chưa sửa gì: `updated_at` phải ≥ `created_at` (sàn của MAX).
///
/// ⚠️ **KHÔNG khẳng định BẰNG NHAU.** `create_work` chèn `work` rồi `chapter` trong CÙNG một
/// giao dịch, nhưng mỗi câu `INSERT` tự gọi `strftime('%Y-%m-%dT%H:%M:%fZ','now')` RIÊNG —
/// hai lời gọi đồng hồ tách rời, không một ảnh chụp thời gian dùng chung cho cả giao dịch.
/// `chapter` được chèn SAU `work`, nên `chapter.updated_at` thường LỚN HƠN `work.created_at`
/// vài mili-giây (đo được: một ca trước đó đo lệch 2ms) — đúng, không phải lỗi giờ hệ thống.
/// Bất biến ĐÚNG là "không bao giờ lùi về TRƯỚC mốc tạo", không phải "bằng hệt mốc tạo".
#[test]
fn rebuild_derives_updated_at_no_earlier_than_created_at_for_a_freshly_created_work() {
    let root = temp_dir("rebuild-updated-at-fresh");

    let opened = create_work_from_text(&root, "Vua Tao", "en", "", "van ban".to_owned())
        .expect("tao tac pham that bai");
    let store = opened.store;

    let rebuilt = WorkMeta::rebuild_from_store(&store).expect("dung lai meta.json that bai");
    assert!(
        rebuilt.updated_at >= rebuilt.created_at,
        "updated_at ({}) khong duoc LUI ve TRUOC created_at ({})",
        rebuilt.updated_at,
        rebuilt.created_at
    );

    drop(store);
    cleanup(&root);
}

/// AC8 -- một Chương vừa đổi trạng thái (mô phỏng bằng chính câu SQL mà
/// `commands::lifecycle::set_chapter_status` chạy) đẩy `chapter.updated_at` lên một mốc SAU
/// `created_at`; `rebuild_from_store` phải đọc ra mốc MỚI đó, không còn đứng yên ở mốc tạo.
#[test]
fn rebuild_derives_updated_at_from_chapter_updated_at_when_it_is_the_latest() {
    let root = temp_dir("rebuild-updated-at-chapter");

    let opened = create_work_from_text(&root, "Cap Nhat Chuong", "en", "", "van ban".to_owned())
        .expect("tao tac pham that bai");
    let store = opened.store;

    let later = "2099-01-01T00:00:00.000Z";
    store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE chapter SET updated_at = ?1 WHERE ord = 1", [later]))
        .expect("day chapter.updated_at ve sau that bai");

    let rebuilt = WorkMeta::rebuild_from_store(&store).expect("dung lai meta.json that bai");
    assert_eq!(
        rebuilt.updated_at, later,
        "work.updated_at phai TIEN theo chapter.updated_at, khong dung yen o moc tao \
         (day la doi chung cho AC8)"
    );

    drop(store);
    cleanup(&root);
}

/// Cùng lý lẽ AC8, cho `segment.updated_at` — nguồn CÒN LẠI trong MAX ba nguồn (mốc sửa văn
/// bản thuần, không đổi trạng thái Chương nào — xem `commands/segment.rs:1186`).
#[test]
fn rebuild_derives_updated_at_from_segment_updated_at_when_it_is_the_latest() {
    let root = temp_dir("rebuild-updated-at-segment");

    let opened = create_work_from_text(&root, "Cap Nhat Doan", "en", "", "cau mot. cau hai.".to_owned())
        .expect("tao tac pham that bai");
    let store = opened.store;

    let later = "2099-06-01T00:00:00.000Z";
    store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET updated_at = ?1 WHERE id = (SELECT MIN(id) FROM segment)",
                [later],
            )
        })
        .expect("day segment.updated_at ve sau that bai");

    let rebuilt = WorkMeta::rebuild_from_store(&store).expect("dung lai meta.json that bai");
    assert_eq!(
        rebuilt.updated_at, later,
        "mot segment sua SAU created_at/chapter.updated_at phai keo updated_at cua Tac pham theo"
    );

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
// tức là nó **ĐÒI HỎI** `create_work` xoá một thư mục không phải do nó tạo. Ca đó xanh,
// và nó khoá lại thành hợp đồng đúng đường mất dữ liệu mà lượt code review 2026-08-06 tìm
// ra: tạo Tác phẩm trùng tên ⇒ `INSERT ... VALUES (1, …)` đụng `CHECK (id = 1)` ⇒ nhánh
// dọn dẹp `remove_dir_all` cả `.atproj` của người dùng.
//
// ⚠️ **Vì sao không còn một ca "trượt giữa chừng" bơm lỗi từ bên ngoài:** sau khi
// `create_work_folder` chuyển sang **tạo độc quyền** (`fs::create_dir`, không `_all`),
// không còn cách nào từ ngoài ép một lượt gọi đi vào một thư mục đã có — mọi thư mục
// đã tồn tại đều bị bỏ qua và lượt gọi nhận một tên mới. Đó chính là điều làm đường xoá
// nhầm **không tới được nữa**, và nó là một bất biến **theo cấu trúc**, không phải
// theo một phép kiểm lúc chạy. Vế "không để lại thư mục nửa vời" của AC8 vì thế được
// canh bằng ba ca **từ chối trước khi ghi** ở trên/dưới (`.docx`, không UTF-8, quá
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

    // ③ ⚠️ Hệ quả đã biết, khoá lại bằng một assert để không ai đọc nhầm: `meta.name`
    //    giữ **nguyên tên người dùng gõ** ở CẢ HAI — hai Tác phẩm hiển thị giống hệt nhau,
    //    chỉ tên thư mục khác. Đó là cái giá của tự-đánh-số so với từ-chối.
    let second_meta = WorkMeta::read(&second_dir).expect("doc meta.json moi that bai");
    assert_eq!(first_meta.name, second_meta.name);

    cleanup(&root);
}

/// Ba lần liên tiếp cùng một tên ⇒ ba thư mục, không hai.
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

    let err = create_work_from_file(&root, "Khong Phai Utf8", "zh", "", &bad_path)
        .expect_err("noi dung khong phai UTF-8 phai bi tu choi");

    // 🔵 SỬA (vòng rà đối kháng 2026-09-04, item 9) — khẳng định RÕ khoá lỗi, để ca này
    // nói được nó đang canh CHÍNH THẤT BẠI CỦA CHUỖI PIPELINE (`run_import`), không phải
    // một thất bại tầng khác trùng hợp cũng cho `is_err()`. Trước Story 6.2, `import_file`
    // tự kiểm UTF-8 và trả lỗi TRƯỚC KHI `.atproj` được tạo; sau Story 6.2, `import_file`
    // chỉ đọc byte thô, và lỗi này giờ nảy ra TỪ `run_import` SAU KHI thư mục đã được tạo —
    // doc-comment của `create_work` khẳng định nó "cuộn lại TRỌN VẸN" trên đường đó. Phép
    // kiểm thư mục rỗng ngay dưới đây là cổng cho đúng mệnh đề ấy.
    assert_eq!(
        err.message_key(),
        MessageKey::ImportNotUtf8,
        "loi phai la `import.not_utf8` -- cung khoa voi doc-comment cua `ImportError::NotUtf8`"
    );

    let entries: Vec<_> = fs::read_dir(&root).unwrap().collect();
    assert!(
        entries.is_empty(),
        "khong thu muc .atproj nao duoc tao khi noi dung khong phai UTF-8 -- `create_work`          phai cuon lai TRON VEN sau khi `run_import` (chay SAU khi thu muc da tao) tra loi"
    );

    cleanup(&root);
    cleanup(&source_dir);
}

/// 🔵 **THÊM (vòng rà đối kháng 2026-09-04, item 8)** — đường ghi N > 1 của `create_work`
/// (vòng `ord` 1..N, `first_chapter_id`, `insert_segments` mỗi Chương, `meta.json` dựng lại
/// trên N Chương) là mã sản phẩm MỚI của Story 6.2 mà 0 ca nào từng chạm: sản phẩm hôm nay
/// luôn truyền `chapter_pattern: None` (N = 1), và không test nào lái `create_work` bằng
/// hình dạng `PipelineShape::Chapters`. Ca này gọi thẳng `create_work` (hàm THUẦN, không cần
/// một `tauri::AppHandle`) với ba đơn vị đã-là-Chương và đọc lại TRỰC TIẾP từ `project.db`.
#[test]
fn create_work_writes_every_chapter_and_its_segments_when_the_pipeline_yields_more_than_one() {
    let root = temp_dir("n-chapters");

    let shape = PipelineShape::Chapters(vec![
        ChapterInput::AlreadyText("Chuong mot. Cau hai.".to_owned()),
        ChapterInput::AlreadyText("Chuong hai noi tiep.".to_owned()),
        ChapterInput::AlreadyText("Chuong ba ket thuc.".to_owned()),
    ]);
    let opened = create_work(&root, "Nhieu Chuong", "en", "", shape)
        .expect("tao Tac pham voi N > 1 Chuong that bai");

    let rows: Vec<(i64, i64, String)> = opened
        .store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id, ord, source_text FROM chapter ORDER BY ord")?;
            let mut rows_iter = stmt.query([])?;
            let mut out = Vec::new();
            while let Some(row) = rows_iter.next()? {
                out.push((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?));
            }
            Ok(out)
        })
        .expect("doc lai chapter that bai");

    assert_eq!(rows.len(), 3, "phai co dung 3 hang chapter, N Chuong day du");
    assert_eq!(rows[0].1, 1, "Chuong dau tien phai mang ord = 1");
    assert_eq!(rows[1].1, 2, "Chuong thu hai phai mang ord = 2");
    assert_eq!(rows[2].1, 3, "Chuong thu ba phai mang ord = 3");
    assert_eq!(rows[0].2, "Chuong mot. Cau hai.");
    assert_eq!(rows[1].2, "Chuong hai noi tiep.");
    assert_eq!(rows[2].2, "Chuong ba ket thuc.");

    // segment phai ton tai cho MOI Chuong, khong chi Chuong dau -- AC13 khong doi tren N > 1.
    for (chapter_id, ord, _) in &rows {
        let seg_count: i64 = opened
            .store
            .read(move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM segment WHERE chapter_id = ?1",
                    [chapter_id],
                    |row| row.get(0),
                )
            })
            .expect("dem segment that bai");
        assert!(seg_count > 0, "Chuong ord={ord} (id={chapter_id}) phai co segment");
    }

    // OpenWork.chapter_id phai chot vao Chuong DAU TIEN (ord = 1), khong phai Chuong cuoi.
    assert_eq!(
        opened.chapter_id, rows[0].0,
        "OpenWork::chapter_id phai tro dung Chuong ord = 1, khong phai Chuong duoc chen sau cung"
    );

    drop(opened);
    cleanup(&root);
}

/// 🔵 **SỬA 2026-09-04 (Story 6.2, AD-39)** — `import_text`/`import_file` không còn tự
/// chạy hết chuỗi (chúng chỉ còn cung cấp BƯỚC ĐẦU VÀO, xem
/// `core::segment::import` doc-comment). Mệnh đề của ca này KHÔNG đổi — dán văn bản và đọc
/// tệp vẫn phải cho CÙNG một kết quả — chỉ đường quan sát nó đổi: chạy cả hai hình dạng qua
/// `run_import` rồi so `source_text` của Chương đầu ra.
#[test]
fn pasted_text_and_a_read_file_travel_the_same_import_path() {
    let source_dir = temp_dir("same-path");
    let content = "mot doan van ban giong het nhau\nqua ca hai duong";

    let from_paste = run_import(PipelineInput::default_shaped(import_text(content.to_owned()), "en"))
        .expect("run_import (dan van ban) that bai");

    let path = write_file(&source_dir, "sample.txt", content.as_bytes());
    let file_shape = import_file(&path).expect("doc tep .txt hop le that bai");
    let from_file = run_import(PipelineInput::default_shaped(file_shape, "en"))
        .expect("run_import (doc tep) that bai");

    // 🔵 SỬA (vòng rà đối kháng 2026-09-04, item 15) — bản đầu index thẳng `chapters[0]` mà
    // không khẳng định `len() == 1` trước: một kết quả RỖNG (ví dụ do một lượt refactor sau
    // này lỡ tay tách văn bản này) sẽ PANIC ở chỗ index, không phải một thông điệp trượt
    // (`assert_eq!`) đọc được. Khẳng định độ dài TRƯỚC khi vào so nội dung.
    assert_eq!(from_paste.chapters.len(), 1, "dan van ban phai cho dung 1 Chuong");
    assert_eq!(from_file.chapters.len(), 1, "doc tep phai cho dung 1 Chuong");

    assert_eq!(
        from_paste.chapters[0].source_text, from_file.chapters[0].source_text,
        "dan van ban va doc tep phai di qua CUNG mot chuoi va cho ra cung mot ket qua"
    );
    // 🔵 THÊM (vòng rà đối kháng 2026-09-04, item 15) — chuỗi nay CŨNG sinh `segments`
    // (bước 7); so cả hai vế làm mệnh đề "cùng một chuỗi" MẠNH HƠN mà không tốn thêm gì --
    // hai segment khác nhau trên cùng một `source_text` sẽ vô hình nếu chỉ so văn bản.
    assert_eq!(
        from_paste.chapters[0].segments, from_file.chapters[0].segments,
        "dan van ban va doc tep phai cho CUNG mot tap segment, khong chi cung van ban"
    );

    cleanup(&source_dir);
}

/// Đối chứng dương AC8 — `.docx` bị từ chối bằng đúng khoá `MessageKey`, không bằng
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

/// AC5 vế hai — *"**không** đường dẫn tuyệt đối nào của máy cũ nằm trong `meta.json`
/// hay `project.db`"*.
///
/// 🔴 AC5 viết cùng khuôn AC3: *"test chứng minh, không phải một lời khẳng định"*. Vế
/// "mở lại được ở đường dẫn khác" đã có ca riêng; vế này trước lượt review **không có
/// ca nào** — mệnh đề đúng về cấu trúc (không trường/cột nào chứa đường dẫn) nhưng không
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
/// story này; CRLF là **chuẩn hoá**, không đụng (Epic 6).
///
/// 🔴 `EF BB BF` là UTF-8 **hợp lệ**, nên nó đi lọt `String::from_utf8` mà không cổng
/// nào kêu — và AD-4 đóng băng ranh giới segment tính lúc nhập, nên một `U+FEFF` nằm lại
/// sẽ thành ký tự đầu của segment #1 **vĩnh viễn**.
/// 🔵 **SỬA 2026-09-04 (Story 6.2, AD-39)** — strip BOM đã chuyển vào
/// `core::segment::pipeline::Step::DecodeEncoding` (xem doc-comment `strip_bom` ở đó). Mệnh
/// đề của Ice (2026-08-06) KHÔNG đổi: BOM là tạo tác giải mã, cắt; CRLF là chuẩn hoá, không
/// đụng. Đường quan sát đổi: `import_file` chỉ còn đọc byte thô, nên phải chạy qua
/// `run_import` để bước giải mã thật sự thực thi.
#[test]
fn a_utf8_bom_is_stripped_but_line_endings_are_left_alone() {
    let root = temp_dir("bom");
    let source_dir = temp_dir("bom-src");

    let path = write_file(&source_dir, "notepad.txt", b"\xEF\xBB\xBFCHUONG MOT\r\nCau hai");
    let shape = import_file(&path).expect("tep UTF-8 co BOM phai nhap duoc");
    let imported = run_import(PipelineInput::default_shaped(shape, "en"))
        .expect("run_import that bai")
        .chapters
        .remove(0);

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

    // Chỉ cắt ở ĐẦU: một U+FEFF giữa văn bản là zero-width no-break space, nội dung thật.
    let inner = write_file(&source_dir, "giua.txt", "AB\u{feff}CD".as_bytes());
    let inner_shape = import_file(&inner).expect("nhap that bai");
    let imported_inner = run_import(PipelineInput::default_shaped(inner_shape, "en"))
        .expect("run_import that bai")
        .chapters
        .remove(0);
    assert_eq!(imported_inner.source_text, "AB\u{feff}CD");

    cleanup(&root);
    cleanup(&source_dir);
}

/// Trần 100 MB (Ice chốt 2026-08-06) — và trần đó phải chặn **trước khi** đọc.
#[test]
fn a_file_past_the_size_ceiling_is_refused_before_a_single_byte_is_written() {
    let root = temp_dir("too-large");
    let source_dir = temp_dir("too-large-src");

    // ⚠️ Không ghi 100 MB thật ra đĩa trong một test — dựng một tệp THƯA (sparse):
    // `set_len` khai kích thước mà không cấp phát khối nào.
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

/// Tệp không có phần mở rộng ⇒ một hạng lỗi RIÊNG, không phải `unsupported_format`
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
    // Không phải tên thiết bị thì không đụng.
    assert_eq!(sanitize_name("CONtent"), "CONtent");

    // ② Trần theo BYTE, cắt ở biên ký tự — không panic giữa một ký tự nhiều byte.
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

/// 🔴 **AC8** — chưa Tác phẩm nào mở ⇒ một lỗi CÓ TÊN RIÊNG, không phải một lỗi kho
/// (`store.*`): `OpenWorkState` rỗng là một trạng thái sản phẩm bình thường, không một
/// tệp nào hỏng.
#[test]
fn reading_the_open_chapter_without_a_work_open_is_a_named_error() {
    let err = read_open_chapter(None).expect_err("chua Tac pham nao mo phai la mot loi");

    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
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

/// `source_lang` đọc từ Tác phẩm, không đoán từ nội dung — một Chương tiếng Anh với
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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 2.11 · FR26 — chuyển Chương trong Workspace
//
// 🔴 VÌ SAO NHỮNG CA DƯỚI ĐÂY CHÈN CHƯƠNG THỨ HAI BẰNG **SQL TRỰC TIẾP**
// ─────────────────────────────────────────────────────────────────────────────────
// Không đường sản phẩm nào sinh ra Chương thứ hai hôm nay: `create_work` chèn đúng một
// hàng `chapter` với `ord = 1` viết cứng, một lượt, không vòng lặp (`project.rs:137-142`),
// và `grep "INSERT INTO chapter" src-tauri/src` cho **đúng một** kết quả. Đường sinh
// Chương thứ hai thuộc epic khác — FR14 (nhập hàng loạt) → Epic 6, FR15 (gộp/tách Chương)
// → Epic 5.
//
// ⇒ Ice ký **Quyết định #1 đường (a)** ngày 2026-08-18: dựng **trọn** cơ chế, nghiệm thu
// AC1/AC2 bằng **hợp đồng dữ liệu** ở đây, và ghi một món nợ **có chủ** cho vế *"không
// đường sản phẩm nào"*. Khuôn này đã có chữ ký hai lần: chữ ký #8(a) của Story 2.6
// (`retired_at`) và AC3 của Story 2.7 (xuất xứ phi-`self`) — cả hai cùng lý do, cả hai
// ghi nợ thay vì tự chấm đạt.
//
// ⚠️ **Giá đã trả và phải nói ra:** AC1/AC2 **không** có đường e2e. Bốn đường nghiệm thu
// của dự án có bốn vai, và vai *"hợp đồng dữ liệu ở biên và với `ord` thưa"* là của
// `cargo test`. Nhưng *"người dùng bấm phím và Chương đổi"* thì hôm nay **không đường nào
// với tới được** — đó là món nợ, không phải một ca đã xanh.
// ═════════════════════════════════════════════════════════════════════════════════

/// Chèn một hàng `chapter` **thẳng bằng SQL** và trả `chapter.id` vừa sinh.
///
/// ⚠️ `ord` là **tham số**, có chủ ý: hai ca dưới đây cần dựng một bảng `chapter` mà
/// `ord` **trùng nhau** — thứ mà không đường sản phẩm nào sinh ra được hôm nay, nhưng
/// lược đồ **cho phép** (`schema.rs:249` cố ý không `UNIQUE`).
fn insert_chapter_directly(opened: &auratranslate_lib::commands::project::OpenWork, ord: i64, text: &str) -> i64 {
    let text = text.to_owned();
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (?1, NULL, ?2, 'not_started', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                (ord, &text),
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("chen Chuong bang SQL truc tiep that bai")
}

/// 🔴 **AC1** — gọi *Chương sau* mở ra Chương kế tiếp, và `OpenWork` **nhớ** lượt đổi đó.
///
/// Vế thứ hai mới là vế dễ hụt: một lệnh trả đúng dữ liệu nhưng **không** dời con trỏ
/// Chương trên `OpenWork` sẽ cho lượt gọi thứ hai trả lại **cùng** một Chương — và không
/// một phép khẳng định nào về giá trị trả về bắt được chuyện đó.
#[test]
fn moving_to_the_next_chapter_opens_the_following_row_and_remembers_the_move() {
    let root = temp_dir("next-chapter");
    let mut opened = create_work_from_text(&root, "Chuyen Chuong", "zh", "", "Chuong mot.".to_owned())
        .expect("tao tac pham that bai");

    let first = opened.chapter_id;
    let second = insert_chapter_directly(&opened, 2, "Chuong hai.");
    let third = insert_chapter_directly(&opened, 3, "Chuong ba.");

    let moved = open_adjacent_chapter(Some(&mut opened), ChapterDirection::Next)
        .expect("chuyen sang Chuong sau that bai");
    assert_eq!(moved.outcome, ChapterSwitchOutcome::Moved);
    let chapter = moved.chapter.expect("mot luot doi THAT SU phai mang Chuong moi");
    assert_eq!(chapter.chapter_id, second);
    assert_eq!(chapter.source_text, "Chuong hai.");
    assert_eq!(
        opened.chapter_id, second,
        "con tro Chuong tren OpenWork phai doi — neu khong, luot goi ke tiep tra lai cung mot Chuong"
    );

    // Lượt thứ hai đi tiếp, không đứng yên — đây là phép đối chứng cho câu trên.
    let again = open_adjacent_chapter(Some(&mut opened), ChapterDirection::Next)
        .expect("chuyen sang Chuong sau lan hai that bai");
    assert_eq!(again.outcome, ChapterSwitchOutcome::Moved);
    assert_eq!(again.chapter.expect("phai co Chuong").chapter_id, third);

    assert_ne!(first, second, "hai Chuong phai la hai hang khac nhau");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// 🔴 **AC2** — *Chương trước* mở ra Chương liền trước, đối xứng với AC1.
#[test]
fn moving_to_the_previous_chapter_opens_the_row_immediately_before() {
    let root = temp_dir("prev-chapter");
    let mut opened = create_work_from_text(&root, "Chuong Truoc", "zh", "", "Chuong mot.".to_owned())
        .expect("tao tac pham that bai");

    let first = opened.chapter_id;
    let second = insert_chapter_directly(&opened, 2, "Chuong hai.");

    opened.chapter_id = second;

    let moved = open_adjacent_chapter(Some(&mut opened), ChapterDirection::Prev)
        .expect("chuyen ve Chuong truoc that bai");
    assert_eq!(moved.outcome, ChapterSwitchOutcome::Moved);
    assert_eq!(moved.chapter.expect("phai co Chuong").chapter_id, first);
    assert_eq!(opened.chapter_id, first);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// 🔴 **AC4** — ở Chương CUỐI, lệnh *Chương sau* báo biên và **không quay vòng**.
///
/// Ba mệnh đề trong một ca, và cả ba đều cần: kết quả là một trạng thái **phân biệt được**
/// (không phải `Err` — *"đã ở Chương cuối"* không phải một lỗi); **không** Chương nào được
/// trả về; và con trỏ Chương trên `OpenWork` **đứng nguyên**.
#[test]
fn the_last_chapter_reports_the_boundary_instead_of_wrapping_around() {
    let root = temp_dir("last-chapter");
    let mut opened = create_work_from_text(&root, "Bien Cuoi", "zh", "", "Chuong duy nhat.".to_owned())
        .expect("tao tac pham that bai");

    let only = opened.chapter_id;

    let stopped = open_adjacent_chapter(Some(&mut opened), ChapterDirection::Next)
        .expect("vuot bien KHONG duoc la mot Err — do khong phai mot loi");
    assert_eq!(stopped.outcome, ChapterSwitchOutcome::AtLast);
    assert!(
        stopped.chapter.is_none(),
        "khong doi Chuong thi khong mang mot Chuong nao ve — mot khoi source_text thua o day \
         la mot loi moi de webview thay Chuong da doi"
    );
    assert_eq!(opened.chapter_id, only, "KHONG quay vong ve dau");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// 🔴 **AC4**, nửa đối xứng — ở Chương ĐẦU, lệnh *Chương trước* báo biên.
///
/// ⚠️ Mọi Chương tồn tại hôm nay vừa là Chương đầu **vừa là** Chương cuối, nên ca này và
/// ca trên chạy trên **cùng** một hình dạng dữ liệu mà sản phẩm thật sinh ra.
#[test]
fn the_first_chapter_reports_the_boundary_instead_of_wrapping_around() {
    let root = temp_dir("first-chapter");
    let mut opened = create_work_from_text(&root, "Bien Dau", "zh", "", "Chuong duy nhat.".to_owned())
        .expect("tao tac pham that bai");

    let only = opened.chapter_id;

    let stopped = open_adjacent_chapter(Some(&mut opened), ChapterDirection::Prev)
        .expect("vuot bien KHONG duoc la mot Err");
    assert_eq!(stopped.outcome, ChapterSwitchOutcome::AtFirst);
    assert!(stopped.chapter.is_none());
    assert_eq!(opened.chapter_id, only, "KHONG quay vong ve cuoi");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// 🔴 **`ord` KHÔNG `UNIQUE`, và Chương kề phải phân giải bằng bộ đôi `(ord, id)`.**
///
/// `schema.rs:249` cố ý không đặt `UNIQUE` trên `chapter.ord`, và không gì bảo đảm nó liên
/// tục. Một cài đặt `ord + 1` **biên dịch sạch, đi qua mọi cổng**, và trỏ sai hàng ngay khi
/// hai Chương chia nhau một `ord` — đúng lớp lỗi mà lượt code review 2026-08-17 trên
/// `segment.rs` gọi là *"một phép trừ im lặng trỏ sai hàng"*.
///
/// Ca này dựng đúng hình dạng đó: **ba** Chương cùng `ord = 1`, phân biệt nhau **chỉ** bằng
/// `id`. Với `ord + 1` thì không hàng nào có `ord = 2` ⇒ lệnh báo *"đã ở Chương cuối"* trên
/// một Tác phẩm còn hai Chương phía sau.
#[test]
fn adjacent_chapters_break_ord_ties_by_id_never_by_arithmetic() {
    let root = temp_dir("ord-ties");
    let mut opened = create_work_from_text(&root, "Ord Trung", "zh", "", "Chuong mot.".to_owned())
        .expect("tao tac pham that bai");

    let first = opened.chapter_id;
    let second = insert_chapter_directly(&opened, 1, "Chuong hai.");
    let third = insert_chapter_directly(&opened, 1, "Chuong ba.");

    let step_one = open_adjacent_chapter(Some(&mut opened), ChapterDirection::Next)
        .expect("buoc mot that bai");
    assert_eq!(step_one.outcome, ChapterSwitchOutcome::Moved);
    assert_eq!(
        step_one.chapter.expect("phai co Chuong").chapter_id,
        second,
        "ba Chuong cung ord = 1: hang ke tiep phai phan giai bang `id`, khong bang mot phep cong"
    );

    let step_two = open_adjacent_chapter(Some(&mut opened), ChapterDirection::Next)
        .expect("buoc hai that bai");
    assert_eq!(step_two.chapter.expect("phai co Chuong").chapter_id, third);

    // Và đường về cũng phải phân giải bằng `id`, không chỉ đường đi.
    let back = open_adjacent_chapter(Some(&mut opened), ChapterDirection::Prev)
        .expect("buoc lui that bai");
    assert_eq!(back.chapter.expect("phai co Chuong").chapter_id, second);
    assert_ne!(first, second);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// `ord` **thưa** (1, 7, 900) — cùng luật, một hình dạng khác: `ord + 1` cũng chết ở đây.
#[test]
fn adjacent_chapters_work_on_a_sparse_ord_sequence() {
    let root = temp_dir("ord-sparse");
    let mut opened = create_work_from_text(&root, "Ord Thua", "zh", "", "Chuong mot.".to_owned())
        .expect("tao tac pham that bai");

    let far = insert_chapter_directly(&opened, 900, "Chuong xa.");
    let middle = insert_chapter_directly(&opened, 7, "Chuong giua.");

    let step_one = open_adjacent_chapter(Some(&mut opened), ChapterDirection::Next)
        .expect("buoc mot that bai");
    assert_eq!(step_one.chapter.expect("phai co Chuong").chapter_id, middle);

    let step_two = open_adjacent_chapter(Some(&mut opened), ChapterDirection::Next)
        .expect("buoc hai that bai");
    assert_eq!(step_two.chapter.expect("phai co Chuong").chapter_id, far);

    let step_three = open_adjacent_chapter(Some(&mut opened), ChapterDirection::Next)
        .expect("buoc ba that bai");
    assert_eq!(step_three.outcome, ChapterSwitchOutcome::AtLast);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// Chưa Tác phẩm nào mở ⇒ **cùng** khoá `project.no_work_open`, không một khoá thứ hai.
#[test]
fn switching_chapters_without_a_work_open_reuses_the_named_error() {
    let err = open_adjacent_chapter(None, ChapterDirection::Next)
        .expect_err("chua Tac pham nao mo phai la mot loi");

    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
    assert!(!err.retryable());
}

/// 🔴 **Món nợ `deferred-work.md:650`, giao đích danh story này** — hàng `chapter` vắng mặt
/// ⇒ một lỗi **CÓ TÊN**, không `store.read_failed`.
///
/// Trước story này `conn.query_row(...)` ném `QueryReturnedNoRows`, đi qua
/// `From<StoreError>` thành `store.read_failed`, và người dùng đọc *"không mở được kho dữ
/// liệu"* cho một Tác phẩm hoàn toàn lành lặn. Ca này dựng đúng trạng thái đó bằng cách
/// **xoá** hàng `chapter` — một hình dạng mà Epic 5 (gộp/tách/sắp lại Chương, FR15) sẽ với
/// tới thật.
#[test]
fn a_missing_chapter_row_is_a_named_error_not_a_store_error() {
    let root = temp_dir("chapter-gone");
    let opened = create_work_from_text(&root, "Chuong Bien Mat", "zh", "", "Noi dung.".to_owned())
        .expect("tao tac pham that bai");

    let gone = opened.chapter_id;
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("DELETE FROM segment WHERE chapter_id = ?1", [gone])?;
            tx.execute("DELETE FROM chapter WHERE id = ?1", [gone])?;
            Ok(())
        })
        .expect("xoa hang chapter that bai");

    let err = read_open_chapter(Some(&opened)).expect_err("Chuong vang mat phai la mot loi");

    assert_eq!(
        err.code(),
        "segment.chapter_not_found",
        "phai la mot loi CO TEN, khong phai `store.read_failed` — mot Tac pham lanh lan \
         khong duoc doc thanh mot kho hong"
    );
    assert_eq!(err.message_key(), MessageKey::SegmentChapterNotFound);
    assert_eq!(
        err.params().get("chapter_id").map(String::as_str),
        Some(gone.to_string().as_str()),
        "tham so mang DU LIEU (chapter_id), khong mang mot cau"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.1 — ba mệnh đề của Epic 5 §Requirements chỉ sống trong chú thích trước story
// này: không có thực thể tầng thứ ba, `source_lang` bất biến, Glossary/TM phân giải ở
// `Tier::Work`. `naming_boundary.rs` cưỡng chế QUY ƯỚC ĐẶT TÊN (tĩnh trên cây nguồn); ba ca
// dưới đây cưỡng chế HÌNH DẠNG DỮ LIỆU và HÀNH VI LÚC CHẠY — trộn hai thứ vào một tệp là
// làm hỏng đúng thứ khiến cả hai đọc được (xem doc-comment đầu `naming_boundary.rs`).
// ═════════════════════════════════════════════════════════════════════════════════

/// Mọi bảng KHÔNG PHẢI thực thể tầng — chi tiết/thuộc tính gắn theo `chapter`/`work`
/// (`segment`/`segment_version`, `glossary_entry`/`glossary_candidate`,
/// `chapter_position` — Story 5.7; `reading_mark` — Story 5.13), sổ sách nội bộ của chính cơ chế di trú
/// (`schema_migration_log`), hoặc do SQLite tự sinh (`sqlite_sequence`, mọi bảng dùng
/// `AUTOINCREMENT`).
///
/// 🔵 **THÊM (2026-08-29, Story 5.7) — `chapter_position`.** Không phải một CONTAINER giữa
/// `work` và `chapter`: nó là một thuộc tính GẮN THEO một hàng `chapter` (vị trí caret của
/// chính Chương đó), khoá chính là `chapter_id`, cùng vai với `segment`/`segment_version`
/// gắn theo `chapter`/`segment`. Xem §Design Notes "Vì sao một BẢNG riêng cho vị trí" của
/// `5-7-danh-sach-chuong-va-mo-chuong-vao-workspace.md`.
const NON_ENTITY_DETAIL_TABLES: [&str; 8] = [
    "chapter_position",
    "glossary_candidate",
    "glossary_entry",
    "reading_mark",
    "schema_migration_log",
    "segment",
    "segment_version",
    "sqlite_sequence",
];

/// **Hàm thuần** — lọc `tables` qua [`NON_ENTITY_DETAIL_TABLES`]. Tách khỏi thân test để
/// [`the_entity_table_check_would_actually_flag_a_seeded_third_tier_table`] gọi được trên
/// một danh sách DỰNG TAY, không cần mở `Store` — cùng khuôn đối chứng dương mà
/// `the_update_source_lang_check_would_actually_flag_a_seeded_violation` đã dùng cho vế
/// `source_lang`.
fn entity_tables(tables: &[String]) -> Vec<String> {
    tables.iter().filter(|t| !NON_ENTITY_DETAIL_TABLES.contains(&t.as_str())).cloned().collect()
}

/// 🔴 **Story 5.1** — `project.db` không mang một bảng THỰC THỂ nào giữa `work` và
/// `chapter`. AD-9/Story 1.15 nói mô hình là hai tầng, đúng hai: một tài liệu đơn lẻ là một
/// Tác phẩm có đúng một Chương, không một cấp trung gian (`volume`/`book`/`series`, …).
///
/// Một bảng MỚI xuất hiện ngoài [`NON_ENTITY_DETAIL_TABLES`] bắt ca dưới đây đỏ — đọc lý do
/// bảng đó tồn tại trước khi thêm nó vào miễn trừ: nếu nó là một CONTAINER giữa Work và
/// Chapter, đó là vi phạm AD-9 thật và là quyết định phạm vi của Ice, không phải một lượt vá
/// tiện tay.
#[test]
fn project_db_has_no_entity_table_between_work_and_chapter() {
    let root = temp_dir("no-third-tier");

    let opened = create_work_from_text(&root, "Ba Tang", "zh", "", "noi dung".to_owned())
        .expect("tao tac pham that bai");

    let tables: Vec<String> = opened
        .store
        .read(|conn| {
            let mut stmt =
                conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")?;
            let mut rows = stmt.query([])?;
            let mut out = Vec::new();
            while let Some(row) = rows.next()? {
                out.push(row.get::<_, String>(0)?);
            }
            Ok(out)
        })
        .expect("doc danh sach bang cua project.db that bai");

    assert_eq!(
        entity_tables(&tables),
        vec!["chapter".to_owned(), "work".to_owned()],
        "project.db mang mot bang THUC THE ngoai `work`/`chapter`: toan bo bang tim thay la \
         {tables:?}. Neu day la mot chi tiet hop le (nhu segment/glossary), them vao \
         NON_ENTITY_DETAIL_TABLES kem ly do tai sao no khong phai mot container. Neu day la \
         mot thuc the tang THU BA giua Work va Chapter -- DUNG LAI, day la vi pham AD-9 that \
         va la quyet dinh pham vi cua Ice."
    );

    drop(opened.store);
    cleanup(&root);
}

/// 🔴 **Vòng rà đối kháng — P8.** Ca trên chỉ đối chiếu `sqlite_master` THẬT với một danh
/// sách cứng — không có gì chứng minh vị từ [`entity_tables`] thật sự BẮT được một bảng thực
/// thể tầng ba nếu nó xuất hiện. Ca này gieo tay một bảng `volume` (container giả định giữa
/// Work và Chapter) vào một danh sách bảng dựng tay, độc lập với `project.db` hôm nay có gì.
#[test]
fn the_entity_table_check_would_actually_flag_a_seeded_third_tier_table() {
    let seeded_tables: Vec<String> = [
        "chapter",
        "glossary_candidate",
        "glossary_entry",
        "schema_migration_log",
        "segment",
        "segment_version",
        "sqlite_sequence",
        "volume", // <- gieo: mot container tang BA dung tay, giua Work va Chapter
        "work",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();

    let entities = entity_tables(&seeded_tables);

    assert_eq!(
        entities,
        vec!["chapter".to_owned(), "volume".to_owned(), "work".to_owned()],
        "ca DUONG THAT: mot bang `volume` gieo tay phai lot qua bo loc \
         NON_ENTITY_DETAIL_TABLES va bi vi tu bat, nhung entity_tables tra ve {entities:?}"
    );
}

/// Đường dẫn dựa trên `CARGO_MANIFEST_DIR` — không đi qua `naming_boundary.rs` (tệp đó KHÔNG
/// canh mệnh đề nào về `source_lang`; xem phân công ở đầu §Story 5.1 của tệp này) nên walk
/// cây nguồn được viết lại cục bộ, tối giản, đúng khuôn mọi `*_boundary.rs` khác.
fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("doc thu muc {}: {e}", dir.display()));
    for entry in entries {
        // 🔵 VA (vong ra doi khang) — ban truoc `entries.flatten()` NUOT mot `DirEntry` loi va
        // BO QUA IM LANG ca tep do, lam phep quet hut ma khong co tin hieu nao. Mot phep kiem
        // "cay rong doc thanh sach" khong dong duoc gi neu tung ENTRY co the tu lang le bien mat.
        let entry = entry.unwrap_or_else(|e| panic!("doc mot muc trong {}: {e}", dir.display()));
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)
            .unwrap_or_else(|e| panic!("lstat {}: {e}", path.display()));
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Mọi chuỗi literal (`"..."`/`r"..."`/`r#"..."#`/`b"..."`) trong `text`, nối các đoạn bị cắt
/// bởi `\` cuối dòng — đúng ngữ nghĩa nối chuỗi của Rust cho khuôn `"UPDATE segment SET ... \`
/// xuống dòng `WHERE ..."` mà `core/store/schema.rs`/`commands/segment.rs` dùng khắp nơi.
///
/// 🔴 **VÁ (vòng rà đối kháng)** — bản trước có BA lỗ, cả ba cho XANH GIẢ:
/// 1. **Ký tự literal chứa `"`** (`'"'`) làm một bộ tách chỉ phản ứng với `"` lệch pha: nó đọc
///    `'` như một ký tự thường, rồi đọc `"` bên trong làm MỞ một chuỗi, và nuốt mọi thứ tới
///    dấu `"` thật kế tiếp — có thể trôi qua nhiều dòng, làm mất đúng chuỗi SQL cần quét.
/// 2. **Raw string** (`r"…"`/`r#"…"#`) không có escape `\` — bản trước xử `\` bên trong raw
///    string như một escape THẬT, làm sai nội dung trích ra.
/// 3. **Comment `//` được cắt Ở TẦNG DÒNG, trước khi hàm này chạy** (bản trước nhận `text` đã
///    bị `no_update_statement_anywhere_touches_source_lang` xoá trắng mọi dòng bắt đầu bằng
///    `//`): một dòng NỐI TIẾP của một chuỗi xuyên dòng (sau dấu `\` cuối dòng trước) mà tình
///    cờ mở đầu bằng `//` (sau khi trim) bị xoá trắng NHƯ MỘT DÒNG COMMENT dù nó là NỘI DUNG
///    CHUỖI — mất một phần literal mà không có dấu hiệu nào.
///
/// Sửa bằng MỘT bộ quét CÓ TRẠNG THÁI xuyên suốt `text` gốc (không tiền xử lý dòng trước):
/// comment `//` chỉ bị cắt khi con trỏ đang Ở NGOÀI một chuỗi; ký tự literal (`'x'`, `'\n'`,
/// `'\''`, `'\u{2019}'`, …) được nhận diện và NHẢY QUA nguyên khối, không để `"` bên trong nó
/// mở nhầm một chuỗi; raw string được trích theo đúng số dấu `#` của chính nó, không xử `\`.
///
/// ⚠️ **GIỚI HẠN CÒN LẠI, ghi ra thay vì để người sau tự phát hiện** (đúng chuẩn mà
/// `naming_boundary.rs` đã theo cho mọi vị từ quét tĩnh của nó):
/// - **Không xử khối `/* … */`** — cùng lý do `naming_boundary.rs::the_rust_tree_still_has_zero_block_comments`
///   đã đo và khoá lại: `src-tauri/src/**` có 0 khối comment C thật (đo 2026-08-27). Nếu số đó
///   đổi, ca đó đỏ trước, không phải ca này đỏ oan hay bỏ sót.
/// - **SQL ghép bằng `format!`/`concat!`/nối `String` lúc chạy nằm NGOÀI tầm quét**: đây là
///   một bộ trích LITERAL TĨNH, không phải một trình thông dịch Rust. Một `UPDATE` chạm
///   `source_lang` được LẮP RA từ nhiều mảnh literal rời (không mảnh nào tự nó mang cả hai từ
///   khoá `UPDATE` và `source_lang`) sẽ không bị ca này bắt. Cả kho hôm nay không chỗ nào ghép
///   SQL theo cách đó (mọi câu SQL trong `core/store/**`/`commands/**` là MỘT literal, nối
///   bằng `\`-continuation) — nhưng đây là một giả định về HIỆN TRẠNG, không phải một bảo đảm
///   của chính vị từ.
fn rust_string_literals(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out = Vec::new();
    let mut i = 0usize;

    while i < n {
        // Raw string (tuỳ chọn tiền tố `b`): `r"…"`, `r#"…"#`, `br"…"`, `br#"…"#`, …
        if chars[i] == 'r' || (chars[i] == 'b' && chars.get(i + 1) == Some(&'r')) {
            let r_at = if chars[i] == 'b' { i + 1 } else { i };
            if chars.get(r_at) == Some(&'r') {
                let mut k = r_at + 1;
                let mut hashes = 0usize;
                while chars.get(k) == Some(&'#') {
                    hashes += 1;
                    k += 1;
                }
                if chars.get(k) == Some(&'"') {
                    let content_start = k + 1;
                    let closing: Vec<char> =
                        std::iter::once('"').chain(std::iter::repeat_n('#', hashes)).collect();
                    let mut m = content_start;
                    while m < n && chars[m..(m + closing.len()).min(n)] != closing[..] {
                        m += 1;
                    }
                    out.push(chars[content_start..m].iter().collect());
                    i = (m + closing.len()).min(n);
                    continue;
                }
            }
        }

        // Ký tự literal — nhảy qua NGUYÊN KHỐI, không để `"` bên trong (vd. `'"'`) mở nhầm một
        // chuỗi. Không khớp được (vd. dấu `'` mở đầu MỘT lifetime như `'a`) ⇒ coi là một ký tự
        // thường, đi tiếp — lifetime không có `"` bên trong nên không có gì để lệch pha.
        if chars[i] == '\'' {
            if let Some(end) = char_literal_end(&chars, i) {
                i = end;
                continue;
            }
            i += 1;
            continue;
        }

        // Comment dòng — CHỈ cắt khi đang NGOÀI một chuỗi (đúng vị trí trong vòng lặp này).
        if chars[i] == '/' && chars.get(i + 1) == Some(&'/') {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Chuỗi thường — nối các đoạn bị cắt bởi `\` cuối dòng.
        if chars[i] == '"' {
            let mut j = i + 1;
            let mut literal = String::new();
            while j < n {
                match chars[j] {
                    '\\' => {
                        j += 1;
                        if j >= n {
                            break;
                        }
                        if chars[j] == '\n' {
                            j += 1;
                            while j < n && matches!(chars[j], ' ' | '\t') {
                                j += 1;
                            }
                        } else {
                            literal.push(chars[j]);
                            j += 1;
                        }
                    }
                    '"' => {
                        j += 1;
                        break;
                    }
                    c => {
                        literal.push(c);
                        j += 1;
                    }
                }
            }
            out.push(literal);
            i = j;
            continue;
        }

        i += 1;
    }

    out
}

/// `chars[start] == '\''` — trả về chỉ số NGAY SAU dấu `'` đóng nếu đây đúng là một KÝ TỰ
/// literal (`'x'`, `'\n'`, `'\''`, `'\u{2019}'`, …), hoặc `None` nếu không khớp hình dạng đó
/// (nhiều khả năng nhất: một APOSTROPHE mở đầu LIFETIME, vd. `&'a str`, không có dấu đóng).
fn char_literal_end(chars: &[char], start: usize) -> Option<usize> {
    let n = chars.len();
    let mut i = start + 1;
    if i >= n {
        return None;
    }
    if chars[i] == '\\' {
        i += 1;
        if i >= n {
            return None;
        }
        if chars[i] == 'u' && chars.get(i + 1) == Some(&'{') {
            i += 2;
            while i < n && chars[i] != '}' {
                i += 1;
            }
            if i >= n {
                return None;
            }
            i += 1;
        } else {
            i += 1;
        }
    } else {
        i += 1;
    }
    (chars.get(i) == Some(&'\'')).then_some(i + 1)
}

/// 🔴 **Story 5.1** — `work.source_lang` là bất biến (AD-18, AC1): *"ngôn ngữ nguồn được đặt
/// lúc tạo và không đổi được về sau"*. `schema.rs` tự khai mệnh đề này bằng CHỮ ở doc-comment
/// của `WORK_DDL` (*"bất biến này được cưỡng chế ở tầng ứng dụng… không có `UPDATE` nào chạm
/// cột này"*) — ca này biến lời tự khai đó thành một phép đo: quét MỌI chuỗi SQL trong
/// `src-tauri/src/**`, đòi không chuỗi nào vừa mang từ khoá `UPDATE` vừa nhắc `source_lang`.
///
/// ⚠️ Quét chuỗi literal, không quét theo TÊN BẢNG `work`: một `UPDATE` tương lai chạm cột
/// khác của `work` (`name`, `genre`, …) là hợp lệ và không phải mệnh đề ca này canh — chỉ
/// `source_lang` là bất biến, không phải cả hàng.
///
/// 🔵 **MIỄN TRỪ CÓ TÊN, THÊM 2026-08-27 (Story 5.3) — bảng `library_work` không thuộc phạm
/// vi ca này.** `library_work` (`library-index.db`) là một CACHE DẪN XUẤT (AD-8): TOÀN BỘ
/// hàng của nó — kể cả cột `source_lang`, chép nguyên văn từ `meta.json` — được GHI LẠI ở
/// MỌI lượt `Indexer::rebuild` (Story 5.3 đổi ngữ nghĩa từ xoá-sạch-ghi-lại sang đối chiếu/
/// UPSERT). Đây KHÔNG phải bất biến AD-18/AC1 mà ca này tồn tại để canh (`work.source_lang`
/// trong `project.db`, đặt lúc tạo và không đổi được) — hai bảng, hai kho, hai luật khác
/// nhau chỉ trùng TÊN CỘT. Giá trị bên trong không đổi (WorkMeta::source_lang cũng bất biến
/// theo đúng luật đó), chỉ CÂU LỆNH ghi lại nó mỗi lượt quét — cùng cách toàn bộ `name`/
/// `genre`/`chapter_count` cũng được ghi lại. Miễn trừ CÓ TÊN (`"library_work"` — chỉ SQL
/// nhắm đúng bảng đó mới được loại), không phải một sự hạ ngưỡng: một `UPDATE work SET
/// source_lang` tương lai (không nhắc `library_work`) vẫn bị bắt, xem
/// [`the_update_source_lang_check_would_actually_flag_a_seeded_violation`] ngay dưới.
#[test]
fn no_update_statement_anywhere_touches_source_lang() {
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk_rs_files(&src_root, &mut files);

    assert!(
        files.len() >= 40,
        "chi tim thay {} tep `.rs` duoi `src-tauri/src/**` -- cay qua nho de la that, nghi \
         pham: goc quet sai",
        files.len()
    );

    let mut violations: Vec<String> = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap_or_else(|e| panic!("doc {}: {e}", file.display()));

        // ⚠️ `rust_string_literals` NHẬN VĂN BẢN GỐC, không một bản đã tiền xử lý xoá dòng
        // `//` — chính hàm đó cắt comment CÓ TRẠNG THÁI (chỉ ngoài chuỗi). Tiền xử lý theo
        // DÒNG từng làm mất một dòng NỐI TIẾP của một chuỗi xuyên dòng nếu dòng đó tình cờ mở
        // đầu bằng `//` sau khi trim — xem doc-comment của `rust_string_literals`.
        for literal in rust_string_literals(&text) {
            if !literal.contains("UPDATE") || !literal.contains("source_lang") {
                continue;
            }
            // Miễn trừ CÓ TÊN — xem khối 🔵 ở doc-comment ngay trên hàm này.
            if literal.contains("library_work") {
                continue;
            }
            violations.push(format!("{}: {literal:?}", file.display()));
        }
    }

    assert!(
        violations.is_empty(),
        "{} chuoi SQL vua mang `UPDATE` vua nhac `source_lang`:\n{}\n\n\
         AD-18/AC1: `source_lang` la truong BAT BIEN, dat luc tao Tac pham va khong doi duoc \
         ve sau. Neu day la mot thay doi CO CHU (vd. mo duong sua ngon ngu nguon sau khi tao \
         -- chua story nao dinh nghia), day la mot quyet dinh kien truc can Ice chot, khong \
         phai mot lan sua tien tay.",
        violations.len(),
        violations.join("\n")
    );
}

/// Vị từ đối chứng của [`rust_string_literals`] — cùng đối chứng dương/âm mà mọi phép quét
/// tĩnh khác trong kho đòi: chứng minh vị từ NỔ ĐƯỢC trên một chuỗi vi phạm dựng tay, độc
/// lập với việc cây nguồn hôm nay có gì.
#[test]
fn the_update_source_lang_check_would_actually_flag_a_seeded_violation() {
    let seeded = r#"
        tx.execute(
            "UPDATE work SET source_lang = ?1 \
             WHERE id = 1",
            (&new_lang,),
        )?;
    "#;

    let literals = rust_string_literals(seeded);
    assert!(
        literals
            .iter()
            .any(|l| l.contains("UPDATE") && l.contains("source_lang")),
        "ca DUONG THAT dung tay: mot UPDATE cham source_lang phai bi vi tu bat, nhung \
         rust_string_literals tra ve {literals:?}"
    );

    let clean = r#"
        tx.execute(
            "INSERT INTO work (id, work_id, name, source_lang, genre, created_at, updated_at) \
             VALUES (1, ?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
             strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (&work_id, &name, &source_lang, &genre),
        )?;
    "#;
    let clean_literals = rust_string_literals(clean);
    assert!(
        !clean_literals
            .iter()
            .any(|l| l.contains("UPDATE") && l.contains("source_lang")),
        "ca AM: mot INSERT nhac `source_lang` (khong `UPDATE`) khong duoc bi bat oan: \
         {clean_literals:?}"
    );
}

/// **THÊM Story 5.3.** Đối chứng của miễn trừ `"library_work"` — xem khối 🔵 ở doc-comment
/// của [`no_update_statement_anywhere_touches_source_lang`]. Vị từ CHUNG (`.contains("UPDATE")
/// && .contains("source_lang") && !.contains("library_work")`) phải cho ra hai kết luận khác
/// nhau trên hai chuỗi chỉ khác nhau ĐÚNG một điểm: bảng đích.
#[test]
fn the_library_work_exemption_is_named_and_does_not_swallow_the_work_table() {
    let is_violation = |literal: &str| {
        literal.contains("UPDATE") && literal.contains("source_lang") && !literal.contains("library_work")
    };

    // Câu UPSERT thật của `Indexer::rebuild` (Story 5.3) -- MIỄN TRỪ, vì nó nhắm `library_work`,
    // một cache dẫn xuất (AD-8), không phải bất biến AD-18 của `work`.
    let library_work_upsert = "INSERT INTO library_work (work_id, source_lang) VALUES (?1, ?2) \
         ON CONFLICT (work_id) DO UPDATE SET source_lang = excluded.source_lang";
    assert!(
        !is_violation(library_work_upsert),
        "UPSERT nhắm `library_work` phải được miễn trừ -- nó là một cache dẫn xuất, không \
         phải bất biến AD-18 của bảng `work`"
    );

    // Đổi ĐÚNG một điểm -- bảng đích thành `work` -- và vị từ phải bắt lại NGAY, chứng minh
    // miễn trừ không nuốt luôn cả một `UPDATE work` thật.
    let work_update = "UPDATE work SET source_lang = ?1 WHERE id = 1";
    assert!(
        is_violation(work_update),
        "một UPDATE nhắm bảng `work` (không nhắc `library_work`) PHẢI vẫn bị bắt -- miễn trừ \
         không được rộng hơn đúng bảng nó đặt tên"
    );
}

/// 🔴 **Vòng rà đối kháng — P5.** Ba lỗ của `rust_string_literals` (ký tự literal chứa dấu
/// nháy kép, raw string, dòng nối tiếp mở đầu bằng `//`), mỗi lỗ một ca dựng tay, độc lập với
/// cây nguồn hôm nay có gì.
#[test]
fn rust_string_literals_handles_char_literals_raw_strings_and_wrapped_comment_shaped_lines() {
    // (1) Ky tu literal chua dau nhay kep khong duoc mo nham mot chuoi -- SQL that dung NGAY
    // SAU no van phai trich duoc nguyen ven, khong bi nuot vao "chuoi" bat dau tu dau nhay kep
    // ben trong ky tu literal do.
    let with_quote_char_literal = "const QUOTE: char = '\"';\ntx.execute(\"UPDATE work SET source_lang = ?1 WHERE id = 1\", (&lang,))?;";
    let literals_1 = rust_string_literals(with_quote_char_literal);
    assert!(
        literals_1.iter().any(|l| l.contains("UPDATE") && l.contains("source_lang")),
        "ca DUONG THAT (1): ky tu chua dau nhay kep lam lech pha bo tach, nen UPDATE that sau \
         no bi mat. Nhan duoc: {literals_1:?}"
    );

    // (2) Raw string khong co escape backslash -- mot backslash ben trong raw string KHONG
    // duoc noi dong hay bi nuot; noi dung phai ra dung nguyen van, gom ca dau backslash do.
    let raw = "let pattern = r\"UPDATE work SET source_lang = \\d\";";
    let literals_2 = rust_string_literals(raw);
    assert_eq!(
        literals_2,
        vec!["UPDATE work SET source_lang = \\d".to_owned()],
        "ca DUONG THAT (2): raw string phai ra dung nguyen van, backslash KHONG phai escape \
         ben trong no. Nhan duoc: {literals_2:?}"
    );

    // (3) Mot dong NOI TIEP cua chuoi xuyen dong ma mo dau bang `//` sau khi trim khong duoc bi
    // hieu nham la mot dong comment -- no van la NOI DUNG CHUOI, vi con tro dang O TRONG chuoi
    // khi gap `//` do.
    let wrapped =
        "tx.execute(\n    \"UPDATE work SET \\\n     // source_lang = ?1\",\n    (),\n)?;";
    let literals_3 = rust_string_literals(wrapped);
    assert!(
        literals_3.iter().any(|l| l.contains("UPDATE") && l.contains("source_lang")),
        "ca DUONG THAT (3): dong noi tiep mo dau bang `//` van la NOI DUNG CHUOI, khong phai \
         comment -- nhan duoc {literals_3:?}"
    );

    // Doi chung AM: mot comment `//` THAT (ngoai chuoi) mang ca hai tu khoa khong duoc bat oan.
    let real_comment =
        "// UPDATE work SET source_lang = ?1 -- vi du trong comment, khong phai ma";
    let literals_4 = rust_string_literals(real_comment);
    assert!(
        literals_4.is_empty(),
        "ca AM: mot dong COMMENT THAT nhac ca hai tu khoa khong duoc sinh ra literal nao: \
         {literals_4:?}"
    );
}

/// 🔴 **Vòng rà đối kháng — P6.** Comment `//` cuối dòng (SAU một chuỗi thật trên cùng dòng)
/// phải bị cắt, không được đọc như một phần chuỗi.
#[test]
fn a_trailing_line_comment_after_real_code_is_not_read_as_part_of_a_literal() {
    let line = "tx.execute(\"SELECT 1\", ())?; // UPDATE work SET source_lang = ?1 trong comment";
    let literals = rust_string_literals(line);
    assert_eq!(
        literals,
        vec!["SELECT 1".to_owned()],
        "comment `//` cuoi dong phai bi cat -- chi chuoi THAT truoc no duoc giu lai: {literals:?}"
    );
}

/// 🔴 **Story 5.1** — `ScopeKind::Glossary` và `ScopeKind::TranslationMemory` đều phân giải
/// được ở `Tier::Work`, không chỉ khai NGỮ NGHĨA đúng trong bảng AD-18 (`kinds.rs`). Epic 5
/// §Requirements: *"Glossary và TM gắn ở tầng Tác phẩm"* — mệnh đề đó chưa từng có phép kiểm
/// trước story này (Approach của story tự khai điều đó).
///
/// Glossary khai [`auratranslate_lib::core::scope::Semantics::Override`]: một khoá trùng ở
/// cả hai tầng phải thắng về tầng Work. TranslationMemory khai
/// [`auratranslate_lib::core::scope::Semantics::Merge`]: cả hai tầng cùng áp, và mục đến từ
/// tầng Work phải mang đúng nhãn `Tier::Work` của chính nó (AD-19, không khử trùng lặp).
#[test]
fn glossary_and_translation_memory_both_resolve_at_the_work_tier() {
    let resolver = ScopeResolver::with_work(WorkScope { work_id: "work-under-test".to_owned() });

    // Glossary — Semantics::Override (AD-18): khoa trung o ca hai tang phai thang ve Work.
    let global_glossary: BTreeMap<&str, &str> = BTreeMap::from([("hero", "global-nghia")]);
    let work_glossary: BTreeMap<&str, &str> = BTreeMap::from([("hero", "work-nghia")]);
    let resolved = resolver
        .apply_override("glossary", &global_glossary, Some(&work_glossary))
        .expect("Glossary phai phan giai duoc voi mot tang Work khong rong");
    assert_eq!(
        resolved.get("hero").map(|r| r.tier()),
        Some(Tier::Work),
        "khoa trung o ca hai tang phai thang ve TANG WORK cho Glossary (Semantics::Override)"
    );
    assert_eq!(
        resolved.get("hero").map(|r| *r.value()),
        Some("work-nghia"),
        "gia tri thang phai la gia tri cua tang Work, khong phai tang Global bi che"
    );

    // TranslationMemory — Semantics::Merge (AD-18): ca hai tang cung ap, tang la khoa phu.
    let global_tm = vec!["cap global"];
    let work_tm = vec!["cap work"];
    let merged = resolver
        .apply_merge::<&str>("translation_memory", &global_tm, Some(&work_tm), None)
        .expect("TranslationMemory phai phan giai duoc voi mot tang Work khong rong");
    assert!(
        merged.iter().any(|t| t.tier() == Tier::Work && *t.value() == "cap work"),
        "ket qua hop nhat cua TranslationMemory phai co it nhat mot muc mang nhan Tier::Work: \
         {merged:?}"
    );
    assert!(
        merged.iter().any(|t| t.tier() == Tier::Global && *t.value() == "cap global"),
        "AD-19: giu nguyen bat dong -- muc Global khong bi khu trung lap hay bi tang Work che \
         mat: {merged:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.7 — mở lại một `.atproj` đã có trên đĩa (`open_work`) + danh sách/mở Chương.
// ═════════════════════════════════════════════════════════════════════════════════

/// Dựng một [`auratranslate_lib::core::library::indexer::IndexedWork`] khớp một [`OpenWork`]
/// đã tạo — cùng dữ kiện `library-index.db` sẽ chứa cho Tác phẩm đó (không cần mở
/// `Indexer` thật cho các ca ở tệp này; `library_index_contract.rs` sở hữu hành vi của
/// chính `Indexer::find_work`).
fn indexed_work_from(
    opened: &auratranslate_lib::commands::project::OpenWork,
) -> auratranslate_lib::core::library::indexer::IndexedWork {
    auratranslate_lib::core::library::indexer::IndexedWork {
        work_id: opened.meta.work_id.clone(),
        atproj_path: opened.dir.clone(),
        name: opened.meta.name.clone(),
        source_lang: opened.meta.source_lang.clone(),
        genre: opened.meta.genre.clone(),
        created_at: opened.meta.created_at.clone(),
        updated_at: opened.meta.updated_at.clone(),
        chapter_count: opened.meta.chapter_count,
        status: opened.meta.status.clone(),
        status_is_override: opened.meta.status_is_override,
        chapter_done_count: opened.meta.chapter_done_count,
    }
}

/// 🔴 **AC1** — mở lại một `.atproj` đã có trên đĩa: `ScopeResolver` phải là `with_work`
/// (KHÔNG `global_only`), và mọi mục Glossary tầng Tác phẩm ghi ở phiên trước đọc lại được
/// — nghiệm thu bằng một ca Rust, không bằng suy luận.
#[test]
fn opening_an_existing_atproj_resolves_with_work_scope_and_keeps_glossary_data() {
    let root = temp_dir("open-existing");
    let opened = create_work_from_text(&root, "Mo Lai", "zh", "", "Noi dung mau.".to_owned())
        .expect("tao tac pham that bai");
    let work_id = opened.meta.work_id.clone();

    // Ghi mot muc Glossary tang Tac pham TRUOC khi dong -- AC1 doi no doc lai duoc SAU khi
    // mo lai.
    opened
        .store
        .write(|tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO glossary_entry (source_term, translation, note, category, \
                 term_origin, created_at) VALUES ('Nhan vat A', 'Character A', '', 'person', \
                 'manual', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [],
            )?;
            Ok(())
        })
        .expect("ghi glossary_entry that bai");

    let indexed = indexed_work_from(&opened);
    let dir = opened.dir.clone();
    drop(opened); // dong Store truoc khi mo lai -- Windows tu choi mo tep dang mo hai lan.

    let reopened = auratranslate_lib::commands::project::open_work(&work_id, Some(&indexed))
        .expect("mo lai .atproj that bai");

    assert!(
        reopened.scope.has_work_tier(),
        "ScopeResolver cua mot Tac pham vua mo lai phai la with_work, khong global_only"
    );

    let glossary_count: i64 = reopened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM glossary_entry", [], |row| row.get(0)))
        .expect("doc glossary_entry that bai");
    assert_eq!(
        glossary_count, 1,
        "muc Glossary tang Tac pham ghi o phien truoc phai doc lai duoc sau khi mo lai"
    );

    drop(reopened);
    cleanup(&dir);
}

/// `work_id` lạ (không có trong chỉ mục) ⇒ `library.work_not_indexed` — một lỗi CÓ TÊN.
#[test]
fn opening_a_work_id_that_is_not_indexed_is_a_named_error() {
    let err = auratranslate_lib::commands::project::open_work("khong-ton-tai", None)
        .expect_err("work_id la phai la mot loi");

    assert_eq!(err.code(), "library.work_not_indexed");
    assert_eq!(err.message_key(), MessageKey::LibraryWorkNotIndexed);
    assert_eq!(err.params().get("work_id").map(String::as_str), Some("khong-ton-tai"));
    assert!(!err.retryable());
}

/// Thư mục `.atproj` đã biến mất khỏi đĩa (chỉ mục còn hàng, đĩa thì không) ⇒
/// `work.open_failed` — KHÔNG một lỗi kho chung chung.
#[test]
fn opening_a_work_whose_folder_has_vanished_is_a_named_open_failed_error() {
    let root = temp_dir("open-vanished");
    let opened = create_work_from_text(&root, "Bien Mat", "zh", "", "Noi dung.".to_owned())
        .expect("tao tac pham that bai");
    let indexed = indexed_work_from(&opened);
    let dir = opened.dir.clone();
    drop(opened);
    fs::remove_dir_all(&dir).expect("xoa thu muc .atproj that bai");

    let err = auratranslate_lib::commands::project::open_work(&indexed.work_id, Some(&indexed))
        .expect_err("thu muc bien mat phai la mot loi");

    assert_eq!(err.code(), "work.open_failed");
    assert_eq!(err.message_key(), MessageKey::WorkOpenFailed);
    assert_eq!(err.params().get("name").map(String::as_str), Some("Bien Mat"));

    cleanup(&root);
}

/// AC8 — `meta.json` phiên bản MỚI HƠN ⇒ `work.meta_too_new`, kèm `found`/`supported`, và
/// KHÔNG một byte nào trong `.atproj` bị ghi.
#[test]
fn opening_a_work_with_a_newer_meta_schema_is_refused_without_touching_a_single_byte() {
    let root = temp_dir("open-meta-new");
    let opened = create_work_from_text(&root, "Phien Ban Tuong Lai", "zh", "", "text".to_owned())
        .expect("tao tac pham that bai");
    let indexed = indexed_work_from(&opened);
    let dir = opened.dir.clone();
    drop(opened);

    let mut future_meta = WorkMeta::read(&dir).expect("doc meta.json that bai");
    future_meta.meta_schema_version = META_SCHEMA_VERSION + 1;
    future_meta.write_atomic(&dir).expect("ghi meta.json phien ban tuong lai that bai");
    let bytes_before = fs::read(dir.join("meta.json")).expect("doc meta.json that bai");

    let err = auratranslate_lib::commands::project::open_work(&indexed.work_id, Some(&indexed))
        .expect_err("meta.json phien ban moi hon phai bi tu choi");

    assert_eq!(err.code(), "work.meta_too_new");
    assert_eq!(err.message_key(), MessageKey::WorkMetaTooNew);
    assert_eq!(
        err.params().get("found").map(String::as_str),
        Some((META_SCHEMA_VERSION + 1).to_string().as_str())
    );
    assert_eq!(
        err.params().get("supported").map(String::as_str),
        Some(META_SCHEMA_VERSION.to_string().as_str())
    );

    let bytes_after = fs::read(dir.join("meta.json")).expect("doc meta.json that bai");
    assert_eq!(
        bytes_before, bytes_after,
        "mot lan mo bi tu choi khong duoc phep ghi mot byte nao vao .atproj"
    );

    cleanup(&root);
}

/// Mở lại **chính** Tác phẩm đang mở (`work_id` trùng) — hai lượt mở nối tiếp, đóng lượt
/// trước rồi mới mở lượt sau, đều phải thành công (§I/O Matrix "Mở lại chính Tác phẩm đang
/// mở").
#[test]
fn opening_the_same_work_twice_in_a_row_succeeds() {
    let root = temp_dir("open-twice");
    let opened = create_work_from_text(&root, "Mo Lai Hai Lan", "zh", "", "Noi dung.".to_owned())
        .expect("tao tac pham that bai");
    let indexed = indexed_work_from(&opened);
    let dir = opened.dir.clone();
    drop(opened);

    let first = auratranslate_lib::commands::project::open_work(&indexed.work_id, Some(&indexed))
        .expect("lan mo thu nhat that bai");
    drop(first);

    let second = auratranslate_lib::commands::project::open_work(&indexed.work_id, Some(&indexed))
        .expect("lan mo thu hai (lai chinh Tac pham do) that bai");
    assert!(second.scope.has_work_tier());

    drop(second);
    cleanup(&dir);
}

/// §I/O Matrix hàng *"`project.db` phiên bản MỚI HƠN"* — đo qua **CHÍNH đường `open_work`**,
/// không chỉ qua `Store::open`.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO CA NÀY TỒN TẠI DÙ CƠ CHẾ ĐÃ CÓ TEST
/// ─────────────────────────────────────────────────────────────────────────────
/// `segment_contract.rs::a_project_database_newer_than_the_app_is_refused_and_never_written_to`
/// đã canh **cơ chế** (AD-30 ở tầng `Store::open`), và nó xanh từ trước story này. Nhưng
/// `open_work` là một **chỗ nối MỚI** tới cơ chế đó — đường mở lại một `.atproj` chưa từng
/// tồn tại — và `AGENTS.md::Known pitfalls` ghi bằng chữ: *"một bộ test xanh KHÔNG chứng minh
/// chỗ nối mới được canh"*, kèm năm lần dính trong bảy ngày ở Epic 3. Một `open_work` nuốt
/// lỗi kho rồi trả một `IpcError` chung chung *(hoặc tệ hơn: rơi về một nhánh "mở được")* đi
/// qua **sạch** cả hai bộ ca kia.
///
/// ⚠️ Fixture bơm bước **18 GIẢ** lên chính `project.db` vừa tạo — cùng kỹ thuật và cùng lý do
/// đã ghi ở ca `segment_contract.rs` nói trên. Bước 18 phải là một số **chưa có thật**; khi
/// `PROJECT_MIGRATIONS` mọc thêm một bước, ca này (và ca kia) phải nâng theo, nếu không nó
/// **xanh mà không bao giờ chạm nhánh AD-30**.
#[test]
fn opening_a_work_whose_project_db_is_newer_than_the_app_is_refused_through_open_work() {
    use auratranslate_lib::core::store::{Migration, PROJECT_MIGRATIONS};

    let root = temp_dir("open-db-too-new");
    let opened = create_work_from_text(&root, "Kho Tuong Lai", "zh", "", "Noi dung.".to_owned())
        .expect("tao tac pham that bai");
    let indexed = indexed_work_from(&opened);
    let dir = opened.dir.clone();
    let db = dir.join("project.db");
    drop(opened);

    // Mot "ban ung dung tuong lai" chay them dung MOT buoc len `project.db` da co.
    //
    // 🔴 `Box::leak` chu KHONG mot mang `static` chep tay, va day la mot lua chon CO LY DO.
    // `StoreSpec::migrations` doi `&'static [Migration]`. Ca tuong duong o
    // `segment_contract.rs` giai bang mot mang `static [Migration; N]` liet ke tung phan tu —
    // va doc-comment cua chinh ca do dem duoc **muoi** luot phai nang tay khi
    // `PROJECT_MIGRATIONS` moc them mot buoc, moi luot quen la mot ca "xanh ma khong bao gio
    // cham nhanh AD-30". Dung so THAT cua `PROJECT_MIGRATIONS` roi noi them dung mot buoc
    // gia lam ca nay TU nang theo. Ro ri mot lan trong mot tien trinh test la mien phi.
    let mut future_steps: Vec<Migration> = PROJECT_MIGRATIONS.to_vec();
    let future_version = PROJECT_MIGRATIONS
        .last()
        .expect("PROJECT_MIGRATIONS khong duoc rong")
        .to_version
        + 1;
    future_steps.push(Migration {
        to_version: future_version,
        sql: "CREATE TABLE tu_tuong_lai (id INTEGER PRIMARY KEY);",
    });
    let future_steps: &'static [Migration] = Box::leak(future_steps.into_boxed_slice());
    let future = Store::open(StoreSpec {
        migrations: future_steps,
        ..StoreSpec::project(db.clone())
    })
    .expect("dung fixture o phien ban tuong lai that bai");
    assert_eq!(future.schema_version(), future_version);
    drop(future);

    let bytes_before = fs::metadata(&db).expect("doc metadata truoc").len();

    let err = auratranslate_lib::commands::project::open_work(&indexed.work_id, Some(&indexed))
        .expect_err("mot `project.db` phien ban moi hon PHAI bi tu choi qua `open_work`");

    assert_eq!(
        err.message_key(),
        MessageKey::StoreSchemaTooNew,
        "phep tu choi phai PHAN BIET DUOC o tang lenh — khong duoc nuot thanh mot loi mo kho \
         chung chung, va khong duoc doi lop thanh `work.open_failed`"
    );
    assert_eq!(err.code(), "store.schema_too_new");

    assert_eq!(
        fs::metadata(&db).expect("doc metadata sau").len(),
        bytes_before,
        "mot lan mo bi tu choi KHONG duoc dung toi mot byte nao cua `project.db` (AD-30)"
    );

    cleanup(&dir);
}

/// 🔴 **`meta.json` LÀNH nhưng `project.db` VẮNG ⇒ TỪ CHỐI, và KHÔNG tạo một tệp nào.**
///
/// Ca này ra đời ở lượt review 2026-08-29. `Store::open` đi qua `pragmas::open_connection`,
/// hàm mang `SQLITE_OPEN_CREATE` — nên trước bản vá, `open_work` trên một `.atproj` mất
/// `project.db` **âm thầm tạo** một kho rỗng ngay trong thư mục của người dùng, chạy trọn
/// bộ di trú lên nó, rồi mới trượt bằng một lỗi kho chung chung. Hai phép khẳng định dưới
/// đây khoá **cả hai** vế: câu báo đúng LOẠI, và **đĩa không bị chạm**.
#[test]
fn opening_a_work_whose_project_db_is_missing_is_refused_and_creates_no_file() {
    let root = temp_dir("open-db-missing");
    let opened = create_work_from_text(&root, "Mat Kho", "zh", "", "Noi dung.".to_owned())
        .expect("tao tac pham that bai");
    let indexed = indexed_work_from(&opened);
    let dir = opened.dir.clone();
    drop(opened);

    let db = dir.join("project.db");
    // Xoa ca ba tep cua kho -- `project.db` cong hai sidecar WAL/SHM neu con.
    fs::remove_file(&db).expect("xoa project.db that bai");
    let _ = fs::remove_file(dir.join("project.db-wal"));
    let _ = fs::remove_file(dir.join("project.db-shm"));
    assert!(!db.exists(), "tien de cua ca nay: project.db phai vang mat");

    let err = auratranslate_lib::commands::project::open_work(&indexed.work_id, Some(&indexed))
        .expect_err("mot .atproj mat project.db PHAI bi tu choi mo");

    assert_eq!(err.code(), "work.open_failed");
    assert_eq!(err.message_key(), MessageKey::WorkOpenFailed);

    // 🔴 Ve NANG hon cua ca nay: mot lan tu choi KHONG duoc de lai mot tep nao tren dia.
    assert!(
        !db.exists(),
        "`open_work` da TAO mot `project.db` rong o mot nhanh le ra chi duoc TU CHOI -- \
         doc-comment cua chinh ham do noi \"chi duoc phep TU CHOI MO\""
    );
    assert!(
        WorkMeta::read(&dir).is_ok(),
        "`meta.json` cua nguoi dung phai con nguyen sau mot lan mo bi tu choi"
    );

    cleanup(&dir);
}

/// 🔴 **`project.db` lành nhưng KHÔNG hàng `chapter` nào ⇒ một lỗi CÓ TÊN, không phải lỗi kho.**
///
/// Cùng lớp *"một câu SAI VỀ LOẠI"* mà Story 2.11 đã sửa ở `chapter_not_found`: không tệp nào
/// hỏng, nên *"khong mo duoc kho du lieu"* là một câu nói dối về nguyên nhân.
#[test]
fn opening_a_work_with_no_chapter_rows_is_a_named_error_not_a_store_error() {
    let root = temp_dir("open-no-chapters");
    let opened = create_work_from_text(&root, "Khong Chuong", "zh", "", "Noi dung.".to_owned())
        .expect("tao tac pham that bai");
    let indexed = indexed_work_from(&opened);
    let dir = opened.dir.clone();
    // Xoa het hang `chapter` NGAY TREN kho dang mo, roi tha no de dong tep.
    opened
        .store
        .write(|tx: &Transaction<'_>| {
            tx.execute("DELETE FROM chapter", [])?;
            Ok(())
        })
        .expect("xoa hang chapter that bai");
    drop(opened);

    let err = auratranslate_lib::commands::project::open_work(&indexed.work_id, Some(&indexed))
        .expect_err("mot project.db khong hang chapter nao PHAI bi tu choi mo");

    assert_eq!(
        err.message_key(),
        MessageKey::WorkOpenFailed,
        "phai la mot loi TANG TAC PHAM co ten, khong phai `store.read_failed`"
    );
    assert_eq!(err.code(), "work.open_failed");

    cleanup(&dir);
}

/// AC2 — `list_chapters` trả đúng thứ tự `(ord, id)` và KHÔNG mang `source_text`.
#[test]
fn list_chapters_returns_rows_in_ord_id_order_without_source_text() {
    let root = temp_dir("list-chapters-order");
    let opened = create_work_from_text(&root, "Liet Ke Chuong", "zh", "", "Chuong mot.".to_owned())
        .expect("tao tac pham that bai");
    let second = insert_chapter_directly(&opened, 5, "Chuong nam.");
    let third = insert_chapter_directly(&opened, 2, "Chuong hai.");

    let rows = auratranslate_lib::commands::chapter::list_chapters(Some(&opened))
        .expect("liet ke Chuong that bai");

    let ids: Vec<i64> = rows.iter().map(|r| r.chapter_id).collect();
    assert_eq!(
        ids,
        vec![opened.chapter_id, third, second],
        "thu tu phai la (ord, id) tang dan: ord=1 (Chuong mo dau), ord=2 (third), ord=5 (second)"
    );
    assert_eq!(rows.len(), 3);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// Chưa Tác phẩm nào mở ⇒ `list_chapters` trả `project.no_work_open` — cùng khoá tái dùng.
#[test]
fn list_chapters_without_a_work_open_reuses_the_named_error() {
    let err = auratranslate_lib::commands::chapter::list_chapters(None)
        .expect_err("chua Tac pham nao mo phai la mot loi");
    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
}

/// `title IS NULL` đi qua dây nguyên vẹn — webview dựng nhãn từ `ord`, Rust không tự đoán
/// một chuỗi thay thế.
#[test]
fn list_chapters_carries_a_null_title_through_untouched() {
    let root = temp_dir("list-chapters-untitled");
    let opened = create_work_from_text(&root, "Chuong Khong Ten", "zh", "", "Chuong mot.".to_owned())
        .expect("tao tac pham that bai");

    let rows = auratranslate_lib::commands::chapter::list_chapters(Some(&opened))
        .expect("liet ke Chuong that bai");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, None, "Chuong moi tao chua co tieu de, phai la None qua day");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// AC3 — mở một Chương đích danh dời con trỏ **SAU** khi truy vấn thành công.
#[test]
fn opening_a_named_chapter_moves_the_cursor_only_after_a_successful_lookup() {
    let root = temp_dir("open-chapter-named");
    let mut opened = create_work_from_text(&root, "Mo Chuong Dich Danh", "zh", "", "Chuong mot.".to_owned())
        .expect("tao tac pham that bai");
    let second = insert_chapter_directly(&opened, 2, "Chuong hai.");

    let switched = auratranslate_lib::commands::chapter::open_chapter(Some(&mut opened), second)
        .expect("mo Chuong dich danh that bai");
    assert_eq!(switched.chapter_id, second);
    assert_eq!(opened.chapter_id, second, "con tro Chuong phai doi sang dung Chuong vua mo");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// `chapter_id` lạ ⇒ `segment.chapter_not_found`, con trỏ Chương **không đổi**.
#[test]
fn opening_an_unknown_chapter_id_does_not_move_the_cursor() {
    let root = temp_dir("open-chapter-unknown");
    let mut opened = create_work_from_text(&root, "Chuong La", "zh", "", "Chuong mot.".to_owned())
        .expect("tao tac pham that bai");
    let original = opened.chapter_id;

    let err = auratranslate_lib::commands::chapter::open_chapter(Some(&mut opened), original + 999)
        .expect_err("chapter_id la phai la mot loi");
    assert_eq!(err.code(), "segment.chapter_not_found");
    assert_eq!(err.message_key(), MessageKey::SegmentChapterNotFound);
    assert_eq!(
        opened.chapter_id, original,
        "con tro Chuong KHONG duoc doi khi truy van that bai"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// Chưa Tác phẩm nào mở ⇒ `open_chapter` trả `project.no_work_open`.
#[test]
fn opening_a_chapter_without_a_work_open_reuses_the_named_error() {
    let err = auratranslate_lib::commands::chapter::open_chapter(None, 1)
        .expect_err("chua Tac pham nao mo phai la mot loi");
    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.8 — TỔ CHỨC LẠI CHƯƠNG SAU KHI NHẬP (FR15, AD-32)
// ═════════════════════════════════════════════════════════════════════════════════
// `create_work_from_text` TỰ ĐỘNG tách segment từ văn bản đưa vào (AC13, Story 2.1) — mọi ca
// dưới đây cần kiểm soát TUYỆT ĐỐI tập hàng `segment`/`chapter` nên gọi nó với văn bản RỖNG
// (0 segment auto-sinh) rồi tự dựng dữ liệu bằng SQL trần, đúng khuôn `insert_chapter_directly`
// đã có ở trên.

/// Chèn một hàng `segment` **thẳng bằng SQL**, mọi cột không liệt ở tham số mang giá trị mặc
/// định vô hại (`target_text = ''`, `status = 'draft'`, `is_omitted = 0`,
/// `translation_origin = ''`) — trả `segment.id` vừa sinh.
fn insert_segment_directly(opened: &OpenWork, chapter_id: i64, ord: i64, source_text: &str, retired: bool) -> i64 {
    let source_text = source_text.to_owned();
    let retired_at: Option<&'static str> = if retired { Some("2026-08-29T00:00:00.000Z") } else { None };
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, target_text, \
                 is_paragraph_end, is_target_paragraph_end, retired_at, status, is_omitted, \
                 translation_origin, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, '', 0, 0, ?4, 'draft', 0, '', \
                 '2026-08-29T00:00:00.000Z', '2026-08-29T00:00:00.000Z')",
                (chapter_id, ord, &source_text, retired_at),
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("chen segment bang SQL truc tiep that bai")
}

/// Chèn một hàng `segment_version` **thẳng bằng SQL** — trả `id` vừa sinh.
fn insert_segment_version_directly(opened: &OpenWork, segment_id: i64, target_text: &str) -> i64 {
    let target_text = target_text.to_owned();
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO segment_version (segment_id, target_text, created_at) \
                 VALUES (?1, ?2, '2026-08-29T00:00:00.000Z')",
                (segment_id, &target_text),
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("chen segment_version bang SQL truc tiep that bai")
}

fn set_chapter_status_directly(opened: &OpenWork, chapter_id: i64, status: &str) {
    let status = status.to_owned();
    opened
        .store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE chapter SET status = ?1 WHERE id = ?2", (status, chapter_id)))
        .expect("dat status Chuong bang SQL truc tiep that bai");
}

fn set_chapter_source_text_directly(opened: &OpenWork, chapter_id: i64, text: &str) {
    let text = text.to_owned();
    opened
        .store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE chapter SET source_text = ?1 WHERE id = ?2", (text, chapter_id)))
        .expect("dat source_text Chuong bang SQL truc tiep that bai");
}

fn read_chapter_status(opened: &OpenWork, chapter_id: i64) -> String {
    opened
        .store
        .read(move |conn| conn.query_row("SELECT status FROM chapter WHERE id = ?1", [chapter_id], |row| row.get(0)))
        .expect("doc status Chuong that bai")
}

fn read_chapter_source_text(opened: &OpenWork, chapter_id: i64) -> String {
    opened
        .store
        .read(move |conn| conn.query_row("SELECT source_text FROM chapter WHERE id = ?1", [chapter_id], |row| row.get(0)))
        .expect("doc source_text Chuong that bai")
}

fn read_chapter_title(opened: &OpenWork, chapter_id: i64) -> Option<String> {
    opened
        .store
        .read(move |conn| conn.query_row("SELECT title FROM chapter WHERE id = ?1", [chapter_id], |row| row.get(0)))
        .expect("doc title Chuong that bai")
}

/// `(chapter.id, chapter.ord)` của MỌI Chương, sắp theo `id` — để so sánh "trước/sau" của một
/// lượt bị TỪ CHỐI (§Always: "một lượt ghi không đi đâu phải nói vì sao ... không một hàng
/// nào bị chạm").
fn read_all_chapter_ord(opened: &OpenWork) -> Vec<(i64, i64)> {
    opened
        .store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id, ord FROM chapter ORDER BY id")?;
            let mapped = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
            mapped.collect::<SqlResult<Vec<(i64, i64)>>>()
        })
        .expect("doc (id, ord) cua moi Chuong that bai")
}

/// Ảnh chụp TOÀN BỘ một hàng `segment` — mọi cột (AD-32 là mệnh đề nghiệm thu chính: gộp/tách
/// đổi ĐÚNG HAI cột, `chapter_id` và `ord`; mọi cột khác giữ nguyên từng byte).
#[derive(Debug, Clone, PartialEq, Eq)]
struct SegmentSnapshot {
    id: i64,
    chapter_id: i64,
    ord: i64,
    source_text: String,
    target_text: String,
    is_paragraph_end: i64,
    is_target_paragraph_end: i64,
    retired_at: Option<String>,
    status: String,
    is_omitted: i64,
    translation_origin: String,
    created_at: String,
    updated_at: String,
}

fn snapshot_segments(opened: &OpenWork) -> Vec<SegmentSnapshot> {
    opened
        .store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, chapter_id, ord, source_text, target_text, is_paragraph_end, \
                 is_target_paragraph_end, retired_at, status, is_omitted, translation_origin, \
                 created_at, updated_at FROM segment ORDER BY id",
            )?;
            let mapped = stmt.query_map([], |row| {
                Ok(SegmentSnapshot {
                    id: row.get(0)?,
                    chapter_id: row.get(1)?,
                    ord: row.get(2)?,
                    source_text: row.get(3)?,
                    target_text: row.get(4)?,
                    is_paragraph_end: row.get(5)?,
                    is_target_paragraph_end: row.get(6)?,
                    retired_at: row.get(7)?,
                    status: row.get(8)?,
                    is_omitted: row.get(9)?,
                    translation_origin: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })?;
            mapped.collect::<SqlResult<Vec<SegmentSnapshot>>>()
        })
        .expect("chup anh toan bo hang segment that bai")
}

/// Ảnh chụp TOÀN BỘ một hàng `segment_version` — mọi cột.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SegmentVersionSnapshot {
    id: i64,
    segment_id: i64,
    target_text: String,
    created_at: String,
}

fn snapshot_segment_versions(opened: &OpenWork) -> Vec<SegmentVersionSnapshot> {
    opened
        .store
        .read(|conn| {
            let mut stmt =
                conn.prepare("SELECT id, segment_id, target_text, created_at FROM segment_version ORDER BY id")?;
            let mapped = stmt.query_map([], |row| {
                Ok(SegmentVersionSnapshot {
                    id: row.get(0)?,
                    segment_id: row.get(1)?,
                    target_text: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?;
            mapped.collect::<SqlResult<Vec<SegmentVersionSnapshot>>>()
        })
        .expect("chup anh toan bo hang segment_version that bai")
}

/// 🔴 **AD-32 là mệnh đề nghiệm thu chính của cả story — GỘP.** Chụp TOÀN BỘ mọi cột của mọi
/// hàng `segment` cộng mọi hàng `segment_version` TRƯỚC và SAU, rồi khẳng định đúng hai cột
/// đổi (`chapter_id`, `ord`) trên đúng những hàng đã dời — không một cột nào khác, không một
/// hàng `segment_version` nào bị đụng.
#[test]
fn merging_two_chapters_changes_only_chapter_id_and_ord_on_every_segment_column() {
    let root = temp_dir("merge-full-columns");
    let mut opened = create_work_from_text(&root, "Gop Chuong", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;

    // A: ba câu -- ord 1 (sống), ord 2 (VỀ HƯU), ord 3 (sống). Một lịch sử phiên bản gắn
    // vào câu 1 -- phải sống sót nguyên vẹn qua lượt gộp.
    let a1 = insert_segment_directly(&opened, a_id, 1, "A cau mot.", false);
    let a2 = insert_segment_directly(&opened, a_id, 2, "A cau hai da ve huu.", true);
    let a3 = insert_segment_directly(&opened, a_id, 3, "A cau ba.", false);
    insert_segment_version_directly(&opened, a1, "ban dich cu cua A1");

    // B: hai câu, liền ngay sau A.
    let b_id = insert_chapter_directly(&opened, 2, "");
    let b1 = insert_segment_directly(&opened, b_id, 1, "B cau mot.", false);
    let b2 = insert_segment_directly(&opened, b_id, 2, "B cau hai.", false);

    let before = snapshot_segments(&opened);
    let versions_before = snapshot_segment_versions(&opened);
    assert_eq!(before.len(), 5, "fixture phai co dung 5 hang segment");

    merge_chapter_into_previous(Some(&mut opened), b_id).expect("gop that bai");

    let after = snapshot_segments(&opened);
    let versions_after = snapshot_segment_versions(&opened);

    assert_eq!(versions_before, versions_after, "khong hang segment_version nao duoc dung toi boi luot gop");
    assert_eq!(after.len(), 5, "gop KHONG tao/huy mot hang segment nao");

    for (b, a) in before.iter().zip(after.iter()) {
        assert_eq!(b.id, a.id, "thu tu doc theo id phai giu nguyen");
        assert_eq!(b.source_text, a.source_text, "id={}", a.id);
        assert_eq!(b.target_text, a.target_text, "id={}", a.id);
        assert_eq!(b.is_paragraph_end, a.is_paragraph_end, "id={}", a.id);
        assert_eq!(b.is_target_paragraph_end, a.is_target_paragraph_end, "id={}", a.id);
        assert_eq!(b.retired_at, a.retired_at, "id={}", a.id);
        assert_eq!(b.status, a.status, "id={}", a.id);
        assert_eq!(b.is_omitted, a.is_omitted, "id={}", a.id);
        assert_eq!(b.translation_origin, a.translation_origin, "id={}", a.id);
        assert_eq!(b.created_at, a.created_at, "id={}", a.id);
        assert_eq!(b.updated_at, a.updated_at, "id={}", a.id);
    }

    let by_id = |id: i64| after.iter().find(|s| s.id == id).unwrap();
    assert_eq!(by_id(a1).chapter_id, a_id);
    assert_eq!(by_id(a1).ord, 1);
    assert_eq!(by_id(a2).chapter_id, a_id);
    assert_eq!(by_id(a2).ord, 2);
    assert_eq!(by_id(a3).chapter_id, a_id);
    assert_eq!(by_id(a3).ord, 3);
    assert_eq!(by_id(b1).chapter_id, a_id, "hang cua B phai doi chapter_id sang A");
    assert_eq!(by_id(b1).ord, 4, "ord cua B tinh tien bang MAX(ord) cu cua A (3)");
    assert_eq!(by_id(b2).chapter_id, a_id);
    assert_eq!(by_id(b2).ord, 5);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// Bản song sinh của test trên, cho lượt **TÁCH** — cùng phương pháp: chụp toàn bộ cột trước
/// và sau, khẳng định đúng hai cột đổi. Cũng nghiệm thu AC "Chương mới mang cùng `segment.id`,
/// `ord` là `1, 2`, lịch sử phiên bản của câu bị tách còn nguyên và vẫn tra được, không câu
/// nào mang `retired_at`", và §Design Notes "status/title của B chép từ A (title thành NULL)".
#[test]
fn splitting_a_chapter_changes_only_chapter_id_and_ord_on_every_segment_column() {
    let root = temp_dir("split-full-columns");
    let mut opened = create_work_from_text(&root, "Tach Chuong", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    set_chapter_status_directly(&opened, a_id, "in_progress");

    let s1 = insert_segment_directly(&opened, a_id, 1, "Cau mot.", false);
    let s2 = insert_segment_directly(&opened, a_id, 2, "Cau hai da ve huu.", true);
    let s3 = insert_segment_directly(&opened, a_id, 3, "Cau ba.", false);
    let s4 = insert_segment_directly(&opened, a_id, 4, "Cau bon.", false);
    insert_segment_version_directly(&opened, s3, "ban dich cu cua cau ba");

    let before = snapshot_segments(&opened);
    let versions_before = snapshot_segment_versions(&opened);
    assert_eq!(before.len(), 4);

    split_chapter_at_segment(Some(&mut opened), s3).expect("tach that bai");

    let after = snapshot_segments(&opened);
    let versions_after = snapshot_segment_versions(&opened);
    assert_eq!(versions_before, versions_after, "lich su phien ban cua cau ba song sot nguyen ven va van tra duoc");
    assert_eq!(after.len(), 4, "tach KHONG tao/huy mot hang segment nao");

    for (b, a) in before.iter().zip(after.iter()) {
        assert_eq!(b.id, a.id);
        assert_eq!(b.source_text, a.source_text, "id={}", a.id);
        assert_eq!(b.target_text, a.target_text, "id={}", a.id);
        assert_eq!(b.is_paragraph_end, a.is_paragraph_end, "id={}", a.id);
        assert_eq!(b.is_target_paragraph_end, a.is_target_paragraph_end, "id={}", a.id);
        assert_eq!(b.retired_at, a.retired_at, "khong cau nao duoc gan retired_at boi luot tach -- id={}", a.id);
        assert_eq!(b.status, a.status, "id={}", a.id);
        assert_eq!(b.is_omitted, a.is_omitted, "id={}", a.id);
        assert_eq!(b.translation_origin, a.translation_origin, "id={}", a.id);
        assert_eq!(b.created_at, a.created_at, "id={}", a.id);
        assert_eq!(b.updated_at, a.updated_at, "id={}", a.id);
    }

    let by_id = |id: i64| after.iter().find(|s| s.id == id).unwrap();
    assert_eq!(by_id(s1).chapter_id, a_id);
    assert_eq!(by_id(s1).ord, 1);
    assert_eq!(by_id(s2).chapter_id, a_id, "cau da ve huu dung TRUOC diem cat -- o lai A");
    assert_eq!(by_id(s2).ord, 2);

    let b_id = by_id(s3).chapter_id;
    assert_ne!(b_id, a_id, "s3 la diem cat -- phai doi sang mot Chuong MOI");
    assert_eq!(by_id(s3).ord, 1, "cung segment.id, ord danh lai tu 1 trong Chuong moi");
    assert_eq!(by_id(s4).chapter_id, b_id);
    assert_eq!(by_id(s4).ord, 2);

    assert_eq!(opened.chapter_id, a_id, "con tro Chuong dang mo giu nguyen A sau lot tach");
    assert_eq!(opened.meta.chapter_count, 2, "mot Chuong moi da duoc chen -- chapter_count phai theo kip");
    assert_eq!(read_chapter_status(&opened, b_id), "in_progress", "status cua B chep tu A");
    assert_eq!(read_chapter_title(&opened, b_id), None, "title cua B luon NULL, khong chep tu A");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// Không đường TỔ CHỨC nào được phép cho một segment về hưu — kiểm cả hai chiều (gộp rồi
/// tách) trên cùng một fixture, đếm `retired_at IS NOT NULL` trước/sau mỗi lượt.
#[test]
fn no_segment_is_ever_retired_by_a_chapter_reorganisation() {
    fn count_retired(opened: &OpenWork) -> i64 {
        opened
            .store
            .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment WHERE retired_at IS NOT NULL", [], |row| row.get(0)))
            .expect("dem segment ve huu that bai")
    }

    let root = temp_dir("no-new-retirement");
    let mut opened = create_work_from_text(&root, "Khong Ve Huu", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    let s1 = insert_segment_directly(&opened, a_id, 1, "Cau mot.", false);
    let s2 = insert_segment_directly(&opened, a_id, 2, "Cau hai.", false);
    let b_id = insert_chapter_directly(&opened, 2, "");
    insert_segment_directly(&opened, b_id, 1, "Cau ba.", false);

    assert_eq!(count_retired(&opened), 0);
    merge_chapter_into_previous(Some(&mut opened), b_id).expect("gop that bai");
    assert_eq!(count_retired(&opened), 0, "gop KHONG duoc cho segment nao ve huu");

    split_chapter_at_segment(Some(&mut opened), s2).expect("tach that bai");
    assert_eq!(count_retired(&opened), 0, "tach KHONG duoc cho segment nao ve huu");

    let _ = s1;
    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §Design Notes "Vì sao gộp `done` + chưa xong ra `in_progress`" — hai ca trong một fixture:
/// một nửa chưa xong hạ xuống `in_progress`; cả hai nửa `done` giữ `done`.
#[test]
fn a_merged_chapter_never_claims_done_when_one_half_was_not_done() {
    // Ca 1 -- A done, B chua xong ⇒ ha xuong in_progress.
    let root = temp_dir("merge-status-half-done");
    let mut opened = create_work_from_text(&root, "Trang Thai Gop Nua", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    set_chapter_status_directly(&opened, a_id, "done");
    let b_id = insert_chapter_directly(&opened, 2, "");
    // B mac dinh 'not_started' tu `insert_chapter_directly`.

    merge_chapter_into_previous(Some(&mut opened), b_id).expect("gop that bai");
    assert_eq!(
        read_chapter_status(&opened, a_id),
        "in_progress",
        "done + chua xong phai ha xuong in_progress -- khong duoc khai done cho van ban chua ai xac nhan"
    );
    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);

    // Ca 2 -- CA HAI done ⇒ giu done.
    let root2 = temp_dir("merge-status-both-done");
    let mut opened2 = create_work_from_text(&root2, "Trang Thai Gop Ca Hai", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a2_id = opened2.chapter_id;
    set_chapter_status_directly(&opened2, a2_id, "done");
    let b2_id = insert_chapter_directly(&opened2, 2, "");
    set_chapter_status_directly(&opened2, b2_id, "done");

    merge_chapter_into_previous(Some(&mut opened2), b2_id).expect("gop that bai");
    assert_eq!(
        read_chapter_status(&opened2, a2_id),
        "done",
        "ca hai nua done thi Chuong gop phai giu done"
    );
    let dir2 = opened2.dir.clone();
    drop(opened2);
    cleanup(&dir2);
}

/// §Always "Một lượt ghi không đi đâu phải NÓI VÌ SAO" — dời LÊN ở Chương ĐẦU.
#[test]
fn moving_the_first_chapter_up_touches_no_row_and_is_a_named_error() {
    let root = temp_dir("move-up-at-first");
    let mut opened = create_work_from_text(&root, "Doi Bien Len", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    insert_chapter_directly(&opened, 2, "");

    let before = read_all_chapter_ord(&opened);
    let err = move_chapter(Some(&mut opened), a_id, ChapterDirection::Prev)
        .expect_err("doi len o Chuong dau phai la mot loi CO TEN");
    assert_eq!(err.code(), "chapter.at_first");
    assert_eq!(err.message_key(), MessageKey::ChapterAtFirst);
    assert_eq!(read_all_chapter_ord(&opened), before, "0 hang bi cham");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// Bản song sinh, biên CUỐI: dời XUỐNG ở Chương cuối.
#[test]
fn moving_the_last_chapter_down_touches_no_row_and_is_a_named_error() {
    let root = temp_dir("move-down-at-last");
    let mut opened = create_work_from_text(&root, "Doi Bien Xuong", "zh", "", String::new())
        .expect("tao tac pham that bai");
    insert_chapter_directly(&opened, 2, "");
    let c_id = insert_chapter_directly(&opened, 3, "");

    let before = read_all_chapter_ord(&opened);
    let err = move_chapter(Some(&mut opened), c_id, ChapterDirection::Next)
        .expect_err("doi xuong o Chuong cuoi phai la mot loi CO TEN");
    assert_eq!(err.code(), "chapter.at_last");
    assert_eq!(err.message_key(), MessageKey::ChapterAtLast);
    assert_eq!(read_all_chapter_ord(&opened), before, "0 hang bi cham");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// Bản song sinh thứ hai: GỘP ở Chương đầu (không có Chương liền trước).
#[test]
fn merging_the_first_chapter_touches_no_row_and_is_a_named_error() {
    let root = temp_dir("merge-at-first");
    let mut opened = create_work_from_text(&root, "Gop O Dau", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id; // Chuong duy nhat -- khong co Chuong lien truoc.
    insert_segment_directly(&opened, a_id, 1, "Cau mot.", false);

    let chapters_before = read_all_chapter_ord(&opened);
    let segments_before = snapshot_segments(&opened);

    let err = merge_chapter_into_previous(Some(&mut opened), a_id)
        .expect_err("gop o Chuong dau phai la mot loi CO TEN");
    assert_eq!(err.code(), "chapter.at_first");
    assert_eq!(err.message_key(), MessageKey::ChapterAtFirst);

    assert_eq!(read_all_chapter_ord(&opened), chapters_before, "0 hang chapter bi cham");
    assert_eq!(snapshot_segments(&opened), segments_before, "0 hang segment bi cham");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Tách tại câu đầu" — một Chương rỗng không phải một kết quả có nghĩa.
#[test]
fn splitting_at_the_first_segment_is_refused_before_any_write() {
    let root = temp_dir("split-at-first-segment");
    let mut opened = create_work_from_text(&root, "Tach O Dau", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    let s1 = insert_segment_directly(&opened, a_id, 1, "Cau mot.", false);
    insert_segment_directly(&opened, a_id, 2, "Cau hai.", false);

    let segments_before = snapshot_segments(&opened);
    let chapters_before = read_all_chapter_ord(&opened);

    let err = split_chapter_at_segment(Some(&mut opened), s1)
        .expect_err("tach tai cau DAU phai la mot loi CO TEN -- Chuong con lai se RONG");
    assert_eq!(err.code(), "chapter.split_leaves_empty");
    assert_eq!(err.message_key(), MessageKey::ChapterSplitLeavesEmpty);

    assert_eq!(snapshot_segments(&opened), segments_before, "0 hang segment bi cham");
    assert_eq!(read_all_chapter_ord(&opened), chapters_before, "0 hang chapter bi cham -- khong Chuong moi nao duoc chen");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// `str::trim()` của RUST, không `trim()` của SQLite — một tên chỉ gồm U+3000 (khoảng trắng
/// toàn chiều rộng) phải lưu thành `NULL`, không một chuỗi trông "có chữ" mà hiện ra trống.
#[test]
fn renaming_to_an_ideographic_space_stores_null_not_a_blank_title() {
    let root = temp_dir("rename-ideographic-space");
    let mut opened = create_work_from_text(&root, "Doi Ten Rong", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;

    let rows = rename_chapter(Some(&mut opened), a_id, "\u{3000}").expect("doi ten that bai");
    let row = rows.iter().find(|r| r.chapter_id == a_id).expect("phai co hang cho Chuong vua doi ten");
    assert_eq!(row.title, None, "mot ten chi gom U+3000 phai luu thanh NULL, khong mot chuoi trong");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// AC1 — đổi tên đổi ĐÚNG một hàng `chapter`, `updated_at` mới, và **0 hàng `segment`/
/// `segment_version`** nào bị đụng.
#[test]
fn renaming_a_chapter_updates_only_its_own_row_and_leaves_every_segment_untouched() {
    let root = temp_dir("rename-touches-nothing-else");
    let mut opened = create_work_from_text(&root, "Doi Ten Sach", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    let s1 = insert_segment_directly(&opened, a_id, 1, "Cau mot.", false);
    insert_segment_version_directly(&opened, s1, "ban dich cu");

    let segments_before = snapshot_segments(&opened);
    let versions_before = snapshot_segment_versions(&opened);

    let rows = rename_chapter(Some(&mut opened), a_id, "Hoi 1").expect("doi ten that bai");
    let row = rows.iter().find(|r| r.chapter_id == a_id).expect("phai co hang cho Chuong vua doi ten");
    assert_eq!(row.title.as_deref(), Some("Hoi 1"));

    assert_eq!(snapshot_segments(&opened), segments_before, "doi ten KHONG duoc cham mot hang segment nao");
    assert_eq!(snapshot_segment_versions(&opened), versions_before, "doi ten KHONG duoc cham mot hang segment_version nao");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// `chapter_id` lạ ⇒ `segment.chapter_not_found`, đúng khoá đã có — Task 6.
#[test]
fn renaming_an_unknown_chapter_id_reuses_the_named_error() {
    let root = temp_dir("rename-unknown-chapter");
    let mut opened = create_work_from_text(&root, "Doi Ten Chuong La", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let original = opened.chapter_id;

    let err = rename_chapter(Some(&mut opened), original + 999, "Hoi Moi")
        .expect_err("chapter_id la phai la mot loi");
    assert_eq!(err.code(), "segment.chapter_not_found");
    assert_eq!(err.message_key(), MessageKey::SegmentChapterNotFound);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §Always "Chuẩn hoá `chapter.ord` về `1..N` ... TRƯỚC mỗi thao tác" — dựng một dãy `ord`
/// thưa/trùng `(5, 5, 9)` bằng `insert_chapter_directly`, dời Chương giữa lên, rồi khẳng định
/// `ord` của cả ba là `1, 2, 3` liên tục.
///
/// 🔵 **ĐỔI TÊN 2026-08-29 (lượt rà): `..._after_a_merge_...` → `..._after_a_move_...`.** Tên
/// cũ nói *"sau một lượt GỘP"* trong khi thân hàm chỉ gọi `move_chapter` — một cái tên khai
/// nhiều hơn thứ nó đo, tức đúng thứ làm người đọc sau tưởng đường gộp đã có lưới. Đường gộp
/// nay có ca riêng: [`chapter_ord_stays_dense_from_one_after_a_merge_on_a_sparse_ord_sequence_too`].
#[test]
fn chapter_ord_stays_dense_from_one_after_a_move_on_a_sparse_ord_sequence() {
    let root = temp_dir("move-sparse-ord");
    let opened_bootstrap = create_work_from_text(&root, "Ord Thua", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let default_id = opened_bootstrap.chapter_id;
    // Xoá Chương mặc định (0 segment -- văn bản rỗng) để dựng TRỌN ba Chương `ord` thưa/trùng
    // do chính story này chỉ ra: `(5, 5, 9)`.
    opened_bootstrap
        .store
        .write(move |tx: &Transaction<'_>| tx.execute("DELETE FROM chapter WHERE id = ?1", [default_id]))
        .expect("xoa Chuong mac dinh that bai");
    let mut opened = opened_bootstrap;

    let c1 = insert_chapter_directly(&opened, 5, "Chuong mot.");
    let c2 = insert_chapter_directly(&opened, 5, "Chuong hai.");
    let c3 = insert_chapter_directly(&opened, 9, "Chuong ba.");
    // Chuẩn hoá theo `(ord, id)`: c1 (ord=5, id nhỏ hơn) -> 1; c2 (ord=5, id lớn hơn) -> 2;
    // c3 (ord=9) -> 3. c2 là Chương GIỮA sau chuẩn hoá.
    opened.chapter_id = c2;

    move_chapter(Some(&mut opened), c2, ChapterDirection::Prev).expect("doi len that bai");

    let rows = read_all_chapter_ord(&opened);
    let mut ords: Vec<i64> = rows.iter().map(|(_, ord)| *ord).collect();
    ords.sort_unstable();
    assert_eq!(ords, vec![1, 2, 3], "ord phai lien tuc tu 1 sau chuan hoa, du dau vao thua/trung");

    let ord_of = |id: i64| rows.iter().find(|(rid, _)| *rid == id).unwrap().1;
    assert!(ord_of(c2) < ord_of(c1), "doi LEN phai hoan vi c2 len truoc c1");
    assert!(ord_of(c1) < ord_of(c3), "c3 khong bi dung toi");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// AD-5 "vẫn mở được về ĐÚNG VỊ TRÍ" áp cho lượt TỔ CHỨC CHƯƠNG: một hàng VỀ HƯU giữa hai hàng
/// SỐNG phải dời CÙNG khối, giữ đúng vị trí tương đối — không bị phép tịnh tiến bỏ lại hay xô
/// lệch.
#[test]
fn a_reorganisation_moves_retired_segments_with_their_living_neighbours() {
    let root = temp_dir("retired-moves-with-neighbours");
    let mut opened = create_work_from_text(&root, "Ve Huu Di Cung", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    let a1 = insert_segment_directly(&opened, a_id, 1, "A mot.", false);

    let b_id = insert_chapter_directly(&opened, 2, "");
    let b1 = insert_segment_directly(&opened, b_id, 1, "B mot.", false);
    let b2_retired = insert_segment_directly(&opened, b_id, 2, "B hai da ve huu.", true);
    let b3 = insert_segment_directly(&opened, b_id, 3, "B ba.", false);

    merge_chapter_into_previous(Some(&mut opened), b_id).expect("gop that bai");

    let after = snapshot_segments(&opened);
    let by_id = |id: i64| after.iter().find(|s| s.id == id).unwrap();

    // shift = MAX(ord) cua A truoc gop = 1 (chi co a1).
    assert_eq!(by_id(a1).ord, 1);
    assert_eq!(by_id(b1).chapter_id, a_id);
    assert_eq!(by_id(b1).ord, 2);
    assert_eq!(by_id(b2_retired).chapter_id, a_id, "hang VE HUU cung phai doi chapter_id");
    assert_eq!(by_id(b2_retired).ord, 3, "hang VE HUU giu dung vi tri TUONG DOI giua hai hang song ke no");
    assert!(by_id(b2_retired).retired_at.is_some(), "van con VE HUU -- gop khong hoi sinh no");
    assert_eq!(by_id(b3).chapter_id, a_id);
    assert_eq!(by_id(b3).ord, 4, "hang song SAU no van dung ke, khong bi phep tinh tien lam lech");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// AC3/AC4 — con trỏ `OpenWork::chapter_id` dời sang Chương SỐNG SÓT sau một lượt gộp Chương
/// đang mở, và một lượt `read_open_chapter` kế tiếp **không** trả `segment.chapter_not_found`.
#[test]
fn the_open_chapter_cursor_follows_the_surviving_chapter_after_a_merge() {
    let root = temp_dir("cursor-follows-merge");
    let mut opened = create_work_from_text(&root, "Con Tro Theo Gop", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    let b_id = insert_chapter_directly(&opened, 2, "");
    opened.chapter_id = b_id; // B ĐANG MỞ.

    merge_chapter_into_previous(Some(&mut opened), b_id).expect("gop that bai");

    assert_eq!(opened.chapter_id, a_id, "B bi xoa -- con tro phai doi sang A SAU khi giao dich commit");

    let reopened = read_open_chapter(Some(&opened)).expect("doc lai Chuong dang mo (A) sau gop phai thanh cong");
    assert_eq!(reopened.chapter_id, a_id);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §Design Notes "Vì sao `source_text` của lượt GỘP nối thô" — `A.source_text = A.source_text
/// ‖ "\n\n" ‖ B.source_text`, không mất một byte nào kể cả khi cả hai Chương chưa từng tách
/// segment (25 Chương Epic 1).
#[test]
fn merging_concatenates_raw_source_text_with_a_blank_line_between() {
    let root = temp_dir("merge-source-text-concat");
    let mut opened = create_work_from_text(&root, "Noi Van Ban Tho", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    set_chapter_source_text_directly(&opened, a_id, "Van ban A.");
    let b_id = insert_chapter_directly(&opened, 2, "Van ban B.");

    merge_chapter_into_previous(Some(&mut opened), b_id).expect("gop that bai");

    assert_eq!(read_chapter_source_text(&opened, a_id), "Van ban A.\n\nVan ban B.");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §Design Notes "Vì sao `source_text` của lượt TÁCH dựng lại từ segment" — cả hai nửa dựng
/// lại bằng phép nối `segment.source_text` CÒN SỐNG, phân tách bằng `"\n"`, bỏ qua hàng VỀ HƯU.
#[test]
fn splitting_rebuilds_source_text_from_living_segments_joined_by_newline() {
    let root = temp_dir("split-source-text-rebuild");
    let mut opened = create_work_from_text(&root, "Dung Lai Van Ban", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    insert_segment_directly(&opened, a_id, 1, "Cau mot.", false);
    insert_segment_directly(&opened, a_id, 2, "Cau ve huu khong duoc dem.", true);
    let s3 = insert_segment_directly(&opened, a_id, 3, "Cau ba.", false);

    split_chapter_at_segment(Some(&mut opened), s3).expect("tach that bai");

    assert_eq!(read_chapter_source_text(&opened, a_id), "Cau mot.", "A chi con cau SONG dung TRUOC diem cat");

    let b_id = snapshot_segments(&opened).iter().find(|s| s.id == s3).unwrap().chapter_id;
    assert_eq!(read_chapter_source_text(&opened, b_id), "Cau ba.");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// **Bước 4** — "một lượt tổ chức thành công thì `library_work.chapter_count` phải theo
/// kịp NGAY", cùng khuôn `lifecycle_contract.rs::setting_a_chapter_status_leaves_the_new_
/// value_in_the_library_index`. Gọi CHÍNH `merge_chapter_into_previous_indexed` -- đối chứng
/// để chạy lại về sau (§Verification): gỡ lời gọi `finish_lifecycle_write` khỏi bốn hàm
/// `*_indexed` rồi chạy `cargo test --locked` ⇒ ca này phải ĐỎ.
#[test]
fn merging_two_chapters_leaves_the_smaller_chapter_count_in_the_library_index() {
    let root = temp_dir("merge-indexed-reaches-library");
    let side = temp_dir("merge-indexed-reaches-library-side");
    let indexer = auratranslate_lib::core::library::indexer::Indexer::open(side.join("library-index.db"))
        .unwrap_or_else(|e| panic!("mo indexer: {e}"));
    let global = Store::open(StoreSpec::global(side.join("global.db"))).unwrap_or_else(|e| panic!("mo global.db: {e}"));

    let mut opened = create_work_from_text(&root, "Gop Chi Muc", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let work_id = opened.meta.work_id.clone();
    let a_id = opened.chapter_id;
    let b_id = insert_chapter_directly(&opened, 2, "");

    // `insert_chapter_directly` ghi thẳng bằng SQL, KHÔNG đi qua khuôn bốn bước -- `meta.json`
    // trên đĩa còn mang `chapter_count = 1` cũ. Dựng lại nó TRƯỚC lượt reindex đầu, đúng bước
    // 2+3 mà mọi lệnh sản phẩm tự chạy, để phép so sánh "trước/sau" dưới đây so đúng hai lượt
    // ĐÃ đồng bộ -- không so một chỗ chưa kịp ghi với một chỗ đã ghi.
    let meta = WorkMeta::rebuild_from_store(&opened.store).unwrap_or_else(|e| panic!("rebuild_from_store: {e}"));
    meta.write_atomic(&opened.dir).unwrap_or_else(|e| panic!("write_atomic: {e}"));
    opened.meta = meta;

    // Trạng thái ban đầu phải có mặt trong chỉ mục TRƯỚC khi ca này chứng minh được gì.
    auratranslate_lib::commands::lifecycle::reindex_after_lifecycle_write(Some(&indexer), Some(&global), &root);
    let chapter_count_of = |indexer: &auratranslate_lib::core::library::indexer::Indexer| {
        indexer
            .list_works(auratranslate_lib::core::library::indexer::WorkQuery::default())
            .unwrap_or_else(|e| panic!("list_works: {e}"))
            .works
            .into_iter()
            .find(|w| w.work_id == work_id)
            .map(|w| w.chapter_count)
    };
    assert_eq!(chapter_count_of(&indexer), Some(2), "truoc luot gop, chi muc phai mang dung 2 Chuong");

    auratranslate_lib::commands::chapter::merge_chapter_into_previous_indexed(
        Some(&mut opened),
        Some(&indexer),
        Some(&global),
        &root,
        b_id,
    )
    .unwrap_or_else(|e| panic!("merge_chapter_into_previous_indexed: {e:?}"));

    assert_eq!(
        chapter_count_of(&indexer),
        Some(1),
        "sau luot gop, library_work.chapter_count phai theo kip NGAY -- neu ca nay xanh trong \
         khi buoc 4 da bi go thi cho noi khong co ai canh"
    );

    let _ = a_id;
    drop(opened);
    indexer.close();
    global.close();
    cleanup(&root);
    cleanup(&side);
}

/// §I/O Matrix hàng *"Dời lên/xuống"* — **CHỈ cột `ord` đổi**, và đúng trên hai hàng.
///
/// 🔴 Ca này ra đời từ lượt rà ma trận, không từ lượt viết đầu.
/// `chapter_ord_stays_dense_from_one_after_a_merge_on_a_sparse_ord_sequence` đã chạy một lượt
/// dời, nhưng nó khẳng định về **thứ tự** (`ord` liên tục, c2 lên trước c1) — **không** về
/// những thứ KHÔNG được đổi. Hàng ma trận nói *"chỉ cột `ord` đổi"*, và một mệnh đề phủ định
/// chỉ nghiệm thu được bằng một phép chụp trước/sau.
#[test]
fn moving_a_chapter_changes_ord_only_and_leaves_titles_status_and_every_segment_untouched() {
    let root = temp_dir("move-touches-ord-only");
    let mut opened = create_work_from_text(&root, "Doi Chi Ord", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    rename_chapter(Some(&mut opened), a_id, "Chuong A").expect("dat ten A that bai");
    set_chapter_status_directly(&opened, a_id, "done");
    insert_segment_directly(&opened, a_id, 1, "A mot.", false);

    let b_id = insert_chapter_directly(&opened, 2, "Nguyen van B.");
    rename_chapter(Some(&mut opened), b_id, "Chuong B").expect("dat ten B that bai");
    insert_segment_directly(&opened, b_id, 1, "B mot.", false);
    let b2_retired = insert_segment_directly(&opened, b_id, 2, "B hai ve huu.", true);

    let segments_before = snapshot_segments(&opened);
    let versions_before = snapshot_segment_versions(&opened);
    let a_source_before = read_chapter_source_text(&opened, a_id);
    let b_source_before = read_chapter_source_text(&opened, b_id);

    move_chapter(Some(&mut opened), b_id, ChapterDirection::Prev).expect("doi len that bai");

    // ① `ord` ĐỔI, và đúng phép hoán vị.
    let rows = read_all_chapter_ord(&opened);
    let ord_of = |id: i64| rows.iter().find(|(rid, _)| *rid == id).unwrap().1;
    assert_eq!(ord_of(b_id), 1, "B len dau");
    assert_eq!(ord_of(a_id), 2, "A xuong sau");

    // ② MỌI THỨ KHÁC giữ nguyên -- day la ve ma hang ma tran doi.
    assert_eq!(read_chapter_title(&opened, a_id).as_deref(), Some("Chuong A"));
    assert_eq!(read_chapter_title(&opened, b_id).as_deref(), Some("Chuong B"));
    assert_eq!(read_chapter_status(&opened, a_id), "done", "dat lai thu tu KHONG dung toi status");
    assert_eq!(read_chapter_source_text(&opened, a_id), a_source_before);
    assert_eq!(read_chapter_source_text(&opened, b_id), b_source_before);
    assert_eq!(
        snapshot_segments(&opened),
        segments_before,
        "sap lai Chuong KHONG duoc cham mot cot nao cua bang segment -- ke ca hang da ve huu"
    );
    assert_eq!(snapshot_segment_versions(&opened), versions_before);
    assert!(
        snapshot_segments(&opened).iter().find(|s| s.id == b2_retired).unwrap().retired_at.is_some(),
        "hang ve huu van ve huu"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix hàng *"Tách trên `segment_id` lạ"* — một câu KHÔNG thuộc Chương đang mở bị từ
/// chối bằng khoá đã có, và **0 hàng bị chạm**.
///
/// 🔴 Đây là ca phân biệt giữa *"tách"* và *"tách nhầm kho"*: `segment.id` là
/// `AUTOINCREMENT` trong TỪNG `project.db`, nên một id có thật ở Chương KHÁC đi qua sạch nếu
/// hàm chỉ kiểm sự tồn tại thay vì kiểm QUYỀN SỞ HỮU.
#[test]
fn splitting_on_a_segment_of_another_chapter_is_refused_before_any_write() {
    let root = temp_dir("split-foreign-segment");
    let mut opened = create_work_from_text(&root, "Tach Nham Chuong", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    insert_segment_directly(&opened, a_id, 1, "A mot.", false);
    insert_segment_directly(&opened, a_id, 2, "A hai.", false);

    // Câu này thuộc Chương B, trong khi Chương ĐANG MỞ là A.
    let b_id = insert_chapter_directly(&opened, 2, "");
    let b1 = insert_segment_directly(&opened, b_id, 1, "B mot.", false);

    let chapters_before = read_all_chapter_ord(&opened);
    let segments_before = snapshot_segments(&opened);

    let err = split_chapter_at_segment(Some(&mut opened), b1)
        .expect_err("tach tren mot cau cua Chuong KHAC phai la mot loi CO TEN");
    assert_eq!(err.code(), "segment.not_found");

    assert_eq!(read_all_chapter_ord(&opened), chapters_before, "0 hang chapter bi cham");
    assert_eq!(snapshot_segments(&opened), segments_before, "0 hang segment bi cham");

    // Đối chứng ÂM cho cùng khoá: một `segment_id` chưa từng tồn tại đi cùng đường.
    let khong_ton_tai = b1 + 9_999;
    let err2 = split_chapter_at_segment(Some(&mut opened), khong_ton_tai)
        .expect_err("tach tren mot id khong ton tai phai la mot loi CO TEN");
    assert_eq!(err2.code(), "segment.not_found");
    assert_eq!(snapshot_segments(&opened), segments_before, "van 0 hang segment bi cham");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix hàng *"Chưa mở Tác phẩm"* — **cả bốn** lệnh tổ chức tái dùng `work.none_open`,
/// và không lệnh nào chạm SQL.
///
/// ⚠️ Bốn lời gọi, một ca: mệnh đề là *"danh mục ĐÓNG — không đúc khoá thứ hai cho cùng
/// câu"*, và nó chỉ đọc được khi bốn lượt đứng cạnh nhau. Tách thành bốn ca làm mệnh đề ấy
/// biến mất khỏi cả bốn.
#[test]
fn every_chapter_organise_command_without_a_work_open_reuses_the_named_error() {
    for err in [
        rename_chapter(None, 1, "Ten moi").expect_err("khong co Tac pham mo"),
        move_chapter(None, 1, ChapterDirection::Prev).expect_err("khong co Tac pham mo"),
        merge_chapter_into_previous(None, 1).expect_err("khong co Tac pham mo"),
        split_chapter_at_segment(None, 1).expect_err("khong co Tac pham mo"),
    ] {
        assert_eq!(err.code(), "work.none_open");
        assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
    }
}

/// §Always *"Một lượt ghi không đi đâu phải NÓI VÌ SAO"* áp cho một `chapter_id` **không tồn
/// tại** — cả `move_chapter` LẪN `merge_chapter_into_previous`, đúng khuôn
/// [`renaming_an_unknown_chapter_id_reuses_the_named_error`].
///
/// 🔴 **Ca này ra đời từ lượt rà, và nó có một PHÉP ĐO đứng sau.** Trước lượt vá 2026-08-29,
/// hai hàm này đọc `SELECT ord FROM chapter WHERE id = ?1` bằng `query_row` mà không kiểm hàng
/// tồn tại; một `chapter_id` lạ cho `QueryReturnedNoRows`, `Store::write` gói nó thành
/// `StoreError::WriteFailed`, và mã lỗi đi ra là **`store.write_failed`** — đo được bằng cách
/// chạy đúng hai lời gọi dưới đây trên cây chưa vá. Người dùng đọc một câu LOẠI *"kho hỏng"*
/// cho một Tác phẩm hoàn toàn lành lặn: đúng lớp lỗi mà Story 2.11 đã sửa MỘT LẦN khi dựng
/// `chapter_not_found` (xem doc-comment của hàm đó).
#[test]
fn moving_or_merging_an_unknown_chapter_id_reuses_the_named_error_not_a_store_error() {
    let root = temp_dir("unknown-id-move-merge");
    let mut opened = create_work_from_text(&root, "Id La", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let a_id = opened.chapter_id;
    insert_segment_directly(&opened, a_id, 1, "A mot.", false);
    insert_chapter_directly(&opened, 2, "");

    let chapters_before = read_all_chapter_ord(&opened);
    let segments_before = snapshot_segments(&opened);
    // `AUTOINCREMENT` không bao giờ phát lại, nên một số vượt xa mọi id đã cấp là "không tồn
    // tại" một cách chắc chắn -- không phụ thuộc thứ tự chạy của các ca khác.
    let khong_ton_tai = 999_999_i64;

    for err in [
        move_chapter(Some(&mut opened), khong_ton_tai, ChapterDirection::Next)
            .expect_err("doi mot Chuong khong ton tai phai la mot loi CO TEN"),
        move_chapter(Some(&mut opened), khong_ton_tai, ChapterDirection::Prev)
            .expect_err("doi mot Chuong khong ton tai phai la mot loi CO TEN"),
        merge_chapter_into_previous(Some(&mut opened), khong_ton_tai)
            .expect_err("gop mot Chuong khong ton tai phai la mot loi CO TEN"),
    ] {
        assert_eq!(
            err.code(),
            "segment.chapter_not_found",
            "mot `chapter_id` la la mot loi NGHIEP VU, khong mot loi KHO -- `store.write_failed` \
             o day la mot cau SAI VE LOAI (khong tep nao hong)"
        );
        assert_eq!(err.message_key(), MessageKey::SegmentChapterNotFound);
    }

    assert_eq!(read_all_chapter_ord(&opened), chapters_before, "0 hang chapter bi cham");
    assert_eq!(snapshot_segments(&opened), segments_before, "0 hang segment bi cham");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// Bản song sinh của [`chapter_ord_stays_dense_from_one_after_a_merge_on_a_sparse_ord_sequence`]
/// cho đường **GỘP** — ca kia mang chữ *"after_a_merge"* trong tên nhưng chỉ chạy một lượt
/// `move_chapter`, nên trước ca này `normalize_chapter_ord` **chưa từng** được nghiệm thu trên
/// đường gộp với một dãy `ord` thưa/trùng. (Tên ca kia đã sửa cùng lượt.)
#[test]
fn chapter_ord_stays_dense_from_one_after_a_merge_on_a_sparse_ord_sequence_too() {
    let root = temp_dir("merge-sparse-ord");
    let opened_bootstrap = create_work_from_text(&root, "Gop Ord Thua", "zh", "", String::new())
        .expect("tao tac pham that bai");
    let default_id = opened_bootstrap.chapter_id;
    opened_bootstrap
        .store
        .write(move |tx: &Transaction<'_>| tx.execute("DELETE FROM chapter WHERE id = ?1", [default_id]))
        .expect("xoa Chuong mac dinh that bai");
    let mut opened = opened_bootstrap;

    let c1 = insert_chapter_directly(&opened, 5, "Chuong mot.");
    let c2 = insert_chapter_directly(&opened, 5, "Chuong hai.");
    let c3 = insert_chapter_directly(&opened, 9, "Chuong ba.");
    let s1 = insert_segment_directly(&opened, c1, 1, "Mot.", false);
    let s2 = insert_segment_directly(&opened, c2, 1, "Hai.", false);
    let s3 = insert_segment_directly(&opened, c3, 1, "Ba.", false);
    opened.chapter_id = c1;

    // Sau chuẩn hoá: c1 → 1, c2 → 2, c3 → 3. Gộp c2 vào c1.
    merge_chapter_into_previous(Some(&mut opened), c2).expect("gop that bai");

    let rows = read_all_chapter_ord(&opened);
    let mut ords: Vec<i64> = rows.iter().map(|(_, ord)| *ord).collect();
    ords.sort_unstable();
    assert_eq!(ords, vec![1, 2], "hai Chuong con lai phai mang ord lien tuc 1, 2");
    assert!(rows.iter().any(|(id, ord)| *id == c1 && *ord == 1));
    assert!(rows.iter().any(|(id, ord)| *id == c3 && *ord == 2), "c3 tinh tien xuong 2, khong con 3");

    let after = snapshot_segments(&opened);
    let by_id = |id: i64| after.iter().find(|s| s.id == id).unwrap();
    assert_eq!(by_id(s2).chapter_id, c1, "cau cua c2 di sang c1");
    assert_eq!(by_id(s2).ord, 2, "tinh tien bang MAX(ord) cua c1 = 1");
    assert_eq!(by_id(s1).ord, 1);
    assert_eq!(by_id(s3).chapter_id, c3, "c3 khong bi dung toi");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}
