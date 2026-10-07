use super::*;

/// **Một hàng lịch sử phiên bản** — thứ đi ra qua dây cho màn hình FR101.
///
/// Story 2.6, AC1 · AC5.
///
/// 🔴 **KHÔNG** `#[serde(rename_all = "camelCase")]`, và đây là một thói quen viết Tauri phải
/// cưỡng lại chứ không một lượt quên. Bốn tên trường dưới đây là **dây**: `src/config/segment.ts`
/// đọc đúng bốn cái tên `snake_case` này. ⚠️ Chiều ngược lại thì khác hẳn — `invoke()` gửi
/// **tham số** ở dạng camelCase (`segmentId`) dù hàm Rust nhận `snake_case`. Hai chiều khác
/// nhau là chỗ dễ sai nhất trên dây, và `src/config/segment.ts` là chỗ duy nhất gõ cả hai.
///
/// ⚠️ Bốn trường, đúng bốn cột của bảng — [`crate::core::store::schema::SEGMENT_STATUS_AND_VERSION_DDL`].
/// **Không** cột xuất xứ *(FR117, Story 2.7)*, **không** cột cặp TM *(FR56, Epic 7)*: cả hai
/// được khai bằng chữ ở doc-comment của DDL là **cố ý chưa thêm**. Quyết định #5 đường (a)
/// (Ice ký 2026-08-16) giữ nguyên lằn ranh đó — màn hình hiện **thời điểm** và không nhãn nào,
/// vì bốn nhãn của mockup trỏ vào bốn năng lực chưa dựng.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SegmentVersionRow {
    pub id: i64,
    pub segment_id: i64,
    pub target_text: String,
    pub created_at: String,
}

/// **Lịch sử phiên bản của một segment**, mới nhất trước — **hàm thuần, đây là thứ test gọi**.
/// Story 2.6 · FR101 · AC1 · AC3 · AC4 · AC5.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - `segment_id` không tồn tại ⇒ `segment.not_found`.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 ĐƯỜNG NÀY **KHÔNG** TỪ CHỐI MỘT SEGMENT ĐÃ VỀ HƯU — AC4, và nó ngược ba lệnh kia
/// ─────────────────────────────────────────────────────────────────────────────
/// Ba lệnh ghi hiện có ([`confirm_segment`], [`set_segment_omitted`],
/// [`set_segment_paragraph_end`]) đều trả `MessageKey::SegmentRetired` khi `retired_at` khác
/// `NULL`. Đúng — **vì chúng ghi**. Một segment về hưu là một tombstone (AD-5), và ghi lên một
/// tombstone là sửa lịch sử.
///
/// Lượt này **đọc**, và AC4 nói thẳng: *"segment đã về hưu do gộp hoặc tách ⇒ lịch sử vẫn tra
/// lại được"*. ⇒ Sao chép phép từ chối của ba lệnh kia sang đây là **phá AC4**, và nó sẽ trông
/// rất giống một lượt "cho nhất quán". Quyết định #8 đường (a) (Ice ký 2026-08-16).
///
/// ⚠️ **Vế còn hở, ghi ra thay vì tự chấm đạt:** hôm nay **không đường mã nào** cho một segment
/// về hưu — `retired_at` là `None` cho mọi hàng, và Story 2.8 *(gộp/tách tường minh)* là
/// `backlog`. Ca hợp đồng dựng trạng thái đó bằng SQL trực tiếp. Vế **bề mặt vào** ghi nợ, chủ
/// **Story 2.8**.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 RỖNG **CÓ LÝ DO** ≠ RỖNG **IM LẶNG** — AC3
/// ─────────────────────────────────────────────────────────────────────────────
/// Một segment **chưa từng được xác nhận** trả về một danh sách **rỗng**, không một lỗi và
/// không `null`. Nhưng nó phải **phân biệt được** với *"không tìm thấy segment"* — đó là lý do
/// hàm hỏi sự tồn tại của segment **trước**, bằng một truy vấn riêng, thay vì suy từ việc
/// `segment_version` trả 0 hàng.
///
/// ⚠️ Suy từ 0 hàng là đúng lớp lỗi trung tâm của dự án *(`project-context.md:473`)*: một
/// `segment_id` gõ sai và một câu chưa ai ký cho **cùng một** kết quả rỗng trong 0,01 ms, không
/// lỗi nào ném ra, và triệu chứng là *"lịch sử không hiện gì"* — không ai lần được nguyên nhân.
/// Hai truy vấn ở đây mua đúng một phép phân biệt đó.
///
/// 🔴 Và vế **giao diện** của AC3 nằm ở chỗ khác, không ở đây: trạng thái rỗng phải **nói ra cơ
/// chế** *("lịch sử sinh ra khi xác nhận, không phải khi gõ")*. Tầng này chỉ chịu trách nhiệm
/// làm cái rỗng đó **phân biệt được**.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ `ORDER BY created_at DESC` VÀ INDEX CỦA BƯỚC 10 LÀ MỘT CẶP
/// ─────────────────────────────────────────────────────────────────────────────
/// `idx_segment_version_segment_created` *(`segment_id, created_at DESC`, bước di trú 10)* dựng
/// **cùng lượt với hàm này** — đó là món nợ có chủ mà bước 7 ghi bằng chữ lúc dựng bảng. Đổi
/// mệnh đề `ORDER BY` ở đây mà không đổi index là làm index thành vô dụng trong im lặng.
///
/// ⚠️ **`created_at` chứ không phải `segment.updated_at`** — hai mốc **không nói cùng một
/// chuyện**, và đây là món nợ thứ hai có chủ 2.6 *(`deferred-work.md §*Deferred from: 2-3-hop-dong-flush-va-trang-thai-da-luu (2026-08-12)*`)*, đã **đọc mã
/// xác nhận lại** chứ không chép: `segment.updated_at` do [`save_segment_targets`] sinh và mang
/// nghĩa *"mốc sửa **văn bản**"*; `segment_version.created_at` là **mốc ký**, sinh trong SQL bằng
/// `strftime('%Y-%m-%dT%H:%M:%fZ','now')` ngay trong [`confirm_segment`]. Một lượt ký **không**
/// sửa một ký tự nào nên nó **không** đụng `updated_at` — đã xác nhận lại 2026-08-16: câu
/// `UPDATE` của `confirm_segment` không có `updated_at`.
/// 🔵 **Sửa tại chỗ 2026-08-16 (Story 2.7):** dòng trên trước đó **trích nguyên văn** câu SQL
/// (`SET status = ?1 WHERE id = ?2`), và câu đó nay là `SET status = ?1, translation_origin =
/// ?2 WHERE id = ?3`. Mệnh đề *"không có `updated_at`"* **vẫn đúng**; thứ hết đúng là bản
/// trích. ⇒ Bỏ bản trích chứ không sửa nó: một doc-comment chép một câu SQL của **hàm khác**
/// là một bản sao sẽ mục lại ở mỗi story chạm hàm đó, và không cổng nào canh.
/// ⇒ Màn hình lịch sử đọc `created_at`. Dùng `updated_at` sẽ cho một danh sách mà **mọi hàng
/// mang cùng một mốc**, và mốc đó là lần gõ cuối chứ không phải lần ký.
pub fn read_segment_history(
    open: Option<&OpenWork>,
    segment_id: i64,
) -> Result<Vec<SegmentVersionRow>, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    // ① Segment co ton tai khong? Mot truy van RIENG, va no la thu lam "rong" phan biet duoc
    //    voi "khong tim thay" (AC3). KHONG hoi `retired_at` -- duong DOC khong tu choi mot
    //    tombstone (AC4), nen cot do khong lien quan o day.
    let exists: bool = open
        .store
        .read(move |conn| {
            conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM segment WHERE id = ?1)",
                [segment_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|n| n != 0)
        })
        .map_err(IpcError::from)?;

    if !exists {
        return Err(segment_not_found(segment_id));
    }

    // ② Lich su. Rong la mot cau tra loi HOP LE o day -- mot segment chua tung duoc ky.
    //
    // ⚠️ `, id DESC` la mot PHEP GO HOA, khong mot dong trang tri. `created_at` sinh bang
    //    `strftime('%Y-%m-%dT%H:%M:%fZ','now')` -- chinh xac toi MILI GIAY, va hai luot ky
    //    trong cung mot mili giay cho HAI hang bang nhau o cot sap. SQLite khong bao dam
    //    thu tu cua cac hang bang nhau, nen thieu ve nay thi thu tu hien thi cua hai hang do
    //    la KHONG TAT DINH -- doi giua hai luot doc, tren cung mot du lieu.
    //    `id` la AUTOINCREMENT nen don dieu ⇒ no la thu tu ky THAT, va no go hoa duoc moi ca.
    // ⚠️ Ve nay KHONG nam trong index cua buoc 10 (`segment_id, created_at DESC`). Co y:
    //    index phuc vu phep LOC va phep SAP CHINH; mot cot thu ba chi de go hoa cho vai hang
    //    trung mili giay la mot cai gia thuong truc cho mot ca hiem. SQLite sap not phan con
    //    lai trong bo nho, tren mot tap DA duoc index thu hep ve mot `segment_id`.
    let rows = open.store.read(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT id, segment_id, target_text, created_at FROM segment_version \
             WHERE segment_id = ?1 ORDER BY created_at DESC, id DESC",
        )?;
        let rows = stmt.query_map([segment_id], |row| {
            Ok(SegmentVersionRow {
                id: row.get(0)?,
                segment_id: row.get(1)?,
                target_text: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        rows.collect::<SqlResult<Vec<SegmentVersionRow>>>()
    })?;

    Ok(rows)
}

/// Lý do một lượt **khôi phục** bị từ chối, mang ra khỏi closure ghi.
///
/// ⚠️ Cùng khuôn và cùng lý do định lượng với [`ConfirmReject`] và [`OmitReject`]:
/// `Store::write` gói mọi `Err` thành `StoreError::WriteFailed { detail: String }`, nên đoán
/// lại lý do từ chuỗi `detail` là một phép so chuỗi — thứ vỡ im lặng ở lượt đổi câu chữ đầu tiên.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RestoreReject {
    /// Không có hàng `segment` nào mang `segment_id` đó.
    NotFound,
    /// `retired_at` khác `NULL` (AD-5). 🔴 Đường này **ghi**, nên nó từ chối — ngược với
    /// [`read_segment_history`], thứ **đọc** và phải trả đủ (AC4).
    Retired,
    /// `version_id` có thật nhưng **thuộc một segment khác** — hoặc không tồn tại.
    VersionNotOfThisSegment,
}

/// Kết quả một lượt **khôi phục** — thứ đi ra qua dây. Story 2.6, AC2.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RestoreOutcome {
    /// Segment vừa được khôi phục.
    pub segment_id: i64,
    /// Trạng thái **sau** lượt gọi.
    pub status: String,
    /// Lượt gọi này có **ghi** hay không.
    ///
    /// `false` cùng với `needs_confirmation = false` ⇒ khôi phục về **đúng nội dung đang có**,
    /// một lượt vô hại (xem doc-comment của hàm).
    pub restored: bool,
    /// 🔴 **Lượt ghi bị GIỮ LẠI vì nó sắp xoá vĩnh viễn một bản nháp chưa từng được ký.**
    ///
    /// Chữ ký #2(a) của Ice (2026-08-16). Khi trường này `true`, **không một byte nào được
    /// ghi**; webview hỏi lại người dùng rồi gọi lại với `force = true`.
    pub needs_confirmation: bool,
    /// Bản nháp sắp bị ghi đè, để webview **hiện ra** thay vì chỉ nói *"có thứ sẽ mất"*.
    /// `Some` khi và chỉ khi `needs_confirmation`.
    pub unsigned_draft: Option<String>,
}

/// **Khôi phục văn bản đích của một segment về một phiên bản cũ** — hàm thuần, đây là thứ
/// test gọi. Story 2.6 · FR101 · AC2 · Quyết định #1 đường (a) và #2 đường (a)
/// (Ice ký 2026-08-16).
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - `segment_id` không có trong Tác phẩm đang mở ⇒ `segment.not_found`;
/// - segment **đã về hưu** ⇒ `segment.retired` (AD-5);
/// - `version_id` không thuộc segment đó ⇒ `segment.not_found` *(xem ghi chú bên dưới)*.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 KHÔNG `INSERT` MỘT HÀNG `segment_version` NÀO — Quyết định #1 đường (a)
/// ─────────────────────────────────────────────────────────────────────────────
/// Đây là chỗ **hai tài liệu quy hoạch nói ngược nhau**, và mâu thuẫn đó đo được:
/// - Bảng Rule của **AD-31** (`ARCHITECTURE-SPINE.md`) có đúng **sáu** hàng và
///   **không hàng nào là "khôi phục"**. Hàng duy nhất tạo một `SegmentVersion` là *"Xác nhận
///   segment (FR24)"*.
/// - **Mockup** (`mockups/data-integrity.html:226-229`) viết đậm: *"Khôi phục là tạo phiên
///   bản mới… đẩy nó lên thành phiên bản thứ sáu"*.
///
/// Ice ký **(a)**: AD-31 **không sửa một chữ**, và AC2 đúng nguyên văn *(trạng thái về chưa
/// xác nhận)*. Một hàng `segment_version` ghi cho văn bản mà **chưa ai ký** làm bảng mang hai
/// nghĩa trộn lẫn — *"bản đã được ký"* và *"bản từng hiện diện"*.
///
/// ⚠️ Lời hứa *"lịch sử chỉ dài thêm, không bao giờ ngắn đi"* của mockup **vẫn giữ**: phiên
/// bản *"thứ sáu"* xuất hiện ở **lượt xác nhận kế tiếp**, do chính hàng 2 của AD-31 sinh ra.
/// Mockup sai về **thời điểm**, không về cơ chế — ghi nợ có chủ **Ice**, và dev **không** sửa
/// mockup.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 CHỐT CHỐNG MẤT BẢN NHÁP CHƯA KÝ — Quyết định #2 đường (a), và không AC nào nêu nó
/// ─────────────────────────────────────────────────────────────────────────────
/// **Số đo:** một hàng `segment_version` chỉ sinh ở **đúng một chỗ** — bên trong
/// [`confirm_segment`], sau chốt AC13. ⇒ Văn bản đích **chưa bao giờ được ký** *(status
/// `'draft'` có nội dung)* **không có một bản sao nào ở bất cứ đâu**. Một lượt khôi phục lên
/// nó **xoá vĩnh viễn**, và đó là §*"Dữ liệu người dùng — chỗ hỏng là VĨNH VIỄN"*.
///
/// ⇒ Khi văn bản hiện tại **chưa từng được ký** và **khác** bản sắp khôi phục, lượt ghi bị
/// **giữ lại**: hàm trả `needs_confirmation = true` kèm chính bản nháp đó, **không** ghi một
/// byte nào. Webview hỏi lại rồi gọi lại với `force = true`.
///
/// 🔴 **Phép so là "văn bản này có bản sao trong `segment_version` không", KHÔNG phải một cờ
/// `dirty`** — hợp đồng phụ bắt buộc của AD-31 (`ARCHITECTURE-SPINE.md`). Và nó là phép
/// so **đúng** chứ không chỉ là phép so hợp lệ: thứ đáng lo không phải *"đã sửa hay chưa"* mà
/// *"cái sắp mất có bản sao ở đâu không"*. Một câu người dùng gõ rồi hoàn tác về đúng một bản
/// đã ký thì **không mất gì** — cờ dirty sẽ hỏi thừa, phép so này thì không.
///
/// ⚠️ **Một nghĩa vụ của TẦNG GỌI, ghi ra vì Rust không cưỡng chế được:** phép so này chạy
/// trên **đĩa**. `editorEditedText` phía webview có thể mang ký tự **chưa flush** (AD-35: idle
/// 2 s, trần cứng 5 s). ⇒ Webview **phải flush trước khi gọi**, nếu không chốt này so với một
/// bản cũ hơn thứ người dùng đang nhìn. Đường thay thế *(gửi văn bản đang gõ qua dây làm tham
/// số)* bị loại: nó cho tầng UI **khai** trạng thái đĩa, tức một quy tắc nghiệp vụ chạy ở
/// TypeScript (AD-1).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 KHÔI PHỤC VỀ ĐÚNG NỘI DUNG ĐANG CÓ LÀ **VÔ HẠI** — ca biên AC2 không nêu
/// ─────────────────────────────────────────────────────────────────────────────
/// [`confirm_segment`] đã có tiền lệ cho đúng lớp này (AC13): ký lại một câu **đã ký** trả
/// `Ok(false)`, không hàng mới, không đổi `updated_at`. Khuôn tương ứng ở đây: khôi phục về
/// một phiên bản mang **đúng** văn bản đang có ⇒ **không ghi một byte nào**, `restored = false`.
///
/// 🔴 Và vế **không hạ `status`** là nửa quan trọng hơn, chứ không phải một phép tối ưu: một
/// segment đang `'confirmed'` mà bị hạ xuống `'draft'` vì người dùng bấm khôi phục lên chính
/// bản đang dùng là một lượt **huỷ chữ ký của họ mà không đổi lấy gì**. Người dùng phải ký
/// lại một câu chưa hề đổi.
///
/// ⚠️ `restored = false` vì thế **phải phân biệt được** với `needs_confirmation = true`: một
/// cái là *"không có gì để làm"*, cái kia là *"đang chờ bạn"*. Gộp chúng vào một `bool` là
/// dựng lại đúng lớp lỗi mà [`ConfirmOutcome::version_created`] tồn tại để tránh.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ AD-46 — MỘT KHOẢNG HỞ THẬT, ghi ra thay vì tự chấm là đã xét
/// ─────────────────────────────────────────────────────────────────────────────
/// `is_target_paragraph_end` (bước di trú 9, Story 2.5d) là **dữ liệu riêng của bản dịch**, và
/// nó sống trên `segment` chứ **không** trên `segment_version` — bảng có đúng bốn cột và không
/// cột nào là cờ. ⇒ Một lượt khôi phục trả `target_text` về bản cũ nhưng **giữ nguyên cấu trúc
/// đoạn hiện tại**, và với một bản dịch từng ngắt đoạn khác đi thì hai thứ đó **không còn nói
/// cùng một chuyện**.
///
/// AC2 nói khôi phục *"văn bản đích"* và **không nói cờ đích**; AD-31 lẫn AD-46 đều không nói
/// tới ca này. ⇒ Hàm này **chỉ đụng văn bản**, và khoảng hở ghi nợ có chủ **Ice** *(quyết định
/// ngữ nghĩa, không một lượt cài đặt)*. Ba đường thoát đã liệt kê ở `deferred-work.md`.
pub fn restore_segment_version(
    open: Option<&OpenWork>,
    segment_id: i64,
    version_id: i64,
    force: bool,
) -> Result<RestoreOutcome, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    let reject: Arc<Mutex<Option<RestoreReject>>> = Arc::new(Mutex::new(None));
    let reject_in = Arc::clone(&reject);
    // Ban nhap sap mat, mang RA khoi closure ghi -- cung ly do voi `reject`.
    let held: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let held_in = Arc::clone(&held);

    let outcome = open.store.write(move |tx: &Transaction<'_>| {
        let set_reject = |r: RestoreReject| {
            *reject_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(r);
        };

        // ① Trang thai hien tai — TRONG cung giao dich voi luot ghi. Doc ca `status` vi ca
        //    bien "khoi phuc ve dung noi dung dang co" phai biet co nen ha no khong.
        let found = tx.query_row(
            "SELECT target_text, retired_at, status FROM segment WHERE id = ?1",
            [segment_id],
            |row| {
                let target_text: String = row.get(0)?;
                let retired_at: Option<String> = row.get(1)?;
                let status: String = row.get(2)?;
                Ok((target_text, retired_at, status))
            },
        );

        let (current_text, retired_at, current_status) = match found {
            Ok(value) => value,
            Err(SqlError::QueryReturnedNoRows) => {
                set_reject(RestoreReject::NotFound);
                return Err(SqlError::QueryReturnedNoRows);
            }
            Err(err) => return Err(err),
        };

        // ② 🔴 Duong nay GHI, nen no tu choi mot tombstone -- nguoc voi `read_segment_history`.
        if retired_at.is_some() {
            set_reject(RestoreReject::Retired);
            return Err(SqlError::QueryReturnedNoRows);
        }

        // ③ Phien ban phai THUOC chinh segment nay. Mot `version_id` cua segment khac di lot
        //    o day la mot luot ghi van ban cua cau NAY bang van ban cua cau KHAC -- hong am
        //    tham, va nguoi dung khong co duong nao lan ra. Menh de `AND segment_id = ?2` la
        //    hang rao, khong mot phep loc cho gon.
        let version_text = tx.query_row(
            "SELECT target_text FROM segment_version WHERE id = ?1 AND segment_id = ?2",
            [version_id, segment_id],
            |row| row.get::<_, String>(0),
        );
        let version_text = match version_text {
            Ok(text) => text,
            Err(SqlError::QueryReturnedNoRows) => {
                set_reject(RestoreReject::VersionNotOfThisSegment);
                return Err(SqlError::QueryReturnedNoRows);
            }
            Err(err) => return Err(err),
        };

        // ④ DA dung noi dung do ⇒ khong ghi mot byte nao, va KHONG ha `status`. Khuon AC13
        //    cua `confirm_segment`. Xem doc-comment: ha mot chu ky ma khong doi lay gi la
        //    mot luot huy cong cua nguoi dung.
        if current_text == version_text {
            return Ok((false, false, current_status));
        }

        // ⑤ 🔴 CHOT CHONG MAT BAN NHAP CHUA KY — chu ky #2(a).
        //    "Chua tung duoc ky" = van ban hien tai KHONG co ban sao nao trong `segment_version`.
        //    Day la phep so VAN BAN (AD-31 §Hop dong phu), khong mot co `dirty`.
        //
        // 🔵 VE `!current_text.is_empty()` LA MOT MIEN TRU CO CHU DICH — ghi ra 2026-08-16
        //    (code review), vi ban dau no khong mang mot dong ly do nao.
        //    Mot `target_text` RONG khong co gi de mat: chot nay ton tai de chan viec ghi de len
        //    chu ma nguoi dung da go, va chuoi rong khong phai chu ai go. Bo ve nay di thi moi
        //    cau CHUA DICH deu hoi mot cau vo nghia o luot khoi phuc dau tien -- "ban duoi day
        //    se mat" tro toi mot o trong -- va mot hop thoai hoi thua la thu lam nguoi dung bam
        //    "dong y" theo phan xa, tuc lam chot THAT mat tac dung. Cung ly do da ghi cho phep
        //    so "co ban sao chua" thay vi mot co `dirty`.
        //    ⚠️ GIOI HAN THAT: mot nguoi dung XOA SACH mot ban dich roi khoi phuc se khong duoc
        //    hoi. Ho khong mat mot ky tu nao (o da rong), nhung ho cung khong duoc bao rang
        //    luot xoa vua roi khong con duong ve. Vo hai hom nay; neu Story 2.8 hay Review Mode
        //    lam "rong" mang mot nghia RIENG (khac "chua dich"), doc lai cho nay.
        if !force && !current_text.is_empty() {
            let has_copy: i64 = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM segment_version \
                 WHERE segment_id = ?1 AND target_text = ?2)",
                (segment_id, &current_text),
                |row| row.get(0),
            )?;
            if has_copy == 0 {
                *held_in
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    Some(current_text.clone());
                // KHONG ghi mot byte nao. Day KHONG phai mot loi -- xem `needs_confirmation`.
                return Ok((false, true, current_status));
            }
        }

        // ⑥ Ghi. HAI cot, MOT giao dich: van ban va trang thai (AC2).
        //    ⚠️ `updated_at` CO doi o day, khac `confirm_segment`: mot luot khoi phuc SUA van
        //    ban that su, nen no dung nghia "moc sua van ban" ma `SEGMENT_DDL` khai cho cot do.
        write_non_user_target(tx, segment_id, &version_text, TRANSLATION_ORIGIN_OTHER, None)?;

        Ok((true, false, SEGMENT_STATUS_DRAFT.to_owned()))
    });

    match outcome {
        Ok((restored, needs_confirmation, status)) => {
            let unsigned_draft = held
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            Ok(RestoreOutcome {
                segment_id,
                status,
                restored,
                needs_confirmation,
                unsigned_draft,
            })
        }
        Err(err) => {
            let taken = reject
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            match taken {
                Some(RestoreReject::NotFound) => Err(segment_not_found(segment_id)),
                Some(RestoreReject::Retired) => Err(segment_retired(segment_id)),
                // ⚠️ Tai dung `segment.not_found` co chu dich: tu goc nhin cua nguoi dung,
                //    mot `version_id` khong thuoc segment nay la mot phien ban KHONG TIM THAY
                //    o segment nay. Mot khoa `err.segment.*` THU NAM chi dang mo neu no noi
                //    mot chuyen KHAC voi bon khoa da co -- va `message_keys!` la danh muc DONG,
                //    nen mot khoa them vao "cho ro" ma khong ai doc la mot muc chet.
                Some(RestoreReject::VersionNotOfThisSegment) => Err(segment_not_found(segment_id)),
                // O rong ⇒ day la mot loi KHO that, khong mot phep tu choi nghiep vu.
                None => Err(err.into()),
            }
        }
    }
}
