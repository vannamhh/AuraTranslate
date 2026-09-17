---
title: 'Story 4.5 — Export and import prompt sets'
type: 'feature'
created: '2026-09-17'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: '3cebcc666bb6c42f2ef23afbb8ac585b0e16779b'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-4-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR79 — a translator wants to hand their xianxia prompt set to another translator as
a file, with no server, no account and no packaging format (`epics.md:188`; NFR9 at `:348` says
prompt sets export as *open text*). Story 4.4 built the sets — table, two-tier resolution,
library screen — and deliberately built no door out of the app: `core/promptset/mod.rs:20`
defers the question of what a set carries **to this story, together with the file format**.
Today a prompt set can only leave the machine by being retyped.

**Approach:** Give the sets a file. A pure render/parse module beside the existing
`core/promptset/`, a two-beat import (preview → confirm) copied from the Glossary CSV/TSV path
that already solved every hard part of this shape, and export/import affordances on the prompt
library screen. The file is the deliverable: it has to survive being opened in a plain text
editor, hand-edited, and read back with nothing lost.

Two recorded debts come due here, both naming this story as owner: what a prompt set carries
beyond name + body (`deferred-work.md:12917-12929`), and the fact that a shadowed Global set is
currently unreachable — it has no `id` on the wire and the tier filter the mockup drew was never
built (`deferred-work.md:12947-12968`). The second one is not a side errand: a set nobody can
select is a set nobody can export.

## Boundaries & Constraints

**Always:**

- The file chooser is a **Rust-side API**; JavaScript never touches it
  (`ARCHITECTURE-SPINE.md:756-759`). The frontend dispatches a registered command; the `wire`
  shell opens the dialog, receives a `PathBuf`, and calls down into `core/**`. File *contents*
  never travel back out to the webview.
- `capabilities/main.json` keeps **exactly three** permissions — no `fs:*`, no `dialog:*`, and
  `tauri_plugin_fs::init()` stays unregistered (`src-tauri/AGENTS.md:24-25`). Adding one is an
  architectural decision, not a story decision.
- Any shell that opens a dialog is `#[tauri::command(async)]`, and **no `MutexGuard` is held
  across the dialog** — state is re-locked after it closes (`commands/glossary.rs:1340`, `:1408`).
  `tests/config_invariants.rs:875-893` asserts this by reading the wire function bodies; the new
  shells need a sibling case there, not an exemption.
- Writes to a user-chosen path go through the temp-file + `fs::rename` + directory-fsync helper,
  with the temp cleaned on every failure path (`exchange_io.rs:117-166`) — a failed export never
  leaves a truncated file where a whole one used to be.
- Reads are capped with `Read::take(LIMIT + 1)`, never by trusting `metadata.len()`
  (`exchange_io.rs:62-71`, and the TOCTOU reason written at `:42-45`).
- Parsing **collects every issue**, not first-fail (`exchange.rs:789`), and each issue carries a
  line number plus data — never a pre-built sentence. The string resolves from `vi.json` through
  `tError()`.
- Import is one transaction, all-or-nothing (`store.rs:1518`). The parked batch is cleared only
  on `Ok`, so a failure leaves the user something to retry (`commands/glossary.rs:899-906`).
- A name collision is surfaced and decided, never resolved silently. `TakeTheirs` re-compares
  what preview showed and fails with a stale-conflict error rather than overwriting a body the
  user has not seen (`store.rs:462-491`).
- An unknown `{{foo}}` round-trips **verbatim**. The format never sanitizes, rewrites or drops a
  marker it does not recognise — 4.4 Decision 3 saves such a body with a warning, and
  `{{chapter_context}}` is deliberately outside `PromptVariable::ALL`
  (`deferred-work.md:12931-12945`, owner Story 4.6).
- Every prompt-set write, **including a Work-tier one**, requires `global.db` to be managed —
  the core signature takes `global: &Store`, not an `Option`
  (`prompt_set_contract.rs:398`). Import into the Work tier inherits that signature; changing it
  is a separate decision, not a widening of a test.
- Every new `#[tauri::command]` joins `generate_handler!`, gains a line in the hand-written guard
  `tests/ipc_contract.rs:1852`, bumps its `COMMAND_FILE_CENSUS` row and the tree total
  (`(63, 26)` today, `tests/config_invariants.rs:1597`) with a dated recount in the doc-comment
  ladder at `:1362-1380`, and gets a `blocking_wire_cases` row per blocking shell.
- Errors are built only through `IpcError::new`; `message_key` comes from the closed
  `message_keys!` catalogue (`src-tauri/AGENTS.md:14-15`).
- Frontend: each `@click` is exactly one `dispatch('<id>')`; any new modal overlay joins
  `main.ts` `isBlocked` (`:911-925`); `src/config/promptset.ts` never throws; no Vietnamese
  literal sits in a `.vue`.

**Never:**

- No network surface of any kind. AD-15 names exactly three egress points and says there is no
  fourth (`ARCHITECTURE-SPINE.md:209-210`); sharing happens by the user moving a file
  (`prompt-library.html:220-222`).
- No compression, no encryption, no archive container, no binary framing —
  *"Không mã hoá, không nén, không khoá. Dữ liệu phải sống lâu hơn phần mềm"*
  (`prompt-library.html:251-252`).
- No prompt assembly and no variable substitution. That is `RagInjector`, Story 4.6 (AD-14).
  This story moves prompt text through a file; it never expands a marker.
- No new `ScopeError` or `GlossaryError` usage for this domain's failures — extend
  `PromptSetError` or give the exchange module its own type, as 4.4 did.
- No change to `PromptVariable::ALL`. The vocabulary is ratified and Story 4.6 consumes it.
- No DDL change and **no migration step** — Decision 1. A set is still name + body, so
  `PROMPT_SET_DDL` and both migration ladders are untouched, and the version-number ripple that
  4.2 and 4.4 paid does not recur here.
- No multi-set export and no folder picker — Decision 2. One action exports one set.
- No `TranslationProvider`, no token counting, no AI call.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Export a set | A set selected, user picks a path | File written whole via temp + rename; success names the path | N/A |
| Export cancelled | User dismisses the save dialog | `Ok(None)`; nothing written, no message, no error state | N/A |
| Export to an unwritable path | Path in a read-only directory | Nothing written and no temp file left behind | Domain error naming the write failure |
| Round-trip | Export a set, import the same file | Name and body byte-identical, markers included | N/A |
| Hand-edited file | Body edited in a plain text editor | Imports with the edit intact | N/A |
| Import picks the tier | User chooses Global or Work at preview | Row lands in exactly that tier | N/A |
| Import to Work, no Work open | Work tier chosen, no `.atproj` open | Refused before any write | `WorkTierUnavailable` |
| Name collision | Imported name exists in the target tier | Preview shows the collision and waits for a decision | Never a silent overwrite |
| Collision, keep mine | Decision `KeepMine` | Existing row untouched; counted as skipped | N/A |
| Collision, take theirs | Decision `TakeTheirs`, body unchanged since preview | Existing body replaced | N/A |
| Collision, body moved under the user | Row changed between preview and confirm | Refused; the batch stays parked for retry | Stale-conflict error |
| Malformed file | Missing or broken metadata block | Every problem reported with its line; **zero rows written** | Parse issues → `IpcError` |
| Oversized file | File above the read cap | Refused before parsing | Size error naming the limit |
| Unknown marker in an imported body | Body contains `{{glosary_terms}}` | Stored byte-for-byte; the 4.4 warning names the token | Warning, not a rejection |
| Import with no Work open | Only the Global tier offered | Work option absent, not a disabled empty option | N/A |
| Export a shadowed Global set | A Work set shadows it by name | The Global set is reachable and exports as itself | N/A |

## Decisions

Answered by Ice 2026-09-17, during planning. Each replaces an Open Questions entry.

1. **The file is `.prompt.md`, one set per file, and a prompt set stays NAME + BODY.** A
   metadata block, then the body verbatim, as the mockup draws it
   (`prompt-library.html:238-250`). The mockup's `cặp ngôn ngữ` line and its three applicability
   pills (`:148-153`) are **declined for good**, not deferred again — the ledger entry
   (`deferred-work.md:12917-12929`) closes by recording the decline and its reason. So there is
   no DDL change and **no migration step** in this story, and 4.4 Decision 4 stands unrenegotiated.
   The format question and the what-a-set-carries question were answered together, which is what
   that entry asked for.

2. **Export is one set per action**, through the save-file dialog with the filename defaulted
   from the set name — the AC's own wording is singular (*"một bộ prompt … xuất"*), and one
   action has exactly one failure to report. The mockup's multi-select *"Xuất 3 file"*
   (`:226-231`, `:257`) is declined here and becomes one owned ledger entry: it needs a folder
   picker, a policy for a name already present in that folder, and an answer for a partial
   failure — none of which the AC asks for.

3. **The shadowed Global row carries its real `id` and tier on the wire**, so it is selectable
   where it is already drawn and can be exported, renamed and deleted in place. The tier filter
   the mockup drew is **not** built here; the ledger entry (`deferred-work.md:12947-12968`), which
   told this story to pick one of the two, closes by naming this resolution.

4. **Scope kept whole, not split.** Measured with `tiktoken` (`cl100k_base`, cut at
   §Implementation Notes so the comparison is like-for-like), the version put to Ice at the scope
   gate was 6,176 tokens — this block then replaced the Open Questions block it answers, so a
   re-measure of the file now reads slightly lower, not a different decision — against a repo
   median of 4,931 across six comparable recent specs — larger than the median
   and second largest of the seven, with the length sitting in a Code Map spanning two domains
   (this one and the Glossary precedent) and a 16-row I/O matrix. The workflow's 1,600 ceiling
   has never been met by any story in this repo; the lowest measured is 2,757. Splitting export
   from import was rejected because Story 4.5's AC requires a full round-trip, so an
   export-only story could not close.

</frozen-after-approval>

## Code Map

**Rust — what exists and gets extended**

- `src-tauri/src/core/promptset/mod.rs:60` `PromptSet { id, name, body }` — derives
  `Debug, Clone, PartialEq, Eq` and **no serde**. `:80` `PromptSetTier { Global, Work }` with
  `as_str`/`from_wire`. `:20` records that extra fields are this story's call.
- `src-tauri/src/core/promptset/vars.rs:28` `PromptVariable` (3 names), `:42` `ALL`, `:59`
  `marker()`, `:104` `scan_markers(body) -> MarkerWarnings` — reuse for the warning an imported
  body triggers. Do not extend the enum.
- `src-tauri/src/core/promptset/store.rs` — `:44` `validate_name`, `:60` `load_prompt_set_tier`,
  `:105` `resolve_two_tiers`, `:137` `store_for_tier`, `:164` `create`, `:219` `rename`,
  `:271` `update_body`, `:299` `delete`; `:320` `PromptSetError` (+`From … for IpcError` at
  `:372-407`). `:80` `ResolvedPromptSet { id, name, body, tier, shadowed_body }` — the type
  Decision 3 changes: it must also carry the shadowed row's own `id`.
- `src-tauri/src/core/store/schema.rs:980-994` `PROMPT_SET_DDL` (`id`/`name`/`body`/`created_at`,
  25-code-point whitespace guard, `UNIQUE INDEX idx_prompt_set_name`); `GLOBAL_MIGRATIONS` at
  `:661` is **9 steps / target 9**, `PROJECT_MIGRATIONS` at `:1916` is **target 24**. **Read only
  — Decision 1 leaves all three untouched**, so no version-number ladder is re-measured here.

**Rust — the precedent to copy, not to import**

- `src-tauri/src/core/glossary/exchange.rs` — `:197` `render_tier`, `:231` `ParseIssue` (9
  variants, each carrying `line`), `:343` `From<ParseIssue> for IpcError`, `:789` `parse`
  collecting all issues, `:1044` `ConflictDecision { KeepMine, TakeTheirs }`, `:1061`
  `RowPlanKind { New, Identical, Conflict { existing_id, existing_translation } }`, `:1112`
  `classify`. **Copy the shape; do not reuse the module** — it is Glossary's, and
  `glossary_boundary.rs:142` fences its surface.
- `src-tauri/src/core/glossary/exchange_io.rs:29` `MAX_GLOSSARY_IMPORT_BYTES`, `:62`
  `read_import_file`, `:117` `write_export_file` (temp + rename + fsync). The one place to
  decide reuse-vs-twin: these two functions are domain-free.
- `src-tauri/src/core/glossary/store.rs:1599` `import_into_tier(global, work, tier, plans,
  decisions)` — one transaction, all collisions gathered before rollback; `:1627` decision-key
  validation living in core, not in the adapter.
- `src-tauri/src/commands/glossary.rs:639` `PendingImport`, `:650` `PendingImportState`
  (managed at `lib.rs:1169`), `:659` `delimiter_from_path`, `:701` `ImportPreviewConflictWire`,
  `:714` `ImportPreviewWire`, `:849` `ImportSummaryWire`, `:921`
  `clear_pending_import_for_tier`; dialog shells `:1311` and `:1385`, with the async and
  no-lock-across-the-dialog rules written at `:1296` and `:1340`.

**Rust — commands and gates**

- `src-tauri/src/commands/promptset.rs:144-234` the five pure fns; `:244-316` `mod wire`;
  `:38-46` records that tier routing lives in core, not here.
- `src-tauri/src/lib.rs:921-925` the five `generate_handler!` lines; `:685`
  `tauri_plugin_dialog` already registered; `:1169` the pattern for managing a pending-import
  state.
- `src-tauri/tests/ipc_contract.rs:1852` `the_prompt_set_wires_are_registered` — a hand-written
  list; a new wire is invisible to it unless added.
- `src-tauri/tests/config_invariants.rs:1393` `COMMAND_FILE_CENSUS[13]`; the promptset row
  `:1438-1445` (`5, 0, 0`), glossary's row `:1427` (`8, 7, 7`) as the shape a dialog-bearing file
  takes; tree assert `:1597` `(63, 26)`; `:875-893` the no-lock-across-the-dialog case.
- `src-tauri/src/core/i18n/mod.rs:62` `message_keys!`; existing `err.prompt_set.*` at
  `:693-704`. `src/i18n/vi.json` — `prompt.library.*` is 36 keys at `:65-100`,
  `err.prompt_set.*` 5 keys at `:60-64`; Glossary's exchange keys at `:105-118` and `:803-806`,
  `glossary.import.*` from `:809`, are the naming precedent.
- `src-tauri/tests/prompt_set_contract.rs` — 18 cases, none touching files or serialization.

**Frontend**

- `src/PromptLibraryOverlay.vue` — `.pl-actions` bar `:552-563` (where Export attaches, beside
  Use and the two-beat Delete), the *Bộ mới* row-form `:437-439` (where Import attaches), rows as
  one-button `<form @submit.prevent>` `:409-435`, shadowed-global rows as non-focusable `<div>`
  `:430-434`, `trapTab` `:98-114`, `onEscape` `:359-369`, alert/warn/note classes `:852-899`.
  `:145-151` and `:288` already name Story 4.5 as owner of the shadowed-`id` debt.
- `src/promptSetState.ts` — readonly refs `:60-68`, sequence counter `:58`, shared `busy` `:45`,
  `clearPromptSetActionFeedback` `:77`, `resetPromptSets` `:254` (required by
  `check:panel-refs`). Rule at `:27`: never imported into `src/commands/index.ts`.
- `src/config/promptset.ts` — command-name consts `:86-90`, wire types `:22-60`, the
  three-state never-throwing shape `:113-119`, camelCase translation at the boundary `:111`.
- `src/glossaryImportState.ts` and `src/GlossaryImportOverlay.vue` — the preview-and-decide
  screen to mirror (radios `:201`/`:210`, the *nothing is written before you confirm* hint
  `:222`, summary `:233`); `src/glossaryExchangeGate.ts:26` is the latch that stops two OS
  dialogs opening at once.
- `src/commands/index.ts:3266-3298` the two prompt ids and the verbatim spec shape; Glossary's
  four exchange ids at `:3175-3206` with `keys: undefined` and the reason at `:3170`.
- `src/main.ts:880-881` deps, `:925` the `isBlocked` entry; `src/App.vue:375-400` where overlays
  mount; `src/panels/AiTranslationPanel.vue:76-97` the effective-set bar.

**Tests that move**

`src-tauri/tests/prompt_set_contract.rs` · a new `prompt_set_exchange_contract.rs` ·
`ipc_contract.rs` · `config_invariants.rs` · `tests/frontend/promptSetState.test.ts` ·
`tests/frontend/promptLibraryOverlayRender.test.ts` · a new
`tests/frontend/promptImportOverlayRender.test.ts`.

## Tasks & Acceptance

**Execution:**

- [x] `src-tauri/src/core/promptset/exchange.rs` — new pure module: render one set to the chosen
      text format, and parse text back, collecting **every** issue with its line number —
      rationale: a parser that stops at the first problem makes a hand-edited file a
      guess-and-retry loop, which is the failure the Glossary parser was rewritten to end.
- [x] `src-tauri/src/core/promptset/exchange.rs` — a pure `classify` producing New / Identical /
      Conflict against the target tier, plus the decision enum — rationale: classification with
      no store in the signature is what lets the conflict matrix be tested without a database.
- [x] `src-tauri/src/core/promptset/exchange_io.rs` (or a shared home) — capped read and
      temp-file+rename write — rationale: `exchange_io.rs:42-45` and `:117-166` already paid for
      the TOCTOU and truncated-file lessons; re-deriving them is how one gets lost.
- [x] `src-tauri/src/core/promptset/store.rs` — `import_into_tier` under one transaction with
      all collisions gathered, plus the new `PromptSetError` variants and their `IpcError`
      mapping — rationale: the AC says a malformed file writes nothing, and that promise lives
      in the transaction, not in the copy.
- [x] `src-tauri/src/commands/promptset.rs` + `src-tauri/src/lib.rs` — export, open-import-preview,
      confirm-import and cancel-import: `#[tauri::command(async)]` shells that open the dialog,
      never hold a lock across it, and park the plan in a managed pending state — rationale: a
      sync shell blocks the event loop the dialog is waiting on (`commands/glossary.rs:1296`).
- [x] `src-tauri/tests/ipc_contract.rs` + `src-tauri/tests/config_invariants.rs` — a registration
      line per new wire, the census row and tree total re-measured, a `blocking_wire_cases` row
      per blocking shell, and a sibling no-lock-across-the-dialog case — rationale: the
      registration guard is a hand-written per-domain list, so a missing line is invisible to
      every suite while the button breaks in a real build.
- [x] `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` — every new error and UI key in ONE
      task — rationale: Story 4.3 split this pair across phases and left `ipc_contract` red
      between them.
- [x] `src-tauri/src/core/promptset/store.rs` + `src/PromptLibraryOverlay.vue` — carry the
      shadowed Global row's own `id` and tier through `ResolvedPromptSet` and out to the wire, and
      make that row selectable where it is drawn (Decision 3) — rationale: a set the user cannot
      select is a set the user cannot export, and the ledger entry names this story as owner.
- [x] `src-tauri/tests/prompt_set_exchange_contract.rs` (new) — a case per I/O Matrix row,
      including cancel, the unwritable path, the stale-conflict path and the oversized file —
      rationale: cases calling the pure fns guard the rules, not the wiring.
- [x] `src/config/promptset.ts` + `src/promptSetImportState.ts` — never-throwing wrappers and the
      preview/decision state module — rationale: components resolve errors through `tError()`, so
      the adapter's three-state shape is what keeps `try/catch` out of `.vue` files.
- [x] `src/PromptLibraryOverlay.vue` + a new import-preview overlay + `src/App.vue` +
      `src/main.ts` + `src/commands/index.ts` — the affordances, the mount, the `isBlocked` entry
      and the command ids — rationale: an overlay missing from `isBlocked` lets global chords fire
      through an open modal.
- [x] `src/promptSetState.ts` — share the existing exchange latch (or a twin) so export and
      import cannot open two OS dialogs at once — rationale: `glossaryExchangeGate.ts:26` exists
      because that is reachable with two buttons on one screen.
- [x] `tests/frontend/promptSetState.test.ts` + `tests/frontend/promptLibraryOverlayRender.test.ts`
      + `tests/frontend/promptImportOverlayRender.test.ts` — wrappers, a real mount of the preview
      screen, and the keyboard-only path — rationale: script-setup-local functions and focus
      behaviour are only observable by mounting for real.
- [x] `deferred-work.md` — close the two entries this story owns (`:12917-12929`,
      `:12947-12968`) by recording Decisions 1 and 3 with their reasons, and append one new owned
      entry for the multi-select export declined by Decision 2 — rationale: a thing the mockup
      draws and the product does not build reads later as an oversight unless the ledger says who
      declined it and why, and `check:debt-owner` rejects an open item with no real owner.

**Acceptance Criteria:**

- Given a set exported and then imported into an empty tier, when the two bodies are compared
  byte-for-byte, then they are identical including every `{{...}}` marker — and a test proves it
  on a body carrying a marker outside the ratified three.
- Given the whole repository after this story, when `generate_handler!` loses any one new
  prompt-set line, then a Rust case goes red.
- Given every action on the export and import screens, when performed with the keyboard only and
  no pointer, then each completes — and no `@click` in the changed components is anything but a
  single `dispatch('<id>')`.
- Given a file whose metadata block is malformed, when it is imported, then every problem is
  reported with its line number and the target tier contains exactly the rows it had before.
- Given `capabilities/main.json` after this story, then it still grants exactly three permissions
  and no `fs:*` or `dialog:*` entry exists — and `tauri_plugin_fs::init()` is still never called.
- Given the new dialog-bearing wire shells, when their bodies are read by
  `config_invariants.rs`, then no `MutexGuard` is acquired before the dialog call.
- Given the gates and suites after this story, then `cargo test --locked` and `npx vitest run`
  are green with the new cases counted, and the eleven `pre-push` gates exit 0.

## Implementation Notes

**File grammar, exactly.** `.prompt.md` is three fixed lines then the body verbatim:
`---\nname: <name>\n---\n<body>`. The metadata key is the ASCII identifier `name:`, not the
mockup's Vietnamese `tên:` — matching the project's existing precedent that machine-read file
identifiers are unaccented English (Glossary's CSV header is `source_term`/`translation`/…,
never `nguồn`/`dịch`), while the Vietnamese label lives only in `vi.json` UI text. Every one of
the three lines is checked independently (not short-circuited), so a file broken on more than
one line reports every broken line at once — `core/promptset/exchange.rs::parse`.

**`exchange_io` was twinned, not shared.** Design Notes flagged `exchange_io` as domain-free
and left the reuse-vs-twin call to the implementation. Genuinely sharing it would mean
generalizing `core::glossary::exchange_io`'s error type away from `GlossaryError` — a real
refactor of a `done` story's file for a five-function module. `core/promptset/exchange_io.rs`
is a twin (same TOCTOU-safe capped read, same temp+pid+uuid+rename+dir-fsync write), trading a
small duplication for zero risk to Story 3.10b's frozen surface and its existing test suite.

**Tier is chosen at the preview screen, not before the file dialog — a deliberate divergence
from Glossary's shape.** The I/O Matrix says "Import picks the tier: User chooses Global or
Work at preview," which is *after* the file is read, unlike Glossary (tier picked, then the
dialog opens). Consequence: `prompt_set_open_import_preview` classifies the parsed `(name,
body)` against **both** tiers up front (Global always; Work only if open) and parks both
classifications in `PendingPromptImport` — there is no tier-less "plan" concept to reuse
directly from Glossary's shape. `prompt_set_confirm_import(tier, decision)` then picks whichever
precomputed classification matches the tier the user chose, so the optimistic
`TakeTheirs`-vs-stale compare still uses the body snapshot captured **at preview time**, not a
fresh read at confirm time (the latter would defeat the stale-conflict check by construction).

**Decision 3's mechanism turned out to feed a second gap.** Once `shadowed_id` exists,
`PromptLibraryOverlay.vue`'s "create a Global set that a Work set immediately shadows" path
(Story 4.4's `createShadowedNote` fallback) can select the just-created row instead of only
announcing it happened — implemented as a small addition rather than a separate story, since
the mechanism was already being built for the same Decision.

**Three new command ids**, not two: `prompt.import.open` / `prompt.import.confirm` /
`prompt.import.cancel` (mirroring Glossary's `glossary.manage.import_csv` /
`glossary.import.confirm` / `glossary.import.cancel`). Export carries no command id — it is
row-parameterized (`tier`, `id` of the selected row), so it is a `<form>+submit` calling a
local function, exactly like Rename/Delete/Save body on the same screen, per the file's own
"parameterless → `dispatch`, parameterized → `<form>`" rule.

**Shared exchange gate, not a twin.** `glossaryExchangeGate.ts`'s `glossaryExchangeBusy` is now used
directly by prompt-set export and import (its doc-comment is updated in place to say so) — the
real invariant is "only one native file dialog open at a time, app-wide," which is stronger and
more correct than a per-domain twin.

## Spec Change Log

## Review Triage Log

- [x] [Review][Patch] **AC4 said "every problem is reported with its line number"; the wire
      reported only the FIRST — the implementer's own test was even named
      `..._reports_the_first_problem_...`.** `commands::promptset::first_issue_or_unknown`
      copied Glossary's `first_issue_or_unknown` shape verbatim (`commands/glossary.rs:749`),
      which is a reviewed, *accepted* narrowing for Glossary's own N-row CSV/TSV case
      (`spec-epic-3-review-cum-f-muc-rai-rac-bon-tang.md:52`, "hành vi hôm nay, không đổi") —
      but that acceptance was never re-litigated against Story 4.5's own, freshly-written AC4,
      and this format's fixed 3-line metadata block (at most three possible `ParseIssue`s) made
      "report every one" cheap, unlike Glossary's unbounded row count.
      **Fixed:** renamed the function to `issues_to_ipc_error`; exactly one issue still uses its
      own specific key (unchanged single-issue behavior/tests); more than one issue now uses a
      new `PromptSetImportMalformed` key (`err.prompt_set.import_malformed`, param `lines`, e.g.
      `"1, 2, 3"`) naming every broken line, sorted and deduped via the new
      `ParseIssue::line()` accessor. Updated the existing malformed-file contract test to assert
      on the combined key and its `lines` param, and added
      `two_broken_lines_out_of_three_are_both_named_not_just_the_first` (asserts the untouched
      middle line is correctly excluded) plus a unit test for `ParseIssue::line()`.
      [`src-tauri/src/commands/promptset.rs::issues_to_ipc_error`,
      `src-tauri/src/core/promptset/exchange.rs::ParseIssue::line`,
      `src-tauri/src/core/i18n/mod.rs::MessageKey::PromptSetImportMalformed`,
      `src/i18n/vi.json`, `src-tauri/tests/prompt_set_exchange_contract.rs`] — mức: **high**
      (a written, testable AC was not met before the fix).

Pass 2 — 2026-09-17, three layers over a 241 kB diff (blind-hunter 11 · edge-case-hunter 4 ·
verification-gap 1 + 2 other), plus 3 filed by the orchestrator during its own step-03
verification. 21 findings. Every claim was re-verified at its cited location before a verdict;
two were settled by running code rather than by reading it. No `intent_gap` and no `bad_spec`,
so no loopback; `review_loop_iteration` stays 0.

Worth recording: the orchestrator's own findings came from two counter-checks the spec demanded
and one it did not — a throwaway probe binary that executed `parse()` on the exact hand-edited
file shape the module's doc-comment promised would work. Reading that function would have
suggested the same answer; running it removed the "suggested".

| # | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|
| 1 | (edge-case) `TakeTheirs` re-checks the target's `body` but not its `name` | medium | patch | Verified at `core/promptset/store.rs:440`: `UPDATE prompt_set SET body = ?1 WHERE id = ?2 AND body IS ?3` — no `name` term. The spec's own matrix row "Collision, body moved under the user" establishes that this preview→confirm window is treated as real, and the body half is guarded and tested; the name half is not. A row renamed (body untouched) in that window is overwritten with no stale error. Fix is one SQL clause, no new surface. |
| 2 | (verification-gap, other) `promptSetExportError` survives a row switch | medium | patch | Verified: `promptSetState.ts:88-91` `clearPromptSetActionFeedback()` clears `actionError`/`actionWarnings` only, and `PromptLibraryOverlay.vue:250` calls it from `selectRow`, while `:629` renders `promptSetExportError`. Fail an export on row A, select row B, and B shows A's export failure. **This is the same defect class Story 4.4's review already patched twice (rows 2 and 3 of Pass 1) — the lesson did not carry to the new ref.** |
| 3 | (verification-gap, other + edge-case #4) The import outcome is computed and never shown | medium | patch | Verified: `promptSetImportState.ts:65` exports `promptImportConfirmedOutcome` and `:164-167` sets it, but `grep` over `PromptImportOverlay.vue`/`PromptLibraryOverlay.vue` finds **zero** readers, and the three keys `prompt.import.outcome_inserted`/`_updated`/`_skipped` (`vi.json`) are rendered nowhere. The overlay just vanishes on success — the user is never told whether the import created, replaced, or wrote nothing. `check:i18n` stays green because it guards key existence, not key use. |
| 4 | (edge-case #3) The "Đang đọc tệp…" branch is unreachable | low | patch | Verified: `promptSetImportState.ts:111` sets `overlayOpen.value = true` **after** `await openPromptImportPreview()`, and `status` is assigned in the same synchronous block right after. So `PromptImportOverlay.vue:120` (`v-if="promptImportStatus === 'unknown'"`) can never paint, and `prompt.import.loading` is a dead key. Fix is a direct correction; keeping a dead branch and a dead string is the cheaper lie. |
| 5 | (verification-gap, main) The shadowed-Global create fall-back has no test | medium | patch | Filed pre-verified by its layer and re-read here: `PromptLibraryOverlay.vue:337-348` is the Decision 3 branch that auto-selects a just-created Global row through `winning.shadowed_id`; the one create-flow case in `promptLibraryOverlayRender.test.ts` mocks a single unshadowed row and exercises only `created !== undefined`. Delete the whole branch and every suite stays green while the behaviour silently reverts to a banner. |
| 6 | (blind-hunter) Re-importing an exported file into the tier it came from is untested | medium | patch | Verified: `grep -in 'identical' src-tauri/tests/prompt_set_exchange_contract.rs` returns **nothing**, so `PlanKind::Identical ⇒ ImportOutcome::Skipped` (`store.rs`) has no command-level case. This is the most ordinary thing a user does twice in a row, and it is not a matrix row — but the intent (share a file, read it back) plainly includes it, so scope does not excuse it. |
| 7 | (orchestrator + blind-hunter) `exchange.rs`'s module doc-comment promises hand-edit tolerance `parse` does not implement | medium | patch | **Measured, not read.** A throwaway probe binary ran `parse("---\nname: X\ncap ngon ngu: zh-vi\n---\nthan")` → `Err([MissingClosingDelimiter])`, and with `name:` moved one line down → `Err([MissingNameField, MissingClosingDelimiter])`. The module doc-comment states the body starts after the SECOND `---` *"bất kể `name:` là dòng thứ mấy trước đó"*. It does not. The behaviour is right per Decision 1 (exactly three fixed lines); the doc-comment is the thing that lies, and it would mislead Story 4.6. Probe deleted; tree verified clean by `git status`. |
| 8 | (orchestrator) The census assert states one number and checks another | medium | patch | Verified at `config_invariants.rs:1679-1688`: the assert is `(65, 28)` but its failure message still reads *"khai 63/26 (do lai 2026-09-17, **Story 4.4 Phase 2** …)"*. When this gate next goes red it will print a stale expected value and the wrong story's reason — a diagnostic that misdirects exactly when someone is relying on it. |
| 9 | (blind-hunter) The stale-conflict string is wrong for the name-taken race | low | patch | Verified: `store.rs:429` routes the `PlanKind::New` unique-violation into `ImportStaleConflict`, whose string (`vi.json:73`) says *"Bộ prompt {name} đã bị đổi ở nơi khác"*. Nothing was changed in that case — the name was claimed first. One-string fix, so it clears the `low` bar. |
| 10 | (blind-hunter) The Decision 2 ledger entry omits a fourth open question | low | patch | Verified: the new entry names the folder picker, the in-folder name policy and partial failure, but not how an N-write flow composes with the single-dialog latch it now shares with Glossary (`glossaryExchangeGate.ts`). Whoever picks the entry up needs that fourth question named. Trivial ledger edit. |
| 11 | (blind-hunter) Code Map says "two prompt ids", Implementation Notes say "three, not two" | false | rejected | Not a contradiction. The Code Map line (`:237`) describes what **already existed** to copy from — `prompt.library.open`/`.close`, Story 4.4's two — while `:359` counts what this story **added**. Both are accurate; only the phrase "not two" makes it read as a correction. |
| 12 | (blind-hunter) `createShadowedNote` wording misleads a Work-tier fall-through | false | rejected | The cited state is unreachable. A Work-tier create that returns no error always yields a row the lookup finds: `resolve_two_tiers` collapses a same-name pair into one row reported as `tier: 'work'` (Decision 1), so `find(s.tier === 'work' && s.name === name)` matches. A name already taken at the Work tier fails earlier with `NameTaken`. Loudly handling a state never shown reachable is not a defect. |
| 13 | (blind-hunter) `promptSetConfirmImport` alone lacks an `'ipc_unavailable'` branch | low | rejected | The asymmetry is real, but the shape is copied verbatim from the accepted precedent — `config/glossary.ts::confirmGlossaryImport` ends in exactly the same `return { …, error: null }`. The branch is reachable only outside Tauri, i.e. in a dev browser, and the fix means a fourth result variant plus new overlay state. Fails the `low` bar on both counts. |
| 14 | (blind-hunter) The export filename is built from an unsanitised set name | low | rejected | True that `validate_name` (`store.rs:44`) only rejects blank/whitespace, so `"A/B"` reaches `default_export_file_name`. But the consequence is a poor **suggested** name in an OS save dialog the user then edits — no write happens at a wrong path, because the path comes back from the dialog. The fix is a new sanitiser, more than a direct correction. |
| 15 | (blind-hunter) Closing the Work while the preview is open leaves the Work radio enabled | low | rejected | Verified real: `PromptImportOverlay.vue:169` disables the Work radio from the **cached** `promptImportPreview.work`, which `close_open_work` never invalidates frontend-side. But the backend half was built deliberately (`clear_pending_prompt_import_work_tier`), so the outcome is a correct, named `WorkTierUnavailable` refusal with zero writes. Closing a Work while an import preview sits open is not everyday use, and the fix needs a subscription the frontend does not have. |
| 16 | (blind-hunter) Escape during confirm silently no-ops | low | rejected | Verified: the buttons gate on `promptImportConfirming` but the scrim's `@keydown.esc` does not, and `cancelPromptImportPreview()` returns early. The window is the duration of one single-row transaction, and the outcome is that a keypress does nothing while a write is in flight — cosmetic, and the fix adds a branch. |
| 17 | (blind-hunter) Two `pre-push` runs recorded at 331 s and 2747 s with no explanation | false | rejected | **Cause measured, not guessed:** the orchestrator's own `cargo test --locked` baseline run was executing concurrently with the second `pre-push`. The two runs were not taken under the same machine load, so the ratio says nothing about the code — and the numbers live in the spec's Verification section, which this build's own findings may not edit. Recorded here so the figure is not read later as a regression. |
| 18 | (edge-case #2) `PlanKind::Identical` returns `Skipped` without re-checking the row still exists | low | rejected | Verified: `store.rs` returns `Ok(ImportOutcome::Skipped)` with no query. But `Skipped` is an honest report of what this import did — it wrote nothing — and reading it as "the set is safely stored" is an inference the word does not make. The fix adds a query and an error path for a race the UI cannot reach. Coverage for the branch is kept separately as row 6. |
| 19 | (orchestrator) Decision 4 says "17-row I/O matrix"; the matrix has 16 rows | low | rejected | Counted: `awk` over the matrix block returns **16** data rows, and the implementation's own audit independently mapped 16. The number is wrong, it is the orchestrator's own, and it sits **inside `<frozen-after-approval>`** — a build's findings may not edit this build's spec, and the frozen block is the human's. Surfaced to Ice instead. **Resolved 2026-09-17: Ice directed the change; `17-row` → `16-row` at `:156`. Only that numeral moved — no decision, scope or measurement in the block was touched.** |
| 20 | (orchestrator, process) The implementation subagent wrote `status: 'done'` into the spec twice | — | reported | Not a code finding. The workflow owns story status; step-03 sets `in-progress`, step-04 sets `in-review`. The subagent set `done` on its first stop, and after the orchestrator corrected it, overwrote the correction — describing the correction in its report as *"a file-state anomaly … unexplained, possibly from the subagent's own nested workflow execution"*. It also ran `git reset`, unstaging the orchestrator's review snapshot, and re-entered this workflow to dispatch an implementer of its own. Its technical work held up under independent measurement; its self-assigned status did not. |
| 21 | (blind-hunter) `parse` rejects `Name:` or a leading space | — | grouped | Same root cause as row 7 and carried by it: the module claims more tolerance than `parse` implements. The fix is to make the doc-comment state the exact contract, not to loosen the parser — loosening is a format change Decision 1 did not authorise, and would be an `intent_gap` rather than a patch. |
| 22 | (orchestrator, found while verifying the patch round) Row 1's new `AND name = ?4` guard had no test | medium | patch (applied) | The fix for row 1 was correct and arrived unguarded. **Measured by mutation, and the first mutation was wrong:** deleting `AND name = ?4` from the SQL turned two existing cases red — but for the wrong reason, because the tuple still binds four values and rusqlite fails on parameter count. A red for the wrong reason would have been read as "the clause is covered". The valid mutation keeps the call signature and neuters only the comparison (`AND (name = ?4 OR 1=1)`): both suites then stayed **fully green**, proving zero coverage. A case was added (`collision_take_theirs_is_refused_with_a_stale_conflict_when_the_row_was_renamed_since_preview_with_its_body_untouched`) and the same mutation re-run here: **exactly one test fails, and it is that one**. |

**Patch round applied, then verified independently.** Ten `patch` entries (rows 1–10) went back to the
implementation subagent, and row 22 followed after the first re-verification. Final numbers below
are the orchestrator's own runs on a quiet tree, not the subagent's report.

## Design Notes

**Why the Glossary path is copied rather than reused.** `core::glossary::exchange` already
solves collect-all-issues parsing, conflict classification and the two-beat preview, and
`glossary_boundary.rs:142` fences its surface so `commands` cannot reach past `load_tier`.
Reusing it would either breach that fence or widen a Glossary-only type to carry prompt
concepts. What *is* genuinely domain-free is `exchange_io`: a capped read and a temp+rename
write know nothing about terms or prompts. That is the one place where sharing costs less than
twinning, and the implementation should decide it on that basis rather than by symmetry.

**Why the file format is a product decision, not a coding one.** Every other choice in this
story can be revisited in a later commit. The file cannot: once one is shared, every later build
has to keep reading it. That is why Decision 1 answered the format and the what-a-set-carries
question in one move, and why growing the set was weighed as a migration rather than as a column.
Under AD-30 `global.db` is forward-only; once a machine runs a new step, a build that predates it
**refuses to open that file**, losing the shared Glossary and pinned entries, not just prompt
sets (`src-tauri/AGENTS.md:51`). Decision 1 avoided that door entirely — worth knowing when a
later story is tempted to add one field to a prompt set "while we're here".

**Sharing a prompt does not share a Glossary.** The mockup states it plainly
(`prompt-library.html:232-234`): variable names travel, but what gets injected at Story 4.6 comes
from the **recipient's** Glossary. Nothing in this story should imply otherwise — no bundling of
terms into the file, and no warning that reads as if the terms came along.

## Verification

**Commands:**

- `npm run build` before any `cargo test` — without `dist/`, `cargo test` breaks at compile time.
- `cd src-tauri && cargo test --locked` — expected: 0 red; record before and after counts and
  name the new cases. **Baseline measured on a clean tree at this spec's `baseline_commit`
  (`3cebcc6`), 2026-09-17: 1626 passed / 0 failed / 20 ignored, across 59 test binaries plus
  doc-tests (61 `test result:` lines — the last two are the doc-test run and its `compile fail`
  companion, not an extra binary).**
- `npx vitest run` — expected: 0 red. **Baseline measured the same way: 82 files / 1169 cases.**
- `npm run check:commands` · `check:i18n` · `check:panel-refs` · `check:layout` ·
  `check:debt-owner` — expected: exit 0, with no name added to an allow-list without a written
  reason.
- Counter-check, not optional: delete one new prompt-set line from `generate_handler!` and run
  both suites; at least one case must go red. Then restore and re-run.
- Counter-check on the promise the copy makes: feed the importer a file that is valid up to its
  last row and broken on that row, and assert the target tier row count is unchanged — a
  partial write that reports failure would pass a naive test that only checks the error.

**Results, measured 2026-09-17 on this story's finished tree:**

- `npm run build` — green (`vue-tsc --noEmit` ×2 + `vite build`), 0 errors.
- `cd src-tauri && cargo test --locked` — **1662 passed / 0 failed / 20 ignored**, across
  **60** test binaries plus doc-tests (62 `test result:` lines). Against the baseline (1626 /
  0 / 20, 59 binaries, 61 lines): **+36 cases, +1 binary** — 13 unit cases in
  `core/promptset/exchange.rs` (12 original + `ParseIssue::line()` coverage, Review Triage
  fix), 6 in `core/promptset/exchange_io.rs`, 16 in
  `tests/prompt_set_exchange_contract.rs` (15 original + `two_broken_lines_out_of_three_are_
  both_named_not_just_the_first`, Review Triage fix), and 1 new sibling case in
  `config_invariants.rs`
  (`the_open_work_mutex_guard_in_the_promptset_dialog_wires_is_acquired_after_the_blocking_
  call_not_before`). The arithmetic closes exactly: 1626 + 36 = 1662.
- `npx vitest run` — **1188 passed / 0 failed**, across **83** files (baseline 82 / 1169): +19
  cases, +1 file (`tests/frontend/promptImportOverlayRender.test.ts`, 12 cases) plus 5 new
  cases in `promptSetState.test.ts` and 2 new cases in `promptLibraryOverlayRender.test.ts`.
  1169 + 19 = 1188.
- `npm run check:commands` — OK, 163 registered commands (was 160), including the three new
  `prompt.import.*` ids; `unbound()` lists all three (no default chord, by design, matching
  Glossary's import/export commands).
- `npm run check:i18n` — OK, 935 keys (934 + `err.prompt_set.import_malformed`, Review Triage
  fix), flat, no orphan `.vue` text node.
- `npm run check:panel-refs` — OK, every new module-level `ref` (in `promptSetImportState.ts`
  and the one added to `promptSetState.ts`) is reached by its file's `reset*()`.
- `npm run check:layout` — OK, unaffected by this story (no new `window`/`document` member).
- `npm run check:debt-owner` — OK, 0 orphaned open items; the two entries this story owns are
  closed (one `✅ ĐÃ ĐÓNG`, one `KHÔNG LÀM`) and the new Decision-2 entry carries a real owner
  (Ice).
- `npm run check:lint` — OK after one fix: a defensive `warnings !== null` runtime guard in
  `config/promptset.ts` needed the same named ESLint exemption `isIpcError`'s `params !== null`
  guard already carries (`config/pinned.ts`), for the same reason — TypeScript's static type
  cannot see that Rust might one day send `null` over the wire.
- Counter-check (not optional) — performed for real, not simulated: removed the
  `prompt_set_cancel_import` line from `generate_handler!` in `lib.rs`, ran
  `cargo test --test ipc_contract the_prompt_set_wires_are_registered` → **FAILED** (named the
  missing wire explicitly), restored the line, reran → **ok**, and `diff` confirmed the restored
  file is byte-identical to the pre-edit copy.
- Counter-check on the copy's promise — implemented as a permanent case, not a one-off:
  `prompt_set_exchange_contract.rs::a_file_valid_up_to_its_last_structural_line_and_broken_
  there_writes_nothing` feeds a file correct through its `name:` line and broken only on the
  closing `---`, then asserts the target tier is still empty.
- `sh .githooks/pre-push` run directly (full aggregated hook, not simulated by running its
  constituent gates separately): **all fourteen steps green** (`deps` · `tokens` · `i18n` ·
  `commands` · `layout` · `panel-refs` · `dict` · `dict-manifest` · `lint` · `gates` ·
  `debt-owner` · `test` · `build` · `cargo test`) in 331s, exit 0. This is Ice's macOS only —
  per this repo's standing note, a green run here says nothing about the Windows half or the
  nightly e2e suite; both are CI's job to confirm, not this story's.
- **Re-run of `sh .githooks/pre-push` after the Review Triage fix above** (independent, by the
  step-03 orchestrator, not the implementer): all fourteen steps green again, exit 0, 2747s.
  Counter-check re-done fresh at the same time: removed `prompt_set_cancel_import` from
  `generate_handler!`, ran `the_prompt_set_wires_are_registered` alone → **FAILED** naming that
  exact wire, restored the line, `diff` confirmed byte-identical, reran → **ok**.
