use super::*;
use crate::core::segment::split::split_source_text;

/// Kết quả một lượt tách tường minh — thứ đi ra qua dây.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SplitOutcome {
    /// Chương vừa được tách.
    pub chapter_id: i64,
    /// Số hàng `segment` vừa ghi xuống. **0 là một giá trị hợp lệ** — một Chương chỉ chứa
    /// khoảng trắng cho 0 segment, và đó là hành vi đúng (ca biên ① của bộ tách).
    pub segment_count: usize,
}

/// Chèn các hàng `segment` của một Chương — **dùng chung** giữa đường nhập
/// ([`crate::commands::project::create_work`]) và lệnh tách tường minh.
///
/// 🔴 **Nhận `&Transaction`, không nhận `&Store`.** Đó là toàn bộ điểm của hàm này: AC13 đòi
/// segment ghi xuống **CÙNG** giao dịch với hàng `chapter` sinh ra chúng, và một chữ ký nhận
/// `&Store` sẽ mở một giao dịch **thứ hai** — tức dựng lại đúng trạng thái
/// *"một Chương tồn tại mà segment của nó chưa tồn tại"* mà story này tồn tại để dọn.
///
/// ⚠️ `ord` đánh số **từ 1**, liên tục, không lỗ — cùng gốc với `chapter.ord`, và Story 2.10
/// (*"segment kế tiếp"*) đứng trên giả định đó.
///
/// ⚠️ `is_paragraph_end` đi xuống dạng `INTEGER` 0/1: SQLite không có kiểu boolean, và tầng
/// Rust là chỗ cưỡng chế giá trị hợp lệ (cùng khuôn `chapter.status`).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `prepare_cached` MỘT LẦN, KHÔNG `tx.execute` MỖI HÀNG — Story 2.2 · AC17 · Task 8
/// ─────────────────────────────────────────────────────────────────────────────
/// Bản trước gọi `tx.execute` với SQL **literal bên trong vòng lặp**, nên `rusqlite` parse
/// lại câu lệnh **mỗi hàng**. `deferred-work.md §*Deferred from: Story 1.22 — bề mặt dữ liệu thật THỨ HAI (2026-08-11)*` ghi món này với chủ là Story 2.2
/// và ghi thẳng lý do hoãn: *"hoãn vì **chưa ai đo**, không phải vì nó nhỏ"*.
///
/// **Đã đo, 2026-08-12, `cargo test --release` trên macOS, 9.850 hàng — quy mô THẬT của
/// Chương lớn nhất có thật, ba lượt:**
///
/// | lượt | `tx.execute` literal mỗi hàng | `prepare_cached` một lần | chênh |
/// |---|---|---|---|
/// | 1 | 105,51 ms | 44,76 ms | **60,75 ms** (57,6 %) |
/// | 2 | 106,90 ms | 49,75 ms | **57,15 ms** (53,5 %) |
/// | 3 | 112,47 ms | 48,28 ms | **64,19 ms** (57,1 %) |
///
/// ⇒ Vá, và lý do là con số chứ không phải linh cảm: **~60 ms tiết kiệm được** nằm **trên**
/// trần một frame của NFR2 (50 ms) chỉ bằng một mình nó, và nó nằm trong closure của
/// `Store::write` — tức trên writer **duy nhất, nối tiếp** của AD-11, nơi nó chặn **mọi**
/// lượt ghi khác của tiến trình. Cùng điểm nghẽn mà `commands/project/` đã kéo
/// `split_source_text` ra ngoài để né.
///
/// ⚠️ `prepare_cached` (không phải `prepare`): bộ nhớ đệm sống trên **kết nối**, mà kết nối
/// ghi là một kết nối dài hạn của pool — nên Chương **thứ hai** trở đi không phải parse lại
/// một lần nào nữa. Với `prepare`, mỗi lượt gọi hàm này vẫn mất đúng một lượt parse.
pub(crate) fn insert_segments(
    tx: &Transaction<'_>,
    chapter_id: i64,
    segments: &[WovenSegment],
) -> SqlResult<()> {
    // 🔴 `is_target_paragraph_end` SET TƯỜNG MINH, không để `DEFAULT 0` cấp — Story 2.5d, AC2.
    //
    // ⚠️ Đây là chỗ mà bước di trú 9 **không** với tới được, và nó là ca THƯỜNG NHẤT của AC2.
    // Bước 9 backfill các hàng **đã có trên đĩa**; một Chương nhập **sau** lượt di trú đi qua
    // đúng câu `INSERT` này, nơi `DEFAULT 0` của `ALTER TABLE` là thứ duy nhất cấp giá trị.
    // ⇒ Không có `?5` dưới đây, mọi Chương mới có cờ đích **tắt hết** trong khi cờ nguồn bật
    // đúng chỗ — tức "bản dịch soi gương bản gốc" sai ngay từ giây đầu tiên, và sai **im
    // lặng**: bề mặt tiêu thụ của cột này là đường xuất (Epic 8), chưa tồn tại.
    // 🔴 Đo được, không suy: ca `a_freshly_imported_chapter_mirrors_the_source_flag_...` đỏ
    // với `[(2, true, false), (3, true, false)]` trước lượt sửa này.
    //
    // ⚠️ Và **một giá trị, hai cột** ở đây là đúng AD-46 chứ không phải một bản sao thừa: cờ
    // đích *bắt đầu* bằng cờ nguồn rồi sống độc lập. Bộ tách vẫn chỉ sinh **một** cờ
    // (`SplitSegment`), nên không có nguồn sự thật thứ hai nào được dựng ở đây.
    // 🔴 `translation_origin` SET TƯỜNG MINH, không để `DEFAULT ''` cấp — Story 2.7, AC1.
    //
    // ⚠️ **Cùng bài học, cột thứ hai liên tiếp** — xem khối ngay trên. Bước di trú 11 backfill
    // các hàng **đã có trên đĩa**; một Chương nhập **sau** lượt di trú đi qua đúng câu `INSERT`
    // này. Ở đây giá trị đúng **trùng** với `DEFAULT ''`, nên bỏ `?6` đi thì hôm nay **không ca
    // nào đỏ** — và đó chính là lý do phải viết nó ra: ngày Epic 6 dựng đường nhập song ngữ
    // (FR115), giá trị đúng lúc `INSERT` **thôi là mặc định**, và một cột im lặng lấy giá trị
    // sai sẽ không có gì báo. Một câu `INSERT` khai đủ mọi cột nó sở hữu là thứ làm lượt sửa đó
    // thành một dòng, không một cuộc chẩn đoán.
    //
    // 🔴 Và giá trị `''` ở đây là một **mệnh đề**, không một chỗ trống: một segment vừa tách ra
    // từ văn bản nguồn **chưa có bản dịch**, nên nó chưa có xuất xứ nào để khai. Cho nó
    // `TRANSLATION_ORIGIN_SELF` là ký thay người dùng đúng lớp lỗi mà `DEFAULT 'draft'` của
    // bước 7 đã ghi bằng chữ.
    //
    // 🔴 `role` SET TƯỜNG MINH, không để `DEFAULT` của `ALTER TABLE` cấp — Story 6.13, §Always.
    // Bước di trú 21 (`SEGMENT_ROLE_DDL`) không có `DEFAULT` (cột `NULL`-able), nên bỏ `?7` ở
    // đây vẫn cho SQLite chèn `NULL` một cách IM LẶNG — không một ca đọc thô 14 cột nào đỏ, vì
    // `NULL` trùng đúng giá trị mà đa số hàng (segment văn xuôi) cần. Cái giá của việc BỎ SÓT
    // chỉ lộ ra ở ĐÚNG những hàng vai — cùng lớp lỗi mà cột `translation_origin` (`?6`) đã ghi
    // bằng chữ ở khối ngay trên, nhưng ở đây KHÔNG có sự trùng hợp "giá trị đúng = DEFAULT" để
    // mà che giấu nó một phần: một `alt`/`caption` bỏ `?7` sẽ luôn ghi `role = NULL`, tức chưa
    // bao giờ mang được vai của nó xuống đĩa.
    let mut stmt = tx.prepare_cached(
        "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, \
         is_target_paragraph_end, translation_origin, role, baseline_target_text, \
         baseline_translation_origin, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, '', ?6, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
    )?;
    for (index, segment) in segments.iter().enumerate() {
        let ord = i64::try_from(index).unwrap_or(i64::MAX).saturating_add(1);
        let paragraph_end = i64::from(segment.is_paragraph_end);
        let role: Option<&str> = segment.role.map(crate::core::segment::role::SegmentRole::as_str);
        stmt.execute((
            chapter_id,
            ord,
            &segment.text,
            paragraph_end,
            paragraph_end,
            TRANSLATION_ORIGIN_NONE,
            role,
        ))?;
    }
    Ok(())
}

/// **THÊM 2026-09-11 (Story 6.16, FR115/AD-47 ③)** — chèn segment ĐÃ CẶP nguồn/đích của
/// đường nhập song ngữ. Hàm RIÊNG với [`insert_segments`] (không sửa hàm đó — §Always spec
/// 6.16: "existing path unchanged"): câu `INSERT` ở đây khai THÊM cột `target_text`, đúng
/// AD-47 ③ ("each segment's `target_text` and `translation_origin = bilingual_import` are
/// written in the same INSERT"). Cùng khuôn `insert_segments`: nhận `&Transaction`, không
/// `&Store` (AC13 — cùng giao dịch với hàng `chapter`), `prepare_cached` một lần.
///
/// `status` KHÔNG được khai ở đây — cùng lý do `insert_segments` không khai: `segment.status`
/// đã có `DEFAULT 'draft'` (`SEGMENT_STATUS_AND_VERSION_DDL`), và `'draft'` LÀ giá trị đúng
/// cho một hàng vừa nhập (§Always spec 6.16 — "status = 'draft'").
///
/// 🔵 **SỬA 2026-09-12 (Story 6.17, FR116) — `translation_origin` không còn HẰNG cho MỌI
/// hàng.** Trước story này, mọi segment tới đây LUÔN mang một bản dịch thật (nguồn/đích cặp
/// số câu bằng nhau) nên hằng `TRANSLATION_ORIGIN_BILINGUAL_IMPORT` đúng vô điều kiện. "Bỏ
/// qua hàng này" (§Always spec 6.17: "the source sentences import untranslated (empty target,
/// no `bilingual_import` origin)") giờ có thể đưa tới đây một segment với `target_text` RỖNG —
/// và [`crate::core::segment::bilingual::split_source_text`]/`apply_cuts` không bao giờ sinh
/// ra một mảnh rỗng cho đường CẶP ĐƯỢC (rỗng sau trim luôn bị từ chối ở đó), nên `target_text
/// rỗng` là tín hiệu PHÂN BIỆT ĐƯỢC, không đoán: đúng và chỉ đúng cho một segment "chưa dịch"
/// của lượt Bỏ qua. Origin đi theo tín hiệu đó — cùng hằng `TRANSLATION_ORIGIN_NONE` mà
/// `insert_segments` (đường văn xuôi) đã dùng cho đúng ý nghĩa "chưa có bản dịch nào để khai".
pub(crate) fn insert_bilingual_segments(
    tx: &Transaction<'_>,
    chapter_id: i64,
    segments: &[crate::core::segment::bilingual::BilingualSegment],
) -> SqlResult<()> {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, \
         is_target_paragraph_end, translation_origin, target_text, baseline_target_text, \
         baseline_translation_origin, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?6, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
    )?;
    for (index, segment) in segments.iter().enumerate() {
        let ord = i64::try_from(index).unwrap_or(i64::MAX).saturating_add(1);
        // AD-46 — cờ đích MIRROR cờ nguồn tại lúc nhập, cùng giá trị ghi vào CẢ hai cột
        // (đúng khuôn `insert_segments`, không một nguồn sự thật thứ hai).
        let paragraph_end = i64::from(segment.is_paragraph_end);
        let pair_origin =
            if segment.target_text.is_empty() { TRANSLATION_ORIGIN_NONE } else { TRANSLATION_ORIGIN_BILINGUAL_IMPORT };
        stmt.execute((
            chapter_id,
            ord,
            &segment.source_text,
            paragraph_end,
            paragraph_end,
            pair_origin,
            &segment.target_text,
        ))?;
    }
    Ok(())
}

/// Tách một Chương **đã có trên đĩa** thành các hàng `segment` — **hàm thuần, đây là thứ
/// test gọi**.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - `chapter_id` không có trong Tác phẩm đang mở ⇒ `segment.chapter_not_found`;
/// - Chương đã có segment ⇒ `segment.already_split` (**không** ghi đè);
/// - đường đọc/ghi trượt ⇒ lỗi kho (`store.*`), qua `From<StoreError>`.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 KHOÁ `OpenWorkState` Ở [`wire`] LÀ THỨ ĐANG CHẮN MỘT CUỘC ĐUA GHI TRÙNG
/// ─────────────────────────────────────────────────────────────────────────────
/// **Đừng nhả nó sớm để "tối ưu".** Code review 2026-08-12 dựng lại đường hỏng đầy đủ.
///
/// Phép kiểm *"Chương đã có segment chưa"* nằm ở lượt `store.read` ngay dưới đây, **tách
/// rời** lượt `store.write` ở cuối hàm. Giữa hai lượt đó, tầng kho **không** giữ gì cả:
/// `Store::read` mượn một kết nối `query_only` từ pool rồi trả lại ngay, và `Store::write`
/// mở một giao dịch **mới** sau đó. Thứ duy nhất tuần tự hoá hai lượt gọi đồng thời là
/// `MutexGuard` mà [`wire::split_chapter_into_segments`] giữ **xuyên suốt** lời gọi này.
///
/// Nhả khoá sớm (ví dụ clone một handle `Store` rồi thả guard trước khi tách) ⇒ hai lượt
/// `invoke` song song cùng đọc `count = 0`, cùng chạy bộ tách, cùng ghi ⇒ **segment nhân
/// đôi**, `ord` trùng từng cặp. `SEGMENT_DDL` **không** khai `UNIQUE(chapter_id, ord)` —
/// cố ý, vì Epic 2 cần để hở tạm khi sắp lại — nên không có lưới nào ở tầng SQL bắt được.
/// Và AD-4 đóng băng đống đó vĩnh viễn.
///
/// ⚠️ Cái giá của khoá này đã cân: nó giữ `OpenWorkState` suốt cả lượt đọc, phép tách CPU
/// và lượt ghi, nên `read_open_chapter` phải xếp hàng sau. Chương lớn nhất đo được là
/// **48.640** ký tự — phép tách trên đó nằm dưới ngưỡng nhìn thấy, nên đánh đổi này rẻ hơn
/// hẳn rủi ro ở trên. Nếu về sau có Chương đủ lớn để nó thành vấn đề, đường đúng **không**
/// phải nhả khoá sớm mà là **đưa phép kiểm `already_split` vào trong chính giao dịch ghi**.
pub fn split_chapter_into_segments(
    open: Option<&OpenWork>,
    chapter_id: i64,
) -> Result<SplitOutcome, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    // MOT luot doc lay ca hai thu can quyet dinh: van ban nguon, va so segment DA CO.
    let found: Option<(String, i64)> = open.store.read(move |conn| {
        let row = conn.query_row(
            "SELECT c.source_text, \
             (SELECT COUNT(*) FROM segment s WHERE s.chapter_id = c.id) \
             FROM chapter c WHERE c.id = ?1",
            [chapter_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );
        match row {
            Ok(value) => Ok(Some(value)),
            // ⚠️ `OptionalExtension::optional()` khong duoc `core::store` tai xuat, va them
            // mot tai xuat chi cho mot cho goi la mo rong be mat cua tang do. Ro nhanh
            // `QueryReturnedNoRows` bang tay — `SqlError` DA duoc tai xuat.
            Err(SqlError::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err),
        }
    })?;

    let (source_text, existing) = found.ok_or_else(|| chapter_not_found(chapter_id))?;
    if existing > 0 {
        return Err(already_split(chapter_id, existing));
    }

    // 🔴 Phep tach chay **NGOAI** closure ghi — Quyet dinh #3 cua Story 1.15, va AD-11:
    // mot writer duy nhat noi tiep, nen thoi gian CPU trong closure chan MOI luot ghi khac
    // cua tien trinh. Closure ghi chi mang SQL.
    //
    // 🔴 **Story 6.13** — Chương ĐI QUA LỆNH NÀY không bao giờ có `blocks`/ảnh (nó chỉ tồn tại
    // cho Chương CŨ, tách trước khi bảng `segment` ra đời — xem doc-comment đầu module): mọi
    // hàng vì thế mang `role = NULL`, đúng `WovenSegment::from` (không dệt gì).
    let segments: Vec<WovenSegment> = split_source_text(&source_text, &open.meta.source_lang)
        .into_iter()
        .map(WovenSegment::from)
        .collect();
    let segment_count = segments.len();

    open.store
        .write(move |tx: &Transaction<'_>| insert_segments(tx, chapter_id, &segments))?;

    Ok(SplitOutcome {
        chapter_id,
        segment_count,
    })
}
