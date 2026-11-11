//! Pure core of the grouped batch translation (AD-60): call planning, the block grammar the
//! prompt and the reader share, the incremental block reader, and the per-call reconciliation.
//! No store, no network, no clock.

use std::collections::BTreeSet;

pub const TRANSLATE_CALL_MAX_SEGMENTS: usize = 100;
pub const CONTEXT_GAP_MAX: usize = 3;
pub const CONTEXT_BEFORE: usize = 3;
pub const TM_PAIRS_PER_SEGMENT: usize = super::rag::TM_PAIR_CAP;

// aura-allow-text: prompt text sent to the model, not UI text
pub const SEGMENT_OPEN: &str = "<<<SEGMENT";
// aura-allow-text: prompt text sent to the model, not UI text
pub const SEGMENT_CLOSE: &str = "SEGMENT>>>";
// aura-allow-text: prompt text sent to the model, not UI text
pub const CONTEXT_OPEN: &str = "<<<CONTEXT";
// aura-allow-text: prompt text sent to the model, not UI text
pub const CONTEXT_CLOSE: &str = "CONTEXT>>>";
// aura-allow-text: prompt text sent to the model, not UI text
pub const TM_OPEN: &str = "<<<TM";
// aura-allow-text: prompt text sent to the model, not UI text
pub const TM_CLOSE: &str = "TM>>>";

// aura-allow-text: prompt text sent to the model, not UI text
pub const PROOFREAD_SOURCE_OPEN: &str = "<<<SOURCE";
// aura-allow-text: prompt text sent to the model, not UI text
pub const PROOFREAD_SOURCE_CLOSE: &str = "SOURCE>>>";
// aura-allow-text: prompt text sent to the model, not UI text
pub const PROOFREAD_TEXT_OPEN: &str = "<<<TEXT";
// aura-allow-text: prompt text sent to the model, not UI text
pub const PROOFREAD_TEXT_CLOSE: &str = "TEXT>>>";

// aura-allow-text: prompt text sent to the model, not UI text
pub const TM_BLOCK_LEAD: &str = "A TM block inside a SEGMENT lists close earlier translations as source → translation. They are reference only; never copy a TM block into the reply.";

// aura-allow-text: prompt text sent to the model, not UI text
const BLOCK_FORMAT_INSTRUCTION: &str = "Reply format: translate every SEGMENT block. For each one write a line <<<SEGMENT n (n is the number after SEGMENT in the input), then the translation of that segment only, then a line SEGMENT>>>. Keep the input order, one reply block per input block, never merge or skip a segment, and write nothing outside the blocks. CONTEXT blocks are already translated lines shown for context only: do not translate or copy them.";

const ZERO_WIDTH_SPACE: char = '\u{200B}';

/// The system message of a batch call: the resolved system prompt (Work, Global, default) with
/// the block format instruction after it, or the instruction alone when the prompt is empty.
pub fn system_with_block_format(resolved: Option<&str>) -> String {
    match resolved.filter(|s| !s.trim().is_empty()) {
        Some(system) => format!("{system}\n\n{BLOCK_FORMAT_INSTRUCTION}"),
        None => BLOCK_FORMAT_INSTRUCTION.to_owned(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Marker {
    SegmentOpen(u32),
    SegmentClose,
    ContextOpen,
    ContextClose,
    TmOpen,
    TmClose,
}

fn classify_line(line: &str) -> Option<Marker> {
    let t = line.trim();
    match t {
        SEGMENT_CLOSE => return Some(Marker::SegmentClose),
        CONTEXT_OPEN => return Some(Marker::ContextOpen),
        CONTEXT_CLOSE => return Some(Marker::ContextClose),
        TM_OPEN => return Some(Marker::TmOpen),
        TM_CLOSE => return Some(Marker::TmClose),
        _ => {}
    }
    let rest = t
        .strip_prefix(SEGMENT_OPEN)
        .or_else(|| t.strip_prefix("SEGMENT"))?;
    let digits = rest.strip_prefix(char::is_whitespace)?.trim();
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok().map(Marker::SegmentOpen)
}

/// `true` while `partial` could still grow into a marker line, so it must not be shown yet.
fn could_become_marker(partial: &str) -> bool {
    let t = partial.trim();
    if t.is_empty() {
        return true;
    }
    if [
        SEGMENT_OPEN,
        SEGMENT_CLOSE,
        CONTEXT_OPEN,
        CONTEXT_CLOSE,
        TM_OPEN,
        TM_CLOSE,
    ]
    .iter()
    .any(|m| m.starts_with(t))
    {
        return true;
    }
    let Some(rest) = t
        .strip_prefix(SEGMENT_OPEN)
        .or_else(|| t.strip_prefix("SEGMENT"))
    else {
        return false;
    };
    match rest.strip_prefix(char::is_whitespace) {
        Some(digits) => digits.trim().bytes().all(|b| b.is_ascii_digit()),
        None => false,
    }
}

/// Puts U+200B inside a line that is, whole, a marker; any other line is returned unchanged.
pub fn guard_marker_line(line: &str) -> String {
    if classify_line(line).is_none() {
        return line.to_owned();
    }
    split_after_first_char(line)
}

fn is_proofread_scan_marker(line: &str) -> bool {
    matches!(
        classify_line(line),
        Some(Marker::SegmentOpen(_) | Marker::SegmentClose)
    ) || [
        PROOFREAD_SOURCE_OPEN,
        PROOFREAD_SOURCE_CLOSE,
        PROOFREAD_TEXT_OPEN,
        PROOFREAD_TEXT_CLOSE,
    ]
    .contains(&line.trim())
}

/// [`guard_text`] for the user's proofread instruction: its marker set is the scan's SEGMENT,
/// SOURCE and TEXT markers, not the translation path's.
pub fn guard_proofread_instruction(text: &str) -> String {
    text.split('\n')
        .map(|line| {
            if is_proofread_scan_marker(line) {
                split_after_first_char(line)
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn split_after_first_char(line: &str) -> String {
    let Some((at, first)) = line.char_indices().find(|(_, c)| !c.is_whitespace()) else {
        return line.to_owned();
    };
    let split = at + first.len_utf8();
    format!("{}{ZERO_WIDTH_SPACE}{}", &line[..split], &line[split..])
}

/// Inverse of [`guard_marker_line`]: U+200B is dropped only from a line that is a marker once it
/// is gone; the line stays text. Every other U+200B is kept.
pub fn unguard_marker_line(line: &str) -> String {
    if !line.contains(ZERO_WIDTH_SPACE) {
        return line.to_owned();
    }
    let stripped: String = line.chars().filter(|&c| c != ZERO_WIDTH_SPACE).collect();
    if classify_line(&stripped).is_some() {
        stripped
    } else {
        line.to_owned()
    }
}

pub fn guard_text(text: &str) -> String {
    text.split('\n')
        .map(guard_marker_line)
        .collect::<Vec<_>>()
        .join("\n")
}

// ═════════════════════════════════════════════════════════════════════════════════
// Planning
// ═════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanSegment {
    pub id: i64,
    pub is_omitted: bool,
    /// `target_text` is non-empty after trimming.
    pub translated: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum PlanScope<'a> {
    /// Every untranslated, non-omitted segment, with already translated neighbours as context.
    Chapter,
    /// Exactly these ids (omitted ones excepted), translated or not.
    Selection(&'a BTreeSet<i64>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallEntry {
    Context(i64),
    Translate(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedCall {
    /// Counted from 1.
    pub call_index: u32,
    pub entries: Vec<CallEntry>,
}

impl PlannedCall {
    /// The segments this call translates, in block order (block `n` is element `n - 1`).
    pub fn segment_ids(&self) -> Vec<i64> {
        self.entries
            .iter()
            .filter_map(|e| match e {
                CallEntry::Translate(id) => Some(*id),
                CallEntry::Context(_) => None,
            })
            .collect()
    }
}

/// Plans every call of one operation from the chapter snapshot, in `ord` order. Omitted
/// segments are invisible: never sent and never breaking adjacency.
pub fn plan_calls(segments: &[PlanSegment], scope: PlanScope<'_>) -> Vec<PlannedCall> {
    let visible: Vec<&PlanSegment> = segments.iter().filter(|s| !s.is_omitted).collect();
    let calls = match scope {
        PlanScope::Selection(selected) => plan_selection(&visible, selected),
        PlanScope::Chapter => plan_chapter(&visible),
    };
    calls
        .into_iter()
        .map(|entries| with_leading_context(&visible, entries))
        .enumerate()
        .map(|(i, entries)| PlannedCall {
            call_index: u32::try_from(i + 1).unwrap_or(u32::MAX),
            entries,
        })
        .collect()
}

/// Replaces the leading context of a call with the up to `CONTEXT_BEFORE` visible segments
/// immediately before its first translated segment.
fn with_leading_context(visible: &[&PlanSegment], entries: Vec<CallEntry>) -> Vec<CallEntry> {
    let Some(first_at) = entries
        .iter()
        .position(|e| matches!(e, CallEntry::Translate(_)))
    else {
        return entries;
    };
    let CallEntry::Translate(first_id) = entries[first_at] else {
        return entries;
    };
    let Some(pos) = visible.iter().position(|s| s.id == first_id) else {
        return entries;
    };
    let from = pos.saturating_sub(CONTEXT_BEFORE);
    visible[from..pos]
        .iter()
        .map(|s| CallEntry::Context(s.id))
        .chain(entries.into_iter().skip(first_at))
        .collect()
}

fn plan_selection(visible: &[&PlanSegment], selected: &BTreeSet<i64>) -> Vec<Vec<CallEntry>> {
    let mut runs: Vec<Vec<i64>> = Vec::new();
    let mut in_run = false;
    for seg in visible {
        if selected.contains(&seg.id) {
            if !in_run {
                runs.push(Vec::new());
                in_run = true;
            }
            if let Some(run) = runs.last_mut() {
                run.push(seg.id);
            }
        } else {
            in_run = false;
        }
    }
    runs.iter()
        .flat_map(|run| run.chunks(TRANSLATE_CALL_MAX_SEGMENTS))
        .map(|chunk| chunk.iter().map(|&id| CallEntry::Translate(id)).collect())
        .collect()
}

fn plan_chapter(visible: &[&PlanSegment]) -> Vec<Vec<CallEntry>> {
    let mut calls: Vec<Vec<CallEntry>> = Vec::new();
    let mut current: Vec<CallEntry> = Vec::new();
    let mut current_need = 0usize;
    let mut gap: Vec<i64> = Vec::new();

    for seg in visible {
        if seg.translated {
            gap.push(seg.id);
            continue;
        }
        if calls.is_empty() && current_need == 0 {
            // first segment to translate in the chapter: nothing before it is context
        } else if gap.len() > CONTEXT_GAP_MAX {
            calls.push(std::mem::take(&mut current));
            current.extend(
                gap[gap.len() - CONTEXT_GAP_MAX..]
                    .iter()
                    .map(|&id| CallEntry::Context(id)),
            );
            current_need = 0;
        } else if current_need >= TRANSLATE_CALL_MAX_SEGMENTS {
            calls.push(std::mem::take(&mut current));
            current_need = 0;
        } else {
            current.extend(gap.iter().map(|&id| CallEntry::Context(id)));
        }
        gap.clear();
        current.push(CallEntry::Translate(seg.id));
        current_need += 1;
    }
    if current_need > 0 {
        calls.push(current);
    }
    calls
}

// ═════════════════════════════════════════════════════════════════════════════════
// Reading
// ═════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockEnd {
    Closed,
    /// The next open marker arrived before this block closed.
    Superseded,
    /// The stream ended while the block was open.
    StreamEnd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBlock {
    pub index: u32,
    pub text: String,
    pub end: BlockEnd,
}

/// Text of block `index` that is safe to show while the stream is still running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPiece {
    pub index: u32,
    pub text: String,
}

enum ReadState {
    Outside,
    Inside {
        index: u32,
        lines: Vec<String>,
        in_aside: bool,
        shown_any: bool,
    },
}

/// Reads the model reply incrementally. One machine serves both the shown text pieces and the
/// final blocks, so they cannot disagree.
pub struct BlockStream {
    line: String,
    emitted: usize,
    state: ReadState,
    blocks: Vec<ParsedBlock>,
    taken: usize,
}

impl Default for BlockStream {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockStream {
    pub fn new() -> Self {
        Self {
            line: String::new(),
            emitted: 0,
            state: ReadState::Outside,
            blocks: Vec::new(),
            taken: 0,
        }
    }

    /// Blocks that closed properly since the last call, in reading order. Does not alter what
    /// [`BlockStream::finish`] returns, so [`reconcile`] still judges the whole reply.
    pub fn take_closed(&mut self) -> Vec<ParsedBlock> {
        let fresh = self.blocks[self.taken..]
            .iter()
            .filter(|b| b.end == BlockEnd::Closed)
            .cloned()
            .collect();
        self.taken = self.blocks.len();
        fresh
    }

    pub fn push(&mut self, chunk: &str) -> Vec<TextPiece> {
        let mut out = Vec::new();
        let mut rest = chunk;
        loop {
            match rest.find('\n') {
                Some(i) => {
                    self.line.push_str(&rest[..i]);
                    self.complete_line(&mut out);
                    rest = &rest[i + 1..];
                }
                None => {
                    self.line.push_str(rest);
                    self.show_partial(&mut out);
                    break;
                }
            }
        }
        out
    }

    /// How many openings of block `index` have ended so far (closed or superseded).
    pub fn ended_openings(&self, index: u32) -> usize {
        self.blocks.iter().filter(|b| b.index == index).count()
    }

    pub fn finish(mut self) -> Vec<ParsedBlock> {
        if !self.line.is_empty() {
            let mut ignored = Vec::new();
            self.complete_line(&mut ignored);
        }
        if let ReadState::Inside { index, lines, .. } =
            std::mem::replace(&mut self.state, ReadState::Outside)
        {
            self.blocks.push(ParsedBlock {
                index,
                text: join_block_text(&lines),
                end: BlockEnd::StreamEnd,
            });
        }
        self.blocks
    }

    fn show_partial(&mut self, out: &mut Vec<TextPiece>) {
        let ReadState::Inside {
            index,
            in_aside,
            shown_any,
            ..
        } = &mut self.state
        else {
            return;
        };
        if *in_aside || could_become_marker(&self.line) || self.emitted >= self.line.len() {
            return;
        }
        let mut text = String::new();
        if *shown_any && self.emitted == 0 {
            text.push('\n');
        }
        text.push_str(&self.line[self.emitted..]);
        self.emitted = self.line.len();
        *shown_any = true;
        out.push(TextPiece {
            index: *index,
            text,
        });
    }

    fn complete_line(&mut self, out: &mut Vec<TextPiece>) {
        let line = std::mem::take(&mut self.line);
        let marker = classify_line(&line);
        let emitted = std::mem::take(&mut self.emitted);
        match (&mut self.state, marker) {
            (ReadState::Outside, Some(Marker::SegmentOpen(n))) => {
                self.state = ReadState::Inside {
                    index: n,
                    lines: Vec::new(),
                    in_aside: false,
                    shown_any: false,
                };
            }
            (ReadState::Outside, _) => {}
            (ReadState::Inside { index, lines, .. }, Some(Marker::SegmentClose)) => {
                self.blocks.push(ParsedBlock {
                    index: *index,
                    text: join_block_text(lines),
                    end: BlockEnd::Closed,
                });
                self.state = ReadState::Outside;
            }
            (ReadState::Inside { index, lines, .. }, Some(Marker::SegmentOpen(n))) => {
                self.blocks.push(ParsedBlock {
                    index: *index,
                    text: join_block_text(lines),
                    end: BlockEnd::Superseded,
                });
                self.state = ReadState::Inside {
                    index: n,
                    lines: Vec::new(),
                    in_aside: false,
                    shown_any: false,
                };
            }
            (ReadState::Inside { in_aside, .. }, Some(Marker::TmOpen | Marker::ContextOpen)) => {
                *in_aside = true;
            }
            (ReadState::Inside { in_aside, .. }, Some(Marker::TmClose | Marker::ContextClose)) => {
                *in_aside = false;
            }
            (
                ReadState::Inside {
                    index,
                    lines,
                    in_aside,
                    shown_any,
                },
                _,
            ) => {
                if *in_aside {
                    return;
                }
                let text = unguard_marker_line(&line);
                if emitted < line.len() {
                    let mut shown = String::new();
                    if *shown_any && emitted == 0 {
                        shown.push('\n');
                    }
                    shown.push_str(&line[emitted..]);
                    *shown_any = true;
                    out.push(TextPiece {
                        index: *index,
                        text: shown,
                    });
                }
                lines.push(text);
            }
        }
    }
}

fn join_block_text(lines: &[String]) -> String {
    lines.join("\n").trim().to_owned()
}

// ═════════════════════════════════════════════════════════════════════════════════
// Reconciliation
// ═════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscardReason {
    DuplicateIndex,
    Unclosed,
    Empty,
    Missing,
    NoFinish,
    Interrupted,
    Retired,
}

impl DiscardReason {
    pub fn as_str(self) -> &'static str {
        match self {
            DiscardReason::DuplicateIndex => "duplicate_index",
            DiscardReason::Unclosed => "unclosed",
            DiscardReason::Empty => "empty",
            DiscardReason::Missing => "missing",
            DiscardReason::NoFinish => "no_finish",
            DiscardReason::Interrupted => "interrupted",
            DiscardReason::Retired => "retired",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Received { text: String },
    Discarded { reason: DiscardReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamEnd<'a> {
    /// The provider ended the stream cleanly; `finish_reason` is whatever it sent last.
    Finished { finish_reason: Option<&'a str> },
    /// A provider error or a cancel cut the call.
    Interrupted,
}

/// One verdict per index `1..=k`, matched by index alone. A block is received when it is
/// closed, its index is in range and appears once, and its text is not empty.
pub fn reconcile(k: usize, blocks: &[ParsedBlock], end: &StreamEnd<'_>) -> Vec<Verdict> {
    let stopped = matches!(
        end,
        StreamEnd::Finished {
            finish_reason: Some("stop")
        }
    );
    let interrupted = matches!(end, StreamEnd::Interrupted);
    (1..=k)
        .map(|i| {
            let index = u32::try_from(i).unwrap_or(u32::MAX);
            let mut appearances = blocks.iter().filter(|b| b.index == index);
            let first = appearances.next();
            if appearances.next().is_some() {
                return Verdict::Discarded {
                    reason: DiscardReason::DuplicateIndex,
                };
            }
            let discarded = |reason| Verdict::Discarded { reason };
            match first {
                None if interrupted => discarded(DiscardReason::Interrupted),
                None if stopped => discarded(DiscardReason::Missing),
                None => discarded(DiscardReason::NoFinish),
                Some(block) => match block.end {
                    BlockEnd::Superseded => discarded(DiscardReason::Unclosed),
                    BlockEnd::StreamEnd if interrupted => discarded(DiscardReason::Interrupted),
                    BlockEnd::StreamEnd if stopped => discarded(DiscardReason::Unclosed),
                    BlockEnd::StreamEnd => discarded(DiscardReason::NoFinish),
                    BlockEnd::Closed if block.text.trim().is_empty() => {
                        discarded(DiscardReason::Empty)
                    }
                    BlockEnd::Closed => Verdict::Received {
                        text: block.text.clone(),
                    },
                },
            }
        })
        .collect()
}

/// A translation keeps a Han character the source also has (AD-44's definition of one).
pub fn has_han(source_text: &str, translated_text: &str) -> bool {
    source_text.chars().any(crate::core::dict::is_han)
        && translated_text.chars().any(crate::core::dict::is_han)
}
