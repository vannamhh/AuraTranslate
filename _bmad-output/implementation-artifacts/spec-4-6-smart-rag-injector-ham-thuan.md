---
title: 'Story 4.6 — Smart RAG Injector is a pure function'
type: 'feature'
created: '2026-09-17'
status: 'done'
route: 'dispatch'
review_loop_iteration: 1
baseline_commit: 'a4a552637adbbe466c954a42a7e61b0129001157'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-4-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Confirmed Glossary terms never reach the AI. `core/ai/` is a doc-comment-only
stub, marker substitution exists nowhere in the repo (`PromptVariable::marker()` has zero
product callers), and no product code has ever asked Glossary which terms occur in a sentence
for the purpose of injection. Without this story every later AI call would grow its own prompt
string, which is what makes FR71's inspector impossible to keep honest.

**Approach:** Glossary answers the question it alone can answer — *which confirmed terms occur
in this sentence, and where* — through one new function that shares its matching and overlap
arbitration with the grid. `core/ai/rag.rs` then holds a pure assembler that turns a prompt body
plus that answer into the finished prompt and a ledger of what it injected. No IPC command, no
UI, no network — Story 4.8 calls this, Story 4.7 displays its ledger.

## Boundaries & Constraints

**Always:**
- `ai/` reaches Glossary data through exactly one function — the new injection door — and
  nothing else. Tier resolution, overlap arbitration and the confirmed-only filter all happen
  inside `core/glossary/**`.
- The grid and the prompt agree about one sentence **by construction, not by discipline**: the
  new door and `marks_for_source_text` run the same resolution, the same `find_terms` call shape
  and the same overlap arbitration from one shared implementation inside `core/glossary/**`.
  Neither re-derives the other's rule.
- The assembler is pure: no `Store`, no `ScopeResolver`, no matching, no clock, no I/O.
  Identical inputs produce a byte-identical prompt.
- The assembler never edits the prompt body except where a marker stands.
- Three-valued honesty, everywhere a result can be absent: *not asked* (the body carries no
  `{{glossary_terms}}`, so no query ran), *asked and empty*, and *not built yet* (TM until
  Epic 7) are three distinct ledger states and are never collapsed into one.
- Epic 7 fills the TM argument without changing the assembler's signature.

**Never:**
- No new table, no migration, no `#[tauri::command]`, no `vi.json` key, no frontend file.
- Do not write the literal `glossary_entry`, `load_tier`, `insert_manual_entry`,
  `confirm_translation` or `insert_candidate` anywhere under `core/ai/**` —
  `glossary_boundary.rs` matches bare substrings and will go red on a log tag or a comment.
- Do not change `find_terms` to arbitrate overlaps: its doc-comment records that decision, and
  it serves TM as well as Glossary.
- Do not move overlap arbitration out of `core/glossary/**`. `ai/` never arbitrates anything.
- Do not widen `entries_eligible_for_injection`'s return type to carry the tier label. That debt
  stays open and moves to Story 4.7, the first screen that must show tier.
- No refusal behaviour for a malformed prompt body — assembling always succeeds and reports.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Confirmed Global term in sentence | body with `{{glossary_terms}}`, term confirmed at Global | pair injected with its confirmed translation; listed in the ledger | N/A |
| Work overrides Global | same term confirmed at both tiers | Work translation injected, Global absent | N/A |
| Pending entry | matching entry has `translation IS NULL` | not injected, not listed | N/A |
| Pending Work shadows confirmed Global | Work pending, Global confirmed, same term | neither injected | N/A |
| Overlapping confirmed terms | a confirmed term and a confirmed sub-term | longest wins, ties leftmost — identical to the grid's marks | N/A |
| **Pending term overlaps a confirmed term** | confirmed `dog`, pending `dog walker`, sentence contains `dog walker` | the pending term wins arbitration and is then dropped; **neither is injected**, and the ledger says the confirmed one was suppressed by a pending overlap | N/A |
| No term matches | body has the marker, no term occurs | the `{{glossary_terms}}` line is removed; ledger records *asked, none matched* | N/A |
| Marker absent | body has no `{{glossary_terms}}` | prompt assembled; ledger records *not asked*, and **no Glossary query runs at all** | N/A |
| Unknown marker | body contains `{{chapter_context}}` | left verbatim, reported as unknown | N/A |
| TM not built | TM argument is the not-built value | the `{{tm_similar_segments}}` line is removed; ledger records *not built yet*, never *none matched* | N/A |
| TM searched | TM argument carries segments | ledger records *searched* with those segments; the marker line is still removed this story | N/A |
| Marker shares its line, nothing to inject | `Terms: {{glossary_terms}} end.` | only the marker text is removed; the line survives | N/A |
| **Marker shares its line, two or more pairs** | `Terms: {{glossary_terms}} end.` with two pairs | the pair block starts on its own line and the trailing text goes to its own line — never spliced mid-line | N/A |
| Repeated marker | `{{source_segment}}` appears twice | every occurrence expanded | N/A |
| **Body omits `{{source_segment}}`** | no source marker anywhere in the body | prompt assembled; ledger flags the sentence as missing so Story 4.7 can show it and Story 4.8 can refuse | N/A |
| **Injected text contains a marker** | a confirmed translation is literally `{{source_segment}}` | injected text is never re-scanned: it reaches the prompt verbatim | N/A |
| Work tier closed | `work` is `None` | Global tier only, no error | N/A |
| Store unreadable | the Glossary query fails | error propagates from the gathering layer; the assembler never sees it | `GlossaryError` |

## Decisions

**1. `{{chapter_context}}` stays out of the ratified vocabulary.** `PromptVariable::ALL` keeps its
three names. A body that types `{{chapter_context}}` still stores fine, scans as an unknown
marker, is left verbatim in the assembled prompt, and is named in the ledger — the behaviour
Story 4.4 already built. Adding it would force a fourth parameter (the neighbouring sentences)
onto a signature this story is meant to freeze, make Story 4.8 responsible for supplying it, and
spend BYOK tokens on four extra sentences per call. The mockup itself marks it *chưa dùng*
(`prompt-library.html:189`). This closes `deferred-work.md:12949` as declined, with the reason.

**2. `{{glossary_terms}}` expands to bare pairs and nothing else.** One `source term → confirmed
translation` per line, in the order the ledger reports. The instruction wording stays the user's,
exactly as `prompt-library.html:157-165` draws it; the *"Dùng đúng các cách gọi sau:"* line in
`prompt-inspector.html:122-127` is display chrome for that screen, not text that goes to the
provider. Consequence accepted: a user who writes no instruction around the marker sends the
model a bare list. A fixed Rust-authored header would also collide with `src-tauri/AGENTS.md:16`.

**3. A marker with nothing to inject loses its line, and the collapse is LOCAL to that removal.**
Removing the marker removes its entire line. If *that removal* brings two blank lines together,
one is dropped. A marker sharing its line with other text loses only the marker text and the line
survives. Blank lines the user typed away from any removed marker are never touched — measured in
loop 0: a whole-text collapse pass rewrote `"Doan mot.\n\n\nDoan hai.\n…{{source_segment}}"`, a
body in which nothing is removed at all, and the prompt body is the translator's own writing.
The prompt makes no claim about an empty section; the distinction between *not asked*, *asked and
empty* and *not built yet* lives in the ledger, which is what Story 4.7's inspector reads. It
never reaches the provider, so no Rust-authored display string is needed.

**4. When a pending term overlaps a confirmed one, the grid wins: arbitrate first, filter
confirmed afterwards.** Measured in loop 0: with `dog` confirmed and `dog walker` pending, the
grid marked `dog walker` alone (confirmed subset empty) while the prompt injected `dog` — two
screens describing one sentence differently. The order is now the same one the Glossary door
itself already uses for tiers (resolve first, filter `is_confirmed` last): arbitrate overlaps
across **every** resolved term, then drop the unconfirmed survivors. Consequence accepted: an
unconfirmed candidate suppresses a confirmed term it covers, so the AI receives fewer terms in
that sentence — the translator has signalled that the longer span is its own term, and injecting
a translation for a fragment of it would mislead the model. The ledger records the suppression so
Story 4.7 can show it in *"considered but not injected"*.

**5. The single door moves into `core/glossary/`.** Because Decision 4 needs the pending terms,
and the only function `ai/` was permitted to call returns confirmed entries only, matching and
arbitration move behind a **new** Glossary function that takes the sentence and returns the
confirmed terms that survived arbitration, with their spans. The grid's
`marks_for_source_text` and this new door share one private implementation of
resolve → `find_terms` → arbitrate → convert spans, so the two cannot drift. `ai/` performs
exactly one Glossary call and arbitrates nothing. The assembler stays pure by receiving that
result. Rejected alternatives: calling `marks_for_source_text` directly (it batch-looks-up
Hán-Việt for pending entries — wasted work on a per-sentence path — and would drag `DictLayers`
into `ai/`), and a second query for pending terms (kills the one-query rule and scans both
tables twice on the path `deferred-work.md:5882` demands be measured first).

**6. A body with no `{{source_segment}}` is reported, not refused.** The ledger carries an
explicit flag, the same shape `MarkerWarnings::glossary_terms_missing` already uses. Assembling
still succeeds — refusal is Story 4.8's, and the frozen Never keeps it out of here — but a prompt
with no sentence to translate must not leave this function describing itself as fine. Silent
emptiness is the failure class root `AGENTS.md` names as this project's central one.

</frozen-after-approval>

## Code Map

**Inside `core/glossary/` — where the new door goes and what it must share**

- `src-tauri/src/core/glossary/store.rs:1341` `marks_for_source_text(resolver, global, work,
  text, lang, layers, disabled) -> Result<Vec<GlossaryMark>, GlossaryError>` — the grid's path
  and the shape to factor with. It resolves both tiers **without** filtering `is_confirmed`
  (`:1319` doc states this explicitly), builds `terms` from the resolved keys, calls `find_terms`
  once (`:1381`), arbitrates via `resolve_overlaps` (`:1285`), converts byte → **codepoint**
  spans, then batch-looks-up Hán-Việt for pending entries. The new door needs everything up to
  and including the span conversion; it needs neither the Hán-Việt lookup nor `layers`/`disabled`.
- `src-tauri/src/core/glossary/store.rs:1285` `resolve_overlaps` — private, greedy
  longest-wins/leftmost-ties, re-sorted to `find_terms`' promised order. **It stays here.** Both
  callers reach it through the shared helper; `core/ai/**` never sees it.
- `src-tauri/src/core/glossary/store.rs:1218` `match_lang_for_source_lang(&str) -> MatchLang` —
  `pub` re-exported at `mod.rs:293`, on no ban list.
- `src-tauri/src/core/glossary/entry.rs:265` `GlossaryMark { start, end, tier, is_confirmed,
  translation, id, source_term, han_viet_suggestion, han_viet_status }` — `start`/`end` are
  **codepoints**. The new door's return type should use codepoints too, so the AC comparison
  needs no unit conversion to be true.
- `src-tauri/src/core/glossary/store.rs:753` `entries_eligible_for_injection` — no longer this
  story's door. Leave it, its doc-comment and its eight `glossary_contract.rs` call sites
  (`:87,129,189,231,267,1406,2145,2622`) alone; record in `deferred-work.md` that it is back to
  zero product callers and who owns that.
- `src-tauri/src/core/glossary/entry.rs:150-153` — `id` is unique only within one `Store`; key
  anything that must be unique on `source_term`, which two-tier resolution collapses by.

**The matcher — reuse, do not touch**

- `src-tauri/src/core/matching/mod.rs:470` `find_terms(text, terms, lang) -> Vec<TermMatch>`;
  `:215` `TermMatch { term_index, span }` — **byte** spans, `term_index` into the caller's slice.
  Overlaps are returned; `:211-213` says arbitration is the caller's job. `:163` `warm()` runs on
  the chapter-open path; cold jieba init is 179–329 ms and must never land inside a measurement.
- `src-tauri/tests/matching_boundary.rs:62` bars `jieba_rs`/`tantivy_stemmers` outside
  `core/matching/**`; `:82` bars `core/matching` from depending outward.

**The prompt set**

- `src-tauri/src/core/promptset/vars.rs:28` `PromptVariable` (3 variants), `:42` `ALL`, `:59`
  `marker()` → the literal `{{glossary_terms}}` / `{{source_segment}}` / `{{tm_similar_segments}}`,
  `:104` `scan_markers(body) -> MarkerWarnings { unknown_markers, glossary_terms_missing }`
  (`:75`). Reuse it; do not write a second scanner. `marker()` has zero product callers today.
- `src-tauri/src/core/promptset/mod.rs:62` `PromptSet { id, name, body }` — no language field, no
  active-set field. **There is no stored "set in use"** (`src/promptSetState.ts:19-25`:
  webview-local, zero `invoke`), so the assembler takes the body as a parameter.

**Where the new code lands**

- `src-tauri/src/core/ai/mod.rs` — 17 lines, doc-comment only, zero `pub mod`. Add the module
  declaration here. `src-tauri/src/core/mod.rs:6` already has `pub mod ai;` — do **not** add a
  `pub use ai::…` re-export (`tests/ai_boundary.rs:410` fails on it).
- `src-tauri/src/core/tm/mod.rs` — 3 lines, doc-comment only. The TM element type must live here,
  not under `core/ai/**`: `ai_boundary.rs:99` would otherwise bar Epic 7's own module from naming
  its own type.

**Gates that watch this story**

- `src-tauri/tests/glossary_boundary.rs:93` `FORBIDDEN_TABLES`, `:142` `GLOSSARY_ONLY_SURFACE`
  (bare substring match over `src-tauri/src/**`; `core/ai/` is not exempt).
- `src-tauri/tests/ai_boundary.rs:99` bars other modules from naming `crate::core::ai`; every file
  under `core/ai/**` is skipped at `:273-276`, so it says nothing about what `ai/` imports. Floors
  (`:89` `SRC_RS_FLOOR = 68`, `:65` `AI_FLOOR = 1`) are `>=` minimums. `:316` is the
  seeded-violation control to copy.
- `src-tauri/tests/config_invariants.rs:1472` `COMMAND_FILE_CENSUS` needs a row only for a file
  carrying `#[tauri::command]` (`:1666-1672`) — this story adds none.
- `src-tauri/tests/naming_boundary.rs:103` bars `Project`/`Book`/`Novel`/`Document`.
- `scripts/check-debt-owner.mjs` reads the text **after the literal `Chủ:`**. A hand-off written
  as `Chủ phần còn lại: Story 4.7` is invisible to it — write `Chủ: Story 4.7 (phần còn lại)`.

**What loop 0 measured — do not re-derive these**

- `src-tauri/tests/story_6_18_debt_probes.rs:55-108` — the `perf_probe_*` convention: `Instant`,
  a printed figure, not `#[ignore]`d when fast.
- Baseline at `baseline_commit`, measured in a separate worktree: **1664 passed / 0 failed / 20
  ignored across 60 test binaries** (62 `test result:` lines — the last two are the doc-test run
  and its `compile fail` companion, **not** binaries; counting those lines as binaries is how
  loop 0 reported "63 binaries"). `npx vitest run`: 83 files / 1189 tests.
- `deferred-work.md:5843` (tier label dropped, `id` collides), `:5882` (full scan + triple clone
  per call — measure first), `:8173` (AD-13's allowed direction has never had a positive case),
  `:12949` (`{{chapter_context}}`) — four open items owned by this story or Epic 4.

## Tasks & Acceptance

**Execution:**

- [x] `src-tauri/src/core/glossary/store.rs` — factor the shared half of `marks_for_source_text`
      (resolve both tiers → `find_terms` → `resolve_overlaps` → byte-to-codepoint) into a private
      helper, and add the new public injection door on top of it: same sentence in, confirmed
      survivors with codepoint spans out — rationale: Decision 5 buys "grid and prompt cannot
      drift" with a shared implementation, and a door that re-derives the rule buys nothing.
- [x] `src-tauri/src/core/tm/mod.rs` — declare the minimal similar-segment type the TM parameter
      carries, doc-commented as Epic 7's to grow — rationale: the element type cannot live under
      `core/ai/**` without barring Epic 7's own module from naming it.
- [x] `src-tauri/src/core/ai/rag.rs` (new) + `src-tauri/src/core/ai/mod.rs` — the gathering
      function (one Glossary call, skipped entirely when the body carries no `{{glossary_terms}}`)
      and the pure assembler returning prompt plus ledger, with the ledger carrying all three
      Glossary states, both TM states, unknown markers, the missing-sentence flag, and the terms
      a pending overlap suppressed — rationale: Decisions 3, 4 and 6 are all statements about what
      the ledger must be able to say.
- [x] `src-tauri/src/core/ai/rag.rs` — expand every marker in **one pass over the original body**,
      so injected text is never re-scanned, and never splice a multi-line block into a shared line
      (Decision 5 of the matrix: block and trailing text each get their own line) — rationale:
      measured in loop 0, a confirmed translation of the literal `{{source_segment}}` was expanded
      as a template; Glossary content is user data.
- [x] `src-tauri/tests/ai_rag_contract.rs` (new) — one case per I/O Matrix row; assert the exact
      `result.prompt` for a body with **two** injected pairs, not a single-pair `contains`;
      produce `TmInjectionStatus::Searched` from a real `Some(&[…])`; and compare the grid's marks
      against the ledger on one sentence, including the pending-overlap row — rationale: loop 0
      measured that `.take(1)` in the pair renderer and collapsing the TM arm both left the whole
      suite green.
- [x] `src-tauri/tests/ai_boundary.rs` — add the outbound gate: under `core/ai/**` the only
      `core::glossary` names permitted are the new door, its return type, `GlossaryError` and
      `match_lang_for_source_lang`. Scan the **whole file text**, not line by line, and assert the
      scan actually collected a non-zero number of names. Seed the control with a disallowed name
      inside a genuine multi-line `use …glossary::{` group, with a real newline — rationale: loop 0
      measured that the line-by-line predicate inspected **zero** names in `rag.rs` and that
      seeding `marks_for_source_text` into the multi-line group left both boundary suites green.
- [x] `src-tauri/tests/ai_boundary.rs` — assert the Glossary door is named **at most once** under
      `core/ai/**` — rationale: "exactly one query" is an AC that nothing enforces today; a second
      call site leaves every suite green.
- [x] `src-tauri/tests/ai_rag_contract.rs` — a `perf_probe_*` that times the **whole per-sentence
      path** (the gathering call on a sentence that actually matches seeded terms), taken after
      `warm()`, with the assertion outside the timed loop, reporting per-call cost, row count and
      build profile — rationale: `deferred-work.md:5882` is about cost per translated sentence;
      loop 0 timed only the tier load, on seeded terms that matched no sentence.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — close or re-own the four items
      named in the Code Map, plus one new item for `entries_eligible_for_injection` returning to
      zero product callers. Every hand-off must be written as `Chủ: <owner>`; state build and
      population on every measurement; do not delete or absorb the `- source_spec:` line that
      opens a neighbouring item — rationale: loop 0 broke exactly that line and `check:debt-owner`
      stayed green, because it only counts **open** items with no owner.

**Acceptance Criteria:**

- [x] Given `core/ai/**` after this story, when the Glossary door is replaced by any other route
      to Glossary data — **including inside a multi-line `use` group** — then the boundary case
      goes red and names the offending identifier.
- [x] Given one sentence and the same two tiers, when the grid's marks and the injector's ledger
      are computed, then every term the ledger injects is a confirmed mark at the same span, and
      every confirmed mark the grid drops to a pending overlap is absent from the ledger and named
      as suppressed.
- [x] Given a prompt body and a sentence, when the assembler is called twice, then the two prompts
      are equal byte for byte and the ledger lists the same terms in the same order.
- [x] Given the whole repository after this story, then no `#[tauri::command]` was added, no
      migration step was added to either ladder, and `capabilities/main.json` still grants exactly
      three permissions.
- [x] Given the gates and suites after this story, then `cargo test --locked` and `npx vitest run`
      are green with the new cases counted and named against the measured baseline of 1664 / 60
      binaries, and the eleven `pre-push` gates exit 0.

## Implementation Notes

## Spec Change Log

- 2026-09-17 — **Loop 1, triggered by three `intent_gap` entries in the loop-0 review** (rows 4,
  13 and 14 of the Review Triage Log). Amended: the frozen Boundaries now require the grid and the
  prompt to agree *by construction*; the matrix gained rows for a pending overlap, a shared line
  with content, a body missing `{{source_segment}}` and injected text containing a marker; and
  Decisions 3–6 were added or corrected. Known-bad state avoided: two screens describing one
  sentence differently, with no test able to see it, and a prompt leaving with no sentence to
  translate while the ledger reports nothing wrong.
  **KEEP — what loop 0 got right and must survive re-derivation:** the two-layer split (impure
  gatherer + pure assembler) and its justification against AD-14; the three-valued TM argument and
  its `AGENTS.md` rationale; keying the ledger on `source_term` rather than the non-unique `id`;
  the local, removal-anchored blank-line collapse together with its two guard cases and both
  mutation counter-checks; the seeded-violation control style in `ai_boundary.rs`; and the
  measured baseline (1664 / 0 / 20, 60 binaries) so it is never re-derived from `spec-4-5`'s
  figure, which is 2 low.
  **DROPPED deliberately:** moving `resolve_overlaps` into `core::matching`. Decision 5 keeps all
  arbitration inside `core/glossary/**`, so the move is no longer needed and `core/matching` is
  left untouched.

## Review Triage Log

Pass 1 — 2026-09-17, three layers over a 125 kB diff (blind-hunter 13 · edge-case-hunter 12 ·
verification-gap 3 + 2 other). Findings from the verification-gap layer arrive pre-verified. Every
other claim was re-checked at its cited location before a verdict, and the four that decide the
outcome were settled by **running code**, not by reading it. The tree was SHA-256 re-compared after
the reviewers' own mutations and matches the diff they read, byte for byte.

| # | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|
| 1 | (all three layers) The new outbound gate inspects **zero** names in the only file it exists for: `names_after_glossary_double_colon` parses per line, and `use crate::core::glossary::{` with no `}` on that line hits `else { continue }` | high | **patch** | **Measured by me, not read.** Seeding `marks_for_source_text` — a real second route into Glossary data, on no ban list — *inside* `rag.rs`'s existing multi-line `use` group leaves `cargo test --test ai_boundary --test glossary_boundary` **fully green (8 passed, 13 passed)**. My own step-03 counter-check used a single-line seed and passed, which is exactly how a positive control certifies a gate around its blind spot. AC 1 does not hold for the import shape rustfmt produces. |
| 2 | (blind-hunter, edge-case ×2) The seeded control only plants single-line shapes; its "grouped use" negative case is a `\`-continued literal, which Rust joins into one line | high | patch | Same root cause as #1 — the control never contains a real `\n` inside `glossary::{…}`, so it certifies the predicate on a form the production file does not use. |
| 3 | (blind-hunter) A disallowed name could enter via a bare import plus a bare call, which the `glossary::`-anchored predicate never sees | medium | patch | Real, and subsumed by the #1 fix only for the qualified-path half: a whole-file scan catches `use …glossary::{list_all_entries}`, but a later bare call is still invisible. Grouped with #1; the fix must read imported identifiers, not only qualified paths. |
| 4 | (edge-case, high) Grid and ledger **arbitrate over different populations**: `marks_for_source_text` runs `resolve_overlaps` over all resolved entries (its doc: *"**không lọc** `is_confirmed`"*), `find_glossary_injections` over confirmed-only | high | **intent_gap** | **Measured.** Global `dog` confirmed + `dog walker` pending, sentence `"The dog walker arrived."`: grid marks `[("dog walker", false)]`, confirmed subset **∅**; ledger injects **`["dog"]`**. The frozen Boundaries say the two *"must never disagree about one sentence"* — they do. Neither new AC case seeds an overlapping **pending** term. See the note below the table: two frozen constraints collide and only Ice can cut it. |
| 5 | (verification-gap, pre-verified) `render_glossary_pairs` is only ever asserted with **one** injected pair | medium | patch | Filed with its own mutation: inserting `.take(1)` — so only the first term reaches the prompt — leaves `ai_rag_contract` at **19 passed, 0 failed**. The two assertions that read pairs out of `result.prompt` are single-term `contains`; the multi-term cases assert on the ledger only. A regression dropping every term but the first ships green. |
| 6 | (verification-gap + blind-hunter) The TM `Searched` branch is produced by **no** test; `SimilarSegment` is constructed by none | medium | patch | Filed with its own mutation: rewriting the arm to `Some(_) => NotBuiltYet` — collapsing the two states the story calls its central invariant — leaves `ai_rag_contract` at **19 passed, 0 failed**. All 19 call sites pass `None`. |
| 7 | (blind-hunter, edge-case) Injected Glossary content is re-expanded by the later marker passes | medium | patch | **Measured by me.** A confirmed translation of the literal `{{source_segment}}` yields prompt `"widget → A widget here.\n---\nA widget here."` — user data expanded as template — and `unknown_markers` stays `[]` because `scan_markers` reads the original body. Exotic input, but it is user data treated as code. |
| 8 | (blind-hunter) Nothing enforces *"exactly one Glossary query"*; the allow-list permits the door anywhere under `core/ai/**` and no test counts occurrences | medium | patch | Verified by reading the gate: a second call site, or a loop inside `gather_glossary_context`, leaves every suite green. One occurrence-count assert closes it. |
| 9 | (blind-hunter) The perf probe times only the gather half, and its seeded terms (`term-g-{i}`) appear in no sentence, so `find_terms` cost is never measured — while `deferred-work.md:5882` is about cost **per translated sentence** | medium | patch | Verified against the probe body and the debt text. The recorded entry reads as though the per-sentence cost now has a number; it does not. Either measure `assemble_prompt` on a matching sentence, or narrow the entry to say which half was measured. |
| 10 | (blind-hunter) The hand-off prose `**Chủ phần còn lại: Story 4.7/4.8**` is invisible to `check:debt-owner`, which reads the text after the literal `Chủ:` | medium | patch | Verified against `scripts/check-debt-owner.mjs`. Both items stay green only on their pre-existing `Chủ: Epic 4` line — a stale owner — while the new hand-off is unreadable to the gate. Same class as the `Chủ: chưa phân` miss the gate was already patched for once. |
| 11 | (blind-hunter) `translation.clone().unwrap_or_default()` puts a silent empty stand-in where the code says it cannot happen | low | patch | Unreachable today (`is_confirmed()` is `translation.is_some()`), which is exactly why no gate would catch a regression. The fix is a direct simplification — `filter_map` over `(entry, translation)` pairs so the type carries the guarantee — not an added guard, so it clears the `low` bar. |
| 12 | (edge-case) The `resolve_overlaps` move note was left as `///`, so it now opens `marks_for_source_text`'s rustdoc | low | patch | Verified in the diff at `core/glossary/store.rs`: the note sits between the removed body and the next item's own doc-comment. One character (`///` → `//`) fixes it; a direct correction. |
| 13 | (blind-hunter, edge-case) A marker sharing its line **with content to inject** splices a multi-line block mid-line (`"Terms: a → b\nc → d end."`) | medium | **intent_gap** | Real and unspecified: the matrix's *"Marker shares its line"* row and its case both use a sentence with no matching term, so only the empty variant is covered. What the shared-line form should do when there IS content is a product decision, not an inference — folded into the same question to Ice as #4. |
| 14 | (blind-hunter, edge-case) A body with no `{{source_segment}}` assembles "successfully" with no sentence to translate, and the ledger has no field to say so | medium | **intent_gap** | Verified: `MarkerWarnings` reports only `glossary_terms_missing`; `InjectionLedger` carries Glossary, TM and unknown markers. The frozen Never says assembling *"always succeeds and reports"* — it does not say what it reports here, and the fix adds public surface to a ledger Story 4.7 reads. Folded into the question to Ice. |
| 15 | (edge-case) A removed marker line preceded by non-blank text and followed by 2+ blank lines leaves 2+ blanks | low | rejected | The blank pair already existed in the user's body; the removal did not create it, so Decision #3's *"if THAT leaves"* does not reach it. Collapsing it would be the whole-text behaviour just removed as a defect. |
| 16 | (edge-case) Blank line at the body's leading/trailing edge after a removal | low | rejected | Real but cosmetic inside a prompt, and the fix adds edge branches to a function just simplified. Fails the `low` bar on the complexity half. |
| 17 | (edge-case) A body of only marker lines with nothing injected yields an empty prompt | low | rejected | Reachable only by a body carrying no prose at all, which the prompt editor makes obvious; Story 4.8 is the layer that refuses to send, and refusal behaviour is explicitly out of this story's frozen scope. |
| 18 | (edge-case) Cold jieba init (179–329 ms) can land inside `assemble_prompt` when `warm()` never ran | low | defer | Real for a caller that skips `warm()`, but no product caller exists until Story 4.8, and `warm()` already runs on the chapter-open path. Belongs with 4.8's own measurement, not here. |
| 19 | (verification-gap, other) The perf probe keeps `assert_eq!(result.len(), 250)` inside the timed loop | low | rejected | Negligible at this magnitude and the recorded numbers state their build and population; folded into #9's rewrite of that entry rather than carried separately. |
| 20 | (blind-hunter, aside) The unreadable-store case hard-codes `to_version: 4`/`5` with no assertion about the real ladder's length | low | defer | Real drift risk the first time a migration step is added, but pre-existing in shape and not caused by this story's intent. |
| 21 | (blind-hunter) `sprint-status.yaml` says `in-progress` while the spec ships `in-review` | false | rejected | Not a defect: the workflow sets `in-progress` at implementation and syncs the story to `review` at presentation. The diff was taken mid-flight, which is the state it is supposed to be in. |


Pass 2 — 2026-09-18, three layers over a 139 kB diff (blind-hunter 18 · edge-case-hunter 16 ·
verification-gap 6 + 4 other). No `intent_gap` and no `bad_spec`: where the code and the spec
disagreed this pass, the spec was right and the code did not follow it, so `review_loop_iteration`
stays 1. The tree was SHA-256 re-compared after the reviewers' own mutations
(`rag.rs` `1f37e2f5…`, `store.rs` `8e2c6460…`) and the diff is byte-identical to the one they read.

**The pattern of this pass:** five separate mutations to product code left all 25 contract cases
green. Loop 0's lesson was that a positive control can certify a gate around its blind spot; loop 1
shows the same shape one layer down — the renderer is now pinned, but the thing that *produces*
what it renders is not.

| # | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|
| 22 | (verification-gap) `an_unreadable_store_…` never calls the gathering layer — it asserts `Store::open(..).is_err()` and stops | high | patch | Pre-verified by mutation: changing the `?` on the Glossary call to `.unwrap_or_default()` leaves **25 passed, 0 failed**. The matrix row *"Store unreadable ⇒ `GlossaryError`"* is covered by nothing, and a swallowed error would report `Asked { injected: [] }` — "asked and found nothing" — the exact silent-emptiness class this story's three-valued ledger exists to prevent. I confirmed the body by reading it; the test's name is a claim its body does not make. |
| 23 | (verification-gap) The rag-layer ledger never observes a **non-empty** suppression list | high | patch | Pre-verified by mutation: mapping `suppressed_by_pending_overlap` to `Vec::new()` at the rag layer leaves **25 passed, 0 failed**. The two tests that read the field expect it empty; the one test with a real suppressed term asserts on the **door's** outcome and never goes through `gather_glossary_context`. Decision 4's entire reason for existing can be dropped silently. |
| 24 | (blind-hunter, edge-case ×2) The ledger drops `start`/`end`/`tier`, which the door already computes | high | patch | **Verified by me.** `GlossaryInjectionTerm` carries `start`, `end`, `tier`; `InjectedGlossaryTerm` carries `source_term` and `translation` only. AC 2 says *"every term the ledger injects is a confirmed mark at the same span"* — the ledger has no span, so the AC is unverifiable at the surface Story 4.7 reads, and the row-6 case compares the **outcome**, not the ledger, and compares no spans. The Code Map's reason for codepoint units ("the AC comparison needs no unit conversion") describes a comparison nothing performs. Fix keeps fields a type *this diff created* already receives — no surface grows beyond what the AC requires, so this stays a patch rather than a spec loopback. |
| 25 | (verification-gap) `marker_absent_runs_no_glossary_query_at_all` — the trap is inert | medium | patch | Pre-verified: the test deletes `global.db` then calls `open_global` again, which creates a fresh valid database, so a query that *did* run would succeed. Moving the early return to after the Glossary call leaves **25 passed, 0 failed**. The repo's own `store.close()` pattern (`glossary_marks_contract.rs:534-562`) makes a run observable. |
| 26 | (verification-gap) No door-level case ever produces **two** injected pairs | medium | patch | Pre-verified: `.take(1)` inside `confirmed_terms_for_injection`'s `injected` chain leaves `ai_rag_contract` + `glossary_contract` + `glossary_marks_contract` all green. This is loop 0's finding #5 moved one layer down — the renderer is pinned by an exact two-pair assertion built from hand-written literals, while the producer is not. |
| 27 | (verification-gap) The orphan-`{{` guard is executed by no case | medium | patch | Pre-verified: disabling it leaves **25 passed, 0 failed**. A body such as `"Dùng {{ để mở marker: {{source_segment}}"` then swallows the real marker, the sentence never enters the prompt, and `source_segment_missing` stays `false` because the literal *is* in the body — a sentence-less prompt with a clean ledger. |
| 28 | (blind-hunter, edge-case) `source_segment_missing` is a second scanner: `body.contains(marker())` rather than the token walk | medium | patch | Verified: the Code Map says *"Reuse it; do not write a second scanner."* Divergence: `"{{{source_segment}}"` — `contains` finds the marker at index 1 and reports *not missing*, while the expander tokenises `{source_segment` and leaves it verbatim. Decision 6 exists precisely to stop a sentence-less prompt describing itself as fine. |
| 29 | (blind-hunter) `expand_prompt_body` hand-copies `PromptVariable::from_token`'s body | medium | patch | Verified at `core/promptset/vars.rs`: `ALL`'s doc-comment claims it is the only source of truth with no second hand-written copy anywhere in the repo. This diff makes that sentence false. Making `from_token` `pub(crate)` and calling it is a direct correction. |
| 30 | (blind-hunter, edge-case) Blank-line collapse reads `prev_line_blank` from the **original** body | medium | patch | Verified by reading: for `"A\n\n{{glossary_terms}}\n{{tm_similar_segments}}\n\nB"` with nothing to inject, neither removal sees a blank neighbour — each sees the *other marker line* — and the output carries two blank lines, which is what Decision 3 says to collapse. No case covers two adjacent marker-only lines. |
| 31 | (blind-hunter) `the_glossary_injection_door_is_called_at_most_once_under_core_ai` asserts `total_calls <= 1`, so **zero** calls is green | medium | patch | Verified by reading the gate. Its sibling in the same file got this right (`total_names_collected > 0`). Deleting the only call site keeps this one green — the same vacuous-gate shape loop 0 was built to close. `== 1` is a one-character fix. |
| 32 | (edge-case) `glossary_names_named` collects nothing from a glob `use crate::core::glossary::*;` | medium | patch | Verified by reading the predicate: it anchors on `glossary::` and then reads an identifier or a `{…}` group; `*` yields an empty name and is skipped. Every Glossary name would then be reachable by bare call with the gate green — loop 0's blind spot in a different shape. |
| 33 | (verification-gap, other) `glossary_boundary.rs`'s failure message still names `entries_eligible_for_injection` as the one permitted door | medium | patch | Verified by seeding: the gate reddens correctly but prints *"Module khác chỉ được gọi `core::glossary::entries_eligible_for_injection`"* — the function this story just demoted to zero product callers. The next person to hit that gate is told to route through the abandoned door. |
| 34 | (blind-hunter) The perf entry concludes "reloads BOTH tiers every sentence" from a run with `work: None` | medium | patch | Verified: the probe passes `None` for the Work tier and seeds 500 Global-only rows, so exactly **one** `load_tier` ran. `AGENTS.md` requires a measurement to state the population it ran on; the conclusion states a population the run did not have. |
| 35 | (edge-case) The same confirmed term occurring twice in one sentence renders a duplicate pair line | medium | patch | Verified by reading: `find_terms` returns both spans, the door emits both, and `render_glossary_pairs` maps each one, so the prompt carries `X → Y` twice. Those are BYOK tokens and the instruction reads as broken. Dedupe in the renderer only; the ledger keeps both occurrences for the inspector. |
| 36 | (blind-hunter) `#[derive(Default)]` on `GlossaryInjectionOutcome` re-opens the collapse the enum exists to prevent | low | patch | `Default::default()` is byte-identical to *"asked, matched nothing"*, twenty lines from the enum introduced so that state could not be confused with any other. Nothing constructs it today; deleting the derive is a direct deletion. |
| 37 | (blind-hunter) `6,633 ms` is ambiguous against this file's own `.` thousands separator | low | patch | Verified: the probe prints Rust `Duration` Debug (`6.633ms`) while the entry writes a comma, and the same file writes `94.760 B` for a byte count. Record the figure in the form the probe emits. A one-character class of fix in a measurement record. |
| 38 | (blind-hunter) The suppression dedup key `(term_index, span.start, span.end)` can never fire | low | patch | Verified: `find_terms` already returns each such triple at most once. What *can* duplicate is the ledger row — one confirmed term suppressed at two occurrences emits two indistinguishable entries. Closed by #24 (spans reach the ledger), which makes the rows distinguishable. |
| 39 | (blind-hunter) "Confirmed" is re-derived three times inside the door instead of calling `is_confirmed()` | low | patch | Verified: `translation.clone()?`, a `let … else`, and `.translation.is_none()` each restate the rule the named predicate owns. Direct correction; no new surface. |
| 40 | (blind-hunter) `core/glossary/mod.rs`'s new doc block links to a private `fn` and mis-states an access level | low | patch | Verified: `resolve_and_match` is a bare private `fn`, so the intra-doc link is broken and *"vẫn `pub(super)`/private"* is half wrong. Direct correction. |
| 41 | (edge-case, blind-hunter) `resolve_and_match` stops after `find_terms`, so arbitration and span conversion are sequenced separately at both call sites | low | rejected | Verified — and weaker than filed: `resolve_overlaps`, `codepoint_boundaries` and `byte_to_codepoint` are each **one shared function** called from both places; only the ordering is written twice. The frozen invariant asks for one implementation of the rule, which holds. Outcome-level drift is now caught by the grid-versus-ledger case. Folding the rest into the helper is a refactor, not a direct correction, and the door additionally needs the pre-arbitration matches the helper would consume. |
| 42 | (blind-hunter, edge-case) The shared-helper refactor made the grid's chapter-open path clone every resolved entry | medium | defer | Verified: `resolve_and_match` returns `Vec<(GlossaryTier, GlossaryEntry)>` where `marks_for_source_text` previously borrowed. Real, and caused by this change — but the number that would size it belongs to the chapter-open path, which this story has no harness for, and the fix is a lifetime refactor rather than a correction. |
| 43 | (verification-gap) "Exactly one query" is a textual occurrence count: a loop around the single call site still reads as one | medium | defer | Filed with its reasoning read rather than run. Closing it properly needs a query counter `Store` does not expose; Story 4.8 is the first caller where call frequency is measurable. The zero-call half is fixed here as #31. |
| 44 | (edge-case) CRLF bodies leave an orphan carriage return when a line is collapsed | low | defer | Real and reachable — `.prompt.md` files imported on Windows (Story 4.5) can carry CRLF — but every figure in this story was measured on macOS and the repo's Windows story is read from CI, not from here. |
| 45 | (edge-case) `debug_assert_eq!(resolver.has_work_tier(), work.is_some())` is inert in release | low | rejected | Pre-existing shape, copied verbatim from `entries_eligible_for_injection` (`store.rs:767-774`), not introduced by this change. Turning it into a returned error is a behaviour change to a pattern the codebase applies consistently. |
| 46 | (blind-hunter, verification-gap) `count_calls(&joined_code(&import_only), …)` applies `joined_code` twice | low | patch | Verified in the diff: `import_only` is already that function's output. Harmless, but it sits in the only control that proves the import-versus-call distinction, so a reader must check idempotence before trusting it. Direct correction. |
| 47 | (blind-hunter) The new gate's recorded blind-spot list names only module aliasing | low | patch | Verified: a trailing or block comment mentioning a barred name inside `core/ai/**` reddens the gate with no violation, and a nested group stops at the first `}`. Recording both in the `GIỚI HẠN THẬT` block is a documentation correction; #32 fixes the glob half in code. |
| 48 | (blind-hunter) Exact-match asserts ratify a trailing space in provider-bound text (`"Terms: \nNothing here."`) | low | rejected | Real, but the whitespace is what the user typed around their own marker; trimming it would be the assembler editing the body away from a marker, which the frozen Boundaries forbid. The tests ratify the correct behaviour. |

**Patch round applied, then verified independently.** Twenty-one `patch` entries went back to
the loop-1 implementation subagent (the same one, re-engaged — not a fresh launch). Three entries
route to `defer` and are now in `deferred-work.md` with real owners: the grid path's new per-entry
clone (Story 10.9, the story with a chapter-open harness), the textual-only "exactly one query"
count (Story 4.8), and CRLF bodies leaving an orphan carriage return (Story 4.8). Three were
rejected on their refutation (#41, #45, #48).

The three mutations the review used to prove the holes were re-run by the orchestrator after the
patch round, and each now reddens exactly one case: swallowing the Glossary error reddens
`an_unreadable_store_propagates_a_glossary_error_from_the_gathering_layer`; a glob
`use crate::core::glossary::*;` reddens the outbound gate; dropping the rag-layer suppression list
reddens the pending-overlap case. The first attempt at the error-swallowing mutation did **not**
compile — `#[derive(Default)]` had just been removed by patch #36, so the shorthand for
manufacturing an empty outcome no longer exists. That red was a compile error, not evidence, and
was replaced with an explicit struct literal before any verdict was read.

**Final numbers, measured by the orchestrator on a quiet tree, 2026-09-18:**

- `npm run build` — green, 0 errors.
- `cd src-tauri && cargo test --locked` — **1701 passed / 0 failed / 20 ignored across 61 test
  binaries**. Against the measured baseline (1664 / 0 / 20, 60 binaries): **+37 cases, +1 binary**,
  and the accounting closes per file — `ai_rag_contract.rs` 31 (new), `ai_boundary.rs` 6 → 12,
  `glossary_boundary.rs` 13 → 13 (its failure message changed, not its case count). 31 + 6 = 37.
- `npx vitest run` — **83 files / 1189 passed**, unchanged; the diff contains no frontend file.
- All eleven `pre-push` gates run individually — every one exit 0.
- `deferred-work.md` — 206 `- source_spec:` items (203 + the 3 deferred entries above); the
  structural corruption loop 0 introduced has not recurred.

## Design Notes

**How the signature satisfies AD-14 without a `Store` in the pure half.** AD-14 writes the
injector as `(source sentence, scope, Glossary, TM) -> assembled prompt`. Purity and a
`ScopeResolver` pull in opposite directions: resolving the scope means reading two stores. The
story splits the sentence in two — the gathering function holds `(resolver, global, work)` and
makes the single Glossary call; the assembler receives what that produced. Together they are
AD-14's signature; separately, only the second has to be pure, and it is the one the whole matrix
tests. Epic 7 adds its argument to the same two functions without reshaping either.

**Why the matching moved to Glossary rather than the arbitration moving to `ai/`.** Loop 0 put
`find_terms` + arbitration inside `core/ai/rag.rs` and shared only the arbitration helper. That
made the two paths share a *rule* while running it over *different populations* — the grid over
every resolved term, the prompt over confirmed ones only — and the disagreement it produced was
invisible to every test. Sharing the whole resolve-match-arbitrate step instead makes the
agreement structural: there is one implementation, one population, one arbitration, and the only
difference is which survivors each caller keeps.

**The TM argument is three-valued, not two.** Root `AGENTS.md` names silent emptiness as this
project's central failure class, with the specific instruction that a value which can be UNKNOWN
takes an `Option` rather than a zero. TM is not empty today — it does not exist. An argument that
cannot tell "Epic 7 has not been built" from "the search ran and matched nothing" would hand the
model the second statement while the first is true, and no gate would notice. The same shape now
covers Glossary: *not asked* is a third state beside *asked and empty*.

## Verification

**Commands:**

- `npm run build` before any `cargo test` — without `dist/`, `cargo test` breaks at compile time.
- `cd src-tauri && cargo test --locked` — expected: 0 red. Baseline is **measured, not inherited**:
  1664 passed / 0 failed / 20 ignored across 60 binaries at `baseline_commit`. Name every new case.
- `npx vitest run` — expected: 0 red and unchanged at 83 files / 1189 tests; this story touches no
  frontend file.
- The eleven `pre-push` gates, run individually: `check:deps` · `tokens` · `i18n` · `commands` ·
  `layout` · `panel-refs` · `dict` · `dict-manifest` · `lint` · `gates` · `debt-owner`.
- Counter-check the outbound boundary gate **in the shape the code actually uses**: seed a
  disallowed `core::glossary` name inside the multi-line `use …glossary::{` group in `rag.rs`, run
  `cargo test --test ai_boundary --test glossary_boundary`, and read **why** each red fires — one
  must name the seeded identifier, not a compile error. A single-line seed does not discharge this.
- Counter-check the shared implementation by removal: make the new door stop going through the
  shared helper and run `glossary_marks_contract` and the new contract file — the grid-versus-ledger
  case must go red naming both sides.
- Counter-check the pair renderer: insert `.take(1)` into it; a case must go red. Counter-check the
  TM arm: collapse `Some(..)` into the not-built state; a case must go red.
- Counter-check the perf probe: run it twice on a quiet tree and report both figures with the row
  count and the build profile. A figure taken while another suite runs is not a measurement.
- After push, read CI. Every figure above is macOS-only and says nothing about Windows or UTC.
