---
title: 'Story 6.26: Tên Tác phẩm tự điền, màn xem trước điền sẵn Tên · Ngôn ngữ · Thể loại'
type: 'feature'
ticket: '26'
created: '2026-11-15'
status: 'built'
baseline_revision: '5741625933558137430f24834e7d97b96316ed56'
route: 'full'
route_source: 'auto'
risk: 'low'
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

**Problem:** The import overlay refuses to go on until the user types a Work name, although the file name or the first pasted line already says it. The bilingual preview shows Name and Source language read-only and has no Genre; the text preview has no Genre either. So a wrong field means closing the preview and starting over (FR13, FR122).

**Approach:** Per EXPERIENCE.md §Đường nhập — màn nhập and the mockup: an empty Name field shows an auto-filled name that follows the active tab until the user types; an empty Name no longer locks the overlay's primary button; both previews let the user edit Name, Source language and Genre, prefilled from the overlay, and keep `Nhập` locked while Name is blank.

## Boundaries & Constraints

**Always:**
- Auto name, new-Work destination only: Tệp = stem of the first listed file; Dán = first non-blank line, trimmed, not truncated; URL = empty in the overlay (placeholder *Để trống: lấy tiêu đề bài đầu tiên sau khi tải*), filled in the preview from the first Chapter title once it loads, if Name is still blank.
- The overlay field displays the auto name live, styled as auto with the mockup's hint (*Tự điền từ dòng đầu. Gõ đè thì app không đổi lại nữa.* / *Tự điền từ tên tệp đầu tiên.*). A typed name is never overwritten; clearing the field brings the auto name back.
- The file stem comes from Rust on the classified row (AD-1); TS never parses an extension.
- An existing-Work destination keeps the Work's values, read-only, in the overlay and both previews. No auto-fill runs.
- Changing Source language in the bilingual preview rebuilds it (no `auto` there, as today).
- Both previews lock `Nhập` while a new Work's Name is blank.

**Never:**
- No change to Rust confirm commands; they already take name, lang and genre.
- No "Thêm Chương…" opener (Story 27), no bilingual import into an existing Work (Story 28), no `.docx` detection (Story 29).
- No `localStorage`. No new dependency.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected | Error handling |
|---|---|---|---|
| File, no name | Tệp: `tuyet-trung.txt` | field shows `tuyet-trung` (auto); preview Name `tuyet-trung`; Work created with that name | — |
| Typed first | user types `Tuyết Trung`, then adds files | field keeps `Tuyết Trung` | — |
| Paste | Dán: `\n\n  Đệ nhất chương\nnội dung` | auto `Đệ nhất chương` | — |
| Follows source | auto active; user removes first file / switches tab | auto updates to the new first file / the new tab's source | — |
| URL | URL tab, Name blank, 3 links | button unlocked; preview Name = first Chapter title | title null: Name blank, `Nhập` locked |
| Existing Work | Đưa vào = Kiếm Lai | no auto; fields show Kiếm Lai's values, disabled | — |
| Preview genre | change Thể loại in text or `.csv` preview | created Work carries the new genre | — |
| Bilingual lang | `.csv` preview, switch Tiếng Trung → Tiếng Anh | preview rebuilds with the new lang | rebuild error shown as today |
| Blank in preview | user clears Name in either preview | `Nhập` locked | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/segment/import.rs:1114` -- `ClassifiedImportPath`: add `stem: String` (file stem of the path), filled in `classify_import_paths`; extend its unit tests. Map it in the wire struct in `src-tauri/src/commands/library.rs` (`library_pick_import_files`, `library_classify_import_paths`); TS type in `src/config/library.ts`; double in `tests/frontend/support/importClassify.ts`.
- `src/modes/libraryImport.ts` -- `name` :113, `effectiveName` :177, `destinationMode` :139. Add a computed auto name from the active tab (first classified file's `stem`, or first non-blank line of the pasted text) and fold it into `effectiveName` when the typed name is blank and the destination is new. `importSubmitReady` :757-767: drop the three `effectiveName…trim() !== ''` clauses.
- `src/LibraryImportOverlay.vue` -- Name input :279 (disabled copy :280): show typed name or auto name, auto style + hint, URL placeholder. `onSubmitChord` :111 and `:disabled` :470 read `importSubmitReady` (no change needed beyond it).
- `src/importPreviewState.ts` -- `pendingName` :149, `pendingGenre`, `setImportPreviewPendingName` :560, `importPreviewCanConfirm` :543 (name lock already there). Add `setImportPreviewPendingGenre` (no-op on existing Work). URL branch: where `preview.value = result.batch.encoding_preview` is set (:1548/:1685/:1720), fill `pendingName` from `importPreviewSelectedChapters` :715 `chapters[0].title` when blank and new.
- `src/ImportPreviewOverlay.vue` -- summary `<dl class="ip-summary">` :1312; Name :1322-1344 and `SourceLangChips` :1348 already editable. Add a Genre row: `SelectMenu variant="field"` with `genreChoices()` (`src/modes/libraryWorkCard.ts:64`), read-only `genreLabel()` for an existing Work.
- `src/bilingualImportPreviewState.ts` -- pending fields :57-62 set in `openBilingualImportPreview` :375, exported read-only :129. Add setters for name, lang (calls `rebuildBilingualImportPreview` :318 with the new lang) and genre; `bilingualImportPreviewCanConfirm` :151 gains the blank-name lock for a new Work. `confirmBilingualImportPreview` :660 already passes `pendingGenre`.
- `src/BilingualImportPreviewOverlay.vue` -- read-only name :420-423 and lang :425-428 (`sourceLangLabel` :373): replace with the text preview's Name field shape, `SourceLangChips :include-auto="false"`, Genre `SelectMenu`; keep read-only rows when `pendingDestinationMode` is existing.
- `src/i18n/vi.json` (`mode.library.*`, `import.preview.*`) -- the two hints, the URL placeholder, a Genre label for the previews if none fits.
- Tests to update: `tests/frontend/importPreviewDestination.test.ts:781` (asserts the empty-name lock on all three tabs; now the opposite), :184, :815; `importPreviewFooterLayout.test.ts:160-215`; `importPreviewBilingual.test.ts`; `libraryImportEntryMatrix.test.ts`.
- e2e: `e2e/support/importForm.mjs` has `selectImportTab` :134, `openUrlImportTab` :143, `dropFiles` :155; `e2e/specs/story-6-25-import-entry.e2e.mjs` writes fixtures to `mkdtempSync` (:43-49). No checked-in fixtures; create `tuyet-trung.txt` and a `.csv` the same way.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/segment/import.rs`, `src-tauri/src/commands/library.rs` -- `stem` on each classified row, unit test incl. a dotted name (`a.b.txt` → `a.b`) -- AD-1.
- [ ] `src/config/library.ts`, `src/modes/libraryImport.ts`, `src/LibraryImportOverlay.vue` -- auto name, unlock, field display -- the overlay half of the story.
- [ ] `src/importPreviewState.ts`, `src/ImportPreviewOverlay.vue` -- Genre editable; URL name from first Chapter title.
- [ ] `src/bilingualImportPreviewState.ts`, `src/BilingualImportPreviewOverlay.vue` -- Name, Lang, Genre editable; blank-name lock.
- [ ] `src/i18n/vi.json` -- new strings.
- [ ] `tests/frontend/…` -- update the specs listed in Code Map; cover every matrix row except the e2e ones (fold into existing files where they fit).
- [ ] `e2e/specs/story-6-26-name-autofill.e2e.mjs` -- the ticket's verify scenario: drop `tuyet-trung.txt`, no typed name, preview Name `tuyet-trung`, `Nhập` creates a Work named so; a typed name survives; genre changed in the preview, also for a `.csv`, is the created Work's genre.

**Acceptance Criteria:**
- Given the Tệp tab with `tuyet-trung.txt` and an empty Name, when the user presses `Mod+Enter`, then the preview opens with Name `tuyet-trung`.
- Given the URL tab with links and an empty Name, when the user reads the footer, then `Tải N link` is enabled.
- Given the bilingual preview of a `.csv` into a new Work, when the user edits Name, Source language and Genre, then the created Work carries all three edited values.

## Implementation Notes

- Auto name is one computed, `importAutoName`, folded into `effectiveName` only when the typed name is blank (whitespace counts as blank) and the destination is new; `name` keeps only what the user typed, so the submit paths are unchanged.
- The overlay field is `:value` + `@input`, not `v-model`: emptying it restores the auto name on `nextTick`. Typing into the shown auto text edits it as a prefilled value (the mockup renders it as field text).
- URL: `fillBlankNameFromFirstChapterTitle` runs on open and after every batch rebuild, so a Name cleared in the preview is refilled by the next rebuild.
- Bilingual language change reverts the label only on `error`; an overtaken refresh leaves the newer one in charge.
- e2e: the driver's `setValue('')` fires no `input` on the Name field, so the spec dispatches `input` from the page; genre `SelectMenu` opens by `realClick` on `summary`, then `.select-item`. Spec green 3 of 3 before the review patches (comment and revert-condition only afterwards).
- Counter-check: removing the auto-name fallback in `effectiveName` turns the two overlay cases red (`'' ≠ 'tuyet-trung'`).

## Plan Change Log

## Review Triage Log

Pass 1 (quick, loop 0): high 0 · medium 0 · low 4 · false 5 · maybe-false 0.

- low · patch — story ids in the new e2e header comment and in two changed vitest `describe` names break the AGENTS.md comment rule; the e2e `describe` keeps the file-name convention of `story-6-25`.
- low · patch — `setBilingualImportPreviewSourceLang` reverts on any non-`loaded` status, so a refresh overtaken by a newer one (`mySequence !== sequence`, status still `loading`) restores the old label while the newer rebuild used the new language; revert only on `error`.
- low · patch — doc comment on `setBilingualImportPreviewSourceLang` restates the body.
- low · patch — no vitest case presses `Mod+Enter` with a blank Name on Tệp (first AC); the button path is covered, the chord path is not.
- false — label reverts after a failed rebuild while content "disagrees": on `error` the overlay renders the error message (`BilingualImportPreviewOverlay.vue:425`), no rows.
- false — `libraryImportEntryMatrix.test.ts` untouched: nothing in it depends on the removed lock (it sets a typed name), and the matrix rows are covered in the other files.
- false — tab-switch/paste-change rows untested: `importPreviewDestination.test.ts` moves paste → files → urls and asserts the auto name at each step, and removing the first file.
- false — caret typing appends to the shown auto name: the mockup renders the auto value as field text (`import-entry.html:127-128`), so editing a prefilled value is the specified behaviour; a typed name is never overwritten.
- false — an unsupported first file supplies the auto name: the verdict locks the button, and after `Bỏ` the name follows the next file.

## Design Notes

One auto-name computed, folded into `effectiveName`, keeps the submit paths untouched: every `openImportPreviewFrom…` already receives `effectiveName`. Storing the auto value into `name` instead would make "typed vs auto" unknowable and break the never-overwrite rule.

## Verification

**Commands:**
- `npm run build && cargo test --lib core::segment::import` -- classify tests pass.
- `npx vitest run tests/frontend/importPreviewDestination.test.ts tests/frontend/importPreviewFooterLayout.test.ts tests/frontend/importPreviewBilingual.test.ts tests/frontend/libraryImportEntryMatrix.test.ts` -- pass.
- `npm run check:i18n && npm run check:tokens && npm run check:commands` -- pass.
- `npm run test:e2e -- --spec e2e/specs/story-6-26-name-autofill.e2e.mjs`, outside the sandbox, after rebuilding with `--features wdio`.

**Manual checks:**
- None beyond e2e; no new real-app-only behaviour (Story 25's drag-from-Finder debt already covers the drop path).
