//! Mọi hàng của I/O Matrix + mọi Decision của spec 4.6 (Smart RAG Injector là một hàm thuần)
//! — `core::ai::rag::{gather_glossary_context, assemble_prompt}` cộng cửa Glossary
//! (`core::glossary::confirmed_terms_for_injection`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO TỆP NÀY, KHÔNG THÊM VÀO `glossary_marks_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! Đây là hợp đồng của module `core::ai` (một hàm TẠP + một hàm THUẦN), không của
//! `core::glossary`. `glossary_marks_contract.rs` canh lưới (`marks_for_source_text`); tệp
//! này canh prompt + ledger, và một số ca so hai bên với NHAU trên cùng một câu (AC của
//! story: lưới và ledger không được lệch nhau) — SO Ở LEDGER (`gather_glossary_context`), vì
//! đó là thứ Story 4.7 thật sự đọc, không so ở cửa (`confirmed_terms_for_injection`) rồi suy
//! ra ledger giống hệt cửa.
//!
//! Dựng fixture qua `Store::open`/`add_manual_term`, đúng khuôn `glossary_marks_contract.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use auratranslate_lib::core::ai::rag::{
    GlossaryInjectionStatus, InjectedGlossaryTerm, SuppressedGlossaryTerm, TmInjectionStatus,
    assemble_prompt, gather_glossary_context,
};
use auratranslate_lib::core::glossary::{
    Category, GlossaryError, GlossaryInjectionTerm, GlossaryTier, add_manual_term,
    confirmed_terms_for_injection, marks_for_source_text,
};
use auratranslate_lib::core::matching::{self, MatchLang};
use auratranslate_lib::core::scope::ScopeResolver;
use auratranslate_lib::core::store::{Store, StoreSpec};
use auratranslate_lib::core::tm::SimilarSegment;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-ai-rag-{}-{}-{}",
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

const GLOSSARY_BODY: &str = "Terms: {{glossary_terms}}\nSentence: {{source_segment}}";

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 1 — Confirmed Global term in sentence
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_confirmed_global_term_is_injected_and_listed_in_the_ledger() {
    let dir = temp_dir("confirmed-global");
    let global = open_global(&dir);
    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them thuat ngu");

    let resolver = ScopeResolver::global_only();
    let sentence = "A dragon roared.";
    let status = gather_glossary_context(GLOSSARY_BODY, &resolver, &global, None, "en", sentence)
        .expect("gather khong loi");

    let (prompt, ledger) = assemble_prompt(GLOSSARY_BODY, sentence, status, None);

    assert_eq!(prompt, "Terms: dragon → rong\nSentence: A dragon roared.");
    match &ledger.glossary {
        GlossaryInjectionStatus::Asked { injected, suppressed_by_pending_overlap } => {
            // "A dragon roared." -- "dragon" chiem diem ma 2..8.
            assert_eq!(
                injected,
                &vec![InjectedGlossaryTerm {
                    source_term: "dragon".to_owned(),
                    translation: "rong".to_owned(),
                    start: 2,
                    end: 8,
                    tier: GlossaryTier::Global,
                }]
            );
            assert!(suppressed_by_pending_overlap.is_empty());
        }
        other => panic!("mong Asked, nhan {other:?}"),
    }

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC 2 — "every term the ledger injects is a confirmed mark at the same span" — kiểm Ở
// LEDGER (không phải ở cửa), trên nhiều thuật ngữ cùng lúc.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn ledger_injected_terms_are_at_the_same_span_as_the_grids_confirmed_marks() {
    let dir = temp_dir("ac2-span-cross-check");
    let global = open_global(&dir);
    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them dragon");
    add_manual_term(&global, None, GlossaryTier::Global, "castle", Some("lau dai"), "", Category::Other)
        .expect("them castle");

    let resolver = ScopeResolver::global_only();
    let sentence = "A dragon guards the castle.";

    let status = gather_glossary_context(GLOSSARY_BODY, &resolver, &global, None, "en", sentence)
        .expect("gather khong loi");
    let layers = auratranslate_lib::core::dict::DictLayers::empty();
    let disabled = std::collections::BTreeSet::new();
    let marks =
        marks_for_source_text(&resolver, &global, None, sentence, MatchLang::En, &layers, &disabled)
            .expect("marks khong loi");

    let GlossaryInjectionStatus::Asked { injected, .. } = &status else {
        panic!("mong Asked, nhan {status:?}");
    };
    assert_eq!(injected.len(), 2, "{injected:?}");

    for term in injected {
        let mark = marks
            .iter()
            .find(|m| m.source_term == term.source_term)
            .unwrap_or_else(|| panic!("khong tim thay dau luoi cho '{}': {marks:?}", term.source_term));
        assert!(mark.is_confirmed, "dau luoi cho '{}' phai DA CHOT", term.source_term);
        assert_eq!(
            (term.start, term.end),
            (mark.start, mark.end),
            "ledger va luoi phai khop DUNG SPAN cho '{}'",
            term.source_term
        );
        assert_eq!(term.tier, mark.tier, "ledger va luoi phai khop DUNG TANG cho '{}'", term.source_term);
    }

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 2 — Work overrides Global
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn work_tier_overrides_global_for_the_same_term() {
    let global_dir = temp_dir("override-global");
    let work_dir = temp_dir("override-work");
    let global = open_global(&global_dir);
    let work = open_project(&work_dir);

    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong toan cuc"), "", Category::Other)
        .expect("them global");
    add_manual_term(
        &global,
        Some(&work),
        GlossaryTier::Work,
        "dragon",
        Some("rong tac pham"),
        "",
        Category::Other,
    )
    .expect("them work");

    let resolver = ScopeResolver::with_work(auratranslate_lib::core::scope::WorkScope {
        work_id: "w1".to_owned(),
    });
    let outcome = confirmed_terms_for_injection(&resolver, &global, Some(&work), "A dragon roared.", MatchLang::En)
        .expect("cua khong loi");

    assert_eq!(outcome.injected.len(), 1);
    assert_eq!(outcome.injected[0].translation, "rong tac pham");
    assert_eq!(outcome.injected[0].tier, GlossaryTier::Work);

    drop(global);
    drop(work);
    cleanup(&global_dir);
    cleanup(&work_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 3 — Pending entry
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_pending_entry_is_not_injected_and_not_listed() {
    let dir = temp_dir("pending");
    let global = open_global(&dir);
    add_manual_term(&global, None, GlossaryTier::Global, "dragon", None, "", Category::Other)
        .expect("them muc cho chot");

    let resolver = ScopeResolver::global_only();
    let outcome = confirmed_terms_for_injection(&resolver, &global, None, "A dragon roared.", MatchLang::En)
        .expect("cua khong loi");

    assert!(outcome.injected.is_empty());
    assert!(outcome.suppressed_by_pending_overlap.is_empty());

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 4 — Pending Work shadows confirmed Global
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn pending_work_shadowing_confirmed_global_injects_neither() {
    let global_dir = temp_dir("shadow-global");
    let work_dir = temp_dir("shadow-work");
    let global = open_global(&global_dir);
    let work = open_project(&work_dir);

    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them global da chot");
    add_manual_term(&global, Some(&work), GlossaryTier::Work, "dragon", None, "", Category::Other)
        .expect("them work cho chot");

    let resolver = ScopeResolver::with_work(auratranslate_lib::core::scope::WorkScope {
        work_id: "w1".to_owned(),
    });
    let outcome = confirmed_terms_for_injection(&resolver, &global, Some(&work), "A dragon roared.", MatchLang::En)
        .expect("cua khong loi");

    assert!(outcome.injected.is_empty(), "muc Work cho chot che muc Global da chot -- khong ben nao duoc chen");
    assert!(
        outcome.suppressed_by_pending_overlap.is_empty(),
        "day la muc CHE THEO TANG (AD-18), khong phai muc CHE THEO CHONG SPAN (Decision 4) -- \
         khong co gi de bao 'suppressed_by_pending_overlap'"
    );

    drop(global);
    drop(work);
    cleanup(&global_dir);
    cleanup(&work_dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 5 — Overlapping confirmed terms
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn overlapping_confirmed_terms_the_longer_span_wins() {
    let dir = temp_dir("overlap-confirmed");
    let global = open_global(&dir);
    add_manual_term(&global, None, GlossaryTier::Global, "fire dragon", Some("hoa long"), "", Category::Other)
        .expect("them cum dai");
    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them cum ngan");

    let resolver = ScopeResolver::global_only();
    let outcome = confirmed_terms_for_injection(&resolver, &global, None, "A fire dragon roars.", MatchLang::En)
        .expect("cua khong loi");

    assert_eq!(outcome.injected.len(), 1, "chi mot cap duoc chen: {:?}", outcome.injected);
    assert_eq!(outcome.injected[0].source_term, "fire dragon");
    assert_eq!(outcome.injected[0].translation, "hoa long");

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Cửa — hai cặp ĐÃ CHỐT khác nhau trong MỘT câu, đúng thứ tự xuất hiện
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn confirmed_terms_for_injection_returns_two_pairs_in_sentence_order() {
    let dir = temp_dir("door-two-pairs");
    let global = open_global(&dir);
    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them dragon");
    add_manual_term(&global, None, GlossaryTier::Global, "castle", Some("lau dai"), "", Category::Other)
        .expect("them castle");

    let resolver = ScopeResolver::global_only();
    let sentence = "A dragon guards the castle.";
    // "dragon" o diem ma 2..8; "castle" o diem ma 20..26.
    assert_eq!(&sentence[2..8], "dragon", "tien de vi tri");
    assert_eq!(&sentence[20..26], "castle", "tien de vi tri");

    let outcome = confirmed_terms_for_injection(&resolver, &global, None, sentence, MatchLang::En)
        .expect("cua khong loi");

    assert_eq!(
        outcome.injected,
        vec![
            GlossaryInjectionTerm {
                start: 2,
                end: 8,
                tier: GlossaryTier::Global,
                source_term: "dragon".to_owned(),
                translation: "rong".to_owned(),
            },
            GlossaryInjectionTerm {
                start: 20,
                end: 26,
                tier: GlossaryTier::Global,
                source_term: "castle".to_owned(),
                translation: "lau dai".to_owned(),
            },
        ],
        "hai cap phai ra DUNG theo thu tu xuat hien trong cau"
    );

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 6 — Pending term overlaps a confirmed term (Decision 4) — cùng AC "lưới và ledger
// không được lệch nhau": so LEDGER (`gather_glossary_context`) với `marks_for_source_text`
// (lưới) trên ĐÚNG một câu.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_pending_term_overlapping_a_confirmed_term_suppresses_both_and_the_ledger_names_it() {
    let dir = temp_dir("pending-overlap");
    let global = open_global(&dir);
    add_manual_term(&global, None, GlossaryTier::Global, "dog", Some("cho"), "", Category::Other)
        .expect("them dog da chot");
    add_manual_term(&global, None, GlossaryTier::Global, "dog walker", None, "", Category::Other)
        .expect("them dog walker cho chot");

    let resolver = ScopeResolver::global_only();
    let sentence = "The dog walker arrived.";
    // "The dog walker arrived." -- "dog" chiem diem ma 4..7.
    assert_eq!(&sentence[4..7], "dog", "tien de vi tri");

    let status = gather_glossary_context(GLOSSARY_BODY, &resolver, &global, None, "en", sentence)
        .expect("gather khong loi");

    match &status {
        GlossaryInjectionStatus::Asked { injected, suppressed_by_pending_overlap } => {
            assert!(injected.is_empty(), "khong ben nao duoc chen: {injected:?}");
            assert_eq!(
                suppressed_by_pending_overlap,
                &vec![SuppressedGlossaryTerm {
                    source_term: "dog".to_owned(),
                    translation: "cho".to_owned(),
                    start: 4,
                    end: 7,
                    tier: GlossaryTier::Global,
                }]
            );
        }
        other => panic!("mong Asked, nhan {other:?}"),
    }

    // Đối chiếu SPAN của LEDGER (không phải của cửa) với LƯỚI (`marks_for_source_text`) trên
    // đúng câu này: grid phải đánh dấu ĐÚNG "dog walker" (chưa chốt) ở ĐÚNG span mà "dog" bị
    // che, KHÔNG đánh dấu "dog" — cùng kết luận mà ledger vừa nêu, đọc từ phía LƯỚI.
    let layers = auratranslate_lib::core::dict::DictLayers::empty();
    let disabled = std::collections::BTreeSet::new();
    let marks =
        marks_for_source_text(&resolver, &global, None, sentence, MatchLang::En, &layers, &disabled)
            .expect("marks khong loi");
    assert_eq!(marks.len(), 1, "grid: {marks:?}");
    assert_eq!(marks[0].source_term, "dog walker");
    assert!(!marks[0].is_confirmed);
    assert_eq!(
        marks[0].start, 4,
        "khoang cua nguoi THANG ('dog walker') phai bat dau dung tai diem 'dog' bi che"
    );

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 7 — No term matches
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn no_term_matches_removes_the_marker_line_and_ledger_records_asked_empty() {
    let dir = temp_dir("no-match");
    let global = open_global(&dir);
    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them thuat ngu khong khop cau nay");

    let resolver = ScopeResolver::global_only();
    let body = "Terms: {{glossary_terms}}\n{{source_segment}}";
    let status = gather_glossary_context(body, &resolver, &global, None, "en", "Nothing here.")
        .expect("gather khong loi");

    let (prompt, ledger) = assemble_prompt(body, "Nothing here.", status, None);

    assert_eq!(prompt, "Terms: \nNothing here.", "marker mat nhung dong con lai vi Terms: khong rong");
    match &ledger.glossary {
        GlossaryInjectionStatus::Asked { injected, .. } => assert!(injected.is_empty()),
        other => panic!("mong Asked (rong), nhan {other:?}"),
    }

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 8 — Marker absent: KHÔNG lượt gọi Glossary nào chạy
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn marker_absent_runs_no_glossary_query_at_all() {
    let dir = temp_dir("marker-absent");
    let global = open_global(&dir);
    // Đóng NGAY, TRƯỚC khi gọi `gather_glossary_context` — một lượt Glossary LỠ chạy sẽ đọc
    // từ kho đã đóng (`StoreError::PoolClosed`, xem `glossary_marks_contract.rs:534-562`) và
    // trả `Err`, khiến ca này đỏ QUAN SÁT ĐƯỢC (Err thay vì Ok(NotAsked)) thay vì im lặng
    // "thành công" như một `Store::open` lại (tạo file MỚI hợp lệ, không mở lại kho đã đóng)
    // từng làm.
    global.close();

    let resolver = ScopeResolver::global_only();
    let body = "{{source_segment}}"; // KHONG mang {{glossary_terms}}

    let status = gather_glossary_context(body, &resolver, &global, None, "en", "Hello.")
        .expect("gather khong duoc loi -- khong query Glossary nao duoc phep chay");
    assert_eq!(status, GlossaryInjectionStatus::NotAsked);

    let (prompt, ledger) = assemble_prompt(body, "Hello.", status, None);
    assert_eq!(prompt, "Hello.");
    assert_eq!(ledger.glossary, GlossaryInjectionStatus::NotAsked);

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 9 — Unknown marker
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_unknown_marker_is_left_verbatim_and_reported() {
    let body = "{{chapter_context}} {{source_segment}}";
    let (prompt, ledger) = assemble_prompt(body, "Hello.", GlossaryInjectionStatus::NotAsked, None);

    assert_eq!(prompt, "{{chapter_context}} Hello.");
    assert_eq!(ledger.unknown_markers, vec!["{{chapter_context}}".to_owned()]);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 10/11 — TM not built / TM searched
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn tm_not_built_removes_the_marker_line_and_ledger_records_not_built_yet() {
    let body = "TM:\n{{tm_similar_segments}}\n{{source_segment}}";
    let (prompt, ledger) = assemble_prompt(body, "Hello.", GlossaryInjectionStatus::NotAsked, None);

    assert_eq!(prompt, "TM:\nHello.");
    assert_eq!(ledger.tm, TmInjectionStatus::NotBuiltYet);
}

#[test]
fn tm_searched_records_the_real_segments_from_a_real_some_slice() {
    let segments = vec![SimilarSegment {
        source_text: "A dog barked.".to_owned(),
        target_text: "Mot con cho sua.".to_owned(),
    }];
    let body = "TM:\n{{tm_similar_segments}}\n{{source_segment}}";
    let (prompt, ledger) =
        assemble_prompt(body, "Hello.", GlossaryInjectionStatus::NotAsked, Some(&segments));

    assert_eq!(prompt, "TM:\nHello.", "marker van bi go -- tiem noi dung TM that la viec Epic 7");
    assert_eq!(ledger.tm, TmInjectionStatus::Searched(segments));
}

#[test]
fn tm_searched_with_an_empty_slice_is_still_searched_not_not_built_yet() {
    let segments: Vec<SimilarSegment> = Vec::new();
    let (_prompt, ledger) = assemble_prompt(
        "{{tm_similar_segments}}",
        "Hello.",
        GlossaryInjectionStatus::NotAsked,
        Some(&segments),
    );
    assert_eq!(ledger.tm, TmInjectionStatus::Searched(Vec::new()));
    assert_ne!(ledger.tm, TmInjectionStatus::NotBuiltYet);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 12 — Marker shares its line, nothing to inject
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_marker_sharing_its_line_with_nothing_to_inject_removes_only_the_marker_text() {
    let body = "Terms: {{glossary_terms}} end.";
    let (prompt, _ledger) = assemble_prompt(body, "Hello.", GlossaryInjectionStatus::NotAsked, None);
    assert_eq!(prompt, "Terms:  end.");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 13 — Marker shares its line, two or more pairs (review finding #5's counter-check:
// so KHỚP TOÀN VĂN, không `.contains` một cặp)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_marker_sharing_its_line_with_two_or_more_pairs_never_splices_mid_line() {
    let status = GlossaryInjectionStatus::Asked {
        injected: vec![
            InjectedGlossaryTerm {
                source_term: "a".to_owned(),
                translation: "b".to_owned(),
                start: 0,
                end: 1,
                tier: GlossaryTier::Global,
            },
            InjectedGlossaryTerm {
                source_term: "c".to_owned(),
                translation: "d".to_owned(),
                start: 2,
                end: 3,
                tier: GlossaryTier::Global,
            },
        ],
        suppressed_by_pending_overlap: Vec::new(),
    };
    let body = "Terms: {{glossary_terms}} end.";
    let (prompt, _ledger) = assemble_prompt(body, "Hello.", status, None);

    assert_eq!(
        prompt, "Terms: \na → b\nc → d\n end.",
        "khoi cap phai bat dau dong RIENG, van ban con lai phai xuong dong RIENG -- khong noi \
         giua dong"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 14 — Repeated marker
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_repeated_marker_expands_every_occurrence() {
    let body = "{{source_segment}} -- {{source_segment}}";
    let (prompt, _ledger) =
        assemble_prompt(body, "Hi.", GlossaryInjectionStatus::NotAsked, None);
    assert_eq!(prompt, "Hi. -- Hi.");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 15 — Body omits {{source_segment}}
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_body_omitting_source_segment_is_assembled_and_the_ledger_flags_it() {
    let body = "Chi co glossary: {{glossary_terms}}";
    let (prompt, ledger) = assemble_prompt(body, "Hello.", GlossaryInjectionStatus::NotAsked, None);

    assert_eq!(prompt, "Chi co glossary: ");
    assert!(ledger.source_segment_missing);
}

#[test]
fn a_body_carrying_source_segment_is_not_flagged_missing() {
    let (_prompt, ledger) =
        assemble_prompt("{{source_segment}}", "Hello.", GlossaryInjectionStatus::NotAsked, None);
    assert!(!ledger.source_segment_missing);
}

/// 🔴 rà soát 2026-09-18 — `source_segment_missing` phải đến từ ĐÚNG lượt quét-và-thay
/// (token hoá thật), không phải một `body.contains("{{source_segment}}")` thô. Chuỗi con
/// `"{{source_segment}}"` NẰM LỌT bên trong `"{{{source_segment}}"` (ba dấu `{` mở) — một
/// `.contains` sẽ báo "có mặt", trong khi lượt token hoá THẬT đọc token `"{source_segment"`
/// (KHÔNG khớp biến số nào) và để nguyên văn — không câu nào thật sự được tiêm.
#[test]
fn a_triple_open_brace_before_source_segment_is_not_mistaken_for_the_real_marker() {
    let body = "{{{source_segment}}";
    let (prompt, ledger) = assemble_prompt(body, "Hi.", GlossaryInjectionStatus::NotAsked, None);
    assert_eq!(prompt, body, "khong token nao duoc cong nhan -- than giu NGUYEN VAN");
    assert!(
        ledger.source_segment_missing,
        "mot hinh dang GIONG marker (khop chuoi con nhung khong khop token that) khong duoc \
         tinh la co mat"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 16 — Injected text contains a marker: KHÔNG được quét lại
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn injected_text_containing_a_literal_marker_is_never_rescanned() {
    let status = GlossaryInjectionStatus::Asked {
        injected: vec![InjectedGlossaryTerm {
            source_term: "widget".to_owned(),
            translation: "{{source_segment}}".to_owned(),
            start: 0,
            end: 6,
            tier: GlossaryTier::Global,
        }],
        suppressed_by_pending_overlap: Vec::new(),
    };
    let body = "{{glossary_terms}}\n---\n{{source_segment}}";
    let (prompt, ledger) = assemble_prompt(body, "A widget here.", status, None);

    assert_eq!(
        prompt, "widget → {{source_segment}}\n---\nA widget here.",
        "ban dich chua literal {{{{source_segment}}}} phai di vao prompt NGUYEN VAN, khong bi \
         coi la mot marker thu hai"
    );
    assert!(
        ledger.unknown_markers.is_empty(),
        "scan_markers doc THAN GOC, khong doc chuoi da tiem -- {{{{source_segment}}}} ben \
         trong ban dich khong duoc bao la unknown marker: {:?}",
        ledger.unknown_markers
    );
}

/// Đối chứng dương cho hình dạng "{{" mồ côi ĐỨNG TRƯỚC một marker THẬT (khác Hàng 16, nơi
/// nội dung TIÊM mang marker giả) — bản thân THÂN GỐC mang một `{{` mồ côi rồi mới tới
/// `{{source_segment}}` thật. Chưa ca nào của tệp này đi qua guard này với một marker THẬT
/// theo sau; vô hiệu guard đó (bỏ nhánh `token.contains("{{")`) sẽ không bị bắt nếu thiếu ca
/// này.
#[test]
fn an_orphan_opening_brace_before_a_real_source_segment_marker_does_not_swallow_it() {
    let body = "note {{ stray: {{source_segment}} end";
    let (prompt, ledger) = assemble_prompt(body, "Hi.", GlossaryInjectionStatus::NotAsked, None);
    assert_eq!(prompt, "note {{ stray: Hi. end");
    assert!(!ledger.source_segment_missing);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Decision 2 — khối `{{glossary_terms}}` gom theo `source_term`; ledger giữ nguyên TỪNG lần
// khớp
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn render_glossary_pairs_dedupes_by_source_term_but_the_ledger_keeps_every_occurrence() {
    let status = GlossaryInjectionStatus::Asked {
        injected: vec![
            InjectedGlossaryTerm {
                source_term: "dog".to_owned(),
                translation: "cho".to_owned(),
                start: 4,
                end: 7,
                tier: GlossaryTier::Global,
            },
            InjectedGlossaryTerm {
                source_term: "dog".to_owned(),
                translation: "cho".to_owned(),
                start: 16,
                end: 19,
                tier: GlossaryTier::Global,
            },
        ],
        suppressed_by_pending_overlap: Vec::new(),
    };
    let body = "{{glossary_terms}}\n{{source_segment}}";
    let (prompt, ledger) = assemble_prompt(body, "The dog and the dog.", status, None);

    assert_eq!(
        prompt, "dog → cho\nThe dog and the dog.",
        "khoi van ban gui nha cung cap chi liet DUNG MOT lan cho moi source_term, du no khop \
         hai lan trong cau"
    );
    match &ledger.glossary {
        GlossaryInjectionStatus::Asked { injected, .. } => assert_eq!(
            injected.len(),
            2,
            "ledger phai giu CA HAI lan khop -- chi khoi VAN BAN gui nha cung cap moi gom"
        ),
        other => panic!("mong Asked, nhan {other:?}"),
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 17 — Work tier closed
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn work_tier_closed_matches_the_global_tier_only_with_no_error() {
    let dir = temp_dir("work-closed");
    let global = open_global(&dir);
    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them global");

    let resolver = ScopeResolver::global_only();
    let outcome = confirmed_terms_for_injection(&resolver, &global, None, "A dragon roared.", MatchLang::En)
        .expect("khong tang Work van phai khong loi");

    assert_eq!(outcome.injected.len(), 1);
    assert_eq!(outcome.injected[0].tier, GlossaryTier::Global);

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 18 — Store unreadable: lỗi truyền từ tầng gom, assembler không thấy nó
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 rà soát 2026-09-18 — ca trước chỉ dựng một `Store` HỎNG rồi dừng ở `Store::open(..)
/// .is_err()`, KHÔNG hề gọi `gather_glossary_context` — một `?` đổi thành
/// `.unwrap_or_default()` bên trong hàm đó vẫn để ca này xanh. Dựng lại đúng khuôn
/// `glossary_marks_contract.rs:534-562`: mở kho THẬT rồi đóng GIỮA CHỪNG
/// (`Store::close()`, an toàn gọi trước `drop`), gọi THẬT `gather_glossary_context` trên một
/// thân MANG `{{glossary_terms}}` (bắt buộc — nếu không, lượt gọi Glossary còn không chạy),
/// và khẳng định `Err(GlossaryError::Store(_))` thật sự quay về.
#[test]
fn an_unreadable_store_propagates_a_glossary_error_from_the_gathering_layer() {
    let dir = temp_dir("unreadable");
    let global = open_global(&dir);
    global.close();

    let resolver = ScopeResolver::global_only();
    let err = gather_glossary_context(
        GLOSSARY_BODY, // MANG {{glossary_terms}} -- bat buoc de tang gom THAT SU goi Glossary
        &resolver,
        &global,
        None,
        "en",
        "A dragon roared.",
    )
    .expect_err("kho dong giua chung PHAI la mot loi, khong Ok");

    match err {
        GlossaryError::Store(_) => {}
        other => panic!("ky vong GlossaryError::Store, nhan {other:?}"),
    }

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Decision 1 — {{chapter_context}} vẫn NGOÀI vocabulary đã ratify
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn chapter_context_is_still_outside_the_ratified_vocabulary() {
    let body = "{{chapter_context}}";
    let (prompt, ledger) = assemble_prompt(body, "Hello.", GlossaryInjectionStatus::NotAsked, None);
    assert_eq!(prompt, "{{chapter_context}}");
    assert_eq!(ledger.unknown_markers, vec!["{{chapter_context}}".to_owned()]);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Decision 3, vế thứ hai — collapse LOCAL, không đụng blank lines ở chỗ khác
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn removing_a_marker_alone_on_its_line_collapses_two_blank_lines_it_creates_into_one() {
    let body = "Before.\n\n{{tm_similar_segments}}\n\nAfter.";
    let (prompt, _ledger) =
        assemble_prompt(body, "Hello.", GlossaryInjectionStatus::NotAsked, None);
    assert_eq!(
        prompt, "Before.\n\nAfter.",
        "go dong marker dua hai dong trang lai gan nhau -- phai con DUNG MOT dong trang"
    );
}

/// 🔴 rà soát 2026-09-18 — HAI marker-chỉ-có-marker LIÊN TIẾP (mỗi marker rỗng, đứng một
/// mình trên dòng của nó), giữa hai dòng trắng thật. Bản trước đọc "dòng trước" từ `body`
/// GỐC nên hai lượt gỡ không phối hợp được với nhau: lượt gỡ dòng thứ nhất không thấy "dòng
/// sau" (đó là dòng marker thứ hai, chưa gỡ) là trắng; lượt gỡ dòng thứ hai đọc "dòng trước"
/// từ `body` gốc — vẫn là văn bản `{{...}}` của marker thứ nhất, không phải dòng trắng thật
/// đứng trước nó SAU KHI marker thứ nhất đã biến mất. Kết quả sai: hai dòng trắng còn nguyên,
/// không co lại thành một.
#[test]
fn two_adjacent_marker_only_lines_between_two_blank_lines_collapse_to_one_blank_line() {
    let body = "A\n\n{{glossary_terms}}\n{{tm_similar_segments}}\n\nB";
    let (prompt, _ledger) =
        assemble_prompt(body, "Hello.", GlossaryInjectionStatus::NotAsked, None);
    assert_eq!(
        prompt, "A\n\nB",
        "hai dong CHI CO marker (ca hai deu rong o day) nam giua hai dong trang phai co lai \
         thanh DUNG MOT dong trang, khong phai hai"
    );
}

#[test]
fn blank_lines_untouched_by_any_removal_survive_verbatim() {
    // Đúng phản ví dụ đo được ở loop 0: KHÔNG marker nào bị gỡ ở đây ({{source_segment}} luôn
    // có nội dung) -- ba dấu `\n` liên tiếp là văn bản của chính người dịch, không được đụng.
    let body = "Doan mot.\n\n\nDoan hai.\n{{source_segment}}";
    let (prompt, _ledger) =
        assemble_prompt(body, "Cau dich.", GlossaryInjectionStatus::NotAsked, None);
    assert_eq!(prompt, "Doan mot.\n\n\nDoan hai.\nCau dich.");
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC — gọi hai lần cho cùng đầu vào ⇒ hai prompt byte-for-byte, ledger cùng thứ tự
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn calling_assemble_prompt_twice_on_the_same_inputs_is_byte_identical() {
    let status = GlossaryInjectionStatus::Asked {
        injected: vec![
            InjectedGlossaryTerm {
                source_term: "a".to_owned(),
                translation: "b".to_owned(),
                start: 0,
                end: 1,
                tier: GlossaryTier::Global,
            },
            InjectedGlossaryTerm {
                source_term: "c".to_owned(),
                translation: "d".to_owned(),
                start: 2,
                end: 3,
                tier: GlossaryTier::Global,
            },
        ],
        suppressed_by_pending_overlap: Vec::new(),
    };
    let (prompt1, ledger1) = assemble_prompt(GLOSSARY_BODY, "Hello.", status.clone(), None);
    let (prompt2, ledger2) = assemble_prompt(GLOSSARY_BODY, "Hello.", status, None);

    assert_eq!(prompt1, prompt2);
    assert_eq!(ledger1, ledger2);
}

// ═════════════════════════════════════════════════════════════════════════════════
// perf_probe — đường TRỌN gói mỗi câu (gather thật, sau `warm()`, gieo thuật ngữ THẬT khớp)
// ═════════════════════════════════════════════════════════════════════════════════
//
// `deferred-work.md` (Chủ: Epic 4, "full scan + triple clone per call — measure first") đòi
// đo trước khi coi đây là vấn đề. Loop 0 đo nhầm nửa KHÔNG khớp gì (thuật ngữ gieo không
// xuất hiện trong câu) — probe này khớp THẬT, trên câu thật chứa thuật ngữ đã gieo.
//
// ⚠️ Quần thể đo: 500 mục **tầng Global**, `work: None` — CHỈ MỘT `load_tier` chạy (không
// hai). Xem `deferred-work.md` §Deferred from: 4-6-… cho lý do con số này không đại diện cho
// chi phí "cả hai tầng".
#[test]
fn perf_probe_gather_glossary_context_whole_per_sentence_path_on_a_matching_sentence() {
    matching::warm(); // khong tinh chi phi khoi tao Jieba lanh vao con so nay (En khong dung
    // Jieba, nhung gieo cung cho khop truoc voi con duong that su chay o Story 4.8)

    let dir = temp_dir("perf-probe");
    let global = open_global(&dir);

    const ROW_COUNT: usize = 500;
    for i in 0..ROW_COUNT {
        add_manual_term(
            &global,
            None,
            GlossaryTier::Global,
            &format!("term-g-{i}"),
            Some(&format!("dich-{i}")),
            "",
            Category::Other,
        )
        .expect("them thuat ngu gieo");
    }
    // Thuật ngữ THẬT SỰ nằm trong câu sẽ dịch -- khác loop 0 (đo nửa không khớp gì).
    add_manual_term(&global, None, GlossaryTier::Global, "dragon", Some("rong"), "", Category::Other)
        .expect("them thuat ngu khop that");

    let resolver = ScopeResolver::global_only();
    const CALLS: usize = 200;
    let mut injected_count = 0usize;

    let t0 = Instant::now();
    for _ in 0..CALLS {
        let status = gather_glossary_context(
            GLOSSARY_BODY,
            &resolver,
            &global,
            None,
            "en",
            "A dragon roared in the distance.",
        )
        .expect("gather khong loi");
        if let GlossaryInjectionStatus::Asked { injected, .. } = status {
            injected_count += injected.len();
        }
    }
    let elapsed = t0.elapsed();

    assert_eq!(
        injected_count,
        CALLS,
        "moi lot goi phai khop dung MOT thuat ngu that (dragon) -- con so nay chung minh \
         probe do tren mot cau THAT SU khop, khong phai mot cau trong"
    );

    let per_call = elapsed / CALLS as u32;
    eprintln!(
        "[perf_probe_gather_glossary_context] rows={ROW_COUNT} (Global-only, work=None) \
         calls={CALLS} total={elapsed:?} per_call={per_call:?} build={} injected_per_call=1",
        if cfg!(debug_assertions) { "debug" } else { "release" }
    );

    drop(global);
    cleanup(&dir);
}
