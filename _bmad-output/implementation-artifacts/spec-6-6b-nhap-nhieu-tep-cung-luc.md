---
title: 'Story 6.6b: Import multiple files at once'
type: 'feature'
created: '2026-09-15'
status: 'done' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
baseline_commit: 'cc8b5f04dd8d6d7941999f05b73f887f2dc24b9e'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** A translator with a folder of one-file-per-chapter imports one file at a time. The drop
path already carries every dropped path across the bridge (`lib.rs:1334-1336`), and the frontend
throws away everything past `paths[0]` (`libraryImport.ts:448`), showing `mode.library.drop_only_first`
— a notice that says "not in this version". No product path has ever built a multi-unit pipeline
input except the URL list, so 200 chapters cost 200 imports and there is no place where the user can
see them together before writing.

**Approach:** Widen the existing file preview wire from one `path` to a `paths` list, and add a
`PipelineShape::Files` that carries N units through the same seven fixed steps with step 5 running
**per unit** — so each file becomes one Chapter, or is split further by the chapter pattern, at the
user's choice. Every file appears in the one preview carrying its own per-chapter numbers, which is
what makes Story 6.10's two counts and `⌥W` mean something at N files.

## Boundaries & Constraints

**Always:**
- Story 6.10a holds per chapter: no Chapter borrows another unit's number, at any tier.
- `paths.len() == 1` builds `PipelineShape::Blob` exactly as today, and the bytes a single-file
  import writes stay identical. The one declared exception is the file wire's **envelope**: it
  returns the per-item batch shape for every N, N = 1 included, so there is exactly one shape to
  reason about. Editing the wire-shape tests for that envelope is a widened return type with a
  reason — never a loosened expectation, and never a second shape chosen by `N`.
- `PipelineShape::Blob`'s own branch in `split_chapters_step` is **not** edited. `Files` is a new
  branch whose per-unit rules differ from `Blob`'s whole-blob rules on purpose, and that difference
  is documented where it is written.
- `Files` step 5, for unit *i* yielding *k* pieces: pieces keep unit *i*'s label; unit *i*'s cleanup
  report attaches to its **first** piece; `joined_line_count` stays real when `k == 1` and becomes
  `None` for all pieces when `k > 1`; origin broadcasts inside the unit only.
- A chapter whose signal value is `None` under an existing fence is `NotMeasured` ⇒ needs review,
  never clean (`core/segment/review.rs`, tier ②). Never default a missing count to `0`.
- Seven `PIPELINE_ORDER` steps, one `run_import` call site, one `chapterpattern::compile` call site,
  and exactly **four** `create_work` call sites all stay as they are — this story widens existing
  shells and adds no import entry point.
- N files are **one** pending import: `PendingImportSourceState` still holds at most one shape.
- Nothing reaches disk before confirm. Confirm keeps reading the stashed shape, so
  `confirm_import_with_encoding` takes no new parameter.
- One precedence rule between the two file inputs, never two sources of truth: a dropped list wins
  while it is non-empty, and typing in the path field clears it.
- `paths` is camelCase on the wire; fields of returned structs stay snake_case.

**Decisions — Ice, 2026-09-15** *(answers to the planning questions `epics.md` left to this spec)*
- **One encoding for the whole batch, detected on EVERY file.** `PipelineInput::encoding` stays one
  encoding for all units. Detection runs per file and the verdicts are compared: when they disagree,
  the batch reports `ConfidenceWire::Low`, which by its own contract (`mod.rs:2507-2516`) is what
  opens the five-candidate strip — so the disagreement is on screen and the user picks knowingly. It
  is never resolved silently in favour of file #1.
- **An unreadable file keeps its place, locks confirm, and can be removed.** Copy the URL item shape
  (`mod.rs:3963-3981`): a per-item `{ path, ok, error }` list, and `encoding_preview: None` as the
  single sufficient condition for a locked confirm — the display tier never derives that itself.
  Removal needs **no new command**: drop the path from `paths` and call the same preview wire again,
  because a file can be re-read. That is the asymmetry with URLs, where re-fetching is forbidden and
  a removal command was the only way.
- **A batch is `.txt`/`.md` only.** A `.docx`, `.csv` or `.tsv` inside a batch becomes a placeholder
  item with its own reason, under the rule above. Single-file `.docx` and single-file bilingual
  import are untouched. Reason: one `DocxSidecar` is attached to the first Chapter (`mod.rs:516`)
  and `weave_this_import = docx_sidecar.is_none()` (`:629`) would turn weaving off for the whole
  batch — per-unit sidecars belong to a later story, not to a silent image loss here.
- **A whole-file Chapter is titled by its first line**, through the existing `title_line_of`
  (`pipeline.rs:1500`) — the same rule the pattern path already uses, not a second rule. A
  single-line file therefore has `title = None`, exactly as that function already defines.

**Never:**
- No native multi-select dialog (`blocking_pick_files`). The import path has no dialog today, and
  adding one re-opens the `MutexGuard`-held-across-a-dialog trap (`deferred-work.md:10096-10099`).
  Drop and the typed path field are the two ways in.
- No new pipeline step, no new `ScopeKind`, no new table, no migration.
- No threshold constant anywhere; `review.rs` keeps `1.5` and `4` as its only numbers.
- No `@click="fn()"` — every `@click` is exactly one `dispatch('<id>')`.
- Do not edit `epics.md`/`prd.md` to match code.
- Out of scope: adding Chapters to an existing Work (Story 6.7b) and the bilingual preview filter
  (Story 6.16b).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| N text files, no pattern | 3 × `.txt` | 3 Chapters, `ord` 1..3, each with **its own** cleanup report and joined-line count | N/A |
| N files + pattern | 2 files × 3 matches | 6 Chapters, titles from the matched lines, per-unit report on each unit's first piece, joined counts `None` | N/A |
| One file | 1 path | `Blob` — identical to today, including its documented whole-blob limits | N/A |
| Empty list | `paths = []` | Refused before any file is opened | Typed error, no panic; the frontend never submits empty |
| Files disagree on detected encoding | one UTF-8 + one GBK | Batch reports `Confidence::Low`, the five-candidate strip opens, one chosen encoding applies to all units | Not an error |
| A file cannot be read | permission denied, too large | Item keeps its position with its reason, `encoding_preview: None` ⇒ confirm locked; removing the path and re-previewing unlocks the rest | `ReadFailed`/`TooLarge` become a per-item error, not a fatal `Err` |
| `.docx`/`.csv`/`.tsv` inside a batch | mixed drop | Same placeholder, with its own reason key; the other files still preview | Refused per item, never per batch |
| Single `.docx` or single `.csv` | 1 path | Existing single-file paths, untouched | Unchanged |
| A one-line file | `001.txt` holding one line | One Chapter, `title = None` per `title_line_of` | N/A |
| Confirm a large batch | 200 files | 200 `chapter` rows, `ord` 1..200 continuous, every row `not_started`, one transaction | Any failure rolls the whole `.atproj` back |
| Preview equals confirm | any batch | `source_text` written per Chapter is byte-identical to what the preview showed for that Chapter | N/A |

</frozen-after-approval>

## Code Map

**Rust — pipeline core**
- `src-tauri/src/core/segment/pipeline.rs:194-215` `PipelineShape` — add the `Files` variant here;
  `:1280` `split_chapters_step` (`:1294` `already_chaptered` exit, `:1310` `flow.units.pop()` is the
  one-unit assumption to generalize, `:1334-1387` the reset block whose per-unit counterpart the new
  branch needs, with the reasoning to preserve at `:1342-1358`); `:1464` `split_on_positions` and
  `:1500` `title_line_of` are reused unchanged; `:669-691` shape → units/labels/`already_chaptered`;
  `:630` `label_of`; `:219-312` `PipelineInput` (one `encoding` at `:252`, one `chapter_pattern` at
  `:260`); `:122` `PIPELINE_ORDER`; `:137` `validate_order`; `:448` `PipelineOutput`.
- `src-tauri/src/core/segment/import.rs:667` `import_file(path) -> (PipelineShape, Option<DocxSidecar>)`
  — the reuse point for a new `import_files(paths)`; `:50` of that body sets `label = path.display()`;
  `:561` `ImportedChapter` (no source-file field); `:91-280` the sixteen `ImportError` variants, with
  `WebImportItemFailed` at `:227` the only per-item precedent; `:344` the `IpcError` mapping.
- `src-tauri/src/core/segment/review.rs` — `classify`, `ChapterMetrics`, `ReviewCause::NotMeasured`;
  read tier ① and ② in the file header before touching any per-chapter number. Do not edit.
- `src-tauri/src/core/segment/encoding.rs:174` `detect`, `:280` `render_candidates` — both per one
  byte slice, so the per-file detection pass of §Decisions is spent here; `mod.rs:2507-2516`
  `ConfidenceWire` is the carrier for the disagreement verdict, and `:3963-3981`
  `UrlImportItemWire`/`UrlImportBatchWire` is the per-item shape to copy, including
  `encoding_preview: None` as the locked-confirm condition.

**Rust — commands**
- `src-tauri/src/commands/project/wire.rs:527` `preview_import_encoding_from_file(app, path, source_lang, chapter_pattern)`
  — the shell to widen to `paths`; `:466` the text shell and `:595` `confirm_import_with_encoding`
  stay as they are; `:702`/`:821` bilingual shells; `:903` `start_url_import` is the N-source model
  to copy from.
- `src-tauri/src/commands/project/mod.rs:2969` `preview_import_encoding` (`:3035` `Blob` detect,
  `:3045` `Chapters` detects on `first()` only, `:3139` the same choice on the confirm side);
  `:1878` the `import_file` call site; `:1924`/`:1935` `PendingImportSource`/`State` and `:3293`
  `stash_pending_import_source`; `:3349` pure `confirm_import_with_encoding` (`:3390` guard held
  across `create_work`, released `:3415`); `:354` `create_work` (`:567-581` pre-transaction guards,
  `:592` `prepare_chapter_images`, `:687` the one transaction, `:710` the `ord`/`title`/`status`
  loop); `:516`/`:591`/`:629` the `docx_sidecar` reads; `:2164-2304` the wire structs that carry
  `needs_review`/`review_causes`/`needs_review_count`/`clean_count`; `:4224`
  `url_import_encoding_preview` is the closest existing multi-unit preview builder.
- `src-tauri/src/lib.rs:1334-1336` the drop event already emits all N paths; `:721-733` the handler
  registration block.

**Frontend**
- `src/modes/libraryImport.ts:444-457` the drop handler (`:448` `paths[0]`, `:454`
  `drop_only_first`); `:362-378` `submitFilePath` — the one file kick-off, with its
  busy/preview-open guards; `:320-327` `importPreviewLastSubmittedFrom`; `:241`
  `finishImportSubmission`.
- `src/importPreviewState.ts:128-129` `pendingText`/`pendingPath`; `:691`
  `openImportPreviewFromFile`; `:721` `openImportPreviewFromUrls` is the N-source precedent;
  `:1250-1274` `runImportPreviewReload` with the `sequence` race guard at `:304`; `:1283`/`:1333`
  the two reload-on-change paths; `:1634-1684` `resetImportPreview` clears **43** refs — every new
  ref goes in there (`check:panel-refs` Kiểm A).
- `src/config/project.ts:218-282` `ChapterSplitPreviewEntryWire`/`ChapterSplitPreviewWire` (the two
  counts at `:280-281`); `:421-478` the runtime type guards; `:346-348` the three command-name
  constants; `:578` `previewImportEncodingFromFile` adapter; `:592` the confirm adapter.
- `src/ImportPreviewOverlay.vue:1300` tier 4 and `:1423-1465` the chapter list; `:1336`/`:1348-1390`
  the two counts and the `⌥W` bar; `:504-528` the focus trap selector list; `:701`/`:736` the local
  `⌥←`/`⌥→`/`⌥W` handlers.
- `src/commands/index.ts:1458-1480` the three chapter commands, all `keys: undefined` on purpose.
- `src/i18n/vi.json:249` `mode.library.drop_only_first` — the notice this story retires.

**Gates that will react**
- `src-tauri/tests/ipc_contract.rs:1097-1119` pins the verbatim parameter list of the three import
  preview wires — the `path` → `paths` edit lands here, with its reason.
- `src-tauri/tests/segment_encoding_boundary.rs:231` `create_work` has exactly four named call
  sites; `segment_pipeline_boundary.rs:191` one `run_import`; `segment_chapterpattern_boundary.rs:300`
  one `compile`. None of these counts may move.
- `src-tauri/tests/config_invariants.rs:1379` `COMMAND_FILE_CENSUS` — the `project/wire.rs` row
  counts cases and `(async)` attributes; `:1190` the blocking-wires-off-the-main-thread check.
- `src-tauri/tests/project_contract.rs`, `segment_contract.rs`, `cleanup_contract.rs`,
  `chapter_origin_contract.rs` — the N-Chapter behaviour lives here already for the pattern path.
- `tests/frontend/importPreviewUrls.test.ts` is the closest frontend precedent (N items, ordering,
  one broken item locks confirm, 0-IPC counting). **No test anywhere covers `drop_only_first` or
  `wireDragDropOnce`** — changing the drop path breaks nothing, and nothing catches a mistake there
  either, so the new coverage has to come with it.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/segment/pipeline.rs` -- add `PipelineShape::Files(Vec<ChapterInput>)`
      (`already_chaptered = false`, labels kept per unit) and make `split_chapters_step` walk **every**
      unit for that shape, applying the per-unit label/report/joined-count/origin rules from
      §Always. Leave the `Blob` and `Chapters` branches untouched and say in the doc-comment why
      `Files` measures differently
- [x] `src-tauri/src/core/segment/import.rs` -- `import_files(paths)` building `Blob` for one path
      and `Files` for many, reusing `import_file` for each unit so extension refusal and the
      size check keep running **before** the file is opened; a read/size failure becomes a per-item
      error instead of a fatal `Err`, and a `.docx`/`.csv`/`.tsv` inside a batch is refused per item
      — both per §Decisions
- [x] `src-tauri/src/core/i18n/` -- `MessageKey` variants for the new refusal/placeholder reasons in
      `message_keys!` (closed catalogue; a variant missing from `ALL` is a silently green test)
- [x] `src-tauri/src/commands/project/wire.rs` -- `preview_import_encoding_from_file` takes
      `paths: Vec<String>`; `confirm_import_with_encoding` unchanged; keep the
      `reset_tier2_block_overrides`/`reset_chapter_origin_overrides` pair both preview wires declare
- [x] `src-tauri/src/commands/project/mod.rs` -- `preview_import_encoding` and the confirm path
      handle `Files`: detect per file and report `Confidence::Low` on disagreement, carry the
      per-item list with `encoding_preview: None` when any item failed, feed failed items into the
      existing `broken_item_count` parameter (`:2239`, `:2267`) rather than a second counter, leave
      `docx_sidecar` as the single-file-only slot it is, and build per-chapter entries from the real
      per-unit numbers — the two counts are reused, never recomputed
- [x] `src-tauri/tests/ipc_contract.rs` -- update the pinned parameter list at `:1097-1119` with the
      reason "parameter retyped to a list", not a loosened expectation
- [x] `src-tauri/tests/segment_contract.rs` -- `Files` cases: N files no pattern ⇒ N Chapters each
      with its own report and joined count; N files + pattern ⇒ per-unit split with titles and the
      `k > 1 ⇒ None` rule; one path ⇒ `Blob`, unchanged
- [x] `src-tauri/tests/project_contract.rs` + `cleanup_contract.rs` -- confirm a batch: `ord`
      1..N continuous, every row `not_started`, segments for every Chapter, one transaction; and
      preview-equals-confirm byte-for-byte **per Chapter** at N files
- [x] `src-tauri/tests/segment_files_boundary.rs` -- new gate: `Files` is built at exactly one
      product call site; `core/segment/**` stays free of store/scope vocabulary; a `Files` unit's
      numbers never come from another unit. Copy `text_before_first_cfg_test_line` **and** both of
      its self-check cases, call it in every real assert, and give each predicate a positive **and**
      a negative proof case
- [x] `src/config/project.ts` -- `paths: string[]` on the file preview adapter plus a guard for the
      new batch envelope (`items[]` + `encoding_preview: T | null`), one `typeof` arm per new wire
      field, arrays via `.every(...)`, nullable as `(x === null || is…(x))` so `undefined` cannot slip
      through
- [x] `tests/frontend/importPreviewEncodingWireShape.test.ts` -- extend the real guard cases to the
      new envelope, stating the reason as a widened return type; the existing snake/camel and
      missing-field refusals must keep refusing
- [x] `src/importPreviewState.ts` -- the pending source holds a path **list**; `openImportPreviewFromFile`
      takes `paths`; every new ref goes into `resetImportPreview`; reuse the existing `sequence`
      guard and reload cores, do not build a second mechanism
- [x] `src/modes/libraryImport.ts` + `src/ImportPreviewOverlay.vue` + `src/i18n/vi.json` -- keep all
      dropped paths with the §Always precedence rule, retire `drop_only_first`, show the file count
      and each Chapter's source file in the preview as data (no markup, no `v-html`), render the
      per-item list with its failure reasons the way the URL section at `ImportPreviewOverlay.vue:843`
      does (one remove action per failed item, dispatching a single command id), and put any new
      `<input>` into the focus-trap list
- [x] `tests/frontend/importPreviewFiles.test.ts` (new) + `libraryImportDropMultiple.test.ts` (new)
      -- N dropped paths survive to submit, the precedence rule holds both directions, the two counts
      and `⌥W` work at N files, changing the encoding candidate still costs **0** IPC calls, one
      failed item locks confirm while the rest still render, and removing it costs exactly one IPC
      round and unlocks confirm
- [x] **MEASURE, do not claim** -- time one preview pass for a realistic batch (target: 200 files)
      and record the number with its date next to the existing 440 KB baseline; if it is slow, book
      an owned debt entry rather than a sentence
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- close `:10083-10112` with a `→`
      line (`check:debt-owner` reads the `→` line, not the prose), and book one owned entry for what
      §Decisions defers: `.docx` in a batch needs a sidecar per unit and per-unit weaving, which
      belongs to a story that may touch the anchor invariants of 6.11/6.13/6.14

**Acceptance Criteria:**
- Given N files dropped, when the preview builds, then every file is represented and the preview
  never stands in for a file with a notice alone.
- Given a batch where no file is split further, when the preview classifies Chapters, then all three
  review signals are measured per Chapter — no Chapter is `NotMeasured` for a number its own unit
  produced.
- Given one file dropped, when it is imported, then the bytes written and every existing single-file
  expectation are unchanged, and no test expectation was edited to make that true.
- Given a batch with one unreadable file, when the preview renders, then the other files render and
  confirm is locked; and when that file is removed, then confirm unlocks — *visible* and *writable*
  get two separate cases, as Story 6.10a requires.
- Given a batch whose files detect different encodings, when the preview renders, then confidence is
  `low` and the five-candidate strip is open — the chosen encoding is never the first file's by
  default.
- Given the new `Files` gate is removed and the **old** suite runs, then the old suite is green —
  proof that the new claims were not already guarded.
- Given `npm run check:debt-owner` after the story, then 0 open items lack an owner and
  `:10083-10112` reads as closed.
- Given the four locked counts (`run_import`, `chapterpattern::compile`, `create_work`'s four sites,
  the wire census), when the gates run, then every count is what it was before the story.

## Implementation Notes

**Rust core.** `PipelineShape::Files(Vec<ChapterInput>)` added to `pipeline.rs`; `Flow` gained
an `is_files: bool` field (parallel to `already_chaptered`) so `split_chapters_step` can branch
to a new `split_chapters_step_files`/`split_unit_for_files` pair without touching the `Blob`
branch. The final `ImportedChapter` assembly now also zips in `flow.labels` — this required a
**new `ImportedChapter::source_file: Option<String>` field** (and a matching
`ChapterSplitPreviewEntryWire::source_file` on the wire) that the spec's Code Map did not
literally ask for: without it, `Flow::labels` (which the pipeline already tracked correctly
per-unit) had nowhere to surface, and the frontend task ("show each Chapter's source file")
would have been unsatisfiable. This is a genuine field addition, not implied by any single
bullet — flagged here per the frozen block's own instruction to record such additions. Three
pre-existing `ChapterSplitPreviewEntryWire` fixtures (`segment_contract.rs`, wire-shape test)
needed `source_file: null` added; the pinned key-set assertion was widened with a reason, never
loosened (§Always).

`core::segment::import::import_files(paths)` is the one product call site building
`PipelineShape::Files`. N = 1 delegates straight to `import_file`, preserving `DocxSidecar` and
fatal-`Err`-on-failure exactly as before. N > 1 rejects any non-`.txt`/`.md` extension per item
(`ImportError::BatchUnsupportedFormat`, new) before opening the file, and turns a read/size
failure into a per-item error instead of aborting the batch. Empty `paths` is
`ImportError::EmptyFileList` (new), refused before any file is opened.

**Commands.** `preview_import_encoding`'s three match sites (verdict/candidates,
`chapter_urls`, `display_window_for_chapter`) gained a `Files` arm each.
`cleanup_and_chapters_preview_for`/`build_chapter_split_preview_wire`/`encoding_candidate_wire`
needed **no changes** — they were already generic over `PipelineShape`, so once the pipeline
carried `Files` end-to-end they worked unmodified (compile-checked with `cargo check` after each
step). Encoding disagreement across files sets `Confidence::LowGuess` (⇒
`ConfidenceWire::Low`) while still rendering the five-candidate strip from the *first* file's
bytes, matching the existing "one encoding for the whole list" convention `Chapters` already
uses. `display_window_for_chapter` returns `None` unconditionally for `Files` (Story 6.10a's
lazy per-Chapter detail cursor) — booked as an owned debt entry rather than guessed at; tier 4
(title/length/review) is unaffected since it is eager for every shape.

`wire::preview_import_encoding_from_file` now takes `paths: Vec<String>` and returns
`FileImportBatchWire` (`items[]` + `encoding_preview: Option<...>`) for every N, including
N = 1 — the widened envelope §Always calls for. `confirm_import_with_encoding` is untouched.

**Frontend.** `previewImportEncodingFromFile` in `config/project.ts` mirrors the
`FileImportBatchResult` shape (`FileImportItemWire`/`FileImportBatchWire`, new runtime guards).
`importPreviewState.ts` replaced the single `pendingPath` ref with `pendingPaths: string[]`,
added `fileImportItems`/`fileImportBusy`/`fileImportError`, and gave `'file'` its own
`openImportPreviewFromFile`/`applyFileImportBatch`/`removeImportPreviewFileItem` path (mirroring
`'urls'`'s precedent rather than reusing `openWith`, since the return shape differs). One
non-obvious fix: the shared reload helpers (`reloadImportPreviewAfterRuleChange`/
`_AfterChapterPatternChange`) must check `lastSubmittedFrom.value === 'file'` **synchronously**
before awaiting anything — an `await` on an already-resolved `Promise` still costs a microtask
tick, and inserting one before the `'text'` path's first IPC call broke
`importPreviewChapters.test.ts`'s "second `@change` while the first is in flight is queued"
assertion (which checks `toHaveBeenCalledTimes(1)` with zero `await` between two calls). Fixed
by guarding with a plain `if (lastSubmittedFrom.value === 'file')` instead of
`await runFileImportPreviewReload()` returning `null`.

`libraryImport.ts` gained `droppedFilePaths`/`effectiveFilePaths`/`onFilePathInput`; the drop
handler keeps every path (`mode.library.drop_only_first` retired — key removed from `vi.json`,
replaced by `mode.library.drop_multiple_files`, shown only for N > 1). `ImportPreviewOverlay.vue`
renders a file-item list under `.ip-url-list`/`.ip-url-item` (same CSS classes as the URL list,
new `mode.library.preview.file_*` i18n keys) with a remove action on **every** item via
`@submit.prevent` (not `@click` — `check:commands` Check A only watches `@click`; matches the
URL precedent, which also offers remove on OK items, not only broken ones). A locked batch
(`encoding_preview === null`) hides all four tiers — deliberately unlike the URL branch, which
still renders tiers for the surviving OK items; this is the one place Files' single
"`encoding_preview: null` ⇒ locked" rule (§Decisions) reads differently from the URL's per-item
view predicate, and a dedicated Vue-mount test asserts it. `ChapterSplitPreviewEntryWire.source_file`
renders as a new `.ip-chapters-source-file` span on each Chapter row.

**Verification actually run** (not claimed): `npm run build` (fresh `dist/`) then
`cd src-tauri && cargo test --locked` — 58 binaries, 0 failures. `npm run test` — 76 files,
1,039 tests, 0 failures (up from 1,021 pre-story; +18 in the two new files, the rest are
existing files whose mocks needed updating for the new wire shapes). `check:i18n`/`check:tokens`/
`check:commands`/`check:gates`/`check:panel-refs`/`check:debt-owner` — all 0 findings.
Both counter-proofs run for real: (1) removing `segment_files_boundary.rs` and re-running —
57 binaries, 0 failures (green, as required). (2) mutating `split_chapters_step_files` to reuse
unit 0's cleanup report/joined-line count for every unit (chapter *count* stays correct at 2,
only the per-unit *numbers* are wrong) — the new boundary test goes red specifically on the
per-unit claim (`left: 1, right: 0` on the cleanup match count), matching the spec's explicit
"red on the per-unit number claim, not only on the count." Mutation reverted before landing.

**Measured, not claimed** (§Tasks "MEASURE, do not claim"): one `preview_import_encoding` pass
on `PipelineShape::Files` with 200 `.txt` files, fixture enlarged so the total is genuinely
comparable to the 440 KB baseline (observed **436,600 bytes**, printed by the probe itself, not
hand-claimed), one cleanup rule, debug build, `cargo test`, 2026-09-16: **353–382 ms across four
runs** (353.4, 363.2, 372.8, 381.7 ms) — see
`src-tauri/tests/segment_contract.rs::perf_probe_file_batch_preview_on_two_hundred_files`,
recorded next to the pre-story 440 KB/6-run baseline it cites
(`cleanup_contract.rs::perf_probe_chapter_split_preview_on_two_thousand_chapters`, 2026-09-05).
🔵 **SỬA 2026-09-16 (phản biện)** — an earlier fixture pass (20 lines/file) produced only
364,600 bytes total (17% under the 440 KB the comment invoked for comparability) and was
measured at 295 ms; the fixture now builds 24 lines/file so the printed total is actually
~437 KB, and the number above is what that corrected fixture measured, not the earlier one.
Sub-second for a full five-candidate preview of 200 files; not booked as debt.

**Known limitations, recorded as owned debt in `deferred-work.md` (2026-09-15/16), not guessed
at or silently shipped:** (1) `.docx` inside an N > 1 batch is refused per item — no per-unit
`DocxSidecar`/weaving this story (§Decisions was explicit that this is deferred, not a gap);
closing it needs `DocxSidecar` to become one-per-Chapter and touches the anchor invariants
Story 6.11/6.13 rely on.

(2) 🔴 **SỬA 2026-09-16 (vòng rà đối kháng 2, mục 6/G5) — bản SỬA ngay dưới đây (từ vòng rà
trước) overclaims; sửa tại chỗ.** `display_window_for_chapter` does now take `chapter_pattern`
and, when it is `None`, `units.get(chapter_index)` is an exact lookup (`split_unit_for_files`
yields exactly one piece per unit in that case, so the mapping is the identity, not a guess —
covered by
`cleanup_contract.rs::chapter_detail_for_index_on_a_files_batch_with_no_pattern_returns_that_units_own_window`).
That is correct groundwork inside the pure function. But the `⌥←`/`⌥→` lazy per-Chapter detail
cursor (Story 6.10a tiers 2/3) does **not** reach that groundwork for `Files` at all — in
**either** the pattern or pattern-less case — because two gates sit in front of it, both
pre-dating this story: `loadImportPreviewChapterDetail` (`src/importPreviewState.ts`) returns
immediately at `if (lastSubmittedFrom.value !== 'urls')`, so a file-submitted preview never
calls the lazy-detail IPC command regardless of the cursor; and even if that gate were lifted,
`wire::preview_chapter_detail` builds its shape only from `UrlImportItemsState` via
`chapters_shape_for_view`, which only ever yields `PipelineShape::Chapters` — there is no code
path that hands it a `PipelineShape::Files` for `display_window_for_chapter` to receive. No user
action reaches either the old unconditional `None` or the new per-unit window. Tier 4
(title/length/review) is unaffected — it is eager for every Chapter regardless of shape or
pattern, which is why this went unnoticed: tier 4 already answers most of what the cursor would.
The debt (see `deferred-work.md`) is therefore not "narrowed to the pattern case" — it stays
fully open for `Files`, and closing it needs both gates opened (a `lastSubmittedFrom` guard
change plus a new IPC path reading `PendingImportSourceState`/`FilesImportOutcome`), not just
the pure-function change already made.

## Spec Change Log

- 2026-09-15/16 (dev) — Added `ImportedChapter::source_file: Option<String>` /
  `ChapterSplitPreviewEntryWire::source_file: string | null` (not in the original Code Map) so
  `Flow::labels` — already tracked correctly per-unit by the pipeline — has somewhere to surface
  on the wire; required by the frontend task's "show each Chapter's source file." Three existing
  `ChapterSplitPreviewEntryWire` test fixtures widened accordingly (reason: widened return
  type, not a loosened expectation).
- 2026-09-15/16 (dev) — `display_window_for_chapter` returns `None` unconditionally for `Files`
  (Story 6.10a's lazy per-Chapter detail cursor) rather than attempting a partial mapping;
  booked as owned debt (see Implementation Notes) instead of guessed at.
- 2026-09-16 (dev, coordinator review round) — Three fixes from an independent diff review
  (coordinator re-ran the whole suite and read against the spec, not the implementer's report):
  (1) `display_window_for_chapter`'s `Files` arm no longer returns `None` unconditionally — it
  now takes `chapter_pattern` and returns the exact per-unit window when no pattern is
  configured (the primary journey; a frozen §Always line was being violated for it), keeping
  `None` only for the pattern case where the chapter_index↔unit mapping genuinely isn't the
  identity; the `deferred-work.md` entry was narrowed to match (`→ 🟡` correction, not a
  silent rewrite) and two new tests (positive + negative) were added to
  `cleanup_contract.rs`. (2) `segment_contract.rs`'s "single file untouched" test claimed
  `.docx` coverage its body never exercised (it wrote a `.txt`); split into the original `.txt`
  test (doc-comment corrected to not overclaim) plus a new test that builds a real `.docx` via
  `fixtures_docx::plain()` (shared module, same mechanism `docx_contract.rs` uses) and asserts
  shape + `DocxSidecar` equality against `import_file` directly. (3) The 200-file perf-probe
  fixture printed 364,600 bytes while its own comment claimed "~440 KB ... CÙNG cỡ ... để so
  được" — enlarged the fixture (20→24 lines/file) so the printed total is genuinely ~437 KB,
  re-measured for real (353–382 ms across four runs, 2026-09-16), and corrected the recorded
  number in this file's Implementation Notes accordingly (previous 295 ms/364.6 KB pair was
  wrong; replaced with a `🔵 SỬA 2026-09-16` note stating the old and new figures side by side —
  no strikethrough markup used, correcting an earlier Change Log line that claimed one was).

## Review Triage Log

### Pass 1 — 2026-09-16 (blind-hunter · edge-case-hunter · verification-gap)

39 findings filed across three layers; grouped into 12 entries by shared root cause plus 8
rejections. No entry routed to intent_gap or bad_spec, so no loopback: every surviving entry is a
defect inside the diff, not a gap in the captured intent.

**G1 — `source_file` is set for EVERY shape, not only `Files` — `high`, patch.** Verified by
probe: `run_import` on `Blob(RawBytes{label})` with no pattern returns
`source_file = Some("/Users/…/chuong-001.txt")`, and on `Chapters` returns
`Some("https://example.com/…")`; the same Blob **with** a pattern returns `None` (labels cleared in
that branch only). The overlay renders the field for every source, so a single-file import now
displays the user's absolute filesystem path and a URL import labels a URL as its "source file" —
on two paths this story was required to leave unchanged. Rows: verification-gap gap 1 (filed
`patch`, carried as filed); edge-case #1, #2, and its `source_file` claim (all confirmed by the
same probe).

**G2 — the remove-file path mutates state before its failure branches — `high`, patch.**
`importPreviewState.ts` `removeImportPreviewFileItem` assigns `pendingPaths.value = next` above
both the `result.error !== null` and `result.batch === null` returns, so on failure the on-screen
`fileImportItems` and `preview` keep the pre-removal batch while the pending list is already
shorter: indexes desynchronise (a second remove drops the wrong path) and, when the removed file
was the last one, `import_files` returns `EmptyFileList`, the wire returns through `?` **before**
either `stash_pending_import_source` or `cancel_import_preview`, so Rust still holds the old shape,
`preview.value` stays non-null and confirm writes the file the user just removed. Rows:
blind-hunter #3, #4; verification-gap other-finding 1; edge-case #3, #5, #4 (the wire half — a
stale pending shape surviving an `Err` from `import_files`).

**G3 — confirm is not blocked during a per-file round trip — `medium`, patch.** The confirm button
guards `importPreviewUrlImportBusy` but not `importPreviewFileImportBusy`, and
`confirmImportPreview` guards `confirming`/`chapterPatternSending`/`canConfirm`/`selectedEncoding`
only, while `removeImportPreviewFileItem` does refuse to start during `confirming` — the block is
one-directional, so a confirm fired mid-removal reads a pending cell the in-flight preview is
re-stashing or clearing. The URL branch closes exactly this window. Rows: blind-hunter #1;
edge-case #6, #7.

**G4 — a zero-byte FIRST file collapses the whole preview — `high`, patch.** Verified by probe on
a 3-file batch whose first file is empty: `candidates = 0`, `confidence = SelfDeclared`, tier 4
absent, and `self_declared_chapters = Some(1)` — the preview claims **one** chapter for three
files, while confirm runs the real chain and writes three. Contrast run with all files non-empty:
5 candidates, `High`, tier 4 = every chapter. Cause: the `Files` arm takes `units.first()` as the
encoding representative, and empty bytes yield zero candidates, which routes the whole preview into
the self-declared fallback that rebuilds a single `Blob` from one text. This breaks the frozen
matrix row "Preview equals confirm" and is a silent under-count. Row: edge-case #8 (its stated
consequence understated the damage — it predicted an empty strip, not a 3→1 chapter collapse).

**G5 — the `Files` arm of `display_window_for_chapter` is unreachable, while the records claim the
journey is restored — `medium`, patch.** `loadImportPreviewChapterDetail` returns at
`if (lastSubmittedFrom.value !== 'urls')` and `wire::preview_chapter_detail` builds its shape from
`UrlImportItemsState` through `chapters_shape_for_view`, which only ever yields
`PipelineShape::Chapters` — so no user action reaches either the old `None` or the new per-unit
window. The gate pre-dates this story; what this story added is the claim, in `deferred-work.md`
and in Implementation Notes, that the cursor journey is now available for pattern-less `Files`.
The code is correct groundwork; the record is false. Rows: verification-gap gap 3 (filed `patch`,
carried as filed); edge-case `⌥←`/`⌥→` claim; blind-hunter #11 (the stale Commands paragraph).

**G6 — the `LowGuess` predicate has no negative case, and mixes two verdict sources — `medium`,
patch.** One test reads confidence on a `Files` shape and it asserts `Low`; nothing pins that a
batch whose files all detect the same encoding keeps its real confidence, so an unconditional
downgrade would stay green. The predicate also compares `verdict_and_candidates(first_bytes, …)`
against raw `encoding::detect(bytes)` — two sources for the same question. Row: blind-hunter #2.

**G7 — the all-ok / lock / stash decision lives only in the IPC shell — `medium`, patch.** The one
expression that implements §Decisions' locked confirm sits inside
`wire::preview_import_encoding_from_file`; every test that names that wire compares its source text
(registration, verbatim parameter list, command census) and none executes it, while the frontend
tests mock the adapter and hand-build `encoding_preview: null`. Inverting it would produce a Work
silently missing the failed file with nothing red. The repo's own two-layer rule puts such a
decision in a pure function. Row: verification-gap gap 2 (filed `patch`, carried as filed).

**G8 — an N = 1 failure renders "Danh sách tệp (0)" and hides the failing path — `medium`,
patch.** The section is gated on `importPreviewLastSubmittedFrom === 'file'` alone, and a global
`Err` (the documented N = 1 behaviour) leaves `fileImportItems` empty, so the header counts zero
files while its own comment promises the list is shown "kể cả N = 1". Rows: blind-hunter #5;
edge-case #12 and its AC1 claim.

**G9 — `chapter_urls` for `Files` carries real filesystem paths at per-unit arity — `low`, patch.**
The arm maps `chapter_input_page_url` over the units while its sibling defensive arms use an empty
placeholder; with a pattern, N units produce more than N chapters, so the vector is misaligned by
construction and its reader would take a local path as a page URL. No bad outcome today —
`prepare_chapter_images` reads it only per pending image and `Files` produces no blocks, and the
1:1 assertion fires only when `extract_main_content` — so this is dev-only, but the fix is a
one-line direct correction that removes the hazard rather than guarding it. Row: blind-hunter #8.

**G10 — stale prose and dead pointers in new code and logs — `low`, patch.** The perf-probe comment
re-states the byte figures it just got wrong and still carries a leftover instruction to the reader
("đo lại bằng Python trước khi chốt con số này"); `libraryImport.ts` points at a "mục lịch sử" in
`vi.json`, which is a flat key→string object with no comments; `libraryImportDropMultiple.test.ts`
attributes a sentence to `AGENTS.md` that lives in this spec; and the Spec Change Log claims the
295 ms figure "is struck through there" when no strikethrough exists. Rows: blind-hunter #10, #12,
#13.

**G11 — `Files` can be built with 0 or 1 surviving units, contradicting the variant's own
doc-comment — `low`, patch.** With three of four paths failing, `inputs` holds one unit; with all
failing, none — while `pipeline.rs` states "N = 1 KHÔNG đi qua nhánh này". Unreachable today only
because the wire skips the preview when any item failed: an invariant held by a caller, not by the
constructor. Smallest honest fix is to correct the doc-comment to describe what the type actually
permits. Rows: blind-hunter #6; edge-case #10.

**G12 — the chapter pattern is matched twice per unit — `low`, patch.** `match_starts` compiles the
regex on every call (`let re = compile(&self.pattern)?`), and `split_unit_for_files` calls it to
test for emptiness and then delegates to `split_on_positions`, which calls it again — so the
pattern path pays two compiles and two scans per file, a factor this story introduced and then
multiplied by N. Confined to private functions. Row: edge-case #13.

**Rejected — 8 findings.**
- The §Tasks bullet "feed failed items into the existing `broken_item_count` parameter" is not
  implemented (the wire passes a literal `0`, and by construction it can only ever be `0` because
  the preview is built only when every item is OK). Rejected: the bullet contradicts the frozen
  §Decisions rule that `encoding_preview: None` is the single locked-confirm condition, the frozen
  text won, and the only available fix is an edit to this build's spec. Raised to the human
  instead. Rows: blind-hunter #7; verification-gap other-finding 2; edge-case
  `broken_item_count` claim.
- AC "the chosen encoding is never the first file's by default" — `false` as filed. The filed bad
  outcome is that a batch "silently decodes all files as file #1"; it is not silent, because the
  downgrade to `low` is precisely what opens the five-candidate strip, which is the mechanism
  §Decisions chose. The AC's tail clause is looser than the code only in wording, and its fix would
  edit this build's spec. Raised to the human. Row: edge-case AC5 claim.
- AC "the other files render and confirm is locked" — the item list does render all N with their
  reasons, but all four tiers blank out, because `encoding_preview: None` (the frozen mechanism)
  necessarily removes them. Not a code defect; a product consequence of the frozen decision, and
  its fix would edit this build's spec. Raised to the human. Row: edge-case AC4 claim.
- Unbounded batch size — `low`. `import_files` reads every file into memory with only the
  per-file 100 MB cap above it. Rejected because the fix adds a cap constant, an error variant and
  a branch, and a folder of chapter files does not approach it in everyday use. Raised to the human
  as a known unbounded path. Row: edge-case #9.
- Duplicate paths become duplicate Chapters — `low`. Rejected on the same rule: the fix adds a set
  plus an error variant. Row: edge-case #11.
- `reject_batch_unsupported_extension` reuses `MissingExtension`, whose string names `.docx` as
  openable while a batch refuses it — `low`, reachable only by an extensionless file inside a
  batch, and the fix adds a message key (new surface) for that path. Row: blind-hunter #9.
- Bookkeeping drift and refs absent from `openWith` — `false`. The sprint/spec statuses are the
  coordinator's and are consistent as of this pass; the new refs follow the URL branch's own
  precedent and are cleared in `resetImportPreview`, which is what `check:panel-refs` inspects. No
  named harm. Row: blind-hunter #14.

## Design Notes

**Why a new shape instead of reusing `Chapters`.** `Chapters` means *already chaptered* and skips
step 5 by shape, deliberately not by `units.len()` (`pipeline.rs:198-202`). N files must be able to
split **further** by the pattern, which is the opposite decision, so reusing `Chapters` would either
break the URL path's guarantee or make the skip depend on something other than shape. `Files` states
the third case honestly: N units, step 5 runs, per unit.

**Why `Files` measures differently from `Blob`, and why that is not an inconsistency.** On the
`Blob` path the cleanup report and joined-line count are measured on the whole document before step
5, so they belong to the document and not to any Chapter — that is why the report is pinned to
Chapter 1 and the joined count is reset to `None` for all N (`pipeline.rs:1342-1380`). With N files
each unit is decoded, cleaned and normalized on its own (steps 1-4 already loop per unit), so a file
that stays one Chapter has numbers that are genuinely its own. Keeping them is not a loosened rule;
discarding them would be the lie, and pretending pieces of a split file have them would be the other
lie. Hence the `k == 1` / `k > 1` split in §Always.

**Why no second managed state for the file items.** The URL path needs `UrlImportItemsState` because
a fetched byte buffer must never be fetched twice, so the items have to survive between commands. A
file can be re-read, so the item list is **derived** at preview time and returned on the wire; the
only thing that persists is the shape, and a shape containing a failed item never reaches confirm
because confirm is locked while one exists. One fewer piece of state than the path it copies.

**Why the batch stays one pending import.** `PendingImportSourceState` is a single-slot mutex whose
guard is deliberately held across `create_work` to block a double confirm (`mod.rs:3390-3415`).
N files are one user action producing one Work, so they belong in that one slot; making the slot a
list would weaken the double-confirm guard to buy nothing this story needs.

## Verification

**Commands:**
- `npm run build && cd src-tauri && cargo test --locked` -- expected: green; `dist/` must exist
  before `cargo test` or the break is at compile time, not at an assert
- `npm run test` -- expected: the pre-story count green, 0 red, no expectation edited except the
  pinned parameter list
- `npm run check:i18n && npm run check:tokens && npm run check:commands && npm run check:gates && npm run check:panel-refs && npm run check:debt-owner`
  -- expected: 0 findings each
- Red counter-proof: remove `src-tauri/tests/segment_files_boundary.rs` and run the old suite --
  expected: **green**
- Red counter-proof 2: make `split_chapters_step` handle only the first unit for `Files` and run the
  new cases -- expected: **red**, and red on the per-unit number claim, not only on the count

**Manual checks (if no CLI):**
- Drop a folder of files onto the Library: the count matches what was dropped, each Chapter names its
  source file, `⌥W` filters, `⌥←`/`⌥→` stop at both ends, `Tab` stays inside the overlay, and
  confirming writes exactly the Chapters the preview showed.
</content>
</invoke>
