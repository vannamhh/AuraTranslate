//! Export and import of one convention tier (FR159, AD-67 rule 10, 12b): the I/O matrix of the
//! exchange story, driven through the pure command functions the wires wrap.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::convention::{
    ConventionImportPreviewWire, ConventionMatcherState, ConventionPendingImportState,
    ConventionSuggestionItem, clear_pending_convention_import_for_work, convention_add_rule,
    convention_cancel_import, convention_confirm_import, convention_edit_rule,
    convention_export_tier, convention_export_tier_after_dialog, convention_open_import_preview,
    convention_open_import_preview_after_dialog, convention_scan_suggestions,
    convention_set_enabled,
};
use auratranslate_lib::commands::project::{OpenWork, OpenWorkState, create_work_from_text};
use auratranslate_lib::commands::segment::read_open_chapter_segments;
use auratranslate_lib::core::convention::{
    ConflictDecision, RuleRow, RuleTier, apply_import, classify_import, list_rules, parse_import,
    render_tier, visible_rows,
};
use auratranslate_lib::core::delimited_text::Delimiter;
use auratranslate_lib::core::i18n::IpcError;
use auratranslate_lib::core::store::{Store, StoreSpec};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-convention-exchange-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

struct Fixture {
    global: Store,
    work: OpenWork,
    cache: ConventionMatcherState,
    pending: ConventionPendingImportState,
    dir: PathBuf,
}

fn fixture(tag: &str) -> Fixture {
    let dir = temp_dir(tag);
    let global = Store::open(StoreSpec::global(dir.join("global.db"))).expect("global");
    let work = create_work_from_text(&dir, "Exchange", "en", "", "One. Two.".to_owned())
        .expect("create work");
    Fixture {
        global,
        work,
        cache: ConventionMatcherState::default(),
        pending: Mutex::new(None),
        dir,
    }
}

impl Fixture {
    fn open(&self, tier: RuleTier) -> Option<&OpenWork> {
        (tier == RuleTier::Work).then_some(&self.work)
    }

    fn add(&self, tier: RuleTier, avoid: &str, replacement: Option<&str>) {
        convention_add_rule(
            Some(&self.global),
            Some(&self.work),
            &self.cache,
            tier,
            avoid,
            replacement,
        )
        .expect("add rule");
    }

    fn rows(&self, tier: RuleTier) -> Vec<RuleRow> {
        let store = match tier {
            RuleTier::Global => &self.global,
            RuleTier::Work => &self.work.store,
        };
        visible_rows(tier, list_rules(store).expect("rows"))
    }

    fn pairs(&self, tier: RuleTier) -> Vec<(String, Option<String>)> {
        let mut out: Vec<_> = self
            .rows(tier)
            .into_iter()
            .map(|r| (r.avoid_phrase, r.replacement_phrase))
            .collect();
        out.sort();
        out
    }

    fn all_stored(&self, tier: RuleTier) -> Vec<RuleRow> {
        let store = match tier {
            RuleTier::Global => &self.global,
            RuleTier::Work => &self.work.store,
        };
        list_rules(store).expect("stored")
    }

    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.dir.join(name);
        fs::write(&path, bytes).expect("write file");
        path
    }

    fn preview(
        &self,
        tier: RuleTier,
        path: &PathBuf,
    ) -> Result<ConventionImportPreviewWire, IpcError> {
        convention_open_import_preview(
            Some(&self.global),
            self.open(tier),
            &self.pending,
            tier,
            path,
        )
    }

    fn confirm(
        &self,
        tier: RuleTier,
        decisions: &BTreeMap<String, ConflictDecision>,
    ) -> Result<(u32, u32, u32), IpcError> {
        convention_confirm_import(
            Some(&self.global),
            self.open(tier),
            &self.cache,
            &self.pending,
            decisions,
        )
        .map(|s| (s.added, s.replaced, s.kept))
    }

    fn export(&self, tier: RuleTier, name: &str) -> PathBuf {
        let path = self.dir.join(name);
        convention_export_tier(Some(&self.global), self.open(tier), tier, &path).expect("export");
        path
    }
}

fn pair(avoid: &str, replacement: Option<&str>) -> (String, Option<String>) {
    (avoid.to_owned(), replacement.map(str::to_owned))
}

fn take(keys: &[&str]) -> BTreeMap<String, ConflictDecision> {
    keys.iter()
        .map(|k| ((*k).to_owned(), ConflictDecision::TakeFile))
        .collect()
}

fn code<T: std::fmt::Debug>(result: Result<T, IpcError>) -> String {
    result.expect_err("expected an error").code().to_owned()
}

#[test]
fn a_tier_round_trips_through_csv_and_tsv_into_an_empty_tier() {
    for (name, tier) in [("out.csv", RuleTier::Global), ("out.tsv", RuleTier::Work)] {
        let f = fixture(name);
        f.add(tier, "đảm bảo", Some("bảo đảm"));
        f.add(tier, "một cách", None);
        f.add(tier, "-ạ", None);
        let path = f.export(tier, name);

        let empty = fixture(&format!("empty-{name}"));
        let preview = empty.preview(tier, &path).expect("preview");
        assert_eq!(
            (
                preview.row_count,
                preview.new_count,
                preview.identical_count
            ),
            (3, 3, 0)
        );
        assert_eq!(
            empty.confirm(tier, &BTreeMap::new()).expect("confirm"),
            (3, 0, 0)
        );
        assert_eq!(
            empty.pairs(tier),
            vec![
                pair("-ạ", None),
                pair("một cách", None),
                pair("đảm bảo", Some("bảo đảm")),
            ]
        );
    }
}

#[test]
fn the_formula_guard_is_written_and_stripped_in_both_delimiters() {
    let f = fixture("guard");
    f.add(RuleTier::Global, "-ạ", None);
    for (name, delimiter) in [("g.csv", Delimiter::Csv), ("g.tsv", Delimiter::Tsv)] {
        let path = f.export(RuleTier::Global, name);
        let text = fs::read_to_string(&path).expect("read");
        assert!(
            text.contains("'-ạ"),
            "{name} must guard the formula: {text:?}"
        );
        let rows = parse_import(&text).expect("parse");
        assert_eq!(rows[0].avoid_phrase, "-ạ");
        assert_eq!(rows[0].replacement_phrase, None);
        assert_eq!(render_tier(&f.rows(RuleTier::Global), delimiter), text);
    }
}

#[test]
fn a_bom_is_stripped_and_the_delimiter_follows_the_header() {
    let f = fixture("bom");
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice("avoid\treplacement\nđảm bảo\tbảo đảm\n".as_bytes());
    let path = f.file("bom.tsv", &bytes);
    let preview = f.preview(RuleTier::Global, &path).expect("preview");
    assert_eq!(preview.new_count, 1);
    assert_eq!(
        f.confirm(RuleTier::Global, &BTreeMap::new())
            .expect("confirm"),
        (1, 0, 0)
    );
    assert_eq!(
        f.pairs(RuleTier::Global),
        vec![pair("đảm bảo", Some("bảo đảm"))]
    );
}

#[test]
fn classification_counts_new_identical_and_conflict_and_the_default_adds_only_the_new() {
    let f = fixture("classify");
    f.add(RuleTier::Global, "a", Some("x"));
    f.add(RuleTier::Global, "b", Some("old"));
    let path = f.file("in.csv", b"avoid,replacement\na,x\nb,new\nc,z\n");
    let preview = f.preview(RuleTier::Global, &path).expect("preview");
    assert_eq!(
        (
            preview.new_count,
            preview.identical_count,
            preview.conflicts.len()
        ),
        (1, 1, 1)
    );
    let conflict = &preview.conflicts[0];
    assert_eq!(conflict.existing_replacement_phrase.as_deref(), Some("old"));
    assert_eq!(conflict.file_replacement_phrase.as_deref(), Some("new"));

    assert_eq!(
        f.confirm(RuleTier::Global, &BTreeMap::new())
            .expect("confirm"),
        (1, 0, 2)
    );
    assert_eq!(
        f.pairs(RuleTier::Global),
        vec![
            pair("a", Some("x")),
            pair("b", Some("old")),
            pair("c", Some("z"))
        ]
    );
}

#[test]
fn taking_the_file_replaces_that_conflict_only() {
    let f = fixture("take");
    f.add(RuleTier::Global, "a", Some("x"));
    f.add(RuleTier::Global, "b", Some("old"));
    let path = f.file("in.csv", b"avoid,replacement\na,y\nb,new\n");
    let preview = f.preview(RuleTier::Global, &path).expect("preview");
    assert_eq!(preview.conflicts.len(), 2);
    assert_eq!(
        f.confirm(RuleTier::Global, &take(&["b"])).expect("confirm"),
        (0, 1, 1)
    );
    assert_eq!(
        f.pairs(RuleTier::Global),
        vec![pair("a", Some("x")), pair("b", Some("new"))]
    );
}

#[test]
fn a_file_row_matches_a_disabled_global_flag_as_new_and_switches_it_back_on() {
    let f = fixture("flag");
    f.add(RuleTier::Global, "thập kỷ", Some("thập kỉ"));
    convention_set_enabled(Some(&f.global), Some(&f.work), &f.cache, "thập kỷ", false)
        .expect("disable in work");
    assert!(f.rows(RuleTier::Work).is_empty());
    assert_eq!(f.all_stored(RuleTier::Work).len(), 1);

    let exported = f.export(RuleTier::Work, "work.csv");
    assert_eq!(
        fs::read_to_string(&exported).expect("read"),
        "avoid,replacement\n",
        "a flag row is not exported"
    );

    let path = f.file(
        "in.csv",
        "avoid,replacement\nthập kỷ,thập niên\n".as_bytes(),
    );
    let preview = f.preview(RuleTier::Work, &path).expect("preview");
    assert_eq!((preview.new_count, preview.conflicts.len()), (1, 0));
    assert_eq!(
        f.confirm(RuleTier::Work, &BTreeMap::new())
            .expect("confirm"),
        (1, 0, 0)
    );
    assert_eq!(
        f.pairs(RuleTier::Work),
        vec![pair("thập kỷ", Some("thập niên"))]
    );
    assert_eq!(
        f.all_stored(RuleTier::Work).len(),
        1,
        "the flag row is reused, not duplicated"
    );
}

#[test]
fn a_file_with_a_fault_is_refused_whole_with_zero_writes_and_the_first_error() {
    let f = fixture("faults");
    f.add(RuleTier::Global, "keep", Some("me"));
    let before = f.pairs(RuleTier::Global);
    let cases: [(&str, &[u8], &str); 7] = [
        (
            "dup.csv",
            "avoid,replacement\nĐảm bảo,a\nđảm bảo,b\n".as_bytes(),
            "convention.import_duplicate_avoid",
        ),
        (
            "missing.csv",
            b"avoid,other\na,b\n",
            "convention.import_missing_column",
        ),
        (
            "blank.csv",
            b"avoid,replacement\n,b\n",
            "convention.import_blank_avoid_phrase",
        ),
        (
            "quote.csv",
            b"avoid,replacement\n\"a,b\n",
            "convention.import_unterminated_quoted_field",
        ),
        (
            "utf8.csv",
            &[0x61, 0x76, 0xFF, 0xFE],
            "convention.import_not_utf8",
        ),
        (
            "cells.csv",
            b"avoid,replacement\na,b,c\n",
            "convention.import_cell_count_mismatch",
        ),
        (
            "nodelim.csv",
            b"avoid\na\n",
            "convention.import_delimiter_unresolved",
        ),
    ];
    for (name, bytes, expected) in cases {
        let path = f.file(name, bytes);
        let got = code(f.preview(RuleTier::Global, &path));
        assert_eq!(got, expected, "{name}");
        assert_eq!(f.pairs(RuleTier::Global), before, "{name} wrote");
        assert_eq!(
            code(f.confirm(RuleTier::Global, &BTreeMap::new())),
            "convention.no_pending_import"
        );
    }
}

#[test]
fn a_file_over_the_ceiling_is_refused_before_any_plan_is_kept() {
    let f = fixture("big");
    let mut text = String::from("avoid,replacement\n");
    while text.len() <= 1024 * 1024 {
        text.push_str("aaaaaaaaaaaaaaaaaaaa,b\n");
    }
    let path = f.file("big.csv", text.as_bytes());
    let got = code(f.preview(RuleTier::Global, &path));
    assert!(got.contains("too_large"), "{got}");
    assert!(f.pending.lock().expect("lock").is_none());
}

#[test]
fn a_rule_edited_after_the_preview_makes_the_confirm_stale_with_zero_writes() {
    let f = fixture("stale");
    f.add(RuleTier::Global, "b", Some("old"));
    let path = f.file("in.csv", b"avoid,replacement\nb,new\nc,z\n");
    f.preview(RuleTier::Global, &path).expect("preview");

    let id = f.rows(RuleTier::Global)[0].id;
    convention_edit_rule(
        Some(&f.global),
        None,
        &f.cache,
        RuleTier::Global,
        id,
        "b",
        Some("edited"),
    )
    .expect("edit");

    assert_eq!(
        code(f.confirm(RuleTier::Global, &take(&["b"]))),
        "convention.import_stale"
    );
    assert_eq!(f.pairs(RuleTier::Global), vec![pair("b", Some("edited"))]);
    assert!(
        f.pending.lock().expect("lock").is_some(),
        "a failed confirm keeps the plan"
    );
}

#[test]
fn the_in_transaction_recheck_catches_a_change_the_command_could_not_see() {
    let f = fixture("stale-core");
    f.add(RuleTier::Global, "b", Some("old"));
    let rows = parse_import("avoid,replacement\nb,new\nc,z\n").expect("parse");
    let plans = classify_import(&rows, &f.rows(RuleTier::Global));
    let id = f.rows(RuleTier::Global)[0].id;
    convention_edit_rule(
        Some(&f.global),
        None,
        &f.cache,
        RuleTier::Global,
        id,
        "b",
        Some("edited"),
    )
    .expect("edit");
    let err = apply_import(&f.global, RuleTier::Global, &plans, &take(&["b"])).expect_err("stale");
    assert_eq!(IpcError::from(err).code(), "convention.import_stale");
    assert_eq!(f.pairs(RuleTier::Global), vec![pair("b", Some("edited"))]);
}

#[test]
fn a_decision_for_a_key_that_is_not_a_conflict_is_refused_with_zero_writes() {
    let f = fixture("unknown-key");
    f.add(RuleTier::Global, "a", Some("x"));
    let path = f.file("in.csv", b"avoid,replacement\na,x\nc,z\n");
    f.preview(RuleTier::Global, &path).expect("preview");
    assert_eq!(
        code(f.confirm(RuleTier::Global, &take(&["c"]))),
        "convention.import_decision_unknown_rule"
    );
    assert_eq!(f.pairs(RuleTier::Global), vec![pair("a", Some("x"))]);
}

#[test]
fn a_cancelled_dialog_changes_nothing_and_raises_nothing() {
    let f = fixture("cancel-dialog");
    f.add(RuleTier::Global, "a", Some("x"));
    assert_eq!(
        convention_export_tier_after_dialog(Some(&f.global), None, RuleTier::Global, None)
            .expect("export"),
        None
    );
    let path = f.file("in.csv", b"avoid,replacement\nb,y\n");
    f.preview(RuleTier::Global, &path).expect("first preview");
    let kept_plan_before = f.pending.lock().expect("lock").is_some();
    let after = convention_open_import_preview_after_dialog(
        Some(&f.global),
        None,
        &f.pending,
        RuleTier::Global,
        None,
    )
    .expect("preview");
    assert!(after.is_none());
    assert!(kept_plan_before && f.pending.lock().expect("lock").is_some());
    assert_eq!(f.pairs(RuleTier::Global), vec![pair("a", Some("x"))]);
}

#[test]
fn the_after_dialog_functions_write_and_read_the_picked_path() {
    let f = fixture("after-dialog");
    f.add(RuleTier::Global, "a", Some("x"));
    let path = f.dir.join("picked.csv");
    let shown = convention_export_tier_after_dialog(
        Some(&f.global),
        None,
        RuleTier::Global,
        Some(path.clone()),
    )
    .expect("export")
    .expect("path");
    assert_eq!(shown, path.display().to_string());
    let preview = convention_open_import_preview_after_dialog(
        Some(&f.global),
        None,
        &f.pending,
        RuleTier::Global,
        Some(path),
    )
    .expect("preview")
    .expect("wire");
    assert_eq!(preview.identical_count, 1);
    assert_eq!(preview.file_name, "picked.csv");
}

#[test]
fn cancel_and_a_closed_work_drop_the_pending_plan() {
    let f = fixture("pending");
    let path = f.file("in.csv", b"avoid,replacement\nb,y\n");
    f.preview(RuleTier::Work, &path).expect("preview");
    convention_cancel_import(&f.pending);
    assert_eq!(
        code(f.confirm(RuleTier::Work, &BTreeMap::new())),
        "convention.no_pending_import"
    );

    f.preview(RuleTier::Work, &path).expect("preview");
    clear_pending_convention_import_for_work(&f.pending);
    assert_eq!(
        code(f.confirm(RuleTier::Work, &BTreeMap::new())),
        "convention.no_pending_import"
    );

    f.preview(RuleTier::Global, &path).expect("preview");
    clear_pending_convention_import_for_work(&f.pending);
    assert!(
        f.pending.lock().expect("lock").is_some(),
        "a Global plan survives a Work closing"
    );
}

#[test]
fn the_work_tier_without_an_open_work_is_unavailable() {
    let f = fixture("no-work");
    let path = f.dir.join("x.csv");
    let got = code(convention_export_tier(
        Some(&f.global),
        None,
        RuleTier::Work,
        &path,
    ));
    assert_eq!(got, "convention.work_tier_unavailable");
    assert!(!path.exists());
}

#[test]
fn a_successful_confirm_drops_the_cached_matcher() {
    let f = fixture("cache");
    let work = create_work_from_text(&f.dir, "Scan", "en", "", "Assure safety.".to_owned())
        .expect("scan work");
    let segment_id = read_open_chapter_segments(Some(&work))
        .expect("segments")
        .segments[0]
        .id;
    let state: OpenWorkState = Mutex::new(Some(work));
    let scan = |f: &Fixture| {
        convention_scan_suggestions(
            Some(&f.global),
            Some(&state),
            &f.cache,
            1,
            vec![ConventionSuggestionItem {
                segment_id,
                text: "Đảm bảo an toàn.".to_owned(),
            }],
        )
        .expect("scan")
        .items
        .iter()
        .map(|item| item.spans.len())
        .sum::<usize>()
    };
    assert_eq!(scan(&f), 0);
    let path = f.file("in.csv", "avoid,replacement\nđảm bảo,bảo đảm\n".as_bytes());
    f.preview(RuleTier::Global, &path).expect("preview");
    f.confirm(RuleTier::Global, &BTreeMap::new())
        .expect("confirm");
    assert_eq!(
        scan(&f),
        1,
        "the cached matcher must not outlive the import"
    );
}
