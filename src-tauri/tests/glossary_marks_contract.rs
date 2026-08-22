//! Mọi hàng của I/O Matrix — Story 3.4 (`marks_for_source_text` + `glossary_marks_for_chapter`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO TỆP NÀY, KHÔNG THÊM VÀO `glossary_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! Cùng lý do `glossary_commands_contract.rs` tách khỏi `glossary_contract.rs`: đây là
//! hợp đồng của MỘT hàm phơi ra mới (`marks_for_source_text`) cộng bề mặt IPC của nó
//! (`commands::glossary::glossary_marks_for_chapter`), không phải một phép kiểm rải rác
//! thêm vào một tệp đã có gần 60 ca cho ba hàm khác. Tên hàm test là một CÂU khẳng định
//! (`AGENTS.md`), và mỗi ca ứng với ĐÚNG một hàng của I/O Matrix trong spec.
//!
//! Dựng fixture qua `create_work_from_text` — đúng khuôn
//! `glossary_commands_contract.rs`/`project_contract.rs`, không phải một cách dựng riêng.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::glossary::glossary_marks_for_chapter;
use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::core::glossary::{
    Category, GlossaryError, GlossaryTier, add_manual_term, marks_for_source_text,
};
use auratranslate_lib::core::matching::MatchLang;
use auratranslate_lib::core::scope::ScopeResolver;
use auratranslate_lib::core::store::{Store, StoreSpec};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-glossary-marks-{}-{}-{}",
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

/// Đúng khuôn `glossary_commands_contract.rs` — `create_work_from_text` là chỗ SẢN PHẨM
/// DUY NHẤT dựng một `OpenWork` với `ScopeResolver::with_work(...)` thật.
fn open_work(root: &Path, tag: &str) -> auratranslate_lib::commands::project::OpenWork {
    create_work_from_text(root, tag, "zh", "", "noi dung mau".to_owned())
        .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"))
}

// ═════════════════════════════════════════════════════════════════════════════════
// Trung, khớp chính xác
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn chinese_exact_match_marks_the_whole_term() {
    let global_dir = temp_dir("zh-exact-global");
    let global = open_global(&global_dir);
    add_manual_term(
        &global,
        None,
        GlossaryTier::Global,
        "中國",
        Some("Trung Quoc"),
        "",
        Category::Place,
    )
    .expect("them thuat ngu vao tang Global");

    let resolver = ScopeResolver::global_only();
    let marks = marks_for_source_text(&resolver, &global, None, "中國人", MatchLang::Zh)
        .expect("khop khong duoc loi");

    assert_eq!(marks.len(), 1, "phai co dung mot dau: {marks:?}");
    assert_eq!(marks[0].start, 0);
    assert_eq!(marks[0].end, 2, "dau phai phu dung hai diem ma dau cua 中國人 -- 中國");
    assert_eq!(marks[0].tier, GlossaryTier::Global);
    assert!(marks[0].is_confirmed);
    assert_eq!(marks[0].translation.as_deref(), Some("Trung Quoc"));

    drop(global);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Cắt ngang một từ jieba
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_term_cutting_across_a_jieba_token_boundary_produces_no_mark() {
    let global_dir = temp_dir("zh-cut-global");
    let global = open_global(&global_dir);
    add_manual_term(&global, None, GlossaryTier::Global, "文", Some("van"), "", Category::Other)
        .expect("them thuat ngu vao tang Global");

    let resolver = ScopeResolver::global_only();
    // `文化` la MOT token cua jieba (tu dien mac dinh) -- `文` cat ngang giua token do, nen
    // khong duoc nhan. Xem doc-comment cua `find_terms` cho cung dung vi du nay.
    let marks =
        marks_for_source_text(&resolver, &global, None, "文化", MatchLang::Zh).expect("khong loi");

    assert!(marks.is_empty(), "文 cat ngang token 文化 -- khong duoc co dau nao: {marks:?}");

    drop(global);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Anh, biến thể hình thái
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_english_inflected_form_is_marked_from_its_base_term() {
    let global_dir = temp_dir("en-inflect-global");
    let global = open_global(&global_dir);
    add_manual_term(&global, None, GlossaryTier::Global, "run", Some("chay"), "", Category::Other)
        .expect("them thuat ngu vao tang Global");

    let resolver = ScopeResolver::global_only();
    let text = "the dog is running now";
    let marks =
        marks_for_source_text(&resolver, &global, None, text, MatchLang::En).expect("khong loi");

    assert_eq!(marks.len(), 1, "phai co dung mot dau: {marks:?}");
    assert_eq!(&text[marks[0].start..marks[0].end], "running", "dau phai phu dung tu running");
    assert_eq!(marks[0].translation.as_deref(), Some("chay"));

    drop(global);
    cleanup(&global_dir);
}

/// Story 3.6 — mệnh đề mà cả story đứng lên: bề mặt cắt từ văn bản KHÔNG phải khoá ghi. Một
/// dải chốt đọc `GlossaryMark::source_term` (không phải `text[start..end]`) để biết ghi
/// đúng hàng `dragon` nào, kể cả khi bề mặt trên màn hình là `dragons`.
#[test]
fn an_english_inflected_surface_carries_the_base_source_term_not_the_surface() {
    let global_dir = temp_dir("en-inflect-key-global");
    let global = open_global(&global_dir);
    let entry_id = add_manual_term(
        &global,
        None,
        GlossaryTier::Global,
        "dragon",
        Some("rong"),
        "",
        Category::Other,
    )
    .expect("them thuat ngu vao tang Global");

    let resolver = ScopeResolver::global_only();
    let text = "the dragons flew over the mountain";
    let marks =
        marks_for_source_text(&resolver, &global, None, text, MatchLang::En).expect("khong loi");

    assert_eq!(marks.len(), 1, "phai co dung mot dau: {marks:?}");
    assert_eq!(
        &text[marks[0].start..marks[0].end],
        "dragons",
        "be mat tren man hinh la dang so nhieu"
    );
    assert_eq!(
        marks[0].source_term, "dragon",
        "khoa ghi phai la GOC, khong phai be mat da khop tren man hinh"
    );
    assert_eq!(marks[0].id, entry_id, "id phai dung hang glossary_entry vua them");

    drop(global);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Anh, cực cấp -- GIỚI HẠN CÓ TÊN, Ice ký 2026-08-21 (Porter2 khong co luat -er/-est)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn english_superlative_forms_are_not_marked_a_named_porter2_limit_ice_signed_2026_08_21() {
    let global_dir = temp_dir("en-superlative-global");
    let global = open_global(&global_dir);
    add_manual_term(
        &global,
        None,
        GlossaryTier::Global,
        "happy",
        Some("hanh phuc"),
        "",
        Category::Other,
    )
    .expect("them thuat ngu vao tang Global");

    let resolver = ScopeResolver::global_only();
    let marks = marks_for_source_text(
        &resolver,
        &global,
        None,
        "she is the happiest person i know",
        MatchLang::En,
    )
    .expect("khong loi");

    // `deferred-work.md:422`: Porter2 KHONG co luat cho hau to so sanh/cuc cap (`-er`/`-est`)
    // -- `happiest` khong ve duoc `happy` (`happi`). Day la mot GIOI HAN DA DO, khong phai
    // mot cho chua lam: ghi ra bang mot ca test co ten thay vi de nguoi sau tuong no da
    // duoc xet.
    assert!(
        marks.is_empty(),
        "happiest KHONG duoc khop voi thuat ngu happy -- gioi han Porter2 da ky: {marks:?}"
    );

    drop(global);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mục chờ chốt
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_pending_entry_is_marked_with_is_confirmed_false_and_translation_null() {
    let global_dir = temp_dir("zh-pending-global");
    let global = open_global(&global_dir);
    add_manual_term(&global, None, GlossaryTier::Global, "慕容", None, "", Category::Person)
        .expect("them muc cho chot vao tang Global");

    let resolver = ScopeResolver::global_only();
    let marks = marks_for_source_text(&resolver, &global, None, "慕容说话了", MatchLang::Zh)
        .expect("khong loi");

    assert_eq!(marks.len(), 1, "muc cho chot van phai ra dau: {marks:?}");
    assert!(!marks[0].is_confirmed, "muc cho chot phai mang is_confirmed=false");
    assert_eq!(marks[0].translation, None, "muc cho chot khong co ban dich");

    drop(global);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Trùng hai tầng -- AD-18
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_work_tier_wins_over_global_when_both_tiers_share_a_source_term() {
    let root = temp_dir("zh-ad18");
    let global_dir = temp_dir("zh-ad18-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "AD18 Marks");

    add_manual_term(
        &global,
        None,
        GlossaryTier::Global,
        "青丘",
        Some("Thanh Khau Toan Cuc"),
        "",
        Category::Place,
    )
    .expect("them muc o tang Global");
    add_manual_term(
        &global,
        Some(&opened.store),
        GlossaryTier::Work,
        "青丘",
        Some("Thanh Khau Rieng"),
        "",
        Category::Place,
    )
    .expect("them muc trung ten o tang Tac pham");

    let marks = marks_for_source_text(
        &opened.scope,
        &global,
        Some(&opened.store),
        "青丘之地",
        MatchLang::Zh,
    )
    .expect("khong loi");

    assert_eq!(marks.len(), 1, "trung thuat ngu hai tang van phai ra DUNG MOT dau: {marks:?}");
    assert_eq!(marks[0].tier, GlossaryTier::Work, "tang Tac pham phai thang (AD-18)");
    assert_eq!(marks[0].translation.as_deref(), Some("Thanh Khau Rieng"));

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Chưa mở Tác phẩm
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn with_no_work_open_only_the_global_tier_is_matched_and_no_error_is_raised() {
    let global_dir = temp_dir("en-no-work-global");
    let global = open_global(&global_dir);
    add_manual_term(
        &global,
        None,
        GlossaryTier::Global,
        "hello",
        Some("xin chao"),
        "",
        Category::Other,
    )
    .expect("them thuat ngu vao tang Global");

    // Qua ham LOI (core), khong OpenWork nao ca.
    let resolver = ScopeResolver::global_only();
    let marks = marks_for_source_text(&resolver, &global, None, "hello world", MatchLang::En)
        .expect("chua mo Tac pham KHONG duoc la mot loi");
    assert_eq!(marks.len(), 1);
    assert_eq!(marks[0].tier, GlossaryTier::Global);

    // Va qua chinh be mat IPC (`commands::glossary::glossary_marks_for_chapter`) voi
    // `open: None` -- cung mot menh de, do o CA HAI tang.
    let via_command = glossary_marks_for_chapter(Some(&global), None, "hello world", "en")
        .expect("lenh IPC cung khong duoc loi khi chua mo Tac pham");
    assert_eq!(via_command.len(), 1);
    assert_eq!(via_command[0].tier, "global");

    drop(global);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Chồng nhau — span dài nhất thắng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn overlapping_matches_of_different_lengths_keep_only_the_longest_span() {
    // 🔴 Hai tang KHAC NHAU, hai ban dich KHAC NHAU -- khong chi hai `start`/`end`. Neu
    // `marks_for_source_text` xao tron anh xa `terms` (tu `resolved.keys()`) <-> `payload`
    // (tu `resolved.values()`), ca nay se di qua neu chi kiem `start`/`end` (ca hai ung vien
    // cung phu dung mot cho): dau con lai co the mang dung SPAN nhung SAI tang/ban dich cua
    // mot thuat ngu KHAC. `tier`/`translation` duoi day moi la thu phan biet duoc loi do.
    let root = temp_dir("zh-overlap-longest");
    let global_dir = temp_dir("zh-overlap-longest-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Overlap Longest");

    add_manual_term(&global, None, GlossaryTier::Global, "中國", Some("a"), "", Category::Other)
        .expect("them 中國 o tang Global");
    add_manual_term(
        &global,
        Some(&opened.store),
        GlossaryTier::Work,
        "中國人",
        Some("b"),
        "",
        Category::Other,
    )
    .expect("them 中國人 o tang Tac pham");

    let marks = marks_for_source_text(
        &opened.scope,
        &global,
        Some(&opened.store),
        "中國人",
        MatchLang::Zh,
    )
    .expect("khong loi");

    assert_eq!(
        marks.len(),
        1,
        "中國 va 中國人 cung khop tai cung mot cho -- phai con DUNG MOT dau: {marks:?}"
    );
    assert_eq!(marks[0].start, 0);
    assert_eq!(marks[0].end, 3, "span DAI NHAT (中國人, ba diem ma) phai thang, khong phai 中國");
    assert_eq!(
        marks[0].tier,
        GlossaryTier::Work,
        "dau con lai phai la CUA 中國人 (tang Tac pham), khong phai cua 中國 (tang Global) \
         chay lon anh xa term_index<->payload"
    );
    assert_eq!(
        marks[0].translation.as_deref(),
        Some("b"),
        "ban dich phai la cua 中國人 ('b'), khong phai cua 中國 ('a')"
    );

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Chồng nhau — hoà thì trái nhất
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn overlapping_matches_of_equal_length_keep_only_the_leftmost_span() {
    let global_dir = temp_dir("zh-overlap-tie-global");
    let global = open_global(&global_dir);
    // 🔴 HAI thuat ngu KHAC NHAU (khong phai cung mot thuat ngu khop hai lan) -- moi phan
    // biet duoc mot loi xao tron `term_index` bang `translation`: dung lai CUNG mot thuat
    // ngu se khong bao gio lo loi do (ca hai lan khop deu tro ve DUNG mot payload).
    //
    // `𠧜`(U+209DC)/`𠀀`(U+20000)/`𠀁`(U+20001) deu ngoai BMP, deu 4 byte UTF-8, va deu KHONG
    // co trong tu dien jieba mac dinh -- nen `𠧜𠀀𠀁` cat ra TUNG ky tu rieng (cung ly le voi
    // `matching_contract.rs::overlapping_occurrences_of_the_same_chinese_term_are_all_reported`).
    // "𠧜𠀀" khop byte 0..8; "𠀀𠀁" khop byte 4..12 -- CUNG do dai (8 byte, hai diem ma), chong
    // nhau (0<12 && 4<8). Ben trai ("𠧜𠀀") phai thang.
    add_manual_term(&global, None, GlossaryTier::Global, "𠧜𠀀", Some("trai"), "", Category::Other)
        .expect("them 𠧜𠀀");
    add_manual_term(&global, None, GlossaryTier::Global, "𠀀𠀁", Some("phai"), "", Category::Other)
        .expect("them 𠀀𠀁");

    let resolver = ScopeResolver::global_only();
    let marks = marks_for_source_text(&resolver, &global, None, "𠧜𠀀𠀁", MatchLang::Zh)
        .expect("khong loi");

    assert_eq!(marks.len(), 1, "hai lan khop chong nhau cung do dai -- phai con DUNG MOT: {marks:?}");
    assert_eq!(marks[0].start, 0, "hoa thi TRAI NHAT phai thang");
    assert_eq!(marks[0].end, 2);
    assert_eq!(
        marks[0].translation.as_deref(),
        Some("trai"),
        "dau con lai phai la CUA 𠧜𠀀 ('trai'), khong phai cua 𠀀𠀁 ('phai') -- neu day la \
         'phai' thi term_index dang tro sai payload sau resolve_overlaps"
    );

    drop(global);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ký tự ngoài BMP
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_term_outside_the_bmp_produces_a_correct_codepoint_span_without_panicking() {
    let global_dir = temp_dir("zh-bmp-global");
    let global = open_global(&global_dir);
    add_manual_term(&global, None, GlossaryTier::Global, "𠧜", Some("x"), "", Category::Other)
        .expect("them thuat ngu chua ky tu ngoai BMP");

    let resolver = ScopeResolver::global_only();
    // "你" (1 diem ma, 3 byte) roi "𠧜" (1 diem ma, 4 byte) -- span DIEM MA cua 𠧜 phai la
    // 1..2, KHONG phai 1..3 (UTF-16, hai code unit) va KHONG phai lech theo byte.
    let marks = marks_for_source_text(&resolver, &global, None, "你𠧜", MatchLang::Zh)
        .expect("khong panic, khong loi");

    assert_eq!(marks.len(), 1, "{marks:?}");
    assert_eq!(marks[0].start, 1, "𠧜 dung sau DUNG MOT diem ma (你), khong phai sau 3 byte");
    assert_eq!(marks[0].end, 2, "𠧜 la DUNG MOT diem ma, khong phai hai (UTF-16) hay bon (byte)");

    drop(global);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Glossary rỗng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_empty_glossary_matches_nothing_and_ok_empty_is_distinguishable_from_a_lookup_failure() {
    let global_dir = temp_dir("empty-global");
    let global = open_global(&global_dir);

    let resolver = ScopeResolver::global_only();
    let result = marks_for_source_text(&resolver, &global, None, "bat ky van ban nao", MatchLang::En);

    // `Ok(vec![])` -- mot KET QUA thanh cong rong, phan biet duoc voi `Err(..)` (ca ke tiep)
    // boi chinh KIEU `Result`, khong can mot co rieng.
    match result {
        Ok(marks) => assert!(marks.is_empty(), "glossary rong -- khong dau nao: {marks:?}"),
        Err(e) => panic!("glossary rong KHONG phai mot loi: {e}"),
    }

    drop(global);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// `Store` đóng giữa chừng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_missing_global_store_is_an_error_carrying_a_message_key_not_an_empty_ok() {
    // `global: None` -- kho chua bao gio duoc `app.manage`, cung hinh dang "kho khong mo
    // duoc" ma moi lenh khac cua `commands::glossary` dung `store_is_missing()` de dien dat.
    let err = glossary_marks_for_chapter(None, None, "van ban", "en")
        .expect_err("global.db vang mat PHAI la mot loi, khong `Ok(vec![])`");

    assert_eq!(err.code(), "store.open_failed", "phai mang dung message_key cua store");
}

/// 🔴 Nhánh RIÊNG của `marks_for_source_text` -- `work.map(load_tier).transpose()?` --
/// khác hẳn nhánh `global_tier = load_tier(global)?` mà ca ngay trên đã phủ. Đóng tầng Tác
/// phẩm GIỮA CHỪNG (`Store::close()`, idempotent, an toàn gọi trước `drop`) mô phỏng đúng
/// hình dạng "kho đóng giữa chừng" của I/O Matrix cho đúng nhánh này: mọi lượt `.read()` sau
/// đó trả `StoreError::PoolClosed` (`core/store/reader.rs::acquire`), không phải một tệp bị
/// xoá hay chưa từng mở.
#[test]
fn a_work_tier_store_closed_mid_session_is_an_error_not_an_empty_ok() {
    let root = temp_dir("work-store-closed");
    let global_dir = temp_dir("work-store-closed-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "Kho Dong Giua Chung");

    // Dong TAY tang Tac pham SAU khi da mo qua `ScopeResolver::with_work` that (qua
    // `open_work`) -- `resolver.has_work_tier()` van `true`, dung khuon voi `work.is_some()`
    // duoi day nen khong cham `debug_assert_eq!` cua `marks_for_source_text`.
    opened.store.close();

    let err = marks_for_source_text(
        &opened.scope,
        &global,
        Some(&opened.store),
        "van ban bat ky",
        MatchLang::Zh,
    )
    .expect_err("tang Tac pham dong giua chung PHAI la mot loi, khong `Ok(vec![])`");

    match err {
        GlossaryError::Store(_) => {}
        other => panic!("ky vong GlossaryError::Store (tu nhanh work), nhan {other:?}"),
    }

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Trùng hai tầng qua CHÍNH bề mặt IPC -- AD-18 đi qua `glossary_marks_for_chapter`,
// không chỉ qua `marks_for_source_text` lắp tay
// ═════════════════════════════════════════════════════════════════════════════════

/// Ca `the_work_tier_wins_over_global_when_both_tiers_share_a_source_term` ở trên gọi thẳng
/// `marks_for_source_text` với `resolver`/`store` dựng tay -- nó KHÔNG đi qua
/// `work_context(open)` bên trong `commands::glossary::glossary_marks_for_chapter`, tức
/// không nghiệm thu được đường mà bề mặt IPC thật sự bóc `OpenWork` ra hai nửa
/// `(&Store, &ScopeResolver)`. Ca này lấp đúng khoảng đó: gọi `glossary_marks_for_chapter`
/// với `open: Some(&opened)` thật.
#[test]
fn the_work_tier_wins_over_global_through_the_real_glossary_marks_for_chapter_surface() {
    let root = temp_dir("ad18-marks-command");
    let global_dir = temp_dir("ad18-marks-command-global");
    let global = open_global(&global_dir);
    let opened = open_work(&root, "AD18 Marks Command");

    add_manual_term(
        &global,
        None,
        GlossaryTier::Global,
        "青丘",
        Some("Thanh Khau Toan Cuc"),
        "",
        Category::Place,
    )
    .expect("them muc o tang Global");
    add_manual_term(
        &global,
        Some(&opened.store),
        GlossaryTier::Work,
        "青丘",
        Some("Thanh Khau Rieng"),
        "",
        Category::Place,
    )
    .expect("them muc trung ten o tang Tac pham");

    let marks = glossary_marks_for_chapter(Some(&global), Some(&opened), "青丘之地", "zh")
        .expect("khong loi qua be mat IPC that");

    assert_eq!(marks.len(), 1, "trung thuat ngu hai tang qua IPC van phai ra DUNG MOT dau: {marks:?}");
    assert_eq!(marks[0].tier, "work", "tang Tac pham phai thang (AD-18) qua chinh be mat IPC");
    assert_eq!(marks[0].translation.as_deref(), Some("Thanh Khau Rieng"));

    drop(global);
    drop(opened);
    cleanup(&root);
    cleanup(&global_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Chất nối `\n` không cho một dấu bắc cầu — Story 3.4b, AC "đo trên cả hai nhánh Zh và En"
// ═════════════════════════════════════════════════════════════════════════════════
//
// `3-4b-danh-dau-thuat-ngu-o-cot-nguyen-van-cua-luoi.md` §Design Notes nối `segment.source_text`
// bằng `\n` để chia mark tuyệt đối về từng segment ở tầng TypeScript (`glossaryMarksMap.ts`).
// Phép nối đó CHỈ đúng nếu KHÔNG dấu nào từ `marks_for_source_text` bắc cầu qua đúng ký tự
// `\n` mà chính nó chèn vào -- nếu có, một thuật ngữ đặt sát biên hai segment sẽ vẽ dấu lấn
// sang segment kế bên, một nguồn dữ liệu người dùng KHÔNG viết ra.
//
// 🔴 Đây LÀ mệnh đề mà spec ghi "chưa ai đo" -- hai ca dưới đây đo THẬT qua chính bề mặt
// `marks_for_source_text`, không suy từ đọc mã. Đặt ở tệp này (không ở `matching_boundary.rs`)
// vì bề mặt MARK là chủ của mệnh đề "chất nối `\n`" -- `core::matching` không biết gì về việc
// ai nối segment bằng ký tự gì; nó chỉ tình cờ có sẵn luật "từ chối bắc cầu qua `\n`" từ trước
// (2026-08-05, cho ranh giới CÂU), và story 3.4b dựa vào đúng luật có sẵn đó.

#[test]
fn a_chinese_term_placed_right_across_the_newline_joiner_produces_no_mark() {
    let global_dir = temp_dir("nl-bridge-zh-global");
    let global = open_global(&global_dir);
    // Chép NGUYÊN VÍ DỤ đã có trong doc-comment của `find_terms` (`core/matching/mod.rs`):
    // "萧炎" không có trong từ điển jieba nên rơi ra TỪNG KÝ TỰ (HMM = false) -- tức mọi biên
    // ký tự đều là biên token, và phép khớp không phụ thuộc việc jieba có "biết" cụm này.
    add_manual_term(&global, None, GlossaryTier::Global, "萧炎", Some("Tieu Viem"), "", Category::Person)
        .expect("them thuat ngu vao tang Global");

    let resolver = ScopeResolver::global_only();

    // ── ĐỐI CHỨNG DƯƠNG: liền nhau (không `\n`) ⇒ PHẢI khớp -- chứng minh ca âm dưới không
    //    phải "thuật ngữ này chưa từng khớp được" mà đúng là `\n` đã chặn nó.
    let lien =
        marks_for_source_text(&resolver, &global, None, "萧炎和林动", MatchLang::Zh).expect("khong loi");
    assert_eq!(lien.len(), 1, "doi chung: 萧炎 lien nhau phai khop dung mot dau: {lien:?}");
    assert_eq!((lien[0].start, lien[0].end), (0, 2), "dau phai phu dung hai diem ma dau cua 萧炎");

    // ── CA CHÍNH: `萧` kết thúc "segment 1", `炎和林动` mở "segment 2", nối bằng `\n` --
    //    đúng hình dạng chuỗi mà `glossaryMarksMap.ts::joinSegmentSourceText` dựng.
    let bac_cau = marks_for_source_text(&resolver, &global, None, "萧\n炎和林动", MatchLang::Zh)
        .expect("khong loi");
    assert!(
        bac_cau.is_empty(),
        "0 dau bac cau qua \\n -- 萧炎 KHONG duoc khop khi bi \\n chen giua: {bac_cau:?}"
    );

    drop(global);
    cleanup(&global_dir);
}

#[test]
fn an_english_multi_word_term_placed_right_across_the_newline_joiner_produces_no_mark() {
    let global_dir = temp_dir("nl-bridge-en-global");
    let global = open_global(&global_dir);
    add_manual_term(
        &global,
        None,
        GlossaryTier::Global,
        "fire dragon",
        Some("Hoa Long"),
        "",
        Category::Other,
    )
    .expect("them thuat ngu vao tang Global");

    let resolver = ScopeResolver::global_only();

    // ── ĐỐI CHỨNG DƯƠNG: một dấu cách phân tách (không `\n`) ⇒ PHẢI khớp.
    let lien = marks_for_source_text(&resolver, &global, None, "a fire dragon roars", MatchLang::En)
        .expect("khong loi");
    assert_eq!(lien.len(), 1, "doi chung: fire dragon lien nhau phai khop dung mot dau: {lien:?}");
    assert_eq!(
        &"a fire dragon roars"[lien[0].start..lien[0].end],
        "fire dragon",
        "dau phai phu dung cum fire dragon"
    );

    // ── CA CHÍNH: "fire" kết thúc "segment 1", "dragon roars" mở "segment 2", nối bằng `\n`.
    let bac_cau = marks_for_source_text(&resolver, &global, None, "a fire\ndragon roars", MatchLang::En)
        .expect("khong loi");
    assert!(
        bac_cau.is_empty(),
        "0 dau bac cau qua \\n -- fire dragon KHONG duoc khop khi bi \\n chen giua: {bac_cau:?}"
    );

    drop(global);
    cleanup(&global_dir);
}
