//! TM management surface (FR62, FR63): list both tiers with filters, edit or delete one pair,
//! bulk-delete the others side, push one Work pair up to Global.
//!
//! Strings here are unaccented; `scripts/check-i18n.mjs` scans `src-tauri/**/*.rs`.

use std::collections::BTreeMap;

use crate::commands::project::OpenWork;
use crate::commands::segment::{global_store_missing, side_wire, tier_wire, tm_lookup_failed, tm_pair_not_found};
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::store::Store;
use crate::core::tm::{CopyRef, OriginFilter, PushOutcome, TierFilter, TmPair, TmTier};

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

fn parse_filters(tier: &str, origin: &str) -> Result<(TierFilter, OriginFilter), IpcError> {
    let tier = TierFilter::from_wire(tier).ok_or_else(|| invalid_filter("tier", tier))?;
    let origin = OriginFilter::from_wire(origin).ok_or_else(|| invalid_filter("origin", origin))?;
    Ok((tier, origin))
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
    origin: OriginFilter,
    search: String,
    resolver: crate::core::scope::ScopeResolver,
    snapshot: crate::core::tm::ManageSnapshot,
}

/// Validates the filters and loads every pair of both tiers; no Work open means Global only.
pub fn prepare_tm_list(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    origin: &str,
    tier: &str,
    search: &str,
) -> Result<TmListScan, IpcError> {
    let (tier, origin) = parse_filters(tier, origin)?;
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
    Ok(TmListScan { work_open: open.is_some(), tier, origin, search: search.to_owned(), resolver, snapshot })
}

pub fn score_tm_list(scan: TmListScan) -> Result<TmPairList, IpcError> {
    let listing = crate::core::tm::rank_manage_listing(&scan.resolver, scan.snapshot, scan.tier, scan.origin, &scan.search)
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
            .map(|(origin, count)| TmOriginCountWire { translation_origin: origin.as_str(), count })
            .collect(),
        total_pairs: listing.total_pairs,
        total_groups: listing.total_groups,
        groups,
    })
}

/// Lists pairs of both tiers narrowed by `origin` (`all`, `mine`, `others` or a stored value),
/// `tier` (`both`, `work`, `global`) and `search` (FR62), grouped by source (FR63).
pub fn tm_list_pairs(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    origin: &str,
    tier: &str,
    search: &str,
) -> Result<TmPairList, IpcError> {
    score_tm_list(prepare_tm_list(global, open, origin, tier, search)?)
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

pub mod wire {
    use super::{TmCopyArg, TmDeleteOthersOutcome, TmPairList, TmPairWire};
    use crate::commands::project::OpenWorkState;
    use crate::core::i18n::IpcError;

    /// Wire shell of [`super::tm_list_pairs`]; `origin`, `tier`, `search` on the wire. Async, and
    /// the `OpenWorkState` lock is released before filtering.
    #[tauri::command(async)]
    pub fn tm_list_pairs<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        origin: String,
        tier: String,
        search: String,
    ) -> Result<TmPairList, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<crate::core::store::Store>();
        let scan = match app.try_state::<OpenWorkState>() {
            None => super::prepare_tm_list(global.as_deref(), None, &origin, &tier, &search)?,
            Some(state) => {
                let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                super::prepare_tm_list(global.as_deref(), guard.as_ref(), &origin, &tier, &search)?
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
}
