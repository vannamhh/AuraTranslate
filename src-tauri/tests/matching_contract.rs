//! Hợp đồng HÀNH VI của `core::matching` — Story 1.12 (AD-17).
//!
//! ⚠️ Tệp riêng có chủ ý, đúng cặp tên đã dùng cho `store_*` và `scope_*`: đây là phép
//! nghiệm thu **hành vi lúc chạy**; `matching_boundary.rs` là phép kiểm **tĩnh trên cây
//! nguồn**. Trộn hai thứ là làm hỏng đúng thứ khiến cả hai đọc được.
//!
//! 🔴 **Không một tệp dữ liệu ngoài nào.** Mọi ca dựng chuỗi ngay tại chỗ, nên
//! **100%** ca ở đây chạy được trong CI — khác Story 1.11/1.11b, nơi phần đắt nhất phải
//! `#[ignore]` vì cần một tệp `.db` thật.
//!
//! 🔴 **Mọi chuỗi stem trong tệp này là ĐẦU RA THẬT của `english_porter_2`**, chép ra từ
//! một lượt chạy, **không** phải một chuỗi chép từ mô tả kinh điển của Porter. Đó là
//! yêu cầu tường minh của AC7 — và nó đã bắt được một ca ngay ở lượt dựng: `happiest`
//! **không** về `happi` (xem
//! [`stemming_is_not_lemmatization_irregular_and_comparative_forms_never_reach_their_lemma`]).

use auratranslate_lib::core::matching::{
    MatchLang, MatchToken, TermMatch, find_terms, ngrams, normalize, tokenize,
};

/// Span của mọi token luôn là một cặp ranh giới UTF-8 hợp lệ, và cắt ra đúng `text`.
///
/// ⚠️ Khẳng định bằng `text.get(span)` chứ không bằng `&text[span]`: phép cắt bằng
/// ngoặc **panic** ở biên sai, và một panic đọc thành *"test hỏng"* chứ không thành
/// *"span sai"*.
fn assert_spans_are_valid(text: &str, tokens: &[MatchToken<'_>]) {
    for token in tokens {
        let slice = text.get(token.span.clone());
        assert_eq!(
            slice,
            Some(token.text),
            "span {:?} của token {:?} không cắt lại đúng token đó trong {text:?}. Một \
             span không rơi vào ranh giới UTF-8 trả `None` ở đây, và nó sẽ PANIC ở \
             Story 3.4 nơi văn bản bị cắt bằng `&text[span]` để tô màu.",
            token.span,
            token.text
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC5 · AC6 — tokenize hai đường
// ═════════════════════════════════════════════════════════════════════════════════

/// **AC5** — đường `Zh` đi qua jieba và trả span byte **đã có sẵn** từ crate.
#[test]
fn chinese_tokenization_returns_byte_spans_into_the_original_text() {
    let text = "我喜欢中国人的文化";
    let tokens = tokenize(text, MatchLang::Zh);

    assert_spans_are_valid(text, &tokens);
    assert_eq!(
        tokens.iter().map(|t| t.text).collect::<Vec<_>>(),
        ["我", "喜欢", "中国", "人", "的", "文化"],
        "phép tách của jieba đã đổi. Đây là đầu ra ĐO THẬT của `jieba-rs` 0.10.3 với dict \
         mặc định và `hmm = false` (2026-08-05). Nếu ca này đỏ thì hoặc phiên bản crate \
         đổi, hoặc cờ `HMM` của module đã bị đổi — và cờ đó đổi kết quả khớp của CẢ \
         Glossary lẫn TM cùng lúc (AD-17)."
    );

    // Ranh giới byte, không phải ranh giới ký tự: mỗi chữ Hán ở đây là 3 byte.
    assert_eq!(tokens[0].span, 0..3);
    assert_eq!(tokens[2].span, 9..15, "`中国` là 2 ký tự nhưng 6 BYTE");
}

/// **AC6** — đường `En` tách theo `char::is_alphanumeric` của `std`, không crate mới.
#[test]
fn english_tokenization_splits_on_non_alphanumeric_and_keeps_byte_spans() {
    let text = "The running dogs, ok?";
    let tokens = tokenize(text, MatchLang::En);

    assert_spans_are_valid(text, &tokens);
    assert_eq!(
        tokens
            .iter()
            .map(|t| (t.text, t.span.clone()))
            .collect::<Vec<_>>(),
        [
            ("The", 0..3),
            ("running", 4..11),
            ("dogs", 12..16),
            ("ok", 18..20)
        ],
        "dấu câu và khoảng trắng là DẤU TÁCH và không vào token nào; token mang dạng \
         GỐC (`The` viết hoa), chưa chuẩn hoá."
    );
}

/// **Task 2.4** — ba ca biên không panic, và span vẫn hợp lệ ở cả hai đường.
#[test]
fn tokenization_survives_empty_punctuation_only_and_mixed_script_input() {
    for lang in [MatchLang::Zh, MatchLang::En] {
        for text in ["", "   ", "!!!, ... ？？", "abc中国def", "中国，很大"] {
            let tokens = tokenize(text, lang);
            assert_spans_are_valid(text, &tokens);
        }
    }

    assert!(
        tokenize("", MatchLang::Zh).is_empty(),
        "chuỗi rỗng ⇒ không token nào"
    );
    assert!(
        tokenize("", MatchLang::En).is_empty(),
        "chuỗi rỗng ⇒ không token nào"
    );
    assert!(
        tokenize("!!!, ...", MatchLang::En).is_empty(),
        "chuỗi toàn dấu câu ⇒ đường `En` không token nào (mọi ký tự đều là dấu tách)"
    );

    // Chuỗi lẫn Hán + Latin: đường `Zh` giữ CẢ HAI, và span vẫn cắt đúng. `中国` là một
    // từ trong dict mặc định nên nó ra một token; `abc`/`def` ra nguyên khối.
    let mixed = tokenize("abc中国def", MatchLang::Zh);
    assert_eq!(
        mixed.iter().map(|t| t.text).collect::<Vec<_>>(),
        ["abc", "中国", "def"]
    );
}

/// 🔴 **Vá lúc code review (2026-08-05)** — đường `En` giới hạn về ASCII, không dùng
/// `char::is_alphanumeric` Unicode-rộng: bản trước dính chữ Hán/script khác vào token
/// tiếng Anh liền kề, làm `"hello世界world"` thành MỘT token vô nghĩa. Ca này khoá cả
/// hành vi ĐÚNG (tách theo script) lẫn đánh đổi ĐÃ CHẤP NHẬN (chữ Latin có dấu bị cắt).
#[test]
fn english_tokenization_is_ascii_only_and_never_fuses_other_scripts() {
    let tokens = tokenize("hello世界world", MatchLang::En);
    assert_eq!(
        tokens.iter().map(|t| t.text).collect::<Vec<_>>(),
        ["hello", "world"],
        "chữ Hán phải là DẤU TÁCH ở đường `En`, không được dính vào token liền kề"
    );

    // ⚠️ Đánh đổi ĐÃ CHẤP NHẬN (quyết định lúc code review): giới hạn ASCII cắt sai chữ
    // Latin có dấu — `"café"` ra `"caf"` chứ không phải `"café"` nguyên vẹn. Ghi lại
    // TƯỜNG MINH bằng một ca có tên, không phải một câu trong doc-comment, đúng khuôn
    // đã dùng cho giới hạn `happiest` của AC8.
    let tokens = tokenize("café", MatchLang::En);
    assert_eq!(
        tokens.iter().map(|t| t.text).collect::<Vec<_>>(),
        ["caf"],
        "đánh đổi đã biết: giới hạn ASCII cắt `é` ra khỏi token — nếu ca này đỏ vì \
         `\"café\"` đi nguyên vẹn thì giới hạn ASCII đã bị gỡ, đọc lại doc-comment của \
         `tokenize` trước khi đổi"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC6 · AC7 — chuẩn hoá
// ═════════════════════════════════════════════════════════════════════════════════

/// **AC5 vế chuẩn hoá** — `Zh` là phép ĐỒNG NHẤT, và đó là hành vi đúng chứ không
/// phải một chỗ chưa làm: chữ Hán không có hình thái từ để chuẩn hoá.
#[test]
fn chinese_normalization_is_the_identity_because_han_has_no_inflection() {
    for token in ["中国", "文化", "萧炎", "中國", "𠧜"] {
        assert_eq!(normalize(token, MatchLang::Zh), token);
    }
}

/// 🔴 **AC7** — biến thể hình thái tiếng Anh về **cùng một** dạng chuẩn hoá với dạng gốc.
///
/// ⚠️ Mọi chuỗi bên phải là **đầu ra thật** của `english_porter_2`, chép ra từ một lượt
/// chạy — **không** đoán trước rồi bắt hàm khớp phỏng đoán (AC7 nói thẳng điều này).
/// `dictionary ⇒ dictionari` và `study ⇒ studi` **không** phải lỗi chính tả: Porter2
/// đổi `y` cuối thành `i` và không dựng lại một từ có thật.
#[test]
fn english_normalization_maps_inflected_forms_onto_their_base_form() {
    // Đầu ra thật, đo 2026-08-05.
    for (input, expected) in [
        ("running", "run"),
        ("run", "run"),
        ("dogs", "dog"),
        ("dog", "dog"),
        ("studies", "studi"),
        ("study", "studi"),
        ("dictionary", "dictionari"),
        ("happy", "happi"),
    ] {
        assert_eq!(
            normalize(input, MatchLang::En),
            expected,
            "`english_porter_2({input:?})` không còn cho {expected:?}"
        );
    }

    // 🔴 Đây LÀ cơ chế của FR40: hai vế đi qua CÙNG MỘT phép chuẩn hoá rồi gặp nhau.
    for (variant, base) in [("running", "run"), ("dogs", "dog"), ("studies", "study")] {
        assert_eq!(
            normalize(variant, MatchLang::En),
            normalize(base, MatchLang::En),
            "{variant:?} và {base:?} phải gặp nhau ở dạng chuẩn hoá — đó là toàn bộ cơ \
             chế nhận diện biến thể, không có một bảng biến thể viết tay nào."
        );
    }
}

/// 🔴 **AC6 vế thứ tự** — hạ chữ thường TRƯỚC, rồi mới stem.
///
/// Crate nói thẳng: *"Tokens are expected to be lowercased beforehand"*. Sai thứ tự ⇒
/// `Running` và `running` cho hai stem khác nhau ⇒ đúng lỗ chữ HOA mà AD-44 ③ vừa bịt ở
/// đường tra cứu, tái sinh ở đường khớp.
#[test]
fn english_normalization_lowercases_before_stemming_so_case_never_splits_a_term() {
    for variant in ["running", "Running", "RUNNING", "RuNnInG"] {
        assert_eq!(
            normalize(variant, MatchLang::En),
            "run",
            "{variant:?} không về `run` — nghi phạm: stem chạy TRƯỚC khi hạ chữ thường."
        );
    }
}

/// **Task 3.5** — phép hạ chữ thường là `str::to_lowercase` của Rust, **không** phụ
/// thuộc locale.
///
/// ⚠️ AD-44 ③ đã trả giá cho bài học này một lần: một phép fold theo locale cho **cùng
/// một đầu vào hai kết quả trên hai máy** cài ngôn ngữ hệ điều hành khác nhau — một hồi
/// quy không tái lập được trên máy người sửa. Ca kinh điển là `"I"`: trong locale
/// tiếng Thổ, phép hạ chữ thường theo locale cho `"ı"` (dotless i) chứ không cho
/// `"i"`.
#[test]
fn english_lowercasing_is_locale_independent() {
    assert_eq!(
        normalize("I", MatchLang::En),
        "i",
        "`\"I\"` phải cho `\"i\"` trên MỌI máy. Nếu nó cho `\"ı\"` (dotless i) thì phép \
         hạ chữ thường đang đi qua locale của hệ điều hành."
    );
    assert_eq!(normalize("İ", MatchLang::En), "i̇");
}

/// 🔴 **AC8** — giới hạn *stemming ≠ lemmatization* là một **ca test có tên**, không
/// phải một câu trong doc-comment.
///
/// FR40 tuyên bố giới hạn này (`epics.md:156`). Ca này **đỏ** vào ngày ai đó đổi sang
/// một lemmatizer — và lúc đó người sửa **phải** đọc lý do trước khi đổi con số.
///
/// 🔴 `happiest` là một phát hiện **đo được của chính story 1.12**, không nằm trong
/// danh sách bất quy tắc mà AC8 liệt kê: Porter2 **không** có luật cho hậu tố so
/// sánh/cực cấp (`-er` · `-est`), nên một biến thể **có quy tắc** cũng không về được
/// dạng gốc. Nó đứng chung hàng với `went`/`mice`, không phải một lỗi cài đặt.
#[test]
fn stemming_is_not_lemmatization_irregular_and_comparative_forms_never_reach_their_lemma() {
    for (variant, lemma) in [
        ("went", "go"),
        ("gone", "go"),
        ("children", "child"),
        ("mice", "mouse"),
        ("better", "good"),
        ("happiest", "happy"),
    ] {
        assert_ne!(
            normalize(variant, MatchLang::En),
            normalize(lemma, MatchLang::En),
            "{variant:?} và {lemma:?} ĐÃ gặp nhau ở dạng chuẩn hoá.\n\n\
             Nếu bạn vừa đổi `core::matching` sang một LEMMATIZER thì ca này đỏ ĐÚNG Ý — \
             nhưng đọc trước khi đổi con số: FR40 (`epics.md:156`) tuyên bố giới hạn này \
             là *stemming, KHÔNG phải lemmatization*, và AD-44 ③ đo được rằng đường tra \
             cứu từ điển không cần nó (16/16 mẫu thử đã có sẵn mọi biến thể làm đầu mục \
             riêng). Đổi thuật toán là đổi kết quả khớp của CẢ Glossary lẫn TM cùng lúc, \
             và NFR15 đòi rà giấy phép TRƯỚC khi thêm bất kỳ phụ thuộc mới nào."
        );
    }

    // Đầu ra thật, đo 2026-08-05 — chép ra từ một lượt chạy.
    assert_eq!(normalize("mice", MatchLang::En), "mice");
    assert_eq!(normalize("mouse", MatchLang::En), "mous");
    assert_eq!(normalize("happiest", MatchLang::En), "happiest");
    assert_eq!(normalize("happy", MatchLang::En), "happi");
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC5 · AC6 — n-gram
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC5** — `Zh` là n-gram **KÝ TỰ**, cửa sổ trượt theo ký tự chứ không theo token.
#[test]
fn chinese_ngrams_slide_over_characters_not_over_tokens() {
    assert_eq!(ngrams("中國人", MatchLang::Zh, 2), ["中國", "國人"]);
    assert_eq!(ngrams("中國人", MatchLang::Zh, 1), ["中", "國", "人"]);
    assert_eq!(ngrams("中國人", MatchLang::Zh, 3), ["中國人"]);

    // ⚠️ Đối chứng: jieba cắt `我喜欢` thành `我` + `喜欢`, nhưng n-gram ký tự KHÔNG
    // biết tới ranh giới đó — `epics.md` §Story 7.6: *"n-gram ký tự — không có ranh giới từ"*.
    assert_eq!(
        ngrams("我喜欢", MatchLang::Zh, 2),
        ["我喜", "喜欢"],
        "`我喜` cắt NGANG ranh giới token của jieba, và đó là hành vi ĐÚNG cho Story 7.6."
    );
}

/// 🔴 **Bẫy đắt nhất của Story 1.11** — phép đếm là `chars().count()`, không `len()`.
///
/// `"中國"` là **2 ký tự** nhưng **6 byte**. Một cửa sổ trượt theo byte không chỉ trả
/// sai — nó **panic** ở một biên không phải ranh giới UTF-8.
#[test]
fn chinese_ngram_population_is_counted_in_characters_never_in_bytes() {
    // `"中國"` có 2 ký tự ⇒ `n = 3` vượt quần thể ⇒ RỖNG. Đếm bằng `len()` sẽ thấy 6 và
    // cố cắt ba lát 1 byte giữa một ký tự 3 byte.
    assert!(
        ngrams("中國", MatchLang::Zh, 3).is_empty(),
        "`\"中國\"` có 2 KÝ TỰ (6 byte). `n = 3` phải trả rỗng — nếu ca này panic thì phép \
         đếm đang chạy trên `len()`."
    );
    assert_eq!(ngrams("中國", MatchLang::Zh, 2), ["中國"]);
}

/// **Task 4.4** — ca đối chứng sống bằng chữ Hán **ngoài BMP**.
///
/// `𠧜` (U+209DC) là **4 byte** UTF-8 và **2 đơn vị** UTF-16. Nó đã là ca thật một lần:
/// một định nghĩa `is_han` chỉ-BMP đọc nó thành *"không phải chữ Hán"* (Story 1.11b).
#[test]
fn chinese_ngrams_handle_characters_outside_the_basic_multilingual_plane() {
    assert_eq!(ngrams("𠧜中𠧜", MatchLang::Zh, 2), ["𠧜中", "中𠧜"]);
    assert_eq!(
        "𠧜".len(),
        4,
        "tiền đề của ca này: `𠧜` là 4 byte, không phải 3"
    );
    assert_eq!("𠧜中𠧜".chars().count(), 3);
}

/// **AC6** — `En` là **token** n-gram, và cửa sổ trượt trên danh sách token **ĐÃ STEM**.
#[test]
fn english_ngrams_slide_over_stemmed_tokens_not_over_the_raw_string() {
    assert_eq!(
        ngrams("the running dogs", MatchLang::En, 2),
        ["the run", "run dog"],
        "cửa sổ phải trượt trên danh sách token ĐÃ STEM (`the` · `run` · `dog`), không \
         trên chuỗi gốc — `epics.md` §Story 7.6."
    );
    assert_eq!(
        ngrams("The Running Dogs!", MatchLang::En, 3),
        ["the run dog"]
    );
}

/// **Task 4.3** — ba ca biên trả **rỗng**, không panic, không n-gram cụt.
#[test]
fn ngrams_return_empty_for_zero_n_empty_text_and_n_larger_than_the_population() {
    for lang in [MatchLang::Zh, MatchLang::En] {
        assert!(ngrams("中国 dogs", lang, 0).is_empty(), "`n = 0` ⇒ rỗng");
        assert!(ngrams("", lang, 1).is_empty(), "chuỗi rỗng ⇒ rỗng");
        assert!(ngrams("", lang, 5).is_empty(), "chuỗi rỗng ⇒ rỗng");
        assert!(
            ngrams("中国", lang, 99).is_empty(),
            "`n` lớn hơn quần thể ⇒ RỖNG, không phải một n-gram cụt. Một phần tử ngắn \
             hơn `n` là thứ chỗ gọi không phân biệt được với một n-gram thật ⇒ một lỗi \
             đếm im lặng ở Story 7.6."
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC2 · AC5 · AC6 · AC7 — find_terms
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC7 đầu-cuối** — một biến thể hình thái trong văn bản khớp thuật ngữ ở dạng gốc.
#[test]
fn an_english_inflected_form_in_the_text_matches_a_base_form_term() {
    let text = "The running dogs are here";
    let found = find_terms(text, &["run", "dog"], MatchLang::En);

    assert_eq!(
        found,
        [
            TermMatch {
                term_index: 0,
                span: 4..11
            },
            TermMatch {
                term_index: 1,
                span: 12..16
            },
        ]
    );

    // 🔴 Task 5.5 — span dùng được THẬT: nó trỏ vào chuỗi GỐC, không vào chuỗi đã
    // chuẩn hoá. Đây là đúng cụm người dùng sẽ thấy tô màu ở Story 3.4.
    assert_eq!(&text[found[0].span.clone()], "running");
    assert_eq!(&text[found[1].span.clone()], "dogs");
}

/// **Task 5.1** — thuật ngữ nhiều từ khớp một dãy token liền nhau; span gồm cả khoảng
/// giữa chúng trong văn bản gốc.
#[test]
fn a_multi_word_english_term_matches_a_run_of_adjacent_tokens() {
    let text = "I saw the running dogs today";
    let found = find_terms(text, &["running dog"], MatchLang::En);

    assert_eq!(found.len(), 1);
    assert_eq!(&text[found[0].span.clone()], "running dogs");
}

/// **Task 5.4** — span trỏ vào chuỗi GỐC kể cả khi văn bản có ký tự **không phải
/// ASCII** đứng trước.
///
/// 🔴 Đây là ca mà một span đo trên chuỗi đã hạ chữ thường sẽ **không** đỏ nếu văn bản
/// thuần ASCII: `"Café"` dài 4 ký tự nhưng **5 byte**. Một lỗi lệch offset đi trọn bộ
/// test tiếng Anh thuần mà không đỏ một ca nào — nên nó phải có ca riêng.
#[test]
fn english_match_spans_point_into_the_original_text_even_after_non_ascii_bytes() {
    let text = "Café — the running dogs";
    let found = find_terms(text, &["run"], MatchLang::En);

    assert_eq!(found.len(), 1);
    assert_eq!(
        &text[found[0].span.clone()],
        "running",
        "span lệch ⇒ nghi phạm: offset đo trên chuỗi đã chuẩn hoá chứ không trên chuỗi \
         gốc. `\"Café\"` là 4 ký tự nhưng 5 byte, và `\"—\"` là 3 byte."
    );
}

/// 🔴 **AC5** — `Zh` là **khớp chính xác**, và ranh giới token của jieba là thứ phân xử.
///
/// ⚠️ Cả ba hàng dưới đây là **số đo** (`jieba-rs` 0.10.3, dict mặc định, `hmm = false`,
/// 2026-08-05), không phải trực giác. Xem doc-comment của `find_terms`.
#[test]
fn chinese_term_matching_is_exact_and_arbitrated_by_jieba_token_boundaries() {
    // ✅ Giản thể: jieba cắt `中国人` ⇒ `中国` · `人`. Cả hai đầu của `中国` là ranh giới
    // ⇒ NHẬN. jieba tự nói `中国` là một từ ở đây.
    let simplified = "我喜欢中国人的文化";
    let found = find_terms(simplified, &["中国"], MatchLang::Zh);
    assert_eq!(found.len(), 1);
    assert_eq!(&simplified[found[0].span.clone()], "中国");

    // ✅ Phồn thể: dict mặc định của jieba là GIẢN THỂ, nên `中國人` rơi ra từng ký tự
    // ⇒ mọi biên đều là ranh giới ⇒ NHẬN.
    let traditional = "我喜歡中國人的文化";
    let found = find_terms(traditional, &["中國"], MatchLang::Zh);
    assert_eq!(found.len(), 1);
    assert_eq!(&traditional[found[0].span.clone()], "中國");

    // Thuật ngữ CẮT NGANG một từ jieba đã nhận diện ⇒ TỪ CHỐI. jieba cắt `文化` thành
    // MỘT token, nên `文` bắt đầu ở ranh giới nhưng KẾT ở giữa token.
    assert!(
        find_terms("我喜欢中国人的文化", &["文"], MatchLang::Zh).is_empty(),
        "`文` cắt ngang token `文化` và phải bị TỪ CHỐI. Tô màu nửa token là nói với \
         người dịch rằng thuật ngữ của họ có mặt ở một chỗ nó không có mặt."
    );
}

/// **Task 5.2** — một tên riêng không có trong từ điển jieba **vẫn** khớp.
///
/// 🔴 Đây là ca biện hộ cho `HMM = false`: thứ Glossary chứa nhiều nhất trong dịch
/// truyện là danh từ riêng không có trong từ điển. Với `hmm = false` chúng rơi ra
/// **từng ký tự** ⇒ luôn nằm gọn trong một dãy token liền nhau ⇒ luôn khớp.
#[test]
fn a_chinese_proper_noun_absent_from_the_jieba_dictionary_still_matches() {
    let text = "萧炎和林动一起走了";
    let found = find_terms(text, &["萧炎", "林动"], MatchLang::Zh);

    assert_eq!(found.len(), 2);
    assert_eq!(&text[found[0].span.clone()], "萧炎");
    assert_eq!(&text[found[1].span.clone()], "林动");
}

/// **AC2 vế hình dạng** — hàm dùng được từ `core::glossary`/`core::tm` mà không cần
/// một lớp bọc nào: nhận `&str` + `MatchLang`, trả dữ liệu thuần.
///
/// ⚠️ `term_index` trỏ vào lát của **chỗ gọi**, không phải một id Glossary — điều
/// kiện để module là **lá** trong đồ thị phụ thuộc (AD-13).
#[test]
fn term_matches_carry_the_callers_own_index_not_a_domain_identifier() {
    let text = "the dogs and the running dog";
    let found = find_terms(text, &["cat", "dog"], MatchLang::En);

    assert!(
        found.iter().all(|m| m.term_index == 1),
        "mọi lượt khớp phải trỏ về `terms[1]` (`\"dog\"`); `\"cat\"` không có mặt"
    );
    assert_eq!(found.len(), 2);
}

/// **Task 5.6 · thứ tự tất định** — kết quả sắp theo vị trí, không theo thứ tự lát
/// `terms`.
#[test]
fn term_matches_come_back_in_a_deterministic_position_order() {
    let text = "dogs run and cats run";
    let found = find_terms(text, &["run", "cat", "dog"], MatchLang::En);

    let spans: Vec<_> = found.iter().map(|m| m.span.start).collect();
    let mut sorted = spans.clone();
    sorted.sort_unstable();
    assert_eq!(
        spans, sorted,
        "hai lượt chạy trên cùng đầu vào không được cho hai thứ tự tô màu khác nhau"
    );
    assert_eq!(found.len(), 4);
}

/// Ca biên của `find_terms`: văn bản rỗng · lát `terms` rỗng · thuật ngữ rỗng · thuật
/// ngữ chỉ gồm dấu tách ⇒ **rỗng**, không panic.
#[test]
fn find_terms_never_matches_an_empty_or_separator_only_term() {
    for lang in [MatchLang::Zh, MatchLang::En] {
        assert!(find_terms("", &["dog", "中国"], lang).is_empty());
        assert!(find_terms("dogs 中国", &[], lang).is_empty());
        assert!(find_terms("dogs 中国", &[""], lang).is_empty());
    }
    assert!(
        find_terms("dogs and cats", &["   "], MatchLang::En).is_empty(),
        "một thuật ngữ chỉ gồm dấu tách sinh 0 token và không bao giờ được khớp — nếu \
         nó khớp thì nó khớp ở MỌI vị trí."
    );

    // 🔴 Vá lúc code review (2026-08-05): nhánh `Zh` trước đây chỉ chặn `term.is_empty()`,
    // không chặn thuật ngữ chỉ gồm khoảng trắng — lệch với nhánh `En` ở trên và với
    // chính lời hứa của doc-comment `find_terms`. jieba giữ khoảng trắng làm token riêng
    // (`hmm = false`), nên một thuật ngữ toàn khoảng trắng có thể khớp đúng token đó nếu
    // không có cổng chặn.
    assert!(
        find_terms("中国 人", &[" "], MatchLang::Zh).is_empty(),
        "một thuật ngữ chỉ gồm khoảng trắng không bao giờ được khớp ở đường `Zh`, kể cả \
         khi jieba tách khoảng trắng đó thành một token riêng"
    );
}

/// 🔴 **Vá lúc code review (2026-08-05)** — một thuật ngữ nhiều từ **không** được nối
/// hai token nằm ở hai câu khác nhau, dù chúng là hai token liền kề trong danh sách token.
///
/// Trước lượt vá này, `tokenize` (En) coi dấu chấm câu và khoảng trắng là dấu tách giống
/// hệt nhau, nên `find_terms` có thể nối `"fast"` ở cuối câu 1 với `"Dog"` ở đầu câu 2
/// thành một lượt khớp giả cho thuật ngữ `"fast dog"`.
#[test]
fn english_multi_word_terms_never_join_across_a_sentence_boundary() {
    let text = "The wolf ran fast. Dog barked loudly.";
    assert!(
        find_terms(text, &["fast dog"], MatchLang::En).is_empty(),
        "\"fast\" và \"Dog\" thuộc hai câu khác nhau (ngăn bởi dấu chấm) — không được \
         khớp thành một cụm \"fast dog\", dù chúng là hai token liền kề"
    );

    // ⚠️ Đối chứng: cùng hai từ, cùng một câu ⇒ VẪN khớp bình thường — lượt vá này chỉ
    // chặn nối XUYÊN câu, không chặn khớp nhiều từ nói chung (Task 5.1 vẫn đứng).
    let found = find_terms("The wolf ran fast dog today", &["fast dog"], MatchLang::En);
    assert_eq!(found.len(), 1, "cùng câu thì vẫn phải khớp bình thường");
}

/// Hai lượt xuất hiện **chồng nhau** của cùng một thuật ngữ đều là lượt xuất hiện thật.
#[test]
fn overlapping_occurrences_of_the_same_chinese_term_are_all_reported() {
    // jieba cắt `𠧜𠧜𠧜` ra từng ký tự (không có trong dict) ⇒ mọi biên là ranh giới.
    // `𠧜` là 4 byte, nên ca này cũng là một phép kiểm sống rằng phép nhích của vòng tìm
    // đi theo RANH GIỚI UTF-8 chứ không theo byte.
    let text = "𠧜𠧜𠧜";
    let found = find_terms(text, &["𠧜𠧜"], MatchLang::Zh);

    assert_eq!(
        found.iter().map(|m| m.span.clone()).collect::<Vec<_>>(),
        [0..8, 4..12],
        "`𠧜𠧜` xuất hiện HAI lần chồng nhau trong `𠧜𠧜𠧜`"
    );
}
