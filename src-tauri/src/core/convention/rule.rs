/// The store a rule lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleTier {
    Global,
    Work,
}

impl RuleTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Work => "work",
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
