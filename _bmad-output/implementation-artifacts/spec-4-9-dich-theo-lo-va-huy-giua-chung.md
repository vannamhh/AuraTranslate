---
title: 'Story 4.9 — Batch-translate the selected segments, with progress and mid-flight cancel'
type: 'feature' # feature | bugfix | refactor | chore
created: '2026-09-21'
status: 'done' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
baseline_commit: '2376aebbc713cd80beea3ecd64e2fba68919f904'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR73 asks for translation of *"several consecutive segments"* with cancel mid-flight,
and Story 4.8 built only the single-segment path. Two things block a batch outright: the app has
**no multi-segment selection at all** (`editorCaretSegmentId` is one `number | null`, and grid rows
carry `hasCaret`, never `selected`), and 4.8's stream is a `Channel<String>` of bare token text, so
nothing on the wire says which sentence a token belongs to.

**Approach:** Build a range selection over the open Chapter's ordered segments — anchor at the
caret, extended by chord, marked in the grid — then one Rust batch command that resolves config and
reads the API key **once**, translates the selected segments in document order, streams events
carrying their own `segment_id` over a single Channel, re-checks the existing generation counter
between sentences, and stops at the first error naming the sentence it stopped on.

**Decisions taken with Ice, 2026-09-21:**

1. **The batch input is a real range selection in the grid**, not a derived "next N from the caret".
   Anchor at the caret, extend with a chord, the grid marks the rows, the panel reports the count.
   This is the repo's first selection model and later stories inherit it. Accepted consequence,
   stated when the choice was made: there is **no ceiling on N** — the user may select a whole
   184-sentence Chapter, and that is the user's own spend decision.
2. **Promote stays per sentence.** A finished batch is promoted one sentence at a time through the
   existing `ai.translate.promote` (`Mod+Shift+Enter`). No bulk promote in this story. Measured
   reason: `promote_ai_translation` writes `target_text` with **no** `segment_version` row — the
   only `INSERT INTO segment_version` in the tree is `confirm_segment` (`segment.rs:2449`) — so a
   bulk promote would overwrite N translations with no undo path.
3. **The FR20 debt owned by Epic 4 closes here as KHÔNG LÀM.** What FR20 wanted — the same sentence
   lined up in two panels — is already delivered by the caret: promote is per sentence (Decision 2),
   each promote targets the caret segment, and moving the caret already scrolls its grid row into
   view through `focus()` (`GridPanel.vue:1177`, measured three ways in Story 2.10, Ice signed
   2026-08-18). The reason is the caret, **not** the size of the batch list — it therefore survives
   Decision 1's unbounded selection.
4. **The Chapter-switch reset gap that 4.8 left is closed in this same pass.** Measured 2026-09-21:
   `resetAiTranslate` is wired into the two **Work**-change clusters only, so switching Chapter
   inside one Work leaves a single run streaming and billing against a Chapter the user has left.
   This story is already editing that exact call site to wire the batch reset, so it wires the
   single-run reset beside it and covers both with cases. Accepted consequence, stated when the
   choice was made: this story's diff carries one fix that is not its own AC, and it changes
   behaviour that 4.8's 17 cases guard.

## Boundaries & Constraints

**Always:**

- The batch is exactly the user's selection, translated in **document order**.
- One AI call in flight process-wide. Starting a batch supersedes an in-flight single run and vice
  versa — there is one `AiTranslateGeneration` (`lib.rs:1215`) and it stays one.
- Cancel stops at the **current sentence boundary**: the in-flight sentence's partial text is
  discarded, every sentence already finished keeps its result in the panel, and sentences not yet
  reached are never called and therefore never charged.
- The first error **stops** the batch and names the sentence. Never skip a failed sentence and carry
  on. No automatic retry anywhere — with BYOK every call is the user's money.
- Results stay in the AI Translation panel. Nothing auto-writes into the Editor.
- A segment carrying `is_omitted` is skipped **before** any provider call and before the keychain is
  read, the same order 4.8 established (`aitranslate.rs:223-235`).
- Errors cross IPC only through `IpcError::new(code, message_key, params, retryable)` (AD-21); no
  display text in Rust, and new `message_key` variants are declared inside `message_keys!`.
- Every module-level ref added under `src/**/*.ts` is cleared by an exported `reset*()` in its own
  file and joins **both** Work-change clusters (`modes/libraryChapters.ts:298`,
  `modes/libraryImport.ts:428`).

**Never:**

- No "translate the whole Chapter" control. The batch is only what the user selected.
- No fourth port (AD-2 allows three; `TranslationProvider` is the third), no new crate, no new Tauri
  permission, no new gate script.
- No auto-reconnecting SSE client and no SSE crate (AD-22; NFR15 has not cleared one).
- No second `Channel` per sentence and no loose Tauri events — one Channel for the whole batch.
- Not in this story: token counts and cost estimates (FR76 → Story 4.11), error copy and retry
  policy (FR75 → Story 4.10), bulk promote, narrow-layout calibration (Story 4.12), and TM
  injection (Epic 7 fills `RagInjector`'s TM parameter, signature unchanged).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Batch over a selection | 5 consecutive segments selected, AI configured | Each is translated in document order; every streamed event carries its own `segment_id` and lands on that sentence's row; outcome `done` | N/A |
| Cancel mid-batch | Batch of 12, cancel while sentence 6 streams | Sentence 6's partial text is discarded; sentences 1–5 keep their results; 7–12 are never called; outcome `cancelled` | N/A |
| Error mid-batch | Provider fails on sentence 6 of 12 | Batch stops; 1–5 keep their results; 7–12 are never called | `Err(IpcError)` code `ai_translate.batch_stopped`, param `segment_id`, `retryable` from 4.8's existing `From<OpenAiClientError>` mapping |
| Omitted segment inside the selection | Segment 3 of 5 has `is_omitted` | Skipped with no provider call and no keychain read; its row shows skipped; the batch continues to segment 4 | N/A — skipping is a state, not an error |
| AI not configured | No key in the keychain | Outcome `not_configured` before any provider call; the selection is untouched | N/A — "not configured" is not an error (FR77) |
| Selection of exactly one | One segment selected | Runs as a batch of one through the batch command; does not silently fall back to `ai_translate_segment` | N/A |
| Work or Chapter changes while an AI call runs | User opens another Work, imports, or switches Chapter inside the same Work | The in-flight call is cancelled and its results cleared on **all three** paths, for a batch **and** for a single run (Decision 4) | N/A — 4.8 review finding #1 covered only the two Work paths; the Chapter path resets the Editor but calls no AI reset today |
| Selection changes while a batch runs | User extends the selection mid-batch | The running batch keeps the frozen list it started with; the new selection only affects the next run | N/A |
| Empty selection | Nothing selected, or caret is `null` | The batch command is disabled; no IPC call is made | N/A |

</frozen-after-approval>

## Code Map

**What 4.8 left, and what of it is reusable as-is**

- `src-tauri/src/commands/aitranslate.rs:183` `prepare_translate_call(global, open, record,
  prompt_set_name, segment_id) -> Result<PrepareOutcome, IpcError>` — sync, returns owned data, no
  lock held across an await. The batch needs the same shape over a **list**; copy the discipline,
  not the body.
- `:278` `run_translate_call<P: TranslationProvider>(provider, prepared, channel, should_cancel)` —
  the seam a fake provider substitutes at. The batch loop wraps this, it does not replace it.
- `:134` `PreparedTranslateCall { endpoint, model, temperature, max_tokens, api_key, prompt }` —
  deliberately **no** `derive(Debug)`; anything holding it inherits that ban.
- `:260` `AiTranslateGeneration(Arc<AtomicU64>)`, `:263` `next()`, `:267` `is_current()` — private to
  this module, which is where the batch also lives, so no visibility change is needed. One counter
  per process (`lib.rs:1215`); that is the mutual-supersede rule, not a limitation to work around.
- `:353` `AiTranslateOutcomeWire { NotConfigured, Done, Cancelled }`, `#[serde(tag = "state",
  rename_all = "snake_case")]` — **reuse it** for the batch outcome; the three values are exactly
  right and a second enum would drift.
- `:231-235` the keychain read (`keychain::read()` → `expose_secret()`), after the `is_omitted`
  guard. The batch performs this **once** for the whole run, not per sentence.
- `:102-124` `impl From<OpenAiClientError> for IpcError` — all provider variants already collapse to
  one code with the right `retryable` flags. The batch wraps, it does not re-map.
- `src-tauri/src/ports/translation_provider.rs:120-148` the trait, `:80` `TranslateRequest<'_>`,
  `:102` `TranslateOutcome { Done, Cancelled }` — **frozen**; the batch adds no method and no port.
- `src-tauri/src/core/ai/client.rs:217` the `TranslationProvider` impl, `:306`
  `enforce_sse_buffer_cap`, `:337` `interpret_sse_event`, `:370` `split_sse_frames` — all pure and
  already tested. Untouched by this story.
- `src-tauri/src/commands/aiprompt.rs:381` `assemble_and_record_prompt(...)`, `:474`
  `mark_prompt_as_sent(record, segment_id, sent_prompt, model, sent_at)`. ⚠️ `:326`
  `LastAssembledPromptState = Mutex<Option<AssembledPromptRecord>>` holds **one** record for the
  whole session, so a batch of N overwrites it N times and the FR71 inspector ends up showing the
  **last sentence actually sent**. That reading is still true to FR71's words and is accepted; record
  it in §Implementation Notes rather than widening the record here.
- `src-tauri/src/commands/segment.rs:1050` `read_open_chapter_segments(open)` — the row list already
  carries `is_omitted` and the ordering, so both the selection and the omitted-skip cost one field
  access, not a query.
- `src-tauri/src/commands/segment.rs:2107` `promote_ai_translation(open, segment_id, target_text)` —
  unchanged by this story (Decision 2). 🔴 It writes `target_text` with no `segment_version` row; do
  not build anything on top of it that writes more than one segment per user action.

**The selection model has no precedent — measured, twice, independently**

- The only "current segment" in the repo is `src/panels/editorPanelState.ts:106`
  `caretSegmentId`, exported read-only at `:122` as `editorCaretSegmentId`. Grid rows carry
  `hasCaret` only (`src/panels/editorSegments.ts:187-191`).
- ⚠️ Do not mistake these for a selection model: `src/panels/selectionContract.ts` and
  `extendSelectionLeft|Right|WordLeft|WordRight` (`src/main.ts:809-813`) are **text** selection
  inside one cell; `src/modes/readingState.ts:49` `anchorSegmentId` is a Reading-mode scroll anchor;
  every `shiftKey` hit in `src/**` is overlay Tab-cycling.
- Story 3.8's batch approve is **not** a selection: `src/glossaryQueueState.ts:69-70`
  `rows`/`cursor`, one integer cursor over a filtered queue, one id per IPC call. It is still the
  right precedent for *rows with per-item status*, so copy its shape, not its input model.
- The only per-item-status wire shape in the tree is `FileImportBatchWire`
  (`src-tauri/src/commands/project/mod.rs:2910`, built by the pure `build_file_import_batch_wire` at
  `:2927`) — a `Vec<ItemWire>` where each item carries its own error. Returned once at the end, not
  streamed, so it is a shape precedent only.

**The chord — the mockup's `⇧↓` is measurably wrong here**

- 🔴 `src/commands/keys.ts:510` drops any chord matching
  `lacksPrimaryMod(entry.mods) && isTypingZone(event.target)`, and `:415`
  `lacksPrimaryMod = (m) => !m.meta && !m.ctrl`. The grid's translation cells are **always**
  `contenteditable` (`src/panels/GridPanel.vue:1009`), so a bare `Shift+ArrowDown` would be swallowed
  exactly where the user needs it. Use `Mod+Shift+ArrowDown` / `Mod+Shift+ArrowUp`.
- Measured free: the whole repo registers only `Mod+Shift+Enter` in that family
  (`src/commands/index.ts:3480`); `Shift+ArrowLeft|Right` and `Alt+Shift+ArrowLeft|Right` belong to
  `selection.extend_*` (`index.ts:2412-2415`), and `Mod+Alt+ArrowDown` belongs elsewhere (`:2542`).
  `createKeymap` **throws** on a collision at `installCommands()` time (`keys.ts:483-489`), which
  crashes the app at startup rather than turning a gate red — verify before adding.
- Key names must exist in `NAMED_CODES` (`keys.ts:97-122`); `ArrowUp`/`ArrowDown` are there.

**Webview**

- `src/aiTranslateState.ts:39` the state union is closed at five values by 4.8's AC and stays closed.
  `:88` `runAiTranslate` refuses while `state === 'generating'`; `:155` `resetAiTranslate` cancels an
  in-flight call then bumps `sequence`. The batch gets its **own** module rather than widening this
  one — see §Design Notes.
- `src/config/aitranslate.ts:26` `import { Channel, invoke }`, `:88-92` the Channel construction and
  `invoke` idiom, `:121` `cancelAiTranslateCall()` (no args, process-wide). Adapters never throw and
  return `{ value, error }`.
- `src/panels/AiTranslationPanel.vue:232` `.ai-surface` holds 4.8's single-run block (`:233` the
  three buttons, `:263` the status lines, `:293` `.ai-translate-text`). The batch list belongs in the
  same surface. `@click` must be exactly one `dispatch('<id>')`; every text node goes through `t()`.
- `src/commands/index.ts:3456/:3467/:3478` the three 4.8 ids; `:3428-3453` records why run and cancel
  carry `keys: undefined`. `src/main.ts:942-963` is where dispatch-time guards already live.
- ⚠️ `resetAiTranslate` has exactly **two** product call sites, `modes/libraryChapters.ts:298` and
  `modes/libraryImport.ts:428`, and both are **Work** changes. Switching Chapter inside one Work
  runs `resetEditorPanel()` (`src/panels/editorPanelState.ts:1713`) and resets the Source panel, but
  touches no AI state. Do not read "the reset cluster" as covering a Chapter switch. Decision 4
  closes this for both call shapes; `tests/frontend/aiTranslate.test.ts:641-661` is where 4.8's two
  reset-cancel cases live and is the file that must grow the Chapter-path case.
- `scripts/check-panel-refs.mjs` subset rules: a `const x = ref(...)` must be assigned inside an
  exported `reset*()` in the same file; multi-declarator and two-line declarations are a hard fail.

**Gates that count, measured 2026-09-21 on this tree**

- `src-tauri/tests/config_invariants.rs:1489` `COMMAND_FILE_CENSUS: [CommandFileCensusRow; 15]`, row
  for `src/commands/aitranslate.rs` at `:1510` reading `2, 0, 0`; `:1722-1724` pins
  `(tree_plain, tree_async) == (70, 28)`. A literal `async fn` under a plain `#[tauri::command]`
  counts as **plain**. Re-count both; never adjust them until they match.
- `src-tauri/tests/ai_boundary.rs:194` `AI_TRANSLATE_SEAM_COMMAND_FILE = "commands/aitranslate.rs"`,
  `:213` marker `"crate::core::ai::client::"`, `:399`
  `ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM = ["OpenAiChatClient", "OpenAiClientError"]`. Staying in
  that file and using only those two names needs **no** gate edit; a third name or a new file does.
  Floors `:65` `AI_FLOOR = 1`, `:89` `SRC_RS_FLOOR = 68`.
- `src-tauri/tests/aiconfig_keychain_boundary.rs:256` the whole-file exemption is
  `commands/aitranslate.rs`; its doc-comment (`:251-255`) requires a **second** constant, never a
  prefix, if a second file ever reads the key. The batch keeps the read in this file, so no edit.
- `src-tauri/tests/ipc_contract.rs:2020`
  `the_ai_translate_wires_are_registered_and_keep_their_parameter_names` — extend it; `:232`
  `every_message_key_exists_in_vi_json` and `:325`
  `every_message_key_declares_the_params_its_string_needs` both run over `MessageKey::ALL`, so a new
  variant is falsely green until `vi.json` carries its key with the same `{placeholders}`.
- `src-tauri/src/core/i18n/mod.rs:100` `message_keys!` — 98 variants today (counted 2026-09-21; the
  "101" in spec 4.8's Code Map was wrong), 10 beginning `Ai`. Declaration form:
  `Variant => "err.some.key" ["param"],`.

**Tests that move**

`src-tauri/tests/ai_translate_contract.rs` · `src-tauri/tests/config_invariants.rs` ·
`src-tauri/tests/ipc_contract.rs` · new `tests/frontend/segmentSelection.test.ts` · new
`tests/frontend/aiTranslateBatch.test.ts`. `tests/frontend/aiTranslate.test.ts:57-60` is the example
that mocks the **adapter** boundary rather than `@tauri-apps/api/core`, with `pendingRun()` (`:80`)
as its fake stream; there is no precedent anywhere for mocking `Channel` itself.

## Tasks & Acceptance

Four phases, each handed to a **fresh agent** through this file (root `AGENTS.md`: one agent must
not implement a whole story).

**Execution:**

*Phase 1 — the selection model, frontend only, no AI in it*
- [x] `src/panels/segmentSelectionState.ts` (new) — anchor plus focus over the open Chapter's
      ordered segment ids; exports read-only refs, the derived selected id list and its count, the
      extend/clear actions, and `resetSegmentSelection()` — rationale: the batch has no input until
      this exists, and it must satisfy `check:panel-refs` in its own file.
- [x] `src/panels/editorSegments.ts` + `src/panels/GridPanel.vue` — carry an `isSelected` flag per
      row and mark selected rows using existing colour tokens only — rationale: a selection the user
      cannot see is not a selection; no new colour, no shadow, no floating layer.
- [x] `src/commands/index.ts` — register `segment.selection.extend_down`
      (`Mod+Shift+ArrowDown`), `segment.selection.extend_up` (`Mod+Shift+ArrowUp`) and
      `segment.selection.clear` (`keys: undefined`) — rationale: the measured typing-zone rule bars
      the mockup's bare `⇧↓`; state that reason in place.
- [x] `src/modes/libraryChapters.ts` + `src/modes/libraryImport.ts` — add `resetSegmentSelection()`
      to **both** Work-change clusters — rationale: a selection surviving a Work change points at
      segment ids that no longer exist.
- [x] `tests/frontend/segmentSelection.test.ts` (new) — extend, shrink, clear, cross a Chapter
      boundary, and the two reset clusters.

*Phase 2 — the Rust batch path*
- [x] `src-tauri/src/commands/aitranslate.rs` — add the batch event enum carrying `segment_id` per
      event, a pure `prepare_batch_call(...)` resolving config and reading the key **once** and
      classifying omitted segments, a `run_batch_call<P: TranslationProvider>(...)` seam looping in
      document order and re-checking `should_cancel()` **between** sentences, and the
      `#[tauri::command]` shell taking `Vec<i64>` plus one `Channel` — rationale: one keychain read,
      one generation, one stream, and a seam a fake provider can drive with no socket.
- [x] `src-tauri/src/core/i18n/mod.rs` + `src/i18n/vi.json` — one new `message_keys!` variant for
      "the batch stopped at this sentence", declaring `["segment_id"]`, with the matching flat
      `vi.json` string using the same placeholder — rationale: `MessageKey::ALL` drives two gates, so
      a variant without its string is falsely green.
- [x] `src-tauri/src/lib.rs` — register the new command in `generate_handler!` beside `:949-950`;
      no new managed state — rationale: the batch reuses `AiTranslateGeneration`.
- [x] `src-tauri/tests/config_invariants.rs` + `src-tauri/tests/ipc_contract.rs` — re-count the
      census row and the `(70, 28)` pair, and extend the registration test with the batch wire's
      parameter names — rationale: both numbers are asserts, not documentation.

*Phase 3 — the webview*
- [x] `src/config/aitranslate.ts` — one adapter for the batch: construct the typed `Channel`, hand it
      to `invoke`, never throw, return `{ value, error }` — rationale: the only new part of the 4.8
      idiom is the typed Channel payload.
- [x] `src/aiTranslateBatchState.ts` (new) — one row per selected sentence with its own status and
      text, the done/running/remaining counts, the run and cancel actions, a monotonic `sequence`
      guard, and `resetAiTranslateBatch()` — rationale: 4.8's state module is shaped for one run and
      its five-value union is frozen; see §Design Notes.
- [x] `src/panels/AiTranslationPanel.vue` — inside `.ai-surface`, the batch header line (how many
      done, which one is running, how many remain) and the per-sentence rows; the batch run button
      and the existing cancel button driven by whichever call is in flight — rationale: AC2 is a
      display requirement and nothing in the panel shows per-item status today.
- [x] `src/commands/index.ts` + `src/main.ts` — register `ai.translate.batch_run` with
      `keys: undefined`, reuse the existing `ai.translate.cancel` for both call shapes, and put the
      mutual-exclusion guard (a batch refuses while a single run streams, and the reverse) at the
      dispatch layer in `main.ts` — rationale: the guard cannot live in either state module without
      a two-way import cycle.
- [x] `src/modes/libraryChapters.ts` + `src/modes/libraryImport.ts` + the Chapter-switch path in
      `src/panels/editorPanelState.ts` (the `resetEditorPanel()` call at `:1713`, step ③ of a
      Chapter switch) — add `resetAiTranslateBatch()` to **all three**, and on the Chapter path add
      `resetAiTranslate()` beside it (Decision 4, Ice 2026-09-21) — rationale: a call that keeps
      streaming into a Chapter the user has left keeps spending the user's money invisibly, and the
      single-run path has the same hole. Say in the commit message that the single-run line is
      Decision 4 and not this story's AC, so the diff stays readable.

*Phase 4 — tests that move, the measurements, and the ledger*
- [x] `src-tauri/tests/ai_translate_contract.rs` — one case per I/O-matrix row, driven by a fake
      provider and a **real** `Channel` built from a collecting closure
      (`tauri-2.11.5/src/ipc/channel.rs:213` takes a plain `Fn`, no `Runtime`, no `AppHandle`) —
      rationale: including *"after cancel, no further Channel message arrives"*, which must be
      measured as an absence, not asserted as a flag.
- [x] `tests/frontend/aiTranslateBatch.test.ts` (new) — progress counts, cancel keeping finished
      rows, error stopping at the named row, and the reset clusters cancelling an in-flight batch.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — discharge the entry re-owned to this
      story at `:4517` (owner line `:4550`): measure `entries_eligible_for_injection` on a **release**
      build with a real Work open, recording **both** the one-tier and the two-tier branch with the
      population each ran on; if the two-tier per-call cost times the largest batch the UI permits
      exceeds one second of local work before the first token, cache the eligible set for the run —
      otherwise close the item with the numbers. Close the FR20 item with
      `→ KHÔNG LÀM 2026-09-21 (Story 4.9)` and Decision 3's measured reason — rationale: both items
      name this story as owner and neither closes by inference.

      🔵 **CORRECTED 2026-09-22 — this bullet names the wrong function, and its two-outcome rule did
      not fit what the right one measures.** `entries_eligible_for_injection` is what the ledger
      entry named, but `grep -rnE "entries_eligible_for_injection\s*\(" src-tauri/src/` returns
      exactly one line — its own definition at `core/glossary/store.rs:753`. Nothing calls it. The
      per-sentence path is `core/ai/rag.rs:214` inside `gather_glossary_context`, which calls
      `confirmed_terms_for_injection` (`core/glossary/store.rs:1559`), and that one also takes the
      sentence, so its cost has a second axis this bullet never anticipated. Measured on release:
      one-tier/short `1.730391ms`, two-tier/short `3.818571ms`, two-tier/long `4.844849ms` per call
      over 500 Global + 500 Work terms, 200 calls each. The rule's threshold is therefore crossed at
      N ≈ 206–262 sentences, a size Decision 1 explicitly permits — but its remedy ("cache the
      eligible set for the run") cannot be taken here, because both `assemble_prompt` and
      `gather_glossary_context` carry signatures AD-14 freezes, so caching means either a new AD or
      hidden state inside `core/glossary/store.rs`. The item is therefore **half-closed** with the
      measured numbers and both options written out for Ice, which is one of the three closings root
      `AGENTS.md` sanctions — not the "cache it" branch and not the "close with the numbers" branch
      this bullet offered.
- [x] Counter-check by removal, then restore: delete the between-sentence `should_cancel()` check and
      confirm a **named** cancel case goes red; delete the omitted-segment guard and confirm a named
      case goes red naming the column; delete the `isSelected` wiring and confirm a selection case
      goes red — rationale: a compile error is not a discharge, and an assert that holds on both
      branches guards neither.

**Acceptance Criteria:**

- Given the caret sits on a segment, when the user presses the extend chord *n* times, then *n*+1
  consecutive rows are marked selected in the grid and the AI panel reports that count.
- Given a batch is running, when the panel renders, then it shows which sentences are finished,
  which one is running, and how many remain.
- Given any AI call is in flight, single or batch, when the user invokes cancel, then the call stops
  at the current sentence boundary.
- Given a batch has finished or been cancelled, when the user promotes one of its results, then it
  goes through the existing per-sentence promote and the row is written with
  `translation_origin = 'other'`.
- Given the batch-run command and the cancel command, when the user opens the shortcuts screen, then
  both appear and can be rebound.
- Given the per-sentence Glossary lookup now runs once per sentence of a batch, when the story
  closes, then both branch costs are recorded with the build and the population they were measured
  on, and the ledger entry is discharged either way. 🔵 **CORRECTED 2026-09-22** — this criterion
  said `entries_eligible_for_injection`; the function actually on that path is
  `confirmed_terms_for_injection`, reached through `gather_glossary_context` (`core/ai/rag.rs:214`).
  See the 🔵 note on the ledger task above for the measurement and why the discharge is a half-close.
- Given a single-segment run is streaming, when the user switches Chapter inside the same Work, then
  the run is cancelled and its result cleared (Decision 4).

## Implementation Notes

### 2026-09-21 — Phase 1 (the selection model) — verified against the tree, not the report

Files: new `src/panels/segmentSelectionState.ts` and `tests/frontend/segmentSelection.test.ts`;
edits to `editorSegments.ts`, `GridPanel.vue`, `commands/index.ts`, `main.ts`, `vi.json` and the two
Work-change clusters. Zero Rust files and zero 4.8 AI-state files touched, measured by `git status`.

Chords landed as `Mod+Shift+ArrowDown` / `Mod+Shift+ArrowUp` (`commands/index.ts:2483-2484`), the
pair §Design Notes measured as the only one that survives the typing-zone rule. Both module cells
(`segmentSelectionState.ts:55-56`) are assigned inside `resetSegmentSelection()`, so
`check:panel-refs` has something real to hold.

**Counter-check by removal, run twice, restored from a scratchpad copy rather than `git checkout`
(the file carried uncommitted work):** deleting `resetSegmentSelection()` from
`modes/libraryChapters.ts` turns **1 of 12** red with the right reason — `expected 2 to be +0`, the
old Work's selection surviving — while the other 11 stay green; deleting it from
`modes/libraryImport.ts` turns a different **1 of 12** red with `expected 11 to be null`. Both files
restored byte-identical (`diff -q` silent). So each case guards its own branch, not the mere
existence of the function.

Suite on this session's own run after the mutations were undone: vitest **89 files · 1 290 passed ·
0 failed**.

⚠️ The implementing agent flagged *"retired segments are not filtered out of the selection range"*
as an open assumption. **Refuted by measurement:** the row query feeding the grid is
`… FROM segment WHERE chapter_id = ?1 AND retired_at IS NULL ORDER BY ord, id`
(`commands/segment.rs:908`), and `:236` records that `retired_at` is `None` for every row today
because no path retires a segment yet. A retired segment cannot reach the selection.

### 2026-09-21 — Phase 2 (the Rust batch path) — verified, and one blind spot the counts expose

Files: `commands/aitranslate.rs`, `core/i18n/mod.rs`, `lib.rs`, `tests/config_invariants.rs`,
`tests/ipc_contract.rs`, plus the `vi.json` string. The new wire is `ai_translate_batch`, a second
literal `async fn` command, registered at `lib.rs:958`. Cancel reuses the one
`AiTranslateGeneration` and `ai_translate_cancel` unchanged; `should_cancel()` is asked **between**
sentences at `aitranslate.rs:548` as well as inside the provider's frame loop.

Gate numbers were re-counted, not adjusted: the census row for `commands/aitranslate.rs` reads `3`
plain (`config_invariants.rs:1517-1519`) and the tree-wide pair moved to `(71, 28)` at `:1731-1732`.
That assert computes the tree's own counts and compares, so a green run is the measurement.

🔴 **The Rust suite is unchanged at 65 binaries · 1 772 passed · 0 failed — the same total 4.8
finished on.** That is arithmetically correct, because Phase 2 added no test *case*: its gate
changes are asserts inside existing cases and the `ipc_contract` work extended an existing test
function. It also means every line of the batch path is currently guarded by **nothing**. Phase 4
owns that coverage; until it lands, a green suite here says only that nothing else broke.

⚠️ **Design consequence found while reading the diff, not reported by the agent:**
`prepare_batch_call` calls `assemble_and_record_prompt` for **every** item up front
(`aitranslate.rs:399`), before the first provider call. Two effects. It is *good* for money — an
unknown segment id rejects the whole batch before a single call is billed — and it is *bad* for
latency, because all N passes through the RAG injector happen before the first token. That is
exactly the cost the ledger entry re-owned to this story measures, so Phase 4's threshold clause
(*"one second of local work before the first token"*) applies to the up-front total, not to a
per-sentence trickle.

### 2026-09-21 — Phase 3 (the webview) — verified, and the coverage debt is now the whole story

Files: new `src/aiTranslateBatchState.ts`; edits to `config/aitranslate.ts`,
`panels/AiTranslationPanel.vue`, `commands/index.ts`, `main.ts`, `vi.json`, the two Work clusters,
and `panels/editorPanelState.ts:1729-1730` where Decision 4 lands — `resetAiTranslate()` and
`resetAiTranslateBatch()` now both run on the Chapter-switch path.

Two frozen things were checked rather than assumed. `src/aiTranslateState.ts` is **unmodified**
(`git diff --stat` empty) and its five-value union still reads exactly
`'not_configured' | 'generating' | 'done' | 'error' | 'cancelled'` at `:39`, so 4.8's accepted AC is
intact; the batch's per-row status is a **different axis**, not a sixth panel state. And there is no
import cycle: `aiTranslateBatchState.ts` imports only `vue`, `config/aitranslate` and the `IpcError`
type, while `aiTranslateState.ts` names the batch module zero times — the mutual-exclusion guard
sits in `main.ts`, where the spec put it.

The implementing agent also caught itself on a real contradiction and said so: its first draft wiped
the failing row's partial text on a provider error, which would have contradicted both §Always
(discard is promised for **cancel** only) and 4.8's existing case
`a_provider_error_after_partial_tokens_leaves_earlier_tokens_visible_and_propagates_the_error`. Only
cancelled rows lose their text.

One thing was built that no task bullet named: `main.ts`'s promote handler now falls back to the
batch rows when the single-run module has nothing for the caret segment. It is not scope creep —
without it AC4 cannot pass, because batch results live in a module the 4.8 handler never read.

Measured on this session's own run, not taken from the report: `check:panel-refs` · `check:commands`
· `check:i18n` · `check:tokens` · `check:lint` all PASS, `npm run build` PASS (run explicitly,
because 4.8 spent several rounds green on vitest while `build` was red), vitest **89 files · 1 290
passed · 0 failed**.

🔴 **That total has not moved since Phase 1.** Phase 2 added no case and Phase 3 added no case, so
across the whole story only the 12 selection cases guard anything. Every line of the Rust batch
path, the adapter, the batch state module, the panel and the promote fallback is currently
unguarded. Phase 4 is therefore not a tidying step — it is where the story becomes checkable, and it
is split into 4a (Rust) and 4b (webview, ledger, counter-checks) so neither agent carries the whole
surface.

### 2026-09-21 — Phase 4a (the Rust contract cases) — accepted on its removals, not its report

`ai_translate_contract.rs` went 30 → 40 cases; full Rust suite 1 772 → 1 782, measured here, delta
exactly the ten new cases. Both counter-checks were real removals with the product file backed up to
the scratchpad first (`git checkout` would have destroyed Phases 1-3's uncommitted work) and both
reds were read rather than counted:

- removing the between-sentence `should_cancel()` reddened
  `cancelling_exactly_between_two_sentences_never_calls_the_provider_for_the_next_sentence` on
  *"MultiItemProvider bi goi lan thu 3 nhung chi cap 2 cong thuc"*;
- removing the `is_omitted` guard reddened
  `an_omitted_segment_inside_the_selection_is_classified_as_omitted_not_to_translate` on
  *"hang thu 3 (segment_id=3, is_omitted=true) phai la PreparedBatchItem::Omitted"*.

The cancel case earns its keep by construction: it fires cancel from **inside the real Channel's
collecting closure** the instant sentence 2's `Done` lands, so 4.8's intra-provider `should_cancel()`
cannot be what catches it. Only the between-items check can — the two nudges are told apart instead
of being asserted together. Verified independently afterwards: `aitranslate.rs` is back to
`460 insertions / 24 deletions`, `should_cancel` still appears 14 times, the `is_omitted` guard is
back.

### 2026-09-22 — Phase 4b — one measurement rejected, redone, and the verdict changed with it

Landed: `batch_stopped_error` made `pub` (the minimum change, exactly the reason 4.8 made
`prepare_translate_call` and `run_translate_call` `pub`) plus the case that asserts the real
`IpcError` — code, `MessageKey`, `segment_id` param, and `retryable` on both a retryable and a
non-retryable variant; `tests/frontend/aiTranslateBatch.test.ts` with six cases; one case added to
`segmentSelection.test.ts` for the `isSelected` wiring, which the agent found had **zero** coverage
anywhere in the tree, proved by gutting `selectedRowClassById`'s loop and reading the red
(`expected false to be true` on the `row-selected` class).

🔴 **Its first measurement was rejected.** It measured `entries_eligible_for_injection` — the
function the ledger entry names, and a function nothing calls:
`grep -rnE "entries_eligible_for_injection\s*\(" src-tauri/src/` returns one line, its own
definition at `core/glossary/store.rs:753`. The per-sentence path is `core/ai/rag.rs:214` →
`confirmed_terms_for_injection` (`core/glossary/store.rs:1559`). Root `AGENTS.md` names this exact
failure: a number measured on something no production line calls is not a number about the product.
Sent back with the three greps; it reproduced them and redid the work.

The corrected numbers changed the conclusion rather than decorating it. On release, 500 Global + 500
Work terms, 200 calls per combination, asserting row counts and **never** elapsed time:
`one_tier_short 1.730391ms` · `two_tier_short 3.818571ms` · `two_tier_long 4.844849ms` per call. The
right function also takes the sentence, so length is a second real axis (+27% from a 9× longer
sentence). The spec's own threshold then lands at N ≈ 206–262 sentences instead of the wrong
function's N ≈ 323 — a batch size Decision 1 explicitly permits. So the item is **half-closed**, not
closed, with both remedies written out and their numbers, because the remedy the task bullet named
runs into AD-14: `assemble_prompt` and `gather_glossary_context` both carry frozen signatures, so
caching is a new AD or hidden state in `core/glossary/store.rs`, not a line of code. See the 🔵 note
on that task bullet. FR20's closure never depended on this number and stands on Decision 3.

The probe now carries `#[ignore]`, so the default suite pays nothing for a timing measurement whose
debug numbers would be meaningless — `cargo test --test ai_rag_contract` is 34 passed, 1 ignored.

Suites measured on this session's own still tree: Rust **65 blocks · 1 783 passed · 0 failed · 22
ignored** (1 782 + the one new case, and the probe moving into `ignored`), vitest **90 files · 1 297
passed**, `npm run build` PASS, `check:debt-owner` PASS.

### 2026-09-22 — Gap pass: the Matrix Test Audit was red, and one trap was the textbook one

Running step-03's Matrix Test Audit against the tests that actually existed turned up four holes.
Three matrix rows had no covering case at all — *"Selection changes while a batch runs"*, *"Empty
selection"*, and the Chapter half of *"Work or Chapter changes while an AI call runs"* — and the
Decision 4 acceptance criterion had none either.

The fourth is the one worth naming. `aiTranslateBatch.test.ts` did have reset cases, but they called
`resetAiTranslateBatch()` **directly**, which guards the function and says nothing about whether any
real call site invokes it. Root `AGENTS.md` records this exact shape from Story 6.15, where all six
real calls were deleted from `mod wire` and the suite stayed 15/15 green. All three call sites could
have been deleted here and every case would have stayed green.

Closed with `tests/frontend/aiTranslateBatchResetWiring.test.ts` (new, four cases) plus three cases
appended to `aiTranslateBatch.test.ts`. Frontend went **90 files · 1 297** → **91 files · 1 304**,
delta exactly the seven new cases.

Each wiring case was proved by removing its own line and restoring from a scratchpad copy. The
discriminating detail is that the two adjacent lines — `resetAiTranslate()` at
`editorPanelState.ts:1729` and `resetAiTranslateBatch()` at `:1730` — were removed **separately**,
and each time the sibling case stayed green. An assert that holds on both branches guards neither;
these two hold on exactly one line each.

Re-run independently by the orchestrator rather than taken from the report: deleting line 1729 alone
turns **1 of 4** red — `🔴 Quyết định 4 (AC7 spec 4.9) …` on `expected "vi.fn()" to be called 1
times, but got 0 times` — while the batch sibling and the two Work-cluster cases stay green.
Restored byte-identical (`diff -q` silent), 4/4 green again, and
`git diff --stat src/panels/editorPanelState.ts` back to its Phase 3 value of 17 insertions.

### 2026-09-22 — Patch round after review, verified against the tree

Seven findings routed to patch, handed back to the agents that wrote each file so no two touched the
same one, and the two `deferred-work.md` items were done by the orchestrator for the same reason.

Landed: the reseed guard in `segmentSelectionState.ts` now resolves the anchor's index the same way
focus's is resolved, so a merge or split that retires the anchor's row no longer leaves the extend
chord moving an invisible focus over a permanently empty selection; `canPromoteAiTranslate` now reads
the same pure helper `main.ts`'s handler uses, so the button and `Mod+Shift+Enter` stop disagreeing;
three duplicated CSS blocks joined the selector groups they were copied from; the `retired_at` half
of a new doc-comment was corrected in place against `commands/segment.rs:908`; and four test cases
closed the promote-fallback and mutual-exclusion gaps plus two Rust cases for `prepare_batch_call`'s
error branches.

One agent found a real defect in its own new test while writing it — `promoteMock` was never reset in
`freshPanel()`, so the negative case was inheriting the previous case's call and passing for the
wrong reason. Another flagged a 13th case in a file it did not write rather than deleting or
silently keeping it; it belonged to the earlier gap pass.

Counter-check run by the orchestrator, not taken from a report: deleting the new `anchorIndex === -1`
clause turns **1 of 14** red — the named case — on `expected [] to deeply equal [ 12, 13 ]`, which is
the defect's own symptom, while the other 13 stay green. Restored byte-identical, 14/14 again.

Final measurement on the patched, still tree: eleven gates PASS · vitest **91 files · 1 309 passed**
· `npm run build` PASS · `cargo test --locked` **65 blocks · 1 785 passed · 0 failed · 22 ignored**.
Both deltas close exactly (+5 frontend cases, +2 Rust cases).

⚠️ **Nightly read before writing `done`, as this repo requires.** The latest completed `schedule` run
is `35534874121` (2026-09-20) and it is **red**, at `check (windows-2025)` → a panic at
`tests\store_contract.rs:890` — the same intermittent WAL-threshold case Story 4.8 recorded, which
changes platform and changes night. Nothing in this story touches `store_contract` or the WAL path.
The 2026-09-21 nightly had not appeared yet at the time of writing. Recorded rather than absorbed.

## Spec Change Log

## Review Triage Log

### 2026-09-22 — Pass 1, three layers over a 237 kB diff (blind-hunter 11 · edge-case 6 · verification-gap 2)

No `intent_gap` and no `bad_spec`, so no loopback.

| # | Finding | Verdict | Evidence | Route |
|---|---|---|---|---|
| 1 | `moveSegmentSelectionFocus` reseeds on a stale `focus` but never on a stale `anchor` | **medium** | Verified: the guard at `segmentSelectionState.ts:115` tests `anchor === null \|\| focus === null \|\| focusIndex === -1`, while `segmentSelectionIds:79-80` returns `[]` unless **both** indices resolve. Merging or splitting a segment (Stories 2.8/2.9 retire the old rows) can drop the anchor's id while focus survives, and from then on the extend chord moves `focus` invisibly and the selection reads empty with no error — this project's named central failure class | patch |
| 2 | `canPromoteAiTranslate` was not extended for batch results | **medium** (same root cause as #3) | Verified: `AiTranslationPanel.vue:172-176` reads only the single-run state, and `:313` binds the button's `:disabled` to it, while `main.ts`'s handler does promote a `done` batch row at the caret. The chord works and the button does not — the same command, two answers | patch |
| 3 | The batch-fallback branch of `promoteAiTranslate` has no test | **medium** | Pre-verified by the gap layer and re-checked: `aiTranslateBatch.test.ts` stubs `promoteAiTranslate` as a no-op claiming coverage in `aiTranslate.test.ts`, which this diff never touches and which carries no batch reference. Swapping `caretId` for the stale run id would promote the wrong sentence with the suite green | patch |
| 4 | The mutual-exclusion guard between a single run and a batch has no test | **medium** | Pre-verified by the gap layer: `aiTranslateBatch.test.ts` copies both guards into its own stub but never dispatches `ai.translate.run` while the batch streams, and `runSegmentMock` is never asserted against. Deleting either cross-check would let two calls race on one generation counter, green | patch |
| 5 | `prepare_batch_call`'s `segment_not_in_chapter` and `keychain_unavailable` branches are untested on the batch path | **low** | Verified: both branches exist in `prepare_batch_call`, and Phase 4a's cases cover those error kinds only for the single-segment path. The fix is two test cases and no product change | patch |
| 6 | The new doc-comment claims retired segments reach the selection layer | **low** | Verified, and measured independently earlier in this build: `commands/segment.rs:908` is `… WHERE chapter_id = ?1 AND retired_at IS NULL ORDER BY ord, id`, and `:236` records that nothing retires a segment yet. The `is_omitted` half of the sentence is true; the `retired_at` half is false | patch |
| 7 | `.ai-translate-batch-run` duplicates the existing button rule block verbatim | **low** (same root cause as #8) | Verified: byte-identical to the `.ai-translate-run, .ai-translate-cancel, .ai-translate-promote` group instead of joining its selector list. The fix is a merge, a direct simplification, not added complexity | patch |
| 8 | `.ai-batch-selection-hint, .ai-batch-progress, .ai-batch-running` duplicates `.ai-translate-hint, .ai-translate-status` | **low** | Verified: same `margin`/`font-family`/`font-size`/`line-height`/`color`. Same root cause as #7 — new styles written standalone rather than joining the group they match | patch |
| 9 | The ledger's Big-O for `find_terms` omits a factor | **low** | Verified: `core/matching/mod.rs:470+` walks each term across the text with `start + term.len()` inside the per-term loop, so the shape carries a per-term length factor the entry's "terms × sentence tokens" leaves out. The fix corrects a sentence in `deferred-work.md`, not in this build's spec | patch |
| 10 | The `last_sent` / `mark_prompt_as_sent` logic inside `wire::ai_translate_batch` is untested | **medium** | Verified: only `ipc_contract.rs` names the wire, and that is a registration and parameter-name check. Real, and the same shape 4.8 already deferred as its finding #15 for the single-segment path — reaching it needs an `AppHandle`-level harness this repo has no precedent for, which is more than a patch | defer |
| 11 | The batch row list is un-virtualised and unmeasured at the N this story refuses to cap | **medium** | Verified: `<li v-for="row in aiTranslateBatchRows">` with no windowing, and the backing array is replaced on every streamed event. Real asymmetry against the backend cost this same build measured at N≈184-262. Its fix is a measurement or a virtualiser, neither trivial | defer |
| 12 | An error with no `segment_id` param strands the running row at `running` | **low** | Verified in code: `aiTranslateBatchState.ts` only repaints a row when `Number(params.segment_id)` parses, and `batch_panicked_error()` carries none. But the only error that reaches it after streaming starts is a `JoinError`, and `panic = "abort"` makes a panic process death rather than an observable join failure; the two state-missing guards are setup errors their own comments call unreachable on the product path. Unlikely in everyday use, and the fix adds a branch | rejected |
| 13 | `prepare_batch_call` silently translates fewer rows when `segment_ids` repeats an id | **low** | Verified as code behaviour, but the only producer is `segmentSelectionIds`, which returns `ids.slice(lo, hi + 1)` of a distinct ordered list — a duplicate cannot arise through the product, only through a hand-made IPC call. The fix adds a guard | rejected |
| 14 | The frontend row-index `Map` loses a duplicated id the same way | **low** | Same refutation as #13 — the selection cannot emit a duplicate — and the same added-guard fix | rejected |
| 15 | An empty `segment_ids` reaches Rust as `Done` with zero events instead of a signal | **low** | Verified as behaviour; the frontend refuses an empty selection twice over (the disabled button and `runAiTranslateBatch`'s own length check, both now tested), so the state is unreachable through the product. The fix adds a branch | rejected |
| 16 | §Always says a batch "supersedes" an in-flight single run, but the handlers refuse instead | **false** | Refuted: `wire::ai_translate_batch` does call `generation_state.next()`, so at the layer that bullet describes a batch genuinely supersedes an in-flight run. The frontend guard means the user never reaches that state — a stricter enforcement of the same "one AI call in flight process-wide" invariant, not a contradiction of it | rejected |
| 17 | Blind layer's duplicate of #1 | **medium** | Same finding, same location, folded into entry #1 | patch |
| 18 | Blind layer's duplicate of #3 | **medium** | Same finding, folded into entry #3 | patch |
| 19 | Blind layer's duplicate of #2 | **medium** | Same finding, folded into entry #2 | patch |

## Design Notes

**Why a separate batch state module instead of widening `aiTranslateState.ts`.** That module holds
one `runSegmentId`, one `accumulatedText` and one five-value state, and the five values are fixed by
4.8's accepted AC — 4.8's review already rejected a sixth value for that reason. A batch needs a row
per sentence with its own status, which is a different shape, not a bigger one. Keeping them apart
also keeps 4.8's 17 cases meaningful instead of rewriting them around a generalised container.

**Why one Rust batch command rather than a frontend loop over `ai_translate_segment`.** A loop in the
webview would re-resolve config and re-read the OS keychain once per sentence, would put N
independent cancel windows where the product promises one, and would still need an envelope on the
stream to tell the rows apart. One command reads the key once, holds one generation, and makes
*"stop at the current sentence boundary"* a property of a single loop instead of a race between the
webview and the backend.

**Why the extend chord carries `⌘`.** Measured, not preferred: `keys.ts:510` drops a chord with no
`meta`/`ctrl` when the focused element is a typing zone, and the grid's translation cells are always
`contenteditable`. The mockup's `⇧↓` would therefore be dead in the exact place it is meant to be
used. Horizontal `Shift+Arrow` stays text selection inside a cell; vertical `Mod+Shift+Arrow` becomes
segment range selection — the two do not collide and the split reads sensibly.

## Verification

**Commands:**
- `npm run build` **before** any `cargo test` — without `dist/`, `cargo test` fails at compile time.
- `cd src-tauri && cargo test --locked` — measure the baseline on this story's own starting commit
  (`2376aebbc713cd80beea3ecd64e2fba68919f904`) in a still tree first; do not inherit 4.8's counts.
- `npx vitest run` — 0 red; the case count rises by the two new frontend files only.
- `npm run test:story 4-9` for the scoped dev loop; it reports THIẾU KHAI for any test file the diff
  touched that this spec never named.
- The eleven `pre-push` gates individually: `check:deps` · `tokens` · `i18n` · `commands` · `layout` ·
  `panel-refs` · `dict` · `dict-manifest` · `lint` · `gates` · `debt-owner`.
- ⚠️ The real `pre-push` order is eleven gates → vitest → **`npm run build`** → `cargo test`. Running
  the gates and `cargo test` while skipping `build` is what hid a red `vue-tsc` for several rounds in
  4.8; run all fourteen before calling a phase done.
- `cargo test --test ai_boundary --test aiconfig_keychain_boundary --test config_invariants` after
  Phase 2, and read **why** each red fires rather than its colour.

**Manual checks:**
- Start the app and press the extend chord with the caret inside a grid translation cell. If nothing
  happens, the typing-zone rule ate it — that is the measured trap in §Design Notes, not a wiring bug.
- Read the latest nightly `schedule` e2e run before writing `done`; if it is red, write the reason
  down. ⚠️ 4.8 inherited an intermittent red in
  `store_contract::the_wal_stops_growing_once_it_crosses_the_threshold` that changes platform and
  changes night; re-read the run at the end rather than assuming the same red.
