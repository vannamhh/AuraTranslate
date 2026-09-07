//! `Extractor` — byte HTML ĐÃ GIẢI MÃ (văn bản, không phải byte thô) → mô hình khối có thứ tự
//! CHO CẢ TRANG. **0 dòng chạm mạng** — không `reqwest`, không `TcpStream`, không literal
//! `http://`/`https://` ở đây (`webimport_boundary.rs` canh mệnh đề này bằng cách quét TĨNH
//! tệp này).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 `Article::content` (HTML) KHÔNG BAO GIỜ ĐƯỢC ĐỌC — AD-16 §Rule mục 1/2
//! ─────────────────────────────────────────────────────────────────────────────
//! [`extract`] chỉ đọc `Article::text_content` (văn bản thuần); `Article::content`
//! (`StrTendril`, HTML) không được gán vào biến, không được trả về, không được ghi log —
//! `webimport_boundary.rs` canh "0 dòng đọc trường đó" bằng cách quét literal `.content` trong
//! tệp này (bất kỳ dạng `x.content` nào). Hệ quả: ta KHÔNG biết trực tiếp phần tử HTML nào
//! Readability đã giữ lại — chỉ biết VĂN BẢN cuối cùng của phần được giữ. Xem §Design Notes
//! spec 6.9 "Vì sao mô hình phải chở khối ĐÃ BỊ LOẠI" cho lý do kiến trúc, và phần dưới đây
//! cho CƠ CHẾ suy ra khối nào được giữ chỉ từ `text_content`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! CƠ CHẾ — hai lượt phân tích HTML, đối chiếu bằng SO KHỚP CHÍNH XÁC sau chuẩn hoá khoảng
//! trắng (KHÔNG một hằng ngưỡng, đúng §Ask First spec 6.9)
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. `Readability::parse()` chạy như cũ, cho `article.text_content` — văn bản CỦA RIÊNG phần
//!    Readability giữ lại, đã chuẩn hoá khoảng trắng qua `TextMode::Formatted`
//!    (`dom_query::NodeRef::formatted_text`, xem doc-comment cũ ở dưới cho lý do dùng
//!    `Formatted`).
//! 2. `dom_query::Document::from(html)` phân tích LẠI **HTML GỐC** (Readability sửa TRỰC TIẾP
//!    cây của chính nó khi `parse()` chạy — khối bị loại không còn để mà duyệt nếu dùng lại
//!    cây đó — nên đây PHẢI là một lượt phân tích thứ hai, độc lập, tốn kém nhưng không có
//!    đường nào khác). Duyệt mọi phần tử "khối mang chữ" (xem [`is_block_candidate_tag`]) theo
//!    ĐÚNG thứ tự tài liệu.
//! 3. Với mỗi khối mang CHỮ (không phải `<img>`), chuẩn hoá khoảng trắng của văn bản riêng nó
//!    rồi TÌM chuỗi đó làm chuỗi con của `text_content`. Tìm thấy ⇒ khối này CÓ CƠ HỘI
//!    [`Block::machine_kept`]; không thấy ở đâu cả ⇒ khối bị loại (`ornament`). Đây là so khớp
//!    CHÍNH XÁC (không một hằng ngưỡng "gần đúng bao nhiêu %") — đúng lớp lỗi mà §Design Notes
//!    `extractor.rs:32-36` (bản cũ) đã từ chối cho `is_probably_readable()`.
//!    🔴 **SỬA 2026-09-07 (vòng rà spec 6.9) — "con trỏ chỉ tiến, khớp ĐẦU TIÊN tìm thấy" ĐÃ
//!    ĐO SAI trên `a03.html` (mẫu bàn đo 6.1).** Một mục điều hướng ngắn ("欧洲" — "Châu Âu",
//!    2 ký tự) trùng NGẪU NHIÊN với một từ THẬT xuất hiện muộn trong bài viết; khớp-đầu-tiên
//!    tham lam gán nó vào đó, đẩy con trỏ vượt qua ~1920 ký tự nội dung THẬT đứng TRƯỚC —
//!    30 đoạn văn thật liền sau đó không còn tìm thấy gì (con trỏ đã đi qua), bị gắn nhầm
//!    `ornament`. Đối chứng đo: `extract_covers_all_seven_bench_fixtures_without_losing_headings_or_list_items`
//!    (`webimport_contract.rs`) đỏ ở đúng mẫu này trước bản vá. Đường sửa KHÔNG một hằng
//!    ngưỡng: [`assign_matches_by_max_weight_order_preserving_dp`] tìm phép gán TOÀN CỤC tối
//!    ưu — trong mọi cách gán mỗi khối vào MỘT trong các vị trí khớp CHÍNH XÁC của nó sao cho
//!    thứ tự (theo chỉ số khối) được giữ và các dải không đè nhau, chọn cách gán TỐI ĐA HOÁ
//!    tổng độ dài khớp. Đây vẫn là so khớp CHÍNH XÁC (0 khoan dung), chỉ khác ở CHỖ khớp được
//!    chọn khi có nhiều lựa chọn — một phép tối ưu tổ hợp, không phải một tham số phải đoán.
//! 4. Khoảng trắng THẬT (byte-for-byte) đứng giữa hai khối `machine_kept` LIỀN NHAU trong
//!    `text_content` được CHỤP LẠI (`Block::exact_gap_before`) — đây là thứ cho phép
//!    `core::segment::pipeline::join_kept_blocks` ghép lại ĐÚNG TỪNG BYTE với `text_content`
//!    khi không ai sửa gì (AC "trùng đúng đầu ra Story 6.7" của spec 6.9), mà KHÔNG cần đoán
//!    lại luật xuống-dòng-đơn/-đôi của `dom_smoothie` (nó phụ thuộc cấu trúc lồng nhau — xem
//!    `dom_query::node::text_formatting::format_text` — không tái tạo được từ một danh sách
//!    khối PHẲNG).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO `TextMode::Formatted`, KHÔNG PHẢI MẶC ĐỊNH `Raw`
//! ─────────────────────────────────────────────────────────────────────────────
//! **Task 1 — ĐO TRƯỚC KHI VIẾT (2026-09-06, Story 6.7).** `TextMode::Raw` (mặc định của
//! `dom_smoothie::Config`) nối các đoạn liền nhau — mất ranh giới. Đo thật trên bảy trang
//! HTML cache của bàn đo 6.1 (`_bmad-output/implementation-artifacts/6-1-ban-do/fixtures/html/`)
//! với `TextMode::Formatted`: số lần "xuống hai dòng liên tiếp" trong `text_content` khớp
//! (lệch đúng 1, do N đoạn có N-1 ranh giới) với số thẻ `<p` xấp xỉ trong `article.content`
//! trên SÁU trong bảy mẫu (a01: 6↔5, a02: 8↔7, a03: 41↔45, a04: 5↔4, a05: 33↔35, a06: 9↔8);
//! mẫu còn lại (a07) là chính trang KHÔNG PHẢI bài viết mà bàn đo 6.1 đã ghi nhận — số liệu
//! hỗn loạn ở đó là kỳ vọng đúng, không phải một thất bại của phép đo. Kết luận: `Formatted`
//! GIỮ được ranh giới đoạn bằng `"\n\n"` — bước 4 (`normalize::normalize`, luật gộp dòng)
//! và bước 5 (tách Chương) vẫn tính đúng trên cấu trúc dòng. Không DỪNG.
//!
//! 🔵 **SỬA 2026-09-07 (Story 6.9) — phủ tag "0 khối" đã HẾT ĐÚNG cho `a07`.** Bản Story 6.7
//! chỉ đọc `article.text_content` PHẲNG (không mô hình khối) nên "loại bộ chọn nào" chưa có ý
//! nghĩa. Đo lại 2026-09-07 (vòng rà spec 6.9): bảy mẫu có 4–34 thẻ `<p>` nhưng 4–10 tiêu đề
//! (`h1`-`h6`) và 37–118 mục `<li>` — `a07` có **0** thẻ `<p>` (đúng mẫu "không phải bài viết"
//! Task 1 đã ghi nhận) nhưng vẫn có nội dung dạng danh sách/tiêu đề. Bộ chọn khối của story
//! này ([`is_block_candidate_tag`]) vì thế phủ CẢ heading/`li`/`blockquote`/`pre`/`div` lá,
//! không riêng `p`/`img`/`figcaption`.

use std::collections::HashSet;

// 🔴 Bí danh `HtmlDocument`, KHÔNG `Document` trần — `tests/naming_boundary.rs` cấm
// `Project`/`Book`/`Novel`/`Document` cho khái niệm tầng Tác phẩm (`AGENTS.md:41`); kiểu này
// là của `dom_query` (mô hình DOM một trang HTML, không liên quan `Work`) nhưng cổng quét
// LITERAL không phân biệt được "dùng lại tên của thư viện ngoài" khỏi "đặt tên một thực thể
// mới" — đổi bí danh để tệp này không viết ra chuỗi `Document` đứng ĐẦU một định danh (biên
// TRƯỚC của phép so chỉ đòi một phía, xem `line_names_a_forbidden_entity`), đúng luật "sửa
// nguồn cho nó nói thật, đừng nhét miễn trừ" — không thêm một mục nào vào danh sách miễn trừ
// của cổng đó.
use dom_query::{Document as HtmlDocument, NodeId};
use dom_smoothie::{Config, Readability, TextMode};

/// Mọi cách [`extract`] thất bại — MỘT lý do duy nhất phía người dùng thấy được
/// ("không bóc được nội dung chính", I/O Matrix spec 6.7), `detail` chỉ để chẩn đoán/log.
///
/// 🔴 **Không dùng `Readability::is_probably_readable()` làm điều kiện từ chối.** Bàn đo 6.1
/// (`REPORT.md` §Giới hạn) đã ghi nhận ÍT NHẤT một âm tính giả (mẫu `a04` — một bài viết
/// THẬT bị cờ này chấm sai "không giống bài viết"). Điều kiện đúng là NỘI DUNG THẬT SỰ bóc
/// ra được hay không (`parse()` thành công VÀ `text_content` không rỗng sau khi trim), không
/// phải một cờ suy đoán trước khi bóc.
#[derive(Debug)]
pub struct ExtractError {
    pub detail: String,
}

/// Thân một khối — đúng BA nhánh (spec 6.9 §Code Map): đoạn văn/tiêu đề/mục danh sách/trích
/// dẫn (`Paragraph`), ảnh (`Image`), chú thích ảnh (`Caption`). Không nhánh nào mang HTML hay
/// một chuỗi đánh dấu (AD-16 §2) — luật "0 markup" sống trong CHÍNH KIỂU này, không trong một
/// `if` ở chỗ gọi.
///
/// ⚠️ `Image`/`Caption` CHƯA có chỗ gọi sản phẩm nào tới 6.11 (hiển thị ảnh trong bản dịch)/
/// 6.13 — nợ có chủ, ghi ở `deferred-work.md`. Trường thân đặt tên `body`, không `content`, để
/// không chạm cổng quét literal `.content` của tệp này (KEEP mục ④ vòng rà spec 6.9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockBody {
    /// `h1`-`h6`, `p`, `li`, `blockquote`, `pre`, hoặc một `div` LÁ (không phần tử khối nào
    /// lồng bên trong) mang chữ — văn bản ĐÃ chuẩn hoá khoảng trắng qua `text_content` (xem
    /// doc-comment đầu tệp mục 3/4), không phải một normalize riêng của tệp này.
    Paragraph(String),
    /// `<img>` — thuộc tính bóc RIÊNG, không một byte HTML nào. `machine_kept` của khối này là
    /// một suy đoán (không có "chữ" để mà so khớp với `text_content`) — xem
    /// [`infer_image_kept_state`].
    Image { src: Option<String>, alt: Option<String> },
    /// `<figcaption>` — có CHỮ nên đi qua đúng lượt so khớp như `Paragraph`.
    Caption(String),
}

/// Một khối trong dãy CẢ TRANG, theo đúng thứ tự tài liệu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub body: BlockBody,
    /// Phán đoán CỦA THUẬT TOÁN Readability — `true` = khối này có mặt trong phần được giữ.
    /// KHÔNG phải trạng thái người dùng (đó là việc của tầng gọi,
    /// `commands::project::Tier2BlockOverridesState` — spec 6.9 §Code Map).
    pub machine_kept: bool,
    /// Đoạn khoảng trắng/nối GỐC, byte-for-byte từ `text_content`, đứng NGAY TRƯỚC khối này
    /// — CHỈ có nghĩa khi `machine_kept` VÀ khối `machine_kept` liền trước nó (theo thứ tự
    /// duyệt) chính là khối đã sinh ra đoạn này (xem
    /// `core::segment::pipeline::join_kept_blocks`, nơi điều kiện đó được kiểm). Rỗng cho
    /// khối `machine_kept` ĐẦU TIÊN, cho `Image`, và cho khối bị loại.
    pub exact_gap_before: String,
    /// `true` khi khối đến từ thẻ `<li>` — dùng làm khuôn nối DỰ PHÒNG (một khoảng trắng đơn,
    /// không đôi) khi hai khối liền kề trong bản ĐÃ SỬA không phải một cặp liền kề GỐC trong
    /// `text_content` (một khối đã loại đứng giữa chúng vừa bị người dùng đổi trạng thái).
    pub is_list_item: bool,
}

/// Bóc nội dung chính từ `html` (văn bản ĐÃ GIẢI MÃ — bước 1 AD-39 đứng TRƯỚC bước này
/// trong `PIPELINE_ORDER`, xem `core::segment::pipeline::Step::ExtractMainContent`).
/// `url` dùng để `dom_smoothie` phân giải các đường dẫn TƯƠNG ĐỐI bên trong tài liệu khi
/// tính điểm/cấu trúc — không ảnh hưởng `text_content`, và `Article::content` không bao giờ
/// được đọc từ hàm này (xem doc-comment đầu tệp).
pub fn extract(html: &str, url: &str) -> Result<Vec<Block>, ExtractError> {
    // 🔴 `TextMode::Formatted` — xem doc-comment đầu tệp (Task 1). Mọi trường khác giữ mặc
    // định: `max_elements_to_parse = 0` (không trần — trần byte đã có ở `Fetcher`, một trần
    // SỐ PHẦN TỬ thứ hai ở đây không có phép đo nào chống lưng, đừng đúc thêm một hằng số
    // phù thuỷ).
    let config = Config { text_mode: TextMode::Formatted, ..Config::default() };

    let mut readability = Readability::new(html.to_owned(), Some(url), Some(config))
        .map_err(|e| ExtractError { detail: format!("readability_new: {e}") })?;

    let article = readability
        .parse()
        .map_err(|e| ExtractError { detail: format!("readability_parse: {e}") })?;

    // `Article::content` (HTML) KHÔNG được đọc ở đây, cố ý — xem doc-comment đầu tệp.
    let text_content: String = article.text_content.to_string();

    if text_content.trim().is_empty() {
        return Err(ExtractError { detail: "text_content rong sau khi trim".to_owned() });
    }

    Ok(build_blocks(html, &text_content))
}

/// Tag nào được coi là "khối" — Readability giữ lại chữ ở các phần tử này (spec 6.9, đo
/// 2026-09-07 trên bảy mẫu bàn đo 6.1). `div` xử RIÊNG (chỉ khi LÁ — xem [`build_blocks`]).
const TEXT_BLOCK_TAGS: [&str; 11] =
    ["p", "h1", "h2", "h3", "h4", "h5", "h6", "li", "blockquote", "pre", "figcaption"];

fn is_text_block_tag(name: &str) -> bool {
    TEXT_BLOCK_TAGS.contains(&name)
}

/// Selector CSS gộp — dùng để duyệt CẢ ứng viên khối lẫn để hỏi "phần tử này có khối lồng bên
/// trong không" (kiểm div lá). `img` có mặt để một `div` BỌC ảnh không bị coi là lá (ảnh phải
/// tách thành khối RIÊNG của chính nó, không bị nuốt vào văn bản của `div` cha).
const BLOCK_SELECTOR: &str =
    "p, h1, h2, h3, h4, h5, h6, li, blockquote, pre, figcaption, img, div";

/// Duyệt HTML GỐC (lượt phân tích thứ hai, xem doc-comment đầu tệp) lấy dãy khối CẢ TRANG, rồi
/// đối chiếu với `text_content` để đánh dấu `machine_kept` + chụp khoảng trắng gốc.
fn build_blocks(html: &str, text_content: &str) -> Vec<Block> {
    let document = HtmlDocument::from(html);
    let selection = document.select(BLOCK_SELECTOR);

    // ─────────────────────────────────────────────────────────────────────────
    // Lượt 1 — chọn ứng viên theo thứ tự tài liệu, bỏ khối LỒNG trong một khối đã nhận (không
    // đếm hai lần), và bỏ `div` không phải LÁ (chữ của nó sẽ do các con của nó tự mang).
    // ─────────────────────────────────────────────────────────────────────────
    enum Candidate {
        Text { is_list_item: bool, is_caption: bool, raw_text: String },
        Image { src: Option<String>, alt: Option<String> },
    }

    let mut accepted_ids: HashSet<NodeId> = HashSet::new();
    let mut candidates: Vec<Candidate> = Vec::new();

    for node in selection.nodes() {
        let name = node.node_name().map(|n| n.to_string()).unwrap_or_default();
        let name_str = name.as_str();

        // 🔴 **SỬA 2026-09-07 (vòng rà bước 4) — `img` KHÔNG bị chặn bởi tổ tiên đã nhận.**
        // Bản trước kiểm `has_accepted_ancestor` TRƯỚC khi biết tên thẻ, nên `<p><img></p>`/
        // `<li><img></li>` (hình dạng ảnh PHỔ BIẾN NHẤT trên trang truyện — ảnh minh hoạ nằm
        // NGAY TRONG đoạn văn/mục danh sách của nó) không bao giờ sinh `BlockBody::Image`: cha
        // của `img` (thẻ `p`/`li`) đã được `accepted_ids` nhận trước đó trong CÙNG một lượt
        // duyệt (thứ tự tài liệu: cha luôn đến trước con), nên `img` con bị bỏ qua vĩnh viễn —
        // đúng đầu vào mà FR127 (tải ảnh) cần lại chính là hình dạng bị mất. `img` không có
        // "chữ" nên tách nó ra khỏi cha không đổi `raw_text` của cha (`.text()` không đếm
        // `<img>`) — 0 rủi ro đếm trùng nội dung.
        if name_str == "img" {
            let src = node.attr("src").map(|v| v.to_string());
            let alt = node.attr("alt").map(|v| v.to_string());
            candidates.push(Candidate::Image { src, alt });
            continue;
        }

        let has_accepted_ancestor =
            node.ancestors_it(None).any(|a| accepted_ids.contains(&a.id));
        if has_accepted_ancestor {
            continue;
        }

        if name_str == "div" {
            // Lá == không phần tử nào trong `BLOCK_SELECTOR` lồng bên trong. Một `div` KHÔNG
            // lá bị bỏ qua HOÀN TOÀN (không nhận vào `accepted_ids`) — con của nó (kể cả một
            // `div` lá cháu, hay một `img`) vẫn còn cơ hội được duyệt riêng ở các lượt sau
            // của vòng lặp này.
            let is_leaf = dom_query::Selection::from(*node).select(BLOCK_SELECTOR).nodes().is_empty();
            if !is_leaf {
                // 🔴 **THÊM 2026-09-07 (vòng rà spec 6.9) — đo trên `a07.html` (bàn đo 6.1).**
                // Một `div` KHÔNG lá vẫn có thể mang CHỮ TRỰC TIẾP xen giữa các phần tử con
                // (`<div>頭條 1/12<img></div>` — nhãn số trang cạnh một ảnh) — chữ đó không
                // thuộc về bất kỳ khối con nào (ảnh không có chữ) nên bị RỚT HẲN nếu không bắt
                // ở đây. `immediate_text()` (chữ CỦA RIÊNG node, không đệ quy con) chụp đúng
                // phần đó; không đánh dấu `accepted_ids` cho `node` — con của nó (bao gồm
                // `img`) vẫn được duyệt riêng như bình thường.
                let immediate = node.immediate_text().to_string();
                if !immediate.trim().is_empty() {
                    candidates.push(Candidate::Text {
                        is_list_item: false,
                        is_caption: false,
                        raw_text: immediate,
                    });
                }
                continue;
            }
        } else if !is_text_block_tag(name_str) {
            continue;
        }

        let raw_text = node.text().to_string();
        // Sau `char::is_whitespace` gạn hết ⇒ phần tử rỗng (`<p></p>`, một `div` lá chỉ chứa
        // khoảng trắng) — không có gì để mà hiển thị/gạt trạng thái, bỏ qua hẳn thay vì tạo
        // một khối rỗng vô nghĩa.
        if raw_text.trim().is_empty() {
            continue;
        }

        accepted_ids.insert(node.id);
        let is_list_item = name_str == "li";
        // 🔴 **SỬA 2026-09-07 (vòng rà bước 4)** — `figcaption` phải sinh `BlockBody::Caption`,
        // không `Paragraph`. Bản trước gộp mọi thẻ chữ vào MỘT nhánh dây (`Candidate::Text`
        // không phân biệt) rồi đúc thẳng thành `BlockBody::Paragraph` ở lượt 3 — hàng I/O
        // Matrix ĐÃ ĐÓNG BĂNG của spec ("Ảnh và caption... Mô hình chở đúng nhánh
        // `Image`/`Caption`") vì thế chưa từng đúng cho riêng nhánh `Caption`.
        let is_caption = name_str == "figcaption";
        candidates.push(Candidate::Text { is_list_item, is_caption, raw_text });
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Lượt 2 — so khớp chính xác (sau chuẩn hoá khoảng trắng) các khối CÓ CHỮ với
    // `text_content`. Đúng thứ tự khai ở đầu tệp mục 3/4 — NHƯNG "con trỏ chỉ tiến" một mình
    // KHÔNG đủ, xem [`assign_matches_by_max_weight_order_preserving_dp`].
    // ─────────────────────────────────────────────────────────────────────────
    let (norm_tc, positions) = normalize_with_positions(text_content);

    // `norm_blocks[k]` song song với `candidates` — `None` cho `Image` VÀ cho một khối chữ có
    // thân RỖNG sau chuẩn hoá (bỏ qua khỏi mô hình hoàn toàn, giữ nguyên hành vi cũ).
    let norm_blocks: Vec<Option<String>> = candidates
        .iter()
        .map(|c| match c {
            Candidate::Image { .. } => None,
            Candidate::Text { raw_text, .. } => {
                let n = normalize_ws(raw_text);
                if n.is_empty() { None } else { Some(n) }
            }
        })
        .collect();

    let assignment = assign_matches_by_max_weight_order_preserving_dp(&norm_tc, &norm_blocks);

    let mut blocks: Vec<Block> = Vec::with_capacity(candidates.len());
    let mut last_kept_end: usize = 0;
    for (i, candidate) in candidates.into_iter().enumerate() {
        match candidate {
            Candidate::Image { src, alt } => {
                blocks.push(Block {
                    body: BlockBody::Image { src, alt },
                    // Điền tạm `false` — [`infer_image_kept_state`] sửa lại ở lượt 3.
                    machine_kept: false,
                    exact_gap_before: String::new(),
                    is_list_item: false,
                });
            }
            Candidate::Text { is_list_item, is_caption, .. } => {
                let Some(norm_block) = &norm_blocks[i] else { continue };
                let make_body = |text: String| -> BlockBody {
                    if is_caption { BlockBody::Caption(text) } else { BlockBody::Paragraph(text) }
                };
                match assignment.get(&i) {
                    Some(&(abs_start, abs_end)) => {
                        let char_start = norm_tc[..abs_start].chars().count();
                        let char_end = norm_tc[..abs_end].chars().count();
                        let orig_start = positions[char_start];
                        let orig_end = positions[char_end];
                        let gap = text_content[last_kept_end..orig_start].to_owned();
                        let body_text = text_content[orig_start..orig_end].to_owned();
                        last_kept_end = orig_end;
                        blocks.push(Block {
                            body: make_body(body_text),
                            machine_kept: true,
                            exact_gap_before: gap,
                            is_list_item,
                        });
                    }
                    None => {
                        blocks.push(Block {
                            body: make_body(norm_block.clone()),
                            machine_kept: false,
                            exact_gap_before: String::new(),
                            is_list_item,
                        });
                    }
                }
            }
        }
    }

    infer_image_kept_state(&mut blocks);

    // ─────────────────────────────────────────────────────────────────────────
    // Lưới an toàn — "0 khối ĐANG GIỮ dù `text_content` không rỗng".
    // ─────────────────────────────────────────────────────────────────────────
    // 🔴 **SỬA 2026-09-07 (vòng rà bước 4) — điều kiện bản trước MÙ với ca NGUY HIỂM HƠN.**
    // Bản trước hỏi "có TỒN TẠI một khối Paragraph/Caption nào không" — câu đó đúng ngay cả
    // khi MỌI khối tồn tại đều `machine_kept == false` (assignment của DP trả RỖNG — ví dụ
    // toàn bộ chuẩn hoá khoảng trắng của các ứng viên không khớp `text_content` ở đâu cả, một
    // lỗi ở lớp ĐO chứ không phải "trang không có khối"). Ca đó lọt qua lưới cũ, rồi
    // `join_kept_blocks` (chỉ đọc `machine_kept == true`) trả `""`, và Chương được ghi RỖNG
    // TRONG IM LẶNG — đúng lớp lỗi AC của story này tuyên bố đóng. Điều kiện đúng phải hỏi có
    // khối ĐANG GIỮ hay không, không phải có khối TỒN TẠI hay không.
    if !blocks.iter().any(|b| b.machine_kept && matches!(b.body, BlockBody::Paragraph(_) | BlockBody::Caption(_)))
    {
        blocks.push(Block {
            body: BlockBody::Paragraph(text_content.to_owned()),
            machine_kept: true,
            exact_gap_before: String::new(),
            is_list_item: false,
        });
    }

    blocks
}

/// `Image` không có "chữ" để so khớp với `text_content` — suy `machine_kept` từ khối MANG CHỮ
/// gần nhất: trước hết nhìn LÙI (ảnh thường đứng ngay trong/cạnh đoạn chứa nó), rỗng thì nhìn
/// TỚI; không còn khối mang chữ nào cả (trang toàn ảnh) ⇒ mặc định `true` (giữ) — thà hiện dư
/// một ảnh còn hơn giấu nó khỏi màn sửa tay mà không ai biết nó từng tồn tại.
fn infer_image_kept_state(blocks: &mut [Block]) {
    let text_kept: Vec<Option<bool>> = blocks
        .iter()
        .map(|b| match b.body {
            BlockBody::Paragraph(_) | BlockBody::Caption(_) => Some(b.machine_kept),
            BlockBody::Image { .. } => None,
        })
        .collect();

    for i in 0..blocks.len() {
        if !matches!(blocks[i].body, BlockBody::Image { .. }) {
            continue;
        }
        let backward = text_kept[..i].iter().rev().find_map(|v| *v);
        let forward = text_kept[i + 1..].iter().find_map(|v| *v);
        blocks[i].machine_kept = backward.or(forward).unwrap_or(true);
    }
}

/// Chuẩn hoá khoảng trắng — MỘT khoảng trắng giữa các từ, cắt hai đầu. Cùng ngữ nghĩa
/// `char::is_whitespace` mà `dom_query::node::text_formatting::push_normalized_text` dùng cho
/// `text_content` (Task 1/[`TextMode::Formatted`]) — hai bên chuẩn hoá theo CÙNG một vị từ, đó
/// là điều kiện để phép so khớp ở [`build_blocks`] không lệch vì hai luật khoảng trắng khác
/// nhau.
fn normalize_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending_space = false;
    let mut started = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if started {
                pending_space = true;
            }
        } else {
            if pending_space {
                out.push(' ');
                pending_space = false;
            }
            out.push(c);
            started = true;
        }
    }
    out
}

/// Như [`normalize_ws`], nhưng cộng một bảng ánh xạ NGƯỢC: `positions[k]` là toạ độ BYTE trong
/// `s` GỐC của ký tự thứ `k` (theo CHỈ SỐ KÝ TỰ, không phải byte) của chuỗi đã chuẩn hoá —
/// cộng một phần tử lính canh cuối (`s.len()`) để lấy được ranh giới PHẢI của ký tự cuối cùng.
/// Đây là thứ cho phép [`build_blocks`] cắt lại nguyên vẹn một dải BYTE-FOR-BYTE của `s` ứng
/// với một dải KÝ TỰ đã khớp trên bản chuẩn hoá — không đoán, đọc thẳng từ bảng.
fn normalize_with_positions(s: &str) -> (String, Vec<usize>) {
    let mut out = String::with_capacity(s.len());
    let mut positions: Vec<usize> = Vec::with_capacity(s.len());
    let mut pending_space_at: Option<usize> = None;
    let mut started = false;
    for (byte_idx, c) in s.char_indices() {
        if c.is_whitespace() {
            if started && pending_space_at.is_none() {
                pending_space_at = Some(byte_idx);
            }
        } else {
            if let Some(space_at) = pending_space_at.take() {
                out.push(' ');
                positions.push(space_at);
            }
            out.push(c);
            positions.push(byte_idx);
            started = true;
        }
    }
    positions.push(s.len());
    (out, positions)
}

/// Mọi vị trí (không chồng nhau) mà `needle` xuất hiện NGUYÊN VĂN trong `haystack`, quét từ
/// trái sang phải — `needle` rỗng cho `vec![]` (không có gì gọi là "khớp" một chuỗi rỗng ở
/// đây, khác ngữ nghĩa `str::find` vốn khớp rỗng ở MỌI vị trí).
fn find_all_occurrences(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    if needle.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while cursor <= haystack.len() {
        let Some(rest) = haystack.get(cursor..) else { break };
        let Some(rel) = rest.find(needle) else { break };
        let start = cursor + rel;
        let end = start + needle.len();
        out.push((start, end));
        cursor = end; // không chồng nhau — nhảy qua hết chỗ vừa khớp.
    }
    out
}

/// Một trạng thái đã đạt được trong DP — "đã gán xong `weight` ký tự, dải cuối cùng dùng kết
/// thúc ở byte `end` của `norm_tc`". `prev`/`block_idx`/`occ` cho phép LẦN NGƯỢC ra đúng tập
/// khối đã chọn — xem [`assign_matches_by_max_weight_order_preserving_dp`].
struct DpNode {
    end: usize,
    weight: usize,
    block_idx: usize,
    occ: (usize, usize),
    prev: Option<usize>,
}

/// Gán mỗi khối-có-chữ (chỉ số trong `norm_blocks`, `None` = ảnh hoặc thân rỗng, bị bỏ qua)
/// vào NHIỀU NHẤT một dải khớp CHÍNH XÁC trong `norm_tc`, sao cho:
/// - mỗi khối được gán ĐÚNG MỘT trong các vị trí nó khớp CHÍNH XÁC (hoặc không gán, "loại"),
/// - các dải đã gán KHÔNG chồng nhau, và thứ tự của chúng trong `norm_tc` khớp đúng thứ tự
///   chỉ số khối (Readability không đảo thứ tự phần tử — bất biến CƠ CHẾ, không phải một giả
///   định lỏng lẻo),
///
/// trong khi TỐI ĐA HOÁ tổng độ dài (ký tự đã chuẩn hoá) của các khối được gán. Đây là DP
/// "dãy con tăng có trọng số" tiêu chuẩn (mỗi khối là một "công việc" có thể có NHIỀU khoảng
/// thời gian khả dĩ) — 0 hằng ngưỡng, 0 phỏng đoán: một khối ngắn (dễ trùng ngẫu nhiên ở một
/// vị trí xa) chỉ thắng khi phần thưởng của nó (độ dài) VƯỢT tổng phần thưởng của MỌI khối dài
/// hơn mà việc chọn vị trí xa đó sẽ chặn mất — chính xác điều mà một phép "khớp đầu tiên tìm
/// thấy" (bản trước, xem doc-comment đầu tệp mục 3) không cân nhắc.
///
/// Trả về map chỉ số khối → dải đã gán (byte offset trong `norm_tc`).
fn assign_matches_by_max_weight_order_preserving_dp(
    norm_tc: &str,
    norm_blocks: &[Option<String>],
) -> std::collections::HashMap<usize, (usize, usize)> {
    let mut nodes: Vec<DpNode> = Vec::new();

    // `best_prior(s, limit)` — trọng số LỚN NHẤT đạt được bởi một trạng thái CÓ TRƯỚC chỉ số
    // `limit` trong `nodes` (kể cả trạng thái RỖNG ban đầu, trọng số 0) mà `end <= s`. Trả kèm
    // chỉ số nút để LẦN NGƯỢC, `None` nghĩa là trạng thái rỗng ban đầu.
    let best_prior = |nodes: &[DpNode], s: usize, limit: usize| -> (usize, Option<usize>) {
        let mut best_weight = 0usize;
        let mut best_idx: Option<usize> = None;
        for (idx, node) in nodes[..limit].iter().enumerate() {
            if node.end <= s && node.weight > best_weight {
                best_weight = node.weight;
                best_idx = Some(idx);
            }
        }
        (best_weight, best_idx)
    };

    // 🔴 **`limit` CHỐT TRƯỚC VÒNG LẶP CỦA TỪNG KHỐI, không đọc `nodes.len()` TRONG vòng lặp
    // occurrence.** Nếu không, occurrence THỨ HAI của CHÍNH khối `block_idx` sẽ thấy được nút
    // của occurrence THỨ NHẤT (hai occurrence của cùng một khối luôn thoả `end_1 <= start_2`
    // theo cách [`find_all_occurrences`] quét không chồng) và "nối" vào nó — đếm trùng ĐỘ DÀI
    // của CHÍNH khối đó hai lần trong tổng trọng số, một lỗi tính sai kín đáo. Chốt `limit`
    // trước vòng `for occ` biến việc này thành BẤT KHẢ THEO CẤU TRÚC: mọi nút mà một occurrence
    // của khối `block_idx` có thể nối vào đều thuộc về một khối có chỉ số NHỎ HƠN thật sự
    // (`nodes` chỉ được `push` sau khi khối trước đã xử lý xong toàn bộ occurrence của nó).
    for (block_idx, maybe_norm) in norm_blocks.iter().enumerate() {
        let Some(norm_block) = maybe_norm else { continue };
        let limit = nodes.len();
        for occ in find_all_occurrences(norm_tc, norm_block) {
            let (start, end) = occ;
            let (prior_weight, prior_idx) = best_prior(&nodes, start, limit);
            nodes.push(DpNode {
                end,
                // 🔴 **SỬA 2026-09-07 (vòng rà bước 4) — `.chars().count()`, KHÔNG `.len()`.**
                // Doc-comment hàm này khai trọng số là "tổng độ dài (KÝ TỰ đã chuẩn hoá)", còn
                // `.len()` trả số BYTE UTF-8 — một ký tự Hán/Việt có dấu chiếm 3 byte, một ký
                // tự Latin chiếm 1. Trên một trang trộn Hán-Latin, dùng byte làm trọng số lệch
                // tới ~3× khỏi điều doc-comment tuyên bố, và có thể làm DP chọn giữ một khối
                // Latin NGẮN (ví dụ một dòng bản quyền tiếng Anh) thay vì một khối Hán NGẮN HƠN
                // VỀ BYTE nhưng dài hơn hoặc bằng về Ý (số ký tự) — sai đúng thứ mà "tối đa hoá
                // độ dài" phải bảo toàn.
                weight: prior_weight + norm_block.chars().count(),
                block_idx,
                occ,
                prev: prior_idx,
            });
        }
    }

    // Nút cuối chuỗi tối ưu là nút mang trọng số LỚN NHẤT trong TOÀN BỘ `nodes` — KHÔNG nhất
    // thiết là nút được tạo SAU CÙNG (một khối sớm hơn với trọng số cao hơn có thể đứng cuối
    // chuỗi tối ưu nếu các khối sau nó không khớp được gì).
    let Some((best_idx, _)) = nodes.iter().enumerate().max_by_key(|(_, n)| n.weight) else {
        return std::collections::HashMap::new();
    };

    let mut result = std::collections::HashMap::new();
    let mut cursor = Some(best_idx);
    while let Some(idx) = cursor {
        let node = &nodes[idx];
        result.insert(node.block_idx, node.occ);
        cursor = node.prev;
    }
    result
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.9 — THÊM 2026-09-07 (vòng rà bước 4, mục 15/16/20/21/22). Trước bản vá này, các
// hàm THUẦN dưới đây (`assign_matches_by_max_weight_order_preserving_dp`,
// `infer_image_kept_state`, `normalize_ws`, luật lá của `div`, cặp
// `TEXT_BLOCK_TAGS`/`BLOCK_SELECTOR`) chỉ được chạm GIÁN TIẾP qua bảy mẫu bàn đo 6.1 trong
// `webimport_contract.rs` — đủ để bắt một hồi quy TRÊN BẢY MẪU ĐÓ, không đủ để khoá riêng
// TỪNG quy tắc bằng một fixture tối giản, tự giải thích.
// ═════════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dp_assignment_prefers_the_long_genuine_block_over_a_short_blocks_coincidental_match_inside_it()
     {
        // Mô phỏng ĐÚNG hình dạng lỗi `a03.html` (xem doc-comment đầu tệp): một khối NGẮN (ví
        // dụ một mục điều hướng, bản thân nó KHÔNG thực sự có mặt trong `text_content`) trùng
        // NGẪU NHIÊN với một từ THẬT nằm SAU nó, bên trong một khối DÀI (nội dung THẬT). Khớp
        // đầu tiên tìm thấy (thuật toán CŨ, "con trỏ chỉ tiến") sẽ gán khối ngắn vào đúng chỗ
        // trùng ngẫu nhiên đó, đẩy con trỏ vượt qua toàn bộ khối dài — khối dài bị gán nhầm
        // `machine_kept = false`. DP phải tối đa hoá TỔNG độ dài: bỏ khối ngắn, giữ khối dài.
        let long = "mot doan van that su dai de dam bao trong so cua no vuot han mot muc \
                     dieu huong ngan di truoc no trong thu tu tai lieu";
        let short = "da"; // xuất hiện BÊN TRONG `long` ("dai"/"dam"), không ở đầu `long`.
        assert!(long.contains(short), "fixture phai chua occurrence cua khoi ngan BEN TRONG khoi dai");
        assert!(!long.starts_with(short), "khoi ngan khong duoc trung voi DAU cua khoi dai");

        let norm_tc = long.to_owned();
        let norm_blocks = vec![Some(short.to_owned()), Some(long.to_owned())];
        let assignment = assign_matches_by_max_weight_order_preserving_dp(&norm_tc, &norm_blocks);

        assert_eq!(
            assignment.get(&1),
            Some(&(0usize, long.len())),
            "khoi DAI (chi so 1) phai duoc gan dung TOAN BO pham vi cua no"
        );
        assert!(
            !assignment.contains_key(&0),
            "khoi NGAN (chi so 0) khong duoc gianh mat cho trung ngau nhien -- neu no gianh, \
             thuat toan da tai pham dung loi a03 ma doc-comment dau tep ghi lai"
        );
    }

    #[test]
    fn dp_assignment_maximizes_character_count_not_byte_count_on_a_han_latin_conflict() {
        // 🔴 THÊM 2026-09-07 (vòng rà bước 4, mục 3/16) — khoá lại CHÍNH điều mục 3 sửa: trọng
        // số DP phải là SỐ KÝ TỰ đã chuẩn hoá, không phải SỐ BYTE UTF-8. Dựng một cuộc xung đột
        // THẬT giữa hai khối không thể cùng tồn tại (thứ tự tài liệu buộc chọn MỘT): một khối
        // Hán 4 KÝ TỰ (12 BYTE, do mỗi ký tự Hán chiếm 3 byte UTF-8) đứng ở chỉ số 0 (buộc phải
        // đứng TRƯỚC trong `norm_tc` nếu muốn cùng tồn tại với khối 1) nhưng vị trí THẬT của nó
        // lại nằm SAU một khối Latin 10 KÝ TỰ (cũng 10 BYTE, ASCII 1-byte/ký-tự) — hai khối vì
        // vậy loại trừ nhau. Đo bằng BYTE sẽ chọn nhầm khối Hán (12 byte > 10 byte) dù nó NGẮN
        // HƠN về Ý (4 ký tự < 10 ký tự); đo bằng KÝ TỰ (đúng doc-comment hàm này) chọn đúng
        // khối Latin dài hơn.
        let latin = "abcdefghij"; // 10 ky tu, 10 byte (ASCII).
        let han = "萧炎哮天"; // 4 ky tu, 12 byte (UTF-8, 3 byte/ky tu).
        assert_eq!(latin.chars().count(), 10);
        assert_eq!(han.chars().count(), 4);
        assert!(han.len() > latin.len(), "tien de: khoi Han phai NANG HON ve BYTE");
        assert!(latin.chars().count() > han.chars().count(), "tien de: khoi Latin phai NANG HON ve KY TU");

        let norm_tc = format!("{latin} {han}");
        // Candidate 0 = khoi Han (dung o CHI SO tai lieu DAU, nhung vi tri THAT cua no trong
        // `norm_tc` lai o SAU khoi Latin) -- ep xung dot thu tu voi candidate 1.
        let norm_blocks = vec![Some(han.to_owned()), Some(latin.to_owned())];
        let assignment = assign_matches_by_max_weight_order_preserving_dp(&norm_tc, &norm_blocks);

        assert!(
            assignment.contains_key(&1) && !assignment.contains_key(&0),
            "DP phai chon khoi Latin (chi so 1, 10 ky tu) thay vi khoi Han (chi so 0, chi 4              ky tu nhung 12 byte) -- neu ca nay do, trong so dang do BYTE chu khong phai KY TU"
        );
    }

    #[test]
    fn dp_assignment_picks_the_heavier_block_when_two_candidates_only_match_overlapping_spans() {
        let norm_tc = "hello world hello universe";
        let block_a = "hello world"; // 0..11
        let block_b = "world hello universe"; // 6..26 -- CHONG LAN voi block_a o "world".

        let start_b = norm_tc.find(block_b).expect("fixture phai chua block_b");
        assert!(start_b < block_a.len(), "hai khoi phai THAT SU chong nhau de ca nay co y nghia");

        let norm_blocks = vec![Some(block_a.to_owned()), Some(block_b.to_owned())];
        let assignment = assign_matches_by_max_weight_order_preserving_dp(&norm_tc, &norm_blocks);

        assert!(
            !assignment.contains_key(&0),
            "hai khoi chong nhau khong the cung duoc gan -- khoi NANG HON (block_b, {} ky tu) \
             phai thang khoi NHE HON (block_a, {} ky tu)",
            block_b.chars().count(),
            block_a.chars().count()
        );
        assert_eq!(assignment.get(&1), Some(&(start_b, start_b + block_b.len())));
    }

    #[test]
    fn infer_image_kept_state_matches_kept_state_of_nearest_text_neighbor() {
        fn text_block(kept: bool) -> Block {
            Block {
                body: BlockBody::Paragraph("x".to_owned()),
                machine_kept: kept,
                exact_gap_before: String::new(),
                is_list_item: false,
            }
        }
        fn image_block() -> Block {
            Block {
                body: BlockBody::Image { src: None, alt: None },
                // Giá trị TẠM trước khi `infer_image_kept_state` chạy -- cùng khuôn `build_blocks`.
                machine_kept: false,
                exact_gap_before: String::new(),
                is_list_item: false,
            }
        }

        // Ảnh THÂN BÀI — đứng giữa hai đoạn máy đã GIỮ ⇒ suy ra GIỮ.
        let mut body_case = vec![text_block(true), image_block(), text_block(true)];
        infer_image_kept_state(&mut body_case);
        assert!(body_case[1].machine_kept, "anh giua hai doan GIU phai duoc suy la GIU");

        // Ảnh khu vực ĐIỀU HƯỚNG — đứng giữa hai đoạn máy đã LOẠI ⇒ suy ra LOẠI (khớp trạng
        // thái của khối chữ LÂN CẬN, không phải "có lân cận là mặc định giữ").
        let mut nav_case = vec![text_block(false), image_block(), text_block(false)];
        infer_image_kept_state(&mut nav_case);
        assert!(!nav_case[1].machine_kept, "anh giua hai doan bi LOAI phai duoc suy la LOAI");

        // Không còn khối chữ nào cả (trang toàn ảnh) ⇒ mặc định GIỮ (thà hiện dư).
        let mut all_image = vec![image_block()];
        infer_image_kept_state(&mut all_image);
        assert!(all_image[0].machine_kept, "khong con khoi chu nao de doi chieu -- mac dinh GIU");
    }

    #[test]
    fn normalize_ws_treats_a_non_breaking_space_as_whitespace_like_char_is_whitespace_does() {
        // U+00A0 (NBSP) — trang web thường mang nó giữa số/đơn vị, hoặc sau khi `&nbsp;` được
        // giải mã thực thể HTML. `char::is_whitespace()` (dùng ở CẢ HAI hàm chuẩn hoá của tệp
        // này lẫn `dom_query::node::text_formatting`) coi NBSP là khoảng trắng — doc-comment
        // của `normalize_ws` khai đúng điều đó, ca này khoá lại bằng đo THẬT thay vì đọc lời
        // khai suông.
        assert_eq!(
            normalize_ws("hello\u{00A0}world"),
            "hello world",
            "NBSP giua hai tu phai gop thanh MOT dau cach, giong char::is_whitespace"
        );
        assert_eq!(
            normalize_ws("\u{00A0}\u{00A0}leading"),
            "leading",
            "NBSP o dau phai bi cat, giong khoang trang thuong"
        );
        assert_eq!(
            normalize_ws("trailing\u{00A0}\u{00A0}"),
            "trailing",
            "NBSP o cuoi phai bi cat, giong khoang trang thuong"
        );
    }

    #[test]
    fn a_leaf_div_becomes_one_block_but_a_div_wrapping_a_block_element_does_not_duplicate_it() {
        // Lá — `div` KHÔNG lồng phần tử nào khớp `BLOCK_SELECTOR` bên trong ⇒ CHÍNH nó sinh
        // đúng một khối.
        let leaf_text = "Mot dong chu la con cua mot the div khong long gi ca ben trong";
        let leaf_html = format!("<div>{leaf_text}</div>");
        let leaf_blocks = build_blocks(&leaf_html, leaf_text);
        assert_eq!(leaf_blocks.len(), 1, "mot div LA phai sinh dung MOT khoi");
        assert!(matches!(&leaf_blocks[0].body, BlockBody::Paragraph(t) if t == leaf_text));
        assert!(leaf_blocks[0].machine_kept);

        // KHÔNG lá — `div` bọc một `<p>` ⇒ chỉ `<p>` con sinh khối, `div` cha KHÔNG tự sinh
        // một khối RIÊNG cho toàn bộ chữ của nó (tránh đếm hai lần CÙNG một đoạn văn).
        let wrapped_text = "Doan van con nam trong the p, cha la mot div KHONG la";
        let wrapped_html = format!("<div><p>{wrapped_text}</p></div>");
        let wrapped_blocks = build_blocks(&wrapped_html, wrapped_text);
        assert_eq!(
            wrapped_blocks.len(),
            1,
            "div KHONG la (co <p> long ben trong) khong duoc tu sinh mot khoi RIENG cho chinh \
             no -- chi <p> con moi sinh khoi"
        );
        assert!(matches!(&wrapped_blocks[0].body, BlockBody::Paragraph(t) if t == wrapped_text));
    }

    #[test]
    fn block_selector_contains_every_text_block_tag_plus_exactly_two_extra_members() {
        // `TEXT_BLOCK_TAGS` (dùng để lọc "thẻ khối mang chữ") và `BLOCK_SELECTOR` (dùng để
        // DUYỆT DOM, cộng kiểm "div có lá không") là HAI nguồn sự thật riêng cho cùng một họ
        // thẻ — mục 22 đòi một phép kiểm chéo thay vì tin hai hằng số này không bao giờ trôi
        // khỏi nhau qua các lượt sửa sau.
        let selector_tags: std::collections::HashSet<&str> = BLOCK_SELECTOR.split(", ").collect();
        for tag in TEXT_BLOCK_TAGS {
            assert!(
                selector_tags.contains(tag),
                "BLOCK_SELECTOR thieu tag '{tag}' dang co trong TEXT_BLOCK_TAGS"
            );
        }
        let text_tag_set: std::collections::HashSet<&str> = TEXT_BLOCK_TAGS.into_iter().collect();
        let extra: std::collections::HashSet<&str> =
            selector_tags.difference(&text_tag_set).copied().collect();
        assert_eq!(
            extra,
            std::collections::HashSet::from(["img", "div"]),
            "BLOCK_SELECTOR chi duoc mang THEM dung hai tag 'img'/'div' ngoai TEXT_BLOCK_TAGS -- \
             mot tag la them ngoai y muon se khong duoc is_text_block_tag() nhan dien dung"
        );
    }
}
