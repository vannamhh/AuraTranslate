//! Hành vi ba nhánh tra cứu — Story 1.11, AC1 tới AC8.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ VÌ SAO TỆP NÀY ĐƯỢC PHÉP `use rusqlite`
//! ─────────────────────────────────────────────────────────────────────────────
//! `store_boundary.rs` cưỡng chế ranh giới trên `src-tauri/src/**`; `tests/**` nằm ngoài,
//! **có tên và có lý do** (doc-comment `store_boundary.rs:27-31`). Lý do ở đây: mọi ca
//! dưới đây cần một tệp `.db` mang **đúng lược đồ của `tools/dict-build`**, và không
//! tệp `.db` nào nằm trong git (`.gitignore: *.db` — đó là AD-25, và doc-comment của
//! dòng đó viết *"Đừng gỡ dòng này"*). Tệp thật nặng **195 MB**, nên CI không có gì để
//! tra. Fixture phải dựng trong test, và dựng nó là việc của `rusqlite`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CỔNG CHỐNG TRÔI LƯỢC ĐỒ
//! ─────────────────────────────────────────────────────────────────────────────
//! DDL dưới đây **chép** từ `tools/dict-build/src/schema.rs`, và
//! [`fixture_ddl_is_verbatim_from_dict_build_schema`] đọc tệp đó **dưới dạng văn bản** rồi
//! khẳng định từng khối có mặt nguyên văn. Không có cổng đó, hai cây trôi khỏi nhau trong
//! im lặng và mọi ca dưới đây kiểm một database **không tồn tại trong sản phẩm**.
//!
//! ⚠️ Cổng so **văn bản**, không so `sqlite_master` — đó chính là điều kiện để nó chạy
//! được mà không cần một tệp `.db` nào, tức để nó ở được trong CI.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT — thừa kế nguyên từ `store_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + bộ đếm nguyên tử). Không `tempfile` —
//!    nó là dev-dependency của `tools/dict-build`, **không** của `src-tauri`.
//! 2. **Drop `ReadOnlyDb` TRƯỚC khi xoá thư mục** — Windows từ chối xoá tệp đang mở
//!    (NFR14).
//! 3. **Không ngưỡng thời gian trong CI.** Phép đo NFR1 là
//!    [`bench_three_branches_on_the_real_dictionary`]: `#[ignore]`, lái bằng biến môi
//!    trường, và vắng biến thì bỏ qua.
//! 4. **Đường dẫn tương đối lấy qua `env!("CARGO_MANIFEST_DIR")`.**

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::core::dict::{
    LookupMode, QueryBranch, QueryRoute, is_han, lookup, pick_branch, pick_route,
};
use auratranslate_lib::core::store::{ReadOnlyDb, StoreKind};

/// 🔴 Trần pha một (Quyết định #4, Story 1.17) — mọi fixture của tệp này có dưới mười
/// hàng, nên một trần lớn giữ nguyên hành vi trước story: không ca nào trong tệp này
/// nhắm tới việc đo `truncated` — xem `dict_sources.rs` cho các ca đó (AC12).
const UNLIMITED: usize = 10_000;

// ═════════════════════════════════════════════════════════════════════════════════
// DDL — CHÉP NGUYÊN VĂN từ `tools/dict-build/src/schema.rs`
// ═════════════════════════════════════════════════════════════════════════════════
//
// Đừng "dọn dẹp" khoảng trắng ở đây. Cổng parity so **chuỗi con nguyên văn**; một
// lượt canh lề tử tế làm nó đỏ, và người sửa tiếp theo sẽ sửa bằng cách nới cổng.

const DICT_META_DDL: &str = "\
CREATE TABLE dict_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);";

const DICT_SOURCE_DDL: &str = "\
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
);";

const DICT_ENTRY_DDL: &str = "\
CREATE TABLE dict_entry (
  id            INTEGER PRIMARY KEY,
  source_id     INTEGER NOT NULL REFERENCES dict_source(id),
  lang          TEXT NOT NULL,
  headword      TEXT NOT NULL,
  headword_simp TEXT,
  reading       TEXT,
  han_viet      TEXT,
  nom_reading   TEXT
);";

const DICT_SENSE_DDL: &str = "\
CREATE TABLE dict_sense (
  id        INTEGER PRIMARY KEY,
  entry_id  INTEGER NOT NULL REFERENCES dict_entry(id),
  source_id INTEGER NOT NULL REFERENCES dict_source(id),
  pos       TEXT,
  pos_lang  TEXT,
  gloss     TEXT NOT NULL,
  note      TEXT,
  ord       INTEGER NOT NULL
);";

const CHAR_IDX_DDL: &str = "\
CREATE TABLE char_idx (
  ch       TEXT    NOT NULL,
  entry_id INTEGER NOT NULL REFERENCES dict_entry(id),
  PRIMARY KEY (ch, entry_id)
) WITHOUT ROWID;";

const ENTRY_INDEXES_DDL: &str = "\
CREATE INDEX idx_entry_headword      ON dict_entry(headword);
CREATE INDEX idx_entry_headword_simp ON dict_entry(headword_simp);
CREATE INDEX idx_sense_entry         ON dict_sense(entry_id);
CREATE INDEX idx_example_sense       ON dict_example(sense_id);
CREATE INDEX idx_citation_sense      ON dict_citation(sense_id);";

const ENTRY_FTS_DDL: &str = "\
CREATE VIRTUAL TABLE entry_fts USING fts5(
  headword, content='dict_entry', content_rowid='id', tokenize=\"trigram\");";

/// ⚠️ Hai bảng này **không** được story dùng, nhưng `ENTRY_INDEXES_DDL` dựng chỉ mục
/// trên chúng — nên fixture phải có chúng, nếu không `CREATE INDEX` gãy. Chép cùng nguồn,
/// cùng lý do, và chúng cũng đi qua cổng parity.
const DICT_EXAMPLE_DDL: &str = "\
CREATE TABLE dict_example (
  id               INTEGER PRIMARY KEY,
  sense_id         INTEGER NOT NULL REFERENCES dict_sense(id),
  text             TEXT NOT NULL,
  translation      TEXT,
  translation_lang TEXT,
  ord              INTEGER NOT NULL
);";

const DICT_CITATION_DDL: &str = "\
CREATE TABLE dict_citation (
  id       INTEGER PRIMARY KEY,
  sense_id INTEGER NOT NULL REFERENCES dict_sense(id),
  text     TEXT NOT NULL,
  work     TEXT,
  author   TEXT,
  ord      INTEGER NOT NULL
);";

/// Mọi khối DDL đã chép — quần thể của cổng parity.
const COPIED_DDL: &[(&str, &str)] = &[
    ("DICT_META_DDL", DICT_META_DDL),
    ("DICT_SOURCE_DDL", DICT_SOURCE_DDL),
    ("DICT_ENTRY_DDL", DICT_ENTRY_DDL),
    ("DICT_SENSE_DDL", DICT_SENSE_DDL),
    ("DICT_EXAMPLE_DDL", DICT_EXAMPLE_DDL),
    ("DICT_CITATION_DDL", DICT_CITATION_DDL),
    ("CHAR_IDX_DDL", CHAR_IDX_DDL),
    ("ENTRY_INDEXES_DDL", ENTRY_INDEXES_DDL),
    ("ENTRY_FTS_DDL", ENTRY_FTS_DDL),
];

// ═════════════════════════════════════════════════════════════════════════════════
// Hạ tầng dùng chung
// ═════════════════════════════════════════════════════════════════════════════════

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

/// Một thư mục tạm **của riêng ca này** — khuôn `store_contract.rs:54`, không phát
/// minh bản thứ hai. `cargo test` chạy các ca song song trong cùng một tiến trình; hai ca
/// dùng chung một đường dẫn `.db` sẽ đỏ ngẫu nhiên và bị đọc thành flaky.
fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-dict-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));
    dir
}

/// ⚠️ Gọi **sau** khi `ReadOnlyDb` đã drop. Xem luật 2.
fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn sidecar(db: &Path, suffix: &str) -> PathBuf {
    let mut raw = db.as_os_str().to_owned();
    raw.push(suffix);
    PathBuf::from(raw)
}

/// Một đầu mục của fixture: `(id, source_id, lang, headword, headword_simp)`.
type Seed = (i64, i64, &'static str, &'static str, Option<&'static str>);

/// Dữ liệu fixture — **nhỏ mà đủ ba nhánh và đủ cả ba đối chứng âm**.
///
/// | id | đầu mục   | vai trò                                                        |
/// |----|-----------|----------------------------------------------------------------|
/// | 1  | `山`      | nhánh 2, một ký tự                                              |
/// | 2  | `中國`    | nhánh 1 và nhánh 2, hai ký tự; mang `headword_simp = 中国`       |
/// | 3  | `中國人`  | nhánh 3, ba ký tự — và một hàng khớp của truy vấn `中國`         |
/// | 4  | `國中`    | 🔴 **dương tính giả** của nhánh 2 khi tra `中國` (AC4)            |
/// | 5  | `國`      | cặp phồn/giản `國`/`国` — khoá vế `headword_simp` (Bẫy 8 của 1.9) |
/// | 6  | `高山`    | hàng thứ hai của truy vấn `山`, để "khác rỗng" không là "một"  |
/// | 7  | `lock`    | 🔴 `lang='en'` — đối chứng âm AC3, nhánh 1                        |
/// | 8  | `dictionary` | 🔴 `lang='en'` — đối chứng âm AC3, nhánh 3 qua truy vấn `dic`  |
///
/// **Story 1.11b bổ sung hai hàng — cả hai là đối chứng của một lỗ ĐO ĐƯỢC:**
///
/// | id | đầu mục   | vai trò                                                        |
/// |----|-----------|----------------------------------------------------------------|
/// | 9  | `running` | 🔴 đầu mục **chữ thường** — tra `Running` (chữ HOA đầu câu) phải ra nó. Đo thật trên `dict-core.db`: `headword = 'running'` ⇒ **1** hàng, `headword = 'Running'` ⇒ **0**. |
/// | 10 | `API`     | 🔴 đầu mục **chữ HOA có nghĩa** (**1.635** cái như thế) — hạ chữ thường là khoá **THÊM**, không phải khoá **THAY**. |
const SEEDS: &[Seed] = &[
    (1, 1, "zh", "山", None),
    (2, 1, "zh", "中國", Some("中国")),
    (3, 2, "zh", "中國人", Some("中国人")),
    (4, 1, "zh", "國中", Some("国中")),
    (5, 2, "zh", "國", Some("国")),
    (6, 2, "zh", "高山", None),
    (7, 1, "en", "lock", None),
    (8, 1, "en", "dictionary", None),
    (9, 1, "en", "running", None),
    (10, 2, "en", "API", None),
];

/// Bảy dải CJK, viết **đúng như văn bản nguồn** của
/// `tools/dict-build/src/char_idx.rs::is_han` — quần thể của cổng parity
/// [`han_ranges_are_verbatim_from_dict_build_char_idx`].
///
/// 🔴 Story 1.11b đã **xoá** bản sao `fn is_han` chỉ-BMP (3 dải) từng nằm ở đây. Bản sao
/// đó **đã lệch thật** so với bảy dải của build tool, và một định nghĩa lệch định tuyến
/// một truy vấn ngoài BMP sang đường tiếng Trung rồi tra vào một `char_idx` **chưa bao
/// giờ lập chỉ mục ký tự đó** ⇒ rỗng, không lỗi. Fixture nay gọi
/// `auratranslate_lib::core::dict::is_han` — **một** định nghĩa trong toàn `src-tauri/**`.
const HAN_RANGES: &[&str] = &[
    "0x3400..=0x4DBF",
    "0x4E00..=0x9FFF",
    "0xF900..=0xFAFF",
    "0x20000..=0x2A6DF",
    "0x2A700..=0x2EBEF",
    "0x2F800..=0x2FA1F",
    "0x30000..=0x3134F",
];

/// Dựng một tệp `.db` fixture và trả về đường dẫn của nó.
///
/// ⚠️ Fixture **không** đặt `journal_mode`; mặc định là `delete` — **giống hệt ba tệp
/// thật** (đã đo: `PRAGMA journal_mode` của cả ba = `delete`). Đặt WAL ở đây làm ca AC7
/// mất hết ý nghĩa, vì nó chính là chế độ mà đường đọc từ điển không được chạm tới.
fn build_fixture(dir: &Path) -> PathBuf {
    let path = dir.join("dict-fixture.db");
    let conn = rusqlite::Connection::open(&path)
        .unwrap_or_else(|e| panic!("dựng fixture {}: {e}", path.display()));

    for (name, ddl) in COPIED_DDL {
        conn.execute_batch(ddl)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }

    conn.execute_batch(
        "INSERT INTO dict_meta (key, value) VALUES ('schema_version', '1');
         INSERT INTO dict_source
           (id, code, display_name, license_kind, license_id, license_text,
            attribution, source_version, source_url)
         VALUES
           (1, 'fixture-alpha', 'Fixture Alpha', 'public-domain', NULL, 'x', 'x', '1', 'x'),
           (2, 'fixture-beta',  'Fixture Beta',  'public-domain', NULL, 'x', 'x', '1', 'x');",
    )
    .unwrap_or_else(|e| panic!("nạp dict_source: {e}"));

    for (id, source_id, lang, headword, simp) in SEEDS {
        conn.execute(
            "INSERT INTO dict_entry (id, source_id, lang, headword, headword_simp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, source_id, lang, headword, simp],
        )
        .unwrap_or_else(|e| panic!("nạp dict_entry {id}: {e}"));

        // ⚠️ `char_idx` chỉ sinh cho hàng `zh`, đúng như `tools/dict-build` làm — và phủ
        // **cả** `headword` **lẫn** `headword_simp` (Bẫy 8 của Story 1.9: phủ mỗi phồn
        // thể làm `国` trả rỗng mà không lỗi nào được ném).
        if *lang != "zh" {
            continue;
        }
        let mut seen = std::collections::HashSet::new();
        for c in headword.chars().chain(simp.unwrap_or("").chars()) {
            if is_han(c) && seen.insert(c) {
                conn.execute(
                    "INSERT OR IGNORE INTO char_idx (ch, entry_id) VALUES (?1, ?2)",
                    rusqlite::params![c.to_string(), id],
                )
                .unwrap_or_else(|e| panic!("nạp char_idx {c} / {id}: {e}"));
            }
        }
    }

    // 🔴 `entry_fts` là external-content ⇒ nó **không** tự đầy khi `dict_entry` được
    // nạp. Không có dòng này, nhánh 3 trả rỗng trên fixture và mọi ca của nó "xanh" theo
    // đúng cách sai nhất.
    conn.execute_batch("INSERT INTO entry_fts(entry_fts) VALUES('rebuild');")
        .unwrap_or_else(|e| panic!("rebuild entry_fts: {e}"));

    // ⚠️ Đóng kết nối dựng fixture **trước** khi bất kỳ `ReadOnlyDb` nào chạm vào tệp —
    // luật 2, và cũng là điều kiện để ca "tệp không đổi một byte" đo được thứ nó nói.
    conn.close()
        .unwrap_or_else(|(_, e)| panic!("đóng fixture: {e}"));

    path
}

/// Mở fixture qua đường sản phẩm.
fn open_fixture(path: &Path) -> ReadOnlyDb {
    ReadOnlyDb::open(path.to_path_buf(), StoreKind::Dict)
        .unwrap_or_else(|e| panic!("mở {}: {e:?}", path.display()))
}

/// Tra qua đường sản phẩm và trả về danh sách đầu mục đã khớp.
fn hits(db: &ReadOnlyDb, query: &str, mode: LookupMode, route: QueryRoute) -> Vec<String> {
    db.read(|conn| lookup(conn, query, mode, route, UNLIMITED))
        .unwrap_or_else(|e| panic!("tra {query:?}: {e:?}"))
        .hits
        .into_iter()
        .map(|hit| hit.headword)
        .collect()
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC1 — nhánh chọn bằng số KÝ TỰ
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **Ca đắt nhất của cả story**, và nó chạy mà không cần một tệp `.db` nào.
///
/// `"山".len()` là **3** và `"中國".len()` là **6** (UTF-8). Chọn nhánh theo `len()` đẩy
/// mọi truy vấn tiếng Trung 1–2 ký tự vào FTS5 trigram, nơi chúng trả **0** hàng trong
/// 0,01 ms mà không lỗi nào được ném — đúng nguyên văn lớp lỗi mà FR39 và AD-26 tồn
/// tại để chặn, và đúng thứ mũi thăm dò Giai đoạn 0 đã đo.
///
/// Ca này không được xoá.
#[test]
fn branch_is_picked_by_char_count_not_byte_length() {
    // Bằng chứng rằng cái bẫy là thật, không phải một lo xa.
    assert_eq!("山".len(), 3, "tiền đề của cả ca này");
    assert_eq!("中國".len(), 6, "tiền đề của cả ca này");

    assert_eq!(
        pick_branch("山", LookupMode::Substring, QueryRoute::Zh),
        QueryBranch::CharIdx,
        "một ký tự Hán đi vào `char_idx`, KHÔNG vào FTS5 trigram — `len()` là 3 nhưng \
         `chars().count()` là 1"
    );
    assert_eq!(
        pick_branch("中國", LookupMode::Substring, QueryRoute::Zh),
        QueryBranch::CharIdx,
        "hai ký tự Hán đi vào `char_idx` — `len()` là 6 nhưng `chars().count()` là 2"
    );
    assert_eq!(
        pick_branch("中國人", LookupMode::Substring, QueryRoute::Zh),
        QueryBranch::FtsTrigram
    );

    // Latin: cùng luật, và ở đây `len()` và `chars().count()` trùng nhau — nên ca này một
    // mình KHÔNG bắt được Bẫy 1. Nó có mặt để khẳng định ngưỡng là **2**, không phải để
    // khẳng định phép đo.
    assert_eq!(
        pick_branch("ab", LookupMode::Substring, QueryRoute::Zh),
        QueryBranch::CharIdx
    );
    assert_eq!(
        pick_branch("abc", LookupMode::Substring, QueryRoute::Zh),
        QueryBranch::FtsTrigram
    );
}

/// Tra chính xác không phụ thuộc độ dài — không có fallback dây chuyền (Quyết định #5).
#[test]
fn exact_mode_always_takes_the_btree_branch() {
    for query in ["山", "中國", "中國人", "a", "abcdefgh"] {
        assert_eq!(
            pick_branch(query, LookupMode::Exact, QueryRoute::Zh),
            QueryBranch::ExactBtree,
            "truy vấn {query:?} ở chế độ Exact"
        );
    }
}

/// Nhánh đã đi **quan sát được từ ngoài** — AC1 vế cuối. Một `eprintln!` không khẳng
/// định được trong test, nên nó không nghiệm thu được AC1.
#[test]
fn the_branch_that_ran_is_part_of_the_returned_value() {
    let dir = temp_dir("branch-observable");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        let one = db
            .read(|c| lookup(c, "山", LookupMode::Substring, QueryRoute::Zh, UNLIMITED))
            .unwrap();
        let three = db
            .read(|c| lookup(c, "中國人", LookupMode::Substring, QueryRoute::Zh, UNLIMITED))
            .unwrap();
        let exact = db
            .read(|c| lookup(c, "中國", LookupMode::Exact, QueryRoute::Zh, UNLIMITED))
            .unwrap();

        assert_eq!(one.branch, QueryBranch::CharIdx);
        assert_eq!(three.branch, QueryBranch::FtsTrigram);
        assert_eq!(exact.branch, QueryBranch::ExactBtree);
    }

    cleanup(&dir);
}

/// Truy vấn RỖNG ở chế độ `Substring` ⇒ khác rỗng thành **rỗng**, không panic, không
/// `Err` — nhánh `char_idx::char_idx()` có một `else` tường minh cho 0 ký tự, và ca này
/// khoá đúng nhánh đó thay vì để nó chỉ được nói tới trong doc-comment.
#[test]
fn an_empty_substring_query_returns_no_rows() {
    let dir = temp_dir("empty-query");
    let path = build_fixture(&dir);
    {
        let db = open_fixture(&path);
        assert_eq!(
            hits(&db, "", LookupMode::Substring, QueryRoute::Zh),
            Vec::<String>::new()
        );
    }
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC2 — ba truy vấn mốc khác rỗng, cộng ĐỐI CHỨNG ÂM
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_one_character_query_returns_rows() {
    let dir = temp_dir("one-char");
    let path = build_fixture(&dir);
    {
        let db = open_fixture(&path);
        let mut found = hits(&db, "山", LookupMode::Substring, QueryRoute::Zh);
        found.sort();
        assert_eq!(found, vec!["山".to_owned(), "高山".to_owned()]);
    }
    cleanup(&dir);
}

#[test]
fn a_two_character_query_returns_rows() {
    let dir = temp_dir("two-char");
    let path = build_fixture(&dir);
    {
        let db = open_fixture(&path);
        let mut found = hits(&db, "中國", LookupMode::Substring, QueryRoute::Zh);
        found.sort();
        assert_eq!(found, vec!["中國".to_owned(), "中國人".to_owned()]);
    }
    cleanup(&dir);
}

#[test]
fn a_three_character_query_returns_rows() {
    let dir = temp_dir("three-char");
    let path = build_fixture(&dir);
    {
        let db = open_fixture(&path);
        assert_eq!(
            hits(&db, "中國人", LookupMode::Substring, QueryRoute::Zh),
            vec!["中國人".to_owned()]
        );
    }
    cleanup(&dir);
}

/// 🔴 **Đối chứng âm của AC2** — và nó là **bằng chứng dương** rằng nhánh 2 phải tồn tại.
///
/// FTS5 với tokenizer `trigram` không lập chỉ mục cho token ngắn hơn **ba** ký tự, nên
/// một truy vấn 1–2 ký tự khớp **0** hàng. Đo được nguyên như thế trên tệp thật:
/// `entry_fts MATCH '"山"'` ⇒ 0, `entry_fts MATCH '"中國"'` ⇒ 0.
///
/// Ca này không được xoá vì "nó khẳng định một thứ hỏng". Nó khẳng định đúng cái
/// khiếm khuyết mà AD-26 dựng ba nhánh để đi vòng qua; xoá nó là xoá lý do tồn tại của
/// nhánh 2, và người sửa tiếp theo sẽ gộp hai nhánh lại.
#[test]
fn fts_returns_nothing_for_one_and_two_character_queries() {
    let dir = temp_dir("fts-negative");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        for query in ["山", "中國"] {
            let phrase = format!("\"{query}\"");
            let count: i64 = db
                .read(|conn| {
                    conn.query_row(
                        "SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH ?1",
                        [&phrase],
                        |row| row.get(0),
                    )
                })
                .unwrap_or_else(|e| panic!("MATCH {phrase}: {e:?}"));

            assert_eq!(
                count, 0,
                "`entry_fts MATCH {phrase}` trả {count} hàng. Nếu con số này khác 0 thì \
                 tokenizer đã đổi, và ngưỡng 2 ký tự của `pick_branch` phải được đo lại \
                 chứ KHÔNG phải điều chỉnh cho khớp ca này."
            );
        }

        // Đối chứng dương: cùng bảng đó **có** trả hàng cho ba ký tự — nếu không, ca trên
        // xanh y hệt trên một `entry_fts` rỗng vì `rebuild` bị quên.
        let count: i64 = db
            .read(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH '\"中國人\"'",
                    [],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert!(count > 0, "`entry_fts` rỗng — `rebuild` đã không chạy");
    }

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC3 — mọi nhánh lọc `lang = 'zh'`
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC3**, và nó **chỉ đo được bằng truy vấn LATIN.**
///
/// Với truy vấn thuần Hán, rò rỉ đo được trên tệp thật là **0** — trigram Latin không
/// khớp trigram Hán. Với truy vấn Latin thì rò rỉ là **thật và lớn**: `headword = 'lock'`
/// cho **1** hàng và `entry_fts MATCH '"dic"'` cho **572** hàng, **100%** `lang='en'`.
///
/// Fixture mang đúng hai hàng đó, nên ca này đỏ ngay khi một nhánh bỏ mệnh đề `lang`.
#[test]
fn every_branch_filters_out_english_entries() {
    let dir = temp_dir("lang-filter");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);

        assert!(
            hits(&db, "lock", LookupMode::Exact, QueryRoute::Zh).is_empty(),
            "nhánh 1 trả về một đầu mục `lang='en'` — nó sẽ lên giao diện DÁN NHÃN kết \
             quả tiếng Trung"
        );
        assert!(
            hits(&db, "dic", LookupMode::Substring, QueryRoute::Zh).is_empty(),
            "nhánh 3 trả về một đầu mục `lang='en'`"
        );

        // Đối chứng dương: hai hàng đó **có thật trong fixture**. Không có phép kiểm này,
        // ca trên xanh y hệt trên một fixture không có hàng tiếng Anh nào — tức nó
        // không kiểm gì cả.
        let english: i64 = db
            .read(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM dict_entry WHERE lang = 'en'",
                    [],
                    |row| row.get(0),
                )
            })
            .unwrap();
        // ⚠️ Con số này là **quần thể fixture**, không phải một mệnh đề của Story 1.11:
        // 1.11b thêm `running` và `API` (id 9, 10) để nghiệm thu đường tiếng Anh, nên nó
        // đi từ 2 lên 4. Ý nghĩa của phép kiểm không đổi — *"fixture CÓ hàng `lang='en'`
        // thật, nên hai phép khẳng định `is_empty()` ở trên có việc để làm"*.
        assert_eq!(english, 4, "fixture phải mang đúng bốn hàng `lang='en'`");
    }

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC4 — chuỗi con phải được XÁC MINH LẠI
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC4** — `char_idx` trả lời *"chứa cả hai ký tự"*, không trả lời *"chứa hai ký
/// tự ĐÓ LIỀN NHAU"*.
///
/// `國中` chứa cả `中` lẫn `國` nên nó **là** một ứng viên của `INTERSECT`, và nó không
/// **không** được có mặt trong kết quả. Đo trên tệp thật: `中國` cho **390** ứng viên →
/// **350** sau xác minh ⇒ **40** dương tính giả bị loại.
///
/// ⚠️ Ca này không phát biểu được bằng `> 0`: bản bỏ bước xác minh trả **nhiều hơn**,
/// không phải rỗng. Người dùng tra *"Trung Quốc"* nhận về *"trong trường"*.
#[test]
fn char_idx_candidates_are_verified_as_real_substrings() {
    let dir = temp_dir("verify-substring");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        let found = hits(&db, "中國", LookupMode::Substring, QueryRoute::Zh);

        assert!(
            !found.contains(&"國中".to_owned()),
            "`國中` lọt vào kết quả của `中國` — bước xác minh chuỗi con đã bị bỏ. \
             Kết quả: {found:?}"
        );
        assert!(
            found.contains(&"中國人".to_owned()),
            "`中國人` phải có mặt — nó chứa `中國` liền nhau. Kết quả: {found:?}"
        );

        // Đối chứng dương cho chính phép lọc: `國中` **là** một ứng viên của `INTERSECT`,
        // tức bước xác minh có việc thật để làm. Không có phép kiểm này, ca trên xanh y
        // hệt trên một `char_idx` dựng sai (phép HỢP thay vì phép GIAO trả `國中` luôn,
        // phép GIAO thiếu dữ liệu thì không trả gì cả).
        let candidates: i64 = db
            .read(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM dict_entry e WHERE e.lang = 'zh' AND e.id IN (
                       SELECT entry_id FROM char_idx WHERE ch = '中'
                       INTERSECT SELECT entry_id FROM char_idx WHERE ch = '國')",
                    [],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert_eq!(
            candidates, 3,
            "ứng viên `INTERSECT` phải là 中國 · 中國人 · 國中 — không phải {candidates}"
        );
        assert_eq!(found.len(), 2, "sau xác minh còn đúng hai: {found:?}");
    }

    cleanup(&dir);
}

/// Bẫy 8 của Story 1.9 — vế `headword_simp` không bỏ được ở **cả** `char_idx` **lẫn**
/// bước xác minh. Bỏ một trong hai làm `国` trả rỗng trong 0,01 ms, không lỗi nào ném.
#[test]
fn simplified_headwords_are_reachable() {
    let dir = temp_dir("simplified");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        let one_char = hits(&db, "国", LookupMode::Substring, QueryRoute::Zh);
        assert!(
            !one_char.is_empty(),
            "tra `国` (giản thể) trả rỗng — `char_idx` hoặc bước xác minh đã bỏ vế \
             `headword_simp`"
        );

        // Hai ký tự giản thể: đi qua `INTERSECT` **và** qua bước xác minh, nên nó khoá vế
        // `headword_simp` ở đúng chỗ mà ca một-ký-tự không chạm tới (một ký tự không
        // xác minh — AC4 mệnh đề cuối).
        let two_char = hits(&db, "中国", LookupMode::Substring, QueryRoute::Zh);
        assert!(
            two_char.contains(&"中國".to_owned()),
            "tra `中国` phải ra đầu mục `中國` qua `headword_simp`. Kết quả: {two_char:?}"
        );
    }

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC6 — kết quả mang `source_code`, không mang `source_id`
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC6** — `id = 1` tồn tại ở **cả ba** tệp `.db` và trỏ ba nguồn khác nhau, nên
/// khoá theo `id` dán nhãn sai ngay khi Story 1.13 gom nhiều tệp. Fixture mang hai nguồn
/// để phép khẳng định *"phân biệt được bằng chuỗi"* có nghĩa.
#[test]
fn results_carry_the_source_code_not_the_id() {
    let dir = temp_dir("source-code");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        let result = db
            .read(|conn| lookup(conn, "山", LookupMode::Substring, QueryRoute::Zh, UNLIMITED))
            .unwrap();

        assert!(!result.hits.is_empty());
        for hit in &result.hits {
            assert!(
                !hit.source_code.is_empty(),
                "đầu mục {} không mang `source_code`",
                hit.headword
            );
            assert_eq!(
                hit.lang, "zh",
                "đầu mục {} mang `lang` khác 'zh' dù đã đi qua bộ lọc `lang = 'zh'`",
                hit.headword
            );
        }

        let codes: std::collections::BTreeSet<&str> =
            result.hits.iter().map(|h| h.source_code.as_str()).collect();
        assert_eq!(
            codes,
            ["fixture-alpha", "fixture-beta"].into_iter().collect(),
            "hai nguồn của fixture phải phân biệt được **bằng chuỗi**, không bằng số"
        );
    }

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bẫy 4 — `entry_fts MATCH` và cú pháp truy vấn FTS5
// ═════════════════════════════════════════════════════════════════════════════════

/// Truy vấn chứa ký tự có nghĩa trong **cú pháp FTS5** ⇒ `Ok`, không `Err`.
///
/// Không bọc ngoặc kép, một truy vấn chứa `*` `-` `^` `(` `:` hay từ `NEAR` làm SQLite trả
/// `SQLITE_ERROR` — tức **tra cứu báo lỗi vì nội dung người dùng bôi đen**. Tệ hơn hẳn trả
/// rỗng, và nó chỉ lộ ra ở tay người dùng thật chứ không ở CI, nơi fixture chỉ có chữ
/// Hán sạch.
#[test]
fn an_fts_query_with_syntax_characters_does_not_error() {
    let dir = temp_dir("fts-syntax");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        for query in ["a*b", "a-b-c", "a\"b\"c", "x(y):z", "NEAR foo", "^abc"] {
            let outcome =
                db.read(|conn| lookup(conn, query, LookupMode::Substring, QueryRoute::Zh, UNLIMITED));
            assert!(
                outcome.is_ok(),
                "truy vấn {query:?} làm tra cứu BÁO LỖI thay vì trả rỗng: {:?}",
                outcome.err()
            );
        }
    }

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC7 — chỉ đọc, và không một byte nào bị ghi
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC7 mệnh đề 1 và 2** — mở, tra, đóng ⇒ tệp **y hệt từng byte**, và không tệp
/// `-wal`/`-shm` nào xuất hiện cạnh nó.
///
/// ⚠️ So **nội dung tệp** bằng `std::fs::read` thay vì băm SHA-256: tương đương về mặt
/// khẳng định, rẻ hơn, và không thêm một crate nào (fixture chỉ vài KB).
///
/// Vì sao mệnh đề `-wal` là bắt buộc chứ không phải sở thích: `PRAGMA journal_mode = WAL`
/// **GHI VÀO** database ⇒ SHA-256 đổi ⇒ `dict-manifest.toml` thành sai ⇒ AD-25 vỡ, và không
/// không cổng nào bắt (`check-dict-manifest.mjs` cố ý không đọc `.db`).
#[test]
fn opening_a_dictionary_leaves_the_file_byte_identical() {
    let dir = temp_dir("byte-identical");
    let path = build_fixture(&dir);

    let before = fs::read(&path).expect("đọc fixture trước khi mở");

    {
        let db = open_fixture(&path);
        // Chạy cả ba nhánh — nếu một nhánh nào đó ghi, ca này phải thấy.
        let _ = hits(&db, "山", LookupMode::Substring, QueryRoute::Zh);
        let _ = hits(&db, "中國", LookupMode::Substring, QueryRoute::Zh);
        let _ = hits(&db, "中國人", LookupMode::Substring, QueryRoute::Zh);
        let _ = hits(&db, "中國", LookupMode::Exact, QueryRoute::Zh);
        db.close();
    }

    let after = fs::read(&path).expect("đọc fixture sau khi đóng");
    assert_eq!(
        before.len(),
        after.len(),
        "cỡ tệp từ điển đã đổi — một thứ gì đó trên đường đọc đã GHI vào nó"
    );
    assert!(
        before == after,
        "nội dung tệp từ điển đã đổi. AD-25: tệp đi kèm checksum trong \
         `dict-manifest.toml`; ghi vào nó một byte là làm checksum thành sai, và không \
         cổng nào bắt được điều đó."
    );

    for suffix in ["-wal", "-shm", "-journal"] {
        let side = sidecar(&path, suffix);
        assert!(
            !side.exists(),
            "tệp `{}` đã xuất hiện — đường đọc từ điển KHÔNG được chạm `journal_mode`",
            side.display()
        );
    }

    cleanup(&dir);
}

/// 🔴 **AC7 mệnh đề 3** — đường dẫn không tồn tại ⇒ `Err`, và **không tệp rỗng nào
/// được tạo ra**.
///
/// Với `SQLITE_OPEN_CREATE`, một đường dẫn gõ sai (hoặc một tệp `$RESOURCE` chưa được
/// đóng gói) không trả lỗi: SQLite dựng một tệp rỗng, mọi truy vấn sau đó trả rỗng, không
/// không lỗi nào được ném, và người dùng chỉ thấy *"tra từ không ra kết quả"*.
#[test]
fn opening_a_missing_dictionary_fails_and_creates_nothing() {
    let dir = temp_dir("missing");
    let path = dir.join("khong-ton-tai.db");

    let outcome = ReadOnlyDb::open(path.clone(), StoreKind::Dict);
    assert!(
        outcome.is_err(),
        "mở một đường dẫn không tồn tại phải trả `Err`"
    );
    assert!(
        !path.exists(),
        "một tệp rỗng đã được tạo ra tại {} — cờ mở còn `SQLITE_OPEN_CREATE`",
        path.display()
    );

    cleanup(&dir);
}

/// **Bằng chứng dương của `query_only = 1`** — cùng khuôn `Store::read`.
///
/// Chỉ-đọc ở đây là cưỡng chế của **SQLite**, không phải kỷ luật của người viết: một
/// `INSERT` qua đường này **thất bại**, với lỗi của SQLite.
#[test]
fn a_write_through_the_dictionary_handle_is_refused() {
    let dir = temp_dir("write-refused");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        let outcome = db.read(|conn| {
            conn.execute(
                "INSERT INTO dict_entry (id, source_id, lang, headword)
                 VALUES (999, 1, 'zh', 'X')",
                [],
            )
        });
        assert!(
            outcome.is_err(),
            "một `INSERT` qua `ReadOnlyDb::read` đã THÀNH CÔNG — `query_only` hoặc cờ \
             `SQLITE_OPEN_READ_ONLY` đã biến mất"
        );
    }

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.11b · AC6 — tập khoá {nguyên văn, hạ chữ thường} trong MỘT truy vấn
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC6, và đây là lỗ mà cả AD-44 ③ tồn tại để bịt.**
///
/// Đo thật trên `dict-core.db`: `headword = 'running'` ⇒ **1** hàng,
/// `headword = 'Running'` ⇒ **0**. Bôi đen một từ ở **đầu câu** là thao tác thường ngày,
/// và không có khoá thứ hai nó trả **rỗng**, **không lỗi**.
#[test]
fn an_uppercase_english_query_still_finds_a_lowercase_headword() {
    let dir = temp_dir("en-exact-upper");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        assert_eq!(
            hits(&db, "Running", LookupMode::Exact, QueryRoute::En),
            vec!["running".to_owned()],
            "tra `Running` KHÔNG ra `running` — tập khoá chỉ có một phần tử"
        );
    }

    cleanup(&dir);
}

/// 🔴 **AC6 vế "THÊM một khoá, không THAY khoá gốc".**
///
/// **1.635** đầu mục tiếng Anh mang chữ hoa **có nghĩa** (`API` · `Wikipedia` ·
/// `English`). Một cài đặt hạ chữ thường **thay** khoá gốc làm cả 1.635 cái đó biến mất.
#[test]
fn an_uppercase_headword_is_still_reachable_by_its_own_spelling() {
    let dir = temp_dir("en-exact-acronym");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        assert_eq!(
            hits(&db, "API", LookupMode::Exact, QueryRoute::En),
            vec!["API".to_owned()],
            "tra `API` KHÔNG ra `API` — khoá nguyên văn đã bị THAY bằng dạng hạ chữ \
             thường thay vì được GIỮ bên cạnh nó"
        );
    }

    cleanup(&dir);
}

/// 🔴 **AC6 mệnh đề cuối — BẤT ĐỐI XỨNG CÓ CHỦ Ý, và ca này ghi lại nó.**
///
/// `Running` ⇒ `running` (**có**). `api` ⇒ `API` (**không**). Hạ chữ thường xảy ra
/// phía **truy vấn**, **không** phía **đầu mục**.
///
/// **Đừng "sửa" ca này.** Khớp hai chiều đòi một **chỉ mục hàm `lower(headword)` lúc
/// build** ⇒ đổi `tools/dict-build/src/schema.rs`, dựng lại `dict-core.db`, điền lại
/// `[base].sha256` của `dict-manifest.toml`, đo lại NFR6, và làm **184** nhóm đầu mục
/// *(chỉ phân biệt nhau bằng chữ hoa)* **sập vào nhau**. Đó là một quyết định **tầng
/// PRD/kiến trúc**, không phải một lượt vá ở tầng story.
#[test]
fn lowercasing_happens_on_the_query_never_on_the_headword() {
    let dir = temp_dir("en-exact-asymmetry");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        assert_eq!(
            hits(&db, "api", LookupMode::Exact, QueryRoute::En),
            Vec::<String>::new(),
            "tra `api` đã ra `API` — phép hạ chữ thường đã lan sang phía ĐẦU MỤC. Hệ quả: \
             184 nhóm đầu mục chỉ phân biệt nhau bằng chữ hoa sẽ sập vào nhau."
        );

        // Đối chứng dương cho chính ca này: `API` **có thật** trong fixture, nên phép
        // khẳng định rỗng ở trên có việc để làm. Không có nó, ca xanh y hệt trên một
        // fixture không có đầu mục chữ hoa nào.
        assert_eq!(
            hits(&db, "API", LookupMode::Exact, QueryRoute::En),
            vec!["API".to_owned()]
        );
    }

    cleanup(&dir);
}

/// **AC6 vế "MỘT truy vấn"** — `IN (?1, ?2)` trả mỗi hàng **đúng một lần** kể cả khi hai
/// khoá trùng nhau.
///
/// ⚠️ Ca này bắt đúng thứ một `UNION ALL` sẽ hỏng: tra một đầu mục **vốn đã chữ thường**
/// làm tập khoá thành `{running, running}`, và một phép hợp không khử trùng sẽ trả
/// **hai** hàng cho **một** đầu mục — người dùng thấy cùng một từ hai lần, không lỗi
/// nào được ném.
#[test]
fn a_query_that_is_already_lowercase_returns_each_row_exactly_once() {
    let dir = temp_dir("en-exact-dupe");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        assert_eq!(
            hits(&db, "running", LookupMode::Exact, QueryRoute::En),
            vec!["running".to_owned()],
            "một đầu mục trả về NHIỀU HƠN một lần — tập khoá `{{running, running}}` đã đi \
             qua một phép HỢP không khử trùng thay vì `IN (?1, ?2)`"
        );
    }

    cleanup(&dir);
}

/// **[Review fix]** — chuỗi con tiếng Anh lệch hoa/thường vẫn khớp: `verify_substring`
/// từng phân biệt hoa/thường trong khi tokenizer `trigram` của FTS5 thì không, nên một
/// truy vấn `api` (thường) tìm ứng viên `API` (hoa) qua FTS5 rồi bị chính bước xác minh
/// loại mất — rỗng im lặng, đúng lớp lỗi AD-26 cấm. `API` có id 10 trong `SEEDS`.
#[test]
fn an_english_substring_query_matches_a_headword_of_different_case() {
    let dir = temp_dir("en-trigram-case");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        let result = db
            .read(|conn| lookup(conn, "api", LookupMode::Substring, QueryRoute::En, UNLIMITED))
            .unwrap();

        assert_eq!(result.branch, QueryBranch::FtsTrigram);
        assert_eq!(
            result
                .hits
                .iter()
                .map(|h| h.headword.as_str())
                .collect::<Vec<_>>(),
            vec!["API"],
            "tra chuỗi con `api` (thường) KHÔNG ra đầu mục `API` (hoa) — xác minh chuỗi \
             con đang phân biệt hoa/thường trong khi FTS5 trigram thì không"
        );
    }

    cleanup(&dir);
}

/// **AC5 + AC6** — nhánh chuỗi con tiếng Anh chạy qua FTS5 `trigram`, và nhánh **được
/// trả về** đúng như nó đã chạy.
#[test]
fn an_english_substring_query_of_three_characters_uses_the_trigram_branch() {
    let dir = temp_dir("en-trigram");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        let result = db
            .read(|conn| lookup(conn, "dic", LookupMode::Substring, QueryRoute::En, UNLIMITED))
            .unwrap();

        assert_eq!(result.branch, QueryBranch::FtsTrigram);
        assert_eq!(
            result
                .hits
                .iter()
                .map(|h| h.headword.as_str())
                .collect::<Vec<_>>(),
            vec!["dictionary"],
            "tra `dic` trên đường En KHÔNG ra `dictionary`"
        );
        for hit in &result.hits {
            assert_eq!(
                hit.lang, "en",
                "đầu mục {} mang `lang` khác 'en' dù đã đi qua bộ lọc `lang = 'en'`",
                hit.headword
            );
        }
    }

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.11b · AC7 — < 3 ký tự: KHÔNG HỖ TRỢ, và không một câu SQL nào chạy
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC7** — `NoBranchQueryTooShort`, `hits` rỗng, và trạng thái đó **phân biệt được**
/// với *"đã tra mà không tìm thấy gì"*.
///
/// **Rỗng im lặng bị cấm; rỗng có lý do thì không.** Panel Lookup (Story 1.17) nói *"truy
/// vấn quá ngắn"* cho trạng thái này và *"không tìm thấy"* cho trạng thái kia — hai câu
/// dẫn người dùng đi hai đường, nên hai trạng thái không được nhập một.
#[test]
fn a_short_english_substring_query_reports_not_supported_not_no_results() {
    let dir = temp_dir("en-too-short");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);

        // Ca 0 ký tự đi CÙNG đường với 1–2 ký tự — vị từ độ dài là MỘT mệnh đề
        // `chars().count() < 3`, không phải hai mệnh đề với một ca đặc biệt ở giữa.
        for query in ["", "l", "lo"] {
            let result = db
                .read(|conn| lookup(conn, query, LookupMode::Substring, QueryRoute::En, UNLIMITED))
                .unwrap_or_else(|e| panic!("tra {query:?}: {e:?}"));

            assert_eq!(
                result.branch,
                QueryBranch::NoBranchQueryTooShort,
                "truy vấn {query:?} khai nhánh {:?} — nó đã TRÀN qua một nhánh khác, và \
                 `branch` nay NÓI DỐI về đường đã đi",
                result.branch
            );
            assert!(result.hits.is_empty(), "truy vấn {query:?} trả hàng");
        }

        // 🔴 **Phân biệt được**: cùng chế độ, cùng đường, một truy vấn ĐỦ DÀI mà không
        // khớp gì trả về một nhánh KHÁC với `hits` cũng rỗng. Không có phép so này, hai
        // trạng thái đọc giống hệt nhau ở phía chỗ gọi.
        let ran = db
            .read(|conn| lookup(conn, "zzz", LookupMode::Substring, QueryRoute::En, UNLIMITED))
            .unwrap();
        assert_eq!(ran.branch, QueryBranch::FtsTrigram);
        assert!(ran.hits.is_empty());
        assert_ne!(
            ran.branch,
            QueryBranch::NoBranchQueryTooShort,
            "*\"đã tra, không thấy gì\"* và *\"quá ngắn để tra\"* KHÔNG được đọc \
             giống nhau từ ngoài"
        );
    }

    cleanup(&dir);
}

/// 🔴 **AC7 vế "không chạm database"** — và nó **quan sát được**, không phải một lời
/// hứa trong doc-comment.
///
/// Cách đo: mở một tệp `.db` hợp lệ nhưng **không có** một bảng từ điển nào. Một truy
/// vấn quá ngắn phải trả `Ok`; một truy vấn đủ dài — cùng chế độ, cùng đường — phải trả
/// `Err` vì câu SQL của nó **được chuẩn bị thật** và không tìm thấy bảng.
///
/// ⚠️ Vế `Err` là **đối chứng dương** và nó không bỏ được: không có nó, ca này xanh y
/// hệt trên một cài đặt không bao giờ chạm database ở **bất kỳ** nhánh nào.
#[test]
fn a_too_short_english_query_prepares_no_sql_at_all() {
    let dir = temp_dir("en-no-sql");
    let path = dir.join("no-dict-tables.db");

    {
        let conn = rusqlite::Connection::open(&path)
            .unwrap_or_else(|e| panic!("dựng {}: {e}", path.display()));
        // Một bảng bất kỳ, chỉ để tệp là một database SQLite hợp lệ và không rỗng.
        conn.execute_batch("CREATE TABLE marker (x INTEGER);")
            .unwrap_or_else(|e| panic!("dựng marker: {e}"));
        conn.close().unwrap_or_else(|(_, e)| panic!("đóng: {e}"));
    }

    {
        let db = open_fixture(&path);

        for query in ["", "l", "lo"] {
            let outcome =
                db.read(|conn| lookup(conn, query, LookupMode::Substring, QueryRoute::En, UNLIMITED));
            let result = outcome.unwrap_or_else(|e| {
                panic!(
                    "truy vấn {query:?} đã CHUẨN BỊ một câu SQL trên một database không \
                     có bảng từ điển nào: {e:?}. AD-44 ④ nói nhánh này KHÔNG chạm \
                     database."
                )
            });
            assert_eq!(result.branch, QueryBranch::NoBranchQueryTooShort);
            assert!(result.hits.is_empty());
        }

        // Đối chứng dương: một truy vấn ĐỦ DÀI **có** chạm database, nên nó đỏ ở đây.
        let outcome = db.read(|conn| lookup(conn, "dic", LookupMode::Substring, QueryRoute::En, UNLIMITED));
        assert!(
            outcome.is_err(),
            "một truy vấn 3 ký tự KHÔNG chạm database — vậy thì ca ở trên không kiểm \
             gì cả, vì không nhánh nào chạm database"
        );
    }

    cleanup(&dir);
}

/// 🔴 **AC7 mệnh đề cuối** — **không hạ ngưỡng trigram xuống 1.**
///
/// FTS5 với tokenizer `trigram` **không** lập chỉ mục token ngắn hơn ba ký tự. Đo được
/// nguyên như thế: `entry_fts MATCH '"lo"'` ⇒ **0** hàng, dù `lock` có trong fixture.
/// Để một truy vấn 1–2 ký tự chạy nhánh trigram là để nó trả **rỗng im lặng** — đúng thứ
/// `NoBranchQueryTooShort` sinh ra để thay thế.
#[test]
fn fts_returns_nothing_for_short_latin_queries_too() {
    let dir = temp_dir("en-fts-negative");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        for query in ["l", "lo"] {
            let phrase = format!("\"{query}\"");
            let count: i64 = db
                .read(|conn| {
                    conn.query_row(
                        "SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH ?1",
                        [&phrase],
                        |row| row.get(0),
                    )
                })
                .unwrap_or_else(|e| panic!("MATCH {phrase}: {e:?}"));

            assert_eq!(
                count, 0,
                "`entry_fts MATCH {phrase}` trả {count} hàng. Nếu con số này khác 0 thì \
                 tokenizer đã đổi, và ngưỡng < 3 của đường tiếng Anh phải được ĐO LẠI chứ \
                 KHÔNG phải điều chỉnh cho khớp ca này."
            );
        }

        // Đối chứng dương: ba ký tự **có** khớp — nếu không, ca trên xanh y hệt trên một
        // `entry_fts` rỗng vì `rebuild` bị quên.
        let count: i64 = db
            .read(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH '\"loc\"'",
                    [],
                    |row| row.get(0),
                )
            })
            .unwrap();
        assert!(count > 0, "`entry_fts` rỗng — `rebuild` đã không chạy");
    }

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.11b · AC8 — mọi nhánh tiếng Anh lọc `lang = 'en'` TƯỜNG MINH
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC8**, và nó **chỉ đo được vì `route` là một THAM SỐ.**
///
/// `pick_route` **không bao giờ** sinh tổ hợp `(truy vấn Hán, route = En)` — nhưng test
/// **ép được** nó, và chỉ khi ép được thì bộ lọc `lang = 'en'` mới trở thành thứ **nghiệm
/// thu được** thay vì thứ *"chắc là đúng vì đầu vào không bao giờ tới đó"*. Đó là lý do
/// thứ ba trong ba lý do `route` không phải một phép đoán bên trong `lookup`.
///
/// Cả **hai** nhánh tiếng Anh đều bị ép, vì cả hai đều mang một câu SQL riêng.
#[test]
fn both_english_branches_filter_out_chinese_entries() {
    let dir = temp_dir("en-lang-filter");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);

        // ── Nhánh tra chính xác ──────────────────────────────────────────────────
        assert!(
            hits(&db, "中國", LookupMode::Exact, QueryRoute::En).is_empty(),
            "`exact_en` trả về một đầu mục `lang='zh'` — vế `AND e.lang = 'en'` đã biến \
             mất, và kết quả sẽ lên giao diện DÁN NHÃN sai ngôn ngữ"
        );

        // Đối chứng dương của AC8: **≥ 2** hàng `lang='zh'` mà đúng truy vấn đó khớp khi
        // `route = Zh`. Không có phép kiểm này, ca trên xanh y hệt trên một fixture không
        // không có hàng tiếng Trung nào.
        let mut zh = hits(&db, "中國", LookupMode::Substring, QueryRoute::Zh);
        zh.sort();
        assert_eq!(zh, vec!["中國".to_owned(), "中國人".to_owned()]);

        // ── Nhánh trigram ────────────────────────────────────────────────────────
        let forced = db
            .read(|conn| lookup(conn, "中國人", LookupMode::Substring, QueryRoute::En, UNLIMITED))
            .unwrap();
        assert_eq!(
            forced.branch,
            QueryBranch::FtsTrigram,
            "truy vấn 3 ký tự bị ép sang đường En phải CHẠY nhánh trigram — nếu nó rẽ đi \
             chỗ khác thì ca này không kiểm được bộ lọc `lang`"
        );
        assert!(
            forced.hits.is_empty(),
            "`fts_trigram_en` trả về {} đầu mục `lang='zh'` — vế `AND e.lang = 'en'` đã \
             biến mất. `entry_fts` lập chỉ mục trigram trên MỌI hàng, cả zh lẫn en.",
            forced.hits.len()
        );

        // Đối chứng dương: cùng truy vấn, cùng chế độ, đường `Zh` ⇒ khác rỗng.
        assert_eq!(
            hits(&db, "中國人", LookupMode::Substring, QueryRoute::Zh),
            vec!["中國人".to_owned()]
        );
    }

    cleanup(&dir);
}

/// **Bẫy 3, phía tiếng Anh** — cú pháp FTS5 và chuỗi Latin.
///
/// ⚠️ **Rủi ro CAO HƠN NHIỀU với tiếng Anh** so với tiếng Trung: một truy vấn Latin dễ
/// chứa `'`, `-`, `*`, `:` (`don't`, `state-of-the-art`) trong khi fixture tiếng Trung
/// chỉ có chữ Hán sạch. Không bọc cụm ⇒ SQLite trả `SQLITE_ERROR`, tức **tra cứu báo lỗi
/// vì nội dung người dùng bôi đen**.
#[test]
fn an_english_fts_query_with_syntax_characters_does_not_error() {
    let dir = temp_dir("en-fts-syntax");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);
        for query in [
            "don't",
            "state-of-the-art",
            "a*b",
            "NEAR foo",
            "x(y):z",
            "a\"b\"c",
            "^abc",
        ] {
            let outcome =
                db.read(|conn| lookup(conn, query, LookupMode::Substring, QueryRoute::En, UNLIMITED));
            assert!(
                outcome.is_ok(),
                "truy vấn {query:?} làm tra cứu BÁO LỖI thay vì trả rỗng: {:?}",
                outcome.err()
            );
        }
    }

    cleanup(&dir);
}

/// **AC11** — `lang` là một **TRƯỜNG**, không phải một **KIỂU**, và cả hai đường dùng
/// **cùng một** [`auratranslate_lib::core::dict::EntryHit`].
///
/// Không có bản ghi kết quả thứ hai cho tiếng Anh (AD-44 ⑤), nên ca này khẳng định
/// được bằng **cùng một** kiểu cho cả hai lượt tra — và mọi hit vẫn mang `source_code`
/// dạng **chuỗi**, không `source_id`.
#[test]
fn both_routes_return_the_same_record_shape() {
    let dir = temp_dir("en-record-shape");
    let path = build_fixture(&dir);

    {
        let db = open_fixture(&path);

        let zh = db
            .read(|conn| lookup(conn, "山", LookupMode::Substring, QueryRoute::Zh, UNLIMITED))
            .unwrap();
        let en = db
            .read(|conn| lookup(conn, "API", LookupMode::Exact, QueryRoute::En, UNLIMITED))
            .unwrap();

        assert!(!zh.hits.is_empty() && !en.hits.is_empty());

        for (hit, expected_lang) in zh
            .hits
            .iter()
            .map(|h| (h, "zh"))
            .chain(en.hits.iter().map(|h| (h, "en")))
        {
            assert_eq!(hit.lang, expected_lang);
            assert!(
                !hit.source_code.is_empty(),
                "đầu mục {} không mang `source_code`",
                hit.headword
            );
        }

        // Đầu mục tiếng Anh không có dạng giản thể — trường vẫn có mặt, giá trị là
        // `None`. Đó là **một hình dạng bản ghi**, không phải hai.
        for hit in &en.hits {
            assert_eq!(hit.headword_simp, None);
        }
    }

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC9 — NFR1, đo THẬT trên `dict-core.db`. KHÔNG chạy trong CI.
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 Phép đo p95 của NFR1 — `#[ignore]`, lái bằng **biến môi trường**.
///
/// ```sh
/// AURA_DICT_BENCH_DB=tools/dict-build/out/dict-core.db \
///   cargo test --manifest-path src-tauri/Cargo.toml -- --ignored --nocapture
/// ```
///
/// **Ca này không được phép chạy trong CI, và cả hai lớp chặn đều cần thiết:**
/// `#[ignore]` (CI không truyền `--ignored`) **và** biến môi trường vắng mặt ⇒ bỏ qua.
/// CI không có tệp `.db` nào — `.gitignore: *.db` (AD-25), tệp thật nặng 195 MB — nên
/// một ngưỡng thời gian ở đây là một test flaky sẽ bị gỡ trong tháng, đúng tiền lệ
/// `unmeasured` của Story 1.3.
///
/// Đường dẫn **không** viết cứng: một đường dẫn cứng biến ca này thành đỏ trên mọi
/// máy không phải máy đã viết nó.
#[test]
#[ignore = "can tep .db that; chay tay qua AURA_DICT_BENCH_DB"]
fn bench_three_branches_on_the_real_dictionary() {
    let Ok(raw) = std::env::var("AURA_DICT_BENCH_DB") else {
        println!("AURA_DICT_BENCH_DB vắng mặt — bỏ qua phép đo.");
        return;
    };

    let path = PathBuf::from(&raw);
    assert!(
        path.exists(),
        "AURA_DICT_BENCH_DB trỏ tới {} — tệp không tồn tại",
        path.display()
    );

    let db = ReadOnlyDb::open(path.clone(), StoreKind::Dict)
        .unwrap_or_else(|e| panic!("mở {}: {e:?}", path.display()));

    println!("\n═══ Tệp: {} ═══", path.display());

    // ── AC2 + AC3 + AC4: số hàng, đo lại trên tệp thật ──────────────────────────
    //
    // ⚠️ Lệch ⇒ DỪNG. Tệp `.db` mang `built_at = 2026-08-04T23:53:16Z` và không lượt
    // build nào chạy giữa hai lần đo, nên một con số lệch là mã sai chứ không phải
    // "dữ liệu đổi rồi".
    // ⚠️ Tên rút gọn **chỉ trong hàm này**, và chỉ để hai bảng dưới đây đọc được thành
    // BẢNG. Mọi tệp khác gõ đủ `LookupMode::…` / `QueryRoute::…` / `QueryBranch::…`.
    use LookupMode::{Exact, Substring};
    use QueryBranch::{CharIdx, ExactBtree, FtsTrigram};
    use QueryRoute::{En, Zh};

    let cases: &[(&str, LookupMode, QueryRoute, QueryBranch, usize)] = &[
        // ── Đường tiếng Trung (Story 1.11) — KHÔNG đổi một con số nào ──────────
        ("山", Exact, Zh, ExactBtree, 6),
        ("山", Substring, Zh, CharIdx, 3_177),
        ("中國", Exact, Zh, ExactBtree, 4),
        ("中國", Substring, Zh, CharIdx, 350),
        ("中國人", Substring, Zh, FtsTrigram, 33),
        // 🔴 Đối chứng âm AC3 — CHỈ đo được bằng truy vấn LATIN.
        ("lock", Exact, Zh, ExactBtree, 0),
        ("dic", Substring, Zh, FtsTrigram, 0),
        // ── Đường tiếng Anh (Story 1.11b) ────────────────────────────────────────
        //
        // 🔴 `Running` và `running` cho **CÙNG một** hàng — đó là toàn bộ lý do AD-44 ③
        // tồn tại. Đo trước khi vá: `headword = 'Running'` ⇒ **0** hàng.
        ("running", Exact, En, ExactBtree, 1),
        ("Running", Exact, En, ExactBtree, 1),
        // 🔴 Bất đối xứng có chủ ý: khoá gốc được GIỮ, không bị THAY.
        ("API", Exact, En, ExactBtree, 1),
        ("api", Exact, En, ExactBtree, 0),
        // 🔴 **571, không phải 572.** `entry_fts MATCH '"dic"'` cho **572** ứng viên
        // `lang='en'`; `verify_substring` loại **1** dương tính giả (tokenizer `trigram`
        // không phân biệt chữ hoa, `str::contains` thì có). Con số đó là bằng chứng
        // rằng bước xác minh **có việc thật để làm** trên đường tiếng Anh nữa.
        ("dic", Substring, En, FtsTrigram, 571),
        // 🔴 Đối chứng âm của AC8, ép tổ hợp mà `pick_route` không bao giờ sinh.
        ("中國人", Substring, En, FtsTrigram, 0),
        ("中國", Exact, En, ExactBtree, 0),
    ];

    println!("\n── AC2/AC3/AC4 + AD-44: số hàng ──");
    for (query, mode, route, expected_branch, expected_rows) in cases {
        let result = db
            .read(|conn| lookup(conn, query, *mode, *route, UNLIMITED))
            .unwrap_or_else(|e| panic!("tra {query:?}: {e:?}"));

        assert_eq!(
            result.branch, *expected_branch,
            "truy vấn {query:?} ({route:?}) đi sai nhánh"
        );
        assert_eq!(
            result.hits.len(),
            *expected_rows,
            "truy vấn {query:?} ({mode:?}, {route:?}) trả {} hàng, chờ {expected_rows}. \
             ĐỪNG sửa con số này — tệp không được dựng lại giữa hai lần đo.",
            result.hits.len()
        );
        println!(
            "  {query:8} {mode:12?} {route:4?} {:22?}  {:>6} hàng  ✓",
            result.branch,
            result.hits.len()
        );
    }

    // ── AC9: p50 · p95 · p99, ≥ 200 lượt, bỏ 10 lượt làm nóng ──────────────────
    const WARMUP: usize = 10;
    const RUNS: usize = 200;
    /// Trần **dẫn xuất**: NFR1 cho 100 ms đầu-cuối, và PRD dành ~99,95 ms cho vòng IPC
    /// Tauri cộng render frontend (giả định `[A1]`). 10 ms giữ lại ≥ 90 ms cho hai thứ
    /// chưa ai đo.
    const CEILING_MS: f64 = 10.0;

    println!("\n── AC9: p50 · p95 · p99 ({RUNS} lượt, bỏ {WARMUP} lượt làm nóng) ──");
    println!(
        "  {:<18} {:<12} {:>9} {:>9} {:>9}",
        "nhánh", "truy vấn", "p50", "p95", "p99"
    );

    let mut worst_p95 = 0.0f64;
    let mut worst_branch = String::new();

    // ⚠️ Nhánh 2 đo **cả hai** độ dài của nó (1 ký tự và 2 ký tự): bảng AD-26 công bố một
    // dải 0,15–4,5 ms cho nhánh này, và hai đầu dải là hai câu SQL khác nhau — một tập
    // `char_idx` với 3.177 hàng, và một `INTERSECT` hai tập. Đo một đầu rồi kết luận cho
    // cả dải là đo một thứ khác thứ mình khai.
    //
    // 🔴 Đường tiếng Anh đo **BA** tổ hợp, không phải một (AD-44 ⑥): nhánh tra chính
    // xác với truy vấn **chữ thường** (tập khoá hai phần tử **trùng nhau**), cùng nhánh
    // đó với truy vấn **chữ HOA** (hai phần tử **khác nhau** — hai lượt dò B-tree), và
    // nhánh trigram. Đo một tổ hợp rồi kết luận cho cả đường là đo một thứ khác thứ mình
    // khai — cùng bài học đã học ở dải 0,15–4,5 ms của nhánh 2 zh.
    let bench: &[(&str, LookupMode, QueryRoute, &str)] = &[
        ("山", Exact, Zh, "zh-1-btree"),
        ("山", Substring, Zh, "zh-2-charidx-1"),
        ("中國", Substring, Zh, "zh-2-charidx-2"),
        ("中國人", Substring, Zh, "zh-3-trigram"),
        ("running", Exact, En, "en-1-btree-lower"),
        ("Running", Exact, En, "en-1-btree-upper"),
        ("dic", Substring, En, "en-2-trigram"),
    ];

    for (query, mode, route, label) in bench {
        for _ in 0..WARMUP {
            let _ = db.read(|conn| lookup(conn, query, *mode, *route, UNLIMITED)).unwrap();
        }

        let mut samples = Vec::with_capacity(RUNS);
        for _ in 0..RUNS {
            let start = std::time::Instant::now();
            let _ = db.read(|conn| lookup(conn, query, *mode, *route, UNLIMITED)).unwrap();
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).expect("không có NaN trong phép đo"));

        let pct = |p: f64| -> f64 {
            // Chỉ số kiểu "nearest-rank", có trần — không nội suy, không tràn.
            let idx = ((p / 100.0) * samples.len() as f64).ceil() as usize;
            samples[idx.saturating_sub(1).min(samples.len() - 1)]
        };
        let (p50, p95, p99) = (pct(50.0), pct(95.0), pct(99.0));

        println!("  {label:<18} {query:<12} {p50:>8.3}ms {p95:>8.3}ms {p99:>8.3}ms");

        if p95 > worst_p95 {
            worst_p95 = p95;
            worst_branch = format!("{label} ({query})");
        }
    }

    println!("\n  Nhánh chậm nhất: {worst_branch} — p95 {worst_p95:.3} ms (trần {CEILING_MS} ms)");

    // 🔴 VƯỢT trần ⇒ ca này ĐỎ. Số đã in ở trên, nên người chạy có đủ dữ kiện để báo
    // lại; ĐỪNG tự thêm chỉ mục và đừng tự đổi lược đồ của `tools/dict-build`.
    assert!(
        worst_p95 <= CEILING_MS,
        "p95 của nhánh chậm nhất là {worst_p95:.3} ms, VƯỢT trần {CEILING_MS} ms \
         ({worst_branch}). Ghi số, nêu nhánh, rồi DỪNG và báo. Không tự thêm chỉ mục, \
         không tự đổi lược đồ."
    );

    db.close();
}

// ═════════════════════════════════════════════════════════════════════════════════
// Cổng parity lược đồ
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 DDL của fixture phải **nguyên văn** như `tools/dict-build/src/schema.rs`.
///
/// Thất bại ⇒ lược đồ hai cây đã trôi khỏi nhau, và **mọi ca ở trên đang kiểm một
/// database không tồn tại trong sản phẩm**.
///
/// ⚠️ Phép so làm trên **văn bản nguồn**, không trên `sqlite_master` — đó là điều kiện
/// để nó chạy mà không cần một tệp `.db` nào, tức để nó ở được trong CI.
///
/// ⚠️ Dấu `"` được escape lại trước khi so: trong `schema.rs`, `ENTRY_FTS_DDL` viết
/// `tokenize=\"trigram\"` ở **mã nguồn** trong khi **giá trị** của hằng là
/// `tokenize="trigram"`. So thẳng giá trị với văn bản nguồn sẽ đỏ trên đúng một khối, và
/// người sửa tiếp theo sẽ gỡ khối đó ra khỏi cổng.
#[test]
fn fixture_ddl_is_verbatim_from_dict_build_schema() {
    let schema_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tools")
        .join("dict-build")
        .join("src")
        .join("schema.rs");

    let source = fs::read_to_string(&schema_rs).unwrap_or_else(|e| {
        panic!(
            "đọc {}: {e}. Cổng parity KHÔNG được nới thành `if let Ok(...)` — một tệp \
             nguồn không đọc được là một cổng chết, không phải một cổng đã đạt.",
            schema_rs.display()
        )
    });

    for (name, ddl) in COPIED_DDL {
        let needle = ddl.replace('"', "\\\"");
        assert!(
            source.contains(&needle),
            "khối DDL `{name}` trong `tests/dict_lookup.rs` KHÔNG còn khớp nguyên văn \
             với `tools/dict-build/src/schema.rs`.\n\n\
             Lược đồ hai cây đã trôi khỏi nhau. MỌI ca trong tệp này đang kiểm một \
             database không tồn tại trong sản phẩm.\n\n\
             Đường sửa: chép lại khối đó từ `schema.rs`, rồi đọc lại các ca ở trên xem \
             chúng còn nói đúng thứ chúng định nói không. ĐỪNG gỡ khối này ra khỏi \
             `COPIED_DDL`.\n\n\
             Đang tìm:\n{needle}"
        );
    }

    // Sàn quần thể — một `COPIED_DDL` bị cắt làm vòng lặp trên xanh mà không kiểm gì.
    assert!(
        COPIED_DDL.len() >= 9,
        "chỉ {} khối DDL trong `COPIED_DDL` — fixture đã bị cắt",
        COPIED_DDL.len()
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.11b · AC2 — MỘT định nghĩa `is_han`, và một cổng kiểm chéo hai workspace
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC2** — bảy dải CJK của `core::dict::is_han` phải **nguyên văn** như
/// `tools/dict-build/src/char_idx.rs::is_han`.
///
/// Vì sao đây là một cổng chứ không phải một lượt đọc bằng mắt: hai định nghĩa lệch
/// nhau định tuyến một truy vấn sang đường tiếng Trung rồi tra nó vào một `char_idx`
/// **chưa bao giờ lập chỉ mục ký tự đó** ⇒ kết quả **rỗng**, **không lỗi** — đúng lớp
/// lỗi AD-26 ra đời để chặn. Hai workspace tách rời **có chủ ý** (AC4 của Story 1.9) nên
/// một lời gọi chéo là không được phép; phép so làm trên **văn bản nguồn**, đúng khuôn
/// [`fixture_ddl_is_verbatim_from_dict_build_schema`], và vì cùng lý do: nó chạy được mà
/// không cần một tệp `.db` nào, tức nó ở được trong CI.
///
/// ⚠️ Cổng có **hai vế**, và bỏ vế nào cũng làm nó thành trang trí:
/// 1. **Văn bản** — bảy chuỗi dải có mặt nguyên văn trong `char_idx.rs`.
/// 2. **Hành vi** — chính hàm `core::dict::is_han` nhận đúng bảy dải đó, kiểm ở **cả hai
///    biên** cộng hai điểm ngay ngoài biên. Không có vế này, một `is_han` chỉ-BMP vẫn qua
///    cổng vì hằng `HAN_RANGES` không nói gì về mã đang chạy.
#[test]
fn han_ranges_are_verbatim_from_dict_build_char_idx() {
    let char_idx_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tools")
        .join("dict-build")
        .join("src")
        .join("char_idx.rs");

    let source = fs::read_to_string(&char_idx_rs).unwrap_or_else(|e| {
        panic!(
            "đọc {}: {e}. Cổng parity KHÔNG được nới thành `if let Ok(...)` — một tệp \
             nguồn không đọc được là một cổng chết, không phải một cổng đã đạt.",
            char_idx_rs.display()
        )
    });

    for range in HAN_RANGES {
        assert!(
            source.contains(range),
            "dải `{range}` KHÔNG còn có mặt nguyên văn trong \
             `tools/dict-build/src/char_idx.rs`.\n\n\
             Hai định nghĩa `is_han` đã trôi khỏi nhau. Hệ quả không phải một lỗi: một \
             truy vấn chứa ký tự thuộc dải bị lệch đi SANG đường tiếng Trung rồi tra vào \
             một `char_idx` chưa bao giờ lập chỉ mục nó ⇒ RỖNG, KHÔNG lỗi.\n\n\
             Đường sửa: chép lại bảy dải từ `char_idx.rs` sang `core/dict/mod.rs::is_han` \
             VÀ sang `HAN_RANGES`. ĐỪNG gỡ dải này ra khỏi `HAN_RANGES`."
        );
    }

    // Sàn quần thể — một `HAN_RANGES` bị cắt cụt làm vòng lặp trên xanh mà không kiểm
    // gì cả, và đó chính là cách bản sao 3-dải cũ từng "đạt".
    assert!(
        HAN_RANGES.len() >= 7,
        "chỉ {} dải trong `HAN_RANGES` (sàn 7) — bảng dải đã bị cắt cụt",
        HAN_RANGES.len()
    );

    // ── Vế hành vi: hàm ĐANG CHẠY mang đúng bảy dải đó ──────────────────────────
    for range in HAN_RANGES {
        let (lo, hi) = range
            .split_once("..=")
            .unwrap_or_else(|| panic!("dải {range:?} sai khuôn `0xAAAA..=0xBBBB`"));
        let parse = |s: &str| {
            u32::from_str_radix(s.trim().trim_start_matches("0x"), 16)
                .unwrap_or_else(|e| panic!("đọc {s:?}: {e}"))
        };
        let (lo, hi) = (parse(lo), parse(hi));

        for cp in [lo, hi] {
            let c = char::from_u32(cp).unwrap_or_else(|| panic!("U+{cp:04X} không hợp lệ"));
            assert!(
                is_han(c),
                "`core::dict::is_han('\\u{{{cp:04X}}}')` trả `false`, nhưng U+{cp:04X} là \
                 một biên của dải {range}. Hàm sản phẩm KHÔNG mang đủ bảy dải."
            );
        }

        // Hai điểm ngay NGOÀI biên — không có vế này, một `is_han` trả `true` cho mọi thứ
        // cũng qua cổng.
        for cp in [lo.wrapping_sub(1), hi + 1] {
            if HAN_RANGES.iter().any(|r| {
                let (l, h) = r.split_once("..=").expect("khuôn dải");
                let p =
                    |s: &str| u32::from_str_radix(s.trim().trim_start_matches("0x"), 16).unwrap();
                (p(l)..=p(h)).contains(&cp)
            }) {
                continue;
            }
            if let Some(c) = char::from_u32(cp) {
                assert!(
                    !is_han(c),
                    "`core::dict::is_han('\\u{{{cp:04X}}}')` trả `true`, nhưng U+{cp:04X} \
                     nằm NGOÀI cả bảy dải — hàm đã bị nới rộng."
                );
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.11b · AC1 — vị từ điều phối: hình dạng CHUỖI, nhị phân, không chạm DB
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC1** — và như ca `branch_is_picked_by_char_count_not_byte_length`, nó chạy mà không
/// **không cần một tệp `.db` nào**: `pick_route` là hàm thuần, nên phép kiểm đắt nhất của
/// story này nghiệm thu được trong CI, nơi không có tệp từ điển nào.
#[test]
fn a_query_containing_any_han_character_routes_to_the_chinese_path() {
    for query in ["山", "中國", "中國人", "中國API", "API中", "日本語"] {
        assert_eq!(
            pick_route(query),
            QueryRoute::Zh,
            "truy vấn {query:?} chứa ít nhất một ký tự Hán ⇒ đường `Zh`"
        );
    }
}

/// 🔴 **AC1 vế "NHỊ PHÂN"** — **không có nhánh thứ ba.** Chuỗi rỗng, chữ số thuần, dấu
/// câu thuần, và một hệ chữ viết **thứ ba** đều đi đường `En`.
///
/// Vì sao vế này là một AC chứ không phải một chi tiết: một vị từ ba nhánh
/// (`Zh` / `En` / `Unknown`) đẩy câu hỏi *"làm gì với `Unknown`"* xuống mọi chỗ gọi, và
/// mỗi chỗ gọi sẽ trả lời khác nhau. `"Ελλάδα"` không có trong từ điển nào — nó đi
/// đường `En`, chạy một nhánh thật, và trả **rỗng có lý do**.
#[test]
fn everything_that_is_not_han_routes_to_the_english_path() {
    for query in [
        "",
        "2026",
        "...",
        "Ελλάδα",
        "ひらがな",
        "API",
        "running",
        "   ",
    ] {
        assert_eq!(
            pick_route(query),
            QueryRoute::En,
            "truy vấn {query:?} không chứa ký tự Hán nào ⇒ đường `En`, KHÔNG một \
             nhánh thứ ba"
        );
    }
}

/// 🔴 **AC1 + AC2 nối vào nhau** — ca này chỉ xanh nếu `is_han` mang **bảy** dải.
///
/// `𠧜` (U+209DC) nằm ở CJK Extension B, **ngoài BMP**. Bản sao 3-dải mà story này xoá
/// đọc nó thành *"không phải chữ Hán"* ⇒ `pick_route` trả `En` ⇒ truy vấn chạy nhánh
/// tiếng Anh, lọc `lang = 'en'`, và trả **rỗng** cho một đầu mục tiếng Trung có thật.
/// Đây là **đối chứng sống** cho AC2, không phải một ca biên cho vui.
#[test]
fn a_han_character_outside_the_bmp_still_routes_to_the_chinese_path() {
    assert_eq!(
        "𠧜".chars().count(),
        1,
        "tiền đề: một ký tự, không phải hai"
    );
    assert_eq!(pick_route("𠧜"), QueryRoute::Zh);
    assert!(
        is_han('𠧜'),
        "`is_han` không nhận ký tự ngoài BMP — bảng dải đã tụt về bản chỉ-BMP"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.11b · AC5 — đường tiếng Anh có HAI nhánh, không phải ba
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC5** — bảng nhánh của đường `En`, và nó chạy mà không cần một tệp `.db` nào.
///
/// **Không** ô `char_idx` cho tiếng Anh, và đó là một **số đo** chứ không phải một
/// sở thích: lớp `viwiktionary-en` sinh **đúng 9** cặp `char_idx` trên **119.039** đầu
/// mục (0,0076%). Bảng đảo ngược không áp được cho tiếng Anh.
#[test]
fn the_english_path_has_exactly_two_branches_and_a_not_supported_state() {
    // Tra chính xác không phụ thuộc độ dài — giống hệt đường zh.
    for query in ["a", "ap", "api", "API", "state-of-the-art", ""] {
        assert_eq!(
            pick_branch(query, LookupMode::Exact, QueryRoute::En),
            QueryBranch::ExactBtree,
            "truy vấn {query:?} ở chế độ Exact trên đường En"
        );
    }

    // Chuỗi con ≥ 3 ký tự ⇒ trigram.
    for query in ["dic", "dictionary", "don't"] {
        assert_eq!(
            pick_branch(query, LookupMode::Substring, QueryRoute::En),
            QueryBranch::FtsTrigram,
            "truy vấn {query:?} ({} ký tự) phải đi nhánh trigram",
            query.chars().count()
        );
    }

    // 🔴 Chuỗi con < 3 ký tự ⇒ KHÔNG nhánh nào chạy. Ca 0 ký tự đi CÙNG đường với
    // 1–2: vị từ độ dài là **một** mệnh đề, không phải hai mệnh đề với một ca đặc biệt
    // ở giữa.
    for query in ["", "l", "lo"] {
        assert_eq!(
            pick_branch(query, LookupMode::Substring, QueryRoute::En),
            QueryBranch::NoBranchQueryTooShort,
            "truy vấn {query:?} ({} ký tự) phải là `NoBranchQueryTooShort`, KHÔNG tràn \
             qua một nhánh khác — một nhánh chạy trên nó sẽ trả rỗng IM LẶNG",
            query.chars().count()
        );
    }

    // Đường En KHÔNG BAO GIỜ sinh nhánh `CharIdx`, ở bất kỳ độ dài nào, bất kỳ chế
    // độ nào. Ca này là đối chứng cho cả bảng.
    for query in ["", "a", "ab", "abc", "abcdefgh", "don't"] {
        for mode in [LookupMode::Exact, LookupMode::Substring] {
            assert_ne!(
                pick_branch(query, mode, QueryRoute::En),
                QueryBranch::CharIdx,
                "truy vấn {query:?} ({mode:?}) đi nhánh `char_idx` trên đường En — bảng \
                 đảo ngược đó chỉ có **9** cặp cho toàn bộ 119.039 đầu mục tiếng Anh"
            );
        }
    }
}

/// 🔴 **AC5 vế `chars().count()`** — ngưỡng **< 3** của đường En dùng **cùng** phép đo mà
/// Bẫy 1 của Story 1.11 đã trả giá để học.
///
/// Ca này không nói về tiếng Trung: nó nói rằng một chuỗi Latin-mở-rộng **hai ký tự**
/// có `len()` **bốn** byte, nên một ngưỡng viết bằng `len()` đẩy nó sang nhánh trigram —
/// nơi FTS5 `trigram` không lập chỉ mục token < 3 ký tự ⇒ **0** hàng, **0 lỗi**.
#[test]
fn the_english_length_threshold_counts_characters_not_bytes() {
    assert_eq!("üé".len(), 4, "tiền đề của cả ca này");
    assert_eq!("üé".chars().count(), 2, "tiền đề của cả ca này");

    assert_eq!(
        pick_branch("üé", LookupMode::Substring, QueryRoute::En),
        QueryBranch::NoBranchQueryTooShort,
        "hai ký tự Latin-mở-rộng bị đọc thành bốn byte — ngưỡng đã viết bằng `len()`"
    );

    assert_eq!("üéa".chars().count(), 3, "tiền đề của cả ca này");
    assert_eq!(
        pick_branch("üéa", LookupMode::Substring, QueryRoute::En),
        QueryBranch::FtsTrigram
    );
}

/// **AC1 mệnh đề cuối** — vị từ nói về **hình dạng chuỗi truy vấn**, không nói về ngôn
/// ngữ của Tác phẩm.
///
/// Bôi đen `API` trong một truyện tiếng Trung phải ra **kết quả**, không ra rỗng
/// (AD-44 Prevents #2). Cùng một chuỗi luôn cho cùng một đường, không phụ thuộc ngữ
/// cảnh nào — và một hàm **thuần một tham số** là cách mệnh đề đó cưỡng chế được.
#[test]
fn the_route_depends_only_on_the_query_never_on_surrounding_context() {
    assert_eq!(pick_route("API"), QueryRoute::En);
    assert_eq!(pick_route("running"), QueryRoute::En);
}

// ═════════════════════════════════════════════════════════════════════════════════
// STORY 1.18 — TRẦN AN TOÀN CHO TẬP ỨNG VIÊN (`deferred-work.md:631`)
// ═════════════════════════════════════════════════════════════════════════════════

/// Fixture riêng: **60 đầu mục không CÓ CÁI NÀO là chuỗi con thật của truy vấn.**
///
/// 🔴 Hình dạng này là cả nội dung của ca test dưới đây. Mọi đầu mục chứa **cả** `山` lẫn
/// `河` nên `char_idx … INTERSECT …` trả về **toàn bộ 60** làm ứng viên — nhưng không cái nào
/// chứa `山河` **liền nhau**, nên `verify_substring` loại **sạch**. Đó chính xác là ca mà
/// một cài đặt chỉ gọi `cap()` sẽ báo `truncated = false` — *"danh sách này đầy đủ"* — sau
/// khi SQL vừa cắt mất hàng chục hàng.
fn build_ceiling_fixture(dir: &Path) -> PathBuf {
    let path = dir.join("dict-ceiling.db");
    let conn = rusqlite::Connection::open(&path)
        .unwrap_or_else(|e| panic!("dựng fixture {}: {e}", path.display()));

    for (name, ddl) in COPIED_DDL {
        conn.execute_batch(ddl).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
    conn.execute_batch(
        "INSERT INTO dict_meta (key, value) VALUES ('schema_version', '1');
         INSERT INTO dict_source
           (id, code, display_name, license_kind, license_id, license_text,
            attribution, source_version, source_url)
         VALUES (1, 'fixture-ceiling', 'Fixture Ceiling', 'public-domain', NULL, 'x', 'x', '1', 'x');",
    )
    .unwrap_or_else(|e| panic!("nạp dict_source: {e}"));

    // `河山` — không BAO GIỜ `山河`. Ký tự thứ ba lấy từ một dải Hán liên tục, mỗi hàng một
    // ký tự khác nhau, nên không hàng nào trùng đầu mục với hàng nào.
    for i in 0..60u32 {
        let filler = char::from_u32(0x4E00 + i).expect("dải Hán hợp lệ");
        let headword = format!("河山{filler}");
        conn.execute(
            "INSERT INTO dict_entry (id, source_id, lang, headword, headword_simp)
             VALUES (?1, 1, 'zh', ?2, NULL)",
            rusqlite::params![i64::from(i) + 1, headword],
        )
        .unwrap_or_else(|e| panic!("nạp dict_entry {i}: {e}"));

        for c in headword.chars() {
            if is_han(c) {
                conn.execute(
                    "INSERT OR IGNORE INTO char_idx (ch, entry_id) VALUES (?1, ?2)",
                    rusqlite::params![c.to_string(), i64::from(i) + 1],
                )
                .unwrap_or_else(|e| panic!("nạp char_idx: {e}"));
            }
        }
    }

    conn.execute_batch("INSERT INTO entry_fts(entry_fts) VALUES('rebuild');")
        .unwrap_or_else(|e| panic!("rebuild entry_fts: {e}"));
    conn.close().unwrap_or_else(|(_, e)| panic!("đóng fixture: {e}"));
    path
}

/// 🔴 **`deferred-work.md:631` — cờ `truncated` không ĐƯỢC NÓI DỐI khi trần an toàn chạm.**
///
/// Trần ứng viên là `limit * 50`, nên `limit = 1` ⇒ **50**. Fixture có **60** ứng viên và
/// **0** trong số đó qua được `verify_substring`.
///
/// **Ca này ĐỎ trên bản trước Story 1.18**: ở đó nhánh không có `LIMIT` nào ở SQL, cả 60
/// hàng vào RAM, `verify_substring` loại sạch, `cap(vec![], 1)` trả `truncated = false`, và
/// panel nói *"không tìm thấy trong từ điển"* mà không nói rằng nó **không hề nhìn hết**.
#[test]
fn the_candidate_ceiling_keeps_the_truncated_flag_honest() {
    let dir = temp_dir("ceiling-honest");
    let path = build_ceiling_fixture(&dir);
    {
        let db = open_fixture(&path);
        let result = db
            .read(|conn| lookup(conn, "山河", LookupMode::Substring, QueryRoute::Zh, 1))
            .expect("tra `山河`");

        assert_eq!(result.branch, QueryBranch::CharIdx, "hai ký tự Hán ⇒ nhánh char_idx");
        assert!(
            result.hits.is_empty(),
            "không đầu mục nào chứa `山河` liền nhau — mọi ứng viên là `河山…`, tức dương tính giả"
        );
        assert!(
            result.truncated,
            "🔴 `deferred-work.md:631` — SQL đã cắt ở trần 50/60 ứng viên, nên lượt tra này \
             không nhìn hết. Một `truncated = false` ở đây là câu *danh sách đầy đủ*, và nó SAI."
        );
    }
    cleanup(&dir);
}

/// Trần **không được cắt vào phần Bẫy 11 nói tới** — một tập ứng viên nhỏ đi qua nguyên vẹn.
#[test]
fn the_candidate_ceiling_never_touches_an_ordinary_lookup() {
    let dir = temp_dir("ceiling-ordinary");
    let path = build_fixture(&dir);
    {
        let db = open_fixture(&path);
        // `中國` — 390 ứng viên trên từ điển thật, 3 trên fixture. Trần với `limit = 20` là
        // 1.000, tức hai bậc độ lớn trên số ứng viên khả dĩ.
        let result = db
            .read(|conn| lookup(conn, "中國", LookupMode::Substring, QueryRoute::Zh, 20))
            .expect("tra `中國`");
        assert!(
            !result.truncated,
            "trần an toàn không được biến một lượt tra bình thường thành *danh sách chưa đầy đủ*"
        );
        assert!(
            result.hits.iter().any(|h| h.headword == "中國"),
            "và nó không được cắt mất kết quả thật"
        );
    }
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// STORY 1.18 — ĐƯỜNG LUI `Substring` (Ice chốt 2026-08-07)
// ═════════════════════════════════════════════════════════════════════════════════

/// Vị từ đường lui đếm **KÝ TỰ**, không byte — cùng cái bẫy `len()` mà `pick_branch` mang.
#[test]
fn the_substring_fallback_threshold_counts_characters_not_bytes() {
    use auratranslate_lib::commands::dict::should_try_substring;

    // Tiền đề: bốn ký tự Hán là **mười hai** byte.
    assert_eq!("山河大地".len(), 12, "tiền đề của cả ca này");
    assert_eq!("山河大地".chars().count(), 4);

    assert!(should_try_substring("山"), "một ký tự Hán");
    assert!(should_try_substring("山河大地"), "một thành ngữ bốn ký tự — trần, vẫn TRONG");
    assert!(
        !should_try_substring("山河大地人"),
        "năm ký tự — quá dài để còn đáng tra như một chuỗi con"
    );
    assert!(!should_try_substring(""), "rỗng không đáng một lượt tra thứ hai");
    assert!(should_try_substring("abc"), "ba ký tự Latin");
    assert!(!should_try_substring("dictionary"), "mười ký tự Latin");
}
