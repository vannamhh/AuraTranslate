use super::*;

/// Kết quả một lượt **gộp** hoặc **tách** segment — thứ đi ra qua dây. Story 2.8, AC1 · AC2.
///
/// 🔴 **Hàng mới đi ra ĐẦY ĐỦ, và đó là chữ ký #4(a) của Ice** *(2026-08-17)*, không một
/// lượt trả về hào phóng. AD-47 ① đòi mỗi lượt ghi không-phải-người-dùng đặt lại **mốc so
/// sánh** của segment về đúng văn bản vừa ghi — và mốc **không sống trên đĩa**: Quyết định
/// #2(b) của Story 2.7 đặt nó ở webview, trong mảng `segments` (`editorPanelState.ts:34`).
/// Segment mới mang một `id` **chưa từng có** trong mảng đó, nên cách duy nhất đặt được mốc
/// mà không dựng một nguồn sự thật thứ hai là: lệnh trả hàng đầy đủ, webview chèn vào mảng.
///
/// 🔴 **`retired` mang HÀNG ĐẦY ĐỦ, không một danh sách `id`** — và lý do là một luật, không
/// một lượt hào phóng thứ hai. Chữ ký #6(b) giữ hàng về hưu **ở lại trong lưới** với vạch
/// `ornament`, và vị từ vẽ vạch đó là `retiredAt !== null` (`editorSegments.ts:162`). Với một
/// danh sách `id` trần, webview chỉ còn cách **bịa** một mốc thời gian để vạch hiện lên — một
/// giá trị dựng ở frontend cho một cột của đĩa, đúng thứ AD-1 cấm.
///
/// ⚠️ Và nó cũng không suy được theo chiều kia: *"những id không có trong `new_segments`"*
/// đúng cho gộp và **sai cho tách** *(một id cũ, hai id mới)*.
///
/// ⚠️ **Không** `#[serde(rename_all = ...)]` — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RegroupOutcome {
    /// Những segment vừa **về hưu** (AD-5), ở hình dạng dây. Gộp: hai hàng. Tách: một hàng.
    pub retired: Vec<ChapterSegment>,
    /// Những hàng vừa **tạo mới**, đã ở hình dạng dây. Gộp: một. Tách: hai.
    pub new_segments: Vec<ChapterSegment>,
}

/// Lý do một lượt gộp/tách bị **từ chối**, mang ra khỏi closure ghi.
///
/// ⚠️ Cùng khuôn và cùng lý do định lượng với [`OmitReject`]: `Store::write` gói mọi `Err`
/// thành `StoreError::WriteFailed { detail: String }`, nên đoán lại lý do từ chuỗi `detail`
/// là một chẩn đoán SAI.
enum RegroupReject {
    /// Không có hàng `segment` nào mang id đó.
    NotFound(i64),
    /// `retired_at` khác `NULL` (AD-5) — đường **ghi** từ chối.
    Retired(i64),
    /// Gộp mà segment đang chọn là câu **ĐẦU** Chương: không có câu nào liền trên nó.
    NoPreviousSegment(i64),
    /// Chỗ cắt để lại một mảnh **rỗng**, hoặc nằm ngoài `source_text`.
    CutLeavesEmptyPiece(i64),
    /// A segment involved carries a `role`; regrouping would drop it.
    RoleSegment(i64),
    /// A source segment's `baseline_translation_origin` is outside the closed set.
    UnknownOrigin(i64, String),
}

impl RegroupReject {
    fn into_ipc(self) -> IpcError {
        match self {
            Self::NotFound(id) => segment_not_found(id),
            Self::Retired(id) => segment_retired(id),
            Self::NoPreviousSegment(id) => segment_has_no_previous(id),
            Self::CutLeavesEmptyPiece(id) => segment_cut_leaves_empty_piece(id),
            Self::RoleSegment(id) => segment_has_role(id),
            Self::UnknownOrigin(id, value) => segment_unknown_translation_origin(id, &value),
        }
    }
}

fn segment_has_role(segment_id: i64) -> IpcError {
    IpcError::new(
        "segment.has_role",
        MessageKey::SegmentHasRole,
        BTreeMap::from([("segment_id".to_owned(), segment_id.to_string())]),
        false,
    )
}

/// Không có segment nào đứng **liền trên** segment này trong Chương — Story 2.8, AC1.
///
/// ⚠️ Một khoá **mới**, và nó không nói lại điều gì đã có: ba khoá `err.segment.*` hiện có
/// nói về *"không tìm thấy"*, *"đã về hưu"* và *"không có gì để xác nhận"*. *"Câu đầu Chương
/// nên không có câu nào để gộp lên"* là một lý do từ chối **thứ tư**, phân biệt được, và
/// người dùng phải đọc ra được nó thay vì một câu chung chung.
fn segment_has_no_previous(segment_id: i64) -> IpcError {
    IpcError::new(
        "segment.no_previous",
        MessageKey::SegmentNoPrevious,
        BTreeMap::from([("segment_id".to_owned(), segment_id.to_string())]),
        false,
    )
}

/// Chỗ cắt để lại một mảnh rỗng — Story 2.8, AC2.
///
/// 🔴 `retryable = false`: cùng một chỗ cắt sẽ cho cùng một kết quả mãi mãi. Một lượt thử
/// lại tự động ở đây là một vòng lặp im lặng.
fn segment_cut_leaves_empty_piece(segment_id: i64) -> IpcError {
    IpcError::new(
        "segment.cut_leaves_empty_piece",
        MessageKey::SegmentCutLeavesEmptyPiece,
        BTreeMap::from([("segment_id".to_owned(), segment_id.to_string())]),
        false,
    )
}

/// Một hàng `segment` đọc lên để đưa vào phép tính thuần — **sở hữu**, không mượn.
///
/// ⚠️ Sở hữu vì nó phải sống qua ranh giới closure của `Store::write` (`'static`, nó đi
/// sang luồng writer của AD-11), đúng lý do `confirm_segment` phải `to_owned()` cái mốc.
struct LoadedSegment {
    id: i64,
    ord: i64,
    source_text: String,
    target_text: String,
    flags: ParagraphFlags,
    is_omitted: bool,
    /// The AD-50 arbitration of this row, not the stored `translation_origin`.
    translation_origin: String,
    role: Option<String>,
}

impl LoadedSegment {
    /// Mượn xuống tầng thuần. Không chép một byte nào.
    fn as_part(&self) -> SegmentPart<'_> {
        SegmentPart {
            source_text: &self.source_text,
            target_text: &self.target_text,
            flags: self.flags,
            is_omitted: self.is_omitted,
            translation_origin: &self.translation_origin,
        }
    }
}

/// Đọc một hàng `segment` **trong** giao dịch ghi, và từ chối nếu nó đã về hưu.
///
/// ⚠️ Đọc **trong** giao dịch, cùng lý do `confirm_segment` và `set_segment_omitted` đã ghi:
/// giữa một lượt `read` và một lượt `write` sau đó, tầng kho **không giữ gì cả**.
fn load_segment_for_write(
    tx: &Transaction<'_>,
    segment_id: i64,
) -> Result<LoadedSegment, RegroupReject> {
    let found = tx.query_row(
        "SELECT ord, source_text, target_text, is_paragraph_end, is_target_paragraph_end, \
         is_omitted, baseline_target_text, baseline_translation_origin, retired_at, role \
         FROM segment WHERE id = ?1",
        [segment_id],
        |row| {
            let ord: i64 = row.get(0)?;
            let source_text: String = row.get(1)?;
            let target_text: String = row.get(2)?;
            let source_flag: i64 = row.get(3)?;
            let target_flag: i64 = row.get(4)?;
            let is_omitted: i64 = row.get(5)?;
            let baseline_text: String = row.get(6)?;
            let baseline_origin: String = row.get(7)?;
            let retired_at: Option<String> = row.get(8)?;
            let role: Option<String> = row.get(9)?;
            Ok((
                LoadedSegment {
                    id: segment_id,
                    ord,
                    source_text,
                    target_text,
                    // 🔴 HAI co, doc RIENG tung cot. Xem `paragraph.rs:88-98`: lay co nguon
                    // roi suy "co dich chac cung vay" XOA quyet dinh ngat doan cua nguoi dung.
                    flags: ParagraphFlags {
                        source: source_flag != 0,
                        target: target_flag != 0,
                    },
                    is_omitted: is_omitted != 0,
                    translation_origin: String::new(),
                    role,
                },
                baseline_text,
                baseline_origin,
                retired_at,
            ))
        },
    );

    match found {
        Ok((mut row, baseline_text, baseline_origin, None)) => {
            match crate::core::segment::translation_origin::arbitrate(
                &row.target_text,
                &baseline_text,
                &baseline_origin,
            ) {
                Ok(arbitrated) => {
                    row.translation_origin = arbitrated.as_str().to_owned();
                    Ok(row)
                }
                Err(crate::core::segment::translation_origin::UnknownBaselineOrigin(value)) => {
                    Err(RegroupReject::UnknownOrigin(segment_id, value))
                }
            }
        }
        Ok((_, _, _, Some(_))) => Err(RegroupReject::Retired(segment_id)),
        Err(SqlError::QueryReturnedNoRows) => Err(RegroupReject::NotFound(segment_id)),
        // ⚠️ Mot loi KHO that di nguoc len nguyen dang — no khong phai mot phep tu choi
        // nghiep vu, va gan cho no mot `MessageKey` la noi doi ve nguyen nhan.
        Err(_) => Err(RegroupReject::NotFound(segment_id)),
    }
}

/// Ghi trọn một lượt gộp/tách xuống đĩa, **trong đúng một giao dịch**.
///
/// `retire` là những segment về hưu; `fresh` là những hàng mới, **theo đúng thứ tự** chúng
/// phải đứng trong Chương.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 KHÔNG MỘT HÀNG `segment_version` NÀO ĐƯỢC TẠO — AD-31, và nó dễ làm sai
/// ─────────────────────────────────────────────────────────────────────────────
/// Bảng máy trạng thái của AD-31 (`ARCHITECTURE-SPINE.md`) có một hàng viết thẳng:
/// *"Về hưu do gộp/tách (AD-5) | về hưu | **không** tạo"*. Bản năng khi đọc *"đừng để mất
/// bản dịch của người dùng"* là chụp một `segment_version` trước khi cho về hưu — và lượt
/// chụp đó **phá AC3** *(segment mới bắt đầu với lịch sử RỖNG)* theo một cách đọc rất giống
/// một tính năng.
///
/// ⇒ Bản dịch cũ **không mất**: nó đi vào `target_text` của hàng mới (chữ ký #3(b)). Cái
/// không được giữ là một **hàng lịch sử**, và AD-5 nói đúng điều đó bằng chữ.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 CÂU `INSERT` KHAI **ĐỦ MỌI CỘT NÓ SỞ HỮU** — bài học 2.5d, lặp lại ở 2.7
/// ─────────────────────────────────────────────────────────────────────────────
/// `DEFAULT` của lược đồ **không với tới** một hàng do đường mã này tạo ra. Ba cột
/// (`is_omitted` · `is_target_paragraph_end` · `translation_origin`) đã vào kho theo đúng
/// khuôn đó và cả ba đều phải sửa lại ở story sau; câu dưới đây khai **cả chín** cột.
///
/// ⚠️ **`status` KHÔNG có trong danh sách, và đó là chủ ý:** `DEFAULT 'draft'` của bước 7 là
/// **đúng** giá trị AD-5 đòi (*"chưa xác nhận"*), và khai lại nó ở đây là chép một hằng vào
/// chỗ thứ hai. Một `INSERT` viết `'draft'` tường minh sẽ **không** đỏ ngày ai đó đổi
/// `DEFAULT` — nó chỉ lặng lẽ giữ giá trị cũ.
fn write_regroup(
    tx: &Transaction<'_>,
    chapter_id: i64,
    retire: &[i64],
    fresh: &[NewSegment],
    ord_dau_nhom: i64,
) -> SqlResult<Vec<i64>> {
    // 🔴 THÊM (Ice chốt 2026-09-09, "duy trì hàng `asset` qua bốn đường tổ chức lại Chương",
    // mệnh đề GOM NHÓM) — dời `anchor_after_segment_ord` của mọi ảnh trong Chương này ĐÚNG
    // theo phép đánh số lại mà bước ③ dưới đây sắp làm cho `segment`, TRƯỚC KHI bất kỳ
    // `UPDATE segment` nào chạy (công thức chỉ cần `ord_dau_nhom`/K/M, không cần đọc lại
    // `segment` sau khi đổi).
    //
    // K = số hàng VỀ HƯU (`retire.len()`), M = số hàng MỚI (`fresh.len()`). Trước lượt này,
    // mọi hàng SỐNG có `ord` liên tục 1..N_cu (bất biến do CHÍNH bước ③ của lượt gọi trước
    // giữ). Ba miền:
    // - `k < ord_dau_nhom` — ảnh neo TRƯỚC TRỌN nhóm bị gộp/tách: không đổi (segment nó đếm
    //   giữ nguyên `ord`).
    // - `ord_dau_nhom <= k < ord_dau_nhom + K` — ảnh neo TẠI HOẶC BÊN TRONG nhóm cũ (kể cả
    //   ranh giới "ngay sau segment CUỐI của nhóm cũ", trường hợp THƯỜNG GẶP nhất — ảnh
    //   đứng giữa hai khối mà một lượt gộp/tách vừa chạm đúng ranh giới đó). Snap ảnh về
    //   NGAY SAU nhóm MỚI (`ord_dau_nhom + M - 1`) — nhóm cũ không còn tồn tại dưới hình
    //   dạng cũ để mà trỏ vào GIỮA nó nữa, và "sau nhóm" giữ ảnh gần nhất với phần văn bản
    //   nó từng đứng cạnh. Hướng snap này là quyết định đã chốt cho đúng ca biên này.
    // - `k >= ord_dau_nhom + K` — ảnh neo SAU TRỌN nhóm cũ: dời đúng `M - K` (cùng công thức
    //   segment dùng ở bước ③: một segment tại `ord` p >= ord_dau_nhom+K có `ord` mới =
    //   p - K + M).
    {
        let k_old = i64::try_from(retire.len()).unwrap_or(i64::MAX);
        let m_new = i64::try_from(fresh.len()).unwrap_or(i64::MAX);
        tx.execute(
            "UPDATE asset SET anchor_after_segment_ord = CASE \
                 WHEN anchor_after_segment_ord < ?1 THEN anchor_after_segment_ord \
                 WHEN anchor_after_segment_ord < ?1 + ?2 THEN ?1 + ?3 - 1 \
                 ELSE anchor_after_segment_ord - ?2 + ?3 \
             END \
             WHERE chapter_id = ?4",
            (ord_dau_nhom, k_old, m_new, chapter_id),
        )?;
    }

    // ① VE HUU. `strftime` cua SQLite, cung khuon moc thoi diem voi ca kho (ISO-8601 UTC).
    {
        let mut stmt = tx.prepare_cached(
            "UPDATE segment SET retired_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?1",
        )?;
        for id in retire {
            stmt.execute([id])?;
        }
    }

    // ② TAO MOI. `ord` tam thoi = `ord` cua hang dau nhom; buoc ③ danh lai tat ca.
    //
    // ⚠️ `ord` KHONG `UNIQUE`, va `schema.rs:279-282` ghi do la chu y — de ho tam trong mot
    // giao dich nhieu buoc. Day dung la giao dich do.
    let mut new_ids = Vec::with_capacity(fresh.len());
    {
        let mut stmt = tx.prepare_cached(
            "INSERT INTO segment (chapter_id, ord, source_text, target_text, is_paragraph_end, \
             is_target_paragraph_end, is_omitted, translation_origin, baseline_target_text, \
             baseline_translation_origin, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?4, ?8, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
             strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
        )?;
        for row in fresh {
            stmt.execute((
                chapter_id,
                ord_dau_nhom,
                &row.source_text,
                &row.target_text,
                i64::from(row.flags.source),
                i64::from(row.flags.target),
                i64::from(row.is_omitted),
                &row.translation_origin,
            ))?;
            new_ids.push(tx.last_insert_rowid());
        }
    }

    // ③ DANH LAI `ord` LIEN TUC 1..N cho moi hang CON SONG — chu ky #7(a) cua Ice.
    //
    // `ORDER BY ord, id` la thu tu dung: hang moi mang `ord` cua hang dau nhom, nen chung
    // dung DUNG CHO nhom cu tung dung, va giua chung voi nhau thi `id` tang dan = thu tu
    // chen = thu tu manh.
    let song: Vec<i64> = {
        let mut stmt = tx.prepare(
            "SELECT id FROM segment WHERE chapter_id = ?1 AND retired_at IS NULL ORDER BY ord, id",
        )?;
        let rows = stmt.query_map([chapter_id], |row| row.get(0))?;
        rows.collect::<SqlResult<Vec<i64>>>()?
    };
    {
        let mut stmt = tx.prepare_cached("UPDATE segment SET ord = ?1 WHERE id = ?2")?;
        for (index, id) in song.iter().enumerate() {
            let ord = i64::try_from(index).unwrap_or(i64::MAX).saturating_add(1);
            stmt.execute((ord, id))?;
        }
    }

    // ④ HANG VE HUU nhan `ord` cua hang dau nhom — AD-5 hua "van mo duoc ve DUNG VI TRI
    //    trong Chuong", va mot cho danh dau (FR119) tro toi no phai dap xuong dung do.
    {
        let mut stmt = tx.prepare_cached("UPDATE segment SET ord = ?1 WHERE id = ?2")?;
        for id in retire {
            stmt.execute((ord_dau_nhom, id))?;
        }
    }

    // ⑤ REBASE NEO MARKER trong CHÍNH giao dịch retire + insert. `segment_id` gốc không
    // đổi; chỉ `navigation_segment_id` đang trỏ vào bất kỳ hàng vừa về hưu nào chuyển sang
    // ID mới đầu tiên. Lượt regroup sau lại bắt neo ấy nếu chính ID mới tiếp tục về hưu.
    if let Some(first_new_id) = new_ids.first().copied() {
        let mut stmt = tx.prepare_cached(
            "UPDATE reading_mark SET navigation_segment_id = ?1 WHERE navigation_segment_id = ?2",
        )?;
        for retired_id in retire {
            stmt.execute((first_new_id, retired_id))?;
        }
    }

    crate::core::export::move_members_of_retired(tx, retire, &new_ids)?;

    Ok(new_ids)
}

/// Đọc lại một nhóm hàng ở **hình dạng dây** — chữ ký #4(a).
///
/// ⚠️ Dùng cho **cả hai** phía của một lượt gộp/tách, và luôn gọi **SAU** lượt ghi: hàng về
/// hưu chỉ mang `retired_at` khác `NULL` và `ord` đã đánh lại sau bước ④ của
/// [`write_regroup`]. Một bản chụp trước lượt ghi sẽ nói dối về cả hai.
fn read_fresh_rows(tx: &Transaction<'_>, ids: &[i64]) -> SqlResult<Vec<ChapterSegment>> {
    let mut stmt = tx.prepare(
        "SELECT id, ord, source_text, target_text, is_paragraph_end, retired_at, status, \
         is_omitted, is_target_paragraph_end, role, translation_origin FROM segment WHERE id = ?1",
    )?;
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        out.push(stmt.query_row([id], |row| {
            let flag: i64 = row.get(4)?;
            let omitted: i64 = row.get(7)?;
            let target_para_end: i64 = row.get(8)?;
            Ok(ChapterSegment {
                id: row.get(0)?,
                ord: row.get(1)?,
                source_text: row.get(2)?,
                target_text: row.get(3)?,
                is_paragraph_end: flag != 0,
                retired_at: row.get(5)?,
                status: row.get(6)?,
                is_omitted: omitted != 0,
                is_target_paragraph_end: target_para_end != 0,
                // 🔴 Story 6.13, AC "Gộp/tách một segment vai" — hàng MỚI của một lượt gộp/
                // tách (`write_regroup`'s `INSERT INTO segment` không đặt cột `role`) luôn
                // đọc lại `NULL` ở đây: vai KHÔNG nhân bản, AD-5 về hưu + tạo mới đúng nghĩa
                // "một câu mới không thừa kế vai của câu cũ".
                role: row.get(9)?,
                translation_origin: row.get(10)?,
            })
        })?);
    }
    Ok(out)
}

/// Hàng mới có phải câu **CUỐI Chương** không — ca ① của AD-37.
///
/// 🔴 *"Luôn luôn"* của ca ① nghĩa là **kể cả khi người dùng đã tự bật cờ đích**: một đoạn
/// không thể kết thúc sau câu cuối cùng. Đây là ca biên duy nhất **không** hỏi cờ cũ.
///
/// Trả `true` khi không còn segment **còn sống** nào đứng sau `ord_cuoi_nhom` trong Chương,
/// sau khi đã trừ những segment sắp về hưu.
fn nhom_o_cuoi_chuong(
    tx: &Transaction<'_>,
    chapter_id: i64,
    ord_cuoi_nhom: i64,
    retire: &[i64],
) -> SqlResult<bool> {
    let mut stmt = tx.prepare(
        "SELECT COUNT(*) FROM segment \
         WHERE chapter_id = ?1 AND retired_at IS NULL AND ord > ?2",
    )?;
    let sau: i64 = stmt.query_row((chapter_id, ord_cuoi_nhom), |row| row.get(0))?;
    // Nhung hang sap ve huu van dang `retired_at IS NULL` luc dem, nhung ca hai lenh chi ve
    // huu nhung hang co `ord <= ord_cuoi_nhom`, nen chung khong the roi vao phep dem tren.
    let _ = retire;
    Ok(sau == 0)
}

/// **GỘP hai segment liền nhau** — hàm thuần, đây là thứ test gọi. Story 2.8, AC1 · AC3 ·
/// AC5 · AC6.
///
/// Gộp `segment_id` với segment **liền trên** nó trong Chương — **chữ ký #1(a) của Ice**
/// (2026-08-17). AC6 (*"gộp một **nhóm**"*) vì thế đóng **một nửa**: bảng luật ở
/// [`crate::core::segment::regroup::merge`] nhận `n` bất kỳ, còn bề mặt hôm nay gọi nó với
/// `n = 2`. Món nợ có chủ, ghi ở `deferred-work.md` — năng lực **chọn nhiều hàng** không tồn
/// tại trong kho (`editorPanelState.ts:57` là một `Ref<number | null>`) và không tài liệu
/// nào của dự án mô tả cơ chế chọn đó.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - `segment_id` không có ⇒ `segment.not_found`;
/// - segment đã về hưu ⇒ `segment.retired` (AD-5, đường **ghi** từ chối);
/// - segment là câu **đầu** Chương ⇒ `segment.no_previous`;
/// - đường đọc/ghi trượt ⇒ lỗi kho (`store.*`).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT THAO TÁC RỜI RẠC ⇒ GHI **NGAY**, KHÔNG QUA BỘ ĐỆM GÕ
/// ─────────────────────────────────────────────────────────────────────────────
/// Cùng hạng với `confirm_segment`, FR94 và FR58. Định tuyến lượt này qua bộ đệm gõ của
/// AD-35 khiến một thao tác người dùng **thấy đã xong** — hai câu đã biến khỏi lưới — nằm
/// chờ tới **5 giây** rồi biến mất nếu app sập.
///
/// ⚠️ **Nghĩa vụ của tầng gọi:** flush mọi ký tự đang chờ **trước** lượt gọi này. Bản dịch
/// đi vào hàng mới đọc từ **đĩa**, và `editorEditedText` có thể còn giữ ký tự chưa xuống WAL.
pub fn merge_segments(
    open: Option<&OpenWork>,
    segment_id: i64,
) -> Result<RegroupOutcome, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let source_lang = open.meta.source_lang.clone();

    let reject: Arc<Mutex<Option<RegroupReject>>> = Arc::new(Mutex::new(None));
    let reject_in = Arc::clone(&reject);

    let outcome = open.store.write(move |tx: &Transaction<'_>| {
        let set_reject = |r: RegroupReject| {
            *reject_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(r);
        };

        // ① Doc hang dang chon — TRONG cung giao dich voi luot ghi.
        let duoi = match load_segment_for_write(tx, segment_id) {
            Ok(row) => row,
            Err(r) => {
                set_reject(r);
                return Err(SqlError::QueryReturnedNoRows);
            }
        };

        let chapter_id: i64 =
            tx.query_row("SELECT chapter_id FROM segment WHERE id = ?1", [segment_id], |row| {
                row.get(0)
            })?;

        // ② Tim cau LIEN TREN. `ORDER BY ord DESC, id DESC LIMIT 1` chu khong `ord - 1`:
        //    `ord` lien tuc la mot bat bien ma buoc ③ cua `write_regroup` GIU, khong mot
        //    bat bien duong doc nay duoc phep GIA DINH — mot Chuong dang do dang giua hai
        //    lenh se lam mot phep tru im lang tro sai hang.
        let tren_id: Option<i64> = {
            let mut stmt = tx.prepare(
                "SELECT id FROM segment \
                 WHERE chapter_id = ?1 AND retired_at IS NULL AND ord < ?2 \
                 ORDER BY ord DESC, id DESC LIMIT 1",
            )?;
            let mut rows = stmt.query_map((chapter_id, duoi.ord), |row| row.get(0))?;
            match rows.next() {
                Some(Ok(id)) => Some(id),
                Some(Err(err)) => return Err(err),
                None => None,
            }
        };

        let Some(tren_id) = tren_id else {
            set_reject(RegroupReject::NoPreviousSegment(segment_id));
            return Err(SqlError::QueryReturnedNoRows);
        };

        let tren = match load_segment_for_write(tx, tren_id) {
            Ok(row) => row,
            Err(r) => {
                set_reject(r);
                return Err(SqlError::QueryReturnedNoRows);
            }
        };

        if let Some(id) = [&tren, &duoi].iter().find(|row| row.role.is_some()).map(|row| row.id) {
            set_reject(RegroupReject::RoleSegment(id));
            return Err(SqlError::QueryReturnedNoRows);
        }

        // ③ PHEP TINH THUAN. Bon luat (AD-37 · AD-47 ④ · chu ky #5(a) · #3(b)) song o
        //    `core::segment::regroup`, khong o day.
        let nhom = [tren.as_part(), duoi.as_part()];
        let Some(mut moi) = merge(&nhom, &source_lang) else {
            // `merge` chi tra `None` cho mot nhom RONG, va nhom o day luon co hai phan tu.
            // Nhanh nay viet ra thay vi `.expect()` — `panic = "abort"` giet ca tien trinh.
            set_reject(RegroupReject::NotFound(segment_id));
            return Err(SqlError::QueryReturnedNoRows);
        };

        // ④ Ca ① cua AD-37 — segment CUOI Chuong: ca hai co TAT, luon luon.
        let retire = [tren.id, duoi.id];
        if nhom_o_cuoi_chuong(tx, chapter_id, duoi.ord, &retire)? {
            moi.flags = at_end_of_chapter(moi.flags);
        }

        let ord_dau = tren.ord.min(duoi.ord);
        let new_ids = write_regroup(tx, chapter_id, &retire, std::slice::from_ref(&moi), ord_dau)?;

        Ok(RegroupOutcome {
            // Doc lai SAU luot ghi — hai hang nay nay mang `retired_at` khac NULL va `ord`
            // da danh lai. Mot ban chup truoc luot ghi se noi doi ve ca hai.
            retired: read_fresh_rows(tx, &retire)?,
            new_segments: read_fresh_rows(tx, &new_ids)?,
        })
    });

    ket_qua_regroup(outcome, &reject)
}

/// **TÁCH một segment tại `cut`** — hàm thuần, đây là thứ test gọi. Story 2.8, AC2 · AC3 ·
/// AC5 · AC7.
///
/// `cut` là chỉ số tính bằng **ký tự Unicode** của `source_text`, không byte.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - `segment_id` không có ⇒ `segment.not_found`;
/// - segment đã về hưu ⇒ `segment.retired`;
/// - chỗ cắt để lại một mảnh rỗng ⇒ `segment.cut_leaves_empty_piece`;
/// - đường đọc/ghi trượt ⇒ lỗi kho (`store.*`).
///
/// ⚠️ Cùng nghĩa vụ *"ghi ngay, không qua bộ đệm gõ"* và cùng nghĩa vụ flush của tầng gọi
/// như [`merge_segments`].
pub fn split_segment(
    open: Option<&OpenWork>,
    segment_id: i64,
    cuts: Vec<usize>,
) -> Result<RegroupOutcome, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    let reject: Arc<Mutex<Option<RegroupReject>>> = Arc::new(Mutex::new(None));
    let reject_in = Arc::clone(&reject);

    let outcome = open.store.write(move |tx: &Transaction<'_>| {
        let set_reject = |r: RegroupReject| {
            *reject_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(r);
        };

        let goc = match load_segment_for_write(tx, segment_id) {
            Ok(row) => row,
            Err(r) => {
                set_reject(r);
                return Err(SqlError::QueryReturnedNoRows);
            }
        };

        let chapter_id: i64 =
            tx.query_row("SELECT chapter_id FROM segment WHERE id = ?1", [segment_id], |row| {
                row.get(0)
            })?;

        if goc.role.is_some() {
            set_reject(RegroupReject::RoleSegment(goc.id));
            return Err(SqlError::QueryReturnedNoRows);
        }

        let Some(mut manh) = split_at(&goc.as_part(), &cuts) else {
            set_reject(RegroupReject::CutLeavesEmptyPiece(segment_id));
            return Err(SqlError::QueryReturnedNoRows);
        };

        // Ca ① cua AD-37 ap cho manh CUOI — no la hang cuoi cua nhom moi.
        let retire = [goc.id];
        if nhom_o_cuoi_chuong(tx, chapter_id, goc.ord, &retire)? {
            // `split_at` tra it nhat hai manh khi no tra `Some`; nhanh `if let` viet ra thay
            // vi mot chi so tran, vi mot `panic` o day giet ca tien trinh.
            if let Some(cuoi) = manh.last_mut() {
                cuoi.flags = at_end_of_chapter(cuoi.flags);
            }
        }

        let new_ids = write_regroup(tx, chapter_id, &retire, &manh, goc.ord)?;

        Ok(RegroupOutcome {
            // Cung ly do voi `merge_segments`: doc lai SAU luot ghi.
            retired: read_fresh_rows(tx, &retire)?,
            new_segments: read_fresh_rows(tx, &new_ids)?,
        })
    });

    ket_qua_regroup(outcome, &reject)
}

/// Đổi kết quả của một lượt `Store::write` thành `Result<RegroupOutcome, IpcError>`.
///
/// ⚠️ Rút ra vì **hai** lệnh dùng chung, không vì nó dài: một bản chép thứ hai của bảng
/// `match` này là chỗ để một lượt sửa sau chỉ chạm một nửa — đúng khuôn *"chữ ký thi hành
/// đúng MỘT NỬA"* đã lặp bốn lần ở 2.5b và 2.6.
fn ket_qua_regroup(
    outcome: Result<RegroupOutcome, crate::core::store::StoreError>,
    reject: &Arc<Mutex<Option<RegroupReject>>>,
) -> Result<RegroupOutcome, IpcError> {
    match outcome {
        Ok(value) => Ok(value),
        Err(err) => {
            let taken = reject
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            match taken {
                Some(r) => Err(r.into_ipc()),
                // O rong ⇒ day la mot loi KHO that, khong mot phep tu choi nghiep vu.
                None => Err(err.into()),
            }
        }
    }
}
