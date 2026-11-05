/// The store a rule lives in. The Work tier arrives with the management screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleTier {
    Global,
}

impl RuleTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
        }
    }
}

/// An enabled rule, ready to build a matcher from. `key` is the avoid phrase after NFC and
/// Unicode lowercase (AD-67 rule 2); `i`/`y` are never merged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub key: String,
    pub avoid_phrase: String,
    pub replacement_phrase: Option<String>,
    pub tier: RuleTier,
}
