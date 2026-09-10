//! Phân giải neo ảnh (`asset.anchor_after_segment_ord`) thành vị trí trong không gian **ID**,
//! cộng gắn `alt`/`caption` — Story 6.14, FR42/FR43.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! CƠ CHẾ — MỘT PHÉP LỌC CHO CẢ NEO LẪN VỆT ALT/CAPTION
//! ─────────────────────────────────────────────────────────────────────────────
//! `asset.anchor_after_segment_ord` sống trong không gian **`ord`** — một VỊ TRÍ. AD-3 cấm dữ
//! liệu gắn theo segment tham chiếu vị trí, và webview đã có luật riêng cấm đọc `ord`
//! (`segmentNavigation.ts:138-142`). [`resolve_chapter_images`] là chỗ DUY NHẤT phân giải
//! `ord` đó thành `id` của segment đứng NGAY TRƯỚC ảnh — dây chỉ chở `id`.
//!
//! [`after_segment_id`] tìm segment có `ord` LỚN NHẤT còn `<= anchor`, trong tập segment THOẢ
//! một vị từ "đủ điều kiện làm neo" — vị từ đó khác nhau giữa hai bề mặt, và khác biệt đó GIẢI
//! QUYẾT cả ba hàng biên trong §I/O Matrix của spec bằng ĐÚNG một phép lọc:
//!
//! - **Lưới** (`include_omitted = true`, `exclude_roles = false`): MỌI segment còn sống (đã
//!   lọc `retired_at`) đều đủ điều kiện — lưới không lọc cắt bỏ (FR44), và một hàng `alt`/
//!   `caption` vẫn là một ô nguyên văn bình thường để neo vào.
//! - **Chế độ đọc** (`include_omitted = false`, `exclude_roles = true`): chỉ segment CÒN
//!   TRONG BẢN DỊCH (chưa cắt bỏ) VÀ không mang vai mới đủ điều kiện — đó CHÍNH LÀ tập segment
//!   thật sự lên trang (`ChapterAsset`/`ReadingSegment`, sau [`strip_role_segments`]). Neo
//!   thẳng vào một segment không tồn tại trên trang là một tham chiếu treo mà webview không
//!   có cách nào theo.
//!
//! Anchor trỏ vào một `ord` đã VỀ HƯU/CẮT BỎ/mang VAI (tuỳ bề mặt) không có segment nào khớp
//! `ord == anchor` trong tập đủ điều kiện ⇒ phép lọc tự nhiên rơi về segment đủ điều kiện GẦN
//! NHẤT đứng trước — đúng "dời về câu còn sống liền trước" của I/O Matrix, không cần một
//! nhánh `if` riêng cho từng ca biên. Không segment nào đủ điều kiện có `ord <= anchor` (mọi
//! thứ trước đó đã về hưu/cắt bỏ/mang vai) ⇒ `None`, tức "về đầu Chương" — cùng ý nghĩa với
//! neo `0`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VỆT ALT/CAPTION LUÔN DÙNG NEO "CHUẨN" (LƯỚI), KHÔNG DÙNG NEO HIỂN THỊ CỦA TỪNG BỀ MẶT
//! ─────────────────────────────────────────────────────────────────────────────
//! `weave_chapter_segments` (Story 6.13) chèn `alt`/gắn `caption` NGAY SAU neo THẬT của ảnh
//! (đã DỜI cho đúng những ảnh đứng trước nó — xem doc-comment `WovenChapter::shifted_anchor_by_block`),
//! nên "vệt segment mang vai đứng ngay sau neo" phải đi tìm bắt đầu từ neo CHUẨN đó (tức phép
//! lọc của LƯỚI — `include_omitted = true, exclude_roles = false` — vì đó là phép lọc DUY NHẤT
//! khớp CHÍNH XÁC `ord` mà `weave_chapter_segments` đã dùng). Dùng neo HIỂN THỊ của Chế độ đọc
//! (vốn có thể NHẢY QUA cả một vệt vai của một ảnh KHÁC, đúng ca "hai ảnh cùng một neo") sẽ gán
//! nhầm alt/caption của ảnh này sang ảnh kia — xem ca hợp đồng
//! `two_images_sharing_an_anchor_each_keep_their_own_alt_text`.

use crate::commands::segment::ChapterSegment;

use super::role::SegmentRole;

/// Một hàng `asset` thô, như đọc thẳng từ bảng — chưa phân giải neo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawAsset {
    pub id: i64,
    pub file_name: String,
    pub source_url: Option<String>,
    pub anchor_after_segment_ord: i64,
}

/// Một ảnh đã phân giải vị trí + `alt`/`caption` — sẵn sàng đi ra dây (qua một DTO riêng của
/// từng bề mặt, `commands::segment::ChapterAsset`/`ReadingImage`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImage {
    pub asset_id: i64,
    pub file_name: String,
    pub source_url: Option<String>,
    /// `id` của segment đứng NGAY TRƯỚC ảnh, trong không gian phù hợp với BỀ MẶT đang gọi
    /// (xem tham số `include_omitted`/`exclude_roles` của [`resolve_chapter_images`]). `None`
    /// ⇒ ảnh đứng ở ĐẦU Chương, trước mọi nội dung của bề mặt đó.
    pub after_segment_id: Option<i64>,
    /// `target_text` của segment `role = Alt` đứng ngay sau neo CHUẨN — `None` khi ảnh không
    /// có segment `alt` nào (§Always: "ảnh trang trí", `alt=""` khi ra dây).
    pub alt_text: Option<String>,
    /// `target_text` của segment `role = Caption` đứng ngay sau neo CHUẨN — `None` khi ảnh
    /// không có segment `caption` nào. Chuỗi RỖNG (caption chưa dịch) vẫn được trả nguyên vẹn
    /// ở đây; "không chỗ trống khi chưa dịch" là việc của TẦNG HIỂN THỊ, không của hàm này.
    pub caption_text: Option<String>,
}

/// `id` của segment đủ điều kiện làm neo, có `ord` LỚN NHẤT còn `<= anchor` — xem doc-comment
/// đầu module cho vì sao MỘT phép lọc này giải quyết mọi ca biên của §I/O Matrix.
///
/// `anchor <= 0` ⇒ `None` ngay (neo `0` nghĩa là "trước segment đầu tiên", không có gì để tìm).
fn after_segment_id(
    segments: &[ChapterSegment],
    anchor: i64,
    include_omitted: bool,
    exclude_roles: bool,
) -> Option<i64> {
    if anchor <= 0 {
        return None;
    }
    segments
        .iter()
        .filter(|s| {
            s.ord <= anchor && (include_omitted || !s.is_omitted) && (!exclude_roles || s.role.is_none())
        })
        .max_by_key(|s| (s.ord, s.id))
        .map(|s| s.id)
}

/// `alt`/`caption` của MỘT ảnh — vệt segment mang VAI đứng NGAY SAU vị trí `after` (neo
/// CHUẨN, xem doc-comment đầu module), dừng ở segment `role = None` đầu tiên hoặc hết dãy.
/// AD-42 hứa "nhiều nhất một segment mỗi vai" cho một ảnh; hàm này giữ giá trị ĐẦU TIÊN gặp
/// của mỗi vai và bỏ qua phần dư — phòng thủ, không panic, không giả định hợp đồng đó đứng
/// vững mãi mãi.
fn attached_role_text(segments: &[ChapterSegment], after: Option<i64>) -> (Option<String>, Option<String>) {
    let start = match after {
        None => 0,
        Some(id) => match segments.iter().position(|s| s.id == id) {
            Some(idx) => idx + 1,
            // Phòng thủ: `after` không tìm thấy trong CHÍNH `segments` truyền vào (không thể
            // xảy ra trên đường sản phẩm -- `after_segment_id` chỉ trả `id` lấy từ đúng dãy
            // này) -- không vệt nào để đọc thay vì panic bằng `.unwrap()`/chỉ số âm.
            None => return (None, None),
        },
    };

    let mut alt_text = None;
    let mut caption_text = None;
    for seg in &segments[start..] {
        match seg.role.as_deref().and_then(SegmentRole::from_str) {
            Some(SegmentRole::Alt) if alt_text.is_none() => alt_text = Some(seg.target_text.clone()),
            Some(SegmentRole::Caption) if caption_text.is_none() => {
                caption_text = Some(seg.target_text.clone());
            }
            // 🔴 Gap lai MOT vai da dien ⇒ vet cua ANH NAY da het, va segment nay thuoc ve
            // ANH KE TIEP -- DUNG. Ban dau cho nay `{}` (bo qua, di tiep) voi ly le "phong thu
            // truoc mot AD-42 bi pha"; vong ra 2026-09-10 do duoc rang chinh no la mot loi:
            // Prose(1) Alt_A(2) Alt_B(3) Caption_B(4) ⇒ anh A muon dung caption cua anh B.
            // Xem `an_image_with_no_caption_does_not_borrow_the_next_images_caption`.
            Some(_) => break,
            None => break, // vet ket thuc.
        }
    }
    (alt_text, caption_text)
}

/// Phân giải MỌI ảnh của một Chương — hàm THUẦN, 0 điểm panic, không chạm đĩa.
///
/// # Tham số
/// - `segments`: dãy `ChapterSegment` của Chương, đã lọc `retired_at IS NULL` (khuôn
///   `select_chapter_segments`), `ORDER BY ord, id`.
/// - `assets`: dãy hàng `asset` thô của CHÍNH Chương đó.
/// - `include_omitted`/`exclude_roles`: quy tắc "đủ điều kiện làm neo" của BỀ MẶT đang gọi —
///   `(true, false)` cho Lưới, `(false, true)` cho Chế độ đọc. Xem doc-comment đầu module.
///
/// # Trả về
/// Dãy [`ResolvedImage`], sắp theo `asset.id` TĂNG DẦN — hai ảnh cùng một neo hiện theo đúng
/// thứ tự đó (§I/O Matrix "Hai ảnh cùng một neo").
pub fn resolve_chapter_images(
    segments: &[ChapterSegment],
    assets: &[RawAsset],
    include_omitted: bool,
    exclude_roles: bool,
) -> Vec<ResolvedImage> {
    let mut ordered: Vec<&RawAsset> = assets.iter().collect();
    ordered.sort_by_key(|a| a.id);

    ordered
        .into_iter()
        .map(|asset| {
            // Neo CHUẨN (lưới) -- MỘT lần, dùng cho vệt alt/caption bất kể bề mặt gọi là gì.
            // Xem doc-comment đầu module cho vì sao đây KHÔNG phải `display_after`.
            let canonical_after =
                after_segment_id(segments, asset.anchor_after_segment_ord, true, false);
            let display_after =
                after_segment_id(segments, asset.anchor_after_segment_ord, include_omitted, exclude_roles);
            let (alt_text, caption_text) = attached_role_text(segments, canonical_after);
            ResolvedImage {
                asset_id: asset.id,
                file_name: asset.file_name.clone(),
                source_url: asset.source_url.clone(),
                after_segment_id: display_after,
                alt_text,
                caption_text,
            }
        })
        .collect()
}

/// Bớt segment mang VAI (`role.is_some()`) khỏi từng đoạn ĐÃ GOM bởi
/// [`super::reading::paragraphs_in_translation`] — Story 6.14, một chốt lọc KHÁC hẳn "cắt bỏ"
/// (`super::omit`), tên riêng và doc-comment riêng theo đúng luật đã ghi ở đầu `omit.rs`.
///
/// Alt-text/caption không hiện như văn xuôi trên trang đọc — alt đi vào thuộc tính `alt` của
/// `<img>`, caption đi vào `<figcaption>` riêng dưới ảnh (Design Notes spec 6.14).
///
/// ⚠️ **Không cần một phép "chuyển cờ kết đoạn" thứ hai kiểu `omit.rs`.** Ranh giới đoạn
/// (`is_target_paragraph_end`) đã được `paragraphs_in_translation` dùng để QUYẾT nơi một đoạn
/// kết thúc TRƯỚC KHI hàm này chạy — kể cả khi segment mang cờ đó lại là một segment VAI. Hàm
/// này chỉ BỚT PHẦN TỬ khỏi một đoạn đã gom xong; nó không tính lại ranh giới, nên không có gì
/// để "chuyển" thêm một lần nữa.
///
/// Một đoạn mà MỌI segment đều mang vai (vd: `alt` rồi `caption` liền nhau tình cờ tạo thành
/// "một đoạn" theo `is_target_paragraph_end`) trở thành RỖNG sau khi bớt — nó KHÔNG lọt ra
/// ngoài, cùng luật "không bịa" mà `paragraphs_in_translation` đã giữ cho đoạn cắt bỏ hết.
pub fn strip_role_segments<'a>(
    paragraphs: Vec<Vec<&'a ChapterSegment>>,
) -> Vec<Vec<&'a ChapterSegment>> {
    paragraphs
        .into_iter()
        .map(|group| group.into_iter().filter(|s| s.role.is_none()).collect::<Vec<_>>())
        .filter(|group: &Vec<&ChapterSegment>| !group.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(id: i64, ord: i64, role: Option<&str>, is_omitted: bool, is_target_paragraph_end: bool) -> ChapterSegment {
        ChapterSegment {
            id,
            ord,
            source_text: format!("src {id}"),
            target_text: format!("tgt {id}"),
            is_paragraph_end: is_target_paragraph_end,
            retired_at: None,
            status: "draft".to_owned(),
            is_omitted,
            is_target_paragraph_end,
            role: role.map(str::to_owned),
        }
    }

    fn asset(id: i64, anchor: i64) -> RawAsset {
        RawAsset { id, file_name: format!("{id}.jpg"), source_url: None, anchor_after_segment_ord: anchor }
    }

    // ── §I/O Matrix, hàng theo hàng ──────────────────────────────────────────────────

    #[test]
    fn an_image_between_two_sentences_shows_after_sentence_k_on_both_surfaces() {
        let segments = vec![seg(1, 1, None, false, false), seg(2, 2, None, false, false)];
        let assets = vec![asset(100, 1)];

        let grid = resolve_chapter_images(&segments, &assets, true, false);
        let reading = resolve_chapter_images(&segments, &assets, false, true);
        assert_eq!(grid[0].after_segment_id, Some(1));
        assert_eq!(reading[0].after_segment_id, Some(1));
    }

    #[test]
    fn anchor_zero_resolves_to_none_on_both_surfaces() {
        let segments = vec![seg(1, 1, None, false, false)];
        let assets = vec![asset(100, 0)];

        let grid = resolve_chapter_images(&segments, &assets, true, false);
        let reading = resolve_chapter_images(&segments, &assets, false, true);
        assert_eq!(grid[0].after_segment_id, None);
        assert_eq!(reading[0].after_segment_id, None);
    }

    #[test]
    fn an_anchor_pointing_at_a_cut_sentence_stays_put_on_the_grid_but_moves_back_for_reading() {
        // Cau 1 (song) -- cau 2 (CAT BO) -- neo = 2.
        let segments = vec![
            seg(1, 1, None, false, false),
            seg(2, 2, None, true, false), // is_omitted = true
        ];
        let assets = vec![asset(100, 2)];

        let grid = resolve_chapter_images(&segments, &assets, true, false);
        assert_eq!(grid[0].after_segment_id, Some(2), "luoi khong loc cat bo");

        let reading = resolve_chapter_images(&segments, &assets, false, true);
        assert_eq!(reading[0].after_segment_id, Some(1), "doc doi ve cau con song lien truoc");
    }

    #[test]
    fn an_anchor_whose_ord_no_longer_exists_falls_back_to_the_nearest_surviving_ord() {
        // `segments` da loc `retired_at` o tang goi -- ord=2 "bien mat" khoi day nay (ve huu).
        let segments = vec![seg(1, 1, None, false, false), seg(3, 3, None, false, false)];
        let assets = vec![asset(100, 2)];

        let grid = resolve_chapter_images(&segments, &assets, true, false);
        assert_eq!(grid[0].after_segment_id, Some(1), "khong con ord=2 -- doi ve ord=1");
    }

    #[test]
    fn an_anchor_with_nothing_surviving_before_it_falls_back_to_the_start_of_the_chapter() {
        let segments = vec![seg(5, 5, None, false, false)];
        let assets = vec![asset(100, 2)];

        let resolved = resolve_chapter_images(&segments, &assets, true, false);
        assert_eq!(resolved[0].after_segment_id, None, "khong segment nao <= anchor -- ve dau Chuong");
    }

    #[test]
    fn two_images_sharing_an_anchor_show_both_ordered_by_ascending_asset_id() {
        let segments = vec![seg(1, 1, None, false, false)];
        let assets = vec![asset(200, 1), asset(100, 1)]; // co y dao thu tu id trong input.

        let resolved = resolve_chapter_images(&segments, &assets, true, false);
        assert_eq!(resolved.iter().map(|r| r.asset_id).collect::<Vec<_>>(), vec![100, 200]);
        assert!(resolved.iter().all(|r| r.after_segment_id == Some(1)));
    }

    #[test]
    fn an_image_without_a_caption_segment_gets_no_caption_block() {
        let segments = vec![seg(1, 1, None, false, false), seg(2, 2, Some("alt"), false, false)];
        let assets = vec![asset(100, 1)];

        let resolved = resolve_chapter_images(&segments, &assets, true, false);
        assert_eq!(resolved[0].alt_text.as_deref(), Some("tgt 2"));
        assert_eq!(resolved[0].caption_text, None);
    }

    #[test]
    fn an_image_without_an_alt_segment_yields_no_alt_text() {
        let segments = vec![seg(1, 1, None, false, false), seg(2, 2, Some("caption"), false, false)];
        let assets = vec![asset(100, 1)];

        let resolved = resolve_chapter_images(&segments, &assets, true, false);
        assert_eq!(resolved[0].alt_text, None);
        assert_eq!(resolved[0].caption_text.as_deref(), Some("tgt 2"));
    }

    // ── Ca chống hồi quy đặc thù thiết kế: vệt luôn dùng neo CHUẨN ─────────────────────

    #[test]
    fn two_images_sharing_an_anchor_each_keep_their_own_alt_text() {
        // Prose(ord1) -- Image A & B cung neo goc, ca hai co alt. Sau khi det (Story 6.13),
        // dung DB: Prose(1) Alt_A(2,role=alt) Alt_B(3,role=alt). Asset A.anchor=1 (chua dich
        // chuyen), asset B.anchor=2 (da dich chuyen vi Alt_A da chen truoc no) -- dung khuon
        // `weave_chapter_segments` that.
        let segments = vec![
            seg(1, 1, None, false, false),
            seg(2, 2, Some("alt"), false, false),
            seg(3, 3, Some("alt"), false, false),
        ];
        let assets = vec![asset(10, 1), asset(20, 2)];

        let resolved = resolve_chapter_images(&segments, &assets, true, false);
        let by_id = |id: i64| resolved.iter().find(|r| r.asset_id == id).unwrap();
        assert_eq!(by_id(10).alt_text.as_deref(), Some("tgt 2"), "anh A giu dung alt cua no");
        assert_eq!(by_id(20).alt_text.as_deref(), Some("tgt 3"), "anh B giu dung alt cua no, khong lay lai cua A");
    }

    #[test]
    fn reading_display_anchor_does_not_leak_a_role_segments_streak_to_the_wrong_image() {
        // Cung du lieu tren, nhung xet NEO HIEN THI cho Che do doc: ca hai anh phai doi ve
        // Prose(1) (vi Alt_A/Alt_B khong hien tren trang doc) -- NHUNG alt cua tung anh van
        // phai dung rieng (kiem lai o day thay vi tin cay ca test tren).
        let segments = vec![
            seg(1, 1, None, false, false),
            seg(2, 2, Some("alt"), false, false),
            seg(3, 3, Some("alt"), false, false),
        ];
        let assets = vec![asset(10, 1), asset(20, 2)];

        let resolved = resolve_chapter_images(&segments, &assets, false, true);
        assert!(resolved.iter().all(|r| r.after_segment_id == Some(1)));
        let by_id = |id: i64| resolved.iter().find(|r| r.asset_id == id).unwrap();
        assert_eq!(by_id(10).alt_text.as_deref(), Some("tgt 2"));
        assert_eq!(by_id(20).alt_text.as_deref(), Some("tgt 3"));
    }

    #[test]
    fn an_image_with_no_caption_does_not_borrow_the_next_images_caption() {
        // Prose(1) -- Anh A (co alt, KHONG caption) -- Anh B (co alt VA caption).
        // Sau khi det (6.13): Prose(1) Alt_A(2) Alt_B(3) Caption_B(4);
        // A.anchor = 1, B.anchor = 2 (da dich chuyen vi Alt_A chen truoc no).
        let segments = vec![
            seg(1, 1, None, false, false),
            seg(2, 2, Some("alt"), false, false),
            seg(3, 3, Some("alt"), false, false),
            seg(4, 4, Some("caption"), false, false),
        ];
        let assets = vec![asset(10, 1), asset(20, 2)];

        let resolved = resolve_chapter_images(&segments, &assets, true, false);
        let by_id = |id: i64| resolved.iter().find(|r| r.asset_id == id).unwrap();

        assert_eq!(by_id(10).alt_text.as_deref(), Some("tgt 2"));
        assert_eq!(
            by_id(10).caption_text,
            None,
            "anh A KHONG co caption cua rieng no -- khong duoc muon caption cua anh B"
        );
        assert_eq!(by_id(20).alt_text.as_deref(), Some("tgt 3"));
        assert_eq!(by_id(20).caption_text.as_deref(), Some("tgt 4"));
    }

    // ── strip_role_segments ────────────────────────────────────────────────────────

    #[test]
    fn strip_role_segments_drops_role_bearing_elements_but_keeps_the_paragraph() {
        let prose1 = seg(1, 1, None, false, false);
        let alt = seg(2, 2, Some("alt"), false, false);
        let prose2 = seg(3, 3, None, false, true);
        let paragraphs = vec![vec![&prose1, &alt, &prose2]];

        let stripped = strip_role_segments(paragraphs);
        assert_eq!(stripped.len(), 1);
        assert_eq!(stripped[0].iter().map(|s| s.id).collect::<Vec<_>>(), vec![1, 3]);
    }

    #[test]
    fn a_paragraph_made_entirely_of_role_segments_disappears_instead_of_rendering_empty() {
        let alt = seg(1, 1, Some("alt"), false, false);
        let caption = seg(2, 2, Some("caption"), false, true);
        let prose = seg(3, 3, None, false, true);
        let paragraphs = vec![vec![&alt, &caption], vec![&prose]];

        let stripped = strip_role_segments(paragraphs);
        assert_eq!(stripped.len(), 1, "doan toan vai phai bien mat, khong lot ra rong: {stripped:?}");
        assert_eq!(stripped[0][0].id, 3);
    }
}
