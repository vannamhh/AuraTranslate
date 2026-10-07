use super::*;

/// Kết quả một lượt xác nhận — thứ đi ra qua dây. Story 2.5, AC2 · AC13.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
///
/// 🔴 `version_created` **không** suy ra được từ `status`, và đó là toàn bộ lý do nó tồn
/// tại: hai lượt cùng cho `status = "confirmed"` — một lượt **chuyển tiếp thật** và một lượt
/// **xác nhận lại vô hại** (AC13) — khác nhau ở đúng trường này. Gộp chúng lại là làm webview
/// không phân biệt được *"vừa ký"* với *"đã ký từ trước"*, và Story 2.7 (xuất xứ) cùng Epic 7
/// (cặp TM) móc vào **chuyển tiếp**, không móc vào trạng thái.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfirmOutcome {
    /// Segment vừa được xác nhận.
    pub segment_id: i64,
    /// Trạng thái **sau** lượt gọi. Luôn `"confirmed"` khi `Ok` — một lượt trả `Ok` mà
    /// trạng thái chưa tới đích là đúng thứ *"trả 'đã xong' cho một lượt không ghi gì"* mà
    /// AC14 cấm.
    pub status: String,
    /// Lượt gọi này có **chuyển tiếp** hay không. `false` ⇒ segment đã ở đích từ trước
    /// (AC13), và **không** hàng `segment_version` nào được sinh.
    pub version_created: bool,
}

/// `segment_id` không có trong `project.db` của Tác phẩm đang mở.
///
/// ⚠️ `pub(crate)` từ 2026-08-29 (Story 5.8), cùng tiền lệ `no_work_open`/`chapter_not_found`
/// của `commands::chapter`: `commands::chapter::split_chapter_at_segment` tái dùng ĐÚNG hàm
/// này cho ca "segment_id không thuộc Chương đang mở" — cùng câu, cùng nghĩa, và một khoá thứ
/// hai cho nó là hai chuỗi phải giữ khớp nhau bằng kỷ luật.
pub(crate) fn segment_not_found(segment_id: i64) -> IpcError {
    IpcError::new(
        "segment.not_found",
        MessageKey::SegmentNotFound,
        BTreeMap::from([("segment_id".to_owned(), segment_id.to_string())]),
        false,
    )
}

/// Segment đã về hưu (AD-5) ⇒ không xác nhận được.
pub(super) fn segment_retired(segment_id: i64) -> IpcError {
    IpcError::new(
        "segment.retired",
        MessageKey::SegmentRetired,
        BTreeMap::from([("segment_id".to_owned(), segment_id.to_string())]),
        false,
    )
}

/// Segment là câu **cuối Chương** ⇒ không đặt được cờ kết đoạn cho bản dịch.
///
/// 🔴 Ca ① của AD-37, và là ca biên duy nhất **không** hỏi cờ cũ — code review 2026-08-16,
/// Ice ký đường (a). Hàm thuần phát biểu ca này là
/// [`crate::core::segment::paragraph::at_end_of_chapter`]; đây là chỗ nó được cưỡng chế
/// trên **đường ghi**, thứ mà `split::mark_paragraph_end` chỉ làm được cho đường **nhập**.
pub(super) fn segment_ends_chapter(segment_id: i64) -> IpcError {
    IpcError::new(
        "segment.ends_chapter",
        MessageKey::SegmentEndsChapter,
        BTreeMap::from([("segment_id".to_owned(), segment_id.to_string())]),
        false,
    )
}

/// Câu chưa dịch ⇒ **từ chối**. Quyết định #7, Ice ký 2026-08-14.
fn segment_nothing_to_confirm(segment_id: i64) -> IpcError {
    IpcError::new(
        "segment.nothing_to_confirm",
        MessageKey::SegmentNothingToConfirm,
        BTreeMap::from([("segment_id".to_owned(), segment_id.to_string())]),
        false,
    )
}

pub(super) fn segment_unknown_translation_origin(segment_id: i64, value: &str) -> IpcError {
    IpcError::new(
        "segment.unknown_translation_origin",
        MessageKey::SegmentUnknownTranslationOrigin,
        BTreeMap::from([
            ("segment_id".to_owned(), segment_id.to_string()),
            ("value".to_owned(), value.to_owned()),
        ]),
        false,
    )
}

/// Lý do một lượt xác nhận bị **từ chối**, mang ra khỏi closure ghi.
///
/// ⚠️ Cùng khuôn và cùng lý do định lượng với [`BatchReject`]: `Store::write` gói mọi `Err`
/// thành `StoreError::WriteFailed { detail: String }`, nên đoán lại lý do từ chuỗi `detail`
/// là một chẩn đoán SAI cho bốn ca khác hẳn nhau. Ô lỗi **có kiểu**, không đoán lại từ chuỗi.
enum ConfirmReject {
    /// Không có hàng `segment` nào mang `segment_id` đó.
    NotFound,
    /// `retired_at` khác `NULL` (AD-5).
    Retired,
    /// `target_text` rỗng — chưa có gì để ký.
    NothingToConfirm,
    /// `baseline_translation_origin` lies outside [`TRANSLATION_ORIGINS`].
    UnknownOrigin(String),
}

/// **Xác nhận một segment** — hàm thuần, đây là thứ test gọi. Story 2.5 · FR24 · AD-31.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - `segment_id` không có trong Tác phẩm đang mở ⇒ `segment.not_found`;
/// - segment đã về hưu ⇒ `segment.retired` (AD-5, hàng rào viết trước — chủ Story 2.8);
/// - `target_text` rỗng ⇒ `segment.nothing_to_confirm` (Quyết định #7);
/// - đường đọc/ghi trượt ⇒ lỗi kho (`store.*`), qua `From<StoreError>`.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT GIAO DỊCH, VÀ PHÉP PHÂN XỬ NẰM **TRONG** NÓ
/// ─────────────────────────────────────────────────────────────────────────────
/// Đọc trạng thái ở một lượt `Store::read` rồi ghi ở một lượt `Store::write` sau đó là mở
/// đúng khe hở mà [`split_chapter_into_segments`] đã ghi bằng chữ: giữa hai lượt, tầng kho
/// **không giữ gì cả**. Ở đây khe hở đó đắt hơn hẳn — hai lượt `invoke` song song cùng đọc
/// `status = 'draft'`, cùng chạy nhánh chuyển tiếp, và ghi **hai** `SegmentVersion` cho một
/// lần bấm phím. Đó chính là hố (1) của AD-31 §Prevents *(lịch sử đầy bản sao ⇒ FR101 vô
/// dụng)*, chỉ khác đường tới.
///
/// ⇒ Đọc **và** ghi trong **cùng** `Store::write`. Hàm này vì thế **không** cần dựa vào
/// `MutexGuard` của [`wire`] để đúng — khác [`split_chapter_into_segments`], và khác có chủ
/// ý: nó không phải chạy một phép CPU nào ở giữa. Đây đúng thứ mà doc-comment của hàm kia
/// gọi là *"đường đúng nếu về sau cần"*.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 AC13 — MỘT SEGMENT ĐÃ Ở ĐÍCH THÌ **KHÔNG CHUYỂN TIẾP**
/// ─────────────────────────────────────────────────────────────────────────────
/// Bảng AD-31 lập chỉ mục theo **sự kiện chuyển tiếp**, và AC2 khoá vào chữ *"chuyển sang"*
/// đã xác nhận. Giữ phím xác nhận trên một câu đã ký vì thế là một **no-op**: không hàng
/// `segment_version` thứ hai, và **không** đụng `updated_at`.
///
/// ⚠️ Vì sao phép kiểm *"văn bản có đổi không"* KHÔNG cần ở đây: một lượt sửa văn bản đã hạ
/// segment về `'draft'` trước đó ([`unconfirm_edited_segments`]). ⇒ `status = 'confirmed'`
/// **đã hàm ý** *"văn bản y nguyên từ lần ký"*, và một lượt đọc `segment_version` để so lại
/// là hỏi một câu mà máy trạng thái đã trả lời.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `updated_at` KHÔNG đổi ở lượt xác nhận — và đó là một quyết định, không một lượt quên
/// ─────────────────────────────────────────────────────────────────────────────
/// `updated_at` mang nghĩa *"mốc sửa **văn bản**"* — nó do [`save_segment_targets`] sinh, và
/// `SEGMENT_DDL` phân biệt nó với `created_at` (*"mốc TẠO, không phải mốc sửa"*). Một lượt ký
/// không sửa một ký tự nào. Thời điểm ký **có** chỗ ghi riêng và chính xác hơn:
/// `segment_version.created_at`, thứ Story 2.6 đọc. Bơm `updated_at` ở đây là cho một cột hai
/// nghĩa, rồi Story 2.6 sẽ so hai mốc sinh ra từ hai sự kiện khác nhau.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔵 MỐI NỐI ĐỂ MỞ — VẾ XUẤT XỨ **ĐÃ ĐÓNG** 2026-08-16 (Story 2.7), VẾ CẶP TM Ở LẠI
/// ─────────────────────────────────────────────────────────────────────────────
/// Nhánh `Ok(true)` ngay dưới **là** chuyển tiếp *"sang đã xác nhận"* của AD-31, và đó là
/// **chỗ duy nhất** hai thứ sau được ghi:
/// - **xuất xứ (FR117)** — ✅ **đã cài ở chính story này**, xem khối phân xử ngay dưới. Câu
///   *"chủ: Story 2.7"* của bản trước đã hết đúng; sửa tại chỗ thay vì để nó lặng lẽ sai.
/// - **cặp TM (FR56)** — ✅ ghi bởi [`crate::core::tm::insert_pair`] trong cùng giao dịch,
///   mang đúng giá trị `translation_origin` vừa ghi cho segment. Nhánh `Ok(false)` không ghi
///   cặp nào. AD-47 ⑥ khai phép chiếu xuất xứ ba giá trị → **trục nhị phân FR118**; đó là
///   việc của đường đọc.
///
/// Origin arbitration (AD-50 rule 4): the origin comes from [`crate::core::segment::translation_origin::arbitrate`]
/// over `target_text` and the stored baseline columns, read in the same transaction. The webview
/// sends only `segment_id`. A baseline origin outside [`TRANSLATION_ORIGINS`] rejects with
/// `segment.unknown_translation_origin` and writes nothing.
pub fn confirm_segment(
    open: Option<&OpenWork>,
    segment_id: i64,
) -> Result<ConfirmOutcome, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    let reject: Arc<Mutex<Option<ConfirmReject>>> = Arc::new(Mutex::new(None));
    let reject_in = Arc::clone(&reject);

    let outcome = open.store.write(move |tx: &Transaction<'_>| {
        let set_reject = |r: ConfirmReject| {
            *reject_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(r);
        };

        // ① Doc trang thai hien tai — TRONG cung giao dich voi luot ghi.
        //
        // The baseline columns are read here, in the writing transaction, never from the caller.
        let found = tx.query_row(
            "SELECT target_text, status, retired_at, baseline_target_text, \
             baseline_translation_origin, source_text FROM segment WHERE id = ?1",
            [segment_id],
            |row| {
                let target_text: String = row.get(0)?;
                let status: String = row.get(1)?;
                let retired_at: Option<String> = row.get(2)?;
                let baseline_target_text: String = row.get(3)?;
                let baseline_translation_origin: String = row.get(4)?;
                let source_text: String = row.get(5)?;
                Ok((
                    target_text,
                    status,
                    retired_at,
                    baseline_target_text,
                    baseline_translation_origin,
                    source_text,
                ))
            },
        );

        let (target_text, status, retired_at, baseline_text, baseline_origin, source_text) = match found {
            Ok(value) => value,
            Err(SqlError::QueryReturnedNoRows) => {
                set_reject(ConfirmReject::NotFound);
                return Err(SqlError::QueryReturnedNoRows);
            }
            Err(err) => return Err(err),
        };

        // ② Phan xu, theo dung thu tu cua AC14 — moi loi tu choi PHAN BIET DUOC.
        if retired_at.is_some() {
            set_reject(ConfirmReject::Retired);
            return Err(SqlError::QueryReturnedNoRows);
        }
        // 🔵 Code review 2026-08-14: `is_empty()` MOT MINH de lot mot ca that.
        //
        // Ban dau day la `target_text.is_empty()`. Mot cau chi co khoang trang ("   ", hay mot
        // `U+00A0` do contenteditable de lai) KHONG rong theo `str::is_empty()`, nen no di
        // thang qua Quyet dinh #7 va duoc KY. Hau qua doc ra tu chinh doc-comment cua
        // `SegmentNothingToConfirm` ngay tren: mot `SegmentVersion` gan nhu trong di vao lich
        // su FR101, va Epic 7 ghi mot cap TM co ve dich la khoang trang — roi FR58 dien san
        // dung khoang trang do o mot Chuong sau. Cung duong hong, cung tinh vinh vien.
        //
        // `str::trim()` cat theo `char::is_whitespace` cua Unicode, nen no phu ca `U+00A0`.
        if target_text.trim().is_empty() {
            set_reject(ConfirmReject::NothingToConfirm);
            return Err(SqlError::QueryReturnedNoRows);
        }

        // ③ AC13 — DA o dich thi KHONG chuyen tiep. Khong ghi mot byte nao.
        if status == SEGMENT_STATUS_CONFIRMED {
            return Ok(false);
        }

        // ④ Transition: the origin is the single AD-50 arbitration, status and origin in one UPDATE.
        let confirmed_origin = match crate::core::segment::translation_origin::arbitrate(
            &target_text,
            &baseline_text,
            &baseline_origin,
        ) {
            Ok(crate::core::segment::translation_origin::Arbitrated::PairOrigin(pair_origin)) => pair_origin,
            Ok(crate::core::segment::translation_origin::Arbitrated::Unsigned) => {
                set_reject(ConfirmReject::NothingToConfirm);
                return Err(SqlError::QueryReturnedNoRows);
            }
            Err(crate::core::segment::translation_origin::UnknownBaselineOrigin(value)) => {
                set_reject(ConfirmReject::UnknownOrigin(value));
                return Err(SqlError::QueryReturnedNoRows);
            }
        };
        tx.execute(
            "UPDATE segment SET status = ?1, translation_origin = ?2 WHERE id = ?3",
            (SEGMENT_STATUS_CONFIRMED, confirmed_origin.as_str(), segment_id),
        )?;
        tx.execute(
            "INSERT INTO segment_version (segment_id, target_text, created_at) \
             VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (segment_id, &target_text),
        )?;
        crate::core::tm::insert_pair(tx, &source_text, &target_text, confirmed_origin)?;

        Ok(true)
    });

    match outcome {
        Ok(version_created) => Ok(ConfirmOutcome {
            segment_id,
            status: SEGMENT_STATUS_CONFIRMED.to_owned(),
            version_created,
        }),
        Err(err) => {
            let taken = reject
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            match taken {
                Some(ConfirmReject::NotFound) => Err(segment_not_found(segment_id)),
                Some(ConfirmReject::Retired) => Err(segment_retired(segment_id)),
                Some(ConfirmReject::NothingToConfirm) => Err(segment_nothing_to_confirm(segment_id)),
                Some(ConfirmReject::UnknownOrigin(value)) => {
                    Err(segment_unknown_translation_origin(segment_id, &value))
                }
                // O rong ⇒ day la mot loi KHO that, khong mot phep tu choi nghiep vu.
                None => Err(err.into()),
            }
        }
    }
}

/// Kết quả một lượt cắt bỏ / bỏ cờ — thứ đi ra qua dây. Story 2.5c, AC1 · AC4.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
///
/// ⚠️ Khác [`ConfirmOutcome`], ở đây **không** có trường kiểu `version_created`, và đó là
/// một mệnh đề chứ không một lượt bỏ sót: cắt bỏ **không phải** một chuyển tiếp AD-31, nên
/// không có sự kiện nào để webview phân biệt *"vừa cắt"* với *"đã cắt từ trước"*. Trạng thái
/// mới nói đủ.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OmitOutcome {
    /// Segment vừa được đặt cờ.
    pub segment_id: i64,
    /// Trạng thái **sau** lượt gọi — không phải trạng thái trước.
    pub is_omitted: bool,
}

/// Kết quả một lượt đặt **cờ kết đoạn của bản dịch** — Story 2.5d, FR134 · AD-46.
///
/// ⚠️ Cùng hình dạng và cùng lý do với [`OmitOutcome`] ngay trên: trạng thái **sau** lượt
/// gọi, không một cờ *"vừa đổi"*. Đổi cờ đoạn **không phải** một chuyển tiếp AD-31, nên
/// không có sự kiện nào để webview phân biệt *"vừa bật"* với *"đã bật từ trước"*.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ParagraphEndOutcome {
    /// Segment vừa được đặt cờ.
    pub segment_id: i64,
    /// Trạng thái **sau** lượt gọi.
    pub is_target_paragraph_end: bool,
}

/// Lý do một lượt cắt bỏ / bỏ cờ bị **từ chối**, mang ra khỏi closure ghi.
///
/// ⚠️ Cùng khuôn và cùng lý do định lượng với [`ConfirmReject`]: `Store::write` gói mọi
/// `Err` thành `StoreError::WriteFailed { detail: String }`, nên đoán lại lý do từ chuỗi
/// `detail` là một chẩn đoán SAI. Ô lỗi **có kiểu**, không đoán lại từ chuỗi.
pub(super) enum OmitReject {
    /// Không có hàng `segment` nào mang `segment_id` đó.
    NotFound,
    /// `retired_at` khác `NULL` (AD-5).
    Retired,
    /// Segment là câu **cuối Chương** — chỉ [`set_segment_paragraph_end`] dùng nhánh này.
    ///
    /// ⚠️ Vì sao nó sống trong `OmitReject` chứ không một enum thứ ba: hai lệnh dùng chung
    /// enum này đọc **cùng một hàng** và từ chối theo **cùng một khuôn**; một enum riêng
    /// cho đúng một biến thể là hai bảng phải giữ khớp bằng kỷ luật. Lệnh cắt bỏ không bao
    /// giờ dựng biến thể này, và `match` của nó nói ra điều đó bằng một nhánh tường minh.
    EndsChapter,
}

/// **Cắt bỏ một câu khỏi bản dịch, hoặc bỏ cờ đó** — hàm thuần, đây là thứ test gọi.
/// Story 2.5c · FR133 · AC1 · AC2 · AC4.
///
/// `omitted = true` là *"câu này không thuộc bản dịch"*; `false` là bỏ cờ.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - `segment_id` không có trong Tác phẩm đang mở ⇒ `segment.not_found`;
/// - segment đã về hưu ⇒ `segment.retired` (AD-5, hàng rào viết trước — chủ Story 2.8);
/// - đường đọc/ghi trượt ⇒ lỗi kho (`store.*`), qua `From<StoreError>`.
///
/// ⚠️ **Không khoá `err.segment.*` mới nào** ra đời cùng hàm này. Ba nhánh trên đã có khoá
/// riêng từ Story 2.5, và không nhánh nào của cắt bỏ là một **lý do từ chối mới** — một
/// khoá thứ tư nói cùng một điều là một danh mục đóng bị nới không lý do.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT THAO TÁC RỜI RẠC ⇒ GHI **NGAY**, KHÔNG QUA BỘ ĐỆM GÕ
/// ─────────────────────────────────────────────────────────────────────────────
/// Một `open.store.write`, một giao dịch — khuôn là [`confirm_segment`], **không** phải
/// [`save_segment_targets`]. Định tuyến lượt này qua bộ đệm gõ của AD-35 khiến một thao tác
/// người dùng **thấy đã xong** nằm chờ tới **5 giây** rồi biến mất nếu app sập
/// (`project-context.md` §Dữ liệu người dùng, cùng hạng với FR94 và FR58).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 CÂU `UPDATE` CHẠM **ĐÚNG MỘT** CỘT — kể cả `updated_at` cũng không
/// ─────────────────────────────────────────────────────────────────────────────
/// Đây là AC2 (*"trục độc lập"*) viết bằng SQL. `updated_at` mang nghĩa **"mốc sửa văn
/// bản"** — nó do [`save_segment_targets`] sinh, và một lượt cắt bỏ không sửa một ký tự
/// nào. Bơm nó ở đây là cho một cột hai nghĩa, đúng lý do [`confirm_segment`] đã từ chối
/// làm việc đó.
///
/// 🔴 Và hệ quả lớn hơn: **AC4 đúng mà không một dòng mã khôi phục nào.** Lượt cắt bỏ không
/// xoá `status` lẫn `target_text`, nên lượt bỏ cờ không phải dựng lại gì — *"quay về đúng
/// trạng thái cũ với nội dung cũ"* là **hệ quả của việc không đụng vào**, không phải kết
/// quả của một đường lưu-rồi-phục-hồi. Một cài đặt hạ `status` về `'draft'` lúc cắt bỏ sẽ
/// **không bao giờ** đoán lại được `'confirmed'` lúc bỏ cờ.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ ĐỌC VÀ GHI TRONG **CÙNG MỘT** `Store::write`
/// ─────────────────────────────────────────────────────────────────────────────
/// Cùng lý do [`confirm_segment`] đã ghi: giữa một lượt `read` và một lượt `write` sau đó,
/// tầng kho **không giữ gì cả**. Cái giá ở đây thấp hơn *(không `segment_version` để nhân
/// bản)*, nhưng phép phân xử *"về hưu chưa"* vẫn phải đọc **trong** giao dịch ghi, nếu
/// không nó phân xử trên một trạng thái có thể đã cũ.
pub fn set_segment_omitted(
    open: Option<&OpenWork>,
    segment_id: i64,
    omitted: bool,
) -> Result<OmitOutcome, IpcError> {
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
            "SELECT is_omitted, retired_at FROM segment WHERE id = ?1",
            [segment_id],
            |row| {
                let is_omitted: i64 = row.get(0)?;
                let retired_at: Option<String> = row.get(1)?;
                Ok((is_omitted != 0, retired_at))
            },
        );

        let (current, retired_at) = match found {
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

        // ② DA o dung gia tri ⇒ khong ghi mot byte nao. Cung luat AC13 cua `confirm_segment`:
        //    giu phim khong duoc sinh ra mot luot ghi thu hai.
        if current == omitted {
            return Ok(());
        }

        // ③ DUNG MOT cot. Khong `updated_at` — xem doc-comment o tren.
        tx.execute(
            "UPDATE segment SET is_omitted = ?1 WHERE id = ?2",
            (i64::from(omitted), segment_id),
        )?;

        Ok(())
    });

    match outcome {
        Ok(()) => Ok(OmitOutcome {
            segment_id,
            is_omitted: omitted,
        }),
        Err(err) => {
            let taken = reject
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            match taken {
                Some(OmitReject::NotFound) => Err(segment_not_found(segment_id)),
                Some(OmitReject::Retired) => Err(segment_retired(segment_id)),
                // 🔴 Nhanh nay KHONG DUNG DEN o day, va no viet ra thay vi mot `_ =>`:
                // "cau cuoi Chuong" khong phai mot ly do tu choi cua lenh CAT BO — mot cau
                // cuoi van cat bo duoc. Mot `_ =>` gop chung se nuot mat su that do vao
                // ngay bien the thu tu ra doi.
                Some(OmitReject::EndsChapter) => Err(err.into()),
                // O rong ⇒ day la mot loi KHO that, khong mot phep tu choi nghiep vu.
                None => Err(err.into()),
            }
        }
    }
}

/// **Đường flush đầy đủ của AD-35** — hàm thuần, đây là thứ vỏ `wire` gọi và thứ test gọi.
/// Story 2.5, AC3 · AC8.
///
/// Trả `(số hàng bị hạ về 'draft', kết quả lô ghi)`.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 HÀM NÀY TỒN TẠI VÌ MỘT PHÉP ĐO, KHÔNG VÌ MỘT SỞ THÍCH VỀ CẤU TRÚC
/// ─────────────────────────────────────────────────────────────────────────────
/// Bản đầu của Story 2.5 đặt thứ tự *"hạ trước, ghi sau"* ngay trong vỏ
/// [`wire::save_segment_targets`]. **Đo 2026-08-14:** đảo hai dòng đó rồi chạy
/// `cargo test --locked --test segment_contract` cho **54/54 XANH** — không một cổng nào
/// bắt được, vì một vỏ `#[tauri::command]` cần `AppHandle` nên `tests/**` **gọi nó không
/// được**. ⇒ Một quy tắc sống trong vỏ là một quy tắc *"vi phạm được mà không cổng nào đỏ"*,
/// đúng hạng mục mà `project-context.md` §Critical Don't-Miss Rules định nghĩa.
///
/// **Hệ quả đo được:** thứ tự chuyển xuống đây, nơi
/// `segment_contract.rs::the_flush_path_lowers_the_state_before_it_writes_the_new_text`
/// gọi thẳng được. Vỏ còn đúng **một** lời gọi và không một quyết định nào.
///
/// ⚠️ Vì sao thứ tự này chứ không thứ tự kia — lý lẽ đầy đủ ở doc-comment của
/// [`unconfirm_edited_segments`]: sập giữa hai giao dịch theo chiều này để lại một trạng
/// thái **hồi phục được**; theo chiều kia để lại hố (2) của AD-31 §Prevents, im lặng vĩnh viễn.
pub fn flush_segment_targets(
    open: Option<&OpenWork>,
    chapter_id: i64,
    edits: &[SegmentTargetEdit],
) -> Result<(usize, SaveOutcome), IpcError> {
    let lowered = unconfirm_edited_segments(open, chapter_id, edits)?;
    let saved = save_segment_targets(open, chapter_id, edits)?;
    Ok((lowered, saved))
}

/// **Hạ về `'draft'` những segment đã ký mà văn bản vừa đổi** — AD-31 hàng 3, AC3.
/// Hàm thuần, đây là thứ test gọi.
///
/// Trả về **số hàng thật sự bị hạ**. `0` là một giá trị hợp lệ và là ca thường nhật nhất.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - 🔵 *(code review 2026-08-14)* `chapter_id` không có ⇒ `chapter.not_found`;
/// - 🔵 *(code review 2026-08-14)* có `segment.id` không thuộc Chương đó ⇒
///   `segment.unknown_ids`, và **chưa một hàng nào bị hạ**. Xem bước ② trong thân hàm: phép
///   kiểm này đứng **trước** lượt hạ chính vì `flush_segment_targets` chạy hai giao dịch, và
///   một lô hỏng phải chết ở giao dịch **một**, không phải nửa chừng.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO MỘT HÀM RIÊNG CHỨ KHÔNG MỘT DÒNG TRONG [`save_segment_targets`] — AC8
/// ─────────────────────────────────────────────────────────────────────────────
/// Bảng AD-31 có **hai** hàng nói về cùng một lượt flush, và chúng không mâu thuẫn:
///
/// | hàng | sự kiện | trạng thái |
/// |---|---|---|
/// | 1 | Auto-save (FR100) | **không đổi** |
/// | 3 | Sửa văn bản của segment **đã xác nhận** | → **chưa xác nhận** |
///
/// Hàng 1 là luật **chung** *(lượt lưu tự nó không ký và không huỷ chữ ký)*; hàng 3 là luật
/// **riêng** cho đúng ca văn bản của một câu đã ký thật sự đổi. Nhét `status` vào câu
/// `UPDATE` của [`save_segment_targets`] làm hàng 1 sai cho **mọi** lượt flush — kể cả lượt
/// mang đúng văn bản đang có — và cổng đứng đó là
/// `tests/segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`.
///
/// ⚠️ Hai hàm ⇒ hai giao dịch, và **thứ tự giữa chúng là một quyết định về mất mát dữ liệu**,
/// không phải một chi tiết. Vỏ [`wire::save_segment_targets`] gọi hàm này **TRƯỚC**:
/// - hạ-rồi-ghi, sập ở giữa ⇒ trạng thái `'draft'` nhưng văn bản chưa đổi. Người dùng thấy
///   một câu *"chưa ký"* mà nội dung đúng như cũ, và ký lại một phím. **Hồi phục được.**
/// - ghi-rồi-hạ, sập ở giữa ⇒ văn bản **đã đổi** mà segment vẫn `'confirmed'`. Không lần xác
///   nhận nào nữa xảy ra ⇒ **cặp TM mới không bao giờ được ghi** — đúng hố (2) của AD-31
///   §Prevents, và nó im lặng vĩnh viễn.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 SO BẰNG **VĂN BẢN**, CẤM CỜ `dirty` — hợp đồng phụ của AD-31
/// ─────────────────────────────────────────────────────────────────────────────
/// `AND target_text <> ?3` **là** phép so đó, và nó chạy ở tầng SQL nơi nó không quên được.
/// Hai cách cho kết quả khác nhau ở đúng ca người dùng gõ rồi **hoàn tác về nguyên trạng**:
/// cờ dirty nói *đã sửa* và huỷ một chữ ký thật; so sánh văn bản nói *không đổi* và giữ nó.
///
/// ⚠️ `AND status = ?4` giữ câu `UPDATE` chỉ chạm những hàng **thật sự chuyển tiếp**, nên
/// `changes()` đọc ra đúng số hàng bị hạ chứ không phải cỡ của lô.
///
/// ⚠️ Hàm này **không** đụng `updated_at`: nó không sửa một ký tự nào — lượt
/// [`save_segment_targets`] ngay sau đó mới sửa, và chính lượt đó bơm `updated_at`.
pub fn unconfirm_edited_segments(
    open: Option<&OpenWork>,
    chapter_id: i64,
    edits: &[SegmentTargetEdit],
) -> Result<usize, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    // Lo RONG khong mo mot giao dich nao — cung ly do va cung khuon `save_segment_targets`.
    if edits.is_empty() {
        return Ok(0);
    }

    let payload: Vec<(i64, String)> = edits
        .iter()
        .map(|e| (e.id, e.target_text.clone()))
        .collect();

    let expected = payload.len();

    // 🔵 Code review 2026-08-14 — O BAO LY DO TU CHOI, them vao hàm này vì một khe hở đo được.
    // Cung khuon va cung ly do dinh luong nhu `save_segment_targets`: mot phep tu choi nghiep
    // vu PHAI di ra bang mot o CO KIEU, khong bang cach doan lai chuoi `WriteFailed`.
    let reject: Arc<Mutex<Option<BatchReject>>> = Arc::new(Mutex::new(None));
    let reject_in = Arc::clone(&reject);

    let outcome = open.store.write(move |tx: &Transaction<'_>| {
        let set_reject = |r: BatchReject| {
            *reject_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(r);
        };

        // ① Chuong co thuoc `project.db` cua Tac pham dang mo khong — cung phep kiem, cung
        //    vi tri va cung ly do nhu `save_segment_targets` buoc ①.
        let chapter_rows: i64 = tx.query_row(
            "SELECT COUNT(*) FROM chapter WHERE id = ?1",
            [chapter_id],
            |row| row.get(0),
        )?;
        if chapter_rows == 0 {
            set_reject(BatchReject::ChapterNotFound);
            return Err(SqlError::QueryReturnedNoRows);
        }

        // ─────────────────────────────────────────────────────────────────────────────
        // 🔴 ② MOI id PHAI THUOC CHUONG NAY — VA PHEP KIEM DUNG TRUOC KHI HA MOT HANG NAO
        // ─────────────────────────────────────────────────────────────────────────────
        // 🔵 Code review 2026-08-14 bat mot khe ho ma ca 11 cong, vitest, cargo test va e2e
        // deu bo lot, vi no chi lo ra khi doc HAI ham cung luc:
        //
        // `flush_segment_targets` chay HAI giao dich noi tiep — ham nay COMMIT truoc, roi
        // `save_segment_targets` moi chay. Ma phep kiem *"lo phai ghi DU"* (`touched !=
        // expected` ⇒ tu choi TRON) lai nam o giao dich THU HAI. Ban dau ham nay khong co
        // phep kiem tuong ung: cau `UPDATE` chi ap dieu kien cho TUNG hang, nen mot id la
        // don gian cho `changes = 0` va giao dich VAN commit.
        //
        // Duong hong do duoc: mot lo mang [ (id hop le, DA KY, van ban vua doi), (id la) ].
        // Giao dich 1 ha id hop le ve 'draft' va commit — chu ky that MAT. Giao dich 2 thay
        // id la, tu choi tron lo, rollback — van ban moi KHONG duoc ghi. Nguoi dung nhan mot
        // loi *"lo bi tu choi"*, tuong khong co gi doi, trong khi mot cau da am tham mat
        // trang thai 'confirmed'. Dung lop *"mot ket qua sai trong nhu binh thuong"* ma
        // `project-context.md` §Critical Don't-Miss Rules dinh nghia.
        //
        // ⇒ Phep kiem chuyen len TRUOC luot ha. Hai giao dich van la hai giao dich va thu tu
        //   ha-truoc-ghi-sau van nguyen (ly le day du o doc-comment tren) — cai doi la lo
        //   HONG bay gio chet o giao dich MOT, khi chua mot hang nao bi cham.
        //
        // ⚠️ Khong dung `changes()` cua chinh cau `UPDATE` lam phep dem duoc: cau do con mang
        //    `AND status = ?` va `AND target_text <> ?`, nen `changes = 0` la ca THUONG NHAT
        //    (cau chua ky, hoac van ban khong doi) chu khong phai dau hieu id la.
        let mut exists = tx.prepare_cached(
            "SELECT COUNT(*) FROM segment WHERE id = ?1 AND chapter_id = ?2 AND retired_at IS NULL",
        )?;
        let mut present = 0usize;
        for (id, _) in &payload {
            let rows: i64 = exists.query_row((id, chapter_id), |row| row.get(0))?;
            present += usize::try_from(rows).unwrap_or(0);
        }
        if present != expected {
            set_reject(BatchReject::UnknownIds(expected - present));
            return Err(SqlError::QueryReturnedNoRows);
        }

        // ③ HA. `prepare_cached` MOT LAN cho ca lo — cung ly do do duoc o `insert_segments`.
        let mut stmt = tx.prepare_cached(
            "UPDATE segment SET status = ?1 \
             WHERE id = ?2 AND chapter_id = ?3 AND status = ?4 AND target_text <> ?5",
        )?;
        let mut n = 0usize;
        for (id, text) in &payload {
            n += stmt.execute((
                SEGMENT_STATUS_DRAFT,
                id,
                chapter_id,
                SEGMENT_STATUS_CONFIRMED,
                text,
            ))?;
        }
        Ok(n)
    });

    match outcome {
        Ok(touched) => Ok(touched),
        Err(err) => {
            let taken = reject
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            match taken {
                Some(BatchReject::ChapterNotFound) => Err(chapter_not_found(chapter_id)),
                Some(BatchReject::UnknownIds(missing)) => {
                    Err(unknown_segment_ids(chapter_id, missing))
                }
                // O rong ⇒ day la mot loi KHO that, khong mot phep tu choi nghiep vu.
                None => Err(err.into()),
            }
        }
    }
}
