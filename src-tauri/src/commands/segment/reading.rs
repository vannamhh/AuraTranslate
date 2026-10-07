use super::*;

/// Một câu, cho Chế độ đọc — Story 5.11, **thêm** `is_confirmed` ở Story 5.12.
///
/// ⚠️ **KHÔNG** `is_omitted`, `is_paragraph_end`, `status` NGUYÊN VĂN: câu đã cắt bỏ không
/// bao giờ đi tới đây (lọc ở [`paragraphs_in_translation`]), và cấu trúc đoạn đã được RÚT
/// GỌN thành hình dạng `ReadingParagraph` trước khi ra dây — webview không có gì để tự tính
/// lại. `is_confirmed` là NGOẠI LỆ có chủ ý duy nhất: nó là kết quả của một phép so `status
/// == SEGMENT_STATUS_CONFIRMED`, không phải chuỗi `status` thô — AC6 đòi trang đọc gạch chấm
/// nhẹ câu **chưa xác nhận** trong một Chương đã `Done` (đặt `Done` bằng tay không đồng
/// nghĩa với mọi câu đã ký), và webview không được tự đoán điều đó từ một trường vắng mặt.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadingSegment {
    pub id: i64,
    pub source_text: String,
    pub target_text: String,
    pub is_confirmed: bool,
    /// Marker bền vững thuộc Tác phẩm đang mở. Rust quyết bằng `reading_mark.segment_id`;
    /// webview chỉ render, không tự giữ một danh sách song song để đoán.
    pub is_marked: bool,
}

/// Một marker đã phân giải để hiển thị và điều hướng — Story 5.13, FR119.
///
/// `segment_id`/văn bản thuộc câu GỐC; `chapter_id` và thứ tự thuộc NEO SỐNG. Vì vậy một
/// lượt tổ chức lại Chương tự hiện đúng Chương mới mà không cần cập nhật hàng marker.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadingMark {
    pub segment_id: i64,
    pub navigation_segment_id: i64,
    pub chapter_id: i64,
    pub chapter_ord: i64,
    pub chapter_title: Option<String>,
    pub source_text: String,
    pub target_text: String,
    pub is_retired: bool,
    pub marked_at: String,
}

fn select_reading_marks(conn: ReadHandle<'_>) -> SqlResult<Vec<ReadingMark>> {
    let stored_count: i64 = conn.query_row("SELECT COUNT(*) FROM reading_mark", [], |row| row.get(0))?;
    let mut stmt = conn.prepare(
        "SELECT m.segment_id, m.navigation_segment_id, a.chapter_id, c.ord, c.title, \
                original.source_text, original.target_text, original.retired_at IS NOT NULL, \
                m.marked_at \
         FROM reading_mark m \
         JOIN segment original ON original.id = m.segment_id \
         JOIN segment a ON a.id = m.navigation_segment_id AND a.retired_at IS NULL \
         JOIN chapter c ON c.id = a.chapter_id \
         ORDER BY c.ord, a.ord, m.segment_id",
    )?;
    let marks = stmt.query_map([], |row| {
        Ok(ReadingMark {
            segment_id: row.get(0)?,
            navigation_segment_id: row.get(1)?,
            chapter_id: row.get(2)?,
            chapter_ord: row.get(3)?,
            chapter_title: row.get(4)?,
            source_text: row.get(5)?,
            target_text: row.get(6)?,
            is_retired: row.get(7)?,
            marked_at: row.get(8)?,
        })
    })?
    .collect::<SqlResult<Vec<_>>>()?;

    // `reading_mark` cố ý không có FK vì kho tắt `PRAGMA foreign_keys`. Do đó INNER JOIN
    // một mình nó có thể biến một neo thiếu/retired thành danh sách ngắn hơn mà không lỗi —
    // đúng lớp "rỗng im lặng" đã hụt ba lần. Đếm trong cùng snapshot khiến bất biến hỏng
    // đi ra `store.read_failed`, không được ngụy trang thành "không có marker".
    if i64::try_from(marks.len()).unwrap_or(i64::MAX) != stored_count {
        return Err(SqlError::InvalidQuery);
    }
    Ok(marks)
}

/// Đánh dấu một câu sống. `INSERT OR IGNORE` biến lần bấm `M` thứ hai thành đúng thao tác
/// idempotent, không thành toggle/gỡ. Phép kiểm câu sống và lượt chèn ở cùng transaction.
pub fn mark_reading_segment(
    open: Option<&OpenWork>,
    segment_id: i64,
) -> Result<ReadingMark, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let missing = Arc::new(Mutex::new(false));
    let missing_in = Arc::clone(&missing);
    let result = open.store.write(move |tx: &Transaction<'_>| {
        let live: i64 = tx.query_row(
            "SELECT COUNT(*) FROM segment WHERE id = ?1 AND retired_at IS NULL",
            [segment_id],
            |row| row.get(0),
        )?;
        if live == 0 {
            *missing_in.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = true;
            return Err(SqlError::QueryReturnedNoRows);
        }
        tx.execute(
            "INSERT OR IGNORE INTO reading_mark (segment_id, navigation_segment_id, marked_at) \
             VALUES (?1, ?1, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            [segment_id],
        )?;
        select_reading_marks(tx)?.into_iter().find(|m| m.segment_id == segment_id).ok_or(
            SqlError::QueryReturnedNoRows,
        )
    });
    match result {
        Ok(mark) => Ok(mark),
        Err(_err) if *missing.lock().unwrap_or_else(std::sync::PoisonError::into_inner) => {
            Err(segment_not_found(segment_id))
        }
        Err(err) => Err(err.into()),
    }
}

/// Liệt kê marker của đúng Tác phẩm đang mở. `project.db` là biên cách ly Work; không nhận
/// `work_id` từ webview nên không có đường trộn dữ liệu giữa hai kho.
pub fn list_reading_marks(open: Option<&OpenWork>) -> Result<Vec<ReadingMark>, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    open.store.read(select_reading_marks).map_err(Into::into)
}

/// Một đoạn — nhóm các [`ReadingSegment`] liên tiếp thuộc CÙNG một đoạn của bản dịch,
/// đã qua [`paragraphs_in_translation`]. Không đoạn nào rỗng (xem doc-comment hàm đó).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadingParagraph {
    pub segments: Vec<ReadingSegment>,
}

/// Một Chương trong dãy đọc, đã gom đoạn — thứ [`read_reading_run`] trả ra dây, một phần tử
/// của [`ReadingRun::chapters`].
///
/// `chapter_ord`/`chapter_title` đi kèm để trang đọc dựng được tiêu đề Chương mà không cần
/// một lệnh IPC thứ hai — cùng lý lẽ `chapter_id` đã có sẵn trên [`ChapterSegments`].
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadingChapter {
    pub chapter_id: i64,
    pub chapter_ord: i64,
    /// `NULL` ⇒ Chương chưa đặt tên — cùng ngữ nghĩa `ChapterRow::title` (Story 5.7).
    pub chapter_title: Option<String>,
    pub paragraphs: Vec<ReadingParagraph>,
    /// **THÊM Story 5.12.** Số hàng `segment` CÒN SỐNG của Chương này, đếm TRONG CÙNG lượt
    /// đọc — KHÔNG một `COUNT(*)` thứ hai, KHÔNG một lệnh IPC phụ (`listChapters()`) như
    /// `readingState.ts` (Story 5.11) từng làm để phân biệt "Chương rỗng" với "mọi câu đã
    /// cắt bỏ". Dữ kiện này đã nằm sẵn trong dãy `ChapterSegment` TRƯỚC khi lọc cắt bỏ —
    /// đưa nó lên dây tốn một `i64` mỗi Chương và xoá nguyên một nhánh trạng thái
    /// (`'empty-unknown'`) ở webview cùng lý do nó ra đời đã biến mất (§Design Notes của
    /// story: "một nhánh biến mất vì NGUYÊN NHÂN của nó biến mất").
    pub segment_count: i64,
    /// **THÊM Story 6.14 (FR42/FR43)** — ảnh của Chương này, phân giải neo với quy tắc "đủ
    /// điều kiện làm neo" của CHẾ ĐỘ ĐỌC (`include_omitted = false, exclude_roles = true` —
    /// chỉ segment CÒN trong bản dịch VÀ không mang vai mới thật sự lên trang). `after_segment_id`
    /// vì thế luôn khớp `id` của một [`ReadingSegment`] có mặt trong [`Self::paragraphs`]
    /// (hoặc `None` ⇒ trước đoạn đầu tiên) — webview không cần tra cứu gì thêm để biết chèn
    /// ảnh vào đâu.
    pub images: Vec<ReadingImage>,
}

/// Một ảnh đã phân giải vị trí, ra dây cho CHẾ ĐỘ ĐỌC — Story 6.14, FR42/FR43.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ReadingImage {
    pub asset_id: i64,
    /// Tên tệp TƯƠNG ĐỐI trong `assets/` — ghép với [`ReadingRun::assets_dir`] ở webview.
    pub file_name: String,
    pub source_url: Option<String>,
    /// `id` của một [`ReadingSegment`] ĐÃ hiện trên trang (không cắt bỏ, không mang vai) —
    /// `None` ⇒ ảnh đứng TRƯỚC đoạn đầu tiên của Chương.
    pub after_segment_id: Option<i64>,
    /// `target_text` của segment `role = 'alt'` — vào thuộc tính `alt` của `<img>`, KHÔNG BAO
    /// GIỜ hiện thành văn bản trên trang (Design Notes spec 6.14). `None` ⇒ `alt=""`.
    pub alt_text: Option<String>,
    /// `target_text` của segment `role = 'caption'` — vào `<figcaption>`. `None` hoặc chuỗi
    /// RỖNG (chưa dịch) đều nghĩa là "không dựng `<figcaption>` nào" (không chỗ trống).
    pub caption_text: Option<String>,
}

impl From<crate::core::segment::image::ResolvedImage> for ReadingImage {
    fn from(r: crate::core::segment::image::ResolvedImage) -> Self {
        ReadingImage {
            asset_id: r.asset_id,
            file_name: r.file_name,
            source_url: r.source_url,
            after_segment_id: r.after_segment_id,
            alt_text: r.alt_text,
            caption_text: r.caption_text,
        }
    }
}

/// Vì sao dãy đọc DỪNG ở nơi nó dừng — **phân biệt được**, không một `Option` trần (cùng lý
/// lẽ [`crate::commands::chapter::ChapterSwitchOutcome`]: *"Rỗng IM LẶNG bị cấm; rỗng CÓ LÝ
/// DO thì không"*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ReadingFrontierKind {
    /// Chương ngay sau dãy vừa đọc **chưa** `Done` ⇒ trường `chapter` của [`ReadingFrontier`]
    /// nêu đích danh nó.
    #[serde(rename = "next-not-done")]
    NextNotDone,
    /// Mọi Chương từ Chương đang mở tới hết Tác phẩm đều `Done` — không Chương nào chặn.
    #[serde(rename = "end-of-work")]
    EndOfWork,
}

/// Chương đứng CHẶN dãy đọc — trường `chapter` của [`ReadingFrontier`] khi
/// `kind == NextNotDone`.
///
/// 🔴 `status` là CHUỖI THÔ trên cột `chapter.status` (chuỗi tự do ở tầng SQL, không
/// `CHECK`), KHÔNG phải một `LifecycleStatus` đã phân giải — mốc biên phải chở được đúng
/// nguyên văn của một giá trị **lạ** (§I/O Matrix "Trạng thái lạ": dãy dừng trước nó, và
/// `frontier.chapter.status` mang nguyên văn giá trị đó). Webview tra nhãn hiển thị qua bảng
/// `lifecycle.*`; không khớp thì hiện chuỗi thô — đó là ca "giá trị lạ" đọc lên như một điều
/// bất thường, không rơi vào một nhãn nào đó (§Design Notes của story).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadingFrontierChapter {
    pub chapter_id: i64,
    pub chapter_ord: i64,
    pub chapter_title: Option<String>,
    pub status: String,
}

/// Mốc biên giới của một lượt đọc — thứ nói VÌ SAO dãy `ReadingRun::chapters` dừng ở đó,
/// khuôn trực tiếp [`crate::commands::chapter::ChapterSwitch`].
///
/// 🔴 `chapter` là `Some` **khi và chỉ khi** `kind == ReadingFrontierKind::NextNotDone` —
/// cùng mệnh đề đã ký ở `ChapterSwitch::chapter`, viết ra thành chữ vì một `Option` có điều
/// kiện mà không phát biểu điều kiện là một hợp đồng ngầm.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadingFrontier {
    pub kind: ReadingFrontierKind,
    pub chapter: Option<ReadingFrontierChapter>,
}

/// Một lượt đọc trọn vẹn — thứ [`read_reading_run`] trả ra dây. Story 5.12.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReadingRun {
    pub chapters: Vec<ReadingChapter>,
    pub frontier: ReadingFrontier,
    /// **THÊM Story 6.14** — đường dẫn TUYỆT ĐỐI tới `<dir>/assets` của Tác phẩm đang mở, MỘT
    /// lần cho CẢ LƯỢT ĐỌC (không lặp lại ở từng `ReadingChapter`/`ReadingImage`) — cùng lý lẽ
    /// [`ChapterSegments::assets_dir`].
    pub assets_dir: String,
}

/// **Đọc một LƯỢT ĐỌC bắt đầu tại Chương đang mở** — hàm thuần, đây là thứ test gọi.
/// Story 5.12, FR120.
///
/// Đọc danh sách Chương, đếm `segment_count` VÀ mọi `ChapterSegment` của mọi Chương trong
/// dãy trong **MỘT** [`crate::core::store::Store::read`] duy nhất — cùng lý do §Story 2.11
/// đã ghi cho `caret_segment_id` ở [`read_open_chapter_segments`]: hai lượt đọc rời nhau là
/// hai ảnh chụp có thể lệch nhau nếu một lượt ghi (đổi trạng thái, gộp/tách Chương) chen
/// giữa.
///
/// # Thuật toán
/// ① `SELECT id, ord, title, status` mọi Chương, `ORDER BY ord, id` (cùng bộ đôi so sánh mà
/// `commands::chapter` dùng cho mọi lượt đọc theo thứ tự — `ord` cố ý KHÔNG `UNIQUE`).
/// ② Tìm vị trí `open.chapter_id` trong danh sách đó; vắng mặt (hàng Chương đã biến mất
/// giữa lúc mở Chương và lúc vào Chế độ đọc) ⇒ `Err(chapter_not_found(...))`.
/// ③ Từ vị trí đó đi TỚI, lấy TIỀN TỐ liên tiếp mà
/// `LifecycleStatus::from_wire(&status) == Some(Done)` — một giá trị lạ (ngoài bốn giá trị)
/// đọc thành KHÔNG `Done`, dãy dừng NGAY tại đó, không đoán, không bỏ qua im lặng (§Always).
/// ④ Với mỗi Chương của tiền tố, gọi [`select_chapter_segments`] rồi
/// [`crate::core::segment::reading::paragraphs_in_translation`] — `segment_count` đếm từ
/// CHÍNH dãy vừa đọc, không một `COUNT(*)` thứ hai.
/// ⑤ Dựng `frontier`: hàng ngay sau tiền tố ⇒ `NextNotDone` kèm dữ liệu hàng đó (kể cả khi
/// tiền tố RỖNG — hàng đó khi ấy chính là Chương đang mở); hết bảng ⇒ `EndOfWork` + `None`.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `work.none_open`;
/// - `OpenWork::chapter_id` không còn trong bảng `chapter` ⇒ `segment.chapter_not_found`;
/// - đường đọc trượt (kho hỏng) ⇒ `store.read_failed` (qua `From<StoreError>`).
pub fn read_reading_run(open: Option<&OpenWork>) -> Result<ReadingRun, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let current_chapter_id = open.chapter_id;
    // **THÊM Story 6.14** — cùng lý do `read_open_chapter_segments`: cấu tạo NGOÀI closure,
    // `move` mang giá trị đã sẵn sàng vào trong. Same UTF-8 check as there, before the closure.
    let assets_dir = open.dir.join("assets");
    let assets_dir_str = assets_dir.to_str().ok_or_else(assets_dir_not_utf8)?.to_owned();

    let found = open.store.read(move |conn| {
        let mut stmt = conn.prepare("SELECT id, ord, title, status FROM chapter ORDER BY ord, id")?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?
            .collect::<SqlResult<Vec<(i64, i64, Option<String>, String)>>>()?;

        let Some(start) = rows.iter().position(|(id, ..)| *id == current_chapter_id) else {
            // Hàng Chương đang mở đã biến mất khỏi `project.db` giữa lúc mở Chương và lúc
            // vào Chế độ đọc — tầng này báo `None`, tầng trên đổi nó thành lỗi CÓ TÊN.
            return Ok(None);
        };

        // Tiền tố các Chương `Done` LIÊN TIẾP, bắt đầu TẠI Chương đang mở — một giá trị
        // `status` lạ (không nằm trong bốn giá trị của `LifecycleStatus`) đọc thành KHÔNG
        // `Done`, dãy dừng ngay tại đó (§I/O Matrix "Trạng thái lạ").
        let mut prefix_end = start;
        while prefix_end < rows.len() {
            let is_done = LifecycleStatus::from_wire(&rows[prefix_end].3) == Some(LifecycleStatus::Done);
            if !is_done {
                break;
            }
            prefix_end += 1;
        }

        // Cùng snapshot với nội dung. Không một lượt IPC/Store::read phụ có thể chen một
        // thao tác `M` vào giữa rồi cho trang hiện chữ và marker từ hai thời điểm khác nhau.
        let marked_ids: BTreeSet<i64> = {
            // Story 5.13 yêu cầu trạng thái marker đi cùng chính snapshot đọc nội dung. LEFT JOIN
            // bắt đầu từ `segment` giữ hình dạng đó minh bạch: segment không marker vẫn vào run,
            // còn `m.segment_id IS NOT NULL` chỉ tạo tập membership — không có một IPC phụ có thể
            // chen một lượt ghi giữa nội dung và trạng thái.
            let mut stmt = conn.prepare(
                "SELECT s.id FROM segment s \
                 LEFT JOIN reading_mark m ON m.segment_id = s.id \
                 WHERE m.segment_id IS NOT NULL",
            )?;
            stmt.query_map([], |row| row.get(0))?.collect::<SqlResult<BTreeSet<i64>>>()?
        };

        let mut chapters = Vec::with_capacity(prefix_end - start);
        for (chapter_id, chapter_ord, title, _status) in &rows[start..prefix_end] {
            let segments = select_chapter_segments(conn, *chapter_id)?;
            let segment_count = segments.len() as i64;
            // **THÊM Story 6.14** — segment mang VAI (`alt`/`caption`) bị loại khỏi dòng văn
            // xuôi qua một chốt lọc RIÊNG (`image::strip_role_segments`), chạy SAU
            // `paragraphs_in_translation` chứ không thay nó: ranh giới đoạn đã đúng vị trí từ
            // lượt gom đó (kể cả khi ranh giới nằm trên chính một segment vai), nên đây chỉ
            // bớt PHẦN TỬ chứ không tính lại gì. Xem doc-comment của hàm đó.
            let paragraphs = crate::core::segment::image::strip_role_segments(
                crate::core::segment::reading::paragraphs_in_translation(&segments),
            )
            .into_iter()
            .map(|group| ReadingParagraph {
                segments: group
                    .into_iter()
                    .map(|s| ReadingSegment {
                        id: s.id,
                        source_text: s.source_text.clone(),
                        target_text: s.target_text.clone(),
                        is_confirmed: s.status == SEGMENT_STATUS_CONFIRMED,
                        is_marked: marked_ids.contains(&s.id),
                    })
                    .collect(),
            })
            .collect();

            // **THÊM Story 6.14** — ảnh của Chương này, quy tắc "đủ điều kiện làm neo" của
            // CHẾ ĐỘ ĐỌC: chỉ segment CÒN trong bản dịch VÀ không mang vai (đúng tập vừa lên
            // trang ở trên) mới đủ điều kiện — xem doc-comment `image::resolve_chapter_images`.
            let raw_assets = select_chapter_assets(conn, *chapter_id)?;
            let images: Vec<ReadingImage> = crate::core::segment::image::resolve_chapter_images(
                &segments,
                &raw_assets,
                false, // include_omitted -- cau da cat bo khong len trang doc.
                true,  // exclude_roles -- segment vai cung khong len trang doc.
            )
            .into_iter()
            .map(ReadingImage::from)
            .collect();

            chapters.push(ReadingChapter {
                chapter_id: *chapter_id,
                chapter_ord: *chapter_ord,
                chapter_title: title.clone(),
                paragraphs,
                segment_count,
                images,
            });
        }

        let frontier = if prefix_end < rows.len() {
            let (chapter_id, chapter_ord, title, status) = &rows[prefix_end];
            ReadingFrontier {
                kind: ReadingFrontierKind::NextNotDone,
                chapter: Some(ReadingFrontierChapter {
                    chapter_id: *chapter_id,
                    chapter_ord: *chapter_ord,
                    chapter_title: title.clone(),
                    status: status.clone(),
                }),
            }
        } else {
            ReadingFrontier { kind: ReadingFrontierKind::EndOfWork, chapter: None }
        };

        Ok(Some(ReadingRun {
            chapters,
            frontier,
            assets_dir: assets_dir_str,
        }))
    })?;

    found.ok_or_else(|| chapter_not_found(current_chapter_id))
}

/// **THÊM Story 5.7 (AC4/AC6).** Ghi vị trí caret của một Chương xuống `chapter_position` —
/// hàm thuần, đây là thứ test gọi. `INSERT … ON CONFLICT(chapter_id) DO UPDATE`: một hàng
/// DUY NHẤT mỗi Chương (`chapter_id` là khoá chính) — mở lại rồi rê caret lần nữa GHI ĐÈ,
/// không cộng dồn lịch sử vị trí.
///
/// 🔴 **Nhịp ghi này KHÔNG mang bảo đảm AD-35** (xem doc-comment của
/// `src/panels/positionFlush.ts`): mất MỘT lượt ghi vị trí là mất MỘT LỜI NHẮC, không mất
/// công việc — khác hẳn `save_segment_targets`, nơi mất một lượt là mất bản dịch người dùng
/// đã gõ. Vì vậy hàm này không cần trần cứng chống mất chữ; lỗi ghi chỉ cần một chẩn đoán ở
/// TẦNG GỌI, không một hộp thoại chặn người dùng (§I/O Matrix "Ghi vị trí": *"Lỗi ghi ⇒ chẩn
/// đoán, KHÔNG hộp thoại"*).
///
/// `save_segment_targets::UPDATE` (dòng ~1216) **không đổi một dòng** vì lượt này — bảng mới,
/// không chạm bảng `segment`.
///
/// # Lỗi
/// - `chapter_id` không thuộc `project.db` đang mở ⇒ `segment.chapter_not_found` (tái dùng
///   khoá đã có) — KHÔNG hàng nào được ghi.
/// - `segment_id` không thuộc ĐÚNG `chapter_id` được chỉ (Chương tồn tại, segment tồn tại,
///   nhưng là một CẶP LỆCH) ⇒ `segment.not_found` — KHÔNG hàng nào được ghi.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔵 THÊM PHÉP KIỂM CẶP (2026-08-29, Story 5.8) — đóng mục `deferred` #1 của Story 5.7
/// ─────────────────────────────────────────────────────────────────────────────
/// Trước lượt này hàm chỉ kiểm `chapter_id` có tồn tại, KHÔNG kiểm `segment_id` có THẬT SỰ
/// thuộc Chương đó. Story 5.8 là story ĐẦU TIÊN sinh ra được một cặp lệch thật: lượt TÁCH
/// đổi `segment.chapter_id` của một số câu sang một Chương mới, nên một vị trí đã ghi TRƯỚC
/// lượt tách (`(A, câu 3)`) có thể trỏ vào một segment nay đã thuộc Chương KHÁC. Không kiểm
/// cặp thì `INSERT ... ON CONFLICT` vẫn ghi thành công một hàng `chapter_position` chỉ vào
/// một segment không thuộc Chương đó — rỗng im lặng ở dạng tệ nhất: lần mở Chương kế tiếp,
/// `read_open_chapter_segments` rơi vào nhánh "vị trí trỏ vào segment đã VỀ HƯU" (dòng ~905)
/// dù nguyên nhân thật là một CẶP LỆCH, một chẩn đoán đúng chữ nhưng sai nguyên nhân.
///
/// Khuôn kiểm giống hệt `flush_segment_targets` bước ② (`:2088`): `SELECT COUNT(*) FROM
/// segment WHERE id = ?1 AND chapter_id = ?2` — 0 hàng nghĩa là segment không tồn tại HOẶC
/// tồn tại ở một Chương khác; cả hai đều là "cặp lệch" từ góc nhìn của lời gọi này.
pub fn save_chapter_position(
    open: Option<&OpenWork>,
    chapter_id: i64,
    segment_id: i64,
) -> Result<(), IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    // 🔴 Cùng ô CÓ KIỂU cho lý do từ chối, cùng lý lẽ định lượng đã ghi ở
    // `save_segment_targets` ngay trên: `Store::write` gói MỌI `Err` (kể cả một từ chối
    // nghiệp vụ) thành `StoreError::WriteFailed { detail: String }`, và đoán lại lý do từ
    // `detail` bằng chuỗi là một chẩn đoán SAI cho ba ca khác hẳn nhau (chapter lạ / cặp lệch /
    // kho hỏng thật).
    #[derive(Clone, Copy)]
    enum PositionReject {
        ChapterMissing,
        SegmentMismatch,
    }
    let reject: Arc<Mutex<Option<PositionReject>>> = Arc::new(Mutex::new(None));
    let reject_in = Arc::clone(&reject);

    let result = open.store.write(move |tx: &Transaction<'_>| {
        // TRONG cùng giao dịch với lượt ghi — không khe hở nào giữa phép kiểm và phép ghi,
        // cùng khuôn `save_segment_targets` bước ①.
        let chapter_rows: i64 = tx.query_row(
            "SELECT COUNT(*) FROM chapter WHERE id = ?1",
            [chapter_id],
            |row| row.get(0),
        )?;
        if chapter_rows == 0 {
            *reject_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(PositionReject::ChapterMissing);
            return Err(SqlError::QueryReturnedNoRows);
        }

        // 🔵 THÊM (Story 5.8) — phép kiểm CẶP, khuôn có sẵn ở `flush_segment_targets:2088`.
        let segment_rows: i64 = tx.query_row(
            "SELECT COUNT(*) FROM segment WHERE id = ?1 AND chapter_id = ?2",
            (segment_id, chapter_id),
            |row| row.get(0),
        )?;
        if segment_rows == 0 {
            *reject_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(PositionReject::SegmentMismatch);
            return Err(SqlError::QueryReturnedNoRows);
        }

        tx.execute(
            "INSERT INTO chapter_position (chapter_id, segment_id, updated_at) \
             VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ','now')) \
             ON CONFLICT(chapter_id) DO UPDATE SET \
             segment_id = excluded.segment_id, updated_at = excluded.updated_at",
            (chapter_id, segment_id),
        )?;
        Ok(())
    });

    match result {
        Ok(()) => Ok(()),
        Err(err) => {
            let taken = *reject.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            match taken {
                Some(PositionReject::ChapterMissing) => Err(chapter_not_found(chapter_id)),
                Some(PositionReject::SegmentMismatch) => Err(segment_not_found(segment_id)),
                None => Err(err.into()),
            }
        }
    }
}

/// **Đặt cờ kết đoạn của BẢN DỊCH cho một câu** — hàm thuần, đây là thứ test gọi.
/// Story 2.5d · FR134 · AD-46 · AC2 · AC4 · Quyết định #3 đường (c) (Ice ký 2026-08-15).
///
/// `ends_paragraph = true` là *"sau câu này, bản dịch xuống đoạn mới"*; `false` là bỏ.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 GHI **RỜI RẠC**, KHÔNG QUA BỘ ĐỆM GÕ — và đây là một luật, không một lựa chọn
/// ─────────────────────────────────────────────────────────────────────────────
/// `project-context.md:520-522`: *"Thao tác RỜI RẠC ghi NGAY, không qua bộ đệm gõ. Định
/// tuyến chúng qua bộ đệm khiến một thao tác người dùng **thấy đã xong** nằm chờ tới 5 giây
/// rồi biến mất nếu app sập"*. Đổi ranh giới đoạn là một thao tác như vậy: người dùng bấm,
/// thấy hàng đổi hình, và coi như xong.
/// ⇒ Một `open.store.write`, một giao dịch, ngay lập tức. **Không** đi qua
/// [`save_segment_targets`] và **không** đi qua lịch flush của AD-35.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 KHÔNG ĐỤNG `updated_at`, VÀ KHÔNG ĐỤNG `is_paragraph_end`
/// ─────────────────────────────────────────────────────────────────────────────
/// `updated_at` — cùng lý do đã ghi cho [`set_segment_omitted`]: nó là mốc của **văn bản**,
/// và cổng AC8 (`a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`) đứng
/// trên mệnh đề đó.
/// `is_paragraph_end` — **AD-37 vẫn sở hữu cờ nguồn**, và AD-46 khai bằng chữ *"AD-37 không
/// sửa một chữ"*. Một lượt đổi cờ đích chạm sang cờ nguồn là đổi **cấu trúc của bản gốc**,
/// thứ mà AD-4 nói tính một lần lúc nhập và không bao giờ tính lại.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - `segment_id` không có trong Tác phẩm đang mở ⇒ `segment.not_found`;
/// - segment đã về hưu (AD-5) ⇒ `segment.retired`;
/// - segment là câu **cuối Chương** và lượt gọi xin **BẬT** cờ ⇒ `segment.ends_chapter`.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (code review) — mệnh đề cũ ở đây đã HẾT ĐÚNG, sửa tại chỗ.**
/// Bản đầu viết *"**Không** khoá `err.segment.*` mới: hai nhánh từ chối ở đây là **đúng
/// hai** nhánh mà `set_segment_omitted` đã có… Dựng một khoá thứ ba nói cùng một chuyện là
/// hai cách gọi tên cho một sự thật"*. Vế **lý lẽ** vẫn đúng và vẫn được giữ; vế **đếm**
/// thì sai, vì lượt rà tìm ra một nhánh thứ ba mà bản đầu bỏ sót: ca ① của AD-37.
/// ⇒ `SegmentEndsChapter` **không** phải "một cách gọi tên thứ hai": nó nói câu **tồn
/// tại**, **còn sống**, và vẫn không mang cờ được — một sự thật mà `not_found` lẫn
/// `retired` đều **không** diễn đạt nổi. Ba nhánh, ba sự thật, ba khoá. Ice ký đường (a).
pub fn set_segment_paragraph_end(
    open: Option<&OpenWork>,
    segment_id: i64,
    ends_paragraph: bool,
) -> Result<ParagraphEndOutcome, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    let reject: Arc<Mutex<Option<OmitReject>>> = Arc::new(Mutex::new(None));
    let reject_in = Arc::clone(&reject);

    let outcome = open.store.write(move |tx: &Transaction<'_>| {
        let set_reject = |r: OmitReject| {
            *reject_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(r);
        };

        // ① Doc trang thai hien tai — TRONG cung giao dich voi luot ghi.
        let found = tx.query_row(
            "SELECT is_target_paragraph_end, retired_at, chapter_id, ord \
             FROM segment WHERE id = ?1",
            [segment_id],
            |row| {
                let flag: i64 = row.get(0)?;
                let retired_at: Option<String> = row.get(1)?;
                let chapter_id: i64 = row.get(2)?;
                let ord: i64 = row.get(3)?;
                Ok((flag != 0, retired_at, chapter_id, ord))
            },
        );

        let (current, retired_at, chapter_id, ord) = match found {
            Ok(value) => value,
            Err(SqlError::QueryReturnedNoRows) => {
                set_reject(OmitReject::NotFound);
                return Err(SqlError::QueryReturnedNoRows);
            }
            Err(err) => return Err(err),
        };

        if retired_at.is_some() {
            set_reject(OmitReject::Retired);
            return Err(SqlError::QueryReturnedNoRows);
        }

        // ② CA ① CUA AD-37 — segment CUOI Chuong khong mang co ket doan, LUON LUON.
        //
        // 🔴 Code review 2026-08-16, Ice ky duong (a). Truoc luot va nay hang rao chi song
        // trong `split::mark_paragraph_end` (duong NHAP) va trong ham thuan
        // `paragraph::at_end_of_chapter` — nen mot lenh `Mod+Alt+P` tren cau cuoi Chuong
        // BAT duoc co, va luoi ve mot ranh gioi doan duoi cau cuoi cung (`tgt-para-end`).
        // AC3 doi ba ca bien ap **y nguyen** cho co dich; day la ca thu nhat, va no phai
        // duoc cuong che o CHO GHI, khong chi o duong nhap.
        //
        // ⚠️ **Chi chan chieu BAT.** Mot lenh BO co tren cau cuoi Chuong di tiep: neu dia
        // dang mang `1` o do (du lieu tu truoc luot va nay, hoac mot lan sua bang SQL) thi
        // day la duong DUY NHAT sua no ve dung. Tu choi ca hai chieu la khoa cung mot hang
        // sai vinh vien.
        //
        // ⚠️ Va no dung TRUOC nhanh no-op ben duoi, co chu y: neu dia da mang `1` va nguoi
        // dung lai xin `1`, nhanh no-op tra `Ok` — tuc mot hang SAI di qua ma khong ai keu.
        if ends_paragraph {
            let has_successor: i64 = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM segment \
                 WHERE chapter_id = ?1 AND ord > ?2 AND retired_at IS NULL)",
                (chapter_id, ord),
                |row| row.get(0),
            )?;
            if has_successor == 0 {
                set_reject(OmitReject::EndsChapter);
                return Err(SqlError::QueryReturnedNoRows);
            }
        }

        // ③ DA o dung gia tri ⇒ khong ghi mot byte nao, cung luat voi `set_segment_omitted`.
        if current == ends_paragraph {
            return Ok(());
        }

        // ④ DUNG MOT cot.
        tx.execute(
            "UPDATE segment SET is_target_paragraph_end = ?1 WHERE id = ?2",
            (i64::from(ends_paragraph), segment_id),
        )?;

        Ok(())
    });

    match outcome {
        Ok(()) => Ok(ParagraphEndOutcome {
            segment_id,
            is_target_paragraph_end: ends_paragraph,
        }),
        Err(err) => {
            let taken = reject
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            match taken {
                Some(OmitReject::NotFound) => Err(segment_not_found(segment_id)),
                Some(OmitReject::Retired) => Err(segment_retired(segment_id)),
                Some(OmitReject::EndsChapter) => Err(segment_ends_chapter(segment_id)),
                // O rong ⇒ day la mot loi KHO that, khong mot phep tu choi nghiep vu.
                None => Err(err.into()),
            }
        }
    }
}
