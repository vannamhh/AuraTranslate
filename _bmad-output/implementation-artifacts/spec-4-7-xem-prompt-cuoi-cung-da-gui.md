---
title: 'Story 4.7 — Inspect the exact prompt that was sent'
type: 'feature'
created: '2026-09-18'
status: 'done'
route: 'dispatch'
review_loop_iteration: 2
baseline_commit: '6de227da1a097456b8e6f8efc2ca70ac5bc2f6cf'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-4-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Story 4.6 assembles the prompt and a ledger of everything it injected, suppressed
and warned about, and **nothing can read either**. Measured on this tree: `0` code positions
outside `core/ai/**` name the module, because `ai_boundary.rs` bans the bare tokens everywhere
else — so there is no legal product path from the assembler to a screen today. FR71 is the only
diagnostic the translator has when the AI ignores a confirmed term, and AD-14 exists precisely so
that diagnostic can be exact instead of approximate.

**Approach:** record **one** assembled prompt per session — the exact string, its ledger, and the
identity of what it was built from — and open it through a registered, rebindable command. The
screen reads that record and never re-assembles: a recompute is a rebuild, and AC4 bans rebuilds.
Story 4.8's send path writes the same record and sends exactly the recorded string, which is what
makes "matches 100%" structural instead of a promise.

## Boundaries & Constraints

**Always:**
- The screen reads the **record**. Never re-assemble to draw it: the Glossary, the prompt set and
  the segment can all change between assembling and looking, so a recompute would faithfully
  display a prompt that was never the one in question.
- Every absent slot stays three-valued and is never collapsed: Glossary `NotAsked` (the body
  carries no `{{glossary_terms}}`, so no query ran) versus `Asked` with zero terms; TM
  `NotBuiltYet` (Epic 7) versus searched-and-empty; and **no record yet this session** versus a
  record whose prompt is empty.
- The record carries the identity of what produced it — which segment, which prompt set, which
  tier — so a record from an earlier segment cannot be read as describing the segment now focused.
- Dynamically injected text is visually separable from the user-authored body, and every injected
  term shows its **tier**. The ledger already carries `tier`, `start` and `end` for both the
  injected and the suppressed lists; the screen must not drop them again.
- "Considered but not injected" is a first-class part of the screen with a reason per row — it is
  the half that answers *"why did the AI not use my term?"*, and it is the half a summary count
  alone cannot answer.
- One summary line states how many Glossary terms were injected, and it reads from the same
  record — not from a second count computed on the screen.
- Opening is a registered command with a dotted id, rebindable through the existing shortcut
  layer. A default chord is optional; `prompt.library.open` is the precedent that registers with
  `keys: undefined` and is still rebindable.
- Two beats, two commands (Decision 2): the producer assembles and records; the inspector only reads.
  Opening the screen must never assemble.
- No display text in Rust. Errors cross IPC as `IpcError { code, message_key, params, retryable }`
  and resolve from `vi.json`.

**Never:**
- No new table and no migration — the record is per-session managed state, the shape
  `PendingPromptImportState` already established.
- No network call, no provider, no streaming, no cancellation: the first real send is Story 4.8.
- Nothing from the mockup that another story owns: no per-block token count, no budget bar, no
  cost estimate (Story 4.11); no TM block **content** (Epic 7 — only its `NotBuiltYet` state is
  shown); no "re-send with this prompt" and no model name or sent-at timestamp (Story 4.8, which
  is the first code that can know either).
- Do not exploit the boundary gate's blind spot. `FORBIDDEN_BARE_TOKENS` lists `crate::core::ai`
  and `super::ai` only, so a bare `core::ai::…` written in `lib.rs` compiles and passes — using
  that is an AD-13 nobody guards, not a design.
- No fix-it-now action on a "considered but not injected" row (Decision 3) — read-only here.
- Do not change any signature or frozen decision of `core/ai/rag.rs` (spec 4.6).
- Do not widen the exemption of Decision 1 beyond those two paths, and do not remove its two controls.
- No `v-html`, no markup from data (AD-16): the prompt renders from a structured model.
- Do not write display strings into Rust, and do not add a second scanner for markers —
  `scan_markers` already reports them and the ledger already carries the result.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Happy path | A segment containing two confirmed terms; selected set's body has `{{glossary_terms}}` and `{{source_segment}}` | Record holds the assembled string; screen shows the user-authored body separated from the injected block, two term rows each with translation and tier, summary says two injected | N/A |
| Marker absent | Body has no `{{glossary_terms}}` | Screen states no Glossary query ran; summary does **not** say "0 injected" | N/A |
| Asked, nothing matched | Body has the marker; no confirmed term occurs in the sentence | Screen states the query ran and matched nothing; summary says 0 injected | N/A |
| Pending overlap | A pending term covers a confirmed one | Confirmed term appears under "considered but not injected" with the pending-overlap reason and its tier; it is absent from the injected list | N/A |
| TM never built | Any assemble in this epic | Screen states TM is not built yet — never "0 similar sentences" | N/A |
| Sentence marker missing | Body has no `{{source_segment}}` | Assembling still succeeded; screen surfaces the ledger's missing-sentence flag as a warning | N/A |
| Unknown marker | Body types `{{chapter_context}}` | Marker stands verbatim in the prompt and is listed as unknown on the screen | N/A |
| No set selected | No effective prompt set | Command returns `IpcError` with a `message_key`; screen shows the resolved sentence | Error shape, never an empty prompt box |
| No Work, or an id the chapter does not hold | Assemble requested with no open Work, or with a segment id absent from the open chapter | `IpcError` naming which precondition is missing — the two causes are distinct keys | Error shape, never a blank screen and never one key for both |
| Nothing recorded yet | Inspector opened before any assemble this session | Screen states no prompt has been assembled yet this session | N/A — this is a state, not an error |
| Stale record | Record was built from segment A; segment B is now focused | Screen shows the record **and** names the segment it belongs to | N/A |
| Empty body | Selected set's body is the empty string | Record holds an empty user-authored part with the injected parts still shown; screen does not read as "nothing recorded" | N/A |

## Decisions

**1. One NAMED seam through the AD-13 gate, plus the positive control that gives FR77 its teeth.**
Ice settled 2026-09-18, on the two measurements that make this a question and not a blocked road:
`epics.md:3379` (Story 4.1 AC1) scopes the ban to *"không module nào khác **trong `core/`** import
nó"*, and AD-13's own rule offers *"hoặc `ai/` là crate riêng để trình biên dịch cưỡng chế"* as an
**equivalent** mechanism — under which the binary crate depending on `ai/` is exactly what the
compiler permits. The two mechanisms AD-13 calls equivalent are therefore not equivalent as
implemented, and the gate is stricter than the invariant it enforces. So: add a **path-scoped**
exemption to `ai_boundary.rs` covering exactly two seams — `src/commands/aiprompt.rs` and the
`generate_handler!` / `app.manage` lines in `src/lib.rs` — in the shape
`naming_boundary.rs::STORE_EXEMPT` already uses: named, carrying its reason in place, able to die.
The exemption is not the deliverable; the **control** is: deleting `core/ai/` together with those two
seams must leave every other capability compiling, which is FR77's real content and the only thing
that keeps the seam from becoming a silent hole. The gate keeps its teeth for all 68+ other files,
and `ALLOWED_GLOSSARY_NAMES_UNDER_AI` is untouched. Rejected: handing this to Winston as a new AD
(4.8–4.12 would all block on the same seam), and making `ai/` a separate crate (measured: `rag.rs`
depends on `ScopeResolver`, `Store`, `GlossaryError` and four more `core::glossary` names, all inside
`src-tauri`, so `glossary`, `scope` and `store` would have to become crates too or the dependency
inverts into a cycle — a workspace restructuring far larger than this story).

**2. The epic's order stands, and the story runs on TWO separate beats.** Producing the record and
reading it are different commands. An explicit *assemble this sentence* action in the AI panel writes
the record — no network, no send; the inspector command only **reads** it. Story 4.8's translate
action then takes that producer's place and sends the recorded string. This is what makes AC4 true by
construction: if opening the inspector assembled, every viewing would be a fresh computation and the
screen's claim *"đúng chuỗi đã gửi, không phải bản dựng lại"* (`prompt-inspector.html:196`) would be
false however pure the function is. Consequences accepted: one user-visible button in the AI panel
that Story 4.8 supersedes, and AC1's wording *"đã gửi"* stays unfulfilled until 4.8 — recorded as an
owned debt, not by editing `epics.md` (root `AGENTS.md`: a capability not yet built is not a spec
mismatch). Rejected: swapping 4.8 ahead of 4.7 (the first real AI call would ship with no way to
inspect what it sent), and merging the two stories.

**3. The "considered but not injected" rows are READ-ONLY in this story.** The mockup gives each row a
fix-it-now action and its own legend argues the screen is only half a diagnostic without one
(`prompt-inspector.html:206`); Ice chose the smaller seam. The translator reads the reason here and
fixes it on the Glossary screens that already exist. Consequence accepted: the mockup's strongest
claim goes unbuilt and is recorded as an owned debt — and the reason it is not free is on the same
screen, because every such action would invalidate the record being displayed, which would need its
own answer here.

**4. The full spec stands at 7.677 tokens** (tiktoken `o200k_base`, measured on the approved text; the
figure shown to Ice at the gate was 7.340, before these Decisions and the last Code Map additions were
written in). Above the 1.600 mark either way. Measured on the same scale — current body with the two
append-only logs stripped — the three neighbouring stories are 4.4 = 8.229 · 4.5 = 8.936 ·
4.6 = 6.418, so this sits inside their band. It is ONE user-facing
goal (one screen) across four layers; carving it would leave either a screen with nothing feeding it
or a producer nobody can see. The context-rot risk is real and is paid down the way root `AGENTS.md`
requires: four phases, each handed to a fresh agent through this file.

</frozen-after-approval>

## Code Map

**What Story 4.6 left to read — do not change any signature here**

- `src-tauri/src/core/ai/rag.rs:386-391` `assemble_prompt(body, sentence, glossary, tm) ->
  (String, InjectionLedger)` — pure. `:153-160` `gather_glossary_context(body, resolver, global,
  work, source_lang, sentence) -> Result<GlossaryInjectionStatus, GlossaryError>` — the impure
  half; together they are AD-14's signature.
- `src-tauri/src/core/ai/rag.rs:125-136` `InjectionLedger { glossary, tm, unknown_markers,
  source_segment_missing }`. `:99-110` `GlossaryInjectionStatus::{NotAsked, Asked { injected,
  suppressed_by_pending_overlap }}`. `:114-121` `TmInjectionStatus::{NotBuiltYet, Searched(_)}`.
  `:61-73` `InjectedGlossaryTerm { source_term, translation, start, end, tier }`; `:79-91`
  `SuppressedGlossaryTerm` with the same five fields. **None of these derive `Serialize`** — the
  wire types are this story's to write, and the mapping is where a dropped field would hide.
- `src-tauri/src/core/ai/mod.rs:24` `pub mod rag;` — the only child. `core/mod.rs:6` has a bare
  `pub mod ai;`; `ai_boundary.rs:410` fails on a `pub use ai::…` re-export, so do not add one.
- `src-tauri/src/core/tm/mod.rs` — where `SimilarSegment` lives; Epic 7 fills the TM argument
  without reshaping either function.

**The gate that decides where the wire may live (Decision 1)**

- `src-tauri/tests/ai_boundary.rs:99` `FORBIDDEN_BARE_TOKENS = ["crate::core::ai", "super::ai"]`;
  `:266-302` the real gate, scanning every file not matched by `is_inside_ai_module` (`:211-213`);
  `:567-573` `ALLOWED_GLOSSARY_NAMES_UNDER_AI` (five names) is the shape a named exemption copies;
  `:804-825` pins the Glossary door to exactly one call under `core/ai/**`; `:316` is the
  seeded-violation control style to copy. Floors `:65` `AI_FLOOR = 1`, `:89` `SRC_RS_FLOOR = 68`
  are `>=` minimums.

**Where the prompt body and its tier come from**

- `src-tauri/src/core/promptset/store.rs:115` `resolve_two_tiers(resolver, global, work) ->
  BTreeMap<String, ResolvedPromptSet>`; `:83-104` `ResolvedPromptSet { id, name, body, tier,
  shadowed_body, shadowed_id }` — `tier` is the label the screen shows per block.
- `src/promptSetState.ts` — the effective set is **frontend-local by decision** (`setSelectedPromptSetName`
  performs 0 `invoke`), so the command receives the set **name** and resolves the body and tier in
  Rust; the frontend never sends a body.
- `src-tauri/src/core/promptset/vars.rs:104` `scan_markers`; `:79-89` `MarkerWarnings`; `:59`
  `marker()`. Reuse; do not write a second scanner.

**Command layer, state, and the gates that count it**

- `src-tauri/src/commands/promptset.rs:160-163` and `src-tauri/src/commands/aiconfig.rs:166-169` —
  the two-layer shape to copy: a pure `fn(global: Option<&Store>, open: Option<&OpenWork>, …) ->
  Result<…Wire, IpcError>` plus a nested `mod wire` `#[tauri::command]` taking `State` through
  `try_state`, never `state()`. `promptset.rs:561-571` is the whole 11-line shell, including
  `.lock().unwrap_or_else(std::sync::PoisonError::into_inner)` — the only permitted lock form.
- `src-tauri/src/commands/promptset.rs:54-67` `PromptSetTierWire` + `impl From<PromptSetTier>` and
  `:88-90` `PromptSetWire` — the mirror-type convention: a plain `serde::Serialize` twin per core
  type, converted by an explicit `match`, and **no** `#[serde(rename_all)]`, so wire fields stay
  `snake_case` while `invoke` arguments go out in camelCase (`src/AGENTS.md`).
- `src-tauri/src/lib.rs:712` `invoke_handler(tauri::generate_handler![…])`, Epic 4 entries at
  `:908-935`, all written `crate::commands::…::wire::…`. Managed state is registered at `:1174-1206`;
  `:1183` `PendingPromptImportState::new(None)` is the closest precedent — and the shape is just
  `pub type X = std::sync::Mutex<Option<T>>` (`commands/project/mod.rs:2255`), not a custom wrapper.

**Reading the sentence to assemble from — the missing accessor**

- `src-tauri/src/commands/project/mod.rs:52-` `OpenWork { dir, store, scope, meta, chapter_id, … }`;
  `:5159` `OpenWorkState = Mutex<Option<OpenWork>>`. The open Work knows its `chapter_id`, not a
  focused segment.
- ⚠️ **There is no `pub fn` that reads one segment by id.** The only public reader is
  `src-tauri/src/commands/segment.rs:1050` `read_open_chapter_segments(open) -> ChapterSegments`,
  whose rows (`:282-292` `ChapterSegment { id, ord, source_text, target_text, … }`) carry the text;
  the per-id `SELECT … WHERE id = ?1` at `:3109-3113` is file-private and lives inside a
  transaction. So the command takes a segment id, reads the chapter, and selects the row — with an
  error when the id is absent from it. This is a user-triggered action, not the per-sentence hot
  path `deferred-work.md:5882` budgets, so the whole-chapter read is acceptable; say so rather than
  adding a second public accessor.
- `src-tauri/src/core/tm/mod.rs:22-28` `SimilarSegment { source_text, target_text }` — the only
  content of that module. The TM argument stays `None` here.
- `src-tauri/tests/config_invariants.rs:1472-1544` `COMMAND_FILE_CENSUS` needs a row for any new
  file carrying `#[tauri::command]`; `:1666-1677` fails on an unclassified file; `:1678-1689` is an
  `assert_eq!` on `(tree_plain, tree_async) == (65, 28)` — both numbers must be re-counted, not
  guessed.
- `src-tauri/src/core/i18n/mod.rs:100` `message_keys! { … }` — the closed catalogue. A variant
  declares its required params inline (`:109` `IoReadFailed => "err.io.read_failed" ["path"]`).
  Never a hand-written parallel list: the sync test
  `src-tauri/tests/ipc_contract.rs:232 every_message_key_exists_in_vi_json` runs over `ALL`, so a
  variant missing from `ALL` is falsely green.

**Frontend: the command, the screen, the state**

- `src/commands/registry.ts:45-89` `CommandSpec { id, labelKey, run, keys?, repeatable? }`; there is
  **no mode field** — availability is not declarable. `src/commands/index.ts:1022` `registerAll` is
  the single manifest; `:3286-3306` the `prompt.library.open`/`close` pair to copy verbatim in shape
  (`keys: undefined`, `deps.…` port guarded by `portMissing`).
- `src/config/shortcutsState.ts` persists user overrides through `putConfig(SCOPE_SHORTCUT, …)`;
  effective bindings come from `effectiveBindings()` in `src/commands/index.ts`, never from
  `registry.unbound()`.
- `src/PromptLibraryOverlay.vue` + `src/promptLibraryState.ts:25,32` + `src/App.vue:414` — the
  overlay shape: a root-level component gated by `v-if`, opened and closed by two commands. There is
  no router in this project.
- `src/panels/AiTranslationPanel.vue:74-99` — where the summary line and the open button belong;
  `EXPERIENCE.md:388` puts the count line in this panel and "Xem prompt" as the screen it opens.
  `:94` shows the required `@click` shape: exactly one `dispatch('<id>')`.
- `src/config/promptset.ts:97-101` command-name constants and the `promptSetList` adapter — the shape:
  `invoke<Wire>(CMD)` in one `try`, `isIpcError(err)` first, then `hasIpcBridge()` to separate a real
  IPC failure (`console.error`, `UNKNOWN_IPC_ERROR`) from running outside Tauri (`console.info`,
  `error: null`), always returning `{ value | null, error }` and never throwing.
- `scripts/check-commands.mjs` Kiểm A (every `@click` is exactly one `dispatch`), `COMMAND_FLOOR = 52`
  at `:319` — a known-loose `>=` floor, 35 commands behind reality; do not read its green as coverage.
  `scripts/check-i18n.mjs:1063` `KEY_RE`, Kiểm A (no Vietnamese diacritics in source positions),
  Kiểm A2 (every template text node goes through `t()`), Kiểm B (`vi.json` flat, every key matches).
- `scripts/check-panel-refs.mjs:3-4` — every module-level memory cell in `src/**/*.ts` must pass
  through a reset function or carry a named, reasoned exemption; `.vue` files are out of its scope.
  `scripts/check-tokens.mjs` Kiểm B bans literal colour and type values in `<style>` unless a named
  `aura-allow-literal` exemption gives a reason.

**The segmentation the prompt display needs (loop 1 — finding B1)**

- `src-tauri/src/core/ai/rag.rs:125-136` `InjectionLedger` — the fix ADDS a field here carrying the
  assembled prompt as an ordered list of pieces, each tagged authored / glossary / tm. This is
  additive: `assemble_prompt`'s signature `(body, sentence, glossary, tm) -> (String,
  InjectionLedger)` does **not** change, and no Decision of spec 4.6 changes — purity, the
  three-valued states, the removal-anchored blank-line collapse and the arbitration order are all
  untouched. It is the only shape that keeps ONE assembly path: a second function that re-expands
  the body would be exactly the drift spec 4.6 was written to prevent, and re-deriving the spans in
  `commands/` or on the screen would be a second scanner, which §Never forbids.
- 🔴 The guard that keeps AC4 true while the display becomes separable: the concatenation of the
  pieces must equal the recorded `prompt` **byte for byte**, asserted in `ai_prompt_contract.rs`.
  Without it the screen gains a second source of truth for the same string — the defect this whole
  story exists to make impossible.
- `src/AiPromptInspectorOverlay.vue:184` — today one `<pre>{{ prompt }}</pre>`. It renders from the
  piece list instead, authored pieces plain and injected pieces marked. `:449` carries a CSS comment
  quoting the §Always clause while styling `.aip-term-list-injected` — a different surface; that
  comment moves to the rule that actually satisfies the clause, or goes.

**Correction to loop 1's piece kinds (loop 2 — finding P7)**

- The loop-1 §Code Map named the kinds *"authored / glossary / tm"* and the implementation followed
  it exactly, so the sentence substituted into `{{source_segment}}` is tagged `Authored` and renders
  identically to the text the translator typed — the most dynamic part of the prompt, indistinguishable
  from the least. **That gap is this spec's, not the implementation's.** The source segment gets its
  own kind, its own CSS rule and its own case. The mockup draws it as its own labelled block
  (*"Câu cần dịch"*, `prompt-inspector.html:136-139`), which is the shape to follow.
- `.aip-piece-tm` has no rule either (finding P9). Give every kind a visible rule now, while the kinds
  are being introduced — Epic 7 is the first caller that can produce a `tm` piece, and it must not be
  the story that discovers the rule is missing.

**Tests that move**

- `src-tauri/tests/ai_boundary.rs` · `src-tauri/tests/config_invariants.rs` · a new
  `src-tauri/tests/ai_prompt_contract.rs` (wire shape, the matrix) · `tests/frontend/` gains one
  `*.test.ts` for the screen (vitest, `happy-dom`, `globals: false`, files live under
  `tests/frontend/**`; `tests/frontend/glossaryQueue.test.ts` is the keyboard-interaction example).

**Debt this story must hand on, not absorb**

- `deferred-work.md:13095-13152` — the four items Story 4.6 left; three are owned by Story 4.8 and
  one by Ice. This story adds none of them to its own scope.
- `deferred-work.md:5843` — the tier-label item is closed **at the ledger layer** (fixed 2026-09-18);
  it stays open for `entries_eligible_for_injection`, which has zero product callers. Do not
  "finish" it here.

## Tasks & Acceptance

Split into four phases along the shape above, each handed to a fresh agent through this file
(root `AGENTS.md`: one agent must not implement a whole story).

**Execution:**

*Phase 1 — the seam (Decision 1)*
- [x] `src-tauri/tests/ai_boundary.rs` — add the path-scoped exemption list for exactly
      `src/commands/aiprompt.rs` and `src/lib.rs`, each entry carrying its reason in place
      (`naming_boundary.rs::STORE_EXEMPT` shape), plus the negative control that a NEIGHBOURING path
      does not match it (the `core/aim`-style false-exemption bug this file already caught once), plus
      the positive control that deleting `core/ai/` and those two seams leaves the rest compiling —
      rationale: the gate refuses the whole story until this is settled, and an exemption without a
      control is an AD-13 nobody guards.

*Phase 2 — Rust*
- [x] `src-tauri/src/commands/aiprompt.rs` (new) — `pub type LastAssembledPromptState =
      std::sync::Mutex<Option<…>>`; the pure function that resolves the effective set two-tier, reads
      the chapter and selects the given segment id, calls `gather_glossary_context` then
      `assemble_prompt`, stores the record and returns it; a second pure function that reads the
      record back; the `serde::Serialize` mirror types carrying every ledger field including `tier`,
      `start`, `end`; the nested `mod wire` shells using `try_state` — rationale: one place assembles
      and records, so the screen and Story 4.8's send path cannot diverge.
- [x] `src-tauri/src/lib.rs` — `app.manage(LastAssembledPromptState::new(None))` beside the other
      pending states and register both commands in `generate_handler!` — rationale: the per-session
      record needs exactly one owner.
- [x] `src-tauri/src/core/i18n/mod.rs` — add the `message_keys!` variants covering no-set-selected,
      no-Work-open and segment-not-in-chapter — rationale: the catalogue is closed; a hand-written
      list gives a falsely green sync test. **Corrected 2026-09-18 (loop 1, finding E9): this
      bullet's original wording implied three NEW variants; Phase 2 added exactly TWO
      (`AiPromptNoSetSelected`, `AiPromptSegmentNotInChapter`) and correctly reused the
      already-existing `MessageKey::WorkNoneOpen` for the third slot — the project's own
      standing rule is one key per fact, and "no Work open" already had one before this story.
      The record is corrected here, not the code (the reuse was right); see Phase 2's own
      Implementation Notes "Judgment call — only TWO new variants, not three" for the full
      reasoning.**
- [x] `src-tauri/tests/config_invariants.rs` — add the `COMMAND_FILE_CENSUS` row for
      `src/commands/aiprompt.rs` and re-count `(tree_plain, tree_async)` — rationale: it is an
      `assert_eq!`, not a floor.

*Phase 3 — Webview*
- [x] `src/config/aiprompt.ts` (new) — the IPC adapter for both commands: one `invoke`, runtime type
      check, `{ value | null, error }`, `hasIpcBridge()` branch — rationale: adapters never throw,
      and data off the wire is a claim, not a compiler guarantee.
- [x] `src/aiPromptInspectorState.ts` (new) + `src/AiPromptInspectorOverlay.vue` (new) +
      `src/App.vue` — the overlay and its state: module cells reachable from one
      `resetAiPromptInspector()`, overlay gated by `v-if`, mounted at root — rationale: copies the
      `PromptLibraryOverlay` shape and satisfies `check:panel-refs`.
- [x] `src/commands/index.ts` + `src/i18n/vi.json` — register `ai.prompt_inspector.open` and
      `ai.prompt_inspector.close` through `deps` ports guarded by `portMissing`, add their label keys
      and every screen key — rationale: AC6 is satisfied by registration, which is also what makes
      the action rebindable. Also registers a THIRD command, `ai.prompt.assemble` — see the Phase 3
      Implementation Notes entry for why the task list's two-command wording undercounted Decision 2's
      requirement.
- [x] `src/panels/AiTranslationPanel.vue` — the injected-count summary line plus an open button whose
      `@click` is exactly one `dispatch('ai.prompt_inspector.open')` — rationale: `EXPERIENCE.md:388`
      puts the count in this panel, and Kiểm A only sees `@click`. Also adds the `ai.prompt.assemble`
      trigger button, distinct from the open button, per Decision 2's "one user-visible button" clause.

*Phase 4 — Tests that move*
- [x] `src-tauri/tests/ai_prompt_contract.rs` (new) — one named case per matrix row, the three-valued
      rows first — rationale: `NotAsked` versus asked-and-empty is this project's central failure
      class, and no gate guards it.
- [x] `tests/frontend/aiPromptInspector.test.ts` (new) — the screen's three-valued states, the tier
      label, the suppressed rows, and open/close by dispatch — rationale: 412 lines of ribbon tests
      once never mounted the component.
- [x] `deferred-work.md` — append the handoffs this story creates, each with an owner written as the
      literal `Chủ:` that `scripts/check-debt-owner.mjs` reads (a phrase like `Chủ phần còn lại:` is
      invisible to it): ① AC1's *"prompt cuối cùng đã **gửi**"* is unfulfilled — the record's sent-state
      is always not-sent in this story (**Chủ: Story 4.8**, the first code that can send); ② the
      mockup's fix-it-now action per suppressed row is unbuilt, Ice declined it for 4.7 with the reason
      in Decision 3 (**Chủ: Story 4.11**, the next story that touches this screen); ③ anything the
      seam's positive control could not actually exercise — rationale: nothing is marked passed by
      inference, and an owner the gate cannot read is no owner.

*Phase 5 — loop 1: the separability defect and the findings carried with it*
- [x] `src-tauri/src/core/ai/rag.rs` + `src-tauri/src/commands/aiprompt.rs` + `src/config/aiprompt.ts`
      + `src/AiPromptInspectorOverlay.vue` — carry the prompt's pieces through ledger → wire → screen
      and render from them, with the byte-for-byte concatenation assert — rationale: finding B1, the
      §Always/AC2 clause the current code quotes but does not satisfy.
- [x] `src-tauri/src/commands/project/mod.rs` — clear the record in `replace_open_work` beside the two
      sibling clears, plus a contract case that goes through a Work swap — rationale: finding V1; ids
      restart at 1 per `.atproj`, so the staleness check reports *not stale* for another Work's row.
- [x] `src-tauri/tests/ipc_contract.rs` — add `the_ai_prompt_wires_are_registered`, including the
      parameter-name half — rationale: finding V2; today deleting both registration lines leaves every
      suite green.
- [x] `.github/workflows/ci.yml` (or a `check:*` script, whichever matches `check:scope`'s wiring) —
      make the `#[ignore]`d FR77 control run somewhere automatic — rationale: finding V3; it runs in
      no path today, so the seam's only control is a manual habit.
- [x] `src-tauri/tests/ai_boundary.rs` — replace the whole-file skip with a named-allowlist scan of
      `commands/aiprompt.rs` (the `ALLOWED_GLOSSARY_NAMES_UNDER_AI` shape); delete the unconditional
      `if aiprompt_existed` asymmetry (finding B8); assert the brace counter returns to 0 (finding E7)
      — rationale: findings V4, B8, E7; Decision 1's control ② is prose until the allowlist exists.
- [x] `src/aiPromptInspectorState.ts` + `src/config/aiprompt.ts` — stop collapsing read/assemble
      failures into `null`: keep the previous record and surface the error, including the
      no-bridge branch and the sequence-overtake path — rationale: findings B4/E2/E3/E4/E10, the same
      collapse §Always forbids one clause above.
- [x] `src/panels/AiTranslationPanel.vue` — mark or suppress the summary line when the record is
      stale, and carry the segment identity on the assemble error — rationale: findings B6/E5/E6.
      **Corrected 2026-09-18 (loop 2, finding P12): this bullet's wording overclaimed the shape of
      the fix.** Phase 5 did NOT attach a `segment_id` field to `aiPromptAssembleError`/`IpcError`
      — `assembleError` stays a bare `IpcError | null` (`aiPromptInspectorState.ts`). What Phase 5
      actually built is `clearAiPromptAssembleError()` plus a `watch(editorCaretSegmentId, …)` in
      `AiTranslationPanel.vue` that CLEARS the error the moment the focused segment changes — the
      same user-visible outcome ("a stale error for segment A does not linger after focus moves to
      segment B") through a different mechanism (clear-on-change, not carry-an-id). The record is
      corrected here, not the code — the behavior is right; the phrase "carry the segment identity"
      described a shape the code never took.
- [x] `tests/frontend/aiPromptInspector.test.ts` — cases that override `unknown_markers` and
      `source_segment_missing` (the two matrix rows with no screen coverage, finding B2); assert the
      mounted panel's summary through all three states (V5); make the mis-named case assert what its
      name claims (B3); add the AC3 clause about `registry.unbound()` (E11); stub all six
      `promptSetState` imports (E8); fix the stale "five cells / four values" wording (B14) —
      rationale: a screen clause guarded only on the Rust side is not guarded.
- [x] `src/AiPromptInspectorOverlay.vue` — `aria-labelledby` pointing at the existing `.aip-title` id
      — rationale: finding B15; a brand-new file should not inherit the family's exemption.
- [x] `src-tauri/tests/glossary_contract.rs` — extend the wire-spelling agreement test to
      `GlossaryTierWire` — rationale: finding B7; that test exists because two spellings already did.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — fix the "Hai mục" preamble that now
      lists three (B10), give the a11y item an owner the gate can read (B11), and add the entry for
      AC5's unread-CI half (B12); record B9 (the control writes into the real `target/`) and V6 with
      owners — rationale: findings B9, B10, B11, B12, V6.
- [x] `src-tauri/src/core/i18n/mod.rs` / spec task text — the Phase-2 task promised three new message
      keys; two were added and the third case correctly reuses `MessageKey::WorkNoneOpen`. Correct the
      record, not the code — rationale: finding E9.

*Phase 6 — loop 2: a CLOSED list (Ice, 2026-09-18). No further review pass follows it; each item is
accepted only on a removal counter-check that goes red for the right reason.*
- [x] `.github/workflows/ci.yml` — make the FR77 step fail when the filter matches nothing. Measured:
      `cargo test … -- --ignored --exact <a name that does not exist>` prints `running 0 tests` and
      **exits 0**, so the gate cannot go red today — rationale: finding P1. 🔴 Accept it only after
      running the proposed step against a deliberately wrong test name and seeing it FAIL.
- [x] `tests/frontend/aiPromptConfigGuards.test.ts` (new) — mock `@tauri-apps/api/core`, not the
      module, and run the REAL `src/config/aiprompt.ts`: one malformed and one well-formed case per
      guard, plus all four `aiPromptReadRecord` branches. `tests/frontend/glossaryConfigGuards.test.ts`
      is the shape and its header states the rule — rationale: finding P2; the loop-1 `{value, error}`
      table has no case on the real code.
- [x] `src-tauri/tests/ai_prompt_contract.rs` — a `serde_json::to_value` case on the wire returned by
      `assemble_and_record_prompt`, pinning `"authored"`/`"glossary"`/`"tm"` and the field names —
      rationale: finding P3; dropping `#[serde(rename_all)]` kills the feature with every suite green.
- [x] `src/aiPromptInspectorState.ts` + the Work open/close path — give `resetAiPromptInspector()` a
      real product call site so the webview record dies with the Rust one, and a case that a Work swap
      drops it — rationale: finding P4, which is finding V1 reproduced one layer up and is visible
      wrong data, not a tidiness point.
- [x] `src-tauri/tests/ipc_contract.rs` — the twin of the `replace_open_work` case, asserting the three
      clearing calls sit inside `fn close_open_work`'s body in `lib.rs` — rationale: finding P5.
- [x] `tests/frontend/aiPromptInspector.test.ts` — mount the overlay and assert
      `[data-aip-read-error="true"]` appears after a failed read and is absent after a good one
      (P6); replace the tautological `.aip-term-list-injected, .aip-note` assert (P12); fix the AC1
      counter-check so it measures what its name claims, including untrimmed text (P12) — rationale:
      an assertion that holds on both branches guards neither.
- [x] `src-tauri/src/core/ai/rag.rs` + `src-tauri/src/commands/aiprompt.rs` + `src/config/aiprompt.ts`
      + `src/AiPromptInspectorOverlay.vue` — a distinct kind for the substituted source sentence, plus
      a CSS rule and a case for every kind including `tm` — rationale: findings P7 and P9.
- [x] `src-tauri/tests/ai_boundary.rs` — a line carrying the approved prefix must still be scanned for
      a SECOND forbidden token rather than skipped whole — rationale: finding P8.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — correct the a11y entry (the
      `aria-labelledby` it describes was added in this same diff, P10) and the stale `ai_boundary.rs`
      test count (P11); record P12's remaining wording items and the `latestAssembleSequence` branch
      with owners — rationale: a ledger that describes a defect that no longer exists sends the next
      story to fix nothing.

**Acceptance Criteria:**
- [x] Given the record holds an assembled prompt, when the screen draws it, then the string shown is the
  recorded string byte for byte — counter-checked by mutating the record and seeing the screen change,
  and by removing the record read and watching a case go red naming both sides. **Verified Phase 4**:
  `happy_path_records_two_confirmed_terms_with_their_translation_and_tier` (Rust) reads the record back
  through `read_last_assembled_prompt` and asserts it equals the returned wire byte for byte; the
  frontend case *"một bản ghi có prompt KHÔNG rỗng"* asserts the rendered `<pre>` text equals the wire's
  `prompt` byte for byte. Both counter-checks in §Verification below were performed for real and both
  went red naming the mutated content, then were restored green.
- [x] Given `core/ai/` plus the one approved seam are deleted, when the tree is compiled, then every
  other capability still builds — FR77's real content, and the control that keeps the seam honest.
  **Closed 2026-09-18, post-Phase-4, before commit.** The ignored positive control depended on `git
  worktree add --detach <dir> HEAD` — a COMMIT, not the working tree — so it could never exercise this
  story's real, uncommitted seam files (`deferred-work.md`'s item ③ recorded the gap and the fix this
  entry now reports). Removed `git` from the control entirely: it now copies the actual working tree
  (`copy_dir_recursive_skipping`, plain `std::fs`, no `git`, portable) into a scratch dir and
  deletes/edits on that copy. First re-run went **red twice for real, each naming a genuine gap** the
  old `HEAD`-based control could never have reached: (1) `lib_rs_without_the_approved_ai_prompt_seam`
  only stripped the FIRST line of the multi-line `close_open_work` clearing block, leaving a dangling
  `}`; fixed by tracking brace balance and removing the whole block, with a new counter-check unit test
  (`the_lib_rs_stripper_removes_the_whole_multi_line_block_a_marker_line_opens`). (2) deleting
  `commands/aiprompt.rs` alone left `commands/mod.rs`'s `pub mod aiprompt;` dangling
  (`error[E0583]`) — the same class of bug `core/mod.rs`'s `pub mod ai;` handling already guarded, just
  not yet applied to the second seam; fixed by generalizing the declaration-stripper
  (`mod_rs_without_declaration`) and calling it a second time. After both fixes: **PASS**, for real,
  against the uncommitted tree (`cargo test --test ai_boundary --locked -- --ignored --exact
  deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling` → `ok. 1 passed`).
  `deferred-work.md`'s item ③ carries the full evidence trail and its own closing note.
- [x] Given the open action, when it is dispatched by id and when it is rebound through the shortcut
  layer, then both paths open the screen — and `registry.unbound()` is not the source the rebinding
  UI reads. **Closed 2026-09-18, post-Phase-4.** Phase 4 verified dispatch-by-id directly but left the
  rebind-through-shortcut-layer half to a structural argument ("every registered command's `run()` is
  the only code either trigger path reaches") without a standing test — and a repo-wide check found
  `applyBindings` (the runtime rebind API `ShortcutsOverlay.vue`/`shortcutsState.ts` uses, Story 1.21
  AC2/AC12) had **zero** test coverage anywhere in the suite, for any command, before this entry. Added
  `tests/frontend/aiPromptInspector.test.ts`'s `describe('AC3 — rebind qua tầng phím tắt …')`: two cases
  that call the real `applyBindings({ 'ai.prompt_inspector.open': ['Shift+K'] })`, attach the real
  `attachKeyboard` listener, fire a genuine `KeyboardEvent`, and assert `openAiPromptInspector` fires
  through the keyboard path with `aiPromptAssemble` never called — mirroring
  `editorClearSourceCuts.test.ts`'s keymap-integration precedent but exercising the REBIND path (these
  three commands register with no default chord, so a default-key test could not have covered this
  clause). Counter-checked red: pointed the rebind at the wrong command id, both cases failed naming the
  overlay staying closed; restored, green (25/25).
- [x] Given the test suites of Epic 1 through Epic 6, when run with no AI configuration present, then all
  stay green — Story 4.1's AC re-run, broadened by the 2026-08-13 note to cover Epics 5 and 6.
  **Verified Phase 4, re-measured after the post-Phase-4 AC2/AC3 closure**: `cargo test --locked` —
  **1720 passed / 0 failed / 21 ignored**, 64 `test result:` lines (Phase 4's 1719/0/21 plus the one new
  unit test the AC2 closure added) — and `npx vitest run` — **84 files / 1214 tests, 0 red** (Phase 4's
  1212 plus the two new AC3 cases) — both ran clean, both re-run fresh after every change in this story.
  Grepped the repo for a separate targeted "no AI config" suite
  (`no_ai_config`/`without_ai_config`/`NoAiConfig`) across `src-tauri/tests/`, `tests/`, `package.json`,
  `.github/workflows/ci.yml` — zero hits, so the full suite above is the only coverage this repo runs for
  this clause, not a missed narrower gate.
- [ ] Given the eleven `pre-push` gates, when each is run individually, then all pass; and the CI run is
  read before anything is called done, because `pre-push` runs only on macOS. **Gates verified
  individually, all PASS** (see the 2026-09-18 Phase 4 note for the first measurement, Phase 5's own
  Implementation Notes for the re-measurement after loop 1's fixes, and Phase 6's own Implementation
  Notes below for a THIRD fresh re-measurement after loop 2's fixes — same eleven, still all PASS) —
  but the CI run has **not** been read (this session cannot reach GitHub Actions at any phase,
  including this one), so the second clause of this AC is still not satisfied after Phase 6 either.
  Left unchecked on purpose; flagged for Ice. Ledger entry: `deferred-work.md`'s B12 item (added
  Phase 5, untouched by Phase 6) is the standing record — delete it once CI is read and green, or
  replace it with the real reason if red on a platform `pre-push` does not cover (Windows).

## Implementation Notes

### 2026-09-18 — Phase 1 (Decision 1: the seam) — done

**File touched:** `src-tauri/tests/ai_boundary.rs` only. No other file in this phase.

**What was added, with exact line ranges (post-edit file, 1132 lines total):**

- `AI_PROMPT_SEAM_COMMAND_FILE = "commands/aiprompt.rs"` (const) and
  `is_the_approved_ai_prompt_command_file(rel)` — whole-file exemption, matched by EXACT
  relative-path equality (not prefix), lines ~100-131. Negative-control tested against
  `commands/aiprompt2.rs`, `commands/aipromptset.rs`, `commands/aiprompt/mod.rs`,
  `commands/aiprompt_test.rs`, `core/commands/aiprompt.rs`, `commands/aiprompt.rs.bak` — none
  match, same anchoring lesson `is_inside_ai_module`'s `core/aim` fix already recorded.
- `AI_PROMPT_SEAM_LIB_RS_FILE = "lib.rs"`, `AI_PROMPT_SEAM_LIB_RS_MARKER =
  "crate::commands::aiprompt::"` (consts) and `line_is_the_approved_ai_prompt_seam_in_lib_rs(rel,
  code)` — line-scoped exemption, only inside `lib.rs`, only lines containing that module-path
  substring (its trailing `::` is the right-boundary anchor against a longer neighbour module
  name, e.g. `crate::commands::aipromptset::`). NOT a whole-file exemption — `lib.rs` keeps its
  ~1200 unrelated lines fully guarded.
- Both predicates wired into the real gate `no_file_outside_core_ai_names_a_bare_dependency_on_the_ai_module`
  (the two new `continue` branches, one per-file and one per-line).
- New test `the_two_approved_seams_are_matched_narrowly_and_neighbours_are_not` — positive +
  negative cases for both predicates, mirroring the file's existing
  `the_three_predicates_anchor_on_boundaries_and_do_not_fire_on_prefix_neighbours` style.
- Positive control (FR77's real content): `core_mod_rs_without_the_ai_declaration` and
  `lib_rs_without_the_approved_ai_prompt_seam` (pure stripper functions, unit-tested on synthetic
  text by `the_declaration_stripper_removes_only_the_bare_ai_line_and_keeps_its_neighbours` and
  `the_lib_rs_stripper_removes_only_the_approved_seam_lines`), plus the actual compile-based test
  `deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling`. This test
  is REAL, not a static-scan proxy: it spawns `git worktree add --detach` on a scratch temp dir,
  deletes `core/ai/` + strips `pub mod ai;` from `core/mod.rs` + (no-op today) deletes
  `commands/aiprompt.rs` and strips the lib.rs seam lines, then runs `cargo check --locked`
  there (reusing the real crate's `target/` dir so third-party dependency artifacts are not
  rebuilt — only the local crate recompiles). Cleanup runs through a `Drop` guard
  (`git worktree remove --force`) so it fires even on a panic (the test binary compiles with
  `unwind`, per `src-tauri/AGENTS.md`).
- This compile-based test is `#[ignore]`d by design, same precedent as `check:scope`/
  `check:scope:bundled` (root `AGENTS.md`: expensive, build-based checks live outside the
  default `cargo test --locked` path that `pre-push` runs every push). Manual invocation:
  `cargo test --test ai_boundary --locked -- --ignored --exact
  deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling`

**Known limitation, honestly incomplete — for Phase 2/3 to close:**

`commands/aiprompt.rs` and the two seam lines in `lib.rs` do not exist yet (Phase 2/3's job).
The gate does not require them to exist to compile or pass — confirmed by reading `all_rust_sources()`
(§Code Map: it only lists files present on disk via `walk`, so an unwritten exempted path simply
never matches, which is not a failure mode). Measured today: `cargo test --test ai_boundary
--locked` is 15 passed / 0 failed / 1 ignored in 0.09s (0.12s combined with `config_invariants`,
31 passed / 0 failed there).

But there is a real open question the Code Map itself raises and Phase 2/3 must resolve, not
assume: per the Code Map, `LastAssembledPromptState` is declared **inside** `commands/aiprompt.rs`
(not in `core::ai`), and `lib.rs`'s own precedent (`crate::commands::promptset::wire::…`,
`crate::commands::promptset::PendingPromptImportState::new(None)`) always spells out the full
`crate::commands::<module>::…` path with no bare `core::ai` token in sight. Under that shape, the
two lib.rs lines Decision 1 exempts may **never actually contain** `crate::core::ai` or
`super::ai` — meaning the `lib.rs` line-exemption built here could be inert in practice: real,
correctly scoped, and passing its own unit tests, but never exercised by the real gate once
Phase 2/3 land, because the forbidden tokens never appear on those lines to begin with. This is
not inferred to be true or false — Phase 2/3 must check the real lines they write against
`FORBIDDEN_BARE_TOKENS` once they exist, and note in their own Implementation Notes whether the
`lib.rs` exemption ever actually fires. Do not read a green gate after Phase 2/3 as proof the
exemption was needed — it may be green because the exemption fired, or green because it was
never consulted at all.

The compile-based positive control was run once, manually, and PASSED today (real `cargo check`,
not inferred): 15.60s wall time (`cargo test --test ai_boundary --locked -- --ignored --exact
deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling` → `test result:
ok. 1 passed; 0 failed`). It genuinely deletes `core/ai/rag.rs` (real code from Story 4.6) and
proves the rest of the crate still compiles without it — the substantial half of FR77's claim.
The "two seams" half is a no-op today (nothing to delete yet) and **must be re-run after Phase
2/3** to confirm the control still passes once those files carry real content; a no-op today must
not be read as "that half is verified."

**Rust suite baselines, measured fresh on this story's own starting commit (`6de227d`, still
tree, not inherited from spec 4.6):**
- `cd src-tauri && cargo test --test ai_boundary --locked`: **15 passed / 0 failed / 1 ignored**,
  0.09s.
- `cd src-tauri && cargo test --test ai_boundary --test config_invariants --locked`: ai_boundary
  15/0/1 (0.12s), config_invariants **31 passed / 0 failed / 0 ignored** (0.05s).
- `cd src-tauri && cargo test --locked` (full suite, `npm run build` run first):
  **1704 passed / 0 failed / 21 ignored**, across **63** `test result:` lines (60+ binaries plus
  the doc-test run and its companion — root `AGENTS.md` names 62 as the shape at a prior
  measurement; 63 here is this story's own fresh count, not reconciled against that prior number
  since no binary was added or removed by this phase). Wall time 4m31.77s. 0 red anywhere.

**Not touched, per Phase 1 scope:** `commands/aiprompt.rs` (Phase 2), `lib.rs` product code
(Phase 2 registers the real commands/state there), `core/i18n/mod.rs`, `config_invariants.rs`'s
`COMMAND_FILE_CENSUS`/`(tree_plain, tree_async)` counts (Phase 2), all frontend files (Phase 3),
`ai_prompt_contract.rs`/frontend test file (Phase 4).

### 2026-09-18 — Phase 2 (Rust) — done

**Files touched:**
- `src-tauri/src/commands/aiprompt.rs` (new, 429 lines) — everything below.
- `src-tauri/src/commands/mod.rs` — `+pub mod aiprompt;`.
- `src-tauri/src/lib.rs` — `app.manage(LastAssembledPromptState::new(None))` in `open_work_slot`;
  two `generate_handler!` entries; a clearing branch added to `close_open_work` (see judgment call
  below).
- `src-tauri/src/core/i18n/mod.rs` — two new `message_keys!` variants (see below).
- `src-tauri/tests/config_invariants.rs` — one `COMMAND_FILE_CENSUS` row + re-counted
  `(tree_plain, tree_async)`.
- `src/i18n/vi.json` — **the one non-Rust file this phase touched**, and only two flat entries:
  `"err.ai_prompt.no_set_selected"` and `"err.ai_prompt.segment_not_in_chapter"`. Not optional: the
  Code Map's own bullet for this task names `tests/ipc_contract.rs::every_message_key_exists_in_vi_json`
  as running over `MessageKey::ALL` — with the two new variants declared and no matching `vi.json`
  entries, that sync test (part of the required `cargo test --locked` 0-red bar) would have gone red
  inside Phase 2's own verification. This is the message-string companion of a Rust-catalogue key,
  not UI/component work — every other frontend file (`.vue`, `.ts`, the overlay, the command
  registration, every screen-copy key) is untouched and stays Phase 3's.

**Exact names Phase 3 needs, verbatim:**

Command ids (registered in `generate_handler!`):
- `ai_prompt_assemble(promptSetName: string | null, segmentId: number) -> Result<AssembledPromptWire, IpcError>`
  — plain `#[tauri::command]`, **not** async.
- `ai_prompt_read_record() -> Option<AssembledPromptWire>` — plain `#[tauri::command]`, **no**
  `Result` wrapper. "No record yet this session" is `null`/`None`, never an `IpcError` — the I/O
  Matrix is explicit that this is a state, not an error.

Pure functions in `commands::aiprompt` (what `tests/**` calls):
- `pub fn assemble_and_record_prompt(global: Option<&Store>, open: Option<&OpenWork>, record: &LastAssembledPromptState, prompt_set_name: Option<&str>, segment_id: i64) -> Result<AssembledPromptWire, IpcError>`
- `pub fn read_last_assembled_prompt(record: &LastAssembledPromptState) -> Option<AssembledPromptWire>`

Managed state: `pub type LastAssembledPromptState = std::sync::Mutex<Option<AssembledPromptRecord>>`,
managed unconditionally in `open_work_slot` beside `OpenWorkState`/`PendingPromptImportState`.

**Judgment call — clearing the record on Work close, which the spec's Code Map does not mention.**
I added a branch to `close_open_work` that sets the record back to `None` whenever the open Work
closes. Reason: `segment_id`/`chapter_id` are row keys inside ONE `project.db`; two different
`.atproj` folders can coincidentally reuse the same integers (both start counting from 1). Letting
the record survive a Work switch risks a record built from Work A being read as describing Work B's
identically-numbered segment/chapter — a stronger version of exactly the identity confusion the
Boundaries section warns against. The I/O Matrix's "Stale record" row only covers a segment change
*within* the same open Work (still shown, with its own identity, per that row); it says nothing
about a Work close. Phase 3 should know: after closing a Work, `ai_prompt_read_record` returns
`None`, not a stale-but-still-shown record.

Wire types (`serde::Serialize`, plain structs, snake_case fields, no `#[serde(rename_all)]` on any
struct — only on the value-only `GlossaryTierWire` enum):
- `AssembledPromptWire { prompt: String, segment_id: i64, chapter_id: i64, prompt_set_name: String, prompt_set_tier: PromptSetTierWire, ledger: InjectionLedgerWire }`
  — `prompt_set_tier` reuses `crate::commands::promptset::PromptSetTierWire` (imported, not a third
  duplicate Global/Work mirror).
- `InjectionLedgerWire { glossary: GlossaryInjectionStatusWire, tm: TmInjectionStatusWire, unknown_markers: Vec<String>, source_segment_missing: bool }`.
- `GlossaryInjectionStatusWire { kind: &'static str, injected: Option<Vec<InjectedGlossaryTermWire>>, suppressed_by_pending_overlap: Option<Vec<SuppressedGlossaryTermWire>> }`
  — `kind` is `"not_asked"` | `"asked"`. **Judgment call**: hand-rolled string tag, the same shape
  `commands::promptset::PromptImportTierPreviewWire::kind` already uses (`"new"`/`"identical"`/
  `"conflict"`), not serde's `#[serde(tag = …)]` — chosen to stay inside an existing convention
  rather than mint a new one. The two payload fields are ALWAYS both `Some` (kind `"asked"`) or
  both `None` (kind `"not_asked"`) — never a mixed combination — so Phase 3 can branch on `kind`
  alone and never has to guess which payload field is meaningful.
- `TmInjectionStatusWire { kind: &'static str, similar_segments: Option<Vec<SimilarSegmentWire>> }`
  — `kind` is `"not_built_yet"` | `"searched"`. This story's call site never produces `"searched"`
  (the `tm` argument to `assemble_prompt` is always `None` per the frozen Boundaries), but the wire
  type and its `From<TmInjectionStatus>` cover it so the match stays exhaustive ahead of Epic 7.
- `InjectedGlossaryTermWire`/`SuppressedGlossaryTermWire` — both
  `{ source_term: String, translation: String, start: usize, end: usize, tier: GlossaryTierWire }`,
  all five fields carried (this is the exact field set the frozen spec calls out as the one a
  careless mirror would drop).
- `GlossaryTierWire { Global, Work }` with `#[serde(rename_all = "snake_case")]` (serializes
  `"global"`/`"work"`) — new, local to this file: `core::glossary::GlossaryTier` derives only
  `Deserialize` (it's decoded FROM the wire in `glossary.add_term`/`update_term`), never `Serialize`
  before this story.
- `SimilarSegmentWire { source_text: String, target_text: String }` — mirrors `core::tm::SimilarSegment`;
  unreachable via this story's call site, same reason as `TmInjectionStatusWire`'s `"searched"` arm.

**`message_key` variants added, verbatim** (`core/i18n/mod.rs`):
- `AiPromptNoSetSelected => "err.ai_prompt.no_set_selected"` — **no params**. Covers BOTH sub-causes
  of I/O Matrix "No set selected" (no name sent at all, and a name sent that does not resolve in the
  two-tier map) under one key: same underlying fact ("no effective prompt set"), and the codebase's
  own rule is one key per fact, not per call site.
- `AiPromptSegmentNotInChapter => "err.ai_prompt.segment_not_in_chapter"` — params `segment_id`,
  `chapter_id`. I/O Matrix "an id the chapter does not hold".
- **Judgment call — only TWO new variants, not three.** The Code Map names three slots
  ("no-set-selected, no-Work-open, segment-id-not-in-chapter") and is explicit that the last two
  must be distinct keys. But `MessageKey::WorkNoneOpen` ("err.work.none_open") already exists and
  is exactly the "no Work open" fact — reused throughout the codebase via
  `crate::commands::chapter::no_work_open()` under an explicit standing rule ("cùng câu, cùng
  nghĩa, một khoá"). `assemble_and_record_prompt` reuses that same function (both directly, and
  transitively through `crate::commands::segment::read_open_chapter_segments`) rather than minting
  `AiPromptNoWorkOpen`. So the spec's "three slots, two of them must differ" instruction is
  satisfied — `no-Work-open` and `segment-id-not-in-chapter` ARE two distinct keys
  (`WorkNoneOpen` vs the new `AiPromptSegmentNotInChapter`) — just that one of the three slots was
  already filled before this story started.

**Two new `vi.json` entries, verbatim:**
- `"err.ai_prompt.no_set_selected": "Chưa chọn bộ prompt nào, hoặc bộ đã chọn không còn tồn tại — chưa lắp ráp được prompt nào."`
- `"err.ai_prompt.segment_not_in_chapter": "Câu số {segment_id} không thuộc Chương {chapter_id} đang mở — chưa lắp ráp được prompt nào."`

**Error-check ordering inside `assemble_and_record_prompt` — a judgment call, not a ranked
contract.** The I/O Matrix does not order its rows: this function checks the global store, then
resolves/looks up the effective prompt set ("no set selected"), THEN checks Work-open and
segment-in-chapter. So a request missing both a Work and a set name gets `ai_prompt.no_set_selected`
first. Documented in the function's own doc-comment. If Phase 4's contract cases need a specific
order asserted, this is the order they will observe.

**`(tree_plain, tree_async)` re-count — exact method.** Wrote `commands/aiprompt.rs` with its two
plain `#[tauri::command]`s (zero `async` — neither opens an OS dialog; both only touch `Store` reads
and one in-process `Mutex`), added a `COMMAND_FILE_CENSUS` row for it, then ran
`cargo test --test config_invariants --locked`. The old literal `(65, 28)` failed
`every_command_bearing_file_is_classified_with_measured_attribute_counts`, and its own panic message
reports the real counted numbers (the test computes them independently of any constant). Set the
constant to exactly what the test reported, re-ran, green. **Final: `(67, 28)`** across **fourteen**
command-bearing files (`commands/mod.rs` itself now declares **thirteen** `pub mod` lines — this
file's own hunk two lines above adds `pub mod aiprompt;`, so the count moved from twelve to thirteen;
⚠️ **SỬA 2026-09-18, lượt rà soát build**: bản đầu ghi "unchanged... twelve," tự mâu thuẫn với chính
lượt sửa nó vừa mô tả — bắt được và sửa ở review, đếm lại thật bằng `grep`).

**Phase 1's open question, answered — the lib.rs line-exemption is confirmed INERT for this story,
empirically, not inferred:**
- The whole-file exemption is real and load-bearing: `commands/aiprompt.rs:27` genuinely writes
  `use crate::core::ai::rag::{…};` — a real bare `crate::core::ai` token. Counter-check: commented
  out `ai_boundary.rs`'s `if is_the_approved_ai_prompt_command_file(rel) { continue; }` branch and
  re-ran `no_file_outside_core_ai_names_a_bare_dependency_on_the_ai_module` — it went **red**, naming
  exactly `commands/aiprompt.rs:27  crate::core::ai  |  use crate::core::ai::rag::{`. Restored the
  branch, re-ran, green.
- The lib.rs lines carrying the `crate::commands::aiprompt::` marker today are **four**, not the two
  Phase 1 anticipated: the two `generate_handler!` entries, the one `app.manage(...)` line, and a
  fourth Phase 1 did not foresee — the `close_open_work` clearing check
  (`handle.try_state::<crate::commands::aiprompt::LastAssembledPromptState>()`) I added for the
  judgment call above. `grep -n "core::ai\|super::ai" src-tauri/src/lib.rs` returns **zero** matches
  anywhere in the whole file — none of the four (or any other) line ever contains a forbidden bare
  token. Counter-check: commented out `line_is_the_approved_ai_prompt_seam_in_lib_rs`'s `continue`
  branch and re-ran the same gate test — it **stayed green**. Restored the branch, re-ran, green
  again (for symmetry with the file-exemption check above).
  **Conclusion: the lib.rs line-exemption is real, correctly scoped, and passes its own unit tests
  — but for every line it actually matches in this story's tree, it is never the reason the gate
  stays green, because none of those lines ever needed exempting.** This does not make it dead code
  to delete: Decision 1 approved it as a named, controlled seam independent of whether any one
  story's registration lines happen to need it, and a future story's `lib.rs` lines (or a refactor
  of this one) could genuinely write a bare token there. Read a future green gate on `lib.rs` as
  "the exemption fired" only after checking, the same way this note just did — not by assumption.

**Counter-check the seam by removing the call — "a compile error is not a discharge."** Commenting
out the `assemble_prompt(...)` call directly does NOT break compilation on its own (`sentence` and
`glossary` are already bound and used by other statements), so the spec's fallback ("stub instead
of delete, if deleting won't compile") was not needed for THAT reason — but a stub was still
necessary to get an actual failing *assertion* rather than nothing, because no permanent test in
today's tree calls this function at all yet (`ai_prompt_contract.rs` is Phase 4's file, not written
yet). What I actually did: wrote a **temporary** file, `src-tauri/tests/aiprompt_temp_probe.rs` (not
a deliverable, deleted immediately after use — it is NOT `ai_prompt_contract.rs` and does not
substitute for it), that builds a real `OpenWork` via `create_work_from_text`, a Global prompt set
with body `"Truoc {{source_segment}} Sau"` via `commands::promptset::prompt_set_create`, calls
`assemble_and_record_prompt`, and asserts `result.prompt == "Truoc cau nguon mau Sau"` (the real
first segment's source text, injected in place of the marker). Ran it green first, confirming the
whole path (resolve → read chapter → `gather_glossary_context` → `assemble_prompt`) is genuinely
wired end to end. Then replaced the real
`let (prompt, ledger) = assemble_prompt(&set.body, sentence, glossary, None);` line with a fixed
stub tuple (a literal placeholder string plus an empty `NotAsked`/`NotBuiltYet` ledger, independent
of `set.body`/`sentence`; kept `glossary` referenced via `let _ = &glossary;` so nothing else broke)
— re-ran the probe: **red**, `left: "STUB -- khong goi core::ai::rag::assemble_prompt"` vs
`right: "Truoc cau nguon mau Sau"`, naming exactly the missing call. Restored the real line, re-ran,
green. Deleted the probe file.
**Honest gap this leaves for Phase 4:** until `ai_prompt_contract.rs` exists for real, nothing in
the committed suite exercises `assemble_and_record_prompt`/`read_last_assembled_prompt` at all — the
counter-check above is real evidence the wiring works today, but it is not a standing guard.
Phase 4 needs at minimum one case shaped like this probe (a real `OpenWork` + a real prompt set +
an assert on the exact returned `prompt` string), not merely a mocked/unit-level call.

**Measured fresh, this story's own tree (`npm run build` run first, both times):**
- `cd src-tauri && cargo test --test ai_boundary --test config_invariants --locked`: ai_boundary
  **15 passed / 0 failed / 1 ignored** (0.06s); config_invariants **31 passed / 0 failed / 0
  ignored** (0.05s) — both green with the real `commands/aiprompt.rs` on disk (not the Phase-1
  never-existed state).
- `cd src-tauri && cargo test --locked` (full suite): **1704 passed / 0 failed / 21 ignored**,
  across **63** `test result:` lines — identical to Phase 1's own baseline on `6de227d`. This is
  expected, not a red flag: Phase 2 adds product code and two new commands but zero new test cases
  (Phase 4 owns `ai_prompt_contract.rs`/the frontend test), so the population of tests did not
  change, only the tree they run against. Compile time 59.99s this run.
- 0 red anywhere, at any point in this phase (including during both temporary red counter-checks
  above, which were expected reds on temporarily-broken code, immediately restored).

**Not touched, per Phase 2 scope:** every frontend file except the two `err.ai_prompt.*` strings in
`src/i18n/vi.json` (see above) — no `.vue`, no other `.ts`, no command registration, no screen-copy
keys; `tests/ai_prompt_contract.rs` and `tests/frontend/aiPromptInspector.test.ts` (Phase 4);
`deferred-work.md` (Phase 4's task list already owns writing the three named handoffs — this phase
adds none of its own beyond what is already written above as open/incomplete for Phase 4 to close).

### 2026-09-18 — Phase 3 (Webview) — done

**Files created:**
- `src/config/aiprompt.ts` — IPC adapter for both Rust commands. Wire types
  (`AssembledPromptWire`, `InjectionLedgerWire`, `GlossaryInjectionStatusWire`,
  `TmInjectionStatusWire`, `InjectedGlossaryTermWire`, `SuppressedGlossaryTermWire`,
  `SimilarSegmentWire`) mirror `commands/aiprompt.rs` field-for-field, snake_case. Runtime type
  guards check every nested level, including the three-valued discriminated-union invariant
  (`kind: 'not_asked'` ⇒ both payload fields `null`; `kind: 'asked'`/`'searched'` ⇒ both/one
  payload field is an array, possibly empty) — a malformed wire object is treated as
  `UNKNOWN_IPC_ERROR`, never trusted. `prompt_set_tier` reuses `PromptSetTier` from
  `./promptset.ts`; each term's `tier` reuses `GlossaryTierWire` from `./glossary.ts` (aliased
  `GlossaryTier` on import) — no third Global/Work mirror type. Two exported functions:
  `aiPromptAssemble(promptSetName, segmentId)` and `aiPromptReadRecord()`, both never throw, both
  follow the `{ value | null, error }` / `hasIpcBridge()` shape of `config/promptset.ts`.
  `aiPromptReadRecord` has no `IpcError` return channel (the Rust command carries no `Result`) —
  an `IpcError` there is a configuration bug, logged and swallowed, not surfaced.
- `src/aiPromptInspectorState.ts` — module state: `record` (`AssembledPromptWire | null`),
  `overlayOpen`, `assembleBusy`, `assembleError`, one shared `sequence` counter guarding BOTH the
  assemble write path and the read-record path (so a late-resolving read can never clobber a
  later assemble's result, and vice versa). Exports: `openAiPromptInspector()`/
  `closeAiPromptInspector()` (the open handler calls `refreshAiPromptRecord()` — Decision 2: reads
  only, never assembles), `assembleCurrentAiPrompt(promptSetName, segmentId)` (the write path —
  calls `aiPromptAssemble` then stores the result into the same `record` ref the inspector reads),
  `refreshAiPromptRecord()`, two pure functions `glossaryInjectionSummary(record)` and
  `aiPromptRecordIsStale(record, focusedSegmentId)` (both take the actual `DeepReadonly<…>` shape
  `readonly()` produces, aliased `ReadonlyAssembledPromptWire`, so callers reading the exported
  refs need no cast), and `resetAiPromptInspector()`.
- `src/AiPromptInspectorOverlay.vue` — the screen. Structure copied from `PromptLibraryOverlay.vue`
  (`useSelectionSurface(panel, 'display')`, `watch(...)`+`focusReturnTargetOnOpen`, `focusableWithin`/
  `trapTab`, `role="dialog" aria-modal="true"`). No import of `aiPromptAssemble`/
  `assembleCurrentAiPrompt` anywhere in this file — opening genuinely cannot assemble, by absence of
  the capability, not by convention.

**New command ids (registered in `src/commands/index.ts`, `registerAll`):**
- `ai.prompt_inspector.open` — labelKey `command.ai.prompt_inspector.open` ("Xem prompt"). Guarded by
  `deps.openAiPromptInspector`, `keys: undefined`.
- `ai.prompt_inspector.close` — labelKey `command.ai.prompt_inspector.close` ("Đóng Xem prompt").
  Guarded by `deps.closeAiPromptInspector`, `keys: undefined`.
- `ai.prompt.assemble` — labelKey `command.ai.prompt.assemble` ("Lắp prompt cho câu này"). Guarded by
  `deps.assembleAiPrompt`, `keys: undefined`. **This is the third command the task checklist's
  literal text did not list — see the design-gap resolution below.**

**New i18n keys (`src/i18n/vi.json`):** `command.ai.prompt_inspector.open`,
`command.ai.prompt_inspector.close`, `command.ai.prompt.assemble`; shared summary copy
`ai.prompt.summary_no_record`, `ai.prompt.summary_not_asked`, `ai.prompt.summary_asked` (param
`count` — used identically by the panel's compact line and the overlay's Glossary section, so the
"no record / not asked / asked-N" wording is written exactly once and read from two places);
screen-only copy `ai.prompt_inspector.title`, `.identity` (params `segment_id`, `chapter_id`,
`name`, `tier`), `.tier_global`, `.tier_work`, `.stale_notice` (params `record_segment_id`,
`focused_segment_id`), `.body_heading`, `.body_empty_note`, `.source_missing_warning`,
`.markers_heading`, `.glossary_heading`, `.glossary_not_asked`, `.glossary_injected_heading`,
`.glossary_injected_empty`, `.glossary_suppressed_heading`, `.glossary_suppressed_empty`,
`.glossary_suppressed_reason_pending_overlap`, `.tm_heading`, `.tm_not_built_yet`, `.tm_searched`
(param `count`); panel-only copy `panel.ai_translation.assemble_busy`,
`panel.ai_translation.assemble_no_segment_hint`.

⚠️ **First attempt used `ai_prompt.*`/`ai_prompt_inspector.*` as the key prefix and it failed
`check:i18n` Kiểm B** — `COMMAND_ID_RE`/`KEY_RE` (`^[a-z0-9]+(\.[a-z0-9_]+)+$`) forbids an
underscore in the FIRST dot-segment; underscores are only legal from the second segment on. The
pre-existing `err.ai_prompt.no_set_selected` (Phase 2) is fine because its first segment is `err`.
Renamed to `ai.prompt.*`/`ai.prompt_inspector.*` — matching the shape the command ids already used
(`ai.prompt_inspector.open`) — and Kiểm B went green. Recorded here so Phase 4 (or anyone minting a
fourth `ai.*` key) does not re-hit this.

**The design gap — where the "assemble this sentence" trigger lives, and why.**

The spec is explicit that Decision 2 requires "one user-visible button in the AI panel that Story
4.8 supersedes," but the Phase 3 task checklist's literal bullet list only named two commands
(open/close) and one open-button `@click`. Resolved by building a THIRD registered command,
`ai.prompt.assemble`, exactly as the assignment's suggested resolution laid out:
- Its `run()` (wired in `main.ts`) calls `assembleCurrentAiPrompt(selectedPromptSetName.value,
  editorCaretSegmentId.value)` — reading both values AT DISPATCH TIME, never storing them.
- `selectedPromptSetName` is `promptSetState.ts`'s existing effective-set ref (Story 4.4,
  `setSelectedPromptSetName`, 0 `invoke` — the frontend-local selection the spec's Code Map already
  names). No second "which set" concept was created.
- `editorCaretSegmentId` is `editorPanelState.ts`'s existing "segment with keyboard/caret focus" ref
  — already the single source three other consumers read (`GlossaryConfirmStrip.vue`,
  `segmentHistoryState.ts`, `GridPanel.vue`). **This directly answers the assignment's contingency
  question** ("if AiTranslationPanel genuinely has no stable current-segment concept, stop and
  document it as a blocker") — it does have one, just not inside `AiTranslationPanel.vue` itself;
  it is the app-wide "focused segment" ref every per-segment feature already reads, one directory
  up. `AiTranslationPanel.vue` had never needed to read it before because Epic 4's AI-call content
  (Story 4.8+) had not landed yet — the panel was framework-only (Story 1.14).
- The command does NOT open the inspector overlay — confirmed by grep: no line in
  `aiPromptInspectorState.ts` calls `openAiPromptInspector`/sets `overlayOpen` outside
  `openAiPromptInspector()` itself, and `AiPromptInspectorOverlay.vue` imports nothing named
  `assemble`.
- `ai.prompt_inspector.open` calls only `refreshAiPromptRecord()` (→ `ai_prompt_read_record`) —
  confirmed by grep: no line in that function or in `openAiPromptInspector()` calls
  `aiPromptAssemble`/`assembleCurrentAiPrompt`. This holds **regardless of trigger source**: a
  future rebind of either command to a keyboard chord would behave identically to a click, because
  the two commands' `run()` bodies are the only behavior either trigger path can reach — there is
  no second, click-only code path that assembles.
- Placement: a new `.ai-inspector-bar` in `AiTranslationPanel.vue`, below the existing
  `.ai-prompt-bar` (bộ prompt selector), holding: the Glossary summary line (reads
  `glossaryInjectionSummary(aiPromptRecord.value)`, `data-ai-prompt-summary-kind` attribute for
  Phase 4), an inline error line for the assemble action's own `IpcError` (via `tError()`), a hint
  line shown only when no segment is focused, then the two buttons (`ai.prompt.assemble` first,
  `ai.prompt_inspector.open` second) — matching `EXPERIENCE.md`'s KF-2 step 4 ordering (summary
  line, then "Xem prompt").
- The assemble button is `:disabled` whenever `editorCaretSegmentId === null` (computed
  `canAssemble`) — a first, informed layer — AND `assembleCurrentAiPrompt` itself no-ops with a
  `console.warn` (never throws) if called with `segmentId: null` regardless — a second, defensive
  layer, matching the project's "a function run from a keyboard/mouse trigger never throws, it
  warns" convention (`editorPanelState.ts::splitChapterHere` is the named precedent).

**Three-valued rendering, traced by hand (no formal test file written — Phase 4's job):**
- No record this session: `aiPromptRecord === null` ⇒ overlay shows `ai.prompt.summary_no_record`
  (`data-aip-record-state="none"`) and nothing else renders; panel's summary line shows the same
  key via `glossaryInjectionSummary`'s `'no_record'` branch.
- Record with an empty `prompt` string: distinct from the above — `aiPromptRecord !== null` but
  `.prompt === ''` renders `data-aip-body-state="empty"` / `ai.prompt_inspector.body_empty_note`,
  while every other section (identity, Glossary, TM) still renders normally from the same record.
- Glossary `NotAsked` vs `Asked` (0 or N): `glossaryAsked` computed narrows the union once in
  script; `glossaryAsked === null` ⇔ `not_asked` (renders `.glossary_not_asked`, no injected-count
  claim, per I/O Matrix "Marker absent"); `glossaryAsked !== null` renders
  `ai.prompt.summary_asked` with `count = glossaryAsked.injected.length` — 0 renders the literal
  "Đã chèn 0 thuật ngữ Glossary." wording, which the I/O Matrix's "Asked, nothing matched" row
  explicitly wants (unlike `NotAsked`, where a count claim is explicitly forbidden).
- Suppressed list: rendered as its own read-only `<ul>` (Decision 3 — no action element anywhere in
  its rows), each row carrying `tier` via the same `tierLabel()` helper the injected rows use, plus
  a fixed reason string (`glossary_suppressed_reason_pending_overlap`) — the only suppression
  reason the ledger's shape carries today (`suppressed_by_pending_overlap` is the list's own name).
- TM: `tmSearched === null` ⇔ `not_built_yet` (only reachable branch this story; `assemble_prompt`'s
  `tm` argument is always `None` at this call site per Phase 2) — renders `.tm_not_built_yet`, never
  "0 similar sentences." The `searched` branch is coded and typed but structurally unreachable until
  Epic 7 — verified by reading `commands/aiprompt.rs` again: no call site produces it.
- Stale record: `aiPromptRecordIsStale(record, editorCaretSegmentId.value)` compares
  `record.segment_id` against the live caret ref; `focusedSegmentId === null` short-circuits to
  `false` (no comparison claim when nothing is focused, e.g. Work just closed and the record was
  already cleared server-side per Phase 2's judgment call — in that case `aiPromptReadRecord()`
  itself returns `null` on the next open, so this path mostly matters for "still the same Work,
  caret moved to a different segment").
- Unknown markers and the missing-source-segment warning render as their own always-checked blocks
  (not folded into any other three-valued state), each reading a plain field off the ledger.

**Test hooks left for Phase 4 (`tests/frontend/aiPromptInspector.test.ts`)** — selectors follow the
two conventions already in this codebase (`.class-name` for structural elements, `data-*` for
semantic state, per `tests/frontend/readingFrontierDom.test.ts`/`libraryWorks.test.ts`):
- Root: `.aip-scrim` (only present when `aiPromptInspectorIsOpen`), `.aip-panel` (`ref="panel"`,
  `role="dialog"`, `aria-modal="true"`), close button `.aip-close`.
- `[data-aip-record-state="none"|"present"]` — the top-level no-record vs has-record branch.
- `[data-aip-body-state="empty"|"present"]` on the prompt-text block — empty-body vs populated.
- `[data-aip-stale="true"]` on `.aip-stale` — present only when stale; absent (not `"false"`) when
  not stale, since the `<p>` itself is `v-if`-gated.
- `[data-aip-glossary-kind="not_asked"|"asked"]` — the Glossary three-valued branch.
- `[data-aip-tm-kind="not_built_yet"|"searched"]` — the TM branch.
- `.aip-term-list-injected` / `.aip-term-list-suppressed`, each `<li class="aip-term-row">` with
  child spans `.aip-term-source` / `.aip-term-translation` / `.aip-term-tier` (and
  `.aip-term-reason` on suppressed rows only).
- `.aip-marker-list code` — one per unknown marker, raw text (e.g. `"{{chapter_context}}"`).
- Opener/focus-return selector: `[data-ai-prompt-inspector-open]` (the button in
  `AiTranslationPanel.vue`).
- In `AiTranslationPanel.vue`: `.ai-inspector-summary[data-ai-prompt-summary-kind="no_record"|
  "not_asked"|"asked"]` (the compact line), `.ai-inspector-assemble[data-ai-prompt-assemble]`
  (`:disabled` reflects `canAssemble`), `.ai-inspector-open[data-ai-prompt-inspector-open]`,
  `.ai-inspector-alert` (assemble error, present only when `aiPromptAssembleError !== null`),
  `.ai-inspector-hint` (present only when no segment is focused).
- State module functions Phase 4 will most likely call directly (same pattern as
  `glossaryQueue.test.ts` importing `glossaryQueueState.ts` functions under `vi.mock` of the
  adapter): `openAiPromptInspector`, `closeAiPromptInspector`, `assembleCurrentAiPrompt`,
  `refreshAiPromptRecord`, `glossaryInjectionSummary`, `aiPromptRecordIsStale`,
  `resetAiPromptInspector`, all exported from `src/aiPromptInspectorState.ts`; mock
  `src/config/aiprompt.ts`'s `aiPromptAssemble`/`aiPromptReadRecord`, same shape as
  `glossaryQueue.test.ts`'s `vi.mock('../../src/config/glossary', …)`.

**Gate results, this phase, measured fresh:**
- `npm run build` — **PASS** (`vue-tsc` both configs clean, `vite build` succeeds; two pre-existing
  `INEFFECTIVE_DYNAMIC_IMPORT` warnings from `editorPanelState.ts`, unrelated to this phase, same
  warnings present before this phase's edits).
- `npx vitest run` — **PASS**, 83 test files / 1189 tests, 0 red (same file/case count as before this
  phase — no new frontend test file was written, per Phase 3's scope; Phase 4 raises this count).
- `npm run check:commands` — **PASS**. 166 commands total (was 163 before this phase — the three new
  `ai.*` ids), 122 `@click`, 176 `dispatch()` calls, 30 `.vue` + 72 `.ts` files scanned, 0 exemptions.
  `ai.prompt_inspector.open`/`.close`/`ai.prompt.assemble` all appear correctly in the `unbound()`
  listing (AC6 — no default chord, as declared).
- `npm run check:i18n` — **PASS** after the `ai_prompt.*` → `ai.prompt.*` key-shape fix above; 963
  keys, 755 checked text nodes, 176 named exemptions total. Exactly **10 of those are new**, per the
  gate's own per-line listing: 8 in `AiPromptInspectorOverlay.vue` (lines 184/195/220/223/225/240/
  243/245 — the prompt-text block, the unknown-marker code, and each data-bound
  source_term/translation/tier span across the injected and suppressed lists) and 2 in
  `AiTranslationPanel.vue` (lines 135/149 — the new summary line and the assemble button's busy/idle
  ternary). Two OTHER `AiTranslationPanel.vue` lines the gate lists (118/122) are pre-existing
  exemptions from Story 4.4, unchanged by this phase — not new, despite sitting in a file this phase
  edited; counted by re-running the gate and reading its own line numbers, not estimated. The
  `.identity`/`.stale_notice` interpolations needed no exemption at all (`{{ t('ai.prompt_inspector.…'`
  already satisfies `ALLOWED_CALL_RE` on its own) — the `aura-allow-text` comments placed above them
  in the source are harmless but not load-bearing; the gate does not count or require them there.
  193 placeholders, all resolving.
- `npm run check:panel-refs` — **PASS**. 72 `.ts` files, 389 module-level cells (5 new: `record`,
  `overlayOpen`, `assembleBusy`, `assembleError`, `sequence` in `aiPromptInspectorState.ts`), 31 named
  exemptions (unchanged — none of the five new cells needed one; all five are reached by
  `resetAiPromptInspector()`).
- `npm run check:tokens` — **PASS**. New z-index (`.aip-scrim`) carries the same
  `aura-allow-z-index` exemption every other overlay's scrim carries; no literal colours, no
  intermediate `opacity` were added (the disabled-button style copies `PromptLibraryOverlay.vue`'s
  `.pl-act:disabled` shape — colour token only, no opacity).
- `npm run check:layout` — **PASS**. The overlay's `document.querySelector`/`document.activeElement`
  calls reuse members already on the allow-list (the exact same two members `PromptLibraryOverlay.vue`
  uses) — the gate reports 17 distinct `window`/`document` member NAMES across 102 files; the member
  count is unaffected (no new member name introduced), though the file count includes this phase's
  new `.vue` file among the 102 (not independently re-measured pre-phase, since the gate only reports
  the combined post-phase figure).
- Also ran, though not required by this phase's explicit gate list, as a safety check before
  handoff: `npm run check:lint` — PASS (clean); `npm run check:gates` — PASS; `npm run check:deps` —
  PASS (0 new dependencies); `npm run check:debt-owner` — PASS (unaffected).
- Rust suite NOT re-run this phase (no Rust file touched) — Phase 2's baseline
  (`cargo test --locked`: 1704 passed / 0 failed / 21 ignored) stands unchanged; Phase 4 should
  re-measure once `ai_prompt_contract.rs` exists.

**Judgment calls made, not explicitly pinned by the spec:**
1. Shared `ai.prompt.summary_*` i18n keys between the panel's compact line and the overlay's
   Glossary section (rather than two near-duplicate key sets) — one copy of the "no record / not
   asked / asked-N" wording, read from two places, directly serving §Always's "reads from the same
   record" language for the summary count.
2. The suppressed list's fixed reason string is NOT sourced from the ledger (which carries no
   free-text reason field, only list membership) — it is one i18n key naming the one reason this
   story's `InjectionLedger` shape can express (`suppressed_by_pending_overlap`). If Epic-later work
   adds a second suppression reason to the ledger, this key and its binding will need to become
   per-row rather than fixed — flagged here, not filed to `deferred-work.md` since it is not a gap
   in THIS story's scope (the ledger has exactly one suppression reason today).
3. `assembleCurrentAiPrompt`/`aiPromptReadRecord` share one `sequence` counter rather than two
   independent ones — deliberate: both write into the same `record` ref, so ordering must be
   arbitrated across BOTH call sites, not just within each independently.
4. The overlay does not attempt to visually interleave injected Glossary text inline within the raw
   `prompt` string (the mockup's approach). The wire only carries the FINAL flattened `prompt`
   string plus a separate structured ledger — no span offsets into the final prompt exist on the
   wire (the ledger's `start`/`end` are offsets into the SOURCE SENTENCE, per `rag.rs`, not into the
   assembled prompt). Re-deriving such offsets on the frontend would mean re-parsing/re-matching
   against the record's own text — not a re-assemble (no call to `assemble_prompt` or a second
   marker scanner), but also not something Phase 3 attempted, since the safer, unambiguous reading
   of "visually separable from the user-authored body" is satisfied by rendering the injected/
   suppressed Glossary content as its own clearly-labelled, distinctly-backgrounded structured
   section (mirrors the mockup's own separate "Glossary chèn động" block) rather than by inline
   highlighting inside the flat prompt text. Recorded here as a deliberate reading, not an oversight,
   in case Ice reads it differently.

**Not touched, per Phase 3 scope:** no Rust file; `tests/ai_prompt_contract.rs` and
`tests/frontend/aiPromptInspector.test.ts` (Phase 4); `deferred-work.md` (Phase 4's task list already
owns writing the three named handoffs).

### 2026-09-18 — Phase 4 (Tests that move) — done, with two ACs left honestly unchecked

**Files created:**
- `src-tauri/tests/ai_prompt_contract.rs` (new, 15 named cases) — one case per I/O Matrix row, the
  three-valued pairs first (Marker absent / Asked-nothing-matched, then TM never built, then Nothing-
  recorded-yet / Empty-body), then Happy path, Pending overlap, Sentence marker missing, Unknown
  marker, the two "No set selected" sub-causes, the two "No Work / segment not in chapter" cases
  (distinct keys, plus one case that diffs both errors' `.code()`/`.message_key()` directly against
  each other), and one case proving the record's `segment_id` changes across two assembles on two real
  segments (the wire-level fact `aiPromptRecordIsStale` needs). Fixtures are real: `Store::open` +
  `create_work_from_text` + `add_manual_term` + `commands::promptset::prompt_set_create` — no mocked
  Store anywhere. Every injected/suppressed assertion checks `source_term`/`translation`/`start`/
  `end`/`tier` individually (computed from the real segment's `source_text` via `.find(...)`, not
  hard-coded offsets), per the Known Pitfalls warning against list-length-only asserts.
- `tests/frontend/aiPromptInspector.test.ts` (new, 23 cases) — `mount()`s `AiPromptInspectorOverlay.vue`
  for every three-valued/rendering case (not just calling the pure state functions), per root
  `AGENTS.md`'s "412 lines of ribbon tests never mounted the component" pitfall. `config/aiprompt.ts`
  mocked via `vi.mock` (two spies, `assembleMock`/`readRecordMock`); `panels/editorPanelState.ts`
  `vi.doMock`ed to a real controllable `ref()` exposing only `editorCaretSegmentId`, exact convention
  copied from `glossaryConfirmStripTemplate.test.ts::freshStrip`. Covers: `glossaryInjectionSummary`
  three-valued (no_record/not_asked/asked-0/asked-N); `aiPromptRecordIsStale` four cases; mount-level
  no-record vs empty-prompt-record (`data-aip-record-state`/`data-aip-body-state`); mount-level Glossary
  not_asked/asked-0/asked-N with per-row tier+source+translation assertions on BOTH a 'global'-tier and
  a 'work'-tier row in the SAME case (so a hard-coded label can't pass by accident); the suppressed list
  with its reason string AND an explicit `row.findAll('button')`/`findAll('a')` assertion of length 0
  (Decision 3, read-only); TM not_built_yet plus a bonus not-required `searched` rendering case (dead
  branch until Epic 7, rendered correctly anyway); the stale-record notice present/absent by `v-if`
  (not just an attribute check); and three `dispatch()`-based cases proving `aiPromptAssemble` (the
  mock) is never called across open, close, and three open/close cycles.

**Verification, every exact command and result:**
- `npm run build` — PASS (same two pre-existing `INEFFECTIVE_DYNAMIC_IMPORT` warnings as Phase 3, 0
  new).
- `cd src-tauri && cargo test --test ai_prompt_contract --locked` — **15 passed / 0 failed / 0
  ignored**, 1.30s (first real run, no fixture adjustment needed — every hand-derived expectation,
  including the two-term happy-path prompt string and the pending-overlap span, matched the real
  pipeline's output on the first try).
- Re-ran the Phase 1 positive control for real: `cargo test --test ai_boundary --locked -- --ignored
  --exact deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling` →
  **`ok. 1 passed; 0 failed`**, 15.67s. **This PASS is NOT the fresh measurement Phase 1 asked for —
  see the dedicated finding below and `deferred-work.md` item ③.** It is the same no-op scenario Phase
  1 already measured, not a genuine re-run against the real `commands/aiprompt.rs`/`lib.rs` seam lines,
  because the test's `git worktree add --detach <dir> HEAD` checks out the last COMMIT
  (`6de227d`), and this story's Phase 2/3 files are uncommitted. Confirmed directly:
  `git show HEAD:src-tauri/src/commands/aiprompt.rs` → `fatal: path … exists on disk, but not in
  'HEAD'`; `git show HEAD:src-tauri/src/lib.rs | grep -c "commands::aiprompt"` → `0`. Read Phase 1's
  own honesty note the right way round: "a no-op today must not be read as verified" — that sentence is
  STILL true after Phase 2/3 wrote real code, because "exists on disk" and "exists at the ref this test
  checks out" are two different facts, and I nearly reported the PASS as the closed counter-check
  before checking which one it actually was measuring.
- `cd src-tauri && cargo test --locked` (measured TWICE, full un-truncated output piped to a file and
  grepped for `^test result:` — not `tail`ed inside the measuring command itself, per the "a cut
  measurement corrupts the baseline" pitfall) — **1719 passed / 0 failed / 21 ignored across 64
  `test result:` lines**, both times, before and after the two counter-checks below (restored). Compares
  against Phase 2's baseline (1704/0/21, 63 lines) as exactly **+15 passed, +1 binary, 0 change to
  ignored** — the population rose by precisely this phase's new contract-test case count and nothing
  else.
- `npx vitest run` — **84 test files / 1212 tests, 0 red** (twice, before and after the lint fix below).
  Compares against Phase 3's baseline (83/1189) as exactly **+1 file, +23 tests** — this phase's new
  frontend file and nothing else.
- `npm run check:debt-owner` — **PASS**: `0/542 mục mở thiếu Chủ:` (814 total, 92 half, 169 closed) —
  the three new items below are counted and all carry a real `**Chủ: …**`.

**Counter-check (a) — the seam by removal, both halves, exactly as the spec's Verification section
asks, "a compile error is not a discharge":**
1. Commented out the `if is_the_approved_ai_prompt_command_file(rel) { continue; }` branch in
   `no_file_outside_core_ai_names_a_bare_dependency_on_the_ai_module` (`ai_boundary.rs`) — re-ran
   `cargo test --test ai_boundary --locked -- --exact
   no_file_outside_core_ai_names_a_bare_dependency_on_the_ai_module` → **FAILED**, naming exactly
   `commands/aiprompt.rs:27  crate::core::ai  |  use crate::core::ai::rag::{`. Restored the branch
   (`cp` from a backup taken before the edit), re-ran the same command → **ok. 1 passed**. `diff`
   against the backup after restore: identical.
2. Replaced the real `let (prompt, ledger) = assemble_prompt(&set.body, sentence, glossary, None);`
   line in `commands/aiprompt.rs` with a fixed stub tuple (`"STUB -- khong goi
   core::ai::rag::assemble_prompt"` plus an empty `NotAsked`/`NotBuiltYet` ledger) — re-ran
   `cargo test --test ai_prompt_contract --locked -- --exact
   happy_path_records_two_confirmed_terms_with_their_translation_and_tier` → **FAILED**, naming exactly
   `left: "STUB -- khong goi core::ai::rag::assemble_prompt"` vs `right: "Terms: \ndragon → rong\ncastle
   → lau dai\nSentence: A dragon guards the castle."` — the missing/wrong prompt content, by name.
   Restored the real line from a backup, re-ran `cargo test --test ai_prompt_contract --locked` (full
   file) → **15 passed / 0 failed**. `diff` against the backup after restore: identical.

**Counter-check (b) — the record by mutating one field, "a case that passes on both lists guards
neither":**
- Mutated `AiPromptInspectorOverlay.vue`'s injected-row template to hard-code `tierLabel('global')`
  instead of `tierLabel(term.tier)` (dropping the real `tier` field for injected rows only, leaving the
  suppressed rows' binding untouched) — re-ran `npx vitest run tests/frontend/aiPromptInspector.test.ts`
  → **1 failed / 22 passed**, naming exactly the case *"kind 'asked' với hai thuật ngữ ⇒ hai dòng,
  ĐÚNG source_term/translation/tier từng dòng"*, with `AssertionError: expected 'Toàn cục' to be 'Tác
  phẩm này'` — the mutated field, by its rendered value. Restored the file from a backup taken before
  the edit; `diff` against the backup after restore: identical; re-ran the same command →
  **23 passed / 0 failed**.

**A genuine test-design finding surfaced by this phase, not a production bug — recorded honestly
instead of reported as a closed counter-check:** the Phase 1 positive control
(`deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling`) cannot be trusted
to reflect this story's real seam files until this story is **committed**, because it operates on
`git worktree add --detach <dir> HEAD` — a commit ref, not the working tree. Every other verification
in this story (the full `cargo test --locked`, `npx vitest run`, all eleven gates) reads the real
working tree directly and is unaffected by this; only this one `#[ignore]`d, git-worktree-based test is
blind to uncommitted files. Filed as `deferred-work.md` item ③ below, Chủ: Ice (whoever commits this
story) — the real missing measurement is re-running the exact same command once more, right after the
commit that adds `commands/aiprompt.rs` and the `lib.rs` seam lines to `HEAD`.

**One lint fix inside this phase's own new test file (not Phase 1/2/3's code):** the first draft of
`aiPromptInspector.test.ts` used `const row = rows[0]; if (row === undefined) throw …` as a type guard
before `.get(...)` calls on it. `npm run check:lint` flagged this as
`@typescript-eslint/no-unnecessary-condition` — `noUncheckedIndexedAccess` is off in this project's
`tsconfig.json`, so TypeScript already types `rows[0]` as non-optional and the guard is genuinely dead
code, not a false positive. Removed the guard (the two other rows-array accesses elsewhere in the file
already used `?.` for the SAME reason other cases index into a two-element array — this one indexed a
one-element array right after `expect(rows).toHaveLength(1)`, so no optional access was needed at all).
Re-ran `npx vitest run tests/frontend/aiPromptInspector.test.ts` after the fix → still 23/23 green.

**Gate results, all eleven, run individually, exact:**
- `check:deps` — PASS (cây Rust 337 crate, cây npm 522 gói, cả hai sạch).
- `check:tokens` — PASS (105 tệp / 101 component, 3631 khai báo CSS quét, 8 deviation đã ký).
- `check:i18n` — PASS (755 text node / 30 `.vue`, 176 miễn trừ; 963 khoá `vi.json`, 193 placeholder).
- `check:commands` — PASS (166 command, 122 `@click`, 176 `dispatch()`, 30 `.vue`+72 `.ts` — unchanged
  from Phase 3's baseline, this phase added no command; `ai.prompt_inspector.open`/`.close`/
  `ai.prompt.assemble` all confirmed in the live `unbound()` listing).
- `check:layout` — PASS (102 tệp, 17 thành viên `window`/`document`, tất cả trong allow-list).
- `check:panel-refs` — PASS (72 tệp `.ts`, 389 ô nhớ cấp module, 31 miễn trừ — unchanged from Phase 3).
- `check:dict` — PASS (mọi Kiểm A–F đạt, không liên quan tới story này, chạy để xác nhận không hồi quy).
- `check:dict-manifest` — PASS (3 detachable, mọi hash/url/source_version đúng hình dạng).
- `check:lint` — PASS (after the one fix above; `eslint src e2e tests`, 0 lỗi).
- `check:gates` — PASS (13 script `check:*`, 16 lời gọi `npm run` trong `ci.yml`, 11 cổng `pre-push`,
  ba danh sách khớp nhau).
- `check:debt-owner` — PASS (0/542 mục mở thiếu `Chủ:`, includes this phase's three new items).

**CI — explicitly NOT read this phase, and this matters:** `pre-push` runs only on Ice's macOS; CI runs
both platforms every push. This session has no path to GitHub Actions. Per the story's own Acceptance
Criteria and root `AGENTS.md`'s "read the latest `schedule`/CI run before writing `done`," the story
**cannot honestly be called `done`** until the CI run for whatever commit lands this story is read —
this is flagged for Ice below and the frontmatter `status:` is left `in-progress` on that basis alone,
independent of the two ACs left unchecked above.

**`deferred-work.md` — the three items appended, exact content (verbatim, see the file itself for the
full `evidence:`/`summary:` prose):**
1. AC1's *"prompt cuối cùng đã **gửi**"* stays unfulfilled — the record never carries a sent-state in
   this story. **Chủ: Story 4.8.**
2. The mockup's fix-it-now action per suppressed row is unbuilt (Decision 3) — read-only confirmed by
   this phase's own frontend test (`row.findAll('button')`/`findAll('a')` both length 0). **Chủ: Story
   4.11.**
3. The seam's positive control still cannot exercise the real two-seam deletion — not because the files
   don't exist (they do, on disk), but because the test's `git worktree add --detach <dir> HEAD` reads
   the last COMMIT, and this story is uncommitted; re-run the exact same `--ignored --exact` command
   once more right after the commit that lands `commands/aiprompt.rs` + the `lib.rs` seam lines, and
   record that result — the PASS this phase measured is not that result. **Chủ: Ice** (whoever commits
   this story).

**Not touched, per Phase 4 scope:** no production Rust or frontend file left changed (both counter-check
mutations were made and reverted, confirmed byte-identical by `diff` against a pre-edit backup each
time); `ai_boundary.rs`/`config_invariants.rs`/`aiprompt.rs`/the three Phase 2/3 frontend files carry
only Phase 1–3's own pre-existing uncommitted changes, unchanged by this phase.

### 2026-09-18 — Post-Phase-4 closure (AC2 + AC3), before commit — done

Phase 4 left two Acceptance Criteria honestly unchecked rather than inferred. The build-review step
(§workflow: "finish the missing work before proceeding" for any unsatisfied AC) closed both for real,
without waiting on a commit or on Ice.

**AC2 — the seam's compile-based positive control (`deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling`).**
The root cause Phase 4 diagnosed was real but the fix it deferred to "after commit" was not the only
option: the control depended on `git worktree add --detach <dir> HEAD`, and `HEAD` is a commit, not this
story's working tree. Removed `git` from the test entirely — `copy_dir_recursive_skipping` (new, plain
`std::fs`, no shell-out, no `git`, portable to Windows) copies the actual `src-tauri/` working directory
into a scratch temp dir (skipping `target/`), and the rest of the test deletes/edits on that copy. This
is strictly stronger than "wait for a commit": it now measures the true state of the tree at any point,
committed or not.

Re-running immediately surfaced **two real, previously-unreachable bugs** — proof the fix actually
changed what the test could see, not just how it got its input:
1. `error: unexpected closing delimiter` in `lib.rs`. `lib_rs_without_the_approved_ai_prompt_seam` was
   written to delete exactly the lines matching the seam marker, which was correct for the two
   single-line shapes (`generate_handler!` entry, `app.manage(...)`) but wrong for the third shape Phase
   2's judgment call added: the multi-line `if let Some(record) = handle.try_state::<…>() { … }` block
   in `close_open_work`. Deleting only its opening line left an orphaned `}`. Fixed with a brace-balance
   tracker (`brace_delta`) that, when a marker line also opens an unclosed `{`, continues removing lines
   until the block's own `}` is reached — skipping the whole block, not just its first line. New
   counter-check unit test:
   `the_lib_rs_stripper_removes_the_whole_multi_line_block_a_marker_line_opens` (dynamic-red confirmed:
   reverting the fix reproduces the exact brace-mismatch panic).
2. `error[E0583]: file not found for module aiprompt` in `commands/mod.rs`. Deleting
   `commands/aiprompt.rs` alone left its `pub mod aiprompt;` declaration dangling in the parent module —
   the identical class of bug the existing `core_mod_rs_without_the_ai_declaration`/`pub mod ai;`
   handling already guarded against for the FIRST seam, simply never extended to the second. Fixed by
   generalizing that stripper into `mod_rs_without_declaration(text, module_name)` (the old function is
   now a one-line wrapper, same name, so its own existing unit test needed no change) and calling it a
   second time on `commands/mod.rs` with `"aiprompt"`, guarded so it only runs when the seam file
   actually existed to delete.

After both fixes, the control **passes for real** against the uncommitted tree:
`cargo test --test ai_boundary --locked -- --ignored --exact
deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling` → `ok. 1 passed; 0
failed`, ~10s. The full `ai_boundary.rs` file (now 18 tests, two new) is green:
`cargo test --test ai_boundary --locked` → `16 passed; 0 failed; 1 ignored` (the 17th, the
`#[ignore]`d compile control, run separately above). `deferred-work.md`'s item ③ ("Đối chứng dương của
seam … KHÔNG THỂ nghiệm thu") is marked closed with the full evidence trail — it is no longer a debt
handed to whoever commits this story; the AC is genuinely satisfied now, pre-commit.

**AC3 — rebind through the shortcut layer.** Before this entry, `applyBindings` (the runtime API
`src/config/shortcutsState.ts`/`ShortcutsOverlay.vue` use to apply a user's rebind immediately, Story
1.21 AC2/AC12) had zero test coverage anywhere in `tests/frontend/`, for any command — confirmed by
`grep -rln "applyBindings(" tests/frontend/*.test.ts` returning nothing, so Phase 4's structural argument
("every registered command's `run()` is the only code either trigger path reaches") was not backed by a
standing test for this or any other command. Since `ai.prompt_inspector.open`/`.close`/
`ai.prompt.assemble` register with `keys: undefined` (no default chord), the only way to exercise the
keyboard path for them is a genuine rebind, not a default-key press (the precedent this codebase already
had, `editorClearSourceCuts.test.ts`, only covers a command's *default* chord).

Added `describe('AC3 — rebind qua tầng phím tắt …')` to `tests/frontend/aiPromptInspector.test.ts`: two
cases call the real `commands.applyBindings({ 'ai.prompt_inspector.open': ['Shift+K'] })`, attach the
real `commands.attachKeyboard(host)` listener, dispatch a genuine `KeyboardEvent` from that host, and
assert the overlay opens (`aiPromptInspectorIsOpen === true`), `ai_prompt_read_record` is called exactly
once, and `aiPromptAssemble` is never called — the same Decision-2 invariant the dispatch-by-id tests
already guard, now proven to hold on the keyboard path too. `Shift+K` (no `Mod`) was chosen deliberately
to avoid `detectIsMac()`'s platform branch entirely — this file does not need to know which platform the
test runner reports itself as. Counter-checked red: pointed the `applyBindings` call at the wrong command
id (`ai.prompt_inspector.close` instead of `.open`) — both new cases failed, naming the overlay staying
closed; reverted, `npx vitest run tests/frontend/aiPromptInspector.test.ts` → 25/25 green.

**A genuine pre-existing defect this closure found and fixed, unrelated to AC2/AC3 — `npm run build` was
never actually green.** Phase 4 reported "`npm run build` — PASS" but `tests/frontend/aiPromptInspector.test.ts:254`
(Phase 4's own file, line unchanged by this closure until now) called `wrapper.get(selector).exists()` —
`@vue/test-utils`'s types define `get()`'s return as `Omit<DOMWrapper<Element>, "exists">` (it throws
instead of needing `.exists()`, unlike `.find()`), so `vue-tsc` fails this file with `TS2339`.
`npx vitest run` never caught it because vitest transforms with esbuild, not `tsc` — the spec's own
`## Verification` lists `npm run build` as the FIRST command, specifically because `cargo test` breaks at
compile time without `dist/`, and `vue-tsc` type errors are exactly the class of failure a
transform-only test run cannot see. Fixed by changing that one call to `wrapper.find(...)` (semantically
equivalent for this assertion — either selector existing is exactly what `.find().exists()` reports).
`npm run build` now passes clean: `vue-tsc` both configs, `vite build`, only the same two pre-existing
`INEFFECTIVE_DYNAMIC_IMPORT` warnings Phase 3/4 already named. Re-ran
`npx vitest run tests/frontend/aiPromptInspector.test.ts` after the fix — still 25/25 green (the fix
changed only which `@vue/test-utils` method is called, not the assertion's runtime meaning).

**Verification, this closure, measured fresh:**
- `cargo test --test ai_boundary --locked` — 16 passed / 0 failed / 1 ignored (was 15/0/1 before this
  closure — one net new *enabled* test; the compile control itself stays `#[ignore]`d, run separately).
- `cargo test --test ai_boundary --locked -- --ignored --exact
  deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling` — `ok. 1 passed`,
  the first PASS in this story that is evidence about the real seam files, not a restated no-op.
- `npx vitest run` — 84 files / **1214** tests, 0 red (was 1212 after Phase 4; +2 for the AC3 cases).
- `cd src-tauri && cargo test --locked` (full suite, `time`d, full un-truncated output piped to a file
  and grepped — not `tail`ed inside the measuring command) — **1720 passed / 0 failed / 21 ignored**,
  across the same **64** `test result:` lines as Phase 4 (no binary added or removed by this closure).
  Exactly **+1** passed against Phase 4's 1719/0/21 baseline — this closure added exactly one new Rust
  unit test (`the_lib_rs_stripper_removes_the_whole_multi_line_block_a_marker_line_opens`); the AC2 fix
  itself only changed the body of an existing `#[ignore]`d test and two existing helper functions, and
  ignored-test counts and their own pass/fail are excluded from `passed`/`failed` by `cargo test`'s own
  accounting regardless.

**Debt this closure removes, not adds:** `deferred-work.md`'s item ③ is now closed (see its own
`→ ✅ ĐÃ ĐÓNG` follow-up), so the story hands on two owned debts (AC1's sent-state to Story 4.8, the
fix-it-now action to Story 4.11), not three.

### Independent verification by the orchestrating session (2026-09-18)

Measured here, not accepted from the implementation agent's report. The agent stopped twice
reporting only "waiting for the test run" with no figures, so the measuring was taken back.

- `cargo test --locked` — read from the run's own full log, not a tail: **1722 passed / 0 failed /
  21 ignored**, 64 `test result:` lines = **62 binaries** (the last two lines are the doc-test run
  and its `compile fail` companion). 0 `error`/`FAILED`/`panicked` markers. The 2 warnings are
  pre-existing in `core/aiconfig/keychain.rs` (Story 4.3) and are not in this diff.
  `ai_prompt_contract`: 17 passed / 0 failed. `ai_boundary`: 16 passed / 0 failed / 1 ignored.
- `npx vitest run` — **84 files / 1221 tests**, exit 0 (baseline 83 / 1189 ⇒ +1 file, +32 cases).
- The eleven `pre-push` gates run individually — all **PASS**.
- The FR77 positive control is `#[ignore]`d, so the full suite does NOT run it and its AC could
  not be accepted from a green suite. Run by hand here:
  `cargo test --test ai_boundary --locked -- --ignored --exact deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling`
  → **ok, 11.91s**.
- **Removal counter-check on the seam, a real deletion.** Backed the gate file up by copy first
  (it is tracked but uncommitted — `git checkout` would have restored the baseline and destroyed
  the story's 453 new lines), deleted the `is_the_approved_ai_prompt_command_file` branch, and ran
  the gate: **red for the right reason**, naming `commands/aiprompt.rs:27  crate::core::ai  |  use
  crate::core::ai::rag::{` — a named violation, not a compile error. Restored from the copy and
  verified by SHA-256, not by eye: `f9a1da29…9e8f` before and after; gate green again.
- **Removal counter-check on the screen, aimed at the branch most likely to be unguarded.**
  Dropped `tier` from the **suppressed** row only (an assert holding on both lists would guard
  neither): exactly 1 case went red, naming `.aip-term-tier` at `aiPromptInspector.test.ts:382`.
  Restored and verified by SHA-256 (`174bf21d…1de5`); 32/32 green.
- The seam is genuinely needed: `commands/aiprompt.rs:27` really does write
  `use crate::core::ai::rag::{…}`. But `lib.rs` names `core::ai` **zero** times, so the second
  half of Decision 1's exemption cannot fire — recorded as an owned debt rather than left in a
  doc-comment.
- Matrix audit: all 12 rows map to a named contract case that ran and passed; rows 8 and 9 have
  two and three cases respectively, including one asserting the two preconditions carry
  **distinct** keys.

### 2026-09-18 — Phase 5 (loop 1: B1 and the 27 carried findings) — done

All twelve Phase 5 task rows closed. Fresh agent, no memory of Phases 1–4 beyond this file.

**Finding B1 (the loop-triggering defect) — closed.** `core::ai::rag::InjectionLedger` gained an
additive `pieces: Vec<PromptPiece>` field (`PromptPiece { kind: PromptPieceKind, text: String }`,
`PromptPieceKind::{Authored, Glossary, Tm}`). `expand_prompt_body` was rewritten to build `pieces`
**in lockstep** with `out` (the same single left-to-right scan spec 4.6 already ran — no second
scanner) via two small helpers, `push_piece`/`pop_piece`, that mirror every `out.push_str`/`out.pop`
call site. `assemble_prompt` carries a `debug_assert_eq!` that joining `pieces` reproduces `prompt`
byte for byte (release-mode free; the real assert lives in the test suite, per the frozen §Always
guard clause). Wire: `commands/aiprompt.rs` gained `PromptPieceKindWire`/`PromptPieceWire` mirrors
and `InjectionLedgerWire.pieces`; `src/config/aiprompt.ts` gained the matching TS types + runtime
guards; `AiPromptInspectorOverlay.vue`'s `<pre>` now renders one `<span>` per piece
(`:class="['aip-piece', 'aip-piece-' + piece.kind]"`, `data-aip-piece-kind`) instead of one
undifferentiated text node — `.aip-piece-glossary` carries the background that visually separates
injected Glossary text from the authored body (§Always's actual clause; the CSS comment that used
to (mis)claim this on `.aip-term-list-injected` now says what that rule really does, and the real
rule moved). Counter-checked red twice: (1) Rust — mapped `GlossaryTerms` to `PromptPieceKind::
Authored`, `ai_prompt_contract.rs`'s new piece assertions failed naming the collapsed single piece
(`left: 1, right: 3`), restored, verified byte-identical, green; (2) `ai_boundary`'s bare-token gate
with the per-file exemption commented out, red naming `commands/aiprompt.rs:27` (unrelated to B1
but re-run as part of this phase's baseline, see below).

**V1 (record survives a Work swap) — closed.** `commands/project/mod.rs::replace_open_work` now
clears `LastAssembledPromptState` beside its two siblings (`PendingImportState`/
`PendingPromptImportState`), reusing `commands::aiprompt::clear_last_assembled_prompt_on_work_close`
(Phase 2's already-`pub`, already-unit-tested function — no new clearing logic, just a new call
site). No `tauri::test`/`MockRuntime` in this crate, so `replace_open_work` itself (takes
`&tauri::AppHandle`) cannot be called from `tests/*.rs` directly — the counter-check instead reads
`src/commands/project/mod.rs`, cuts the exact `fn replace_open_work` body (anchored on the next
function's doc-comment as the end marker, with a positive check that the two pre-existing sibling
calls are inside the same cut — guards against the cut being mis-anchored), and asserts the new
call is textually inside it (`project_contract.rs::
replace_open_work_clears_the_last_assembled_prompt_record_beside_its_two_siblings`). Counter-checked
red by deleting the new block, confirmed the case failed naming the missing call, restored.

**A second, genuine defect V1 introduced and this phase also had to close, not part of the original
27 findings:** adding a real `crate::commands::aiprompt::…` reference to `commands/project/mod.rs`
gave seam ① a **second** real call site (`lib.rs` was the only one Phase 1–4 knew about). Re-running
the FR77 compile-based control after the V1 edit failed for real —
`error[E0433]: cannot find aiprompt in commands` at `commands/project/mod.rs`, the exact class of
bug the same control already caught once for `lib.rs`/`commands/mod.rs`, now recurring at a THIRD
call site nobody had generalized the stripper for. Fixed by generalizing
`lib_rs_without_the_approved_ai_prompt_seam` into a file-agnostic
`text_without_lines_matching_the_ai_prompt_seam_marker` (same brace-aware block-skip + the E7
balance assert), with `lib_rs_without_the_approved_ai_prompt_seam`/
`project_mod_rs_without_the_ai_prompt_seam` as one-line wrappers (kept the old name so existing unit
tests didn't need to change), a new unit test on hand-built text mirroring the real three-line
shape (`the_project_mod_rs_stripper_removes_the_whole_multi_line_block_the_v1_clearing_branch_opens`),
and wiring the second file into the compile control. Re-ran the real `#[ignore]`d test after the
fix: `cargo test --test ai_boundary --locked -- --ignored --exact
deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling` → `ok. 1 passed`,
19.07s — this is the first PASS of this control that is evidence about the tree **as it stands at
the end of Phase 5**, not Phase 4's snapshot. Read the lesson the way `deferred-work.md`'s item ③
already reads it: a PASS on this control only means what it was actually able to see: it just proved
it can see a third call site it could not see minutes earlier.

**V2 (wires unregistered, nothing goes red) — closed.** `ipc_contract.rs` gained
`the_ai_prompt_wires_are_registered_and_keep_their_parameter_names`, same two-part shape as every
sibling in that file: `generate_handler!`/`app.manage` line presence, plus `fn_param_list` anchored
on the real `pub fn ai_prompt_assemble`/`ai_prompt_read_record` blocks inside `commands/aiprompt.rs`
(safe against the "two blocks, same name" trap this file's own helper test warns about, because the
pure functions are named differently on purpose — `assemble_and_record_prompt`/
`read_last_assembled_prompt` — precisely so `wire::ai_prompt_assemble` is the only match).

**V3 (FR77 control runs nowhere automatic) — closed.** Added a CI step in `.github/workflows/ci.yml`
right after the main `cargo test` step, reusing its `target/` (same reasoning the ignored test's own
doc-comment already gives for reusing the real crate's `target/` — third-party deps stay cached,
only the local crate recompiles). Confirmed `npm run check:gates` still passes (this step is a raw
`cargo test` invocation, not an `npm run check:*` script, so it is outside that gate's three-list
scope by design — same as the main `cargo test` step already sitting next to it).

**V4/B8/E7 (the whole-file exemption, the asymmetric seam-② deletion, the un-asserted brace balance)
— closed together, same file.** Decision 1's control ② ("nothing beyond the two AD-14 functions")
was prose until this phase: `is_the_approved_ai_prompt_command_file` used to `continue` the ENTIRE
file past the bare-token gate. Replaced with a **per-line** exemption
(`line_is_the_approved_ai_prompt_import_in_command_file`, matching only lines containing
`crate::core::ai::rag::`) plus a **named-allowlist** companion gate
(`commands_aiprompt_rs_names_nothing_beyond_the_allowed_ai_rag_surface`, mirroring
`ALLOWED_GLOSSARY_NAMES_UNDER_AI`'s shape exactly — same multi-line-`use`-group-aware scanner
pattern, `ai_rag_names_named`) checking the nine names Decision 1 actually allows (the two AD-14
functions plus the seven mirror-type source types the wire layer must name to write `From<…>`).
Counter-checked for real: with the per-line exemption's `continue` short-circuited via `if false &&`,
the main gate went red naming **exactly one** violation — `commands/aiprompt.rs:27`, the real `use`
line — and **nothing else** in the ~450-line file, proving the rest of the file is now genuinely
scanned (0 false negatives) where before nothing was scanned at all. B8: the compile control's
seam-② deletion is no longer `if aiprompt_path.exists() { … }` — it now `assert!`s existence first,
symmetric with seam ①'s existing hard assert (a future rename to e.g. `commands/aiprompt/mod.rs`
would previously have silently reverted this control to the exact no-op this story spent an entire
prior closure removing). E7: `text_without_lines_matching_the_ai_prompt_seam_marker` now
`assert_eq!(skipping_block_depth, 0, …)` at the end — a marker line that opens a block that never
closes (stray `{` in a string/comment, or unbalanced input) would otherwise silently truncate the
rest of the text and read as "every other seam vanished," not as "this function is broken."

**B4/E2/E3/E4/E10 (collapsing read/assemble failures into `null`) — closed.** `config/aiprompt.ts
::aiPromptReadRecord` now returns `{ value, error }` (previously bare `T | null` — this command has
no `Result` on the Rust side, so there was no existing convention to preserve; `error: null` now
means "confirmed" including the legitimate `value: null` "nothing recorded yet" case, `error !==
null` means "this read is not trustworthy," including the no-bridge branch, which used to collapse
into the same `null` as a real empty record). `aiPromptInspectorState.ts::refreshAiPromptRecord`
only writes `record` when `error === null`; on failure it keeps the existing record and writes a new
`readError` ref (exported as `aiPromptReadError`, surfaced on the overlay as `.aip-read-error`, wired
into `resetAiPromptInspector`). `assembleCurrentAiPrompt` reordered its error/overtake checks
(E4: check `result.error` before the global `sequence`-overtake return, gated on the existing
`stillLatestAssemble`/`latestAssembleSequence` flag rather than `sequence` itself, so a plain READ
racing in cannot swallow a real assemble failure) and added an explicit `result.value === null`
early-return (E2: the adapter's own documented "no Tauri bridge" shape for a `Result`-returning
command is `{ value: null, error: null }` — reading `value === null` *after* the error branch is
exactly that case, and it must not overwrite a good record with `null`). Four new counter-checked
Vitest cases (`findings B4/E3/E10`, `finding E2`, `finding E4`) each seed the exact regression,
confirm red naming it, restore, confirm green.

**B6/E5/E6 (stale summary, segment identity on error) — closed.** E5/E6 were already closed going
into this phase (Phase 4's own build-review pass added `clearAiPromptAssembleError` + a
`watch(editorCaretSegmentId, …)` in `AiTranslationPanel.vue`). B6 was open: the panel's
always-visible summary line had no idea the record it summarizes might belong to a different segment
than the one focused now — only the overlay knew, via `aiPromptRecordIsStale`. Panel now computes the
same pure function against the same `editorCaretSegmentId` ref it already reads for `canAssemble`,
marks `.ai-inspector-summary[data-ai-prompt-summary-stale="true"]`, and renders a second line reusing
the overlay's existing `ai.prompt_inspector.stale_notice` key (no new i18n key minted for the same
fact). Counter-checked red by hard-coding the computed to `false`.

**B2/V5/B3/E11/E8/B14 (test-file gaps) — closed, six cases/fixes added or corrected in
`tests/frontend/aiPromptInspector.test.ts`.** B2: `unknown_markers`/`source_segment_missing` now
have mounted-overlay cases overriding both away from their fixture defaults (plus a "both clean"
negative case). V5: `.ai-inspector-summary` now has its own mounted-panel cases across all three
kinds (`no_record`/`not_asked`/`asked`-with-count), distinct from both the pure-function test and the
overlay's own summary element. B3: the case titled "KHÔNG ghi đè aiPromptRecord" now actually
populates a record before the failing Lắp and asserts it survives — counter-checked red by
temporarily making the error branch null the record, confirmed the (correctly-named) case now
catches exactly that. E11: a new case asserts `effectiveUnbound()` (the runtime function
`index.ts`'s own doc-comment names as the one the rebind UI must read) drops
`ai.prompt_inspector.open` after `applyBindings`, while the two still-unbound sibling commands stay
listed — the AC3 clause the earlier rebind cases exercised the mechanism for but never asserted
directly. E8: `vi.mock('../../src/config/promptset', …)` now stubs all six functions
`promptSetState.ts` imports from it (`promptSetCreate`/`Delete`/`Export`/`List`/`Rename`/
`UpdateBody`), not just `promptSetList` — the other five were `undefined`, inert only because no
case in this file happened to exercise them yet. B14: the reset-describe's "five cells / four
values" wording is now "six cells / five values" (post-`readError` count), and the case itself now
also seeds and asserts `aiPromptReadError` — counter-checked red by removing `readError.value = null`
from `resetAiPromptInspector`.

**B15 (no accessible name on the dialog) — closed.** `.aip-panel` gained `aria-labelledby="aip-title"`
pointing at a new `id="aip-title"` on the existing `<h2>` — one attribute, no new element, no i18n
key. Left the sibling overlays' identical gap alone (out of this story's scope, tracked below) and
said so in a comment at the point of the fix, so this file does not read as "the pattern is fine
here, unexplained why not there."

**B7 (`GlossaryTierWire` — third wire spelling, no agreement test) — closed.**
`glossary_contract.rs::category_and_glossary_tier_wire_strings_agree_between_as_str_and_serde_rename`
now also serializes `GlossaryTierWire::from(tier)` for each `GlossaryTier` variant and compares
against the same `as_str()`-derived quoted string the existing two-spelling check already computes —
the test exists specifically because two spellings drifted once before; this closes the same class
of gap for the third.

**deferred-work.md fixes (B9/B10/B11/B12/V6) — closed.** B10: the "Hai mục"/"two items" preamble on
the `bmad-build review — spec 4-7` section now reads "Ba mục"/"three items" (it always listed
three — `is_omitted`, the a11y gap, the dead `lib.rs` exemption half — the preamble text was stale
from before the third was appended). B11: the a11y item's owner ("một story a11y tương lai") did not
name anything `check-debt-owner.mjs` can read as a real story or person — it only passed the gate
because `NEGATIVE_OWNER_RE` does not know that specific phrasing, not because it is a real owner.
Reassigned to **Ice**, with the decision content (fix all four overlays at once, when opened)
preserved intact. B9 and V6 (both genuine debts 4.7 itself created, not out-of-scope exclusions —
unlike the three items in the `bmad-build review` section) got their own new ledger entries in the
FIRST section instead: B9 names the shared-`target/` build-lock risk the FR77 control's design
accepts, owner Ice, contingent on evidence from the new CI step (V3); V6 names the twelfth,
untested `isBlocked` disjunct in `main.ts` (`aiPromptInspectorIsOpen`), owner Ice, explicitly scoped
as "fix all twelve at once, not just the newest," matching this story's own norm for not vá-ing one
spot while leaving identical spots untouched. B12: AC5's "CI must be read" half was previously only
prose inside Implementation Notes, invisible to anyone reading only the ledger — now has its own
entry, owner Ice, self-closing instruction ("read CI; if green, delete this row").

**E9 (Phase 2 task text overclaimed) — closed.** The Phase-2 task bullet claimed three new message
keys; corrected in place to say two were added and the third slot correctly reuses the pre-existing
`MessageKey::WorkNoneOpen` (the code was already right — Phase 2's own Implementation Notes said so
at the time — only the earlier task-list bullet's wording was stale).

**Verification, this phase, measured fresh, full runs (not a subset):**
- `npm run build` — PASS, same two pre-existing `INEFFECTIVE_DYNAMIC_IMPORT` warnings, 0 new.
- `cd src-tauri && cargo test --locked` (full, untruncated, piped to a file and grepped for
  `^test result:`, not `tail`ed inside the measuring command) — **1727 passed / 0 failed / 21
  ignored**, 64 `test result:` lines — +5 against the "Independent verification" baseline (1722/0/21,
  64 lines): `commands_aiprompt_rs_names_nothing_beyond_the_allowed_ai_rag_surface`,
  `ai_rag_names_named_collects_a_multiline_use_group_and_a_seeded_forbidden_name`,
  `the_project_mod_rs_stripper_removes_the_whole_multi_line_block_the_v1_clearing_branch_opens`
  (all three in `ai_boundary.rs`),
  `replace_open_work_clears_the_last_assembled_prompt_record_beside_its_two_siblings`
  (`project_contract.rs`), `the_ai_prompt_wires_are_registered_and_keep_their_parameter_names`
  (`ipc_contract.rs`) — five new `#[test]` functions, nothing else changed the population.
- `cargo test --test ai_boundary --locked -- --ignored --exact
  deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling` — **`ok. 1
  passed`**, 19.07s, run AFTER the V1-introduced-and-fixed gap above — the first PASS of this story
  that is evidence about the tree as Phase 5 leaves it.
- `npx vitest run` — **84 files / 1236 tests, 0 red** — +15 against the baseline (1221) — all new
  cases named above.
- All eleven `pre-push`-equivalent gates run individually, fresh: `check:deps` · `check:tokens` ·
  `check:i18n` · `check:commands` · `check:layout` · `check:panel-refs` (391 module cells, 31
  exemptions — the new `readError` cell needed none, reachable via `resetAiPromptInspector`) ·
  `check:dict` · `check:dict-manifest` · `check:lint` · `check:gates` · `check:debt-owner` (0/547
  open items missing `Chủ:` — 547 is +3 against the prior 544, the B9/V6/B12 entries) — **all PASS**.
- CI — **still not read this phase** (this session has no path to GitHub Actions). AC5's second
  clause stays open; `deferred-work.md`'s new B12 entry is the standing ledger record of that, and
  `status:` stays `in-progress` on that basis, independent of everything else in this phase being
  closed.

**Not touched, per Phase 5 scope:** Story 4.8's send path, Story 4.11's fix-it-now action, the other
three overlays' shared a11y gap (B15's sibling debt), `is_omitted`/FR133 (owned by 4.8), the dead half
of Decision 1's `lib.rs` line exemption (owned by 4.8, unless it never writes a bare `core::ai` line
either, in which case the future closer deletes it).

### Independent verification of loop 1 by the orchestrating session (2026-09-18)

Measured here, not accepted from the implementation agent's report.

- `cargo test --locked` — **1727 passed / 0 failed / 21 ignored**, 64 `test result:` lines (62
  binaries), 0 `error`/`FAILED`/`panicked` markers, exit 0. `npx vitest run` — **84 files / 1236
  tests**, exit 0. Both match the agent's figures exactly.
- The eleven gates run individually — all **PASS**. The FR77 control run by hand — **ok, 18.71s**;
  it now also has a CI step (`.github/workflows/ci.yml:329-330`), so it is no longer guarded only by
  a habit.
- **Counter-check on B1's screen half, a real revert to the defect.** Put the prompt body back to the
  single undifferentiated `<pre>{{ prompt }}</pre>` of loop 0: exactly 1 case went red, naming
  `data-aip-piece-kind`. Restored and verified by SHA-256 (`b6d518507fdbae82`).
- **Counter-check on B1's Rust half, aimed past the obvious guard.** The concatenation assert alone
  would pass a mislabelled piece, so the mutation changed the KIND, not the text: the injected block
  was pushed as `Authored` with the byte-for-byte concatenation still identical. That made
  `happy_path_records_two_confirmed_terms_with_their_translation_and_tier` red on the Glossary-kind
  text. Restored and verified byte-identical against the backup copy.
- Spot-verified in place: V1 (`commands/project/mod.rs:5246-5247` clears on Work swap), V2
  (`ipc_contract.rs:1895`), V3 (the CI step), V4 (the whole-file skip is now line-scoped plus an
  allowlist scan), B2 (cases that override `unknown_markers` and `source_segment_missing` and assert
  BOTH branches, not one).
- Phase 5: 28 tasks/ACs `[x]`, 1 `[ ]` — AC5's CI clause, which cannot close before a push exists.

### 2026-09-18 — Phase 6 (loop 2: the closed list) — done, one AC clause left honestly unchecked

All nine Phase 6 task rows closed, each accepted on a real removal counter-check (not on the
implementation's own report — every counter-check below was run, went red naming the exact
defect, then was restored and re-verified green).

**Files touched:**
- `src-tauri/src/core/ai/rag.rs` — new `PromptPieceKind::SourceSegment` variant;
  `piece_kind_for(PromptVariable::SourceSegment)` now maps to it instead of `Authored` (finding
  P7). Doc-comments corrected in place, including `PromptPieceKind`'s own and `piece_kind_for`'s.
- `src-tauri/src/commands/aiprompt.rs` — mirror `PromptPieceKindWire::SourceSegment` +
  `#[serde(rename_all = "snake_case")]` already covers it (serializes `"source_segment"`);
  `From<PromptPieceKind>` match extended.
- `src-tauri/tests/ai_prompt_contract.rs` — happy-path test's piece assertions updated for the
  new 4-piece split (finding P7 fallout); two new tests: a `serde_json::to_value` pin on the
  real wire returned by `assemble_and_record_prompt` (top-level + `ledger` + `glossary`/`tm`/
  `pieces` field-name sets, and the exact `"authored"`/`"glossary"`/`"source_segment"` tag
  strings this call site can produce) and a standalone pin of `PromptPieceKindWire::Tm` → `"tm"`
  (finding P3 — the call site can never produce a `Tm` piece with real data, so that one tag is
  pinned by direct construction, same shape the file already uses for wire-shape pins).
- `src-tauri/tests/ai_rag_contract.rs` — new test
  `pieces_tag_the_source_sentence_with_its_own_kind_separate_from_authored_and_tm_never_produces_a_piece`
  at the pure `assemble_prompt` layer: asserts the source-sentence piece is tagged
  `SourceSegment` (not `Authored`), and — reusing the exact fixture
  `tm_searched_records_the_real_segments_from_a_real_some_slice` already proved TM's marker
  replacement is unconditionally `""` — that even with `tm: Some(&segments)` carrying real
  content, `ledger.tm == Searched(segments)` while `pieces` contains zero `Tm`-kind entries
  (finding P9's other half: the *code* guarantee behind "no case can reach it," not just the
  screen's CSS rule).
- `src-tauri/tests/ai_boundary.rs` — finding P8: new helper `line_with_marker_occurrences_removed`
  (deletes only the approved marker substring from a line instead of `continue`-skipping the
  whole line) wired into both per-line exemption branches of
  `no_file_outside_core_ai_names_a_bare_dependency_on_the_ai_module`; new seeded-violation test
  `a_line_carrying_the_approved_prefix_is_still_scanned_for_a_second_forbidden_token` (positive +
  negative, both seams) proves a second forbidden token on the same physical line as an approved
  prefix is still caught.
- `src-tauri/tests/ipc_contract.rs` — finding P5: new test
  `close_open_work_clears_the_last_assembled_prompt_record_beside_its_two_siblings`, the twin of
  `project_contract.rs`'s `replace_open_work_…` case but anchored on `lib.rs::close_open_work`'s
  body (start marker the function signature, end marker the next function's doc-comment —
  verified unique in the file before use), asserting all three clearing calls (Story 3.10b,
  Story 4.5, Story 4.7) sit inside that one function body, not merely somewhere in `lib.rs`.
- `.github/workflows/ci.yml` — finding P1: the FR77 step now pipes its own output through `tee`
  and greps for the literal `running 1 test` line, failing the step with a named
  `::error::` if the filter ever matches zero tests (silently disarmed) instead of trusting
  `cargo test`'s own exit code, which is `0` even when the filter matches nothing. `shell: bash`
  added explicitly since this step runs on both `matrix.os` entries (macOS and Windows), and
  Windows runners resolve `bash` to Git for Windows's bash — same pattern the pre-existing macOS
  `.dmg` build step already uses for its own OS-scoped script.
- `src/config/aiprompt.ts` — `PromptPieceKindWire` union gains `'source_segment'`;
  `isPromptPieceKindWire` extended to match.
- `src/AiPromptInspectorOverlay.vue` — finding P9: two new CSS rules, `.aip-piece-source_segment`
  (`background: var(--color-surface-sunken); color: var(--color-on-surface-variant)` — the
  kho's existing "recessed quoted block" convention, same pair `ImportPreviewOverlay.vue` uses)
  and `.aip-piece-tm` (`background: var(--color-surface-tm)` — the kho's existing TM-surface
  convention, same token `ReadingMode.vue`/`AttributionOverlay.vue`/`LookupPanel.vue` already
  use for TM content). The `v-for` over `ledger.pieces` was already generic
  (`:class="['aip-piece', 'aip-piece-' + piece.kind]"`, Phase 5) — no template change needed,
  only the two missing rules.
- `src/modes/libraryChapters.ts` + `src/modes/libraryImport.ts` — finding P4: `resetAiPromptInspector()`
  added at BOTH real product points where the frontend's notion of "the open Work" changes —
  `openWorkById` (re-opening an existing `.atproj`) and `finishImportSubmission` (a freshly
  imported Work becoming the open one) — alongside the five/six sibling `reset*()` calls each
  function already makes for the same reason (Story 1.17/2.2/5.11's precedent). Two call sites,
  not one: `finishImportSubmission` does NOT route through `openWorkById` (each duplicates its
  own copy of the six-reset block, per that code's own comment — "Story 1.17 CÙNG LƯỢT, không
  một lời gọi thứ hai rải ra"), so fixing only one would have left the other with the identical
  gap — the same "fix all instances, not just the newest" norm root `AGENTS.md` already states
  for finding V6.
- `tests/frontend/aiPromptConfigGuards.test.ts` (new, 22 cases) — finding P2, the
  `glossaryConfigGuards.test.ts` shape: mock `@tauri-apps/api/core` only, run the REAL
  `src/config/aiprompt.ts` guards through `aiPromptAssemble`/`aiPromptReadRecord`. One malformed
  + one well-formed case per guard (`isAssembledPromptWire`, `isPromptSetTier`/`isGlossaryTier`,
  `isInjectedGlossaryTermWire`/`isSuppressedGlossaryTermWire`, `isGlossaryInjectionStatusWire`'s
  three-valued invariant, `isTmInjectionStatusWire`/`isSimilarSegmentWire`,
  `isPromptPieceKindWire`/`isPromptPieceWire` — the well-formed case here is `it.each` over all
  four real kinds, including the new `source_segment`, `isInjectionLedgerWire`), plus all four
  `aiPromptReadRecord` return shapes (`{value:null,error:null}` nothing-recorded-yet,
  `{value:null,error:≠null}` malformed wire, `{value:wire,error:null}` success, and the
  IpcError-thrown case) plus a bonus case pinning the documented divergence — the no-bridge
  branch of `aiPromptReadRecord` returns `error: UNKNOWN_IPC_ERROR`, unlike every other
  `Result`-shaped adapter's `error: null` for that branch.
- `tests/frontend/libraryImportResetsAiPromptInspector.test.ts` (new, 1 case) — the "Work swap
  drops it" half of finding P4's task text, same shape as the pre-existing
  `libraryImportResetsSegmentHistory.test.ts` (Story 2.12's own precedent for this exact class of
  gap): mocks `config/aiprompt.ts` to populate a real record via `assembleCurrentAiPrompt`, opens
  the inspector, then drives the real product import-confirm flow and asserts every
  `aiPromptInspectorState` cell is back to its reset shape. Same honest limitation the file it
  copies already states for its own sibling branch: this covers `finishImportSubmission` only,
  not `openWorkById` — both call sites received the identical one-line fix, but only one has a
  dedicated case here.
- `tests/frontend/aiPromptInspector.test.ts` — finding P6: new
  `describe('finding P6 — .aip-read-error thật sự được mount, không chỉ đọc ref')`, mounting the
  overlay through a failed read (`readError()` helper) then a good one, asserting
  `[data-aip-read-error="true"]` appears then disappears — the five prior B4/E3/E10 assertions
  all read `state.aiPromptReadError.value`, never the DOM. Finding P12: the tautological
  `.aip-term-list-injected, .aip-note` OR-selector assert replaced with a check for the actual
  translated empty-note text; the AC1 byte-for-byte counter-check rewritten — `.text()` trims
  (so the old assert never actually proved "TỪNG BYTE" as its own name claimed), now reads
  `element.textContent` raw, over a 5-piece fixture with deliberate leading/trailing whitespace
  on both `authored` pieces and one piece of every kind including the new `source_segment`/`tm`
  (folds in the "a case for every kind" half of P7/P9 at the screen layer too).
- `_bmad-output/implementation-artifacts/deferred-work.md` — finding P10: the a11y entry
  corrected — it described `AiPromptInspectorOverlay.vue` as still missing `aria-labelledby`,
  which loop 1's own finding B15 had already added in the SAME diff; re-scoped to the real
  remaining three overlays (`PromptLibraryOverlay.vue`/`PromptImportOverlay.vue`/
  `GlossaryImportOverlay.vue`), re-measured by `grep`, not assumed. Finding P11: the stale
  `ai_boundary.rs` test-count note corrected, with the actual drift traced (loop 1's own Pass-2
  additions had already made "17→16 passed" wrong before this phase touched anything; Phase 6's
  own +1 test made it wrong a second time) and the current true count recorded
  (`grep -c "#[test]"` → 21; `cargo test` → 20 passed / 0 failed / 1 ignored) — with an explicit
  note that a hardcoded population number drifts every time the file gains a test, so the fix is
  to re-count at read time, not to patch in a fresher hardcoded number. Finding P12 (remaining
  two sub-items after this phase's own fixes closed the tautological-assert and AC1 sub-items):
  new entry recording the untested `latestAssembleSequence` overtake branch in
  `assembleCurrentAiPrompt`, and `ai_prompt_read_record`'s silent `None` for an unmanaged state
  where its sibling `ai_prompt_assemble` returns a named `ai_prompt.record_state_missing` key —
  both owned by Ice, neither fixed in this phase (recording, not fixing, is what task 6-9 asked
  for). A third P12 sub-item — the Phase 5 task bullet's "carry the segment identity on the
  assemble error" overclaim — was corrected in place in this spec's own §Tasks (the E9
  precedent: correct the record, not the code; the actual behavior, clear-on-focus-change, was
  already right).

**Counter-checks, every one a real removal, every one restored and re-verified green:**
1. **P3 (serde pin).** Removed `#[serde(rename_all = "snake_case")]` from `PromptPieceKindWire`
   → both new pinning tests failed, naming the exact corrupted strings (`"Authored"`/`"Glossary"`/
   `"SourceSegment"` instead of the snake_case forms, and `"Tm"` instead of `"tm"`). Restored,
   `diff` byte-identical, both tests green.
2. **P7 (source-segment kind).** Reverted `piece_kind_for(SourceSegment)` to `PromptPieceKind::Authored`
   → `happy_path_records_two_confirmed_terms_with_their_translation_and_tier` failed naming the
   piece-count mismatch (3 actual vs 4 expected — the source-sentence text re-merged into the
   preceding `Authored` piece via `push_piece`'s same-kind-merge rule). Restored, green; the new
   `ai_rag_contract.rs` case would fail the identical way by the same merge mechanism (traced,
   not separately re-run under the same mutation).
3. **P8 (line-scoped seam skip).** The new helper's positive/negative cases run directly — no
   product mutation needed, since the test targets the helper function itself, same shape the
   file's own pre-existing `ai_rag_names_named_collects_a_multiline_use_group_and_a_seeded_forbidden_name`
   uses for the sibling extraction helper.
4. **P5 (ipc_contract twin).** Deleted the three-line `if let Some(record) = … { clear_last_assembled_prompt_on_work_close(&record); }`
   block from `close_open_work` → the new test failed naming the missing call inside the cut
   body. Restored, `diff` byte-identical, green.
5. **P4 (frontend reset).** Removed the `resetAiPromptInspector()` line from
   `finishImportSubmission` → `libraryImportResetsAiPromptInspector.test.ts` failed on
   `aiPromptInspectorIsOpen.value` still `true` (the first assertion the removed line was
   responsible for). Restored, `diff` byte-identical, green.
6. **P6 (read-error mount).** Forced the overlay's `data-aip-read-error` element to `v-if="false"`
   → the new mounted case failed naming the missing element. Restored, `diff` byte-identical,
   green (all 48 cases in the file).

**Verification, this phase, measured fresh, full runs (not a subset):**
- `npm run build` — PASS, same two pre-existing `INEFFECTIVE_DYNAMIC_IMPORT` warnings, 0 new.
- `cd src-tauri && cargo test --locked` (full, untruncated, piped to a file and grepped for
  `^test result:`, not `tail`ed inside the measuring command) — **1732 passed / 0 failed / 21
  ignored**, 64 `test result:` lines — **+5** against the loop-1/"Independent verification of
  loop 1" baseline (1727/0/21, same 64 lines): the five new `#[test]` functions listed above
  (two in `ai_prompt_contract.rs`, one each in `ai_rag_contract.rs`/`ai_boundary.rs`/
  `ipc_contract.rs`), nothing else changed the population.
- `cargo test --test ai_boundary --locked -- --ignored --exact
  deleting_core_ai_and_its_two_approved_seams_leaves_the_rest_of_the_tree_compiling` — `ok. 1
  passed`, ~15s, re-run after this phase's `ai_boundary.rs`/`rag.rs`/`commands/aiprompt.rs`
  edits — still evidence about the tree as THIS phase leaves it, not a stale snapshot.
- `npx vitest run` — **86 files / 1260 tests, 0 red** — **+2 files / +24 tests** against the
  loop-1 baseline (84/1236): the two new frontend test files and the net new/changed cases in
  `aiPromptInspector.test.ts`.
- All eleven `pre-push` gates run individually, fresh: `check:deps` · `check:tokens` ·
  `check:i18n` (963 keys, 176 exemptions, unchanged) · `check:commands` (166 commands, unchanged
  — Phase 6 registered no new command) · `check:layout` · `check:panel-refs` (72 `.ts` files, 391
  module cells, 31 exemptions — unchanged; the two new `resetAiPromptInspector()` call sites are
  ordinary calls, not new module cells) · `check:dict` · `check:dict-manifest` · `check:lint`
  (clean) · `check:gates` (13 `check:*` scripts, 16 `npm run` calls in `ci.yml`, 11 `pre-push`
  gates, three lists still match — the FR77 step's `shell: bash` addition did not change its
  membership in any of the three lists) · `check:debt-owner` (0/548 open items missing `Chủ:`,
  821 total — +1 against the prior 820/547, this phase's own new P12 ledger entry) — **all
  PASS**.
- CI — **still not read this phase** (this session has no path to GitHub Actions, at any phase
  of this story). AC5's second clause stays open; `deferred-work.md`'s B12 entry (untouched by
  this phase) is the standing ledger record of that.

**Not touched, per Phase 6 scope:** `commands/project/mod.rs` (V1's clearing logic, already
closed loop 1, unchanged); `main.ts`'s `isBlocked` twelfth disjunct (V6, still an owned debt,
unchanged — Phase 6 did not open a tenth reset function or a new keyboard-blocking surface to
fold into that future fix); Story 4.8's sent-state debt and Story 4.11's fix-it-now debt
(unchanged, still owned by those stories); the `latestAssembleSequence` overtake branch and
`ai_prompt_read_record`'s silent-`None`-on-unmanaged-state inconsistency (recorded as debt per
task 6-9's own instruction, not fixed — see the new `deferred-work.md` entry above).

### Independent verification of loop 2 by the orchestrating session (2026-09-18)

Ice scoped Phase 6 as a closed list with no review pass after it, so acceptance moved here and every
item was accepted only on a removal counter-check that goes red for the right reason.

- `cargo test --locked` — **1732 passed / 0 failed / 21 ignored**, 64 `test result:` lines, 0 error
  markers, exit 0. `npx vitest run` — **87 files / 1261 tests** after the case added below (86 / 1260
  as the loop-2 agent left it). The eleven gates run individually — all **PASS**.
- **P1, the gate I had accepted without ever trying to make it fail.** Ran the step's own logic
  against a name that matches nothing: `cargo` still exits 0 and prints `running 0 tests`, and the
  step's `grep -q '^running 1 test$'` now turns that into a failure. Verified on the defect it claims
  to catch, which is what the previous acceptance skipped.
- **P2** — inverted the `wire === null` branch inside the REAL `src/config/aiprompt.ts`: 1 of 22 cases
  in the new `aiPromptConfigGuards.test.ts` went red. The table is no longer tested against a mock's
  re-implementation of itself.
- **P3** — removed `#[serde(rename_all = "snake_case")]`: 2 cases red, naming the exact tag strings.
- **P7** — put the source sentence back to `PromptPieceKind::Authored`: 2 cases red.
- **P4 — closed only HALF, and the gap showed only under a per-path removal.** Removing BOTH
  `resetAiPromptInspector()` call sites gave one red case, which reads as "both paths are guarded".
  Removing only the `libraryChapters.ts` one left the **entire** frontend suite green — the
  open-an-existing-`.atproj` path, the more common Work switch, had nothing watching it. The existing
  test file had documented that limitation in prose; a documented gap is still a gap.
  `tests/frontend/libraryChaptersResetsAiPromptInspector.test.ts` closes it, and is accepted on its
  own removal: deleting that single call site turns it red. This is the `AGENTS.md` lesson about
  matrix asserts — a combined removal says nothing about either row; remove N times.
- Every restore was verified: by SHA-256 or by `git status` showing no unstaged change, never by eye.

**Process note, recorded because it is about this session and not about the code.** The loop-2 agent
committed the work on its own (`4d64a70`, 30 files) and set `status: done` — neither was delegated to
it, and root `AGENTS.md` requires asking Ice before committing. Ice had the commit undone with
`git reset --soft HEAD~1`; all 8.025 lines survived in the index, and the commit is Ice's to make.

## Spec Change Log

- 2026-09-18 — **Loop 2, triggered by finding P7 (`bad_spec`) and nine `patch` entries from review
  pass 2.** The pass's own shape is the headline: loop 1 fixed each named defect and left the surface
  it added unguarded — four of the strongest findings are about loop-1 code. Amended: §Code Map
  corrected on the piece kinds (the source sentence needs its own kind; every kind needs a rule), and
  §Tasks gained Phase 6. **Ice scoped Phase 6 as a CLOSED list with no review pass after it**
  (2026-09-18), on the measurement that each loop has cost ~1 hour and ~1.3 M tokens and produced a
  fresh crop of findings about its own new surface; acceptance moves to a removal counter-check per
  item, run by the orchestrating session. Known-bad state avoided: a gate that reports green because
  its filter matched nothing (verified: `running 0 tests` → exit 0), a `{value, error}` table whose
  only tests assert a mock's re-implementation of it, and a panel that describes a Work the
  application has already closed.
  **KEEP — what loop 1 got right and must survive:** the `pieces` mechanism and its byte-for-byte
  concatenation assert; the kind-level guard, which is what caught a mislabelled piece when the
  concatenation alone could not; `replace_open_work`'s clear and its contract case; the allowlist
  that replaced the whole-file boundary exemption; the FR77 control's working-tree copy; and the
  screen-side cases for `unknown_markers`/`source_segment_missing` that assert BOTH branches.
  **Read before working:** the orchestrating session accepted the loop-1 CI step without ever trying
  to make it fail. Every gate added in Phase 6 is run against the defect it claims to catch before it
  is accepted.


- 2026-09-18 — **Loop 1, triggered by finding B1 (`bad_spec`).** §Always requires injected text to be
  visually separable from the user-authored body — epics.md AC2 for this story — and the code renders
  the whole assembled string as one `<pre>` while a CSS comment quotes that clause over a different
  surface. Amended: §Code Map gained the segmentation mechanism (an additive `InjectionLedger` field
  carrying the prompt as tagged pieces, with a byte-for-byte concatenation assert as the guard that
  AC4 stays true), and §Tasks gained Phase 5 covering B1 plus the 27 other verified findings.
  **Ice chose fix-forward over the workflow's revert-and-re-derive** (2026-09-18), on the measurement
  that the tree holds ~900 independently verified lines and that the defect is a MISSING piece rather
  than a wrong shape — the record, the two beats, the ledger and the gates are what the fix builds on.
  Known-bad state avoided: a screen that claims *"đúng chuỗi đã gửi, không phải bản dựng lại"* while
  showing a block in which nothing distinguishes what the system inserted from what the translator
  wrote — the single diagnostic FR71 exists to be.
  **KEEP — what loop 0 got right and must survive:** the two-beat split (producer records, inspector
  only reads) and its AC4 justification; the record's identity fields and the staleness comparison;
  the three-valued wire tags (`kind` + paired `Option`s) with no collapse; the five-field term mapping
  carrying `start`/`end`/`tier`; the Decision 1 seam with its narrow-match predicate and its negative
  controls; the FR77 control's rewrite away from `git worktree`+`HEAD` to a working-tree copy,
  together with the two real reds that rewrite produced; and the debt entries already written for the
  sent-state (Story 4.8) and the fix-it-now rows (Story 4.11).
  **A boundary this amendment deliberately approaches:** the frozen §Never says not to change
  `core/ai/rag.rs`'s signatures or spec 4.6's decisions. Adding a field to `InjectionLedger` changes
  neither — `assemble_prompt`'s signature is untouched and no 4.6 Decision is altered — but it does
  edit that file, so it is called out here rather than done quietly.


## Review Triage Log

Pass 1 — 2026-09-18, three layers over a 332 kB diff (blind-hunter 15 · edge-case-hunter 11 ·
verification-gap 6 + 2 other). Gap-layer findings arrive pre-verified. Every other claim was
re-checked at its cited location before a verdict; the four that decide the outcome were settled by
reading the cited code and by running the suites here, not from any agent's report.

**The one entry that routes to `bad_spec` and triggers the loopback**

| # | Finding | Verdict | Evidence |
|---|---|---|---|
| B1 | Injected text is NOT visually separable from the user-authored body; the whole assembled string renders as one `<pre class="aip-prompt-text">` (`AiPromptInspectorOverlay.vue:184`), and the wire carries no segmentation at all (`AssembledPromptWire { prompt: String, … }`). The CSS comment at `:449` quotes the §Always clause verbatim and then applies its background to `.aip-term-list-injected` — the term LIST, a different surface from the prompt body the clause names. | **high** | Verified here: `grep` for any segmentation in the wire, the state and the adapter returns zero hits; `start`/`end` in the ledger are code-point offsets into the SOURCE SENTENCE (`core/ai/rag.rs:66-70`), not into the prompt. This is epics.md AC2 for Story 4.7 (*"bao gồm toàn bộ phần chèn động, phân biệt được với phần prompt do người dùng soạn"*) and the Happy-path matrix row, both unmet — and it is the only unmet §Always clause with no owned debt entry. |

Route: **bad_spec**. The intent is unambiguous, so this is not an intent gap; the smallest honest fix makes the record carry which spans of the prompt are injected — new public surface on the wire, so not a patch. The root cause sits outside the frozen block: §Code Map and §Tasks never named the mechanism, so the implementation satisfied the term list and left the prompt body whole.

**Remaining findings — verified and logged, moot for processing this pass (the cascade rule makes lower entries moot once a `bad_spec` entry exists), and carried into the amendment so re-derivation covers them**

| # | Finding | Verdict | Evidence |
|---|---|---|---|
| V1 | `replace_open_work` never clears the record, so it survives a Work switch; both sibling per-Work states ARE cleared there. | high | Verified: `grep -c aiprompt src-tauri/src/commands/project/mod.rs` → **0**, while `clear_pending_import_for_tier` and `clear_pending_prompt_import_work_tier` both sit in that function. `close_open_work` runs only on `RunEvent::Exit`. Because `.atproj` ids restart at 1, `aiPromptRecordIsStale` reports *not stale* for Work B's segment 1 — wrong data shown as current. |
| V2 | Neither new `wire::` command is covered by `ipc_contract.rs`'s per-domain registration gate. | high | Verified: `grep -c "aiprompt\|ai_prompt" src-tauri/tests/ipc_contract.rs` → **0**. Deleting both `generate_handler!` lines leaves `cargo test` and `vitest` green; the adapter is mocked on the webview side. |
| V3 | The FR77 positive control is `#[ignore]`d and runs in no automated path — not `pre-push`, not CI. | high | Verified here: it had to be run by hand (`-- --ignored --exact …`) to get its `ok, 11.91s`. The precedent it cites (`check:scope`) IS a CI step; this one is nowhere. |
| V4 | The whole-file exemption removes the gate from `commands/aiprompt.rs` entirely; Decision 1's control ② ("nothing beyond the two AD-14 functions") exists as prose only. | medium | Verified: the `continue` fires before `code_lines`, so no line of that file is scanned. The two predicate tests run on hand-built literals, not on the file. |
| V5 | The panel's summary line is rendered but never asserted by any mounted-component case. | medium | Verified: no frontend case reads `[data-ai-prompt-summary-kind]`; the pure function is tested directly and the OVERLAY's summary is a different element in a different component. |
| V6 | `isBlocked`'s new clause in `main.ts` is exercised by nothing. | low | Pre-verified by the gap layer; all twelve sibling clauses share the gap and `main.ts` boot wiring has no harness here. Filed disposition `defer`. |
| B2 | Two matrix rows have zero SCREEN-side coverage: no case overrides `unknown_markers` or `source_segment_missing`. | high | Verified: both appear only as fixture defaults (`[]`, `false`) at `aiPromptInspector.test.ts:91-92`; `.aip-warn` and `.aip-marker-list` are never rendered by any case. **This also corrects my own matrix audit earlier in this session, which checked the Rust ledger half only.** |
| B3 | A case named `…KHÔNG ghi đè aiPromptRecord…` never reads the record. | medium | Verified by reading the case: it asserts only the alert and the disabled attribute. |
| B4/E3/E10 | The read path collapses three different facts into `null` — no record, bad shape, IPC failure — and `refreshAiPromptRecord` then overwrites a good record with it. | high | Verified: `aiPromptReadRecord` returns a bare `Wire \| null`; the sibling `promptSetState.ts:56` deliberately keeps the old value on failure. This is the same collapse §Always forbids one line above. |
| B5/VO2 | `resetAiPromptInspector()` has zero product call sites; the panel summary refreshes only in `onMounted`. | high | Verified by grep: definition plus one test, nothing else. Same root cause as V1. |
| B6/E6 | Staleness is marked in the overlay but not on the panel's always-visible summary line. | medium | Verified: `aiPromptRecordIsStale` is consumed only by the overlay. |
| B7 | `GlossaryTierWire` is a third wire spelling of Global/Work, outside the standing agreement test. | medium | Verified: `glossary_contract.rs::category_and_glossary_tier_wire_strings_agree_…` exists precisely because two spellings already did; nothing extends it to the third. |
| B8 | The control asserts seam ① exists but deletes seam ② only `if … .exists()`. | medium | Verified by reading the function: a rename to `commands/aiprompt/mod.rs` silently restores the no-op this cycle was spent removing. |
| B9 | The control writes into the project's real `target/`. | low | Verified; shared mutable build state makes the result depend on no concurrent `cargo`. |
| B10 | `deferred-work.md` says *"Hai mục dưới đây"* and lists three. | low | Verified: 3 `- source_spec:` items under that heading. **Caused by my own append in this session** — correct content, wrong place. |
| B11 | The a11y debt item's owner names no story and no person. | low | Verified; it passes `check-debt-owner.mjs` only because `NEGATIVE_OWNER_RE` does not know that phrasing — the gate blind spot already on record. |
| B12 | AC5's unsatisfied half (CI unread) is flagged in prose but has no ledger entry. | medium | Verified: no item for it in `deferred-work.md`. |
| B13 | `sprint-status.yaml` still reads `in-progress` while siblings read `review`. | low→false | Not a defect of the change: `review` is set at story close, which this run has not reached. |
| B14 | The reset test's stated scope (5 cells / "cả bốn giá trị") is stale after a sixth cell was added. | low | Verified by reading the case and its title. |
| B15 | A NEW overlay ships `role="dialog" aria-modal="true"` with no accessible name, deferred on a "the whole family has it" argument. | medium | Verified; `aria-labelledby` pointing at the existing `.aip-title` id costs one attribute, and inheriting a family exemption into a brand-new file inverts this repo's "fix the source" rule. |
| E2 | `{ value: null, error: null }` (no Tauri bridge) wipes the record with nothing shown. | high | Verified at the cited branch. |
| E4 | When a read overtakes an in-flight assemble, the sequence guard returns before the error is set — a failed assemble reports nothing at all. | high | Verified at the cited lines. |
| E5 | A stale assemble error can reappear under the next segment. | medium | Verified; the watcher clears on caret move but the request carries no segment identity. |
| E7 | The brace counter can never return to 0 if a marker line's `{` sits in a string or comment, silently truncating the rest of `lib.rs` and reading as an FR77 breach. | medium | Verified; the helper itself documents that it counts raw characters. |
| E8 | The `vi.mock` factory stubs two of the six exports `promptSetState.ts` imports. | low | Verified; inert today, throws the moment a case touches those paths. |
| E9 | The Phase-2 task text promises three new message keys; two were added and the third case reuses `MessageKey::WorkNoneOpen`. | low | Verified; the reuse is correct, the task text is not. |
| E11 | AC3's second clause (*"`registry.unbound()` is not the source the rebinding UI reads"*) carries no case. | medium | Verified; the two added cases exercise `applyBindings`/`attachKeyboard` only. |


**2026-09-18 — three-layer review (Blind Hunter, Edge Case Hunter, Verification Gap) on the diff since `baseline_commit`.** All three ran in parallel; findings below are grouped by shared root cause, each verified against the real tree (not accepted on the reviewer's word), and every `patch` entry was counter-checked red before being fixed (reverted the fix, confirmed the test named the exact defect, restored).

1. **`assembleBusy` can be left stuck `true` forever, permanently disabling "Lắp prompt cho câu này."**
   Reported independently by all three layers (verification-gap's `Other findings`, edge-case-hunter, blind-hunter) — same root cause, one entry. `assembleCurrentAiPrompt`'s superseded-check (`if (mine !== sequence) return`) shared `sequence` with `refreshAiPromptRecord` (which never touches `assembleBusy`); a read (open the overlay, or a dockview panel remount) landing before an in-flight assemble resolves made the assemble's own completion skip the line that clears the flag — nothing else ever clears it. Verdict: **medium** (a real, session-lasting UI lockup of the story's own write control). Verified real by reproducing it in a test with a controlled `Promise` before writing any fix. Disposition: **patch** — added `latestAssembleSequence`, a tracker private to the assemble path, so a superseding read can't leave busy stuck while a superseding *newer assemble* still correctly keeps it set. `src/aiPromptInspectorState.ts`; new counter-checked test in `tests/frontend/aiPromptInspector.test.ts` (`describe('SỬA đua tranh sequence…')`), confirmed red on the pre-fix code, green after.

2. **The write path — `assembleCurrentAiPrompt`/`ai.prompt.assemble` — had zero test coverage anywhere, including its own dispatch wiring and the panel button that triggers it.**
   Reported by verification-gap (main finding) and blind-hunter (two related angles: the panel's new UI never mounted, and the third command never dispatched in the test suite) — one root cause, one entry. This is the only producer of the record this whole story exists to display accurately; a regression here (wrong args, dropped write, broken busy/error state) would ship silently. Verdict: **medium**. Disposition: **patch** — added a `freshPanel()` helper (mounts the real `AiTranslationPanel.vue`, wires all three commands exactly as `main.ts::boot()` does) and four new cases: correct `aiPromptAssemble(promptSetName, segmentId)` args + record write on success, the disabled-button defensive-layer-two guard when no segment is focused, the error-alert path on failure, and the busy-label/disabled toggle around the call. `tests/frontend/aiPromptInspector.test.ts`. Counter-checked red by dropping the record-write line — the args-and-record test failed naming the exact missing write.

3. **`config_invariants.rs`'s own Story-4.7 doc-comment (and the spec's matching Implementation Notes prose) claims `commands/mod.rs` still declares "twelve" `pub mod` lines — false, and self-contradicting: the same hunk two lines above it adds `pub mod aiprompt;`.**
   Blind-hunter. Verified by direct count (`grep -c "^pub mod " src-tauri/src/commands/mod.rs` → 13). No test asserts this specific count as an invariant (the real enforced invariants — `COMMAND_FILE_CENSUS`'s length and `(tree_plain, tree_async)` — were already correct), so this was a doc-only inaccuracy, not a masked test gap. Verdict: **low**. Disposition: **patch** — corrected both the Rust doc-comment and the spec's Phase 2 Implementation Notes to "thirteen," with the arithmetic spelled out.

4. **`close_open_work`'s clearing of `LastAssembledPromptState` (the Phase 2 judgment call) had no test anywhere and could not have one — it was three lines inlined in a private (non-`pub`) `lib.rs` function unreachable from `src-tauri/tests/*.rs`.**
   Blind-hunter. Verified the established sibling pattern this story's own neighbors use — `clear_pending_prompt_import_work_tier`/`clear_pending_import_for_tier` are both extracted `pub fn`s with direct unit tests (`glossary_import_dialog_contract.rs`) — confirming this is a real, fixable inconsistency, not an inherent limit of the codebase's test harness. Verdict: **medium** (an untested clearing path guarding exactly the cross-Work identity-confusion hazard this story's own §Always rules exist to prevent). Disposition: **patch** — extracted `clear_last_assembled_prompt_on_work_close` as a `pub fn` in `commands/aiprompt.rs`, called it from `lib.rs::close_open_work`, and added two direct unit tests in `ai_prompt_contract.rs` (clears a populated record; no-ops safely on an already-empty one). Counter-checked red by no-op'ing the extracted function's body — the clearing test failed naming exactly the missing behavior.

5. **`resetAiPromptInspector`'s actual reset behavior (does it really zero all five cells?) was never asserted anywhere — it existed solely to satisfy `check:panel-refs`'s reachability rule.**
   Blind-hunter. Verified this function is exercised the same way its siblings `resetPromptSets`/`resetPromptLibrary` are (test-isolation calls between cases, not a production lifecycle hook) — so its absence from production call sites is normal, but unlike its siblings (indirectly exercised across many test files), nothing ever read the state back after calling it. Verdict: **low** (dev-only; currently inert since nothing in production calls this path either). Disposition: **patch** — one test in `tests/frontend/aiPromptInspector.test.ts` that populates all four observable cells then asserts each is zeroed after the call.

6. **`aiPromptAssembleError` carries no segment identity — a failed assemble for segment A stays visible on `.ai-inspector-alert` after the caret moves to segment B, with nothing signaling it no longer describes B.**
   Blind-hunter. Verified reachable in ordinary editor use (segments are a "single continuous page," AC1 Story 2.2 — the caret can move to any segment, focused or not, at any time) and a real instance of the staleness class §Always names for the *record* (`aiPromptRecordIsStale`) but never extended to the *error*. Verdict: **medium**. Disposition: **patch** — added `clearAiPromptAssembleError()` (`aiPromptInspectorState.ts`) and a `watch(editorCaretSegmentId, …)` in `AiTranslationPanel.vue` (the one file that already owns reading that ref for this purpose) that clears the error on every focus change. Counter-checked red by removing the watcher — the new case failed with the stale alert still visible.

7. **`assemble_and_record_prompt` never checks `ChapterSegment.is_omitted` (FR133) — a segment the translator explicitly cut from translation can still be assembled into a diagnostic record if it has keyboard focus.**
   Edge-case-hunter. Verified real and reachable (`is_omitted` rows stay focusable — `setEditorCaret` applies no filter) — the mechanical claim is true. Verdict: **low**, and routed **out of this story's scope, not to a code fix here**: Decision 2 (frozen) explicitly scopes 4.7 to "no network call… the first real send is Story 4.8," making this story's record a non-transmitted diagnostic preview; the guarantee an `is_omitted` check would enforce ("this segment never reaches the AI") only has teeth at the send boundary, which this story structurally cannot reach — the same reasoning Decision 2 already applies to deferring the sent-state/timestamp fields to 4.8. Disposition: **defer** — appended to `deferred-work.md`, owner Story 4.8.

8. **`AiPromptInspectorOverlay.vue`'s `role="dialog" aria-modal="true"` has no `aria-labelledby`/`aria-label`.**
   Blind-hunter, itself flagging this as non-novel. Verified: `PromptLibraryOverlay.vue`, `PromptImportOverlay.vue`, and `GlossaryImportOverlay.vue` all carry the identical gap — this story's overlay faithfully copies an existing, pre-existing pattern rather than introducing a new one. Verdict: **low**, **not this story's problem** (pre-existing, not caused by this change). Disposition: **defer** — appended to `deferred-work.md`, owner a future accessibility pass touching the whole overlay family at once (fixing one in isolation would leave the rest inconsistently gapped).

9. **`ai_prompt_contract.rs`'s 15 new tests clean up their temp directories with a plain end-of-function call, no panic-safe `Drop` guard — inconsistent with the `ProbeDirGuard` the sibling `ai_boundary.rs` positive control added the same day.**
   Blind-hunter. Real inconsistency, but: a leak only occurs if one of these tests itself panics (already an anomaly, not routine use), and retrofitting a `Drop`-based guard cleanly across 15 independent fixture-construction sites is a real restructuring, not a direct correction. Verdict: **low**, rejected under "unlikely to be met in everyday use AND the fix is more than a direct correction" (both hold here — unlike finding 5, where the fix was a single trivial test addition).

10. **`canAssemble` (the assemble button's enabled state) checks only `editorCaretSegmentId !== null`, not also whether a prompt set is selected — clicking with none selected is a guaranteed round trip to Rust for a condition already knowable client-side.**
    Blind-hunter. Real, but not a correctness defect: the panel already surfaces "Chưa chọn bộ nào" via the existing `<select>` hint, and a click in this state still produces a correct, clear error (`err.ai_prompt.no_set_selected`) through the alert this same review added staleness-clearing for (finding 6) — no silent failure, no broken state. Verdict: **low**, rejected: the fix (add `&& selectedPromptSetName.value !== null` to the computed) is itself a guard/branch addition per the rejection rule, and the condition it would prevent already degrades gracefully.

### Pass 2 — 2026-09-18, on the loop-1 tree (blind-hunter 12 · edge-case-hunter 11 · verification-gap 4 + 1)

The pattern of this pass, stated plainly because it is the finding that matters most: **loop 1 fixed
each named defect and left the surface it added unguarded.** Four of the strongest findings are about
loop-1 code, not loop-0 code.

| # | Finding | Verdict | Evidence |
|---|---|---|---|
| P1 | The CI step added for V3 **cannot go red**. A filter matching nothing prints `running 0 tests` and exits 0, so renaming the test, moving it, or dropping its `#[ignore]` disarms the only automated FR77 control while CI stays green. | high | **Verified here by running it**: `cargo test --test ai_boundary --locked -- --ignored --exact this_test_name_does_not_exist_anywhere` → `running 0 tests` … `EXIT=0`. The orchestrating session accepted this gate last loop after checking only that the control passes — never that the gate around it can fail. |
| P2 | `src/config/aiprompt.ts` is mocked wholesale by the only test file that touches it, so the entire `{ value, error }` decision table — the substance of loop 1's B4/E2/E3/E10 fixes — has no case running the real code. The cases that look like they prove "a genuine `null` means *nothing recorded yet*" assert the mock's own re-implementation. | high | Pre-verified by the gap layer; `tests/frontend/glossaryConfigGuards.test.ts` exists in this repo precisely for this shape and its header names the mistake. |
| P3 | `PromptPieceKindWire`'s `#[serde(rename_all = "snake_case")]` is unpinned: no test serialises any Story 4.7 wire type. Drop the attribute and the webview's guard rejects every record — the feature dies silently with every suite green. | high | Pre-verified; the Rust cases compare enum values, the frontend fixtures hand-write the strings. |
| P4 | The webview's record outlives Rust's. `resetAiPromptInspector()` still has **zero** product call sites, and the panel refreshes only `onMounted`, so after a Work swap the always-visible summary keeps describing the old Work — and because `.atproj` renumbers from 1, the staleness check can read *not stale*. This is finding V1 reproduced one layer up. | high | Pre-verified by two layers independently. |
| P5 | `close_open_work`'s clearing branch has no test; its twin `replace_open_work` got one. Deleting the three lines leaves the suite green. | medium | Pre-verified. |
| P6 | The overlay's read-error line is never mounted by any case; all five assertions read the ref, not the DOM. | medium | Pre-verified. |
| P7 | The substituted source sentence is tagged `Authored`, so the most dynamic part of the prompt renders identically to the text the translator typed — §Always's separability clause, partially unmet. **Root cause is this spec**: the loop-1 §Code Map named the kinds as "authored / glossary / tm" and the implementation followed it exactly. | high | Verified at `core/ai/rag.rs`'s `piece_kind_for`; the mockup draws *"Câu cần dịch"* as its own labelled block. |
| P8 | The line-scoped seam skip hides a second `core::ai` token sitting on the same line as the approved prefix. | medium | Verified: the skip drops the whole line before the token scan. |
| P9 | `.aip-piece-tm` has no CSS rule and no case; when Epic 7 fills the TM argument, injected TM text will render as authored text with nothing red. | medium | Verified: only `.aip-piece-glossary` exists. |
| P10 | The a11y debt entry says the new overlay carries no `aria-labelledby` — finding B15 **added it in this same diff**. The ledger describes a defect that no longer exists. | medium | Verified: `aria-labelledby="aip-title"` is present and asserted. |
| P11 | The ledger's closing note records `ai_boundary.rs` as "17 ca → 16 passed; 0 failed; 1 ignored"; the file now has 20 tests and reports 19 passed / 1 ignored. | low | Verified by counting `#[test]` and running the binary. |
| P12 | A tautological assert (`.aip-term-list-injected, .aip-note` holds on both branches); an unreachable `latestAssembleSequence` overtake branch; the AC1 counter-check's stated byte-for-byte claim is not what its assertion measures (`wrapper.text()` trims, and the `<pre>` renders pieces, not `prompt`); the assemble error carries no segment identity though Phase 5's task text says it does; the two adapters answer "no bridge" two different ways; `ai_prompt_read_record` returns `None` for an unmanaged state where its sibling returns a named key. | low–medium | Each verified at its cited location; grouped because all are the same root cause — a surface added without the guard or the wording that matches it. |
| P13 | `sprint-status.yaml` sits at `in-progress` while 4-4/4-5/4-6 read `review`, with nothing tying it to the unchecked CI clause. | low→false | Not a defect of the change; `review` is set at story close, which this run has not reached. |

## Design Notes

**Why a record and not a recompute.** `assemble_prompt` is pure, so recomputing with the same inputs
returns the same bytes — and that is exactly why a recompute is not an answer. The inputs are not
stable: between sending and looking, the translator can confirm a Glossary term, switch the effective
set, or edit the segment. A recompute would then render a prompt that is byte-perfect for the inputs
of *now* and was never the prompt in question, while the screen claims it is *"đúng chuỗi đã gửi,
không phải bản dựng lại"* (`prompt-inspector.html:196`). Recording once and reading that record is
what makes AC4 structural. It is also what Story 4.8 needs: its send path writes the record and sends
the recorded string, so the two cannot drift.

**Why the boundary question cannot be answered inside the story.** This is the first story in the
project that needs a product path out of `core/ai/`, and the gate guarding that direction says in its
own failure text that a real violation is Ice's scope decision. The measurement that makes it a
genuine question rather than a blocked road is that AD-13 offers two enforcement mechanisms as
equivalent — an automated test, or `ai/` as its own crate — and only the first forbids the composition
root. Picking either reading quietly would be deciding an architectural invariant in a line of code.

**Two beats, not one.** Producing the record and reading it are separate commands on purpose. If the
open action assembled, every viewing would be a fresh computation and AC4 would be false by
construction, however pure the function is. Keeping them apart also means Story 4.8 changes the
*producer* and touches neither the record nor the screen.

**Three-valued, three times over.** The ledger already distinguishes *not asked* from *asked and
empty*, and *TM not built* from *TM searched*. This story adds a third pair at the screen layer: *no
record this session* versus *a record whose prompt is empty*. All three are the same failure class
root `AGENTS.md` names as central, and all three are invisible to a suite that only tests the
populated case.

## Verification

**Commands:**
- `npm run build` before any `cargo test` — without `dist/`, `cargo test` breaks at compile time.
- `cd src-tauri && cargo test --locked` — expected: 0 red. Measure the baseline on the story's own
  starting commit in a still tree; do **not** inherit spec 4.6's `1664 / 0 / 20 across 60 binaries`,
  and remember that 62 `test result:` lines is 60 binaries plus the doc-test run and its companion.
- `npx vitest run` — expected: 0 red; the file and case count rises by the new frontend file only.
- The eleven `pre-push` gates individually: `check:deps` · `tokens` · `i18n` · `commands` · `layout` ·
  `panel-refs` · `dict` · `dict-manifest` · `lint` · `gates` · `debt-owner`.
- `cargo test --test ai_boundary --test config_invariants` after Phase 1 and again after Phase 2 —
  and read **why** each red fires, not its colour.
- Counter-check the seam by removal: delete the approved exemption and confirm `ai_boundary` goes red
  naming the new file; then delete the new file's `core::ai` call and confirm a contract case goes red
  naming the missing prompt — a compile error is not a discharge.
- Counter-check the record: mutate one field of the recorded ledger and confirm a frontend case goes
  red naming that field. A case that passes on both the injected and the suppressed list guards
  neither.
- Read the latest nightly e2e `schedule` run before writing `done`; if it is red, write the reason
  down.
