# Handoff: ly-do-ai-chua-san-sang

## Phase 1 (Rust) done

Wire change on `ai_config_get` (`AiConfigGetWire`, serde default field names, so snake_case on the wire; Tauri does not rename response fields):

```json
{ "work_tier_available": true, "fields": [], "key_configured": false, "ready": false,
  "not_ready": ["endpoint_missing", "model_missing", "key_missing"] }
```

- `not_ready` enum strings: `"endpoint_missing"`, `"model_missing"`, `"key_missing"`, always in that order.
- `key_missing` only appears when endpoint AND model are set, provider needs a key (anything except ollama/lmstudio, including empty/unknown), and keychain answered `false`.
- `ready == false` iff `not_ready` non-empty. `ready == null` (keychain unanswered) and `ready == true` give `[]`. Keep `key_unchecked` handling from `key_configured === null`.
- The wire does NOT carry the provider name or the work-override field list; the frontend derives those from `fields[]` (`field`, `value`, `tier === 'work'`) for the sentence "Nhà cung cấp X cần khoá API".
- Rust: `call_ready` replaced by `call_readiness(provider, endpoint, model, key) -> (Option<bool>, Vec<AiNotReadyReason>)` in `src-tauri/src/commands/aiconfig.rs`.
- Tests: `not_ready_*` in `src-tauri/tests/aiconfig_contract.rs`; `cargo test --test aiconfig_contract` 31 green, clippy clean.

## Phase 2 (Webview) done

Checks green: `check:commands`, `check:lint`, `check:i18n`, `check:tokens`, `vue-tsc`. `tests/frontend/` untouched.

- `src/config/aiconfig.ts`: exports `AiNotReadyReason`; `aiConfigGet()` result gains `notReady: AiNotReadyReason[]` (`[]` on every failure path).
- `src/aiConfigState.ts` (set in `applyReadiness`, cleared in `resetAiConfigSection`): readonly refs `aiConfigNotReady`, `aiConfigWorkOverrides` (field names with `tier === 'work'`), `aiConfigReadinessWorkIsOpen` (from `work_tier_available`).
- Command `settings.ai_config.open_work` (no chord, label `command.settings.ai_config.open_work`), port `openSettingsToAiConfigWork?: () => void` in `CommandDeps`; `main.ts` wires `openSettingsToAiConfigWork: () => { void openSettingsToAiConfigWork() }`. `settingsState.ts::openSettingsToAiConfigWork` is async: opens overlay, resets tier, awaits `loadAiConfigSection()`, then `selectAiConfigViewTier('work')` if the AI section is still showing.
- Panel (`data-ai-unconfigured` block): one `<p data-ai-unconfigured-reason data-ai-unconfigured-reason-kind="endpoint_missing|model_missing|key_missing">` per reason; `<p data-ai-unconfigured-overrides>` only when a Work is open AND overrides exist; button `data-ai-unconfigured-open-settings` now dispatches `settings.ai_config.open` (was `settings.open`); button `data-ai-unconfigured-open-work-settings` (same condition as overrides line) dispatches `settings.ai_config.open_work`. The old title and `unconfigured_other` paragraph stay; `unconfigured_local` key removed.
- i18n keys: `panel.ai_translation.reason_endpoint_missing | reason_model_missing | reason_key_missing {provider} | reason_key_missing_no_provider | work_overrides {fields} | unconfigured_open_work_settings`. Provider label = `settings.ai_config.provider_<p>` when in `AI_PROVIDERS` (case-insensitive), else raw value; empty provider uses the `_no_provider` key.
- Broken existing test for phase 3: `tests/frontend/aiBatchPanelRows.test.ts` "ready = false ⇒ lời mời ..." (~line 417-435). Its `aiConfigGet` mock has no `notReady`, so the panel render throws (`notReady.value.map` on undefined); it also asserts the old two-paragraph text and `settings.open` dispatch. Mock needs `notReady`, expectations need the new sentences and `settings.ai_config.open`.

## Phase 3 (Tests) done

- `tests/frontend/aiBatchPanelRows.test.ts`: mock gains `notReady`, new `openSettingsToAiConfig` mock, sentences updated (19 green).
- New `tests/frontend/aiNotReadyReasons.test.ts` (11 green): six Matrix rows, AC1 through real `clearAiConfigOverride` + reload, AC2, `openSettingsToAiConfigWork` awaits the load, `main.ts` wiring (comment lines skipped).
- Counter-checks (seam removed, test run, file restored from copy): main.ts port removed ⇒ wiring red; `await` on load dropped ⇒ open_work red; `applyReadiness` dropped from `loadAiConfigSection` ⇒ AC1 red; work-open guard dropped ⇒ AC2 red; `notReady` not carried ⇒ 5 Matrix rows red.
- Not guardable: the final `selectAiConfigViewTier('work')` is redundant (reset to null tier already defaults to Work once `workIsOpen`), so removing it stays green.
- Checks green: `check:lint`, `vue-tsc`, `check:i18n`, `check:tokens`.

## Sau lượt review 1 (2026-10-26)

Plan đã chỉnh (xem Plan Change Log): `settings.ai_config.open_work` đã bị gỡ, panel còn một nút `settings.ai_config.open`. Các mục nói về `open_work` ở trên không còn đúng.
