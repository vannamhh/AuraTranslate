//! I/O & Edge-Case Matrix của Story 3.5 (quét ứng viên khi nhập tài liệu) — TẦNG THUẦN,
//! không `Store` nào dựng ở tệp này. `core::glossary::scan::scan_candidates_controlled` là
//! module LÁ (doc-comment của chính nó) — điều kiện để mọi ca dưới đây chạy TẤT ĐỊNH, không
//! cần một `.atproj` nào.
//!
//! ⚠️ Ca *"Đã có trong Glossary"*/*"Đã từng bị bỏ"* của I/O Matrix chạm SQL
//! (`WHERE NOT EXISTS`/`ON CONFLICT DO NOTHING`) — chúng nằm ở `glossary_contract.rs`
//! (`insert_import_scan_candidates`), không ở đây.
//!
//! ⚠️ Mỗi hàng của I/O Matrix là ĐÚNG MỘT ca, tên hàm là một CÂU khẳng định.

use auratranslate_lib::core::glossary::scan::{
    CONTEXT_EXAMPLE_CHAR_LIMIT, DictionaryProbe, ScanCandidate, ScanOutcome,
    scan_candidates_controlled,
};
use auratranslate_lib::core::glossary::surnames::{COMMON_SURNAMES, TRADITIONAL_SURNAME_ALIASES};
use auratranslate_lib::core::matching::MatchLang;
use auratranslate_lib::core::scope::store::{
    DEFAULT_GLOSSARY_SCAN_THRESHOLD, parse_glossary_scan_threshold, resolve_library_root_value,
};

/// Vị từ `is_known` không bao giờ trả `true` — dùng cho mọi ca không cần một từ điển giả.
fn nothing_known(_term: &str) -> bool {
    false
}

/// Adapter TEST `bool -> DictionaryProbe`, giữ CỤC BỘ ở bàn test này — 🔵 2026-08-26 (cụm
/// F ③). `core::glossary::scan::scan_candidates` (vỏ `bool` công khai) đã bị xoá: nó có 0
/// chỗ gọi sản phẩm và biến một layer LỖI thành "không có trong từ điển", đúng lớp rỗng im
/// lặng trung tâm của dự án. Mọi ca dưới đây chỉ cần một vị từ `bool` tất định nên tự giữ
/// đúng phần thân adapter đã xoá, không phục hồi một API sản phẩm. ⚠️ `commands/project.rs`
/// (`#[cfg(test)] mod tests::scan_candidates_bool_probe`) giữ một bản CHÉP SONG SONG của
/// đúng phần thân này — hai crate test khác nhau (tích hợp `tests/**` so với đơn vị trong
/// `src/`) không chia sẻ mã được, và NFR15 cấm thêm một crate hỗ trợ mới chỉ để hợp nhất
/// hai bản chép sáu dòng.
fn scan_candidates(
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
        ScanOutcome::DictionaryInconclusive | ScanOutcome::Cancelled => Vec::new(),
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 1 — Zh, tên lặp
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_chinese_name_repeated_forty_times_produces_one_row_with_the_right_count_and_context() {
    // 40 câu riêng biệt, mỗi câu chứa `萧炎` đúng một lần -- segment đã tách câu (Story 2.1),
    // không nối `\n`.
    let segments: Vec<String> = (0..40).map(|i| format!("萧炎在第{i}章出现。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "萧炎")
        .unwrap_or_else(|| panic!("khong thay `萧炎` trong: {out:?}"));
    assert_eq!(hit.occurrence_count, 40);
    assert_eq!(hit.context_example, "萧炎在第0章出现。");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 2 — Zh, n-gram lồng
// ═════════════════════════════════════════════════════════════════════════════════

/// `萧炎` 40 lần, MỖI lần đi kèm CÙNG một hậu tố cố định (`萧炎的`) -- tần suất hai chuỗi
/// BẰNG NHAU ⇒ chuỗi dài là rác đuôi, GIỮ chuỗi ngắn, chuỗi dài KHÔNG xuất hiện trong kết
/// quả.
#[test]
fn a_nested_ngram_with_equal_frequency_to_its_substring_is_dropped_as_padding() {
    let segments: Vec<String> = (0..40)
        .map(|i| format!("萧炎的实力在第{i}章提升。"))
        .collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        out.iter().any(|c| c.source_term == "萧炎"),
        "chuoi ngan `萧炎` phai co mat: {out:?}"
    );
    assert!(
        !out.iter().any(|c| c.source_term == "萧炎的"),
        "chuoi dai `萧炎的` phai bi loai -- tan suat bang chuoi con, la rac duoi: {out:?}"
    );
}

/// Cùng chuỗi con, nhưng chuỗi dài xuất hiện Ở MỘT SỐ LƯỢT KHÁC — tần suất KHÁC nhau ⇒ CẢ
/// HAI là chuỗi thật, cả hai giữ lại.
///
/// ⚠️ **Cả HAI chuỗi con của `萧炎的` (`萧炎` VÀ `炎的`) phải LỆCH tần suất với nó**, không
/// chỉ một — `matches_child(drop_last) || matches_child(drop_first)` là một phép HOẶC:
/// khớp một bên là đủ để bị loại. Vì vậy phải thêm occurrence RIÊNG cho `炎的` (không đi
/// kèm `萧`) để nó không tình cờ bằng đúng tần suất của `萧炎的`.
#[test]
fn a_nested_ngram_with_a_different_frequency_from_its_substring_is_kept_alongside_it() {
    let mut segments: Vec<String> = (0..40).map(|i| format!("萧炎在第{i}章出现。")).collect();
    // Thêm MƯỜI câu mang `萧炎的` -- `萧炎` tổng cộng 50 lần (40 đứng một mình + 10 kèm hậu
    // tố `的`), `萧炎的` 10 lần, và `炎的` (drop_first của `萧炎的`) CŨNG 10 lần nếu dừng ở
    // đây -- đúng cái bẫy phải né, xử lý ở khối dưới.
    for i in 0..10 {
        segments.push(format!("萧炎的实力在第{i}章提升。"));
    }
    // NĂM câu nữa mang `炎的` KHÔNG đi kèm `萧` -- đẩy tần suất `炎的` lên 15, lệch khỏi 10
    // của `萧炎的`, để phép so KHÔNG tình cờ khớp qua nhánh `drop_first`.
    for i in 0..5 {
        segments.push(format!("此人脾气炎的很怪第{i}回。"));
    }
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let short = out
        .iter()
        .find(|c| c.source_term == "萧炎")
        .unwrap_or_else(|| panic!("khong thay `萧炎`: {out:?}"));
    assert_eq!(short.occurrence_count, 50);

    let long = out
        .iter()
        .find(|c| c.source_term == "萧炎的")
        .unwrap_or_else(|| {
            panic!(
                "khong thay `萧炎的` -- tan suat khac ca hai chuoi con, phai giu ca hai: {out:?}"
            )
        });
    assert_eq!(long.occurrence_count, 10);
}

/// Rác **neo-ĐẦU**: chuỗi dài khớp chuỗi con qua nhánh `drop_first` và **CHỈ** nhánh đó.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO CA NÀY TỒN TẠI — hai ca ngay trên KHÔNG canh được nửa `drop_first`
/// ─────────────────────────────────────────────────────────────────────────────
/// `zh_nested_padding` loại một chuỗi khi `matches_child(&drop_last) ||
/// matches_child(&drop_first)`. Không ca nào trước đây **cô lập** được vế PHẢI:
/// - Ca thứ nhất (`..._equal_frequency_...`) khớp **CẢ HAI** vế — trong `萧炎的实力在第{i}章
///   提升。` thì `萧炎` và `炎的` đều đúng 40 như `萧炎的`. Một phép HOẶC mà cả hai vế cùng
///   đúng thì cắt vế nào nó cũng xanh, nên nó không nói được gì về vế phải.
/// - Ca thứ hai (`..._different_frequency_...`) **cố ý thêm câu riêng cho `炎的`** để phép so
///   KHÔNG tình cờ chạm vế phải (đọc chú thích ⚠️ của nó).
///
/// ⇒ Vế `drop_first` chưa từng có ai canh. *(🔵 2026-08-26 — bản đầu của chú thích này viết
/// hai ca kia *"đều đi qua vế TRÁI"*; vế trái thì đúng, nhưng ca thứ nhất khớp cả hai, và
/// nói *"đi qua vế trái"* làm người đọc tưởng vế phải đã bị loại trừ ở đó. Sửa tại chỗ.)*
///
/// ⚠️ **Đo 2026-08-26 trên `3be0f5f`** (vòng rà Epic 3, cụm E): cắt bỏ hẳn
/// `|| matches_child(&drop_first)` khỏi `scan.rs` rồi chạy — `glossary_scan_contract` 25/25 ·
/// `glossary_commands_contract` 29/29 · `glossary_boundary` 11/11 · `glossary_contract` 72/72,
/// **XANH TRỌN**. Hệ quả thật: ứng viên rác neo-đầu (`在萧炎`, `的实力`, `了一个`) đi thẳng vào
/// bảng chờ mà UI Story 3.2/3.8 cho phép duyệt vào Glossary bằng MỘT phím.
///
/// **Cách fixture cô lập đúng một nhánh:** `在萧炎` xuất hiện 40 lần, và
/// - `drop_first` = `萧炎` cũng đúng **40** *(chuỗi này không đứng ở đâu khác)* ⇒ vế phải KHỚP;
/// - `drop_last` = `在萧` là **47** *(bảy câu `在萧家` không kèm `炎`)* ⇒ vế trái KHÔNG khớp.
///
/// 🔴 **Hai con số ấy được KHẲNG ĐỊNH trong ca, không chỉ viết ở chú thích** (vòng rà bước 4,
/// lăng kính blind-hunter, 2026-08-26). Nếu một lượt sửa fixture về sau vô tình làm `在萧` cũng
/// bằng 40 thì vế TRÁI khớp, `在萧炎` vẫn bị loại, và ca này **xanh vì một lý do khác hẳn** —
/// một phép kiểm xanh trên một mệnh đề nó không còn kiểm nữa.
///
/// ⚠️ `萧` nằm trong `COMMON_SURNAMES`, nên `effective_threshold` hạ ngưỡng của `萧炎` xuống 4.
/// Ở fixture này điều đó **trơ**: mọi tần suất (40 · 47 · 49) đều vượt xa cả 4 lẫn 5. Ghi ra
/// vì luật hạ ngưỡng theo họ là một đường mã KHÁC hẳn đường dedup đang canh, và một người đọc
/// sau nên biết hai đường ấy tình cờ chạm nhau ở đây chứ không phụ thuộc nhau.
///
/// Gỡ vế phải ra ⇒ `在萧炎` sống sót ⇒ ca này ĐỎ ở đúng `assert!` cuối.
#[test]
fn a_head_anchored_ngram_matching_only_its_drop_first_child_is_dropped_as_padding() {
    // 40 câu mang `在萧炎` -- `萧炎` KHÔNG bao giờ đứng ngoài cụm này, nên hai tần suất bằng nhau.
    let mut segments: Vec<String> = (0..40)
        .map(|i| format!("他在萧炎身旁站了第{i}天。"))
        .collect();
    // BẢY câu mang `在萧` KHÔNG kèm `炎` -- đẩy `在萧` lên 47, lệch khỏi 40 của `在萧炎`, để
    // phép so KHÔNG tình cờ khớp qua nhánh `drop_last`. Không có bảy câu này thì ca vẫn xanh
    // sau khi gỡ vế phải, tức nó không canh gì cả.
    for i in 0..7 {
        segments.push(format!("他在萧家住了第{i}天。"));
    }
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let short = out
        .iter()
        .find(|c| c.source_term == "萧炎")
        .unwrap_or_else(|| panic!("khong thay `萧炎` -- chuoi con phai o lai: {out:?}"));
    assert_eq!(short.occurrence_count, 40);

    // 🔴 Ghim SỐ HỌC của fixture trước khi khẳng định kết luận. `在萧` phải LỆCH 40, nếu không
    // thi ve TRAI cung khop va ca nay xanh vi mot ly do khac han (xem doc-comment).
    let drop_last_sibling = out
        .iter()
        .find(|c| c.source_term == "在萧")
        .unwrap_or_else(|| panic!("khong thay `在萧` -- fixture khong dung nhu chu thich: {out:?}"));
    assert_eq!(
        drop_last_sibling.occurrence_count, 47,
        "`在萧` phai LECH khoi 40 de ve `drop_last` KHONG khop -- day la dieu kien duy nhat \
         lam ca nay co-lap duoc nhanh `drop_first`"
    );

    assert!(
        !out.iter().any(|c| c.source_term == "在萧炎"),
        "chuoi dai `在萧炎` phai bi loai qua nhanh `drop_first` -- tan suat bang chuoi con \
         `萧炎` (40), con `在萧` (47) thi lech. Con no trong ket qua nghia la ve \
         `|| matches_child(&drop_first)` khong chay: {out:?}"
    );
}

/// Đối chứng CHIỀU NGƯỢC của ca ngay trên: cùng hình dạng neo-đầu, nhưng **cả hai** chuỗi
/// con lệch tần suất ⇒ `在萧炎` là một chuỗi thật và phải được GIỮ.
///
/// 🔴 Không thừa, và không trùng ca `..._different_frequency_...` ở trên: ca đó dựng chiều
/// **đuôi**. Không có ca này thì `assert!` phủ định của ca trên xanh cả trong một thế giới
/// nơi `在萧炎` không bao giờ ra được khỏi lượt quét vì một lý do khác hẳn (bộ lọc
/// `is_alphanumeric`, ngưỡng, hay `ZH_NGRAM_LENGTHS`) — tức một phép kiểm khẳng định một
/// điều nó chưa từng quan sát.
#[test]
fn a_head_anchored_ngram_matching_neither_child_is_kept() {
    let mut segments: Vec<String> = (0..40)
        .map(|i| format!("他在萧炎身旁站了第{i}天。"))
        .collect();
    // `在萧` lên 47 -- lệch khỏi 40 của `在萧炎` (vế `drop_last` không khớp).
    for i in 0..7 {
        segments.push(format!("他在萧家住了第{i}天。"));
    }
    // `萧炎` lên 49 -- lệch khỏi 40 (vế `drop_first` cũng không khớp). Đây là DÒNG DUY NHẤT
    // khác ca trên, và nó phải đủ để lật kết luận.
    for i in 0..9 {
        segments.push(format!("萧炎独自离开第{i}处山谷。"));
    }
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let short = out
        .iter()
        .find(|c| c.source_term == "萧炎")
        .unwrap_or_else(|| panic!("khong thay `萧炎`: {out:?}"));
    assert_eq!(short.occurrence_count, 49);

    let long = out
        .iter()
        .find(|c| c.source_term == "在萧炎")
        .unwrap_or_else(|| {
            panic!(
                "khong thay `在萧炎` -- tan suat (40) khac CA HAI chuoi con (`在萧` 47, \
                 `萧炎` 49), phai giu ca hai: {out:?}"
            )
        });
    assert_eq!(long.occurrence_count, 40);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 3 — Zh, có trong từ điển
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_chinese_term_already_in_the_dictionary_produces_zero_rows() {
    let segments: Vec<String> = (0..80).map(|i| format!("他在{i}天里修炼。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = |term: &str| term == "修炼";
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        !out.iter().any(|c| c.source_term == "修炼"),
        "`修炼` da co trong tu dien -- phai bi loai o buoc tra, khong duoc ra hang: {out:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 4/5 — Zh, đoán tên người (bảng họ)
// ═════════════════════════════════════════════════════════════════════════════════

/// Chuỗi 2 ký tự, ký tự đầu (`萧`) nằm trong bảng họ, tần suất = ngưỡng − 1 (4 lần, ngưỡng
/// 5) ⇒ VẪN ra một hàng nhờ bảng họ hạ ngưỡng.
#[test]
fn a_two_char_surname_shaped_string_below_threshold_by_one_is_kept_via_the_surname_table() {
    let segments: Vec<String> = (0..4).map(|i| format!("萧风在第{i}章登场。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "萧风")
        .unwrap_or_else(|| {
            panic!("`萧风` (ho + 1 ky tu, 4 lan, nguong 5-1=4) phai co mat: {out:?}")
        });
    assert_eq!(hit.occurrence_count, 4);
}

/// Cùng hình dạng nhưng CHỈ 3 lần (dưới cả ngưỡng đã hạ, `5 - 1 = 4`) ⇒ vẫn 0 hàng — bảng
/// họ NỚI đúng một bậc, không xoá hẳn ngưỡng.
#[test]
fn a_surname_shaped_string_still_below_the_lowered_threshold_produces_zero_rows() {
    let segments: Vec<String> = (0..3).map(|i| format!("萧风在第{i}章登场。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        !out.iter().any(|c| c.source_term == "萧风"),
        "3 lan < nguong da ha (4) -- van phai bi loai: {out:?}"
    );
}

/// `COMMON_SURNAMES` chủ ý chỉ giữ dạng giản thể để không dựng hai bảng luật song song;
/// alias phồn thể phải đi qua cùng phép nới đúng một bậc.
#[test]
fn a_traditional_surname_alias_below_threshold_by_one_is_kept() {
    let segments: Vec<String> = (0..4).map(|i| format!("蕭炎在第{i}章登场。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "蕭炎")
        .unwrap_or_else(|| panic!("`蕭炎` phai dung cung luat ha nguong voi `萧炎`: {out:?}"));
    assert_eq!(hit.occurrence_count, 4);
}

/// Ký tự đầu trong bảng họ, nhưng chuỗi 4 ký tự (quá dài cho luật nới) ⇒ đi đường ngưỡng
/// ĐẦY ĐỦ — 4 lần (== ngưỡng đã hạ nhưng < ngưỡng đầy đủ 5) phải bị loại.
#[test]
fn a_surname_shaped_prefix_on_a_four_char_string_does_not_get_the_lowered_threshold() {
    let segments: Vec<String> = (0..4).map(|i| format!("萧风雷动在第{i}章登场。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        !out.iter().any(|c| c.source_term == "萧风雷动"),
        "4 ky tu -- bang ho KHONG noi (chi noi cho 2-3 ky tu); 4 lan < nguong day du 5: {out:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Cụm F ④ — năm cặp alias phồn thể mới (`陳/陈 張/张 劉/刘 楊/杨 黃/黄`), Ice chốt 2026-08-26
// ═════════════════════════════════════════════════════════════════════════════════

/// §I/O Matrix ④a — chuỗi 2 ký tự bắt đầu bằng MỘT trong năm chữ phồn thể mới (`陳`), tần
/// suất = ngưỡng − 1 ⇒ ngưỡng hạ còn `threshold - 1`, giống hệt vế giản thể `陈` (cùng cơ
/// chế đã canh ở `a_traditional_surname_alias_below_threshold_by_one_is_kept` cho `蕭`).
///
/// Đối chứng GỠ-CHỖ-NỐI (§Boundaries cụm F): gỡ cặp `('陳', '陈')` khỏi
/// `TRADITIONAL_SURNAME_ALIASES` ⇒ ca này phải ĐỎ.
#[test]
fn a_new_traditional_surname_alias_below_threshold_by_one_is_kept() {
    let segments: Vec<String> = (0..4).map(|i| format!("陳風在第{i}章登场。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "陳風")
        .unwrap_or_else(|| panic!("`陳風` (ho phon the + 1 ky tu, 4 lan, nguong 5-1=4) phai co mat: {out:?}"));
    assert_eq!(hit.occurrence_count, 4);
}

/// §I/O Matrix ④b — đối chứng NGƯỢC: chuỗi 2 ký tự bắt đầu bằng một chữ phồn thể **không**
/// phải họ (`鬍`, "râu" — chính chữ mà §Boundaries nêu tên là bẫy nếu ai đó nhập trọn 134
/// cặp đo được), tần suất = ngưỡng − 1 ⇒ ngưỡng GIỮ NGUYÊN `threshold`, 0 hàng — bảng chỉ
/// NỚI cho họ, không nới bừa cho mọi chữ phồn thể.
#[test]
fn a_non_surname_traditional_character_does_not_get_the_lowered_threshold() {
    let segments: Vec<String> = (0..4).map(|i| format!("鬍子在第{i}章登场。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        !out.iter().any(|c| c.source_term == "鬍子"),
        "`鬍` khong phai mot ho -- 4 lan < nguong DAY DU 5 phai bi loai (khong duoc noi): {out:?}"
    );
}

/// §I/O Matrix ④c — ca QUẦN THỂ: khẳng định CẢ HAI mệnh đề mà bảng đo 134 cặp cho thấy dễ
/// vỡ nhất, trên TOÀN BỘ `TRADITIONAL_SURNAME_ALIASES` chứ không chỉ cặp mới thêm — mọi vế
/// GIẢN phải là một họ thật (nằm trong `COMMON_SURNAMES`), và vế PHỒN không được TỰ NÓ là
/// một họ KHÁC trong bảng. `於` là ca duy nhất trong 134 cặp đo được mắc vế thứ hai (nó
/// nằm SẴN trong `COMMON_SURNAMES`) — đây là ca sẽ ĐỎ nếu ai đó dán nguyên bảng đo vào
/// `TRADITIONAL_SURNAME_ALIASES` thay vì năm cặp Ice đã chốt.
///
/// ⚠️ **GIỚI HẠN THẬT của ca này — ghi ra thay vì để người sau tin nó bắt được mọi cặp sai
/// (vòng rà 1, 2026-08-26).** Ca này CHỈ chặn được lớp *"vế phồn tự nó là một họ KHÁC trong
/// bảng"* (khuôn `於→于`). Nó KHÔNG chặn được lớp *"chữ phồn KHÔNG phải họ, vế giản LÀ một
/// họ thật"* — `衚→胡` (ngõ hẹp) lọt qua CẢ HAI `assert!` y hệt `鬍→胡` lọt qua ca ④b: `衚`
/// không nằm trong `COMMON_SURNAMES` (qua được vế 2) và `胡` là một họ thật (qua được vế 1),
/// nhưng `衚` không hề mang hình dạng tên người. Đo được: cả năm chữ phồn của lớp 1
/// (`鬍 週 鬱 餘 衚`, xem `surnames.rs` doc-comment của `TRADITIONAL_SURNAME_ALIASES`) đều
/// qua trót lọt ca quần thể này nếu ai đó thêm chúng vào bảng — **duyệt qua ca ④c KHÔNG ĐỦ**
/// để nghiệm thu một cặp mới; còn cần đối chiếu TAY (hoặc một ca mới) rằng vế PHỒN thật sự
/// mang hình dạng một họ, không chỉ "không phải một họ khác đã có".
#[test]
fn every_traditional_surname_alias_maps_to_a_real_surname_and_the_traditional_side_is_not_itself_a_listed_surname()
 {
    for &(traditional, simplified) in TRADITIONAL_SURNAME_ALIASES {
        assert!(
            COMMON_SURNAMES.contains(&simplified),
            "ve GIAN `{simplified}` cua cap ({traditional}, {simplified}) khong nam trong \
             COMMON_SURNAMES -- moi alias phai chuan hoa VE MOT HO THAT, khong phai mot chu tuy y"
        );
        assert!(
            !COMMON_SURNAMES.contains(&traditional),
            "ve PHON `{traditional}` cua cap ({traditional}, {simplified}) TU NO da la mot ho \
             KHAC trong COMMON_SURNAMES -- day dung la bay `於→于` (`於` da la mot ho rieng): \
             mot alias nhu vay se noi nguong cho MOI chuoi bat dau bang `{traditional}`, du no \
             khong mang hinh dang ten nguoi"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 6/7 — En, cụm hoa
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_capitalized_phrase_repeated_mid_sentence_produces_one_row() {
    let segments: Vec<String> = (0..12)
        .map(|i| format!("The beast known as Fire Dragon appeared again at hour {i}."))
        .collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::En, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "Fire Dragon")
        .unwrap_or_else(|| panic!("khong thay `Fire Dragon`: {out:?}"));
    assert_eq!(hit.occurrence_count, 12);
}

#[test]
fn whitespace_and_comma_variants_share_one_normalized_capitalized_phrase_key() {
    let segments = [
        "A beast called Fire Dragon appeared.",
        "A beast called Fire  Dragon appeared.",
        "A beast called Fire, Dragon appeared.",
    ];

    let mut is_known = nothing_known;
    let out = scan_candidates(&segments, MatchLang::En, 3, COMMON_SURNAMES, &mut is_known);

    let matching: Vec<_> = out
        .iter()
        .filter(|c| c.source_term.contains("Fire"))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "ba bien the phai gom vao mot key: {out:?}"
    );
    assert_eq!(matching[0].source_term, "Fire Dragon");
    assert_eq!(matching[0].occurrence_count, 3);
    assert!(!matching[0].source_term.contains(','));
    assert!(!matching[0].source_term.contains("  "));
}

#[test]
fn a_capitalized_word_opening_three_hundred_segments_produces_zero_rows() {
    let segments: Vec<String> = (0..300)
        .map(|i| format!("The hero walked away, tired, sentence {i}."))
        .collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::En, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        !out.iter().any(|c| c.source_term == "The"),
        "`The` dung dau moi segment -- vi tri dau segment bi loai: {out:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 8 — Dưới ngưỡng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_string_repeated_four_times_at_threshold_five_produces_zero_rows() {
    // 🔴 KHÔNG đứng đầu segment — nếu không, ca này đo nhầm luật "hoa đầu câu" (Hàng 7) chứ
    // không đo luật "dưới ngưỡng" mà tên hàm khai.
    let segments: Vec<String> = (0..4)
        .map(|i| format!("A beast called Fire Dragon roared at hour {i}."))
        .collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::En, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        !out.iter().any(|c| c.source_term == "Fire Dragon"),
        "4 lan < nguong 5: {out:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Review 2026-08-22 — thứ tự lọc, outcome ba trạng thái và huỷ trong pha đếm
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_term_below_threshold_never_reaches_the_dictionary_predicate() {
    let segments: Vec<String> = (0..4)
        .map(|i| format!("A beast called Fire Dragon roared at hour {i}."))
        .collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
    let mut calls = 0usize;
    let mut probe = |_term: &str| {
        calls += 1;
        DictionaryProbe::Missing
    };
    let mut never_cancelled = || false;

    let outcome = scan_candidates_controlled(
        &refs,
        MatchLang::En,
        5,
        COMMON_SURNAMES,
        &mut probe,
        &mut never_cancelled,
    );

    assert_eq!(outcome, ScanOutcome::Completed(Vec::new()));
    assert_eq!(calls, 0, "duoi nguong phai bi loai TRUOC dictionary");
}

#[test]
fn a_term_at_threshold_reaches_the_dictionary_predicate_exactly_once() {
    let segments: Vec<String> = (0..5)
        .map(|i| format!("A beast called Fire Dragon roared at hour {i}."))
        .collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
    let mut calls = 0usize;
    let mut probe = |term: &str| {
        assert_eq!(term, "Fire Dragon");
        calls += 1;
        DictionaryProbe::Missing
    };
    let mut never_cancelled = || false;

    let outcome = scan_candidates_controlled(
        &refs,
        MatchLang::En,
        5,
        COMMON_SURNAMES,
        &mut probe,
        &mut never_cancelled,
    );

    let ScanOutcome::Completed(out) = outcome else {
        panic!("du nguong va dictionary Missing phai hoan tat");
    };
    assert_eq!(out.len(), 1);
    assert_eq!(calls, 1, "dedup xong moi lookup dung mot lan cho key");
}

#[test]
fn an_inconclusive_dictionary_probe_aborts_the_batch_without_candidates() {
    let segments: Vec<String> = (0..5)
        .map(|i| format!("A beast called Fire Dragon roared at hour {i}."))
        .collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
    let mut probe = |_term: &str| DictionaryProbe::Inconclusive;
    let mut never_cancelled = || false;

    let outcome = scan_candidates_controlled(
        &refs,
        MatchLang::En,
        5,
        COMMON_SURNAMES,
        &mut probe,
        &mut never_cancelled,
    );

    assert_eq!(outcome, ScanOutcome::DictionaryInconclusive);
}

#[test]
fn cancellation_during_count_stops_before_any_dictionary_probe() {
    let segments: Vec<String> = (0..500)
        .map(|i| format!("A beast called Fire Dragon roared at hour {i}."))
        .collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
    let mut dictionary_calls = 0usize;
    let mut probe = |_term: &str| {
        dictionary_calls += 1;
        DictionaryProbe::Missing
    };
    let mut cancel_checks = 0usize;
    let mut cancel_while_counting = || {
        cancel_checks += 1;
        cancel_checks == 3
    };

    let outcome = scan_candidates_controlled(
        &refs,
        MatchLang::En,
        5,
        COMMON_SURNAMES,
        &mut probe,
        &mut cancel_while_counting,
    );

    assert_eq!(outcome, ScanOutcome::Cancelled);
    assert_eq!(dictionary_calls, 0);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 9 — Chương rỗng / toàn khoảng trắng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_empty_chapter_produces_zero_candidates() {
    let refs: Vec<&str> = Vec::new();
    let mut is_known = nothing_known;

    let out_zh = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);
    assert!(out_zh.is_empty());

    let out_en = scan_candidates(&refs, MatchLang::En, 5, COMMON_SURNAMES, &mut is_known);
    assert!(out_en.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════════════
// Rà ba lớp 2026-08-22 — `context_example` phải mang một TRẦN độ dài, cắt ở biên KÝ TỰ
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_context_example_longer_than_the_limit_is_truncated_at_a_character_boundary_not_a_byte_one() {
    // Một segment DÀI, KHÔNG dấu kết câu ở giữa (mô phỏng đúng ca sinh ra khoảng hở --
    // `split_source_text` vẫn phát một segment cho một đoạn thiếu dấu kết câu) -- toàn chữ
    // Hán NHIỀU BYTE, để một lượt cắt sai biên byte sẽ panic ngay tại test này.
    let mut long_segment = String::new();
    for i in 0..80 {
        // "萧炎" lặp lại nhiều lần TRONG CHÍNH đoạn dài này -- đủ tần suất qua ngưỡng 5, và
        // đủ dài (mỗi vòng lặp thêm ký tự) để tổng độ dài vượt xa CONTEXT_EXAMPLE_CHAR_LIMIT.
        long_segment.push_str(&format!("萧炎在遥远的第{i}处大陆之上继续修炼着"));
    }
    assert!(
        long_segment.chars().count() > CONTEXT_EXAMPLE_CHAR_LIMIT,
        "fixture phai THAT SU dai hon tran de ca nay co y nghia"
    );

    let refs: Vec<&str> = vec![long_segment.as_str()];
    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "萧炎")
        .unwrap_or_else(|| panic!("khong thay `萧炎` trong: {out:?}"));

    let char_count = hit.context_example.chars().count();
    assert!(
        char_count <= CONTEXT_EXAMPLE_CHAR_LIMIT,
        "context_example phai bi cat ve toi da {CONTEXT_EXAMPLE_CHAR_LIMIT} ky tu, nhan {char_count}"
    );
    assert_eq!(
        char_count, CONTEXT_EXAMPLE_CHAR_LIMIT,
        "segment goc dai hon tran nen ban cat phai dung KHOP tran, khong ngan hon"
    );

    // Đối chứng biên: bản cắt phải là ĐÚNG {CONTEXT_EXAMPLE_CHAR_LIMIT} ký tự ĐẦU của segment
    // gốc -- không phải một chuỗi con nào khác, không mất/thêm ký tự nào ở đầu.
    let expected_prefix: String = long_segment
        .chars()
        .take(CONTEXT_EXAMPLE_CHAR_LIMIT)
        .collect();
    assert_eq!(hit.context_example, expected_prefix);

    // `String` hợp lệ tự nó đã chứng minh không cắt giữa một ký tự nhiều byte -- Rust không
    // cho tồn tại một `String` không hợp lệ UTF-8. Khẳng định thêm cho rõ ý test.
    assert!(
        hit.context_example
            .is_char_boundary(hit.context_example.len())
    );
}

#[test]
fn a_context_example_shorter_than_the_limit_is_kept_whole() {
    let segments: Vec<String> = (0..6).map(|i| format!("萧炎在第{i}章出现")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "萧炎")
        .unwrap_or_else(|| panic!("khong thay `萧炎` trong: {out:?}"));
    assert_eq!(
        hit.context_example, "萧炎在第0章出现",
        "segment ngan hon tran phai giu NGUYEN VEN, khong bi cat mot ky tu nao"
    );
}

#[test]
fn multiple_missing_terms_first_seen_in_one_segment_share_the_same_truncated_context_without_changing_output()
 {
    let long_segment = (0..12)
        .map(|i| format!("a witness met Fire Dragon then Ice Phoenix near marker {i}"))
        .collect::<Vec<_>>()
        .join(" and ");
    assert!(long_segment.chars().count() > CONTEXT_EXAMPLE_CHAR_LIMIT);
    let refs = [long_segment.as_str()];
    let mut is_known = nothing_known;

    let out = scan_candidates(&refs, MatchLang::En, 5, COMMON_SURNAMES, &mut is_known);
    let expected_context: String = long_segment
        .chars()
        .take(CONTEXT_EXAMPLE_CHAR_LIMIT)
        .collect();
    let fire = out
        .iter()
        .find(|candidate| candidate.source_term == "Fire Dragon")
        .unwrap_or_else(|| panic!("khong thay Fire Dragon: {out:?}"));
    let ice = out
        .iter()
        .find(|candidate| candidate.source_term == "Ice Phoenix")
        .unwrap_or_else(|| panic!("khong thay Ice Phoenix: {out:?}"));

    assert_eq!(fire.occurrence_count, 12);
    assert_eq!(ice.occurrence_count, 12);
    assert_eq!(fire.context_example, expected_context);
    assert_eq!(ice.context_example, expected_context);
    assert_eq!(fire.context_example, ice.context_example);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 10 — Ngưỡng cấu hình sai (`core::scope::store::parse_glossary_scan_threshold`)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_missing_threshold_key_falls_back_to_the_default() {
    assert_eq!(
        parse_glossary_scan_threshold(None),
        DEFAULT_GLOSSARY_SCAN_THRESHOLD
    );
}

#[test]
fn a_non_numeric_threshold_value_falls_back_to_the_default() {
    assert_eq!(
        parse_glossary_scan_threshold(Some("abc")),
        DEFAULT_GLOSSARY_SCAN_THRESHOLD
    );
}

#[test]
fn a_zero_threshold_value_falls_back_to_the_default() {
    // "0" phan tich DUOC thanh 0u32 nhung bi chan tuong minh -- mot nguong 0 tat het bo loc.
    assert_eq!(
        parse_glossary_scan_threshold(Some("0")),
        DEFAULT_GLOSSARY_SCAN_THRESHOLD
    );
}

#[test]
fn a_negative_threshold_value_falls_back_to_the_default() {
    assert_eq!(
        parse_glossary_scan_threshold(Some("-3")),
        DEFAULT_GLOSSARY_SCAN_THRESHOLD
    );
}

#[test]
fn a_valid_threshold_value_parses_through_unchanged() {
    assert_eq!(parse_glossary_scan_threshold(Some("12")), 12);
}

// ═════════════════════════════════════════════════════════════════════════════════
// P7 (vòng rà bốn lớp, 2026-08-27) — `core::scope::store::resolve_library_root_value`
// (Story 5.3) KHÔNG một ca nào, dù doc-comment của chính nó tự khai "hàm thuần, đây là thứ
// test gọi" và "chép khuôn `parse_glossary_scan_threshold`". Đặt CÙNG TỆP với hàm anh em đó
// (`parse_glossary_scan_threshold`, ngay trên) theo đúng chỉ dẫn P7 — KHÔNG phải một hàng
// của §I/O Matrix Story 3.5 mà tệp này sở hữu, chỉ mượn cùng mái nhà vì lý do đó.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_missing_library_root_value_resolves_to_not_configured() {
    assert_eq!(resolve_library_root_value(None), None);
}

#[test]
fn an_empty_library_root_value_resolves_to_not_configured() {
    assert_eq!(resolve_library_root_value(Some("")), None);
}

#[test]
fn a_whitespace_only_library_root_value_resolves_to_not_configured() {
    assert_eq!(resolve_library_root_value(Some("   \t  ")), None);
}

#[test]
fn a_real_library_root_value_is_trimmed_and_kept() {
    assert_eq!(
        resolve_library_root_value(Some("  /tmp/thu-vien  ")),
        Some("/tmp/thu-vien".to_owned())
    );
}

#[test]
fn a_real_library_root_value_without_surrounding_whitespace_passes_through_unchanged() {
    assert_eq!(
        resolve_library_root_value(Some("/tmp/thu-vien")),
        Some("/tmp/thu-vien".to_owned())
    );
}
