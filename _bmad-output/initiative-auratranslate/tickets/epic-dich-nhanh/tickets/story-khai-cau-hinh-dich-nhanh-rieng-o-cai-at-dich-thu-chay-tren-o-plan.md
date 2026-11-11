---
title: 'Cấu hình Dịch nhanh riêng ở Cài đặt'
type: 'feature'
ticket: '2'
created: '2026-11-11'
status: 'built'
baseline_revision: 'e0e5b48dc8fddf1b81810fe0b6402747a4b08cac'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `Dịch thử` uses the active Work's main AI configuration. Users need an inexpensive configuration shared across Works.

**Approach:** Implement AD-64 §2,13: Global-only quick provider, endpoint, model and a separate keychain account. Route the existing context-reference call through this configuration. Add the `Dịch nhanh` block to Settings → `AI và mô hình` per EXPERIENCE.

## Boundaries & Constraints

**Always:** Resolve through `ScopeResolver::global_only()`. Dedicated calls use `ai_quick_provider_api_key`, with temperature/max_tokens absent. Both endpoint and model blank → entire main Global configuration and key. Exactly one present → `not_configured` before either key read. Preserve secrecy and main-account guards. Rust owns validation and readiness.

**Never:** Change `resolve_call_config`, real translation batches, cache behavior or prompt assembly. Require an open Work for quick-call preparation. Add automatic translation or its toggle (15.5), cache management, dependencies or migrations.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Dedicated | Complete quick config; Work overrides main | Quick provider/model/key used | Missing required key → `not_configured` |
| Fallback | Quick endpoint and model blank | Entire main Global config used, ignoring Work | Incomplete main config → `not_configured` |
| Partial | Only quick endpoint or model present | `not_configured`; no main key read | No network |
| No Work | Global config available | Preparation and connection test work | Same readiness rules |
| Work tier | Settings scope is Work | Global quick values visible but locked; `Sửa ở tầng Toàn cục.` | No quick writes |
| Keychain denied | Probe/save/read fails | Unknown key status or translated error | Never return/log secret |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/aiconfig/{mod.rs,store.rs,keychain.rs}`: six main fields; storage already accepts arbitrary keys. Add separate quick field type/keys; preserve main `read()` wrapper.
- `src-tauri/src/commands/aitranslate.rs`: `prepare_context_reference_call` uses Work-dependent resolver; its wire holds Work lock during keychain access. Reuse connection-check transport.
- `src/aiConfigState.ts`, `src/config/aiconfig.ts`: lifecycle, form and adapter patterns.
- `src/SettingsQuickAiSection.vue`: quick form extracted from `SettingsOverlay.vue`; `scripts/check-{commands,tokens}.mjs`: measured scan-floor refresh.
- `tests/frontend/settingsOverlayAiConfigTierRender.test.ts`: mounted scope tests. `aiConfigConnectionTest.test.ts`: cancellation patterns.

## Tasks & Acceptance

Four phases, fresh agents, with a sibling handoff file carrying working notes.

**Execution:**
- [x] Rust: `src-tauri/src/core/aiconfig/{mod.rs,store.rs,keychain.rs}` — Global-only quick keys `quick_provider/quick_endpoint/quick_model`, independent secrets; validate writes.
- [x] Rust: `src-tauri/src/commands/{aiconfig.rs,aitranslate.rs}`, `src-tauri/src/lib.rs` — expose quick get/save/key/test/cancel IPC; implement `resolve_quick_call_config`, move context preparation to it, remove the quick wire's Work lock, and give connection tests independent cancellation.
- [x] Webview: new `src/config/quickAiConfig.ts`, `src/quickAiConfigState.ts` — runtime wire checks, drafts/flush/reset, secret lifecycle and separate test status; consume Rust readiness.
- [x] Webview: `src/{SettingsOverlay.vue,settingsState.ts,commands/index.ts,main.ts,i18n/vi.json}` — block below main config with divider, `Dùng chung mọi Tác phẩm`, four fields, key actions and test/cancel controls. Lock it at Work tier. Show the three exact configuration-status lines from EXPERIENCE §Cài đặt › AI và mô hình; omit the future toggle.
- [x] Webview: `src/GlossaryQueueOverlay.vue` — replace main-config readiness gate/refresh with quick readiness, so configured quick calls remain available when the main Work config is incomplete.
- [x] Tests: `src-tauri/tests/{aiconfig_contract.rs,aiconfig_keychain_boundary.rs,ai_translate_contract.rs,glossary_context_reference_contract.rs}` — matrix, account isolation, actual quick wire with/without Work. AD-64 counter-check (b): remove production routing/partial rejection, preserve signatures, prove targeted red, restore.
- [x] Tests: new `tests/frontend/quickAiConfig.test.ts` — mounted form, Global saves, locked Work tier, status lines, independent test cancellation, error/unknown key status and draft disposal. Add IPC adapter cases where appropriate.

**Acceptance Criteria:**
- Given conflicting Work settings, when `Dịch thử` runs, then dedicated quick settings win, or blank quick fields use main Global settings.
- Given partial quick settings, when invoked, then `not_configured`, zero key reads/network.
- Given saved Global quick fields, when Settings reopens at Work scope, then values persist and mutations are disabled.
- Given new guards, when production seams are removed, then targeted tests fail for the intended reason.

## Implementation Notes

- Dedicated quick parameters are absent; blank quick endpoint/model selects the complete main Global configuration. The retained preparation signature ignores its Work argument; the actual quick wire never acquires Work state.
- Keychain access remains inside the existing boundary; quick getters expose Rust readiness and nullable key status. Six IPC commands use the existing scanner's named command constants.
- Locked fields render persisted values while editable rejected drafts remain at Global scope. Successful saved changes invalidate connection verdicts and late results; cancellation stays available after switching scope.
- Full Rust and frontend passed before review patches; focused frontend and build passed afterward. Existing panel-refs/clippy failures are owned in the ledger; real-app/Windows checks remain with Epic 15/B7. Detailed evidence is in the [phase handoff](story-khai-cau-hinh-dich-nhanh-rieng-o-cai-at-dich-thu-chay-tren-o-handoff.md).

## Plan Change Log

## Review Triage Log

Quick pass: high 0 · medium 3 · low 0 · false 0 · maybe-false 0.

| # | Finding | Verdict | Route | Evidence / action |
|---|---|---|---|---|
| 1 | Dirty rejected drafts appear at locked Work scope | medium | patch | Corrected locked rendering to saved Global values; rejected editable drafts preserved. Regression passes and turns red on removal. |
| 2 | Work scope disables cancellation of a running quick test | medium | patch | Cancel stays enabled after switching scope, matching main test cancellation. Mounted dispatch regression passes and turns red on removal. |
| 3 | Saved configuration/key changes retain old connection verdict | medium | patch | Successful field/key changes invalidate verdict and late results, preserving save-before-test flow. Completed/in-flight regressions pass and turn red on removal. |

## Verification

- `npm run build` before Rust tests.
- Target Rust contracts and frontend files above; `npm run check:commands` and `npm run check:i18n` must pass without increasing `POINTER_DEBT`.
- IPC registration changes shared wiring: run the full suite once under AGENTS.md rules, not once per phase.
- Inspect CI (including Windows/UTC) and latest scheduled macOS e2e before completion; record unavailable evidence and owned real-app/Windows gaps in the ledger.
