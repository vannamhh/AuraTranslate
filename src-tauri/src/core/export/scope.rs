use std::collections::BTreeSet;

use crate::core::segment::omit::IN_TRANSLATION_SQL;
use crate::core::store::{Store, StoreError};

/// Phạm vi xuất (FR89): `Chapters` mang một hoặc nhiều Chương đã chọn; `Work` là mọi Chương
/// hiện có. Mọi định dạng xuất nhận phạm vi qua kiểu này và đếm qua [`count_scope`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExportScope {
    Work,
    Chapters { chapter_ids: Vec<i64> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct ScopeCounts {
    pub chapter_count: i64,
    /// Câu còn sống và thuộc bản dịch (không về hưu, không bị cắt bỏ).
    pub segment_count: i64,
    /// Trong `segment_count`, số câu chưa ở trạng thái `confirmed`.
    pub unconfirmed_count: i64,
}

#[derive(Debug)]
pub enum ScopeError {
    Empty,
    ChapterNotFound(i64),
    Read(StoreError),
}

impl From<StoreError> for ScopeError {
    fn from(err: StoreError) -> Self {
        ScopeError::Read(err)
    }
}

/// Chương của phạm vi, theo `(ord, id)`, không trùng.
pub fn resolve_chapter_ids(store: &Store, scope: &ExportScope) -> Result<Vec<i64>, ScopeError> {
    let existing: Vec<i64> = store.read(|conn| {
        let mut stmt = conn.prepare("SELECT id FROM chapter ORDER BY ord, id")?;
        let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
        rows.collect()
    })?;
    match scope {
        ExportScope::Work => {
            if existing.is_empty() {
                return Err(ScopeError::Empty);
            }
            Ok(existing)
        }
        ExportScope::Chapters { chapter_ids } => {
            if chapter_ids.is_empty() {
                return Err(ScopeError::Empty);
            }
            let wanted: BTreeSet<i64> = chapter_ids.iter().copied().collect();
            if let Some(missing) = wanted.iter().find(|id| !existing.contains(id)) {
                return Err(ScopeError::ChapterNotFound(*missing));
            }
            Ok(existing.into_iter().filter(|id| wanted.contains(id)).collect())
        }
    }
}

pub fn count_scope(store: &Store, scope: &ExportScope) -> Result<ScopeCounts, ScopeError> {
    let chapter_ids = resolve_chapter_ids(store, scope)?;
    let (segment_count, unconfirmed_count) = store.read(|conn| {
        let mut stmt = conn.prepare(&format!(
            "SELECT COUNT(*), COALESCE(SUM(status <> 'confirmed'), 0) FROM segment \
             WHERE chapter_id = ?1 AND retired_at IS NULL AND {IN_TRANSLATION_SQL}"
        ))?;
        let mut total = 0_i64;
        let mut unconfirmed = 0_i64;
        for id in &chapter_ids {
            let (n, u): (i64, i64) = stmt.query_row([id], |row| Ok((row.get(0)?, row.get(1)?)))?;
            total += n;
            unconfirmed += u;
        }
        Ok((total, unconfirmed))
    })?;
    Ok(ScopeCounts {
        chapter_count: i64::try_from(chapter_ids.len()).unwrap_or(i64::MAX),
        segment_count,
        unconfirmed_count,
    })
}
