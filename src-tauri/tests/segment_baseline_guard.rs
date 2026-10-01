//! AD-50 rule 3: a SQL statement that writes `segment.target_text` also writes
//! `baseline_target_text`, except the typing-buffer flush.

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;

const SRC_RS_FLOOR: usize = 88;

const FLUSH_EXEMPT_FILE: &str = "commands/segment.rs";
const FLUSH_EXEMPT_PREFIX: &str = "UPDATE segment SET target_text = ?1, updated_at";

/// Statements that write `target_text` on `segment`, as `(1-based line, collapsed text)`.
/// Comments are stripped first, so a commented-out statement is never read.
fn target_writes(text: &str) -> Vec<(usize, String)> {
    let code: Vec<(usize, String)> =
        boundary_scan::code_lines(&boundary_scan::without_test_modules(text)).collect();
    let mut found = Vec::new();
    for (index, (line, _)) in code.iter().enumerate() {
        let joined: String = code[index..]
            .iter()
            .take(12)
            .map(|(_, content)| content.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        for opener in ["UPDATE segment", "INSERT INTO segment", "REPLACE INTO segment"] {
            let Some(at) = joined.find(opener) else { continue };
            if !code[index].1.contains(opener) {
                continue;
            }
            let after = &joined[at + opener.len()..];
            if after.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
                continue;
            }
            let statement = after.split('"').next().unwrap_or("");
            let statement = statement.split(" WHERE ").next().unwrap_or(statement);
            let collapsed = format!("{opener}{statement}").replace('\\', " ");
            let collapsed = collapsed.split_whitespace().collect::<Vec<_>>().join(" ");
            if collapsed.replace("baseline_target_text", "").contains("target_text") {
                found.push((*line, collapsed));
            }
        }
    }
    found
}

fn lacks_baseline(statement: &str) -> bool {
    !(statement.contains("baseline_target_text") && statement.contains("baseline_translation_origin"))
}

fn is_flush(rel: &str, statement: &str) -> bool {
    rel == FLUSH_EXEMPT_FILE && statement.starts_with(FLUSH_EXEMPT_PREFIX)
}

fn product_sources() -> Vec<(String, String)> {
    boundary_scan::rust_sources(&boundary_scan::src_root())
}

#[test]
fn the_scanned_tree_is_large_enough_to_be_real() {
    boundary_scan::assert_population_floor(
        SRC_RS_FLOOR,
        product_sources().len(),
        "SRC_RS_FLOOR",
        "`.rs` files under `src-tauri/src/**`",
    );
}

#[test]
fn every_statement_writing_segment_target_text_also_writes_the_baseline_except_the_flush() {
    let mut violations = Vec::new();
    for (rel, text) in product_sources() {
        for (line, statement) in target_writes(&text) {
            if lacks_baseline(&statement) && !is_flush(&rel, &statement) {
                violations.push(format!("{rel}:{line}: {statement}"));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "a write to `segment.target_text` without `baseline_target_text` (AD-50 rule 3): {violations:#?}"
    );
}

#[test]
fn the_guard_sees_the_known_writers_and_exactly_one_flush_exemption() {
    let mut with_baseline = 0usize;
    let mut flushes = 0usize;
    for (rel, text) in product_sources() {
        for (_, statement) in target_writes(&text) {
            if lacks_baseline(&statement) {
                if is_flush(&rel, &statement) {
                    flushes += 1;
                }
            } else if rel == "commands/segment.rs" {
                with_baseline += 1;
            }
        }
    }
    assert_eq!(flushes, 1, "the typing-buffer flush is the single named exemption");
    assert!(
        with_baseline >= 3,
        "write_non_user_target, insert_bilingual_segments and write_regroup must all be seen, found {with_baseline}"
    );
}

#[test]
fn the_predicate_catches_a_write_without_the_baseline_and_ignores_comments() {
    let bad = "fn a() {\n    tx.execute(\"UPDATE segment SET target_text = ?1 WHERE id = ?2\", ());\n}\n";
    let hits = target_writes(bad);
    assert_eq!(hits.len(), 1);
    assert!(lacks_baseline(&hits[0].1));

    let ok = "fn a() {\n    tx.execute(\"UPDATE segment SET target_text = ?1, \\\n baseline_target_text = ?1, baseline_translation_origin = ?2 WHERE id = ?2\", ());\n}\n";
    let hits = target_writes(ok);
    assert_eq!(hits.len(), 1);
    assert!(!lacks_baseline(&hits[0].1));

    let half = "fn a() {\n    tx.execute(\"UPDATE segment SET target_text = ?1, baseline_target_text = ?1 WHERE id = ?2\", ());\n}\n";
    let hits = target_writes(half);
    assert_eq!(hits.len(), 1);
    assert!(lacks_baseline(&hits[0].1), "baseline text without baseline origin is a violation");

    let commented = "// \"UPDATE segment SET target_text = ?1\"\nfn a() {}\n";
    assert!(target_writes(commented).is_empty());

    let other = "fn a() { tx.execute(\"UPDATE segment SET status = ?1 WHERE id = ?2\", ()); }\n";
    assert!(target_writes(other).is_empty());
}
