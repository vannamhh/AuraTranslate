# Q1 digest: single-segment AI translate request assembly and response reading (commit 7bd1f90)

## Call chain
- Webview `runAiTranslate(promptSetName, segmentId)` -> `runAiTranslateSegment(...)`: src/aiTranslateState.ts:103-108 (runAiTranslate) @ 7bd1f90. The webview assembles no prompt; Rust does.
- `prepare_translate_call`: src-tauri/src/commands/aitranslate.rs:388-442 @ 7bd1f90. Order: resolve config (310) -> reject omitted segment (417) -> read API key (421) -> `assemble_and_record_prompt` (425) -> `PreparedTranslateCall{prompt, system, ...}` (433-441).
- `run_translate_call` builds `TranslateRequest` from the prepared call and calls `provider.translate`: aitranslate.rs:670-693 @ 7bd1f90.
- `assemble_and_record_prompt_with_tm`: src-tauri/src/commands/aiprompt.rs:485-556 @ 7bd1f90. It resolves the prompt set by NAME (500-505), takes the source sentence from the segment row `source_text` (512), gathers glossary (515) and TM (523-528), then `assemble_prompt(&set.body, sentence, glossary, tm)` (529) gives the user prompt. `system` comes from `resolve_system_prompt` (530-532). Both are stored in `AssembledPromptRecord` (534-553).
- `TranslateRequest` fields: endpoint, model, temperature: Option, max_tokens: Option, api_key, prompt, system: Option. src-tauri/src/ports/translation_provider.rs:73-91 @ 7bd1f90. Load-bearing: the doc says `prompt` goes through verbatim, with no second assembly (58-61).

## Message order (load-bearing)
`build_request_body`: src-tauri/src/core/ai/client.rs:333-355 @ 7bd1f90.
1. If `request.system` is Some and non-blank: `{role:"system", content: system}` (335-340).
2. Always: `{role:"user", content: request.prompt}` (341-344).
So the order is system, then user. There is no assistant, no few-shot, and no further message. The system message is omitted when the text is blank. `resolve_system_prompt` returns the winning tier's row, else `DEFAULT_SYSTEM_PROMPT`, and returns None if the text is blank (non-blank text is never trimmed): src-tauri/src/core/aiconfig/mod.rs:229-243 @ 7bd1f90.

## Default system prompt (load-bearing)
src-tauri/src/core/aiconfig/default_system_prompt.txt:1-5 @ 7bd1f90, included by `include_str!` at aiconfig/mod.rs:231. Verbatim, in Vietnamese:
1. "Bạn là dịch giả chuyên nghiệp, dịch sang tiếng Việt."  (You are a professional translator, translating into Vietnamese.)
2. "Chỉ trả về bản dịch của đoạn được yêu cầu: không giải thích, không ghi chú, không chép lại nguyên văn."  (Return only the translation of the requested passage: no explanation, no notes, no copying the original.)
3. "Giữ số dòng và dấu câu tương ứng với đoạn nguồn."  (Keep line count and punctuation matching the source passage.)
4. "Khi có danh sách thuật ngữ, dùng đúng bản dịch đã cho."  (When a glossary list is present, use the given translations exactly.)
5. "Không dùng Markdown: không thêm tiêu đề, dấu ** hay ký hiệu định dạng."  (No Markdown: no headings, ** or formatting symbols.)
No trailing newline. It is sent only when no tier (global or work) holds a `system_prompt` row. If a row exists and is blank, no system message is sent.

## User message: prompt-set body expansion
`assemble_prompt` (src-tauri/src/core/ai/rag.rs:704-764 @ 7bd1f90) calls `expand_prompt_body` (557-696): one left-to-right scan over the ORIGINAL body, so injected content is never rescanned.
- Three markers: `{{glossary_terms}}`, `{{tm_similar_segments}}`, `{{source_segment}}` (`PromptVariable`, replacement_for rag.rs:438-449).
- Unknown `{{x}}` markers stay verbatim (610-620).
- An empty replacement whose marker sits alone on its line removes the whole line, and collapses two adjacent blank lines (637-673).
- A multi-line replacement is isolated on its own line when the marker shares the line (683-690).
- The glossary block is "source -> translation" lines (the arrow is a real U+2192), deduplicated by source_term: rag.rs:380-388 render_glossary_pairs.
- The TM block is "source -> target" lines, at most `TM_PAIR_CAP = 3` (rag.rs:282). The user's own pairs come first, then others' pairs, sorted by percent. Others' pairs are preceded once by the label "Văn phong tham khảo (bản dịch của người khác, không phải văn phong của người dùng):" (rag.rs:284-286, 390-434).
- The source segment is the raw `source_text` of the segment row (aiprompt.rs:512).

## Genre template body as shipped (load-bearing)
src/promptTemplates.ts:3-120 (GUIDANCE per GenreId), :122-133 (frame), :135-137 (PROMPT_TEMPLATES) @ 7bd1f90.
`frame(guidance)` yields exactly:
```
<guidance, 4 lines of Vietnamese style/terminology advice>

Thuật ngữ bắt buộc (trống thì bỏ qua):
{{glossary_terms}}

Câu tương tự đã dịch, tham khảo văn phong (trống thì bỏ qua):
{{tm_similar_segments}}

Đoạn nguồn (tiếng Trung hoặc tiếng Anh):
{{source_segment}}
```
So the guidance sits first, then the glossary marker, then the TM marker, and the source marker is last, at the very end of the body (no text after it). Every template has the same frame; only the guidance differs. There is one template per genre (news, commentary, tcm, health, cuisine, family, travel, culture, history_figures, spiritual, arts, business, science, legal, nonfiction, xianxia, xuanhuan, wuxia, urban, romance, historical, scifi, game, horror, mystery, fanfic, literary, children, other). Example, `news` (promptTemplates.ts:4-7): "Đây là tin tức thời sự. Văn phong báo chí, câu ngắn, trung tính, không thêm bình luận của người dịch." followed by 3 more lines on names, titles and numbers.
Templates are not applied automatically at translate time. `PROMPT_TEMPLATES[id]` is only copied into a new prompt set by the user (`promptSetCreate(tier, name, PROMPT_TEMPLATES[id])`, src/promptSetState.ts:305; preview in src/PromptTemplatePicker.vue:26).

## Is a prompt set used when none is chosen?
No. With `prompt_set_name` None, blank, or not matching a resolved set, `assemble_and_record_prompt_with_tm` returns `ai_prompt.no_set_selected` (aiprompt.rs:500-505; `no_set_selected` defined aiprompt.rs:58). No fallback body exists in the Rust path I read, so no translate request is built.
Webview selection (src/promptSetState.ts:19-26, 175-236): a hand-picked name is remembered per Work (`work.chosen_prompt_set_name`); otherwise `applyGenre` auto-selects the set whose name equals the genre label (case-insensitive, 226-235); otherwise the selection is null. So the system prompt has a built-in default, the user prompt body has none.

## Request body fields (load-bearing)
`ChatCompletionsRequestBody`: client.rs:294-312; construction client.rs:345-354 @ 7bd1f90.
- `model`: from config.
- `temperature`: Option<f64>; `skip_serializing_if is_none`, so it is absent unless set. Config value "provider default" or unparsable -> None (aitranslate.rs:327-332).
- `max_tokens`: Option<u32>; omitted when None; 0 or unparsable -> None (aitranslate.rs:333-341).
- `stream: true` (always).
- `stream_options: {"include_usage": true}` (always, client.rs:310, 318-321, 350-352).
- `messages`: as above.
Nothing else is sent. A `git grep -i 'reasoning|thinking|response_format|reasoning_effort'` over src-tauri/src and src (excluding _bmad-output) returned no match outside i18n (the grep filtered i18n lines, so an i18n-only hit would not show). The struct has exactly these six fields, so NO reasoning/thinking control is sent. Headers: Authorization Bearer (sensitive; omitted when the key is empty, client.rs:500-509), Content-Type application/json, Accept text/event-stream (511-514). URL is `chat_completions_url(endpoint)` = base + `/chat/completions` (616-618). Read timeout is 60 s (client.rs:70).

## Prefix stability across segments
The system message is identical across segments for the same config. In the user message the shared part is the guidance text plus the fixed headings up to the first marker. Varying per segment, in order: (1) glossary block, in the "Thuật ngữ bắt buộc" slot, entries matched to this sentence (gather_glossary_context takes `sentence`, aiprompt.rs:515-522); (2) TM block, up to 3 pairs, in the second slot (selected per sentence, rag.rs:392-421); (3) the source sentence, at the very end. A stable prefix therefore runs through the guidance text and the heading "Thuật ngữ bắt buộc (trống thì bỏ qua):", and ends where the glossary content begins. If a segment has no glossary hits, that line is removed with the marker (rag.rs:637-673), and the prefix then continues into the TM heading, and so on. The source sentence is always last, so the suffix after it is empty. Only the system message plus the guidance text are guaranteed identical. This describes the text order and does not claim anything about provider-side caching.

## Reading the response back (load-bearing)
`OpenAiChatClient::translate`: client.rs:468-596 @ 7bd1f90.
- Non-2xx: reads the body for a provider message and returns `NonSuccessStatus{status, provider_message}` (521-528).
- Stream: loop reads bytes, `split_sse_frames` splits events on "\n\n" or "\r\n\r\n" (807-820), joins `data:` lines (845-858), and `interpret_sse_event` parses each (770-794). Cancellation is checked before each network read and each event (541, 565).
- `interpret_sse_event`: empty -> Ignore; the literal `[DONE]` -> Done; else JSON parsed into choices[0].delta.content (text), top-level `usage`, and choices[0].finish_reason. Empty strings count as absent. Usage with all three counts None is dropped (782). If all three parts are absent -> Ignore, else `SseEventOutcome::Chunk(SseChunk{text, usage, finish_reason})` (789-793). Only choices[0] is read (783). Malformed JSON -> `MalformedEvent` (777-780).
- Per Chunk in `translate`: `on_token(text)` is called with the raw delta (579-581), `has_text` is set only when the text is not whitespace-only (581-583), `last_usage` is overwritten (585-587) and `last_finish_reason` too (588-590).
- On Done: if `!has_text` -> `Err(EmptyReply{finish_reason})` (570-574); else `Ok(Done(last_usage.map(into_translate_usage(model))))` (575-576). Stream closed before `[DONE]` -> `StreamEndedWithoutDone` (552-557).
- `into_translate_usage`: client.rs:444-458. total falls back to prompt+completion; `cost_usd` only when both counts are Some, via pricing.
- `run_translate_call` forwards each token over a Tauri `Channel<String>` (aitranslate.rs:685-689).
- Webview: `aiTranslateAccumulatedText` is the plain concatenation of tokens (src/aiTranslateState.ts:138-143), with no trimming or post-processing (grep of that file shows none). The translation is not written to the segment automatically. `promoteAiTranslate` sends the accumulated text to `promoteAiTranslationToEditor(segmentId, text)` (src/aiTranslateHandlers.ts:73-95).
- `promote_ai_translation`: src-tauri/src/commands/segment/targets.rs:295-388 @ 7bd1f90. It rejects missing, retired, or other-chapter segments, and if the segment holds an unsigned draft and `!force` it writes nothing and returns `needs_confirmation`. Otherwise it writes via `write_non_user_target` with `TRANSLATION_ORIGIN_OTHER` (350-356) and status draft. I did not read `write_non_user_target`.

## Not found / not read
- The `extra_content.google.thought_signature` handling: not read in code (only mentioned in a ticket, which is not evidence). The serde structs ignore unknown fields, so it is dropped.
- The batch path (`ai_translate_batch`) and `prepare_resend_call` (aitranslate.rs:451+) were not traced; the resend reuses the recorded prompt and system byte for byte (456-464).
- Files read: about 10, within the 15 budget.
