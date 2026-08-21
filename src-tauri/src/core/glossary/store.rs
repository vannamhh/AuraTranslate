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

use crate::core::i18n::{IpcError, MessageKey};
use crate::core::matching::{MatchLang, TermMatch, find_terms};
use crate::core::scope::{ScopeError, ScopeResolver, Tier as ScopeTier};
use crate::core::store::{ReadHandle, SqlError, SqlResult, SqlType, Store, StoreError, Transaction};

use super::entry::{Category, GlossaryEntry, GlossaryMark, GlossaryTier, TermOrigin};

/// Khoá dây của `ScopeKind::Glossary` (`core/scope/kinds.rs:162`), chép lại đây làm
/// literal — module này không được `use` `ScopeKind`.
const GLOSSARY_SCOPE_KIND: &str = "glossary";

/// Câu `INSERT` DÙNG CHUNG cho **cả hai** đường ghi vào `glossary_entry` — Story 3.2.
///
/// 🔴 **Chỉ hai chỗ gọi được phép tồn tại, và cả hai đều ở trong `core/glossary/**`:**
/// [`insert_manual_entry`] (ngay dưới, luôn `term_origin = manual`) và
/// [`crate::core::glossary::candidate_store::approve_candidate`] (luôn suy `term_origin`
/// từ `candidate_origin` của chính hàng ứng viên). Đây là vế CẤU TRÚC của FR55 ("không cơ
/// chế nào được tự ghi vào Glossary"): trước Story 3.2, `insert_entry` cũ nhận
/// `term_origin: TermOrigin` từ NƠI GỌI — một module quét chỉ cần truyền
/// `TermOrigin::ImportScan` là ghi thẳng, biên dịch sạch, qua mọi cổng. Thu hẹp về **một**
/// hàm `pub(super)` không tham số `term_origin` tự do làm vi phạm đó KHÔNG BIỂU DIỄN ĐƯỢC:
/// mọi giá trị `term_origin` đi vào đây đều đã bị khoá bởi CHÍNH LOGIC của chỗ gọi (hằng
/// `manual`, hoặc một `CandidateOrigin::to_term_origin()` toàn phần), không phải một tham
/// số người viết mã bên ngoài `core/glossary/**` có thể tự ý đặt.
///
/// ⚠️ Chữ ký nhận **chuỗi đã chuẩn bị sẵn** (đã trim, đã `as_str()`) — không tự trim, không
/// tự gọi `Category::as_str()`/`TermOrigin::as_str()`. Cắt khoảng trắng là việc của TỪNG
/// chỗ gọi vì hai chỗ gọi cắt hai đầu vào khác nhau (`source_term`+`translation` của
/// `insert_manual_entry`; `translation` — `source_term` đã được `insert_candidate` cắt từ
/// trước — của `approve_candidate`).
pub(super) fn insert_entry_row(
    tx: &Transaction<'_>,
    source_term: &str,
    translation: Option<&str>,
    note: &str,
    category: &str,
    term_origin: &str,
) -> SqlResult<i64> {
    tx.execute(
        "INSERT INTO glossary_entry
            (source_term, translation, note, category, term_origin, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        (&source_term, &translation, &note, &category, &term_origin),
    )?;
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
}

impl std::fmt::Display for GlossaryError {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlossaryError::Store(e) => write!(f, "glossary[store] {e}"),
            GlossaryError::Scope(e) => write!(f, "glossary[scope] {e}"),
            GlossaryError::EntryMissing => write!(f, "glossary[entry_missing]"),
            GlossaryError::WorkTierUnavailable => write!(f, "glossary[work_tier_unavailable]"),
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

    insert_manual_entry(store, source_term, translation, note, category).map_err(GlossaryError::from)
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
    boundaries.binary_search(&byte_offset).unwrap_or_else(|insert_at| insert_at)
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
pub fn marks_for_source_text(
    resolver: &ScopeResolver,
    global: &Store,
    work: Option<&Store>,
    text: &str,
    lang: MatchLang,
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

    let boundaries = codepoint_boundaries(text);
    let marks = selected
        .into_iter()
        .map(|m| {
            let (tier, entry) = payload[m.term_index];
            GlossaryMark {
                start: byte_to_codepoint(&boundaries, m.span.start),
                end: byte_to_codepoint(&boundaries, m.span.end),
                tier,
                is_confirmed: entry.is_confirmed(),
                translation: entry.translation.clone(),
            }
        })
        .collect();

    Ok(marks)
}
