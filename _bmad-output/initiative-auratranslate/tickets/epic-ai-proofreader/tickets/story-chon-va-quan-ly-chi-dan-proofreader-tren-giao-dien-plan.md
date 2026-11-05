---
title: 'Chọn và quản lý chỉ dẫn Proofreader trên giao diện'
type: 'feature'
ticket: '15'
created: '2026-11-05'
status: 'done'
baseline_revision: '601193defffa4cbd5b18d75f0c25121e4ed4d2e3'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-68 mục 2, 3, 4 (chỉ đọc mục AD-68)'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 9.13–9.14 cho Rust đủ đường để lượt quét dùng chỉ dẫn `proofread`, nhưng thư viện prompt cứng loại `translate` (`LIBRARY_KIND`), không ô nào chọn bộ ở tầng Global hay Tác phẩm, và xoá bộ đang chọn không báo hậu quả — người dùng chưa chạm được FR160.

**Approach:** Thư viện prompt có hai thẻ theo loại; thẻ `proofread` soạn chữ thuần (không marker, không cảnh báo), có ô chọn Global và ô ba trạng thái của Tác phẩm do Rust đọc và tự báo tên đã mất; xoá bộ đang được chọn hiện hậu quả Rust tính (AD-68 mục 2–4).

## Boundaries & Constraints

**Always:**
- Thẻ "Bộ dịch" giữ nguyên hành vi hiện có, kể cả lựa chọn bộ dịch của Tác phẩm; đổi thẻ không bao giờ xoá hay đổi lựa chọn bộ dịch.
- Ô Global chỉ liệt kê bộ `proofread` tầng Global, cộng "Không chọn" (ghi `NULL`). Ô Tác phẩm: "Theo Global", "Không dùng chỉ dẫn", rồi mỗi tên `proofread` phân giải được qua hai tầng; ẩn hoặc khoá khi không có Tác phẩm mở.
- Một lệnh Rust mới đọc cả hai lựa chọn và trả cờ `missing` tính bằng đúng phép tìm hai tầng của `resolve_proofread_instruction`; webview không tự so tên. Tên mất hiện kèm dấu thiếu, không hiện ô trống.
- Thẻ `proofread`: Rust không trả cảnh báo marker và không trả danh sách biến cho loại này (sửa ở nguồn, không ẩn bằng `v-if`); webview không hiện chip chèn biến và không tô marker trong bản soi.
- Xoá bộ `proofread`: lượt bấm đầu gọi `prompt_set_proofread_delete_consequence`; `global_takes_over`/`set_missing` hiện câu hậu quả tương ứng thay câu nhắc chung, `none` giữ câu nhắc chung. Xoá không tự bỏ chọn.
- Sau đổi tên hay xoá, ô chọn đọc lại từ Rust, nên hiện tên mới hoặc tên đã mất kèm dấu thiếu.
- Nhập tệp đi theo loại trong tệp; xong thì thư viện hiện thẻ của loại đó.
- Ba miễn trừ `prompt_set_proofread_*` rời `NO_FRONTEND_CALLER`; mục nợ `Chủ: Story 9.15` đóng bằng chữ.
- Chuỗi mới nằm trong `src/i18n/vi.json`, qua `check:i18n`; màu và kích thước qua token (`check:tokens`).
- Quyết định (Ice, 2026-11-05): ô chọn Global và ô ba trạng thái nằm trong thẻ chỉ dẫn của `PromptLibraryOverlay`, không trong Cài đặt. Giữ kế hoạch nguyên một story dù vượt trần token.

**Never:**
- Không đổi logic phân giải, đổi tên, hậu quả xoá hay lắp prompt của 9.14; không đổi đường dịch.
- Không đánh dấu lượt dùng thật của Ice là đã chạy khi Ice chưa chạy.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Chưa chọn | Global không khoá, Work theo Global | ô Global "Không chọn"; ô Work "Theo Global" | — |
| Chọn Global | chọn bộ Global "B" | ghi khoá; lượt quét ở mọi Tác phẩm theo Global dùng "B" | lỗi ghi hiện trong thẻ |
| Work tắt | chọn "Không dùng chỉ dẫn" | ghi `off=1`; lượt quét như trước AD-68 | — |
| Đổi tên | bộ đang chọn "A"→"C" | ô chọn hiện "C" | — |
| Xoá có bộ Global thay | xoá Work "A" che Global "A", Work chọn "A" | câu "bộ Global cùng tên sẽ thay vào" | — |
| Xoá bộ đang chọn | xoá Global "B" đang chọn | câu "lượt quét sau sẽ báo thiếu bộ"; sau xoá ô hiện "B" kèm dấu thiếu | lượt quét báo `instruction_missing` nêu "B" |
| Tên mất từ `.atproj` | Work chọn "X", máy thiếu "X" | ô Work hiện "X" kèm dấu thiếu | — |
| Marker trong thân | bộ `proofread` chứa `{{source_segment}}` | không cảnh báo, không tô | — |
| Nhập tệp | tệp `kind: proofread` khi đang ở thẻ dịch | bộ vào loại `proofread`, thẻ chuyển sang chỉ dẫn | va chạm tên chỉ trong loại |

</frozen-after-approval>

## Code Map

Số dòng ở `601193de`.

- `src/promptSetState.ts` -- `LIBRARY_KIND` cứng :50, truyền ở list :157, create :263, template :307, rename :338, updateBody :361, delete :385, export :412/:426. ⚠️ `reloadPromptSets` :179 xoá im lặng tên đã chọn khi không có trong danh sách: nạp danh sách `proofread` vào cùng state sẽ xoá lựa chọn bộ dịch. `setSelectedPromptSetName` :217, `restorePromptSetForWork` :241 chỉ của bộ dịch.
- `src/PromptLibraryOverlay.vue` -- overlay (mount `App.vue:384`); dùng bộ chọn :584-587/:885, huy hiệu `in_use_badge` :687/:708, dòng `in_use_line` :277-281, lỗi chọn :656; chip biến :847, bản soi `splitPromptBody` :857, cảnh báo :871-876; xoá hai lượt bấm `onDeleteSubmit` :518-530, gợi ý :911, Esc :595; nút nhập :756.
- `src/config/promptset.ts` -- wrapper theo loại có sẵn; thêm wrapper cho `prompt_set_proofread_choice_set_global {name}` (wire `commands/promptset.rs:824`), `…_set_work {name, off}` (:835, không Work ⇒ `no_work_open`), `…_delete_consequence {tier, id}` (:852, trả `none|global_takes_over|set_missing`) và lệnh đọc mới. Xem trước nhập có `set_kind` :368.
- `src-tauri/src/core/promptset/store.rs` -- `read_proofread_work_choice` :408 (`ProofreadWorkChoice` :402), `proofread_choice_in_effect` :473 (riêng), `resolve_proofread_instruction` :492 (lỗi `ProofreadSetMissing` :502); gọi `scan_markers` ở create :213, update_body :328.
- `src-tauri/src/core/scope/store.rs` -- `GlobalConfig::proofread_prompt_set()` :365, `load_global_config` :429.
- `src-tauri/src/core/promptset/vars.rs` -- `scan_markers` :115/:156 không biết loại. `commands/promptset.rs:178` trả `variables = PromptVariable::ALL` cho mọi loại.
- `src-tauri/src/lib.rs` -- `generate_handler!` (đăng ký lệnh đọc mới ⇒ một lượt đủ ở cuối).
- `src-tauri/tests/ipc_argument_contract.rs:631-638` -- `NO_FRONTEND_CALLER`; `tests/config_invariants.rs` census và `tests/ipc_contract.rs` nhận lệnh mới.
- `src/i18n/vi.json` -- 67 khoá `prompt.library.*`; `err.ai_proofread.instruction_missing` :132.
- Test frontend: `tests/frontend/promptLibraryOverlayRender.test.ts`, `promptSetState.test.ts`. e2e khuôn: `e2e/specs/story-12-14-prompt-library.e2e.mjs`.
- Mockup `ux-auratranslate/mockups/prompt-library.html` chỉ có bộ dịch, không thẻ.

## Tasks & Acceptance

Bốn pha, mỗi pha một agent mới, bàn giao qua `epic-ai-proofreader/handoff-9-15-phases/`.

**Execution:**
- [x] `src-tauri/src/core/promptset/store.rs`, `src-tauri/src/commands/promptset.rs`, `src-tauri/src/lib.rs` -- lệnh `prompt_set_proofread_choice_get` trả `{global: {name, missing}|null, work: {mode: follow_global|off|named, name, missing}|null}`, `missing` qua cùng phép tìm của `resolve_proofread_instruction`; `proofread` không có cảnh báo marker và `variables` rỗng -- AD-68 mục 2, 3.
- [x] `src/config/promptset.ts`, `src/promptSetState.ts` (hoặc module state riêng cho chỉ dẫn) -- loại theo thẻ; wrapper bốn lệnh; lựa chọn bộ dịch không bị chạm khi ở thẻ chỉ dẫn.
- [x] `src/PromptLibraryOverlay.vue`, `src/i18n/vi.json` -- hai thẻ; ô Global và ô Tác phẩm; dấu thiếu; câu hậu quả xoá; chuyển thẻ sau nhập.
- [x] `src-tauri/tests/ipc_argument_contract.rs`, test Rust lệnh đọc, `tests/frontend/`, một spec e2e thư viện chỉ dẫn -- ca theo I/O; gỡ ba miễn trừ.
- [x] `deferred-work.md` -- đóng mục nợ `Chủ: Story 9.15`; thêm mục `Chủ: Epic 9` cho lượt dùng thật của Ice nếu chưa chạy.

**Acceptance Criteria:**
- Given đang ở thẻ chỉ dẫn và Tác phẩm có bộ dịch đã chọn, when đổi qua lại hai thẻ và tạo, sửa, xoá bộ chỉ dẫn, then lựa chọn bộ dịch không đổi.
- Given gỡ thật nhánh tính `missing` trong lệnh đọc (trả luôn `false`), when chạy target test của lệnh đọc, then ca tên mất đỏ đúng lý do.
- Given gỡ thật lời gọi hậu quả xoá ở webview, when chạy tệp vitest của overlay, then ca câu hậu quả đỏ.
- Given `NO_FRONTEND_CALLER` không còn ba lệnh `prompt_set_proofread_*`, when chạy `cargo test --test ipc_argument_contract`, then xanh.
- Given Ice viết chỉ dẫn "chỉ so bản dịch với bản gốc của chính câu đó", chọn ở Global, quét lại Chương có seg 162/163, then kết quả ghi vào Implementation Notes, hoặc thành mục nợ `Chủ: Epic 9`.

## Implementation Notes

- Lệnh đọc `prompt_set_proofread_choice_get` (`core/promptset/store.rs::read_proofread_choices`) tính `missing` bằng `resolve_two_tiers`, cùng phép tìm với `resolve_proofread_instruction`; `work` là `null` khi không có Tác phẩm mở, không lỗi. `missing` của Global cũng coi một bộ tầng Work cùng tên là phân giải được.
- Loại `proofread` không có cảnh báo marker và `variables` rỗng ngay ở Rust (create, update_body, list, xem trước nhập); webview thêm không tô marker và không chip cho thẻ này.
- State chỉ dẫn tách riêng khỏi state bộ dịch (`loadProofreadInstructions` …) vì `reloadPromptSets` xoá im lặng tên đã chọn không có trong danh sách; lựa chọn bộ dịch không bị chạm khi đổi thẻ (ca vitest 1).
- Ô Tác phẩm liệt kê mọi tên phân giải được qua hai tầng; ô Global chỉ tên tầng Global (kể cả bộ bị che).
- Ba miễn trừ `prompt_set_proofread_*` đã rời `NO_FRONTEND_CALLER`; bẫy của bộ quét: không truyền hằng `CMD_*` làm đối số thứ hai không phải đối tượng cho hàm trợ giúp.
- Đối chứng AC2: thay `!sets.contains_key(&name)` bằng `false` làm đỏ ba ca (`…flags_a_chosen_name…`, `…agrees_with_the_scan_after_a_delete_and_a_rename`, `…without_an_open_work…`) đúng ở assert `missing`; đã khôi phục. Đối chứng AC3 (thay lời gọi hậu quả xoá bằng `'none'`) làm đỏ 3 ca trong `promptLibraryProofreadCard.test.ts`.
- e2e `story-9-15-proofread-instruction-library.e2e.mjs` viết theo khuôn 12-14 nhưng CHƯA chạy trong lượt này (cần ngoài sandbox, app thật); phải chạy trước khi push.
- Chưa có mockup cho thẻ chỉ dẫn và thẻ chưa được nhìn trong cửa sổ thật; lượt dùng thật của Ice (AC cuối) chưa chạy ⇒ mục nợ `Chủ: Epic 9` trong `deferred-work.md`; mục nợ `Chủ: Story 9.15` đã đóng bằng chữ.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick, 2026-11-05): high 0 · medium 1 · low 2 · false 0 · maybe-false 0, cộng 2 phát hiện bị loại.

- medium · patch — ô Global trống khi Rust trả `{name, missing: false}` cho một tên chỉ còn là bộ tầng Work của Tác phẩm đang mở (bộ Global bị xoá hoặc đổi tên): `globalChoiceNames` chỉ có tên tầng Global, `:value` không khớp option nào; luôn dựng option cho tên đang chọn, thêm ca vitest. Gộp phát hiện "test không phủ ca này".
- low · patch — lời gọi hậu quả xoá trượt thì `proofreadDeleteConsequence` trả `'none'` và lượt bấm sau xoá mà không có câu hậu quả; lượt trượt không bật `deletePending`, thêm ca vitest.
- low · rejected — cờ `loading`/`loadError` dùng chung hai loại, lượt nạp `proofread` về muộn sau khi đổi sang thẻ dịch có thể ghi vào thẻ dịch: hai lệnh đọc là IPC cục bộ vài ms, người dùng khó gặp; sửa cần thêm điều kiện chéo giữa hai bộ đếm.
- rejected — ô Execution chưa đánh dấu và Triage Log trống: sửa là sửa plan; ô đã đánh dấu khi ghi log này.

## Design Notes

Ô chọn nằm trong thư viện vì ô chọn bộ dịch nằm ở đó (ticket: theo khuôn ô chọn hiện có); không có mockup cho thẻ chỉ dẫn. Cờ `missing` do Rust tính để webview và lượt quét không thể bất đồng (AD-68 Prevents 3). Ô Global không liệt kê bộ tầng Work vì tên đó không có ở Tác phẩm khác.

## Verification

**Commands:**
- `npm run build` rồi `cd src-tauri && cargo test --test prompt_set_contract --test ipc_argument_contract --test ipc_contract --test config_invariants` -- xanh.
- `npx vitest run tests/frontend/promptLibraryOverlayRender.test.ts tests/frontend/promptSetState.test.ts` -- xanh, mã thoát 0, không dòng Errors.
- `npm run check:i18n && npm run check:tokens` -- xanh.
- Một lượt đủ cuối pha Tests (đổi `lib.rs`); spec e2e mới chạy ngoài sandbox trước khi push.

**Manual checks:**
- Ice: lượt dùng thật như AC cuối.
