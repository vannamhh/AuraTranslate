//! Vietnamese writing conventions (FR159, AD-67): the I/O matrix of the first story, rule 12
//! (a)(b)(c)(f), and the promise that a scan reads and never writes.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::convention::{
    ConventionMatcherState, ConventionScanWire, convention_active_rule, convention_add_rule,
    convention_list_rules, convention_scan_chapter,
};
use auratranslate_lib::commands::glossary::glossary_add_term;
use auratranslate_lib::commands::project::{OpenWork, OpenWorkState, create_work_from_text};
use auratranslate_lib::commands::segment::{
    ConventionWriteOutcome, convention_accept_range, convention_revert_range,
    read_open_chapter_segments,
};
use auratranslate_lib::core::convention::{Matcher, Rule, RuleTier, rule_key, transfer_case};
use auratranslate_lib::core::glossary::{Category, GlossaryTier};
use auratranslate_lib::core::i18n::IpcError;
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};
use unicode_normalization::UnicodeNormalization;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-convention-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn rule(avoid: &str, replacement: Option<&str>) -> Rule {
    Rule {
        key: rule_key(avoid),
        avoid_phrase: avoid.to_owned(),
        replacement_phrase: replacement.map(str::to_owned),
        tier: RuleTier::Global,
    }
}

fn spans(matcher: &Matcher, text: &str) -> Vec<(u32, u32)> {
    matcher
        .scan(text)
        .hits
        .iter()
        .map(|hit| (hit.start, hit.end))
        .collect()
}

fn utf16_slice(text: &str, start: u32, end: u32) -> String {
    let units: Vec<u16> = text.encode_utf16().collect();
    String::from_utf16(&units[start as usize..end as usize]).expect("whole chars")
}

#[test]
fn a_match_at_the_start_of_a_sentence_spans_the_phrase_and_keeps_the_capital() {
    let matcher = Matcher::build(vec![rule("đảm bảo", Some("bảo đảm"))]);
    let scan = matcher.scan("Đảm bảo an toàn.");
    assert_eq!(scan.hits.len(), 1);
    assert_eq!((scan.hits[0].start, scan.hits[0].end), (0, 7));
    assert_eq!(transfer_case("Đảm bảo", "bảo đảm"), "Bảo đảm");
}

#[test]
fn case_transfer_follows_the_matched_text() {
    assert_eq!(transfer_case("ĐẢM BẢO", "bảo đảm"), "BẢO ĐẢM");
    assert_eq!(transfer_case("đảm bảo", "Bảo đảm"), "bảo đảm");
    assert_eq!(transfer_case("đẢm Bảo", "bảo Đảm"), "bảo Đảm");
    assert_eq!(transfer_case("Ê", "ê"), "Ê");
}

#[test]
fn i_and_y_are_never_merged() {
    let matcher = Matcher::build(vec![rule("kỷ", Some("kỉ"))]);
    assert!(spans(&matcher, "kỉ luật").is_empty());
    assert_eq!(spans(&matcher, "kỷ luật"), vec![(0, 2)]);
}

#[test]
fn a_match_must_sit_on_word_edges() {
    let matcher = Matcher::build(vec![rule("an", Some("yên"))]);
    assert_eq!(spans(&matcher, "toàn an"), vec![(5, 7)]);
}

#[test]
fn nfd_text_gives_utf16_spans_on_the_stored_string() {
    let matcher = Matcher::build(vec![rule("đảm bảo", Some("bảo đảm"))]);
    let stored: String = "Ta Đảm bảo an toàn.".nfd().collect();
    let found = spans(&matcher, &stored);
    assert_eq!(found.len(), 1);
    let phrase = utf16_slice(&stored, found[0].0, found[0].1);
    assert_eq!(phrase.nfc().collect::<String>(), "Đảm bảo");
    assert!(phrase.chars().count() > "Đảm bảo".chars().count());
}

#[test]
fn an_nfd_rule_matches_nfc_text() {
    let avoid: String = "đảm bảo".nfd().collect();
    let matcher = Matcher::build(vec![rule(&avoid, None)]);
    assert_eq!(spans(&matcher, "đảm bảo"), vec![(0, 7)]);
}

#[test]
fn the_longer_rule_wins_an_overlap_and_the_loser_is_counted() {
    let matcher = Matcher::build(vec![
        rule("bảo đảm", Some("đảm bảo")),
        rule("bảo đảm bảo", Some("x")),
    ]);
    let scan = matcher.scan("bảo đảm bảo vệ");
    assert_eq!(scan.hits.len(), 1);
    assert_eq!((scan.hits[0].start, scan.hits[0].end), (0, 11));
    assert_eq!(scan.overlapped, 1);
}

#[test]
fn two_scans_of_the_same_input_are_equal() {
    let matcher = Matcher::build(vec![rule("đảm bảo", None), rule("an", None)]);
    let text = "Đảm bảo an toàn, đảm bảo an.";
    assert_eq!(matcher.scan(text), matcher.scan(text));
}

struct Fixture {
    global: Store,
    state: OpenWorkState,
    cache: ConventionMatcherState,
    segment_ids: Vec<i64>,
}

fn open_of(fixture: &Fixture) -> std::sync::MutexGuard<'_, Option<OpenWork>> {
    fixture.state.lock().expect("open work lock")
}

fn fixture(tag: &str) -> Fixture {
    let global = Store::open(StoreSpec::global(temp_dir(tag).join("global.db"))).expect("global");
    let open = create_work_from_text(
        &temp_dir(tag),
        "Convention",
        "en",
        "",
        "Assure safety. Second one. Third one.".to_owned(),
    )
    .expect("create work");
    let segment_ids: Vec<i64> = read_open_chapter_segments(Some(&open))
        .expect("segments")
        .segments
        .iter()
        .map(|s| s.id)
        .collect();
    assert_eq!(segment_ids.len(), 3);
    Fixture {
        global,
        state: Mutex::new(Some(open)),
        cache: ConventionMatcherState::default(),
        segment_ids,
    }
}

impl Fixture {
    fn seed(
        &self,
        index: usize,
        target: &str,
        baseline_origin: &str,
        stored_origin: &str,
        status: &str,
    ) {
        let id = self.segment_ids[index];
        let (target, baseline_origin, stored_origin, status) = (
            target.to_owned(),
            baseline_origin.to_owned(),
            stored_origin.to_owned(),
            status.to_owned(),
        );
        open_of(self)
            .as_ref()
            .expect("open")
            .store
            .write(move |tx: &Transaction<'_>| {
                tx.execute(
                    "UPDATE segment SET target_text = ?1, baseline_target_text = ?1, \
                     baseline_translation_origin = ?2, translation_origin = ?3, status = ?4 \
                     WHERE id = ?5",
                    (&target, &baseline_origin, &stored_origin, &status, id),
                )
            })
            .expect("seed");
    }

    fn add_rule(&self, avoid: &str, replacement: Option<&str>) {
        convention_add_rule(Some(&self.global), &self.cache, avoid, replacement).expect("add rule");
    }

    fn scan(&self) -> ConventionScanWire {
        convention_scan_chapter(Some(&self.global), Some(&self.state), &self.cache, 7)
            .expect("scan")
    }

    fn dump(&self) -> Vec<(i64, String, String, String, String)> {
        open_of(self)
            .as_ref()
            .expect("open")
            .store
            .read(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, target_text, baseline_target_text, translation_origin, status \
                     FROM segment ORDER BY id",
                )?;
                stmt.query_map([], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
                })?
                .collect::<Result<Vec<_>, _>>()
            })
            .expect("dump")
    }

    fn count(&self, sql: &'static str, arg: String) -> i64 {
        open_of(self)
            .as_ref()
            .expect("open")
            .store
            .read(move |conn| conn.query_row(sql, [arg], |r| r.get(0)))
            .expect("count")
    }

    fn versions(&self, index: usize) -> i64 {
        self.count(
            "SELECT COUNT(*) FROM segment_version WHERE segment_id = ?1",
            self.segment_ids[index].to_string(),
        )
    }

    fn pairs_with(&self, target: &str) -> i64 {
        self.count(
            "SELECT COUNT(*) FROM tm_unit WHERE target_text = ?1",
            target.to_owned(),
        )
    }

    fn accept(
        &self,
        index: usize,
        text: &str,
        start: u32,
        end: u32,
        phrase: &str,
        key: &str,
    ) -> Result<ConventionWriteOutcome, IpcError> {
        let rule = convention_active_rule(Some(&self.global), key)?;
        convention_accept_range(
            open_of(self).as_ref(),
            self.segment_ids[index],
            text,
            start,
            end,
            phrase,
            &rule,
        )
    }

    fn revert(
        &self,
        index: usize,
        text: &str,
        start: u32,
        end: u32,
        phrase: &str,
        original: &str,
    ) -> Result<ConventionWriteOutcome, IpcError> {
        convention_revert_range(
            open_of(self).as_ref(),
            self.segment_ids[index],
            text,
            start,
            end,
            phrase,
            original,
        )
    }
}

const KEY: &str = "đảm bảo";

#[test]
fn adding_a_rule_makes_the_open_chapter_scan_find_it() {
    let f = fixture("add-then-scan");
    f.seed(0, "Đảm bảo an toàn.", "other", "other", "draft");
    assert!(f.scan().segments.is_empty());
    f.add_rule("đảm bảo", Some("bảo đảm"));
    let scan = f.scan();
    assert_eq!(scan.generation, 7);
    assert_eq!(scan.segments.len(), 1);
    let span = &scan.segments[0].spans[0];
    assert_eq!((span.start, span.end), (0, 7));
    assert_eq!(span.replacement_cased.as_deref(), Some("Bảo đảm"));
    assert_eq!(span.rule_key, KEY);
    assert_eq!(scan.ignored, 0);
    let listed = convention_list_rules(Some(&f.global)).expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].replacement_phrase.as_deref(), Some("bảo đảm"));
}

#[test]
fn a_blank_or_repeated_avoid_phrase_is_refused_and_writes_nothing() {
    let f = fixture("add-refused");
    let blank = convention_add_rule(Some(&f.global), &f.cache, "  \u{3000} ", None)
        .expect_err("blank avoid phrase");
    assert_eq!(blank.code(), "convention.empty_avoid_phrase");
    f.add_rule("Đảm bảo", Some("bảo đảm"));
    let repeated =
        convention_add_rule(Some(&f.global), &f.cache, "đảm bảo", None).expect_err("same key");
    assert_eq!(repeated.code(), "convention.duplicate_avoid_phrase");
    assert_eq!(
        convention_list_rules(Some(&f.global)).expect("list").len(),
        1
    );
}

#[test]
fn omitted_and_empty_sentences_are_not_scanned() {
    let f = fixture("not-scanned");
    f.add_rule("đảm bảo", None);
    f.seed(0, "Đảm bảo.", "other", "other", "draft");
    f.seed(1, "Đảm bảo nữa.", "other", "other", "draft");
    let omitted = f.segment_ids[1];
    open_of(&f)
        .as_ref()
        .expect("open")
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("UPDATE segment SET is_omitted = 1 WHERE id = ?1", [omitted])
        })
        .expect("omit");
    let scan = f.scan();
    assert_eq!(scan.segments.len(), 1);
    assert_eq!(scan.segments[0].segment_id, f.segment_ids[0]);
    assert_eq!(scan.not_scanned, 1);
}

#[test]
fn a_rule_without_a_replacement_marks_but_cannot_be_accepted() {
    let f = fixture("no-replacement");
    f.add_rule("đảm bảo", None);
    f.seed(0, "Đảm bảo an toàn.", "other", "other", "draft");
    let scan = f.scan();
    assert_eq!(scan.segments[0].spans[0].replacement_cased, None);
    let before = f.dump();
    let refused = f
        .accept(0, "Đảm bảo an toàn.", 0, 7, "Đảm bảo", KEY)
        .expect_err("no replacement");
    assert_eq!(refused.code(), "convention.rule_unavailable");
    assert_eq!(f.dump(), before);
}

#[test]
fn a_scan_writes_nothing() {
    let f = fixture("scan-readonly");
    f.add_rule("đảm bảo", Some("bảo đảm"));
    f.seed(0, "Đảm bảo an toàn.", "other", "other", "draft");
    f.seed(1, "Hãy đảm bảo.", "self", "self", "confirmed");
    let before = f.dump();
    let first = f.scan();
    let second = f.scan();
    assert_eq!(first, second);
    assert_eq!(f.dump(), before);
    assert_eq!(f.versions(1), 0);
}

#[test]
fn accepting_on_an_ai_sentence_keeps_the_other_translated_origin() {
    let f = fixture("accept-ai");
    f.add_rule("đảm bảo", Some("bảo đảm"));
    f.seed(0, "Đảm bảo an toàn.", "other", "other", "draft");
    let done = f
        .accept(0, "Đảm bảo an toàn.", 0, 7, "Đảm bảo", KEY)
        .expect("accept");
    assert_eq!(done.target_text, "Bảo đảm an toàn.");
    assert_eq!(done.translation_origin, "other");
    assert_eq!(done.status, "draft");
    assert_eq!(
        (done.start, done.end, done.phrase.as_str()),
        (0, 7, "Bảo đảm")
    );
    let row = &f.dump()[0];
    assert_eq!(row.1, "Bảo đảm an toàn.");
    assert_eq!(row.2, "Bảo đảm an toàn.");
    assert_eq!(row.3, "other");
    assert_eq!(f.versions(0), 0);
    assert_eq!(f.pairs_with("Bảo đảm an toàn."), 0);
}

#[test]
fn accepting_on_a_sentence_the_user_typed_over_keeps_self() {
    let f = fixture("accept-self");
    f.add_rule("đảm bảo", Some("bảo đảm"));
    f.seed(0, "Đảm bảo an toàn.", "other", "other", "draft");
    open_of(&f)
        .as_ref()
        .expect("open")
        .store
        .write({
            let id = f.segment_ids[0];
            move |tx: &Transaction<'_>| {
                tx.execute(
                    "UPDATE segment SET baseline_target_text = 'khác hẳn' WHERE id = ?1",
                    [id],
                )
            }
        })
        .expect("diverge baseline");
    let done = f
        .accept(0, "Đảm bảo an toàn.", 0, 7, "Đảm bảo", KEY)
        .expect("accept");
    assert_eq!(done.translation_origin, "self");
}

#[test]
fn accepting_on_a_confirmed_sentence_keeps_it_confirmed_and_adds_one_version_and_one_pair() {
    let f = fixture("accept-confirmed");
    f.add_rule("đảm bảo", Some("bảo đảm"));
    f.seed(0, "Hãy đảm bảo an toàn.", "other", "other", "confirmed");
    let done = f
        .accept(0, "Hãy đảm bảo an toàn.", 4, 11, "đảm bảo", KEY)
        .expect("accept");
    assert_eq!(done.status, "confirmed");
    assert!(done.version_created);
    assert_eq!(f.dump()[0].4, "confirmed");
    assert_eq!(f.versions(0), 1);
    assert_eq!(f.pairs_with("Hãy bảo đảm an toàn."), 1);

    let undone = f
        .revert(0, "Hãy bảo đảm an toàn.", 4, 11, "bảo đảm", "đảm bảo")
        .expect("revert");
    assert_eq!(undone.target_text, "Hãy đảm bảo an toàn.");
    assert_eq!(f.dump()[0].1, "Hãy đảm bảo an toàn.");
    assert_eq!(f.dump()[0].4, "confirmed");
    assert_eq!(f.versions(0), 2);
    assert_eq!(f.pairs_with("Hãy bảo đảm an toàn."), 1);
    assert_eq!(f.pairs_with("Hãy đảm bảo an toàn."), 1);
}

#[test]
fn accepting_on_nfd_text_keeps_every_byte_outside_the_span() {
    let f = fixture("accept-nfd");
    f.add_rule("đảm bảo", Some("bảo đảm"));
    let stored: String = "Ta Đảm bảo an toàn.".nfd().collect();
    f.seed(0, &stored, "other", "other", "draft");
    let found = f.scan();
    let span = &found.segments[0].spans[0];
    let phrase = utf16_slice(&stored, span.start, span.end);
    let done = f
        .accept(0, &stored, span.start, span.end, &phrase, KEY)
        .expect("accept");
    let tail = utf16_slice(
        &stored,
        span.end,
        u32::try_from(stored.encode_utf16().count()).unwrap(),
    );
    assert_eq!(done.target_text, format!("Ta Bảo đảm{tail}"));
    assert!(done.target_text.ends_with(&tail));
}

#[test]
fn a_changed_text_or_range_is_refused_and_writes_nothing() {
    let f = fixture("accept-stale");
    f.add_rule("đảm bảo", Some("bảo đảm"));
    f.seed(0, "Đảm bảo an toàn.", "other", "other", "draft");
    let before = f.dump();
    let changed = f
        .accept(0, "Đảm bảo an toàn!", 0, 7, "Đảm bảo", KEY)
        .expect_err("text changed");
    assert_eq!(changed.code(), "convention.text_changed");
    let wrong_phrase = f
        .accept(0, "Đảm bảo an toàn.", 1, 8, "ảm bảo ", KEY)
        .expect_err("phrase is not the rule's");
    assert!(
        ["convention.rule_unavailable", "convention.text_changed"].contains(&wrong_phrase.code())
    );
    let outside = f
        .accept(0, "Đảm bảo an toàn.", 0, 99, "Đảm bảo", KEY)
        .expect_err("range outside");
    assert_eq!(outside.code(), "convention.range_invalid");
    assert_eq!(f.dump(), before);
}

#[test]
fn a_rule_disabled_after_the_scan_is_refused_and_writes_nothing() {
    let f = fixture("accept-rule-gone");
    f.add_rule("đảm bảo", Some("bảo đảm"));
    f.seed(0, "Đảm bảo an toàn.", "other", "other", "draft");
    f.global
        .write(|tx: &Transaction<'_>| tx.execute("UPDATE convention_rule SET enabled = 0", []))
        .expect("disable");
    let before = f.dump();
    let refused = f
        .accept(0, "Đảm bảo an toàn.", 0, 7, "Đảm bảo", KEY)
        .expect_err("rule disabled");
    assert_eq!(refused.code(), "convention.rule_unavailable");
    assert_eq!(f.dump(), before);
}

#[test]
fn a_write_that_would_leave_the_sentence_blank_is_refused() {
    let f = fixture("accept-blank");
    f.seed(0, "bảo đảm", "other", "other", "draft");
    let before = f.dump();
    let refused = f
        .revert(0, "bảo đảm", 0, 7, "bảo đảm", "  ")
        .expect_err("blank result");
    assert_eq!(refused.code(), "convention.text_would_be_empty");
    assert_eq!(f.dump(), before);
}

#[test]
fn a_match_inside_a_pinned_glossary_translation_gets_no_mark() {
    let f = fixture("glossary-wins");
    f.add_rule("đảm bảo", Some("bảo đảm"));
    f.seed(0, "Đội đảm bảo.", "other", "other", "draft");
    f.seed(1, "Hãy đảm bảo.", "other", "other", "draft");
    {
        let guard = open_of(&f);
        glossary_add_term(
            Some(&f.global),
            guard.as_ref(),
            GlossaryTier::Global,
            "Assure",
            Some("đội đảm bảo"),
            "",
            Category::Other,
        )
        .expect("pin term");
    }
    let scan = f.scan();
    assert_eq!(scan.glossary_suppressed, 1);
    assert_eq!(scan.segments.len(), 1);
    assert_eq!(scan.segments[0].segment_id, f.segment_ids[1]);
}
