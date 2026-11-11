//! IPC for Vietnamese writing conventions (FR159, AD-67): the Global rule list and the scan of
//! the open Chapter. The scan reads stored text only and writes nothing.
//!
//! Strings in this file are written without diacritics (`scripts/check-i18n.mjs`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::commands::project::{OpenWork, OpenWorkState};
use crate::commands::segment::select_chapter_segments;
use crate::core::convention::{
    ConflictDecision, ConventionSignature, ConventionStoreError, MAX_CONVENTION_IMPORT_BYTES,
    Matcher, RowPlan, RowPlanKind, RuleRow, RuleStatus, RuleTier, add_ignore, add_rule,
    apply_import, classify_import, clear_ignores, delete_rule, disable_in_work, edit_rule,
    effective_row, enable_in_work, filter_ignored, find_row_by_key, list_ignore_rows, list_ignores,
    list_rules, parse_import, phrase_spans, remove_ignore, render_tier, resolve_two_tiers,
    row_status, rule_key, signature_of, transfer_case, visible_rows,
};
use crate::core::delimited_text::{Delimiter, read_text_file, write_text_file_atomic};
use crate::core::glossary::{GlossarySnapshot, match_lang_for_source_lang};
use crate::core::i18n::IpcError;
use crate::core::segment::utf16::utf16_to_byte;
use crate::core::store::{SqlError, SqlResult, Store, StoreError, StoreKind};

pub(crate) fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

/// The matcher built from the rules in force for one open Work, kept until the rule list changes
/// or another Work opens.
#[derive(Default)]
pub struct ConventionMatcherState(Mutex<Option<(Option<PathBuf>, Arc<Matcher>)>>);

fn work_key(open: Option<&OpenWork>) -> Option<PathBuf> {
    open.map(|open| open.dir.clone())
}

fn work_rows(open: Option<&OpenWork>) -> Result<Vec<RuleRow>, IpcError> {
    Ok(match open {
        Some(open) => list_rules(&open.store)?,
        None => Vec::new(),
    })
}

impl ConventionMatcherState {
    fn invalidate(&self) {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    fn matcher(&self, global: &Store, open: Option<&OpenWork>) -> Result<Arc<Matcher>, IpcError> {
        let key = work_key(open);
        let mut slot = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((cached_key, matcher)) = slot.as_ref()
            && *cached_key == key
        {
            return Ok(Arc::clone(matcher));
        }
        let rules = resolve_two_tiers(&list_rules(global)?, &work_rows(open)?);
        let matcher = Arc::new(Matcher::build(rules));
        *slot = Some((key, Arc::clone(&matcher)));
        Ok(matcher)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConventionRuleWire {
    pub id: i64,
    pub rule_key: String,
    pub avoid_phrase: String,
    pub replacement_phrase: Option<String>,
    pub enabled: bool,
    pub tier: String,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConventionTierFilter {
    Global,
    Work,
    Both,
}

fn rule_wire(row: RuleRow, tier: RuleTier, status: RuleStatus) -> ConventionRuleWire {
    ConventionRuleWire {
        id: row.id,
        rule_key: row.rule_key,
        avoid_phrase: row.avoid_phrase,
        replacement_phrase: row.replacement_phrase,
        enabled: row.enabled,
        tier: tier.as_str().to_owned(),
        status: status.as_str().to_owned(),
    }
}

fn store_for_tier<'a>(
    tier: RuleTier,
    global: Option<&'a Store>,
    open: Option<&'a OpenWork>,
) -> Result<&'a Store, IpcError> {
    match tier {
        RuleTier::Global => global.ok_or_else(store_is_missing),
        RuleTier::Work => open
            .map(|open| &open.store)
            .ok_or_else(|| ConventionStoreError::WorkTierUnavailable.into()),
    }
}

/// The rules of `tier` with their status against the other tier. `Both` lists the Global rows,
/// then the Work rows when a Work is open.
pub fn convention_list_rules(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: ConventionTierFilter,
) -> Result<Vec<ConventionRuleWire>, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    if tier == ConventionTierFilter::Work && open.is_none() {
        return Err(ConventionStoreError::WorkTierUnavailable.into());
    }
    let global_rows = list_rules(global)?;
    let work = work_rows(open)?;
    let mut out = Vec::new();
    if tier != ConventionTierFilter::Work {
        for row in &global_rows {
            let status = row_status(RuleTier::Global, row, &work);
            out.push(rule_wire(row.clone(), RuleTier::Global, status));
        }
    }
    if tier != ConventionTierFilter::Global {
        for row in work.into_iter().filter(|row| row.enabled) {
            let status = row_status(RuleTier::Work, &row, &[]);
            out.push(rule_wire(row, RuleTier::Work, status));
        }
    }
    Ok(out)
}

/// The enabled rule in force for `rule_key` across both tiers; a rule that is gone or switched
/// off refuses.
pub fn convention_active_rule(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    rule_key: &str,
) -> Result<RuleRow, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    effective_row(rule_key, &list_rules(global)?, &work_rows(open)?)
        .ok_or_else(crate::commands::segment::rule_unavailable)
}

/// Adds a rule to `tier` and drops the cached matcher. A blank avoid phrase or a key already
/// present in that tier is refused with nothing written.
pub fn convention_add_rule(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    cache: &ConventionMatcherState,
    tier: RuleTier,
    avoid_phrase: &str,
    replacement_phrase: Option<&str>,
) -> Result<i64, IpcError> {
    let store = store_for_tier(tier, global, open)?;
    let id = add_rule(store, avoid_phrase, replacement_phrase)?;
    cache.invalidate();
    Ok(id)
}

pub fn convention_edit_rule(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    cache: &ConventionMatcherState,
    tier: RuleTier,
    id: i64,
    avoid_phrase: &str,
    replacement_phrase: Option<&str>,
) -> Result<(), IpcError> {
    let store = store_for_tier(tier, global, open)?;
    edit_rule(store, id, avoid_phrase, replacement_phrase)?;
    cache.invalidate();
    Ok(())
}

pub fn convention_delete_rule(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    cache: &ConventionMatcherState,
    tier: RuleTier,
    id: i64,
) -> Result<(), IpcError> {
    let store = store_for_tier(tier, global, open)?;
    delete_rule(store, id)?;
    cache.invalidate();
    Ok(())
}

/// Switches the Global rule `rule_key` off (`enabled = false`) or back on for the open Work only.
/// Off writes a Work-tier row with the same key and `enabled = 0`; on deletes that row.
pub fn convention_set_enabled(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    cache: &ConventionMatcherState,
    rule_key: &str,
    enabled: bool,
) -> Result<(), IpcError> {
    let work = store_for_tier(RuleTier::Work, global, open)?;
    if enabled {
        enable_in_work(work, rule_key)?;
    } else {
        let global = global.ok_or_else(store_is_missing)?;
        let row = find_row_by_key(global, rule_key)?.ok_or(ConventionStoreError::RuleMissing)?;
        disable_in_work(work, &row.avoid_phrase, &row.rule_key)?;
    }
    cache.invalidate();
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConventionSpanWire {
    pub start: u32,
    pub end: u32,
    pub rule_key: String,
    pub avoid: String,
    pub replacement_cased: Option<String>,
    pub tier: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConventionSegmentWire {
    pub segment_id: i64,
    pub scanned_text: String,
    pub spans: Vec<ConventionSpanWire>,
}

/// Matches of one scan of the open Chapter. `ignored` counts the matches the "not an error" memory
/// silenced; `not_scanned` counts omitted sentences.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConventionScanWire {
    pub generation: u64,
    pub chapter_id: i64,
    pub segments: Vec<ConventionSegmentWire>,
    pub ignored: u32,
    pub glossary_suppressed: u32,
    pub overlapped: u32,
    pub not_scanned: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct ConventionScanProgressWire {
    pub done: u32,
    pub total: u32,
}

const SCAN_PROGRESS_EVERY: u32 = 250;

struct SentenceInput {
    segment_id: i64,
    source_text: String,
    target_text: String,
}

struct ScanInput {
    chapter_id: i64,
    source_lang: String,
    sentences: Vec<SentenceInput>,
    not_scanned: u32,
    ignored: Vec<ConventionSignature>,
}

/// One read of the open Chapter, copied out so the caller can drop the `OpenWorkState` lock before
/// matching. With `only` set, just those live sentences of the open Chapter are read; an id that is
/// unknown, retired or in another Chapter is skipped.
fn read_scan_input(open: Option<&OpenWork>, only: Option<&[i64]>) -> Result<ScanInput, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let chapter_id = open.chapter_id;
    let rows: Vec<(i64, String, String, bool)> = open.store.read(|conn| match only {
        None => select_chapter_segments(conn, chapter_id).map(|rows| {
            rows.into_iter()
                .map(|row| (row.id, row.source_text, row.target_text, row.is_omitted))
                .collect()
        }),
        Some(ids) => select_segments_by_id(conn, chapter_id, ids),
    })?;
    let mut not_scanned = 0u32;
    let mut sentences = Vec::new();
    for (id, source_text, target_text, is_omitted) in rows {
        if is_omitted {
            not_scanned += 1;
        } else if !target_text.trim().is_empty() {
            sentences.push(SentenceInput {
                segment_id: id,
                source_text,
                target_text,
            });
        }
    }
    Ok(ScanInput {
        chapter_id,
        source_lang: open.meta.source_lang.clone(),
        sentences,
        not_scanned,
        ignored: list_ignores(&open.store)?,
    })
}

fn select_segments_by_id(
    conn: crate::core::store::ReadHandle<'_>,
    chapter_id: i64,
    ids: &[i64],
) -> SqlResult<Vec<(i64, String, String, bool)>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_text, target_text, is_omitted FROM segment \
         WHERE id = ?1 AND chapter_id = ?2 AND retired_at IS NULL",
    )?;
    let mut seen = std::collections::HashSet::new();
    let mut rows = Vec::new();
    for &id in ids {
        if !seen.insert(id) {
            continue;
        }
        let found = stmt.query_row([id, chapter_id], |row| {
            let omitted: i64 = row.get(3)?;
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, omitted != 0))
        });
        match found {
            Ok(row) => rows.push(row),
            Err(SqlError::QueryReturnedNoRows) => {}
            Err(err) => return Err(err),
        }
    }
    Ok(rows)
}

fn load_glossary(
    global: &Store,
    open: Option<&OpenWork>,
    chapter_id: i64,
) -> Result<Option<GlossarySnapshot>, IpcError> {
    let Some(open) = open.filter(|open| open.chapter_id == chapter_id) else {
        return Ok(None);
    };
    Ok(Some(GlossarySnapshot::load(
        &open.scope,
        global,
        Some(&open.store),
    )?))
}

/// Turns the raw hits of one sentence into wire spans, dropping and counting the ones that lie
/// inside a pinned Glossary translation of that sentence (AD-67 rule 5), then the ones the
/// "not an error" memory silences (rule 9). Returns the spans, the Glossary count and the memory
/// count. The one place all three scans filter.
fn resolve_spans(
    matcher: &Matcher,
    lang: crate::core::matching::MatchLang,
    glossary: Option<&GlossarySnapshot>,
    source_text: &str,
    text: &str,
    hits: Vec<crate::core::convention::Hit>,
    ignored: &[ConventionSignature],
) -> (Vec<ConventionSpanWire>, u32, u32) {
    let pinned: Vec<(u32, u32)> = glossary
        .map(|glossary| glossary.confirmed_terms(source_text, lang).injected)
        .unwrap_or_default()
        .iter()
        .flat_map(|term| phrase_spans(text, &term.translation))
        .collect();
    let mut spans = Vec::new();
    let mut suppressed = 0u32;
    let mut unpinned = Vec::with_capacity(hits.len());
    for hit in hits {
        if pinned.iter().any(|&(s, e)| s <= hit.start && hit.end <= e) {
            suppressed += 1;
            continue;
        }
        unpinned.push(hit);
    }
    let (remaining, silenced) = filter_ignored(text, unpinned, ignored, |hit| {
        (hit.start, hit.end, matcher.rule(hit.rule).key.as_str())
    });
    for hit in remaining {
        let rule = matcher.rule(hit.rule);
        let matched = utf16_to_byte(text, hit.start)
            .zip(utf16_to_byte(text, hit.end))
            .map(|(a, b)| &text[a..b])
            .unwrap_or_default();
        spans.push(ConventionSpanWire {
            start: hit.start,
            end: hit.end,
            rule_key: rule.key.clone(),
            avoid: rule.avoid_phrase.clone(),
            replacement_cased: rule
                .replacement_phrase
                .as_deref()
                .map(|replacement| transfer_case(matched, replacement)),
            tier: rule.tier.as_str().to_owned(),
        });
    }
    (spans, suppressed, silenced)
}

/// Matches `input` against `matcher`. A match that lies inside a pinned Glossary translation found
/// in the same sentence is dropped and counted (AD-67 rule 5); the snapshot is requested only once a
/// sentence has a match.
fn match_sentences(
    generation: u64,
    mut input: ScanInput,
    matcher: &Matcher,
    mut snapshot: impl FnMut() -> Result<Option<GlossarySnapshot>, IpcError>,
    mut progress: impl FnMut(u32, u32),
) -> Result<ConventionScanWire, IpcError> {
    let total = u32::try_from(input.sentences.len()).unwrap_or(u32::MAX);
    let lang = match_lang_for_source_lang(&input.source_lang);
    let mut loaded: Option<Option<GlossarySnapshot>> = None;
    let mut segments = Vec::new();
    let (mut overlapped, mut suppressed, mut ignored) = (0u32, 0u32, 0u32);
    let remembered = std::mem::take(&mut input.ignored);
    for (index, sentence) in input.sentences.into_iter().enumerate() {
        let done = u32::try_from(index).unwrap_or(u32::MAX);
        if done > 0 && done % SCAN_PROGRESS_EVERY == 0 {
            progress(done, total);
        }
        let scan = matcher.scan(&sentence.target_text);
        overlapped += scan.overlapped;
        if scan.hits.is_empty() {
            continue;
        }
        if loaded.is_none() {
            loaded = Some(snapshot()?);
        }
        let (spans, dropped, silenced) = resolve_spans(
            matcher,
            lang,
            loaded.as_ref().and_then(Option::as_ref),
            &sentence.source_text,
            &sentence.target_text,
            scan.hits,
            &remembered,
        );
        suppressed += dropped;
        ignored += silenced;
        if !spans.is_empty() {
            segments.push(ConventionSegmentWire {
                segment_id: sentence.segment_id,
                scanned_text: sentence.target_text,
                spans,
            });
        }
    }
    progress(total, total);
    Ok(ConventionScanWire {
        generation,
        chapter_id: input.chapter_id,
        segments,
        ignored,
        glossary_suppressed: suppressed,
        overlapped,
        not_scanned: input.not_scanned,
    })
}

/// The scan with every input given: the open Work is read once, then matched without it.
pub fn convention_scan_chapter(
    global: Option<&Store>,
    open_state: Option<&OpenWorkState>,
    cache: &ConventionMatcherState,
    generation: u64,
) -> Result<ConventionScanWire, IpcError> {
    convention_scan_chapter_with_progress(global, open_state, cache, generation, |_, _| {})
}

/// [`convention_scan_chapter`] reporting `(done, total)` sentences every 250 sentences and once at
/// the end (`done == total`, also for a Chapter with nothing to scan).
pub fn convention_scan_chapter_with_progress(
    global: Option<&Store>,
    open_state: Option<&OpenWorkState>,
    cache: &ConventionMatcherState,
    generation: u64,
    progress: impl FnMut(u32, u32),
) -> Result<ConventionScanWire, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let lock = || {
        open_state.map(|state| {
            state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        })
    };
    let (input, matcher) = {
        let guard = lock();
        let open = guard.as_deref().and_then(Option::as_ref);
        let input = read_scan_input(open, None)?;
        let matcher = cache.matcher(global, open)?;
        (input, matcher)
    };
    let chapter_id = input.chapter_id;
    match_sentences(
        generation,
        input,
        &matcher,
        || {
            let guard = lock();
            load_glossary(
                global,
                guard.as_deref().and_then(Option::as_ref),
                chapter_id,
            )
        },
        progress,
    )
}

/// Re-scan of just `segment_ids`, read from disk like the whole-Chapter scan and returning the same
/// wire. Ids that are unknown, retired or outside the open Chapter are skipped without an error.
pub fn convention_scan_segments(
    global: Option<&Store>,
    open_state: Option<&OpenWorkState>,
    cache: &ConventionMatcherState,
    generation: u64,
    segment_ids: &[i64],
) -> Result<ConventionScanWire, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let lock = || {
        open_state.map(|state| {
            state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        })
    };
    let (input, matcher) = {
        let guard = lock();
        let open = guard.as_deref().and_then(Option::as_ref);
        let input = read_scan_input(open, Some(segment_ids))?;
        let matcher = cache.matcher(global, open)?;
        (input, matcher)
    };
    let chapter_id = input.chapter_id;
    match_sentences(
        generation,
        input,
        &matcher,
        || {
            let guard = lock();
            load_glossary(
                global,
                guard.as_deref().and_then(Option::as_ref),
                chapter_id,
            )
        },
        |_, _| {},
    )
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ConventionSuggestionItem {
    pub segment_id: i64,
    pub text: String,
}

/// Matches of one batch of AI suggestions. `chapter_id` is `None` when no Work is open, which is a
/// valid empty answer rather than an error. `items` holds only the suggestions that have a mark,
/// each carrying the text that was scanned.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConventionSuggestionScanWire {
    pub generation: u64,
    pub chapter_id: Option<i64>,
    pub items: Vec<ConventionSegmentWire>,
    pub ignored: u32,
    pub glossary_suppressed: u32,
    pub overlapped: u32,
}

/// Scan of suggestion texts. The text comes from the caller (a suggestion exists nowhere on disk);
/// the source sentence is read from disk by `segment_id`, only for suggestions that have a match,
/// and one Glossary snapshot serves the whole batch. Writes nothing.
pub fn convention_scan_suggestions(
    global: Option<&Store>,
    open_state: Option<&OpenWorkState>,
    cache: &ConventionMatcherState,
    generation: u64,
    items: Vec<ConventionSuggestionItem>,
) -> Result<ConventionSuggestionScanWire, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let lock = || {
        open_state.map(|state| {
            state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        })
    };
    let (chapter_id, source_lang, matcher, remembered) = {
        let guard = lock();
        let open = guard.as_deref().and_then(Option::as_ref);
        let Some(open) = open else {
            return Ok(ConventionSuggestionScanWire {
                generation,
                chapter_id: None,
                items: Vec::new(),
                ignored: 0,
                glossary_suppressed: 0,
                overlapped: 0,
            });
        };
        (
            open.chapter_id,
            open.meta.source_lang.clone(),
            cache.matcher(global, Some(open))?,
            list_ignores(&open.store)?,
        )
    };
    let lang = match_lang_for_source_lang(&source_lang);
    let mut overlapped = 0u32;
    let mut matched = Vec::new();
    for item in items {
        if item.text.trim().is_empty() {
            continue;
        }
        let scan = matcher.scan(&item.text);
        overlapped += scan.overlapped;
        if !scan.hits.is_empty() {
            matched.push((item, scan.hits));
        }
    }
    let mut out = Vec::new();
    let (mut suppressed, mut ignored) = (0u32, 0u32);
    if !matched.is_empty() {
        let ids: Vec<i64> = matched.iter().map(|(item, _)| item.segment_id).collect();
        let (sources, glossary) = {
            let guard = lock();
            let open = guard.as_deref().and_then(Option::as_ref);
            let sources: std::collections::HashMap<i64, String> = match open {
                Some(open) if open.chapter_id == chapter_id => open
                    .store
                    .read(|conn| select_segments_by_id(conn, chapter_id, &ids))?
                    .into_iter()
                    .map(|(id, source, _, _)| (id, source))
                    .collect(),
                _ => std::collections::HashMap::new(),
            };
            (sources, load_glossary(global, open, chapter_id)?)
        };
        for (item, hits) in matched {
            let Some(source_text) = sources.get(&item.segment_id) else {
                continue;
            };
            let (spans, dropped, silenced) = resolve_spans(
                &matcher,
                lang,
                glossary.as_ref(),
                source_text,
                &item.text,
                hits,
                &remembered,
            );
            suppressed += dropped;
            ignored += silenced;
            if !spans.is_empty() {
                out.push(ConventionSegmentWire {
                    segment_id: item.segment_id,
                    scanned_text: item.text,
                    spans,
                });
            }
        }
    }
    Ok(ConventionSuggestionScanWire {
        generation,
        chapter_id: Some(chapter_id),
        items: out,
        ignored,
        glossary_suppressed: suppressed,
        overlapped,
    })
}

/// One remembered "not an error" place, as the management card draws it
/// (`...{context_before}[{avoid_phrase}]{context_after}...`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConventionIgnoreWire {
    pub id: i64,
    pub rule_key: String,
    pub avoid_phrase: String,
    pub context_before: String,
    pub context_after: String,
    pub created_at: String,
}

fn work_store(open: Option<&OpenWork>) -> Result<&Store, IpcError> {
    store_for_tier(RuleTier::Work, None, open)
}

/// Remembers the match `[start, end)` (UTF-16) of `segment_id` as "not an error" for the whole Work
/// (AD-67 rule 9). The range goes through `check_proofread_range` (AD-53 rule 4), the signature
/// comes from [`signature_of`], and remembering the same place twice is one row. Writes no text.
pub fn convention_ignore_add(
    open: Option<&OpenWork>,
    segment_id: i64,
    expected_text: &str,
    start: u32,
    end: u32,
    expected_phrase: &str,
    key: &str,
) -> Result<(), IpcError> {
    let store = work_store(open)?;
    let open_chapter_id = open.map(|open| open.chapter_id).unwrap_or_default();
    if rule_key(expected_phrase) != key {
        return Err(crate::commands::segment::rule_unavailable());
    }
    let (expected_text, expected_phrase, key) = (
        expected_text.to_owned(),
        expected_phrase.to_owned(),
        key.to_owned(),
    );
    store.write(move |tx| {
        let checked = match crate::commands::segment::check_proofread_range(
            tx,
            open_chapter_id,
            segment_id,
            &expected_text,
            start,
            end,
            &expected_phrase,
        )? {
            Ok(checked) => checked,
            Err(err) => {
                return Ok(Err(crate::commands::segment::as_convention_error(
                    err, segment_id,
                )));
            }
        };
        let Some(signature) = signature_of(&checked.text, start, end, &key) else {
            return Ok(Err(crate::commands::segment::as_convention_error(
                crate::commands::segment::range_invalid(segment_id),
                segment_id,
            )));
        };
        add_ignore(tx, &signature)?;
        Ok(Ok(()))
    })?
}

/// Every remembered place of the open Work, newest first.
pub fn convention_ignore_list(
    open: Option<&OpenWork>,
) -> Result<Vec<ConventionIgnoreWire>, IpcError> {
    Ok(list_ignore_rows(work_store(open)?)?
        .into_iter()
        .map(|row| ConventionIgnoreWire {
            id: row.id,
            rule_key: row.rule_key,
            avoid_phrase: row.avoid_phrase,
            context_before: row.context_before,
            context_after: row.context_after,
            created_at: row.created_at,
        })
        .collect())
}

/// Forgets one remembered place; `false` when the row is already gone.
pub fn convention_ignore_remove(open: Option<&OpenWork>, id: i64) -> Result<bool, IpcError> {
    Ok(remove_ignore(work_store(open)?, id)?)
}

/// Forgets every remembered place of the open Work and returns how many rows went.
pub fn convention_ignore_clear(open: Option<&OpenWork>) -> Result<u32, IpcError> {
    Ok(clear_ignores(work_store(open)?)?)
}

/// The plan of an import that has been previewed and not yet confirmed. The file content never
/// reaches the webview; only counts and the disagreements do.
#[derive(Debug)]
pub struct ConventionPendingImport {
    pub tier: RuleTier,
    pub plans: Vec<RowPlan>,
}

pub type ConventionPendingImportState = Mutex<Option<ConventionPendingImport>>;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConventionImportConflictWire {
    pub rule_key: String,
    pub existing_avoid_phrase: String,
    pub existing_replacement_phrase: Option<String>,
    pub file_avoid_phrase: String,
    pub file_replacement_phrase: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConventionImportPreviewWire {
    pub file_name: String,
    pub tier: String,
    pub row_count: usize,
    pub new_count: usize,
    pub identical_count: usize,
    pub conflicts: Vec<ConventionImportConflictWire>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct ConventionImportSummaryWire {
    pub added: u32,
    pub replaced: u32,
    pub kept: u32,
}

fn delimiter_from_path(path: &Path) -> Delimiter {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("tsv") => Delimiter::Tsv,
        _ => Delimiter::Csv,
    }
}

/// Writes the rules of `tier` (a Work's disabled-Global flags excluded) to `path`; the delimiter
/// follows the extension (`.tsv` is tab, anything else comma).
pub fn convention_export_tier(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: RuleTier,
    path: &Path,
) -> Result<(), IpcError> {
    let store = store_for_tier(tier, global, open)?;
    let rows = visible_rows(tier, list_rules(store)?);
    let contents = render_tier(&rows, delimiter_from_path(path));
    write_text_file_atomic(path, &contents).map_err(ConventionStoreError::from)?;
    Ok(())
}

/// A cancelled dialog (`None`) is `Ok(None)`: nothing is read or written and no error is shown.
pub fn convention_export_tier_after_dialog(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: RuleTier,
    picked_path: Option<PathBuf>,
) -> Result<Option<String>, IpcError> {
    let Some(path) = picked_path else {
        return Ok(None);
    };
    convention_export_tier(global, open, tier, &path)?;
    Ok(Some(path.display().to_string()))
}

fn pending_slot(
    pending: &ConventionPendingImportState,
) -> std::sync::MutexGuard<'_, Option<ConventionPendingImport>> {
    pending
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Reads, parses and classifies `path` against `tier` and keeps the plan. A file with any error
/// is refused whole and no plan is kept. A new preview replaces a pending one.
pub fn convention_open_import_preview(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    pending: &ConventionPendingImportState,
    tier: RuleTier,
    path: &Path,
) -> Result<ConventionImportPreviewWire, IpcError> {
    let store = store_for_tier(tier, global, open)?;
    let text =
        read_text_file(path, MAX_CONVENTION_IMPORT_BYTES).map_err(ConventionStoreError::from)?;
    let rows = parse_import(&text)?;
    let plans = classify_import(&rows, &visible_rows(tier, list_rules(store)?));

    let count = |wanted: fn(&RowPlanKind) -> bool| plans.iter().filter(|p| wanted(&p.kind)).count();
    let conflicts = plans
        .iter()
        .filter_map(|plan| match &plan.kind {
            RowPlanKind::Conflict {
                existing_avoid_phrase,
                existing_replacement_phrase,
            } => Some(ConventionImportConflictWire {
                rule_key: plan.rule_key.clone(),
                existing_avoid_phrase: existing_avoid_phrase.clone(),
                existing_replacement_phrase: existing_replacement_phrase.clone(),
                file_avoid_phrase: plan.avoid_phrase.clone(),
                file_replacement_phrase: plan.replacement_phrase.clone(),
            }),
            _ => None,
        })
        .collect();
    let preview = ConventionImportPreviewWire {
        file_name: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        tier: tier.as_str().to_owned(),
        row_count: plans.len(),
        new_count: count(|kind| matches!(kind, RowPlanKind::New)),
        identical_count: count(|kind| matches!(kind, RowPlanKind::Identical)),
        conflicts,
    };

    *pending_slot(pending) = Some(ConventionPendingImport { tier, plans });
    Ok(preview)
}

/// A cancelled dialog (`None`) is `Ok(None)`: nothing is read and no plan is kept or replaced.
pub fn convention_open_import_preview_after_dialog(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    pending: &ConventionPendingImportState,
    tier: RuleTier,
    picked_path: Option<PathBuf>,
) -> Result<Option<ConventionImportPreviewWire>, IpcError> {
    let Some(path) = picked_path else {
        return Ok(None);
    };
    convention_open_import_preview(global, open, pending, tier, &path).map(Some)
}

/// Writes the pending plan in one transaction and drops the cached matcher. `decisions` maps a
/// `rule_key` to what to do with that disagreement; a missing key keeps the existing rule. Any
/// failure leaves the plan pending and writes nothing.
pub fn convention_confirm_import(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    cache: &ConventionMatcherState,
    pending: &ConventionPendingImportState,
    decisions: &BTreeMap<String, ConflictDecision>,
) -> Result<ConventionImportSummaryWire, IpcError> {
    let mut slot = pending_slot(pending);
    let Some(batch) = slot.as_ref() else {
        return Err(ConventionStoreError::NoPendingImport.into());
    };
    let store = store_for_tier(batch.tier, global, open)?;
    let summary = apply_import(store, batch.tier, &batch.plans, decisions)?;
    *slot = None;
    cache.invalidate();
    Ok(ConventionImportSummaryWire {
        added: summary.added,
        replaced: summary.replaced,
        kept: summary.kept,
    })
}

pub fn convention_cancel_import(pending: &ConventionPendingImportState) {
    *pending_slot(pending) = None;
}

/// Drops a pending Work-tier plan when the Work it was classified against goes away.
pub fn clear_pending_convention_import_for_work(pending: &ConventionPendingImportState) {
    let mut slot = pending_slot(pending);
    if slot
        .as_ref()
        .is_some_and(|batch| batch.tier == RuleTier::Work)
    {
        *slot = None;
    }
}

fn state_missing() -> IpcError {
    crate::core::i18n::state_missing(
        "convention",
        "ConventionMatcherState",
        "convention.state_missing",
    )
}

pub mod wire {
    use std::collections::BTreeMap;

    use super::{
        ConventionIgnoreWire, ConventionImportPreviewWire, ConventionImportSummaryWire,
        ConventionMatcherState, ConventionPendingImportState, ConventionRuleWire,
        ConventionScanProgressWire, ConventionScanWire, ConventionSuggestionItem,
        ConventionSuggestionScanWire, ConventionTierFilter, state_missing,
    };
    use crate::commands::project::OpenWorkState;
    use crate::core::convention::{ConflictDecision, ConventionStoreError, RuleTier};
    use crate::core::i18n::IpcError;
    use crate::core::store::Store;
    use tauri_plugin_dialog::DialogExt as _;

    #[tauri::command]
    pub fn convention_list_rules<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        tier: ConventionTierFilter,
    ) -> Result<Vec<ConventionRuleWire>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::convention_list_rules(global.as_deref(), None, tier);
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::convention_list_rules(global.as_deref(), guard.as_ref(), tier)
    }

    #[tauri::command]
    pub fn convention_add_rule<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        tier: RuleTier,
        avoid_phrase: String,
        replacement_phrase: Option<String>,
    ) -> Result<i64, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(cache) = app.try_state::<ConventionMatcherState>() else {
            return Err(state_missing());
        };
        let work_state = app.try_state::<OpenWorkState>();
        let guard = work_state.as_deref().map(|state| {
            state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        super::convention_add_rule(
            global.as_deref(),
            guard.as_deref().and_then(Option::as_ref),
            &cache,
            tier,
            &avoid_phrase,
            replacement_phrase.as_deref(),
        )
    }

    #[tauri::command]
    pub fn convention_edit_rule<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        tier: RuleTier,
        id: i64,
        avoid_phrase: String,
        replacement_phrase: Option<String>,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(cache) = app.try_state::<ConventionMatcherState>() else {
            return Err(state_missing());
        };
        let work_state = app.try_state::<OpenWorkState>();
        let guard = work_state.as_deref().map(|state| {
            state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        super::convention_edit_rule(
            global.as_deref(),
            guard.as_deref().and_then(Option::as_ref),
            &cache,
            tier,
            id,
            &avoid_phrase,
            replacement_phrase.as_deref(),
        )
    }

    #[tauri::command]
    pub fn convention_delete_rule<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        tier: RuleTier,
        id: i64,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(cache) = app.try_state::<ConventionMatcherState>() else {
            return Err(state_missing());
        };
        let work_state = app.try_state::<OpenWorkState>();
        let guard = work_state.as_deref().map(|state| {
            state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        super::convention_delete_rule(
            global.as_deref(),
            guard.as_deref().and_then(Option::as_ref),
            &cache,
            tier,
            id,
        )
    }

    #[tauri::command]
    pub fn convention_set_enabled<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        rule_key: String,
        enabled: bool,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(cache) = app.try_state::<ConventionMatcherState>() else {
            return Err(state_missing());
        };
        let work_state = app.try_state::<OpenWorkState>();
        let guard = work_state.as_deref().map(|state| {
            state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        super::convention_set_enabled(
            global.as_deref(),
            guard.as_deref().and_then(Option::as_ref),
            &cache,
            &rule_key,
            enabled,
        )
    }

    /// Async: the match scales with the Chapter. The Work lock is held to copy the text and while
    /// the matcher is built; the match itself runs outside it.
    #[tauri::command(async)]
    pub fn convention_scan_chapter<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        generation: u64,
    ) -> Result<ConventionScanWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(cache) = app.try_state::<ConventionMatcherState>() else {
            return Err(state_missing());
        };
        let state = app.try_state::<OpenWorkState>();
        super::convention_scan_chapter(global.as_deref(), state.as_deref(), &cache, generation)
    }

    /// Same scan and result as `convention_scan_chapter`; `progress` receives the sentence count
    /// as it goes. A closed channel never fails the scan.
    #[tauri::command(async)]
    pub fn convention_scan_chapter_progress<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        generation: u64,
        progress: tauri::ipc::Channel<ConventionScanProgressWire>,
    ) -> Result<ConventionScanWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(cache) = app.try_state::<ConventionMatcherState>() else {
            return Err(state_missing());
        };
        let state = app.try_state::<OpenWorkState>();
        super::convention_scan_chapter_with_progress(
            global.as_deref(),
            state.as_deref(),
            &cache,
            generation,
            |done, total| {
                let _ = progress.send(ConventionScanProgressWire { done, total });
            },
        )
    }

    /// Async: reads the listed sentences from disk and matches them outside the Work lock.
    #[tauri::command(async)]
    pub fn convention_scan_segments<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        generation: u64,
        segment_ids: Vec<i64>,
    ) -> Result<ConventionScanWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(cache) = app.try_state::<ConventionMatcherState>() else {
            return Err(state_missing());
        };
        let state = app.try_state::<OpenWorkState>();
        super::convention_scan_segments(
            global.as_deref(),
            state.as_deref(),
            &cache,
            generation,
            &segment_ids,
        )
    }

    /// Async: one match per suggestion and one Glossary snapshot per batch. Read-only; no command
    /// accepts a span of a suggestion.
    #[tauri::command(async)]
    pub fn convention_scan_suggestions<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        generation: u64,
        items: Vec<ConventionSuggestionItem>,
    ) -> Result<ConventionSuggestionScanWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(cache) = app.try_state::<ConventionMatcherState>() else {
            return Err(state_missing());
        };
        let state = app.try_state::<OpenWorkState>();
        super::convention_scan_suggestions(
            global.as_deref(),
            state.as_deref(),
            &cache,
            generation,
            items,
        )
    }

    fn with_open<R: tauri::Runtime, T>(
        app: &tauri::AppHandle<R>,
        job: impl FnOnce(Option<&crate::commands::project::OpenWork>) -> Result<T, IpcError>,
    ) -> Result<T, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return job(None);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        job(guard.as_ref())
    }

    #[tauri::command]
    pub fn convention_ignore_add<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        segment_id: i64,
        expected_text: String,
        start: u32,
        end: u32,
        expected_phrase: String,
        rule_key: String,
    ) -> Result<(), IpcError> {
        with_open(&app, |open| {
            super::convention_ignore_add(
                open,
                segment_id,
                &expected_text,
                start,
                end,
                &expected_phrase,
                &rule_key,
            )
        })
    }

    #[tauri::command]
    pub fn convention_ignore_list<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<Vec<ConventionIgnoreWire>, IpcError> {
        with_open(&app, super::convention_ignore_list)
    }

    #[tauri::command]
    pub fn convention_ignore_remove<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        id: i64,
    ) -> Result<bool, IpcError> {
        with_open(&app, |open| super::convention_ignore_remove(open, id))
    }

    #[tauri::command]
    pub fn convention_ignore_clear<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<u32, IpcError> {
        with_open(&app, super::convention_ignore_clear)
    }

    fn work_tier_is_open<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> bool {
        use tauri::Manager as _;

        app.try_state::<OpenWorkState>().is_some_and(|state| {
            state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
        })
    }

    /// Async: the save dialog blocks, and on the main thread that deadlocks the event loop the
    /// dialog waits on. No `OpenWorkState` lock is held across the dialog: the Work-tier check takes
    /// and releases it first, and the rows are read under a fresh lock after the dialog closes.
    #[tauri::command(async)]
    pub fn convention_export_tier<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        tier: RuleTier,
    ) -> Result<Option<String>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        if global.is_none() {
            return Err(super::store_is_missing());
        }
        if tier == RuleTier::Work && !work_tier_is_open(&app) {
            return Err(ConventionStoreError::WorkTierUnavailable.into());
        }
        let picked_path = match app
            .dialog()
            .file()
            .add_filter("CSV", &["csv"])
            .set_file_name(format!("convention_{}.csv", tier.as_str()))
            .blocking_save_file()
        {
            None => None,
            Some(picked) => Some(
                picked
                    .into_path()
                    .map_err(|_| IpcError::from(ConventionStoreError::DialogPathInvalid))?,
            ),
        };
        with_open(&app, |open| {
            super::convention_export_tier_after_dialog(global.as_deref(), open, tier, picked_path)
        })
    }

    /// Async for the same reason as `convention_export_tier`.
    #[tauri::command(async)]
    pub fn convention_open_import_preview<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        tier: RuleTier,
    ) -> Result<Option<ConventionImportPreviewWire>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        if global.is_none() {
            return Err(super::store_is_missing());
        }
        if tier == RuleTier::Work && !work_tier_is_open(&app) {
            return Err(ConventionStoreError::WorkTierUnavailable.into());
        }
        let Some(pending) = app.try_state::<ConventionPendingImportState>() else {
            return Err(state_missing());
        };
        let picked_path = match app
            .dialog()
            .file()
            .add_filter("CSV/TSV", &["csv", "tsv"])
            .blocking_pick_file()
        {
            None => None,
            Some(picked) => Some(
                picked
                    .into_path()
                    .map_err(|_| IpcError::from(ConventionStoreError::DialogPathInvalid))?,
            ),
        };
        with_open(&app, |open| {
            super::convention_open_import_preview_after_dialog(
                global.as_deref(),
                open,
                &pending,
                tier,
                picked_path,
            )
        })
    }

    /// Async: one transaction over every row of the plan, and it holds the pending slot for the
    /// whole write.
    #[tauri::command(async)]
    pub fn convention_confirm_import<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        decisions: BTreeMap<String, ConflictDecision>,
    ) -> Result<ConventionImportSummaryWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let (Some(cache), Some(pending)) = (
            app.try_state::<ConventionMatcherState>(),
            app.try_state::<ConventionPendingImportState>(),
        ) else {
            return Err(state_missing());
        };
        with_open(&app, |open| {
            super::convention_confirm_import(global.as_deref(), open, &cache, &pending, &decisions)
        })
    }

    /// Async: locks the slot `convention_confirm_import` holds for its whole write.
    #[tauri::command(async)]
    pub fn convention_cancel_import<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        if let Some(pending) = app.try_state::<ConventionPendingImportState>() {
            super::convention_cancel_import(&pending);
        }
        Ok(())
    }
}
