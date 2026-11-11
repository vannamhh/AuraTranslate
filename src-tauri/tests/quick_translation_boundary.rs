//! Source-tree boundary of AD-64 rule 3 and AD-13: only `core/quick_translation/mod.rs` opens
//! `quick-translation.db` (the declaration lives in `core/store/mod.rs`), and the module does
//! not depend on `core::ai`.

use std::fs;
use std::path::PathBuf;

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;
use boundary_scan::{rel_posix, src_root};

const EXEMPT_FILES: [&str; 2] = ["core/quick_translation/mod.rs", "core/store/mod.rs"];
const FORBIDDEN: [&str; 2] = [
    "StoreSpec::quick_translation",
    "StoreKind::QuickTranslation",
];
const RS_FLOOR: usize = 133;

fn all_rust_sources() -> (PathBuf, Vec<PathBuf>) {
    let root = src_root();
    let files = boundary_scan::rust_sources(&root)
        .into_iter()
        .map(|(rel, _)| root.join(rel))
        .collect();
    (root, files)
}

fn line_names_the_quick_translation_store(code: &str) -> Option<&'static str> {
    FORBIDDEN.into_iter().find(|needle| code.contains(needle))
}

fn code_lines_naming_the_store(text: &str) -> Vec<(usize, &'static str)> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim_start().starts_with("//"))
        .filter_map(|(index, line)| {
            line_names_the_quick_translation_store(line).map(|needle| (index + 1, needle))
        })
        .collect()
}

fn files_naming_the_store(root: &std::path::Path, files: &[PathBuf]) -> Vec<String> {
    let mut out: Vec<String> = files
        .iter()
        .filter(|f| {
            !code_lines_naming_the_store(&fs::read_to_string(f).unwrap_or_default()).is_empty()
        })
        .map(|f| rel_posix(root, f))
        .collect();
    out.sort();
    out
}

#[test]
fn the_scanned_tree_is_large_enough_to_be_real() {
    let (_, files) = all_rust_sources();
    boundary_scan::assert_population_floor(
        RS_FLOOR,
        files.len(),
        "RS_FLOOR",
        "`.rs` files under `src-tauri/src/**`",
    );
}

#[test]
fn only_the_two_named_files_may_name_the_quick_translation_store() {
    let (root, files) = all_rust_sources();
    let mut violations = Vec::new();
    for file in &files {
        let rel = rel_posix(&root, file);
        if EXEMPT_FILES.contains(&rel.as_str()) {
            continue;
        }
        let text = fs::read_to_string(file).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        for (line, needle) in code_lines_naming_the_store(&text) {
            violations.push(format!("{rel}:{line}  {needle}"));
        }
    }
    assert!(
        violations.is_empty(),
        "quick-translation.db is opened by `QuickTranslationStore::open` only:\n{}",
        violations.join("\n")
    );
}

#[test]
fn the_forbidden_needles_appear_in_exactly_the_two_exempt_files() {
    let (root, files) = all_rust_sources();
    let mut expected: Vec<String> = EXEMPT_FILES.iter().map(|s| s.to_string()).collect();
    expected.sort();
    assert_eq!(files_naming_the_store(&root, &files), expected);
}

#[test]
fn the_module_really_opens_the_store_through_both_needles() {
    let path = src_root().join("core/quick_translation/mod.rs");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    for needle in FORBIDDEN {
        assert!(
            text.contains(needle),
            "`core/quick_translation/mod.rs` no longer names `{needle}`: the exemption guards an empty set"
        );
    }
}

#[test]
fn a_hand_built_open_or_comparison_is_caught_and_a_comment_is_not() {
    assert_eq!(
        line_names_the_quick_translation_store("Store::open(StoreSpec::quick_translation(path))"),
        Some("StoreSpec::quick_translation")
    );
    assert_eq!(
        line_names_the_quick_translation_store("if kind == StoreKind::QuickTranslation {"),
        Some("StoreKind::QuickTranslation")
    );
    assert_eq!(
        line_names_the_quick_translation_store("StoreSpec::library_index(path)"),
        None
    );
    assert!(code_lines_naming_the_store("// StoreKind::QuickTranslation").is_empty());
    assert_eq!(
        code_lines_naming_the_store("let a = 1;\nlet k = StoreKind::QuickTranslation;").len(),
        1
    );
}

#[test]
fn the_quick_translation_module_does_not_depend_on_core_ai() {
    let dir = src_root().join("core/quick_translation");
    let mut seen = 0;
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let path = entry.expect("entry").path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            seen += 1;
            let text = fs::read_to_string(&path).expect("read");
            for (line, code) in boundary_scan::code_lines(&text) {
                assert!(
                    !code.contains("core::ai") && !code.contains("super::ai"),
                    "{}:{line} depends on core::ai (AD-13)",
                    path.display()
                );
            }
        }
    }
    assert!(seen >= 1, "no source file scanned");
}
