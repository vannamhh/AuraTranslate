# Handoff: Phase Rust -> Phase Webview

## Changed
- `AssembledPromptWire` (`ai_prompt_assemble`, `ai_prompt_read_record`) gains `sent_usage: {prompt_tokens, completion_tokens, total_tokens, cost_usd|null} | null`. `null` = no real count (not sent, provider returned none, or batch-assembled record). Never 0.
- `ledger.glossary` (kind `asked`) gains `pending_matched: [{id, source_term, start, end, tier}]` (matched entries awaiting translation; `null` when `not_asked`). `id` is the glossary entry id.
- `ledger.tm` gains `omitted_count: number | null` (matching pairs cut by the 3-pair cap; `null` when `not_asked`).
- New command `ai_translate_resend` (args: only `channel: Channel<String>`; no prompt, no segment id). Returns `AiTranslateOutcomeWire` like `ai_translate_segment` (`not_configured` | `done{usage}` | `cancelled`). Sends `record.prompt` byte for byte, shares the one `AiTranslateGeneration` (starts supersede a batch/single run; `ai_translate_cancel` cancels it), stamps `sent_at/sent_model/sent_usage` after `done`. Writes no segment. Error `ai_translate.no_prompt_to_resend` when no record exists. The result belongs to `record.segment_id` (read it from `aiPromptRecord`).
- `mark_prompt_as_sent` takes a trailing `usage: Option<AiTranslateUsageWire>`; single run and resend pass the provider usage, batch passes `None`.
- New `MessageKey::AiTranslateNoPromptToResend` = `err.ai_translate.no_prompt_to_resend`.
- Rust internals: `PendingMatchedTerm`/`PendingGlossaryTerm`, `TmInjectionStatus::Searched { pairs, omitted_count }`, `prepare_resend_call`. Registered in `lib.rs`; `ai_boundary.rs` allow-list +`PendingGlossaryTerm`; census counts updated; two existing tests only got the new field/arg.

## Webview must do
- Add `"err.ai_translate.no_prompt_to_resend"` to `src/i18n/vi.json` (no params). Until then `ipc_contract` `every_message_key_exists_in_vi_json` and `every_message_key_declares_the_params_its_string_needs` are red (expected, I did not touch `src/`).
- `src/config/aiprompt.ts`: types + `isAssembledPromptWire` guard accept `sent_usage`, `pending_matched`, `omitted_count` (null-able as above).
- `src/config/aitranslate.ts`: `runAiTranslateResendCall` (Channel + `{value,error}`, command `ai_translate_resend`, no payload besides `channel`).
- Everything else per the plan's Phase Webview.

## Verified
- `cargo check --all-targets`, `cargo clippy --all-targets`, `cargo fmt --check` clean.
- `cargo test --no-fail-fast` for `ai_translate_contract`, `ai_prompt_contract`, `ai_rag_contract`, `ai_translate_wire`, `ai_boundary`, `config_invariants`: green. `ipc_contract`: 2 red only for the missing vi.json key above.
- Not run: full `cargo test` (lib.rs touched; run once after Webview), new Rust cases (Phase Tests), red-removal checks.

# Handoff: Phase Webview -> Phase Tests

## Changed
- `src/config/aiprompt.ts`: wire types and guard now require `sent_at`, `sent_model`, `sent_usage` (all nullable), `glossary.pending_matched` (null iff `not_asked`), `tm.omitted_count` (null iff `not_asked`). Strict: a record missing them is rejected.
- `src/config/aitranslate.ts`: `runAiTranslateResendCall(onToken)` (command `ai_translate_resend`, payload only `channel`).
- `src/aiTranslateState.ts`: `runAiTranslate` and new `resendAiPrompt(segmentId)` share private `runTranslateWith(segmentId, call)`; same refs, so the result lands in the AI panel. Null-segment check now runs before the `generating` check.
- `src/aiTranslateHandlers.ts`: `resendAiPrompt` handler (gates: no record, batch generating, single generating; selects Result tab; refreshes the record after the call). Handler count 8 -> 9.
- New `src/aiPromptInspectorHandlers.ts` (spread in `main.ts`): `copyAiPrompt`, `selectAiPromptResultTab`, `selectAiPromptPromptTab`, `openAiPromptTmConcordance`, `confirmAiPromptTermNow`, `openAiPromptGlossaryEntry`, `openAiPromptWorkPrompts`. The last four close the overlay via `closeAiPromptInspectorThen` (awaits `nextTick` so the focus-return watcher cannot steal focus), then dispatch.
- `src/commands/index.ts`: 8 commands `ai.prompt_inspector.{resend,copy,tab_result,tab_prompt,tm_concordance,confirm_now,open_glossary_entry,open_work_prompts}`, no keys. The plan listed 5; the last 3 are needed because every `@click` is one dispatch and the overlay must close first.
- `src/aiPromptInspectorState.ts`: `aiPromptInspectorTab` (reset to `prompt` on open and reset), `aiPromptCopyState`, `selectAiPromptInspectorTab`, `closeAiPromptInspectorThen`, `copyAiPromptToClipboard`, pure `aiPromptTierBlocks`, `aiPromptCharsByKind`, `aiPromptDynamicRatio` (glossary+tm over all chars, 0-100, null when 0 chars), `formatPercentParam`, `formatSentAtClock`, `aiPromptSourceText`, `aiPromptTokenLine` (unsent | unavailable | real), `aiPromptResultFor`.
- `src/AiPromptInspectorOverlay.vue` rewritten: full-screen, tabs, tier blocks, sidebar, excluded section. Kept `data-aip-*` hooks; added `data-aip-tab`, `data-aip-tab-panel`, `data-aip-block-kind`, `data-aip-token-state` (real|unavailable|unsent), `data-aip-dynamic-ratio`, `data-aip-cost`, `data-aip-copy(-state)`, `data-aip-resend`, `data-aip-result-state`, `data-aip-excluded-kind` (pending|suppressed|tm_omitted), `data-aip-excluded-empty`.
- `src/i18n/vi.json`: `err.ai_translate.no_prompt_to_resend`, 8 `command.ai.prompt_inspector.*`, ~27 `ai.prompt_inspector.*`.
- `scripts/check-commands.mjs`: `CLICK_FLOOR` 197 -> 214, `COMMAND_FLOOR` 230 -> 250 (gate demanded it; live 251 / 293).
- Existing tests adapted only for the new required fields / structure: `aiPromptInspector.test.ts` (fixtures; byte-for-byte test now joins `.aip-piece` spans; the suppressed-list tests now read the excluded section and assert one fix button instead of none), `aiPromptConfigGuards.test.ts` (fixtures), `aiTranslateHandlersWiring.test.ts` (9 handlers).

Decisions to know: authored pieces cannot be split into "system" vs "genre" (one body per prompt set), so one "Thân bộ prompt" block carries the prompt-set tier. "Chốt ngay" is hidden when the record is stale (it would focus the wrong segment); there is no webview check that the pending entry is still in the current sentence. Injected-term and TM-pair lists were kept under the blocks (tier, percent, reference tag).

## Tests still needed (matrix rows with no test yet)
- Sau một lượt đơn / token thật: `aiPromptTokenLine` real + cost, ratio value, meta line "gửi lúc".
- Bản ghi do lô lắp (`sent_at` set, `sent_usage` null) -> "unavailable", no 0; mới lắp (`sent_at` null) -> "unsent".
- Gửi lại: result flows into `aiTranslateState` (`runSegmentId = record.segment_id`), no segment-write command called, lock when batch/single generating or record null, not_configured, error with no retry; stale record still resends.
- Tab Kết quả: `aiPromptResultFor` per `segment_id`, other segment's result hidden, empty state text, batch vs single by stamp.
- Phần bị loại: pending row + "Chốt ngay" closes then focuses (`glossary.confirm.focus`), hidden when stale; "Xem cả N" closes, selects concordance tab, runs `runConcordance(source)`, writes nothing; empty-source case warns.
- Chép: clipboard fake success -> "Đã chép", rejection -> failed label; copies `record.prompt` verbatim.
- `aiPromptConfigGuards`: new fields rejected when missing/wrong type (`pending_matched` null on asked, `omitted_count` null on searched, `sent_usage` bad shape).
- `runAiTranslateResendCall` adapter (payload is only `channel`; wrong shape -> error).
- Rust cases listed under Phase Tests in the plan (not touched here).

## Checks run
- `npx vue-tsc --noEmit` clean; `npm run check:commands`, `check:panel-refs`, `check:i18n`, `check:tokens`, `check:lint` all exit 0.
- Full `npx vitest run`: 186 files, 2617 tests green.
- `cargo test --test ipc_contract every_message_key*`: both green (vi.json key present).
- Not run: full `cargo test`, e2e, real app (webview e2e stuck at `tauri://localhost`), `npm run build`.

# Handoff: Phase Tests (done)

## Matrix row -> test
- Lượt đơn (dòng gửi lúc, khối tầng, 412 token, tỉ lệ, chi phí): `aiPromptInspectorScreen.test.ts` "lượt đơn: dòng gửi lúc · mô hình…" + pure `aiPromptTokenLine`/`aiPromptTierBlocks`/`aiPromptDynamicRatio`; Rust `resend_sends_the_recorded_prompt…`, `stamped_usage_serializes…`.
- Lô lắp / provider không usage: "provider không trả usage…", `a_send_whose_provider_returned_no_usage…` (None, JSON null, không 0).
- Mới lắp: "mới lắp, chưa gửi" (unsent, không chi phí, khối + tỉ lệ vẫn hiện).
- Gửi lại: "đã cấu hình…" (tab Kết quả, chữ chảy dần, runSegmentId = record, usage vào aiTranslateState, `segmentCalls` rỗng, nạp lại bản ghi), "chưa cấu hình", "lỗi provider" (đều đúng 1 lần gọi); Rust `resend_sends_the_recorded_prompt_byte_for_byte_writes_no_segment…` (so byte qua `RecordingProvider`, target_text/origin/source không đổi), `resend_with_nothing_assembled…`, `resend_reports_not_configured…`.
- Khoá: "khoá khi lượt đơn đang chạy", "khoá khi lô đang chạy", "chưa có bản ghi" (nút không tồn tại + handler cảnh báo).
- Stale: "bản ghi của câu khác… vẫn gửi".
- Phần bị loại: pending + Chốt ngay (đóng rồi `glossary.confirm.focus`), "mục chờ chốt không còn…" (xem Not done/fix), stale ẩn nút, "hơn 3 cặp TM" (Xem cả N: đóng, tab concordance, `tmConcordance('SSSSSSSSSS')`, không ghi segment), thiếu câu nguồn, rỗng, Xem mục / Mở prompt Tác phẩm; Rust `ai_rag_contract` 4 ca `pending_matched` + 3 ca `omitted_count`; `ai_prompt_contract` 2 ca wire JSON (`pending_matched`, `omitted_count` 0 khác null).
- Tab Kết quả: "chưa có kết quả" (rỗng có chữ), "câu KHÁC không hiện", "kết quả lô của câu bản ghi", hàm thuần `aiPromptResultFor`.
- Chép: "chép đúng toàn bộ record.prompt", "clipboard từ chối" (failed + console.error), "mở lại màn → idle".
- Ca bản ghi bị lắp lại sau khi gửi: `a_record_reassembled_after_a_send…` (sent_* và usage về None, stamp muộn bị bỏ qua).
- Guard dây: `aiPromptConfigGuards.test.ts` mục ⑨; adapter: `aiTranslateResendCall.test.ts` (payload chỉ `channel`).
- Wiring: `aiTranslateHandlersWiring.test.ts` (spread `aiPromptInspectorHandlers`, bảy handler).

## Fix to code (known Webview gap)
Webview CAN tell: `GlossaryConfirmStrip` giữ mục đang hỏi (`tier`+`id`). Added `confirmStripEntryId` in `src/glossaryConfirmStripState.ts`; overlay `canConfirmNow(term)` = `!isStale && strip id === term.id && strip tier === term.tier`. Nút "Chốt ngay" nay ẩn khi mục đã chốt / Để sau / rời câu / dải đang hỏi mục khác (nút sẽ tới nhầm mục). Hàng và lý do vẫn hiện (không rỗng im lặng).

## Counter-checks
- Rust: trong `prepare_resend_call` đổi `r.prompt.clone()` thành `r.prompt.trim().to_uppercase()` (chữ ký giữ) -> `resend_sends_the_recorded_prompt_byte_for_byte…` đỏ, left `"CHUOI LA…"` vs right chuỗi bản ghi; khôi phục (cmp trùng).
- Rust: trong `mark_prompt_as_sent` thay `r.sent_usage = usage;` bằng `let _ = usage;` -> `resend_sends…` và `stamped_usage_serializes…` đỏ (2 ca), ca None vẫn xanh đúng như dự kiến; khôi phục.
- Webview: `canConfirmNow` rút còn `!isStale` -> ca "mục chờ chốt không còn…" đỏ (1 ca); `aiPromptTokenLine` trả `real 0` thay `unavailable` -> 2 ca đỏ; cả hai khôi phục (cmp trùng).

## Commands run
- `npm run build`; `cargo fmt`; `cargo test --test ai_translate_contract --test ai_prompt_contract --test ai_rag_contract --test ipc_contract --test config_invariants --test ai_boundary` (xanh, cuối cùng sau mọi sửa).
- `npx vitest run` trên: aiPromptInspector, aiPromptInspectorScreen (29), aiTranslateHandlersWiring, aiPromptConfigGuards, aiTranslateResendCall -> 131 xanh.
- `npx vue-tsc --noEmit`, eslint các tệp chạm, `check:commands|panel-refs|i18n|tokens` -> 0. `npm run test:story 12-12 -- --list`.

## Not done
- `wire::ai_translate_resend` là `#[tauri::command]` cần AppHandle: không gọi được từ test; ba bước nó nối (`prepare_resend_call` · `run_translate_call` · `mark_prompt_as_sent` có usage) được test riêng, phần nối (truyền `usage` vào stamp) chỉ được bảo vệ bằng đọc mã + `ipc_contract` (tên tham số, đăng ký).
- `test:story` báo `aiTranslateResendCall.test.ts` chưa khai trong plan §Code Map "Tests that move" (không sửa plan theo luật; orchestrator quyết).
- Full cargo test / full vitest / e2e / thử trong app thật chưa chạy (webview e2e đứng ở `tauri://localhost`).

# Review patches (Rust)
- `tests/ai_translate_wire.rs`: loopback SSE server can now end with a usage chunk (`Harness::with_usage`) and records request bodies. New cases: single run stamps `sent_usage` 11/7/18; resend sends the recorded prompt (body contains it), returns `Done`, stamps `sent_at`/`sent_model`/`sent_usage`, leaves every `target_text` unchanged; resend without record gives `ai_translate.no_prompt_to_resend`; batch keeps `sent_usage == None`.
- Counter-check (real removal, `usage.map(AiTranslateUsageWire::from)` -> `None`, signature kept, only `cargo test --test ai_translate_wire`): resend branch red in the resend case, single branch red in the single case, both at the `sent_usage` expect (tests/ai_translate_wire.rs:577); restored, 14 passed.
- Removed the "Story 12-12" banners in `ai_translate_contract.rs`, `ai_prompt_contract.rs`, `ai_rag_contract.rs`; `PendingGlossaryTerm` / `PendingMatchedTerm` docs are one English line; the Vietnamese key comment in `core/i18n/mod.rs` is deleted.
- Commands: `cargo test --test ai_translate_wire` (x4), `cargo fmt`.
