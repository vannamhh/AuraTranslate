//! Bề mặt IPC tách segment **tường minh** — Story 2.1, AC3 · AC8 · AC13 · AC14.
//!
//! Cùng khuôn `commands::chapter`/`commands::project`: hàm thuần trước, `#[tauri::command]`
//! chỉ là vỏ mỏng trong `wire`. Hàm thuần nhận `Option<&OpenWork>` — đây là thứ `tests/**`
//! gọi được **mà không cần webview**.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MỘT LỆNH TƯỜNG MINH CHỨ KHÔNG MỘT BƯỚC DI TRÚ DỮ LIỆU — Quyết định #4
//! ─────────────────────────────────────────────────────────────────────────────
//! `deferred-work.md:542` để ngỏ đúng hai đường cho **25 Chương Epic 1** đang mang
//! `segment_count = 0`: một thao tác tách tường minh, hoặc một bước di trú dữ liệu. Đường
//! thứ hai bị loại vì ba lý do độc lập:
//!
//! 1. Một bước di trú là **DDL**; chạy một quy tắc nghiệp vụ trong đó trộn hai tầng.
//! 2. Nó chạy **im lặng** lúc mở Tác phẩm — khó phân biệt với đúng cái *"đường tính ngầm
//!    lúc nạp Chương"* mà AC3 cấm bằng chữ.
//! 3. Bản sao lưu trước di trú **không nguyên tử và không xác minh lại**
//!    (`deferred-work.md:254`, chưa ai vá), và đó sẽ là lượt di trú thật đầu tiên chạy trên
//!    một `project.db` **đã có dữ liệu người dùng**.
//!
//! ⇒ Bước di trú 5 chỉ làm **một việc**: `CREATE TABLE segment`. Chương **mới** nhập được
//! tách tự động trong `create_work` (cùng giao dịch — AC13); Chương **cũ** đi qua lệnh này,
//! một Chương một lượt.
//!
//! ⚠️ Lệnh **từ chối** một Chương đã có segment thay vì ghi đè. AD-4 đóng băng ranh giới
//! vĩnh viễn và AD-3 cấm tái dùng id đã về hưu — một lượt ghi đè im lặng là một lượt về hưu
//! im lặng, và lịch sử của Story 2.6 sẽ trỏ vào những id không ai biết đã mất.
//!
//! ⚠️ **AC8 vế hai** (*"không có đường nào tự động tách lại toàn bộ Thư viện"*) được giao ở
//! đây bằng cách **không tồn tại**, và `tests/segment_boundary.rs` khẳng định điều đó. Vế
//! một (nút tái tách kèm cảnh báo về dữ liệu sẽ về hưu) thuộc **Story 2.8** — hôm nay chưa
//! có `SegmentVersion` để mà giữ lại.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::commands::project::OpenWork;
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::segment::split::{SplitSegment, split_source_text};
use crate::core::store::{SqlError, SqlResult, Transaction};

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
/// lại câu lệnh **mỗi hàng**. `deferred-work.md:2012-2024` ghi món này với chủ là Story 2.2
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
/// lượt ghi khác của tiến trình. Cùng điểm nghẽn mà `commands/project.rs:120-127` đã kéo
/// `split_source_text` ra ngoài để né.
///
/// ⚠️ `prepare_cached` (không phải `prepare`): bộ nhớ đệm sống trên **kết nối**, mà kết nối
/// ghi là một kết nối dài hạn của pool — nên Chương **thứ hai** trở đi không phải parse lại
/// một lần nào nữa. Với `prepare`, mỗi lượt gọi hàm này vẫn mất đúng một lượt parse.
pub(crate) fn insert_segments(
    tx: &Transaction<'_>,
    chapter_id: i64,
    segments: &[SplitSegment],
) -> SqlResult<()> {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, \
         created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
         strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
    )?;
    for (index, segment) in segments.iter().enumerate() {
        let ord = i64::try_from(index).unwrap_or(i64::MAX).saturating_add(1);
        stmt.execute((
            chapter_id,
            ord,
            &segment.text,
            i64::from(segment.is_paragraph_end),
        ))?;
    }
    Ok(())
}

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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ChapterSegment {
    pub id: i64,
    pub ord: i64,
    pub source_text: String,
    pub target_text: String,
    pub is_paragraph_end: bool,
    pub retired_at: Option<String>,
    pub status: String,
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
}

/// Chương không có trong `project.db` của Tác phẩm đang mở.
fn chapter_not_found(chapter_id: i64) -> IpcError {
    IpcError::new(
        "segment.chapter_not_found",
        MessageKey::SegmentChapterNotFound,
        BTreeMap::from([("chapter_id".to_owned(), chapter_id.to_string())]),
        false,
    )
}

/// Chương đã có segment ⇒ **từ chối**, không ghi đè. Xem doc-comment đầu module.
fn already_split(chapter_id: i64, count: i64) -> IpcError {
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
    let segments = split_source_text(&source_text, &open.meta.source_lang);
    let segment_count = segments.len();

    open.store
        .write(move |tx: &Transaction<'_>| insert_segments(tx, chapter_id, &segments))?;

    Ok(SplitOutcome {
        chapter_id,
        segment_count,
    })
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
/// người dùng chỉ đích danh trong Thư viện, và `deferred-work.md:542` đếm 25 Chương như
/// vậy.
///
/// ⚠️ Story 2.11 sở hữu biến thể nhận `chapter_id`. **Đừng** thêm sẵn một tham số
/// `Option<i64>` hôm nay: một nhánh không chỗ gọi nào đi qua là một nhánh không ai nghiệm
/// thu được — cùng luật đã ghi cho danh mục `MessageKey` (`core::i18n`).
///
/// ⚠️ `ORDER BY ord` là **có chủ đích**, không phải trang trí: `idx_segment_chapter_ord`
/// (`chapter_id, ord`) thành covering cho đúng lượt đọc này, nên SQLite khỏi một lượt sắp
/// tạm trên **9.850** hàng của Chương lớn nhất có thật.
pub fn read_open_chapter_segments(open: Option<&OpenWork>) -> Result<ChapterSegments, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    let loaded = open.store.read(|conn| {
        // Cung quy tac chon Chuong voi `read_open_chapter`: mot Tac pham mot Chuong o Epic 1.
        let chapter_id: i64 =
            conn.query_row("SELECT id FROM chapter ORDER BY ord LIMIT 1", [], |row| {
                row.get(0)
            })?;

        let mut stmt = conn.prepare(
            "SELECT id, ord, source_text, target_text, is_paragraph_end, retired_at, status \
             FROM segment WHERE chapter_id = ?1 ORDER BY ord",
        )?;
        let rows = stmt.query_map([chapter_id], |row| {
            // ⚠️ `is_paragraph_end` la INTEGER 0/1 duoi SQLite (khong co kieu boolean); phep
            // doi sang `bool` la viec cua tang nay, dung khuon `chapter.status`.
            let flag: i64 = row.get(4)?;
            Ok(ChapterSegment {
                id: row.get(0)?,
                ord: row.get(1)?,
                source_text: row.get(2)?,
                target_text: row.get(3)?,
                is_paragraph_end: flag != 0,
                retired_at: row.get(5)?,
                status: row.get(6)?,
            })
        })?;
        let segments = rows.collect::<SqlResult<Vec<ChapterSegment>>>()?;

        Ok(ChapterSegments {
            chapter_id,
            segments,
        })
    })?;

    Ok(loaded)
}

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
fn unknown_segment_ids(chapter_id: i64, count: usize) -> IpcError {
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
/// `ARCHITECTURE-SPINE.md:376` nói bằng một hàng bảng: *"Auto-save (FR100) | trạng thái
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
             WHERE id = ?2 AND chapter_id = ?3",
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
enum BatchReject {
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

/// Hai giá trị hợp lệ của `segment.status`, và **đúng hai** — Quyết định #5 (Ice ký
/// 2026-08-14).
///
/// ⚠️ Cưỡng chế ở **tầng Rust**, không bằng một `CHECK` trong DDL: cùng khuôn
/// `chapter.status` và `config_value.kind`, và thêm một `CHECK` ở một bảng mà hai bảng anh
/// em không có là dựng hai quy ước cho cùng một việc *(doc-comment `schema.rs`)*.
pub const SEGMENT_STATUS_DRAFT: &str = "draft";
/// Xem [`SEGMENT_STATUS_DRAFT`].
pub const SEGMENT_STATUS_CONFIRMED: &str = "confirmed";

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
fn segment_not_found(segment_id: i64) -> IpcError {
    IpcError::new(
        "segment.not_found",
        MessageKey::SegmentNotFound,
        BTreeMap::from([("segment_id".to_owned(), segment_id.to_string())]),
        false,
    )
}

/// Segment đã về hưu (AD-5) ⇒ không xác nhận được.
fn segment_retired(segment_id: i64) -> IpcError {
    IpcError::new(
        "segment.retired",
        MessageKey::SegmentRetired,
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
/// 🔴 MỐI NỐI ĐỂ MỞ, KHÔNG BỊ CHÔN — AC11
/// ─────────────────────────────────────────────────────────────────────────────
/// Nhánh `Ok(true)` ngay dưới **là** chuyển tiếp *"sang đã xác nhận"* của AD-31, và đó là
/// **chỗ duy nhất** hai thứ sau sẽ được ghi:
/// - **xuất xứ (FR117)** — chủ: **Story 2.7**. Nó so *văn bản đích hiện tại* với *bản lúc
///   nạp segment*, **không dùng cờ dirty** (hợp đồng phụ AD-31). Mốc so sánh sống ở webview
///   (`editorPanelState.ts`, `segments` giữ bản lúc nạp) và **phải giữ tách rời** `editedText`.
/// - **cặp TM (FR56)** — chủ: **Epic 7**. `EXPERIENCE.md:294`: *"Cặp TM mới được ghi ngay tại
///   chuyển tiếp đó (AD-31)"*.
///
/// ⚠️ Story 2.5 **không cài** hai thứ đó. Nó để lại mối nối ở một chỗ **gọi tên được** —
/// chính nhánh này — thay vì rải chúng ra sau.
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
        let found = tx.query_row(
            "SELECT target_text, status, retired_at FROM segment WHERE id = ?1",
            [segment_id],
            |row| {
                let target_text: String = row.get(0)?;
                let status: String = row.get(1)?;
                let retired_at: Option<String> = row.get(2)?;
                Ok((target_text, status, retired_at))
            },
        );

        let (target_text, status, retired_at) = match found {
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

        // ④ CHUYEN TIEP. Day la cho Story 2.7 (xuat xu) va Epic 7 (cap TM) moc vao.
        tx.execute(
            "UPDATE segment SET status = ?1 WHERE id = ?2",
            (SEGMENT_STATUS_CONFIRMED, segment_id),
        )?;
        tx.execute(
            "INSERT INTO segment_version (segment_id, target_text, created_at) \
             VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (segment_id, &target_text),
        )?;

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
            "SELECT COUNT(*) FROM segment WHERE id = ?1 AND chapter_id = ?2",
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

/// Một vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{
        ChapterSegments, ConfirmOutcome, IpcError, SaveOutcome, SegmentTargetEdit, SplitOutcome,
    };
    use crate::commands::project::OpenWorkState;

    /// Vỏ IPC của [`super::split_chapter_into_segments`].
    ///
    /// ⚠️ `try_state`, không `state()` — cùng lý do `commands::chapter::wire`: state có thể
    /// chưa từng được `app.manage` (lỗi cấu hình `setup()`), và `panic = "abort"` giết cả
    /// tiến trình nếu ta thẳng tay `.unwrap()`.
    ///
    /// ⚠️ `chapter_id` đi trên dây dưới tên **`chapterId`** — `invoke()` gửi tham số ở dạng
    /// camelCase. `src/config/segment.ts` là chỗ duy nhất gõ cái tên đó.
    #[tauri::command]
    pub fn split_chapter_into_segments(
        app: tauri::AppHandle,
        chapter_id: i64,
    ) -> Result<SplitOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::split_chapter_into_segments(None, chapter_id);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::split_chapter_into_segments(guard.as_ref(), chapter_id)
    }

    /// Vỏ IPC của [`super::read_open_chapter_segments`].
    ///
    /// ⚠️ **Không tham số nào đi trên dây** — xem doc-comment của hàm thuần. `invoke()` phía
    /// webview gọi nó với một payload rỗng.
    #[tauri::command]
    pub fn read_open_chapter_segments(app: tauri::AppHandle) -> Result<ChapterSegments, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::read_open_chapter_segments(None);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::read_open_chapter_segments(guard.as_ref())
    }

    /// Vỏ IPC của [`super::save_segment_targets`] — đường flush của AD-35. Story 2.3.
    ///
    /// ⚠️ `chapter_id` đi trên dây dưới tên **`chapterId`**, và `edits` dưới tên **`edits`** —
    /// `invoke()` gửi tham số ở dạng camelCase. `src/config/segment.ts` là chỗ duy nhất gõ
    /// hai cái tên đó.
    ///
    /// 🔴 `MutexGuard` giữ **XUYÊN SUỐT** lời gọi, cùng lý do và cùng đường hỏng mà
    /// [`super::split_chapter_into_segments`] đã ghi ở doc-comment của nó. Nhả sớm để "tối ưu"
    /// là mở lại một cuộc đua ghi: `replace_open_work` có thể trỏ `OpenWorkState` sang một Tác
    /// phẩm **khác** giữa lúc lô này đang bay, và lúc đó lô ghi vào `project.db` của Tác phẩm
    /// vừa bị thay.
    #[tauri::command]
    pub fn save_segment_targets(
        app: tauri::AppHandle,
        chapter_id: i64,
        edits: Vec<SegmentTargetEdit>,
    ) -> Result<SaveOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::save_segment_targets(None, chapter_id, &edits);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // ⚠️ **KHÔNG một quyết định nào ở đây** — kể cả thứ tự hai lượt ghi của AD-31 hàng 3.
        // Nó sống ở `super::flush_segment_targets`, và lý do là một phép đo: đặt nó ở vỏ này
        // thì đảo thứ tự đi qua **54/54 ca xanh**, vì `tests/**` gọi một vỏ cần `AppHandle`
        // không được. Xem doc-comment của hàm thuần đó.
        //
        // ⚠️ Cả hai lượt ghi bên trong đi dưới **cùng một** `MutexGuard`: nhả giữa chừng là mở
        // lại đúng cuộc đua đã ghi ở dưới *(`replace_open_work` trỏ sang Tác phẩm khác giữa
        // lúc lô đang bay)*.
        super::flush_segment_targets(guard.as_ref(), chapter_id, &edits).map(|(_, saved)| saved)
    }

    /// Vỏ IPC của [`super::confirm_segment`] — Story 2.5, FR24 · AD-31.
    ///
    /// ⚠️ `segment_id` đi trên dây dưới tên **`segmentId`** — `invoke()` gửi tham số ở dạng
    /// camelCase. `src/config/segment.ts` là chỗ duy nhất gõ cái tên đó.
    ///
    /// 🔴 `try_state`, **không** `state()` — mở kho có thể đã thất bại và `app.manage()` chưa
    /// từng chạy ⇒ `state()` panic ⇒ `panic = "abort"` giết cả tiến trình.
    ///
    /// 🔴 `MutexGuard` giữ **XUYÊN SUỐT** lời gọi, cùng lý do [`save_segment_targets`] đã ghi.
    ///
    /// ⚠️ **AD-35 vế (c) — flush TRƯỚC, xác nhận SAU — KHÔNG được cưỡng chế ở đây, và đó là
    /// một giới hạn thật, ghi ra thay vì để người sau tự phát hiện.** Lệnh này chỉ đọc thứ
    /// **đã ở trên đĩa**; nó không biết gì về văn bản đang gõ trong webview. Vế *"flush xong
    /// rồi mới ghi trạng thái"* được giao ở **tầng TypeScript**, tại chỗ command được gọi
    /// (`src/commands/index.ts` — `await flushEditorNow()` rồi mới `invoke`). Cái giá: một
    /// chỗ gọi tương lai quên `await` sẽ ký một văn bản **cũ hơn** thứ người dùng đang nhìn,
    /// và **không cổng nào ở tầng Rust bắt được**. Lưới ở đây là một test frontend, không một
    /// hợp đồng Rust.
    #[tauri::command]
    pub fn confirm_segment(
        app: tauri::AppHandle,
        segment_id: i64,
    ) -> Result<ConfirmOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::confirm_segment(None, segment_id);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::confirm_segment(guard.as_ref(), segment_id)
    }
}
