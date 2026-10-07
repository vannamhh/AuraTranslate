use super::*;

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmFuzzyMatch {
    /// `"work"` or `"global"`; with `unit_id` it identifies the pair.
    pub tier: &'static str,
    pub unit_id: i64,
    pub percent: u8,
    pub source_text: String,
    pub target_text: String,
    /// The pair's source (old) against the caret segment's source (new), AD-51.
    pub diff: Vec<crate::core::matching::DiffSpan>,
    /// `"mine"` or `"others"` (AD-47 ⑥).
    pub side: &'static str,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmFuzzyMatches {
    /// The segment the scan was run for; the webview drops a response for another segment.
    pub segment_id: i64,
    pub matches: Vec<TmFuzzyMatch>,
    /// Every distinct target of an exact source match, AD-18 order; non-empty only with 2+
    /// distinct targets, and then `matches` is empty.
    pub exact: Vec<TmExactTarget>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmExactTarget {
    /// `"work"` or `"global"`; with `unit_id` it identifies the pair.
    pub tier: &'static str,
    pub unit_id: i64,
    pub target_text: String,
    /// `"mine"` or `"others"` (AD-47 ⑥).
    pub side: &'static str,
    /// ISO-8601 UTC with milliseconds.
    pub created_at: String,
}

pub(crate) fn tier_wire(tier: crate::core::tm::TmTier) -> &'static str {
    match tier {
        crate::core::tm::TmTier::Work => "work",
        crate::core::tm::TmTier::Global => "global",
    }
}

pub(crate) fn side_wire(pair_origin: crate::core::tm::PairOrigin) -> &'static str {
    match pair_origin.side() {
        crate::core::tm::PairSide::Mine => "mine",
        crate::core::tm::PairSide::Others => "others",
    }
}

/// What a fuzzy scan needs once the caller's locks are released: rows already read, so the
/// scoring (the slow part) touches neither `OpenWorkState` nor a store.
pub struct TmFuzzyScan {
    segment_id: i64,
    source: String,
    lang: crate::core::matching::MatchLang,
    threshold: u8,
    resolver: crate::core::scope::ScopeResolver,
    candidates: crate::core::tm::TierRows,
}

pub enum TmFuzzyPrepared {
    Empty(TmFuzzyMatches),
    Scan(TmFuzzyScan),
}

/// Reads the segment, rules out an exact pair, and loads every candidate row (FR59). Holds
/// whatever lock the caller holds only for reads; [`score_tm_fuzzy`] does the scoring.
pub fn prepare_tm_fuzzy(
    global: Option<&crate::core::store::Store>,
    open: Option<&OpenWork>,
    segment_id: i64,
) -> Result<TmFuzzyPrepared, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let global = global.ok_or_else(global_store_missing)?;

    let found = open.store.read(move |conn| {
        match conn.query_row(
            "SELECT source_text, retired_at IS NOT NULL FROM segment WHERE id = ?1",
            [segment_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?)),
        ) {
            Ok(v) => Ok(Some(v)),
            Err(SqlError::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err),
        }
    })?;
    let (source, retired) = found.ok_or_else(|| segment_not_found(segment_id))?;
    if retired {
        return Err(segment_retired(segment_id));
    }
    let empty = || {
        TmFuzzyPrepared::Empty(TmFuzzyMatches { segment_id, matches: Vec::new(), exact: Vec::new() })
    };
    if source.trim().is_empty() {
        return Ok(empty());
    }

    let exact = crate::core::tm::pairs_for_source(&open.scope, global, Some(&open.store), &source)
        .map_err(|e| tm_lookup_failed(&e))?;
    if !exact.is_empty() {
        let distinct = crate::core::tm::distinct_exact_targets(exact);
        if distinct.len() < 2 {
            return Ok(empty());
        }
        let exact = distinct
            .into_iter()
            .map(|p| TmExactTarget {
                tier: tier_wire(p.tier),
                unit_id: p.id,
                side: side_wire(p.translation_origin),
                target_text: p.target_text,
                created_at: p.created_at,
            })
            .collect();
        return Ok(TmFuzzyPrepared::Empty(TmFuzzyMatches {
            segment_id,
            matches: Vec::new(),
            exact,
        }));
    }

    let threshold = crate::core::scope::load_global_config(global)?.tm_fuzzy_threshold();
    let candidates = crate::core::tm::load_tier_rows(global, Some(&open.store))
        .map_err(|e| tm_lookup_failed(&e))?;
    Ok(TmFuzzyPrepared::Scan(TmFuzzyScan {
        segment_id,
        lang: crate::core::glossary::match_lang_for_source_lang(&open.meta.source_lang),
        source,
        threshold: u8::try_from(threshold).unwrap_or(u8::MAX),
        resolver: open.scope.clone(),
        candidates,
    }))
}

/// Scores a prepared scan: top three at or above the threshold, each with its source diff.
pub fn score_tm_fuzzy(prepared: TmFuzzyPrepared) -> Result<TmFuzzyMatches, IpcError> {
    let scan = match prepared {
        TmFuzzyPrepared::Empty(done) => return Ok(done),
        TmFuzzyPrepared::Scan(scan) => scan,
    };
    let pairs = crate::core::tm::rank_fuzzy_candidates(
        &scan.resolver,
        scan.candidates,
        &scan.source,
        scan.lang,
        scan.threshold,
    )
    .map_err(|e| tm_lookup_failed(&e))?;

    let matches = pairs
        .into_iter()
        .map(|f| TmFuzzyMatch {
            tier: match f.pair.tier {
                crate::core::tm::TmTier::Work => "work",
                crate::core::tm::TmTier::Global => "global",
            },
            unit_id: f.pair.id,
            percent: f.percent,
            diff: crate::core::matching::diff_spans(&f.pair.source_text, &scan.source, scan.lang),
            source_text: f.pair.source_text,
            target_text: f.pair.target_text,
            side: match f.pair.translation_origin.side() {
                crate::core::tm::PairSide::Mine => "mine",
                crate::core::tm::PairSide::Others => "others",
            },
        })
        .collect();
    Ok(TmFuzzyMatches { segment_id: scan.segment_id, matches, exact: Vec::new() })
}

/// Fuzzy TM matches for a segment's source (FR59): both tiers scored by `core::matching`,
/// top three at or above `tm_fuzzy_threshold`. Empty when an exact pair exists (pre-fill owns
/// that case) or nothing reaches the threshold.
pub fn tm_fuzzy_matches(
    global: Option<&crate::core::store::Store>,
    open: Option<&OpenWork>,
    segment_id: i64,
) -> Result<TmFuzzyMatches, IpcError> {
    score_tm_fuzzy(prepare_tm_fuzzy(global, open, segment_id)?)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmConcordanceHit {
    /// `"work"` or `"global"`.
    pub tier: &'static str,
    pub unit_id: i64,
    pub source_text: String,
    pub target_text: String,
    /// `"mine"` or `"others"` (AD-47 ⑥).
    pub side: &'static str,
    /// ISO-8601 UTC with milliseconds.
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmConcordance {
    /// The query as searched; the webview drops a response for another query.
    pub query: String,
    /// Both tiers hold no pair at all.
    pub tm_empty: bool,
    /// Matching pairs before the cap; `hits.len()` is smaller when capped.
    pub total: usize,
    pub hits: Vec<TmConcordanceHit>,
}

/// Rows already read, so the substring filter touches neither `OpenWorkState` nor a store.
pub struct TmConcordanceScan {
    query: String,
    resolver: crate::core::scope::ScopeResolver,
    candidates: crate::core::tm::TierRows,
}

/// Loads every pair of both tiers; no Work open means the Global tier only.
pub fn prepare_tm_concordance(
    global: Option<&crate::core::store::Store>,
    open: Option<&OpenWork>,
    query: &str,
) -> Result<TmConcordanceScan, IpcError> {
    let global = global.ok_or_else(global_store_missing)?;
    let resolver = match open {
        Some(open) => open.scope.clone(),
        None => crate::core::scope::ScopeResolver::global_only(),
    };
    let candidates = crate::core::tm::load_tier_rows(global, open.map(|o| &o.store))
        .map_err(|e| tm_lookup_failed(&e))?;
    Ok(TmConcordanceScan { query: query.to_owned(), resolver, candidates })
}

pub fn score_tm_concordance(scan: TmConcordanceScan) -> Result<TmConcordance, IpcError> {
    let found = crate::core::tm::rank_concordance(&scan.resolver, scan.candidates, &scan.query)
        .map_err(|e| tm_lookup_failed(&e))?;
    let hits = found
        .hits
        .into_iter()
        .map(|pair| TmConcordanceHit {
            tier: tier_wire(pair.tier),
            unit_id: pair.id,
            source_text: pair.source_text,
            target_text: pair.target_text,
            side: side_wire(pair.translation_origin),
            created_at: pair.created_at,
        })
        .collect();
    Ok(TmConcordance { query: scan.query, tm_empty: found.tm_empty, total: found.total, hits })
}

/// Concordance (FR60): every pair of both tiers whose source contains `query`.
pub fn tm_concordance(
    global: Option<&crate::core::store::Store>,
    open: Option<&OpenWork>,
    query: &str,
) -> Result<TmConcordance, IpcError> {
    score_tm_concordance(prepare_tm_concordance(global, open, query)?)
}

pub fn tm_pair_not_found(tier: &str, unit_id: i64) -> IpcError {
    IpcError::new(
        "tm.pair_not_found",
        MessageKey::Unknown,
        BTreeMap::from([("tier".to_owned(), tier.to_owned()), ("unit_id".to_owned(), unit_id.to_string())]),
        false,
    )
}

/// Accepts one fuzzy row (FR59, AD-51 rule 8): re-reads the pair by tier and id, then writes
/// its target as an unconfirmed `draft` with origin `other` through the promote path, so a
/// draft with text needs `force` after `needs_confirmation`. `tier` is `"work"` or `"global"`.
pub fn accept_tm_fuzzy(
    global: Option<&crate::core::store::Store>,
    open: Option<&OpenWork>,
    segment_id: i64,
    tier: &str,
    unit_id: i64,
    force: bool,
) -> Result<PromoteAiTranslationOutcome, IpcError> {
    let work = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let global = global.ok_or_else(global_store_missing)?;
    let (store, pair_tier) = match tier {
        "work" => (&work.store, crate::core::tm::TmTier::Work),
        "global" => (global, crate::core::tm::TmTier::Global),
        _ => return Err(tm_pair_not_found(tier, unit_id)),
    };
    let pair = crate::core::tm::pair_by_id(store, pair_tier, unit_id)
        .map_err(|e| tm_lookup_failed(&e))?
        .ok_or_else(|| tm_pair_not_found(tier, unit_id))?;
    promote_ai_translation(open, segment_id, &pair.target_text, force)
}

/// Picks one target of an exact-source list (FR63): re-reads the pair by tier and id, refuses
/// it when its source is no longer the segment's source, then writes the pair's own origin as an
/// unconfirmed `draft` (AD-47 ③ "Điền sẵn từ TM khớp 100%"). Asks (`needs_confirmation`, nothing
/// written) only when the current text was typed by the user: non-empty, different from
/// `baseline_target_text`, and with no copy in `segment_version`.
pub fn accept_tm_exact(
    global: Option<&crate::core::store::Store>,
    open: Option<&OpenWork>,
    segment_id: i64,
    tier: &str,
    unit_id: i64,
    force: bool,
) -> Result<PromoteAiTranslationOutcome, IpcError> {
    let work = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let global = global.ok_or_else(global_store_missing)?;
    let (store, pair_tier) = match tier {
        "work" => (&work.store, crate::core::tm::TmTier::Work),
        "global" => (global, crate::core::tm::TmTier::Global),
        _ => return Err(tm_pair_not_found(tier, unit_id)),
    };
    let pair = crate::core::tm::pair_by_id(store, pair_tier, unit_id)
        .map_err(|e| tm_lookup_failed(&e))?
        .ok_or_else(|| tm_pair_not_found(tier, unit_id))?;
    let pair_origin = pair.translation_origin.as_str();
    let target = pair.target_text;
    let pair_source = pair.source_text;

    enum Picked {
        Missing,
        Retired,
        SourceChanged,
        Row(String, String, String, bool),
    }

    let outcome = work.store.write(move |tx: &Transaction<'_>| {
        let found = tx.query_row(
            "SELECT source_text, target_text, baseline_target_text, translation_origin, \
             retired_at IS NOT NULL, status FROM segment WHERE id = ?1",
            [segment_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, bool>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        );
        let (source, current_text, baseline, current_origin, retired, current_status) = match found {
            Ok(value) => value,
            Err(SqlError::QueryReturnedNoRows) => return Ok(Picked::Missing),
            Err(err) => return Err(err),
        };
        if retired {
            return Ok(Picked::Retired);
        }
        if source != pair_source {
            return Ok(Picked::SourceChanged);
        }
        if !force && !current_text.is_empty() && current_text != baseline {
            let has_copy: i64 = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM segment_version \
                 WHERE segment_id = ?1 AND target_text = ?2)",
                (segment_id, &current_text),
                |row| row.get(0),
            )?;
            if has_copy == 0 {
                return Ok(Picked::Row(current_text, current_origin, current_status, true));
            }
        }
        write_non_user_target(tx, segment_id, &target, pair_origin, Some(pair_origin))?;
        Ok(Picked::Row(target, pair_origin.to_owned(), SEGMENT_STATUS_DRAFT.to_owned(), false))
    })?;

    let (target_text, translation_origin, status, needs_confirmation) = match outcome {
        Picked::Missing => return Err(segment_not_found(segment_id)),
        Picked::Retired => return Err(segment_retired(segment_id)),
        Picked::SourceChanged => return Err(tm_pair_not_found(tier, unit_id)),
        Picked::Row(text, pair_origin, status, ask) => (text, pair_origin, status, ask),
    };
    let unsigned_draft = needs_confirmation.then(|| target_text.clone());
    Ok(PromoteAiTranslationOutcome {
        segment_id,
        target_text,
        translation_origin,
        status,
        needs_confirmation,
        unsigned_draft,
    })
}
