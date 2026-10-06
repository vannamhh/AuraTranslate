---
type: handoff
title: "Story 11.7 lot B — Phase handoff notes"
status: done
created: 2026-09-30
skill: bmad-build
---

# Story 11.7 lot B — Phase handoff notes

Spec: `spec-11-7-lo-b-ai-module.md`. Task 0: `11-7-task0-2026-09-29.md` (groups C and D). Baseline `7dbcf2301e638bff521b25ef6256715999196b0b`.

Phases, in order: 1 Rust · 2 Webview · 3 Tests (counter-checks + full suite once) · 4 Ledger. Each phase appends one `## Phase N — <name>` section and ends it with a `### Remaining for the next phase` list. A later agent reads the spec, then only the latest section of each earlier phase. Nothing is committed between phases.

## Phase 1 — Rust

### What changed
- `wire::ai_translate_segment` / `ai_translate_batch` are `<R: tauri::Runtime>` over `AppHandle<R>`; `ipc_contract.rs::fn_param_list_async` accepts the generic form. `send_prepared_translate_call` now returns `IpcError`, its `JoinError` arm and `batch_panicked_error()` (now `pub`) build the new `err.ai_translate.internal_failure` key (`code` `ai_translate.internal_failure`, not retryable). New key registered in `core/i18n/mod.rs`.
- Harness: `tauri` dev-dependency gains feature `test` (same `=2.11.5` pin, `Cargo.lock` unchanged, `check:deps` green); spine Stack row `tauri` notes it. New target `src-tauri/tests/ai_translate_wire.rs` (MockRuntime app + loopback SSE server + mock keychain): single-run `Done` stamps `sent_at`/`sent_model`; batch stamps only the last non-omitted sentence (tail item omitted). Each case takes ~25 s alone: the first `reqwest` client build before connecting (measured on the server thread, unoptimised test profile, this Mac); not the harness.
- `promote_ai_translation` reads `retired_at` with the row and returns `segment_retired` before any write; case `promoting_into_a_retired_segment_is_refused_and_leaves_the_tombstone_untouched` (`ai_translate_contract.rs`). Case `a_panicked_blocking_task_is_an_internal_failure_not_a_network_error` guards the function only, not the call in `wire::ai_translate_batch`.
- `aiprompt.rs`: stale "no MockRuntime" sentence removed; `read_record_or_report_unmanaged(Option<&State>, report)` extracted (wire passes `eprintln!`), case in `ai_prompt_contract.rs`. No wire-shape change.
- Export: `core/promptset/exchange_io.rs` gets `export_file_name` (path-unsafe chars to `_`, trailing dots/spaces trimmed, empty to `prompt-set`) and `unique_export_path` (`-2`, `-3`, checks disk and the batch, case-insensitive); the single-set export dialog name uses the same derivation. New pure `commands::promptset::prompt_set_export_many(global, open, &[PromptSetExportRef], dir)` and `(async)` wire `prompt_set_export_many` (one `blocking_pick_folder`, lock taken after the dialog), registered in `lib.rs`. Cases in `prompt_set_exchange_contract.rs` (collision, same name across tiers, `a/b`, third-file write failure keeps the first two, Work-tier failure alone).
- `entries_eligible_for_injection` deleted (store.rs, mod.rs re-export, docs); `glossary_contract.rs` re-pointed at `confirmed_terms_for_injection` through an `injected_terms` helper; `glossary_boundary.rs` gate renamed `no_raw_glossary_surface_function_may_be_called_from_outside_glossary` (its surface list already held `load_tier`, so the re-cut is the rename plus messages; the positive control on the shared predicate is untouched). `ai_rag_contract.rs` comments still name the old function as history (not touched).
- Shared rule table: `tests/frontend/support/ai-config-validity.json` (`{about, rows:[{field,input,valid}]}`, 52 rows, field names are the wire names `provider|model|endpoint|temperature|max_tokens`); Rust reads it in `aiconfig_contract.rs::every_row_of_the_shared_validity_table_gets_the_same_verdict_from_the_rust_rules` (green). Row `max_tokens` `"4294967296"` and `"99999999999999999999"` are `valid:false`: TS currently accepts them.
- L10836 screening (structural, not timed) of the 70 plain wires: nine flipped to `(async)`, each with a `blocking_wire_cases()` row and census update (census now 61 plain / 41 async, tree total asserts `(61, 41)`): `ai_config_get`, `ai_config_save_key`, `ai_config_delete_key` (OS keychain); `ai_prompt_assemble` (whole-chapter read + Glossary match per segment); `open_work` (`Store::open`, migration backup, reader pool); `tier2_block_set_kept`, `tier2_block_confirm_range`, `preview_chapter_detail` (`pending_destination` locks `PendingImportSourceState`, same mutex as 11.6 Decision 13); `glossary_cancel_import` (locks `PendingImportState`, held by `glossary_confirm_import` for the whole write). Not flipped, for the ledger: `segment.rs` (15) and `chapter.rs` plain (4) = editor flush path (AD-35), `(async)` removes main-thread ordering, needs a measurement first; wires locking `OpenWorkState` = the existing guard-scope debt, a flip does not cure it; `library_list_works`/`library_list_orphans` read the prebuilt index (NFR4 owned by Story 10.9); `cleanup_*` (5), `config` (3), `dict` (3), `pinned` (3), `promptset` plain (7), `glossary` plain 7 single-row wires, `lifecycle::read_work_lifecycle`, `list_domain_log`, `set_chapter_origin_override`, `ai_prompt_read_record`, `*_cancel*` and the two `lib.rs` wires = one row, one `Mutex` or one atomic. The census notes for aiconfig/aiprompt/promptset were rewritten.

### New wire names / keys / shapes for the webview phase
- Command `prompt_set_export_many`, args `{ sets: [{ tier: 'global'|'work', id: number }] }` (camelCase args; `PromptSetTier` deserialises as the existing single-export `tier` does). Returns `Option<Vec<PromptSetExportFileWire>>`: `null` when the folder dialog is cancelled, else one entry per requested set in request order: `{ tier: 'global'|'work', id, file_name: string|null, path: string|null, error: IpcError|null }` (snake_case fields; `error` set XOR `path`/`file_name`). The whole call is `Err(IpcError)` only when `global.db` is missing. Not registered in any TS adapter yet (`src/config/promptset.ts` has only `prompt_set_export`).
- Message key `err.ai_translate.internal_failure` is in `MessageKey::ALL` but NOT in `src/i18n/vi.json`: `ipc_contract.rs::every_message_key_exists_in_vi_json` and `every_message_key_declares_the_params_its_string_needs` are red until the webview phase adds the sentence (no params).
- `err.ai_translate.provider_unreachable` is no longer produced for a panicked/aborted blocking task on either path; `aiTranslateBatchState.ts:251` comment lists the family keys and may want the new one.
- The shared table path for the TS test: `tests/frontend/support/ai-config-validity.json`; the TS `max_tokens` bound must become `n <= 4294967295` (`isAiConfigValueValid`, `src/aiConfigState.ts`).
- Wire signatures of the flipped commands are unchanged; `e2e/` needs no edit for them.

### Tests added (all green except the two vi.json cases above)
`ai_translate_wire` (2), `ai_translate_contract` (+2), `ai_prompt_contract` (+1), `prompt_set_exchange_contract` (+5), `aiconfig_contract` (+1), `glossary_contract` (re-pointed, 83 green), `glossary_boundary` (13 green), `config_invariants` (31 green), `ipc_contract` 35/37. Only these targets were run (plus `check:deps`, `check:doc-refs`, `check:i18n`, `check:commands`); no full suite.

### Remaining for the next phase
- Webview (Phase 2): add `err.ai_translate.internal_failure` to `vi.json` (turns the two `ipc_contract` cases green); TS adapter + state + UI for `prompt_set_export_many` with the single `glossaryExchangeGate` latch around the one folder dialog and a per-file result list; read the shared table in `aiConfigState.test.ts` and fix the TS `max_tokens` bound; tier selector; handler extraction; `stillLatestAssemble` vitest.
- Tests phase (3): real-removal counter-checks still to run and record — delete `mark_prompt_as_sent` on the single-run `Done` branch, delete the batch `if let Some(..) = last_sent` block (`ai_translate_wire`, each case ~25 s), delete the `retired` branch in `promote_ai_translation`, restore `RequestFailed` in `batch_panicked_error`, remove `unique_export_path` collision handling, change Rust `u32` to `u64` in `validate_max_tokens` (table row goes red on the Rust side); a raw `load_tier` call from outside `core/glossary` for the renamed gate; delete one new `blocking_wire_cases()` row. Disposition 6 (`aiconfig_contract` `KEYCHAIN_KEY_TEST_LOCK` on every case reaching `ai_config_get`, rewrite the comment at :47-59) is untouched. Full suite once (`Cargo.toml`, `lib.rs` changed).
- Ledger phase (4): name the nine flipped wires and the not-flipped groups above under L10836; the ~25 s first-client-build cost in `ai_translate_wire` may deserve its own note (unmeasured in a release build).

## Phase 2 — Webview

### What changed
- Tier selector (Disposition 1): `src/aiConfigState.ts` gains `aiConfigViewTier` (computed: `'work'` only when `workIsOpen` and the user did not pick Global; default Work), `selectAiConfigViewTier`, `aiConfigValueAtViewTier`; drafts are rebuilt per tier (`tier === 'work' ? value : (wire.tier === 'work' ? shadowed ?? '' : value)`); `saveAiConfigField` derives the tier from `effectiveTier()` at call time; the file-header comment that said "no segmented selector" is replaced. `SettingsOverlay.vue`: radiogroup `.ai-tier` (Global / Work, Work `disabled` when `!aiConfigWorkIsOpen`, one line of explanation when none is open), Global view status keys `status_global` / `status_global_shadowed`, "Trả về kế thừa" only in the Work view. The key form is untouched (Global-only). vi.json: `settings.ai_config.tier_*`, `status_global*`.
- `max_tokens` bound: `isAiConfigValueValid` now rejects above `4294967295`. Shared table read by `aiConfigState.test.ts` (`readFileSync` of `tests/frontend/support/ai-config-validity.json`; all 52 rows plus 4294967295/4294967296 spot checks).
- `vi.json`: `err.ai_translate.internal_failure` added (the two `ipc_contract` message-key cases are green: 37/37).
- Multi-set export: `config/promptset.ts::promptSetExportMany` (validates the array; `error` XOR `path`), `promptSetState.ts::exportPromptSets` (same `glossaryExchangeGate` latch, one folder dialog, results in `promptSetExportFiles`), `PromptLibraryOverlay.vue` checkbox block `.pl-export-many` (all rows, both tiers) with a per-file result list `.pl-export-files`; vi.json `prompt.library.export_many_*`. Single-set export unchanged.
- Handler extraction (Disposition 5): new `src/aiTranslateHandlers.ts` (`aiTranslateHandlers`, six handlers); `main.ts` spreads it and lost its 12 AI imports. `aiTranslate.test.ts` and `aiTranslateBatch.test.ts` now install the real module instead of copies.
- L11973: vitest in `aiPromptInspector.test.ts` for the stale `stillLatestAssemble === false` branch (assemble A, reset, assemble B, A errors: no error written, busy stays true).
- Outside the phase line, one edit: `scripts/check-commands.mjs` HANDLER_TABLE gets `PromptLibraryOverlay.vue::onExportPicked` (`R_SUBMIT_DIRECT`); `check:commands` Kiểm K is frozen both ways and was red without it. No new gate.

### Tests added
`tests/frontend/`: `aiConfigState.test.ts` (+7: shared table, tier view/save cases), `settingsOverlayAiConfigTierRender.test.ts` (3), `promptSetExportMany.test.ts` (5), `promptSetExportManyAdapter.test.ts` (4), `aiTranslateHandlersWiring.test.ts` (3), `aiPromptInspector.test.ts` (+1). Verified: full vitest run (121 files, 1635 green — cheap here, not a required step), `vue-tsc`, eslint, `check:commands|i18n|tokens|panel-refs|doc-refs|lint` green, `cargo test --test ipc_contract` 37/37. Counter-check done by me: `aiPromptInspectorState.ts` `if (stillLatestAssemble) assembleError.value = ...` made unconditional turns the new inspector case red (restored; git diff of that file is empty).

### Remaining for the next phase
- Real-removal counter-checks to run (production seam removed, then that one test file only):
  - `aiConfigState.ts::saveAiConfigField` back to `workIsOpen.value ? 'work' : 'global'` (ignores the selector) ⇒ `aiConfigState.test.ts` "Toàn cục được chọn khi Work mở" and `settingsOverlayAiConfigTierRender.test.ts` red.
  - Delete the `n <= MAX_TOKENS_LIMIT` clause ⇒ `aiConfigState.test.ts` shared-table case red on `max_tokens` `4294967296` rows.
  - In `valueAtTier` return `wire.value` for Global ⇒ the shadowed-value case red; drop `:disabled="!aiConfigWorkIsOpen"` on the Work radio ⇒ render case red.
  - `promptSetState.ts::exportPromptSets`: remove `setGlossaryExchangeBusy(true)` ⇒ `promptSetExportMany.test.ts` latch case red; make it call `promptSetExportMany` once per set ⇒ the single-call assertion red.
  - Remove `...aiTranslateHandlers,` from `main.ts` (or re-add an inline `runAiTranslate:`) ⇒ `aiTranslateHandlersWiring.test.ts` red. Empty a handler body in `aiTranslateHandlers.ts` (e.g. the batch-vs-single exclusion in `runAiTranslate`) ⇒ `aiTranslate.test.ts`/`aiTranslateBatch.test.ts` red.
  - Loosen `isExportFile` in `config/promptset.ts` (drop the `exclusive` clause) ⇒ `promptSetExportManyAdapter.test.ts` shape case red.
- Not covered, for the Ledger/real-app pass: the multi-export block is a plain checkbox list under the group lists, not the mockup's two-card layout; no real folder dialog was driven (Rust `prompt_set_export_many` is covered only by its Rust cases). `aiTranslateBatchState.ts:251` comment does not list `internal_failure` (comment only, untouched).
- `check:commands` table edit above lives in `scripts/`; Ledger or Tests phase should mention it in the commit message.

## Phase 3 — Tests

### What changed
- Disposition 6: `KEYCHAIN_KEY_TEST_LOCK` now held by the ten `aiconfig_contract.rs` cases that reach `ai_config_get` without it (the shared-table case does not touch the keychain); the false comment above the static is rewritten in English. 20 consecutive runs of the built binary, 0 failures.
- One production-blind guard found and fixed: the full suite was red in `ipc_argument_contract.rs` (its signature scan only knew `fn name(`, so the generic `ai_translate_segment<R: tauri::Runtime>` was "not found"). Added `signature_markers` (plain and `<R: tauri::Runtime>` forms) used by `rust_required_params` and `extract_param_list_text`. Green afterwards.
- Full suite once (after `npm run build`): `cargo test --no-fail-fast` 68 targets + doc-tests, 0 failed (1931 passed in the summed counts); full vitest 121 files / 1635 green; the 12 pre-push gates green. A first plain `cargo test` stopped at the `ipc_argument_contract` failure, hence the `--no-fail-fast` rerun.

### Counter-checks (production seam really removed, that target only, then restored; `git diff` shows no leftover)
| guard | seam removed | red reason | restored |
|---|---|---|---|
| `ai_translate_wire` single run | whole `mark_prompt_as_sent` block on the `Done` branch of `ai_translate_segment` | `record.sent_at.is_some()` assertion (`:197`) | yes (byte-equal to backup) |
| `ai_translate_wire` batch | whole `if let Some(..) = last_sent` block in `ai_translate_batch` | `sent_at.is_some()` assertion in the batch case (`:234`) | yes |
| `ai_translate_contract` promote retired | `if retired { return Promoted::Retired }` in `promote_ai_translation` | `promoting_into_a_retired_segment_is_refused_...` panics at `.err().expect(..)` (write went through) | yes |
| `ai_translate_contract` panic key | `batch_panicked_error()` back to `OpenAiClientError::RequestFailed{..}.into()` | `a_panicked_blocking_task_is_an_internal_failure_not_a_network_error`, `assert_eq` on the message key | yes |
| `prompt_set_exchange_contract` | `unique_export_path` collision/`taken` check (always returns the first candidate) | `exporting_three_sets_never_overwrites_renames_collisions_...` `assert_eq` (only that case; 21 others green) | yes |
| `aiconfig_contract` shared table | `validate_max_tokens` `parse::<u32>` to `parse::<u64>` | table row `max_tokens` `"4294967296"`, valid=false | yes (file no longer modified) |
| `glossary_boundary` renamed gate | added `crate::core::glossary::load_tier` reference in `core/ai/mod.rs` | `no_raw_glossary_surface_function_may_be_called_from_outside_glossary` violation list | yes |
| `config_invariants` census | deleted the `glossary_cancel_import` row from `blocking_wire_cases()` | `every_command_bearing_file_is_classified_...`: 7 rows vs census 8 | yes |
| vitest `aiConfigState`+`settingsOverlayAiConfigTierRender` | `saveAiConfigField` tier back to `workIsOpen ? 'work' : 'global'` | last call expected `['global','model','new-global']`; render case same | yes |
| vitest `aiConfigState` shared table | dropped `n <= MAX_TOKENS_LIMIT` | table case: `max_tokens` rows deep-equal `[]` fails | yes |
| vitest tier view | `valueAtTier` Global returns `wire.value` | `'work-model'` where `'global-model'` expected (both files) | yes |
| vitest render | dropped `:disabled="!aiConfigWorkIsOpen"` on the Work radio | "no Work open" case: expected element undefined | yes |
| `promptSetExportMany.test` latch | removed `setGlossaryExchangeBusy(true)` in `exportPromptSets` | latch case `expected false to be true` | yes |
| `promptSetExportMany.test` one call | one `promptSetExportMany` call per set | both single-call assertions: called 3 times | yes |
| `aiTranslateHandlersWiring.test` | removed `...aiTranslateHandlers,` from `main.ts` | spread case `expected false to be true` | yes |
| `aiTranslateBatch.test` | removed the batch-vs-single exclusion in `runAiTranslate` | "LÔ đang generating" case: `runAiTranslateSegment` called once | yes |
| `promptSetExportManyAdapter.test` | dropped `exclusive` in `isExportFile` | shape case `expected 'done' to be 'error'` | yes |

All 17 went red on the first try for the reason measured; no guard needed fixing. Not counter-checked: the `Err(BatchCallError::Panicked) => Err(batch_panicked_error())` call in `wire::ai_translate_batch` and the single-run `JoinError` arm (no harness case can inject a panic; the function-level case is the only guard, as Phase 1 said); the `ipc_argument_contract` marker fix (found by the full suite, green afterwards).

### Measurements
- ~25 s first-client cost: not new in this lot. `webimport_contract` (existing loopback + `reqwest::blocking::Client`, HEAD-era) one case alone: 28.7 s and 28.0 s; `ai_translate_wire` one case alone: 28.8 s and 32.5 s; both codesigned debug binaries, this Mac, machine load ~7. `ai_translate_contract` has no client case at all (its header forbids one), so there is no SSE case to compare in that target. Build profile: unoptimised test, not a release build.
- Environment (not code): an unsigned freshly built test binary sat in state `U` at exec for 20 s to minutes (taskgated busy), so bare loops of the binary timed out (rc 142). `codesign -f -s - <binary>` cured it. The 20-run loop used a codesigned copy. Sequential codesign of ~110 targets took ~35 min.

### Remaining for the next phase
- Ledger phase (4): per the Phase 1 list (nine flipped wires, not-flipped groups under L10836; L12365 closed by the lock; the ~25 s note can point at `webimport_contract` as an older instance, unmeasured in release); mention in the commit message the `scripts/check-commands.mjs` HANDLER_TABLE row (Phase 2) and the `ipc_argument_contract.rs` marker fix (Phase 3). Do not touch `deferred-work.md` outside that phase.
- Nothing committed. Debt for the ledger: the `wire::ai_translate_batch` panic arm and the single-run JoinError arm are unguarded at the call site.

## Phase 4 — Ledger

`deferred-work.md` only; no code touched. Each disposition is a `→` line appended to the item's end (a single-line bullet gets an indented `→` line under it). Nothing committed.

- L7287 (4.1 two risks): split; Windows `walk()` half is a new item, `Chủ: B7`; visual `.status` half `Chủ: Epic 11`.
- L11465, L11512: closed, pointer to `93fe113dc2fc0b71012388d7f043c2527b66a5c5` (both), L11465 also to the tier selector.
- L11530 (no close-Work command, #98): closed, tier selector plus its removal counter-checks.
- L11492 (two validity rule copies): closed, shared 52-row table plus `max_tokens` bound fix.
- L11769 (multi-set export, #99): closed, Ice's four answers plus counter-checks; the checkbox-list-vs-mockup and real-dialog gap goes to a new `Chủ: Epic 11` item.
- L11796 (`entries_eligible_for_injection`): closed by deletion, gate renamed and re-cut.
- L11973 (P12): closed, both halves.
- L10836 (eight unnamed sync wires): 🟡, `Chủ: Amelia`; names the nine flipped wires, the not-flipped groups with reasons, and what stays open (`segment.rs` 15 plus `chapter.rs` 4 plain wires need a measurement first). The original eight were never named, so no claim which they were.
- L12170, L12227 (`mark_prompt_as_sent` harness): closed; L12170 carries the ~25-32 s first-client note with `webimport_contract` as an older instance (release build unmeasured).
- L12275 (`batch_panicked_error`): closed, option A key.
- L12365 (keychain flake): closed, lock on ten cases, 20 clean runs.
- L12490 (main.ts wiring copies): closed, `aiTranslateHandlers.ts`.
- L12553 (promote into a retired row): closed, `segment_retired` before any write.

New items (end of the ledger, "Deferred from: Story 11-7 lô B"): panic arms in `wire::ai_translate_batch` and the single-run `JoinError` unguarded at the call site (`Chủ: Amelia`); real-app pass for tier selector and multi-set export (`Chủ: Epic 11`); whitespace edge characters absent from the shared table (`Chủ: Amelia`); Windows `walk()` half (`Chủ: B7`).

Verified: `npm run check:debt-owner` green (0/295 open items on a done or missing owner), `npm run check:doc-refs` green.

### Remaining for the caller
- Commit message should mention: `scripts/check-commands.mjs` HANDLER_TABLE row (Phase 2), `ipc_argument_contract.rs` signature-marker fix (Phase 3), and the unsigned-binary `codesign` environment note. Story 11-7 spec `status` and `sprint-status.yaml` are untouched (the story goes `done` only after Ice signs).
