//! Vietnamese writing conventions (FR159, AD-67): rules, the matcher over stored text and the
//! case transfer. Pure; it never imports `core::ai`, and `core::ai` never imports it.

mod casing;
mod exchange;
mod ignore;
mod matcher;
mod normalize;
mod resolve;
mod rule;
mod splice;
mod store;

pub use casing::transfer_case;
pub use exchange::{
    ConflictDecision, ImportRow, ImportSummary, MAX_CONVENTION_IMPORT_BYTES, RowPlan, RowPlanKind,
    apply_import, classify as classify_import, parse as parse_import, render_tier, visible_rows,
};
pub use ignore::{
    CONVENTION_CONTEXT_CHARS, ConventionSignature, filter_ignored, signature_matches, signature_of,
};
pub use matcher::{Hit, Matcher, Scan, phrase_spans};
pub use normalize::rule_key;
pub use resolve::{RuleStatus, effective_row, resolve_two_tiers, row_status};
pub use rule::{Rule, RuleTier};
pub use splice::apply_spans;
pub use store::{
    ConventionStoreError, IgnoreRow, RuleRow, add_ignore, add_rule, clear_ignores, delete_rule,
    disable_in_work, edit_rule, enable_in_work, find_active_rule, find_row_by_key,
    list_ignore_rows, list_ignores, list_rules, list_rules_in, remove_ignore,
};
