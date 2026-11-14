//! Quick-lookup groups for the Mod+K palette (AD-69): pinyin and Vietnamese-to-Chinese.
//!
//! The Chinese-to-Vietnamese and English-to-Vietnamese groups reuse the panel Lookup path in
//! `commands/dict.rs`. Every group carries one of four named states, never a bare empty list.

use std::collections::{BTreeMap, BTreeSet};

use crate::core::diacritics::{SearchMode, classify_query, fold_diacritics_case_preserving};
use crate::core::i18n::IpcError;
use crate::core::store::{ReadHandle, SqlResult, ToSql};

use super::DictLayers;
use super::pinyin::pinyin_key;
use super::syllables::could_be_vietnamese;

pub const QUICK_GROUP_ROWS: usize = 20;

const FETCH_ROWS: i64 = QUICK_GROUP_ROWS as i64 + 1;

const NEWEST_COLUMNS_SCHEMA_VERSION: u32 = 4;

const WHOLE_CEILING: i64 = 200;

const WHOLE_GLOSS_SLACK: i64 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum QuickGroupId {
    #[serde(rename = "pinyin")]
    Pinyin,
    #[serde(rename = "vi_zh")]
    VietnameseToChinese,
    #[serde(rename = "en_vi")]
    EnglishToVietnamese,
    #[serde(rename = "zh_vi")]
    ChineseToVietnamese,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum QuickStatus {
    #[serde(rename = "rows")]
    Rows,
    #[serde(rename = "empty")]
    Empty,
    #[serde(rename = "outdated_dictionary")]
    OutdatedDictionary,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct QuickRow {
    pub headword: String,
    pub headword_simp: Option<String>,
    pub reading: Option<String>,
    pub gloss: Option<String>,
    pub source_code: String,
    pub source_name: String,
    pub source_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct QuickGroup {
    pub id: QuickGroupId,
    pub status: QuickStatus,
    pub rows: Vec<QuickRow>,
    pub has_more: bool,
    pub incomplete: bool,
    pub error: Option<IpcError>,
}

#[derive(Default)]
pub struct LayerTally {
    outdated: usize,
    error: Option<IpcError>,
}

impl QuickGroup {
    pub(crate) fn settle(
        id: QuickGroupId,
        rows: Vec<QuickRow>,
        has_more: bool,
        tally: LayerTally,
    ) -> QuickGroup {
        let incomplete = tally.outdated > 0 || tally.error.is_some();
        let (status, error) = if !rows.is_empty() {
            (QuickStatus::Rows, None)
        } else if let Some(error) = tally.error {
            (QuickStatus::Error, Some(error))
        } else if tally.outdated > 0 {
            (QuickStatus::OutdatedDictionary, None)
        } else {
            (QuickStatus::Empty, None)
        };
        QuickGroup {
            id,
            status,
            rows,
            has_more,
            incomplete: incomplete && status == QuickStatus::Rows,
            error,
        }
    }

    pub(crate) fn from_rows(id: QuickGroupId, rows: Vec<QuickRow>, has_more: bool) -> QuickGroup {
        QuickGroup::settle(id, rows, has_more, LayerTally::default())
    }
}

fn disabled_json(disabled: &BTreeSet<String>) -> String {
    serde_json::to_string(disabled).unwrap_or_else(|_| "[]".to_owned())
}

fn phrase(text: &str) -> String {
    format!("\"{}\"", text.replace('"', "\"\""))
}

// ─────────────────────────────────────────────────────────────────────────────
// Pinyin
// ─────────────────────────────────────────────────────────────────────────────

const PINYIN_EXACT_SQL: &str = "\
SELECT e.headword, COUNT(DISTINCT s.code), \
       COALESCE(MIN(CASE WHEN s.gloss_lang = 'vi' THEN e.id END), MIN(e.id)) \
FROM dict_entry e JOIN dict_source s ON s.id = e.source_id \
WHERE e.reading_key = ?1 AND e.lang = 'zh' \
  AND s.code NOT IN (SELECT value FROM json_each(?2)) \
GROUP BY e.headword ORDER BY length(e.headword), e.headword LIMIT ?3";

const PINYIN_PREFIX_SQL: &str = "\
SELECT e.headword, COUNT(DISTINCT s.code), \
       COALESCE(MIN(CASE WHEN s.gloss_lang = 'vi' THEN e.id END), MIN(e.id)) \
FROM dict_entry e JOIN dict_source s ON s.id = e.source_id \
WHERE e.reading_key > ?1 AND e.reading_key < ?2 AND e.lang = 'zh' \
  AND s.code NOT IN (SELECT value FROM json_each(?3)) \
GROUP BY e.headword ORDER BY length(e.headword), e.headword LIMIT ?4";

const ENTRY_ROW_SQL: &str = "\
SELECT e.headword_simp, e.reading, s.code, s.display_name, \
       (SELECT d.gloss FROM dict_sense d WHERE d.entry_id = e.id ORDER BY d.ord, d.id LIMIT 1) \
FROM dict_entry e JOIN dict_source s ON s.id = e.source_id WHERE e.id = ?1";

struct PinyinHit {
    headword: String,
    tier: u8,
    source_count: i64,
    layer_index: usize,
    entry_id: i64,
}

fn pinyin_tier(
    db: ReadHandle<'_>,
    sql: &str,
    params: &[&dyn ToSql],
    tier: u8,
    layer_index: usize,
) -> SqlResult<Vec<PinyinHit>> {
    let mut stmt = db.prepare_cached(sql)?;
    let rows = stmt.query_map(params, |row| {
        Ok(PinyinHit {
            headword: row.get(0)?,
            tier,
            source_count: row.get(1)?,
            layer_index,
            entry_id: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn pinyin_group(layers: &DictLayers, query: &str, disabled: &BTreeSet<String>) -> QuickGroup {
    let id = QuickGroupId::Pinyin;
    let Some(key) = pinyin_key(query) else {
        return QuickGroup::from_rows(id, Vec::new(), false);
    };
    let upper = format!("{key}{{");
    let excluded = disabled_json(disabled);

    let mut tally = LayerTally::default();
    let mut by_headword: BTreeMap<String, PinyinHit> = BTreeMap::new();
    for (layer_index, layer) in layers.layers().iter().enumerate() {
        if layer.schema_version() < NEWEST_COLUMNS_SCHEMA_VERSION {
            tally.outdated += 1;
            continue;
        }
        let found = layer.read(|db| {
            let mut hits = pinyin_tier(
                db,
                PINYIN_EXACT_SQL,
                &[&key, &excluded, &FETCH_ROWS],
                0,
                layer_index,
            )?;
            if hits.len() < FETCH_ROWS as usize {
                hits.extend(pinyin_tier(
                    db,
                    PINYIN_PREFIX_SQL,
                    &[&key, &upper, &excluded, &FETCH_ROWS],
                    1,
                    layer_index,
                )?);
            }
            Ok(hits)
        });
        match found {
            Ok(hits) => {
                for hit in hits {
                    match by_headword.get_mut(&hit.headword) {
                        Some(known) => {
                            known.source_count += hit.source_count;
                            if (hit.tier, hit.layer_index) < (known.tier, known.layer_index) {
                                known.tier = hit.tier;
                                known.layer_index = hit.layer_index;
                                known.entry_id = hit.entry_id;
                            }
                        }
                        None => {
                            by_headword.insert(hit.headword.clone(), hit);
                        }
                    }
                }
            }
            Err(err) => tally.error = tally.error.or(Some(err.into())),
        }
    }

    let mut ranked: Vec<PinyinHit> = by_headword.into_values().collect();
    ranked.sort_by(|a, b| {
        (a.tier, a.headword.chars().count(), &a.headword).cmp(&(
            b.tier,
            b.headword.chars().count(),
            &b.headword,
        ))
    });
    let has_more = ranked.len() > QUICK_GROUP_ROWS;
    ranked.truncate(QUICK_GROUP_ROWS);

    let mut rows = Vec::with_capacity(ranked.len());
    for hit in ranked {
        let layer = &layers.layers()[hit.layer_index];
        match layer.read(|db| read_entry_row(db, hit.entry_id)) {
            Ok(Some((headword_simp, reading, source_code, source_name, gloss))) => {
                rows.push(QuickRow {
                    headword: hit.headword,
                    headword_simp,
                    reading,
                    gloss,
                    source_code,
                    source_name,
                    source_count: u32::try_from(hit.source_count).unwrap_or(u32::MAX),
                });
            }
            Ok(None) => {}
            Err(err) => tally.error = tally.error.or(Some(err.into())),
        }
    }
    QuickGroup::settle(id, rows, has_more, tally)
}

type EntryRow = (
    Option<String>,
    Option<String>,
    String,
    String,
    Option<String>,
);

fn read_entry_row(db: ReadHandle<'_>, entry_id: i64) -> SqlResult<Option<EntryRow>> {
    let mut stmt = db.prepare_cached(ENTRY_ROW_SQL)?;
    let mut rows = stmt.query([entry_id])?;
    match rows.next()? {
        Some(row) => Ok(Some((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))),
        None => Ok(None),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Vietnamese to Chinese
// ─────────────────────────────────────────────────────────────────────────────

const SENSE_COLUMNS: &str = "d.id, e.headword, e.headword_simp, e.reading, s.code, \
    s.display_name, d.gloss";

/// No `ORDER BY` on purpose: the FTS index already yields rowids in ascending order and stops at `LIMIT`.
fn gloss_sql(fts_table: &str, tail: &str) -> String {
    format!(
        "SELECT {SENSE_COLUMNS} FROM {fts_table} f \
         JOIN dict_sense d ON d.id = f.rowid \
         JOIN dict_entry e ON e.id = d.entry_id \
         JOIN dict_source s ON s.id = d.source_id \
         WHERE {fts_table} MATCH ?1 AND e.lang = 'zh' AND s.gloss_lang = 'vi' \
           AND s.code NOT IN (SELECT value FROM json_each(?2)) {tail}"
    )
}

struct GlossHit {
    sense_id: i64,
    headword: String,
    headword_simp: Option<String>,
    reading: Option<String>,
    source_code: String,
    source_name: String,
    gloss: String,
}

fn gloss_hits(db: ReadHandle<'_>, sql: &str, params: &[&dyn ToSql]) -> SqlResult<Vec<GlossHit>> {
    let mut stmt = db.prepare_cached(sql)?;
    let rows = stmt.query_map(params, |row| {
        Ok(GlossHit {
            sense_id: row.get(0)?,
            headword: row.get(1)?,
            headword_simp: row.get(2)?,
            reading: row.get(3)?,
            source_code: row.get(4)?,
            source_name: row.get(5)?,
            gloss: row.get(6)?,
        })
    })?;
    rows.collect()
}

fn normalized(text: &str, mode: SearchMode) -> String {
    let lowered = text.trim().to_lowercase();
    match mode {
        SearchMode::Lenient => fold_diacritics_case_preserving(&lowered),
        SearchMode::Exact => lowered,
    }
}

/// 0 = the whole gloss, 1 = gloss starts with the query at a word boundary, 2 = anything else.
fn gloss_rank(gloss: &str, query: &str, mode: SearchMode) -> u8 {
    let folded = normalized(gloss, mode);
    if folded == query {
        return 0;
    }
    match folded.strip_prefix(query) {
        Some(rest) if rest.chars().next().is_none_or(|c| !c.is_alphanumeric()) => 1,
        _ => 2,
    }
}

pub fn vietnamese_to_chinese_group(
    layers: &DictLayers,
    query: &str,
    disabled: &BTreeSet<String>,
) -> QuickGroup {
    let id = QuickGroupId::VietnameseToChinese;
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return QuickGroup::from_rows(id, Vec::new(), false);
    }
    if !could_be_vietnamese(trimmed) {
        return QuickGroup::from_rows(id, Vec::new(), false);
    }
    let mode = classify_query(trimmed);
    let wanted = normalized(trimmed, mode);
    let plain = trimmed.to_lowercase();
    let fts_table = match mode {
        SearchMode::Lenient => "sense_fts_nd",
        SearchMode::Exact => "sense_fts",
    };
    let whole_sql = gloss_sql(fts_table, "AND length(d.gloss) BETWEEN ?4 AND ?5 LIMIT ?3");
    let anchored_sql = gloss_sql(fts_table, "LIMIT ?3");
    let plain_sql = gloss_sql(fts_table, "LIMIT ?3");
    let shortest = trimmed.chars().count() as i64;
    let longest = shortest + WHOLE_GLOSS_SLACK;
    let anchored_match = format!("gloss : ^{}", phrase(trimmed));
    let plain_match = format!("gloss : {}", phrase(trimmed));
    let excluded = disabled_json(disabled);

    let mut tally = LayerTally::default();
    let mut cut = false;
    let mut hits: Vec<((u8, u8), GlossHit)> = Vec::new();
    for layer in layers.layers() {
        if layer.schema_version() < NEWEST_COLUMNS_SCHEMA_VERSION {
            tally.outdated += 1;
            continue;
        }
        let found = layer.read(|db| {
            let mut seen: BTreeSet<i64> = BTreeSet::new();
            let mut found: Vec<GlossHit> = Vec::new();
            let whole = gloss_hits(
                db,
                &whole_sql,
                &[
                    &anchored_match,
                    &excluded,
                    &WHOLE_CEILING,
                    &shortest,
                    &longest,
                ],
            )?;
            seen.extend(whole.iter().map(|hit| hit.sense_id));
            found.extend(whole);
            let anchored_limit = FETCH_ROWS + seen.len() as i64;
            let anchored = gloss_hits(
                db,
                &anchored_sql,
                &[&anchored_match, &excluded, &anchored_limit],
            )?;
            for hit in anchored {
                if seen.insert(hit.sense_id) {
                    found.push(hit);
                }
            }
            let plain_limit = FETCH_ROWS + seen.len() as i64;
            let plain = gloss_hits(db, &plain_sql, &[&plain_match, &excluded, &plain_limit])?;
            let plain_cut = plain.len() as i64 >= plain_limit;
            let fresh = plain
                .into_iter()
                .filter(|hit| !seen.contains(&hit.sense_id));
            found.extend(fresh);
            Ok((found, plain_cut))
        });
        match found {
            Ok((layer_hits, layer_cut)) => {
                cut |= layer_cut;
                hits.extend(layer_hits.into_iter().map(|hit| {
                    let unfolded = u8::from(!hit.gloss.trim().to_lowercase().starts_with(&plain));
                    ((gloss_rank(&hit.gloss, &wanted, mode), unfolded), hit)
                }));
            }
            Err(err) => tally.error = tally.error.or(Some(err.into())),
        }
    }

    hits.sort_by(|(rank_a, a), (rank_b, b)| {
        (rank_a, a.gloss.chars().count(), &a.headword, a.sense_id).cmp(&(
            rank_b,
            b.gloss.chars().count(),
            &b.headword,
            b.sense_id,
        ))
    });

    let mut order: Vec<String> = Vec::new();
    let mut by_headword: BTreeMap<String, (GlossHit, BTreeSet<String>)> = BTreeMap::new();
    for (_, hit) in hits {
        match by_headword.get_mut(&hit.headword) {
            Some((_, sources)) => {
                sources.insert(hit.source_code.clone());
            }
            None => {
                order.push(hit.headword.clone());
                let sources = BTreeSet::from([hit.source_code.clone()]);
                by_headword.insert(hit.headword.clone(), (hit, sources));
            }
        }
    }
    let has_more = order.len() > QUICK_GROUP_ROWS || cut;
    let rows: Vec<QuickRow> = order
        .into_iter()
        .take(QUICK_GROUP_ROWS)
        .filter_map(|headword| by_headword.remove(&headword))
        .map(|(hit, sources)| QuickRow {
            headword: hit.headword,
            headword_simp: hit.headword_simp,
            reading: hit.reading,
            gloss: Some(hit.gloss),
            source_code: hit.source_code,
            source_name: hit.source_name,
            source_count: u32::try_from(sources.len()).unwrap_or(u32::MAX),
        })
        .collect();
    QuickGroup::settle(id, rows, has_more, tally)
}
