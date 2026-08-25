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

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::core::dict::DictLayers;
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::matching::{MatchLang, TermMatch, find_terms};
use crate::core::scope::{ScopeError, ScopeResolver, Tier as ScopeTier};
use crate::core::store::{
    ReadHandle, SqlError, SqlResult, SqlType, Store, StoreError, Transaction,
    is_unique_constraint_violation,
};

use super::entry::{Category, GlossaryEntry, GlossaryMark, GlossaryTier, TermOrigin};
use super::exchange::{ConflictDecision, Delimiter, ImportRow, ImportSummary, RowPlan, RowPlanKind};
use super::han_viet_suggestion::{HanVietSuggestion, suggest_han_viet_batch};

/// Khoá dây của `ScopeKind::Glossary` (`core/scope/kinds.rs:162`), chép lại đây làm
/// literal — module này không được `use` `ScopeKind`.
const GLOSSARY_SCOPE_KIND: &str = "glossary";

/// Câu `INSERT` DÙNG CHUNG cho **mọi** đường ghi vào `glossary_entry` — Story 3.2.
///
/// 🔴 **Bốn chỗ gọi được phép tồn tại, và cả bốn đều ở trong `core/glossary/**`:**
/// [`insert_manual_entry`] (ngay dưới, luôn `term_origin = manual`),
/// [`crate::core::glossary::candidate_store::approve_candidate`] (luôn suy `term_origin`
/// từ `candidate_origin` của chính hàng ứng viên), [`promote_to_global`] (mang nguyên
/// `term_origin` của hàng Work đang đẩy — giá trị đó đã bị khoá bởi MỘT trong ba chỗ gọi
/// còn lại từ lúc hàng đó được tạo ra), và [`import_into_tier`] (Story 3.10, luôn
/// `term_origin = file_import`). Đây là vế CẤU TRÚC của FR55 ("không cơ chế nào được tự ghi
/// vào Glossary"): trước Story 3.2, `insert_entry` cũ nhận `term_origin: TermOrigin` từ NƠI
/// GỌI — một module quét chỉ cần truyền `TermOrigin::ImportScan` là ghi thẳng, biên dịch
/// sạch, qua mọi cổng. Thu hẹp về **một** hàm `pub(super)` không tham số `term_origin` tự do
/// làm vi phạm đó KHÔNG BIỂU DIỄN ĐƯỢC: mọi giá trị `term_origin` đi vào đây đều đã bị khoá
/// bởi CHÍNH LOGIC của chỗ gọi, không phải một tham số người viết mã bên ngoài
/// `core/glossary/**` có thể tự ý đặt.
///
/// 🔵 **SỬA 2026-08-24 (Story 3.10) — mệnh đề "chỉ HAI chỗ gọi" ở trên đã SAI TỪ Story 3.9,
/// sửa tại chỗ.** [`promote_to_global`] (Story 3.9) đã là chỗ gọi THỨ BA từ trước lượt này —
/// không ai cập nhật câu trên khi nó ra đời. Đọc số THẬT (`grep insert_entry_row
/// src/core/glossary/**`) trước khi tin một câu đã viết sẵn, đúng bài học
/// `AGENTS.md`: "Đo trước khi chốt kiến trúc".
///
/// 🔵 **SỬA 2026-08-24 (Story 3.10) — tham số MỚI `created_at: Option<&str>`.** Ba chỗ gọi
/// cũ đều truyền `None` (giữ nguyên hành vi: SQL tự sinh mốc bằng `strftime('now')`).
/// [`import_into_tier`] là chỗ gọi DUY NHẤT có thể truyền `Some(giá trị)` — vòng tròn
/// xuất→nhập của I/O Matrix đòi `created_at` của tệp được GIỮ NGUYÊN, không bị ghi đè bằng
/// thời điểm nhập; xem doc-comment của [`import_into_tier`].
///
/// ⚠️ Chữ ký nhận **chuỗi đã chuẩn bị sẵn** (đã trim, đã `as_str()`) — không tự trim, không
/// tự gọi `Category::as_str()`/`TermOrigin::as_str()`. Cắt khoảng trắng là việc của TỪNG
/// chỗ gọi vì mỗi chỗ gọi cắt đầu vào của chính nó theo quy tắc riêng.
pub(super) fn insert_entry_row(
    tx: &Transaction<'_>,
    source_term: &str,
    translation: Option<&str>,
    note: &str,
    category: &str,
    term_origin: &str,
    created_at: Option<&str>,
) -> SqlResult<i64> {
    match created_at {
        None => tx.execute(
            "INSERT INTO glossary_entry
                (source_term, translation, note, category, term_origin, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            (&source_term, &translation, &note, &category, &term_origin),
        )?,
        Some(created_at) => tx.execute(
            "INSERT INTO glossary_entry
                (source_term, translation, note, category, term_origin, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (&source_term, &translation, &note, &category, &term_origin, &created_at),
        )?,
    };
    Ok(tx.last_insert_rowid())
}

/// Chèn một mục **nhập tay** mới. `translation = None` ⇒ hàng vào ngay trạng thái *chờ
/// chốt* (FR114). `term_origin` luôn `manual` — KHÔNG còn tham số cho chỗ gọi tự chọn.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.2) — đổi tên từ `insert_entry`, MẤT tham số
/// `term_origin: TermOrigin`.** Trước lượt này, bất kỳ chỗ gọi nào (kể cả một module quét
/// tương lai ở `core/segment/**`) chỉ cần truyền `TermOrigin::ImportScan` là ghi thẳng vào
/// Glossary — biên dịch sạch, qua cả mười một cổng của Story 3.1. Đường ghi phi-manual duy
/// nhất còn lại là [`crate::core::glossary::candidate_store::approve_candidate`], và nó
/// KHÔNG nhận `term_origin` từ chỗ gọi — nó suy ra từ `candidate_origin` của chính hàng
/// ứng viên đã tồn tại từ trước. Xem doc-comment của [`insert_entry_row`] cho lý do đầy đủ.
///
/// # Lỗi
/// [`StoreError::WriteFailed`] nếu `source_term` đã tồn tại (`UNIQUE INDEX
/// idx_glossary_entry_source_term`), hoặc `translation` là `Some("")`/khoảng trắng
/// (`CHECK` của `GLOSSARY_ENTRY_DDL`) — cả hai đều là lỗi giao dịch SQLite lan qua
/// `Store::write`, không phải một nhánh được kiểm tay ở đây.
pub fn insert_manual_entry(
    store: &Store,
    source_term: &str,
    translation: Option<&str>,
    note: &str,
    category: Category,
) -> Result<i64, StoreError> {
    // 🔴 CẮT KHOẢNG TRẮNG BIÊN, KHÔNG HẠ CHỮ THƯỜNG, KHÔNG CHUẨN HOÁ UNICODE — Story 3.1.
    //
    // Chỉ trim: `" 慕容"` và `"慕容"` không được thành hai hàng dưới một chỉ mục tự xưng
    // là "một thuật ngữ, một mục" (`idx_glossary_entry_source_term`) — trim ở đây làm
    // chúng va vào ĐÚNG một `UNIQUE` và lượt chèn thứ hai bị từ chối thay vì âm thầm tạo
    // ra một mục trùng có hình dạng khác. `str::trim()` của Rust cắt theo thuộc tính
    // Unicode `White_Space` — 25 điểm mã.
    //
    // 🔵 LƯỢT RÀ SOÁT #2 (2026-08-19) — BẢN TRƯỚC CỦA COMMENT NÀY NÓI SAI QUAN HỆ HAI LỚP.
    // Nó viết "cùng tập ký tự mà `CHECK` hai tham số chặn, nên hai lớp nói cùng một ngôn
    // ngữ". Sai lúc đó: bảng của `CHECK` khi ấy có BẢY ký tự, còn Rust cắt 25 — Rust là
    // tập CHA THỰC SỰ, và chính lớp Rust (không phải `CHECK`) mới là thứ đang đóng 17 điểm
    // mã còn lại. Ai đọc comment cũ mà tin `CHECK` là lưới cấu trúc rồi bỏ `.trim()` dưới
    // đây sẽ mở lại đúng lớp U+2009/U+202F/U+205F.
    //
    // Nay hai lớp ĐÃ thật sự cùng một tập: `GLOSSARY_ENTRY_DDL` liệt trọn 25 điểm mã
    // `White_Space` (đo từng điểm một). Câu "hai lớp nói cùng một ngôn ngữ" từ nay đúng —
    // và nó đúng nhờ một phép đo, không nhờ hai chỗ tình cờ trông giống nhau. ⇒ Thêm ký tự
    // vào một lớp thì phải thêm vào lớp kia CÙNG LƯỢT.
    //
    // KHÔNG hạ chữ thường: `API` ≠ `api` có nghĩa trong tiếng Anh — cùng luật mà
    // `AGENTS.md` đã khoá cho từ điển ("hạ chữ thường là THÊM một khoá, không THAY khoá
    // gốc"). KHÔNG chuẩn hoá Unicode (NFC/NFKC, …): chính sách chuẩn hoá thuật ngữ là
    // quyết định của Story 3.4 (khớp thuật ngữ theo ngôn ngữ), không phải của story này —
    // đoán trước nó ở đây là đóng băng một lựa chọn chưa ai ký.
    // 🔵 CẬP NHẬT 2026-08-20 (Story 3.3) — `note` NAY ĐƯỢC TRIM, ĐÓNG `deferred-work.md:
    // 5380-5385`. Bản trước (Story 3.1) chỉ trim `source_term`/`translation`; `note` đi
    // qua `to_owned()` trần. Không có `CHECK` nào canh khoảng trắng biên của `note` (nó
    // được phép rỗng — `GLOSSARY_ENTRY_DDL` đặt `NOT NULL DEFAULT ''`), nên một ghi chú
    // `"   "` từng đứng nguyên trên đĩa thay vì gọn về `""`. Ice ký 2026-08-20: một cách
    // biểu diễn duy nhất cho "không có ghi chú" — trim trước, để `""` và `"   "` không
    // phải hai hàng khác nhau trong mắt người đọc màn hình quản lý Glossary (Story 3.9).
    let source_term = source_term.trim().to_owned();
    let translation = translation.map(|t| t.trim().to_owned());
    let note = note.trim().to_owned();
    let category = category.as_str();

    store.write(move |tx: &Transaction<'_>| {
        insert_entry_row(
            tx,
            &source_term,
            translation.as_deref(),
            &note,
            category,
            TermOrigin::Manual.as_str(),
            None,
        )
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
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.3) — `id` KHÔNG khớp hàng nào NAY LÀ MỘT LỖI, đóng
/// `deferred-work.md:5348-5352` phần "Chủ: Story 3.3".** Doc-comment trước của hàm này
/// cảnh báo đúng rủi ro *"rỗng im lặng"* mà `AGENTS.md` liệt vào Known pitfalls trung tâm.
/// `tx.execute` trả về SỐ HÀNG bị đổi (`usize`); `0` giờ là [`StoreError::WriteFailed`] với
/// một câu chẩn đoán đọc được, không còn `Ok(())` cho một lượt ghi không đổi gì.
///
/// 🔵 **SỬA 2026-08-20 (lượt rà soát Story 3.3) — LÝ DO ghi ở lượt trên SAI, sửa tại chỗ.**
/// Bản trước viết Story 3.3 là *"chỗ gọi SẢN PHẨM đầu tiên chạm hàm này (qua
/// `commands::glossary::glossary_update_term`, gián tiếp qua `update_manual_term`)"*. Đo lại
/// thì không đúng ở cả hai chặng: [`update_manual_term`] tự phát câu `UPDATE` ba cột của
/// riêng nó và KHÔNG gọi hàm này một lần nào, còn `commands::glossary` thì bị
/// `glossary_boundary.rs::GLOSSARY_ONLY_SURFACE` **cấm** gõ chính cái tên `confirm_translation`.
/// `grep` trên `src-tauri/src/**`: hàm này vẫn có **0** chỗ gọi sản phẩm — chỗ duy nhất gọi
/// nó là `glossary_contract.rs`.
///
/// Lượt sửa hành vi vẫn ĐÚNG, chỉ lý do là sai: hàm này được `core::glossary::mod` tái xuất
/// công khai và là đường chốt mà Story 3.8 (duyệt hàng loạt)/3.9 sẽ đi qua, nên để `Ok(())`
/// cho một lượt `UPDATE` 0 hàng là gài sẵn đúng cái bẫy *rỗng im lặng* cho story đầu tiên
/// gọi tới — số chỗ gọi hôm nay là 0 hay 1 không đổi được điều đó. Một mệnh đề mô tả một
/// đường gọi không tồn tại là đúng lớp nợ mà luật *"sửa tại chỗ, kèm 🔵 + ngày"* của
/// `AGENTS.md` sinh ra để chống — không xoá, không để nó lặng lẽ sai.
///
/// # Lỗi
/// [`StoreError::WriteFailed`] nếu `translation` là chuỗi rỗng/khoảng trắng (`CHECK`), HOẶC
/// nếu `id` không khớp hàng nào trong `glossary_entry`.
///
/// ⚠️ Khối `# Lỗi` ở đây từng có HAI bản (một bản cũ chỉ kể ca `CHECK`, đứng trước khối 🔵
/// đầu tiên) — `rustdoc` gộp cả hai thành một mục và in bản cũ TRƯỚC, tức người đọc gặp
/// danh sách thiếu trước danh sách đủ. Gộp về một bản 2026-08-20 cùng lượt sửa trên.
pub fn confirm_translation(store: &Store, id: i64, translation: &str) -> Result<(), StoreError> {
    // Cùng lý do cắt khoảng trắng biên đã ghi ở `insert_manual_entry` — chốt qua đường này
    // cũng phải không tạo ra một bản dịch mang khoảng trắng thừa mà `insert_manual_entry`
    // đã cấm.
    let translation = translation.trim().to_owned();

    store.write(move |tx: &Transaction<'_>| {
        let changed = tx.execute(
            "UPDATE glossary_entry SET translation = ?1 WHERE id = ?2",
            (&translation, id),
        )?;
        if changed == 0 {
            return Err(row_missing_error(0, "glossary_entry", id));
        }
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

/// Nạp và phân giải Glossary hai tầng **một lần cho cả batch**, rồi trả tập thuật ngữ đã
/// tồn tại ở kết quả phân giải.
///
/// Lượt quét Story 3.5 dùng tập này để loại ứng viên trước khi enqueue. Không gọi phân giải
/// từng term: hai query chỉ lấy `source_term` và một lượt `ScopeResolver::apply_override`
/// là ảnh chụp có chủ. Đường này không cần `translation`/`note`/`created_at`, nên dựng cả
/// `GlossaryEntry` cho mọi hàng chỉ tăng allocation và kéo dài read critical section.
/// `WHERE NOT EXISTS` tầng Work trong chính câu `INSERT` vẫn đứng sau để chặn race giữa ảnh
/// chụp và giao dịch ghi. Hai database không có snapshot nguyên tử và hàm này cố ý không
/// `ATTACH`/dựng giao dịch chéo.
fn resolved_source_terms(
    resolver: &ScopeResolver,
    global: &Store,
    work: &Store,
) -> Result<BTreeSet<String>, GlossaryError> {
    debug_assert!(
        resolver.has_work_tier(),
        "resolved_source_terms requires the resolver and work store to travel together"
    );

    let load_keys = |store: &Store| {
        store.read(|conn: ReadHandle<'_>| {
            let mut stmt =
                conn.prepare("SELECT source_term FROM glossary_entry ORDER BY source_term")?;
            let mut rows = stmt.query([])?;
            let mut out = BTreeMap::new();
            while let Some(row) = rows.next()? {
                out.insert(row.get::<_, String>(0)?, ());
            }
            Ok(out)
        })
    };
    let global_tier: BTreeMap<String, ()> = load_keys(global)?;
    let work_tier: BTreeMap<String, ()> = load_keys(work)?;
    let resolved = resolver.apply_override(GLOSSARY_SCOPE_KIND, &global_tier, Some(&work_tier))?;
    Ok(resolved.into_keys().collect())
}

/// Lọc tại chỗ lô ứng viên theo Glossary hai tầng đã phân giải và trả số hàng bị loại.
/// Giá trị trả về đi thẳng vào `skipped` của sự kiện hoàn tất; phép lọc nằm cạnh phép
/// phân giải để không một chỗ gọi nào có thể lỡ kiểm riêng Global/Work trước override.
pub(crate) fn filter_import_scan_candidates_by_scope(
    resolver: &ScopeResolver,
    global: &Store,
    work: &Store,
    candidates: &mut Vec<crate::core::glossary::scan::ScanCandidate>,
) -> Result<i64, GlossaryError> {
    let resolved_terms = resolved_source_terms(resolver, global, work)?;
    let before = candidates.len();
    candidates.retain(|candidate| !resolved_terms.contains(&candidate.source_term));
    Ok(i64::try_from(before.saturating_sub(candidates.len())).unwrap_or(i64::MAX))
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
            format!("glossary_entry.category tren dia khong khop CHECK -- gia tri: {raw:?}").into(),
        )
    })
}

/// Cùng lý do [`decode_category`] — và cùng mức nghiêm trọng hơn hẳn, vì `Manual` là giá
/// trị đáng tin nhất trong các giá trị của `term_origin` (bốn kể từ Story 3.10 — 🔵 SỬA
/// 2026-08-24, câu cũ nói "ba" đã hết đúng) (xem doc-comment ngay trên).
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

/// Lỗi Rust-layer "đọc được" cho ca *"`UPDATE` khớp 0 hàng"* — Story 3.3, đóng
/// `deferred-work.md:5348-5352` (`confirm_translation`) và mảnh *"sửa một `id` đã biến
/// mất"* của I/O Matrix (`update_manual_term`).
///
/// Cùng khuôn `candidate_store.rs::already_decided_error`: không biến thể `rusqlite::Error`
/// nào đặt tên cho "quy tắc nghiệp vụ bị vi phạm" (0 hàng đổi không phải một lỗi SQL —
/// SQLite coi đó là thành công), nên `FromSqlConversionFailure` (mang một `Box<dyn Error>`
/// tự do) là chỗ mượn hợp lý nhất để chở một câu chẩn đoán tự chọn.
fn row_missing_error(col: usize, table: &str, id: i64) -> SqlError {
    SqlError::FromSqlConversionFailure(
        col,
        SqlType::Integer,
        format!("{table} id={id} khong ton tai -- UPDATE khop 0 hang").into(),
    )
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
    /// 🔵 **THÊM 2026-08-20 (Story 3.3).** [`update_manual_term`] nhắm vào một `id` đã biến
    /// mất (bị xoá giữa chừng — đua với Story 3.9, hay một `id` cũ còn kẹt ở webview) —
    /// I/O Matrix *"Sửa một `id` đã biến mất"*. **Không** mang `id`/`table` như
    /// [`row_missing_error`]: `params` của `IpcError` phải mang DỮ LIỆU chứ không mang
    /// CÂU, và một `id` nội bộ (khoá hàng SQLite) không phải thứ người dùng đọc được gì từ
    /// nó — nó chỉ là chẩn đoán, ở lại trong `Display` của biến thể này.
    EntryMissing,
    /// 🔵 **THÊM 2026-08-20 (Story 3.3).** Chỗ gọi chọn tầng Tác phẩm ([`GlossaryTier::Work`])
    /// cho [`add_manual_term`]/[`update_manual_term`] nhưng không có `&Store` nào của
    /// `project.db` để dùng — I/O Matrix *"Chọn tầng Tác phẩm khi chưa mở Tác phẩm"*. Trên
    /// đường gọi ĐÚNG của `commands::glossary`, ca này không nên xảy ra (dải "Thêm thuật
    /// ngữ" đã ẩn lựa chọn tầng Tác phẩm khi `OpenWorkState` là `None` — xem
    /// `glossaryQuickAddState.ts`), nhưng nó KHÔNG phải lỗi lập trình theo nghĩa
    /// [`ScopeError`]: một đua giữa "đóng Tác phẩm" và "bấm Lưu" là một trạng thái người
    /// dùng thật có thể chạm tới, nên nó đi qua IPC như một lỗi bình thường, không
    /// `debug_assert!`.
    WorkTierUnavailable,
    /// 🔵 **THÊM 2026-08-24 (Story 3.9).** [`promote_to_global`] tìm thấy `source_term` của
    /// mục Work đã có sẵn ở `global.db` — I/O Matrix *"Đẩy tầng, đích đã có"*. Kiểm tra ở
    /// TRƯỚC lượt `INSERT` (xem doc-comment của [`promote_to_global`]), nên biến thể này
    /// luôn đi kèm **0 lượt ghi**: không mục nào ở Global bị ghi đè, không mục nào ở Work
    /// bị xoá.
    GlobalTermExists,
    /// 🔵 **THÊM 2026-08-24 (Story 3.10).** [`import_into_tier`] phân loại một hàng là *mới*
    /// lúc `classify()`, nhưng lúc mở giao dịch thật thì `source_term` đó đã bị MỘT lượt ghi
    /// khác chèn vào tầng đích — I/O Matrix *"Va UNIQUE giữa chừng"*. Giao dịch của
    /// `import_into_tier` rollback TRỌN (0 hàng ghi).
    ///
    /// 🔵 **SỬA 2026-08-25 (vòng rà ba lớp, P5+P6) — hai lỗi ở bản đầu, cả hai sửa cùng
    /// lượt:**
    /// 1. **Chẩn đoán SAI nguyên nhân.** Bản đầu không phân biệt "giao dịch trượt vì `UNIQUE`"
    ///    với "giao dịch trượt vì trigger AD-36 (lượt `TakeTheirs` lùi bản dịch về rỗng)" — cả
    ///    hai đều bị gán nhãn `ImportUniqueConflict`, che mất đúng nguyên nhân thật ở ca sau.
    ///    Nay chỉ gán nhãn này khi lỗi SQL gốc THẬT SỰ là vi phạm `UNIQUE` (kiểm bằng
    ///    [`is_unique_constraint_violation`] ngay tại chỗ lỗi xảy ra, không đoán ngược từ việc
    ///    nạp lại tầng).
    /// 2. **Chỉ báo va chạm ĐẦU TIÊN.** Bản đầu dừng ở hàng `New` đầu tiên va — một lô đua với
    ///    NHIỀU lượt chèn khác bắt người dùng thử lại từng lần một. Nay gom TRỌN danh sách.
    ImportUniqueConflict {
        /// MỌI thuật ngữ va — dữ liệu người dùng vừa thấy trong tệp của họ, không phải một
        /// câu. Không bao giờ rỗng khi biến thể này được dựng.
        source_terms: Vec<String>,
    },

    // ── Story 3.10b (AD-48) — BẢY biến thể MỚI, hộp thoại chọn tệp nối vào ──────────
    //
    // Sinh ra ở `super::exchange_io` (bốn ca I/O đầu) hoặc ở `commands::glossary::wire`
    // (ba ca còn lại — không cần chạm đĩa để phát hiện).
    /// Tệp nhập vượt trần [`super::exchange_io::MAX_GLOSSARY_IMPORT_BYTES`] (16 MiB).
    /// `size`/`limit` là số byte thô.
    ///
    /// 🔵 **SỬA 2026-08-25 (vòng rà ba lớp, mục ⑧) — CÂU TRÊN HẾT ĐÚNG.** Bản
    /// trước khai kiểm bằng `metadata` TRƯỚC khi đọc byte nào — đúng cho HÌNH DẠNG CŨ của
    /// [`super::exchange_io::read_import_file`] (`metadata` ⇒ so ⇒ `std::fs::read` không
    /// chặn), nhưng hình dạng đó có một cửa sổ TOCTOU thật (tệp lớn lên GIỮA lúc `metadata`
    /// đo và lúc `read` đọc). Mục ⑧ thay nó bằng `File::open` + `Read::take(LIMIT + 1)` +
    /// `read_to_end` — quyết định "quá trần" nay dựa trên SỐ BYTE THẬT SỰ ĐÃ NẠP, không dựa
    /// trên một con số `metadata` có thể đã cũ.
    ImportFileTooLarge {
        /// Số byte THẬT SỰ đã đọc — `LIMIT + 1` khi trần bị vượt (mẹo chuẩn để phân biệt
        /// "tệp dài ĐÚNG BẰNG trần" với "tệp dài HƠN trần" mà không cần đọc trọn một tệp
        /// khổng lồ), KHÔNG PHẢI kích thước THẬT của tệp trên đĩa — hệ quả trực tiếp của
        /// việc không còn dựa vào `metadata` để quyết định chặn.
        size: u64,
        /// Trần đang áp.
        limit: u64,
    },
    /// Nội dung tệp không giải mã được bằng UTF-8 (`String::from_utf8`, KHÔNG
    /// `_lossy`) — không đoán bảng mã, dò bảng mã là Epic 6.
    ImportNotUtf8 {
        /// Đường dẫn tệp — chẩn đoán, và tham số `path` của `MessageKey::ImportNotUtf8`.
        path: String,
    },
    /// Mở/đọc tệp nhập thất bại vì lý do KHÁC kích thước và bảng mã (quyền truy cập,
    /// tệp bị xoá giữa lúc chọn và lúc đọc, …).
    ImportReadFailed {
        /// Đường dẫn tệp — chẩn đoán, và tham số `path` của `MessageKey::IoReadFailed`.
        path: String,
        /// Lỗi thô. Không đi lên giao diện.
        detail: String,
    },
    /// Ghi tệp xuất thất bại (hệ điều hành từ chối thư mục người dùng chọn, hết dung
    /// lượng, …). `exchange_io::write_export_file` dọn `.tmp` ở CẢ HAI nhánh lỗi trước
    /// khi biến thể này được dựng — không tệp cụt nào bị để lại.
    ExportWriteFailed {
        /// Đường dẫn đích người dùng đã chọn — dữ liệu, không phải câu.
        path: String,
        /// Lỗi thô. Không đi lên giao diện.
        detail: String,
    },
    /// `FilePath::into_path()` của `tauri-plugin-dialog` trả lỗi (`InvalidPathUrl`) —
    /// hộp thoại trả về một giá trị không quy đổi được thành `PathBuf`.
    DialogPathInvalid,
    /// Bản đồ quyết định của nhịp hai mang một khoá `source_term` KHÔNG có trong
    /// `Vec<RowPlan>` của lô đang treo — §Always: "một quyết định trỏ tới `source_term`
    /// không có trong lô là một lỗi tường minh", đóng `deferred-work.md:6798`.
    ImportDecisionUnknownTerm {
        /// Thuật ngữ lạ đọc được từ khoá của bản đồ quyết định.
        term: String,
    },
    /// Xác nhận lượt nhập (nhịp hai) khi chưa qua nhịp một, hoặc lô đã bị dọn (huỷ, mở
    /// lô khác, đóng Tác phẩm ở tầng Work khi lô đang treo thuộc tầng đó).
    NoPendingImport,
}

impl std::fmt::Display for GlossaryError {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlossaryError::Store(e) => write!(f, "glossary[store] {e}"),
            GlossaryError::Scope(e) => write!(f, "glossary[scope] {e}"),
            GlossaryError::EntryMissing => write!(f, "glossary[entry_missing]"),
            GlossaryError::WorkTierUnavailable => write!(f, "glossary[work_tier_unavailable]"),
            GlossaryError::GlobalTermExists => write!(f, "glossary[global_term_exists]"),
            GlossaryError::ImportUniqueConflict { source_terms } => {
                write!(f, "glossary[import_unique_conflict] source_terms={source_terms:?}")
            }
            GlossaryError::ImportFileTooLarge { size, limit } => {
                write!(f, "glossary[import_file_too_large] size={size} limit={limit}")
            }
            GlossaryError::ImportNotUtf8 { path } => {
                write!(f, "glossary[import_not_utf8] path={path}")
            }
            GlossaryError::ImportReadFailed { path, detail } => {
                write!(f, "glossary[import_read_failed] path={path} detail={detail}")
            }
            GlossaryError::ExportWriteFailed { path, detail } => {
                write!(f, "glossary[export_write_failed] path={path} detail={detail}")
            }
            GlossaryError::DialogPathInvalid => write!(f, "glossary[dialog_path_invalid]"),
            GlossaryError::ImportDecisionUnknownTerm { term } => {
                write!(f, "glossary[import_decision_unknown_term] term={term}")
            }
            GlossaryError::NoPendingImport => write!(f, "glossary[no_pending_import]"),
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

/// 🔵 **THÊM 2026-08-20 (Story 3.3)** — khuôn chép từ `core/store/mod.rs:483-506`
/// (`impl From<StoreError> for IpcError`): đi **qua [`IpcError::new`]**, không dựng struct
/// literal, cùng lý do đã ghi ở đó.
///
/// 🔴 **Nhánh `Scope` mang `code` ổn định và KHÔNG THAM SỐ.** `Display` của `ScopeError` là
/// một câu CHẨN ĐOÁN cho log (`"scope[glossary] declares Override but was resolved as
/// Merge"`) — đúng lớp chuỗi mà tham số `IpcError::params` bị cấm mang (params chở DỮ
/// LIỆU, không chở CÂU; xem doc-comment của `IpcError`). Nhét `Display` vào `params` sẽ đặt
/// nguyên văn một câu tiếng Anh chẩn đoán lên màn hình người dùng qua cửa sau. Nhánh này
/// không nên xảy ra trên đường gọi đúng (xem doc-comment của [`GlossaryError::Scope`]);
/// khi nó xảy ra, người dùng chỉ cần biết "có lỗi", còn chẩn đoán thật đi vào log Rust qua
/// `Display`/`Debug` ở chỗ gọi (`commands::glossary::wire`), không qua `IpcError`.
impl From<GlossaryError> for IpcError {
    fn from(err: GlossaryError) -> Self {
        match err {
            GlossaryError::Store(e) => e.into(),
            GlossaryError::Scope(_) => IpcError::new(
                "glossary.scope_error",
                MessageKey::GlossaryScopeError,
                BTreeMap::new(),
                false,
            ),
            GlossaryError::EntryMissing => IpcError::new(
                "glossary.entry_missing",
                MessageKey::GlossaryEntryMissing,
                BTreeMap::new(),
                false,
            ),
            GlossaryError::WorkTierUnavailable => IpcError::new(
                "glossary.work_tier_unavailable",
                MessageKey::GlossaryWorkTierUnavailable,
                BTreeMap::new(),
                false,
            ),
            GlossaryError::GlobalTermExists => IpcError::new(
                "glossary.global_term_exists",
                MessageKey::GlossaryGlobalTermExists,
                BTreeMap::new(),
                false,
            ),
            GlossaryError::ImportUniqueConflict { source_terms } => {
                let mut params = BTreeMap::new();
                // `value` -- du lieu nguoi dung vua thay trong tep cua ho, khong phai cau.
                // `IpcError::params` la BTreeMap<String, String> PHANG -- khong cho mot danh
                // sach that. Noi bang ", " la lua chon HIEN THI: tung source_term khong bi
                // cat mat, chi khong tach lai duoc bang may o day (frontend chi hien thi
                // nguyen van, khong can tach).
                params.insert("value".to_owned(), source_terms.join(", "));
                IpcError::new(
                    "glossary.import_unique_conflict",
                    MessageKey::GlossaryImportUniqueConflict,
                    params,
                    false,
                )
            }
            // 🔵 Story 3.10b — ba biến thể ĐẦU mượn khoá CHUNG với `core::segment::import`
            // (`MessageKey::ImportTooLarge`/`ImportNotUtf8`/`IoReadFailed`): câu đúng là
            // câu chung, không câu riêng của Glossary — xem chú thích tại khai báo khoá.
            GlossaryError::ImportFileTooLarge { size, limit } => {
                let mut params = BTreeMap::new();
                params.insert("size".to_owned(), size.to_string());
                params.insert("limit".to_owned(), limit.to_string());
                IpcError::new(
                    "glossary.import_file_too_large",
                    MessageKey::ImportTooLarge,
                    params,
                    false,
                )
            }
            GlossaryError::ImportNotUtf8 { path } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                IpcError::new("glossary.import_not_utf8", MessageKey::ImportNotUtf8, params, false)
            }
            GlossaryError::ImportReadFailed { path, .. } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                IpcError::new("glossary.import_read_failed", MessageKey::IoReadFailed, params, false)
            }
            GlossaryError::ExportWriteFailed { path, .. } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                IpcError::new(
                    "glossary.export_write_failed",
                    MessageKey::GlossaryExportWriteFailed,
                    params,
                    false,
                )
            }
            GlossaryError::DialogPathInvalid => IpcError::new(
                "glossary.dialog_path_invalid",
                MessageKey::GlossaryDialogPathInvalid,
                BTreeMap::new(),
                false,
            ),
            GlossaryError::ImportDecisionUnknownTerm { term } => {
                let mut params = BTreeMap::new();
                params.insert("value".to_owned(), term);
                IpcError::new(
                    "glossary.import_decision_unknown_term",
                    MessageKey::GlossaryImportDecisionUnknownTerm,
                    params,
                    false,
                )
            }
            GlossaryError::NoPendingImport => IpcError::new(
                "glossary.no_pending_import",
                MessageKey::GlossaryNoPendingImport,
                BTreeMap::new(),
                false,
            ),
        }
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
    // 🔵 THÊM 2026-08-20 (Story 3.3) — `deferred-work.md:5348-5352`: không chỗ gọi nào bắt
    // khớp `resolver.has_work_tier()` với `work.is_some()`. Hai giá trị này PHẢI đi cùng
    // nhau trên mọi đường gọi đúng: `resolver` chỉ mang `Some(WorkScope)` sau
    // `ScopeResolver::with_work`, và đó chính xác là lúc `OpenWork::store` (tầng
    // `project.db`) tồn tại để truyền vào đây làm `work`. Lệch nhau (resolver nói "có Tác
    // phẩm" mà `work` lại `None`, hoặc ngược lại) là một lỗi LẬP TRÌNH ở chỗ gọi — hai
    // trường của cùng một `OpenWork` bị tách rời nhau khi truyền xuống. `debug_assert_eq!`
    // không bắn ở bản release (`Chủ: Story 3.9` — `deferred-work.md`), nên đây là lưới cho
    // debug/`cargo test`, không phải một cưỡng chế production.
    debug_assert_eq!(
        resolver.has_work_tier(),
        work.is_some(),
        "entries_eligible_for_injection -- resolver.has_work_tier()={} nhung work.is_some()={} \
         -- hai gia tri nay phai di cung nhau tren moi duong goi dung (deferred-work.md:5348)",
        resolver.has_work_tier(),
        work.is_some()
    );

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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.3 — BA HÀM PHƠI RA MỚI, bề mặt IPC đầu tiên của `core/glossary/**`
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 Ba tên dưới đây là đường DUY NHẤT `commands::glossary` được phép chạm module này.
// `GLOSSARY_ONLY_SURFACE` của `glossary_boundary.rs` vẫn cấm `insert_manual_entry`/
// `confirm_translation`/`load_tier` ngoài `core/glossary/**` — ba hàm mới KHÔNG nằm trong
// danh sách đó, và đó chính là điểm của chúng (Ice ký ở `glossary_boundary.rs:80-88`, tiền
// lệ Story 3.1 đã đi qua đúng vòng luẩn quẩn "sửa CHỮ KÝ thay vì nới cổng" này một lần).

/// Tra một `source_term` qua **hai tầng** để dải "Thêm thuật ngữ" quyết định chế độ THÊM
/// hay SỬA (§Design Notes: `mode(source_term, lookup)`).
///
/// 🔴 **KHÔNG lọc `is_confirmed`** — khác hẳn [`entries_eligible_for_injection`]. Một mục
/// *chờ chốt* bị lọc mất ở đây sẽ làm dải mở nhầm ở chế độ THÊM, và `UNIQUE INDEX
/// idx_glossary_entry_source_term` chặn lượt lưu — người dùng thấy "không thêm được" mà
/// không ai nói vì sao (§Design Notes). `entries_eligible_for_injection` tồn tại đúng để
/// LỌC; hàm này tồn tại đúng để KHÔNG lọc — hai hàm phục vụ hai câu hỏi khác nhau
/// ("thuật ngữ nào được phép ép vào prompt" và "cụm này đã có trong Glossary chưa"),
/// không phải hai cách viết cùng một câu hỏi.
///
/// 🔴 **Trả `(GlossaryTier, GlossaryEntry)`, không chỉ `GlossaryEntry`.** `id` chỉ duy nhất
/// TRONG một `Store` — xem doc-comment của [`GlossaryTier`]. Không có tầng đi kèm, một
/// lượt SỬA sau đó không biết `id` này thuộc `global.db` hay `project.db`.
///
/// Cụm có ở CẢ hai tầng ⇒ trả về mục **tầng Tác phẩm** (AD-18: tầng Tác phẩm thắng theo
/// từng thuật ngữ) — `resolver.apply_override` đã phân giải đúng việc này; hàm ở đây chỉ
/// đọc `Resolved::tier()` của kết quả, không tự so sánh gì thêm.
///
/// # Lỗi
/// Cùng hai họ lỗi với [`entries_eligible_for_injection`] — xem [`GlossaryError`].
pub fn resolve_term_for_quick_add(
    resolver: &ScopeResolver,
    global: &Store,
    work: Option<&Store>,
    source_term: &str,
) -> Result<Option<(GlossaryTier, GlossaryEntry)>, GlossaryError> {
    // Cùng lưới `entries_eligible_for_injection` — hai trường của `OpenWork` không được
    // tách rời nhau trên đường xuống đây.
    debug_assert_eq!(
        resolver.has_work_tier(),
        work.is_some(),
        "resolve_term_for_quick_add -- resolver.has_work_tier()={} nhung work.is_some()={}",
        resolver.has_work_tier(),
        work.is_some()
    );

    // ⚠️ Trim TRƯỚC khi tra — `insert_manual_entry`/`insert_candidate` đều lưu `source_term`
    // đã trim (`idx_glossary_entry_source_term` khoá trên giá trị ĐÃ trim), nên một truy
    // vấn mang khoảng trắng biên (ô nguồn của dải chưa ai gõ thêm gì, chỉ dán nguyên vùng
    // chọn) phải trim để khớp đúng khoá.
    let source_term = source_term.trim();

    let global_tier = load_tier(global)?;
    let work_tier = work.map(load_tier).transpose()?;

    let resolved =
        resolver.apply_override(GLOSSARY_SCOPE_KIND, &global_tier, work_tier.as_ref())?;

    Ok(resolved.get(source_term).map(|resolved_entry| {
        let tier = match resolved_entry.tier() {
            ScopeTier::Global => GlossaryTier::Global,
            ScopeTier::Work => GlossaryTier::Work,
        };
        (tier, resolved_entry.value().clone())
    }))
}

/// Thêm một mục Glossary **nhập tay** vào tầng người dùng chọn — chế độ THÊM của dải.
///
/// Chọn `&Store` theo `tier` rồi gọi xuống [`insert_manual_entry`] (đường ghi phi-manual
/// vẫn chỉ có một cửa — [`crate::core::glossary::candidate_store::approve_candidate`] —
/// hàm này không mở thêm cửa nào, nó chỉ định tuyến `&Store` theo tầng).
///
/// # Lỗi
/// [`GlossaryError::WorkTierUnavailable`] nếu `tier` là [`GlossaryTier::Work`] mà `work` là
/// `None`. Còn lại, xem [`insert_manual_entry`] (`source_term`/`translation` rỗng ⇒
/// `CHECK`; `source_term` đã có ⇒ `UNIQUE`).
pub fn add_manual_term(
    global: &Store,
    work: Option<&Store>,
    tier: GlossaryTier,
    source_term: &str,
    translation: Option<&str>,
    note: &str,
    category: Category,
) -> Result<i64, GlossaryError> {
    let store = match tier {
        GlossaryTier::Global => global,
        GlossaryTier::Work => work.ok_or(GlossaryError::WorkTierUnavailable)?,
    };

    insert_manual_entry(store, source_term, translation, note, category)
        .map_err(GlossaryError::from)
}

/// Sửa `translation`/`note`/`category` của mục `(tier, id)` — chế độ SỬA của dải.
///
/// 🔴 **Nhận `(tier, id)`, không chỉ `id`** — cùng lý do [`resolve_term_for_quick_add`] trả
/// về tầng đi kèm: `id` chỉ duy nhất TRONG một `Store`, nên `tier` là thứ chọn ĐÚNG kho để
/// chạy `UPDATE` lên. Bỏ `tier` là một lượt `UPDATE` có thể nhắm nhầm kho, im lặng và
/// không cổng nào đỏ (§Design Notes).
///
/// ⚠️ **Không tự chốt "chờ chốt → đã chốt"** như [`confirm_translation`]: hàm này ghi CẢ
/// BA cột trong một câu `UPDATE`, đúng khuôn "sửa có hiệu lực ngay" mà Story 3.9 sẽ dùng
/// lại. Trigger `glossary_entry_lifecycle_is_one_way` vẫn đứng ở tầng SQL — một lượt SỬA cố
/// đưa `translation` từ đã chốt về `NULL` vẫn bị `RAISE(ABORT)` từ chối, không phải một
/// nhánh được kiểm tay ở đây.
///
/// # Lỗi
/// [`GlossaryError::WorkTierUnavailable`] nếu `tier` là [`GlossaryTier::Work`] mà `work` là
/// `None`. [`GlossaryError::EntryMissing`] nếu `(tier, id)` không khớp hàng nào — I/O Matrix
/// *"Sửa một `id` đã biến mất"*. [`GlossaryError::Store`] nếu `translation`/`note` vi phạm
/// `CHECK`, hoặc trigger một chiều từ chối.
pub fn update_manual_term(
    global: &Store,
    work: Option<&Store>,
    tier: GlossaryTier,
    id: i64,
    translation: Option<&str>,
    note: &str,
    category: Category,
) -> Result<(), GlossaryError> {
    let store = match tier {
        GlossaryTier::Global => global,
        GlossaryTier::Work => work.ok_or(GlossaryError::WorkTierUnavailable)?,
    };

    // Cùng lý do cắt khoảng trắng biên đã ghi ở `insert_manual_entry`.
    let translation = translation.map(|t| t.trim().to_owned());
    let note = note.trim().to_owned();
    let category = category.as_str();

    let changed = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE glossary_entry SET translation = ?1, note = ?2, category = ?3 WHERE id = ?4",
            (&translation, &note, category, id),
        )
    })?;

    if changed == 0 {
        return Err(GlossaryError::EntryMissing);
    }
    Ok(())
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.9 — BA HÀM PHƠI RA MỚI: liệt kê cả hai tầng · xoá · đẩy tầng (Work → Global)
// ═════════════════════════════════════════════════════════════════════════════════

/// Mọi mục Glossary của **cả hai tầng** — khuôn chép [`entries_eligible_for_injection`]
/// (cùng lượt `load_tier` × 2 rồi `ScopeResolver::apply_override`), nhưng khác nó ở hai
/// điểm mà chính màn hình "Quản lý Glossary" cần và `entries_eligible_for_injection`
/// (dựng cho Epic 4) không được phép có:
///
/// 1. **Không lọc `is_confirmed`** — một mục chờ chốt phải hiện ra để người dùng SỬA/XOÁ
///    nó, không chỉ mục đã chốt.
/// 2. **Phát cả `Resolved::shadowed()` thành một hàng thứ hai**, `(GlossaryTier::Global, ..,
///    true)` — đây là chỗ DUY NHẤT trong `core::glossary` biết mục Global nào đang bị một
///    mục Work cùng `source_term` che. Bỏ nó đi là làm một mục Global CÓ THẬT biến mất khỏi
///    màn hình mà không dòng nào giải thích — đúng chỗ hở mà §Intent của spec 3.9 mô tả.
///
/// Mỗi phần tử trả về là `(tier, entry, is_shadowed)`; hàng thắng của một khoá luôn đứng
/// trước hàng bị che của khoá đó (nếu có) — thứ tự tổng thể theo `source_term` (khoá của
/// `BTreeMap` bên trong `resolver.apply_override`), KHÔNG theo `id`/`created_at`.
///
/// 🔵 **SỬA 2026-08-24 (vòng rà ba lớp).** Bản trước viết tiếp *"sắp lại theo cột nào người
/// dùng chọn là việc của frontend"* và dẫn §Design Notes của spec. Sai hai vế: §Design Notes
/// chỉ nói **LỌC** chạy trong bộ nhớ, chưa bao giờ nói **SẮP XẾP**; và Story 3.9 không dựng
/// một cột bấm-để-sắp nào — `GlossaryManageOverlay.vue` render thẳng `manageFilteredRows`
/// theo đúng thứ tự hàm này trả về. Một doc-comment mô tả một năng lực chưa từng tồn tại là
/// thứ người sau sẽ tưởng đã được xét. Thứ tự trả về là thứ tự hiển thị, và chỉ thế.
///
/// # Lỗi
/// Cùng hai họ lỗi với [`entries_eligible_for_injection`] — xem [`GlossaryError`].
pub fn list_all_entries(
    resolver: &ScopeResolver,
    global: &Store,
    work: Option<&Store>,
) -> Result<Vec<(GlossaryTier, GlossaryEntry, bool)>, GlossaryError> {
    debug_assert_eq!(
        resolver.has_work_tier(),
        work.is_some(),
        "list_all_entries -- resolver.has_work_tier()={} nhung work.is_some()={}",
        resolver.has_work_tier(),
        work.is_some()
    );

    let global_tier = load_tier(global)?;
    let work_tier = work.map(load_tier).transpose()?;

    let resolved =
        resolver.apply_override(GLOSSARY_SCOPE_KIND, &global_tier, work_tier.as_ref())?;

    let mut out = Vec::with_capacity(resolved.len());
    for resolved_entry in resolved.into_values() {
        let tier = match resolved_entry.tier() {
            ScopeTier::Global => GlossaryTier::Global,
            ScopeTier::Work => GlossaryTier::Work,
        };
        // `shadowed()` đọc TRƯỚC khi `value()` tiêu thụ `resolved_entry` — cả hai đều
        // mượn, không xung đột, và thứ tự đọc-rồi-tiêu-thụ này là thứ làm dòng dưới hợp lệ.
        let shadowed = resolved_entry.shadowed().cloned();
        out.push((tier, resolved_entry.value().clone(), false));
        if let Some(shadowed_entry) = shadowed {
            // `Resolved::new` cấm `tier == Global` mang `shadowed`, nên hàng CHE luôn ở
            // tầng Work và hàng BỊ CHE luôn ở tầng Global — đúng AD-18 ("tầng Work thắng"),
            // không có chiều ngược lại để mà xử.
            out.push((GlossaryTier::Global, shadowed_entry, true));
        }
    }
    Ok(out)
}

/// Xoá mục `(tier, id)` — khuôn chép [`add_manual_term`] cho việc định tuyến `&Store` theo
/// `tier`, khuôn chép [`update_manual_term`] cho việc dịch "0 hàng đổi" thành
/// [`GlossaryError::EntryMissing`] thay vì một `Ok(())` rỗng im lặng.
///
/// 🔴 **Xoá một mục ĐÃ CHỐT là hợp lệ** *(Ice chốt 2026-08-24, §Always của spec 3.9)* —
/// trigger `glossary_entry_lifecycle_is_one_way` chỉ khớp `UPDATE OF translation`, không
/// bao giờ khớp `DELETE`. Vòng đời một chiều của AD-36 nói *"không lượt `UPDATE` nào lùi
/// trạng thái trong im lặng"*, không nói *"không thao tác nào tái tạo được một mục chờ
/// chốt"* — xoá rồi thêm lại (nếu người dùng muốn) là HAI thao tác thấy được, không một
/// đường lách ngầm quanh trigger.
///
/// # Lỗi
/// [`GlossaryError::WorkTierUnavailable`] nếu `tier` là [`GlossaryTier::Work`] mà `work` là
/// `None`. [`GlossaryError::EntryMissing`] nếu `(tier, id)` không khớp hàng nào — mục đã bị
/// xoá ở nơi khác giữa chừng.
pub fn delete_manual_term(
    global: &Store,
    work: Option<&Store>,
    tier: GlossaryTier,
    id: i64,
) -> Result<(), GlossaryError> {
    let store = match tier {
        GlossaryTier::Global => global,
        GlossaryTier::Work => work.ok_or(GlossaryError::WorkTierUnavailable)?,
    };

    let changed = store
        .write(move |tx: &Transaction<'_>| tx.execute("DELETE FROM glossary_entry WHERE id = ?1", [id]))?;

    if changed == 0 {
        return Err(GlossaryError::EntryMissing);
    }
    Ok(())
}

/// Đẩy mục `id` ở tầng **Tác phẩm** lên tầng **Toàn cục** — `INSERT global` TRƯỚC, `DELETE
/// work` SAU (§Always của spec 3.9: hai kho KHÔNG có giao dịch chung; sập giữa hai bước để
/// lại mục ở CẢ HAI tầng, Work vẫn thắng theo AD-18, ngữ nghĩa không đổi, làm lại được —
/// thứ tự ngược lại làm mục biến mất hẳn).
///
/// Nhận thẳng `&Store work` (không `Option`) — khác mọi hàm khác của module này: chỗ gọi
/// (`commands::glossary::glossary_promote_term_to_global`) đã tự từ chối khi chưa mở Tác
/// phẩm nào (`no_work_open`, cùng khuôn `glossary_approve_candidate`/`glossary_reject_
/// candidate`), vì bảng `glossary_entry` tầng Work chỉ tồn tại trong MỘT `project.db` — hàm
/// này không có gì để đẩy nếu không có nó.
///
/// 🔴 **Kiểm tra "đích đã có" TRƯỚC khi ghi, không bắt lỗi `UNIQUE` sau khi `INSERT` trượt**
/// — §Always: "`source_term` đã có ở tầng Toàn cục ⇒ đẩy tầng TRẢ LỖI CÓ TÊN, không ghi đè"
/// VÀ "0 lượt ghi". Một lượt `INSERT` trượt vì `UNIQUE` vẫn đúng "0 lượt ghi" (giao dịch
/// rollback), nhưng lỗi đó đi ra dưới hình dạng `StoreError::WriteFailed` chung
/// (`store.write_failed`), không phân biệt được với MỌI lượt ghi trượt khác — spec đòi một
/// lỗi CÓ TÊN. Đọc TRƯỚC (khuôn `commands::segment::split_chapter_into_segments::
/// already_split`, kiểm "đã có segment" bằng một `read` riêng trước khi ghi) mở một cửa sổ
/// đua NHỎ (không giao dịch chung giữa hai kho — xem đoạn trên) nhưng cho một lỗi ĐỌC ĐƯỢC
/// ở đường thường; nếu đua thật sự xảy ra, `UNIQUE INDEX idx_glossary_entry_source_term`
/// vẫn là lưới cuối ở tầng SQL và biến nó thành một `store.write_failed` chung — không một
/// "nửa ghi" nào lọt qua.
///
/// # Lỗi
/// [`GlossaryError::EntryMissing`] nếu `id` không khớp hàng nào ở `work` — kể cả khi nó
/// biến mất GIỮA bước đọc và bước `DELETE` (đua với một lượt Xoá khác): lúc đó mục đã có ở
/// CẢ HAI tầng (bước `INSERT global` đã xong), Work vẫn thắng, làm lại được — đúng ngữ
/// nghĩa "sập giữa chừng" mà §Always mô tả, dù nguyên nhân là một thao tác khác chứ không
/// phải một lượt sập tiến trình.
/// [`GlossaryError::GlobalTermExists`] nếu `source_term` đã có ở `global` — **0 lượt ghi**.
/// [`GlossaryError::Store`] nếu một bước I/O khác trượt.
pub fn promote_to_global(global: &Store, work: &Store, id: i64) -> Result<(), GlossaryError> {
    // Đọc hàng Work TRƯỚC — `id` chỉ duy nhất TRONG `Store` của nó, và ta cần dữ liệu để
    // dựng hàng Global tiếp theo.
    let found: Option<(String, Option<String>, String, String, String)> =
        work.read(move |conn: ReadHandle<'_>| {
            let row = conn.query_row(
                "SELECT source_term, translation, note, category, term_origin
                 FROM glossary_entry WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            );
            match row {
                Ok(value) => Ok(Some(value)),
                // Cùng khuôn `commands::segment::split_chapter_into_segments` — `SqlError`
                // đã được `core::store` tái xuất, `OptionalExtension` thì không.
                Err(SqlError::QueryReturnedNoRows) => Ok(None),
                Err(err) => Err(err),
            }
        })?;
    let (source_term, translation, note, category_raw, term_origin_raw) =
        found.ok_or(GlossaryError::EntryMissing)?;

    // Kiểm tra đích TRƯỚC khi ghi bất cứ gì — xem khối 🔴 ở doc-comment trên.
    let already_exists: bool = {
        let source_term_check = source_term.clone();
        global.read(move |conn: ReadHandle<'_>| {
            conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM glossary_entry WHERE source_term = ?1)",
                [&source_term_check],
                |r| r.get(0),
            )
        })?
    };
    if already_exists {
        return Err(GlossaryError::GlobalTermExists);
    }

    // Bước 1: INSERT global.
    //
    // `created_at`: None -- day tang lai la lan dau mot mục nay ton tai o global.db, nen
    // moc "duoc tao luc nao" tu nhien sinh lai bang thoi diem day tang (khac import_into_tier,
    // noi ca duong nhap la mot LAN THAY THE mot hang chua tung co, khong phai mot lan DI
    // CHUYEN mot hang da co).
    global.write(move |tx: &Transaction<'_>| {
        insert_entry_row(
            tx,
            &source_term,
            translation.as_deref(),
            &note,
            &category_raw,
            &term_origin_raw,
            None,
        )
    })?;

    // Bước 2: DELETE work.
    //
    // 🔵 **SỬA 2026-08-24 (vòng rà ba lớp) — bản trước SAI cả hành vi lẫn lý do.** Nó viết
    // rằng `changed == 0` nghĩa là *"mục giờ tồn tại ở CẢ HAI tầng — Work vẫn thắng"* rồi trả
    // `EntryMissing`. Cả hai vế đều không đúng, và chúng gộp HAI kịch bản khác hẳn nhau:
    //
    // - **Sập giữa hai bước** (tiến trình chết sau `INSERT`, trước `DELETE`): hàng Work còn
    //   nguyên ⇒ mục ở cả hai tầng, Work thắng, làm lại được. Đó là ca mà §Always mô tả, và
    //   không đường mã nào ở đây chạy để quan sát nó.
    // - **`changed == 0`**: hàng Work đã biến mất TRƯỚC lượt `DELETE` này (đua với một lượt
    //   Xoá khác ở đúng hàng đó). Mục KHÔNG ở cả hai tầng — nó chỉ còn ở Global, tức đúng
    //   trạng thái đích mà lượt đẩy tầng nhắm tới. Trả `EntryMissing` ở đây là **báo trượt
    //   một lượt đã thành công**, và lượt thử lại sau đó cũng trượt vì hàng Work không còn.
    //   Người dùng thấy một câu lỗi trong khi thuật ngữ đã lên Global — và nếu họ vừa cố Xoá
    //   nó, thứ họ thấy là một mục **sống lại** ở tầng khác.
    //
    // ⇒ Trạng thái đích đã đạt thì trả `Ok`. Không cuộn lại lượt `INSERT`: cuộn lại cho ra
    // *"thuật ngữ không còn ở đâu cả"*, tức đổi một trạng thái DƯ lấy một trạng thái THIẾU —
    // ngược đúng nguyên tắc chọn thứ tự hai kho đã ghi ở §Always của spec.
    work.write(move |tx: &Transaction<'_>| tx.execute("DELETE FROM glossary_entry WHERE id = ?1", [id]))?;
    Ok(())
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.6 — HÀM PHƠI RA: chốt bản dịch lần đầu gặp cho một mục CHỜ CHỐT
// ═════════════════════════════════════════════════════════════════════════════════

/// Chốt `translation` cho mục `(tier, id)` — khuôn CHÉP từ [`add_manual_term`]: định tuyến
/// `&Store` theo `tier` rồi gọi xuống [`confirm_translation`] (hàm bị `GLOSSARY_ONLY_SURFACE`
/// cấm gọi từ `commands/**`), đúng đường "sửa CHỮ KÝ thay vì nới cổng" mà Story 3.1 đã đi.
///
/// 🔴 **Không phân biệt "chốt lần đầu" và "sửa mục đã chốt" ở tầng này** — cùng như
/// [`confirm_translation`], hàm này dùng được ở CẢ HAI chiều hợp lệ. Story 3.6 chỉ dựng
/// đường gọi cho chiều đầu (dải mọc khi gặp mục *chờ chốt*), nhưng không có lý do cấu trúc
/// nào để hàm THUẦN này hẹp hơn hàm nó bọc.
///
/// # Lỗi
/// [`GlossaryError::WorkTierUnavailable`] nếu `tier` là [`GlossaryTier::Work`] mà `work` là
/// `None`. Còn lại, xem [`confirm_translation`] (`translation` rỗng ⇒ `CHECK`; `id` không
/// khớp hàng nào ⇒ lỗi đọc được, không `Ok(())` cho một lượt ghi 0 hàng).
pub fn confirm_pending_translation(
    global: &Store,
    work: Option<&Store>,
    tier: GlossaryTier,
    id: i64,
    translation: &str,
) -> Result<(), GlossaryError> {
    let store = match tier {
        GlossaryTier::Global => global,
        GlossaryTier::Work => work.ok_or(GlossaryError::WorkTierUnavailable)?,
    };

    confirm_translation(store, id, translation).map_err(GlossaryError::from)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.4 — HÀM PHƠI RA THỨ TƯ: khớp thuật ngữ theo ngôn ngữ (FR50/FR51)
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 `marks_for_source_text` gọi `core::matching::find_terms` (AD-17) — KHÔNG cài lại
// phép khớp. `core/glossary/**` là module MIỀN sở hữu tra hai tầng + phân xử chồng nhau;
// `core::matching` là module LÁ sở hữu chính phép khớp — hai trách nhiệm khác nhau, và
// ranh giới đó được cưỡng chế ở `tests/glossary_boundary.rs` (bốn hàm phơi ra, không nới
// `GLOSSARY_ONLY_SURFACE`) lẫn `tests/matching_boundary.rs` (module kia là LÁ).

/// Suy [`MatchLang`] từ `source_lang` của một Tác phẩm — **điểm DUY NHẤT** của Story 3.4 nơi
/// phép chọn `source_lang == LANG_CHINESE` được viết ra.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 SỬA 2026-08-21 (rà soát ba lớp) — GOM VỀ MỘT HÀM, ĐÓNG MỘT HỒI QUY ĐANG MỌC
/// ─────────────────────────────────────────────────────────────────────────────
/// Bản trước của story có **HAI** chỗ tự viết `source_lang == LANG_CHINESE`
/// (hàm này và `commands::glossary::glossary_marks_for_chapter`), nối nhau chỉ bằng một
/// dòng chú thích "cùng nhánh split.rs:219" — đúng lớp lỗi mà chính chú thích đó cảnh báo:
/// hai chỗ viết tay có thể trôi khỏi nhau ở lần sửa thứ ba. Gộp về đây; chỗ gọi kia giờ hỏi
/// thẳng hàm này.
///
/// 🔴 **Vì sao hàm sống ở `core::glossary`, KHÔNG ở `core::matching`.** `core::matching` tự
/// tuyên bố (doc-comment của [`crate::core::matching::MatchLang`]) **không tồn tại** một vị
/// từ dò script nào trong module đó, và nó là **LÁ** trong đồ thị phụ thuộc (AD-13) — không
/// `use crate::core::*`. Đặt hàm này ở đó buộc nó `use crate::core::segment::split` (cho
/// [`crate::core::segment::split::LANG_CHINESE`]), phá cả hai bất biến cùng lúc.
/// `core::glossary` không mang ràng buộc "LÁ" đó, và nó đã phụ thuộc cả `core::matching`
/// (cho chính [`MatchLang`]) lẫn `core::segment` (qua module này) — đây là chỗ tự nhiên nhất
/// để một kiểu của module kia gặp một hằng của module khác nữa.
///
/// Dùng LẠI đúng một hằng ([`crate::core::segment::split::LANG_CHINESE`]), KHÔNG đúc một
/// phép chọn thứ hai — cùng nhánh mà `core::segment::split::split_source_text` đã dùng.
pub fn match_lang_for_source_lang(source_lang: &str) -> MatchLang {
    if source_lang == crate::core::segment::split::LANG_CHINESE {
        MatchLang::Zh
    } else {
        MatchLang::En
    }
}

/// Hâm nóng `Jieba` cho một Chương mang `source_lang` — Story 3.4, đóng
/// `deferred-work.md:413`.
///
/// 🔴 **Gọi từ đường MỞ CHƯƠNG (`commands::chapter::read_open_chapter` /
/// `open_adjacent_chapter`), KHÔNG từ thân [`marks_for_source_text`].** Xem doc-comment của
/// [`crate::core::matching::warm`] cho số đo đầy đủ (179–329 ms bản release, lần gọi ĐẦU
/// TIÊN). Nếu lượt hâm nằm trong đường khớp, chi phí đó rơi đúng vào khung hình đang gõ —
/// đúng thứ NFR2 cấm. Nằm trên đường mở Chương, nó rơi vào một thao tác đã chấp nhận độ trễ
/// vài trăm ms.
///
/// ⚠️ **Chỉ hâm khi [`match_lang_for_source_lang`] trả [`MatchLang::Zh`].** `Jieba` chỉ được
/// [`crate::core::matching::tokenize`]/[`find_terms`] chạm tới ở nhánh đó (xem doc-comment
/// của `core::matching`); hâm nó cho một Chương tiếng Anh là trả 179–329 ms mà không ai
/// hưởng lợi.
pub fn warm_jieba_for_source_lang(source_lang: &str) {
    if match_lang_for_source_lang(source_lang) == MatchLang::Zh {
        crate::core::matching::warm();
    }
}

/// Tính danh sách ranh giới ĐIỂM MÃ của `text` — phần tử thứ `i` là vị trí BYTE của điểm mã
/// thứ `i`; phần tử cuối là `text.len()` (ranh giới SAU điểm mã cuối cùng).
///
/// Dùng để quy đổi span byte của [`find_terms`] sang span điểm mã bằng `binary_search`:
/// mọi span mà `find_terms` trả về đều rơi đúng một ranh giới UTF-8 hợp lệ (doc-comment
/// của `TermMatch`), và với cả hai nhánh `Zh`/`En` ranh giới đó luôn TRÙNG một ranh giới
/// ĐIỂM MÃ (jieba cắt theo điểm mã; nhánh `En` cắt theo `char_indices`) — nên
/// `binary_search` luôn `Ok`, không bao giờ rơi vào nhánh `Err` (giữ nhánh đó chỉ để không
/// panic nếu giả định này có ngày sai).
fn codepoint_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries: Vec<usize> = text.char_indices().map(|(byte, _)| byte).collect();
    boundaries.push(text.len());
    boundaries
}

/// Quy đổi một vị trí BYTE sang vị trí ĐIỂM MÃ, dùng bảng của [`codepoint_boundaries`].
///
/// ⚠️ `unwrap_or_else` là lưới an toàn, không phải đường THẬT: xem doc-comment của
/// [`codepoint_boundaries`] — `byte_offset` luôn khớp `Ok` trên đường gọi đúng.
fn byte_to_codepoint(boundaries: &[usize], byte_offset: usize) -> usize {
    boundaries
        .binary_search(&byte_offset)
        .unwrap_or_else(|insert_at| insert_at)
}

/// Phân xử CHỒNG NHAU giữa các [`TermMatch`] — **span dài nhất thắng, hoà thì trái nhất**
/// (§Design Notes của Story 3.4).
///
/// 🔴 **Vì sao phải phân xử, không trả nguyên [`find_terms`]:** `find_terms` trả span
/// CHỒNG NHAU được — `AA` trong `AAA` là hai lượt xuất hiện thật, và hai thuật ngữ khác
/// nhau phủ lên nhau cũng vậy (doc-comment của chính nó). Một kênh ĐÁNH DẤU thì không phân
/// thân được: hai dấu chồng lên nhau ở cùng một chỗ không vẽ được. Phân xử **ở đây** — nơi
/// duy nhất trong kho biết cả tập khớp — thay vì đẩy luật đó xuống cho nửa giao diện tự
/// nghĩ ra một luật thứ hai.
///
/// Thuật toán: sắp theo `(độ dài giảm dần, vị trí bắt đầu tăng dần)` rồi chọn THAM LAM theo
/// đúng thứ tự đó, bỏ qua mọi lượt khớp chồng lấn với một lượt đã chọn. Đó chính là
/// "dài nhất thắng" (độ dài đứng trước trong khoá sắp) và "hoà thì trái nhất" (vị trí đứng
/// sau, chỉ so khi độ dài bằng nhau).
fn resolve_overlaps(mut matches: Vec<TermMatch>) -> Vec<TermMatch> {
    matches.sort_by(|a, b| {
        let len_a = a.span.end - a.span.start;
        let len_b = b.span.end - b.span.start;
        len_b
            .cmp(&len_a)
            .then_with(|| a.span.start.cmp(&b.span.start))
            .then_with(|| a.term_index.cmp(&b.term_index))
    });

    let mut selected: Vec<TermMatch> = Vec::new();
    for candidate in matches {
        let overlaps = selected.iter().any(|kept| {
            kept.span.start < candidate.span.end && candidate.span.start < kept.span.end
        });
        if !overlaps {
            selected.push(candidate);
        }
    }

    // Trả lại đúng thứ tự tất định mà `find_terms` đã hứa — chỗ gọi (Story 3.4b, nửa giao
    // diện) không phải tự sắp lại.
    selected.sort_by(|a, b| {
        (a.span.start, a.span.end, a.term_index).cmp(&(b.span.start, b.span.end, b.term_index))
    });
    selected
}

/// **Hàm phơi ra THỨ TƯ** của `core::glossary` — Story 3.4, FR50/FR51. Tra hai tầng qua
/// `ScopeResolver::apply_override` (**không lọc** `is_confirmed` — cùng lý do
/// [`resolve_term_for_quick_add`]: một mục *chờ chốt* vẫn phải ra dấu, mang cờ phân biệt),
/// rồi gọi [`find_terms`] (AD-17) trên tập thuật ngữ đã phân giải và quy đổi span
/// byte → điểm mã **một lần, ở đúng một chỗ** (§Design Notes).
///
/// 🔴 **Không lọc `is_confirmed`** — khác hẳn [`entries_eligible_for_injection`]. Mục chờ
/// chốt vẫn được đánh dấu (I/O Matrix: *"Mục chờ chốt ⇒ Có dấu, `is_confirmed=false`,
/// `translation=null`"*) — chỉ khoá nào **đã chốt** mới được ép vào prompt (AD-36), nhưng cả
/// hai trạng thái đều đáng được người dịch NHÌN THẤY trên lưới.
///
/// 🔴 **Chồng nhau được phân xử NGAY tại đây** — xem [`resolve_overlaps`]. `find_terms`
/// KHÔNG được sửa để tự phân xử: nó phục vụ CẢ Glossary lẫn TM (AD-17, Story 7.6), và luật
/// "một dấu cho một chỗ" là luật RIÊNG của kênh đánh dấu Glossary, không phải luật của
/// chính phép khớp.
///
/// # Lỗi
/// [`GlossaryError::Store`] nếu một trong hai lượt [`load_tier`] thất bại (kể cả kho không
/// mở được — I/O Matrix *"`Store` đóng giữa chừng ⇒ lỗi mang `message_key`, KHÔNG
/// `Ok(vec![])`"*); [`GlossaryError::Scope`] nếu `ScopeResolver::apply_override` từ chối
/// (lỗi lập trình, không nên xảy ra trên đường gọi đúng).
///
/// 🔵 **THÊM 2026-08-24 (Story 3.7, FR113)** — `layers`/`disabled` thêm để đề xuất âm Hán
/// Việt tính được TRONG CÙNG lượt mở Chương, không một vòng IPC thứ hai. Chỉ các mục **CHỜ
/// CHỐT** (`!entry.is_confirmed()`) được gom `source_term` rồi tra qua
/// [`suggest_han_viet_batch`] — **một lượt gọi cho cả tập**, không một lượt cho mỗi dấu; mục
/// ĐÃ CHỐT nhận thẳng [`HanVietSuggestion::NotRequested`] (§Design Notes của story: một đề
/// xuất cho mục đã có bản dịch thật không có chỗ tiêu thụ).
pub fn marks_for_source_text(
    resolver: &ScopeResolver,
    global: &Store,
    work: Option<&Store>,
    text: &str,
    lang: MatchLang,
    layers: &DictLayers,
    disabled: &BTreeSet<String>,
) -> Result<Vec<GlossaryMark>, GlossaryError> {
    // Cùng lưới `entries_eligible_for_injection`/`resolve_term_for_quick_add` — hai trường
    // của cùng một `OpenWork` không được tách rời nhau trên đường xuống đây.
    debug_assert_eq!(
        resolver.has_work_tier(),
        work.is_some(),
        "marks_for_source_text -- resolver.has_work_tier()={} nhung work.is_some()={}",
        resolver.has_work_tier(),
        work.is_some()
    );

    let global_tier = load_tier(global)?;
    let work_tier = work.map(load_tier).transpose()?;

    let resolved =
        resolver.apply_override(GLOSSARY_SCOPE_KIND, &global_tier, work_tier.as_ref())?;

    // `payload[i]` la (source_term, tang, muc) cua khoa thu `i` cua `resolved` -- CUNG mot
    // thu tu voi `terms` duoi day, vi ca hai deu duyet DUNG MOT lan tren cung mot BTreeMap.
    // `term_index` cua `TermMatch` tro vao vi tri nay.
    let payload: Vec<(GlossaryTier, &GlossaryEntry)> = resolved
        .values()
        .map(|resolved_entry| {
            let tier = match resolved_entry.tier() {
                ScopeTier::Global => GlossaryTier::Global,
                ScopeTier::Work => GlossaryTier::Work,
            };
            (tier, resolved_entry.value())
        })
        .collect();
    let terms: Vec<&str> = resolved.keys().map(String::as_str).collect();

    let raw_matches = find_terms(text, &terms, lang);
    let selected = resolve_overlaps(raw_matches);

    // 🔵 THÊM 2026-08-24 (Story 3.7) — gom `source_term` của các mục CHỜ CHỐT trong tập ĐÃ
    // CHỌN (sau `resolve_overlaps`, không phải toàn bộ `payload`): tra Hán Việt cho một mục
    // đã bị một span dài hơn đè lên là công vô ích, nó không bao giờ ra dấu.
    let pending_terms: Vec<&str> = selected
        .iter()
        .map(|m| payload[m.term_index].1)
        .filter(|entry| !entry.is_confirmed())
        .map(|entry| entry.source_term.as_str())
        .collect();
    let suggestions = suggest_han_viet_batch(layers, disabled, &pending_terms);
    // `source_term -> HanVietSuggestion` -- `pending_terms`/`suggestions` cùng thứ tự, cùng
    // độ dài (`suggest_han_viet_batch` giữ nguyên vị trí đầu vào), nên zip là an toàn; khoá
    // bằng `source_term` (không bằng vị trí) để tra lại KHÔNG phụ thuộc thứ tự lặp bên dưới.
    let suggestion_by_term: BTreeMap<&str, HanVietSuggestion> =
        pending_terms.into_iter().zip(suggestions).collect();

    let boundaries = codepoint_boundaries(text);
    let marks = selected
        .into_iter()
        .map(|m| {
            let (tier, entry) = payload[m.term_index];
            let is_confirmed = entry.is_confirmed();
            // Mục ĐÃ CHỐT không đi qua `suggest_han_viet_batch` -- gán thẳng
            // `NotRequested` (§Design Notes của Story 3.7: một đề xuất cho mục đã có bản
            // dịch thật không có chỗ tiêu thụ, và nhãn "không phải tiếng Trung" sẽ SAI cho
            // một thuật ngữ chữ Hán đã chốt).
            //
            // ⚠️ GIỚI HẠN THẬT, ghi ra thay vì để người sau tưởng đã được xét (đo trong vòng
            // rà Bước 4, 2026-08-24): nhánh `if is_confirmed` này là **PHÒNG THỦ DƯ**, không
            // phải chỗ gánh. Vệ THẬT là `.filter(|entry| !entry.is_confirmed())` lúc dựng
            // `pending_terms` ngay trên -- một thuật ngữ đã chốt không bao giờ vào
            // `suggestion_by_term`, nên `unwrap_or(&NotRequested)` ở nhánh `else` đã đỡ sẵn.
            // Đo được: vô hiệu RIÊNG nhánh này ⇒ **0 ca đỏ**; phải gỡ CẢ HAI vệ mới có ca đỏ
            // (`glossary_han_viet_suggestion_contract.rs::
            // a_confirmed_chinese_mark_is_not_requested_never_not_chinese`).
            // ⇒ Giữ nó vì nó nói ra Ý ĐỊNH ngay tại chỗ đọc, nhưng ĐỪNG tin nó là hàng rào:
            // gỡ dòng `.filter(...)` kia sẽ không làm cổng nào đỏ, chỉ làm mọi Chương trả
            // thêm một lượt tra vô ích cho MỌI thuật ngữ đã chốt (món nợ có chủ Ice, gộp vào
            // phép đo NFR2 -- xem `deferred-work.md` §Deferred from: 3-7-...).
            let suggestion = if is_confirmed {
                &HanVietSuggestion::NotRequested
            } else {
                suggestion_by_term
                    .get(entry.source_term.as_str())
                    .unwrap_or(&HanVietSuggestion::NotRequested)
            };
            GlossaryMark {
                start: byte_to_codepoint(&boundaries, m.span.start),
                end: byte_to_codepoint(&boundaries, m.span.end),
                tier,
                is_confirmed,
                translation: entry.translation.clone(),
                // 🔵 THÊM 2026-08-22 (Story 3.6) — `entry` là `&GlossaryEntry` đã phân giải,
                // đọc thẳng từ `payload`; không truy vấn thêm nào (doc-comment của
                // `marks_for_source_text`).
                id: entry.id,
                source_term: entry.source_term.clone(),
                // 🔵 THÊM 2026-08-24 (Story 3.7) — xem khối `suggestion` ngay trên.
                han_viet_suggestion: suggestion.suggestion_text().map(str::to_owned),
                han_viet_status: suggestion.as_status_str(),
            }
        })
        .collect();

    Ok(marks)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.10 — XUẤT/NHẬP CSV/TSV: nửa CÓ chạm kho (nửa định dạng thuần sống ở
// `super::exchange`). Vẫn không chạm tệp — xem §Boundaries của spec 3.10.
// ═════════════════════════════════════════════════════════════════════════════════

/// Xuất **một tầng** thành `String` — gọi [`load_tier`] rồi [`super::exchange::render_tier`].
///
/// 🔴 **Không đi qua [`list_all_entries`]** — nó phát hàng bị che thành hàng THỨ HAI (cùng
/// `source_term` với hàng thắng, khác tầng), nên dùng nó ở đây sẽ sinh `source_term` trùng
/// trong tệp và lượt nhập lại va `UNIQUE INDEX idx_glossary_entry_source_term` ngay ở tầng
/// vừa xuất ra — đúng lỗi mà §Never của spec 3.10 cấm đích danh.
///
/// # Lỗi
/// [`GlossaryError::Store`] nếu [`load_tier`] thất bại (mở kho, đọc, hoặc một hàng vi phạm
/// `CHECK` mà một bản ứng dụng khác đã lỡ ghi).
pub fn export_tier(store: &Store, delimiter: Delimiter) -> Result<String, GlossaryError> {
    let tier = load_tier(store)?;
    Ok(super::exchange::render_tier(&tier, delimiter))
}

/// Phân loại `rows` đã phân tích (từ [`super::exchange::parse`]) so với tầng `tier` —
/// Story 3.10b, nhịp MỘT của lượt nhập.
///
/// 🔴 **Bọc `load_tier` để giải một vòng luẩn quẩn ranh giới, cùng tiền lệ [`export_tier`]
/// ngay trên.** `commands::glossary` cần `Vec<RowPlan>` để giữ trong `PendingImportState`
/// (AD-48 §Rule ①: kế hoạch ở lại Rust), nhưng `glossary_boundary.rs::GLOSSARY_ONLY_SURFACE`
/// cấm nó tự gọi `load_tier`. `classify` (hàm THUẦN) không bị cấm, nhưng không tự có tầng
/// ĐÍCH đã nạp để so — hàm này là đường DUY NHẤT dựng ra tham số đó mà không mở cổng.
pub fn classify_import_rows(
    global: &Store,
    work: Option<&Store>,
    tier: GlossaryTier,
    rows: &[ImportRow],
) -> Result<Vec<RowPlan>, GlossaryError> {
    let store = match tier {
        GlossaryTier::Global => global,
        GlossaryTier::Work => work.ok_or(GlossaryError::WorkTierUnavailable)?,
    };
    let existing = load_tier(store)?;
    Ok(super::exchange::classify(rows, &existing))
}

/// Lỗi ĐÁNH DẤU để buộc `store.write` rollback khi vòng lặp của [`import_into_tier`] đã tự
/// gom xong danh sách va chạm — nội dung của nó KHÔNG bao giờ được đọc lại: danh sách thật
/// đi qua kênh riêng (`Arc<Mutex<Vec<String>>>`), không qua `Display` của giá trị này. Xem
/// doc-comment của [`import_into_tier`].
fn unique_conflict_marker_error() -> SqlError {
    SqlError::FromSqlConversionFailure(
        0,
        SqlType::Text,
        "glossary import -- rollback: mot hoac nhieu hang New va UNIQUE".into(),
    )
}

/// Ghi kết quả một lượt nhập vào `tier` — **một** `store.write` (§Always: "Không ghi một
/// phần" — một lô nhập đi TRỌN trong một giao dịch, `Ok` ⇒ commit, `Err` ⇒ rollback).
///
/// 🔴 **`term_origin` LUÔN [`TermOrigin::FileImport`], KHÔNG nhận qua tham số** — cùng
/// nguyên tắc mà [`insert_manual_entry`]/[`crate::core::glossary::candidate_store::approve_candidate`]
/// đã giữ từ Story 3.2 (FR55).
///
/// 🔴 **Ba nhánh của [`RowPlanKind`], ba hành vi:**
/// - [`RowPlanKind::New`] ⇒ `INSERT`, mang `plan.created_at` NGUYÊN VĂN nếu tệp có cột đó
///   (vòng tròn xuất→nhập giữ nguyên mốc — I/O Matrix), hoặc để SQL tự sinh (`None`) nếu
///   tệp không có cột `created_at` (I/O Matrix "Vắng cột tuỳ chọn ⇒ `created_at` = hôm nay").
/// - [`RowPlanKind::Identical`] ⇒ **không ghi gì** — I/O Matrix: "không đề nghị gì, không
///   ghi".
/// - [`RowPlanKind::Conflict`] ⇒ tra `decisions` theo `source_term` (vắng mặt ⇒
///   [`ConflictDecision::KeepMine`], §Always: mặc định giữ của tôi). `KeepMine` không ghi
///   gì.
///   🔴 **SỬA 2026-08-25 (vòng rà ba lớp, P1, Ice chốt) — `TakeTheirs` ghi CHỈ cột
///   `translation`, KHÔNG BAO GIỜ chạm `note`/`category`, kể cả khi tệp CÓ mang giá trị cho
///   chúng.** Bản đầu ghi cả ba cột vô điều kiện (khuôn `update_manual_term`) — nhưng
///   `exchange.rs` điền `Category::Other`/`""` cho hàng VẮNG cột, nên nhập một tệp hai cột
///   (`source_term,translation` — đúng hình dạng mockup) rồi chọn *lấy của file* XOÁ SẠCH
///   ghi chú người dùng tự viết và HẠ phân loại `person` xuống `other`, ngược đúng §Always
///   *"Không im lặng ghi đè"*: người dùng chỉ đồng ý đổi BẢN DỊCH, không đồng ý đổi hai cột
///   kia. §I/O Matrix ba hàng mới (*"Bất đồng, người dùng lấy của file"* · *"…tệp thiếu cột
///   note/category"* · *"…tệp CÓ cột note mang giá trị khác"*) khoá đúng mệnh đề này.
///   Trigger `glossary_entry_lifecycle_is_one_way` (AD-36) vẫn đứng ở tầng SQL — một hàng ĐÃ
///   CHỐT nhận `translation = None` từ tệp qua `TakeTheirs` vẫn bị `RAISE(ABORT)` từ chối,
///   đúng I/O Matrix "Trigger AD-36 chặn lượt lùi về rỗng ⇒ `store.write_failed`, cả lô
///   rollback".
///
/// 🔴 **`GlossaryError::ImportUniqueConflict` — gom TRỌN danh sách va, và chỉ gán nhãn này
/// khi lỗi THẬT SỰ là vi phạm `UNIQUE`.**
///
/// 🔵 **SỬA 2026-08-25 (vòng rà ba lớp, P5+P6) — thiết kế lại từ đầu, hai lỗi ở bản trước:**
/// bản trước đọc lại `tier` SAU khi `store.write` trả `Err`, rồi ĐOÁN nguyên nhân bằng cách
/// tìm một hàng `New` mà `source_term` của nó NAY tồn tại trên đĩa. Phép đoán đó không xác
/// nhận lỗi GỐC thật sự là gì — nếu lô CÙNG lúc vừa có một hàng `New` (tình cờ, không liên
/// quan) đã tồn tại từ một đường ghi khác, VỪA có một `TakeTheirs` khác vi phạm trigger
/// AD-36, thì trigger mới là nguyên nhân khiến giao dịch trượt, nhưng phép đoán vẫn gán
/// nhãn `ImportUniqueConflict` — che mất đúng nguyên nhân thật. Nó cũng dừng ở va chạm ĐẦU
/// TIÊN tìm thấy, không gom hết.
///
/// Thiết kế mới kiểm NGAY tại chỗ lỗi xảy ra, trong chính giao dịch: mỗi `INSERT` của một
/// hàng `New` thất bại được hỏi ngay bằng [`is_unique_constraint_violation`] — nếu ĐÚNG,
/// `source_term` được GOM vào một danh sách cục bộ và vòng lặp TIẾP TỤC (không `?`, không
/// rollback ngay) để gom hết mọi va chạm khác trong CÙNG lô; nếu SAI (bất kỳ lỗi nào khác —
/// bao gồm trigger AD-36), giao dịch abort NGAY qua `?`/`return Err` với lỗi SQL GỐC, không
/// bị gán nhãn `ImportUniqueConflict`. Sau vòng lặp, nếu danh sách gom được không rỗng, hàm
/// trả một lỗi ĐÁNH DẤU ([`unique_conflict_marker_error`]) để buộc `Store::write` rollback
/// TRỌN (§Always: "0 hàng ghi" — kể cả những hàng `New` khác đã `INSERT` thành công trước đó
/// trong cùng vòng lặp). Danh sách va chạm thật đi ra ngoài closure qua một kênh riêng
/// (`Arc<Mutex<Vec<String>>>`) — không qua `Display`/chuỗi lỗi SQL, nên không cần phân tích
/// chuỗi thô ở phía đọc kết quả.
///
/// # Lỗi
/// [`GlossaryError::WorkTierUnavailable`] nếu `tier` là [`GlossaryTier::Work`] mà `work` là
/// `None`. [`GlossaryError::ImportUniqueConflict`]/[`GlossaryError::Store`] — xem trên.
pub fn import_into_tier(
    global: &Store,
    work: Option<&Store>,
    tier: GlossaryTier,
    plans: &[RowPlan],
    decisions: &BTreeMap<String, ConflictDecision>,
) -> Result<ImportSummary, GlossaryError> {
    let store = match tier {
        GlossaryTier::Global => global,
        GlossaryTier::Work => work.ok_or(GlossaryError::WorkTierUnavailable)?,
    };

    let plans_owned: Vec<RowPlan> = plans.to_vec();
    let decisions_owned: BTreeMap<String, ConflictDecision> = decisions.clone();

    // Kênh riêng mang danh sách va chạm THẬT ra khỏi closure -- xem khối 🔵 ở doc-comment.
    let conflicts: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let conflicts_for_closure = Arc::clone(&conflicts);

    let result = store.write(move |tx: &Transaction<'_>| {
        let mut summary = ImportSummary::default();
        let mut local_conflicts: Vec<String> = Vec::new();

        for plan in &plans_owned {
            match &plan.kind {
                RowPlanKind::New => {
                    let inserted = insert_entry_row(
                        tx,
                        &plan.source_term,
                        plan.translation.as_deref(),
                        &plan.note,
                        plan.category.as_str(),
                        TermOrigin::FileImport.as_str(),
                        plan.created_at.as_deref(),
                    );
                    match inserted {
                        Ok(_) => summary.inserted += 1,
                        // ĐÚNG là va UNIQUE -- gom lại, KHÔNG abort ngay, để vòng lặp tiếp
                        // tục tìm hết va chạm còn lại trong cùng lô (P6).
                        Err(e) if is_unique_constraint_violation(&e) => {
                            local_conflicts.push(plan.source_term.clone());
                        }
                        // Bất kỳ lỗi nào KHÁC (kể cả một `UNIQUE` ở một ràng buộc khác, nếu
                        // có ngày nào đó) -- abort ngay với lỗi GỐC, không gán nhãn sai (P5).
                        Err(e) => return Err(e),
                    }
                }
                RowPlanKind::Identical => {
                    summary.identical += 1;
                }
                RowPlanKind::Conflict { existing_id, .. } => {
                    let decision = decisions_owned
                        .get(&plan.source_term)
                        .copied()
                        .unwrap_or(ConflictDecision::KeepMine);
                    if decision == ConflictDecision::TakeTheirs {
                        // 🔴 CHỈ `translation` -- xem khối 🔴 P1 ở doc-comment trên. `note`/
                        // `category` KHÔNG đi vào câu UPDATE này, dù `plan` có mang gì.
                        let changed = tx.execute(
                            "UPDATE glossary_entry SET translation = ?1 WHERE id = ?2",
                            (&plan.translation, existing_id),
                        )?;
                        // ⚠️ Ca này KHÔNG có mặt trong §I/O Matrix của spec (chỉ ca "New va
                        // UNIQUE giữa chừng" được liệt) — nhưng "0 hàng đổi" đi qua trong im
                        // lặng đúng là lớp lỗi trung tâm mà AGENTS.md cấm (`Known pitfalls`:
                        // "Rỗng IM LẶNG"). Hàng `existing_id` biến mất GIỮA lúc `classify()`
                        // chụp ảnh và lúc giao dịch này chạy (đua với một lượt Xoá khác) ⇒
                        // trả lỗi để CẢ LÔ rollback, thay vì báo thành công cho một `UPDATE`
                        // không đổi gì. Rơi về `store.write_failed` chung (không phải một
                        // biến thể `GlossaryError` riêng — ca này ngoài phạm vi đã ký của
                        // story, ghi nợ ở đây thay vì bịa một tên mới không ai nghiệm thu).
                        if changed == 0 {
                            return Err(row_missing_error(0, "glossary_entry", *existing_id));
                        }
                        summary.updated += 1;
                    }
                }
            }
        }

        if !local_conflicts.is_empty() {
            *conflicts_for_closure.lock().unwrap_or_else(|p| p.into_inner()) = local_conflicts;
            return Err(unique_conflict_marker_error());
        }

        Ok(summary)
    });

    match result {
        Ok(summary) => Ok(summary),
        Err(e) => {
            let collected = conflicts.lock().unwrap_or_else(|p| p.into_inner()).clone();
            if !collected.is_empty() {
                Err(GlossaryError::ImportUniqueConflict { source_terms: collected })
            } else {
                Err(GlossaryError::from(e))
            }
        }
    }
}
