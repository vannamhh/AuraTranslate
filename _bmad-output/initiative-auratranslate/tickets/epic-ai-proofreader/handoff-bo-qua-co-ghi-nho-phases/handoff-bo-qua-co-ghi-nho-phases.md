# Handoff: Rust phase -> Webview phase (bo-qua-co-ghi-nho)

Plan: `../story-bo-qua-co-ghi-nho-plan.md`. Rust phase done, nothing committed.

## Wire (Tauri commands, all in `commands/proofread.rs::wire`, registered in `lib.rs` after `ai_proofread_cancel`)

Tauri camelCases argument names on the JS side. Every command errors `work.none_open` with no Work open.

| Command | Args (JS) | Returns |
|---|---|---|
| `ai_proofread_ignore_add` | `segmentId, expectedText, start, end, expectedPhrase, kind` (same meaning and UTF-16 offsets as `ai_proofread_accept`; `kind` = `spelling\|grammar\|mistranslation\|too_loose\|obscure`) | `null` (unit). Remembering the same signature twice is one row, no error |
| `ai_proofread_ignore_list` | none | `[{id, kind, phrase, context_before, context_after, created_at}]` newest first. Field names are snake_case (serde default, like the other wires). Spelling rows have both contexts `""` |
| `ai_proofread_ignore_remove` | `id` | `boolean` (false = no such row; not an error) |
| `ai_proofread_ignore_clear` | none | `number` rows removed. No confirmation in Rust; the two-click guard is webview only |
| `ai_proofread_scan_record` | `workId, scope, reported, filtered` | `null` |

- `scope` must be `"segment" | "selection" | "chapter"`; `reported`/`filtered` are signed i64, negatives rejected. `workId` must equal the open Work's `meta.work_id` (the webview already has it from the open-Work meta); a mismatch is rejected. All three rejections give the same error `ai_proofread.scan_record_rejected` (not retryable, no params). A key was added to `src/i18n/vi.json`: `err.ai_proofread.scan_record_rejected`.
- Errors of `ignore_add` are exactly those of `check_proofread_range`: `ai_proofread.text_changed`, `ai_proofread.range_invalid` (params `segment_id`), plus `segment.retired`, `segment.not_in_open_chapter`, `segment.not_found`. Empty-after-trim phrase is `range_invalid`. They need no new i18n.
- `reported` for `scan_record`: the webview decides what to count (plan: findings that reached the strip). `filtered`: sum of `Done.filtered`.

## Scan outcome change

`AiProofreadOutcomeWire::Done` is now `{state:"done", not_scanned, filtered}`. `Cancelled` and `NotConfigured` are unchanged (no `filtered`). `isProofreadOutcomeWire` in `config/proofread.ts` must read `filtered` (non-negative integer) and reject a `done` without it. Silenced findings are removed in Rust before the Channel `Batch` event: the webview never sees them, only the `filtered` total at `Done`. `ambiguous` per segment is unchanged (repeated context-scoped phrases have no position, are not remembered and not filtered).

## Behaviour the webview can rely on

- Signature = `(kind, phrase)` for spelling; `(kind, phrase, context_before, context_after)` for the other four. Phrase is trimmed and NFC; context is NFC, at most N = 32 Unicode chars (`PROOFREAD_CONTEXT_CHARS` in `core/ai/proofread.rs`). Match rule: stored side empty matches only an empty current side; non-empty stored side matches if it is the tail (before) or head (after) of the current side. Single function pair `signature_of` / `signature_matches` / `filter_remembered`.
- Scope is the open Work (one `project.db` per Work). Verified: a second Work still reports the same phrase.
- Verified: grammar memory survives `merge_segments` and `split_segment` (phrase with context both sides); a phrase touching the segment edge is reported again after merge (AD-53 rule 7 accepted loss).
- After `ignore_add` succeeds, the webview should drop that finding from its own state at once (the next scan filters it in Rust anyway). The webview "Bo qua" (9.4) stays memory-only.
- `scan_record` is never called by Rust; the webview calls it after `Done` only (not after Cancelled or error).

## Surprises

- `commands/segment/proofread_accept.rs` must not name `crate::core::ai::` (`ai_boundary` gate). `utf16_to_byte` was therefore moved to a new `core/segment/utf16.rs`, used by both `proofread_accept.rs` and `core/ai/proofread.rs`.
- `PreparedProofreadRun` gained a field `remembered: Vec<ProofreadSignature>`; tests that build it literally must set it (ai_proofread_contract's `run_of` does).
- `ai_boundary` `ALLOWED_AI_PATHS_IN_PROOFREAD_SEAM` grew 7 -> 10 (`ProofreadSignature`, `filter_remembered`, `signature_of`).
- `config_invariants` census: `commands/proofread.rs` 2 -> 7 plain, tree total 82 -> 87 plain / 56 async.
- Schema step 34 `PROOFREAD_MEMORY_DDL` (`proof_ignore`, `proof_scan`). Six step-count test files plus `project_contract` `NON_ENTITY_DETAIL_TABLES` (18 -> 20) updated. `schema.rs` header now says "Ba muoi ba buoc, dich la phien ban 34" (the `segment_contract` doc-header gate reads it).
- `check:commands` gate counts webview commands (`COMMAND_FLOOR 200`, `DISPATCH_FLOOR 220`); it is green now and was not touched. The webview phase adds `ai.proofread.memory.{open,close,forget,clear}` + a "not an error" command; if it needs a floor bump, that is Ice's call (stop and ask).

## Verified (how)

- `cargo test` targets green: `proofread_memory_contract` (10, new), `ai_proofread_contract`, `proofread_write_boundary`, `ai_boundary`, `ipc_contract` (new case `the_proofread_memory_wires_are_registered_and_keep_their_parameter_names`), `config_invariants`, `project_contract`, `chapter_origin_contract`, `segment_role_contract`, `tm_contract`, `review_harvest_contract`, `segment_contract`, `pinned_contract`; lib unit tests in `core::ai::proofread` (6 new). Full `cargo test --no-fail-fast`: 89 binaries ok, 1 red (below).
- Counter-checks (real removal, then restored): replacing the `filter_remembered` call in `run_proofread_call` by `(findings, 0)` turned 6/10 memory tests red; making `signature_of` drop the context turned the "other context" and "edge after merge" tests red.
- `ai_proofread_contract::full_snapshot` now dumps every table; the scan test asserts no table changed (including `proof_*`).
- `npm run check:i18n`, `check:commands`, `check:tokens`, `check:layout`, `check:panel-refs` green (before any webview change).

## Left for the webview phase

Everything in the plan marked webview, the `deferred-work.md` item (`Chu: Epic 9`), and the story's Implementation Notes. Not run: `npm run test:story 9-6` (no webview tests exist yet).

## Known red until the webview phase

`cargo test --test ipc_argument_contract` (`every_registered_commands_business_parameters_match_camel_case_across_the_wire`) fails: the five new commands are registered but no `invoke()` in `src/**/*.ts` calls them yet. It goes green once `config/proofread.ts` has one literal `invoke` per command with exactly the JS keys in the table above (`segmentId, expectedText, start, end, expectedPhrase, kind` / `id` / `workId, scope, reported, filtered` / no args). Do not add them to `NO_FRONTEND_CALLER`. Re-run this target at the end of the webview phase.

---

# Handoff: Webview phase -> next phase (tests that move / gates / review)

## Done (webview)

- `config/proofread.ts`: `Done` reads `filtered` (non-negative integer, a `done` without it is rejected); five adapters, one `invoke` each (`addProofreadIgnore`, `listProofreadIgnores`, `removeProofreadIgnore`, `clearProofreadIgnores`, `recordProofreadScan`). `ipc_argument_contract` is green. The scanner treats every `CMD_X,` occurrence as an invoke call site, so error helpers take a label string, not the constant.
- `proofreadState.ts`: `ignoreProofreadFinding` (flush, Rust `ignore_add`, then the finding leaves the strip; on error it stays and `proofreadActionError` shows it); `proofreadFiltered`; `runProofread(ids, scope | null)` records the scan after `Done` only (fire and forget, failure is a `console.warn`). `reported` = findings that arrived in Batch events (before any dismiss).
- New `proofreadMemoryState.ts` + `ProofreadMemoryOverlay.vue`; commands `ai.proofread.ignore`, `ai.proofread.memory.{open,close,forget,clear}` (no default keys); title-bar button `data-proofread-memory-open` in `App.vue`; `main.ts` chord block. Rows are selected by `pointerdown`/`focus` (dispatch carries no arguments), `forget` acts on the selected row; `clear` is armed by the first click, selecting a row disarms it.
- NEW `src/openWorkIdentity.ts`: the handoff said the webview already held the open Work id; it did not. `setOpenWorkId` is called at the two Work-replacing points (`libraryChapters.openWorkById`, `libraryImport.finishImportSubmission` when `created !== null`). Until one of them ran, `scan_record` is skipped with a warning (Rust starts with no Work open, so this is only the pre-open case).
- Tests: `tests/frontend/proofreadMemory.test.ts` (new, 23), `proofreadConfig.test.ts` extended, `filtered: 0` added to the Done mocks of six proofread specs. Counter-check done: removing the `scan_record` call turned 4 red; removing the first-click arming turned 2 red (both restored).
- `scripts/check-commands.mjs` HANDLER_TABLE: two entries for the new overlay (`onEscape`, `trapTab`), same as every overlay. `deferred-work.md`: one `Chu: Epic 9` item.

## Needs Ice (not changed)

Adding one `.vue` and three `.ts` files under `src/` trips file-count floors that are 80% rules: `VUE_FLOOR` 36 -> 40 in `scripts/check-commands.mjs` and `scripts/check-i18n.mjs`, `FILE_FLOOR` 128 -> 139 in `scripts/check-layout.mjs` (the gates print `ceil(0.85 x live)`). With those three numbers raised temporarily (then restored), `check:commands`, `check:i18n`, `check:layout` were green; `check:tokens` and `check:panel-refs` are green as is. Command count was not the issue (`COMMAND_FLOOR`/`DISPATCH_FLOOR` untouched, no complaint).

## Left

`npm run test:story 9-6` still lists only 4 specs; the new `proofreadMemory.test.ts` is selected, but the plan's "Tests that move" section should name it. Full suite once at the end (migration and `lib.rs` changed). Story Implementation Notes and status are untouched.

Also: `finishImportSubmission` now reads `created.meta.work_id`; four test fixtures that passed a malformed `CreatedWork` (no `meta`) were reshaped (`libraryImportResetsSegmentHistory`, `libraryImportResetsAiPromptInspector`, `segmentSelection`, `aiTranslateBatchResetWiring`). The 25 library/import specs pass with `--maxWorkers=2`; at default workers on this machine (load 117-170) six of them time out at 5 s, unrelated to the change.
