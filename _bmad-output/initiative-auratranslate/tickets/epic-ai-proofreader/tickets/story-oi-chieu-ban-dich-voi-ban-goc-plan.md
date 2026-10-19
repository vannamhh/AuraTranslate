---
title: 'Đối chiếu bản dịch với bản gốc'
type: 'feature'
ticket: '2'
created: '2026-10-10'
status: done
baseline_revision: '9f8fa8aa9e122413f437c29a00c49b148b0af126'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Lượt quét của 9.1 chỉ đọc bản dịch, nên không bắt được câu dịch sai, thoát nghĩa quá xa hay tối nghĩa so với bản gốc (FR81); và loại phát hiện còn là enum trong `core/ai/`, trái AD-53 mục 1.

**Approach:** Cùng một lượt quét `ai.proofread.run`, prompt đọc cả nguồn lẫn đích và trả thêm ba loại nghi về nghĩa; tập loại đóng chuyển sang `core/segment/`, mỗi loại khai hai thuộc tính của AD-53; webview vẽ loại nghĩa bằng một highlight thứ hai màu `tm-rule`.

**Decision (Ice 2026-10-10):** loại nghĩa vẽ bằng `tm-rule` đúng tiêu chí gốc; độ tương phản 2,48:1 và nghĩa của màu vẫn là `unknown` của 9.5. Giữ trọn plan (3.225 token), không tách e2e.

## Boundaries & Constraints

**Always:** AD-13 (chỉ `commands/proofread.rs` chạm `core::ai`; `core/segment` không import `core::ai`), AD-14 (prompt lắp bằng hàm thuần, văn bản prompt mang `aura-allow-text`), AD-15, AD-22, AD-53 mục 1: tập loại `spelling · grammar · mistranslation · too_loose · obscure` ở `core/segment/`, mỗi loại khai nhánh chữ ký (chính tả: theo cụm; còn lại: theo cụm và ngữ cảnh) và luật xuất xứ (chính tả, ngữ pháp: phân xử văn bản trước; ba loại nghĩa: người khác dịch); nhãn model ngoài tập ⇒ đếm vào `unlocated`, không làm hỏng cả phản hồi. Một lời gọi AI cho mọi loại (FR82). Cụm vẫn chỉ tìm trong bản dịch; `scanned_text` vẫn là `target_text`. Màu chỉ từ token, CSS `::highlight()` chỉ dùng longhand.

**Never:** quét Chương hay vùng chọn (9.3); chấp nhận, bỏ qua, ghi nhớ, `ambiguous` (9.4, 9.6); ghi `project.db`; thêm token màu; sửa `assemble_prompt` của dịch; re-export loại qua `core::ai`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Lệch nghĩa | nguồn + bản dịch lệch nghĩa, model trả `mistranslation` | phát hiện loại nghĩa, gạch chân `tm-rule` dưới cụm | — |
| Hai loại cùng câu | một lỗi chính tả + một cụm thoát nghĩa | hai highlight khác màu cùng lúc | — |
| Chưa dịch | `target_text` rỗng | `done` rỗng, không gọi mạng (như 9.1) | — |
| Nguồn rỗng | nguồn rỗng sau trim, bản dịch có chữ | prompt không có khối nguồn và không xin loại nghĩa; phát hiện loại nghĩa model vẫn trả ⇒ `unlocated` | — |
| Nhãn lạ | model trả `kind: "style"` | phát hiện đó vào `unlocated`, phần còn lại vẫn hiện | — |
| Phản hồi hỏng | không phải mảng JSON, thiếu `quote` | không vẽ | `ai_proofread.reply_malformed`, như 9.1 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/ai/proofread.rs` -- `FindingKind` :8-13 rời khỏi đây; `RawFinding.kind` :38 thành `String`, ánh xạ qua tập ở `core/segment`, không khớp ⇒ `unlocated`; `locate_findings` :61 và `utf16_len` giữ nguyên logic đặt cụm; test :172-181 (`"style"` ⇒ Malformed) đổi thành ⇒ `unlocated`, ca thiếu `quote` vẫn Malformed.
- `src-tauri/src/core/segment/` -- module mới (vd. `proofread_kind.rs`) + dòng `pub mod` và đoạn doc trong `mod.rs`; khuôn tập đóng ↔ chuỗi dây: `PairOrigin::from_stored`/`as_str` (`core/tm/mod.rs:44-66`), `Arbitrated::as_str` (`core/segment/translation_origin.rs:12-27`); hai thuộc tính là `match` trên enum.
- `src-tauri/src/core/ai/rag.rs` -- bảy hằng :669-682 (`KIND_RULE` :674 cứng hai loại); `assemble_proofread_prompt(target)` :686 đổi thành nhận `(source, target)`, thêm khối nguồn và luật loại nghĩa; test :693-704. Không đụng `assemble_prompt` :614.
- `src-tauri/src/commands/proofread.rs` -- `prepare_proofread_call` :76 đã có `row.source_text` (`ChapterSegment`, `chapter_read.rs:72`), :106 truyền thêm nó; nhánh `EmptyText` :91-93 giữ; `ProofreadFinding.kind` lấy kiểu từ `core::segment`. `tests/ai_boundary.rs:513-519` không đổi (năm đường vẫn được gọi tên; đường `core::segment` không bị đếm).
- `src/config/proofread.ts` -- kiểu loại :11 và `isFindingWire` :40 nhận năm chuỗi.
- `src/panels/proofreadHighlight.ts` -- một khoá `proofread-error` :1; thêm khoá loại nghĩa, `paintProofreadRanges` :38-46 tách range theo loại, set hoặc delete từng khoá.
- `src/panels/GridPanel.vue` -- `repaintProofread` :1093-1102; quy tắc `::highlight(proofread-error)` :2421-2427 (longhand, có comment WKWebView); thêm quy tắc anh em `var(--color-tm-rule)`. `check:tokens` cho qua `text-decoration-color` dùng token vai `stroke`.
- Test: `src-tauri/tests/ai_proofread_contract.rs` (`FakeProvider` :83-109, import loại :19, malformed :194, prompt mang đích :253); `tests/frontend/proofreadConfig.test.ts`, `proofreadHighlight.test.ts` (:66-74); `e2e/specs/proofread-underline.e2e.mjs` (`errorPixels` :59 dùng chung được, highlight và token :115-127).

## Tasks & Acceptance

**Execution:** ba pha, mỗi pha một agent mới, bàn giao qua tệp trong scratchpad: **Rust** (bốn task đầu), **Webview** (hai task kế), **Đo & đối chứng** (hai task cuối, bộ đầy đủ một lần).
- [x] `src-tauri/src/core/segment/` -- tập loại đóng, chuỗi dây, hai thuộc tính AD-53; unit test chuỗi ↔ loại và hai thuộc tính của từng loại.
- [x] `src-tauri/src/core/ai/proofread.rs` -- dùng tập mới, nhãn lạ vào `unlocated`; test các hàng ma trận Nhãn lạ, Phản hồi hỏng.
- [x] `src-tauri/src/core/ai/rag.rs` -- prompt nhận nguồn và đích, xin năm loại; nguồn rỗng ⇒ không khối nguồn, không xin loại nghĩa; test cả hai biến thể.
- [x] `src-tauri/src/commands/proofread.rs` + `tests/ai_proofread_contract.rs` -- truyền nguồn; ca: prompt mang nguồn, phát hiện loại nghĩa ra dây đúng chuỗi, nguồn rỗng, nhãn lạ.
- [x] `src/config/proofread.ts`, `src/panels/proofreadHighlight.ts`, `GridPanel.vue` -- nhận năm loại, hai highlight, CSS `tm-rule`.
- [x] vitest -- adapter nhận loại nghĩa, highlight tách đúng khoá, khoá rỗng bị delete.
- [x] `e2e/specs/proofread-underline.e2e.mjs` -- thêm một cụm loại nghĩa, đo pixel `tm-rule` dưới cụm đó và `error` dưới cụm kia.
- [x] Đối chứng: gỡ thật ánh xạ nhãn lạ ⇒ `unlocated`, khối nguồn trong prompt, nhánh tách khoá highlight; ca canh tương ứng đỏ đúng lý do.

**Acceptance Criteria:**
- Given một segment dịch lệch nghĩa cố ý, when quét với provider giả trả phát hiện loại nghĩa, then gạch chân `tm-rule` hiện dưới đúng cụm (e2e) và một segment chưa dịch không sinh phát hiện loại này.
- Given một lỗi chính tả và một cụm thoát nghĩa trong cùng câu, when hiển thị, then hai gạch chân khác màu phân biệt được không cần đọc nhãn.
- Given cây nguồn, when chạy `ai_boundary`, then `core/segment` không import `core::ai` và allow-list của seam proofread không đổi.

## Implementation Notes

- Tập loại là `ProofreadKind` trong `core/segment/proofread_kind.rs`: chuỗi dây, `signature_scope`, `acceptance_origin`, và `needs_source`. `core/ai` ánh xạ nhãn model qua `from_wire`. Webview có `isMeaningKind` chỉ để chọn màu.
- `locate_findings` nhận thêm `has_source`, nên Rust (không phải webview) đếm vào `unlocated` một phát hiện loại nghĩa khi nguồn rỗng.
- Prompt có hai biến thể. Biến thể không nguồn giữ câu dẫn "spelling and grammar" của 9.1 (vá sau review).
- Bộ đầy đủ bắt được `naming_boundary` đỏ vì một biến `origin` trần trong bảng test, đổi thành `acceptance`.
- e2e (WKWebView, theme sáng): điểm màu `error` dưới cụm chính tả 0→16, điểm `tm-rule` dưới cụm nghĩa 0→12, điểm màu `error` dưới cụm nghĩa 0.
- Đối chứng, mỗi chỗ gỡ thật rồi khôi phục: ánh xạ nhãn lạ, lọc `has_source`, khối nguồn trong prompt, tách khoá ở `GridPanel.vue`, nhánh delete của `setHighlight`. Mỗi chỗ làm đỏ đúng ca canh của nó.
- Bộ đầy đủ chạy một lần: vitest 151 tệp, 2.108 ca; `cargo test` 88 binary, 0 lỗi. Sau hai bản vá review đã chạy lại `ai_proofread_contract`, `ai_boundary`, unit `proofread`, vitest proofread, `check:tokens` và `check:i18n`.
- Lượt quét với AI thật trong app thật trên một câu lệch nghĩa thuộc mục nợ `Chủ: Epic 9` sẵn có.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 0 high · 0 medium · 3 low · 3 false.
- false — task đối chứng "chưa có bằng chứng": pha Đo & đối chứng đã gỡ thật năm chỗ (ánh xạ nhãn lạ, lọc `has_source`, khối nguồn, tách khoá ở GridPanel, nhánh delete của `setHighlight`), mỗi ca canh đỏ đúng lý do rồi khôi phục; ghi ở Implementation Notes.
- false — AC "segment chưa dịch không sinh phát hiện loại này" thiếu ca: `prepare_proofread_call` trả `EmptyText` trước khi lắp prompt (ca `ai_proofread_contract.rs:274`, chạy trong bộ đầy đủ), nên không loại nào được xin.
- low · patch — `use …core::segment::proofread_kind` đứng trước `…core::aiconfig` trong `ai_proofread_contract.rs:19`.
- low · patch — câu dẫn của prompt không nguồn đổi từ "spelling and grammar errors" thành "errors", nới lời mời nhãn lạ cho lượt quét không nguồn.
- low · loại — nguồn chứa `SOURCE>>>`/`<<<TEXT` phá khung prompt: hiếm, sửa phải thêm thoát chuỗi; cùng hạng với hàng `TEXT>>>` đã loại ở 9.1.
- false — e2e không phân biệt hai màu: `errorPixels` lấy tổng chênh ba kênh ≤ 60, `error` và `tm-rule` cách nhau 209 (sáng) và 92 (tối); lượt đo cho 0 điểm màu `error` dưới cụm nghĩa.

## Design Notes

Một lời gọi cho mọi loại thay vì hai lượt gọi: FR82 tính chi phí theo lượt người dùng gọi, và cụm của cả hai họ đều đặt vào cùng `scanned_text`. Loại nghĩa là phán đoán so với nguồn, nên chữ ký ghi nhớ (9.6) mang ngữ cảnh và chấp nhận (9.4) đặt *người khác dịch* — 9.2 chỉ khai, không dùng. Tiêu chí *"gắn đúng segment, tham chiếu `segment.id`"* đã thoả ở dây 9.1 (lượt quét mang `segment_id`); quét cả Chương là 9.3.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test ai_proofread_contract`, `--test ai_boundary`, và unit test `core::segment`, `core::ai` -- xanh.
- `npx vitest run tests/frontend/proofread*.test.ts` -- xanh; `npm run check:tokens check:i18n` -- xanh.
- `npm run test:e2e -- --spec e2e/specs/proofread-underline.e2e.mjs` -- hai màu dưới hai cụm.

**Manual checks (if no CLI):**
- Lượt quét với AI thật trong app thật trên một câu lệch nghĩa: gộp vào mục nợ `Chủ: Epic 9` sẵn có cho ⌘P thật.
