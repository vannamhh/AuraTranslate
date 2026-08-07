//! Lưới chống tái diễn lỗi Unihan — Story 1.10c, AC5.
//!
//! §Phát hiện ② của story đo được: trên 1.173 ký tự mà Unihan có âm VÀ en.wiktionary có
//! dữ liệu, **92,4%** giá trị `kVietnamese` cũ TRÙNG một âm NÔM đã gắn nhãn — tức cột
//! `han_viet` cũ gần như luôn mang âm Nôm, không phải âm Hán Việt. Module này viết chẩn
//! đoán đó thành một PHÉP KIỂM chạy lúc build: đối chiếu MỌI âm nạp vào
//! `dict_entry.han_viet` với tập âm Nôm đã gắn nhãn (`dict_entry.nom_reading`) của CÙNG
//! ký tự, và báo số ký tự đáng ngờ.

use std::collections::HashMap;

use rusqlite::Connection;

/// Ngưỡng phán quyết "`han_viet` đáng ngờ" — quá nửa số ký tự đối chiếu được trùng một
/// âm Nôm đã gắn nhãn thì nguồn ghi vào `han_viet` gần chắc đang ghi âm NÔM, không phải
/// âm Hán Việt thật (đúng hình dạng lỗi Unihan). Đặt ở **50%**: cao hơn nhiều xác suất
/// trùng ngẫu nhiên giữa hai tập âm đọc tiếng Việt của cùng một ký tự, và thấp hơn nhiều
/// con số đo được của chính lỗi Unihan (92,4%, §Phát hiện ②) — một khoảng cách đủ rộng
/// để ngưỡng không dao động theo nhiễu dữ liệu giữa các lượt cập nhật nguồn.
pub const SUSPICIOUS_RATIO_THRESHOLD: f64 = 0.5;

/// Kết quả một lượt đối chiếu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuspiciousHanViet {
    /// Số ký tự có ÍT NHẤT MỘT âm `han_viet` trùng một âm Nôm đã gắn nhãn của chính nó.
    pub suspicious: usize,
    /// Tổng số ký tự đối chiếu được (có ít nhất một âm `han_viet` khác rỗng sau khi tách).
    pub total_checked: usize,
}

impl SuspiciousHanViet {
    pub fn ratio(&self) -> f64 {
        if self.total_checked == 0 {
            0.0
        } else {
            self.suspicious as f64 / self.total_checked as f64
        }
    }

    /// AC5 — ngưỡng phán quyết có tên, không phải một số trần rải rác trong mã.
    pub fn exceeds_threshold(&self) -> bool {
        self.ratio() > SUSPICIOUS_RATIO_THRESHOLD
    }
}

/// Cắt một chuỗi âm đọc trên CẢ BA quy ước phân tách đã biết trong lược đồ này — `|`
/// (Thiều Chửu), `,` (Trần Văn Chánh, en-wiktionary-vi), khoảng trắng (Unihan cũ) — bỏ
/// mảnh rỗng, trim từng mảnh. Bẫy 4 của story: ba quy ước tồn tại song song trong dữ
/// liệu THẬT; hàm này chỉ dùng để SO SÁNH nội bộ của phép kiểm AC5, không ghi ngược
/// giá trị đã tách vào `.db` — Story 1.16 mới là nơi chuẩn hoá đường ĐỌC.
pub fn split_readings(raw: &str) -> Vec<&str> {
    raw.split(['|', ',', ' '])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

/// Review Findings — siết vế "nhãn Nôm ĐÃ XÁC NHẬN": chỉ giữ các âm `nom_reading` của
/// [`LABELED_NOM_SOURCE`] mà KHÔNG ĐỒNG THỜI được CHÍNH nguồn đó gắn nhãn
/// `han-viet-reading` cho CÙNG ký tự (`han_viet` của cùng hàng `dict_entry`).
///
/// 🔴 Không siết vế này, phép đối chiếu xuyên nguồn (`count_suspicious`) báo động giả
/// hàng loạt: đo THẬT trên `thieu-chuu` (nguồn Hán Việt chuẩn, dùng làm đối chứng ở
/// chính §Phát hiện ① của story) cho **369/582 = 63,4%** "đáng ngờ" — vượt ngưỡng — chỉ
/// vì `en-wiktionary-vi` tự gắn CẢ HAI nhãn cho cùng một âm rất thường xuyên (445/1.145
/// = 38,9%, thực tế ngôn ngữ học, không phải lỗi gán nhãn — xem doc-comment
/// `count_suspicious`). Một âm CŨNG được gắn `han-viet-reading` ⇒ không phải bằng
/// chứng "chỉ là âm Nôm" nữa, nên loại khỏi vế đối chứng — đúng tinh thần hai-trục
/// (HV/Nôm) của §Phát hiện ② thay vì một-trục.
pub fn nom_only_readings(han_viet: Option<&str>, nom_reading: Option<&str>) -> Option<String> {
    let nom_reading = nom_reading?;
    let han_viet_set: std::collections::HashSet<&str> = han_viet
        .map(split_readings)
        .unwrap_or_default()
        .into_iter()
        .collect();
    let filtered: Vec<&str> = split_readings(nom_reading)
        .into_iter()
        .filter(|r| !han_viet_set.contains(r))
        .collect();
    if filtered.is_empty() {
        None
    } else {
        Some(filtered.join(","))
    }
}

/// Đối chiếu MỌI `(headword, mã nguồn, chuỗi âm han_viet thô)` với tập âm Nôm đã gắn
/// nhãn của CÙNG headword — hàm THUẦN, không chạm database, để test được trên bất kỳ tập
/// dữ liệu nào (kể cả dữ liệu Unihan CŨ).
///
/// 🔴 **Chỉ đối chiếu XUYÊN NGUỒN** — một âm Nôm do CHÍNH nguồn đang được kiểm gắn nhãn
/// không tính vào phép so. Lý do: `en-wiktionary-vi` tự nó gắn CẢ HAI nhãn trên CÙNG
/// một ký tự khá thường xuyên (đo thật: 445/1.145 = 38,9% — một âm hợp lệ ở CẢ hai vai
/// là thực tế ngôn ngữ học bình thường, không phải lỗi gán nhãn). So một nguồn đã tự
/// phân biệt HV/Nôm với CHÍNH nó là một phép so vô nghĩa và sẽ luôn cho dương tính giả
/// cao. Phép kiểm này tồn tại để bắt một nguồn KHÁC (như `unihan` cũ) ghi vào `han_viet`
/// một giá trị mà nguồn ĐÃ CÓ NHÃN xác nhận là âm Nôm — xuyên nguồn mới là đúng câu hỏi.
pub fn count_suspicious<'a>(
    han_viet_rows: impl IntoIterator<Item = (&'a str, &'a str, &'a str)>,
    nom_rows: impl IntoIterator<Item = (&'a str, &'a str, &'a str)>,
) -> SuspiciousHanViet {
    let mut nom_by_char: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
    for (ch, source_code, raw) in nom_rows {
        // dict-build:allow .entry( — dựng CHỈ MỤC ĐỌC tạm thời để ĐỐI CHIẾU (AC5), giữ
        // NGUYÊN mã nguồn của từng âm; không ghi vào dict_entry/dict_sense và không
        // không hợp nhất Ý NGHĨA xuyên nguồn (AD-19) — mỗi phần tử vẫn mang `source_code`
        // riêng, dùng để LOẠI so sánh cùng nguồn ở vòng lặp dưới.
        let readings = nom_by_char.entry(ch).or_default();
        for reading in split_readings(raw) {
            readings.push((source_code, reading));
        }
    }

    // Review Findings — GỘP nhiều HÀNG cùng (ký tự, nguồn) thành MỘT tập âm đọc trước
    // khi đối chiếu. Một nguồn có thể góp nhiều `dict_entry` cho cùng headword (vd
    // `tran_van_chanh.rs` KHÔNG gộp theo headword — xem test
    // `duplicate_headword_lines_stay_as_separate_entries`); đếm mỗi HÀNG là một "ký tự
    // đối chiếu được" sẽ đếm trùng CÙNG một ký tự nhiều lần và làm lệch tỉ lệ AC5.
    let mut han_viet_by_char_source: HashMap<(&str, &str), std::collections::HashSet<&str>> =
        HashMap::new();
    for (ch, source_code, raw) in han_viet_rows {
        // dict-build:allow .entry( — GỘP (dedupe) readings của cùng (ký tự, nguồn), không
        // không hợp nhất xuyên nguồn (AD-19) — khoá map vẫn giữ NGUYÊN `source_code`.
        let readings = han_viet_by_char_source.entry((ch, source_code)).or_default();
        readings.extend(split_readings(raw));
    }

    let mut total_checked = 0usize;
    let mut suspicious = 0usize;
    for ((ch, source_code), readings) in han_viet_by_char_source {
        if readings.is_empty() {
            continue;
        }
        // Chỉ đối chiếu được nếu ký tự này có ÍT NHẤT MỘT âm Nôm gắn nhãn từ một nguồn
        // KHÁC — không có gì để so thì không đếm vào `total_checked` (đối tượng đo là
        // "tỉ lệ đáng ngờ TRONG SỐ những ký tự đối chiếu được", không phải trong số
        // MỌI ký tự có han_viet — đa số ký tự Unihan không có mặt trong en.wiktionary,
        // pha loãng tỉ lệ và che mất chẩn đoán nếu tính vào mẫu số).
        let Some(nom_candidates) = nom_by_char
            .get(ch)
            .map(|list| list.iter().filter(|(src, _)| *src != source_code))
        else {
            continue;
        };
        let mut nom_candidates = nom_candidates.peekable();
        if nom_candidates.peek().is_none() {
            continue;
        }
        total_checked += 1;
        if nom_candidates.any(|(_, nom_reading)| readings.contains(nom_reading)) {
            suspicious += 1;
        }
    }
    SuspiciousHanViet { suspicious, total_checked }
}

/// 🔴 Nguồn DUY NHẤT mà `nom_reading` của nó là một NHÃN TƯỜNG MINH (`tags:
/// ["nom-reading"]`, gắn bởi chính người biên soạn en.wiktionary) — đúng nguyên văn
/// Given clause của AC5: *"nhãn han-viet-reading/nom-reading CỦA EN.WIKTIONARY"*.
///
/// `unihan` CŨNG ghi `nom_reading` (Story 1.10c AC1 — `kVietnamese` đổi vai), nhưng giá
/// trị đó là một SUY DIỄN THỐNG KÊ của story này (§Phát hiện: 92,4% trùng, không phải
/// 100%) — Unicode chưa bao giờ tự gắn nhãn "đây là âm Nôm" cho từng giá trị riêng lẻ.
/// Dùng `nom_reading` của `unihan` làm "nhãn đã xác nhận" để đối chiếu sẽ tự tạo dương
/// tính giả cao (đo thật: 323/460 = 70,2% khi so `han_viet` của `en-wiktionary-vi` với
/// `nom_reading` của CHÍNH `unihan` — hai tập âm đọc CÙNG một ký tự tự nhiên trùng nhau
/// nhiều, không phải bằng chứng gán nhãn sai) — nên bị LOẠI khỏi vế "nhãn đã gắn".
const LABELED_NOM_SOURCE: &str = "en-wiktionary-vi";

/// Đọc trực tiếp từ một kết nối `.db` vừa dựng (còn mở, TRƯỚC `finalize::finish`) — mọi
/// hàng `dict_entry` có `han_viet` khác NULL (BẤT KỲ nguồn nào), đối chiếu với
/// `nom_reading` của [`LABELED_NOM_SOURCE`] (nhãn tường minh thật, xem doc-comment hằng
/// đó) — cộng `dict_source.code` của từng hàng (điều kiện đối chiếu XUYÊN NGUỒN của
/// `count_suspicious`).
///
/// Review Findings — `external_labeled_nom`: cặp `(headword, nom_reading)` của
/// [`LABELED_NOM_SOURCE`] nạp TỪ BÊN NGOÀI kết nối này, **đã lọc qua [`nom_only_readings`]
/// ở caller** (không tự lọc lại ở đây). Bắt buộc cho MỌI tệp gỡ rời: mỗi tệp gỡ rời chỉ
/// mang ĐÚNG MỘT `dict_source` (AD-10 — test
/// `each_detachable_file_holds_exactly_one_dict_source_row_with_its_own_code`), nên
/// truy vấn `nom_reading` bằng `s.code = ?` bên trong CHÍNH tệp đó luôn rỗng — không có
/// tham số này, phép kiểm AC5 cho ba tệp gỡ rời VĨNH VIỄN là `0/0`, bất kể `han_viet` của
/// chúng sai đến đâu. Caller cho `dict-core.db` (nơi `en-wiktionary-vi` đã tự có mặt)
/// truyền lát rỗng để tránh cộng trùng cùng một cặp hai lần.
pub fn count_suspicious_in_db(
    conn: &Connection,
    external_labeled_nom: &[(String, String)],
) -> rusqlite::Result<SuspiciousHanViet> {
    let mut hv_stmt = conn.prepare(
        "SELECT e.headword, s.code, e.han_viet \
         FROM dict_entry e JOIN dict_source s ON s.id = e.source_id \
         WHERE e.han_viet IS NOT NULL",
    )?;
    let hv_rows: Vec<(String, String, String)> = hv_stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;

    // Review Findings — cần CẢ `han_viet` của cùng hàng để loại âm "tự-trùng-vai" qua
    // `nom_only_readings` (xem doc-comment hàm đó — không siết vế này báo động giả
    // 63,4% trên Thiều Chửu).
    let mut nom_stmt = conn.prepare(
        "SELECT e.headword, s.code, e.nom_reading, e.han_viet \
         FROM dict_entry e JOIN dict_source s ON s.id = e.source_id \
         WHERE e.nom_reading IS NOT NULL AND s.code = ?1",
    )?;
    let mut nom_rows: Vec<(String, String, String)> = nom_stmt
        .query_map([LABELED_NOM_SOURCE], |r| {
            let headword: String = r.get(0)?;
            let source_code: String = r.get(1)?;
            let nom_reading: String = r.get(2)?;
            let han_viet: Option<String> = r.get(3)?;
            Ok((headword, source_code, nom_reading, han_viet))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .filter_map(|(headword, source_code, nom_reading, han_viet)| {
            let filtered = nom_only_readings(han_viet.as_deref(), Some(&nom_reading))?;
            Some((headword, source_code, filtered))
        })
        .collect();
    nom_rows.extend(
        external_labeled_nom
            .iter()
            .map(|(headword, reading)| (headword.clone(), LABELED_NOM_SOURCE.to_string(), reading.clone())),
    );

    let hv_refs: Vec<(&str, &str, &str)> = hv_rows
        .iter()
        .map(|(a, b, c)| (a.as_str(), b.as_str(), c.as_str()))
        .collect();
    let nom_refs: Vec<(&str, &str, &str)> = nom_rows
        .iter()
        .map(|(a, b, c)| (a.as_str(), b.as_str(), c.as_str()))
        .collect();

    Ok(count_suspicious(hv_refs, nom_refs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_readings_handles_all_three_conventions() {
        assert_eq!(split_readings("đinh|chênh"), vec!["đinh", "chênh"]);
        assert_eq!(split_readings("đáng, đương"), vec!["đáng", "đương"]);
        assert_eq!(split_readings("tợ tử"), vec!["tợ", "tử"]);
        assert_eq!(split_readings("bắc"), vec!["bắc"]);
        assert_eq!(split_readings(""), Vec::<&str>::new());
    }

    /// Ca thật `北`: `han_viet = "bắc"` VÀ `nom_reading` (gồm cả "bắc") đến từ CÙNG một
    /// nguồn (`en-wiktionary-vi` tự gắn cả hai nhãn cho cùng ký tự — thực tế ngôn ngữ
    /// học, đo thật 445/1.145 = 38,9%) ⇒ KHÔNG đáng ngờ, vì so một nguồn với chính nó là
    /// vô nghĩa (xem doc-comment `count_suspicious`).
    #[test]
    fn same_source_overlap_is_not_suspicious() {
        let hv = vec![("北", "en-wiktionary-vi", "bắc")];
        let nom = vec![("北", "en-wiktionary-vi", "bậc,bấc,bước,bắt,bắc,bác,bực")];
        let result = count_suspicious(hv, nom);
        assert_eq!(result.suspicious, 0);
        assert_eq!(
            result.total_checked, 0,
            "chỉ nguồn tự nhãn cả hai vai cho cùng ký tự ⇒ không có dữ liệu Nôm XUYÊN NGUỒN để so"
        );
        assert!(!result.exceeds_threshold());
    }

    /// Hình dạng lỗi Unihan tái tạo tối giản: một nguồn KHÁC (`unihan`) ghi `han_viet`
    /// trùng ĐÚNG một âm Nôm mà `en-wiktionary-vi` đã gắn nhãn cho CÙNG ký tự ⇒ đáng
    /// ngờ — đây chính là câu hỏi AC5 tồn tại để trả lời.
    #[test]
    fn a_han_viet_value_from_a_different_source_matching_a_labeled_nom_reading_is_suspicious() {
        let hv = vec![("繭", "unihan", "kén")];
        let nom = vec![("繭", "en-wiktionary-vi", "kén")];
        let result = count_suspicious(hv, nom);
        assert_eq!(result.suspicious, 1);
        assert_eq!(result.total_checked, 1);
    }

    /// 🔴 AC5 — phép kiểm phải ĐỎ ĐƯỢC: tái tạo tối giản hình dạng lỗi Unihan §Phát hiện
    /// ② (đa số ký tự có `kVietnamese`, nguồn `unihan`, trùng một âm Nôm nguồn
    /// `en-wiktionary-vi` đã gắn nhãn) phải vượt ngưỡng.
    #[test]
    fn a_majority_cross_source_overlap_crosses_the_suspicious_threshold() {
        let hv = vec![
            ("繭", "unihan", "kén"),
            ("抉", "unihan", "khoét"),
            ("蓉", "unihan", "rong"),
            ("死", "unihan", "tợ tử"),
        ];
        let nom = vec![
            ("繭", "en-wiktionary-vi", "kén"),
            ("抉", "en-wiktionary-vi", "khoét"),
            ("蓉", "en-wiktionary-vi", "rong"),
            ("死", "en-wiktionary-vi", "tợ"),
        ];
        let result = count_suspicious(hv, nom);
        assert_eq!(result.total_checked, 4);
        assert_eq!(result.suspicious, 4);
        assert!(result.exceeds_threshold(), "4/4 = 100% phải vượt ngưỡng 50%");
    }

    /// Đối chứng âm: ký tự KHÔNG có dữ liệu Nôm nào từ một nguồn khác ⇒ KHÔNG đối chiếu
    /// được — loại khỏi `total_checked` (không đếm là "đã kiểm, sạch"), vì không có gì
    /// để so là một câu khác với "đã so và không trùng".
    #[test]
    fn a_character_with_no_comparable_nom_data_is_excluded_from_total_checked() {
        let hv = vec![("永", "unihan", "vĩnh")];
        let nom: Vec<(&str, &str, &str)> = vec![];
        let result = count_suspicious(hv, nom);
        assert_eq!(result.suspicious, 0);
        assert_eq!(result.total_checked, 0, "không có dữ liệu Nôm để so ⇒ không đối chiếu được, không phải 'sạch'");
    }

    #[test]
    fn zero_total_checked_has_a_zero_ratio_not_a_division_panic() {
        let result = SuspiciousHanViet { suspicious: 0, total_checked: 0 };
        assert_eq!(result.ratio(), 0.0);
        assert!(!result.exceeds_threshold());
    }

    /// 🔴 Đo THẬT trên `--layer all` (2026-08-06): so `han_viet` của `en-wiktionary-vi`
    /// với `nom_reading` của CHÍNH `unihan` (thay vì chỉ nhãn tường minh) cho
    /// 323/460 = 70,2% — vượt ngưỡng dù KHÔNG một nguồn nào sai. `count_suspicious_in_db`
    /// phải LOẠI `nom_reading` của `unihan` khỏi vế "nhãn đã gắn" (chỉ dùng
    /// `en-wiktionary-vi`) — test này dựng một CSDL tối giản tái tạo đúng hình dạng đó
    /// và khoá lại: kết quả phải AN TOÀN.
    #[test]
    fn count_suspicious_in_db_ignores_unihans_own_nom_reading_as_a_label() {
        let conn = Connection::open_in_memory().unwrap();
        crate::insert::create_schema(&conn).unwrap();

        // Hai nguồn: `unihan` (nom_reading suy diễn, KHÔNG phải nhãn tường minh) và
        // `en-wiktionary-vi` (han_viet nhãn tường minh). Chọn giá trị TRÙNG nhau để mô
        // phỏng đúng hình dạng đo được thật (Unihan và en-wiktionary-vi tự nhiên trùng
        // âm khá thường xuyên, không phải bằng chứng gán nhãn sai).
        for (code, display) in [("unihan", "Unihan"), ("en-wiktionary-vi", "EN-WIKT-VI")] {
            conn.execute(
                "INSERT INTO dict_source (code, display_name, license_kind, license_text, attribution, source_version, source_url)
                 VALUES (?1, ?2, 'open', 'x', 'x', 'x', 'x')",
                rusqlite::params![code, display],
            )
            .unwrap();
        }
        let unihan_id: i64 = conn
            .query_row("SELECT id FROM dict_source WHERE code = 'unihan'", [], |r| r.get(0))
            .unwrap();
        let vi_id: i64 = conn
            .query_row("SELECT id FROM dict_source WHERE code = 'en-wiktionary-vi'", [], |r| r.get(0))
            .unwrap();

        conn.execute(
            "INSERT INTO dict_entry (source_id, lang, headword, nom_reading) VALUES (?1, 'zh', '永', 'vĩnh')",
            rusqlite::params![unihan_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO dict_entry (source_id, lang, headword, han_viet) VALUES (?1, 'zh', '永', 'vĩnh')",
            rusqlite::params![vi_id],
        )
        .unwrap();

        let result = count_suspicious_in_db(&conn, &[]).unwrap();
        assert_eq!(
            result.total_checked, 0,
            "'永' không có dữ liệu Nôm từ nguồn en-wiktionary-vi ⇒ không đối chiếu được"
        );
        assert_eq!(result.suspicious, 0);
    }

    /// Review Findings — lớp gỡ rời KHÔNG có hàng `en-wiktionary-vi` (AD-10: một tệp một
    /// `dict_source`), nên nhãn Nôm phải nạp TỪ BÊN NGOÀI mới đối chiếu được. Tái tạo
    /// đúng hình dạng lỗi Unihan (§Phát hiện ②) NHƯNG trong một CSDL một-nguồn — trước
    /// bản vá này, `total_checked` luôn là 0 ở đây bất kể `external_labeled_nom` nói gì.
    #[test]
    fn count_suspicious_in_db_uses_external_labels_for_a_single_source_detachable_db() {
        let conn = Connection::open_in_memory().unwrap();
        crate::insert::create_schema(&conn).unwrap();

        conn.execute(
            "INSERT INTO dict_source (code, display_name, license_kind, license_text, attribution, source_version, source_url)
             VALUES ('thieu-chuu', 'Thiều Chửu', 'open', 'x', 'x', 'x', 'x')",
            [],
        )
        .unwrap();
        let thieu_chuu_id: i64 = conn
            .query_row("SELECT id FROM dict_source WHERE code = 'thieu-chuu'", [], |r| r.get(0))
            .unwrap();
        conn.execute(
            "INSERT INTO dict_entry (source_id, lang, headword, han_viet) VALUES (?1, 'zh', '繭', 'kén')",
            rusqlite::params![thieu_chuu_id],
        )
        .unwrap();

        let no_external = count_suspicious_in_db(&conn, &[]).unwrap();
        assert_eq!(
            no_external.total_checked, 0,
            "không truyền external_labeled_nom ⇒ vẫn 0/0 như hành vi cũ (đối chứng âm)"
        );

        let external = vec![("繭".to_string(), "kén".to_string())];
        let with_external = count_suspicious_in_db(&conn, &external).unwrap();
        assert_eq!(with_external.total_checked, 1);
        assert_eq!(with_external.suspicious, 1, "'kén' trùng nhãn Nôm ngoài ⇒ đáng ngờ");
    }
}
