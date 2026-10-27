# verify-r1 (accessed 2026-10-27)

## V1 reasoning lowers MT quality: VERIFIED (weakly, mixed)
- arXiv 2510.11919 (Zebaze, Bawden, Sagot; 2025-10), search snippet only: Qwen3 0.6B-32B, thinking vs non-thinking about equal, non-thinking slightly ahead on MetricX (<0.5).
- arXiv 2511.10984 DiscoX (2025-11), snippet only: thinking models underperform non-thinking on discourse-level expert-domain MT (Qwen3-235B 59.66 vs 49.97 per summary; attributed to over-summarising/omission). Not independently opened.
- Counter: arXiv 2504.08120 (2025): o3-mini improved with reasoning; R-1 worse than non-reasoning. So "no reliable gain, often slight loss", not "always lowers". Correction: soften the claim.

## V2 Gemini 3 thinking cannot be off on OpenAI endpoint: VERIFIED (docs); independent token count UNVERIFIED
- ai.google.dev/gemini-api/docs/openai (Google, read: full via fetch): "Reasoning cannot be turned off for Gemini 2.5 Pro or 3 models"; only 2.5 accept reasoning_effort "none"; effort levels minimal/low/medium/high map to thinking_level.
- ai.google.dev/gemini-api/docs/pricing (read): output price labelled "including thinking tokens".
- Independent: only Google's Gemini 3 guide / Vertex docs (snippets) saying minimal "does not guarantee thinking is off"; thought signatures still required. No issue/forum with raw token counts for 3 Flash minimal/low found. Default level is HIGH.

## V3 DeepSeek thinking default ON, disable via thinking:{type:disabled}: VERIFIED (official only)
- api-docs.deepseek.com/guides/thinking_mode (DeepSeek, read: full). Thinking enabled by default, effort default high. Chat Completions via OpenAI SDK: `extra_body={"thinking":{"type":"disabled"}}` ("enabled"/"disabled"); reasoning_effort low/high/max is a top-level param. In thinking mode temperature/presence/frequency penalties have no effect (no error). Page does not distinguish deepseek-flash vs pro defaults except top_p behaviour. No independent observation sought (budget).

## V4 Gemini cached_tokens via OpenAI endpoint: UNVERIFIED (leaning "not reported")
- discuss.ai.google.dev/t/question-with-usage-in-openai-compatibility/98760 (Google forum, 2025-08-19/22, read: full): Gemini 2.5 Pro via OpenAI compat returned only prompt/completion/total tokens (total = prompt+completion+thinking, so completion excludes thinking); no prompt_tokens_details; unresolved. Same as the known post, no 2026 evidence found. Docs openai page silent on prompt_tokens_details. Native usage_metadata does carry cached counts (Simon Willison 2025-05-09, snippet).
- Note: usage.completion_tokens may EXCLUDE thinking tokens on this endpoint (inference from the post's arithmetic): relevant for cost accounting.

## V5 DelTA 30 -> 8, 50 -> 10 omitted: VERIFIED numbers; independent source PARTIAL
- arXiv 2410.08143 HTML (read: first 100k chars): Sec 3.1 / Table 1: missing target sentences 8 (window 30), 10 (window 50); sCOMET 83.88 / 83.70.
- Independent: arXiv 2503.10494 (2025-03, snippet): single-pass long-document translation shows reference-vs-hypothesis length gap indicating omissions; segment/multi-turn mitigates. arXiv 2507.19537 (2025-07, snippet): batch prompts chosen against for less error resilience (SKOS thesauri). LLMTranslate R package NEWS: numbered item mapping to fix batch misalignment (changelog, not a study). No study measuring omission rate vs batch size per request.

## V6 Gemini 3.8 Flash price: VERIFIED
- ai.google.dev/gemini-api/docs/pricing (read via fetch): $0.75 in / $0.075 cached / $3.75 out through 2026-12-31; $1.50 / $0.15 cached / $7.50 from 2027-01-01 (page also doubles cache storage $0.50 -> $1.00 per 1M tokens-hour). Correction: the page says cached also doubles to $0.15.
- Independent-ish: digitalapplied.com (2026-09-02, read: summary) cites a footnote in Google's launch post for $1.50/$7.50 from 2027-01-01; eesel.ai, techplained.com, layer3labs (snippets) agree. Blogs disagree on cached input post-2027 (one says unchanged, "verify"); Google pricing page says $0.15. Another blog (2026-10-08, apicostcalc, snippet) says 3.6/3.7 Flash have the same hike. Google launch post itself not opened.
