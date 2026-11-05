//! Vietnamese writing conventions (FR159, AD-67): rules, the matcher over stored text and the
//! case transfer. Pure; it never imports `core::ai`, and `core::ai` never imports it.

mod casing;
mod matcher;
mod normalize;
mod rule;
mod store;

pub use casing::transfer_case;
pub use matcher::{Hit, Matcher, Scan, phrase_spans};
pub use normalize::rule_key;
pub use rule::{Rule, RuleTier};
pub use store::{ConventionStoreError, RuleRow, add_rule, find_active_rule, list_rules};
