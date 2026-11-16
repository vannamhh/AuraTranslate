---
title: 'Xuất Chương qua Save As'
type: 'feature'
ticket: ''
created: '2026-11-16'
status: 'built'
baseline_revision: 'dbf63880f05fe8009faf1982a735f129417cc86b'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - 'AGENTS.md'
  - 'src/AGENTS.md'
  - 'src-tauri/AGENTS.md'
  - 'tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Export Chapter opens a folder picker and invents the filename. Users need Save As to choose both name and location.

**Approach:** Keep the existing export screen. Its Export action opens native Save As with a suggested filename and a format filter. Write the exact selected path. ReadingMode and Library share this behavior.

## Boundaries & Constraints

**Always:** AD-1, AD-48; FR87–FR89, FR121, FR130–FR131. Rust owns the dialog and filesystem. Release store locks before opening the dialog. Keep format-specific default suffixes. Native Save As confirms replacement of an existing file; never silently number the selected main filename. Prepare output before replacing the old file. Image links must match their actual sidecar directory.

**Never:** Add dependencies or permissions; change translation content, export options or scope. Do not stage/commit pre-existing `website/assets/` or `plan-website-thong-tin-chinh-xac.md`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| New file | Custom path, each format | Correct docx/md/txt filter; exact selected path written and reported | No error |
| Cancel | Cancel Save As | No writer call; export screen remains usable | No error |
| Replacement | Existing file; native confirmation accepted | Replace at selected path without numbering | Preparation/write failure preserves old file |
| Images | Custom filename; sidecar name occupied | Keep main filename; allocate a fresh sidecar; references match it | Never overwrite old images; clean only new artifacts on failure |
| Invalid destination | Missing parent, directory target, write failure | No false success | Show IPC error; allow retry |
| Stale dialog | Scope/format changes or overlay closes | Never write new options under a stale destination | Freeze controls or discard stale result |

</frozen-after-approval>

## Code Map

- `src/exportState.ts`: `runExport`, `pickFolder`, `writeFormat`; snapshot the export request across the dialog.
- `src/config/export.ts`: picker and three write adapters; replace folder contract with file destination and retain runtime validation.
- `src/ExportOverlay.vue`, `src/i18n/vi.json`: picker state and folder-specific wording.
- `src-tauri/src/commands/export.rs`: dialog wire, three pure writers, `write_export_file`; currently validates directories and invents names.
- `src-tauri/src/core/export/{new_file,image_files,mod}.rs`: current non-overwriting numbered output; preserve unrelated users. `options.rs` already supplies extensions/suffixes.
- `src-tauri/src/core/delimited_text.rs:294`: existing atomic replacement pattern using a sibling temporary file, sync and rename.
- `src-tauri/src/lib.rs`, `src-tauri/tests/config_invariants.rs`: registration and dialog-signature guard if command is renamed.
- `tests/frontend/export*.test.ts`, `readingExportChapter.test.ts`: mocked picker contracts and real command/overlay behavior.
- `src-tauri/tests/export_{contract,docx_contract,block_contract,text_contract}.rs`: file, image, content and collision contracts.

## Tasks & Acceptance

**Execution:**
- [x] Rust: red tests for exact paths, replacement, failures and images; implement atomic file replacement and independent, exclusive sidecar allocation.
- [x] Rust wire: native Save As with default name/filter; update command registration and its guard; keep locks out of dialog lifetime.
- [x] Webview: update adapter/state contracts, wording and stale-result handling; test through `export.run`.
- [x] Verify touched contracts, counter-check a new guard, obtain independent review and record manual/CI limits.

**Acceptance Criteria:**
- Given a Chapter is being read, when Export Chapter then Export is pressed, then Save As allows changing name/location while retaining that Chapter scope.
- Given each supported format, when Save As succeeds, then content, image references and reported path match the selected destination and format.
- Given cancellation or failure, when returning to export, then no false success appears and retry remains possible.

## Implementation Notes

- Ice authorized continuing with the dirty tree and leaving its existing files uncommitted.
- Website edits in `website/app.js` and `website/index.html` also predate implementation and stay outside this change.
- Full route: cross-layer change with replacement semantics; more than 100 changed lines expected. Native dialog uses existing format suffixes; writers replace the exact destination.
- Allocate sidecars independently of the main filename. Copy/build before atomic replacement; preserve all previous sidecars and remove only new artifacts on failure.

## Plan Change Log

## Review Triage Log

- medium · patch — Full Rust verification found the publishing-copy fixture in `reviewer_import_contract` still passing a directory; it now passes an explicit `.docx` destination and all 27 tests in that target pass.

- low · patch — Verification gate `check:panel-refs` caught `pickerSequence` missing from `resetExport`; incrementing it on reset now invalidates pending picker tokens and the gate passes.

- medium · patch — Custom macOS filenames containing `?` reach Markdown sidecar URLs unescaped; `destination_of`/`markdown_destination` treat it as a query delimiter. Resolved by sanitizing/bounding the independent sidecar stem; selected main filename remains exact. Regression observed red, then green.
- medium · patch — Atomic temporary names append 41 characters to the chosen filename; valid 220-character ASCII names exceed a 255-character component limit. Resolved with an independent UUID sibling temp name. A 220-byte selected filename regression observed red, then green.

## Verification

- Red then green: touched frontend export tests and Rust export integration targets; build frontend before Rust tests.
- Counter-check one new production seam by removing it, running its target to prove the expected failure, and restoring it.
- Changing command registration requires one full-suite run under AGENTS.md; do not repeat without new evidence.
- Read current CI and latest nightly e2e. Report only observed results.
- Real app: inspect Save As defaults/filter, cancel, custom name and replacement. If unavailable, record owned debt; never infer native dialog behavior from mocks or mark done.

### Observed evidence

- Phase logs and matrix audit: `/tmp/auratranslate-save-as/{rust,webview,tests}-result.md`. Every automated matrix row has exercised tests; native dialog rendering/confirmation remains explicitly manual debt.
- Atomic publication counter-check removed the rename operation: custom replacement case failed parsing retained old bytes; restoration passed. Stale-completion guard removal exposed an old result; restoration passed.
- Latest push CI [43478764582](https://github.com/vannamhh/AuraTranslate/actions/runs/43478764582) passed macOS and Windows on `7e6b6b1e5f25a196288123ba889f1075b3514c23`; this predates the change and proves nothing about its Windows behavior.
- Latest scheduled run [43769215036](https://github.com/vannamhh/AuraTranslate/actions/runs/43769215036) passed macOS checks and nightly e2e, but Windows failed `quick_translation_contract::purging_all_rows_shrinks_db_plus_wal_on_disk`: `wal_checkpoint(TRUNCATE) was blocked by a reader` (expected no error). Same older revision; no Save As code was pushed.
- Native checks are recorded in `deferred-work.md` with Ice/macOS and B7/Windows ownership.

### Final local verification

- All 15 pre-push gates passed, including lint, Clippy, formatting, debt ownership and FR77. Frontend: 288 files / 4,651 tests passed; production build passed. Existing chunk/dynamic-import warnings remain.
- Rust: all 115 integration targets and both unit-test targets passed across the unrestricted full attempt and its continuation (3,213 passing cases; 32 explicitly ignored). Doctests: 1 passed, 2 explicitly ignored. No successful target was needlessly repeated after the remaining fixture fix.
- The first sandboxed Rust attempt could not bind localhost in AI wire tests; unrestricted execution passed those targets. The unrestricted attempt stopped at the outdated publishing fixture; after its one-line correction, that target and all 32 not-yet-run integration targets passed.
- Final formatting/diff checks passed. Only the corrected reviewer fixture changed after the tested code snapshot, and its full target was rerun successfully.
- Native dialog/macOS and Windows confirmation remain owned manual checks, so this plan is built, not done.
