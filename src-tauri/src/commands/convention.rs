//! IPC for Vietnamese writing conventions (FR159, AD-67): the Global rule list and the scan of
//! the open Chapter. The scan reads stored text only and writes nothing.
//!
//! Strings in this file are written without diacritics (`scripts/check-i18n.mjs`).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::commands::project::{OpenWork, OpenWorkState};
use crate::commands::segment::select_chapter_segments;
use crate::core::convention::{
    ConventionStoreError, Matcher, RuleRow, RuleStatus, RuleTier, add_rule, delete_rule,
    disable_in_work, edit_rule, effective_row, enable_in_work, find_row_by_key, list_rules,
    phrase_spans, resolve_two_tiers, row_status, transfer_case,
};
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

/// Matches of one scan of the open Chapter. `ignored` stays 0 until the "not an error" memory
/// exists; `not_scanned` counts omitted sentences.
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
/// inside a pinned Glossary translation of that sentence (AD-67 rule 5).
fn resolve_spans(
    matcher: &Matcher,
    lang: crate::core::matching::MatchLang,
    glossary: Option<&GlossarySnapshot>,
    source_text: &str,
    text: &str,
    hits: Vec<crate::core::convention::Hit>,
) -> (Vec<ConventionSpanWire>, u32) {
    let pinned: Vec<(u32, u32)> = glossary
        .map(|glossary| glossary.confirmed_terms(source_text, lang).injected)
        .unwrap_or_default()
        .iter()
        .flat_map(|term| phrase_spans(text, &term.translation))
        .collect();
    let mut spans = Vec::new();
    let mut suppressed = 0u32;
    for hit in hits {
        if pinned.iter().any(|&(s, e)| s <= hit.start && hit.end <= e) {
            suppressed += 1;
            continue;
        }
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
    (spans, suppressed)
}

/// Matches `input` against `matcher`. A match that lies inside a pinned Glossary translation found
/// in the same sentence is dropped and counted (AD-67 rule 5); the snapshot is requested only once a
/// sentence has a match.
fn match_sentences(
    generation: u64,
    input: ScanInput,
    matcher: &Matcher,
    mut snapshot: impl FnMut() -> Result<Option<GlossarySnapshot>, IpcError>,
    mut progress: impl FnMut(u32, u32),
) -> Result<ConventionScanWire, IpcError> {
    let total = u32::try_from(input.sentences.len()).unwrap_or(u32::MAX);
    let lang = match_lang_for_source_lang(&input.source_lang);
    let mut loaded: Option<Option<GlossarySnapshot>> = None;
    let mut segments = Vec::new();
    let (mut overlapped, mut suppressed) = (0u32, 0u32);
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
        let (spans, dropped) = resolve_spans(
            matcher,
            lang,
            loaded.as_ref().and_then(Option::as_ref),
            &sentence.source_text,
            &sentence.target_text,
            scan.hits,
        );
        suppressed += dropped;
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
        ignored: 0,
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
    let (chapter_id, source_lang, matcher) = {
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
    let mut suppressed = 0u32;
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
            let (spans, dropped) = resolve_spans(
                &matcher,
                lang,
                glossary.as_ref(),
                source_text,
                &item.text,
                hits,
            );
            suppressed += dropped;
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
        ignored: 0,
        glossary_suppressed: suppressed,
        overlapped,
    })
}

fn state_missing() -> IpcError {
    crate::core::i18n::state_missing(
        "convention",
        "ConventionMatcherState",
        "convention.state_missing",
    )
}

pub mod wire {
    use super::{
        ConventionMatcherState, ConventionRuleWire, ConventionScanProgressWire, ConventionScanWire,
        ConventionSuggestionItem, ConventionSuggestionScanWire, ConventionTierFilter,
        state_missing,
    };
    use crate::commands::project::OpenWorkState;
    use crate::core::convention::RuleTier;
    use crate::core::i18n::IpcError;
    use crate::core::store::Store;

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
}
