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

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::commands::project::OpenWork;
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::store::{SqlResult, Store, Transaction};

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
    fetch_chapter_rows(&open.store)
}

/// Câu SQL của [`list_chapters`], rút ra thành một hàm nhận thẳng `&Store` — **THÊM Story
/// 5.8**. `rename_chapter` gọi lại đúng câu này để trả `Vec<ChapterRow>` đã dựng lại (Task 6:
/// *"webview không phải đoán"*) mà không đúc một bản chép SQL thứ hai của cùng một câu.
fn fetch_chapter_rows(store: &Store) -> Result<Vec<ChapterRow>, IpcError> {
    let rows = store.read(|conn| {
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
        mapped.collect::<SqlResult<Vec<ChapterRow>>>()
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

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.8 — TỔ CHỨC LẠI CHƯƠNG SAU KHI NHẬP (FR15, AD-32)
// ═════════════════════════════════════════════════════════════════════════════════
// Bốn thao tác: đổi tên · dời lên/xuống · gộp vào Chương liền trước · tách tại một câu.
// AD-32 là mệnh đề nghiệm thu chính của cả bốn: gộp/tách đổi ĐÚNG HAI cột trên các hàng
// `segment` liên quan (`chapter_id` và `ord`) — không `retired_at`, không cột nào khác, và
// mọi hàng `segment_version` giữ nguyên từng byte (khác AD-5, gộp/tách SEGMENT).

/// Dời một Chương lên đã ở Chương đầu, hoặc gộp một Chương không có Chương liền trước —
/// cùng một sự thật cho hai lệnh khác nhau: *"không có hàng liền trước theo `(ord, id)`"*.
/// **0 hàng bị chạm.**
fn chapter_at_first(chapter_id: i64) -> IpcError {
    // ⚠️ `chapter_id` GIU o chu ky nhung KHONG di vao `params` — luot ra 2026-08-29. Ba cau nay
    // ban ra o mot thao tac THUONG NHAT (bam "Doi len" tren Chuong dau), va mot `chapter.id`
    // (`AUTOINCREMENT`, cuc bo trong tung `project.db`) khong khop mot con so nao nguoi dung
    // nhin thay tren man hinh -- danh sach hien `chapter.ord`, khong hien `id`. Tien le:
    // `MessageKey::LibraryRootInvalid` ("Khong tham so: duong dan cu the khong phai du lieu can
    // thiet cho cau nay"). Tham so giu o chu ky vi chuoi chan doan cua tang goi van dung no.
    let _ = chapter_id;
    IpcError::new("chapter.at_first", MessageKey::ChapterAtFirst, std::collections::BTreeMap::new(), false)
}

/// Dời một Chương xuống đã ở Chương cuối — không có hàng liền sau theo `(ord, id)`.
/// **0 hàng bị chạm.**
fn chapter_at_last(chapter_id: i64) -> IpcError {
    // ⚠️ `chapter_id` GIU o chu ky nhung KHONG di vao `params` — luot ra 2026-08-29. Ba cau nay
    // ban ra o mot thao tac THUONG NHAT (bam "Doi len" tren Chuong dau), va mot `chapter.id`
    // (`AUTOINCREMENT`, cuc bo trong tung `project.db`) khong khop mot con so nao nguoi dung
    // nhin thay tren man hinh -- danh sach hien `chapter.ord`, khong hien `id`. Tien le:
    // `MessageKey::LibraryRootInvalid` ("Khong tham so: duong dan cu the khong phai du lieu can
    // thiet cho cau nay"). Tham so giu o chu ky vi chuoi chan doan cua tang goi van dung no.
    let _ = chapter_id;
    IpcError::new("chapter.at_last", MessageKey::ChapterAtLast, std::collections::BTreeMap::new(), false)
}

/// Tách tại một câu để lại Chương đứng trước RỖNG — không hàng SỐNG nào còn đứng trước điểm
/// cắt trong Chương đó. **0 hàng bị chạm.** `chapter_id` là Chương sẽ bị để rỗng (Chương đang
/// mở, phía trước điểm cắt).
fn chapter_split_leaves_empty(chapter_id: i64) -> IpcError {
    // ⚠️ `chapter_id` GIU o chu ky nhung KHONG di vao `params` — luot ra 2026-08-29. Ba cau nay
    // ban ra o mot thao tac THUONG NHAT (bam "Doi len" tren Chuong dau), va mot `chapter.id`
    // (`AUTOINCREMENT`, cuc bo trong tung `project.db`) khong khop mot con so nao nguoi dung
    // nhin thay tren man hinh -- danh sach hien `chapter.ord`, khong hien `id`. Tien le:
    // `MessageKey::LibraryRootInvalid` ("Khong tham so: duong dan cu the khong phai du lieu can
    // thiet cho cau nay"). Tham so giu o chu ky vi chuoi chan doan cua tang goi van dung no.
    let _ = chapter_id;
    IpcError::new("chapter.split_leaves_empty", MessageKey::ChapterSplitLeavesEmpty, std::collections::BTreeMap::new(), false)
}

/// **Chuẩn hoá `chapter.ord` về `1..N`** theo `(ord, id)`, chạy ĐẦU mọi giao dịch tổ chức —
/// Task 5 của story.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO BƯỚC NÀY BẮT BUỘC, VÀ VÌ SAO `WHERE ord <> ?1`
/// ─────────────────────────────────────────────────────────────────────────────
/// `chapter.ord` cố ý KHÔNG `UNIQUE` và không hứa liên tục (`schema.rs::CHAPTER_DDL`) — đúng
/// khuôn `segment.ord`. Một phép `ord ± 1` trần trên một dãy thưa/trùng (`5, 5, 9`) trỏ sai
/// hàng hoặc thành no-op im lặng, đúng lớp lỗi mà `open_adjacent_chapter` đã phải viết so
/// sánh bộ đôi `(ord, id)` để tránh. Chuẩn hoá MỘT LẦN ở đầu mỗi giao dịch làm mọi phép còn
/// lại (hoán vị, chèn ở `ordA + 1`, tịnh tiến `± 1`) là số học AN TOÀN trên một dãy `1..N`
/// liên tục.
///
/// `WHERE ord <> ?1` chỉ `UPDATE` những hàng THẬT SỰ đổi — một Chương đã đúng vị trí không bị
/// đụng, nên `chapter.updated_at` của nó (KHÔNG có trong câu `UPDATE` này, xem bên dưới)
/// không nhảy oan. Chuẩn hoá là một phép SẮP LẠI, không phải một phép ghi nội dung — cùng
/// triết lý `ord` dời bằng phép tịnh tiến của segment (§Design Notes): nó không đổi
/// `updated_at`.
fn normalize_chapter_ord(tx: &Transaction<'_>) -> SqlResult<()> {
    let ids: Vec<i64> = {
        let mut stmt = tx.prepare("SELECT id FROM chapter ORDER BY ord, id")?;
        let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
        rows.collect::<SqlResult<Vec<i64>>>()?
    };

    let mut stmt = tx.prepare("UPDATE chapter SET ord = ?1 WHERE id = ?2 AND ord <> ?1")?;
    for (index, id) in ids.into_iter().enumerate() {
        // `index` bắt đầu từ 0 -- `ord` đánh số từ 1 (AD-3, tiền đề của Story 2.10).
        let new_ord = i64::try_from(index).unwrap_or(i64::MAX) + 1;
        stmt.execute((new_ord, id))?;
    }
    Ok(())
}

/// **Đổi tên một Chương** — hàm thuần, đây là thứ test gọi. Story 5.8, Task 6.
///
/// `str::trim()` của RUST, không `trim()` của SQLite (§Always: SQLite chỉ cắt dấu cách
/// ASCII, đo 2026-08-19 — một tên chỉ gồm U+3000 sẽ lọt xuống đĩa thành "có chữ" mà màn hình
/// hiện ra trống). Rỗng sau khi cắt ⇒ `NULL` — *chưa đặt tên*, một trạng thái hợp lệ, không
/// một lỗi.
///
/// Trả `Vec<ChapterRow>` đã dựng lại để webview không phải đoán (Task 6) — dù chỗ gọi hôm
/// nay (`libraryChapters.ts::renameCurrentChapter`) vẫn tự `loadChapters()` sau đó theo đúng
/// khuôn ba thao tác kia; giá trị trả về ở đây là để bề mặt IPC tự đủ, không bắt chỗ gọi
/// phải biết "gọi `list_chapters` lần hai để thấy tên mới".
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `work.none_open`;
/// - `chapter_id` không tồn tại ⇒ `segment.chapter_not_found` (tái dùng khoá đã có) — **0
///   hàng `segment` nào bị chạm**, và bản thân `chapter` cũng khớp 0 hàng.
pub fn rename_chapter(
    open: Option<&mut OpenWork>,
    chapter_id: i64,
    title: &str,
) -> Result<Vec<ChapterRow>, IpcError> {
    let open = open.ok_or_else(no_work_open)?;

    let trimmed = title.trim();
    let title_value: Option<String> = if trimmed.is_empty() { None } else { Some(trimmed.to_owned()) };

    let touched: usize = open.store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE chapter SET title = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') \
             WHERE id = ?2",
            (title_value, chapter_id),
        )
    })?;

    if touched == 0 {
        return Err(chapter_not_found(chapter_id));
    }

    crate::commands::lifecycle::write_lifecycle_after_change(open)?;
    fetch_chapter_rows(&open.store)
}

/// **Dời một Chương lên/xuống** — hoán vị `ord` với hàng liền kề — hàm thuần, đây là thứ test
/// gọi. Story 5.8, Task 7.
///
/// Tìm hàng kề bằng ĐÚNG câu SQL của [`open_adjacent_chapter`] (so sánh bộ đôi `(ord, id)`,
/// không `ord ± 1` trần — xem doc-comment của hàm đó). `ChapterDirection::Prev` = *dời lên*
/// (hoán vị với Chương liền TRƯỚC, biên là *"đã ở đầu"*); `ChapterDirection::Next` = *dời
/// xuống* (hoán vị với Chương liền SAU, biên là *"đã ở cuối"*) — cùng chiều ngữ nghĩa
/// `open_adjacent_chapter` đã dùng cho *"Chương trước/sau"*.
///
/// 🔴 **Chỉ cột `ord` đổi trên đúng hai hàng** (§I/O Matrix) — câu `UPDATE` của lượt hoán vị
/// KHÔNG mang `updated_at`: đây là một phép SẮP LẠI, không phải một phép ghi nội dung, cùng
/// lý lẽ đã ghi ở [`normalize_chapter_ord`].
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `work.none_open`;
/// - `chapter_id` không tồn tại ⇒ `segment.chapter_not_found` (tái dùng khoá đã có), **0 hàng
///   bị chạm** — xem khối 🔴 trong thân hàm cho phép đo đã bắt được lỗ này;
/// - đã ở biên (không có hàng kề theo hướng đã chọn) ⇒ `err.chapter.at_first` /
///   `err.chapter.at_last`, **0 hàng bị chạm** — không một `Ok` im lặng.
pub fn move_chapter(
    open: Option<&mut OpenWork>,
    chapter_id: i64,
    direction: ChapterDirection,
) -> Result<(), IpcError> {
    let open = open.ok_or_else(no_work_open)?;

    let at_boundary: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let at_boundary_in = Arc::clone(&at_boundary);
    let khong_ton_tai: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let khong_ton_tai_in = Arc::clone(&khong_ton_tai);

    let touched: usize = open.store.write(move |tx: &Transaction<'_>| {
        normalize_chapter_ord(tx)?;

        // 🔴 KIEM HANG TON TAI TRUOC MOI THU KHAC — luot ra 2026-08-29 bat duoc: khong co
        // khoi nay, `query_row` duoi day tra `QueryReturnedNoRows` cho mot `chapter_id` la, va
        // `Store::write` goi no thanh `StoreError::WriteFailed` ⇒ nguoi dung doc mot cau LOI
        // KHO ("khong ghi duoc kho du lieu") cho mot Tac pham hoan toan lanh lan. Do 2026-08-29
        // trước lượt vá: `move_chapter`/`merge_chapter_into_previous` tra `store.write_failed`,
        // trong khi `rename_chapter` ngay tren tra `segment.chapter_not_found`.
        //
        // ⚠️ Day DUNG lop loi ma Story 2.11 da sua MOT LAN cho `chapter_not_found` (xem
        // doc-comment cua ham do: mot cau SAI VE LOAI, khong tep nao hong). Mot lop loi da co
        // ten ma lai lot lai o hai ham moi la ly do khoi nay mang chu ky nay thay vi mot dong
        // `?` im lang.
        let ton_tai: i64 =
            tx.query_row("SELECT COUNT(*) FROM chapter WHERE id = ?1", [chapter_id], |row| row.get(0))?;
        if ton_tai == 0 {
            *khong_ton_tai_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
            return Ok(0);
        }

        let current_ord: i64 =
            tx.query_row("SELECT ord FROM chapter WHERE id = ?1", [chapter_id], |row| row.get(0))?;

        // Cùng câu SQL của `open_adjacent_chapter` -- Task 7 đòi đúng chữ.
        let sql = match direction {
            ChapterDirection::Next => {
                "SELECT id, ord FROM chapter \
                 WHERE ord > ?1 OR (ord = ?1 AND id > ?2) \
                 ORDER BY ord, id LIMIT 1"
            }
            ChapterDirection::Prev => {
                "SELECT id, ord FROM chapter \
                 WHERE ord < ?1 OR (ord = ?1 AND id < ?2) \
                 ORDER BY ord DESC, id DESC LIMIT 1"
            }
        };

        let neighbour: Option<(i64, i64)> = {
            let mut stmt = tx.prepare(sql)?;
            let mut rows = stmt.query_map((current_ord, chapter_id), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })?;
            rows.next().transpose()?
        };

        let Some((neighbour_id, neighbour_ord)) = neighbour else {
            *at_boundary_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
            return Ok(0);
        };

        // 🔴 Chỉ cột `ord` đổi -- xem doc-comment cua ham nay.
        tx.execute("UPDATE chapter SET ord = ?1 WHERE id = ?2", (neighbour_ord, chapter_id))?;
        tx.execute("UPDATE chapter SET ord = ?1 WHERE id = ?2", (current_ord, neighbour_id))?;
        Ok(2)
    })?;

    if touched == 0 {
        if *khong_ton_tai.lock().unwrap_or_else(std::sync::PoisonError::into_inner) {
            return Err(chapter_not_found(chapter_id));
        }
        if *at_boundary.lock().unwrap_or_else(std::sync::PoisonError::into_inner) {
            return Err(match direction {
                ChapterDirection::Next => chapter_at_last(chapter_id),
                ChapterDirection::Prev => chapter_at_first(chapter_id),
            });
        }
    }

    crate::commands::lifecycle::write_lifecycle_after_change(open)?;
    Ok(())
}

/// **Gộp một Chương vào Chương liền trước nó** — hàm thuần, đây là thứ test gọi. Story 5.8,
/// Task 8, và AD-32 là mệnh đề nghiệm thu chính của nó.
///
/// Mọi hàng `segment` của Chương bị gộp (B) -- SỐNG và VỀ HƯU -- đổi `chapter_id` sang Chương
/// đích (A) và `ord` tịnh tiến bằng `MAX(ord)` hiện có của A (`0` nếu A chưa có segment nào,
/// đúng 25 Chương Epic 1 chưa từng tách câu) -- **không mệnh đề `retired_at`**, không hàng nào
/// bị bỏ lại. `A.source_text` nối THÔ với `B.source_text` (xem §Design Notes: đây là đường
/// DUY NHẤT không mất byte nào và đúng cả khi A/B chưa từng tách segment).
///
/// Trạng thái vòng đời: `done` chỉ sống sót khi CẢ HAI nửa là `done`; mọi ca khác giữ nguyên
/// `status` của A, hạ `done` xuống `in_progress` -- không bao giờ khai `done` cho văn bản
/// chưa ai xác nhận (§Design Notes "Vì sao gộp done + chưa xong ra in_progress").
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `work.none_open`;
/// - `chapter_id` không tồn tại ⇒ `segment.chapter_not_found` (tái dùng khoá đã có), **0 hàng
///   bị chạm** — cùng phép đo đã ghi ở [`move_chapter`];
/// - không có Chương liền trước ⇒ `err.chapter.at_first`, **0 hàng bị chạm**.
pub fn merge_chapter_into_previous(
    open: Option<&mut OpenWork>,
    chapter_id: i64,
) -> Result<(), IpcError> {
    let open = open.ok_or_else(no_work_open)?;

    let no_previous: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let no_previous_in = Arc::clone(&no_previous);
    let merged_into: Arc<Mutex<Option<i64>>> = Arc::new(Mutex::new(None));
    let merged_into_in = Arc::clone(&merged_into);
    let khong_ton_tai: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let khong_ton_tai_in = Arc::clone(&khong_ton_tai);

    let touched: usize = open.store.write(move |tx: &Transaction<'_>| {
        normalize_chapter_ord(tx)?;

        // 🔴 KIEM HANG TON TAI TRUOC MOI THU KHAC — luot ra 2026-08-29 bat duoc: khong co
        // khoi nay, `query_row` duoi day tra `QueryReturnedNoRows` cho mot `chapter_id` la, va
        // `Store::write` goi no thanh `StoreError::WriteFailed` ⇒ nguoi dung doc mot cau LOI
        // KHO ("khong ghi duoc kho du lieu") cho mot Tac pham hoan toan lanh lan. Do 2026-08-29
        // trước lượt vá: `move_chapter`/`merge_chapter_into_previous` tra `store.write_failed`,
        // trong khi `rename_chapter` ngay tren tra `segment.chapter_not_found`.
        //
        // ⚠️ Day DUNG lop loi ma Story 2.11 da sua MOT LAN cho `chapter_not_found` (xem
        // doc-comment cua ham do: mot cau SAI VE LOAI, khong tep nao hong). Mot lop loi da co
        // ten ma lai lot lai o hai ham moi la ly do khoi nay mang chu ky nay thay vi mot dong
        // `?` im lang.
        let ton_tai: i64 =
            tx.query_row("SELECT COUNT(*) FROM chapter WHERE id = ?1", [chapter_id], |row| row.get(0))?;
        if ton_tai == 0 {
            *khong_ton_tai_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
            return Ok(0);
        }

        let b_ord: i64 =
            tx.query_row("SELECT ord FROM chapter WHERE id = ?1", [chapter_id], |row| row.get(0))?;

        // "Liền trước" -- ĐÚNG câu so sánh bộ đôi mà `open_adjacent_chapter`/`move_chapter`
        // dùng cho hướng Prev.
        let previous: Option<(i64, String)> = {
            let mut stmt = tx.prepare(
                "SELECT id, status FROM chapter \
                 WHERE ord < ?1 OR (ord = ?1 AND id < ?2) \
                 ORDER BY ord DESC, id DESC LIMIT 1",
            )?;
            let mut rows = stmt.query_map((b_ord, chapter_id), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?;
            rows.next().transpose()?
        };

        let Some((a_id, a_status)) = previous else {
            *no_previous_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
            return Ok(0);
        };

        let b_status: String =
            tx.query_row("SELECT status FROM chapter WHERE id = ?1", [chapter_id], |row| row.get(0))?;

        // `COALESCE(MAX(ord), 0)` -- A chưa có segment nào (25 Chương Epic 1) ⇒ shift = 0,
        // và hàng của B đi vào A giữ nguyên `ord` tương đối của chúng.
        let shift: i64 = tx.query_row(
            "SELECT COALESCE(MAX(ord), 0) FROM segment WHERE chapter_id = ?1",
            [a_id],
            |row| row.get(0),
        )?;

        // 🔴 Mọi hàng -- SỐNG và VỀ HƯU -- đi cùng khối, không mệnh đề `retired_at` nào lọc
        // bớt (§Always: đây là điểm khác AD-5 cố ý).
        tx.execute(
            "UPDATE segment SET chapter_id = ?1, ord = ord + ?2 WHERE chapter_id = ?3",
            (a_id, shift, chapter_id),
        )?;

        // §Design Notes "Vì sao gộp done + chưa xong ra in_progress".
        //
        // 🔴 Qua `LifecycleStatus::…as_str()`, KHÔNG hai chuỗi viết thẳng. §Verification của
        // Story 5.4 khai luật đó bằng chữ (*"mọi lần xuất hiện ở vị trí mã nằm trong
        // core/lifecycle/mod.rs; chỗ khác chỉ được nhắc qua LifecycleStatus::…"*), và
        // ⚠️ **KHÔNG cổng nào canh nó** — đo 2026-08-29: `grep '"done"\|"in_progress"\|
        // "not_started"\|"paused"'` trên `src-tauri/src/**` ngoài `core/lifecycle/mod.rs`
        // cho **0** kết quả ở vị trí mã, nên hai chuỗi viết thẳng ở đây sẽ là hai chỗ DUY
        // NHẤT phải giữ khớp bằng kỷ luật, và chúng trôi trong im lặng ngày giá trị trên dây
        // đổi. `LifecycleStatus` là danh mục ĐÓNG sinh từ một khai báo (`lifecycle_statuses!`)
        // nên một lượt đổi ở đó kéo theo chỗ này qua trình biên dịch.
        let done = crate::core::lifecycle::LifecycleStatus::Done.as_str();
        let merged_status: String = if a_status == done && b_status != done {
            crate::core::lifecycle::LifecycleStatus::InProgress.as_str().to_owned()
        } else {
            a_status
        };

        // Nối THÔ -- B vẫn còn hàng ở đây, `SELECT` con đọc được `source_text` của nó TRƯỚC
        // khi bị xoá bên dưới.
        tx.execute(
            "UPDATE chapter SET source_text = source_text || char(10) || char(10) || \
             (SELECT source_text FROM chapter WHERE id = ?2), status = ?3, \
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?1",
            (a_id, chapter_id, merged_status),
        )?;

        tx.execute("DELETE FROM chapter_position WHERE chapter_id = ?1", [chapter_id])?;
        tx.execute("DELETE FROM chapter WHERE id = ?1", [chapter_id])?;
        // Chương SAU B tịnh tiến `ord - 1` -- một phép SẮP LẠI, không `updated_at`.
        tx.execute("UPDATE chapter SET ord = ord - 1 WHERE ord > ?1", [b_ord])?;

        *merged_into_in
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(a_id);
        Ok(1)
    })?;

    if touched == 0 {
        if *khong_ton_tai.lock().unwrap_or_else(std::sync::PoisonError::into_inner) {
            return Err(chapter_not_found(chapter_id));
        }
        if *no_previous.lock().unwrap_or_else(std::sync::PoisonError::into_inner) {
            return Err(chapter_at_first(chapter_id));
        }
    }

    // 🔴 Con trỏ đổi SAU khi giao dịch commit, không trước (luật đã ghi ở
    // `open_adjacent_chapter`). Chương B không còn tồn tại -- nếu B đang mở, con trỏ dời
    // sang A.
    if let Some(a_id) = *merged_into.lock().unwrap_or_else(std::sync::PoisonError::into_inner) {
        if open.chapter_id == chapter_id {
            open.chapter_id = a_id;
        }
    }

    crate::commands::lifecycle::write_lifecycle_after_change(open)?;
    Ok(())
}

/// **Tách Chương đang mở tại một câu** — hàm thuần, đây là thứ test gọi. Story 5.8, Task 9.
/// Điểm tách sống ở Editor (`editorCaretSegmentId`), nên hàm này làm việc trên
/// `open.chapter_id`, KHÔNG nhận một `chapter_id` riêng — `segment_id` phải thuộc đúng Chương
/// ĐANG MỞ.
///
/// Chương mới (B) chèn ngay sau Chương đang mở (A); mọi hàng `segment` TẠI và SAU `(ord, id)`
/// của `s` -- SỐNG và VỀ HƯU -- đổi sang B, `ord` tịnh tiến để B đếm lại từ 1. `status` của B
/// chép từ A; `title` của B luôn `NULL` (một Chương mới tách chưa từng được ai đặt tên).
/// `source_text` của CẢ HAI Chương dựng lại từ phép nối `source_text` của segment CÒN SỐNG,
/// phân tách bằng `"\n"` (§Design Notes "Vì sao `source_text` của lượt TÁCH dựng lại từ
/// segment" -- không đường nào giữ nguyên byte ở lượt tách, và đây là phương án DUY NHẤT
/// không cần hai bản cài đặt của cùng một quy tắc).
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `work.none_open`;
/// - `segment_id` không tồn tại, đã VỀ HƯU, hoặc không thuộc Chương đang mở ⇒
///   `segment.not_found` (tái dùng [`crate::commands::segment::segment_not_found`]), **0
///   hàng bị chạm**;
/// - `s` là câu ĐẦU Chương (không còn hàng SỐNG nào đứng trước nó) ⇒
///   `err.chapter.split_leaves_empty`, **0 hàng bị chạm** -- một Chương rỗng không phải một
///   kết quả có nghĩa.
pub fn split_chapter_at_segment(
    open: Option<&mut OpenWork>,
    segment_id: i64,
) -> Result<(), IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    let chapter_a = open.chapter_id;

    #[derive(Clone, Copy)]
    enum SplitRefusal {
        SegmentNotFound,
        LeavesEmpty,
    }
    let refusal: Arc<Mutex<Option<SplitRefusal>>> = Arc::new(Mutex::new(None));
    let refusal_in = Arc::clone(&refusal);

    let touched: usize = open.store.write(move |tx: &Transaction<'_>| {
        // ① `segment_id` phải là một hàng SỐNG của ĐÚNG Chương đang mở -- một segment về hưu
        // hoặc của Chương khác đọc lên GIỐNG HỆT "không tồn tại" từ góc nhìn của lệnh này.
        let found: Option<(i64, i64)> = {
            let mut stmt =
                tx.prepare("SELECT chapter_id, ord FROM segment WHERE id = ?1 AND retired_at IS NULL")?;
            let mut rows = stmt.query_map([segment_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })?;
            rows.next().transpose()?
        };

        let Some((seg_chapter_id, seg_ord)) = found else {
            *refusal_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(SplitRefusal::SegmentNotFound);
            return Ok(0);
        };
        if seg_chapter_id != chapter_a {
            *refusal_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(SplitRefusal::SegmentNotFound);
            return Ok(0);
        }

        // ② Không hàng SỐNG nào đứng trước `s` trong A ⇒ Chương A sẽ RỖNG sau lượt tách.
        let living_before: i64 = tx.query_row(
            "SELECT COUNT(*) FROM segment WHERE chapter_id = ?1 AND retired_at IS NULL \
             AND (ord < ?2 OR (ord = ?2 AND id < ?3))",
            (chapter_a, seg_ord, segment_id),
            |row| row.get(0),
        )?;
        if living_before == 0 {
            *refusal_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(SplitRefusal::LeavesEmpty);
            return Ok(0);
        }

        normalize_chapter_ord(tx)?;

        let (ord_a, status_a): (i64, String) = tx.query_row(
            "SELECT ord, status FROM chapter WHERE id = ?1",
            [chapter_a],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        tx.execute(
            "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
             VALUES (?1, NULL, '', ?2, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
             strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (ord_a + 1, status_a),
        )?;
        let chapter_b = tx.last_insert_rowid();

        // Chương sau A (trừ B vừa chèn) tịnh tiến `+1` -- một phép SẮP LẠI, không `updated_at`.
        tx.execute(
            "UPDATE chapter SET ord = ord + 1 WHERE ord > ?1 AND id <> ?2",
            (ord_a, chapter_b),
        )?;

        // 🔴 Mọi hàng TẠI và SAU `(ord, id)` của `s` -- SỐNG và VỀ HƯU -- đổi sang B, không
        // mệnh đề `retired_at` nào lọc bớt. `seg_ord` đọc TRƯỚC chuẩn hoá vẫn đúng: chuẩn hoá
        // chỉ đổi `chapter.ord`, không đổi `segment.ord`.
        tx.execute(
            "UPDATE segment SET chapter_id = ?1, ord = ord - (?2 - 1) \
             WHERE chapter_id = ?3 AND (ord > ?2 OR (ord = ?2 AND id >= ?4))",
            (chapter_b, seg_ord, chapter_a, segment_id),
        )?;

        // Hàng vị trí ĐÃ DỜI theo câu -- xác định "đã dời" bằng cách đọc lại `chapter_id`
        // SAU lượt UPDATE ngay trên, không bằng một tập `id` tính tay ở Rust.
        tx.execute(
            "UPDATE chapter_position SET chapter_id = ?1 \
             WHERE chapter_id = ?2 AND segment_id IN \
             (SELECT id FROM segment WHERE chapter_id = ?1)",
            (chapter_b, chapter_a),
        )?;

        // Dựng lại `source_text` của CẢ HAI Chương từ segment CÒN SỐNG, nối bằng "\n" --
        // §Design Notes.
        for id in [chapter_a, chapter_b] {
            let text: String = {
                let mut stmt = tx.prepare(
                    "SELECT source_text FROM segment WHERE chapter_id = ?1 AND retired_at IS NULL \
                     ORDER BY ord, id",
                )?;
                let parts = stmt
                    .query_map([id], |row| row.get::<_, String>(0))?
                    .collect::<SqlResult<Vec<String>>>()?;
                parts.join("\n")
            };
            tx.execute(
                "UPDATE chapter SET source_text = ?1, \
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?2",
                (text, id),
            )?;
        }

        Ok(1)
    })?;

    if touched == 0 {
        return Err(match *refusal.lock().unwrap_or_else(std::sync::PoisonError::into_inner) {
            Some(SplitRefusal::SegmentNotFound) => {
                crate::commands::segment::segment_not_found(segment_id)
            }
            Some(SplitRefusal::LeavesEmpty) => chapter_split_leaves_empty(chapter_a),
            // Không đường nào tới đây: `touched == 0` LUÔN đi kèm một trong hai nhánh từ chối
            // ở trên đã đặt `refusal`. Rơi về đây là một lỗi LẬP TRÌNH, không một ca nghiệp vụ
            // -- `Unknown` đúng vai của nó (xem doc-comment `IpcError::new`).
            None => IpcError::new(
                "chapter.split_internal_error",
                MessageKey::Unknown,
                BTreeMap::new(),
                false,
            ),
        });
    }

    // 🔴 `open.chapter_id` GIỮ NGUYÊN A -- Chương đang mở không đổi định danh sau một lượt
    // tách; nó chỉ mất bớt vài câu cuối sang B.
    crate::commands::lifecycle::write_lifecycle_after_change(open)?;
    Ok(())
}

// ── Bốn hàm `*_indexed` -- khuôn `commands::lifecycle::set_chapter_status_indexed` -- cộng
// bước 4, dùng bởi `tests/**` để chứng minh `library_work` theo kịp SAU một lượt tổ chức. ────

/// [`rename_chapter`] cộng bước 4.
pub fn rename_chapter_indexed(
    open: Option<&mut OpenWork>,
    indexer: Option<&crate::core::library::indexer::Indexer>,
    global: Option<&Store>,
    root: &std::path::Path,
    chapter_id: i64,
    title: &str,
) -> Result<Vec<ChapterRow>, IpcError> {
    crate::commands::lifecycle::finish_lifecycle_write(
        rename_chapter(open, chapter_id, title),
        indexer,
        global,
        root,
    )
}

/// [`move_chapter`] cộng bước 4.
pub fn move_chapter_indexed(
    open: Option<&mut OpenWork>,
    indexer: Option<&crate::core::library::indexer::Indexer>,
    global: Option<&Store>,
    root: &std::path::Path,
    chapter_id: i64,
    direction: ChapterDirection,
) -> Result<(), IpcError> {
    crate::commands::lifecycle::finish_lifecycle_write(
        move_chapter(open, chapter_id, direction),
        indexer,
        global,
        root,
    )
}

/// [`merge_chapter_into_previous`] cộng bước 4.
pub fn merge_chapter_into_previous_indexed(
    open: Option<&mut OpenWork>,
    indexer: Option<&crate::core::library::indexer::Indexer>,
    global: Option<&Store>,
    root: &std::path::Path,
    chapter_id: i64,
) -> Result<(), IpcError> {
    crate::commands::lifecycle::finish_lifecycle_write(
        merge_chapter_into_previous(open, chapter_id),
        indexer,
        global,
        root,
    )
}

/// [`split_chapter_at_segment`] cộng bước 4.
pub fn split_chapter_at_segment_indexed(
    open: Option<&mut OpenWork>,
    indexer: Option<&crate::core::library::indexer::Indexer>,
    global: Option<&Store>,
    root: &std::path::Path,
    segment_id: i64,
) -> Result<(), IpcError> {
    crate::commands::lifecycle::finish_lifecycle_write(
        split_chapter_at_segment(open, segment_id),
        indexer,
        global,
        root,
    )
}

/// Một vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{ChapterDirection, ChapterRow, ChapterSwitch, IpcError, OpenChapter};
    use crate::commands::project::OpenWorkState;
    use crate::core::library::indexer::Indexer;
    use crate::core::store::Store;

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

    // ═════════════════════════════════════════════════════════════════════════════════
    // 🔴 STORY 5.8 — bốn vỏ tổ chức lại Chương, cộng `finish_with_reindex` dùng chung
    // ═════════════════════════════════════════════════════════════════════════════════
    // 🔴 `#[tauri::command(async)]` bắt buộc cho cả bốn — bước 4 (`finish_with_reindex`) quét
    // TOÀN BỘ thư mục gốc Library, cùng lý do đã ghi ở `commands::lifecycle::wire`. Cổng canh:
    // `config_invariants.rs::the_blocking_wires_run_off_the_main_thread`.

    /// Vỏ IPC của [`super::rename_chapter`]. Story 5.8.
    ///
    /// ⚠️ `chapter_id`/`title` đi trên dây dưới tên **`chapterId`**/`title` — `invoke()` gửi
    /// tham số ở dạng camelCase, và `title` là một từ đơn nên hai chiều trùng nhau ở đây.
    #[tauri::command(async)]
    pub fn rename_chapter(
        app: tauri::AppHandle,
        chapter_id: i64,
        title: String,
    ) -> Result<Vec<ChapterRow>, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::rename_chapter(None, chapter_id, &title);
        };
        // 🔴 Khoá `OpenWorkState` NHẢ trước bước 4 — cùng khuôn `commands::lifecycle::wire`:
        // giữ nó qua một lượt quét đĩa chặn mọi lệnh khác đọc Tác phẩm đang mở.
        let result = {
            let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            super::rename_chapter(guard.as_mut(), chapter_id, &title)
        };
        finish_with_reindex(&app, result)
    }

    /// Vỏ IPC của [`super::move_chapter`]. Story 5.8.
    ///
    /// ⚠️ `chapter_id`/`direction` đi trên dây dưới tên **`chapterId`**/`direction`.
    #[tauri::command(async)]
    pub fn move_chapter(
        app: tauri::AppHandle,
        chapter_id: i64,
        direction: ChapterDirection,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::move_chapter(None, chapter_id, direction);
        };
        let result = {
            let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            super::move_chapter(guard.as_mut(), chapter_id, direction)
        };
        finish_with_reindex(&app, result)
    }

    /// Vỏ IPC của [`super::merge_chapter_into_previous`]. Story 5.8.
    ///
    /// ⚠️ `chapter_id` đi trên dây dưới tên **`chapterId`**.
    #[tauri::command(async)]
    pub fn merge_chapter_into_previous(
        app: tauri::AppHandle,
        chapter_id: i64,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::merge_chapter_into_previous(None, chapter_id);
        };
        let result = {
            let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            super::merge_chapter_into_previous(guard.as_mut(), chapter_id)
        };
        finish_with_reindex(&app, result)
    }

    /// Vỏ IPC của [`super::split_chapter_at_segment`]. Story 5.8.
    ///
    /// ⚠️ `segment_id` đi trên dây dưới tên **`segmentId`**. Điểm tách sống ở Editor, nên
    /// hàm này KHÔNG nhận `chapterId` — Chương làm việc trên chính là `OpenWork::chapter_id`.
    #[tauri::command(async)]
    pub fn split_chapter_at_segment(
        app: tauri::AppHandle,
        segment_id: i64,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::split_chapter_at_segment(None, segment_id);
        };
        let result = {
            let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            super::split_chapter_at_segment(guard.as_mut(), segment_id)
        };
        finish_with_reindex(&app, result)
    }

    /// Bước 4 cho cả bốn vỏ ở trên — khuôn `commands::lifecycle::wire::finish_with_reindex`,
    /// chép TẠI ĐÂY vì mỗi tệp `wire` tự giải quyết `State` của chính nó (spec Task 11: "cộng
    /// `finish_with_reindex` riêng của tệp"). **Không một quy tắc nào sống ở đây** — hàm này
    /// chỉ biết cách lấy `Indexer`/`Store`/`root` ra khỏi `AppHandle` rồi giao TRỌN quyết định
    /// cho [`crate::commands::lifecycle::finish_lifecycle_write`].
    fn finish_with_reindex<T>(
        app: &tauri::AppHandle,
        result: Result<T, IpcError>,
    ) -> Result<T, IpcError> {
        use tauri::Manager as _;

        let store = app.try_state::<Store>();
        let root = match crate::commands::project::resolve_library_root(app, store.as_deref()) {
            Ok(root) => root,
            Err(err) => {
                eprintln!("chapter[reindex] khong giai quyet duoc thu muc goc Library: {err:?}");
                return result;
            }
        };
        let indexer = app.try_state::<Indexer>();
        crate::commands::lifecycle::finish_lifecycle_write(result, indexer.as_deref(), store.as_deref(), &root)
    }
}
