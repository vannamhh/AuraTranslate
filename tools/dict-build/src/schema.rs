//! Lược đồ `dict-core.db` — hằng, đọc được cạnh mã. Chốt cứng ở §Quyết định #2 của
//! Story 1.9, khớp khuôn `store::schema::SCHEMA_MIGRATION_LOG_DDL` của `src-tauri`:
//! DDL là `&'static str`, một hằng cho một khối logic, không dựng bằng `format!`
//! từ trạng thái lúc chạy.
//!
//! Tệp này KHÔNG di trú (§Quyết định #7): `dict-core.db` chỉ đọc trọn đời, được thay
//! nguyên tệp qua release mới. `PRAGMA user_version` và `dict_meta('schema_version', …)`
//! tồn tại để đường ĐỌC (Story 1.11) từ chối một tệp mới hơn nó biết, không phải để
//! dựng một bộ di trú.

/// Phiên bản lược đồ hiện tại. Ghi vào cả `PRAGMA user_version` lẫn `dict_meta`.
///
/// 🔴 1 → 2 ở Story 1.10c: thêm cột `dict_entry.nom_reading` (AC6). `schema.rs` KHÔNG di
/// trú (xem doc-comment module) — bờ đọc (`src-tauri/src/core/dict/layer.rs::
/// SUPPORTED_SCHEMA_VERSION`) phải nâng CÙNG LƯỢT, nếu không mọi tệp `.db` mới bị từ chối
/// bằng `SchemaTooNew` (§Bẫy 1 của story).
///
/// 🔴 2 → 3 ở Story 1.19 *(Ice chốt ở code review 2026-08-10)*: thêm cột `dict_source.lang`
/// — xem [`DICT_SOURCE_DDL`]. Nâng số ở đây là **bắt buộc**, không một thủ tục cho đủ: bờ
/// đọc `SELECT … lang FROM dict_source`, nên một tệp v2 *(không có cột đó)* làm câu lệnh
/// gãy bằng `no such column`, và `list_source_attributions` sẽ **bỏ im lặng cả lớp** khỏi
/// bảng ghi công kèm một dòng `stderr` mà không ai đọc. Với số này, cùng tệp đó bị từ chối
/// bằng `SkipReason::SchemaTooNew` — một câu **có tên**, hiện ra được trên màn hình.
pub const SCHEMA_VERSION: u32 = 3;

/// Siêu dữ liệu của chính tệp — khoá/giá trị, đọc được bằng mắt qua `sqlite3` CLI.
pub const DICT_META_DDL: &str = "\
CREATE TABLE dict_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);";

/// Một nguồn từ điển, tự mang giấy phép và ghi công của chính nó (AD-19, AD-10).
///
/// Cùng khuôn với lớp gỡ rời — Story 1.10 dùng LẠI bảng này khi dựng từng tệp lớp gỡ
/// rời riêng, không dựng bảng khác. `license_kind` là chuỗi mở, KHÔNG phải enum các
/// giấy phép mở (AD-10): mô hình hoá thành enum sẽ khiến một giấy phép mới (như
/// `author-grant` của HVTĐTD, Story 1.10) bị gán nhãn sai ngay trên màn hình Attribution.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `lang` — TẬP ĐƯỜNG NGÔN NGỮ NGUỒN NÀY PHỤC VỤ (Ice chốt ở code review 2026-08-10)
/// ─────────────────────────────────────────────────────────────────────────────
/// Chuỗi *"cắt theo `,`, trim, bỏ rỗng"* — **cùng quy ước** với `core::scope::
/// parse_disabled_sources` và `decodeDisabled` phía webview, để không sinh ra một quy ước
/// mã hoá thứ hai. Trên dữ liệu thật hôm nay mọi nguồn cho đúng **một** giá trị (`"zh"`
/// hoặc `"en"`), nhưng cột giữ một **TẬP** vì bất biến *"một nguồn đúng một `lang`"* là một
/// **số đo**, không một mệnh đề: một nguồn bắc qua hai đường sẽ không làm gãy gì cả.
///
/// 🔴 **ĐO từ `dict_entry` đã chèn, KHÔNG khai tay trong `SourceMeta`** — xem
/// [`crate::insert::backfill_source_langs`]. `lang` là một dữ kiện **của dữ liệu**, nên một
/// hằng viết tay cạnh `license_kind` là một nguồn sự thật thứ hai, và nó sẽ lệch vào đúng
/// ngày một parser đổi nhãn `lang` mà không ai nhớ sửa hằng kia (AD-44 ① vá A2).
///
/// ⚠️ Vì sao cột này tồn tại thay vì để runtime tự `SELECT DISTINCT`: đo 2026-08-10 trên
/// bốn tệp thật, dẫn xuất lúc đọc tốn **~480 ms** mỗi lượt khởi động *(`dict-core.db` 374 ms
/// cho 594.770 hàng, `dict-vietphrase.db` 97 ms cho 679.302 hàng)* vì `dict_entry` không có
/// index trên `source_id`. Trả giá đó một lần lúc dựng là **0 ms** lúc đọc.
pub const DICT_SOURCE_DDL: &str = "\
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

/// Một đầu mục. `headword_simp` NULL khi nguồn không phân biệt phồn/giản.
///
/// 🔴 Story 1.10c AC2 — HAI cột âm đọc tiếng Việt, ranh giới TÁCH BẠCH:
/// - `han_viet` là ÂM ĐỌC, không phải NGHĨA — trộn vào `dict_sense` làm Panel Lookup
///   hiện âm đọc như một định nghĩa (§Thông tin kỹ thuật #3). Sau story 1.10c, giá trị ở
///   đây LUÔN đến từ một nhãn "âm Hán Việt" gắn tường minh (Thiều Chửu · en-wiktionary-vi
///   · Trần Văn Chánh) — KHÔNG BAO GIỜ từ `Unihan kVietnamese` (đó là âm NÔM, xem
///   `nom_reading`; §Phát hiện của story đo được 92,4% giá trị cũ trùng một âm Nôm).
/// - `nom_reading` là ÂM ĐỌC tiếng Việt khi ký tự dùng làm chữ NÔM, không phải âm Hán
///   Việt. `Unihan kVietnamese` đổ vào ĐÂY (AC1: đổi vai, không mất dữ liệu); nhãn
///   `nom-reading` của en-wiktionary-vi cũng đổ vào ĐÂY (AC3).
pub const DICT_ENTRY_DDL: &str = "\
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

/// Một nghĩa. FR29: một từ nhiều từ loại ⇒ nhiều hàng ở đây, không gộp thành một
/// chuỗi `gloss`. `source_id` 🔴 `NOT NULL` — đây là cưỡng chế của AC2, không phải một
/// gợi ý; cộng `PRAGMA foreign_keys = ON` lúc mở kết nối (Bẫy 7: mặc định TẮT trong
/// SQLite, phải bật MỖI kết nối). `pos_lang` tồn tại vì FR35 — nhãn từ loại ngoại ngữ
/// phải được ĐÁNH DẤU RÕ, không đoán được từ nội dung `pos`.
pub const DICT_SENSE_DDL: &str = "\
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

/// Ví dụ minh hoạ cho một nghĩa. FR30: treo vào `sense_id` (theo TỪ LOẠI), không
/// treo vào `entry_id`.
pub const DICT_EXAMPLE_DDL: &str = "\
CREATE TABLE dict_example (
  id               INTEGER PRIMARY KEY,
  sense_id         INTEGER NOT NULL REFERENCES dict_sense(id),
  text             TEXT NOT NULL,
  translation      TEXT,
  translation_lang TEXT,
  ord              INTEGER NOT NULL
);";

/// Trích dẫn văn bản cho một nghĩa — bảng RIÊNG với ví dụ vì nó mang xuất xứ
/// (`work`, `author`); ví dụ thì không (FR30).
pub const DICT_CITATION_DDL: &str = "\
CREATE TABLE dict_citation (
  id       INTEGER PRIMARY KEY,
  sense_id INTEGER NOT NULL REFERENCES dict_sense(id),
  text     TEXT NOT NULL,
  work     TEXT,
  author   TEXT,
  ord      INTEGER NOT NULL
);";

/// Chỉ mục ký tự Hán → đầu mục, phủ CẢ phồn thể lẫn giản thể (Bẫy 8: phủ mỗi phồn thể
/// làm `国` trả rỗng trong 0,01ms mà không lỗi nào được ném — đúng lớp lỗi FR39 chặn).
/// `WITHOUT ROWID` vì khoá chính hỗn hợp `(ch, entry_id)` đã là khoá tự nhiên, tránh một
/// tầng rowid thừa cho một bảng thuần tra cứu.
pub const CHAR_IDX_DDL: &str = "\
CREATE TABLE char_idx (
  ch       TEXT    NOT NULL,
  entry_id INTEGER NOT NULL REFERENCES dict_entry(id),
  PRIMARY KEY (ch, entry_id)
) WITHOUT ROWID;";

/// `idx_example_sense`/`idx_citation_sense` tồn tại vì cùng lý do `idx_sense_entry` tồn
/// tại: cả ba đều là mục tiêu JOIN trên khoá ngoại mà đường đọc của Story 1.11 sẽ dùng
/// (Review Findings Group A) — thiếu chỉ mục thì JOIN quét toàn bảng.
pub const ENTRY_INDEXES_DDL: &str = "\
CREATE INDEX idx_entry_headword      ON dict_entry(headword);
CREATE INDEX idx_entry_headword_simp ON dict_entry(headword_simp);
CREATE INDEX idx_sense_entry         ON dict_sense(entry_id);
CREATE INDEX idx_example_sense       ON dict_example(sense_id);
CREATE INDEX idx_citation_sense      ON dict_citation(sense_id);";

/// Chỉ mục trigram trên ĐẦU MỤC (AD-26 nhánh 3 — "chuỗi con 3+ ký tự" của đầu mục,
/// không phải của nghĩa, Bẫy 5). External-content vì `dict_entry` chỉ đọc trọn đời
/// (§Quyết định #3) — không `UPDATE` nào để trigger phải theo.
pub const ENTRY_FTS_DDL: &str = "\
CREATE VIRTUAL TABLE entry_fts USING fts5(
  headword, content='dict_entry', content_rowid='id', tokenize=\"trigram\");";

/// Chỉ mục FTS CHÍNH trên nghĩa (AD-27) — `remove_diacritics 0`, PHÂN BIỆT dấu tiếng
/// Việt. Tên trần vì nó là mặc định (§Quyết định #3, Bẫy 4: thiếu `tokenize` rơi về
/// `remove_diacritics 1` im lặng — không lỗi, không cảnh báo, chỉ sai kết quả).
pub const SENSE_FTS_DDL: &str = "\
CREATE VIRTUAL TABLE sense_fts USING fts5(
  gloss, content='dict_sense', content_rowid='id',
  tokenize=\"unicode61 remove_diacritics 0\");";

/// Chỉ mục FTS PHỤ, xoá dấu (`remove_diacritics 2` — bản đầy đủ hơn `1`, cần SQLite
/// ≥ 3.27, `bundled` vượt xa). Hậu tố `_nd` nói rõ đây KHÔNG phải mặc định — một giai
/// đoạn sau đọc tên bảng chứ không đọc AD-27.
pub const SENSE_FTS_ND_DDL: &str = "\
CREATE VIRTUAL TABLE sense_fts_nd USING fts5(
  gloss, content='dict_sense', content_rowid='id',
  tokenize=\"unicode61 remove_diacritics 2\");";

/// Thứ tự dựng bảng — tôn trọng FK: `dict_source` trước `dict_entry`, `dict_entry`
/// trước `dict_sense`/`char_idx`, `dict_sense` trước `dict_example`/`dict_citation`.
/// Ba bảng FTS5 external-content KHÔNG cần dữ liệu có trước lúc `CREATE`, nhưng dựng
/// sau cùng để đọc mã theo đúng thứ tự logic: bảng gốc rồi mới tới chỉ mục trên nó.
pub const ALL_TABLE_DDL: &[&str] = &[
    DICT_META_DDL,
    DICT_SOURCE_DDL,
    DICT_ENTRY_DDL,
    DICT_SENSE_DDL,
    DICT_EXAMPLE_DDL,
    DICT_CITATION_DDL,
    CHAR_IDX_DDL,
    ENTRY_INDEXES_DDL,
    ENTRY_FTS_DDL,
    SENSE_FTS_DDL,
    SENSE_FTS_ND_DDL,
];
