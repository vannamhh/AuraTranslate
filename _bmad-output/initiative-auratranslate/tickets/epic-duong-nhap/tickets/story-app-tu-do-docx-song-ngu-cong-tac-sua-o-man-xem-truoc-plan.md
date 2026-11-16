---
title: 'Story 6.29: App tự dò .docx song ngữ, công tắc sửa ở màn xem trước'
type: 'feature'
ticket: '29'
created: '2026-11-16'
status: 'built'
baseline_revision: '5fda255c0f7eba7a39769f5b1bf839602e05996e'
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

**Problem:** A `.docx` dropped in the Files tab is read as text unless the user ticks a temporary per-row *Đọc như song ngữ* box before preview, so a bilingual table document opens the wrong preview and fixing it means going back to the file list.

**Approach:** Per the AD-39 addendum (Ice, 2026-11-15): opening a lone `.docx` is one Rust call that reads it once, detects the shape from its content, stashes enough to build either shape, and returns the chosen shape with its preview. Both previews get a *Đọc như văn bản thường / Đọc như song ngữ* switch that rebuilds the other shape from the stash, without reading the path again. The per-row box goes.

## Boundaries & Constraints

**Always:**
- Detection is one Rust function over one `read_docx` result; no detection error — no qualifying table means text.
- `PendingImportSourceState` stays the only holder of the source; the switch never reads the file path (the `ipc_contract` "never reads the file" guard is extended to the switch wire).
- After a switch, the pending shape matches the confirm wire of the shown preview (text confirm sees `Blob` + `DocxSidecar`; bilingual confirm sees `BilingualTables`). Confirm wires are unchanged.
- Name, Ngôn ngữ nguồn, Thể loại and the destination carry across a switch.
- `.csv`/`.tsv` are always bilingual, `.txt`/`.md` always text; their paths are unchanged.
- Decision (Ice, 2026-11-16): a `.docx` is bilingual when its top-level tables of ≥2 columns hold ≥ 50 % of the document's text characters. Measured: the repo's only real `.docx` (`docs/Bản sao của Chuộc tội…(Phần 1).docx`) is 1 table, 1 row, 2 columns, all 12 366 chars in the table, so a row-count rule would miss it.
- Decision (Ice, 2026-11-16): each switch into bilingual starts like a fresh open (column roles 0/1, header off, table unchosen, no regroupings); the chapter pattern carries both ways.
- Decision (Ice, 2026-11-16): Ngôn ngữ nguồn = Tự dò with a docx detected bilingual follows the `.csv` rule: the overlay shows `BILINGUAL_NEEDS_SOURCE_LANG_ERROR`, and after picking a language the user presses Xem trước again without re-picking the file. A text → bilingual switch passes the language the text preview detected.
- Decision (Ice, 2026-11-16): the plan stays whole despite exceeding the 1600-token guideline.

**Never:**
- No change to segmenting, confirm or append write paths. No new dependency.
- No detection for a `.docx` inside a multi-file batch (it stays a per-item `BatchUnsupportedFormat`, as today).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error handling |
|---|---|---|---|
| Plain docx | paragraphs only | text preview, switch shows *văn bản thường* | — |
| Bilingual docx | one 2-column table holding the text | bilingual preview | — |
| Novel with a small table | long prose + one small 2-column table | text preview (table share < 50 %) | — |
| Tự dò + bilingual docx | Ngôn ngữ nguồn = Tự dò | no preview opens | `BILINGUAL_NEEDS_SOURCE_LANG_ERROR`; Xem trước again after picking a language |
| Switch to bilingual | text preview of a docx with no 2-column table | stays text | `import.bilingual_no_two_column_table`, preview unchanged |
| Switch and back | bilingual → text → bilingual | each preview rebuilt from the stash; file deleted from disk meanwhile still works | — |
| Several tables | bilingual docx with 2 qualifying tables | bilingual preview asks for the table, as Story 16c | — |
| Confirm after switch | switched to text, Nhập | Work created as a text docx import (blocks/images kept) | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/docx/mod.rs:134,159,176` -- `DocxParsed` (`text`, `blocks`, `body`, `images`), `top_level_tables()`, `read_docx`. Reuse; do not change.
- `src-tauri/src/core/segment/import.rs` -- text docx branch `import_file` :883-901 (`Blob` + `DocxSidecar` :817) and bilingual `import_bilingual_docx` :1253 (`docx_table_rows` :1295, `BilingualNoTwoColumnTable` :295) each read the file. Add one detect function taking the parsed docx and returning a `DocxSource { text, sidecar, tables }` plus the chosen shape; both existing branches build from it. `classify_import_paths(paths, docx_as_bilingual)` :1134, `ClassifiedImportPath.is_docx` :1121 -- drop the toggle param; a lone `.docx` gets its own kind/verdict (*dò khi xem trước*), a docx in a multi-file batch keeps today's handling. Unit tests :1354+.
- `src-tauri/src/core/segment/pipeline.rs:196-247` -- `PipelineShape` (`Blob`, `BilingualTables` :218 stash-only); `select_bilingual_table` is `import.rs:1306`.
- `src-tauri/src/commands/project/mod.rs:856` `PendingImportSource { shape, docx_sidecar, destination_work_id }` -- add `docx_source: Option<DocxSource>`; `stash_pending_import_source_for` :2721 keeps it. Text confirm core :2891, `ImportEncodingPreview` :1580, `ChapterSplitPreviewWire.already_chaptered` :1225, `build_file_import_batch_wire` :1680.
- `src-tauri/src/commands/project/wire.rs` -- text `preview_import_encoding_from_file` :713 (a lone docx through it reads as text and also stashes `docx_source`, so pattern reloads that re-read the disk keep the switch working); bilingual `preview_bilingual_import_from_file` :1080, `rebuild_bilingual_import_preview` :1151, `preview_bilingual_import` core `bilingual.rs:222` (docx branch :246-285). New wires: `preview_docx_import(path, source_lang, chapter_pattern, destination, …)` → envelope `{ shape: 'text' | 'bilingual', text?: FileImportBatchWire, bilingual?: BilingualImportEncodingPreview }`; `switch_docx_import_shape(shape, …same params…)` → same envelope from the stash. Register in `lib.rs:1131-1142`.
- `src-tauri/tests/ipc_contract.rs:62,~1187-1249` -- registration lists and the never-reads-the-file guard; add the two wires. `tests/fixtures_docx.rs` (`plain` :52, `table_two_columns` :78, `table_one_row_multi_paragraph` :105); `bilingual_import_contract.rs:2655 mod docx_source`.
- `src/config/project.ts` -- adapters + validators (`isFileImportBatchWire` :678, `isBilingualImportEncodingPreview` :894); add `previewDocxImport`, `switchDocxImportShape`, `isDocxImportPreviewWire`.
- `src/modes/libraryImport.ts` -- `submitFilePath` :744 branches on verdict; add the lone-docx branch calling `previewDocxImport` and opening the module the response names. Remove `docxAsBilingual` :278, `setDocxReadAsBilingual` :382. `src/config/library.ts:542-550` classify adapter.
- `src/importPreviewState.ts` (2910 lines, read by offset) `openImportPreviewFromFile` :1461; `src/bilingualImportPreviewState.ts` `openBilingualImportPreview` :379 (resets :396-410). Each gains an open-from-response entry that does not call its from-file wire, and a close that keeps the Rust pending.
- `src/ImportPreviewOverlay.vue`, `src/BilingualImportPreviewOverlay.vue` -- the switch, shown only when the source is a docx. Siblings in `src/modes/LibraryMode.vue:379-380`.
- `src/LibraryImportOverlay.vue:100,411-414` -- remove the checkbox; the docx row reads *.docx, dò khi xem trước*. i18n `src/i18n/vi.json` :1126-1134 (`mode.library.file_docx_as_bilingual` goes).
- vitest that reference `docxAsBilingual`: `libraryImportEntryMatrix`, `libraryImportDropMultiple`, `libraryImportBlocksResubmitWhilePreviewOpen`, `importPreviewDestination`, `libraryImportRetryAfterFailedConfirm`.
- e2e: `e2e/support/importForm.mjs` `dropFiles` :155; `e2e/support/miniDocx.mjs` builds a store-zip docx without tables -- add a two-column table builder. No docx e2e exists yet.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/segment/import.rs` -- one detect function + `DocxSource`; both docx branches build from it; classifier without the toggle -- one read, one decision.
- [x] `src-tauri/src/commands/project/{mod,wire,bilingual}.rs`, `src-tauri/src/lib.rs` -- `docx_source` on pending; `preview_docx_import`, `switch_docx_import_shape`.
- [x] `src-tauri/tests/…` -- detect unit cases (matrix rows 1–3, threshold edges); contract cases for switch-and-back with the file deleted, switch refusal leaving pending unchanged, confirm after switch; `ipc_contract` lists and file-read guard.
- [x] `src/config/project.ts`, `src/config/library.ts`, `src/modes/libraryImport.ts`, both state modules -- docx opener and the switch.
- [x] `src/ImportPreviewOverlay.vue`, `src/BilingualImportPreviewOverlay.vue`, `src/LibraryImportOverlay.vue`, `src/i18n/vi.json` -- switch UI; checkbox removed; row label.
- [x] `tests/frontend/…` -- update the five specs; cases: docx response opens the named preview; switch calls the wire and swaps overlays keeping Name/language/genre/destination.
- [x] `e2e/support/miniDocx.mjs`, `e2e/specs/story-6-29-docx-detect.e2e.mjs` -- the ticket's `verify`.

**Acceptance Criteria:**
- Given a plain `.docx` dropped on the Files tab, when the user presses Xem trước, then the text preview opens and its switch reads *Đọc như văn bản thường*.
- Given a two-column-table `.docx`, when the user presses Xem trước, then the bilingual preview opens.
- Given either preview of a `.docx`, when the user flips the switch twice, then the other preview appears and then the first again, without choosing the file again.
- Given the import overlay, when a `.docx` is listed, then no *Đọc như song ngữ* box exists and the row reads *.docx, dò khi xem trước*.

## Implementation Notes

- Detection counts non-whitespace characters of paragraphs plus top-level table cells; a table qualifies at >=2 columns in its widest row; the share test is `2 * table_chars >= total` (exactly half is bilingual).
- The refusal for a switch without a two-column table reuses the existing `import.bilingual_no_table` (`ImportError::BilingualNoTwoColumnTable`); the plan's `import.bilingual_no_two_column_table` names no registered key.
- Tự dò + bilingual docx: Rust refuses with the new `ImportError::BilingualNeedsSourceLang` (existing key `err.import.bilingual_needs_source_lang`) and keeps nothing pending; the lone-docx open path shows any error in the import overlay instead of opening a preview in error state.
- `FilesImportOutcome` is unchanged (`segment_files_boundary` scans its literal); the text from-file wire builds a lone `.docx` outcome from `read_docx_source` so it also stashes `docx_source`.
- The switch is a `SelectMenu` (`@change`, no new command id); the switch wire takes `shape`, `source_lang`, `chapter_pattern` and reads the destination from the pending source.
- e2e `story-6-29-docx-detect`: 2/2 green outside the sandbox (wdio build of the patched tree). The text → bilingual switch sends the language the text preview adopted (`adoptPreviewSourceLang` replaces `auto`), guarded by a vitest case.

## Plan Change Log

## Review Triage Log

Pass 1 (quick, loop 0): high 0 · medium 1 · low 4 · false 1 · rejected low 2.

- medium · patch — `switch_docx_import_shape` built a fresh preview without resetting `Tier2BlockOverridesState`/`ChapterOriginOverridesState` (only `preview_docx_import` did), so a stale chapter-origin override could reach the text confirm; resets added and the reset guard extended.
- low · patch — detection judged ≥2 columns on raw rows while `tables` dropped blank rows, so a table whose only 2-cell row was blank detected bilingual and the open failed instead of falling back to text; detection now uses the same qualifying rule.
- low · patch — switch vitest asserted Name and language only; genre and destination assertions added.
- low · patch — `is_docx` read by nothing after the toggle went; field, validator entry and its unit test deleted.
- low · patch — restating comments on private `docxSource` refs and `DocxShape` deleted.
- false — "gap between reset and open lets a submit through": `openFileImportPreview`/`openBilingualPreview` set `opening` synchronously in the same tick as the reset and the injected call resolves in a microtask, so no user event can run while both open flags are false.
- low · rejected — `DocxSource` duplicates sidecar blocks/images in pending (up to 2× a docx's images during preview); not met in everyday use and the fix restructures the confirm path.
- low · rejected — an unknown `shape` string on the switch wire answers `import.no_pending_source`; the webview only sends `text`/`bilingual`, and a dedicated key adds surface.

## Design Notes

The switch closes one state module without `cancel_import_preview` and opens the other from the wire's response; only the wire touches Rust pending. A lone `.docx` through the old text from-file wire still reads as text but stashes `docx_source` too, so the text preview's pattern reload (which re-reads the disk, `importPreviewState.ts:2240`) does not lose the way back to bilingual.

## Verification

**Commands:**
- `npm run build && cargo test --test bilingual_import_contract && cargo test --test ipc_contract && cargo test --test docx_contract` -- pass.
- `cargo test --lib core::segment::import` -- detect and classifier unit cases pass.
- `npx vitest run` on the five updated specs and the new ones -- pass.
- `npm run check:i18n && npm run check:commands && npm run check:tokens` -- pass.
- `npm run test:e2e -- --spec e2e/specs/story-6-29-docx-detect.e2e.mjs` outside the sandbox -- pass.
