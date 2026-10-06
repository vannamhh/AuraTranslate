---
type: handoff
title: "Story 7.5 — phase handoff (working notes, not spec)"
status: done
created: 2026-10-02
skill: bmad-build
---

# Story 7.5 — phase handoff (working notes, not spec)

Spec: `spec-7-5-khop-mo.md`. Baseline `d028a95aadb92a80b6a8a906194f8caeba899957`.

Blocked until AD-51 (Winston, `planning-artifacts/ad-brief-2026-10-02-diff-khop-mo-tm.md`): the diff crate and wrapper, the diff field on the wire, `accept_tm_fuzzy`, and the spine AD-47 ③ row. Phases before that leave those out and say so here.

## Rust phase

Done (all in `src-tauri/`, nothing committed):
- `core/matching/mod.rs`: `SimilarityScorer::new(query, lang)` + `.percent(candidate) -> u8` (Dice over pooled 1-grams and 2-grams, interned, sorted-vec intersection, floor, so 100 = identical multisets; 0 on an empty side). Replaces the first draft (`SimilarityProfile`), which was 4.0 s per 2x100k; interning and a stem cache brought it to ~0.7 s.
- `core/tm/mod.rs`: `TmPair.id`; `fuzzy_pairs_for_source(resolver, global, work, source, lang, threshold) -> Vec<FuzzyPair{pair, percent}>` (full scan of both tiers, raw-equal source excluded, top `FUZZY_MATCH_LIMIT`=3, stable sort by percent desc keeps the AD-18 order from `merge_tiers`). `pairs_for_source` now shares `merge_tiers`.
- `core/scope/store.rs`: `tm_fuzzy_threshold` key, `parse_tm_fuzzy_threshold` (50..=99 else 65), `GlobalConfig::tm_fuzzy_threshold()`. Out-of-range is rejected on READ (falls back to 65), the write path is the generic `put_config` like `glossary_scan_threshold`.
- `commands/config.rs`: `BootstrapConfig.tm_fuzzy_threshold: u32` (9th wire field, `ipc_contract.rs` updated). Webview must add it to `src/config/bootstrap.ts` and Settings.
- `commands/segment.rs` + `lib.rs`: `wire::tm_fuzzy_matches(segmentId)` (`#[tauri::command(async)]`, registered). Wire shape: `{segment_id, matches: [{tier: "work"|"global", unit_id, percent, source_text, target_text, side: "mine"|"others"}]}`. Empty when an exact pair exists, below threshold, or blank source. Errors: no Work, no global Store, segment missing/retired.

NOT done (blocked on AD-51): diff wrapper and the `diff` field on each match; `accept_tm_fuzzy(segment_id, tier, unit_id, force)` (will use `write_non_user_target(..., TRANSLATION_ORIGIN_OTHER, Some(OTHER))`, force + `needs_confirmation` like `promote_ai_translation`); AD-47 3 row; Stack row.

Tests added: `tm_contract.rs` (8 fuzzy cases + 1 ignored latency probe), `matching_contract.rs` (4), `scope_contract.rs` (1 parser). Counter-checks: removing the exact early-return in the command turned `an_exact_pair_suppresses_the_fuzzy_list` red alone; removing the scan call turned 4 cases red while `a_pair_below_the_threshold_gives_no_match` and the exact case stayed green.

Latency (AC5, release build, this Mac, load under 1 min of nothing else, synthetic 9-word En sentences, 100,000 pairs in each tier, no match above threshold): ~650-715 ms per call (~82 ms of it is reading 100k rows per tier, ~310 ms per tier scoring). Over the 100 ms line => goes to Ice before `review`. Options to measure, not picked: (a) std::thread::scope chunked scoring, (b) a cheap pre-bound on gram count before building a profile, (c) both. Both leave read time (~160 ms for 200k rows) over 100 ms unless rows are filtered in SQL, and an n-gram index table is forbidden by the spec.

Risk: the wire holds the `OpenWorkState` mutex for the whole scan, so other commands wait up to that latency.
Note for webview: the `exact exists` early return lives in the command, so the strip needs no exact check of its own; `unit_id` + `tier` is the accept identity.

Full `cargo test --no-fail-fast` (debug, after the fixes above): everything green except `ipc_argument_contract::every_registered_commands_business_parameters_match_camel_case_across_the_wire`, which is red ONLY because `tm_fuzzy_matches` is registered with no `invoke('tm_fuzzy_matches', { segmentId })` in `src/**/*.ts` yet. The webview phase's adapter in `src/config/segment.ts` turns it green; re-run `cargo test --test ipc_argument_contract` then. `tests/config_invariants.rs` census edited (segment.rs now 15 plain / 1 async / 1 case row; tree 61/42) because the shell is `(async)`.

## AD-51 landed (orchestrator)

AD-51 is in the spine (after AD-50): `similar` =3.1.1, `diff_spans(old, new, lang)` in `core/matching`, wire `DiffSpan{kind, text}`, NFC both sides, absorb pass on `equal` changes of 2 chars or fewer, AD-47 ③ row `other`. Ice accepted the ~0.7 s latency (spec Decisions); the only latency work left is releasing the `OpenWorkState` lock before scoring.

## Rust phase 2

Done (src-tauri/, nothing committed):
- `Cargo.toml`: `similar = "=3.1.1"` (replaces the deferred-diff note). `Cargo.lock` also gains `bstr` 1.13.1: it is an OPTIONAL dependency of `similar` (feature `bytes`, off) so it is listed but never compiled (`cargo tree -i bstr` empty); the spine's "0 gói MỚI" holds for the build.
- `core/matching`: `diff_spans(old, new, lang) -> Vec<DiffSpan{kind: DiffKind, text}>` (`kind` serialises `equal|delete|insert`), trim + NFC both sides, chars for Zh, words for En, equal runs of 2 chars or fewer between two changes fold into a delete+insert pair. `matching_boundary.rs` `MATCHING_ONLY_CRATES` now includes `similar`.
- `core/tm`: `load_fuzzy_candidates` + `rank_fuzzy_candidates` (the old `fuzzy_pairs_for_source` is their composition), `pair_by_id(store, tier, id)`.
- `commands/segment.rs`: `tm_fuzzy_matches` is now `prepare_tm_fuzzy` (reads under the lock: segment, exact check, all candidate rows, resolver clone) + `score_tm_fuzzy` (scores and diffs only the kept <=3). The wire shell drops the `OpenWorkState` guard before scoring. Each match now has `diff: [{kind, text}]`. Reading ~160 ms/200k rows still happens under the lock; scoring does not.
- `accept_tm_fuzzy(global, open, segment_id, tier, unit_id, force)` re-reads the pair by tier + id and calls `promote_ai_translation` (so the one `write_non_user_target(.., other, Some(other))`, `needs_confirmation`, `unsigned_draft`, same `PromoteAiTranslationOutcome` wire shape). Wire: `accept_tm_fuzzy(segmentId, tier, unitId, force)`, registered in `lib.rs`. Unknown tier or missing pair => `IpcError` code `tm.pair_not_found` with `MessageKey::Unknown` (params tier, unit_id); no new catalogue key, so the webview should show its own text for that code.
- Tests: `tm_contract` +5 (diff rebuilds both sides, accept empty draft, accept over draft needs_confirmation then force, pair gone, global row + my pair declares other, prepare/score split); `matching_contract` +4 (rebuild invariant on 6 pairs, unchanged text, absorb, whole English words). Counter-check: `diff: Vec::new()` in the command turned `a_hit_carries_the_source_diff...` red.

Webview must add: `invoke('tm_fuzzy_matches', { segmentId })` and `invoke('accept_tm_fuzzy', { segmentId, tier, unitId, force })` in `src/config/segment.ts` (this also turns `ipc_argument_contract` green), the `tm_fuzzy_threshold` field in `src/config/bootstrap.ts`.

Verification (debug): full `cargo test --no-fail-fast` split in two runs because of a 28-minute cap on this loaded machine. Only red: `ipc_argument_contract` (no `invoke('tm_fuzzy_matches'|'accept_tm_fuzzy')` in `src/**/*.ts` yet; the adapters turn it green, re-run `cargo test --test ipc_argument_contract`). No new `message_keys!` entry and no i18n catalogue change. Not re-measured: release latency (spec AC5 was recorded in Rust phase 1; the diff adds `diff_spans` for at most 3 rows).

## Webview phase

Done (src/, scripts/, tests/frontend; nothing committed):
- `config/segment.ts`: `tmFuzzyMatches(segmentId)` and `acceptTmFuzzy(segmentId, tier, unitId, force)` with runtime shape guards (a malformed payload is an error, never an empty list). The `failureOf(err, command)` helper takes the command name SECOND on purpose: `ipc_argument_contract` treats `(CMD_X,` as an invoke call site and panics on a non-object second argument. `ipc_argument_contract` is green.
- `config/bootstrap.ts`: `tm_fuzzy_threshold` (field, `KEY_TM_FUZZY_THRESHOLD`, `bootstrapTmFuzzyThreshold`, default 65, integers 50-99 else 65).
- `tmFuzzyStripState.ts` (leaf, no import of editorPanelState): scan on caret change, sequence counter drops stale responses and responses naming another segment, per-segment Esc set, aim, pending overwrite, scan error kept visible. `editorPanelState.ts::acceptTmFuzzyToEditor` is the accept path (flush first, `acceptTmFuzzy`, `needs_confirmation` -> pending, else `replaceEditorSegment` + close); `resetTmFuzzyStrip()` is called beside both `resetGlossaryConfirmStrip()` calls.
- `TmFuzzyStrip.vue` (App.vue, after `GlossaryConfirmStrip`), `inlineStripEligibility.ts` (one list of eligible strips for the component and `main.ts`). Visibility = `topmostStrip(...) === 'tm_fuzzy'`.
- Commands (`commands/index.ts`, wired in `main.ts`): `tm.fuzzy.focus` (`Mod+Alt+E`), `tm.fuzzy.next|prev|accept|confirm_overwrite|hide`, `tm.settings.save` (no chords); `HANDLER_TABLE` has `TmFuzzyStrip.vue::onKeydown`. Inside the strip: arrows, Enter, 1-3, Esc; a modified key is ignored; Enter/arrows act only when the strip root itself has focus.
- Settings: new section `tm` (`SettingsTmSection.vue`, `tmSettingsState.ts`, `settings.nav.tm`), range 50-99 checked in TS before the write because the write path is the generic `put_config`.
- Tokens: `diff-add-bg|ink`, `diff-del-bg|ink` in both themes, new roles `diff-text`/`diff-fill` (kept out of `roles.text`/`roles.surface` so no 28 excluded cross pairs), two contrast pairs. `check-tokens.mjs` frozen tables updated (21 colours per theme, `EMITTED_VAR_FLOOR` 147). `check-panel-refs.mjs` EXEMPT gained 6 entries (same GlobalOnly reasoning as glossary settings).
- Tests: `tmFuzzyStrip.test.ts` (18), `tmFuzzyAdapters.test.ts` (10), `settingsFrame.test.ts` section list. Counter-checks: slot ignoring `topmostStrip` turned the priority case red alone; removing the caret `watch` turned 13/17 red; removing `resetTmFuzzyStrip()` from the editor reset turned the reset-wiring case red alone.

Checks green: vue-tsc, eslint, check:deps/tokens/i18n/commands/layout/panel-refs/doc-refs/gates, `cargo test --test ipc_argument_contract`, full vitest (125 files, 1699 tests; one unrelated teardown warning from `editorNavNotice.test.ts` once, the file passes alone).

Left / risk:
- Dark diff values (`#2b3a2a/#a9c99a`, `#3e2824/#e5867a`, ratios 6.61 and 5.23) are mine; Sally signs them (Q4). DESIGN.md colour table (17 tokens) was NOT updated.
- No debounce on caret moves: each move starts a scan (about 0.7 s at 100k pairs per tier); stale ones are dropped on arrival but the Rust work still runs.
- Not run: e2e, real-app pass (focus return to the grid cell, 2-column Zh layout), full `cargo test`, `check:scope`.
