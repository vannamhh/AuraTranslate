---
ticket: 6
title: 'Story 7.6 — Glossary/TM variant parity on the one shared Matcher'
type: 'feature'
created: '2026-10-03'
status: done
route: 'dispatch'
baseline_revision: 'e45fe9e957164a4fed93b8cf50034f9b426b7ac6'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-7-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR61/AD-17 promise that Glossary and TM catch the same variants through one Matcher, but nothing proves it. The 7.5 scorer rebuilds its own 1-/2-gram windowing instead of calling `ngrams`. `glossary/scan.rs` keeps a second copy of the sentence-boundary rule from `find_terms`. No test enforces that `core/matching` does no I/O (AD-15). The module's doc comment still holds stale measurements and says "glossary and tm do not exist yet". Four ledger items name Story 7.6 as owner.

**Approach:** Prove parity through the real consumers: `glossary::store::marks_for_source_text` and `tm::rank_fuzzy_candidates`, both given `match_lang_for_source_lang`. Add a black-box test that keeps the scorer equal to Dice over `ngrams`. Move the sentence-boundary rule into `core/matching` as the one copy. Add the AD-15 guard. Clean the doc comment. Close or reassign each ledger item.

## Boundaries & Constraints

**Decisions (Ice, 2026-10-03):**
- NFC (ledger L588) = B: no normalisation in this story; the parity table carries an NFD-vs-NFC `café` row that neither consumer catches, and L588 is reassigned to Winston beside L455/L7915 so both consumers flip together when that AD lands.
- Spec kept whole above 1600 tokens.

**Always:**
- "Variant" means two surface forms that the Matcher maps to one unit: English case and Porter2 stem, and Chinese identity. Location rules are not variants. These are Chinese jieba-boundary acceptance in `find_terms` and the sentence-boundary rejection.
- Glossary catches a variant when `marks_for_source_text` marks the term written in the other form. TM catches it when two sentences that differ only by that form score the top value from `rank_fuzzy_candidates` (99 after the cap).
- The parity table is an iff check. Each row must hold in both directions, and the table has both kinds of rows:
  - Rows both catch: inflection, case, inflection plus case.
  - Rows neither catches: lemma pairs such as `went`/`go`, the non-ASCII `café`/`cafe`, traditional/simplified `翻译`/`翻譯`, full-/half-width.
- The scorer keeps its interned fast path (Ice accepted about 0.7 s at 100,000 pairs per tier in 7.5). Parity with `ngrams` is guarded by a test, not by routing the scorer through `ngrams`.
- Every guard has a real counter-check (AGENTS.md).

**Never:**
- No change to the score formula, threshold, cap, exact-exclusion, diff, wire shapes or webview.
- No new crate.
- No lemmatization, width folding or traditional/simplified folding.
- No touch to `core/dict` (AD-17 ⚠️: the dict path is not a Matcher consumer, AD-44 ③).
- No Concordance (7.7).
- Do not loosen any `matching_boundary.rs` list or lower a floor.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Inflection | term `translation`, text `translations` | Glossary marks; TM top score | N/A |
| Case | `TRANSLATION` vs `translation` | both catch | N/A |
| Lemma | `went` vs `go` | neither catches | N/A |
| Non-ASCII | `café` vs `cafe` | neither catches | N/A |
| Zh script | `翻译` vs `翻譯` | neither catches | N/A |
| NFD vs NFC | `café` NFD vs NFC | neither catches | N/A |
| Scorer vs ngrams | any Zh/En sentence pair | `percent` = floor(Dice over `ngrams` n=1 ∪ n=2) | N/A |
| Empty side | `""` vs text | same value from both computations | N/A |
| Sentence boundary | multi-word term split by `.` | `find_terms` and `scan.rs` both reject, via one fn | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/matching/mod.rs`:
  - `tokenize` :252, `normalize` :333, `ngrams` :374.
  - `SimilarityScorer` :409. Private `units` :433 and `profile` :450 do their own windowing; `percent` :461.
  - `find_terms` :633. Its inline sentence-boundary rule is at :700-705; extract it to one `pub fn` taking the text and two `MatchToken`s.
  - Doc lines to delete or rewrite: :20, :23, :27-28, :59-69, :103, :136, :156, :314, :405-408 (keep the claim, now guarded), :628.
- `src-tauri/src/core/glossary/scan.rs:390-405`: `gap_crosses_sentence_boundary` is the copy. Replace it with the matching fn. Also fix the comments at :321 and :395.
- `src-tauri/src/core/glossary/store.rs`:
  - `match_lang_for_source_lang` :1265.
  - `marks_for_source_text` :1440 is the Glossary consumer.
  - :1278 repeats the "179–329 ms" figure; delete it.
- `src-tauri/src/core/tm/mod.rs:272`: `rank_fuzzy_candidates(resolver, candidates, source_text, lang)` is the TM consumer, with cap `.min(99)` at :286.
- `src-tauri/tests/matching_boundary.rs`:
  - The failure message at :31-32 omits `SimilarityScorer`/`diff_spans`.
  - `MATCHING_FORBIDDEN_USES` is at :74. Add a separate I/O-token list (`std::fs`, `std::io`, `std::net`, `std::process`, `rusqlite`, `reqwest`, `tauri`) scanned on code lines only, same scanner style as :279.
- `src-tauri/tests/matching_contract.rs`: imports `auratranslate_lib::core::matching` at :17; the score tests are at :558-642. Add the scorer-vs-`ngrams` Dice test here.
- The parity test needs both stores. Seed them the way `tests/tm_contract.rs` (fuzzy cases) and the Glossary marks tests do. Put it in `tm_contract.rs` or a new file. A new file must raise any test-file floor to the live count.
- `_bmad-output/implementation-artifacts/deferred-work.md`, items for Story 7.6 (append-only close lines, never delete):
  - L575 (AD-15 guard) ⇒ ✅.
  - L578 (measurements in doc) ⇒ ✅.
  - L579 (re-tokenize) ⇒ KHÔNG LÀM: no path calls `find_terms` and `ngrams`/scorer on the same text. The scorer is used only at `tm/mod.rs:279`, `ngrams` only at `scan.rs:231`.
  - L588 (NFC) ⇒ reassign to Winston (`→ … Chủ: Winston`), pointing at the parity row.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/matching/mod.rs` -- extract the sentence-boundary fn; doc clean-up -- one copy of the rule; no stale numbers.
- [x] `src-tauri/src/core/glossary/scan.rs`, `store.rs` -- call the matching fn; drop the duplicate figure.
- [x] `src-tauri/tests/matching_boundary.rs` -- AD-15 I/O guard; message names every public item.
- [x] `src-tauri/tests/matching_contract.rs` -- scorer equals Dice over `ngrams` on a Zh+En corpus that includes an empty side.
- [x] Parity test (see Code Map) -- the matrix rows through `marks_for_source_text` and `rank_fuzzy_candidates` with `match_lang_for_source_lang`.
- [x] `deferred-work.md` -- the four close/reassign lines.

**Acceptance Criteria:**
- Given the parity test, when the scorer's `units` stops calling `normalize` (raw lowercase only), then the Inflection row goes red and the Lemma row stays green.
- Given the parity test, when `rank_fuzzy_candidates` is given a lang other than its caller's (a real code change), then a row goes red.
- Given the scorer-vs-`ngrams` test, when `profile` drops its 2-grams, then it goes red.
- Given the AD-15 guard, when a `std::fs` call is really added to `core/matching`, then it goes red, and a comment that mentions `std::fs` stays green.
- Given `scan.rs`, when its copy is gone, then a grep for `contains(['.', '!', '?', '\n'])` under `src-tauri/src` finds exactly one hit, in `core/matching`.

## Implementation Notes

- The sentence-boundary rule is `core::matching::gap_crosses_sentence_boundary(text, a, b)`, called by `find_terms` and `glossary/scan.rs`; a new scan case (`a_capitalized_run_split_by_a_full_stop_never_joins_into_one_candidate`) covers the scan side, which had no cross-sentence case before.
- The parity test (`tm_contract.rs::glossary_and_tm_catch_exactly_the_same_variants_in_both_directions`) puts each form in one sentence frame per language, runs both directions, and collects every mismatched row before failing, so a counter-check shows which rows stay green.
- TM "catches" means `rank_fuzzy_candidates` at threshold 99 keeps the pair. Full-width Latin under `En` yields no token at all, so in a single-word test it scores 0 on both sides; the sentence frame keeps that row meaningful.
- The scorer keeps its own windowing; `matching_contract.rs::the_scorer_equals_dice_over_ngrams_one_and_two_on_a_chinese_and_english_corpus` recomputes Dice from `ngrams` n=1 and n=2 and is the only case that fails when `profile` drops its 2-grams.
- Counter-checks (code really changed, then restored byte for byte):
  - Scorer `units` lowercases without stemming ⇒ only the four inflection rows go red; the case, lemma, non-ASCII, NFD, width and Zh rows stay green.
  - `rank_fuzzy_candidates` scores with the other language ⇒ the six En "both catch" rows go red.
  - The boundary fn returns `false` ⇒ the new scan case and `english_multi_word_terms_never_join_across_a_sentence_boundary` go red.
  - `profile` drops 2-grams ⇒ only the scorer-vs-`ngrams` case goes red.
  - A real `std::fs::metadata` call in `percent` ⇒ `the_matching_module_performs_no_io` goes red, naming that line.
- An NFD `café` tokenizes to ASCII `cafe`, so both consumers catch NFD vs `cafe`. The Glossary mark stops before U+0301, which is why the parity helper checks "mark starts at the form and ends inside it" rather than an exact substring. The grapheme cut is deferred to Winston.

## Spec Change Log

## Review Triage Log

- Blind+Edge+Verif: AD-15 guard misses grouped `use std::{fs}` and `tauri_plugin_*`/`keyring`/`libsqlite3_sys`/`std::env` — medium, patch: substring scan has no `std::fs` text in a grouped import, and `_` blocks the `tauri` word match.
- Blind+Edge: parity table has no Chinese row both catch — low, patch: a Zh lang swap left every Zh row green; identity rows added, TM side via the exact read.
- Blind+Verif: NFD `café` vs ASCII `cafe` is caught by both (ASCII-only tokenizer) but had no row — low, patch: row added so the NFC AD flips it visibly.
- Blind: Glossary half counts any mark — low, patch: assert the marked substring equals the variant.
- Blind+Edge: parity helpers skip `DirGuard`; `Box::leak` per call — low, patch.
- Blind+Edge: stale find_terms "out of scope: ranking, threshold — 7.5/7.6" — low, patch (listed in Code Map, missed).
- Blind+Edge: "179–329 ms"/"16/16"/"0,052–0,961 ms"/"jieba-rs 0.10.3" still in glossary/mod.rs, commands/chapter.rs, matching_boundary.rs header, mod.rs — low, patch: same figure ledger L578 closes.
- Blind+Edge: failure message omits `warm` — low, patch; types are not call paths, rejected.
- Blind: scorer-vs-ngrams corpus lacks mixed script, whitespace/punctuation-only, ASCII+non-ASCII, newline — low, patch.
- Blind: L575 close line cites the self-test fixture, not the real counter-check — low, patch.
- Blind: ledger cites siblings by line number in an append-only 1 MB file — low, patch.
- Blind: new lines exceed 100 columns; double blank line — low, patch (repo is not rustfmt-clean and no gate runs fmt; only lines this diff wrote).
- Blind+Edge: `gap_crosses_sentence_boundary` panics on reversed tokens, ignores `。！？` — doc precondition patched; panic is false as a defect: both callers pass adjacent tokens of `tokenize(text, En)`, and loud failure on unshown misuse is correct; full-width stops are pre-existing En-branch semantics.
- Blind: no direct table test for `!`/`?`/`\n` — low, rejected: rule moved unchanged; fix adds tests beyond a direct correction.
- Blind: no gate stops a second copy of the boundary rule — low, rejected: a text-scan gate is easily evaded by an equivalent form; adds a gate.
- Blind: `Jieba::cut`/Porter2 tables unasserted — low, rejected: behaviour examples, not measurements.
- Blind: story ids/dates elsewhere in mod.rs — rejected: pre-existing lines outside the diff (AGENTS.md: no mass cleanup).
- Edge: `"tauri"` inside a string literal would false-alarm — low, rejected: no such literal; unlikely.
- Edge: parity test calls consumers directly, production lang wiring unguarded — false: flipping lang at `segment.rs:2430` turned 3 tm_contract wire cases red; flipping it at `commands/glossary.rs:300` turned `glossary_marks_contract::the_work_tier_wins_over_global_through_the_real_glossary_marks_for_chapter_surface` red.
- Verif other: Glossary term `café` also matches bare `caf`/`cafè` (ASCII-only tokenizer) — low, pre-existing, defer to Winston with the NFC family.

## Verification

**Commands:**
- `cargo test --test matching_contract --test matching_boundary` -- green, including the new cases.
- `cargo test --test <parity target>` and the Glossary scan tests that cover `scan.rs` -- green.
- `npm run build` must run first if `dist/` is missing.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 7.6 (v6, nay ở archive-v6).

**Covers:** FR61

As a người dựng,
I want TM khớp bằng đúng cơ chế mà từ điển và Glossary đang dùng,
So that ba nơi không bao giờ bắt được những biến thể khác nhau.

**Acceptance Criteria:**

**Given** khớp TM
**When** cài đặt
**Then** dùng **đúng component `Matcher` dùng chung** của Epic 1

**Given** văn bản tiếng Trung
**When** khớp mờ
**Then** dùng **n-gram ký tự** — không có ranh giới từ

**Given** văn bản tiếng Anh
**When** khớp mờ
**Then** dùng **token n-gram sau stemming**

**Given** `core/matching/`
**When** rà
**Then** vẫn chỉ có **một** cài đặt, phục vụ `dict/`, `glossary/` và `tm/`

**Given** một biến thể mà Glossary bắt được
**When** thử với TM
**Then** TM cũng bắt được — và ngược lại
