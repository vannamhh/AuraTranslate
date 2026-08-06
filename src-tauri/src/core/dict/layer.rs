//! Một **tệp `.db` = một lớp** (AD-10), và tập lớp phát hiện bằng **QUÉT THƯ MỤC**.
//!
//! ⛔ **Tệp này ⛔ không bao giờ gọi vị từ điều phối** — `route` đi xuống từ tầng gom như
//! một tham số (AD-44 ①). `tests/dict_boundary.rs` cưỡng chế điều đó bằng máy, đếm **tệp**.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO ⛔ KHÔNG TỒN TẠI MỘT SỔ ĐĂNG KÝ
//! ─────────────────────────────────────────────────────────────────────────────
//! AD-44 ① vá A2: *"⛔ **Không tồn tại sổ đăng ký "tệp `.db` nào chứa ngôn ngữ nào"**. Một
//! sổ như thế là nguồn sự thật thứ hai cho một dữ kiện đã nằm trong dữ liệu […] và nó sai
//! **im lặng** vào đúng ngày một lớp gỡ rời được thêm hay gỡ đi (FR112)."*
//!
//! Luật đó viết cho **ngôn ngữ**; module này áp nó cho **danh tính lớp** vì cùng một lý do,
//! và vì FR36 nói *"gỡ một lớp = xoá một file"* — một danh sách tên tệp viết cứng trong mã
//! làm mệnh đề đó thành **sai**. Nên:
//!
//! - Tập lớp = **mọi** tệp `*.db` trong một thư mục, ⛔ không một danh sách tên nào.
//! - Danh tính lớp đọc từ **`dict_meta('layer')` của chính tệp**, ⛔ không từ tên tệp.
//! - Nguồn đọc từ **`dict_source` của chính tệp**, ⛔ không từ một bảng tra ở tầng gom.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ CHÍNH SÁCH PHIÊN BẢN SỐNG Ở ĐÂY, ⛔ KHÔNG Ở `ReadOnlyDb`
//! ─────────────────────────────────────────────────────────────────────────────
//! `core/store/readonly.rs:57-60` giao thẳng: *"⛔ **Không đọc `PRAGMA user_version`, ⛔
//! không di trú, ⛔ không kiểm phiên bản lược đồ ở đây.** Việc từ chối một tệp mới hơn ứng
//! dụng là quyết định của **tầng gọi (Story 1.13**, nơi biết mình đang mở *lớp* nào và làm
//! gì khi một lớp bị từ chối)"*. Đẩy phép kiểm ngược vào `ReadOnlyDb` là **chôn một chính
//! sách vào một cơ chế**.
//!
//! ⚠️ Mọi chuỗi chẩn đoán ở tệp này viết **KHÔNG DẤU** — `scripts/check-i18n.mjs` Kiểm A
//! quét `src-tauri/**/*.rs` và tệp này ⛔ không nằm trong danh sách miễn trừ. Comment thì
//! được, **chuỗi thì không**.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::store::{ReadHandle, ReadOnlyDb, SqlError, SqlResult, StoreError, StoreKind};
use crate::ports::DictionarySource;

use super::{HanVietHit, LookupResult, QueryBranch, QueryRoute, SenseRecord, SourceInfo, han_viet, senses};

/// Phiên bản lược đồ tệp `.db` mà đường đọc này hiểu.
///
/// 🔴 Phải **bằng** `tools/dict-build/src/schema.rs::SCHEMA_VERSION`. Hai workspace tách
/// rời **có chủ ý** (AC4 của Story 1.9) nên ⛔ không có import chéo nào giữ hai hằng dính
/// nhau — `tests/dict_sources.rs::the_supported_schema_version_matches_dict_build` đọc tệp
/// kia **dưới dạng văn bản** và canh đúng mệnh đề đó.
///
/// 🔴 1 → 2 ở Story 1.10c, CÙNG LƯỢT với `tools/dict-build`: cột `dict_entry.nom_reading`
/// mới (AC6). Một tệp `.db` **v2** phải mở được; một tệp `.db` **v3** giả lập vẫn bị từ
/// chối bằng `SkipReason::SchemaTooNew` (AD-30 — mở tiến, ⛔ không mở lùi).
pub const SUPPORTED_SCHEMA_VERSION: u32 = 2;

/// Danh tính của lớp **nền**. Mọi giá trị khác là một lớp **gỡ rời**.
///
/// ⚠️ Đây ⛔ **không** phải một mã nguồn (`dict_source.code`) — nó là giá trị của
/// `dict_meta('layer')`, do `tools/dict-build/src/insert.rs:140` ghi vào từng tệp.
///
/// 🔴 `pub(super)` chứ ⛔ không `private`: tầng gom (`mod.rs::priority_order`) cần **chính
/// hằng này**, ⛔ không một bản chép thứ hai — xem `mod.rs::BASE_LAYER_NAME`.
pub(super) const BASE_LAYER: &str = "base";

/// Vì sao một tệp trong thư mục ⛔ **không** trở thành một lớp.
///
/// 🔴 Một **GIÁ TRỊ**, ⛔ không phải một dòng `eprintln!`: *"Rỗng im lặng bị cấm; rỗng có
/// lý do thì không"* (AD-44 ④). Panel Lookup (1.17) phải phân biệt được *"đã tra mà ⛔
/// không khớp"* với *"lớp ⛔ không nạp được"* — hai câu đó dẫn người dùng đi hai đường khác
/// nhau, và chúng chỉ phân biệt được nếu lý do đi ra theo **kết quả**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// ⛔ Không mở được tệp.
    OpenFailed {
        /// Lỗi thô, chỉ để chẩn đoán. ⛔ Không đi lên giao diện.
        detail: String,
    },

    /// Mở được, nhưng ⛔ không đọc nổi `dict_meta` — tệp ⛔ không mang lược đồ từ điển.
    MetaUnreadable {
        /// Lỗi thô, chỉ để chẩn đoán.
        detail: String,
    },

    /// `dict_meta` có, nhưng thiếu một hàng bắt buộc (`layer` hoặc `schema_version`).
    MetaRowMissing {
        /// Khoá vắng mặt — **dữ liệu**, ⛔ không phải một câu.
        key: String,
    },

    /// 🔴 Tệp **mới hơn** ứng dụng. ⛔ Không đoán, ⛔ không di trú — từ chối có tên.
    SchemaTooNew {
        /// `PRAGMA user_version` đọc được từ tệp.
        file_version: u32,
        /// [`SUPPORTED_SCHEMA_VERSION`].
        supported: u32,
    },

    /// 🔴 **Hai chỗ ghi phiên bản NÓI KHÁC NHAU.**
    ///
    /// Story 1.9 §Quyết định #2 ghi cả `PRAGMA user_version` lẫn
    /// `dict_meta('schema_version')` **có chủ ý**. Hai chỗ đó lệch nghĩa là tệp ⛔ **không**
    /// do `tools/dict-build` viết ra — và tin nửa nào cũng là **đoán**.
    SchemaVersionDisagrees {
        /// `PRAGMA user_version`.
        user_version: u32,
        /// `dict_meta('schema_version')` — nguyên văn, kể cả khi ⛔ không phải một số.
        meta_version: String,
    },

    /// ⛔ Không đọc nổi `dict_source` của tệp.
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
    /// ⛔ **Không** im lặng gộp hai tệp vào một nhóm: khoá gom là `code`, nên hai tệp cùng
    /// `code` làm một nhóm mang nghĩa của hai nguồn khác nhau — đúng thứ AD-19 cấm, xảy ra
    /// ở tầng dữ liệu thay vì ở tầng mã.
    DuplicateSourceCode {
        /// Mã bị trùng.
        code: String,
        /// Lớp đã giữ mã đó trước, theo thứ tự tất định.
        first_layer: String,
    },

    /// 🔴 **CHÍNH tệp này** khai hai hàng `dict_source` cùng một `code` — lỗi dữ liệu ngay
    /// trong nó, ⛔ không phải một va chạm giữa hai tệp.
    ///
    /// `source()` chỉ có thể trả **một** [`SourceInfo`] cho một `code`; im lặng giữ hàng
    /// đầu và bỏ hàng sau là giấu một dữ kiện thay vì báo nó — đối xứng với
    /// [`SkipReason::DuplicateSourceCode`], vốn đã bắt đúng ca này khi nó xảy ra **giữa**
    /// hai lớp.
    DuplicateSourceCodeInFile {
        /// Mã bị trùng.
        code: String,
    },

    /// Lớp nạp được nhưng **một lượt tra trên nó** hỏng. ⛔ Không làm hỏng cả lượt tra.
    LookupFailed {
        /// Lỗi thô, chỉ để chẩn đoán.
        detail: String,
    },
}

impl fmt::Display for SkipReason {
    /// ⚠️ **KHÔNG DẤU** — xem doc-comment module. Đây là chuỗi chẩn đoán cho người đang
    /// đọc stderr, ⛔ không phải một câu cho người dùng (câu là việc của `core/i18n`).
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

/// Một lớp ⛔ không nạp được — **đường dẫn + lý do**, cả hai là dữ liệu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedLayer {
    /// Tệp nào.
    pub path: PathBuf,
    /// Vì sao.
    pub reason: SkipReason,
}

/// **Một tệp `.db`**, mở chỉ đọc, đã biết danh tính lớp và các nguồn của mình.
///
/// 🔴 Đơn vị là **một tệp**, ⛔ không bao giờ một **ngôn ngữ** — xem
/// [`crate::ports::dict_source`].
pub struct DictLayer {
    db: ReadOnlyDb,
    layer: String,
    sources: Vec<SourceInfo>,
}

impl DictLayer {
    /// Mở một tệp và đọc **danh tính của nó từ chính nó**.
    ///
    /// Thứ tự bốn phép kiểm ⛔ không tuỳ tiện: phiên bản trước danh tính, vì một tệp của
    /// một lược đồ **chưa biết** thì mọi thứ đọc được từ nó đều là phỏng đoán.
    fn open(path: PathBuf) -> Result<DictLayer, SkipReason> {
        let db = ReadOnlyDb::open(path, StoreKind::Dict).map_err(|err| SkipReason::OpenFailed {
            detail: err.to_string(),
        })?;

        let meta = db
            .read(|conn| {
                // ⚠️ `PRAGMA user_version` trả một số **có dấu** ở tầng SQLite; đọc nó vào
                // `i64` rồi thu hẹp là cách duy nhất ⛔ không im lặng đổi nghĩa một giá trị
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

        // Một giá trị âm hay lớn hơn `u32` ⛔ không phải một phiên bản ứng dụng này biết;
        // nó đi cùng đường với *"quá mới"*, ⛔ không đi đường *"chắc là 1"*.
        let file_version = u32::try_from(raw_version).unwrap_or(u32::MAX);
        if file_version > SUPPORTED_SCHEMA_VERSION {
            return Err(SkipReason::SchemaTooNew {
                file_version,
                supported: SUPPORTED_SCHEMA_VERSION,
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
    ) -> Result<LookupResult, StoreError> {
        // 🔴 Đi qua `super::lookup_with_branch`, ⛔ **KHÔNG** gọi thẳng `query::char_idx` /
        // `query::exact` / `query::fts_trigram`: điều kiện `≤ 2 ký tự` của `char_idx()` chỉ
        // là một `debug_assert!` — **vô tác dụng ở bản release**, nơi nó âm thầm cắt truy
        // vấn còn hai ký tự đầu thay vì báo lỗi. `deferred-work.md` nêu đích danh *"tầng
        // gom Story 1.13"* là ca sẽ cắn.
        //
        // 🔴 `branch` **nhận từ chỗ gọi**, ⛔ **không** tính lại qua `pick_branch` ở đây:
        // tầng gom ([`super::lookup_grouped`]) tính nó **ĐÚNG MỘT LẦN** cho cả lượt tra và
        // truyền cùng giá trị xuống mọi lớp (Task 4.1) — hai tệp tính riêng thì chỉ còn
        // cách khớp nhau bằng một `debug_assert_eq!` vô tác dụng ở bản release.
        self.db
            .read(|conn| super::lookup_with_branch(conn, query, route, branch))
    }

    fn senses(&self, entry_ids: &[i64]) -> Result<Vec<SenseRecord>, StoreError> {
        self.db.read(|conn| senses::read_senses(conn, entry_ids))
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
/// 🔴 `Send + Sync` là điều kiện để nó vào `app.manage(…)` mà chỗ gọi ⛔ không phải bọc
/// thêm `Mutex` — cùng lý do với [`ReadOnlyDb`] và `Store`.
#[derive(Debug)]
pub struct DictLayers {
    layers: Vec<DictLayer>,
    skipped: Vec<SkippedLayer>,
}

impl DictLayers {
    /// Quét `dir`, thử mở **mọi** tệp `*.db`, trả về tập lớp đã sắp tất định.
    ///
    /// 🔴 **⛔ Không bao giờ trả lỗi.** Thư mục ⛔ không tồn tại, hoặc rỗng ⇒ **tập lớp
    /// RỖNG**. Đó ⛔ không phải một sự khoan dung: `src-tauri/resources/dict/` hôm nay rỗng
    /// trong git (AD-25) và `bundle.resources` chưa mang nó (Story 10.1), nên *"⛔ không có
    /// lớp nào"* là một trạng thái **bình thường có tên** — và nó là chính hình dạng FR36
    /// đòi hỏi.
    ///
    /// ⚠️ `dir` **nhận từ chỗ gọi**; module này ⛔ không tự phân giải `$RESOURCE` — đường đó
    /// sống ở `lib.rs`, đúng khuôn `$APPDATA` của `Store`.
    pub fn open(dir: &Path) -> DictLayers {
        let mut paths: Vec<PathBuf> = match fs::read_dir(dir) {
            Ok(entries) => entries
                .filter_map(|entry| match entry {
                    Ok(entry) => Some(entry.path()),
                    Err(err) => {
                        // Một `DirEntry` hỏng giữa chừng ⛔ không được biến mất im lặng —
                        // nó khác hẳn *"thư mục rỗng"*, dù kết quả trả về vẫn phải là tập
                        // lớp rỗng-có-thể, ⛔ không phải một lỗi hay panic (AC3).
                        eprintln!(
                            "dict[layers] cannot read a directory entry under {}: {err}",
                            dir.display()
                        );
                        None
                    }
                })
                .filter(|path| {
                    // ⚠️ So đuôi **⛔ không phân biệt hoa/thường**: Windows coi `X.DB` và
                    // `x.db` là một tệp, và một phép so phân biệt hoa/thường ở đây làm cùng
                    // một thư mục cho hai tập lớp trên hai nền tảng (NFR14).
                    path.extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("db"))
                })
                .collect(),
            // Thư mục ⛔ không tồn tại là trạng thái BÌNH THƯỜNG có tên (AC3) — im lặng.
            // Mọi lỗi KHÁC (quyền truy cập, …) ⛔ không được trông giống hệt nó: chúng đi
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

        // 🔴 Thứ tự lớp là một **GIÁ TRỊ quan sát được**, ⛔ không phải thứ tự thư mục:
        // `base` trước, rồi mã lớp tăng dần, rồi đường dẫn để hai lớp cùng mã vẫn tất định.
        opened.sort_by(|a, b| order_key(a).cmp(&order_key(b)));

        let mut layers: Vec<DictLayer> = Vec::new();
        for layer in opened {
            if let Some(reason) = conflict_with(&layers, &layer) {
                skipped.push(SkippedLayer {
                    path: layer.path().to_path_buf(),
                    reason,
                });
                // ⚠️ Đóng ngay: một lớp bị từ chối ⛔ không được giữ một tệp mở, vì trên
                // Windows một tệp còn mở là một bản cập nhật ⛔ không thay được tệp đó
                // (NFR14, FR112).
                layer.close();
                continue;
            }
            layers.push(layer);
        }

        // Danh sách bỏ qua sắp theo đường dẫn: một danh sách chẩn đoán mà thứ tự đổi giữa
        // hai lượt chạy là một danh sách ⛔ không so sánh được trong test.
        skipped.sort_by(|a, b| a.path.cmp(&b.path));

        DictLayers { layers, skipped }
    }

    /// Tập lớp **rỗng theo tên**, ⛔ không quét gì cả.
    ///
    /// 🔴 Dùng khi chỗ gọi **đã biết chắc** ⛔ không có gì để quét — ví dụ `$RESOURCE` của
    /// chính Tauri không phân giải được. Cùng bất biến với [`Self::open`] trên một thư mục
    /// ⛔ không tồn tại: *"⛔ không có lớp nào"* luôn phải là một trạng thái **quản lý
    /// được**, ⛔ không phải một `app.manage` bị bỏ qua.
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

    /// Các lớp ⛔ không nạp được, mỗi mục mang **đường dẫn + lý do**.
    pub fn skipped(&self) -> &[SkippedLayer] {
        &self.skipped
    }

    /// Lớp mang danh tính `layer`, nếu có.
    ///
    /// 🔴 Đây là đường **pha hai** (§Quyết định #1B): [`crate::core::dict::SourceGroup`]
    /// mang `layer`, nên chỗ gọi cầm một nhóm là cầm đủ thứ để hỏi tiếp nghĩa của nó. Trả
    /// [`Option`] chứ ⛔ không một danh sách rỗng: *"lớp đó ⛔ không có ở đây"* và *"lớp đó
    /// ⛔ không có nghĩa nào"* là hai câu khác nhau.
    pub fn layer(&self, layer: &str) -> Option<&DictLayer> {
        self.layers.iter().find(|found| found.layer == layer)
    }

    /// Đóng **mọi** lớp. Idempotent.
    ///
    /// 🔴 Gọi ở `RunEvent::Exit` — NFR14 và FR112: một tệp từ điển còn mở trên Windows là
    /// một bản cập nhật ⛔ không thay được tệp đó, và chính sách gỡ bỏ dữ liệu đứng trên
    /// đúng khả năng xoá được tệp.
    pub fn close(&self) {
        for layer in &self.layers {
            layer.close();
        }
    }
}

/// Một hàng của `dict_meta`, hoặc [`None`] khi khoá vắng mặt.
///
/// 🔴 *"Khoá vắng mặt"* ⛔ **không** phải một lỗi đọc, và trộn hai thứ đó là làm một tệp
/// thiếu hàng `layer` đọc giống hệt một tệp ⛔ không phải database. Hai ca đó có hai lý do
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

/// Lớp mới có va vào một lớp đã nhận ⛔ không — theo **danh tính** hoặc theo **mã nguồn**.
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
