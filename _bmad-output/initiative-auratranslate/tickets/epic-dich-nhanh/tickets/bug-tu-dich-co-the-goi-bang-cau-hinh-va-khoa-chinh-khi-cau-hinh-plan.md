---
title: 'Tự dịch có thể gọi bằng cấu hình và khoá chính khi cấu hình nhanh vừa bị xoá'
type: 'bugfix'
ticket: '9'
created: '2026-11-12'
status: 'built'
baseline_revision: '5181bd5d20630e274c01b33909ad8f758441b911'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** An automatic quick run (`trigger: auto`) is admitted by `quick_auto_gate`, which reads Global config once; the prepare step then re-resolves config through `resolve_quick_call_config`, which does not know the run is automatic. If the quick endpoint or model is cleared in between, it falls back to the full main config (R1, meant for clicks only) and reads the main key, so an unclicked call bills the main model (violates AD-64 rule 2, R4, R11).

**Approach:** The quick prepare step takes the trigger. For `auto` it re-checks the same two conditions as the gate (`Tự dịch` on, quick endpoint and model both set) at the moment the call is built and returns an `auto_off` outcome before reading any key; `manual` keeps R1 unchanged. Both wires pass their real `trigger` through.

**Decision (2026-11-12, Ice):** the Glossary-surface case drives the production chain (`quick_auto_gate(Auto)` → clear the quick endpoint → `quick_translate` with the real `prepare_context_reference_call(.., Auto)`) and adds no test-only seam to the wire; the blocks-surface case goes through `wire::quick_translate_blocks` using the `cached` channel event as the window. Known gap, recorded in Implementation Notes: a Glossary wire hard-coding `Manual` would not go red.

**Decision (2026-11-12, Ice):** keep the full plan (~2,300–2,600 tokens, single goal), including the prepare-time re-check of `Tự dịch`.

## Boundaries & Constraints

**Always:** the early `quick_auto_gate` stays where it is (it keeps an auto run with no dedicated config from taking the quick slot or reporting cached blocks); an `auto` run never reads the main key nor reaches any provider unless the dedicated config is complete at prepare time; `manual` behaviour (full main fallback when both quick fields are empty, `not_configured` when half-set) is byte-identical.

**Never:** add a variant to the shared `PrepareOutcome` (the main/proofread paths would gain an arm that cannot happen); snapshot the gate's config and reuse it at prepare (AC 1 requires `auto_off`, not a call to the quick provider); touch `resolve_call_config`, the real batch, `AiTranslateGeneration`, the webview, migrations or dependencies; edit the ticket file.

## I/O & Edge-Case Matrix

| Trigger | Config at gate → at prepare | Outcome | Calls (main A / quick B) |
|---|---|---|---|
| auto | dedicated → dedicated | done via B | 0 / 1 |
| auto | dedicated → quick endpoint cleared (model kept) | `auto_off` | 0 / 0 |
| auto | dedicated → quick endpoint and model both cleared | `auto_off`, main key not read | 0 / 0 |
| auto | dedicated, `Tự dịch` on → `Tự dịch` off | `auto_off` | 0 / 0 |
| manual | both quick fields empty | done via A (R1) | 1 / 0 |
| manual | one quick field set | `not_configured` | 0 / 0 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/aitranslate.rs:679` `quick_auto_gate` -- extract its flag + dedicated check into a helper reused at prepare; keep the activity/`Paused` part in the gate.
- `src-tauri/src/commands/aitranslate.rs:984` `resolve_quick_call_config`, `:1037` `read_quick_api_key` -- unchanged; the auto check runs before them.
- `src-tauri/src/commands/aitranslate.rs:1070` `prepare_quick_call`, `:1093` `prepare_context_reference_call` -- gain a `trigger` parameter and return a new quick-only `QuickPrepareOutcome { AutoOff, NotConfigured, Ready(PreparedTranslateCall) }`.
- `src-tauri/src/commands/aitranslate.rs:1300` `quick_translate`, `:1441` `quick_translate_blocks` -- `prepare` closure returns `QuickPrepareOutcome`; map `AutoOff` to `QuickTranslateOutcomeWire::AutoOff` / `QuickBatchOutcomeWire::AutoOff`.
- `src-tauri/src/commands/aitranslate.rs:2915`, `:2980` wires -- pass `trigger` into the prepare call (`trigger` is `Copy`).
- `:285` `PrepareOutcome` -- shared with main and proofread paths; do not change.
- `src-tauri/tests/quick_translation_contract.rs:723` `automatic_command_wires_reject_off_incomplete_and_real_activity_before_provider_calls` -- loopback fixture counting accepted calls, mock app, `write_quick_fields`, `write_quick_auto`; reuse it (factor the server into a helper serving two listeners A and B so calls are counted per endpoint).
- `src-tauri/tests/quick_batch_contract.rs:132` `ready()`, `quick_translation_contract.rs:129` `ready()`, `glossary_context_reference_contract.rs`, `aiconfig_contract.rs:1776-1890` -- mechanical update to `QuickPrepareOutcome` and the new `trigger` argument (`Manual` where they test R1).
- Seam for "after the gate, before prepare" on the blocks surface: `quick_translate_blocks` emits `Cached` events on the channel before calling `prepare` (`:1471` then `:1494`); a channel callback that clears the quick endpoint on the first `cached` event runs deterministically in that window through the real wire.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/commands/aitranslate.rs` -- add `QuickPrepareOutcome`, the shared auto-readiness helper, the `trigger` parameter on both quick prepare fns, the `AutoOff` mapping in both pure fns, and pass `trigger` in both wires -- closes the window.
- [ ] `src-tauri/tests/quick_translation_contract.rs` -- new cases per the I/O matrix with fixtures A and B; for the blocks surface through `wire::quick_translate_blocks` with one remembered text and one new text, clearing the quick endpoint (one case) and turning `Tự dịch` off (another) from the `cached` callback; for the Glossary surface through the production chain per the Decision above -- AC 1, 3.
- [ ] remaining test files in the Code Map -- mechanical signature update; add one manual case asserting both-empty goes to A and half-set is `not_configured` if not already covered by `aiconfig_contract.rs:1776-1890` -- AC 2.

**Acceptance Criteria:**
- Given an `auto` run whose dedicated config is cleared after the gate, when it reaches prepare on either surface, then it returns `auto_off`, A and B each accept 0 connections, and no key is read.
- Given the new prepare-time check is removed for real (the `trigger` branch deleted from `prepare_quick_call`), when the new cases run, then each fails because A accepted one connection.
- Given `manual` with empty or half-set quick config, when it runs, then results match R1 exactly as before.

## Implementation Notes

- The gate and prepare share the private `quick_auto_ready` (`Tự dịch` on and quick endpoint + model set); `Paused` stays in the gate only.
- Known gap (Decision 2026-11-12): the Glossary case drives the production chain, so a Glossary wire hard-coding `Manual` would not go red; the blocks wire is guarded through the real command.
- "Main key not read" is not asserted directly: it follows from `AutoOff` returning before `resolve_quick_call_config`/`read_quick_api_key` and from main endpoint A accepting 0 connections.
- Manual both-empty → main is checked at prepare (call routed to the main endpoint), not by a connection to A; `glossary_context_reference_contract.rs` `quick_routing_uses_dedicated_or_whole_main_global_despite_work_overrides` already covers the routing.
- Counter-check: replacing the auto branch in `prepare_quick_call` with `let _ = trigger;` made both new auto cases red at "main endpoint accepted 0 connections"; restored.
  🔵 2026-11-12: wrong as stated: with `Nothing` first, the matrix stopped at the half-set row (`not_configured` ≠ `auto_off`, no call), so the main-fallback row never ran. The rows now open with `ClearQuickEndpointAndModel`; removing all three auto checks then fails both cases at "main endpoint accepted 1".
- After review, `prepare_quick_call` on `Auto` builds the call only from the one resolved config when it is `dedicated`; the earlier `quick_auto_ready` read remains for the switch. That last check guards only a write landing between two reads, and no test reaches it deterministically.

## Plan Change Log

## Review Triage Log

Pass 1 (quick): high 0 · medium 1 · low 2 · false 2 · maybe-false 0.
- medium · patch — `prepare_quick_call` auto path reads config twice (`quick_auto_ready` then `resolve_quick_call_config`); a write between them still falls back to main config and key. Fix: on `Auto`, return `AutoOff` unless the one resolved config is `dedicated`.
- low · patch — doc comment on `QuickPrepareOutcome` restates the variants and breaks "default: no comment"; delete it.
- low · rejected — "main key not read" (AC 1) is inferred, not asserted; after the patch the auto path only reaches `read_quick_api_key` with `dedicated`, which reads the Quick account only. Asserting it needs keychain fixtures in this target; noted in the report to Ice.
- false — manual both-empty key source untested: `aiconfig_contract.rs` `quick_configuration_is_global_and_keys_are_isolated_with_unknown_denials` asserts `fallback.api_key == "main-secret"`.
- false — Glossary wire hard-coding `Manual` stays green: accepted gap by Ice's Decision 2026-11-12, in the frozen block. Keychain lock in new tests: `ollama` reads no key, no bad outcome today.

## Design Notes

Why not a single read: the gate must stay early so an auto run with no dedicated config does not take the quick slot (which would cancel a running run on the same surface) or emit cached events; so two reads exist, and the second one must enforce the same rule. Re-checking `Tự dịch` at prepare as well follows R4 ("Rust đọc lại hai điều kiện ngay lúc gọi") at no extra cost, since the helper is shared.

## Verification

**Commands:**
- `npm run build` (once, if `dist/` is missing) then `cd src-tauri && cargo test --test quick_translation_contract --test quick_batch_contract --test glossary_context_reference_contract --test aiconfig_contract` -- expected: all pass.
- Counter-check: delete the auto branch in `prepare_quick_call`, re-run `cargo test --test quick_translation_contract` -- expected: the new cases red because A accepted 1 connection; restore.
- `cd src-tauri && cargo clippy --all-targets -- -D warnings` -- expected: clean.
