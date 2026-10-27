# Locator digest, area `src/` @ 7bd1f90

(Returned inline by the locator because its harness was read-only; filed verbatim in substance by the lead.)

Overall: `src/` never builds a provider request or calls a provider. It sends Tauri `invoke` IPC commands and reads results back through Tauri `Channel` frames. Prompt assembly, system/user messages, batching, retries and pricing are in Rust (`commands/aitranslate.rs`, `commands/aiprompt.rs`, `core::ai::pricing`, as named in webview comments).

## Q1
- `src/config/aitranslate.ts:115 (runAiTranslateSegment) @ 7bd1f90` — `invoke('ai_translate_segment', { segmentId, promptSetName, channel })`; only id + prompt-set name cross the wire. Tokens arrive as bare strings on `Channel<string>.onmessage`. Outcome `AiTranslateOutcomeWire` (`:63`): `not_configured` | `done{usage}` | `cancelled`; errors via `catch` as `IpcError`.
- `src/config/aitranslate.ts:144 (runAiTranslateResendCall) @ 7bd1f90` — `ai_translate_resend` with only `channel`; Rust resends the prompt it recorded.
- `src/aiTranslateState.ts:124 (runTranslateWith) @ 7bd1f90` — appends tokens into `accumulatedText`; `sequence` guard drops late tokens; maps outcome to five states; keeps `usage` only on `done`. `:103 (runAiTranslate)` entry point for caret segment.
- `src/config/aiprompt.ts:120 (AssembledPromptWire) @ 7bd1f90` — Rust's record: `system: string | null` (system message before `prompt`), `prompt` (user message), `ledger.pieces` ordered list of `PromptPieceKindWire` `'authored' | 'glossary' | 'source_segment' | 'tm'` (`:91`), joined pieces equal `prompt` byte for byte; `sent_at`, `sent_model`, `sent_usage`. Commands `ai_prompt_assemble` (`:304 aiPromptAssemble`), `ai_prompt_read_record` (`:346 aiPromptReadRecord`).
- `src/aiPromptInspectorState.ts` @ 7bd1f90 — read-only over the record: `:212 assembleCurrentAiPrompt`, `:329 aiPromptSystemText` (empty system = none), `:343 aiPromptTierBlocks`, `:388 aiPromptSourceText`, `:426 aiPromptResultFor`.
- `src/AiPromptInspectorOverlay.vue:447 @ 7bd1f90` — renders system block then tier blocks.

## Q2
- `src/config/aitranslate.ts:208 (runAiTranslateBatchCall) @ 7bd1f90` — one `invoke('ai_translate_batch', { segmentIds, promptSetName, skipTranslated, channel })` per batch over one Channel; frames `AiTranslateBatchEventWire` (`:177`) `token` | `done{usage}` | `skipped`, each carrying `segment_id`; malformed frame dropped with console message.
- `src/aiTranslateBatchState.ts @ 7bd1f90` — header `:1-50`: Rust `run_batch_call` translates segments one after another, at most one row `running`. `:246 runAiTranslateBatch` builds rows in document order and `rowIndexBySegmentId` (`:262`). `:290 streamBatch` row status pending→running→done/skipped; error row named by `IpcError.params.segment_id` (else running row) → `error`; cancel → running row `cancelled`, text cleared (`:350`). `:224 aiTranslateBatchRetryIds` re-sends only error+pending rows. `:268 retryAiTranslateBatch` keeps finished rows. `:197 aiTranslateBatchErrorBlock` batch stops at first error. `:365 cancelAiTranslateBatch`, `:381 resetAiTranslateBatch`.
- `src/aiTranslateHandlers.ts:41 (aiTranslateHandlers) @ 7bd1f90` — single and batch exclude each other; batch uses `segmentSelectionIds`; `runAiTranslateChapter` (`:111`) passes `chapterUntranslatedIds` with `skipTranslated = true`; retry gated on `err.retryable` (`:119`, `:145`); cancel calls both (`:68`).
- `src/chapterTranslateScope.ts:7 (chapterUntranslatedIds)` — not omitted and blank target.
- `src/aiTranslateCancel.ts:3 (cancelWhenGenerating)`, `src/config/aitranslate.ts:258 (cancelAiTranslateCall)` — `ai_translate_cancel` no args, bumps a generation counter in Rust.
- `src/aiTranslateRunClock.ts:3 (nextAiTranslateRunStamp)`; `src/panels/segmentSelectionState.ts:81 (segmentSelectionIds)`; `src/panels/AiTranslationPanel.vue:143 (chapterUntranslatedCount)`, `:325 (canRetryAiTranslateBatch)`; `src/commands/index.ts:5193` command ids `ai.translate.batch_run`, `chapter_run` (`:5205`), `batch_retry` (`:5239`); `src/main.ts:1105` wiring.
- Concurrency: no limit, `Promise.all` or backoff on this path in `src/`.

## Q3
- `src/config/proofread.ts:124 (runProofreadSegments) @ 7bd1f90` — `invoke('ai_proofread_segments', { segmentIds, channel })`; events `ProofreadEventWire` (`:45`): `planned{segments, batches}`, `batch{usage, segments: ProofreadSegmentResultWire[]}` with `scanned_text`, findings UTF-16 `start`/`end`, `kind`, `explanation`, `suggestion` (`:19-34`). Outcome (`:49`) `not_configured` | `cancelled{not_scanned}` | `done{not_scanned, filtered, blocked}`.
- `src/proofreadState.ts @ 7bd1f90` — `:165 runProofread` (flush Editor, call Rust, `not_scanned` from `err.params`), `:133 applyEvent` (stores per segment, sorts findings, skips segments edited during scan, sums usage), `:237 cancelProofread`, `:110 proofreadUsageSummary`.
- `src/proofreadScope.ts:9 (scannableIds)`, `:17 (proofreadScopeIds)`; `src/proofreadHandlers.ts:25 (scan)` refuses while a translation generates; `src/ProofreaderStrip.vue:166 (usageText)`.
- Batch sizes not in `src/`; webview sees only `planned.batches`.

## Q4
- `src/config/aitranslate.ts:35 (AiTranslateUsageWire) @ 7bd1f90` — `prompt_tokens`, `completion_tokens`, `total_tokens`, `cost_usd`; `cost_usd: null` = model not in Rust's bundled price table `core::ai::pricing`; `usage: null` on `done` (`:63`) = provider sent no usage.
- `src/aiUsageLine.ts:6 (formatUsdParam)`, `:11 (aiUsageLine)`, `:39 (aiUsageSummaryLine)`.
- `src/aiTranslateBatchState.ts:139 (aiTranslateBatchUsageSummary)` — sums reporting rows; cost only if every reporting row has cost.
- `src/proofreadState.ts:110`, `:151`; `src/panels/AiTranslationPanel.vue:222`, `:254`, `:611`; `src/aiPromptInspectorState.ts:402 (aiPromptTokenLine)`, `src/AiPromptInspectorOverlay.vue:255`, `:671`; `src/ProofreaderStrip.vue:166`; `src/aiUsageVisibleState.ts:10`; `src/SettingsOverlay.vue:246`; `src/i18n/vi.json:136-138, 155-158, 197-200, 2123-2126`.

## Q5
- `src/promptTemplates.ts:63 (frame)`, `:76 (PROMPT_TEMPLATES) @ 7bd1f90` — 29 genre templates: guidance then markers `{{glossary_terms}}`, `{{tm_similar_segments}}`, `{{source_segment}}`. `src/promptSetState.ts:291 (addPromptTemplates)`.
- `src/promptSetState.ts:215, :226 (applyGenre), :239 (restorePromptSetForWork)` — per-Work choice: saved set, else genre's set, else none; `src/config/promptset.ts:216, :237`.
- `src/config/promptset.ts:22-50` — Global and Work tiers, body text, warnings for unknown markers / missing `{{glossary_terms}}`.
- `src/promptMarkers.ts:9 (splitPromptBody)` — same rule as Rust `scan_markers`.
- `src/config/aiconfig.ts:14 (AiConfigField)`, `:51 (AiConfigGetWire)` — `system_prompt` two tiers plus `default_system_prompt`; temperature, max_tokens. `src/aiConfigState.ts:162, :441, :457`.
- `src/config/aiprompt.ts:59 (GlossaryInjectionStatusWire)`, `:81 (TmInjectionStatusWire)` — glossary injected / suppressed by pending overlap / matched pending; TM similar segments side mine/others, percent, `reference`, `omitted_count`/`hidden_count`.
- `src/config/glossary.ts:813 (GlossaryEntry.is_shadowed)` — shadowed Global entry not put into prompt.
- `src/tmPolicyState.ts:1` — per-Work "only my pairs" switch.
- Neighbouring segments: none found; no marker or wire field.

## Q6
- No tests in `src/`. Outside area: 42 files under `tests/frontend` (e.g. `aiTranslate.test.ts`, `aiTranslateBatch.test.ts`, `aiChapterRun.test.ts`, `aiTranslateResendCall.test.ts`, `aiPromptInspector.test.ts`, `proofreadState.test.ts`, `proofreadConfig.test.ts`, `proofreadWiring.test.ts`), 3 e2e specs (`e2e/specs/proofread-underline.e2e.mjs` + 2); fakes via `vi.mock('../../src/config/aitranslate')` and a fake Channel (`tests/frontend/aiTranslate.test.ts:58`). Commit 78bf298 mentions Rust tests using MockRuntime with SSE loopback.

## Q7 (commits seen by the locator; historian to confirm)
- `2376aeb` 2026-09-21 single-segment streaming over Channel (4-8). `df5e576` 2026-09-18 assembled-prompt record (4-7). `915f1af` 2026-09-22 batch translate selected segments (4-9). `d1451a8` 2026-09-22 per-cause errors + retry (4-10). `bb902f9` 2026-09-22 tokens + cost estimate (4-11). `0acd35a` 2026-10-10 (9-9) retry keeps done rows, monotonic clock. `d31e95b` 2026-10-10 (9-12) shared `resolve_call_config`, temperature/max_tokens omitted when unset. `b61e87e` 2026-10-05 (7-11) Smart RAG similar TM pairs into prompt. `78bf298` 2026-10-15 (12-12) prompt inspector, `sent_usage`, `ai_translate_resend`. `983ef03` 2026-10-26 shared system prompt as system message (before: user message only). `7ed0c1a` 2026-10-26 default system prompt forbids Markdown. `521636e` 2026-10-26 Translate whole Chapter, `skip_translated`. `c186f8e` 2026-10-26 29 genre templates.
- Proofreader: `ed7b7f1` (9-1), `cda435d` (9-2), `dc8b39d` (9-3) batching 4,000 source+target chars or 20 segments, measured with tiktoken: ~153 fixed tokens per call, 184-segment chapter ~14k input tokens vs ~40k one by one; cancel checked before each batch. `3e0c02f`, `44653f1`, `31d6087` later.

## Searched, found nothing (@ 7bd1f90, `src/`)
- `role: 'system'|'user'`, `chat/completions`, `fetch(` on AI path, `concurrency`, `p-limit`, `backoff`.
- `pricing|price` beyond display strings/comments.
- `neighbo|surrounding|context_segments|prev_segment` as prompt context.
