# Locator digest, area "rest" (all but src/ and src-tauri/) @ 7bd1f90

(Returned inline by a read-only locator; filed by the lead.) Scope: tests/ (246 files under tests/frontend, fixtures, AGENTS.md), e2e/ (51 specs, 11 support, wdio.conf.mjs), scripts/, tools/, .github/, .githooks/, top-level files. `docs/` not tracked at this commit. Here AI translation/Proofreader exist only as webview tests and fakes, CI wiring, one measuring script.

## Q1
- tests/frontend/aiPromptInspector.test.ts:433 @ 7bd1f90 — prompt is `ledger.pieces` kinds authored/glossary/source_segment/tm, joined = `prompt` byte for byte; `record()` fixture :144 `system: null`.
- tests/frontend/aiPromptInspectorScreen.test.ts:338 @ 7bd1f90 — system block before tier blocks; system null or "" ⇒ no system message (:355, :363).
- tests/frontend/aiConfigSystemPromptAdapter.test.ts:16 @ 7bd1f90 — `system_prompt` field (default + Global/Work tiers).
- tests/frontend/aiTranslate.test.ts:265 @ 7bd1f90 — tokens appended in order (:286); outcome `{state:'done', usage}` as invoke return (:319); not_configured/cancelled/error, non-2xx `provider_refused {status}` (:561).
- tests/frontend/aiTranslateResendCall.test.ts:38; tests/frontend/aiProviderMessageLine.test.ts:65 (`provider_message` single :86, batch :112).

## Q2
- tests/frontend/aiTranslateBatch.test.ts:1 @ 7bd1f90 — events `{kind:'token'|'done'|'skipped', segment_id}`; `segment_id` matches results to segments; `pendingBatchRun` :121; progress :222; skipped for `is_omitted` :465; cancel mid-batch :494 (done rows keep text, running dropped, later never called); error mid-batch :536 stops batch; retry :587 only failed+unrun; single/batch mutual exclusion :855; frozen selection :765.
- tests/frontend/aiBatchPanelRows.test.ts:274 — cancel stops at segment boundary; discard :333; promote :480.
- tests/frontend/aiChapterRun.test.ts:52 — one `ai_translate_batch` with blank non-omitted ids in document order, `skipTranslated: true` (:57); selection `false` (:89).
- tests/frontend/aiTranslateBatchResetWiring.test.ts:194 (:253, :307); tests/frontend/aiTranslate.test.ts:1045; tests/frontend/aiTranslateHandlersWiring.test.ts:11.

## Q3
- tests/frontend/proofreadConfig.test.ts:76 — `ai_proofread_segments` with `segmentIds` + channel; events :52; cancel `ai_proofread_cancel` :114.
- tests/frontend/proofreadState.test.ts:93 — `planned{segments,batches}` first, then `batch{usage, segments:[{segment_id, scanned_text, findings, unlocated, ambiguous}]}` (fixture :31); cancel keeps arrived findings :149; IpcError `not_scanned` :130; stale/late dropped :196; edit clears result :247.
- tests/frontend/proofreadWiring.test.ts:56 (scope caret/selection/chapter; :132), proofreadActions.test.ts:111, proofreadMemory.test.ts:87 (:124), proofreadScopeOverlay.test.ts:60, proofreadDecisionRatio.test.ts:95.
- scripts/measure-proofread-false-alarm.mjs:1, scripts/lib/falseAlarm.mjs:27 (listScans) — read-only false-alarm measure over `proof_scan` in project.db; tested tests/frontend/falseAlarmRate.test.ts:69.
- e2e/specs/proofread-underline.e2e.mjs:142 — wire cannot be stubbed, real scan needs keychain; calls draw helper directly.

## Q4
- tests/frontend/aiTranslate.test.ts:352 — usage `{prompt_tokens, completion_tokens, total_tokens, cost_usd}`; :407 `cost_usd: null` ⇒ no price row, tokens only; :435 no usage reported as such, never 0.
- tests/frontend/aiTranslateBatch.test.ts:274 (:359 partial, :396 missing cost ⇒ tokens only); aiBatchPanelRows.test.ts:310; aiUsageVisibleState.test.ts:31 (on by default, `app_config` Global); aiPromptInspectorScreen.test.ts:261 (:326); proofreadState.test.ts:163; proofreaderStrip.test.ts:24 (:61); aiPromptConfigGuards.test.ts:368 (:446, :456).
- Price data: none in area. Commit bb902f9 body: built-in table keyed by model id with `PRICES_AS_OF` (in src-tauri).

## Q5
- tests/fixtures/prompt-variable-markers.json:2 — known variables exactly `glossary_terms`, `source_segment`, `tm_similar_segments`; read by src-tauri/tests/prompt_set_contract.rs:745 and tests/frontend/promptLibraryOverlayRender.test.ts:727.
- tests/frontend/aiPromptInspector.test.ts:300 — glossary ledger not_asked/asked + injected (:484); considered-not-injected (:556); TM not_asked/searched + reference label (:602); unknown markers like `{{chapter_context}}` reported, not filled (:1461).
- tests/frontend/aiPromptInspectorScreen.test.ts:607 — pending glossary terms, >3 TM pairs, `hidden_count`.
- tests/frontend/promptSetState.test.ts:164 (Work set shadows Global same name); promptTemplates.test.ts:22 (29 templates, all three variables; :32 no template repeats the system prompt); aiConfigState.test.ts:1274 (per-Work system prompt override); tmPolicy.test.ts:36.
- Neighbouring segments: none. Search `neighbo|câu trước|câu kề|previous segment|surrounding|chapter_context` over tests, e2e @ 7bd1f90: only unknown-marker case and aiPromptInspector.test.ts:1643 (glossary counts vs previous assembly).

## Q6
- tests/AGENTS.md:6 — four acceptance paths: static gates, `src-tauri/tests/**`, vitest, `e2e/**`.
- package.json:27 `vitest run`; :28 `test:story`; :32 `test:e2e` (wdio, `wdio` cargo feature).
- vitest uses fakes at IPC boundary, no recorded wire data, no live provider: `vi.mock('../../src/config/aitranslate')` + fake Channel (aiTranslateBatch.test.ts:47; `pendingRun` in aiTranslate.test.ts); `invoke` mocked (aiChapterRun.test.ts:28, proofreadConfig.test.ts:4, proofreadWiring.test.ts:35); `config/proofread` mocked (proofreadState.test.ts:10).
- tests/frontend/aiConfigConnectionTest.test.ts:66 faked; tests/frontend/support/ai-config-validity.json shared with src-tauri/tests/aiconfig_contract.rs:594.
- .github/workflows/ci.yml:337 npm test; :353 cargo test --locked; :388 FR77 step runs ignored `ai_boundary` test `deleting_core_ai_and_its_three_approved_seams_leaves_the_rest_of_the_tree_compiling`, fails unless exactly one ran; :809 e2e job macOS nightly/dispatch.
- .githooks/pre-push:64 / :70 vitest + cargo test --locked; AGENTS.md:27 dev loop.
- e2e/specs/story-12-15-proofread-ratio-reading-marks.e2e.mjs:66 — invokes accept/skip/revert directly, no provider call.
- Live provider: none. Commit bb902f9: "hai phép kiểm tay cần endpoint thật (chưa ai chạy)".
- Rust tests named from here: `ai_boundary`, `ai_translate_wire` (521636e), `ai_translate_contract.rs`, `prompt_set_contract.rs`, `aiconfig_contract.rs`.

## Q7 (from git log of test files; historian to confirm)
- 2376aeb 2026-09-21: `TranslationProvider`, OpenAI-compatible async streaming client, tokens over `tauri::ipc::Channel`, send path sends exactly the recorded assembled string.
- 915f1af 2026-09-22: batch — keychain read once, `segment_id` events over one Channel, cancel at segment boundary via generation counter, stop at first error.
- d1451a8 2026-09-22: six error-family keys + `segment_id`.
- bb902f9 2026-09-22: usage via `stream_options`; price table keyed by model id (`PRICES_AS_OF`); batch usage in `Done{segment_id}`.
- b61e87e 2026-10-05: TM pairs into `{{tm_similar_segments}}`, at most 3, own first; batch reads TM once holding OpenWorkState.
- 0acd35a 2026-10-10; d31e95b 2026-10-10 (`prepare_translate_call` / `prepare_batch_call` via `resolve_call_config`, `read_api_key`).
- dc8b39d 2026-10-11: Proofreader batches 4,000 source+target chars or 20 segments; tiktoken: 153-token fixed prompt per call; 184-segment Chapter ~14k input tokens batched vs ~40k one by one; cancel before each batch; broken batch adds to `not_scanned`.
- 983ef03 2026-10-26: shared editable system prompt as system message before prompt-set body (before: one user message); Proofread does not send it.
- 521636e 2026-10-26: whole Chapter via `ai_translate_batch` + `skip_translated`; Rust re-filters against DB.

## Searched, nothing found @ 7bd1f90
- `price|pricing|litellm|PRICES_AS_OF` in tests/scripts/tools/.github/top-level: only fixture `cost_usd`.
- `createServer|chat/completions|v1/messages|ollama|wiremock|cassette` in e2e, tests: none.
- concurrency settings; commit-message search `song song|đồng thời|concurren|tuần tự|sequential`: only unrelated.
