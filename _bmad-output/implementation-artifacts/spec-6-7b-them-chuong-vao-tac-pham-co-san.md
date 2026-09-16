---
title: 'Story 6.7b: Add Chapters to an existing Work'
type: 'feature' # feature | bugfix | refactor | chore
created: '2026-09-16'
status: 'done' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
baseline_commit: '7fb3aa7202044aefda0d1a0b40d5bf4cbef9276d'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Every import path in the product creates a *new* Work. A translator following a
serial that publishes weekly has no way to put next week's chapters into the Work they are
already translating, so one novel shatters into one Work per import batch. FR122's second
half — *"or add Chapters to an existing Work"* — has never had a line of code: the seven-token
grep (`add_chapter|append_chapter|import_chapter|addChapter|appendChapter|existing_work|target_work`)
over `src-tauri/src` and `src` returns 5 hits, all of them bench-fixture name helpers in
`lib.rs`. This is a capability that was never built, not a spec mismatch.

**Approach:** Give the import preview a destination: **New Work** (today's behaviour, unchanged)
or **an existing Work** picked from the Library. For an existing destination the confirm step
opens that Work's `.atproj` instead of creating a folder, appends the new Chapters after the
last existing one, and re-runs the shared four-step lifecycle template so `meta.json` and the
Library index stay truthful. This is the first import path that writes into pre-existing user
data, so the write must be additive-only and provably so.

## Decisions (Ice, 2026-09-16)

1. **Scope — the three monolingual import paths** get the destination picker: pasted text,
   file, and URL list. Measured basis: all three confirm through the single
   `confirm_import_with_encoding` (`wire.rs:621`), so one parameter serves all three and the
   preview overlay is already shared. The **bilingual** path (`confirm_bilingual_import`,
   `wire.rs:847`) is out of scope and becomes an owned debt entry — Story 6.16b is already
   queued against that screen and the two stories must not edit it mid-flight.

2. **`source_lang` is adopted from the destination Work and shown read-only.** No language
   detection, no refusal. ⚠️ Recorded consequence, not softened: a batch pasted in a different
   language than the destination is sentence-split by the wrong rules and nothing says so.
   This is accepted because `work.source_lang` is immutable — one `INSERT`
   (`commands/project/mod.rs:701`), no `UPDATE` anywhere in the repo — so adopting it is the
   only coherent behaviour, and surfacing a mismatch needs a signal the import path does not
   have. A debt entry with an owner records the gap.

3. **A destination Work that is currently open is allowed**, reusing the `Store` already held
   in `OpenWorkState`. This is the story's common case, not an edge case. 🔴 Constrained by
   `src-tauri/AGENTS.md:30`: never open a second write connection to the same `project.db`.

4. **The wrong `AD` citation is fixed in place** in `epics.md:4861` and
   `sprint-change-proposal-2026-09-15.md:52` — `AD-44` → `AD-8`, with 🔵 and the date, per
   `AGENTS.md:51`. `src-tauri/AGENTS.md:54` already cites AD-8 correctly; this makes the three
   documents agree.

## Boundaries & Constraints

**Always:**

- 🔴 **Nothing reaches disk before confirmation** — including the target Work's `.atproj`.
  The preview stays entirely in memory, exactly as it does for the New Work path today.
- 🔴 **The append is additive only.** Existing `chapter` and `segment` rows — their text,
  translations, version history, status, `ord`, and asset anchors — must be byte-identical
  before and after. New Chapters take `ord = MAX(ord) + 1` upward, in the order the preview
  showed them.
- 🔴 **Go through the shared four-step template** (AD-8, `ARCHITECTURE-SPINE.md:133`):
  SQL write → `WorkMeta::rebuild_from_store` → `meta.write_atomic` → `Indexer::rebuild`.
  `commands/lifecycle.rs:87` `write_lifecycle_after_change` carries steps 2–3 and
  `:230` `reindex_after_lifecycle_write` carries step 4. A hand-copied second instance of this
  sequence is the exact defect the 2026-08-27 measurement caught (removing step four gave
  **0** failures across 34 binaries).
- 🔴 **Work-tier cleanup rules resolve from the DESTINATION Work.** Today
  `resolve_cleanup_rules` (`commands/project/wire.rs:21-38`) locks `OpenWorkState` and merges
  whatever Work happens to be open. For a New Work destination that is already a known defect
  (`deferred-work.md:10063-10081`); with an existing destination it becomes a rule set from
  an unrelated Work silently deleting content — the failure class FR124 exists to prevent.
- 🔴 **One writer per store** (`src-tauri/AGENTS.md:30`). If the destination Work is the one
  currently held in `OpenWorkState`, reuse that `Store`; never open a second write connection
  to the same `project.db`.
- New Chapters are created `NotStarted`; the Work's status is re-derived by
  `derive_work_status` (`core/lifecycle/mod.rs:106`) through `WorkMeta::rebuild_from_store`,
  and a `work.status_override` value keeps winning (`core/library/meta.rs:390-395`).
- `work.source_lang` is set once at creation and no `UPDATE` in the repo touches it
  (verified: the only write is the `INSERT` at `commands/project/mod.rs:701`).

**Never:**

- 🔴 **Never copy `create_work`'s error path.** It calls `remove_folder(&dir)` at **10** sites,
  one of them *after* the transaction has committed (`commands/project/mod.rs:855`). That is
  correct for a folder this function just created and catastrophic for a Work the user already
  owns. The append path's failure mode is "leave the Work exactly as it was", never "delete it".
- Never change `work.name`, `work.genre`, or `work.source_lang` of the destination Work.
- Never add a `work_id` column to `chapter`, and never relax `CHECK (id = 1)` on `work`
  (`core/store/schema.rs:711-720`) — one Work per `project.db` stays the shape.
- Never build the Work-tier cleanup-rule *authoring* surface — `deferred-work.md:10063-10081`
  is owned by Ice. This story needs only the **read** half to name the right Work.
- No new keyboard chord. The destination control follows the existing library-import pattern:
  reached by Tab, committed through `@change`/`@submit`, never a bare `@click` with arguments.
- No new `AD`. If an invariant turns out to need changing, stop and hand the file to Winston
  (`AGENTS.md:13`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| New Work (regression) | Destination = New Work, N Chapters | Identical bytes to today's import; `ord` 1..N | unchanged |
| Append, happy path | Destination = Work W holding M Chapters, N new | W holds M+N Chapters; new ones at `ord` M+1..M+N, all `NotStarted`; no second Work created | N/A |
| Old rows untouched | Same as above | Every pre-existing `chapter`/`segment` row identical before/after, `ord` included | N/A |
| Destination is the open Work | W is in `OpenWorkState` | Append succeeds reusing the open `Store`; open editor state stays valid | N/A |
| Cleanup tier | Destination = W, a *different* Work X is open | Preview applies Global + **W's** rules; X's rules touch nothing | N/A |
| Status re-derivation | W was `Done`, N new `NotStarted` Chapters land | W re-derives to `InProgress` | N/A |
| Status override set | W has `status_override` | Stored status unchanged after append | N/A |
| Preview not confirmed | Destination = W, user closes preview | `project.db`, `meta.json`, `library-index.db` byte-identical; `mtime` untouched | N/A |
| Destination vanished | W deleted or moved between picking and confirming | Confirm refused, named error, **0** rows written | `MessageKey`, not a panic |
| Destination unreadable | W's `project.db` corrupt or `meta.json` schema newer | Confirm refused, named error, W left as-is | reuse `WorkError::OpenFailed` / `MetaTooNew` |
| Write fails mid-append | Transaction error after some inserts | Whole transaction rolls back; W unchanged; **no** folder removal | explicit — do not reuse `create_work`'s cleanup |
| Empty Library | No Works indexed | Destination control offers New Work only, and says why | N/A |

</frozen-after-approval>

## Code Map

**Rust — the write path**

- `src-tauri/src/commands/project/mod.rs:355-864` `create_work` — the whole of today's import
  landing. Creates the folder (`create_work_folder`, `:389`), a fresh `Uuid::new_v4()` work_id
  (`:400`), inserts the single `work` row (`:701`), loops chapters assigning `ord = i + 1`
  (`:722-723`), inserts segments via `commands::segment::insert_segments` (`:768-775`) and
  asset rows (`:781-795`), then `WorkMeta::rebuild_from_store` (`:817`) + `write_atomic`
  (`:843`). 🔴 **Not extendable by a parameter** — no argument can name an existing Work, and
  its 10 `remove_folder(&dir)` sites make it the wrong function to branch. The append path is a
  **sibling** function that reuses the chapter/segment/asset insert shape only.
- `src-tauri/src/commands/project/mod.rs:4500` `open_work(work_id, indexed: Option<&IndexedWork>)`
  — the existing way to address an existing `.atproj` by `work_id` through the Library index
  (`indexed.atproj_path`), with `MetaError::SchemaTooNew` / `Io` already mapped to `WorkError`.
  **Reuse this resolution**; do not invent a second path-from-id.
- `src-tauri/src/commands/lifecycle.rs:87` `write_lifecycle_after_change(open: &mut OpenWork)`
  — steps 2–3 of the four-step template; its doc-comment (`:80-86`) names reuse as *the only*
  way not to cast a second copy. `:230` `reindex_after_lifecycle_write(indexer, global, root)`
  — step 4. Both are the seam the append path must ride.
- `src-tauri/src/commands/chapter.rs:503-517` `normalize_chapter_ord(tx)` — dense 1..N
  renumber by `(ord, id)`; `:1006` carries the shift-insert shape
  (`UPDATE chapter SET ord = ord + 1 WHERE ord > ?1 AND id <> ?2`). Append needs neither if it
  writes at `MAX(ord)+1`, but this is the precedent to read before inventing ordering logic.
- `src-tauri/src/core/store/schema.rs:711-720` `WORK_DDL` — `CHECK (id = 1)`, and the
  doc-comment at `:705-709` stating `source_lang` is enforced immutable at the application
  layer because SQLite has no write-once column. `:1041-1050` `CHAPTER_DDL` — **no `work_id`
  column and deliberately no `UNIQUE` on `ord`** (`:1029`, `:1075`). `project.db` migrations
  top out at `to_version: 22`.
- `src-tauri/src/core/lifecycle/mod.rs:106` `derive_work_status(&[LifecycleStatus])` —
  0 chapters ⇒ `NotStarted`; all `Done` ⇒ `Done`; all `NotStarted` ⇒ `NotStarted`; else
  `InProgress`; never `Paused`. Called from `core/library/meta.rs:393`; the override branch is
  `meta.rs:390-395` (`status_override` wins, `status_is_override = true`).
- `src-tauri/src/core/library/indexer.rs:220` `Indexer::rebuild` — the **only** writer of
  `library-index.db` (AD-8). Four non-test callers: `lib.rs:1237`, `commands/library.rs:193`,
  `commands/project/wire.rs:317`, `commands/lifecycle.rs:230`.

**Rust — IPC shells and pending state**

- `src-tauri/src/commands/project/wire.rs` — every `#[tauri::command]` for this path.
  `:466` `preview_import_encoding_from_text` · `:542` `preview_import_encoding_from_file`
  (async) · `:621` `confirm_import_with_encoding(app, name, source_lang, genre, encoding,
  chapter_pattern)` (async since AI-4) · `:929` `start_url_import(app, urls, source_lang)` ·
  `:971` `reload_url_import_item` · `:1030` `remove_url_import_item` · `:847`
  `confirm_bilingual_import` · `:317` `reindex_library`.
- 🔴 `src-tauri/src/commands/project/wire.rs:21-38` `resolve_cleanup_rules` — locks
  `OpenWorkState` and merges the **open** Work's rules, called from both preview entry points
  (`:477`, `:553`). This is the single function AC3 is about. `:40-52`
  `resolve_cleanup_rules_against` → `core/cleanup/store.rs:109` `resolve_two_tiers`.
- `src-tauri/src/commands/cleanup.rs:36-47` `store_for_tier(tier, global, open)` — resolves the
  Work tier from whatever `Option<&OpenWork>` the caller passed, with **no id parameter**. The
  five cleanup commands (`:185`, `:196`, `:213`, `:231`, `:247`) all infer the Work from open
  state. Widening the read half means giving this a destination, not adding a tier.
- `src-tauri/src/commands/project/mod.rs:1936-1947` `PendingImportSource { shape, docx_sidecar }`
  + `PendingImportSourceState`; `:3462` `stash_pending_import_source`. The destination has to
  live beside these for the life of one import.
- `src-tauri/src/commands/project/mod.rs:3554-3584` — the confirm critical section: the
  `PendingImportSourceState` guard is held across `create_work` and cleared under the same
  guard. Synchronous throughout (no `.await` inside); AI-4 made only the *shell* `(async)`.
- `src-tauri/src/commands/project/mod.rs:4625` `OpenWorkState = Mutex<Option<OpenWork>>`;
  `OpenWork` at `:52-61` (`dir`, `store`, `scope`, `meta`, `chapter_id`, image counters);
  `replace_open_work` `:4647-4702`, called from five sites in `wire.rs`.
- `src-tauri/src/commands/library.rs:682-697` `library_list_works(app, filter, genre,
  source_lang, sort)` → `WorkListReport { total, matched, works, genres, source_langs }` with
  `WorkRow { work_id, atproj_path, name, source_lang, genre, created_at, updated_at,
  chapter_count, status, status_is_override, chapter_done_count }` (`:307-320`). The read
  surface for the picker already exists — nothing new needed here.
- `src-tauri/src/lib.rs:712-913` `generate_handler!` — **79** entries today (78 without the
  `nfr-bench` feature). Registration is asserted by plain substring containment, not macro
  expansion.

**Rust — gates**

- `src-tauri/tests/ipc_contract.rs:1288-1299` `fn_param_list` + `:1046-1131` the worked
  instance for the three preview wires. Adding a command means two edits: its path into the
  `for wire in [...]` containment list, and a `(fn_name, expected_params)` tuple. The
  self-proof that the helper binds to the right block is `:1321-1336` — **copy that discipline**.
- `src-tauri/tests/project_contract.rs` — 4495 lines, 113 `#[test]`. Nearest neighbours:
  `create_work_writes_every_chapter_and_its_segments_when_the_pipeline_yields_more_than_one`
  (`:618`), `create_work_writes_titles_and_continuous_ord_when_n_chapters_come_from_a_chapter_pattern`
  (`:691`), `n_chapters_from_a_url_list_write_clean_text_ord_and_segments_for_every_chapter`
  (`:930`), `pasted_text_and_a_read_file_travel_the_same_import_path` (`:1128`).
- `src-tauri/tests/cleanup_contract.rs:845`
  `choosing_the_work_tier_with_no_work_open_is_refused` and `:280` (two tiers merging on one
  spot). ⚠️ **No case anywhere pins the mismatched-Work case** — a preview resolving Work-tier
  rules against a Work that is not its destination goes unguarded today.
- `src-tauri/tests/library_index_contract.rs` — 88 `#[test]`, home of the AD-8 ordering case
  `orphan_write_order_is_fail_safe_write_global_before_deleting_from_index`.
- Static test population, measured 2026-09-16 by counting `#[test]` attributes across
  `src-tauri/tests/` (53 files): **1357**, of which **25** carry `#[ignore]` in 11 files.
  ⚠️ This is a source count, not a run count — take the real baseline by running the suite
  before touching a line.

**Frontend**

- `src/modes/libraryImport.ts` (519 lines) — `pastedText` `:89`, `filePath` `:92`,
  `droppedFilePaths` `:104`, `effectiveFilePaths` `:112`, `bilingualFilePath` `:136`,
  `pastedUrls` `:146`, `pastedUrlCount` `:172`, `busy` `:175`. Four submit functions —
  `:372`, `:403`, `:421`, `:433` — all sharing one guard shape: `busy || <overlay-open>`
  **inside the function body**, empty-input check, then `beginSubmit()`. A destination ref
  belongs here, beside them.
- `src/importPreviewState.ts` (1985 lines) — `openImportPreviewFromText` `:707`,
  `…FromFile` `:734`, `…FromUrls` `:838`; `lastSubmittedFrom: 'text'|'file'|'urls'|null`
  `:119`; `pendingText` `:130`, `pendingPaths` `:134`; URL per-item state in `urlImportItems`.
  🔴 `resetImportPreview()` `:1931-1985` clears **47** slots (verified 2026-09-16) — every new
  slot goes in, or `check:panel-refs` goes red: its rule is generic (*every module-level slot
  must pass through a `reset*` function or carry a named exemption with a readable reason*),
  not a special case for this file.
- `src/ImportPreviewOverlay.vue` (2564 lines) — template from `:816`; header `:824-846`; URL-list
  block `:856`; file-list block `:934`; tier 1
  🔵 **CORRECTED 2026-09-16 (orchestrator, after Phase 3 disputed it).** This line first read
  *"header with name and source-lang fields `:824-846`"* — **that was wrong**, carried in from an
  investigation report without being measured. `git show HEAD:src/ImportPreviewOverlay.vue`
  lines 824-846 hold a title and a close button and nothing else. The Work name, source
  language and genre fields live in `src/modes/LibraryMode.vue:1233-1241`, on the pre-fetch
  form — which is also what the mockup's right-hand column actually depicts. ⇒ Phase 3 put the
  destination radios and the Work picker in `LibraryMode.vue` and gave the overlay a read-only
  echo, and that placement is **accepted as correct**: the destination must be fixed before the
  first preview call, because `source_lang` has to be the adopted value already on that call and
  `start_url_import` takes the destination exactly once. The spec was wrong; the code is right.
  (encoding) `:1007`; normalized `:1061`; tier 3 (cleanup) `:1232`; tier 4 (chapters) `:1391`.
  The three existing `<select>` controls (`:1288`, `:1370`, `:1410`) are all literal/regex kind
  pickers — **no destination control exists**. `reloadImportPreviewAfterRuleChange` is the
  existing machinery for "a setting changed, rebuild the preview"; a destination change rides it.
- `src/config/library.ts` — `CMD_LIST_WORKS` `:310`, `listLibraryWorks()` `:447`, `WorkRow`
  `:179-191`, `WorkListReport` `:209-215`, `isWorkRowArray` `:223`. Adapter already present.
- `src/config/project.ts` (1407 lines) — the create/confirm adapters; `isIpcError` `:59-66`
  shows the manual per-field `typeof` narrowing every wire type uses. 🔴 `invoke()` sends
  **camelCase** while returned struct fields stay **snake_case** (`src/AGENTS.md:10`).
- `src/commands/index.ts` — `library.import_text` `:1139`, `library.import_file` `:1147`,
  `library.import_urls` `:1157`, `library.import_bilingual` `:1167`; comment `:1133-1136`
  explains all four deliberately bind **no key**.
- `src/i18n/vi.json` — **104** `mode.library.preview.*` keys; **0** keys naming a Work or
  destination picker. `command.import.preview.open_picker` `:215` is the *encoding* picker.
- `tests/frontend/` — 76 test files, **986** `it(`/`test(` call sites total (static count,
  2026-09-16). Of those, 17 files / **232** cases touch the import preview; the largest are
  `importPreviewChapters.test.ts` (43), `importPreviewBilingual.test.ts` (25),
  `importPreviewEncodingWireShape.test.ts` (24), `importPreviewUrls.test.ts` (16),
  `importPreviewFiles.test.ts` (14).

**Mockup**

- `_bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/mockups/web-import.html:198-207`
  — the destination control is **two radios**, not a `<select>`: *"Tác phẩm mới — Tạo một Tác
  phẩm rồi thêm cả 50 Chương vào đó"* and *"Tác phẩm đã có — Thêm Chương vào **cuối** một Tác
  phẩm sẵn có"*, in the same right-hand column as the Work name and source language. 🔵 The
  phrase *"vào cuối"* settles where new Chapters land: **appended at the end**, no position
  chooser. Story 5.8 already owns reordering after import.

**Related debt** — `deferred-work.md:10354-10396` (this story's own entry; three premises
re-measured 2026-09-15, all still true on `7fb3aa7`) · `:10063-10081` (Work-tier authoring
surface, **owner Ice — not this story**).

## Tasks & Acceptance

Split into four phases along the shape above, each handed to a fresh agent through this file
(`AGENTS.md:17` — one agent must not implement a whole story).

**Execution:**

*Phase 1 — measure before writing*
- [x] **MEASURE.** Take the real baseline: `npm run build && cargo test --locked` and
      `npm run test`. Record both pass counts and the date here. The 1357/986 figures in the
      Code Map are **source counts**, not run counts — do not quote them as a bar.
- [x] **MEASURE.** Prove the counter-check target is real before building the gate: with a
      Work X open and an import preview built for a *different* destination, confirm today's
      code merges X's cleanup rules. If it does not reproduce, stop — AC3 would be guarding a
      defect that is not there.

*Phase 2 — Rust append path*
- [x] `src-tauri/src/commands/project/mod.rs` -- new sibling of `create_work` that takes a
      resolved destination `OpenWork` and appends N Chapters at `MAX(ord)+1`, reusing the
      chapter/segment/asset insert shape from `:722-795` inside **one** transaction.
      🔴 Its failure path leaves the Work untouched — no `remove_folder`, no folder creation.
      Then steps 2–4 through `write_lifecycle_after_change` + `reindex_after_lifecycle_write`.
- [x] `src-tauri/src/commands/project/mod.rs` + `wire.rs` -- carry the destination through
      `PendingImportSource` and the confirm command; resolve an existing destination by
      `work_id` through the indexer the way `open_work` (`:4500`) does, and reuse the
      `OpenWorkState` handle when the destination is the Work already open.
      🔵 See Implementation Notes, 2026-09-16 correction — first cut used a per-call parameter
      for confirm too; corrected after coordinator review to carry the destination as a field
      on `PendingImportSource` (session-scoped), because five more mid-session wires turned
      out to need the same value with no per-call parameter of their own.
- [x] `src-tauri/src/commands/project/wire.rs` -- `resolve_cleanup_rules` (`:21-38`) resolves
      the Work tier from the **destination**, not from `OpenWorkState`, for **all nine**
      wires of the three monolingual sessions (not only the three entry points) — see the
      2026-09-16 correction in Implementation Notes for the count that was missing five seams
      on the first pass.
- [x] `src-tauri/src/core/i18n/` -- new `MessageKey` variants through `message_keys!` for:
      destination vanished, destination unreadable, destination schema newer. Closed
      catalogue — no parallel list; declare `params` to match `{ten_tham_so}` in `vi.json`
      (the sync gate checks both directions). 🔵 Satisfied by REUSE, not new variants — see
      Implementation Notes: `WorkMetaTooNew`/`WorkOpenFailed`/`LibraryWorkNotIndexed` (Story
      5.7) already cover exactly these three conditions because destination resolution goes
      through `open_work`.
- [x] `src-tauri/src/lib.rs` -- register any new command in `generate_handler!` (79 entries
      today) beside the existing import commands.

*Phase 3 — Webview*
- [x] `src/config/project.ts` -- destination on the wire (camelCase out, snake_case back), a
      `typeof` clause per new field, plus the call site in the same pass.
      🔵 See Implementation Notes, 2026-09-16 — outbound-only field, no new `isXxx` runtime
      validator (measured, not literally implemented — same shape as Phase 2's Task 4).
- [x] `src/importPreviewState.ts` -- destination slot and its setter; changing it rebuilds the
      preview through the existing `reloadImportPreviewAfterRuleChange` machinery.
      🔴 Every new slot goes into `resetImportPreview()` (47 today).
      🔵 See Implementation Notes — "setter"/"rebuilds" read as *carried through* every
      preview-rebuilding call (open **and** reload), not a live mid-overlay re-pick; the
      picker itself lives pre-open (Phase 3 placement note below).
- [x] `src/modes/libraryImport.ts` -- destination ref shared by the **three monolingual**
      submit functions (`:372` text, `:403` file, `:433` urls), defaulting to New Work; the
      destination travels with the submission. `submitBilingualFilePath` (`:421`) is untouched.
- [x] `src/ImportPreviewOverlay.vue` -- two radios in the header column (`:824-846`) per the
      mockup, plus the existing-Work picker fed by `listLibraryWorks()`. Committed through
      `@change`, never a bare `@click` with arguments. Work name and genre fields become
      read-only, showing the destination's values, when an existing Work is picked; the
      destination's `source_lang` is shown read-only and adopted (Decision 2).
      🔵 **Placement note — see Implementation Notes, 2026-09-16.** The interactive radios +
      picker were built in `src/modes/LibraryMode.vue`'s pre-submit form instead of inside
      `ImportPreviewOverlay.vue`; the overlay carries a read-only echo (name + adopted
      source_lang) in its header, satisfying Decision 2's "shown read-only" half. Measured
      reason, not a preference — recorded in full below.
- [x] `src/i18n/vi.json` -- keys for both radio labels and their descriptions, the picker, the
      read-only adopted-language line, and every new error reason (the three reused error keys
      from Story 5.7 already existed — no new ones needed, see Implementation Notes).

*Phase 4 — tests that move*
- [x] `src-tauri/tests/project_contract.rs` -- append writes `ord` M+1..M+N in preview order,
      all `NotStarted`, segments present for every new Chapter; **and** a case that snapshots
      every pre-existing `chapter`/`segment` row before the append and asserts byte equality
      after. 🔴 Assert on rows that could actually differ — an assert true on both branches
      guards neither (`AGENTS.md:68`).
- [x] `src-tauri/tests/project_contract.rs` -- status re-derivation after append, and a second
      case where `status_override` is set and survives. Two Works in the fixture, so "the right
      Work changed" is distinguishable from "a Work changed".
- [x] `src-tauri/tests/cleanup_contract.rs` -- the mismatched-Work case that exists nowhere
      today: destination W, open Work X, preview applies Global + W's rules and **none** of X's.
- [x] `src-tauri/tests/project_contract.rs` -- confirm refused when the destination vanished or
      cannot be opened, with **0** rows written and the Work left as-is.
- [x] `src-tauri/tests/ipc_contract.rs` -- lock the changed/added command parameter lists
      through `fn_param_list`, following the worked instance at `:1046-1131`.
- [x] `tests/frontend/importPreviewDestination.test.ts` (new) -- destination defaults to New
      Work; picking an existing Work makes name/genre read-only; changing the destination
      triggers exactly **one** preview rebuild; an empty Library offers New Work only. Measure
      IPC by the **difference between mock call counts**, not `not.toHaveBeenCalled()`.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- a `→` line closing
      `:10354-10396`, plus two **new owned** entries: ① the bilingual import path still always
      creates a new Work (Decision 1) · ② a batch whose language disagrees with the
      destination's `source_lang` is split by the wrong rules and nothing says so
      (Decision 2). Both need a real `Chủ:` — `check:debt-owner` rejects "chưa có chủ".
- [x] `_bmad-output/planning-artifacts/epics.md` (`:4861`) and
      `sprint-change-proposal-2026-09-15.md` (`:52`) -- `AD-44` → `AD-8` with 🔵 and the date
      (Decision 4). Touch only those two citations; change nothing else in either file.

**Acceptance Criteria:**

- Given a destination Work W with M Chapters and an import of N Chapters, when confirmed, then
  W holds M+N Chapters and **no** new Work exists anywhere in the Library index.
- Given the same run, when every pre-existing `chapter` and `segment` row of W is compared
  before and after, then each is identical field for field — `ord` and asset anchors included.
- Given the new append path, when step four (`reindex_after_lifecycle_write`) is **removed**
  and the suite is re-run, then a case goes **red**. The 2026-08-27 measurement recorded 0
  failures for exactly this removal; a green run here means the seam is still unguarded.
- Given the new cleanup-tier case, when it is **removed** and the **old** suite is run, then
  the old suite is **green** — proving the mismatched-Work proposition had no guard before.
- Given an import preview that has not been confirmed, when `project.db`, `meta.json` and
  `library-index.db` of the destination are compared byte for byte, then all three are
  unchanged.
- Given the Rust and vitest baselines recorded in Phase 1, when both suites run after the
  story, then both are green with **no expectation loosened**; `project.db` `schema_version`
  is still **22**; `PIPELINE_ORDER` is unchanged.
- Given `npm run check:deps && check:i18n && check:tokens && check:commands && check:layout &&
  check:panel-refs && check:gates && check:debt-owner`, when run after the story, then 0
  findings per gate.
- Given the nightly e2e `schedule` run, when read before writing `done`, then its result is
  recorded here — green, or red with the reason (`AGENTS.md:37`).

## Implementation Notes

### Phase 1 — MEASURE (2026-09-16, on `7fb3aa7202044aefda0d1a0b40d5bf4cbef9276d`)

**Measurement 1 — real baseline (run counts, not source counts).**

- `npm run build` — green. `vue-tsc --noEmit` (both `tsconfig.json` and
  `tsconfig.node.json`) clean, `vite build` produced `dist/` in 1.06 s. Two pre-existing
  `INEFFECTIVE_DYNAMIC_IMPORT` warnings (unrelated to this story — `@tauri-apps/api/core.js`
  and `event.js` both statically and dynamically imported) and one chunk-size-over-500-kB
  advisory; neither is an error and neither is touched by this story.
- `cargo test --locked` (run from `src-tauri/`) — green. Aggregated from every
  `test result:` block in the run: **1536 passed, 0 failed, 20 ignored**, across **56**
  test binaries (58 `test result:` lines — some binaries report a unit-test block and an
  integration block separately, plus the two doc-test summaries). This is a RUN count, and it
  is higher than the Code Map's 1357-source-`#[test]`/25-`#[ignore]` figure because that
  earlier count only grepped `src-tauri/tests/` — this run also includes `src/` unit tests and
  doc-tests. Ignored count (20) differs from the source count (25) for the same reason: the
  25 lives only under `tests/`, this 20 is the ignored count actually reachable in this run
  (some `#[ignore]` cases are gated by `#[cfg(...)]`/feature flags not enabled here).
  Full log kept at
  `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/4a5ee48b-42d7-418e-b01f-fa955e1b5da4/scratchpad/cargo_test_run1.log`
  (session-scoped scratch, not part of the repo).
- `npm run test` (vitest) — green. **76 test files passed (76), 1043 tests passed (1043)**, 0
  failed. Duration 107.11 s. This is the real run count against which the Code Map's
  "17 files / 232 cases touch the import preview" and "986 total `it`/`test` call sites"
  (a static grep of call sites, not a run count either — some `it.each`/parameterised blocks
  expand to more than one run) are read; do not quote 986 as the bar, quote 1043.
- Bar for the rest of this story: **cargo 1536 passed / 0 failed / 20 ignored**, **vitest 76
  files / 1043 tests passed**, both green, dated 2026-09-16 on `7fb3aa7`. Later phases compare
  against these numbers, not against 1357/986.

**Measurement 2 — counter-check target for AC3 is real.**

Method: `resolve_cleanup_rules` (`src-tauri/src/commands/project/wire.rs:21-38`) is a private
function inside `mod wire` taking `&tauri::AppHandle`; this crate carries no
`tauri::test`/`MockRuntime` harness (confirmed by grep and by the comment at
`tests/project_contract.rs:1071` — no test anywhere calls a `#[tauri::command]` wire shell
directly with a constructed `AppHandle`). Its body has exactly one branch when
`OpenWorkState` holds a Work: `Some(open) => resolve_cleanup_rules_against(&open.scope,
global, Some(&open.store))`, which calls straight through to
`core::cleanup::resolve_two_tiers(resolver, global, work)` — a `pub` pure function. There is
no destination parameter anywhere in this call chain (confirmed separately: `grep -n
"resolve_cleanup_rules" wire.rs` shows the function is called with only `&app` at all twelve
call sites, `:477`/`:553` included — no argument ever names which Work a preview is being
built for).

Reproduced by calling `core::cleanup::resolve_two_tiers` directly with the exact argument
shape `resolve_cleanup_rules` passes when a Work is open — a throwaway test
(`src-tauri/tests/zz_scratch_probe_ac3.rs`, written, run, and deleted; not committed, not
part of the tree):
1. Created a real Work **X** via `commands::project::create_work`.
2. Added a Work-tier cleanup rule to **X only**, via `commands::cleanup::cleanup_add_rule(...,
   Some(&work_x), CleanupRuleTier::Work, "RULE_ONLY_ON_X", ...)`.
3. Called `resolve_two_tiers(&work_x.scope, &global, Some(&work_x.store))` — byte-for-byte the
   call `resolve_cleanup_rules` makes when `OpenWorkState` holds X, regardless of what any
   preview is being built for (today, every preview targets "New Work", i.e. a destination
   that is never X).
4. Result: **1 rule returned, tier `Work`, pattern `"RULE_ONLY_ON_X"`** — X's rule is present.

`cargo test --locked --test zz_scratch_probe_ac3 -- --nocapture` output (2026-09-16):
```
DA DO (2026-09-16): resolve_two_tiers voi scope/store cua X dang mo tra ve 1 luat, trong do co
luat tang Tac pham 'RULE_ONLY_ON_X' cua X -- CHUNG MINH khuyet tat AC3 canh la CO THAT: mot
man xem truoc nhap dung cho mot dich khac X van hop nhat luat cua X, vi resolve_cleanup_rules
khong nhan tham so dich.
test today_the_open_works_cleanup_rules_leak_into_a_preview_built_for_a_different_destination
... ok
```

**Conclusion: the counter-check target is real — AC3 guards a genuine defect, not a
hypothetical one.** `resolve_cleanup_rules` merges whatever Work `OpenWorkState` happens to
hold into every import preview, unconditionally, with no way today to say "but the preview is
for a different Work." Phase 2's task (`wire.rs` `resolve_cleanup_rules` resolving from the
destination instead of `OpenWorkState`) and Phase 4's new `cleanup_contract.rs` case are
confirmed as guarding a real, reproduced defect, not a paper one. This finding is consistent
with — and adds a direct behavioural reproduction to — the debt entry at
`deferred-work.md:10063-10081`, which already states in words "nếu một Tác phẩm KHÁC tình cờ
đang mở cùng lúc — đính luật vào `project.db` của Tác phẩm đó, im lặng" and "Nửa ĐỌC/HỢP NHẤT
của tầng Tác phẩm không đổi — `ScopeResolver::apply_merge` vẫn hợp nhất cả hai tầng bình
thường khi có Tác phẩm mở."

### 🔵 2026-09-16 — Measurement 2's method corrected (orchestrator re-check)

The **conclusion** above stands; the **evidence offered for it** does not, and the distinction
matters for Phase 4.

- What holds, re-measured independently: `fn resolve_cleanup_rules(app: &tauri::AppHandle)`
  (`wire.rs:21`) takes exactly one parameter, and all **12** call sites in `wire.rs` pass
  `(&app)` and nothing else (`:477`, `:553`, `:637`, `:749`, `:808`, `:870`, `:942`, `:984`,
  `:1043`, `:1105`, `:1167`, `:1242`). A destination is **not expressible** anywhere in this
  chain. That is a fact at the level of the signature — the defect is real by construction and
  needs no run to establish.
- What does **not** hold: the throwaway probe called `core::cleanup::resolve_two_tiers`
  directly, handing it X's `scope` and X's `store`, and observed X's rule come back. That is
  `resolve_two_tiers` doing its job. It bypasses the function under suspicion, so it would
  return X's rule **whether or not** the defect exists — an assert that holds on both branches
  guards neither (`AGENTS.md:68`). Calling it a "behavioural reproduction" overstates it; it
  is a restatement of the static reading, not a second, independent source of evidence.

🔴 **Consequence for Phase 4 — do not plan the new gate as a call to `resolve_two_tiers`.**
That shape is the same inert probe with a test name on it. The crate has no
`tauri::test`/`MockRuntime` harness, so the wire shell cannot be called directly either. The
way out is the project's own mandated two-layer IPC shape (`src-tauri/AGENTS.md:13`): give the
**pure** function an explicit destination argument and let the thin `wire` shell pass it in.
The new `cleanup_contract.rs` case then calls that pure function with a destination that
differs from the Work in `OpenWorkState` — a distinction today's signature cannot even
express, which is why the case is red before Phase 2 and green after. Counter-check it by
removing the destination argument and confirming the case fails to compile or goes red.

### Phase 2 — Rust append path (2026-09-16, on top of `7fb3aa7` + Phase 1's commits)

**What was built.**

- `src-tauri/src/commands/project/mod.rs::append_chapters_to_work` — the sibling of
  `create_work` the task list asked for. Takes `open: &mut OpenWork` (already resolved —
  caller's job, see below) plus the same pipeline parameters `create_work` takes
  (`source_lang`, `shape`, `encoding`, `cleanup_rules`, `chapter_pattern`, `block_overrides`,
  `origin_overrides`, `domain_log_state`, `docx_sidecar`), minus every `bilingual_*`/
  `regroupings` parameter (Decision 1 scopes this story to the three monolingual paths; the
  function's own doc-comment states `shape` is never `PipelineShape::Bilingual` on this call
  path). Internals mirror `create_work` line for line through the pipeline/image/weave phases
  (`run_pipeline` — the crate's one sanctioned `run_import` call site, unchanged — then
  `prepare_chapter_images`, then the role-weaving loop) and diverge only at the write step:
  no `create_work_folder`, no `Store::open`, no `INSERT INTO work`, and **no error branch
  calls `remove_folder`** (§Never). The write step reads `MAX(ord)` inside the same
  transaction as the inserts (`SELECT COALESCE(MAX(ord), 0) FROM chapter`) and writes new
  chapters at `base_ord + i + 1`, so old rows are never touched (SQL only appends), and the
  `NotStarted` status is fixed rather than branching on `bilingual_segments` (which is always
  `None` here — see doc-comment for why the branch was simplified away instead of copied
  dead). `open.chapter_id` is left untouched (the open editor's chapter does not change);
  `open.images_saved`/`images_failed` are overwritten with *this call's* counts, matching
  `create_work`'s own "this call's images, not a lifetime counter" semantics.
- `append_chapters_to_work`'s failure returns leave `open` exactly as it was — the pipeline/
  image/weave phases run *before* the SQL transaction and touch nothing on `open`; a
  transaction `Err` rolls back naturally through `Store::write`. ⚠️ **One accepted, documented
  gap**: images are fetched and written to `assets/` *before* the SQL transaction (same order
  `create_work` uses). If the pipeline/weave phases succeed but the SQL transaction itself
  then fails (a real but rare case — the only scenarios reaching that far are a `CHECK`
  violation already filtered pre-transaction, or a mid-transaction disk-full), the already-
  written image files become orphans with no `asset` row, because — unlike `create_work` —
  this function is explicitly forbidden from deleting anything. This is the direct, accepted
  cost of §Never's "failure means leaving the Work alone, never deleting it"; recorded here
  rather than silently accepted.
- `confirm_append_import_with_encoding` — same shape as `confirm_import_with_encoding`
  (resolve encoding, lock `PendingImportSourceState`, clone `shape`/`docx_sidecar`, clear the
  pending slot only on success) but calls `append_chapters_to_work` + the existing
  `commands::lifecycle::write_lifecycle_after_change` (steps 2–3 of the four-step template)
  instead of `create_work`. Step 4 (`reindex_after_lifecycle_write`) deliberately stays out of
  this function and runs at the wire layer, after the `OpenWorkState` lock (if any) is
  released — same discipline as `commands::lifecycle::wire::set_chapter_status`.
- ~~`wire::confirm_import_with_encoding` gained one new trailing parameter,
  `destination_work_id: Option<String>`.~~ 🔵 **SỬA 2026-09-16 (coordinator review) — không còn
  đúng, xem mục "2026-09-16 correction" ngay dưới**: `confirm_import_with_encoding`'s parameter
  list is now byte-for-byte unchanged from before this story; it reads the destination from
  `PendingImportSourceState` instead. The resolution logic described next is unchanged in
  substance, only in *where the `work_id` value comes from*.
  `Some(work_id)` (now sourced from `super::current_pending_destination(&pending_state)`)
  resolves the destination two ways, matching the task's instruction: if `OpenWorkState`
  already holds that exact `work_id`, the append runs against a `&mut OpenWork` borrowed
  straight out of the held `MutexGuard` (no second `Store::open` on the same `project.db` —
  `src-tauri/AGENTS.md:30`); otherwise it resolves fresh through `Indexer::find_work` +
  `open_work` (re-run at confirm time, not trusted from preview — see the "destination
  vanished" case below) and, on success, installs the result via the existing
  `replace_open_work`. On a resolution or append failure in the "not already open" branch,
  the freshly-opened `OpenWork` is simply dropped (its `Drop` closes the `Store` — confirmed
  in `core/store/mod.rs::impl Drop for Store` — idempotent with the explicit `.close()` calls
  used elsewhere in this file); nothing on disk is touched beyond what the failed transaction
  already rolled back.
- `wire::resolve_cleanup_rules_for_destination` (Task 3, AC3) — the Work tier for a specific
  destination, never from `OpenWorkState`. Reuses the open `Store` if the destination is the
  Work already open; otherwise resolves through `Indexer::find_work` + `open_work` (same
  `db_path.exists()` guard as `open_work` itself, so a destination missing its `project.db`
  cannot be silently created by `Store::open`'s `SQLITE_OPEN_CREATE` flag — that would be a
  disk write before confirmation) and closes the temporary `Store` immediately after reading.
  Errors fall back to 0 rules with a diagnostic, matching `resolve_cleanup_rules`'s existing
  best-effort philosophy (a broken preview affordance must not crash the whole preview).
- ~~**Design choice not in the literal task wording, made and recorded here**: ... Only
  `preview_import_encoding_from_text`, `preview_import_encoding_from_file`,
  `start_url_import`, and `confirm_import_with_encoding` call the new function — exactly
  Decision 1's three monolingual entry points, plus confirm.~~ 🔵 **SỬA 2026-09-16
  (coordinator review) — INCOMPLETE, not wrong in mechanism, wrong in scope. See "2026-09-16
  correction" below**: `resolve_cleanup_rules_for(app, destination)` still exists exactly as
  built (kept for exactly the reason stated: it avoids widening `resolve_cleanup_rules(app)`
  itself and breaking the bilingual wires' literal-substring test), but it is now called from
  **nine** sites, not four — every wire of the three monolingual sessions that resolves cleanup
  rules for a preview the user can see, not only the three entry points. The bilingual
  three-call-site count and reasoning below stand unchanged.
- ~~**Design choice, also recorded**: the destination is carried as a plain per-call parameter
  (`destination`/`destination_work_id: Option<String>`) on the three preview entry points and
  on `confirm_import_with_encoding`, **not** as a new field stashed on `PendingImportSource`.~~
  🔵 **SỬA 2026-09-16 (coordinator review) — REVERSED, not merely incomplete.** The reasoning
  quoted below (re-read fresh at confirm, matching `cleanup_rules`/`chapter_pattern`) is still
  correct for values that can genuinely change *between calls within a session* — but a
  destination cannot: no screen lets the user re-pick it mid-session, so it is not a per-call
  value at all, it is session state, exactly like `shape` itself. That distinction was missed
  on the first pass and is what the coordinator's review caught. See "2026-09-16 correction"
  below for what changed and why. The original paragraph, kept for the record: "the
  destination is carried as a plain per-call parameter (`destination`/
  `destination_work_id: Option<String>`) on the three preview entry points and on
  `confirm_import_with_encoding`, not as a new field stashed on `PendingImportSource`. ...
  Reason: every other per-call value on this exact command (`cleanup_rules`,
  `block_overrides`, `origin_overrides`, `chapter_pattern`) is already documented and enforced
  to be re-read fresh at confirm time, never trusted from the preview-time cache ... Reusing
  that exact, already-proven pattern for `destination` avoids adding a struct field, a fourth
  `stash_pending_import_source` parameter, and a 'preserve across `reload`/`remove`' plumbing
  problem for the URL session."
- **Task 4 (new `MessageKey` variants) was measured, not implemented as literally worded.**
  Grepped `core/i18n/mod.rs`: `WorkMetaTooNew` (`err.work.meta_too_new`), `WorkOpenFailed`
  (`err.work.open_failed`), and `LibraryWorkNotIndexed` (`err.library.work_not_indexed`)
  already exist (Story 5.7) and are exactly what `open_work`'s error mapping produces for
  "schema too new" / "unreadable for any other reason" / "not in the index" respectively.
  Because destination resolution in Phase 2 goes through `open_work` unconditionally (both at
  preview-time best-effort cleanup-rule resolution and at confirm-time hard resolution), these
  three conditions from the frozen I/O matrix ("Destination vanished" ⇒ named error, not a
  panic; "Destination unreadable" ⇒ "reuse `WorkError::OpenFailed`/`MetaTooNew`" — the matrix
  says *reuse*, explicitly) are already satisfied without a new key. Minting three unused
  variants nobody constructs would itself violate the project's own "a key for a branch no
  call site reaches" rule (`AGENTS.md` Story 1.7 §Completion Notes precedent, cited verbatim
  elsewhere in this same spec file). No `core/i18n/` edit was made for this task.
- **Task 5 (register a new command)**: measured as a no-op. Zero new `#[tauri::command]`
  functions were added — the destination feature rides on new *parameters* to four already-
  registered commands. `generate_handler!` in `lib.rs` is unchanged; confirmed no edit was
  needed by re-reading it after all other changes landed.

**Verification.**

- `cargo check --locked` and `cargo check --locked --tests` — both clean, 0 warnings, 0
  errors, including every test binary (so no pure-function signature change broke an existing
  test call site).
- `npm run build` — green, same two pre-existing `INEFFECTIVE_DYNAMIC_IMPORT` warnings as the
  Phase 1 baseline, no new ones.
- `cargo test --locked --no-fail-fast` (56 binaries + 2 doc-tests, summed from every
  `test result:` line): **1534 passed, 2 failed, 20 ignored** — **1536 total**, matching the
  Phase 1 baseline's total (1536) exactly, both before and after the 2026-09-16 correction
  below (re-measured after the correction; same two names, same counts). The 20 ignored count
  also matches. The 2 failures are both in `tests/ipc_contract.rs`:
  `the_three_import_encoding_preview_wires_are_registered_and_keep_their_parameter_names` and
  `the_three_url_import_wires_are_registered_and_keep_their_parameter_names`. Both fail for
  the *expected* reason: they lock the exact rendered parameter list of
  `preview_import_encoding_from_text`/`preview_import_encoding_from_file`/`start_url_import`
  as a literal string, and this phase added one trailing parameter to each. (`confirm_import_
  with_encoding`'s own parameter-list assertion is unaffected after the correction below —
  its signature ended up unchanged from before this story, since it now reads the destination
  out of `PendingImportSourceState` instead of taking a new parameter.) This is precisely the
  gap Phase 4's task list already names
  ("`ipc_contract.rs` -- lock the changed/added command parameter lists through
  `fn_param_list`, following the worked instance at `:1046-1131`") — left unfixed here on
  purpose, since editing `tests/**` is Phase 4's scope, not Phase 2's. No other test file
  regressed: every other of the 56 binaries stayed 100% green, including
  `the_three_bilingual_import_wires_are_registered_read_cleanup_rules_and_rebuild_never_reads_the_file`,
  which *would* have gone red too under a naive single-function design — see the
  `resolve_cleanup_rules`/`resolve_cleanup_rules_for` split above.
- `npm run test` (vitest) — **76 files / 1043 tests, all passed**, byte-identical to the Phase
  1 baseline. Expected: zero files under `src/` were touched in this phase.
- `npm run check:i18n` — green (Kiểm A through E), confirming every new string literal added
  in Rust (`eprintln!`/error `detail` strings in `append_chapters_to_work`,
  `resolve_cleanup_rules_for_destination`, and the new `confirm_import_with_encoding` branch)
  carries no diacritics, per the file-header rule both touched files declare.
- Not run this phase (no relevant files touched, so not expected to move):
  `check:deps`/`check:tokens`/`check:layout`/`check:commands`/`check:panel-refs`/
  `check:debt-owner` (all frontend- or debt-ledger-scoped; Phase 2 touched neither `src/**`
  nor `deferred-work.md`) and the nightly e2e schedule (out of this phase's control; Phase 4's
  AC explicitly asks for it to be read before writing `done` on the whole story, not per
  phase).

**Left incomplete / risky, for Phase 3 and Phase 4 to pick up.**

- The two `ipc_contract.rs` failures above are real and expected — Phase 4 must update both
  `fn_param_list` expectations (and add a case for the new `destination`/`destination_work_id`
  parameter, following the spec's own instruction to follow the worked instance at
  `:1046-1131`).
- No Glossary scan is spawned on the append path. `confirm_import_with_encoding`'s New-Work
  branch spawns `spawn_import_scan` against the newly-created Work's first chapter; the append
  branch does not spawn anything, because the only chapter id available cheaply
  (`open.chapter_id`) is explicitly *not* the newly appended content (§I/O Matrix: "open
  editor state stays valid") — scanning it would rescan old, already-scanned text, not the new
  chapters. Whether/how to scan the newly appended chapters for Glossary candidates is an open
  product question no AC in this story asks for; flagging it here rather than guessing.
- ~~`resolve_cleanup_rules_for` is wired into exactly the three monolingual preview/URL entry
  points plus confirm — not into `reload_url_import_item`, `remove_url_import_item`,
  `tier2_block_set_kept`, `tier2_block_confirm_range`, or `preview_chapter_detail`. ...~~
  🔵 **CLOSED 2026-09-16 (coordinator review) — see "2026-09-16 correction" below.** All five
  now resolve cleanup rules from the destination too. Left here, struck rather than deleted,
  because the gap was real when first written and the fix is worth being able to find.
- The orphan-image-file risk on a rare mid-transaction failure (documented in
  `append_chapters_to_work`'s doc-comment and above) is accepted, not mitigated — a `find`
  against `library-index.db`'s asset rows to clean up truly orphaned files is out of this
  phase's scope and not requested by any AC.
- Phase 2 does not modify `deferred-work.md` or `epics.md`/`sprint-change-proposal-2026-09-15.md`
  — those are explicitly Phase 4 tasks (debt entries and the AD-44→AD-8 citation fix).
- Untouched, as instructed: Phase 3 (webview) and Phase 4 (tests that move) were not started.
  No `src/**`, `tests/frontend/**`, or `src-tauri/tests/**` file was edited in this phase.

### 🔵 2026-09-16 — coordinator review: AC3 was not actually closed, five seams fixed

**What was wrong.** The first Phase 2 pass wired `resolve_cleanup_rules_for(app, destination)`
into exactly four wires: the three monolingual entry points
(`preview_import_encoding_from_text`/`_from_file`, `start_url_import`) plus
`confirm_import_with_encoding`. Five more wires that rebuild or re-render the SAME preview
mid-session — `reload_url_import_item`, `remove_url_import_item`, `tier2_block_set_kept`,
`tier2_block_confirm_range`, `preview_chapter_detail` — were left calling the one-argument
`resolve_cleanup_rules(app)`, still resolving the Work tier from `OpenWorkState`. The
coordinator counted all twelve `resolve_cleanup_rules*` call sites in `wire.rs` and named this
directly: reloading a URL item or nudging a tier-2 block boundary after picking an existing-Work
destination would show the *open* Work's cleanup rules on screen while the eventual confirm
writes with the *destination*'s rules — a preview that disagrees with the write it is
previewing, exactly the silent-corruption shape FR124 exists to prevent. The frozen AC3 says
the Work tier is the destination's when the preview applies cleanup rules, full stop — not "on
first render only." This was a real gap in Phase 2's own job, not something deferrable to
Phase 4.

**Why the earlier "per-call parameter" choice didn't survive contact with this.** The first
pass judged the plain per-call parameter, resent by the frontend at every call, correct because
`cleanup_rules`/`block_overrides`/`origin_overrides`/`chapter_pattern` all follow that pattern.
But those four values *can* change between two calls in the same session (the user can toggle a
cleanup rule, then reload a URL item — the rules must be read fresh). A destination cannot: no
screen in this product lets the user re-pick the destination mid-session, so it is a property of
the *session*, not of the *call*. Once five more callers inside the same session needed the
identical value with no natural parameter of their own to carry it on, the per-call design would
have meant adding a `destination` parameter to five more `#[tauri::command]` functions just to
thread through a value that never changes across them — five parameters carrying one constant is
the "chép năm lần một thứ vốn chỉ có MỘT giá trị đúng cho cả phiên" the codebase already has
words for (`AGENTS.md`'s "a hand-copied second instance is the exact defect" reasoning,
transplanted from steps-of-a-template to values-of-a-session). That is what changed the design,
not a change of heart about the re-read-fresh principle itself — that principle still holds for
`cleanup_rules` etc.

**What changed, concretely.**

- `PendingImportSource` (`commands/project/mod.rs`) gained `pub destination_work_id:
  Option<String>` — the destination lives on the pending-import session state, exactly where
  `shape` already lives, for exactly the same reason (both are set once when the session opens
  and read unchanged for the rest of the session).
- `stash_pending_import_source(state, shape, docx_sidecar)` — **kept its original three-argument
  signature and behaviour unchanged** (sets `destination_work_id: None` internally). This was a
  deliberate second design correction made *while* fixing the first: an initial attempt added
  `destination_work_id` as this function's fourth parameter directly, which broke compilation
  (not merely a test assertion) of four existing test binaries —
  `tests/cleanup_contract.rs`, `tests/bilingual_import_contract.rs`, `tests/segment_contract.rs`,
  `tests/story_6_18_library.rs` — because it is a `pub fn` called directly by 35+ existing test
  call sites (the two-layer architecture's whole point: pure functions are callable without a
  `tauri::AppHandle`). Widening a function's arity that tests already call directly is exactly
  the kind of `tests/**`-touching edit Phase 2 was told to avoid, and doing it just to add a
  fourth argument those tests don't care about would have meant editing 35+ unrelated call sites
  for no behavioural reason. Fixed the same way `resolve_cleanup_rules`/`resolve_cleanup_rules_for`
  was already split: a new `stash_pending_import_source_for(state, shape, docx_sidecar,
  destination_work_id)` sibling carries the new argument; the original function delegates to it
  with `None`. Zero test files were edited to land this correction.
- New pure fn `current_pending_destination(state: &PendingImportSourceState) -> Option<String>`
  — read-only, clones the currently-stashed destination (or `None`). This is what every
  mid-session wire and `confirm_import_with_encoding` now call instead of taking a parameter.
- `sync_pending_from_url_items` (private to `mod.rs`, so free to change arity — no test calls it
  directly) gained a `destination_work_id: Option<String>` parameter. `start_url_import` passes
  the *newly chosen* value (opening a session); `reload_url_import_item`/`remove_url_import_item`
  read the *existing* value via `current_pending_destination` **before** calling this function
  (not after — the function can clear the pending slot entirely if the refreshed list has no OK
  items left, and reading after would then wrongly observe `None`) and pass it straight through
  unchanged.
- `wire::confirm_import_with_encoding` — the `destination_work_id: Option<String>` parameter
  added in the first pass was **removed**; its signature is now byte-for-byte what it was before
  this story started. It reads the destination via `current_pending_destination(&pending_state)`
  right after obtaining `pending_state`, before anything else. This is a strictly better design
  than "keep both a stashed value and a resent parameter that might disagree": there is now
  exactly one source of truth for a session's destination, and it happens to also shrink the
  ipc_contract blast radius (see the updated failure count below).
- `wire::tier2_block_set_kept`, `tier2_block_confirm_range`, and `preview_chapter_detail` did not
  previously fetch `PendingImportSourceState` at all; each gained a `pending_destination(app)`
  best-effort helper call (new, in `wire.rs`: `app.try_state::<PendingImportSourceState>()`
  `.and_then(current_pending_destination)`, `None` if the state isn't managed — same
  wiring-failure tolerance every other state lookup in this file already has).

**Measurement 1 — the count, after the fix, in the coordinator's own shape.** Grepped
`resolve_cleanup_rules(&app)`/`resolve_cleanup_rules_for(&app` across `commands/project/wire.rs`
(12 real call sites, confirmed by mapping each line to its enclosing `pub fn` with an `awk`
pass, not by eyeballing):

- **9 are destination-aware** (`resolve_cleanup_rules_for`): `preview_import_encoding_from_text`,
  `preview_import_encoding_from_file`, `confirm_import_with_encoding`, `start_url_import`,
  `reload_url_import_item`, `remove_url_import_item`, `tier2_block_set_kept`,
  `tier2_block_confirm_range`, `preview_chapter_detail`.
- **3 are bilingual and legitimately out of scope** (`resolve_cleanup_rules`, one-arg, unchanged):
  `preview_bilingual_import_from_file`, `rebuild_bilingual_import_preview`,
  `confirm_bilingual_import`. Decision 1 excludes the bilingual import path from this story
  entirely (Story 6.16b owns it); these three still resolve rules the way they did before this
  story existed, and the literal-substring test that guards their wiring
  (`the_three_bilingual_import_wires_are_registered_read_cleanup_rules_and_rebuild_never_reads_the_file`)
  stays green because their source text is untouched.
- **0 remain one-arg for any other reason.** Every monolingual-session call site is now
  destination-aware; the count that was "4 of 12" in the coordinator's message is now "9 of 12,"
  with the other 3 being the bilingual wires named as out-of-scope from the start.

**Measurement 2 — counter-check, as requested.** Reverted exactly one of the five newly-fixed
call sites — `reload_url_import_item`'s `resolve_cleanup_rules_for(&app, destination.as_deref())`
back to the one-arg `resolve_cleanup_rules(&app)` — recompiled (`cargo check --locked --tests`,
clean), and ran the full suite (`cargo test --locked --no-fail-fast`, 56 binaries). Result:
**identical to the fixed state** — the same 1534 passed / 2 failed / 20 ignored, the same two
named `ipc_contract.rs` failures, nothing else moved. **Plainly: nothing in the existing suite
catches this regression.** No `cleanup_contract.rs` case exercises `reload_url_import_item`
against a mismatched-destination scenario today — the existing mismatched-Work coverage (Phase
1's Measurement 2, and whatever Phase 4 builds from it) is written against
`resolve_two_tiers`/the confirm path, not against the five mid-session preview-rebuild wires.
This seam is real and, as of this correction, closed in the PRODUCT CODE — but it is **unguarded
by any test that exists today**. The change was reverted immediately after this measurement
(confirmed via `git diff` showing zero remaining occurrences of the temporary marker before
re-verifying the full suite one more time). **Phase 4 has to build a case for this** — the spec's
own Phase 4 task ("the mismatched-Work case that exists nowhere today") should be read as
covering these five wires too, not only `confirm_import_with_encoding`'s write path, or the
"preview disagrees with the write" defect this correction just closed in code will still be
invisible to `cargo test`.

**Verification after the full correction** (re-run in this order: `cargo check --locked --tests`
clean → `cargo test --locked --no-fail-fast` → `npm run build` → `npm run check:i18n`):

- `cargo test --locked --no-fail-fast`: **1534 passed, 2 failed, 20 ignored** (1536 total) —
  identical numbers to the pre-correction measurement above, and to the Phase 1 baseline total.
  The 2 failures are the same two named tests, now for a narrower reason than before the
  correction: only `preview_import_encoding_from_text`, `preview_import_encoding_from_file`, and
  `start_url_import` gained a parameter Phase 4 needs to lock — `confirm_import_with_encoding`'s
  entry in that same test is unaffected because its signature reverted to its pre-story shape.
- `npm run build`: green, same two pre-existing warnings.
- `npm run check:i18n`: green — the new `pending_destination`/`current_pending_destination`
  Rust code carries no display strings beyond what was already checked.
- `npm run test` (vitest) was not re-run for this correction specifically (no `src/**` touched by
  it either); the Phase 2 measurement above still holds unchanged.

### Phase 3 — Webview (2026-09-16, on top of Phase 1+2's uncommitted tree)

**Placement decision made and recorded before writing code — the Code Map's line reference for
`ImportPreviewOverlay.vue` (`:824-846`, "header with name and source-lang fields") does not match
the file on disk.** Read the actual lines: `:824-846` on the pre-Phase-3 file is `<h2>` title +
one close button — no name/source-lang fields exist there, nor anywhere else in the file (grepped
`pendingName|pendingSourceLang|pendingGenre|ip-name|ip-genre` — zero hits). Cross-checked against
the mockup this task cites (`web-import.html:190-207`): the "Đưa vào" radios + "Tên Tác phẩm" +
"Ngôn ngữ nguồn" column sits on the **pre-fetch** "Nhập từ website" screen (before the "Tải" /
submit button), not on the later "Màn xem trước hợp nhất" screen that `ImportPreviewOverlay.vue`
actually renders (read that screen's full markup, `:230-390` of the mockup — no destination/name/
source-lang column anywhere in it; the layout is chapnav + four tiers + footer).

A second, independent, structural reason forces the same conclusion, not just the mockup: `source_lang`
is a parameter of the very **first** preview-building call (`previewImportEncodingFromText`/
`_FromFile`/`startUrlImport`) — Decision 2 requires it to already be the destination's adopted
language, not detected, when the destination is an existing Work. That means the destination must
be resolved **before** that first call, not after the overlay (whose earliest possible existence
is the return of that same call) is on screen. And for the URL path specifically, Phase 2 built
`start_url_import` to take `destination` on its **one and only** call — `reload_url_import_item`/
`remove_url_import_item` read the already-stashed destination back and cannot accept a new one
(confirmed by re-reading their Rust signatures: `(app, index, source_lang)`, no `destination`
parameter). Changing the destination after a URL session has fetched would require a second
network fetch to re-run the pipeline under Rust's own AD-40/41 "one wave of fetches" rule — which
Phase 3 cannot add (webview-only scope) and which the frozen §Always section forbids regardless.

**Conclusion, built into the code below:** the two radios + existing-Work picker live in
`src/modes/LibraryMode.vue`'s pre-submit `import-form` (matching the mockup exactly, and the only
place structurally compatible with all three monolingual paths including URL). Task 2's "setter;
changing it rebuilds the preview through `reloadImportPreviewAfterRuleChange`" is satisfied by a
narrower, verified-necessary mechanism instead of a live mid-overlay re-pick: `previewImportEncodingFromText`/
`_FromFile` **re-stash the whole `PendingImportSource` on every call**, including the reload calls
`reloadImportPreviewAfterRuleChange`'s internals already make on every cleanup-rule CRUD or
chapter-pattern edit (`runImportPreviewReload`, `runFileImportPreviewReload`) — so the destination
had to be threaded through those reload call sites too, or the very first cleanup-rule toggle
after opening the preview would silently reset a chosen existing-Work destination back to "New
Work" server-side, with nothing on screen showing it happened. That bug-shaped gap is exactly what
carrying `pendingDestinationWorkId` through those two functions (plus `removeImportPreviewFileItem`,
a third caller of the same Rust command) closes. `ImportPreviewOverlay.vue`'s header carries a
**read-only** echo (`Đích: …` / adopted source-lang line) built from `pendingName`/`pendingSourceLang`
(now exported readonly) — satisfying Decision 2's "shown read-only" half without pretending a
control exists where the backend cannot honor a change.

**What was built, file by file.**

- `src/config/project.ts` — `previewImportEncodingFromText`, `previewImportEncodingFromFile`, and
  `startUrlImport` each gained a trailing `destinationWorkId: string | null` parameter, sent on
  the wire as `destination` (matches the Rust parameter name exactly — `destination`, no
  camelCase/snake_case split to bridge since it is a single word). `confirmImportWithEncoding` is
  **byte-for-byte unchanged** — Rust reads the destination from `PendingImportSourceState`, not
  from a parameter (Phase 2's 2026-09-16 correction). No new `isXxx()` runtime validator was
  added for `destination`: it is outbound-only (never a field of anything Rust *returns*), already
  narrowed to `string | null` at every TypeScript call site, and none of the three functions'
  return-shape validators (`isImportEncodingPreview`, `isFileImportBatchWire`,
  `isUrlImportBatchWire`) changed shape. Minting an unused `isDestination()` predicate nobody
  calls would repeat the exact "a key/validator for a branch no call site reaches" mistake Phase 2
  named and declined to make for Task 4's `MessageKey` variants.
- `src/importPreviewState.ts` — new module-private `pendingDestinationWorkId` ref, set once per
  session by `openImportPreviewFromText`/`_FromFile`/`_FromUrls` (new trailing parameter on all
  three, threaded through `openWith` for the text branch) and re-sent, unchanged, by every
  function that rebuilds the preview against the same session: `runImportPreviewReload`,
  `runFileImportPreviewReload`, `removeImportPreviewFileItem`. Exported read-only as
  `importPreviewDestinationWorkId`, plus `importPreviewPendingName`/`importPreviewPendingSourceLang`
  (the two other already-existing-but-unexported pending fields the overlay needed to render its
  read-only echo — `pendingGenre` was not exported, since neither the overlay display nor any AC
  needed it there). Added to `resetImportPreview()` — 48th slot swept, verified by
  `check:panel-refs` passing (see Verification).
- `src/modes/libraryImport.ts` — `destinationMode: 'new' | 'existing'`, `destinationWorkId`,
  `destinationWorks` (an **unfiltered** `listLibraryWorks()` result — deliberately *not* a reuse
  of `libraryWorks.ts`'s `libraryWorks`, which is filtered by the grid's own status/genre/
  source-lang filters and would make a Work silently disappear from the destination picker for a
  reason unrelated to the picker, the exact AD-1 class of bug that ref's own doc-comment already
  warns against for `genres`/`source_langs`), `destinationWorksHaveLoaded`, `destinationWorksError`,
  and three derived computeds — `pickedDestinationWork`, `effectiveSourceLang`/`effectiveName`/
  `effectiveGenre` (destination's own values when picked, the typed form values otherwise — §Never
  spec 6.7b: the destination's name/genre/source_lang are never sent as if editable).
  `loadDestinationWorks()` fires from `LibraryMode.vue`'s `onMounted` (not lazily on first radio
  click) because the I/O Matrix's "Empty Library" row requires the control to say why it is
  unavailable from the moment the form renders, not only after a first click. `destinationPending`
  (exported computed) is the single source of truth both the three submit functions' guard clause
  and the three submit buttons' `:disabled` read — one predicate, not a duplicated one in the
  `.vue` and one in the `.ts`. All destination refs use `export const … = ref(...)` (not bare
  `const`), which is this file's own established, intentional pattern for exempting form-data
  slots from `check:panel-refs`' "goes through a `reset*` — this file has none, on purpose" rule
  (see the file's own header comment, already present before this story, on why `libraryImport.ts`
  should not grow a `reset*` function). `submitBilingualFilePath` untouched, per Decision 1.
- `src/modes/LibraryMode.vue` — the destination radios + picker, right above the (now
  conditionally read-only) name/source-lang/genre fields, in the shared `import-form`. Picking an
  existing Work switches those three fields from `v-model` inputs to disabled `:value` displays of
  the picked `WorkRow`'s own fields — never the other way around (New Work always shows the typed
  values). `data-import-destination`/`data-import-destination-picker` markers added for Phase 4's
  future frontend tests, following the file's existing `data-import-preview-open` convention.
  `:disabled` on the three monolingual submit buttons gained `|| destinationPending`.
- `src/ImportPreviewOverlay.vue` — a read-only `<p class="ip-destination">` line right under the
  header, showing "Tác phẩm mới" or "thêm vào \"{name}\"" plus the adopted source-lang label
  (mapped through the same `zh`/`en` → display-string logic `LibraryMode.vue` uses, with a raw
  fallback for any other value — defensive, since `work.source_lang` is contractually only ever
  `zh`/`en`, per `libraryImport.ts::sourceLang`'s own type). No interactive control here — see the
  placement decision above.
- `src/i18n/vi.json` — nine new keys under `mode.library.*` (`field_destination`,
  `destination_new_work`/`_desc`, `destination_existing_work`/`_desc`, `destination_empty_library`,
  `destination_pick_label`, `destination_pick_placeholder`) and three under
  `mode.library.preview.*` (`destination_new_work`, `destination_existing_work` with a `{name}`
  placeholder, `destination_source_lang` with a `{lang}` placeholder). No new error-reason keys:
  Phase 2 measured that destination resolution reuses `WorkMetaTooNew`/`WorkOpenFailed`/
  `LibraryWorkNotIndexed` (Story 5.7) end to end, and those three already have `vi.json` entries
  (`err.work.meta_too_new`, `err.work.open_failed`, `err.library.work_not_indexed` — verified
  present, unchanged).

**Verification.**

- `npx vue-tsc --noEmit -p tsconfig.json`: **0 errors under `src/**`** (grepped the full output for
  lines outside `tests/frontend/`, empty result). **181 errors across 10 `tests/frontend/**`
  files** — every one is `TS2554: Expected N arguments, but got N-1`, the direct, expected
  consequence of widening `openImportPreviewFromText`/`_FromFile`/`_FromUrls`'s arity by one
  parameter. This is the frontend mirror of Phase 2's own two `ipc_contract.rs` failures, left
  unfixed there "on purpose, since editing `tests/**` is Phase 4's scope, not Phase 2's" — same
  reasoning applies here verbatim; the full file/count list is `importPreviewBlocks.test.ts` (13) ·
  `importPreviewChapterOrigin.test.ts` (11) · `importPreviewChapters.test.ts` (45) ·
  `importPreviewCleanup.test.ts` (18) · `importPreviewEncoding.test.ts` (19) ·
  `importPreviewEncodingWireShape.test.ts` (24) · `importPreviewFiles.test.ts` (16) ·
  `importPreviewNormalized.test.ts` (6) · `importPreviewOverlayRender.test.ts` (17) ·
  `importPreviewUrls.test.ts` (12). `npm run build` therefore fails as a whole today — Phase 4's
  task list already names exactly this seam ("`tests that move`"); this is not a hidden gap.
- `npm run test` (vitest, which transforms via esbuild and does not type-check): **73 files
  passed / 3 failed, 1032 passed / 11 failed** (Phase 1 baseline: 76/1043, all green). All 11
  failures are `toHaveBeenCalledWith(...)`-shaped assertions in `importPreviewChapters.test.ts` (5),
  `importPreviewFiles.test.ts` (5), and `libraryImportDropMultiple.test.ts` (1) — each one
  asserting the OLD, one-shorter argument list against a call that now correctly carries the
  trailing `destination`/`null` argument. Same class of expected fallout as the `vue-tsc` count
  above, same owner (Phase 4). No test outside this exact call-shape class regressed — in
  particular, all four test files that mount `LibraryMode.vue` (`libraryWorks.test.ts`,
  `libraryChapters.test.ts`, `libraryRescan.test.ts`, `librarySearch.test.ts`) still pass, meaning
  the new destination markup renders without crashing and `loadDestinationWorks()`'s `onMounted`
  IPC call degrades the same best-effort way every other IPC call in this codebase does when run
  outside a real Tauri bridge.
- `npm run check:i18n` — green (Kiểm A–E): 811 keys, all nine new placeholders match their call
  sites' param objects both directions, no new bare-text nodes (the two `aura-allow-text` markers
  added — one for the destination-picker `<option>` text, one for `tError(destinationWorksError)`
  — follow the file's existing precedent for the exact same shape, e.g. the URL-item list's
  `{{ item.url }}`).
- `npm run check:panel-refs` — green: 64 files, 347 module-level slots, 31 named exemptions
  (unchanged count) — `pendingDestinationWorkId` (the one new *bare* `const … = ref(...)` this
  phase added, in `importPreviewState.ts`) is swept by `resetImportPreview()`; every new
  `libraryImport.ts` ref is `export const`, outside this gate's syntax subset, matching that
  file's pre-existing, intentional pattern.
- `npm run check:tokens` — green, no new violations (94 files, 2967 CSS declarations scanned; the
  new `.field-group`/`.radio-field`/`.radio-copy`/`.radio-title`/`.radio-desc`/`.hint`/`.hint-error`
  classes in `LibraryMode.vue` and `.ip-destination`/`.ip-destination-lang` in
  `ImportPreviewOverlay.vue` all reference existing design tokens, no new raw colors/shadows/
  z-indices).
- `npm run check:commands` — green: the new `@change` handlers on the two radios and the picker
  `<select>` fall outside Kiểm A's `@click`-only scope by design (matches the frozen §Never:
  "reached by Tab, committed through `@change`/`@submit`, never a bare `@click` with arguments");
  157 commands, 161 `dispatch()` calls, both unchanged in count.
- `npm run check:layout` — green, unchanged (17 `window`/`document` members, all allow-listed;
  this phase touched neither).
- `npm run check:gates` / `npm run check:debt-owner` / `npm run check:deps` — green, unchanged
  (this phase touched neither `.githooks/pre-push`, `ci.yml`, `deferred-work.md`, nor any
  dependency manifest — the debt-ledger and AD-44→AD-8 citation fixes are Phase 4 tasks).
- `npx eslint src/config/project.ts src/importPreviewState.ts src/modes/libraryImport.ts
  src/modes/LibraryMode.vue src/ImportPreviewOverlay.vue` — clean, 0 findings.
- `cargo check --locked` — clean (Phase 3 touched no `src-tauri/**` file; re-run only as a
  sanity check that nothing in this phase's tree state broke the Rust side Phase 2 left).

**Left incomplete / risky, for Phase 4 to pick up.**

- The 181 `vue-tsc` errors and 11 vitest failures above are real, expected, and named — Phase 4's
  existing task ("`tests/frontend/importPreviewDestination.test.ts` (new)") does not by itself
  cover updating the ten pre-existing files' call-shape assertions; that update is necessarily
  part of closing this gap too (the same way Phase 4's `ipc_contract.rs` task covers Phase 2's two
  Rust failures). Flagging explicitly so it is not missed as "only write one new test file."
- The **placement deviation** (radios/picker in `LibraryMode.vue`, not `ImportPreviewOverlay.vue`)
  is a judgment call made under Auto Mode without stopping for Ice's sign-off, on the strength of
  the mockup evidence and the structural fetch-lock argument above. It is recorded here in full,
  with the counter-evidence (the exact grep showing the cited overlay lines don't exist) rather
  than silently substituted — if Ice reads the mockup differently, this is the one Phase 3 design
  decision most likely to need revisiting, and the new `data-import-destination`/
  `data-import-destination-picker` markers on `LibraryMode.vue`'s controls are named for exactly
  the day a frontend test needs to find them regardless of which file they end up living in.
- `destinationMode`/`destinationWorkId` are **not** reset after a successful import — deliberately
  matching `name`/`sourceLang`/`genre`'s existing behavior (none of those three reset in
  `finishImportSubmission` either; only `pastedText`/`filePath`/`pastedUrls` do). Whether a
  translator adding a second batch to the same Work in the same sitting wants the destination to
  stick, or whether it should snap back to "New Work" after one successful append, is a product
  question no AC in this story answers — flagging rather than guessing either way.
- The overlay's read-only echo shows name + adopted source-lang; it does **not** show genre.
  Nothing in the frozen I/O Matrix or Decision 2 asks for genre there, and `pendingGenre` stayed
  unexported to avoid an unused-import lint finding — easy to add if a later review wants it.
- Phase 3 did not touch `deferred-work.md`, `epics.md`, or `sprint-change-proposal-2026-09-15.md`
  — those three edits are explicitly Phase 4 tasks (the two new owned debt entries and the
  AD-44→AD-8 citation fix).

### Phase 4 — tests that move, the debt ledger, and the AD-44→AD-8 citation fix (2026-09-16, on top of Phase 1-3's uncommitted tree)

**AC3's gate — the pure function the Phase 1 correction demanded.**

The Phase 1 correction (see above) forbade shaping the new `cleanup_contract.rs` case as a
direct call to `core::cleanup::resolve_two_tiers` — that probe is inert (it bypasses the
selection logic under suspicion and returns the destination's own rules whether or not the
defect exists). Phase 2's `wire::resolve_cleanup_rules_for_destination(app, global, work_id)`
needs an `AppHandle` (no `tauri::test`/`MockRuntime` harness exists in this crate), so it
cannot be called directly from `tests/**` either. Closed the gap with the two-layer split the
codebase already uses everywhere else (pure fn / thin wire shell):

- **New `pub fn` `commands::project::resolve_work_tier_cleanup_rules_for_destination(destination:
  &OpenWork, open: Option<&OpenWork>, global: &Store) -> Result<Vec<CleanupRule>,
  CleanupStoreError>`** (`src-tauri/src/commands/project/mod.rs`, right after
  `append_chapters_to_work`). Takes both `destination` and `open` (whatever's in
  `OpenWorkState`, if anything) **explicitly** — no `AppHandle`, no `work_id`-by-string lookup.
  When `open` matches `destination`'s `work_id`, reuses `open`'s store (never a second write
  connection to the same `project.db`, `src-tauri/AGENTS.md:30`); otherwise resolves from
  `destination`'s own store. This is testable with two real `OpenWork`s built via `create_work`
  — no mocking, no `AppHandle`.
- `wire::resolve_cleanup_rules_for_destination(app, global, work_id)` was refactored to be the
  **thin** shell: it now only does the impure part (borrowing `OpenWorkState`, resolving the
  destination fresh via `Indexer::find_work` + `open_work` when it doesn't match what's open)
  and delegates the actual rule-selection decision to the new pure function. Behavior for every
  existing call site is unchanged — verified by the full `cargo test --locked --no-fail-fast`
  re-run below, which is identical to the pre-refactor numbers.
- New `cleanup_contract.rs` cases:
  `resolving_cleanup_rules_for_a_destination_applies_global_plus_the_destinations_own_rules_not_the_open_works`
  (destination W, a DIFFERENT Work X open, a Work-tier rule on each — only W's rule comes back)
  and its positive counterpart
  `resolving_cleanup_rules_when_the_destination_is_the_open_work_still_returns_its_own_rules`
  (destination == open — its own rule still returns, proving the function doesn't just always
  discard `open`).
- **Counter-check run, as the correction demanded**: mutated the pure function's body to always
  prefer `open` when `Some` (`match open { Some(o) => o, None => destination }` — i.e. reverted
  to the exact pre-story defect), re-ran `cargo test --locked --test cleanup_contract`. Result:
  `resolving_cleanup_rules_for_a_destination_applies_global_plus_the_destinations_own_rules_not_the_open_works`
  went **red** (`luat tang Work CUA DICH (W) phai co mat: [CleanupRule { ... pattern: "CHI_TREN_X"
  ... }]` — it returned X's rule for a preview built for W). The positive counterpart stayed
  green (destination == open in that case, so the mutation is a no-op there — correctly
  distinguishing the two cases). Reverted immediately after the measurement; re-ran the same two
  cases to confirm both green again before moving on. **This is a real, non-inert guard.**

**`project_contract.rs` — four new cases for the append path itself.**

- `append_writes_new_chapters_at_max_ord_plus_one_all_not_started_with_segments_for_each` —
  Work with 3 pre-existing Chapters (built through `create_work`, not hand-inserted SQL, so
  the "old chapter" shape is the real thing an import produces), append 2 new ones via
  `append_chapters_to_work` directly (the pure sibling of `create_work`, no `PendingImportSourceState`
  plumbing needed for this proposition). Asserts `ord` 4 and 5 in the exact order the shape
  listed them, both `not_started`, and a `segment` row count `> 0` for each new chapter — same
  shape as `create_work_writes_every_chapter_and_its_segments_when_the_pipeline_yields_more_than_one`
  but for the append path.
- `append_leaves_every_pre_existing_chapter_and_segment_row_byte_identical` — the AC's own
  load-bearing proposition. New `ChapterSnapshot`/`snapshot_chapters` helper (all 11 columns,
  the four origin columns from Story 6.10 included) added next to the existing
  `SegmentSnapshot`/`snapshot_segments`. Snapshots every pre-existing `chapter` and `segment`
  row **before** the append, appends one new chapter, snapshots again, filters the "after" set
  down to the ids that existed before, and asserts full-row equality — same discipline as
  `merging_two_chapters_changes_only_chapter_id_and_ord_on_every_segment_column`.
- `status_re_derives_to_in_progress_after_append_and_only_on_the_right_work` — **two** Works
  in the fixture (X, the append target; Y, an unrelated decoy), both manually driven to `Done`
  (`set_chapter_status_directly` + a manual `WorkMeta::rebuild_from_store`/`write_atomic`, same
  pattern `merging_two_chapters_leaves_the_smaller_chapter_count_in_the_library_index` already
  used, since direct-SQL status writes don't run the four-step template themselves). Appends
  through `confirm_append_import_with_encoding` (the full pure confirm path, steps 1-3) into X
  only; asserts X re-derives to `in_progress` (mixed `Done`+`NotStarted` — `derive_work_status`'s
  `else` branch) **and** that Y's `meta.json` on disk still reads `done`, untouched — so a future
  regression that re-derives the wrong Work's status (the same mismatched-Work failure class
  AC3 guards, but on the status axis instead of cleanup rules) would be caught here, not just
  "X changed" in isolation.
- `status_override_survives_an_append_that_would_otherwise_re_derive_status` — same append,
  but `work.status_override` set to `paused` first via `set_work_status_override`; asserts the
  override still wins after the append (§Always: `status_override` keeps winning through
  `WorkMeta::rebuild_from_store`, `core/library/meta.rs:390-395`).

**`project_contract.rs` — destination unreadable, zero rows written.**

`confirm_append_is_refused_and_writes_zero_rows_when_the_destination_project_db_has_vanished`
mirrors the existing Story 5.7 case
`opening_a_work_with_a_newer_meta_schema_is_refused_without_touching_a_single_byte` exactly, but
for the "project.db missing" branch specifically (`open_work`'s own `db_path.exists()` guard,
which exists precisely to stop `Store::open`'s `SQLITE_OPEN_CREATE` from silently creating a
project.db on a rejected open). ⚠️ **What this test does and does not prove, stated plainly**:
`open_work`'s error behavior for "vanished"/"unreadable" destinations was already proven by
Story 5.7's own tests, unchanged by this story (the frozen I/O Matrix says "reuse
`WorkError::OpenFailed`/`MetaTooNew`" — explicitly reuse, not re-litigate). What is *new* to
this story is that `wire::confirm_import_with_encoding`'s append branch (`wire.rs:882-883`)
is a second, new caller of `open_work` reaching the exact same failure modes — and that branch
cannot be exercised directly from `tests/**` (it needs an `AppHandle`, and its structure is two
plain `?`-propagated calls in a row: `indexer.find_work(&work_id)?` then `open_work(&work_id,
indexed.as_ref())?`, both **before** `confirm_append_import_with_encoding` is ever reached). The
"0 rows written" half of the AC is therefore a type-level guarantee here, not a runtime
observation: there is no live `&mut OpenWork` to pass to `confirm_append_import_with_encoding`
on the `Err` branch of `open_work`, so the program cannot call it — recorded as such rather than
padding the test with an assertion that would only restate what the compiler already enforces.

**`ipc_contract.rs` — the two known-red cases, plus the trailing `destination` parameter.**

Both named cases
(`the_three_import_encoding_preview_wires_are_registered_and_keep_their_parameter_names`,
`the_three_url_import_wires_are_registered_and_keep_their_parameter_names`) updated to expect
the trailing `destination: Option<String>` on exactly the three wires Phase 2 actually added it
to: `preview_import_encoding_from_text`, `preview_import_encoding_from_file`, `start_url_import`.
`confirm_import_with_encoding`'s expected param list is **unchanged** (it reads the destination
from `PendingImportSourceState` per Phase 2's 2026-09-16 correction, so its wire signature is
byte-for-byte what it was before this story); `reload_url_import_item`/`remove_url_import_item`
likewise unchanged (confirmed against the current source — neither carries a `destination`
parameter). Both cases green after the edit; verified by running them in isolation
(`cargo test --locked --test ipc_contract -- the_three_import_encoding_preview_wires...
the_three_url_import_wires...` → 2 passed).

**Frontend — `vue-tsc`'s 181 errors and vitest's 11 known-red cases.**

All 181 `TS2554` errors (10 files, exact set named in Phase 3's Implementation Notes) were a
mechanical "trailing argument now required" gap: every flagged call site got exactly one
trailing `, null` argument inserted (scripted — read each error's `file:line` from `vue-tsc`'s
own output, inserted before that line's last `)`, re-ran `vue-tsc` to confirm 0 errors under
`src/**` and `tests/frontend/**` both). Spot-checked a sample of diffs by hand (object-argument
call sites in `importPreviewEncodingWireShape.test.ts`, no-`await` call sites in
`importPreviewFiles.test.ts`) — the insertion never landed inside a nested object/array
argument, only at the true end of the call's argument list.

`npm run test` after that mechanical pass: **13 failed**, not the 11 the baseline named — the
scripted fix above added a NEW, correct trailing `null` argument to two calls into
`previewImportEncodingFromText`/`_FromFile` inside `importPreviewEncodingWireShape.test.ts`
that assert the **exact wire payload** sent to `invoke()` (not just the JS-side argument count);
those two assertions still expected the pre-story payload shape (`{ text, sourceLang,
chapterPattern }`, missing the new `destination` field `src/config/project.ts` now sends
verbatim under that name). Added `destination: null` to both expected payload objects — this
is a real, new consequence of Phase 3's own change (not a story-6.7b-Phase-4 defect), and is
recorded here because the baseline given to Phase 4 undercounted it by exactly these two cases.

The remaining 11 (`importPreviewChapters.test.ts` × 5, `importPreviewFiles.test.ts` × 5,
`libraryImportDropMultiple.test.ts` × 1) were all `toHaveBeenCalledWith`/`toHaveBeenNthCalledWith`
assertions against the OLD, one-argument-shorter call — each got the same trailing `, null`
(or, for the two `chapterPattern`-bearing cases in `importPreviewChapters.test.ts`, `, null`
after the pattern object) inserted at the exact reported line. `npm run test` afterward: **77
files / 1049 tests, all green** — the Phase 1 baseline (76/1043) plus the one new test file
(6 cases) below.

**New `tests/frontend/importPreviewDestination.test.ts`.**

🔵 **Interpretation note, recorded rather than silently assumed**: the task's literal wording
("changing the destination triggers exactly one preview rebuild") describes a live, mid-overlay
re-pick — but Phase 3 measured and recorded that no such control exists (the destination is
fixed *before* the first preview-opening call, because `source_lang` must already be the
adopted value on that call, and the URL path's `start_url_import` accepts a destination exactly
once). Read literally against the actual architecture, "changing the destination" is a pre-submit
action, and "triggers exactly one preview rebuild" is read here as *"picking a destination and
then submitting produces exactly one new preview-opening IPC call, carrying that destination"* —
measured by the **difference** in `previewImportEncodingFromText`'s mock call count before and
after, per the task's own instruction not to use `not.toHaveBeenCalled()`. Six cases:

1. `destinationMode` defaults to `'new'`, `destinationWorkId` is `null`, and `effectiveName`/
   `effectiveSourceLang`/`effectiveGenre` read the three hand-typed refs when no destination is
   picked.
2. Mount `LibraryMode.vue` for real (`mockInvoke` serving `library_list_works`, same command
   both the grid and the destination picker call), pick "existing Work" then pick the one Work
   in the list: the name/source-lang/genre fields all become `disabled` inputs carrying the
   destination's own values (`Truyen Da Co` / `zh` / `action`) — found by querying
   `.import-form .field input[disabled]`, not by matching translated label text (label text is
   the SAME regardless of which value renders, so it cannot distinguish "still editable" from
   "now read-only").
3. Empty Library: the "existing Work" radio renders `disabled`, the picker itself does not
   render at all, and the "why" hint (`data-import-destination .hint`) is present.
4. `submitPastedText()` called directly (no DOM — same style as
   `libraryImportBlocksResubmitWhilePreviewOpen.test.ts`) with an existing-Work destination
   pre-set: exactly one new `previewImportEncodingFromText` call, carrying the destination's own
   `source_lang` (not the hand-typed one — Decision 2) and the destination's `work_id` as the
   trailing argument.
5. Radio "existing" chosen but no Work picked yet (`destinationWorkId === null`,
   `destinationPending` true): `submitPastedText()` is a no-op, **0** new preview calls — the
   task's own §Always concern ("never send `destinationWorkId: null` silently read as New Work
   when the radio disagrees").
6. Destination left at New Work (default): one new preview call, `destination` argument `null`.

**Counter-check on case 5** (the one most likely to be vacuously green): temporarily replaced
`libraryImport.ts`'s `if (destinationPending.value) return` guard in `submitPastedText` with
`if (false) return`, re-ran the file. Result: case 5 went **red** (`expected 1 to be +0` — a
preview call fired that should not have). Reverted immediately; re-ran the full file (6/6 green)
and then the full `npm run test` (77/77 files, 1049/1049 tests) before moving on.

**Debt ledger.**

- `deferred-work.md:10354-10396` (the "6-7-nhap-tu-url-bang-danh-sach-link — tách phạm vi ở
  bước định tuyến" entry — this story's own origin item) closed with a `→ ✅ ĐÓNG 2026-09-16`
  line naming what actually landed (`append_chapters_to_work`,
  `confirm_append_import_with_encoding`, the destination-resolution wiring in `wire.rs`, and the
  picker's real location in `LibraryMode.vue`), and pointing at the bilingual-path debt entry
  below for the one piece Decision 1 explicitly left open.
- Two new owned entries under a new `## Deferred from: 6-7b-them-chuong-vao-tac-pham-co-san
  (2026-09-16)` header: ① the bilingual import path (`confirm_bilingual_import`) still always
  creates a new Work — **Chủ: Story 6.16b** (the story already queued against that screen,
  Decision 1's own stated reason for excluding it here); ② a batch whose language disagrees
  with the destination's adopted `source_lang` is sentence-split by the wrong rules with no
  signal — **Chủ: Ice** (a product decision on whether/how to warn, explicitly not decided by
  Decision 2, which only records the consequence). 🔴 First draft of entry ② used `Chủ: chưa
  gán`, which `check:debt-owner`'s Kiểm A correctly reads as a NEGATIVE claim ("no owner yet"),
  not a real owner — caught by running `npm run check:debt-owner` (1/516 items failed) before
  moving on, fixed to `Chủ: Ice` (the precedent this file already uses for "a future story must
  decide," e.g. the `chapter_detail_for_index`/`Files` debt item), re-ran to confirm 0/516.

**AD-44 → AD-8 citation fix (Decision 4).**

Fixed in place, not silently replaced, per `AGENTS.md`'s "a claim that stops being true gets
FIXED IN PLACE with 🔵 and a date": `epics.md:4861` and
`sprint-change-proposal-2026-09-15.md:52`, both citing a nonexistent/wrong `AD-44` for "the
Library index has one write path," corrected to `AD-8` (`Indexer::rebuild` is the only writer
of `library-index.db`) with an inline `🔵 SỬA 2026-09-16 (Story 6.7b)` note at each site, cross-
referencing `src-tauri/AGENTS.md:54` (already correct) and each other. **Touched only those two
citations** — `sprint-change-proposal-2026-09-15.md:109` also says `AD-44` (in a longer
"Ghi chú cài đặt" paragraph) and was deliberately left alone, since the task named exactly
`:52` and nothing in Decision 4 asked for a sweep of every occurrence in every file.

**Verification, full run (2026-09-16, on the complete Phase 1-4 tree).**

- `cargo check --locked` / `--tests`: clean, 0 warnings, 0 errors.
- `cargo test --locked --no-fail-fast`: **1543 passed, 0 failed, 20 ignored** (56 binaries + 2
  doc-test summaries) — the Phase 1 baseline (1536/0/20) plus the 7 new cases this phase added
  (5 in `project_contract.rs`, 2 in `cleanup_contract.rs`). The two previously-red
  `ipc_contract.rs` cases are green. `schema_version()` still 22 (untouched by this phase).
- `npm run build`: green — `vue-tsc` clean on both `tsconfig.json` and `tsconfig.node.json`,
  `vite build` succeeds, same two pre-existing `INEFFECTIVE_DYNAMIC_IMPORT` warnings as every
  prior phase, no new ones.
- `npm run test`: **77 files / 1049 tests, all green** — the Phase 1 baseline (76/1043) plus the
  one new file (6 cases).
- `npm run check:deps && check:i18n && check:tokens && check:commands && check:layout &&
  check:panel-refs && check:gates && check:debt-owner`: **all eight green**, run in this order,
  after every source/doc edit above landed (including the debt-ledger and citation edits, which
  land in files these gates scan).
- Counter-checks run and reported above, not merely claimed: AC3's cleanup-rule selection
  (mutate-and-revert, red confirmed), the append path's no-op guard for a pending-but-unpicked
  destination (mutate-and-revert, red confirmed). The append path's other two ACs from
  §Verification (removing `reindex_after_lifecycle_write`; writing at `ord = 1` instead of
  `MAX(ord)+1`) were **not** re-run as counter-checks in this phase — they are named in the
  spec's top-level §Verification section as counter-checks for the *whole story*, not listed
  among Phase 4's own task bullets, and are flagged here as left to whoever signs the story off
  as `done`, not silently assumed to hold.

**Left incomplete / risky, for the story's final sign-off.**

- The nightly e2e `schedule` run named in the top-level Acceptance Criteria
  ("read before writing `done`... recorded here — green, or red with the reason") was **not**
  read or triggered in this phase — it is a whole-story closing action, out of the "tests that
  move, the debt ledger, and the citation fix" scope this phase was handed. Flagging explicitly
  so it is not silently skipped when the story moves to `done`.
- 🔵 **CẬP NHẬT 2026-09-16 (coordinator review) — bản ghi dưới đây khi viết lần đầu nói "chưa
  chạy", và điều đó hết đúng ngay sau khi viết.** Coordinator CHẠY cả hai counter-check ①/③ của
  §Verification (không phải tôi đoán "plausible"): ① cho **0 ca đỏ** (byte-identical với đo
  2026-08-27) — một khiếm khuyết THẬT, đóng ở mức "Phase 4 correction" ngay trên (seam
  `confirm_append_import_with_encoding_indexed` mới, cộng giới hạn còn lại được ghi thẳng, không
  giấu). ③ cho ĐÚNG 1 ca đỏ
  (`append_writes_new_chapters_at_max_ord_plus_one_all_not_started_with_segments_for_each`), và
  mệnh đề gốc của spec ("old rows untouched" cũng phải đỏ) hoá ra SAI — ép `base_ord = 0` để lại
  mọi hàng CŨ nguyên vẹn (chỉ tạo `ord` trùng lặp giữa hàng cũ/mới), hai mệnh đề khác nhau.
  Coordinator tự sửa dòng đó trong spec; không đụng tới ở đây.
- The `confirm_append_is_refused_and_writes_zero_rows_when_the_destination_project_db_has_vanished`
  test's "0 rows written" half rests on a type-level argument (no live `&mut OpenWork` on the
  `Err` branch), not a runtime assertion against `wire::confirm_import_with_encoding` itself —
  that wire function cannot be called from `tests/**` at all (no `tauri::test` harness in this
  crate). If a future refactor changes the wire's control flow (e.g., resolving the destination
  AFTER some partial append work, instead of strictly before), this test would not catch it —
  flagged rather than silently assumed covered.
- Two vitest fixes (`destination: null` added to the wire-payload assertions in
  `importPreviewEncodingWireShape.test.ts`) were outside the 11-case baseline this phase was
  given; recorded above as a measured correction to that baseline, not a silent scope change.

### Phase 4 correction (2026-09-16, coordinator review) — counter-check ① found the step-four seam genuinely unguarded; closed as far as this crate's test boundary allows, and the remaining gap is named rather than papered over

**What the coordinator found.** Running the removal counter-check named in §Verification
(remove `reindex_library(&app, &root)` from the append branch of
`wire::confirm_import_with_encoding`, `wire.rs:905`) gave **0 failures** across the full suite —
byte-identical to the 2026-08-27 measurement the AC is named after. The append path had **no
seam at all** for step 4, in any shape — unlike `set_chapter_status`/`merge_chapter_into_previous`,
which both already had a test-facing `*_indexed` pure combinator. This was a real, reported gap,
not a false alarm.

**What was built.** `commands::project::confirm_append_import_with_encoding_indexed` — a new
`pub fn`, exact precedent shape of `commands::lifecycle::set_chapter_status_indexed`: it wraps
[`confirm_append_import_with_encoding`]'s `Result` through
`crate::commands::lifecycle::finish_lifecycle_write` (the SAME single choke point
`wire::set_chapter_status`'s `finish_with_reindex` already uses — `is_ok() ⇒ reindex` decided in
exactly one place in the whole crate). Two new `project_contract.rs` cases:

- `appending_through_the_indexed_confirm_path_updates_the_library_indexs_chapter_count_and_full_text_search`
  — real `Indexer`/`global.db`, a real append through `PendingImportSourceState` +
  `confirm_append_import_with_encoding_indexed` (not raw SQL, not a bare `Indexer::rebuild` call
  from the test), asserting BOTH measures the coordinator named: `library_work.chapter_count`
  goes 1→2, and a full-text search for a marker string unique to the newly-appended chapter
  finds **0** hits before the append and **≥1** hit (on the right `work_id`) after.
- `skipping_step_four_after_an_append_leaves_the_library_index_stale_on_both_measures` — a
  permanent, checked-in positive demonstration (same style as this suite's other self-checks,
  e.g. `fn_param_list_would_actually_bind_to_the_right_function_block`): calls
  `confirm_append_import_with_encoding` **without** the indexed wrapper (i.e., skips step 4 at
  the pure-function layer, the same shape as the coordinator's wire-level removal) and asserts
  the index stays stale on both measures — `chapter_count` stuck at the old value, the marker
  never found. This is the "0 failures" the 2026-08-27 measurement recorded, now pinned as a
  permanent green case proving the *assertions themselves* can go red, not just a claim.
- Added the cheap assertion the coordinator suggested to the existing
  `append_writes_new_chapters_at_max_ord_plus_one_all_not_started_with_segments_for_each` case:
  `distinct_ords.len() == rows.len()` across all 5 chapters (old + new) — fits cleanly, no shape
  change needed, since the test already reads every row's `ord` into memory.

**Counter-check run on the new gate (not merely claimed):** temporarily mutated
`commands::lifecycle::finish_lifecycle_write`'s body to `if false { reindex_after_lifecycle_write
(...) }` (disabling the ONE shared step-4 implementation) and ran
`cargo test --locked --no-fail-fast`. Result: **4 cases red** —
`appending_through_the_indexed_confirm_path_updates_the_library_indexs_chapter_count_and_full_text_search`
(the new one), plus three PRE-EXISTING cases that share the same choke point
(`setting_a_chapter_status_leaves_the_new_value_in_the_library_index`,
`overriding_the_work_status_leaves_both_the_value_and_the_override_flag_in_the_index`,
`merging_two_chapters_leaves_the_smaller_chapter_count_in_the_library_index`) — 1541 passed, 4
failed, 20 ignored. Reverted immediately (file restored, re-diffed to confirm no residue); re-ran
to confirm 1545 passed / 0 failed / 20 ignored again before moving on. **The new gate is real and
is now part of the same systemic family the rest of the four-step template already belongs to.**

**Then the coordinator's EXACT original counter-check was re-run, honestly, as instructed.**
Reproduced the identical mutation (`reindex_library(&app, &root);` at `wire.rs:905` replaced with
a no-op) and ran the full suite again. Result: **1545 passed, 0 failed, 20 ignored — still
green.** 🔴 **Reporting this plainly rather than reshaping anything to force red, per the
coordinator's explicit instruction:** the new `_indexed` seam does not and structurally cannot
catch a deletion of the WIRE'S OWN call site, because no test in this crate can invoke
`wire::confirm_import_with_encoding` at all — there is no `tauri::test`/`MockRuntime` harness
here (confirmed at Phase 1, unchanged since). This is not unique to the append path: the exact
same gap already existed, unnoticed, for `wire::set_chapter_status`'s own `finish_with_reindex`
call before this session — no test in the suite would catch someone deleting THAT line either,
for the identical structural reason. What Phase 4 closed is the half that a `pub fn`, no-`AppHandle`
seam CAN reach — "does the append confirm path's own reindex machinery actually update the
derived store" — matching exactly what `set_chapter_status_indexed`/
`merge_chapter_into_previous_indexed` already prove for their operations, no more and no less.
The half that remains open — "does the wire remember to call it" — is a standing limitation of
this crate's two-layer test boundary, pre-existing this story, not something this phase invented
or can close without adding a `tauri::test` harness (a materially larger undertaking, out of this
phase's scope, and not requested).

**Verification after this correction:** `cargo check --locked`/`--tests` clean; full
`cargo test --locked --no-fail-fast`: **1545 passed, 0 failed, 20 ignored** (the post-Phase-4
baseline of 1543 plus these 2 new cases); `npm run build` green, same two pre-existing warnings;
all eight `check:*` gates re-run, all exit 0 (`npm run test` unaffected — no `src/**`/
`tests/frontend/**` file touched by this correction).

### Phase 4 correction #2 (2026-09-16, Ice via coordinator) — source-scanning gate for all five `reindex_library` call sites in `wire.rs`

**What Ice settled.** "No test can invoke a wire shell" is not the end of the road for guarding
its *source*: this repo already guards un-invokable wiring by reading source text —
`ipc_contract.rs`'s `generate_handler!` containment checks, and
`webimport_boundary.rs`'s `code_lines()`/`text_before_first_cfg_test_line()` discipline (with its
own self-checks at `:474`/`:486`) so a commented-out line doesn't count as present. Measured gap
before this correction: `grep -rn 'contains("reindex' src-tauri/tests` returned **0** — the shape
did not exist. Built it, covering all five call sites named
(`src-tauri/src/commands/project/wire.rs:499`, `:545`, `:837`, `:905`, `:1102`), not just the
append seam this story added.

**What was built, in `src-tauri/tests/ipc_contract.rs`:**

- `REINDEX_LIBRARY_CALL_LINE` — the exact literal `"reindex_library(&app, &root);"`, confirmed
  byte-identical across all five sites by direct inspection before writing anything.
- `code_lines(text)` — line-level comment filter, same discipline as
  `webimport_boundary.rs::code_lines` (strips `//`/`///`/`/*`/`* `/`*/`-prefixed and blank lines).
- `count_reindex_library_calls(text)` — counts **exact, full-line** matches of
  `REINDEX_LIBRARY_CALL_LINE` within `code_lines(text)`. A commented-out call
  (`// reindex_library(&app, &root);`) does **not** count — this is the specific fake-removal
  trap `AGENTS.md:68` names, and it is what a bare `grep`/`contains` over the raw file would miss.
- `wire_fn_body(wire_src, fn_name)` — slices one `pub fn`'s body from its `pub fn {name}(`
  signature to the next `\n    pub fn ` (or end of file), generalizing the ad hoc `code_lines_of`
  closure already living inside
  `the_three_import_encoding_preview_wires_are_registered_and_keep_their_parameter_names` (same
  file, written in Phase 2/4) into a reusable top-level function.
- `all_five_reindex_library_call_sites_in_wire_rs_are_present` — the gate itself. **Anchored to
  the enclosing command for four of the five seams** (`create_work_from_text`,
  `create_work_from_file`, `confirm_bilingual_import` — exactly 1 occurrence each). 🔵 **Decision
  recorded, as asked**: `confirm_import_with_encoding` holds **two** of the five call sites (the
  New-Work branch, `:837`, and the Append branch, `:905`) inside one Rust function — per-function
  anchoring alone (`count == 2`) would catch either disappearing but couldn't say *which* branch
  died without a smaller anchor. Chose to split that one function's body at its own existing,
  substantive doc-comment boundary — `"// Đường APPEND (Story 6.7b)"`, the literal line already
  marking where the Append branch begins — into two sub-scopes, each asserted at exactly 1. This
  is anchoring to "which seam", not strictly "which Rust `fn`", because the two seams are not
  separate functions today; if a future story splits `confirm_import_with_encoding` into two
  functions (a plausible refactor, since New-Work and Append are already fairly separate), this
  split point stops existing and the test panics naming exactly why (`"khong tim thay neo..."`)
  rather than silently mis-scoping — a loud failure forcing an update, not a false green. A
  whole-file population floor (`total == 5`) runs *in addition to*, not instead of, the five
  per-seam assertions, catching a stray sixth occurrence appearing/disappearing elsewhere in
  `mod wire` that none of the five named anchors would see.
- `count_reindex_library_calls_is_not_fooled_by_a_commented_out_line` — the required
  positive/negative self-check pair, `webimport_boundary.rs`-style: a real code line counts (1);
  a `//`-commented line does not (0); a `///`-doc-commented line does not (0); and a
  near-identical line with a *different* argument (`&different_root`) does not count as a match
  (0) — guarding against a hand-edit that changes the call's arguments without deleting it.
- `wire_fn_body_stops_before_the_next_pub_fn_and_does_not_bleed_into_it` — self-check for the
  body-slicing helper itself, same spirit as
  `fn_param_list_would_actually_bind_to_the_right_function_block`: proves `foo`'s slice contains
  exactly its own call and never bleeds into `bar`'s.

**Counter-check — five separate removals, reported in full, none adjusted after the fact.**
Backed up `wire.rs` to the session scratchpad (`wire.rs.backup_before_5_removals`, MD5
`727c64b321898ae239bbafdc562d1a06`) before any mutation — restored from that backup after each of
the five removals below, **never from `git`/HEAD** (the tree carries uncommitted Phase 1-4 work
that a HEAD restore would have destroyed). Verified every restore two ways: an MD5 checksum match
against the scratchpad backup, and `git status --short` showing the file only as modified relative
to HEAD (i.e., exactly the pre-existing Phase 1-4 diff, nothing more/less) — per instruction, not
`git diff`.

| # | Line removed | Seam | Gate result | Failing assertion named |
|---|---|---|---|---|
| 1 | `:499` | `create_work_from_text` | **RED** | `create_work_from_text (:499)` — found 0, expected 1 |
| 2 | `:545` | `create_work_from_file` | **RED** | `create_work_from_file (:545)` — found 0, expected 1 |
| 3 | `:837` | `confirm_import_with_encoding`, New-Work branch | **RED** | `confirm_import_with_encoding, nhanh Tac pham MOI (:837)` — found 0, expected 1 |
| 4 | `:905` | `confirm_import_with_encoding`, Append branch | **RED** | `confirm_import_with_encoding, nhanh APPEND (:905)` — found 0, expected 1 |
| 5 | `:1102` | `confirm_bilingual_import` | **RED** | `confirm_bilingual_import (:1102)` — found 0, expected 1 |

All five removals went red, each naming the specific seam that died (not merely "the total
changed"). All five restores verified via MD5 match to the scratchpad backup, immediately
followed by a green re-run of the gate before the next removal. Removal #4 is byte-for-byte the
same mutation the coordinator performed in the prior review round (`wire.rs:905`, no-op'd) — this
gate now catches it, closing the specific hole that prompted this correction.

**Full verification after landing the gate (backup file deleted from scratchpad afterward):**
`cargo check --locked`/`--tests` clean; `cargo test --locked --no-fail-fast`: **1548 passed, 0
failed, 20 ignored** (1545 plus the 3 new `ipc_contract.rs` cases: the gate itself, the
comment-fooling self-check, and the body-slicing self-check); `npm run test`: **77 files / 1049
tests, unchanged** (no `src/**`/`tests/frontend/**` file touched by this correction); `npm run
build`: green, same two pre-existing warnings; all eight `check:*` gates re-run, all exit 0.

**What this does and does not close, stated plainly.** This closes exactly what Ice asked for:
source-level proof that all five step-4 call sites exist in `wire.rs`, surviving a comment-out or
a deletion at any one of the five, with a whole-file population floor as a backstop. It does
**not** newly prove these calls execute correctly at runtime inside a real Tauri app (that half
was already separately guarded — for the shared `finish_lifecycle_write`/`_indexed` family — by
Phase 4 correction #1 above, and remains unreachable for the bare wire-level composition for the
structural reason recorded there: no `tauri::test`/`MockRuntime` harness in this crate). The two
corrections together cover both halves this crate's test boundary can reach: "the call sites
exist in source" (this correction) and "the shared reindex machinery those call sites depend on
actually updates the derived store when invoked" (correction #1) — the remaining, named gap is
"does the wire actually invoke it at runtime", which no test in this crate closes for any wire
function, not only this story's.

### Matrix-coverage audit close-out (2026-09-16, orchestrator)

**What triggered this pass.** An audit of the frozen §I/O & Edge-Case Matrix found four rows
with no covering test that actually runs: "Destination is the open Work" (write half only —
`cleanup_contract.rs` already covered the read half), "Preview not confirmed", "Destination
unreadable", and "Write fails mid-append". Added one Rust case per row to
`src-tauri/tests/project_contract.rs`, right after
`confirm_append_is_refused_and_writes_zero_rows_when_the_destination_project_db_has_vanished`
and before the Phase-4-correction "indexed" cases. `snapshot_chapters` (`project_contract.rs`)
changed signature from `&OpenWork` to `&Store` — the new "Destination unreadable" case needs to
snapshot a `project.db` reopened on its own (no `OpenWork` exists on that path, since
`open_work` is the function being refused); the two pre-existing call sites became
`snapshot_chapters(&opened.store)`.

**"Destination is the open Work" — the write half.**
`appending_into_the_work_already_held_in_open_work_state_reuses_its_store_and_stays_usable`
constructs a real `OpenWorkState` (`std::sync::Mutex<Option<OpenWork>>` — no `AppHandle`
needed, it is a bare type alias), puts the destination Work inside it, calls
`confirm_append_import_with_encoding` on the `&mut OpenWork` borrowed straight out of the
guard, then — still reading through the *same* `Store` that sat in `OpenWorkState` the whole
time, no second `open_work`/`Store::open` call anywhere in the test — asserts the chapter count
went up by the right amount and `chapter_id` (the open editor's cursor) is untouched.
**Counter-check**: added `open.store.close();` at the end of `append_chapters_to_work` (right
after the `images_saved`/`images_failed` assignment, before `Ok(appended_count)`) — the case
went **red** (`store.pool_closed` bubbling out of `write_lifecycle_after_change`, which needs
the store to still be open for steps 2–3). Reverted from the scratchpad backup (MD5-verified,
`git status --short` showed only the pre-existing Phase 1–4 diff), re-ran green.

**"Preview not confirmed" — nothing reaches disk.**
`an_unconfirmed_preview_leaves_the_destinations_stores_unchanged` builds a destination Work,
reindexes it once into a real `Indexer`/`global.db`, closes everything (checkpoint + truncate
WAL) to get a byte-deterministic baseline, then simulates a real preview being built for this
destination — `stash_pending_import_source` (pure in-memory) plus
`resolve_work_tier_cleanup_rules_for_destination` against a freshly reopened `OpenWork` (the
same read the wire layer performs for a destination that is not already open) — and asserts
`project.db`, `meta.json`, and `library-index.db` are byte-identical before/after, comparing
file content (both snapshots taken with every store closed) rather than `mtime`. **Counter-check**:
temporarily made `reopened` mutable and inserted a real call to
`confirm_append_import_with_encoding` right after the cleanup-rule read (i.e., simulated "the
user never confirmed, but a bug appends anyway") — the `project.db`-unchanged assertion went
**red** (bytes differed, full SQLite page dump visible in the panic). Reverted from backup
(MD5-verified), re-ran green. This is the seed-the-forbidden-violation shape, since there is no
product-code call site to delete here — previews are structurally in-memory-only.

**"Destination unreadable" — corrupt `project.db`, or `meta.json` schema newer.**
`confirm_append_is_refused_with_a_named_message_key_when_the_destination_is_unreadable` covers
both conditions the row names, each against a `create_destination_work` fixture with 2
pre-existing Chapters (not a bare/empty Work), reusing `open_work` directly — the exact
function `wire::confirm_import_with_encoding`'s APPEND branch calls before
`confirm_append_import_with_encoding` is ever reached (same reasoning already recorded above
for the "vanished" case: the wire itself cannot be invoked from `tests/**`, no
`tauri::test`/`MockRuntime` harness in this crate).
- (a) `meta.json` bumped to `META_SCHEMA_VERSION + 1` ⇒ `work.meta_too_new` /
  `MessageKey::WorkMetaTooNew` — reusing Story 5.7's own key, pinned again here in the append
  context, plus a `snapshot_chapters` before/after equality proving the destination's 2 existing
  Chapters survive the refusal untouched.
- (b) `project.db` overwritten with non-SQLite garbage bytes (file present, content invalid —
  the "vanished" case already covers "file absent") ⇒ measured, not guessed, by a throwaway
  probe (`Store::open` directly on a garbage-filled `project.db`, run once and deleted) before
  writing the assertion: this does **not** land in `WorkError::OpenFailed`/`work.open_failed` as
  the matrix's "reuse `WorkError::OpenFailed`/`MetaTooNew`" phrasing suggested — it fails inside
  `Store::open`'s `PRAGMA user_version` read (`schema::read_user_version`) before `open_work` is
  able to distinguish error classes at all, and surfaces as `StoreError::OpenFailed` /
  `store.open_failed` / `MessageKey::StoreOpenFailed`. Still a named, non-panicking error — the
  row's actual requirement — just a different key than the matrix prose guessed, because
  `db_path.exists()` is true for a corrupt-but-present file, so `open_work`'s own
  `WorkError::OpenFailed` branch (reserved for "absent"/"0-chapter" cases) is never reached for
  this condition. Recorded here rather than silently forcing the wrong expected key.
  **Counter-checks**: ① swapped `open_work`'s `MetaError::SchemaTooNew` arm to return
  `WorkError::OpenFailed` instead of `WorkError::MetaTooNew` — both this new case's assertion
  *and* the pre-existing Story 5.7 case
  (`opening_a_work_with_a_newer_meta_schema_is_refused_without_touching_a_single_byte`) went
  **red** on the same mutation (consistent — they guard the same seam). ② seeded a real write to
  `meta.json` between the "before"/"after" reads in sub-case (b) — the byte-equality assertion
  went **red**, proving it is not an assert-true-on-both-branches. Both reverted from backup
  (MD5-verified, `git status --short` clean against the pre-existing diff), re-ran green.

**"Write fails mid-append" — the highest-stakes row.**
`a_transaction_failure_mid_append_rolls_back_the_whole_batch_leaves_the_work_intact_and_removes_no_folder`
forces a *real* `SQLITE_FULL` inside `append_chapters_to_work`'s own transaction — no product
code changed to make this possible — by reading the destination's current `PRAGMA page_count`
and then, in a preceding `Store::write` job on the same connection, setting
`PRAGMA max_page_count` to `current_pages + 3` (a per-connection session pragma that persists
across jobs because `Store` owns exactly one writer connection for its lifetime, AD-11). Three
new Chapters (300 repeated sentences each, several hundred segment rows) then reliably exceed
that budget partway through the single transaction, so `Store::write` rolls back the whole job.
Asserted all three things the task named: ① `chapter_count` back to exactly 2 (no partial new
Chapter survives), ② the 2 pre-existing Chapters byte-identical via `snapshot_chapters`
before/after, ③ the `.atproj` folder, `project.db`, and `meta.json` all still exist.
**Counter-check**: added `if write_result.is_err() { remove_folder(&open.dir); }` right before
`write_result?;` in `append_chapters_to_work` — the literal violation §Never forbids, copied
from `create_work`'s own error-path shape. The case went **red** exactly on assertion ③ ("thu
muc .atproj ... khong duoc bi xoa"). Reverted from backup (MD5-verified), re-ran green. The
rollback mechanism itself (①/②) was not separately counter-checked beyond this — it rests on
`Store::write`'s own transaction contract, already proven generically by
`segment_contract.rs::a_failure_midway_leaves_neither_a_chapter_nor_a_segment` (AC13); this
case's own load-bearing, story-specific claim is the folder-removal prohibition, and that is
what was counter-checked.

**Verification, full run (2026-09-16).** `cargo check --locked`/`--tests`: clean. `cargo test
--locked --no-fail-fast`: **1552 passed, 0 failed, 20 ignored** (the prior 1548 baseline plus
these 4 new cases). `npm run build`: green, same two pre-existing warnings. `npm run test`:
**77 files / 1049 tests, unchanged** (no `src/**`/`tests/frontend/**` file touched). `npm run
check:deps && check:i18n && check:tokens && check:commands && check:layout &&
check:panel-refs && check:gates && check:debt-owner`: all eight exit 0. Every counter-check
above was run and reported, including the one ("nothing went red" is an acceptable answer) that
did not apply here — all five mutations attempted (one per new case, plus the extra sensitivity
check on sub-case (b) of "Destination unreadable") went red as expected; none needed to be
reported as a non-finding.

**Left incomplete / risky.** The nightly e2e `schedule` run — named in the top-level Acceptance
Criteria as a whole-story closing action — was not read or triggered by this pass; it was out
of scope for a matrix-coverage-audit fix and remains the closing agent's job. The nine `deferred
2026-09-16` items already recorded in `deferred-work.md`/Phase 4 are untouched by this pass.

### 🔵 2026-09-16 — Ice ruling: "Destination unreadable" fixes the CODE, not the matrix; nightly CI read before `done`

**The ruling.** The frozen matrix row "Destination unreadable" reads *"reuse
`WorkError::OpenFailed`/`MetaTooNew`"*. The matrix-coverage audit above measured that the
corrupt-`project.db` sub-case instead surfaces `StoreError::OpenFailed`/`store.open_failed`/
`MessageKey::StoreOpenFailed` — a store-layer code naming no Work — and recorded that as a
finding rather than a defect, since the frozen row was written before the probe existed. Ice
read that finding and ruled the CODE wrong, not the row: on the append confirm path, a corrupt
destination must surface `WorkError::OpenFailed { name, detail }`, so the message names the
Work the user just picked. The `meta.json`-too-new sub-case already yields `WorkMetaTooNew`
unchanged and needed no touch.

**What was built.** `commands::project::open_destination_for_append(work_id, indexed)` — a new
`pub fn` immediately after `open_work` (`src-tauri/src/commands/project/mod.rs`), wrapping
`open_work` without changing it. It passes every error through unchanged EXCEPT
`store.open_failed`: that one is remapped to
`crate::core::library::WorkError::OpenFailed { name: indexed's name, detail: format!("{err:?}")
}`. `open_work` itself is untouched — it still returns `store.open_failed` verbatim for its
OTHER caller, `wire::open_work` (Story 5.7's "reopen a `.atproj` from the Library"), where a
store-layer code is still the right diagnosis and Ice's ruling does not reach. Only the append
branch of `wire::confirm_import_with_encoding` (`wire.rs`, "Đường APPEND (Story 6.7b)") was
switched from calling `super::open_work(&work_id, indexed.as_ref())?` to
`super::open_destination_for_append(&work_id, indexed.as_ref())?` — its own two-`?` resolution
shape (`indexer.find_work` then this call) is otherwise unchanged.

`tests/project_contract.rs::confirm_append_is_refused_with_a_named_message_key_when_the_destination_is_unreadable`
(the pinning case, `:5045`) was updated to call `open_destination_for_append` on both sub-cases
(not raw `open_work`), so it keeps measuring the exact resolution the APPEND path takes.
Sub-case (a) (`meta.json` too new) is unaffected in outcome — the wrapper only intercepts
`store.open_failed`, and `open_work` already returns `WorkMetaTooNew` before `Store::open` ever
runs for that sub-case. Sub-case (b) (corrupt `project.db`) now asserts `err.code() ==
"work.open_failed"`, `err.message_key() == MessageKey::WorkOpenFailed`, and
`err.params().get("name") == Some("Kho Bi Hong")` (the destination's own display name) — where
it previously asserted `"store.open_failed"`/`MessageKey::StoreOpenFailed`.

**Counter-check, run as asked.** Backed up `src-tauri/src/commands/project/mod.rs` to the
session scratchpad before mutating (MD5 `1ce1fe5fab216ce03c21a620145c2a35` for the fixed file),
never restored from `git`/HEAD — the tree carries the uncommitted Phase 1-4 work a HEAD restore
would destroy. Mutated `open_destination_for_append`'s body to `open_work(work_id,
indexed).map_err(|err| err)` — the mapping removed, the function and the wire call site left in
place. Ran `cargo test --locked --test project_contract --
confirm_append_is_refused_with_a_named_message_key_when_the_destination_is_unreadable
opening_a_work_with_a_newer_meta_schema_is_refused_without_touching_a_single_byte
opening_a_work_whose_folder_has_vanished_is_a_named_open_failed_error`:

- With the mapping removed: **1 failed, 2 passed** — the new case fails exactly on
  `err_b.code()`, `left: "store.open_failed"`, `right: "work.open_failed"` (the sub-case (b)
  assertion), while both Story 5.7 cases
  (`opening_a_work_with_a_newer_meta_schema_is_refused_without_touching_a_single_byte`,
  `opening_a_work_whose_folder_has_vanished_is_a_named_open_failed_error`) stayed green — they
  do not touch a corrupt `project.db`, so the mutation cannot reach them.
- Restored from the scratchpad backup (not HEAD), verified by MD5 match
  (`1ce1fe5fab216ce03c21a620145c2a35`) and `git status --short` showing the file only as
  MODIFIED relative to HEAD (the same pre-existing Phase 1-4 diff, nothing added or removed).
  Re-ran the same three cases: **3 passed, 0 failed.**

**Blast radius, checked as asked.** `wire::open_work` (Story 5.7's own command) still calls
`open_work` directly, unwrapped — its behaviour for a corrupt `project.db` is byte-for-byte
what it was before this fix, since nothing about `open_work` itself changed. `resolve_
cleanup_rules_for_destination`/`resolve_work_tier_cleanup_rules_for_destination` (Phase 4's AC3
seam) also still call `open_work` directly and are unaffected — their best-effort philosophy
(fall back to 0 rules with a diagnostic) does not distinguish `store.open_failed` from any other
error today, so this fix changes nothing there either.

**Full verification after landing (2026-09-16).**

- `cargo check --locked` / `--tests`: clean, 0 warnings, 0 errors.
- `npm run build`: green (`dist/` produced before `cargo test`, per the project's own
  ordering rule), same two pre-existing `INEFFECTIVE_DYNAMIC_IMPORT` warnings as every prior
  phase, no new ones.
- `cargo test --locked --no-fail-fast`: **1552 passed, 0 failed, 20 ignored** — identical to
  the matrix-coverage-audit baseline immediately above (this fix changes an error's SHAPE, not
  which branch runs or how many cases exist).
- `npm run test`: **77 files / 1049 tests, all green** — unchanged (no `src/**`/
  `tests/frontend/**` file touched by this fix).
- `npm run check:deps && check:i18n && check:tokens && check:commands && check:layout &&
  check:panel-refs && check:gates && check:debt-owner`: all eight exit 0.

**Nightly e2e `schedule` — read before writing `done`, per the top-level Acceptance Criteria.**
Read run `35021855877` (started 2026-09-15T20:48Z, `schedule` trigger) — RED, the fifth
consecutive red night going back to 2026-09-11 (`AGENTS.md`'s own "SEVEN nights running" pattern
recurring on a shorter streak). Two jobs failed:

- `e2e (macos-26)` — `npm run test:e2e`; failing spec `e2e/specs/story-3-5-review.e2e.mjs`.
- `check (windows-2025)` — `cargo test`; failing case
  `the_wal_stops_growing_once_it_crosses_the_threshold` at `tests\store_contract.rs:890` — the
  subject of the existing `spec-ca-wal-do-tren-windows.md`, not a new failure this story
  introduced.

**Measured reason these are pre-existing, not this story's regression.** Both failures predate
this story's own `baseline_commit`, `7fb3aa7202044aefda0d1a0b40d5bf4cbef9276d` — run `35021855877`
ran against the commit at the head of `master` on 2026-09-15T20:48Z, before any of this story's
Phase 1-4 work (still uncommitted in this working tree) existed, and neither failing surface
touches the import path: `story-3-5-review.e2e.mjs` exercises Epic 3's review screen, and
`the_wal_stops_growing_once_it_crosses_the_threshold` exercises `core::store`'s WAL checkpoint
threshold on Windows — neither Chapter import, `append_chapters_to_work`,
`confirm_append_import_with_encoding`, nor any file this story's Phase 1-4 touched.

**Stated plainly, per the AC's own wording ("green, or red with the reason"): this run is RED,
for reasons this story did not cause and cannot fix from Rust/webview changes alone.** The
1552-case green `cargo test` run quoted throughout this story's Phase 1-4 (and reconfirmed just
above) was taken on **macOS only** (`Store::open`'s temp-dir plumbing and this whole session ran
on Ice's Mac) — the Windows half of this story's own test suite has never been measured, and
the one CI signal that DOES cover Windows (`check (windows-2025)`) was already red before this
story started, on an unrelated file. This story's own Windows correctness is therefore UNKNOWN,
not "probably fine because CI is green elsewhere" — `AGENTS.md`'s own "pre-push green is not
CI green" and "read the schedule run before `done`" rules exist for exactly this gap, and this
is the measurement they ask for, recorded rather than inferred.

### Review Pass 1 fixes (2026-09-16) — the eight patch-routed findings, one commit's worth of surface

Fixed every finding the §Review Triage Log routed to `patch` (E1b · E4+E5 · E6+E7+B5 · B2+B3 ·
B4 · B7 · B9 · B10/B11/VG1/VG2), each with the smallest change that closes it, each counter-checked
by breaking the guarded behaviour, confirming red, restoring from a scratchpad backup (never
`git`/HEAD — the tree carries the uncommitted Phase 1-4 work), and confirming green again.

**E1b — TOCTOU between the `already_open` check and the re-lock (`wire.rs:853-868`).** The lock
released at the end of the `is_some_and` closure and re-acquired a statement later; `replace_
open_work` (5 call sites, async commands on a threadpool) could swap the held Work in that
window, so `guard.as_mut()` then yielded the WRONG Work (silent append into it) or `None` (fatal
`.expect()` under `panic = "abort"`). Fixed by re-checking `open.meta.work_id == work_id` via
`guard.as_mut().filter(...)` immediately after re-acquiring, returning
`crate::core::library::WorkError::OpenFailed` (reused — `MessageKey::WorkOpenFailed`, no new
variant minted) on a mismatch instead of `.expect()`.
**Counter-check — nothing went red.** Reverted to the raw `.expect()` and ran the full suite
(`cargo test --locked --no-fail-fast`): 1552 passed, 0 failed, 20 ignored — byte-identical to the
fixed state. This crate has no `tauri::test`/`MockRuntime` harness (confirmed unchanged since
Phase 1), so no test in `tests/**` can invoke `wire::confirm_import_with_encoding` at all — the
exact same structural gap already documented repeatedly above for this same function's sibling
branches (e.g. the "destination vanished" case's "0 rows written" rests on a type-level argument,
not a runtime one, for the identical reason). Reported plainly rather than reshaped for a red.

**E4/E5 — `destinationPending` only tested `destinationWorkId === null` (`libraryImport.ts:474`).**
A stale id no longer present in `destinationWorks` (Work deleted/renamed between pick and submit)
and the `''` `LibraryMode.vue`'s disabled placeholder option emits both compared unequal to
`null`, so both slipped past the guard and shipped as a destination. Fixed by switching the
predicate to `pickedDestinationWork.value === null` — that computed already resolves `null`,
`''`, and any unknown id to `null` via its `.find()`, closing both holes with one change.
**Counter-check — red confirmed.** Reverted to `destinationWorkId.value === null`; both new
`destinationPending` cases (stale id, empty string) failed with `expected false to be true`.
Restored from the post-fix backup (MD5-verified, `git status --short` showed only the
pre-existing story diff), re-ran green (21/21 in the test file).

**E6 — the "Library trống" hint rendered on a load FAILURE too (`libraryImport.ts:150`).**
`destinationExistingWorkAvailable` (`!haveLoaded || length > 0`) is `false` both when the Library
is genuinely empty and when the last load errored (list stays `[]`), so the empty-library hint
and the error hint rendered side by side, disagreeing about why. Added a new computed,
`destinationLibraryGenuinelyEmpty` (`haveLoaded && length === 0 && error === null`), and pointed
`LibraryMode.vue`'s empty-hint `v-if` at it instead — `destinationExistingWorkAvailable` itself is
untouched (still drives the radio's `:disabled`, a separate, deliberately unchanged concern).
**Counter-check — red confirmed, two ways.** (1) Reverted the new computed to drop the
`error === null` clause; the unit case asserting `destinationLibraryGenuinelyEmpty === false`
during an error went red (`expected true to be false`). (2) The DOM-mount case (`LibraryMode.vue`
failing its `onMounted` load) went red too: `.hint:not(.hint-error)` existed alongside
`.hint-error`, proving the double-hint symptom is real, not just a computed's arithmetic.
Restored (MD5-verified), re-ran green.

**E7 — the picker never retried after one failed load (`libraryImport.ts:178`).**
`setDestinationMode` only called `loadDestinationWorks()` `if (!destinationWorksHaveLoaded)`, and
the failure path sets that flag `true` — one transient IPC hiccup locked the picker's retry path
for the rest of the session. Added `|| destinationWorksError.value !== null` to the condition.
**Counter-check — red confirmed.** Reverted to the single-clause guard; the case driving an
error-then-success sequence through `loadDestinationWorks()` then `setDestinationMode('existing')`
asserted `calls === 2` and got `1` (red). Restored (MD5-verified), re-ran green.

**B5 — `destinationWorks` loaded once, never refreshed (`libraryImport.ts`).** A Work created (or
appended to) earlier in the same session never appeared in the picker without a remount. Added a
`void loadDestinationWorks()` call inside `finishImportSubmission`'s `created !== null` branch,
right after the per-branch pending-field cleanup — best-effort, same philosophy as every other
secondary read in this module.
**Counter-check — red confirmed.** Removed the added call; the case asserting a second
`library_list_works` call after a successful `finishImportSubmission` got `1` instead of `2`
(red). Restored (MD5-verified), re-ran green.

**B4 — the three submit buttons always read "create a Work from …" (`LibraryMode.vue`).** Added
three new i18n keys (`mode.library.submit_text_append` / `_file_append` / `_urls_append` —
"Thêm Chương từ …") and a ternary on `destinationMode === 'existing'` at each of the three
button labels (`aura-allow-text` markers added, matching the file's existing ternary precedent at
the URL-count line — `check:i18n` Kiểm A2 cannot read a ternary statically).
**Counter-check — red confirmed.** Reverted just the "paste text" button's ternary back to the
literal `t('mode.library.submit_text')`; the case asserting all three buttons read "Thêm Chương…"
when an existing Work is picked failed on exactly that one button (`expected 'Tạo Tác phẩm từ văn
bản' to be 'Thêm Chương từ văn bản'`). Restored (MD5-verified), re-ran green.

**B7 — the two destination radios had no programmatic grouping (`LibraryMode.vue`).** Wrapped
them (plus the hints and the picker) in `<fieldset class="field field-group" data-import-destination">`
+ `<legend>`, replacing the bare `<div>`/`<span>`; added a CSS reset (`border: none; margin: 0;
padding: 0;` on `.field-group`, same on `legend`) since a bare `<fieldset>` carries UA-default
border/padding this project's token discipline forbids.
**Counter-check — red confirmed.** Reverted to `<div>`/`<span>` (both the open and close tags);
the new case asserting `group.element.tagName === 'FIELDSET'` failed (`expected 'DIV' to be
'FIELDSET'`). Restored (MD5-verified), re-ran green.

**B9 — `confirm_append_import_with_encoding_indexed` had 0 production call sites (1 definition +
6 test call sites only).** The append branch of `wire::confirm_import_with_encoding` hand-composed
`confirm_append_import_with_encoding` + a separate `reindex_library(&app, &root)` instead of
calling the `_indexed` sibling that already wraps that exact sequence through
`finish_lifecycle_write` — the case exercising `_indexed` proved something about a parallel
function, not the product path, and the two could drift apart silently.
🔵 **Not a blind swap — the "already open" sub-branch keeps the raw call, on purpose.** The
`_indexed` function's own doc-comment (Phase 4 correction #1) already explains why: it takes
`open: &mut OpenWork` and reindexes internally, so calling it while `open` is borrowed out of the
`OpenWorkState` `MutexGuard` (the "already open" sub-branch) would hold that lock through a
whole-Library disk scan — the exact risk `wire::set_chapter_status`'s doc-comment names, and the
reason that branch's own guard is scoped to drop BEFORE the shared `reindex_library` call runs
today. That branch is unaffected by this fix. Only the "not already open" sub-branch — where
`opened` is a freshly-opened `Store` with no shared lock held — was switched to call
`confirm_append_import_with_encoding_indexed(&mut opened, …, Some(indexer.inner()),
global.as_deref(), &root)` directly, and the trailing shared `reindex_library(&app, &root)` call
(previously unconditional after the `if`/`else`) is now gated `if already_open` so the
already-reindexed "not already open" path doesn't scan twice. Updated the `_indexed` function's
own doc-comment in place (🔵 dated) since its claim "not the function the wire calls, at all" had
gone half-false.
**Counter-check — nothing went red, reported plainly.** Reverted the "not already open" branch
back to the hand-composed `confirm_append_import_with_encoding` + an unconditional
`reindex_library(&app, &root)` (byte-for-byte the pre-fix shape) and ran the full suite: 1552
passed, 0 failed, 20 ignored — identical to the fixed state. Same structural reason as E1b: no
`tauri::test`/`MockRuntime` harness reaches `wire::confirm_import_with_encoding`, so a change to
which function its body calls is invisible to `cargo test` either way. The two pure-function
`_indexed` cases from Phase 4 correction #1 still pass either way — they call the pure function
directly and never touch this wire-level plumbing, which is exactly the gap B9 named.

**B10/B11/VG1/VG2 — `importPreviewDestination.test.ts` reached `submitPastedText` only.** A
regression swapping `effectiveSourceLang` back to `sourceLang` in `submitFilePath` or
`submitPastedUrls`, or in the confirm-time wiring, would have shipped green; the overlay's
`.ip-destination` banner, the `destinationWorksError` branch, and the radio-reset path had no
case anywhere. Added, all measuring IPC by the difference between mock call counts (per the
task's own instruction, not `not.toHaveBeenCalled()`):
- `submitFilePath`/`submitPastedUrls` each gained a case pinning that an existing-Work
  destination sends the destination's OWN `source_lang` (not the hand-typed one) and its
  `work_id`, exactly once — new `previewFileMock`/`startUrlMock` mocks added alongside the
  existing `previewTextMock`.
- `ImportPreviewOverlay.vue` is now mounted (two cases: New Work, existing Work) to read
  `.ip-destination`'s actual rendered text, not just the state it's built from.
- The `destinationWorksError`/empty-hint distinction (E6) and the retry path (E7) each gained
  their own case (folded in above).
- The radio-reset path (`setDestinationMode('new')` after a Work was picked clears
  `destinationWorkId`) gained a direct case.
- `LibraryMode.vue`'s `<fieldset>`/`<legend>` grouping (B7) and the button-wording swap (B4) each
  gained a mount-and-read case.
**Counter-checks — red confirmed for every new production-code assertion.** Seeded the exact
regression the task named (`sourceLang.value` in place of `effectiveSourceLang.value`) in
`submitFilePath` — red (`expected 'zh' to be 'en'`); same in `submitPastedUrls` — red; forced the
overlay's destination `<span v-if="importPreviewDestinationWorkId === null">` to `v-if="true"` —
red (banner kept reading "Tác phẩm mới" for an existing-Work destination). All three reverted
(MD5-verified backups), full file re-run green (21/21).

**Full verification after all eight fixes (2026-09-16, this pass).**
- `cargo check --locked` / `--tests`: clean.
- `cargo test --locked --no-fail-fast`: **1552 passed, 0 failed, 20 ignored** — identical to the
  Phase 4 baseline (this pass changed error SHAPES and call-site wiring, not which cases exist
  or how many; the two new project_contract.rs cases from the matrix-coverage audit were already
  counted there).
- `npm run build`: green, same two pre-existing `INEFFECTIVE_DYNAMIC_IMPORT` warnings.
- `npx vue-tsc --noEmit -p tsconfig.json`: clean, 0 errors.
- `npm run test` (vitest): **77 files / 1064 tests, all green** (baseline 77/1049 plus the 15 net
  new cases in `importPreviewDestination.test.ts`).
- `npm run check:deps && check:i18n && check:tokens && check:commands && check:layout &&
  check:panel-refs && check:gates && check:debt-owner`: all eight exit 0 — `check:debt-owner` now
  reports 518 open items (516 + the two new B2/B3 entries below), 0 missing a real `Chủ:`.

**Debt ledger — B2/B3, two new owned entries under the existing `## Deferred from:
6-7b-them-chuong-vao-tac-pham-co-san (2026-09-16)` header** (both self-disclosed in Phase 2's own
Implementation Notes but never given an owned entry until this pass):
- ① No Glossary scan runs on the newly-appended Chapters (`open.chapter_id` — the only cheap id
  available at the wire layer — is the open editor's cursor, not one of the new Chapters).
  **Chủ: Ice** — a product decision (scan which Chapter(s), and when), not a bug.
- ② A mid-transaction failure on the append path (after images are already written to `assets/`)
  leaves orphaned image files on disk, same class of risk already owned by Ice for `create_work`
  at `spec-6-11-anh-tai-ve-atproj-neo-vi-tri-va-url-goc.md`, but NOT closable the same way here —
  §Never spec 6.7b forbids the append path from deleting anything on error, so `create_work`'s own
  potential remedy (delete what was just written) does not transfer. **Chủ: Ice** — same owner as
  the 6.11 entry, cross-referenced so the two don't grow two different fixes for one failure class.

**Left untouched, and why.** No change to `append_chapters_to_work`'s own internals, the four-step
template, `resolve_work_tier_cleanup_rules_for_destination`, or any Phase 1-4 test case — every
finding routed `rejected` in the Review Triage Log (E1a, E2, E3, E8, B1, B8) stayed rejected, on
the same evidence already recorded there; nothing here reopens that triage.

## Spec Change Log

## Review Triage Log

### Pass 1 — 2026-09-16 (blind-hunter · edge-case-hunter · verification-gap)

| # | Finding | Verdict | Evidence |
|---|---|---|---|
| E1a | `.expect()` on `open_state` at `wire.rs:860` can fire | `false` | `already_open` is computed from `open_state.as_ref().is_some_and(...)` and `open_state` is a local binding that cannot change between the two statements. This specific `.expect()` is unreachable. |
| E1b | TOCTOU at `wire.rs:854-865`: lock released between the `already_open` check and the re-lock | `high` | Verified by reading: the guard in the `is_some_and` closure drops at the end of that expression, and a second `state.lock()` follows. `replace_open_work` has five wire call sites and Tauri dispatches `(async)` commands on a threadpool, so a swap in between is representable. `guard.as_mut()` then returns `Some` of a **different** Work and the append writes into it — no panic, no red gate, wrong destination. `None` (app teardown) hits the `.expect()` instead, fatal under `panic = "abort"`. |
| E2 | `OpenWorkState` guard held across the whole append, including network image fetches | `low` | Real: the guard spans `confirm_append_import_with_encoding`. But confirm runs behind the modal import overlay, so the editor paths that contend for this lock are not reachable by the user at that moment. Fix is a restructuring, not a direct correction ⇒ rejected under the `low` rule. Recorded here rather than dropped. |
| E3 | `resolve_cleanup_rules_for` holds `OpenWorkState` across a disk open of another Work | `low` | Same shape as E2, same reasoning, same rejection. |
| E4 | `destinationPending` false when `destinationWorkId` names a Work no longer in the list | `medium` | `libraryImport.ts:474-476` tests `destinationWorkId.value === null` only; `pickedDestinationWork` (`:129`) independently returns `null` for an unknown id, so `effectiveName`/`effectiveSourceLang` silently fall back to the typed fields while a stale id still ships. |
| E5 | `''` from the disabled placeholder defeats the `=== null` check | `medium` | Confirmed: `LibraryMode.vue:1297` renders `<option value="" disabled>` and `:1295` passes the raw value into `setDestinationWorkId(workId: string)` (`libraryImport.ts:182-184`), which assigns it unchanged. `''` is neither `null` nor a real id. |
| E6 / B6 | Load failure renders the "Library is empty" hint next to the real error | `medium` | `destinationExistingWorkAvailable` (`libraryImport.ts:150`) is `!haveLoaded \|\| length > 0`; on failure `:159-160` sets the error **and** `haveLoaded = true` with an empty list, so both messages show at once. The project's central failure class is exactly an empty list that does not say why. |
| E7 | A single transient IPC failure locks the picker out for the session | `medium` | `libraryImport.ts:178` retries only `if (!destinationWorksHaveLoaded.value)`, and the failure path at `:160` sets that flag to `true`. No retry anywhere. |
| E8 | Spec task text places the picker in `ImportPreviewOverlay.vue` | rejected | The claim is true, but its fix edits this build's spec, which triage rejects by rule. Already carried: §Code Map holds a dated 🔵 correction recording the same measurement and accepting the `LibraryMode.vue` placement. |
| B1 | The two new files appear under `=== NEW FILE:` markers, not real `diff --git` headers | `false` | That is how the orchestrator assembled the review staging file (tracked diff plus untracked files appended); it is a property of the review artifact, not of the change. |
| B2 | No Glossary scan on the append path, and no owned debt entry for it | `medium` | Verified: the omission is reasoned in `wire.rs`'s append branch and in Implementation Notes, but `deferred-work.md` gained only two new entries this story (bilingual path, language mismatch). `AGENTS.md:45` requires an owner for any clause not accepted at this layer. |
| B3 | Orphaned asset file on a mid-transaction failure, no owned debt entry | `medium` | Same shape as B2: disclosed in `append_chapters_to_work`'s doc-comment as accepted, never recorded with an owner. |
| B4 | Submit buttons still read "create a Work from …" when the destination is an existing Work | `medium` | `LibraryMode.vue` keeps `mode.library.submit_text`/`submit_file`/`submit_urls` unconditionally; the button misdescribes the action the user is about to take. |
| B5 | `destinationWorks` loaded once in `onMounted`, never refreshed | `medium` | `libraryImport.ts:178` is the only load trigger and is gated on `!destinationWorksHaveLoaded`. A Work created earlier in the same session cannot be chosen as an append target without a remount. |
| B7 | The two destination radios have no `<fieldset>`/`<legend>` grouping | `low` | True, and the fix is a direct correction (wrap the pair), so it is kept rather than rejected. |
| B8 | The picker is a flat `<select>` of every Work, no filter or search | `low` | Real at scale, but unlikely to be met in everyday use at today's Library sizes and the fix adds a search surface ⇒ rejected under the `low` rule. |
| B9 | `confirm_append_import_with_encoding_indexed` is `pub` with no production caller | `medium` | Measured: 1 occurrence under `src-tauri/src` (its own definition) and 6 under `src-tauri/tests`. `wire.rs` hand-composes the equivalent sequence instead of calling it, so the case that exercises it proves something about a parallel function, not about the product path, and the two can drift apart silently. |
| B10 / B11 / VG1 / VG2 | Frontend destination coverage is thin: `submitFilePath`/`submitPastedUrls`, the overlay's `.ip-destination` banner, the `destinationWorksError` branch, and the radio-reset path all untested | `medium` | Pre-verified by the verification-gap layer for the first two and re-checked here: `importPreviewDestination.test.ts` exercises `submitPastedText` only and never mounts `ImportPreviewOverlay`; grep for `ip-destination`, `destination_existing_work`, `destinationSourceLangLabel` across `tests/` returns zero. A regression swapping `effectiveSourceLang` back to `sourceLang` on the file or URL path would ship green. |

**Routing.** No `intent_gap` and no `bad_spec` ⇒ no loopback; `review_loop_iteration` stays `0`. Eight entries route to `patch` (E1b · E4+E5 · E6+E7+B5 · B2+B3 · B4 · B7 · B9 · B10/B11/VG1/VG2). Five findings rejected on their evidence above (E1a, E2, E3, E8, B1, B8).



## Design Notes

**Why a sibling function and not a parameter on `create_work`.** `create_work` is 509 lines
whose every failure branch ends in `remove_folder(&dir)` — ten of them, one *after* the
transaction commits. That shape is correct precisely because the folder did not exist a moment
earlier. Threading an `Option<existing>` through it would put a `remove_folder` on a path that
can reach a Work the user has been translating for months, and the compiler would say nothing.
The destructive default is the argument: the two operations differ in what failure means, and
that is exactly the kind of difference that should be two functions.

**Why the four-step template is not optional.** `library-index.db` is derived and `Indexer` is
its only writer (AD-8). A new write into `chapter`/`segment` that skips `Indexer::rebuild`
makes full-text search lie silently — finding retired sentences, missing new ones — and
`src-tauri/AGENTS.md:54` records the measurement that makes this concrete: removing step four
produced **0** failures across 34 binaries. That is why this spec asks for the removal
counter-check by name rather than trusting a green suite.

**Why the cleanup tier is an AC and not a detail.** `resolve_cleanup_rules` reads
`OpenWorkState`. Today the import preview always builds a Work that does not exist yet, so the
Work tier resolving to "whatever else is open" is a known, recorded defect
(`deferred-work.md:10063-10081`) with a limited blast radius. Once a destination can be an
existing Work, the same line becomes another Work's delete rules silently removing text from
this one — and the user sees a clean preview, because the preview is what applied them. This
story needs only the **read** half pointed at the right Work; authoring Work-tier rules stays
Ice's item.

**Why "append at the end" did not become an open question.** The UX mockup states it in words
— *"Thêm Chương vào cuối một Tác phẩm sẵn có"* (`web-import.html:201`) — and Story 5.8 already
owns reordering after import. `chapter.ord` carries no `UNIQUE` constraint and
`normalize_chapter_ord` exists, so a position chooser stays buildable later without a migration.
The repository answered this one; it is recorded as a decision, not put to Ice.

## Verification

**Commands:**
- `npm run build && cargo test --locked` -- expected: green, no expectation loosened.
  `dist/` must exist **before** `cargo test`, or the failure is at compile time, not an assert.
- `npm run test` -- expected: green against the Phase 1 baseline.
- `npm run check:deps && npm run check:i18n && npm run check:tokens && npm run check:commands &&
  npm run check:layout && npm run check:panel-refs && npm run check:gates &&
  npm run check:debt-owner` -- expected: 0 findings per gate.
- Counter-check ①: remove the `reindex_after_lifecycle_write` call from the append path, run
  the suite -- expected: **red**. Restore, re-run -- expected: green.
- Counter-check ②: `git stash` the new cleanup-tier case, run the **old** suite -- expected:
  **green** (the proposition was unguarded before).
- Counter-check ③: make the append path write at `ord = 1` instead of `MAX(ord)+1`, run the
  suite -- expected: **red** in the ordering case.
  🔵 **CORRECTED 2026-09-16 (orchestrator, after running it).** This line first demanded red
  *"in the 'old rows untouched' case, not only in the ordering case"* — **that expectation was
  wrong**. Measured: forcing `base_ord = 0` gives exactly **1** red,
  `append_writes_new_chapters_at_max_ord_plus_one_all_not_started_with_segments_for_each`, and
  the byte-equality case correctly stays green, because restarting `ord` does not modify a
  single existing row — it creates **duplicate** ordinals. Those are two different
  propositions and one case cannot carry both. ⚠️ What the correction exposes: `chapter` has
  no `UNIQUE` on `ord` (`core/store/schema.rs:1029`, deliberate), so duplicate ordinals are
  representable and the byte-equality case will never notice them.
- `cargo test --locked --test segment_contract` -- expected: `schema_version() == 22` and the
  AD-39 "nothing before the write step" cases green with their propositions unchanged.
- Re-run the same binary twice before blaming the code for a red run: LuLu treats every freshly
  compiled test binary as a new stranger (`AGENTS.md:39`).

**Manual checks (if no CLI):**
- Import 2 chapters into a Work that already has 3: the Library grid shows 5, the chapter list
  shows the new two last, and the first three open with their translations intact.
- Open Work X, then import into Work W with a Work-tier delete rule defined on X: the preview's
  "about to be removed" list shows nothing from X's rule.
- Close the preview without confirming: the destination `.atproj` folder's modification time is
  unchanged.
- Tab through the new destination control and its picker: focus stays inside the overlay.
