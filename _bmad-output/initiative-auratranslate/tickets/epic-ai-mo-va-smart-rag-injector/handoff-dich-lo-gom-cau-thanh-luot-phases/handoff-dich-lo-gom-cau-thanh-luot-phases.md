# Handoff Phase 2 (Rust) -> Phase 3 (Webview), story 4-15

Not committed. Plan: `tickets/story-dich-lo-gom-cau-thanh-luot-plan.md`. Rust tasks R1-R10 done.

## Wire the webview must consume

Event stream: `Channel<AiTranslateBatchEventWire>`, serde `tag = "kind"`, snake_case, all fields snake_case.

| kind | fields |
|---|---|
| `planned` | `calls: [{ call_index, segment_ids }]` -- once, first; `segment_ids` are the sentences to translate in block order (context ids are never listed) |
| `call_started` | `call_index` |
| `token` | `call_index, segment_id, text` -- display only (text already stripped of markers, TM copies and out-of-range blocks; lines after the first start with `\n`) |
| `received` | `call_index, segment_id, text, has_han, source_hash` -- the only source of a row's text |
| `discarded` | `call_index, segment_id, reason` -- reason in `duplicate_index, unclosed, empty, missing, no_finish, interrupted, retired` |
| `call_done` | `call_index, finish_reason: string\|null, usage: { prompt_tokens, completion_tokens, total_tokens, cost_usd: number\|null }\|null` |

Order per call: `call_started`, `token`*, one `received`/`discarded` per planned sentence of the call (planned order), `call_done`. `received`/`discarded` are sent only after the call's stream ended. A call whose sentences were all retired still gets `call_started`, `discarded(retired)` x n, `call_done` (usage null). Calls never reached (cancel, error before them) get no event at all: the webview marks their rows `not_sent` from `planned` once the command returns. No event carries `operation_id`.

Commands (invoke args are camelCase at top level only; keys inside `scope` stay snake_case):
- `ai_translate_batch({ scope, promptSetName, channel })`. `scope` is `{ kind: "selection", segment_ids: number[] }` or `{ kind: "chapter" }`. The old `segmentIds` / `skipTranslated` args are gone. Chapter mode = sentences with blank `target_text` (+ already translated neighbours as read-only context inside a call); selection mode = exactly those ids, translated or not, never context. Retry is always `selection` on `missing` + `not_sent` ids.
- `ai_translate_resend_call({ operationId, callIndex, channel })`: replays the recorded prompt of one call; events are those of a one-call operation with the original `call_index` (a `planned` with one call first). Needs the `operation_id` from `ai_prompt_read_record(...).call.operation_id`.
- Both return the outcome `{ state: "done", usage: null, maybe_truncated: false } | { state: "cancelled" } | { state: "not_configured" }`. Errors are `IpcError` (existing six families); params now include `call_index, first_segment_id, last_segment_id` (the old `segment_id` is gone from the batch path) plus the usual `status` / `provider_message` / `finish_reason`. The messages do not render these three yet: W5 composes the "call k, sentences a-b" text from params (the keys declare no placeholder for them).
- `ai_translate_segment` / `ai_translate_resend` outcome: `{ state: "done", usage, maybe_truncated: boolean }` (true when finish_reason is absent or not `"stop"`).
- `promote_ai_translation({ segmentId, targetText, force, sourceHash? })`: new optional `sourceHash`; mismatch => `IpcError` code `segment.source_changed`, key `err.segment.source_changed` (`{segment_id}`), nothing written. Pass the `received.source_hash` of the row.
- `ai_prompt_read_record({ segmentId? })`: with `segmentId` the newest record naming that segment as a sentence to translate wins (grouped call or one-sentence), else the newest call that has it as context only, else the old one-sentence record. Without it: old behaviour. `AssembledPromptWire` gained `call: null | { operation_id, call_index, segment_ids, context_segment_ids, requested_role: "translate"|"context", block_number: number|null }`. For a call record `prompt`/`system` are the call's, `system` always `Some`, `segment_id` is the requested one, `sent_usage` is the call's.
- Prompt text for the inspector highlight: blocks are `<<<SEGMENT n` ... `SEGMENT>>>` (one `source_segment` piece holds all blocks), context `<<<CONTEXT` ... `CONTEXT>>>` (content `source -> target` on one line), TM sub-block `<<<TM` ... `TM>>>` inside a SEGMENT block. A source line equal to a marker is stored with U+200B after its first character, so scanning for whole-line markers is safe.

## What changed

- `ports/translation_provider.rs`, `core/ai/client.rs`: `TranslateOutcome::Done { usage, finish_reason }`, no longer `Copy`. Proofreader matches `Done { usage, .. }`.
- NEW `core/ai/batch.rs` (pure): constants (`TRANSLATE_CALL_MAX_SEGMENTS=100`, `CONTEXT_GAP_MAX=3`, `TM_PAIRS_PER_SEGMENT`), marker constants, `guard_marker_line`/`unguard_marker_line`/`guard_text`, `plan_calls`, `BlockStream`, `reconcile`, `has_han`, `system_with_block_format`. SEGMENT markers moved here from rag.rs (same bytes).
- `core/ai/rag.rs`: `assemble_call_prompt` + `CallPromptEntry`.
- `commands/aiprompt.rs`: `OperationPromptContext` / `prepare_operation_prompt` (prompt set + system + TM rows, once), `assemble_call`, `AiCallRecord` / `AiCallRecords` / `AiCallRecordsState` (max 64, keyed `(operation_id, call_index)`), `read_prompt_record_for_segment`, shared record clock (`stamp` on `AssembledPromptRecord`), `ai_prompt_read_record(segment_id)`.
- `commands/aitranslate.rs`: old `PreparedBatchItem` / `prepare_batch_call*` / `run_batch_call` removed. New `AiTranslateScope`, `prepare_translate_operation`, `CallSource` trait (`OperationCallSource`, `ResendCallSource`), `StoreAccess` (`AppStores`), `run_translate_operation` (async seam), `operation_result_wire`, `batch_stopped_error(call_index, first, last, err)`, `ai_translate_resend_call`. `operation_id` = the `AiTranslateGeneration` value at operation start.
- `commands/segment/{targets,wire}.rs`: `promote_inner(.., expected_source_hash)`, `source_text_hash` (std `DefaultHasher`, hex), `segment_source_changed`; `core/i18n` key `SegmentSourceChanged` + `src/i18n/vi.json` `err.segment.source_changed`.
- `lib.rs`: registered `ai_translate_resend_call`, `app.manage(AiCallRecordsState)`, cleared on Work close, E7 bench call uses `Selection`.
- Gates: `ai_boundary.rs` (rag names 14 -> 16: `assemble_call_prompt`, `CallPromptEntry`; `AI_FLOOR` 4 -> 6; second seam marker `crate::core::ai::batch::` for `commands/aitranslate.rs` with 12 frozen names + its own names test), `config_invariants.rs` census (aitranslate 6 -> 7 plain, tree 109 -> 110), `ipc_contract.rs`, `ai_prompt_contract.rs` (`call` key).
- Tests: NEW `tests/ai_batch_wire.rs` (28 cases incl. AD-60 (a)-(u) minus webview halves of (n)/(s2)/(t)); `ai_translate_contract.rs` / `ai_translate_wire.rs` old batch cases deleted, config-state and error-family cases ported.

## Decisions a later reader needs

- `EmptyReply` (stream with only `[DONE]`) inside a call is a finished call without text: every sentence `discarded(no_finish)` (the port trait cannot expose the finish_reason of the error), operation goes on.
- Planner precedence when a cut is due: gap > 3 translated wins (new call carries the last 3 as leading context), else cap 100 (no leading context), else inline context. Translated sentences before the first untranslated one and after the last are never context.
- Prompt text and hashes come from the plan-time snapshot; the chapter is re-read per call only to retire sentences (gone, now omitted, or a different chapter open => all retired).
- Glossary is still gathered per sentence at call time (store read per sentence); TM rows, prompt set and system prompt are read once per operation (case (u) proves TM).
- Format instruction and TM lead line are English constants in `batch.rs`; the format instruction is appended in the command layer (`system_with_block_format`), so `OperationPromptContext` holds the raw resolved system.
- A batch no longer writes `LastAssembledPromptState` nor stamps `sent_*` there; the call record carries `sent_at`/`sent_usage` (`sent_model` mirrors the config model).

## Red by design

- `cargo test --test ipc_argument_contract`: `ai_translate_resend_call` has no `invoke()` in `src/**/*.ts` yet, and `ai_translate_batch` args changed. Green after W1.
- Existing vitest / TS for the old batch wire (`runAiTranslateBatchCall`, `aiTranslateBatchState.ts`, `chapterTranslateScope.ts`, handlers) is untouched and now mismatches the Rust wire. Not run.
- `cargo test --test ai_boundary -- --ignored` (compile probe) not run: Phase 4.

## Ran (this build: macOS, Homebrew rustc 1.99.1, dist/ present)

All green unless noted: `ai_batch_wire` 28 · `ai_translate_contract` 75 · `ai_translate_wire` 30 · `ai_boundary` 31 (+1 ignored) · `ai_prompt_contract` 31 · `ai_rag_contract` 50 · `ai_proofread_contract` 22 · `proofread_memory_contract` 13 · `ipc_contract` 48 · `segment_wire` 4 · `tm_contract` 152 · `config_invariants` 50. `ipc_argument_contract` red (above). `cargo clippy --all-targets` clean after boxing `PreparedOperation`. No red-control (real removal) run yet: all 14 AD-60 removal controls of the plan table are Phase 4 work; none of the cases has been proven able to fail.

## Fix: Glossary snapshot

- `core::glossary::GlossarySnapshot` (store.rs): `load(resolver, global, work)` reads both tiers once with the AD-18 override; `confirmed_terms(text, lang)` is the pure matching half. `confirmed_terms_for_injection` is now `load` + `confirmed_terms`, so the confirmed-only and pending-overlap rules have one body.
- `core::ai::rag`: `load_glossary_snapshot` (None when the body has no `{{glossary_terms}}`) and `gather_glossary_context_from_snapshot`; `gather_glossary_context` unchanged for the single-segment path.
- `OperationPromptContext` carries `glossary`, read in `prepare_operation_prompt`; `assemble_call` no longer takes the stores (caller in aitranslate.rs updated).
- `ai_boundary.rs` allow-lists widened by the new names only (`GlossarySnapshot`; the two rag functions in the aiprompt seam).
- New case `the_glossary_is_read_once_per_operation_so_an_edit_between_calls_changes_nothing` (ai_batch_wire.rs): a term is re-confirmed and another added after call 1; call 2 must keep the start-of-operation text. Counter-check: snapshot read moved back to per call, case red on "call 2 keeps the translation read at the start"; restored, green.

## Phase 3 (Webview)

Not committed. No Rust file touched. W1-W7 done.

### What changed

- `src/config/aitranslate.ts`: new event union (`planned|call_started|token|received|discarded|call_done`, all runtime-checked), `AiTranslateScopeWire`, `runAiTranslateBatchCall(scope, promptSetName, onEvent)`, `resendAiCall(operationId, callIndex, onEvent)`; outcome `done` requires `maybe_truncated: boolean`. The two invoke sites are literal objects (`ipc_argument_contract` scanner rejects spreads and `CMD_*` passed to helpers).
- `src/aiTranslateBatchState.ts` rewritten: row `{segmentId, callIndex, status: queued|streaming|received|missing|not_sent, reason, text, hasHan, sourceHash}`; rows come from `planned`; `received`/`discarded` are the only text source (discarded clears streamed text); usage kept per `"<run>:<call_index>"` (one per call, retry runs add); after the command returns `queued -> not_sent`, `streaming -> missing(interrupted)`; `runAiTranslateBatch(set, ids|null)` (null = chapter), `retryAiTranslateBatch`, `resendAiTranslateCall`, `aiTranslateBatchMayRetry`, `aiTranslateBatchErrorSpan` (reads `call_index/first_segment_id/last_segment_id` from error params), `aiTranslateBatchResultForSegment`. No call-size constant.
- `src/chapterTranslateScope.ts` deleted (worktree only). Chapter button is now count-less and enabled whenever the Editor has loaded sentences (nothing in TS knows which are untranslated).
- `aiTranslateHandlers.ts`: chapter -> `runAiTranslateBatch(set, null)`; retry allowed on done/cancelled/retryable-error when missing+not_sent ids exist; promote of a batch row passes `sourceHash`; resend with `record.call !== null` goes through `resendAiTranslateCall(operation_id, call_index)`.
- `config/segment.ts` `promoteAiTranslation(.., sourceHash?)` sends `sourceHash: sourceHash ?? null`; `editorPanelState.ts` threads `sourceHash` through single/batch/confirm paths (`PromoteBatchItem.sourceHash` required, `PendingPromote.sourceHash`).
- `aiTranslateState.ts`: `aiTranslateMaybeTruncated`. Panel shows `maybe_truncated` (single run) and `has_han` (row) flags, missing/not_sent counts line, running call, error "call k (sentences a-b)" from params, retry button "Thu lai N cau".
- `config/aiprompt.ts`: `AssembledPromptCallWire` + `call` in the checked shape, `aiPromptReadRecord(segmentId?)` (conditional invoke so no-arg calls stay argument-free); `aiPromptInspectorState.ts`: `refreshAiPromptRecord` passes `editorActiveSegmentId`, `aiPromptCallNote`, `aiPromptHighlightParts`; `AiPromptInspectorOverlay.vue` renders note + highlighted block (`aip-piece-mine`).
- `vi.json`: row status keys replaced (queued/streaming/received/missing/not_sent), added `batch_gaps`, `batch_retry_count`, `flag_has_han`, `flag_maybe_truncated`, `ai.prompt_inspector.call_context_only|call_block`; batch usage texts now say "luot"; removed 11 dead keys.

### Ran (macOS, this tree)

- vitest, the 11 plan files plus `aiTranslate`, `aiProviderMessageLine`, `aiPromptConfigGuards`, `editorNavNotice` and the 10 other files that import the touched modules: all green (252 + 166 tests in two runs; the only failure found, `aiPromptConfigGuards`, fixed by adding `call`).
- `npm run check:i18n check:commands check:panel-refs check:lint check:tokens` exit 0; `npm run build` exit 0.
- `cargo test --test ipc_argument_contract` green (was red by design; Rust untouched).
- Counter-checks (real removal, reverted): usage keyed without `call_index` -> `a call_done repeated for the same call_index is counted once` red; appending `const MAX = 100` to the state file -> `no call-size constant` red.

### Red or unverified

- Not run: the full vitest suite, `check:gates`/other gates, e2e, anything Phase 4 lists. Counter-checks for (n) has_han (webview only forwards the flag, covered by the panel flag case but not removal-checked), (h)/(h2) etc. are Rust-side Phase 4 work.
- Behaviour change to confirm with Ice: "Dich ca Chuong" is no longer disabled when everything is translated (webview cannot know); Rust returns an empty `planned` and the state ends `done` with no rows.
- No resend button exists on the panel itself; resend of a call is only the inspector "Gui lai" (via `record.call`). Operation id of the newest call only comes from `ai_prompt_read_record`.
- Inspector result tab still shows one text per segment (`aiTranslateBatchTextForSegment`), unchanged.

## Phase 4 (Tests that move)

Not committed. Production code unchanged except the restore below.

- Restored `src/chapterTranslateScope.ts` (from fd7fec3), panel count label `chapter_run_count` (vi.json key back) and button disabled at N = 0, handler `runAiTranslateChapter` makes no call when N = 0 (still sends `scope: chapter`). Tests: `aiChapterRun.test.ts` (count by shape, no call at 0), `aiChapterRunPanel.test.ts` (label with N, disabled at 0). Counter-check: guard in handler removed -> "makes no call" red.
- Real removals (file patched, `cargo test --test ai_batch_wire` run, file restored, diff vs backup empty): (a) (b) (c) (d) (e) (f) (g) (h before-call) (h CallDone on cancel) (h2 Received on block close) (i) (j) (k) (l) (m) (n) (o) (p) (q) (r) (s; also `segment_wire`) (s2) (t) (u) and the single-sentence `maybe_truncated` flag: every one red for its own case. (u) needed the two `OperationPromptContext` fields made pub during the probe only (reverted). (n) webview half: `hasHan: false` in the state -> 2 vitest red. Script: scratchpad `ctl.py`.
- `cargo test --test ai_boundary -- --ignored`: 1 passed (compile probe).
- Debt item `Chủ: Epic 4` appended to `deferred-work.md`; 🔵 line added to story 4.9 plan (first error rule now per AD-60 rules 7-8).
- Full `cargo test` then full `vitest run` were started once, logs in scratchpad `full-cargo.log` / `full-vitest.log`; result not read when this agent ended.
- Not done: CI and nightly e2e (nothing pushed), nothing committed or pushed.
