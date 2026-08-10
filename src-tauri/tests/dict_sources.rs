//! Hành vi tầng **GOM** — Story 1.13, AC3 tới AC13.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ VÌ SAO TỆP NÀY ĐƯỢC PHÉP `use rusqlite`
//! ─────────────────────────────────────────────────────────────────────────────
//! Cùng lý do đã ghi ở `dict_lookup.rs:4-11`: `store_boundary.rs` cưỡng chế ranh giới
//! trên `src-tauri/src/**`, `tests/**` nằm ngoài **có tên và có lý do**; và không tệp
//! `.db` nào nằm trong git (`.gitignore: *.db` — AD-25), nên fixture phải dựng trong test.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO FIXTURE Ở ĐÂY LÀ **BA TỆP**, KHÔNG PHẢI MỘT
//! ─────────────────────────────────────────────────────────────────────────────
//! `dict_lookup.rs` dựng **một** tệp, vì Story 1.11 chạy trên một tệp một lượt. Ba cái
//! bẫy đắt nhất của story này **không quan sát được** trên một tệp:
//!
//! 1. **`source_id` trùng giữa các tệp.** Mỗi tệp `.db` mang bảng `dict_source` RIÊNG, nên
//!    `id = 1` tồn tại ở **cả ba** và trỏ ba nguồn khác nhau. Cả ba tệp fixture dưới đây
//!    dùng `id = 1` **có chủ ý** — gom theo `id` dán nhãn sai nguồn, và không một ca
//!    một-tệp nào đỏ.
//! 2. **Thứ tự lớp.** Tên tệp cố tình xếp `aaa` · `mmm` · `zzz` trong khi thứ tự đúng là
//!    `base` · `hv-fixture` · `vp-fixture` — tức **ngược** thứ tự chữ cái của tên tệp. Một
//!    cài đặt tin vào thứ tự `read_dir` sẽ đỏ ở đây thay vì đỏ trên **một** nhánh CI.
//! 3. **FR36.** *"Gỡ một lớp = xoá một file"* không nghiệm thu được nếu chỉ có một file.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT — thừa kế nguyên từ `dict_lookup.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + bộ đếm nguyên tử). Không `tempfile`.
//! 2. **Drop `ReadOnlyDb` TRƯỚC khi xoá tệp** — Windows từ chối xoá tệp đang mở (NFR14).
//!    🔴 Ở tệp này luật đó không còn là dọn dẹp: nó là **điều kiện để AC12 chạy được**.
//! 3. **Không ngưỡng thời gian trong CI** — phép đo NFR1 là
//!    [`bench_the_grouped_path_on_the_real_dictionaries`]: `#[ignore]` + biến môi trường.
//! 4. **Đường dẫn tương đối lấy qua `env!("CARGO_MANIFEST_DIR")`.**

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::core::dict::{
    DictLayers, GroupedLookup, HAN_VIET_BATCH, HanVietHit, HanVietLookup, LookupMode, QueryBranch,
    MINIMUM_SCHEMA_VERSION, QueryRoute, SENSE_BATCH, SUPPORTED_SCHEMA_VERSION, SenseRecord,
    SkipReason, is_han,
};
use auratranslate_lib::ports::DictionarySource;

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 1.19 — HAI VỎ *"KHÔNG TẮT NGUỒN NÀO"*, CỐ Ý MANG ĐÚNG TÊN HÀM THẬT
// ═════════════════════════════════════════════════════════════════════════════════
//
// `lookup_grouped`/`lookup_han_viet` nay nhận thêm tập `code` **BỊ TẮT** (§Quyết định #2a
// và #3a). Bốn mươi ca đã có của tệp này hỏi những câu **không liên quan** tới bộ lọc, và
// rải `&BTreeSet::new()` vào cuối từng lời gọi chỉ làm chúng khó đọc hơn mà không canh
// thêm gì.
//
// ⚠️ Hai vỏ này **che** tên đã import, có chủ ý và đúng một hướng: một ca **về** bộ lọc gọi
// thẳng `auratranslate_lib::core::dict::lookup_grouped` với tập thật, và sự khác biệt đó
// đọc được ngay tại chỗ gọi. Đừng thêm tham số vào hai vỏ này — chúng tồn tại để nói
// *"ca này không tắt nguồn nào"*, không phải để làm một API thứ hai.

/// Tra cứu **không tắt nguồn nào** — hình dạng của mọi ca có trước Story 1.19.
fn lookup_grouped(
    layers: &DictLayers,
    query: &str,
    mode: LookupMode,
    limit: usize,
) -> GroupedLookup {
    auratranslate_lib::core::dict::lookup_grouped(layers, query, mode, limit, &BTreeSet::new())
}

/// Gom âm Hán Việt **không tắt nguồn nào**.
fn lookup_han_viet(layers: &DictLayers, chars: &[&str]) -> HanVietLookup {
    auratranslate_lib::core::dict::lookup_han_viet(layers, chars, &BTreeSet::new())
}

/// Đường sản phẩm Panel Lookup (`commands::dict::lookup`), **không tắt nguồn nào**.
fn command_lookup(
    layers: Option<&DictLayers>,
    query: &str,
) -> auratranslate_lib::commands::dict::LookupResponse {
    auratranslate_lib::commands::dict::lookup(layers, query, &BTreeSet::new())
}

/// Đường sản phẩm tab Hán Việt (`commands::dict::read_han_viet`), **không tắt nguồn nào**.
fn command_read_han_viet(layers: Option<&DictLayers>, chars: &[String]) -> HanVietLookup {
    auratranslate_lib::commands::dict::read_han_viet(layers, chars, &BTreeSet::new())
}

/// 🔴 Trần pha một (Quyết định #4, Story 1.17) — mọi fixture của tệp này có dưới hai
/// mươi hàng một nguồn, nên một trần lớn giữ nguyên hành vi trước story cho các ca không
/// không nhắm tới `truncated`. Các ca AC12 (§Quyết định #4 hệ quả ②) dùng một trần nhỏ
/// **tường minh**, không hằng này.
const UNLIMITED: usize = 10_000;

// ═════════════════════════════════════════════════════════════════════════════════
// DDL — CHÉP NGUYÊN VĂN từ `tools/dict-build/src/schema.rs`
// ═════════════════════════════════════════════════════════════════════════════════
//
// Đừng "dọn dẹp" khoảng trắng ở đây. Cổng parity so **chuỗi con nguyên văn**; một lượt
// canh lề tử tế làm nó đỏ, và người sửa tiếp theo sẽ sửa bằng cách nới cổng.
//
// 🔴 `DICT_CITATION_DDL` ở tệp này **không** còn là một khối chép cho đủ: story này là
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
    /// 🔴 Tên tệp **cố tình** không nói gì về lớp bên trong — AD-44 ① vá A2: danh tính
    /// lớp đọc từ `dict_meta('layer')` của chính tệp, không từ tên tệp.
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
    // 🔴 Một đầu mục **không có nghĩa nào** — trạng thái **hợp lệ**, không phải một
    // lỗi: `dict_entry` mang `reading` và `han_viet` (âm đọc), và một nguồn có thể ghi âm
    // đọc mà không ghi nghĩa. Pha hai phải trả **danh sách rỗng**, không trả lỗi.
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
// 🔴 Đây là **FIXTURE**, không phải dữ liệu HVTĐTD thật — `dict-hvtdtd.db` không tồn
// tại vì chưa có nguồn thô (`src-tauri/resources/dict/README.md:13`, `prd.md:856` [A2]).
// Nó nghiệm thu đúng thứ nghiệm thu được hôm nay: *đường mã có phân biệt được nhãn tiếng
// Việt với nhãn ngoại ngữ không*.
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
/// ⚠️ Fixture **không** đặt `journal_mode`; mặc định `delete` — giống hệt ba tệp thật.
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

    // 🔴 `entry_fts` là external-content ⇒ nó **không** tự đầy. Thiếu dòng này, nhánh 3
    // trả rỗng **cho từng tệp** và mọi ca của nó "xanh" theo đúng cách sai nhất.
    conn.execute_batch("INSERT INTO entry_fts(entry_fts) VALUES('rebuild');")
        .unwrap_or_else(|e| panic!("rebuild entry_fts: {e}"));

    // 🔴 **Story 1.19 — `dict_source.lang` ĐO từ `dict_entry`, y hệt đường sản phẩm.**
    //
    // Câu lệnh này là bản sao **nguyên văn** của `tools/dict-build/src/insert.rs::
    // backfill_source_langs`, và bản sao đó có chủ ý: hai workspace tách rời (AC4 của Story
    // 1.9) nên không có import chéo nào. Điều fixture phải giữ là *"lang đến từ dữ liệu"* —
    // gán tay một chuỗi ở `LayerSeed` sẽ để lọt đúng lỗi mà cột này sinh ra để chặn (một
    // nguồn khai `en` mà 0 đầu mục nào `en`).
    conn.execute_batch(
        "UPDATE dict_source SET lang = IFNULL(
           (SELECT GROUP_CONCAT(lang, ',') FROM
              (SELECT DISTINCT lang FROM dict_entry
                WHERE source_id = dict_source.id ORDER BY lang)),
           '');",
    )
    .unwrap_or_else(|e| panic!("đo dict_source.lang: {e}"));

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

/// 🔴 **AC3** — thứ tự là `base` trước rồi mã lớp tăng dần, và nó **không** phải thứ tự
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
        "thứ tự lớp phải là `base` trước rồi mã lớp TĂNG DẦN, không phải thứ tự \
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

/// 🔴 **AC3 vế cuối** — thư mục không tồn tại, hoặc rỗng ⇒ **tập lớp RỖNG**, không
/// lỗi, không panic.
///
/// Đây **không** phải một ca phòng xa: `src-tauri/resources/dict/` hôm nay **rỗng**
/// (không tệp `.db` nào trong git — AD-25) và `bundle.resources` chưa mang thư mục đó
/// (Story 10.1). *"Không có lớp nào"* là một trạng thái **bình thường có tên**, và nó
/// là chính hình dạng FR36 đòi hỏi.
#[test]
fn a_missing_or_empty_directory_is_an_empty_layer_set_not_an_error() {
    let dir = temp_dir("empty");

    let empty = DictLayers::open(&dir);
    assert!(empty.layers().is_empty(), "thư mục rỗng ⇒ tập lớp rỗng");
    assert!(
        empty.skipped().is_empty(),
        "thư mục rỗng ⇒ không lớp nào bị bỏ qua"
    );

    let missing = DictLayers::open(&dir.join("khong-ton-tai"));
    assert!(
        missing.layers().is_empty(),
        "thư mục không tồn tại ⇒ tập lớp rỗng, KHÔNG lỗi"
    );

    // Và một lượt tra trên tập rỗng vẫn trả **đường đã đi**, không panic.
    let result = lookup_grouped(&empty, "山", LookupMode::Exact, UNLIMITED);
    assert_eq!(result.route, QueryRoute::Zh);
    assert_eq!(result.branch, QueryBranch::ExactBtree);
    assert!(result.groups.is_empty());
    assert!(
        !result.layers_loaded,
        "🔴 AC6 ca thứ năm (Story 1.17) — 0 lớp gắn PHẢI phân biệt được với 'đã tra mà \
         không khớp' thuần tuý qua trường layers_loaded, không suy từ groups rỗng"
    );

    cleanup(&dir);
}

/// 🔴 **AC6 ca thứ năm, nửa còn lại** — CÓ lớp gắn nhưng không khớp gì ⇒ `layers_loaded` PHẢI
/// `true`, phân biệt với ca thư mục rỗng ở trên (cả hai đều cho `groups` rỗng).
#[test]
fn layers_loaded_is_true_even_when_nothing_matches() {
    let dir = temp_dir("layers-loaded-no-match");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = lookup_grouped(&layers, "tu-khong-ton-tai-zzz", LookupMode::Exact, UNLIMITED);
    assert!(result.groups.is_empty());
    assert!(
        result.layers_loaded,
        "ba lớp ĐÃ nạp và ĐÃ tra — không khớp là một kết quả BÌNH THƯỜNG, không phải trông \
         giống ca '0 lớp'"
    );

    layers.close();
    cleanup(&dir);
}

/// Tệp không phải `.db` ⇒ **không** được thử mở, và không vào danh sách bỏ qua.
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
        "tệp không mang đuôi `.db` KHÔNG phải một lớp bị bỏ qua — nó không phải \
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
/// đường dẫn + lý do, không phải một dòng `eprintln!`.
#[test]
fn a_broken_layer_is_skipped_by_name_and_the_rest_still_answer() {
    let dir = temp_dir("broken");
    build_all_layers(&dir);

    // (a) không phải một tệp SQLite.
    fs::write(
        dir.join("garbage.db"),
        b"day khong phai mot database SQLite",
    )
    .unwrap_or_else(|e| panic!("ghi garbage: {e}"));

    // (b) Một tệp SQLite hợp lệ nhưng không có `dict_meta`.
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

    // (d2) 🔴 Tệp **CŨ HƠN** thứ đường đọc còn đọc nổi — Ice chốt ở code review 2026-08-10.
    //      Ca THẬT, không giả định: bờ đọc gõ `dict_source.lang` *(cột của lược đồ v3)*, nên
    //      một tệp v2 lọt cửa sẽ gãy bằng `no such column` ở GIỮA đường và bị
    //      `list_source_attributions` **nuốt** — dải chip biến mất không dấu vết trong khi
    //      tra cứu vẫn chạy. Ice gặp đúng ca này ở lần chạy thử đầu tiên.
    build_layer(
        &dir,
        &LayerSeed {
            file: "too-old.db",
            layer: "too-old-fixture",
            sources: &[(1, "fx-too-old", "Fixture Too Old")],
            entries: HV_ENTRIES,
        },
        &(MINIMUM_SCHEMA_VERSION - 1).to_string(),
        MINIMUM_SCHEMA_VERSION - 1,
    );

    // (d) Hai chỗ ghi phiên bản **NÓI KHÁC NHAU** ⇒ tệp không do `tools/dict-build`
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
        "bốn tệp hỏng KHÔNG được kéo theo ba lớp lành"
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

    assert_eq!(skipped.len(), 5, "chờ đúng năm lớp bị bỏ qua: {skipped:?}");

    let reason_of = |file: &str| -> SkipReason {
        skipped
            .iter()
            .find(|(name, _)| name == file)
            .map(|(_, reason)| reason.clone())
            .unwrap_or_else(|| panic!("không thấy {file} trong danh sách bỏ qua: {skipped:?}"))
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
        reason_of("too-old.db"),
        SkipReason::SchemaTooOld {
            file_version: MINIMUM_SCHEMA_VERSION - 1,
            minimum: MINIMUM_SCHEMA_VERSION,
        },
        "một tệp CŨ HƠN thứ đọc nổi phải bị từ chối Ở CỬA với lý do RIÊNG — không lọt vào \
         rồi gãy im lặng ở câu `SELECT` đầu tiên gõ tên một cột nó không có"
    );
    assert_eq!(
        reason_of("disagreeing.db"),
        SkipReason::SchemaVersionDisagrees {
            user_version: SUPPORTED_SCHEMA_VERSION,
            meta_version: "99".to_owned(),
        },
        "hai chỗ ghi phiên bản nói khác nhau phải có lý do RIÊNG"
    );

    // 🔴 Vế *"các lớp còn lại vẫn tra được BÌNH THƯỜNG"* — không chỉ *"vẫn nạp được"*.
    let result = lookup_grouped(&layers, "山", LookupMode::Exact, UNLIMITED);
    assert_eq!(
        groups_of(&result)
            .into_iter()
            .map(|(code, _)| code)
            .collect::<Vec<_>>(),
        vec!["fx-core-a", "fx-hv", "fx-vp"],
        "bốn tệp hỏng KHÔNG được làm hỏng lượt tra của ba lớp lành"
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Task 1 (Story 1.17) — hình dạng bản ghi TRÊN DÂY: `skipped` rút gọn còn mã máy
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **Ca test hành vi bắt buộc của Task 1** — `SkipReason::detail` (lỗi thô SQLite)
/// **KHÔNG BAO GIỜ** được xuất hiện trong JSON đi qua dây, và đường dẫn tệp cũng vậy.
/// Đây là ca làm cổng AD-21 ĐỎ nếu ai đó derive `Serialize` thẳng lên `SkipReason`/
/// `SkippedLayer` thay vì đi qua [`SkipReason::wire_code`] — kiểu đó sẽ **không compile
/// được** với `#[serde(serialize_with = "serialize_skipped_as_wire_codes")]` trên
/// `GroupedLookup::skipped` nếu ai đó đổi kiểu trường, và nếu ai đó bỏ luôn thuộc tính đó
/// để derive trực tiếp, assertion `!json.contains(&raw_detail)` dưới đây sẽ đỏ.
///
/// 🔴 Giá trị đem serialize đến từ **đường sản phẩm thật** — `lookup_grouped` trên một tập
/// lớp có một tệp hỏng thật (`garbage.db`, cùng khuôn ca `a_broken_layer_…` ở trên) — không
/// không một `GroupedLookup` dựng tay, đúng nguyên tắc `ipc_contract.rs` đã đặt ra.
#[test]
fn skip_reason_detail_never_reaches_the_wire() {
    let dir = temp_dir("skip-wire");
    build_all_layers(&dir);
    fs::write(
        dir.join("garbage.db"),
        b"day khong phai mot database SQLite",
    )
    .unwrap_or_else(|e| panic!("ghi garbage: {e}"));

    let layers = DictLayers::open(&dir);
    let result = lookup_grouped(&layers, "山", LookupMode::Exact, UNLIMITED);

    assert_eq!(result.skipped.len(), 1, "chờ đúng một lớp bị bỏ qua: {:?}", result.skipped);
    let raw_detail = match &result.skipped[0].reason {
        SkipReason::MetaUnreadable { detail } => detail.clone(),
        other => panic!("kỳ vọng MetaUnreadable, được {other:?}"),
    };
    assert!(
        !raw_detail.is_empty(),
        "phải có nội dung lỗi thô THẬT để phép kiểm dưới đây có ý nghĩa"
    );

    let json = serde_json::to_string(&result).unwrap_or_else(|e| panic!("serialize: {e}"));

    assert!(
        !json.contains(&raw_detail),
        "🔴 AD-21 vỡ — lỗi thô SQLite '{raw_detail}' lộ ra JSON đi qua dây: {json}"
    );
    assert!(
        !json.contains("garbage.db"),
        "đường dẫn TỆP không được đi qua dây: {json}"
    );
    assert!(
        json.contains("\"meta_unreadable\""),
        "mã máy PHẢI có mặt để panel chẩn đoán/hiển thị được: {json}"
    );
    assert!(
        json.contains("\"branch\":\"exact_btree\""),
        "QueryBranch phải ra CHUỖI ĐỊNH DANH MÁY, không số thứ tự biến thể: {json}"
    );
    assert!(
        json.contains("\"route\":\"zh\""),
        "QueryRoute phải ra CHUỖI ĐỊNH DANH MÁY: {json}"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC6 vế cuối** — hai lớp khai **cùng một `code`** là một **lỗi dữ liệu CÓ TÊN**,
/// **không** phải một lượt gộp im lặng hai tệp vào một nhóm.
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

/// 🔴 **AC5** — `route` và `branch` xuất hiện **một lần** trong kết quả gom, không phải
/// một lần cho mỗi tệp.
#[test]
fn the_route_and_the_branch_are_one_value_of_the_whole_lookup() {
    let dir = temp_dir("route");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let exact = lookup_grouped(&layers, "山", LookupMode::Exact, UNLIMITED);
    assert_eq!(exact.route, QueryRoute::Zh);
    assert_eq!(exact.branch, QueryBranch::ExactBtree);

    let one_char = lookup_grouped(&layers, "山", LookupMode::Substring, UNLIMITED);
    assert_eq!(
        one_char.branch,
        QueryBranch::CharIdx,
        "1 ký tự Hán ⇒ nhánh 2"
    );

    let three_chars = lookup_grouped(&layers, "中國人", LookupMode::Substring, UNLIMITED);
    assert_eq!(
        three_chars.branch,
        QueryBranch::FtsTrigram,
        "3 ký tự Hán ⇒ nhánh 3 — và fixture không có đầu mục nào khớp, nên nhánh là \
         thứ DUY NHẤT quan sát được ở đây"
    );

    let english = lookup_grouped(&layers, "dictionary", LookupMode::Exact, UNLIMITED);
    assert_eq!(
        english.route,
        QueryRoute::En,
        "không ký tự Hán ⇒ đường `En`"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC5 mệnh đề cuối** — `NoBranchQueryTooShort` **sống sót qua tầng gom**.
///
/// **Không phải "không có kết quả":** hai câu đó dẫn người dùng đi hai đường khác nhau
/// (AD-44 ④), và **1.17 đọc đúng trường này** để nói *"truy vấn quá ngắn"*. Một tầng gom
/// dịch nó thành `groups: []` là làm mệnh đề đó không nghiệm thu được ở story sau.
#[test]
fn the_query_too_short_state_survives_the_grouping_layer() {
    let dir = temp_dir("tooshort");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    for query in ["", "a", "ab"] {
        let result = lookup_grouped(&layers, query, LookupMode::Substring, UNLIMITED);
        assert_eq!(
            result.branch,
            QueryBranch::NoBranchQueryTooShort,
            "truy vấn {query:?} (chuỗi con tiếng Anh < 3 ký tự) phải giữ nguyên trạng thái \
             KHÔNG HỖ TRỢ qua tầng gom"
        );
        assert!(
            result.groups.is_empty(),
            "truy vấn {query:?} ⇒ không nhóm nào"
        );
        assert!(
            result.skipped.is_empty(),
            "truy vấn {query:?}: *rỗng vì quá ngắn* KHÔNG phải *rỗng vì một lớp hỏng*"
        );
    }

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC6 — nhóm theo NGUỒN, khoá gom là `code`, và KHÔNG hợp nhất
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC6, và đây là bẫy im lặng nhất của cả story.**
///
/// Ba tệp fixture đều dùng `dict_source.id = 1`, đúng như ba tệp thật. Gom theo `id` dán
/// nhãn *"Fixture Han Viet"* cho một đầu mục thật ra của lớp nền — **FR31 vỡ, không
/// lỗi, không test hành vi nào đỏ** trừ ca này.
#[test]
fn groups_are_keyed_by_the_source_code_not_by_the_numeric_id() {
    let dir = temp_dir("groupkey");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = lookup_grouped(&layers, "山", LookupMode::Exact, UNLIMITED);

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

/// 🔴 **AC6 / FR32** — hai nguồn **bất đồng** ⇒ **cả hai nhóm có mặt**, không nhóm nào
/// bị chọn làm *"câu trả lời"*.
#[test]
fn two_sources_that_disagree_both_survive_with_their_meanings_intact() {
    let dir = temp_dir("disagree");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = lookup_grouped(&layers, "中國", LookupMode::Exact, UNLIMITED);

    assert_eq!(
        groups_of(&result),
        vec![
            ("fx-core-a".to_owned(), vec!["中國".to_owned()]),
            ("fx-vp".to_owned(), vec!["中國".to_owned()]),
        ],
        "lớp nền nói `China`, VietPhrase nói `Trung Quốc` — KHÔNG nhóm nào được biến mất"
    );

    // Và nghĩa **mâu thuẫn** phải đi hết đường ra tới bản ghi, không bị chọn một cái.
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

/// 🔴 **AC6** — một nguồn **đã tra mà không khớp gì** ⇒ **không sinh nhóm rỗng**.
///
/// Trạng thái đó phải phân biệt được với *"lớp không nạp được"*, và chỗ phân biệt là
/// danh sách `skipped`. Hai thứ đó **không** được phép trông giống nhau ở 1.17.
#[test]
fn a_source_that_matched_nothing_produces_no_empty_group() {
    let dir = temp_dir("nogroup");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    // `高山` chỉ có ở lớp nền, và chỉ ở nguồn `fx-core-b`.
    let result = lookup_grouped(&layers, "高山", LookupMode::Exact, UNLIMITED);
    assert_eq!(
        groups_of(&result),
        vec![("fx-core-b".to_owned(), vec!["高山".to_owned()])],
        "ba lớp được tra, một lớp khớp ⇒ ĐÚNG MỘT nhóm, không hai nhóm rỗng đi kèm"
    );
    assert!(
        result.skipped.is_empty(),
        "không lớp nào hỏng ⇒ danh sách bỏ qua RỖNG — đó là thứ phân biệt *đã tra mà \
         không khớp* với *chưa bao giờ được tra*"
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC12 (Story 1.17) — `LIMIT` cấp-tệp KHÔNG được xoá sạch một nguồn IM LẶNG
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 Điều kiện tiên quyết của lỗi: MỘT tệp, HAI nguồn. Fixture ba-lớp-mỗi-lớp-một-nguồn
// ở trên (`LAYERS`) KHÔNG dựng được nó — mỗi tệp ở đó chỉ mang một hoặc hai nguồn
// nhưng lớp nền (`BASE_ENTRIES`) không có đủ số hàng ≥ trần để bắt được ca cắt. Đây là
// một tệp fixture RIÊNG.

static LIMIT_ENTRIES: &[EntrySeed] = &[
    // Nguồn `fx-limit-a` — BA đầu mục cùng `headword`, `id` NHỎ.
    EntrySeed {
        id: 1,
        source_id: 1,
        lang: "zh",
        headword: "共",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 2,
        source_id: 1,
        lang: "zh",
        headword: "共",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 3,
        source_id: 1,
        lang: "zh",
        headword: "共",
        simp: None,
        senses: &[],
    },
    // Nguồn `fx-limit-b` — MỘT đầu mục, `id` LỚN HƠN mọi `id` của nguồn trên.
    EntrySeed {
        id: 100,
        source_id: 2,
        lang: "zh",
        headword: "共",
        simp: None,
        senses: &[],
    },
];

static LIMIT_LAYER: LayerSeed = LayerSeed {
    file: "limit.db",
    layer: "limit-fixture",
    sources: &[
        (1, "fx-limit-a", "Fixture Limit A"),
        (2, "fx-limit-b", "Fixture Limit B"),
    ],
    entries: LIMIT_ENTRIES,
};

/// Dựng **một** tệp fixture — nhánh 1-nguồn không áp dụng ở đây, nên không dùng `build_all_layers`.
fn build_limit_fixture(dir: &Path) -> DictLayers {
    build_layer(
        dir,
        &LIMIT_LAYER,
        &SUPPORTED_SCHEMA_VERSION.to_string(),
        SUPPORTED_SCHEMA_VERSION,
    );
    DictLayers::open(dir)
}

/// 🔴 **AC12** — ca test BẮT BUỘC của story: nhánh `ExactBtree` (SQL `LIMIT ?N` thật).
///
/// Trần = 3 = đúng số đầu mục của `fx-limit-a` ⇒ `fx-limit-b` (id 100, lớn hơn mọi id
/// của `fx-limit-a`) **hoàn toàn văng khỏi** `groups` nếu `LIMIT` cấp-tệp áp thẳng.
///
/// **Ca này phải ĐỎ trên một cài đặt `LIMIT` cấp-tệp NGÂY THƠ** (một cài đặt cắt bớt
/// mà không có trường `truncated`/`truncated_layers` nào báo lại) — nó không compile
/// được nếu thiếu hai trường đó, và nếu ai đó âm thầm đặt `truncated_layers: vec![]` thì
/// assertion dưới đây đỏ.
#[test]
fn a_file_level_limit_flags_truncation_instead_of_silently_dropping_a_source() {
    let dir = temp_dir("limit-exact");
    let layers = build_limit_fixture(&dir);

    let result = lookup_grouped(&layers, "共", LookupMode::Exact, 3);
    assert_eq!(result.branch, QueryBranch::ExactBtree);

    assert_eq!(
        groups_of(&result),
        vec![("fx-limit-a".to_owned(), vec!["共".to_owned(), "共".to_owned(), "共".to_owned()])],
        "trần 3 khớp ĐÚNG số đầu mục của fx-limit-a ⇒ fx-limit-b (id lớn hơn) văng khỏi trang"
    );
    assert_eq!(
        result.truncated_layers,
        vec!["limit-fixture".to_owned()],
        "🔴 Quyết định #4 hệ quả ② — đường (b): nguồn có thể vắng mặt, NHƯNG panel phải \
         biết để nói ra 'danh sách nguồn chưa đầy đủ'. Một `truncated_layers` rỗng ở đây \
         là FR31 vỡ không cổng nào đỏ — đúng lớp lỗi AC12 tồn tại để chặn."
    );

    // 🔴 §Hệ quả ③ đường (a) — nguồn bị cắt SẠCH phải được GỌI TÊN, không chỉ đếm.
    // FR31: *"mọi định nghĩa hiển thị nguồn"* — một nguồn bị giấu tên là không hiển thị.
    assert_eq!(
        result.hidden_sources,
        vec![("Fixture Limit B".to_owned(), 1_i64)],
        "🔴 AC12 — trần cắt sạch `fx-limit-b` khỏi `groups`, nên `hidden_sources` phải nói \
         RA nó bằng `display_name` THẬT (AC2) cộng số đầu mục thật. Một danh sách rỗng ở \
         đây đưa ta về đúng câu chung chung mà §hệ quả ③ tồn tại để thay thế."
    );

    // Nguồn CÒN mặt mà bị cắt bớt thì mang số đếm ĐẦY ĐỦ, không số của phần còn lại.
    let kept = &result.groups[0];
    assert_eq!(
        kept.total_entries,
        Some(3),
        "`fx-limit-a` có đúng 3 đầu mục và cả 3 đều lọt trang — số đếm đầy đủ vẫn phải có \
         mặt khi lớp bị đánh dấu `truncated`, để thanh nhịp không phải đoán."
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **Trần KHÔNG chạm ⇒ không một truy vấn `COUNT` nào chạy, và `total_entries` là `None`.**
///
/// §Hệ quả ③ chốt đường (a) **kèm điều kiện**: `COUNT` chạy *"CHỈ KHI `truncated = true`"*
/// — tránh trả giá cho ca thường (phần lớn lượt tra không chạm trần). Ca này ghim đúng vế
/// điều kiện đó: `None` ⇔ *"`entries` đã là toàn bộ, không cần hỏi thêm"*.
#[test]
fn an_untruncated_lookup_carries_no_source_counts_at_all() {
    let dir = temp_dir("limit-untruncated");
    let layers = build_limit_fixture(&dir);

    let result = lookup_grouped(&layers, "共", LookupMode::Exact, UNLIMITED);

    assert!(result.truncated_layers.is_empty(), "trần lớn ⇒ không lớp nào bị cắt");
    assert!(
        result.hidden_sources.is_empty(),
        "không cắt ⇒ không nguồn nào bị giấu"
    );
    for group in &result.groups {
        assert_eq!(
            group.total_entries, None,
            "🔴 `None` ⇔ không chạy `COUNT`. Một `Some(...)` ở đây nghĩa là điều kiện \
             'CHỈ KHI truncated' đã bị bỏ, và mọi lượt tra đang trả giá một truy vấn thừa."
        );
    }

    layers.close();
    cleanup(&dir);
}

/// 🔴 **Số đếm đầy đủ đi qua `verify_substring` — không đếm ỨNG VIÊN** (Bẫy 11, đổi dấu).
///
/// Nhánh 2-ký-tự trả **ứng viên** rồi mới lọc. Một `COUNT(*)` ở SQL cho nhánh này đếm cả
/// dương tính giả ⇒ thanh nhịp nói một con số **to hơn sự thật** — cùng lớp lỗi Bẫy 11,
/// chỉ ngược chiều. Fixture `TRAP11` có sẵn một dương tính giả (`國中`) xen giữa.
#[test]
fn source_counts_are_verified_not_candidate_counts() {
    let dir = temp_dir("limit-count-verify");
    build_layer(
        &dir,
        &TRAP11_LAYER,
        &SUPPORTED_SCHEMA_VERSION.to_string(),
        SUPPORTED_SCHEMA_VERSION,
    );
    let layers = DictLayers::open(&dir);

    // Trần 1 ⇒ chắc chắn `truncated` ⇒ đường `COUNT` chạy.
    let result = lookup_grouped(&layers, "中國", LookupMode::Substring, 1);
    assert_eq!(result.branch, QueryBranch::CharIdx);

    let total: i64 = result
        .groups
        .iter()
        .filter_map(|g| g.total_entries)
        .sum::<i64>()
        + result.hidden_sources.iter().map(|(_, n)| n).sum::<i64>();

    let verified = lookup_grouped(&layers, "中國", LookupMode::Substring, UNLIMITED)
        .groups
        .iter()
        .map(|g| g.entries.len() as i64)
        .sum::<i64>();

    assert_eq!(
        total, verified,
        "🔴 Số đếm đầy đủ phải BẰNG số hàng đã xác minh. Lớn hơn ⇒ đang đếm ứng viên \
         `char_idx` (gồm dương tính giả `國中`), và thanh nhịp sẽ hứa những mục không tồn tại."
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **`usize::MAX` là "không giới hạn", không phải "không trả gì cả".**
///
/// `usize::MAX as i64` = **-1**, và `-1 + 1 = 0` ⇒ `LIMIT 0` ⇒ **0 hàng, `truncated =
/// false`**: mất sạch dữ liệu, im lặng, ở một hàm `pub`. Bắt ở code review 2026-08-07.
/// Ca này ĐỎ trên phép ép kiểu `limit as i64`, XANH trên `i64::try_from(...)`.
///
/// ⚠️ Ghim **cả ba** nhánh có `LIMIT ?N` ở SQL — đó là ba chỗ phép ép kiểu từng sống.
#[test]
fn an_unbounded_limit_returns_everything_not_nothing() {
    let dir = temp_dir("limit-usize-max");
    let layers = build_limit_fixture(&dir);

    for mode in [LookupMode::Exact, LookupMode::Substring] {
        let result = lookup_grouped(&layers, "共", mode, usize::MAX);
        let rows: usize = result.groups.iter().map(|g| g.entries.len()).sum();

        assert_eq!(
            rows, 4,
            "🔴 `usize::MAX` phải trả về CẢ BỐN đầu mục của fixture. 0 hàng ở đây là phép \
             ép kiểu `limit as i64` tràn thành -1 ⇒ `LIMIT 0` — im lặng, không lỗi nào."
        );
        assert!(
            result.truncated_layers.is_empty(),
            "không cắt gì thì không được báo `truncated`"
        );
    }

    layers.close();
    cleanup(&dir);
}

/// 🔴 **`limit == 0` không được cho ra hai câu loại trừ nhau cùng lúc.**
///
/// Một cỡ trang `0` (API `pub`, không chặn) cho `groups` rỗng **kèm** `truncated = true`, và
/// panel khi đó hiện ĐỒNG THỜI *"không tìm thấy"* và *"danh sách không đầy đủ"*. Sàn dưới
/// `effective_limit` coi `0` như `1`, và cờ `truncated` nói phần còn lại.
#[test]
fn a_zero_page_size_still_returns_one_row_and_flags_the_rest() {
    let dir = temp_dir("limit-zero");
    let layers = build_limit_fixture(&dir);

    let result = lookup_grouped(&layers, "共", LookupMode::Exact, 0);
    let rows: usize = result.groups.iter().map(|g| g.entries.len()).sum();

    assert_eq!(rows, 1, "sàn dưới đọc `0` thành `1`, không thành 'rỗng'");
    assert_eq!(
        result.truncated_layers,
        vec!["limit-fixture".to_owned()],
        "và phần bị cắt vẫn phải được báo — rỗng-im-lặng là đúng thứ cờ này ngăn"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 Cùng ca AC12, nhánh **`CharIdx` một ký tự** — nhánh ĐẮT NHẤT (§Debug Log References
/// của story), nơi `LIMIT` ở SQL thật sự mua được thời gian (đo ~10×). Cùng fixture,
/// `LookupMode::Substring` với một truy vấn 1 ký tự đi qua chính nhánh đó.
#[test]
fn a_file_level_limit_on_the_char_idx_branch_also_flags_truncation() {
    let dir = temp_dir("limit-charidx");
    let layers = build_limit_fixture(&dir);

    let result = lookup_grouped(&layers, "共", LookupMode::Substring, 3);
    assert_eq!(result.branch, QueryBranch::CharIdx);

    assert_eq!(
        groups_of(&result),
        vec![("fx-limit-a".to_owned(), vec!["共".to_owned(), "共".to_owned(), "共".to_owned()])],
    );
    assert_eq!(result.truncated_layers, vec!["limit-fixture".to_owned()]);

    layers.close();
    cleanup(&dir);
}

// ─────────────────────────────────────────────────────────────────────────────────
// Bẫy 11 — trần áp SAU `verify_substring`, không trước
// ─────────────────────────────────────────────────────────────────────────────────

static TRAP11_ENTRIES: &[EntrySeed] = &[
    // TRUE — chứa "中國" NGUYÊN VĂN, id NHỎ NHẤT.
    EntrySeed {
        id: 1,
        source_id: 1,
        lang: "zh",
        headword: "中國人",
        simp: None,
        senses: &[],
    },
    // DƯƠNG TÍNH GIẢ — có cả 中 và 國 (⇒ lọt INTERSECT) nhưng không liền nhau.
    EntrySeed {
        id: 2,
        source_id: 1,
        lang: "zh",
        headword: "國中生",
        simp: None,
        senses: &[],
    },
    // TRUE.
    EntrySeed {
        id: 3,
        source_id: 1,
        lang: "zh",
        headword: "中國史",
        simp: None,
        senses: &[],
    },
    // DƯƠNG TÍNH GIẢ thứ hai.
    EntrySeed {
        id: 4,
        source_id: 1,
        lang: "zh",
        headword: "國立中",
        simp: None,
        senses: &[],
    },
    // TRUE thứ ba — id LỚN NHẤT, đứng sau cả hai dương tính giả.
    EntrySeed {
        id: 5,
        source_id: 1,
        lang: "zh",
        headword: "中國夢",
        simp: None,
        senses: &[],
    },
];

static TRAP11_LAYER: LayerSeed = LayerSeed {
    file: "trap11.db",
    layer: "trap11-fixture",
    sources: &[(1, "fx-trap11", "Fixture Trap 11")],
    entries: TRAP11_ENTRIES,
};

/// 🔴 **Bẫy 11** — `LIMIT` đặt TRƯỚC `verify_substring` cắt ứng viên trước khi lọc dương
/// tính giả, cho ra trang **ít hơn `N`** mục THẬT và một dòng "còn M nữa" NÓI DỐI.
///
/// Năm ứng viên theo `id` tăng dần: `[TRUE(1), GIẢ(2), TRUE(3), GIẢ(4), TRUE(5)]`. Trần
/// `limit = 2`.
///
/// - **Cài đặt SAI** (cắt 2 ứng viên ĐẦU rồi mới verify): `[1, 2]` → verify loại `2` →
///   còn **MỘT** hit (`id=1`), và `truncated` báo `false` (2 ứng viên thô không vượt trần)
///   dù thật ra còn **hai** mục thật (`3`, `5`) chưa được thấy — dối cả về SỐ LƯỢNG lẫn
///   về CỜ `truncated`.
/// - **Cài đặt ĐÚNG** (verify TOÀN BỘ 5 ứng viên trước, rồi cắt): verify ra `[1, 3, 5]`
///   (ba mục thật), cắt còn `[1, 3]`, `truncated = true` (3 > 2) — đúng.
#[test]
fn the_limit_is_applied_after_verification_not_before() {
    let dir = temp_dir("trap11");
    build_layer(
        &dir,
        &TRAP11_LAYER,
        &SUPPORTED_SCHEMA_VERSION.to_string(),
        SUPPORTED_SCHEMA_VERSION,
    );
    let layers = DictLayers::open(&dir);

    let result = lookup_grouped(&layers, "中國", LookupMode::Substring, 2);
    assert_eq!(result.branch, QueryBranch::CharIdx);

    let headwords: Vec<String> = result
        .groups
        .iter()
        .flat_map(|g| g.entries.iter().map(|hit| hit.headword.clone()))
        .collect();
    assert_eq!(
        headwords,
        vec!["中國人".to_owned(), "中國史".to_owned()],
        "hai mục THẬT đầu tiên theo id (1, 3) — không phải id 1 rồi dừng vì id 2 bị \
         verify loại sau khi đã cắt"
    );
    assert_eq!(
        result.truncated_layers,
        vec!["trap11-fixture".to_owned()],
        "còn id 5 (một mục THẬT thứ ba) chưa hiện ⇒ truncated PHẢI true"
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC7 — một từ nhiều TỪ LOẠI ⇒ nhiều mục riêng biệt (FR29)
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC7** — mỗi hàng `dict_sense` là **một mục riêng**, không nối `gloss` thành một
/// chuỗi.
///
/// Một chuỗi nối là một quyết định **trình bày** chôn vào tầng dữ liệu, và 1.17 không gỡ
/// ngược ra được: `"mountain; surname Shan"` không nói được từ loại nào đi với nghĩa nào.
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
        "hai hàng `dict_sense` ⇒ HAI mục, không một chuỗi nối"
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
/// hai lượt chạy cho hai thứ tự — tức một ca **flaky**, và một ca flaky **bị gỡ** chứ không
/// không được sửa.
///
/// ⚠️ Ca này khẳng định **kết quả**; luật *"không `ORDER BY ord` trần"* được cưỡng chế
/// riêng bằng máy ở `tests/dict_boundary.rs::every_ord_ordering_carries_its_tiebreaker` —
/// hai lớp cần **cả hai**, vì trên một tập nhỏ SQLite thường trả đúng thứ tự **do may mắn**,
/// và một ca hành vi một mình không phân biệt được may mắn với đúng.
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

/// Đầu mục không có nghĩa nào ⇒ **danh sách rỗng**, không lỗi. Và một `entry_id` không
/// không tồn tại cũng vậy — pha hai không phải một phép kiểm tồn tại.
#[test]
fn an_entry_without_senses_is_an_empty_list_not_an_error() {
    let dir = temp_dir("nosense");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);
    let base = layers.layer("base").expect("lớp nền");

    assert!(
        base.senses(&[6])
            .expect("đầu mục không có nghĩa")
            .is_empty(),
        "một đầu mục chỉ mang âm đọc là HỢP LỆ"
    );
    assert!(
        base.senses(&[9_999])
            .expect("id không tồn tại")
            .is_empty(),
        "một `entry_id` lạ ⇒ rỗng, không lỗi"
    );
    assert!(
        base.senses(&[]).expect("tập rỗng").is_empty(),
        "tập rỗng ⇒ không một lượt chạm database nào"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC13 vế *"không N+1"*, đo được ở tầng hành vi.**
///
/// Một tập id trải trên **nhiều lô** *(200 id ⇒ 4 lô ở cỡ [`SENSE_BATCH`] = 64)* phải cho
/// **đúng cùng** kết quả với tập id thật, không trùng một hàng nào và không rơi hàng
/// nào. Đây là ca duy nhất chạm được phép **đệm lô cuối**: lô cuối lặp lại một id đã hỏi,
/// và `IN` là phép kiểm **tập hợp** — một cài đặt đệm bằng phép **nối** sẽ nhân đôi hàng ở
/// đây và không ở đâu khác.
#[test]
fn reading_senses_across_many_batches_never_duplicates_or_drops_a_row() {
    let dir = temp_dir("batches");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);
    let base = layers.layer("base").expect("lớp nền");

    let straight = base.senses(&[1, 2, 3]).expect("ba đầu mục");

    // 200 id: ba đầu mục thật cộng một đuôi dài id không tồn tại — bốn lô, lô cuối đệm.
    let mut many: Vec<i64> = vec![1, 2, 3];
    many.extend(1_000..1_197);
    assert!(many.len() > SENSE_BATCH * 3, "phải trải qua ÍT NHẤT bốn lô");

    let batched = base.senses(&many).expect("nhiều lô");

    assert_eq!(
        batched, straight,
        "chia lô là chi tiết CÀI ĐẶT — nó KHÔNG được đổi kết quả. Trùng hàng ở đây là \
         một phép đệm sai; thiếu hàng là một lô bị bỏ."
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 Một `entry_id` **lặp lại ở HAI LÔ khác nhau** — không phải trong cùng một lô, ca
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
         mang đủ ví dụ/trích dẫn — không phải hai bản, và không phải một bản kèm một \
         bản rỗng"
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC8 — ví dụ gắn theo TỪ LOẠI; trích dẫn là trường RIÊNG có xuất xứ (FR30)
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC8** — ví dụ treo vào **`sense_id`**, không vào `entry_id`.
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
        "nghĩa `proper noun` KHÔNG có ví dụ — một cài đặt treo ví dụ theo `entry_id` sẽ \
         gắn ví dụ của nghĩa thứ nhất vào đây"
    );

    // `translation_lang` không bỏ được: nó là thứ AC10 dùng để nói *"bản dịch ví dụ này
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
/// Trộn hai bảng vào một danh sách là làm mất đúng thứ FR30 phân biệt: một *ví dụ* do
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
        "ví dụ và trích dẫn KHÔNG trộn vào nhau"
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
/// AD-44 ⑤: *"`lang` là một **trường**, không phải một **kiểu** — không tồn tại bản
/// ghi kết quả thứ hai dành riêng cho tiếng Anh"*. Một `EnSourceGroup` song song sẽ buộc
/// **mọi** chỗ gọi phân nhánh theo kiểu, và bước hợp nhất hai nhánh đó lại chính là thứ
/// AD-19 cấm.
#[test]
fn an_english_entry_travels_the_same_grouping_path_and_the_same_record_shape() {
    let dir = temp_dir("english");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = lookup_grouped(&layers, "lock", LookupMode::Exact, UNLIMITED);
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
    // xếp cả hai vào **cùng một** bộ sưu tập: một `EnSenseRecord` song song **không biên
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
/// **Không** đoán từ nội dung `pos`, không một bảng tra `"noun" ⇒ tiếng Anh` nào: một
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
/// ⚠️ **FIXTURE, không phải dữ liệu HVTĐTD thật** — xem doc-comment của [`HV_SENSES_SHAN`].
#[test]
fn a_han_viet_shaped_layer_stands_beside_the_base_layer_not_instead_of_it() {
    let dir = temp_dir("hvshape");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = lookup_grouped(&layers, "山", LookupMode::Exact, UNLIMITED);
    let codes: Vec<&str> = result
        .groups
        .iter()
        .map(|g| g.source.code.as_str())
        .collect();
    assert!(
        codes.contains(&"fx-hv") && codes.contains(&"fx-core-a"),
        "cả hai nhóm phải có mặt — KHÔNG nhóm nào bị chọn làm *câu trả lời* (FR32): {codes:?}"
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

    // Và lớp nền cạnh nó vẫn mang nhãn NGOẠI NGỮ — hai nhãn cùng tồn tại, không nhãn nào
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

/// Bộ mệnh đề tra cứu **không phụ thuộc một lớp gỡ rời nào**.
///
/// 🔴 Đây là hợp đồng của AC12: **cùng** hàm này chạy trước và sau khi xoá, không một
/// nhánh `#[cfg]` nào, không một tham số *"lớp X có mặt không"* nào. Nếu một mệnh đề
/// dưới đây phải biết lớp nào đang có, nó **không thuộc về đây**.
fn the_layer_independent_lookups_still_hold(layers: &DictLayers) {
    let shan = lookup_grouped(layers, "山", LookupMode::Exact, UNLIMITED);
    assert_eq!(shan.route, QueryRoute::Zh);
    assert_eq!(shan.branch, QueryBranch::ExactBtree);
    assert!(
        shan.skipped.is_empty(),
        "một tệp ĐÃ XOÁ KHÔNG phải một lớp *bị bỏ qua* — nó không còn là một lớp: {:?}",
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

    // 🔴 *"Rơi về nhãn tiếng Anh của lớp nền, không có đường tra cứu nào hỏng"*
    // (`epics.md:1575`) — và nó phải đúng **kể cả khi lớp Hán Việt còn đó**.
    let base = layers.layer("base").expect("lớp nền luôn có mặt");
    let senses = base.senses(&[1]).expect("đọc nghĩa lớp nền");
    assert_eq!(senses.len(), 2);
    assert_eq!(senses[0].pos_lang.as_deref(), Some("en"));
    assert!(!senses[0].examples.is_empty());

    // Một nguồn khác của **cùng** tệp nền vẫn tra được — nhóm theo `code`, không theo tệp.
    let gaoshan = lookup_grouped(layers, "高山", LookupMode::Exact, UNLIMITED);
    assert_eq!(
        groups_of(&gaoshan),
        vec![("fx-core-b".to_owned(), vec!["高山".to_owned()])]
    );

    // Đường tiếng Anh cũng vậy — nó không đi qua một lớp gỡ rời nào.
    let lock = lookup_grouped(layers, "lock", LookupMode::Exact, UNLIMITED);
    assert_eq!(
        groups_of(&lock),
        vec![("fx-core-a".to_owned(), vec!["lock".to_owned()])]
    );

    // Và trạng thái *"quá ngắn"* vẫn là chính nó.
    assert_eq!(
        lookup_grouped(layers, "ab", LookupMode::Substring, UNLIMITED).branch,
        QueryBranch::NoBranchQueryTooShort
    );
}

/// 🔴 **AC12 — món nợ FR36 mở từ Story 1.10, và nó ĐÓNG Ở ĐÂY.**
///
/// *"Xoá file → chạy lại bộ test tra cứu → hệ thống vẫn hoạt động đầy đủ với các nguồn còn
/// lại"* (AD-10). `deferred-work.md` chốt: *"Không đánh dấu FR36 là 'đã nghiệm thu' cho
/// tới khi 1.13 viết phép thử này"*.
///
/// ⚠️ **Ca dễ trượt nhất, và nó trượt XANH:** một bộ test dựng fixture rồi **luôn** mở đủ
/// ba tệp sẽ *"đạt"* AC này mà chưa bao giờ chạy đường thiếu tệp. Nên ca này **xoá tệp
/// thật** rồi **mở lại tập lớp** — và trên **Windows**, xoá một tệp còn mở là một lỗi
/// (NFR14), nên `DictLayers` phải được **drop trước** khi xoá. Đó là luật 2 của tệp này, và
/// đây là lý do luật đó tồn tại.
///
/// 🔴 Danh sách lớp gỡ rời **dẫn xuất từ chính tập lớp**, không viết cứng: mệnh đề của
/// `epics.md:1572` là *"một lớp gỡ rời **BẤT KỲ**"*, và nó không nghiệm thu được bằng một
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
        "fixture phải có ÍT NHẤT hai lớp gỡ rời — *một lớp bất kỳ* không nghiệm thu được \
         trên một lớp duy nhất. Thấy: {detachable:?}"
    );

    for target in &detachable {
        let dir = temp_dir(&format!("fr36-{target}"));
        build_all_layers(&dir);

        // ── Đối chứng dương (Task 6.3) ───────────────────────────────────────────
        //
        // 🔴 Không có nó thì *"xoá xong vẫn xanh"* và *"lớp đó chưa bao giờ được nạp"*
        // đọc **giống hệt nhau**, và ca này sẽ xanh trên một cài đặt không bao giờ mở
        // lớp gỡ rời nào.
        let layers = DictLayers::open(&dir);
        let before = lookup_grouped(&layers, "山", LookupMode::Exact, UNLIMITED);
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

        // ── Mở LẠI tập lớp: đây là thứ phép thử đo, không phải một lượt tra lại ─────
        let layers = DictLayers::open(&dir);
        assert!(
            layers.layer(target).is_none(),
            "lớp {target} vừa bị xoá khỏi đĩa mà vẫn nạp được"
        );
        assert_eq!(
            layers.layers().len(),
            detachable.len(),
            "xoá một lớp ⇒ đúng một lớp biến mất, không kéo theo lớp nào khác"
        );
        assert!(
            layers.skipped().is_empty(),
            "một tệp ĐÃ XOÁ KHÔNG được xuất hiện trong danh sách bỏ qua — *gỡ một lớp* là \
             một thao tác BÌNH THƯỜNG (FR112), không phải một lỗi dữ liệu: {:?}",
            layers.skipped()
        );

        // 🔴 **CÙNG** bộ mệnh đề, không sửa một ca nào, không một nhánh `#[cfg]` nào.
        the_layer_independent_lookups_still_hold(&layers);

        layers.close();
        cleanup(&dir);
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC13 — NFR1 trên ĐƯỜNG GOM. KHÔNG chạy trong CI.
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
/// - `cargo test` chạy nhị phân với thư mục làm việc là **`src-tauri/`**, không phải gốc
///   kho — nên `tools/dict-build/out` tương đối trỏ vào hư không, và ca này đỏ với
///   *"không phải một thư mục"* thay vì đo.
/// - Bản **debug** chậm hơn khoảng **2×** (Story 1.11 đo 7,324 ms release so với 15,045 ms
///   debug trên cùng một nhánh). Số nghiệm thu là số của bản người dùng chạy.
///
/// ⚠️ Biến **mới**, không dùng lại `AURA_DICT_BENCH_DB` của `dict_lookup.rs`: đường gom
/// cần một **thư mục**, còn biến kia trỏ một **tệp**. Một biến mang hai nghĩa là một biến
/// sẽ bị truyền sai đúng một lần, và lần đó cho một tập lớp rỗng — tức mọi con số ra `0`
/// hàng và bảng đo *"đạt"* theo đúng cách sai nhất.
///
/// **Cả hai lớp chặn đều cần thiết:** `#[ignore]` (CI không truyền `--ignored`) **và**
/// biến vắng mặt ⇒ bỏ qua. CI không có tệp `.db` nào (`.gitignore: *.db` — AD-25).
///
/// 🔴 **KHÔNG có `assert!` ngưỡng thời gian ở đây, và đó là mệnh đề của AC13** — không
/// phải một chỗ bỏ sót. AC13 nói thẳng: *"vượt trần ⇒ **GHI SỐ VÀ BÀN GIAO**, KHÔNG tự
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
        "AURA_DICT_BENCH_DIR trỏ tới {} — không phải một thư mục",
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
        "không lớp nào nạp được từ {} — mọi con số dưới đây sẽ là 0 và bảng sẽ *đạt* \
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
            let _ = lookup_grouped(&layers, query, *mode, UNLIMITED);
        }

        let mut samples = Vec::with_capacity(RUNS);
        for _ in 0..RUNS {
            let start = std::time::Instant::now();
            let _ = lookup_grouped(&layers, query, *mode, UNLIMITED);
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).expect("không có NaN trong phép đo"));

        let result = lookup_grouped(&layers, query, *mode, UNLIMITED);
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
            samples.sort_by(|a, b| a.partial_cmp(b).expect("không có NaN"));
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

    // ═════════════════════════════════════════════════════════════════════════════
    // 🔴 STORY 1.17 · TASK 8 — ĐO SAU `LIMIT`, qua ĐÚNG đường sản phẩm
    // `commands::dict::lookup()` (pha một + pha hai GỘP, cùng lượt IPC thật sẽ chạy).
    // ═════════════════════════════════════════════════════════════════════════════
    println!("\n── SAU `LIMIT` (Quyết định #4) — `commands::dict::lookup()`, LOOKUP_PAGE_LIMIT = 20 ──");
    println!(
        "  {:<18} {:<10} {:>7} {:>7} {:>9} {:>9} {:>9}",
        "nhánh", "truy vấn", "nhóm", "hàng", "p50", "p95", "p99"
    );

    let mut after_worst = (0.0f64, String::new());
    let mut json_bytes_worst = 0usize;

    // ⚠️ `commands::dict::lookup()` cố định `LookupMode::Exact` (Quyết định #3) — nhánh
    // `char_idx`/`fts_trigram` (Substring) không đi qua đường sản phẩm thật. Đo chúng vẫn cần
    // thiết cho hệ quả ①/Bẫy 11 (đã đo ở Task 0 qua `lookup_grouped` trực tiếp); ở ĐÂY đo
    // đúng những gì Panel Lookup THẬT SỰ gọi — chỉ nhánh `ExactBtree`.
    let after_cases: &[&str] = &["山", "中國", "打", "running", "Running"];

    for query in after_cases {
        for _ in 0..WARMUP {
            let _ = command_lookup(Some(&layers), query);
        }

        let mut samples = Vec::with_capacity(RUNS);
        for _ in 0..RUNS {
            let start = std::time::Instant::now();
            let _ = command_lookup(Some(&layers), query);
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).expect("không có NaN trong phép đo"));

        let response = command_lookup(Some(&layers), query);
        let rows: usize = response.grouped.groups.iter().map(|g| g.entries.len()).sum();
        let (p50, p95, p99) = (
            pct(&samples, 50.0),
            pct(&samples, 95.0),
            pct(&samples, 99.0),
        );

        println!(
            "  {:<18} {query:<10} {:>7} {rows:>7} {p50:>8.3}ms {p95:>8.3}ms {p99:>8.3}ms",
            "exact_btree",
            response.grouped.groups.len(),
        );
        if p95 > after_worst.0 {
            after_worst = (p95, format!("lookup({query:?})"));
        }

        // 🔴 Ước tính chi phí SERIALIZE JSON — phần duy nhất của "vòng IPC" đo được không cần
        // một tiến trình Tauri thật đang chạy (xem §giới hạn phép đo dưới).
        let json = serde_json::to_string(&response).expect("serialize LookupResponse");
        json_bytes_worst = json_bytes_worst.max(json.len());
    }

    // char_idx 1 ký tự (Substring) — nhánh ĐẮT NHẤT theo Task 0, đo TRỰC TIẾP qua
    // `lookup_grouped` với LIMIT vì `commands::dict::lookup()` không đi nhánh Substring
    // (Quyết định #3 cố định Exact) — đây là con số "nếu Auto-Lookup 1.18 dùng Substring".
    {
        let query = "山";
        for _ in 0..WARMUP {
            let _ = lookup_grouped(&layers, query, LookupMode::Substring, PAGE);
        }
        let mut samples = Vec::with_capacity(RUNS);
        for _ in 0..RUNS {
            let start = std::time::Instant::now();
            let _ = lookup_grouped(&layers, query, LookupMode::Substring, PAGE);
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).expect("không có NaN"));
        let (p50, p95, p99) = (
            pct(&samples, 50.0),
            pct(&samples, 95.0),
            pct(&samples, 99.0),
        );
        println!(
            "  {:<18} {query:<10} {:>7} {:>9.3}ms {:>9.3}ms {:>9.3}ms  (LIMIT={PAGE}, chỉ pha một — Substring không qua command)",
            "char_idx-1(sub)", "-", p50, p95, p99
        );
        if p95 > after_worst.0 {
            after_worst = (p95, format!("lookup_grouped(Substring, {query:?}, LIMIT={PAGE})"));
        }
    }

    println!(
        "\n  Chậm nhất SAU LIMIT: {} — p95 {:.3} ms (trần đầu-cuối NFR1: 100 ms; trần backend dẫn xuất: {CEILING_MS} ms) ⇒ {}",
        after_worst.1,
        after_worst.0,
        if after_worst.0 < 100.0 { "NFR1 DAT (đầu-cuối < 100ms)" } else { "CHUA DAT — can khao sat them" }
    );
    println!("  JSON LookupResponse lớn nhất đo được: {json_bytes_worst} byte (ước tính chi phí serialize/truyền IPC).");

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
            "khối DDL `{name}` trong `tests/dict_sources.rs` KHÔNG còn khớp nguyên văn \
             với `tools/dict-build/src/schema.rs`.\n\n\
             Lược đồ hai cây đã trôi khỏi nhau. MỌI ca trong tệp này đang kiểm một database \
             không tồn tại trong sản phẩm.\n\n\
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
/// Vì sao đây là một cổng chứ không phải một dòng ghi chú: hai workspace tách rời **có
/// chủ ý** (AC4 của Story 1.9) nên không có import chéo nào giữ hai hằng dính nhau. Một
/// lượt nâng `SCHEMA_VERSION` ở build tool mà quên bên đọc làm **mọi** tệp `.db` mới bị
/// AC4 từ chối với lý do *"quá mới"* — tức từ điển biến mất sạch, không lỗi nào được
/// ném, và triệu chứng lộ ra ở tay người dùng chứ không ở CI.
#[test]
fn the_supported_schema_version_matches_dict_build() {
    let source = read_dict_build_schema();
    let needle = format!("pub const SCHEMA_VERSION: u32 = {SUPPORTED_SCHEMA_VERSION};");

    assert!(
        source.contains(&needle),
        "`tools/dict-build/src/schema.rs` KHÔNG chứa `{needle}`.\n\n\
         `core::dict::SUPPORTED_SCHEMA_VERSION` là {SUPPORTED_SCHEMA_VERSION}, và đường đọc \
         TỪ CHỐI mọi tệp mang `user_version` lớn hơn nó (AC4). Hai hằng lệch nhau nghĩa là \
         mọi tệp `.db` do build tool viết ra sẽ bị từ chối — từ điển biến mất sạch mà không \
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
            "đọc {}: {e}. Cổng parity KHÔNG được nới thành `if let Ok(...)` — một tệp \
             nguồn không đọc được là một cổng chết, không phải một cổng đã đạt.",
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
        "chỉ `山` mang han_viet; `國` chưa được ghi ⇒ KHÔNG một hàng nào cho nó \
         (bộ lọc `IS NOT NULL`, không một ô trống câm)"
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
/// gom (Task 3), method này không được tự ý cắt chuỗi.
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

/// Tập ký tự rỗng ⇒ danh sách rỗng, **không** một lượt chạm database nào — cùng luật
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

    // 200 ký tự giả (Khu vực dùng riêng, không đầu mục nào khớp) cộng ba ký tự thật
    // chen vào giữa — bốn lô, lô cuối đệm, và ba ký tự thật rơi vào NHIỀU lô khác nhau.
    //
    // 🔴 `国` VÀ `國` là **cùng một hàng** (`id = 6` của `zzz.db`: `headword = 國`,
    // `headword_simp = 国`) nhưng nằm ở **HAI LÔ KHÁC NHAU** — đây là điều kiện tiên quyết
    // của lỗi trùng-hàng, và bản đầu của test này **không** dựng nổi nó: cả `山` lẫn `國`
    // đều chỉ khớp qua MỘT trường, nên không hàng nào có thể trả lời ở hai lô.
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
        "chia lô là chi tiết CÀI ĐẶT — nó KHÔNG được đổi kết quả. Trùng hàng ở đây là \
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
/// TOÀN BỘ nguồn đã đóng góp cho lượt hiện tại (một dòng, không một nhãn mỗi ký tự).
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
/// - `"tây,tê"` *(không khoảng trắng)* → bản đầu trả **một** phần tử `"tây,tê"`.
/// - `"chiêm, thiềm"` *(phẩy + khoảng trắng)* → bản đầu trả `primary = "chiêm,"`, **dấu
///   phẩy đuôi lên màn hình** (`str::trim` chỉ cắt khoảng trắng).
#[test]
fn multiple_readings_split_on_the_comma_convention_too() {
    let dir = temp_dir("hanviet-split-comma");
    build_all_layers(&dir);
    set_han_viet(&dir, "mmm.db", 1, "tây,tê"); // khuôn en-wiktionary-vi: không khoảng trắng
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
        "KHÔNG được mang dấu phẩy đuôi — `str::trim` chỉ cắt khoảng trắng"
    );
    assert_eq!(r2.all, vec!["chiêm".to_owned(), "thiềm".to_owned()]);

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC4 — ba trạng thái, không một.** (1) ký tự có âm; (2) ký tự không có âm ở
/// bất kỳ lớp nào (nhưng CÓ lớp đang gắn); (3) không lớp nào đang gắn.
#[test]
fn three_distinct_states_never_collapse_into_one() {
    let dir = temp_dir("hanviet-states");
    build_all_layers(&dir);
    set_han_viet(&dir, "zzz.db", 1, "sơn");

    // (1) và (2) cùng lúc: `山` có âm, `高` (từ `高山`, KHÔNG match vì headword 2 ký tự)
    // không có âm dù CÓ lớp đang gắn.
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
        "0 lớp ⇒ không âm nào, nhưng lý do PHẢI phân biệt được qua `layers_loaded`"
    );

    cleanup(&dir);
    cleanup(&empty_dir);
}

/// 🔴 **AC5 / FR36 — nghiệm thu ở mức DEGRADATION, bằng test THẬT xoá tệp.**
///
/// Xoá CẢ HAI lớp gỡ rời ⇒ tab vẫn trả âm từ lớp NỀN, không một đường nào hỏng. Phủ
/// giảm là kết quả ĐÚNG, không phải một lỗi cần sửa.
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
    assert!(after.layers_loaded, "lớp nền vẫn nạp được ⇒ không một đường nào hỏng");
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

/// `layers = None` phải đối xử GIỐNG HỆT một tập lớp rỗng — không một nhánh lỗi riêng.
#[test]
fn read_han_viet_command_treats_a_missing_state_like_an_empty_layer_set() {
    let result = command_read_han_viet(None, &["山".to_owned()]);
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
        command_read_han_viet(Some(&layers), &["山".to_owned(), "高".to_owned()]);
    let direct = lookup_han_viet(&layers, &["山", "高"]);

    assert_eq!(via_command, direct);

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Task 2 (Story 1.17) — bề mặt IPC `commands::dict::lookup`
// ═════════════════════════════════════════════════════════════════════════════════

/// `layers = None` phải đối xử GIỐNG HỆT một tập lớp rỗng — cùng luật `read_han_viet`.
#[test]
fn lookup_command_treats_a_missing_state_like_an_empty_layer_set() {
    let result = command_lookup(None, "山");
    assert!(result.grouped.groups.is_empty());
    assert!(result.grouped.skipped.is_empty());
    assert!(result.senses_by_layer.is_empty());
}

/// Vỏ IPC hydrate ĐÚNG các đầu mục pha một vừa trả về, đi qua ĐÚNG LỚP của mỗi nhóm.
#[test]
fn lookup_command_hydrates_senses_for_exactly_the_returned_entries() {
    let dir = temp_dir("lookup-command");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = command_lookup(Some(&layers), "山");

    // `山` khớp ở CẢ BA lớp — `base` (nguồn `fx-core-a`), `hv-fixture` (nguồn `fx-hv`),
    // `vp-fixture` (nguồn `fx-vp`) — AD-19, cả ba nhóm cùng có mặt, không hợp nhất.
    let layers_in_groups: Vec<&str> = result
        .grouped
        .groups
        .iter()
        .map(|g| g.layer.as_str())
        .collect();
    assert_eq!(layers_in_groups, vec!["base", "hv-fixture", "vp-fixture"]);

    // Pha hai hydrate ĐÚNG ba lớp đó, không hơn.
    let hydrated_layers: Vec<&String> = result.senses_by_layer.keys().collect();
    assert_eq!(
        hydrated_layers,
        vec![&"base".to_owned(), &"hv-fixture".to_owned(), &"vp-fixture".to_owned()]
    );

    // Nghĩa hydrate khớp ĐÚNG nội dung fixture — `BASE_SENSES_SHAN` có hai nghĩa
    // ("mountain", "surname Shan").
    let base_senses = &result.senses_by_layer["base"];
    let glosses: Vec<&str> = base_senses.iter().map(|s| s.gloss.as_str()).collect();
    assert_eq!(glosses, vec!["mountain", "surname Shan"]);

    layers.close();
    cleanup(&dir);
}

/// 🔴 Pha hai **KHÔNG** hydrate cả từ điển — chỉ đúng tập `entry_id` pha một trả về.
/// Nhánh này gián tiếp chứng minh: nếu ai đó "tối ưu nhầm" bằng cách hydrate MỌI đầu mục
/// của một lớp có nhóm (không riêng tập đã khớp), số nghĩa trả về sẽ vượt fixture.
#[test]
fn lookup_command_does_not_hydrate_entries_the_first_phase_never_matched() {
    let dir = temp_dir("lookup-command-scope");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    // `高山` chỉ khớp MỘT đầu mục (source_id 2, lớp base) — `BASE_SENSES_GAOSHAN`, MỘT nghĩa.
    let result = command_lookup(Some(&layers), "高山");
    let base_senses = &result.senses_by_layer["base"];
    assert_eq!(
        base_senses.len(),
        1,
        "chỉ MỘT nghĩa của riêng `高山` — không lẫn nghĩa của `山`/`中國` cùng lớp"
    );
    assert_eq!(base_senses[0].gloss, "alpine");

    layers.close();
    cleanup(&dir);
}

/// 🔴 **Bẫy 3 — ca test BẮT BUỘC của Task 3.** Lớp `base` VÀ lớp `hv-fixture` cùng dùng
/// `entry_id = 1` cho đầu mục `山` (fixture cố ý — xem doc-comment `LAYERS`, luật 1: "cả ba
/// tệp dùng `id = 1` có chủ ý"), nhưng NGHĨA của chúng khác nhau hoàn toàn: lớp `base` ghi
/// "mountain"/"surname Shan", lớp `hv-fixture` ghi "núi". Một cài đặt trộn `entry_id` xuyên
/// lớp (đọc nhầm nghĩa của lớp KIA cho cùng số `1`) sẽ đi qua ca MỘT lớp mà không lộ ra —
/// điều kiện tiên quyết của lỗi CHỈ dựng được với HAI lớp cùng `entry_id`.
#[test]
fn phase_two_never_mixes_entry_ids_across_two_layers_sharing_the_same_number() {
    let dir = temp_dir("phase-two-no-mix");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = command_lookup(Some(&layers), "山");

    let base_glosses: Vec<&str> = result.senses_by_layer["base"]
        .iter()
        .map(|s| s.gloss.as_str())
        .collect();
    let hv_glosses: Vec<&str> = result.senses_by_layer["hv-fixture"]
        .iter()
        .map(|s| s.gloss.as_str())
        .collect();

    assert_eq!(base_glosses, vec!["mountain", "surname Shan"]);
    assert_eq!(hv_glosses, vec!["núi"]);
    assert!(
        !hv_glosses.contains(&"mountain") && !base_glosses.contains(&"núi"),
        "entry_id 1 của lớp base rò rỉ sang lớp hv-fixture (hoặc ngược lại): \
         base={base_glosses:?} hv={hv_glosses:?}"
    );

    layers.close();
    cleanup(&dir);
}

/// `senses(&[])` không chạm database — cùng luật `han_viet(&[])`. Chứng minh gián tiếp qua
/// một truy vấn không khớp gì: `senses_by_layer` phải RỖNG, không một khoá lớp nào với danh sách
/// nghĩa rỗng đi kèm (điều đó sẽ là bằng chứng đã gọi `senses(&[])` một cách vô ích).
#[test]
fn lookup_command_calls_senses_with_an_empty_batch_for_no_layer() {
    let dir = temp_dir("lookup-command-empty-batch");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let result = command_lookup(Some(&layers), "tu-khong-ton-tai-zzz");
    assert!(
        result.senses_by_layer.is_empty(),
        "không nhóm nào khớp ⇒ không lớp nào cần hydrate: {:?}",
        result.senses_by_layer
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 `deferred-work.md:363` — một truy vấn dài hơn sàn `QUERY_LENGTH_CEILING` (200 ký tự,
/// riêng của `commands::dict`) bị CẮT trước khi vào đường tra, không panic. Chứng minh
/// bằng hiệu ứng quan sát được: 200 ký tự Latin + MỘT ký tự Hán ở CUỐI. Nếu bị cắt trước
/// khi qua `pick_route`, phần bị cắt KHÔNG còn ký tự Hán nào ⇒ `route = En`. Nếu không
/// bị cắt, ký tự Hán ở cuối vẫn còn trong chuỗi ⇒ `route = Zh`.
#[test]
fn a_query_past_the_length_ceiling_is_truncated_before_it_reaches_the_lookup() {
    let long_query: String = "x".repeat(200) + "山";
    assert_eq!(long_query.chars().count(), 201);

    let result = command_lookup(None, &long_query);
    assert_eq!(
        result.grouped.route,
        auratranslate_lib::core::dict::QueryRoute::En,
        "201 ký tự bị cắt còn 200 ký tự Latin thuần TRƯỚC khi pick_route chạy — route phải \
         là En, không Zh (nếu Zh thì ký tự Hán ở vị trí 201 đã lọt qua sàn)"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.17 — code review 2026-08-07: FR35 *"ngoại ngữ"* không ĐỒNG NGHĨA *"có ngôn ngữ"*
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **FR35 / AC4 — `pos_lang = "vi"` không PHẢI một nhãn ngoại ngữ.**
///
/// Bản đầu của 1.17 bật dấu hiệu ngoại ngữ ở webview theo `pos_lang !== null`, nên **mọi**
/// nhãn có ghi ngôn ngữ đều bị dán chip — kể cả nhãn **tiếng Việt** (`"danh từ"`). Fixture
/// mang **cả hai** ca nên ca test này không dựng thêm dữ liệu: `山` có `pos_lang = "en"` (thật
/// sự ngoại ngữ), `lock`/`dictionary` có `pos_lang = "vi"` (bản ngữ).
///
/// ⚠️ Vị từ sống ở **Rust** (AD-1) nên đây là chỗ nó được ghim — một ca test ở webview
/// chỉ ghim được bản chép thứ hai.
#[test]
fn a_native_language_pos_label_is_not_marked_as_foreign() {
    let dir = temp_dir("foreign-flag");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let base = layers.layer("base").expect("fixture có lớp nền");

    // Nhãn NGOẠI NGỮ — `pos_lang = "en"`.
    let shan = base.senses(&[1]).expect("đọc nghĩa của 山");
    let shan_sense = shan.first().expect("山 có ít nhất một nghĩa");
    assert_eq!(shan_sense.pos_lang.as_deref(), Some("en"));
    assert!(
        shan_sense.pos_is_foreign,
        "🔴 `pos_lang = \"en\"` là nhãn NGOẠI NGỮ — FR35 đòi đánh dấu rõ."
    );
    let example = shan_sense.examples.first().expect("nghĩa có ví dụ");
    assert!(
        example.translation_is_foreign,
        "AC4: *cùng luật* áp cho `ExampleRecord::translation_lang`"
    );

    // Nhãn BẢN NGỮ — `pos_lang = "vi"`.
    // ⚠️ `lock` là **entry_id 4** (nghĩa của nó mang `sense_id 5`) — hai số khác nhau, và
    // truyền nhầm số cho ra một danh sách rỗng chứ không một lỗi.
    let lock = base.senses(&[4]).expect("đọc nghĩa của lock");
    let lock_sense = lock.first().expect("lock có một nghĩa");
    assert_eq!(lock_sense.pos_lang.as_deref(), Some("vi"));
    assert!(
        !lock_sense.pos_is_foreign,
        "🔴 `pos_lang = \"vi\"` là nhãn TIẾNG VIỆT — FR35 không đòi đánh dấu nó, và một chip \
         `VI` cạnh chữ `danh từ` là đúng thứ AC4 gọi là sai. Ca này ĐỎ trên vị từ \
         `pos_lang !== null`."
    );
    let lock_example = lock_sense.examples.first().expect("nghĩa có ví dụ");
    assert!(
        !lock_example.translation_is_foreign,
        "bản dịch ví dụ tiếng Việt cũng không phải ngoại ngữ — cùng luật, cùng hàm"
    );

    layers.close();
    cleanup(&dir);
}

/// Vị từ thuần, ghim **trực tiếp** — kể cả các ca fixture không dựng được.
#[test]
fn the_foreign_language_predicate_reads_the_field_and_nothing_else() {
    use auratranslate_lib::core::dict::is_foreign_lang;

    assert!(!is_foreign_lang(None), "không ghi ngôn ngữ ⇒ không có gì để đánh dấu (AC4)");
    assert!(!is_foreign_lang(Some("vi")), "bản ngữ");
    assert!(
        !is_foreign_lang(Some("VI")),
        "mã ngôn ngữ đến từ mười nguồn dựng khác nhau — một `VI` viết hoa không được biến một \
         nhãn bản ngữ thành ngoại ngữ"
    );
    assert!(is_foreign_lang(Some("en")), "ngoại ngữ");
    assert!(is_foreign_lang(Some("zh")), "ngoại ngữ");
}

/// 🔴 **AC12 / Bẫy: truy vấn bị trần ĐỘ DÀI cắt không được đọc thành "không tìm thấy".**
///
/// Story dựng đúng cơ chế cần thiết (`truncated`/`truncated_layers`) cho trần **số hàng**
/// mà không áp cùng nguyên tắc cho trần **độ dài** — bắt ở code review 2026-08-07. Một lượt
/// bôi đen dài hơn trần bị cắt rồi tra `Exact` ⇒ chắc chắn 0 kết quả ⇒ panel nói *"không tìm
/// thấy trong từ điển"*, trong khi hệ thống không hề tra thứ người dùng chọn.
#[test]
fn an_over_long_query_is_flagged_not_silently_truncated() {
    let dir = temp_dir("query-ceiling");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let short = command_lookup(Some(&layers), "山");
    assert!(
        !short.query_truncated,
        "một truy vấn bình thường không được báo là đã cắt"
    );

    // 201 ký tự — vượt `QUERY_LENGTH_CEILING` đúng một ký tự.
    let long: String = std::iter::repeat_n('山', 201).collect();
    let result = command_lookup(Some(&layers), &long);

    assert!(
        result.query_truncated,
        "🔴 Truy vấn đã bị cắt thì phải NÓI RA. Im lặng ở đây làm panel hiện một câu SAI: \
         *'không tìm thấy trong từ điển'* cho một thứ không hề được tra."
    );

    layers.close();
    cleanup(&dir);
}

// ─────────────────────────────────────────────────────────────────────────────────
// Cỡ trang sản phẩm — fixture 25 đầu mục cùng `headword`, HƠN `LOOKUP_PAGE_LIMIT` (20)
// ─────────────────────────────────────────────────────────────────────────────────
//
// 🔴 Phải > cỡ trang thật, không chỉ > 1: một fixture 2 hàng cho cùng kết quả với mọi trần
// từ 2 trở lên, tức nó không ghim được con số nào cả — đúng lỗ mà ca này vá.

static PAGE_SIZE_ENTRIES: &[EntrySeed] = &[
    EntrySeed {
        id: 1,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 2,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 3,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 4,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 5,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 6,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 7,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 8,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 9,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 10,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 11,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 12,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 13,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 14,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 15,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 16,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 17,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 18,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 19,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 20,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 21,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 22,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 23,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 24,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
    EntrySeed {
        id: 25,
        source_id: 1,
        lang: "zh",
        headword: "頁",
        simp: None,
        senses: &[],
    },
];

static PAGE_SIZE_LAYER: LayerSeed = LayerSeed {
    file: "page-size.db",
    layer: "page-size-fixture",
    sources: &[(1, "fx-page", "Fixture Page Size")],
    entries: PAGE_SIZE_ENTRIES,
};

/// 🔴 **Ghim `LOOKUP_PAGE_LIMIT` bằng HÀNH VI** — hằng mang chính sách sản phẩm của
/// Quyết định #4 là thứ **duy nhất** của story không có lưới hồi quy (bắt ở code review
/// 2026-08-07): hai ca AC12 truyền trần **tay** (`3`) nên không đi qua `commands::dict::lookup`,
/// và mọi ca đi qua command dùng fixture ≤ 2 đầu mục mỗi lớp ⇒ đặt `LOOKUP_PAGE_LIMIT = 1`
/// vẫn xanh toàn bộ.
///
/// Ca này dựng **hơn** cỡ trang thật rồi đọc số hàng qua **đúng đường sản phẩm**.
#[test]
fn the_product_page_size_is_pinned_by_behaviour_not_only_by_a_constant() {
    let dir = temp_dir("page-size");
    build_layer(
        &dir,
        &PAGE_SIZE_LAYER,
        &SUPPORTED_SCHEMA_VERSION.to_string(),
        SUPPORTED_SCHEMA_VERSION,
    );
    let layers = DictLayers::open(&dir);

    let result = command_lookup(Some(&layers), "頁");
    let rows: usize = result.grouped.groups.iter().map(|g| g.entries.len()).sum();

    assert_eq!(
        rows, 20,
        "🔴 Cỡ trang thật của đường sản phẩm là **20** (chốt ở Task 8 theo số đo). Fixture \
         có 25 đầu mục khớp, nên con số này không đến từ dữ liệu — nó đến từ `LOOKUP_PAGE_LIMIT`. \
         Ca này ĐỎ ngay khi hằng đó đổi mà không ai đo lại."
    );
    assert!(
        !result.grouped.truncated_layers.is_empty(),
        "25 > 20 ⇒ lớp phải được đánh dấu đã cắt"
    );

    layers.close();
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// STORY 1.18 — ĐƯỜNG LUI `Substring` QUA ĐƯỜNG SẢN PHẨM (Ice chốt 2026-08-07)
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 Ba ca dưới đây đo **`commands::dict::lookup`** — đường mà webview thật đi qua — chứ
// không `lookup_grouped`. Khác biệt là cả nội dung: `lookup_grouped` nhận `mode` làm tham số,
// còn chính sách *"khi nào dùng `Substring`"* sống ở tầng command (AD-1: quy tắc nghiệp vụ
// ở Rust, và không phải thứ webview tự chọn lại mỗi lượt).

/// 🔴 **không HỒI QUY AC1** — một truy vấn `Exact` ĐANG trả lời được thì không đổi hành vi.
///
/// `山` là một đầu mục thật. Trước Story 1.18 nó trả về đúng `山`; nếu đường lui được cài
/// thành *"ngắn thì dùng `Substring`"* (thay vì *"rỗng thì thử `Substring`"*) thì lượt tra
/// này nay trả thêm `高山` — tức nghĩa người dùng hỏi bị đẩy xuống dưới nhiễu, và
/// *"`Mod+Alt+L` vẫn hoạt động y hệt trước story này"* vỡ.
#[test]
fn an_exact_hit_never_falls_back_to_substring() {
    let dir = temp_dir("fallback-exact-wins");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let response = command_lookup(Some(&layers), "山");
    let found: BTreeSet<String> = response
        .grouped
        .groups
        .iter()
        .flat_map(|g| g.entries.iter().map(|h| h.headword.clone()))
        .collect();

    assert!(found.contains("山"), "đầu mục chính xác phải có mặt");
    assert!(
        !found.contains("高山"),
        "🔴 AC1 — `Exact` đã trả lời được thì đường lui `Substring` không ĐƯỢC chạy. `高山` ở \
         đây nghĩa là đường lui đã thay thế `Exact` thay vì đỡ cho nó."
    );
    assert_eq!(
        response.grouped.branch,
        QueryBranch::ExactBtree,
        "nhánh đã đi phải là nhánh của `Exact` — `branch` là một GIÁ TRỊ, nó không được nói dối"
    );
    cleanup(&dir);
}

/// 🔴 **Đường lui chạy khi `Exact` rỗng** — và nó là thứ Ice chốt ở §Câu hỏi #1.
///
/// `高` không là đầu mục nào, nhưng `高山` chứa nó. Trước Story 1.18 lượt tra này trả **rỗng**
/// và panel nói *"không tìm thấy trong từ điển"*.
#[test]
fn an_empty_exact_lookup_falls_back_to_substring() {
    let dir = temp_dir("fallback-fires");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let response = command_lookup(Some(&layers), "高");
    let found: BTreeSet<String> = response
        .grouped
        .groups
        .iter()
        .flat_map(|g| g.entries.iter().map(|h| h.headword.clone()))
        .collect();

    assert!(
        found.contains("高山"),
        "`高` không là đầu mục nào ⇒ `Exact` rỗng ⇒ đường lui `Substring` phải tìm ra `高山`"
    );
    assert_eq!(
        response.grouped.branch,
        QueryBranch::CharIdx,
        "một ký tự Hán ở chế độ `Substring` đi nhánh `char_idx` — và `branch` phải NÓI RA \
         nhánh thật sự đã chạy, không nhánh của lượt đầu"
    );
    cleanup(&dir);
}

/// 🔴 **`query_too_short` NAY THỰC THI ĐƯỢC** — đóng `deferred-work.md:615`.
///
/// Mục `:615` ghi rằng `QueryBranch::NoBranchQueryTooShort` *"không thể xảy ra qua đường sản
/// phẩm thật"*: `commands::dict::lookup` cố định `Exact`, và `pick_branch` cho `Exact` luôn
/// trả `ExactBtree` bất kể độ dài. Nhánh đó chỉ sinh ra khi `mode = Substring` **và** route
/// `En` **và** độ dài < 3 — một tổ hợp không tồn tại trong bất kỳ lời gọi nào của 1.17.
///
/// Đường lui của Story 1.18 dựng đúng tổ hợp đó: `zz` là Latin (route `En`), không là đầu mục
/// nào (nên `Exact` rỗng), và hai ký tự (nên `Substring` ⇒ `NoBranchQueryTooShort`).
#[test]
fn a_short_latin_selection_now_reaches_the_query_too_short_state() {
    let dir = temp_dir("fallback-too-short");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let response = command_lookup(Some(&layers), "zz");

    assert_eq!(
        response.grouped.branch,
        QueryBranch::NoBranchQueryTooShort,
        "🔴 `deferred-work.md:615` — trạng thái này không tới được trước Story 1.18. Panel \
         Lookup đọc ĐÚNG trường này để nói *đoạn đang chọn quá ngắn* thay vì *không tìm thấy*, \
         và hai câu đó dẫn người dùng đi hai đường khác nhau (AD-44 ④)."
    );
    assert!(response.grouped.groups.is_empty());
    cleanup(&dir);
}

/// Truy vấn DÀI mà không tìm thấy ⇒ **không lượt tra thứ hai nào** — trần đường lui là thật.
#[test]
fn a_long_miss_does_not_pay_for_a_second_lookup() {
    let dir = temp_dir("fallback-ceiling");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    // Năm ký tự Hán — trên trần `SUBSTRING_FALLBACK_CEILING` (4).
    let response = command_lookup(Some(&layers), "山河大地人");
    assert_eq!(
        response.grouped.branch,
        QueryBranch::ExactBtree,
        "quá dài để còn đáng tra như một chuỗi con ⇒ dừng ở nhánh `Exact`"
    );
    assert!(response.grouped.groups.is_empty());
    cleanup(&dir);
}

/// 🔴 **STORY 1.18 · AC4 — ĐO NFR1 TRÊN ĐƯỜNG SẢN PHẨM THẬT, ≥ 100 TRUY VẤN KHÁC NHAU.**
///
/// ```sh
/// AURA_DICT_BENCH_DIR=/duong/dan/tuyet/doi/src-tauri/resources/dict \
///   cargo test --release --manifest-path src-tauri/Cargo.toml --test dict_sources \
///   -- --ignored --nocapture bench_the_auto_lookup_path
/// ```
///
/// ⚠️ Khác `bench_the_grouped_path_on_the_real_dictionaries` ở **ba** điểm, và cả ba đều là
/// mệnh đề của AC4:
///
/// ① Nó đo **`commands::dict::lookup`** — đường mà webview thật gọi — chứ không `lookup_grouped`.
///    Chỉ đường này mang trần độ dài, đường lui `Substring` của Story 1.18, **và** pha hai
///    (hydrate nghĩa). Đo `lookup_grouped` rồi báo cáo như đã đo đường sản phẩm là bỏ mất
///    đúng phần đắt nhất.
/// ② **100+ truy vấn KHÁC NHAU**, không 100 lượt cùng một chữ: Quyết định #5 bật **dedupe**, nên
///    một trăm lượt cùng một chữ sẽ đo đúng đường dedupe, không đường tra.
/// ③ **HAI lượt đo độc lập** — Bẫy 8. Story 1.17 đo `p99 70,742 ms` ở lượt đầu và không tái lập
///    được; ba lượt sau cho 0,566 / 1,136 / 1,793 ms. Nguyên nhân: **nhiễu page-cache**. ⇒ không
///    kết luận trên một lượt đo, và kết luận đứng trên **p99**, không chỉ p95.
///
/// 🔴 **GIỚI HẠN, KHAI TRƯỚC KHI ĐO — không SAU:** con số ở đây là **đường Rust**, nó không bao gồm
/// vòng IPC Tauri thật lẫn lượt vẽ của webview. Đó là món nợ Story 1.17 để lại và story này
/// **không đóng nó**. Xem §Completion Notes.
#[test]
#[ignore = "can thu muc chua tep .db that; chay tay qua AURA_DICT_BENCH_DIR"]
fn bench_the_auto_lookup_path_on_distinct_queries() {
    let Ok(raw) = std::env::var("AURA_DICT_BENCH_DIR") else {
        println!("AURA_DICT_BENCH_DIR vắng mặt — bỏ qua phép đo.");
        return;
    };
    let dir = PathBuf::from(&raw);
    assert!(dir.is_dir(), "AURA_DICT_BENCH_DIR trỏ tới {} — không một thư mục", dir.display());

    let layers = DictLayers::open(&dir);
    assert!(
        !layers.layers().is_empty(),
        "không lớp nào nạp được từ {} — mọi con số dưới đây sẽ là 0 và bảng *đạt* theo cách sai nhất",
        dir.display()
    );
    println!("\n═══ {} lớp từ {} ═══", layers.layers().len(), dir.display());

    // ── Bộ truy vấn: cửa sổ trượt trên văn xuôi THẬT, đúng hình dạng một lượt bôi đen ──
    const PROSE: &str = "他打開了那扇門走進了黑暗之中山河大地日月星辰春夏秋冬東南西北\
                         天地玄黃宇宙洪荒風雨雷電花草樹木江湖海洋金木水火土";
    let chars: Vec<char> = PROSE.chars().collect();
    let mut queries: Vec<String> = Vec::new();
    for width in 1..=3 {
        for start in (0..chars.len().saturating_sub(width)).step_by(1) {
            queries.push(chars[start..start + width].iter().collect());
        }
    }
    // Đường tiếng Anh — AD-44 ⑥: *"NFR1 đo TRÊN đường tiếng Anh, không suy ra từ số đo tiếng
    // Trung"*. Ice chốt bật `Substring`, nên nhánh `fts_trigram_en` nay CÓ trên đường sản
    // phẩm và phải được đo riêng.
    for w in ["running", "dictionary", "lock", "api", "dic", "ing", "the", "zzq", "ab", "xy"] {
        queries.push(w.to_owned());
    }
    queries.sort();
    queries.dedup();
    assert!(
        queries.len() >= 100,
        "AC4 đòi ≥ 100 lượt tra liên tiếp bằng truy vấn KHÁC NHAU — mới có {}",
        queries.len()
    );

    let pct = |s: &[f64], p: f64| -> f64 {
        let idx = ((p / 100.0) * s.len() as f64).ceil() as usize;
        s[idx.saturating_sub(1).min(s.len() - 1)]
    };

    let mut passes: Vec<Vec<f64>> = Vec::new();
    for pass_no in 1..=2 {
        let mut samples: Vec<f64> = Vec::with_capacity(queries.len());
        let mut fallback_hits = 0usize;
        let mut too_short = 0usize;
        for q in &queries {
            let start = std::time::Instant::now();
            let response = command_lookup(Some(&layers), q);
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
            if response.grouped.branch != QueryBranch::ExactBtree {
                fallback_hits += 1;
            }
            if response.grouped.branch == QueryBranch::NoBranchQueryTooShort {
                too_short += 1;
            }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).expect("không NaN trong phép đo"));
        println!(
            "\n── LƯỢT {pass_no} — n={} truy vấn KHÁC NHAU · {} lượt đi đường lui `Substring` \
             · {} lượt `query_too_short` ──",
            samples.len(),
            fallback_hits,
            too_short
        );
        println!(
            "  p50 {:>8.3} ms · p95 {:>8.3} ms · p99 {:>8.3} ms · max {:>8.3} ms",
            pct(&samples, 50.0),
            pct(&samples, 95.0),
            pct(&samples, 99.0),
            samples[samples.len() - 1]
        );
        passes.push(samples);
    }

    println!("\n── ĐỐI CHIẾU HAI LƯỢT (Bẫy 8 — nhiễu page-cache của lượt đầu) ──");
    for (i, s) in passes.iter().enumerate() {
        println!(
            "  lượt {}  p95 {:>8.3}  p99 {:>8.3}  max {:>8.3}",
            i + 1,
            pct(s, 95.0),
            pct(s, 99.0),
            s[s.len() - 1]
        );
    }
    println!(
        "\n🔴 GIỚI HẠN: con số trên là ĐƯỜNG RUST. Nó không gồm vòng IPC Tauri lẫn lượt vẽ của\n\
         webview — món nợ `deferred-work.md` của Story 1.17, và story này không ĐÓNG nó."
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// STORY 1.19 — BẬT/TẮT NGUỒN (AC2–AC6) VÀ GHI CÔNG (AC7–AC10)
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 Bốn luật của tệp này áp nguyên cho mọi ca dưới đây: thư mục tạm RIÊNG cho mỗi ca ·
// `close()` trước khi xoá · không một ngưỡng thời gian nào chạy trong CI · đường dẫn đi
// qua `CARGO_MANIFEST_DIR`.

use auratranslate_lib::core::dict::{SourceAttribution, list_source_attributions};
use auratranslate_lib::core::scope::parse_disabled_sources;

/// Tập `code` bị tắt, viết gọn ngay tại chỗ gọi.
fn off(codes: &[&str]) -> BTreeSet<String> {
    codes.iter().map(|c| (*c).to_owned()).collect()
}

/// Ghi đè **sáu trường giấy phép** của một nguồn trong một tệp fixture.
///
/// ⚠️ `UPDATE` sau `build_layer` chứ không một cột mới trong `LayerSeed` — cùng khuôn
/// [`set_han_viet`], và nó giữ được mệnh đề *"DDL chép nguyên văn"* mà cổng parity đứng lên.
#[allow(clippy::too_many_arguments)]
fn set_license(
    dir: &Path,
    file: &str,
    code: &str,
    license_kind: &str,
    license_id: Option<&str>,
    license_text: &str,
    attribution: &str,
    source_version: &str,
    source_url: &str,
) {
    let conn = rusqlite::Connection::open(dir.join(file))
        .unwrap_or_else(|e| panic!("mở {file} để ghi giấy phép: {e}"));
    let changed = conn
        .execute(
            "UPDATE dict_source SET license_kind = ?1, license_id = ?2, license_text = ?3, \
             attribution = ?4, source_version = ?5, source_url = ?6 WHERE code = ?7",
            rusqlite::params![
                license_kind,
                license_id,
                license_text,
                attribution,
                source_version,
                source_url,
                code
            ],
        )
        .unwrap_or_else(|e| panic!("cập nhật giấy phép cho {code} trong {file}: {e}"));
    assert_eq!(changed, 1, "{code} phải tồn tại trong {file}");
    conn.close()
        .unwrap_or_else(|(_, e)| panic!("đóng {file}: {e}"));
}

/// Ghi công của một `code`, hoặc `None` nếu bảng không có nó.
fn attribution_of<'a>(rows: &'a [SourceAttribution], code: &str) -> Option<&'a SourceAttribution> {
    rows.iter().find(|row| row.code == code)
}

/// Mã nguồn của từng nhóm, theo đúng thứ tự kết quả.
fn group_codes(result: &GroupedLookup) -> Vec<String> {
    result
        .groups
        .iter()
        .map(|group| group.source.code.clone())
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────────
// AC7 · AC8 — BẢNG GHI CÔNG DẪN XUẤT TỪ TỆP CÓ MẶT
// ─────────────────────────────────────────────────────────────────────────────────

/// 🔴 **AC7** — mọi nguồn của mọi tệp đang gắn, kèm sáu trường giấy phép của **chính tệp**.
///
/// Fixture ba lớp mang **bốn** nguồn trên **ba** tệp — cùng hình dạng "một tệp nhiều nguồn"
/// mà `dict-core.db` thật có (bảy nguồn trong một tệp).
#[test]
fn the_attribution_table_lists_every_source_of_every_present_file() {
    let dir = temp_dir("attr-all");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let rows = list_source_attributions(&layers);

    assert_eq!(
        rows.iter().map(|r| r.code.as_str()).collect::<Vec<_>>(),
        vec!["fx-core-a", "fx-core-b", "fx-hv", "fx-vp"],
        "thứ tự phải TẤT ĐỊNH: thứ tự lớp, rồi `ORDER BY code` trong tệp"
    );

    let core_a = attribution_of(&rows, "fx-core-a").expect("fx-core-a phải có mặt");
    assert_eq!(core_a.display_name, "Fixture Core A");
    assert_eq!(core_a.layer, "base");
    assert!(
        core_a.is_base,
        "`is_base` đọc từ `dict_meta('layer')` của CHÍNH tệp, không từ tên tệp \
         (`zzz.db` cố ý không nói gì về lớp bên trong) — AD-44 ① vá A2"
    );

    let hv = attribution_of(&rows, "fx-hv").expect("fx-hv phải có mặt");
    assert_eq!(hv.layer, "hv-fixture");
    assert!(!hv.is_base, "một lớp gỡ rời không bao giờ là lớp nền");

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC6 — `dict_source.lang` ĐO ĐƯỢC, và nó là thứ cho webview hỏi đúng câu.**
///
/// Ice chốt ở code review 2026-08-10. Trước lượt này, vị từ *"mọi nguồn đều tắt"* hỏi
/// **toàn tập**, nên tắt riêng nguồn **DUY NHẤT** của đường tiếng Anh vẫn cho `false` và
/// panel nói *"không tìm thấy trong từ điển"* — một câu SAI, hệ thống không hề tra. Trường
/// này là dữ kiện tối thiểu để hỏi *"mọi nguồn CỦA ĐƯỜNG NÀY đều tắt chưa"*.
///
/// 🔴 Phép kiểm neo vào **hai** mệnh đề, không một:
/// ① giá trị là một **TẬP** *(`fx-core-a` có cả hàng `zh` lẫn hàng `en` ⇒ `"en,zh"`)* — bất
///    biến *"một nguồn đúng một `lang`"* trên dữ liệu thật hôm nay là một **số đo**, không
///    một mệnh đề, và cột này không được phép gãy vào ngày nó hết đúng;
/// ② thứ tự trong tập là **tất định** (`ORDER BY lang`) — build phải tái lập được, và một
///    `GROUP_CONCAT` không sắp xếp cho ra `sha256` khác nhau giữa hai lượt dựng cùng dữ liệu.
#[test]
fn every_attribution_carries_the_language_routes_measured_from_its_own_entries() {
    let dir = temp_dir("attr-lang");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let rows = list_source_attributions(&layers);

    let core_a = attribution_of(&rows, "fx-core-a").expect("fx-core-a phải có mặt");
    assert_eq!(
        core_a.lang, "en,zh",
        "`fx-core-a` mang đầu mục CẢ HAI đường trong fixture, nên `lang` phải là một TẬP \
         hai phần tử — và sắp xếp tăng dần để hai lượt dựng cho cùng một byte"
    );

    let hv = attribution_of(&rows, "fx-hv").expect("fx-hv phải có mặt");
    assert_eq!(
        hv.lang, "zh",
        "`fx-hv` chỉ có đầu mục `zh` ⇒ tập một phần tử, không một chuỗi rỗng và không `NULL`"
    );

    // 🔴 Đối chứng NGƯỢC: giá trị **đo từ dữ liệu**, không chép từ một chỗ khai. Không một
    // hàng nào được mang một đường mà chính nó 0 đầu mục — đó là đúng hình dạng lỗi mà một
    // trường khai tay ở `SourceMeta` sẽ tạo ra, và là lý do phép đo tồn tại.
    for row in &rows {
        assert!(
            !row.lang.is_empty(),
            "`{}` mang `lang` rỗng — mọi nguồn trong fixture đều có đầu mục, nên chuỗi rỗng \
             ở đây nghĩa là lượt đo không chạy",
            row.code
        );
        for route in row.lang.split(',') {
            assert!(
                matches!(route, "zh" | "en"),
                "`{}` khai đường `{route}` mà fixture không hề gieo đầu mục nào như vậy",
                row.code
            );
        }
    }

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC8 / FR36** — xoá một tệp ⇒ ghi công của **mọi** nguồn trong tệp đó biến mất,
/// **0** mục mồ côi, và đường tra cứu vẫn chạy đầy đủ trên các lớp còn lại.
///
/// ⚠️ **Đối chứng dương bắt buộc**: khẳng định nguồn đó CÓ MẶT trước khi xoá. Thiếu nó, ca
/// này xanh trên một cài đặt không bao giờ đọc `dict_source` của lớp gỡ rời nào.
#[test]
fn deleting_a_file_removes_its_whole_attribution_block_and_leaves_no_orphan() {
    let dir = temp_dir("attr-delete");
    build_all_layers(&dir);

    let layers = DictLayers::open(&dir);
    let before = list_source_attributions(&layers);
    assert!(
        attribution_of(&before, "fx-hv").is_some(),
        "đối chứng dương — fx-hv phải có mặt TRƯỚC khi xoá tệp của nó"
    );
    let victim = layers
        .layer("hv-fixture")
        .expect("lớp gỡ rời vừa xác nhận có mặt")
        .path()
        .to_path_buf();
    layers.close();
    fs::remove_file(&victim).unwrap_or_else(|e| panic!("xoá {}: {e}", victim.display()));

    let layers = DictLayers::open(&dir);
    let after = list_source_attributions(&layers);

    assert!(
        attribution_of(&after, "fx-hv").is_none(),
        "ghi công của một tệp đã xoá không được ở lại: {:?}",
        after.iter().map(|r| &r.code).collect::<Vec<_>>()
    );
    assert_eq!(
        after.iter().map(|r| r.code.as_str()).collect::<Vec<_>>(),
        vec!["fx-core-a", "fx-core-b", "fx-vp"],
        "và KHÔNG mục mồ côi nào ở lại"
    );

    // FR36 — đường tra cứu vẫn trả lời đầy đủ trên các lớp còn lại (phép thử của AD-10).
    the_layer_independent_lookups_still_hold(&layers);

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC10 — TẮT ≠ GỠ.** Một nguồn đang tắt **vẫn** có mặt đầy đủ trong bảng ghi công.
///
/// Đây là mệnh đề dễ cài sai nhất của story: nghĩa vụ CC-BY-SA gắn với việc **phân phối**
/// dữ liệu, không với việc hiển thị nó — một bảng ghi công rụng mất một hàng vì người dùng
/// tắt một chip là một bảng ghi công **sai**.
#[test]
fn a_disabled_source_still_appears_in_full_in_the_attribution_table() {
    let dir = temp_dir("attr-disabled");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    // Đối chứng: nó THẬT SỰ biến mất khỏi kết quả tra cứu…
    let hidden = auratranslate_lib::core::dict::lookup_grouped(
        &layers,
        "山",
        LookupMode::Exact,
        UNLIMITED,
        &off(&["fx-hv"]),
    );
    assert!(
        !group_codes(&hidden).contains(&"fx-hv".to_owned()),
        "fx-hv đã tắt ⇒ không nhóm nào của nó trong kết quả"
    );

    // …mà vẫn có mặt ĐẦY ĐỦ trong bảng ghi công.
    let rows = list_source_attributions(&layers);
    let hv = attribution_of(&rows, "fx-hv").expect("fx-hv vẫn phải được ghi công khi đang TẮT");
    assert_eq!(hv.display_name, "Fixture Han Viet");
    assert!(!hv.attribution.is_empty(), "ghi công không được rỗng");

    layers.close();
    cleanup(&dir);
}

// ─────────────────────────────────────────────────────────────────────────────────
// AC9 — `license_kind` KHÔNG BỊ ÉP VÀO ENUM, VÀ CHỖ GIỮ CHO GIẤY PHÉP RIÊNG
// ─────────────────────────────────────────────────────────────────────────────────

/// 🔴 **AC9** — hai nguồn thật mang `license_id = NULL` (`tran-van-chanh` · `vietphrase`).
/// `None` phải đi qua nguyên vẹn, **không** biến thành chuỗi rỗng: hai thứ đó dẫn màn hình
/// đi hai đường khác nhau (một ô trống, hay câu của `license_kind`).
#[test]
fn a_null_license_id_stays_none_and_never_becomes_an_empty_string() {
    let dir = temp_dir("attr-null-id");
    build_all_layers(&dir);
    set_license(
        &dir,
        "aaa.db",
        "fx-vp",
        "unknown",
        None,
        "",
        "khong xac dinh duoc tac gia",
        "2026-01",
        "https://example.invalid/vp",
    );
    let layers = DictLayers::open(&dir);

    let rows = list_source_attributions(&layers);
    let vp = attribution_of(&rows, "fx-vp").expect("fx-vp phải có mặt");

    assert_eq!(vp.license_kind, "unknown");
    assert_eq!(
        vp.license_id, None,
        "`NULL` phải tới webview là `null`, KHÔNG chuỗi rỗng"
    );
    assert_eq!(
        vp.license_text_len, 0,
        "một `license_text` rỗng phân biệt được với một `license_text` có nội dung"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC9, đối chứng âm BẮT BUỘC** — một `license_kind` **bịa ra, chưa gặp bao giờ**.
///
/// AD-10 nói bằng chữ: *"mô hình hoá trường này thành enum các giấy phép mở sẽ khiến nó bị
/// gán nhãn sai ngay trên màn hình Attribution"*. Ca này là bằng chứng chạy được: giá trị đi
/// qua **nguyên văn**, không panic, không rơi về một biến thể *"khác"* đã mất thông tin —
/// nhánh mặc định của bảng ánh xạ ở `vi.json` mới là chỗ nó thành một câu tiếng Việt.
#[test]
fn a_license_kind_never_seen_before_travels_verbatim_and_never_panics() {
    let dir = temp_dir("attr-unknown-kind");
    build_all_layers(&dir);
    set_license(
        &dir,
        "aaa.db",
        "fx-vp",
        "some-licence-nobody-has-modelled-yet",
        Some("XYZZY-9.9"),
        "van ban giay phep gia lap",
        "ghi cong cua nguon nay",
        "0",
        "https://example.invalid/xyzzy",
    );
    let layers = DictLayers::open(&dir);

    let rows = list_source_attributions(&layers);
    let vp = attribution_of(&rows, "fx-vp").expect("fx-vp phải có mặt");

    assert_eq!(
        vp.license_kind, "some-licence-nobody-has-modelled-yet",
        "chuỗi MỞ — một enum ở tầng Rust sẽ nuốt giá trị này thành một biến thể sai"
    );
    assert_eq!(vp.license_id.as_deref(), Some("XYZZY-9.9"));
    assert_eq!(vp.license_text_len, "van ban giay phep gia lap".len() as i64);

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC9 — CHỖ GIỮ `author-grant`, nghiệm thu bằng FIXTURE.**
///
/// ⚠️ **GIỚI HẠN, ghi thẳng ra:** **0** nguồn THẬT nào mang `license_kind = "author-grant"`
/// hôm nay, và HVTĐTD — nguồn mà AC gốc của epic neo vào — **sẽ không tới** (Ice chốt
/// 2026-08-08: không tìm được nguồn dữ liệu). Ca này nghiệm thu **CƠ CHẾ**, không một tính
/// năng đã chạy trên dữ liệu thật.
///
/// Mệnh đề: thả một tệp mang một `license_kind` như thế vào thư mục ⇒ nó hiện đủ tên · giấy
/// phép · lớp · ghi công **mà không sửa một dòng mã nào**; xoá đi ⇒ biến mất, không mồ côi.
/// Và **danh tính tác giả đọc từ `dict_source.attribution` của chính tệp** — đó là điều kiện
/// để chỗ giữ này dùng lại được cho một nguồn KHÁC với một tác giả KHÁC.
#[test]
fn the_author_grant_placeholder_lands_and_leaves_with_its_file() {
    let dir = temp_dir("attr-author-grant");
    build_all_layers(&dir);

    // Một tệp thứ tư, dựng bằng đúng khuôn ba tệp kia — **không** một nhánh mã riêng.
    let seed = LayerSeed {
        file: "grant.db",
        layer: "grant-fixture",
        sources: &[(1, "fx-grant", "Fixture Author Grant")],
        entries: HV_ENTRIES,
    };
    let dropped = build_layer(
        &dir,
        &seed,
        &SUPPORTED_SCHEMA_VERSION.to_string(),
        SUPPORTED_SCHEMA_VERSION,
    );
    set_license(
        &dir,
        "grant.db",
        "fx-grant",
        "author-grant",
        None,
        "toan van phep rieng",
        "(c) Mot Tac Gia Nao Do -- tac gia cho phep bang van ban",
        "2026-08",
        "https://example.invalid/grant",
    );

    let layers = DictLayers::open(&dir);
    let rows = list_source_attributions(&layers);
    let grant = attribution_of(&rows, "fx-grant").expect("thả tệp vào ⇒ nguồn phải hiện");

    assert_eq!(grant.license_kind, "author-grant");
    assert_eq!(grant.display_name, "Fixture Author Grant");
    assert_eq!(grant.layer, "grant-fixture");
    assert!(!grant.is_base, "lớp gỡ rời");
    assert!(
        grant.attribution.contains("Mot Tac Gia Nao Do"),
        "DANH TÍNH TÁC GIẢ đọc từ `attribution` của CHÍNH tệp — không một cái tên nào \
         viết cứng trong mã hay trong `vi.json` (canh bằng máy ở `tests/dict_boundary.rs`)"
    );
    assert_eq!(
        grant.license_id, None,
        "một phép riêng tác giả cấp không có mã SPDX nào để mang"
    );
    layers.close();

    fs::remove_file(&dropped).unwrap_or_else(|e| panic!("xoá {}: {e}", dropped.display()));
    let layers = DictLayers::open(&dir);
    let after = list_source_attributions(&layers);
    assert!(
        attribution_of(&after, "fx-grant").is_none(),
        "xoá tệp ⇒ ghi công biến mất, không mồ côi: {:?}",
        after.iter().map(|r| &r.code).collect::<Vec<_>>()
    );

    layers.close();
    cleanup(&dir);
}

// ─────────────────────────────────────────────────────────────────────────────────
// AC3 · AC4 — NGUỒN BỊ TẮT BIẾN MẤT, CÁC NGUỒN CÒN LẠI KHÔNG ĐỔI
// ─────────────────────────────────────────────────────────────────────────────────

/// 🔴 **AC3** — tắt một nguồn ⇒ còn `k−1` nhóm, và **từng nhóm còn lại giống hệt** bản
/// trước: cùng đầu mục, cùng nghĩa, cùng thứ tự.
///
/// So sánh **cả cấu trúc**, không chỉ đếm: một bộ lọc cài sai thứ tự nhóm vẫn cho đúng số
/// lượng, và số lượng là thứ duy nhất một phép đếm nhìn thấy.
#[test]
fn disabling_one_source_leaves_every_other_group_untouched() {
    let dir = temp_dir("filter-others-intact");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let before = lookup_grouped(&layers, "山", LookupMode::Exact, UNLIMITED);
    let k = before.groups.len();
    assert!(
        k >= 3,
        "ca này chỉ có nghĩa với ít nhất ba nhóm — thấy {k}: {:?}",
        group_codes(&before)
    );

    let after = auratranslate_lib::core::dict::lookup_grouped(
        &layers,
        "山",
        LookupMode::Exact,
        UNLIMITED,
        &off(&["fx-hv"]),
    );

    assert_eq!(after.groups.len(), k - 1, "đúng MỘT nhóm biến mất");
    assert!(!group_codes(&after).contains(&"fx-hv".to_owned()));

    let survivors_before: Vec<_> = before
        .groups
        .iter()
        .filter(|g| g.source.code != "fx-hv")
        .collect();
    let survivors_after: Vec<_> = after.groups.iter().collect();
    assert_eq!(
        survivors_before, survivors_after,
        "từng nhóm còn lại phải GIỐNG HỆT bản trước — cùng nguồn, cùng đầu mục, cùng thứ tự"
    );

    // Đối chứng âm bắt buộc (AC2): bật lại ⇒ kết quả giống hệt trước khi tắt.
    let restored = lookup_grouped(&layers, "山", LookupMode::Exact, UNLIMITED);
    assert_eq!(
        restored, before,
        "bật lại một nguồn phải trả về ĐÚNG kết quả trước khi tắt — không một dấu vết nào ở lại"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC3, ca ĐẶC BIỆT — MỘT TỆP, HAI NGUỒN, TRẦN `LIMIT` ĐANG CHẠM.**
///
/// Đây là ca mà một bộ lọc ở **webview** vỡ (§Quyết định #2a lý do 1) và là ca đắt nhất của
/// story. Hai mệnh đề, và mệnh đề thứ hai là thứ mà Bẫy 2 nói tới:
///
/// ① các nguồn còn lại **nhiều kết quả hơn hoặc bằng**, **không bao giờ ít hơn**;
/// ② `hidden_sources` / `count_by_source` **không đếm** nguồn đã tắt — nếu quên, thanh nhịp
///    gọi tên một nguồn mà người dùng vừa tự tay tắt đi.
#[test]
fn the_filter_holds_at_the_limit_ceiling_and_never_shrinks_the_survivors() {
    let dir = temp_dir("filter-limit");
    let layers = build_limit_fixture(&dir);

    // Trần 3 = đúng số đầu mục của `fx-limit-a` ⇒ `fx-limit-b` bị cắt SẠCH khỏi trang.
    let before = lookup_grouped(&layers, "共", LookupMode::Exact, 3);
    assert!(
        before.truncated_layers.contains(&"limit-fixture".to_owned()),
        "đối chứng dương — ca này chỉ có nghĩa khi trần ĐANG chạm"
    );
    assert_eq!(
        before.hidden_sources,
        vec![("Fixture Limit B".to_owned(), 1)],
        "trước khi tắt gì cả, fx-limit-b bị trần cắt sạch nên nó được GỌI TÊN"
    );

    // ── ① tắt `fx-limit-b` ⇒ `fx-limit-a` KHÔNG được ít kết quả hơn ────────────────
    let without_b = auratranslate_lib::core::dict::lookup_grouped(
        &layers,
        "共",
        LookupMode::Exact,
        3,
        &off(&["fx-limit-b"]),
    );
    let a_before = before.groups[0].entries.len();
    let a_after = without_b.groups[0].entries.len();
    assert!(
        a_after >= a_before,
        "tắt một nguồn KHÔNG BAO GIỜ được làm nguồn còn lại ít kết quả đi ({a_before} → {a_after})"
    );

    // ── ② Bẫy 2 — nguồn đã tắt không được đếm ở `count_by_source` ─────────────────
    assert!(
        without_b.hidden_sources.is_empty(),
        "fx-limit-b đã TẮT ⇒ thanh nhịp không được gọi tên nó nữa: {:?}",
        without_b.hidden_sources
    );

    // ── ③ tắt nguồn ĐANG chiếm hết trang ⇒ nguồn kia vẫn được gọi tên đúng số ─────
    let without_a = auratranslate_lib::core::dict::lookup_grouped(
        &layers,
        "共",
        LookupMode::Exact,
        3,
        &off(&["fx-limit-a"]),
    );
    assert!(
        group_codes(&without_a).is_empty(),
        "trần cấp-tệp chạy TRƯỚC bộ lọc, nên trang này rỗng — hành vi ĐÚNG của \
         §Quyết định #2a, và chính là món nợ mà #2b (lọc trong SQL) sẽ trả"
    );
    assert_eq!(
        without_a.hidden_sources,
        vec![("Fixture Limit B".to_owned(), 1)],
        "nguồn ĐANG BẬT mà trần cắt sạch vẫn phải được GỌI TÊN (FR31)"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC3** — hai nguồn trong **CÙNG một tệp**, tắt một. Đây là hình dạng THẬT
/// (`dict-core.db` mang bảy nguồn), và nó là thứ phân biệt *"tắt một nguồn"* với *"bỏ một
/// lớp"*: lớp vẫn được tra, vẫn nạp được, chỉ những hàng của nguồn đó bị bỏ.
#[test]
fn two_sources_in_one_file_and_only_the_disabled_one_goes() {
    let dir = temp_dir("filter-same-file");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let before = lookup_grouped(&layers, "山", LookupMode::Exact, UNLIMITED);
    assert!(
        group_codes(&before).contains(&"fx-core-a".to_owned()),
        "đối chứng dương — fx-core-a phải có nhóm trước khi tắt"
    );

    let after = auratranslate_lib::core::dict::lookup_grouped(
        &layers,
        "山",
        LookupMode::Exact,
        UNLIMITED,
        &off(&["fx-core-a"]),
    );

    assert!(!group_codes(&after).contains(&"fx-core-a".to_owned()));
    assert!(
        after.layers_loaded,
        "lớp NỀN vẫn nạp bình thường — tắt một nguồn KHÔNG phải bỏ một lớp"
    );
    assert!(
        after.skipped.is_empty(),
        "và nó cũng không phải một lớp HỎNG: {:?}",
        after.skipped
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC6** — tắt **mọi** nguồn là một trạng thái có tên, **không** phải *"chưa gắn lớp
/// nào"*. `layers_loaded` phải ở lại `true`; nếu không, panel nói *"chưa gắn lớp từ điển
/// nào"* trong khi bốn tệp đang nằm ngay đó — một câu SAI, và AD-44 ④ cấm đích danh.
#[test]
fn turning_every_source_off_is_not_the_same_state_as_having_no_layers() {
    let dir = temp_dir("filter-all-off");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let all_off = auratranslate_lib::core::dict::lookup_grouped(
        &layers,
        "山",
        LookupMode::Exact,
        UNLIMITED,
        &off(&["fx-core-a", "fx-core-b", "fx-hv", "fx-vp"]),
    );

    assert!(all_off.groups.is_empty(), "không nguồn nào còn bật");
    assert!(
        all_off.layers_loaded,
        "🔴 `layers_loaded` phải ở lại TRUE — *mọi nguồn đều tắt* và *chưa gắn lớp nào* là \
         hai trạng thái khác nhau, và panel nói hai câu khác nhau (AC6)"
    );
    assert_eq!(
        all_off.branch,
        QueryBranch::ExactBtree,
        "lượt tra ĐÃ CHẠY — nó chỉ không còn gì để trả về"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC5** — một `code` đã lưu mà tệp của nó **không còn** ⇒ bỏ qua **im lặng**: không
/// lỗi, không panic, và không một chip mồ côi.
#[test]
fn a_disabled_code_with_no_file_behind_it_is_ignored_in_silence() {
    let dir = temp_dir("filter-ghost-code");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    let baseline = lookup_grouped(&layers, "山", LookupMode::Exact, UNLIMITED);
    let with_ghost = auratranslate_lib::core::dict::lookup_grouped(
        &layers,
        "山",
        LookupMode::Exact,
        UNLIMITED,
        &off(&["fx-mot-nguon-khong-ton-tai", "fx-mot-nguon-khac-nua"]),
    );

    assert_eq!(
        baseline, with_ghost,
        "một `code` không khớp tệp nào chỉ đơn giản không lọc được gì — nó KHÔNG được \
         là một lỗi, và cũng không được làm biến mất thứ khác"
    );
    assert!(
        list_source_attributions(&layers)
            .iter()
            .all(|row| row.code != "fx-mot-nguon-khong-ton-tai"),
        "và nó không dựng ra một hàng ghi công mồ côi"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC5 · §Quyết định #1a** — mã hoá trên đĩa là tập **BỊ TẮT**, và phép tách chịu được
/// khoảng trắng thừa, phần rỗng, chuỗi rỗng.
///
/// ⚠️ Mệnh đề *"nguồn MỚI mặc định BẬT"* nằm ngay trong hình dạng này: một `code` chưa từng
/// được lưu **không** có trong tập ⇒ nó bật. Lưu tập được-bật sẽ làm điều ngược lại.
#[test]
fn the_stored_shape_is_the_disabled_set_so_a_brand_new_source_defaults_to_on() {
    assert_eq!(parse_disabled_sources(""), BTreeSet::new());
    assert_eq!(parse_disabled_sources("   "), BTreeSet::new());
    assert_eq!(parse_disabled_sources(",,,"), BTreeSet::new());
    assert_eq!(parse_disabled_sources("a"), off(&["a"]));
    assert_eq!(parse_disabled_sources(" a , b ,, c "), off(&["a", "b", "c"]));

    // 🔴 Mệnh đề trung tâm: một nguồn xuất hiện ở bản sau (`d`) KHÔNG có trong tập đã lưu,
    // nên nó **bật**. Đây là đối chứng âm của lỗi *"rỗng im lặng"* mà AD-44 ④ cấm.
    let stored = parse_disabled_sources("a,b");
    assert!(!stored.contains("d"), "nguồn mới phải mặc định BẬT");
}

// ─────────────────────────────────────────────────────────────────────────────────
// §QUYẾT ĐỊNH #3a · BẪY 6 — BỘ LỌC ĐỔI ÂM HÁN VIỆT, KHÔNG CHỈ GIẤU BỚT
// ─────────────────────────────────────────────────────────────────────────────────

/// 🔴 **Bẫy 6, và ca này phải khẳng định ÂM CỤ THỂ.**
///
/// Một ca chỉ khẳng định *"`sources_used` không chứa nguồn đã tắt"* sẽ **XANH** trong khi âm
/// hiển thị đã đổi mà không ai đo. `priority_order` đẩy lớp NỀN xuống cuối, nên tắt một lớp
/// gỡ rời làm ký tự **rơi về âm của lớp nền** — hành vi ĐÚNG (cùng cơ chế FR36 dựa vào khi
/// một lớp bị gỡ khỏi bản cài), nhưng nó phải ĐO được, không để người đọc phát hiện sau.
#[test]
fn disabling_a_detachable_source_changes_the_reading_it_does_not_erase_it() {
    let dir = temp_dir("hanviet-filter");
    build_all_layers(&dir);
    set_han_viet(&dir, "zzz.db", 1, "am-cua-lop-nen");
    set_han_viet(&dir, "mmm.db", 1, "am-cua-lop-go-roi");

    let layers = DictLayers::open(&dir);

    let before = lookup_han_viet(&layers, &["山"]);
    let reading = before.characters[0]
        .reading
        .as_ref()
        .expect("đối chứng dương — 山 phải có âm khi chưa tắt gì");
    assert_eq!(reading.primary, "am-cua-lop-go-roi");
    assert_eq!(reading.source_code, "fx-hv");

    let after =
        auratranslate_lib::core::dict::lookup_han_viet(&layers, &["山"], &off(&["fx-hv"]));
    let reading = after.characters[0]
        .reading
        .as_ref()
        .expect("🔴 tắt lớp gỡ rời KHÔNG được xoá âm — ký tự phải rơi về lớp kế tiếp");
    assert_eq!(
        reading.primary, "am-cua-lop-nen",
        "🔴 ÂM ĐỔI, không chỉ ẩn đi — đây là con số mà Bẫy 6 đòi ghi ra"
    );
    assert_eq!(reading.source_code, "fx-core-a");
    assert!(
        !after.sources_used.contains(&"fx-hv".to_owned()),
        "và nguồn đã tắt không được viết tên mình lên tab Hán Việt (FR37)"
    );
    assert!(
        after.layers_loaded,
        "tắt một nguồn không phải *chưa gắn lớp nào*"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **§Quyết định #3a** — tắt **mọi** nguồn mang âm ⇒ ký tự về `None`, nhưng
/// `layers_loaded` vẫn `true`. Ba trạng thái của Story 1.16 không được gộp lại thành hai.
#[test]
fn turning_off_every_reading_source_leaves_the_layers_loaded_flag_alone() {
    let dir = temp_dir("hanviet-all-off");
    build_all_layers(&dir);
    set_han_viet(&dir, "zzz.db", 1, "am-cua-lop-nen");
    set_han_viet(&dir, "mmm.db", 1, "am-cua-lop-go-roi");

    let layers = DictLayers::open(&dir);
    let result = auratranslate_lib::core::dict::lookup_han_viet(
        &layers,
        &["山"],
        &off(&["fx-core-a", "fx-core-b", "fx-hv", "fx-vp"]),
    );

    assert!(result.characters[0].reading.is_none());
    assert!(result.sources_used.is_empty());
    assert!(
        result.layers_loaded,
        "*mọi nguồn đều tắt* ≠ *chưa gắn lớp nào* — AD-25 và AC6"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **AC4** — đường sản phẩm (`commands::dict::lookup`) truyền tập bị tắt xuống **CẢ
/// HAI** lượt tra, kể cả đường lui `Substring` của Story 1.18.
///
/// Bỏ nó ở lượt thứ hai là lôi ngược một nguồn đã tắt lên màn hình ở đúng ca *"lượt đầu
/// không tìm thấy gì"* — và ca đó là ca mà người dùng nhìn thấy nhiều nhất.
#[test]
fn the_substring_fallback_honours_the_filter_too() {
    let dir = temp_dir("filter-fallback");
    build_all_layers(&dir);
    let layers = DictLayers::open(&dir);

    // `國` không khớp `Exact` ở đâu cả nhưng khớp `Substring` (`中國`) — đúng ca đường lui.
    let before = auratranslate_lib::commands::dict::lookup(Some(&layers), "國", &BTreeSet::new());
    let codes_before = group_codes(&before.grouped);
    assert!(
        !codes_before.is_empty(),
        "đối chứng dương — đường lui `Substring` phải THẬT SỰ trả về gì đó: {codes_before:?}"
    );

    let target = codes_before[0].clone();
    let after = auratranslate_lib::commands::dict::lookup(Some(&layers), "國", &off(&[&target]));
    assert!(
        !group_codes(&after.grouped).contains(&target),
        "nguồn đã tắt không được sống lại ở lượt tra thứ hai"
    );

    layers.close();
    cleanup(&dir);
}

/// 🔴 **STORY 1.19 · AC12 — ĐO LẠI NFR1 VỚI BỘ LỌC, KHÔNG SUY TỪ SỐ CŨ.**
///
/// ```sh
/// AURA_DICT_BENCH_DIR=/duong/dan/tuyet/doi/tools/dict-build/out \
///   cargo test --release --manifest-path src-tauri/Cargo.toml --test dict_sources \
///   -- --ignored --nocapture bench_the_source_filter
/// ```
///
/// Ba cấu hình mà AC12 gọi đích danh — **0 nguồn tắt** · **1 nguồn tắt** · **9/10 nguồn tắt**
/// — trên cùng một bộ truy vấn, cùng đường sản phẩm (`commands::dict::lookup`), hai lượt đo
/// độc lập cho mỗi cấu hình (Bẫy 8 của Story 1.18: nhiễu page-cache của lượt đầu).
///
/// 🔴 Nó cũng đếm **tỉ lệ lượt tra chạm trần `LIMIT`** trước và sau. Nếu tỉ lệ đó **tăng
/// đáng kể**, §Quyết định #2b *(lọc thẳng trong SQL)* thành một **món nợ có số** — không một
/// linh cảm — và nó đi vào `deferred-work.md` kèm con số đo được.
///
/// 🔴 **GIỚI HẠN, khai TRƯỚC khi đo:** con số ở đây là **đường Rust**. Nó không gồm vòng IPC
/// Tauri lẫn lượt vẽ của webview — món nợ Story 1.17 để lại, và story này **không đóng nó**.
/// Và nó **không** gồm lượt đọc `global.db` để lấy tập bị tắt: đường sản phẩm đọc nó một lần
/// mỗi lượt tra ở `commands::dict::wire`, còn phép đo này truyền thẳng tập vào hàm thuần.
#[test]
#[ignore = "can thu muc chua tep .db that; chay tay qua AURA_DICT_BENCH_DIR"]
fn bench_the_source_filter_on_the_real_dictionaries() {
    let Ok(raw) = std::env::var("AURA_DICT_BENCH_DIR") else {
        println!("AURA_DICT_BENCH_DIR vắng mặt — bỏ qua phép đo.");
        return;
    };
    let dir = PathBuf::from(&raw);
    assert!(
        dir.is_dir(),
        "AURA_DICT_BENCH_DIR trỏ tới {} — không một thư mục",
        dir.display()
    );

    let layers = DictLayers::open(&dir);
    assert!(
        !layers.layers().is_empty(),
        "không lớp nào nạp được từ {} — mọi con số dưới đây sẽ là 0 và bảng *đạt* theo cách sai nhất",
        dir.display()
    );

    // 🔴 Danh sách nguồn DẪN XUẤT từ tệp có mặt (AC1) — không một `code` viết cứng trong
    // chính phép đo. Đây cũng là đối chứng dương của `list_source_attributions`.
    let all = list_source_attributions(&layers);
    println!(
        "\n═══ {} lớp · {} nguồn từ {} ═══",
        layers.layers().len(),
        all.len(),
        dir.display()
    );
    for row in &all {
        println!(
            "  {:<20} {:<8} {:<14} license_id={:<14} len(license_text)={:>6}  {}",
            row.code,
            if row.is_base { "nen" } else { "go-roi" },
            row.license_kind,
            row.license_id.as_deref().unwrap_or("NULL"),
            row.license_text_len,
            row.display_name
        );
    }
    assert!(
        all.len() >= 2,
        "AC12 đòi ba cấu hình bộ lọc — cần ít nhất hai nguồn để có cái mà tắt"
    );

    const PROSE: &str = "他打開了那扇門走進了黑暗之中山河大地日月星辰春夏秋冬東南西北\
                         天地玄黃宇宙洪荒風雨雷電花草樹木江湖海洋金木水火土";
    let chars: Vec<char> = PROSE.chars().collect();
    let mut queries: Vec<String> = Vec::new();
    for width in 1..=3 {
        for start in 0..chars.len().saturating_sub(width) {
            queries.push(chars[start..start + width].iter().collect());
        }
    }
    for w in ["running", "dictionary", "lock", "api", "dic", "ing", "the", "zzq", "ab", "xy"] {
        queries.push(w.to_owned());
    }
    queries.sort();
    queries.dedup();
    assert!(
        queries.len() >= 100,
        "AC12 đòi ≥ 100 lượt liên tiếp bằng truy vấn KHÁC NHAU — mới có {}",
        queries.len()
    );

    let pct = |s: &[f64], p: f64| -> f64 {
        let idx = ((p / 100.0) * s.len() as f64).ceil() as usize;
        s[idx.saturating_sub(1).min(s.len() - 1)]
    };

    // 🔴 Ba cấu hình của AC12. Nguồn bị tắt lấy theo **thứ tự tất định** của bảng ghi công,
    // không một `code` viết cứng.
    let one_off: BTreeSet<String> = all.iter().take(1).map(|r| r.code.clone()).collect();
    let most_off: BTreeSet<String> = all
        .iter()
        .take(all.len().saturating_sub(1))
        .map(|r| r.code.clone())
        .collect();
    let configs: [(&str, &BTreeSet<String>); 3] = [
        ("0 nguon tat", &BTreeSet::new()),
        ("1 nguon tat", &one_off),
        ("9/10 nguon tat", &most_off),
    ];

    println!(
        "\n┌─────────────────┬──────┬──────────┬──────────┬──────────┬──────────┬───────────┬──────────┐"
    );
    println!(
        "│ cau hinh        │ luot │  p50 ms  │  p95 ms  │  p99 ms  │  max ms  │ cham tran │  nhom TB │"
    );
    println!(
        "├─────────────────┼──────┼──────────┼──────────┼──────────┼──────────┼───────────┼──────────┤"
    );

    for (label, disabled) in configs {
        for pass_no in 1..=2 {
            let mut samples: Vec<f64> = Vec::with_capacity(queries.len());
            let mut truncated = 0usize;
            let mut groups_total = 0usize;
            for q in &queries {
                let start = std::time::Instant::now();
                let response = auratranslate_lib::commands::dict::lookup(Some(&layers), q, disabled);
                samples.push(start.elapsed().as_secs_f64() * 1000.0);
                if !response.grouped.truncated_layers.is_empty() {
                    truncated += 1;
                }
                groups_total += response.grouped.groups.len();
            }
            samples.sort_by(|a, b| a.partial_cmp(b).expect("không NaN trong phép đo"));
            let n = samples.len();
            println!(
                "│ {label:<15} │  {pass_no}   │ {:>8.3} │ {:>8.3} │ {:>8.3} │ {:>8.3} │ {:>6.1} %  │ {:>8.2} │",
                pct(&samples, 50.0),
                pct(&samples, 95.0),
                pct(&samples, 99.0),
                samples[n - 1],
                100.0 * truncated as f64 / n as f64,
                groups_total as f64 / n as f64,
            );
        }
    }
    println!(
        "└─────────────────┴──────┴──────────┴──────────┴──────────┴──────────┴───────────┴──────────┘"
    );

    // ── Ca xấu nhất mà Story 1.17 tìm được (`"山"`, p95 6,535 ms) — MỐC SO SÁNH ──────────
    println!("\n── CA XAU NHAT (moc so sanh 1.17: p95 6,535 ms cho \"山\") ──");
    for (label, disabled) in configs {
        const RUNS: usize = 120;
        let mut samples: Vec<f64> = Vec::with_capacity(RUNS);
        for _ in 0..RUNS {
            let start = std::time::Instant::now();
            let _ = auratranslate_lib::commands::dict::lookup(Some(&layers), "山", disabled);
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).expect("không NaN"));
        println!(
            "  {label:<15}  p50 {:>8.3}  p95 {:>8.3}  p99 {:>8.3}  max {:>8.3}",
            pct(&samples, 50.0),
            pct(&samples, 95.0),
            pct(&samples, 99.0),
            samples[samples.len() - 1]
        );
    }

    println!(
        "\n🔴 GIOI HAN: con so tren la DUONG RUST — khong gom vong IPC Tauri lan luot ve cua\n\
         webview (mon no cua Story 1.17, story nay KHONG dong no), va khong gom luot doc\n\
         `global.db` de lay tap bi tat."
    );
}
