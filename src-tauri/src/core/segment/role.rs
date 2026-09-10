//! Trường **vai** của segment — Story 6.13, FR129, AD-42.
//!
//! AD-42 nói caption và alt-text là `Segment` mang trường **vai**, không một cột text riêng
//! trên `asset` — điều đó là cần thiết để cả hai tự động vào Translation Memory/Glossary
//! (Epic 7) qua ĐÚNG luồng mà mọi segment khác đi qua (`confirm_segment`, `segment_version`),
//! không một luồng thứ hai. Module này là nơi DUY NHẤT đúc ra hai chuỗi `'alt'`/`'caption'`
//! ([`SegmentRole::as_str`]) và nơi hai cơ chế của Quyết định 1/2 (spec 6.13) sống:
//!
//! 1. [`force_caption_segment_boundaries`] — chạy ở [`super::pipeline::Step::SplitSegments`]
//!    (bước 7 AD-39), TRƯỚC khi neo được tính: ép ranh giới segment tại hai đầu khối `Caption`
//!    ĐƯỢC CHỌN của mỗi ảnh sở hữu ([`chosen_caption_indices`]), để nó cho **đúng một** segment
//!    (Quyết định 1). Khối `Caption` KHÔNG có ảnh đứng trước (I/O Matrix "Caption không có ảnh
//!    trước nó") đi tiếp như văn xuôi — hàm CHỈ ép ranh giới cho khối SẼ được gán vai, không ép
//!    cho mọi `Caption`, vì bản thân yêu cầu "nhiều nhất một segment mỗi vai" của AD-42 chỉ có
//!    nghĩa cho một khối THẬT SỰ có vai.
//! 2. [`weave_chapter_segments`] — chạy ở `commands::project::create_work`, SAU khi
//!    `core::segment::anchor::compute_anchor` đã tính neo trên dãy segment văn xuôi (đã qua
//!    (1)): dệt một segment `role: Alt` mới ngay sau neo của mỗi ảnh GIỮ có `alt` khác rỗng
//!    (Quyết định 2), gắn `role: Caption` vào đúng segment (đã là MỘT segment nhờ (1)) của
//!    khối `Caption` ĐƯỢC CHỌN của mỗi ảnh sở hữu, và trả về neo **đã dời** theo số segment vai
//!    chèn trước nó — đúng khuôn dời neo mà gộp/tách Chương và gộp/tách câu đã làm
//!    (`schema.rs:958-975`).
//!
//! Quan hệ caption ⇄ ảnh suy theo **VỊ TRÍ**, không theo khoảng cách byte: ảnh GIỮ **gần nhất
//! đứng trước** một khối `Caption` (theo thứ tự khối trong tài liệu) là ảnh sở hữu nó — mô
//! hình khối của `Extractor` PHẲNG, không nhóm `<figure>` (`extractor.rs`), nên đây là cách
//! DUY NHẤT suy quan hệ đó mà không cần phân tích lại HTML gốc (`.atproj` không giữ nó, AD-4).
//!
//! 🔴 **"NHIỀU NHẤT MỘT segment mỗi vai" (AD-42) áp CHO CẢ khối `Caption` THỨ HAI trở đi của
//! CÙNG một ảnh** — sửa 2026-09-10, bắt được ở vòng rà: mô hình khối PHẲNG không cấm một ảnh
//! có nhiều hơn một `<figcaption>` liền kề, và bản trước gán vai cho MỌI khối như vậy (đo
//! được: hai `<figcaption>` liên tiếp của cùng một ảnh GIỮ cho **hai** hàng `role='caption'`).
//! [`chosen_caption_indices`] là nơi DUY NHẤT chọn — chỉ khối `Caption` GIỮ ĐẦU TIÊN sau mỗi
//! ảnh sở hữu được chọn; khối thứ hai trở đi đi tiếp NHƯ VĂN XUÔI (không ép ranh giới, không
//! vai). Cả hai hàm (1)/(2) gọi CHUNG hàm chọn này — hai nơi đọc override phải thấy CÙNG một
//! tập, nếu không ép ranh giới và gán vai sẽ rẽ khỏi nhau.
//!
//! ⚠️ **Vai KHÔNG thuộc về [`super::split::SplitSegment`]** — bộ tách là hàm thuần cấp câu,
//! không biết khối nào là ảnh/caption (đó là dữ kiện của TẦNG NHẬP). Nhét `role` vào đó sẽ cho
//! một trường mà bộ tách không bao giờ đặt, một chỗ hở để người sau tưởng bộ tách có trách
//! nhiệm điền nó.

use std::collections::{BTreeMap, HashSet};

use crate::core::cleanup::CleanupRule;
use crate::core::webimport::{Block, BlockBody};

use super::anchor;
use super::split::{SplitSegment, split_source_text};

/// Vai của một segment — `'alt'` | `'caption'` trên đĩa (`segment.role`, `NULL` khi không
/// vai). **Kiểu Rust đóng, DUY NHẤT đúc ra hai chuỗi này** — cùng khuôn `status`, `is_omitted`,
/// `translation_origin` (`schema.rs:1516-1517`): giá trị hợp lệ cưỡng chế ở tầng Rust, không
/// trong một `CHECK` của DDL (`SEGMENT_ROLE_DDL`, `schema.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentRole {
    Alt,
    Caption,
}

impl SegmentRole {
    /// Chuỗi `'alt'` ghi xuống cột `segment.role` — hằng, không literal rải rác, để một ca hợp
    /// đồng khoá cặp chuỗi ⇄ kiểu (khuôn `the_backfill_literal_matches_the_origin_constant_it_copies`
    /// đã dùng cho `translation_origin`).
    pub const ALT_STR: &'static str = "alt";
    /// Chuỗi `'caption'` ghi xuống cột `segment.role`.
    pub const CAPTION_STR: &'static str = "caption";

    pub fn as_str(self) -> &'static str {
        match self {
            SegmentRole::Alt => Self::ALT_STR,
            SegmentRole::Caption => Self::CAPTION_STR,
        }
    }
}

/// Một segment sẵn sàng ghi xuống bảng `segment` — văn xuôi (`role: None`) hoặc segment vai
/// (`role: Some(..)`), đã dệt hoặc chưa cần dệt gì.
///
/// ⚠️ Đây là kiểu mà [`crate::commands::segment::insert_segments`] nhận, KHÔNG phải
/// [`SplitSegment`] — hai kiểu tách rời đúng ranh giới "vai không thuộc về bộ tách" đã ghi ở
/// doc-comment đầu module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WovenSegment {
    pub text: String,
    pub is_paragraph_end: bool,
    pub role: Option<SegmentRole>,
}

impl From<SplitSegment> for WovenSegment {
    /// Đường KHÔNG có `blocks` (`.txt`, dán tay, `.docx`) đi qua đây — mọi hàng `role = NULL`,
    /// đúng I/O Matrix "Chương 0 ảnh".
    fn from(s: SplitSegment) -> Self {
        WovenSegment {
            text: s.text,
            is_paragraph_end: s.is_paragraph_end,
            role: None,
        }
    }
}

/// Kết quả một lượt dệt (2) — xem doc-comment đầu module.
#[derive(Debug, Clone, Default)]
pub struct WovenChapter {
    /// Dãy segment CUỐI CÙNG, đúng thứ tự `ord` sẽ ghi xuống (1-based khi
    /// [`crate::commands::segment::insert_segments`] đánh số).
    pub segments: Vec<WovenSegment>,
    /// Neo `anchor_after_segment_ord` ĐÃ DỜI cho từng ảnh GIỮ, theo CHỈ SỐ KHỐI (`usize`,
    /// index trong `blocks`) — `commands::project::create_work` tra bảng này để cập nhật
    /// `SavedAsset::anchor_after_segment_ord` (đã tính trên dãy TRƯỚC khi dệt) trước khi ghi
    /// `INSERT INTO asset`. Chỉ ảnh mà [`compute_anchor`] tính neo THÀNH CÔNG mới có mặt ở
    /// đây — cùng khuôn "một ảnh trượt không dừng cả lượt nhập" mà
    /// `commands::project::prepare_chapter_images` đã theo.
    pub shifted_anchor_by_block: BTreeMap<usize, i64>,
}

/// `effective_kept[idx]`, rơi về `block.machine_kept` khi `effective_kept` ngắn hơn `blocks`
/// (cùng khuôn [`super::pipeline::effective_kept_for_blocks`] — hàm CHIA SẺ ý nghĩa, không
/// một bản chép độc lập của quy tắc "override thắng, ngắn hơn thì rơi về `machine_kept`").
fn is_kept(blocks: &[Block], effective_kept: &[bool], idx: usize) -> bool {
    effective_kept
        .get(idx)
        .copied()
        .unwrap_or_else(|| blocks[idx].machine_kept)
}

/// `true` khi phần khoảng trắng DẪN ĐẦU của `s` chứa một ký tự xuống dòng — cùng vị từ
/// `skip_gap`/`mark_paragraph_end` của [`super::split`] dùng để quyết cờ kết đoạn, áp cho một
/// KHE đã biết trước (không cần quét cả khe, dừng ngay ký tự không-trắng đầu tiên).
fn leading_gap_has_linebreak(s: &str) -> bool {
    s.chars()
        .take_while(|c| c.is_whitespace())
        .any(|c| c == '\n' || c == '\r')
}

/// Chỉ số khối (trong `blocks`) của khối `Caption` GIỮ ĐƯỢC CHỌN mang vai — ĐÚNG MỘT khối cho
/// mỗi ảnh sở hữu, đúng vế "nhiều nhất một segment mỗi vai" của AD-42 (xem 🔴 ở doc-comment
/// đầu module cho lịch sử lượt sửa). Khối `Caption` GIỮ ĐẦU TIÊN đứng sau một ảnh GIỮ (và
/// trước ảnh GIỮ kế tiếp, nếu có) là khối được chọn; khối thứ hai trở đi của CÙNG ảnh đó không
/// được chọn — nó đi tiếp NHƯ VĂN XUÔI ở cả hai hàm dùng chung hàm này.
///
/// 🔴 **DÙNG CHUNG bởi [`force_caption_segment_boundaries`] VÀ [`weave_chapter_segments`]** —
/// một bản chép độc lập ở mỗi hàm là đúng lớp lỗi mà doc-comment
/// [`anchor::compute_block_prefix_len`] đã cảnh báo cho tiền tố: hai nơi đọc override phải
/// thấy CÙNG một kết quả, nếu không "ép ranh giới" và "gán vai" sẽ rẽ khỏi nhau trên đúng ca
/// một ảnh có từ hai `<figcaption>` liền kề.
///
/// ⚠️ Ảnh KHÔNG GIỮ không đổi ai đang là "ảnh sở hữu hiện tại" — chỉ ảnh GIỮ mới cập nhật nó,
/// đúng khuôn "ảnh GIỮ gần nhất đứng trước" (không phải "ảnh gần nhất bất kể giữ hay không").
fn chosen_caption_indices(blocks: &[Block], effective_kept: &[bool]) -> HashSet<usize> {
    let mut chosen = HashSet::new();
    let mut has_owner = false;
    let mut owner_already_chosen = false;
    for (idx, block) in blocks.iter().enumerate() {
        let kept = is_kept(blocks, effective_kept, idx);
        match &block.body {
            BlockBody::Image { .. } => {
                if kept {
                    has_owner = true;
                    owner_already_chosen = false;
                }
            }
            BlockBody::Caption(_) if kept && has_owner && !owner_already_chosen => {
                chosen.insert(idx);
                owner_already_chosen = true;
            }
            _ => {}
        }
    }
    chosen
}

/// (1) Ép ranh giới segment tại hai đầu MỌI khối `Caption` GIỮ **có ảnh sở hữu** — Quyết định
/// 1 spec 6.13. Khối `Caption` không có ảnh GIỮ nào đứng trước nó đi tiếp NHƯ VĂN XUÔI (I/O
/// Matrix "Caption không có ảnh trước nó") — hàm không ép ranh giới cho nó.
///
/// Vị trí [start, end) của một khối `Caption` trong `full_text` suy bằng ĐÚNG máy dựng tiền tố
/// mà [`anchor::compute_anchor`] dùng ([`anchor::compute_block_prefix_len`]), KHÔNG bằng một
/// phép `find()` — hai chỗ đọc override (đây và `weave_chapter_segments`) phải thấy CÙNG một
/// kết quả. `start` = tiền tố qua các khối TRƯỚC nó; `end` = tiền tố qua CHÍNH nó (bao gồm
/// khe/gap đứng trước nó, xem doc-comment [`join_kept_blocks`]) — khoảng `[start, end)` vì thế
/// gồm khe-trước-caption CỘNG văn bản của chính nó; `.trim()` bóc khe đó ra, cùng cách
/// [`super::split::split_source_text`] luôn TRIM hai đầu một segment.
///
/// Không đủng tới một byte nào của `full_text`: mỗi mảnh đưa vào [`split_source_text`] hoặc
/// đưa thẳng vào một [`SplitSegment`] MỚI là một chuỗi con THẬT, và ba mảnh nối lại
/// (trước-caption, caption, sau-caption) phủ TRỌN `full_text` không chồng không hở.
///
/// ⚠️ **Giới hạn đã biết, ghi ra thay vì để người sau tự tìm:** cắt `full_text` tại `start`
/// rồi chạy `split_source_text` RIÊNG trên từng mảnh có thể cho một kết quả khác một lượt chạy
/// TRỌN nếu luật viết tắt/số thập phân (`split.rs::en_run_is_boundary`) đọc VẮT QUA đúng ranh
/// giới `start`/`end` đó. Trên dữ liệu thật hai ranh giới này LUÔN trùng một `\n` (khối cấp
/// khối của `dom_smoothie`/`TextMode::Formatted` luôn chèn xuống dòng giữa hai khối —
/// `join_kept_blocks`), tức đã là một ranh giới CỨNG mà `split_source_text` cũng sẽ tự cắt ở
/// đó dù không bị ép — nên đây là một giới hạn LÝ THUYẾT trên dữ liệu KHÔNG tới từ
/// `dom_smoothie` (một fixture test dựng tay cố ý phá bất biến đó), không một hồi quy đã đo
/// được trên đường sản phẩm.
pub fn force_caption_segment_boundaries(
    full_text: &str,
    blocks: &[Block],
    effective_kept: &[bool],
    cleanup_rules: &[CleanupRule],
    source_lang: &str,
) -> Vec<SplitSegment> {
    let chosen = chosen_caption_indices(blocks, effective_kept);
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for (idx, block) in blocks.iter().enumerate() {
        if !matches!(block.body, BlockBody::Caption(_)) || !chosen.contains(&idx) {
            continue;
        }
        let start = anchor::compute_block_prefix_len(
            blocks,
            effective_kept,
            idx,
            full_text,
            cleanup_rules,
            source_lang,
        );
        let end = anchor::compute_block_prefix_len(
            blocks,
            effective_kept,
            idx + 1,
            full_text,
            cleanup_rules,
            source_lang,
        );
        // Lỗi tính tiền tố (hiếm, xem `AnchorError`) hoặc `end <= start` (không thể xảy ra
        // trên dữ liệu hợp lệ -- phòng thủ) ⇒ bỏ qua, khối đó đi tiếp NHƯ VĂN XUÔI thay vì làm
        // gãy cả Chương vì một khối caption không ép được.
        if let (Ok(start), Ok(end)) = (start, end) {
            if end > start {
                spans.push((start, end));
            }
        }
    }

    if spans.is_empty() {
        return split_source_text(full_text, source_lang);
    }

    let mut out: Vec<SplitSegment> = Vec::new();
    let mut cursor = 0usize;
    for (start, end) in spans {
        // Phòng thủ: `block_index` tăng dần ⇒ tiền tố (theo cấu trúc `join_kept_blocks`) tăng
        // dần -- một `start` lùi lại là dấu hiệu dữ liệu bất thường, bỏ qua khối đó thay vì
        // panic trên một lát cắt đảo ngược.
        if start < cursor {
            continue;
        }
        if start > cursor {
            let mut piece = split_source_text(&full_text[cursor..start], source_lang);
            // Khe GIỮA đoạn văn xuôi này và khối caption (`full_text[start..end]`, phần đầu)
            // có chứa xuống dòng hay không -- sửa lại cờ kết đoạn của segment CUỐI mảnh này,
            // vì `split_source_text` chạy TRÊN MỘT LÁT không thấy được phần sau `start`.
            if let Some(last) = piece.last_mut() {
                last.is_paragraph_end = leading_gap_has_linebreak(&full_text[start..end]);
            }
            out.extend(piece);
        }
        let caption_text = full_text[start..end].trim();
        if !caption_text.is_empty() {
            out.push(SplitSegment {
                text: caption_text.to_owned(),
                is_paragraph_end: leading_gap_has_linebreak(&full_text[end..]),
            });
        }
        cursor = end;
    }
    if cursor < full_text.len() {
        out.extend(split_source_text(&full_text[cursor..], source_lang));
    }
    // AC7 (`split_source_text`) áp cho CẢ CHƯƠNG: segment cuối cùng luôn tắt cờ kết đoạn, bất
    // kể đường nào sinh ra nó (ba mảnh nối lại ở trên, hay nhánh `spans.is_empty()` phía trên
    // đã tự làm đúng việc này rồi).
    if let Some(last) = out.last_mut() {
        last.is_paragraph_end = false;
    }
    out
}

/// (2) Dệt segment vai vào dãy segment văn xuôi ĐÃ QUA (1) — xem doc-comment đầu module.
///
/// # Tham số
/// - `segments`: `chapter.segments` sau khi [`force_caption_segment_boundaries`] đã chạy (bước
///   7 AD-39) — mỗi khối `Caption` có ảnh sở hữu đã là ĐÚNG MỘT segment trong dãy này.
/// - `full_source_text`/`cleanup_rules`/`source_lang`: ĐÚNG tham số mà
///   [`anchor::compute_anchor`] cần cho CHÍNH Chương này — hàm này tính neo cho MỌI ảnh GIỮ
///   (không chỉ những ảnh đã tải/ghi thành công), vì Quyết định 2 (spec 6.13) sinh segment
///   `alt` cho MỌI ảnh giữ có `alt` khác rỗng, ĐỘC LẬP với việc tải ảnh có thành công hay
///   không (`ASSET` mang neo độc lập với segment đi kèm, epic-6-context.md).
///
/// # Trả về
/// [`WovenChapter`] — không bao giờ lỗi: một ảnh/caption không tính được neo bị BỎ QUA (log
/// chẩn đoán ở chỗ gọi thấy phù hợp, không ở đây — hàm này thuần, không I/O), cùng triết lý "một
/// ảnh trượt không dừng cả lượt nhập" của `commands::project::prepare_chapter_images`.
pub fn weave_chapter_segments(
    blocks: &[Block],
    effective_kept: &[bool],
    segments: &[SplitSegment],
    full_source_text: &str,
    cleanup_rules: &[CleanupRule],
    source_lang: &str,
) -> WovenChapter {
    struct ImageInfo {
        block_index: usize,
        anchor: i64,
        alt: Option<String>,
    }

    // (a) Mọi ảnh GIỮ, kèm neo (tính ĐỘC LẬP với `prepare_chapter_images` -- xem doc-comment
    // trên) và `alt` đã trim, `None` khi rỗng/chỉ khoảng trắng (§Always: "alt rỗng ⇒ không
    // sinh segment").
    let mut images: Vec<ImageInfo> = Vec::new();
    for (idx, block) in blocks.iter().enumerate() {
        if !is_kept(blocks, effective_kept, idx) {
            continue;
        }
        let BlockBody::Image { alt, .. } = &block.body else { continue };
        let Ok(a) = anchor::compute_anchor(
            blocks,
            effective_kept,
            idx,
            full_source_text,
            segments,
            cleanup_rules,
            source_lang,
        ) else {
            continue; // neo khong tinh duoc -- bo qua anh nay, khong dung ca luot det.
        };
        let alt = alt
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned);
        images.push(ImageInfo { block_index: idx, anchor: a, alt });
    }

    // (b) Khối `Caption` ĐƯỢC CHỌN của mỗi ảnh sở hữu -- CHUNG hàm với
    // `force_caption_segment_boundaries` (xem [`chosen_caption_indices`]), để "ép ranh giới"
    // và "gán vai" không bao giờ rẽ khỏi nhau trên một ảnh có từ hai `<figcaption>` liền kề.
    let chosen_captions = chosen_caption_indices(blocks, effective_kept);

    // (c) Chỉ số segment (0-based, trong `segments`) của mỗi khối caption ĐƯỢC CHỌN -- (1) đã
    // ép nó thành NHIỀU NHẤT MỘT segment, nên "số segment đứng TRƯỚC nó" (`compute_anchor` tái
    // dùng nguyên vẹn, nó không đòi `blocks[idx]` phải là `Image`) LÀ chỉ số 0-based của segment
    // đó -- NHƯNG chỉ khi phần mở rộng tiền tố qua ĐÚNG khối này thật sự sinh ra ĐÚNG MỘT
    // segment mới.
    //
    // 🔴 **SỬA 2026-09-10 — không còn suy CHỈ SỐ một mình.** Bản trước chỉ đọc `count_before`
    // rồi coi `segments[count_before]` LÀ caption một cách vô điều kiện. Khi một `CleanupRule`
    // xoá TRẮNG văn bản của khối caption, `force_caption_segment_boundaries` không đẩy segment
    // nào cho span rỗng đó (đúng hành vi) -- nhưng `count_before` vẫn là một chỉ số HỢP LỆ
    // trong mảng `segments` đã "co lại", và nó trỏ vào đoạn văn xuôi TIẾP THEO một cách tình
    // cờ. Đo được: `Some(Caption): "Doan ba dong y nghia…"` -- một đoạn văn xuôi thật bị gắn
    // nhầm vai. So `count_after` (tiền tố qua khối kế tiếp) với `count_before + 1` phân biệt
    // được hai ca: đúng MỘT segment mới sinh ra (caption thật) so với KHÔNG segment nào (bị
    // cleanup xoá trắng) -- chỉ gán vai ở ca đầu.
    let mut caption_segment_indices: HashSet<usize> = HashSet::new();
    for &cap_idx in &chosen_captions {
        let Ok(count_before) = anchor::compute_anchor(
            blocks,
            effective_kept,
            cap_idx,
            full_source_text,
            segments,
            cleanup_rules,
            source_lang,
        ) else {
            continue;
        };
        let Ok(count_after) = anchor::compute_anchor(
            blocks,
            effective_kept,
            cap_idx + 1,
            full_source_text,
            segments,
            cleanup_rules,
            source_lang,
        ) else {
            continue;
        };
        if count_after != count_before + 1 {
            // 0 segment moi (cleanup da xoa trang no) hoac nhieu hon 1 (khong the xay ra tren
            // duong da qua (1), phong thu) -- khong co segment CAPTION THAT nao o day de gan vai.
            continue;
        }
        if let Ok(idx) = usize::try_from(count_before) {
            if idx < segments.len() {
                caption_segment_indices.insert(idx);
            }
        }
    }

    // (d) Điểm chèn `alt` + neo đã dời -- xử ẢNH THEO ĐÚNG THỨ TỰ TÀI LIỆU (neo tăng dần, rồi
    // chỉ số khối tăng dần -- "hai ảnh liền nhau" có thể cùng một neo GỐC).
    images.sort_by_key(|i| (i.anchor, i.block_index));
    let mut shift: i64 = 0;
    let mut shifted_anchor_by_block = BTreeMap::new();
    let mut alt_insertions: Vec<(i64, String)> = Vec::new();
    for img in &images {
        let shifted = img.anchor + shift;
        shifted_anchor_by_block.insert(img.block_index, shifted);
        if let Some(alt_text) = &img.alt {
            alt_insertions.push((img.anchor, alt_text.clone()));
            shift += 1;
        }
    }

    // (e) Dựng dãy cuối cùng: tại vị trí (0-based) `i` trong `segments`, chèn mọi `alt` có
    // điểm chèn == `i` (thứ tự tài liệu, đã sắp ở bước (d)) NGAY TRƯỚC `segments[i]`; nếu `i`
    // là chỉ số của một segment caption có chủ, gắn `role: Caption` cho nó.
    let mut out = Vec::with_capacity(segments.len() + alt_insertions.len());
    let mut alt_iter = alt_insertions.into_iter().peekable();
    for i in 0..=segments.len() {
        // 🔴 SỬA 2026-09-10 — `peek()` rồi `next().unwrap_or_else(|| unreachable!(...))` ngay
        // sau là một điểm PANIC dưới `panic = "abort"` (giết cả tiến trình, không unwind) nếu
        // một lượt tách rời logic tương lai làm hai lời gọi đó không còn đồng bộ. Viết lại
        // bằng `let ... else` trên CHÍNH `next()`: không còn `peek()` riêng, không còn nhánh
        // nào giả định "chắc chắn `Some`" mà không tự kiểm.
        loop {
            let Some(&(pos, _)) = alt_iter.peek() else { break };
            if pos != i as i64 {
                break;
            }
            let Some((_, text)) = alt_iter.next() else { break };
            out.push(WovenSegment {
                text,
                is_paragraph_end: false,
                role: Some(SegmentRole::Alt),
            });
        }
        if i < segments.len() {
            let role = caption_segment_indices
                .contains(&i)
                .then_some(SegmentRole::Caption);
            out.push(WovenSegment {
                text: segments[i].text.clone(),
                is_paragraph_end: segments[i].is_paragraph_end,
                role,
            });
        }
    }

    WovenChapter {
        segments: out,
        shifted_anchor_by_block,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::segment::pipeline::join_kept_blocks;

    fn text_block(text: &str) -> Block {
        Block { body: BlockBody::Paragraph(text.to_owned()), machine_kept: true, exact_gap_before: String::new(), is_list_item: false }
    }

    fn text_block_gap(text: &str, gap: &str) -> Block {
        Block { body: BlockBody::Paragraph(text.to_owned()), machine_kept: true, exact_gap_before: gap.to_owned(), is_list_item: false }
    }

    fn image_block(alt: Option<&str>) -> Block {
        Block {
            body: BlockBody::Image { src: None, alt: alt.map(str::to_owned) },
            machine_kept: true,
            exact_gap_before: String::new(),
            is_list_item: false,
        }
    }

    fn caption_block_gap(text: &str, gap: &str) -> Block {
        Block { body: BlockBody::Caption(text.to_owned()), machine_kept: true, exact_gap_before: gap.to_owned(), is_list_item: false }
    }

    fn all_kept(n: usize) -> Vec<bool> {
        vec![true; n]
    }

    // ── force_caption_segment_boundaries ───────────────────────────────────────────

    #[test]
    fn a_multi_sentence_caption_with_an_owning_image_becomes_exactly_one_segment() {
        let blocks = vec![
            text_block("Cau mo dau."),
            image_block(Some("mo ta")),
            caption_block_gap("Cau chu thich mot. Cau chu thich hai.", "\n\n"),
            text_block_gap("Cau ket thuc.", "\n\n"),
        ];
        let kept = all_kept(blocks.len());
        let full = join_kept_blocks(&blocks, &kept);

        let segments = force_caption_segment_boundaries(&full, &blocks, &kept, &[], "en");

        // Van xuoi ghep tu 3 khoi chu (mo dau, chu thich x2 cau, ket thuc) -- neu KHONG ep,
        // chu thich se tach thanh HAI segment (hai dau cham). Co ep: dung BA segment.
        assert_eq!(segments.len(), 3, "phai dung ba segment (mo dau, chu thich MOT segment, ket thuc): {segments:?}");
        assert_eq!(segments[1].text, "Cau chu thich mot. Cau chu thich hai.");
        // Segment cuoi CHUONG luon tat co ket doan (AC7).
        assert!(!segments.last().unwrap().is_paragraph_end);
        // Moi segment phai la CHUOI CON THAT cua full text, dung thu tu, khong mat byte.
        let mut cursor = 0usize;
        for seg in &segments {
            let rel = full[cursor..].find(seg.text.as_str()).expect("segment phai la chuoi con that");
            cursor += rel + seg.text.len();
        }
    }

    #[test]
    fn a_caption_with_no_preceding_kept_image_is_split_as_ordinary_prose() {
        let blocks = vec![
            caption_block_gap("Chu thich mo dau. Cau hai.", ""),
            text_block_gap("Doan sau.", "\n\n"),
        ];
        let kept = all_kept(blocks.len());
        let full = join_kept_blocks(&blocks, &kept);

        let segments = force_caption_segment_boundaries(&full, &blocks, &kept, &[], "en");

        // Khong anh nao dung truoc -- khong ep, hai cau cua caption tach thanh HAI segment.
        assert_eq!(segments.len(), 3, "caption khong co anh phai tach nhu van xuoi thuong: {segments:?}");
    }

    #[test]
    fn no_caption_blocks_falls_through_to_plain_split() {
        let blocks = vec![text_block("Mot cau."), image_block(None)];
        let kept = all_kept(blocks.len());
        let full = join_kept_blocks(&blocks, &kept);
        let segments = force_caption_segment_boundaries(&full, &blocks, &kept, &[], "en");
        assert_eq!(segments, split_source_text(&full, "en"));
    }

    // ── weave_chapter_segments ──────────────────────────────────────────────────────

    #[test]
    fn an_image_with_alt_gets_an_alt_segment_right_after_its_anchor() {
        let blocks = vec![text_block("Cau mot."), image_block(Some("mo ta anh")), text_block_gap("Cau hai.", "\n\n")];
        let kept = all_kept(blocks.len());
        let full = join_kept_blocks(&blocks, &kept);
        let segments = split_source_text(&full, "en");
        assert_eq!(segments.len(), 2);

        let woven = weave_chapter_segments(&blocks, &kept, &segments, &full, &[], "en");

        assert_eq!(woven.segments.len(), 3, "them dung mot segment alt: {:?}", woven.segments);
        assert_eq!(woven.segments[0].role, None);
        assert_eq!(woven.segments[1].text, "mo ta anh");
        assert_eq!(woven.segments[1].role, Some(SegmentRole::Alt));
        assert_eq!(woven.segments[2].role, None);
        assert_eq!(woven.shifted_anchor_by_block.get(&1), Some(&1), "neo cua anh (block 1) khong doi vi khong co anh nao truoc no chen them segment");
    }

    #[test]
    fn an_image_without_alt_inserts_nothing() {
        let blocks = vec![text_block("Cau mot."), image_block(None), text_block_gap("Cau hai.", "\n\n")];
        let kept = all_kept(blocks.len());
        let full = join_kept_blocks(&blocks, &kept);
        let segments = split_source_text(&full, "en");

        let woven = weave_chapter_segments(&blocks, &kept, &segments, &full, &[], "en");
        assert_eq!(woven.segments.len(), segments.len(), "khong alt -- khong segment nao them vao");
        assert!(woven.segments.iter().all(|s| s.role.is_none()));
    }

    #[test]
    fn an_alt_only_owner_shifts_the_anchor_of_images_that_come_after_it() {
        // Hai anh LIEN NHAU, khong chu gi o giua -- ca hai cung mot neo GOC.
        let blocks = vec![
            text_block("Cau mot."),
            image_block(Some("anh A")),
            image_block(Some("anh B")),
            text_block_gap("Cau hai.", "\n\n"),
        ];
        let kept = all_kept(blocks.len());
        let full = join_kept_blocks(&blocks, &kept);
        let segments = split_source_text(&full, "en");

        let woven = weave_chapter_segments(&blocks, &kept, &segments, &full, &[], "en");

        let anchor_a = *woven.shifted_anchor_by_block.get(&1).expect("anh A phai co neo");
        let anchor_b = *woven.shifted_anchor_by_block.get(&2).expect("anh B phai co neo");
        assert_eq!(anchor_b, anchor_a + 1, "neo B phai cong them 1 vi alt cua A da chen truoc no");
    }

    #[test]
    fn a_caption_owned_by_an_image_gets_tagged_and_ordered_after_its_alt() {
        let blocks = vec![
            text_block("Cau mot."),
            image_block(Some("mo ta")),
            caption_block_gap("Chu thich.", "\n\n"),
            text_block_gap("Cau sau.", "\n\n"),
        ];
        let kept = all_kept(blocks.len());
        let full = join_kept_blocks(&blocks, &kept);
        let segments = force_caption_segment_boundaries(&full, &blocks, &kept, &[], "en");

        let woven = weave_chapter_segments(&blocks, &kept, &segments, &full, &[], "en");

        let roles: Vec<Option<SegmentRole>> = woven.segments.iter().map(|s| s.role).collect();
        let texts: Vec<&str> = woven.segments.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(texts, vec!["Cau mot.", "mo ta", "Chu thich.", "Cau sau."]);
        assert_eq!(roles, vec![None, Some(SegmentRole::Alt), Some(SegmentRole::Caption), None]);
    }

    /// 🔴 Vòng rà 2026-09-10 — AD-42 "nhiều nhất một segment mỗi vai" áp CHO CẢ khối `Caption`
    /// thứ hai trở đi của CÙNG một ảnh. Hai khối caption liền kề, không ảnh nào chen giữa: chỉ
    /// khối ĐẦU TIÊN được ép ranh giới/gán vai, khối thứ hai đi tiếp như văn xuôi.
    #[test]
    fn two_captions_owned_by_the_same_image_only_the_first_gets_the_caption_role() {
        let blocks = vec![
            text_block("Cau mot."),
            image_block(Some("mo ta")),
            caption_block_gap("Chu thich mot.", "\n\n"),
            caption_block_gap("Chu thich hai.", "\n\n"),
            text_block_gap("Cau sau.", "\n\n"),
        ];
        let kept = all_kept(blocks.len());
        let full = join_kept_blocks(&blocks, &kept);
        let segments = force_caption_segment_boundaries(&full, &blocks, &kept, &[], "en");

        let woven = weave_chapter_segments(&blocks, &kept, &segments, &full, &[], "en");

        let caption_rows: Vec<&WovenSegment> =
            woven.segments.iter().filter(|s| s.role == Some(SegmentRole::Caption)).collect();
        assert_eq!(
            caption_rows.len(),
            1,
            "AD-42: nhieu nhat MOT segment role='caption' cho MOT anh, du no co hai \
             <figcaption> lien ke: {:?}",
            woven.segments
        );
        assert_eq!(caption_rows[0].text, "Chu thich mot.", "khoi DAU TIEN moi duoc chon");
        assert!(
            woven.segments.iter().any(|s| s.text == "Chu thich hai." && s.role.is_none()),
            "khoi THU HAI phai di tiep NHU VAN XUOI (role=None), khong bi ep rieng: {:?}",
            woven.segments
        );
    }

    /// 🔴 Vòng rà 2026-09-10 — một `CleanupRule` xoá TRẮNG văn bản của khối caption đã chọn
    /// không được để đoạn văn xuôi KẾ TIẾP thừa hưởng `role='caption'` theo chỉ số.
    #[test]
    fn a_cleanup_rule_that_erases_the_caption_text_tags_no_segment_at_all() {
        fn enabled_literal_rule(pattern: &str) -> crate::core::cleanup::CleanupRule {
            crate::core::cleanup::CleanupRule {
                tier: crate::core::cleanup::CleanupRuleTier::Global,
                id: 1,
                pattern: pattern.to_owned(),
                kind: crate::core::cleanup::CleanupRuleKind::Literal,
                enabled: true,
            }
        }

        let blocks = vec![
            text_block("Cau mot."),
            image_block(Some("mo ta")),
            caption_block_gap("Chu thich se bi xoa.", "\n\n"),
            text_block_gap("Doan van xuoi sau day khong duoc mang vai nao ca.", "\n\n"),
        ];
        let kept = all_kept(blocks.len());
        let rules = vec![enabled_literal_rule("Chu thich se bi xoa.")];
        let full = join_kept_blocks(&blocks, &kept);
        // Chuoi that: cleanup roi chuan hoa chay TRUOC khi bo tach thay -- day la dung
        // `full_source_text` ma ca hai ham doc (khuon cac test khac trong tep nay dung `&[]`
        // roi dung nguyen `full`; o day PHAI ap cleanup that vi do la dieu ca nay do).
        let cleaned = crate::core::cleanup::apply(&full, &rules).expect("cleanup toan van ban");
        let source_text = crate::core::segment::normalize::normalize(&cleaned.text, "en").text;
        assert!(!source_text.contains("Chu thich se bi xoa"), "fixture phai THAT SU bi xoa trang");

        let segments = force_caption_segment_boundaries(&source_text, &blocks, &kept, &rules, "en");
        assert!(
            !segments.iter().any(|s| s.text.contains("Chu thich")),
            "0 segment nao con mang chu cua caption da bi xoa trang: {segments:?}"
        );

        let woven = weave_chapter_segments(&blocks, &kept, &segments, &source_text, &rules, "en");
        assert!(
            woven.segments.iter().all(|s| s.role != Some(SegmentRole::Caption)),
            "khong hang nao duoc gan role='caption' -- caption that da bi cleanup xoa trang, \
             khong con segment nao la no: {:?}",
            woven.segments
        );
        let prose = woven
            .segments
            .iter()
            .find(|s| s.text.contains("Doan van xuoi sau day"))
            .expect("doan van xuoi sau caption phai con nguyen");
        assert_eq!(
            prose.role, None,
            "doan van xuoi KE TIEP caption da bi xoa khong duoc thua huong role='caption' \
             theo CHI SO: {:?}",
            woven.segments
        );
    }

    #[test]
    fn an_owned_caption_without_a_matching_image_alt_still_gets_tagged_with_no_alt_inserted() {
        let blocks = vec![image_block(None), caption_block_gap("Chu thich rieng.", "\n\n")];
        let kept = all_kept(blocks.len());
        let full = join_kept_blocks(&blocks, &kept);
        let segments = force_caption_segment_boundaries(&full, &blocks, &kept, &[], "en");

        let woven = weave_chapter_segments(&blocks, &kept, &segments, &full, &[], "en");
        assert_eq!(woven.segments.len(), 1, "khong alt de chen, chi con segment caption: {:?}", woven.segments);
        assert_eq!(woven.segments[0].role, Some(SegmentRole::Caption));
    }
}
