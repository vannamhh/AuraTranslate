---
title: 'Story 4.17 — Mục Glossary được chèn mang theo ghi chú, ô ghi chú nói rõ được gửi cho AI'
type: 'feature'
ticket: 17
created: '2026-11-03'
status: 'built'
baseline_revision: '82a5ddc8894f65224c3435679f1fc74eaa201f8c'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Glossary chỉ gửi `nguồn → bản dịch`, nên mô hình biết tên mà không biết xưng hô, giới tính, cách gọi; các Chương trôi khỏi nhau. Trường `note` hôm nay chỉ người dùng đọc, và người dùng không được báo rằng nó sẽ bị gửi đi.

**Approach:** Thi hành AD-66. Truy vấn chèn của `core/glossary` trả thêm `note`, `InjectedGlossaryTerm` mang `note`, `render_glossary_pairs` in `nguồn → bản dịch — ghi chú`. Đây là chỗ in duy nhất cho cả đường một câu lẫn đường lô. Hai ô nhập ghi chú (thêm nhanh, quản lý Glossary) hiện một dòng nhắc từ một khoá i18n mới.

## Boundaries & Constraints

**Always:**
- Ghi chú đi cùng mục thắng phân giải hai tầng (AD-18), không bao giờ ghép từ hai tầng.
- `note` rỗng sau khi trim ⇒ dòng giữ dạng `nguồn → bản dịch`, không có ` — `.
- Mọi dãy khoảng trắng (kể cả xuống dòng, `\r\n`, tab) trong ghi chú gộp thành một dấu cách, cắt hai đầu. Ghi chú gửi trọn, không có trần độ dài.
- Đường lô vẫn bỏ trùng theo `source_term` (AD-60 mục 3); mục giữ lại mang ghi chú của chính nó.
- Câu nhắc dùng khoá `glossary.note.sent_to_ai` = "Ghi chú của mục đã chốt được gửi cho AI cùng thuật ngữ mỗi lần dịch." Hai ô dùng chung khoá, hiện được bằng mắt, nối vào ô qua `aria-describedby`.

**Never:**
- Đổi `InjectedGlossaryTermWire`/IPC: AD-66 chỉ buộc ba chỗ, và FR71 đã được đáp ứng vì ghi chú nằm trong chữ prompt đã ghi.
- Đổi thân prompt mặc định, bộ ba biến prompt, Proofreader, dịch nhanh (AD-64), cách quét và chọn mục (AD-36), `SuppressedGlossaryTerm`/`PendingGlossaryTerm`.
- Thêm trần độ dài hay một trường "ghi chú cho AI" riêng (phương án bị loại của AD-66).

## I/O & Edge-Case Matrix

| Ca (AD-66 mục 7) | Trạng thái | Prompt |
|---|---|---|
| (a) | Đã chốt, note "Lão long, xưng ta", có trong câu; dịch một câu và dịch lô | có dòng `dragon → Rong A — Lão long, xưng ta` |
| (b) | Chờ chốt có note | không chứa chữ nào của note |
| (c) | Tầng Tác phẩm note rỗng ghi đè tầng Global có note | `dragon → <bản tầng Tác phẩm>`, không có ` — ` |
| (d) | note `"a\n  b\r\nc"` | in `a b c` trên đúng một dòng |
| (e) | Một lượt lô, hai câu cùng chứa thuật ngữ có note | note xuất hiện đúng một lần |
| (f) | Thân không có `{{glossary_terms}}` | không có note; sổ chèn `NotAsked` |
| — | Mục không note và mục có note trong cùng câu | mỗi mục một dòng, theo thứ tự câu |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/glossary/store.rs` -- `GlossaryInjectionTerm` :1833 (thêm `note: String`); `confirmed_terms_in_snapshot` :1925, khối `Some(GlossaryInjectionTerm{..})` :1946 lấy `entry.note.clone()`. Hai đường cùng qua đây: `confirmed_terms_for_injection` :1913 (một câu) và `GlossarySnapshot::confirmed_terms` :1666 (lô). `GlossaryEntry::note: String` ở `entry.rs:218`.
- `src-tauri/src/core/ai/rag.rs` -- `InjectedGlossaryTerm` :69 (thêm `note`); `asked_status` :276 map thêm `note`; `render_glossary_pairs` :417, chỗ in duy nhất, gọi từ `assemble_prompt` :749 và `assemble_call_prompt` :888. Doc comment dài ở :399-416 có ngày và lịch sử: rút về một hai dòng theo luật comment.
- `src-tauri/src/commands/aiprompt.rs` -- `From<InjectedGlossaryTerm> for InjectedGlossaryTermWire` :133 giữ nguyên năm trường. Đường một câu `assemble_and_record_prompt_with_tm` :520 và đường lô `prepare_operation_prompt` :748 + `assemble_call` :800 không cần sửa.
- `src-tauri/tests/ai_rag_contract.rs` -- 8 chỗ dựng `InjectedGlossaryTerm` (:101, :864, :871, :954, :1002, :1009, :1239, :1246) cần thêm `note`. Khuôn có store: `a_confirmed_global_term_is_injected_and_listed_in_the_ledger` :70, ghi đè tầng `work_tier_overrides_global_for_the_same_term` :208, `add_manual_term(.., note, ..)`.
- `src-tauri/tests/ai_batch_wire.rs` -- `Harness::with_set(tag, text, conns, echo_responder(), "{{glossary_terms}}\n{{source_segment}}")` :301, `h.prompts()` :418 trả prompt thật gửi tới máy chủ giả, `run_single(&h, id)` :547 (một câu), `h.run(h.selection(..))` (lô), `insert_manual_entry(&open.store, term, Some(tr), note, Category::Other)`; khuôn ở :1206 và :1534. Mỗi ca cần `lock_keychain("sk-batch-<tag>")` riêng.
- `src/GlossaryQuickAdd.vue` :152-160 ô `quickAddNote`; `src/GlossaryManageOverlay.vue` :686-698 ô `manageEditNote` (chỉ có khi đang sửa), form `gm-edit-form` :716.
- `src/i18n/vi.json` -- khoá ghi chú hiện có :1690, :1983. Test: `tests/frontend/glossaryQuickAddStrip.test.ts` (mount `GlossaryQuickAdd`), `tests/frontend/glossaryManage.test.ts`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/glossary/store.rs` -- thêm `note` vào `GlossaryInjectionTerm`, điền từ mục thắng -- AD-66 chỗ 1.
- [ ] `src-tauri/src/core/ai/rag.rs` -- thêm `note` vào `InjectedGlossaryTerm`, map ở `asked_status`, `render_glossary_pairs` chuẩn hoá khoảng trắng rồi nối ` — note` khi khác rỗng -- AD-66 chỗ 2, Rule 2.
- [ ] `src-tauri/tests/ai_rag_contract.rs` -- thêm `note` vào các chỗ dựng; ca mới (c), (d), (f) và ca hai mục trộn, có store thật -- canh phân giải tầng và cách in.
- [ ] `src-tauri/tests/ai_batch_wire.rs` -- ca mới (a) qua cả `run_single` lẫn `h.run`, (b) ở cả hai đường, (e) một lượt lô hai câu -- canh đúng dây lắp prompt của hai đường.
- [ ] `src/i18n/vi.json` + `src/GlossaryQuickAdd.vue` + `src/GlossaryManageOverlay.vue` -- thêm khoá, hiện dòng nhắc cạnh mỗi ô ghi chú với `id` riêng và `aria-describedby` -- AD-66 Rule 4.
- [ ] `tests/frontend/glossaryQuickAddStrip.test.ts`, `tests/frontend/glossaryManage.test.ts` -- ô ghi chú có `aria-describedby` trỏ tới phần tử chứa đúng chữ của khoá.

**Acceptance Criteria:**
- Given ca (a), when gỡ phần nối ghi chú trong `render_glossary_pairs` (giữ chữ ký) và chạy lại `cargo test --test ai_batch_wire`, then cả hai nửa của ca (a) đỏ vì thiếu ` — Lão long, xưng ta`; trả lại thì xanh.
- Given ca (d), when bỏ bước gộp khoảng trắng, then ca (d) đỏ.
- Given người dùng mở thêm nhanh Glossary, hoặc sửa một hàng trong quản lý Glossary, when nhìn ô ghi chú, then thấy câu "Ghi chú của mục đã chốt được gửi cho AI cùng thuật ngữ mỗi lần dịch."; when gỡ dòng nhắc khỏi template, then test frontend tương ứng đỏ.

## Implementation Notes

- `note` đi dạng `String` thô qua `GlossaryInjectionTerm` → `InjectedGlossaryTerm`; chuẩn hoá khoảng trắng chỉ ở `render_glossary_pairs` (`split_whitespace().join(" ")`), nên không có chỗ in thứ hai.
- Ca (a) gom kết quả hai đường thành `[single, batch]` trước khi assert: bản đầu dừng ở prompt đầu, nên phép gỡ chỉ chứng minh được nửa một câu đỏ. Gỡ phần nối ghi chú ⇒ `[false, false]`; ca (e) cũng đỏ (0 thay vì 1), ca (b) xanh đúng.
- Bỏ bước gộp khoảng trắng (chỉ `trim()`) ⇒ ca (d) đỏ. Gỡ dòng nhắc khỏi hai template ⇒ hai ca vitest mới đỏ.
- Với từ hai cặp trở lên, `{{glossary_terms}}` cùng dòng với chữ khác vẫn sang dòng riêng (`Terms: \n…`), hành vi có sẵn; ca (d) khẳng định nguyên văn.
- Ô ghi chú ở quản lý Glossary chỉ có khi đang sửa hàng, nên dòng nhắc cũng chỉ hiện lúc đó.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 0 high · 1 medium · 1 low · 4 false · 0 maybe-false.

- medium · patch — `GlossaryQuickAdd.vue`: dòng nhắc nằm trong `<label>` bọc ô nên thành một phần tên truy cập, trình đọc màn hình đọc hai lần; vá bằng `aria-label` "Ghi chú" cho ô.
- low · patch — ca ghi chú dùng chuỗi ASCII "Lao long, xung ta" thay vì "Lão long, xưng ta" của ma trận; đổi sang có dấu.
- false — "đối chứng đỏ không có bằng chứng": cả ba phép gỡ đã chạy trong phiên này, đầu ra đỏ được đọc (`[false, false]`, ca (d), 2 ca vitest); đánh dấu ô việc là sửa plan, bị loại.
- false — ca (c) không đọc tầng trong sổ chèn: "rong tac pham" chỉ có ở tầng Tác phẩm nên khớp nguyên văn đã chứng minh mục tầng Tác phẩm thắng; tầng trong sổ đã có ca `work_tier_overrides_global_for_the_same_term`.
- false — bộ chọn `.gqa-field:nth-of-type(3)` "gãy im lặng": đổi thứ tự thì `aria-describedby` là `undefined` và `toBeTruthy` đỏ to, không im lặng.
- false — quản lý Glossary còn đường thêm có ô ghi chú: grep chỉ thấy một `v-model` ghi chú (`manageEditNote`).

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test ai_rag_contract` và `cargo test --test ai_batch_wire` (trong `src-tauri/`) -- expected: xanh, kể cả các ca mới.
- `npx vitest run tests/frontend/glossaryQuickAddStrip.test.ts tests/frontend/glossaryManage.test.ts` -- expected: xanh.
- `npm run check:i18n` và `npm run check:tokens` -- expected: xanh.
- Đối chứng đỏ theo hai AC đầu: phép gỡ thật, chỉ chạy lại đúng target.

**Manual checks:**
- Thấy dòng nhắc trong app thật là việc chỉ người làm được ⇒ một mục nợ `Chủ: Epic 4` trong `deferred-work.md`, cho lượt dùng thật trước khi Epic 4 `done`.
