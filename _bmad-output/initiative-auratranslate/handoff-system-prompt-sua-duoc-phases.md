
## Phase 1 (Rust)

Done: all `src-tauri/` tasks of the plan. No `src/` or `tests/frontend/` touched. Nothing committed.

### Deviations from the plan text
- Default text lives in `src-tauri/src/core/aiconfig/default_system_prompt.txt` (+ `DEFAULT_SYSTEM_PROMPT`, `resolve_system_prompt` in `core/aiconfig/mod.rs`), not `core/ai/`: `core/aiconfig` and `commands/aiconfig` may not name `core::ai` (AD-13, `ai_boundary`).
- `assemble_prompt` signature unchanged (24 test callers). The system prompt is resolved in `assemble_and_record_prompt_with_tm` and stored on `AssembledPromptRecord.system`.
- No new command: `ai_config_clear_override` gained an optional `tier` argument (see wire shape), so command registration and the `config_invariants` census did not move.

### Wire shape the webview consumes
- `ai_config_get` -> `AiConfigGetWire` gains `default_system_prompt: string` (app text). `fields` now has 6 entries; the new one is `field: "system_prompt"`.
  - No row at any tier: `value` = the app default text, `tier: "global"`, `shadowed: null`.
  - Row saved blank: `value: ""` (= "Tắt"). Row present: its text verbatim.
  - Status for the UI: `value === ""` -> Tắt; `value === default_system_prompt` and tier global -> Mặc định của app; else Đã sửa. (An explicit row equal to the default reads as default; same effect.)
- Save: existing `ai_config_save_field {tier, field: "system_prompt", value}` and `ai_config_save_fields`. Validator keeps text verbatim (no trim), accepts `""`. Never rejects.
- Restore default: `ai_config_clear_override {field: "system_prompt", tier: "global" | "work"}`. `tier` is optional and defaults to `"work"` (existing callers unchanged). Global clear deletes the row, so the field reads the app default again; Work clear inherits Global.
- Do NOT put `system_prompt` through the preset path (`applyAiConfigPreset` writes provider/endpoint/model/temperature/max_tokens only).
- Inspector: `AssembledPromptWire` (from `ai_prompt_assemble` / `ai_prompt_read_record`) gains `system: string | null` (null = none sent: "Không gửi system prompt"). `ledger.pieces` still describe only the user message.
- Resend uses the record's `system`; proofread never sends a system message.

### Verified (cargo, `src-tauri/`, after `dist/` present)
- Green: `ai_translate_contract` (80), `ai_prompt_contract` (31), `ai_proofread_contract` (21), `ai_rag_contract`, `aiconfig_contract` (33), `ai_translate_wire`, `proofread_memory_contract`.
- Counter-checks (real removal, restored afterwards): removing the system push in `build_request_body` -> 2 reds (`a_request_with_a_system_prompt_sends_system_then_user_messages_verbatim`, `a_translate_call_sends_exactly_the_system_and_prompt_the_inspector_record_holds`); making `prepare_resend_call` read current config instead of the record -> `a_resend_uses_the_system_prompt_recorded_with_the_first_send_not_the_current_config` red.
- Existing tests adapted: `mark_prompt_as_sent` now takes `sent_system: Option<&str>` after `sent_prompt`; field-count/`""` assertions in `aiconfig_contract` and the wire-key pin in `ai_prompt_contract` know the new field.

### Remains
- Webview phase (types `AI_CONFIG_FIELDS`/union, textarea in SettingsOverlay saved on blur, status line, restore button, inspector block, i18n keys) and frontend tests.
- Also check `tests/frontend` fixtures that mock `ai_config_get` / `ai_prompt_*` wire: they now need `default_system_prompt` / `system`.
- Full-suite run not done by this phase (touched shared wiring only via test-visible struct fields; `lib.rs` untouched).

## Phase 2 (Webview)

Done: every `src/` task of the plan. `src-tauri/` untouched, `tests/frontend` untouched, nothing committed.

### Changed
- `src/config/aiconfig.ts`: `AiConfigField` gains `'system_prompt'`; `aiConfigGet()` also returns `defaultSystemPrompt` (string, `''` if the wire lacks it); `aiConfigClearOverride(field, tier?)` sends `{field}` or `{field, tier}`.
- `src/aiConfigState.ts`: `AI_CONFIG_FIELDS` has 6 entries (system_prompt last). `isAiConfigValueValid('system_prompt', _)` is always true. `saveAiConfigField` sends system_prompt verbatim (no trim). `commitAiConfigField('system_prompt')` saves when draft !== value at view tier (blank allowed = Tắt; blank never reverts). Global view with a Work row and `shadowed === null` reads the app default. New exports: `aiConfigDefaultSystemPrompt()`, `aiConfigSystemPromptStatus()` -> `'default'|'edited'|'off'`, `aiConfigSystemPromptCanRestore()` (Global: status !== default; Work: Work row exists), `restoreAiConfigSystemPrompt()` (clears at the viewed tier), type `AiSystemPromptStatus`.
- `src/SettingsOverlay.vue`: system_prompt removed from `AI_TEXT_FIELDS`; own block `[data-ai-field="system_prompt"]` with `textarea[data-ai-system-prompt]` (rows 6, saved on blur / submit), status `[data-ai-system-prompt-status]`, restore button `[data-ai-system-prompt-restore]` inside a `.ai-field-clear-form`. Presets untouched.
- `src/config/aiprompt.ts`: `AssembledPromptWire.system: string | null`, validated at runtime (a fixture WITHOUT `system` is now rejected, by design).
- `src/aiPromptInspectorState.ts`: `aiPromptSystemText(record)` (`null` for null/`''`). `src/AiPromptInspectorOverlay.vue`: first in `.aip-doc`, `section[data-aip-system="present"]` (+ `[data-aip-system-chars]`, text in `.aip-piece-system`) or `p[data-aip-system="none"]`. System block is not in `aiPromptTierBlocks` / char ratio.
- i18n (`vi.json`): `settings.ai_config.field_system_prompt`, `.system_prompt_note`, `.system_prompt_status_default|edited|off`, `.system_prompt_restore`; `ai.prompt_inspector.block_system`, `.system_none`.
- `scripts/check-commands.mjs`: `HANDLER_ATTR_FLOOR` 146 -> 157 (gate demanded it: two new `@submit`/`@blur`-class attrs pushed 80% of live 184 above 146).

### Green
`vue-tsc` clean for `src/`; `eslint src` clean; `check:tokens`, `check:i18n`, `check:commands` pass. vitest: aiConfigState, aiTranslateResendCall, aiPromptInspector (runtime), aiPromptInspectorScreen pass as-is.

### Red / for phase 3
- `tests/frontend/settingsOverlayAiConfigTierRender.test.ts` "chọn Toàn cục: ... nút Trả về kế thừa ẩn": `findAll('[data-ai-field] .ai-field-clear-form')` now finds the system_prompt restore form because the fixture has no system_prompt wire (value `''` reads as "Tắt", so Restore shows). Fix the fixture (add a `system_prompt` field wire equal to the default, plus `default_system_prompt`) or scope the selector.
- `vue-tsc` on tests: `tests/frontend/aiPromptInspector.test.ts:144` and `aiPromptInspectorScreen.test.ts:102` build `AssembledPromptWire` without `system` (type error; the runtime validator also rejects them where they go through the adapter). Add `system`.
- New cases to write: status default/edited/off, restore at Global and Work (invoke args `{field:'system_prompt', tier}`), verbatim newlines/spaces on save, blank saves `''`, inspector present/none blocks, Work tier "off" at one Work only.
- Also in `scripts/check-commands.mjs`: `HANDLER_TABLE` entry `onRestoreAiSystemPrompt` (`R_SUBMIT_DIRECT`); `check:commands` now exits 0.

## Phase 3 (Tests)

Production code untouched (no defect found). Nothing committed. Tests only.

### Changed
- Fixtures: `settingsOverlayAiConfigTierRender.test.ts` (system_prompt row + `defaultSystemPrompt` in `ai_config_get` mock; fixed the red "nút Trả về kế thừa ẩn"), `aiPromptInspector.test.ts` + `aiPromptInspectorScreen.test.ts` + `aiPromptConfigGuards.test.ts` (`system: null` in `AssembledPromptWire`; the guards file had 14 reds phase 2 did not list, all from the runtime validator).
- New cases: `aiConfigState.test.ts` (describe "system_prompt — trạng thái…", 8), `settingsOverlayAiConfigTierRender.test.ts` (describe "ô System prompt", 8), `aiPromptInspectorScreen.test.ts` (describe "khối System prompt…", 3), `aiPromptConfigGuards.test.ts` (1, `system` field guard), new file `aiConfigSystemPromptAdapter.test.ts` (2: wire `default_system_prompt`, clear-override arg shape).
- Counter-checks (real removal, restored byte-identical): `saveAiConfigField` trimming system_prompt + `restoreAiConfigSystemPrompt` dropping the tier arg -> 5 reds (state verbatim, state restore, overlay Ice sửa, overlay verbatim, overlay Work); inspector `v-if="systemText !== null"` -> `false` -> 1 red (present block).

### Verified
- vitest (11 files: aiConfigState, settingsOverlayAiConfigTierRender, aiPromptInspectorScreen, aiTranslateResendCall, aiPromptInspector, aiPromptConfigGuards, aiConfigSystemPromptAdapter, aiNotReadyReasons, settingsFrame, settingsOverlayAiConfigKeyRender, aiConfigConnectionTest): 304 passed. ⚠️ One run under load (vue-tsc/eslint in parallel) timed out 6 cases at 5 s; same files green on re-run alone.
- `vue-tsc --noEmit` and `eslint` on the touched test files: no output (clean).
- cargo (ai_translate_contract 80, ai_rag_contract 50, ai_prompt_contract 31, ai_proofread_contract 21, aiconfig_contract 33): all ok. Full suite NOT run.

### Row map (matrix row / AC -> test -> passed)
- Mặc định -> Rust `the_record_carries_the_app_default_system_prompt_when_no_tier_has_a_row` (ai_prompt_contract); UI "I/O Matrix "Mặc định" + AC Khôi phục…" (settingsOverlay) ; state "trạng thái: văn bản của app ⇒ default…" -> passed.
- Ice sửa -> `a_global_system_prompt_is_recorded_verbatim_including_newlines_and_edge_spaces`; `a_translate_call_sends_exactly_the_system_and_prompt_the_inspector_record_holds` (ai_translate_contract); UI "I/O Matrix "Ice sửa"…"; state "lưu giữ NGUYÊN văn…" -> passed.
- Ghi đè Tác phẩm -> `a_work_system_prompt_overrides_global_and_clearing_it_falls_back_to_global`; UI "I/O Matrix "Ghi đè Tác phẩm" + "Tắt ở một Tác phẩm"…" (Work save + restore clear tier work); state "Khôi phục: Toàn cục… Tác phẩm…" -> passed.
- Tắt -> `a_request_without_a_system_prompt_or_with_an_empty_one_sends_only_the_user_message`; `the_system_prompt_validator_keeps_the_text_verbatim_and_accepts_a_blank_one`; `a_blank_saved_system_prompt_reads_back_blank…` (aiconfig_contract); UI "I/O Matrix "Tắt"…" + "xoá hết ô… lưu chuỗi rỗng"; inspector "system null ⇒ dòng Không gửi…" + "system rỗng" -> passed.
- Gửi lại -> `a_resend_uses_the_system_prompt_recorded_with_the_first_send_not_the_current_config` (red on counter-check in phase 1) -> passed. Webview resend path unchanged (aiTranslateResendCall passes).
- Tắt ở một Tác phẩm -> `a_blank_system_prompt_row_turns_it_off_only_at_the_tier_that_saved_it`; UI same Work case (Work off, Global still "Đã sửa"); state "xem Toàn cục khi Tác phẩm tắt…" -> passed.
- Proofread -> `a_proofread_request_carries_no_system_prompt_even_when_one_is_configured` (ai_proofread_contract) -> passed.
- Lỗi lưu -> UI "I/O Matrix "Lỗi lưu"…" and state "lưu trượt ⇒ lỗi hiện…, nháp giữ nguyên" (tError alert + draft kept, no reload) -> passed. ⚠️ "Không ghi một phần" has no Rust IO-failure test; single-row save, so not a seam (see Remains).
- AC1 (Xem prompt: khối System prompt = mặc định, trên thân) -> Rust default test + inspector "có system ⇒ khối present đứng TRƯỚC mọi khối tầng…" + guard `system` field test -> passed.
- AC2 (sửa rồi dịch tiếp, request + Xem prompt cùng văn bản) -> `a_translate_call_sends_exactly_the_system_and_prompt_the_inspector_record_holds` + UI verbatim save -> passed.
- AC3 (Khôi phục mặc định ở Toàn cục -> "Mặc định của app") -> UI "Ice sửa" case (clear args `('system_prompt','global')`, then box = app text, status "Mặc định của app", no restore button); adapter arg-shape test -> passed.
- Preset không chạm system_prompt -> existing exact-array `aiConfigSaveFieldsMock` assertions (five fields) in the preset cases -> passed.
- Request shape `[system,user]` vs `[user]` -> `a_request_with_a_system_prompt_sends_system_then_user_messages_verbatim` / `a_request_without_…` -> passed.

### Remains
- Not run: e2e, `check:*` gates beyond what phase 2 ran, full `cargo test`/vitest (pre-push runs them). `check:i18n`/`check:tokens` not re-run for the test files.
- No Rust test forces an IO error on the `ai_config` write to prove nothing partial is written; the save is one row, so risk is low.
- No real-app pass (Ice) done for the new textarea; owner is Ice before the plan goes done.
