# Quick AI configuration phase handoff

## Rust phase complete

Changed `src-tauri/src/core/aiconfig/{mod.rs,store.rs,keychain.rs}`, `src-tauri/src/commands/{aiconfig.rs,aitranslate.rs}`, and registration/state setup in `src-tauri/src/lib.rs`.

No dependencies, schema migrations, cache/prompt/batch behavior changes. Main `resolve_call_config` is untouched. `prepare_context_reference_call(global, _open, context_example)` keeps its original signature for existing callers/tests, ignores `_open`, and delegates to public `resolve_quick_call_config(global)`. Its actual quick wire no longer acquires or even reads `OpenWorkState`.

## Exact IPC contract for Webview

Six commands, all registered:

- `quick_ai_config_get`, no arguments. Returns `{ fields: [{ field: "quick_provider", value: string }, { field: "quick_endpoint", value: string }, { field: "quick_model", value: string }], key_configured: boolean | null, source: "dedicated" | "main_global" | "incomplete", ready: boolean | null, not_ready: ("endpoint_missing" | "model_missing" | "key_missing")[] }`.
- `quick_ai_config_save_fields`, arguments `{ tier: "global" | "work", fields: { field: "quick_provider" | "quick_endpoint" | "quick_model", value: string }[] }`. Writes all fields transactionally; empty/repeated field list is invalid. Empty/whitespace field values persist as empty (needed to clear dedicated settings). Nonempty endpoint uses existing main URL validation; other values trim. Work tier is rejected before validation/store/key access.
- `quick_ai_config_save_key`, arguments `{ tier: "global" | "work", value: string }`. Existing key validation/errors; dedicated quick account only.
- `quick_ai_config_delete_key`, arguments `{ tier: "global" | "work" }`. Dedicated quick account only, missing entry succeeds.
- `quick_ai_config_test_connection`, no arguments. Returns same `ConnectionCheckWire` as main test (`{ kind: "connected", model_count: number } | { kind: "cancelled" } | { kind: "not_configured" }`); same provider transport/errors.
- `quick_ai_config_test_connection_cancel`, no arguments. Cancels only quick test generation, independent of main test and translation generations.

No secret is ever returned. `key_configured` is the dedicated quick account probe, while `ready` describes the selected effective call config. For `main_global`, readiness uses main GLOBAL fields/key, ignoring Work. For `incomplete`, neither key account is probed/read; readiness is false with missing endpoint/model reason, `key_configured` is null. All other probe failures preserve null rather than false. Complete local providers remain ready without an API key, same rules as main.

Frontend integration beyond Settings: `glossaryQueueState.ts` currently derives Dịch thử readiness from main `aiConfigState`. Replace that dependency with quick readiness, including library/no Work path; dedicated quick can be usable when main config is not.

## Rust seams for Tests phase

- `QuickAiConfigField`, `QuickConfigSource`, `quick_config_source`, `validate_quick_field` in core aiconfig; `store::write_quick_fields` shares transactional writer with main.
- Public pure commands `quick_ai_config_get`, `quick_ai_config_save_fields`, `quick_ai_config_save_key`, `quick_ai_config_delete_key`.
- Public `resolve_quick_call_config(global) -> Result<Option<ResolvedQuickCallConfig>, IpcError>` returns config with public provider/endpoint/model/temperature/max_tokens/dedicated; partial returns None before key reads. Dedicated generation params are absent. Blank endpoint+model fallback copies main Global generation params too; fixed quick prompt/system behavior stays as before.
- Public `prepare_quick_connection_check(global)`; `QuickConnectionCheckGeneration(pub ConnectionCheckGeneration)` is separately managed and directly constructible for actual-wire tests.
- Keychain `KeyAccount::{Main,Quick}` and `*_account` helpers are pub(crate). Main `read()` wrapper remains unchanged externally, delegates to `read_account(Main)` (bench dummy path retained). Quick account literal is `ai_quick_provider_api_key`; main literal remains `ai_provider_api_key`.
- IMPORTANT: extend `aiconfig_keychain_boundary.rs` scanner to guard new `keychain::read_account` including direct imports/brace aliases, without broadening allowed callers. Existing scanner only recognizes `read`, so its existing nine green tests are not evidence that the new accessor is guarded.
- Tests still needed: plan matrix including actual wire with/without Work; account isolation/denials; independent test cancellation; AD-64(b) real-removal counterchecks; frontend guards. Existing contract signatures required no edits.

## Validation already run

- `npm run build`: passed, existing Vite dynamic import/chunk-size warnings only.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: completed; `git diff --check` passed.
- `ai_translate_contract`: 76/76 passed.
- `aiconfig_contract`: 33/34 passed in sandbox; sole remaining existing loopback test failed at bind with EPERM. Ran same binary/test twice unchanged (same EPERM), then outside sandbox: 1/1 passed. Environment issue, no test/production code changed to bypass it.
- `aiconfig_keychain_boundary`: existing 9/9 passed.
- `glossary_context_reference_contract`: existing 6/6 passed, including actual wire.

Full suite intentionally deferred to final Tests phase/root, once because registration changes shared wiring. Frontend command adapter is not present yet, so command catalogue checks wait until Webview phase. CI Windows/UTC/nightly evidence and owned real-app gaps remain root/final-phase work.

## Webview phase complete

New files: `src/config/quickAiConfig.ts` (six runtime-checked, non-throwing adapters), `src/quickAiConfigState.ts` (Global drafts, blur/close flush, Rust readiness/source, secret disposal, separate cancellable test), `src/SettingsQuickAiSection.vue` (four fields and actions, Work lock, exact three EXPERIENCE status lines; no future toggle). `SettingsOverlay.vue` mounts the extracted block below main configuration. `settingsState.ts` loads it on entry, flushes on close/leaving AI section, resets its state with Settings. `commands/index.ts`/`main.ts` register four separate quick-key/test commands, all with literal pointer controls. `GlossaryQueueOverlay.vue` now imports quick readiness/refresh directly; there is no main readiness dependency to replace in `glossaryQueueState.ts`.

State exports to use in tests: `loadQuickAiConfigSection`, `refreshQuickAiConfigReadiness`, `setQuickAiConfigDraft(field,value)`, `commitQuickAiConfigDrafts`, `setQuickAiConfigKeyDraft`, `saveQuickAiConfigKey`, `deleteQuickAiConfigKey`, `runQuickAiConfigTest`, `cancelQuickAiConfigTest`, `flushQuickAiConfigDrafts`, `resetQuickAiConfig`; readonly `quickAiConfig*` refs. Uses `aiConfigViewTier` only to lock UI mutations; actual readiness and source always come from Rust. All three fields are plain text controls, persisted on blur; no frontend config validation. Key drafts are emptied before save/delete and on close/leave/locking. Test waits for pending field writes and supports cancellation during that wait without starting a network call. Runtime parsing rejects malformed field sets, nullable booleans/source/reasons/outcomes without logging returned data or secrets.

Mounted selectors: `[data-quick-ai-section]`, `[data-quick-ai-field="quick_endpoint"]` (and provider/model), `[data-quick-ai-key]`, `[data-quick-ai-key-status]`, `[data-quick-ai-config-status]`, `[data-quick-ai-test="..."]`, `[data-quick-ai-locked]`. Action command ids: `settings.quick_ai.key_save`, `.key_delete`, `.test`, `.test_cancel`; install their matching dependency callbacks when mounting the full overlay. Set the main view tier to Global before editable tests (Settings defaults to Work when a Work is open).

Measured scan floors updated as explicitly demanded by guards: `check-commands.mjs` POINTER_FLOOR 273→292 for 343 live pointer commands (ceil(.85×343)); `check-tokens.mjs` FILE_FLOOR 194→209 for 245 scanned frontend files (ceil(.85×245)). No POINTER_DEBT entry or exemption changed. These script files join the shared-wiring reason for the one final full-suite run.

### Webview verification

- Final `npm run build`: passed (existing Vite dynamic import/chunk-size warnings).
- Targeted ESLint across all touched frontend files: passed.
- `npm run check:commands`, `npm run check:i18n`, `npm run check:tokens`: passed. `git diff --check`: passed.
- Existing main Settings tier/key + connection-test suites: 54/54 passed across 3 files.
- Existing `glossaryContextReference.test.ts`: 24/26 passed. Both readiness tests intentionally still mock `aiConfigGet` (main); update them to mock the new quick adapter and add the dedicated-quick-ready/main-not-ready regression. Failures are the two “chưa cấu hình AI” mounted tests expecting disabled translate/retranslate buttons; no production workaround applied.
- Fresh Tests phase still owns new quick UI/adapter tests and Rust/counterchecks. Full suite, CI/nightly evidence and real-app acceptance remain pending. No visual browser/real-app acceptance claimed.

## Tests phase

No runtime behavior changes were needed by the new guards. Added quick account isolation/denial and partial-zero-key-read cases in `aiconfig_contract.rs`, dedicated/fallback Global routing versus Work overrides and the actual no-Work quick/connection-test wire in `glossary_context_reference_contract.rs`, and extended the existing keychain boundary to recognize `read_account` (including direct/aliased brace imports) without widening caller exemptions. New mounted/state/adapter tests are in `tests/frontend/quickAiConfig.test.ts`. Migrated Glossary readiness mocks to quick IPC and added the dedicated-ready/main-incomplete mounted regression.

### Matrix audit

| Frozen row | Guard | Evidence |
| --- | --- | --- |
| Dedicated; missing quick key | `quick_configuration_is_global_and_keys_are_isolated_with_unknown_denials`; `quick_routing_uses_dedicated_or_whole_main_global_despite_work_overrides` | Target Rust green; dedicated params absent, separate quick secret, main key insufficient |
| Fallback; incomplete main | routing guard above; `an_unconfigured_provider_is_a_state_reached_before_any_network_call` | Target Rust green; full Global generation parameters and no Work override; no configuration returns NotConfigured |
| Partial | `partial_quick_configuration_rejects_before_either_key_read_or_fallback` | Both one-field combinations retain both injected account errors; preparation, readiness and connection preparation reject |
| No Work | `quick_wire_and_connection_test_work_without_any_work_state_and_cancel_independently` | Actual wires reach closed port with no OpenWorkState registered; main/batch generations untouched |
| Work tier | mounted `mounts Global values, saves on blur, then shows the persisted values locked at Work scope`; account guard above | UI displays persisted Global values and exact lock sentence; state and Rust refuse writes |
| Keychain denied | account guard above; mounted `disposes secrets on failure, flush and reset, rendering translated key errors` | Unknown readiness/probe status, translated save/read errors, no secret returned/rendered |

### Commands and evidence

- `npx vitest run tests/frontend/quickAiConfig.test.ts tests/frontend/glossaryContextReference.test.ts`: initial run 29 green/2 new-test fixture errors (Work availability not supplied; expected unknown-key sentence wrong); corrected fixtures. Next run 31/31 green, `/tmp/quick-vitest.log`. The final full run includes the later explicit status/persistence assertions and Glossary dedicated-ready regression.
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test aiconfig_contract --test aiconfig_keychain_boundary --test glossary_context_reference_contract`: first compile rejected a test assumption that `ConnectionCheckPrep` was Option; corrected to its NotConfigured variant. Next sandbox run: aiconfig 35/36, existing loopback-server bind EPERM. Same binary `src-tauri/target/debug/deps/aiconfig_contract-0c36c287ea45fa1a --exact test_connection_without_endpoint_or_model_or_key_invites_configuration_and_opens_no_socket` run twice unchanged reproduced EPERM (`/tmp/quick-env-repeat{1,2}.log`). Allowed unrestricted rerun: 36/36 +10/10 +8/8 green, `/tmp/quick-rust-unrestricted.log`.
- AD-64(b) routing countercheck: actually removed dedicated field selection from `resolve_quick_call_config` (signature unchanged), selecting main Global fields. `cargo test --locked --manifest-path src-tauri/Cargo.toml --test glossary_context_reference_contract quick_routing_uses_dedicated_or_whole_main_global_despite_work_overrides` failed on `http://localhost:11434/v1` versus `http://quick.invalid/v1`. `/tmp/quick-counter-routing.log`. Restored exact production backup.
- AD-64(b) partial countercheck: actually removed incomplete classification in `quick_config_source`, sending partial config to fallback (signature unchanged). `cargo test --locked --manifest-path src-tauri/Cargo.toml --test aiconfig_contract partial_quick_configuration_rejects_before_either_key_read_or_fallback` failed with the injected main-account `ai_config.keychain_unavailable`, proving forbidden key read. `/tmp/quick-counter-partial.log`. Restored exact production backup.
- `cargo fmt --manifest-path src-tauri/Cargo.toml` and `git diff --check`: passed after restoration.
- One sequential final full check script `/tmp/quick-final-checks.sh`: each step capped by `perl -e 'alarm shift; exec @ARGV' 900`, EXIT prints `QUICK_FINAL_CHECKS_EXIT`. Logs `/tmp/auratranslate-quick-final/<step>.log`, statuses in `results.log`. Runs 14 pre-push gates, full frontend, build, full Rust, FR77. Final results appended below when complete.
- Initial full gate failures: `check:panel-refs` reports existing `src/glossaryQueueState.ts:123 lastRunId` not reset; no reset/exemption added because monotonic run ids prevent stale cancel collisions. Baseline evidence: `git show e0e5b48dc8fddf1b81810fe0b6402747a4b08cac:src/glossaryQueueState.ts | rg -n 'lastRunId|resetGlossaryQueue'` shows declaration123/increment208/use209/reset572. `check:clippy` reports pre-existing `src-tauri/tests/convention_exchange_contract.rs:115 path: &PathBuf` ptr_arg under local 1.99. No story file reported. Root owns disposition/debt.
- No remote operations performed by Tests phase; CI/Windows/UTC/nightly and real-app evidence remain root-owned.

### First full-run results and static-guard migration

- First full frontend exit **1** despite 268 files/4245 assertions green: Vitest `EnvironmentTeardownError: [vitest-worker]: Closing rpc while "onUserConsoleLog" was pending`, attributed `tests/frontend/editorConfirmSegment.test.ts`; not accepted as a full pass. `/tmp/auratranslate-quick-final/frontend.log`.
- Build passed. Full Rust exit101 stopped at `config_invariants` (48/51); preceding `ai_translate_contract`76, aiconfig36 and keychain boundary10 passed after countercheck restoration. FR77 passed; script emitted `QUICK_FINAL_CHECKS_EXIT`.
- `config_invariants` needed the command census and dummy-key guard migrated to the actual new surfaces. Inspected production bodies: quick get/save-key/delete-key call the potentially blocking OS keychain and retain `#[tauri::command(async)]`; added exactly those three blocking-wire cases. Quick field-save writes three fixed configuration rows, like existing main field saves, so retained plain classification. Quick test is async fn spawning blocking transport, its cancel only updates its dedicated atomic generation, so both are plain attributes. Measured aiconfig 4 plain/6 async, aitranslate12 plain/0 async, whole source128 plain/83 async (baseline125/80; six added IPCs). No unclassified files or exemptions introduced. Main default read explicitly must delegate `read_account(KeyAccount::Main)` and the account accessor must invoke `entry(account)?.get_password()`; the bench stub guards remain.
- First targeted `cargo test --locked --manifest-path src-tauri/Cargo.toml --test config_invariants`: 50/51 after per-file census migration; revealed the independent whole-tree125/80 population assertion, updated to measured128/83. `/tmp/auratranslate-quick-final/config-invariants-recheck.log`. Next targeted run and full rerun results follow below.
- Strengthened the dedicated-ready Glossary regression before final frontend rerun: load a main-adapter verdict `ready:false/notReady:['endpoint_missing']` with Work available and assert main readiness `unconfigured`, then mount quick-ready Glossary and assert its translate control enabled. This explicitly guards main-incomplete, rather than merely unknown main readiness.
- Second targeted config-invariants run passed51/51 (`config-invariants-recheck2.log`). Full Rust retry then reached `ipc_argument_contract` and exit101: all six quick adapters were invisible to that existing scanner, which discovers `const CMD_*` call sites. Aligned `src/config/quickAiConfig.ts` with the existing main adapter's six named CMD constants; IPC names/arguments and runtime behavior unchanged, scanner unchanged. `/tmp/auratranslate-quick-final/rust-repeat.log`.
- The two unchanged isolated editorConfirmSegment replays passed22/22 each (`editorConfirmSegment-repeat{1,2}.log`), followed by full `npm test` exit0:268 files/4245 tests, no teardown error (`frontend-repeat.log`). No subset was used as proof of full success.
- Final test strengthening: retained partial-account errors must be specifically `keyring_core::Error::Invalid`, not any error (NoEntry could otherwise hide consumption). New main-read/account assertions operate on `boundary_scan::code_lines` to ignore commented source.
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test ipc_argument_contract --test config_invariants --test aiconfig_contract`: final guards green36+51+1 (`targeted-final.log`).
- Final complete Rust scope uses `cargo test --locked --no-fail-fast --manifest-path src-tauri/Cargo.toml`, because each earlier fail-fast full run stopped before later integration binaries. `/tmp/quick-rust-complete.sh` caps this step1200s and subsequent targeted frontend/build/lint steps180s, emits `QUICK_RUST_COMPLETE_EXIT`, and preserves `/tmp/auratranslate-quick-final/rust-complete.log` separately. No production change beyond adapter CMD naming was needed.

### Final verification

- Full `cargo test --locked --no-fail-fast --manifest-path src-tauri/Cargo.toml`: **exit0**, 3052 passed/0 failed/33 ignored across113 result summaries (111 test binaries, including109 integration binaries, plus doctest groups). Debug test profile, local `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, no added feature flags. `rust-complete.log`; script emitted `QUICK_RUST_COMPLETE_EXIT`.
- Full `npm test` rerun: **exit0**,268 files/4245 tests (`frontend-repeat.log`). The first full frontend exit1 remains recorded above.
- Final `npx vitest run tests/frontend/quickAiConfig.test.ts tests/frontend/glossaryContextReference.test.ts`: **32/32**, including explicit main-unconfigured/quick-ready mounted regression (`frontend-targeted-final.log`).
- Final `npm run build`, targeted ESLint for quick adapter/new frontend tests/Glossary tests, and `git diff --check`: **passed** (`build-final.log`, `lint-final.log`, `diff-final.log`). Existing Vite dynamic-import warnings remain.
- Routing-provider fixture strengthened after the full Rust binary compiled: dedicated provider `lmstudio`, main Global provider `ollama`; explicit resolved-provider assertion avoids symmetric fixtures. Final `cargo test --locked --manifest-path src-tauri/Cargo.toml --test glossary_context_reference_contract`: **8/8**, including restored routing and actual wires (`context-provider-final.log`); `cargo fmt` completed.
- Final `npm run check:commands` and `npm run check:i18n`: **passed**, no POINTER_DEBT change (`commands-final.log`, `i18n-final.log`).
- Full gate outcome: deps/tokens/i18n/commands/layout/doc-refs/dict/dict-manifest/lint/fmt/gates/debt-owner and FR77 green. Two pre-existing gates remain red: panel-refs (`lastRunId`) and clippy (`convention_exchange_contract.rs:115 &PathBuf`); exact baseline evidence/logs above. No unrelated repair or exemption was added. CI/nightly, real-app/Windows acceptance and owned-debt disposition remain root-owned.

## Webview review patches

- Locked field rendering now reads persisted wire values through a private component helper; rejected editable drafts remain available when returning to Global. Running connection tests retain an enabled Cancel control at Work scope.
- Successful field/key writes invalidate completed test verdicts. Writes during a live network test also cancel it and invalidate its sequence, so late old-config success is ignored. The save-before-test wait remains exempt, preserving its cancellation flow. No public API added.
- Added focused regressions in `quickAiConfig.test.ts`: rejected draft close/reopen at Work, mounted Work-scope Cancel, field/key-save/key-delete verdict invalidation and stale-result rejection, save-before-test success. Target: 14/14 passed; targeted ESLint passed. Removing the three production seams (preserving signatures) produced 8 intended failures; restored all seams afterward. Broader verification remains with root.

## Root completion evidence

- After review patches: root ran quick/Glossary frontend targets, 41/41 passed; production build, check:commands, check:i18n and git diff --check passed. Full Rust 3052 passed/33 ignored and full frontend 4245 passed are pre-review-patch results; patches touched frontend only, so no redundant full suite was run.
- check:debt-owner passed after recording pre-existing panel-refs and clippy failures; real-app and Windows acceptance remain the existing Epic 15/B7 items.
- Read latest push CI [42559159109](https://github.com/vannamhh/AuraTranslate/actions/runs/42559159109): macOS/Windows gates success, test/build steps skipped for documentation-only push. Read latest scheduled CI [42917495709](https://github.com/vannamhh/AuraTranslate/actions/runs/42917495709): macOS/Windows full checks and macOS e2e success. Both ran remote revision d2cf9d65d71004a0c084bec289757e6fef6a8e08, not this story. Current Windows/UTC/real-app validation remains unverified until a later push/CI; no push was performed.
