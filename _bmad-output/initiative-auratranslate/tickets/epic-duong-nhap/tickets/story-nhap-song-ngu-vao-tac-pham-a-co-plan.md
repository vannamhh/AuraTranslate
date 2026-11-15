---
title: 'Story 6.28: Nhập song ngữ vào Tác phẩm đã có'
type: 'feature'
ticket: '28'
created: '2026-11-15'
status: 'built'
baseline_revision: '8094be23d297e58581bb3761f18e7e02104a73f0'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The import overlay lets the user pick *Tác phẩm đã có* on every tab, but a `.csv`/`.tsv` (or a `.docx` read as bilingual) can only create a new Work: `confirm_bilingual_import` takes no destination, and Story 25 left a notice plus a *Tạo Tác phẩm mới* button on the bilingual preview.

**Approach:** Carry the destination through the bilingual path the way Story 7b carries it through the text path: stash it with the pending source at preview time, and on confirm append the new Chapters after the Work's last Chapter. Remove the temporary notice and button.

## Boundaries & Constraints

**Always:**
- New Chapters take `ord = MAX(ord) + 1` upward, inside the same single write transaction (AD-8, AD-11). Existing Chapter and Segment rows stay byte-identical.
- Paired segments write `target_text`, the comparison baseline and `translation_origin = bilingual_import` in the same `INSERT` (AD-47), through the existing `insert_bilingual_segments`. A Chapter that carries bilingual segments starts `InProgress`, as on the new-Work path.
- The destination lives in `PendingImportSource.destination_work_id`, set only by the preview-open call. Rebuilds and confirm read it back; the webview never resends it at confirm.
- Cleanup rules on preview, rebuild and confirm resolve against the destination Work's tier (`resolve_cleanup_rules_for`), as the text path does.
- Every refusal (mismatched rows, empty source, destination gone, `project.db` broken) writes nothing to the Work and keeps the pending source for a retry.
- Name, Ngôn ngữ nguồn and Thể loại stay locked to the Work's values on the bilingual preview when the destination is an existing Work.
- Decision (Ice, 2026-11-15): the append choreography of the text confirm (open-vs-indexed destination, re-check after relock, reindex after the lock drops) is extracted into one helper that both the text and the bilingual confirm call; it is not copied.
- Decision (Ice, 2026-11-15): the plan stays whole despite exceeding the 1600-token guideline.

**Never:**
- No change to the new-Work bilingual path's behaviour.
- No Glossary candidate scan after a bilingual append (the new-Work bilingual path does not scan either).
- No `.docx` detection or *Đọc như* switch (Story 29). No new dependency.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error handling |
|---|---|---|---|
| Append | Work with 3 Chapters, `.csv` of 2 paired rows, pattern splits each row | Chapters 4 and 5, segments `bilingual_import`, both `InProgress`; Chapters 1–3 unchanged | — |
| Unlinked | mismatched row, *Nhập, để hàng lệch chưa nối* | appended, row is one unlinked segment as on the new-Work path | empty source cell: refused as today |
| Mismatch | mismatched row, plain confirm | nothing written | `import.bilingual_mismatched_rows`, pending kept |
| Destination open | the target Work is the open Work | append reuses the open store; open editor state stays valid | — |
| Destination closed | the target Work is not open | opened from the index, appended, becomes the open Work | vanished/broken: `work.open_failed` naming it |
| New Work | Đưa vào = Tác phẩm mới | unchanged | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/project/work_creation.rs:726` `append_chapters_to_work_with_progress` -- hardcodes `.with_bilingual_columns(0, 1, false)` and inserts every Chapter `NotStarted` via `insert_segments`. Add the bilingual params `create_work_with_progress` already takes (:93-112: columns, header, `regroupings`, `mismatched_rows`), feed them to the pipeline, and branch status + insert on `chapter.bilingual_segments` exactly as `create_work` does (:496-531). Keep the `append_chapters_to_work` wrapper (:974) signature; it passes `0, 1, false, &[], Refuse` (docx/project contract tests call it).
- `src-tauri/src/commands/project/mod.rs:2934` `confirm_append_import_with_encoding_with_progress` -- the only other caller; passes the neutral bilingual values. Model for the new bilingual append core (lock pending, call append, clear pending only on success, then `write_lifecycle_after_change`); `:3108` `_indexed_with_progress` adds `finish_lifecycle_write`. `stash_pending_import_source_for` :2725, `current_pending_destination` :2748.
- `src-tauri/src/commands/project/bilingual.rs:471` `confirm_bilingual_import_with_policy` -- new-Work core; add a sibling `confirm_bilingual_append_import_with_policy(open, state, source_lang, encoding, rules, pattern, cols, header, regroupings, table_index, policy)` reusing `select_bilingual_table`.
- `src-tauri/src/commands/project/wire.rs` -- `preview_bilingual_import_from_file` (~:1100) gains `destination: Option<String>`, stashes via `stash_pending_import_source_for`, resolves rules with `resolve_cleanup_rules_for`; the rebuild wire (~:1199) resolves rules via `pending_destination(&app)` (:158). `confirm_bilingual_import` (:1243): `None` destination = unchanged; `Some` = append. The text confirm's append choreography (:936-1047: `AppendInProgressGuard`, open-vs-indexed branch, re-check `work_id` after relock, reindex after the lock drops, `replace_open_work`) must be shared, not copied — see Design Notes. Update the doc comment at :22-28 that says bilingual has no destination.
- `src-tauri/tests/ipc_contract.rs:1236-1300` -- locks the literal `resolve_cleanup_rules(&app)` inside the three bilingual wires; widen it to `resolve_cleanup_rules_for(&app` (it guards that user rules reach the bilingual path, which still holds).
- `src-tauri/tests/bilingual_import_contract.rs` -- helpers `temp_dir`, `write_file`, `pending_state`, `read_chapters`, `read_segments`; example at :222.
- `src/config/project.ts:915` `previewBilingualImportFromFile` -- add `destinationWorkId` → `destination`.
- `src/bilingualImportPreviewState.ts` -- `openBilingualImportPreview(…, destinationMode)` :378 becomes `destinationWorkId: string | null`; keep `bilingualImportPreviewDestinationMode` as a derived read (the overlay's `isNewWork` :385 uses it). Drop `pendingDestinationMode === 'new'` from `CanConfirm` :152 and `CanImportUnlinked`; the empty-Name check applies only to a new Work. Setter guards :414-428 stay.
- `src/modes/libraryImport.ts:768` `openBilingualFromFiles` passes `destinationWorkId.value`; delete `switchBilingualImportToNewWork` :811.
- `src/BilingualImportPreviewOverlay.vue:871-879,945-947` -- remove the button and notice. Add the destination line the text preview shows (`ImportPreviewOverlay.vue:1318-1325`, keys `mode.library.preview.destination_new_work` / `destination_existing_work`).
- Remove `import.preview.bilingual_use_new_work`: `src/commands/index.ts:234,1685`, its dep in `src/main.ts`, i18n `command.import.preview.bilingual_use_new_work` :845 and `mode.library.preview.bilingual_existing_work_unavailable` :1307.
- vitest `tests/frontend/libraryImportEntryMatrix.test.ts`, `libraryAddChapters.test.ts` reference the removed notice/command.
- e2e: `e2e/support/importForm.mjs` (`createWorkThroughForm`, `selectImportTab`, `dropFiles`); destination pick pattern in `e2e/specs/story-6-27-add-chapters.e2e.mjs`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/project/work_creation.rs`, `mod.rs` -- bilingual params on the append core; the text caller passes neutral values -- one append path for both shapes.
- [x] `src-tauri/src/commands/project/bilingual.rs` -- `confirm_bilingual_append_import_with_policy` -- the bilingual append core, testable without `AppHandle`.
- [x] `src-tauri/src/commands/project/wire.rs` -- destination on bilingual preview/rebuild/confirm; shared append choreography used by both confirms -- see Design Notes.
- [x] `src-tauri/tests/bilingual_import_contract.rs` -- the matrix rows Append, Unlinked, Mismatch (pending kept, Work untouched), plus New Work regression; `ipc_contract.rs` guard widened.
- [x] `src/config/project.ts`, `src/bilingualImportPreviewState.ts`, `src/modes/libraryImport.ts` -- destination id through the bilingual preview; confirm unlocked for an existing Work.
- [x] `src/BilingualImportPreviewOverlay.vue`, `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json` -- drop the temporary notice, button and command; add the destination line.
- [x] `tests/frontend/…` -- update the two specs; a case that an existing-Work bilingual preview can confirm and the IPC receives the destination.
- [x] `e2e/specs/story-6-28-bilingual-into-existing.e2e.mjs` -- create a Work, open the overlay on Tác phẩm đã có, drop a `.csv`, Xem trước, Nhập: the overlay closes and the Work shows the added Chapters.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- append `→ ✅ ĐÃ ĐÓNG <date> (Story 6.28)` under the item *Deferred from: màn nhập một khối chung, ba tab nguồn (2026-11-15)* (grep it, never read the file whole).

**Acceptance Criteria:**
- Given Đưa vào = an existing Work and one `.csv`, when the user opens the bilingual preview, then no "chưa làm được" notice or *Tạo Tác phẩm mới* button exists, the preview names the destination Work, and *Nhập* is enabled once rows pair.
- Given that preview confirmed, when the user looks at the Work, then the new Chapters follow its previous last Chapter and their target cells show the imported translation.

## Implementation Notes

- `append_chapters_to_work_with_progress` now carries the bilingual params; text callers pass `0, 1, false, &[], Refuse`. Status and segment insert branch on `chapter.bilingual_segments` as in `create_work`.
- `wire::append_into_destination` (private) holds the open-vs-indexed choreography for both confirms; the bilingual branch spawns no Glossary scan. The dead `confirm_append_import_with_encoding_indexed[_with_progress]` pair was deleted and its `project_contract` case now composes `finish_lifecycle_write` the way the helper does.
- `ipc_contract`: the reindex guard measures `append_into_destination` as its own seam (`wire_fn_body` now bounds at any column-0 `fn`); the bilingual cleanup guard accepts `resolve_cleanup_rules_for(&app`.
- The destination travels only on `preview_bilingual_import_from_file` (`destination`); rebuild and confirm read it from the pending source. A failed preview open cancels the pending source and so drops the destination (unchanged behaviour).
- The bilingual preview shows the destination line with the existing keys `destination_new_work` / `destination_existing_work`.
- Counter-check: with the `insert_bilingual_segments` branch removed from the append core, the Append and Unlinked cases go red at `translation_origin` / segment count; green again after restore.
- e2e `story-6-28-bilingual-into-existing`: two cases (destination open, destination closed), 2/2 green outside the sandbox. `browser.keys(['Meta','1'])` timed out after the workspace opened on an appended Chapter; the spec returns to the Library by clicking the mode tab, and the timeout is an owned debt item (`Chủ: Epic 6`).

## Plan Change Log

## Review Triage Log

Pass 1 (quick, loop 0): high 0 · medium 4 · low 7 · false 0 · maybe-false 0.

- medium · patch — `BilingualImportPreviewOverlay.vue` CSS: `.bip-destination` split the `.bip-head-note, .bip-hint` group, so the subtitle gains a bottom margin on every bilingual preview, new-Work included.
- medium · patch — `confirm_append_import_with_encoding_indexed[_with_progress]` has no production caller after the extraction; `project_contract.rs:6521` guards a parallel function and its doc comment names a caller that is gone.
- medium · patch — `ipc_contract::all_five_reindex_library_call_sites…` finds the append reindex inside the private helper only because `wire_fn_body` stops at the next `pub fn`; the bilingual append has no seam.
- medium · patch — matrix row "Destination closed" has no test through the real wiring; the e2e covers only a destination that is the open Work, and the "New Work regression" case calls the unchanged pure function.
- low · patch — `importPreviewBilingual.test.ts:425` `toHaveLength(11)` holds whatever production passes (the mock wrapper drops extras); assert the real adapter's `invoke` payload instead.
- low · patch — `snapshot_chapters` compares `id, title, status` and a partial segment row while its message claims byte-identical rows.
- low · patch — `append_into_destination` returns `indexer_is_missing()` on the open-Work branch (unreachable, but names the wrong cause).
- low · patch — stale doc comments: `wire.rs` `confirm_bilingual_import` (core is no longer only `create_work`), `mod.rs` `PendingImportSource::destination_work_id` (bilingual now stashes it).
- low · patch — comment rules: doc comment on the private helper, rewritten comment lines left in Vietnamese (`wire.rs` cleanup-rule docs, `work_creation.rs` append doc).
- low · patch — `bilingual_import_contract.rs` `append_bilingual` carries a reasonless `#[allow(clippy::too_many_arguments)]` on a 4-parameter fn.
- low · patch — e2e checks AC2 through IPC only; nothing on screen shows the new Chapter or its target text.

## Design Notes

The text confirm already solves the hard part of appending: the destination may or may not be the open Work, the lock is released between check and use, and reindexing must run after the `OpenWorkState` lock drops. Extract that block from `confirm_import_with_encoding` into one helper in `wire.rs` that takes the work id and a closure `FnOnce(&mut OpenWork) -> Result<(), IpcError>` (the core, including `write_lifecycle_after_change`), and returns the `CreatedWork` plus `new_chapter_ids`; the not-open branch wraps the closure's result in `finish_lifecycle_write`. Both confirms call it. A second copy of that lock dance would drift the way the drop path drifted before Story 25.

## Verification

**Commands:**
- `npm run build && cargo test --test bilingual_import_contract && cargo test --test ipc_contract` -- pass.
- `cargo test --test project_contract append` and `cargo test --test docx_contract` -- the text append path still passes.
- `npx vitest run tests/frontend/libraryImportEntryMatrix.test.ts tests/frontend/libraryAddChapters.test.ts tests/frontend/importPreviewBilingual*` -- pass.
- `npm run check:i18n && npm run check:commands && npm run check:tokens` -- pass.
- `npm run test:e2e -- --spec e2e/specs/story-6-28-bilingual-into-existing.e2e.mjs`, outside the sandbox, after rebuilding with `--features wdio`.
