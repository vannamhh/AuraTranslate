# Handoff: Rust phase -> Webview phase / Tests phase (16-6)

Working notes only; the spec is the plan beside this file. Nothing committed.

## Done (Rust phase)

- `core/delimited_text.rs` (new, `pub mod` in `core/mod.rs`): `Delimiter`, `strip_bom`, formula guard (`needs_formula_guard`, `strip_formula_guard`), `field_needs_quoting`, `quote_field`, `render_field`, `strip_zero_width`, `split_first_logical_line`, `count_line_breaks`, `logical_lines` (error `DelimitedError::UnterminatedQuotedField`), `split_fields`, `unquoted_char_present`, plus file I/O `read_text_file(path, limit) -> ReadError` and `write_text_file_atomic(path, contents) -> WriteError`.
- `core/glossary/exchange.rs`: moved items deleted, `pub use`/`pub(crate) use` of the old names; `logical_lines` stays here as a thin wrapper mapping `DelimitedError` to `ParseIssue` (bilingual.rs compiles unchanged). `exchange_io.rs`: `read_import_file`/`write_export_file` are error-mapping shells (Glossary error codes unchanged, its own unit tests untouched and green).
- `core/convention/exchange.rs` (new): `parse` (first error rejects whole file), `render_tier`, `visible_rows` (Work tier drops `enabled = 0` flag rows), `classify` (New / Identical / Conflict), `apply_import` (one `store.write`; re-classifies every plan row inside the transaction, any drift gives `ImportStale`, 0 writes; unknown decision key refused before the transaction). `MAX_CONVENTION_IMPORT_BYTES` = 1 MiB. Re-exported from `core::convention` as `parse_import`, `classify_import`, `render_tier`, `apply_import`, `visible_rows`, `ConflictDecision`, `ImportRow`, `RowPlan`, `RowPlanKind`, `ImportSummary`.
- `core/convention/store.rs`: `ConventionStoreError` gained the import/export variants and IPC mapping (codes below). IO errors reuse `err.import.too_large`, `err.import.not_utf8`, `err.io.read_failed`.
- `commands/convention.rs`: pure `convention_export_tier`, `convention_export_tier_after_dialog`, `convention_open_import_preview`, `convention_open_import_preview_after_dialog`, `convention_confirm_import`, `convention_cancel_import`, `clear_pending_convention_import_for_work`; `ConventionPendingImportState`; four `wire` commands, all `#[tauri::command(async)]` (cancel too: it locks the slot confirm holds for its whole write, same as Glossary). `OpenWorkState` locked only after the dialog closes (`with_open`).
- `lib.rs`: four commands registered, `app.manage(ConventionPendingImportState::new(None))`, pending Work plan cleared in `close_open_work`; same clear added in `commands/project/mod.rs` `replace_open_work` (swapping Work).
- `src/i18n/vi.json`: `err.convention.import_*`, `no_pending_import`, `export_write_failed`, `dialog_path_invalid`; also (Webview-facing, added early) `convention.manage.export|import|exchange_tier_label|export_done|import_summary|import_keep_mine|import_take_file|import_apply|import_cancel`, `command.convention.manage.export|import`, `command.convention.import.confirm|cancel|keep_mine|take_file`; `convention.manage.empty_rules` text updated to the EXPERIENCE wording.
- Gate tables: `config_invariants.rs` (4 `blocking_wire_cases`, `COMMAND_FILE_CENSUS` convention.rs 9 plain / 8 async, global total 124/80), `convention_boundary.rs` `CONVENTION_FLOOR` 9, `ipc_argument_contract.rs` `REGISTERED_COMMAND_FLOOR` 159 -> 171 (201 registered commands; floor drifted below 80%).

## Wire contract for the Webview phase

Tier arg is `RuleTier` lowercase JSON: `"global"` | `"work"`. The webview derives it: filter `both` => `work` when a Work is open, else `global`; Rust never sees `both`. Webview must not send paths.

| Command | Args | Returns |
|---|---|---|
| `convention_export_tier` | `{ tier }` | `string \| null` (path written; `null` = dialog cancelled, stay silent) |
| `convention_open_import_preview` | `{ tier }` | `ConventionImportPreviewWire \| null` (`null` = cancelled; no plan kept) |
| `convention_confirm_import` | `{ decisions }` : map `rule_key -> "keep_mine" \| "take_file"` | `{ added, replaced, kept }` |
| `convention_cancel_import` | none | `void` |

Preview: `{ file_name, tier, row_count, new_count, identical_count, conflicts: [{ rule_key, existing_avoid_phrase, existing_replacement_phrase: string|null, file_avoid_phrase, file_replacement_phrase: string|null }] }`. Send only `take_file` entries; a missing key means keep mine. A key that is not a conflict is refused (`err.convention.import_decision_unknown_rule`). `import_apply` label numbers: `new_count` from the preview; `kept_count` = `identical_count` + conflicts still on keep-mine.

Error codes (IpcError `code`): `convention.import_delimiter_unresolved`, `import_missing_column`, `import_duplicate_column`, `import_cell_count_mismatch`, `import_blank_avoid_phrase`, `import_duplicate_avoid`, `import_unterminated_quoted_field`, `import_file_too_large`, `import_not_utf8`, `import_read_failed`, `import_decision_unknown_rule`, `import_stale`, `no_pending_import`, `export_write_failed`, `dialog_path_invalid`, `work_tier_unavailable`.

Behaviour to design the overlay around:
- Failed confirm keeps the plan pending (like Glossary). `import_stale` is permanent for that plan (every retry re-checks against the live tier and fails again), so the overlay should show the error and leave Cancel / reopen as the way out. `no_pending_import` also happens after the Work closed or was replaced.
- Success: matcher cache already invalidated in Rust; webview reloads the list and rescans via `afterWrite`.
- A new `convention_open_import_preview` replaces a pending plan.
- Parser notes: extra columns beyond `avoid`/`replacement` are ignored; blank cell 2 = `replacement_phrase` NULL; zero-width chars and a leading formula guard `'` are stripped from cells; a one-column file reports `import_delimiter_unresolved` (no delimiter in header), an empty file reports `import_missing_column`.

## Left for the Webview phase

- `src/config/convention.ts` adapters for the four commands (`ipc_argument_contract` is RED until a real caller exists in `src/**/*.ts`: it lists the four commands under "no invoke() found"; this is the only red in the gates I ran).
- State, overlay, App wiring, commands/handlers, `scripts/check-commands.mjs` ceilings, footer label `exchange_tier_label`, status bar `export_done` (`{path}`).
- The new i18n keys above already exist; add `exporting`/busy keys only if the design needs them.

## Left for the Tests phase

- `tests/convention_exchange_contract.rs` (every I/O-matrix row, both delimiters, BOM, formula guard `-ạ`, stale counter-check by really removing the in-transaction re-classification in `exchange.rs::apply_in` while keeping the `apply_import` signature). Public API for it: `core::convention::{parse_import, render_tier, classify_import, apply_import, visible_rows, ...}`, `core::delimited_text::Delimiter`, and `commands::convention::{convention_export_tier(_after_dialog), convention_open_import_preview(_after_dialog), convention_confirm_import, convention_cancel_import, clear_pending_convention_import_for_work}`.
- Consider extending `config_invariants.rs::the_open_work_mutex_guard_in_the_dialog_wires_is_acquired_after_the_blocking_call_not_before` (glossary-only today) to the two convention dialog wires.
- `tests/frontend/conventionImport.test.ts`.
- Full `cargo test` once at the end (`lib.rs` changed).

## Verified in the Rust phase

Green: `cargo test --test config_invariants --test convention_boundary --test glossary_exchange_contract --test glossary_import_dialog_contract --test prompt_set_exchange_contract --test bilingual_import_contract --test ipc_contract`, `cargo test --lib -- exchange_io bilingual delimited`, `cargo clippy --lib --tests` (0 warnings), `cargo fmt --check`. A throwaway probe (deleted) round-tripped `-ạ`, empty replacement, quoted comma/quote in CSV and TSV with BOM, and hit the duplicate / blank / unterminated / delimiter errors. Not run: `check:i18n`, `check:commands`, vitest, full `cargo test`.

# Handoff: Webview phase -> Tests phase (16-6)

Working notes only. Nothing committed.

## Done (Webview phase)

- `src/config/convention.ts`: `exportConventionTier`, `openConventionImportPreview`, `confirmConventionImport`, `cancelConventionImport` (never throw; export/open return `done|loaded|cancelled|ipc_unavailable|error`; confirm returns `{applied, error}`, no bridge maps to `UNKNOWN_IPC_ERROR`). Types `ConventionImportPreviewWire`, `ConventionImportConflictWire`, `ConventionImportDecision` (`keep_mine|take_file`).
- `src/conventionManageState.ts`: `conventionManageExchangeTier` (computed: both => work if a Work is open else global), `conventionManageExchangeBusy`, `setConventionExchangeBusy`, `conventionManageExportedPath`, `exportConventionManageTier` (cancelled dialog silent; no-bridge shows `UNKNOWN_IPC_ERROR`), `reloadConventionManageAfterImport` (= `afterWrite` ok ack: reload + one `scanConventionNow`). Busy flag and exported path are cleared in `clearConventionManage cells`.
- `src/conventionImportState.ts` (new): `openConventionImportOverlay`, `confirmConventionImportPreview`, `cancelConventionImportPreview`, `decideConventionImportAtCursor`, `setConventionImportDecision`, cursor/step, `conventionImportSummary`, `conventionImportBlocked` (true after `convention.import_stale` / `convention.no_pending_import`: Apply disabled, Cancel is the way out), `resetConventionImport`. Only `take_file` entries are sent. Open is guarded by the manage-state exchange flag and `busy`.
- `src/ConventionImportOverlay.vue` (new, z-index 11, mounted in `App.vue`): summary line, one row per conflict with two `aria-pressed` buttons (*Giữ của tôi* / *Lấy của file*), roving tabindex on the cursor row, Up/Down moves the cursor, rows set the cursor on `mousedown`/`focusin`; Esc and Huỷ dispatch `convention.import.cancel`, focus returns to `[data-convention-import-open]`.
- `src/ConventionManageOverlay.vue`: header `div.cvm-head-actions` with Xuất, Nhập, Đóng (DOM order = Tab stops 1, 2, 3); new `footer.cvm-foot` with `Tầng xuất/nhập: <tier>` (`data-convention-exchange-tier`) and the `export_done` status line (`data-convention-export-done`, `role="status"`). Hooks: `data-convention-export`, `data-convention-import-open`, `data-convention-import-keep`, `data-convention-import-take`, `data-convention-import-apply`, `data-convention-import-row`.
- Commands (`src/commands/index.ts`, `src/conventionHandlers.ts`): `convention.manage.export`, `.import`, `convention.import.confirm`, `.cancel`, `.keep_mine`, `.take_file` (the last two act on the cursor row; they have a mouse point so no `pointer` declaration). `resetConventionImport()` is called next to `resetConventionManage()` in `modes/libraryChapters.ts` and `modes/libraryImport.ts`.
- Gates: `check-commands.mjs` HANDLER_TABLE rows for the three new overlay handlers, floors `TS_FLOOR` 131->140 and `DISPATCH_FLOOR` 412->445 (live counts had drifted past the 80% line); same drift fixed in `check-panel-refs.mjs` `FILE_FLOOR` 131->140 and `check-tokens.mjs` `COMPONENT_FILE_FLOOR` 189->202.

## Deviations from the Rust handoff

- `import_apply` label: `new_count` = `new_count` + conflicts set to take-file (Glossary precedent), `kept_count` = `identical_count` + conflicts still keep-mine. The Rust handoff said plain `new_count`; this counts what the click really adds or replaces.

## Verified

Green: `vue-tsc` (via `npm run build`), eslint on the touched files, `check:commands`, `check:i18n`, `check:panel-refs`, `check:tokens`, `cargo test --test ipc_argument_contract --test config_invariants --test convention_boundary`, vitest `libraryChapters libraryImport conventionManage` (11 files, 130 tests). Not run: full vitest, full `cargo test`, `check:scope`, e2e.

## Left for the Tests phase

- `tests/frontend/conventionImport.test.ts` per the plan (derived tier at three filter values, take-file then confirm sends only that decision, cancelled dialog silent, confirm triggers exactly one rescan, Xuất and Nhập are Tab stops 1-2 in the overlay, blocked state after `import_stale`). Mock factories of `./config/convention` and `./conventionManageState` in other specs need the new names if they import the new state.
- `tests/convention_exchange_contract.rs` and the `the_open_work_mutex_guard...` extension from the Rust handoff; one full `cargo test` and full vitest at the end.

# Handoff: Tests phase (16-6) -> review

Working notes only. Nothing committed.

## Done

- `src-tauri/tests/convention_exchange_contract.rs` (new, 16 cases): every I/O-matrix row, CSV and TSV round trip, formula guard `-ạ`, BOM, seven fault files (0 writes, `no_pending_import` after), over-ceiling file, stale (command level and `apply_import` level), unknown decision key, cancelled dialogs, cancel / Work-closed plan clearing, Work tier unavailable, matcher cache dropped after confirm (observed through `convention_scan_suggestions`).
- Counter-check: replaced the in-transaction `if now != plan.kind { return ImportStale }` in `exchange.rs::apply_in` with `let _ = now;` (signature kept); exactly the two stale cases went red; restored afterwards.
- `config_invariants.rs`: new sibling test that `with_open(` and any `.lock()` come after the blocking dialog call in the two convention dialog wires.
- `tests/frontend/conventionImport.test.ts` (new, 13 cases): derived tier at three filter values (and Both without a Work), export path/cancel silence, import open/cancel/busy, take-file sends only `{b: take_file}`, one rescan after confirm, stale blocks Apply, cancel, Export/Import/Close Tab order via the real `trapTab`, button dispatches, row button moves cursor.

## Verified

- Full `cargo test`: exit 0 (111 `test result: ok` lines, none failed). Targeted gates (config_invariants, convention_boundary, ipc_argument_contract) green. `check:commands`, `check:i18n` green; conventionImport + conventionManage vitest green.
- Full vitest, first run (concurrent with full cargo): 8 files / 17 tests timed out and 2 unhandled rejections (GridBlock context). Re-run of those 8 files alone: exit 0, 333 passed. No mock-factory breakage from the new exports was found. I did not re-run the full vitest after load cleared; only those 8 files.

## Risks

- Not run: `check:scope`, e2e, CI Windows half. Native dialogs have no e2e (plan Design Notes).
