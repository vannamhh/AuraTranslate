//! Hành vi của bộ lọc "cần xem" (Story 6.10, FR132) ở tầng LỆNH/DÂY — đầu-cuối qua
//! [`cleanup_and_chapters_preview_for`], không chỉ ở tầng hàm thuần `core::segment::review`
//! (canh riêng, sâu hơn, ở `core/segment/review.rs::tests`).
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `cleanup_contract.rs`/`glossary_contract.rs` — một tệp,
//! một mối quan tâm.

use auratranslate_lib::commands::project::{cleanup_and_chapters_preview_for, ReviewCauseWire};
use auratranslate_lib::core::cleanup::{CleanupRule, CleanupRuleKind, CleanupRuleTier};
use auratranslate_lib::core::segment::chapterpattern::ChapterPattern;
use auratranslate_lib::core::segment::pipeline::{ChapterInput, PipelineShape};

fn chapters_of(texts: &[&str]) -> PipelineShape {
    PipelineShape::Chapters(texts.iter().map(|t| ChapterInput::AlreadyText((*t).to_owned())).collect())
}

/// Mười Chương THẬT (đường `Chapters`, mỗi Chương có `length` riêng của chính nó) — một
/// Chương RẤT NGẮN so với chín Chương còn lại phải bị gắn cờ `ShortLength`.
#[test]
fn a_very_short_chapter_among_ten_real_ones_is_flagged_short_length() {
    // Chín Chương "bình thường" phải TẢN RỘNG thật (không chỉ chín lần một fixture chép lại)
    // — một Chương ngắn đứng lẫn giữa CHÍN giá trị giống hệt nhau sẽ bị chính hàng rào "nuốt"
    // (Q1/Q3 rơi vào đúng giá trị lặp lại, IQR = 0) — đúng bẫy đã bắt được ở
    // `core/segment/review.rs::tests::a_degenerate_iqr_excludes_only_that_signal`.
    let mut texts: Vec<String> = (0..9).map(|i| "a".repeat(500 + i * 20)).collect();
    texts.push("x".repeat(5)); // Chương thứ mười -- cực ngắn so với chín Chương kia.
    let shape = chapters_of(&texts.iter().map(String::as_str).collect::<Vec<_>>());

    let (_cleanup, chapters_wire, _blocks) = cleanup_and_chapters_preview_for(
        shape,
        encoding_rs::UTF_8,
        None,
        "",
        "en",
        &[],
        false,
        false,
        &[],
        0,
        0, // broken_item_count -- duong tay/dan tay, khong co muc hong.
    );

    assert_eq!(chapters_wire.chapter_count, 10);
    assert!(chapters_wire.any_signal_participated, "10 Chuong voi length tan rong phai co hang rao");
    let last = chapters_wire.chapters.last().expect("Chuong thu muoi");
    assert!(last.needs_review, "Chuong cuc ngan phai la CAN XEM");
    assert!(last.review_causes.contains(&ReviewCauseWire::ShortLength));
    assert_eq!(chapters_wire.needs_review_count, 1);
    assert_eq!(chapters_wire.clean_count, 9);
    assert_eq!(chapters_wire.broken_item_count, 0);
}

/// **THÊM (vòng rà đối kháng bước 4, 2026-09-08)** — vế SONG SONG của ca ngay trên, ở tầng
/// DÂY thật (`core/segment/review.rs::tests::each_signal_only_checks_the_fence_side_that_makes_sense_for_it`
/// đã canh ở tầng hàm thuần; ca này canh lại đúng mệnh đề đó qua `cleanup_and_chapters_preview_for`
/// thật). Một Chương RẤT DÀI so với chín Chương còn lại KHÔNG được gắn cờ `ShortLength` —
/// `length` chỉ xét hàng rào DƯỚI, một Chương dài bất thường không phải dấu hiệu lỗi nhập.
#[test]
fn a_very_long_chapter_among_ten_real_ones_is_not_flagged_short_length() {
    let mut texts: Vec<String> = (0..9).map(|i| "a".repeat(500 + i * 20)).collect();
    texts.push("a".repeat(50000)); // Chương thứ mười -- rat DAI so voi chin Chuong kia.
    let shape = chapters_of(&texts.iter().map(String::as_str).collect::<Vec<_>>());

    let (_cleanup, chapters_wire, _blocks) = cleanup_and_chapters_preview_for(
        shape,
        encoding_rs::UTF_8,
        None,
        "",
        "en",
        &[],
        false,
        false,
        &[],
        0,
        0, // broken_item_count -- duong tay/dan tay, khong co muc hong.
    );

    assert_eq!(chapters_wire.chapter_count, 10);
    assert!(chapters_wire.any_signal_participated, "10 Chuong voi length tan rong phai co hang rao");
    let last = chapters_wire.chapters.last().expect("Chuong thu muoi");
    assert!(!last.needs_review, "Chuong DAI bat thuong KHONG phai dau hieu loi nhap");
    assert!(
        !last.review_causes.contains(&ReviewCauseWire::ShortLength),
        "length chi xet hang rao DUOI -- mot Chuong dai hon khong duoc gan co ShortLength"
    );
    assert!(last.review_causes.is_empty());
    assert_eq!(chapters_wire.needs_review_count, 0);
    assert_eq!(chapters_wire.clean_count, 10);
}

/// Dưới bốn Chương — KHÔNG tín hiệu nào tham gia, `any_signal_participated == false`, và
/// KHÔNG Chương nào bị phán oan là "cần xem" hay "sạch" theo một hàng rào không tồn tại.
#[test]
fn fewer_than_four_chapters_makes_no_signal_participate_at_the_wire_level() {
    let shape = chapters_of(&["mot", "hai ba", "bon nam sau"]);

    let (_cleanup, chapters_wire, _blocks) = cleanup_and_chapters_preview_for(
        shape, encoding_rs::UTF_8, None, "", "en", &[], false, false, &[], 0, 0,
    );

    assert_eq!(chapters_wire.chapter_count, 3);
    assert!(
        !chapters_wire.any_signal_participated,
        "N = 3 phai bao 'chua du Chuong de so', khong tinh hieu nao duoc tham gia"
    );
    for chapter in &chapters_wire.chapters {
        assert!(!chapter.needs_review, "khong tin hieu nao tham gia thi khong Chuong nao bi phan");
    }
}

/// `broken_item_count` cộng vào `needs_review_count`, KHÔNG BAO GIỜ vào `clean_count` — vế
/// link hỏng của §Always spec 6.10 ("hai con số do RUST cộng, kể cả vế link hỏng").
#[test]
fn broken_item_count_is_added_by_rust_into_needs_review_never_into_clean() {
    let long = "b".repeat(500);
    let texts: Vec<String> = (0..6).map(|_| long.clone()).collect();
    let shape = chapters_of(&texts.iter().map(String::as_str).collect::<Vec<_>>());

    let (_cleanup, chapters_wire, _blocks) = cleanup_and_chapters_preview_for(
        shape, encoding_rs::UTF_8, None, "", "en", &[], false, false, &[], 0,
        3, // ba muc URL hong cua CA lot nhap.
    );

    assert_eq!(chapters_wire.chapter_count, 6);
    assert_eq!(chapters_wire.broken_item_count, 3);
    // Sau Chuong deu dai bang nhau -- length suy bien, khong tin hieu nao gan co that -- nen
    // ca sau deu SACH, va needs_review_count CHI mang dung 3 (tu link hong).
    assert_eq!(chapters_wire.clean_count, 6);
    assert_eq!(chapters_wire.needs_review_count, 3, "3 link hong phai duoc CONG vao ve can xem");
}

/// I/O Matrix spec 6.10, hàng *"Tệp một khối + mẫu phân tách, 10 Chương"* — trên đường `Blob`
/// + `chapter_pattern`, CHỈ Chương `ord = 1` có báo cáo làm sạch THẬT (bước 3 chạy TRƯỚC bước
/// 5 trên MỘT đơn vị); `joined_line_count` là `None` cho CẢ MƯỜI (kể cả `ord = 1` — bước 4
/// cũng chạy trên toàn blob, xem `core::segment::pipeline::Flow::joined_line_counts`). Hai
/// tín hiệu ấy KHÔNG ĐỦ DỮ LIỆU (dưới bốn giá trị đo được — chỉ MỘT Chương có `cleanup`, KHÔNG
/// Chương nào có `joined_line`) ⇒ KHÔNG THAM GIA — không Chương nào bị hai tín hiệu này gán cờ
/// (`HighCleanupMatches`/`HighJoinedLines`/`NotMeasured` không xuất hiện ở BẤT KỲ Chương nào).
/// Đây chính là đối chứng đỏ ① của §Verification spec 6.10 ở tầng DÂY: nếu `cleanup_match_count`
/// bị mặc định hoá `None → Some(0)` (`.unwrap_or(0)` trên `usize` trần) TRƯỚC khi vào
/// `core::segment::review::classify`, chín Chương còn lại (giá trị giả `0`) sẽ đủ bốn để hàng
/// rào tồn tại — và một luật THẬT SỰ có khớp trên `ord = 1` (số dương) làm `IQR > 0` — khiến
/// chín Chương CHƯA AI ĐO bị đếm là "đo được và sạch" trong `clean_count`, đúng lỗi mà AC
/// 2026-09-08 tồn tại để chặn.
#[test]
fn a_blob_with_a_chapter_pattern_excludes_the_two_signals_that_only_the_first_chapter_measures() {
    let rule = CleanupRule {
        tier: CleanupRuleTier::Global,
        id: 1,
        pattern: "QUANGCAO".to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    };
    // Muoi Chuong, moi Chuong bat dau bang "==="; CHI van ban o Chuong dau (truoc khop dau
    // tien khong tinh -- ca nay bat dau ngay bang mau) mang "QUANGCAO" that.
    let mut text = String::new();
    for i in 0..10 {
        text.push_str("===\n");
        if i == 0 {
            text.push_str("QUANGCAO mot. QUANGCAO hai. noi dung That Su.\n");
        } else {
            text.push_str(&format!("noi dung Chuong so {i}, khong co gi de xoa ca.\n"));
        }
    }
    let shape = PipelineShape::Blob(ChapterInput::AlreadyText(text));
    let pattern = ChapterPattern::literal("===");

    let (_cleanup, chapters_wire, _blocks) = cleanup_and_chapters_preview_for(
        shape,
        encoding_rs::UTF_8,
        Some(&pattern),
        "",
        "en",
        &[rule],
        false,
        false,
        &[],
        0,
        0,
    );

    assert_eq!(chapters_wire.chapter_count, 10);
    assert_eq!(chapters_wire.chapters[0].cleanup_match_count, Some(2), "tien de: ord=1 co bao cao THAT");
    for (i, chapter) in chapters_wire.chapters.iter().enumerate() {
        assert_eq!(chapter.joined_line_count_in_chapter, None, "Chuong {i}: joined_line luon None tren Blob");
        if i != 0 {
            assert_eq!(chapter.cleanup_match_count, None, "Chuong {i}: khong phai ord=1, khong co bao cao that");
        }
        assert!(
            !chapter.review_causes.contains(&ReviewCauseWire::HighCleanupMatches),
            "Chuong {i}: tin hieu cleanup KHONG DU DU LIEU (chi 1 gia tri do duoc), khong duoc gan co"
        );
        assert!(
            !chapter.review_causes.contains(&ReviewCauseWire::HighJoinedLines),
            "Chuong {i}: tin hieu joined_line TOAN None, khong duoc gan co"
        );
        assert!(
            !chapter.review_causes.contains(&ReviewCauseWire::NotMeasured),
            "Chuong {i}: NotMeasured chi xuat hien khi hang rao TON TAI -- ca hai tin hieu nay deu KHONG ton tai o day"
        );
    }
}
