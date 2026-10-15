---
title: 'Panel AI và hàng lô'
type: 'feature'
ticket: '11'
created: '2026-10-15'
status: 'built'
baseline_revision: 'b9aca374363e97158cc50fc9c7156fe7d2e782f0'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/ai-batch-translate.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Panel Đề xuất AI (`AiTranslationPanel.vue`) lệch `ai-batch-translate.html`: đầu panel không nêu nhà cung cấp/mô hình, "Xem prompt" nằm trong một thanh riêng, hàng lô chỉ có số câu và nhãn trạng thái (không chữ, không token), không có "Bỏ lô", "Huỷ dịch" không phân biệt lô, lỗi giữa lô là một cảnh báo trần, trạng thái chưa cấu hình thiếu tiêu đề và dòng Ollama/LM Studio, và không có lối đưa cả lô sang Bản dịch.

**Approach:** Đầu panel đọc cấu hình AI (nhà cung cấp · mô hình) và đặt link Xem prompt cạnh tiêu đề; mỗi hàng lô hiện số câu, chữ rút gọn (nguồn khi chưa có bản dịch), token và chữ trạng thái; thêm "Bỏ lô", nhãn "Huỷ lô", khối lỗi ba dòng và trạng thái chưa cấu hình theo mockup; "Đưa N câu đã xong sang Bản dịch" là webview lặp lệnh `promote_ai_translation` hiện có (B22).

**Decisions (Ice, 2026-10-15):**
- B22 chọn phương án (a): webview lặp `promote_ai_translation`, một lần flush, gom mọi `needs_confirmation` vào một câu hỏi; không lệnh Rust mới, không đổi `lib.rs`. Lô không nguyên tử: lỗi giữa chừng để lại phần đã ghi, báo số câu đã ghi / bị bỏ. Không đo hai phương án.
- "Bỏ lô" bỏ ngay một lần bấm như mockup, kể cả khi còn hàng xong chưa đưa sang.
- Giữ một plan dù vượt cỡ 1.600 token.
- Trạng thái chưa cấu hình không ghi "Ollama / LM Studio chạy không cần khoá" của mockup: lượt dịch hiện đòi khoá cả với nhà cung cấp cục bộ (`read_api_key` trong `aitranslate.rs`), nên đoạn đó nói đúng hành vi hiện tại (nhà cung cấp cục bộ vẫn cần một khoá bất kỳ trong Cài đặt). Lệch mockup thành một mục nợ.

## Boundaries & Constraints

**Always:**
- Mọi lượt ghi `target_text` của "đưa sang" đi qua `write_non_user_target` (một câu `UPDATE` đặt chữ, hai mốc so và `translation_origin = 'other'`, AD-47 ①/③, AD-50 quy tắc 3); bản nháp chưa ký phải hỏi trước khi ghi (AD-49 iii, khuôn `needs_confirmation`/`force`). Không một đường ghi thứ hai.
- Lỗi giữa lô không tự thử lại và không ghi vào Bản dịch (AD-22, FR75); "Huỷ" dừng ở ranh giới câu và giữ phần đã xong (FR73); "Bỏ lô" khác "Huỷ": dừng lượt đang chạy rồi xoá mọi hàng khỏi panel.
- "Chưa cấu hình" là lời mời, không phải lỗi: không màu `error`, không `role="alert"` (UX-DR27, FR77). Điều kiện "đã cấu hình" do Rust quyết (cùng vị từ với `resolve_call_config` + khoá), webview không chép lại (AD-1); khoá không hỏi được (`key_configured = None`) là trạng thái thứ ba, không vẽ như "chưa cấu hình".
- Mọi `@click` là đúng một `dispatch('<id>')`; lệnh mới có `labelKey` trong `vi.json`, handler phím/chuột mới có dòng `HANDLER_TABLE` (`scripts/check-commands.mjs`); ô nhớ module mới có hàm `reset*` (`check:panel-refs`); chữ mới qua `vi.json`, màu/cỡ qua token.
- Lô có lỗi ở vitest dựng bằng `invoke` giả ở biên `config/aitranslate` (khuôn `tests/frontend/aiTranslateBatch.test.ts`); phía Rust dùng seam `run_translate_call` / `run_batch_call` + `FakeProvider` của `tests/ai_translate_contract.rs`.

**Never:**
- Không dựng màn Xem prompt đầy đủ, token thật sau gửi hay Gửi lại (Story 12.12); link Xem prompt chỉ gọi `ai.prompt_inspector.open` hiện có. Không chọn nhà cung cấp/mô hình, không Thử kết nối (Story 12.18): đầu panel chỉ đọc.
- Không gán `⌘↵` (đã là `editor.confirm_segment`), `Esc` hay `⌘T` (mockup ghi, 12.18 sở hữu) cho lệnh AI — chúng nằm ngoài story; không ước tính chi phí trước khi gọi, không bảng "Token và chi phí phiên này" / "Phiên này: N token" ở thanh trạng thái (mockup mục 3, không dòng nào của story).
- Không tự thử lại, không lặp im lặng; không đổi `AiTranslateBatchEventWire` hay luật một `Channel` cho cả lô; không đổi `promote_ai_translation` từng câu (lệnh đơn vẫn là nguồn duy nhất cho nút "Đưa sang bản dịch" hiện có).
- Không dependency mới; không `v-html`; không đổi lược đồ DB.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Lô chạy dở có một lỗi | Lô 20 câu; 5 xong, câu 6 lỗi `retryable`, 14 chưa gọi | Mỗi hàng: số câu · chữ (bản dịch rút gọn, hoặc nguồn nếu chưa có) · chữ trạng thái · token (chỉ hàng xong có usage). Hàng 6 "Lỗi", hàng 7–20 "Chưa tới lượt". Khối lỗi: "Lô dừng ở câu 6", "1–5 còn nguyên", "6 + nguyên nhân", "7–20 chưa gọi, chưa tính phí"; nút Thử lại từ câu 6, Đưa 5 câu đã xong sang, Bỏ lô | Lỗi không `retryable`: ẩn Thử lại, còn Đưa sang và Bỏ lô |
| Huỷ giữa lô | Lô đang chạy, bấm Huỷ lô | Dừng ở ranh giới câu; câu đang sinh dở bỏ chữ; câu xong giữ chữ và token | — |
| Bỏ lô | Có hàng (đang chạy, lỗi hoặc xong) | Lượt đang bay bị huỷ, toàn bộ hàng, dòng tiến độ, tổng token và lỗi lô biến mất; Bản dịch không đổi | Không hàng nào: nút ẩn |
| Chưa cấu hình | Endpoint hoặc mô hình rỗng, hoặc khoá chưa có | Đầu panel không có "nhà cung cấp · mô hình"; thân panel hiện tiêu đề + hai đoạn (mọi năng lực khác vẫn chạy; nhà cung cấp cục bộ như Ollama / LM Studio vẫn cần một khoá bất kỳ) + chỉ lối mở Cài đặt; nút Dịch vẫn tới `ai.translate.run` và nhận `not_configured` như nay | Khoá không hỏi được: dòng riêng "không kiểm tra được khoá", không phải lời mời |
| Đã cấu hình | `ai_config_get` trả nhà cung cấp "Ollama", mô hình "qwen2.5:14b" | Đầu panel: "Ollama · qwen2.5:14b" cạnh tiêu đề, link Xem prompt cạnh đó | Đọc cấu hình lỗi: không dòng nhà cung cấp, panel vẫn dùng được |
| Đưa cả lô sang | 5 hàng xong; 2 câu đích có bản nháp chưa ký | Ba câu ghi ngay (chữ + mốc + `translation_origin = 'other'`); hỏi một lần cho hai câu còn lại (hiện bản nháp), đồng ý thì ghi, giữ thì bỏ hai câu đó | Một câu không còn trong Chương: câu đó lỗi, các câu khác vẫn ghi; báo số câu đã ghi / bị bỏ |
| Đưa sang khi lô đang chạy hoặc không hàng xong | Lô `generating` hoặc 0 hàng `done` | Nút khoá | — |
| Số liệu Glossary đổi so với câu trước | Lắp prompt câu A (2 thuật ngữ) rồi câu B (3) | Dòng tóm tắt: "Đã chèn 3 thuật ngữ Glossary — tăng 1 so với câu trước" | Lắp lại cùng câu hoặc chưa có bản ghi trước: không phần "so với" |

</frozen-after-approval>

## Code Map

- `src/panels/AiTranslationPanel.vue` (892 dòng) -- template: `ai-prompt-bar` + `ai-inspector-bar` (nút `data-ai-prompt-inspector-open`, `data-ai-prompt-assemble`), `ai-translate-actions` (Dịch/Huỷ/Đưa/Thử lại), `ai-batch-bar`, danh sách `ai-batch-rows` (~595: chỉ `row.segmentId` và nhãn trạng thái). `aiTranslateBatchRowStatusKey`, `aiTranslateUsageLine`, `aiTranslateBatchUsageLine`, `canPromoteAiTranslate`, `canCancelAiTranslate`. Trạng thái chưa cấu hình hiện chỉ qua `PanelFrame status-key="panel.ai_translation.status"` và `data-ai-translate-state="not_configured"` sau một lượt chạy.
- `src/aiTranslateBatchState.ts` -- `AiTranslateBatchRow {segmentId,status,text,usage}`, `aiTranslateBatchRows`, `aiTranslateBatchUsageSummary`, `aiTranslateBatchRetryIds`, `resetAiTranslateBatch` (đã huỷ lượt đang bay + xoá hàng: dùng lại cho "Bỏ lô"), `cancelAiTranslateBatch`. Hàng `error` giữ `text`; hàng `cancelled` xoá `text`.
- `src/config/aitranslate.ts` -- `runAiTranslateBatchCall`, `AiTranslateBatchEventWire`, `AiTranslateUsageWire` (`prompt/completion/total_tokens`, `cost_usd`).
- `src/commands/index.ts` -- `ai.translate.*` ~4578–4880 (`run`, `cancel`, `promote` keys `Mod+Shift+Enter` ~4787, `batch_run` ~4835, `batch_retry` ~4869), `CommandDeps` ~1155–1215 (`promoteAiTranslate`, `runAiTranslateBatch`, `retryAiTranslateBatch`), `ai.prompt_inspector.open` ~4516. `ai.translate.run`/`cancel` cố ý không phím (comment ~4557–4580). `src/main.ts` -- handler thật của các deps trên (cổng loại trừ lô/đơn, `aiTranslateBatchTextForSegment` cho promote).
- `src/panels/editorPanelState.ts` -- `promoteAiTranslationToEditor` (365: flush rồi `promoteAiTranslation`, `needs_confirmation` → `editorPendingPromote`, `replaceEditorSegment`), `confirmPendingPromote` 413, `cancelPendingPromote`, `editorSegments` (có `source_text`/`target_text`). `src/config/segment.ts:1115` `promoteAiTranslation`.
- `src-tauri/src/commands/segment/targets.rs:276` `promote_ai_translation` (một giao dịch/câu; `write_non_user_target` 237; `PromoteAiTranslationOutcome` 211); vỏ IPC `wire.rs:339`; đăng ký `lib.rs:1410`. `tm_match.rs` / `proofread_accept.rs` đã gọi lại `promote_ai_translation` hoặc cùng kiểu kết quả.
- `src-tauri/src/commands/aitranslate.rs` -- `resolve_call_config` 246 (rỗng endpoint/model ⇒ `None`), `read_api_key` (`Ok(None)` ⇒ chưa cấu hình), `run_batch_call` 670 (seam; dừng ở lỗi đầu, hỏi `should_cancel` giữa hai câu), `ai_translate_batch` ~982, `AiTranslateGeneration`, `ai_translate_cancel`.
- `src-tauri/src/commands/aiconfig.rs` -- `ai_config_get` ~150, `AiConfigGetWire` 123 (`fields`, `work_tier_available`, `key_configured: Option<bool>`); `src/config/aiconfig.ts` `aiConfigGet` 90; `src/aiConfigState.ts` `loadAiConfigSection`, `aiConfigFieldWire(field)`, `aiConfigKeyConfigured` (`AI_CONFIG_FIELDS`: provider, endpoint, model, temperature, max_tokens).
- `src/aiPromptInspectorState.ts` -- `aiPromptRecord` (`ledger.glossary.injected`), `glossaryInjectionSummary` 216, `refreshAiPromptRecord` 97, `resetAiPromptInspector` 248. Rust chỉ giữ MỘT bản ghi lắp (`LastAssembledPromptState`, `aiprompt.rs:353`), nên "so với câu trước" phải do webview nhớ.
- `src/i18n/vi.json` -- `panel.ai_translation.*` 1186–1205, 1316–1319; `ai.translate.*` 104–106, 160–164; `ai.prompt.*` 165–169; `command.ai.translate.*`, `command.ai.prompt_inspector.open` 637.
- Mockup: `ux-auratranslate/mockups/ai-batch-translate.html` (mục 1 lô đang chạy, mục 2 lỗi giữa lô); ảnh `change-hoan-thien-giao-dien-theo-mockup/anh/de-xuat-ai-chua-cau-hinh-co-tac-pham.png`. Dòng nguồn: `bang-lech-b.md` 152, 155, 160, 162; `bang-lech-a.md` 170; `phan-loai-loai-5-b.md` hàng 19–23.
- Tests: Rust `src-tauri/tests/ai_translate_contract.rs` (có sẵn ca lô lỗi ở câu 6/12 ~3121, dừng không gọi câu 7–12; ca promote + AD-49 iii ~1945–2036; `FakeProvider`, `collecting_batch_channel`), `ipc_contract.rs`/`config_invariants.rs` cho hình dạng wire cấu hình. Vitest: `aiTranslateBatch.test.ts`, `aiTranslateBatchResetWiring.test.ts`, `aiTranslatePromoteConfirmFlow.test.ts`, `aiPromptInspector.test.ts`, `aiConfigState.test.ts`, `aiTranslateHandlersWiring.test.ts`, `settingsOverlayAiConfig*Render.test.ts`. e2e không giả được provider AI.
- Nợ trong `deferred-work.md` (grep "12.11", "B21–B23", "Chủ: Story 12.11"): không có mục nào mang `Chủ` là story này.

## Tasks & Acceptance

**Execution:**

Phase Rust:
- [ ] `src-tauri/src/commands/aiconfig.rs`, `src/config/aiconfig.ts` -- thêm `ready: Option<bool>` vào `AiConfigGetWire` (endpoint + mô hình không rỗng và khoá có; `None` khi khoá không hỏi được), dùng chung vị từ với `resolve_call_config` (tách một hàm thuần) -- điều kiện chưa cấu hình do Rust quyết (AD-1), a:170.

Phase Webview (không phụ thuộc lựa chọn):
- [ ] `src/aiConfigState.ts`, `src/panels/AiTranslationPanel.vue` -- panel nạp cấu hình khi dựng (không đụng `reset` của mục Cài đặt); đầu panel: tiêu đề + "nhà cung cấp · mô hình" (chỉ khi `ready === true`) + link Xem prompt (`dispatch('ai.prompt_inspector.open')`), gỡ nút trùng ở `ai-inspector-actions` -- b:152, B20.
- [ ] `src/panels/AiTranslationPanel.vue`, `src/i18n/vi.json` -- trạng thái chưa cấu hình: tiêu đề + hai đoạn + lối mở Cài đặt (`dispatch('settings.open')` nếu lệnh đó có; xác minh id bằng grep), thay `panel.ai_translation.status`; phân biệt `ready === null` -- a:170.
- [ ] `src/panels/AiTranslationPanel.vue`, `src/aiTranslateBatchState.ts` -- hàng lô: số câu · chữ (bản dịch rút gọn một dòng, hoặc nguồn từ `editorSegments` khi chưa có) · token (`usage.total_tokens`, ẩn nếu `null`) · chữ trạng thái; giữ `data-ai-translate-batch-row-status` -- b:155, B21.
- [ ] `src/panels/AiTranslationPanel.vue`, `src/commands/index.ts`, `src/main.ts`, `src/aiTranslateBatchState.ts`, `scripts/check-commands.mjs` -- lệnh `ai.translate.batch_discard` (không phím) gọi `resetAiTranslateBatch`; nhãn "Huỷ lô" khi lô đang chạy (cùng `ai.translate.cancel`, đổi khoá nhãn); khối lỗi ba dòng "dừng ở câu k / 1–(k-1) còn nguyên / k nguyên nhân / (k+1)–n chưa gọi, chưa tính phí" dựng từ hàng (hàm thuần theo khoá + tham số) -- b:160, b:162, B23.
- [ ] `src/aiPromptInspectorState.ts`, `src/panels/AiTranslationPanel.vue`, `src/i18n/vi.json` -- nhớ số thuật ngữ của bản ghi trước (câu khác); dòng tóm tắt thêm "tăng/giảm N so với câu trước"; hàm `reset` xoá ô nhớ -- B19.

- [ ] `src/panels/editorPanelState.ts` (+ `src/main.ts`, `src/commands/index.ts`) -- `promoteAiTranslationBatchToEditor(ids)`: một lần flush, lặp `promoteAiTranslation`, gom mọi `needs_confirmation` vào một câu hỏi duy nhất (mở rộng `editorPendingPromote` thành danh sách), đồng ý thì lặp `force = true`; lệnh `ai.translate.promote_batch` (`Mod+Shift+Alt+Enter`, xác minh chord hợp lệ và chưa bị chiếm bằng grep trước) -- B22.
- [ ] `src/panels/AiTranslationPanel.vue` -- nút "Đưa N câu đã xong sang Bản dịch" (N = số hàng `done`, khoá khi 0 hoặc lô đang chạy) cạnh Bỏ lô và Thử lại -- b:162, B22.

Phase Tests:
- [ ] `tests/frontend/` -- tệp mới `aiBatchPanelRows.test.ts` (hàng có chữ + token + trạng thái; lô 20 câu 1 lỗi bằng `invoke` giả: 5 xong, hàng 6 lỗi, 7–20 chưa gọi, khối lỗi đúng; Bỏ lô huỷ lượt đang bay và xoá mọi thứ; Huỷ lô giữ hàng xong; chưa cấu hình hiện lời mời không `role="alert"`; `ready = null` khác `false`; đầu panel "nhà cung cấp · mô hình"; delta Glossary); sửa `aiTranslateBatch.test.ts`, `aiTranslateBatchResetWiring.test.ts`, `aiPromptInspector.test.ts` theo template mới; ca đưa cả lô gồm đường `needs_confirmation` một lần hỏi và lỗi một câu.
- [ ] `src-tauri/tests/ai_translate_contract.rs` (+ `ipc_contract.rs`/`config_invariants.rs` cho `ready`) -- ca `ready` ba giá trị.
- [ ] `_bmad-output/initiative-auratranslate/deferred-work.md` -- mục `Chủ: Epic 12`: Ice thử trong app thật một lô nhà cung cấp thật có lỗi thật, "Bỏ lô" giữa chừng, "Đưa cả lô" với bản nháp chưa ký, trạng thái chưa cấu hình trên bản đóng gói macOS (e2e không giả được provider AI). Thêm một mục thứ hai `Chủ: Story 12.18`: lượt dịch đòi khoá cả với Ollama/LM Studio nên panel không ghi "chạy không cần khoá" như mockup; bỏ yêu cầu khoá cho nhà cung cấp cục bộ thì sửa lại câu đó.

**Acceptance Criteria:**
- Given một lô 20 câu chạy dở có lỗi ở câu 6, when nhìn panel, then mỗi hàng ghi số câu, chữ, chữ trạng thái và token (hàng xong), khối lỗi nêu câu 6, nguyên nhân và 14 câu chưa gọi, và Bản dịch không đổi.
- Given lô đang chạy hoặc đã lỗi, when bấm Bỏ lô, then lượt còn lại dừng, panel không còn hàng/tiến độ/tổng token, và không câu nào bị gọi thêm.
- Given cấu hình AI trống, when mở panel, then panel hiện trạng thái chưa cấu hình của mockup (lời mời, không cảnh báo) và mọi nút khác còn dùng được.
- Given cấu hình đủ, when nhìn đầu panel, then thấy "nhà cung cấp · mô hình" và link Xem prompt mở lớp phủ hiện có.
- Given 5 hàng xong trong đó 2 câu đích có bản nháp chưa ký, when bấm "Đưa 5 câu đã xong sang Bản dịch", then chỉ một câu hỏi hiện ra, và mọi câu được ghi mang `translation_origin = 'other'` cùng mốc so mới.

## Implementation Notes

- B22 theo (a): `promoteAiTranslationBatchToEditor(items)` nhận `{segmentId, text}[]` để `editorPanelState` không import trạng thái lô; câu hỏi của lô là ref riêng `editorPendingPromoteBatch`, `ai.translate.confirm_promote`/`cancel_promote` xử lý lô trước khi tới câu đơn.
- `ready` nằm ở `aiconfig_contract.rs` (nơi đã có mock keychain và khoá test), không ở `ai_translate_contract.rs`; vị từ chung là `call_ready`/`endpoint_and_model_set` trong `aitranslate.rs`.
- Lượt dịch đòi khoá cả với Ollama/LM Studio nên đoạn chưa cấu hình nói thế, lệch mockup (nợ `Chủ: Story 12.18`).
- `PanelFrame` của panel tắt dòng `panel.ai_translation.status` vì câu đó nói "chưa có nhà cung cấp" ở mọi trạng thái; khoá vẫn dùng ở dải Proofreader.
- Phím `Mod+Alt+Shift+Enter` theo thứ tự cố định Mod → Meta → Ctrl → Alt → Shift của `keys.ts`.
- Chưa thử trong app thật hay e2e: webview đứng ở `tauri://localhost` trên máy này (nợ `Chủ: Epic 12`).

## Plan Change Log

- 2026-10-15 sau Phase Rust: Ice chọn sửa câu chữ trạng thái chưa cấu hình thay vì cho phép chạy không khoá (Decisions, hàng Chưa cấu hình, task nợ).
- 2026-10-15 sau Phase Webview: bước đồng ý ghi đè của lô flush lại trước lượt `force = true`, như `confirmPendingPromote` đơn câu; "một lần flush" là cho lượt đầu của cả lô, không bỏ flush trước lượt ghi đè (người dùng có thể đã gõ vào câu đang chờ hỏi).

## Review Triage Log

- quick · Bỏ lô để lại câu hỏi đưa cả lô đang chờ (đồng ý vẫn ghi) · medium · patch: handler Bỏ lô gọi `cancelPendingPromote()` khi lô đang chờ hỏi; ca mới đỏ khi gỡ dòng đó, xanh khi trả lại.
- quick · Bỏ lô để lại dòng báo "Đã đưa N câu" · low · reject: dòng đó nói đúng những câu đã ghi; xoá cần thêm export mới.
- quick · khối doc của `loadAiConfigSection` bị đẩy lên trên `type AiConfigGetResult` · low · patch: dời các hàm mới lên trên khối doc.
- quick · đầu tệp test ghi slug story · low · patch: bỏ slug (AGENTS.md cấm id story trong mã).
- quick · tên `describe` có "matrix:" và JSDoc chỉ nhắc chữ ký · false: tên test không phải comment; JSDoc nêu điều chữ ký không nói (chỉ hàng `done` có chữ, giữ thứ tự lô).
- quick · `intact` đếm cả hàng `skipped` là "còn nguyên" · low · reject: lô dừng ở lỗi đầu nên dải trước lỗi chỉ gồm done/skipped, hàng skipped không mất gì.
- quick · đưa cả lô không chặn khi lượt đơn đang chạy · false: đưa sang là lượt ghi từng câu qua `promote_ai_translation` có `needs_confirmation`, không phải lượt chạy; đưa đơn câu cũng không chặn theo lô.
- quick · phím ghi `Mod+Alt+Shift` khác plan `Mod+Shift+Alt` · false: `keys.ts` quy định thứ tự Mod → Meta → Ctrl → Alt → Shift; `check:commands` xanh.
- quick · test `ready` thiếu ca endpoint/mô hình rỗng + keychain không hỏi được · low · patch: thêm một assert `Some(false)`.
- quick · không test ghim khoá `ready` trong hình dạng wire · low · reject: webview đọc `ready` qua kiểu TS và mock; thêm test hình dạng ngoài plan.
- quick · bốn đối chứng đỏ không thấy trong diff · false: đã gỡ thật rồi trả lại, kết quả ghi trong tệp handoff.
- quick · `:show-status="false"` ẩn dòng trạng thái ở mọi trạng thái · false: chữ của khoá đó là câu "chưa có nhà cung cấp AI", sai khi đã cấu hình; task của plan thay nó.

## Design Notes

B22 theo (a): `promote_ai_translation` đã là một giao dịch/câu và `needs_confirmation` không ghi gì, nên webview gom các câu chờ hỏi thành một câu hỏi rồi gọi lại với `force = true`; AD-47/AD-49 iii giữ nguyên vì không có đường ghi `target_text` thứ hai.

`ready` ở Rust thay cho vị từ trong TS: `resolve_call_config` trả `None` khi endpoint/mô hình rỗng, `read_api_key` trả `None` khi chưa có khoá; hai điều kiện đó đã quyết "chưa cấu hình" cho lượt dịch nên panel đọc cùng kết quả, không chép logic (AD-1) và không lệch khi Rust đổi vị từ.

Khối lỗi ba dòng chỉ dựng từ hàng của lô (`error` đầu tiên là câu dừng, các hàng `done` trước nó, các hàng `pending` sau nó); vì lô dừng ở lỗi đầu (`run_batch_call`), phân hoạch này luôn đủ, không cần Rust trả thêm.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test ai_translate_contract --test ipc_contract --test config_invariants` -- xanh.
- `npx vitest run tests/frontend/aiBatchPanelRows.test.ts tests/frontend/aiTranslateBatch.test.ts tests/frontend/aiTranslateBatchResetWiring.test.ts tests/frontend/aiTranslatePromoteConfirmFlow.test.ts tests/frontend/aiPromptInspector.test.ts tests/frontend/aiConfigState.test.ts` -- xanh; `npm run test:story 12-11 -- --list` để xem tập chọn.
- `npm run check:commands` và `npm run check:panel-refs` và `npm run check:i18n` -- xanh.
- Đối chứng đỏ (gỡ THẬT, giữ chữ ký, chạy đúng target của guard): gỡ lượt gọi lại `force = true` sau khi đồng ý khỏi `promoteAiTranslationBatchToEditor` → ca hai câu nháp được ghi của vitest đưa cả lô đỏ; thay vị từ `ready` bằng `Some(true)` → ca ba giá trị đỏ; xoá lời gọi `resetAiTranslateBatch` khỏi handler Bỏ lô → ca Bỏ lô của `aiBatchPanelRows.test.ts` đỏ; gỡ lệnh `cancelWhenGenerating` khỏi đường Bỏ lô → ca "không câu nào bị gọi thêm" đỏ.
- Không chạy toàn bộ bộ test trừ khi một lỗi trông phụ thuộc thứ tự (không đụng wiring chung).

**Manual checks (if no CLI):**
- Trong `npm run tauri dev` với Ollama cục bộ: chạy lô 20 câu, ngắt Ollama giữa chừng; đối chiếu panel với mục 1 và 2 của `ai-batch-translate.html`; Bỏ lô; xoá endpoint trong Cài đặt rồi mở lại panel (đối chiếu `de-xuat-ai-chua-cau-hinh-co-tac-pham.png`). Ghi vào nợ `Chủ: Epic 12` nếu e2e/app không mount được.
