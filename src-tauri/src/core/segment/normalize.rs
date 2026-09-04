//! Chuẩn hoá xuống dòng và khoảng trắng — thân THẬT của bước 4 chuỗi AD-39 (Story 6.4,
//! FR124/FR125).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO BƯỚC 4 TỪNG LÀ NO-OP, VÀ CÁI GIÁ CỦA CHUYỆN ĐÓ — Design Notes spec 6.4
//! ─────────────────────────────────────────────────────────────────────────────
//! `split.rs:243-252` coi `\n`/`\r` là ranh giới CỨNG của segment; một dòng bị ngắt giữa
//! câu (tệp Windows với `\r\n`, hoặc một trình soạn thảo tự xuống dòng ở 80 cột) ra thành
//! HAI segment thay vì một. AD-4 đóng băng ranh giới đó xuống `.atproj` **vĩnh viễn** —
//! không đường mã nào tính lại lúc nạp. Module này cho thân thật vào bước 4 để hai bước
//! (chuẩn hoá rồi mới tách segment) chạy ĐÚNG THỨ TỰ, đúng chuỗi AD-39.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 HÀM THUẦN, KHÔNG BẢNG RIÊNG — §Always spec 6.4
//! ─────────────────────────────────────────────────────────────────────────────
//! [`normalize`] là một hàm THUẦN của văn bản đã giải mã: cùng đầu vào ⇒ cùng đầu ra, không
//! đọc đồng hồ/kho/cấu hình. Vị từ *"dòng này có kết thúc một câu không"* SỐNG trong
//! `split.rs` cạnh bảng kết câu ([`super::split::line_ends_a_sentence`]) — module này chỉ
//! GỌI nó, không chép lại `ZH_TERMINATORS`/`EN_TERMINATORS`/`TRAILING_CLOSERS`. Cùng luật
//! cho bảng nối theo ngôn ngữ nguồn ([`super::regroup::source_joiner`]). Hai chủ, hai bảng,
//! module này không có bảng nào của riêng nó — `tests/segment_normalize_boundary.rs`
//! cưỡng chế mệnh đề đó bằng cách quét 0 ký tự bảng kết câu trong chính tệp này.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA PHÉP, THỨ TỰ CỐ ĐỊNH — Design Notes spec 6.4 "Thứ tự ba phép là bắt buộc"
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. Thống nhất xuống dòng (`\r\n`/`\r` trần → `\n`) — TRƯỚC hai phép kia, nếu không
//!    `"A。\r\n\r\nB。"` đọc thành MỘT dòng trống mang `\r` bên trong, và phép đếm dòng
//!    trống lệch.
//! 2. Trim hai đầu MỖI dòng (đúng tập 25 điểm mã `White_Space` mà `str::trim()` cắt, cùng
//!    tập `schema.rs:305-323` đã khai triển tay) — TRƯỚC khi xét nối, nếu không một dòng
//!    kết bằng `"。   "` (dấu kết câu cộng khoảng trắng đuôi) không khớp bảng kết câu (ký
//!    tự CUỐI là khoảng trắng, không phải dấu kết câu) và bị nối OAN — đúng ca ma trận I/O
//!    `"\u{3000}\u{3000}他走了。   \n"`.
//! 3. Gộp dòng giữa câu + thu dòng trống liên tiếp về ĐÚNG MỘT — luật gộp: nối CHỈ KHI dòng
//!    kết thúc mà KHÔNG có dấu kết câu (đã bỏ dấu đóng ở đuôi), VÀ hai dòng nằm trong CÙNG
//!    một đoạn. Một dòng trống LUÔN LUÔN là ranh giới đoạn — không bao giờ nối qua nó.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ GIỚI HẠN PHẢI GHI RA — Design Notes spec 6.4 "Luật gộp dòng, và cái nó KHÔNG cứu được"
//! ─────────────────────────────────────────────────────────────────────────────
//! Một TIÊU ĐỀ không dấu chấm đứng riêng một dòng mà KHÔNG có dòng trống ở sau **vẫn bị
//! nối** vào câu kế — luật gộp không phân biệt được "dòng ngắt giữa câu" với "tiêu đề đứng
//! một mình". Không có ngưỡng độ dài hay luật "dòng ngắn thì đừng nối" nào được dựng để vá
//! chuyện này (cả hai đều là phỏng đoán chưa đo). Story 6.6 (tách Chương theo mẫu) bóc tiêu
//! đề ra khỏi thân TRƯỚC khi chuyện này xảy ra; tiêu đề nằm TRONG một Chương (không đứng
//! đầu) vẫn mở — nợ ghi ở `deferred-work.md`, chủ Story 6.6. Tầng xem trước
//! (`commands::project::preview_import_encoding`/`core::segment::encoding`) đếm SỐ DÒNG ĐÃ
//! NỐI để thiệt hại nhìn thấy được TRƯỚC khi xác nhận — đó là tấm lưới an toàn thật, không
//! phải luật này.

use super::regroup::source_joiner;
use super::split::{line_ends_a_sentence, LANG_CHINESE};

/// Kết quả một lượt chuẩn hoá — văn bản đã chuẩn hoá cộng HAI số đếm thiệt hại, để tầng xem
/// trước đo được TRƯỚC khi xác nhận (§Always spec 6.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalized {
    /// Văn bản sau ba phép — thứ được LƯU xuống `chapter.source_text` (AC4/AC5, hệ quả cấu
    /// trúc của việc tiêm ở bước 4 — xem Design Notes spec 6.4).
    pub text: String,
    /// Số LẦN hai dòng bị nối làm một — mỗi lần nối một run K dòng liên tiếp tính K-1 lần
    /// (một dòng bị nối vào dòng trước nó = một lần).
    pub joined_lines: usize,
    /// Số dòng trống bị XOÁ để đưa mỗi run dòng trống liên tiếp về ĐÚNG MỘT (không tính
    /// dòng trống CÒN LẠI sau khi thu — chỉ tính phần đã mất).
    pub blank_lines_removed: usize,
}

/// Chuẩn hoá TOÀN VĂN — bước 4 của chuỗi AD-39, thân thật (Story 6.4). Hàm THUẦN: cùng
/// `(text, source_lang)` luôn cho cùng [`Normalized`] (đối chứng "Bất động" của ma trận
/// I/O — chạy hai lần trên chính kết quả của lượt đầu phải cho lại y hệt).
///
/// `source_lang` chỉ quyết định HAI thứ, cả hai đều SỐNG ở chỗ khác: nhánh tiếng Trung của
/// [`line_ends_a_sentence`] (bảng ở `split.rs`) và [`source_joiner`] (bảng ở `regroup.rs`).
/// Module này không tự đọc `source_lang` để quyết bất kỳ điều gì khác.
#[must_use]
pub fn normalize(text: &str, source_lang: &str) -> Normalized {
    let chinese = source_lang == LANG_CHINESE;

    // Phép 1 — thống nhất xuống dòng. `str::replace` hai lượt là AN TOÀN: sau lượt thứ
    // nhất, không còn `"\r\n"` nào sống sót để lượt thứ hai (`\r` trần → `\n`) hiểu nhầm
    // thành một `\n` THỨ HAI (cả `\r` và `\n` đều là mã ASCII một byte, không bao giờ là
    // một phần của chuỗi byte UTF-8 đa byte — cắt/thay ở tầng byte không làm hỏng ký tự
    // nào khác).
    let unified = text.replace("\r\n", "\n").replace('\r', "\n");

    // Phép 2 — trim hai đầu MỖI dòng (str::trim cắt đúng tập 25 điểm mã White_Space, xem
    // doc-comment đầu tệp) — TRƯỚC khi phép 3 xét dòng nào kết thúc một câu.
    let trimmed_lines: Vec<&str> = unified.split('\n').map(str::trim).collect();

    // Phép 3 — nhóm thành từng ĐOẠN (ranh giới là một dòng trống trở lên), rồi trong TỪNG
    // đoạn: nối dòng mà dòng TRƯỚC không kết thúc một câu.
    let mut paragraphs: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut input_blank_lines = 0usize;
    for &line in &trimmed_lines {
        if line.is_empty() {
            input_blank_lines += 1;
            if !current.is_empty() {
                paragraphs.push(std::mem::take(&mut current));
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        paragraphs.push(current);
    }

    let joiner = source_joiner(source_lang);
    let mut joined_lines = 0usize;
    let paragraph_texts: Vec<String> = paragraphs
        .iter()
        .map(|para| {
            let mut acc = String::new();
            for (i, &line) in para.iter().enumerate() {
                if i == 0 {
                    acc.push_str(line);
                    continue;
                }
                if line_ends_a_sentence(para[i - 1], chinese) {
                    // Dòng TRƯỚC đã trọn câu — hai dòng Ở LẠI, cùng đoạn, phân tách bằng
                    // MỘT `\n` thật (ca ma trận I/O "Dòng đã trọn câu"/"Dấu kết + dấu
                    // đóng": lời thoại đứng riêng dòng trong cùng một đoạn không bị nối).
                    acc.push('\n');
                    acc.push_str(line);
                } else {
                    // Dòng TRƯỚC ngắt giữa câu — nối bằng dấu nối THEO NGÔN NGỮ NGUỒN
                    // (chủ [`source_joiner`], không phải một hằng thứ hai ở đây).
                    acc.push_str(joiner);
                    acc.push_str(line);
                    joined_lines += 1;
                }
            }
            acc
        })
        .collect();

    // Đúng MỘT dòng trống giữa hai đoạn liên tiếp — 0 dòng trống ở đầu/cuối văn bản (không
    // có đoạn nào đứng trước/sau để mà phân tách).
    let output_blank_lines = paragraph_texts.len().saturating_sub(1);
    let blank_lines_removed = input_blank_lines.saturating_sub(output_blank_lines);

    Normalized { text: paragraph_texts.join("\n\n"), joined_lines, blank_lines_removed }
}

/// Chuẩn hoá một CỬA SỔ — dùng bởi tầng xem trước bảng mã
/// (`core::segment::encoding::render_candidates`) để dựng bản chuẩn hoá của MỖI ứng viên
/// mà không phải đọc/giải mã toàn bộ Chương (Chương được phép nặng tới 100 MB,
/// `import::MAX_IMPORT_BYTES`).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO PHẢI BỎ DÒNG CUỐI — Design Notes spec 6.4 "Cắt cửa sổ ở đâu"
/// ─────────────────────────────────────────────────────────────────────────────
/// Chuẩn hoá KHÔNG bất động theo tiền tố: dòng CUỐI của một cửa sổ có nối hay không phụ
/// thuộc dòng SAU nó — byte NẰM NGOÀI cửa sổ, mà hàm này không có. `text.len() <= max_bytes`
/// (toàn văn bản đã lọt trong cửa sổ — Chương ngắn hơn cửa sổ) ⇒ KHÔNG có gì để cắt, gọi
/// thẳng [`normalize`]. Ngược lại: cắt TẠI RANH GIỚI DÒNG (bỏ mọi nội dung sau `\n`/`\r`
/// CUỐI CÙNG lọt trong `max_bytes` — đó là một dòng DANG DỞ, bị cắt cụt giữa chừng bởi
/// `max_bytes`), rồi BỎ dòng TRỌN VẸN cuối cùng còn lại (quyết định nối của chính nó vẫn
/// phụ thuộc dòng dang dở vừa bị cắt). Bỏ một dòng làm bản dựng NGẮN HƠN sự thật; GIỮ lại
/// làm nó SAI. Ngắn thì đọc được, sai thì không.
#[must_use]
pub fn normalize_window(text: &str, source_lang: &str, max_bytes: usize) -> Normalized {
    if text.len() <= max_bytes {
        return normalize(text, source_lang);
    }

    let mut boundary = max_bytes.min(text.len());
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    let truncated = &text[..boundary];

    let cut_at_last_break = match truncated.rfind(['\n', '\r']) {
        Some(idx) => &truncated[..idx],
        // Cửa sổ không chứa nổi MỘT ranh giới dòng nào — không dòng nào TRỌN VẸN để hiện.
        None => return Normalized { text: String::new(), joined_lines: 0, blank_lines_removed: 0 },
    };

    // `cut_at_last_break` giờ chỉ mang các dòng TRỌN VẸN (mỗi dòng được bao bởi `\n`/`\r`
    // CẢ HAI phía trong `truncated`) — trừ dòng CUỐI CÙNG của nó, quyết định nối của dòng
    // đó vẫn phụ thuộc dòng dang dở vừa bị cắt bỏ. Bỏ nốt dòng đó bằng cách cắt tới ranh
    // giới dòng KẾ TIẾP (lùi thêm một bước).
    let unified = cut_at_last_break.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<&str> = unified.split('\n').collect();
    lines.pop(); // dòng TRỌN VẸN cuối — quyết định nối của nó nằm ngoài cửa sổ, bỏ.

    normalize(&lines.join("\n"), source_lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts_eq(got: &Normalized, want: &str) {
        assert_eq!(got.text, want, "van ban chuan hoa lech ky vong: {got:?}");
    }

    // ── Ma trận I/O spec 6.4 ─────────────────────────────────────────────────────

    #[test]
    fn mid_sentence_break_in_english_joins_with_a_single_space() {
        let got = normalize("Han nhin ve phia\nngon nui xa.", "en");
        texts_eq(&got, "Han nhin ve phia ngon nui xa.");
        assert_eq!(got.joined_lines, 1);
    }

    #[test]
    fn mid_sentence_break_in_chinese_joins_with_no_separator() {
        let got = normalize("他转过头看向\n远处的山。", LANG_CHINESE);
        texts_eq(&got, "他转过头看向远处的山。");
        assert_eq!(got.joined_lines, 1);
    }

    #[test]
    fn a_line_that_already_ends_a_sentence_is_not_joined() {
        let got = normalize("他转过头。\n“谁？”", LANG_CHINESE);
        texts_eq(&got, "他转过头。\n“谁？”");
        assert_eq!(got.joined_lines, 0);
    }

    #[test]
    fn a_trailing_closer_is_skipped_before_checking_the_terminator() {
        let got = normalize("「走吧。」\n第二天。", LANG_CHINESE);
        texts_eq(&got, "「走吧。」\n第二天。");
        assert_eq!(got.joined_lines, 0);
    }

    #[test]
    fn a_run_of_blank_lines_collapses_to_exactly_one() {
        let got = normalize("A。\n\n\n\n\nB。", LANG_CHINESE);
        texts_eq(&got, "A。\n\nB。");
        assert_eq!(got.blank_lines_removed, 3);
    }

    #[test]
    fn a_blank_line_is_never_joined_across() {
        let got = normalize("Han nhin ve phia\n\nngon nui xa.", "en");
        texts_eq(&got, "Han nhin ve phia\n\nngon nui xa.");
        assert_eq!(got.joined_lines, 0);
    }

    #[test]
    fn crlf_and_bare_cr_both_become_lf() {
        assert_eq!(normalize("A。\r\nB。", LANG_CHINESE).text, "A。\nB。");
        assert_eq!(normalize("A。\rB。", LANG_CHINESE).text, "A。\nB。");
    }

    #[test]
    fn both_ends_of_every_line_are_trimmed() {
        let got = normalize("\u{3000}\u{3000}他走了。   \n", LANG_CHINESE);
        texts_eq(&got, "他走了。");
    }

    #[test]
    fn whitespace_only_input_normalizes_to_an_empty_string() {
        let got = normalize("\u{3000}\n \t \r\n ", LANG_CHINESE);
        texts_eq(&got, "");
        assert_eq!(got.joined_lines, 0);
    }

    #[test]
    fn normalizing_an_already_normalized_string_is_idempotent() {
        let once = normalize("A。\n\nB。 nguoi。\nhet cau.", LANG_CHINESE);
        let twice = normalize(&once.text, LANG_CHINESE);
        assert_eq!(once.text, twice.text, "chuan hoa lan hai phai cho lai Y HET");
        assert_eq!(twice.joined_lines, 0, "van ban da chuan hoa khong con gi de noi");
        assert_eq!(twice.blank_lines_removed, 0, "van ban da chuan hoa khong con dong trong thua");
    }

    // ── `normalize_window` ───────────────────────────────────────────────────────

    #[test]
    fn a_window_shorter_than_the_text_cuts_at_a_line_boundary_and_drops_the_last_line() {
        // Moi dong ket bang "." (dau ket cau) -- KHONG dong nao bi NOI, nen ca nay chi do
        // rieng phep cat cua so + bo dong cuoi, tach khoi luat noi dong (da co ca rieng).
        let text = "L1.\nL2.\nL3.\nL4.";
        // Cua so 13 byte lay tron "L1.\nL2.\nL3.\n" (12 byte) cong ky tu dau cua "L4.".
        let got = normalize_window(text, "en", 13);
        texts_eq(&got, "L1.\nL2.");
        assert_eq!(got.joined_lines, 0);
    }

    #[test]
    fn a_window_at_least_as_long_as_the_text_is_the_same_as_a_plain_normalize() {
        let text = "A。\nB。";
        let windowed = normalize_window(text, LANG_CHINESE, text.len());
        let plain = normalize(text, LANG_CHINESE);
        assert_eq!(windowed, plain, "cua so KHONG ngan hon van ban that -- phai giong normalize() thuong");
    }

    #[test]
    fn a_window_with_no_line_boundary_at_all_yields_an_empty_build() {
        let got = normalize_window("mot dong rat dai khong co xuong dong nao ca", "en", 5);
        texts_eq(&got, "");
    }
}
