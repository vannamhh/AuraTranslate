# Q5 digest: what contributes text to the AI provider prompt (commit 7bd1f90)

Read via `git show 7bd1f90:<path>`; code not run. Files read: 11 of 15.

## Final wire shape (every provider call)
- [LOAD-BEARING] Chat body has at most two messages: optional `system` (skipped if None or blank) and one `user` message whose content is `request.prompt` verbatim. src-tauri/src/core/ai/client.rs:333-355 (build_request_body) @ 7bd1f90. Also sends `stream:true`, `stream_options.include_usage`, `temperature`, `max_tokens` (:345-353). No other message (no history, no assistant turns).
- Connection check sends no prompt: GET `<base>/models` only. client.rs:664-690 (check_connection) @ 7bd1f90.

## Call sites that put text into system/user message (3 `.translate(` sites)
1. Single sentence: commands/aitranslate.rs:425-441 (prepare) calls `assemble_and_record_prompt` -> `PreparedTranslateCall{prompt: assembled.prompt, system: assembled.system}`; sent at aitranslate.rs:676-684 (run_translate_call) @ 7bd1f90.
2. Batch translate: aitranslate.rs:617-638 (prepare batch; one `TmRowsCache` shared, :609) -> sent at aitranslate.rs:1058-1066 (run_batch_call), one request per sentence, same prompt/system source.
3. Proofread: commands/proofread.rs:194-203 builds `prompt: assemble_proofread_batch_prompt(&blocks)`, `system: None`; sent at proofread.rs:288-296 (run_proofread_call).
   - Proofread prompt text (core/ai/rag.rs:766-832, assemble_proofread_batch_prompt): fixed English instruction constants (:767-792) + per segment `<<<SEGMENT n`, optional `<<<SOURCE\n{source_text}\nSOURCE>>>`, `<<<TEXT\n{target_text}\nTEXT>>>`, `SEGMENT>>>`. Only the segments' own source/target; no glossary, no TM, no neighbours. Source block omitted when source blank (:820).
   - PROOFREAD_CONTEXT_CHARS (core/ai/proofread.rs:212-219) builds the ignore-memory signature, not prompt text.

## Translate prompt assembly (the only one that injects context)
Entry: commands/aiprompt.rs:485-556 (assemble_and_record_prompt_with_tm) @ 7bd1f90.
- Inputs it reads: prompt sets of both tiers via `promptset::resolve_two_tiers` (aiprompt.rs:498; promptset/store.rs:115-126: load Global + Work tier, `apply_override` by name, a Work set with the same name replaces the whole Global set); chosen set body by `prompt_set_name` (:500-505, errors `ai_prompt.no_set_selected` when empty/unknown); open Work chapter's segments (:508) -> only the target segment's `source_text` (:509-512) and Work `meta.source_lang` (:513).
- Glossary: `gather_glossary_context(&set.body, ..., source_lang, sentence)` (:515); TM: `load_tm_rows` once per cache (:523-526) then `gather_tm_context` (:527); `assemble_prompt(&set.body, sentence, glossary, tm)` (:529).
- System: `resolve_system_prompt(aiconfig::resolve_two_tiers(...))` (:530-532).

### Prompt body expansion (the user-authored set body is the template)
- Only three variables exist: `{{glossary_terms}}`, `{{source_segment}}`, `{{tm_similar_segments}}`. promptset/vars.rs:28-37 (PromptVariable), :42-46 (ALL), :59-65 (marker).
- [LOAD-BEARING] `expand_prompt_body` single left-to-right scan of the body, replacing those markers; unknown `{{x}}` kept verbatim; injected content never re-scanned. core/ai/rag.rs:557-696 (expand_prompt_body), replacement map :438-449 (replacement_for). An empty replacement on a marker alone on its line removes the whole line (:637-640). A body without `{{source_segment}}` still assembles (flag `source_segment_missing`, :753).
- So nothing reaches the user message except: the set body text, the sentence, the glossary block, the TM block. (Nothing else is concatenated anywhere in assemble_prompt, rag.rs:704-764.)

### Glossary
- Skipped entirely (no query) if body lacks `{{glossary_terms}}`. rag.rs:236-238 (gather_glossary_context).
- Lang: `match_lang_for_source_lang`: source_lang == LANG_CHINESE -> Zh else En. glossary/store.rs:1469-1475.
- [LOAD-BEARING] Candidate set = entries of BOTH tiers (Global + Work), Work overrides Global by source_term, loaded in full (`load_tier`), unfiltered by confirmation. glossary/store.rs:1594-1598 (resolve_and_match); then only entries whose source_term OCCURS in this sentence are kept: `find_terms(text, terms, lang)` (:1616).
- Matching (core/matching/mod.rs:668-740, find_terms): Zh = substring `find` accepted only when both ends are jieba token boundaries (:675-697); En = tokenise + stem both sides (normalize), multi-word term = contiguous token run not crossing a sentence boundary (:705-737). Empty/separator-only terms never match.
- Overlaps resolved by `resolve_overlaps` (glossary/store.rs:1839); then only `is_confirmed()` entries (have translation) are injected (:1842-1862); pending entries are not sent (reported in ledger only :1864-1879). Order = order of `selected` (span order).
- No cap on the number of glossary pairs found in rag.rs/glossary store (searched `take(`/cap constants in rag.rs: only TM_PAIR_CAP exists).
- Rendered format: one line per distinct source_term, `"{source_term} → {translation}"` joined with `\n`; duplicate source_term (several matches in one sentence) collapsed. rag.rs:380-388 (render_glossary_pairs). No header/label line.

### Translation memory
- Skipped (no store read, even the Work switch) if body lacks `{{tm_similar_segments}}`. rag.rs:305-312 (load_tm_rows).
- Side policy: Work's `tm_only_mine` flag (SELECT from `work` row) -> if set only own pairs admitted. tm/mod.rs:221-229 (from_work), :240-242 (admits), :246-253 (read_only_mine). No Work open -> keeps both sides.
- Threshold: global app config `tm_fuzzy_threshold`; default 65, valid 50..=99, else default. scope/store.rs:124-144 (consts, parse_tm_fuzzy_threshold), :305 (tm_fuzzy_threshold); read at rag.rs:314-319.
- [LOAD-BEARING] Candidates: ALL tm_unit rows of Global and Work read once (tm/mod.rs:458-468 read_tier_rows), each scored against the sentence by SimilarityScorer = Sorensen-Dice over multiset of 1-grams + 2-grams (chars for Zh, stemmed tokens for En), integer percent rounded down, capped at 99 (core/matching/mod.rs:390-468; tm/mod.rs:576). Rows whose source equals the sentence exactly are excluded (tm/mod.rs:571); whitespace-only difference scores 99 (:442, :573-574). Keep `percent >= threshold` (:578). Tiers merged by `merge_tagged_tiers` (AD-18 order) then policy applied (:584-588). Empty/blank sentence -> empty context (rag.rs:334-339).
- Cap/selection at assembly (rag.rs:392-421, select_tm_pairs): drop pairs with source_text == sentence; "Mine" side first, then "Others", each sorted by percent desc (stable); dedupe on (source,target); `TM_PAIR_CAP = 3` (rag.rs:282). Counts `omitted_count` for the ledger.
- Rendered (rag.rs:423-434, render_tm_pairs): lines `"{source_text} → {target_text}"`, one per pair; before the first pair that is from "Others" (reference) one label line is inserted: "Văn phong tham khảo (bản dịch của người khác, không phải văn phong của người dùng):" (rag.rs:284-286). Percent, tier are NOT in the text.

### System prompt
- core/aiconfig/mod.rs:229-243 (DEFAULT_SYSTEM_PROMPT, resolve_system_prompt): winning tier's `system_prompt` row (Work over Global via aiconfig::resolve_two_tiers, store.rs:58) else the shipped default `core/aiconfig/default_system_prompt.txt` (5 Vietnamese lines: professional translator into Vietnamese; return only translation; keep line count/punctuation; use given glossary translations; no Markdown). Blank row -> no system message; non-blank text not trimmed.
- Other aiconfig fields (provider, endpoint, model, temperature, max_tokens: aiconfig/mod.rs:48-61) go to request parameters, not prompt text.

## Per-Work settings that reach the prompt
- Work-tier prompt sets (override by name), Work-tier glossary entries, Work-tier TM rows, Work-tier `system_prompt`, `work.tm_only_mine`, Work `meta.source_lang` (picks matcher language only). All cited above.

## Looked for, not found (git grep / reads at 7bd1f90, excluding _bmad-output etc.)
- Work title, genre, chapter title, previous/next segments, earlier translations of neighbouring segments, summaries: none appear in assemble_prompt/expand_prompt_body inputs (inputs are body, sentence, GlossaryInjectionStatus, TmContext only; rag.rs:704-709). In aiprompt.rs:508-513 only the matched segment's `source_text` and Work source_lang are taken from the chapter; the other chapter segments are read but unused for prompt text. Not searched exhaustively for a `chapter_context` variable beyond noting that unknown markers such as `{{chapter_context}}` are kept verbatim (rag.rs:545, comment) and PromptVariable::ALL has only three entries (vars.rs:42-46).
- Target-side text of the segment being translated is not included in the translate prompt.
- Frontend (src/) was not searched; only src-tauri.
- Not read in full: tm/mod.rs merge_tagged_tiers/police (AD-18 ordering internals), glossary resolve_overlaps, matching normalize/tokenize, promptset/store.rs load_prompt_set_tier, aiconfig/store.rs.
