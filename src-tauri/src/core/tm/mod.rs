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
    source_text: String,
    target_text: String,
    translation_origin: PairOrigin,
}

fn load_pair_rows(
    store: &crate::core::store::Store,
    source_text: &str,
) -> Result<Vec<RawPair>, TmStoreError> {
    let source = source_text.to_owned();
    let raw: Vec<(String, String, String)> = store.read(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT source_text, target_text, translation_origin FROM tm_unit \
             WHERE source_text = ?1 ORDER BY id",
        )?;
        let rows = stmt
            .query_map([&source], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<crate::core::store::SqlResult<Vec<_>>>()?;
        Ok(rows)
    })?;
    raw.into_iter()
        .map(|(source_text, target_text, origin)| {
            let translation_origin = PairOrigin::from_stored(&origin)
                .ok_or(TmStoreError::UnknownOrigin { value: origin })?;
            Ok(RawPair { source_text, target_text, translation_origin })
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
                source_text: raw.source_text.clone(),
                target_text: raw.target_text.clone(),
                translation_origin: raw.translation_origin,
                tier,
            }
        })
        .collect())
}
