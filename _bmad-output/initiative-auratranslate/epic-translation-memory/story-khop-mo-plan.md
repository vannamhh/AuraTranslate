---
ticket: 5
title: 'Story 7.5 — Fuzzy TM match strip with match percentage and source diff'
type: 'feature'
created: '2026-10-02'
status: done
route: 'dispatch'
baseline_revision: 'd028a95aadb92a80b6a8a906194f8caeba899957'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-7-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** TM reads back only exact matches (7.4). A sentence that differs by one word from one already translated opens with nothing, so the translator retypes it. No similarity score exists in `core/matching` (only `ngrams`), and `tm_unit` is queried by equality only.

**Approach:** When the caret enters a segment with no exact pair, a Rust command scores every pair of both tiers against its source with one score function in `core/matching` built on `ngrams` (AD-17), and returns the top matches at or above the threshold, each with its percentage, a source diff (old vs current) and its origin side. The webview shows them in the `tm_fuzzy` slot of the existing inline-strip registry; accepting one writes its target through `write_non_user_target` as an unconfirmed `draft` with origin `other`.

## Boundaries & Constraints

**Decisions (Ice, 2026-10-02):**
- Q1 = A: the score function is built here in `core/matching`; Story 7.6 keeps the Glossary/TM variant-parity proof and its four ledger items.
- Q2 = A: the diff library (`similar` vs `dissimilar`) is chosen in this story by an AD from Winston (`planning-artifacts/ad-brief-2026-10-02-diff-khop-mo-tm.md`, AD-51), measured on real Zh/En sentences; both options with measurements go to Ice. The diff and accept tasks wait for that AD.
- Q3 = A: accepting a fuzzy suggestion sets origin `other` (new AD-47 ③ row, same AD). Confirm unedited ⇒ `other`, even if the pair was mine.
- Q4 = A: four new tokens `diff-add-bg`, `diff-add-ink`, `diff-del-bg`, `diff-del-ink` in both themes with contrast pairs; Sally signs the dark values.
- Q5 = B: one registered command (`Mod+Alt+E`, free on both platforms, rebindable) moves focus from the cell into the strip; inside it ↑/↓ move, Enter or 1–3 accept, Esc hides the strip for that segment and returns focus to the cell. The cell's bare `Escape` is untouched.
- Q6 = B: one settings key `tm_fuzzy_threshold` (integer percent, default 65, accepted 50–99, one parser like `parse_glossary_scan_threshold`), editable in Settings. Row count fixed at 3.
- Spec kept whole above 1600 tokens (Ice).
- AD-51 crate (Ice, 2026-10-02): option A, `similar` =3.1.1 with chars for Zh, words for En, plus the absorb pass for equal runs of 2 or fewer (`planning-artifacts/ad-51-draft-2026-10-02.md`).
- Latency (Ice, 2026-10-02): about 650-715 ms per call at 100,000 pairs per tier (release, loaded machine) is accepted; no speed-up work. The scan must not hold the `OpenWorkState` lock while scoring.

**Always:**
- One score function, in `core/matching`, for Zh (char n-grams) and En (stemmed token n-grams); the caller passes `MatchLang` from `open_work.meta.source_lang`. No second similarity or diff implementation anywhere.
- No strip when an exact pair exists (7.4 owns that), when the best score is below the threshold, or after Esc in the strip on that segment (this session).
- Order: score descending, then `pairs_for_source` order (AD-18: mine, Work, id).
- Never auto-fill below 100 %. Accept is a discrete write: flush first (`flushEditorBeforeDiscreteWrite`), status `draft`, no `SegmentVersion`, and text in a draft is never replaced without the `needs_confirmation` round-trip that `promote_ai_translation` uses.
- Accept sends the pair's identity (tier + `tm_unit.id`), not its text; Rust re-reads the pair.
- The strip renders in the shared slot above `<StatusBar />` through `topmostStrip`, so Glossary strips win (UX-DR21); every action is a registered command or a listed `HANDLER_TABLE` handler.
- Each row shows: percentage, old source with diff against the current source, old target, origin side (mine / others'), tier.
- The scan runs off the UI thread; a response for a segment the caret has left is dropped.

**Never:**
- No Concordance, multi-translation picker, management, TMX or RAG change (7.7–7.11); no Proofreader.
- No "Chapter N · sentence M" or "see whole sentence" in a row: `tm_unit` has no segment link (AD-6).
- No new `tm_unit` or `segment` column; no n-gram index table.
- No raw hex colour (`check:tokens`); no diff crate before the AD.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Hit | caret on `S'`, pair `S` scores ≥ threshold | strip lists it with %, diff `S`→`S'`, side, tier | N/A |
| Exact exists | pair with source = `S'` | no strip | N/A |
| Below threshold | best < threshold | no strip | N/A |
| Threshold setting | threshold 80, pair at 70 | no strip | out-of-range value rejected, 65 kept |
| More than 3 | 5 pairs above threshold | top 3, ordered | N/A |
| Accept empty draft | accept row 1 | target = pair target, `draft`, origin `other`, 0 versions, strip closes | N/A |
| Accept over text | draft has text | `needs_confirmation`, nothing written until confirmed | N/A |
| Esc in strip | Esc | strip hides, focus back in cell; same segment stays hidden; next segment can show | N/A |
| Glossary strip eligible | both eligible | Glossary strip shows, TM waits | N/A |
| Caret moves mid-scan | response for old id | ignored | N/A |
| Pair gone | accept a row whose pair no longer exists | nothing written | error message |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/matching/mod.rs` -- `MatchLang` (:188), `ngrams(text, lang, n)` (:374, char n-grams for Zh, stemmed token n-grams for En, empty when n exceeds the population). Add the score function here; module must stay I/O-free (AD-13/15, `tests/matching_boundary.rs`).
- `src-tauri/src/core/tm/mod.rs` -- `TmPair` (:91, add `id`), `PairOrigin::side()`, `pairs_for_source` (:171, exact, merge through `ScopeResolver`, private `load_pair_rows` :138). Add the fuzzy read beside it; keep AD-18 order for ties.
- `src-tauri/src/core/store/schema.rs` -- `TM_UNIT_DDL` (:891); only index `tm_unit_source_text`. No migration expected.
- `src-tauri/src/commands/segment.rs` -- `write_non_user_target` (:2219), `promote_ai_translation` (:2261, wrapper :3986) is the accept template (force + `needs_confirmation`). Global `Store` via `app.try_state::<Store>()`; missing is an error (7.4 rule). Register the new commands in `lib.rs` (shared wiring ⇒ full run).
- `src/panels/inlineStripPriority.ts` -- `InlineStripKind` already has `tm_fuzzy` (last); `topmostStrip`. `src/main.ts:854` repeats the eligibility for focus.
- `src/App.vue:331-343` -- slot of `GlossaryQuickAdd`/`GlossaryConfirmStrip` above `<StatusBar />`; `GlossaryConfirmStrip.vue:45-60` is the pattern (watch `editorCaretSegmentId`, Esc → dispatch).
- `src/panels/editorPanelState.ts` -- `editorCaretSegmentId` (:153), `flushEditorBeforeDiscreteWrite` (:729), `promoteAiTranslationToEditor` (:336) with `pendingPromote` (:384-395), `replaceEditorSegment` (:281), `dropTmFilled` (:132).
- `src/config/segment.ts` -- IPC adapters with runtime guards (`promoteAiTranslation` :1095).
- `src/commands/index.ts` -- `target.register` in `installCommands` (:3772); Glossary confirm commands (:3104-3138) as pattern. `scripts/check-commands.mjs` Kiểm K (:2236, `HANDLER_TABLE` :2313) must list every new `@keydown`.
- `src/tokens/tokens.json` -- `surface-tm`, `tm-rule`, `tm-text` (:22,32-33 light; :41,51-52 dark); contrast pairs :129/154/179.
- `src-tauri/src/core/scope/store.rs` -- `KEY_GLOSSARY_SCAN_THRESHOLD` (:116) and `parse_glossary_scan_threshold` are the pattern for `tm_fuzzy_threshold`; exposed through `commands/config.rs` (:103, :156) and `src/config/bootstrap.ts` (:86, :202, :276). Settings UI pattern: `src/SettingsGlossarySection.vue` + `src/glossarySettingsState.ts`.
- `src-tauri/Cargo.toml:164-167` -- the deferred-diff note; replaced by the AD-51 Stack row when the crate is added.
- Tests that move: `src-tauri/tests/tm_contract.rs`, `src-tauri/tests/matching_contract.rs`, `src-tauri/tests/matching_boundary.rs`, `src-tauri/tests/scope_contract.rs`, `src-tauri/tests/config_invariants.rs`, `src-tauri/tests/ipc_contract.rs`, `src-tauri/tests/ipc_argument_contract.rs`, `src-tauri/tests/naming_boundary.rs`, `src-tauri/tests/segment_boundary.rs`; `tests/frontend/tmFuzzyStrip.test.ts`, `tests/frontend/tmFuzzyAdapters.test.ts`, `tests/frontend/settingsFrame.test.ts`.
- UX: `ux-designs/ux-AuraTranslate-2026-08-02/mockups/tm-fuzzy-match.html` (`.band` :159-194, rules :231-236, :290-293), `EXPERIENCE.md:73-83`, UX-DR21 (`epics.md:567`).

## Tasks & Acceptance

**Execution:**
- [x] AD-51 (Winston, from `planning-artifacts/ad-brief-2026-10-02-diff-khop-mo-tm.md`) -- diff crate measured on real Zh/En sentences, Ice picks, Stack row + licence, AD-47 ③ row `other`. Blocks the diff and accept tasks only.
- [x] `src-tauri/src/core/matching/mod.rs` -- score function (Dice over `ngrams`, n per language) and, after AD-51, the diff wrapper; contract cases in `tests/matching_contract.rs`.
- [x] `src-tauri/src/core/tm/mod.rs` -- fuzzy read over both tiers: exact excluded, threshold, top 3, order; `TmPair.id`.
- [x] `src-tauri/src/core/scope/store.rs`, `commands/config.rs` -- `tm_fuzzy_threshold` key and its single parser.
- [x] `src-tauri/src/commands/segment.rs` + `lib.rs` -- `tm_fuzzy_matches(segment_id)` and `accept_tm_fuzzy(segment_id, tier, unit_id, force)` through `write_non_user_target` with origin `other`; `tests/tm_contract.rs` one case per matrix row at wire level.
- [x] `src/config/segment.ts`, `src/config/bootstrap.ts`, `src/panels/TmFuzzyStrip.vue` (+ state module), `src/App.vue`, `src/main.ts`, `src/commands/index.ts` (`Mod+Alt+E`), settings section, `src/i18n/vi.json`, `src/tokens/tokens.json` (four diff tokens), `scripts/check-commands.mjs` `HANDLER_TABLE` -- vitest for strip state, slot priority, keys and accept.

**Acceptance Criteria:**
- Given a pair `S` within threshold of the caret's source, when the scan is really removed from the command, then the Hit case goes red and Below-threshold stays green.
- Given the exact-exists case, when the exact exclusion is really removed, then that case goes red.
- Given an accepted row, when the `write_non_user_target` call is really removed, then Accept-empty-draft goes red.
- Given both strips eligible, when the `tm_fuzzy` slot ignores `topmostStrip`, then the priority vitest goes red.
- Given 100,000 pairs per tier, when the command runs on a release build, then its latency is measured and recorded with build and population; above 100 ms goes to Ice before `review`.

## Implementation Notes

- Score: Sorensen-Dice over the pooled 1-gram and 2-gram multiset (chars for Zh, stemmed tokens for En), floored, so 100 means identical multisets; units are interned and stems cached (first draft was 4.0 s per call, now about 0.7 s).
- Exact exclusion is raw `source_text` equality and lives in the command (`prepare_tm_fuzzy`), so the strip has no exact check of its own; a pair differing only in whitespace can therefore show at 100 %.
- Lock: rows of both tiers are read under `OpenWorkState` (about 160 ms per 200,000 rows), scoring and diff run after the guard drops. No test goes red if the wire shell keeps the guard.
- `accept_tm_fuzzy` re-reads the pair by tier and id, then calls `promote_ai_translation`, so it shares the AI-promote writer, `needs_confirmation` and outcome shape. A gone pair is `tm.pair_not_found` with `MessageKey::Unknown`; the webview shows its own text for that code.
- `similar` brings `bstr` into `Cargo.lock` as an optional dependency behind the off `bytes` feature; `cargo tree -i bstr` is empty, so nothing new is compiled.
- Diff tokens get their own roles (`diff-text`, `diff-fill`) instead of joining `roles.text`/`roles.surface`, which avoids 28 excluded cross pairs. Dark values `#2b3a2a`/`#a9c99a` and `#3e2824`/`#e5867a` were picked by the agent and await Sally; `DESIGN.md` colour table not updated.
- Review patches: the caret-driven scan is debounced 150 ms; a non-raw-equal pair is capped at 99 %; the strip's command deps live in one factory (`src/tmFuzzyCommandDeps.ts`) shared by `main.ts` and the test. Floors raised to live counts: `COMPONENT_FILE_FLOOR` 93 → 100, `FILE_FLOOR` (check:layout), `FRONTEND_FLOOR` (`naming_boundary.rs`) and `WEBVIEW_FLOOR` (`segment_boundary.rs`) 94 → 101.
- Counter-checks (code really removed): scan call removed ⇒ 4 `tm_contract` cases red, Below-threshold and Exact green; exact early-return removed ⇒ only the Exact case red; `diff_spans` call replaced by an empty vec ⇒ the diff case red; the promote `write_non_user_target` call removed ⇒ the three accept cases plus two AI-promote cases red, Pair-gone green; the strip ignoring `topmostStrip` ⇒ only the priority vitest red; the 99 cap removed ⇒ only the letter-case case red.
- Census in `config_invariants.rs` moved to 16 plain / 1 async in `segment.rs` and 62/42 tree-wide (Rust phase 2 had left it red).

## Spec Change Log

## Review Triage Log

- V1 (gap) `main.ts` strip deps are re-written inside `tmFuzzyStrip.test.ts`, so mis-wiring `main.ts` stays green — medium, patch: move the closures to one exported factory used by both.
- V2 (gap) top-3 test never asserts which pairs are kept — low, patch: assert the exact three targets and the two dropped ones.
- V3 (gap) latency probe is `#[ignore]` and asserts nothing — low, rejected: Ice accepted the measured latency (Decisions); it is a measurement, not a contract.
- B1 `bstr` enters `Cargo.lock` with no Stack note — low, patch: `cargo tree -i bstr` is empty (optional, `bytes` feature off), so the Stack row only needs to say so.
- B2 / E9 no debounce: holding an arrow key queues one full scan per segment, each reading every row under the `OpenWorkState` lock — medium, patch: debounce the caret-driven scan in the webview.
- B3 rows are read under the lock — false: Ice's latency decision forbids holding the lock while scoring only; the read is the accepted part.
- B4 / E12 saving a new threshold does not re-run the scan for the caret segment — low, patch: re-sync after a successful save.
- B5 a click without a prior `mouseenter` (pointer already resting on the row when the strip appears; WebKit does not focus buttons on click) accepts the aimed row, not the clicked one — medium, patch: aim the row on `mousedown` (listed in `HANDLER_TABLE`).
- B6 = V1.
- B7 out-of-range threshold accepted by generic `put_config`, bounds repeated in three places — low, rejected: same shape as `glossary_scan_threshold`; the form validates and the reader falls back.
- B8 / E14 `tm.pair_not_found` uses `MessageKey::Unknown` — low, rejected: the strip is the only consumer and maps the code; a catalogue key adds surface for no reachable caller.
- B9 accept does not check that the pair belongs to the segment — false: the webview is the only caller and sends the identity it was given (frozen rule); the retired guard runs because accept goes through `promote_ai_translation`.
- B10 / E10 scorer and exact exclusion use raw text while the diff trims and NFCs — low, rejected: both sides come from stored `segment.source_text`, CJK ideographs are NFC-stable, and the visible symptom is E11.
- E11 an En pair differing only by case scores 100 % (the scorer lowercases) yet is not exact — low, patch: cap a non-raw-equal score at 99.
- B11 English stemming and English absorb are not asserted — low, patch: a stemming case that fails without stems, and one En absorb case.
- B12 `ad-51-draft-2026-10-02.md` still says "not in the spine, Ice to choose" and points at a scratchpad harness — low, patch: one 🔵 line saying it is merged into AD-51, option A chosen, harness not kept.
- B13 `// 21 = 17 + four diff tokens.` restates the next line — low, patch: delete it and re-derive `EMITTED_VAR_FLOOR` from `ceil(0.85 × live)`; the ignore-reason string is not a comment (false).
- B14 no test for a retired segment or a whitespace source in `tm_fuzzy_matches`, nor for equal unit ids in both tiers — low, patch: tests only (equal ids happen daily, both tiers start at 1).
- E1 pressing 2 or 3 with fewer rows accepts the previously aimed row — medium, patch: ignore a digit beyond the shown rows.
- E2 an accept resolving after the caret moved clears the new segment's strip and drops its scan — low, patch: clear only when the shown segment is the accepted one.
- E3 a second Enter while the first accept is in flight sends a second write — medium, patch: refuse while `accepting`.
- E4 a failed pre-write flush is only logged — low, rejected: same as `promoteAiTranslationToEditor`; rare.
- E5 a scan error stays on screen after the caret moves to a suppressed segment — low, patch: clear the error when the segment changes.
- E6 after a chord entry, moving the caret without Esc leaves `enteredViaChord` set, so the next Esc restores focus to the old cell — low, patch: reset the saved focus when the segment changes.
- E7 accept over a confirmed segment writes without asking — false: same as AI promote (`promoting_onto_a_confirmed_segment_returns_it_to_draft…`); the confirmed text survives as a `SegmentVersion`, and the frozen rule covers drafts.
- E8 pair edited, deleted or id reused between display and accept — false: `tm_unit` is append-only (AD-6), `AUTOINCREMENT` never reuses ids, and a Work switch resets the strip.
- E13 a reset with an unchanged caret id leaves no strip until the caret moves — low, rejected: the reset paths run before the caret is re-established, so a re-scan there is not a direct fix.
- E-claim1 = E2. E-claim2 = E7. E-claim3 = E1.

## Verification

**Commands:**
- `npm run test:story 7-5` -- expected: selected Rust and vitest targets green.
- `npm run check:commands && npm run check:tokens && npm run check:i18n` -- expected: green.
- full `cargo test` + vitest once (new commands in `lib.rs`) -- expected: green.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 7.5 (v6, nay ở archive-v6).

**Covers:** FR59

As a người dịch,
I want thấy câu gần giống mình từng dịch và biết nó khác chỗ nào,
So that tôi sửa nhanh thay vì dịch lại từ đầu.

**Acceptance Criteria:**

**Given** một segment có cặp TM tương tự nhưng không y hệt
**When** mở
**Then** một **dải mọc** dưới câu đang sửa hiện các bản dịch cũ tương tự

**Given** mỗi gợi ý khớp mờ
**When** hiển thị
**Then** kèm **phần trăm khớp**

**Given** mỗi gợi ý khớp mờ
**When** hiển thị
**Then** kèm **diff phần khác biệt** giữa câu nguồn cũ và câu nguồn hiện tại

**Given** một câu kích hoạt đồng thời chốt Glossary, phát hiện Proofreader và gợi ý TM
**When** hiển thị
**Then** gợi ý TM **nhường cả hai** — bỏ qua chỉ tốn công gõ lại một câu, và câu đó vẫn nằm nguyên trong TM cho lần sau

**Given** dải gợi ý TM
**When** hiển thị
**Then** **đẩy văn bản xuống** chứ không phủ lên, và thu lại ngay khi xong

**Given** người dùng chọn một gợi ý
**When** xảy ra
**Then** văn bản chuyển vào Editor và segment ở trạng thái **chưa xác nhận**

**Given** toàn bộ thao tác trên dải
**When** thực hiện
**Then** làm được bằng bàn phím
