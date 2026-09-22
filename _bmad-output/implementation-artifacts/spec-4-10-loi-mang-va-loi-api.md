---
title: 'Story 4.10 — Network errors and API errors'
type: 'feature' # feature | bugfix | refactor | chore
created: '2026-09-22'
status: 'done' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
review_loop_iteration: 0
baseline_commit: '915f1af6a84ff81b1184cadd26c00dccd551d317'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-4-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR75 asks for three things. Two are already true and must not be rebuilt: an AI
error never touches the Editor (measured — `editorPanelState.ts:308` is the sole writer into
segment state and both its gates are promote-only, `main.ts:1004`/`:1011`), and nothing retries
automatically (there is no retry code of any kind). The third is missing. All seven
`OpenAiClientError` variants collapse into ONE `message_key`, so the user reads the same sentence
whether the network dropped, the provider refused the key, or the provider sent unreadable bytes —
`err.ai_translate.provider_call_failed`, `vi.json:77`. And there is no retry button: `retryable`
is classified per cause in Rust (`commands/aitranslate.rs:104-113`) and crosses IPC intact, yet
nothing in `src/**` branches on it. `core/i18n/mod.rs:776-779` already records in writing that
this story owns the per-cause copy and the retry affordance.

**Approach:** Split the single provider-error key into one key per cause family, so the
`message_key` itself carries the cause and `tError()` needs no new machinery. Add a retry command
for each path — single segment and batch — shown only when `retryable` is true, and never fired by
anything but a click or a chord. Retry needs no new Tauri command: `ai_translate_segment` already
takes one `segment_id` and `ai_translate_batch` already takes `Vec<i64>`, so a batch retry is the
existing command called again with the sentences that never ran.

## Boundaries & Constraints

**Always:** A retry is a user action. `retryable` grants only the right to SHOW a button
(`src/i18n/index.ts:31-32`) — no timer, no loop, no automatic second call at any layer (AD-22,
FR75: with BYOK every call is the user's money). Errors keep crossing IPC as
`IpcError::new(code, message_key, params, retryable)` and only through that constructor. Copy names
the provider or this machine, never the user. Text already streamed stays on screen through an
error, both paths (already true — `aiTranslateState.ts:117` returns without touching
`accumulatedText`; `aiTranslateBatchState.ts:194` patches status only). Every new key is declared
in `message_keys!` with the params its `vi.json` sentence interpolates, or `ipc_contract.rs:325`
goes red. Rust string literals stay unaccented.

**Never:** No auto-retry, no reconnecting SSE client, no retry that re-sends a sentence already
finished. No change to `ports/translation_provider.rs` — the trait is frozen. No new
`#[tauri::command]`: the census row `("src/commands/aitranslate.rs", 3, 0, 0)`
(`config_invariants.rs:1517-1520`) and the pinned `(tree_plain, tree_async) == (71, 28)` (`:1731`)
must both stay untouched, and a story that needs a new command has taken a wrong turn. No widening
of `promote_ai_translation` (see Decision 1). No error-copy catalogue for causes this code cannot
distinguish.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Provider unreachable, single | `RequestFailed` / `ReadFailed` | Cause sentence "could not reach the provider", retry button shown | `retryable: true` |
| Provider refuses, single | `NonSuccessStatus{status}` | Sentence names the refusal and the status, NO retry button | `retryable: false`, `{status}` |
| Stream cut before `[DONE]` | `StreamEndedWithoutDone` after N tokens | Explicit error, the N tokens stay visible, retry shown | `retryable: true` |
| Provider reply unreadable | `MalformedEvent` / `BufferOverflow` | Sentence says the reply could not be read, no retry | `retryable: false` |
| This machine cannot build the client | `ClientBuildFailed` | Sentence names this machine, not the provider, no retry | `retryable: false` |
| Saved key cannot enter a header | `HeaderValue::from_str` fails (`client.rs:245`) | Own sentence pointing at the saved key, NO retry | `retryable: false` — today `true` |
| Error with zero tokens received | provider fails before the first token | Error state, empty result area, no stale text | untested today, both paths |
| Batch stops at sentence 6 of 12 | provider error on row 6 | Row 6 `error` keeping its partial text, 1-5 keep results, 7-12 stay `pending`, retry runs 6-12 only | `retryable` from the same table |
| Batch retry after a cancel | rows 1-5 `done`, 6 `cancelled`, 7-12 `pending` | NO retry affordance — a cancel produces no `IpcError`, so nothing grants the button; the user reselects the unrun sentences and runs the batch again | no error |
| Non-2xx inside a batch | `NonSuccessStatus{status}` on row 6 | Status survives to the panel | today `batch_stopped_error` drops `{status}` |

## Decisions

**Decision 1 — bulk promote stays out, recorded as owned debt.** The mockup's
"Đưa 5 câu đã xong sang" would write several segments per click through
`promote_ai_translation`, which writes `target_text` with no `segment_version` row. AD-47 requires
every non-typing write to `target_text` to set the comparison baseline and the origin column in the
same operation, so the button cannot exist before that row does. The mockup is reference; the
measured 🔴 in spec 4.9 §Code Map wins. Promote stays one sentence per user action.

**Decision 2 — six cause keys, and the row list keeps naming the stopped sentence.** One
`IpcError` carries exactly one `message_key`, so the key carries the CAUSE and the batch row list
built in 4.9 carries WHICH row failed. No per-path duplicate of the family. The six families:
unreachable provider (`RequestFailed`, `ReadFailed`, retryable) · provider refused the request
(`NonSuccessStatus`, not retryable, `{status}`) · stream cut before `[DONE]`
(`StreamEndedWithoutDone`, retryable) · reply could not be read (`MalformedEvent`,
`BufferOverflow`, not retryable) · this machine could not build the client (`ClientBuildFailed`,
not retryable) · the saved key cannot enter a header (new variant split out of `RequestFailed`,
not retryable).

**Decision 3 — the `AppHandle` test harness stays with Ice, both halves.** The ledger's two
entries describe one missing harness; this story re-defers the batch half with that reason and a
pointer to its twin, rather than building test infrastructure the repo has no precedent for.

</frozen-after-approval>

## Code Map

**Rust — the error surface, and the six families it already distinguishes internally**

- `src-tauri/src/core/ai/client.rs:94-119` `OpenAiClientError`, seven variants. `:245` is the
  `HeaderValue::from_str` site that must become its own variant — its `detail` is
  `e.to_string()` on `InvalidHeaderValue`, whose `Display` is "failed to parse header value" and
  carries no value, so no key leaks today; keep it that way.
- `src-tauri/src/commands/aitranslate.rs:120-134` `impl From<OpenAiClientError> for IpcError` —
  one key for all seven, `{status}` attached only for `NonSuccessStatus` (`:123-125`). `:104-113`
  `openai_client_error_is_retryable`. ⚠️ `:95-99`'s doc-comment claims all four non-matrix
  branches are retryable; the code makes `ClientBuildFailed` and `MalformedEvent` false and never
  mentions `BufferOverflow`. Fix the comment in place with 🔵, do not copy it.
- `:150-155` `batch_stopped_error(segment_id, err)` — reuses the same retryable classification and
  **drops `{status}`**. `:161-166` `batch_panicked_error()` synthesises `RequestFailed`, so it
  reads as retryable.
- `:225-295` `prepare_translate_call`, `:345-431` `prepare_batch_call`, `:456-476`
  `run_translate_call` — all sync, owned data, no lock across an await. Reuse, do not reshape.
  `:422-424` is an `.expect` guarded by `needs_translation` (`:389`); under `panic = "abort"` leave
  that guard alone.
- `:726-737`, `:763-773`, `:824-835`, `:861-871` — four state-missing errors shipping
  `MessageKey::Unknown`, so two `code`s have no catalogue key. Their comments call them
  unreachable on the product path; 4.9's review pass already rejected a finding on this branch as
  unreachable. Record, do not chase.
- `src-tauri/src/core/i18n/mod.rs:100-799` `message_keys!` — **104 variants, 11 beginning `Ai`**
  (counted 2026-09-22 on this tree). ⚠️ Count it the way this spec did: five declarations put
  `=>` at the END of the line (`:371`, `:379`, `:387`, `:714`, `:722`), so any one-line
  `=> "err.` pattern reports 99 and misses them. The "98" in spec 4.9's Code Map and the "101" in
  spec 4.8's are both that mistake. `:780` `AiTranslateProviderCallFailed` declares `[]` while
  `{status}` is really attached — adding `"status"` to its required params would break the six
  variants that send `{}`, which is why the families need separate keys. `:849-872` `IpcError`,
  four private fields, sole constructor `:893-930` (a missing required param downgrades
  `message_key` to `Unknown` in release, silently).

**Webview — where the error already lands, and the one thing missing**

- `src/aiTranslateState.ts:39` the five-value union stays closed. `:114-118` the single transition
  into `error`, storing the whole `IpcError` (`:44`). `:89` the run guard refuses only while
  `generating`, so an error does not block a new call.
- `src/aiTranslateBatchState.ts:58` the six row statuses, `:182-196` the batch error transition,
  `:191` reads `params.segment_id` to mark the row — the one param a panel already consumes.
  `:119` `aiTranslateBatchTextForSegment` returns `null` unless the row is `done`, which is what
  keeps a failed row out of the Editor. A retry needs the ids of rows that are `pending` plus the
  `error` row, in the frozen list order.
- `src/config/aitranslate.ts:40-51` the runtime `IpcError` shape check, `:102`/`:175` pass the
  error through unflattened, `:57-62` the synthetic `UNKNOWN_IPC_ERROR`. Adapters never throw.
- `src/panels/AiTranslationPanel.vue:349-352` the single-run alert (`tError`, `role="alert"`, and
  ⚠️ no `data-*` hook, unlike its three sibling state lines), `:404-412` the batch alert (which
  has one). `:303-329` the three single-run buttons, `:377-385` the batch run button — a new button
  joins the existing rule group rather than copying it; 4.9's review raised two findings on
  exactly that duplication. `:163-168` `canRunAiTranslate`, `:179-188` `canPromoteAiTranslate`.
- `src/commands/index.ts:3535-3595` the four `ai.translate.*` ids; `:3520-3533` records why run and
  cancel carry `keys: undefined` and which chords collide. `src/main.ts:970-1031` holds the
  dispatch-time guards and the single/batch mutual exclusion.
- ⚠️ `src/panels/editorPanelState.ts:270-274` exports `editorPromoteAiTranslationError` and
  nothing in `src/**` reads it, so a failed promote is silent — this story's Decision 4.

**Gates that count, measured 2026-09-22 on this tree**

- `config_invariants.rs:1496` `[CommandFileCensusRow; 15]` (array length breaks only on a new
  command FILE), `:1517-1520` the aitranslate row `3, 0, 0`, `:1731` `(71, 28)`. No new Tauri
  command ⇒ all three stay as they are. ⚠️ `:1197`'s doc-comment still says 53/26.
- `ai_boundary.rs:399` `ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM: [&str; 2]` — naming a third item
  under `crate::core::ai::client::` inside the command file needs this array to grow. `:65`
  `AI_FLOOR = 1`, `:89` `SRC_RS_FLOOR = 68`.
- `ipc_contract.rs:232` `every_message_key_exists_in_vi_json` (one direction only — an orphaned
  `vi.json` key is NOT caught), `:325` `every_message_key_declares_the_params_its_string_needs`
  (both directions, per key), `:266` no duplicate keys. ⚠️ `:2026`
  `the_ai_translate_wires_are_registered_and_keep_their_parameter_names` checks an INCLUSIVE list —
  it does not count, so it stays green on an addition. It is not a net.
- `check-commands.mjs` — a new command id needs `command.<id>` in `vi.json` (`:994`); the floors
  (`COMMAND_FLOOR = 52`, `CLICK_FLOOR = 27`, `DISPATCH_FLOOR = 40`) are lower bounds, so adding
  never reddens them. `@click` must be exactly one `dispatch('<id>')`.
- `check-i18n.mjs` Check D bans the words `chúng tôi` and `bạn` in `vi.json` (`:1198`) — the
  no-blame rule is already a gate, and the copy must pass it.
- `check-panel-refs.mjs` — a module-level `ref(...)` must be assigned inside an exported `reset*()`
  in the same file; a two-line declaration or two declarators on one line is a hard fail.

**Coverage today, and the holes this story must not inherit**

`ai_translate_contract.rs` covers mid-stream failure on both paths (`:895`, `:1676`), keychain
(`:434`, `:1528`), not-configured (`:373`, `:407`, `:1437`), not-in-chapter (`:534`, `:1476`),
cancel (`:870`, `:1618`, `:1811`), and the full seven-variant mapping (`:964` — ⚠️ its
doc-comment at `:960` says six, the array is seven). Holes: **a provider failure BEFORE the first
token is untested on both paths** (every fake error case emits at least one token first),
`work.none_open` is untested on the batch path, and `:1733` checks only two of the seven variants
for batch. Frontend: `aiTranslate.test.ts:305` and `aiTranslateBatch.test.ts:346` already assert
that streamed text survives an error and that the alert renders — extend those files, do not
restate them.

**Tests that move**

`src-tauri/tests/ai_translate_contract.rs` · `src-tauri/tests/ipc_contract.rs` ·
`tests/frontend/aiTranslate.test.ts` · `tests/frontend/aiTranslateBatch.test.ts`

## Tasks & Acceptance

**Execution:**

- [x] `src-tauri/src/core/ai/client.rs` -- split the `HeaderValue::from_str` failure at `:245` out
      of `RequestFailed` into its own `OpenAiClientError` variant -- a saved key that cannot enter
      a header fails identically every time, so sharing a bucket with a dropped connection is what
      makes it wrongly retryable (ledger §"Deferred from: bmad-build review — spec 4-7").
- [x] `src-tauri/src/core/aiconfig/keychain.rs` -- keep `map_err(|_| KeychainUnavailable)` as the
      returned error but log the `keyring::Error` variant NAME first, unaccented, on the
      diagnostic path only (Decision 5) -- today a real keychain failure cannot tell "the user
      pressed Deny on the OS prompt" from "no platform store could be created".
- [x] `src-tauri/src/core/i18n/mod.rs` -- declare the six cause keys of Decision 2, each
      with exactly the params its sentence interpolates; the non-2xx key takes `["status"]` --
      `ipc_contract.rs:325` compares both directions per key.
- [x] `src-tauri/src/commands/aitranslate.rs` -- map each `OpenAiClientError` variant to its
      family key in `From<OpenAiClientError> for IpcError`; give `batch_stopped_error` the
      `{status}` it drops today; correct the `:95-99` doc-comment in place with 🔵 and a date --
      it names two branches retryable that the code makes false and omits `BufferOverflow`.
- [x] `src/i18n/vi.json` -- one sentence per new key plus the `command.<id>` label for each new
      command id -- every sentence names the provider or this machine; `check-i18n.mjs` Check D
      already bans `bạn`.
- [x] `src/aiTranslateBatchState.ts` -- expose the ids a retry would run: the `error` row plus
      every `pending` row, in frozen-list order -- a retry must never re-send a `done` sentence,
      which is the whole reason the user pressed stop rather than losing the work.
- [x] `src/commands/index.ts` + `src/main.ts` -- register a retry id for each path with its
      dispatch-time guard, refusing while either module is `generating` and when `retryable` is
      false -- the mutual-exclusion gate already lives at `main.ts:970-975`.
- [x] `src/panels/AiTranslationPanel.vue` -- render the retry button only when the stored
      `IpcError.retryable` is true, joining the existing button rule group rather than copying it;
      give the single-run alert the `data-*` hook its three siblings have; ~~surface
      `editorPromoteAiTranslationError` through `tError()` (Decision 4)~~ -- `@click` is exactly one
      `dispatch('<id>')`.
- [x] `src-tauri/tests/ai_translate_contract.rs` -- one case per I/O-matrix row, including the two
      holes measured above: a provider failure with ZERO tokens received on both paths, and
      `work.none_open` on the batch path; extend `:1733` to all variants -- then counter-check by
      REMOVING the retryable classification and confirming the OLD cases go red for the right
      reason.
- [x] `tests/frontend/aiTranslate.test.ts` + `tests/frontend/aiTranslateBatch.test.ts` -- the retry
      button appears only for a retryable cause, a batch retry dispatches exactly the unrun ids,
      and a `cancelled` row is not offered as an error -- assert the ids passed, not just that a
      call happened.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- discharge or re-defer, in words,
      the three entries naming this story, per Decisions 1, 3 and 5 -- cite
      §SECTION NAME, never a line number.

**Acceptance Criteria:**

- Given an AI error of any of the six families, when it reaches the panel, then the sentence on
  screen distinguishes that family from the other five, and names the provider or this machine
  rather than the user.
- Given a `retryable: false` cause, when the error renders, then no retry affordance is offered at
  all — the flag is the only thing that may grant it, and nothing else may.
- Given any error, when the user does nothing, then no second provider call is made by any layer,
  ever; and when the user presses retry, then exactly one new call is made.
- Given a batch that stopped at sentence N of M, when the user presses retry, then the provider is
  called for sentence N and the sentences after it that never ran, and for none of the sentences
  already finished.
- Given an error during a batch, when the panel updates, then the Editor's Translation column and
  caret are unchanged — asserted, not assumed, because the only writer into segment state is
  promote and its gates are unchanged by this story.
- Given a `cargo test --locked` run on this story's baseline commit in a still tree, when the
  counts are compared after the change, then the census row `3, 0, 0` and the tree total
  `(71, 28)` are unchanged, because this story adds no `#[tauri::command]`.

## Implementation Notes

### Phase 1 (Rust: six cause families + repair) — 2026-09-22

**Changed:**

- `src-tauri/src/core/ai/client.rs` — split `OpenAiClientError::ApiKeyHeaderInvalid { detail:
  String }` out of `RequestFailed` (Task 1). The single call site was `HeaderValue::from_str` at
  the old `:245`; only that `.map_err` moved to the new variant, nothing else in `translate()`
  changed. `Display` impl carries a matching unaccented arm
  (`ai_client[api_key_header_invalid] detail=...`).
- `src-tauri/src/core/aiconfig/keychain.rs` — added `keyring_error_variant_name` (11-arm match
  over `keyring::Error`, `#[non_exhaustive]` so a wildcard `_ => "Unknown"` closes it) and
  `log_keyring_error(context, err)`, which `eprintln!`s the variant NAME only (never the
  variant's payload/message) before every one of the five `map_err(|_| KeychainUnavailable)`
  sites (`entry`, `set`, `delete`, `configured`, `read`). The returned error type is byte-for-byte
  unchanged (Task 2, Decision 5) — this is a diagnostic-only change, nothing new crosses IPC.
- `src-tauri/src/core/i18n/mod.rs` — removed `AiTranslateProviderCallFailed` and
  `AiTranslateBatchStopped` from `message_keys!` (nothing constructs them anymore; both
  `impl From<OpenAiClientError> for IpcError` and `batch_stopped_error` were the only two call
  sites and both were rewritten in this phase). Added six new variants (Task 3, Decision 2) —
  exact names and vi.json keys for Phase 2:
  - `AiTranslateProviderUnreachable` → `err.ai_translate.provider_unreachable` — params `[]`
  - `AiTranslateProviderRefused` → `err.ai_translate.provider_refused` — params `["status"]`
  - `AiTranslateStreamEndedWithoutDone` → `err.ai_translate.stream_ended_without_done` — params `[]`
  - `AiTranslateReplyUnreadable` → `err.ai_translate.reply_unreadable` — params `[]`
  - `AiTranslateClientBuildFailed` → `err.ai_translate.client_build_failed` — params `[]`
  - `AiTranslateApiKeyHeaderInvalid` → `err.ai_translate.api_key_header_invalid` — params `[]`
- `src-tauri/src/commands/aitranslate.rs` (Task 4) — replaced `openai_client_error_is_retryable`
  with `openai_client_error_family(&OpenAiClientError) -> (&'static str, MessageKey, bool)`,
  returning `(code, message_key, retryable)` for all EIGHT variants (the 7 original + the new
  `ApiKeyHeaderInvalid`). `impl From<OpenAiClientError> for IpcError` and `batch_stopped_error`
  both call it — the same function, not a copy, exactly the precedent the old doc-comment
  described. `batch_stopped_error` now attaches `status` (as well as `segment_id`) when the
  variant is `NonSuccessStatus`, closing the "today batch_stopped_error drops `{status}`" gap the
  spec named. The stale `:95-99` doc-comment (claimed two branches retryable that the code made
  `false`, never mentioned `BufferOverflow`) was replaced, not deleted — the new doc-comment
  quotes the old wrong claim verbatim inside a 🔵 2026-09-22 block before correcting it, per the
  "claim that stops being true gets fixed in place" convention. Three other doc-comments in the
  same file that named the retired `ai_translate.batch_stopped`/`ai_translate.provider_call_failed`
  strings or the retired function name were also updated in place (lines near
  `AiTranslateBatchEventWire`, `send_prepared_translate_call`, `BatchCallError`) — none of these
  are executable code, only stale prose that would otherwise lie about the diff.
- `src-tauri/tests/ai_translate_contract.rs` (Task 9, repair half) — repaired the four cases the
  spec named: `provider_returns_a_non_2xx_maps_to_a_non_retryable_ipc_error_naming_the_status`
  (`:941` originally) now asserts `AiTranslateProviderRefused`/`ai_translate.provider_refused`;
  `stream_ended_without_done_maps_to_a_retryable_ipc_error` (`:953`) now asserts
  `AiTranslateStreamEndedWithoutDone`/`ai_translate.stream_ended_without_done`; the 7-variant
  sweep (`:964`) was renamed to
  `every_openai_client_error_variant_maps_to_its_own_family_key_with_the_documented_retryable_flag`
  and widened to 8 tuples `(variant, expected_key, expected_retryable)`, one per
  `OpenAiClientError` variant including the new `ApiKeyHeaderInvalid`; the batch pair (`:1733`)
  now asserts the SAME family keys as the single-run path plus `segment_id`, and additionally
  asserts `status` now survives for the non-2xx batch case (previously dropped). A stale ⚠️
  block comment above §5 (claimed `batch_stopped_error` was still private and untestable without
  an `AppHandle`; that was already false since Story 4.9 Phase 4b made it `pub`) was corrected in
  place with a 🔵 2026-09-22 note rather than left to keep lying next to code this phase changed.
- `src/i18n/vi.json` — appended exactly six lines (the six cause sentences above), all placed
  right after the existing `err.ai_translate.batch_stopped` line. Nothing else in the file was
  touched. `git diff --stat` confirms `1 file changed, 6 insertions(+)`.

**Deliberately left as-is (not a gap, a scope boundary):**

- The two old vi.json lines — `err.ai_translate.provider_call_failed` and
  `err.ai_translate.batch_stopped` — are still in `src/i18n/vi.json`, now orphaned (no
  `MessageKey` variant maps to them anymore). Phase 1's boundary says "add only the six cause
  sentences" to that file, so they were not deleted. `ipc_contract.rs::
  every_message_key_exists_in_vi_json` only checks one direction (an orphaned `vi.json` key is
  not caught, per the spec's own §Gates note), so this is inert, not a red gate waiting to
  happen. `tests/frontend/aiTranslate.test.ts`/`aiTranslateBatch.test.ts` build fixture
  `IpcError` objects using these exact old strings and resolve them through the real
  `i18n.tError()` at test time — since the vi.json lines are still there, those fixtures still
  resolve to real text and those tests are believed unaffected (not run this phase — Phase 1's
  gates stop at `cargo test`; Phase 2 runs `npx vitest run` and will find out for certain).

**Measured (fresh, this tree, not copied from the spec):**

- `npm run build` — green. `vue-tsc --noEmit` (both configs) + `vite build`, no errors.
- `cd src-tauri && cargo test --locked` — green, exit code 0, **1785 passed, 0 failed** across the
  whole suite (measured via `grep -oE "[0-9]+ passed; [0-9]+ failed" | awk '{sum}'` over a full
  un-truncated log — the first attempt piped through `tail -200` and its exit code was `tail`'s,
  not `cargo test`'s, so it was re-run to a file with the real exit code captured).
  `ai_translate_contract.rs`: 43/43 — same `#[test]` count as the pre-phase tree (verified via
  `grep -c "^#\[test\]"` on `git show HEAD:...` vs the working tree, both 43); this phase repaired
  four EXISTING cases in place and widened one array from 7 to 8 tuples, it added no new `#[test]`
  function. `ipc_contract.rs`: 34/34, including
  `every_message_key_exists_in_vi_json` and `every_message_key_declares_the_params_its_string_needs`
  green on the six new keys. `config_invariants.rs`: 31/31, including
  `every_command_bearing_file_is_classified_with_measured_attribute_counts` — **census row `3, 0,
  0` and tree total `(71, 28)` confirmed unchanged**, no new `#[tauri::command]` was added.
  `ai_boundary.rs`: 25/25 (1 ignored, pre-existing). `aiconfig_keychain_boundary.rs`: 9/9.
- `npm run check:i18n` — green (Kiểm A–E all pass; Kiểm B counts 992 keys, up from 986 by the six
  new lines; Kiểm C counts 204 placeholders, up by one — the `{status}` in the new
  `provider_refused` sentence).
- `npm run check:commands` — green (untouched by this phase; run only as a sanity check since
  `check:i18n`'s scope overlaps `src/i18n/vi.json`).
- `git status --porcelain` — confirms the diff touches exactly: `src-tauri/src/commands/
  aitranslate.rs`, `src-tauri/src/core/ai/client.rs`, `src-tauri/src/core/aiconfig/keychain.rs`,
  `src-tauri/src/core/i18n/mod.rs`, `src-tauri/tests/ai_translate_contract.rs`,
  `src/i18n/vi.json` — no `tests/frontend/**` file touched, no other `src/**` file touched.

**What Phase 2 needs (per the phase file's own list):**

- The six `MessageKey` variant names and vi.json keys are listed above under "Changed."
- Retryable, as the code now reads (measured from `openai_client_error_family` in
  `commands/aitranslate.rs`, not copied from Decision 2's prose):
  - `AiTranslateProviderUnreachable` (`RequestFailed`, `ReadFailed`) → **retryable: true**
  - `AiTranslateProviderRefused` (`NonSuccessStatus`) → **retryable: false**
  - `AiTranslateStreamEndedWithoutDone` (`StreamEndedWithoutDone`) → **retryable: true**
  - `AiTranslateReplyUnreadable` (`MalformedEvent`, `BufferOverflow`) → **retryable: false**
  - `AiTranslateClientBuildFailed` (`ClientBuildFailed`) → **retryable: false**
  - `AiTranslateApiKeyHeaderInvalid` (`ApiKeyHeaderInvalid`) → **retryable: false**
  - This matches Decision 2's prose exactly (measured, not assumed) — no drift was found between
    the spec's table and the shipped classification.
- `batch_stopped_error(segment_id, err) -> IpcError` is unchanged in signature and stays `pub`;
  its `params` now always carries `segment_id`, plus `status` when the family is
  `AiTranslateProviderRefused`. Frontend code reading `result.error.params.segment_id`
  (`src/aiTranslateBatchState.ts:192`, untouched this phase) needs no change — that read does not
  depend on which of the six `message_key`s came back.
- `IpcError.code()` is now one of six family-specific strings (`ai_translate.provider_unreachable`
  / `provider_refused` / `stream_ended_without_done` / `reply_unreadable` / `client_build_failed`
  / `api_key_header_invalid`) for BOTH the single-run and batch paths — grepped, nothing in
  `src/**` branches on `IpcError.code`'s exact value today (only type-checks it's a string in
  `src/config/aitranslate.ts:44`), so this is not expected to require a frontend change, but Phase
  2 should not assume a stale `'ai_translate.provider_call_failed'`/`'ai_translate.batch_stopped'`
  literal anywhere in `src/**` still matches a real backend value.

### Phase 2 (Webview: the retry affordance) — 2026-09-22

**Changed:**

- `src/aiTranslateBatchState.ts` (Task 6) — added `aiTranslateBatchRetryIds(currentRows)`, a
  pure function returning the `segmentId`s of every row whose `status` is `error` or `pending`,
  in `currentRows` order — same shape as the existing `aiTranslateBatchTextForSegment`. `done`,
  `skipped`, `cancelled`, and `running` are excluded by construction, so a retry can never
  re-send a finished sentence.
- `src/commands/index.ts` (Task 5, `command.<id>` half, and Task 7's registration half) —
  registered two new commands, `ai.translate.retry` and `ai.translate.batch_retry`, both
  `keys: undefined` (same reasoning already recorded for `ai.translate.run`/`.cancel`: no
  default chord is free — `Mod+Enter`/`Escape` are already claimed — reachable by panel button
  and the shortcut-assignment screen). Added the matching `retryAiTranslate?: () => void` /
  `retryAiTranslateBatch?: () => void` ports to `CommandDeps`.
- `src/main.ts` (Task 7's handler half) — real handlers for both new commands, following the
  existing two-layer-defence shape every other AI command in this file uses:
  - `retryAiTranslate` refuses (warns, does not throw) while either module is `generating`,
    then refuses again unless `aiTranslateStateValue.value === 'error'` **and**
    `aiTranslateError.value.retryable === true` — re-checking the flag rather than trusting the
    panel already gated the button, per §Always spec 4.10 ("`retryable` grants only the right
    to SHOW a button"). Retries against `aiTranslateRunSegmentId` (the segment the error
    belongs to), not `editorCaretSegmentId` — the caret may have moved, same reasoning
    `promoteAiTranslate` already uses for the single-run branch.
  - `retryAiTranslateBatch` mirrors that shape, reading `aiTranslateBatchError`/
    `aiTranslateBatchStateValue`, and calls `runAiTranslateBatch` with
    `aiTranslateBatchRetryIds(aiTranslateBatchRows.value)`.
- `src/panels/AiTranslationPanel.vue` (Task 8) —
  - Two new `v-if`-gated buttons (not `:disabled` — the button is absent from the DOM, not
    shown greyed out, when `retryable` is not true): `.ai-translate-retry` next to
    `.ai-translate-promote` in the single-run action row, `.ai-translate-batch-retry` next to
    the batch alert. Both joined the EXISTING `.ai-translate-run, .ai-translate-cancel,
    .ai-translate-promote, .ai-translate-batch-run { … }` CSS rule group (added their two
    selectors) rather than copying the ruleset, per the phase file's explicit instruction and
    4.9's review findings on exactly that duplication.
  - `canRetryAiTranslate`/`canRetryAiTranslateBatch` computeds gate on `state === 'error' &&
    error !== null && error.retryable === true`, plus the same cross-module mutual-exclusion
    already used by `canRunAiTranslate`/`canRunAiTranslateBatch`.
  - The single-run alert (`:349-352` in the spec's Code Map) now carries
    `data-ai-translate-state="error"` — the same attribute name its three sibling state lines
    (`generating`/`cancelled`/`not_configured`) already use, read as "the error line is itself
    one more case of `aiTranslateStateValue`," not a new hook shape.
- `src/i18n/vi.json` — two lines: `"command.ai.translate.retry": "Thử lại"` and
  `"command.ai.translate.batch_retry": "Thử lại các câu chưa xong"`.

**Dropped — Decision 4 (silent promote error), and why, measured:**

🔴 Decision 4 ("surface `editorPromoteAiTranslationError` through `tError()`", Task 8's third
clause) was implemented, then reverted, on direct measurement. Importing
`editorPromoteAiTranslationError` from `editorPanelState.ts` into `AiTranslationPanel.vue` and
rendering it made `npx vitest run` fail **24 tests across 2 files** —
`tests/frontend/aiTranslate.test.ts` (14 of 17) and `tests/frontend/aiPromptInspector.test.ts`
(10 of 10) — because both files `vi.doMock('../../src/panels/editorPanelState', () => ({ ... }))`
with a fixed export list that does not include this new name, and the component now reads it at
render time on every mount. This phase's own boundary forbids touching `tests/frontend/**` (the
phase file: "Phase 3 owns the cases, and a phase that writes both the code and its only test
removes the next phase's ability to find the gap"), so the mocks could not be repaired here.
Decision 4 itself names the way out: *"Drop it if the scope is tight."* Reverted (the import and
the one `<p v-if>`/`tError()` case); confirmed `npx vitest run` back to **1309 passed, 1309
total, 91/91 files**. `editorPromoteAiTranslationError` (`editorPanelState.ts:272`) is therefore
still exported and read by nothing — the hole Decision 4 named is still open, now explicitly
deferred rather than silently dropped. Not re-added to `deferred-work.md` by this phase — Task
11 (ledger) is Phase 4's, orchestrator-only per the phase file.

**Left for Phase 3, measured from this diff:**

- Command ids registered: `ai.translate.retry`, `ai.translate.batch_retry` — both `keys:
  undefined`, both gated in `main.ts` by (state === 'error') && (error.retryable === true) &&
  (the other module is not 'generating'). Both are also gated in the panel by `v-if`, so a test
  driving them through `dispatch()` directly (bypassing the button, the way this story's own
  §Tests already do for `run`/`cancel`/`promote`) exercises the `main.ts` guard on its own.
- `data-*` hooks added this phase: `data-ai-translate-retry` (button),
  `data-ai-translate-batch-retry` (button), `data-ai-translate-state="error"` (the single-run
  alert, joining the existing `data-ai-translate-state` family). The batch alert's
  `data-ai-translate-batch-alert` (Story 4.9) is unchanged.
- `aiTranslateBatchRetryIds(currentRows)` (`src/aiTranslateBatchState.ts`) is the pure function
  Task 9's frontend cases (`tests/frontend/aiTranslateBatch.test.ts`) should call directly to
  assert the exact id set a batch retry dispatches, rather than re-deriving it from row statuses
  inline.
- Decision 4 is UNBUILT, not partially built — grep for `editorPromoteAiTranslationError` finds
  only its declaration in `editorPanelState.ts`, no reader. If Phase 3 (or Ice) wants it closed,
  the fix is the same one this phase reverted, plus updating the two mocks named above to
  include the new export — which Phase 3's boundary allows and this phase's did not.

**Measured (fresh, this tree):**

- `npm run build` — green, both `vue-tsc` configs plus `vite build`, no errors (two pre-existing
  `INEFFECTIVE_DYNAMIC_IMPORT` warnings from `editorPanelState.ts`, unrelated to this diff and
  present before it).
- The eleven `pre-push` gates run INDIVIDUALLY, all green: `check:deps` · `check:tokens` ·
  `check:i18n` (Kiểm B measured **994 khoá** — Phase 1 closed at 992, this phase's two
  `command.ai.translate.retry`/`.batch_retry` label lines account for the +2) · `check:commands`
  (175 commands, up from 173;
  `ai.translate.retry`/`ai.translate.batch_retry` both listed in the `unbound()` census, both
  found in `vi.json`) · `check:layout` · `check:panel-refs` · `check:dict` · `check:dict-manifest`
  · `check:lint` (0 findings) · `check:gates` · `check:debt-owner`.
- `npx vitest run` — green, **1309 passed, 1309 total, 91/91 files** (after the Decision-4
  revert; see above for the 24-failure state before it).
- `cd src-tauri && cargo test --locked` — green, exit 0, **1785 passed, 0 failed, 22 ignored**,
  65 `test result:` lines — identical to Phase 1's measured baseline, because this phase touched
  no file under `src-tauri/**` (`git status --porcelain -- src-tauri/` shows only Phase 1's five
  files, all pre-existing modifications).
- `git status --porcelain` — this phase's diff is exactly: `src/aiTranslateBatchState.ts`,
  `src/commands/index.ts`, `src/i18n/vi.json`, `src/main.ts`, `src/panels/AiTranslationPanel.vue`.
  No `tests/frontend/**` file touched, no `src-tauri/**` file touched.

### Phase 3 (the coverage this story is for) — 2026-09-22

**🔴 Orphan deletion done FIRST, and it found the "bigger finding" the phase file warned about.**
Deleted `err.ai_translate.provider_call_failed` and `err.ai_translate.batch_stopped` from
`src/i18n/vi.json` (the two lines Phase 1 was barred from touching), then ran `npx vitest run`
on `tests/frontend/aiTranslate.test.ts`/`aiTranslateBatch.test.ts` WITHOUT touching the
fixtures, exactly as ordered. **Both files stayed GREEN — 2/2 files, 30/30 tests.** Read WHY
before touching anything: both files' only assertion on the error text was
`expect(alert.text()).toBe(i18n.tError(SOME_ERROR))` / `...toBe(i18n.tError(BATCH_STOPPED_ON_12))`
— the SAME `tError()` function called a second time on the SAME fixture payload, not a literal
string. `resolve.ts`'s documented behaviour for a missing key is "return the key itself,
verbatim" (`t()`'s `!has(catalog, safeKey)` branch), so once the key was deleted BOTH sides of
`toBe` independently resolved to the same fallback (the bare key string) and stayed trivially
equal — the assertion never actually read `vi.json`'s content, only that `tError` is
deterministic, which is true regardless of whether the key exists or the text is right. This is
a bigger finding than the one Phase 3 was sent to fix: neither file had ever really canh'd the
sentence it claimed to assert on. Fixed at the same two call sites (not just renamed): both now
compare against a Vietnamese string LITERAL copied from `vi.json`, so a wrong key OR a changed
`vi.json` sentence now actually reddens the test. Only after this measurement were the two
fixtures moved onto real family keys — `SOME_ERROR` → `STREAM_ENDED_ERROR`
(`err.ai_translate.stream_ended_without_done`, retryable, matches the test's own title "stream
lỗi giữa chừng") and `BATCH_STOPPED_ON_12` → `PROVIDER_UNREACHABLE_ON_12`
(`err.ai_translate.provider_unreachable`, retryable, `segment_id` in `params` as
`batch_stopped_error` always attaches it). One more thing the family-key sentences changed:
Decision 2 means the CAUSE sentence no longer interpolates `{segment_id}` (only the batch row
list does — "no per-path duplicate of the family"), so the old test title's claim
"`.ai-translate-alert` nêu đúng câu 12" is no longer true; the title was corrected in place to
say the row list is what identifies câu 12 now, not the alert text.

**New Rust coverage (`ai_translate_contract.rs`), the two holes named in §Coverage plus the
`:1733` widening, all added — none removed, none restated:**

- `a_provider_error_with_zero_tokens_received_leaves_the_channel_empty_and_propagates_the_error`
  — single-run path, `FakeProvider { tokens: vec![], finish: Err(...) }`; asserts the `Channel`
  received ZERO frames (not "the same one frame as the partial-tokens case").
- `provider_error_with_zero_tokens_received_on_a_batch_sentence_leaves_no_event_for_that_sentence_and_stops_the_batch`
  — batch path, sentence 6/12 fails before its first token; asserts sentences 1-5 keep their
  `Token`+`Done` pair, sentence 6 has NO event at all (not even a `Token`), sentences 7-12 are
  never called (`call_count == 6`, same `MultiItemProvider` panic-on-overrun guard the existing
  cancel/error-mid-batch cases use).
- `no_work_open_on_a_batch_reuses_the_existing_work_none_open_key` — `prepare_batch_call(Some(&global), None, ...)`;
  confirms the batch path rejects at the SAME `open.ok_or_else(...)` line the single-run path
  does, before `segment_ids` is ever read.
- `batch_stopped_error_names_the_sentence_and_maps_the_documented_retryable_flag` — WIDENED in
  place from 2 of 8 `OpenAiClientError` variants to all 8 (mirrors the single-run
  `every_openai_client_error_variant_maps_to_its_own_family_key_with_the_documented_retryable_flag`
  sweep exactly, same 8-tuple shape, only the call site differs:
  `batch_stopped_error(SEGMENT_ID, variant)` instead of `variant.into()`). Two assertions added
  per case that the 2-variant version didn't need to make explicit: `segment_id` is present on
  EVERY variant (not just the two originally covered), and `status` is present on exactly one
  (`NonSuccessStatus`) and absent on the other seven.

**Counter-check performed as a real removal at the seam, then restored — not left in the diff.**
Temporarily edited `openai_client_error_family` in `src-tauri/src/commands/aitranslate.rs`,
forcing `RequestFailed`/`ReadFailed`/`StreamEndedWithoutDone` to map to `retryable: false`
(backed up the file first with `cp`). Ran `cargo test --test ai_translate_contract`: **exactly
3 failed, 43 passed** —
`every_openai_client_error_variant_maps_to_its_own_family_key_with_the_documented_retryable_flag`,
`batch_stopped_error_names_the_sentence_and_maps_the_documented_retryable_flag`, and
`stream_ended_without_done_maps_to_a_retryable_ipc_error` — each failing on an `assert_eq!`
naming the mismatched `retryable` flag (e.g. `RequestFailed { detail: "x" } phai anh xa
retryable=true / left: false, right: true`), the right REASON, not a panic or an unrelated
compile/behaviour break. Restored the file from the backup (`cp` back, confirmed via
`git diff --stat` matching Phase 1/2's own prior diff exactly, and via
`grep "COUNTER-CHECK ONLY"` returning no match); re-ran `cargo test --test ai_translate_contract`
green, 46/46.

**No I/O-matrix row was left unreachable.** All ten rows have a test: the six cause families
(pre-existing family-mapping tests, unchanged this phase), zero-tokens (both paths, new this
phase), batch-stops-at-N-of-M (pre-existing), batch-retry-after-cancel (new frontend test, this
phase — see below), and non-2xx-inside-a-batch (covered by the widened `batch_stopped_error_...`
sweep, which now asserts `status` survives for the `NonSuccessStatus` case).

**New frontend coverage — asserts the exact ids/text, not just "a call happened":**

- `tests/frontend/aiTranslate.test.ts`, new describe `nút "Thử lại" lượt ĐƠN`: (a) a retryable
  error shows `[data-ai-translate-retry]`; moving the caret to a DIFFERENT segment before
  clicking retry still calls `runAiTranslateSegment` with the FAILED segment's id (3), not the
  caret's new one (99) — `toHaveBeenLastCalledWith(3, null, expect.any(Function))`; (b) a
  non-retryable error renders no retry button at all, AND dispatching `ai.translate.retry`
  directly (the chord path, bypassing the hidden button) is a no-op — `runMock` not called, state
  stays `error`. `retryAiTranslate` was added to this file's `freshPanel()`'s `installCommands`
  (Phase 2 left no test-side wiring for it, correctly — that was Phase 3's to add), copying
  `main.ts`'s real two-layer guard shape (retryable-only, keyed off `aiTranslateRunSegmentId` not
  the caret).
- `tests/frontend/aiTranslateBatch.test.ts`, new describe `nút "Thử lại" LÔ`: (a) an error on row
  2/3 (retryable) shows `[data-ai-translate-batch-retry]`; `aiTranslateBatchRetryIds(...)` called
  directly returns exactly `[12, 13]`; clicking retry dispatches `runAiTranslateBatchCall` with
  exactly `[12, 13]` (`toHaveBeenCalledWith([12, 13], null, expect.any(Function))`) — NOT
  `[11, 12, 13]`, proving the already-`done` sentence 11 is excluded; (b) a non-retryable error
  (row 3/3) shows no retry button, and dispatching `ai.translate.batch_retry` directly is a
  no-op; (c) **the amended frozen matrix row** — cancel sentence 2/3 mid-flight, confirm NO
  `[data-ai-translate-batch-retry]` appears (state `cancelled` carries no `IpcError`, so nothing
  can grant the button) AND `aiTranslateBatchRetryIds(...)` returns exactly `[13]` — the cancelled
  row (12) is excluded, only the never-run row (13) counts, matching the Spec Change Log's KEEP
  clause verbatim — and dispatching `ai.translate.batch_retry` directly is still a no-op (the
  second guard layer, `aiTranslateBatchError.value === null`, refuses it independently of the
  button being hidden). `retryAiTranslate`/`retryAiTranslateBatch` were added to this file's
  `installCommands`, same two-layer shape as `main.ts` (mutual-exclusion first, then the
  retryable-error check).

**One production-file change outside the two `vi.json` lines — disclosed, not hidden.** Fixed a
now-stale doc-comment in `src-tauri/src/core/i18n/mod.rs` (directly above the six
`AiTranslate*` family keys) that asserted the two orphaned `vi.json` lines "được GIỮ NGUYÊN, mồ
côi có chủ ý" — true when Phase 1 wrote it (Phase 1's own boundary forbade deleting them), false
the moment Phase 3 deleted them above. Per this repo's own convention ("a claim that stops being
true gets FIXED IN PLACE with 🔵 and a date; don't delete it, and don't let it quietly lie" —
`AGENTS.md`), left in place it would be exactly the kind of lying comment that convention exists
to prevent. Fixed with a 🔵 2026-09-22 block, prose only, zero bytes of logic changed, zero
`#[test]` results affected (`cargo test --test ipc_contract --test config_invariants` reconfirmed
green immediately after, and the full-suite recount below is unchanged from before this edit).
Flagging it explicitly since the phase file's line "do not change production code" apart from
`vi.json` is otherwise absolute.

**Measured (fresh, this tree, in order):**

- `npm run build` — green, both `vue-tsc` configs + `vite build`, same two pre-existing
  `INEFFECTIVE_DYNAMIC_IMPORT` warnings as Phase 2, nothing new.
- `cd src-tauri && cargo test --locked` — green, exit 0, **1788 passed, 0 failed, 22 ignored**, 65
  `test result:` lines (measured twice, once before and once after the `mod.rs` doc-comment fix —
  identical both times). 1785 (Phase 2 baseline) + 3 new `#[test]` functions this phase added
  (the fourth Rust change, the `:1733` widening, is the SAME test function, not a new one) = 1788.
  `ai_translate_contract.rs` alone: 46/46 (was 43 pre-phase).
- `npx vitest run` — green, **1314 passed, 1314 total, 91/91 files**. 1309 (Phase 2 baseline) + 2
  (`aiTranslate.test.ts`) + 3 (`aiTranslateBatch.test.ts`) = 1314.
- The eleven `pre-push` gates, run individually: `check:deps` · `check:tokens` · `check:i18n`
  (Kiểm B: **992 khoá** — back down from Phase 2's 994, because this phase's net `vi.json` change
  is −2 lines: −2 orphans, +0 new keys) · `check:commands` (175 commands, unchanged — no new
  command id this phase) · `check:layout` · `check:panel-refs` · `check:dict` ·
  `check:dict-manifest` · `check:lint` (0 findings) · `check:gates` · `check:debt-owner` — all
  green.
- `npm run test:story 4-10 -- --list` then without `--list` — scope: 5 Rust targets
  (`ai_boundary`, `ai_translate_contract`, `aiconfig_keychain_boundary`, `config_invariants`,
  `ipc_contract`) + 3 frontend files (`aiPromptInspector.test.ts`, `aiTranslate.test.ts`,
  `aiTranslateBatch.test.ts`). `aiPromptInspector.test.ts` is in scope NOT because this phase
  touched it (it didn't — `git status --porcelain` confirms zero diff on that file) but because
  the tool's primary source is the WHOLE story's uncommitted diff (all three phases), and that
  file also mounts `AiTranslationPanel.vue`, which Phase 2 changed — correct behaviour of the
  tool, not a gap. Exit 0, no THIẾU KHAI reported (every touched file was already in the spec's
  §Tests that move list). All 5 Rust targets green, all 3 frontend files green (83/83 tests).
- `git status --porcelain` — this phase's diff is exactly: `src/i18n/vi.json` (net −2 lines),
  `src-tauri/src/core/i18n/mod.rs` (doc-comment only, disclosed above),
  `src-tauri/tests/ai_translate_contract.rs`, `tests/frontend/aiTranslate.test.ts`,
  `tests/frontend/aiTranslateBatch.test.ts`. No other file touched.

**Left incomplete / risky — none identified beyond what Phase 4 already owns.** Task 11
(`deferred-work.md`) is explicitly the orchestrator's own job per the phase file ("Phase 4 —
Ledger. The orchestrator does this one; no agent is dispatched for it") — not touched here.
Decision 4 (silent promote error) stays dropped, as Phase 2 left it; this phase added no case for
it and did not attempt to revisit it (out of scope — Task 8's third clause was already struck
through). The four `MessageKey::Unknown` state-missing branches and the unreachable
`batch_panicked_error` retryable-wording note (both flagged 🔴/⚠️ in Phase 1's notes) remain
recorded, not chased, per the spec's own instruction to record rather than chase them.

## Spec Change Log

### 2026-09-22 — frozen I/O Matrix row "Batch retry after a cancel", amended on Ice's signature

**Finding that triggered it.** Read from the Phase 2 diff, not from a test: `canRetryAiTranslate`
and `canRetryAiTranslateBatch` (`AiTranslationPanel.vue:238-251`) both require
`state === 'error'` plus a non-null `IpcError` with `retryable === true`. A cancel leaves the batch
in `'cancelled'` with no `IpcError` at all, so no retry affordance can appear — while the matrix
row promised "Retry offered for the sentences that never ran".

**The contradiction was inside the frozen block, between two clauses both written at planning
time.** §Always says "`retryable` grants only the right to SHOW a button"; after a cancel there is
nothing to grant it, as the row's own Error-Handling column ("no error") states. The two could not
both hold, so no implementation could satisfy the matrix.

**Amended.** The row now reads that a cancel offers NO retry affordance and the user reselects the
unrun sentences. Ice settled it 2026-09-22, choosing this over widening §Always — which would have
weakened the one clause that keeps auto-retry out — and over a separately-named
"continue the rest" command, which was real added scope.

**Known-bad state this avoids.** Phase 3 would have written a matrix-row test that cannot pass,
and the pressure at that point is to quietly soften the assertion until it goes green, which is
how a frozen promise becomes a test that proves nothing.

**KEEP.** The id-set half of the row was already right and must survive any re-derivation:
`aiTranslateBatchRetryIds` returns the `error` row plus every `pending` row and excludes `done`,
`skipped` and `cancelled`, so in the cancel scenario it yields exactly the never-run sentences.
Also keep the two-layer guard shape — the real handler in `main.ts` re-reads `retryable` rather
than trusting that the button was disabled correctly.

### 2026-09-22 — Decision 4 dropped under its own escape clause

Decision 4 (surface `editorPromoteAiTranslationError` through `tError()`) said "Drop it if the
scope is tight." Measured reason it is being dropped rather than deferred to a later phase:
`tests/frontend/aiTranslate.test.ts:119` and `aiPromptInspector.test.ts` both `vi.doMock`
`editorPanelState.ts` with a factory returning a FIXED object literal, and neither names this
export, so importing it into the panel leaves it `undefined` and 24 existing cases fail. Fixing
those mocks in the same phase that adds the panel line would have one agent write both the code
and its only test, which is what the phase boundaries exist to prevent. Task 8's clause is struck
through rather than deleted, and the gap goes to the ledger with an owner in Phase 4 — a failed
promote is still silent on screen, exactly as it was before this story.

## Review Triage Log

### 2026-09-22 — Pass 1, three layers over a 128 kB diff (blind-hunter 10 · verification-gap 1+1 · edge-case 2)

No `intent_gap` and no `bad_spec`, so no loopback. The diff handed to the layers deliberately
EXCLUDED the two untracked BMAD artifacts (this spec, the phases file): the spec goes to the
edge-case layer alone as `claims_file` by design, and putting it in the shared diff would have
removed the blind layer's blindness.

| # | Finding | Verdict | Evidence | Route |
|---|---|---|---|---|
| 1 | The retry commands' mutual-exclusion guard has no test, though the identical guard on the `run` commands has one | **medium** | Pre-verified by the gap layer: `main.ts:1048/:1052/:1075/:1079` carry the guard, and the pre-existing `describe('Cổng loại trừ lẫn nhau giữa một lượt ĐƠN và một LÔ')` in `aiTranslateBatch.test.ts` covers the same shape for `run`/`batch_run` but was never extended. Dropping either retry guard leaves every test green | patch |
| 2 | Nothing asserts the rendered text of the ONE new key carrying a required placeholder | **medium** | Verified: `provider_refused` is the only new key with `["status"]`, and `grep` over both frontend test files finds only the fixture `params: { status: '401' }` — no assertion on the substituted sentence. `resolve.ts:113` returns the placeholder VERBATIM when a param is missing, so the failure mode is a literal `{status}` on the user's screen, and this story's own lesson is that an unasserted rendering hides exactly that | patch |
| 3 | `aiTranslateBatchState.ts:203` comment names `ai_translate.batch_stopped`, a `code` this change removed | **low** | Verified by reading the line; found independently by the gap layer and the edge-case layer, one root cause, one entry. The repo's own convention is that a claim which stops being true gets fixed in place, and a `code` string that greps to nothing is the developer-facing harm | patch |
| 4 | The ledger entry for `batch_panicked_error` rests on release-only reachability while the harm can occur in a dev build | **low** | Verified in `src-tauri/Cargo.toml`: `panic = "abort"` is set only under `[profile.release]:208`, `[profile.bench-release]:232` is `unwind`, there is no `[profile.dev]`, and line 214 records that Cargo always builds a test target with `unwind`. The entry does say "trong một build release", but the conclusion it supports — record, do not chase, the treatment for an unreachable path — is wider than that scope | patch |
| 5 | `log_keyring_error`'s doc-comment states as fact that `NoStorageAccess` means the user pressed Deny | **low** | Verified at `core/aiconfig/keychain.rs:122-123`. `keyring::Error::NoStorageAccess` covers a store that is locked or otherwise inaccessible, not only a declined consent prompt, so the named harm is a developer reading the log mapping and stopping at the wrong cause | patch |
| 6 | The single-segment `From` sweep asserts `message_key()` and `retryable()` but never `code()` | **low** | Verified: the loop at `ai_translate_contract.rs:1002+` asserts two of three tuple members; the batch sibling asserts `code()` for all 8. Both read the same `openai_client_error_family`, so the strings are covered transitively — but a future edit that hardcoded a `code` inside `impl From` alone would pass. One assert in an existing loop closes it | patch |
| 7 | `err.ai_translate.api_key_header_invalid` names neither the provider nor this machine, falsifying AC1 as written | **low** | Verified: five of six sentences name "nhà cung cấp AI" or "Máy này"; this one names the saved key. But the filed consequence — the user cannot tell what is at fault — is wrong: the sentence says the saved key is. The cause is neither the provider's nor the machine's, so AC1's two-way enumeration simply has no slot for it, and the copy still satisfies the no-blame rule and `check-i18n.mjs` Check D. Its only fix edits this build's spec | rejected |
| 8 | `keyring_error_variant_name`'s 11 arms and its `_ => "Unknown"` fallback have no test | **low** | Verified: `grep` finds no reference outside the file it lives in. Harm is developer-only (a wrong name in a diagnostic log), and `keyring::Error` is `#[non_exhaustive]`, so constructing its arms from a test crate is not a cheap addition | rejected |
| 9 | The batch alert never got a state-marker attribute while the single-run alert gained `data-ai-translate-state="error"` | **low** | Verified: `AiTranslationPanel.vue:386` versus `:446`. No harm is named beyond style — the batch alert already carries `data-ai-translate-batch-alert` and Phase 3's cases select on it successfully | rejected |
| 10 | `sprint-status.yaml` moves 4-10 to `in-progress` while 4-7/4-8/4-9 sit at `review` | **false** | Refuted: `in-progress` is what step-03 of this workflow writes when implementation starts, and the story is still inside that workflow. It is the current state, not a stale one, and the phase-tracking file the finding says it would need is exactly what records that | rejected |
| 11 | The spec and phases files are untracked and absent from the diff, so quoted claims cannot be checked | **false** | Refuted: their absence is this orchestrator's deliberate choice, stated at the top of this entry, and the spec was handed to the edge-case layer as `claims_file`. That layer reported it verified every Execution-task claim and AC2–AC6 against the diff, which is the check this finding says nobody can perform | rejected |
| 12 | The retry handlers reject a stale or chord dispatch with only `console.warn`, giving the user nothing | **false** | Refuted by the repo's explicit rule, `src/AGENTS.md:16`: "A function run from a keyboard chord NEVER throws — it logs a diagnostic naming the cause and returns `false`. Don't 'fix' it by switching mode: that is guessing the user's intent." The requested behaviour is the one the rule forbids, and the path is unreachable through the UI because the button is hidden | rejected |
| 13 | The `editorPromoteAiTranslationError` gap is named but nothing schedules who unblocks it | **false** | Refuted: the ledger entry added by this story carries `(Chủ: Ice — cùng lớp quyết định hạ tầng test mà Ice đã nhận cho bàn đo `AppHandle`)`, and `check:debt-owner` reports `mở KHÔNG có Chủ: 0`. It is scheduled to an owner, which is what this repo means by scheduled | rejected |


## Design Notes

**Why retry needs no new Tauri command.** `ai_translate_segment` already takes one `segment_id`
and `ai_translate_batch` already takes `Vec<i64>`. A retry is those commands called again with a
narrower list, so the whole feature lands in the webview plus copy. That is also what keeps the
two census numbers frozen, which is the cheapest possible proof that the Rust surface did not
grow.

**Why the cause has to live in `message_key` rather than in a param.** `IpcError` carries exactly
one `message_key`, and `IpcError::new` downgrades it to `Unknown` in release when a declared
required param is missing (`core/i18n/mod.rs:924-929`). `AiTranslateProviderCallFailed` declares no
params while `{status}` is really attached for non-2xx only, so making `"status"` required would
break the six variants that send `{}` — silently, in release, with `debug_assert` catching it only
in debug. Separate keys per family is the only shape where every sentence's params are always
present.

**Decision 4 — the silent promote error gets a voice.** `editorPromoteAiTranslationError` is
exported and read by nothing, so a promote that fails against the store shows the user nothing at
all. It is a store-write error rather than a network error, so it is not FR75; it is included
because this story owns the panel's error surface and this is its last silent hole, and because the
panel is already on the `tError()` path rather than the fixed-string path the ledger's UX-DR30 debt
describes. One `v-if` and one case. Drop it if the scope is tight.

**Decision 5 — the keychain cause goes to the log, not to the screen.** The ledger transferred the
`map_err(|_| KeychainUnavailable)` diagnostic loss here and called it a decision rather than a
line of code. The decision: log the `keyring::Error` variant NAME, never its message, on the
unaccented diagnostic path, and change nothing that crosses IPC. A variant name distinguishes a
denied OS prompt from a missing platform store, which is the whole complaint, while spec 4.8's
§Always still forbids the key from reaching a log line, an error message or an `IpcError` param —
and a variant name cannot carry key material. This is recorded as a decision rather than asked,
because the user never sees it.

## Verification

**Commands:**

- `npm run build` **before** any `cargo test` — without `dist/`, `cargo test` fails at compile time.
- `cd src-tauri && cargo test --locked` — measure the baseline on this story's own starting commit
  (`915f1af6a84ff81b1184cadd26c00dccd551d317`) in a still tree FIRST, and do not inherit 4.9's
  counts. Two timings are comparable only from the same cache state.
- `npx vitest run` — 0 red.
- `npm run test:story 4-10` for the scoped dev loop; it reports THIẾU KHAI for any test file the
  diff touched that this spec never named.
- The eleven `pre-push` gates individually: `check:deps` · `tokens` · `i18n` · `commands` ·
  `layout` · `panel-refs` · `dict` · `dict-manifest` · `lint` · `gates` · `debt-owner`.
- ⚠️ The real `pre-push` order is eleven gates → vitest → `npm run build` → `cargo test`. Running
  the gates and `cargo test` while skipping `build` is what hid a red `vue-tsc` for several rounds
  in 4.8. Run all fourteen before calling a phase done, and read CI afterwards — `pre-push` runs
  only on macOS and says nothing about the Windows half.
- `cargo test --test ai_boundary --test config_invariants --test ipc_contract` after the Rust
  phase, and read **why** each red fires rather than its colour.

**Manual checks:**

- Point the provider endpoint at a port nothing listens on, translate one segment, and read the
  sentence: it must name an unreachable provider and offer retry. Then set a real endpoint with a
  wrong key and read it again: a different sentence, naming a refusal with its status, and NO retry
  button. Two different sentences from the same screen is the whole story in one check.
- Start a batch of several sentences against the dead endpoint after a few have finished, press
  retry, and confirm from the panel that the finished sentences were not called again.
