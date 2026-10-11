//! The closed set of proofreader finding kinds (AD-53 rule 1).
//!
//! `core::ai` assigns a kind from the model's label; consumers outside it read the kind here.

use serde::{Deserialize, Serialize};

/// How a finding is identified for "not an error" memory (AD-53 rule 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureScope {
    Phrase,
    PhraseAndContext,
}

/// Which origin accepting or reverting a suggestion declares (AD-53 rule 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptanceOrigin {
    ArbitratedBeforeWrite,
    OtherTranslated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofreadKind {
    Spelling,
    Grammar,
    Mistranslation,
    TooLoose,
    Obscure,
}

impl ProofreadKind {
    pub const ALL: [Self; 5] =
        [Self::Spelling, Self::Grammar, Self::Mistranslation, Self::TooLoose, Self::Obscure];

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Spelling => "spelling",
            Self::Grammar => "grammar",
            Self::Mistranslation => "mistranslation",
            Self::TooLoose => "too_loose",
            Self::Obscure => "obscure",
        }
    }

    #[must_use]
    pub fn from_wire(label: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == label)
    }

    /// Kinds judged against the source text rather than the translation alone.
    #[must_use]
    pub fn needs_source(self) -> bool {
        !matches!(self, Self::Spelling | Self::Grammar)
    }

    #[must_use]
    pub fn signature_scope(self) -> SignatureScope {
        match self {
            Self::Spelling => SignatureScope::Phrase,
            Self::Grammar | Self::Mistranslation | Self::TooLoose | Self::Obscure => {
                SignatureScope::PhraseAndContext
            }
        }
    }

    #[must_use]
    pub fn acceptance_origin(self) -> AcceptanceOrigin {
        match self {
            Self::Spelling | Self::Grammar => AcceptanceOrigin::ArbitratedBeforeWrite,
            Self::Mistranslation | Self::TooLoose | Self::Obscure => AcceptanceOrigin::OtherTranslated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_round_trips_through_its_wire_string() {
        for kind in ProofreadKind::ALL {
            assert_eq!(ProofreadKind::from_wire(kind.as_str()), Some(kind));
            assert_eq!(serde_json::to_string(&kind).unwrap(), format!("\"{}\"", kind.as_str()));
        }
        assert_eq!(ProofreadKind::from_wire("style"), None);
        assert_eq!(ProofreadKind::from_wire("Spelling"), None);
    }

    #[test]
    fn each_kind_declares_its_signature_scope_and_acceptance_origin() {
        use AcceptanceOrigin::{ArbitratedBeforeWrite as Arb, OtherTranslated as Other};
        use SignatureScope::{Phrase, PhraseAndContext as Ctx};
        let table = [
            (ProofreadKind::Spelling, Phrase, Arb, false),
            (ProofreadKind::Grammar, Ctx, Arb, false),
            (ProofreadKind::Mistranslation, Ctx, Other, true),
            (ProofreadKind::TooLoose, Ctx, Other, true),
            (ProofreadKind::Obscure, Ctx, Other, true),
        ];
        for (kind, scope, acceptance, needs_source) in table {
            assert_eq!(kind.signature_scope(), scope, "{kind:?}");
            assert_eq!(kind.acceptance_origin(), acceptance, "{kind:?}");
            assert_eq!(kind.needs_source(), needs_source, "{kind:?}");
        }
    }
}
