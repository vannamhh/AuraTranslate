//! Thuật toán quét ứng viên khi nhập tài liệu — Story 3.5, FR47.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 MODULE THUẦN — KHÔNG CHẠM DB, KHÔNG BIẾT `Store` TỒN TẠI
//! ─────────────────────────────────────────────────────────────────────────────
//! Cùng lớp `core::matching`: mọi thứ ở đây là hàm trên `&str`/`char`, không `use
//! crate::core::store`, không `use crate::ports`. Đây là điều kiện để
//! `tests/glossary_scan_contract.rs` kiểm được TẤT ĐỊNH mà không cần dựng một `Store` nào.
//! 🔵 CẬP NHẬT 2026-08-22 — vị từ bool chỉ còn ở wrapper tương thích [`scan_candidates`].
//! Đường sản phẩm đi qua [`scan_candidates_controlled`] với [`DictionaryProbe`] ba trạng
//! thái, vì một layer lỗi không được phép bị ép thành “không có trong từ điển”. Chỗ gọi thật
//! (`commands::project`) tiêm closure gọi [`crate::core::dict::lookup_grouped`]; test tiêm
//! closure tất định.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 LỌC THEO TẦN SUẤT TRƯỚC, TRA TỪ ĐIỂN SAU — thứ tự là kiến trúc, không phải tối ưu sớm
//! ─────────────────────────────────────────────────────────────────────────────
//! [`ports::dict_source::DictionarySource`] không có một vị từ tra-có/không rẻ (không
//! `exists()`), và [`crate::core::dict::lookup_grouped`] lặp qua **mọi** lớp đang mở, không
//! tắt sớm — số đo pha 1 duy nhất trong kho là **p95 7.324 ms** cho MỘT lượt tra một lớp.
//! [`scan_candidates_controlled`] gọi probe **chỉ** cho các chuỗi đã qua bộ lọc tần suất (+
//! dedup lồng, cho `Zh`) — hàng trăm chuỗi cho một Chương thật, không hàng trăm nghìn. Đảo
//! thứ tự (tra trước, đếm sau) là biến một lượt quét thành hàng chục nghìn lượt tra
//! `lookup_grouped`.
//!
//! ⚠️ Mọi chuỗi trong `src-tauri/src/**` viết KHÔNG DẤU; doc-comment có dấu là hợp lệ.

use std::collections::HashMap;

use crate::core::matching::{MatchLang, ngrams, tokenize};

use super::surnames::TRADITIONAL_SURNAME_ALIASES;

/// Độ dài n-gram (KÝ TỰ, nhánh `Zh`) mà lượt quét sinh ra — 2 tới 4. Khớp đúng số đã đo và
/// ghi ở §Design Notes của story: *"Một Chương 48.640 ký tự sinh khoảng 146.000 n-gram độ
/// dài 2–4"*. Không sinh n-gram 1 ký tự (một ký tự đơn không phải một "thuật ngữ") và không
/// vượt 4 (chi phí tăng tuyến tính theo số độ dài mà không thêm được mấy ứng viên thật —
/// phần lớn tên riêng/thuật ngữ tiếng Trung rơi vào 2–4 ký tự).
const ZH_NGRAM_LENGTHS: [usize; 3] = [2, 3, 4];

/// Một ứng viên đã qua trọn bộ lọc — tần suất, dedup lồng (`Zh`), họ (`Zh`), và từ điển.
/// Đây là hình dạng mà [`super::candidate_store`] ghi lô xuống `glossary_candidate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanCandidate {
    /// Chuỗi nguồn — đã cắt khoảng trắng biên là KHÔNG CẦN THIẾT ở đây: một n-gram/cụm hoa
    /// không bao giờ mang khoảng trắng biên (`ngrams`/`tokenize` không sinh ra dạng đó).
    pub source_term: String,
    /// Tổng số lần chuỗi này xuất hiện, cộng dồn qua MỌI segment truyền vào.
    pub occurrence_count: i64,
    /// Segment ĐẦU TIÊN (theo thứ tự mảng `segments`) chứa chuỗi này — `context_example` của
    /// I/O Matrix. **Cắt ở [`CONTEXT_EXAMPLE_CHAR_LIMIT`] ký tự** nếu dài hơn — xem
    /// [`truncated_context_example`] cho lý do và cho luật cắt (biên KÝ TỰ, không byte).
    pub context_example: String,
}

/// Kết luận của MỘT lượt tra từ điển cho ứng viên đã qua lọc tần suất.
///
/// `Inconclusive` là giá trị bắt buộc: `lookup_grouped` có thể trả kết quả rỗng **và**
/// danh sách layer lỗi cùng lúc. Ép ba trạng thái này vào `bool` biến layer lỗi thành
/// “không có trong từ điển”, rồi sinh ứng viên giả — đúng lớp rỗng im lặng trung tâm của
/// dự án.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DictionaryProbe {
    Known,
    Missing,
    Inconclusive,
}

/// Outcome của trọn lượt quét, trước mọi lượt ghi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanOutcome {
    Completed(Vec<ScanCandidate>),
    DictionaryInconclusive,
    Cancelled,
}

/// Trần độ dài của [`ScanCandidate::context_example`] — TÍNH BẰNG KÝ TỰ (không byte).
///
/// 🔴 **VÁ 2026-08-22 (rà ba lớp)** — bản trước ghi thẳng `segments[first_segment].to_owned()`
/// không một trần nào. `split_source_text` (Story 2.1) phát ra một segment cho MỌI đoạn,
/// kể cả một đoạn KHÔNG kết thúc bằng dấu kết câu (`a_tail_without_a_terminator_is_still_a_
/// segment`, `segment_contract.rs`) — tức một Chương thiếu dấu câu ở đâu đó có thể sinh một
/// "câu" dài hàng nghìn ký tự, và nó sẽ đi NGUYÊN VẸN vào `glossary_candidate.context_example`
/// nếu không chặn. Một hàng bảng chờ nặng hàng KB không giúp gì cho việc "nghiệm thu bằng
/// mắt" mà §Intent của story đòi — nó chỉ làm bảng khó tải và khó đọc.
///
/// 200 ký tự là đủ để đọc trọn một CÂU THẬT (câu dài nhất đo được trong bàn đo của story —
/// văn bản tổng hợp mô phỏng Chương lớn nhất có thật — ngắn hơn nhiều) mà vẫn có một trần rõ
/// ràng cho ca bất thường (đoạn không dấu câu).
pub const CONTEXT_EXAMPLE_CHAR_LIMIT: usize = 200;

/// Cắt `segment` về tối đa [`CONTEXT_EXAMPLE_CHAR_LIMIT`] ký tự — **biên KÝ TỰ, không
/// byte**: một ký tự Hán/nhiều-byte bị cắt giữa chừng bằng chỉ số BYTE sẽ tạo một chuỗi
/// KHÔNG hợp lệ UTF-8 và panic (`panic = "abort"` giết cả tiến trình — không được phép trên
/// đường quét). `str::chars().take(N).collect()` chỉ dừng ở ranh giới điểm mã, không bao giờ
/// giữa một ký tự.
fn truncated_context_example(segment: &str) -> String {
    if segment.chars().count() <= CONTEXT_EXAMPLE_CHAR_LIMIT {
        return segment.to_owned();
    }
    segment.chars().take(CONTEXT_EXAMPLE_CHAR_LIMIT).collect()
}

/// Quét `segments` (đã tách câu lúc nhập, Story 2.1 — KHÔNG tự đoán lại ranh giới) tìm chuỗi
/// lặp **≥ ngưỡng** và **không có trong từ điển nhúng** (`is_known`), trả về theo thứ tự
/// `source_term` tăng dần (tất định — hai lượt chạy trên cùng đầu vào cho cùng thứ tự).
///
/// `threshold` — ngưỡng đọc từ `app_config` (`core::scope::store`), đã qua `parse::<u32>` +
/// mặc định + chặn `<= 0` ở TẦNG GỌI; hàm này tin nó là một số dương hợp lệ.
///
/// `surnames` — bảng họ phổ biến ([`super::surnames::COMMON_SURNAMES`]); chỉ dùng ở nhánh
/// `Zh` để hạ ngưỡng xuống `threshold - 1` cho MỘT hình dạng hẹp: chuỗi 2–3 ký tự có ký tự
/// ĐẦU nằm trong bảng họ (§Design Notes: *"Bảng họ chỉ NỚI, không THÊM cột"*).
///
/// `is_known` — vị từ "chuỗi này đã có trong từ điển nhúng chưa", gọi ĐÚNG MỘT LẦN cho mỗi
/// ứng viên đã qua bộ lọc tần suất (thứ tự lọc là bất biến trung tâm — xem doc-comment đầu
/// module).
pub fn scan_candidates(
    segments: &[&str],
    lang: MatchLang,
    threshold: u32,
    surnames: &[char],
    is_known: &mut dyn FnMut(&str) -> bool,
) -> Vec<ScanCandidate> {
    let mut probe = |term: &str| {
        if is_known(term) {
            DictionaryProbe::Known
        } else {
            DictionaryProbe::Missing
        }
    };
    let mut never_cancelled = || false;
    match scan_candidates_controlled(
        segments,
        lang,
        threshold,
        surnames,
        &mut probe,
        &mut never_cancelled,
    ) {
        ScanOutcome::Completed(out) => out,
        // Wrapper tương thích không có đường tạo hai outcome này: callback bool không thể
        // trả `Inconclusive`, còn `never_cancelled` luôn false. Trả rỗng thay vì panic —
        // `panic = "abort"` không cho phép một assert phòng thủ trên đường sản phẩm.
        ScanOutcome::DictionaryInconclusive | ScanOutcome::Cancelled => Vec::new(),
    }
}

/// Biến thể sản phẩm của [`scan_candidates`]: callback từ điển giữ BA trạng thái và hook
/// huỷ được hỏi ngay trong pha đếm lẫn trước từng lookup. Tần suất/dedup vẫn chạy trước
/// lookup; việc thêm control không dựng một đường thuật toán thứ hai.
pub fn scan_candidates_controlled(
    segments: &[&str],
    lang: MatchLang,
    threshold: u32,
    surnames: &[char],
    probe_dictionary: &mut dyn FnMut(&str) -> DictionaryProbe,
    is_cancelled: &mut dyn FnMut() -> bool,
) -> ScanOutcome {
    let counted = match lang {
        MatchLang::Zh => count_zh_candidates(segments, is_cancelled),
        MatchLang::En => count_en_candidates(segments, is_cancelled),
    };
    let Some(counted) = counted else {
        return ScanOutcome::Cancelled;
    };

    let mut out = Vec::new();
    // Chỉ ứng viên THẬT SỰ `Missing` mới cần context. Khoá theo chỉ số segment để nhiều
    // term cùng xuất hiện lần đầu trong một câu chỉ cắt đúng một `String`; mỗi candidate
    // sau đó clone bản đã cắt thay vì clone/cắt cả segment ngay trong pha đếm.
    let mut context_by_segment: HashMap<usize, String> = HashMap::new();
    for (term, count, first_segment) in counted {
        if count
            < i64::from(effective_threshold(
                term.as_str(),
                lang,
                threshold,
                surnames,
            ))
        {
            continue;
        }
        if is_cancelled() {
            return ScanOutcome::Cancelled;
        }
        match probe_dictionary(&term) {
            DictionaryProbe::Known => continue,
            DictionaryProbe::Missing => {
                let context_example = context_by_segment
                    .entry(first_segment)
                    .or_insert_with(|| truncated_context_example(segments[first_segment]))
                    .clone();
                out.push(ScanCandidate {
            source_term: term,
            occurrence_count: count,
                    context_example,
                });
            }
            DictionaryProbe::Inconclusive => return ScanOutcome::DictionaryInconclusive,
        }
    }

    out.sort_by(|a, b| a.source_term.cmp(&b.source_term));
    ScanOutcome::Completed(out)
}

/// Ngưỡng THẬT SỰ áp cho `term` — `threshold - 1` khi (và chỉ khi) `lang == Zh`, `term` dài
/// 2–3 ký tự, và ký tự ĐẦU của nó nằm trong `surnames`. Mọi ca khác giữ nguyên `threshold`.
///
/// `saturating_sub(1)`: nếu `threshold` đã là 1 (một cấu hình biên, không phải mặc định 5),
/// ngưỡng họ không lùi xuống 0 rồi tràn số — nó dừng ở 1, tức không nới thêm được nữa. Đây
/// là lưới an toàn cho một giá trị mà tầng gọi đã hứa luôn `>= 1` (chặn `<= 0`), không phải
/// một đường được kỳ vọng chạy trên đường sản phẩm.
fn effective_threshold(term: &str, lang: MatchLang, threshold: u32, surnames: &[char]) -> u32 {
    if lang != MatchLang::Zh {
        return threshold;
    }
    let char_count = term.chars().count();
    if char_count != 2 && char_count != 3 {
        return threshold;
    }
    let Some(first) = term.chars().next() else {
        return threshold;
    };
    let listed = surnames.contains(&first)
        || TRADITIONAL_SURNAME_ALIASES
            .iter()
            .find_map(|&(traditional, simplified)| (traditional == first).then_some(simplified))
            .is_some_and(|simplified| surnames.contains(&simplified));
    if listed {
        threshold.saturating_sub(1).max(1)
    } else {
        threshold
    }
}

/// `(chuỗi, tổng tần suất, chỉ số segment xuất hiện ĐẦU TIÊN)` cho nhánh `Zh` — n-gram ký
/// tự độ dài [`ZH_NGRAM_LENGTHS`], sinh RIÊNG cho từng segment (không nối `\n` — segment đã
/// LÀ ranh giới câu, một n-gram không bao giờ được phép bắc cầu qua nó) rồi cộng dồn.
///
/// Ba bước, đúng thứ tự: (1) đếm mọi n-gram, mọi độ dài; (2) loại "chuỗi cha là rác đuôi"
/// (§Design Notes "N-gram lồng" — chuỗi dài hơn có CÙNG tần suất với một chuỗi con của nó
/// là phần đuôi/đầu ăn theo, không phải một thuật ngữ thật); (3) trả phần còn lại, CHƯA lọc
/// ngưỡng/từ điển — hai bước đó là việc của [`scan_candidates`].
fn count_zh_candidates(
    segments: &[&str],
    is_cancelled: &mut dyn FnMut() -> bool,
) -> Option<Vec<(String, i64, usize)>> {
    // `freq`/`first_seen` khoá theo CHUỖI — một `HashMap` là đủ vì bước dedup lồng ngay
    // dưới chỉ cần TRA (không cần thứ tự); `scan_candidates` sắp lại kết quả cuối cùng.
    let mut freq: HashMap<String, i64> = HashMap::new();
    let mut first_segment: HashMap<String, usize> = HashMap::new();

    for (segment_index, &segment) in segments.iter().enumerate() {
        if is_cancelled() {
            return None;
        }
        for &n in &ZH_NGRAM_LENGTHS {
            for gram in ngrams(segment, MatchLang::Zh, n) {
                if is_cancelled() {
                    return None;
                }
                // 🔴 Loại n-gram bắc qua dấu câu/khoảng trắng — `ngrams` sinh MỌI cửa sổ
                // trượt, kể cả những cửa sổ chứa dấu phẩy/xuống dòng/khoảng trắng ở giữa. Một
                // "thuật ngữ" mang dấu câu không phải một chuỗi lặp có nghĩa — nó là rác hình
                // học của cửa sổ trượt. `char::is_alphanumeric()` chấp nhận CẢ chữ Hán lẫn
                // chữ/số Latin lẫn trong đoạn (không phải một định nghĩa "là chữ Hán" thứ
                // hai — kho chỉ cho phép đúng MỘT định nghĩa `is_han`,
                // `core::dict::is_han`, và hàm này không đụng tới nó).
                if !gram.chars().all(char::is_alphanumeric) {
                    continue;
                }
                *freq.entry(gram.clone()).or_insert(0) += 1;
                first_segment.entry(gram).or_insert(segment_index);
            }
        }
    }

    let dropped = zh_nested_padding(&freq);

    Some(
    freq.into_iter()
        .filter(|(term, _)| !dropped.contains(term))
        .map(|(term, count)| {
                let segment_index = first_segment.remove(&term).unwrap_or_default();
                (term, count, segment_index)
        })
            .collect(),
    )
}

/// Tập chuỗi "rác đuôi" — chuỗi DÀI HƠN mà tần suất **bằng ĐÚNG** tần suất của một chuỗi
/// con liền kề (bớt ký tự đầu HOẶC ký tự cuối). Quy tắc (§Design Notes): tần suất **bằng
/// nhau** nghĩa là mọi lần chuỗi ngắn xuất hiện đều kéo theo đúng một ký tự láng giềng cố
/// định — chuỗi dài là một cửa sổ trượt ăn theo, không phải một thuật ngữ độc lập. Tần suất
/// **khác nhau** nghĩa là cả hai cùng đứng độc lập ở đâu đó — giữ cả hai.
///
/// Chạy từ độ dài DÀI nhất xuống, so với chuỗi con độ dài `n - 1` (nằm trong
/// [`ZH_NGRAM_LENGTHS`] với mọi `n > 2`, nên độ dài 2 không có gì để so — nó luôn đứng
/// một mình ở lớp dedup này, đúng I/O Matrix *"chuỗi 2–3 ký tự là chuỗi thật khi tần suất
/// bằng nhau"*).
fn zh_nested_padding(freq: &HashMap<String, i64>) -> std::collections::HashSet<String> {
    let mut dropped: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut lengths: Vec<usize> = ZH_NGRAM_LENGTHS.to_vec();
    lengths.sort_unstable_by(|a, b| b.cmp(a)); // dài nhất trước

    for &n in &lengths {
        if n <= 2 {
            continue; // khong con chuoi con nao ngan hon trong tap da sinh (min = 2)
        }
        // Chép danh sách trước khi so — `freq` không đổi trong vòng lặp này, chỉ `dropped`
        // đổi, nên một `Vec` chụp một lần là đủ và tất định (sắp để lặp cùng thứ tự mọi lần).
        let mut terms: Vec<&String> = freq.keys().filter(|t| t.chars().count() == n).collect();
        terms.sort();

        for term in terms {
            let term_freq = *freq.get(term).expect("term den tu chinh freq.keys()");
            let chars: Vec<char> = term.chars().collect();
            let drop_last: String = chars[..chars.len() - 1].iter().collect();
            let drop_first: String = chars[1..].iter().collect();

            let matches_child = |child: &str| freq.get(child).is_some_and(|&f| f == term_freq);

            if matches_child(&drop_last) || matches_child(&drop_first) {
                dropped.insert(term.clone());
            }
        }
    }

    dropped
}

/// `(chuỗi, tổng tần suất, chỉ số segment xuất hiện ĐẦU TIÊN)` cho nhánh `En` — cụm hoa
/// LIỀN NHAU, KHÔNG đứng đầu segment (I/O Matrix: *"The mở đầu 300 segment ⇒ 0 hàng"*).
///
/// Một "cụm hoa" là dãy TOKEN liên tiếp (theo [`tokenize`]) mà mỗi token bắt đầu bằng một
/// chữ cái ASCII hoa, và không dấu kết câu/xuống dòng nào chen giữa hai token liên tiếp
/// trong dãy (cùng luật `crosses_sentence_boundary` mà `core::matching::find_terms` đã áp
/// cho lý do khác — Story 3.4b tiêu thụ lại đúng luật này cho một mục đích thứ ba). Dãy
/// LẤY TRỌN VẸN (không sinh mọi dãy con) làm MỘT ứng viên — module này không cố đoán thêm
/// một tên ngắn hơn nằm bên trong một cụm dài hơn.
fn count_en_candidates(
    segments: &[&str],
    is_cancelled: &mut dyn FnMut() -> bool,
) -> Option<Vec<(String, i64, usize)>> {
    let mut freq: HashMap<String, i64> = HashMap::new();
    let mut first_segment: HashMap<String, usize> = HashMap::new();

    for (segment_index, &segment) in segments.iter().enumerate() {
        if is_cancelled() {
            return None;
        }
        let tokens = tokenize(segment, MatchLang::En);
        let mut i = 0usize;
        while i < tokens.len() {
            if is_cancelled() {
                return None;
            }
            if !is_capitalized(tokens[i].text) {
                i += 1;
                continue;
            }

            let run_start = i;
            let mut run_end = i;
            while run_end + 1 < tokens.len()
                && is_capitalized(tokens[run_end + 1].text)
                && !gap_crosses_sentence_boundary(segment, &tokens, run_end, run_end + 1)
            {
                run_end += 1;
            }

            // 🔴 Vị trí ĐẦU segment bị loại NGUYÊN CỤM — không chỉ token đầu tiên. Một
            // "Fire Dragon" đứng ngay đầu câu bị loại y hệt một "The" đứng đầu câu; cả hai
            // đều là ca "hoa đầu câu", không phải "cụm hoa giữa câu".
            if run_start != 0 {
                // 🔴 Chuẩn hoá từ chính token, không từ lát cắt thô của segment. Nhờ đó
                // `Fire Dragon`, `Fire  Dragon` và `Fire, Dragon` cùng một khoá và không
                // mang dấu câu vào `source_term`; tần suất được cộng dồn thay vì tách ba
                // ứng viên giả chỉ vì khoảng trắng/dấu phẩy.
                let phrase = tokens[run_start..=run_end]
                    .iter()
                    .map(|token| token.text)
                    .collect::<Vec<_>>()
                    .join(" ");
                *freq.entry(phrase.clone()).or_insert(0) += 1;
                first_segment.entry(phrase).or_insert(segment_index);
            }

            i = run_end + 1;
        }
    }

    Some(
        freq.into_iter()
        .map(|(term, count)| {
                let segment_index = first_segment.remove(&term).unwrap_or_default();
                (term, count, segment_index)
        })
            .collect(),
    )
}

/// Ký tự ĐẦU của `token` là một chữ cái ASCII hoa. `tokenize` (nhánh `En`) chỉ sinh token
/// gồm toàn `is_ascii_alphanumeric`, nên một token số thuần (`"2024"`) có ký tự đầu là
/// chữ số — không hoa, không rơi vào nhánh này, đúng hành vi mong đợi.
fn is_capitalized(token: &str) -> bool {
    token.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

/// Có dấu kết câu (`.`/`!`/`?`) hay xuống dòng nằm giữa token `a`/`b` (liền kề trong mảng
/// `tokens`) không — CÙNG luật `core::matching::find_terms` áp cho nhánh `En`, đọc lại ở
/// đây cho một mục đích khác (không nối một cụm hoa XUYÊN hai câu).
fn gap_crosses_sentence_boundary(
    segment: &str,
    tokens: &[crate::core::matching::MatchToken<'_>],
    a: usize,
    b: usize,
) -> bool {
    segment[tokens[a].span.end..tokens[b].span.start].contains(['.', '!', '?', '\n'])
}
