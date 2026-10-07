//! TMX 1.4b render and parse for one TM tier (FR64, NFR9). No file I/O here (AD-15): bytes in,
//! text out; `tmx_io` touches the disk.

use std::collections::HashSet;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use super::{PairOrigin, TmPair};

/// Largest TMX file read; the caller's reader enforces it, the error lives here.
pub const MAX_TMX_BYTES: u64 = 256 * 1024 * 1024;

const ORIGIN_PROP: &str = "x-aura-origin";
const CREATED_AT_PROP: &str = "x-aura-created-at";
const TARGET_LANG: &str = "vi";
const ALL_LANGS: &str = "*all*";
const INLINE_ELEMENTS: [&str; 6] = ["bpt", "ept", "ph", "it", "ut", "sub"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TmxError {
    Malformed { line: usize, detail: String },
    NoBody,
    NotUtf8,
    TooLarge { limit: u64 },
    /// `source_lang` is the Work's language when the Work tier was asked for.
    NoUsablePair { source_lang: Option<String> },
    ReadFailed { detail: String },
    WriteFailed { detail: String },
}

impl std::fmt::Display for TmxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed { line, detail } => write!(f, "tmx[malformed] line {line}: {detail}"),
            Self::NoBody => write!(f, "tmx[no_body]"),
            Self::NotUtf8 => write!(f, "tmx[not_utf8]"),
            Self::TooLarge { limit } => write!(f, "tmx[too_large] limit {limit}"),
            Self::NoUsablePair { source_lang } => write!(f, "tmx[no_usable_pair] {source_lang:?}"),
            Self::ReadFailed { detail } => write!(f, "tmx[read_failed] {detail}"),
            Self::WriteFailed { detail } => write!(f, "tmx[write_failed] {detail}"),
        }
    }
}

impl std::error::Error for TmxError {}

/// Which side of the pair the source language comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TmxTier<'a> {
    Work { source_lang: &'a str },
    Global,
}

fn has_han(text: &str) -> bool {
    text.chars().any(crate::core::dict::is_han)
}

fn xml_representable(text: &str) -> bool {
    text.chars().all(|c| matches!(c, '\t' | '\n' | '\r') || ((c as u32) >= 0x20 && !matches!(c as u32, 0xFFFE | 0xFFFF)))
}

fn escape_into(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\r' => out.push_str("&#13;"),
            '\t' | '\n' => out.push(c),
            c if (c as u32) < 0x20 || matches!(c as u32, 0xFFFE | 0xFFFF) => {}
            c => out.push(c),
        }
    }
}

/// `2026-10-04T12:34:56.789Z` as `20261004T123456Z`; `None` when the value is not that shape.
fn tmx_date_from_iso(created_at: &str) -> Option<String> {
    let iso = parse_iso_millis(created_at)?;
    Some(format!(
        "{}{}{}T{}{}{}Z",
        &iso[0..4],
        &iso[5..7],
        &iso[8..10],
        &iso[11..13],
        &iso[14..16],
        &iso[17..19]
    ))
}

fn in_range(s: &str, lo: u32, hi: u32) -> bool {
    s.parse::<u32>().is_ok_and(|v| (lo..=hi).contains(&v))
}

fn date_parts_valid(year: &str, month: &str, day: &str, hour: &str, minute: &str, second: &str) -> bool {
    year.bytes().all(|b| b.is_ascii_digit())
        && in_range(month, 1, 12)
        && in_range(day, 1, 31)
        && in_range(hour, 0, 23)
        && in_range(minute, 0, 59)
        && in_range(second, 0, 60)
}

/// The value when it is `YYYY-MM-DDTHH:MM:SS.mmmZ` with every field in range.
pub fn parse_iso_millis(value: &str) -> Option<&str> {
    let b = value.as_bytes();
    if b.len() != 24 || !value.is_ascii() {
        return None;
    }
    let shape = b[4] == b'-' && b[7] == b'-' && b[10] == b'T' && b[13] == b':' && b[16] == b':' && b[19] == b'.' && b[23] == b'Z';
    let millis = b[20..23].iter().all(u8::is_ascii_digit);
    (shape && millis && date_parts_valid(&value[0..4], &value[5..7], &value[8..10], &value[11..13], &value[14..16], &value[17..19]))
        .then_some(value)
}

/// `20261004T123456Z` as `2026-10-04T12:34:56.000Z`.
fn iso_from_tmx_date(value: &str) -> Option<String> {
    let b = value.as_bytes();
    if b.len() != 16 || !value.is_ascii() || b[8] != b'T' || b[15] != b'Z' {
        return None;
    }
    let (y, mo, d, h, mi, s) = (&value[0..4], &value[4..6], &value[6..8], &value[9..11], &value[11..13], &value[13..15]);
    date_parts_valid(y, mo, d, h, mi, s).then(|| format!("{y}-{mo}-{d}T{h}:{mi}:{s}.000Z"))
}

/// A TMX document and how many pairs it leaves out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedTmx {
    pub text: String,
    /// Pairs holding a character XML 1.0 cannot represent; they are not in `text`.
    pub left_out: usize,
}

/// Renders distinct pairs as a TMX 1.4b document. A pair whose source, target or date holds a
/// character XML 1.0 cannot represent is left out and counted, never written altered.
pub fn render_tmx(pairs: &[TmPair], tier: TmxTier<'_>) -> RenderedTmx {
    let srclang = match tier {
        TmxTier::Work { source_lang } => source_lang,
        TmxTier::Global => ALL_LANGS,
    };
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<tmx version=\"1.4\">\n  <header creationtool=\"AuraTranslate\" creationtoolversion=\"");
    escape_into(&mut out, env!("CARGO_PKG_VERSION"));
    out.push_str("\" segtype=\"sentence\" o-tmf=\"AuraTranslate\" adminlang=\"vi\" srclang=\"");
    escape_into(&mut out, srclang);
    out.push_str("\" datatype=\"plaintext\"/>\n  <body>\n");
    let mut left_out = 0;
    for pair in pairs {
        if ![&pair.source_text, &pair.target_text, &pair.created_at].into_iter().all(|t| xml_representable(t)) {
            left_out += 1;
            continue;
        }
        let source_lang = match tier {
            TmxTier::Work { source_lang } => source_lang,
            TmxTier::Global if has_han(&pair.source_text) => "zh",
            TmxTier::Global => "en",
        };
        out.push_str("    <tu");
        if let Some(date) = tmx_date_from_iso(&pair.created_at) {
            out.push_str(" creationdate=\"");
            out.push_str(&date);
            out.push('"');
        }
        out.push_str(">\n      <prop type=\"");
        out.push_str(ORIGIN_PROP);
        out.push_str("\">");
        out.push_str(pair.translation_origin.as_str());
        out.push_str("</prop>\n      <prop type=\"");
        out.push_str(CREATED_AT_PROP);
        out.push_str("\">");
        escape_into(&mut out, &pair.created_at);
        out.push_str("</prop>\n      <tuv xml:lang=\"");
        escape_into(&mut out, source_lang);
        out.push_str("\"><seg>");
        escape_into(&mut out, &pair.source_text);
        out.push_str("</seg></tuv>\n      <tuv xml:lang=\"");
        out.push_str(TARGET_LANG);
        out.push_str("\"><seg>");
        escape_into(&mut out, &pair.target_text);
        out.push_str("</seg></tuv>\n    </tu>\n");
    }
    out.push_str("  </body>\n</tmx>\n");
    RenderedTmx { text: out, left_out }
}

/// UTF-8 (BOM allowed) or UTF-16 with BOM to text; anything else is `NotUtf8`.
pub fn decode_tmx_bytes(bytes: &[u8]) -> Result<String, TmxError> {
    if bytes.len() as u64 > MAX_TMX_BYTES {
        return Err(TmxError::TooLarge { limit: MAX_TMX_BYTES });
    }
    let utf16 = |rest: &[u8], le: bool| -> Result<String, TmxError> {
        if rest.len() % 2 != 0 {
            return Err(TmxError::NotUtf8);
        }
        let units: Vec<u16> = rest
            .chunks_exact(2)
            .map(|p| if le { u16::from_le_bytes([p[0], p[1]]) } else { u16::from_be_bytes([p[0], p[1]]) })
            .collect();
        String::from_utf16(&units).map_err(|_| TmxError::NotUtf8)
    };
    match bytes {
        [0xFF, 0xFE, rest @ ..] => utf16(rest, true),
        [0xFE, 0xFF, rest @ ..] => utf16(rest, false),
        [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8(rest.to_vec()).map_err(|_| TmxError::NotUtf8),
        _ => String::from_utf8(bytes.to_vec()).map_err(|_| TmxError::NotUtf8),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TmxUnit {
    pub creationdate: Option<String>,
    pub pair_origin: Option<String>,
    pub created_at: Option<String>,
    /// `(xml:lang as read, text as read)` in file order.
    pub variants: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedTmx {
    pub srclang: Option<String>,
    pub units: Vec<TmxUnit>,
}

fn local_name(start: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(start.local_name().as_ref()).into_owned()
}

fn attribute(start: &BytesStart<'_>, local: &str) -> Option<String> {
    start
        .attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == local.as_bytes())
        .and_then(|a| a.normalized_value(quick_xml::XmlVersion::Implicit1_0).ok().map(|v| v.into_owned()))
}

fn line_at(text: &str, position: u64) -> usize {
    let end = usize::try_from(position).unwrap_or(usize::MAX).min(text.len());
    1 + text.as_bytes()[..end].iter().filter(|&&b| b == b'\n').count()
}

fn predefined_entity(name: &str) -> Option<char> {
    match name {
        "lt" => Some('<'),
        "gt" => Some('>'),
        "amp" => Some('&'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => None,
    }
}

#[derive(Default)]
struct Walk {
    stack: Vec<String>,
    inline_depth: usize,
    seen_tmx: bool,
    seen_body: bool,
    unit: Option<TmxUnit>,
    lang: Option<String>,
    seg: Option<String>,
    prop_type: Option<String>,
    prop: String,
    parsed: ParsedTmx,
}

impl Walk {
    fn start(&mut self, e: &BytesStart<'_>) {
        let name = local_name(e);
        match name.as_str() {
            "tmx" if self.stack.is_empty() => self.seen_tmx = true,
            "body" if self.stack.len() == 1 => self.seen_body = true,
            "header" if self.stack.len() == 1 => self.parsed.srclang = attribute(e, "srclang"),
            "tu" if self.in_body() => {
                self.unit = Some(TmxUnit { creationdate: attribute(e, "creationdate"), ..TmxUnit::default() });
            }
            "tuv" if self.unit.is_some() => self.lang = Some(attribute(e, "lang").unwrap_or_default()),
            "seg" if self.lang.is_some() => self.seg = Some(String::new()),
            "prop" if self.unit.is_some() && self.lang.is_none() => {
                self.prop_type = attribute(e, "type");
                self.prop.clear();
            }
            n if self.seg.is_some() && INLINE_ELEMENTS.contains(&n) => self.inline_depth += 1,
            _ => {}
        }
        self.stack.push(name);
    }

    fn in_body(&self) -> bool {
        self.seen_body && self.stack.len() == 2 && self.stack[1] == "body"
    }

    fn end(&mut self) {
        let Some(name) = self.stack.pop() else { return };
        match name.as_str() {
            "tu" if self.stack.len() == 2 => {
                if let Some(unit) = self.unit.take() {
                    self.parsed.units.push(unit);
                }
            }
            "tuv" if self.lang.is_some() && self.seg.is_none() => self.lang = None,
            "seg" if self.seg.is_some() && self.inline_depth == 0 => {
                if let (Some(unit), Some(lang), Some(seg)) = (self.unit.as_mut(), self.lang.take(), self.seg.take()) {
                    unit.variants.push((lang, seg));
                }
            }
            "prop" if self.prop_type.is_some() => {
                if let (Some(unit), Some(kind)) = (self.unit.as_mut(), self.prop_type.take()) {
                    let value = self.prop.trim().to_owned();
                    match kind.as_str() {
                        ORIGIN_PROP => unit.pair_origin = Some(value),
                        CREATED_AT_PROP => unit.created_at = Some(value),
                        _ => {}
                    }
                }
            }
            n if self.seg.is_some() && INLINE_ELEMENTS.contains(&n) => {
                self.inline_depth = self.inline_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn text(&mut self, text: &str) {
        if let Some(seg) = self.seg.as_mut() {
            if self.inline_depth == 0 {
                seg.push_str(text);
            }
        } else if self.prop_type.is_some() {
            self.prop.push_str(text);
        }
    }
}

/// Reads a TMX document into units; nothing is interpreted beyond structure.
pub fn parse_tmx(text: &str) -> Result<ParsedTmx, TmxError> {
    let mut reader = Reader::from_str(text);
    let mut walk = Walk::default();
    let malformed = |reader: &Reader<&[u8]>, detail: String| TmxError::Malformed {
        line: line_at(text, reader.error_position().max(reader.buffer_position())),
        detail,
    };
    loop {
        let event = match reader.read_event() {
            Ok(event) => event,
            Err(e) => return Err(malformed(&reader, e.to_string())),
        };
        match event {
            Event::Eof => break,
            Event::Start(e) => walk.start(&e),
            Event::Empty(e) => {
                walk.start(&e);
                walk.end();
            }
            Event::End(_) => walk.end(),
            Event::Text(t) => match t.xml_content(quick_xml::XmlVersion::Implicit1_0) {
                Ok(s) => walk.text(&s),
                Err(e) => return Err(malformed(&reader, e.to_string())),
            },
            Event::CData(c) => match c.xml_content(quick_xml::XmlVersion::Implicit1_0) {
                Ok(s) => walk.text(&s),
                Err(e) => return Err(malformed(&reader, e.to_string())),
            },
            Event::GeneralRef(r) => {
                let resolved = match r.resolve_char_ref() {
                    Ok(Some(c)) => Some(c),
                    Ok(None) => r.decode().ok().and_then(|n| predefined_entity(&n)),
                    Err(e) => return Err(malformed(&reader, e.to_string())),
                };
                match resolved {
                    Some(c) => walk.text(c.encode_utf8(&mut [0u8; 4])),
                    None => return Err(malformed(&reader, "unknown entity".to_owned())),
                }
            }
            _ => {}
        }
    }
    if !walk.stack.is_empty() {
        return Err(malformed(&reader, format!("unclosed element {}", walk.stack.join("/"))));
    }
    if !walk.seen_tmx || !walk.seen_body {
        return Err(TmxError::NoBody);
    }
    Ok(walk.parsed)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedPair {
    pub source_text: String,
    pub target_text: String,
    /// The label read from the file; `None` when missing or unknown.
    pub translation_origin: Option<PairOrigin>,
    /// `YYYY-MM-DDTHH:MM:SS.mmmZ`; `None` means the writer stamps now.
    pub created_at: Option<String>,
}

/// What the preview counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPlan {
    /// `<tu>` elements read.
    pub unit_count: usize,
    /// `<tu>` missing a side or holding a blank one.
    pub skipped_count: usize,
    /// `<tu>` repeating an earlier (source, target) of the same file.
    pub duplicate_in_file_count: usize,
    /// Distinct pairs of the file, file order.
    pub pairs: Vec<PlannedPair>,
}

fn primary_subtag(lang: &str) -> String {
    lang.split(['-', '_']).next().unwrap_or("").to_ascii_lowercase()
}

fn created_at_for(unit: &TmxUnit) -> Option<String> {
    unit.created_at
        .as_deref()
        .and_then(parse_iso_millis)
        .map(str::to_owned)
        .or_else(|| unit.creationdate.as_deref().and_then(iso_from_tmx_date))
}

fn source_variant<'a>(unit: &'a TmxUnit, wanted: Option<&str>) -> Option<&'a str> {
    unit.variants
        .iter()
        .find(|(lang, _)| {
            let primary = primary_subtag(lang);
            primary != TARGET_LANG && wanted.is_none_or(|w| primary == w)
        })
        .map(|(_, text)| text.as_str())
}

/// Picks each unit's (source, target), counts what is skipped or repeated, and fails with
/// `NoUsablePair` when nothing is left.
pub fn plan_import(parsed: &ParsedTmx, tier: TmxTier<'_>) -> Result<ImportPlan, TmxError> {
    let wanted: Option<String> = match tier {
        TmxTier::Work { source_lang } => Some(primary_subtag(source_lang)),
        TmxTier::Global => parsed
            .srclang
            .as_deref()
            .map(primary_subtag)
            .filter(|s| !s.is_empty() && s != ALL_LANGS),
    };
    let mut plan = ImportPlan { unit_count: parsed.units.len(), skipped_count: 0, duplicate_in_file_count: 0, pairs: Vec::new() };
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for unit in &parsed.units {
        let source = source_variant(unit, wanted.as_deref());
        let target = unit.variants.iter().find(|(lang, _)| primary_subtag(lang) == TARGET_LANG).map(|(_, t)| t.as_str());
        let (Some(source), Some(target)) = (source, target) else {
            plan.skipped_count += 1;
            continue;
        };
        if source.trim().is_empty() || target.trim().is_empty() {
            plan.skipped_count += 1;
            continue;
        }
        if !seen.insert((source.to_owned(), target.to_owned())) {
            plan.duplicate_in_file_count += 1;
            continue;
        }
        plan.pairs.push(PlannedPair {
            source_text: source.to_owned(),
            target_text: target.to_owned(),
            translation_origin: unit.pair_origin.as_deref().and_then(PairOrigin::from_stored),
            created_at: created_at_for(unit),
        });
    }
    if plan.pairs.is_empty() {
        let source_lang = match tier {
            TmxTier::Work { source_lang } => Some(source_lang.to_owned()),
            TmxTier::Global => None,
        };
        return Err(TmxError::NoUsablePair { source_lang });
    }
    Ok(plan)
}

/// Pairs of a tier as stored, for the "already there" check.
pub fn existing_pair_keys(store: &crate::core::store::Store) -> Result<HashSet<(String, String)>, super::TmStoreError> {
    let keys = store.read(|conn| {
        let mut stmt = conn.prepare("SELECT source_text, target_text FROM tm_unit")?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
            .collect::<crate::core::store::SqlResult<HashSet<_>>>()?;
        Ok(rows)
    })?;
    Ok(keys)
}

/// Pairs of the plan the tier does not hold yet.
pub fn pairs_not_in(plan: &ImportPlan, existing: &HashSet<(String, String)>) -> Vec<PlannedPair> {
    plan.pairs
        .iter()
        .filter(|p| !existing.contains(&(p.source_text.clone(), p.target_text.clone())))
        .cloned()
        .collect()
}

/// Only a file the user declares as their own translation may land pairs as `self`; a
/// `bilingual_import` label stays, everything else is `other`.
pub fn pair_origin_for(label: Option<PairOrigin>, file_is_mine: bool) -> PairOrigin {
    match label {
        Some(PairOrigin::BilingualImport) => PairOrigin::BilingualImport,
        Some(PairOrigin::SelfTranslated) | None if file_is_mine => PairOrigin::SelfTranslated,
        _ => PairOrigin::Other,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriteOutcome {
    pub inserted: usize,
    pub already_there: usize,
    /// Inserted pairs whose file date lay after the import moment and was replaced by it.
    pub future_dated: usize,
}

/// Inserts every pair the tier lacks in ONE transaction, re-checking identity inside it; any
/// failing row rolls the whole batch back.
pub fn write_planned_pairs(
    store: &crate::core::store::Store,
    pairs: Vec<PlannedPair>,
    file_is_mine: bool,
) -> Result<WriteOutcome, super::TmStoreError> {
    let outcome = store.write(move |tx| {
        let now: String = tx.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')", [], |r| r.get(0))?;
        let mut inserted = 0;
        let mut already_there = 0;
        let mut future_dated = 0;
        for pair in &pairs {
            let exists: i64 = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM tm_unit WHERE source_text = ?1 AND target_text = ?2)",
                (&pair.source_text, &pair.target_text),
                |r| r.get(0),
            )?;
            if exists != 0 {
                already_there += 1;
                continue;
            }
            let created_at = match &pair.created_at {
                Some(at) if *at > now => {
                    future_dated += 1;
                    &now
                }
                Some(at) => at,
                None => &now,
            };
            let pair_origin = pair_origin_for(pair.translation_origin, file_is_mine);
            tx.execute(
                "INSERT INTO tm_unit (source_text, target_text, translation_origin, created_at) \
                 VALUES (?1, ?2, ?3, ?4)",
                (&pair.source_text, &pair.target_text, pair_origin.as_str(), created_at),
            )?;
            inserted += 1;
        }
        Ok(WriteOutcome { inserted, already_there, future_dated })
    })?;
    Ok(outcome)
}

/// One `<tu>` per distinct (source, target) of a tier, carrying its first copy in AD-18 order
/// (mine first, newest, highest id); output in the id order of those copies.
pub fn distinct_tier_pairs(store: &crate::core::store::Store, tier: super::TmTier) -> Result<Vec<TmPair>, super::TmStoreError> {
    let rows = super::load_all_pair_rows(store)?;
    let resolver = crate::core::scope::ScopeResolver::global_only();
    let ordered = match tier {
        super::TmTier::Global => super::merge_tiers(&resolver, rows, None)?,
        super::TmTier::Work => super::merge_tiers(&resolver, Vec::new(), Some(rows))?,
    };
    let mut pairs = super::first_per_text_pair(ordered, |p| p);
    pairs.sort_by_key(|p| p.id);
    Ok(pairs)
}
