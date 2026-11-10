use std::collections::BTreeMap;

use super::rule::{Rule, RuleTier};
use super::store::RuleRow;

/// How the management list shows one stored rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleStatus {
    InUse,
    Shadowed,
    DisabledInWork,
}

impl RuleStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InUse => "in_use",
            Self::Shadowed => "shadowed",
            Self::DisabledInWork => "disabled_in_work",
        }
    }
}

/// The rules in force for the open Work. A Work row wins over the Global row with the same key
/// whether or not it is enabled; that is the only way a Global rule is switched off for one Work.
pub fn resolve_two_tiers(global: &[RuleRow], work: &[RuleRow]) -> Vec<Rule> {
    let mut by_key: BTreeMap<&str, (RuleTier, &RuleRow)> = BTreeMap::new();
    for row in global {
        by_key.insert(row.rule_key.as_str(), (RuleTier::Global, row));
    }
    for row in work {
        by_key.insert(row.rule_key.as_str(), (RuleTier::Work, row));
    }
    by_key
        .into_values()
        .filter(|(_, row)| row.enabled)
        .map(|(tier, row)| row.clone().into_rule(tier))
        .collect()
}

/// The status of a stored row given the other tier. `tier` is the tier `row` came from.
pub fn row_status(tier: RuleTier, row: &RuleRow, work: &[RuleRow]) -> RuleStatus {
    match tier {
        RuleTier::Global => match work.iter().find(|w| w.rule_key == row.rule_key) {
            Some(w) if w.enabled => RuleStatus::Shadowed,
            Some(_) => RuleStatus::DisabledInWork,
            None => RuleStatus::InUse,
        },
        RuleTier::Work => RuleStatus::InUse,
    }
}

/// The enabled row that wins for `key` across both tiers, or `None` when the rule is gone or
/// switched off. Uses the same precedence as [`resolve_two_tiers`].
pub fn effective_row(key: &str, global: &[RuleRow], work: &[RuleRow]) -> Option<RuleRow> {
    let winner = work
        .iter()
        .find(|row| row.rule_key == key)
        .or_else(|| global.iter().find(|row| row.rule_key == key))?;
    winner.enabled.then(|| winner.clone())
}
