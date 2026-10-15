//! AD-57: the Work's "only mine" switch decides which side of the TM the using surfaces read,
//! counts what it hid, and leaves lookup, management and export alone.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::segment::{
    TmPrefillStatus, load_open_chapter_segments, prepare_tm_concordance,
    read_open_chapter_segments, score_tm_concordance,
};
use auratranslate_lib::commands::tm::{tm_export_counts, tm_export_tier};
use auratranslate_lib::core::ai::rag::{
    GlossaryInjectionStatus, TmInjectionStatus, assemble_prompt, gather_tm_context, load_tm_rows,
};
use auratranslate_lib::core::matching::MatchLang;
use auratranslate_lib::core::scope::ScopeResolver;
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};
use auratranslate_lib::core::tm::tmx::distinct_tier_pairs;
use auratranslate_lib::core::tm::{
    MANAGE_GROUP_LIMIT, ManageSort, PairOrigin, PairOriginFilter, TierFilter, TmExportFilter,
    TmSidePolicy, TmStoreError, TmTier, pairs_for_source, rank_fuzzy_candidates,
    rank_manage_listing, read_only_mine, read_tier_rows, target_suggestions_for_term,
    write_only_mine,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-tm-policy-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("tao thu muc");
    dir
}

struct Fixture {
    root: PathBuf,
    global: Store,
    open: OpenWork,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(Path::new(&self.root));
    }
}

fn fixture(tag: &str, text: &str) -> Fixture {
    let root = temp_dir(tag);
    let open = create_work_from_text(&root, tag, "zh", "", text.to_owned()).expect("tao tac pham");
    let global = Store::open(StoreSpec::global(root.join("global.db"))).expect("mo global.db");
    Fixture { root, global, open }
}

fn seed(store: &Store, rows: &[(&str, &str, &str)]) {
    let rows: Vec<(String, String, String)> = rows
        .iter()
        .map(|(a, b, c)| ((*a).to_owned(), (*b).to_owned(), (*c).to_owned()))
        .collect();
    store
        .write(move |tx: &Transaction<'_>| {
            for (source, target, origin) in &rows {
                tx.execute(
                    "INSERT INTO tm_unit (source_text, target_text, translation_origin, created_at) \
                     VALUES (?1, ?2, ?3, '2026-01-01T00:00:00.000Z')",
                    (source, target, origin),
                )?;
            }
            Ok(())
        })
        .expect("gieo tm_unit");
}

fn policy(f: &Fixture) -> TmSidePolicy {
    TmSidePolicy::from_work(Some(&f.open.store)).expect("doc cong tac")
}

fn exact(f: &Fixture, source: &str) -> (Vec<String>, usize) {
    let (pairs, hidden) = pairs_for_source(
        &f.open.scope,
        &f.global,
        Some(&f.open.store),
        source,
        policy(f),
    )
    .expect("tra chinh xac")
    .into_parts();
    (pairs.into_iter().map(|p| p.target_text).collect(), hidden)
}

#[test]
fn a_new_work_starts_with_the_switch_off_and_the_column_refuses_other_values() {
    let f = fixture("default", "一。");
    assert!(!read_only_mine(&f.open.store).unwrap());
    assert!(!policy(&f).only_mine());
    write_only_mine(&f.open.store, true).unwrap();
    assert!(read_only_mine(&f.open.store).unwrap());
    let refused = f.open.store.write(|tx: &Transaction<'_>| {
        tx.execute("UPDATE work SET tm_only_mine = 2 WHERE id = 1", [])
    });
    assert!(refused.is_err());
    assert!(!TmSidePolicy::from_work(None).unwrap().only_mine());
    assert!(!TmSidePolicy::browse().only_mine());
}

#[test]
fn exact_lookup_drops_the_others_side_and_counts_distinct_hidden_pairs_only() {
    let f = fixture("exact", "一。");
    seed(
        &f.global,
        &[
            ("S", "mine", "self"),
            ("S", "other", "other"),
            ("S", "imported", "bilingual_import"),
            ("S", "both", "other"),
        ],
    );
    seed(
        &f.open.store,
        &[("S", "both", "self"), ("S", "other", "other")],
    );

    assert_eq!(exact(&f, "S").1, 0);
    assert_eq!(exact(&f, "S").0.len(), 6);

    write_only_mine(&f.open.store, true).unwrap();
    let (targets, hidden) = exact(&f, "S");
    assert_eq!(targets, ["both", "mine"]);
    assert_eq!(
        hidden, 2,
        "`other` and `imported` are hidden; `both` keeps a copy of mine"
    );
}

#[test]
fn the_strip_filters_before_the_cut_of_three() {
    let f = fixture("strip", "一。");
    seed(
        &f.global,
        &[
            ("一二三四五六七八九十。", "o1", "other"),
            ("一二三四五六七八九十，", "o2", "other"),
            ("一二三四五六七八九十！", "o3", "bilingual_import"),
            ("一二三四五六七", "mine", "self"),
        ],
    );
    let rank = |policy| {
        rank_fuzzy_candidates(
            &ScopeResolver::global_only(),
            read_tier_rows(&f.global, None, policy).unwrap(),
            "一二三四五六七八九十",
            MatchLang::Zh,
            40,
        )
        .unwrap()
        .into_parts()
    };
    let (all, none_hidden) = rank(TmSidePolicy::browse());
    assert_eq!(none_hidden, 0);
    assert!(
        all.iter().all(|m| m.pair.target_text != "mine"),
        "precondition: three other pairs outrank mine"
    );

    write_only_mine(&f.open.store, true).unwrap();
    let (kept, hidden) = rank(policy(&f));
    assert_eq!(
        kept.iter()
            .map(|m| m.pair.target_text.as_str())
            .collect::<Vec<_>>(),
        ["mine"]
    );
    assert_eq!(hidden, 3);
}

#[test]
fn glossary_suggestions_carry_the_hidden_count() {
    let f = fixture("suggest", "一。");
    seed(
        &f.global,
        &[("T", "x", "other"), ("T", "y", "bilingual_import")],
    );
    let found = |f: &Fixture| {
        target_suggestions_for_term(
            &f.open.scope,
            &f.global,
            Some(&f.open.store),
            "T",
            policy(f),
        )
        .unwrap()
    };
    assert_eq!(found(&f).hidden_count, 0);
    assert_eq!(found(&f).suggestions.len(), 2);
    write_only_mine(&f.open.store, true).unwrap();
    let on = found(&f);
    assert!(on.suggestions.is_empty());
    assert_eq!(on.hidden_count, 2);
}

#[test]
fn an_unknown_stored_origin_stays_an_error_under_the_switch() {
    let f = fixture("unknown", "一。");
    seed(&f.global, &[("S", "g", "x")]);
    write_only_mine(&f.open.store, true).unwrap();
    let err = pairs_for_source(
        &f.open.scope,
        &f.global,
        Some(&f.open.store),
        "S",
        policy(&f),
    )
    .expect_err("nguon goc la");
    assert_eq!(
        err,
        TmStoreError::UnknownPairOrigin {
            value: "x".to_owned()
        }
    );
}

#[test]
fn prefill_skips_the_others_side_and_reports_what_it_hid() {
    let f = fixture("prefill", "一二三。");
    let source = read_open_chapter_segments(Some(&f.open)).unwrap().segments[0]
        .source_text
        .clone();
    seed(&f.global, &[(&source, "cua nguoi khac", "other")]);
    write_only_mine(&f.open.store, true).unwrap();

    let loaded = load_open_chapter_segments(Some(&f.global), Some(&f.open), true).unwrap();
    assert_eq!(loaded.tm_prefill, TmPrefillStatus::Ran { hidden_count: 1 });
    assert!(loaded.tm_filled_segment_ids.is_empty());

    write_only_mine(&f.open.store, false).unwrap();
    let loaded = load_open_chapter_segments(Some(&f.global), Some(&f.open), true).unwrap();
    assert_eq!(loaded.tm_prefill, TmPrefillStatus::Ran { hidden_count: 0 });
    assert_eq!(loaded.tm_filled_segment_ids.len(), 1);
}

#[test]
fn the_prompt_ledger_carries_the_hidden_count_and_a_batch_keeps_the_policy_it_read() {
    let f = fixture("rag", "一。");
    seed(
        &f.global,
        &[
            ("一二三四五六七八九十。", "cua nguoi khac", "other"),
            ("一二三四五六七八九", "cua toi", "self"),
        ],
    );
    let body = "TM:\n{{tm_similar_segments}}\n{{source_segment}}";
    let sentence = "一二三四五六七八九十";
    let hidden_of = |rows: &_| {
        let context = gather_tm_context(Some(rows), &f.open.scope, "zh", sentence)
            .unwrap()
            .expect("da hoi TM");
        let (prompt, ledger) = assemble_prompt(
            body,
            sentence,
            GlossaryInjectionStatus::NotAsked,
            Some(&context),
        );
        let TmInjectionStatus::Searched { hidden_count, .. } = ledger.tm else {
            panic!("phai la Searched");
        };
        (prompt, hidden_count)
    };

    write_only_mine(&f.open.store, true).unwrap();
    let batch = load_tm_rows(body, &f.global, Some(&f.open.store))
        .unwrap()
        .expect("co marker");
    write_only_mine(&f.open.store, false).unwrap();
    let (prompt, hidden) = hidden_of(&batch);
    assert!(!prompt.contains("cua nguoi khac"));
    assert!(prompt.contains("cua toi"));
    assert_eq!(hidden, 1, "the batch keeps the value read at its start");

    let next = load_tm_rows(body, &f.global, Some(&f.open.store))
        .unwrap()
        .expect("co marker");
    let (prompt, hidden) = hidden_of(&next);
    assert!(prompt.contains("cua nguoi khac"));
    assert_eq!(hidden, 0);

    assert!(
        load_tm_rows("no marker", &f.global, Some(&f.open.store))
            .unwrap()
            .is_none()
    );
}

#[test]
fn the_mine_export_filter_is_independent_of_the_switch_and_keeps_a_text_pair_with_a_mine_copy() {
    let f = fixture("export", "一。");
    seed(
        &f.open.store,
        &[
            ("A", "a", "self"),
            ("B", "b", "other"),
            ("C", "c", "bilingual_import"),
            ("D", "d", "other"),
            ("D", "d", "self"),
        ],
    );
    let export =
        |filter| distinct_tier_pairs(&f.open.store, TmTier::Work, filter).expect("doc tang");
    for flag in [false, true] {
        write_only_mine(&f.open.store, flag).unwrap();
        let mine = export(TmExportFilter::Mine);
        assert_eq!(
            mine.iter()
                .map(|p| p.target_text.as_str())
                .collect::<Vec<_>>(),
            ["a", "d"]
        );
        assert!(
            mine.iter()
                .all(|p| p.translation_origin == PairOrigin::SelfTranslated)
        );
        assert_eq!(export(TmExportFilter::All).len(), 4);
    }
}

#[test]
fn concordance_lists_the_others_and_imported_pairs_with_the_switch_on() {
    let f = fixture("concordance", "一。");
    seed(
        &f.open.store,
        &[
            ("师父在此", "other-pair", "other"),
            ("师父来了", "imported-pair", "bilingual_import"),
            ("师父走了", "mine-pair", "self"),
        ],
    );
    let targets = |f: &Fixture| -> Vec<String> {
        let scan = prepare_tm_concordance(Some(&f.global), Some(&f.open), "师父").unwrap();
        let mut found: Vec<String> = score_tm_concordance(scan)
            .unwrap()
            .hits
            .into_iter()
            .map(|h| h.target_text)
            .collect();
        found.sort();
        found
    };
    let expected = ["imported-pair", "mine-pair", "other-pair"];
    assert_eq!(targets(&f), expected);
    write_only_mine(&f.open.store, true).unwrap();
    assert_eq!(targets(&f), expected);
}

#[test]
fn the_mine_export_count_equals_the_tu_written_and_ignores_the_switch() {
    let f = fixture("export-count", "一。");
    seed(
        &f.open.store,
        &[
            ("A", "a", "self"),
            ("B", "b", "other"),
            ("C", "c", "bilingual_import"),
            ("D", "d", "other"),
            ("D", "d", "self"),
        ],
    );
    for flag in [false, true] {
        write_only_mine(&f.open.store, flag).unwrap();
        let counts = tm_export_counts(Some(&f.global), Some(&f.open), "work").unwrap();
        let path = f.root.join(format!("mine-{flag}.tmx"));
        tm_export_tier(Some(&f.global), Some(&f.open), "work", "mine", &path).unwrap();
        let tmx = fs::read_to_string(&path).unwrap();
        let written = tmx.matches("<tu>").count() + tmx.matches("<tu ").count();
        assert_eq!(counts.mine, 2);
        assert_eq!(written, counts.mine);
        assert_eq!(counts.all, 4);
    }
}

fn seed_groups(store: &Store, groups: usize, mine_from: usize) {
    store
        .write(move |tx: &Transaction<'_>| {
            for i in 0..groups {
                let origin = if i >= mine_from { "self" } else { "other" };
                tx.execute(
                    "INSERT INTO tm_unit (source_text, target_text, translation_origin, created_at) \
                     VALUES (?1, ?2, ?3, ?4)",
                    (
                        format!("src{i:03}"),
                        format!("tgt{:03}", groups - 1 - i),
                        origin,
                        format!("2026-01-01T00:{:02}:{:02}.000Z", i / 60, i % 60),
                    ),
                )?;
            }
            Ok(())
        })
        .expect("gieo nhom");
}

#[test]
fn every_manage_sort_axis_orders_the_whole_store_before_the_cap_of_200_groups() {
    let f = fixture("sort", "一。");
    let groups = MANAGE_GROUP_LIMIT + 50;
    seed_groups(&f.global, groups, groups - 20);
    let list = |sort| {
        let snapshot = read_tier_rows(&f.global, None, TmSidePolicy::browse()).expect("doc tang");
        rank_manage_listing(
            &f.open.scope,
            snapshot,
            TierFilter::Both,
            PairOriginFilter::All,
            "",
            sort,
        )
        .expect("lap danh sach")
    };
    let sources = |sort| -> Vec<String> {
        list(sort)
            .groups
            .into_iter()
            .map(|g| g.source_text)
            .collect()
    };

    let source = list(ManageSort::Source);
    assert_eq!(source.total_groups, groups);
    assert_eq!(source.groups.len(), MANAGE_GROUP_LIMIT);
    assert_eq!(source.groups[0].source_text, "src000");
    assert_eq!(source.groups[MANAGE_GROUP_LIMIT - 1].source_text, "src199");

    let newest = sources(ManageSort::Newest);
    assert_eq!(newest[0], format!("src{:03}", groups - 1));
    assert_eq!(newest[MANAGE_GROUP_LIMIT - 1], "src050");
    let oldest = sources(ManageSort::Oldest);
    assert_eq!(oldest[0], "src000");
    assert_eq!(oldest[MANAGE_GROUP_LIMIT - 1], "src199");

    let target = list(ManageSort::Target);
    assert_eq!(target.groups[0].rows[0].pair.target_text, "tgt000");
    assert_eq!(
        target.groups[0].source_text,
        format!("src{:03}", groups - 1)
    );
    assert_eq!(
        target.groups[MANAGE_GROUP_LIMIT - 1].rows[0]
            .pair
            .target_text,
        "tgt199"
    );

    let side = list(ManageSort::Side);
    let leading: Vec<PairOrigin> = side
        .groups
        .iter()
        .take(20)
        .map(|g| g.rows[0].pair.translation_origin)
        .collect();
    assert!(
        leading.iter().all(|o| *o == PairOrigin::SelfTranslated),
        "20 nhom cua toi o cuoi bang chu cai van phai len dau truoc khi cat 200: {leading:?}"
    );
    assert_eq!(
        side.groups[20].rows[0].pair.translation_origin,
        PairOrigin::Other
    );
}

#[test]
fn the_manage_listing_ignores_the_switch_and_counts_tiers_with_none_for_a_closed_work() {
    let f = fixture("listing", "一。");
    seed(
        &f.open.store,
        &[("A", "a", "other"), ("B", "b", "bilingual_import")],
    );
    seed(&f.global, &[("C", "c", "self")]);
    write_only_mine(&f.open.store, true).unwrap();
    let rank = |work: Option<&Store>| {
        let policy = TmSidePolicy::browse();
        rank_manage_listing(
            &f.open.scope,
            read_tier_rows(&f.global, work, policy).expect("doc tang"),
            TierFilter::Both,
            PairOriginFilter::All,
            "",
            ManageSort::Newest,
        )
        .expect("lap danh sach")
    };
    let open = rank(Some(&f.open.store));
    assert_eq!(
        open.total_pairs, 3,
        "danh sach van thay du cap khi cong tac bat"
    );
    assert_eq!(open.tier_totals.work, Some(2));
    assert_eq!(open.tier_totals.global, 1);
    let closed = rank(None);
    assert_eq!(closed.tier_totals.work, None);
    assert_eq!(closed.tier_totals.global, 1);
}
