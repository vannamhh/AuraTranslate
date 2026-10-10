use super::*;

/// Một mục của lô ghi bản dịch — Story 2.3, AC13.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
/// Trường đi trên dây đúng tên này: `id` · `target_text`.
///
/// 🔴 Khoá theo **`segment.id`**, KHÔNG theo `ord`. Story 2.8 sắp lại `ord` mà giữ nguyên
/// `id` (AD-3), nên một lô khoá theo `ord` sẽ ghi bản dịch vào câu khác sau lượt sắp lại —
/// im lặng. Cùng luật `commands/segment.rs` đã ghi cho khoá `v-for` ở [`ChapterSegment`].
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SegmentTargetEdit {
    pub id: i64,
    pub target_text: String,
}

/// Kết quả một lượt flush — thứ đi ra qua dây. Story 2.3, AC13.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SaveOutcome {
    /// Chương vừa nhận lô.
    pub chapter_id: i64,
    /// Số hàng `segment` thật sự được `UPDATE`. **0 là hợp lệ** — một lô rỗng.
    pub saved: usize,
}

/// Một `segment.id` trong lô không thuộc Chương được chỉ ⇒ **từ chối trọn lô**.
pub(super) fn unknown_segment_ids(chapter_id: i64, count: usize) -> IpcError {
    IpcError::new(
        "segment.unknown_ids",
        MessageKey::SegmentUnknownIds,
        BTreeMap::from([
            ("chapter_id".to_owned(), chapter_id.to_string()),
            ("count".to_owned(), count.to_string()),
        ]),
        false,
    )
}

/// Ghi bản dịch cho **một LÔ** segment của một Chương — **hàm thuần, đây là thứ test gọi**.
/// Story 2.3 · FR100 · AD-35 · AC4 · AC12 · AC13 · AC14 · AC16.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - `chapter_id` không có trong Tác phẩm đang mở ⇒ `segment.chapter_not_found`;
/// - một `id` nào trong lô không thuộc Chương đó ⇒ `segment.unknown_ids` (**từ chối trọn lô**);
/// - đường đọc/ghi trượt ⇒ lỗi kho (`store.*`), qua `From<StoreError>`.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 AD-31 HÀNG 1 — AUTO-SAVE **KHÔNG** ĐỔI TRẠNG THÁI VÀ **KHÔNG** TẠO `SegmentVersion`
/// ─────────────────────────────────────────────────────────────────────────────
/// `ARCHITECTURE-SPINE.md#AD-31` nói bằng một hàng bảng: *"Auto-save (FR100) | trạng thái
/// **không đổi** | **không** tạo `SegmentVersion`"*. Hàm này giao mệnh đề đó bằng cách câu
/// `UPDATE` dưới đây chạm **đúng hai cột** — và mệnh đề đó có lưới ở
/// `tests/segment_contract.rs` *(bảy cột kia y nguyên từng byte)*.
///
/// ⚠️ **Một test *"không có `SegmentVersion` nào"* hôm nay là một test XANH RỖNG**, vì cả hai
/// thứ đó chưa tồn tại: cột `segment.status` thuộc **Story 2.5**, bảng `segment_version` thuộc
/// **Story 2.6**. Nên mệnh đề được giao ở đây bằng **doc-comment tại chỗ gọi tên hai story
/// chủ** — cùng khuôn `editorSegments.ts:132-135` đã đặt cho ba nhánh vạch chưa có nguồn.
/// 🔴 Story nào thêm `status` phải đọc lại hàm này: nếu nó thêm `status` vào câu `UPDATE`,
/// nó phá AD-31 hàng 1, và cổng duy nhất đứng đó là ca *"bảy cột kia y nguyên"*.
///
/// ⚠️ Hàm này cũng **không được huỷ** bản gốc-lúc-nạp của segment: FR117 (xuất xứ, Story 2.7)
/// so *"văn bản đích **hiện tại** với bản **lúc nạp segment**"*, **không** dùng cờ dirty.
/// `editorSegments`/`editorPanelState` phía webview giữ vai đó và phải giữ tiếp.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT LÔ, MỘT GIAO DỊCH, `prepare_cached` MỘT LẦN — và đó là một con số, không một gu
/// ─────────────────────────────────────────────────────────────────────────────
/// AD-35 nói flush chạy **mỗi 2 giây**, và một nhịp flush có thể mang **nhiều** segment đã
/// đổi *(gõ xuyên qua ba câu trong 5 giây là chuyện thường)*. Một lệnh **mỗi câu** cho N
/// giao dịch trên writer **duy nhất, nối tiếp** của AD-11, và `Store::write` **chặn** — tức
/// N lượt xếp hàng. Story 2.2 vừa đo trên đúng đường đó: riêng chi phí **parse** đáng
/// **57,15 – 64,19 ms** cho 9.850 hàng (xem [`insert_segments`]).
///
/// Và **không** gửi cả Chương: Chương lớn nhất có thật là **9.850** câu / **48.640** ký tự,
/// nên gửi lại nguyên khối mỗi 2 giây là ghi lại phần lớn là những câu không đổi.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `updated_at` SINH Ở TẦNG SQL, KHÔNG TRUYỀN TỪ RUST — AC14
/// ─────────────────────────────────────────────────────────────────────────────
/// `strftime('%Y-%m-%dT%H:%M:%fZ','now')` ngay trong câu lệnh, cùng khuôn
/// [`insert_segments`]. Truyền từ Rust là mở hai nguồn thời gian cho cùng một bảng, và
/// Story 2.6 (*lịch sử phiên bản*) sẽ so hai mốc sinh ra từ hai đồng hồ.
///
/// ⚠️ Phép kiểm *"mọi id thuộc Chương này"* nằm **TRONG** giao dịch ghi, không ở một lượt
/// `Store::read` tách rời — khác [`split_chapter_into_segments`], và khác có chủ ý. Lý do
/// chính là đường hỏng mà doc-comment của hàm đó ghi: giữa một lượt `read` và một lượt
/// `write` tầng kho **không giữ gì cả**. `split` phải dựa vào `MutexGuard` của `wire` vì nó
/// cần chạy một phép tách CPU ở giữa; hàm này không cần, nên nó đi đường chặt hơn — đúng
/// thứ `split_chapter_into_segments` đã ghi là *"đường đúng nếu về sau cần"*.
pub fn save_segment_targets(
    open: Option<&OpenWork>,
    chapter_id: i64,
    edits: &[SegmentTargetEdit],
) -> Result<SaveOutcome, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    // Lo RONG khong phai mot loi: nhip flush co the bat gap mot luot khong con gi de ghi.
    // Tra ve som de KHONG mo mot giao dich rong tren writer noi tiep cua AD-11.
    if edits.is_empty() {
        return Ok(SaveOutcome {
            chapter_id,
            saved: 0,
        });
    }

    // Chi mang du lieu di vao closure ghi — `edits` la `&[..]`, khong `move` duoc.
    let payload: Vec<(i64, String)> = edits
        .iter()
        .map(|e| (e.id, e.target_text.clone()))
        .collect();
    let expected = payload.len();

    // 🔴 O bao ly do TU CHOI ra khoi closure, va no ton tai vi mot ly do dinh luong.
    //
    // `Store::write` doi `SqlResult<T>`: `Ok` ⇒ commit, `Err` ⇒ rollback. Mot phep tu choi
    // nghiep vu PHAI la `Err` — neu khong, lo ghi mot phan duoc commit. Nhung `Err` di ra
    // duoi dang `StoreError::WriteFailed { detail: String }`, va `WriteFailed` cung phu
    // MOI loi SQL that (dia day, kho hong). Doan lai ly do tu `detail` bang chuoi la mot
    // chan doan SAI cho hai ca khac han nhau.
    //
    // ⇒ ly do di ra bang mot o CO KIEU. `Arc<Mutex<..>>` vi closure phai `Send + 'static`.
    let reject: Arc<Mutex<Option<BatchReject>>> = Arc::new(Mutex::new(None));
    let reject_in = Arc::clone(&reject);

    let outcome = open.store.write(move |tx: &Transaction<'_>| {
        let set_reject = |r: BatchReject| {
            *reject_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(r);
        };

        // ① Chuong co thuoc `project.db` cua Tac pham dang mo khong — TRONG cung giao dich
        //    voi luot ghi, nen khong co khe ho nao giua phep kiem va phep ghi.
        let chapter_rows: i64 = tx.query_row(
            "SELECT COUNT(*) FROM chapter WHERE id = ?1",
            [chapter_id],
            |row| row.get(0),
        )?;
        if chapter_rows == 0 {
            set_reject(BatchReject::ChapterNotFound);
            return Err(SqlError::QueryReturnedNoRows);
        }

        // ② `prepare_cached` MOT LAN cho ca lo — xem khoi doc-comment o tren.
        //    Cau `UPDATE` cham DUNG HAI COT: `target_text` va `updated_at`.
        let mut stmt = tx.prepare_cached(
            "UPDATE segment \
             SET target_text = ?1, \
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') \
             WHERE id = ?2 AND chapter_id = ?3 AND retired_at IS NULL",
        )?;
        let mut touched = 0usize;
        for (id, text) in &payload {
            // `AND chapter_id = ?3` la nua thu hai cua phep kiem id: mot id thuoc Chuong
            // KHAC cho `changes = 0` thay vi ghi vao Chuong do.
            touched += stmt.execute((text, id, chapter_id))?;
        }

        // ③ Lo phai ghi DU. Thieu mot hang ⇒ co id khong thuoc Chuong nay ⇒ TU CHOI TRON,
        //    va `Err` o day lam ca giao dich ROLLBACK. Khong co lo nao ghi mot phan.
        if touched != expected {
            set_reject(BatchReject::UnknownIds(expected - touched));
            return Err(SqlError::QueryReturnedNoRows);
        }

        Ok(touched)
    });

    match outcome {
        Ok(saved) => Ok(SaveOutcome { chapter_id, saved }),
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

/// Lý do một lô ghi bị **từ chối trọn**, mang ra khỏi closure ghi. Xem [`save_segment_targets`].
pub(super) enum BatchReject {
    /// `chapter_id` không có trong `project.db` của Tác phẩm đang mở.
    ChapterNotFound,
    /// Số `segment.id` trong lô không thuộc Chương đó.
    UnknownIds(usize),
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 2.5 — MÁY TRẠNG THÁI SEGMENT (AD-31), và nó sống Ở ĐÂY chứ không ở TypeScript
//
// 🔴 AD-1, và ngoại lệ **duy nhất, tường minh** của nó là **văn bản đang gõ** — không phải
// trạng thái. Vue chỉ render vạch lề; không một quy tắc nào của bảng AD-31 được cài lại ở
// TypeScript (AC12). Bất biến #4 của story là *"vi phạm được mà không cổng nào đỏ"*.
// ═════════════════════════════════════════════════════════════════════════════════

/// Kết quả một lượt PROMOTE — thứ đi ra qua dây. Story 4.8 (FR72, AD-47①/③); `force`/
/// `needs_confirmation` theo khuôn AD-49 iii.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PromoteAiTranslationOutcome {
    /// Segment vừa được ghi.
    pub segment_id: i64,
    /// Văn bản vừa ghi — webview mirror lại bằng `replaceEditorSegment` (§Code Map spec 4.8:
    /// "that mirror is not cosmetic — the confirm baseline is read from the loaded snapshot").
    /// Khi `needs_confirmation`, đây là văn bản HIỆN CÓ trên đĩa (không đổi), không phải đề
    /// xuất AI sắp ghi đè.
    pub target_text: String,
    /// Xuất xứ SAU lượt gọi. Khi `needs_confirmation`, đây là xuất xứ hiện có (không đổi).
    pub translation_origin: String,
    /// Status after the call: `draft` when new text was written, unchanged when `needs_confirmation`.
    pub status: String,
    /// 🔴 **Lượt ghi bị GIỮ LẠI vì nó sắp xoá vĩnh viễn một bản nháp chưa từng được ký** — cùng
    /// khuôn [`RestoreOutcome::needs_confirmation`] (FR101). Khi `true`,
    /// **không một byte nào được ghi**; webview hỏi lại người dùng rồi gọi lại với `force = true`.
    pub needs_confirmation: bool,
    /// Bản nháp sắp bị ghi đè. `Some` khi và chỉ khi `needs_confirmation`.
    pub unsigned_draft: Option<String>,
}

/// The single non-user write of `target_text` (AD-50 rule 3): text, both baseline columns,
/// `translation_origin` and `status` in one statement. `translation_origin = None` leaves the
/// stored origin untouched (restore, AD-47 ⑤); the baseline origin is always set.
pub(super) fn write_non_user_target(
    tx: &Transaction<'_>,
    segment_id: i64,
    target_text: &str,
    baseline_translation_origin: &str,
    translation_origin: Option<&str>,
) -> SqlResult<()> {
    tx.execute(
        "UPDATE segment SET target_text = ?1, baseline_target_text = ?1, \
         baseline_translation_origin = ?2, translation_origin = COALESCE(?3, translation_origin), \
         status = ?4, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?5",
        (
            target_text,
            baseline_translation_origin,
            translation_origin,
            SEGMENT_STATUS_DRAFT,
            segment_id,
        ),
    )?;
    Ok(())
}

/// Ghi một kết quả AI vào `target_text` **và** đặt `translation_origin = TRANSLATION_ORIGIN_OTHER`
/// trong **MỘT** câu `UPDATE` — Story 4.8, AD-47①. Đây là chỗ ghi RIÊNG của đường promote
/// (§Code Map spec 4.8: "the promote path therefore needs its own write, not a reuse of
/// `save_segment_targets`") — ba writer `target_text` đã có (`save_segment_targets`,
/// `flush_segment_targets`, `restore_segment_version`) đều cố ý để nguyên `translation_origin`,
/// vì cả ba đều ghi văn bản NGƯỜI DÙNG gõ hoặc một bản chép CŨ của chính người dùng; dùng lại
/// một trong ba cho một văn bản MÔ HÌNH sinh ra sẽ đóng dấu tên người dịch lên câu của AI.
///
/// 🔴 **Đây là một lượt ghi non-user, dưới AD-47①** — nó KHÔNG đọc mốc để phân xử
/// `self`/giữ nguyên như [`confirm_segment`] làm: mốc so của FR117 không áp dụng ở đây, vì lượt
/// này không phải một lượt XÁC NHẬN, nó là lượt THAY THẾ nội dung bằng đề xuất của AI — AD-47③
/// đã CHỐT sẵn giá trị xuất xứ cho đúng cơ chế này (*"Đưa đề xuất AI sang Editor"* → **người
/// khác dịch**), không có nhánh thứ hai để mà phân xử.
///
/// Mirrors `restore_segment_version`'s unsigned-draft guard (AD-49 iii): an unconditional write
/// here would destroy a draft with no copy in `segment_version`; the caller must flush first.
///
/// # Lỗi
/// - chưa có Tác phẩm nào đang mở ⇒ `work.none_open`;
/// - `segment_id` không có trong `project.db` của Tác phẩm đang mở ⇒ `segment.not_found`.
pub fn promote_ai_translation(
    open: Option<&OpenWork>,
    segment_id: i64,
    target_text: &str,
    force: bool,
) -> Result<PromoteAiTranslationOutcome, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    enum Promoted {
        Missing,
        Retired,
        OtherChapter,
        Row(String, String, String, bool),
    }

    let open_chapter_id = open.chapter_id;
    let payload = target_text.to_owned();
    let outcome = open.store.write(move |tx: &Transaction<'_>| {
        let found = tx.query_row(
            "SELECT target_text, translation_origin, retired_at IS NOT NULL, status, chapter_id \
             FROM segment WHERE id = ?1",
            [segment_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, bool>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        );
        let (current_text, current_origin, retired, current_status, segment_chapter_id) = match found {
            Ok(value) => value,
            Err(SqlError::QueryReturnedNoRows) => return Ok(Promoted::Missing),
            Err(err) => return Err(err),
        };
        if retired {
            return Ok(Promoted::Retired);
        }
        if segment_chapter_id != open_chapter_id {
            return Ok(Promoted::OtherChapter);
        }

        // Cùng phép so VĂN BẢN của `restore_segment_version` — "chưa từng được ký" nghĩa là
        // KHÔNG có bản sao trong `segment_version`, không một cờ `dirty`. Cùng miễn trừ
        // `!current_text.is_empty()`: một `target_text` RỖNG không có gì để mất.
        if !force && !current_text.is_empty() {
            let has_copy: i64 = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM segment_version \
                 WHERE segment_id = ?1 AND target_text = ?2)",
                (segment_id, &current_text),
                |row| row.get(0),
            )?;
            if has_copy == 0 {
                // KHONG ghi mot byte nao. Day KHONG phai mot loi -- xem `needs_confirmation`.
                return Ok(Promoted::Row(current_text, current_origin, current_status, true));
            }
        }

        write_non_user_target(
            tx,
            segment_id,
            &payload,
            TRANSLATION_ORIGIN_OTHER,
            Some(TRANSLATION_ORIGIN_OTHER),
        )?;

        Ok(Promoted::Row(
            payload,
            TRANSLATION_ORIGIN_OTHER.to_owned(),
            SEGMENT_STATUS_DRAFT.to_owned(),
            false,
        ))
    })?;

    let (target_text, translation_origin, status, needs_confirmation) = match outcome {
        Promoted::Missing => return Err(segment_not_found(segment_id)),
        Promoted::Retired => return Err(segment_retired(segment_id)),
        Promoted::OtherChapter => {
            return Err(segment_not_in_open_chapter(segment_id, open_chapter_id));
        }
        Promoted::Row(text, pair_origin, status, needs_confirmation) => {
            (text, pair_origin, status, needs_confirmation)
        }
    };
    // `target_text` already holds the draft when `needs_confirmation`: that branch of the
    // write closure above returns `current_text` unchanged (see the struct field doc).
    let unsigned_draft = needs_confirmation.then(|| target_text.clone());

    Ok(PromoteAiTranslationOutcome {
        segment_id,
        target_text,
        translation_origin,
        status,
        needs_confirmation,
        unsigned_draft,
    })
}
