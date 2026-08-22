//! SQL của `glossary_candidate` — bảng chờ ứng viên TÁCH RIÊNG khỏi `glossary_entry`
//! (AD-20, AD-36) — Story 3.2.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 `approve_candidate`/`reject_candidate` ĐỌC `resolution` TRƯỚC KHI GHI — lớp Rust
//! CHO MỘT LỖI ĐỌC ĐƯỢC, lớp trigger cho BẢO ĐẢM
//! ─────────────────────────────────────────────────────────────────────────────
//! Cùng khuôn hai lớp mà `.trim()`/`CHECK` đã dùng ở Story 3.1: một `UPDATE … WHERE id =
//! ?1 AND resolution IS NULL` một mình sẽ khớp **0 hàng** cho CẢ HAI ca "id không tồn tại"
//! và "id đã quyết" — giống hệt ca `confirm_translation` "0 hàng vẫn `Ok`"
//! (`store.rs::confirm_translation`) mà doc-comment của nó cảnh báo đừng nhân bản. Ở đây
//! hậu quả nặng hơn: "id đã quyết" mà báo `Ok` như "id không tồn tại" là chỗ một lượt
//! duyệt hàng loạt (Story 3.8) hay một cú bấm đúp có thể im lặng bỏ qua một ứng viên đã
//! bỏ mà không ai biết đường ghi đó bị từ chối.
//!
//! ⇒ Cả hai hàm mở đầu bằng một `tx.query_row` đọc `resolution` (cùng cột khác mục đích ở
//! mỗi hàm). 0 hàng ⇒ lỗi tự nhiên của `rusqlite` (`QueryReturnedNoRows`) lan qua `?`,
//! phân biệt được với nhánh dưới đây bằng chính hình dạng lỗi. `resolution` đã có giá trị
//! ⇒ [`already_decided_error`] — một lỗi **đọc được**, không phải một chuỗi chẩn đoán của
//! SQLite. Trigger `glossary_candidate_resolution_is_one_way` (`schema.rs`) là lớp BẢO
//! ĐẢM đứng sau: nó vẫn nổ nếu lớp Rust này bị bỏ qua (một chỗ gọi tương lai viết `UPDATE`
//! thẳng), đúng khuôn "kỷ luật không đủ, cấu trúc mới đủ" mà toàn bộ Epic 3 dựa vào.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! `approve_candidate` LÀ MỘT GIAO DỊCH `store.write` — `resolution` VÀ `glossary_entry`
//! CÙNG SỐNG HOẶC CÙNG CHẾT
//! ─────────────────────────────────────────────────────────────────────────────
//! `UPDATE glossary_candidate` và `INSERT glossary_entry` chạy trong CÙNG một closure của
//! `Store::write`, tức cùng một `Transaction`. Một `CHECK` trắng ở `translation` (chuỗi
//! rỗng/khoảng trắng) làm `INSERT` thất bại ⇒ toàn bộ đóng gồm cả `UPDATE resolution` vừa
//! chạy trước đó bị ROLLBACK — không có trạng thái nửa vời nào "đã quyết mà chưa sinh mục
//! Glossary".
//!
//! ⚠️ Mọi chuỗi trong `src-tauri/src/**` viết KHÔNG DẤU; doc-comment có dấu là hợp lệ.

use crate::core::store::{SqlError, SqlResult, SqlType, Store, StoreError, Transaction};

use super::candidate::{CandidateOrigin, GlossaryCandidate, Resolution};
use super::entry::Category;
use super::store::insert_entry_row;

/// Chèn một ứng viên mới — LUÔN vào bảng chờ, KHÔNG BAO GIỜ vào `glossary_entry` (AD-20).
///
/// # Lỗi
/// [`StoreError::WriteFailed`] nếu `source_term` (đã cắt khoảng trắng) đã tồn tại
/// (`UNIQUE INDEX idx_glossary_candidate_source_term`) — đúng cơ chế chặn "quét lại một
/// chuỗi đã bỏ/đã duyệt không quay lại bảng chờ": hàng cũ ở lại (không `DELETE`), nên
/// `UNIQUE` va vào chính nó. Cũng `WriteFailed` nếu `source_term` trắng hoàn toàn (`CHECK`).
pub fn insert_candidate(
    store: &Store,
    source_term: &str,
    candidate_origin: CandidateOrigin,
) -> Result<i64, StoreError> {
    // Cùng lý do cắt khoảng trắng biên đã ghi ở `insert_manual_entry` (store.rs): `"
    // 慕容"` và `"慕容"` không được thành hai hàng dưới một chỉ mục tự xưng là "một
    // thuật ngữ, một mục".
    let source_term = source_term.trim().to_owned();
    let candidate_origin = candidate_origin.as_str();

    store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO glossary_candidate (source_term, candidate_origin, created_at)
             VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            (&source_term, &candidate_origin),
        )?;
        Ok(tx.last_insert_rowid())
    })
}

/// Mọi ứng viên **chờ duyệt** — `resolution IS NULL`.
///
/// ⚠️ `ORDER BY source_term` là đối chiếu BYTE — vô nghĩa cho chữ Hán/tiếng Việt, và
/// `WHERE resolution IS NULL` chưa có chỉ mục riêng. Cả hai là món nợ có chủ (Story 3.8,
/// `deferred-work.md`) chứ không phải bị bỏ sót: story này chưa có bề mặt duyệt hàng loạt
/// nào để cần một thứ tự có ý nghĩa với người dùng, và bảng chờ hôm nay không đủ lớn để
/// đo được chi phí thiếu chỉ mục.
pub fn pending_candidates(store: &Store) -> Result<Vec<GlossaryCandidate>, StoreError> {
    store.read(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, source_term, candidate_origin, resolution, created_at, \
                    occurrence_count, context_example
             FROM glossary_candidate
             WHERE resolution IS NULL
             ORDER BY source_term",
        )?;
        let mut rows = stmt.query([])?;

        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let candidate_origin_raw: String = row.get(2)?;
            let resolution_raw: Option<String> = row.get(3)?;

            out.push(GlossaryCandidate {
                id: row.get(0)?,
                source_term: row.get(1)?,
                candidate_origin: decode_candidate_origin(2, &candidate_origin_raw)?,
                resolution: resolution_raw
                    .as_deref()
                    .map(|raw| decode_resolution(3, raw))
                    .transpose()?,
                created_at: row.get(4)?,
                // 🔵 THÊM 2026-08-22 (Story 3.5) — hai cột bước di trú 14. Hàng CŨ (Story
                // 3.2, trước lượt quét đầu tiên) đọc `occurrence_count = 0`/
                // `context_example = NULL` đúng giá trị `DEFAULT`/nullable của cột.
                occurrence_count: row.get(5)?,
                context_example: row.get(6)?,
            });
        }
        Ok(out)
    })
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.5 — hàm ghi LÔ, chỗ gọi sản phẩm đầu tiên NGOÀI `core/glossary/**`
// ═════════════════════════════════════════════════════════════════════════════════

/// Ghi lô ứng viên từ một lượt quét khi nhập — MỘT [`Store::write`], `prepare_cached` một
/// lần rồi lặp (khuôn `commands/segment.rs::insert_segments`), `ON CONFLICT (source_term)
/// DO NOTHING`.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 LỌC `glossary_entry` NGAY TRONG CÂU `INSERT` — không một lượt `SELECT` riêng trước đó
/// ─────────────────────────────────────────────────────────────────────────────
/// `WHERE NOT EXISTS (SELECT 1 FROM glossary_entry WHERE source_term = ?1)` chặn đúng ca
/// *"Đã có trong Glossary"* của I/O Matrix — một chuỗi đã là `glossary_entry` (nhập tay
/// trước khi lượt quét chạy tới, hoặc đã được duyệt ở một Chương khác trong cùng phiên)
/// không bao giờ được ghi vào bảng chờ, đóng đúng món nợ có chủ của story này
/// (`deferred-work.md:5606-5617`, "quét không được sinh ứng viên trùng `source_term` với
/// `glossary_entry`, nếu không `approve_candidate` hỏng vĩnh viễn").
///
/// `ON CONFLICT (source_term) DO NOTHING` chặn ca *"Đã từng bị bỏ"* — một `source_term` đã
/// có hàng `glossary_candidate` (bất kể `resolution` là gì) không được chèn hàng thứ hai;
/// `idx_glossary_candidate_source_term` là `UNIQUE`, và câu lệnh này **không bao giờ**
/// `UPDATE` cột nào của hàng cũ — `resolution` một chiều (`glossary_candidate_resolution_is_
/// one_way`) không bị chạm.
///
/// Trả `(đã chèn, đã bỏ qua)`, không `()`: một cặp `(0, 0)` (quét ra 0 ứng viên) và một cặp
/// `(0, N)` (quét ra N ứng viên, cả N đều trùng dữ liệu đã có) đều là *0 hàng mới*, nhưng
/// chúng là hai câu chuyện khác nhau — cặp số là thứ duy nhất phân biệt được *"quét chưa
/// chạy"* với *"quét đã chạy và không có gì mới"* (§Boundaries: *"Mọi số đếm báo ra, kể cả
/// 0 và kể cả số bị bỏ qua"*).
///
/// # Lỗi
/// [`StoreError::WriteFailed`] nếu đường ghi trượt (kho đóng giữa chừng, …) — toàn lô
/// rollback cùng nhau, đúng khuôn một giao dịch của `Store::write`.
pub fn insert_import_scan_candidates(
    store: &Store,
    candidates: &[crate::core::glossary::scan::ScanCandidate],
) -> Result<(i64, i64), StoreError> {
    // Sở hữu tường minh: job ghi chạy trên luồng writer nên nó phải `Send + 'static` —
    // `candidates` không thoả (mượn), nên sao một bản trước khi `move` vào closure. Mọi
    // hàng của LÔ NÀY mang cùng `candidate_origin`: hàm này chỉ phục vụ một lượt quét khi
    // nhập (`ImportScan`), không nhận xuất xứ từ chỗ gọi.
    let rows: Vec<(String, i64, String)> = candidates
        .iter()
        .map(|c| (c.source_term.clone(), c.occurrence_count, c.context_example.clone()))
        .collect();

    store.write(move |tx: &Transaction<'_>| {
        let mut stmt = tx.prepare_cached(
            "INSERT INTO glossary_candidate
                (source_term, candidate_origin, occurrence_count, context_example, created_at)
             SELECT ?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE NOT EXISTS (SELECT 1 FROM glossary_entry WHERE source_term = ?1)
             ON CONFLICT (source_term) DO NOTHING",
        )?;

        let mut inserted = 0i64;
        let mut skipped = 0i64;
        let candidate_origin = CandidateOrigin::ImportScan.as_str();
        for (source_term, occurrence_count, context_example) in &rows {
            let changed = stmt.execute((source_term, candidate_origin, occurrence_count, context_example))?;
            if changed > 0 {
                inserted += 1;
            } else {
                skipped += 1;
            }
        }
        Ok((inserted, skipped))
    })
}

/// Duyệt ứng viên `id`: đặt `resolution = 'approved'` **và** chèn `glossary_entry` mang
/// `term_origin` suy từ `candidate_origin` của chính hàng ứng viên — trong MỘT giao dịch.
///
/// `translation = None` ⇒ mục Glossary sinh ra ở trạng thái *chờ chốt* (Story 3.1, FR114)
/// — duyệt một ứng viên không bắt buộc phải chốt bản dịch ngay.
///
/// # Lỗi
/// [`StoreError::WriteFailed`] khi: `id` không khớp hàng nào; ứng viên `id` ĐÃ quyết (đã
/// duyệt hoặc đã bỏ — xem doc-comment đầu module); `translation` là `Some("")`/khoảng
/// trắng (`CHECK` của `GLOSSARY_ENTRY_DDL`, qua `insert_entry_row`); hoặc `source_term`
/// của ứng viên trùng một `glossary_entry` đã có (`UNIQUE INDEX
/// idx_glossary_entry_source_term`) — ca này để lại ứng viên vĩnh viễn ở bảng chờ, món nợ
/// có chủ cho Story 3.5 (`deferred-work.md`).
pub fn approve_candidate(
    store: &Store,
    id: i64,
    translation: Option<&str>,
    category: Category,
) -> Result<i64, StoreError> {
    // Cùng lý do cắt khoảng trắng biên đã ghi ở `insert_manual_entry` (store.rs).
    // `source_term` KHÔNG cần cắt lại ở đây — `insert_candidate` đã cắt nó lúc ứng viên
    // ra đời, và giá trị đọc lại từ đĩa đã là dạng đã cắt.
    let translation = translation.map(|t| t.trim().to_owned());
    let category = category.as_str();

    store.write(move |tx: &Transaction<'_>| {
        let (source_term, candidate_origin_raw, resolution_raw): (String, String, Option<String>) =
            tx.query_row(
                "SELECT source_term, candidate_origin, resolution
                 FROM glossary_candidate WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?;

        if let Some(resolution_raw) = resolution_raw {
            let resolution = decode_resolution(2, &resolution_raw)?;
            return Err(already_decided_error(2, id, resolution));
        }

        let term_origin = decode_candidate_origin(1, &candidate_origin_raw)?.to_term_origin();

        tx.execute(
            "UPDATE glossary_candidate SET resolution = 'approved' WHERE id = ?1",
            [id],
        )?;

        insert_entry_row(
            tx,
            &source_term,
            translation.as_deref(),
            "",
            category,
            term_origin.as_str(),
        )
    })
}

/// Bỏ ứng viên `id`: đặt `resolution = 'rejected'`. Hàng ứng viên KHÔNG bị xoá (xem
/// doc-comment `GLOSSARY_CANDIDATE_DDL`) — đó là thứ làm `UNIQUE (source_term)` chặn được
/// việc quét lại chèn cùng chuỗi.
///
/// # Lỗi
/// [`StoreError::WriteFailed`] khi: `id` không khớp hàng nào; ứng viên `id` ĐÃ quyết —
/// gồm cả ca `id` đã ĐƯỢC DUYỆT: mục Glossary đã sinh ra không bao giờ bị gỡ bởi một lượt
/// `reject_candidate` muộn màng, hai bảng không được phép nói ngược nhau.
pub fn reject_candidate(store: &Store, id: i64) -> Result<(), StoreError> {
    store.write(move |tx: &Transaction<'_>| {
        let resolution_raw: Option<String> = tx.query_row(
            "SELECT resolution FROM glossary_candidate WHERE id = ?1",
            [id],
            |r| r.get(0),
        )?;

        if let Some(resolution_raw) = resolution_raw {
            let resolution = decode_resolution(0, &resolution_raw)?;
            return Err(already_decided_error(0, id, resolution));
        }

        tx.execute(
            "UPDATE glossary_candidate SET resolution = 'rejected' WHERE id = ?1",
            [id],
        )?;
        Ok(())
    })
}

/// Lỗi Rust-layer "đọc được" cho ca `resolution` đã có giá trị — phân biệt được với
/// `SqlError::QueryReturnedNoRows` ("id không tồn tại") bằng chính HÌNH DẠNG lỗi, không
/// chỉ bằng nội dung chuỗi. Dùng lại đúng cơ chế mà [`decode_candidate_origin`] đã dùng để
/// dựng một `SqlError` mang thông điệp tự chọn — không có biến thể `rusqlite::Error` nào
/// đặt tên cho "một quy tắc nghiệp vụ bị vi phạm", nên `FromSqlConversionFailure` (vốn
/// mang một `Box<dyn Error>` tự do) là chỗ mượn hợp lý nhất, cùng khuôn `store.rs::decode_category`.
///
/// ⚠️ Nhận `Resolution` ĐÃ GIẢI MÃ, không phải chuỗi thô — cả hai chỗ gọi phải đi qua
/// [`decode_resolution`] TRƯỚC khi tới đây. Không có bước đó, "mỗi biến thể `Resolution`
/// đi vòng qua giải mã" chỉ đúng cho `pending_candidates` (nơi `resolution` luôn `NULL` do
/// chính `WHERE` của nó) — tức nhánh `Some(_)` của `decode_resolution` không bao giờ chạy
/// trên đường sản phẩm, đúng lớp "khoảng trống nghiệm thu" mà Story 3.1 từng bắt cho
/// `decode_category`/`decode_term_origin`.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (lượt rà soát ba lớp) — `col` là THAM SỐ, không hằng `2` viết
/// cứng.** Bản trước khoá cứng chỉ số cột `2` — đúng cho `approve_candidate` (`SELECT`
/// BA cột, `resolution` ở vị trí 2) nhưng SAI cho `reject_candidate` (`SELECT` MỘT cột
/// duy nhất, `resolution` ở vị trí 0). Lỗi không lộ ra ở test: `col` chỉ đi vào
/// `SqlError::FromSqlConversionFailure` như một con số chẩn đoán, không ảnh hưởng phán
/// quyết `Err`/`Ok`, nên hiệu ứng thật (chỉ số cột chẩn đoán sai) không cổng nào bắt được
/// bằng `matches!`. Đúng khuôn [`decode_candidate_origin`]/[`decode_resolution`]: mỗi chỗ
/// gọi tự truyền chỉ số THẬT của chính truy vấn nó.
fn already_decided_error(col: usize, id: i64, resolution: Resolution) -> SqlError {
    SqlError::FromSqlConversionFailure(
        col,
        SqlType::Text,
        format!(
            "glossary_candidate id={id} da co resolution={resolution} -- khong the quyet \
             lai (vong doi mot chieu, AD-36)"
        )
        .into(),
    )
}

/// `candidate_origin` trên đĩa đã đi qua `CHECK (candidate_origin IN (…))` của
/// `GLOSSARY_CANDIDATE_DDL` ở mọi lượt `INSERT` — một chuỗi lạ ở đây chỉ xảy ra nếu đĩa đã
/// trôi khỏi đường ghi của chính module này. Cùng nguyên tắc `store.rs::decode_term_origin`:
/// TRẢ LỖI, không rơi về một giá trị mặc định trông đáng tin.
fn decode_candidate_origin(col: usize, raw: &str) -> SqlResult<CandidateOrigin> {
    CandidateOrigin::from_wire(raw).ok_or_else(|| {
        SqlError::FromSqlConversionFailure(
            col,
            SqlType::Text,
            format!(
                "glossary_candidate.candidate_origin tren dia khong khop CHECK -- gia tri: {raw:?}"
            )
            .into(),
        )
    })
}

/// Cùng lý do [`decode_candidate_origin`], cho cột `resolution` khi nó KHÔNG `NULL`.
fn decode_resolution(col: usize, raw: &str) -> SqlResult<Resolution> {
    Resolution::from_wire(raw).ok_or_else(|| {
        SqlError::FromSqlConversionFailure(
            col,
            SqlType::Text,
            format!("glossary_candidate.resolution tren dia khong khop CHECK -- gia tri: {raw:?}")
                .into(),
        )
    })
}
