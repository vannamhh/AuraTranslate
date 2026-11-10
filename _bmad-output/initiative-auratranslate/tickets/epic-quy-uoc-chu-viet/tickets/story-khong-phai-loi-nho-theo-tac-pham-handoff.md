# Handoff: Rust phase -> Webview phase / Tests phase (16-5)

Working notes, not spec. Rust phase done; `cargo check --tests` green.

## Commands registered (lib.rs, `commands::convention::wire::*`, all synchronous, not `async`)

| command | args (JS camelCase = Rust snake_case) | returns |
|---|---|---|
| `convention_ignore_add` | `segmentId: i64, expectedText: string, start: u32, end: u32, expectedPhrase: string, ruleKey: string` | `void` |
| `convention_ignore_list` | none | `ConventionIgnoreWire[]` newest first (`created_at DESC, id DESC`) |
| `convention_ignore_remove` | `id: i64` | `boolean` (`false` = row already gone, no error) |
| `convention_ignore_clear` | none | `number` (rows deleted, u32) |

- `start`/`end` are UTF-16 of `scanned_text` (= span.start/end). `expectedPhrase` = `scannedText.slice(start, end)` exactly (checked byte for byte). `ruleKey` = `span.rule_key`; Rust refuses `convention.rule_unavailable` unless `rule_key(expectedPhrase) === ruleKey`.
- No Work open: all four fail `err.convention.work_tier_unavailable` (code `convention.work_tier_unavailable`), NOT `chapter.no_work_open`. Webview must render the "needs a Work" card state from that code, or from its own open-Work check.
- Add errors: `convention.text_changed`, `convention.range_invalid` (mapped from the AD-53 check via `as_convention_error`, now `pub(crate)`), plus the usual segment_retired / not_in_open_chapter codes from `check_proofread_range`. Add does not write text, does not invalidate the matcher cache, does not touch `proof_decision`.
- Adding the same place twice is one row, no error (`INSERT OR IGNORE`).

## Wire row type (Rust `ConventionIgnoreWire`, serde snake_case, no rename)

`{ id: number, rule_key: string, avoid_phrase: string, context_before: string, context_after: string, created_at: string }`

- `avoid_phrase` is the MATCHED text (trim + NFC, original case), not the rule's avoid phrase; card draws `...{context_before}[{avoid_phrase}]{context_after}...`.
- Context is up to 32 NFC chars each side, case preserved; empty string = segment edge.
- No `blocked_count`. No `kind`.

## Scan wires: unchanged shape

`ConventionScanWire.ignored` and `ConventionSuggestionScanWire.ignored` are now real counts (were constant 0). Same field names; `glossary_suppressed` still counts Glossary-pinned hits first (a hit both pinned and remembered counts as Glossary only).

## Core signatures (`core::convention`, re-exported from mod.rs)

- `pub const CONVENTION_CONTEXT_CHARS: usize = 32`
- `pub struct ConventionSignature { rule_key, phrase, before, after }` (all `String`)
- `pub fn signature_of(text: &str, start_utf16: u32, end_utf16: u32, rule_key: &str) -> Option<ConventionSignature>` (None when range has no text after trim)
- `pub fn signature_matches(stored: &ConventionSignature, current: &ConventionSignature) -> bool` (rule_key equal; empty stored side only matches empty current side; else ends_with / starts_with)
- `pub fn filter_ignored<'k, T>(text, matches: Vec<T>, ignored: &[ConventionSignature], locate: impl Fn(&T) -> (u32, u32, &'k str)) -> (Vec<T>, u32)`; returns (kept, dropped). Skips all normalisation when `ignored` or `matches` is empty. One NFC pass per sentence (internal `ContextIndex`).
- Deviation from plan's Design Notes: context is NOT taken from `normalize()` (that lowercases); `ignore.rs` has its own cluster walk producing NFC-cased chars + UTF-16 offsets, same one-pass cost.
- Store (`core::convention::store`): `add_ignore(tx: &Transaction, &ConventionSignature) -> SqlResult<bool>` (takes the caller's tx, so the command writes inside the `check_proofread_range` transaction), `list_ignores(&Store) -> Vec<ConventionSignature>`, `list_ignore_rows(&Store) -> Vec<IgnoreRow>`, `remove_ignore(&Store, id) -> bool`, `clear_ignores(&Store) -> u32`.

## Command-layer signatures for the tests phase (`commands::convention`)

- `convention_ignore_add(open: Option<&OpenWork>, segment_id, expected_text: &str, start, end, expected_phrase: &str, key: &str) -> Result<(), IpcError>`
- `convention_ignore_list(open) -> Result<Vec<ConventionIgnoreWire>, IpcError>`; `convention_ignore_remove(open, id) -> Result<bool, _>`; `convention_ignore_clear(open) -> Result<u32, _>`
- Filter site: `resolve_spans(matcher, lang, glossary, source_text, text, hits, ignored: &[ConventionSignature]) -> (spans, glossary_count, ignored_count)`; the single filter point of all three scans. The signature set is read once per scan inside the existing `lock` block (`read_scan_input` for Chapter + by-sentence; the `(chapter_id, source_lang, matcher, remembered)` tuple for suggestions), not cached in `ConventionMatcherState`.

## Schema / ladder

- Project step 45: `CONVENTION_IGNORE_DDL` (`convention_ignore`: id AUTOINCREMENT, rule_key, avoid_phrase, context_before, context_after, created_at; `UNIQUE(rule_key, context_before, context_after)`, `CHECK(avoid_phrase <> '')`). `GLOBAL_MIGRATIONS` untouched.
- Bumped to 45 / table added to rollback fixtures: `segment_contract`, `pinned_contract`, `project_contract` (`NON_ENTITY_DETAIL_TABLES` 23 -> 24 with `convention_ignore`), `lifecycle_contract`, `proofread_decision_contract`, `prompt_set_contract`, `prompt_set_kind_migration_contract` (only the :162 project fixture; the :104 one is the GLOBAL store `IN (16, 17)` and was left alone), `tm_contract` (4 spots), plus `glossary_unconfirm_contract` and `review_harvest_contract` which assert the project version.
- `config_invariants.rs`: `convention.rs` census row `5,4,4` -> `9,4,4`; whole-tree plain total 120 -> 124; no `blocking_wire_cases` row added.
- `ipc_contract.rs`: new test `the_convention_ignore_wires_are_registered_and_keep_their_parameter_names` (names + param lists). `ipc_argument_contract` untouched.

## Not done (owned by later phases)

`convention_boundary.rs` `CONVENTION_FLOOR` 7 -> 8 (`ignore.rs` is the 8th file; boundary test currently still green only if floor is a lower bound, check it), the contract cases in `convention_contract.rs`, `story_16_5_measure_convention_ignore.rs`, vitest, the red counter-checks, `deferred-work.md`.

## Known red until later phases

- `convention_boundary::the_scanned_trees_are_large_enough_to_be_real`: floor 7 is now below 85% of 9 live files; needs `CONVENTION_FLOOR = 8` (tests phase).
- `ipc_argument_contract::every_registered_commands_business_parameters_match_camel_case_across_the_wire`: red until `src/config/convention.ts` has an `invoke` for each of the four `convention_ignore_*` names (webview phase; do not add to `NO_FRONTEND_CALLER`).
- Fixture traps found: `segment_contract` `STEP_THIRTYFOUR` future-version fixture is now `[Migration; 45]` with `PROJECT_MIGRATIONS[43]` + fake step 46; `review_harvest_contract` uses `PROJECT_MIGRATIONS.len() - 14` to land on step 31. Any new fixture that slices the project ladder by `len() - N` or hard-codes the target must move by one.

# Handoff: Webview phase -> Tests phase (16-5)

Webview done. Green: `vue-tsc`, eslint on touched files, `check:i18n`, `check:commands`, `check:panel-refs`, `check:tokens`. No vitest written or edited.

## What exists (names for specs)

- `config/convention.ts`: `addConventionIgnore({segmentId, expectedText, start, end, expectedPhrase, ruleKey}) -> {ok, error}`, `listConventionIgnores() -> {rows, error}`, `removeConventionIgnore(id) -> {removed, error}`, `clearConventionIgnores() -> {cleared, error}`, type `ConventionIgnoreRowWire`.
- `conventionState.ts`: `ignoreConventionSpot()` (async): flush, `addConventionIgnore`, drops exactly one span from `results` (entry deleted when it was the last), `counts.ignored += 1`, fixes `opened.index`, then steps to the next mark in TEXT order past the removed span (helper `stepPast`, not `stepMarks(1)`: after removing the span the index already points at the next one so `stepMarks(1)` would skip one), then `say('convention.status.ignored')` (no params). `skipConventionSpot` untouched.
- `conventionManageState.ts`: `conventionManageTab` ('rules'|'ignored'), `setConventionManageTab`, `conventionManageIgnoreRows`, `conventionManageIgnoreSelectedId`, `selectConventionManageIgnore(id)`, `forgetConventionManageIgnore()`, `conventionManageIgnoreClearPending`, `clearConventionManageIgnores()` (arm then clear), `conventionManageIgnoreDone` ({kind:'forgot'|'cleared', count}). `load()` now ALSO calls `listConventionIgnores()` (only read into state when the Work tier is available). Forget/clear go through `afterWrite`, so they reload and rescan the open Chapter.
- Commands (all `keys: undefined`, handlers in `conventionHandlers`): `convention.ignore` (`ignoreConvention`), `convention.manage.tab_rules|tab_ignored|ignored_forget|ignored_clear`.
- DOM hooks: strip button `[data-convention-ignore]` (`@mousedown.prevent` via the actions wrapper, `:disabled=conventionBusy`); tabs `[data-convention-tab="rules|ignored"]`; card `[data-convention-ignore-card]`, rows `[data-convention-ignore-row]`, per-row `[data-convention-ignore-forget]`, `[data-convention-ignore-clear]`, `[data-convention-ignore-empty]`, `[data-convention-ignore-needs-work]`, `[data-convention-ignore-done]`.
- i18n keys added: `command.convention.ignore`, `command.convention.manage.tab_rules|tab_ignored|ignored_forget|ignored_clear`, `convention.strip.ignore`, `convention.status.ignored`, `convention.manage.tabs_label|tab_rules|tab_ignored|ignored_list_label|ignored_empty|ignored_needs_work|ignored_context|ignored_forget|ignored_forget_done|ignored_clear|ignored_clear_confirm|ignored_clear_hint|ignored_clear_done`.

## Known red until Tests phase

- `tests/frontend/conventionManage.test.ts`: 14 cases fail because its `vi.mock('../../src/config/convention')` factory lacks `listConventionIgnores` (and will need `removeConventionIgnore`, `clearConventionIgnores`, `addConventionIgnore` for new cases). Other three convention/strip specs pass (69 cases).
- Other specs whose config/convention mock factory is used by code that imports `conventionManageState` may need the same four names; run the full vitest once.
- `ipc_argument_contract` should now be green (each of the four invoke names exists in `config/convention.ts`); not run by me.

## Risks

- Overlay: the Luật content is wrapped in `<template v-else>` after the tab bar; indentation of that block was not re-flowed. `onKeydown` (arrow/Enter/Space/Delete) is disabled on the Ignored tab.
- Vue DOM/focus behaviour (WebKit) not checked in a real app.

# Handoff: Tests phase (16-5) -> review / close

Working notes. No production code was changed in this phase.

## Added or changed

- `src-tauri/tests/convention_contract.rs`: 16 new cases (73 total), plus `Fixture::{ignore, remembered, reopen}`. Every row of the I/O matrix has a case. `reopen` is copied from `proofread_decision_contract.rs`.
- `src-tauri/tests/story_16_5_measure_convention_ignore.rs` (`#[ignore]`, `bench-release`): 2,000 rules, 9,850 sentences, 2,000 remembered places (493 real, one per matching sentence, plus 1,507 synthetic). It also asserts `ignored == 493`, so a fast number cannot come from a filter that never ran.
- vitest: `conventionManage.test.ts` mock completed with four adapters plus 7 new card cases; `conventionState.test.ts` +5 (Bỏ qua calls no Rust; Không phải lỗi once, one span, next spot); new `conventionStripIgnore.test.ts` (3), `conventionIgnoreCard.test.ts` (5).
- `libraryImportResetsConventionManage.test.ts` also needed `listConventionIgnores` in its partial mock: with a Work tier available, `load()` fails on an un-mocked ignore list and `hasLoaded` stays false. No other spec that mocks `config/convention` imports `conventionManageState`.
- `deferred-work.md`: one item, `Chủ: Story 16.7` (real-WKWebView pass of the strip button and the card).

## Counter-checks (real removal, `cargo test --test convention_contract`, restored by copy)

| removed | red |
|---|---|
| filter call in `resolve_spans` (`(unpinned, 0)`) | 7 cases |
| empty-side rule in `signature_matches` (`true`) | exactly `a_spot_at_the_edge_of_its_sentence_is_marked_again_once_the_sentence_grows` |
| the `ignored` count (`0` returned) | same 7 cases as the filter |
| `check_proofread_range` in `convention_ignore_add` | `a_changed_text_is_refused…`, `a_bad_range_is_refused…` |

## Measurement (bench-release, this Mac, load 9-15)

Chapter scan p95: empty memory 160-185 ms; 2,000 remembered places 157-207 ms in 6 of 7 runs (one run 275 ms right after the 16-minute LTO build, one 613 ms outlier). The filter itself is within noise of the baseline; the 250 ms budget is met on the quiet runs, not on every run.

## Left

- Full vitest at load 150-250: 8 cases red in files unrelated to this story (editorTypingZone, gridBlocks, gridSentenceMenu, settingsOverlayAiConfigTierRender, statusBarWorkspace); all 150 cases green when those five files run alone.
- Full `cargo test`, `check:*` gates and CI (Windows half, UTC run) were not run in this phase.
- `test:story` warns the plan's Code Map lacks `glossary_unconfirm_contract`, `review_harvest_contract` and the four vitest specs above.

## 🔵 2026-11-11 — số đo và danh sách "chưa chạy" ở trên ĐÃ BỊ THAY

Đọc `## Implementation Notes` của plan, không đọc các con số trong tệp này. Cụ thể: phép đo p95 157–207/275/613 ms là lượt chạy ở tải 150–250 và đã được đo lại; `cargo test` đủ, cả 14 cổng và lượt vitest đủ ĐÃ chạy sau khi tệp này được viết; `CONVENTION_FLOOR` đã nâng lên 8. Phép chia bộ nhớ theo khoá luật đã được viết rồi gỡ lại — lý do ở plan.
