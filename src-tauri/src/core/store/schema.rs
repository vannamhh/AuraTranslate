//! Phiên bản lược đồ, từ chối mở lùi, di trú **chỉ tiến** — AD-30, AC6, AC7.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! QUY ƯỚC PHIÊN BẢN, KHAI TƯỜNG MINH
//! ─────────────────────────────────────────────────────────────────────────────
//! Phiên bản nằm ở `PRAGMA user_version`, mặc định là **0** — nên *"database mới tinh"*
//! và *"database ở phiên bản 0"* **không phân biệt được**. Quy ước vì thế phải nói ra
//! thay vì để mỗi người đọc tự suy:
//!
//! - **0 = chưa có lược đồ.** Không có gì để sao lưu, và không có gì mất khi di trú.
//! - Bước di trú đầu tiên đánh số **1**.
//! - `to_version` tăng dần nghiêm ngặt. Không có bước lùi, không có bước
//!   *"sửa cho vừa"* — một bước như vậy là hai đường lược đồ khác nhau cho cùng một số,
//!   và chúng sẽ rẽ nhau ở máy người dùng chứ không ở đây.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SAO LƯU BẰNG `fs::copy` TỆP `.db` TRẦN LÀ MỘT BẢN SAO **KHÔNG ĐẦY ĐỦ**
//! ─────────────────────────────────────────────────────────────────────────────
//! Khi WAL đang bật, dữ liệu **đã commit nhưng chưa checkpoint** sống trong `.db-wal`,
//! không trong `.db`. Copy mình tệp `.db` cho ra một bản sao **thiếu đúng những thay đổi
//! gần nhất** — và bản sao đó trông hoàn toàn hợp lệ: mở được, không lỗi, chỉ thiếu.
//! Đây là bản sao lưu mà AC6 dựa vào để **cho phép** di trú, nên nó hỏng ở đúng chỗ đắt
//! nhất: chỗ người dùng tin là mình có đường lui.
//!
//! → [`backup_before_migration`]: `wal_checkpoint(TRUNCATE)` → **xác nhận `busy == 0`** →
//!   rồi mới `fs::copy`.
//!
//! ⚠️ Feature `backup` của `rusqlite` đang **TẮT** (`Cargo.toml:75`) ⇒
//! `Connection::backup` **không tồn tại**. Bật nó là thêm bề mặt API mới vào một crate
//! đã ghim — ngoài phạm vi story này, và `check-deps.mjs` sẽ đỏ.

use std::path::Path;

use rusqlite::Connection;

use super::{StoreError, StoreKind, pragmas};

/// Lược đồ của bảng nhật ký di trú — **bước 1 của `global.db`**.
///
/// Vì sao một bảng nhật ký chứ không phải một bảng nghiệp vụ: `global.db` **chưa có**
/// nghiệp vụ nào ở story này (cấu hình là Story 1.8, phím tắt là 1.21). Nhưng AC6 nói
/// *"chạy các bước di trú chỉ tiến trong một giao dịch, sau khi đã sao lưu"*, và **không
/// có bước nào thì AC6 không có gì để nghiệm thu trên đường sản phẩm** — chỉ nghiệm thu
/// được bằng một bộ di trú giả trong test, tức lại đúng hình dạng *"mệnh đề vòng"* mà
/// lượt review Story 1.5 đã bắt (`deferred-work.md:38`).
///
/// - `applied_at` lấy bằng `strftime` **của chính SQLite** — ISO-8601 UTC theo
///   Consistency Conventions, và không phải thêm `chrono`/`time` cho một dòng.
/// - `app_version` lấy từ `env!("CARGO_PKG_VERSION")`.
/// - Bản ghi được chèn **trong cùng giao dịch** với bước sinh ra nó. Ghi ngoài giao dịch
///   là mở đúng ca *"sổ nói đã chạy mà lược đồ thì chưa"*.
pub const SCHEMA_MIGRATION_LOG_DDL: &str = "\
CREATE TABLE schema_migration_log (
  version     INTEGER PRIMARY KEY,
  applied_at  TEXT NOT NULL,
  app_version TEXT NOT NULL
);";

/// Một bước di trú. `sql` chạy **trọn trong một giao dịch**; trả `Err` ⇒ rollback.
///
/// ⚠️ `sql` là `&'static str` có chủ ý: một bước di trú là **hằng của bản ứng dụng**, đọc
/// được cạnh mã, và không bao giờ được sinh ra lúc chạy. Một bước di trú dựng bằng
/// `format!` từ trạng thái là một lược đồ khác nhau trên mỗi máy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Migration {
    /// Phiên bản mà database đạt được **sau khi** bước này commit. Tăng dần nghiêm ngặt.
    pub to_version: u32,
    /// Câu (hoặc nhiều câu) SQL của bước.
    pub sql: &'static str,
}

/// Lược đồ bảng cấu hình khoá-giá trị — **bước 2 của `global.db`**, Story 1.8 AC5.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT BẢNG, KHÔNG PHẢI BA — và không phải một bảng cho MỌI loại
/// ─────────────────────────────────────────────────────────────────────────────
/// Hai cám dỗ đối nghịch, cả hai đều sai:
///
/// - **Ba bảng** (`keybinding` + `layout_preset` + `app_config`) là dựng lược đồ cho hai
///   tính năng chưa tồn tại (Story 1.14, Story 1.21). Quy tắc đã khoá ngay trên đây:
///   *mỗi story sở hữu bước di trú của chính nó, cùng lúc với bảng mà nó cần*.
/// - **Một bảng cho tất cả** — tức cả Glossary, TM, Prompt và luật làm sạch cùng nhét vào
///   cột `value TEXT` — là dựng một lược đồ EAV mà bốn epic sau phải bóc ra: Glossary có
///   phân loại/xuất xứ/vòng đời ba trạng thái (Story 3.1), TM có cặp văn bản + xuất xứ
///   (AD-6), luật làm sạch có mẫu regex + cờ bật tắt (Story 6.5).
///
/// **Chốt:** bảng này phục vụ **riêng** ba loại `Semantics::GlobalOnly` của
/// `core::scope::ScopeKind` — `shortcut`, `layout_preset`, `app_config`. Mỗi module miền
/// mang bảng riêng của nó, ở epic của nó.
///
/// ⚠️ Cột `kind` là chuỗi chứ không phải một `CHECK` liệt kê ba giá trị: một `CHECK` biến
/// mọi loại `GlobalOnly` mới thành một bước di trú, trong khi phép cưỡng chế thật đã nằm
/// ở `ScopeKind` phía Rust — nơi trình biên dịch làm việc đó (AC4).
///
/// ⚠️ Không cột `tier`. Bảng này **là** tầng Global; một cột tầng ở đây là mời người
/// sau ghi một hàng `tier = 'work'` vào `global.db`, tức đúng thứ
/// `ScopeError::WorkTierForbidden` tồn tại để từ chối.
pub const CONFIG_VALUE_DDL: &str = "\
CREATE TABLE config_value (
  kind       TEXT NOT NULL,
  key        TEXT NOT NULL,
  value      TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (kind, key)
);";

/// Lược đồ bảng `pinned_entry` — **bước 3 của `global.db`**, Story 1.20, AC2 · AC3.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 PHẠM VI **TOÀN ỨNG DỤNG**, KHÔNG THEO TÁC PHẨM — Ice ký lại 2026-08-11
/// ─────────────────────────────────────────────────────────────────────────────
/// Ngày 2026-08-10 Quyết định #1 chốt `project.db` (phạm vi Tác phẩm), dựa trên hai câu
/// trong mockup và một lý lẽ ngữ nghĩa. Một phép đo ngày hôm sau lật nó, và phép đo đó là
/// thứ bảng so sánh của story **không có hàng nào cho**:
///
/// **Hôm nay không tồn tại đường mở lại một `.atproj` từ đĩa.** `OpenWorkState` khởi động
/// với `None` và **chỉ** `create_work_*` đặt được giá trị vào đó — 11 command IPC, không
/// cái nào đọc một Tác phẩm có sẵn (`commands/chapter.rs` ghi mệnh đề này bằng chữ).
/// ⇒ Với ghim ở `project.db`, đóng app rồi mở lại là **không Tác phẩm nào đang mở**, nên
/// bộ ghim không có đường nào để đọc tới. **AC3** — *"đóng rồi mở lại ứng dụng, mục ghim
/// vẫn còn"* — đúng trên đĩa mà **không bao giờ đúng trên màn hình**, cho tới Epic 5.
///
/// `global.db` mở **một lần** ở `setup()` và sống suốt vòng đời tiến trình, nên nó là chỗ
/// duy nhất AC3 có nghĩa được hôm nay.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT BẢNG RIÊNG, KHÔNG MỘT KHOÁ `config_value` — Quyết định #2, VẪN ĐỨNG
/// ─────────────────────────────────────────────────────────────────────────────
/// Lượt đổi phạm vi ở trên **không** kéo theo lượt đổi hình dạng, và lý do vẫn nguyên vẹn:
///
/// - `KEY_DICT_DISABLED` chở **một tập mã ngắn** (`"cvdict,thieuchuu"`) — một chuỗi phẳng.
/// - Một mục ghim chở **nhiều trường có cấu trúc**: nguồn, `entry_id`, đầu mục, nghĩa rút
///   gọn để hiện lại mà không phải tra lại, thời điểm ghim.
/// - [`CONFIG_VALUE_DDL`] là `(kind, key) → TEXT`. Nhồi một danh sách bản ghi vào đó là
///   dựng lại đúng lược đồ EAV mà doc-comment của nó cấm bằng chữ — và nay `config_value`
///   **có mặt** trong cùng kho, nên cám dỗ đó là thật chứ không còn lý thuyết.
/// - Bảng này **không** phục vụ một `ScopeKind` nào: `CONFIG_VALUE_DDL` dành riêng cho ba
///   loại `Semantics::GlobalOnly`, còn mục ghim là dữ liệu miền. **0** `ScopeKind` mới.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `UNIQUE (source_code, entry_id)` LÀ HỢP ĐỒNG Ở TẦNG LƯỢC ĐỒ
/// ─────────────────────────────────────────────────────────────────────────────
/// *"Ghim hai lần cùng một mục không sinh hai hàng"* được **SQLite** cưỡng chế, không một
/// lượt `SELECT` trước `INSERT` ở tầng ứng dụng mà hai luồng có thể chen vào giữa — cùng
/// doctrine `CHECK (id = 1)` của [`WORK_DDL`].
///
/// `AUTOINCREMENT` cùng lý do [`CHAPTER_DDL`]: AD-3 nói id đã về hưu không bao giờ được
/// phát lại, và `INTEGER PRIMARY KEY` trần tái dùng rowid lớn nhất vừa xoá.
///
/// `gloss` để `NULL` được: một mục ghim từ một lượt tra không có nghĩa nào lấy về vẫn ghim
/// được. `headword` và `gloss` là **ảnh chụp** để hiện lại hàng mà không phải tra lại —
/// chấp nhận rằng chúng cũ đi nếu tệp `.db` nguồn được thay ở một bản phát hành sau.
///
/// **Không** cột `lookup_count`. Một số đếm bền vững cần một lượt ghi đĩa **mỗi lượt tra**,
/// tức đưa một `Store::write` vào đường nóng của Auto-Lookup và cho nó cạnh tranh hàng đợi
/// ghi nối tiếp với auto-save Editor (NFR2, AD-11/AD-12). Không AC nào đòi số đếm sống qua
/// phiên — AC3 chỉ đòi **mục ghim** còn.
pub const PINNED_ENTRY_DDL: &str = "\
CREATE TABLE pinned_entry (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  source_code TEXT NOT NULL,
  entry_id    INTEGER NOT NULL,
  headword    TEXT NOT NULL,
  gloss       TEXT,
  pinned_at   TEXT NOT NULL,
  UNIQUE (source_code, entry_id)
);";

/// Lược đồ bảng `glossary_entry` — **bước 4 của `global.db`, bước 12 của `project.db`** —
/// Story 3.1, AD-18 · AD-36 · FR46/FR47/FR114.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT HẰNG, DÙNG CHO **HAI** THANG DI TRÚ — đây là điều làm "hai tầng cùng hình dạng"
/// đúng THEO ĐỊNH NGHĨA, không nhờ hai chỗ tình cờ đồng ý
/// ─────────────────────────────────────────────────────────────────────────────
/// Glossary có hai tầng (Global · Tác phẩm), và mỗi tầng sống trong một `Store` khác nhau
/// (`global.db` / `project.db` của chính `.atproj` đang mở) — không phải một cột `tier`
/// trong cùng một bảng, cùng khuôn với mọi bảng hai tầng khác của dự án này (xem
/// [`CONFIG_VALUE_DDL`]: "Không cột `tier`"). `ScopeResolver::apply_override` chỉ phân
/// giải ĐÚNG khi cả hai tầng trả về **cùng một hình dạng hàng** — một trường lệch giữa
/// hai bảng là một bug không lộ ra ở `cargo test` (mỗi bảng test độc lập) mà chỉ lộ ra
/// lúc gọi `apply_override` thật, dưới dạng một cột đọc nhầm sang cột khác.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÒNG ĐỜI KHOÁ BẰNG **CẤU TRÚC**, KHÔNG BẰNG KỶ LUẬT — AD-36
/// ─────────────────────────────────────────────────────────────────────────────
/// `translation IS NULL` **LÀ** trạng thái *chờ chốt*. Không cột `status` song song —
/// hai dữ kiện nói cùng một chuyện thì chúng lệch được, và lệch trong im lặng (đúng ca
/// AD-36 sinh ra để chặn: *"đã chốt mà bản dịch rỗng"*). Vị từ `is_confirmed()` ở
/// [`crate::core::glossary::entry::GlossaryEntry`] là chỗ DUY NHẤT đọc bất biến này.
///
/// Trigger `glossary_entry_lifecycle_is_one_way` cưỡng chế chiều **một chiều**
/// *(đã chốt → không bao giờ lùi về chờ chốt)* bằng SQL — không phải một quy ước ở tầng
/// gọi mà một `UPDATE` bất cẩn phá được. `BEFORE UPDATE OF translation` + `WHEN OLD.translation
/// IS NOT NULL AND NEW.translation IS NULL` bắt đúng và chỉ đúng chiều lùi; đặt lại CÙNG
/// một bản dịch đã chốt, hay đổi các cột khác (`note`, `category`), đi qua bình thường.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ `CHECK` BẮT CHUỖI RỖNG LÀ CỐ Ý, VÀ NÓ **KHÁC** `segment.target_text`
/// ─────────────────────────────────────────────────────────────────────────────
/// Ở `segment`, *"chưa dịch"* là chuỗi **rỗng** `NOT NULL DEFAULT ''` ([`SEGMENT_TARGET_TEXT_DDL`])
/// — vì mọi segment luôn tồn tại. Ở đây **ngược lại**: vắng mặt là một trạng thái **có
/// nghĩa** (*chờ chốt*), nên `NULL` mang nghĩa và chuỗi rỗng bị `CHECK` cấm — một `INSERT`
/// hay `UPDATE` đặt `translation = ''`/`'   '` bị SQLite từ chối thẳng, không lặng lẽ tạo
/// ra ca "đã chốt mà bản dịch rỗng". Hai bảng chọn ngược nhau **có lý do**; đừng "đồng bộ"
/// chúng.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `trim(X)` MỘT THAM SỐ CHỈ CẮT DẤU CÁCH ASCII — đo được, và đã ĐO SAI ở bản đầu
/// ─────────────────────────────────────────────────────────────────────────────
/// Đo 2026-08-19 trên SQLite 3.53.4: `trim("   ")` bị `CHECK` một-tham-số chặn, nhưng
/// `"\t"` · `"\n"` · `"\r"` · `"\v"` (`char(11)`) · `"\f"` (`char(12)`) · NBSP
/// (`char(160)`, U+00A0) · dấu cách biểu ý (`char(12288)`, U+3000) đều **LỌT**. Hệ quả nếu
/// không sửa: một bản dịch `"\t"` làm `is_confirmed()` trả `true` với nội dung trắng —
/// đúng ca AD-36 sinh ra để chặn, và Epic 4 sẽ chèn một trường trống vào prompt.
///
/// ⇒ Cả hai `CHECK` dưới đây dùng dạng **hai tham số** `trim(X, <bảng ký tự>)`. Bảng ký tự
/// viết **khai triển tại chỗ** trong cả hai `CHECK`, không đặt tên hằng phụ —
/// `Migration::sql` là `&'static str` và `concat!` chỉ nhận literal (cùng ràng buộc đã ghi
/// ở doc-comment của [`PROJECT_MIGRATIONS`]).
///
/// 🔵 **CẬP NHẬT 2026-08-19 (lượt rà soát #2) — BẢY KÝ TỰ LÀ CHƯA ĐỦ, nay là ĐỦ 25.**
/// Bản vá đầu liệt **bảy** loại trắng và dừng ở đó. Đo lại: bảng bảy ký tự vẫn để **17**
/// điểm mã `White_Space` khác đi lọt — U+0085 (NEL) · U+1680 (dấu cách Ogham) ·
/// U+2000‥U+200A (mười một dấu cách in ấn, gồm U+2009 THIN SPACE) · U+2028 · U+2029 ·
/// U+202F (NBSP hẹp) · U+205F. Tức bản vá đầu **thu hẹp** lỗ hổng chứ không đóng nó, và
/// tuyên bố *"ca đã chốt mà bản dịch rỗng không biểu diễn được"* vẫn còn sai với 17 điểm mã
/// đó. Bảng dưới đây liệt **trọn** thuộc tính Unicode `White_Space` (25 điểm mã) — đo từng
/// điểm một: cả 25 bị chặn, và `"Mộ Dung"` lẫn `" 慕容 "` vẫn **nhận**.
///
/// ⚠️ **Vì sao đúng 25, không phải một tập tự chọn khác:** đây chính là tập mà
/// `str::trim()` của Rust cắt, nên hai lớp phòng thủ khoá cùng một tập — xem
/// [`crate::core::glossary::store::insert_manual_entry`], nơi ghi rõ lớp Rust và lớp SQL quan hệ
/// thế nào. Thêm một ký tự vào một lớp mà quên lớp kia là dựng lại đúng khoảng lệch mà
/// lượt rà soát này vừa đóng.
///
/// `source_term` mang cùng lỗ hổng và cùng bản vá: không có rào rỗng nào trước bản vá này
/// ngoài `NOT NULL`, và nó vừa là khoá tra cứu vừa là khoá của
/// `idx_glossary_entry_source_term` — một `insert_manual_entry("", …)` chiếm vĩnh viễn ô chuỗi
/// rỗng của chỉ mục UNIQUE đó. `CHECK` thứ nhất của bảng dưới đây đóng lỗ này.
///
/// 🔴 **CỬA SỔ DI TRÚ MỘT LẦN, GHI RA VÌ NÓ ĐỔI HÀNH VI TRÊN ĐĨA CỤC BỘ:** bản vá này sửa
/// MỘT hằng DDL mà bước 4 (`global.db`) / bước 12 (`project.db`) **đã từng chạy** trên máy
/// dev trước khi bản vá tồn tại — đúng khuôn "sửa hằng cũ tại chỗ là hai lược đồ cho cùng
/// một số phiên bản" mà vết sẹo số 4 của [`PROJECT_MIGRATIONS`] ghi lại. Khác vết sẹo đó,
/// cửa sổ này **còn đóng được**: chưa phát hành, nên không `.db` nào ngoài máy dev từng
/// chạm bước này. Nhưng **mọi `global.db`/`project.db` cục bộ đang ở `user_version = 4`/`12`
/// từ trước bản vá phải bị XOÁ rồi dựng lại** — nếu không, chúng giữ nguyên `CHECK` cũ
/// trong lược đồ đã ghi, và `PRAGMA user_version` sẽ nói dối rằng lược đồ đã ở bản vá. Bộ
/// test dùng thư mục tạm dựng mới mỗi lần nên không dính.
///
/// 🔴 **Cửa sổ này đã dùng HAI lần** — lượt đầu (bảng bảy ký tự) và lượt rà soát #2 (bảng
/// 25 ký tự), cả hai trước khi phát hành. Lượt thứ ba **không** có: sau bản phát hành đầu
/// tiên chạm bước 4/12, mọi lượt sửa bảng ký tự phải là một bước di trú MỚI, không phải
/// một lượt sửa hằng tại chỗ nữa.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `term_origin`, KHÔNG PHẢI `origin` TRẦN
/// ─────────────────────────────────────────────────────────────────────────────
/// Chữ *"xuất xứ"* trong PRD chỉ **bốn** thực thể rời nhau trong dự án này — bản dịch
/// (`segment.translation_origin`), mục Glossary (cột này), tài liệu nguồn, trích dẫn từ
/// điển — nên định danh phải tự phân biệt được. `segment.translation_origin`
/// ([`SEGMENT_TRANSLATION_ORIGIN_DDL`]) đã lấy khuôn `<mô tả cái gì>_origin`; cột này giữ
/// đúng khuôn đó thay vì đúc một quy ước thứ hai cho cùng một khái niệm.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// TỪNG CỘT, VÀ NÓ NEO VÀO ĐÂU
/// ─────────────────────────────────────────────────────────────────────────────
/// - `source_term` — khoá tra cứu, **không rỗng sau khi cắt khoảng trắng** (`CHECK` thứ
///   nhất — P2 ở trên). `UNIQUE INDEX idx_glossary_entry_source_term` cưỡng chế *"một
///   thuật ngữ nguồn, một mục"* ở tầng SQLite — cùng doctrine `UNIQUE (source_code,
///   entry_id)` của [`PINNED_ENTRY_DDL`]: không có cửa sổ đua giữa một `SELECT` kiểm trùng
///   và một `INSERT` mà hai luồng có thể chen vào giữa. `core::glossary::store::insert_manual_entry`
///   cắt khoảng trắng biên **ở tầng Rust** trước khi ghi, để `" 慕容"` và `"慕容"` không
///   thành hai hàng dưới một chỉ mục tự xưng là "một thuật ngữ, một mục" — `CHECK` ở đây
///   là lưới THỨ HAI, không phải lưới duy nhất.
/// - `translation` — `NULL`-able, xem hai mục 🔴/⚠️ ở trên.
/// - `note` — `NOT NULL DEFAULT ''`: một ghi chú vắng mặt và một ghi chú rỗng là **cùng
///   một điều** (không có nhánh nghiệp vụ nào phân biệt chúng), khác hẳn `translation` —
///   cùng lý lẽ phân biệt hai cột này với nhau ở dòng ngay trên.
/// - `category` (**bốn** giá trị: `person` · `place` · `domain_term` · `other`) và
///   `term_origin` (**ba** giá trị: `manual` · `import_scan` · `review_harvest`) — chuỗi cố
///   định, cưỡng chế bằng `CHECK … IN (…)` **khác** khuôn `chapter.status`/`segment.status`/
///   `config_value.kind` (không `CHECK`, cưỡng chế ở tầng Rust). Khác biệt có chủ: ba cột
///   kia đổi giá trị hợp lệ theo epic tới sau (AD-31 dự trù một trạng thái thứ ba cho
///   `segment.status`); bộ giá trị của `category` và `term_origin` đã đóng ở FR46/FR47 và
///   Story 3.2 tồn tại đúng để KHÔNG cho `term_origin` có giá trị thứ tư ("candidate" ở lại
///   bảng chờ riêng — AD-20).
/// - `created_at` — cùng khuôn `chapter`/`segment`: sinh ở tầng SQL bằng `strftime`, không
///   truyền từ Rust.
///
/// **Không cột `tier`** — xem mục 🔴 đầu tiên ở trên.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.2) — câu "không bảng ứng viên" đã HẾT ĐÚNG.** Bảng
/// [`GLOSSARY_CANDIDATE_DDL`] (bước **13** của `PROJECT_MIGRATIONS`, chỉ ở `project.db` —
/// KHÔNG ở `GLOBAL_MIGRATIONS`) nay tồn tại. Trạng thái *ứng viên* vẫn không nằm trong
/// CHÍNH bảng `glossary_entry` — nó sống trong bảng ứng viên tách riêng đó, đúng AD-20:
/// không cơ chế tự động nào ghi thẳng vào `glossary_entry`, chỉ `approve_candidate` mới
/// chèn được, và luôn suy `term_origin` từ `candidate_origin` của chính hàng ứng viên.
pub const GLOSSARY_ENTRY_DDL: &str = "\
CREATE TABLE glossary_entry (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  source_term  TEXT    NOT NULL,
  translation  TEXT,
  note         TEXT    NOT NULL DEFAULT '',
  category     TEXT    NOT NULL,
  term_origin  TEXT    NOT NULL,
  created_at   TEXT    NOT NULL,
  CHECK (trim(source_term, ' ' || char(9) || char(10) || char(11) || char(12) || char(13)
                               || char(133) || char(160) || char(5760)
                               || char(8192) || char(8193) || char(8194) || char(8195)
                               || char(8196) || char(8197) || char(8198) || char(8199)
                               || char(8200) || char(8201) || char(8202)
                               || char(8232) || char(8233) || char(8239) || char(8287)
                               || char(12288)) <> ''),
  CHECK (translation IS NULL
         OR trim(translation, ' ' || char(9) || char(10) || char(11) || char(12) || char(13)
                                  || char(133) || char(160) || char(5760)
                                  || char(8192) || char(8193) || char(8194) || char(8195)
                                  || char(8196) || char(8197) || char(8198) || char(8199)
                                  || char(8200) || char(8201) || char(8202)
                                  || char(8232) || char(8233) || char(8239) || char(8287)
                                  || char(12288)) <> ''),
  CHECK (category    IN ('person','place','domain_term','other')),
  CHECK (term_origin IN ('manual','import_scan','review_harvest'))
);
CREATE UNIQUE INDEX idx_glossary_entry_source_term ON glossary_entry (source_term);
CREATE TRIGGER glossary_entry_lifecycle_is_one_way
BEFORE UPDATE OF translation ON glossary_entry
WHEN OLD.translation IS NOT NULL AND NEW.translation IS NULL
BEGIN SELECT RAISE(ABORT, 'glossary lifecycle is one-way'); END;";

/// Lược đồ bảng `glossary_candidate` — **bước 13 của `project.db`, KHÔNG của `global.db`**
/// — Story 3.2, AD-20 · AD-36.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 CHỈ Ở TẦNG TÁC PHẨM — không thêm bước tương ứng vào `GLOBAL_MIGRATIONS`
/// ─────────────────────────────────────────────────────────────────────────────
/// Khác [`GLOSSARY_ENTRY_DDL`] (một hằng dùng cho HAI thang), bảng này chỉ có MỘT thang:
/// một ứng viên sinh ra từ việc quét/thu hoạch một Tác phẩm cụ thể, và AC của Story 3.2
/// khoá thẳng vào Tác phẩm đó — "Bảng ứng viên ở `global.db`" nằm trong §Never của story.
/// Nhân bản hằng này sang `GLOBAL_MIGRATIONS` là đúng lỗi mà §Never cấm.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 HÀNG ỨNG VIÊN KHÔNG BỊ XOÁ KHI BỎ — `resolution` GHI LẠI, KHÔNG `DELETE`
/// ─────────────────────────────────────────────────────────────────────────────
/// `epics.md:2854-2857` viết *"nó rời bảng chờ"* khi đọc lướt qua nghe như một `DELETE`,
/// nhưng "rời" ở đây là rời DANH SÁCH CHỜ DUYỆT (`resolution IS NULL`), không phải rời
/// đĩa. Xoá hàng thật thì lần quét sau chèn lại được — `UNIQUE (source_term)` không còn gì
/// để chặn — và AC kế tiếp ("không quay lại") chết ngay trong cùng một câu. `resolution`
/// là bộ nhớ vĩnh viễn của quyết định người dùng, tách khỏi "hàng còn tồn tại trên đĩa".
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `resolution` KHÔNG SUY ĐƯỢC TỪ `glossary_entry` — cột tường minh, không phải `EXISTS`
/// ─────────────────────────────────────────────────────────────────────────────
/// Story 3.9 cho xoá một mục `glossary_entry`. Nếu "đã duyệt" được đọc bằng một phép
/// `EXISTS (SELECT 1 FROM glossary_entry WHERE source_term = …)` thay vì cột này, lượt xoá
/// đó làm một ứng viên đã duyệt sống lại thành "chưa duyệt" trong im lặng — đúng lớp lỗi
/// *rỗng im lặng* mà `AGENTS.md` gọi là trung tâm của dự án. Cột `resolution` ghi lại
/// *"người dùng đã quyết"*, một sự thật khác hẳn *"mục hiện có mặt trên đĩa"*, và hai bảng
/// không được phép nói ngược nhau (AC "reject trên một id đã duyệt bị từ chối, mục
/// Glossary đã sinh ra vẫn nguyên").
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 TRIGGER MỘT CHIỀU CANH `OLD.resolution IS NOT NULL` — MỌI GIÁ TRỊ, KHÔNG RIÊNG NULL
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ **Lượt rà soát #1 (2026-08-20) bắt một lỗ hổng ở chính DDL nháp đầu của story này:**
/// bản đầu chỉ chặn `WHEN OLD.resolution IS NOT NULL AND NEW.resolution IS NULL` — tức
/// đúng khuôn của [`glossary_entry_lifecycle_is_one_way`], vốn hợp lý cho MỘT cột hai giá
/// trị (`NULL`/`không NULL`). Ở đây `resolution` có BA giá trị (`NULL` · `approved` ·
/// `rejected`), nên khuôn đó chỉ chặn chiều LÙI VỀ `NULL` và bỏ lọt chiều NGANG:
/// `reject_candidate` rồi `approve_candidate` trên CÙNG một `id` chạy sạch và sinh một
/// hàng `glossary_entry` MỚI — đúng AC trung tâm *"ứng viên bị bỏ không quay lại"* chết,
/// vì `UNIQUE (source_term)` chỉ canh đường `INSERT`, không canh đường DUYỆT LẠI. Chiều
/// ngược (`approve` rồi `reject`) để lại `resolution = 'rejected'` cạnh một mục Glossary
/// còn sống — hai bảng nói ngược nhau, không lỗi nào ném ra. Đo trên mã đã dựng trước khi
/// sửa: cả hai chiều đều chạy qua, không một cổng nào đỏ.
///
/// ⇒ `WHEN` rút về **`OLD.resolution IS NOT NULL`** — đã quyết thì không quyết lại, KỂ CẢ
/// quyết lại y hệt giá trị cũ. Đây là lớp BẢO ĐẢM; lớp Rust "đọc được" đứng trước nó ở
/// [`crate::core::glossary::candidate_store::approve_candidate`]/`reject_candidate` — cùng
/// khuôn hai lớp mà `.trim()`/`CHECK` đã dùng ở Story 3.1: lớp Rust cho một lỗi phân biệt
/// được với "id không tồn tại", lớp trigger cho bảo đảm không phá được kể cả khi lớp Rust
/// bị bỏ qua (đua giữa hai luồng, một lượt duyệt hàng loạt của Story 3.8, một cú bấm đúp).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// TỪNG CỘT, VÀ NÓ NEO VÀO ĐÂU
/// ─────────────────────────────────────────────────────────────────────────────
/// - `source_term` — cùng `CHECK` hai tham số + cùng `UNIQUE INDEX` mà `glossary_entry`
///   dùng cho cột cùng tên; bảng ký tự khoảng trắng **TRÙNG TỪNG BYTE** với
///   [`GLOSSARY_ENTRY_DDL`] — `tests/glossary_contract.rs` khoá mệnh đề này bằng một phép
///   so sánh chuỗi, không chỉ bằng mắt. `UNIQUE` là cơ chế chặn "không quay lại ở lần quét
///   sau" — không phải một phép kiểm ở tầng gọi.
/// - `candidate_origin` — **hai** giá trị: `import_scan` · `review_harvest`. Không có
///   `manual`: một mục nhập tay không đi qua bảng chờ (`insert_manual_entry` ghi thẳng vào
///   `glossary_entry`), nên `CandidateOrigin` (kiểu Rust ở
///   [`crate::core::glossary::candidate`]) chỉ khai đúng hai biến thể — không biểu diễn
///   được ca thứ ba mà lược đồ này không cần.
/// - `resolution` — `NULL` == *chờ duyệt* (cùng khuôn `glossary_entry.translation`: một
///   cột, không `status` song song). Non-`NULL` là MỘT trong hai giá trị đóng
///   (`approved`/`rejected`), cưỡng chế bằng `CHECK`.
/// - `created_at` — cùng khuôn `glossary_entry`/`chapter`/`segment`: sinh ở tầng SQL bằng
///   `strftime`, không truyền từ Rust.
///
/// 🔵 **CẬP NHẬT 2026-08-22 (Story 3.5) — `số lần xuất hiện`/`ví dụ ngữ cảnh` ĐÃ CÓ.** Câu
/// dưới đây từng đúng và nay hết đúng cho hai cột đầu tiên; sửa tại chỗ thay vì để nó lặng
/// lẽ sai. Chúng tới bằng bước di trú **14**
/// ([`GLOSSARY_CANDIDATE_OCCURRENCE_CONTEXT_DDL`]), **không** bằng một lượt sửa hằng này —
/// cùng tiền lệ `SEGMENT_TARGET_TEXT_DDL` đã ghi ở trên: sửa `GLOSSARY_CANDIDATE_DDL` tại
/// chỗ sẽ làm một `project.db` cũ (di trú tới bước 13, KHÔNG chạy lại DDL này) và một kho
/// mới (chạy DDL đã sửa) lệch lược đồ dưới cùng một số phiên bản. **Không `bản dịch đề
/// xuất`** (3.7) · **không `phân loại`/`con trỏ đang duyệt`** (3.8) · **không `tỉ lệ nhất
/// quán`** (Epic 8) — ba cột đó vẫn giữ nguyên chủ. `segment` nhận sáu bước `ALTER` rải
/// khắp Epic 2 — đó là TIỀN LỆ, không phải một thiếu sót ở bảng này.
pub const GLOSSARY_CANDIDATE_DDL: &str = "\
CREATE TABLE glossary_candidate (
  id                INTEGER PRIMARY KEY AUTOINCREMENT,
  source_term       TEXT NOT NULL,
  candidate_origin  TEXT NOT NULL,
  resolution        TEXT,
  created_at        TEXT NOT NULL,
  CHECK (trim(source_term, ' ' || char(9) || char(10) || char(11) || char(12) || char(13)
                               || char(133) || char(160) || char(5760)
                               || char(8192) || char(8193) || char(8194) || char(8195)
                               || char(8196) || char(8197) || char(8198) || char(8199)
                               || char(8200) || char(8201) || char(8202)
                               || char(8232) || char(8233) || char(8239) || char(8287)
                               || char(12288)) <> ''),
  CHECK (candidate_origin IN ('import_scan','review_harvest')),
  CHECK (resolution IS NULL OR resolution IN ('approved','rejected'))
);
CREATE UNIQUE INDEX idx_glossary_candidate_source_term ON glossary_candidate (source_term);
CREATE TRIGGER glossary_candidate_resolution_is_one_way
BEFORE UPDATE OF resolution ON glossary_candidate
WHEN OLD.resolution IS NOT NULL
BEGIN SELECT RAISE(ABORT, 'glossary candidate resolution is one-way'); END;";

/// Hai cột thêm vào `glossary_candidate` — **bước 14 của `project.db`**, Story 3.5.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `ALTER TABLE` RIÊNG, KHÔNG SỬA [`GLOSSARY_CANDIDATE_DDL`] TẠI CHỖ
/// ─────────────────────────────────────────────────────────────────────────────
/// Đúng tiền lệ [`SEGMENT_TARGET_TEXT_DDL`] (bước 6 của `segment`): một `project.db` đã di
/// trú tới bước 13 KHÔNG BAO GIỜ chạy lại `GLOSSARY_CANDIDATE_DDL` — sửa hằng đó tại chỗ
/// làm kho CŨ (bảng thiếu hai cột) và kho MỚI (bảng có hai cột, tạo từ đầu) lệch lược đồ
/// trong khi cùng báo `user_version = 13`. Hai cột phải tới bằng một bước **mới**.
///
/// - `occurrence_count INTEGER NOT NULL DEFAULT 0` — số lần chuỗi lặp trong Chương vừa
///   quét. `NOT NULL DEFAULT 0` là giá trị AN TOÀN cho mọi hàng CŨ (Story 3.2, trước story
///   này) — chúng không tới từ một lượt quét nên "0 lần" là câu trung thực duy nhất, không
///   phải một chỗ trống.
/// - `context_example TEXT` — **nullable**, không `NOT NULL DEFAULT ''`. Cùng lý do
///   `occurrence_count`: một hàng CŨ không có câu ví dụ nào để kể, và `NULL` nói đúng điều
///   đó; một chuỗi rỗng `''` sẽ trông như "đã quét nhưng câu ví dụ rỗng" — hai trạng thái
///   khác nhau bị một giá trị che mất.
///
/// ⚠️ **Không `CHECK` nào canh hai cột này.** `occurrence_count` luôn `>= ngưỡng` trên
/// đường ghi sản phẩm (Rust là lớp lọc duy nhất, `core::glossary::scan`), và một `CHECK
/// (occurrence_count >= 0)` chỉ canh một bất biến mà đường ghi hôm nay không bao giờ vi
/// phạm — thêm nó là một ràng buộc chưa ai đo cần.
pub const GLOSSARY_CANDIDATE_OCCURRENCE_CONTEXT_DDL: &str = concat!(
    "ALTER TABLE glossary_candidate ADD COLUMN occurrence_count INTEGER NOT NULL DEFAULT 0;",
    "ALTER TABLE glossary_candidate ADD COLUMN context_example TEXT;"
);

/// Dựng lại `glossary_entry` để `CHECK (term_origin IN (…))` nhận giá trị **thứ tư**,
/// `'file_import'` — **bước 5 của `global.db`, bước 15 của `project.db`** — Story 3.10,
/// FR49/NFR9 (xuất/nhập Glossary qua CSV/TSV).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT `CHECK` KHÔNG `ALTER` ĐƯỢC — DỰNG LẠI BẢNG LÀ ĐƯỜNG DUY NHẤT
/// ─────────────────────────────────────────────────────────────────────────────
/// SQLite không có `ALTER TABLE … ALTER CHECK`. [`GLOSSARY_ENTRY_DDL`] (bước 4/12) mang một
/// `CHECK (term_origin IN ('manual','import_scan','review_harvest'))` cứng; thêm
/// `'file_import'` bắt buộc đi qua khuôn kinh điển "tạo bảng mới → chép hàng → xoá bảng cũ →
/// đổi tên": `CREATE TABLE glossary_entry_new (…)` với `CHECK` bốn giá trị → `INSERT … SELECT`
/// **giữ nguyên `id`** → `DROP TABLE glossary_entry` (cuốn theo `UNIQUE INDEX` VÀ trigger) →
/// `ALTER TABLE … RENAME TO glossary_entry` → dựng lại **cả hai** thứ vừa mất.
///
/// 🔴 **KHÔNG sửa [`GLOSSARY_ENTRY_DDL`] tại chỗ** — doc-comment của nó đã ghi nguyên lý:
/// một kho đã di trú tới bước 4/12 không bao giờ chạy lại hằng đó, nên sửa tại chỗ cho ra
/// hai lược đồ khác nhau cùng báo một `user_version`. Hằng NÀY là bước MỚI.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `id` KHÔNG ĐƯỢC TÁI DÙNG — MỐC `sqlite_sequence` PHẢI ĐI THEO, KHÔNG SUY LẠI TỪ HÀNG
/// ─────────────────────────────────────────────────────────────────────────────
/// Bảng dùng `AUTOINCREMENT`; nếu đã có hàng bị xoá trước lượt di trú này, `id` cao nhất
/// TỪNG CẤP không còn nằm trong bảng nữa — không đọc lại được bằng `MAX(id)` sau khi chép
/// hàng. Nhưng chính hàng `sqlite_sequence` của bảng CŨ (`glossary_entry`) đã giữ đúng mốc đó
/// suốt vòng đời AUTOINCREMENT của nó, kể cả cho id đã về hưu. Đo được (kiểm tay bằng
/// `sqlite3` 2026-08-24, xem Verification): chèn tường minh `id` vào bảng MỚI **không** tự
/// nâng mốc của bảng mới lên đúng giá trị lịch sử đó (nó chỉ theo `id` lớn nhất VỪA CHÈN, bỏ
/// sót id đã xoá) — nên mốc phải được **mang theo** bằng tay:
/// 1. `INSERT INTO sqlite_sequence (name, seq) SELECT 'glossary_entry_new', 0 WHERE NOT
///    EXISTS (…)` — bảo đảm bảng mới CÓ một hàng `sqlite_sequence` để `UPDATE` sau nhắm vào,
///    kể cả khi lượt `INSERT … SELECT` phía trên chưa tự tạo ra hàng đó (kho rỗng chưa từng
///    cấp `id` nào).
/// 2. `UPDATE sqlite_sequence SET seq = MAX(seq, COALESCE((SELECT seq FROM sqlite_sequence
///    WHERE name = 'glossary_entry'), 0)) WHERE name = 'glossary_entry_new'` — `MAX(a, b)`
///    **hai tham số** là dạng SCALAR của SQLite (khác `MAX(x)` dạng aggregate), nên đây so
///    trực tiếp hai số, không cần `GROUP BY`. Chạy TRƯỚC `DROP TABLE glossary_entry` — dòng đó
///    xoá luôn hàng `sqlite_sequence` của bảng cũ (đã đo, xem Verification), nên mốc phải
///    được đọc ra TRƯỚC khi nó biến mất.
///
/// `ALTER TABLE … RENAME TO glossary_entry` sau đó tự đổi tên hàng `sqlite_sequence` từ
/// `'glossary_entry_new'` sang `'glossary_entry'` (cơ chế nội bộ của chính lệnh `RENAME`, đã
/// đo — xem Verification) — không cần một câu `UPDATE sqlite_sequence SET name = …` viết tay.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `DROP TABLE` CUỐN THEO INDEX VÀ TRIGGER — DỰNG LẠI CẢ HAI
/// ─────────────────────────────────────────────────────────────────────────────
/// `idx_glossary_entry_source_term` (`UNIQUE`) và `glossary_entry_lifecycle_is_one_way`
/// (vòng đời một chiều, AD-36) đều gắn vào TÊN BẢNG `glossary_entry`, không sống sót qua
/// `DROP TABLE`. Thiếu trigger ⇒ AD-36 chết trong im lặng (một `UPDATE` lùi về `NULL` chạy
/// sạch); thiếu index ⇒ `source_term` trùng được chèn. Cả hai được tạo lại NGUYÊN VĂN, sau
/// `RENAME`.
///
/// Bốn `CHECK` giữ NGUYÊN VĂN — bảng khoảng trắng 25 điểm mã **trùng từng byte** với
/// [`GLOSSARY_ENTRY_DDL`] (khoá bằng phép so chuỗi ở `glossary_contract.rs`, không chỉ bằng
/// mắt) — chỉ `CHECK (term_origin IN (…))` mọc thêm `'file_import'`.
pub const GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL: &str = "\
CREATE TABLE glossary_entry_new (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  source_term  TEXT    NOT NULL,
  translation  TEXT,
  note         TEXT    NOT NULL DEFAULT '',
  category     TEXT    NOT NULL,
  term_origin  TEXT    NOT NULL,
  created_at   TEXT    NOT NULL,
  CHECK (trim(source_term, ' ' || char(9) || char(10) || char(11) || char(12) || char(13)
                               || char(133) || char(160) || char(5760)
                               || char(8192) || char(8193) || char(8194) || char(8195)
                               || char(8196) || char(8197) || char(8198) || char(8199)
                               || char(8200) || char(8201) || char(8202)
                               || char(8232) || char(8233) || char(8239) || char(8287)
                               || char(12288)) <> ''),
  CHECK (translation IS NULL
         OR trim(translation, ' ' || char(9) || char(10) || char(11) || char(12) || char(13)
                                  || char(133) || char(160) || char(5760)
                                  || char(8192) || char(8193) || char(8194) || char(8195)
                                  || char(8196) || char(8197) || char(8198) || char(8199)
                                  || char(8200) || char(8201) || char(8202)
                                  || char(8232) || char(8233) || char(8239) || char(8287)
                                  || char(12288)) <> ''),
  CHECK (category    IN ('person','place','domain_term','other')),
  CHECK (term_origin IN ('manual','import_scan','review_harvest','file_import'))
);
INSERT INTO glossary_entry_new
  (id, source_term, translation, note, category, term_origin, created_at)
  SELECT id, source_term, translation, note, category, term_origin, created_at
  FROM glossary_entry;
INSERT INTO sqlite_sequence (name, seq)
  SELECT 'glossary_entry_new', 0
  WHERE NOT EXISTS (SELECT 1 FROM sqlite_sequence WHERE name = 'glossary_entry_new');
UPDATE sqlite_sequence
  SET seq = MAX(seq, COALESCE((SELECT seq FROM sqlite_sequence WHERE name = 'glossary_entry'), 0))
  WHERE name = 'glossary_entry_new';
DROP TABLE glossary_entry;
ALTER TABLE glossary_entry_new RENAME TO glossary_entry;
CREATE UNIQUE INDEX idx_glossary_entry_source_term ON glossary_entry (source_term);
CREATE TRIGGER glossary_entry_lifecycle_is_one_way
BEFORE UPDATE OF translation ON glossary_entry
WHEN OLD.translation IS NOT NULL AND NEW.translation IS NULL
BEGIN SELECT RAISE(ABORT, 'glossary lifecycle is one-way'); END;";

/// Lược đồ bảng `library_orphan` — **bước 6 của `global.db`, KHÔNG có bước song sinh ở
/// `PROJECT_MIGRATIONS`/`LIBRARY_INDEX_MIGRATIONS`** — phán quyết Ice #1 (2026-08-27, lật
/// §Design Notes vòng một của `5-3-quet-lai-thu-muc.md`).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO CỜ MỒ CÔI SỐNG Ở ĐÂY, KHÔNG Ở `library-index.db` — LẬT QUYẾT ĐỊNH VÒNG MỘT
/// ─────────────────────────────────────────────────────────────────────────────
/// Story 5.3 vòng một chọn giữ cờ mồ côi làm một cột (`orphaned`) NGAY TRONG `library_work`
/// (xem lịch sử ở doc-comment của [`LIBRARY_WORK_DDL`]) — lý lẽ khi đó: "hẹp hơn, không kho
/// mới, mất mát khi xoá chỉ mục chỉ là MẤT MỘT LỜI NHẮC". Ice bác lý lẽ đó 2026-08-27: một
/// LỜI NHẮC mà người dùng phải **chủ động gỡ** (`forget_orphan`, không có đường tự động nào
/// xoá nó) không phải một cache — nó là một **quyết định người dùng đã ghi lại** ("tôi biết
/// đường dẫn cũ, tôi CHƯA gỡ nó"), và một quyết định người dùng không được phép biến mất chỉ
/// vì `library-index.db` bị xoá tay hoặc lệch phiên bản (AD-8 hứa "xoá chỉ mục là an toàn" —
/// lời hứa đó chỉ ĐÚNG khi kho không giữ gì ngoài thứ suy ra được từ `.atproj`). ⇒ Cờ mồ côi
/// chuyển sang `global.db`, đúng mái nhà của mọi dữ liệu người dùng khác không gắn với một
/// `.atproj` cụ thể (mục ghim — [`PINNED_ENTRY_DDL`], Glossary chung — [`GLOSSARY_ENTRY_DDL`]),
/// và `library_work` quay lại dẫn xuất TRỌN VẸN (xem [`LIBRARY_WORK_DDL`]).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 CỬA MỘT CHIỀU — HẠ CẤP ỨNG DỤNG SAU KHI BƯỚC NÀY PHÁT HÀNH SẼ MẤT ĐƯỜNG VÀO GLOSSARY
/// ─────────────────────────────────────────────────────────────────────────────
/// Di trú của `global.db` CHỈ TIẾN (AD-30): gặp `PRAGMA user_version` MỚI HƠN bản ứng dụng
/// hiểu ⇒ [`super::Store::open`] TỪ CHỐI MỞ, không bao giờ ghi vào (bước 3 của `Store::open`,
/// `StoreError::SchemaTooNew`) — khác hẳn `library-index.db` (AD-8, dẫn xuất, xoá-và-dựng-lại
/// vô hại). `global.db` mang Glossary chung, mục ghim, và MỌI cấu hình `AppConfig` — không
/// phải một kho có thể "xoá rồi dựng lại" mà không mất gì. **Hệ quả PHẢI nói thẳng, không để
/// người sau tự phát hiện:** một khi một `global.db` đã di trú qua bước 6 (đích 6) trên máy
/// người dùng, HẠ CẤP xuống một bản ứng dụng cũ hơn bước này (`GLOBAL_MIGRATIONS` đích ≤ 5) sẽ
/// làm bản cũ đó THẤY `user_version = 6 > 5` và TỪ CHỐI MỞ `global.db` — người dùng **mất
/// đường vào Glossary chung và mọi mục đã ghim** cho tới khi họ nâng cấp trở lại. Đây không
/// phải một lỗi tiềm ẩn cần vá; đó là bản chất của AD-30 áp dụng cho bước NÀY như mọi bước
/// khác của `GLOBAL_MIGRATIONS` — ghi ra ở đây để một quyết định "phát hành rồi hạ cấp" trong
/// tương lai không phải tự suy luận lại cái giá của nó.
///
/// - `work_id` — khoá chính, TRÙNG [`crate::core::library::meta::WorkMeta::work_id`] (cùng
///   định danh với `library_work.work_id`, chỉ khác BẢNG/KHO đang giữ nó).
/// - `atproj_path` — đường dẫn CŨ, TUYỆT ĐỐI trên máy này, giữ NGUYÊN VĂN từ lúc hàng thành
///   mồ côi — đủ để màn hình nêu "nó trỏ tới đâu" (AC3) mà KHÔNG cần đọc `library-index.db`.
/// - `name` — ảnh chụp tên Tác phẩm lúc thành mồ côi, cùng lý do `headword`/`gloss` là ảnh
///   chụp ở [`PINNED_ENTRY_DDL`]: đủ để hiện lại hàng mà không phải tra `library-index.db`.
///
/// **Không** cột thời điểm (`orphaned_at`) — cùng lý lẽ mà §Design Notes của
/// `5-3-quet-lai-thu-muc.md` đã ghi cho cột `orphaned` cũ ("một cột cho một câu hỏi chưa ai
/// hỏi"): chưa AC nào đòi sắp mục mồ côi theo thời gian.
pub const LIBRARY_ORPHAN_DDL: &str = "\
CREATE TABLE library_orphan (
  work_id     TEXT PRIMARY KEY,
  atproj_path TEXT NOT NULL,
  name        TEXT NOT NULL
);";

/// Bộ di trú của `global.db`. Hôm nay **sáu** bước — Story 1.7 · 1.8 · 1.20 · 3.1 · 3.10 ·
/// phán quyết Ice #1 (Story 5.3, 2026-08-27).
///
/// 🔴 **Sáu bước, và đích là phiên bản 6.** Không số nào bị bỏ trống ở bộ này (khác
/// [`PROJECT_MIGRATIONS`], nơi số 4 là một số **đã cháy**), nên ở đây số bước và đích trùng
/// nhau — và điều đó **không** làm câu trên thừa: nó là mệnh đề mà cổng
/// `tests/segment_contract.rs::the_migration_doc_headers_state_the_target_their_array_reaches`
/// đọc. 🔵 Câu này thêm 2026-08-25 để **cả hai** bộ khai đích bằng cùng một hình dạng máy
/// đọc được; trước đó chỉ [`PROJECT_MIGRATIONS`] khai, và chính nó là bộ để tiêu đề lệch
/// khỏi mảng suốt ba ngày.
///
/// Không thêm bước cho một lược đồ chưa tồn tại. Mỗi story sở hữu bước di trú của
/// chính nó, cùng lúc với bảng mà nó cần.
///
/// ⚠️ Thêm một bước ở đây làm `tests/store_contract.rs` đỏ ở **đúng một** ca
/// (`a_fresh_database_migrates_up_to_target_and_logs_it`, ca duy nhất chạy trên bộ di trú
/// THẬT), và đó là hành vi đúng: số phiên bản đổi phải là một quyết định có người ký, chứ
/// không phải một hiệu ứng phụ. Đừng "sửa cho nhất quán" các con số trong `TWO_STEP` /
/// `BROKEN_STEP_TWO` — chúng là fixture cục bộ và không phụ thuộc hằng này.
///
/// 🔵 **CẬP NHẬT 2026-08-19 (Story 3.1):** đích chuyển từ **3** lên **4** — bước
/// [`GLOSSARY_ENTRY_DDL`] (tầng Global của Glossary, AD-18). Câu *"ba bước, đích là 3"* đã
/// hết đúng, sửa tại chỗ thay vì để nó lặng lẽ sai.
///
/// 🔵 **CẬP NHẬT 2026-08-24 (Story 3.10):** đích chuyển từ **4** lên **5** — bước
/// [`GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL`] (giá trị `term_origin` thứ tư,
/// `file_import`, CÙNG một hằng với bước 15 của `project.db`).
///
/// 🔵 **CẬP NHẬT 2026-08-27 (phán quyết Ice #1, Story 5.3):** đích chuyển từ **5** lên **6** —
/// bước [`LIBRARY_ORPHAN_DDL`] (bảng `library_orphan`, cờ mồ côi của Library chuyển từ
/// `library-index.db` sang đây). Câu *"năm bước, đích là 5"* đã hết đúng, sửa tại chỗ.
pub const GLOBAL_MIGRATIONS: &[Migration] = &[
    Migration {
        to_version: 1,
        sql: SCHEMA_MIGRATION_LOG_DDL,
    },
    Migration {
        to_version: 2,
        sql: CONFIG_VALUE_DDL,
    },
    Migration {
        to_version: 3,
        sql: PINNED_ENTRY_DDL,
    },
    // Story 3.1 — tang Global cua Glossary (AD-18/AD-36): bang glossary_entry, CUNG mot
    // hang voi buoc 12 cua project.db. Xem doc-comment cua GLOSSARY_ENTRY_DDL.
    Migration {
        to_version: 4,
        sql: GLOSSARY_ENTRY_DDL,
    },
    // Story 3.10 -- gia tri term_origin thu tu, 'file_import' (FR49/NFR9), CUNG mot hang voi
    // buoc 15 cua project.db. Xem doc-comment cua GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL.
    Migration {
        to_version: 5,
        sql: GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL,
    },
    // Phan quyet Ice #1 (2026-08-27, Story 5.3) -- co mo coi cua Library chuyen tu
    // library-index.db sang day. Xem doc-comment cua LIBRARY_ORPHAN_DDL cho ly le day du
    // (bao gom canh bao CUA MOT CHIEU: ha cap sau buoc nay mat duong vao Glossary/muc ghim).
    Migration {
        to_version: 6,
        sql: LIBRARY_ORPHAN_DDL,
    },
];

/// Lược đồ bảng `work` — **bước 1 của `project.db`**, Story 1.15, AC4.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 ĐÚNG MỘT HÀNG, và `CHECK (id = 1)` là cơ chế bắt buộc số đó
/// ─────────────────────────────────────────────────────────────────────────────
/// `project.db` mang **một** Tác phẩm — hình dạng `.atproj/` của AD-9 khoá điều đó ở tầng
/// thư mục. Bảng này phản ánh đúng bất biến ở tầng lược đồ thay vì để nó thành một quy ước
/// không ai canh: một `INSERT` thứ hai vi phạm `CHECK` và **SQLite** từ chối, không phải
/// một `debug_assert!` mà bản release im lặng bỏ qua.
///
/// `work_id` là UUID v4 (AD-28) — sinh **một lần** lúc tạo, không đổi được, và là khoá
/// dựng lại `meta.json` (xem [`super::super::readonly`] không áp — đây là `project.db`).
/// `source_lang` là trường **bất biến** (AD-18): AC1 nói *"ngôn ngữ nguồn được đặt lúc tạo
/// và không đổi được về sau"* — bất biến này được cưỡng chế ở tầng ứng dụng
/// (`core/segment/import.rs`, không có `UPDATE` nào chạm cột này), không phải một
/// `CHECK`/trigger SQL, vì SQLite không có cú pháp "cột chỉ ghi một lần".
pub const WORK_DDL: &str = "\
CREATE TABLE work (
  id          INTEGER PRIMARY KEY,
  work_id     TEXT NOT NULL,
  name        TEXT NOT NULL,
  source_lang TEXT NOT NULL,
  genre       TEXT NOT NULL,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL,
  CHECK (id = 1)
);";

/// Thêm cột `work.status_override` — **bước 16 MỚI** của [`PROJECT_MIGRATIONS`], Story 5.4,
/// FR6.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `ALTER TABLE` RIÊNG, KHÔNG SỬA [`WORK_DDL`] TẠI CHỖ — vết sẹo số 4
/// ─────────────────────────────────────────────────────────────────────────────
/// Đúng tiền lệ [`SEGMENT_TARGET_TEXT_DDL`]/[`GLOSSARY_CANDIDATE_OCCURRENCE_CONTEXT_DDL`]:
/// một `project.db` đã di trú qua bước 2 (`WORK_DDL`) KHÔNG BAO GIỜ chạy lại hằng đó — sửa
/// nó tại chỗ làm kho CŨ (bảng bảy cột) và kho MỚI (bảng tám cột, tạo từ đầu) lệch lược đồ
/// trong khi cùng báo `user_version = 2`. Cột mới đi bằng một bước MỚI, số **16** — bước
/// cuối hiện tại của [`PROJECT_MIGRATIONS`] là **15**, và vết sẹo số 4 (xem doc-comment của
/// hằng đó) đã dạy: đọc chính danh sách `Migration` để biết bước kế tiếp, đừng đếm bằng mắt.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ KHÔNG `CHECK` — cưỡng chế giá trị hợp lệ là việc của tầng Rust
/// ─────────────────────────────────────────────────────────────────────────────
/// `status_override` mang một trong bốn giá trị của [`crate::core::lifecycle::LifecycleStatus`]
/// hoặc `NULL` (= đang suy ra, chưa ghi đè) — đúng khuôn `chapter.status`/`segment.status`/
/// `config_value.kind`: chuỗi tự do ở tầng SQL, cưỡng chế ở tầng Rust
/// (`commands::lifecycle::set_work_status_override`), không bằng `CHECK … IN (…)`. Một
/// `CHECK` ở đây biến mọi lượt nới danh mục bốn giá trị (nếu Ice từng chốt một giá trị thứ
/// năm) thành một bước di trú riêng cho MỖI kho đã phát hành, trong khi phép cưỡng chế thật
/// đã nằm ở `LifecycleStatus` phía Rust — nơi trình biên dịch làm việc đó.
///
/// `NULL`-hoặc-giá-trị, không một cờ boolean riêng (§Always của story): `status_override IS
/// NULL` ⇒ đang suy ra; có giá trị ⇒ giữ nguyên giá trị đó bất kể Chương đổi thế nào, cho
/// tới khi người dùng bỏ ghi đè.
pub const WORK_STATUS_OVERRIDE_DDL: &str = "ALTER TABLE work ADD COLUMN status_override TEXT;";

/// Lược đồ bảng `chapter_position` — **bước 17 MỚI của `project.db`**, Story 5.7, AD-3.
///
/// Giữ *"câu đang làm"* của mỗi Chương: `segment_id` là `segment.id` nơi caret đứng lúc
/// người dùng rời Chương lần gần nhất. Cố ý là `segment.id`, KHÔNG một `scrollTop` pixel —
/// AD-3 (Ice ký 2026-08-18) cấm tường minh đường pixel, và đường `editorCaretPlacement` đã
/// có (`GridPanel.vue:1110`) tự cuộn qua `focus()` một khi caret đặt đúng segment.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ VÌ SAO MỘT BẢNG RIÊNG, KHÔNG MỘT CỘT TRÊN `chapter`
/// ─────────────────────────────────────────────────────────────────────────────
/// (1) **Vắng hàng là một trạng thái phân biệt được**: AC5 đòi *"Chương chưa từng mở ⇒
/// segment đầu"* — một cột `chapter.last_segment_id NULL` cũng nói được điều đó, nhưng nói
/// **cùng chỗ** với dữ liệu nội dung Chương, nên mọi lượt đọc `chapter` (danh sách, tách
/// segment, vòng đời) kéo theo một cột không liên quan tới vai của lượt đọc đó.
/// (2) **`chapter` đang bị ba đường đọc/ghi khác chạm** (`WorkMeta::rebuild_from_store`,
/// `commands/lifecycle.rs` `UPDATE chapter SET status`, `create_work` `INSERT INTO
/// chapter`) — thêm một cột đổi theo **mỗi lượt rê caret** vào chính bảng đó đặt một giá trị
/// nhịp-cao cạnh những giá trị nhịp-thấp, và `chapter.updated_at` (thứ `rebuild_from_store`
/// dùng để tính `updated_at` của Tác phẩm, Story 5.6) sẽ nhảy theo mỗi lượt **đọc** của
/// người dùng — một hồi quy im lặng cho AC4 của Story 5.6.
/// (3) **Bảng riêng không đòi `chapter.updated_at` phải đổi**: `chapter_position.updated_at`
/// là mốc của chính hàng vị trí, tách hẳn.
///
/// ⚠️ **Giới hạn thật, ghi ra thay vì để người sau tưởng đã xét:** KHÔNG `FOREIGN KEY` tới
/// `chapter` — cùng khuôn cả lược đồ, `PRAGMA foreign_keys` mặc định TẮT trong SQLite, một
/// khoá ngoại khai ra mà không bật pragma là một lời hứa không ai giữ.
///
/// 🔵 **SỬA 2026-08-29 (Story 5.8) — mệnh đề "một Chương bị xoá để lại một hàng vị trí mồ
/// côi" đã HẾT ĐÚNG, và story này chính là chủ đã hứa dọn nó.** `commands::chapter` nay có
/// đúng một đường xoá hàng `chapter`: `merge_chapter_into_previous`, và nó
/// `DELETE FROM chapter_position WHERE chapter_id = <Chương bị gộp>` TRONG CÙNG giao dịch
/// trước khi `DELETE FROM chapter` — không hàng vị trí nào sống sót Chương chủ của nó. Đường
/// **tách** (`split_chapter_at_segment`) không xoá hàng `chapter` nào — nó chỉ chèn thêm một
/// Chương — nhưng nó **dời** hàng vị trí của Chương gốc sang Chương mới khi câu vị trí trỏ
/// tới đã đổi `chapter_id` (`UPDATE chapter_position SET chapter_id = ... WHERE segment_id
/// IN (...)`), nên không hàng nào bị bỏ lại trỏ vào một `chapter_id` đã dời chỗ mà không có
/// nó. ⇒ Không còn đường sản phẩm nào tạo ra một hàng `chapter_position` mồ côi.
pub const CHAPTER_POSITION_DDL: &str = "\
CREATE TABLE chapter_position (
  chapter_id INTEGER PRIMARY KEY,
  segment_id INTEGER NOT NULL,
  updated_at TEXT NOT NULL
);";

/// Lược đồ bảng `chapter` — **bước 1 của `project.db`**, Story 1.15, AC4.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `AUTOINCREMENT`, KHÔNG `INTEGER PRIMARY KEY` TRẦN
/// ─────────────────────────────────────────────────────────────────────────────
/// `INTEGER PRIMARY KEY` trần là bí danh của `rowid`, và SQLite **tái dùng** rowid đã xoá
/// khi nó là rowid lớn nhất từng cấp — cụ thể, xoá hàng cuối rồi chèn hàng mới sẽ nhận
/// lại đúng `id` vừa mất. AD-3 nói id đã về hưu **không bao giờ** được tái dùng.
/// `AUTOINCREMENT` giữ một sổ riêng (`sqlite_sequence`) và không bao giờ phát lại một giá
/// trị đã dùng, đổi lại chi phí ghi nhỏ mà không ai đo được ở quy mô một cuốn sách.
///
/// `ord` là **cột riêng** cho thứ tự hiển thị (AD-3, AD-32) — sắp lại được (Epic 2 gộp/tách
/// Chương) mà không đụng `id`. **Không** `UNIQUE` trên `ord` ở story này: Epic 2 tự
/// quyết cơ chế sắp lại (có thể để hở tạm thời trong một giao dịch nhiều bước).
///
/// `status` mang trạng thái vòng đời ban đầu *Chưa bắt đầu* (FR5) — chuỗi tự do ở tầng
/// SQL, cưỡng chế giá trị hợp lệ là việc của tầng Rust gọi nó (cùng khuôn với
/// `config_value.kind` ở `CONFIG_VALUE_DDL`, xem doc-comment ở trên).
///
/// `source_text` mang **nguyên khối** văn bản nguồn của Chương, và nó **ở lại nguyên khối**
/// sau Story 2.1: [`SEGMENT_DDL`] (bước 5) dựng các hàng `segment` **cạnh** nó chứ không
/// thay nó. Story 1.15 cố ý không dựng bảng `segment` — Quyết định #4 của story đó: AD-4
/// đóng băng ranh giới segment tính một lần lúc nhập, nên một bộ tách "tạm" ở đó là đóng
/// băng vĩnh viễn những ranh giới sai.
pub const CHAPTER_DDL: &str = "\
CREATE TABLE chapter (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  ord         INTEGER NOT NULL,
  title       TEXT,
  source_text TEXT NOT NULL,
  status      TEXT NOT NULL,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
);";

/// Lược đồ bảng `segment` — **bước 5 của `project.db`**, Story 2.1, AC3 · AC4 · AC5 · AC9.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `AUTOINCREMENT` LÀ CƠ CHẾ **DUY NHẤT** THOẢ AC5, VÀ NÓ PHẢI Ở TRONG DDL
/// ─────────────────────────────────────────────────────────────────────────────
/// AC5 nói *"một `segment.id` đã về hưu không bao giờ được tái dùng"*. Đó **không** phải một
/// lời hứa ở tầng Rust — nó là một thuộc tính của DDL. [`CHAPTER_DDL`] đã phân xử nguyên
/// văn cùng mệnh đề này: `INTEGER PRIMARY KEY` trần là bí danh của `rowid`, và SQLite **tái
/// dùng** rowid đã xoá khi nó là rowid lớn nhất từng cấp — xoá hàng cuối rồi chèn hàng mới
/// nhận lại **đúng** `id` vừa mất.
///
/// Cái giá của việc mất mệnh đề này lớn hơn hẳn ở `segment` so với ở `chapter`: `SegmentVersion`
/// của Story 2.6 gắn lịch sử **theo `segment.id`**, và AD-5 nói lịch sử phải tra được **kể cả
/// sau khi segment về hưu**. Một id phát lại là lịch sử của hai câu khác nhau chồng lên nhau.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// TỪNG CỘT, VÀ NÓ NEO VÀO ĐÂU
/// ─────────────────────────────────────────────────────────────────────────────
/// - `chapter_id` — AD-32: gộp/tách **Chương** chỉ đổi `chapter_id` và `ord`, giữ nguyên
///   `segment.id`. **Không** `FOREIGN KEY`: cùng khuôn với phần còn lại của lược đồ này,
///   nơi chưa bảng nào khai ràng buộc ngoài, và `PRAGMA foreign_keys` mặc định TẮT trong
///   SQLite — một khoá ngoại khai ra mà không bật pragma là một lời hứa không ai giữ.
/// - `ord` **cột riêng** — AC4 + AD-3, sắp lại được mà không đụng `id`. **Không** `UNIQUE`
///   trên `ord`, cùng lý do [`CHAPTER_DDL`] đã ghi: Epic 2 tự quyết cơ chế sắp lại, và nó có
///   thể để hở tạm thời trong một giao dịch nhiều bước. Đánh số **từ 1**, liên tục, không
///   lỗ — Story 2.10 (*"segment kế tiếp"*) đứng trên giả định đó.
/// - `is_paragraph_end` — AC6 + AD-37. Cờ kết đoạn của **nguyên văn**. `INTEGER` 0/1 vì
///   SQLite không có kiểu boolean; cưỡng chế giá trị hợp lệ là việc của tầng Rust, cùng
///   khuôn `chapter.status` và `config_value.kind`.
///   🔵 **CẬP NHẬT 2026-08-16 (Story 2.5d) — dòng này đã HẾT ĐÚNG VỀ MÃ và được sửa tại
///   chỗ.** Bản cũ viết *"**Một** cột, dùng chung cho nguyên văn và bản dịch; **không**
///   `source_paragraph_end`/`target_paragraph_end`"*. Nó đúng từ Story 2.1 tới 2.5c, và
///   **AD-46** (FR134) là thứ nới nó: bản dịch nay có cờ riêng
///   [`SEGMENT_TARGET_PARAGRAPH_END_DDL`] *(`is_target_paragraph_end`, bước 9)*, vì nhịp của
///   tiếng Việt không buộc phải là nhịp của bản gốc.
///   🔴 **AD-37 vẫn SỞ HỮU cột này** và không sửa một chữ — AD-46 khai đúng như vậy. Vế
///   *"tính MỘT LẦN lúc nhập, không đường mã nào tính lại lúc nạp"* áp cho **cả hai** cờ.
///   ⚠️ Và đây là chỗ dễ đọc nhầm nhất: cột thứ hai **không** phải `source_paragraph_end` —
///   không có cột nào tên vậy, và sẽ không có. Cờ nguồn giữ nguyên tên lịch sử của nó.
/// - `retired_at` — AD-5 *"về hưu = tombstone"*. Story 2.1 **không** cho segment nào về hưu;
///   cột có mặt để Story 2.8 không phải mở một bước di trú thứ hai chỉ để thêm một cột, và
///   để `ornament` (giá trị vạch lề thứ 5) có chỗ đọc.
/// - `created_at`/`updated_at` — cùng khuôn `chapter`: sinh ở tầng SQL bằng
///   `strftime('%Y-%m-%dT%H:%M:%fZ','now')`, không truyền từ Rust.
///
/// **Ba cột CỐ Ý không có, và mỗi cột có chủ:** `target_text` (bản dịch) → Story 2.2/2.3,
/// đi kèm bước di trú 6 — thêm hôm nay là đoán trước hợp đồng flush của AD-35 mà 2.3 chưa
/// chốt. `status` (máy trạng thái AD-31) → Story 2.5. `role` (`alt` | `caption`, AD-42) →
/// Story 6.13.
///
/// 🔴 **CẬP NHẬT 2026-08-12 (Story 2.2, Quyết định #1 do Ice chốt):** `target_text` **đã có**
/// — nó tới bằng bước di trú **6** ([`SEGMENT_TARGET_TEXT_DDL`]), **không** bằng một lượt
/// sửa hằng này. Hằng này là DDL của một bảng **tạo mới**, và một `project.db` đã ở phiên
/// bản 5 không bao giờ chạy lại nó; sửa nó tại chỗ cho ra hai lược đồ khác nhau cho cùng
/// một số phiên bản — đúng lớp lỗi mà vết sẹo số 4 ở [`PROJECT_MIGRATIONS`] ghi lại.
/// Hai cột còn vắng vẫn giữ nguyên chủ: `status` → Story 2.5, `role` → Story 6.13.
///
/// 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5, Quyết định #5 và #6 do Ice chốt):** `status` **đã
/// có** — nó tới bằng bước di trú **7** ([`SEGMENT_STATUS_AND_VERSION_DDL`]), cùng lượt với
/// bảng `segment_version`, và **không** bằng một lượt sửa hằng này *(cùng lý do bước 6 đã
/// ghi ngay trên)*. ⇒ Danh sách *"ba cột cố ý không có"* nay đọc là **một**: `role`
/// (`alt` | `caption`, AD-42) → **Story 6.13**.
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 INDEX ĐẦU TIÊN CỦA TOÀN KHO — Ice ký 2026-08-12, code review
/// ─────────────────────────────────────────────────────────────────────────────
/// Trước lượt này lược đồ **không có `CREATE INDEX` nào**, ở cả `global.db` lẫn `project.db`.
/// `segment` là bảng đầu tiên xứng đáng một cái, vì nó là bảng đầu tiên **phình theo nội
/// dung** chứ không theo số Tác phẩm: đo thật 2026-08-12 cho **10.477** hàng từ chỉ 21
/// Chương. Không index, phép đếm ở `commands::segment` và mọi lượt *"tải segment của một
/// Chương"* của Story 2.2 đều quét **toàn bảng** của cả Tác phẩm.
///
/// ⚠️ **Quy ước đặt tên do chỗ này dựng ra** (không có tiền lệ để chép): `idx_<bảng>_<cột
/// theo thứ tự trong index>`.
///
/// ⚠️ **Vì sao `(chapter_id, ord)` chứ không chỉ `chapter_id`:** mọi chỗ đọc segment của một
/// Chương đều cần chúng **theo `ord`** (Story 2.2 render, Story 2.10 điều hướng *"segment kế
/// tiếp"*). Cột thứ hai biến index thành covering cho phép sắp, nên SQLite khỏi một lượt sắp
/// tạm. **Không** `UNIQUE` — cùng lý do đã ghi cho `ord` ở trên: Epic 2 được phép để hở tạm
/// trong một giao dịch nhiều bước.
///
/// ⚠️ **Bước 5 vì thế không còn "chỉ làm một việc" đúng chữ Quyết định #4 của Story 2.1 —
/// và đó vẫn đúng tinh thần.** Thứ Quyết định #4 cấm nhét vào một bước di trú là một **quy
/// tắc nghiệp vụ** (chạy phép tách câu), vì nó trộn tầng và chạy im lặng lúc mở Tác phẩm.
/// Một `CREATE INDEX` là **DDL** — cùng tầng, cùng giao dịch, không quy tắc nào. Lý do sửa
/// ngay bây giờ thay vì mở bước 6: lúc review, cả **21** `project.db` thật còn ở
/// `user_version = 3`, chưa db nào chạm bước 5 — cửa sổ sửa miễn phí, và nó đóng lại ngay
/// lượt mở app đầu tiên.
pub const SEGMENT_DDL: &str = "\
CREATE TABLE segment (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  chapter_id       INTEGER NOT NULL,
  ord              INTEGER NOT NULL,
  source_text      TEXT    NOT NULL,
  is_paragraph_end INTEGER NOT NULL,
  retired_at       TEXT,
  created_at       TEXT    NOT NULL,
  updated_at       TEXT    NOT NULL
);
CREATE INDEX idx_segment_chapter_ord ON segment (chapter_id, ord);";

/// Cột `segment.target_text` — **bước 6 của `project.db`**, Story 2.2, AC13 · Task 1.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO SỐ **6**, VÀ VÌ SAO MỘT `ALTER TABLE` CHỨ KHÔNG SỬA [`SEGMENT_DDL`]
/// ─────────────────────────────────────────────────────────────────────────────
/// Bước 5 đã chạy trên `project.db` thật kể từ Story 2.1. Sửa [`SEGMENT_DDL`] tại chỗ chỉ
/// đổi lược đồ của các tệp **tạo mới**; tệp đã ở phiên bản 5 không chạy lại bước đó, nên
/// hai tệp cùng mang `user_version = 5` sẽ có hai lược đồ khác nhau — đúng thứ vết sẹo số
/// 4 ở [`PROJECT_MIGRATIONS`] ghi lại bằng chữ. Một bước mới là đường duy nhất.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `NOT NULL DEFAULT ''` — CHUỖI RỖNG, KHÔNG `NULL`. Story 2.2 · Task 1.4
/// ─────────────────────────────────────────────────────────────────────────────
/// *"Chưa dịch"* là một chuỗi **rỗng**, không phải một giá trị **vắng mặt**. Đó không phải
/// khẩu vị: nó quyết nhánh *"không vạch"* của AC3, và một `Option<String>` ở đó cho hai
/// cách nói cùng một điều — `None` và `Some("")` — mà tầng hiển thị phải gộp lại bằng kỷ
/// luật ở **mọi** chỗ đọc. Cột `NOT NULL` đẩy phép gộp đó xuống SQLite, nơi nó không quên
/// được.
///
/// ⚠️ `DEFAULT ''` cũng là thứ làm bước này chạy được trên bảng **đã có dữ liệu**: SQLite
/// đòi một `DEFAULT` không phải `NULL` cho mọi `ADD COLUMN … NOT NULL`. Mọi hàng `segment`
/// có sẵn (đo 2026-08-12: **10.477** hàng trên dữ liệu thật của Epic 1) nhận chuỗi rỗng,
/// tức trạng thái *"chưa dịch"* — đúng sự thật, không phải một giá trị mồi.
///
/// ⚠️ **KHÔNG** `CREATE INDEX` nào cho cột này. Không đường đọc nào lọc theo `target_text`;
/// index của lượt nạp là `idx_segment_chapter_ord` đã dựng ở bước 5.
pub const SEGMENT_TARGET_TEXT_DDL: &str =
    "ALTER TABLE segment ADD COLUMN target_text TEXT NOT NULL DEFAULT '';";

/// Máy trạng thái segment (AD-31) — **bước 7 của `project.db`**, Story 2.5, AC9 · Quyết
/// định #5 và #6 (Ice ký 2026-08-14).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO SỐ **7**, VÀ VÌ SAO KHÔNG PHẢI 5
/// ─────────────────────────────────────────────────────────────────────────────
/// `sprint-status.yaml` mang từ Story 2.1 một dòng nói *"bước di trú kế tiếp phải đánh số
/// 5"*. Mệnh đề đó đúng ở thời điểm nó được viết và **đã hết đúng**: 5 đã tiêu
/// ([`SEGMENT_DDL`], Story 2.1) và 6 đã tiêu ([`SEGMENT_TARGET_TEXT_DDL`], Story 2.2).
/// ⇒ Đọc [`PROJECT_MIGRATIONS`] ngay dưới chứ đừng đọc một ghi chép ở nơi khác; số kế tiếp
/// là **7**. *(Vế **vĩnh viễn** đúng của dòng đó là "số 4 đã cháy, không tái dùng" — cổng
/// `segment_contract.rs::the_project_migration_set_never_reuses_the_burned_number_four`.)*
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO MỘT `ALTER TABLE` CHỨ KHÔNG SỬA [`SEGMENT_DDL`] — cùng lý do bước 6
/// ─────────────────────────────────────────────────────────────────────────────
/// [`SEGMENT_DDL`] là DDL của một bảng **tạo mới**; một `project.db` đã ở phiên bản 5 không
/// bao giờ chạy lại nó. Sửa nó tại chỗ cho ra **hai lược đồ khác nhau cho cùng một số phiên
/// bản** — đúng vết sẹo số 4. Đo 2026-08-14: **21** `project.db` thật đang ở phiên bản 6.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `status TEXT NOT NULL DEFAULT 'draft'` — Quyết định #5, đường (a)
/// ─────────────────────────────────────────────────────────────────────────────
/// Hai giá trị hợp lệ: `'draft'` | `'confirmed'`. **Cưỡng chế ở tầng Rust**, đúng khuôn
/// `chapter.status` và `config_value.kind` — và **không** `CHECK`. Thêm một `CHECK` ở một
/// bảng mà hai bảng anh em không có là dựng hai quy ước cho cùng một việc.
///
/// `TEXT` chứ không `INTEGER` 0/1: một cột boolean đóng cứng máy trạng thái ở hai giá trị,
/// và AD-31 đã có sẵn hai ứng viên cho giá trị thứ ba *(về hưu do AD-5; nhập từ tài liệu
/// song ngữ)*. Cái giá của `TEXT` là vài byte mỗi hàng; cái giá của `INTEGER` là **một bước
/// di trú nữa** vào ngày một giá trị thứ ba xuất hiện.
///
/// ⚠️ `DEFAULT 'draft'` là thứ làm bước này chạy được trên bảng **đã có dữ liệu**: SQLite
/// đòi một `DEFAULT` không phải `NULL` cho mọi `ADD COLUMN … NOT NULL`.
/// 🔴 Và giá trị mặc định đó là một **quyết định nghiệp vụ**, không một chi tiết kỹ thuật:
/// mọi bản dịch có sẵn trên đĩa nhận `'draft'`, tức *"chưa ai ký"*. Cho chúng `'confirmed'`
/// là ký thay người dùng hàng nghìn lần, và ở Epic 7 mỗi chữ ký giả đó thành **một cặp TM
/// chưa ai duyệt** trong một kho dùng chung.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT BƯỚC CHỨ KHÔNG HAI — và đó không mâu thuẫn Quyết định #4 của Story 2.1
/// ─────────────────────────────────────────────────────────────────────────────
/// Cột `status` và bảng `segment_version` là **DDL của cùng một khái niệm** *(máy trạng
/// thái AD-31: trạng thái đi đâu, và phiên bản sinh ra ở đâu)*, cùng tầng, cùng giao dịch.
/// Thứ Quyết định #4 của Story 2.1 cấm nhét vào một bước là một **quy tắc nghiệp vụ** —
/// một câu DDL thứ hai thì không, và bước 5 đã có tiền lệ (`CREATE TABLE` + `CREATE INDEX`).
/// Tách thành 7 và 8 là dựng một `user_version` trung gian mà **không `project.db` nào từng
/// dừng ở đó**.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// TỪNG CỘT CỦA `segment_version`, VÀ AI ĐỌC NÓ
/// ─────────────────────────────────────────────────────────────────────────────
/// - `id` — `AUTOINCREMENT`, cùng lý do [`SEGMENT_DDL`]: một `INTEGER PRIMARY KEY` trần
///   **tái dùng** rowid lớn nhất vừa xoá, và Story 2.6 trỏ vào phiên bản theo `id`.
/// - `segment_id` — khoá về câu. **Không** `FOREIGN KEY`: cùng khuôn cả lược đồ này, và
///   `PRAGMA foreign_keys` mặc định TẮT trong SQLite ⇒ một khoá ngoại khai ra mà không bật
///   pragma là một lời hứa không ai giữ. ⚠️ Cũng **không** `ON DELETE CASCADE` — AD-5 nói
///   segment **về hưu** chứ không bị xoá, và AC của Story 2.6 đòi *"lịch sử của segment đã
///   về hưu vẫn tra được"*.
/// - `target_text` — bản dịch **tại thời điểm ký**. Đây là thứ FR101 khôi phục về.
/// - `created_at` — ISO-8601 UTC, sinh ở tầng SQL bằng `strftime`, **không** truyền từ
///   Rust. Story 2.6 đòi *thời điểm*, và nó phải sinh từ **một** đồng hồ.
///
/// ⚠️ **ĐÚNG BỐN CỘT, và con số bốn là một mệnh đề nghiệm thu.** Xuất xứ (FR117, Story 2.7)
/// và cặp TM (FR56, Epic 7) ghi tại **cùng một chuyển tiếp**, nhưng cột của chúng thuộc
/// story chủ của chúng — thêm sẵn hôm nay là đoán trước một hợp đồng chưa ai chốt, đúng thứ
/// doc-comment của [`SEGMENT_DDL`] đã cấm bằng chữ cho `target_text`.
///
/// ⚠️ **KHÔNG** `CREATE INDEX` nào cho bảng này, và đó là một quyết định chứ không một lượt
/// quên. Story 2.5 **chỉ ghi**, không đọc — không đường sản phẩm nào truy vấn
/// `segment_version` ở story này, nên một index ở đây là một phép tối ưu cho một đường đọc
/// **chưa ai đo**. Cùng luật mà [`SEGMENT_TARGET_TEXT_DDL`] đã ghi cho `target_text`.
/// **Chủ: Story 2.6** — nó mang đường đọc, nên nó mang index cùng lượt, đúng cách bước 5
/// mang `idx_segment_chapter_ord` cùng lúc với đường đọc cần nó.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.6): món nợ trên ĐÃ ĐÓNG, và câu *"không `CREATE INDEX`
/// nào cho bảng này"* nay chỉ còn đúng về **hằng này**, không về **lược đồ**.**
/// `segment_version` hôm nay **có** một index — `idx_segment_version_segment_created`
/// `(segment_id, created_at DESC)` — nhưng nó đến từ [`SEGMENT_VERSION_INDEX_DDL`] ở **bước
/// 10**, không từ đây. 🔴 Và nó **phải** ở đó chứ không ở đây: một `project.db` đã ở phiên
/// bản 7 không bao giờ chạy lại hằng này, nên thêm một dòng vào đây cho ra hai lược đồ khác
/// nhau mang cùng số **7**. Sửa tại chỗ thay vì để mệnh đề lặng lẽ sai.
pub const SEGMENT_STATUS_AND_VERSION_DDL: &str = "\
ALTER TABLE segment ADD COLUMN status TEXT NOT NULL DEFAULT 'draft';
CREATE TABLE segment_version (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  segment_id  INTEGER NOT NULL,
  target_text TEXT    NOT NULL,
  created_at  TEXT    NOT NULL
);";

/// Cờ **cắt bỏ câu khỏi bản dịch** (FR133) — **bước 8 của `project.db`**, Story 2.5c, AC7 ·
/// Quyết định #5 đường (a) (Ice ký 2026-08-15).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO SỐ **8**
/// ─────────────────────────────────────────────────────────────────────────────
/// 5 đã tiêu ([`SEGMENT_DDL`], Story 2.1), 6 đã tiêu ([`SEGMENT_TARGET_TEXT_DDL`], Story
/// 2.2), 7 đã tiêu ([`SEGMENT_STATUS_AND_VERSION_DDL`], Story 2.5). ⇒ Đọc
/// [`PROJECT_MIGRATIONS`] ngay dưới chứ đừng đọc một ghi chép ở nơi khác — `sprint-status.yaml`
/// còn mang một dòng từ Story 2.1 nói *"bước kế tiếp là 5"*, và dòng đó đã hết đúng **ba
/// lần** kể từ khi được viết. *(Vế **vĩnh viễn** đúng của nó là "số 4 đã cháy, không tái
/// dùng" — cổng `segment_contract.rs::the_project_migration_set_never_reuses_the_burned_number_four`.)*
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO MỘT `ALTER TABLE` CHỨ KHÔNG SỬA [`SEGMENT_DDL`] — cùng lý do bước 6 và 7
/// ─────────────────────────────────────────────────────────────────────────────
/// [`SEGMENT_DDL`] là DDL của một bảng **tạo mới**; một `project.db` đã ở phiên bản 5 không
/// bao giờ chạy lại nó. Sửa nó tại chỗ cho ra **hai lược đồ khác nhau cho cùng một số phiên
/// bản** — đúng vết sẹo số 4 ghi ở [`PROJECT_MIGRATIONS`].
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `is_omitted INTEGER NOT NULL DEFAULT 0` — Quyết định #5, đường (a)
/// ─────────────────────────────────────────────────────────────────────────────
/// Tiền lệ ở **chính bảng này**: `is_paragraph_end INTEGER NOT NULL` (bước 5). Phương án
/// còn lại — `omitted_at TEXT` theo khuôn `retired_at` — cũng hợp lệ và **không** sai; nó
/// chở thêm *khi nào*. Ice ký đường (a) 2026-08-15: cột này chỉ cần trả lời **thuộc hay
/// không thuộc bản dịch**, và *khi nào* là dữ liệu chưa AC nào đòi.
///
/// 🔴 **Đây là một TRỤC ĐỘC LẬP, không phải giá trị thứ ba của `status`** (AC2). *"Cắt bỏ"*
/// là quyết định về **thuộc hay không thuộc bản dịch**; `status` là **mức độ hoàn thành**.
/// Một câu đã cắt bỏ **vẫn giữ nguyên** `status` và `target_text` của nó — đó là thứ làm
/// AC4 (*"bỏ cờ ⇒ câu quay về đúng trạng thái cũ với nội dung cũ"*) đúng **mà không cần một
/// dòng mã khôi phục nào**: không gì bị mất thì không gì phải khôi phục.
///
/// ⚠️ **KHÔNG** `CHECK` — cùng khuôn `status` và `chapter.status`. Thêm một `CHECK` ở một
/// bảng mà hai bảng anh em không có là dựng hai quy ước cho cùng một việc. Giá trị hợp lệ
/// cưỡng chế ở tầng Rust.
///
/// ⚠️ `DEFAULT 0` là thứ làm bước này chạy được trên bảng **đã có dữ liệu**: SQLite đòi một
/// `DEFAULT` không phải `NULL` cho mọi `ADD COLUMN … NOT NULL`, và **không** nhận biểu thức
/// ở vị trí đó.
/// 🔴 Và số `0` là một **quyết định nghiệp vụ**: mọi hàng có sẵn *(đo 2026-08-12: **10.477**
/// hàng `segment` từ 21 Chương dữ liệu thật)* nhận *"chưa ai bấm cắt bỏ câu này"* — đúng sự
/// thật. Backfill `1` là một lớp lỗi **tệ hơn hẳn** một giá trị mồi thông thường: AC5 đòi
/// câu đã cắt bỏ **ẩn hoàn toàn, không dấu vết**, nên một cờ đặt nhầm ở đây không biểu hiện
/// thành một lỗi nào — nó biểu hiện thành **bản dịch biến mất khỏi mọi đầu ra**.
///
/// ⚠️ **KHÔNG** `CREATE INDEX`. Không đường đọc nào lọc theo `is_omitted`: lượt nạp lấy cả
/// Chương rồi lọc trong bộ nhớ, và index của nó là `idx_segment_chapter_ord` dựng ở bước 5.
/// Cùng luật mà [`SEGMENT_TARGET_TEXT_DDL`] và [`SEGMENT_STATUS_AND_VERSION_DDL`] đã ghi.
pub const SEGMENT_OMITTED_DDL: &str =
    "ALTER TABLE segment ADD COLUMN is_omitted INTEGER NOT NULL DEFAULT 0;";

/// Cờ **kết đoạn của BẢN DỊCH** (FR134) — **bước 9 của `project.db`**, Story 2.5d, AC2 ·
/// AC5 · Quyết định #5 đường (c) (Ice ký 2026-08-15).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO SỐ **9**
/// ─────────────────────────────────────────────────────────────────────────────
/// 5 · 6 · 7 · 8 đã tiêu ([`SEGMENT_DDL`] 2.1, [`SEGMENT_TARGET_TEXT_DDL`] 2.2,
/// [`SEGMENT_STATUS_AND_VERSION_DDL`] 2.5, [`SEGMENT_OMITTED_DDL`] 2.5c). ⇒ Đọc
/// [`PROJECT_MIGRATIONS`] ngay dưới chứ đừng đọc một ghi chép ở nơi khác — `sprint-status.yaml`
/// còn mang một dòng từ Story 2.1 nói *"bước kế tiếp là 5"*, và dòng đó đã hết đúng **bốn**
/// lần kể từ khi được viết.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO MỘT `ALTER TABLE` CHỨ KHÔNG SỬA [`SEGMENT_DDL`] — cùng lý do bước 6, 7 và 8
/// ─────────────────────────────────────────────────────────────────────────────
/// [`SEGMENT_DDL`] là DDL của một bảng **tạo mới**; một `project.db` đã ở phiên bản 5 không
/// bao giờ chạy lại nó. Sửa nó tại chỗ cho ra **hai lược đồ khác nhau cho cùng một số phiên
/// bản** — đúng vết sẹo số 4 ghi ở [`PROJECT_MIGRATIONS`].
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO MỘT CÂU `UPDATE` ĐI CÙNG — bước ĐẦU TIÊN của kho trộn DDL với DML
/// ─────────────────────────────────────────────────────────────────────────────
/// AC2 đòi cờ đích **mặc định bằng cờ nguồn**, tức một giá trị **theo hàng**. Mà `DEFAULT`
/// của SQLite **phải là hằng** — `DEFAULT is_paragraph_end` không tồn tại *(cùng ràng buộc
/// mà bước 6 và 7 đã ghi lại tại chỗ)*. ⇒ Vế *"bằng cờ nguồn"* **không** diễn đạt được
/// trong `ADD COLUMN`, và nó phải là một câu thứ hai.
///
/// [`migrate`] chạy `tx.execute_batch(m.sql)`, nên **nhiều câu ngăn bằng `;` chạy trọn
/// trong MỘT giao dịch** — hoặc cả hai câu cùng vào, hoặc không câu nào. Đó là thứ làm
/// đường này an toàn trên **21** `project.db` thật *(**10.477** hàng `segment`, đo
/// 2026-08-12)*.
///
/// ⚠️ **Tiền lệ chỉ đi được nửa đường, ghi ra thay vì giấu:** [`SEGMENT_STATUS_AND_VERSION_DDL`]
/// (bước 7) đã dùng đúng cơ chế nhiều câu / một giao dịch, nhưng đó là **DDL + DDL**. Đây
/// là **DDL + DML**, và kho chưa bước nào làm vậy. Ice ký đường (c) 2026-08-15 sau khi cả
/// hai vế được đặt lên bàn: đường còn lại *(backfill ở một lượt ghi Rust riêng sau khi mở
/// kho)* giữ bước di trú thuần DDL, nhưng mở một **cửa sổ** mà đĩa mang cờ đích sai và
/// **không** `PRAGMA user_version` nào nói ra điều đó.
///
/// 🔴 Và một câu `UPDATE` như thế **không** phá lằn ranh mà Quyết định #4 của Story 2.1
/// đặt ra *(xem [`SEGMENT_DDL`])*: lằn ranh đó cấm nhét một **quy tắc nghiệp vụ đang chạy**
/// vào lược đồ. Câu này không phát biểu một quy tắc — nó **chép một giá trị đã có sang một
/// cột mới đúng một lần**, tại đúng thời điểm cột đó ra đời. Quy tắc *"cờ đích soi gương cờ
/// nguồn cho tới khi người dùng đổi"* sống ở tầng Rust: đường nhập set cờ tường minh, và
/// lệnh đổi cờ ghi rời rạc.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 TÊN CỘT — Quyết định #5, đường (c)
/// ─────────────────────────────────────────────────────────────────────────────
/// `is_target_paragraph_end`, giữ tiền tố `is_` như `is_paragraph_end` và `is_omitted`.
/// Đường (a) *(`target_paragraph_end`, không tiền tố)* cũng hợp lệ; Ice ký (c) cho một từ
/// vựng đồng nhất. ⇒ **Dùng MỘT từ ở mọi tầng** — TS, lệnh, tài liệu — không đặt từ thứ
/// hai, đúng luật Quyết định #5 của Story 2.5c.
///
/// ⚠️ **AD-37 vẫn SỞ HỮU cờ nguồn.** Cột này **không** thay `is_paragraph_end` và **không**
/// đổi nghĩa của nó. AD-46 là thứ nới AD-37, và nó khai bằng chữ *"AD-37 không sửa một
/// chữ"*.
///
/// ⚠️ **KHÔNG** `CHECK` — cùng khuôn `status`, `is_omitted` và `chapter.status`.
/// ⚠️ **KHÔNG** `CREATE INDEX`: không đường đọc nào lọc theo cột này; lượt nạp lấy cả
/// Chương rồi lọc trong bộ nhớ.
pub const SEGMENT_TARGET_PARAGRAPH_END_DDL: &str = concat!(
    "ALTER TABLE segment ADD COLUMN is_target_paragraph_end INTEGER NOT NULL DEFAULT 0;",
    "UPDATE segment SET is_target_paragraph_end = is_paragraph_end;"
);

/// Index cho **đường đọc lịch sử phiên bản** (FR101) — **bước 10 của `project.db`**,
/// Story 2.6, AC1 · AC5 · Quyết định #7 đường (a) (Ice ký 2026-08-16).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO SỐ **10**
/// ─────────────────────────────────────────────────────────────────────────────
/// 5 · 6 · 7 · 8 · 9 đã tiêu ([`SEGMENT_DDL`] 2.1, [`SEGMENT_TARGET_TEXT_DDL`] 2.2,
/// [`SEGMENT_STATUS_AND_VERSION_DDL`] 2.5, [`SEGMENT_OMITTED_DDL`] 2.5c,
/// [`SEGMENT_TARGET_PARAGRAPH_END_DDL`] 2.5d). ⇒ Đọc [`PROJECT_MIGRATIONS`] ngay dưới chứ
/// đừng đọc một ghi chép ở nơi khác — `sprint-status.yaml` còn mang một dòng từ Story 2.1
/// nói *"bước kế tiếp là 5"*, và dòng đó đã hết đúng **năm** lần kể từ khi được viết.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO INDEX ĐẾN **BÂY GIỜ** MỚI CÓ — một món nợ có chủ, không một lượt quên
/// ─────────────────────────────────────────────────────────────────────────────
/// [`SEGMENT_STATUS_AND_VERSION_DDL`] dựng bảng `segment_version` ở bước 7 và cố ý **không**
/// kèm index. Doc-comment của chính hằng đó ghi lý do bằng chữ: Story 2.5 **chỉ ghi, không
/// đọc**, nên một index ở đó là một phép tối ưu cho một đường đọc **chưa ai đo** — và nó đặt
/// tên chủ luôn: *"Chủ: Story 2.6 — nó mang đường đọc, nên nó mang index cùng lượt"*.
///
/// ⇒ Đây là lượt đóng món nợ đó. Story 2.6 là story **đầu tiên** đọc `segment_version`
/// (`SELECT … WHERE segment_id = ?1 ORDER BY created_at DESC`), nên index tới cùng lượt với
/// thứ biện minh cho nó — đúng cách bước 5 mang `idx_segment_chapter_ord` cùng lúc với
/// đường đọc cần nó, và đúng luật *"không tối ưu cho một đường đọc chưa tồn tại"*.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO MỘT `CREATE INDEX` CHỨ KHÔNG SỬA [`SEGMENT_STATUS_AND_VERSION_DDL`] TẠI CHỖ
/// ─────────────────────────────────────────────────────────────────────────────
/// Cám dỗ ở đây mạnh hơn ở bước 6/8/9, vì thêm một dòng `CREATE INDEX` vào cuối hằng của
/// bước 7 **trông** sạch hơn hẳn một bước di trú mới. Nó sai đúng một lớp:
///
/// Một `project.db` đã ở phiên bản 7 **không bao giờ chạy lại** hằng của bước 7 — [`migrate`]
/// lọc `m.to_version > from`. ⇒ Sửa hằng đó tại chỗ cho ra **hai lược đồ khác nhau mang cùng
/// một số phiên bản**: db mới tạo có index, db cũ thì không, và `PRAGMA user_version` nói
/// **7** ở cả hai. Không cổng nào phân biệt được chúng, và chúng rẽ nhau ở máy người dùng
/// chứ không ở đây. Đó chính là vết sẹo số 4 ghi ở [`PROJECT_MIGRATIONS`], ở một hình dạng
/// êm hơn — vết sẹo số 4 ít nhất còn làm `Store::open` từ chối; lượt này thì **im lặng**.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 HÌNH DẠNG INDEX — Quyết định #7, đường (a)
/// ─────────────────────────────────────────────────────────────────────────────
/// `(segment_id, created_at DESC)` khớp **đúng** hình dạng truy vấn của AC1: lọc theo
/// `segment_id`, sắp theo thời điểm giảm dần. Tên theo khuôn `idx_segment_chapter_ord`
/// (bảng, rồi các cột).
///
/// Hai đường còn lại và vì sao chúng bị loại — nói ra thay vì giả vờ đã cân nhắc:
/// - **(b)** chỉ `(segment_id)`, để SQLite tự sắp. ⚠️ Với một segment có **ít** phiên bản
///   thì (a) và (b) **không phân biệt được bằng phép đo** — ghi ra thay vì bịa một con số.
///   (a) được ký vì nó khớp truy vấn, không vì có một phép đo bác (b).
/// - **(c)** sắp theo `id DESC` và **không** index nào: `id` là `AUTOINCREMENT` nên đơn
///   điệu, tức thứ tự `id` **là** thứ tự thời gian. Nó đúng, và nó bị loại vì nó khoá thứ
///   tự hiển thị vào một **chi tiết cài đặt của SQLite** thay vì vào cột mà AC5 nói tới —
///   và vì nó bỏ luôn món nợ có chủ ở trên.
///
/// ⚠️ **KHÔNG** `CHECK` — giá trị hợp lệ cưỡng chế ở tầng Rust, cùng khuôn `status`,
/// `is_omitted` và `chapter.status`.
/// ⚠️ **KHÔNG** `FOREIGN KEY` mới trên `segment_version`. Bảng cố ý không có khoá ngoại và
/// không `ON DELETE CASCADE` — AD-5 *"về hưu = tombstone"*, không phải một lượt xoá — và đó
/// là thứ làm **AC4 đúng theo cấu trúc**: lịch sử của một segment đã về hưu không đi đâu cả.
/// Một `CREATE INDEX` không đụng tới mệnh đề đó, và đừng nhân lúc này mà thêm.
/// ⚠️ **Thuần DDL, một câu.** Khác bước 9, ở đây **không** có vế backfill: index là một cấu
/// trúc dẫn xuất, SQLite dựng nó từ dữ liệu đã có ngay trong câu `CREATE`. Không hàng
/// `segment_version` nào đổi một byte.
pub const SEGMENT_VERSION_INDEX_DDL: &str =
    "CREATE INDEX idx_segment_version_segment_created ON segment_version (segment_id, created_at DESC);";

/// Xuất xứ bản dịch ở cấp segment (FR117/AD-47) — **bước 11 của `project.db`**, Story 2.7,
/// AC1 · AC3 · AC6 · Quyết định #3 đường (b′) và #6 đường (a) (Ice ký 2026-08-16).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO SỐ **11**
/// ─────────────────────────────────────────────────────────────────────────────
/// 5 · 6 · 7 · 8 · 9 · 10 đã tiêu ([`SEGMENT_DDL`] 2.1, [`SEGMENT_TARGET_TEXT_DDL`] 2.2,
/// [`SEGMENT_STATUS_AND_VERSION_DDL`] 2.5, [`SEGMENT_OMITTED_DDL`] 2.5c,
/// [`SEGMENT_TARGET_PARAGRAPH_END_DDL`] 2.5d, [`SEGMENT_VERSION_INDEX_DDL`] 2.6).
/// ⇒ Đọc [`PROJECT_MIGRATIONS`] ngay dưới chứ đừng đọc một ghi chép ở nơi khác —
/// `sprint-status.yaml` còn mang một dòng từ Story 2.1 nói *"bước kế tiếp là 5"*, và dòng đó
/// đã hết đúng **sáu** lần kể từ khi được viết.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 CỘT NẰM TRÊN `segment`, KHÔNG TRÊN `segment_version` — Quyết định #1 đường (a)
/// ─────────────────────────────────────────────────────────────────────────────
/// Hai tài liệu chỉ vào hai bảng khác nhau: AD-31 + ERD nói *"ghi vào segment"*, còn
/// doc-comment của [`SEGMENT_STATUS_AND_VERSION_DDL`] để ngỏ một cột trên `segment_version`.
/// **Phép đo phân xử**, không phải một lượt chọn theo khẩu vị: AC3 đòi một giá trị đọc được
/// **lúc nạp**, tức trước bất kỳ lượt xác nhận nào — và một segment chưa từng xác nhận có
/// **0 hàng** `segment_version` *(đường `INSERT` duy nhất nằm trong nhánh chuyển tiếp của
/// `commands::segment::confirm_segment`)*. ⇒ Một cột chỉ ở `segment_version` **không biểu
/// diễn được AC3**.
///
/// ⚠️ **Hệ quả có tên, không phải một lượt bỏ sót:** `segment_version` **không** mang xuất xứ,
/// nên khôi phục (FR101, Story 2.6) trả **văn bản** về mà không trả xuất xứ về — AD-47 ⑤ khai
/// điều đó bằng chữ và đặt chủ cho món nợ. Đừng "sửa" bằng cách thêm một cột ở đây.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 TÊN CỘT `translation_origin` — và vì sao **không** phải `origin` trần
/// ─────────────────────────────────────────────────────────────────────────────
/// §Consistency Conventions của spine ghi bằng chữ: chữ *"xuất xứ"* trong PRD chỉ **bốn** thực
/// thể rời nhau — bản dịch (FR117) · mục Glossary (FR47) · tài liệu nguồn (FR128/FR131) ·
/// trích dẫn từ điển (FR30) — nên định danh phải **tự phân biệt được**. Và `origin` trần đã
/// đông nghĩa, đo được: `WorkspaceDock.vue` dùng `origin === 'user'` cho lượt kích hoạt panel
/// dockview, `editorPanelState.ts` gọi nhánh gốc của flush là *originator*. Tiền tố
/// `translation_` là thứ làm cái tên trả lời được câu *"xuất xứ của cái gì"* ngay tại chỗ đọc.
/// ⚠️ Ứng viên còn lại là `translated_by`; nó bị loại vì nó đọc như một **tên người**, mà cột
/// này chở một **hạng mục** — và vì `''` *(chưa có bản dịch)* không phải một câu trả lời cho
/// *"ai dịch"*.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `TEXT NOT NULL DEFAULT ''` — Quyết định #3, đường (b′)
/// ─────────────────────────────────────────────────────────────────────────────
/// Bốn giá trị trên đĩa: `''` *(chưa có bản dịch)* cộng đúng ba giá trị của FR117. Danh mục
/// và ánh xạ xuống trục nhị phân FR118 nằm ở AD-47 ⑥; các hằng là
/// `commands::segment::TRANSLATION_ORIGIN_*`.
///
/// Hai đường bị loại, nói ra thay vì giả vờ đã cân nhắc:
/// - **(a)** `DEFAULT '<tôi dịch>'` ⇒ mọi câu **chưa ai viết** mang sẵn nhãn *tôi dịch*, và
///   nhãn đó đi thẳng vào kho TM ở Epic 7. Một lời khai sai về một câu chưa tồn tại.
/// - **(b)** `NULL`-able cho *"chưa có bản dịch"* ⇒ **mâu thuẫn một quyết định đã ký của chính
///   bảng này**: doc-comment của [`SEGMENT_TARGET_TEXT_DDL`] cấm bằng chữ hình dạng
///   `Option<String>` cho `target_text` *("`None` và `Some(\"\")` là hai cách nói cùng một
///   điều")*. (b′) lấy đúng vế đúng của (b) mà không phá tiền lệ.
///
/// ⚠️ `DEFAULT ''` cũng là thứ làm bước này chạy được trên bảng **đã có dữ liệu**: SQLite đòi
/// một `DEFAULT` không phải `NULL` cho mọi `ADD COLUMN … NOT NULL`.
/// ⚠️ **KHÔNG** `CHECK` — cùng khuôn `status`, `is_omitted`, `is_target_paragraph_end` và
/// `chapter.status`: giá trị hợp lệ cưỡng chế ở tầng Rust.
/// ⚠️ **KHÔNG** `CREATE INDEX`: không đường đọc nào lọc theo cột này. Bộ lọc theo xuất xứ mà
/// FR62 hứa sống trên **kho TM** (Epic 7), không trên bảng này.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 BACKFILL THEO **HÀNG** — Quyết định #6 đường (a), và vì sao nó đúng sự thật
/// ─────────────────────────────────────────────────────────────────────────────
/// `status = 'confirmed'` ⇒ *tôi dịch*. Đo được, không suy: hôm nay **không cơ chế nào khác**
/// đặt văn bản vào một segment — FR115 (nhập song ngữ) là Epic 6, FR58 (điền sẵn từ TM) là
/// Epic 7, đề xuất AI là Epic 4, chấp nhận thay đổi FR94 là Epic 8. Mọi câu **đã ký** trên đĩa
/// hôm nay **chắc chắn** do người dùng gõ. ⇒ Câu `UPDATE` này **chép một sự thật đã có sang một
/// cột mới đúng một lần**, nó không phát biểu một quy tắc đang chạy — cùng lập luận mà bước 9
/// đã ghi cho lằn ranh của Quyết định #4 (Story 2.1).
///
/// Đường **(b)** *(backfill đồng loạt)* bị loại vì nó khai *tôi dịch* cho cả những câu **chưa
/// ai viết**; đường **(c)** *(không backfill)* với `DEFAULT ''` để lại mọi câu đã ký mang
/// *"chưa có bản dịch"* — một lời khai tự mâu thuẫn ngay trên cùng một hàng.
///
/// ⚠️ **DDL + DML trong một hằng** — tiền lệ là bước 9, và [`migrate`] chạy `execute_batch`
/// trong **một** giao dịch nên hai câu này cùng sống hoặc cùng chết.
/// 🔴 **`'self'` viết thẳng ở đây là một bản sao của [`crate::commands::segment::
/// TRANSLATION_ORIGIN_SELF`], và bản sao đó KHÔNG tránh được:** `Migration::sql` là
/// `&'static str` và `concat!` chỉ nhận **literal**, không nhận một `const` đặt tên *(cùng ràng
/// buộc mà doc-comment của [`PROJECT_MIGRATIONS`] đã ghi cho luật "mỗi bước một hằng")*. Lưới
/// cho bản sao đó là một test hợp đồng —
/// `segment_contract.rs::the_backfill_literal_matches_the_origin_constant_it_copies` — chứ
/// không phải kỷ luật của người sửa.
pub const SEGMENT_TRANSLATION_ORIGIN_DDL: &str = concat!(
    "ALTER TABLE segment ADD COLUMN translation_origin TEXT NOT NULL DEFAULT '';",
    "UPDATE segment SET translation_origin = 'self' WHERE status = 'confirmed';"
);

/// Bộ di trú của `project.db`. Hôm nay **mười sáu** bước — Story 1.15 · 2.1 · 2.2 · 2.5 ·
/// 2.5c · 2.5d · 2.6 · 2.7 · 3.1 · 3.2 · 3.5 · 3.10 · 5.4 · 5.7.
///
/// 🔴 **Mười sáu bước, và đích là phiên bản 17.** Số **4** bị **bỏ trống có chủ ý** — xem vết
/// sẹo ở cuối doc-comment này. `validate_strictly_increasing` chấp nhận một lỗ hổng số
/// (`[1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]` tăng dần nghiêm ngặt), và
/// [`migrate`] lọc theo `to_version > from` nên một lỗ hổng không làm bước nào bị bỏ qua.
///
/// ⚠️ Con số này đọc **bảy**, không sáu: bước 4 mà bản đầu của Story 1.20 thêm vào đã bị
/// gỡ ở lượt Ice ký lại 2026-08-11 *(vết sẹo ghi đầy đủ ở cuối doc-comment này)*. Một
/// dòng tiêu đề nói một số mà bảng hằng ngay dưới nói một số khác là đúng thứ rot mà cả
/// kiến trúc này dựa vào doc-comment để tránh — bắt ở code review 2026-08-11.
///
/// 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5):** đích chuyển từ **6** lên **7** — bước
/// [`SEGMENT_STATUS_AND_VERSION_DDL`]. Câu *"năm bước, đích là 6"* đã hết đúng, sửa tại chỗ
/// thay vì để nó lặng lẽ sai.
///
/// 🔵 **CẬP NHẬT 2026-08-15 (Story 2.5c):** đích chuyển từ **7** lên **8** — bước
/// [`SEGMENT_OMITTED_DDL`] (FR133). Câu *"sáu bước, đích là 7"* đã hết đúng, sửa tại chỗ.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.5d):** đích chuyển từ **8** lên **9** — bước
/// [`SEGMENT_TARGET_PARAGRAPH_END_DDL`] (FR134/AD-46). Câu *"bảy bước, đích là 8"* đã hết
/// đúng, sửa tại chỗ.
/// 🔴 Bước 9 là bước **đầu tiên** của kho mang **DDL + DML** trong một hằng. Lý do đầy đủ ở
/// doc-comment của chính hằng đó; điều đáng nhớ **ở đây** là mệnh đề *"mỗi bước một hằng"*
/// ngay dưới nói về **số hằng**, không về **số câu SQL** — [`migrate`] đã chạy
/// `execute_batch` từ đầu, nên nhiều câu trong một bước là hình dạng **sẵn có**, không một
/// lượt nới.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.6):** đích chuyển từ **9** lên **10** — bước
/// [`SEGMENT_VERSION_INDEX_DDL`] (FR101). Câu *"tám bước, đích là 9"* đã hết đúng, sửa tại
/// chỗ.
/// 🔴 Bước 10 là bước **đầu tiên** của kho **không thêm một cột nào** — nó chỉ dựng một cấu
/// trúc dẫn xuất. Điều đáng nhớ ở đây: một bước di trú không nhất thiết đổi **hình dạng dữ
/// liệu**, và bước này cố ý **không** đụng một hàng nào. Nó tồn tại vì hằng của bước 7
/// **không được phép sửa tại chỗ** *(một `project.db` đã ở v7 không bao giờ chạy lại nó)* —
/// lý do đầy đủ ở doc-comment của chính hằng bước 10.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.7):** đích chuyển từ **10** lên **11** — bước
/// [`SEGMENT_TRANSLATION_ORIGIN_DDL`] (FR117/AD-47). Câu *"chín bước, đích là 10"* đã hết
/// đúng, sửa tại chỗ.
/// 🔴 Bước 11 là bước **thứ hai** mang DDL + DML *(bước 9 là bước đầu)*, và nó khác bước 9 ở
/// một điểm đáng nhớ: bước 9 backfill từ một cột **cùng hàng** (`is_paragraph_end`), còn bước
/// này backfill từ một **mệnh đề về thế giới** — *"hôm nay không cơ chế nào ngoài người dùng
/// đặt được văn bản vào một segment"*. Mệnh đề đó **đúng lúc này và sẽ hết đúng** ở Epic 4 · 6
/// · 7 · 8; nó không hết đúng **lùi về quá khứ**, nên câu `UPDATE` chạy một lần này vẫn trung
/// thực mãi. Lý do đầy đủ ở doc-comment của chính hằng bước 11.
///
/// 🔵 **CẬP NHẬT 2026-08-19 (Story 3.1):** đích chuyển từ **11** lên **12** — bước
/// [`GLOSSARY_ENTRY_DDL`] (tầng Tác phẩm của Glossary, AD-18/AD-36). Câu *"mười bước, đích
/// là 11"* đã hết đúng, sửa tại chỗ. 🔴 **Cùng một hằng** với bước 4 của [`GLOBAL_MIGRATIONS`]
/// — xem "MỘT HẰNG, DÙNG CHO HAI THANG DI TRÚ" ở doc-comment của chính hằng đó; hai tầng của
/// Glossary phải cùng hình dạng THEO ĐỊNH NGHĨA, không nhờ hai chỗ tình cờ chép giống nhau.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.2):** đích chuyển từ **12** lên **13** — bước
/// [`GLOSSARY_CANDIDATE_DDL`] (bảng chờ ứng viên, AD-20/AD-36). Câu *"mười một bước, đích
/// là 12"* đã hết đúng, sửa tại chỗ. 🔴 **KHÁC** bước 4/12 của Glossary: bước này KHÔNG có
/// bước song sinh ở [`GLOBAL_MIGRATIONS`] — bảng ứng viên chỉ tồn tại ở tầng Tác phẩm
/// (§Never của story: "Bảng ứng viên ở `global.db`").
///
/// 🔵 **CẬP NHẬT 2026-08-22 (Story 3.5):** đích chuyển từ **13** lên **14** — bước
/// [`GLOSSARY_CANDIDATE_OCCURRENCE_CONTEXT_DDL`] (`occurrence_count`/`context_example` của
/// `glossary_candidate`). Câu *"mười hai bước, đích là 13"* đã hết đúng, sửa tại chỗ. Cùng
/// lý do bước 13: **KHÔNG** có bước song sinh ở [`GLOBAL_MIGRATIONS`].
///
/// 🔵 **CẬP NHẬT 2026-08-24 (Story 3.10):** đích chuyển từ **14** lên **15** — bước
/// [`GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL`] (giá trị `term_origin` thứ tư,
/// `file_import`, CÙNG một hằng với bước 5 của `global.db`). Câu *"mười ba bước, đích là
/// 14"* đã hết đúng, sửa tại chỗ.
/// 🔴 **Khối này VIẾT MUỘN — 2026-08-25, ở vòng rà Epic 3, không cùng lượt với Story 3.10.**
/// Lượt đó bump đúng doc-comment của [`GLOBAL_MIGRATIONS`] cho bước song sinh mà bỏ sót bộ
/// này, nên suốt ba ngày tiêu đề đọc *"mười ba bước, đích là 14"* trong khi mảng ngay dưới
/// có **14** mục và chạm `to_version` **15**. Đúng thứ rot mà đoạn ⚠️ phía trên đã gọi tên
/// một lần rồi (bắt ở code review 2026-08-11) — lần thứ hai, nên nó nay có một cổng thật:
/// `tests/segment_contract.rs::the_migration_doc_headers_state_the_target_their_array_reaches`.
/// Kỷ luật của người sửa đã hụt hai lần; một ca test thì không hụt.
///
/// 🔵 **CẬP NHẬT 2026-08-27 (Story 5.4):** đích chuyển từ **15** lên **16** — bước
/// [`WORK_STATUS_OVERRIDE_DDL`] (`work.status_override`, FR6). Câu *"mười bốn bước, đích là
/// 15"* đã hết đúng, sửa tại chỗ. **KHÔNG** có bước song sinh ở [`GLOBAL_MIGRATIONS`]: bảng
/// `work` chỉ tồn tại ở `project.db`.
///
/// 🔵 **CẬP NHẬT 2026-08-29 (Story 5.7):** đích chuyển từ **16** lên **17** — bước
/// [`CHAPTER_POSITION_DDL`] (vị trí làm việc của mỗi Chương, AD-3). Câu *"mười lăm bước,
/// đích là 16"* đã hết đúng, sửa tại chỗ. **KHÔNG** có bước song sinh ở
/// [`GLOBAL_MIGRATIONS`]: `chapter_position` chỉ tồn tại ở `project.db`.
///
/// ⚠️ **Mỗi bước một hằng, không gộp** — và đó là hệ quả của một ràng buộc kỹ thuật, ghi ra
/// thay vì giấu: `Migration::sql` là `&'static str`, và `concat!` (thứ duy nhất nối được
/// hai chuỗi ở **compile time** mà không thêm phụ thuộc) chỉ nhận **literal**, không
/// nhận một `const` đặt tên. Nối [`SCHEMA_MIGRATION_LOG_DDL`] (hằng **tái dùng** từ
/// `global.db`) với [`WORK_DDL`]/[`CHAPTER_DDL`] thành một chuỗi duy nhất buộc phải chép
/// lại nguyên văn của hằng kia — đúng thứ *"tái dùng, đừng viết lại"* cấm. Các bước tách
/// rời, mỗi bước một hằng, giữ **mỗi** DDL có **đúng một** nguồn sự thật, cùng khuôn
/// [`GLOBAL_MIGRATIONS`] đã tách `SCHEMA_MIGRATION_LOG_DDL` (bước 1) khỏi
/// `CONFIG_VALUE_DDL` (bước 2). "Mỗi bước một giao dịch" là bất biến sẵn có của
/// [`migrate`] — không AC nào đòi `work`/`chapter`/`segment` phải cùng một giao dịch
/// SQL với nhật ký di trú.
///
/// Không thêm bước cho một lược đồ chưa tồn tại — cùng luật với [`GLOBAL_MIGRATIONS`].
/// **Không** bảng TM/prompt/asset ở đây; mỗi epic còn lại mang bảng riêng của nó cùng lúc
/// với bước di trú cần nó. [`SEGMENT_DDL`] có mặt vì Story 2.1 dựng chính bảng đó, không vì
/// Epic 2 sẽ cần nó — cùng lý do bước 12 ([`GLOSSARY_ENTRY_DDL`]) và bước 13
/// ([`GLOSSARY_CANDIDATE_DDL`]) có mặt: Story 3.1/3.2 dựng đúng bảng mà mỗi story cần,
/// không phải một epic khác đoán trước nó.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ MỘT VẾT SẸO CÓ THẬT: `user_version = 4` ĐÃ TỒN TẠI TRÊN MÁY — Story 1.20
/// ─────────────────────────────────────────────────────────────────────────────
/// Bản đầu của Story 1.20 (2026-08-10) thêm **bước 4** đặt [`PINNED_ENTRY_DDL`] vào bộ
/// này, theo Quyết định #1 chốt ghim ở phạm vi Tác phẩm. Ngày 2026-08-11 Ice ký lại: ghim
/// chuyển sang `global.db` *(lý do đo được ghi ở doc-comment của chính DDL đó)*, và bước 4
/// **bị gỡ** — bộ này về đúng ba bước như trước story.
///
/// 🔴 Hệ quả **không** giấu: một `project.db` tạo ra trong khoảng giữa hai lượt ký mang
/// `user_version = 4`, tức **cao hơn target**. [`super::Store::open`] sẽ **từ chối mở** nó
/// bằng `store.schema_too_new` (AC7 của Story 1.7) — không hỏng im lặng, nhưng cũng không
/// mở được. Đo 2026-08-11: **6** thư mục `.atproj` ở trạng thái đó, tất cả là tạo tác thử
/// nghiệm của lượt nghiệm thu, và Ice chốt xoá chúng.
///
/// ⚠️ Số **4** vì thế là một số **đã cháy**: bước di trú kế tiếp của `project.db` phải
/// đánh số **5**, không được tái dùng 4. Doc-comment đầu module nói bằng chữ vì sao —
/// *"một bước như vậy là hai đường lược đồ khác nhau cho cùng một số, và chúng sẽ rẽ nhau
/// ở máy người dùng chứ không ở đây"*. Đây là chỗ mệnh đề đó thành một ràng buộc thật.
///
/// 🔴 [`validate_strictly_increasing`] **KHÔNG** bắt được lỗi tái dùng số 4 — `[1, 2, 3, 4]`
/// là một danh sách tăng dần nghiêm ngặt hoàn hảo, và nó sẽ đi qua **mọi** cổng hiện có mà
/// không một dòng đỏ nào. Cổng thật là
/// `tests/segment_contract.rs::the_project_migration_set_never_reuses_the_burned_number_four`,
/// dựng ở Story 2.1 để đóng món nợ `deferred-work.md:1169-1180`.
///
/// ⚠️ **HỆ QUẢ của việc nâng target lên 5, ghi ra vì nó đổi hành vi trên dữ liệu có thật:**
/// một `project.db` mang `user_version = 4` trước lượt này bị [`super::Store::open`] **từ
/// chối** (4 > target 3); sau lượt này nó **mở được** và di trú thẳng lên 5 (4 < target 5),
/// mang theo một bảng `pinned_entry` mồ côi mà `project.db` không còn dùng tới. Vô hại về
/// dữ liệu, và cả **6** thư mục ở trạng thái đó đã bị Ice xoá ngày 2026-08-11 — nên đây là
/// một ghi chép để không ai phải chẩn đoán lại, không phải một bản vá phải viết.
pub const PROJECT_MIGRATIONS: &[Migration] = &[
    Migration {
        to_version: 1,
        sql: SCHEMA_MIGRATION_LOG_DDL,
    },
    Migration {
        to_version: 2,
        sql: WORK_DDL,
    },
    Migration {
        to_version: 3,
        sql: CHAPTER_DDL,
    },
    // 🔴 **5, KHÔNG phải 4** — số 4 đã cháy. Xem vết sẹo ở doc-comment ngay trên.
    Migration {
        to_version: 5,
        sql: SEGMENT_DDL,
    },
    // Story 2.2 — cột bản dịch. `ALTER TABLE`, không sửa `SEGMENT_DDL`; lý do đầy đủ ở
    // doc-comment của [`SEGMENT_TARGET_TEXT_DDL`].
    Migration {
        to_version: 6,
        sql: SEGMENT_TARGET_TEXT_DDL,
    },
    // Story 2.5 — máy trạng thái AD-31: cột `segment.status` + bảng `segment_version`.
    // 🔴 **7, không phải 5** — 5 và 6 đã tiêu. Lý do đầy đủ ở doc-comment của
    // [`SEGMENT_STATUS_AND_VERSION_DDL`].
    Migration {
        to_version: 7,
        sql: SEGMENT_STATUS_AND_VERSION_DDL,
    },
    // Story 2.5c — co cat bo cau khoi ban dich (FR133): cot `segment.is_omitted`.
    // 🔴 **8, khong phai 5** — 5, 6 va 7 da tieu. Ly do day du o doc-comment cua
    // [`SEGMENT_OMITTED_DDL`].
    Migration {
        to_version: 8,
        sql: SEGMENT_OMITTED_DDL,
    },
    // Story 2.5d — co ket doan cua BAN DICH (FR134/AD-46): cot
    // `segment.is_target_paragraph_end`, cong mot cau `UPDATE` backfill bang co nguon.
    // 🔴 **9, khong phai 5** — 5, 6, 7 va 8 da tieu. Buoc DAU TIEN cua kho mang DDL + DML
    // trong mot hang; ly do day du o doc-comment cua [`SEGMENT_TARGET_PARAGRAPH_END_DDL`].
    Migration {
        to_version: 9,
        sql: SEGMENT_TARGET_PARAGRAPH_END_DDL,
    },
    // Story 2.6 — duong DOC lich su phien ban (FR101): index tren `segment_version`.
    // 🔴 **10, khong phai 5** — 5, 6, 7, 8 va 9 da tieu. Buoc DAU TIEN cua kho khong them
    // mot cot nao: no dong mot mon no CO CHU ma buoc 7 ghi bang chu luc dung bang
    // `segment_version`. Ly do day du o doc-comment cua [`SEGMENT_VERSION_INDEX_DDL`].
    Migration {
        to_version: 10,
        sql: SEGMENT_VERSION_INDEX_DDL,
    },
    // Story 2.7 — xuat xu ban dich cap segment (FR117/AD-47): cot
    // `segment.translation_origin`, cong mot cau `UPDATE` backfill cho cac hang DA KY.
    // 🔴 **11, khong phai 5** — 5, 6, 7, 8, 9 va 10 da tieu. Ly do day du o doc-comment cua
    // [`SEGMENT_TRANSLATION_ORIGIN_DDL`].
    Migration {
        to_version: 11,
        sql: SEGMENT_TRANSLATION_ORIGIN_DDL,
    },
    // Story 3.1 -- tang Tac pham cua Glossary (AD-18/AD-36): bang glossary_entry, CUNG mot
    // hang voi buoc 4 cua global.db. Xem doc-comment cua GLOSSARY_ENTRY_DDL.
    // 12, khong phai 5 -- 5, 6, 7, 8, 9, 10 va 11 da tieu.
    Migration {
        to_version: 12,
        sql: GLOSSARY_ENTRY_DDL,
    },
    // Story 3.2 -- bang cho ung vien glossary_candidate (AD-20/AD-36), TACH HAN khoi
    // glossary_entry va KHONG co buoc song sinh o GLOBAL_MIGRATIONS. Xem doc-comment cua
    // GLOSSARY_CANDIDATE_DDL.
    // 13, khong phai 5 -- 5, 6, 7, 8, 9, 10, 11 va 12 da tieu.
    Migration {
        to_version: 13,
        sql: GLOSSARY_CANDIDATE_DDL,
    },
    // Story 3.5 -- hai cot occurrence_count/context_example cua glossary_candidate, cung
    // KHONG co buoc song sinh o GLOBAL_MIGRATIONS. Xem doc-comment cua
    // GLOSSARY_CANDIDATE_OCCURRENCE_CONTEXT_DDL.
    // 14, khong phai 5 -- 5, 6, 7, 8, 9, 10, 11, 12 va 13 da tieu.
    Migration {
        to_version: 14,
        sql: GLOSSARY_CANDIDATE_OCCURRENCE_CONTEXT_DDL,
    },
    // Story 3.10 -- gia tri term_origin thu tu, 'file_import' (FR49/NFR9), CUNG mot hang voi
    // buoc 5 cua global.db. Xem doc-comment cua GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL.
    // 15, khong phai 5 -- 5, 6, 7, 8, 9, 10, 11, 12, 13 va 14 da tieu.
    Migration {
        to_version: 15,
        sql: GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL,
    },
    // Story 5.4 -- ghi de trang thai vong doi tang Tac pham (FR6): cot work.status_override.
    // ALTER TABLE rieng, khong sua WORK_DDL tai cho -- vet seo so 4. Khong co buoc song sinh
    // o GLOBAL_MIGRATIONS: `work` chi ton tai o project.db.
    // 16, khong phai 5 -- 5..15 da tieu.
    Migration {
        to_version: 16,
        sql: WORK_STATUS_OVERRIDE_DDL,
    },
    // Story 5.7 -- vi tri lam viec cua moi Chuong (AD-3): bang chapter_position rieng, KHONG
    // mot cot tren `chapter`. Xem doc-comment cua CHAPTER_POSITION_DDL.
    // 17, khong phai 5 -- 5..16 da tieu.
    Migration {
        to_version: 17,
        sql: CHAPTER_POSITION_DDL,
    },
];

/// Lược đồ bảng `library_work` — **bước 1, VÀ DUY NHẤT, MÃI MÃI, của `library-index.db`** —
/// Story 5.2, AD-8.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT BƯỚC DUY NHẤT — VÀ VÌ SAO NÓ KHÔNG BAO GIỜ CÓ BƯỚC 2
/// ─────────────────────────────────────────────────────────────────────────────
/// [`GLOBAL_MIGRATIONS`]/[`PROJECT_MIGRATIONS`] thêm một bước MỚI mỗi khi lược đồ đổi — di
/// trú chỉ tiến, không bao giờ sửa một hằng đã chạy trên đĩa thật (AD-30, và vết sẹo số 4 của
/// [`PROJECT_MIGRATIONS`] ghi lại cái giá của việc phá luật đó). `library-index.db` là kho
/// **DẪN XUẤT** (AD-8): nó không giữ một byte dữ liệu nào mà `.atproj` không còn khai, nên
/// không có gì để **DI TRÚ** — chỉ có gì để **DỰNG LẠI**. Khi hình dạng bảng này đổi, hằng
/// NÀY được viết lại TẠI CHỖ và `to_version` TĂNG — ngược hẳn quy tắc "sửa hằng cũ tại chỗ là
/// hai lược đồ cho cùng một số" mà mọi bảng khác trong tệp này phải theo: ở ĐÓ viết lại tại
/// chỗ là lỗi vì kho đó **phải** di trú; Ở ĐÂY viết lại tại chỗ là **đúng** vì kho này không
/// bao giờ di trú. [`super::Store::open`] không bao giờ chạm nhánh từ chối mở
/// ([`StoreError::SchemaTooNew`]) cho kho này: [`crate::core::library::indexer::Indexer::open`]
/// so `PRAGMA user_version` với đích TRƯỚC khi gọi `Store::open`, và XOÁ tệp (cả hai chiều
/// lệch) thay vì để `Store::open` quyết — nên bước 1 luôn chạy trên một tệp RỖNG, không bao
/// giờ nửa chừng qua một `ALTER TABLE`.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 CẢ `schema_migration_log` LẪN `library_work` TRONG CÙNG MỘT BƯỚC — VÌ SAO BẮT BUỘC
/// ─────────────────────────────────────────────────────────────────────────────
/// [`migrate`] (dùng CHUNG cho mọi kho) ghi một hàng vào `schema_migration_log` NGAY SAU khi
/// chạy SQL của mỗi bước — kể cả bước 1. Bảng đó vì thế phải tồn tại TRƯỚC câu `INSERT` đó
/// chạy, và vì đây là bước DUY NHẤT của kho này (không có bước 2 để "đợi" bảng nghiệp vụ, khác
/// [`GLOBAL_MIGRATIONS`]/[`PROJECT_MIGRATIONS`] nơi bước 1 chỉ mang bảng nhật ký), cả hai
/// `CREATE TABLE` phải nằm trong CÙNG một hằng.
///
/// ⚠️ Văn bản `schema_migration_log` dưới đây **PHẢI TRÙNG BYTE** với [`SCHEMA_MIGRATION_LOG_DDL`]
/// — không tái dùng được hằng đó trực tiếp vì `Migration::sql` đòi một `&'static str` và
/// `concat!` chỉ nhận literal, không nhận đường dẫn hằng (cùng ràng buộc mà doc-comment của
/// [`GLOSSARY_CANDIDATE_OCCURRENCE_CONTEXT_DDL`] đã ghi).
///
/// 🔵 **SỬA (vòng rà ba lớp, P4) — con số cũ SAI, đếm lại cho đúng.** Bản trước viết *"bản
/// chép tay THỨ BA của cùng DDL đó, sau bước 1 của `GLOBAL_MIGRATIONS` và bước 1 của
/// `PROJECT_MIGRATIONS`"* — sai: bước 1 của cả hai bộ đó chỉ **tham chiếu** hằng
/// [`SCHEMA_MIGRATION_LOG_DDL`] bằng tên (`sql: SCHEMA_MIGRATION_LOG_DDL`), không chép tay lại
/// văn bản của nó. Đếm bằng `grep -n "CREATE TABLE schema_migration_log"` trên chính tệp này
/// (2026-08-27): văn bản `CREATE TABLE schema_migration_log (…)` xuất hiện LITERAL đúng **hai**
/// lần — hằng gốc [`SCHEMA_MIGRATION_LOG_DDL`] và hằng NÀY. ⇒ Đây là bản chép tay **THỨ HAI**,
/// không phải thứ ba.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// TỪNG CỘT, VÀ NÓ NEO VÀO ĐÂU
/// ─────────────────────────────────────────────────────────────────────────────
/// - `work_id` — **khoá chính**, trùng [`crate::core::library::meta::WorkMeta::work_id`].
///   `PRIMARY KEY` trần (không `AUTOINCREMENT` — không có rowid nào cần giữ ổn định qua thời
///   gian; kho này bị XOÁ TOÀN BỘ và dựng lại mỗi lần lệch phiên bản).
///
///   🔵 **SỬA (2026-08-27, vòng rà THỨ HAI P5) — mệnh đề cũ HẾT ĐÚNG, và VẾ ĐÃ MẤT phải nói
///   thẳng, không chỉ đổi câu mô tả.** Bản trước viết *"mỗi lượt `Indexer::rebuild` xoá sạch
///   bảng rồi chèn lại"* và gọi `PRIMARY KEY` này là *"cơ chế phát hiện trùng của §Boundaries
///   — SQLite tự từ chối hàng thứ hai"*. Từ Story 5.3, `rebuild` là ĐỐI CHIẾU
///   (`INSERT … ON CONFLICT (work_id) DO UPDATE`, xem `core/library/indexer.rs`), không còn
///   xoá-sạch-ghi-lại — và hệ quả là **lưới chắn SQL đó không còn tồn tại**: một hàng trùng
///   `work_id` lọt tới câu `INSERT` này sẽ đi vào nhánh `DO UPDATE` và **ghi đè ÊM ÁI**, không
///   nổ lỗi ràng buộc nữa. `PRIMARY KEY` giờ chỉ còn giữ vai trò cấu trúc (đảm bảo tại-mọi-
///   thời-điểm không có HAI hàng cùng `work_id` cùng tồn tại), KHÔNG còn là nơi phát hiện
///   trùng — việc phát hiện trùng `.atproj` cùng `work_id` (§Boundaries) nay diễn ra HOÀN
///   TOÀN ở tầng Rust, TRƯỚC khi chạm SQL: `Indexer::rebuild` tự giữ một `HashMap` `first_seen`
///   và chỉ đưa ĐÚNG MỘT `(work_id, meta)` (mục đầu theo thứ tự quét đã sắp) vào câu UPSERT
///   mỗi lượt; các mục trùng sau bị gạt ra `WorkIdConflict` trước khi tới `store.write`, nên
///   SQL không bao giờ THẤY một work_id thứ hai để mà từ chối.
/// - `atproj_path` — đường dẫn **TUYỆT ĐỐI trên máy này** tới `<Tên>.atproj/`. Khác
///   `meta.json`, nơi AC5 của Story 1.15 **cấm** đường tuyệt đối (nó theo `.atproj` khi Tác
///   phẩm bị copy sang máy khác) — chỉ mục thì **không** theo: nó là dẫn xuất **cục bộ**, và
///   Library cần biết Tác phẩm nằm ở đâu trên **máy này** để mở nó.
/// - Sáu cột kế (`name`/`source_lang`/`genre`/`created_at`/`updated_at`/`chapter_count`)
///   — **đúng** các trường tương ứng của [`crate::core::library::meta::WorkMeta`], không hơn
///   không kém.
///   🔵 **SỬA (2026-08-28, Story 5.5) — mệnh đề "không cột tiến độ (chủ Story 5.5)" đã HẾT
///   ĐÚNG: đây CHÍNH LÀ story đó, và cột `chapter_done_count` dưới đây là bề mặt của nó.**
///   🔵 **SỬA (2026-08-28, Story 5.6) — "không `cover` (chủ Story 5.6)" đã HẾT ĐÚNG là "còn
///   CHỜ Story 5.6 quyết".** Đây CHÍNH LÀ story đó, và quyết định là KHÔNG thêm cột này. Đo
///   2026-08-28: `grep -rni cover src-tauri/src src` ⇒ 3 kết quả, 0 cái là một trường dữ liệu
///   (hai là chữ "covering index", một là chính câu đã sửa này); `grep -n "bìa"
///   _bmad-output/planning-artifacts/epics.md` ⇒ 0 story nào mở một đường cho người dùng ĐẶT
///   ảnh bìa (FR3 chỉ ghi "ảnh bìa (tuỳ chọn)", không AC nào dựng đường chọn tệp). Một cột
///   luôn `NULL` cho một giao diện luôn vẽ biểu diễn thay thế là đúng thứ Story 1.7 §Completion
///   Notes #3 và §Never của Story 5.1 cấm — xem §Design Notes "Vì sao KHÔNG thêm cột `cover` ở
///   lượt này" của `5-6-luoi-tac-pham-loc-va-sap-xep.md`. Món nợ chuyển sang chủ MỚI: story
///   ĐẦU TIÊN mở đường cho người dùng ĐẶT một ảnh bìa — story đó chưa tồn tại trong
///   `epics.md`, nên người quyết định kế tiếp là **Ice**, không một tên story giả cho có.
/// 🔵 **SỬA (2026-08-27, phán quyết Ice #1, LẬT quyết định 5.3) — cột `orphaned` đã BỊ GỠ,
/// `to_version` 2 → 3.** Story 5.3 từng thêm cột này ngay tại đây (`to_version` 1 → 2) vì lúc
/// đó cờ mồ côi được coi là một mẩu trạng thái của CHÍNH chỉ mục dẫn xuất. Ice chốt lại
/// 2026-08-27: cờ mồ côi là **dữ liệu người dùng** ("người dùng CHỌN giữ lại một lời nhắc",
/// không phải một sự thật suy ra được từ đĩa) — nó KHÔNG thuộc về một kho tự xưng là "xoá đi
/// dựng lại vô hại" (AD-8). Cờ mồ côi nay sống ở bảng `library_orphan` của `global.db` (xem
/// [`LIBRARY_ORPHAN_DDL`]), và `library_work` quay lại ĐÚNG nghĩa cũ: "những gì đang có mặt
/// trên đĩa ngay bây giờ" — dẫn xuất TRỌN VẸN, không hàng nào sống sót một lượt xoá-dựng-lại.
/// Xem §Spec Change Log + §Design Notes của `5-3-quet-lai-thu-muc.md` cho lý lẽ đầy đủ và
/// phương án đã cân.
///
/// 🔵 **SỬA (2026-08-27, Story 5.4) — dòng ngay trên "không cột trạng thái vòng đời (chủ
/// Story 5.4)" đã HẾT ĐÚNG: đây CHÍNH LÀ story đó, và hai cột dưới đây là bề mặt của nó.**
/// `to_version` 3 → 4. Hai cột mới:
/// - `status TEXT` — cho phép `NULL`. `NULL` **không** nghĩa là `NotStarted`; nó nghĩa là
///   **CHƯA BIẾT** — một `meta.json` v1 (viết trước khi story này tồn tại) không mang trường
///   `status`, và [`crate::core::library::meta::WorkMeta::status`] đọc nó ra `None`, đi
///   nguyên vẹn xuống cột này qua chính lượt UPSERT của [`crate::core::library::indexer::Indexer::rebuild`].
///   Một Tác phẩm **đã dịch xong** không được phép hiện *"Chưa bắt đầu"* chỉ vì chỉ mục chưa
///   từng thấy trạng thái thật của nó — xem §Design Notes "Vì sao `Option<String>`" của
///   story.
/// - `status_is_override INTEGER NOT NULL DEFAULT 0` — `1` ⇔ giá trị ở `status` đến từ
///   `work.status_override` (ghi đè thủ công), `0` ⇔ giá trị suy ra tự động (hoặc `status IS
///   NULL`, tức chưa biết). `NOT NULL DEFAULT 0` là giá trị AN TOÀN cho một hàng vừa UPSERT
///   lần đầu — cùng khuôn `occurrence_count`/`is_omitted` ở nơi khác trong kho: SQLite không
///   có kiểu boolean, `INTEGER` 0/1 là quy ước, không một cờ `boolean` song song với `status`
///   (§Always: *"ghi đè là `NULL`-hoặc-giá-trị, không phải một cờ boolean riêng"* — câu đó
///   nói về `work.status_override`; ở ĐÂY, tại kho dẫn xuất, cờ RIÊNG là cần thiết vì
///   `status` đã bị GIẢN LƯỢC về một chuỗi duy nhất, không còn phân biệt được "suy ra" với
///   "ghi đè" nếu không có nó — Library phải lọc/hiện dấu phân biệt mà KHÔNG mở SQLite của
///   từng Tác phẩm).
/// 🔵 **NÂNG (2026-08-28, Story 5.5) — `to_version` 4 → 5.** Cột mới:
/// - `chapter_done_count INTEGER` — cho phép `NULL`, đúng khuôn `status`/`status_is_override`
///   ngay trên. `NULL` **không** nghĩa là `0`; nó nghĩa là **CHƯA BIẾT** — một `meta.json`
///   v1/v2 (viết trước khi story này tồn tại) không mang trường `chapter_done_count`, và
///   [`crate::core::library::meta::WorkMeta::chapter_done_count`] đọc nó ra `None`, đi nguyên
///   vẹn xuống cột này qua chính lượt UPSERT của [`crate::core::library::indexer::Indexer::rebuild`].
///   Một Tác phẩm đã dịch xong nhiều Chương không được phép hiện `0 / n` chỉ vì chỉ mục chưa
///   từng thấy tiến độ thật của nó — xem §Design Notes "Vì sao `Option<u32>` chứ không `u32`"
///   của `5-5-tien-do-tac-pham.md`.
pub const LIBRARY_WORK_DDL: &str = "\
CREATE TABLE schema_migration_log (
  version     INTEGER PRIMARY KEY,
  applied_at  TEXT NOT NULL,
  app_version TEXT NOT NULL
);
CREATE TABLE library_work (
  work_id             TEXT PRIMARY KEY,
  atproj_path         TEXT NOT NULL,
  name                TEXT NOT NULL,
  source_lang         TEXT NOT NULL,
  genre               TEXT NOT NULL,
  created_at          TEXT NOT NULL,
  updated_at          TEXT NOT NULL,
  chapter_count       INTEGER NOT NULL,
  status              TEXT,
  status_is_override  INTEGER NOT NULL DEFAULT 0,
  chapter_done_count  INTEGER
);";

/// Bộ di trú của `library-index.db` — **đúng MỘT bước, mãi mãi**. Xem doc-comment của
/// [`LIBRARY_WORK_DDL`] cho lý do đây KHÔNG phải một thiếu sót: kho dẫn xuất không di trú
/// (AD-8), nó bị xoá-và-dựng-lại khi lược đồ đổi, không bao giờ được thêm bước 2.
///
/// 🔵 **NÂNG 2026-08-27 (Story 5.3): `to_version` 1 → 2** — cột `orphaned` thêm vào
/// [`LIBRARY_WORK_DDL`] (viết lại TẠI CHỖ, không một bước di trú thứ hai). ~~Mọi
/// `library-index.db` ở `to_version` 1 bị `Indexer::open` xoá-và-dựng-lại…~~
///
/// 🔵 **NÂNG LẦN HAI (2026-08-27, phán quyết Ice #1): `to_version` 2 → 3** — cột `orphaned`
/// vừa thêm ở bản nâng trên đã bị GỠ (xem doc-comment của [`LIBRARY_WORK_DDL`]). Đây vẫn là
/// một lượt VIẾT LẠI TẠI CHỖ đúng luật của kho dẫn xuất — không một bước di trú thứ hai/ba.
/// Mọi `library-index.db` ở `to_version` 1 HOẶC 2 bị `Indexer::open` xoá-và-dựng-lại như một
/// tệp lệch phiên bản bình thường. ⚠️ **Cửa sổ này chưa phát hành** (Story 5.3 mới đi vào
/// nhánh chính cùng ngày) nên không `library-index.db` thật nào ngoài máy dev từng mang cột
/// `orphaned`; hàng mồ côi (nếu có) trong một tệp `to_version = 2` cục bộ sẽ biến mất cùng
/// tệp bị xoá, KHÔNG được chuyển sang `global.db` — đúng lời hứa gốc của AD-8 ("xoá chỉ
/// mục là an toàn, chỉ mất MỘT LỜI NHẮC, không mất dữ liệu người dùng thật": bản thân `.atproj`
/// trên đĩa không hề bị chạm). Không mất dữ liệu người dùng THẬT, chỉ mất chính chỉ mục.
///
/// 🔵 **NÂNG LẦN BA (2026-08-27, Story 5.4): `to_version` 3 → 4** — hai cột `status`/
/// `status_is_override` thêm vào [`LIBRARY_WORK_DDL`] (viết lại TẠI CHỖ, không một bước di
/// trú thứ tư). Mọi `library-index.db` ở `to_version` 1, 2, HOẶC 3 bị `Indexer::open`
/// xoá-và-dựng-lại như một tệp lệch phiên bản bình thường — không mất dữ liệu người dùng
/// thật (kho này dẫn xuất trọn vẹn từ `.atproj`, AD-8).
///
/// 🔵 **NÂNG LẦN TƯ (2026-08-28, Story 5.5): `to_version` 4 → 5** — cột `chapter_done_count`
/// thêm vào [`LIBRARY_WORK_DDL`] (viết lại TẠI CHỖ, không một bước di trú thứ năm). Mọi
/// `library-index.db` ở `to_version` 1..4 bị `Indexer::open` xoá-và-dựng-lại như một tệp lệch
/// phiên bản bình thường — không mất dữ liệu người dùng thật.
pub const LIBRARY_INDEX_MIGRATIONS: &[Migration] = &[Migration {
    to_version: 5,
    sql: LIBRARY_WORK_DDL,
}];

/// Phiên bản cao nhất mà một bộ di trú đạt tới. Bộ rỗng ⇒ 0.
///
/// 🔴 Chỉ đáng tin **sau** [`validate_strictly_increasing`]: hàm này tin `.last()` là
/// lớn nhất, và một danh sách khai lộn thứ tự làm giả định đó sai mà không gì báo.
pub(crate) fn target_version(migrations: &[Migration]) -> u32 {
    migrations.last().map(|m| m.to_version).unwrap_or(0)
}

/// Xác nhận bộ di trú **tăng dần nghiêm ngặt** — bất biến của chính bộ di trú.
///
/// ⚠️ Phải chạy TRƯỚC khi [`target_version`] được tin, không chỉ trước khi [`migrate`]
/// chạy: [`super::Store::open`] dùng `target` để quyết định từ chối mở (AC7) trước cả
/// bước sao lưu và di trú — một `target` tính sai từ một danh sách lộn thứ tự làm quyết
/// định đó sai ở đúng bước không được phép sai.
pub(crate) fn validate_strictly_increasing(
    migrations: &[Migration],
    kind: StoreKind,
) -> Result<(), StoreError> {
    let mut previous = 0u32;
    for m in migrations {
        if m.to_version <= previous {
            return Err(StoreError::OpenFailed {
                store: kind,
                detail: format!(
                    "migration list is not strictly increasing: {} follows {}",
                    m.to_version, previous
                ),
            });
        }
        previous = m.to_version;
    }
    Ok(())
}

/// Đọc `PRAGMA user_version`.
///
/// 🔴 **Chỉ đọc**, và nó là bước thứ hai của [`super::Store::open`] chứ không phải bước
/// thứ tư — xem doc-comment của `core::store`. Đảo thứ tự là AC7 trượt im lặng.
pub(crate) fn read_user_version(conn: &Connection, kind: StoreKind) -> Result<u32, StoreError> {
    let raw: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("read PRAGMA user_version: {e}"),
        })?;

    // `user_version` là INTEGER 32-bit có dấu trong header SQLite. Một số âm ở đó nghĩa
    // là tệp không do ứng dụng này viết ra; đừng ép kiểu im lặng thành một số dương
    // khổng lồ rồi kết luận "lược đồ quá mới".
    u32::try_from(raw).map_err(|_| StoreError::OpenFailed {
        store: kind,
        detail: format!("PRAGMA user_version is {raw}, expected a non-negative integer"),
    })
}

/// Sao lưu **trước bước di trú đầu tiên**, và chỉ khi đã có lược đồ (`from >= 1`).
///
/// Trình tự là hợp đồng, xem doc-comment của module: TRUNCATE → xác nhận `busy == 0` →
/// `fs::copy`. Tệp đích là `<tên>.db.bak-v<n>` **cạnh tệp gốc**, với `n` là phiên bản
/// **trước** khi di trú — cái tên nói được nó là bản sao của cái gì.
pub(crate) fn backup_before_migration(
    conn: &Connection,
    path: &Path,
    kind: StoreKind,
    from: u32,
) -> Result<(), StoreError> {
    let outcome = pragmas::wal_checkpoint(conn, "TRUNCATE", kind)?;

    // 🔴 `busy != 0` nghĩa là TRUNCATE **không chép hết** — tức `.db` vẫn thiếu phần nằm
    // trong WAL, tức bản sao sắp tạo ra là bản sao không đầy đủ. Không đi tiếp: một
    // bản sao lưu sai còn tệ hơn không có, vì nó làm người ta dám di trú.
    if outcome.busy != 0 {
        return Err(StoreError::OpenFailed {
            store: kind,
            detail: format!(
                "backup aborted: wal_checkpoint(TRUNCATE) reported busy={} log={} checkpointed={}",
                outcome.busy, outcome.log, outcome.checkpointed
            ),
        });
    }

    let mut name = path.file_name().unwrap_or_default().to_owned();
    name.push(format!(".bak-v{from}"));
    let target = path.with_file_name(name);

    std::fs::copy(path, &target).map_err(|e| StoreError::OpenFailed {
        store: kind,
        detail: format!("copy backup to {}: {e}", target.display()),
    })?;

    Ok(())
}

/// Chạy các bước di trú **chỉ tiến**, mỗi bước trong **một** giao dịch.
///
/// Trả về phiên bản sau khi xong. Một bước ném lỗi ⇒ giao dịch của **chính bước đó**
/// rollback và `user_version` giữ nguyên giá trị trước bước đó; các bước đã commit trước
/// nó thì ở lại — đúng nghĩa "chỉ tiến", và đó là lý do mỗi bước một giao dịch chứ không
/// phải cả loạt một giao dịch.
pub(crate) fn migrate(
    conn: &mut Connection,
    kind: StoreKind,
    from: u32,
    migrations: &[Migration],
) -> Result<u32, StoreError> {
    // ⚠️ Idempotent với lần kiểm ở `Store::open` (xem `validate_strictly_increasing`):
    // `migrate` không có cách nào biết chỗ gọi đã kiểm chưa, và cái giá của kiểm lại một
    // danh sách nhỏ mỗi lần mở là không đáng kể so với cái giá của một lần bỏ sót.
    validate_strictly_increasing(migrations, kind)?;

    let app_version = env!("CARGO_PKG_VERSION");
    let mut current = from;

    for m in migrations.iter().filter(|m| m.to_version > from) {
        let tx = conn.transaction().map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("begin transaction for migration {}: {e}", m.to_version),
        })?;

        tx.execute_batch(m.sql).map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("migration {} failed: {e}", m.to_version),
        })?;

        // ⚠️ `strftime` của SQLite, không phải đồng hồ của Rust: ISO-8601 UTC theo
        // Consistency Conventions mà không phải kéo `chrono`/`time` về cho một dòng.
        tx.execute(
            "INSERT INTO schema_migration_log (version, applied_at, app_version) \
             VALUES (?1, strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?2)",
            rusqlite::params![m.to_version, app_version],
        )
        .map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("log migration {}: {e}", m.to_version),
        })?;

        // ⚠️ `PRAGMA` không nhận tham số ràng buộc. Giá trị là `u32` của chính chương
        // trình, không bao giờ là dữ liệu người dùng.
        tx.execute_batch(&format!("PRAGMA user_version = {}", m.to_version))
            .map_err(|e| StoreError::OpenFailed {
                store: kind,
                detail: format!("set user_version to {}: {e}", m.to_version),
            })?;

        tx.commit().map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("commit migration {}: {e}", m.to_version),
        })?;

        current = m.to_version;
    }

    Ok(current)
}
