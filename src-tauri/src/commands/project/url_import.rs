//! Nhập từ URL bằng danh sách link (Story 6.7, AD-15 · AD-40 · AD-41 · FR122) — tách ra
//! khỏi `commands/project/mod.rs`, không đổi hành vi.
//!
//! `use super::*;` mang mọi kiểu/hàm dùng chung của `commands::project` vào đây. Khối
//! Tầng 2/xuất xứ Chương (`Tier2BlockOverridesState`, `ChapterOriginOverride`,
//! `effective_origin_fields`, …) VẪN sống ở `mod.rs` — nó đan xen với bảng mã và Chương,
//! không phải một trong ba khối tự chứa (xem `deferred-work.md`), nên nằm GIỮA hai nửa
//! của tệp này theo đúng thứ tự đã có.

use super::*;

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.7 — Nhập từ URL bằng danh sách link (AD-15 · AD-40 · AD-41 · FR122)
// ═════════════════════════════════════════════════════════════════════════════════
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 KHÔNG MỘT ĐƯỜNG `create_work` THỨ HAI — TÁI DÙNG TOÀN BỘ MÁY XEM TRƯỚC BẢNG MÃ CŨ
// ─────────────────────────────────────────────────────────────────────────────
// N link đã tải thành công dựng ĐÚNG một [`PipelineShape::Chapters`] — hình dạng
// `preview_import_encoding`/`confirm_import_with_encoding` (Story 6.3) đã biết xử từ
// `chapters.first()` (xem `PipelineShape::Chapters` trong `preview_import_encoding` —
// "danh sách URL là Story 6.7", nhánh đó viết sẵn từ Story 6.3). [`start_url_import`] vì
// thế KHÔNG gọi [`create_work`] trực tiếp: nó `stash_pending_import_source` đúng như hai
// nhánh dán-văn-bản/tệp, và màn xác nhận CUỐI CÙNG đi qua [`confirm_import_with_encoding`]
// KHÔNG ĐỔI MỘT DÒNG — dải năm ứng viên bảng mã, khối làm sạch (tầng 3), khối tách Chương
// (tầng 4, ở đây luôn hiện ĐÚNG N Chương vì `already_chaptered = true` khiến bước 5 bỏ qua)
// đều MIỄN PHÍ. Điều kiện DUY NHẤT: `PendingImportSourceState` phải được ĐỒNG BỘ với danh
// sách mục hiện tại SAU MỌI thao tác (tải, tải lại một mục, bỏ một mục) — xem
// [`sync_pending_from_url_items`].

/// Một MỤC trong danh sách URL đang chờ — byte thô THÀNH CÔNG, HOẶC một [`IpcError`] mang
/// đúng MỘT trong tám lý do của [`WebImportItemFailureReason`]. Đúng MỘT trong hai, không
/// cả hai — không kiểu Rust nào ép được bất biến "đúng một trong hai trường" ở ĐÂY mà không
/// một `enum` (giữ `struct` phẳng để `UrlImportItemWire::from` không phải `match`), nên
/// mọi hàm DỰNG giá trị này (không phải `derive`) đều đặt ĐÚNG MỘT trong `raw`/`error`.
#[derive(Debug, Clone)]
pub struct UrlImportItem {
    /// URL đã dán — TRIM, không rỗng (dòng rỗng bị lọc TRƯỚC khi tới đây).
    pub url: String,
    /// Byte HTML thô — `Some` khi tải THÀNH CÔNG và `content-type` là HTML.
    pub raw: Option<Vec<u8>>,
    /// Lý do thất bại — `Some` khi `raw` là `None`.
    pub error: Option<IpcError>,
}

/// Trạng thái từng mục, sống CẠNH [`PendingImportSourceState`] trong bộ nhớ (§Task list spec
/// 6.7) — `None` == chưa có lượt dán URL nào đang treo, cùng khuôn `PendingImportSourceState`.
pub type UrlImportItemsState = std::sync::Mutex<Option<Vec<UrlImportItem>>>;

/// P6 (vòng rà đối kháng bước 4) — dọn [`UrlImportItemsState`] khi và CHỈ KHI `create_work`
/// vừa THÀNH CÔNG. Trước bản vá này, `wire::confirm_import_with_encoding` chỉ dọn
/// [`PendingImportSourceState`] (`*guard = None` trong [`confirm_import_with_encoding`] ở
/// trên) — `UrlImportItemsState` không hề bị chạm ở bất kỳ đâu ngoài ba lệnh dây
/// `start_url_import`/`reload_url_import_item`/`remove_url_import_item`. Hệ quả: byte HTML
/// thô của N link nằm lại trong bộ nhớ sau khi Tác phẩm đã tạo xong, và một danh sách CŨ vẫn
/// còn đó nếu người dùng mở lại màn nhập URL. **Hàm thuần, `pub`**.
/// `tests/project_contract.rs` calls it directly; no case drives it through the wire.
pub fn clear_url_import_items_after_successful_confirm(items_state: &UrlImportItemsState) {
    let mut guard = items_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = None;
}

/// Hình dạng DÂY của [`UrlImportItem`] — vị trí là INDEX trong `Vec` (frontend giữ nguyên
/// thứ tự, không sắp lại), `error` mang `IpcError` ĐẦY ĐỦ (khoá i18n + tham số) để frontend
/// dịch bằng `tError()`, cùng khuôn mọi lỗi IPC khác của dự án.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct UrlImportItemWire {
    pub url: String,
    pub ok: bool,
    pub error: Option<IpcError>,
}

impl From<&UrlImportItem> for UrlImportItemWire {
    fn from(item: &UrlImportItem) -> Self {
        UrlImportItemWire { url: item.url.clone(), ok: item.error.is_none(), error: item.error.clone() }
    }
}

/// Progress event name for the page-fetch phase (`wire::start_url_import`).
pub const URL_IMPORT_PAGE_PROGRESS_EVENT: &str = "url_import_page_progress";
/// Progress event name for the image-download phase (`create_work`/
/// `append_chapters_to_work`'s image loop, after every link has been fetched).
pub const URL_IMPORT_IMAGE_PROGRESS_EVENT: &str = "url_import_image_progress";

/// Shared payload for both progress events above. `completed`/`total` count in that
/// phase's own unit (links for the page phase, images for the image phase); `completed`
/// is 1-based, i.e. how many are done after the just-finished one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct ImportProgressEvent {
    pub completed: usize,
    pub total: usize,
}

/// Kết quả trả về của cả ba lệnh (tải danh sách, tải lại một mục, bỏ một mục) — danh sách
/// mục HIỆN TẠI cộng xem trước bảng mã khi TOÀN BỘ đã OK. `encoding_preview: None` là điều
/// kiện đủ để frontend biết nút xác nhận phải khoá (đồng bộ với
/// [`sync_pending_from_url_items`] — cùng điều kiện, không suy luận riêng ở tầng hiển thị).
#[derive(Debug, Clone, serde::Serialize)]
pub struct UrlImportBatchWire {
    pub items: Vec<UrlImportItemWire>,
    pub encoding_preview: Option<ImportEncodingPreview>,
    /// 🔵 **THÊM Story 6.8** — số domain PHÂN BIỆT trong nhật ký của CẢ PHIÊN CHẠY (không chỉ
    /// lượt gọi vừa rồi) tại thời điểm trả lời — chân màn xem trước đọc trực tiếp trường này
    /// (mockup *"Đã gọi **N** domain · xem"*), không một lệnh IPC thứ hai chỉ để có một số.
    /// `wire::start_url_import`/`reload_url_import_item`/`remove_url_import_item` cùng tính
    /// qua `wire::domain_log_domain_count` — ba vỏ, một nguồn.
    pub domain_log_domain_count: usize,
    /// Duplicate links (per [`webimport::assets::normalize_url_for_dedup`]) merged away
    /// during the latest [`wire::start_url_import`]; `0` for the other four wires, which
    /// edit an already-deduped list rather than admitting new links.
    pub duplicate_urls_dropped: usize,
}

/// Hình dạng DÂY của một [`webimport::DomainLogEntry`] — Story 6.8, NFR19. `kind`/`decision`
/// đi qua như DỮ LIỆU (chuỗi định danh máy, AD-21), KHÔNG một câu — cùng khuôn
/// `CleanupRuleTierWire` (`Global`/`Work`, `cleanupTierLabelKey` phía TS ánh xạ sang câu).
/// Frontend dựng câu "vì sao được phép/từ chối" bằng một hàm THUẦN có nhánh mặc định, không
/// nhận nguyên văn từ Rust.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DomainLogEntryWire {
    pub at_epoch_ms: u64,
    pub domain: String,
    /// `"page"` | `"image"` — khớp [`webimport::ResourceKind`] (biến thể `Page`/`Image`),
    /// `serde(rename_all = "snake_case")` trên chính kiểu đó (định nghĩa ở
    /// `core::webimport::allowlist`).
    ///
    /// 🔵 SỬA 2026-09-09 (vòng rà đối kháng 2, mục D8) — câu cũ khai `"document"`, một chuỗi
    /// KHÔNG BAO GIỜ thật sự đi qua dây (lỗi từ Story 6.8): biến thể Rust là `Page`, và
    /// `snake_case` của `Page` là `"page"`, không phải `"document"`.
    pub kind: webimport::ResourceKind,
    pub allowed: bool,
    /// `"tier1"` | `"tier2"` | `"denied"` — tầng đã CẤP PHÉP khi `allowed == true`; luôn
    /// `"denied"` khi `allowed == false`. Một trường DUY NHẤT (không `Option<Tier>` +
    /// `bool` rời) để phía TS không phải tự đối chiếu hai trường có nhất quán không.
    pub tier: &'static str,
    /// **THÊM 2026-09-08 (Story 6.11)** — `null` khi `allowed == false` (0 kết nối, không có
    /// kết quả mạng nào để mà báo); một trong tám chuỗi `snake_case` của
    /// [`webimport::DomainLogOutcome`] khi `allowed == true`. Đây là trường trả lời "và rồi
    /// SAO" cho một chặng đã cho phép — thứ hai ô `Error Handling` của I/O Matrix spec 6.11
    /// đòi mà `tier`/`allowed` một mình không nói được.
    pub outcome: Option<webimport::DomainLogOutcome>,
}

impl From<&webimport::DomainLogEntry> for DomainLogEntryWire {
    fn from(entry: &webimport::DomainLogEntry) -> Self {
        let (allowed, tier) = match entry.decision {
            webimport::DomainLogDecision::Allowed(webimport::Tier::One) => (true, "tier1"),
            webimport::DomainLogDecision::Allowed(webimport::Tier::Two) => (true, "tier2"),
            webimport::DomainLogDecision::Denied => (false, "denied"),
        };
        DomainLogEntryWire {
            at_epoch_ms: entry.at_epoch_ms,
            domain: entry.domain.clone(),
            kind: entry.kind,
            allowed,
            tier,
            outcome: entry.outcome,
        }
    }
}

/// Tải MỘT URL và phân loại kết quả thành [`UrlImportItem`] — **0 dòng phân tích nội dung
/// ngoài việc đọc header `content-type`** (kiểm giao thức, không phải nội dung; xem
/// doc-comment [`webimport::looks_like_html`]). Trả kèm nhật ký domain phát sinh từ CHÍNH
/// lượt gọi này (§Always spec 6.8: "một bản ghi cho mọi lời gọi") — chỗ gọi nối vào
/// [`webimport::DomainLogState`] của phiên chạy; hàm này (và [`webimport::fetch`] bên dưới
/// nó) không tự ghi vào state Tauri nào — cả hai đều là hàm THUẦN.
///
/// 🔴 **`ResourceKind::Page` LUÔN LUÔN** — đây là chỗ gọi sản phẩm DUY NHẤT của
/// [`webimport::fetch`] hôm nay, và nó tải đúng những gì người dùng đã dán (một bài viết),
/// không bao giờ một ảnh. `ResourceKind::Image` chỉ có mặt trong kiểu để bốn mệnh đề bắt
/// buộc của AD-41 phát biểu được (xem doc-comment đầu `core::webimport::allowlist`) — Story
/// 6.11 sẽ là chỗ gọi sản phẩm ĐẦU TIÊN của nó.
pub(crate) fn fetch_url_import_item(url: &str, allowlist: &webimport::Allowlist) -> (UrlImportItem, Vec<webimport::DomainLogEntry>) {
    let (result, log) = webimport::fetch(url, allowlist, webimport::ResourceKind::Page);
    let item = match result {
        Ok(page) => {
            if webimport::looks_like_html(page.content_type.as_deref()) {
                UrlImportItem { url: url.to_owned(), raw: Some(page.bytes), error: None }
            } else {
                UrlImportItem {
                    url: url.to_owned(),
                    raw: None,
                    error: Some(web_import_item_failure_ipc_error(url, WebImportItemFailureReason::NotHtml, None)),
                }
            }
        }
        Err(e) => {
            let http_status = match &e {
                webimport::FetchError::HttpStatus { status } => Some(*status),
                _ => None,
            };
            let reason = WebImportItemFailureReason::from(e);
            UrlImportItem {
                url: url.to_owned(),
                raw: None,
                error: Some(web_import_item_failure_ipc_error(url, reason, http_status)),
            }
        }
    };
    (item, log)
}

/// Bước ĐẦU VÀO — TRIM mỗi dòng, BỎ dòng rỗng khi đếm (I/O Matrix spec 6.7: "Dòng rỗng bỏ
/// khi đếm; dòng rác thành mục hỏng"), rồi tải TUẦN TỰ ĐÚNG THỨ TỰ đã dán. **Hàm thuần** —
/// không `tauri::`, `tests/**` gọi được trực tiếp.
/// Cắt hai đầu một dòng dán vào **đúng như `String.prototype.trim()` của JS làm** — thứ
/// `src/modes/libraryImport.ts::pastedUrlLines` dùng để đếm hai con số *N link · N Chương*.
///
/// 🔴 **ĐO 2026-09-07 (vòng rà bước 4) — `str::trim()` của Rust KHÔNG bằng `trim()` của JS,
/// và chỗ lệch nằm đúng trên một ký tự người dùng hay dán phải.** Đo trực tiếp cả hai bên trên
/// bốn ký tự: `U+00A0`, `U+2028`, `U+200B` cho kết quả GIỐNG nhau, nhưng **`U+FEFF`** (BOM /
/// zero-width no-break space) thì JS coi là khoảng trắng còn Rust **không** (`char::is_whitespace`
/// theo thuộc tính Unicode `White_Space`, và `U+FEFF` không có thuộc tính đó).
///
/// Hai ca hỏng THẬT mà chỗ lệch này sinh ra, cả hai đều đánh vào bất biến trung tâm của story:
/// ① một dòng CHỈ có `U+FEFF` — JS cắt thành rỗng nên KHÔNG đếm, Rust giữ nên sinh THÊM một
/// mục ⇒ *N link* trên màn hình khác số Chương sắp tạo, đúng thứ AC4 dựng một test để bắt.
/// ② một URL hợp lệ mang BOM ở ĐẦU (dán từ Windows/Excel/trang web là ca thường gặp) — JS cắt
/// nên màn hình đếm nó là link hợp lệ, Rust giữ nên `Url::parse` trượt ⇒ mục hỏng oan, nút xác
/// nhận khoá, và người dùng không có cách nào nhìn ra một ký tự vô hình.
///
/// ⇒ Cắt thêm `U+FEFF`. Không mở rộng gì khác: ba ký tự còn lại đã khớp, và một phép cắt RỘNG
/// HƠN JS lại tạo ra chỗ lệch theo chiều ngược lại.
///
/// 🔵 **2026-09-15 (spec AI-7, D1)** — câu kết luận TRÊN không còn đúng nguyên văn: hàm này
/// nay dựng trên [`webimport::chapter_origin_trim`], luật cắt CHUNG cho bốn cột
/// `ChapterOrigin` — và luật đó KHÔNG dừng ở "thêm `U+FEFF`", nó còn BỎ `U+0085` (NEL) khỏi
/// tập cắt, vì JS `.trim()` không coi NEL là khoảng trắng còn `str::trim()`/`is_whitespace`
/// của Rust thì có. Đây là một phép mở rộng RỘNG HƠN ba ký tự đã đo hôm 2026-09-07 — nhưng
/// vẫn đúng hướng cảnh báo của câu trên ("một phép cắt RỘNG HƠN JS lại tạo chỗ lệch NGƯỢC"):
/// D1 đo lại toàn bộ 0x0..=0x10FFFF trên cả hai máy JS trước khi mở rộng, nên tập mới KHÔNG
/// rộng hơn JS — nó ĐÚNG BẰNG JS. Một dòng dán chỉ có `U+0085` trước bản vá này được `N link`
/// đếm là 0 mục (bị cắt oan) trong khi `pastedUrlLines` phía JS giữ nó là 1 dòng — chính chỗ
/// lệch NGƯỢC CHIỀU mà D1 đóng, xem `webimport_contract.rs`.
fn trim_like_the_paste_box(line: &str) -> &str {
    webimport::chapter_origin_trim(line)
}

fn trim_and_filter_urls(urls: Vec<String>) -> Vec<String> {
    urls.into_iter().map(|s| trim_like_the_paste_box(&s).to_owned()).filter(|s| !s.is_empty()).collect()
}

/// Maximum links accepted per paste; more are refused outright, never silently truncated.
pub const MAX_URL_IMPORT_LINKS: usize = 200;

fn too_many_urls_error(count: usize) -> IpcError {
    let mut params = std::collections::BTreeMap::new();
    params.insert("count".to_owned(), count.to_string());
    params.insert("limit".to_owned(), MAX_URL_IMPORT_LINKS.to_string());
    IpcError::new(
        "import.too_many_urls",
        crate::core::i18n::MessageKey::ImportTooManyUrls,
        params,
        false,
    )
}

/// Prepares the pasted link list before fetching: trims/drops blank lines (same as
/// [`fetch_url_import_items`]), refuses the WHOLE list when it exceeds
/// [`MAX_URL_IMPORT_LINKS`] (nothing is fetched, nothing silently truncated), then dedups
/// by [`webimport::assets::normalize_url_for_dedup`], keeping each key's first occurrence
/// in paste order.
///
/// Kept separate from [`fetch_url_import_items`] so performance probes can still call the
/// raw fetch path directly, bypassing this cap.
///
/// # Errors
/// More than [`MAX_URL_IMPORT_LINKS`] lines after trim/filter ⇒ `import.too_many_urls`,
/// no URL is fetched.
pub fn prepare_url_import_list(urls: Vec<String>) -> Result<(Vec<String>, usize), IpcError> {
    let trimmed = trim_and_filter_urls(urls);
    if trimmed.len() > MAX_URL_IMPORT_LINKS {
        return Err(too_many_urls_error(trimmed.len()));
    }

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut duplicates_dropped = 0usize;
    let deduped: Vec<String> = trimmed
        .into_iter()
        .filter(|u| {
            let key = webimport::assets::normalize_url_for_dedup(u).unwrap_or_else(|| u.clone());
            if seen.insert(key) { true } else {
                duplicates_dropped += 1;
                false
            }
        })
        .collect();

    Ok((deduped, duplicates_dropped))
}

/// 🔵 **Story 6.8** — chữ ký đổi: trả kèm nhật ký domain của CẢ lượt (`log`), và tự dựng
/// [`webimport::Allowlist`] từ CHÍNH `urls` đã TRIM/lọc rỗng — allowlist một-lần-nhập đúng
/// theo CẤU TẠO (§Design Notes spec 6.8): không có `Allowlist` nào tồn tại NGOÀI thân hàm
/// này, nên không có gì để mà rò rỉ sang lượt nhập kế tiếp.
///
/// Thin wrapper over [`fetch_url_import_items_with_progress`] with a no-op progress
/// callback.
pub fn fetch_url_import_items(urls: Vec<String>) -> (Vec<UrlImportItem>, Vec<webimport::DomainLogEntry>) {
    fetch_url_import_items_with_progress(urls, &mut |_, _| {})
}

/// Same as [`fetch_url_import_items`], plus `on_progress(completed, total)` called after
/// each link fetch, success or not.
pub fn fetch_url_import_items_with_progress(
    urls: Vec<String>,
    on_progress: &mut dyn FnMut(usize, usize),
) -> (Vec<UrlImportItem>, Vec<webimport::DomainLogEntry>) {
    let trimmed = trim_and_filter_urls(urls);
    let allowlist = webimport::Allowlist::from_urls(trimmed.iter().map(String::as_str));
    let total = trimmed.len();

    let mut log = Vec::new();
    let items = trimmed
        .into_iter()
        .enumerate()
        .map(|(idx, u)| {
            let (item, entries) = fetch_url_import_item(&u, &allowlist);
            log.extend(entries);
            on_progress(idx + 1, total);
            item
        })
        .collect();
    (items, log)
}

/// Allowlist cho một lượt TẢI LẠI một mục — dựng từ URL của **TOÀN BỘ** danh sách hiện tại
/// (`items`), không chỉ URL của mục đang tải lại: allowlist là một-lần-NHẬP, và tải lại vẫn
/// thuộc CÙNG lần nhập với lượt [`fetch_url_import_items`] ban đầu (`UrlImportItemsState`
/// giữ nguyên danh sách đó giữa các lượt gọi — xem doc-comment [`UrlImportItemsState`]).
pub(crate) fn allowlist_from_items(items: &[UrlImportItem]) -> webimport::Allowlist {
    webimport::Allowlist::from_urls(items.iter().map(|it| it.url.as_str()))
}

/// Dựng [`PipelineShape::Chapters`] từ `items` khi và CHỈ KHI danh sách KHÔNG rỗng, KHÔNG mục
/// nào mang lỗi, VÀ mọi mục không-lỗi thật sự mang `raw` — `None` khi còn một mục hỏng (đúng
/// lúc nút xác nhận phải KHOÁ, §Always spec 6.7), danh sách rỗng (đóng vế còn mở của nợ
/// `:9067`), hoặc một mục vỡ bất biến "đúng một trong `raw`/`error` là `Some`" (không kiểu nào
/// cưỡng chế bất biến đó — 🔴 vỡ thì hàm này KHÔNG được lặng lẽ dựng một Chương RỖNG từ
/// `unwrap_or_default()`, đó chính là lớp lỗi "rỗng im lặng" mà `AGENTS.md` gọi là trung tâm
/// của dự án; trả `None` giống hệt đường "còn mục hỏng" là lựa chọn AN TOÀN duy nhất). **Hàm
/// thuần, `pub`** để `tests/project_contract.rs` gọi trực tiếp — chứng minh "còn MỘT mục hỏng
/// ⇒ 0 hàng ghi xuống" mà không cần dựng một `tauri::AppHandle`.
pub fn chapters_shape_if_all_ok(items: &[UrlImportItem]) -> Option<PipelineShape> {
    if items.is_empty() || items.iter().any(|it| it.error.is_some()) {
        return None;
    }
    let mut chapters = Vec::with_capacity(items.len());
    for it in items {
        let bytes = it.raw.clone()?;
        chapters.push(ChapterInput::RawBytes { bytes, label: it.url.clone() });
    }
    Some(PipelineShape::Chapters(chapters))
}

/// **THÊM Story 6.10a** — vị từ **XEM**, tách khỏi [`chapters_shape_if_all_ok`] (vị từ **GHI**,
/// GIỮ NGUYÊN Ở TRÊN, không đụng — đường `create_work` vẫn khoá cho tới khi MỌI mục sạch).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO CẦN MỘT HÀM THỨ HAI — "XEM ĐƯỢC" VÀ "GHI ĐƯỢC" LÀ HAI MỆNH ĐỀ KHÁC NHAU
/// ─────────────────────────────────────────────────────────────────────────────
/// Trước story này, `url_import_encoding_preview` dùng CHÍNH [`chapters_shape_if_all_ok`] —
/// nên "một mục hỏng" đồng thời làm CẢ nút xác nhận khoá LẪN màn xem trước biến mất hoàn
/// toàn, cho 4 Chương ĐÃ TẢI được rất tốt. Hàm này dựng [`PipelineShape::Chapters`] từ CÁC
/// MỤC OK — bỏ QUA mục hỏng, không đợi TOÀN BỘ danh sách sạch mới có gì đó để mà xem — và
/// `items` KHÔNG hề bị đụng vào (mục hỏng vẫn đứng NGUYÊN vị trí của nó trong danh sách,
/// hàm này chỉ ĐỌC để lọc, không sắp lại/xoá gì ở `items`).
///
/// `None` khi KHÔNG mục OK nào (mọi mục đều hỏng — I/O Matrix: "rỗng CÓ LÝ DO", không có gì
/// để mà xem, khác hẳn "sẵn sàng ghi").
///
/// 🔴 Kết quả hàm này KHÔNG BAO GIỜ được dùng để quyết định nút xác nhận khoá hay không — chỉ
/// [`chapters_shape_if_all_ok`] mới có quyền đó (xem chỗ gọi ở `sync_pending_from_url_items`,
/// KHÔNG đổi). Phía frontend, tín hiệu khoá nút đọc thẳng `UrlImportItemWire::ok` của TỪNG
/// mục (`items.every(ok)`), KHÔNG còn đọc `encoding_preview === null` — trộn hai tín hiệu đó
/// lại đúng là lỗi mà §Always story 6.10a cấm.
pub fn chapters_shape_for_view(items: &[UrlImportItem]) -> Option<PipelineShape> {
    let chapters: Vec<ChapterInput> = items
        .iter()
        .filter_map(|it| it.raw.clone().map(|bytes| ChapterInput::RawBytes { bytes, label: it.url.clone() }))
        .collect();
    if chapters.is_empty() { None } else { Some(PipelineShape::Chapters(chapters)) }
}

/// Đồng bộ [`PendingImportSourceState`] với `items` HIỆN TẠI — gọi lại sau MỌI thao tác đổi
/// danh sách (tải lần đầu, tải lại một mục, bỏ một mục). Còn mục hỏng hoặc danh sách rỗng ⇒
/// DỌN ô đang chờ (một lượt `confirm_import_with_encoding` kế tiếp trả `no_pending_source`
/// — nút xác nhận khoá THẬT, không chỉ khoá ở tầng hiển thị). Toàn bộ mục OK ⇒ GHI ĐÈ ô đang
/// chờ bằng [`PipelineShape::Chapters`] mới dựng từ CHÍNH danh sách này.
///
/// 🔴 **THÊM 2026-09-16 (Story 6.7b)** — tham số `destination_work_id`. `start_url_import`
/// (mở phiên) truyền giá trị NGƯỜI DÙNG vừa chọn; `reload_url_import_item`/
/// `remove_url_import_item` (giữa phiên, danh sách đổi nhưng đích thì KHÔNG) đọc giá trị
/// HIỆN CÓ qua [`current_pending_destination`] rồi truyền LẠI nguyên vẹn — xem doc-comment
/// [`PendingImportSource::destination_work_id`] cho lý lẽ đầy đủ.
pub(crate) fn sync_pending_from_url_items(
    pending: &PendingImportSourceState,
    items: &[UrlImportItem],
    destination_work_id: Option<String>,
) {
    match chapters_shape_if_all_ok(items) {
        // Đường URL không bao giờ mang một `DocxSidecar` (đó là đường tệp `.docx`, Story
        // 6.12) — `None` cố định.
        Some(shape) => stash_pending_import_source_for(pending, shape, None, destination_work_id),
        None => cancel_import_preview(pending),
    }
}

/// Xem trước bảng mã cho danh sách URL — `None` khi KHÔNG mục OK nào/danh sách rỗng (I/O
/// Matrix: "rỗng có lý do"). Chốt từ đơn vị ĐẦU — [`PipelineInput::encoding`] doc-comment "một
/// bảng mã cho cả danh sách".
///
/// 🔵 **SỬA 2026-09-08 (Story 6.10a) — đọc [`chapters_shape_for_view`] (vị từ XEM), KHÔNG còn
/// [`chapters_shape_if_all_ok`] (vị từ GHI).** Trước bản sửa này, một mục hỏng làm CẢ hàm này
/// trả `None` — bốn Chương ĐÃ TẢI tốt biến mất khỏi màn xem trước chỉ vì một mục THỨ NĂM
/// hỏng. Nút xác nhận khoá KHÔNG còn phụ thuộc kết quả hàm này (xem doc-comment
/// [`chapters_shape_for_view`]) — `sync_pending_from_url_items` (đường GHI) vẫn gọi
/// [`chapters_shape_if_all_ok`] y nguyên.
/// 🔴 **THÊM 2026-09-08 (Story 6.10)** — đếm số mục `error.is_some()` của CẢ `items` rồi
/// truyền vào [`preview_import_encoding`] — đây là chỗ DUY NHẤT tính `broken_item_count` cho
/// đường URL, đọc thẳng từ `items` (không từ danh sách Chương — link hỏng không bao giờ vào
/// đó, §Always spec 6.10).
pub(crate) fn url_import_encoding_preview(
    items: &[UrlImportItem],
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    block_overrides: &[Option<bool>],
    origin_overrides: &[Option<ChapterOriginOverride>],
) -> Option<ImportEncodingPreview> {
    let shape = chapters_shape_for_view(items)?;
    let broken_item_count = items.iter().filter(|it| it.error.is_some()).count();
    Some(preview_import_encoding(
        &shape,
        source_lang,
        cleanup_rules,
        None,
        block_overrides,
        broken_item_count,
        origin_overrides,
    ))
}

/// Dựng [`UrlImportBatchWire`] từ trạng thái HIỆN TẠI — dùng chung bởi cả ba lệnh
/// (tải/tải lại/bỏ một mục) VÀ hai lệnh MỚI Story 6.9 (đặt/gỡ override một khối, đặt dải) để
/// không có năm lượt lắp dây khác nhau cho CÙNG một hình dạng. `domain_log_domain_count` đi
/// vào từ THAM SỐ (đọc từ [`webimport::DomainLogState`] ở lớp vỏ) — hàm này ở lại **thuần**,
/// không tự cầm `AppHandle`/`State` nào.
///
/// 🔴 **THÊM 2026-09-07 (Story 6.9) — tham số `block_overrides`.** Đường URL là đường DUY
/// NHẤT `extract_main_content == true` đi qua được (§Always spec 6.7/6.9) — tầng 2 CHỈ có
/// nghĩa ở đây, nên đây CŨNG là chỗ DUY NHẤT override cần chảy vào lượt xem trước.
pub(crate) fn url_import_batch_wire(
    items: &[UrlImportItem],
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    block_overrides: &[Option<bool>],
    domain_log_domain_count: usize,
    // 🔴 THÊM 2026-09-10 (Story 6.15) — cùng lý do `block_overrides` ngay trên: đường URL là
    // đường DUY NHẤT `extract_main_content == true`, nên đây CŨNG là chỗ DUY NHẤT xuất xứ có
    // gì để mà hiện.
    origin_overrides: &[Option<ChapterOriginOverride>],
    duplicate_urls_dropped: usize,
) -> UrlImportBatchWire {
    UrlImportBatchWire {
        items: items.iter().map(UrlImportItemWire::from).collect(),
        encoding_preview: url_import_encoding_preview(
            items,
            source_lang,
            cleanup_rules,
            block_overrides,
            origin_overrides,
        ),
        domain_log_domain_count,
        duplicate_urls_dropped,
    }
}

/// Lỗi dự phòng cho một nhánh KHÔNG NÊN xảy ra trên đường sản phẩm — state Tauri chưa được
/// `.manage(...)` (lỗi lắp dây ở `lib.rs`), hoặc một `index` ngoài phạm vi danh sách hiện tại
/// (frontend luôn gửi một index đọc từ CHÍNH mảng nó đang hiện). Cùng khuôn
/// [`ImportError::InvalidPipelineOrder`]/`InvalidCleanupPattern` — [`MessageKey::Unknown`],
/// không tự đúc một khoá mới cho một nhánh không chỗ gọi SẢN PHẨM nào đi qua.
pub(crate) fn url_import_internal_error() -> IpcError {
    IpcError::new(
        "import.web_internal_error",
        crate::core::i18n::MessageKey::Unknown,
        std::collections::BTreeMap::new(),
        false,
    )
}

/// `Tier2BlockOverridesState` only carries block structure for the FIRST Chapter, so
/// editing block overrides is allowed only there.
pub(crate) fn tier2_edit_allowed(detail_chapter_index: usize) -> bool {
    detail_chapter_index == 0
}

/// Distinct from [`url_import_internal_error`]: this is not a wiring bug but a real user
/// state (cursor on a Chapter other than the first), which the webview shows via
/// `message_key` in the StatusBar.
pub(crate) fn tier2_edit_locked_to_first_chapter() -> IpcError {
    IpcError::new(
        "import.tier2_edit_locked_to_first_chapter",
        crate::core::i18n::MessageKey::ImportTier2EditLockedToFirstChapter,
        std::collections::BTreeMap::new(),
        false,
    )
}

#[cfg(test)]
mod tier2_edit_allowed_tests {
    use super::tier2_edit_allowed;

    #[test]
    fn only_chapter_zero_is_allowed() {
        assert!(tier2_edit_allowed(0));
        assert!(!tier2_edit_allowed(1));
        assert!(!tier2_edit_allowed(5));
    }
}
