---
title: 'Story 11.7, lot B — the AI module: tier selector, multi-set export, a real command-shell harness, and runs that report what happened'
type: 'chore'
created: '2026-09-29'
status: 'draft'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** On HEAD `32d933c`, 15 of the 33 items ending in `Chủ: Story 11.7` belong to the AI module (Task 0: `11-7-task0-2026-09-29.md`, groups C and D; `L…` numbers below are Task 0's). Signed decisions #75, #98 and #99 are not in the code: with a Work open, Global AI config cannot be edited until restart; prompt export takes one set at a time; no test reaches a `#[tauri::command]` shell, so deleting either `mark_prompt_as_sent` call stays green. Promoting an AI translation can write into a retired segment row.

**Approach:** Lot B, the last lot; it starts after lot A (`spec-11-7-lo-a-shared-ui-foundation.md`) lands, because the tier selector goes into the rebuilt Settings frame. The story goes `done` after it. Each item gets one disposition below; every guard is counter-checked by really removing its seam.

## Boundaries & Constraints

**Always:** Ledger items keep their text; only `→` lines are appended, a stale claim gets a 🔵 fix in place. The API key form stays Global-only. The tier selector reads `workIsOpen` from the authority, never `currentMode`. A wire-shape change updates `ipc_contract.rs` and the direct `invoke()` calls in `e2e/`. The `tauri` `test` feature is added to `[dev-dependencies]` at the same `=2.11.5` pin (an empty feature, 0 new crates) and noted in the spine's Stack row before `Cargo.toml` changes; that change triggers the full suite once. A Windows-only check goes to `Chủ: B7`; a real-app check to `Chủ: Epic 11`.

**Never:** Add a gate, a migration, or any other dependency. Add a close-Work command (#98). Build session token totals or a multi-model price table (#100, #101). Write decision numbers, dates or provenance into code comments.

## Dispositions

Agent, 2026-09-29 (from Task 0 and a re-read at HEAD; Ice may override any line at approval):
1. ✅ fix, webview (#98): L11530 — a Global/Work segmented selector in the Settings AI section, as `settings.html:137-140` draws it; default Work when a Work is open, Work disabled when none is; the Global view shows `wire.shadowed`, the Work view `wire.value`; save re-checks `workIsOpen` at call time. Rust already takes `tier`.
2. ✅ fix, Rust + test harness (#75): L12170, L12227 — `wire::ai_translate_segment` and `wire::ai_translate_batch` become generic over `R: tauri::Runtime`; a new test target builds a `MockRuntime` app with the states `lib.rs:1207-1229` manages and a local SSE server, and asserts `sent_at` is stamped on the single-run `Done` branch and only for the last `ToTranslate` item of a batch. The stale "no MockRuntime" comment at `aiprompt.rs:503` goes.
3. ✅ fix, Rust: L12553 — `promote_ai_translation` refuses a retired row with `segment_retired` before any write, in `restore_segment_version`'s order; L10836 — screen all 69 plain wires (not the frozen 53) against spec AI-4's D4 criteria, name each in the ledger, flip the ones that qualify to `async` with a `blocking_wire_cases()` row each.
4. ✅ fix, both: L11973 — a vitest for the stale `stillLatestAssemble === false` branch; `ai_prompt_read_record` logs before returning `None` for an unmanaged state, tested through an extracted pure fn (no wire-shape change).
5. ✅ fix, webview: L12490 — the six AI handlers leave `main.ts::boot()` for one importable module that both test files import instead of copying.
6. ✅ test-only: L12365 — every `aiconfig_contract.rs` test that reaches `ai_config_get` holds `KEYCHAIN_KEY_TEST_LOCK`; the false comment at :47-59 is rewritten.
7. Self-closed with a pointer: L11465, L11512 (`93fe113dc2fc0b71012388d7f043c2527b66a5c5`, the write tier reads `OpenWorkState`).
8. Reassign: L7287 split — the Windows `walk()` half → `Chủ: B7`, the `.status` visual half → `Chủ: Epic 11`.

Ice, 2026-09-29 (L11769 answers given before any code, as #99 required):
9. L11769 destination: one folder picker (`pick_folder`); the one-dialog latch in `glossaryExchangeGate.ts` is reused unchanged.
10. L11769 collision: auto-rename `name-2.prompt.md`, never overwrite; the same rule covers two selected sets with the same name in different tiers.
11. L11769 file name: the raw set name with `/`, `:` and other path-unsafe characters replaced; the single-set export uses the same derivation.
12. L11769 failure mid-way: keep the files already written and return a per-file result list; the UI reports each file.
13. L11492: one shared JSON rule table read by `aiconfig_contract.rs` and `aiConfigState.test.ts`; the TS `max_tokens` bound is fixed to `u32::MAX` in the same pass.
14. L11796: delete `entries_eligible_for_injection`; its tests re-point at `confirmed_terms_for_injection`, and `glossary_boundary.rs` is re-cut so it still fails on a raw `load_tier` call from outside.
15. L12275 (option A): a new key `err.ai_translate.internal_failure`, not retryable, for both the batch panic and the single-run `JoinError` arm.

</frozen-after-approval>

## Code Map

- AI config: `src-tauri/src/commands/aiconfig.rs` (`store_for_tier` :44, `work_tier_available` :126/:184, `ai_config_save_key` :197 Global-only, `ai_config_save_field` :231); `src-tauri/src/core/aiconfig/mod.rs` (`validate_max_tokens` :161, `validate_field` :216); `src/aiConfigState.ts` (`workIsOpen` :86, `isAiConfigValueValid` :165, `saveAiConfigField` :246-262 tier at :253, `resetAiConfigSection` ~:349); `src/SettingsOverlay.vue` AI section (moved by lot A); tests `tests/frontend/aiConfigState.test.ts`, `settingsOverlayAiConfigKeyRender.test.ts`, `src-tauri/tests/aiconfig_contract.rs` (lock :60, one-shot :88-113, flaky case :762; probe `commands/aiconfig.rs:182`).
- Prompt export: `src-tauri/src/commands/promptset.rs` (`default_export_file_name` :271, `prompt_set_export` :285, wire :679); `src-tauri/src/core/promptset/exchange_io.rs:62` `write_export_file` (atomic); `src/promptSetState.ts:275`; `src/glossaryExchangeGate.ts`; `src/PromptLibraryOverlay.vue:401-406`; mockup `prompt-library.html:226-238`.
- Injector: `src-tauri/src/core/glossary/store.rs:828`, `core/glossary/mod.rs:310`; `src-tauri/tests/glossary_boundary.rs:121,390,415`; `glossary_contract.rs`.
- Inspector: `src/aiPromptInspectorState.ts` (:150-200, guard :183, reset :247); `tests/frontend/aiPromptInspector.test.ts:832`; `src-tauri/src/commands/aiprompt.rs` (`mark_prompt_as_sent` :474, comment :503, `ai_prompt_read_record` :561).
- Runs: `src-tauri/src/commands/aitranslate.rs` (`batch_panicked_error` :178, JoinError arm :664-668, `wire` :778, `ai_translate_segment` :797 Done :867-878, `ai_translate_batch` :895 `last_sent` :960-965, Panicked :1003); `commands/segment.rs` (`promote_ai_translation` :2140, `segment_retired` :2240, retired check pattern :775-797); `src-tauri/src/lib.rs` (handlers :954/:963, states :1207-1229, `Store` :1079); `src-tauri/Cargo.toml` (`tauri` :32, `[dev-dependencies]` :191); tauri source `~/.cargo/registry/src/*/tauri-2.11.5/src/test/mod.rs`.
- Tests that text-scan wire signatures: `src-tauri/tests/ipc_contract.rs:2077-2160`, `config_invariants.rs:1469-1607` (census), `:992` (`blocking_wire_cases`); helpers to reuse `ai_translate_contract.rs:83-204`, SSE server :535, promote cases :1511-1585.
- Webview runs: `src/main.ts:973-1092` (six handlers); pattern `src/editorClearSourceCuts.ts`; copies `tests/frontend/aiTranslate.test.ts:133-207`, `aiTranslateBatch.test.ts:167-260`.
- Census source: `spec-ai-4-sau-lenh-nhap-roi-luong-giao-dien.md` (D2/D4 criteria).

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/**`, spine Stack row -- Dispositions 2-4 Rust halves, L10836 screening, Decisions 9-15 Rust halves -- Rust phase.
- [ ] `src/**`, `vi.json`, `tests/frontend/*` -- Dispositions 1, 4, 5 and Decisions 9-13, 15 webview halves -- Webview phase.
- [ ] Disposition 6, a real-removal counter-check for every new guard; the full suite once (`Cargo.toml`, `lib.rs`) -- Tests phase.
- [ ] `deferred-work.md` -- one `→` disposition for each of the 15 items -- Ledger phase.

**Acceptance Criteria:**
- Given lot B is done, when the 15 item lines are read, then each ends in one `→` disposition and `npm run check:debt-owner` is green.
- Given a Work open and the selector on Global, when a field is saved, then Global changes and the Work override does not; given no Work open, then the Work segment is disabled.
- Given the harness, when the `mark_prompt_as_sent` call on the single-run `Done` branch or the batch `last_sent` block is really removed, then its case goes red.
- Given a segment retired by a merge, when its AI translation is promoted, then `SegmentRetired` returns and the tombstone row is unchanged.
- Given three prompt sets selected, two named `Tiên hiệp` in different tiers and one named `a/b`, when they are exported into a folder that already holds `Tiên hiệp.prompt.md`, then one folder dialog opens, no existing file is overwritten, the colliding names get `-2`/`-3`, `a/b` becomes a safe file name, and a failure on the third file keeps the first two and reports each file.
- Given `max_tokens` = 4294967296, when it is checked in TS and in Rust, then both reject it (the shared table row).
- Given `aiconfig_contract` run twenty times at default parallelism, then it is green every time.
- Given each new guard, when its seam is really removed, then it goes red for that reason.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo test --test <new harness target>`, `--test ai_translate_contract`, `--test aiconfig_contract`, `--test config_invariants` -- green (after `npm run build`).
- `npm run check:commands && npm run check:i18n && npm run check:debt-owner` -- green.
