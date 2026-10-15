# Handoff: Phase Rust -> Phase Webview (panel AI và hàng lô)

## Changed
- `src-tauri/src/commands/aitranslate.rs`: pure `endpoint_and_model_set(endpoint, model)` and `call_ready(endpoint, model, key_configured) -> Option<bool>`; `resolve_call_config` now uses the former.
- `src-tauri/src/commands/aiconfig.rs`: `AiConfigGetWire.ready: Option<bool>` computed from resolved endpoint/model + `key_configured` via `call_ready`. `Some(false)` = endpoint/model empty or key absent; `Some(true)` = ready; `None` = endpoint+model set but keychain cannot be asked (not "unconfigured").
- `src/config/aiconfig.ts`: wire `ready: boolean | null`; `aiConfigGet()` now returns `ready: boolean | null` (null also on failed call).
- `src-tauri/tests/aiconfig_contract.rs`: `ready_is_true_false_or_none_by_endpoint_model_and_key`.

## Webview must do
- Add `ready` to `aiConfigState.ts` (new ref, loaded by the panel on mount; do not touch Settings `reset`). Mocks of `aiConfigGet` in `tests/frontend/` (settingsFrame, aiConfigState, settingsOverlayAiConfigTierRender) omit `ready`; fine at runtime (undefined) but fix types if vue-tsc complains.
- Plan note: translate path needs a key even for Ollama (`read_api_key()? else not_configured`), so `ready` includes the key, per plan.

## Verified
- `cargo test --test aiconfig_contract --test ai_translate_contract --test ipc_contract --test config_invariants`: all green (70 in ai_translate_contract etc.; aiconfig_contract 24).
- Counter-check: replaced the `call_ready` call with `Some(true)` -> `ready_is_...` test red (line 1236); restored.
- No `npm run build` / vitest / tsc run in this phase.

# Handoff: Phase Webview -> Phase Tests

## Changed (all Phase Webview tasks done)
- `src/aiConfigState.ts`: `aiConfigReadiness` ('unknown' | 'ready' | 'unconfigured' | 'key_unchecked'; unknown = not read or read failed, key_unchecked = Rust `ready === null` with fields present), `aiConfigEffectiveProvider/Model`, `refreshAiConfigReadiness()` (panel mount; touches no drafts). `loadAiConfigSection` also fills them; `resetAiConfigSection` clears them.
- `src/aiPromptInspectorState.ts`: `aiPromptGlossaryDelta` (computed, signed, null when no previous record of another segment / not asked / zero). Previous count kept only when the new record is for a different segment; re-assemble of the same segment clears it, a plain read of the same record keeps it. Cleared by `resetAiPromptInspector`. `glossaryInjectionSummary` shape unchanged. vi keys `ai.prompt.summary_asked_{up,down}`, `summary_asked_tm_{up,down}`.
- `src/aiTranslateBatchState.ts`: pure `aiTranslateBatchErrorBlock(rows)` -> `{stoppedAt (1-based), intact {from,to}|null, notCalled {from,to}|null}|null`; `aiTranslateBatchPromotableItems(rows)` ([{segmentId,text}] of `done` rows with text), `aiTranslateBatchPromotableCount`.
- `src/panels/editorPanelState.ts`: `promoteAiTranslationBatchToEditor(items: {segmentId,text}[])` (items, not ids: deviation from plan so editor state does not import batch state) -> 'done' | 'needs-confirmation' | 'refused'. One flush, loops `promoteAiTranslation(force=false)`, gathers `needs_confirmation` into `editorPendingPromoteBatch` ({items:[{segmentId,text,draft}], written, skipped}) - a SEPARATE ref, `editorPendingPromote` shape unchanged. `confirmPendingPromote` / `cancelPendingPromote` (same commands `ai.translate.confirm_promote` / `cancel_promote`) settle the batch first when it is pending (confirm repeats with force=true; cancel counts the drafts as skipped). `editorPromoteBatchReport` {written, skipped}. Reset with the chapter-change reset block (also `promoteBatchBusy`).
- `src/aiTranslateHandlers.ts` (handlers live here, not main.ts): `discardAiTranslateBatch` (= `resetAiTranslateBatch`, which already calls `cancelWhenGenerating`), `promoteAiTranslateBatch` (refuses while batch generating or 0 done rows). `src/commands/index.ts`: deps + commands `ai.translate.batch_discard` (no key), `ai.translate.promote_batch` (`Mod+Alt+Shift+Enter`, no existing use found by grep). `check-commands.mjs` needed no change (no new keydown/mouse handlers).
- `src/panels/AiTranslationPanel.vue`: PanelFrame `:show-status="false"`; head `.ai-panel-head` with `[data-ai-panel-provider]` ("provider · model", only when readiness 'ready') and the single `[data-ai-prompt-inspector-open]`; `[data-ai-unconfigured]` block (title + 2 paragraphs + `[data-ai-unconfigured-open-settings]` -> `settings.open`, no role=alert) when 'unconfigured'; `[data-ai-key-unchecked]` hint when 'key_unchecked'; post-run `[data-ai-translate-state="not_configured"]` paragraph is hidden when 'unconfigured'. Cancel button label = "Huỷ lô" (`panel.ai_translation.cancel_batch`) while batch generating. Batch rows: `data-ai-translate-batch-row-no` (1-based position, NOT segment id), `-row-text` (translation or source from `editorSegments`), `-row-tokens` (only when usage), status label; `data-ai-translate-batch-row-segment-id` carries the id. Error block `[data-ai-translate-batch-alert]` role=alert with lines `[data-ai-batch-error-stopped|-intact|-cause|-not-called]`. `[data-ai-translate-batch-actions]` holds `-batch-retry` (label "Thử lại từ câu k"), `[data-ai-translate-promote-batch]` ("Đưa N câu đã xong sang Bản dịch", disabled when batch generating or 0 done), `[data-ai-translate-batch-discard]`. Batch confirm `[data-ai-translate-promote-batch-confirm]` reuses `data-ai-translate-promote-confirm/-cancel`; report `[data-ai-translate-promote-batch-report]`.
- `src/i18n/vi.json`: new `panel.ai_translation.*`, `command.ai.translate.{batch_discard,promote_batch}`, `ai.prompt.summary_asked_*`. Old `panel.ai_translation.status` kept (Proofreader strip + post-run paragraph use it).
- Existing tests touched only to keep them green: `editorPanelState` mocks in `aiTranslate`, `aiPromptInspector`, `aiTranslationPromoteErrorAlert` tests got the 3 new exports; `aiTranslateBatch.test.ts` two expectations (cause line now `Câu 2: ...` inside the alert; retry label "Thử lại từ câu 2"); `aiTranslateHandlersWiring.test.ts` handler list now 8 names.

## Tests still needed (matrix rows with no test yet)
Whole Phase Tests list: `aiBatchPanelRows.test.ts` (rows text/token/status, 20-row batch with error at 6, Bỏ lô, Huỷ lô, unconfigured invitation without role=alert, `ready = null` vs false vs failed read, header provider·model, Glossary delta incl. same-segment and first-record cases), batch promote (2 of 5 with drafts -> one question, confirm force=true, decline skips them, one segment failing), Rust `ready` already covered by Phase Rust. Mock `aiConfigGet` in `config/aiconfig` with `ready` to drive readiness; unmocked outside Tauri it yields 'unknown'. Counter-checks per plan Verification (force re-call, reset call in discard handler).

## Checks run
- `vue-tsc --noEmit` clean; `check:commands`, `check:panel-refs`, `check:i18n`, `check:tokens` green; eslint on touched src files clean.
- vitest: aiTranslate*, aiPrompt*, aiConfig*, proofreader*, aiTranslationPromote*, checkCommandsScan, commandsRegistry*, settings* all green (no full suite). `npm run build` not run (vue-tsc only).
- Not done: Rust-side nothing; no manual app check; `Mod+Alt+Shift+Enter` not verified in a live webview.

# Handoff: Phase Tests (done)

## Matrix row -> test (all ran and passed)
New file `tests/frontend/aiBatchPanelRows.test.ts` (16 cases) unless noted.
- Lô chạy dở có một lỗi: "hàng lô 20 câu... mỗi hàng nêu số câu · chữ · token...", "khối lỗi ba dòng đúng chữ, ba nút... Bản dịch không đổi", "lỗi KHÔNG retryable ⇒ ẩn Thử lại".
- Huỷ giữa lô: "Huỷ lô (nhãn nút... giữ chữ và token hàng xong, bỏ chữ dở)".
- Bỏ lô: three cases (no rows => hidden; running => cancel + all cleared + late frames ignored + no extra call; after error => cleared); wiring through the real handler in `aiTranslateBatchResetWiring.test.ts` ("cụm reset #4").
- Chưa cấu hình: "ready = false ⇒ lời mời... KHÔNG role=alert... nút Dịch vẫn tới ai.translate.run"; "ready = null ⇒ dòng riêng" (distinct from false); read failure case.
- Đã cấu hình: "ready = true ⇒ Ollama · qwen2.5:14b... MỘT link Xem prompt"; "đọc cấu hình lỗi ⇒ không dòng nhà cung cấp, panel vẫn dùng được".
- Đưa cả lô sang: "5 hàng xong, 2 câu có bản nháp... MỘT câu hỏi; đồng ý ⇒ force = true", "từ chối câu hỏi", "một câu không còn trong Chương", plus Plan Change Log case "gõ vào câu đang chờ hỏi rồi đồng ý ⇒ flush TRƯỚC lượt force = true".
- Đưa sang khi lô đang chạy / 0 hàng xong: "lô đang chạy ⇒ nút khoá và dispatch không gọi promote; không hàng xong ⇒ nút khoá".
- Số liệu Glossary đổi: `aiPromptInspector.test.ts` describe "dòng tóm tắt Glossary so với câu trước" (tăng 1, giảm 1, cùng câu, chưa có bản ghi trước + reset). Put there, not in the new file, because that file already owns the `config/aiprompt` mock harness.
- Rust `ready` ba giá trị: `aiconfig_contract.rs::ready_is_true_false_or_none_by_endpoint_model_and_key` (Phase Rust; true/false/None covered). No new Rust test needed in ai_translate_contract/ipc_contract/config_invariants.
- Orthogonal: `aiTranslateBatch.test.ts` needed no new edits beyond Phase Webview's.

## Counter-checks (real removal, signature kept, only that target re-run, restored)
- Remove the `force = true` loop in `settlePendingPromoteBatch` -> 3 red in aiBatchPanelRows (two-drafts-written, one-segment-fails, flush case).
- Remove the flush in `settlePendingPromoteBatch` (Plan Change Log) -> flush-before-force case red (1).
- `discardAiTranslateBatch` body emptied -> 3 red in aiBatchPanelRows + 1 red in ResetWiring "cụm reset #4".
- Remove `cancelWhenGenerating(state)` from `resetAiTranslateBatch` -> the "không câu nào bị gọi thêm" case red.
- `ready` predicate replaced by `Some(true)` in `aiconfig.rs` -> `ready_is_...` red at line 1236.
All sources restored (diff against backups clean).

## Commands run
- `npm run build` OK (vue-tsc + vite).
- `cargo test --test aiconfig_contract --test ai_translate_contract --test ipc_contract --test config_invariants`: 24 + 70 + 46 + 44 passed.
- vitest on the six plan files: 6 files, 163 passed. `vue-tsc` clean; eslint on touched tests clean.
- `check:commands`, `check:panel-refs`, `check:i18n`, `check:debt-owner` green.
- Debt items appended to `deferred-work.md` (`Chủ: Epic 12`, `Chủ: Story 12.18`).

## Not done
- No e2e / live-app check (webview does not mount here); covered by the Epic 12 debt item. No commit.
