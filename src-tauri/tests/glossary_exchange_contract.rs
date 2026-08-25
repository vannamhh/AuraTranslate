//! Hợp đồng Story 3.10 — xuất/nhập Glossary qua CSV/TSV. §I/O & Edge-Case Matrix của spec
//! `3-10-xuat-va-nhap-glossary-qua-csv-tsv.md`, TRỌN.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `glossary_contract.rs` (Story 3.1-3.9) — một tệp, một
//! story. Phép kiểm **tĩnh trên cây nguồn** (token xuất xứ tự động, tên bảng) sống ở
//! `glossary_boundary.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HAI NỬA CỦA TỆP NÀY
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Định dạng THUẦN** (`core::glossary::exchange`) — `render_tier`/`parse`/`classify`,
//!    không chạm `Store`. Dựng dữ liệu bằng `GlossaryEntry { .. }` trực tiếp, không cần một
//!    thư mục tạm nào.
//! 2. **Đường ghi** (`core::glossary::store::export_tier`/`import_into_tier`) — cần `Store`,
//!    kế thừa bốn luật của `glossary_contract.rs` (mỗi ca một thư mục tạm, Drop trước khi
//!    xoá, không `sleep` dài, không ca nào treo khi trượt).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! CA VÒNG TRÒN LÀ CA TRUNG TÂM — xem `round_trip_preserves_five_user_visible_fields_...`
//! ─────────────────────────────────────────────────────────────────────────────
//! §Verification của spec đòi một phép đối chứng GỠ chỗ nối: gỡ lớp bọc nháy kép trong
//! `render_field`/`quote_field` (`exchange.rs`) rồi chạy lại CHÍNH ca này — nó phải ĐỎ. Đây
//! là phép kiểm thủ công (không chạy trong CI), ghi vào §Spec Change Log của spec sau khi
//! chạy.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::core::glossary::{
    Category, ConflictDecision, Delimiter, GlossaryEntry, GlossaryError, GlossaryTier,
    ImportRow, ParseIssue, RowPlan, RowPlanKind, TermOrigin, add_manual_term, classify,
    delete_manual_term, export_tier, import_into_tier, load_tier, parse, render_tier,
    update_manual_term,
};
use auratranslate_lib::core::i18n::{IpcError, MessageKey};
use auratranslate_lib::core::store::{Store, StoreSpec};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-glossary-exchange-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn open_global(dir: &Path) -> Store {
    Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db")
}

fn open_project(dir: &Path) -> Store {
    Store::open(StoreSpec::project(dir.join("project.db"))).expect("mo project.db")
}

fn entry(id: i64, source_term: &str, translation: Option<&str>) -> GlossaryEntry {
    GlossaryEntry {
        id,
        source_term: source_term.to_owned(),
        translation: translation.map(str::to_owned),
        note: String::new(),
        category: Category::Other,
        term_origin: TermOrigin::Manual,
        created_at: "2026-08-24T00:00:00.000Z".to_owned(),
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// XUẤT — render_tier
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix "Xuất một tầng" — hàng tiêu đề + N hàng, 6 cột, KHÔNG cột `id`.
#[test]
fn exporting_a_tier_produces_a_header_plus_six_columns_per_entry_without_an_id_column() {
    let mut tier = BTreeMap::new();
    tier.insert("apple".to_owned(), entry(1, "apple", Some("qua tao")));
    tier.insert("banana".to_owned(), entry(2, "banana", None));

    let text = render_tier(&tier, Delimiter::Csv);
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(
        lines[0], "source_term,translation,note,category,term_origin,created_at",
        "hang tieu de phai dung SAU cot, KHONG DAU (dinh danh may doc), va KHONG cot id"
    );
    assert_eq!(lines.len(), 3, "mot hang tieu de + hai hang du lieu");
    assert_eq!(lines[1], "apple,qua tao,,other,manual,2026-08-24T00:00:00.000Z");
    assert_eq!(lines[2], "banana,,,other,manual,2026-08-24T00:00:00.000Z");
    assert!(!text.contains(",1,") && !text.contains(",2,"), "id khong duoc xuat hien tren dong du lieu");
}

/// I/O Matrix "Xuất tầng rỗng" — CHỈ hàng tiêu đề, một tệp CÓ tiêu đề nói "rỗng", khác một
/// tệp trống hoàn toàn không nói gì.
#[test]
fn exporting_an_empty_tier_produces_only_the_header_row() {
    let tier: BTreeMap<String, GlossaryEntry> = BTreeMap::new();
    let text = render_tier(&tier, Delimiter::Tsv);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1, "chi mot hang -- hang tieu de");
    assert_eq!(lines[0], "source_term\ttranslation\tnote\tcategory\tterm_origin\tcreated_at");
}

/// I/O Matrix "Trường chứa dấu phân cách / nháy kép / xuống dòng" — bọc nháy kép, nháy kép
/// nhân đôi (RFC 4180), áp cho **cả** CSV và TSV.
#[test]
fn fields_needing_escaping_are_quoted_and_doubled_for_both_delimiters() {
    for delimiter in [Delimiter::Csv, Delimiter::Tsv] {
        let mut tier = BTreeMap::new();
        let mut e = entry(1, "term", Some("a,b\"c\nd"));
        e.note = "note\twith\ttabs".to_owned();
        tier.insert("term".to_owned(), e);

        let text = render_tier(&tier, delimiter);
        let data_line_start = text.find('\n').expect("phai co it nhat mot dong xuong hang") + 1;
        let data = &text[data_line_start..];

        assert!(
            data.contains("\"a,b\"\"c\nd\""),
            "truong translation chua dau phay/nhay kep/xuong dong phai duoc BOC va nhay kep \
             NHAN DOI (delimiter={delimiter:?}). Du lieu: {data:?}"
        );
        if delimiter == Delimiter::Tsv {
            assert!(
                data.contains("\"note\twith\ttabs\""),
                "TSV theo le khong boc, nhung mot ky tu Tab dan vao mot o TU DO (note) phai \
                 van duoc boc -- day chinh la thu §Design Notes cua spec goi la 'mot duong \
                 boc dung chung cho ca hai dau phan cach'"
            );
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mục ① (vòng rà ba lớp, cụm B 2026-08-25) — vô hiệu hoá công thức (CSV/TSV injection)
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix "①" — một ô BẮT ĐẦU bằng `=`/`+`/`-`/`@` phải được xuất kèm một tiền tố vô
/// hiệu hoá công thức, và nhập lại CHÍNH TỆP ĐÓ bằng `parse` của kho phải trả về đúng TỪNG
/// BYTE giá trị GỐC (không còn tiền tố).
///
/// 🔴 **CA ĐỐI CHỨNG BẮT BUỘC (thủ công): làm `needs_formula_guard` luôn trả `false` rồi
/// chạy lại ca này — nó PHẢI ĐỎ** (ô xuất ra trần `=1+1`, một bảng tính thật sẽ chạy nó như
/// công thức).
#[test]
fn a_cell_starting_with_a_formula_trigger_character_is_neutralized_on_export_and_recovered_verbatim_on_import()
 {
    for trigger in ["=1+1", "+A1", "-A1", "@SUM(A1:A2)"] {
        let mut tier = BTreeMap::new();
        tier.insert("term".to_owned(), entry(1, "term", Some(trigger)));

        let text = render_tier(&tier, Delimiter::Csv);
        let data_line = text.lines().nth(1).expect("phai co mot hang du lieu");
        let fields: Vec<&str> = data_line.split(',').collect();
        assert!(
            fields[1].starts_with('\''),
            "o BAT DAU bang ky tu cong thuc ({trigger:?}) phai duoc xuat kem tien to vo hieu \
             hoa vo hieu hoa `'`. Dong: {data_line:?}"
        );
        assert_eq!(
            &fields[1][1..],
            trigger,
            "sau tien to, phan con lai cua o phai la CHINH GIA TRI GOC"
        );

        let parsed = parse(&text).expect("tep vua xuat phai tu phan tich duoc");
        assert_eq!(
            parsed.rows[0].translation.as_deref(),
            Some(trigger),
            "nhap lai phai tra DUNG chuoi GOC ({trigger:?}), khong con tien to vo hieu hoa"
        );
    }
}

/// I/O Matrix "①", ca ĐỐI SOÁT của Ice (đo được, gỡ ra) — một giá trị GỐC đã tự bắt đầu
/// bằng `'` rồi theo sau bởi một ký tự kích hoạt (`'=1+1`, `'+A1`, `'-A1`, `'@SUM(A1)`)
/// PHẢI round-trip nguyên vẹn, không mất dấu `'` dẫn đầu. Bản trước của `needs_formula_
/// guard`/`strip_formula_guard` chỉ nhìn ĐÚNG MỘT ký tự đầu, nên `'=1+1` bị coi là "không
/// cần rào" lúc xuất (ký tự đầu là `'`) rồi bị `strip_formula_guard` CẮT NHẦM dấu `'` gốc
/// lúc nhập (vì ký tự ngay sau nó là `=`) — vi phạm thẳng AC ① của spec cụm B ("giá trị đọc
/// ra bằng đúng từng byte giá trị đã xuất").
///
/// 🔴 **CA ĐỐI CHỨNG BẮT BUỘC (thủ công): trả `needs_formula_guard`/`char_after_leading_
/// quotes` về bản chỉ nhìn ký tự ĐẦU TIÊN (không bỏ hết `'` dẫn đầu trước khi nhìn) rồi chạy
/// lại — ca này PHẢI ĐỎ** (`Some("=1+1")` thay vì `Some("'=1+1")`, mất ký tự đầu).
#[test]
fn a_value_that_already_starts_with_a_quote_followed_by_a_formula_trigger_character_round_trips_with_the_leading_quote_intact()
 {
    for original in ["'=1+1", "'+A1", "'-A1", "'@SUM(A1)"] {
        let mut tier = BTreeMap::new();
        tier.insert("term".to_owned(), entry(1, "term", Some(original)));

        let text = render_tier(&tier, Delimiter::Csv);
        let parsed = parse(&text).expect("tep vua xuat phai tu phan tich duoc");
        assert_eq!(
            parsed.rows[0].translation.as_deref(),
            Some(original),
            "gia tri GOC da tu mang mot dau ' dan dau van phai round-trip NGUYEN VEN, khong \
             mat ky tu dau -- xuat them DUNG MOT dau ' (thanh hai dau), nhap bo DUNG MOT \
             (tra ve dung mot, khong phai khong con dau nao)"
        );
    }

    // Doi chung: mot gia tri bat dau bang ' nhung KHONG theo sau boi ky tu kich hoat khong
    // duoc rao, va giu nguyen tron ven.
    let mut tier = BTreeMap::new();
    tier.insert("term".to_owned(), entry(1, "term", Some("'abc")));
    let text = render_tier(&tier, Delimiter::Csv);
    let parsed = parse(&text).expect("tep vua xuat phai tu phan tich duoc");
    assert_eq!(
        parsed.rows[0].translation.as_deref(),
        Some("'abc"),
        "'abc' khong theo sau boi ky tu kich hoat -- khong duoc rao, giu nguyen"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// VÒNG TRÒN — ca trung tâm, đối chứng gỡ chỗ nối bắt buộc chạy tay (xem §Verification)
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix "Vòng tròn xuất→nhập" — NĂM trường giữ NGUYÊN, `term_origin` thành
/// `file_import` bất kể xuất xứ gốc.
///
/// 🔴 **CA ĐỐI CHỨNG BẮT BUỘC (thủ công, xem §Verification của spec):** gỡ lớp bọc nháy kép
/// trong `exchange::quote_field`/`field_needs_quoting` rồi chạy LẠI đúng ca này — nó PHẢI
/// ĐỎ. Không gỡ thì cổng này không chứng minh được gì về lớp bọc, chỉ chứng minh về đường
/// ống nói chung.
#[test]
fn round_trip_preserves_five_user_visible_fields_and_marks_origin_as_file_import() {
    let mut tier = BTreeMap::new();
    tier.insert(
        "note,tricky".to_owned(),
        GlossaryEntry {
            id: 1,
            source_term: "note,tricky".to_owned(),
            translation: Some("ban dich \"trich dan\"".to_owned()),
            note: "dong mot\ndong hai, co phay".to_owned(),
            category: Category::Person,
            term_origin: TermOrigin::Manual,
            created_at: "2026-08-01T12:00:00.000Z".to_owned(),
        },
    );
    tier.insert("plain".to_owned(), entry(2, "plain", None));

    for delimiter in [Delimiter::Csv, Delimiter::Tsv] {
        let text = render_tier(&tier, delimiter);
        let parsed = parse(&text).expect("tep vua xuat phai tu phan tich duoc");
        assert_eq!(parsed.rows.len(), 2, "hai hang phai nhap lai duoc trong ({delimiter:?})");

        let empty_target: BTreeMap<String, GlossaryEntry> = BTreeMap::new();
        let plans = classify(&parsed.rows, &empty_target);
        assert!(
            plans.iter().all(|p| matches!(p.kind, RowPlanKind::New)),
            "nhap vao mot tang RONG -- ca hai hang phai phan loai MOI"
        );

        let tricky = parsed.rows.iter().find(|r| r.source_term == "note,tricky").expect("hang tricky");
        assert_eq!(tricky.translation.as_deref(), Some("ban dich \"trich dan\""));
        assert_eq!(tricky.note, "dong mot\ndong hai, co phay");
        assert_eq!(tricky.category, Category::Person);
        assert_eq!(
            tricky.created_at.as_deref(),
            Some("2026-08-01T12:00:00.000Z"),
            "created_at phai giu NGUYEN tu tep, khong bi ghi de bang thoi diem nhap"
        );

        let plain = parsed.rows.iter().find(|r| r.source_term == "plain").expect("hang plain");
        assert_eq!(plain.translation, None, "translation rong phai la CHO CHOT, khong phai chuoi rong");
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// NHẬP — đoán dấu phân cách
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn delimiter_is_inferred_from_whichever_separator_the_header_contains() {
    let csv = "source_term,translation\na,b\n";
    let parsed = parse(csv).expect("header chi co dau phay");
    assert_eq!(parsed.rows.len(), 1);

    let tsv = "source_term\ttranslation\na\tb\n";
    let parsed = parse(tsv).expect("header chi co Tab");
    assert_eq!(parsed.rows.len(), 1);
}

/// I/O Matrix "Đoán dấu phân cách" — CẢ hai hoặc KHÔNG cái nào ⇒ lỗi có tên, 0 lượt ghi.
#[test]
fn delimiter_cannot_be_resolved_when_the_header_has_both_or_neither() {
    let both = "source_term,translation\tnote\na,b,c\n";
    assert_eq!(parse(both), Err(vec![ParseIssue::DelimiterUnresolved]));

    let neither = "source_term;translation;note\na;b;c\n";
    assert_eq!(parse(neither), Err(vec![ParseIssue::DelimiterUnresolved]));
}

/// I/O Matrix "④" — hàng tiêu đề TSV có một ô BỌC chứa dấu phẩy KHÔNG được hiểu nhầm là
/// "cả hai dấu phân cách cùng có mặt". Bản trước dò trên văn bản THÔ
/// (`header_text.contains(',')`) và trượt oan; bản vá dò trên phần NGOÀI ô bọc.
///
/// 🔴 **CA ĐỐI CHỨNG BẮT BUỘC (thủ công): trả `unquoted_char_present` về `header_text.
/// contains(target)` (bản cũ) rồi chạy lại — ca này PHẢI ĐỎ** (`Err([DelimiterUnresolved])`
/// thay vì nhận TSV).
#[test]
fn a_quoted_comma_inside_a_tsv_header_cell_does_not_confuse_delimiter_detection() {
    let text = "source_term\t\"note, extra\"\ntam\tMo Dung\n";
    let parsed = parse(text)
        .expect("dau phay NAM TRONG mot o boc cua header TSV khong duoc bien thanh DelimiterUnresolved");
    assert_eq!(parsed.rows.len(), 1);
    assert_eq!(parsed.rows[0].source_term, "tam");
    assert_eq!(
        parsed.ignored_columns,
        vec!["note, extra".to_owned()],
        "ten cot LA (mang dau phay TRONG o boc) van phai duoc nhan dien dung nguyen van"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// NHẬP — cột
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix "Thiếu một cột bắt buộc" — lỗi nêu TÊN cột thiếu, 0 lượt ghi.
#[test]
fn missing_the_required_source_term_column_is_refused_naming_the_column() {
    let text = "translation,note\na,b\n";
    assert_eq!(
        parse(text),
        Err(vec![ParseIssue::MissingColumn { column: "source_term" }])
    );
}

/// I/O Matrix "Thừa cột lạ" — bỏ qua cột đó và NÓI RA, không im lặng vứt (KHÔNG phải lỗi).
#[test]
fn unknown_header_columns_are_ignored_and_reported_not_silently_dropped() {
    let text = "source_term,translation,usage_count\na,b,42\n";
    let parsed = parse(text).expect("cot la KHONG phai mot loi");
    assert_eq!(parsed.rows.len(), 1);
    assert_eq!(parsed.rows[0].source_term, "a");
    assert_eq!(
        parsed.ignored_columns,
        vec!["usage_count".to_owned()],
        "cot la phai duoc liet ra de NOI RA, khong bien mat khong dau vet"
    );
}

/// I/O Matrix "⑤" — hai cột trùng TÊN ĐÃ BIẾT ở hàng tiêu đề phải là một lỗi CÓ TÊN RIÊNG
/// mang tên cột trùng — bản trước để cột THỨ HAI mất im lặng (`position(..)` chỉ tìm khớp
/// ĐẦU TIÊN, và nó KHÔNG lọt vào `ignored_columns` vì không phải một tên LẠ).
///
/// 🔴 **CA ĐỐI CHỨNG BẮT BUỘC (thủ công): bỏ khối kiểm `known_column_counts` rồi chạy lại —
/// ca này PHẢI ĐỎ** (`parse` thành công, cột `translation` thứ hai biến mất không dấu vết).
#[test]
fn two_header_columns_sharing_a_known_name_are_refused_naming_the_duplicated_column() {
    let text = "source_term,translation,translation\na,x,y\n";
    assert_eq!(
        parse(text),
        Err(vec![ParseIssue::DuplicateColumn { column: "translation".to_owned() }])
    );
}

/// I/O Matrix "Vắng cột tuỳ chọn" — chỉ 4 cột như mockup vẫn nhập được; `created_at` = hôm
/// nay (`None` ở tầng phân tích, chỗ ghi tự điền), `term_origin` = `file_import` (chỗ ghi tự
/// đặt, không đọc từ tệp).
#[test]
fn only_four_of_six_columns_present_still_imports_with_sensible_defaults() {
    let text = "source_term,translation,category,note\nterm,ban dich,person,ghi chu\n";
    let parsed = parse(text).expect("bon cot nhu mockup phai nhap duoc");
    assert_eq!(parsed.rows.len(), 1);
    let row = &parsed.rows[0];
    assert_eq!(row.source_term, "term");
    assert_eq!(row.translation.as_deref(), Some("ban dich"));
    assert_eq!(row.category, Category::Person);
    assert_eq!(row.note, "ghi chu");
    assert_eq!(row.created_at, None, "vang cot created_at -- chua tay ghi se dien hom nay");
}

/// I/O Matrix "Số ô lệch hàng tiêu đề" — lỗi mang SỐ DÒNG và số ô đếm được.
#[test]
fn a_row_with_the_wrong_cell_count_is_refused_naming_the_line_and_count() {
    let text = "source_term,translation,note,category,term_origin,created_at\n\
                a,b,c,other,manual,2026-08-24T00:00:00.000Z\n\
                g,h\n";
    assert_eq!(
        parse(text),
        Err(vec![ParseIssue::CellCountMismatch { line: 3, expected: 6, found: 2 }])
    );
}

/// I/O Matrix "`category` lạ" — lỗi mang SỐ DÒNG và giá trị đọc được.
#[test]
fn an_unknown_category_is_refused_naming_the_line_and_value() {
    let text = "source_term,category\na,weapon\n";
    assert_eq!(
        parse(text),
        Err(vec![ParseIssue::UnknownCategory { line: 2, value: "weapon".to_owned() }])
    );
}

/// I/O Matrix "`source_term` rỗng/toàn khoảng trắng" — lỗi mang số dòng, bắt ở Rust TRƯỚC
/// khi SQL bắt. `U+3000` (dấu cách biểu ý) là ca minh hoạ mà spec nêu đích danh.
#[test]
fn a_blank_or_whitespace_only_source_term_is_refused_naming_the_line() {
    let text = "source_term,translation\n\u{3000},x\n";
    assert_eq!(parse(text), Err(vec![ParseIssue::BlankSourceTerm { line: 2 }]));

    let text = "source_term,translation\n   ,x\n";
    assert_eq!(parse(text), Err(vec![ParseIssue::BlankSourceTerm { line: 2 }]));
}

/// I/O Matrix "⑥" — `source_term` chỉ gồm một ký tự ZERO-WIDTH (U+200B) phải bị bắt là
/// `BlankSourceTerm`, CÙNG khoá/CÙNG câu với ô rỗng — `str::trim()` KHÔNG cắt nó (đo được
/// 2026-08-25, xem §Design Notes của spec cụm B).
///
/// 🔴 **CA ĐỐI CHỨNG BẮT BUỘC (thủ công): bỏ `strip_zero_width` khỏi đường trích `source_term`
/// rồi chạy lại — ca này PHẢI ĐỎ** (`Ok`, một mục Glossary vô hình được tạo ra).
#[test]
fn a_source_term_containing_only_a_zero_width_character_is_refused_as_blank() {
    let text = "source_term,translation\n\u{200B},x\n";
    assert_eq!(parse(text), Err(vec![ParseIssue::BlankSourceTerm { line: 2 }]));

    // U+FEFF GIỮA tệp (không ở đầu tệp — `strip_bom` không chạm ca này) là ca minh hoạ mà
    // §Design Notes nêu đích danh: dán hai tệp xuất nối liền để lại một U+FEFF giữa văn bản.
    let text = "source_term,translation\n\u{FEFF},x\n";
    assert_eq!(parse(text), Err(vec![ParseIssue::BlankSourceTerm { line: 2 }]));
}

/// I/O Matrix "⑥b" — một `source_term` HỢP LỆ mang một ký tự zero-width kèm theo (ở đây, ở
/// CUỐI) phải được NHẬN, và ký tự đó bị CẮT khỏi giá trị lưu xuống — không giữ một ký tự vô
/// hình trong dữ liệu đã lưu.
#[test]
fn a_valid_source_term_with_a_trailing_zero_width_character_is_accepted_with_it_stripped() {
    let text = "source_term,translation\n萧炎\u{200B},x\n";
    let parsed = parse(text).expect("zero-width DI KEM mot thuat ngu hop le khong phai loi");
    assert_eq!(
        parsed.rows[0].source_term, "萧炎",
        "ky tu zero-width phai bi CAT khoi gia tri luu xuong, khong giu nguyen"
    );
}

/// I/O Matrix "Mục không có bản dịch" — vào CHỜ CHỐT (`translation = None`), KHÔNG phải đã
/// chốt với chuỗi rỗng.
#[test]
fn a_blank_translation_cell_parses_as_pending_not_confirmed_with_an_empty_string() {
    let text = "source_term,translation\na,\nb,   \n";
    let parsed = parse(text).expect("ban dich rong khong phai mot loi");
    assert_eq!(parsed.rows[0].translation, None);
    assert_eq!(parsed.rows[1].translation, None, "chi khoang trang cung phai thanh None");
}

/// I/O Matrix "`source_term` trùng trong chính tệp" — lỗi mang CẢ HAI số dòng, không "dòng
/// sau thắng" im lặng.
#[test]
fn a_source_term_duplicated_within_the_file_is_refused_naming_both_lines() {
    let text = "source_term,translation\na,x\nb,y\na,z\n";
    assert_eq!(
        parse(text),
        Err(vec![ParseIssue::DuplicateSourceTerm { first_line: 2, second_line: 4 }])
    );
}

/// I/O Matrix "⑦" — `source_term` trùng mà lần đầu đã bị bác vì lý do KHÁC (category lạ)
/// vẫn phải bị gắn cờ trùng ở lần THỨ HAI. Bản trước chỉ ghi `seen` cho hàng ĐÃ QUA MỌI kiểm
/// khác (`row_ok`), nên một hàng đầu bị bác vì lý do khác làm hàng SAU "không thấy" trùng.
///
/// 🔴 **CA ĐỐI CHỨNG BẮT BUỘC (thủ công): trả `seen.entry(..).or_insert(line)` về khối `if
/// let Some(&first_line) = seen.get(source_term) { .. }` CŨ (chỉ đọc, không ghi ở đây) rồi
/// chạy lại — ca này PHẢI ĐỎ** (chỉ báo `UnknownCategory`, mất hẳn `DuplicateSourceTerm`).
#[test]
fn a_duplicate_whose_first_occurrence_was_rejected_for_an_unrelated_reason_is_still_flagged_as_a_duplicate()
 {
    let text = "source_term,category\nX,weapon\nY,other\nX,other\n";
    assert_eq!(
        parse(text),
        Err(vec![
            ParseIssue::UnknownCategory { line: 2, value: "weapon".to_owned() },
            ParseIssue::DuplicateSourceTerm { first_line: 2, second_line: 4 },
        ]),
        "dong 2 (X, category la) VA loi trung (2,4) deu phai duoc bao -- khong duoc 'khong \
         thay trung' chi vi lan dau bi bac vi ly do khac"
    );
}

/// Story 3.10b, đóng `deferred-work.md:6776` — "Nháy kép đặt sai chỗ": một `"` KHÔNG đứng
/// đầu ô là một ký tự THƯỜNG, ở CẢ bước cắt dòng lẫn bước cắt ô. Trước bản vá, bước cắt
/// DÒNG (`split_first_logical_line`) đảo `in_quotes` trên MỌI `"` gặp được — một dấu nháy
/// đặt sai chỗ ở dòng 2 tự "mở một ô bọc giả" nuốt luôn dấu xuống dòng thật sau đó, nên
/// dòng 3 (`d,e`) bị gộp vào CÙNG một dòng logic với dòng 2 và hàng này bị từ chối vì
/// `CellCountMismatch { line: 2, expected: 2, found: 3 }` thay vì phân tích ra HAI hàng
/// hợp lệ.
#[test]
fn a_stray_quote_not_at_the_start_of_a_field_is_literal_in_both_the_line_and_field_splitter() {
    let text = "source_term,translation\na\"b,c\nd,e\n";
    let parsed = parse(text).expect("nhay kep dat sai cho van phai phan tich duoc TRON VEN");
    assert_eq!(
        parsed.rows,
        vec![
            ImportRow {
                line: 2,
                source_term: "a\"b".to_owned(),
                translation: Some("c".to_owned()),
                note: String::new(),
                category: Category::Other,
                created_at: None,
            },
            ImportRow {
                line: 3,
                source_term: "d".to_owned(),
                translation: Some("e".to_owned()),
                note: String::new(),
                category: Category::Other,
                created_at: None,
            },
        ]
    );
}

/// Story 3.10b, đóng `deferred-work.md:6787` — "Hàng trùng `source_term` VÀ `category` lạ":
/// dòng thứ hai sai CẢ HAI phải báo CẢ HAI lỗi trong MỘT lượt, không chỉ lỗi ĐẦU TIÊN tìm
/// được. Trước bản vá, kiểm `category` `continue` sớm nên lượt kiểm trùng của CHÍNH dòng đó
/// không bao giờ chạy tới.
#[test]
fn a_row_that_is_both_a_duplicate_and_has_an_unknown_category_reports_both_issues() {
    let text = "source_term,category\na,other\na,weapon\n";
    assert_eq!(
        parse(text),
        Err(vec![
            ParseIssue::UnknownCategory { line: 3, value: "weapon".to_owned() },
            ParseIssue::DuplicateSourceTerm { first_line: 2, second_line: 3 },
        ])
    );
}

/// I/O Matrix "Văn bản rỗng / chỉ có tiêu đề" — 0 mục, KHÔNG lỗi, phân biệt được với "tệp
/// hỏng".
#[test]
fn blank_text_and_header_only_text_both_parse_to_zero_rows_with_no_error() {
    let parsed = parse("").expect("van ban rong hoan toan khong phai loi");
    assert_eq!(parsed.rows.len(), 0);
    assert_eq!(parsed.ignored_columns.len(), 0);

    let parsed = parse("source_term,translation\n").expect("chi hang tieu de khong phai loi");
    assert_eq!(parsed.rows.len(), 0);

    let parsed = parse("   \n\n").expect("chi khoang trang/dong rong khong phai loi");
    assert_eq!(parsed.rows.len(), 0);
}

/// I/O Matrix "BOM ở đầu tệp" — cắt trước khi phân tích, cùng khuôn `import.rs::strip_bom`.
#[test]
fn a_leading_byte_order_mark_is_stripped_before_parsing() {
    let text = "\u{feff}source_term,translation\na,b\n";
    let parsed = parse(text).expect("BOM phai duoc cat truoc khi phan tich");
    assert_eq!(parsed.rows.len(), 1);
    assert_eq!(parsed.rows[0].source_term, "a");
}

/// I/O Matrix "`\r\n` và `\n` lẫn lộn" — cả hai đọc được, `\r` không lọt vào giá trị ô cuối.
#[test]
fn mixed_crlf_and_lf_line_endings_both_parse_and_do_not_leak_cr_into_the_last_cell() {
    let text = "source_term,translation\r\na,b\nc,d\r\n";
    let parsed = parse(text).expect("crlf lan lon voi lf phai doc duoc");
    assert_eq!(parsed.rows.len(), 2);
    assert_eq!(parsed.rows[0].translation.as_deref(), Some("b"));
    assert_eq!(parsed.rows[1].translation.as_deref(), Some("d"));
    assert!(
        !parsed.rows.iter().any(|r| r.translation.as_deref().unwrap_or("").contains('\r')),
        "ky tu \\r khong duoc lot vao gia tri o cuoi mot dong CRLF"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mục ②/③ (vòng rà ba lớp, cụm B 2026-08-25) — ngoặc kép không đóng · \r trần trong ô bọc
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix "②" — một ngoặc kép MỞ nhưng KHÔNG BAO GIỜ đóng phải sinh một lỗi CÓ TÊN
/// RIÊNG mang số dòng nơi ô đó mở ra — KHÔNG được nuốt các hàng ĐÚNG phía sau thành một
/// `CellCountMismatch` DUY NHẤT trỏ sai chỗ (ở đây: dòng CUỐI của 300 hàng đúng).
///
/// 🔴 **CA ĐỐI CHỨNG BẮT BUỘC (thủ công): bỏ nhánh trả `in_quotes` ở cuối
/// `split_first_logical_line` (luôn trả `false` cho cờ thứ ba) rồi chạy lại — ca này PHẢI
/// ĐỎ** (mất tên lỗi riêng, đổi thành `CellCountMismatch` ở dòng cuối tệp).
#[test]
fn an_unterminated_quoted_field_is_a_named_error_carrying_the_opening_line_not_a_cell_count_mismatch_swallowing_every_row_after_it()
 {
    let mut text = String::from("source_term,translation\n\"Mo Dung,ten\n");
    for i in 0..300 {
        text.push_str(&format!("hang-{i},x\n"));
    }
    assert_eq!(
        parse(&text),
        Err(vec![ParseIssue::UnterminatedQuotedField { line: 2 }]),
        "phai la MOT loi CO TEN RIENG tro dung dong 2 (noi o mo ngoac kep), khong phai mot \
         CellCountMismatch nuot tron 300 hang dung phia sau"
    );
}

/// Cùng lỗ hổng ②, nhưng ở chính HÀNG TIÊU ĐỀ (dấu ngoặc kép mở trong header không bao giờ
/// đóng) — số dòng phải là 1.
#[test]
fn an_unterminated_quoted_field_in_the_header_row_itself_is_also_a_named_error() {
    let text = "\"source_term,translation\na,b\n";
    assert_eq!(parse(text), Err(vec![ParseIssue::UnterminatedQuotedField { line: 1 }]));
}

/// I/O Matrix "③" — `\r` TRẦN bên trong một ô đã bọc là một RANH GIỚI DÒNG mà người dùng
/// nhìn thấy trong trình soạn thảo, dù nó không phải một `\n`/`\r\n` mà bước tách DÒNG dùng
/// để chia logical line. Số dòng báo lỗi của hàng SAU phải khớp số dòng người dùng tự đếm.
///
/// 🔴 **CA ĐỐI CHỨNG BẮT BUỘC (thủ công): trả `logical_lines` về đếm `line.matches('\n').
/// count()` (bản cũ, chỉ đếm `\n`) thay vì `count_line_breaks` rồi chạy lại — ca này PHẢI
/// ĐỎ** (mong `line: 4`, bản cũ cho `line: 3`).
#[test]
fn a_bare_cr_inside_a_quoted_field_advances_the_line_number_for_rows_after_it() {
    // Dong 2: mot o boc "a\rb" -- nguoi dung mo bang trinh soan thao se thay HAI dong man
    // hinh cho MOT "dong logic". Hang LOI (category la) la dong THU BA theo du lieu nhung
    // dong THU TU theo cach dem cua nguoi dung (header=1, o boc=2..3, hang loi=4).
    let text = "source_term,translation,category\n\"a\rb\",x,other\nc,y,weapon\n";
    assert_eq!(
        parse(text),
        Err(vec![ParseIssue::UnknownCategory { line: 4, value: "weapon".to_owned() }]),
        "hang loi phai duoc danh so DUNG BANG so dong nguoi dung dem trong trinh soan thao \
         (header=1, o boc \\r=2..3, hang loi=4) -- khong phai 3 (neu chi dem \\n)"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// P2 (vòng rà ba lớp 2026-08-25) — dòng logic RỖNG không phải một hàng dữ liệu
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix "Tệp kết thúc bằng dòng trống" — đo được TRƯỚC vá: `parse("<header>\n慕容,
/// ...\n\n")` trả `Err([CellCountMismatch { line: 3, expected: 6, found: 1 }])`. Một tệp
/// hợp lệ, kết thúc bằng `\n\n` (thói quen của vô số trình soạn thảo), bị từ chối TRỌN VẸN.
#[test]
fn a_trailing_blank_logical_line_is_not_a_data_row_and_is_not_an_error() {
    let text = "source_term,translation\n慕容,Mộ Dung\n\n";
    let parsed = parse(text).expect("dong trong cuoi tep khong phai loi -- day chinh la ca do duoc truoc va");
    assert_eq!(parsed.rows.len(), 1);
    assert_eq!(parsed.rows[0].source_term, "慕容");
}

/// 🔴 **Quyết định cho dòng trống Ở GIỮA tệp — VIẾT RA, không để tình cờ (P2).** Cùng luật
/// với dòng trống cuối tệp: bỏ qua lặng lẽ, không lỗi. Người dùng không có cách nào tự phân
/// biệt "trình soạn thảo chèn dòng trống" với "tôi gõ nhầm Enter" để mà sửa cho đúng, nên
/// phạt nó như một lỗi sẽ phạt một thao tác vô hại giống hệt một dòng dữ liệu bị cắt cụt.
/// Số dòng của hàng SAU dòng trống KHÔNG bị lệch — `logical_lines` đánh số theo dòng NGUỒN.
#[test]
fn a_blank_logical_line_in_the_middle_of_the_file_is_also_skipped_by_the_same_rule() {
    let text = "source_term,translation\na,x\n\nc,y\n";
    let parsed = parse(text)
        .expect("dong trong o giua cung khong phai loi -- cung mot luat voi dong trong cuoi tep");
    assert_eq!(parsed.rows.len(), 2, "dong trong o giua khong sinh ra mot hang thu ba nao");
    assert_eq!(parsed.rows[0].source_term, "a");
    assert_eq!(parsed.rows[1].source_term, "c");
    assert_eq!(
        parsed.rows[1].line, 4,
        "'c,y' la dong NGUON thu 4 (header=1, 'a,x'=2, dong trong=3, 'c,y'=4) -- bo qua mot \
         dong khong duoc lam dong sau no doi so"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// P3 (vòng rà ba lớp 2026-08-25) — `created_at` phải khớp hình dạng ISO-8601 UTC
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix "`created_at` sai định dạng" — lỗi mang số dòng và giá trị đọc được.
#[test]
fn an_invalid_created_at_format_is_refused_naming_the_line_and_value() {
    let text = "source_term,created_at\na,hom qua\n";
    assert_eq!(
        parse(text),
        Err(vec![ParseIssue::InvalidCreatedAt { line: 2, value: "hom qua".to_owned() }])
    );
}

/// Đối chứng dương của ca trên: đúng hình dạng thì được nhận, và giá trị giữ NGUYÊN VĂN.
#[test]
fn a_well_formed_created_at_is_accepted_and_kept_verbatim() {
    let text = "source_term,created_at\na,2026-08-01T12:00:00.000Z\n";
    let parsed = parse(text).expect("dinh dang dung phai duoc chap nhan");
    assert_eq!(parsed.rows[0].created_at.as_deref(), Some("2026-08-01T12:00:00.000Z"));
}

// ═════════════════════════════════════════════════════════════════════════════════
// P7 (vòng rà ba lớp 2026-08-25) — cặp ParseIssue ↔ MessageKey được canh
// ═════════════════════════════════════════════════════════════════════════════════

/// Ánh xạ MONG ĐỢI, viết TAY và EXHAUSTIVE (không nhánh `_`) — thêm một biến thể
/// `ParseIssue` mới mà quên cập nhật hàm này là một LỖI BIÊN DỊCH ở đây, không phải một lỗ
/// hổng im lặng. Đây là cơ chế ép: `ParseIssue`/`GlossaryError::ImportUniqueConflict` (lỗi
/// GHI, xem `glossary_contract.rs`) không có một danh sách `ALL` như `MessageKey`, nên phép
/// kiểm này tự dựng lấy sự exhaustive đó bằng chính cú pháp `match`.
fn expected_message_key_for_parse_issue(issue: &ParseIssue) -> MessageKey {
    match issue {
        ParseIssue::DelimiterUnresolved => MessageKey::GlossaryImportDelimiterUnresolved,
        ParseIssue::MissingColumn { .. } => MessageKey::GlossaryImportMissingColumn,
        ParseIssue::CellCountMismatch { .. } => MessageKey::GlossaryImportCellCountMismatch,
        ParseIssue::UnknownCategory { .. } => MessageKey::GlossaryImportUnknownCategory,
        ParseIssue::BlankSourceTerm { .. } => MessageKey::GlossaryImportBlankSourceTerm,
        ParseIssue::DuplicateSourceTerm { .. } => MessageKey::GlossaryImportDuplicateSourceTerm,
        ParseIssue::InvalidCreatedAt { .. } => MessageKey::GlossaryImportInvalidCreatedAt,
        // 🔵 THÊM 2026-08-25 (vòng rà ba lớp, cụm B) — hai biến thể mới của mục ② và ⑤.
        // Thêm một biến thể `ParseIssue` mà quên cập nhật `match` NÀY là một LỖI BIÊN DỊCH
        // ở đây (không nhánh `_`), không phải một lỗ hổng im lặng.
        ParseIssue::UnterminatedQuotedField { .. } => {
            MessageKey::GlossaryImportUnterminatedQuotedField
        }
        ParseIssue::DuplicateColumn { .. } => MessageKey::GlossaryImportDuplicateColumn,
    }
}

/// 🔴 **P7 (vòng rà ba lớp) — cặp `ParseIssue` ↔ `MessageKey` KHÔNG được canh trước ca này.**
/// `ipc_contract.rs` canh `MessageKey` ↔ `vi.json` (khoá có mặt, tham số khớp); nó KHÔNG hỏi
/// "biến thể `ParseIssue` này có thật sự đi ra đúng khoá không" — `impl From<ParseIssue> for
/// IpcError` có thể gán nhầm khoá của một biến thể khác mà không cổng nào đỏ. Ca này dựng
/// MỘT mẫu cho MỖI biến thể, đi qua `.into()` thật, và so khoá THẬT với khoá MONG ĐỢI (hàm
/// exhaustive ngay trên).
#[test]
fn every_parse_issue_variant_maps_to_the_message_key_it_actually_produces() {
    let samples: Vec<ParseIssue> = vec![
        ParseIssue::DelimiterUnresolved,
        ParseIssue::MissingColumn { column: "source_term" },
        ParseIssue::CellCountMismatch { line: 3, expected: 6, found: 1 },
        ParseIssue::UnknownCategory { line: 3, value: "weapon".to_owned() },
        ParseIssue::BlankSourceTerm { line: 3 },
        ParseIssue::DuplicateSourceTerm { first_line: 2, second_line: 5 },
        ParseIssue::InvalidCreatedAt { line: 3, value: "hom qua".to_owned() },
        ParseIssue::UnterminatedQuotedField { line: 2 },
        ParseIssue::DuplicateColumn { column: "translation".to_owned() },
    ];

    for issue in samples {
        let expected = expected_message_key_for_parse_issue(&issue);
        let debug = format!("{issue:?}");
        let ipc: IpcError = issue.into();
        assert_eq!(
            ipc.message_key(),
            expected,
            "ParseIssue {debug} -- khoa THAT ({:?}) khong khop khoa MONG DOI ({expected:?})",
            ipc.message_key(),
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// PHÂN LOẠI — classify
// ═════════════════════════════════════════════════════════════════════════════════

fn row(source_term: &str, translation: Option<&str>) -> ImportRow {
    ImportRow {
        line: 2,
        source_term: source_term.to_owned(),
        translation: translation.map(str::to_owned),
        note: String::new(),
        category: Category::Other,
        created_at: None,
    }
}

/// I/O Matrix "Hàng mới".
#[test]
fn a_source_term_absent_from_the_target_tier_classifies_as_new() {
    let existing: BTreeMap<String, GlossaryEntry> = BTreeMap::new();
    let plans = classify(&[row("moi", Some("x"))], &existing);
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].kind, RowPlanKind::New);
}

/// I/O Matrix "Hàng giống hệt" — không đề nghị gì, không ghi.
#[test]
fn matching_source_term_and_translation_classifies_as_identical() {
    let mut existing = BTreeMap::new();
    existing.insert("term".to_owned(), entry(7, "term", Some("cung mot ban dich")));
    let plans = classify(&[row("term", Some("cung mot ban dich"))], &existing);
    assert_eq!(plans[0].kind, RowPlanKind::Identical);
}

/// I/O Matrix "Hàng bất đồng" — mang CẢ HAI bản dịch.
#[test]
fn differing_translations_for_the_same_source_term_classify_as_conflict_carrying_both() {
    let mut existing = BTreeMap::new();
    existing.insert("term".to_owned(), entry(9, "term", Some("ban dich cu")));
    let plans = classify(&[row("term", Some("ban dich moi"))], &existing);
    match &plans[0].kind {
        RowPlanKind::Conflict { existing_id, existing_translation } => {
            assert_eq!(*existing_id, 9);
            assert_eq!(existing_translation.as_deref(), Some("ban dich cu"));
        }
        other => panic!("phai la Conflict, nhan: {other:?}"),
    }
    assert_eq!(plans[0].translation.as_deref(), Some("ban dich moi"), "ban dich cua TEP van doc duoc tu RowPlan");
}

// ═════════════════════════════════════════════════════════════════════════════════
// ĐƯỜNG GHI — export_tier / import_into_tier (cần Store)
// ═════════════════════════════════════════════════════════════════════════════════

/// Hàng mới ghi với xuất xứ `file_import` — KHÔNG nhận qua tham số.
#[test]
fn import_into_tier_writes_new_rows_tagged_file_import() {
    let dir = temp_dir("import-new-rows");
    let store = open_global(&dir);

    let plans = classify(&[row("慕容", Some("Mộ Dung"))], &BTreeMap::new());
    let summary = import_into_tier(&store, None, GlossaryTier::Global, &plans, &BTreeMap::new())
        .expect("nhap hang moi");
    assert_eq!(summary.inserted, 1);
    assert_eq!(summary.updated, 0);

    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(tier["慕容"].term_origin, TermOrigin::FileImport);
    assert_eq!(tier["慕容"].translation.as_deref(), Some("Mộ Dung"));

    drop(store);
    cleanup(&dir);
}

/// Hàng *giống hệt* không ghi gì.
#[test]
fn import_into_tier_writes_nothing_for_identical_rows() {
    let dir = temp_dir("import-identical");
    let store = open_global(&dir);
    add_manual_term(&store, None, GlossaryTier::Global, "term", Some("x"), "", Category::Other)
        .expect("chen truoc");

    let existing = load_tier(&store).expect("nap tang");
    let plans = classify(&[row("term", Some("x"))], &existing);
    assert_eq!(plans[0].kind, RowPlanKind::Identical);

    let summary = import_into_tier(&store, None, GlossaryTier::Global, &plans, &BTreeMap::new())
        .expect("nhap ca giong het");
    assert_eq!(summary.inserted, 0);
    assert_eq!(summary.updated, 0);
    assert_eq!(summary.identical, 1);

    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(tier["term"].term_origin, TermOrigin::Manual, "muc CU khong bi doi xuat xu");

    drop(store);
    cleanup(&dir);
}

/// §Always: mặc định GIỮ CỦA TÔI — vắng quyết định ⇒ không ghi đè.
#[test]
fn import_into_tier_keeps_the_existing_translation_by_default_on_conflict() {
    let dir = temp_dir("import-keep-mine");
    let store = open_global(&dir);
    add_manual_term(&store, None, GlossaryTier::Global, "term", Some("cu"), "", Category::Other)
        .expect("chen truoc");

    let existing = load_tier(&store).expect("nap tang");
    let plans = classify(&[row("term", Some("moi"))], &existing);

    let summary = import_into_tier(&store, None, GlossaryTier::Global, &plans, &BTreeMap::new())
        .expect("nhap khong quyet dinh nao -- mac dinh giu cua toi");
    assert_eq!(summary.updated, 0, "vang quyet dinh phai la KHONG ghi de");

    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(tier["term"].translation.as_deref(), Some("cu"), "ban dich CU phai con nguyen");

    drop(store);
    cleanup(&dir);
}

/// `TakeTheirs` ghi đè `translation`.
#[test]
fn import_into_tier_take_theirs_updates_the_existing_row() {
    let dir = temp_dir("import-take-theirs");
    let store = open_global(&dir);
    add_manual_term(&store, None, GlossaryTier::Global, "term", Some("cu"), "", Category::Other)
        .expect("chen truoc");

    let existing = load_tier(&store).expect("nap tang");
    let plans = classify(&[row("term", Some("moi"))], &existing);
    let mut decisions = BTreeMap::new();
    decisions.insert("term".to_owned(), ConflictDecision::TakeTheirs);

    let summary = import_into_tier(&store, None, GlossaryTier::Global, &plans, &decisions)
        .expect("nhap voi quyet dinh TakeTheirs");
    assert_eq!(summary.updated, 1);

    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(tier["term"].translation.as_deref(), Some("moi"));

    drop(store);
    cleanup(&dir);
}

/// 🔴 **P1 (vòng rà ba lớp, `intent_gap` Ice chốt 2026-08-25) — ca TRUNG TÂM của bản vá này.**
/// I/O Matrix "Bất đồng, người dùng lấy của file": `TakeTheirs` ghi CHỈ `translation`. Mục
/// đang có mang `note`/`category` THẬT (không phải giá trị mặc định `""`/`Other` mà một hàng
/// TỆP hai cột sẽ mang) — nếu `UPDATE` lỡ đụng cả ba cột như bản đầu, ca này đỏ ngay vì hai
/// trường đó bị GHI ĐÈ bằng `""`/`Other` từ `row()` (mô phỏng đúng hình dạng tệp hai cột
/// `source_term,translation`, đúng mockup).
#[test]
fn take_theirs_updates_only_translation_and_never_touches_note_or_category() {
    let dir = temp_dir("import-take-theirs-translation-only");
    let store = open_global(&dir);
    add_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        "term",
        Some("cu"),
        "ghi chu goc cua nguoi dung",
        Category::Person,
    )
    .expect("chen truoc voi note/category THAT");

    let existing = load_tier(&store).expect("nap tang");
    // `row()` mo phong dung hinh dang mot tep HAI COT (source_term,translation) -- note="",
    // category=Other, giong het gia tri ma `exchange::parse` dien cho cot VANG mat.
    let plans = classify(&[row("term", Some("moi"))], &existing);
    let mut decisions = BTreeMap::new();
    decisions.insert("term".to_owned(), ConflictDecision::TakeTheirs);

    import_into_tier(&store, None, GlossaryTier::Global, &plans, &decisions)
        .expect("TakeTheirs phai thanh cong");

    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(tier["term"].translation.as_deref(), Some("moi"), "translation PHAI doi");
    assert_eq!(
        tier["term"].note, "ghi chu goc cua nguoi dung",
        "note KHONG duoc dung toi -- du RowPlan tu 'tep' mang \"\" (I/O Matrix: 'tep thieu \
         cot note/category')"
    );
    assert_eq!(
        tier["term"].category,
        Category::Person,
        "category KHONG duoc dung toi -- du RowPlan tu 'tep' mang Other"
    );

    drop(store);
    cleanup(&dir);
}

/// I/O Matrix "Bất đồng, tệp CÓ cột `note` mang giá trị khác" — vẫn KHÔNG ghi `note`, kể cả
/// khi tệp mang một giá trị THẬT (không phải mặc định do vắng cột) khác hẳn giá trị đang có.
#[test]
fn take_theirs_ignores_a_note_and_category_the_file_actually_provides() {
    let dir = temp_dir("import-take-theirs-file-provides-note-ignored");
    let store = open_global(&dir);
    add_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        "term",
        Some("cu"),
        "ghi chu goc",
        Category::Person,
    )
    .expect("chen truoc");

    let existing = load_tier(&store).expect("nap tang");
    let file_row = RowPlan {
        line: 2,
        source_term: "term".to_owned(),
        translation: Some("moi".to_owned()),
        note: "ghi chu TU TEP -- khac han ghi chu goc".to_owned(),
        category: Category::Place,
        created_at: None,
        kind: match existing.get("term") {
            Some(e) => RowPlanKind::Conflict {
                existing_id: e.id,
                existing_translation: e.translation.clone(),
            },
            None => panic!("fixture phai co san 'term'"),
        },
    };
    let mut decisions = BTreeMap::new();
    decisions.insert("term".to_owned(), ConflictDecision::TakeTheirs);

    import_into_tier(&store, None, GlossaryTier::Global, &[file_row], &decisions)
        .expect("TakeTheirs phai thanh cong");

    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(tier["term"].translation.as_deref(), Some("moi"));
    assert_eq!(
        tier["term"].note, "ghi chu goc",
        "note cua TEP (mot gia tri THAT, khac han) van bi BO QUA -- duong nhap khong sua ghi \
         chu cua mot muc da co, du tep co noi gi ve no"
    );
    assert_eq!(tier["term"].category, Category::Person, "category cua TEP cung bi BO QUA");

    drop(store);
    cleanup(&dir);
}

/// I/O Matrix "Bất đồng, người dùng lấy của file" — trigger AD-36 chặn lượt lùi về rỗng ⇒
/// `store.write_failed`, CẢ LÔ rollback (không phải nửa lô).
#[test]
fn take_theirs_regressing_a_confirmed_row_to_pending_is_refused_and_rolls_back_the_whole_batch() {
    let dir = temp_dir("import-trigger-rollback");
    let store = open_global(&dir);
    add_manual_term(&store, None, GlossaryTier::Global, "confirmed", Some("da chot"), "", Category::Other)
        .expect("hang da chot");

    let existing = load_tier(&store).expect("nap tang");
    // Mot lo HAI hang: hang dau la New (se ghi neu duoc phep), hang sau la Conflict ma
    // TakeTheirs se lui ban dich ve rong (trigger AD-36 phai chan).
    let plans = classify(
        &[row("hang-moi", Some("se khong duoc ghi")), row("confirmed", None)],
        &existing,
    );
    let mut decisions = BTreeMap::new();
    decisions.insert("confirmed".to_owned(), ConflictDecision::TakeTheirs);

    let result = import_into_tier(&store, None, GlossaryTier::Global, &plans, &decisions);
    assert!(
        matches!(result, Err(GlossaryError::Store(_))),
        "trigger AD-36 phai tu choi qua mot loi Store, nhan: {result:?}"
    );

    let tier = load_tier(&store).expect("nap lai");
    assert!(!tier.contains_key("hang-moi"), "CA LO phai rollback -- hang New cung khong duoc ghi");
    assert_eq!(
        tier["confirmed"].translation.as_deref(),
        Some("da chot"),
        "hang confirmed phai giu NGUYEN ban dich cu -- 0 luot ghi"
    );

    drop(store);
    cleanup(&dir);
}

/// `tier == Work` mà `work` là `None` ⇒ `WorkTierUnavailable`, 0 lượt ghi.
#[test]
fn import_into_tier_refuses_the_work_tier_when_no_work_is_open() {
    let dir = temp_dir("import-no-work");
    let global = open_global(&dir);

    let plans = classify(&[row("term", Some("x"))], &BTreeMap::new());
    let result = import_into_tier(&global, None, GlossaryTier::Work, &plans, &BTreeMap::new());
    assert_eq!(result, Err(GlossaryError::WorkTierUnavailable));

    drop(global);
    cleanup(&dir);
}

/// I/O Matrix "Va `UNIQUE` giữa chừng" — một mục được thêm ở nơi khác SAU lượt phân loại ⇒
/// giao dịch rollback, 0 hàng ghi, lỗi nói va ở `source_term` nào.
#[test]
fn import_into_tier_detects_a_unique_conflict_introduced_after_classification() {
    let dir = temp_dir("import-unique-race");
    let store = open_global(&dir);

    // Chup anh tang RONG (nhu the vua load_tier truoc khi mot lượt ghi khac chen vao).
    let plans = classify(&[row("term", Some("tu tep"))], &BTreeMap::new());

    // Mot lượt ghi KHAC chen "term" vao GIUA luc phan loai va luc nhap that su chay.
    add_manual_term(&store, None, GlossaryTier::Global, "term", Some("da co tu noi khac"), "", Category::Other)
        .expect("mo phong mot luot ghi dua vao");

    let result = import_into_tier(&store, None, GlossaryTier::Global, &plans, &BTreeMap::new());
    assert_eq!(
        result,
        Err(GlossaryError::ImportUniqueConflict { source_terms: vec!["term".to_owned()] }),
        "loi phai noi DUNG source_term va ca lo phai bi tu choi"
    );

    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(
        tier["term"].translation.as_deref(),
        Some("da co tu noi khac"),
        "hang cua luot ghi KHAC phai con nguyen -- lượt nhap khong duoc ghi de no"
    );

    drop(store);
    cleanup(&dir);
}

/// 🔴 **P6 (vòng rà ba lớp) — một lô đua với NHIỀU lượt chèn khác cùng lúc phải nêu HẾT,
/// không dừng ở va chạm ĐẦU TIÊN.** Bản đầu dùng `.find(...)`, chỉ báo được một tên; người
/// dùng phải thử-sửa-thử từng cái một. Ca này dựng BA hàng `New` va CÙNG LÚC.
#[test]
fn import_into_tier_collects_every_unique_conflict_in_one_pass_not_just_the_first() {
    let dir = temp_dir("import-multiple-unique-conflicts");
    let store = open_global(&dir);

    let plans = classify(
        &[row("alpha", Some("a")), row("beta", Some("b")), row("gamma", Some("c"))],
        &BTreeMap::new(),
    );

    // BA lượt ghi KHÁC (không liên quan nhau) chen cả ba source_term vào GIỮA lúc phân loại
    // và lúc nhập thật sự chạy.
    for term in ["alpha", "beta", "gamma"] {
        add_manual_term(&store, None, GlossaryTier::Global, term, Some("da co tu noi khac"), "", Category::Other)
            .expect("mo phong luot ghi dua vao");
    }

    let result = import_into_tier(&store, None, GlossaryTier::Global, &plans, &BTreeMap::new());
    match result {
        Err(GlossaryError::ImportUniqueConflict { mut source_terms }) => {
            source_terms.sort();
            assert_eq!(
                source_terms,
                vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()],
                "loi phai gom TRON ca ba thuat ngu va, khong dung o cai dau tien"
            );
        }
        other => panic!("phai la ImportUniqueConflict voi ca ba thuat ngu, nhan: {other:?}"),
    }

    drop(store);
    cleanup(&dir);
}

/// 🔴 **P5 (vòng rà ba lớp, Ice chốt) — chẩn đoán SAI nguyên nhân là lỗi nặng nhất còn lại
/// sau P1.** Một lô CÙNG LÚC vừa có một hàng `New` mà `source_term` của nó đã tồn tại (do
/// MỘT lượt ghi khác, không liên quan) VỪA có một `TakeTheirs` vi phạm trigger AD-36.
/// Nguyên nhân THẬT khiến giao dịch trượt là trigger — kết quả PHẢI là `GlossaryError::Store`,
/// KHÔNG được gán nhãn `ImportUniqueConflict` (che mất đúng nguyên nhân thật, ngược I/O
/// Matrix hàng "Trigger AD-36 ... ⇒ `store.write_failed`").
#[test]
fn a_batch_with_both_an_unrelated_pre_existing_term_and_a_trigger_violation_reports_the_trigger_as_the_real_cause()
 {
    let dir = temp_dir("import-mixed-failure-reports-real-cause");
    let store = open_global(&dir);

    add_manual_term(&store, None, GlossaryTier::Global, "confirmed", Some("da chot"), "", Category::Other)
        .expect("hang da chot");

    // "da-ton-tai" duoc chup anh la New luc classify (chua co trong tang luc do).
    let existing = load_tier(&store).expect("nap tang truoc classify");
    let plans = classify(
        &[row("da-ton-tai", Some("tu tep")), row("confirmed", None)],
        &existing,
    );

    // Truoc khi import_into_tier THAT SU chay, mot luot ghi KHONG LIEN QUAN da chen
    // "da-ton-tai" vao -- day la thu duy nhat lam hang New nay va UNIQUE, nhung no KHONG
    // phai nguyen nhan khien lo nay that bai (trigger moi la nguyen nhan, xem duoi).
    add_manual_term(&store, None, GlossaryTier::Global, "da-ton-tai", Some("da co tu noi khac"), "", Category::Other)
        .expect("mo phong mot luot ghi KHONG lien quan chen vao giua");

    let mut decisions = BTreeMap::new();
    decisions.insert("confirmed".to_owned(), ConflictDecision::TakeTheirs);

    let result = import_into_tier(&store, None, GlossaryTier::Global, &plans, &decisions);
    assert!(
        matches!(result, Err(GlossaryError::Store(_))),
        "nguyen nhan THAT la trigger AD-36 (TakeTheirs lui 'confirmed' ve rong) -- phai la \
         GlossaryError::Store, KHONG duoc bao nham thanh ImportUniqueConflict chi vi 'da-ton-tai' \
         cung tinh co va UNIQUE. Nhan: {result:?}"
    );

    drop(store);
    cleanup(&dir);
}

/// 🔴 **P5, ca CÔ LẬP ĐÚNG chỗ nối `is_unique_constraint_violation`.** Ca trên (`a_batch_
/// with_both_...`) không thực sự đo được cơ chế phân biệt — lượt trigger luôn abort qua `?`
/// TRƯỚC khi bất kỳ điều gì về `local_conflicts` kịp ảnh hưởng tới kết quả, dù cơ chế phân
/// biệt có mặt hay không. Ca NÀY cô lập đúng một hàng `New` thất bại vì lý do KHÁC `UNIQUE`
/// (ở đây: `CHECK` — `translation` chỉ toàn khoảng trắng, dựng `RowPlan` TRỰC TIẾP để vượt
/// qua lượt kiểm của `exchange::parse` mà đường sản phẩm thật không bao giờ để lọt qua) —
/// đây là kịch bản DUY NHẤT một `INSERT` hàng `New` thất bại mà KHÔNG phải `UNIQUE`, và là
/// chỗ DUY NHẤT chứng minh được `is_unique_constraint_violation` thật sự lọc, không phải
/// một lớp trang trí không làm gì.
#[test]
fn a_new_row_failing_for_a_reason_other_than_unique_reports_the_real_error() {
    let dir = temp_dir("import-new-row-check-not-unique");
    let store = open_global(&dir);

    let plan = RowPlan {
        line: 2,
        source_term: "hang-loi".to_owned(),
        // Vi pham CHECK (chi toan khoang trang) -- KHONG phai UNIQUE.
        translation: Some("   ".to_owned()),
        note: String::new(),
        category: Category::Other,
        created_at: None,
        kind: RowPlanKind::New,
    };

    let result = import_into_tier(&store, None, GlossaryTier::Global, &[plan], &BTreeMap::new());
    assert!(
        matches!(result, Err(GlossaryError::Store(_))),
        "loi CHECK (khong phai UNIQUE) khong duoc gan nham nhan ImportUniqueConflict. \
         Nhan: {result:?}"
    );

    drop(store);
    cleanup(&dir);
}

/// 🔴 **P4 (vòng rà ba lớp) — khoảng hở phép canh: không ca nào trước đây đẩy
/// `Some(created_at)` qua đường GHI.** Helper `row()` ghim cứng `created_at: None`, nên mọi
/// ca dùng `classify`/`row()` không phủ được nhánh `plan.created_at.as_deref()` khi nó là
/// `Some`. Dựng `RowPlan` TRỰC TIẾP để phủ đúng nhánh đó.
#[test]
fn import_into_tier_preserves_a_supplied_created_at_for_a_new_row() {
    let dir = temp_dir("import-new-row-created-at");
    let store = open_global(&dir);

    let plan = RowPlan {
        line: 2,
        source_term: "moc-co-dinh".to_owned(),
        translation: Some("ban dich".to_owned()),
        note: String::new(),
        category: Category::Other,
        created_at: Some("2020-01-02T03:04:05.678Z".to_owned()),
        kind: RowPlanKind::New,
    };

    import_into_tier(&store, None, GlossaryTier::Global, &[plan], &BTreeMap::new())
        .expect("nhap hang moi voi created_at co san");

    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(
        tier["moc-co-dinh"].created_at, "2020-01-02T03:04:05.678Z",
        "created_at phai giu NGUYEN chuoi da cung cap -- thay `plan.created_at.as_deref()` \
         bang `None` trong `import_into_tier` van lam moi ca CU xanh, nhung phai lam CA NAY do \
         (moc that se thanh thoi diem 'now', khong khop chuoi co dinh nay)"
    );

    drop(store);
    cleanup(&dir);
}

/// §Never: xuất KHÔNG đi qua `list_all_entries` — một mục Global bị một mục Work cùng
/// `source_term` che KHÔNG được xuất hiện hai lần trong tệp Global.
#[test]
fn export_tier_reads_exactly_one_tier_and_does_not_duplicate_a_shadowed_entry() {
    let dir = temp_dir("export-one-tier-only");
    let global = open_global(&dir);
    let work = open_project(&dir);

    add_manual_term(&global, None, GlossaryTier::Global, "che-nhau", Some("ban Global"), "", Category::Other)
        .expect("chen Global");
    add_manual_term(&global, Some(&work), GlossaryTier::Work, "che-nhau", Some("ban Work"), "", Category::Other)
        .expect("chen Work");

    let text = export_tier(&global, Delimiter::Csv).expect("xuat tang Global");
    let occurrences = text.matches("che-nhau").count();
    assert_eq!(
        occurrences, 1,
        "export_tier phai doc DUNG mot tang -- xuat hien hai lan la dau hieu no da di qua \
         list_all_entries (phat ca hang bi che)"
    );

    drop(global);
    drop(work);
    cleanup(&dir);
}


/// I/O Matrix "⑥c" — vòng rà ba lớp, Ice ký nới phạm vi 2026-08-25.
///
/// `translation` là cột NẶNG NHẤT trong ba cột văn bản tự do: `.filter(|s| !s.is_empty())`
/// của nó quyết định một mục vào CHỜ CHỐT (`None`) hay ĐÃ CHỐT (`Some`). Một ô toàn
/// zero-width KHÔNG rỗng theo `str::trim()` (`is_whitespace` của Unicode không gồm chúng),
/// nên trước bản vá nó lọt thành `Some("\u{200B}")` — một mục ĐÃ CHỐT mang bản dịch VÔ HÌNH,
/// mà trigger `glossary_entry_lifecycle_is_one_way` (AD-36) khiến KHÔNG lùi lại được.
#[test]
fn a_translation_or_note_of_only_zero_width_characters_does_not_become_an_invisible_value() {
    let text = "source_term,translation\n萧炎,\u{200B}\n";
    let parsed = parse(text).expect("zero-width o cot translation khong phai loi phan tich");
    assert_eq!(
        parsed.rows[0].translation, None,
        "mot `translation` toan zero-width phai doc la VANG MAT (cho chot), khong phai mot \
         gia tri DA CHOT vo hinh -- AD-36 lam trang thai da chot khong lui lai duoc"
    );

    let text = "source_term,translation,note\n萧炎,x,\u{2060}\u{FEFF}\n";
    let parsed = parse(text).expect("zero-width o cot note khong phai loi phan tich");
    assert_eq!(
        parsed.rows[0].note, "",
        "mot `note` toan zero-width phai doc la RONG, khong phai mot chuoi vo hinh"
    );

    // Đối chứng: zero-width ĐI KÈM nội dung thật chỉ bị cắt khỏi giá trị, không giết cả ô.
    let text = "source_term,translation,note\n萧炎,Tieu\u{200B} Viem,ghi\u{200D} chu\n";
    let parsed = parse(text).expect("zero-width di kem noi dung that khong phai loi");
    assert_eq!(parsed.rows[0].translation.as_deref(), Some("Tieu Viem"));
    assert_eq!(parsed.rows[0].note, "ghi chu");
}

/// P6 (vòng rà ba lớp) — rào công thức là DELIMITER-AGNOSTIC, nhưng trước ca này mọi ca
/// round-trip của nó chỉ chạy trên `Delimiter::Csv`, nên mệnh đề "áp cho cả hai" không có
/// phép kiểm nào.
#[test]
fn the_formula_guard_round_trips_identically_under_both_delimiters() {
    for delimiter in [Delimiter::Csv, Delimiter::Tsv] {
        for original in ["=1+1", "+A1", "-A1", "@SUM(A1:A2)", "'=1+1", "'abc", "abc"] {
            let mut tier = BTreeMap::new();
            tier.insert("term".to_owned(), entry(1, "term", Some(original)));

            let text = render_tier(&tier, delimiter);
            let parsed = parse(&text).expect("tep vua xuat phai tu phan tich duoc");
            assert_eq!(
                parsed.rows[0].translation.as_deref(),
                Some(original),
                "vong tron phai tra DUNG TUNG BYTE gia tri goc {original:?} voi \
                 delimiter {delimiter:?}"
            );
        }
    }
}

/// P7 (vòng rà ba lớp) — ba ca biên mà chín mục vá của cụm B để trống.
#[test]
fn three_boundary_cases_the_cluster_b_patches_left_uncovered() {
    // ① Ca ĐỐI XỨNG của mục ④: một dấu TAB nằm trong ô đã bọc ở hàng tiêu đề CSV. Trước
    //    mục ④, phép dò chạy trên văn bản THÔ nên ô này làm `has_tab` bật lên và một tệp
    //    CSV hợp lệ bị bác oan là `DelimiterUnresolved`.
    let text = "source_term,translation,\"note\tphu\"\n萧炎,Tieu Viem,ghi chu\n";
    let parsed = parse(text).expect("mot TAB trong o da boc o tieu de CSV khong duoc bac oan");
    assert_eq!(parsed.rows.len(), 1);
    assert_eq!(parsed.rows[0].source_term, "萧炎");

    // ② Một cột đã biết lặp BA lần — đúng ca lam lo cho sai cua câu "xuat hien hai lan"
    //    (P4). Vị từ bắn khi NHIEU HON mot lan, nên ba lần cũng phải bị bác.
    let text = "source_term,translation,translation,translation\n萧炎,a,b,c\n";
    assert_eq!(
        parse(text),
        Err(vec![ParseIssue::DuplicateColumn { column: "translation".to_owned() }]),
        "mot cot lap BA lan cung phai bi bac, va chi bao MOT lan cho cot do"
    );

    // ③ Một ô đã bọc TRỘN `\r\n` với một `\r` trần. Mục ③ chỉ có ca `\r` trần đơn độc, nên
    //    mệnh đề "đếm cả ba dạng ranh giới" chưa được kiểm ở dạng trộn.
    let text = "source_term,translation\n\"dong1\r\ndong2\rdong3\",x\nBAD,y\n";
    let parsed = parse(text).expect("o da boc tron CRLF va CR tran phai phan tich duoc");
    assert_eq!(parsed.rows.len(), 2);
    assert_eq!(
        parsed.rows[1].line, 5,
        "hang sau mot o da boc chua HAI ranh gioi dong phai mang so dong 5 -- dung so ma \
         nguoi dung dem trong trinh soan thao"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// CỤM C — `spec-epic-3-review-cum-c-dong-thoi-duong-commit-nhap.md`
// C1 (so lạc quan) · C3 (kiểm quyết định lạ dời xuống lõi) · I/O Matrix ①②③③b④⑤⑥⑦
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix ① — `TakeTheirs` trên một hàng mà `translation` đã bị MỘT lượt ghi khác đổi
/// GIỮA nhịp preview (`classify()`) và nhịp xác nhận (`import_into_tier()`). Trước bản vá
/// C1, câu `UPDATE … WHERE id = ?2` không so lại `existing_translation` nên lượt ghi kia bị
/// đè mất trong im lặng — 0 lỗi, 0 rollback. Ca này mô phỏng ĐÚNG cửa sổ đua: một lượt ghi
/// THẬT (`update_manual_term`) chen vào SAU khi `classify()` đã chụp ảnh preview.
#[test]
fn take_theirs_is_refused_when_the_translation_changed_under_the_users_feet_between_preview_and_confirm()
 {
    let dir = temp_dir("import-c1-stale-conflict");
    let store = open_global(&dir);
    let id = add_manual_term(&store, None, GlossaryTier::Global, "term", Some("cu"), "", Category::Other)
        .expect("chen truoc");

    // Nhip MOT (preview): nguoi dung thay "cu" ↔ "tu tep" va chon "lay cua file".
    let existing = load_tier(&store).expect("nap tang o nhip preview");
    let plans = classify(&[row("term", Some("tu tep"))], &existing);
    match &plans[0].kind {
        RowPlanKind::Conflict { existing_translation, .. } => {
            assert_eq!(existing_translation.as_deref(), Some("cu"))
        }
        other => panic!("phai la Conflict, nhan: {other:?}"),
    }

    // GIUA hai nhip: mot luot ghi KHAC (sua tay, khong lien quan gi toi luot nhap) doi
    // translation duoi chan nguoi dung -- dung cua so dua ma spec C1 mo ta.
    update_manual_term(&store, None, GlossaryTier::Global, id, Some("da doi boi noi khac"), "", Category::Other)
        .expect("mo phong mot luot ghi KHAC chen vao giua hai nhip");

    let mut decisions = BTreeMap::new();
    decisions.insert("term".to_owned(), ConflictDecision::TakeTheirs);

    let result = import_into_tier(&store, None, GlossaryTier::Global, &plans, &decisions);
    assert_eq!(
        result,
        Err(GlossaryError::ImportStaleConflict { source_terms: vec!["term".to_owned()] }),
        "gia tri da doi duoi chan nguoi dung phai bi tu choi bang mot loi CO TEN RIENG, \
         khong duoc ghi de am tham. Nhan: {result:?}"
    );

    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(
        tier["term"].translation.as_deref(),
        Some("da doi boi noi khac"),
        "0 luot ghi -- gia tri cua luot ghi GIUA chung phai con nguyen, lượt nhap KHONG duoc \
         de len no"
    );

    drop(store);
    cleanup(&dir);
}

/// I/O Matrix ③ — `TakeTheirs` trên một mục CHỜ CHỐT (`existing_translation = None`) mà
/// không ai chen vào: `NULL` phải so khớp `NULL`. Đối chứng NGƯỢC chiều cho C1 — nếu phép so
/// lạc quan không NULL-an-toàn (`= ?3` thay vì `IS ?3`), ca này ĐỎ dù không có cửa sổ đua
/// nào cả, đúng cảnh báo 🔴 của §Always trong spec.
#[test]
fn take_theirs_matches_null_safely_on_a_pending_row_when_nothing_changed_since_preview() {
    let dir = temp_dir("import-c1-null-safe-pending");
    let store = open_global(&dir);
    add_manual_term(&store, None, GlossaryTier::Global, "term", None, "", Category::Other)
        .expect("chen truoc -- CHO CHOT, translation = None");

    let existing = load_tier(&store).expect("nap tang");
    assert_eq!(existing["term"].translation, None);
    let plans = classify(&[row("term", Some("chot lan dau"))], &existing);
    match &plans[0].kind {
        RowPlanKind::Conflict { existing_translation, .. } => assert_eq!(*existing_translation, None),
        other => panic!("phai la Conflict (cho chot khac 'chot lan dau'), nhan: {other:?}"),
    }

    let mut decisions = BTreeMap::new();
    decisions.insert("term".to_owned(), ConflictDecision::TakeTheirs);

    let summary = import_into_tier(&store, None, GlossaryTier::Global, &plans, &decisions).expect(
        "TakeTheirs tren mot muc CHO CHOT khong ai chen vao phai THANH CONG -- NULL IS NULL \
         phai khop",
    );
    assert_eq!(summary.updated, 1);

    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(tier["term"].translation.as_deref(), Some("chot lan dau"));

    drop(store);
    cleanup(&dir);
}

/// I/O Matrix ③b — mục CHỜ CHỐT vừa được MỘT NGƯỜI KHÁC chốt giữa nhịp preview và nhịp xác
/// nhận: `existing_translation` chụp ở preview là `None`, nhưng đĩa hiện đã mang `Some(..)`.
/// Cùng lỗi CÓ TÊN như ① — không phải một nhánh riêng.
#[test]
fn take_theirs_is_refused_when_a_pending_row_was_confirmed_by_someone_else_between_preview_and_confirm()
 {
    let dir = temp_dir("import-c1-pending-confirmed-elsewhere");
    let store = open_global(&dir);
    let id = add_manual_term(&store, None, GlossaryTier::Global, "term", None, "", Category::Other)
        .expect("chen truoc -- CHO CHOT");

    let existing = load_tier(&store).expect("nap tang o nhip preview");
    let plans = classify(&[row("term", Some("tu tep"))], &existing);
    match &plans[0].kind {
        RowPlanKind::Conflict { existing_translation, .. } => assert_eq!(*existing_translation, None),
        other => panic!("phai la Conflict, nhan: {other:?}"),
    }

    // GIUA hai nhip: MOT NGUOI KHAC chot ban dich cho muc nay.
    update_manual_term(
        &store,
        None,
        GlossaryTier::Global,
        id,
        Some("da chot boi nguoi khac"),
        "",
        Category::Other,
    )
    .expect("mo phong mot nguoi khac chot ban dich giua hai nhip");

    let mut decisions = BTreeMap::new();
    decisions.insert("term".to_owned(), ConflictDecision::TakeTheirs);

    let result = import_into_tier(&store, None, GlossaryTier::Global, &plans, &decisions);
    assert_eq!(
        result,
        Err(GlossaryError::ImportStaleConflict { source_terms: vec!["term".to_owned()] }),
        "mot muc CHO CHOT vua duoc CHOT o noi khac phai bi tu choi CUNG mot loi voi ca ① -- \
         khong phai mot nhanh rieng. Nhan: {result:?}"
    );

    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(
        tier["term"].translation.as_deref(),
        Some("da chot boi nguoi khac"),
        "0 luot ghi -- ban dich da chot o noi khac phai con nguyen"
    );

    drop(store);
    cleanup(&dir);
}

/// I/O Matrix ④ — hàng biến mất GIỮA nhịp preview và nhịp xác nhận (một lượt Xoá khác chen
/// vào). Hành vi GIỮ NGUYÊN như trước bản vá C1 (`row_missing_error` ⇒ `GlossaryError::Store`)
/// — KHÔNG được gộp vào lỗi CÓ TÊN của ①/③b, dù cả hai đều xuất phát từ cùng một
/// `changed == 0`. Đây là ca TRUNG TÂM chứng minh `changed == 0` đã được phân biệt đúng HAI
/// nghĩa sau bản vá.
#[test]
fn take_theirs_keeps_the_pre_existing_row_missing_behavior_when_the_row_was_deleted_mid_flight() {
    let dir = temp_dir("import-c1-row-deleted-mid-flight");
    let store = open_global(&dir);
    let id = add_manual_term(&store, None, GlossaryTier::Global, "term", Some("cu"), "", Category::Other)
        .expect("chen truoc");

    let existing = load_tier(&store).expect("nap tang o nhip preview");
    let plans = classify(&[row("term", Some("tu tep"))], &existing);

    // GIUA hai nhip: hang bi XOA boi mot luot khac -- KHAC han ca ① (hang con song, chi doi
    // gia tri).
    delete_manual_term(&store, None, GlossaryTier::Global, id).expect("mo phong mot luot Xoa chen vao");

    let mut decisions = BTreeMap::new();
    decisions.insert("term".to_owned(), ConflictDecision::TakeTheirs);

    let result = import_into_tier(&store, None, GlossaryTier::Global, &plans, &decisions);
    assert!(
        matches!(result, Err(GlossaryError::Store(_))),
        "hang bi XOA phai giu HANH VI CU (loi Store chung), KHONG duoc gan nham nhan \
         ImportStaleConflict -- hai ca nay la hai cau chuyen khac nhau. Nhan: {result:?}"
    );

    drop(store);
    cleanup(&dir);
}

/// I/O Matrix ⑤ — C3 dời xuống lõi: gọi THẲNG `import_into_tier` (bỏ qua hẳn lớp
/// `commands::glossary`) với một quyết định trỏ một thuật ngữ KHÔNG có trong lô. Trước bản
/// vá, chỉ lớp `commands` chặn ca này — một chỗ gọi thẳng (13 lời gọi thẳng đã có trong tệp
/// này) sẽ ghi quyết định đó mà không ai kiểm.
#[test]
fn import_into_tier_rejects_a_decision_pointing_at_a_term_absent_from_the_batch_entirely() {
    let dir = temp_dir("import-c3-unknown-term-core-level");
    let store = open_global(&dir);

    let plans = classify(&[row("moc", Some("Moc Dung"))], &BTreeMap::new());
    let mut decisions = BTreeMap::new();
    decisions.insert("khong-co-trong-lo".to_owned(), ConflictDecision::TakeTheirs);

    let result = import_into_tier(&store, None, GlossaryTier::Global, &plans, &decisions);
    assert_eq!(
        result,
        Err(GlossaryError::ImportDecisionUnknownTerm { term: "khong-co-trong-lo".to_owned() }),
        "mot khoa la trong ban do quyet dinh phai bi tu choi NGAY O LOI, truoc khi mo giao \
         dich -- nhan: {result:?}"
    );

    let tier = load_tier(&store).expect("nap lai");
    assert!(tier.is_empty(), "0 luot ghi -- kiem TRUOC khi mo giao dich, hang New cung khong duoc chen");

    drop(store);
    cleanup(&dir);
}

/// I/O Matrix ⑥ — cùng luật, gọi THẲNG lõi với một quyết định trỏ một hàng `New` (không phải
/// `Conflict`). `New` không có khái niệm "giữ của tôi"/"lấy của file", nên một quyết định trỏ
/// vào nó phải bị từ chối NHƯ MỘT THUẬT NGỮ LẠ, không được lặng lẽ bỏ qua (P5, giữ nguyên khi
/// dời tầng xuống `import_into_tier`).
#[test]
fn import_into_tier_rejects_a_decision_pointing_at_a_new_row_instead_of_a_conflict() {
    let dir = temp_dir("import-c3-decision-on-new-row-core-level");
    let store = open_global(&dir);
    add_manual_term(&store, None, GlossaryTier::Global, "ly", Some("Ly cu"), "", Category::Other)
        .expect("dung truoc de tao Conflict");

    let existing = load_tier(&store).expect("nap tang");
    // "ly" phan loai Conflict; "moc" phan loai New.
    let plans = classify(&[row("ly", Some("Ly moi")), row("moc", Some("Moc Dung"))], &existing);
    assert!(matches!(plans[0].kind, RowPlanKind::Conflict { .. }));
    assert_eq!(plans[1].kind, RowPlanKind::New);

    let mut decisions = BTreeMap::new();
    decisions.insert("moc".to_owned(), ConflictDecision::TakeTheirs);

    let result = import_into_tier(&store, None, GlossaryTier::Global, &plans, &decisions);
    assert_eq!(
        result,
        Err(GlossaryError::ImportDecisionUnknownTerm { term: "moc".to_owned() }),
        "quyet dinh tro vao hang New phai bi tu choi, khong duoc lang le bo qua. Nhan: {result:?}"
    );

    let tier = load_tier(&store).expect("nap lai");
    assert!(!tier.contains_key("moc"), "0 luot ghi -- hang New cung khong duoc chen");

    drop(store);
    cleanup(&dir);
}

/// I/O Matrix ⑦ — một lô có CẢ va `UNIQUE` (hàng `New`) LẪN một hàng ① (va lạc quan) trong
/// CÙNG một lượt. Thứ tự báo là TẤT ĐỊNH: `ImportUniqueConflict` được ưu tiên, đúng thứ tự
/// khối kiểm `local_conflicts`/`local_stale_conflicts` đã đứng trong `import_into_tier` — xem
/// khối ⚠️ ở doc-comment của `GlossaryError::ImportStaleConflict`.
#[test]
fn a_batch_with_both_a_unique_conflict_and_a_stale_optimistic_conflict_reports_the_unique_conflict_first()
 {
    let dir = temp_dir("import-c1-c3-mixed-unique-and-stale");
    let store = open_global(&dir);
    let id = add_manual_term(&store, None, GlossaryTier::Global, "term", Some("cu"), "", Category::Other)
        .expect("hang se thanh va lac quan");

    let existing = load_tier(&store).expect("nap tang o nhip preview");
    // "da-ton-tai" duoc chup anh la New luc classify (chua co trong tang luc do).
    // "term" duoc chup anh la Conflict, existing_translation = Some("cu").
    let plans = classify(&[row("da-ton-tai", Some("tu tep")), row("term", Some("tu tep khac"))], &existing);

    // GIUA hai nhip: HAI luot ghi khac chen vao -- mot lam "da-ton-tai" va UNIQUE, mot doi
    // "term" duoi chan nguoi dung.
    add_manual_term(&store, None, GlossaryTier::Global, "da-ton-tai", Some("da co tu noi khac"), "", Category::Other)
        .expect("mo phong luot ghi lam va UNIQUE");
    update_manual_term(&store, None, GlossaryTier::Global, id, Some("da doi boi noi khac"), "", Category::Other)
        .expect("mo phong luot ghi lam va lac quan");

    let mut decisions = BTreeMap::new();
    decisions.insert("term".to_owned(), ConflictDecision::TakeTheirs);

    let result = import_into_tier(&store, None, GlossaryTier::Global, &plans, &decisions);
    assert_eq!(
        result,
        Err(GlossaryError::ImportUniqueConflict { source_terms: vec!["da-ton-tai".to_owned()] }),
        "khi CA HAI danh sach cung khong rong, ImportUniqueConflict phai duoc bao TRUOC -- \
         thu tu TAT DINH, khong ngau nhien. Nhan: {result:?}"
    );

    // 0 luot ghi -- ca lo rollback, kem "term" cua vong va lac quan.
    let tier = load_tier(&store).expect("nap lai");
    assert_eq!(tier["term"].translation.as_deref(), Some("da doi boi noi khac"), "0 luot ghi");
    assert_eq!(tier["da-ton-tai"].translation.as_deref(), Some("da co tu noi khac"), "0 luot ghi");

    drop(store);
    cleanup(&dir);
}
