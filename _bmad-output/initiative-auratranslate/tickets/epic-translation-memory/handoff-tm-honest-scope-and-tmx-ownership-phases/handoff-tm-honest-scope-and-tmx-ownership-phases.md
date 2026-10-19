---
type: handoff
title: "R-5 phase handoff: Rust and Webview done"
status: done
created: 2026-10-06
skill: bmad-build
---

# R-5 phase handoff: Rust and Webview done

Working notes for the next agent. Spec: `spec-epic-7-retro-r-5-tm-honest-scope-and-tmx-ownership.md`. Nothing is committed.

## Wire contract (Rust is final)

- `tm_list_pairs` row (`TmGroupRowWire`) gains `hidden_copies: usize` (snake_case on the wire, like its siblings). It counts stored copies of the same (source, target) in the loaded tiers that the tier, origin and search filters hide, minus `copies.length`. Work tier counts only when a Work is open.
- `tm_confirm_import` takes a new argument `file_is_mine: bool`. The Tauri argument key is camelCase: `invoke('tm_confirm_import', { fileIsMine })`. It is required (no default): `ipc_argument_contract` is RED until `src/config/tm.ts::tmConfirmImport` sends `{ fileIsMine }`.
- `TmxImportSummaryWire` gains `future_dated_count: usize` (snake_case on the wire), counting inserted pairs whose file date was after the import moment and replaced by it.
- The preview wire (`TmxImportPreviewWire`) is unchanged.

## Behaviour the UI must reflect

- Box off: file label `self` and unlabelled pairs land as `other`; `bilingual_import` stays. Box on: `self` and unlabelled land as `self`; `other` stays `other`, `bilingual_import` stays.
- Edit and delete scope are unchanged; `hidden_copies` only feeds the notes. A Global note in the edit form needs the row's own `copies` to hold a `global` tier entry (the Rust side sends no separate flag).
- Past or missing dates are not counted in `future_dated_count`.

## Verified in Rust

`cargo test` green for `tm_contract`, `tmx_contract`, `tm_wire`, `ipc_contract`, `config_invariants` (shell prefix with `app` first is intact). `ipc_argument_contract` fails only on `fileIsMine` missing from the TS invoke (expected until phase 2). New named tests: `tmx_contract` (origin box off/on, Global confirm, future date, `pair_origin_for`), `tm_contract::manage` (three `hidden_copies` tests). Counter-checks done by real removal (self passthrough, clamp, hidden = 0): each went red in the named tests above.

## Handed to Webview (done, see Phase 2 notes)

Everything under `src/`, `src/i18n/vi.json`, vitest (`tmManage`, `tmImport`, `tmConfigGuards`), then re-run `ipc_argument_contract`, `npm run check:i18n && npm run check:tokens`.

## Phase 2 (Webview) notes

- Done: `config/tm.ts` (hidden_copies guard via `isCount`, `future_dated_count`, `tmConfirmImport(fileIsMine)`), `tmImportState.ts` (writable `tmImportFileIsMine` ref, reset on open and on `resetTmImport`), `TmImportOverlay.vue` (labelled checkbox above the origin note), `TmManageOverlay.vue` (Global note only with a global copy, hidden-copies note in edit and delete hint, future-date sentence), `vi.json` (edit_hint no longer names Global; new keys `tm.manage.edit_global_note`, `tm.manage.hidden_copies_note`, `tm.import.file_is_mine_label`, `tm.exchange.import_future_dated`; origin_note rewritten).
- Verified: vitest tmManage/tmImport/tmConfigGuards green (104), vue-tsc, check:i18n, check:tokens, check:lint, check:commands, `cargo test --test ipc_argument_contract` green. Counter-checks of the vitest guards by removal were not run in phase 2; the orchestrator ran two afterwards (confirm sends `false` instead of the ref; the edit-form hidden-copies note removed): each turned exactly its own test red, then restored.
- Template text nodes must go through `t()` inline (check:i18n A2 rejects `{{ computedString }}`).
