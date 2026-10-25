# Handoff: preset-cau-hinh-ai, phase 1 (Rust) -> phase 2 (Webview)

Done (uncommitted): `write_fields` (core/aiconfig/store.rs, re-exported from core/aiconfig), pure `commands::aiconfig::ai_config_save_fields`, wire `ai_config_save_fields` registered in lib.rs, tests in aiconfig_contract.rs + ipc_contract.rs, census bumps in tests/config_invariants.rs (aiconfig.rs plain 2->3; global totals 106->107 plain).

## invoke() shape
`invoke('ai_config_save_fields', { tier, fields })`
- `tier`: same wire value as `ai_config_save_field` (`'global'` / `'work'`, see `src/config/aiconfig.ts`).
- `fields`: array of `{ field, value }` (objects, NOT tuples). `field` uses the same names as `ai_config_save_field` (`'provider'`, `'endpoint'`, `'model'`, ...); `value` is a string.
- Returns `null` on success; errors are `IpcError`.

## Behaviour
- Empty list, duplicate field, or any invalid value => `ai_config.invalid_value` (params.field = offending field, or `"fields"` for empty list), 0 rows written. Validation of all values precedes tier routing.
- Work tier with no Work open => `ai_config.work_tier_unavailable`, 0 rows.
- Values are trimmed like single-field saves. One transaction (single `store.write` job).
- Sync wire (no `(async)`), like `ai_config_save_field`.

## Notes
- Frontend tests mocking invoke should add an `ai_config_save_fields` handler (existing mocks key on command names, e.g. tests/frontend/aiConfigConnectionTest.test.ts).
- Verified: `cargo test --test aiconfig_contract --test ipc_contract --test config_invariants` green. Full suite NOT run yet (lib.rs touched: plan requires one full run by hand before done).
- Did not touch `src/` or the plan file; ticket-file 🔵 line and deferred-work item are still open for later phases.

# Phase 2 (Webview) -> phase 3 (tests)

Done (uncommitted): `src/config/aiconfig.ts`, `src/aiConfigState.ts`, `src/SettingsOverlay.vue`, `src/i18n/vi.json`. Green: vue-tsc, eslint on the three src files, check:i18n, check:tokens. Not run: check:lint whole, full vitest.

## Exported names
- `src/config/aiconfig.ts`: `aiConfigSaveFields(tier, fields: {field, value}[]): Promise<IpcError|null>` -> `invoke('ai_config_save_fields', { tier, fields })`.
- `src/aiConfigState.ts`: `AI_PROVIDERS` now 9 (anthropic, openai, gemini, openrouter, mistral, xai, ollama, lmstudio, custom); `AI_PROVIDER_PRESETS: Record<AiProvider,{endpoint,model}>` (replaces the private placeholder table; `aiConfigEndpointPlaceholder()` reads it); types `AiConfigPresetChange`, `AiConfigPresetConfirm`.

## Behaviour of selectAiConfigProvider(provider)
- Fields = provider + endpoint if preset.endpoint != '' + model if preset.model != ''. `custom` -> provider only.
- Conflict = preset value non-empty AND `aiConfigValueAtViewTier(field).trim()` non-empty and different. Conflict -> opens confirm, NO IPC, returns resolved promise. Otherwise ONE `aiConfigSaveFields(effectiveTier(), fields)` then ONE `loadAiConfigSection()` (so exactly one `ai_config_get` after the save). Never calls `ai_config_save_field` any more.
- Failure: error goes to `saveErrors.provider` (shown under the pill row), no reload; `saving` flags for the written fields set/cleared around the call.

## Confirm-state API
- `aiConfigPresetConfirm` (readonly ref): `null` or `{ provider, changes: [{ field: 'endpoint'|'model', oldValue, newValue }] }`.
- `confirmAiConfigPreset(): Promise<void>` applies (clears state first, tracked by `inFlightSave`, so `flushAiConfigDrafts`-style awaiting works); `cancelAiConfigPreset(): void` clears, no IPC.
- State is also cleared by `resetAiConfigSection()` and `selectAiConfigViewTier()`.

## data-* hooks (SettingsOverlay.vue, inside `[data-ai-field="provider"]`)
- pills: `button[data-ai-provider="<provider>"]` (9), `aria-pressed`.
- dialog: `[role="alertdialog"][data-ai-preset-confirm]` with `data-ai-preset-provider="<provider>"`; rows `[data-ai-preset-change="endpoint|model"]`; buttons `[data-ai-preset-apply]`, `[data-ai-preset-cancel]`.
- i18n keys: `settings.ai_config.provider_{gemini,openrouter,mistral,xai}`, `preset_confirm_{title,body,endpoint,model}`, `preset_apply`, `preset_cancel`.

## Existing vitest state (not touched)
Current run of aiConfigState / settingsOverlayAiConfigTierRender / aiConfigConnectionTest: 2 failed, 73 passed, 1 unhandled error. Failures are the two plan-listed old-decision tests (`aiConfigState.test.ts:786`, `settingsOverlayAiConfigTierRender.test.ts:156` "chọn Ollama ... chỉ ghi provider"); the unhandled error is that file's `vi.mock('../../src/config/aiconfig')` factory lacking `aiConfigSaveFields` -> add it to the mock (and `invoke` mocks need an `ai_config_save_fields` handler). No type/compile break.

# Phase 3 (Tests) -> orchestrator

Done (uncommitted): `tests/frontend/aiConfigState.test.ts` (mock `aiConfigSaveFields`; old provider-only test replaced by 12 preset cases: table, empty start, overwrite/confirm, cancel, apply, already-matching, Ollama keeps model, custom, work tier, inherited-conflict at view tier, save failure, temperature untouched, confirm cleared on tier switch/reset), `tests/frontend/settingsOverlayAiConfigTierRender.test.ts` (mock + 6 cases replacing the Ollama old-decision test: 9 pills, Gemini empty start, Ollama keeps qwen, alertdialog cancel/apply, work tier Mistral, failed write alert). Added two 🔵 2026-10-25 lines under the old story plan (decision bullet and Ollama AC), and a `Chủ: Epic 12` item at the end of `deferred-work.md`. Plan file untouched (status still in-progress; Implementation Notes empty). Full suite not run.
