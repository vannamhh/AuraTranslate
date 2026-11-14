//! Query-shape gate of the quick lookup's Vietnamese-to-Chinese group (AD-69): an English word whose
//! letters also occur inside a Vietnamese gloss must not open that group.

use crate::core::diacritics::fold_diacritics_case_preserving;

const ONSETS: &[&str] = &[
    "", "b", "c", "ch", "d", "g", "gh", "gi", "h", "k", "kh", "l", "m", "n", "ng", "ngh", "nh",
    "p", "ph", "q", "qu", "r", "s", "t", "th", "tr", "v", "x",
];

const VOWEL_CLUSTERS: &[&str] = &[
    "a", "e", "i", "o", "u", "y", "ai", "ao", "au", "ay", "eo", "eu", "ia", "ie", "iu", "oa", "oe",
    "oi", "oo", "ua", "ue", "ui", "uo", "uu", "uy", "ya", "ye", "ieu", "yeu", "oai", "oao", "oay",
    "oeo", "uay", "uoi", "uou", "uya", "uye", "uyu",
];

const CODAS: &[&str] = &["", "c", "ch", "m", "n", "ng", "nh", "p", "t"];

/// Every word is a Vietnamese syllable after folding; the last word may still be being typed.
pub fn could_be_vietnamese(query: &str) -> bool {
    let folded = fold_diacritics_case_preserving(query).to_lowercase();
    let words: Vec<&str> = folded.split_whitespace().collect();
    let Some((last, rest)) = words.split_last() else {
        return false;
    };
    rest.iter().all(|word| is_vietnamese_syllable(word))
        && (is_vietnamese_syllable(last) || starts_vietnamese_syllable(last))
}

fn is_vietnamese_syllable(word: &str) -> bool {
    ONSETS.iter().any(|onset| {
        word.strip_prefix(onset).is_some_and(|after_onset| {
            VOWEL_CLUSTERS.iter().any(|cluster| {
                after_onset
                    .strip_prefix(cluster)
                    .is_some_and(|coda| CODAS.contains(&coda))
            })
        })
    })
}

fn starts_vietnamese_syllable(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    ONSETS.iter().any(|onset| {
        onset.starts_with(word)
            || word.strip_prefix(onset).is_some_and(|after_onset| {
                !after_onset.is_empty()
                    && VOWEL_CLUSTERS
                        .iter()
                        .any(|cluster| cluster.starts_with(after_onset))
            })
    })
}
