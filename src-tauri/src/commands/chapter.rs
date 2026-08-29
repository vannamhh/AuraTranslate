//! Bề mặt IPC đọc **Chương đang mở** — Story 1.16, AC8.
//!
//! Cùng khuôn `commands::config`/`commands::project`: hàm thuần trước, `#[tauri::command]`
//! chỉ là vỏ mỏng trong `wire`. Hàm thuần nhận `Option<&OpenWork>` — đúng khuôn
//! `Option<&Store>` của `commands::config` — chứ không nhận `Store` trực tiếp, vì Chương
//! sống trong Tác phẩm **đang mở** (`OpenWorkState`), không phải một kho được `app.manage`
//! thẳng như `global.db`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 STORY NÀY KHÔNG DỰNG "MỞ LẠI MỘT `.atproj` TỪ ĐĨA"
//! ─────────────────────────────────────────────────────────────────────────────
//! Đọc `OpenWorkState` — Tác phẩm mà `create_work_from_text`/`create_work_from_file` (Story
//! 1.15) vừa đặt vào, hoặc `chưa có` khi webview vừa mở lại từ đầu. Không có đường
//! `WorkMeta::read` nào ở đây.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 THÊM 2026-08-21 (Story 3.4) — ĐÂY LÀ ĐƯỜNG "MỞ CHƯƠNG" MÀ `Jieba` HÂM NÓNG VÀO
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 **SỬA 2026-08-29 (Story 5.7) — "HAI điểm duy nhất" nay là BA.** `open_chapter` (mở một
//! Chương ĐÍCH DANH, story này) là điểm thứ ba, và nó gọi cùng hàm hâm nóng vì cùng lý do.
//! Mệnh đề "duy nhất" giữ nguyên hình dạng — mọi đường đưa một `source_lang` mới lên webview
//! đều phải hâm — chỉ con số đổi. *(`open_work` KHÔNG nằm trong danh sách này và đó là đúng:
//! nó không trả `source_text`/`source_lang` nào lên webview; webview luôn theo sau bằng
//! `read_open_chapter`, và lượt đó hâm.)*
//!
//! `read_open_chapter`/`open_adjacent_chapter`/`open_chapter` là ba điểm sản phẩm duy nhất đưa
//! một `source_lang` mới lên webview. Cả ba gọi
//! `core::glossary::warm_jieba_for_source_lang` NGAY sau khi biết `open` tồn tại — đóng
//! `deferred-work.md:413`: khởi tạo lạnh `Jieba` tốn 179–329 ms, và nó phải rơi vào một
//! thao tác đã chấp nhận độ trễ đó (mở Chương), không rơi vào đường gõ.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use crate::commands::project::OpenWork;
use crate::core::i18n::{IpcError, MessageKey};

/// Chương **đang mở**, đọc từ `OpenWorkState` — không phải một hàng chọn được (Epic 2).
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OpenChapter {
    /// `chapter.id` — định danh Chương, `dict_entry.id`-style: chỉ có nghĩa trong
    /// `project.db` của chính Tác phẩm đang mở.
    pub chapter_id: i64,
    /// `chapter.source_text` — nguyên khối, không tách câu/đoạn (AD-4, Story 2.1 sở hữu
    /// việc tách).
    pub source_text: String,
    /// `work.source_lang` — `"zh"` hoặc `"en"`, trường **bất biến** ghi lúc tạo (AD-18).
    /// Panel Source dùng đúng trường này để quyết có tab Hán Việt hay không (AC3), không
    /// không đoán từ nội dung `source_text`.
    pub source_lang: String,
}

/// Tác phẩm đang mở ⇒ lỗi *chưa mở Tác phẩm nào*, và đó là câu đúng theo nghĩa đen.
///
/// Đi qua `IpcError::new` với `MessageKey::WorkNoneOpen` — không phải một lỗi kho
/// (`StoreError`), nên nó không thuộc từ vựng `store.*` (§Quyết định của Story 1.16).
///
/// ⚠️ Riêng tư trở lại từ 2026-08-11: Story 1.20 từng nâng nó lên `pub(crate)` để
/// `commands::pinned` tái dùng, nhưng lượt Ice ký lại chuyển mục ghim sang `global.db` —
/// nơi *"chưa mở Tác phẩm nào"* **không phải** một câu có nghĩa. Đọc Chương lại là chỗ duy
/// nhất nói câu đó, nên nó về đúng phạm vi cũ.
///
/// ⚠️ `pub(crate)` **trở lại** từ 2026-08-11 (Story 2.1), và điều kiện hạ phạm vi ở trên
/// không còn đúng: `commands::segment` tách một Chương **của Tác phẩm đang mở**, nên với nó
/// câu *"chưa mở Tác phẩm nào"* là câu đúng theo nghĩa đen — khác hẳn ca mục ghim. Hai chỗ
/// gọi, **một** khoá; một khoá thứ hai cho cùng câu là hai chuỗi phải giữ khớp nhau bằng
/// kỷ luật.
pub(crate) fn no_work_open() -> IpcError {
    IpcError::new(
        "work.none_open",
        MessageKey::WorkNoneOpen,
        std::collections::BTreeMap::new(),
        false,
    )
}

/// Hàng `chapter` được chỉ **không có** trong `project.db` đang mở.
///
/// 🔵 **THÊM 2026-08-18 (Story 2.11)** — đóng món nợ `deferred-work.md:650`. Trước đây
/// `conn.query_row(...)` ném `QueryReturnedNoRows` khi bảng `chapter` rỗng, đi qua
/// `From<StoreError>` thành `store.read_failed`, và người dùng đọc *"khong mo duoc kho du
/// lieu"* cho một Tác phẩm hoàn toàn lành lặn — một câu **sai về loại**: không tệp nào hỏng.
///
/// ⚠️ **Tái dùng `MessageKey::SegmentChapterNotFound`, KHÔNG một khoá thứ hai.** Cùng câu,
/// cùng nghĩa, cùng tham số `chapter_id`; hai khoá cho cùng một câu là hai chuỗi phải giữ
/// khớp nhau bằng kỷ luật. Cùng lập luận `no_work_open` ở trên đã đi qua hai lần.
///
/// ⚠️ `pub(crate)` từ 2026-08-27 (Story 5.4): `commands::lifecycle::set_chapter_status` tái
/// dùng ĐÚNG hàm này cho ca "`chapter_id` không tồn tại" — cùng lý do `no_work_open` đã lên
/// `pub(crate)` trước đó, không đúc một bản chép thứ hai cho cùng một `IpcError`.
pub(crate) fn chapter_not_found(chapter_id: i64) -> IpcError {
    IpcError::new(
        "segment.chapter_not_found",
        MessageKey::SegmentChapterNotFound,
        std::collections::BTreeMap::from([("chapter_id".to_owned(), chapter_id.to_string())]),
        false,
    )
}

/// Đọc Chương đang mở — **hàm thuần, đây là thứ test gọi**.
///
/// 🔵 **SỬA 2026-08-18 (Story 2.11 · Quyết định #2(a), Ice ký).** Câu SQL cũ là
/// `SELECT ... FROM chapter ORDER BY ord LIMIT 1` — nó **suy ra** Chương đang mở thay vì
/// **hỏi** nó, và mệnh đề đứng sau nó *(một Tác phẩm có đúng một Chương)* hết đúng ngay lượt
/// Chương thứ hai tồn tại. Nay nó đọc [`OpenWork::chapter_id`], nguồn sự thật DUY NHẤT.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `work.none_open`;
/// - hàng `chapter` được chỉ vắng mặt ⇒ `segment.chapter_not_found` *(một lỗi **có tên** —
///   xem [`chapter_not_found`])*;
/// - đường đọc trượt (kho hỏng) ⇒ `store.read_failed` (qua `From<StoreError>`).
pub fn read_open_chapter(open: Option<&OpenWork>) -> Result<OpenChapter, IpcError> {
    // 🔵 SUA 2026-08-18 (Story 2.11) — ba dong o day truoc kia viet: "Epic 1 tao DUNG MOT
    // Chuong cho moi Tac pham, `ord = 1` (Story 1.15). Chon Chuong / chuyen Chuong la Epic 2
    // — khong thuoc pham vi story nay." Ve THU HAI da HET DUNG: story nay LA story do. Ve
    // thu nhat van dung ve mat du lieu (khong duong san pham nao sinh Chuong thu hai, mon no
    // co chu: Epic 6), nhung no khong con la mot tien de cua ma nay.
    let open = open.ok_or_else(no_work_open)?;
    let chapter_id = open.chapter_id;

    // 🔵 THEM 2026-08-21 (Story 3.4) — day la duong MO CHUONG ma deferred-work.md:413 cho
    // ham Jieba vao: khoi tao lanh ton 179-329ms, va lan goi dau tien khong duoc phep roi
    // dung phim dau nguoi dung go. Ham nong o DAY (mot thao tac da chap nhan do tre vai
    // tram ms), khong trong than mot ham khop. Chi ham that su khi `source_lang` la tieng
    // Trung -- xem doc-comment cua `warm_jieba_for_source_lang`.
    crate::core::glossary::warm_jieba_for_source_lang(&open.meta.source_lang);

    // 🔴 `query_map().next()` chu KHONG `query_row`: `query_row` bien "0 hang" thanh mot
    // `QueryReturnedNoRows`, tuc mot loi KHO — xem `chapter_not_found`. `Option` o day la
    // mot trang thai san pham binh thuong, khong mot tep nao hong.
    let found = open.store.read(move |conn| {
        let mut stmt = conn.prepare("SELECT source_text FROM chapter WHERE id = ?1")?;
        let mut rows = stmt.query_map([chapter_id], |row| row.get::<_, String>(0))?;
        rows.next().transpose()
    })?;

    let Some(source_text) = found else {
        return Err(chapter_not_found(chapter_id));
    };

    Ok(OpenChapter {
        chapter_id,
        source_text,
        source_lang: open.meta.source_lang.clone(),
    })
}

/// Hướng của một lượt chuyển Chương — **danh mục ĐÓNG hai giá trị**, Story 2.11 · FR26.
///
/// ⚠️ `#[serde(rename = …)]` **từng biến thể**, không `#[serde(rename_all)]` — đúng tiền lệ
/// `core::dict::QueryRoute`: một câu snake_case tự động đúng cho hai cái tên hôm nay nhưng
/// không đảm bảo cho một biến thể tương lai, còn rename từng cái là tường minh.
///
/// 🔴 **HAI giá trị, không một `i32` mang dấu.** Một `step: i32` cho phép `0`, `2`, `-7` —
/// ba giá trị không nghĩa mà mọi chỗ gọi phải tự từ chối, mỗi chỗ theo một cách.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum ChapterDirection {
    /// Chương kế tiếp theo thứ tự `(ord, id)` tăng dần.
    #[serde(rename = "next")]
    Next,
    /// Chương liền trước theo thứ tự `(ord, id)` giảm dần.
    #[serde(rename = "prev")]
    Prev,
}

/// Kết cục một lượt gọi [`open_adjacent_chapter`] — **phân biệt được**, không phải một
/// `Option` trần.
///
/// 🔴 **VÌ SAO KHÔNG CHỈ MỘT `Option<OpenChapter>`.** `project-context.md:473-499`:
/// *"Rỗng IM LẶNG bị cấm; rỗng CÓ LÝ DO thì không"*. Một `null` trên dây không tự nói vì sao
/// nó `null` — *"đã ở Chương đầu"* và *"đã ở Chương cuối"* là hai câu khác nhau trên màn
/// hình (AC4 đòi *"báo rõ đã ở biên"*), và webview không được phép suy ra chúng từ hướng nó
/// vừa gửi đi: suy ra là dựng một nguồn sự thật thứ hai cho cùng một dữ kiện.
///
/// ⚠️ Và **không** biến thể nào cho *"lỗi"*. Vượt biên **không phải một lỗi** — nó là một
/// kết cục hợp lệ của một thao tác hợp lệ. Lỗi đi bằng `Err(IpcError)`, đúng chỗ của nó.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ChapterSwitchOutcome {
    /// Con trỏ Chương **đã đổi** ⇒ trường `chapter` mang Chương mới.
    #[serde(rename = "moved")]
    Moved,
    /// Đã ở Chương **đầu**, lệnh *Chương trước* không đi đâu cả.
    #[serde(rename = "at-first")]
    AtFirst,
    /// Đã ở Chương **cuối**, lệnh *Chương sau* không đi đâu cả.
    #[serde(rename = "at-last")]
    AtLast,
}

/// Kết quả một lượt chuyển Chương — thứ đi ra qua dây. Story 2.11 · AC1 · AC2 · AC4.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
/// Trường đi trên dây đúng tên này: `outcome` · `chapter`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChapterSwitch {
    /// Vì sao lượt gọi kết thúc như nó đã kết thúc.
    pub outcome: ChapterSwitchOutcome,
    /// Chương **mới** — `Some` khi và chỉ khi `outcome == Moved`.
    ///
    /// 🔴 `None` ở biên là **có chủ ý**, không một chỗ tiết kiệm: `source_text` là nguyên
    /// khối văn bản của cả Chương *(9.850 câu ở Chương lớn nhất có thật)*, và trả lại đúng
    /// Chương cũ ở một lượt **không đổi** là mời webview đọc nó như một lượt **đã đổi**.
    pub chapter: Option<OpenChapter>,
}

/// **Mở Chương kề** theo một hướng — hàm thuần, đây là thứ test gọi. Story 2.11 · FR26.
///
/// Ice ký **Quyết định #3 đường (a)** ngày 2026-08-18: **Rust** quyết Chương kề, webview chỉ
/// nói **hướng**. Hai đường bị loại: `list_chapters()` + webview tự chọn *(webview phải mang
/// luật "kề là gì" ⇒ đụng AD-1)*, và thêm `chapter_id: Option<i64>` vào hai lệnh đọc *(hình
/// dạng mà chú thích ở `commands/segment.rs:773-775` cấm tường minh)*.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 SO SÁNH BỘ ĐÔI `(ord, id)` — VÀ `ord + 1` BỊ CẤM, KHÔNG PHẢI MỘT GU
/// ─────────────────────────────────────────────────────────────────────────────
/// `chapter.ord` **cố ý không `UNIQUE`** (`schema.rs:249`, doc-comment `:233-235`) và không
/// gì bảo đảm nó liên tục. Một cài đặt `WHERE ord = ?1 + 1` biên dịch sạch, đi qua **mọi**
/// cổng, và báo *"đã ở Chương cuối"* trên một Tác phẩm còn nguyên Chương phía sau ngay khi
/// `ord` thưa hoặc trùng. Lượt code review 2026-08-17 trên `commands/segment.rs` đã gọi đúng
/// tên lớp lỗi này: *"một phép trừ im lặng trỏ sai hàng"*.
///
/// ⇒ Vị từ là một so sánh **từ điển** trên bộ đôi: `(ord, id) > (ord0, id0)` viết ra thành
/// `ord > ?1 OR (ord = ?1 AND id > ?2)`, và thứ tự sắp xếp mang **cùng** hai khoá. Đúng khuôn
/// `commands/segment.rs` đã dùng cho lượt tìm câu liền trên.
///
/// ⚠️ **SQLite hiểu cú pháp `(a, b) > (?1, ?2)` từ 3.15**, và kho ghim sàn cao hơn thế nhiều
/// (FTS5 `trigram` đòi ≥ 3.34). Dạng viết tay ở đây được chọn **có chủ ý**: nó là dạng mà
/// planner dùng được index `(ord, id)` trên **mọi** phiên bản, và nó đọc ra thành đúng mệnh
/// đề mà doc-comment này vừa phát biểu.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `work.none_open`;
/// - đi được nhưng hàng đích biến mất giữa hai truy vấn ⇒ `segment.chapter_not_found`;
/// - đường đọc trượt ⇒ `store.read_failed`.
pub fn open_adjacent_chapter(
    open: Option<&mut OpenWork>,
    direction: ChapterDirection,
) -> Result<ChapterSwitch, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    let current = open.chapter_id;

    // 🔵 THEM 2026-08-21 (Story 3.4) — cung ly do da ghi o `read_open_chapter`: day cung la
    // mot duong MO CHUONG. Goi lap voi `read_open_chapter` khong ton gi (LazyLock chi chay
    // ham khoi tao dung mot lan, ~1us tu lan thu hai).
    crate::core::glossary::warm_jieba_for_source_lang(&open.meta.source_lang);

    // ⚠️ `ord` cua chinh Chuong dang mo — nua con lai cua bo doi. Doc trong CUNG mot lan
    // `read` voi luot tim hang ke: hai lan `read` la hai ket noi, va giua chung mot luot ghi
    // cua Epic 5 (sap lai Chuong) chen duoc vao.
    let found = open.store.read(move |conn| {
        let mut current_stmt = conn.prepare("SELECT ord FROM chapter WHERE id = ?1")?;
        let mut current_rows = current_stmt.query_map([current], |row| row.get::<_, i64>(0))?;
        let Some(current_ord) = current_rows.next().transpose()? else {
            // Hang dang mo bien mat ⇒ khong co "ke" nao co nghia. Bao bang mot `None` o
            // tang nay; tang tren doi no thanh mot loi CO TEN.
            return Ok(None);
        };

        let sql = match direction {
            ChapterDirection::Next => {
                "SELECT id, source_text FROM chapter \
                 WHERE ord > ?1 OR (ord = ?1 AND id > ?2) \
                 ORDER BY ord, id LIMIT 1"
            }
            ChapterDirection::Prev => {
                "SELECT id, source_text FROM chapter \
                 WHERE ord < ?1 OR (ord = ?1 AND id < ?2) \
                 ORDER BY ord DESC, id DESC LIMIT 1"
            }
        };

        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query_map((current_ord, current), |row| {
            let id: i64 = row.get(0)?;
            let source_text: String = row.get(1)?;
            Ok((id, source_text))
        })?;
        Ok(Some(rows.next().transpose()?))
    })?;

    let Some(neighbour) = found else {
        return Err(chapter_not_found(current));
    };

    let Some((chapter_id, source_text)) = neighbour else {
        // 🔴 KHONG quay vong. Cung luat, cung ly do da ghi bang chu o
        // `src/panels/segmentNavigation.ts:80-81` cho luot dieu huong CAU: mot luot quay vong
        // im lang dua nguoi dung ve dau ma khong dau hieu nao. AC4 doi mot bao hieu, va day
        // la cho no duoc sinh ra.
        return Ok(ChapterSwitch {
            outcome: match direction {
                ChapterDirection::Next => ChapterSwitchOutcome::AtLast,
                ChapterDirection::Prev => ChapterSwitchOutcome::AtFirst,
            },
            chapter: None,
        });
    };

    // 🔴 Con tro doi SAU khi truy van thanh cong, khong truoc. Dat truoc roi truy van truot
    // la de `OpenWork` tro vao mot Chuong ma webview chua bao gio nap.
    open.chapter_id = chapter_id;

    Ok(ChapterSwitch {
        outcome: ChapterSwitchOutcome::Moved,
        chapter: Some(OpenChapter {
            chapter_id,
            source_text,
            source_lang: open.meta.source_lang.clone(),
        }),
    })
}

/// Một hàng của danh sách Chương — **THÊM Story 5.7 (AC2)**. KHÔNG `source_text` (§Never
/// của story: Chương lớn nhất có thật là 48.640 ký tự, và 2.000 hàng như thế là một lượt
/// IPC vô nghĩa cho một màn hình chỉ cần liệt kê).
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChapterRow {
    pub chapter_id: i64,
    pub ord: i64,
    /// `NULL` ⇒ Chương chưa đặt tên — webview dựng nhãn từ `ord` qua
    /// `t('mode.library.chapter_untitled', { ord })` (§Always: "danh sách rỗng phải nói vì
    /// sao nó rỗng" áp dụng tương tự cho một HÀNG thiếu dữ kiện — không để trống im lặng).
    pub title: Option<String>,
    /// Trạng thái vòng đời (`chapter.status`), chuỗi tự do ở tầng SQL — cưỡng chế ở tầng
    /// Rust gọi nó (`commands::lifecycle`), cùng khuôn `IndexedWork::status`.
    pub status: String,
    /// Số segment **còn sống** (`retired_at IS NULL`) — cùng bộ lọc [`super::super::segment`]
    /// dùng cho lưới Editor, không đếm cả hàng đã về hưu.
    pub segment_count: i64,
}

/// **Liệt kê Chương của Tác phẩm đang mở** — hàm thuần, đây là thứ test gọi. Story 5.7,
/// AC2.
///
/// Sắp theo `(ord, id)` — cùng bộ đôi so sánh mà [`open_adjacent_chapter`] đã dùng, đúng
/// khuôn: `ord` không `UNIQUE` (schema.rs), nên khoá phụ `id` giữ thứ tự ỔN ĐỊNH khi hai
/// Chương trùng `ord`.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `work.none_open`;
/// - đường đọc trượt ⇒ `store.read_failed`.
pub fn list_chapters(open: Option<&OpenWork>) -> Result<Vec<ChapterRow>, IpcError> {
    let open = open.ok_or_else(no_work_open)?;

    let rows = open.store.read(|conn| {
        let mut stmt = conn.prepare(
            "SELECT c.id, c.ord, c.title, c.status, \
             (SELECT COUNT(*) FROM segment s WHERE s.chapter_id = c.id AND s.retired_at IS NULL) \
             FROM chapter c ORDER BY c.ord, c.id",
        )?;
        let mapped = stmt.query_map([], |row| {
            Ok(ChapterRow {
                chapter_id: row.get(0)?,
                ord: row.get(1)?,
                title: row.get(2)?,
                status: row.get(3)?,
                segment_count: row.get(4)?,
            })
        })?;
        mapped.collect::<crate::core::store::SqlResult<Vec<ChapterRow>>>()
    })?;

    Ok(rows)
}

/// **Mở một Chương đích danh** — hàm thuần, đây là thứ test gọi. Story 5.7, AC3.
///
/// 🔴 KIỂM hàng tồn tại TRƯỚC, dời `OpenWork::chapter_id` SAU khi truy vấn thành công —
/// đúng luật đã ghi ở [`open_adjacent_chapter`] (dòng *"Con trỏ đổi SAU khi truy vấn thành
/// công, không trước"*): đặt trước rồi truy vấn trượt là để `OpenWork` trỏ vào một Chương mà
/// webview chưa bao giờ nạp.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `work.none_open`;
/// - `chapter_id` không tồn tại ⇒ `segment.chapter_not_found` (tái dùng khoá đã có) — con
///   trỏ Chương **không đổi**;
/// - đường đọc trượt ⇒ `store.read_failed`.
pub fn open_chapter(
    open: Option<&mut OpenWork>,
    chapter_id: i64,
) -> Result<OpenChapter, IpcError> {
    let open = open.ok_or_else(no_work_open)?;

    let found = open.store.read(move |conn| {
        let mut stmt = conn.prepare("SELECT source_text FROM chapter WHERE id = ?1")?;
        let mut rows = stmt.query_map([chapter_id], |row| row.get::<_, String>(0))?;
        rows.next().transpose()
    })?;

    let Some(source_text) = found else {
        return Err(chapter_not_found(chapter_id));
    };

    // 🔵 THEM (Story 5.7) — cung ly do da ghi o `read_open_chapter`/`open_adjacent_chapter`:
    // day cung la mot duong MO CHUONG. Goi lap khong ton gi (LazyLock chi chay ham khoi tao
    // dung mot lan).
    crate::core::glossary::warm_jieba_for_source_lang(&open.meta.source_lang);

    // Con tro doi SAU khi truy van thanh cong — xem doc-comment cua ham nay.
    open.chapter_id = chapter_id;

    Ok(OpenChapter {
        chapter_id,
        source_text,
        source_lang: open.meta.source_lang.clone(),
    })
}

/// Một vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{ChapterDirection, ChapterRow, ChapterSwitch, IpcError, OpenChapter};
    use crate::commands::project::OpenWorkState;

    /// Vỏ IPC của [`super::read_open_chapter`].
    ///
    /// ⚠️ `try_state`, không `state()` — cùng lý do `commands::config::wire`: state có
    /// thể chưa từng được `app.manage` (lỗi cấu hình `setup()`), và `panic = "abort"` giết
    /// tiến trình nếu ta thẳng tay `.unwrap()`.
    #[tauri::command]
    pub fn read_open_chapter(app: tauri::AppHandle) -> Result<OpenChapter, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::read_open_chapter(None);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::read_open_chapter(guard.as_ref())
    }

    /// Vỏ IPC của [`super::open_adjacent_chapter`]. Story 2.11 · FR26.
    ///
    /// ⚠️ Tên tham số trên dây là **`direction`**, và `invoke()` gửi camelCase — một từ đơn
    /// nên hai chiều trùng nhau ở đây. Giá trị là `"next"` hoặc `"prev"`.
    ///
    /// ⚠️ Khoá `Mutex` giữ **qua** lời gọi, khác vỏ đọc ở trên: hàm thuần nhận `&mut` vì nó
    /// dời con trỏ Chương. Một lượt ghi giữa lúc đọc-rồi-ghi là đúng cuộc đua mà một
    /// `OpenWork` **trong** `Mutex` tồn tại để chặn.
    #[tauri::command]
    pub fn open_adjacent_chapter(
        app: tauri::AppHandle,
        direction: ChapterDirection,
    ) -> Result<ChapterSwitch, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::open_adjacent_chapter(None, direction);
        };
        let mut guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::open_adjacent_chapter(guard.as_mut(), direction)
    }

    /// Vỏ IPC của [`super::list_chapters`]. Story 5.7, AC2.
    #[tauri::command]
    pub fn list_chapters(app: tauri::AppHandle) -> Result<Vec<ChapterRow>, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::list_chapters(None);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::list_chapters(guard.as_ref())
    }

    /// Vỏ IPC của [`super::open_chapter`]. Story 5.7, AC3.
    ///
    /// ⚠️ `chapter_id` đi trên dây dưới tên **`chapterId`** — `invoke()` gửi tham số ở dạng
    /// camelCase.
    ///
    /// 🔴 `MutexGuard` giữ **XUYÊN SUỐT** lời gọi, cùng lý do [`open_adjacent_chapter`] ngay
    /// trên: hàm thuần nhận `&mut` vì nó dời con trỏ Chương.
    #[tauri::command]
    pub fn open_chapter(app: tauri::AppHandle, chapter_id: i64) -> Result<OpenChapter, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::open_chapter(None, chapter_id);
        };
        let mut guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::open_chapter(guard.as_mut(), chapter_id)
    }
}
