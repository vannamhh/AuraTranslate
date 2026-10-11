---
title: 'Quét một Chương hoặc vùng chọn, chỉ khi được gọi'
type: 'feature'
ticket: '3'
created: '2026-10-11'
status: 'built'
baseline_revision: 'cda435d970a5671a65c563a94f0bceb6bab68df0'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Proofreader của 9.1/9.2 chỉ quét segment có con trỏ, nên người dịch không quét được vùng chọn hay cả Chương trước khi bàn giao (FR82). Chưa có ca nào chứng minh proofreader không phát sinh lời gọi AI khi không được gọi, và chưa có ca nào chứng minh quét xong cả Chương mà không xử lý gì thì Chương nguyên vẹn (FR85).

**Approach:** Lệnh quét Rust nhận một danh sách segment, chia thành lô, gọi provider tuần tự và đẩy kết quả từng lô qua Channel. Huỷ giữa chừng thì dừng các lô còn lại và giữ phát hiện đã về. Webview giữ phát hiện theo từng segment, vẽ trên mọi ô, cộng tổng token/chi phí theo khuôn dịch theo lô. Thêm lệnh đăng ký cho phạm vi mới.

**Decision (Ice 2026-10-11):**
- **Lô nhiều segment một lời gọi.** Gom câu cho tới khi nguồn cộng đích đạt khoảng 4.000 ký tự, hoặc đủ 20 câu; một câu dài hơn ngân sách đi một mình. Hai giá trị này là hằng trong Rust. Mỗi khối trong prompt mang chỉ số segment trong lô, và mỗi phát hiện trả về mang chỉ số đó. Cụm chỉ được tìm trong segment đã gán, nên chỉ số lạ hay gán nhầm thì đếm vào `unlocated`. Đã đo bằng tiktoken `o200k_base`: phần cố định của prompt là 153 token mỗi lời gọi; Chương 184 câu tốn khoảng 14k token đầu vào khi gom lô 20, so với khoảng 40k khi gọi từng câu. Chất lượng khi lô lớn để 9.8 đo.
- **Ba lệnh và bộ chọn ba ô** (Câu này · Vùng chọn n câu · Cả Chương n câu), theo `proofreader.html:300-302`. Bộ chọn mở từ một nút `titlebar-act` ở góc phải thanh tiêu đề, theo khuôn lớp phủ sẵn có. Mỗi ô `dispatch` một lệnh. ⌘P giữ nghĩa *câu này*.
- Giữ trọn plan (3.810 token).

## Boundaries & Constraints

**Always:** AD-13 (chỉ `commands/proofread.rs` chạm `core::ai`; mọi tên `core::ai::*` mới vào allow-list seam proofread của `ai_boundary`), AD-14, AD-15, AD-22 (Channel, huỷ được, không tự thử lại), AD-53 mục 1 và 5:
- `scanned_text` là cả segment;
- kết quả mang thế hệ cũ hơn lượt reset gần nhất bị bỏ, kể cả lô về muộn;
- `Done` thêm `not_scanned`: số segment trong phạm vi có lô lỗi hoặc bị huỷ;
- đổi Chương hay Tác phẩm thì reset tất cả.

Hình dạng một phát hiện trên dây giữ nguyên. Segment `is_omitted` hoặc có bản dịch rỗng thì không gửi đi, cũng không tính vào `not_scanned`. Tổng token và chi phí cộng ở webview, theo khuôn `aiTranslateBatchUsageSummary`, và phân biệt trọn với một phần. Lệnh quét và lệnh huỷ đăng ký trong `CommandRegistry`, gán phím lại được. Vẫn từ chối quét khi đang dịch đơn hoặc dịch theo lô.

**Never:** ghi `project.db` hay đổi trạng thái xác nhận; chấp nhận, bỏ qua, ghi nhớ, `ambiguous`, `filtered` (9.4, 9.6); quét nền, quét lại sau khi gõ, hoặc tự chạy khi mở Chương; ước tính chi phí trước khi bấm (mockup có, AC không có, dịch theo lô cũng chưa có ⇒ ghi nợ `Chủ: Sally`); đổi hành vi dịch.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Cả Chương | 30 segment, 3 rỗng, 1 omitted | 26 segment được quét; gạch chân trên mọi ô có phát hiện; tổng token/chi phí; `not_scanned: 0` | — |
| Huỷ giữa chừng | huỷ sau khi lô 2/4 về | provider được gọi đúng 2 lần; phát hiện của lô 1–2 còn trên ô; `cancelled`, `not_scanned` = số segment của lô 3–4; tổng chỉ tính lô đã về | — |
| Phản hồi một lô hỏng | lô 2 trả về không phải JSON | lô 1, 3 vẫn hiện; segment của lô 2 vào `not_scanned` | không làm hỏng cả lượt |
| Lỗi provider | lô 2 lỗi mạng hoặc khoá | dừng; lô 1 giữ; lô 2 trở đi vào `not_scanned` | `IpcError` như 9.1, dải nêu lỗi |
| Vùng chọn | 5 dòng đang chọn | chỉ quét đúng 5 segment, theo thứ tự tài liệu | — |
| Không gọi | gõ, flush, đổi Chương, chọn vùng, dịch | 0 lời gọi `ai_proofread_*` và 0 lời gọi provider | — |
| Lô về muộn | đổi Chương trong lúc quét | lô về sau reset bị bỏ, không vẽ | — |
| Gõ vào ô đã quét | trong lúc lượt quét cả Chương còn chạy | phát hiện của ô đó bị xoá; các ô khác giữ | — |
| Chưa cấu hình | | `not_configured` trước mọi lời gọi | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/proofread.rs` (253 dòng):
  - `ProofreadGeneration` :22 dùng làm cờ huỷ, được kiểm giữa các lô và trong luồng SSE.
  - `prepare_proofread_call` :77 đổi thành nhận `&[i64]`. Theo khuôn `prepare_batch_call` (`aitranslate.rs:409`): đọc `read_open_chapter_segments` một lần, sắp lại theo thứ tự tài liệu, id lạ thì trả `segment_not_in_chapter`, bỏ qua dòng omitted hoặc rỗng.
  - `run_proofread_call` :116 là seam test, thành vòng lặp lô.
  - `AiProofreadOutcomeWire` :49 thêm `not_scanned` vào `Done` và `Cancelled`.
  - `wire::ai_proofread_segment` :199 thay bằng một lệnh nhận `segmentIds` và `Channel` sự kiện, tag `kind`, theo khuôn `AiTranslateBatchEventWire` (`aitranslate.rs:586`). Mỗi lô về là một sự kiện mang `usage` và từng segment `{segment_id, scanned_text, findings, unlocated}`.
  - Đăng ký ở `lib.rs:1294-1295`; `ProofreadGeneration` ở `lib.rs:1581`.
- `src-tauri/src/core/ai/rag.rs:670-707` `assemble_proofread_prompt` thành prompt lô: mỗi khối mang chỉ số và nguồn/đích. Khối nguồn rỗng thì không xin loại nghĩa cho khối đó, giữ luật của 9.2. `core/ai/proofread.rs` (212 dòng): `RawFinding` thêm chỉ số segment, `locate_findings` chạy theo từng segment của lô. Hằng ngân sách lô đặt cạnh nhau, ở `core/ai/proofread.rs`.
- `src-tauri/tests/ai_proofread_contract.rs`:
  - `FakeProvider` :83 chưa đếm lời gọi. Lấy khuôn `MultiItemProvider` (`ai_translate_contract.rs:1818`, các ca huỷ :2379-2415).
  - Cập nhật `ai_boundary.rs:513` nếu thêm tên `core::ai::*`.
  - Nếu tên lệnh wire đổi, cập nhật `tests/ipc_contract.rs`.
- `src/config/proofread.ts` (106 dòng): `runProofreadSegment` :75 thành lệnh nhận danh sách id. Khuôn adapter sự kiện ở `config/aitranslate.ts:101,197`.
- `src/proofreadState.ts` (122 dòng):
  - Từ một segment thành `Map<segmentId, {scannedText, findings, unlocated}>`, cùng tiến độ đã xong trên tổng, `not_scanned` và tổng usage.
  - Giữ `sequence` :19, phép flush trước khi quét, `clearProofreadFor` :102 (chỉ xoá một segment) và `resetProofread` :115, được gọi ở `editorPanelState.ts:1888`, `libraryChapters.ts:321` và `libraryImport.ts:441`.
  - Khuôn tổng: `aiTranslateBatchState.ts:139`.
- `src/proofreadHandlers.ts` lấy phạm vi từ `segmentSelectionIds` (`segmentSelectionState.ts:81`, khuôn `aiTranslateHandlers.ts:69-74`) hoặc từ `editorSegments`.
- `src/commands/index.ts:1075-1078` khai dep port. Lệnh ở :4150-4171. Nhãn `command.*` thêm vào `vi.json`.
- `src/panels/GridPanel.vue`:
  - `repaintProofread` :1094-1112 đổi từ vẽ một ô thành gom range của mọi ô có phát hiện rồi gọi `paintProofreadRanges` một lần. Hàm này thay highlight toàn cục.
  - Watch ở :1114-1118; `reportEdit` → `clearProofreadFor` ở :1592.
- `src/ProofreaderStrip.vue` (133 dòng):
  - Dòng usage :43-48 hiện viết "câu này". Thêm khoá cho tiến độ, tổng và `not_scanned`.
  - Chuyển `aiTranslateBatchUsageLine`, hiện riêng tư trong `AiTranslationPanel.vue:265-300`, sang `src/aiUsageLine.ts` dưới dạng hàm nhận khoá.
  - Điều kiện hiện dải ở `inlineStripEligibility.ts:14-16`.
- Ca vitest cần cập nhật: `tests/frontend/proofreadState.test.ts`, `proofreadWiring.test.ts` (kiểm nguồn GridPanel ở :68-77), `proofreaderStrip.test.ts`. Khuôn emit giả: `aiTranslateBatch.test.ts:113`. e2e không giả provider (`proofread-underline.e2e.mjs:105-106`), nên không thêm e2e.

## Tasks & Acceptance

**Execution:** ba pha, mỗi pha một agent mới, bàn giao qua tệp trong scratchpad: **Rust** (ba task đầu), **Webview** (bốn task kế), **Đo & đối chứng** (hai task cuối, chạy bộ đầy đủ một lần vì `lib.rs` đổi).
- [x] `src-tauri/src/core/ai/`: prompt lô, chia lô theo ngân sách, phân tích chỉ số segment. Unit test cho:
  - biên ngân sách: câu dài đi một mình, trần 20 câu;
  - chỉ số lạ ⇒ `unlocated`;
  - khối có nguồn rỗng.
- [x] `src-tauri/src/commands/proofread.rs`, `lib.rs`, `ai_boundary.rs`, `ipc_contract.rs`: lệnh nhận danh sách id, sự kiện theo lô, `not_scanned`, huỷ giữa các lô, lô hỏng không làm hỏng cả lượt.
- [x] `tests/ai_proofread_contract.rs` có provider đếm lời gọi. Các ca:
  - mỗi hàng Rust của ma trận;
  - huỷ sau lô k ⇒ đúng k lời gọi;
  - quét cả Chương xong thì `target_text`, `status`, hai cột xuất xứ của mọi segment và số `segment_version` bằng trước khi quét.
- [x] `src/config/proofread.ts`, `src/proofreadState.ts`, `src/proofreadHandlers.ts`, `src/commands/index.ts`, `vi.json`:
  - `ai.proofread.run` giữ ⌘P và nghĩa *câu này*;
  - thêm `ai.proofread.run_selection` (không phím mặc định) và `ai.proofread.run_chapter` (⌘⇧P, đã kiểm còn trống);
  - thêm lệnh mở bộ chọn;
  - state theo segment, tổng usage.
- [x] Bộ chọn phạm vi: nút `titlebar-act` trong `App.vue` (các nút hiện có ở :263-345) mở một lớp nhỏ ba ô.
  - Mỗi ô hiện số câu sẽ quét; ô *Vùng chọn* bị tắt khi chưa chọn gì.
  - Tiêu điểm theo UX-DR17; `reset*` nối vào cụm reset mà `check:panel-refs` đòi.
- [x] `GridPanel.vue`, `ProofreaderStrip.vue`, `aiUsageLine.ts`: vẽ mọi ô, dải tiến độ, tổng và `not_scanned`.
- [x] vitest:
  - state theo segment; lô về muộn bị bỏ; `clearProofreadFor` chỉ xoá một ô;
  - tổng trọn, tổng không có giá, tổng một phần;
  - phạm vi vùng chọn và cả Chương; mỗi ô của bộ chọn `dispatch` đúng lệnh của nó;
  - một phiên mô phỏng (gõ, flush, đổi Chương, chọn vùng, dịch) có 0 lời gọi `ai_proofread_*`, rồi `dispatch` lệnh thì đúng 1 lời gọi.
- [x] Đối chứng, mỗi chỗ phải gỡ thật:
  - gỡ phép kiểm huỷ giữa các lô ⇒ ca đếm lời gọi đỏ;
  - gỡ phép so thế hệ ⇒ ca lô về muộn đỏ;
  - gỡ vòng gom range nhiều ô ⇒ ca vẽ đỏ;
  - cắm một lời gọi quét vào đường gõ ⇒ ca 0 lời gọi đỏ.
- [x] `deferred-work.md`: nợ ước tính chi phí trước khi bấm (`Chủ: Sally`). Lượt quét cả Chương với AI thật gộp vào mục nợ `Chủ: Epic 9` sẵn có.

**Acceptance Criteria:**
- Given một Chương đã dịch, when người dùng gọi lệnh quét cả Chương rồi huỷ giữa chừng, then lời gọi provider dừng ngay sau lô đang chạy, gạch chân của phần đã quét còn nguyên, và dải hiện tổng token/chi phí của phần đã chạy cùng số câu chưa quét.
- Given ứng dụng đang chạy, when người dùng làm việc mà không gọi lệnh proofreader, then không có lời gọi `ai_proofread_*` và không có lời gọi provider nào.
- Given lượt quét cả Chương đã xong, when người dùng chưa xử lý phát hiện nào, then mọi segment của Chương giữ nguyên văn bản, trạng thái và xuất xứ.
- Given màn cấu hình phím tắt, when người dùng gán lại phím cho lệnh quét và lệnh huỷ, then các lệnh chạy theo phím mới.
- Given một Chương đang mở, when người dùng bấm nút proofreader trên thanh tiêu đề, then bộ chọn hiện ba phạm vi kèm số câu, và chọn một ô thì quét đúng phạm vi đó.

## Implementation Notes

- Lô gom theo ngân sách 4.000 ký tự nguồn cộng đích hoặc 20 câu (hằng ở `core/ai/proofread.rs`); chỉ số trong lô là vị trí 1-based, nội bộ Rust, không lên dây. Chỉ số lạ hay thiếu đếm vào `unlocated`.
- Lệnh `ai_proofread_segments` thay `ai_proofread_segment`; sự kiện `planned` rồi `batch` (mỗi segment của lô đều có mặt, kể cả 0 phát hiện). Lô hỏng phát `batch` chỉ mang `usage` với `segments` rỗng (đã bị tính tiền) và cộng `not_scanned`; nếu không lô nào đọc được thì trả `ai_proofread.reply_malformed`.
- Lỗi provider dừng cả lượt, `IpcError.params.not_scanned` mang số segment chưa quét; huỷ được kiểm trước mỗi lô và trong lời gọi, nên huỷ sau lô k thì đúng k lời gọi.
- Mỗi lệnh quét mới tăng thế hệ và huỷ lượt đang chạy. Lô về sau `reset*` bị bỏ ở webview bằng `sequence`; Rust không biết reset.
- Tổng token/chi phí cộng ở webview (`proofreadUsageSummary`), phân biệt trọn, không giá và một phần; hàm dòng usage chuyển sang `aiUsageLine.ts` dùng chung với dịch theo lô.
- `clearProofreadFor(id)`: lượt một segment thì huỷ luôn; lượt nhiều segment chỉ xoá ô đó, lượt tiếp tục và kết quả về sau cho ô đó bị bỏ.
- Phạm vi đếm ở webview (`proofreadScope.ts`) bỏ omitted và đích rỗng, khớp Rust. Bộ chọn ba ô mở từ nút thanh tiêu đề, `main.ts` chặn phím khi mở. Bấm nút làm cột bản dịch mất tiêu điểm và con trỏ về `null`, nên phạm vi *câu này* dùng id con trỏ khác `null` gần nhất (cả ⌘P khi tiêu điểm ở panel khác).
- Ngưỡng gate tự đề nghị nâng vì code mới: `CLICK_FLOOR` 140→151, `HANDLER_ATTR_FLOOR` 91→98, `COMPONENT_FILE_FLOOR` 127→136.
- Không đo được: dựng GridPanel thật vẽ nhiều ô (chỉ quét nguồn + vòng lặp đã đối chứng), bố cục bộ chọn trong cửa sổ thật; ghi nợ `Chủ: Epic 9`, ước tính chi phí `Chủ: Sally`.
- Đối chứng gỡ thật (cả bốn đỏ đúng lý do): kiểm huỷ giữa lô (2 ca đỏ), so thế hệ (3 ca đỏ), vòng range nhiều ô (ca nguồn đỏ), lời gọi cắm vào `clearProofreadFor` (ca 0 lời gọi đỏ).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 1 high · 3 medium · 3 low · 0 false.
- high · patch — "Câu này" luôn 0 câu khi mở bộ chọn bằng chuột: nút `titlebar-act` gọi `focusOnPointerDown`, cột bản dịch mất tiêu điểm nên `onColumnFocusOut` (`GridPanel.vue:1033-1036`) đặt con trỏ về `null` trước khi bộ chọn mở; `proofreadScope.ts` giữ id con trỏ khác `null` gần nhất.
- medium · patch — dải tổng và `not_scanned` biến mất ngay khi bấm Huỷ: cú bấm đó cũng đặt con trỏ về `null` mà `inlineStripEligibility.ts` đòi con trỏ nằm trong lượt quét; nay con trỏ `null` vẫn hiện dải.
- medium · patch — lô có phản hồi hỏng đã bị tính tiền nhưng không sự kiện nào mang `usage` của nó, nên tổng token/chi phí thấp hơn số tiêu thật; nay vẫn gửi sự kiện `batch` mang `usage`, kèm `segments` rỗng.
- medium · patch — phạm vi không có câu nào quét được (⌘P trên câu bỏ qua hoặc chưa dịch) kết thúc `done` với 0 câu, và dải báo "không thấy lỗi"; nay dải nói không có câu đã dịch để quét.
- low · patch — `role="dialog"` của bộ chọn chưa có tên truy cập; thêm `aria-labelledby`, cùng cách `AiPromptInspectorOverlay.vue:153`.
- low · loại — chỉ số `segment` không dùng được thì bị đếm vào `unlocated` của segment 1: webview chỉ dùng tổng, nên người dùng không thấy sai; sửa phải thêm trường mới.
- low · loại — sửa một ô trong lúc flush trước lượt quét thì kết quả của ô đó bị bỏ: cửa sổ thời gian rất hẹp, ô không bị gạch nhầm chỗ, chỉ không được tính vào `not_scanned`; sửa phải thêm nhánh cho pha flush.

## Design Notes

Chỉ số segment trong lô là thứ tự trong lô, không phải `segment.id`: model chép số nhỏ ít sai hơn, và Rust ánh xạ ngược. Tổng cộng ở webview chứ không ở Rust, vì dịch theo lô đã làm vậy và Rust không cộng usage ở đâu cả. Phép thử *"không chiếm gì khi không được gọi"* đặt ở webview, vì webview là bên duy nhất khởi động lệnh: Rust chỉ dựng `OpenAiChatClient` cho proofread bên trong lệnh wire (`ai_boundary` ghim đường này), và lệnh wire chỉ chạy khi được invoke. Lỗi provider dừng cả lượt như dịch theo lô (`batch_stopped_error`). Phản hồi hỏng chỉ mất lô đó, vì nó là lỗi của model chứ không phải của kết nối.

## Verification

**Commands:**
- `npm run build`, rồi `cargo test --test ai_proofread_contract --test ai_boundary --test ipc_contract`: xanh.
- `npx vitest run` trên các tệp proofread, `proofreaderStrip`, `aiTranslateBatch` và tệp test của bộ chọn: xanh.
- `npm run check:commands check:i18n check:tokens check:panel-refs`: xanh.
- Bộ đầy đủ chạy một lần ở pha cuối, vì `lib.rs` đổi phần đăng ký lệnh.

**Manual checks (if no CLI):**
- Quét cả Chương với AI thật trong app thật: gộp vào mục nợ `Chủ: Epic 9` sẵn có.
