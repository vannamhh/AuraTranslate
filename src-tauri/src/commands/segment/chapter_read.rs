use super::*;
use crate::core::segment::omit::IN_TRANSLATION_SQL;

/// Một hàng `segment` đi ra qua dây — Story 2.2, AC13.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// TỪNG TRƯỜNG, VÀ AI ĐỌC NÓ Ở PHÍA WEBVIEW
/// ─────────────────────────────────────────────────────────────────────────────
/// - `id` — khoá của mọi thứ Epic 2 gắn vào một câu (`SegmentVersion` của 2.6 theo AD-5).
///   Cũng là khoá `v-for` của trang liền mạch; `ord` KHÔNG dùng được cho vai đó vì Story
///   2.8 sắp lại `ord` mà giữ nguyên `id` (AD-3).
/// - `ord` — thứ tự đọc, đánh số **từ 1**, liên tục.
/// - `source_text` — nguyên văn của câu. Editor của Story 2.2 **không** hiển thị nó *(panel
///   là "Bản dịch")*; nó đi cùng vì Story 2.3 so `target_text` với nguồn, và vì bàn đo cần
///   một chỗ đọc ra được câu nào ứng với vạch nào.
/// - `target_text` — bản dịch. **Chuỗi rỗng nghĩa là "chưa dịch"**, không phải một giá trị
///   vắng mặt (xem `SEGMENT_TARGET_TEXT_DDL`). Đây là nguồn của nhánh *"không vạch"* trong
///   năm giá trị vạch lề (AC3).
/// - `is_paragraph_end` — cờ kết đoạn, **đã lưu** lúc nhập. AD-37 cấm suy ra lúc render, nên
///   nó đi qua dây chứ không tính lại ở TypeScript.
/// - `retired_at` — `None` cho mọi segment hôm nay: **chưa đường nào cho segment về hưu**
///   (Story 2.8 mang nó). Trường có mặt vì nó là nguồn dữ liệu của giá trị vạch `ornament`,
///   và bảng ánh xạ *trạng thái → vạch* của Story 2.2 cài **cả năm** nhánh (Quyết định #4).
/// - `status` — máy trạng thái AD-31, Story 2.5. `'draft'` | `'confirmed'`. Đây là nguồn dữ
///   liệu của giá trị vạch `confirmed`.
///
/// 🔴 **VÌ SAO `status` PHẢI CÓ MẶT Ở ĐÂY, ghi ra vì nó đã HỎNG một lần** — bắt được bằng
/// `e2e/specs/editor-confirm-segment.e2e.mjs` ngày 2026-08-14, sau khi cả bốn đường kia đã xanh.
/// Bản đầu của Story 2.5 thêm `status` vào **kiểu TypeScript** (`config/segment.ts`) và cho
/// `segmentRuleInputOf` đọc nó, nhưng **quên** thêm vào struct này và vào câu `SELECT` của
/// [`read_open_chapter_segments`]. Hệ quả: Rust không bao giờ gửi trường đó ⇒ `segment.status`
/// là `undefined` trong webview ⇒ `isConfirmed` **luôn `false`** ⇒ vạch `confirmed` không bao
/// giờ hiện, **trên sản phẩm thật**.
/// ⚠️ Và nó đi lọt **74/74 test frontend**, vì fixture vitest tự dựng `ChapterSegment` bằng tay
/// và **có** cấp `status`. Một fixture chép tay là một fixture trôi được khỏi sự thật của dây.
/// ⇒ Lưới nay là `segment_contract.rs::the_load_command_carries_the_status_column_over_the_wire`.
///
/// - `is_omitted` — cờ **cắt bỏ câu khỏi bản dịch** (FR133), Story 2.5c. Bước di trú 8.
///   🔴 Một **TRỤC ĐỘC LẬP**, không phải giá trị thứ ba của `status` (AC2): *"cắt bỏ"* nói
///   câu **thuộc hay không thuộc bản dịch**, `status` nói **mức độ hoàn thành**. Một câu đã
///   cắt bỏ giữ nguyên cả `status` lẫn `target_text` — và đó là thứ làm AC4 (*"bỏ cờ ⇒ câu
///   quay về đúng trạng thái cũ với nội dung cũ"*) đúng **mà không một dòng mã khôi phục
///   nào**: không gì bị mất thì không gì phải khôi phục.
///   ⚠️ Cột này đi vào đây **cùng lượt** với bước di trú sinh ra nó, đúng vì vụ `status` ở
///   trên — lưới riêng của nó là
///   `segment_contract.rs::the_load_command_carries_the_is_omitted_column_over_the_wire`.
///
/// - `is_target_paragraph_end` — cờ **kết đoạn của BẢN DỊCH** (FR134/AD-46), Story 2.5d.
///   Bước di trú 9.
///   🔴 **Một cờ THỨ HAI, không phải một cách đọc khác của cờ nguồn** (AC4): nhịp của tiếng
///   Việt không buộc phải là nhịp của bản gốc. Lúc nhập, nó **bằng** cờ nguồn (AC2); từ đó
///   hai cờ sống độc lập và người dùng đổi cờ đích bằng lệnh riêng.
///   🔴 **AC4 nói thẳng: đường mã nào cần cấu trúc đoạn của bản dịch thì ĐỌC CỘT NÀY, không
///   suy ra từ nội dung nguồn.** Đó là lý do nó phải đi qua dây thay vì để webview tự tính
///   — một phép suy ở webview là một nguồn sự thật thứ hai, và nó sẽ rẽ khỏi đĩa đúng vào
///   ngày người dùng đổi cờ đầu tiên.
///   ⚠️ Cột thứ **BA** liên tiếp đi vào đây cùng lượt với bước di trú sinh ra nó, đúng vì vụ
///   `status` ở trên. Lưới riêng:
///   `segment_contract.rs::the_load_command_carries_the_target_paragraph_end_column_over_the_wire`.
///
/// - `role` — vai của segment (`alt` | `caption` | `null`), Story 6.13, AD-42. Bước di trú
///   21. `null` cho tuyệt đại đa số segment (văn xuôi thường) — KHÔNG dịch thành `false`/chuỗi
///   rỗng: đây là "không có vai", khác nghĩa hẳn một vai chưa xác định. Bản chép tay TypeScript
///   (`src/config/segment.ts`) chở trường này CÙNG một lượt — không codegen nào giữ đồng bộ
///   hộ hai tệp.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ChapterSegment {
    pub id: i64,
    pub ord: i64,
    pub source_text: String,
    pub target_text: String,
    pub is_paragraph_end: bool,
    pub retired_at: Option<String>,
    pub status: String,
    pub is_omitted: bool,
    pub is_target_paragraph_end: bool,
    pub role: Option<String>,
    /// Origin of [`Self::target_text`]: one of [`TRANSLATION_ORIGINS`], `""` when none. Display
    /// only; never sent back to Rust (AD-50 rule 5).
    pub translation_origin: String,
}

/// Trọn bộ segment của Chương **đang mở** — thứ đi ra qua dây.
///
/// ⚠️ `chapter_id` đi kèm chứ không để chỗ gọi tự đoán: webview cần nó để gắn mọi lượt ghi
/// của Story 2.3 vào đúng Chương, và một lượt hỏi lại qua `read_open_chapter` sẽ kéo theo
/// **nguyên khối** `source_text` của cả Chương *(đo được: Chương lớn nhất có thật là 48.640
/// ký tự)* chỉ để lấy một số nguyên.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChapterSegments {
    pub chapter_id: i64,
    pub segments: Vec<ChapterSegment>,
    /// **THÊM Story 5.7 (AC4/AC5)** — `segment.id` nơi caret của Editor phải đứng khi
    /// Chương vừa nạp, đọc từ bảng `chapter_position`. `None` chỉ khi Chương **không có
    /// segment nào** — mọi Chương có ít nhất một segment sống LUÔN có một giá trị ở đây,
    /// kể cả khi chưa từng mở (rơi về segment ĐẦU theo `(ord, id)`, AC5) hay khi hàng vị
    /// trí trỏ vào một segment đã VỀ HƯU (rơi về segment đầu, kèm chẩn đoán — xem
    /// [`read_open_chapter_segments`]).
    ///
    /// 🔴 Rust QUYẾT giá trị này, webview KHÔNG suy ra (§Always của story): đường
    /// `editorCaretPlacement` đã có (`GridPanel.vue:1110`) chỉ ĐẶT caret vào đúng
    /// `segment.id` mà trường này nói, không tự tính "segment đầu" một lần nữa.
    pub caret_segment_id: Option<i64>,
    /// **THÊM Story 6.14 (FR42/FR43)** — ảnh của Chương này, đã phân giải neo qua
    /// [`crate::core::segment::image::resolve_chapter_images`] với quy tắc "đủ điều kiện làm
    /// neo" của LƯỚI (`include_omitted = true, exclude_roles = false` — lưới không lọc cắt
    /// bỏ, và một hàng `alt`/`caption` vẫn là một điểm neo hợp lệ). Rỗng cho MỌI Chương không
    /// có hàng `asset` nào — Chương 0 ảnh render trùng đúng byte trước story này (§I/O Matrix).
    pub assets: Vec<ChapterAsset>,
    /// **THÊM Story 6.14** — đường dẫn TUYỆT ĐỐI tới `<dir>/assets` của Tác phẩm đang mở, MỘT
    /// lần cho cả Chương (không lặp lại ở từng phần tử của [`Self::assets`]). Webview ghép
    /// `assets_dir + '/' + file_name` rồi đưa qua `convertFileSrc` — tiền lệ DUY NHẤT
    /// `src/tokens/fonts.ts:136`.
    pub assets_dir: String,
    /// Segments this load pre-filled from an exact TM match (FR58). Only
    /// [`load_open_chapter_segments`] fills it; [`read_open_chapter_segments`] leaves it empty.
    pub tm_filled_segment_ids: Vec<i64>,
    pub tm_prefill: TmPrefillStatus,
}

/// Outcome of the TM pre-fill (FR58) on a Chapter load. `NotAsked` means no fill was attempted;
/// it never reads as "ran, 0 hits".
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind")]
pub enum TmPrefillStatus {
    #[serde(rename = "ran")]
    Ran,
    #[serde(rename = "not_asked")]
    NotAsked,
    #[serde(rename = "skipped")]
    Skipped { code: String },
}

/// Một ảnh đã phân giải vị trí, ra dây cho LƯỚI — Story 6.14, FR42/FR43.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ChapterAsset {
    pub asset_id: i64,
    /// Tên tệp TƯƠNG ĐỐI trong `assets/` (`<uuid>.jpg|png|gif|webp`) — ghép với
    /// [`ChapterSegments::assets_dir`] ở webview.
    pub file_name: String,
    /// `NULL` được — ảnh nhúng trực tiếp trong `.docx` không có URL nguồn (Story 6.12).
    pub source_url: Option<String>,
    /// `id` của segment đứng NGAY TRƯỚC ảnh trong ô nguyên văn của nó — `None` ⇒ đầu ô của
    /// câu ĐẦU TIÊN của Chương (neo `0`).
    pub after_segment_id: Option<i64>,
    /// `target_text` của segment `role = 'alt'` đứng ngay sau neo — `None` ⇒ ảnh trang trí,
    /// webview đặt `alt=""` (không bịa chữ).
    pub alt_text: Option<String>,
    /// `target_text` của segment `role = 'caption'` đứng ngay sau neo — `None`, hoặc chuỗi
    /// RỖNG (chưa dịch), đều nghĩa là "không khối chú thích" ở tầng hiển thị.
    pub caption_text: Option<String>,
}

impl From<crate::core::segment::image::ResolvedImage> for ChapterAsset {
    fn from(r: crate::core::segment::image::ResolvedImage) -> Self {
        ChapterAsset {
            asset_id: r.asset_id,
            file_name: r.file_name,
            source_url: r.source_url,
            after_segment_id: r.after_segment_id,
            alt_text: r.alt_text,
            caption_text: r.caption_text,
        }
    }
}

/// `assets_dir` is not valid UTF-8 — a distinguishable error instead of letting
/// `to_string_lossy()` silently replace the offending bytes with U+FFFD.
pub(super) fn assets_dir_not_utf8() -> IpcError {
    IpcError::new(
        "segment.assets_dir_not_utf8",
        MessageKey::SegmentAssetsDirNotUtf8,
        BTreeMap::new(),
        false,
    )
}

/// Chương không có trong `project.db` của Tác phẩm đang mở.
pub(super) fn chapter_not_found(chapter_id: i64) -> IpcError {
    IpcError::new(
        "segment.chapter_not_found",
        MessageKey::SegmentChapterNotFound,
        BTreeMap::from([("chapter_id".to_owned(), chapter_id.to_string())]),
        false,
    )
}

/// Chương đã có segment ⇒ **từ chối**, không ghi đè. Xem doc-comment đầu module.
pub(super) fn already_split(chapter_id: i64, count: i64) -> IpcError {
    IpcError::new(
        "segment.already_split",
        MessageKey::SegmentAlreadySplit,
        BTreeMap::from([
            ("chapter_id".to_owned(), chapter_id.to_string()),
            ("count".to_owned(), count.to_string()),
        ]),
        false,
    )
}

/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 THÊM Story 5.12 (Task 1) — BẢN DUY NHẤT CỦA CÂU `SELECT` CHÍN CỘT
/// ─────────────────────────────────────────────────────────────────────────────
/// Trước lượt này có **hai** bản chép của chính câu `SELECT` dưới đây — một ở
/// `read_open_chapter_segments`, một ở `read_reading_chapter` (Story 5.11) — và Story 5.11 tự
/// ghi món nợ 🟡 cho chuyện đó: *"sửa bộ lọc ở một nơi làm Chế độ đọc và Workspace nói khác
/// nhau về cùng một Chương"*. `read_reading_run` (Story 5.12) cần gọi câu này **mỗi Chương
/// trong cả một dãy** — để nguyên hai bản chép thì thành ba, càng khó giữ khớp. Đây là hàm
/// **DUY NHẤT** phát ra câu `SELECT` đó; mọi chỗ cần segment còn sống của một Chương gọi lại
/// nó, không viết tay một câu `SELECT` thứ hai.
///
/// ⚠️ Nhận [`ReadHandle`] (tái xuất bởi `core::store`, KHÔNG `rusqlite::Connection` trực
/// tiếp — AD-11, `store_boundary.rs::only_core_store_may_name_rusqlite`), không `&OpenWork`:
/// chỗ gọi phải nằm sẵn TRONG một closure của
/// [`crate::core::store::Store::read`] — hàm này không tự mở lượt đọc riêng, đúng lý lẽ mọi
/// lượt đọc của cùng một ảnh chụp phải chung một `Store::read` (xem doc-comment
/// `read_reading_run`/`caret_segment_id` ngay dưới).
pub(crate) fn select_chapter_segments(conn: ReadHandle<'_>, chapter_id: i64) -> SqlResult<Vec<ChapterSegment>> {
    let mut stmt = conn.prepare(
        "SELECT id, ord, source_text, target_text, is_paragraph_end, retired_at, status, \
         is_omitted, is_target_paragraph_end, role, translation_origin \
         FROM segment WHERE chapter_id = ?1 AND retired_at IS NULL ORDER BY ord, id",
    )?;
    let rows = stmt.query_map([chapter_id], |row| {
        // ⚠️ `is_paragraph_end` la INTEGER 0/1 duoi SQLite (khong co kieu boolean); phep
        // doi sang `bool` la viec cua tang nay, dung khuon `chapter.status`.
        let flag: i64 = row.get(4)?;
        // ⚠️ `is_omitted` cung la INTEGER 0/1 -- cung phep doi, cung ly do (Story 2.5c).
        let omitted: i64 = row.get(7)?;
        // ⚠️ `is_target_paragraph_end` cung la INTEGER 0/1 (Story 2.5d, buoc 9).
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
            role: row.get(9)?,
            translation_origin: row.get(10)?,
        })
    })?;
    rows.collect::<SqlResult<Vec<ChapterSegment>>>()
}

/// **THÊM Story 6.14** — câu `SELECT` DUY NHẤT cho bảng `asset`, khuôn theo
/// [`select_chapter_segments`] ngay trên: mọi chỗ cần hàng `asset` thô của một Chương gọi lại
/// hàm này, không viết tay một câu `SELECT` thứ hai (cùng lý do `select_chapter_segments` đã
/// ghi — `read_reading_run` cần gọi nó MỖI Chương trong cả một dãy).
///
/// `ORDER BY id` — không phải `anchor_after_segment_ord`: thứ tự HIỂN THỊ (hai ảnh cùng neo)
/// là việc của [`crate::core::segment::image::resolve_chapter_images`], hàm này chỉ đọc thô.
pub(super) fn select_chapter_assets(
    conn: ReadHandle<'_>,
    chapter_id: i64,
) -> SqlResult<Vec<crate::core::segment::image::RawAsset>> {
    let mut stmt = conn.prepare(
        "SELECT id, file_name, source_url, anchor_after_segment_ord \
         FROM asset WHERE chapter_id = ?1 ORDER BY id",
    )?;
    let rows = stmt.query_map([chapter_id], |row| {
        Ok(crate::core::segment::image::RawAsset {
            id: row.get(0)?,
            file_name: row.get(1)?,
            source_url: row.get(2)?,
            anchor_after_segment_ord: row.get(3)?,
        })
    })?;
    rows.collect::<SqlResult<Vec<_>>>()
}

/// Nạp trọn bộ segment của Chương **đang mở** — **hàm thuần, đây là thứ test gọi**.
/// Story 2.2, AC13.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - Tác phẩm đang mở không có hàng `chapter` nào ⇒ `store.read_failed` (qua
///   `From<StoreError>`) — cùng đường và cùng lý do với
///   [`crate::commands::chapter::read_open_chapter`].
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO LỆNH NÀY KHÔNG NHẬN `chapter_id`, TRONG KHI LỆNH TÁCH THÌ CÓ
/// ─────────────────────────────────────────────────────────────────────────────
/// Epic 1 tạo **đúng một** Chương cho mỗi Tác phẩm (`commands::project::create_work`), và
/// chọn Chương / chuyển Chương là **Story 2.11**. Một tham số `chapter_id` hôm nay chỉ có
/// **một** giá trị hợp lệ, và cách duy nhất webview biết giá trị đó là gọi
/// [`crate::commands::chapter::read_open_chapter`] trước — tức một lượt IPC thứ hai kéo
/// theo nguyên khối `source_text` của cả Chương chỉ để lấy một số nguyên.
///
/// Lệnh **tách** thì khác và tham số của nó có thật: nó chạy trên một Chương **cũ** mà
/// người dùng chỉ đích danh trong Thư viện, và `deferred-work.md §*Deferred from: 1-12-matcher-dung-chung (2026-08-05)*` đếm 25 Chương như
/// vậy.
///
/// ⚠️ ~~Story 2.11 sở hữu biến thể nhận `chapter_id`. **Đừng** thêm sẵn một tham số
/// `Option<i64>` hôm nay: một nhánh không chỗ gọi nào đi qua là một nhánh không ai nghiệm
/// thu được — cùng luật đã ghi cho danh mục `MessageKey` (`core::i18n`).~~
///
/// 🔵 **SỬA 2026-08-18 — Story 2.11 ĐÃ CHẠY, và nó KHÔNG thêm tham số nào vào lệnh này.**
/// Ice ký Quyết định #2 đường (a) và #3 đường (a): *"Chương đang mở"* sống thành một trường
/// trên [`crate::commands::project::OpenWork`], và Chương kề do **Rust** quyết qua một lệnh
/// riêng ([`crate::commands::chapter::open_adjacent_chapter`]) — webview chỉ nói **hướng**.
/// ⇒ Lệnh này vẫn **không** nhận `chapter_id`, nhưng lý do nay khác hẳn: không phải *"chỉ có
/// một giá trị hợp lệ"* mà là *"giá trị ấy là **quy tắc nghiệp vụ** và nó ở lại Rust"* (AD-1).
/// Đường (c) — thêm `chapter_id: Option<i64>` vào hai lệnh đọc — đã được trình cho Ice và
/// **bị loại**, đúng theo lời cấm nguyên bản ở trên.
///
/// ⚠️ Ba dòng ngay trên đoạn gạch ngang *(Epic 1 tạo đúng một Chương)* **vẫn đúng về dữ
/// liệu** — không đường sản phẩm nào sinh Chương thứ hai hôm nay, món nợ có chủ là Epic 6 —
/// nhưng nó **không còn** là tiền đề của mã: `read_open_chapter_segments` nay đọc
/// `OpenWork::chapter_id`, không `ORDER BY ord LIMIT 1`.
///
/// ⚠️ `ORDER BY ord` là **có chủ đích**, không phải trang trí: `idx_segment_chapter_ord`
/// (`chapter_id, ord`) thành covering cho đúng lượt đọc này, nên SQLite khỏi một lượt sắp
/// tạm trên **9.850** hàng của Chương lớn nhất có thật.
///
/// 🔵 **2026-08-17, code review — thêm khoá phụ `, id`.** `ord` **cố ý không `UNIQUE`**
/// (`schema.rs:279-282`, để hở tạm trong một giao dịch nhiều bước), nên `ORDER BY ord` trần
/// **không** là một thứ tự toàn phần. Hai truy vấn khác của chính story này đã mang khoá phụ
/// — đánh lại số (`ORDER BY ord, id`) và tìm câu liền trên (`ORDER BY ord DESC, id DESC`) —
/// và chú thích ở lượt tìm câu liền trên viết thẳng vì sao: giả định `ord` liên tục *"sẽ làm
/// một phép trừ im lặng trỏ sai hàng"*. Lập luận ấy đúng y nguyên cho lượt đọc này, và bỏ
/// sót nó nghĩa là *"câu liền trên"* mà Rust gộp **không bảo đảm** là hàng người dùng nhìn
/// thấy ở trên — hai truy vấn chỉ trùng nhau nhờ hành vi phá hoà tình cờ của SQLite.
///
/// ⚠️ **Chưa đường mã nào sinh ra `ord` trùng hôm nay** *(`insert_segments` cấp `index + 1`;
/// `write_regroup` đánh lại 1..N liên tục)* — đây là một dòng **phòng thủ**, không một lỗi
/// đang chảy máu. Ghi ra để người sau khỏi đi tìm một biểu hiện không có.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔵 2026-08-17, Story 2.8 — `WHERE retired_at IS NULL`, và nó ĐẾN TỪ MỘT LƯỢT DÙNG THẬT
/// ─────────────────────────────────────────────────────────────────────────────
/// Câu hỏi này chỉ có thật từ story 2.8: trước nó, `retired_at` là `NULL` cho **mọi** segment
/// và không đường mã nào đặt nó, nên *"lọc hay không"* chưa phân biệt được gì.
///
/// **Ice ký hai lần, và lần sau lật lần trước:**
///
/// - Chữ ký #6(b) *(2026-08-17, sáng)* — **giữ** hàng về hưu trong lưới với vạch `ornament`
///   mờ. Ba thứ đứng sau nó: nhánh `ornament` chỉ có một nơi gọi là lưới; UX-DR19
///   khai `ornament` là một trong **sáu** giá trị vạch; AD-5 hứa chỗ đánh
///   dấu FR119 vẫn mở về đúng vị trí. Cái giá đã viết ra và nhận: *"lưới phình theo số lần
///   sửa, vĩnh viễn"*.
/// - 🔴 **Lật, cùng ngày, sau khi Ice DÙNG THẬT:** *"đã tách ra 2 câu, nhưng câu cũ vẫn tồn
///   tại và số thứ tự vẫn chiếm, gây rối nội dung"*. Cái giá kia **đã được viết ra trước khi
///   ký** — nhưng nó chỉ đọc được thành *"gây rối"* khi có người thật nhìn vào một Chương
///   thật. Một bảng đường không thay được một lượt dùng.
///
/// 🔴 **LỌC KHỎI LƯỚI, KHÔNG XOÁ KHỎI ĐĨA.** Hàng vẫn nằm trong `project.db` với `retired_at`
/// khác `NULL`. [`read_segment_history`] **không** hỏi cột đó *(cố ý, `commands/segment.rs`
/// §Đường đọc và đường ghi từ chối khác nhau)*, nên **AC4** — *"segment đã về hưu thì lịch sử
/// phiên bản của nó vẫn tra lại được"* — còn nguyên. Xoá hàng là mất lịch sử **vĩnh viễn**,
/// đúng thứ AD-5 tồn tại để chống.
///
/// ⚠️ **Nửa Rust là nửa BỀN.** `applyRegroup` phía webview cũng gỡ hàng về hưu khỏi ảnh chụp,
/// nhưng đó chỉ là ảnh chụp trong bộ nhớ: thiếu dòng `WHERE` này thì đóng Tác phẩm rồi mở lại
/// là mọi hàng về hưu **quay về** — và không cổng nào đỏ.
///
/// 🔴 **HỆ QUẢ CHƯA ĐÓNG, ghi ra thay vì để người sau tưởng đã xét:** nhánh `'ornament'` của
/// `editorSegments.ts::resolveSegmentRule` nay **không còn đường tới**. Nó **KHÔNG** bị gỡ, và
/// đó là một lựa chọn: `ornament` là một trong sáu giá trị vạch mà UX-DR19 khai, nên gỡ nó là
/// làm mã lệch một UX-DR **đang đứng** — `project-context.md:456-458` cấm sửa spec cho khớp
/// mã. Món nợ có chủ ở `deferred-work.md`.
pub fn read_open_chapter_segments(open: Option<&OpenWork>) -> Result<ChapterSegments, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    // 🔵 SUA 2026-08-18 (Story 2.11 · Quyet dinh #2(a), Ice ky). Ban cu chay
    // `SELECT id FROM chapter ORDER BY ord LIMIT 1` — mot cau SQL THU HAI suy ra Chuong dang
    // mo, doc lap voi cau thu nhat o `commands::chapter::read_open_chapter`. Hai cho suy ra
    // cung mot du kien la hai nguon su that; ngay khi Chuong thu hai ton tai, ca hai tra ve
    // Chuong DAU mai mai va khong cong nao do. Nay ca hai deu HOI `OpenWork::chapter_id`.
    let chapter_id = open.chapter_id;
    // **THÊM Story 6.14** — cấu tạo NGOÀI closure ghi: `assets_dir` không phụ thuộc ảnh chụp
    // đọc, và `open.dir` không sống được bên trong closure `Store::read` (nó mượn `conn`,
    // không `open`). `move` mang `PathBuf` này vào closure như một giá trị đã cấu tạo sẵn.
    //
    // Checked for UTF-8 before entering the closure, so a non-UTF-8 assets_dir errors here.
    let assets_dir = open.dir.join("assets");
    let assets_dir_str = assets_dir.to_str().ok_or_else(assets_dir_not_utf8)?.to_owned();

    let loaded = open.store.read(move |conn| {
        let segments = select_chapter_segments(conn, chapter_id)?;

        // 🔴 **Story 5.7 (AC4/AC5) — TÍNH `caret_segment_id` TRONG CÙNG MỘT LƯỢT `Store::read`
        // NÀY**, không một lượt đọc riêng: `chapter_position` và `segment` phải đến từ CÙNG
        // một ảnh chụp, nếu không một lượt gộp/tách chen giữa hai lần đọc có thể trả một
        // `segment_id` mà `segments` vừa đọc chưa/không còn thấy — đúng lớp lỗi mà AC4 của
        // `Indexer::list_works` (Story 5.6) đã ghi cho một cặp dữ kiện tương tự.
        let position: Option<i64> = {
            let mut stmt =
                conn.prepare("SELECT segment_id FROM chapter_position WHERE chapter_id = ?1")?;
            let mut rows = stmt.query_map([chapter_id], |row| row.get::<_, i64>(0))?;
            rows.next().transpose()?
        };

        let caret_segment_id = match position {
            // Hàng vị trí có mặt VÀ segment đó CÒN SỐNG trong danh sách vừa đọc.
            Some(segment_id) if segments.iter().any(|s| s.id == segment_id) => Some(segment_id),
            // Hàng vị trí có mặt nhưng segment đó đã VỀ HƯU (gộp/tách, AD-5) — rơi về segment
            // đầu CÓ LÝ DO, không im lặng (§I/O Matrix "Vị trí trỏ vào segment ĐÃ VỀ HƯU").
            // ⚠️ Chẩn đoán KHÔNG DẤU (NFR16/Kiểm A `check:i18n`), đây là log không phải văn
            // bản hiển thị.
            Some(stale_segment_id) => {
                eprintln!(
                    "chapter[{chapter_id}] chapter_position tro vao segment {stale_segment_id} \
                     da ve huu, roi ve segment dau"
                );
                segments.first().map(|s| s.id)
            }
            // Chương chưa từng mở (không hàng `chapter_position`) ⇒ segment đầu (AC5).
            // Chương rỗng (`segments` rỗng) hội tụ về `None` ở cả hai nhánh trên VÀ nhánh
            // này — đúng §I/O Matrix "Chương không có segment nào ⇒ caret_segment_id = null".
            None => segments.first().map(|s| s.id),
        };

        // **THÊM Story 6.14** — hàng `asset` thô CÙNG lượt đọc này (không một `Store::read`
        // thứ hai): một lượt gộp/tách chen giữa hai lượt đọc rời có thể làm `segments` và
        // `assets` đến từ hai ảnh chụp khác nhau, cùng lý lẽ `caret_segment_id` đã ghi ở trên.
        let raw_assets = select_chapter_assets(conn, chapter_id)?;
        let assets: Vec<ChapterAsset> = crate::core::segment::image::resolve_chapter_images(
            &segments,
            &raw_assets,
            true,  // include_omitted -- luoi khong loc cat bo (FR44).
            false, // exclude_roles -- mot hang alt/caption van la mot o nguyen van binh thuong.
        )
        .into_iter()
        .map(ChapterAsset::from)
        .collect();

        Ok(ChapterSegments {
            chapter_id,
            segments,
            caret_segment_id,
            assets,
            assets_dir: assets_dir_str,
            tm_filled_segment_ids: Vec::new(),
            tm_prefill: TmPrefillStatus::NotAsked,
        })
    })?;

    Ok(loaded)
}

pub(crate) fn global_store_missing() -> IpcError {
    crate::core::store::StoreError::OpenFailed {
        store: crate::core::store::StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

pub(crate) fn tm_lookup_failed(err: &crate::core::tm::TmStoreError) -> IpcError {
    match err {
        crate::core::tm::TmStoreError::Store(e) => IpcError::from(e.clone()),
        other => {
            eprintln!("tm lookup that bai: {other}");
            IpcError::new("tm.lookup_failed", MessageKey::Unknown, BTreeMap::new(), true)
        }
    }
}

/// Loads the open Chapter after pre-filling every eligible segment from the first exact TM
/// pair (FR58, AD-18 order). The fill writes through [`write_non_user_target`] only. A failed
/// fill degrades to `TmPrefillStatus::Skipped`; only the segment read can fail the load.
pub fn load_open_chapter_segments(
    global: Option<&crate::core::store::Store>,
    open: Option<&OpenWork>,
    prefill: bool,
) -> Result<ChapterSegments, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let outcome = if prefill {
        match global
            .ok_or_else(global_store_missing)
            .and_then(|g| fill_exact_tm_matches(g, open))
        {
            Ok(filled) => Some(Ok(filled)),
            Err(e) => {
                eprintln!("tm prefill bi bo qua: {}", e.code());
                Some(Err(e.code().to_owned()))
            }
        }
    } else {
        None
    };
    let mut chapter = read_open_chapter_segments(Some(open))?;
    match outcome {
        Some(Ok(filled)) => {
            chapter.tm_filled_segment_ids = filled
                .into_iter()
                .filter(|id| chapter.segments.iter().any(|s| s.id == *id))
                .collect();
            chapter.tm_prefill = TmPrefillStatus::Ran;
        }
        Some(Err(code)) => chapter.tm_prefill = TmPrefillStatus::Skipped { code },
        None => {}
    }
    Ok(chapter)
}

fn fill_exact_tm_matches(
    global: &crate::core::store::Store,
    open: &OpenWork,
) -> Result<Vec<i64>, IpcError> {
    let chapter_id = open.chapter_id;
    let candidates: Vec<(i64, String)> = open.store.read(move |conn| {
        let mut stmt = conn.prepare(&format!(
            "SELECT id, source_text FROM segment \
             WHERE chapter_id = ?1 AND retired_at IS NULL AND {IN_TRANSLATION_SQL} AND status = ?2 \
             ORDER BY ord, id"
        ))?;
        let rows = stmt
            .query_map((chapter_id, SEGMENT_STATUS_DRAFT), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok(rows)
    })?;

    let mut by_source: BTreeMap<String, Option<(String, &'static str)>> = BTreeMap::new();
    let mut picks: Vec<(i64, String, &'static str)> = Vec::new();
    for (id, source) in candidates {
        if source.trim().is_empty() {
            continue;
        }
        if !by_source.contains_key(&source) {
            let pairs = crate::core::tm::pairs_for_source(&open.scope, global, Some(&open.store), &source)
                .map_err(|e| tm_lookup_failed(&e))?;
            let first = pairs
                .into_iter()
                .next()
                .map(|p| (p.target_text, p.translation_origin.as_str()));
            by_source.insert(source.clone(), first);
        }
        if let Some(Some((target, pair_origin))) = by_source.get(&source) {
            picks.push((id, target.clone(), pair_origin));
        }
    }
    if picks.is_empty() {
        return Ok(Vec::new());
    }

    let filled = open.store.write(move |tx: &Transaction<'_>| {
        let mut filled = Vec::new();
        for (id, target, pair_origin) in &picks {
            let still_eligible: bool = tx.query_row(
                &format!(
                    "SELECT status = ?2 AND trim(target_text) = '' AND {IN_TRANSLATION_SQL} \
                     AND retired_at IS NULL FROM segment WHERE id = ?1"
                ),
                (id, SEGMENT_STATUS_DRAFT),
                |row| row.get(0),
            )?;
            if still_eligible {
                write_non_user_target(tx, *id, target, pair_origin, Some(pair_origin))?;
                filled.push(*id);
            }
        }
        Ok(filled)
    })?;
    Ok(filled)
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.12 — MỘT LƯỢT ĐỌC (`ReadingRun`): CHỈ CÁC CHƯƠNG ĐÃ `Done` (FR120)
// ═════════════════════════════════════════════════════════════════════════════════
// 🔵 SỬA 2026-08-30 (Story 5.12) — mệnh đề cũ ở đây (Story 5.11) khai "hình dạng dây đã
// CHỐT ở bốn trường" cho một Chương ĐƠN, đọc bất kể `chapter.status`. Mệnh đề đó HẾT ĐÚNG:
// `read_reading_chapter` chưa từng nhìn cột `status` một lần nào, và hệ quả đo được là mở
// một Chương đang dịch dở rồi bấm `⌘3` ném nguyên văn tiếng Trung ra giữa trang đọc tiếng
// Việt — đúng thứ FR120 tồn tại để chặn (§Design Notes "Vì sao Chương đang mở chưa xong ⇒
// không hiện gì" của story này). `readingState.ts:16-25` phía frontend dựa vào chính mệnh
// đề đã sai đó để đi vòng một lượt IPC phụ (`listChapters()`) — sửa tại chỗ cùng lượt.
//
// Bề mặt đọc nay là MỘT LƯỢT ĐỌC (`ReadingRun`): Rust chọn dãy Chương LIÊN TIẾP ở trạng thái
// `Done` bắt đầu TẠI Chương đang mở, trả kèm một MỐC BIÊN (`ReadingFrontier`) nói vì sao dãy
// dừng ở đó. Chương chưa `Done` không bao giờ rời `project.db` — cùng kỷ luật `is_omitted`
// đã dùng ở [`paragraphs_in_translation`]: webview không có gì để lọc vì không có gì được
// gửi (§Always của story).
//
// Bốn struct dưới đây là hình dạng RIÊNG của Chế độ đọc — không dùng lại `ChapterSegment`/
// `ChapterSegments` của Editor, cùng lý lẽ đã ghi ở Story 5.11: Editor cần chín trường (vạch
// lề, xác nhận, cắt bỏ…); Chế độ đọc không có công cụ biên tập nào và không được phép mang
// một đường nào tới `is_omitted` qua dây.
