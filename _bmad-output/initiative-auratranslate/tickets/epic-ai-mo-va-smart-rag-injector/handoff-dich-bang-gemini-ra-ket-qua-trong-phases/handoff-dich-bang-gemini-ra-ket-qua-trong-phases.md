# Handoff Phase 1 (Rust) -> Phase 2 (Tests), bug 4.14

Not committed. Plan: `tickets/bug-dich-bang-gemini-ra-ket-qua-trong-plan.md`.

## Changed
- `src-tauri/src/core/ai/client.rs`
  - `SseEventOutcome` is now `Ignore | Done | Chunk(SseChunk)`. `Token(String)` and `Usage(ChunkUsage)` are REMOVED.
  - New `pub struct SseChunk { text: Option<String>, usage: Option<ChunkUsage>, finish_reason: Option<String> }` (derives Debug, Clone, PartialEq, Default).
  - `interpret_sse_event`: empty `text` and empty `finish_reason` count as absent; `usage` with all-None counts as absent; all three absent => `Ignore`; else `Chunk`. Signature unchanged.
  - `ChunkChoice` gained `finish_reason: Option<String>`.
  - `OpenAiClientError::EmptyReply { finish_reason: Option<String> }` (Display `ai_client[empty_reply] finish_reason=..`, Debug struct).
  - `translate()`: per Chunk calls `on_token(text)` first, then overwrites `last_usage` / `last_finish_reason`; `has_text` set only when text is not whitespace-only. On `Done` with `!has_text` => `Err(EmptyReply { finish_reason: last_finish_reason })`.
- `src-tauri/src/commands/aitranslate.rs`: family `("ai_translate.reply_empty", key, retryable=false)`; key `AiTranslateReplyEmptyWithReason` when finish_reason is Some, else `AiTranslateReplyEmpty`. `insert_status_params` adds param `finish_reason` only when Some. Connection-check mapping (`other @ (...)`) now also lists `EmptyReply` (falls to `IpcError::from`, unreachable in practice).
- `src-tauri/src/core/i18n/mod.rs`: `AiTranslateReplyEmpty` = `err.ai_translate.reply_empty` `[]`; `AiTranslateReplyEmptyWithReason` = `err.ai_translate.reply_empty_with_reason` `["finish_reason"]`.
- `src/i18n/vi.json`: both keys, wording from plan Decisions.

## Red by design
- `tests/ai_translate_contract.rs` does not compile: uses removed `SseEventOutcome::Usage` / `::Token` (~L398, 434, 499, 515, plus doc lines 387, 489, 504). Rewrite to `Chunk(SseChunk{..})`. Case ~L489 `a_frame_carrying_both_usage_and_delta_content_reads_as_usage_and_discards_the_content` pins the wrong behaviour: rewrite to expect text AND usage in one Chunk.
- Other tests may exist that match `OpenAiClientError` exhaustively (none found in the 5 targets below).

## Run (Phase 1)
- `cargo check`: ok.
- `cargo test --test` ai_translate_wire 29 ok · ipc_contract 48 ok · ai_boundary 29 ok (1 ignored) · ai_proofread_contract 21 ok.
- `ai_translate_contract`: compile error (above), expected.
- `npm run check:i18n`: green.
- NOT run: vitest frontend files; red-control (Phase 2 task).
- Note: ai_translate_wire is green only because no existing case sends a frame with both text and usage; new Gemini case is Phase 2.

# Phase 2 (Tests) done
- `ai_translate_contract.rs`: imports `SseChunk`; Usage/Token cases rewritten to `Chunk`; "both" case now keeps text+usage; added multi-frame order, finish_reason-without-text, role-only Ignore, text-only Chunk, `EmptyReply` family case.
- `ai_translate_wire.rs`: added Gemini OK stream, finish_reason empty, bare `[DONE]` empty, batch stop at first sentence (4 cases + helpers `sse_body`, `GEMINI_*_FRAMES`).
- Red counter-check done (text dropped when usage present): 2 contract + 1 wire case red; restored byte-identical from backup.
- Not committed. Implementation Notes line added in plan.
