//! Tạo Tác phẩm mới (dán văn bản/tệp/`.docx`) và pha tải ảnh nhúng của Story 6.11 — tách
//! ra khỏi `commands/project/mod.rs`, không đổi hành vi.
//!
//! `use super::*;` mang mọi kiểu/hàm dùng chung của `commands::project` vào đây; các hàm
//! nhiều mối quan tâm (`run_pipeline`, `resolve_chapter_pattern`, `spawn_import_scan`,
//! `effective_origin_fields`, `guarded_dict_layers`) VẪN sống ở `mod.rs` — Chặng 2 không
//! viết lại hệ xem trước đan xen, chỉ nhấc ba khối tự chứa (xem `deferred-work.md`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA ĐƯỜNG VÀO CỦA AC1 GẶP NHAU Ở ĐÚNG MỘT HÀM — [`create_work`]
//! ─────────────────────────────────────────────────────────────────────────────
//! Dán văn bản đổ vào [`crate::core::segment::import::import_text`] rồi tới đây; kéo-thả
//! và ô nhập đường dẫn đổ vào [`crate::core::segment::import::import_file`] rồi cũng tới
//! đây. [`create_work`] là chỗ **duy nhất** gọi [`crate::core::store::Store::write`] cho
//! `project.db` — không đường nào khác giữ một bản sao.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use super::*;

/// **Hàm thuần** — tạo một Tác phẩm mới trên đĩa từ một [`PipelineShape`] đã có sẵn.
///
/// Thứ tự: dựng thư mục (`core::library::atproj`) → mở `project.db`
/// (`StoreSpec::project`) → chạy TRỌN chuỗi pipeline AD-39 (`run_import`, bước 1-7) →
/// **ghi** hàng `work` + N hàng `chapter` (+ segment của mỗi Chương) trong MỘT giao dịch →
/// dựng lại `meta.json` từ `project.db` vừa commit (Quyết định #3, AD-33) → ghi `meta.json`
/// nguyên tử NGAY SAU giao dịch. Bất kỳ bước nào trượt ⇒ dọn thư mục, không để lại
/// `.atproj/` nửa vời (AC8).
///
/// 🔵 **SỬA 2026-09-04 (Story 6.2, AD-39) — nhận `PipelineShape`, không còn `ImportedChapter`
/// đơn lẻ; ghi N Chương, không còn đúng một.** 🔵 **SỬA 2026-09-05 (Story 6.6) — "N = 1 trên
/// đường sản phẩm hôm nay" đã HẾT ĐÚNG.** Hàm nay nhận thêm tham số `chapter_pattern`
/// (`Option<ChapterPattern>`) và N > 1 là một kết quả THẬT trên đường sản phẩm khi người
/// dùng cấu hình một mẫu khớp được nhiều lần — đường đi đã tổng quát từ Story 6.2, không cần
/// sửa lại hàm này lần nữa.
///
/// 🔵 **SỬA 2026-09-04 (Story 6.3) — thêm tham số `encoding`, ĐIỂM TIÊM DUY NHẤT của bảng
/// mã đã chọn/đã dò.** Trước story này hàm luôn khai UTF-8 cứng qua
/// [`PipelineInput::default_shaped`]; giờ chỗ gọi CHỌN [`PipelineInput::with_encoding`] hay
/// `default_shaped` — `create_work_from_text`/`create_work_from_file` (không đổi chữ ký,
/// dùng bởi `tests/**` và đường sản phẩm KHÔNG đi qua xem trước bảng mã) truyền
/// [`encoding_rs::UTF_8`]; `wire::confirm_import_with_encoding` (Story 6.3, đường CÓ xem
/// trước) truyền bảng mã người dùng đã xác nhận. Đây VẪN là chỗ gọi [`run_import`] DUY NHẤT
/// của cả crate (`segment_pipeline_boundary.rs::run_import_is_the_one_product_call_site`)
/// — không một chỗ gọi thứ hai nào được mở (§Always spec 6.3).
///
/// # Lỗi
/// - dựng thư mục trượt ⇒ `project.create_failed`;
/// - chuỗi pipeline trượt (ví dụ byte không hợp lệ với bảng mã ĐÃ CHỌN) ⇒ lỗi nhập
///   (`import.*`), qua `From<ImportError>`;
/// - mở/ghi `project.db` trượt ⇒ lỗi kho (`store.*`), qua `From<StoreError>`.
///
/// 🔴 **THÊM 2026-09-07 (Story 6.9) — tham số `block_overrides`, đúng cái vòng rà 1 đã hụt.**
/// Trạng thái giữ/loại người dùng đặt bằng `Space`/`[`/`]` ở tầng 2 PHẢI đi tới đây — đây là
/// đường DUY NHẤT ghi Chương xuống `.atproj` (doc-comment ở trên). Không truyền tham số này
/// (hoặc truyền `Vec::new()` một cách sai) làm đĩa nhận phán đoán MÁY trong khi màn hình đã
/// hiện sửa tay — `wire::confirm_import_with_encoding` là chỗ gọi PHẢI đọc
/// `Tier2BlockOverridesState` rồi truyền NGUYÊN VẸN vào đây, reset state đó SAU KHI ghi
/// xong (không phải TRƯỚC — một lượt xác nhận trượt giữ nguyên override để thử lại).
/// 🔴 **THÊM 2026-09-08 (Story 6.11, mục B1 vòng rà đối kháng 3 lớp) — tham số `domain_log_state`.**
/// Trước bản sửa này, pha ảnh tích luỹ `Vec<DomainLogEntry>` cục bộ rồi CHỈ gắn nó vào
/// `OpenWork::pending_domain_log` trên đường THÀNH CÔNG — một lượt nhập trượt (ví dụ đĩa đầy
/// giữa lúc ghi ảnh thứ N) trả `Err` sớm, và không kiểu `Result<OpenWork, IpcError>` nào chở
/// được một `Vec<DomainLogEntry>` kèm theo nhánh `Err`, nên nhật ký của N-1 ảnh ĐÃ tải thành
/// công trước đó biến mất — vi phạm đúng chữ §Always spec 6.11 *"kể cả lượt trượt"*. Sửa bằng
/// cách PUSH THẲNG vào `domain_log_state` ngay khi mỗi lời gọi `fetch` hoàn tất (thành công
/// hay không), thay vì tích luỹ cục bộ rồi trả về SAU CÙNG — một khi đã push, entry đó SỐNG
/// SÓT bất kể phần còn lại của `create_work` có trượt hay không. Test không cần domain log
/// bền qua lượt trượt (đa số) truyền `&Mutex::new(Vec::new())` — một kho tạm, vứt đi sau ca.
///
/// `on_image_progress`/`should_cancel` are called from inside
/// [`prepare_chapter_images`]'s fetch/write loop; see its doc comment. [`create_work`] is a
/// thin wrapper over this with both no-op, for callers that don't need either.
pub fn create_work_with_progress(
    documents_root: &Path,
    name: &str,
    source_lang: &str,
    genre: &str,
    shape: PipelineShape,
    encoding: &'static encoding_rs::Encoding,
    cleanup_rules: Vec<crate::core::cleanup::CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    block_overrides: Vec<Option<bool>>,
    // 🔴 **THÊM 2026-09-11 (Story 6.16, FR115)** — vai cột + cờ tiêu đề của đường nhập song
    // ngữ. Tham số MỖI LƯỢT NHẬP, cùng khuôn `chapter_pattern` (không một state thứ ba —
    // §Boundaries spec 6.16). Vô nghĩa (không đọc) khi `shape` không phải
    // `PipelineShape::Bilingual`.
    bilingual_source_column: usize,
    bilingual_target_column: usize,
    bilingual_has_header: bool,
    // 🔴 **THÊM 2026-09-10 (Story 6.15)** — xuất xứ NGƯỜI DÙNG đã gõ đè, theo CHỈ SỐ Chương của
    // lượt nhập này — xem doc-comment [`ChapterOriginOverridesState`]. `&[]` (mọi chỗ gọi
    // KHÔNG đi qua màn xem trước xuất xứ — `tests/**` cũ, `create_work_from_text`/`_from_file`)
    // là "chưa ai sửa gì", CÙNG hành vi trước story này.
    origin_overrides: &[Option<ChapterOriginOverride>],
    domain_log_state: &webimport::DomainLogState,
    // 🔴 **THÊM 2026-09-09 (Story 6.12)** — khối + ảnh nhúng của một `.docx`, đọc SẴN bởi
    // `import_file` (nó không đi qua `Step::ExtractMainContent`, đó là bóc HTML). `None` cho
    // mọi đường khác (dán tay, `.txt`/`.md`, URL). Xem doc-comment
    // [`crate::core::segment::import::DocxSidecar`] cho giới hạn "chỉ Chương đầu tiên".
    docx_sidecar: Option<crate::core::segment::import::DocxSidecar>,
    // 🔴 **THÊM 2026-09-12 (Story 6.17, FR116)** — quy nhóm câu đích người dùng đã làm cho các
    // hàng lệch cặp của đường song ngữ. Tham số MỖI LƯỢT NHẬP, cùng khuôn `bilingual_source_column`
    // (§Never: "regroupings travel as a per-call param, like chapter_pattern"), KHÔNG một
    // `Mutex` override thứ ba. `&[]` cho mọi chỗ gọi không phải đường song ngữ.
    regroupings: &[crate::core::segment::bilingual::BilingualRegrouping],
    on_image_progress: &mut dyn FnMut(usize, usize),
    should_cancel: &dyn Fn() -> bool,
) -> Result<OpenWork, IpcError> {
    validate_source_lang(source_lang)?;

    let dir = create_work_folder(documents_root, name)?;

    let db_path = dir.join("project.db");
    let store = match Store::open(StoreSpec::project(db_path)) {
        Ok(store) => store,
        Err(err) => {
            remove_folder(&dir);
            return Err(err.into());
        }
    };

    let work_id = Uuid::new_v4().to_string();
    let name_owned = name.to_owned();
    let source_lang_owned = source_lang.to_owned();
    let genre_owned = genre.to_owned();

    // 🔴 AD-39 — TOÀN BỘ chuỗi bảy bước chạy **ở đây, một lần, lúc nhập**, và **NGOÀI**
    // closure ghi bên dưới, có chủ ý — cùng lý do Quyết định #3 cũ của Story 1.15 vẫn giữ:
    // AD-11 giữ **một** writer duy nhất nối tiếp (một `Connection` `move` vào một thread,
    // job đi qua `mpsc::channel`), nên thời gian CPU bên trong closure **chặn mọi lượt ghi
    // khác của tiến trình**. Một Chương dài đi qua chuỗi trong closure là một lượt khoá
    // hàng đợi ghi mà auto-save của Editor (NFR2) phải xếp sau.
    //
    // 🔵 SỬA 2026-09-05 (Story 6.6) — "chapter_pattern: None" (Never clause của spec 6.2) đã
    // HẾT ĐÚNG: `chapter_pattern` giờ là tham số THẬT của chính `create_work`, tới từ màn
    // xem trước nhập. `None` vẫn hợp lệ (không mẫu ⇒ bước 5 no-op, N = 1) — chỉ không còn là
    // giá trị DUY NHẤT có thể tới đây.
    //
    // 🔵 SỬA 2026-09-04 (Story 6.3) — `with_encoding`, không còn `default_shaped` cứng
    // UTF-8: `encoding` giờ là tham số của chính `create_work` (xem doc-comment hàm này).
    // 🔵 SỬA 2026-09-05 (Story 6.5) — qua `run_pipeline` (không gọi `run_import` thẳng ở
    // đây nữa — xem doc-comment của hàm đó), cộng `cleanup_rules` đã phân giải.
    //
    // 🔴 **THÊM 2026-09-06 (Story 6.7)** — `extract_main_content` chốt vào HÌNH DẠNG đầu
    // vào, không phải một tham số riêng của `create_work`. `Chapters(RawBytes, ..)` — byte
    // HTML CHƯA giải mã — là hình dạng DUY NHẤT mà danh sách URL (Story 6.7) dựng ra; không
    // đường sản phẩm nào khác (dán tay, tệp) từng tạo ra nó. 🔴 **Điều kiện KHÔNG ĐƠN GIẢN
    // là "shape là `Chapters`"** — `tests/project_contract.rs::create_work_writes_every_chapter_and_its_segments_when_the_pipeline_yields_more_than_one`
    // dựng tay một `Chapters(AlreadyText, ..)` từ TRƯỚC story này để kiểm thuần cơ chế ghi N
    // Chương, không liên quan gì tới web — một điều kiện chỉ nhìn biến thể `Chapters` sẽ gọi
    // `webimport::extract("Chuong mot...", "")` (nhãn RỖNG, không phải URL tuyệt đối) và làm
    // ca đó ĐỎ OAN. Đúng điều kiện: hình dạng LÀ `Chapters` VÀ đơn vị ĐẦU là `RawBytes` (byte
    // thô CHƯA giải mã — dấu hiệu THẬT của "đến từ mạng", `AlreadyText` không bao giờ cần
    // bóc, dù đứng trong hình dạng nào).
    // ⚠️ **NỢ CÓ CHỦ, ghi ra tại chỗ (vòng rà đối kháng 2, mục D6).** Điều kiện dưới đây đọc
    // MỘT MÌNH `cs.first()` — nếu MỘT `PipelineShape::Chapters` nào đó lỡ TRỘN hình dạng (mục
    // đầu `AlreadyText`, mục SAU `RawBytes`), `extract_main_content` tắt cho TOÀN BỘ danh
    // sách và những mục `RawBytes` phía sau KHÔNG BAO GIỜ được bóc nội dung/tìm ảnh — không
    // panic, không lỗi, `images_failed` vẫn `0` (im lặng, không phải một lượt "thử rồi
    // trượt"). ĐO ĐƯỢC: hai bộ dựng SẢN PHẨM DUY NHẤT của `Chapters` hôm nay
    // (`chapters_shape_if_all_ok`/`chapters_shape_for_view`) luôn dựng một danh sách ĐỒNG
    // NHẤT (toàn `RawBytes`, từ `UrlImportItem` đã tải) —
    // 🔵 SỬA 2026-09-09 (vòng rà đối kháng 3, mục T6): câu trước dẫn `homogeneity_boundary.rs
    // (nếu có)` — tệp đó KHÔNG tồn tại trong `src-tauri/tests/`, một đoạn khai ĐO ĐƯỢC mà
    // mang "(nếu có)" là tự thú chưa kiểm. Test THẬT khoá bất biến này là
    // `webimport_contract.rs::the_two_real_chapters_shape_builders_always_produce_a_homogeneous_list_of_raw_bytes`.
    // Đường KHÁC (test dựng tay `Chapters(AlreadyText, ..)`, xem chú thích trên) cũng đồng
    // nhất. Vì thế điều kiện SAI này chưa từng bị kích hoạt trên đường thật — nhưng
    // `run_import`/`PipelineInput` là một seam CÔNG KHAI, và không gì ở KIỂU ngăn một chỗ gọi
    // tương lai trộn hình dạng. **Chủ: Ice** — sửa đúng cần `extract_main_content` chuyển
    // thành MỘT QUYẾT ĐỊNH THEO TỪNG Chương
    // (không phải một cờ toàn cục), một thay đổi cấu trúc lớn hơn phạm vi lượt vá nhỏ này;
    // xem `deferred-work.md`.
    let extract_main_content = matches!(
        &shape,
        PipelineShape::Chapters(cs) if matches!(cs.first(), Some(ChapterInput::RawBytes { .. }))
    );

    // 🔴 **THÊM 2026-09-08 (Story 6.11, FR127)** — trích URL trang của TỪNG đơn vị TRƯỚC khi
    // `shape` bị `run_pipeline` (ngay dưới) tiêu thụ (Code Map spec 6.11: "URL trang ... đọc
    // được từ `shape` TRƯỚC KHI nó bị run_import nuốt"). Chỉ có ý nghĩa khi `extract_main_content`
    // (đường URL, ảnh cần URL trang để phân giải `src` tương đối) — đường Blob/AlreadyText cho
    // ra một danh sách một phần tử KHÔNG bao giờ được `prepare_chapter_images` đọc tới (nó
    // `continue` ngay ở Chương có `blocks: None`).
    let chapter_urls: Vec<String> = match &shape {
        PipelineShape::Blob(c) => vec![chapter_input_page_url(c)],
        PipelineShape::Chapters(cs) => cs.iter().map(chapter_input_page_url).collect(),
        // 🔴 **THÊM 2026-09-11 (Story 6.16)** — cùng lý do nhánh `Blob` ngay trên: đường song
        // ngữ KHÔNG BAO GIỜ bóc nội dung chính (`extract_main_content` luôn `false` cho hình
        // dạng này — nó không phải `PipelineShape::Chapters`), nên `prepare_chapter_images`
        // không bao giờ đọc tới danh sách này; một phần tử rỗng không mất gì.
        PipelineShape::Bilingual { .. } | PipelineShape::BilingualTables { .. } | PipelineShape::BilingualRows { .. } => {
            vec![String::new()]
        }
        // 🔴 SỬA 2026-09-16 (vòng rà đối kháng 2, mục 10/G9) — bản trước map thật
        // `chapter_input_page_url` qua từng đơn vị, mang đường dẫn hệ thống tệp THẬT ở arity
        // N-đơn-vị, trong khi một mẫu phân tách có thể cho ra > N Chương đầu ra — vector lệch
        // arity với thứ nó SẼ được đọc theo (chỉ số Chương, không phải chỉ số đơn vị) NẾU có
        // ngày một đường đọc nó xuất hiện, và tới lúc đó một đường dẫn cục bộ bị đọc nhầm
        // thành URL trang. Cùng lý do nhánh `Bilingual`/`Blob` ngay trên: đường `Files` KHÔNG
        // BAO GIỜ bóc nội dung chính (`extract_main_content` luôn `false` trên đường tệp —
        // §Always spec 6.6b không đổi luật đó), nên `prepare_chapter_images` không bao giờ đọc
        // tới danh sách này hôm nay — đổi sang MỘT ô rỗng, đúng khuôn `Bilingual` ngay trên,
        // gỡ hẳn mối nguy arity thay vì chỉ canh nó.
        PipelineShape::Files(_) => vec![String::new()],
    };
    // `cleanup_rules`/`block_overrides` bị DI CHUYỂN vào `PipelineInput` ngay dưới — pha ảnh
    // (sau khi chuỗi chạy xong) cần lại đúng hai giá trị này để tính neo (bước 3/4 AD-39 lặp
    // lại trên TIỀN TỐ, xem `core::segment::anchor::compute_anchor`) và để biết Chương nào
    // đọc `block_overrides` (chỉ Chương đầu — cùng luật `Step::ExtractMainContent`), nên clone
    // TRƯỚC khi di chuyển, không sau.
    let cleanup_rules_for_images = cleanup_rules.clone();
    let block_overrides_for_images = block_overrides.clone();
    // 🔴 THÊM 2026-09-10 (Story 6.15) — bản CHÉP thành `Vec` sở hữu, `move` được vào closure
    // ghi `'static` bên dưới (`origin_overrides` tham số chỉ là `&[..]`, không sống đủ lâu).
    let origin_overrides_owned: Vec<Option<ChapterOriginOverride>> = origin_overrides.to_vec();

    let outcome = match run_pipeline(
        PipelineInput::with_encoding(shape, encoding, source_lang_owned.clone())
            .with_cleanup_rules(cleanup_rules)
            .with_chapter_pattern(chapter_pattern)
            .with_extract_main_content(extract_main_content)
            .with_block_overrides(block_overrides)
            .with_bilingual_columns(bilingual_source_column, bilingual_target_column, bilingual_has_header)
            .with_bilingual_regroupings(regroupings.to_vec()),
    ) {
        Ok(outcome) => outcome,
        Err(err) => {
            store.close();
            remove_folder(&dir);
            return Err(err.into());
        }
    };
    // 🔴 **THÊM 2026-09-11 (Story 6.16) — từ chối Ở RUST, không chỉ ở nút webview.** §Boundaries:
    // "Mismatched row ⇒ confirm is locked... Rust-side, not only UI". `create_work` là điểm
    // gọi DUY NHẤT ghi `.atproj` (doc-comment đầu tệp) — canh Ở ĐÂY giữ đúng "không hàng nào
    // ghi được khi còn lệch cặp" cho MỌI chỗ gọi, không riêng `confirm_bilingual_import`.
    if !outcome.bilingual_mismatches.is_empty() {
        let count = outcome.bilingual_mismatches.len();
        store.close();
        remove_folder(&dir);
        return Err(crate::core::segment::import::ImportError::BilingualMismatchedRows { count }.into());
    }
    let mut chapters = outcome.chapters;

    // 🔴 **THÊM 2026-09-09 (Story 6.12)** — khối + ảnh `.docx` gắn vào Chương ĐẦU TIÊN ở
    // đây, NGAY SAU khi chuỗi bảy bước đã ổn định số Chương. `.docx` không đi qua
    // `Step::ExtractMainContent` (đó là bóc HTML qua `dom_smoothie`, `.docx` không phải
    // HTML) nên `chapter.blocks` vẫn `None` tới đây cho MỌI đường `.docx` — đây là chỗ DUY
    // NHẤT nó được gán. Cùng giới hạn "chỉ Chương đầu" mà `block_overrides` đã theo cho
    // đường URL (Story 6.9) — xem doc-comment `DocxSidecar`.
    let docx_chapter_images: Vec<ChapterDocxImage> = docx_sidecar
        .as_ref()
        .map(|sidecar| distribute_docx_blocks_across_chapters(sidecar, &mut chapters))
        .unwrap_or_default();

    // 🔴 THÊM 2026-09-09 (D10 vòng rà đối kháng 3 lớp) — `chapter_urls` (dựng TRƯỚC
    // `run_pipeline`, một phần tử mỗi ĐƠN VỊ đầu vào, xem trên) và `chapters` (SAU khi bảy
    // bước AD-39 chạy) được `prepare_chapter_images` giả định 1:1 THEO CHỈ SỐ
    // (`chapter_urls.get(i)`) — không một dòng nào từng khẳng định điều đó trước sửa này.
    // Giả định ĐÚNG khi `extract_main_content` bật: điều kiện đó buộc `shape` từng là
    // `PipelineShape::Chapters` (khối `matches!` phía trên), tức `already_chaptered = true`,
    // khiến `split_chapters_step` (`core::segment::pipeline`) return SỚM, không đổi số
    // Chương. Đường `Blob` CÓ THỂ nổ ra N > 1 Chương thật (Story 6.6, mẫu phân tách) trong
    // khi `chapter_urls.len() == 1` — nhưng nhánh đó không bao giờ chạy pha ảnh
    // (`extract_main_content = false` ⇒ mọi `chapter.blocks` đều `None`, `prepare_chapter_images`
    // `continue` ngay), nên bất biến chỉ cần đúng trong ĐÚNG nhánh sẽ thật sự đọc
    // `chapter_urls`. Nếu nó SAI, `unwrap_or("")` ở `prepare_chapter_images` khiến mọi `src`
    // tương đối trượt ÂM THẦM, không phân biệt được với "trang không có ảnh" (D10) — một lỗi
    // LẬP TRÌNH thật (không phải điều kiện người dùng có thể gây ra), bắt tại nguồn.
    //
    // 🔵 SỬA (vòng rà đối kháng 2, mục B1) — `assert_eq!` ĐỔI THÀNH `IpcError` + dọn
    // `.atproj`, không còn panic trần. `Cargo.toml` đặt `panic = "abort"` (bán kính nổ TOÀN
    // TIẾN TRÌNH — cùng lý lẽ chú thích "vòng rà đối kháng 2026-09-04, item 4" ngay dưới đây),
    // và tại DÒNG NÀY `create_work_folder`/`Store::open` đã chạy — một panic ở đây giết cả
    // ứng dụng NGƯỜI DÙNG ĐANG DÙNG và để lại một `.atproj` nửa vời, đúng trạng thái mà MỌI
    // nhánh `Err` khác của hàm này dùng `remove_folder` để tránh. Cùng khuôn hai kiểm tra
    // `chapters.is_empty()`/`i64::try_from` ngay dưới đây.
    if extract_main_content && chapters.len() != chapter_urls.len() {
        store.close();
        remove_folder(&dir);
        return Err(crate::core::library::WorkError::CreateFailed {
            detail: format!(
                "bat bien 1:1 giua chapters ({}) va chapter_urls ({}) da vo -- loi lap trinh, \
                 khong phai dieu kien nguoi dung gay ra",
                chapters.len(),
                chapter_urls.len()
            ),
        }
        .into());
    }

    // 🔴 SỬA (vòng rà đối kháng 2026-09-04, item 4) — bán kính nổ của một `.expect()` bên
    // TRONG closure ghi là TOÀN TIẾN TRÌNH: `panic = "abort"` giết ngay khi giao dịch đang
    // mở, không unwind, không rollback. Cả hai bất biến dưới đây được validate ở NGOÀI
    // closure, TRƯỚC khi giao dịch mở.
    // 🔵 SỬA 2026-09-05 (Story 6.6) — "chuỗi sản phẩm luôn N = 1" đã HẾT ĐÚNG (mẫu phân tách
    // cho N > 1 thật trên đường sản phẩm). Hai bất biến dưới vẫn kiểm ở đây vì lý do KHÁC,
    // không đổi: `run_import` là một seam CÔNG KHAI (`PipelineShape::Chapters` cho phép N
    // tuỳ ý), và một Chương RỖNG hay N vượt `i64` là ĐIỀU KIỆN BIÊN của chính N > 1, không
    // phải một điều kiện chỉ tồn tại lúc chuỗi còn luôn N = 1.
    if chapters.is_empty() {
        store.close();
        remove_folder(&dir);
        return Err(crate::core::library::WorkError::CreateFailed {
            detail: "pipeline nhap tra ve 0 Chuong -- khong co gi de ghi".to_owned(),
        }
        .into());
    }
    if i64::try_from(chapters.len()).is_err() {
        store.close();
        remove_folder(&dir);
        return Err(crate::core::library::WorkError::CreateFailed {
            detail: format!("so Chuong ({}) vuot i64 -- khong the ghi cot ord", chapters.len()),
        }
        .into());
    }

    // 🔴 **THÊM 2026-09-08 (Story 6.11, FR127)** — pha ảnh chạy Ở ĐÂY: sau khi chuỗi bảy bước
    // đã ổn định `chapter.source_text`/`segments` (cần cho `anchor::compute_anchor`), TRƯỚC
    // giao dịch ghi SQL bên dưới. Ghi TỆP xảy ra ở đây, NGOÀI closure `store.write` (khuôn
    // `Meta::write_atomic` — job ghi bên dưới CHỈ SQL); một ảnh trượt tải KHÔNG dừng hàm này (đếm
    // vào `images_failed`, log chẩn đoán), nhưng ghi BYTE trượt giữa chừng (đĩa đầy) THÌ
    // dừng — cùng khuôn mọi lỗi khác của hàm này (dọn `.atproj` nửa vời, AC8).
    let image_prep = match prepare_chapter_images(
        &dir,
        &chapters,
        &chapter_urls,
        &block_overrides_for_images,
        &cleanup_rules_for_images,
        &source_lang_owned,
        domain_log_state,
        &docx_chapter_images,
        on_image_progress,
        should_cancel,
    ) {
        Ok(prep) => prep,
        Err(err) => {
            store.close();
            remove_folder(&dir);
            return Err(err);
        }
    };

    // 🔴 **THÊM 2026-09-08 (Story 6.11)** — tách `image_prep` thành các trường rời TRƯỚC
    // closure `move` bên dưới: `saved` di chuyển vào closure (ghi `INSERT INTO asset`, CÙNG
    // giao dịch với `chapter`/`segment` — khuôn `atomic asset row` của spec 6.11); hai trường
    // còn lại ở lại đây để lắp vào `OpenWork` SAU khi giao dịch commit. `domain_log` KHÔNG
    // còn là một trường ở đây (mục B1) — mỗi lời gọi `fetch` đã PUSH THẲNG vào
    // `domain_log_state` ngay khi hoàn tất, nên nó sống sót cả trên đường trượt phía trên.
    let ImagePrepOutcome { saved: mut saved_assets, images_saved, images_failed } = image_prep;

    // 🔴 **THÊM 2026-09-09 (Story 6.13)** — dệt segment vai (Quyết định 1/2 spec 6.13) NGAY Ở
    // ĐÂY: `chapter.segments` đã ổn định (bước 7 AD-39, kể cả lượt ép ranh giới caption của
    // `role::force_caption_segment_boundaries` chạy TRONG chuỗi), và `prepare_chapter_images`
    // vừa xong (mọi neo TRƯỚC-KHI-DỆT đã tính). Đây là chỗ DUY NHẤT có cả `chapter.segments`
    // lẫn `saved_assets` — dệt xong TRƯỚC `store.write` để dãy segment cuối cùng và neo ĐÃ DỜI
    // đi vào CÙNG giao dịch, không một đường ghi thứ hai.
    //
    // 🔴 `docx_sidecar.is_some()` ⇒ KHÔNG dệt gì (Quyết định 3 spec 6.13: "Chỉ mô hình +
    // đường web"). `.docx` gắn `blocks` vào Chương đầu (ở trên, "THÊM 2026-09-09 Story 6.12")
    // nhưng đó là ảnh nhúng tài liệu, không phải bề mặt mà spec 6.13 mở — vế `.docx` là nợ có
    // chủ (Ice), ghi ở `deferred-work.md`.
    let weave_this_import = docx_sidecar.is_none();
    let mut final_segments: Vec<Vec<crate::core::segment::role::WovenSegment>> =
        Vec::with_capacity(chapters.len());
    // `(chapter_index, block_index) -> neo ĐÃ DỜI`, chỉ với những Chương THẬT SỰ được dệt.
    let mut shifted_anchors: std::collections::HashMap<(usize, usize), i64> =
        std::collections::HashMap::new();
    for (i, chapter) in chapters.iter().enumerate() {
        match &chapter.blocks {
            Some(blocks) if weave_this_import => {
                // Cùng luật `Step::ExtractMainContent`/`prepare_chapter_images` — chỉ Chương
                // ĐẦU TIÊN đọc `block_overrides` (§Never spec 6.9, kế thừa nguyên vẹn).
                let overrides_for_unit: &[Option<bool>] =
                    if i == 0 { &block_overrides_for_images } else { &[] };
                let effective_kept = crate::core::segment::pipeline::effective_kept_for_blocks(
                    blocks,
                    overrides_for_unit,
                );
                let woven = crate::core::segment::role::weave_chapter_segments(
                    blocks,
                    &effective_kept,
                    &chapter.segments,
                    &chapter.source_text,
                    &cleanup_rules_for_images,
                    &source_lang_owned,
                );
                for (block_index, anchor) in woven.shifted_anchor_by_block {
                    shifted_anchors.insert((i, block_index), anchor);
                }
                final_segments.push(woven.segments);
            }
            // Đường KHÔNG có `blocks` (`.txt`, dán tay) HOẶC đường `.docx` (có `blocks` nhưng
            // Quyết định 3 loại nó khỏi lượt dệt) — mọi hàng `role = NULL`, KHÔNG một byte nào
            // đổi so với trước story này (§Always spec 6.13).
            _ => {
                final_segments.push(
                    chapter
                        .segments
                        .iter()
                        .cloned()
                        .map(crate::core::segment::role::WovenSegment::from)
                        .collect(),
                );
            }
        }
    }
    // Neo của MỌI `SavedAsset` vốn tính TRƯỚC KHI DỆT — cập nhật lại bằng neo ĐÃ DỜI trước khi
    // ghi `INSERT INTO asset`. Chỉ ảnh thuộc một Chương THẬT SỰ được dệt (và tính neo thành
    // công ở CẢ HAI lượt tính, `prepare_chapter_images` lẫn `weave_chapter_segments`) mới có
    // mặt trong `shifted_anchors`; đường `.docx`/không `blocks` không đổi gì (neo giữ nguyên).
    for saved in &mut saved_assets {
        if let Some(&shifted) = shifted_anchors.get(&(saved.chapter_index, saved.block_index)) {
            saved.anchor_after_segment_ord = shifted;
        }
    }
    let final_segments = final_segments;
    let saved_assets = saved_assets;
    // Lấy mẫu TRƯỚC khi `chapters` bị `move` vào giao dịch dưới đây — `source_text` ở đây đã
    // qua pipeline (giải mã đúng bảng mã thật), cùng khuôn `append_chapters_to_work`.
    let source_lang_mismatch = chapters
        .iter()
        .any(|chapter| source_lang_looks_mismatched(&source_lang_owned, &chapter.source_text));

    // 🔴 Quyết định #3: job ghi CHỈ SQL — không `fs::write` nào bên trong closure này.
    let write_result = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO work (id, work_id, name, source_lang, genre, created_at, updated_at) \
             VALUES (1, ?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
             strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (&work_id, &name_owned, &source_lang_owned, &genre_owned),
        )?;

        // 🔴 AC13 (không đổi) — segment ghi xuống **CÙNG** giao dịch với hàng `chapter`
        // sinh ra chúng. Segment đã tính SẴN trong `chapter.segments` (bước 7 của chuỗi,
        // chạy trong `run_import` NGOÀI closure này) — không tính lại ở đây.
        //
        // 🔵 SỬA 2026-09-05 (Story 6.6) — "N = 1 ở story này" đã HẾT ĐÚNG: `ord` liên tục từ
        // 1, cùng giao dịch với hàng `work`, cho N THẬT (mẫu phân tách có thể khớp nhiều
        // lần). `OpenWork::chapter_id` chốt vào Chương ĐẦU TIÊN — Story 2.11 xoá hẳn lối
        // suy-ra-động (`ORDER BY ord LIMIT 1`).
        //
        // 🔴 KHÔNG `Option<i64>` + `.expect()` — `chapters` đã được xác nhận KHÔNG RỖNG
        // và `chapters.len()` đã được xác nhận VỪA `i64` ở NGOÀI closure này (vòng rà đối
        // kháng 2026-09-04, item 4). `is_first`/`i as i64` vì thế an toàn theo CẤU TRÚC,
        // không phải theo linh cảm "thực tế không xảy ra".
        let mut first_chapter_id: i64 = 0;
        let mut is_first = true;
        let mut all_chapter_ids: Vec<i64> = Vec::with_capacity(chapters.len());
        for (i, chapter) in chapters.iter().enumerate() {
            let ord = i as i64 + 1;
            // 🔵 SỬA 2026-09-05 (Story 6.6) — `title` bơm từ `chapter.title` (dòng khớp mẫu
            // phân tách, `None` cho Chương lời tựa hoặc khi không có mẫu) thay vì `NULL`
            // cứng.
            //
            // 🔴 THÊM 2026-09-10 (Story 6.15, FR128/AD-43) — bốn cột xuất xứ, ÁP override
            // NGƯỜI DÙNG (nếu có, theo chỉ số Chương `i`) lên trên giá trị MÁY đã bóc
            // (`chapter.chapter_origin`) — xem [`effective_origin_fields`]. Đường tệp/dán tay có
            // `chapter.chapter_origin == None` VÀ `origin_overrides` rỗng ⇒ cả bốn cột `NULL`, KHÔNG
            // BACKFILL, đúng §Always.
            let (origin_author, origin_site_name, origin_url, origin_published_at) =
                effective_origin_fields(chapter.chapter_origin.as_ref(), origin_overrides_owned.get(i).and_then(Option::as_ref));
            // 🔴 **THÊM 2026-09-11 (Story 6.16, §Always)** — Chương của đường song ngữ khởi
            // tạo `InProgress`, không `NotStarted`: nó tới với bản dịch SẴN CÓ (dù chưa xác
            // nhận), khác một Chương văn xuôi vừa nhập chưa ai chạm tới.
            let chapter_status = if chapter.bilingual_segments.is_some() {
                LifecycleStatus::InProgress
            } else {
                LifecycleStatus::NotStarted
            };
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at, \
                 origin_author, origin_site_name, origin_url, origin_published_at) \
                 VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?5, ?6, ?7, ?8)",
                (
                    ord,
                    &chapter.title,
                    &chapter.source_text,
                    chapter_status.as_str(),
                    &origin_author,
                    &origin_site_name,
                    &origin_url,
                    &origin_published_at,
                ),
            )?;

            // `last_insert_rowid()` đọc **trong** giao dịch, ngay sau lượt chèn của chính
            // nó — `Store::write` giữ một writer duy nhất nối tiếp, nên không lượt chèn
            // nào khác chen được vào giữa hai dòng này.
            let chapter_id = tx.last_insert_rowid();
            // 🔴 **THÊM 2026-09-11 (Story 6.16, AD-47 ③)** — đường song ngữ ghi `target_text`
            // + `translation_origin = bilingual_import` trong CÙNG một `INSERT`, qua hàm
            // RIÊNG (§Always: "existing path unchanged") — không đi qua dệt vai (Quyết định 3
            // spec 6.13 loại `.docx`; đường này CŨNG không có `blocks` để mà dệt).
            match &chapter.bilingual_segments {
                Some(segments) => {
                    crate::commands::segment::insert_bilingual_segments(tx, chapter_id, segments)?;
                }
                None => {
                    crate::commands::segment::insert_segments(tx, chapter_id, &final_segments[i])?;
                }
            }

            // 🔴 **THÊM 2026-09-08 (Story 6.11, FR127)** — hàng `asset` của CHÍNH Chương này,
            // CÙNG giao dịch với `chapter`/`segment` (§Always spec 6.11: "ghi SQL đi qua
            // `store::Writer` như mọi lệnh ghi khác"). Tệp đã nằm trên đĩa từ TRƯỚC giao dịch
            // này (`prepare_chapter_images`, NGOÀI closure) — ở đây chỉ còn việc ghi HÀNG.
            for saved in saved_assets.iter().filter(|s| s.chapter_index == i) {
                tx.execute(
                    "INSERT INTO asset (chapter_id, file_name, source_url, \
                     anchor_after_segment_ord, byte_len, content_type, created_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                    (
                        chapter_id,
                        &saved.file_name,
                        &saved.source_url,
                        saved.anchor_after_segment_ord,
                        saved.byte_len,
                        &saved.content_type,
                    ),
                )?;
            }

            if is_first {
                first_chapter_id = chapter_id;
                is_first = false;
            }
            all_chapter_ids.push(chapter_id);
        }

        Ok((first_chapter_id, all_chapter_ids))
    });

    let (chapter_id, new_chapter_ids) = match write_result {
        Ok(ids) => ids,
        Err(err) => {
            store.close();
            remove_folder(&dir);
            return Err(err.into());
        }
    };

    // Quyết định #3: `meta.json` ghi NGAY SAU KHI giao dịch commit, ở tầng THAO TÁC —
    // dựng lại từ `project.db` vừa ghi (AD-33), không giữ dữ liệu song song mà trôi.
    let meta = match WorkMeta::rebuild_from_store(&store) {
        Ok(meta) => meta,
        Err(err) => {
            store.close();
            remove_folder(&dir);
            return Err(err.into());
        }
    };

    // 🔴 Loi ghi meta.json PHAI noi ra, KHONG duoc nuot — code review 2026-08-06.
    //
    // Quyet dinh #3 chap nhan **cua so SAP MAY** giua commit va fs::write, va no dung:
    // AD-33 noi meta.json dung lai duoc tu project.db. Nhung no KHONG cho phep di tiep
    // khi ham TRA VE Err. Hai chuyen khac han nhau:
    //   - sap may  ⇒ khong ai chay duoc ma dep, va lan mo sau dung lai duoc;
    //   - Err      ⇒ tien trinh van song, va di tiep nghia la tra ve Ok cho mot .atproj
    //                chi co HAI thanh phan — pha AC2, va pha AC3 (Library doc metadata
    //                ma khong mo SQLite) ngay tu luc tao.
    //
    // Va duong dung lai KHONG TU CHAY: `rebuild_from_store` khong co mot cho goi san
    // pham nao (story nay khong dung man hinh "mo lai mot .atproj"), nen mot meta.json
    // vang mat nam do cho toi Epic 5.
    //
    // ⇒ Cuon lai TRON VEN. An toan vi `create_work_folder` tao DOC QUYEN: `dir` chac chan
    // la thu muc cua chinh luot goi nay, khong phai du lieu co san.
    if let Err(err) = meta.write_atomic(&dir) {
        // ⚠️ Chẩn đoán KHÔNG lặp lại chuỗi "meta.json" viết thẳng (2026-08-28, Story 5.5) --
        // `meta_write_boundary.rs` khoá tên tệp CHỈ ở `core/library/meta.rs`.
        //
        // 🔵 ĐO (2026-08-28, vòng rà thứ hai) -- lượt đổi chữ này KHÔNG làm người vận hành
        // mất đường lần dấu: `{err}` là `MetaError` mà `write_atomic` trả về, và
        // `MetaError::Io::fmt` (`core/library/meta.rs`) in NGUYÊN đường dẫn đầy đủ, luôn kết
        // thúc bằng `meta.json` (`meta[<duong-dan>/meta.json] io failed: <chi tiet>`). Câu
        // log ở đây chỉ đổi phần TIỀN TỐ mô tả thao tác; tên tệp thật vẫn tới log qua `{err}`.
        eprintln!(
            "project[{}] work metadata cache write failed after commit, rolling back: {err}",
            dir.display()
        );
        store.close();
        remove_folder(&dir);
        return Err(crate::core::library::WorkError::from(err).into());
    }

    let scope = crate::core::scope::ScopeResolver::with_work(crate::core::scope::WorkScope {
        work_id: meta.work_id.clone(),
    });

    Ok(OpenWork {
        dir,
        store,
        scope,
        meta,
        chapter_id,
        images_saved,
        images_failed,
        source_lang_mismatch,
        new_chapter_ids,
    })
}

/// Thin wrapper over [`create_work_with_progress`] with a no-op progress/cancel pair.
#[allow(clippy::too_many_arguments)]
pub fn create_work(
    documents_root: &Path,
    name: &str,
    source_lang: &str,
    genre: &str,
    shape: PipelineShape,
    encoding: &'static encoding_rs::Encoding,
    cleanup_rules: Vec<crate::core::cleanup::CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    block_overrides: Vec<Option<bool>>,
    bilingual_source_column: usize,
    bilingual_target_column: usize,
    bilingual_has_header: bool,
    origin_overrides: &[Option<ChapterOriginOverride>],
    domain_log_state: &webimport::DomainLogState,
    docx_sidecar: Option<crate::core::segment::import::DocxSidecar>,
    regroupings: &[crate::core::segment::bilingual::BilingualRegrouping],
) -> Result<OpenWork, IpcError> {
    create_work_with_progress(
        documents_root,
        name,
        source_lang,
        genre,
        shape,
        encoding,
        cleanup_rules,
        chapter_pattern,
        block_overrides,
        bilingual_source_column,
        bilingual_target_column,
        bilingual_has_header,
        origin_overrides,
        domain_log_state,
        docx_sidecar,
        regroupings,
        &mut |_, _| {},
        &|| false,
    )
}

/// **Hàm thuần** — Story 6.7b (FR122, nửa hai: "hoặc thêm Chương vào một Tác phẩm sẵn có").
/// Em sinh của [`create_work`]: ghi thêm N Chương vào một Tác phẩm ĐÃ CÓ, tại
/// `ord = MAX(ord)+1`, tái dùng ĐÚNG khuôn chèn chapter/segment/asset của `create_work`
/// (`:722-795`) TRONG MỘT giao dịch.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 KHÁC `create_work` Ở ĐÚNG MỘT ĐIỂM SỐNG CÒN — KHÔNG TẠO, KHÔNG XOÁ
/// ─────────────────────────────────────────────────────────────────────────────
/// Hàm này nhận `open: &mut OpenWork` ĐÃ MỞ SẴN — chỗ gọi (`wire::confirm_import_with_encoding`)
/// chịu trách nhiệm phân giải: tái dùng `Store` đang mở trong `OpenWorkState` nếu đích trùng
/// Tác phẩm đang mở, hoặc [`open_work`] nếu không (`src-tauri/AGENTS.md:30` — không bao giờ
/// mở HAI kết nối ghi tới cùng một `project.db`). Không `create_work_folder`, không
/// `Store::open`, không `INSERT INTO work` — và **không một nhánh lỗi nào gọi
/// `remove_folder`**: một Tác phẩm người dùng đã sở hữu từ trước không được phép biến mất vì
/// một lượt nhập TIẾP THEO trượt (§Never spec 6.7b: "thất bại là để Tác phẩm nguyên như cũ,
/// không bao giờ là xoá"). Một lỗi ở BẤT KỲ bước nào bên dưới để `open` NGUYÊN VẸN như trước
/// khi hàm này chạy — bước SQL rollback tự nhiên qua `Store::write` khi closure trả `Err`
/// (không có transaction nào commit nửa chừng).
///
/// ⚠️ Ảnh (nếu có) được TẢI và GHI TỆP TRƯỚC giao dịch SQL (cùng thứ tự `create_work`) — một
/// lỗi SAU bước đó (transaction thất bại) có thể để lại tệp `assets/` không hàng `asset` nào
/// trỏ tới, vì `remove_folder` KHÔNG chạy ở đây để dọn nó. Đây là đánh đổi CÓ CHỦ Ý của chính
/// §Never (không xoá đè lên "không tạo" là an toàn hơn), ghi lại ở Implementation Notes chứ
/// không lặng lẽ bỏ qua.
///
/// Không đụng `open.chapter_id`: Chương đang mở trong trình biên soạn giữ nguyên (§I/O Matrix
/// spec 6.7b "Destination is the open Work ... open editor state stays valid").
/// `open.images_saved`/`open.images_failed` được GHI ĐÈ bằng số của ĐÚNG lượt gọi này (cùng
/// ngữ nghĩa "số của lượt gọi NÀY" mà `create_work` đã theo, không phải một bộ đếm luỹ kế
/// qua nhiều lượt append).
///
/// Chỉ phục vụ BA đường đơn ngữ (dán/tệp/URL, Quyết định 1 spec 6.7b) — không tham số
/// `bilingual_*`/`regroupings`: `shape` không bao giờ là `PipelineShape::Bilingual` trên
/// đường gọi này (đường song ngữ đi qua `confirm_bilingual_import`, không đổi).
///
/// Trả về số Chương đã ghi thêm (`chapters.len()`), để chỗ gọi log/hiển thị không phải đếm lại.
///
/// # Lỗi
/// Y hệt [`create_work`] (bilingual mismatch/0 Chương/N vượt `i64`/pha ảnh/SQL) — chỉ khác ở
/// việc dọn dẹp: **không** `remove_folder` ở bất kỳ nhánh nào.
pub fn append_chapters_to_work_with_progress(
    open: &mut OpenWork,
    source_lang: &str,
    shape: PipelineShape,
    encoding: &'static encoding_rs::Encoding,
    cleanup_rules: Vec<crate::core::cleanup::CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    block_overrides: Vec<Option<bool>>,
    origin_overrides: &[Option<ChapterOriginOverride>],
    domain_log_state: &webimport::DomainLogState,
    docx_sidecar: Option<crate::core::segment::import::DocxSidecar>,
    on_image_progress: &mut dyn FnMut(usize, usize),
    should_cancel: &dyn Fn() -> bool,
) -> Result<usize, IpcError> {
    validate_source_lang(source_lang)?;

    let source_lang_owned = source_lang.to_owned();

    // Cùng điều kiện `create_work` dùng cho `extract_main_content`/`chapter_urls` — xem
    // doc-comment ở đó cho lý lẽ đầy đủ (mục D6/G9 của các vòng rà đối kháng).
    let extract_main_content = matches!(
        &shape,
        PipelineShape::Chapters(cs) if matches!(cs.first(), Some(ChapterInput::RawBytes { .. }))
    );
    let chapter_urls: Vec<String> = match &shape {
        PipelineShape::Blob(c) => vec![chapter_input_page_url(c)],
        PipelineShape::Chapters(cs) => cs.iter().map(chapter_input_page_url).collect(),
        PipelineShape::Bilingual { .. } | PipelineShape::BilingualTables { .. } | PipelineShape::BilingualRows { .. } => {
            vec![String::new()]
        }
        PipelineShape::Files(_) => vec![String::new()],
    };
    let cleanup_rules_for_images = cleanup_rules.clone();
    let block_overrides_for_images = block_overrides.clone();
    let origin_overrides_owned: Vec<Option<ChapterOriginOverride>> = origin_overrides.to_vec();

    let outcome = run_pipeline(
        PipelineInput::with_encoding(shape, encoding, source_lang_owned.clone())
            .with_cleanup_rules(cleanup_rules)
            .with_chapter_pattern(chapter_pattern)
            .with_extract_main_content(extract_main_content)
            .with_block_overrides(block_overrides)
            // Duong nay khong bao gio mang PipelineShape::Bilingual -- 0 cot/0 regrouping,
            // cung khuon `confirm_import_with_encoding`.
            .with_bilingual_columns(0, 1, false)
            .with_bilingual_regroupings(Vec::new()),
    )?;
    if !outcome.bilingual_mismatches.is_empty() {
        let count = outcome.bilingual_mismatches.len();
        return Err(
            crate::core::segment::import::ImportError::BilingualMismatchedRows { count }.into(),
        );
    }
    let mut chapters = outcome.chapters;

    let docx_chapter_images: Vec<ChapterDocxImage> = docx_sidecar
        .as_ref()
        .map(|sidecar| distribute_docx_blocks_across_chapters(sidecar, &mut chapters))
        .unwrap_or_default();

    if extract_main_content && chapters.len() != chapter_urls.len() {
        return Err(crate::core::library::WorkError::CreateFailed {
            detail: format!(
                "bat bien 1:1 giua chapters ({}) va chapter_urls ({}) da vo -- loi lap trinh, \
                 khong phai dieu kien nguoi dung gay ra",
                chapters.len(),
                chapter_urls.len()
            ),
        }
        .into());
    }
    if chapters.is_empty() {
        return Err(crate::core::library::WorkError::CreateFailed {
            detail: "pipeline nhap tra ve 0 Chuong -- khong co gi de them".to_owned(),
        }
        .into());
    }
    if i64::try_from(chapters.len()).is_err() {
        return Err(crate::core::library::WorkError::CreateFailed {
            detail: format!("so Chuong ({}) vuot i64 -- khong the ghi cot ord", chapters.len()),
        }
        .into());
    }

    let image_prep = prepare_chapter_images(
        &open.dir,
        &chapters,
        &chapter_urls,
        &block_overrides_for_images,
        &cleanup_rules_for_images,
        &source_lang_owned,
        domain_log_state,
        &docx_chapter_images,
        on_image_progress,
        should_cancel,
    )?;
    let ImagePrepOutcome { saved: mut saved_assets, images_saved, images_failed } = image_prep;

    // Det vai segment giong het `create_work` (`:630-696`) -- xem doc-comment o do cho ly le
    // day du (Story 6.13).
    let weave_this_import = docx_sidecar.is_none();
    let mut final_segments: Vec<Vec<crate::core::segment::role::WovenSegment>> =
        Vec::with_capacity(chapters.len());
    let mut shifted_anchors: std::collections::HashMap<(usize, usize), i64> =
        std::collections::HashMap::new();
    for (i, chapter) in chapters.iter().enumerate() {
        match &chapter.blocks {
            Some(blocks) if weave_this_import => {
                let overrides_for_unit: &[Option<bool>] =
                    if i == 0 { &block_overrides_for_images } else { &[] };
                let effective_kept = crate::core::segment::pipeline::effective_kept_for_blocks(
                    blocks,
                    overrides_for_unit,
                );
                let woven = crate::core::segment::role::weave_chapter_segments(
                    blocks,
                    &effective_kept,
                    &chapter.segments,
                    &chapter.source_text,
                    &cleanup_rules_for_images,
                    &source_lang_owned,
                );
                for (block_index, anchor) in woven.shifted_anchor_by_block {
                    shifted_anchors.insert((i, block_index), anchor);
                }
                final_segments.push(woven.segments);
            }
            _ => {
                final_segments.push(
                    chapter
                        .segments
                        .iter()
                        .cloned()
                        .map(crate::core::segment::role::WovenSegment::from)
                        .collect(),
                );
            }
        }
    }
    for saved in &mut saved_assets {
        if let Some(&shifted) = shifted_anchors.get(&(saved.chapter_index, saved.block_index)) {
            saved.anchor_after_segment_ord = shifted;
        }
    }
    let final_segments = final_segments;
    let saved_assets = saved_assets;
    let appended_count = chapters.len();
    // Lấy mẫu TRƯỚC khi `chapters` bị `move` vào giao dịch dưới đây — `source_text` ở đây đã
    // qua pipeline (giải mã đúng bảng mã thật), khác `create_work_from_text`/`_from_file` vốn
    // phải đoán trên văn bản THÔ vì không có access tới `ImportedChapter` của riêng chúng.
    let mismatch = chapters
        .iter()
        .any(|chapter| source_lang_looks_mismatched(source_lang, &chapter.source_text));

    // 🔴 Bước 1 của khuôn bốn bước AD-8 (`ARCHITECTURE-SPINE.md`) — CHỈ SQL, MỘT giao
    // dịch. `MAX(ord)` đọc TRONG CÙNG giao dịch với lượt chèn — `Store::write` giữ một writer
    // duy nhất nối tiếp (AD-11), nên không lượt ghi nào khác chen được vào giữa lượt đọc và
    // lượt chèn của chính giao dịch này (§Always spec 6.7b: "New Chapters take ord =
    // MAX(ord) + 1 upward"). KHÔNG `INSERT INTO work` — Tác phẩm đã có hàng đó từ lượt tạo.
    let write_result = open.store.write(move |tx: &Transaction<'_>| {
        let base_ord: i64 =
            tx.query_row("SELECT COALESCE(MAX(ord), 0) FROM chapter", [], |row| row.get(0))?;

        let mut appended_chapter_ids: Vec<i64> = Vec::with_capacity(chapters.len());
        for (i, chapter) in chapters.iter().enumerate() {
            let ord = base_ord + i as i64 + 1;
            let (origin_author, origin_site_name, origin_url, origin_published_at) =
                effective_origin_fields(
                    chapter.chapter_origin.as_ref(),
                    origin_overrides_owned.get(i).and_then(Option::as_ref),
                );
            // §Always spec 6.7b — Chuong moi luon NotStarted: duong nay khong bao gio mang
            // chapter.bilingual_segments (shape khong bao gio la Bilingual, xem doc-comment
            // ham nay), khac `create_work` noi nhanh do con duoc doc toi.
            let chapter_status = LifecycleStatus::NotStarted;
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at, \
                 origin_author, origin_site_name, origin_url, origin_published_at) \
                 VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?5, ?6, ?7, ?8)",
                (
                    ord,
                    &chapter.title,
                    &chapter.source_text,
                    chapter_status.as_str(),
                    &origin_author,
                    &origin_site_name,
                    &origin_url,
                    &origin_published_at,
                ),
            )?;

            let chapter_id = tx.last_insert_rowid();
            crate::commands::segment::insert_segments(tx, chapter_id, &final_segments[i])?;

            for saved in saved_assets.iter().filter(|s| s.chapter_index == i) {
                tx.execute(
                    "INSERT INTO asset (chapter_id, file_name, source_url, \
                     anchor_after_segment_ord, byte_len, content_type, created_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                    (
                        chapter_id,
                        &saved.file_name,
                        &saved.source_url,
                        saved.anchor_after_segment_ord,
                        saved.byte_len,
                        &saved.content_type,
                    ),
                )?;
            }
            appended_chapter_ids.push(chapter_id);
        }

        Ok(appended_chapter_ids)
    });
    let appended_chapter_ids = write_result?;

    // Story 6.7b -- dem HANG cua chinh lot append nay (khong cong don qua nhieu lot goi), cung
    // ngu nghia OpenWork::images_saved/images_failed ma create_work da theo: "so anh cua lot
    // goi NAY", khong phai luy ke ca doi song OpenWork.
    open.images_saved = images_saved;
    open.images_failed = images_failed;
    open.source_lang_mismatch = mismatch;
    open.new_chapter_ids = appended_chapter_ids;

    Ok(appended_count)
}

/// Thin wrapper over [`append_chapters_to_work_with_progress`] with a no-op progress/cancel
/// pair.
#[allow(clippy::too_many_arguments)]
pub fn append_chapters_to_work(
    open: &mut OpenWork,
    source_lang: &str,
    shape: PipelineShape,
    encoding: &'static encoding_rs::Encoding,
    cleanup_rules: Vec<crate::core::cleanup::CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    block_overrides: Vec<Option<bool>>,
    origin_overrides: &[Option<ChapterOriginOverride>],
    domain_log_state: &webimport::DomainLogState,
    docx_sidecar: Option<crate::core::segment::import::DocxSidecar>,
) -> Result<usize, IpcError> {
    append_chapters_to_work_with_progress(
        open,
        source_lang,
        shape,
        encoding,
        cleanup_rules,
        chapter_pattern,
        block_overrides,
        origin_overrides,
        domain_log_state,
        docx_sidecar,
        &mut |_, _| {},
        &|| false,
    )
}

/// **THÊM 2026-09-16 (Story 6.7b, Phase 4)** — lõi THUẦN của việc chọn tầng Tác phẩm cho
/// MỘT đích cụ thể (AC3), tách khỏi `wire::resolve_cleanup_rules_for_destination` để
/// `tests/cleanup_contract.rs` gọi được THẲNG, không cần `tauri::AppHandle`. Vỏ `wire::resolve_cleanup_rules_for_destination` chịu
/// trách nhiệm lấy `destination`/`open` ra khỏi `OpenWorkState`/`Indexer` rồi gọi hàm này;
/// hàm này không đọc bất kỳ state Tauri nào.
///
/// Nhận CẢ `destination` LẪN `open` (Tác phẩm đang mở, nếu có) TƯỜNG MINH — không tự suy
/// `work_id` từ đâu khác. Khi `open` trùng `destination` (cùng `work_id`), TÁI DÙNG store của
/// `open` (không mở lần hai cùng `project.db` — `src-tauri/AGENTS.md:30`, cũng là store DUY
/// NHẤT tồn tại nếu đích đang là Tác phẩm mở); mọi trường hợp khác — kể cả `open` là MỘT Tác
/// phẩm KHÁC đích, hoặc không có gì đang mở — dùng store của CHÍNH `destination`. Đây đúng
/// mệnh đề §Always spec 6.7b "Work-tier cleanup rules resolve from the DESTINATION Work",
/// không phải từ bất kỳ Tác phẩm nào tình cờ đang mở.
///
/// 🔴 Đối chứng (Implementation Notes Phase 1 correction, 2026-09-16): BỎ tham số
/// `destination` (hoặc đổi thân hàm để luôn dùng `open` khi có mặt) quay lại ĐÚNG khiếm
/// khuyết AC3 đang canh — chỗ gọi trong `cleanup_contract.rs` mất chỗ tham chiếu `destination`
/// nên KHÔNG BIÊN DỊCH; nếu thay bằng một bản luôn ưu tiên `open`, ca mismatched-Work đi ĐỎ vì
/// trả về luật của Tác phẩm SAI (`open`/X) cho một đích khác (W).
pub fn resolve_work_tier_cleanup_rules_for_destination(
    destination: &OpenWork,
    open: Option<&OpenWork>,
    global: &Store,
) -> Result<Vec<CleanupRule>, crate::core::cleanup::CleanupStoreError> {
    let work = match open {
        Some(o) if o.meta.work_id == destination.meta.work_id => o,
        _ => destination,
    };
    crate::core::cleanup::resolve_two_tiers(&work.scope, global, Some(&work.store))
}

/// Trích URL của một [`ChapterInput`] cho [`prepare_chapter_images`] — URL của TRANG chứa
/// ảnh, không phải một nhãn chẩn đoán chung chung.
///
/// 🔵 **SỬA (vòng rà đối kháng 2, mục F1) — gọi THẲNG `pipeline::label_of`, không còn một
/// bản chép riêng.** `label_of` từng là một hàm LỒNG riêng tư bên trong
/// `run_import_with_order`, buộc module này phải giữ một bản chép tay 4 dòng y hệt (khoá
/// đồng bộ bằng một cổng test riêng, `chapter_page_url_drift_boundary.rs`). `label_of` nay
/// `pub(crate)` ở module scope — gọi thẳng xoá bản chép VÀ xoá luôn cổng canh trôi (một
/// nguồn sự thật duy nhất không thể trôi khỏi chính nó).
fn chapter_input_page_url(c: &ChapterInput) -> String {
    crate::core::segment::pipeline::label_of(c)
}

/// Một hàng `asset` ĐÃ SẴN SÀNG ghi — tệp đã nằm trên đĩa, chỉ còn thiếu `chapter_id` (chưa
/// biết tới khi giao dịch ghi chèn hàng `chapter` và đọc `last_insert_rowid()`), nên trường
/// đó thay bằng `chapter_index` (chỉ số trong `chapters`, ổn định xuyên suốt `create_work`).
pub(crate) struct SavedAsset {
    pub(crate) chapter_index: usize,
    /// **THÊM 2026-09-09 (Story 6.13)** — chỉ số của khối `Image` trong `chapter.blocks` mà
    /// hàng này sinh ra từ đó. `anchor_after_segment_ord` ngay dưới được tính TRƯỚC khi dệt
    /// segment vai (Quyết định 2 spec 6.13) — sau khi dệt, `create_work` tra
    /// `WovenChapter::shifted_anchor_by_block` bằng CẶP `(chapter_index, block_index)` để cập
    /// nhật lại neo trước khi ghi `INSERT INTO asset`, đúng khuôn dời neo đã ghi ở
    /// `schema.rs:958-975`.
    pub(crate) block_index: usize,
    pub(crate) anchor_after_segment_ord: i64,
    pub(crate) file_name: String,
    /// ⚠️ **NỢ CÓ CHỦ, ghi ra tại chỗ (vòng rà đối kháng 2, mục D1) — đây là URL YÊU CẦU
    /// (`p.resolved_url`, `src` đã phân giải tuyệt đối), KHÔNG PHẢI chặng CUỐI sau chuyển
    /// hướng.** `webimport::fetch`/[`FetchedPage`] không trả lại URL cuối cùng đã dừng ở đó
    /// (chỉ trả `bytes`/`content_type`) — thêm trường đó là một thay đổi chữ ký xuyên
    /// `fetcher.rs`→`fetch_and_write_one_asset`→ở đây, ngoài phạm vi lượt vá nhỏ này. Cột
    /// `source_url` mang HAI VAI cùng lúc (xuất xứ hiển thị cho người dùng VÀ khoá dedup
    /// AD-41 — xem `cache` bên dưới, khoá bằng CHÍNH `resolved_url` này) — nếu sửa để ghi
    /// chặng cuối, vai KHOÁ DEDUP phải tách khỏi vai HIỂN THỊ (hai cột, không một). Một ảnh
    /// mà máy chủ chuyển hướng sang host khác sẽ hiển thị SAI xuất xứ cho người dùng hôm nay.
    /// **Chủ: Story 6.14** (màn hình đầu tiên THẬT SỰ hiển thị `source_url` cho người dùng
    /// đọc) — `deferred-work.md`.
    ///
    /// 🔵 **SỬA 2026-09-09 (Story 6.12) — kiểu đổi từ `String` sang `Option<String>`.** Ảnh
    /// nhúng `.docx` không có URL nào để mà ghi — `NULL` đã hợp lệ theo lược đồ
    /// (`schema.rs` — `ASSET_DDL`, `source_url` là cột DUY NHẤT cho `NULL`, doc-comment tại
    /// đó nói thẳng nó dành cho ảnh không đến từ mạng). `Some(url)` cho đường mạng (Story
    /// 6.11), `None` cho đường `.docx` (Story 6.12).
    pub(crate) source_url: Option<String>,
    pub(crate) byte_len: i64,
    pub(crate) content_type: String,
}

/// Kết quả pha ảnh (Story 6.11) — xem doc-comment [`create_work`] cho VỊ TRÍ nó chạy.
///
/// 🔵 **SỬA 2026-09-08 (mục B1 vòng rà đối kháng 3 lớp) — KHÔNG còn trường `domain_log`.**
/// Bản trước tích luỹ `Vec<DomainLogEntry>` ở đây rồi CHỈ gắn vào `OpenWork` trên đường thành
/// công — một lượt trượt giữa chừng (ví dụ đĩa đầy) trả `Err` sớm và không có kiểu nào chở
/// được `Vec` đó đi cùng nhánh lỗi, nên nhật ký của những ảnh ĐÃ tải xong trước đó biến mất
/// (vi phạm §Always "kể cả lượt trượt"). Mỗi lời gọi `fetch` nay PUSH THẲNG vào
/// `webimport::DomainLogState` (tham số mới của `prepare_chapter_images`) NGAY khi hoàn tất —
/// entry sống sót bất kể phần còn lại của `create_work` có trượt hay không.
struct ImagePrepOutcome {
    saved: Vec<SavedAsset>,
    images_saved: u32,
    images_failed: u32,
}

/// Một URL ảnh đã thử tải TRONG CHÍNH lượt nhập này — `None` nghĩa là đã thử và TRƯỢT.
///
/// 🔴 Cache CẢ chiều thất bại (không chỉ chiều thành công): AD-41 vế "không tải lại ảnh đã
/// có" áp cho MỌI kết quả trong cùng lượt nhập — một host trả 404 lặp lại ở tám Chương không
/// đáng tám lượt gọi mạng giống hệt nhau, đúng tinh thần I/O Matrix "0 lời gọi mạng; dùng lại
/// (kết quả đã có)".
/// 🔵 **THÊM `Clone` 2026-09-09 (Story 6.12)** — cache tra theo URL (`prepare_chapter_images`)
/// nay trả một bản CLONE thay vì một tham chiếu mượn, để cùng một vòng lặp xử lý được CẢ
/// nhánh `Remote` (tra cache) LẪN nhánh `Local` (không cache, không mượn gì) mà không phải
/// hai kiểu trả về khác nhau — ba trường đều rẻ để clone (hai `String` ngắn, một `i64`).
#[derive(Clone)]
struct CachedFetch {
    file_name: String,
    byte_len: i64,
    content_type: String,
}

// An embedded .docx image assigned to its Chapter by
// distribute_docx_blocks_across_chapters; local_block_index indexes into that Chapter's
// own `blocks`, not the whole-document `DocxImage::block_index`.
struct ChapterDocxImage {
    chapter_index: usize,
    local_block_index: usize,
    bytes: Vec<u8>,
    content_type: String,
}

// Slices `sidecar.blocks` (flat across the whole .docx) into one run per Chapter, once
// Step::SplitChapters has fixed the chapter boundaries. Walks blocks in document order,
// advancing to the next chapter once the accumulated text length matches that chapter's
// `source_text.len()` — this must track exactly how `join_kept_blocks` built the joined
// text that `Step::SplitChapters` cut, or images anchor to the wrong chapter.
fn distribute_docx_blocks_across_chapters(
    sidecar: &crate::core::segment::import::DocxSidecar,
    chapters: &mut [crate::core::segment::import::ImportedChapter],
) -> Vec<ChapterDocxImage> {
    if chapters.is_empty() {
        return Vec::new();
    }
    let mut per_chapter_blocks: Vec<Vec<webimport::Block>> = vec![Vec::new(); chapters.len()];
    let mut remap: std::collections::HashMap<usize, (usize, usize)> = std::collections::HashMap::new();
    let mut current = 0usize;
    let mut consumed = 0usize;
    for (orig_idx, block) in sidecar.blocks.iter().enumerate() {
        let text_len = match &block.body {
            webimport::BlockBody::Paragraph(t) | webimport::BlockBody::Caption(t) => t.len(),
            webimport::BlockBody::Image { .. } => 0,
        };
        let original_gap_len = block.exact_gap_before.len();
        // Only real text advances the chapter pointer, so an image sitting exactly at a
        // chapter boundary stays in the current chapter. The gap between chapters is
        // credited to the chapter before it, matching where Step::SplitChapters actually cut.
        if text_len > 0 {
            while current + 1 < chapters.len() && consumed + original_gap_len >= chapters[current].source_text.len() {
                current += 1;
                consumed = 0;
            }
        }
        let local_idx = per_chapter_blocks[current].len();
        // The first block of each chapter carries no inherited exact_gap_before: its
        // original gap was already credited to the previous chapter above.
        let mut block_for_chapter = block.clone();
        let contribution = if local_idx == 0 {
            block_for_chapter.exact_gap_before.clear();
            text_len
        } else {
            original_gap_len + text_len
        };
        remap.insert(orig_idx, (current, local_idx));
        consumed += contribution;
        per_chapter_blocks[current].push(block_for_chapter);
    }
    for (chapter, blocks) in chapters.iter_mut().zip(per_chapter_blocks) {
        chapter.blocks = Some(blocks);
    }
    sidecar
        .images
        .iter()
        .filter_map(|img| {
            remap.get(&img.block_index).map(|&(chapter_index, local_block_index)| ChapterDocxImage {
                chapter_index,
                local_block_index,
                bytes: img.bytes.clone(),
                content_type: img.content_type.clone(),
            })
        })
        .collect()
}

/// Pha ảnh của [`create_work`] (Story 6.11, FR127) — chạy TRƯỚC giao dịch ghi SQL, NGOÀI mọi
/// closure `Store::write`.
///
/// Ba việc, theo đúng thứ tự (mỗi ảnh KEPT của mỗi Chương có `blocks: Some(..)`):
/// 1. Tính neo TRƯỚC KHI thử mạng (`core::segment::anchor::compute_anchor`) — một ảnh không
///    tính được neo thì KHÔNG đáng một lượt tải (tránh ghi một tệp mồ côi không có hàng
///    `asset` nào tham chiếu được tới nó).
/// 2. Phân giải `src` thành URL tuyệt đối (`webimport::assets::resolve_absolute_url`), dựng
///    MỘT [`webimport::Allowlist`] tầng 2 từ đúng những host của các ảnh sẽ thử tải (§Always
///    spec 6.11 — tầng 2 dựng từ ĐÚNG ảnh `effective_kept` trả `true`).
/// 3. Tải qua [`webimport::fetch`] (kèm cache dedup theo URL tuyệt đối, cả hai chiều thành
///    công/thất bại), kiểm MIME, ghi byte xuống `assets/` (đường DUY NHẤT `fs::write` của cả
///    hàm).
///
/// # Lỗi
/// Trả `Err` ở HAI nhánh, cả hai đều là điều kiện KHÔNG THỂ NGƯỜI DÙNG GÂY RA (lỗi lập trình
/// hoặc I/O thật, không phải một MIME/host/mạng xấu):
/// 1. Ghi BYTE xuống đĩa thất bại (`std::fs::write`, ví dụ đĩa đầy) — I/O Matrix spec 6.11
///    hàng "Ghi tệp trượt giữa chừng".
/// 2. 🔵 **THÊM (vòng rà đối kháng 2, mục B1)** — bất biến nội bộ vỡ: `extension_for_mime`
///    xác nhận một MIME thuộc danh mục ĐÓNG nhưng `normalized_mime` (đọc CÙNG `content_type`)
///    lại trả `None` — chứng minh được là KHÔNG THỂ xảy ra hôm nay (cùng phép chuẩn hoá nội
///    bộ), nhưng trả `Err` thay vì `unreachable!()` để phòng một lượt tách rời logic sau này
///    (`panic = "abort"` giết cả tiến trình, một `Err` thì không).
///
/// Cả hai nhánh khiến `create_work` dọn SẠCH `.atproj` (không phải hàm này — nó chỉ trả
/// `Err`, chỗ gọi mới `remove_folder`). MỌI lý do khác (host ngoài tầng 2, MIME sai, mạng
/// lỗi, neo không tính được) chỉ đếm vào `images_failed`, KHÔNG BAO GIỜ trả `Err`.
/// **THÊM 2026-09-09 (Story 6.12)** — nguồn byte của một [`PendingImage`]: mạng (Story 6.11)
/// hoặc byte đã đọc SẴN từ zip `.docx` (Story 6.12, không mạng, không `Allowlist`).
enum PendingImageSource {
    /// URL tuyệt đối đã phân giải — đi qua `fetch_and_write_one_asset` (mạng, dedup theo
    /// URL, Allowlist tầng 2).
    Remote { resolved_url: String },
    /// Byte đã có sẵn trong bộ nhớ (ảnh nhúng `.docx`) — đi thẳng
    /// [`write_local_asset_bytes`], không dedup theo URL (không có URL để mà khoá).
    Local { bytes: Vec<u8>, content_type: String },
}

fn prepare_chapter_images(
    dir: &Path,
    chapters: &[crate::core::segment::import::ImportedChapter],
    chapter_urls: &[String],
    block_overrides: &[Option<bool>],
    cleanup_rules: &[crate::core::cleanup::CleanupRule],
    source_lang: &str,
    domain_log_state: &webimport::DomainLogState,
    // .docx images for every Chapter, assigned by distribute_docx_blocks_across_chapters.
    docx_images: &[ChapterDocxImage],
    // Called after each image fetch/write attempt, success or not.
    on_image_progress: &mut dyn FnMut(usize, usize),
    // Checked before each attempt; true stops the loop immediately, keeping images already
    // saved without counting the rest toward images_failed.
    should_cancel: &dyn Fn() -> bool,
) -> Result<ImagePrepOutcome, IpcError> {
    use crate::core::segment::pipeline::effective_kept_for_blocks;
    use crate::core::webimport::BlockBody;

    /// Một ảnh GIỮ mà neo ĐÃ tính được — sẵn sàng đi vào hàng đợi tải/ghi.
    struct PendingImage {
        chapter_index: usize,
        /// **THÊM 2026-09-09 (Story 6.13)** — xem doc-comment [`SavedAsset::block_index`].
        block_index: usize,
        anchor_after_segment_ord: i64,
        source: PendingImageSource,
    }

    let mut pending: Vec<PendingImage> = Vec::new();
    // 🔵 SỬA 2026-09-09 (vòng rà đối kháng 3, mục R5) — MỌI lượt tăng `images_failed` trong
    // hàm này dùng `saturating_add(1)`, không `+= 1` trần. Trước sửa này `images_saved` được
    // bão hoà có chủ (kèm hẳn một đoạn biện hộ, xem cuối hàm) trong khi `images_failed` +=
    // trần WRAP AROUND về 0 trên release build (Rust chỉ panic-on-overflow ở debug) — cùng
    // MIỀN TRÀN (đếm ảnh trong một lượt nhập) nhưng HAI kỷ luật khác nhau là một sự bất nhất
    // không có lý do. Chọn MỘT: bão hoà ở CẢ HAI, cùng lý lẽ hệt `images_saved` — tràn đòi
    // hơn bốn tỷ ảnh trong MỘT lượt nhập, không một trang thật nào chạm tới, và một con số
    // bão hoà ("ít nhất lớn cỡ này") trung thực hơn một số WRAP về 0 im lặng.
    let mut images_failed: u32 = 0;

    for (i, chapter) in chapters.iter().enumerate() {
        let Some(blocks) = &chapter.blocks else { continue };
        if blocks.is_empty() {
            continue;
        }
        // Cùng luật `Step::ExtractMainContent` (`pipeline.rs`) — chỉ Chương ĐẦU TIÊN đọc
        // `block_overrides` (§Never spec 6.9, kế thừa nguyên vẹn ở đây).
        let overrides_for_unit: &[Option<bool>] = if i == 0 { block_overrides } else { &[] };
        let effective_kept = effective_kept_for_blocks(blocks, overrides_for_unit);
        let page_url = chapter_urls.get(i).map(String::as_str).unwrap_or("");

        for (block_idx, block) in blocks.iter().enumerate() {
            let BlockBody::Image { src, .. } = &block.body else { continue };
            if !effective_kept[block_idx] {
                continue;
            }

            // .docx images carry no URL (`src` is always `None` for that shape); their bytes
            // were already read at import_file time. Matched by (chapter_index, block_idx).
            let local_image = docx_images.iter().find(|d| d.chapter_index == i && d.local_block_index == block_idx);

            if src.is_none() && local_image.is_none() {
                images_failed = images_failed.saturating_add(1);
                eprintln!(
                    "asset[src] anh khong co thuoc tinh src va khong co byte .docx tuong ung \
                     -- bo qua (chuong {i}, khoi {block_idx})"
                );
                continue;
            }

            // Neo TRƯỚC mạng/ghi tệp — một ảnh không neo được thì không đáng một lượt tải
            // (tránh một tệp mồ côi không hàng `asset` nào trỏ tới, xem doc-comment hàm này).
            let anchor_after_segment_ord = match crate::core::segment::anchor::compute_anchor(
                blocks,
                &effective_kept,
                block_idx,
                &chapter.source_text,
                &chapter.segments,
                cleanup_rules,
                source_lang,
            ) {
                Ok(ord) => ord,
                Err(err) => {
                    images_failed = images_failed.saturating_add(1);
                    eprintln!(
                        "asset[neo] tinh neo that bai (chuong {i}, khoi {block_idx}): {}",
                        err.detail
                    );
                    continue;
                }
            };

            let source = if let Some(img) = local_image {
                // Ảnh `.docx` nhúng — 0 mạng, 0 `Allowlist` (§Always spec 6.12).
                PendingImageSource::Local { bytes: img.bytes.clone(), content_type: img.content_type.clone() }
            } else if let Some(src) = src.as_deref().filter(|s| s.starts_with("data:")) {
                // `data:` images decode locally, no network, no Allowlist; MIME is gated to
                // the raster allowlist — SVG is still refused (AD-16).
                match webimport::assets::decode_data_uri_image(src) {
                    Ok((bytes, content_type)) => PendingImageSource::Local { bytes, content_type },
                    Err(_err) => {
                        images_failed = images_failed.saturating_add(1);
                        eprintln!("asset[data] data: URI khong giai ma duoc hoac MIME khong phai raster (chuong {i}, khoi {block_idx})");
                        continue;
                    }
                }
            } else if let Some(src) = src {
                match webimport::assets::resolve_absolute_url(src, page_url) {
                    Ok(resolved_url) => PendingImageSource::Remote { resolved_url },
                    Err(_err) => {
                        images_failed = images_failed.saturating_add(1);
                        // E4 (vòng rà đối kháng 2, 3 lớp) — `ResolveUrlError::detail` nhúng
                        // NGUYÊN VĂN `src` (URL người dùng đã dán/trang chứa) — KHÔNG in ra
                        // stderr, kể cả bản release. Vị trí (chương/khối) đủ để chẩn đoán cục
                        // bộ mà không lộ URL.
                        eprintln!("asset[url] khong phan giai duoc src (chuong {i}, khoi {block_idx})");
                        continue;
                    }
                }
            } else {
                // KHÔNG THỂ xảy ra: điều kiện ngay trên đã xác nhận `src.is_some() ||
                // local_image.is_some()`, và nhánh `local_image` vừa xử ở trên — trả một
                // lỗi ĐẾM được thay vì `unreachable!()` (`panic = "abort"`), cùng khuôn mọi
                // bất biến nội bộ khác của hàm này (xem `# Lỗi` doc-comment).
                images_failed = images_failed.saturating_add(1);
                eprintln!(
                    "asset[bat_bien] nhanh khong the xay ra: src/local_image deu None sau \
                     khi da kiem co nguon (chuong {i}, khoi {block_idx})"
                );
                continue;
            };

            pending.push(PendingImage { chapter_index: i, block_index: block_idx, anchor_after_segment_ord, source });
        }
    }

    if pending.is_empty() {
        return Ok(ImagePrepOutcome { saved: Vec::new(), images_saved: 0, images_failed });
    }

    // 🔴 Tầng 2 dựng từ ĐÚNG những host MẠNG của ảnh sẽ thử tải — không tầng 1 (pha này KHÔNG
    // BAO GIỜ tải Trang, §Always spec 6.11), và không đọc ảnh `.docx` (0 host — chúng không
    // đi qua mạng, §Always spec 6.12: "Không dựng Allowlist nào trên đường này"). Tầng 1
    // rỗng ⇒ `Allowlist::decide` cho `ResourceKind::Page` luôn `Denied`, vô hại vì hàm này
    // chỉ gọi `fetch(..., ResourceKind::Image)`.
    let tier2_hosts: std::collections::BTreeSet<String> = pending
        .iter()
        .filter_map(|p| match &p.source {
            PendingImageSource::Remote { resolved_url } => webimport::assets::host_of(resolved_url),
            PendingImageSource::Local { .. } => None,
        })
        .collect();
    let allowlist = webimport::Allowlist::default().with_tier2_hosts(tier2_hosts);

    let assets_dir = dir.join("assets");
    let mut cache: std::collections::HashMap<String, Option<CachedFetch>> = std::collections::HashMap::new();
    let mut saved: Vec<SavedAsset> = Vec::new();
    let total_pending = pending.len();

    for (pending_idx, p) in pending.iter().enumerate() {
        if should_cancel() {
            eprintln!(
                "asset[cancel] huy giua chung sau {pending_idx}/{total_pending} anh -- giu Chuong \
                 va anh da tai, bo phan con lai (khong dem vao images_failed)"
            );
            break;
        }
        // 🔵 SỬA 2026-09-09 (Story 6.12) — nhánh theo `PendingImageSource`. Dedup theo URL
        // (khuôn Story 6.11) chỉ có nghĩa cho `Remote`; `Local` (ảnh `.docx`) không có URL
        // để mà khoá cache, và ghi thẳng qua [`write_local_asset_bytes`] — 0 mạng, 0
        // `Allowlist`, 0 `DomainLogState` (bảng đó chỉ ghi nhận lượt RA MẠNG, §Always spec 6.8).
        let cached: Option<CachedFetch> = match &p.source {
            PendingImageSource::Remote { resolved_url } => {
                if let Some(existing) = cache.get(resolved_url) {
                    existing.clone()
                } else {
                    // 🔵 B1 — `fetch_and_write_one_asset` PUSH THẲNG vào `domain_log_state`
                    // bên trong nó (trước bước ghi tệp có thể trượt); `?` ở đây không còn
                    // vứt mất một Vec cục bộ nào — cái duy nhất `?` lan ra là `IpcError` của
                    // chính lượt ghi byte trượt, nhật ký thì đã AN TOÀN từ trước đó rồi.
                    let outcome =
                        fetch_and_write_one_asset(resolved_url, &allowlist, &assets_dir, domain_log_state)?;
                    cache.insert(resolved_url.clone(), outcome.clone());
                    outcome
                }
            }
            PendingImageSource::Local { bytes, content_type } => {
                write_local_asset_bytes(bytes, content_type, &assets_dir)?
            }
        };

        match cached {
            Some(c) => saved.push(SavedAsset {
                chapter_index: p.chapter_index,
                block_index: p.block_index,
                anchor_after_segment_ord: p.anchor_after_segment_ord,
                file_name: c.file_name,
                source_url: match &p.source {
                    PendingImageSource::Remote { resolved_url } => Some(resolved_url.clone()),
                    // 🔴 `NULL` — §Always spec 6.12: ảnh `.docx` không đến từ mạng, đúng
                    // nghĩa cột `source_url` đã khai từ Story 6.11 (schema.rs).
                    PendingImageSource::Local { .. } => None,
                },
                byte_len: c.byte_len,
                content_type: c.content_type,
            }),
            None => images_failed = images_failed.saturating_add(1),
        }
        on_image_progress(pending_idx + 1, total_pending);
    }

    // 🔵 THÊM (vòng rà đối kháng 3, mục R2) — kiểm TRƯỚC giao dịch ghi, không để một `CHECK`
    // của `ASSET_DDL` vỡ BÊN TRONG `store.write` giết TRỌN lượt nhập. Một hàng vi phạm bên
    // trong giao dịch làm SQLite trả lỗi, `create_work` dọn SẠCH `.atproj` — đúng cho lỗi
    // GHI ĐĨA (§Always spec 6.11), nhưng SAI cho một hàng dữ liệu hỏng của MỘT ảnh: cùng
    // triết lý §Always "một ảnh trượt không được dừng cả lượt nhập" mà `images_failed` tồn
    // tại để giữ. Lọc Ở ĐÂY (ngoài giao dịch, có thể đếm/log an toàn) thay vì để SQLite từ
    // chối bên trong.
    let (saved, newly_failed) = saved.into_iter().partition::<Vec<_>, _>(saved_asset_satisfies_asset_check_constraints);
    for offender in &newly_failed {
        eprintln!(
            "asset[check] hang asset vi pham CHECK cua ASSET_DDL, bo qua truoc giao dich (chuong {}, file_name={:?})",
            offender.chapter_index, offender.file_name
        );
    }
    images_failed = images_failed.saturating_add(u32::try_from(newly_failed.len()).unwrap_or(u32::MAX));

    // 🔵 THÊM (vòng rà đối kháng 3, mục R6) — phép kiểm VÉT CẠN cho `chapter_index`, ĐO ĐƯỢC
    // là an toàn hôm nay (mỗi `chapter_index` chép THẲNG từ `i` của vòng lặp
    // `chapters.iter().enumerate()` ngay trên, cùng hàm này — không đường nào khác gán nó),
    // nhưng chỗ gọi (`create_work`) lọc theo `chapter_index == i` bằng một `filter()` KHÔNG
    // VÉT CẠN — một `SavedAsset` mang `chapter_index` không khớp BẤT KỲ vòng lặp Chương nào
    // (một lượt tách rời logic tương lai lỡ gán sai) sẽ không được lọc trúng ở BẤT KỲ chỉ số
    // nào: 0 lần `INSERT`, tệp vẫn nằm trên đĩa (đã ghi ở `fetch_and_write_one_asset`, TRƯỚC
    // giao dịch), và `images_saved` (tính NGAY DƯỚI ĐÂY, từ `saved.len()`) đếm THỪA đúng
    // hàng đó — một tệp mồ côi VÀ một con số hiển thị sai, không lỗi nào báo. Lọc Ở ĐÂY
    // (cùng vị trí R2, ngoài giao dịch) thay vì phát hiện muộn sau khi đã ghi xong.
    let chapters_len = chapters.len();
    let (saved, out_of_range) =
        saved.into_iter().partition::<Vec<_>, _>(|s| saved_asset_chapter_index_is_in_range(s, chapters_len));
    for offender in &out_of_range {
        eprintln!(
            "asset[chapter_index] chapter_index ({}) vuot qua so Chuong ({chapters_len}) -- bo qua truoc giao dich, file_name={:?}",
            offender.chapter_index, offender.file_name
        );
    }
    images_failed = images_failed.saturating_add(u32::try_from(out_of_range.len()).unwrap_or(u32::MAX));

    // ⚠️ NÓI RA (vòng rà đối kháng 2, mục D7) — cùng lý lẽ hệt `byte_len` ở
    // `fetch_and_write_one_asset`: `unwrap_or(u32::MAX)` là một con số CHẨN ĐOÁN (số hàng
    // `asset` đã chèn thành công), không phải một VỊ TRÍ — tràn `u32` đòi hơn bốn tỷ ảnh
    // trong MỘT lượt nhập, một con số không một trang/Tác phẩm thật nào chạm tới được. Bão
    // hoà thay vì `Err`/panic ở một nhánh không ai từng chạm là lựa chọn có chủ, không phải
    // một lối tắt.
    let images_saved = u32::try_from(saved.len()).unwrap_or(u32::MAX);
    Ok(ImagePrepOutcome { saved, images_saved, images_failed })
}

/// R2 (vòng rà đối kháng 3, lớp 3) — bản Rust CỦA CHÍNH các `CHECK` mà `ASSET_DDL` khai, đo
/// TRƯỚC khi một hàng đi vào giao dịch ghi. `char::is_whitespace()` của Rust đã đúng thuộc
/// tính Unicode `White_Space` (không cần liệt tay 25 điểm mã như SQL — `trim()` của SQLite
/// chỉ cắt ASCII, `str::trim()` của Rust thì không có giới hạn đó), nên vế này ĐƠN GIẢN HƠN
/// bản SQL, không phải một bản chép kém trung thành hơn.
pub(crate) fn saved_asset_satisfies_asset_check_constraints(saved: &SavedAsset) -> bool {
    // 🔵 SỬA 2026-09-09 (Story 6.12) — `source_url` nay `Option<String>` (NULL hợp lệ cho
    // ảnh `.docx`, xem doc-comment trường đó). Vị từ khớp ĐÚNG CHECK của `ASSET_DDL`:
    // `source_url IS NULL OR trim(source_url, ...) <> ''`.
    let source_url_ok = match &saved.source_url {
        None => true,
        Some(s) => !s.trim().is_empty(),
    };
    !saved.file_name.trim().is_empty()
        && source_url_ok
        && !saved.content_type.trim().is_empty()
        && saved.byte_len >= 0
        && saved.anchor_after_segment_ord >= 0
}

/// R6 (vòng rà đối kháng 3, lớp 3) — vị từ VÉT CẠN cho `chapter_index`: chỗ gọi
/// (`create_work`) chỉ ghi hàng có `chapter_index == i` cho `i` chạy trong `0..chapters_len`
/// (đúng vòng lặp `chapters.iter().enumerate()`) — một hàng nằm NGOÀI dải đó không được lọc
/// trúng ở BẤT KỲ `i` nào, để lại một tệp mồ côi trên đĩa và một `images_saved` đếm thừa.
pub(crate) fn saved_asset_chapter_index_is_in_range(saved: &SavedAsset, chapters_len: usize) -> bool {
    saved.chapter_index < chapters_len
}

/// Tải + ghi ĐÚNG MỘT ảnh (đã qua dedup ở [`prepare_chapter_images`]) — đường DUY NHẤT
/// `fs::write` của cả pha ảnh.
///
/// `Ok(None)` cho MỌI lý do khiến ảnh này không có tệp: mạng lỗi/host bị chặn/HTTP lỗi
/// (`FetchError`), hoặc MIME không phải ảnh raster (§Never spec 6.11: SVG bị loại).
///
/// 🔵 **SỬA (vòng rà đối kháng 3, mục T4) — `Err` không còn CHỈ cho ghi byte thất bại.** Câu
/// cũ ở đây hết đúng từ lượt B1 (vòng rà đối kháng 2): `Err` còn nổ ở nhánh bất biến nội bộ
/// `normalized_mime` vỡ (xem `# Lỗi` của [`prepare_chapter_images`] cho chi tiết cả hai
/// nhánh) — cả hai đều là điều kiện lập trình/I-O thật, không phải một MIME/host/mạng xấu.
///
/// 🔵 **SỬA (vòng rà đối kháng, đo được — không suy luận).** Bản trước gọi
/// [`webimport::assets::is_raster_image_mime`] làm một bước GÁC RIÊNG đứng TRƯỚC
/// [`webimport::assets::extension_for_mime`] — hai vị từ cùng canh một mệnh đề ("MIME này có
/// được chấp nhận không") trên đúng một danh mục bốn phần tử. Đo bằng phép GỠ THẬT: gỡ hẳn
/// nhánh gọi `is_raster_image_mime` khỏi hàm này rồi chạy TRỌN `cargo test --locked` (46
/// binary, kể cả `asset_contract.rs`/`webimport_contract.rs`) — **0 ca đỏ**. Đúng thứ mã CHẾT
/// trên đường sản phẩm: `extension_for_mime` đã TỰ đóng vai trò gác cổng qua nhánh `_ => None`
/// của chính nó, nên gọi `is_raster_image_mime` trước đó không đổi một hành vi quan sát được
/// nào — đúng lớp lỗi "hai nguồn sự thật cho một mệnh đề" (không phải "ca đầu-cuối gác nhầm
/// chỗ": ca đầu-cuối ĐÚNG chỗ, vị từ ĐẦU mới là chỗ thừa). ⇒ Gỡ HẲN nhánh gọi rời đó —
/// `extension_for_mime` MỘT MÌNH là điểm quyết định DUY NHẤT trên đường này.
/// `is_raster_image_mime` VẪN ở lại `core::webimport::assets` (xuất khẩu công khai, tự kiểm
/// bằng test riêng của nó) như một vị từ ĐỘC LẬP cho chỗ gọi tương lai (Story 6.14 hay bất kỳ
/// nơi nào cần hỏi "MIME này có phải ảnh raster hay không" mà không cần đuôi tệp) — nó không
/// còn là một BƯỚC trong luồng ghi ảnh của story này.
///
/// 🔵 **SỬA 2026-09-08 (mục B1 vòng rà đối kháng 3 lớp) — nhận `&DomainLogState`, không còn
/// `&mut Vec`.** `Vec<DomainLogEntry>` do `fetch()` trả về được PUSH THẲNG vào state ngay tại
/// đây — TRƯỚC bước ghi byte (dòng có thể trả `Err` và khiến hàm này thoát sớm). Nhờ vậy nhật
/// ký của lượt gọi NÀY luôn an toàn trước khi có cơ hội bị `?` cuốn mất ở tầng gọi.
/// 🔵 **SỬA 2026-09-09 (Story 6.12) — tách thành HAI NỬA, đúng Task list spec 6.12.** Bản
/// Story 6.11 làm cả hai việc ("lấy byte qua mạng" + "ghi + dựng hàng") trong MỘT hàm; đường
/// `.docx` cần nửa THỨ HAI (ghi + dựng hàng) mà không đi qua nửa ĐẦU (mạng) — xem
/// [`write_local_asset_bytes`] ngay dưới, hàm DUY NHẤT ghi byte xuống đĩa bây giờ. Nửa NÀY
/// (mạng) trả `Ok(None)` cho MỌI lý do khiến ảnh không có byte: mạng lỗi/host bị chặn/HTTP
/// lỗi (`FetchError`), hoặc MIME không phải ảnh raster (§Never spec 6.11: SVG bị loại) —
/// TRƯỚC khi gọi nửa ghi, nên nửa ghi không cần biết gì về mạng/nhật ký domain.
fn fetch_asset_bytes_over_network(
    url: &str,
    allowlist: &webimport::Allowlist,
    domain_log_state: &webimport::DomainLogState,
) -> Result<Option<(Vec<u8>, String)>, IpcError> {
    let (result, mut log) = webimport::fetch(url, allowlist, webimport::ResourceKind::Image);

    let page = match result {
        Ok(page) => page,
        Err(_err) => {
            // E4 (vòng rà đối kháng 2, 3 lớp) — KHÔNG `eprintln!` URL của người dùng ra
            // stderr (kể cả bản release, không điều kiện) — `domain_log_state` NGAY DƯỚI đây
            // đã là đúng kênh cho đúng loại sự kiện này (NFR19 audit trail, session-lifetime,
            // không phải một tệp log vô thời hạn trên đĩa).
            webimport::append_domain_log_entries(domain_log_state, log);
            return Ok(None);
        }
    };

    let Some(_ext) = webimport::assets::extension_for_mime(page.content_type.as_deref()) else {
        // E4 (vòng rà đối kháng 2, 3 lớp) — KHÔNG in URL người dùng ra stderr; `content_type`
        // (một chuỗi MIME, không phải dữ liệu định danh cá nhân) đủ để chẩn đoán cục bộ.
        eprintln!("asset[mime] mime khong phai anh raster: {:?}", page.content_type);
        // 🔵 Story 6.11, mục A (Ice ký 2026-09-08) — chặng này ĐÃ được `fetch()` gán
        // `Fetched` (mạng thành công); sửa lại thành `MimeRejected` TRƯỚC khi push, vì
        // `fetcher.rs` không biết gì về danh mục MIME ảnh (AD-40) — chỉ tầng gọi này biết.
        if let Some(last) = log.last_mut() {
            last.outcome = Some(webimport::DomainLogOutcome::MimeRejected);
        }
        webimport::append_domain_log_entries(domain_log_state, log);
        return Ok(None);
    };
    // `append_domain_log_entries` chạy TRƯỚC bất kỳ nhánh `Err` nào bên dưới — chặng mạng ĐÃ
    // hoàn tất (log đã có nội dung thật) không được phép biến mất chỉ vì một bước XỬ LÝ sau
    // đó (dù chỉ là một bất biến nội bộ, không phải I/O) gặp trục trặc.
    webimport::append_domain_log_entries(domain_log_state, log);

    // MIME đã CHUẨN HOÁ lấy THẲNG từ content-type của phản hồi (D1 vòng rà đối kháng 3 lớp) —
    // không dựng lại từ đuôi tệp. `_ext` vừa được `extension_for_mime` xác nhận là MỘT TRONG
    // BỐN kiểu ĐÃ CHẤP NHẬN, nên `normalized_mime` của CHÍNH `content_type` này không thể trả
    // `None` — bất biến chứng minh được bằng ĐỌC MÃ. Trả `Err` thay vì `unreachable!()` (xem
    // lý lẽ B1 gốc, giữ nguyên qua lượt tách): `panic = "abort"` giết NGUYÊN tiến trình nếu
    // một lượt sau tách rời hai bảng MIME mà quên đổi đồng bộ.
    let Some(normalized_content_type) = webimport::normalized_mime(page.content_type.as_deref()) else {
        eprintln!(
            "asset[mime] bat bien noi bo vo (extension_for_mime nhan {_ext} nhung normalized_mime tra None)"
        );
        return Err(IpcError::new(
            "work.create_failed",
            MessageKey::WorkCreateFailed,
            std::collections::BTreeMap::new(),
            false,
        ));
    };

    Ok(Some((page.bytes, normalized_content_type)))
}

/// Ghi byte ảnh xuống đĩa + dựng [`CachedFetch`] — NỬA HAI của cả đường mạng (Story 6.11)
/// LẪN đường `.docx` (Story 6.12); đường DUY NHẤT `fs::write` của cả pha ảnh.
///
/// `content_type` PHẢI đã là một trong bốn MIME ảnh raster đã chấp nhận (hoặc một biến thể
/// mà [`webimport::assets::normalized_mime`] chuẩn hoá về đúng một trong bốn đó) —
/// `content_type` KHÔNG thuộc danh mục đó (MIME lạ suy từ đuôi tệp media `.docx`, ví dụ
/// `bmp`/`emf`/`wmf`) trả `Ok(None)`, đúng khuôn "MIME ngoài danh mục 4 ⇒ bỏ ảnh,
/// `images_failed`" của I/O Matrix — KHÔNG một nhánh `Err` riêng cho ca này.
///
/// 🔵 **SỬA (vòng rà đối kháng 3, mục T4, kế thừa qua lượt tách 2026-09-09)** — `Err` không
/// CHỈ cho ghi byte thất bại: bất biến nội bộ `normalized_mime` vỡ (xem `# Lỗi` doc-comment
/// `prepare_chapter_images`) cũng trả `Err`, cùng lý do B1 gốc (`panic = "abort"`).
fn write_local_asset_bytes(
    bytes: &[u8],
    content_type: &str,
    assets_dir: &Path,
) -> Result<Option<CachedFetch>, IpcError> {
    let Some(ext) = webimport::assets::extension_for_mime(Some(content_type)) else {
        eprintln!("asset[mime] mime khong phai anh raster: {content_type:?}");
        return Ok(None);
    };
    let Some(normalized_content_type) = webimport::normalized_mime(Some(content_type)) else {
        eprintln!(
            "asset[mime] bat bien noi bo vo (extension_for_mime nhan {ext} nhung normalized_mime tra None)"
        );
        return Err(IpcError::new(
            "work.create_failed",
            MessageKey::WorkCreateFailed,
            std::collections::BTreeMap::new(),
            false,
        ));
    };

    let file_name = format!("{}.{ext}", Uuid::new_v4());
    let path = assets_dir.join(&file_name);
    // D3 (vòng rà đối kháng 3 lớp) — CỐ Ý KHÔNG dùng khuôn ghi-nguyên-tử của
    // `core::library::meta::Meta::write_atomic`/`core::glossary::exchange_io::write_export_file`
    // (tạm cạnh đích → sync → rename) ở đây — xem lý lẽ đầy đủ tại doc-comment gốc (giữ
    // nguyên qua lượt tách): `path` là một tên UUID hoàn toàn MỚI, KHÔNG ai đọc nó cho tới khi
    // hàng `INSERT INTO asset` được COMMIT; một `fs::write` trượt giữa chừng khiến chỗ gọi
    // (`prepare_chapter_images` → `create_work`) `remove_folder(&dir)` xoá NGUYÊN `.atproj`.
    if let Err(e) = std::fs::write(&path, bytes) {
        eprintln!("asset[ghi] ghi byte anh xuong dia that bai ({}): {e}", path.display());
        return Err(IpcError::new(
            "asset.write_failed",
            MessageKey::AssetWriteFailed,
            std::collections::BTreeMap::new(),
            false,
        ));
    }

    // ⚠️ NÓI RA (vòng rà đối kháng 2, mục D7) — `unwrap_or(i64::MAX)` là một lượt LÀM TRÒN có
    // chủ (giữ nguyên qua lượt tách): một con số CHẨN ĐOÁN (`byte_len`), tràn KHÔNG THỂ XẢY RA
    // trong thực tế trên đường mạng (`fetcher.rs::MAX_RESPONSE_BYTES` 20 MiB) lẫn đường
    // `.docx` (`MAX_IMPORT_BYTES` 100 MB chặn cả tệp `.docx`, một ảnh nhúng không thể lớn hơn
    // chính tệp chứa nó).
    Ok(Some(CachedFetch {
        file_name,
        byte_len: i64::try_from(bytes.len()).unwrap_or(i64::MAX),
        content_type: normalized_content_type,
    }))
}

/// Tải + ghi ĐÚNG MỘT ảnh MẠNG (đã qua dedup ở [`prepare_chapter_images`]) — hợp hai nửa
/// [`fetch_asset_bytes_over_network`] + [`write_local_asset_bytes`]. Đường `.docx` KHÔNG gọi
/// hàm này — nó gọi thẳng [`write_local_asset_bytes`] (§Always spec 6.12: "Không dựng
/// Allowlist nào trên đường này").
fn fetch_and_write_one_asset(
    url: &str,
    allowlist: &webimport::Allowlist,
    assets_dir: &Path,
    domain_log_state: &webimport::DomainLogState,
) -> Result<Option<CachedFetch>, IpcError> {
    let Some((bytes, content_type)) = fetch_asset_bytes_over_network(url, allowlist, domain_log_state)? else {
        return Ok(None);
    };
    write_local_asset_bytes(&bytes, &content_type, assets_dir)
}

/// Số ký tự CHỮ (CJK + Latin) tối đa lấy mẫu cho [`source_lang_looks_mismatched`] — một trần
/// chi phí, không phải một ngưỡng độ chính xác: script-ratio hội tụ rất nhanh, không cần đọc
/// hết một Chương dài để có tín hiệu đáng tin.
const LANGUAGE_HEURISTIC_SAMPLE_CHARS: usize = 20_000;

/// Danh mục ĐÓNG — `create_work`/`append_chapters_to_work` từ chối MỌI
/// `source_lang` ngoài hai giá trị này, không ghi một byte nào. Trước bản vá này chỉ webview
/// kiểm (`=== 'zh'`, `src/modes/libraryImport.ts`); một `.atproj` sửa tay hoặc một lời gọi
/// `invoke` trần với giá trị lạ đi thẳng xuống đĩa.
fn validate_source_lang(source_lang: &str) -> Result<(), IpcError> {
    if matches!(source_lang, "zh" | "en") {
        Ok(())
    } else {
        Err(crate::core::segment::import::ImportError::UnsupportedSourceLang {
            source_lang: source_lang.to_owned(),
        }
        .into())
    }
}

/// `true` khi ký tự `c` thuộc một khối chữ Hán thường gặp trong văn bản Trung văn — thô, đủ
/// cho một heuristic tỉ lệ chữ, KHÔNG phải một bảng Unicode đầy đủ (bỏ qua các khối hiếm như
/// CJK Extension B+ — một heuristic CẢNH BÁO không cần phủ hết, chỉ cần không dương/âm tính
/// giả trên văn bản Trung văn/Latin THƯỜNG GẶP).
fn is_common_cjk_char(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0xF900..=0xFAFF)
}

/// Tỉ lệ CJK trên tổng số ký tự CHỮ (CJK + chữ cái ASCII) trong `LANGUAGE_HEURISTIC_SAMPLE_CHARS`
/// ký tự đầu của `text` — `None` khi mẫu không có ký tự CHỮ nào (toàn số/khoảng trắng/dấu câu),
/// ca mà một tỉ lệ không nói lên điều gì.
fn cjk_ratio_sample(text: &str) -> Option<f64> {
    let (mut cjk, mut latin) = (0usize, 0usize);
    for c in text.chars().take(LANGUAGE_HEURISTIC_SAMPLE_CHARS) {
        if is_common_cjk_char(c) {
            cjk += 1;
        } else if c.is_alphabetic() {
            latin += 1;
        }
    }
    let total = cjk + latin;
    if total == 0 { None } else { Some(cjk as f64 / total as f64) }
}

/// Heuristic tỉ lệ chữ cho cảnh báo KHÔNG CHẶN — `true` khi nội dung trông như KHÁC hẳn
/// `source_lang` đã khai. Không phải một bộ phát hiện ngôn
/// ngữ đúng nghĩa (không phân biệt được tiếng Việt với tiếng Anh, ví dụ) — chỉ đủ bắt ca dán
/// nhầm cả một Chương chữ Latin vào một Tác phẩm khai `zh`, hoặc ngược lại. `source_lang`
/// ngoài `zh`/`en` (giá trị lạ) không có gì để so — luôn `false`.
fn source_lang_looks_mismatched(source_lang: &str, text: &str) -> bool {
    const CJK_MAJORITY: f64 = 0.5;
    let Some(ratio) = cjk_ratio_sample(text) else { return false };
    match source_lang {
        "zh" => ratio < CJK_MAJORITY,
        "en" => ratio > CJK_MAJORITY,
        _ => false,
    }
}

/// **Hàm thuần** — nhánh dán văn bản của AC1.
///
/// 🔵 **KHÔNG đổi hành vi (Story 6.3)** — vẫn khai UTF-8 cứng qua `encoding_rs::UTF_8` (văn
/// bản dán tay là `ChapterInput::AlreadyText`, bước giải mã bỏ qua vế transcode cho hình
/// dạng đó dù tham số này là gì — xem doc-comment `pipeline::decode_unit`). Đường sản phẩm
/// CÓ xem trước bảng mã là `wire::confirm_import_with_encoding`; hàm này ở lại cho
/// `tests/**` và mọi chỗ gọi không đi qua màn xem trước.
pub fn create_work_from_text(
    documents_root: &Path,
    name: &str,
    source_lang: &str,
    genre: &str,
    text: String,
) -> Result<OpenWork, IpcError> {
    // Đường Blob KHÔNG BAO GIỜ có ảnh (extract_main_content luôn false cho hình dạng này) —
    // một kho tạm, vứt đi ngay sau lượt gọi, đúng khuôn doc-comment `create_work`.
    //
    // `validate_source_lang`/`source_lang_looks_mismatched` chạy BÊN TRONG `create_work` —
    // hàm này không tự lặp lại, cùng lý do mọi chỗ gọi khác của nó (`confirm_import_with_encoding`,
    // `confirm_bilingual_import`, đường URL) đều được canh MIỄN PHÍ qua điểm gọi chung đó.
    create_work(
        documents_root,
        name,
        source_lang,
        genre,
        import_text(text),
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        // Đường dán văn bản không bao giờ là `PipelineShape::Bilingual` — mặc định (0, 1,
        // false) không bao giờ được đọc.
        0,
        1,
        false,
        &[],
        &std::sync::Mutex::new(Vec::new()),
        // Văn bản dán tay không bao giờ có một `DocxSidecar` — xem doc-comment kiểu đó.
        None,
        // Đường dán văn bản không bao giờ là `PipelineShape::Bilingual` — 0 regrouping nào
        // để mà đọc.
        &[],
    )
}


/// **Hàm thuần** — nhánh tệp của AC1 (kéo-thả **hoặc** ô nhập đường dẫn; cả hai đã
/// resolve thành một `path` thật ở lớp gọi, xem AD-1/AD-16).
///
/// 🔵 **KHÔNG đổi hành vi (Story 6.3)** — vẫn khai UTF-8 cứng. Đường sản phẩm CÓ xem trước
/// bảng mã là `wire::confirm_import_with_encoding`; hàm này ở lại cho `tests/**` và mọi chỗ
/// gọi không đi qua màn xem trước (cùng lý do `create_work_from_text`).
///
/// # Lỗi
/// Một phần mở rộng chưa nhận (không phải `.txt`/`.md`/`.docx`) ⇒ `import.unsupported_format`
/// (AC8), **trước khi** thư mục `.atproj` được tạo — [`import_file`] từ chối theo phần mở
/// rộng trước khi mở tệp. 🔵 **SỬA 2026-09-09 (Story 6.12)** — `.docx` không còn ví dụ ở đây,
/// nó nay ĐƯỢC nhận.
pub fn create_work_from_file(
    documents_root: &Path,
    name: &str,
    source_lang: &str,
    genre: &str,
    path: &Path,
) -> Result<OpenWork, IpcError> {
    let (shape, docx_sidecar) = import_file(path)?;
    // Đường Blob KHÔNG BAO GIỜ có ảnh MẠNG — `.docx` (Story 6.12) CÓ THỂ có ảnh NHÚNG, mang
    // trong `docx_sidecar`, truyền NGUYÊN VẸN xuống `create_work`. `validate_source_lang`/
    // `source_lang_looks_mismatched` chạy BÊN TRONG `create_work` — xem doc-comment
    // `create_work_from_text` cho lý do hàm này không tự lặp lại.
    create_work(
        documents_root,
        name,
        source_lang,
        genre,
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        // `import_file` never yields `PipelineShape::Bilingual` (that shape is
        // `import_bilingual_file`'s alone) — defaults unread.
        0,
        1,
        false,
        &[],
        &std::sync::Mutex::new(Vec::new()),
        docx_sidecar,
        // `import_file` never yields `PipelineShape::Bilingual` — 0 regrouping to read.
        &[],
    )
}


#[cfg(test)]
mod distribute_docx_blocks_tests {
    use super::*;
    use crate::core::segment::import::ImportedChapter;

    fn chapter(source_text: &str) -> ImportedChapter {
        ImportedChapter {
            source_text: source_text.to_owned(),
            segments: Vec::new(),
            cleanup_report: None,
            title: None,
            blocks: None,
            joined_line_count: None,
            chapter_origin: None,
            bilingual_segments: None,
            source_file: None,
        }
    }

    fn paragraph(text: &str, gap: &str) -> webimport::Block {
        webimport::Block {
            body: webimport::BlockBody::Paragraph(text.to_owned()),
            machine_kept: true,
            exact_gap_before: gap.to_owned(),
            is_list_item: false,
        }
    }

    fn image() -> webimport::Block {
        webimport::Block {
            body: webimport::BlockBody::Image { src: None, alt: None },
            machine_kept: true,
            exact_gap_before: String::new(),
            is_list_item: false,
        }
    }

    #[test]
    fn splits_blocks_and_remaps_image_indices_at_a_chapter_boundary() {
        // Chuong 2 carries one MORE paragraph before its image than Chuong 1 does, so the two
        // images land at DIFFERENT local_block_index values (2 vs 3) — asserting the exact
        // (chapter_index, local_block_index) pair per image catches this function assigning a
        // block to the wrong chapter even when the two chapters have a different shape, not
        // only the symmetric case both prior fixtures happened to use.
        let blocks = vec![
            paragraph("Chuong 1: Mo Dau", ""),
            paragraph("Anh dau tien o day.", "\n\n"),
            image(),
            paragraph("Chuong 2: Tiep Theo", "\n\n"),
            paragraph("Anh thu hai o day.", "\n\n"),
            paragraph("Mot doan them truoc anh thu hai.", "\n\n"),
            image(),
        ];
        let sidecar = crate::core::segment::import::DocxSidecar {
            blocks: blocks.clone(),
            images: vec![
                crate::core::docx::DocxImage { block_index: 2, bytes: vec![1], content_type: "image/png".to_owned() },
                crate::core::docx::DocxImage { block_index: 6, bytes: vec![2], content_type: "image/png".to_owned() },
            ],
        };
        let mut chapters = vec![
            // Matches Step::SplitChapters's real shape: the gap between chapters stays in
            // the earlier chapter's source_text.
            chapter("Chuong 1: Mo Dau\n\nAnh dau tien o day.\n\n"),
            chapter("Chuong 2: Tiep Theo\n\nAnh thu hai o day.\n\nMot doan them truoc anh thu hai."),
        ];

        let images = distribute_docx_blocks_across_chapters(&sidecar, &mut chapters);

        let c0 = chapters[0].blocks.as_ref().expect("chuong 0 phai co blocks");
        let c1 = chapters[1].blocks.as_ref().expect("chuong 1 phai co blocks");
        assert_eq!(c0.len(), 3, "chuong 0: title + doan + anh");
        assert_eq!(c1.len(), 4, "chuong 1: title + doan + doan them + anh");
        assert!(matches!(c0[2].body, webimport::BlockBody::Image { .. }), "anh 1 phai o cuoi chuong 0");
        assert!(matches!(c1[3].body, webimport::BlockBody::Image { .. }), "anh 2 phai o cuoi chuong 1");
        assert_eq!(c1[0].exact_gap_before, "", "khoi dau cua chuong 1 khong duoc mang gap ke thua tu vi tri toan tai lieu");

        assert_eq!(images.len(), 2);
        assert_eq!((images[0].chapter_index, images[0].local_block_index), (0, 2));
        assert_eq!((images[1].chapter_index, images[1].local_block_index), (1, 3));
    }
}
