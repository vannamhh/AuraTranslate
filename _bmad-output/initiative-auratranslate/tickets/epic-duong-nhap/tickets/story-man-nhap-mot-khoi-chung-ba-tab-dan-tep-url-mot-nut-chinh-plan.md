---
title: 'Story 6.25: Màn nhập một khối chung, ba tab Dán · Tệp · URL, một nút chính'
type: 'feature'
ticket: '25'
created: '2026-11-15'
status: 'built'
baseline_revision: '853229b3688ee13c66dc86efbd78f8de3728906d'
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
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/import-entry.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The import overlay grew one block and one button per source: four ways forward ("…từ văn bản", "…từ tệp", "…từ tệp song ngữ", "Nhập từ website…"), and Name/Language/Genre asked again on the separate URL screen. The drop path takes any file into the monolingual path with no extension check.

**Approach:** Rebuild `LibraryImportOverlay` per EXPERIENCE.md §Đường nhập — màn nhập: the common block once (Đưa vào · Tên · Ngôn ngữ nguồn · Thể loại, locked to the Work's values when an existing Work is picked), a tablist Dán · Tệp · URL, one primary button (`Xem trước ⌘↵`, or `Tải N link ⌘↵` on URL). Delete the URL screen. Rust owns one entry extension list and the file-kind verdict.

## Boundaries & Constraints

**Always:**
- One Rust list `txt md docx csv tsv` feeds the OS dialog filter and the drop classification. Rust also returns the per-file kind (text / bilingual / unsupported). TS only renders it (AD-1).
- Only the selected tab is submitted. Switching tabs never clears another tab's content.
- The URL tab keeps the counter `N link · sẽ tạo N Chương · M domain` and the promise text verbatim (NFR19).
- Every rejected file shows a reason and a `Bỏ` button, and the primary button stays locked with a sentence. Nothing is dropped silently.
- An empty Name still locks the primary button on Dán and Tệp, as today. Story 26 removes that lock.
- Bilingual + existing Work: the bilingual preview states that this is not possible yet and offers a button that switches the target to Tác phẩm mới (until Story 28).
- Keys: `Mod+Enter` = primary, `Esc` = close, `←`/`→` move between tabs while focus is on the tablist (roving tabindex). Each tab, `Bỏ` and invite is a dispatched command with a mouse point.
- Decision (Ice, 2026-11-15): a `.docx` row on the Tệp tab carries a temporary toggle *Đọc như song ngữ*, off by default. Off, the file takes the text path. On, it takes the existing bilingual path and counts as a bilingual file for the one-kind rule. This keeps the bilingual `.docx` table importable until Story 29 replaces the toggle with detection.
- Decision (Ice, 2026-11-15): the last-used tab is remembered only for the session, in a module ref. On every app start the overlay opens on Dán.

**Never:**
- No name auto-fill. No editable Genre in the preview (Story 26).
- No "Thêm Chương…" opener (Story 27).
- No bilingual import into an existing Work (Story 28).
- No `.docx` auto-detect or Đọc như toggle (Story 29).
- No `localStorage`. No new dependency.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error handling |
|---|---|---|---|
| Paste | text on Dán, Name set | `Xem trước` opens the text preview | — |
| Files | 2 `.txt` + 1 `.md` | the multi-file text preview | — |
| One bilingual | 1 `.csv` | the bilingual preview | `auto` language: existing `bilingual_needs_source_lang` error |
| Mixed | 2 `.txt` + 1 `.csv` | `.csv` row in `error` (*song ngữ, khác loại*), primary locked | sentence *Một lượt nhập chỉ một loại…*; `Bỏ` on `.csv` unlocks |
| `.docx` table | 1 `.docx`, toggle *Đọc như song ngữ* on | the bilingual preview | toggle on + another `.txt`: mixed rule applies |
| Two bilingual | 2 `.csv` | both rows marked, primary locked | sentence names the one-file rule |
| Unsupported drop | `.pdf` dropped | row marked *không nhận loại này* | primary locked until `Bỏ` |
| Drop anywhere | drag over overlay on Dán | dashed `primary` frame *Thả để thêm N tệp vào tab Tệp*; on drop, switch to Tệp and append | frame takes no focus |
| URL | 3 links pasted | `3 link · sẽ tạo 3 Chương · …`, button `Tải 3 link` | 0 links: button locked |
| Existing Work | Đưa vào = Kiếm Lai | Name/Lang/Genre show Kiếm Lai's values, disabled, on every tab | bilingual: see Always |
| Empty-Library invites | *dán* / *chọn tệp* | overlay opens on Dán / Tệp | — |

</frozen-after-approval>

## Code Map

- `src/LibraryImportOverlay.vue` (499 lines) -- the overlay to rebuild. Keep the destination fieldset (:154-205), Name (:207), `SourceLangChips` (:213), genre `SelectMenu` (:231), `trapTab` (:104) and Esc (:129). Replace the source blocks (:141-152, :251-323) with the tablist and tab bodies.
- `src/modes/libraryImport.ts` (794 lines) -- module-ref state. Reuse `submitPastedText` :622, `submitFilePath` :665, `submitBilingualFilePath` :692, `submitPastedUrls` :709, the `pastedUrl*` counters :336-359, `beginSubmit` :415 and `wireDragDropOnce` :752 (today it pushes every drop into `droppedFilePaths`). Add an active-tab ref, a classified file list and the drag-count ref.
- `src/modes/libraryImportOverlayState.ts` -- delete `libraryUrlImportScreenIsOpen`.
- `src/modes/LibraryUrlImportScreen.vue` -- delete it and its mount at `LibraryMode.vue:381`. Move the counter and promise markup into the URL tab. `titlebarImportSource.ts` reads the deleted flag, so update it.
- `src/modes/LibraryMode.vue:580,583` -- the two invites both dispatch `library.import.open`. Point them at tab-opening commands.
- `src/commands/index.ts:1615-1692` + `src/main.ts:~619` -- import command registry and deps. Remove `library.import_urls_open/close` and `library.import_bilingual`. Add tab-select, open-on-tab and file-remove commands. No value imports from vue.
- `src/i18n/vi.json` (keys `mode.library.*`, ~:1113-1124) -- the dropzone string still says `.txt/.md`. Drop keys that become unused.
- Tab pattern: `ConventionManageOverlay.vue:253-275` (`role="tablist"`, `.ui-tab`, `src/tokens/controls.css:683-700`). It has no arrow keys; add them in this overlay only. `Mod+Enter` inline as in `ExportOverlay.vue:146-150` (`aria-modal` already blocks the global chord, main.ts:1194).
- `src-tauri/src/core/segment/import.rs:67,1049` -- `SUPPORTED_EXTENSIONS` and `BILINGUAL_SUPPORTED_EXTENSIONS`. `.docx` is in both. Add the entry list and a pure `classify_import_paths` (kind per path plus a batch verdict). Leave the two existing lists alone.
- `src-tauri/src/commands/library.rs:1004-1036` -- two dialog commands. Collapse them into one `library_pick_import_files` with the entry filter that returns classified files. Register the classify command in `lib.rs` (shared wiring, so run the full suite).
- `src/modes/importPreview/BilingualImportPreviewOverlay.vue` + `bilingualImportPreviewState.ts:368` -- the bilingual path has no destination. Pass `destinationMode` in for the notice and the switch-to-new button.
- Tests: vitest `tests/frontend/libraryImportOverlay*.spec.ts`, `libraryImportDropMultiple`, `importPreviewUrls`, `importPreviewBilingual`, `titlebarToolsMenu`. In e2e, `e2e/support/importForm.mjs::createWorkThroughForm` and every spec that clicks `[data-library-url-screen-open]` must move to the URL tab. `bilingual-cut-mouse.e2e.mjs:69-71` and `story-12-6…:204-205` type a CSV path into an input that is now Name (already stale). No spec simulates a drop today.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/segment/import.rs` -- entry list + `classify_import_paths` with unit tests for each matrix kind -- AD-1: Rust owns the verdict, including a `.docx` marked bilingual by the toggle.
- [ ] `src-tauri/src/commands/library.rs`, `src-tauri/src/lib.rs` -- one dialog command with the entry filter; a classify command for drops -- one list for both entry points.
- [ ] `src/modes/libraryImport.ts`, `libraryImportOverlayState.ts`, `src/config/library.ts` -- tab state, classified file list, drop routing to Tệp, dispatch by tab and kind on submit; delete URL-screen state.
- [ ] `src/LibraryImportOverlay.vue` -- common block + tablist + three bodies + footer, drag frame, keys.
- [ ] `src/commands/index.ts`, `src/main.ts`, `src/modes/LibraryMode.vue`, `titlebarImportSource.ts` -- commands and invites; delete `LibraryUrlImportScreen.vue`.
- [ ] `BilingualImportPreviewOverlay.vue`, `bilingualImportPreviewState.ts` -- existing-Work notice + switch to new Work.
- [ ] `src/i18n/vi.json` -- new strings; remove dead keys.
- [ ] `tests/frontend/…` -- update the affected specs; add a matrix spec (mixed, two bilingual, unsupported, tab switch keeps content, existing-Work lock across tabs).
- [ ] `e2e/specs/story-6-25-import-entry.e2e.mjs`, `e2e/support/importForm.mjs` + the specs listed in Code Map -- the ticket's verify scenario. Drops go through the same `aura://file-dropped` listener that Rust feeds (a test seam, not a parallel path).

**Acceptance Criteria:**
- Given the overlay is open, when the user looks for a separate website button or screen, then none exists, and URL import is reachable only from the URL tab.
- Given 3 links on the URL tab, when the user reads the footer, then it shows `3 link` in the counter and a primary button labelled `Tải 3 link`.
- Given 2 `.txt` and 1 `.csv` dropped, when the user removes the `.csv`, then the primary button unlocks and opens the text preview.
- Given the user last used the Tệp tab in this session, when they reopen the overlay with Mod+N, then the Tệp tab is selected.
- Given the Library is empty, when the user clicks *chọn tệp*, then the overlay opens on the Tệp tab.

## Implementation Notes

- Rust verdict: `classify_import_paths(paths, docx_as_bilingual)` in `core/segment/import.rs`. Text wins a mixed batch (the bilingual rows are the rejected ones, `kind_mismatch`); with no text file, more than one bilingual file marks every bilingual row `one_bilingual_only`. Verdict priority: `has_unsupported` > `mixed_kinds` > `multiple_bilingual`. Each row also carries `is_docx`, so TS never reads an extension.
- One command, `library_pick_import_files(docx_as_bilingual)`, replaces the two dialog commands and returns the classification; `library_classify_import_paths` serves drops. `aura://file-drag-enter` now carries the file count for the drop frame.
- The primary button is one `dispatch('library.import.submit')`; `library.import_text/_file/_urls` and their deps are gone. The Name lock also applies on the URL tab (as the old URL screen did); Story 26 removes it everywhere.
- Row removal: `@mousedown`/`@focus` aims the row (`aimImportFile`), `library.import.file_remove` acts on it. The `.docx` toggle is `@change` (not a command).
- A `.docx` on the text path beside other `.txt` files still fails at Rust `import_files` (`BatchUnsupportedFormat`), surfaced as `lastError`, not as a row problem. Story 29 owns that.
- Bilingual + existing Work: the preview carries `destinationMode`, cannot confirm, and `import.preview.bilingual_use_new_work` closes it with the destination reset to Tác phẩm mới (the user presses Xem trước again).
- Tests: `libraryImportEntryMatrix.test.ts` (new), `support/importClassify.ts` is a double of the Rust verdict; e2e `story-6-25-import-entry.e2e.mjs` drops through `plugin:event|emit` on `aura://file-dropped`.
- e2e: the primary button sits in the footer outside `<form>`, so `form.$$('[data-import-preview-open]')` finds nothing; `story-12-6`, `6-23`, `6-24` were rewritten to the panel selector and `dropFiles`. Final run: 7 of 8 touched specs green; `story-6-24` `.txt path` stays red at `:focus-visible` (deferred, `Chủ: Epic 6`). 🔵 2026-11-15: harness cause found (the removed path field's WebDriver typing had supplied the `:focus-visible` that a script `focus()` inherits); the case now types into the preview's Name field first, green twice, red at the hint assert with the CSS rule removed.
- `library.import.close` is a no-op while `busy` (guard in the `main.ts` dep), keeping the old URL screen's rule that a running download cannot be dismissed.

## Plan Change Log

## Review Triage Log

Pass 1 (quick, loop 0): high 0 · medium 5 · low 7 · false 0 · maybe-false 1.

- medium · patch — Tab trap counts inactive tabs (tabindex -1, `button:not([disabled])` matches): Tab stops on all three tabs, roving tabindex undone.
- medium · patch — `library.import.close` and `.li-close` run while `busy`; the deleted URL screen ignored Esc and disabled Cancel during a download.
- low · patch — footer lacks `Huỷ` that EXPERIENCE §Đường nhập — màn nhập and the mockup place between Đích and the primary.
- low · reject — `classifySequence` race can drop a file when a second drop, `Bỏ` or toggle lands while the first classify IPC (a pure Rust function) is in flight; unlikely in use, fix adds state.
- medium · patch — keyboard `Bỏ` unmounts the focused button, focus falls to body outside `.li-scrim`, so Esc, Mod+Enter and the Tab trap stop.
- low · defer — a drop with the overlay closed fills the hidden file list with nothing shown; pre-existing (the old drop path did the same), only the tab switch is new.
- low · patch — `aria-controls` on the two unselected tabs names panels that `v-if` did not render.
- low · patch — `BilingualImportPreviewOverlay.vue:348,363` falls back to `[data-bilingual-import-preview-open]`, which no longer exists.
- medium · patch — matrix test for bilingual + existing Work: mock preview is null so status is not `loaded`; the CanConfirm assert holds on both branches, and the notice/button are never rendered.
- low · patch — matrix test header claims the TS double is checked against the Rust verdict; nothing compares them.
- low · patch — edited comment lines in Vietnamese / with a story id (`lib.rs:237`, `main.ts` ~636-648).
- low · patch — stale comments naming removed commands and deps (`commands/index.ts:211-215`, `libraryImport.ts:13-17,668,675`, `importPreviewState.ts:1413,1557`).
- maybe-false (medium if true) · defer — e2e `story-6-24` `.txt path` case red at `:focus-visible` (line 266), twice on one binary; probe: the driver's `keys('Tab')` arrives with `isTrusted: false` and focus stays on `.ip-panel`; focusing a preview text field first did not turn it green. Last green nightly 7e6b6b1e predates 3f42865b, which already broke the case's setup; settle by running the case on 7e6b6b1e with the probe, or in the real app by keyboard.

## Design Notes

Classify in Rust and render in TS. The drop listener and the dialog both end in the same `ClassifiedFile[]` setter, so they cannot drift. Today the drop path has no filter at all, which is exactly that drift.

## Verification

**Commands:**
- `npm run build && cargo test --lib core::segment::import` -- the classify tests pass.
- `npx vitest run tests/frontend/libraryImportOverlay*.spec.ts tests/frontend/importPreview*.spec.ts` -- passes.
- `npm run check:i18n && npm run check:tokens && npm run check:commands` -- passes.
- Full suite once, because `lib.rs` command registration changed.
- `npm run test:e2e -- --spec e2e/specs/story-6-25-import-entry.e2e.mjs`, outside the sandbox, after rebuilding the binary with `--features wdio`.

**Manual checks:**
- In the real app, drag a Finder file onto the overlay: the frame appears and the file lands on the Tệp tab. This is a debt item `Chủ: Epic 6`.
