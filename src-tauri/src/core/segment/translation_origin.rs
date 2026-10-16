//! The single arbitration of a segment's translation origin (AD-50 rule 4, AD-31 origin table).
//!
//! Compares text with the stored baseline, never a dirty flag. Confirm, merge and split all
//! read it; nothing else in the crate compares `target_text` with the baseline.

use unicode_normalization::UnicodeNormalization;

use crate::core::tm::PairOrigin;

/// The origin arbitration yields for one segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arbitrated {
    /// Empty text after `trim`: no translation, so no origin to declare (`''`).
    Unsigned,
    PairOrigin(PairOrigin),
}

impl Arbitrated {
    /// The stored spelling (`''` for [`Arbitrated::Unsigned`]).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unsigned => "",
            Self::PairOrigin(pair_origin) => pair_origin.as_str(),
        }
    }
}

/// `baseline_translation_origin` holds a value outside the closed set (AD-47 ⑥).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownBaselineOrigin(pub String);

/// Text differing from the baseline (`trim` + NFC), or an empty baseline origin, means the user
/// wrote it; otherwise the baseline origin carries over.
pub fn arbitrate(
    target_text: &str,
    baseline_target_text: &str,
    baseline_translation_origin: &str,
) -> Result<Arbitrated, UnknownBaselineOrigin> {
    let target: String = target_text.trim().nfc().collect();
    if target.is_empty() {
        return Ok(Arbitrated::Unsigned);
    }
    let baseline: String = baseline_target_text.trim().nfc().collect();
    if target != baseline || baseline_translation_origin.is_empty() {
        return Ok(Arbitrated::PairOrigin(PairOrigin::SelfTranslated));
    }
    PairOrigin::from_stored(baseline_translation_origin)
        .map(Arbitrated::PairOrigin)
        .ok_or_else(|| UnknownBaselineOrigin(baseline_translation_origin.to_owned()))
}

/// One stored version, as far as [`version_matching_text`] needs it.
#[derive(Debug, Clone, Copy)]
pub struct VersionText<'a> {
    pub id: i64,
    pub target_text: &'a str,
}

/// The version that stands for `text`: the largest `id` whose `target_text` equals `text` after
/// `trim` + NFC (the comparison `arbitrate` uses). `None` when no version matches. The "in use"
/// label and the origin a restore hands to the next confirm both come from this one function.
pub fn version_matching_text<'a>(
    versions: impl IntoIterator<Item = VersionText<'a>>,
    text: &str,
) -> Option<i64> {
    let wanted: String = text.trim().nfc().collect();
    versions
        .into_iter()
        .filter(|version| {
            let candidate: String = version.target_text.trim().nfc().collect();
            candidate == wanted
        })
        .map(|version| version.id)
        .max()
}
