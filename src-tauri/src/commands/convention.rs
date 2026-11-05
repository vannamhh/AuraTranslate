//! IPC for Vietnamese writing conventions (FR159, AD-67): the Global rule list and the scan of
//! the open Chapter. The scan reads stored text only and writes nothing.
//!
//! Strings in this file are written without diacritics (`scripts/check-i18n.mjs`).

use std::sync::{Arc, Mutex};

use crate::commands::project::{OpenWork, OpenWorkState};
use crate::commands::segment::select_chapter_segments;
use crate::core::convention::{
    Matcher, RuleTier, add_rule, find_active_rule, list_rules, phrase_spans, transfer_case,
};
use crate::core::glossary::{GlossarySnapshot, match_lang_for_source_lang};
use crate::core::i18n::IpcError;
use crate::core::segment::utf16::utf16_to_byte;
use crate::core::store::{Store, StoreError, StoreKind};

pub(crate) fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

/// The matcher built from the enabled rules, kept until the rule list changes.
#[derive(Default)]
pub struct ConventionMatcherState(Mutex<Option<Arc<Matcher>>>);

impl ConventionMatcherState {
    fn invalidate(&self) {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    fn matcher(&self, global: &Store) -> Result<Arc<Matcher>, IpcError> {
        let mut slot = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(matcher) = slot.as_ref() {
            return Ok(Arc::clone(matcher));
        }
        let rules = list_rules(global)?
            .into_iter()
            .filter(|row| row.enabled)
            .map(crate::core::convention::RuleRow::into_rule)
            .collect();
        let matcher = Arc::new(Matcher::build(rules));
        *slot = Some(Arc::clone(&matcher));
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
}

pub fn convention_list_rules(global: Option<&Store>) -> Result<Vec<ConventionRuleWire>, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    Ok(list_rules(global)?
        .into_iter()
        .map(|row| ConventionRuleWire {
            id: row.id,
            rule_key: row.rule_key,
            avoid_phrase: row.avoid_phrase,
            replacement_phrase: row.replacement_phrase,
            enabled: row.enabled,
            tier: RuleTier::Global.as_str().to_owned(),
        })
        .collect())
}

/// The enabled Global rule with `rule_key`; a rule that is gone or disabled refuses.
pub fn convention_active_rule(
    global: Option<&Store>,
    rule_key: &str,
) -> Result<crate::core::convention::RuleRow, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    find_active_rule(global, rule_key)?.ok_or_else(crate::commands::segment::rule_unavailable)
}

/// Adds a Global rule and drops the cached matcher. A blank avoid phrase or a key already present
/// is refused with nothing written.
pub fn convention_add_rule(
    global: Option<&Store>,
    cache: &ConventionMatcherState,
    avoid_phrase: &str,
    replacement_phrase: Option<&str>,
) -> Result<i64, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let id = add_rule(global, avoid_phrase, replacement_phrase)?;
    cache.invalidate();
    Ok(id)
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
/// matching.
fn read_scan_input(open: Option<&OpenWork>) -> Result<ScanInput, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let chapter_id = open.chapter_id;
    let rows = open
        .store
        .read(|conn| select_chapter_segments(conn, chapter_id))?;
    let mut not_scanned = 0u32;
    let mut sentences = Vec::new();
    for row in rows {
        if row.is_omitted {
            not_scanned += 1;
        } else if !row.target_text.trim().is_empty() {
            sentences.push(SentenceInput {
                segment_id: row.id,
                source_text: row.source_text,
                target_text: row.target_text,
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

/// Matches `input` against `matcher`. A match that lies inside a pinned Glossary translation found
/// in the same sentence is dropped and counted (AD-67 rule 5); the snapshot is requested only once a
/// sentence has a match.
fn match_sentences(
    generation: u64,
    input: ScanInput,
    matcher: &Matcher,
    mut snapshot: impl FnMut() -> Result<Option<GlossarySnapshot>, IpcError>,
) -> Result<ConventionScanWire, IpcError> {
    let lang = match_lang_for_source_lang(&input.source_lang);
    let mut loaded: Option<Option<GlossarySnapshot>> = None;
    let mut segments = Vec::new();
    let (mut overlapped, mut suppressed) = (0u32, 0u32);
    for sentence in input.sentences {
        let scan = matcher.scan(&sentence.target_text);
        overlapped += scan.overlapped;
        if scan.hits.is_empty() {
            continue;
        }
        if loaded.is_none() {
            loaded = Some(snapshot()?);
        }
        let pinned: Vec<(u32, u32)> = loaded
            .as_ref()
            .and_then(Option::as_ref)
            .map(|glossary| {
                glossary
                    .confirmed_terms(&sentence.source_text, lang)
                    .injected
            })
            .unwrap_or_default()
            .iter()
            .flat_map(|term| phrase_spans(&sentence.target_text, &term.translation))
            .collect();
        let mut spans = Vec::new();
        for hit in scan.hits {
            if pinned.iter().any(|&(s, e)| s <= hit.start && hit.end <= e) {
                suppressed += 1;
                continue;
            }
            let rule = matcher.rule(hit.rule);
            let matched = utf16_to_byte(&sentence.target_text, hit.start)
                .zip(utf16_to_byte(&sentence.target_text, hit.end))
                .map(|(a, b)| &sentence.target_text[a..b])
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
        if !spans.is_empty() {
            segments.push(ConventionSegmentWire {
                segment_id: sentence.segment_id,
                scanned_text: sentence.target_text,
                spans,
            });
        }
    }
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
    let global = global.ok_or_else(store_is_missing)?;
    let lock = || {
        open_state.map(|state| {
            state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        })
    };
    let input = read_scan_input(lock().as_deref().and_then(Option::as_ref))?;
    let matcher = cache.matcher(global)?;
    let chapter_id = input.chapter_id;
    match_sentences(generation, input, &matcher, || {
        let guard = lock();
        load_glossary(
            global,
            guard.as_deref().and_then(Option::as_ref),
            chapter_id,
        )
    })
}

fn state_missing() -> IpcError {
    crate::commands::aitranslate::state_missing(
        "convention",
        "ConventionMatcherState",
        "convention.state_missing",
    )
}

pub mod wire {
    use super::{ConventionMatcherState, ConventionRuleWire, ConventionScanWire, state_missing};
    use crate::commands::project::OpenWorkState;
    use crate::core::i18n::IpcError;
    use crate::core::store::Store;

    #[tauri::command]
    pub fn convention_list_rules<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<Vec<ConventionRuleWire>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        super::convention_list_rules(global.as_deref())
    }

    #[tauri::command]
    pub fn convention_add_rule<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        avoid_phrase: String,
        replacement_phrase: Option<String>,
    ) -> Result<i64, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(cache) = app.try_state::<ConventionMatcherState>() else {
            return Err(state_missing());
        };
        super::convention_add_rule(
            global.as_deref(),
            &cache,
            &avoid_phrase,
            replacement_phrase.as_deref(),
        )
    }

    /// Async: the match scales with the Chapter, and the Work lock is held only to copy its text.
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
}
