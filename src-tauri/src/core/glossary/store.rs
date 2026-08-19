//! SQL của `glossary_entry` + đúng MỘT hàm phơi ra module khác — Story 3.1, AD-18 · AD-36.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 MODULE MIỀN SỞ HỮU ĐIỀU KIỆN CHÈN — cố ý LỆCH tiền lệ `core/segment/`
//! ─────────────────────────────────────────────────────────────────────────────
//! `core/segment/` giữ logic thuần tách khỏi SQL (SQL ở `commands/`); ở đây điều kiện
//! chèn ("chỉ mục đã chốt" — AD-36) sống NGAY TRONG `core/glossary/`, cùng lớp với SQL nạp
//! dữ liệu. Tiền lệ đúng cho hình dạng này là `core/scope/store.rs`, không
//! `core/segment/**`: dữ liệu hai tầng phải phân giải RỒI MỚI lọc, và chỗ duy nhất biết cả
//! hai vế đó là chỗ vừa nạp cả hai tầng vừa gọi `ScopeResolver` — tách điều kiện chèn ra
//! một module khác nghĩa là module đó phải nạp lại dữ liệu hoặc nhận nó qua tham số, tức
//! dựng thêm một đường "biết cả hai tầng" thứ hai mà không ai xin phép.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 LỌC **SAU** KHI PHÂN GIẢI, KHÔNG TRƯỚC
//! ─────────────────────────────────────────────────────────────────────────────
//! Một mục *chờ chốt* ở tầng Tác phẩm che một mục *đã chốt* ở tầng Global ⇒ thuật ngữ đó
//! **không** đủ điều kiện chèn — lọc trước khi phân giải (vd. loại bỏ mục chưa chốt ở mỗi
//! tầng RỒI MỚI hợp hai tầng) sẽ để lộ mục Global bên dưới ra ngoài, tức chèn bản dịch
//! toàn cục cho đúng thuật ngữ người dùng vừa cố ý để ngỏ ở Tác phẩm này.
//! [`entries_eligible_for_injection`] gọi hai lượt [`load_tier`] rồi
//! `ScopeResolver::apply_override` TRƯỚC, rồi mới `filter(GlossaryEntry::is_confirmed)`
//! trên kết quả ĐÃ phân giải — không có đường nào khác trong hàm này lọc trước.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 CẬP NHẬT 2026-08-19 (lượt rà soát ba lớp, vá cuối) — `entries_eligible_for_injection`
//! TỰ NẠP HAI TẦNG, KHÔNG CÒN NHẬN `BTreeMap` ĐÃ NẠP SẴN
//! ─────────────────────────────────────────────────────────────────────────────
//! Bản trước nhận `global`/`work: &BTreeMap<..>` — tức chỗ gọi phải tự có sẵn kết quả của
//! [`load_tier`]. Cổng `glossary_boundary.rs::only_entries_eligible_for_injection_may_be_called_from_outside_glossary`
//! (thêm cùng lượt rà soát) cấm chính `load_tier` bị gọi ngoài `core/glossary/**` — hai
//! mệnh đề đó cùng đứng thì đường DUY NHẤT dựng được tham số cho hàm phơi ra DUY NHẤT lại
//! bị chính cổng bảo vệ hàm đó cấm. Ice ký nhận đây là lỗi trong chỉ thị vá, không phải một
//! đánh đổi có chủ. Sửa: `entries_eligible_for_injection` nhận `&Store` thẳng và tự gọi
//! `load_tier` bên trong — đúng khuôn `core::scope::store::load_global_config(store:
//! &Store)`, nơi một hàm vừa đọc kho vừa phân giải mà chỗ gọi không phải tự nạp gì trước.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! MODULE NÀY KHÔNG GÕ TÊN `ScopeKind` — cùng luật mọi module miền khác
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/scope_boundary.rs::FORBIDDEN_OUTSIDE_SCOPE` cấm token đó ngoài `core/scope/**`.
//! [`ScopeResolver::apply_override`] nhận `kind: &str` (Story 3.1 đóng
//! `deferred-work.md:272`) đúng để chỗ này gọi bằng một hằng literal
//! ([`GLOSSARY_SCOPE_KIND`]) mà không phải `use` kiểu đó.
//!
//! ⚠️ Mọi chuỗi trong `src-tauri/src/**` viết KHÔNG DẤU; doc-comment có dấu là hợp lệ.

use std::collections::BTreeMap;

use crate::core::scope::{ScopeError, ScopeResolver};
use crate::core::store::{ReadHandle, SqlError, SqlResult, SqlType, Store, StoreError, Transaction};

use super::entry::{Category, GlossaryEntry, TermOrigin};

/// Khoá dây của `ScopeKind::Glossary` (`core/scope/kinds.rs:162`), chép lại đây làm
/// literal — module này không được `use` `ScopeKind`.
const GLOSSARY_SCOPE_KIND: &str = "glossary";

/// Chèn một mục mới. `translation = None` ⇒ hàng vào ngay trạng thái *chờ chốt* (FR114).
///
/// # Lỗi
/// [`StoreError::WriteFailed`] nếu `source_term` đã tồn tại (`UNIQUE INDEX
/// idx_glossary_entry_source_term`), hoặc `translation` là `Some("")`/khoảng trắng
/// (`CHECK` của `GLOSSARY_ENTRY_DDL`) — cả hai đều là lỗi giao dịch SQLite lan qua
/// `Store::write`, không phải một nhánh được kiểm tay ở đây.
pub fn insert_entry(
    store: &Store,
    source_term: &str,
    translation: Option<&str>,
    note: &str,
    category: Category,
    term_origin: TermOrigin,
) -> Result<i64, StoreError> {
    // 🔴 CẮT KHOẢNG TRẮNG BIÊN, KHÔNG HẠ CHỮ THƯỜNG, KHÔNG CHUẨN HOÁ UNICODE — Story 3.1.
    //
    // Chỉ trim: `" 慕容"` và `"慕容"` không được thành hai hàng dưới một chỉ mục tự xưng
    // là "một thuật ngữ, một mục" (`idx_glossary_entry_source_term`) — trim ở đây làm
    // chúng va vào ĐÚNG một `UNIQUE` và lượt chèn thứ hai bị từ chối thay vì âm thầm tạo
    // ra một mục trùng có hình dạng khác. `str::trim()` của Rust cắt theo thuộc tính
    // Unicode `White_Space` — cùng tập ký tự mà `CHECK` hai tham số của
    // `GLOSSARY_ENTRY_DDL` chặn (tab · xuống dòng · NBSP · dấu cách biểu ý U+3000, …), nên
    // hai lớp phòng thủ nói cùng một ngôn ngữ.
    //
    // KHÔNG hạ chữ thường: `API` ≠ `api` có nghĩa trong tiếng Anh — cùng luật mà
    // `AGENTS.md` đã khoá cho từ điển ("hạ chữ thường là THÊM một khoá, không THAY khoá
    // gốc"). KHÔNG chuẩn hoá Unicode (NFC/NFKC, …): chính sách chuẩn hoá thuật ngữ là
    // quyết định của Story 3.4 (khớp thuật ngữ theo ngôn ngữ), không phải của story này —
    // đoán trước nó ở đây là đóng băng một lựa chọn chưa ai ký.
    let source_term = source_term.trim().to_owned();
    let translation = translation.map(|t| t.trim().to_owned());
    let note = note.to_owned();
    let category = category.as_str();
    let term_origin = term_origin.as_str();

    store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO glossary_entry
                (source_term, translation, note, category, term_origin, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            (&source_term, &translation, &note, &category, &term_origin),
        )?;
        Ok(tx.last_insert_rowid())
    })
}

/// Ghi `translation` cho hàng `id` — dùng được ở **CẢ HAI** chiều hợp lệ, không chỉ chiều
/// *chờ chốt → đã chốt*:
/// 1. **Chốt lần đầu** (FR114): `translation` cũ là `NULL`, mới là một chuỗi không rỗng.
/// 2. **Sửa một mục ĐÃ chốt** sang bản dịch khác (Story 3.9, *"sửa có hiệu lực ngay"*):
///    `translation` cũ VÀ mới đều không rỗng. Đây là hành vi **ĐÚNG**, không phải một lỗ
///    hổng — đừng "sửa" nó thành chỉ-nhận-`NULL`-cũ. Vòng đời một chiều của AD-36 cấm
///    đúng MỘT chiều: đã chốt **lùi về** `NULL`. Nó không cấm đổi giá trị đã chốt sang một
///    giá trị đã chốt khác — trigger `glossary_entry_lifecycle_is_one_way` chỉ khớp
///    `WHEN OLD.translation IS NOT NULL AND NEW.translation IS NULL`, và cả hai ca trên
///    đều không rơi vào nhánh đó.
///
/// Chiều **duy nhất** bị cấm — đã chốt → `NULL` — bị trigger
/// `glossary_entry_lifecycle_is_one_way` từ chối ở tầng SQL, không phải một kiểm tra ở đây.
///
/// # Lỗi
/// [`StoreError::WriteFailed`] nếu `translation` là chuỗi rỗng/khoảng trắng (`CHECK`).
///
/// ⚠️ `id` không khớp hàng nào ⇒ câu `UPDATE` chạy **0 hàng** và vẫn trả `Ok(())` — cùng
/// khuôn `delete_value` ("xoá một khoá không tồn tại là THÀNH CÔNG"). Rủi ro cho chỗ gọi
/// SẢN PHẨM đầu tiên: một lượt chốt nhắm vào một `id` đã bị xoá (đua với một thao tác xoá
/// khác, hay một tham số cũ còn kẹt lại) sẽ **im lặng không làm gì** thay vì báo lỗi —
/// đúng lớp *"rỗng im lặng"* mà `AGENTS.md` liệt vào Known pitfalls trung tâm của dự án.
/// Story 3.1 không đóng rủi ro này (không có chỗ gọi sản phẩm nào ở story này để nghiệm
/// thu một phương án). **Chủ: Story 3.3** — story đầu tiên dựng bề mặt IPC chạm tới hàm
/// này; đọc `glossary_contract.rs::confirming_an_unknown_id_succeeds_and_changes_nothing`
/// trước khi quyết định có cần đếm số hàng đổi hay không.
pub fn confirm_translation(store: &Store, id: i64, translation: &str) -> Result<(), StoreError> {
    // Cùng lý do cắt khoảng trắng biên đã ghi ở `insert_entry` — chốt qua đường này cũng
    // phải không tạo ra một bản dịch mang khoảng trắng thừa mà `insert_entry` đã cấm.
    let translation = translation.trim().to_owned();

    store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE glossary_entry SET translation = ?1 WHERE id = ?2",
            (&translation, id),
        )?;
        Ok(())
    })
}

/// Nạp toàn bộ một tầng (một `Store`, tức một `global.db` hoặc `project.db`), khoá theo
/// `source_term` — hình dạng `BTreeMap` mà `ScopeResolver::apply_override` đòi ở tham số
/// `global`/`work`.
pub fn load_tier(store: &Store) -> Result<BTreeMap<String, GlossaryEntry>, StoreError> {
    store.read(|conn: ReadHandle<'_>| {
        let mut stmt = conn.prepare(
            "SELECT id, source_term, translation, note, category, term_origin, created_at
             FROM glossary_entry
             ORDER BY source_term",
        )?;
        let mut rows = stmt.query([])?;

        let mut out = BTreeMap::new();
        while let Some(row) = rows.next()? {
            let source_term: String = row.get(1)?;
            let category_raw: String = row.get(4)?;
            let term_origin_raw: String = row.get(5)?;

            let entry = GlossaryEntry {
                id: row.get(0)?,
                source_term: source_term.clone(),
                translation: row.get(2)?,
                note: row.get(3)?,
                category: decode_category(4, &category_raw)?,
                term_origin: decode_term_origin(5, &term_origin_raw)?,
                created_at: row.get(6)?,
            };
            out.insert(source_term, entry);
        }
        Ok(out)
    })
}

/// `category` trên đĩa đã đi qua `CHECK (category IN (…))` của `GLOSSARY_ENTRY_DDL` ở mọi
/// lượt `INSERT`/`UPDATE` — một chuỗi lạ ở đây chỉ xảy ra nếu đĩa bị sửa ngoài đường của
/// module này (một bản ứng dụng cũ hơn, một lượt sửa tay `.db`, một lỗi di trú tương lai).
///
/// 🔴 **TRẢ LỖI, KHÔNG RƠI VỀ MỘT GIÁ TRỊ MẶC ĐỊNH** — và đặc biệt không rơi về giá trị
/// TRÔNG ĐÁNG TIN NHẤT. Bản trước của hàm này rơi `term_origin` lạ về
/// [`TermOrigin::Manual`] — đúng NGƯỢC CHIỀU: `Manual` nghĩa là *"người dùng tự gõ"*, xuất
/// xứ đáng tin nhất trong ba giá trị, nên một hàng hỏng sẽ trông Y HỆT một mục người dùng
/// tự nhập tay — đúng lớp lỗi mà AD-47 tồn tại để chống ở miền bản dịch
/// (`segment.translation_origin`), nay lặp lại ở miền Glossary nếu không sửa. `Category`
/// mắc lỗi nhẹ hơn (rơi về [`Category::Other`] không giả mạo một PHÂN LOẠI đáng tin hơn
/// các phân loại khác), nhưng cùng một nguyên tắc áp cho cả hai: **không đoán, trả lỗi**.
/// `?` ở [`load_tier`] đưa lỗi này lên [`StoreError::ReadFailed`] qua `Store::read`.
fn decode_category(col: usize, raw: &str) -> SqlResult<Category> {
    Category::from_wire(raw).ok_or_else(|| {
        SqlError::FromSqlConversionFailure(
            col,
            SqlType::Text,
            format!("glossary_entry.category tren dia khong khop CHECK -- gia tri: {raw:?}")
                .into(),
        )
    })
}

/// Cùng lý do [`decode_category`] — và cùng mức nghiêm trọng hơn hẳn, vì `Manual` là giá
/// trị đáng tin nhất trong ba giá trị của `term_origin` (xem doc-comment ngay trên).
fn decode_term_origin(col: usize, raw: &str) -> SqlResult<TermOrigin> {
    TermOrigin::from_wire(raw).ok_or_else(|| {
        SqlError::FromSqlConversionFailure(
            col,
            SqlType::Text,
            format!("glossary_entry.term_origin tren dia khong khop CHECK -- gia tri: {raw:?}")
                .into(),
        )
    })
}

/// Hai họ lỗi gặp nhau ở [`entries_eligible_for_injection`] — chỗ DUY NHẤT trong module
/// này vừa đọc kho (hai lượt [`load_tier`]) vừa phân giải hai tầng
/// (`ScopeResolver::apply_override`).
///
/// 🔴 **HAI BIẾN THỂ, KHÔNG GỘP THÀNH MỘT `String`** — [`StoreError`] và [`ScopeError`] là
/// hai LỚP lỗi khác hẳn nhau và không được phép trộn lẫn: `StoreError` là lỗi I/O đáng cho
/// người dùng biết (rồi đây sẽ đi qua IPC ở Epic 4, qua `From<StoreError> for IpcError` đã
/// có từ Story 1.7); `ScopeError` là lỗi **LẬP TRÌNH** và **KHÔNG BAO GIỜ** vượt ranh giới
/// IPC (xem doc-comment của chính nó). Gộp cả hai vào một chuỗi chẩn đoán ở đây sẽ xoá mất
/// đúng sự phân biệt mà `core::scope` dựng ra để giữ. Giữ hai biến thể tường minh để chỗ
/// gọi (Epic 4) `match` được, và để lượt nối `GlossaryError` vào `IpcError` sau này (nếu
/// cần) biết chính xác nhánh nào được phép đi qua.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlossaryError {
    /// Một trong hai lượt [`load_tier`] thất bại (mở kho, đọc, hoặc một hàng vi phạm
    /// `CHECK` mà một bản ứng dụng khác đã lỡ ghi — xem [`decode_category`]).
    Store(StoreError),
    /// `ScopeResolver::apply_override` từ chối. Không nên xảy ra trên đường gọi đúng — xem
    /// doc-comment của [`entries_eligible_for_injection`].
    Scope(ScopeError),
}

impl std::fmt::Display for GlossaryError {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlossaryError::Store(e) => write!(f, "glossary[store] {e}"),
            GlossaryError::Scope(e) => write!(f, "glossary[scope] {e}"),
        }
    }
}

impl std::error::Error for GlossaryError {}

impl From<StoreError> for GlossaryError {
    fn from(e: StoreError) -> Self {
        GlossaryError::Store(e)
    }
}

impl From<ScopeError> for GlossaryError {
    fn from(e: ScopeError) -> Self {
        GlossaryError::Scope(e)
    }
}

/// **Đúng MỘT hàm phơi cho module khác** (Epic 4, `RagInjector`) — điều kiện chèn nằm
/// TRỌN ở đây, không ở nơi gọi (AD-36). Xem cả hai mục 🔴 ở doc-comment đầu tệp.
///
/// 🔵 **CẬP NHẬT 2026-08-19 (lượt rà soát ba lớp, vá cuối) — nhận `&Store` thẳng, không còn
/// nhận `BTreeMap` đã nạp sẵn.** Bản trước đòi chỗ gọi tự có kết quả của `load_tier`, nhưng
/// `load_tier` bị chính cổng bảo vệ hàm này (`glossary_boundary.rs`) cấm gọi từ ngoài
/// `core/glossary/**` — tức đường DUY NHẤT dựng tham số cho hàm phơi ra DUY NHẤT lại bị cấm.
/// Hàm này giờ tự gọi `load_tier` cho `global` và (nếu có) `work` rồi mới phân giải — chỗ
/// gọi chỉ cần đưa `&Store` đã mở, đúng khuôn `core::scope::store::load_global_config(store:
/// &Store)`. `load_tier` ở lại `pub` (cho `glossary_contract.rs` — xem doc-comment của nó)
/// nhưng không còn ai NGOÀI hàm này cần gọi nó.
///
/// # Lỗi
/// [`GlossaryError::Store`] nếu một trong hai lượt `load_tier` thất bại;
/// [`GlossaryError::Scope`] nếu `ScopeResolver::apply_override` từ chối — lỗi lập trình,
/// không xảy ra trên đường gọi đúng, vì `GLOSSARY_SCOPE_KIND` là một hằng đã khớp
/// `ScopeKind::Glossary::Override`
/// (`scope_contract.rs::the_semantics_table_matches_ad_18_row_by_row` canh mệnh đề đó).
pub fn entries_eligible_for_injection(
    resolver: &ScopeResolver,
    global: &Store,
    work: Option<&Store>,
) -> Result<Vec<GlossaryEntry>, GlossaryError> {
    let global_tier = load_tier(global)?;
    let work_tier = work.map(load_tier).transpose()?;

    let resolved =
        resolver.apply_override(GLOSSARY_SCOPE_KIND, &global_tier, work_tier.as_ref())?;

    Ok(resolved
        .into_values()
        .filter_map(|resolved_entry| {
            let entry = resolved_entry.value().clone();
            // 🔴 LỌC SAU KHI PHÂN GIẢI — `resolved` đã áp AD-18 (tầng Tác phẩm thắng theo
            // từng thuật ngữ) TRƯỚC dòng này. Lọc ở đây không thể lộ một mục Global bị một
            // mục Work *chờ chốt* che, vì mục Global đó đã không còn trong `resolved` nữa —
            // nó nằm trong `shadowed()`, không trong `value()`.
            entry.is_confirmed().then_some(entry)
        })
        .collect())
}
