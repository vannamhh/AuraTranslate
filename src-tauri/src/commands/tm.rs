//! TM management surface (FR62, FR63): list both tiers with filters, edit or delete one pair,
//! bulk-delete the others side, push one Work pair up to Global.
//!
//! Strings here are unaccented; `scripts/check-i18n.mjs` scans `src-tauri/**/*.rs`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::commands::project::OpenWork;
use crate::commands::segment::{global_store_missing, side_wire, tier_wire, tm_lookup_failed, tm_pair_not_found};
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::store::Store;
use crate::core::tm::tmx::{
    ImportPlan, PlannedPair, TmxError, TmxTier, distinct_tier_pairs, existing_pair_keys, pairs_not_in, parse_tmx,
    plan_import, render_tmx, write_planned_pairs,
};
use crate::core::tm::tmx_io::{read_tmx_file, write_tmx_file};
use crate::core::tm::{CopyRef, PairOriginFilter, PushOutcome, TierFilter, TmPair, TmTier};

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmPairWire {
    /// `"work"` or `"global"`.
    pub tier: &'static str,
    pub unit_id: i64,
    pub source_text: String,
    pub target_text: String,
    /// The stored value: `"self"`, `"other"` or `"bilingual_import"`.
    pub translation_origin: &'static str,
    /// `"mine"` or `"others"` (AD-47 6).
    pub side: &'static str,
    /// ISO-8601 UTC with milliseconds.
    pub created_at: String,
}

fn pair_wire(pair: TmPair) -> TmPairWire {
    TmPairWire {
        tier: tier_wire(pair.tier),
        unit_id: pair.id,
        source_text: pair.source_text,
        target_text: pair.target_text,
        translation_origin: pair.translation_origin.as_str(),
        side: side_wire(pair.translation_origin),
        created_at: pair.created_at,
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmCopyWire {
    pub tier: &'static str,
    pub unit_id: i64,
}

/// A copy as the webview hands it back from a list row.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TmCopyArg {
    pub tier: String,
    pub unit_id: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmGroupRowWire {
    /// Tier and id of the first copy.
    pub tier: &'static str,
    pub unit_id: i64,
    /// Every copy of this (source, target) in AD-18 order, the first one included.
    pub copies: Vec<TmCopyWire>,
    pub target_text: String,
    pub translation_origin: &'static str,
    pub side: &'static str,
    pub created_at: String,
    /// Stored copies of this (source, target) the current filters hide.
    pub hidden_copies: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmSourceGroupWire {
    pub source_text: String,
    /// Two or more means the webview shows a group header (FR63).
    pub distinct_targets: usize,
    /// AD-18 order.
    pub rows: Vec<TmGroupRowWire>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmOriginCountWire {
    pub translation_origin: &'static str,
    pub count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmPairList {
    /// A Work is open, so the Work tier and push-up exist.
    pub work_open: bool,
    /// Both tiers hold no pair at all.
    pub tm_empty: bool,
    /// Always three entries (`self`, `other`, `bilingual_import`) over the tier filter alone.
    pub health: Vec<TmOriginCountWire>,
    /// Pairs passing every filter, before the cap.
    pub total_pairs: usize,
    /// Source groups passing every filter, before the cap; `groups.len()` is smaller when capped.
    pub total_groups: usize,
    pub groups: Vec<TmSourceGroupWire>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmDeleteOthersOutcome {
    pub deleted_work: usize,
    pub deleted_global: usize,
}

fn invalid_filter(name: &str, value: &str) -> IpcError {
    IpcError::new(
        "tm.invalid_filter",
        MessageKey::Unknown,
        BTreeMap::from([("filter".to_owned(), name.to_owned()), ("value".to_owned(), value.to_owned())]),
        false,
    )
}

fn target_empty() -> IpcError {
    IpcError::new("tm.target_empty", MessageKey::Unknown, BTreeMap::new(), false)
}

fn global_pair_exists() -> IpcError {
    IpcError::new("tm.global_pair_exists", MessageKey::Unknown, BTreeMap::new(), false)
}

fn parse_filters(tier: &str, pair_origin: &str) -> Result<(TierFilter, PairOriginFilter), IpcError> {
    let tier = TierFilter::from_wire(tier).ok_or_else(|| invalid_filter("tier", tier))?;
    let pair_origin = PairOriginFilter::from_wire(pair_origin).ok_or_else(|| invalid_filter("pair_origin", pair_origin))?;
    Ok((tier, pair_origin))
}

fn copy_refs(copies: &[TmCopyArg]) -> Result<Vec<CopyRef>, IpcError> {
    if copies.is_empty() {
        return Err(tm_pair_not_found("none", 0));
    }
    copies
        .iter()
        .map(|c| match c.tier.as_str() {
            "work" => Ok(CopyRef { tier: TmTier::Work, id: c.unit_id }),
            "global" => Ok(CopyRef { tier: TmTier::Global, id: c.unit_id }),
            _ => Err(tm_pair_not_found(&c.tier, c.unit_id)),
        })
        .collect()
}

fn first_copy_gone(copies: &[TmCopyArg]) -> IpcError {
    tm_pair_not_found(&copies[0].tier, copies[0].unit_id)
}

/// A Work copy needs an open Work, as `tm_delete_others` does for tier `work`.
fn work_store_for<'a>(open: Option<&'a OpenWork>, copies: &[CopyRef]) -> Result<Option<&'a Store>, IpcError> {
    if open.is_none() && copies.iter().any(|c| c.tier == TmTier::Work) {
        return Err(crate::commands::chapter::no_work_open());
    }
    Ok(open.map(|o| &o.store))
}

/// Rows already read, so the filtering touches neither `OpenWorkState` nor a store.
pub struct TmListScan {
    work_open: bool,
    tier: TierFilter,
    pair_origin: PairOriginFilter,
    search: String,
    resolver: crate::core::scope::ScopeResolver,
    snapshot: crate::core::tm::ManageSnapshot,
}

/// Validates the filters and loads every pair of both tiers; no Work open means Global only.
pub fn prepare_tm_list(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    pair_origin: &str,
    tier: &str,
    search: &str,
) -> Result<TmListScan, IpcError> {
    let (tier, pair_origin) = parse_filters(tier, pair_origin)?;
    if tier == TierFilter::Work && open.is_none() {
        return Err(crate::commands::chapter::no_work_open());
    }
    let global = global.ok_or_else(global_store_missing)?;
    let resolver = match open {
        Some(open) => open.scope.clone(),
        None => crate::core::scope::ScopeResolver::global_only(),
    };
    let snapshot = crate::core::tm::load_manage_snapshot(global, open.map(|o| &o.store))
        .map_err(|e| tm_lookup_failed(&e))?;
    Ok(TmListScan { work_open: open.is_some(), tier, pair_origin, search: search.to_owned(), resolver, snapshot })
}

pub fn score_tm_list(scan: TmListScan) -> Result<TmPairList, IpcError> {
    let listing = crate::core::tm::rank_manage_listing(&scan.resolver, scan.snapshot, scan.tier, scan.pair_origin, &scan.search)
        .map_err(|e| tm_lookup_failed(&e))?;
    let groups = listing
        .groups
        .into_iter()
        .map(|group| TmSourceGroupWire {
            source_text: group.source_text,
            distinct_targets: group.distinct_targets,
            rows: group
                .rows
                .into_iter()
                .map(|row| TmGroupRowWire {
                    tier: tier_wire(row.pair.tier),
                    unit_id: row.pair.id,
                    copies: row
                        .copies
                        .iter()
                        .map(|c| TmCopyWire { tier: tier_wire(c.tier), unit_id: c.id })
                        .collect(),
                    target_text: row.pair.target_text,
                    translation_origin: row.pair.translation_origin.as_str(),
                    side: side_wire(row.pair.translation_origin),
                    created_at: row.pair.created_at,
                    hidden_copies: row.hidden_copies,
                })
                .collect(),
        })
        .collect();
    Ok(TmPairList {
        work_open: scan.work_open,
        tm_empty: listing.tm_empty,
        health: listing
            .health
            .into_iter()
            .map(|(pair_origin, count)| TmOriginCountWire { translation_origin: pair_origin.as_str(), count })
            .collect(),
        total_pairs: listing.total_pairs,
        total_groups: listing.total_groups,
        groups,
    })
}

/// Lists pairs of both tiers narrowed by `pair_origin` (`all`, `mine`, `others` or a stored value),
/// `tier` (`both`, `work`, `global`) and `search` (FR62), grouped by source (FR63).
pub fn tm_list_pairs(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    pair_origin: &str,
    tier: &str,
    search: &str,
) -> Result<TmPairList, IpcError> {
    score_tm_list(prepare_tm_list(global, open, pair_origin, tier, search)?)
}

/// Replaces the target on every copy of a row; each copy's origin becomes `self`, id and date
/// stay. Copies are re-read first and only those still holding `source_text` and
/// `expected_target` are touched; none left is `tm.pair_not_found`. A `target_text` equal to
/// `expected_target` writes nothing.
pub fn tm_update_pair_target(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    copies: &[TmCopyArg],
    source_text: &str,
    expected_target: &str,
    target_text: &str,
) -> Result<TmPairWire, IpcError> {
    let global = global.ok_or_else(global_store_missing)?;
    let refs = copy_refs(copies)?;
    let work = work_store_for(open, &refs)?;
    if target_text.trim().is_empty() {
        return Err(target_empty());
    }
    let stored =
        crate::core::tm::update_copies_target(global, work, &refs, source_text, expected_target, target_text)
            .map_err(|e| tm_lookup_failed(&e))?;
    stored.into_iter().next().map(pair_wire).ok_or_else(|| first_copy_gone(copies))
}

/// Deletes every live copy of a row; `tm.pair_not_found` when none is left.
pub fn tm_delete_pair(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    copies: &[TmCopyArg],
    source_text: &str,
    expected_target: &str,
) -> Result<(), IpcError> {
    let global = global.ok_or_else(global_store_missing)?;
    let refs = copy_refs(copies)?;
    let work = work_store_for(open, &refs)?;
    let deleted = crate::core::tm::delete_copies(global, work, &refs, source_text, expected_target)
        .map_err(|e| tm_lookup_failed(&e))?;
    if deleted > 0 { Ok(()) } else { Err(first_copy_gone(copies)) }
}

/// Deletes every pair on the others side (`other`, `bilingual_import`) in the tiers `tier`
/// shows. `tier = work` with no Work open is `work.none_open`; `both` then means Global only.
pub fn tm_delete_others(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: &str,
) -> Result<TmDeleteOthersOutcome, IpcError> {
    let tier = TierFilter::from_wire(tier).ok_or_else(|| invalid_filter("tier", tier))?;
    let global = global.ok_or_else(global_store_missing)?;
    if tier == TierFilter::Work && open.is_none() {
        return Err(crate::commands::chapter::no_work_open());
    }
    let mut outcome = TmDeleteOthersOutcome { deleted_work: 0, deleted_global: 0 };
    if let (true, Some(open)) = (tier != TierFilter::Global, open) {
        outcome.deleted_work = crate::core::tm::delete_others_side(&open.store).map_err(|e| tm_lookup_failed(&e))?;
    }
    if tier != TierFilter::Work {
        outcome.deleted_global = crate::core::tm::delete_others_side(global).map_err(|e| tm_lookup_failed(&e))?;
    }
    Ok(outcome)
}

/// Moves a row to Global keeping origin and date (one copy written to Global first, then every
/// Work copy deleted). `tm.global_pair_exists` when a copy is already Global or Global holds the
/// identical pair, nothing written.
pub fn tm_push_pair_to_global(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    copies: &[TmCopyArg],
    source_text: &str,
    expected_target: &str,
) -> Result<TmPairWire, IpcError> {
    let global = global.ok_or_else(global_store_missing)?;
    let refs = copy_refs(copies)?;
    let work = open.map(|o| &o.store);
    if work.is_none() {
        return Err(crate::commands::chapter::no_work_open());
    }
    match crate::core::tm::push_copies_to_global(global, work, &refs, source_text, expected_target)
        .map_err(|e| tm_lookup_failed(&e))?
    {
        PushOutcome::Moved(pair) => Ok(pair_wire(pair)),
        PushOutcome::PairNotFound => Err(first_copy_gone(copies)),
        PushOutcome::GlobalHasPair => Err(global_pair_exists()),
    }
}

/// A TMX import held between preview and confirm: the plan stays in Rust, the webview only
/// sees counts (AD-48).
#[derive(Debug)]
pub struct PendingTmxImport {
    pub tier: TmTier,
    pub pairs: Vec<PlannedPair>,
    /// The Work the plan was made against; `Some` only for `TmTier::Work`.
    pub work_id: Option<String>,
    /// What the preview reported as already there (in the tier or repeated in the file).
    pub already_count: usize,
}

/// `None` means no import is waiting.
pub type PendingTmxImportState = std::sync::Mutex<Option<PendingTmxImport>>;

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmxImportPreviewWire {
    pub file_name: String,
    /// `"work"` or `"global"`: the tier chosen when the file was opened.
    pub tier: &'static str,
    /// `<tu>` elements read.
    pub unit_count: usize,
    /// Pairs the confirm would add.
    pub new_count: usize,
    /// Pairs already in the tier or repeated earlier in the file.
    pub already_count: usize,
    /// `<tu>` missing a side or holding a blank one.
    pub skipped_count: usize,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct TmxImportSummaryWire {
    pub inserted: usize,
    /// The preview's already-there count plus pairs that appeared in the tier before the
    /// transaction ran.
    pub already_count: usize,
    /// Inserted pairs whose file date was after the import moment and was replaced by it.
    pub future_dated_count: usize,
}

fn tmx_error(err: TmxError) -> IpcError {
    let (code, params, retryable) = match &err {
        TmxError::Malformed { line, .. } => {
            ("tm.tmx_malformed", BTreeMap::from([("line".to_owned(), line.to_string())]), false)
        }
        TmxError::NoBody => ("tm.tmx_no_body", BTreeMap::new(), false),
        TmxError::NotUtf8 => ("tm.tmx_not_utf8", BTreeMap::new(), false),
        TmxError::TooLarge { .. } => ("tm.tmx_too_large", BTreeMap::new(), false),
        TmxError::NoUsablePair { source_lang } => (
            "tm.tmx_no_usable_pair",
            source_lang.iter().map(|l| ("source_lang".to_owned(), l.clone())).collect(),
            false,
        ),
        TmxError::ReadFailed { .. } => ("tm.tmx_read_failed", BTreeMap::new(), true),
        TmxError::WriteFailed { .. } => ("tm.tmx_write_failed", BTreeMap::new(), true),
    };
    eprintln!("tm tmx that bai: {err}");
    IpcError::new(code, MessageKey::Unknown, params, retryable)
}

fn no_pending_tmx_import() -> IpcError {
    IpcError::new("tm.no_pending_import", MessageKey::Unknown, BTreeMap::new(), false)
}

fn tmx_tier_from_wire(tier: &str) -> Result<TmTier, IpcError> {
    match tier {
        "work" => Ok(TmTier::Work),
        "global" => Ok(TmTier::Global),
        other => Err(invalid_filter("tier", other)),
    }
}

/// The store of `tier`; the Work tier needs an open Work.
fn tmx_tier_store<'a>(global: &'a Store, open: Option<&'a OpenWork>, tier: TmTier) -> Result<&'a Store, IpcError> {
    match tier {
        TmTier::Global => Ok(global),
        TmTier::Work => open.map(|o| &o.store).ok_or_else(crate::commands::chapter::no_work_open),
    }
}

/// Pairs read from one tier plus the label data TMX rendering needs, owned so the store access
/// can end before rendering starts.
#[derive(Debug)]
pub struct TierPairs {
    tier: TmTier,
    source_lang: Option<String>,
    pairs: Vec<TmPair>,
}

fn owned_work_label(open: Option<&OpenWork>, tier: TmTier) -> Option<String> {
    match (tier, open) {
        (TmTier::Work, Some(open)) => Some(open.meta.source_lang.clone()),
        _ => None,
    }
}

/// Reads the distinct pairs of one tier. `tier = work` with no Work open is `work.none_open`.
pub fn tm_read_tier_pairs(global: Option<&Store>, open: Option<&OpenWork>, tier: &str) -> Result<TierPairs, IpcError> {
    let tier = tmx_tier_from_wire(tier)?;
    let global = global.ok_or_else(global_store_missing)?;
    let store = tmx_tier_store(global, open, tier)?;
    let pairs = distinct_tier_pairs(store, tier).map_err(|e| tm_lookup_failed(&e))?;
    Ok(TierPairs { tier, source_lang: owned_work_label(open, tier), pairs })
}

/// Renders pairs read by [`tm_read_tier_pairs`] as TMX 1.4b: one `<tu>` per distinct (source, target).
pub fn tm_render_tier_pairs(read: &TierPairs) -> String {
    let label = match (&read.tier, &read.source_lang) {
        (TmTier::Work, Some(source_lang)) => TmxTier::Work { source_lang },
        _ => TmxTier::Global,
    };
    render_tmx(&read.pairs, label)
}

/// Renders one tier as TMX 1.4b. Reads the store only; the file is written by [`tm_write_export`].
pub fn tm_render_tier(global: Option<&Store>, open: Option<&OpenWork>, tier: &str) -> Result<String, IpcError> {
    Ok(tm_render_tier_pairs(&tm_read_tier_pairs(global, open, tier)?))
}

/// Writes rendered TMX to `path` atomically.
pub fn tm_write_export(path: &Path, text: &str) -> Result<(), IpcError> {
    write_tmx_file(path, text).map_err(tmx_error)
}

pub fn tm_export_tier(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: &str,
    path: &Path,
) -> Result<(), IpcError> {
    tm_write_export(path, &tm_render_tier(global, open, tier)?)
}

/// The export once the dialog answered: `None` (cancelled) touches no file.
pub fn tm_export_tier_after_dialog(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: &str,
    picked_path: Option<PathBuf>,
) -> Result<Option<String>, IpcError> {
    let Some(path) = picked_path else { return Ok(None) };
    tm_export_tier(global, open, tier, &path)?;
    Ok(Some(path.display().to_string()))
}

fn counts(plan: &ImportPlan, new_count: usize, file_name: String, tier: TmTier) -> TmxImportPreviewWire {
    TmxImportPreviewWire {
        file_name,
        tier: tier_wire(tier),
        unit_count: plan.unit_count,
        new_count,
        already_count: plan.pairs.len() - new_count + plan.duplicate_in_file_count,
        skipped_count: plan.skipped_count,
    }
}

/// Reads and parses a TMX file; touches no state, so a caller can run it without any lock.
pub fn tm_read_import_file(path: &Path) -> Result<crate::core::tm::tmx::ParsedTmx, IpcError> {
    let text = read_tmx_file(path).map_err(tmx_error)?;
    parse_tmx(&text).map_err(tmx_error)
}

/// Drops any waiting plan: a new preview starts from none, so cancel and failure leave none.
pub fn tm_discard_pending_import(pending: &PendingTmxImportState) {
    *pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

/// What the tier already holds, read under the store access so planning can run without it.
#[derive(Debug)]
pub struct ExistingPairs {
    tier: TmTier,
    source_lang: Option<String>,
    work_id: Option<String>,
    keys: std::collections::HashSet<(String, String)>,
}

pub fn tm_read_existing_pairs(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: &str,
) -> Result<ExistingPairs, IpcError> {
    let tier = tmx_tier_from_wire(tier)?;
    let global = global.ok_or_else(global_store_missing)?;
    let store = tmx_tier_store(global, open, tier)?;
    let keys = existing_pair_keys(store).map_err(|e| tm_lookup_failed(&e))?;
    let work_id = open.filter(|_| tier == TmTier::Work).map(|o| o.meta.work_id.clone());
    Ok(ExistingPairs { tier, source_lang: owned_work_label(open, tier), work_id, keys })
}

/// Plans a parsed file against `existing` and keeps the pairs the tier lacks in `pending`.
pub fn tm_plan_preview(
    pending: &PendingTmxImportState,
    existing: &ExistingPairs,
    parsed: &crate::core::tm::tmx::ParsedTmx,
    file_name: String,
) -> Result<TmxImportPreviewWire, IpcError> {
    let label = match (&existing.tier, &existing.source_lang) {
        (TmTier::Work, Some(source_lang)) => TmxTier::Work { source_lang },
        _ => TmxTier::Global,
    };
    let plan = plan_import(parsed, label).map_err(tmx_error)?;
    let new_pairs = pairs_not_in(&plan, &existing.keys);

    let preview = counts(&plan, new_pairs.len(), file_name, existing.tier);
    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = Some(PendingTmxImport {
        tier: existing.tier,
        pairs: new_pairs,
        work_id: existing.work_id.clone(),
        already_count: preview.already_count,
    });
    Ok(preview)
}

/// Plans a parsed file for `tier` and keeps the pairs the tier lacks in `pending`.
pub fn tm_preview_parsed(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    pending: &PendingTmxImportState,
    tier: &str,
    parsed: &crate::core::tm::tmx::ParsedTmx,
    file_name: String,
) -> Result<TmxImportPreviewWire, IpcError> {
    let existing = tm_read_existing_pairs(global, open, tier)?;
    tm_plan_preview(pending, &existing, parsed, file_name)
}

fn file_name_of(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Drops any earlier plan, then reads and plans a TMX file for `tier`. Every failure leaves no plan.
pub fn tm_open_import_preview(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    pending: &PendingTmxImportState,
    tier: &str,
    path: &Path,
) -> Result<TmxImportPreviewWire, IpcError> {
    tm_discard_pending_import(pending);
    let parsed_tier = tmx_tier_from_wire(tier)?;
    let global_store = global.ok_or_else(global_store_missing)?;
    tmx_tier_store(global_store, open, parsed_tier)?;
    let parsed = tm_read_import_file(path)?;
    tm_preview_parsed(global, open, pending, tier, &parsed, file_name_of(path))
}

/// Writes the pending plan to its tier in one transaction. Cleared only on success; an error keeps
/// the plan for a retry.
pub fn tm_confirm_import(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    pending: &PendingTmxImportState,
    file_is_mine: bool,
) -> Result<TmxImportSummaryWire, IpcError> {
    let global = global.ok_or_else(global_store_missing)?;
    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(batch) = guard.as_ref() else { return Err(no_pending_tmx_import()) };
    if batch.tier == TmTier::Work && open.is_none_or(|o| batch.work_id.as_deref() != Some(o.meta.work_id.as_str())) {
        *guard = None;
        return Err(no_pending_tmx_import());
    }
    let batch_already = batch.already_count;
    let store = tmx_tier_store(global, open, batch.tier)?;
    let outcome = write_planned_pairs(store, batch.pairs.clone(), file_is_mine).map_err(|e| tm_lookup_failed(&e))?;
    *guard = None;
    Ok(TmxImportSummaryWire {
        inserted: outcome.inserted,
        already_count: batch_already + outcome.already_there,
        future_dated_count: outcome.future_dated,
    })
}

/// The tier of the waiting plan, so a wire shell knows whether it needs `OpenWorkState`.
pub fn tm_pending_import_tier(pending: &PendingTmxImportState) -> Option<TmTier> {
    pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner).as_ref().map(|batch| batch.tier)
}

/// Confirms a Global plan without any Work store. `Ok(None)` when the waiting plan is not Global
/// (it changed since [`tm_pending_import_tier`]): nothing is written and the caller takes the Work path.
pub fn tm_confirm_global_import(
    global: Option<&Store>,
    pending: &PendingTmxImportState,
    file_is_mine: bool,
) -> Result<Option<TmxImportSummaryWire>, IpcError> {
    let global = global.ok_or_else(global_store_missing)?;
    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(batch) = guard.as_ref() else { return Err(no_pending_tmx_import()) };
    if batch.tier != TmTier::Global {
        return Ok(None);
    }
    let batch_already = batch.already_count;
    let outcome = write_planned_pairs(global, batch.pairs.clone(), file_is_mine).map_err(|e| tm_lookup_failed(&e))?;
    *guard = None;
    Ok(Some(TmxImportSummaryWire {
        inserted: outcome.inserted,
        already_count: batch_already + outcome.already_there,
        future_dated_count: outcome.future_dated,
    }))
}

/// Drops the pending plan; cancelling with none waiting is harmless.
pub fn tm_cancel_import(pending: &PendingTmxImportState) {
    tm_discard_pending_import(pending);
}

/// A Work plan points at a store that is closing; a Global plan survives.
pub fn clear_pending_tmx_import_for_work(pending: &PendingTmxImportState) {
    let mut guard = match pending.try_lock() {
        Ok(guard) => guard,
        Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(std::sync::TryLockError::WouldBlock) => return,
    };
    if guard.as_ref().is_some_and(|batch| batch.tier == TmTier::Work) {
        *guard = None;
    }
}

pub mod wire {
    use super::{
        PendingTmxImportState, TmCopyArg, TmDeleteOthersOutcome, TmPairList, TmPairWire, TmxImportPreviewWire,
        TmxImportSummaryWire,
    };
    use crate::commands::project::OpenWorkState;
    use crate::core::i18n::IpcError;

    /// Wire shell of [`super::tm_list_pairs`]; `pair_origin`, `tier`, `search` on the wire. Async, and
    /// the `OpenWorkState` lock is released before filtering.
    #[tauri::command(async)]
    pub fn tm_list_pairs<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        pair_origin: String,
        tier: String,
        search: String,
    ) -> Result<TmPairList, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<crate::core::store::Store>();
        let scan = match app.try_state::<OpenWorkState>() {
            None => super::prepare_tm_list(global.as_deref(), None, &pair_origin, &tier, &search)?,
            Some(state) => {
                let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                super::prepare_tm_list(global.as_deref(), guard.as_ref(), &pair_origin, &tier, &search)?
            }
        };
        super::score_tm_list(scan)
    }

    /// Wire shell of [`super::tm_update_pair_target`]; `copies`, `sourceText`, `expectedTarget`,
    /// `targetText` on the wire.
    #[tauri::command]
    pub fn tm_update_pair_target<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        copies: Vec<TmCopyArg>,
        source_text: String,
        expected_target: String,
        target_text: String,
    ) -> Result<TmPairWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<crate::core::store::Store>();
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::tm_update_pair_target(global.as_deref(), None, &copies, &source_text, &expected_target, &target_text);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::tm_update_pair_target(global.as_deref(), guard.as_ref(), &copies, &source_text, &expected_target, &target_text)
    }

    /// Wire shell of [`super::tm_delete_pair`]; `copies`, `sourceText`, `expectedTarget` on the wire.
    #[tauri::command]
    pub fn tm_delete_pair<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        copies: Vec<TmCopyArg>,
        source_text: String,
        expected_target: String,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<crate::core::store::Store>();
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::tm_delete_pair(global.as_deref(), None, &copies, &source_text, &expected_target);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::tm_delete_pair(global.as_deref(), guard.as_ref(), &copies, &source_text, &expected_target)
    }

    /// Wire shell of [`super::tm_delete_others`]; `tier` on the wire. Async: it can delete every
    /// others-side pair of two stores.
    #[tauri::command(async)]
    pub fn tm_delete_others<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        tier: String,
    ) -> Result<TmDeleteOthersOutcome, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<crate::core::store::Store>();
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::tm_delete_others(global.as_deref(), None, &tier);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::tm_delete_others(global.as_deref(), guard.as_ref(), &tier)
    }

    /// Wire shell of [`super::tm_push_pair_to_global`]; `copies`, `sourceText`, `expectedTarget`
    /// on the wire.
    #[tauri::command]
    pub fn tm_push_pair_to_global<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        copies: Vec<TmCopyArg>,
        source_text: String,
        expected_target: String,
    ) -> Result<TmPairWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<crate::core::store::Store>();
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::tm_push_pair_to_global(global.as_deref(), None, &copies, &source_text, &expected_target);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::tm_push_pair_to_global(global.as_deref(), guard.as_ref(), &copies, &source_text, &expected_target)
    }
    fn tier_is_ready<R: tauri::Runtime>(app: &tauri::AppHandle<R>, tier: &str) -> Result<(), IpcError> {
        use tauri::Manager as _;

        if app.try_state::<crate::core::store::Store>().is_none() {
            return Err(crate::commands::segment::global_store_missing());
        }
        if super::tmx_tier_from_wire(tier)? == crate::core::tm::TmTier::Work {
            let open = app
                .try_state::<OpenWorkState>()
                .is_some_and(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner).is_some());
            if !open {
                return Err(crate::commands::chapter::no_work_open());
            }
        }
        Ok(())
    }

    fn picked_to_path(picked: tauri_plugin_dialog::FilePath) -> Result<std::path::PathBuf, IpcError> {
        picked.into_path().map_err(|_| {
            IpcError::new("tm.dialog_path_invalid", crate::core::i18n::MessageKey::Unknown, Default::default(), false)
        })
    }

    /// Runs `f` with the open Work locked for the Work tier only; the Global tier never touches
    /// `OpenWorkState`.
    fn with_tier_store<R: tauri::Runtime, T>(
        app: &tauri::AppHandle<R>,
        tier: &str,
        f: impl FnOnce(Option<&crate::core::store::Store>, Option<&crate::commands::project::OpenWork>) -> T,
    ) -> T {
        use tauri::Manager as _;

        let global = app.try_state::<crate::core::store::Store>();
        if tier != "work" {
            return f(global.as_deref(), None);
        }
        let work_state = app.try_state::<OpenWorkState>();
        let guard = work_state.as_ref().map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        f(global.as_deref(), guard.as_ref().and_then(|g| g.as_ref()))
    }

    /// The export once the dialog picked `path`: the store is read under the lock (Work tier only),
    /// rendering and file I/O run without it.
    pub fn export_tier_to<R: tauri::Runtime>(
        app: &tauri::AppHandle<R>,
        tier: &str,
        path: &std::path::Path,
    ) -> Result<Option<String>, IpcError> {
        let read = with_tier_store(app, tier, |global, open| super::tm_read_tier_pairs(global, open, tier))?;
        super::tm_write_export(path, &super::tm_render_tier_pairs(&read))?;
        Ok(Some(path.display().to_string()))
    }

    /// Wire shell of [`super::tm_export_tier_after_dialog`]; `tier` on the wire. Async: the save
    /// dialog blocks, and on the main thread that deadlocks the event loop it waits on. `None`
    /// is a cancelled dialog.
    #[tauri::command(async)]
    pub fn tm_export_tier<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        tier: String,
    ) -> Result<Option<String>, IpcError> {
        use tauri_plugin_dialog::DialogExt as _;

        tier_is_ready(&app, &tier)?;
        let default_name = if tier == "global" { "tm_global.tmx" } else { "tm_work.tmx" };
        let picked_path = match app.dialog().file().add_filter("TMX", &["tmx"]).set_file_name(default_name).blocking_save_file() {
            None => return Ok(None),
            Some(picked) => picked_to_path(picked)?,
        };
        export_tier_to(&app, &tier, &picked_path)
    }

    /// The preview once the dialog picked `path`: the file is read and parsed unlocked, the Work
    /// store is read under the lock (Work tier only), planning runs without it.
    pub fn open_import_preview_from<R: tauri::Runtime>(
        app: &tauri::AppHandle<R>,
        tier: &str,
        path: &std::path::Path,
    ) -> Result<TmxImportPreviewWire, IpcError> {
        use tauri::Manager as _;

        let Some(pending) = app.try_state::<PendingTmxImportState>() else {
            return Err(super::no_pending_tmx_import());
        };
        let parsed = super::tm_read_import_file(path)?;
        let existing = with_tier_store(app, tier, |global, open| super::tm_read_existing_pairs(global, open, tier))?;
        super::tm_plan_preview(pending.inner(), &existing, &parsed, super::file_name_of(path))
    }

    /// Wire shell of [`super::tm_open_import_preview`]; `tier` on the wire. Async for the same
    /// reason as `tm_export_tier`. `None` is a cancelled dialog; every outcome but success leaves no
    /// plan.
    #[tauri::command(async)]
    pub fn tm_open_import_preview<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        tier: String,
    ) -> Result<Option<TmxImportPreviewWire>, IpcError> {
        use tauri::Manager as _;
        use tauri_plugin_dialog::DialogExt as _;

        tier_is_ready(&app, &tier)?;
        let Some(pending) = app.try_state::<PendingTmxImportState>() else {
            return Err(super::no_pending_tmx_import());
        };
        super::tm_discard_pending_import(pending.inner());
        let Some(picked) = app.dialog().file().add_filter("TMX", &["tmx", "xml"]).blocking_pick_file() else {
            return Ok(None);
        };
        let path = picked_to_path(picked)?;
        open_import_preview_from(&app, &tier, &path).map(Some)
    }

    /// Wire shell of [`super::tm_confirm_import`]. Async: it holds `PendingTmxImportState` for the
    /// whole write. A Global plan never takes `OpenWorkState`; a Work plan takes it (then the pending
    /// lock) for the write.
    #[tauri::command(async)]
    pub fn tm_confirm_import<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        file_is_mine: bool,
    ) -> Result<TmxImportSummaryWire, IpcError> {
        use tauri::Manager as _;

        let Some(pending) = app.try_state::<PendingTmxImportState>() else {
            return Err(super::no_pending_tmx_import());
        };
        let global = app.try_state::<crate::core::store::Store>();
        match super::tm_pending_import_tier(pending.inner()) {
            None => return Err(super::no_pending_tmx_import()),
            Some(crate::core::tm::TmTier::Global) => {
                if let Some(summary) = super::tm_confirm_global_import(global.as_deref(), pending.inner(), file_is_mine)? {
                    return Ok(summary);
                }
            }
            Some(crate::core::tm::TmTier::Work) => {}
        }
        with_tier_store(&app, "work", |global, open| super::tm_confirm_import(global, open, pending.inner(), file_is_mine))
    }

    /// Wire shell of [`super::tm_cancel_import`]. Async: it locks `PendingTmxImportState`, which
    /// `tm_confirm_import` holds for its whole write.
    #[tauri::command(async)]
    pub fn tm_cancel_import<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        if let Some(pending) = app.try_state::<PendingTmxImportState>() {
            super::tm_cancel_import(pending.inner());
        }
        Ok(())
    }
}
