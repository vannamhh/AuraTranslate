# Locator digest, area `src-tauri/` @ 7bd1f90

(Returned inline by a read-only locator; filed by the lead.) Counts: Q1 8, Q2 6, Q3 4, Q4 6, Q5 11, Q6 16, Q7 12 commits. All via `git grep`/`git show` at 7bd1f90.

## Q1
- `src-tauri/src/core/ai/client.rs:333 (build_request_body)` — optional `system` message first (skipped if None/whitespace), then one `user` message with `request.prompt` verbatim; `model`, `stream: true`, `stream_options.include_usage: true`; temperature/max_tokens omitted when unset (`:295 ChatCompletionsRequestBody`).
- `src-tauri/src/core/ai/client.rs:468 (translate)` — POST to `chat_completions_url` (`:616`); Bearer header sensitive, omitted when key empty; non-2xx → `NonSuccessStatus` with provider message (`:269 read_provider_message`, redacted); SSE via `split_sse_frames` `:807`, `interpret_sse_event` `:770`; each text delta → `on_token`; last `usage` kept; `[DONE]` with no non-blank text → `EmptyReply{finish_reason}`; close without `[DONE]` → `StreamEndedWithoutDone`; `READ_TIMEOUT` 60s (`:70`); SSE buffer cap `:80`, `:731`.
- `src-tauri/src/ports/translation_provider.rs:73 (TranslateRequest)` — endpoint, model, temperature, max_tokens, api_key, prompt ("verbatim, not reassembled"), system; `:120 TranslateOutcome`; `:140 TranslationProvider` (doc: no retry).
- `src-tauri/src/commands/aitranslate.rs:388 (prepare_translate_call)` — order: config (`:310 resolve_call_config`, empty endpoint/model ⇒ NotConfigured), `is_omitted` guard, keychain (`:721 read_api_key`), `assemble_and_record_prompt` ⇒ `PreparedTranslateCall` (`:269`); resend reuses recorded prompt+system byte for byte (`:451 prepare_resend_call`).
- `src-tauri/src/commands/aiprompt.rs:485 (assemble_and_record_prompt_with_tm)` — resolves prompt set, reads segment `source_text`, gathers Glossary + TM, calls `assemble_prompt`; system from `resolve_system_prompt` (`:530`); overwrites single session record `LastAssembledPromptState` (`:402`; struct `:377`).
- `src-tauri/src/core/ai/rag.rs:704 (assemble_prompt)` — one left-to-right pass over prompt-set body (`:557 expand_prompt_body`), replaces `{{source_segment}}`, `{{glossary_terms}}`, `{{tm_similar_segments}}`, unknown markers kept literally; order = order in user's body; labelled `PromptPiece`s (`:167`, `:186`).
- `src-tauri/src/core/aiconfig/mod.rs:235 (resolve_system_prompt)` — winning tier's `system_prompt`, else `DEFAULT_SYSTEM_PROMPT` (`:231`, from `src-tauri/src/core/aiconfig/default_system_prompt.txt`, Vietnamese translator instruction banning Markdown); blank row ⇒ no system message.
- Read-back: tokens to webview over `Channel<String>` (`src-tauri/src/commands/aitranslate.rs:670 run_translate_call`), not written to DB; after Done `mark_prompt_as_sent` (`src-tauri/src/commands/aiprompt.rs:611`) stamps `sent_at`/`sent_model`/`sent_usage` only when segment_id, prompt, system still match; DB write on promote: `src-tauri/src/commands/segment/targets.rs:295 (promote_ai_translation)`, one UPDATE via `write_non_user_target` (`:253`) sets `target_text` and `translation_origin = other`; IPC shell `src-tauri/src/commands/segment/wire.rs:363`; error mapping `src-tauri/src/commands/aitranslate.rs:119 (openai_client_error_family)`.

## Q2
- `src-tauri/src/commands/aitranslate.rs:558 (prepare_batch_call_with)` — reads open chapter once, orders ids in document order (unknown ⇒ `segment_not_in_chapter`); `is_omitted` and (with `skip_translated`) translated rows ⇒ `PreparedBatchItem::Omitted` (`:499`); keychain read once if needed; one `PreparedTranslateCall` per segment via `assemble_and_record_prompt_with_tm`, shared `TmRowsCache` (`:609`). Unit = one request per segment, never packed. `prepare_batch_call` `:546`.
- `src-tauri/src/commands/aitranslate.rs:1034 (run_batch_call)` — sequential `for`, one request in flight; no join_all/Semaphore/buffer_unordered in AI files; `should_cancel` before each segment (`:1046`) and between SSE frames; omitted ⇒ `Skipped`; first provider error stops, returns `(segment_id, err)`; no retry, no skip-and-continue.
- `src-tauri/src/commands/aitranslate.rs:978 (AiTranslateBatchEventWire)` — one Channel per batch; Token/Done{usage}/Skipped each with `segment_id`.
- `src-tauri/src/commands/aitranslate.rs:650 (AiTranslateGeneration)` — atomic generation counter, `should_cancel` = `!is_current(gen)`, shared by single and batch; `ai_translate_cancel` (`:1540`) calls `next()`; generation taken before prepare (`:1430 ai_translate_batch`, `:1260 ai_translate_segment`); `send_prepared_translate_call` (`:1098`, check `:1107`), `send_prepared_batch_call` (`:1147`) on `spawn_blocking` + `block_on`; `batch_stopped_error` (`:218`); batch stamps only last sent item via `last_sent`.
- `src-tauri/src/lib.rs:1404 (run)` registration `:1404-1416`, states `:1712`, `:1719`; `src-tauri/src/lib.rs:980 (nfr_bench)` probe (module `:279`, feature-gated); `src-tauri/src/commands/segment/chapter_read.rs:377 (read_open_chapter_segments)`.

## Q3
- `src-tauri/src/commands/proofread.rs:137 (prepare_proofread_call)` — same config/key path; document order minus omitted and blank-target rows; size = source chars + target chars; `plan_batches`; each batch one `PreparedTranslateCall` with `prompt = assemble_proofread_batch_prompt(blocks)`, `system: None`; loads `proof_ignore` (`:223 read_remembered`).
- `src-tauri/src/core/ai/proofread.rs:19 (plan_batches)` — `PROOFREAD_BATCH_CHAR_BUDGET` 4,000 (`:13`), `PROOFREAD_BATCH_MAX_SEGMENTS` 20 (`:15`); oversized segment alone.
- `src-tauri/src/core/ai/rag.rs:805 (assemble_proofread_batch_prompt)` — fixed English instructions (`:767-780`); source-aware variant adds mistranslation/too_loose/obscure; numbered `<<<SEGMENT n` blocks with optional `<<<SOURCE…SOURCE>>>` and `<<<TEXT…TEXT>>>` (`:782-792`); asks for only a JSON array {segment, kind, quote, explanation, suggestion}.
- `src-tauri/src/commands/proofread.rs:258 (run_proofread_call)` — `Planned`, batches sequential, cancel before each; tokens buffered into `reply`; empty reply (`is_empty_reply`) or malformed JSON loses only that batch (→ `not_scanned`, empty `Batch`); other provider error stops (`ProofreadRunError::Provider`); `filter_remembered` then `AiProofreadEventWire::Batch{usage, segments}` (`:83`); none parsed ⇒ `ReplyMalformed`; `ProofreadGeneration` (`:30`), `send_prepared_proofread_call` (`:604`), `ai_proofread_segments` (`:652`), `ai_proofread_cancel` (`:780`).
- `src-tauri/src/core/ai/proofread.rs:109 (locate_batch_findings)` — `strip_code_fence` (`:88`), serde; 1-based `segment` → batch position (`RawFinding::batch_position` ~`:74`); exact-substring `quote` → UTF-16; counts `unlocated`/`ambiguous`; `signature_of` `:194`, `filter_remembered` `:252`.

## Q4
- `src-tauri/src/core/ai/pricing.rs:64 (estimate_cost_usd)` — `Option<f64>`; compile-time `PRICE_TABLE` (`:47`) ONE row: `claude-sonnet-5` $2 in / $10 out per M; `PRICES_AS_OF = "2026-09-22"` (`:30`), hand-sourced from claude.com/pricing, never fetched; unknown model ⇒ None.
- `src-tauri/src/core/ai/client.rs:444 (into_translate_usage)` — `ChunkUsage` `:411` three `Option<u32>`; cost only when both counts present; no local tokenizer.
- `src-tauri/src/ports/translation_provider.rs:108 (TranslateUsage)`; `src-tauri/src/commands/aitranslate.rs:937 (AiTranslateUsageWire)`; single `Done{usage}` ~`:1195`, `single_run_outcome_wire` `:1222`; batch per-segment `Done{usage}` (`:978`), total summed in webview.
- `src-tauri/src/commands/aiprompt.rs:377 (AssembledPromptRecord)` `sent_usage` (None for batch sends); `src-tauri/src/commands/proofread.rs:83` per-batch usage.

## Q5
- Prompt sets: `src-tauri/src/core/promptset/store.rs:115 (resolve_two_tiers)`; `src-tauri/src/core/promptset/vars.rs:28 (PromptVariable)` exactly three (`:51-63`), `from_token` `:72`, `scan_markers` `:115`; `core/promptset/exchange.rs`, `commands/promptset.rs`.
- Source: `src-tauri/src/commands/aiprompt.rs:485` uses `row.source_text`, Work `meta.source_lang`.
- Glossary: `src-tauri/src/core/ai/rag.rs:228 (gather_glossary_context)` skipped when marker absent; `src-tauri/src/core/glossary/store.rs:1826 (confirmed_terms_for_injection)` only gate, confirmed terms after overlap arbitration; `:1469 (match_lang_for_source_lang)`; `src-tauri/src/core/ai/rag.rs:380 (render_glossary_pairs)` one "term → translation" per line, dedup by source term.
- TM: `src-tauri/src/core/ai/rag.rs:305 (load_tm_rows)`, `:325 (gather_tm_context)`, `:392 (select_tm_pairs)` max `TM_PAIR_CAP` 3 (`:282`), own first, identical sources excluded; `:423 (render_tm_pairs)`, `TM_REFERENCE_LABEL` (`:284`); `src-tauri/src/core/tm/mod.rs:458 (read_tier_rows)`, `:560 (fuzzy_pairs_in_candidates)`.
- Settings: `src-tauri/src/core/tm/mod.rs:223 (TmSidePolicy::from_work)`; `src-tauri/src/core/scope/store.rs:305 (tm_fuzzy_threshold)`; `src-tauri/src/core/aiconfig/store.rs:58 (resolve_two_tiers)` Work over Global for system_prompt, model, temperature, max_tokens (`src-tauri/src/core/aiconfig/mod.rs:48 AiConfigField`).
- System: `src-tauri/src/core/aiconfig/mod.rs:235`, `default_system_prompt.txt`.
- Proofreader: `src-tauri/src/core/ai/rag.rs:767-805`; stored target_text + source_text, no system (`src-tauri/src/commands/proofread.rs:137`).
- Neighbouring segments: none.

## Q6
- Fakes at `TranslationProvider` seam + loopback SSE with hand-written frames; no recorded wire data, no live providers, keychain mocked.
- `src-tauri/tests/ai_translate_contract.rs:1415 (FakeProvider)` 87 tests; header `:26` no TcpListener (AD-45); SSE parse cases; `build_request_body` JSON `:2142-2273` (system-then-user `:2187`); `prepare_*`; `run_translate_call`/`run_batch_call` with `MultiItemProvider` `:2686`, `RecordingProvider` `:3974`, `BatchSystemSpy` `:4322`; pricing `:600-679`; batch order `:2733`.
- `src-tauri/tests/ai_translate_wire.rs:71 (spawn_counted_sse_server)` 34 tests; real command shells on `MockRuntime` vs loopback `TcpListener` (`:80`), real `OpenAiChatClient`; `USAGE_CHUNK`, `sse_body` `:1014`, Gemini-shaped stream `:1038`; status/redaction.
- `src-tauri/tests/ai_proofread_contract.rs:134 (ScriptedProvider)` 22 tests (`SystemSpy` `:859`, `:878`).
- `src-tauri/tests/ai_prompt_contract.rs:1821` 31 tests (system tiers `:1821-1857`); `src-tauri/tests/ai_rag_contract.rs` 51 tests (`#[ignore]` perf probe `:1394`); `src-tauri/tests/ai_boundary.rs:174` 30 tests (allowed names `:550-554`, ignored cargo-check probe `:2117`); `src-tauri/tests/connection_check_contract.rs:19` 18 (loopback `:284`); `src-tauri/tests/aiconfig_contract.rs:1411` 34; `aiconfig_keychain_boundary.rs` 9, `proofread_memory_contract.rs` 13, `proofread_decision_contract.rs` 9, `proofread_write_boundary.rs` 17; `story_e7_r4_population.rs` (ignored bench builder); `nfr_bench_wiring_contract.rs`.
- Inline: `src-tauri/src/core/ai/proofread.rs:500` 23 tests (budget `:500`, `:511`); `src-tauri/src/core/ai/rag.rs:594 (proofread_prompt_tests)` 4.
- `src-tauri/AGENTS.md:23-24` no listening port in release (AD-45), AD-22 Channel/cancel rule.

## Q7 (2026)
- `6de227d` 09-18 (4-6) pure Smart RAG Injector, `confirmed_terms_for_injection` gate.
- `2376aeb` 09-21 (4-8) `TranslationProvider`, OpenAI-compatible streaming client, Channel, single user message; async reqwest for `read_timeout`.
- `915f1af` 09-22 (4-9) batch: keychain once, `segment_id` events, cancel at segment boundaries, stop at first error.
- `d1451a8` 09-22 (4-10); `bb902f9` 09-22 (4-11) `include_usage`, price table `PRICES_AS_OF`.
- `b61e87e` 10-05 (7-11) TM pairs cap 3; `d31e95b` 10-10 (9-12) shared path; `dc8b39d` 10-11 (9-3) Proofreader batch sizes after tiktoken (153 fixed tokens, ~14k vs ~40k for 184 segments); `c52c88d` 10-12 (9-11) generation before prepare.
- `983ef03` 10-26 system message before body (before: user only; Proofreader none); `521636e` 10-26 `skip_translated`; `4888d31` 10-27 (4-13); `7bd1f90` 10-27 (4-14) Gemini usage every frame, empty reply error.

## Searched, nothing found @ 7bd1f90 (src-tauri/)
- `neighbo|previous segment|prev_segment|surrounding|context_segments|adjacent` in core/ai, commands/aiprompt.rs, commands/aitranslate.rs, core/promptset.
- `concurren|join_all|buffer_unordered|Semaphore|parallel` in core/ai, commands/aitranslate.rs, commands/proofread.rs, ports.
- Automatic retry loop: none (only `retryable` flags).
- `include_str!`/`include_bytes!` wire fixtures in src-tauri/tests: none. Live-provider tests: none.
