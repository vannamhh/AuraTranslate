//! CSV/TSV exchange of one convention tier (FR159, AD-67 rule 10, 12b): two columns `avoid`,
//! `replacement`; an empty second cell is a rule with no replacement. Pure text in and out; the
//! file and the dialog belong to `commands::convention`.

use std::collections::BTreeMap;

use crate::core::delimited_text::{
    DelimitedError, Delimiter, count_line_breaks, logical_lines, render_field, split_fields,
    split_first_logical_line, strip_bom, strip_formula_guard, strip_zero_width,
    unquoted_char_present,
};
use crate::core::store::{Store, Transaction};

use super::normalize::rule_key;
use super::rule::RuleTier;
use super::store::{ConventionStoreError, RuleRow, list_rules_in};

pub const MAX_CONVENTION_IMPORT_BYTES: u64 = 1024 * 1024;

const COLUMN_AVOID: &str = "avoid";
const COLUMN_REPLACEMENT: &str = "replacement";

/// One data row of an import file, already trimmed and guard-stripped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRow {
    pub rule_key: String,
    pub avoid_phrase: String,
    pub replacement_phrase: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowPlanKind {
    New,
    Identical,
    Conflict {
        existing_avoid_phrase: String,
        existing_replacement_phrase: Option<String>,
    },
}

/// A file row classified against the tier as the list shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowPlan {
    pub rule_key: String,
    pub avoid_phrase: String,
    pub replacement_phrase: Option<String>,
    pub kind: RowPlanKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum ConflictDecision {
    #[serde(rename = "keep_mine")]
    KeepMine,
    #[serde(rename = "take_file")]
    TakeFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportSummary {
    pub added: u32,
    pub replaced: u32,
    pub kept: u32,
}

/// The rows of `tier` as the management list shows them: a Work row that switches a Global rule
/// off is a flag, not a rule, so it is not part of the tier.
pub fn visible_rows(tier: RuleTier, rows: Vec<RuleRow>) -> Vec<RuleRow> {
    match tier {
        RuleTier::Global => rows,
        RuleTier::Work => rows.into_iter().filter(|row| row.enabled).collect(),
    }
}

/// The header plus one line per visible row, `\n`-terminated; an empty tier still has its header.
pub fn render_tier(rows: &[RuleRow], delimiter: Delimiter) -> String {
    let d = delimiter.as_char();
    let sep = d.to_string();
    let mut out = [COLUMN_AVOID, COLUMN_REPLACEMENT].join(&sep);
    out.push('\n');
    for row in rows {
        out.push_str(&render_field(&row.avoid_phrase, d));
        out.push_str(&sep);
        out.push_str(&render_field(
            row.replacement_phrase.as_deref().unwrap_or(""),
            d,
        ));
        out.push('\n');
    }
    out
}

fn cell_text(cell: &str) -> String {
    let cleaned = strip_zero_width(cell);
    strip_formula_guard(cleaned.trim()).to_owned()
}

fn unterminated(issue: DelimitedError) -> ConventionStoreError {
    match issue {
        DelimitedError::UnterminatedQuotedField { line } => {
            ConventionStoreError::ImportUnterminatedQuotedField { line }
        }
    }
}

/// Parses a whole file. The first problem found rejects it: nothing partial is returned.
pub fn parse(text: &str) -> Result<Vec<ImportRow>, ConventionStoreError> {
    let text = strip_bom(text);
    if text.trim().is_empty() {
        return Err(ConventionStoreError::ImportMissingColumn {
            column: COLUMN_AVOID,
        });
    }

    let (header_text, body_text, header_unterminated) = split_first_logical_line(text, None);
    if header_unterminated {
        return Err(ConventionStoreError::ImportUnterminatedQuotedField { line: 1 });
    }
    let delimiter = match (
        unquoted_char_present(header_text, ','),
        unquoted_char_present(header_text, '\t'),
    ) {
        (true, false) => Delimiter::Csv,
        (false, true) => Delimiter::Tsv,
        _ => return Err(ConventionStoreError::ImportDelimiterUnresolved),
    };
    let d = delimiter.as_char();

    let header: Vec<String> = split_fields(header_text, d)
        .iter()
        .map(|cell| cell.trim().to_owned())
        .collect();
    let index_of = |column: &'static str| -> Result<usize, ConventionStoreError> {
        let mut found = header
            .iter()
            .enumerate()
            .filter(|(_, name)| name.as_str() == column);
        let first = found
            .next()
            .ok_or(ConventionStoreError::ImportMissingColumn { column })?;
        if found.next().is_some() {
            return Err(ConventionStoreError::ImportDuplicateColumn { column });
        }
        Ok(first.0)
    };
    let avoid_idx = index_of(COLUMN_AVOID)?;
    let replacement_idx = index_of(COLUMN_REPLACEMENT)?;

    let first_body_line = 1 + count_line_breaks(header_text) + 1;
    let lines = logical_lines(body_text, Some(d), first_body_line).map_err(unterminated)?;

    let mut rows = Vec::new();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (line, raw) in lines {
        if raw.trim().is_empty() {
            continue;
        }
        let cells = split_fields(raw, d);
        if cells.len() != header.len() {
            return Err(ConventionStoreError::ImportCellCountMismatch {
                line,
                expected: header.len(),
                found: cells.len(),
            });
        }
        let avoid = cell_text(&cells[avoid_idx]);
        if avoid.is_empty() {
            return Err(ConventionStoreError::ImportBlankAvoidPhrase { line });
        }
        let key = rule_key(&avoid);
        if let Some(&first_line) = seen.get(&key) {
            return Err(ConventionStoreError::ImportDuplicateAvoidPhrase {
                first_line,
                second_line: line,
            });
        }
        seen.insert(key.clone(), line);
        let replacement = Some(cell_text(&cells[replacement_idx])).filter(|text| !text.is_empty());
        rows.push(ImportRow {
            rule_key: key,
            avoid_phrase: avoid,
            replacement_phrase: replacement,
        });
    }
    Ok(rows)
}

fn kind_against(
    avoid: &str,
    replacement: &Option<String>,
    existing: Option<&RuleRow>,
) -> RowPlanKind {
    match existing {
        None => RowPlanKind::New,
        Some(row) if row.avoid_phrase == avoid && row.replacement_phrase == *replacement => {
            RowPlanKind::Identical
        }
        Some(row) => RowPlanKind::Conflict {
            existing_avoid_phrase: row.avoid_phrase.clone(),
            existing_replacement_phrase: row.replacement_phrase.clone(),
        },
    }
}

fn by_key(rows: &[RuleRow]) -> BTreeMap<&str, &RuleRow> {
    rows.iter()
        .map(|row| (row.rule_key.as_str(), row))
        .collect()
}

/// Classifies each row against `visible`, the rows of the target tier as the list shows them.
pub fn classify(rows: &[ImportRow], visible: &[RuleRow]) -> Vec<RowPlan> {
    let existing = by_key(visible);
    rows.iter()
        .map(|row| RowPlan {
            rule_key: row.rule_key.clone(),
            avoid_phrase: row.avoid_phrase.clone(),
            replacement_phrase: row.replacement_phrase.clone(),
            kind: kind_against(
                &row.avoid_phrase,
                &row.replacement_phrase,
                existing.get(row.rule_key.as_str()).copied(),
            ),
        })
        .collect()
}

fn apply_in(
    tx: &Transaction<'_>,
    tier: RuleTier,
    plans: &[RowPlan],
    decisions: &BTreeMap<String, ConflictDecision>,
) -> crate::core::store::SqlResult<Result<ImportSummary, ConventionStoreError>> {
    let all = list_rules_in(tx)?;
    let stored = by_key(&all);
    let visible: Vec<RuleRow> = visible_rows(tier, all.clone());
    let live = by_key(&visible);

    let mut summary = ImportSummary {
        added: 0,
        replaced: 0,
        kept: 0,
    };
    let mut writes: Vec<(&RowPlan, Option<i64>)> = Vec::new();
    for plan in plans {
        let now = kind_against(
            &plan.avoid_phrase,
            &plan.replacement_phrase,
            live.get(plan.rule_key.as_str()).copied(),
        );
        if now != plan.kind {
            return Ok(Err(ConventionStoreError::ImportStale));
        }
        match plan.kind {
            RowPlanKind::New => {
                summary.added += 1;
                writes.push((plan, stored.get(plan.rule_key.as_str()).map(|row| row.id)));
            }
            RowPlanKind::Identical => summary.kept += 1,
            RowPlanKind::Conflict { .. } => {
                if decisions.get(&plan.rule_key) == Some(&ConflictDecision::TakeFile) {
                    summary.replaced += 1;
                    writes.push((plan, stored.get(plan.rule_key.as_str()).map(|row| row.id)));
                } else {
                    summary.kept += 1;
                }
            }
        }
    }

    for (plan, id) in writes {
        match id {
            Some(id) => {
                tx.execute(
                    "UPDATE convention_rule SET avoid_phrase = ?1, replacement_phrase = ?2, \
                     enabled = 1 WHERE id = ?3",
                    (&plan.avoid_phrase, &plan.replacement_phrase, id),
                )?;
            }
            None => {
                tx.execute(
                    "INSERT INTO convention_rule \
                     (avoid_phrase, replacement_phrase, rule_key, enabled, created_at) \
                     VALUES (?1, ?2, ?3, 1, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                    (&plan.avoid_phrase, &plan.replacement_phrase, &plan.rule_key),
                )?;
            }
        }
    }
    Ok(Ok(summary))
}

/// Applies `plans` to `store` in one transaction. A decision whose key is not a conflict of the
/// plan is refused before the transaction opens; a plan that no longer matches the tier inside
/// the transaction is `ImportStale`. Either way nothing is written.
pub fn apply_import(
    store: &Store,
    tier: RuleTier,
    plans: &[RowPlan],
    decisions: &BTreeMap<String, ConflictDecision>,
) -> Result<ImportSummary, ConventionStoreError> {
    for key in decisions.keys() {
        let is_conflict = plans
            .iter()
            .any(|plan| plan.rule_key == *key && matches!(plan.kind, RowPlanKind::Conflict { .. }));
        if !is_conflict {
            return Err(ConventionStoreError::ImportDecisionUnknownRule {
                rule_key: key.clone(),
            });
        }
    }
    let (plans, decisions) = (plans.to_vec(), decisions.clone());
    store.write(move |tx: &Transaction<'_>| apply_in(tx, tier, &plans, &decisions))?
}
