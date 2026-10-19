---
title: 'Xem prompt: màn đầy đủ tách tầng và gửi lại'
type: 'feature'
ticket: '12'
created: '2026-10-15'
status: done
baseline_revision: '9e13ca129e3d9bc69edc8add09a97e59e8b706c2'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/prompt-inspector.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Lớp phủ `AiPromptInspectorOverlay.vue` lệch `prompt-inspector.html` (b:173–175, b:177): khung ~68 % bề ngang không tab, dòng nhận diện không giờ gửi/mô hình, prompt là một `<pre>` liền không nhãn tầng, "Đã cân nhắc nhưng không chèn" chỉ có một loại và không lối sửa; không có số token thật của lượt gửi, không Chép, không Gửi lại (FR145, B25–B28).

**Approach:** Màn đầy đủ hai tab "Kết quả / Prompt đã gửi"; tab Prompt tách khối theo tầng (hệ thống · thể loại · Glossary · TM · câu cần dịch), token thật và chi phí của lượt gửi lấy từ bản ghi Rust, tỉ lệ chèn động; phần loại có lý do và lối sửa; Chép; Gửi lại là một lệnh Rust mới gửi ĐÚNG chuỗi `prompt` của bản ghi, kết quả chảy vào `aiTranslateState` của panel Đề xuất AI.

**Decisions (Ice, 2026-10-15):**
- Giữ một plan dù vượt cỡ 1.600 token (6.128).
- Số liệu tầng: mỗi khối tầng hiện số KÝ TỰ, kèm token thật cấp lượt; tỉ lệ chèn động tính theo ký tự và ghi rõ đơn vị. Không chia token theo tầng, không tokenizer.
- Token của lô: chỉ lượt đơn và Gửi lại đóng dấu usage; bản ghi do lô lắp hiện "chưa có số token thật". `run_batch_call` không đổi hợp đồng.
- "Xem cả N" của dòng TM bị loại: lệnh không phím đóng màn, chọn tab TM Concordance và chạy tra với câu nguồn của bản ghi (chỉ tra, không ghi).
- "Chốt ngay": nút đóng màn rồi dispatch `glossary.confirm.focus`; không gán `Mod+K`, không phím mới.

**Always:**
- "Gửi lại" gửi `record.prompt` do Rust giữ, byte cho byte, không lắp lại và không nhận prompt từ webview (AD-1, FR71); kết quả chỉ về panel Đề xuất AI, không đường nào ghi `target_text` hay `translation_origin` (FR72, AD-47, FR145).
- Token là số provider trả sau khi gửi (`TranslateUsage`); chưa gửi, hoặc provider không trả, là trạng thái riêng ("chưa có số token thật"), không `0`, không ước tính trước gửi (B26, spec 4.11 cấm tokenizer cục bộ). Tỉ lệ chèn động tính theo ký tự từ `ledger.pieces`.
- Mọi `@click` là đúng một `dispatch('<id>')`; lệnh mới có `labelKey`, handler mới có dòng `HANDLER_TABLE` (`check:commands`); ô nhớ module mới có `reset*` (`check:panel-refs`); chữ qua `vi.json`, màu qua token (màu tầng tái dùng accent = Glossary, nâu vàng = TM của mockup); không `v-html`.
- Mở màn vẫn chỉ ĐỌC bản ghi (Decision 2 spec 4.7); Gửi lại không lắp lại, chỉ đóng dấu `sent_at`/`sent_model`/usage lên bản ghi cùng `segment_id` + `prompt` qua `mark_prompt_as_sent`.
- Trả tiêu điểm về nút mở khi đóng (UX-DR17), Esc đóng, giữ bẫy Tab hiện có.

**Never:**
- Không ước tính token hay chi phí trước khi gửi, không tokenizer/dependency mới; không số token theo từng tầng giả bằng cách chia tổng; không gán `Mod+K`.
- Không đổi lược đồ DB, không thêm bản ghi thứ hai ngoài `LastAssembledPromptState`, không sửa prompt trong màn này; "Mở prompt Tác phẩm để sửa" chỉ mở `prompt.library.open`.
- Không tự thử lại khi Gửi lại lỗi; không chặn lô đang chạy bằng cách huỷ ngầm (dùng cổng loại trừ lô/đơn sẵn có).
- Không dựng phần "bị loại" cho thuật ngữ bị tầng khác ghi đè và cho TM dưới ngưỡng (Rust chưa có dữ liệu; nợ đề xuất, không sửa spec).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Sau một lượt dịch đơn | Bản ghi có `sent_at`, `sent_model`, usage 412 token | Dòng "gửi lúc … · mô hình …"; khối theo tầng có nhãn tầng (Global / Tác phẩm); thanh tổng "Tổng gửi đi 412 token", "Phần chèn động X %", chi phí khi `cost_usd` có | Provider không trả usage: "chưa có số token thật", không thanh |
| Bản ghi do lô lắp | Lô vừa xong; bản ghi của câu cuối có `sent_at`, `sent_usage = null` | Dòng gửi lúc/mô hình hiện; token ghi "chưa có số token thật", không 0, không chi phí | — |
| Mới lắp, chưa gửi | `sent_at = null` | Khối tầng + tỉ lệ chèn động hiện; token ghi "chưa gửi", không 0, không chi phí | — |
| Gửi lại | Bấm "Gửi lại với prompt này", đã cấu hình | Chuyển tab Kết quả, chữ chảy dần; kết quả vào panel Đề xuất AI; `target_text` và nguồn của câu không đổi; bản ghi nhận `sent_at` mới | Chưa cấu hình: lời mời như panel, không cảnh báo; lỗi provider: `tError`, không thử lại |
| Gửi lại khi chưa có bản ghi / lô đang chạy / lượt đơn đang chạy | `record = null`, hoặc `generating` | Nút khoá | Handler lặp lại cổng, chỉ cảnh báo console |
| Bản ghi của câu khác câu đang chọn | `isStale` | Gửi lại vẫn gửi bản ghi, kết quả gắn `segment_id` của bản ghi, thông báo cũ hiện như nay | — |
| Phần bị loại | Câu khớp thuật ngữ đang chờ chốt; có hơn 3 cặp TM khớp | Hàng thuật ngữ: lý do chờ chốt + "Chốt ngay"; hàng "N câu TM" vượt giới hạn + "Xem cả N" | Mục chờ chốt không còn trong câu: nút ẩn, không rỗng im lặng |
| Tab Kết quả | Chưa có kết quả cho `segment_id` bản ghi | Trạng thái rỗng có chữ, không rỗng im lặng | Kết quả lô/đơn của câu khác: không hiện |
| Chép | Bấm Chép | Toàn bộ `record.prompt` vào clipboard, nút báo "Đã chép" | Clipboard từ chối: báo lỗi trên nút, không im lặng |

</frozen-after-approval>

## Code Map

- `src/AiPromptInspectorOverlay.vue` (557 dòng) -- lớp phủ hiện tại: `aip-scrim`/`aip-panel`, `<pre>` vẽ `ledger.pieces` theo `kind`, ba section (thân/Glossary/TM) liệt kê injected + `suppressed_by_pending_overlap`, bẫy Tab, `useSelectionSurface`, trả tiêu điểm theo `[data-ai-prompt-inspector-open]`. Viết lại bố cục; giữ bẫy Tab/tiêu điểm/`data-aip-*`.
- `src/aiPromptInspectorState.ts` (282) -- `aiPromptRecord`, `refreshAiPromptRecord`, `openAiPromptInspector`, `aiPromptRecordIsStale`, `resetAiPromptInspector`. Thêm ô tab hiện tại (`reset` trả về Prompt), hàm thuần tỉ lệ chèn động từ `pieces` và gom khối theo tầng. KHÔNG gọi `aiPromptAssemble`.
- `src-tauri/src/commands/aiprompt.rs` -- `AssembledPromptRecord` 325, `AssembledPromptWire` 369, `mark_prompt_as_sent` 556 (so `segment_id` + `prompt` byte; đặt `sent_at`/`sent_model`), `LastAssembledPromptState` 360. Thêm `sent_usage: Option<AiTranslateUsageWire>` vào record + wire; mở rộng `mark_prompt_as_sent` nhận usage. Các lượt lắp lại phải đặt `None` như `sent_at`.
- `src-tauri/src/commands/aitranslate.rs` -- `PreparedTranslateCall` 207, `resolve_call_config` 263, `read_api_key` 311, `prepare_translate_call` 343 (lắp lại; KHÔNG dùng cho resend), `run_translate_call` 556 (seam test), `send_prepared_translate_call` 750, `single_run_outcome_wire` 874, `wire::ai_translate_segment` ~906 (mẫu vỏ: sinh `generation`, dựng prepared trong khối thả khoá, gửi, `mark_prompt_as_sent`). Thêm `prepare_resend_call(global, open, record)` thuần (đọc `record.prompt`, `resolve_call_config`, `read_api_key`; `NotConfigured` là trạng thái, thiếu bản ghi là `IpcError` khoá mới) và vỏ `wire::ai_translate_resend` cùng khuôn `ai_translate_segment`, trả `AiTranslateOutcomeWire`.
- `src-tauri/src/lib.rs` ~1389 -- `generate_handler!` đăng ký vỏ mới ⇒ chạm wiring chung, chạy full suite tay. Tests đếm/đăng ký lệnh: `ipc_contract.rs:2447` (`the_ai_translate_wires_are_registered_…`), `config_invariants.rs:2084` (hàng `src/commands/aitranslate.rs` plain 3 → 4), `ai_boundary.rs:1869`; chạy cả ba.
- `src-tauri/src/core/ai/rag.rs` -- `InjectionLedger` 190, `GlossaryInjectionStatus::Asked` ~107, `TmInjectionStatus::Searched` 130, `select_tm_pairs` (cắt `TM_PAIR_CAP = 3` sau khi gom pool ≥ ngưỡng), `gather_glossary_context`. Thêm: các mục chờ chốt khớp câu (`pending_matched`: nguồn, tầng, `id`, vị trí) và số cặp TM khớp nhưng không chọn (`omitted_count`).
- `src-tauri/src/core/glossary/store.rs:1691` `confirmed_terms_for_injection` -- đã có `selected` (cả chờ chốt) và `payload`; mục chờ chốt thắng bị lọc ở đây nên là nơi duy nhất thu `pending_matched` mà không quét lần hai. Mở rộng `GlossaryInjectionOutcome` (đang cố ý KHÔNG `Default`).
- `src/config/aiprompt.ts` -- kiểu `AssembledPromptWire`/`InjectionLedgerWire`/`isAssembledPromptWire` (guard dây phải nhận trường mới). `src/config/aitranslate.ts` 114 `runAiTranslateSegment` (mẫu Channel + `{value,error}`): thêm `runAiTranslateResendCall` cùng khuôn, `CMD_TRANSLATE_RESEND`.
- `src/aiTranslateState.ts` -- `runAiTranslate` 188 dòng: tổng quát hoá thân thành hàm nhận "bộ gọi" để `resendAiPrompt()` dùng chung `state/accumulatedText/usage/runSegmentId/sequence` (kết quả về panel Đề xuất AI, `runSegmentId = record.segment_id`). `src/aiTranslateHandlers.ts` -- thêm `resendAiPrompt` có cổng loại trừ lô như `runAiTranslate`.
- `src/commands/index.ts` -- `ai.prompt_inspector.open` 4522/close 4533 (mẫu); thêm `ai.prompt_inspector.resend`, `ai.prompt_inspector.copy`, `ai.prompt_inspector.tab_result`/`tab_prompt` (không phím); `prompt.library.open` 4438 cho "Mở prompt Tác phẩm"; `glossary.confirm.focus` 3652 và `glossary.manage.open` 4227 cho lối sửa. `CommandDeps` thêm deps tương ứng, `src/main.ts` nối.
- "Xem cả N": lệnh `tm.concordance` (`openTmConcordance: () => void`, `src/tmConcordanceCommandDeps.ts`) chỉ đọc vùng chọn, không nhận văn bản -- KHÔNG đổi nó. Lệnh mới `ai.prompt_inspector.tm_concordance` gọi `selectLookupTab('concordance')` (`src/panels/lookupHistoryState.ts:319`) rồi `runConcordance(text)` (`src/panels/concordanceState.ts:49`) với câu nguồn của bản ghi, sau khi đóng màn.
- `src/panels/AiTranslationPanel.vue` -- hiện `aiTranslateAccumulatedText`, `aiTranslateUsage`, nút Đưa sang; không đổi trừ khi kiểm chứng kết quả Gửi lại hiện đúng. `src/aiTranslateBatchState.ts::aiTranslateBatchTextForSegment` -- nguồn kết quả lô cho tab Kết quả.
- `src/i18n/vi.json` -- `ai.prompt_inspector.*` (23 khoá), `ai.prompt.*`, `command.ai.prompt_inspector.*`; thêm khoá lỗi cho "chưa có bản ghi để gửi lại" (kiểm cổng `check:i18n` về khoá lỗi).
- Mockup `ux-auratranslate/mockups/prompt-inspector.html` (đã đọc: tab, `.meta`, năm `.blk`, `.excl`/`.erow`, `.side` thanh ngân sách + 3 nút). Dòng: `bang-lech-b.md` 173–175, 177 (176 loại 1, không làm); `phan-loai-loai-5-b.md` hàng 25–28.
- Tests: Rust `ai_translate_contract.rs` (seam `run_translate_call` + `FakeProvider` 1251 + `dummy_prepared` 1282 + `collecting_channel` 1297; usage qua `FakeFinish::DoneWithUsage`), `ai_prompt_contract.rs`, `ai_rag_contract.rs`; vitest `aiPromptInspector.test.ts`, `tests/frontend/aiPromptInspectorScreen.test.ts` (mới), `tests/frontend/aiTranslateResendCall.test.ts` (mới), `aiTranslateHandlersWiring.test.ts`, `aiPromptConfigGuards.test.ts`. e2e không giả được provider AI.
- Nợ: grep "B25–B28", "Xem prompt", "12.12" trong `deferred-work.md` không có mục nào mang `Chủ` là story này (hai mục "Xem prompt" ~12894, 12956 là TM, không liên quan).

## Tasks & Acceptance

**Execution:**

Phase Rust:
- [ ] `src-tauri/src/core/glossary/store.rs`, `src-tauri/src/core/ai/rag.rs`, `src-tauri/src/commands/aiprompt.rs` -- thêm `pending_matched` (kèm `id` mục và tầng) vào kết quả injection → ledger → wire, và `omitted_count` vào `TmInjectionStatus::Searched`; `sent_usage` vào record/wire; `mark_prompt_as_sent` nhận usage, nhánh lắp lại đặt `None` -- dữ liệu cho tầng/lối sửa/token thật (b:175, b:177, B25–B26).
- [ ] `src-tauri/src/commands/aitranslate.rs` -- truyền `usage` của lượt đơn vào `mark_prompt_as_sent`; thêm `prepare_resend_call` + `wire::ai_translate_resend` (đóng dấu sau `Done`, cùng `AiTranslateGeneration`) -- Gửi lại đúng prompt đã lắp, không ghi `target_text` (B27, FR145).
- [ ] `src-tauri/src/lib.rs`, `src-tauri/tests/config_invariants.rs`, `src-tauri/tests/ipc_contract.rs`, `src-tauri/tests/ai_boundary.rs` -- đăng ký lệnh và cập nhật các bảng đếm/danh sách lệnh.

Phase Webview:
- [ ] `src/config/aiprompt.ts`, `src/config/aitranslate.ts` -- kiểu và guard cho trường mới; `runAiTranslateResendCall`.
- [ ] `src/aiTranslateState.ts`, `src/aiTranslateHandlers.ts`, `src/commands/index.ts`, `src/main.ts`, `scripts/check-commands.mjs` -- `resendAiPrompt` dùng chung ô nhớ với lượt dịch đơn; lệnh `resend`, `copy`, `tm_concordance`, hai lệnh tab (tất cả không phím); `HANDLER_TABLE`.
- [ ] `src/aiPromptInspectorState.ts` -- ô tab (+ `reset`), hàm thuần tỉ lệ chèn động và gom khối tầng từ `pieces`/`prompt_set_tier`.
- [ ] `src/AiPromptInspectorOverlay.vue`, `src/i18n/vi.json` -- màn đầy đủ: thanh tab, dòng "gửi lúc · mô hình · Câu · Chương · bộ prompt", khối theo tầng có nhãn và số ký tự, thanh bên token thật / chi phí / tỉ lệ chèn động, nút Chép · Gửi lại · Mở prompt Tác phẩm, tab Kết quả (chữ chảy dần hoặc kết quả lô của `segment_id` bản ghi), phần "Đã cân nhắc nhưng không chèn" với chờ chốt + bị che + TM vượt giới hạn và lối sửa (Chốt ngay → đóng màn rồi `glossary.confirm.focus`; Xem mục → `glossary.manage.open`; Xem cả N → `ai.prompt_inspector.tm_concordance`) -- b:173–175, b:177, B25–B28.

Phase Tests:
- [ ] `src-tauri/tests/ai_translate_contract.rs`, `ai_prompt_contract.rs`, `ai_rag_contract.rs` -- resend gửi đúng chuỗi bản ghi (so byte với `FakeProvider`), không ghi `segment`, đóng dấu usage; `pending_matched`/`omitted_count`; ca "bản ghi bị lắp lại sau khi gửi" (stamp bỏ qua); usage `None` ≠ 0.
- [ ] `tests/frontend/aiPromptInspector.test.ts` (+ tệp mới `aiPromptInspectorScreen.test.ts`), `aiTranslateHandlersWiring.test.ts` -- khối tầng, tỉ lệ, token thật vs chưa gửi, Gửi lại đưa kết quả vào `aiTranslateState` và không gọi lệnh ghi segment, khoá khi lô chạy, tab Kết quả theo `segment_id`, Chép (clipboard giả, đường lỗi).
- [ ] Đối chứng đỏ (gỡ THẬT): nối `prompt` lại thay vì đọc từ record → ca byte-so đỏ; gỡ `mark_prompt_as_sent` usage → ca token đỏ.

**Acceptance Criteria:**
- Given một lượt dịch AI vừa xong, when mở Xem prompt, then thấy các tầng tách riêng và số token thật của lần gửi đó (không phải ước tính).
- Given màn Xem prompt, when bấm "Gửi lại với prompt này", then một đề xuất mới xuất hiện ở panel Đề xuất AI và `target_text` của câu không đổi.
- Given chưa gửi lần nào, when mở màn, then không hiện số token nào (không `0`).
- Given một bản ghi do lô dịch lắp, when mở màn, then dòng token ghi "chưa có số token thật" và mỗi khối tầng hiện số ký tự của nó.
- Given câu khớp một thuật ngữ đang chờ chốt, when bấm "Chốt ngay", then màn đóng và tiêu điểm ở dải chốt Glossary của câu.
- Given có hơn 3 cặp TM khớp, when bấm "Xem cả N", then màn đóng, panel Tra cứu mở tab TM Concordance với kết quả tra câu nguồn, và không bản dịch nào bị ghi.

## Implementation Notes

- Gửi lại là lệnh `ai_translate_resend` chỉ nhận `channel`: đọc `record.prompt` trong `LastAssembledPromptState`, dùng chung `AiTranslateGeneration`, đóng dấu qua `mark_prompt_as_sent` sau `Done`. Bản ghi bị lắp lại giữa lúc gửi thì dấu bị bỏ, vì phép so byte `prompt` không khớp.
- `sent_usage` là `None` cho lô theo Quyết định của Ice; `TmInjectionStatus::Searched` thành struct `{ pairs, omitted_count }`.
- Webview thêm 8 lệnh không phím, không phải 5: mỗi `@click` là đúng một `dispatch`, mà ba lối sửa phải đóng màn trước nên mỗi lối cần lệnh riêng.
- Thân bộ prompt là một khối `authored` nên không tách được hệ thống / thể loại; Glossary/TM xen giữa cho nhiều khối "Thân bộ prompt" theo đúng thứ tự (nợ `Chủ: Winston`).
- "Chốt ngay" chỉ hiện cho đúng mục dải chốt đang hỏi (`confirmStripEntryId` + tầng), vì `glossary.confirm.focus` không nhắm được mục cụ thể; hàng khác có "Xem mục".
- Phần nối `usage` ở hai vỏ wire được canh qua `ai_translate_wire.rs` (MockRuntime + SSE loopback có khúc usage); đối chứng gỡ thật đỏ ở cả hai nhánh.
- Sàn `CLICK_FLOOR`/`COMMAND_FLOOR` (`check-commands.mjs`) và `ITEM_FLOOR` (`check-debt-owner.mjs`) được nâng theo `ceil(0.85 × live)` vì chính cổng đòi.

## Plan Change Log

## Review Triage Log

- false: "Tổng gửi đi" hiện `prompt_tokens` là đúng: mockup cộng 86+104+42+118+62 = 412 là token ĐẦU VÀO, không gồm completion.
- low · patch: `:key` hàng chờ chốt trùng khi một mục khớp hai lần; khoá theo chỉ số. Ca "khớp hai lần" kiểm hai hàng, nhưng không đỏ khi gỡ (Vue chỉ vấp key trùng lúc vá lại).
- medium · patch: hàng chờ chốt mà dải không đang hỏi không có lối sửa; thêm "Xem mục" (`glossary.manage.open`) khi "Chốt ngay" ẩn, vì `glossary.confirm.focus` chỉ tới mục dải đang hỏi.
- medium · defer: không tách "hệ thống / thể loại" vì bộ prompt chỉ có một thân; mục nợ `Chủ: Winston`.
- medium · patch: chưa ghi mục nợ; đã ghi kiểm app thật (`Chủ: Epic 12`) và hai loại "bị loại" thiếu dữ liệu (`Chủ: Winston`).
- false: cảnh báo THIẾU KHAI của `test:story` đã sửa trước review (Code Map khai đủ hai tệp test mới).
- medium · patch: phần nối `usage` ở nhánh `Done` của hai vỏ wire không test nào canh; thêm ca qua `ai_translate_wire.rs` (MockRuntime + SSE loopback), đối chứng đỏ bằng gỡ thật.
- false: các comment bị xoá ở overlay mang id finding/spec trên dòng đã viết lại; hai bất biến còn đúng (chỉ đọc, ba giá trị không collapse) vẫn ở đầu tệp bằng tiếng Anh.
- low · patch: comment mới tiếng Việt và banner story id trong test; viết lại tiếng Anh hoặc xoá. Chuỗi kiểm kê ở `config_invariants.rs` giữ story id như 48 hàng cạnh nó (dữ liệu, không phải comment).
- low · reject: đóng dấu bị bỏ im lặng khi `now_utc_iso8601` trả `None` — thừa hưởng nhánh lượt đơn, chỉ khi đọc đồng hồ store trượt; sửa cần thêm nhánh.
- medium · patch: tab Kết quả hiện chữ của lượt đã huỷ như kết quả trọn; thêm dòng `panel.ai_translation.state_cancelled`, ca test đỏ khi gỡ.

## Design Notes

Vì sao lệnh Rust mới thay vì webview gửi prompt: `prepare_translate_call` luôn lắp lại, nên "Gửi lại đúng prompt đã lắp" không có đường sẵn; để Rust đọc `LastAssembledPromptState` giữ nguyên mệnh đề FR71 (chuỗi đã gửi = chuỗi đã lắp) và không để webview đưa văn bản bất kỳ ra ngoài. `mark_prompt_as_sent` so `prompt` byte nên đóng dấu lại vẫn đúng.

Token: provider chỉ trả tổng theo lượt (`TranslateUsage`), nên mọi con số "thật" là cấp lượt. Số mẫu 86/104/42/118/62 theo tầng của mockup chỉ có được khi có bộ đếm token (spec 4.11 §Never).

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test ai_translate_contract --test ai_prompt_contract --test ai_rag_contract --test ipc_contract --test config_invariants --test ai_boundary` -- xanh; vì chạm `lib.rs`, chạy full `cargo test` và full vitest một lần bằng tay.
- `npx vitest run tests/frontend/aiPromptInspector.test.ts tests/frontend/aiPromptInspectorScreen.test.ts tests/frontend/aiTranslateHandlersWiring.test.ts tests/frontend/aiPromptConfigGuards.test.ts` -- xanh; `npm run test:story 12-12 -- --list`.
- `npm run check:commands`, `check:panel-refs`, `check:i18n`, `check:tokens` -- xanh.

**Manual checks (if no CLI):**
- `npm run tauri dev` với Ollama: dịch một câu, mở Xem prompt, đối chiếu `prompt-inspector.html`; Gửi lại và xem panel Đề xuất AI, chắc `target_text` không đổi. Webview e2e đứng ở `tauri://localhost`, nên đây là nợ `Chủ: Epic 12`.
