---
title: 'Lượt quét dùng chỉ dẫn đã chọn'
type: 'feature'
ticket: '14'
created: '2026-11-05'
status: 'done'
baseline_revision: '41cb866e563f46f1e90938bb5fb724478e344bd7'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md §AD-68 mục 3, 4, 5, 6, 7, 11 (chỉ đọc mục AD-68)'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Prompt quét của Proofreader cố định trong `rag.rs`; người dùng không dặn được model "chỉ so với bản gốc trong cùng khối" (FR160). 9.13 đã cho bộ `proofread` chỗ ở trong thư viện và di trú sẵn các cột lựa chọn và vết, nhưng chưa ai đọc.

**Approach:** Rust tự phân giải bộ chỉ dẫn đang dùng một lần đầu mỗi lượt quét, lắp nó giữa câu giới thiệu khối và hợp đồng trả lời, ghi vết lên `proof_scan`; thêm hai lệnh ghi lựa chọn, kéo lựa chọn theo khi đổi tên, và tính hậu quả của xoá cho 9.15 (AD-68 mục 3–7).

## Boundaries & Constraints

**Always:**
- Thứ tự phân giải đúng AD-68 mục 3; tên tìm trong loại `proofread` qua hai tầng (`resolve_two_tiers`). Thân rỗng sau khi cắt khoảng trắng = không có chỉ dẫn.
- Tên không phân giải được ⇒ lỗi trước mọi lời gọi mạng, `MessageKey` mới tham số `name` và `tier` (tầng đặt lựa chọn); không rơi về prompt gốc.
- Không có chỉ dẫn ⇒ prompt trùng từng byte bản ghi vàng lấy từ `assemble_proofread_batch_prompt` ở baseline, TRƯỚC mọi sửa đổi, commit cùng story.
- Chỉ dẫn đi qua bộ chèn U+200B với tập dấu SEGMENT, SOURCE, TEXT; thân không mở rộng marker.
- `proof_scan` ghi tên và tầng do Rust phân giải; webview không gửi giá trị này.
- Lệnh ghi lựa chọn tầng Tác phẩm ghi hai cột trong một `UPDATE`; bật `off` thì tên `NULL`; tên rỗng bị từ chối, không mang nghĩa.
- Ca canh (a)(b)(c)(d)(e phần quét)(g)(h)(i)(j) có đối chứng GỠ thật đúng như AD-68 mục 11 ghi trong ngoặc.
- Quyết định (Ice, 2026-11-05): đổi tên kéo lựa chọn của Tác phẩm đang mở chỉ khi bộ bị đổi tên đúng là bộ lựa chọn đó đang phân giải tới. Work chọn "A" có Work "A" che Global "A", đổi Global "A"→"C" ⇒ lựa chọn giữ "A". Lựa chọn Global đi theo khi bộ đổi tên ở tầng Global.
- Quyết định (Ice, 2026-11-05, sau review lượt 1): đổi tên bộ Global sang một tên mà Tác phẩm đang mở đã có bộ Work riêng thì lựa chọn Work KHÔNG đi theo; lượt quét sau báo thiếu bộ tên cũ, tầng `work`. Lựa chọn Global vẫn đi theo. Giữ mã đã có, vá thay vì làm lại.
- Quyết định (Ice, 2026-11-05): Rust giữ `{name, tier}` của lượt quét vừa xong trong state theo `ProofreadGeneration`; kết quả quét trả mã lượt; `ai_proofread_scan_record` nhận thêm mã đó và Rust tự điền hai cột vết. Mã lạ hoặc cũ ⇒ `scan_record_rejected`.

**Never:**
- Không đổi tập dấu, prompt hay byte của đường dịch (`classify_line`, `guard_text` giữ nguyên hành vi); không đổi năm loại lỗi, `locate_batch_findings`, cách chia lô, cách đếm AD-56.
- Không làm ô chọn, màn xác nhận xoá, wrapper TS cho ba lệnh mới (9.15).
- Không đặt `system` cho lời gọi quét.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Không chọn | Global không khoá, Work `off=0`, tên `NULL` | prompt = bản ghi vàng; `proof_scan` hai cột `NULL` | — |
| Global chọn B | Work theo Global | chỉ dẫn B trước `PROOFREAD_FORMAT`; vết (`B`, `global`) | — |
| Work chọn A | Global chọn B | A; vết tầng của bộ A phân giải tới | — |
| Work tắt | `off=1`, Global chọn B | như "Không chọn" | — |
| Tên mất | Work chọn "X" chỉ có ở Global, máy thiếu X (FR97) | 0 lời gọi mạng | lỗi `name=X`, `tier=work` |
| Trùng tên khác loại | `translate` "A" và `proofread` "A", chọn "A" | thân `proofread` | — |
| Chỉ dẫn mang dấu | một dòng `<<<TEXT` | dòng đó có U+200B | — |
| Văn xuôi | có chỉ dẫn, model trả không phải JSON | câu của lô vào số chưa quét | nhánh `Malformed` hiện có |
| Đổi tên | Work đang mở chọn "A", đổi "A"→"C" | lựa chọn thành "C", lượt sau dùng nó | — |
| Đổi tên bộ bị che | Work chọn "A", Work "A" che Global "A", đổi Global "A"→"C" | lựa chọn Work giữ "A"; lựa chọn Global "A" thành "C" | — |
| Ghi vết lạc lượt | `ai_proofread_scan_record` với mã lượt lạ hoặc cũ | không chèn hàng | `scan_record_rejected` |

</frozen-after-approval>

## Code Map

Đường dẫn dưới `src-tauri/` trừ khi ghi khác; số dòng ở `41cb866e`.

- `src/core/ai/rag.rs` -- `assemble_proofread_batch_prompt(blocks)` :965, hằng `PROOFREAD_*` :931-952 (`SOURCE/TEXT_OPEN/CLOSE` :946-952). Header ghép bằng `\n`: giới thiệu, FORMAT, luật loại, QUOTE, EXPLANATION; nguồn/bản dịch không qua `guard_text`. Test riêng `proofread_prompt_tests` :1023 (không có ca vàng).
- `src/core/ai/batch.rs` -- `classify_line` :52, `guard_marker_line` :104, `unguard_marker_line` :117, `guard_text` :129: tập dấu của đường dịch, KHÔNG sửa. Thêm hàm chèn riêng cho chỉ dẫn quét.
- `src/commands/proofread.rs` -- `prepare_proofread_call(global, open, ids)` :137 (lắp prompt :201, `system: None` :202); `run_proofread_call` :258 (mạng :300, `Malformed` :328); `proofread_scan_record` :560 (INSERT :580) do webview gọi SAU lượt quét qua wire `ai_proofread_scan_record` :766; wire quét :652 dùng `ProofreadGeneration`. Lỗi: khuôn `reply_malformed_error` :124.
- `src/core/promptset/store.rs` -- `resolve_two_tiers(resolver, global, work, kind)` :119 → `ResolvedPromptSet{name, body, tier,…}` :87; `rename` :257; `delete` :348; khuôn `read/write_chosen_name` :371/:383 (của đường dịch, không chạm).
- `src/core/scope/store.rs` -- khuôn khoá `KEY_AI_USAGE_VISIBLE` :182, getter trên `GlobalConfig` :263-380, `save_value` :494, `delete_value` :570; bảng là `config_value`.
- `src/commands/promptset.rs` -- `prompt_set_rename` :247 (biết tầng, có cả `global` lẫn `open.store`), `prompt_set_delete` :293; wire :683+.
- `src/core/i18n/mod.rs` -- khuôn khoá có tham số `PromptSetNameTaken … ["name"]` :790, khoá proofread :937-942; `src/i18n/vi.json`.
- `src/lib.rs` -- `generate_handler!` :1121-1467, proofread :1422-1430.
- Webview: `src/proofreadState.ts` (`runProofread` :167, `recordFinishedScan` :245), `src/config/proofread.ts` (`recordProofreadScan` :365), `src/ProofreaderStrip.vue:252` (`tError(proofreadError)`; `tError` nhận tham số ghi đè để đổi `tier` thành nhãn tầng).
- Test phải sửa: `tests/config_invariants.rs` census :2178 (hàng proofread :2213, promptset :2280); `tests/ipc_contract.rs` ~:2634-2705 (tham số wire proofread và `ai_proofread_scan_record`); `tests/ai_boundary.rs` :642 nếu `proofread.rs` gọi thêm tên `core::ai`. Giả provider: `ScriptedProvider` + `call_count()` ở `tests/ai_proofread_contract.rs:103`.

## Tasks & Acceptance

Bốn pha, mỗi pha một agent mới, bàn giao qua `epic-ai-proofreader/handoff-9-14-phases/`.

**Execution:**
- [x] `tests/fixtures/` + `tests/proofread_instruction_contract.rs` -- việc ĐẦU TIÊN của pha Rust: ghi bản vàng (có nguồn và không nguồn) từ hàm ở baseline -- ca (a).
- [x] `src/core/ai/{rag,batch}.rs` -- `assemble_proofread_batch_prompt` nhận `Option<&str>`; hàm chèn U+200B cho chỉ dẫn -- mục 5.
- [x] `src/core/scope/store.rs` + `src/core/promptset/store.rs` -- khoá `proofread_prompt_set`; đọc/ghi lựa chọn Tác phẩm; hàm phân giải trả `{name, tier, body}` hoặc lỗi thiếu bộ; hàm hậu quả của xoá -- mục 3, 4.
- [x] `src/commands/{proofread,promptset}.rs`, `src/lib.rs`, `src/core/i18n/mod.rs` -- phân giải trong `prepare_proofread_call` trước khoá API; vết vào `proof_scan`; hai lệnh ghi + lệnh hậu quả của xoá; đổi tên kéo lựa chọn (kho trước, lựa chọn sau) -- mục 3, 4, 7.
- [x] `src/i18n/vi.json`, `src/proofreadState.ts`, `src/config/proofread.ts`, `src/ProofreaderStrip.vue` -- chuỗi lỗi; nhãn tầng; `recordProofreadScan` chuyển mã lượt.
- [x] Test -- ca (a)(b)(c)(d)(e)(g)(h)(i)(j) + I/O; cập nhật census, `ipc_contract`; chạy `ai_boundary`.

**Acceptance Criteria:**
- Given Global chọn bộ có thân dài, when quét một Chương nhiều lô, then mọi request mang chỉ dẫn và usage của mọi lô đi tới tổng của lượt (FR76).
- Given lần lượt gỡ thật từng seam ghi trong ngoặc ở AD-68 mục 11, when chạy target của ca đó, then ca đỏ đúng lý do được đo.
- Given xoá một bộ `proofread` tầng Work đang được Tác phẩm mở chọn và Global có bộ cùng tên, when hỏi hậu quả, then Rust trả "bộ Global thay vào"; không có bộ Global cùng tên thì "lượt sau báo thiếu bộ"; bộ không được chọn thì không hậu quả.
- Given `cargo test --test ai_boundary`, then xanh.

## Implementation Notes

- Bản vàng `tests/fixtures/proofread_prompt_{with,without}_source.golden.txt` ghi từ hàm ở baseline trước mọi sửa; đối chiếu tay với hằng ở `41cb866e`.
- Dấu SOURCE/TEXT dời sang `core/ai/batch.rs`; `guard_proofread_instruction` có tập dấu riêng, `classify_line`/`guard_text` của đường dịch không đổi.
- Phân giải chạy trong `prepare_proofread_call` trước khoá API và trước nhánh lượt rỗng, nên lượt rỗng cũng báo thiếu bộ và ghi đúng vết.
- `proof_scan` do webview ghi sau lượt quét, nên vết đi qua một khe `{generation, ScanTrace}` trong `ProofreadGeneration`; `Done` mang `scan_id`, `ai_proofread_scan_record` nhận nó làm đối số đầu, dùng một lần.
- Lỗi thiếu bộ: `err.ai_proofread.instruction_missing` tham số `name`, `tier`; `ProofreaderStrip` đổi `tier` thành nhãn tầng có sẵn.
- Đổi tên bộ Global: lựa chọn Work đi theo khi Work không có bộ trùng tên cũ hay tên mới (quyết định của Ice sau review).
- Hậu quả của xoá chỉ phụ thuộc tầng của bộ bị xoá; ba lệnh mới nằm trong `NO_FRONTEND_CALLER` tới 9.15 (mục nợ `Chủ: Story 9.15`).
- e2e không có spec proofread; trước khi push chạy một spec e2e bất kỳ để bắt TDZ.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick, 2026-11-05): high 0 · medium 1 · low 3 · false 2 · maybe-false 0, cộng 2 low bị loại.

- medium · intent_gap → Ice chọn không kéo lựa chọn và vá tại chỗ — đổi tên Global "A"→"C" khi Work chọn "A" (chỉ có ở Global) và Work có bộ riêng "C": `rename_proofread_set` kéo lựa chọn Work sang "C", lượt sau phân giải tới Work "C", thân khác mà không báo (Prevents 4); quyết định đổi tên của Ice không phủ ca này. (Tự đọc lúc dựng diff.)
- false — hậu quả xoá bộ Global "báo thiếu" dù Tác phẩm đang mở `off` hay có bộ che: AD-68 mục 4 đòi xác nhận khi lựa chọn Global phân giải tới bộ đó, vì mọi Tác phẩm đang đóng theo Global sẽ gãy.
- false — (e) phía lắp prompt dịch chưa có ca: thuộc 9.13, ca nằm ở `ai_prompt_contract.rs`; ticket 9.14 ghi "(e) phần lượt quét".
- low · patch — `prepare_proofread_call` trả `Ready` cho lượt rỗng trước khi phân giải chỉ dẫn, nên `proof_scan` ghi vết `NULL` dù đã chọn bộ và tên mất không được báo; dời phân giải lên trước nhánh rỗng.
- low · patch — chú thích mới trái luật AGENTS.md: chú thích tiếng Việt kèm việc phải làm trên `NO_FRONTEND_CALLER` (`ipc_argument_contract.rs`), mục `# Lỗi` của `resolve_proofread_instruction`, doc comment `InstructionTrace` nhắc lại tên kiểu.
- low · patch — ba mục miễn trừ `NO_FRONTEND_CALLER` không có mục nợ mang chủ; thêm mục sổ nợ `Chủ: Story 9.15`.
- low · rejected — một khe vết duy nhất: lượt mới bắt đầu trước khi bản ghi lượt cũ tới Rust, hoặc lời gọi ghi bị từ chối, làm mất vết; webview ghi ngay sau `Done` và không thử lại, sửa cần thêm nhánh.
- low · rejected — ca AC1 chỉ cộng usage từ sự kiện từng lô, không đọc tổng của lượt: tổng do webview cộng ở mã có sẵn (`proofreadState.ts`), thay đổi không chạm.

## Design Notes

Tập dấu riêng cho chỉ dẫn vì thêm SOURCE/TEXT vào `classify_line` sẽ đổi byte của đường dịch (mục 10). Hậu quả của xoá chỉ phụ thuộc tầng của bộ bị xoá: Work ⇒ Global cùng tên thay vào nếu có, không thì báo thiếu; Global ⇒ báo thiếu (Tác phẩm có Work cùng tên không bị ảnh hưởng). Lệnh đọc lựa chọn cho ô chọn là của 9.15.

## Verification

**Commands:**
- `npm run build` rồi `cd src-tauri && cargo test --test proofread_instruction_contract --test ai_proofread_contract --test prompt_set_contract` -- xanh trong vòng sửa.
- `cargo test --test ai_boundary --test ipc_contract --test config_invariants` -- xanh.
- Một lượt đủ ở cuối pha Tests -- bắt buộc vì đổi đăng ký lệnh trong `lib.rs` (AGENTS.md).
- `npx vitest run` các tệp `tests/frontend/` của proofread -- xanh sau pha Webview; một spec e2e proofread trước khi push.
