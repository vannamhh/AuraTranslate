//! Translation Memory — khoá theo CẶP VĂN BẢN, không theo `segment.id` (AD-6).
//!
//! Nhờ vậy TM sống sót qua gộp/tách segment và dùng lại được xuyên Tác phẩm.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 THÊM 2026-09-17 (Story 4.6) — `SimilarSegment`, hình dạng TỐI THIỂU tham số TM của
//! `core::ai::rag` cần
//! ─────────────────────────────────────────────────────────────────────────────
//! `RagInjector` (AD-14) nhận TM làm tham số thứ tư, nhưng Epic 7 (module thật của TM) chạy
//! SAU Epic 4 — chữ ký cần một kiểu PHẦN TỬ thật ngay hôm nay, không phải `()` hay một kiểu
//! đoán mò. Kiểu này KHÔNG thể sống dưới `core/ai/**`: `tests/ai_boundary.rs:99` cấm module
//! khác gõ tên `crate::core::ai`, nên nếu Epic 7 cần đặt TÊN kiểu phần tử của chính module
//! mình (để trả về từ một hàm tìm kiếm thật) mà kiểu đó sống ở `core::ai`, `core::tm` sẽ phải
//! `use crate::core::ai::SimilarSegment` — đúng phụ thuộc mà AD-13 cấm. `core::ai` được PHÉP
//! đọc `core::tm` (chiều ngược của AD-13), không phải chiều kia.
//!
//! Epic 7 sẽ LỚN kiểu này lên (điểm tương đồng, id đoạn nguồn, …) — `core::ai::rag` chỉ nhận
//! `&[SimilarSegment]`, không tháo rời từng trường, nên việc lớn lên không đổi chữ ký của
//! `gather_glossary_context`/`assemble_prompt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarSegment {
    /// Câu nguồn đã lưu trong TM.
    pub source_text: String,
    /// Bản dịch đã lưu song song với `source_text`.
    pub target_text: String,
}

/// FR118 binary axis over the FR117 origins that may enter TM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairSide {
    Mine,
    Others,
}

/// The origins a TM pair may carry (AD-47 ⑥); `''` ("no answer") is not representable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairOrigin {
    SelfTranslated,
    Other,
    BilingualImport,
}

impl PairOrigin {
    pub fn from_stored(value: &str) -> Option<Self> {
        match value {
            "self" => Some(Self::SelfTranslated),
            "other" => Some(Self::Other),
            "bilingual_import" => Some(Self::BilingualImport),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelfTranslated => "self",
            Self::Other => "other",
            Self::BilingualImport => "bilingual_import",
        }
    }

    pub fn side(self) -> PairSide {
        match self {
            Self::SelfTranslated => PairSide::Mine,
            Self::Other | Self::BilingualImport => PairSide::Others,
        }
    }
}

/// Appends a pair in the caller's transaction; existing rows are never updated (AD-6).
pub fn insert_pair(
    tx: &crate::core::store::Transaction<'_>,
    source_text: &str,
    target_text: &str,
    translation_origin: PairOrigin,
) -> crate::core::store::SqlResult<()> {
    tx.execute(
        "INSERT INTO tm_unit (source_text, target_text, translation_origin, created_at) \
         VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
        (source_text, target_text, translation_origin.as_str()),
    )?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TmTier {
    Work,
    Global,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmPair {
    pub id: i64,
    pub source_text: String,
    pub target_text: String,
    pub translation_origin: PairOrigin,
    pub tier: TmTier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TmStoreError {
    Store(crate::core::store::StoreError),
    Scope(crate::core::scope::ScopeError),
    UnknownOrigin { value: String },
}

impl std::fmt::Display for TmStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => write!(f, "tm[store] {e}"),
            Self::Scope(e) => write!(f, "tm[scope] {e}"),
            Self::UnknownOrigin { value } => write!(f, "tm[unknown_origin] {value:?}"),
        }
    }
}

impl std::error::Error for TmStoreError {}

impl From<crate::core::store::StoreError> for TmStoreError {
    fn from(e: crate::core::store::StoreError) -> Self {
        Self::Store(e)
    }
}

impl From<crate::core::scope::ScopeError> for TmStoreError {
    fn from(e: crate::core::scope::ScopeError) -> Self {
        Self::Scope(e)
    }
}

const TM_SCOPE_KIND: &str = "translation_memory";

#[derive(Clone)]
struct RawPair {
    id: i64,
    source_text: String,
    target_text: String,
    translation_origin: PairOrigin,
}

fn load_pair_rows(
    store: &crate::core::store::Store,
    source_text: &str,
) -> Result<Vec<RawPair>, TmStoreError> {
    let source = source_text.to_owned();
    query_pair_rows(
        store,
        "SELECT id, source_text, target_text, translation_origin FROM tm_unit \
         WHERE source_text = ?1 ORDER BY id",
        Some(source),
    )
}

fn load_all_pair_rows(store: &crate::core::store::Store) -> Result<Vec<RawPair>, TmStoreError> {
    query_pair_rows(
        store,
        "SELECT id, source_text, target_text, translation_origin FROM tm_unit ORDER BY id",
        None,
    )
}

fn query_pair_rows(
    store: &crate::core::store::Store,
    sql: &'static str,
    source: Option<String>,
) -> Result<Vec<RawPair>, TmStoreError> {
    let raw: Vec<(i64, String, String, String)> = store.read(move |conn| {
        let mut stmt = conn.prepare(sql)?;
        let map = |r: &crate::core::store::Row<'_>| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?));
        let rows = match &source {
            Some(s) => stmt.query_map([s], map)?.collect::<crate::core::store::SqlResult<Vec<_>>>()?,
            None => stmt.query_map([], map)?.collect::<crate::core::store::SqlResult<Vec<_>>>()?,
        };
        Ok(rows)
    })?;
    raw.into_iter()
        .map(|(id, source_text, target_text, origin)| {
            let translation_origin = PairOrigin::from_stored(&origin)
                .ok_or(TmStoreError::UnknownOrigin { value: origin })?;
            Ok(RawPair { id, source_text, target_text, translation_origin })
        })
        .collect()
}

fn side_rank(side: PairSide) -> u8 {
    match side {
        PairSide::Mine => 0,
        PairSide::Others => 1,
    }
}

/// Pairs whose source equals `source_text`, both tiers merged: mine before others, then Work
/// before Global, then load order. Exact equality; normalization belongs to matching.
pub fn pairs_for_source(
    resolver: &crate::core::scope::ScopeResolver,
    global: &crate::core::store::Store,
    work: Option<&crate::core::store::Store>,
    source_text: &str,
) -> Result<Vec<TmPair>, TmStoreError> {
    let global_rows = load_pair_rows(global, source_text)?;
    let work_rows = work.map(|w| load_pair_rows(w, source_text)).transpose()?;
    merge_tiers(resolver, global_rows, work_rows)
}

fn merge_tiers(
    resolver: &crate::core::scope::ScopeResolver,
    global_rows: Vec<RawPair>,
    work_rows: Option<Vec<RawPair>>,
) -> Result<Vec<TmPair>, TmStoreError> {
    let by_side = |a: &RawPair, b: &RawPair| {
        side_rank(a.translation_origin.side()).cmp(&side_rank(b.translation_origin.side()))
    };
    let tiered = resolver.apply_merge(TM_SCOPE_KIND, &global_rows, work_rows.as_deref(), Some(&by_side))?;
    Ok(tiered
        .into_iter()
        .map(|t| {
            let tier = match t.tier() {
                crate::core::scope::Tier::Work => TmTier::Work,
                crate::core::scope::Tier::Global => TmTier::Global,
            };
            let raw = t.value();
            TmPair {
                id: raw.id,
                source_text: raw.source_text.clone(),
                target_text: raw.target_text.clone(),
                translation_origin: raw.translation_origin,
                tier,
            }
        })
        .collect())
}

/// Number of fuzzy rows the strip shows.
pub const FUZZY_MATCH_LIMIT: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuzzyPair {
    pub pair: TmPair,
    pub percent: u8,
}

/// Every pair of both tiers, read once so scoring can run after the caller released its locks.
pub struct FuzzyCandidates {
    global_rows: Vec<RawPair>,
    work_rows: Option<Vec<RawPair>>,
}

pub fn load_fuzzy_candidates(
    global: &crate::core::store::Store,
    work: Option<&crate::core::store::Store>,
) -> Result<FuzzyCandidates, TmStoreError> {
    Ok(FuzzyCandidates {
        global_rows: load_all_pair_rows(global)?,
        work_rows: work.map(load_all_pair_rows).transpose()?,
    })
}

/// Pairs of both tiers whose source scores at least `threshold` percent against `source_text`,
/// best first, ties in `pairs_for_source` order, at most [`FUZZY_MATCH_LIMIT`]. A pair whose
/// source equals `source_text` is excluded (exact matches belong to the pre-fill).
pub fn fuzzy_pairs_for_source(
    resolver: &crate::core::scope::ScopeResolver,
    global: &crate::core::store::Store,
    work: Option<&crate::core::store::Store>,
    source_text: &str,
    lang: crate::core::matching::MatchLang,
    threshold: u8,
) -> Result<Vec<FuzzyPair>, TmStoreError> {
    let candidates = load_fuzzy_candidates(global, work)?;
    rank_fuzzy_candidates(resolver, candidates, source_text, lang, threshold)
}

pub fn rank_fuzzy_candidates(
    resolver: &crate::core::scope::ScopeResolver,
    candidates: FuzzyCandidates,
    source_text: &str,
    lang: crate::core::matching::MatchLang,
    threshold: u8,
) -> Result<Vec<FuzzyPair>, TmStoreError> {
    let mut scorer = crate::core::matching::SimilarityScorer::new(source_text, lang);
    let mut scores: std::collections::HashMap<(bool, i64), u8> = std::collections::HashMap::new();
    let mut keep = |tier_is_work: bool, rows: Vec<RawPair>| -> Vec<RawPair> {
        rows.into_iter()
            .filter(|row| {
                if row.source_text == source_text {
                    return false;
                }
                let percent = scorer.percent(&row.source_text).min(99);
                if percent < threshold {
                    return false;
                }
                scores.insert((tier_is_work, row.id), percent);
                true
            })
            .collect()
    };
    let global_rows = keep(false, candidates.global_rows);
    let work_rows = candidates.work_rows.map(|rows| keep(true, rows));
    let mut merged: Vec<FuzzyPair> = merge_tiers(resolver, global_rows, work_rows)?
        .into_iter()
        .map(|pair| {
            let percent = scores.get(&(pair.tier == TmTier::Work, pair.id)).copied().unwrap_or(0);
            FuzzyPair { pair, percent }
        })
        .collect();
    merged.sort_by(|a, b| b.percent.cmp(&a.percent));
    merged.truncate(FUZZY_MATCH_LIMIT);
    Ok(merged)
}

/// Most hits one Concordance search ships; the total is reported beside them.
pub const CONCORDANCE_HIT_LIMIT: usize = 50;

/// Every pair of both tiers, read once so the substring filter can run after the caller released its locks.
pub struct ConcordanceCandidates {
    global_rows: Vec<RawPair>,
    work_rows: Option<Vec<RawPair>>,
}

pub fn load_concordance_candidates(
    global: &crate::core::store::Store,
    work: Option<&crate::core::store::Store>,
) -> Result<ConcordanceCandidates, TmStoreError> {
    Ok(ConcordanceCandidates {
        global_rows: load_all_pair_rows(global)?,
        work_rows: work.map(load_all_pair_rows).transpose()?,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcordanceHits {
    /// Both tiers hold zero rows, as opposed to rows that simply do not contain the phrase.
    pub tm_empty: bool,
    /// Matching pairs before the cap.
    pub total: usize,
    /// At most [`CONCORDANCE_HIT_LIMIT`], AD-18 order.
    pub hits: Vec<TmPair>,
}

fn concordance_key(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    text.trim().nfc().collect::<String>().to_lowercase()
}

/// Pairs of both tiers whose source contains `query` as a raw substring: NFC and lower-cased on
/// both sides, no stemming, matches inside words. A blank query hits nothing.
pub fn rank_concordance(
    resolver: &crate::core::scope::ScopeResolver,
    candidates: ConcordanceCandidates,
    query: &str,
) -> Result<ConcordanceHits, TmStoreError> {
    let tm_empty = candidates.global_rows.is_empty()
        && candidates.work_rows.as_ref().is_none_or(Vec::is_empty);
    let needle = concordance_key(query);
    if needle.is_empty() {
        return Ok(ConcordanceHits { tm_empty, total: 0, hits: Vec::new() });
    }
    let keep = |rows: Vec<RawPair>| -> Vec<RawPair> {
        rows.into_iter().filter(|row| concordance_key(&row.source_text).contains(&needle)).collect()
    };
    let global_rows = keep(candidates.global_rows);
    let work_rows = candidates.work_rows.map(keep);
    let mut hits = merge_tiers(resolver, global_rows, work_rows)?;
    let total = hits.len();
    hits.truncate(CONCORDANCE_HIT_LIMIT);
    Ok(ConcordanceHits { tm_empty, total, hits })
}

/// One pair of one tier by id; `None` when it no longer exists.
pub fn pair_by_id(
    store: &crate::core::store::Store,
    tier: TmTier,
    id: i64,
) -> Result<Option<TmPair>, TmStoreError> {
    let raw: Option<(i64, String, String, String)> = store.read(move |conn| {
        match conn.query_row(
            "SELECT id, source_text, target_text, translation_origin FROM tm_unit WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        ) {
            Ok(row) => Ok(Some(row)),
            Err(crate::core::store::SqlError::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err),
        }
    })?;
    raw.map(|(id, source_text, target_text, origin)| {
        let translation_origin = PairOrigin::from_stored(&origin)
            .ok_or(TmStoreError::UnknownOrigin { value: origin })?;
        Ok(TmPair { id, source_text, target_text, translation_origin, tier })
    })
    .transpose()
}
