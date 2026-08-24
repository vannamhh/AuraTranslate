//! Mọi hàng của I/O Matrix — Story 3.7 (`suggest_han_viet_batch`, FR113).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO MỘT TỆP RIÊNG
//! ─────────────────────────────────────────────────────────────────────────────
//! Cùng lý do `glossary_marks_contract.rs` tách khỏi `glossary_contract.rs`: đây là hợp
//! đồng của MỘT hàm phơi ra mới (`core::glossary::suggest_han_viet_batch`), không phải một
//! phép kiểm rải rác thêm vào một tệp đã có. `core/glossary/han_viet_suggestion.rs::tests`
//! canh các hàm THUẦN nội bộ (`capitalize_first`, `as_status_str`) bằng dữ liệu bịa; tệp này
//! canh hành vi TRÊN DỮ LIỆU TỪ ĐIỂN THẬT (fixture `.db`, khuôn `dict_sources.rs`), gồm cả
//! ca `DictLayers::empty() ⇒ DictUnavailable ≠ NoReading` mà cây git rỗng
//! (`resources/dict/`, AD-25) làm thành ca THƯỜNG GẶP NHẤT ở máy dev.
//!
//! Fixture ở đây là bản RÚT GỌN của `dict_sources.rs::build_layer` — chỉ ba bảng
//! `dict_meta`/`dict_source`/`dict_entry` (`DictLayer::open` không đọc gì khác lúc mở, và
//! `core::dict::han_viet` chỉ JOIN đúng hai bảng sau), không phải một bản chép nguyên văn:
//! hai tệp test không được `use` chéo nhau (mỗi `tests/*.rs` là một crate riêng).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::core::dict::DictLayers;
use auratranslate_lib::core::glossary::{HanVietSuggestion, suggest_han_viet_batch};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-han-viet-suggestion-{}-{}-{}",
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

const SCHEMA_VERSION: &str = "3";
const USER_VERSION: u32 = 3;

/// `(headword, headword_simp, han_viet)` — một hàng `dict_entry` cho fixture.
type EntrySeed = (&'static str, Option<&'static str>, Option<&'static str>);

/// Dựng MỘT tệp `.db` fixture RÚT GỌN — chỉ ba bảng mà `DictLayer::open` +
/// `core::dict::han_viet` chạm tới. Không `dict_sense`/`char_idx`/`entry_fts`: cổng này
/// canh `suggest_han_viet_batch`, không canh tra cứu từ điển đầy đủ.
fn build_layer(dir: &Path, file: &str, layer: &str, source_code: &str, entries: &[EntrySeed]) {
    let path = dir.join(file);
    let conn =
        rusqlite::Connection::open(&path).unwrap_or_else(|e| panic!("dung fixture {file}: {e}"));

    conn.execute_batch(
        "CREATE TABLE dict_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
         CREATE TABLE dict_source (
           id             INTEGER PRIMARY KEY,
           code           TEXT NOT NULL UNIQUE,
           display_name   TEXT NOT NULL,
           license_kind   TEXT NOT NULL,
           license_id     TEXT,
           license_text   TEXT NOT NULL,
           attribution    TEXT NOT NULL,
           source_version TEXT NOT NULL,
           source_url     TEXT NOT NULL,
           lang           TEXT NOT NULL DEFAULT ''
         );
         CREATE TABLE dict_entry (
           id            INTEGER PRIMARY KEY,
           source_id     INTEGER NOT NULL REFERENCES dict_source(id),
           lang          TEXT NOT NULL,
           headword      TEXT NOT NULL,
           headword_simp TEXT,
           reading       TEXT,
           han_viet      TEXT,
           nom_reading   TEXT
         );",
    )
    .unwrap_or_else(|e| panic!("DDL fixture {file}: {e}"));

    conn.execute(
        "INSERT INTO dict_meta (key, value) VALUES ('schema_version', ?1)",
        rusqlite::params![SCHEMA_VERSION],
    )
    .unwrap_or_else(|e| panic!("nap schema_version {file}: {e}"));
    conn.execute(
        "INSERT INTO dict_meta (key, value) VALUES ('layer', ?1)",
        rusqlite::params![layer],
    )
    .unwrap_or_else(|e| panic!("nap layer {file}: {e}"));

    conn.execute(
        "INSERT INTO dict_source
           (id, code, display_name, license_kind, license_id, license_text,
            attribution, source_version, source_url)
         VALUES (1, ?1, ?1, 'public-domain', NULL, 'x', 'x', '1', 'x')",
        rusqlite::params![source_code],
    )
    .unwrap_or_else(|e| panic!("nap dict_source {file}: {e}"));

    for (id, (headword, simp, han_viet)) in entries.iter().enumerate() {
        conn.execute(
            "INSERT INTO dict_entry (id, source_id, lang, headword, headword_simp, han_viet)
             VALUES (?1, 1, 'zh', ?2, ?3, ?4)",
            rusqlite::params![(id as i64) + 1, headword, simp, han_viet],
        )
        .unwrap_or_else(|e| panic!("nap dict_entry {headword} trong {file}: {e}"));
    }

    conn.execute_batch(&format!("PRAGMA user_version = {USER_VERSION};"))
        .unwrap_or_else(|e| panic!("dat user_version {file}: {e}"));
    conn.close().unwrap_or_else(|(_, e)| panic!("dong fixture {file}: {e}"));
}

/// Một lớp NỀN duy nhất, mang đủ dữ liệu cho mọi ca KHÔNG liên quan tới ưu tiên nhiều lớp.
fn build_base_only(dir: &Path) {
    build_layer(
        dir,
        "zzz.db",
        "base",
        "fx-core",
        &[
            ("北", None, Some("bắc")),
            ("涼", None, Some("lương")),
            // Đa âm, khuôn en-wiktionary-vi -- không khoảng trắng, phẩy phân tách.
            ("西", None, Some("tây,tê")),
            // "慕" có âm; "容" KHÔNG có hàng nào trong fixture này -- test "thiếu một âm"
            // ghép chúng thành thuật ngữ "慕容".
            ("慕", None, Some("mộ")),
        ],
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Danh từ riêng tra đủ
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_fully_readable_chinese_term_becomes_a_capitalized_space_joined_suggestion() {
    let dir = temp_dir("full");
    build_base_only(&dir);
    let layers = DictLayers::open(&dir);

    let out = suggest_han_viet_batch(&layers, &Default::default(), &["北涼"]);

    assert_eq!(out.len(), 1);
    assert_eq!(out[0], HanVietSuggestion::Ready("Bắc Lương".to_owned()));
    assert_eq!(out[0].as_status_str(), "ok");
    assert_eq!(out[0].suggestion_text(), Some("Bắc Lương"));

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ký tự đa âm -- lấy `primary`
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_multi_reading_character_uses_only_the_primary_reading() {
    let dir = temp_dir("multi-reading");
    build_base_only(&dir);
    let layers = DictLayers::open(&dir);

    let out = suggest_han_viet_batch(&layers, &Default::default(), &["西"]);

    assert_eq!(
        out[0],
        HanVietSuggestion::Ready("Tây".to_owned()),
        "西 mang \"tây,tê\" -- chỉ am DAU (\"tây\") duoc dung, khong ca chuoi tho"
    );

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ứng viên tiếng Anh
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_english_candidate_is_not_chinese_regardless_of_dictionary_state() {
    let dir = temp_dir("english");
    build_base_only(&dir);
    let layers = DictLayers::open(&dir);

    let out = suggest_han_viet_batch(&layers, &Default::default(), &["dragon"]);

    assert_eq!(out[0], HanVietSuggestion::NotChinese);
    assert_eq!(out[0].as_status_str(), "not_chinese");
    assert_eq!(out[0].suggestion_text(), None);

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Chuỗi Hán thiếu một âm -- không đề xuất một phần
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_chinese_term_missing_one_readings_is_no_reading_not_a_partial_suggestion() {
    let dir = temp_dir("missing-reading");
    build_base_only(&dir);
    let layers = DictLayers::open(&dir);

    // "慕" co am ("mo"); "容" khong co hang nao trong fixture -- ca nguyen thuat ngu phai la
    // `NoReading`, khong phai mot chuoi mot ky tu.
    let out = suggest_han_viet_batch(&layers, &Default::default(), &["慕容"]);

    assert_eq!(out[0], HanVietSuggestion::NoReading);
    assert_eq!(out[0].suggestion_text(), None, "khong duoc de xuat MOT PHAN");

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Chuỗi lẫn Hán và Latin
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_mixed_han_and_latin_term_is_not_chinese() {
    let dir = temp_dir("mixed");
    build_base_only(&dir);
    let layers = DictLayers::open(&dir);

    let out = suggest_han_viet_batch(&layers, &Default::default(), &["A北"]);

    assert_eq!(
        out[0],
        HanVietSuggestion::NotChinese,
        "vi tu la MOI ky tu la Han -- mot ky tu Latin lam ca cum thanh khong-phai-tieng-Trung"
    );

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Chưa gắn lớp từ điển nào -- KHÁC "thiếu một âm"
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 Ca mà cả story đứng lên: cây git rỗng `resources/dict/` (AD-25) làm `DictLayers::
/// empty()` thành trạng thái THƯỜNG GẶP NHẤT ở máy dev -- thiếu ca này thì mọi ca khác của
/// tệp này XANH GIẢ trên một máy chưa cài dữ liệu từ điển (chúng đều CÀI fixture riêng).
#[test]
fn zero_layers_loaded_is_dict_unavailable_not_no_reading() {
    let layers = DictLayers::empty();

    let out = suggest_han_viet_batch(&layers, &Default::default(), &["北涼"]);

    assert_eq!(out[0], HanVietSuggestion::DictUnavailable);
    assert_ne!(
        out[0], HanVietSuggestion::NoReading,
        "\"chua tung tra duoc\" va \"da tra ma thieu am\" la HAI nhan RIENG"
    );
    assert_eq!(out[0].as_status_str(), "dict_unavailable");

    // Đối chứng: MỘT lớp thật đang gắn (fixture rỗng dữ liệu nhưng CÓ tệp) thì cùng thuật
    // ngữ đó phải đi nhánh `NoReading`, không `DictUnavailable` -- chứng minh hai nhánh
    // thật sự phân biệt được bằng trạng thái `layers_loaded`, không phải một hằng số bịa.
    let dir = temp_dir("contrast-empty-vs-loaded");
    build_layer(&dir, "zzz.db", "base", "fx-core-empty", &[]);
    let loaded_but_empty = DictLayers::open(&dir);
    let out2 = suggest_han_viet_batch(&loaded_but_empty, &Default::default(), &["北涼"]);
    assert_eq!(out2[0], HanVietSuggestion::NoReading);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Nguồn thắng bị TẮT -- chữ ĐỔI, không biến mất
// ═════════════════════════════════════════════════════════════════════════════════

/// Story 1.19 §Quyết định #3a: đề xuất theo CÙNG bộ lọc nguồn đã tắt với tab Hán Việt.
/// `priority_order` đẩy lớp NỀN xuống cuối -- tắt lớp gỡ rời (ưu tiên cao hơn) phải làm
/// `suggest_han_viet_batch` rơi về âm của lớp NỀN, không trả `NoReading`/`DictUnavailable`.
#[test]
fn disabling_the_winning_source_changes_the_reading_instead_of_dropping_it() {
    let dir = temp_dir("disabled-source");
    // Lớp NỀN ("zzz.db"/"base") -- âm KHÁC, để phân biệt được hai nhánh.
    build_layer(&dir, "zzz.db", "base", "fx-base", &[("北", None, Some("bắc"))]);
    // Lớp GỠ RỜI ("mmm.db") -- ưu tiên CAO HƠN lớp nền (`priority_order` đẩy `base` xuống
    // cuối), âm KHÁC lớp nền.
    build_layer(&dir, "mmm.db", "detachable-fixture", "fx-detachable", &[(
        "北",
        None,
        Some("bối"),
    )]);
    let layers = DictLayers::open(&dir);

    let enabled = suggest_han_viet_batch(&layers, &Default::default(), &["北"]);
    assert_eq!(
        enabled[0],
        HanVietSuggestion::Ready("Bối".to_owned()),
        "lop go roi uu tien cao hon phai thang khi CA HAI nguon deu bat"
    );

    let mut disabled = std::collections::BTreeSet::new();
    disabled.insert("fx-detachable".to_owned());
    let after_disable = suggest_han_viet_batch(&layers, &disabled, &["北"]);
    assert_eq!(
        after_disable[0],
        HanVietSuggestion::Ready("Bắc".to_owned()),
        "tat nguon thang -- chu phai DOI sang am cua lop NEN, khong bien mat"
    );
    assert_ne!(
        after_disable[0], HanVietSuggestion::NoReading,
        "tat mot nguon KHONG duoc bien thanh \"thieu am\" khi con lop khac tra duoc"
    );

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đồng vị trí + số lượng -- một lượt gọi cho cả lô
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_output_preserves_input_order_and_count_across_a_mixed_batch() {
    let dir = temp_dir("order");
    build_base_only(&dir);
    let layers = DictLayers::open(&dir);

    let out = suggest_han_viet_batch(&layers, &Default::default(), &["dragon", "北涼", "西", "慕容"]);

    assert_eq!(out.len(), 4);
    assert_eq!(out[0], HanVietSuggestion::NotChinese);
    assert_eq!(out[1], HanVietSuggestion::Ready("Bắc Lương".to_owned()));
    assert_eq!(out[2], HanVietSuggestion::Ready("Tây".to_owned()));
    assert_eq!(out[3], HanVietSuggestion::NoReading);

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// CHỖ NỐI -- `marks_for_source_text` điền đề xuất vào dấu
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 VÌ SAO BỐN CA DƯỚI ĐÂY TỒN TẠI, và vì sao chúng KHÔNG thừa so với các ca ở trên.
//
// Mọi ca phía trên gọi thẳng `suggest_han_viet_batch` -- chúng canh HÀM. Không ca nào
// trong bộ nghiệm thu đi qua `marks_for_source_text`, tức qua đúng đoạn mã quyết định
// *mục nào được hỏi* và *đề xuất nào rơi vào dấu nào*. Rà 2026-08-24: cả 21 ca của
// `glossary_marks_contract.rs` truyền `DictLayers::empty()`, nên trong TOÀN BỘ bộ
// nghiệm thu Rust hôm nay **không một dấu nào từng mang một đề xuất khác `None`**.
// Đảo nhánh `is_confirmed`, hay bỏ luôn lượt tra `suggestion_by_term`, đều cho một bộ
// test XANH -- đúng lớp lỗi *"canh HÀM thay vì canh CHỖ NỐI"* mà vòng rà Story 3.6 đã
// bắt được một lần (`glossaryConfirmStripResetWiring.test.ts` ra đời từ đó).

fn open_global_store(dir: &Path) -> auratranslate_lib::core::store::Store {
    use auratranslate_lib::core::store::{Store, StoreSpec};
    Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db")
}

#[test]
fn a_pending_chinese_mark_carries_the_suggestion_through_marks_for_source_text() {
    use auratranslate_lib::core::glossary::{Category, GlossaryTier, add_manual_term, marks_for_source_text};
    use auratranslate_lib::core::matching::MatchLang;
    use auratranslate_lib::core::scope::ScopeResolver;

    let dict_dir = temp_dir("wiring-pending-dict");
    build_base_only(&dict_dir);
    let layers = DictLayers::open(&dict_dir);

    let global_dir = temp_dir("wiring-pending-global");
    let global = open_global_store(&global_dir);
    // `translation = None` ⇒ muc CHO CHOT, dung nhanh ma dai chot cua Story 3.6 doc.
    add_manual_term(&global, None, GlossaryTier::Global, "北涼", None, "", Category::Place)
        .expect("them muc cho chot");

    let resolver = ScopeResolver::global_only();
    let marks = marks_for_source_text(
        &resolver,
        &global,
        None,
        "北涼的人",
        MatchLang::Zh,
        &layers,
        &Default::default(),
    )
    .expect("khong loi");

    assert_eq!(marks.len(), 1, "phai co dung mot dau: {marks:?}");
    assert!(!marks[0].is_confirmed);
    assert_eq!(
        marks[0].han_viet_suggestion.as_deref(),
        Some("Bắc Lương"),
        "dau CHO CHOT phai mang de xuat -- day la chinh duong ma dai chot Story 3.6 doc"
    );
    assert_eq!(marks[0].han_viet_status, "ok");

    drop(global);
    cleanup(&global_dir);
    cleanup(&dict_dir);
}

#[test]
fn a_confirmed_chinese_mark_is_not_requested_never_not_chinese() {
    use auratranslate_lib::core::glossary::{Category, GlossaryTier, add_manual_term, marks_for_source_text};
    use auratranslate_lib::core::matching::MatchLang;
    use auratranslate_lib::core::scope::ScopeResolver;

    let dict_dir = temp_dir("wiring-confirmed-dict");
    build_base_only(&dict_dir);
    let layers = DictLayers::open(&dict_dir);

    let global_dir = temp_dir("wiring-confirmed-global");
    let global = open_global_store(&global_dir);
    // CUNG thuat ngu chu Han, nhung DA CHOT (co ban dich).
    add_manual_term(
        &global,
        None,
        GlossaryTier::Global,
        "北涼",
        Some("Bac Luong tay gõ"),
        "",
        Category::Place,
    )
    .expect("them muc da chot");

    let resolver = ScopeResolver::global_only();
    let marks = marks_for_source_text(
        &resolver,
        &global,
        None,
        "北涼的人",
        MatchLang::Zh,
        &layers,
        &Default::default(),
    )
    .expect("khong loi");

    assert_eq!(marks.len(), 1);
    assert!(marks[0].is_confirmed);
    assert_eq!(marks[0].han_viet_suggestion, None, "muc da chot khong mang de xuat");
    // 🔴 Menh de trung tam cua nhanh thu nam: `北涼` VAN la chu Han va VAN tra duoc am
    // (ca ngay tren chung minh dieu do tren cung fixture). Nhan `"not_chinese"` o day
    // se la mot loi noi doi doc duoc, va `"no_reading"` cung vay.
    assert_eq!(
        marks[0].han_viet_status, "not_requested",
        "muc DA CHOT phai la 'not_requested' -- khong muon nhan cua mot ly do khac"
    );

    drop(global);
    cleanup(&global_dir);
    cleanup(&dict_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Một lớp `.db` hỏng lúc tra -- lượt tra vẫn trả đủ
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_corrupt_layer_is_skipped_and_the_remaining_layers_still_answer() {
    let dir = temp_dir("corrupt-layer");
    build_base_only(&dir);
    // Mot tep `.db` KHONG phai SQLite, dat canh lop nen that.
    fs::write(dir.join("aaa-corrupt.db"), b"day khong phai mot tep SQLite")
        .expect("ghi tep hong");

    let layers = DictLayers::open(&dir);
    let out = suggest_han_viet_batch(&layers, &Default::default(), &["北涼"]);

    assert_eq!(
        out[0],
        HanVietSuggestion::Ready("Bắc Lương".to_owned()),
        "mot lop hong bi BO QUA -- no khong duoc lam hong ca luot tra (cung luat `lookup_grouped`)"
    );

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ngắt kết nối mạng -- vẫn chạy đầy đủ
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_suggestion_path_names_no_network_client_so_it_cannot_depend_on_the_network() {
    // ⚠️ GIOI HAN THAT, ghi ra thay vi de nguoi sau tuong da duoc xet: day la mot phep
    // kiem TINH tren van ban nguon, khong phai mot phep do voi cap mang bi rut. No canh
    // dung mot menh de -- module de xuat khong GOI TEN mot client mang nao -- va menh de
    // do la thu duy nhat kiem duoc bang may o tang nay. Ve con lai (moi dau vao la mot
    // tep `.db` cuc bo) da duoc chinh cac ca tren chung minh: chung chay tren fixture
    // tren dia, khong mot lay byte nao tu ngoai.
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/core/glossary/han_viet_suggestion.rs");
    let src = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("day la loi ha tang, khong phai mot phep kiem do: {e}"));

    for token in ["reqwest", "ureq", "hyper", "TcpStream", "tauri_plugin_http", "http://", "https://"] {
        assert!(
            !src.contains(token),
            "module de xuat khong duoc goi ten `{token}` -- FR113 chay hoan toan ngoai tuyen"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// CHỖ NỐI thứ hai -- `glossary_pending_candidates` điền đề xuất vào hàng ứng viên
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 Vòng rà Bước 4 (2026-08-24) bắt được: lượt rà bảng I/O của Bước 3 vá chỗ nối
// `marks_for_source_text` nhưng BỎ SÓT chỗ nối thứ hai. Đo lúc đó:
// `grep -n "han_viet" src-tauri/tests/glossary_commands_contract.rs` = **0**, và cả 5 lời
// gọi `glossary_pending_candidates` trong tệp đó truyền `DictLayers::empty()`. Tức xoá
// sạch lượt tính đề xuất khỏi `commands::glossary::glossary_pending_candidates` cũng
// KHÔNG làm ca nào đỏ -- cùng lớp lỗ, khác bề mặt.
//
// HAI hàng chờ (không phải một) là điều kiện cần: mọi fixture cũ chỉ chèn MỘT ứng viên,
// nên một lượt ghép LỆCH (đề xuất của thuật ngữ này dán lên thuật ngữ kia) là vô hình
// theo cấu trúc. Một ca một hàng không phân biệt được "ghép đúng" với "ghép bừa".

#[test]
fn each_pending_candidate_row_carries_the_suggestion_of_its_own_source_term() {
    use auratranslate_lib::commands::glossary::glossary_pending_candidates;
    use auratranslate_lib::commands::project::create_work_from_text;
    use auratranslate_lib::core::glossary::{insert_import_scan_candidates, scan::ScanCandidate};

    let dict_dir = temp_dir("candidates-wiring-dict");
    build_base_only(&dict_dir);
    let layers = DictLayers::open(&dict_dir);

    let root = temp_dir("candidates-wiring-work");
    let opened = create_work_from_text(&root, "Ung Vien Co De Xuat", "zh", "", "noi dung".to_owned())
        .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));

    // Ba hang, BA ket cuc KHAC NHAU -- de mot luot ghep bua khong the tinh co dung.
    insert_import_scan_candidates(
        &opened.store,
        &[
            ScanCandidate {
                source_term: "北涼".to_owned(),
                occurrence_count: 9,
                context_example: "a".to_owned(),
            },
            ScanCandidate {
                source_term: "dragon".to_owned(),
                occurrence_count: 4,
                context_example: "b".to_owned(),
            },
            ScanCandidate {
                source_term: "慕容".to_owned(),
                occurrence_count: 2,
                context_example: "c".to_owned(),
            },
        ],
    )
    .expect("chen ba ung vien");

    let rows = glossary_pending_candidates(Some(&opened), &layers, &Default::default())
        .expect("liet ke bang cho");

    let pick = |term: &str| {
        rows.iter()
            .find(|c| c.source_term == term)
            .unwrap_or_else(|| panic!("phai co hang {term}"))
    };

    // Tra du am ⇒ de xuat that.
    assert_eq!(pick("北涼").han_viet_suggestion.as_deref(), Some("Bắc Lương"));
    assert_eq!(pick("北涼").han_viet_status, "ok");
    // Khong phai chu Han ⇒ khong de xuat, va LY DO doc duoc.
    assert_eq!(pick("dragon").han_viet_suggestion, None);
    assert_eq!(pick("dragon").han_viet_status, "not_chinese");
    // Chu Han nhung thieu mot am ⇒ mot LY DO KHAC han, khong duoc lan voi ca tren.
    assert_eq!(pick("慕容").han_viet_suggestion, None);
    assert_eq!(pick("慕容").han_viet_status, "no_reading");

    // 🔴 Menh de ma doc-comment cua `GlossaryCandidateWire` KHAI nhung chua ai kiem: mot
    // ung vien la CHO DUYET, khong bao gio da chot, nen `"not_requested"` -- nhan danh cho
    // dau DA CHOT -- khong duoc xuat hien o bat ky hang nao cua bang cho.
    assert!(
        rows.iter().all(|c| c.han_viet_status != "not_requested"),
        "khong hang cho duyet nao duoc mang nhan cua mot dau DA CHOT"
    );

    drop(opened);
    cleanup(&root);
    cleanup(&dict_dir);
}
