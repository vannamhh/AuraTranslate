//! Hành vi tầng **GOM** — Story 1.13, AC3 tới AC13.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ VÌ SAO TỆP NÀY ĐƯỢC PHÉP `use rusqlite`
//! ─────────────────────────────────────────────────────────────────────────────
//! Cùng lý do đã ghi ở `dict_lookup.rs:4-11`: `store_boundary.rs` cưỡng chế ranh giới
//! trên `src-tauri/src/**`, `tests/**` nằm ngoài **có tên và có lý do**; và ⛔ không tệp
//! `.db` nào nằm trong git (`.gitignore: *.db` — AD-25), nên fixture phải dựng trong test.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO FIXTURE Ở ĐÂY LÀ **BA TỆP**, ⛔ KHÔNG PHẢI MỘT
//! ─────────────────────────────────────────────────────────────────────────────
//! `dict_lookup.rs` dựng **một** tệp, vì Story 1.11 chạy trên một tệp một lượt. Ba cái
//! bẫy đắt nhất của story này ⛔ **không quan sát được** trên một tệp:
//!
//! 1. **`source_id` trùng giữa các tệp.** Mỗi tệp `.db` mang bảng `dict_source` RIÊNG, nên
//!    `id = 1` tồn tại ở **cả ba** và trỏ ba nguồn khác nhau. Cả ba tệp fixture dưới đây
//!    dùng `id = 1` **có chủ ý** — gom theo `id` dán nhãn sai nguồn, và ⛔ không một ca
//!    một-tệp nào đỏ.
//! 2. **Thứ tự lớp.** Tên tệp cố tình xếp `aaa` · `mmm` · `zzz` trong khi thứ tự đúng là
//!    `base` · `hv-fixture` · `vp-fixture` — tức **ngược** thứ tự chữ cái của tên tệp. Một
//!    cài đặt tin vào thứ tự `read_dir` sẽ đỏ ở đây thay vì đỏ trên **một** nhánh CI.
//! 3. **FR36.** *"Gỡ một lớp = xoá một file"* ⛔ không nghiệm thu được nếu chỉ có một file.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT — thừa kế nguyên từ `dict_lookup.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + bộ đếm nguyên tử). ⛔ Không `tempfile`.
//! 2. **Drop `ReadOnlyDb` TRƯỚC khi xoá tệp** — Windows từ chối xoá tệp đang mở (NFR14).
//!    🔴 Ở tệp này luật đó ⛔ không còn là dọn dẹp: nó là **điều kiện để AC12 chạy được**.
//! 3. **⛔ Không ngưỡng thời gian trong CI** — phép đo NFR1 là
//!    [`bench_the_grouped_path_on_the_real_dictionaries`]: `#[ignore]` + biến môi trường.
//! 4. **Đường dẫn tương đối lấy qua `env!("CARGO_MANIFEST_DIR")`.**

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::core::dict::{
    DictLayers, HAN_VIET_BATCH, HanVietHit, LookupMode, QueryBranch, QueryRoute, SENSE_BATCH,
    SUPPORTED_SCHEMA_VERSION, SenseRecord, SkipReason, is_han, lookup_grouped, lookup_han_viet,
};
use auratranslate_lib::ports::DictionarySource;

// ═════════════════════════════════════════════════════════════════════════════════
// DDL — CHÉP NGUYÊN VĂN từ `tools/dict-build/src/schema.rs`
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⛔ Đừng "dọn dẹp" khoảng trắng ở đây. Cổng parity so **chuỗi con nguyên văn**; một lượt
// canh lề tử tế làm nó đỏ, và người sửa tiếp theo sẽ sửa bằng cách nới cổng.
//
// 🔴 `DICT_CITATION_DDL` ở tệp này ⛔ **không** còn là một khối chép cho đủ: story này là
// story đầu tiên **đọc** bảng đó (AC8), nên fixture dưới đây nạp dữ liệu thật vào nó.

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
  source_url     TEXT NOT NULL
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

/// Một thư mục tạm **của riêng ca này** — khuôn `dict_lookup.rs:157`.
fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-layers-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));
    dir
}

/// ⚠️ Gọi **sau** khi mọi `DictLayers` đã drop hoặc `close()`. Xem luật 2.
fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

/// Một ví dụ của fixture: `(id, text, translation, translation_lang, ord)`.
type ExampleSeed = (
    i64,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
    i64,
);

/// Một trích dẫn của fixture: `(id, text, work, author, ord)`.
type CitationSeed = (
    i64,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
    i64,
);

struct SenseSeed {
    id: i64,
    pos: Option<&'static str>,
    pos_lang: Option<&'static str>,
    gloss: &'static str,
    note: Option<&'static str>,
    ord: i64,
    examples: &'static [ExampleSeed],
    citations: &'static [CitationSeed],
}

struct EntrySeed {
    id: i64,
    source_id: i64,
    lang: &'static str,
    headword: &'static str,
    simp: Option<&'static str>,
    senses: &'static [SenseSeed],
}

struct LayerSeed {
    /// 🔴 Tên tệp **cố tình** ⛔ không nói gì về lớp bên trong — AD-44 ① vá A2: danh tính
    /// lớp đọc từ `dict_meta('layer')` của chính tệp, ⛔ không từ tên tệp.
    file: &'static str,
    layer: &'static str,
    /// `(id, code, display_name)` — 🔴 **cả ba tệp dùng `id = 1`**, có chủ ý.
    sources: &'static [(i64, &'static str, &'static str)],
    entries: &'static [EntrySeed],
}

// ─────────────────────────────────────────────────────────────────────────────────
// LỚP NỀN — nhãn từ loại **NGOẠI NGỮ** (`pos_lang = 'en'`), đúng hình dạng của
// `dict-core.db` hôm nay (AC10). Mang **hai** nguồn, như tệp thật mang sáu.
// ─────────────────────────────────────────────────────────────────────────────────
static BASE_SENSES_SHAN: &[SenseSeed] = &[
    SenseSeed {
        id: 1,
        pos: Some("noun"),
        pos_lang: Some("en"),
        gloss: "mountain",
        note: Some("base layer note"),
        ord: 0,
        examples: &[(1, "高山", Some("high mountain"), Some("en"), 0)],
        citations: &[],
    },
    SenseSeed {
        id: 2,
        pos: Some("proper noun"),
        pos_lang: Some("en"),
        gloss: "surname Shan",
        note: None,
        ord: 1,
        examples: &[],
        citations: &[],
    },
];

static BASE_SENSES_ZHONGGUO: &[SenseSeed] = &[SenseSeed {
    id: 3,
    pos: Some("noun"),
    pos_lang: Some("en"),
    gloss: "China",
    note: None,
    ord: 0,
    examples: &[],
    citations: &[],
}];

static BASE_SENSES_GAOSHAN: &[SenseSeed] = &[SenseSeed {
    id: 4,
    pos: None,
    pos_lang: None,
    gloss: "alpine",
    note: None,
    ord: 0,
    examples: &[],
    citations: &[],
}];

/// 🔴 AC9 — mục từ **tiếng Anh**: nhãn từ loại + nghĩa **tiếng Việt**, đi qua **cùng**
/// hình dạng bản ghi với mục tiếng Trung (AD-44 ⑤).
static BASE_SENSES_LOCK: &[SenseSeed] = &[SenseSeed {
    id: 5,
    pos: Some("danh từ"),
    pos_lang: Some("vi"),
    gloss: "ổ khoá",
    note: None,
    ord: 0,
    examples: &[(2, "door lock", Some("khoá cửa"), Some("vi"), 0)],
    citations: &[],
}];

static BASE_SENSES_DICTIONARY: &[SenseSeed] = &[SenseSeed {
    id: 6,
    pos: Some("danh từ"),
    pos_lang: Some("vi"),
    gloss: "từ điển",
    note: None,
    ord: 0,
    examples: &[],
    citations: &[],
}];

static BASE_ENTRIES: &[EntrySeed] = &[
    EntrySeed {
        id: 1,
        source_id: 1,
        lang: "zh",
        headword: "山",
        simp: None,
        senses: BASE_SENSES_SHAN,
    },
    EntrySeed {
        id: 2,
        source_id: 1,
        lang: "zh",
        headword: "中國",
        simp: Some("中国"),
        senses: BASE_SENSES_ZHONGGUO,
    },
    EntrySeed {
        id: 3,
        source_id: 2,
        lang: "zh",
        headword: "高山",
        simp: None,
        senses: BASE_SENSES_GAOSHAN,
    },
    EntrySeed {
        id: 4,
        source_id: 1,
        lang: "en",
        headword: "lock",
        simp: None,
        senses: BASE_SENSES_LOCK,
    },
    EntrySeed {
        id: 5,
        source_id: 2,
        lang: "en",
        headword: "dictionary",
        simp: None,
        senses: BASE_SENSES_DICTIONARY,
    },
    // 🔴 Một đầu mục **⛔ không có nghĩa nào** — trạng thái **hợp lệ**, ⛔ không phải một
    // lỗi: `dict_entry` mang `reading` và `han_viet` (âm đọc), và một nguồn có thể ghi âm
    // đọc mà ⛔ không ghi nghĩa. Pha hai phải trả **danh sách rỗng**, ⛔ không trả lỗi.
    EntrySeed {
        id: 6,
        source_id: 2,
        lang: "zh",
        headword: "國",
        simp: Some("国"),
        senses: &[],
    },
];

// ─────────────────────────────────────────────────────────────────────────────────
// LỚP GỠ RỜI #1 — hình dạng **HVTĐTD**: `pos_lang = 'vi'`, ví dụ **và** trích dẫn
// tiếng Việt (AC11).
//
// 🔴 Đây là **FIXTURE**, ⛔ không phải dữ liệu HVTĐTD thật — `dict-hvtdtd.db` ⛔ không tồn
// tại vì chưa có nguồn thô (`src-tauri/resources/dict/README.md:13`, `prd.md:856` [A2]).
// Nó nghiệm thu đúng thứ nghiệm thu được hôm nay: *đường mã có phân biệt được nhãn tiếng
// Việt với nhãn ngoại ngữ ⛔ không*.
// ─────────────────────────────────────────────────────────────────────────────────
static HV_SENSES_SHAN: &[SenseSeed] = &[SenseSeed {
    id: 1,
    pos: Some("danh từ"),
    pos_lang: Some("vi"),
    gloss: "núi",
    note: Some("âm Hán Việt: sơn"),
    ord: 0,
    examples: &[(1, "山川", Some("núi sông"), Some("vi"), 0)],
    citations: &[(
        1,
        "山中無曆日",
        Some("Thái Bình Quảng Ký"),
        Some("Lý Phưởng"),
        0,
    )],
}];

static HV_ENTRIES: &[EntrySeed] = &[EntrySeed {
    id: 1,
    source_id: 1,
    lang: "zh",
    headword: "山",
    simp: None,
    senses: HV_SENSES_SHAN,
}];

// ─────────────────────────────────────────────────────────────────────────────────
// LỚP GỠ RỜI #2 — hình dạng **VietPhrase**: nhiều `dict_sense` **CÙNG `ord`**, đúng như
// `tools/dict-build/src/sources/vietphrase.rs` sinh ra khi tách `/` vô điều kiện
// (`deferred-work.md`, Story 1.10). Đây là quần thể của Bẫy 1 — `ORDER BY ord` trần.
// ─────────────────────────────────────────────────────────────────────────────────
static VP_SENSES_ZHONGGUO: &[SenseSeed] = &[
    SenseSeed {
        id: 1,
        pos: None,
        pos_lang: None,
        gloss: "Trung Quốc",
        note: None,
        ord: 0,
        examples: &[],
        citations: &[],
    },
    SenseSeed {
        id: 2,
        pos: None,
        pos_lang: None,
        gloss: "nước Tàu",
        note: None,
        ord: 0,
        examples: &[],
        citations: &[],
    },
    SenseSeed {
        id: 3,
        pos: None,
        pos_lang: None,
        gloss: "Trung Hoa",
        note: None,
        ord: 1,
        examples: &[],
        citations: &[],
    },
];

static VP_SENSES_SHAN: &[SenseSeed] = &[SenseSeed {
    id: 4,
    pos: None,
    pos_lang: None,
    gloss: "sơn",
    note: None,
    ord: 0,
    examples: &[],
    citations: &[],
}];

static VP_ENTRIES: &[EntrySeed] = &[
    EntrySeed {
        id: 1,
        source_id: 1,
        lang: "zh",
        headword: "中國",
        simp: Some("中国"),
        senses: VP_SENSES_ZHONGGUO,
    },
    EntrySeed {
        id: 2,
        source_id: 1,
        lang: "zh",
        headword: "山",
        simp: None,
        senses: VP_SENSES_SHAN,
    },
];

/// 🔴 Ba lớp — và **tên tệp xếp ngược thứ tự lớp** có chủ ý (xem doc-comment module).
static LAYERS: &[LayerSeed] = &[
    LayerSeed {
        file: "zzz.db",
        layer: "base",
        sources: &[
            (1, "fx-core-a", "Fixture Core A"),
            (2, "fx-core-b", "Fixture Core B"),
        ],
        entries: BASE_ENTRIES,
    },
    LayerSeed {
        file: "mmm.db",
        layer: "hv-fixture",
        sources: &[(1, "fx-hv", "Fixture Han Viet")],
        entries: HV_ENTRIES,
    },
    LayerSeed {
        file: "aaa.db",
        layer: "vp-fixture",
        sources: &[(1, "fx-vp", "Fixture VietPhrase")],
        entries: VP_ENTRIES,
    },
];

/// Thứ tự lớp **đúng** — `base` trước, rồi mã lớp tăng dần (AC3).
const EXPECTED_LAYER_ORDER: &[&str] = &["base", "hv-fixture", "vp-fixture"];

/// Dựng **một** tệp `.db` fixture theo `seed`, trả về đường dẫn.
///
/// ⚠️ Fixture ⛔ **không** đặt `journal_mode`; mặc định `delete` — giống hệt ba tệp thật.
fn build_layer(dir: &Path, seed: &LayerSeed, schema_version: &str, user_version: u32) -> PathBuf {
    let path = dir.join(seed.file);
    let conn = rusqlite::Connection::open(&path)
        .unwrap_or_else(|e| panic!("dựng fixture {}: {e}", path.display()));

    for (name, ddl) in COPIED_DDL {
        conn.execute_batch(ddl)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }

    // 🔴 `dict_meta('layer')` — Story 1.10 §Quyết định #5, và
    // `tools/dict-build/src/insert.rs:110-112` viết sẵn nó cho story này.
    conn.execute(
        "INSERT INTO dict_meta (key, value) VALUES ('schema_version', ?1)",
        rusqlite::params![schema_version],
    )
    .unwrap_or_else(|e| panic!("nạp schema_version: {e}"));
    conn.execute(
        "INSERT INTO dict_meta (key, value) VALUES ('layer', ?1)",
        rusqlite::params![seed.layer],
    )
    .unwrap_or_else(|e| panic!("nạp layer: {e}"));

    for (id, code, display_name) in seed.sources {
        conn.execute(
            "INSERT INTO dict_source
               (id, code, display_name, license_kind, license_id, license_text,
                attribution, source_version, source_url)
             VALUES (?1, ?2, ?3, 'public-domain', NULL, 'x', 'x', '1', 'x')",
            rusqlite::params![id, code, display_name],
        )
        .unwrap_or_else(|e| panic!("nạp dict_source {code}: {e}"));
    }

    for entry in seed.entries {
        conn.execute(
            "INSERT INTO dict_entry (id, source_id, lang, headword, headword_simp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                entry.id,
                entry.source_id,
                entry.lang,
                entry.headword,
                entry.simp
            ],
        )
        .unwrap_or_else(|e| panic!("nạp dict_entry {}: {e}", entry.id));

        for sense in entry.senses {
            conn.execute(
                "INSERT INTO dict_sense (id, entry_id, source_id, pos, pos_lang, gloss, note, ord)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    sense.id,
                    entry.id,
                    entry.source_id,
                    sense.pos,
                    sense.pos_lang,
                    sense.gloss,
                    sense.note,
                    sense.ord
                ],
            )
            .unwrap_or_else(|e| panic!("nạp dict_sense {}: {e}", sense.id));

            for (id, text, translation, translation_lang, ord) in sense.examples {
                conn.execute(
                    "INSERT INTO dict_example (id, sense_id, text, translation, translation_lang, ord)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![id, sense.id, text, translation, translation_lang, ord],
                )
                .unwrap_or_else(|e| panic!("nạp dict_example {id}: {e}"));
            }

            for (id, text, work, author, ord) in sense.citations {
                conn.execute(
                    "INSERT INTO dict_citation (id, sense_id, text, work, author, ord)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![id, sense.id, text, work, author, ord],
                )
                .unwrap_or_else(|e| panic!("nạp dict_citation {id}: {e}"));
            }
        }

        // `char_idx` chỉ sinh cho hàng `zh`, phủ **cả** `headword` lẫn `headword_simp`.
        if entry.lang != "zh" {
            continue;
        }
        let mut seen = BTreeSet::new();
        for c in entry
            .headword
            .chars()
            .chain(entry.simp.unwrap_or("").chars())
        {
            if is_han(c) && seen.insert(c) {
                conn.execute(
                    "INSERT OR IGNORE INTO char_idx (ch, entry_id) VALUES (?1, ?2)",
                    rusqlite::params![c.to_string(), entry.id],
                )
                .unwrap_or_else(|e| panic!("nạp char_idx {c}: {e}"));
            }
        }
    }

    // 🔴 `entry_fts` là external-content ⇒ nó ⛔ **không** tự đầy. Thiếu dòng này, nhánh 3
    // trả rỗng **cho từng tệp** và mọi ca của nó "xanh" theo đúng cách sai nhất.
    conn.execute_batch("INSERT INTO entry_fts(entry_fts) VALUES('rebuild');")
        .unwrap_or_else(|e| panic!("rebuild entry_fts: {e}"));

    conn.execute_batch(&format!("PRAGMA user_version = {user_version};"))
        .unwrap_or_else(|e| panic!("đặt user_version: {e}"));

    conn.close()
        .unwrap_or_else(|(_, e)| panic!("đóng fixture: {e}"));

    path
}

/// Dựng **cả ba** lớp vào một thư mục, đúng phiên bản lược đồ ứng dụng biết.
fn build_all_layers(dir: &Path) {
    for seed in LAYERS {
        build_layer(
            dir,
            seed,
            &SUPPORTED_SCHEMA_VERSION.to_string(),
            SUPPORTED_SCHEMA_VERSION,
        );
    }
}

/// Danh tính lớp theo đúng thứ tự tập lớp trả về.
fn layer_ids(layers: &DictLayers) -> Vec<String> {
    layers
        .layers()
        .iter()
        .map(|layer| layer.layer().to_owned())
        .collect()
}

/// `(mã nguồn, các đầu mục)` của từng nhóm, theo đúng thứ tự kết quả.
fn groups_of(result: &auratranslate_lib::core::dict::GroupedLookup) -> Vec<(String, Vec<String>)> {
    result
        .groups
        .iter()
        .map(|group| {
            (
                group.source.code.clone(),
                group
                    .entries
                    .iter()
                    .map(|hit| hit.headword.clone())
                    .collect(),
            )
        })
        .collect()
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC3 — tập lớp phát hiện bằng QUÉT THƯ MỤC, thứ tự là một GIÁ TRỊ quan sát được
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC3** — thứ tự là `base` trước rồi mã lớp tăng dần, và nó ⛔ **không** phải thứ tự
/// `read_dir` trả về.
///
/// Ca này chỉ có nghĩa vì tên tệp xếp **ngược** thứ tự lớp: `aaa.db` mang `vp-fixture`,
/// `zzz.db` mang `base`. Một cài đặt tin vào `read_dir` cho `["vp-fixture", "hv-fixture",
/// "base"]` trên macOS và một thứ tự **khác** trên Windows — tức một cổng chỉ đỏ trên
/// **một** nhánh của ma trận CI (NFR14).
#[test]
fn the_layer_order_is_deterministic_and_never_the_directory_order() {
    let dir = temp_dir("order");
    build_all_layers(&dir);

    let layers = DictLayers::open(&dir);

    assert_eq!(
        layer_ids(&layers),
        EXPECTED_LAYER_ORDER,
        "thứ tự lớp phải là `base` trước rồi mã lớp TĂNG DẦN, ⛔ không phải thứ tự \
         `read_dir`. Tên tệp ở fixture này xếp ngược có chủ ý (aaa=vp, mmm=hv, zzz=base)."
    );
    assert!(
        layers.skipped().is_empty(),
        "ba tệp hợp lệ mà có lớp bị bỏ qua: {:?}",
        layers.skipped()
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC3 vế cuối** — thư mục ⛔ không tồn tại, hoặc rỗng ⇒ **tập lớp RỖNG**, ⛔ không
/// lỗi, ⛔ không panic.
///
/// Đây ⛔ **không** phải một ca phòng xa: `src-tauri/resources/dict/` hôm nay **rỗng**
/// (⛔ không tệp `.db` nào trong git — AD-25) và `bundle.resources` chưa mang thư mục đó
/// (Story 10.1). *"⛔ Không có lớp nào"* là một trạng thái **bình thường có tên**, và nó
/// là chính hình dạng FR36 đòi hỏi.
#[test]
fn a_missing_or_empty_directory_is_an_empty_layer_set_not_an_error() {
    let dir = temp_dir("empty");

    let empty = DictLayers::open(&dir);
    assert!(empty.layers().is_empty(), "thư mục rỗng ⇒ tập lớp rỗng");
    assert!(
        empty.skipped().is_empty(),
        "thư mục rỗng ⇒ ⛔ không lớp nào bị bỏ qua"
    );

    let missing = DictLayers::open(&dir.join("khong-ton-tai"));
    assert!(
        missing.layers().is_empty(),
        "thư mục ⛔ không tồn tại ⇒ tập lớp rỗng, ⛔ KHÔNG lỗi"
    );

    // Và một lượt tra trên tập rỗng vẫn trả **đường đã đi**, ⛔ không panic.
    let result = lookup_grouped(&empty, "山", LookupMode::Exact);
    assert_eq!(result.route, QueryRoute::Zh);
    assert_eq!(result.branch, QueryBranch::ExactBtree);
    assert!(result.groups.is_empty());

    cleanup(&dir);
}

/// Tệp ⛔ không phải `.db` ⇒ ⛔ **không** được thử mở, và ⛔ không vào danh sách bỏ qua.
#[test]
fn only_db_files_are_probed() {
    let dir = temp_dir("ext");
    build_all_layers(&dir);
    fs::write(dir.join("README.md"), b"khong phai mot tep .db")
        .unwrap_or_else(|e| panic!("ghi README: {e}"));
    fs::write(dir.join("notes.txt"), b"cung khong phai")
        .unwrap_or_else(|e| panic!("ghi notes: {e}"));

    let layers = DictLayers::open(&dir);

    assert_eq!(layer_ids(&layers), EXPECTED_LAYER_ORDER);
    assert!(
        layers.skipped().is_empty(),
        "tệp ⛔ không mang đuôi `.db` ⛔ KHÔNG phải một lớp bị bỏ qua — nó ⛔ không phải \
         một lớp: {:?}",
        layers.skipped()
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC4 — một lớp hỏng / thiếu / QUÁ MỚI ⇒ bỏ qua CÓ TÊN, các lớp còn lại vẫn tra được
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC4** — bốn cách một tệp hỏng, và **cả bốn** phải thành một **GIÁ TRỊ** mang
/// đường dẫn + lý do, ⛔ không phải một dòng `eprintln!`.
#[test]
fn a_broken_layer_is_skipped_by_name_and_the_rest_still_answer() {
    let dir = temp_dir("broken");
    build_all_layers(&dir);

    // (a) ⛔ không phải một tệp SQLite.
    fs::write(
        dir.join("garbage.db"),
        b"day khong phai mot database SQLite",
    )
    .unwrap_or_else(|e| panic!("ghi garbage: {e}"));

    // (b) Một tệp SQLite hợp lệ nhưng ⛔ không có `dict_meta`.
    {
        let conn = rusqlite::Connection::open(dir.join("wrong-schema.db"))
            .unwrap_or_else(|e| panic!("dựng wrong-schema: {e}"));
        conn.execute_batch("CREATE TABLE something_else (id INTEGER PRIMARY KEY);")
            .unwrap_or_else(|e| panic!("dựng bảng lạ: {e}"));
        conn.close().unwrap_or_else(|(_, e)| panic!("đóng: {e}"));
    }

    // (c) `PRAGMA user_version` LỚN HƠN phiên bản ứng dụng biết.
    build_layer(
        &dir,
        &LayerSeed {
            file: "too-new.db",
            layer: "too-new-fixture",
            sources: &[(1, "fx-too-new", "Fixture Too New")],
            entries: HV_ENTRIES,
        },
        &(SUPPORTED_SCHEMA_VERSION + 1).to_string(),
        SUPPORTED_SCHEMA_VERSION + 1,
    );

    // (d) Hai chỗ ghi phiên bản **NÓI KHÁC NHAU** ⇒ tệp ⛔ không do `tools/dict-build`
    //     viết ra, và tin nửa nào cũng là đoán (Story 1.9 §Quyết định #2).
    build_layer(
        &dir,
        &LayerSeed {
            file: "disagreeing.db",
            layer: "disagreeing-fixture",
            sources: &[(1, "fx-disagree", "Fixture Disagree")],
            entries: HV_ENTRIES,
        },
        "99",
        SUPPORTED_SCHEMA_VERSION,
    );

    let layers = DictLayers::open(&dir);

    assert_eq!(
        layer_ids(&layers),
        EXPECTED_LAYER_ORDER,
        "bốn tệp hỏng ⛔ KHÔNG được kéo theo ba lớp lành"
    );

    let skipped: Vec<(String, SkipReason)> = layers
        .skipped()
        .iter()
        .map(|s| {
            (
                s.path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                s.reason.clone(),
            )
        })
        .collect();

    assert_eq!(skipped.len(), 4, "chờ đúng bốn lớp bị bỏ qua: {skipped:?}");

    let reason_of = |file: &str| -> SkipReason {
        skipped
            .iter()
            .find(|(name, _)| name == file)
            .map(|(_, reason)| reason.clone())
            .unwrap_or_else(|| panic!("⛔ không thấy {file} trong danh sách bỏ qua: {skipped:?}"))
    };

    assert!(
        matches!(reason_of("garbage.db"), SkipReason::MetaUnreadable { .. }),
        "tệp rác: {:?}",
        reason_of("garbage.db")
    );
    assert!(
        matches!(
            reason_of("wrong-schema.db"),
            SkipReason::MetaUnreadable { .. }
        ),
        "lược đồ lạ: {:?}",
        reason_of("wrong-schema.db")
    );
    assert_eq!(
        reason_of("too-new.db"),
        SkipReason::SchemaTooNew {
            file_version: SUPPORTED_SCHEMA_VERSION + 1,
            supported: SUPPORTED_SCHEMA_VERSION,
        },
        "một tệp MỚI HƠN ứng dụng phải bị từ chối với lý do RIÊNG"
    );
    assert_eq!(
        reason_of("disagreeing.db"),
        SkipReason::SchemaVersionDisagrees {
            user_version: SUPPORTED_SCHEMA_VERSION,
            meta_version: "99".to_owned(),
        },
        "hai chỗ ghi phiên bản nói khác nhau phải có lý do RIÊNG"
    );

    // 🔴 Vế *"các lớp còn lại vẫn tra được BÌNH THƯỜNG"* — ⛔ không chỉ *"vẫn nạp được"*.
    let result = lookup_grouped(&layers, "山", LookupMode::Exact);
    assert_eq!(
        groups_of(&result)
            .into_iter()
            .map(|(code, _)| code)
            .collect::<Vec<_>>(),
        vec!["fx-core-a", "fx-hv", "fx-vp"],
        "bốn tệp hỏng ⛔ KHÔNG được làm hỏng lượt tra của ba lớp lành"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC6 vế cuối** — hai lớp khai **cùng một `code`** là một **lỗi dữ liệu CÓ TÊN**,
/// ⛔ **không** phải một lượt gộp im lặng hai tệp vào một nhóm.
#[test]
fn two_layers_claiming_the_same_source_code_is_a_named_data_error() {
    let dir = temp_dir("dupcode");

    build_layer(
        &dir,
        &LayerSeed {
            file: "first.db",
            layer: "aaa-first",
            sources: &[(1, "fx-collide", "Fixture Collide One")],
            entries: HV_ENTRIES,
        },
        &SUPPORTED_SCHEMA_VERSION.to_string(),
        SUPPORTED_SCHEMA_VERSION,
    );
    build_layer(
        &dir,
        &LayerSeed {
            file: "second.db",
            layer: "bbb-second",
            sources: &[(1, "fx-collide", "Fixture Collide Two")],
            entries: HV_ENTRIES,
        },
        &SUPPORTED_SCHEMA_VERSION.to_string(),
        SUPPORTED_SCHEMA_VERSION,
    );

    let layers = DictLayers::open(&dir);

    assert_eq!(
        layer_ids(&layers),
        vec!["aaa-first"],
        "lớp đầu tiên theo thứ tự tất định được giữ; lớp thứ hai bị bỏ qua"
    );
    assert_eq!(layers.skipped().len(), 1, "{:?}", layers.skipped());
    assert_eq!(
        layers.skipped()[0].reason,
        SkipReason::DuplicateSourceCode {
            code: "fx-collide".to_owned(),
            first_layer: "aaa-first".to_owned(),
        },
        "trùng `code` giữa hai lớp phải là một lý do CÓ TÊN"
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC5 — `pick_route` một lần, `branch` là thuộc tính của CẢ LƯỢT TRA
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC5** — `route` và `branch` xuất hiện **một lần** trong kết quả gom, ⛔ không phải
/// một lần cho mỗi tệp.
#[test]
fn the_route_and_the_branch_are_one_value_of_the_whole_lookup() {
    let dir = temp_dir("route");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let exact = lookup_grouped(&layers, "山", LookupMode::Exact);
    assert_eq!(exact.route, QueryRoute::Zh);
    assert_eq!(exact.branch, QueryBranch::ExactBtree);

    let one_char = lookup_grouped(&layers, "山", LookupMode::Substring);
    assert_eq!(
        one_char.branch,
        QueryBranch::CharIdx,
        "1 ký tự Hán ⇒ nhánh 2"
    );

    let three_chars = lookup_grouped(&layers, "中國人", LookupMode::Substring);
    assert_eq!(
        three_chars.branch,
        QueryBranch::FtsTrigram,
        "3 ký tự Hán ⇒ nhánh 3 — và fixture ⛔ không có đầu mục nào khớp, nên nhánh là \
         thứ DUY NHẤT quan sát được ở đây"
    );

    let english = lookup_grouped(&layers, "dictionary", LookupMode::Exact);
    assert_eq!(
        english.route,
        QueryRoute::En,
        "⛔ không ký tự Hán ⇒ đường `En`"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC5 mệnh đề cuối** — `NoBranchQueryTooShort` **sống sót qua tầng gom**.
///
/// ⛔ **Không phải "không có kết quả":** hai câu đó dẫn người dùng đi hai đường khác nhau
/// (AD-44 ④), và **1.17 đọc đúng trường này** để nói *"truy vấn quá ngắn"*. Một tầng gom
/// dịch nó thành `groups: []` là làm mệnh đề đó ⛔ không nghiệm thu được ở story sau.
#[test]
fn the_query_too_short_state_survives_the_grouping_layer() {
    let dir = temp_dir("tooshort");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    for query in ["", "a", "ab"] {
        let result = lookup_grouped(&layers, query, LookupMode::Substring);
        assert_eq!(
            result.branch,
            QueryBranch::NoBranchQueryTooShort,
            "truy vấn {query:?} (chuỗi con tiếng Anh < 3 ký tự) phải giữ nguyên trạng thái \
             KHÔNG HỖ TRỢ qua tầng gom"
        );
        assert!(
            result.groups.is_empty(),
            "truy vấn {query:?} ⇒ ⛔ không nhóm nào"
        );
        assert!(
            result.skipped.is_empty(),
            "truy vấn {query:?}: *rỗng vì quá ngắn* ⛔ KHÔNG phải *rỗng vì một lớp hỏng*"
        );
    }

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC6 — nhóm theo NGUỒN, khoá gom là `code`, và ⛔ KHÔNG hợp nhất
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC6, và đây là bẫy im lặng nhất của cả story.**
///
/// Ba tệp fixture đều dùng `dict_source.id = 1`, đúng như ba tệp thật. Gom theo `id` dán
/// nhãn *"Fixture Han Viet"* cho một đầu mục thật ra của lớp nền — **FR31 vỡ, ⛔ không
/// lỗi, ⛔ không test hành vi nào đỏ** trừ ca này.
#[test]
fn groups_are_keyed_by_the_source_code_not_by_the_numeric_id() {
    let dir = temp_dir("groupkey");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = lookup_grouped(&layers, "山", LookupMode::Exact);

    assert_eq!(
        groups_of(&result),
        vec![
            ("fx-core-a".to_owned(), vec!["山".to_owned()]),
            ("fx-hv".to_owned(), vec!["山".to_owned()]),
            ("fx-vp".to_owned(), vec!["山".to_owned()]),
        ],
        "ba nguồn khác nhau, cả ba mang `dict_source.id = 1` trong tệp của chúng"
    );

    // Mỗi nhóm mang `display_name` **của chính tệp chứa nó**.
    let names: Vec<&str> = result
        .groups
        .iter()
        .map(|g| g.source.display_name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["Fixture Core A", "Fixture Han Viet", "Fixture VietPhrase"]
    );

    // Và mỗi nhóm biết mình thuộc **lớp** nào — đó là đường 1.17 gọi pha hai.
    let layers_of_groups: Vec<&str> = result.groups.iter().map(|g| g.layer.as_str()).collect();
    assert_eq!(layers_of_groups, EXPECTED_LAYER_ORDER);

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC6 / FR32** — hai nguồn **bất đồng** ⇒ **cả hai nhóm có mặt**, ⛔ không nhóm nào
/// bị chọn làm *"câu trả lời"*.
#[test]
fn two_sources_that_disagree_both_survive_with_their_meanings_intact() {
    let dir = temp_dir("disagree");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = lookup_grouped(&layers, "中國", LookupMode::Exact);

    assert_eq!(
        groups_of(&result),
        vec![
            ("fx-core-a".to_owned(), vec!["中國".to_owned()]),
            ("fx-vp".to_owned(), vec!["中國".to_owned()]),
        ],
        "lớp nền nói `China`, VietPhrase nói `Trung Quốc` — ⛔ KHÔNG nhóm nào được biến mất"
    );

    // Và nghĩa **mâu thuẫn** phải đi hết đường ra tới bản ghi, ⛔ không bị chọn một cái.
    let base = layers.layer("base").expect("lớp nền");
    let base_glosses: Vec<String> = base
        .senses(&[2])
        .expect("đọc nghĩa lớp nền")
        .into_iter()
        .map(|sense| sense.gloss)
        .collect();
    assert_eq!(base_glosses, vec!["China".to_owned()]);

    let vp = layers.layer("vp-fixture").expect("lớp VietPhrase");
    let vp_glosses: Vec<String> = vp
        .senses(&[1])
        .expect("đọc nghĩa VietPhrase")
        .into_iter()
        .map(|sense| sense.gloss)
        .collect();
    assert_eq!(
        vp_glosses,
        vec![
            "Trung Quốc".to_owned(),
            "nước Tàu".to_owned(),
            "Trung Hoa".to_owned()
        ],
        "🔴 hai nghĩa đầu CÙNG `ord = 0` — thứ tự chỉ tất định nhờ khoá phụ `id` (Bẫy 1)"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC6** — một nguồn **đã tra mà ⛔ không khớp gì** ⇒ ⛔ **không sinh nhóm rỗng**.
///
/// Trạng thái đó phải phân biệt được với *"lớp ⛔ không nạp được"*, và chỗ phân biệt là
/// danh sách `skipped`. Hai thứ đó ⛔ **không** được phép trông giống nhau ở 1.17.
#[test]
fn a_source_that_matched_nothing_produces_no_empty_group() {
    let dir = temp_dir("nogroup");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    // `高山` chỉ có ở lớp nền, và chỉ ở nguồn `fx-core-b`.
    let result = lookup_grouped(&layers, "高山", LookupMode::Exact);
    assert_eq!(
        groups_of(&result),
        vec![("fx-core-b".to_owned(), vec!["高山".to_owned()])],
        "ba lớp được tra, một lớp khớp ⇒ ĐÚNG MỘT nhóm, ⛔ không hai nhóm rỗng đi kèm"
    );
    assert!(
        result.skipped.is_empty(),
        "⛔ không lớp nào hỏng ⇒ danh sách bỏ qua RỖNG — đó là thứ phân biệt *đã tra mà \
         không khớp* với *chưa bao giờ được tra*"
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC7 — một từ nhiều TỪ LOẠI ⇒ nhiều mục riêng biệt (FR29)
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC7** — mỗi hàng `dict_sense` là **một mục riêng**, ⛔ không nối `gloss` thành một
/// chuỗi.
///
/// Một chuỗi nối là một quyết định **trình bày** chôn vào tầng dữ liệu, và 1.17 ⛔ không gỡ
/// ngược ra được: `"mountain; surname Shan"` ⛔ không nói được từ loại nào đi với nghĩa nào.
#[test]
fn a_word_with_many_parts_of_speech_becomes_many_separate_records() {
    let dir = temp_dir("manysenses");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);
    let base = layers.layer("base").expect("lớp nền");

    let senses = base.senses(&[1]).expect("đọc nghĩa của 山");

    assert_eq!(
        senses.len(),
        2,
        "hai hàng `dict_sense` ⇒ HAI mục, ⛔ không một chuỗi nối"
    );
    assert_eq!(
        senses.iter().map(|s| s.gloss.as_str()).collect::<Vec<_>>(),
        vec!["mountain", "surname Shan"]
    );
    assert_eq!(
        senses.iter().map(|s| s.pos.as_deref()).collect::<Vec<_>>(),
        vec![Some("noun"), Some("proper noun")],
        "mỗi mục mang từ loại CỦA CHÍNH NÓ"
    );
    assert_eq!(
        senses.iter().map(|s| s.ord).collect::<Vec<_>>(),
        vec![0, 1],
        "thứ tự là `ord` tăng dần"
    );

    // `note` — phần **thứ sáu** trong sáu phần FR28 liệt kê, và nó đi CÙNG MỤC.
    assert_eq!(senses[0].note.as_deref(), Some("base layer note"));
    assert_eq!(senses[1].note, None);

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC7 vế khoá phụ `id`, và đây là Bẫy 1 của story.**
///
/// `tools/dict-build/src/sources/vietphrase.rs` tách `/` **vô điều kiện** ⇒ nhiều
/// `dict_sense` **cùng `ord`** (`deferred-work.md`, Story 1.10). Với `ORDER BY ord` trần,
/// hai lượt chạy cho hai thứ tự — tức một ca **flaky**, và một ca flaky **bị gỡ** chứ ⛔
/// không được sửa.
///
/// ⚠️ Ca này khẳng định **kết quả**; luật *"⛔ không `ORDER BY ord` trần"* được cưỡng chế
/// riêng bằng máy ở `tests/dict_boundary.rs::every_ord_ordering_carries_its_tiebreaker` —
/// hai lớp cần **cả hai**, vì trên một tập nhỏ SQLite thường trả đúng thứ tự **do may mắn**,
/// và một ca hành vi một mình ⛔ không phân biệt được may mắn với đúng.
#[test]
fn senses_sharing_one_ord_are_still_ordered_deterministically() {
    let dir = temp_dir("sameord");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);
    let vp = layers.layer("vp-fixture").expect("lớp VietPhrase");

    let senses = vp.senses(&[1]).expect("đọc nghĩa của 中國");

    assert_eq!(
        senses
            .iter()
            .map(|s| (s.ord, s.sense_id))
            .collect::<Vec<_>>(),
        vec![(0, 1), (0, 2), (1, 3)],
        "hai nghĩa đầu CÙNG `ord = 0`; thứ tự chỉ tất định nhờ khoá phụ `id`"
    );

    layers.close();
    cleanup(&dir);
}

/// Đầu mục ⛔ không có nghĩa nào ⇒ **danh sách rỗng**, ⛔ không lỗi. Và một `entry_id` ⛔
/// không tồn tại cũng vậy — pha hai ⛔ không phải một phép kiểm tồn tại.
#[test]
fn an_entry_without_senses_is_an_empty_list_not_an_error() {
    let dir = temp_dir("nosense");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);
    let base = layers.layer("base").expect("lớp nền");

    assert!(
        base.senses(&[6])
            .expect("đầu mục ⛔ không có nghĩa")
            .is_empty(),
        "một đầu mục chỉ mang âm đọc là HỢP LỆ"
    );
    assert!(
        base.senses(&[9_999])
            .expect("id ⛔ không tồn tại")
            .is_empty(),
        "một `entry_id` lạ ⇒ rỗng, ⛔ không lỗi"
    );
    assert!(
        base.senses(&[]).expect("tập rỗng").is_empty(),
        "tập rỗng ⇒ ⛔ không một lượt chạm database nào"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC13 vế *"⛔ không N+1"*, đo được ở tầng hành vi.**
///
/// Một tập id trải trên **nhiều lô** *(200 id ⇒ 4 lô ở cỡ [`SENSE_BATCH`] = 64)* phải cho
/// **đúng cùng** kết quả với tập id thật, ⛔ không trùng một hàng nào và ⛔ không rơi hàng
/// nào. Đây là ca duy nhất chạm được phép **đệm lô cuối**: lô cuối lặp lại một id đã hỏi,
/// và `IN` là phép kiểm **tập hợp** — một cài đặt đệm bằng phép **nối** sẽ nhân đôi hàng ở
/// đây và ⛔ không ở đâu khác.
#[test]
fn reading_senses_across_many_batches_never_duplicates_or_drops_a_row() {
    let dir = temp_dir("batches");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);
    let base = layers.layer("base").expect("lớp nền");

    let straight = base.senses(&[1, 2, 3]).expect("ba đầu mục");

    // 200 id: ba đầu mục thật cộng một đuôi dài id ⛔ không tồn tại — bốn lô, lô cuối đệm.
    let mut many: Vec<i64> = vec![1, 2, 3];
    many.extend(1_000..1_197);
    assert!(many.len() > SENSE_BATCH * 3, "phải trải qua ÍT NHẤT bốn lô");

    let batched = base.senses(&many).expect("nhiều lô");

    assert_eq!(
        batched, straight,
        "chia lô là chi tiết CÀI ĐẶT — nó ⛔ KHÔNG được đổi kết quả. Trùng hàng ở đây là \
         một phép đệm sai; thiếu hàng là một lô bị bỏ."
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 Một `entry_id` **lặp lại ở HAI LÔ khác nhau** — ⛔ không phải trong cùng một lô, ca
/// đã canh ở test phía trên bằng phép đệm.
///
/// `IN (...)` khử trùng được **trong** một lô (ngữ nghĩa tập hợp), nhưng nếu cùng một
/// `entry_id` rơi vào hai lô khác nhau, `SENSE_SQL` chạy hai lần cho nó — sinh hai
/// `SenseRecord` giống hệt nhau, và bước gộp ví dụ/trích dẫn theo `sense_id` (dùng
/// `HashMap::remove`) chỉ còn nạp được cho bản đầu; bản lặp nhận danh sách rỗng một cách
/// im lặng. `read_senses` khử trùng `entry_ids` **trước khi chia lô** chính vì ca này.
#[test]
fn a_duplicate_entry_id_spanning_two_batches_is_not_double_counted() {
    let dir = temp_dir("dup-batches");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);
    let base = layers.layer("base").expect("lớp nền");

    let once = base.senses(&[1]).expect("một đầu mục");

    // Lấp đầy lô ĐẦU bằng id giả để id `1` lặp lại rơi đúng vào lô THỨ HAI.
    let mut ids: Vec<i64> = vec![1];
    ids.extend(2_000..2_000 + SENSE_BATCH as i64);
    ids.push(1);

    let with_duplicate = base.senses(&ids).expect("danh sách mang id lặp");

    assert_eq!(
        with_duplicate, once,
        "một `entry_id` lặp lại ở hai lô khác nhau phải chỉ sinh ĐÚNG MỘT `SenseRecord` \
         mang đủ ví dụ/trích dẫn — ⛔ không phải hai bản, và ⛔ không phải một bản kèm một \
         bản rỗng"
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC8 — ví dụ gắn theo TỪ LOẠI; trích dẫn là trường RIÊNG có xuất xứ (FR30)
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC8** — ví dụ treo vào **`sense_id`**, ⛔ không vào `entry_id`.
///
/// Lược đồ đã cưỡng chế điều này (`dict_example.sense_id REFERENCES dict_sense(id)`), và
/// đọc bằng một `JOIN` vòng qua `entry_id` là **tự đánh mất** nó. Ca này chỉ có nghĩa vì
/// đầu mục `山` của lớp nền có **hai** nghĩa và **chỉ nghĩa thứ nhất** có ví dụ — một cài
/// đặt treo theo đầu mục sẽ gắn ví dụ đó vào **cả hai**.
#[test]
fn examples_hang_on_the_part_of_speech_not_on_the_entry() {
    let dir = temp_dir("examples");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);
    let base = layers.layer("base").expect("lớp nền");

    let senses = base.senses(&[1]).expect("đọc nghĩa của 山");

    assert_eq!(senses[0].examples.len(), 1, "nghĩa `noun` có một ví dụ");
    assert_eq!(senses[0].examples[0].text, "高山");
    assert_eq!(
        senses[1].examples.len(),
        0,
        "nghĩa `proper noun` ⛔ KHÔNG có ví dụ — một cài đặt treo ví dụ theo `entry_id` sẽ \
         gắn ví dụ của nghĩa thứ nhất vào đây"
    );

    // `translation_lang` ⛔ không bỏ được: nó là thứ AC10 dùng để nói *"bản dịch ví dụ này
    // là tiếng Anh"*.
    assert_eq!(
        senses[0].examples[0].translation.as_deref(),
        Some("high mountain")
    );
    assert_eq!(
        senses[0].examples[0].translation_lang.as_deref(),
        Some("en")
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC8 vế trích dẫn** — một danh sách **RIÊNG** với ví dụ, mang `work` và `author`.
///
/// ⛔ Trộn hai bảng vào một danh sách là làm mất đúng thứ FR30 phân biệt: một *ví dụ* do
/// người biên soạn đặt ra, một *trích dẫn* đến từ một tác phẩm **có tên và có tác giả**.
#[test]
fn citations_are_a_separate_list_carrying_their_provenance() {
    let dir = temp_dir("citations");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);
    let hv = layers.layer("hv-fixture").expect("lớp Hán Việt");

    let senses = hv.senses(&[1]).expect("đọc nghĩa của 山");
    assert_eq!(senses.len(), 1);

    assert_eq!(
        senses[0].examples.len(),
        1,
        "ví dụ và trích dẫn ⛔ KHÔNG trộn vào nhau"
    );
    assert_eq!(senses[0].examples[0].text, "山川");

    assert_eq!(senses[0].citations.len(), 1);
    assert_eq!(senses[0].citations[0].text, "山中無曆日");
    assert_eq!(
        senses[0].citations[0].work.as_deref(),
        Some("Thái Bình Quảng Ký")
    );
    assert_eq!(senses[0].citations[0].author.as_deref(), Some("Lý Phưởng"));

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC9 / AC10 — `lang` là một TRƯỜNG, `pos_lang` cũng vậy (AD-44 ⑤, FR34, FR35)
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC9 (FR34)** — mục từ **tiếng Anh** đi qua **CÙNG** đường gom và **CÙNG** hình dạng
/// bản ghi với mục tiếng Trung.
///
/// AD-44 ⑤: *"`lang` là một **trường**, ⛔ không phải một **kiểu** — ⛔ không tồn tại bản
/// ghi kết quả thứ hai dành riêng cho tiếng Anh"*. Một `EnSourceGroup` song song sẽ buộc
/// **mọi** chỗ gọi phân nhánh theo kiểu, và bước hợp nhất hai nhánh đó lại chính là thứ
/// AD-19 cấm.
#[test]
fn an_english_entry_travels_the_same_grouping_path_and_the_same_record_shape() {
    let dir = temp_dir("english");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = lookup_grouped(&layers, "lock", LookupMode::Exact);
    assert_eq!(result.route, QueryRoute::En);
    assert_eq!(result.branch, QueryBranch::ExactBtree);
    assert_eq!(
        groups_of(&result),
        vec![("fx-core-a".to_owned(), vec!["lock".to_owned()])]
    );
    assert_eq!(
        result.groups[0].entries[0].lang, "en",
        "`lang` đi ra như một TRƯỜNG của cùng một bản ghi"
    );

    let base = layers.layer("base").expect("lớp nền");
    let senses = base.senses(&[4]).expect("đọc nghĩa của `lock`");
    assert_eq!(senses.len(), 1);
    assert_eq!(
        senses[0].pos.as_deref(),
        Some("danh từ"),
        "mục tiếng Anh mang nhãn từ loại"
    );
    assert_eq!(
        senses[0].gloss, "ổ khoá",
        "và nghĩa TIẾNG VIỆT — đó là toàn bộ FR34"
    );

    // 🔴 *"Một hình dạng bản ghi"* là một mệnh đề của **KIỂU**, và cách nghiệm thu nó là
    // xếp cả hai vào **cùng một** bộ sưu tập: một `EnSenseRecord` song song ⛔ **không biên
    // dịch** ở dòng dưới đây. Đó là phép cưỡng chế mạnh nhất có thể cho AD-44 ⑤, và nó
    // mạnh hơn mọi `assert!` vì nó đỏ ở **thời điểm biên dịch**.
    let zh = base.senses(&[1]).expect("đọc nghĩa của 山");
    let one_shape: Vec<&SenseRecord> = vec![&senses[0], &zh[0]];
    assert_eq!(
        one_shape
            .iter()
            .map(|sense| sense.pos_lang.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("vi"), Some("en")],
        "hai mục khác ngôn ngữ, MỘT bản ghi — khác nhau ở GIÁ TRỊ của một trường"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC10 (FR35)** — nhãn ngoại ngữ nhận ra qua **`dict_sense.pos_lang`**, một **TRƯỜNG**.
///
/// ⛔ **Không** đoán từ nội dung `pos`, ⛔ không một bảng tra `"noun" ⇒ tiếng Anh` nào: một
/// bảng như thế sai **im lặng** với mọi nhãn nó chưa gặp, và nó sai ở **story sau**.
#[test]
fn the_foreign_pos_label_is_marked_by_a_field_not_guessed_from_its_content() {
    let dir = temp_dir("poslang");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let base = layers.layer("base").expect("lớp nền");
    let base_senses = base.senses(&[1]).expect("đọc nghĩa lớp nền");
    assert_eq!(
        base_senses
            .iter()
            .map(|s| s.pos_lang.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("en"), Some("en")],
        "lớp nền hôm nay mang nhãn từ loại NGOẠI NGỮ, và nó phải ĐÁNH DẤU ĐƯỢC"
    );
    assert!(
        !base_senses[0].examples.is_empty(),
        "AC10: mục tiếng Trung trên lớp nền mang ÍT NHẤT một ví dụ khi nguồn có dữ liệu"
    );

    let hv = layers.layer("hv-fixture").expect("lớp Hán Việt");
    assert_eq!(
        hv.senses(&[1]).expect("đọc nghĩa lớp gỡ rời")[0]
            .pos_lang
            .as_deref(),
        Some("vi"),
        "cùng một TRƯỜNG phân biệt được nhãn tiếng Việt với nhãn ngoại ngữ"
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC11 — lớp kiểu HVTĐTD, và cú RƠI VỀ nhãn tiếng Anh khi gỡ
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC11** — lớp mang `pos_lang = 'vi'` cho từ loại · ví dụ · trích dẫn **tiếng Việt**,
/// và nhóm của lớp nền vẫn **có mặt cạnh nó**.
///
/// ⚠️ **FIXTURE, ⛔ không phải dữ liệu HVTĐTD thật** — xem doc-comment của [`HV_SENSES_SHAN`].
#[test]
fn a_han_viet_shaped_layer_stands_beside_the_base_layer_not_instead_of_it() {
    let dir = temp_dir("hvshape");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = lookup_grouped(&layers, "山", LookupMode::Exact);
    let codes: Vec<&str> = result
        .groups
        .iter()
        .map(|g| g.source.code.as_str())
        .collect();
    assert!(
        codes.contains(&"fx-hv") && codes.contains(&"fx-core-a"),
        "cả hai nhóm phải có mặt — ⛔ KHÔNG nhóm nào bị chọn làm *câu trả lời* (FR32): {codes:?}"
    );

    let hv = layers.layer("hv-fixture").expect("lớp Hán Việt");
    let senses = hv.senses(&[1]).expect("đọc nghĩa");
    assert_eq!(senses[0].pos.as_deref(), Some("danh từ"));
    assert_eq!(senses[0].gloss, "núi");
    assert_eq!(
        senses[0].examples[0].translation.as_deref(),
        Some("núi sông")
    );
    assert_eq!(senses[0].citations[0].author.as_deref(), Some("Lý Phưởng"));

    // Và lớp nền cạnh nó vẫn mang nhãn NGOẠI NGỮ — hai nhãn cùng tồn tại, ⛔ không nhãn nào
    // được viết lại ở tầng này.
    let base = layers.layer("base").expect("lớp nền");
    assert_eq!(
        base.senses(&[1]).expect("đọc nghĩa lớp nền")[0]
            .pos_lang
            .as_deref(),
        Some("en")
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 AC12 — FR36 HÀNH VI: xoá tệp `.db` của một lớp gỡ rời BẤT KỲ
// ═════════════════════════════════════════════════════════════════════════════════

/// Bộ mệnh đề tra cứu **⛔ không phụ thuộc một lớp gỡ rời nào**.
///
/// 🔴 Đây là hợp đồng của AC12: **cùng** hàm này chạy trước và sau khi xoá, ⛔ không một
/// nhánh `#[cfg]` nào, ⛔ không một tham số *"lớp X có mặt ⛔ không"* nào. Nếu một mệnh đề
/// dưới đây phải biết lớp nào đang có, nó ⛔ **không thuộc về đây**.
fn the_layer_independent_lookups_still_hold(layers: &DictLayers) {
    let shan = lookup_grouped(layers, "山", LookupMode::Exact);
    assert_eq!(shan.route, QueryRoute::Zh);
    assert_eq!(shan.branch, QueryBranch::ExactBtree);
    assert!(
        shan.skipped.is_empty(),
        "một tệp ĐÃ XOÁ ⛔ KHÔNG phải một lớp *bị bỏ qua* — nó ⛔ không còn là một lớp: {:?}",
        shan.skipped
    );
    let base_group = shan
        .groups
        .iter()
        .find(|group| group.source.code == "fx-core-a")
        .expect("lớp nền phải luôn trả lời cho 山");
    assert_eq!(base_group.layer, "base");
    assert_eq!(
        base_group
            .entries
            .iter()
            .map(|h| h.headword.as_str())
            .collect::<Vec<_>>(),
        vec!["山"]
    );

    // 🔴 *"Rơi về nhãn tiếng Anh của lớp nền, ⛔ không có đường tra cứu nào hỏng"*
    // (`epics.md:1575`) — và nó phải đúng **kể cả khi lớp Hán Việt còn đó**.
    let base = layers.layer("base").expect("lớp nền luôn có mặt");
    let senses = base.senses(&[1]).expect("đọc nghĩa lớp nền");
    assert_eq!(senses.len(), 2);
    assert_eq!(senses[0].pos_lang.as_deref(), Some("en"));
    assert!(!senses[0].examples.is_empty());

    // Một nguồn khác của **cùng** tệp nền vẫn tra được — nhóm theo `code`, ⛔ không theo tệp.
    let gaoshan = lookup_grouped(layers, "高山", LookupMode::Exact);
    assert_eq!(
        groups_of(&gaoshan),
        vec![("fx-core-b".to_owned(), vec!["高山".to_owned()])]
    );

    // Đường tiếng Anh cũng vậy — nó ⛔ không đi qua một lớp gỡ rời nào.
    let lock = lookup_grouped(layers, "lock", LookupMode::Exact);
    assert_eq!(
        groups_of(&lock),
        vec![("fx-core-a".to_owned(), vec!["lock".to_owned()])]
    );

    // Và trạng thái *"quá ngắn"* vẫn là chính nó.
    assert_eq!(
        lookup_grouped(layers, "ab", LookupMode::Substring).branch,
        QueryBranch::NoBranchQueryTooShort
    );
}

/// 🔴 **AC12 — món nợ FR36 mở từ Story 1.10, và nó ĐÓNG Ở ĐÂY.**
///
/// *"Xoá file → chạy lại bộ test tra cứu → hệ thống vẫn hoạt động đầy đủ với các nguồn còn
/// lại"* (AD-10). `deferred-work.md` chốt: *"⛔ Không đánh dấu FR36 là 'đã nghiệm thu' cho
/// tới khi 1.13 viết phép thử này"*.
///
/// ⚠️ **Ca dễ trượt nhất, và nó trượt XANH:** một bộ test dựng fixture rồi **luôn** mở đủ
/// ba tệp sẽ *"đạt"* AC này mà ⛔ chưa bao giờ chạy đường thiếu tệp. Nên ca này **xoá tệp
/// thật** rồi **mở lại tập lớp** — và trên **Windows**, xoá một tệp còn mở là một lỗi
/// (NFR14), nên `DictLayers` phải được **drop trước** khi xoá. Đó là luật 2 của tệp này, và
/// đây là lý do luật đó tồn tại.
///
/// 🔴 Danh sách lớp gỡ rời **dẫn xuất từ chính tập lớp**, ⛔ không viết cứng: mệnh đề của
/// `epics.md:1572` là *"một lớp gỡ rời **BẤT KỲ**"*, và nó ⛔ không nghiệm thu được bằng một
/// lớp được chọn sẵn.
#[test]
fn deleting_any_detachable_layer_keeps_the_whole_lookup_suite_green() {
    let detachable = {
        let probe_dir = temp_dir("fr36-probe");
        build_all_layers(&probe_dir);
        let layers = DictLayers::open(&probe_dir);
        let found: Vec<String> = layers
            .layers()
            .iter()
            .map(|layer| layer.layer().to_owned())
            .filter(|id| id != "base")
            .collect();
        layers.close();
        cleanup(&probe_dir);
        found
    };

    assert!(
        detachable.len() >= 2,
        "fixture phải có ÍT NHẤT hai lớp gỡ rời — *một lớp bất kỳ* ⛔ không nghiệm thu được \
         trên một lớp duy nhất. Thấy: {detachable:?}"
    );

    for target in &detachable {
        let dir = temp_dir(&format!("fr36-{target}"));
        build_all_layers(&dir);

        // ── Đối chứng dương (Task 6.3) ───────────────────────────────────────────
        //
        // 🔴 ⛔ Không có nó thì *"xoá xong vẫn xanh"* và *"lớp đó chưa bao giờ được nạp"*
        // đọc **giống hệt nhau**, và ca này sẽ xanh trên một cài đặt ⛔ không bao giờ mở
        // lớp gỡ rời nào.
        let layers = DictLayers::open(&dir);
        let before = lookup_grouped(&layers, "山", LookupMode::Exact);
        assert!(
            before.groups.iter().any(|group| &group.layer == target),
            "TRƯỚC khi xoá, lớp {target} phải THẬT SỰ đóng góp một nhóm cho `山`: {:?}",
            before.groups.iter().map(|g| &g.layer).collect::<Vec<_>>()
        );
        the_layer_independent_lookups_still_hold(&layers);

        let victim = layers
            .layer(target)
            .expect("lớp vừa xác nhận có mặt")
            .path()
            .to_path_buf();

        // ⚠️ Luật 2 — thả **mọi** kết nối TRƯỚC khi xoá. Trên Windows, một tệp còn mở là
        // một `remove_file` thất bại, và cả ca này sẽ đỏ chỉ trên MỘT nhánh CI.
        drop(layers);
        fs::remove_file(&victim).unwrap_or_else(|e| panic!("xoá {}: {e}", victim.display()));

        // ── Mở LẠI tập lớp: đây là thứ phép thử đo, ⛔ không phải một lượt tra lại ─────
        let layers = DictLayers::open(&dir);
        assert!(
            layers.layer(target).is_none(),
            "lớp {target} vừa bị xoá khỏi đĩa mà vẫn nạp được"
        );
        assert_eq!(
            layers.layers().len(),
            detachable.len(),
            "xoá một lớp ⇒ đúng một lớp biến mất, ⛔ không kéo theo lớp nào khác"
        );
        assert!(
            layers.skipped().is_empty(),
            "một tệp ĐÃ XOÁ ⛔ KHÔNG được xuất hiện trong danh sách bỏ qua — *gỡ một lớp* là \
             một thao tác BÌNH THƯỜNG (FR112), ⛔ không phải một lỗi dữ liệu: {:?}",
            layers.skipped()
        );

        // 🔴 **CÙNG** bộ mệnh đề, ⛔ không sửa một ca nào, ⛔ không một nhánh `#[cfg]` nào.
        the_layer_independent_lookups_still_hold(&layers);

        layers.close();
        cleanup(&dir);
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC13 — NFR1 trên ĐƯỜNG GOM. ⛔ KHÔNG chạy trong CI.
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 Phép đo p95 của NFR1 **trên đường gom** — `#[ignore]`, lái bằng **biến môi trường**.
///
/// ```sh
/// AURA_DICT_BENCH_DIR=/duong/dan/tuyet/doi/tools/dict-build/out \
///   cargo test --release --manifest-path src-tauri/Cargo.toml --test dict_sources \
///   -- --ignored --nocapture
/// ```
///
/// 🔴 **Đường dẫn TUYỆT ĐỐI, và `--release`.** Hai vế, cả hai đều đo được:
///
/// - `cargo test` chạy nhị phân với thư mục làm việc là **`src-tauri/`**, ⛔ không phải gốc
///   kho — nên `tools/dict-build/out` tương đối trỏ vào hư không, và ca này đỏ với
///   *"⛔ không phải một thư mục"* thay vì đo.
/// - Bản **debug** chậm hơn khoảng **2×** (Story 1.11 đo 7,324 ms release so với 15,045 ms
///   debug trên cùng một nhánh). Số nghiệm thu là số của bản người dùng chạy.
///
/// ⚠️ Biến **mới**, ⛔ không dùng lại `AURA_DICT_BENCH_DB` của `dict_lookup.rs`: đường gom
/// cần một **thư mục**, còn biến kia trỏ một **tệp**. Một biến mang hai nghĩa là một biến
/// sẽ bị truyền sai đúng một lần, và lần đó cho một tập lớp rỗng — tức mọi con số ra `0`
/// hàng và bảng đo *"đạt"* theo đúng cách sai nhất.
///
/// ⛔ **Cả hai lớp chặn đều cần thiết:** `#[ignore]` (CI ⛔ không truyền `--ignored`) **và**
/// biến vắng mặt ⇒ bỏ qua. CI ⛔ không có tệp `.db` nào (`.gitignore: *.db` — AD-25).
///
/// 🔴 **⛔ KHÔNG có `assert!` ngưỡng thời gian ở đây, và đó là mệnh đề của AC13** — ⛔ không
/// phải một chỗ bỏ sót. AC13 nói thẳng: *"vượt trần ⇒ **GHI SỐ VÀ BÀN GIAO**, ⛔ KHÔNG tự
/// thêm `LIMIT`"*, vì đường ra là một **quyết định sản phẩm** chạm hợp đồng của Panel Lookup
/// (1.17). Một `assert!` ở đây biến một **bàn giao** thành một **cổng đỏ theo thiết kế**, và
/// một cổng luôn đỏ bị gỡ trong tuần. Con số in ra là thứ đi vào §Completion Notes.
#[test]
#[ignore = "can thu muc chua tep .db that; chay tay qua AURA_DICT_BENCH_DIR"]
fn bench_the_grouped_path_on_the_real_dictionaries() {
    let Ok(raw) = std::env::var("AURA_DICT_BENCH_DIR") else {
        println!("AURA_DICT_BENCH_DIR vắng mặt — bỏ qua phép đo.");
        return;
    };

    let dir = PathBuf::from(&raw);
    assert!(
        dir.is_dir(),
        "AURA_DICT_BENCH_DIR trỏ tới {} — ⛔ không phải một thư mục",
        dir.display()
    );

    let layers = DictLayers::open(&dir);
    println!("\n═══ Thư mục: {} ═══", dir.display());
    for layer in layers.layers() {
        println!(
            "  lớp {:<14} {:>2} nguồn  {}",
            layer.layer(),
            layer.sources().len(),
            layer.path().display()
        );
    }
    for skipped in layers.skipped() {
        println!(
            "  ⚠️  bỏ qua {}: {}",
            skipped.path.display(),
            skipped.reason
        );
    }
    assert!(
        !layers.layers().is_empty(),
        "⛔ không lớp nào nạp được từ {} — mọi con số dưới đây sẽ là 0 và bảng sẽ *đạt* \
         theo đúng cách sai nhất",
        dir.display()
    );

    const WARMUP: usize = 10;
    const RUNS: usize = 200;
    /// Trần **dẫn xuất**, y hệt `dict_lookup.rs`: NFR1 cho 100 ms đầu-cuối, PRD dành
    /// ~99,95 ms cho vòng IPC Tauri cộng render frontend. 10 ms giữ lại ≥ 90 ms.
    const CEILING_MS: f64 = 10.0;

    let pct = |samples: &[f64], p: f64| -> f64 {
        let idx = ((p / 100.0) * samples.len() as f64).ceil() as usize;
        samples[idx.saturating_sub(1).min(samples.len() - 1)]
    };

    // ── PHA MỘT: gom trên N tệp ───────────────────────────────────────────────────
    let cases: &[(&str, LookupMode, &str)] = &[
        ("山", LookupMode::Exact, "zh-1-btree"),
        ("山", LookupMode::Substring, "zh-2-charidx-1"),
        ("中國", LookupMode::Substring, "zh-2-charidx-2"),
        ("中國人", LookupMode::Substring, "zh-3-trigram"),
        ("running", LookupMode::Exact, "en-1-btree-lower"),
        ("Running", LookupMode::Exact, "en-1-btree-upper"),
        ("dic", LookupMode::Substring, "en-2-trigram"),
    ];

    println!(
        "\n── PHA MỘT — `lookup_grouped` trên {} lớp ──",
        layers.layers().len()
    );
    println!(
        "  {:<18} {:<10} {:>7} {:>7} {:>9} {:>9} {:>9}",
        "nhánh", "truy vấn", "nhóm", "hàng", "p50", "p95", "p99"
    );

    let mut worst = (0.0f64, String::new());
    let mut hydrate_plan: Vec<(String, String, Vec<i64>)> = Vec::new();

    for (query, mode, label) in cases {
        for _ in 0..WARMUP {
            let _ = lookup_grouped(&layers, query, *mode);
        }

        let mut samples = Vec::with_capacity(RUNS);
        for _ in 0..RUNS {
            let start = std::time::Instant::now();
            let _ = lookup_grouped(&layers, query, *mode);
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).expect("⛔ không có NaN trong phép đo"));

        let result = lookup_grouped(&layers, query, *mode);
        let rows: usize = result.groups.iter().map(|g| g.entries.len()).sum();
        let (p50, p95, p99) = (
            pct(&samples, 50.0),
            pct(&samples, 95.0),
            pct(&samples, 99.0),
        );

        println!(
            "  {label:<18} {query:<10} {:>7} {rows:>7} {p50:>8.3}ms {p95:>8.3}ms {p99:>8.3}ms",
            result.groups.len()
        );
        if p95 > worst.0 {
            worst = (p95, format!("{label} ({query})"));
        }

        // Ca xấu nhất đã biết là ca đáng đo pha hai nhất — giữ lại tập đầu mục của nó.
        if let Some(group) = result
            .groups
            .iter()
            .max_by_key(|group| group.entries.len())
            .filter(|group| !group.entries.is_empty())
        {
            hydrate_plan.push((
                (*label).to_owned(),
                group.layer.clone(),
                group.entries.iter().map(|hit| hit.entry_id).collect(),
            ));
        }
    }

    // ── PHA HAI: đọc nghĩa cho một tập đầu mục DO CHỖ GỌI CHỌN ────────────────────
    //
    // 🔴 Đo **cả tập** *(thứ phương án A của §Quyết định #1 sẽ phải trả ở pha một)* **và**
    // một trang cỡ 20 *(thứ Panel Lookup thật sẽ hỏi)*. Hai con số đó nói hai chuyện khác
    // nhau, và gộp chúng lại là mất đúng lý do §Quyết định #1 chọn B.
    const PAGE: usize = 20;

    println!("\n── PHA HAI — `senses()` theo lô (SENSE_BATCH = {SENSE_BATCH}) ──");
    println!(
        "  {:<18} {:<14} {:>7} {:>9} {:>9} {:>9}",
        "nhánh", "lớp", "đầu mục", "p50", "p95", "p99"
    );

    for (label, layer_id, entry_ids) in &hydrate_plan {
        let Some(layer) = layers.layer(layer_id) else {
            continue;
        };

        for (tag, ids) in [
            ("trang", &entry_ids[..entry_ids.len().min(PAGE)]),
            ("tất cả", &entry_ids[..]),
        ] {
            for _ in 0..WARMUP.min(3) {
                let _ = layer.senses(ids);
            }

            let runs = if ids.len() > 500 { 20 } else { RUNS };
            let mut samples = Vec::with_capacity(runs);
            for _ in 0..runs {
                let start = std::time::Instant::now();
                let _ = layer.senses(ids).expect("đọc nghĩa");
                samples.push(start.elapsed().as_secs_f64() * 1000.0);
            }
            samples.sort_by(|a, b| a.partial_cmp(b).expect("⛔ không có NaN"));
            let (p50, p95, p99) = (
                pct(&samples, 50.0),
                pct(&samples, 95.0),
                pct(&samples, 99.0),
            );

            println!(
                "  {:<18} {layer_id:<14} {:>7} {p50:>8.3}ms {p95:>8.3}ms {p99:>8.3}ms",
                format!("{label}/{tag}"),
                ids.len()
            );
            if p95 > worst.0 {
                worst = (p95, format!("{label}/{tag} ({layer_id})"));
            }
        }
    }

    println!(
        "\n  Chậm nhất: {} — p95 {:.3} ms (trần {CEILING_MS} ms) ⇒ {}",
        worst.1,
        worst.0,
        if worst.0 <= CEILING_MS {
            "DAT"
        } else {
            "VUOT TRAN — ghi so, neu nhanh, ban giao 1.17. KHONG tu them LIMIT."
        }
    );

    layers.close();
}

// ═════════════════════════════════════════════════════════════════════════════════
// Cổng parity lược đồ — cùng khuôn `dict_lookup.rs`
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 DDL của fixture phải **nguyên văn** như `tools/dict-build/src/schema.rs`.
#[test]
fn fixture_ddl_is_verbatim_from_dict_build_schema() {
    let source = read_dict_build_schema();

    for (name, ddl) in COPIED_DDL {
        let needle = ddl.replace('"', "\\\"");
        assert!(
            source.contains(&needle),
            "khối DDL `{name}` trong `tests/dict_sources.rs` ⛔ KHÔNG còn khớp nguyên văn \
             với `tools/dict-build/src/schema.rs`.\n\n\
             Lược đồ hai cây đã trôi khỏi nhau. MỌI ca trong tệp này đang kiểm một database \
             ⛔ không tồn tại trong sản phẩm.\n\n\
             Đang tìm:\n{needle}"
        );
    }

    assert!(
        COPIED_DDL.len() >= 9,
        "chỉ {} khối DDL trong `COPIED_DDL` — fixture đã bị cắt",
        COPIED_DDL.len()
    );
}

/// 🔴 [`SUPPORTED_SCHEMA_VERSION`] của `src-tauri` phải **bằng** `SCHEMA_VERSION` của
/// `tools/dict-build`.
///
/// Vì sao đây là một cổng chứ ⛔ không phải một dòng ghi chú: hai workspace tách rời **có
/// chủ ý** (AC4 của Story 1.9) nên ⛔ không có import chéo nào giữ hai hằng dính nhau. Một
/// lượt nâng `SCHEMA_VERSION` ở build tool mà quên bên đọc làm **mọi** tệp `.db` mới bị
/// AC4 từ chối với lý do *"quá mới"* — tức từ điển biến mất sạch, ⛔ không lỗi nào được
/// ném, và triệu chứng lộ ra ở tay người dùng chứ ⛔ không ở CI.
#[test]
fn the_supported_schema_version_matches_dict_build() {
    let source = read_dict_build_schema();
    let needle = format!("pub const SCHEMA_VERSION: u32 = {SUPPORTED_SCHEMA_VERSION};");

    assert!(
        source.contains(&needle),
        "`tools/dict-build/src/schema.rs` ⛔ KHÔNG chứa `{needle}`.\n\n\
         `core::dict::SUPPORTED_SCHEMA_VERSION` là {SUPPORTED_SCHEMA_VERSION}, và đường đọc \
         TỪ CHỐI mọi tệp mang `user_version` lớn hơn nó (AC4). Hai hằng lệch nhau nghĩa là \
         mọi tệp `.db` do build tool viết ra sẽ bị từ chối — từ điển biến mất sạch mà ⛔ \
         không lỗi nào được ném."
    );
}

fn read_dict_build_schema() -> String {
    let schema_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tools")
        .join("dict-build")
        .join("src")
        .join("schema.rs");

    fs::read_to_string(&schema_rs).unwrap_or_else(|e| {
        panic!(
            "đọc {}: {e}. Cổng parity ⛔ KHÔNG được nới thành `if let Ok(...)` — một tệp \
             nguồn ⛔ không đọc được là một cổng chết, ⛔ không phải một cổng đã đạt.",
            schema_rs.display()
        )
    })
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.16, Task 2 — `DictionarySource::han_viet`, method thứ ba trên cổng
//
// 🔴 Tái dùng NGUYÊN VẸN fixture ba lớp ở trên (Story 1.13) — KHÔNG một bộ fixture thứ
// hai (Testing standards của story). `EntrySeed`/`build_layer` không mang cột
// `han_viet`/`nom_reading`, nên các ca dưới đây GHI THẲNG cột đó bằng một UPDATE sau khi
// `build_all_layers` đã dựng xong ba tệp — cùng tinh thần "đo/ghi trên dữ liệu thật" mà
// story đòi, chỉ khác là dữ liệu THẬT ở đây là chính fixture đã có, không phải một fixture
// song song.
// ═════════════════════════════════════════════════════════════════════════════════

/// Cập nhật `dict_entry.han_viet` của MỘT hàng trong một tệp fixture đã dựng.
fn set_han_viet(dir: &Path, file: &str, entry_id: i64, han_viet: &str) {
    let conn = rusqlite::Connection::open(dir.join(file))
        .unwrap_or_else(|e| panic!("mở {file} để ghi han_viet: {e}"));
    conn.execute(
        "UPDATE dict_entry SET han_viet = ?1 WHERE id = ?2",
        rusqlite::params![han_viet, entry_id],
    )
    .unwrap_or_else(|e| panic!("cập nhật han_viet cho id {entry_id} trong {file}: {e}"));
    conn.close().unwrap_or_else(|(_, e)| panic!("đóng {file}: {e}"));
}

/// 🔴 **Quyết định #2** — đọc âm Hán Việt cho một ký tự, mang theo `source_code` của
/// chính hàng đã khớp.
#[test]
fn han_viet_reads_the_raw_reading_and_its_source_code() {
    let dir = temp_dir("hanviet-basic");
    build_all_layers(&dir);
    // `id = 1` của `zzz.db` (lớp nền) là đầu mục `山`, nguồn `fx-core-a`.
    set_han_viet(&dir, "zzz.db", 1, "sơn");

    let layers = DictLayers::open(&dir);
    let base = layers.layer("base").expect("lớp nền");

    let hits = base.han_viet(&["山", "國"]).expect("tra hai ký tự");

    assert_eq!(
        hits,
        vec![HanVietHit {
            character: "山".to_owned(),
            reading: "sơn".to_owned(),
            source_code: "fx-core-a".to_owned(),
        }],
        "chỉ `山` mang han_viet; `國` chưa được ghi ⇒ ⛔ KHÔNG một hàng nào cho nó \
         (bộ lọc `IS NOT NULL`, ⛔ không một ô trống câm)"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **Bẫy 8 (tái sinh ở method mới)** — câu SQL phải phủ CẢ `headword` LẪN
/// `headword_simp`. `id = 6` của `zzz.db` là `國` / giản thể `国`.
#[test]
fn han_viet_covers_both_the_traditional_and_the_simplified_headword() {
    let dir = temp_dir("hanviet-simp");
    build_all_layers(&dir);
    set_han_viet(&dir, "zzz.db", 6, "quốc");

    let layers = DictLayers::open(&dir);
    let base = layers.layer("base").expect("lớp nền");

    let traditional = base.han_viet(&["國"]).expect("tra phồn thể");
    assert_eq!(
        traditional,
        vec![HanVietHit {
            character: "國".to_owned(),
            reading: "quốc".to_owned(),
            source_code: "fx-core-b".to_owned(),
        }]
    );

    let simplified = base.han_viet(&["国"]).expect("tra giản thể");
    assert_eq!(
        simplified,
        vec![HanVietHit {
            character: "国".to_owned(),
            reading: "quốc".to_owned(),
            source_code: "fx-core-b".to_owned(),
        }],
        "bỏ vế `headword_simp` làm ca này trả rỗng — đúng Bẫy 8 của Story 1.9, tái sinh"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **Quyết định #3, tiền đề của nó** — `reading` đi ra **CHƯA TÁCH**, dù tệp dùng dấu
/// `|` (khuôn Thiều Chửu) hay khoảng trắng (khuôn Unihan). Tách nhiều âm là việc của tầng
/// gom (Task 3), method này ⛔ không được tự ý cắt chuỗi.
#[test]
fn han_viet_leaves_multi_reading_strings_unsplit() {
    let dir = temp_dir("hanviet-multi");
    build_all_layers(&dir);
    // `id = 1` của `mmm.db` (hv-fixture) và `id = 2` của `aaa.db` (vp-fixture) đều là `山`.
    set_han_viet(&dir, "mmm.db", 1, "đinh|chênh");
    set_han_viet(&dir, "aaa.db", 2, "tợ tử");

    let layers = DictLayers::open(&dir);

    let hv = layers.layer("hv-fixture").expect("lớp Hán Việt");
    assert_eq!(
        hv.han_viet(&["山"]).expect("tra lớp hv-fixture"),
        vec![HanVietHit {
            character: "山".to_owned(),
            reading: "đinh|chênh".to_owned(),
            source_code: "fx-hv".to_owned(),
        }]
    );

    let vp = layers.layer("vp-fixture").expect("lớp VietPhrase");
    assert_eq!(
        vp.han_viet(&["山"]).expect("tra lớp vp-fixture"),
        vec![HanVietHit {
            character: "山".to_owned(),
            reading: "tợ tử".to_owned(),
            source_code: "fx-vp".to_owned(),
        }]
    );

    layers.close();
    cleanup(&dir);
}

/// Tập ký tự rỗng ⇒ danh sách rỗng, ⛔ **không** một lượt chạm database nào — cùng luật
/// [`an_entry_without_senses_is_an_empty_list_not_an_error`].
#[test]
fn an_empty_char_batch_touches_no_database_row() {
    let dir = temp_dir("hanviet-empty");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);
    let base = layers.layer("base").expect("lớp nền");

    let empty: &[&str] = &[];
    assert!(base.han_viet(empty).expect("tập rỗng").is_empty());

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC13-tương-đương của method mới, đo được ở tầng hành vi.**
///
/// Cùng khuôn [`reading_senses_across_many_batches_never_duplicates_or_drops_a_row`]:
/// một tập ký tự trải trên NHIỀU LÔ *(200 ký tự ⇒ 4 lô ở cỡ [`HAN_VIET_BATCH`] = 64)*
/// phải cho **đúng cùng** kết quả (không tính thứ tự) với một tập chỉ mang hai ký tự
/// thật — chỉ khác là 198 ký tự còn lại là **giả** (Khu vực dùng riêng Unicode, U+E000+),
/// dựng RẺ hơn hẳn 200 đầu mục thật mà vẫn buộc lượt tra trải qua nhiều lô.
#[test]
fn han_viet_across_many_batches_never_duplicates_or_drops_a_row() {
    let dir = temp_dir("hanviet-batches");
    build_all_layers(&dir);
    set_han_viet(&dir, "zzz.db", 1, "sơn");
    set_han_viet(&dir, "zzz.db", 6, "quốc");

    let layers = DictLayers::open(&dir);
    let base = layers.layer("base").expect("lớp nền");

    let mut straight = base.han_viet(&["山", "國", "国"]).expect("ba ký tự thật");
    straight.sort_by(|a, b| (a.character.as_str(), a.source_code.as_str())
        .cmp(&(b.character.as_str(), b.source_code.as_str())));

    // 200 ký tự giả (Khu vực dùng riêng, ⛔ không đầu mục nào khớp) cộng ba ký tự thật
    // chen vào giữa — bốn lô, lô cuối đệm, và ba ký tự thật rơi vào NHIỀU lô khác nhau.
    //
    // 🔴 `国` VÀ `國` là **cùng một hàng** (`id = 6` của `zzz.db`: `headword = 國`,
    // `headword_simp = 国`) nhưng nằm ở **HAI LÔ KHÁC NHAU** — đây là điều kiện tiên quyết
    // của lỗi trùng-hàng, và bản đầu của test này ⛔ **không** dựng nổi nó: cả `山` lẫn `國`
    // đều chỉ khớp qua MỘT trường, nên ⛔ không hàng nào có thể trả lời ở hai lô.
    //
    // Với phép lọc theo **tập đầy đủ** (bản đầu của `read_han_viet`), lô chứa `国` và lô
    // chứa `國` mỗi lô đẩy **cả hai** hit ⇒ **5 hit thay vì 3**, và test này ĐỎ. Đó chính là
    // đối chứng âm mà §Testing standards đòi cho mỗi mệnh đề mới.
    let mut many: Vec<String> = (0u32..200)
        .map(|i| char::from_u32(0xE000 + i).expect("PUA hợp lệ").to_string())
        .collect();
    many.insert(10, "山".to_owned());
    many.insert(20, "国".to_owned());
    many.insert(150, "國".to_owned());
    assert!(
        many.len() > HAN_VIET_BATCH * 3,
        "phải trải qua ÍT NHẤT bốn lô: {}",
        many.len()
    );

    let refs: Vec<&str> = many.iter().map(String::as_str).collect();
    let mut batched = base.han_viet(&refs).expect("nhiều lô");
    batched.sort_by(|a, b| (a.character.as_str(), a.source_code.as_str())
        .cmp(&(b.character.as_str(), b.source_code.as_str())));

    assert_eq!(
        batched, straight,
        "chia lô là chi tiết CÀI ĐẶT — nó ⛔ KHÔNG được đổi kết quả. Trùng hàng ở đây là \
         một phép đệm sai; thiếu hàng là một lô bị bỏ; một hàng LẠ là ký tự giả (PUA) bị \
         khớp nhầm."
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.16, Task 3 — tầng gom `lookup_han_viet` (AC5, Quyết định #1 & #3)
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **Quyết định #1, mệnh đề 1** — lớp GỠ RỜI đọc TRƯỚC lớp NỀN. `id = 1` của cả
/// `zzz.db` (nền, nguồn `fx-core-a`) LẪN `mmm.db` (gỡ rời `hv-fixture`, nguồn `fx-hv`)
/// đều là `山` — hai lớp BẤT ĐỒNG về âm, và lớp gỡ rời phải thắng.
#[test]
fn detachable_layers_outrank_the_base_layer() {
    let dir = temp_dir("hanviet-priority");
    build_all_layers(&dir);
    set_han_viet(&dir, "zzz.db", 1, "sơn-tu-lop-nen");
    set_han_viet(&dir, "mmm.db", 1, "sơn-tu-lop-go-roi");

    let layers = DictLayers::open(&dir);
    let result = lookup_han_viet(&layers, &["山"]);

    assert_eq!(result.characters.len(), 1);
    let reading = result.characters[0]
        .reading
        .as_ref()
        .expect("山 phải có âm — cả hai lớp đều có");
    assert_eq!(
        reading.primary, "sơn-tu-lop-go-roi",
        "lớp gỡ rời (hv-fixture) phải thắng lớp nền (base) — Quyết định #1"
    );
    assert_eq!(reading.source_code, "fx-hv");

    layers.close();
    cleanup(&dir);
}

/// 🔴 **Quyết định #1, mệnh đề 3** — mỗi ký tự mang `source_code`, và tầng gom liệt kê
/// TOÀN BỘ nguồn đã đóng góp cho lượt hiện tại (một dòng, ⛔ không một nhãn mỗi ký tự).
#[test]
fn each_character_carries_its_source_and_the_lookup_lists_every_source_used() {
    let dir = temp_dir("hanviet-sources");
    build_all_layers(&dir);
    // `山` thắng ở hv-fixture (gỡ rời); `國`/`国` chỉ có ở lớp nền.
    set_han_viet(&dir, "mmm.db", 1, "sơn");
    set_han_viet(&dir, "zzz.db", 6, "quốc");

    let layers = DictLayers::open(&dir);
    let result = lookup_han_viet(&layers, &["山", "國"]);

    assert_eq!(
        result.characters[0].reading.as_ref().unwrap().source_code,
        "fx-hv"
    );
    assert_eq!(
        result.characters[1].reading.as_ref().unwrap().source_code,
        "fx-core-b"
    );
    assert_eq!(
        result.sources_used,
        vec!["fx-core-b".to_owned(), "fx-hv".to_owned()],
        "danh sách nguồn đã dùng phải DEDUPED và SẮP THEO code"
    );
    assert!(result.layers_loaded);

    layers.close();
    cleanup(&dir);
}

/// 🔴 **Quyết định #3(a)** — tách nhiều âm bằng MỘT luật trên CẢ HAI hình dạng thật.
#[test]
fn multiple_readings_split_on_both_the_pipe_and_whitespace_conventions() {
    let dir = temp_dir("hanviet-split");
    build_all_layers(&dir);
    set_han_viet(&dir, "mmm.db", 1, "đinh|chênh"); // khuôn Thiều Chửu
    set_han_viet(&dir, "aaa.db", 2, "tợ tử"); // khuôn Unihan/en-wiktionary-vi

    let layers = DictLayers::open(&dir);

    let hv_only = lookup_han_viet(&layers, &["山"]);
    // hv-fixture thắng ưu tiên (gỡ rời trước gỡ rời khác theo thứ tự layers() ổn định:
    // hv-fixture đứng trước vp-fixture).
    let reading = hv_only.characters[0].reading.as_ref().unwrap();
    assert_eq!(reading.primary, "đinh", "âm ĐẦU TIÊN sau khi tách");
    assert_eq!(reading.all, vec!["đinh".to_owned(), "chênh".to_owned()]);

    // 🔴 **Đóng lớp TRƯỚC khi xoá tệp** — Luật 2 ở đầu chính tệp test này: Windows từ chối
    // xoá một tệp đang mở (NFR14). Bản đầu xoá `mmm.db` khi `layers` vẫn giữ kết nối, nên
    // nó xanh trên macOS và ĐỎ trên máy người khác — đúng lớp lỗi chỉ lộ ra ở CI nền tảng
    // kia. Test anh em `removing_every_detachable_layer_…` làm đúng từ đầu.
    layers.close();

    // Xoá lớp thắng để buộc kết quả rơi xuống vp-fixture, xem đúng luật tách khoảng trắng.
    fs::remove_file(dir.join("mmm.db")).expect("xoa mmm.db");
    let layers2 = DictLayers::open(&dir);
    let vp_only = lookup_han_viet(&layers2, &["山"]);
    let reading2 = vp_only.characters[0].reading.as_ref().unwrap();
    assert_eq!(reading2.primary, "tợ");
    assert_eq!(reading2.all, vec!["tợ".to_owned(), "tử".to_owned()]);
    assert_eq!(reading2.source_code, "fx-vp");

    layers2.close();
    cleanup(&dir);
}

/// 🔴 **Quy ước THỨ BA: dấu phẩy** — Trần Văn Chánh và en-wiktionary-vi, cả hai hình dạng
/// thật *(bắt ở lượt code review 2026-08-06)*.
///
/// Bản đầu của [`split_readings`] chỉ cắt trên `|` và khoảng trắng, và mục bàn giao của
/// `1-10c` đã cảnh báo đích danh ba quy ước. Đo trên tệp `.db` thật:
/// `dict-core.db` **284/1.145 = 24,8 %** hàng dùng `,` *(chính lớp NỀN mà FR36 rơi về)*;
/// `dict-tran-van-chanh.db` **2.326** hàng *(lớp gỡ rời ưu tiên CAO NHẤT)*.
///
/// Hai hình dạng ĐỀU tồn tại thật và hỏng theo hai kiểu khác nhau — test này giữ cả hai:
/// - `"tây,tê"` *(⛔ không khoảng trắng)* → bản đầu trả **một** phần tử `"tây,tê"`.
/// - `"chiêm, thiềm"` *(phẩy + khoảng trắng)* → bản đầu trả `primary = "chiêm,"`, **dấu
///   phẩy đuôi lên màn hình** (`str::trim` chỉ cắt khoảng trắng).
#[test]
fn multiple_readings_split_on_the_comma_convention_too() {
    let dir = temp_dir("hanviet-split-comma");
    build_all_layers(&dir);
    set_han_viet(&dir, "mmm.db", 1, "tây,tê"); // khuôn en-wiktionary-vi: ⛔ không khoảng trắng
    set_han_viet(&dir, "zzz.db", 6, "chiêm, thiềm"); // khuôn Trần Văn Chánh: phẩy + khoảng trắng

    let layers = DictLayers::open(&dir);

    let no_space = lookup_han_viet(&layers, &["山"]);
    let r1 = no_space.characters[0].reading.as_ref().expect("山 phải có âm");
    assert_eq!(
        r1.primary, "tây",
        "`tây,tê` phải tách thành HAI âm — một luật áp cho MỌI tệp (Quyết định #3a)"
    );
    assert_eq!(r1.all, vec!["tây".to_owned(), "tê".to_owned()]);

    let with_space = lookup_han_viet(&layers, &["國"]);
    let r2 = with_space.characters[0].reading.as_ref().expect("國 phải có âm");
    assert_eq!(
        r2.primary, "chiêm",
        "⛔ KHÔNG được mang dấu phẩy đuôi — `str::trim` chỉ cắt khoảng trắng"
    );
    assert_eq!(r2.all, vec!["chiêm".to_owned(), "thiềm".to_owned()]);

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC4 — ba trạng thái, ⛔ không một.** (1) ký tự có âm; (2) ký tự ⛔ không có âm ở
/// bất kỳ lớp nào (nhưng CÓ lớp đang gắn); (3) ⛔ không lớp nào đang gắn.
#[test]
fn three_distinct_states_never_collapse_into_one() {
    let dir = temp_dir("hanviet-states");
    build_all_layers(&dir);
    set_han_viet(&dir, "zzz.db", 1, "sơn");

    // (1) và (2) cùng lúc: `山` có âm, `高` (từ `高山`, KHÔNG match vì headword 2 ký tự)
    // ⛔ không có âm dù CÓ lớp đang gắn.
    let layers = DictLayers::open(&dir);
    let result = lookup_han_viet(&layers, &["山", "高"]);
    assert!(result.layers_loaded, "ca (1)/(2): CÓ lớp đang gắn");
    assert!(result.characters[0].reading.is_some(), "山 CÓ âm");
    assert!(
        result.characters[1].reading.is_none(),
        "高 đã tra mà KHÔNG có âm — KHÁC ca 0 lớp gắn"
    );
    layers.close();

    // (3): tập lớp hoàn toàn rỗng — trạng thái BÌNH THƯỜNG có tên (AD-25).
    let empty_dir = temp_dir("hanviet-states-empty");
    let empty_layers = DictLayers::open(&empty_dir);
    let empty_result = lookup_han_viet(&empty_layers, &["山"]);
    assert!(!empty_result.layers_loaded, "ca (3): KHÔNG lớp nào đang gắn");
    assert!(
        empty_result.characters[0].reading.is_none(),
        "0 lớp ⇒ ⛔ không âm nào, nhưng lý do PHẢI phân biệt được qua `layers_loaded`"
    );

    cleanup(&dir);
    cleanup(&empty_dir);
}

/// 🔴 **AC5 / FR36 — nghiệm thu ở mức DEGRADATION, bằng test THẬT xoá tệp.**
///
/// Xoá CẢ HAI lớp gỡ rời ⇒ tab vẫn trả âm từ lớp NỀN, ⛔ không một đường nào hỏng. Phủ
/// giảm là kết quả ĐÚNG, ⛔ không phải một lỗi cần sửa.
#[test]
fn removing_every_detachable_layer_still_serves_readings_from_the_base_layer() {
    let dir = temp_dir("hanviet-fr36");
    build_all_layers(&dir);
    set_han_viet(&dir, "zzz.db", 1, "sơn"); // lớp NỀN mang âm của 山
    set_han_viet(&dir, "mmm.db", 1, "am-tu-hv"); // lớp gỡ rời BAN ĐẦU thắng ưu tiên

    let layers_before = DictLayers::open(&dir);
    let before = lookup_han_viet(&layers_before, &["山"]);
    assert_eq!(
        before.characters[0].reading.as_ref().unwrap().source_code,
        "fx-hv",
        "trước khi xoá: lớp gỡ rời thắng"
    );
    layers_before.close();

    // FR36: gỡ = xoá tệp — xoá CẢ HAI lớp gỡ rời khỏi đĩa.
    fs::remove_file(dir.join("mmm.db")).expect("xoa hv-fixture");
    fs::remove_file(dir.join("aaa.db")).expect("xoa vp-fixture");

    let layers_after = DictLayers::open(&dir);
    assert_eq!(
        layer_ids(&layers_after),
        vec!["base"],
        "chỉ còn lớp nền sau khi xoá cả hai lớp gỡ rời"
    );

    let after = lookup_han_viet(&layers_after, &["山"]);
    assert!(after.layers_loaded, "lớp nền vẫn nạp được ⇒ ⛔ không một đường nào hỏng");
    let reading = after.characters[0]
        .reading
        .as_ref()
        .expect("mất lớp gỡ rời KHÔNG được làm mất luôn âm của lớp nền");
    assert_eq!(reading.source_code, "fx-core-a", "rơi về ĐÚNG lớp nền");
    assert_eq!(reading.primary, "sơn");

    layers_after.close();
    cleanup(&dir);
}

/// Đầu ra giữ ĐÚNG vị trí và ĐÚNG số lượng của `chars` truyền vào — kể cả ký tự lặp lại
/// nhiều lần trong văn bản (Panel Source zip trực tiếp với văn bản gốc theo vị trí).
#[test]
fn the_output_keeps_one_slot_per_input_character_including_repeats() {
    let dir = temp_dir("hanviet-positions");
    build_all_layers(&dir);
    set_han_viet(&dir, "zzz.db", 1, "sơn");

    let layers = DictLayers::open(&dir);
    let result = lookup_han_viet(&layers, &["山", "高", "山", "山"]);

    assert_eq!(result.characters.len(), 4, "4 ký tự vào ⇒ 4 phần tử ra");
    assert_eq!(result.characters[0].character, "山");
    assert_eq!(result.characters[1].character, "高");
    assert_eq!(result.characters[2].character, "山");
    assert_eq!(result.characters[3].character, "山");
    assert!(result.characters[0].reading.is_some());
    assert!(result.characters[1].reading.is_none());
    assert!(result.characters[2].reading.is_some());
    assert!(result.characters[3].reading.is_some());

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.16, Task 6/7 chuẩn bị — bề mặt IPC `commands::dict::read_han_viet`
// ═════════════════════════════════════════════════════════════════════════════════

/// `layers = None` phải đối xử GIỐNG HỆT một tập lớp rỗng — ⛔ không một nhánh lỗi riêng.
#[test]
fn read_han_viet_command_treats_a_missing_state_like_an_empty_layer_set() {
    let result = auratranslate_lib::commands::dict::read_han_viet(None, &["山".to_owned()]);
    assert!(!result.layers_loaded);
    assert_eq!(result.characters.len(), 1);
    assert!(result.characters[0].reading.is_none());
    assert!(result.sources_used.is_empty());
}

/// Vỏ IPC gọi ĐÚNG XUỐNG tầng gom — cùng kết quả với gọi thẳng `lookup_han_viet`.
#[test]
fn read_han_viet_command_matches_the_grouping_layer_directly() {
    let dir = temp_dir("hanviet-command");
    build_all_layers(&dir);
    set_han_viet(&dir, "zzz.db", 1, "sơn");

    let layers = DictLayers::open(&dir);
    let via_command =
        auratranslate_lib::commands::dict::read_han_viet(Some(&layers), &["山".to_owned(), "高".to_owned()]);
    let direct = lookup_han_viet(&layers, &["山", "高"]);

    assert_eq!(via_command, direct);

    layers.close();
    cleanup(&dir);
}
