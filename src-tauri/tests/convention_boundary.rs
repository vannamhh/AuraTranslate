//! AD-67 rule 1, 5 and 12 (h): `core/convention/` and `core::ai` do not import each other, and
//! the Glossary check lives outside `core/convention/`.

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;
use boundary_scan::{code_lines, is_inside};

const CONVENTION_DIR: &str = "core/convention";
const AI_DIR: &str = "core/ai";
const CONVENTION_FLOOR: usize = 7;
const AI_FLOOR: usize = 5;
const SRC_RS_FLOOR: usize = 133;

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Single spaces, none next to `:`, `{`, `}` or `,`, so `core :: { a , b }` reads as `core::{a,b}`
/// while `ai as x` keeps the space that ends the identifier.
fn squash_path_whitespace(code: &str) -> String {
    let single = code.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out = String::with_capacity(single.len());
    let chars: Vec<char> = single.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == ' ' {
            let glued = |x: Option<&char>| x.is_some_and(|x| matches!(x, ':' | '{' | '}' | ','));
            if glued(i.checked_sub(1).and_then(|j| chars.get(j))) || glued(chars.get(i + 1)) {
                continue;
            }
        }
        out.push(c);
    }
    out
}

/// `code` names `core::<module>` as a path, including inside a `core::{ a, module }` group.
/// Whitespace is ignored so a path split over lines is still seen.
fn names_core_module(code: &str, module: &str) -> bool {
    let squashed = squash_path_whitespace(code);
    let direct = format!("core::{module}");
    let mut from = 0;
    while let Some(at) = squashed[from..].find(&direct) {
        let end = from + at + direct.len();
        if !squashed[end..].chars().next().is_some_and(is_ident_char) {
            return true;
        }
        from = end;
    }
    let mut from = 0;
    while let Some(at) = squashed[from..].find("core::{") {
        let open = from + at + "core::{".len();
        let mut depth = 1usize;
        let mut close = squashed.len();
        for (i, c) in squashed[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        close = open + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        if squashed[open..close]
            .split(|c: char| !is_ident_char(c))
            .any(|word| word == module)
        {
            return true;
        }
        from = open;
    }
    false
}

fn names_word(code: &str, word: &str) -> bool {
    code.to_ascii_lowercase()
        .split(|c: char| !is_ident_char(c))
        .any(|found| found == word)
}

fn violations_in(
    sources: &[(String, String)],
    dir: &str,
    offends: impl Fn(&str) -> bool,
) -> Vec<String> {
    let mut found = Vec::new();
    for (rel, text) in sources {
        if !is_inside(rel, dir) {
            continue;
        }
        for (line, code) in code_lines(text) {
            if offends(&code) {
                found.push(format!("{rel}:{line}: {code}"));
            }
        }
    }
    found
}

fn sources() -> Vec<(String, String)> {
    boundary_scan::rust_sources(&boundary_scan::src_root())
}

#[test]
fn the_scanned_trees_are_large_enough_to_be_real() {
    let all = sources();
    boundary_scan::assert_population_floor(
        SRC_RS_FLOOR,
        all.len(),
        "SRC_RS_FLOOR",
        "`.rs` files under `src/**`",
    );
    let inside = |dir: &str| all.iter().filter(|(rel, _)| is_inside(rel, dir)).count();
    boundary_scan::assert_population_floor(
        CONVENTION_FLOOR,
        inside(CONVENTION_DIR),
        "CONVENTION_FLOOR",
        "`.rs` files under `src/core/convention/**`",
    );
    boundary_scan::assert_population_floor(
        AI_FLOOR,
        inside(AI_DIR),
        "AI_FLOOR",
        "`.rs` files under `src/core/ai/**`",
    );
}

#[test]
fn core_convention_does_not_import_core_ai() {
    let found = violations_in(&sources(), CONVENTION_DIR, |code| {
        names_core_module(code, "ai") || code.contains("super::ai")
    });
    assert!(
        found.is_empty(),
        "core/convention imports core::ai: {found:#?}"
    );
}

#[test]
fn core_ai_does_not_import_core_convention() {
    let found = violations_in(&sources(), AI_DIR, |code| {
        names_core_module(code, "convention") || code.contains("super::convention")
    });
    assert!(
        found.is_empty(),
        "core/ai imports core::convention: {found:#?}"
    );
}

#[test]
fn the_glossary_check_is_not_inside_core_convention() {
    let found = violations_in(&sources(), CONVENTION_DIR, |code| {
        names_core_module(code, "glossary") || names_word(code, "glossary")
    });
    assert!(
        found.is_empty(),
        "core/convention names the Glossary: {found:#?}"
    );
}

#[test]
fn the_predicates_catch_each_import_shape_and_ignore_longer_names() {
    assert!(names_core_module("use crate::core::ai::rag;", "ai"));
    assert!(names_core_module("use crate::core::{store, ai};", "ai"));
    assert!(names_core_module(
        "use crate::core::{\n store,\n ai::rag,\n};",
        "ai"
    ));
    assert!(names_core_module(
        "crate::core::convention::Matcher::build()",
        "convention"
    ));
    assert!(names_core_module("use crate::core::ai as client;", "ai"));
    assert!(!names_core_module(
        "use crate::core::aiconfig::Field;",
        "ai"
    ));
    assert!(!names_core_module(
        "use crate::core::{store, aiconfig};",
        "ai"
    ));
    assert!(names_word("let glossary = 1;", "glossary"));
    assert!(!names_word("let glossary_x = 1;", "glossary"));
}

#[test]
fn the_gate_fails_on_an_injected_import() {
    let injected = vec![(
        "core/convention/matcher.rs".to_owned(),
        "use crate::core::ai::rag::render;\nfn f() {}\n".to_owned(),
    )];
    let found = violations_in(&injected, CONVENTION_DIR, |code| {
        names_core_module(code, "ai")
    });
    assert_eq!(found.len(), 1);
    let commented = vec![(
        "core/convention/matcher.rs".to_owned(),
        "// use crate::core::ai::rag::render;\n".to_owned(),
    )];
    assert!(
        violations_in(&commented, CONVENTION_DIR, |code| names_core_module(
            code, "ai"
        ))
        .is_empty()
    );
}
