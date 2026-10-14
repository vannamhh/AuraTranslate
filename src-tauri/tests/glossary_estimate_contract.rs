//! FR137: the pre-import Glossary estimate. Pure layer only; the estimate must agree with the
//! real scan (`scan_candidates_controlled`) chapter by chapter, and an unknown number is never 0.

use std::collections::HashSet;

use auratranslate_lib::core::glossary::scan::{ScanOutcome, scan_candidates_controlled};
use auratranslate_lib::core::glossary::surnames::COMMON_SURNAMES;
use auratranslate_lib::core::glossary::{
    DictionaryProbe, EstimateOutcome, estimate_import_candidates,
};
use auratranslate_lib::core::matching::MatchLang;

fn chapter(sentence_for: impl Fn(usize) -> String, count: usize) -> Vec<String> {
    (0..count).map(sentence_for).collect()
}

fn fixture() -> Vec<Vec<String>> {
    vec![
        chapter(|i| format!("萧炎在第{i}章遇到了药老。"), 8),
        chapter(|i| format!("萧炎和林动在第{i}城相见。"), 8),
        chapter(|_| "普通的一天过去了。".to_owned(), 8),
    ]
}

fn real_scan_union(
    chapters: &[Vec<String>],
    threshold: u32,
    known: &HashSet<&str>,
    excluded: &HashSet<String>,
) -> usize {
    let mut union: HashSet<String> = HashSet::new();
    for segments in chapters {
        let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
        let mut probe = |term: &str| {
            if known.contains(term) {
                DictionaryProbe::Known
            } else {
                DictionaryProbe::Missing
            }
        };
        let mut never = || false;
        let ScanOutcome::Completed(found) = scan_candidates_controlled(
            &refs,
            MatchLang::Zh,
            threshold,
            COMMON_SURNAMES,
            &mut probe,
            &mut never,
        ) else {
            panic!("the real scan must complete on the fixture");
        };
        union.extend(
            found
                .into_iter()
                .map(|c| c.source_term)
                .filter(|t| !excluded.contains(t)),
        );
    }
    union.len()
}

#[test]
fn the_exact_estimate_equals_the_union_of_the_real_per_chapter_scans_minus_the_glossary() {
    let chapters = fixture();
    let known: HashSet<&str> = ["药老"].into_iter().collect();
    let excluded: HashSet<String> = ["林动".to_owned()].into_iter().collect();

    let mut probe = |term: &str| {
        if known.contains(term) {
            DictionaryProbe::Known
        } else {
            DictionaryProbe::Missing
        }
    };
    let mut never = || false;
    let estimate = estimate_import_candidates(
        &chapters,
        MatchLang::Zh,
        5,
        COMMON_SURNAMES,
        &excluded,
        &mut probe,
        &mut never,
    );

    let expected = real_scan_union(&chapters, 5, &known, &excluded);
    assert!(expected > 0, "the fixture must queue something");
    assert_eq!(estimate, EstimateOutcome::Estimated(expected));
}

#[test]
fn the_threshold_applies_per_chapter_not_to_the_total() {
    let chapters = vec![
        chapter(|_| "唯一的名字叫做罗峰。".to_owned(), 3),
        chapter(|_| "唯一的名字叫做罗峰。".to_owned(), 3),
    ];
    let mut never = || false;
    let mut missing = |_: &str| DictionaryProbe::Missing;
    let estimate = estimate_import_candidates(
        &chapters,
        MatchLang::Zh,
        5,
        COMMON_SURNAMES,
        &HashSet::new(),
        &mut missing,
        &mut never,
    );
    assert_eq!(
        estimate,
        EstimateOutcome::Estimated(0),
        "3 + 3 occurrences never reach 5 inside one chapter, as in the real scan"
    );
}

#[test]
fn an_inconclusive_probe_makes_the_estimate_inconclusive_not_zero() {
    let chapters = fixture();
    let mut never = || false;
    let mut inconclusive = |_: &str| DictionaryProbe::Inconclusive;
    let estimate = estimate_import_candidates(
        &chapters,
        MatchLang::Zh,
        5,
        COMMON_SURNAMES,
        &HashSet::new(),
        &mut inconclusive,
        &mut never,
    );
    assert_eq!(estimate, EstimateOutcome::Inconclusive);
}

#[test]
fn a_cancelled_estimate_reports_cancelled() {
    let chapters = fixture();
    let mut cancelled = || true;
    let mut missing = |_: &str| DictionaryProbe::Missing;
    let estimate = estimate_import_candidates(
        &chapters,
        MatchLang::Zh,
        5,
        COMMON_SURNAMES,
        &HashSet::new(),
        &mut missing,
        &mut cancelled,
    );
    assert_eq!(estimate, EstimateOutcome::Cancelled);
}
