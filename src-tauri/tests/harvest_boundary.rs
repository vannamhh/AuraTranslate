//! The reviewer-copy harvest path must not name any Review Mode symbol, so Review Mode can be
//! removed without touching the import or the harvest.

use std::fs;
use std::path::PathBuf;

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;

/// Paths relative to the repository root, scanned whole. The tests of the harvest are listed so
/// they keep running once Review Mode is removed; `reviewerImportHooksWiring.test.ts` names
/// `reviewModeState` on purpose, to fail if the harvest path loads it.
const HARVEST_PATH: [&str; 12] = [
    "src-tauri/src/core/export/harvest.rs",
    "src-tauri/src/core/glossary/candidate_store.rs",
    "src/reviewerImportState.ts",
    "src/reviewerImportCommandDeps.ts",
    "src/glossaryQueueState.ts",
    "src/GlossaryQueueOverlay.vue",
    "src/ReviewerImportOverlay.vue",
    "src/config/reviewerImport.ts",
    "src/config/glossary.ts",
    "src-tauri/tests/review_harvest_contract.rs",
    "tests/frontend/reviewerImport.test.ts",
    "tests/frontend/glossaryQueue.test.ts",
];

/// `commands/export.rs` also hosts the `review_diff` command, so only these functions are scanned.
const EXPORT_COMMANDS: &str = "src-tauri/src/commands/export.rs";
const HARVEST_FNS: [&str; 2] = ["fn harvest_reviewer_copies(", "pub fn reviewer_import_confirm("];

const FORBIDDEN: [&str; 4] = ["review_diff", "diff_spans", "reviewModeState", "reviewScrollSync"];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").canonicalize().expect("repository root")
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn hits_in(line: &str) -> Vec<String> {
    let mut found: Vec<String> = FORBIDDEN.iter().filter(|t| line.contains(**t)).map(|t| t.to_string()).collect();
    if line.contains("reviewMode") || line.contains("ReviewMode") {
        found.push("reviewMode/ReviewMode identifier".to_string());
    }
    let mut rest = line;
    while let Some(at) = rest.find("Review") {
        let tail = &rest[at..];
        let ident: String = tail.chars().take_while(|c| is_ident(*c)).collect();
        let starts_ident = rest[..at].chars().next_back().is_none_or(|c| !is_ident(c));
        if starts_ident && (ident == "ReviewDock" || (ident.len() > "ReviewPanel".len() - 1 && ident.ends_with("Panel"))) {
            found.push(ident.clone());
        }
        rest = &tail["Review".len()..];
    }
    found
}

fn violations(files: &[(String, String)]) -> Vec<String> {
    let mut out = Vec::new();
    for (rel, text) in files {
        for (line, content) in boundary_scan::code_lines(text) {
            for token in hits_in(&content) {
                out.push(format!("{rel}:{line}: `{token}`"));
            }
        }
    }
    out
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("harvest path file `{rel}` unreadable: {e}"))
}

/// The function starting at the first `signature`, padded with blank lines so reported line
/// numbers stay those of the whole file.
fn function_text(text: &str, signature: &str) -> String {
    let start = text.find(signature).unwrap_or_else(|| panic!("`{signature}` not found in {EXPORT_COMMANDS}"));
    let chars: Vec<char> = text.chars().collect();
    let start_char = text[..start].chars().count();
    let open = start_char + chars[start_char..].iter().position(|c| *c == '{').expect("function body");
    let close = boundary_scan::matching_close_brace(&chars, open + 1).expect("balanced function body");
    let padding = "\n".repeat(text[..start].matches('\n').count());
    padding + &chars[start_char..=close].iter().collect::<String>()
}

/// Every listed file plus every cut-out function; a renamed or moved file drops below it.
const HARVEST_UNIT_FLOOR: usize = HARVEST_PATH.len() + HARVEST_FNS.len();
const HARVEST_ROOTS: [&str; 4] = ["src", "src-tauri/src", "tests/frontend", "src-tauri/tests"];

fn load() -> Vec<(String, String)> {
    let root = repo_root();
    let mut files: Vec<(String, String)> = HARVEST_ROOTS
        .iter()
        .flat_map(|dir| boundary_scan::paths_with_extensions(&root.join(dir), &["rs", "ts", "vue"]))
        .map(|path| boundary_scan::rel_posix(&root, &path))
        .filter(|rel| HARVEST_PATH.contains(&rel.as_str()))
        .map(|rel| {
            let text = read(&rel);
            (rel, text)
        })
        .collect();
    let commands = read(EXPORT_COMMANDS);
    for signature in HARVEST_FNS {
        files.push((EXPORT_COMMANDS.to_string(), function_text(&commands, signature)));
    }
    files
}

#[test]
fn the_harvest_functions_of_the_export_commands_are_cut_out_alone() {
    let commands = read(EXPORT_COMMANDS);
    for signature in HARVEST_FNS {
        let body = function_text(&commands, signature);
        assert!(body.trim_start().starts_with(signature), "{signature}");
        assert!(!body.contains("fn review_diff("), "{signature} swallowed `review_diff`");
    }
}

#[test]
fn the_harvest_path_names_no_review_mode_symbol() {
    let units = load();
    boundary_scan::assert_population_floor(HARVEST_UNIT_FLOOR, units.len(), "HARVEST_UNIT_FLOOR", "harvest path files and functions");
    let found = violations(&units);
    assert!(found.is_empty(), "harvest path references Review Mode:\n{}", found.join("\n"));
}

#[test]
fn the_scan_catches_a_review_mode_token() {
    let sample = vec![
        (
            "sample.ts".to_string(),
            "import { resetReviewMode } from './elsewhere'\nconst a = review_diff()\nuse ReviewMinePanel\n// reviewScrollSync in a comment\n"
                .to_string(),
        ),
    ];
    let found = violations(&sample);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(found[0].contains("ReviewMode"), "{found:?}");
    assert!(found[0].starts_with("sample.ts:1:"));
    assert!(found[1].starts_with("sample.ts:2:"));
    assert!(found[2].starts_with("sample.ts:3:"));
}
