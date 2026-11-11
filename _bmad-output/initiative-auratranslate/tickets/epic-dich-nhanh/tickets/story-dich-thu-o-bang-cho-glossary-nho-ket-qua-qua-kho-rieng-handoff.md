# Handoff pha Rust -> pha Webview (Story 15-1)

Rust done and verified. Webview may rely on the contract below; nothing else changed on the wire.

## Commands (all in `src-tauri/src/commands/aitranslate.rs`, `mod wire`)

- `ai_context_reference_translate(context_example: string, run_id: number (u64), retranslate: boolean, channel)`
  -> `QuickTranslateOutcomeWire`, tagged `state`:
  `not_configured` | `store_unavailable` | `cached {text, model}` | `done {model, usage: AiTranslateUsageWire|null, maybe_truncated}` | `cancelled`.
  Errors still leave as `IpcError` (invoke rejects). Tokens stream on `channel` as plain strings, as before.
  `done` with `maybe_truncated: true` was NOT stored. The streamed text IS the result (no text in `done`).
  `cached` carries the text; nothing is streamed. Without `retranslate` a store hit returns `cached` even if the call is made blindly; with `retranslate: true` it always sends and REPLACEs the row.
  Order: store missing -> `store_unavailable`; cache (skipped by `retranslate`); config -> `not_configured`; send.
  So `cached` is returned even when AI is not configured.
- `ai_context_reference_cancel(run_id: number)` -> void. Does nothing unless `run_id` equals the active run (a stale or wrong id is a no-op). The webview issues an increasing `run_id` per `Dịch thử` and must send the SAME id on cancel. A new `ai_context_reference_translate` supersedes the previous run.
- `quick_translation_lookup(text: string)` -> `{state: 'miss'} | {state: 'hit', text, model} | {state: 'store_unavailable'}`. Plain (sync) command, reads one row, writes nothing. Not merged into `glossary_context_translation` (unchanged; Work translation lookup stays first on the webview side).

Parameter names are pinned by `tests/ipc_contract.rs::the_context_reference_wires_are_registered_and_keep_their_parameter_names`. Tauri maps them camelCase on invoke: `contextExample`, `runId`, `retranslate`, `channel`; `runId`; `text`.

## What the webview owes (plan, pha Webview)

- Order on opening a row: `glossary_context_translation` (Work) -> `quick_translation_lookup` -> nothing (button `Dịch thử`).
- `store_unavailable` from either command -> `Không mở được kho dịch nhanh.` + disabled button.
- `usage` exists only in `done` of the run just made (not for `cached`); show it with `aiUsageLine`.
- The old `isAiTranslateOutcomeWire` check no longer fits the translate result (new tags `cached`, `store_unavailable`); the adapter needs its own guard. `AiTranslateOutcomeWire` itself is unchanged and shared with the real batch.
- No `busy` state yet (15.3).

## Surprises / decisions

- `ContextReferenceGeneration`, `run_plain_call` and `ReplyShape::Plain` were removed (no remaining caller). The send now lives in `quick_translate<P: TranslationProvider>` (generic, testable with `Fake`).
- Save failure after a paid call is logged and the outcome is still `done` (text was streamed); lookup read failure -> `store_unavailable`, no network.
- `quick_translation_lookup` is a plain command: `tests/config_invariants.rs` census row for `aitranslate.rs` is 10 plain (note says unmeasured) and tree total is 125/80.
- `QuickTranslationStore::count` exists only for tests and the later Dọn story.

## Known red until the Webview phase

Full `cargo test --no-fail-fast` (after the Rust phase): 112 targets green, one red: `tests/ipc_argument_contract.rs::every_registered_commands_business_parameters_match_camel_case_across_the_wire`, "quick_translation_lookup" has no `invoke()` caller in `src/**/*.ts`. It turns green when the adapter in `src/config/glossaryContextReference.ts` calls `invoke('quick_translation_lookup', { text })`. Re-run that one target after the Webview phase; the existing translate/cancel adapters must also start passing `runId` (and `retranslate`).
`npm run build` before `cargo test`; the e2e phase must rebuild with `--features wdio` (cargo test overwrites the wdio binary).

# Handoff pha Webview -> pha Tests that move (Story 15-1)

Webview done. vitest (glossaryQueue, glossaryContextReference) 74 green; `check:commands`, `check:i18n`, `check:tokens`, eslint on touched files, vue-tsc green; `cargo test --test ipc_argument_contract` green now.

## DOM hooks for e2e (GlossaryQueueOverlay.vue, inside `[data-gq-context-ref]`, `data-kind` = kind)

- kinds: `chapter | none | failed | running | ai | cached | truncated | store_unavailable | not_configured` (`looking` draws nothing).
- `[data-gq-context-ref-text]` text; `[data-gq-context-ref-line]` is `role="status"` and holds `[data-gq-context-ref-source]` (label, or `Đang dịch…` / truncated text), `[data-gq-context-ref-model]` (`<model> ·`, only `ai` and `cached`), `[data-gq-context-ref-usage]` (only `ai`/`truncated`, i.e. the run just made).
- Buttons: `[data-gq-context-translate]` (`Dịch thử`), `[data-gq-context-retranslate]` (`Dịch lại`, kinds ai/cached/truncated), `[data-gq-context-cancel]` (`Huỷ`, running). `[data-gq-context-store-unavailable]` is the broken-store text; its translate button is disabled.
- The old kind `ai` still exists but now also carries `model` and `usage`; a reopened Work with a stored sentence shows `cached`, not `ai`. Existing e2e `e2e/specs/glossary-context-reference.e2e.mjs` still uses `data-gq-context-*` hooks and was NOT run or updated in this phase; check it.

## Behaviour

- Open row: `glossary_context_translation` -> `quick_translation_lookup` -> `none`. Lookup adapter error -> `failed`.
- `run_id` is a module counter in `glossaryQueueState.ts` (never reset); the same id goes to `ai_context_reference_cancel`.
- `Dịch lại` cancelled returns to the previous `cached`/`ai`/`truncated` entry; a failed `Dịch lại` shows `failed` (store row untouched).
- Esc: running -> `glossary.context.cancel` only; otherwise `glossary.queue.close` (`onEscape` in overlay, registered in `HANDLER_TABLE` of `scripts/check-commands.mjs`). Focus moves to `Huỷ` on running and returns to `Dịch lại`/`Dịch thử` (else the panel) when focus fell to body.
- New command `glossary.context.retranslate` (i18n `command.glossary.context.retranslate`); new keys `glossary.queue.context_{retranslate,model,truncated,store_unavailable}`.
- Adapter signatures changed: `translateContextReference(context, runId, retranslate, onToken)`, `cancelContextReference(runId)`, new `quickTranslationLookup(text)`.

## Not done / for the next phase

- Real-app behaviour (WebKit focus, streaming) unverified; e2e spec 15-1 and the plan-cau-dich 🔵 note still open. Remember `--features wdio` rebuild after any `cargo test`. Full `cargo test` once before reporting the story done (plan Verification).
