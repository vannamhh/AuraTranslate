---
title: 'Codebase research: AI translation pipeline'
type: codebase
ask: |-
  nghiên cứu lại cách dịch với các mô hình AI làm sao để tối ưu nhất, chi phí thấp và dịch chuẩn, đúng ngữ cảnh. Phân tích cách làm hiện tại của app và khả năng tối ưu hơn
repository: '/Users/hoangnam/LocalSites/addon/AuraTranslate'
commit: '7bd1f90ca2ccd15b661850f3cfd60359df6bdc66'
branch: 'master'
status: complete
validation: 'normal'
claims: {verified: 17, disputed: 1}
created: '2026-10-27'
updated: '2026-10-27'
---

# Codebase research: AI translation pipeline

Readers: the plan step that will change the AI translation code, and a person checking one claim. Every claim is cited to a line at `7bd1f90` or to a commit; nothing here recommends a change. The companion web research, which does, is `../research-llm-translation-cost-and-quality/research-llm-translation-cost-and-quality.md`.

## Questions

1. (how) How is a request to an AI provider for translating a single segment assembled — which pieces of text go into the system and user messages, in what order — and how is the provider's response read back into a translation?
2. (how) How does translating several segments or a whole chapter with AI run — how segments are grouped into requests, how many requests run at once, how retries and cancellation behave, and how each returned translation is matched back to its segment?
3. (how) How does the Proofreader run — what it sends to the provider, how it splits work into batches, and what it does with each reply?
4. (where) Where are token counts and money cost of AI requests computed and shown, and where does the price data come from?
5. (what touches) What code contributes text to the prompt sent to the AI provider — glossary entries, translation memory, prompt sets/templates, per-Work settings, neighbouring segments, anything else?
6. (how) How is the AI translation and Proofreader path tested — which test targets exercise it, against what (fakes, recorded wire data, live providers)?
7. (why) Why are the request shape and the batching of AI translation the way they are — which commits introduced and changed them?

Researchers were fresh subagents given only these questions, the commit and the excluded paths (`_bmad-output/`, `_bmad/`, `.claude/`, `.agents/`, `.agent/`). Sections 2, 3, 4 and 6 are written from locator digests rather than analyzer reads, so all their claims went to verifiers.

How to read the citations:

- A bracketed `[n]` marks a claim in this run's verifier ledger, `.memlog.md` beside this file. Digests are in `digests/`. These numbers are not the source numbers of the companion report.
- Verification result: 17 claims verified and 1 disputed. The disputed one is [1]: translated rows are skipped only when `skip_translated` is set. The wording below already says so.
- When one symbol is cited at two lines, the first line is the definition and the second is the provider call inside it (for example `run_translate_call` at `:670` and `:676`).

## 1. How is a single-segment request assembled and its reply read back?

The webview assembles no prompt. It sends only a segment id and a prompt-set name (`src/config/aitranslate.ts:115 (runAiTranslateSegment) @ 7bd1f90`), and Rust builds the request:

- `src-tauri/src/commands/aitranslate.rs:388 (prepare_translate_call) @ 7bd1f90` — resolve config (`:310 resolve_call_config`), reject an omitted segment (`:417`), read the API key (`:421`), `assemble_and_record_prompt` (`:425`), giving `PreparedTranslateCall{prompt, system}` (`:433-441`).
  - `src-tauri/src/commands/aiprompt.rs:485 (assemble_and_record_prompt_with_tm) @ 7bd1f90` — resolves the prompt set by name (`:500-505`), takes the segment's `source_text` (`:512`), gathers glossary (`:515`) and TM (`:523-528`), calls `assemble_prompt(&set.body, sentence, glossary, tm)` (`:529`), takes `system` from `resolve_system_prompt` (`:530-532`), and stores both in `AssembledPromptRecord` (`:534-553`).
- `src-tauri/src/commands/aitranslate.rs:670 (run_translate_call) @ 7bd1f90` — builds `TranslateRequest` and calls `provider.translate`, forwarding each token over a `Channel<String>` (`:685-689`).

**Messages.** `build_request_body` sends an optional `system` message (skipped when `None` or blank), then exactly one `user` message carrying `request.prompt` verbatim; there is no assistant turn, no few-shot message and no history [9] (`src-tauri/src/core/ai/client.rs:333 (build_request_body) @ 7bd1f90`, `:335-344`).

**System message.** `resolve_system_prompt` returns the winning tier's (Work over Global) `system_prompt` row, else `DEFAULT_SYSTEM_PROMPT`; a blank row means no system message (`src-tauri/src/core/aiconfig/mod.rs:235 (resolve_system_prompt) @ 7bd1f90`, `:229-243`). The default, included with `include_str!` (`:231`), is five Vietnamese lines [10] (`src-tauri/src/core/aiconfig/default_system_prompt.txt`, included at `src-tauri/src/core/aiconfig/mod.rs:231 (DEFAULT_SYSTEM_PROMPT) @ 7bd1f90`):

1. "Bạn là dịch giả chuyên nghiệp, dịch sang tiếng Việt."
2. "Chỉ trả về bản dịch của đoạn được yêu cầu: không giải thích, không ghi chú, không chép lại nguyên văn."
3. "Giữ số dòng và dấu câu tương ứng với đoạn nguồn."
4. "Khi có danh sách thuật ngữ, dùng đúng bản dịch đã cho."
5. "Không dùng Markdown: không thêm tiêu đề, dấu ** hay ký hiệu định dạng."

**User message.** It is the chosen prompt-set body, expanded in one left-to-right pass that replaces `{{glossary_terms}}`, `{{tm_similar_segments}}` and `{{source_segment}}` and keeps unknown markers verbatim. An empty replacement whose marker is alone on its line removes that line (`src-tauri/src/core/ai/rag.rs:704 (assemble_prompt) @ 7bd1f90`; `:557 (expand_prompt_body)`, `:610-620`, `:637-673`). The shipped genre templates all use one frame: four lines of genre guidance, then the glossary heading and marker, then the TM heading and marker, then "Đoạn nguồn (tiếng Trung hoặc tiếng Anh):" and `{{source_segment}}` last, with nothing after it [11] (`src/promptTemplates.ts:122-133 (frame) @ 7bd1f90`). There are 29 templates (`src/promptTemplates.ts:135 (PROMPT_TEMPLATES) @ 7bd1f90`). A template is copied into a prompt set only when the user creates one from it (`src/promptSetState.ts:305 (result) @ 7bd1f90`). When no set is chosen, assembly fails with `ai_prompt.no_set_selected` and no request is built (`src-tauri/src/commands/aiprompt.rs:500-505 (assemble_and_record_prompt_with_tm) @ 7bd1f90`, `:58`). The webview's per-Work fallback picks the set whose name equals the genre label (`src/promptSetState.ts:226 (applyGenre) @ 7bd1f90`).

**Request fields.** The body has six fields: `model`, `temperature` and `max_tokens` (both left out when unset), `stream: true`, `stream_options: {include_usage: true}`, and `messages`. No reasoning, thinking or `response_format` parameter is sent [12] (`src-tauri/src/core/ai/client.rs:295 (ChatCompletionsRequestBody) @ 7bd1f90`, `:345-354`; `src-tauri/src/commands/aitranslate.rs:327-341 (resolve_call_config) @ 7bd1f90`). Headers are a Bearer key (left out when the key is empty), `Content-Type` and `Accept: text/event-stream`, and the URL is the base plus `/chat/completions` (`src-tauri/src/core/ai/client.rs:500-514 (translate) @ 7bd1f90`, `:616-618`). The read timeout is 60 s (`:70`).

**What varies per segment.** The system message and the guidance text are the same for every segment under one config. The glossary block, up to three TM pairs and the source sentence vary per segment and sit in that order, with the source sentence last (analyzer reading of `src-tauri/src/commands/aiprompt.rs:515-522 (assemble_and_record_prompt_with_tm) @ 7bd1f90` and `src-tauri/src/core/ai/rag.rs:392-421 (select_tm_pairs) @ 7bd1f90`).

**Reading the reply.** `translate` POSTs and turns a non-2xx response into `NonSuccessStatus{status, provider_message}`. It splits SSE events on blank lines (`:807 split_sse_frames`) and `interpret_sse_event` parses each one into `choices[0].delta.content`, the top-level `usage` and `choices[0].finish_reason`, reading only `choices[0]`. Each text delta goes to `on_token` unchanged and the last `usage` is kept. `[DONE]` with no non-whitespace text is `EmptyReply{finish_reason}`, and a stream that closes before `[DONE]` is `StreamEndedWithoutDone` (`src-tauri/src/core/ai/client.rs:468 (translate) @ 7bd1f90`, `:521-590`; `:770 (interpret_sse_event)`, `:777-793`). The webview joins the tokens without trimming (`src/aiTranslateState.ts:138-143 (onToken) @ 7bd1f90`) and writes nothing until the user promotes the result. `promote_ai_translation` then writes `target_text` with `translation_origin = other` through `write_non_user_target`, or returns `needs_confirmation` over an unsigned draft (`src-tauri/src/commands/segment/targets.rs:295 (promote_ai_translation) @ 7bd1f90`, `:350-356`).

Not found: handling of any provider-specific field such as a thought signature; the serde structs ignore unknown fields.

## 2. How does translating several segments or a whole chapter run?

The batch unit is one request per segment. `prepare_batch_call_with` reads the open chapter once, orders the requested ids in document order (an unknown id fails with `segment_not_in_chapter`), turns `is_omitted` rows, and already-translated rows when `skip_translated` is set, into `PreparedBatchItem::Omitted`, reads the keychain once only if something needs translating, and builds one `PreparedTranslateCall` per segment through `assemble_and_record_prompt_with_tm`, sharing a `TmRowsCache` [1] (`src-tauri/src/commands/aitranslate.rs:558 (prepare_batch_call_with) @ 7bd1f90`, `:499`, `:609`). Segments are never packed into one prompt. A selection batch passes `skip_translated` as false, so translated rows are skipped only on the whole-chapter path.

`run_batch_call` is a plain sequential `for` loop with one request in flight at a time; `should_cancel` is checked before each segment and between SSE frames; omitted rows emit `Skipped`; the first provider error stops the run and returns `(segment_id, err)`, with no retry and no skip-and-continue [2] (`src-tauri/src/commands/aitranslate.rs:1034 (run_batch_call) @ 7bd1f90`, `:1046`). A search for `concurren|join_all|buffer_unordered|Semaphore|parallel` in `core/ai`, `commands/aitranslate.rs`, `commands/proofread.rs` and `ports` at `7bd1f90` found nothing, and the webview has no concurrency limit, `Promise.all` or backoff on this path either (the header comment of `src/aiTranslateBatchState.ts` says at most one row is `running`).

Results are matched back by id: one Channel carries the whole batch and every `Token`, `Done{usage}` or `Skipped` frame carries its own `segment_id` [3] (`src-tauri/src/commands/aitranslate.rs:978 (AiTranslateBatchEventWire) @ 7bd1f90`); the webview maps frames to rows through `rowIndexBySegmentId` (`src/aiTranslateBatchState.ts:246 (runAiTranslateBatch) @ 7bd1f90`, `:262`).

Cancellation is a shared atomic generation counter: `should_cancel` is `!is_current(gen)`, single and batch share it so a new run supersedes the previous one, `ai_translate_cancel` calls `next()`, and the generation is taken before prepare (`src-tauri/src/commands/aitranslate.rs:650 (AiTranslateGeneration) @ 7bd1f90`, `:1540`, `:1430`, `:1260`). Sends run on `spawn_blocking` plus `block_on` (`:1098`, `:1147`).

Retry is user-driven only: the webview re-sends only `error` and `pending` rows and keeps `done` ones (`src/aiTranslateBatchState.ts:224 (aiTranslateBatchRetryIds) @ 7bd1f90`, `:268 (retryAiTranslateBatch)`), gated on `err.retryable` (`src/aiTranslateHandlers.ts:119 (retryAiTranslate) @ 7bd1f90`, `:145 (retryAiTranslateBatch)`). "Translate whole Chapter" sends blank, non-omitted ids with `skipTranslated = true` through the same `ai_translate_batch` command (`src/aiTranslateHandlers.ts:111 (runAiTranslateChapter) @ 7bd1f90`; `src/chapterTranslateScope.ts:7 (chapterUntranslatedIds) @ 7bd1f90`).

Checks that reject on this path: unknown id → `segment_not_in_chapter`; empty endpoint or model → NotConfigured (`src-tauri/src/commands/aitranslate.rs:310 (resolve_call_config) @ 7bd1f90`); first provider error stops the batch (`src-tauri/src/commands/aitranslate.rs:218 (batch_stopped_error) @ 7bd1f90`).

## 3. How does the Proofreader run?

Unlike translation, the Proofreader packs several segments per request. `prepare_proofread_call` keeps rows in document order minus omitted rows and rows with a blank target, sizes each as source chars plus target chars, calls `plan_batches`, and turns each batch into one `PreparedTranslateCall` with `prompt = assemble_proofread_batch_prompt(blocks)` and `system: None` [4] (`src-tauri/src/commands/proofread.rs:137 (prepare_proofread_call) @ 7bd1f90`). `plan_batches` closes a batch before `PROOFREAD_BATCH_CHAR_BUDGET` = 4,000 chars or `PROOFREAD_BATCH_MAX_SEGMENTS` = 20 segments; an oversized segment goes alone [5] (`src-tauri/src/core/ai/proofread.rs:19 (plan_batches) @ 7bd1f90`, `:13`, `:15`).

The prompt is fixed English instructions (a source-aware variant adds the kinds mistranslation, too_loose and obscure when any block has a source), then numbered `<<<SEGMENT n` blocks holding an optional `<<<SOURCE…SOURCE>>>` and a `<<<TEXT…TEXT>>>`, and asks for only a JSON array of `{segment, kind, quote, explanation, suggestion}` [6] (`src-tauri/src/core/ai/rag.rs:805 (assemble_proofread_batch_prompt) @ 7bd1f90`, `:767-792`).

`run_proofread_call` emits `Planned`, runs batches one after another with a cancel check before each, and buffers tokens into a `reply` string. An empty reply or malformed JSON loses only that batch (added to `not_scanned`, empty `Batch` event); any other provider error stops the run; if no batch parsed the result is `ReplyMalformed` (`src-tauri/src/commands/proofread.rs:258 (run_proofread_call) @ 7bd1f90`). Each reply goes through `strip_code_fence`, a serde parse, maps the 1-based `segment` index back to the batch, finds `quote` by exact substring in the target and converts to UTF-16 offsets, counting `unlocated` and `ambiguous` rather than dropping them (`src-tauri/src/core/ai/proofread.rs:109 (locate_batch_findings) @ 7bd1f90`, `:88`). Findings the user marked "not an error" are filtered from `proof_ignore` (`src-tauri/src/commands/proofread.rs:223 (read_remembered) @ 7bd1f90`; `src-tauri/src/core/ai/proofread.rs:252 (filter_remembered) @ 7bd1f90`).

## 4. Where are token counts and cost computed and shown, and where do prices come from?

Token counts come only from the provider: `into_translate_usage` turns the streamed `usage` frame (three `Option<u32>` fields) into `TranslateUsage`, and there is no local tokenizer (`src-tauri/src/core/ai/client.rs:444 (into_translate_usage) @ 7bd1f90`, `:411`). Usage is requested with `stream_options.include_usage: true` (`src-tauri/src/core/ai/client.rs:333 (build_request_body) @ 7bd1f90`).

Cost is `estimate_cost_usd(model_id, prompt_tokens, completion_tokens) -> Option<f64>` over a compile-time `PRICE_TABLE` with one row, `claude-sonnet-5` at $2 in / $10 out per million tokens, dated `PRICES_AS_OF = "2026-09-22"`; a model not in the table gets `None` [7] (`src-tauri/src/core/ai/pricing.rs:64 (estimate_cost_usd) @ 7bd1f90`, `:47`, `:30`). The price function takes no cached-token count.

Where the numbers leave Rust: single run `AiTranslateOutcomeWire::Done{usage}`; batch, per-segment `Done{usage}`, summed in the webview (`src-tauri/src/commands/aitranslate.rs:937 (AiTranslateUsageWire) @ 7bd1f90`, `:978`); Proofreader per-batch `usage` (`src-tauri/src/commands/proofread.rs:83 (AiProofreadEventWire) @ 7bd1f90`); the prompt inspector's `sent_usage`, which is `None` for batch sends (`src-tauri/src/commands/aiprompt.rs:377 (AssembledPromptRecord) @ 7bd1f90`).

Where they are shown: `src/aiUsageLine.ts:11 (aiUsageLine) @ 7bd1f90` and `:39`; batch total only when every reporting row has a cost (`src/aiTranslateBatchState.ts:139 (aiTranslateBatchUsageSummary) @ 7bd1f90`); `src/panels/AiTranslationPanel.vue:222 (aiTranslateUsageLine) @ 7bd1f90`, `:254`; `src/ProofreaderStrip.vue:166 (usageText) @ 7bd1f90`; `src/aiPromptInspectorState.ts:402 (aiPromptTokenLine) @ 7bd1f90`; switch `src/aiUsageVisibleState.ts:10 (aiUsageVisible) @ 7bd1f90`.

## 5. What code contributes text to the prompt?

Three call sites send text to a provider [13]:

- `src-tauri/src/commands/aitranslate.rs:676 (run_translate_call) @ 7bd1f90` — single segment; prompt and system come from `assemble_and_record_prompt` (`:425-441`).
- `src-tauri/src/commands/aitranslate.rs:1058 (run_batch_call) @ 7bd1f90` — batch, one request per sentence, with the same prompt and system source (`:617-638`).
- `src-tauri/src/commands/proofread.rs:288 (run_proofread_call) @ 7bd1f90` — Proofreader; prompt is `assemble_proofread_batch_prompt(&blocks)`, `system: None` (`:194-203`). It carries only each segment's own source and target, with no glossary, no TM and no neighbours (`src-tauri/src/core/ai/rag.rs:766-832 (mut) @ 7bd1f90`).

The connection check sends no prompt; it only does a GET on `<base>/models` (`src-tauri/src/core/ai/client.rs:664-690 (check_connection) @ 7bd1f90`).

For translation, only four things reach the user message: the prompt-set body, the sentence, the glossary block and the TM block (`src-tauri/src/core/ai/rag.rs:704-764 (assemble_prompt) @ 7bd1f90`; `src-tauri/src/core/promptset/vars.rs:28 (PromptVariable) @ 7bd1f90`, `:42-46`).

**Prompt sets.** Global and Work tiers are both loaded, and a Work set with the same name replaces the whole Global set (`src-tauri/src/core/promptset/store.rs:115 (resolve_two_tiers) @ 7bd1f90`).

**Glossary** [14]:

- It is skipped with no query when the body lacks `{{glossary_terms}}` (`src-tauri/src/core/ai/rag.rs:236-238 (gather_glossary_context) @ 7bd1f90`).
- Candidates are the entries of both tiers, Work over Global by source term. Only those whose source term occurs in this sentence are kept (`src-tauri/src/core/glossary/store.rs:1594-1616 (raw_matches) @ 7bd1f90`).
- Matching is a substring test at jieba token boundaries for Chinese, and stemmed contiguous tokens for English (`src-tauri/src/core/matching/mod.rs:668-740 (find_terms) @ 7bd1f90`).
- After overlap resolution, only confirmed entries are injected; pending ones are reported to the ledger and not sent (`src-tauri/src/core/glossary/store.rs:1826 (confirmed_terms_for_injection) @ 7bd1f90`, `:1842-1879`).
- No cap on the number of entries was found.
- Each entry renders as `"{source_term} → {translation}"`, de-duplicated by source term, with no header line of its own (`src-tauri/src/core/ai/rag.rs:380-388 (render_glossary_pairs) @ 7bd1f90`).

**Translation memory** [15]:

- It is skipped when the body lacks `{{tm_similar_segments}}` (`src-tauri/src/core/ai/rag.rs:305-312 (load_tm_rows) @ 7bd1f90`).
- Every `tm_unit` row of both tiers is read once and scored against the sentence by Sørensen–Dice over 1-grams and 2-grams (characters for Chinese, stemmed tokens for English). Rows whose source equals the sentence exactly are excluded (`src-tauri/src/core/tm/mod.rs:458-468 (read_tier_rows) @ 7bd1f90`, `:571-588`; `src-tauri/src/core/matching/mod.rs:390-468 (SimilarityScorer) @ 7bd1f90`).
- A row is kept at or above `tm_fuzzy_threshold`: default 65, valid range 50 to 99 (`src-tauri/src/core/scope/store.rs:124-144 (parse_tm_fuzzy_threshold) @ 7bd1f90`, `:305`).
- Selection puts the user's own pairs first, then others', each sorted by percent, de-duplicated, capped at `TM_PAIR_CAP = 3` (`src-tauri/src/core/ai/rag.rs:392-421 (select_tm_pairs) @ 7bd1f90`, `:282`).
- Each pair renders as `"{source_text} → {target_text}"`. Others' pairs are preceded once by the label "Văn phong tham khảo (bản dịch của người khác, không phải văn phong của người dùng):" (`src-tauri/src/core/ai/rag.rs:423-434 (render_tm_pairs) @ 7bd1f90`, `:284-286`).
- The Work's `tm_only_mine` switch admits only the user's own pairs (`src-tauri/src/core/tm/mod.rs:221-253 (TmSidePolicy::from_work) @ 7bd1f90`).

**Per-Work settings that reach the prompt:**

- Work-tier prompt sets
- Work-tier glossary entries
- Work-tier TM rows
- Work-tier `system_prompt`
- `work.tm_only_mine`
- the Work's `meta.source_lang`, which only picks the matcher language (`src-tauri/src/core/glossary/store.rs:1469-1475 (match_lang_for_source_lang) @ 7bd1f90`)

Temperature, max tokens, model and endpoint go into request parameters, not prompt text (`src-tauri/src/core/aiconfig/mod.rs:48-61 (AiConfigField) @ 7bd1f90`).

**Not found** [16]: the Work title, genre text, chapter title, previous or next segments, earlier translations of neighbouring segments, and any summary. None of these is an input to `assemble_prompt`, whose inputs are the body, the sentence, the glossary status and the TM context (`src-tauri/src/core/ai/rag.rs:704-709 (assemble_prompt) @ 7bd1f90`). `assemble_and_record_prompt_with_tm` reads the chapter's segments but takes only the target segment's `source_text` (`src-tauri/src/commands/aiprompt.rs:508-513 (assemble_and_record_prompt_with_tm) @ 7bd1f90`). Locator searches for `neighbo|previous segment|prev_segment|surrounding|context_segments|adjacent` in `src-tauri/` and for `neighbo|surrounding|context_segments|prev_segment` in `src/` at `7bd1f90` found nothing. The target text of the segment being translated is not included either.

## 6. How is the AI translation and Proofreader path tested?

No test calls a live provider and none replays recorded provider traffic: Rust tests use fakes at the `TranslationProvider` seam or loopback SSE servers with hand-written frames, and a search for `include_str!`/`include_bytes!` wire fixtures in `src-tauri/tests` found none [8] (`src-tauri/tests/ai_translate_contract.rs:1415 (FakeProvider) @ 7bd1f90`; `src-tauri/tests/ai_translate_wire.rs:71 (spawn_counted_sse_server) @ 7bd1f90`).

- `src-tauri/tests/ai_translate_contract.rs:1415 (FakeProvider) @ 7bd1f90` — 87 tests in the file, whose header comment says no TcpListener is used; SSE parsing; `build_request_body` JSON (`:2142-2273`, system-then-user `:2187`); `run_batch_call` with `MultiItemProvider` (`:2686`), `RecordingProvider` (`:3974`), `BatchSystemSpy` (`:4322`); pricing (`:600-679`); batch order (`:2733`).
- `src-tauri/tests/ai_translate_wire.rs:80 (spawn_counted_sse_server) @ 7bd1f90` — 34 tests; real command shells on `MockRuntime` against a loopback `TcpListener` with the real `OpenAiChatClient`; hand-written frames (`sse_body` `:1014`) and a Gemini-shaped stream (`:1038`).
- `src-tauri/tests/ai_proofread_contract.rs:134 (ScriptedProvider) @ 7bd1f90` — 22 tests (no system prompt: `SystemSpy` `:859`, `:878`); `src-tauri/src/core/ai/proofread.rs:500 (the_budget_closes_a_batch_and_an_oversized_segment_goes_alone) @ 7bd1f90` — 23 inline tests including the batch budget.
- `src-tauri/tests/ai_prompt_contract.rs:1821 (the_record_carries_the_app_default_system_prompt_when_no_tier_has_a_row) @ 7bd1f90` — 31 tests; `src-tauri/tests/ai_rag_contract.rs:1394 (perf_probe_confirmed_terms_for_injection_one_tier_vs_two_tier_short_vs_long_sentence) @ 7bd1f90` — 51 tests, with an `#[ignore]` release perf probe at that line.
- `src-tauri/tests/ai_boundary.rs:174 (AI_TRANSLATE_SEAM_COMMAND_FILE) @ 7bd1f90` — 30 static boundary scans; CI runs its ignored compile probe and fails unless exactly one test ran (`.github/workflows/ci.yml:388 (FR77 — xoá core/ai/ + ba seam đã duyệt vẫn để phần còn lại của cây biên dịch) @ 7bd1f90`).
- Webview: 42 vitest files under `tests/frontend` fake the IPC boundary with `vi.mock('../../src/config/aitranslate')` and a hand-driven fake Channel (`tests/frontend/aiTranslateBatch.test.ts:47 (runSegmentMock) @ 7bd1f90`), or mock `invoke` (`tests/frontend/aiChapterRun.test.ts:28 (invokeMock) @ 7bd1f90`).
- e2e: `e2e/specs/proofread-underline.e2e.mjs:142 (proofread underline in a real WKWebView) @ 7bd1f90` states the wire cannot be stubbed there and a real scan needs the keychain, so it calls the draw helper directly.

Neither locator digest names a test or script that scores translation quality; the only measuring script on the AI path they name is the read-only Proofreader false-alarm measure over `proof_scan` (`scripts/lib/falseAlarm.mjs:27 (listScans) @ 7bd1f90`).

## 7. Why are the request shape and batching this way?

The historian read commit messages and code comments only; story specs sit under the excluded `_bmad-output/`, so the rationale they hold is not here.

**Batching.** Packing several segments into one request was designed and measured only for the Proofreader [18]: `dc8b39d "feat(proofread): 9-3"` (2026-10-11) says "gom câu thành lô (4.000 ký tự nguồn+đích hoặc 20 câu, Ice chọn sau khi đo tiktoken: phần cố định của prompt 153 token mỗi lời gọi, Chương 184 câu khoảng 14k token đầu vào so với 40k nếu gọi từng câu)". No commit message or code comment the historian read shows translation batching being designed or measured as several segments per request (`915f1af`, `d31e95b`, `c52c88d`, `b61e87e`, `521636e`). No commit gives a reason for running requests sequentially rather than concurrently. What the code and commits do show:

- per-segment `Token`/`Done{segment_id, usage}` events
- per-sentence cancellation
- a first error that stops the batch and names the sentence
- per-segment prompt assembly with Glossary and TM injection. `b61e87e "feat(ai): 7-11 Smart RAG chèn các cặp TM có câu nguồn tương tự (cả hai tầng, ngưỡng tm_fuzzy_threshold) vào {{tm_similar_segments}}, vốn trước"` (2026-10-05) reads TM rows once per batch and holds `OpenWorkState` for the whole batch (Ice's choice 4A, "≈ 0,5 s mỗi câu ở 100.000 cặp"); `521636e` keeps that lock for the whole-chapter button.

**Commits that shaped the request:**

- `2376aeb "feat(ai): dịch một segment với kết quả chảy dần ở 4-8"` (2026-09-21) — added `TranslationProvider`, the OpenAI-compatible streaming client and the Channel, with one user message per request. The message records that the async client was needed because the blocking one has no `read_timeout`.
- `915f1af "feat(ai): dịch theo lô các câu đã chọn ở 4-9"` (2026-09-22) [17] — added batch translation as one streaming request per segment in a sequential loop. Per its message, the keychain is read once, events carry `segment_id`, cancellation happens at sentence boundaries through the 4-8 generation counter, and the run stops at the first error. Code comments in that diff say a batch row has "cùng hình dạng `prepare_translate_call` … batch không đúc một kiểu request thứ hai".
- `bb902f9 "feat(ai): số token và ước tính chi phí ở 4-11"` (2026-09-22) — added `stream_options.include_usage`. It also added the price table: "Giá từ một bảng dựng sẵn khoá theo model id (Quyết định Ice 2026-09-22), có `PRICES_AS_OF` đứng cạnh số; id không có hàng thì hiện token và không hiện tiền". The table holds only `claude-sonnet-5`, with "KHÔNG BAO GIỜ FETCH, KHÔNG BAO GIỜ ĐOÁN" under AD-15. The same message records "bảng giá mới có một hàng" as debt.
- `983ef03 "feat(ai): system prompt dùng chung, sửa được, gửi thành message system trước thân bộ prompt"` (2026-10-26) — added the system message. Its rationale: "trước đây mọi bộ prompt phải tự lặp lệnh 'chỉ trả về bản dịch' vì request chỉ có một message user". It applies to translate, batch and resend, and the Proofreader does not send it.
- `7bd1f90` (2026-10-27) — changed `interpret_sse_event` so that one frame yields text, usage and finish_reason. Its message says Gemini sends `usage` on every text frame, measured with a real key.

**Later refactors:**

- `d31e95b "refactor(ai-translate): 9-12 rút đường gọi AI chung cho dịch đơn và dịch lô"` (2026-10-10) — gave single and batch one shared call path, and kept "temperature/max_tokens chưa đặt vẫn gửi vắng mặt".
- `c52c88d` (2026-10-12) — takes the generation before prepare.
- `521636e` (2026-10-26) — the whole-chapter button reuses `ai_translate_batch` with `skip_translated`.

Decisions recorded in these messages:

- Ice, 2026-09-22: a bundled, dated price table, where absence means a local model (`bb902f9`).
- Ice: the Proofreader limits of 4,000 chars or 20 segments, chosen after measuring with tiktoken (`dc8b39d`).
- Ice: keep `OpenWorkState` held for the whole translate batch (`b61e87e`, `521636e`).
- Two failure policies: translation stops at the first error (`915f1af`), while the Proofreader skips a failed or empty batch and keeps scanning (`dc8b39d`, `7bd1f90`).

## Where the answers meet

- `assemble_and_record_prompt_with_tm` — questions 1, 2 and 5. The single call reaches it from `prepare_translate_call`, the batch reaches it once per segment from `prepare_batch_call_with` with a shared `TmRowsCache`, and it is where glossary, TM and system text enter (`src-tauri/src/commands/aiprompt.rs:485 (assemble_and_record_prompt_with_tm) @ 7bd1f90`). It rejects a missing or unknown prompt set with `ai_prompt.no_set_selected` (`src-tauri/src/commands/aiprompt.rs:500-505 (assemble_and_record_prompt_with_tm) @ 7bd1f90`).
- `PreparedTranslateCall` → `TranslationProvider::translate` → `build_request_body` — questions 1, 2, 3 and 4. All three call sites build a `TranslateRequest` and send it through the same client, so the same six-field body, `include_usage` and usage parsing apply to translation and the Proofreader alike (`src-tauri/src/core/ai/client.rs:333 (build_request_body) @ 7bd1f90`; `src-tauri/src/core/ai/client.rs:468 (translate) @ 7bd1f90`). The client rejects a non-2xx response (`NonSuccessStatus`), an empty reply (`EmptyReply`) and a stream with no `[DONE]` (`StreamEndedWithoutDone`). Batch translation stops on any of these, while the Proofreader skips the batch on `EmptyReply` or malformed JSON (`src-tauri/src/commands/proofread.rs:258 (run_proofread_call) @ 7bd1f90`).
- `resolve_call_config` — questions 1, 2 and 3. Every path shares it, and an empty endpoint or model means NotConfigured (`src-tauri/src/commands/aitranslate.rs:310 (resolve_call_config) @ 7bd1f90`).

## Open questions

- Why translation batching is one request per segment and runs sequentially is not stated in any commit the historian read. The next ask: what do the story 4.9 and 9.3 specs say about the batch unit and concurrency? They sit under `_bmad-output/`, which this run left out.
- The bench numbers in `330b65d` were truncated in the historian's read. The next ask: what lock-hold and per-sentence timings does `330b65d` record for a translate batch?

## Prior documents

Research documents in reach, listed by title and not opened: `research-go-cham-luoi-subgrid-webkit`, `research-grid-render-on-caret-move`, `research-grid-text-input-path`, `research-phase-0-spike-results`, `research-technical-auratranslate-tauri-rust-local-first`. No `adr-*.md` exists in the repository; architectural decisions live as ADs in the spine. This run follows up none of them.
