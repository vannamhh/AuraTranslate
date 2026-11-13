---
title: 'Xuất bản dịch: đổi ngoặc kép thẳng thành cong ở hai bản xuất bản'
type: 'feature'
ticket: ''
created: '2026-11-13'
status: 'built'
baseline_revision: '1f62e2ff3145d47c6b0b94f4d308ed457512a30c'
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

**Problem:** Bản dịch mang dấu `"` (U+0022) thẳng, không phân biệt mở/đóng: 42/303 câu tiếng Trung trong `library-index.db` (41 do AI, `translation_origin = other`; 1 `self`). Nguyên văn có 0 dấu thẳng, font Noto Serif CJK TC vẽ `“ ”` cong đúng, nên chỗ hỏng là văn bản bản dịch.

**Approach:** Ở bước xuất, chỉ cho hai bản xuất bản không nhập lại được — `ExportFormat::DocxOneBlock` và `ExportFormat::Plain` (.txt) — đổi mỗi `"` trong văn bản bản dịch thành `“` hoặc `”` theo ký tự liền trước. Một ô chọn mới trong nhóm "Nội dung" của hộp thoại xuất, bật sẵn (Ice chọn phương án A + ô chọn bật sẵn, 2026-11-13).

## Boundaries & Constraints

**Always:**
- Không ghi gì xuống DB: `target_text` giữ nguyên (AD-47 không đụng).
- Luật chiều dấu xét TỪNG dấu `"`, theo ký tự liền trước trong cùng đoạn: không có ký tự trước (đầu đoạn), khoảng trắng (`char::is_whitespace`), hoặc một trong `( [ { « “ ‘ — – :` ⇒ `“`; còn lại ⇒ `”`. Không ghép cặp, nên số dấu lẻ không phải ca đặc biệt.
- Bản xem trước xuất hiện đúng văn bản sẽ ghi ra.
- Ô chọn tắt ⇒ tệp `.docx` một khối / `.txt` byte-identical với hôm nay.

**Never:**
- Không đổi ở `DocxTwoColumn` và `Markdown`: hai bản này nhập lại để rà soát (`commands/export.rs:580-590`) và phép so khớp với `target_text` sẽ báo sai "đã sửa" (phương án B, đã loại).
- Không đụng nguyên văn, tiêu đề Chương, dòng ghi xuất xứ, phụ lục Glossary, alt/caption ảnh, dấu nháy đơn `'`, và `“ ”` đã cong sẵn.
- Không đổi lúc gõ, không đổi kết quả AI, không đổi lúc nhập.

## I/O & Edge-Case Matrix

| Scenario | Input (bản dịch) | Expected Output | Error Handling |
|---|---|---|---|
| Sau dấu hai chấm + cách | `cho biết: "Chiến tranh pháp lý."` | `cho biết: “Chiến tranh pháp lý.”` | — |
| Đầu đoạn | `"Chiến tranh pháp lý" thường` | `“Chiến tranh pháp lý” thường` | — |
| Liệt kê | `đưa "A", "B" và "C"` | `đưa “A”, “B” và “C”` | — |
| Trong ngoặc đơn | `("A")` | `(“A”)` | — |
| Số dấu lẻ | `nói "xin chào` | `nói “xin chào` | — |
| Không có `"` | `Không có ngoặc.` | giữ nguyên | — |
| Đã cong / nháy đơn | `“A” và 'B'` | giữ nguyên | — |
| Bản rà soát, ô chọn bật | Markdown hoặc DocxTwoColumn | byte-identical với ô chọn tắt | Bỏ qua im lặng, không lỗi |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/export/options.rs:13-19` -- `ExportOptions` (`Default` + `#[serde(default)]`, mọi công tắc tắt mặc định, doc :11-12). `check_for` :86-92 là tiền lệ giới hạn theo định dạng (`mark_unconfirmed` báo lỗi); tuỳ chọn mới KHÔNG báo lỗi mà bỏ qua. `annotated()` :21-27 không đổi.
- `src-tauri/src/core/export/block_paragraphs.rs:111-143,161-168,180` -- `build_column` dựng cột bản dịch của `.docx` một khối; chỗ đổi là chuỗi sở hữu ở :133-136, CHỈ cột target. `load_chapter_blocks` đã nhận `options`.
- `src-tauri/src/core/export/docx_block.rs:28-53` -- ghi xuất xứ/tiêu đề/ô nguồn: không đụng.
- `src-tauri/src/core/export/text_export.rs:64-117,120,185,333` -- `emit_items` (dùng chung với Markdown), `load_chapter_text`, `LoadedText::preview`, `render_text`. Chỗ đổi phải biết định dạng là `Plain`; không đụng tiêu đề :349-353, xuất xứ :356-357, phụ lục :343-348, `one_line` :147-148.
- `src-tauri/src/core/export/preview.rs:50,94-127` -- `build_preview` nhận `format` + `content.options`; nhánh một khối đi qua `load_chapter_blocks`, nhánh text qua `load_chapter_text` + `loaded.preview`.
- `src-tauri/src/core/export/mod.rs` -- nơi khai báo module mới.
- `src/config/export.ts:14-18` -- kiểu `ExportOptions` TS.
- `src/exportState.ts:59-61,90-92,108-111,122-128,250-264,376-378` -- ref, `exportMarkAvailable` (mẫu cho hàm "chỉ hai định dạng xuất bản"), `currentExportOptions` (che như `mark_unconfirmed`), setter, reset.
- `src/ExportOverlay.vue:115-116,365-415` -- nhóm ô chọn "Nội dung"; mẫu ô `data-export-number-toggle` :367-376.
- `src/i18n/vi.json:735-746` -- khoá `export.content.*`; `check:i18n` cấm chuỗi tiếng Việt trong `.rs`/`.vue`.
- Test: `src-tauri/tests/export_content_options_contract.rs` (literal đủ trường :263; :377, :475 byte-identical, :617 preview theo định dạng); `tests/frontend/exportContent.test.ts:116-128,235-237`; hằng `OFF` ở `exportImages.test.ts:12`, `exportBlock.test.ts:12`, `exportDocx.test.ts:12`, `exportText.test.ts:11`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/export/quotes.rs` (mới) + `mod.rs` -- hàm thuần `curl_double_quotes(&str) -> Cow<str>` theo luật ở Always; trả `Borrowed` khi không có `"`.
- [ ] `src-tauri/src/core/export/options.rs` -- thêm `curly_quotes: bool` (Rust `Default` vẫn `false` để giữ mệnh đề byte-identical; UI gửi `true`); helper cho biết định dạng có áp dụng không (`DocxOneBlock | Plain`). Cập nhật doc comment :11-12 cho đúng bốn công tắc.
- [ ] `src-tauri/src/core/export/block_paragraphs.rs` -- áp hàm cho cột target khi `options.curly_quotes`.
- [ ] `src-tauri/src/core/export/text_export.rs` (+ `preview.rs` nếu cần) -- áp hàm cho đoạn bản dịch chỉ khi định dạng là `Plain`, ở cả `render_text` lẫn `preview`.
- [ ] `src-tauri/tests/export_content_options_contract.rs` -- thêm trường vào literal; ca ma trận I/O cho hàm; ca tệp `.txt` và `.docx` một khối có/không đổi; ca Markdown/hai cột byte-identical khi bật; ca preview khớp tệp.
- [ ] `src/config/export.ts`, `src/exportState.ts` -- trường mới; ref khởi tạo và reset về `true`; hàm `exportCurlyQuotesAvailable()`; `currentExportOptions` che `false` ngoài hai định dạng.
- [ ] `src/ExportOverlay.vue`, `src/i18n/vi.json` -- ô chọn `data-export-curly-quotes-toggle` cạnh ô đánh số câu, disabled ngoài hai định dạng kèm một dòng ghi chú; khoá `export.content.curly_quotes`, `…_desc`, `…_publish_only`.
- [ ] `tests/frontend/exportContent.test.ts` + bốn hằng `OFF` -- thêm trường; ca mặc định `true`, ca bị che ở Markdown/hai cột.

**Acceptance Criteria:**
- Given hộp thoại xuất vừa mở, when chọn `.txt` hoặc `.docx` một khối, then ô "ngoặc kép cong" đã tích và dùng được.
- Given chọn `.md` hoặc `.docx` hai cột, then ô đó bị vô hiệu kèm ghi chú, và lệnh gửi đi mang `curly_quotes: false`.
- Given một Chương có bản dịch `cho biết: "X."`, when xuất `.txt` với ô bật, then tệp chứa `cho biết: “X.”` và bản xem trước hiện y như vậy; `target_text` trong `project.db` vẫn chứa `"`.
- Given ô tắt, when xuất `.txt` hoặc `.docx` một khối, then tệp byte-identical với bản trước thay đổi.

## Implementation Notes

- `build_chapter_block` nhận cả `ExportOptions` thay cho `numbers` rời, để không vượt `clippy::too_many_arguments`.
- Ca unit của `quotes.rs` dùng fixture ASCII vì `check:i18n` cấm chuỗi tiếng Việt trong mã `.rs`; ca contract giữ câu tiếng Việt thật.
- Phát hiện ngoài phạm vi: `core::docx::read_element_text` bỏ qua `Event::GeneralRef` của quick-xml 0.41, nên `&quot; &amp; &lt; &gt;` trong `w:t` rơi im lặng khi đọc `.docx` (nhập Tác phẩm và nhập lại bản rà soát). Ghi nợ ở `deferred-work.md`.
- Mục nợ "mở bản `.docx` một khối bằng Word/Pages" đứng tên Ice vì Epic 12 (xuất) đã `done` cả 34 vé.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens quick) — high 0 · medium 0 · low 1 · false 5 · maybe-false 0.
- low · patch — preview `DocxOneBlock` không có ca bật `curly_quotes`; vá: ca một khối gọi thêm `export_preview` và assert dấu cong.
- false — đánh số + đổi dấu khác thứ tự ở hai nhánh: tiền tố `[n] ` kết bằng khoảng trắng nên cả hai thứ tự cho cùng kết quả, chính reviewer xác nhận.
- false — thiếu ca preview `.txt` khi tắt: `preview()` nhận `options.curls_quotes_in(format)`, tắt thì là `false` và đi nhánh `text.clone()`; không có kết cục xấu.
- false — doc comment `curl_double_quotes` vi phạm luật comment: câu "never paired, so an odd count only misdirects the quote it concerns" là bất biến không hiển nhiên từ chữ ký, được phép.
- false — `""` chưa có ca: luật xét ký tự GỐC liền trước ⇒ `“”`, đúng ý; thiếu ca không phải kết cục xấu.
- false — sổ sách plan (ô task, Implementation Notes): bản vá là sửa plan, bị loại theo luật triage.

## Design Notes

Tại sao xét ký tự liền trước thay vì ghép cặp: bản dịch AI thỉnh thoảng có số dấu lẻ trong một đoạn (đóng ở đoạn sau); ghép cặp thì một dấu lẻ làm lật chiều mọi dấu sau nó trong đoạn, còn luật ký tự liền trước chỉ sai cục bộ ở đúng dấu đó. Đây là cùng luật mà trình soạn thảo dùng cho smart quote.

Kiểm bằng mắt trong app thật (mở tệp `.docx` một khối bằng Word/Pages) là một mục nợ `Chủ: Epic N` của epic sắp đóng — xác định chủ lúc implement bằng `tickets.py`.

## Verification

**Commands:**
- `npm run build && cd src-tauri && cargo test --test export_content_options_contract` -- expected: xanh, có ca chạm tới nhánh `Plain` và `DocxOneBlock`.
- `npx vitest run tests/frontend/exportContent.test.ts tests/frontend/exportText.test.ts tests/frontend/exportBlock.test.ts tests/frontend/exportDocx.test.ts tests/frontend/exportImages.test.ts` -- expected: xanh, exit 0, không dòng `Errors`.
- `npm run check:i18n` -- expected: xanh.
- Đối chứng đỏ: gỡ lời gọi `curl_double_quotes` ở nhánh `Plain` (giữ chữ ký) ⇒ ca `.txt` của contract đỏ đúng lý do.
