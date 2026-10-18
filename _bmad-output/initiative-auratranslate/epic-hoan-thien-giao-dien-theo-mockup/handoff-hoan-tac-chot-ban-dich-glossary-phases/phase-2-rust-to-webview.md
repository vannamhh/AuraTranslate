# Pha 2 Rust -> Pha 3 Webview (and Pha 4 Tests)

Done (uncommitted in the tree, compiles with `cargo check --tests`):
- Schema: `GLOSSARY_ENTRY_PENDING_CONFIRMED_AT_DDL` (schema.rs), Global step 14, Project step 40. Triggers `glossary_entry_lifecycle_is_one_way` (rebuilt, BEFORE) + `glossary_entry_edit_clears_unconfirm` (AFTER).
- `Store::open` runs `schema::check_glossary_lifecycle` (Global/Project) after `migrate`; only for migration sets that contain the new step (truncated old-step sets used by tests still open). New `StoreError::GlossaryLifecycleMissing` -> code `store.glossary_lifecycle_missing`, `MessageKey::StoreGlossaryLifecycleMissing`, params store+missing. The check also demands the lifecycle trigger SQL mentions `pending_confirmed_at` (catches a rebuild that restores the old trigger body). Measured extra time in the check: 0.6-3.5 ms (debug build, `cargo test`, loaded Mac); whole open 70-170 ms.
- `confirm_translation` -> `Result<Option<String>, StoreError>`; first-confirm branch is `UPDATE .. WHERE id=? AND translation IS NULL RETURNING pending_confirmed_at` (SQLite strftime ms, e.g. `2026-10-18T01:41:01.014Z`); else edit branch (no column); 0 rows -> old missing error.
- `unconfirm_translation` / tier-routed `unconfirm_pending_translation` in core/glossary/store.rs; `GlossaryError::{TranslationChanged, NotUnconfirmable}` -> codes `glossary.translation_changed`, `glossary.not_unconfirmable` (keys GlossaryTranslationChanged / GlossaryNotUnconfirmable); missing row -> existing `glossary.entry_missing`. vi.json has the 3 new strings.
- IPC: `glossary_confirm_pending_translation` now returns `string | null` (pendingConfirmedAt). New `glossary_unconfirm_translation(tier, id, expectedTranslation, expectedPendingConfirmedAt)` -> `void`, registered in lib.rs.
- Fixed `tests/glossary_boundary.rs`: needle scan for GLOSSARY_ONLY_SURFACE is now identifier-boundary (`unconfirm_translation` contained `confirm_translation`). Glossary_boundary 13/13 green.

Verified: scratch test (deleted) on Global and Project: confirm returns mark; wrong text -> TranslationChanged; wrong mark -> NotUnconfirmable; missing id -> EntryMissing; edit of confirmed text clears the mark; unconfirm then re-confirm works.

Known red now, owned by later phases:
- `ipc_argument_contract::every_registered_commands_business_parameters_match_camel_case_across_the_wire`: needs a TS `invoke('glossary_unconfirm_translation', { tier, id, expectedTranslation, expectedPendingConfirmedAt })` (Pha 3, src/config/glossary.ts). Also `REGISTERED_COMMAND_FLOOR` may need +1 (Pha 4 check).
- Step-number tests listed in plan Code Map (Global 13->14, Project 39->40), e.g. `glossary_contract.rs:1007` (`a_row_whose_term_origin_drifted...`) and `:3441`. Only glossary_contract, glossary_boundary, ipc_argument_contract were run; others not run.
- Full `cargo test` not yet run (Cargo-wide wiring touched: lib.rs, migrations) - required before done.
- Webview guard for the confirm result: wrapper in src/config/glossary.ts currently types it `GlossaryWriteResult<true>`; must become `string | null`.
