//! Neo vị trí của một ảnh GIỮ trong Chương — Story 6.11 (FR127), tính **MỘT LẦN lúc nhập**
//! (cùng luật ranh giới segment của AD-4, `AGENTS.md`); không đường mã nào tính lại lúc nạp.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! CƠ CHẾ — chạy lại hai bước biến đổi văn bản trên TIỀN TỐ đứng trước ảnh (§Design Notes
//! spec 6.11)
//! ─────────────────────────────────────────────────────────────────────────────
//! Ảnh bị bỏ khỏi văn bản ở bước ghép (`pipeline::join_kept_blocks`), nên vị trí của nó không
//! sống sót qua hai bước biến đổi sau đó (làm sạch, chuẩn hoá). [`compute_anchor`] tái tạo lại
//! đúng những gì Chương đã trải qua, nhưng chỉ trên phần văn bản đứng TRƯỚC ảnh:
//!
//! 1. Ghép văn bản của các khối GIỮ đứng TRƯỚC ảnh (dùng ĐÚNG [`super::pipeline::join_kept_blocks`]
//!    trên một LÁT của `blocks`/`effective_kept`, không phải một hàm ghép riêng — hai nơi đọc
//!    override phải thấy CÙNG một kết quả, đúng lý do doc-comment của chính hàm đó).
//! 2. Chạy [`crate::core::cleanup::apply`] rồi [`super::normalize::normalize`] trên tiền tố đó
//!    — ĐÚNG hai bước 3/4 của AD-39 mà `chapter.source_text` đã đi qua.
//! 3. **TỰ KIỂM**: tiền tố sau biến đổi PHẢI là một tiền tố THẬT (byte-for-byte) của
//!    `chapter.source_text` — nếu không, [`AnchorError`] trả về, KHÔNG làm tròn về `0`.
//! 4. Đếm bao nhiêu segment (từ `chapter.segments`, đã tính sẵn bởi bước 7 AD-39) nằm TRỌN
//!    trong tiền tố đó — con số này CHÍNH LÀ `anchor_after_segment_ord` (segment `ord` bắt đầu
//!    từ 1 và liên tục trong một Chương, `commands::segment::insert_segments`).
//!
//! ⚠️ **Giới hạn thật (§Ask First spec 6.11), ghi ra thay vì để người sau tự phát hiện:** tính
//! phân tách-theo-tiền-tố là một GIẢ ĐỊNH ĐO ĐƯỢC (8/8 trên bảy mẫu bàn đo 6.1, 2026-09-08),
//! không một định lý — một luật làm sạch khớp VẮT QUA ranh giới tiền tố sẽ phá nó. Vì thế
//! bước 3 tự kiểm và trả lỗi phân biệt được thay vì im lặng làm tròn: một neo sai im lặng đặt
//! ảnh vào giữa một câu và không cổng nào đỏ.

use crate::core::cleanup::CleanupRule;
use crate::core::webimport::Block;

use super::normalize;
use super::pipeline::join_kept_blocks;
use super::split::SplitSegment;

/// [`compute_anchor`] không tính được neo cho ảnh này — `detail` chỉ để chẩn đoán/log
/// (KHÔNG DẤU, NFR16), KHÔNG BAO GIỜ hiển thị cho người dùng nguyên văn.
#[derive(Debug)]
pub struct AnchorError {
    pub detail: String,
}

/// Tính `anchor_after_segment_ord` cho ảnh ở chỉ số `image_index` trong `blocks` — xem
/// doc-comment đầu module cho cơ chế đầy đủ.
///
/// # Tham số
/// - `blocks`/`effective_kept`: mô hình khối CẢ TRANG của Chương cộng trạng thái giữ HIỆU LỰC
///   (đã áp `PipelineInput::block_overrides` qua [`super::pipeline::effective_kept_for_blocks`]
///   — CÙNG cặp mà [`super::pipeline::Step::ExtractMainContent`] đã dùng để ghép
///   `chapter.source_text`, không phải một bản tính lại độc lập).
/// - `image_index`: chỉ số của một khối trong `blocks`, dùng để CẮT LÁT `blocks[..image_index]`
///   — hàm không đọc `blocks[image_index]` một lần nào, nên nó KHÔNG đòi khối đó phải là
///   [`crate::core::webimport::BlockBody::Image`].
///   🔵 **SỬA 2026-09-10 (Story 6.13) — mệnh đề cũ ở đây ("chỗ gọi đảm bảo Image") đã HẾT
///   ĐÚNG, không đổi một dòng hành vi.** `core::segment::role::weave_chapter_segments` gọi
///   hàm này với chỉ số của một khối `Caption` (đếm segment đứng trước/qua nó để suy chỉ số
///   segment tương ứng, không phải để tính neo một ảnh) — tên tham số `image_index` giữ
///   nguyên vì Story 6.11 là chỗ gọi ĐẦU TIÊN, nhưng câu mô tả phải nói đúng những gì hàm THẬT
///   SỰ đòi hỏi ở tham số này.
/// - `full_source_text`: `chapter.source_text` đã ghi xuống — mốc để TỰ KIỂM tiền tố.
/// - `segments`: `chapter.segments` — đã tính sẵn ở bước 7 AD-39, dùng để ĐẾM, không tính lại.
/// - `cleanup_rules`/`source_lang`: ĐÚNG tham số mà bước 3/4 AD-39 đã dùng cho CHÍNH Chương
///   này (`PipelineInput::cleanup_rules`/`source_lang`).
///
/// # Lỗi
/// [`AnchorError`] khi tiền tố sau biến đổi KHÔNG phải một tiền tố byte-for-byte của
/// `full_source_text` — xem §Ask First/§Design Notes spec 6.11.
pub fn compute_anchor(
    blocks: &[Block],
    effective_kept: &[bool],
    image_index: usize,
    full_source_text: &str,
    segments: &[SplitSegment],
    cleanup_rules: &[CleanupRule],
    source_lang: &str,
) -> Result<i64, AnchorError> {
    // 🔵 SỬA (vòng rà đối kháng 2, mục B2) — kiểm biên TRƯỚC khi cắt lát. `blocks[..image_index]`/
    // `effective_kept[..image_index]` PANIC (không phải một `Result`) nếu `image_index` vượt
    // độ dài của MỘT trong hai slice — và `Cargo.toml` đặt `panic = "abort"`, nên một chỗ gọi
    // sai (tương lai, hoặc một fixture test) sẽ giết cả tiến trình thay vì trả một `AnchorError`
    // phân biệt được, đúng lớp lỗi mà B1/module này vừa được sửa để tránh.
    if image_index > blocks.len() || image_index > effective_kept.len() {
        return Err(AnchorError {
            detail: format!(
                "image_index ({image_index}) vuot qua do dai blocks ({}) hoac effective_kept \
                 ({}) -- khong the cat lat TRUOC anh, day la mot loi goi ham, khong phai mot \
                 dieu kien du lieu nguoi dung",
                blocks.len(),
                effective_kept.len()
            ),
        });
    }

    // Lát TRƯỚC ảnh — `join_kept_blocks` chỉ nhìn NGƯỢC (không đọc gì ở chỉ số >= độ dài lát),
    // nên cắt lát rồi ghép cho ĐÚNG kết quả mà việc ghép TOÀN Chương từng cho ở đoạn này.
    let prefix_raw = join_kept_blocks(&blocks[..image_index], &effective_kept[..image_index]);

    let cleaned = crate::core::cleanup::apply(&prefix_raw, cleanup_rules).map_err(|e| AnchorError {
        detail: format!("cleanup tren tien to that bai: {e}"),
    })?;
    let normalized = normalize::normalize(&cleaned.text, source_lang);
    let prefix = normalized.text;

    if !full_source_text.starts_with(prefix.as_str()) {
        return Err(AnchorError {
            detail: format!(
                "tien to sau chuan hoa ({} byte) khong phai tien to cua chapter.source_text \
                 ({} byte) -- gia dinh phan tach-theo-tien-to (Design Notes spec 6.11) da lech \
                 tren du lieu nay, dung lam tron ve 0",
                prefix.len(),
                full_source_text.len(),
            ),
        });
    }

    segment_count_within_prefix(segments, full_source_text, prefix.len()).map(|count| count as i64)
}

/// Độ dài (byte) của tiền tố `full_source_text` sinh ra bởi các khối `blocks[..block_index]`
/// ĐANG GIỮ — **Story 6.13**, cần cho đúng máy dựng tiền tố mà [`compute_anchor`] đã dùng, khi
/// bên gọi cần TOẠ ĐỘ BYTE thay vì SỐ SEGMENT: ép ranh giới segment tại hai đầu một khối
/// `Caption` (Quyết định 1, spec 6.13) đòi biết ĐÚNG nơi khối đó bắt đầu/kết thúc trong
/// `source_text`, việc mà đếm segment không trả lời được.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT HÀM RIÊNG, KHÔNG REFACTOR [`compute_anchor`] ĐỂ DÙNG CHUNG
/// ─────────────────────────────────────────────────────────────────────────────
/// [`compute_anchor`] đã qua ba vòng rà đối kháng với một bộ test dày (xem `#[cfg(test)]` cuối
/// tệp) — tách một tham số MỚI (kiểu trả về) ra khỏi nó là rủi ro không cần thiết cho một thay
/// đổi mà bản thân `compute_anchor` không cần. Ba bước ở đây (ghép + làm sạch + chuẩn hoá) và
/// phép tự kiểm byte-for-byte là bản sao NGẮN, cùng nguồn dữ liệu ([`join_kept_blocks`],
/// [`crate::core::cleanup::apply`], [`normalize::normalize`]) — khác nhau đúng MỘT chỗ: hàm
/// này trả `prefix.len()` (toạ độ byte) thay vì đếm segment. Cùng bất biến, cùng cách trả lỗi
/// phân biệt được — không làm tròn về `0` khi tiền tố tính được KHÔNG phải một tiền tố thật.
///
/// # Tham số
/// Giống hệt [`compute_anchor`], trừ `block_index` thay cho `image_index` — hàm này KHÔNG đòi
/// `blocks[block_index]` phải là một [`crate::core::webimport::BlockBody::Image`] (chỗ gọi
/// của Story 6.13 truyền cả chỉ số một khối `Caption`, và `block_index == blocks.len()` để lấy
/// tiền tố tính TRỌN mọi khối).
///
/// # Lỗi
/// [`AnchorError`] khi `block_index` vượt độ dài `blocks`/`effective_kept`, hoặc khi tiền tố
/// sau biến đổi KHÔNG phải một tiền tố byte-for-byte của `full_source_text` — cùng hai điều
/// kiện mà [`compute_anchor`] đã canh.
pub fn compute_block_prefix_len(
    blocks: &[Block],
    effective_kept: &[bool],
    block_index: usize,
    full_source_text: &str,
    cleanup_rules: &[CleanupRule],
    source_lang: &str,
) -> Result<usize, AnchorError> {
    if block_index > blocks.len() || block_index > effective_kept.len() {
        return Err(AnchorError {
            detail: format!(
                "block_index ({block_index}) vuot qua do dai blocks ({}) hoac effective_kept \
                 ({}) -- khong the cat lat, day la mot loi goi ham, khong phai mot dieu kien du \
                 lieu nguoi dung",
                blocks.len(),
                effective_kept.len()
            ),
        });
    }

    let prefix_raw = join_kept_blocks(&blocks[..block_index], &effective_kept[..block_index]);
    let cleaned = crate::core::cleanup::apply(&prefix_raw, cleanup_rules).map_err(|e| AnchorError {
        detail: format!("cleanup tren tien to that bai: {e}"),
    })?;
    let normalized = normalize::normalize(&cleaned.text, source_lang);
    let prefix = normalized.text;

    if !full_source_text.starts_with(prefix.as_str()) {
        return Err(AnchorError {
            detail: format!(
                "tien to sau chuan hoa ({} byte) khong phai tien to cua chapter.source_text \
                 ({} byte) -- gia dinh phan tach-theo-tien-to (Design Notes spec 6.11) da lech \
                 tren du lieu nay, dung lam tron ve 0",
                prefix.len(),
                full_source_text.len(),
            ),
        });
    }

    Ok(prefix.len())
}

/// Đếm bao nhiêu segment ĐẦU TIÊN (theo thứ tự `ord`) nằm TRỌN trong `full_text[..prefix_len]`.
///
/// `SplitSegment::text` không mang toạ độ byte gốc (chỉ mang văn bản đã TRIM) — nhưng các
/// segment được [`super::split::split_source_text`] cắt theo đúng thứ tự trái-sang-phải,
/// không chồng nhau, không đảo thứ tự, nên TÌM tuần tự (bắt đầu từ nơi segment TRƯỚC đó kết
/// thúc) khôi phục lại đúng toạ độ mà không cần bộ tách trả thêm một trường nào.
///
/// 🔴 **SỬA 2026-09-08 (mục B2 vòng rà đối kháng 3 lớp) — trả `Result`, không còn `usize`
/// trần.** Bản trước có BA nhánh `else { break }`/`if { break }` trong vòng lặp, và CHỈ MỘT
/// trong ba (`abs_end > prefix_len`, "đã đếm đủ, dừng ĐÚNG chỗ") là một điều kiện DỪNG hợp
/// lệ. Hai nhánh còn lại (`full_text.get(cursor..)` trả `None`, `haystack.find(...)` không
/// khớp) là VI PHẠM bất biến "mỗi segment là một chuỗi con THẬT của `full_text`, theo đúng
/// thứ tự" — bản trước `break` y hệt nhánh hợp lệ, nên một lượt vi phạm trả về một `count`
/// NHỎ HƠN sự thật một cách IM LẶNG, đúng lớp lỗi mà doc-comment đầu module thề không bao giờ
/// làm (tự kiểm ở [`compute_anchor`] chỉ phủ bước GHÉP tiền tố, không phủ bước ĐẾM này — hai
/// bước khác nhau, một tự kiểm không tự động phủ luôn bước kia). ⇒ Hai nhánh vi phạm nay trả
/// [`AnchorError`] phân biệt được; nhánh DỪNG hợp lệ vẫn `break`, không đổi.
fn segment_count_within_prefix(
    segments: &[SplitSegment],
    full_text: &str,
    prefix_len: usize,
) -> Result<usize, AnchorError> {
    let mut cursor = 0usize;
    let mut count = 0usize;
    for (i, seg) in segments.iter().enumerate() {
        // 🔵 SỬA (vòng rà đối kháng 2, mục B3) — `seg.text` RỖNG làm `str::find("")` trả
        // `Some(0)` LUÔN LUÔN, khiến `abs_end == cursor` (cursor không tiến) — hàm này vẫn
        // `count += 1` cho segment đó, đếm THỪA một segment không thật sự chiếm byte nào của
        // tiền tố. `split_source_text` (nguồn THẬT của `segments` trên đường sản phẩm) khai
        // KHÔNG BAO GIỜ sinh segment rỗng, nhưng hàm này nhận `&[SplitSegment]` TỪ NGOÀI (một
        // chữ ký công khai trong module), không tự sinh ra `segments` — một fixture test hay
        // một chỗ gọi tương lai đưa vào một segment rỗng vẫn phải bị bắt tại đây, phân biệt
        // được, không đếm thừa âm thầm.
        if seg.text.is_empty() {
            return Err(AnchorError {
                detail: format!(
                    "segment #{i} (ord {}) la CHUOI RONG -- vi pham bat bien 'segment la chuoi \
                     con THAT, khong rong' (split_source_text khong bao gio sinh segment rong; \
                     mot segment rong o day la du lieu goi ham sai, khong phai du lieu Chuong \
                     that), khong dem no de tranh cursor dung yen va dem thua",
                    i + 1
                ),
            });
        }
        let Some(haystack) = full_text.get(cursor..) else {
            return Err(AnchorError {
                detail: format!(
                    "segment #{i} (ord {}): cursor {cursor} vuot qua do dai full_text ({} byte) \
                     -- vi pham bat bien 'segment la chuoi con THAT theo dung thu tu', khong \
                     dem tiep va lam tron",
                    i + 1,
                    full_text.len()
                ),
            });
        };
        let Some(rel) = haystack.find(seg.text.as_str()) else {
            return Err(AnchorError {
                detail: format!(
                    "segment #{i} (ord {}) khong tim thay nhu mot chuoi con cua full_text tu \
                     vi tri byte {cursor} tro di -- vi pham bat bien 'segment la chuoi con THAT \
                     theo dung thu tu', khong dem tiep va lam tron",
                    i + 1
                ),
            });
        };
        let abs_start = cursor + rel;
        let abs_end = abs_start + seg.text.len();
        if abs_end > prefix_len {
            break;
        }
        count += 1;
        cursor = abs_end;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::webimport::BlockBody;
    use crate::core::segment::split::split_source_text;

    fn text_block(text: &str, kept: bool) -> Block {
        text_block_with_gap(text, kept, "")
    }

    /// `exact_gap_before` mô phỏng khoảng trắng GỐC mà `Extractor` thật sự chụp lại giữa hai
    /// khối `machine_kept` liền nhau (xem doc-comment `Block::exact_gap_before`) — một fixture
    /// tự tay dựng phải điền đúng trường này khi hai khối chữ `machine_kept` của nó KHÔNG có
    /// khối chữ nào khác đứng giữa (chỉ ảnh, không tính), nếu không [`join_kept_blocks`] sẽ
    /// nối liền chúng KHÔNG dấu cách, đúng như trên trang thật nó sẽ KHÔNG làm.
    fn text_block_with_gap(text: &str, kept: bool, gap: &str) -> Block {
        Block { body: BlockBody::Paragraph(text.to_owned()), machine_kept: kept, exact_gap_before: gap.to_owned(), is_list_item: false }
    }

    fn image_block() -> Block {
        Block { body: BlockBody::Image { src: None, alt: None }, machine_kept: true, exact_gap_before: String::new(), is_list_item: false }
    }

    #[test]
    fn anchor_before_the_first_kept_block_is_zero() {
        let blocks = vec![image_block(), text_block("Cau mot. Cau hai.", true)];
        let effective_kept = vec![true, true];
        let full = join_kept_blocks(&blocks, &effective_kept);
        let segments = split_source_text(&full, "en");

        let anchor = compute_anchor(&blocks, &effective_kept, 0, &full, &segments, &[], "en")
            .expect("anh dau Chuong phai tinh neo duoc");
        assert_eq!(anchor, 0, "khong segment nao dung TRUOC anh dau Chuong");
    }

    /// C5 (vòng rà đối kháng 2, 3 lớp) — "tìm tuần tự khôi phục đúng toạ độ" (doc-comment
    /// `segment_count_within_prefix`) từng đứng bằng SUY DIỄN, chưa có ca nào gieo văn bản
    /// LẶP (hai câu TRÙNG CHỮ, ví dụ một dòng thoại lặp lại) để đo THẬT. Đo được: vì mỗi lượt
    /// `find` chỉ tìm trong `full_text[cursor..]` (phần CÒN LẠI, không phải toàn văn bản), một
    /// segment trùng chữ với segment TRƯỚC nó vẫn khớp đúng LẦN XUẤT HIỆN KẾ TIẾP (không bị
    /// tìm nhầm lại lần xuất hiện ĐẦU đã tiêu thụ) — kiểm ở CẢ prefix TRỌN VẸN lẫn một prefix
    /// DỪNG GIỮA hai lần lặp.
    #[test]
    fn a_repeated_identical_sentence_still_recovers_correct_sequential_coordinates() {
        let full_text = "Duoc khong. Duoc khong. Cau ba khac han.";
        let segments = split_source_text(full_text, "en");
        assert!(
            segments.len() >= 3,
            "fixture phai tach it nhat ba cau (hai cau dau TRUNG CHU): {segments:?}"
        );
        assert_eq!(
            segments[0].text, segments[1].text,
            "hai segment DAU phai TRUNG CHU -- day chinh la dieu kien ca nay can do"
        );

        // (a) Prefix TRỌN VẸN — mọi segment (kể cả hai segment trùng chữ) phải đếm đủ.
        let result_full = segment_count_within_prefix(&segments, full_text, full_text.len());
        match result_full {
            Ok(n) => assert_eq!(n, segments.len(), "moi segment (ke ca hai segment TRUNG CHU) phai duoc dem du"),
            Err(e) => panic!("khong duoc loi tren mot van ban HOP LE, chi co cau trung chu: {}", e.detail),
        }

        // (b) Prefix DỪNG NGAY SAU lần lặp THỨ NHẤT — phải đếm ĐÚNG MỘT (không phải 0, không
        // phải 2), chứng minh cursor đã tiến qua đúng LẦN XUẤT HIỆN ĐẦU, không đứng yên hay
        // nhảy nhầm sang lần thứ hai.
        let first_end = segments[0].text.len();
        // `full_text` bắt đầu ĐÚNG bằng segments[0].text (không khoảng trắng dẫn đầu).
        assert!(full_text.starts_with(segments[0].text.as_str()));
        let result_partial = segment_count_within_prefix(&segments, full_text, first_end);
        match result_partial {
            Ok(n) => assert_eq!(
                n, 1,
                "prefix dung NGAY SAU lan lap THU NHAT phai dem DUNG MOT segment, khong duoc \
                 nham sang lan thu hai (chung minh cursor tien dung)"
            ),
            Err(e) => panic!("khong duoc loi khi prefix dung dung tai bien mot segment: {}", e.detail),
        }
    }

    /// B3 (vòng rà đối kháng 2, 3 lớp) — một `SplitSegment` mang chuỗi RỖNG phải trả lỗi
    /// phân biệt được, không được đếm THỪA một segment không chiếm byte nào (`find("")` luôn
    /// khớp tại vị trí 0, cursor đứng yên).
    #[test]
    fn a_segment_with_empty_text_is_a_distinguishable_error_not_a_silent_overcount() {
        let full_text = "Cau mot that su.";
        let segments = vec![
            SplitSegment { text: String::new(), is_paragraph_end: false },
            SplitSegment { text: "Cau mot that su.".to_owned(), is_paragraph_end: false },
        ];
        let result = segment_count_within_prefix(&segments, full_text, full_text.len());
        assert!(result.is_err(), "mot segment RONG phai la Err, khong duoc dem thua");
    }

    /// B2 (vòng rà đối kháng 2, 3 lớp) — `image_index` vượt độ dài `blocks`/`effective_kept`
    /// phải trả lỗi PHÂN BIỆT ĐƯỢC, không được `panic` (slice index out of bounds) — một
    /// panic ở đây giết cả tiến trình (`panic = "abort"`), khác hẳn một `Result::Err` mà
    /// chỗ gọi xử lý được.
    #[test]
    fn an_image_index_past_the_end_of_blocks_or_effective_kept_is_a_distinguishable_error_not_a_panic() {
        let blocks = vec![text_block("Cau mot.", true)];
        let effective_kept = vec![true];

        let result = compute_anchor(&blocks, &effective_kept, 5, "Cau mot.", &[], &[], "en");
        assert!(result.is_err(), "image_index vuot qua blocks.len() phai la Err, khong panic");

        let short_kept = vec![];
        let result2 = compute_anchor(&blocks, &short_kept, 1, "Cau mot.", &[], &[], "en");
        assert!(result2.is_err(), "image_index vuot qua effective_kept.len() phai la Err, khong panic");
    }

    #[test]
    fn anchor_in_the_middle_counts_exactly_the_segments_that_precede_it() {
        let blocks = vec![
            text_block("Cau mot day du chu. Cau hai cung day du chu.", true),
            image_block(),
            // Gap "\n\n" mo phong dung khoang trang GOC giua hai doan van tren mot trang
            // that -- xem doc-comment `text_block_with_gap`.
            text_block_with_gap("Cau ba dung sau anh.", true, "\n\n"),
        ];
        let effective_kept = vec![true, true, true];
        let full = join_kept_blocks(&blocks, &effective_kept);
        let segments = split_source_text(&full, "en");
        assert_eq!(segments.len(), 3, "fixture phai tach dung ba cau");

        let anchor = compute_anchor(&blocks, &effective_kept, 1, &full, &segments, &[], "en")
            .expect("anh giua Chuong phai tinh neo duoc");
        assert_eq!(anchor, 2, "hai cau dau phai dung TRUOC anh -- neo = 2");
    }

    #[test]
    fn a_prefix_that_does_not_actually_match_the_chapter_text_is_a_distinguishable_error_not_a_silent_zero() {
        let blocks = vec![text_block("Doan van bat ky.", true), image_block()];
        let effective_kept = vec![true, true];
        // `full_source_text` KHÔNG liên quan gì tới `blocks` -- mô phỏng đúng ca gia định
        // phân tách-theo-tiền-tố bị LỆCH (§Ask First spec 6.11): tiền tố tính được không thể
        // là một tiền tố thật của văn bản này.
        let unrelated_full_text = "Van ban hoan toan khac, khong lien quan gi toi blocks o tren.";
        let segments = split_source_text(unrelated_full_text, "en");

        let result = compute_anchor(&blocks, &effective_kept, 1, unrelated_full_text, &segments, &[], "en");
        assert!(
            result.is_err(),
            "tien to khong khop phai tra loi PHAN BIET DUOC, khong duoc lam tron thanh Ok(0)"
        );
    }

    /// B2 (vòng rà đối kháng 3 lớp) — một `segment` KHÔNG phải chuỗi con thật của `full_text`
    /// (vi phạm bất biến "segment là chuỗi con THẬT theo đúng thứ tự") phải trả lỗi PHÂN BIỆT
    /// ĐƯỢC, không được lặng lẽ dừng đếm sớm và trả về một `count` NHỎ HƠN sự thật.
    #[test]
    fn a_segment_that_is_not_a_real_substring_of_the_full_text_is_a_distinguishable_error_not_a_silent_undercount() {
        let full_text = "Cau mot that su. Cau hai that su.";
        let fake_segment =
            SplitSegment { text: "Cau nay khong he ton tai trong full_text".to_owned(), is_paragraph_end: false };
        let segments = vec![fake_segment];

        let result = segment_count_within_prefix(&segments, full_text, full_text.len());
        assert!(
            result.is_err(),
            "mot segment khong khop phai la mot LOI, khong duoc lam tron thanh Ok(0) hay dung dem som mot cach im lang"
        );
    }

    /// Đối chứng ÂM cho B2 — khi TẤT CẢ segment đều là chuỗi con thật, đúng thứ tự, hàm phải
    /// đếm đúng và KHÔNG bao giờ trả lỗi (lỗi chỉ nổ trên một vi phạm bất biến thật, không nổ
    /// oan trên đường hợp lệ).
    #[test]
    fn every_segment_being_a_real_substring_in_order_never_errors() {
        let full_text = "Cau mot that su. Cau hai that su. Cau ba dung cuoi.";
        let segments = split_source_text(full_text, "en");
        assert!(segments.len() >= 2, "fixture phai co it nhat hai cau");

        let result = segment_count_within_prefix(&segments, full_text, full_text.len());
        match result {
            Ok(n) => assert_eq!(n, segments.len(), "moi segment hop le phai duoc dem du"),
            Err(e) => panic!("khong duoc loi tren duong hop le: {}", e.detail),
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════════
    // C1 (vòng rà đối kháng 3 lớp) — neo với LUẬT LÀM SẠCH THẬT SỰ BẬT, không `&[]`
    // ═════════════════════════════════════════════════════════════════════════════════

    fn enabled_literal_rule(pattern: &str) -> CleanupRule {
        CleanupRule {
            tier: crate::core::cleanup::CleanupRuleTier::Global,
            id: 1,
            pattern: pattern.to_owned(),
            kind: crate::core::cleanup::CleanupRuleKind::Literal,
            enabled: true,
        }
    }

    /// C1 — một luật BẬT khớp chuỗi con NẰM TRỌN trong tiền tố (không vắt qua ranh giới ảnh)
    /// phải cho neo ĐÚNG, y hệt như khi chạy chuỗi thật với luật đó trên TOÀN Chương.
    #[test]
    fn a_real_enabled_cleanup_rule_matching_purely_inside_the_prefix_still_anchors_correctly() {
        let blocks = vec![
            text_block("Cau mot ban co xau ban.", true),
            image_block(),
            text_block_with_gap("Cau hai sau anh.", true, "\n\n"),
        ];
        let effective_kept = vec![true, true, true];
        let rules = vec![enabled_literal_rule("xau ")];

        // Chuỗi bảy bước AD-39 THẬT: bước 3 (cleanup) rồi bước 4 (normalize) trên TOÀN văn
        // bản ghép — đúng những gì `create_work` làm cho `chapter.source_text`.
        let joined = join_kept_blocks(&blocks, &effective_kept);
        let cleaned_full = crate::core::cleanup::apply(&joined, &rules).expect("cleanup toan van ban");
        let full_source_text = normalize::normalize(&cleaned_full.text, "en").text;
        assert!(
            !full_source_text.contains("xau"),
            "fixture phai that su bi luat lam sach xoa -- neu khong ca nay khong kiem gi ca"
        );

        let segments = split_source_text(&full_source_text, "en");
        let anchor = compute_anchor(&blocks, &effective_kept, 1, &full_source_text, &segments, &rules, "en")
            .expect("neo phai tinh duoc khi luat lam sach BAT va khop THUAN TRONG tien to");
        assert_eq!(anchor, 1, "dung mot cau dung TRUOC anh sau khi luat lam sach xoa \"xau \"");
    }

    /// C1, vế ⚠️ (§Ask First spec 6.11) — một luật BẬT khớp một chuỗi VẮT QUA đúng ranh giới
    /// nơi ảnh từng đứng (nối liền văn bản của khối TRƯỚC ảnh với khối SAU ảnh) phá giả định
    /// phân tách-theo-tiền-tố: tiền tố tính RIÊNG (không thấy phần "sau ảnh") không thể nào
    /// áp được cùng một phép xoá VẮT QUA đó. Đo THẬT bằng chính fixture này — ĐÂY LÀ KẾT QUẢ
    /// ĐO, không phải suy luận: [`compute_anchor`] phải trả lỗi PHÂN BIỆT ĐƯỢC (tự kiểm bắt
    /// đúng ca này), KHÔNG được âm thầm trả một neo sai.
    #[test]
    fn a_cleanup_rule_matching_across_the_image_boundary_is_caught_by_the_self_check_not_silently_wrong() {
        let blocks = vec![
            text_block("Cau mot ket thuc.", true),
            image_block(),
            text_block_with_gap("Cau hai bat dau.", true, "\n\n"),
        ];
        let effective_kept = vec![true, true, true];
        // Mẫu VẮT QUA đúng ranh giới ảnh trong văn bản ĐÃ GHÉP: "ket thuc.\n\nCau hai" chỉ tồn
        // tại khi CẢ HAI khối đứng cạnh nhau (đúng vị trí ảnh bị bỏ) -- tiền tố CÔ LẬP (chỉ
        // "Cau mot ket thuc.") không bao giờ chứa được chuỗi này.
        let rules = vec![enabled_literal_rule("ket thuc.\n\nCau hai")];

        let joined = join_kept_blocks(&blocks, &effective_kept);
        assert!(
            joined.contains("ket thuc.\n\nCau hai"),
            "fixture phai THAT SU noi lien hai khoi qua dung cho anh, khong thi ca nay khong kiem gi ca"
        );
        let cleaned_full = crate::core::cleanup::apply(&joined, &rules).expect("cleanup toan van ban");
        let full_source_text = normalize::normalize(&cleaned_full.text, "en").text;

        let segments = split_source_text(&full_source_text, "en");
        let result = compute_anchor(&blocks, &effective_kept, 1, &full_source_text, &segments, &rules, "en");

        // ĐO ĐƯỢC (không suy luận): kết quả THẬT của fixture này là gì?
        match &result {
            Err(e) => {
                // Đúng như §Ask First cảnh báo: giả định phân tách-theo-tiền-tố GÃY trên một
                // luật vắt qua ranh giới, và tự kiểm BẮT ĐƯỢC nó -- an toàn, không phải một
                // hồi quy. Ghi lại chi tiết lỗi để lần đọc sau không phải chạy lại mới biết.
                eprintln!("C1 (vong ra doi khang 3 lop) - do that: tu kiem BAT duoc luat vat qua ranh gioi: {}", e.detail);
            }
            Ok(n) => {
                panic!(
                    "C1 do duoc mot ket qua KHAC voi gia dinh §Ask First: tu kiem KHONG bat duoc \
                     luat vat qua ranh gioi, tra ve Ok({n}) thay vi Err -- day la mot phat hien \
                     THAT, dung sua phep tinh neo de het lech, bao lai cho Ice truoc khi di tiep"
                );
            }
        }
    }
}
