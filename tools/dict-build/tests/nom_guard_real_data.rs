//! AC5, mệnh đề "đỏ được": chạy `nom_guard::count_suspicious` trên dữ liệu THẬT — Unihan
//! `kVietnamese` (giả lập hình dạng LỖI của story: đổ thẳng vào `han_viet` như code CŨ
//! đã làm trước Story 1.10c) đối chiếu chéo nguồn với nhãn `nom-reading` THẬT của
//! `en-wiktionary-vi`.
//!
//! Review Findings — nhãn Nôm đối chứng đi qua `nom_guard::nom_only_readings` TRƯỚC khi
//! so (đúng đường production dùng từ bản vá code review 2026-08-06). Con số đỏ vì vậy
//! là **79,5%** *(882/1.109)*, ⛔ không còn **92,4%** như bản đo gốc của §Phát hiện ② —
//! chênh lệch là vì bản vá loại các âm "tự-trùng-vai" (cũng gắn `han-viet-reading` cho
//! CÙNG ký tự) khỏi vế đối chứng để tránh báo động giả trên các nguồn hợp lệ (đo thật:
//! Thiều Chửu tụt từ 63,4% xuống 5,2%, xem `deferred-work.md`). 79,5% vẫn CÁCH XA ngưỡng
//! 50% và cách xa hẳn 5,2–6,5% của hai nguồn hợp lệ — chẩn đoán §Phát hiện ② vẫn đứng.
//!
//! ⚠️ `#[ignore]` — cùng lý do `dict_sources.rs::bench_the_grouped_path_on_the_real_
//! dictionaries`: phụ thuộc `tools/dict-build/raw/**`, vốn `.gitignore` (AD-25) và
//! KHÔNG có trên CI. Chạy tay: `cargo test --test nom_guard_real_data -- --ignored
//! --nocapture`. Số đo ghi vào Dev Agent Record → Completion Notes của story.

use std::io::BufReader;

#[test]
#[ignore = "cần tools/dict-build/raw/unihan + raw/en_wiktionary_vi thật, không có trên CI"]
fn kvietnamese_reproduces_the_historical_92_percent_overlap_on_real_data() {
    let raw_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("raw");
    let unihan_path = raw_dir.join("unihan").join("Unihan_Readings.txt");
    let kaikki_path = raw_dir.join("en_wiktionary_vi").join("kaikki-en-vi.jsonl");

    if !unihan_path.is_file() || !kaikki_path.is_file() {
        eprintln!("bỏ qua: thiếu raw/unihan hoặc raw/en_wiktionary_vi thật cục bộ");
        return;
    }

    // §Phát hiện của story: `kVietnamese` là hình dạng LỖI — nạp thẳng vào `han_viet`,
    // dán nhãn nguồn "unihan-pre-1.10c-bug" để `count_suspicious` so nó XUYÊN NGUỒN với
    // nhãn nom-reading thật của en-wiktionary-vi (⛔ không phải mã nguồn `unihan` thật
    // của story này, vốn giờ chỉ ghi `nom_reading`, không còn ghi `han_viet` nữa).
    let unihan_bytes = std::fs::read(&unihan_path).expect("đọc Unihan_Readings.txt thật");
    let unihan_entries: Vec<dict_build::model::RawEntry> =
        dict_build::sources::unihan::parse(std::io::Cursor::new(unihan_bytes))
            .filter_map(Result::ok)
            .collect();

    // Tái tạo hành vi CŨ (trước story này): kVietnamese → han_viet. `sources::unihan::
    // parse` giờ đã đổi vai (đổ vào `nom_reading`) — đọc lại từ đó để mô phỏng đúng
    // input mà phép kiểm phải bắt được nếu lỗi tái diễn ở một nguồn tương lai.
    let hv_rows: Vec<(String, String, String)> = unihan_entries
        .iter()
        .filter_map(|e| {
            e.nom_reading
                .as_ref()
                .map(|v| (e.headword.clone(), "unihan-pre-1.10c-bug".to_string(), v.clone()))
        })
        .collect();

    let kaikki_file = std::fs::File::open(&kaikki_path).expect("mở kaikki-en-vi.jsonl thật");
    let vi_entries: Vec<dict_build::model::RawEntry> =
        dict_build::sources::en_wiktionary_vi::parse(BufReader::new(kaikki_file))
            .filter_map(Result::ok)
            .collect();
    // Review Findings — cùng phép lọc `nom_only_readings` production dùng (loại âm
    // "tự-trùng-vai" cũng được gắn han-viet-reading cho CÙNG ký tự): không siết vế này,
    // đối chiếu xuyên nguồn báo động giả hàng loạt trên nguồn hợp lệ (đo thật trên
    // Thiều Chửu: 369/582 = 63,4% trước khi siết, 23/441 = 5,2% sau khi siết). Test này
    // phải phản ánh ĐÚNG phép lọc production dùng, không phải một biến thể chưa siết.
    let nom_rows: Vec<(String, String, String)> = vi_entries
        .iter()
        .filter_map(|e| {
            let filtered =
                dict_build::nom_guard::nom_only_readings(e.han_viet.as_deref(), e.nom_reading.as_deref())?;
            Some((e.headword.clone(), "en-wiktionary-vi".to_string(), filtered))
        })
        .collect();

    let hv_refs: Vec<(&str, &str, &str)> = hv_rows
        .iter()
        .map(|(a, b, c)| (a.as_str(), b.as_str(), c.as_str()))
        .collect();
    let nom_refs: Vec<(&str, &str, &str)> = nom_rows
        .iter()
        .map(|(a, b, c)| (a.as_str(), b.as_str(), c.as_str()))
        .collect();

    let result = dict_build::nom_guard::count_suspicious(hv_refs, nom_refs);

    println!(
        "AC5 — đo trên dữ liệu THẬT: {}/{} = {:.1}% ký tự kVietnamese trùng một âm Nôm \
         đã gắn nhãn của en-wiktionary-vi",
        result.suspicious,
        result.total_checked,
        result.ratio() * 100.0
    );

    assert!(
        result.total_checked > 500,
        "phải đối chiếu được một tập đủ lớn trên dữ liệu thật, got {}",
        result.total_checked
    );
    assert!(
        result.exceeds_threshold(),
        "🔴 AC5 'đỏ được': dữ liệu Unihan CŨ phải vượt ngưỡng {:.0}%, đo được {:.1}% \
         ({}/{}) — nếu số này KHÔNG còn đỏ, chẩn đoán §Phát hiện ② của story cần xem lại",
        dict_build::nom_guard::SUSPICIOUS_RATIO_THRESHOLD * 100.0,
        result.ratio() * 100.0,
        result.suspicious,
        result.total_checked
    );
}
