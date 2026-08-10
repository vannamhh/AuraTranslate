//! Một **tệp `.db` = một lớp** (AD-10), và tập lớp phát hiện bằng **QUÉT THƯ MỤC**.
//!
//! **Tệp này không bao giờ gọi vị từ điều phối** — `route` đi xuống từ tầng gom như
//! một tham số (AD-44 ①). `tests/dict_boundary.rs` cưỡng chế điều đó bằng máy, đếm **tệp**.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO KHÔNG TỒN TẠI MỘT SỔ ĐĂNG KÝ
//! ─────────────────────────────────────────────────────────────────────────────
//! AD-44 ① vá A2: *"**Không tồn tại sổ đăng ký "tệp `.db` nào chứa ngôn ngữ nào"**. Một
//! sổ như thế là nguồn sự thật thứ hai cho một dữ kiện đã nằm trong dữ liệu […] và nó sai
//! **im lặng** vào đúng ngày một lớp gỡ rời được thêm hay gỡ đi (FR112)."*
//!
//! Luật đó viết cho **ngôn ngữ**; module này áp nó cho **danh tính lớp** vì cùng một lý do,
//! và vì FR36 nói *"gỡ một lớp = xoá một file"* — một danh sách tên tệp viết cứng trong mã
//! làm mệnh đề đó thành **sai**. Nên:
//!
//! - Tập lớp = **mọi** tệp `*.db` trong một thư mục, không một danh sách tên nào.
//! - Danh tính lớp đọc từ **`dict_meta('layer')` của chính tệp**, không từ tên tệp.
//! - Nguồn đọc từ **`dict_source` của chính tệp**, không từ một bảng tra ở tầng gom.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ CHÍNH SÁCH PHIÊN BẢN SỐNG Ở ĐÂY, KHÔNG Ở `ReadOnlyDb`
//! ─────────────────────────────────────────────────────────────────────────────
//! `core/store/readonly.rs:57-60` giao thẳng: *"**Không đọc `PRAGMA user_version`, không
//! không di trú, không kiểm phiên bản lược đồ ở đây.** Việc từ chối một tệp mới hơn ứng
//! dụng là quyết định của **tầng gọi (Story 1.13**, nơi biết mình đang mở *lớp* nào và làm
//! gì khi một lớp bị từ chối)"*. Đẩy phép kiểm ngược vào `ReadOnlyDb` là **chôn một chính
//! sách vào một cơ chế**.
//!
//! ⚠️ Mọi chuỗi chẩn đoán ở tệp này viết **KHÔNG DẤU** — `scripts/check-i18n.mjs` Kiểm A
//! quét `src-tauri/**/*.rs` và tệp này không nằm trong danh sách miễn trừ. Comment thì
//! được, **chuỗi thì không**.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::store::{ReadHandle, ReadOnlyDb, SqlError, SqlResult, StoreError, StoreKind};
use crate::ports::DictionarySource;

use super::{
    HanVietHit, LookupResult, QueryBranch, QueryRoute, SenseRecord, SourceAttribution, SourceInfo,
    han_viet, senses,
};

/// Phiên bản lược đồ tệp `.db` mà đường đọc này hiểu.
///
/// 🔴 Phải **bằng** `tools/dict-build/src/schema.rs::SCHEMA_VERSION`. Hai workspace tách
/// rời **có chủ ý** (AC4 của Story 1.9) nên không có import chéo nào giữ hai hằng dính
/// nhau — `tests/dict_sources.rs::the_supported_schema_version_matches_dict_build` đọc tệp
/// kia **dưới dạng văn bản** và canh đúng mệnh đề đó.
///
/// 🔴 1 → 2 ở Story 1.10c, CÙNG LƯỢT với `tools/dict-build`: cột `dict_entry.nom_reading`
/// mới (AC6).
///
/// 🔴 2 → 3 ở Story 1.19 *(code review 2026-08-10)*, CÙNG LƯỢT với `tools/dict-build`: cột
/// `dict_source.lang` mới, mà [`DictLayer::attributions`] **đọc đích danh**. Một tệp `.db`
/// **v3** phải mở được; một tệp **v4** giả lập vẫn bị từ chối bằng
/// [`SkipReason::SchemaTooNew`] (AD-30 — mở tiến, không mở lùi).
///
/// 🔴 **CẢNH BÁO — phép kiểm dưới đây KHÔNG chặn tệp CŨ, chỉ chặn tệp MỚI.** Điều kiện là
/// `file_version > SUPPORTED_SCHEMA_VERSION`, nên một tệp **v2** vẫn được **NHẬN** *(2 > 3
/// là sai)*, rồi mới gãy ở [`DictLayer::attributions`] bằng `no such column: lang`. Hậu quả
/// **không** phải một câu từ chối có tên: [`super::list_source_attributions`] bỏ **im lặng
/// cả lớp** kèm một dòng `stderr`, nên bảng ghi công rỗng và dải chip biến mất, trong khi
/// **tra cứu vẫn chạy bình thường** *(đường tra không đọc cột `lang`)* — hỏng nửa vời.
///
/// ⚠️ Ca này chạm một bản cài **trộn tệp cũ với mã mới**, ví dụ một máy dev chưa chép lại
/// bốn tệp `.db` sau lượt dựng. Bản phát hành không chạm: cả bốn tệp đi cùng một release và
/// `sha256` trong `dict-manifest.toml` ràng chúng lại. Đường bịt thật là một **sàn phiên
/// bản** ở đây *(từ chối `file_version < MINIMUM_SCHEMA_VERSION`)* — chưa cài, ghi ở
/// `deferred-work.md` §code review 1-19.
pub const SUPPORTED_SCHEMA_VERSION: u32 = 3;

/// Phiên bản lược đồ **CŨ NHẤT** đường đọc này còn đọc nổi — Ice chốt ở code review
/// 2026-08-10.
///
/// 🔴 **Đây là vế còn thiếu của [`SUPPORTED_SCHEMA_VERSION`], không một hằng cho đối xứng.**
/// Phép kiểm phiên bản trước lượt này chỉ hỏi *"quá mới?"*, nên nó bảo vệ đúng **một** chiều.
/// Chiều kia hở ra ngay lượt đầu tiên có ai đó nâng lược đồ: bờ đọc gõ `dict_source.lang`,
/// còn một tệp v2 thì không có cột đó — và nó **lọt cửa** rồi gãy ở giữa đường.
///
/// ⚠️ **Giá trị bằng `SUPPORTED_SCHEMA_VERSION` là ĐÚNG cho hôm nay, không phải một chỗ tạm.**
/// Lược đồ này **không di trú** (§Quyết định #7 của Story 1.9: tệp chỉ đọc trọn đời, thay
/// nguyên tệp qua release mới), và bốn tệp đi cùng **một** release dưới `sha256` của
/// `dict-manifest.toml`. Nới nó xuống chỉ có nghĩa vào ngày đường đọc **thật sự** đọc được
/// một tệp cũ hơn — tức ngày ai đó viết một câu `SELECT` biết cách sống thiếu cột mới.
///
/// 🔴 **Nâng `SCHEMA_VERSION` ⇒ nâng CẢ HAI hằng ở đây**, trừ khi có một lý do đo được để
/// giữ chiều lùi. Nâng mỗi `SUPPORTED` là dựng lại đúng cái bẫy vừa gỡ.
pub const MINIMUM_SCHEMA_VERSION: u32 = 3;

/// Danh tính của lớp **nền**. Mọi giá trị khác là một lớp **gỡ rời**.
///
/// ⚠️ Đây **không** phải một mã nguồn (`dict_source.code`) — nó là giá trị của
/// `dict_meta('layer')`, do `tools/dict-build/src/insert.rs:140` ghi vào từng tệp.
///
/// 🔴 `pub(super)` chứ không `private`: tầng gom (`mod.rs::priority_order`) cần **chính
/// hằng này**, không một bản chép thứ hai — xem `mod.rs::BASE_LAYER_NAME`.
pub(super) const BASE_LAYER: &str = "base";

/// Vì sao một tệp trong thư mục **không** trở thành một lớp.
///
/// 🔴 Một **GIÁ TRỊ**, không phải một dòng `eprintln!`: *"Rỗng im lặng bị cấm; rỗng có
/// lý do thì không"* (AD-44 ④). Panel Lookup (1.17) phải phân biệt được *"đã tra mà không
/// không khớp"* với *"lớp không nạp được"* — hai câu đó dẫn người dùng đi hai đường khác
/// nhau, và chúng chỉ phân biệt được nếu lý do đi ra theo **kết quả**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// Không mở được tệp.
    OpenFailed {
        /// Lỗi thô, chỉ để chẩn đoán. Không đi lên giao diện.
        detail: String,
    },

    /// Mở được, nhưng không đọc nổi `dict_meta` — tệp không mang lược đồ từ điển.
    MetaUnreadable {
        /// Lỗi thô, chỉ để chẩn đoán.
        detail: String,
    },

    /// `dict_meta` có, nhưng thiếu một hàng bắt buộc (`layer` hoặc `schema_version`).
    MetaRowMissing {
        /// Khoá vắng mặt — **dữ liệu**, không phải một câu.
        key: String,
    },

    /// 🔴 Tệp **mới hơn** ứng dụng. Không đoán, không di trú — từ chối có tên.
    SchemaTooNew {
        /// `PRAGMA user_version` đọc được từ tệp.
        file_version: u32,
        /// [`SUPPORTED_SCHEMA_VERSION`].
        supported: u32,
    },

    /// 🔴 Tệp **CŨ hơn** thứ đường đọc này còn đọc nổi — Ice chốt ở code review 2026-08-10.
    ///
    /// Vì sao nó phải tồn tại, bằng một ca THẬT: lượt nâng `SCHEMA_VERSION` 2→3 thêm cột
    /// `dict_source.lang`, mà [`DictLayer::attributions`] gõ đích danh cột đó. Trước hằng
    /// [`MINIMUM_SCHEMA_VERSION`], một tệp **v2** vẫn lọt qua cửa *(phép kiểm cũ chỉ hỏi
    /// `file_version > SUPPORTED`)* rồi gãy ở giữa đường bằng `no such column: lang` — và
    /// [`super::list_source_attributions`] **nuốt** lỗi đó, bỏ im lặng cả lớp. Kết quả trên
    /// màn hình: dải chip **biến mất không dấu vết** và bảng ghi công nói *"chưa gắn lớp từ
    /// điển nào"*, trong khi **tra cứu vẫn chạy** *(đường tra không đọc `lang`)*.
    ///
    /// 🔴 **Ca đó không phải giả định — Ice gặp nó ở lần chạy thử đầu tiên**, trên một máy
    /// dev còn giữ bốn tệp `.db` dựng ngày 2026-08-07. Hỏng **nửa vời và không ai biết** là
    /// đúng thứ enum này tồn tại để biến thành một câu đọc được.
    SchemaTooOld {
        /// `PRAGMA user_version` đọc được từ tệp.
        file_version: u32,
        /// [`MINIMUM_SCHEMA_VERSION`].
        minimum: u32,
    },

    /// 🔴 **Hai chỗ ghi phiên bản NÓI KHÁC NHAU.**
    ///
    /// Story 1.9 §Quyết định #2 ghi cả `PRAGMA user_version` lẫn
    /// `dict_meta('schema_version')` **có chủ ý**. Hai chỗ đó lệch nghĩa là tệp **không**
    /// do `tools/dict-build` viết ra — và tin nửa nào cũng là **đoán**.
    SchemaVersionDisagrees {
        /// `PRAGMA user_version`.
        user_version: u32,
        /// `dict_meta('schema_version')` — nguyên văn, kể cả khi không phải một số.
        meta_version: String,
    },

    /// Không đọc nổi `dict_source` của tệp.
    SourcesUnreadable {
        /// Lỗi thô, chỉ để chẩn đoán.
        detail: String,
    },

    /// Hai tệp khai **cùng một danh tính lớp**.
    DuplicateLayer {
        /// Danh tính bị trùng.
        layer: String,
    },

    /// 🔴 Hai lớp khai **cùng một `dict_source.code`** — một **lỗi dữ liệu có tên**.
    ///
    /// **Không** im lặng gộp hai tệp vào một nhóm: khoá gom là `code`, nên hai tệp cùng
    /// `code` làm một nhóm mang nghĩa của hai nguồn khác nhau — đúng thứ AD-19 cấm, xảy ra
    /// ở tầng dữ liệu thay vì ở tầng mã.
    DuplicateSourceCode {
        /// Mã bị trùng.
        code: String,
        /// Lớp đã giữ mã đó trước, theo thứ tự tất định.
        first_layer: String,
    },

    /// 🔴 **CHÍNH tệp này** khai hai hàng `dict_source` cùng một `code` — lỗi dữ liệu ngay
    /// trong nó, không phải một va chạm giữa hai tệp.
    ///
    /// `source()` chỉ có thể trả **một** [`SourceInfo`] cho một `code`; im lặng giữ hàng
    /// đầu và bỏ hàng sau là giấu một dữ kiện thay vì báo nó — đối xứng với
    /// [`SkipReason::DuplicateSourceCode`], vốn đã bắt đúng ca này khi nó xảy ra **giữa**
    /// hai lớp.
    DuplicateSourceCodeInFile {
        /// Mã bị trùng.
        code: String,
    },

    /// Lớp nạp được nhưng **một lượt tra trên nó** hỏng. Không làm hỏng cả lượt tra.
    LookupFailed {
        /// Lỗi thô, chỉ để chẩn đoán.
        detail: String,
    },
}

impl fmt::Display for SkipReason {
    /// ⚠️ **KHÔNG DẤU** — xem doc-comment module. Đây là chuỗi chẩn đoán cho người đang
    /// đọc stderr, không phải một câu cho người dùng (câu là việc của `core/i18n`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkipReason::OpenFailed { detail } => write!(f, "cannot open the file: {detail}"),
            SkipReason::MetaUnreadable { detail } => {
                write!(
                    f,
                    "dict_meta is unreadable, not a dictionary file: {detail}"
                )
            }
            SkipReason::MetaRowMissing { key } => {
                write!(f, "dict_meta has no row for the key {key}")
            }
            SkipReason::SchemaTooNew {
                file_version,
                supported,
            } => write!(
                f,
                "schema version {file_version} is newer than the supported {supported}"
            ),
            SkipReason::SchemaTooOld {
                file_version,
                minimum,
            } => write!(
                f,
                // ⚠️ Câu này đi qua **hai** ranh giới của `core/dict/**`, và cả hai đã bắt
                // được nó ở đúng lượt cài hằng này — chép lại để lượt sửa sau không đạp lại:
                //   ① đuôi tệp từ điển bị cấm (`the_layer_set_never_hardcodes_a_db_filename`)
                //      — một tên tệp viết cứng ở tầng này là một sổ đăng ký, AD-44 ① vá A2;
                //   ② token `matching` bị cấm (`the_dictionary_lookup_path_never_calls_the_matcher`)
                //      — AD-17 thân Rule: đường tra cứu từ điển KHÔNG gọi Matcher.
                // Cả hai cổng đọc **tĩnh**, nên chúng đỏ trên một câu văn xuôi y như trên mã.
                "schema version {file_version} is older than the minimum {minimum} this build \
                 can read; regenerate the dictionary files with the current tools/dict-build"
            ),
            SkipReason::SchemaVersionDisagrees {
                user_version,
                meta_version,
            } => write!(
                f,
                "PRAGMA user_version {user_version} disagrees with dict_meta schema_version {meta_version}"
            ),
            SkipReason::SourcesUnreadable { detail } => {
                write!(f, "dict_source is unreadable: {detail}")
            }
            SkipReason::DuplicateLayer { layer } => {
                write!(f, "another file already declares the layer {layer}")
            }
            SkipReason::DuplicateSourceCode { code, first_layer } => write!(
                f,
                "the source code {code} is already declared by the layer {first_layer}"
            ),
            SkipReason::DuplicateSourceCodeInFile { code } => write!(
                f,
                "dict_source declares the code {code} more than once in this file"
            ),
            SkipReason::LookupFailed { detail } => write!(f, "the lookup failed: {detail}"),
        }
    }
}

impl SkipReason {
    /// 🔴 **Quyết định #2 (Story 1.17)** — mã máy đi qua IPC, không **thay cho** chính kiểu này.
    ///
    /// `SkipReason` **không bao giờ** `derive(Serialize)` — bốn biến thể của nó mang
    /// `detail: String` là **lỗi thô của SQLite**, và đi qua dây nguyên vẹn là vi phạm
    /// AD-21 ở đúng chỗ khó thấy nhất (`check-i18n.mjs` Kiểm A quét **chuỗi trong mã**, không
    /// không quét **dữ liệu chạy qua dây**). Panel Lookup chỉ cần biết *"một phần từ điển
    /// không trả lời"* và mã máy này để chẩn đoán — không cần biết tệp nào hỏng thế nào.
    pub(crate) fn wire_code(&self) -> &'static str {
        match self {
            SkipReason::OpenFailed { .. } => "open_failed",
            SkipReason::MetaUnreadable { .. } => "meta_unreadable",
            SkipReason::MetaRowMissing { .. } => "meta_row_missing",
            SkipReason::SchemaTooNew { .. } => "schema_too_new",
            SkipReason::SchemaTooOld { .. } => "schema_too_old",
            SkipReason::SchemaVersionDisagrees { .. } => "schema_version_disagrees",
            SkipReason::SourcesUnreadable { .. } => "sources_unreadable",
            SkipReason::DuplicateLayer { .. } => "duplicate_layer",
            SkipReason::DuplicateSourceCode { .. } => "duplicate_source_code",
            SkipReason::DuplicateSourceCodeInFile { .. } => "duplicate_source_code_in_file",
            SkipReason::LookupFailed { .. } => "lookup_failed",
        }
    }
}

/// Một lớp không nạp được — **đường dẫn + lý do**, cả hai là dữ liệu.
///
/// ⚠️ **Không** `derive(Serialize)` — xem [`SkipReason::wire_code`]. `GroupedLookup`
/// (Story 1.17) tự tay chuyển `Vec<SkippedLayer>` thành `Vec<&str>` mã máy khi đi qua dây,
/// không serialize kiểu này thẳng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedLayer {
    /// Tệp nào.
    pub path: PathBuf,
    /// Vì sao.
    pub reason: SkipReason,
}

/// **Một tệp `.db`**, mở chỉ đọc, đã biết danh tính lớp và các nguồn của mình.
///
/// 🔴 Đơn vị là **một tệp**, không bao giờ một **ngôn ngữ** — xem
/// [`crate::ports::dict_source`].
pub struct DictLayer {
    db: ReadOnlyDb,
    layer: String,
    sources: Vec<SourceInfo>,
}

impl DictLayer {
    /// Mở một tệp và đọc **danh tính của nó từ chính nó**.
    ///
    /// Thứ tự bốn phép kiểm không tuỳ tiện: phiên bản trước danh tính, vì một tệp của
    /// một lược đồ **chưa biết** thì mọi thứ đọc được từ nó đều là phỏng đoán.
    fn open(path: PathBuf) -> Result<DictLayer, SkipReason> {
        let db = ReadOnlyDb::open(path, StoreKind::Dict).map_err(|err| SkipReason::OpenFailed {
            detail: err.to_string(),
        })?;

        let meta = db
            .read(|conn| {
                // ⚠️ `PRAGMA user_version` trả một số **có dấu** ở tầng SQLite; đọc nó vào
                // `i64` rồi thu hẹp là cách duy nhất không im lặng đổi nghĩa một giá trị
                // âm thành một phiên bản hợp lệ.
                let user_version: i64 =
                    conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

                let layer = meta_row(conn, "layer")?;
                let schema_version = meta_row(conn, "schema_version")?;

                Ok((user_version, layer, schema_version))
            })
            .map_err(|err| SkipReason::MetaUnreadable {
                detail: err.to_string(),
            })?;

        let (raw_version, layer, schema_version) = meta;

        // Một giá trị âm hay lớn hơn `u32` không phải một phiên bản ứng dụng này biết;
        // nó đi cùng đường với *"quá mới"*, không đi đường *"chắc là 1"*.
        let file_version = u32::try_from(raw_version).unwrap_or(u32::MAX);
        if file_version > SUPPORTED_SCHEMA_VERSION {
            return Err(SkipReason::SchemaTooNew {
                file_version,
                supported: SUPPORTED_SCHEMA_VERSION,
            });
        }

        // 🔴 Vế CÒN LẠI, và nó phải đứng ngay đây chứ không ở chỗ đọc `dict_source`: mục
        // đích là từ chối **ở cửa**, trước khi bất kỳ câu `SELECT` nào gõ tên một cột mà tệp
        // cũ không có. Xem [`MINIMUM_SCHEMA_VERSION`] cho ca thật đã gặp.
        if file_version < MINIMUM_SCHEMA_VERSION {
            return Err(SkipReason::SchemaTooOld {
                file_version,
                minimum: MINIMUM_SCHEMA_VERSION,
            });
        }

        let Some(schema_version) = schema_version else {
            return Err(SkipReason::MetaRowMissing {
                key: "schema_version".to_owned(),
            });
        };
        if schema_version.parse::<u32>() != Ok(file_version) {
            return Err(SkipReason::SchemaVersionDisagrees {
                user_version: file_version,
                meta_version: schema_version,
            });
        }

        let Some(layer) = layer else {
            return Err(SkipReason::MetaRowMissing {
                key: "layer".to_owned(),
            });
        };

        let sources = db
            .read(|conn| {
                let mut stmt =
                    conn.prepare("SELECT code, display_name FROM dict_source ORDER BY code")?;
                let rows = stmt.query_map([], |row| {
                    Ok(SourceInfo {
                        code: row.get(0)?,
                        display_name: row.get(1)?,
                    })
                })?;
                rows.collect::<Result<Vec<_>, _>>()
            })
            .map_err(|err| SkipReason::SourcesUnreadable {
                detail: err.to_string(),
            })?;

        // `ORDER BY code` làm hai hàng trùng luôn kề nhau — một hàng trùng bên trong
        // CHÍNH tệp này là dữ liệu hỏng, đối xứng với `conflict_with` bắt ca trùng
        // **giữa** hai tệp.
        if let Some(pair) = sources.windows(2).find(|pair| pair[0].code == pair[1].code) {
            return Err(SkipReason::DuplicateSourceCodeInFile {
                code: pair[0].code.clone(),
            });
        }

        Ok(DictLayer { db, layer, sources })
    }

    /// Đường dẫn tệp `.db` của lớp này.
    pub fn path(&self) -> &Path {
        self.db.path()
    }

    /// Nguồn mang `code`, nếu tệp này có nó.
    pub(super) fn source(&self, code: &str) -> Option<&SourceInfo> {
        self.sources.iter().find(|source| source.code == code)
    }

    /// **Ghi công đầy đủ của mọi nguồn trong tệp này** — Story 1.19, AC7. Một truy vấn.
    ///
    /// 🔴 **Đọc LÚC GỌI, không giữ thường trực trong RAM** (§Quyết định #5a). [`Self::open`]
    /// cố ý vẫn chỉ `SELECT code, display_name`: giữ `license_text` của bảy nguồn (~215 KB
    /// đo được trên `dict-core.db` thật) sống suốt đời tiến trình để phục vụ một màn hình
    /// **hiếm khi mở** là một cái giá không ai xin. Ở đây ta còn không đọc nội dung — chỉ
    /// `length()` (xem [`SourceAttribution::license_text_len`]).
    ///
    /// ⚠️ `ORDER BY code` — cùng lý do và cùng câu với [`Self::open`]: hai hàng trùng nằm
    /// kề nhau, nên phép kiểm [`SkipReason::DuplicateSourceCodeInFile`] đứng lên được. Ở đây
    /// nó còn cho thứ tự **tất định** mà AC7 đòi.
    ///
    /// 🔴 Trả lỗi thay vì nuốt: một tệp mà `dict_source` đọc được lúc mở mà **không** đọc
    /// được lúc này là một dữ kiện thật *(tệp bị thay dưới chân tiến trình)*, và chỗ gọi
    /// ([`super::list_source_attributions`]) quyết định làm gì với nó — bỏ một lớp khỏi bảng
    /// ghi công là một quyết định của tầng gom, không của adapter.
    pub(super) fn attributions(&self) -> Result<Vec<SourceAttribution>, StoreError> {
        let layer = self.layer.clone();
        let is_base = self.layer == BASE_LAYER;
        self.db.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT code, display_name, license_kind, license_id, length(license_text), \
                 attribution, source_version, source_url, lang \
                 FROM dict_source ORDER BY code",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(SourceAttribution {
                    code: row.get(0)?,
                    display_name: row.get(1)?,
                    license_kind: row.get(2)?,
                    license_id: row.get(3)?,
                    // `length()` của SQLite trả `NULL` chỉ khi đối số `NULL`, mà cột này là
                    // `NOT NULL` — đọc vào `Option` rồi rơi về 0 chứ không `unwrap`: một tệp
                    // sửa tay không được quyền giết cả bảng ghi công.
                    license_text_len: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
                    attribution: row.get(5)?,
                    source_version: row.get(6)?,
                    source_url: row.get(7)?,
                    layer: layer.clone(),
                    is_base,
                    // ⚠️ `Option` rồi rơi về rỗng, cùng luật `license_text_len`: cột là
                    // `NOT NULL DEFAULT ''` trong lược đồ hôm nay, nhưng một tệp `.db` dựng
                    // bằng bản `dict-build` **cũ hơn** Story 1.19 sẽ không có cột này chút
                    // nào. Ca đó đã đỏ ở `SELECT` phía trên rồi (`no such column`), và bảng
                    // ghi công của lớp ấy bị bỏ kèm một dòng chẩn đoán — đúng luật
                    // `list_source_attributions`. Đây là vế phòng thủ cho một tệp có cột
                    // nhưng mang `NULL` vì đã bị sửa tay.
                    lang: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    /// Đóng pool kết nối của lớp. Idempotent.
    pub fn close(&self) {
        self.db.close();
    }
}

impl DictionarySource for DictLayer {
    fn layer(&self) -> &str {
        &self.layer
    }

    fn sources(&self) -> &[SourceInfo] {
        &self.sources
    }

    fn lookup(
        &self,
        query: &str,
        route: QueryRoute,
        branch: QueryBranch,
        limit: usize,
    ) -> Result<LookupResult, StoreError> {
        // 🔴 Đi qua `super::lookup_with_branch`, **KHÔNG** gọi thẳng `query::char_idx` /
        // `query::exact` / `query::fts_trigram`: điều kiện `≤ 2 ký tự` của `char_idx()` chỉ
        // là một `debug_assert!` — **vô tác dụng ở bản release**, nơi nó âm thầm cắt truy
        // vấn còn hai ký tự đầu thay vì báo lỗi. `deferred-work.md` nêu đích danh *"tầng
        // gom Story 1.13"* là ca sẽ cắn.
        //
        // 🔴 `branch` **nhận từ chỗ gọi**, **không** tính lại qua `pick_branch` ở đây:
        // tầng gom ([`super::lookup_grouped`]) tính nó **ĐÚNG MỘT LẦN** cho cả lượt tra và
        // truyền cùng giá trị xuống mọi lớp (Task 4.1) — hai tệp tính riêng thì chỉ còn
        // cách khớp nhau bằng một `debug_assert_eq!` vô tác dụng ở bản release. `limit`
        // (Story 1.17) đi theo cùng doctrine.
        self.db
            .read(|conn| super::lookup_with_branch(conn, query, route, branch, limit))
    }

    fn senses(&self, entry_ids: &[i64]) -> Result<Vec<SenseRecord>, StoreError> {
        self.db.read(|conn| senses::read_senses(conn, entry_ids))
    }

    fn count_by_source(
        &self,
        query: &str,
        route: QueryRoute,
        branch: QueryBranch,
    ) -> Result<Vec<(String, i64)>, StoreError> {
        // Cùng doctrine `lookup`: `route`/`branch` nhận từ chỗ gọi, không tính lại ở đây —
        // một phép đếm trên một nhánh khác lượt tra vừa chạy là phép đếm của câu hỏi khác.
        self.db
            .read(|conn| super::query::count_by_source(conn, query, route, branch))
    }

    fn han_viet(&self, chars: &[&str]) -> Result<Vec<HanVietHit>, StoreError> {
        self.db.read(|conn| han_viet::read_han_viet(conn, chars))
    }
}

impl fmt::Debug for DictLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DictLayer")
            .field("layer", &self.layer)
            .field("path", &self.path())
            .field("sources", &self.sources.len())
            .finish_non_exhaustive()
    }
}

/// Tập lớp của **một thư mục** — mọi tệp `*.db` trong đó, cộng danh sách bị bỏ qua.
///
/// 🔴 `Send + Sync` là điều kiện để nó vào `app.manage(…)` mà chỗ gọi không phải bọc
/// thêm `Mutex` — cùng lý do với [`ReadOnlyDb`] và `Store`.
#[derive(Debug)]
pub struct DictLayers {
    layers: Vec<DictLayer>,
    skipped: Vec<SkippedLayer>,
}

impl DictLayers {
    /// Quét `dir`, thử mở **mọi** tệp `*.db`, trả về tập lớp đã sắp tất định.
    ///
    /// 🔴 **Không bao giờ trả lỗi.** Thư mục không tồn tại, hoặc rỗng ⇒ **tập lớp
    /// RỖNG**. Đó không phải một sự khoan dung: `src-tauri/resources/dict/` hôm nay rỗng
    /// trong git (AD-25) và `bundle.resources` chưa mang nó (Story 10.1), nên *"không có
    /// lớp nào"* là một trạng thái **bình thường có tên** — và nó là chính hình dạng FR36
    /// đòi hỏi.
    ///
    /// ⚠️ `dir` **nhận từ chỗ gọi**; module này không tự phân giải `$RESOURCE` — đường đó
    /// sống ở `lib.rs`, đúng khuôn `$APPDATA` của `Store`.
    pub fn open(dir: &Path) -> DictLayers {
        let mut paths: Vec<PathBuf> = match fs::read_dir(dir) {
            Ok(entries) => entries
                .filter_map(|entry| match entry {
                    Ok(entry) => Some(entry.path()),
                    Err(err) => {
                        // Một `DirEntry` hỏng giữa chừng không được biến mất im lặng —
                        // nó khác hẳn *"thư mục rỗng"*, dù kết quả trả về vẫn phải là tập
                        // lớp rỗng-có-thể, không phải một lỗi hay panic (AC3).
                        eprintln!(
                            "dict[layers] cannot read a directory entry under {}: {err}",
                            dir.display()
                        );
                        None
                    }
                })
                .filter(|path| {
                    // ⚠️ So đuôi **không phân biệt hoa/thường**: Windows coi `X.DB` và
                    // `x.db` là một tệp, và một phép so phân biệt hoa/thường ở đây làm cùng
                    // một thư mục cho hai tập lớp trên hai nền tảng (NFR14).
                    path.extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("db"))
                })
                .collect(),
            // Thư mục không tồn tại là trạng thái BÌNH THƯỜNG có tên (AC3) — im lặng.
            // Mọi lỗi KHÁC (quyền truy cập, …) không được trông giống hệt nó: chúng đi
            // cùng một tập lớp rỗng, nhưng kèm một dòng chẩn đoán.
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(err) => {
                eprintln!("dict[layers] cannot scan {}: {err}", dir.display());
                Vec::new()
            }
        };

        // Sắp đường dẫn **trước** khi mở: thứ tự `read_dir` khác nhau giữa macOS và Windows
        // (NFR14), và nó quyết định lớp nào thắng khi hai lớp trùng `code`.
        paths.sort();

        let mut opened: Vec<DictLayer> = Vec::new();
        let mut skipped: Vec<SkippedLayer> = Vec::new();

        for path in paths {
            match DictLayer::open(path.clone()) {
                Ok(layer) => opened.push(layer),
                Err(reason) => skipped.push(SkippedLayer { path, reason }),
            }
        }

        // 🔴 Thứ tự lớp là một **GIÁ TRỊ quan sát được**, không phải thứ tự thư mục:
        // `base` trước, rồi mã lớp tăng dần, rồi đường dẫn để hai lớp cùng mã vẫn tất định.
        opened.sort_by(|a, b| order_key(a).cmp(&order_key(b)));

        let mut layers: Vec<DictLayer> = Vec::new();
        for layer in opened {
            if let Some(reason) = conflict_with(&layers, &layer) {
                skipped.push(SkippedLayer {
                    path: layer.path().to_path_buf(),
                    reason,
                });
                // ⚠️ Đóng ngay: một lớp bị từ chối không được giữ một tệp mở, vì trên
                // Windows một tệp còn mở là một bản cập nhật không thay được tệp đó
                // (NFR14, FR112).
                layer.close();
                continue;
            }
            layers.push(layer);
        }

        // Danh sách bỏ qua sắp theo đường dẫn: một danh sách chẩn đoán mà thứ tự đổi giữa
        // hai lượt chạy là một danh sách không so sánh được trong test.
        skipped.sort_by(|a, b| a.path.cmp(&b.path));

        DictLayers { layers, skipped }
    }

    /// Tập lớp **rỗng theo tên**, không quét gì cả.
    ///
    /// 🔴 Dùng khi chỗ gọi **đã biết chắc** không có gì để quét — ví dụ `$RESOURCE` của
    /// chính Tauri không phân giải được. Cùng bất biến với [`Self::open`] trên một thư mục
    /// không tồn tại: *"không có lớp nào"* luôn phải là một trạng thái **quản lý
    /// được**, không phải một `app.manage` bị bỏ qua.
    pub fn empty() -> DictLayers {
        DictLayers {
            layers: Vec::new(),
            skipped: Vec::new(),
        }
    }

    /// Các lớp đã nạp, theo **thứ tự tất định** của AC3.
    pub fn layers(&self) -> &[DictLayer] {
        &self.layers
    }

    /// Các lớp không nạp được, mỗi mục mang **đường dẫn + lý do**.
    pub fn skipped(&self) -> &[SkippedLayer] {
        &self.skipped
    }

    /// Lớp mang danh tính `layer`, nếu có.
    ///
    /// 🔴 Đây là đường **pha hai** (§Quyết định #1B): [`crate::core::dict::SourceGroup`]
    /// mang `layer`, nên chỗ gọi cầm một nhóm là cầm đủ thứ để hỏi tiếp nghĩa của nó. Trả
    /// [`Option`] chứ không một danh sách rỗng: *"lớp đó không có ở đây"* và *"lớp đó
    /// không có nghĩa nào"* là hai câu khác nhau.
    pub fn layer(&self, layer: &str) -> Option<&DictLayer> {
        self.layers.iter().find(|found| found.layer == layer)
    }

    /// Đóng **mọi** lớp. Idempotent.
    ///
    /// 🔴 Gọi ở `RunEvent::Exit` — NFR14 và FR112: một tệp từ điển còn mở trên Windows là
    /// một bản cập nhật không thay được tệp đó, và chính sách gỡ bỏ dữ liệu đứng trên
    /// đúng khả năng xoá được tệp.
    pub fn close(&self) {
        for layer in &self.layers {
            layer.close();
        }
    }
}

/// Một hàng của `dict_meta`, hoặc [`None`] khi khoá vắng mặt.
///
/// 🔴 *"Khoá vắng mặt"* **không** phải một lỗi đọc, và trộn hai thứ đó là làm một tệp
/// thiếu hàng `layer` đọc giống hệt một tệp không phải database. Hai ca đó có hai lý do
/// bỏ qua riêng ([`SkipReason::MetaRowMissing`] · [`SkipReason::MetaUnreadable`]) chính vì
/// người đọc chẩn đoán phải phân biệt được chúng.
fn meta_row(conn: ReadHandle<'_>, key: &str) -> SqlResult<Option<String>> {
    let mut stmt = conn.prepare_cached("SELECT value FROM dict_meta WHERE key = ?1")?;
    stmt.query_row([key], |row| row.get::<_, String>(0))
        .map(Some)
        .or_else(|err| match err {
            SqlError::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })
}

/// Khoá sắp xếp của AC3 — `base` trước *(`false < true`)*, rồi mã lớp, rồi đường dẫn.
fn order_key(layer: &DictLayer) -> (bool, &str, &Path) {
    (layer.layer != BASE_LAYER, &layer.layer, layer.path())
}

/// Lớp mới có va vào một lớp đã nhận không — theo **danh tính** hoặc theo **mã nguồn**.
fn conflict_with(accepted: &[DictLayer], candidate: &DictLayer) -> Option<SkipReason> {
    if accepted.iter().any(|layer| layer.layer == candidate.layer) {
        return Some(SkipReason::DuplicateLayer {
            layer: candidate.layer.clone(),
        });
    }

    for source in &candidate.sources {
        if let Some(owner) = accepted
            .iter()
            .find(|layer| layer.source(&source.code).is_some())
        {
            return Some(SkipReason::DuplicateSourceCode {
                code: source.code.clone(),
                first_layer: owner.layer.clone(),
            });
        }
    }

    None
}
