//! AD-57: the "only mine" policy is read by the surfaces that use TM, and only the two lookup and
//! management functions may opt out of it with `TmSidePolicy::browse()`.

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;

const BROWSE_CALL: &str = "TmSidePolicy::browse(";
const POLICY_READ_CALL: &str = "TmSidePolicy::from_work(";

const BROWSE_ALLOWED: [(&str, &str); 2] = [
    ("commands/segment/tm_match.rs", "prepare_tm_concordance"),
    ("commands/tm.rs", "prepare_tm_list"),
];

const SRC_RS_FLOOR: usize = 120;
const POLICY_READ_FLOOR: usize = 5;

fn fn_name(code: &str) -> Option<&str> {
    let rest = code.split("fn ").nth(1)?;
    if !(code.starts_with("fn ") || code.contains(" fn ")) {
        return None;
    }
    let end = rest.find(['(', '<', ' '])?;
    Some(&rest[..end])
}

/// `(line, enclosing fn)` of every call of `needle` in the non-test code of `text`.
fn calls_of(text: &str, needle: &str) -> Vec<(usize, String)> {
    let product = boundary_scan::without_test_modules(text);
    let mut current = String::new();
    let mut found = Vec::new();
    for (line, code) in boundary_scan::code_lines(&product) {
        if let Some(name) = fn_name(&code) {
            current = name.to_owned();
        }
        if code.contains(needle) {
            found.push((line, current.clone()));
        }
    }
    found
}

fn browse_violations(sources: &[(String, String)]) -> Vec<String> {
    let mut violations = Vec::new();
    for (rel, text) in sources {
        for (line, name) in calls_of(text, BROWSE_CALL) {
            if !BROWSE_ALLOWED.contains(&(rel.as_str(), name.as_str())) {
                violations.push(format!("{rel}:{line} in `{name}`"));
            }
        }
    }
    violations
}

#[test]
fn browse_is_called_only_by_concordance_and_the_management_list() {
    let sources = boundary_scan::rust_sources(&boundary_scan::src_root());
    assert_eq!(browse_violations(&sources), Vec::<String>::new());
}

#[test]
fn both_allowed_functions_really_call_browse() {
    let sources = boundary_scan::rust_sources(&boundary_scan::src_root());
    for (file, name) in BROWSE_ALLOWED {
        let (_, text) = sources
            .iter()
            .find(|(rel, _)| rel == file)
            .unwrap_or_else(|| panic!("{file} is missing"));
        assert!(
            calls_of(text, BROWSE_CALL).iter().any(|(_, n)| n == name),
            "{file}::{name} no longer calls browse(); the allow-list names a call that is gone"
        );
    }
}

#[test]
fn a_browse_call_added_to_another_function_is_caught() {
    let planted = vec![(
        "commands/segment/chapter_read.rs".to_owned(),
        "fn fill_exact_tm_matches() {\n    let p = crate::core::tm::TmSidePolicy::browse();\n}\n"
            .to_owned(),
    )];
    assert_eq!(
        browse_violations(&planted),
        vec!["commands/segment/chapter_read.rs:2 in `fill_exact_tm_matches`".to_owned()]
    );
}

#[test]
fn a_commented_browse_call_is_not_a_call() {
    let planted = vec![(
        "commands/segment/chapter_read.rs".to_owned(),
        "fn fill() {\n    // TmSidePolicy::browse()\n}\n".to_owned(),
    )];
    assert!(browse_violations(&planted).is_empty());
}

#[test]
fn surfaces_read_the_work_switch() {
    let sources = boundary_scan::rust_sources(&boundary_scan::src_root());
    boundary_scan::assert_population_floor(
        SRC_RS_FLOOR,
        sources.len(),
        "SRC_RS_FLOOR",
        "`.rs` files under `src-tauri/src/**`",
    );
    let reads: usize = sources
        .iter()
        .map(|(_, text)| calls_of(text, POLICY_READ_CALL).len())
        .sum();
    boundary_scan::assert_population_floor(
        POLICY_READ_FLOOR,
        reads,
        "POLICY_READ_FLOOR",
        "calls of `TmSidePolicy::from_work`",
    );
    for (file, name) in [
        ("commands/segment/chapter_read.rs", "fill_exact_tm_matches"),
        ("commands/segment/tm_match.rs", "prepare_tm_fuzzy"),
        ("commands/glossary.rs", "glossary_tm_suggestions"),
        ("core/ai/rag.rs", "load_tm_rows"),
    ] {
        let (_, text) = sources
            .iter()
            .find(|(rel, _)| rel == file)
            .unwrap_or_else(|| panic!("{file} is missing"));
        assert!(
            calls_of(text, POLICY_READ_CALL)
                .iter()
                .any(|(_, n)| n == name),
            "{file}::{name} must read the Work's switch"
        );
    }
}
