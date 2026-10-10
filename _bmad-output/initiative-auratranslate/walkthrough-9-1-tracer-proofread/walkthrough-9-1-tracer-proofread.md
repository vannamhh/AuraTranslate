# Walkthrough: 9-1 tracer proofreader

Đích: commit `ed7b7f13e32b68455828a0b2708888dfb9386fa5` · plan: [story-tracer-…-plan.md](../epic-ai-proofreader/story-tracer-quet-chinh-ta-va-ngu-phap-mot-segment-gach-chan-ngay-plan.md) · log: [walkthrough-9-1-tracer-proofread-log.md](walkthrough-9-1-tracer-proofread-log.md)

Khối hiện tại: **1**

## [ ] 1. Ý định — chưa xem

Chép nguyên văn mục `## Intent` của plan.

**Problem:** Chưa có đường nào đưa bản dịch của người dùng qua AI để bắt lỗi chính tả/ngữ pháp và vẽ kết quả ngay dưới chữ trong ô (FR80, FR85, FR86); hình dạng phát hiện trên dây chưa có nên 9.2–9.6 không có gì để xây lên.

**Approach:** Lát mỏng xuyên lớp: lệnh `ai.proofread.run` (⌘P theo mockup) flush segment có con trỏ → command Rust mới trong file seam mới `commands/proofread.rs` → hàm lắp prompt thuần trong `core/ai/rag.rs` nhận bản dịch → `TranslationProvider` → token qua Channel, kết quả cuối là danh sách phát hiện Rust đã định vị → webview vẽ gạch chân lượn sóng `error` bằng CSS Custom Highlight API. Không ghi `project.db`.

**Decision (Ice 2026-10-10):** văn bản prompt quét cố định trong `core/ai/rag.rs` (đánh dấu `aura-allow-text`), hàm thuần nhận bản dịch; không thêm biến prompt-set. Kế hoạch giữ trọn (3.108 token), không tách e2e.

## [ ] 2. Nét lớn — chưa xem

Một lượt quét đi từ phím tắt xuống Rust rồi quay về ô. Mở theo thứ tự:

1. [src/commands/index.ts:4151](../../../src/commands/index.ts#L4151) — lệnh `ai.proofread.run`, gắn `Mod+P`.
2. [src/proofreadState.ts:39](../../../src/proofreadState.ts#L39) — `runProofread`: flush Editor, gọi Rust, giữ kết quả trong bộ nhớ webview.
3. [src-tauri/src/commands/proofread.rs:197](../../../src-tauri/src/commands/proofread.rs#L197) — command `ai_proofread_segment`: lấy thế hệ huỷ, chuẩn bị, gửi qua provider.
4. [src-tauri/src/core/ai/proofread.rs:61](../../../src-tauri/src/core/ai/proofread.rs#L61) — `locate_findings`: đọc JSON của model, tìm từng cụm trích trong văn bản, tính offset UTF-16.
5. [src/panels/GridPanel.vue:1093](../../../src/panels/GridPanel.vue#L1093) — `repaintProofread`: dựng Range trên ô và vẽ bằng `CSS.highlights`.

## [ ] 3. Command Rust chỉ-đọc và huỷ — chưa xem

`prepare_proofread_call` kiểm theo thứ tự: có Tác phẩm mở, cấu hình AI, segment thuộc Chương đang mở, khoá API — chưa cấu hình thì trả `not_configured` trước mọi lượt mạng; bản dịch rỗng trả `done` rỗng. Lượt quét có bộ đếm thế hệ riêng nên huỷ dịch không huỷ quét; thế hệ được lấy *trước* bước chuẩn bị để một lệnh huỷ đến trong lúc đọc keychain không bị ghi đè. Không đường nào ở đây ghi `project.db`.

- [proofread.rs:22](../../../src-tauri/src/commands/proofread.rs#L22) `ProofreadGeneration`
- [proofread.rs:48](../../../src-tauri/src/commands/proofread.rs#L48) `AiProofreadOutcomeWire` — hình dạng trên dây
- [proofread.rs:76](../../../src-tauri/src/commands/proofread.rs#L76) `prepare_proofread_call`
- [proofread.rs:114](../../../src-tauri/src/commands/proofread.rs#L114) `run_proofread_call` — chỗ test thay provider giả
- [proofread.rs:242](../../../src-tauri/src/commands/proofread.rs#L242) `ai_proofread_cancel`
- [aitranslate.rs:231](../../../src-tauri/src/commands/aitranslate.rs#L231) — `resolve_call_config`, `read_api_key` nâng `pub(crate)` để dùng lại

## [ ] 4. Prompt và định vị cụm chữ — chưa xem

Prompt là chuỗi cố định bọc bản dịch giữa `<<<TEXT` và `TEXT>>>`, dặn model trả mảng JSON `{kind, quote, explanation, suggestion}` và không bao giờ trả offset. Rust tự tìm `quote`: lặp thì lấy lần xuất hiện đầu chưa bị phát hiện trước chiếm; không tìm thấy thì đếm vào `unlocated` thay vì bỏ im lặng; phản hồi lệch lược đồ là lỗi `ai_proofread.reply_malformed` có thể thử lại.

- [rag.rs:670](../../../src-tauri/src/core/ai/rag.rs#L670) — các hằng prompt (`aura-allow-text`)
- [rag.rs:686](../../../src-tauri/src/core/ai/rag.rs#L686) `assemble_proofread_prompt`
- [core/ai/proofread.rs:47](../../../src-tauri/src/core/ai/proofread.rs#L47) `strip_code_fence`
- [core/ai/proofread.rs:61](../../../src-tauri/src/core/ai/proofread.rs#L61) `locate_findings`

## [ ] 5. Ranh giới AD-13 — chưa xem

`commands/proofread.rs` là seam thứ tư được phép chạm `core::ai`. Cổng khoá đúng năm đường nó được gọi; một tên ngoài danh sách làm đỏ. Compile probe (bị `#[ignore]`) xoá cả seam này khi xoá `core/ai`.

- [ai_boundary.rs:509](../../../src-tauri/tests/ai_boundary.rs#L509) — hằng seam và danh sách cho phép
- [ai_boundary.rs:582](../../../src-tauri/tests/ai_boundary.rs#L582) — cổng năm đường

## [ ] 6. Trạng thái webview và adapter — chưa xem

Sáu trạng thái (`idle · scanning · done · error · cancelled · not_configured`). Quét luôn flush trước; flush trượt hay còn dơ thành lỗi có tên. Một bộ đếm `sequence` bỏ kết quả đến muộn. Huỷ trong lúc còn flush thì dừng ngay, không gửi Rust. Gõ trong ô đang quét hay đã quét xoá kết quả của segment đó; đổi Chương, mở hay tạo Tác phẩm thì reset. Adapter không bao giờ ném.

- [proofreadState.ts:39](../../../src/proofreadState.ts#L39) `runProofread` · [:58](../../../src/proofreadState.ts#L58) nhánh flush
- [proofreadState.ts:90](../../../src/proofreadState.ts#L90) `cancelProofread` · [:102](../../../src/proofreadState.ts#L102) `clearProofreadFor` · [:115](../../../src/proofreadState.ts#L115) `resetProofread`
- [config/proofread.ts:69](../../../src/config/proofread.ts#L69) `runProofreadSegment`
- [proofreadHandlers.ts:7](../../../src/proofreadHandlers.ts#L7) — từ chối quét khi đang dịch
- [editorPanelState.ts:1888](../../../src/panels/editorPanelState.ts#L1888) — reset khi đổi Chương

## [ ] 7. Vẽ gạch chân trong ô — chưa xem

Range được dựng từ offset UTF-16 qua mọi text node của ô và chỉ khi `textContent` còn bằng đúng văn bản đã quét. Highlight sống ngoài DOM của Vue nên ô không bị render lại. Trong WKWebView chỉ ba longhand `text-decoration-line/-style/-color` có hiệu lực; offset 4px của mockup không đạt được (nợ của 9.5).

- [proofreadHighlight.ts:6](../../../src/panels/proofreadHighlight.ts#L6) `rangeForOffsets` · [:29](../../../src/panels/proofreadHighlight.ts#L29) `rangesForScan` · [:39](../../../src/panels/proofreadHighlight.ts#L39) `paintProofreadRanges`
- [GridPanel.vue:1093](../../../src/panels/GridPanel.vue#L1093) `repaintProofread` · [:1582](../../../src/panels/GridPanel.vue#L1582) xoá khi gõ · [:2421](../../../src/panels/GridPanel.vue#L2421) CSS `::highlight`

## [ ] 8. Dải trạng thái — chưa xem

Dải dùng khe `proofreader` đã giữ chỗ trong bộ phân xử dải: đang quét kèm nút Huỷ, xong thì số phát hiện, số `unlocated` nếu có, token và chi phí; chưa cấu hình thì mời cấu hình bằng câu của dịch đơn. Dòng token/chi phí được rút ra hàm chung với panel dịch.

- [ProofreaderStrip.vue:18](../../../src/ProofreaderStrip.vue#L18)
- [inlineStripEligibility.ts:16](../../../src/inlineStripEligibility.ts#L16)
- [aiUsageLine.ts:11](../../../src/aiUsageLine.ts#L11)

## [ ] 9. Test — chưa xem

- [ai_proofread_contract.rs:161](../../../src-tauri/tests/ai_proofread_contract.rs#L161) — provider giả: xong, `unlocated`, hỏng, huỷ, chưa cấu hình, ngoài Chương
- [ai_proofread_contract.rs:277](../../../src-tauri/tests/ai_proofread_contract.rs#L277) — quét có phát hiện không đổi `target_text`, xuất xứ, trạng thái của mọi segment
- [proofreadState.test.ts:43](../../../tests/frontend/proofreadState.test.ts#L43) · [proofreadHighlight.test.ts:10](../../../tests/frontend/proofreadHighlight.test.ts#L10) · [proofreadWiring.test.ts:36](../../../tests/frontend/proofreadWiring.test.ts#L36) · [proofreaderStrip.test.ts:26](../../../tests/frontend/proofreaderStrip.test.ts#L26)
- [proofread-underline.e2e.mjs:79](../../../e2e/specs/proofread-underline.e2e.mjs#L79) — WKWebView thật: Range đúng cụm, pixel màu `error` dưới cụm

## [ ] 10. Ngoại vi — chưa xem

- [lib.rs:1294](../../../src-tauri/src/lib.rs#L1294) đăng ký hai command · [lib.rs:1581](../../../src-tauri/src/lib.rs#L1581) quản lý `ProofreadGeneration`
- [i18n/mod.rs:864](../../../src-tauri/src/core/i18n/mod.rs#L864) khoá lỗi mới · [vi.json:102](../../../src/i18n/vi.json#L102) chuỗi giao diện
- [config_invariants.rs:1766](../../../src-tauri/tests/config_invariants.rs#L1766) điều tra tệp command · [ipc_contract.rs:2233](../../../src-tauri/tests/ipc_contract.rs#L2233) ca đăng ký
- [App.vue:400](../../../src/App.vue#L400) gắn dải · [AiTranslationPanel.vue:81](../../../src/panels/AiTranslationPanel.vue#L81) dùng hàm chung
- [config/aitranslate.ts:68](../../../src/config/aitranslate.ts#L68) xuất bốn helper cho adapter mới
- Sàn cổng nâng: [check-doc-refs.mjs:121](../../../scripts/check-doc-refs.mjs#L121), `check-commands`, `check-panel-refs`, `check-tokens`
- [deferred-work.md](../deferred-work.md) — ba mục nợ mới (grep `story-tracer-quet`)
