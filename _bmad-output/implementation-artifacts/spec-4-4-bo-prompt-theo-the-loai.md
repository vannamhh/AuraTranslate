---
title: 'Story 4.4 — Genre-specific prompt sets'
type: 'feature'
created: '2026-09-17'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: '8c816b990c5e273f04c5e6ea189032237b788c15'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-4-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR69 — one translator works across genres and needs a xianxia prompt set and a
journalism one, per Work as well as globally. `ScopeKind::Prompt => "prompt" : Override` has
been reserved since Story 1.15 (`core/scope/kinds.rs:165`) with nothing behind it: no table,
no module, no command, no screen. Settings already shows a body-less `'prompt'` nav slot owned
by "Epic 4" (`src/settingsState.ts:113`).

**Approach:** Give that reservation a body. A `prompt_set` table as a twin step in BOTH stores,
resolved through the existing `ScopeResolver::apply_override` under kind `"prompt"`; a prompt
library overlay for compose / rename / delete; and an effective-set line with a switcher on the
AI Translation panel so changing sets does not mean opening Settings.

This is the repo's **first named collection** under `ScopeResolver`. Glossary keys by
`source_term` and AI config keys by field name — both flat scalar keys. Here the key is a
user-chosen set NAME and the value is the set body.

## Boundaries & Constraints

**Always:**

- Two-layer IPC (`src-tauri/AGENTS.md:13`): pure fns taking `Option<&Store>` + `Option<&OpenWork>`,
  thin `#[tauri::command]` shells in `mod wire` using `try_state`, never `state()`.
- Every write through `Store::write` (`core/store/mod.rs:678`); no `unwrap`/`expect` in a write
  closure — `panic = "abort"` makes it process death.
- Resolution goes through `resolver.apply_override("prompt", global, work)`. The `Resolved<V>`
  shadowed global must survive into the response: the mockup renders it
  (`prompt-library.html:134`, *"bị prompt cùng tên ở trên che"*).
- The set-name empty-guard in DDL lists all 25 `White_Space` code points, same as
  `GLOSSARY_ENTRY_DDL` — SQLite `trim()` strips only ASCII spaces (`src-tauri/AGENTS.md:37`).
- No `tier` column (each tier is its own db file) and no `CHECK` enumerating names — the closed
  Rust type is the enforcement, as for `ai_config` (`schema.rs:895-903`).
- Every new `#[tauri::command]` is added to `generate_handler!` AND to a registration guard in
  `tests/ipc_contract.rs`; the census row for its file is re-measured.
- The three ratified variable names live in ONE closed Rust type that both the validator and the
  compose screen's variable list read from. A second hand-written list anywhere is a defect —
  the same rule `message_keys!` carries (`src-tauri/AGENTS.md:15`).
- A prompt body is never rejected for its markers. Both marker conditions — an unknown token, and
  a missing `{{glossary_terms}}` — warn and save.
- Frontend: each `@click` is exactly one `dispatch('<id>')`; the new overlay is registered in
  `main.ts` `isBlocked`; `src/config/promptset.ts` never throws.

**Never:**

- No language pair and no applicability field on a set — Decision 4. A set is name + body.

- No prompt ASSEMBLY and no variable substitution — that is `RagInjector`, a pure function,
  Story 4.6 (AD-14). This story stores and displays prompt text containing markers; it never
  expands one.
- No export or import — Story 4.5 (FR79). But do not foreclose it: the set NAME is the
  collision key, tier is assignable rather than part of identity, and no field is opaque
  or encoded.
- No `TranslationProvider`, no network call, no token counting.
- Do not reuse `ScopeError` for this domain's failures — it is deliberately Glossary-only
  (`deferred-work.md:6073`); build this domain's own error type, as `AiConfigKeyError` did.
- Do not add a fourth mode. A big screen is the 11th overlay (`src/SettingsOverlay.vue:4-8`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Create at Global | No Work open, valid name | Row in `global.db`; set listed under Toàn cục | N/A |
| Create at Work | Work open, name already at Global | Row in `project.db`; Work set wins, global one listed as shadowed | N/A |
| Blank-ish name | Name is `"\u{3000}"` or `"\t"` | Rejected before SQL | Domain error → `IpcError`, name field flagged |
| Duplicate name in same tier | Name exists in that tier | Rejected; existing row untouched | Domain error naming the collision |
| Rename to an existing name | Target name taken in same tier | Rejected; both rows untouched | Domain error |
| Delete a shadowing Work set | Work set shadows a Global one | Work row gone; the Global set becomes effective | N/A |
| List with no Work open | Global rows only | Work group absent, not an empty group | N/A |
| List with zero sets anywhere | Both tiers empty | Empty state saying nothing is configured | Never a `0`-means-two-things count |
| Switch effective set from AI panel | Work open, two sets resolvable | Effective set changes without opening Settings | N/A |
| Resolve when store missing | `global.db` unopened | Reported as unavailable, not as "no sets" | Domain error; never a silent empty list |
| Save a prompt containing `{{glosary_terms}}` | Token is not one of the three | Saved verbatim; a warning names the unknown token | Warning, not a rejection |
| Save a prompt with no `{{glossary_terms}}` | Required marker absent | Saved; a warning says Glossary Enforcement is off for this set | Warning, not a rejection |

## Decisions

Answered by Ice 2026-09-17, during planning. Each replaces an Open Questions entry.

1. **Override is WHOLE SET, keyed by name.** A Work-tier set whose name matches a Global one
   replaces it entirely; the shadowed global stays visible in the list. This decision is
   recorded in the `core/scope/kinds.rs:165` doc-comment in the shape its two neighbours use
   (`Glossary` "theo từng thuật ngữ", `AiConfig` "theo TỪNG TRƯỜNG", both citing their mockup
   and carrying Ice's signature). It is **not** a new AD — those two settled the same question
   the same way, and the `AiConfig` comment states the reason: settling it at the kind table is
   cheaper than settling it inside Epic 4.

2. **This story ratifies a closed three-name variable vocabulary** and owns it:
   `{{glossary_terms}}` · `{{source_segment}}` · `{{tm_similar_segments}}`. `{{chapter_context}}`
   is excluded — the mockup marks it *chưa dùng*. Story 4.6's `RagInjector` consumes this enum
   rather than declaring a second list. Until now these names existed only as mockup content;
   this is where they become code.

3. **An unknown `{{foo}}` is saved verbatim and warned about, naming the token.** Not rejected —
   that would ban writing `{{` as ordinary text. Not silent — a typo'd marker that quietly
   expands to nothing at Story 4.6 is exactly the failure class `AGENTS.md:62` names as this
   project's central one. Same shape as the warning shown when `{{glossary_terms}}` is absent.

4. **A prompt set is NAME + BODY only.** The mockup's language pair and applicability pills
   (`Áp cho mọi segment` / `lời thoại` / `tả cảnh`) are not stored: no FR and no AC asks for
   them, and nobody has standardised what they mean. They become one debt item owned by Story
   4.5, which decides them together with the export format.

5. **Scope kept whole, not split.** Measured at planning time (cut at §Implementation Notes so
   the comparison is like-for-like), this spec is 4,788 tokens — 4,662 when Ice answered, plus
   the Decisions block itself — against a repo median of 5,132
   across the five comparable recent specs — smaller than each of the last three. The workflow's
   1,600 ceiling has never been met by any story in this repo; the lowest measured is 4,139.

</frozen-after-approval>


## Code Map

**Rust — persistence and scope**

- `src-tauri/src/core/store/schema.rs:657` `GLOBAL_MIGRATIONS` (len 8, target 8) and `:1832`
  `PROJECT_MIGRATIONS` (len 22, target 23). A `PROMPT_SET_DDL` const plus a twin step in both —
  the shape `AI_CONFIG_DDL` used at `:699` / `:1977`.
- `src-tauri/src/core/store/schema.rs:881-910` — `AI_CONFIG_DDL` and its doc-comment: why no
  `tier` column, why no `CHECK` on keys, why values are `TEXT`. Copy the reasoning, not the
  columns; this table needs a real name column with the whitespace guard at `:301-330`
  (`GLOSSARY_ENTRY_DDL`, whose 25-code-point `char(...)` list ends at `:879`).
- `src-tauri/src/core/scope/kinds.rs:165` — `Prompt => "prompt" : Override`, already declared
  and locked by `tests/scope_contract.rs:169`. **The row stays; its doc-comment gains the
  granularity clause** once Open Question 1 is answered.
- `src-tauri/src/core/scope/mod.rs:293` — `apply_override(kind, global, work)`. Generic over
  `BTreeMap<K, V>`, so keying by set name needs no resolver change. `Resolved::shadowed()` at
  `resolve.rs:76` is what the shadowed-global row renders from.
- `src-tauri/src/core/aiconfig/store.rs:58,91,114` — `resolve_two_tiers` / `write_field` /
  `clear_field`: the store-module shape to follow. Its `From … for IpcError` impls at `:163`,
  `:183`, `:240` are the error-conversion pattern.
- `src-tauri/src/core/store/mod.rs:678` — `Store::write`, the only write door.

**Rust — commands, i18n, gates**

- `src-tauri/src/commands/aiconfig.rs:4-8` (convention), `:44` `store_for_tier`, `:265` `mod wire`
  — the two-layer template, including shells that take no `AppHandle`.
- `src-tauri/src/lib.rs:712` `generate_handler!`; aiconfig entries `:908-916`.
- `src-tauri/tests/ipc_contract.rs:1824` `the_aiconfig_key_wires_are_registered` — **a
  hardcoded per-domain list, not a generic check**; its own doc-comment records that the three
  Story 4.2 commands remain unguarded. New prompt-set wires get zero coverage unless a sibling
  guard is added. Same mould: `:1359`.
- `src-tauri/tests/config_invariants.rs:1389` `COMMAND_FILE_CENSUS` (12 rows) and its enforcer
  `:1489` — a new `commands/promptset.rs` needs a new row with a non-empty `why`.
- `src-tauri/src/core/i18n/mod.rs:62` `message_keys!`, catalogue block from `:100`; `IpcError`
  at `:732`, sole constructor `:775`. `src/i18n/vi.json` — 847 keys today; `settings.nav.prompt`
  already exists at `:783`.

**Frontend**

- `src/settingsState.ts:40,53,71,113` — the body-less `'prompt'` section and its "Epic 4" owner
  label; `settingsSectionHasBody` at `:97` is the flag to flip.
- `src/GlossaryManageOverlay.vue` — **the screen to copy for the keyboard AC**: `role="listbox"`
  with `aria-activedescendant` `:458-470`, arrow keys `:316`, Enter-to-edit, two-beat Backspace
  delete `:328-334`, `trapTab` `:215`, esc `:350-352`.
- `src/SettingsOverlay.vue:341-404` — Story 4.3's row shape: two SIBLING `<form @submit.prevent>`
  elements (HTML forbids nesting), status line, error precedence.
- `src/App.vue:375-400` — where the ten overlays mount; the new one is the 11th.
- `src/main.ts:902-914` — the `isBlocked` list every modal overlay must join.
- `src/commands/index.ts:3317` `installCommands`, `:3224` the `settings.open` spec as a template;
  ids follow the dotted grammar (`src/AGENTS.md:15`).
- `src/panels/AiTranslationPanel.vue:49-51` — the empty surface reserved for Epic 4; home for the
  effective-set line and switcher.
- `src/config/aiconfig.ts:77-81,90` — command-name consts and the never-throwing wrapper shape;
  `src/aiConfigState.ts:80-131,216` — the readonly-refs + accessors + request-sequence state shape.

**Tests that move**

- `src-tauri/tests/prompt_set_contract.rs` (new) · `src-tauri/tests/scope_contract.rs` ·
  `src-tauri/tests/ipc_contract.rs` · `src-tauri/tests/config_invariants.rs` ·
  `src-tauri/tests/pinned_contract.rs` · `src-tauri/tests/store_contract.rs` ·
  `src-tauri/tests/project_contract.rs` · `src-tauri/tests/glossary_contract.rs` ·
  `src-tauri/tests/segment_contract.rs` · `tests/frontend/promptSetState.test.ts` (new) ·
  `tests/frontend/promptLibraryOverlayRender.test.ts` (new)

## Tasks & Acceptance

**Execution:**

- [x] `src-tauri/src/core/scope/kinds.rs` — write Decision 1 (whole set, keyed by name) into the
      `Prompt` doc-comment in the shape its two neighbours use, citing `prompt-library.html:134`
      and Ice's 2026-09-17 signature — rationale: a bare row is what left this question open for
      three epics, and both neighbours settled it here rather than in an AD.
- [x] `src-tauri/src/core/promptset/vars.rs` — the closed three-name variable type plus the
      marker scan returning both warning conditions — rationale: Decision 2 makes this story the
      place those names become code, and one type is what stops Story 4.6 declaring a second
      list that can drift from what 4.4 displays.
- [x] `src-tauri/src/core/store/schema.rs` — `PROMPT_SET_DDL` plus a twin migration step in both
      arrays — rationale: each tier is its own db file, so a two-tier table exists twice.
- [x] `src-tauri/tests/{pinned,store,project,glossary,segment}_contract.rs` — re-measure every
      hard-coded version number and step-count ladder — rationale: measured across five
      table-adding commits, the ripple is 6–10 files and 27–99 numeric lines; Story 4.2, the
      closest analogue, hit 10 files / 31 lines. The "6 files / 16 assertions" figure in spec
      4.3 is understated — do not plan against it.
- [x] `src-tauri/src/core/promptset/` — new domain module: the set type, name validator, two-tier
      load/resolve/write/delete, and its own error enum with `From … for IpcError` — rationale:
      `ScopeError` is Glossary-only by decision, and the closed Rust type is what replaces a
      `CHECK`.
- [x] `src-tauri/src/commands/promptset.rs` + `src-tauri/src/lib.rs` — pure fns plus `mod wire`
      shells, registered in `generate_handler!` — rationale: the command name on the wire is the
      function name, so the shell lives in a nested module.
- [x] `src-tauri/tests/ipc_contract.rs` + `src-tauri/tests/config_invariants.rs` — a registration
      guard for every new wire, and the census row — rationale: the existing guard is a hardcoded
      per-domain list; without a sibling, deleting a `generate_handler!` line leaves both suites
      green while the buttons break in a real build.
- [x] `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` — the new error and section keys, in
      ONE task — rationale: Story 4.3 split this pair across phases and left `ipc_contract` red
      at 27/29 between them.
- [x] `src-tauri/tests/prompt_set_contract.rs` — a case per I/O Matrix row, including the
      shadowed-global row and the store-missing row — rationale: a resolver case that calls the
      pure fn directly guards itself, not the wiring.
- [x] `src/config/promptset.ts` + `src/promptSetState.ts` — never-throwing wrapper and the
      readonly-refs state module — rationale: the UI displays errors through `tError()`, so the
      adapter returning a three-state shape is what keeps `try/catch` out of components.
- [x] `src/PromptLibraryOverlay.vue` + `src/App.vue` + `src/main.ts` + `src/commands/index.ts` +
      `src/settingsState.ts` — the overlay, its mount, its `isBlocked` entry, its command ids, and
      flipping the `'prompt'` section to having a body — rationale: an overlay missing from
      `isBlocked` lets global chords fire through an open modal.
- [x] `src/panels/AiTranslationPanel.vue` — effective-set line and switcher — rationale: the AC
      requires changing sets without opening Settings, and this panel is the surface reserved
      for it.
- [x] `tests/frontend/promptSetState.test.ts` + `tests/frontend/promptLibraryOverlayRender.test.ts`
      — validators and a real mount — rationale: script-setup-local functions and the keyboard
      path are only observable by mounting for real.
- [x] `deferred-work.md` — append at EOF: one item for the language pair and applicability pills
      (Decision 4, owner Story 4.5, to be settled with the export format), and one for
      `{{chapter_context}}` staying out of the ratified vocabulary (owner Story 4.6) — rationale:
      a field the mockup draws and the product does not build reads later as an oversight unless
      the ledger says who declined it and why.

**Acceptance Criteria:**

- Given a Work open with a Work-tier set named the same as a Global set, when the library is
  listed, then the Work set is effective and the Global one is shown as shadowed — and removing
  the `apply_override` call turns an existing case red.
- Given the whole repository after this story, when `generate_handler!` loses any one prompt-set
  line, then a Rust case goes red.
- Given every action on the prompt library screen, when performed with the keyboard only and no
  pointer, then each completes — and no `@click` in the new component is anything but a single
  `dispatch('<id>')`.
- Given a store stamped one version ABOVE this story's new target, when the application opens it,
  then it refuses and says so without writing a byte — tested the way `segment_contract.rs:2093`
  already does it, with a synthetic future `Migration` step rather than an actual old build.
- Given a prompt body carrying a token outside the ratified three, when it is saved, then it is
  stored byte-for-byte unchanged AND a warning names that token — neither outcome alone passes.
- Given the compose screen's variable list and the Rust validator, when a name is added to or
  removed from the ratified type, then both change together, because neither holds its own list.
- Given the gates and suites after this story, then `cargo test --locked` and `npx vitest run`
  are green with the new cases counted, and the eleven `pre-push` gates exit 0.

## Implementation Notes

**Phase 1 — Rust domain and schema (2026-09-17).** Tasks 1, 3, 4, 5 closed; Task 2 left open,
see the defect below. Phase boundaries are in `4-4-phases-2026-09-17.md`. Measured by the
orchestrator on the diff since `baseline_commit`, not taken from the phase agent's report:
**1604 green / 0 red / 20 ignored / 60 binaries** (1595 before; +9 are the new inline unit tests).

- Table is `prompt_set(id, name, body, created_at)` with `UNIQUE INDEX idx_prompt_set_name`,
  one twin `Migration` per store: `GLOBAL_MIGRATIONS` 8→9, `PROJECT_MIGRATIONS` 23→24.
- The ripple was **wider than the five test files the spec named**: `chapter_origin_contract.rs`
  and `segment_role_contract.rs` also hard-code the final `schema_version()`, and
  `project_contract.rs::NON_ENTITY_DETAIL_TABLES` went 11→12 for the new table. The phase agent
  records that a grep pass missed one spot in `glossary_contract.rs` and only the full
  `cargo test --locked` found them all — worth carrying into the next schema story.
- `load_tier` could not be reused as a name: `tests/glossary_boundary.rs:142`
  `GLOSSARY_ONLY_SURFACE` is a bare `code.contains(needle)` substring scan over all of
  `src-tauri/src/**` outside `core/glossary/**`, with no idea which domain owns the name.
  Renamed to `load_prompt_set_tier` rather than weakening the gate. Verified at the gate itself.
- `impl From<PromptSetError> for IpcError` is deliberately absent until Task 8 declares the
  `MessageKey` variants — writing it first would make the tree fail to COMPILE, not fail a test.

🔴 **Two defects the orchestrator found in the diff, both measured, neither in the agent's report:**

1. The `PROMPT_SET_DDL` doc-comment claimed the whitespace guard is "TRÙNG TỪNG BYTE" with
   `GLOSSARY_ENTRY_DDL`. Measured: same 25-code-point SET, but 513 / 465 / 483 bytes across the
   three DDLs that carry it — they differ in continuation-line indentation. The phase agent's own
   handoff notes said "not byte-identical" while the doc-comment it wrote said the opposite.
   Corrected in place; a set-based (not string-based) three-way test is queued for Phase 3,
   because `src-tauri/AGENTS.md:37` states nothing guards this — and it is now a trio, not a pair.
2. `scan_markers` mis-warns when a stray `{{` precedes a real marker: the nearest-`}}` rule makes
   the opener swallow the marker. Measured against the real product code —
   `"ghi chu {{ rieng: {{glossary_terms}}"` reports `glossary_terms_missing = true` and names
   `"{{ rieng: {{glossary_terms}}"` as an unknown token, while the control `"{{glossary_terms}}"`
   reports `false`. That breaks the AC *"a warning names that token"*, and it is exactly the input
   Decision 3 legalises. The eight inline tests miss it because the unclosed-brace case they use
   has no later `}}` and so hits the `break`. Handed to Phase 2 with the repro.

**Phase 2 — Rust IPC and the string catalogue (2026-09-17).** Tasks 2, 6, 7 closed; Task 8's
error half closed, its UI-string half moved to Phase 4 (see below). Measured by the orchestrator:
**cargo 1608 green / 0 red / 20 ignored / 60 binaries** (1604 before; +4 = the new tests) and
**vitest 80 files / 1130 cases**, unchanged as expected — no frontend work yet.

- Five commands in a new `commands/promptset.rs` (`prompt_set_list` / `create` / `rename` /
  `update_body` / `delete`), two-layer shape, registered in `generate_handler!`.
- `tests/ipc_contract.rs::the_prompt_set_wires_are_registered` covers all five. The counter-check
  was run for real — one wire line deleted → red, restored → green — and the orchestrator
  verified on the diff that the restore left `lib.rs` with additions only, no residue.
- Census updated in **both** places it is asserted: the row table 12→13 (with a `why` naming an
  owner) and the independent whole-tree total (58,26)→(63,26). 58+5 = 63, consistent.
- Five `MessageKey` variants plus `impl From<PromptSetError> for IpcError`, and the five matching
  `vi.json` error strings (852 keys total).
- The `scan_markers` defect from Phase 1 is fixed: when the span between `{{` and the nearest
  `}}` itself contains `{{`, the scan restarts at the inner opener. Re-measured on product code
  across five bodies including the original control — the swallowed-marker case now reports
  `glossary_terms_missing = false` correctly, and the three neighbouring behaviours (typo'd token
  named, unclosed brace silent, adjacent markers) are unchanged.

🔴 **A defect Phase 2 found in Phase 1's work, worth keeping as a lesson about phase boundaries:**
`PromptSet` and `ResolvedPromptSet` carried no `id`, while `rename` / `update_body` / `delete` all
take `(tier, id)` — so a list call returned nothing a later edit could act on. The column existed
in the table all along; only the Rust type forgot it. Phase 1 wrote a type that no caller
exercised yet, so nothing forced it to be sufficient. When a phase defines types the next phase
consumes, the type is not done until something calls it.

**Carried forward, with owners:**

- The two `MarkerWarnings` fields have no display strings yet — Phase 4 adds them, because Phase 4
  is what knows the sentences it needs. Unused `vi.json` keys are allowed; a missing `MessageKey`
  string is not, and that half is closed.
- Every prompt-set write, Work-tier included, requires `global.db` to be managed — a consequence
  of Phase 1's `(global, work, tier, …)` signature. Phase 3 should pin it with a contract case so
  it is a decision rather than an accident.
- `ResolvedPromptSet.shadowed_body` carries no `id` for the shadowed Global row. Enough for the
  AC as written (the shadowed row is display-only, per the mockup) but not enough to act on that
  row directly. Ledger item, not silence.

**Phase 3 — Rust contract tests (2026-09-17).** Task 9 closed. 16 cases in
`tests/prompt_set_contract.rs`, one per I/O Matrix row plus the three extras handed to the phase.

⚠️ **The phase agent was stopped and the measuring taken back.** It wrote the test file, then
stopped twice in a row reporting only that it was waiting on a background `cargo test`, with no
numbers either time, and left its handoff section empty. Everything below was measured by the
orchestrator after the stop — none of it is the agent's report.

- **cargo 1624 green / 0 red / 20 ignored / 61 binaries.** Phase 2 left 1608 across 60, so the
  delta is +16 cases and +1 binary — exactly the file's 16 cases. That match is what shows they
  all ran, rather than existing but being skipped or filtered out.
- **The counter-check is a real removal.** `lib.rs` was copied outside the tree first (restoring
  from `HEAD` would have wiped four phases of uncommitted work), one registration line deleted →
  `the_prompt_set_wires_are_registered` red at exit 101 naming that wire → restored from the copy,
  `cmp` byte-identical → green again. AC2 is satisfied by measurement, not by assertion.
- `core/store/mod.rs` gained one `pub use` line so the fixture can name `PROMPT_SET_DDL` — Phase 1
  anticipated this need in its notes. No behaviour change.

🟡 **One I/O Matrix row is half open, deliberately recorded rather than ticked.** *"Switch
effective set from AI panel ... without opening Settings"* is covered on the Rust side by
`switching_between_two_resolvable_sets_needs_no_settings_reopen`, which proves the resolution
changes. It cannot prove the rendered half. Phase 4b closes it and must not read that Rust case
as coverage.

**Phase 4a — webview data layer (2026-09-17).** Task 10 closed. `src/config/promptset.ts` (five
adapters, zero `throw`, one `invoke` per `try/catch`, `Promise<IpcError | null>` for the void ones
exactly as `aiConfigSaveKey` does) and `src/promptSetState.ts` (`readonly` refs, accessor
functions, request-sequence guard, `resetPromptSets()`). The wire convention is right where it is
easiest to get wrong: `invoke` is called with `newName` while Rust takes `new_name`.

⚠️ This agent was also stopped without reporting, and its measuring was taken back. Two defects in
**how it measured** are worth more than the stop, because both are traps this repo has already
named: its command was `cargo test --locked 2>&1 | tail -60` on a tree with **61** binaries, which
would have truncated the `test result:` lines and produced a wrong count even if it had read them;
and its watcher, `until ! kill -0 $(pgrep -f "cargo test --locked" | head -1)`, matched its own
command line, so once the real processes exited it waited on itself forever. It had hung, not
waited. Orphan PIDs were killed here. Measured after the stop: **vitest 81 files / 1156 cases**
(+1 file, +26 cases — exactly the new file's 26 `it(`), cargo unchanged at 1624/61.

**Phase 4b — webview screen (2026-09-17).** Tasks 11, 12 and the render half of 13 closed, plus
Task 8's UI-string half (+42 `vi.json` keys). The 11th overlay, its `isBlocked` entry, two new
command ids, the `'prompt'` Settings section given a body that opens the overlay rather than an
inline compose surface, and the effective-set line with a switcher on `AiTranslationPanel.vue`.
This phase reported properly — foreground verification, real numbers — and caught two of its own
bugs by running rather than inspecting: a Vue SFC parse failure from writing `{{glossary_terms}}`
literally inside a template mustache, and an empty state that removed the only way to create the
first set. The mount test closes the I/O Matrix row Phase 3 could only half-cover, and closes it
structurally: it mounts `AiTranslationPanel.vue` alone and never imports `SettingsOverlay.vue`.

**Phase 4c — the second-list defect (2026-09-17).** Phase 4b disclosed, honestly and in a comment
at the site, that `PromptLibraryOverlay.vue` held a literal array of the three variable names
because no wire exposed `PromptVariable::ALL`, and proposed deferring it to Story 4.5/4.6.

🔴 **That deferral was rejected: it is this story's §Always and this story's AC, not a debt.** The
reasoning offered for deferring was true but scoped to one repair — a sixth command would indeed
ripple into the registration guard and the census. Folding the vocabulary into the **existing**
`PromptSetListWire`, which the screen already fetches, costs neither, so the reason for deferring
did not survive contact with the cheaper fix. The AC also is not cosmetic: Story 4.6 consumes this
enum, so a screen holding its own copy means the day 4.6 adds a name, the compose screen keeps
offering the stale three with **no gate red** — the silent class `AGENTS.md:62` calls this
project's central one.

Closed: the wire field is built from `PromptVariable::ALL` (`commands/promptset.rs:154`), the
literal is deleted, and each description's i18n key is derived from the name by convention rather
than a second lookup table keyed by the same strings. Verified here: `PROMPT_VARIABLES` is gone
and no ratified name is hardcoded anywhere in `src/` outside `vi.json`. The frontend case is the
right shape of counter-check — it mocks the wire with `['mock_alpha','mock_beta']`, deliberately
NOT the real three, and asserts the screen renders those; a case feeding the real three would stay
green with the literal still in place.

## Spec Change Log

## Review Triage Log

Pass 1 — 2026-09-17, three layers over a 261 kB diff (blind-hunter 10 · edge-case-hunter 3 ·
verification-gap 0). 13 findings, 12 distinct after grouping the create-feedback pair both
blind-hunter and edge-case-hunter filed independently. Every claim was re-verified at its cited
location by the orchestrator before a verdict. No `intent_gap` and no `bad_spec`, so no loopback;
`review_loop_iteration` stays 0.

Worth recording: the layers did not overlap the way one would expect. verification-gap returned
**zero** findings and explained four candidate gaps as codebase-wide characteristics rather than
things this diff introduced — and its reasoning held up when checked. Yet blind-hunter found a
genuine verification gap it missed (row 8), and edge-case-hunter found a state race neither other
layer saw (row 11). Three layers earned their cost here.

| # | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|
| 1 | Creating a Global set whose name a Work set already shadows silently does nothing visible | medium | patch | Filed by two layers independently. Verified: `PromptLibraryOverlay.vue:277` looks up `s.tier === tier && s.name === name`, but `resolve_two_tiers` collapses a same-name pair into ONE row reported as `tier: 'work'` (Decision 1), so `created` is `undefined`, nothing is selected, and no message is shown — the write succeeded. The shadowed Global row carries `id: null` and `selectRow:215` returns early on it, and **the implementation has no tier filter** (the only `.filter` in the file is the work/global split), so the set is unreachable until the Work set goes. Reachable in ordinary use: create a Global "Tiên hiệp" while the open Work overrides that name. |
| 2 | Stale action error/warning bleeds onto a different row on selection | low | patch | Verified: `selectRow:214-222` resets `deletePending`/`createOpen` but never `promptSetActionError`/`promptSetActionWarnings`, which are module-level refs in `promptSetState.ts`. Select row B after a failed save on row A and B renders A's banner before B has been submitted at all. |
| 3 | Same staleness when opening the "New set" form | low | patch | Same root cause as row 2 — `onOpenCreate` likewise leaves `promptSetActionError` set, so a brand-new Create form opens carrying another row's failure. Grouped with row 2; one fix closes both. |
| 4 | An armed delete stays armed across an unrelated save | low | patch | Verified: `deletePending` is cleared at `:220` (select), `:227` (clear), `:304` (after the real delete) and `:336` (Escape) — not in `onSubmitRename`/`onSubmitBody`. Arm delete, change your mind, save the body instead, and the next click deletes for real. Mitigating and worth stating: the armed state is visible — the button relabels to "Xác nhận xoá vĩnh viễn", takes a danger class, and a hint paragraph shows — so it is not a silent trap. Kept because the fix is a direct correction, two assignments. |
| 5 | The shadowed-Global badge borrows a Glossary-owned i18n key | low | patch | Verified: `PromptLibraryOverlay.vue:408` calls `t('glossary.manage.shadowed_badge')` while this diff added `prompt.library.shadows_badge` (`vi.json:74`) for the opposite direction only; no `prompt.library.shadowed_badge` exists. Named harm: rewording the Glossary string for Glossary's own context silently rewords this screen, with no commit here responsible. |
| 6 | English dev jargon in a string the user reads | low | patch | Verified: `vi.json:91` — "không nằm trong ba biến đã **ratify**". The only occurrence of "ratify" in the entire catalogue; every other string in the diff is fully Vietnamese. It renders on the warning path a user meets by typing an unknown `{{...}}`. |
| 7 | Three empty-state sentences stack when both tiers are empty | low | patch | Verified: `:381`, `:383` and `:394` are independent `v-if`s, and `promptSets.length === 0` implies both group counts are 0, so all three render. The I/O Matrix asks for "an empty state saying nothing is configured", singular. |
| 8 | `WorkTierUnavailable` is documented on four write paths and asserted by no test | medium | patch | Verified, and it is a gap the verification-gap layer missed. The only contract case naming it (`prompt_set_contract.rs:395-409`) asserts the message key must be `store.open_failed` and **not** `work_tier_unavailable`, for a different precondition order. Sibling domains do test theirs (`glossary_commands_contract.rs:325`, `cleanup_contract.rs:858`). Defense-in-depth that nothing exercises is defense nobody has seen work. |
| 9 | The `prompt.library.var_<name>_desc` convention has no guard | medium | patch | Verified: `varDescKey:126-128` builds the key by concatenation, so a static scanner cannot trace it; all three keys exist today but no test asserts any of them resolves to real text. Named harm with a date on it: Story 4.6 adds a variant to `PromptVariable::ALL`, the wire carries it automatically (that is the point of the Phase 4c fix), and the compose screen renders a blank or a raw key with **no gate red** — the same second-source class this story already had to close once. |
| 10 | `src-tauri/AGENTS.md:37` now says "no gate guards this pair" and that is false | low | defer | Verified true: this diff adds a third DDL carrying the 25-code-point guard and a test that checks all three as a set, and the test's own doc-comment says so. Deferred **only** because step-04 routes any fix that edits agent-context files to defer, not because the claim is weak. |
| 11 | `busy` is cleared before the trailing reload settles | low | patch | Verified: `promptSetState.ts:156` sets `busy.value = false`, then `:162` awaits `loadPromptSets()`, so buttons re-enable while the list is still in flight. The worst outcome is bounded by the request-sequence counter, which already prevents a stale load from overwriting a newer one — that is why this is `low` and not higher. Fix is moving one assignment. |
| 12 | Two elements now share `[data-prompt-library-open]`, and the opener lookup is a bare `querySelector` | low | rejected | The ambiguity is real — `SettingsOverlay.vue:417` and `AiTranslationPanel.vue:94` both carry it, and `document.querySelector` takes the first. But the bad outcome does not follow: `focusReturnTargetOnOpen` returns `opener` only when `activeEl === opener`, otherwise `activeElement`, so the primary path returns focus to the button actually used. The bare lookup at `:78` is a fallback reached only when that target is already disconnected, and it then focuses a different valid button rather than nothing. Rejected under the `low` rule: unlikely in everyday use, and scoping the lookup means adding a parameter or branch — more than a direct correction. What would change this: a case where both openers are connected AND the used one is disconnected mid-session. |
| 13 | (verification-gap) No findings | — | — | The layer traced each changed behaviour to its tests and filed nothing. It named four candidate gaps and argued each is a codebase-wide characteristic rather than something this diff introduced; the one most worth checking — that no test exercises real Tauri serialization of the new wire — was verified here and the mapping is correct: Rust declares snake_case, the TS wire type mirrors it, and `config/promptset.ts:111` translates at the adapter boundary (`workTierAvailable: wire.work_tier_available`). |

## Design Notes

**Why this is not just "another `ai_config`".** Both existing two-tier consumers key by a flat
scalar the system owns — `source_term`, or one of five field names from a closed enum. Here the
key is a string the USER types and can rename, and the value is a set body rather than one
scalar. Three consequences follow. The name needs a real empty-guard in DDL, which
`ai_config` never needed. Renaming is a distinct operation from editing, because it moves the
override key and can collide. And uniqueness is per tier, not global — the same name existing
at both tiers is the feature, not a conflict.

**Why the migration is the irreversible half.** `global.db` is under AD-30, forward-only: once
this story's step has run on a machine, a build that predates it REFUSES TO OPEN that file —
losing the shared Glossary and pinned entries, not just prompt sets (`src-tauri/AGENTS.md:51`).
That is a one-way door for anyone who downgrades, and it is the reason the twin step lands in
one task rather than being split across phases.

## Verification

**Commands:**

- `npm run build` before any `cargo test` — without `dist/`, `cargo test` breaks at compile time.
- `cd src-tauri && cargo test --locked` — expected: 0 red; record before and after case counts
  and name the new cases. Baselines were measured at `8a1d8f0`, one commit below this spec's
  `baseline_commit` — `8c816b9` touches only `epic-4-context.md`, so the counts carry unchanged:
  **1595 green / 0 red / 60 binaries**.
- `npx vitest run` — expected: 0 red. Baseline measured 2026-09-17: **80 files / 1130 cases**.
- `npm run check:commands` · `check:i18n` · `check:panel-refs` · `check:layout` ·
  `check:debt-owner` — expected: exit 0, with no name added to an allow-list without a written
  reason.
- Counter-check, not optional: delete one prompt-set line from `generate_handler!` and run both
  suites; at least one case must go red. Then restore and re-run.
