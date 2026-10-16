---
title: 'Xuất: tuỳ chọn nội dung, khối ghi nguồn, xem trước'
type: 'feature'
ticket: '17'
created: '2026-10-16'
status: 'built'
baseline_revision: '3a481934010a8358df47b3ab102aa7a01d502291'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Màn Xuất chỉ có phạm vi, định dạng, ảnh và công tắc khối ghi nguồn kèm tên người dịch. Không chọn được đánh số câu, phụ lục thuật ngữ đã chốt hay dấu câu chưa xác nhận (FR152); bốn trường xuất xứ không hiện; danh sách Chương không có trạng thái; không có tên tệp, nút "Xuất N Chương ra …", Huỷ và ⌘↵; "Xem trước" là một hàng mẫu cố định, không đổi theo định dạng hay tuỳ chọn (c:176, c:180, c:181, c:184, c:185, c:186, c:189; C17 không gồm cột ghi chú trống, C18, C19).

**Approach:** Thêm ba tuỳ chọn nội dung vào lệnh xuất có sẵn (một kiểu `ExportOptions` dùng chung), hiện bốn trường khối ghi nguồn của Chương đầu phạm vi, và dựng xem trước từ dữ liệu thật của phạm vi theo định dạng đang chọn. Số câu và dấu ghép chữ thẳng vào hai ô sẵn có nên tệp đó không nhập lại được; tệp mang một dấu do bộ xuất ghi và đường nhập từ chối to tiếng khi gặp dấu đó.

## Decisions (Ice chọn 2026-10-16)

- Số câu và dấu chưa xác nhận ghép thành chữ trong hai ô sẵn có (không cột `#`, không đổi hình dạng bảng): tệp xuất có một trong hai thì KHÔNG nhập lại được. Mỗi câu mang tiền tố `[n] ` (n: thứ tự trong Chương trên các câu được xuất, 1-based); câu chưa xác nhận có thêm dòng `— câu chưa xác nhận —` cuối ô bản dịch. Không cần AD mới.
- Chống hỏng im lặng: bộ xuất ghi dấu máy đọc được vào tệp khi số câu hoặc dấu bật; đường nhập từ chối to tiếng bằng khoá lỗi `export.annotated_copy_not_reimportable`. Màn Xuất báo ngay lúc bật rằng tệp sẽ không nhập lại được.
- Số câu và phụ lục thuật ngữ áp cho `.docx` hai cột, `.docx` một khối, `.md`, `.txt`. Dấu chưa xác nhận chỉ `.docx` hai cột (FR121 không đổi); các định dạng khác từ chối `export.mark_two_column_only`.
- Phụ lục thuật ngữ = mọi mục đã chốt của hai tầng qua `confirmed_term_translations`, một lượt tải, viết dạng đoạn văn đầu tài liệu (không bảng).
- Xem trước dựng ở Rust bằng lệnh `export_preview`.

## Boundaries & Constraints

**Always:**
- Dấu máy đọc được: `.docx` ghi thuộc tính tuỳ chỉnh `aura.export_annotated=1` (`docProps/custom.xml`, `docx-rs` `custom_property`), `.md` ghi dòng đầu `<!-- aura:export-annotated -->`. Chỉ ghi khi số câu hoặc dấu bật; chỉ phụ lục thì không (phụ lục là đoạn văn, test canh nhập lại không sinh Chương).
- Mọi tuỳ chọn mặc định tắt; tắt cả ba thì tệp xuất giống từng byte so với trước story (FR131 giữ "mặc định tắt").
- Dấu câu chưa xác nhận chỉ ở bản hai cột; `.docx` một khối (FR121) không đổi hình dạng.
- "Chưa xác nhận" dùng đúng định nghĩa của `core/export/scope.rs` (`status <> 'confirmed'`, câu `is_omitted` và câu về hưu không tính), nên khớp số trên màn Xuất.
- "Đã chốt" của Glossary đi qua `is_confirmed()` / `confirmed_term_translations` (AD-36); không truy vấn thẳng bảng.
- Khối ghi nguồn vẫn dựng lúc xuất từ `chapter.origin_*` (AD-43); xem trước gọi cùng hàm dựng, không có chuỗi định dạng thứ hai ở webview. Rust trả dữ liệu, webview dựng chữ (AD-21).
- Chữ giao diện qua `vi.json`; nhãn ghi vào tệp xuất theo khuôn `attribution_lines` (`aura-allow-text`). Đóng lớp phủ trả tiêu điểm về `[data-export-open]` (UX-DR17).

**Never:**
- Không cột ghi chú trống cho Reviewer (loại khỏi C17).
- Không đổi nhận dạng hình dạng bảng của AD-38, không nới `read_docx_copy` nhận ô thứ ba, không đoán dấu từ chữ (tiền tố `[n]`) khi nhập.
- Không dựng lại bộ ghi `.docx`/`.md`/`.txt` (Epic 8); không migration; không phụ thuộc mới.
- Không lưu tuỳ chọn xuất xuống `project.db`/`config`: mỗi lần mở màn là mặc định tắt.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Bật cả ba, hai cột | 2 Chương, 5 câu chưa xác nhận, Glossary có thuật ngữ đã chốt | tệp có số câu, phụ lục thuật ngữ và dấu ở đúng 5 câu | — |
| Dấu ngoài hai cột | định dạng một khối, `.md` hoặc `.txt` | công tắc dấu tắt và ghi rõ lý do; tệp không có dấu | lệnh gọi thẳng với dấu bật: từ chối `export.mark_two_column_only`, không ghi tệp |
| Nhập lại tệp có số câu hoặc dấu | tệp `.docx` hoặc `.md` mang dấu của bộ xuất | từ chối, không ghi gì, không xem trước khớp | `export.annotated_copy_not_reimportable` |
| Bật số câu ở hai cột | công tắc số câu hoặc dấu bật trên `.docx` hai cột | màn báo tệp sẽ không nhập lại được | — |
| Chỉ phụ lục | phụ lục bật, số câu và dấu tắt | tệp vẫn nhập lại được, phụ lục không thành Chương | — |
| Không câu chưa xác nhận | mọi câu đã xác nhận | không dấu nào; công tắc hiện "0 câu" | — |
| Glossary rỗng | không thuật ngữ đã chốt | không phụ lục; công tắc tắt, ghi "chưa có thuật ngữ đã chốt" | — |
| Ghi nguồn thiếu trường | Chương đầu thiếu ngày đăng gốc | trường đó hiện "không có", khối xuất bỏ dòng đó | — |
| Xem trước | đổi định dạng hoặc bật tuỳ chọn | xem trước đổi theo, lấy tối đa vài câu đầu của Chương đầu phạm vi | lỗi đọc: câu lỗi trong khung xem trước, không chặn nút Xuất |
| Câu thuộc Chương bị cắt | `is_omitted` hoặc về hưu | không số câu, không dấu, không vào phụ lục | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/export.rs` -- `export_docx_two_column` L149, `export_docx_one_block` L257, `export_text` L298, vỏ IPC trong `mod wire` ~L940–1090, `export_scope_summary` L55, `attribution_of` L74. Thêm tham số `options: ExportOptions`; thêm lệnh `export_preview` (cùng `wire`). Đăng ký ở `src-tauri/src/lib.rs` L1208–1212.
- `src-tauri/src/core/export/table_rows.rs` -- `load_chapter_tables` đã có `segment.status` qua `ChapterSegment` (`commands/segment/chapter_read.rs:69`); `ExportRow::Text` cần mang số câu và cờ chưa xác nhận. `ExportCell`/`ChapterTable` là nơi thêm trường.
- `src-tauri/src/core/export/docx_table.rs` -- `row_of`/`write_two_column_docx`: chỗ vẽ số câu, dấu, phụ lục. `COLUMN_WIDTH_DXA` và `paragraph_of` dùng chung với `docx_block.rs` (không đổi).
- `src-tauri/src/core/export/text_export.rs` -- `load_chapter_text` L110, `render_text` L301: nhận số câu, phụ lục, và dòng dấu `.md` đầu tệp.
- `src-tauri/src/core/export/attribution.rs` -- `attribution_lines`, `load_origin`: nguồn duy nhất của khối ghi nguồn; xem trước dùng lại. Không đổi nhãn.
- `src-tauri/src/core/export/scope.rs` -- `count_scope`, `UNTRANSLATED_SQL`/`IN_TRANSLATION_SQL`: định nghĩa chưa xác nhận; `ScopeCounts.unconfirmed_count` đã có, webview dùng cho số "n câu".
- `src-tauri/src/core/glossary/store.rs` -- `confirmed_term_translations` L912 (cả hai tầng sau `ScopeResolver`); `confirmed_terms_for_injection` L1702 tải hai tầng mỗi lần gọi, đắt nếu gọi theo câu (không dùng; phụ lục dùng `confirmed_term_translations`).
- `src-tauri/src/core/export/reimport_gate.rs` -- `ReviewerDocx::admit` là cửa duy nhất của đường nhập `.docx` (`commands/export.rs:483`); thêm kiểm dấu ở đây và `ReimportShapeError::AnnotatedCopy` -> `IpcError` khuôn `PublishCopy`. Thêm cửa tương đương cho `.md` gọi ở `read_reviewer_file` L456 trước `read_markdown_copy`. `core/i18n/mod.rs:933` (`MessageKey`) và `vi.json:308` là khuôn khoá lỗi.
- `src-tauri/src/core/docx/mod.rs` -- `read_docx` L170 chưa đọc `docProps/custom.xml`; thêm đọc thuộc tính tuỳ chỉnh (bằng `read_zip_entry` + `quick-xml`, không `docx_rs::read_docx`) vào `DocxParsed`. Struct có literal ở test/fixture: tìm và cập nhật.
- `src-tauri/src/core/export/reviewer_copy.rs` -- `read_docx_copy` L112, `docx_score` L380: KHÔNG đổi. Chúng là thứ dấu bảo vệ.
- `src/ExportOverlay.vue` (594 dòng) -- khối `data-export-attribution` L317–343, khối xem trước `ex-preview` L398–411 (một hàng mẫu), nhãn xem trước `vi.json` `export.preview.*`. Thêm fieldset "Nội dung" và bốn trường; thay xem trước.
- `src/exportState.ts` (271 dòng) -- `attribution` L42, `writeFormat` L216, `runExport` L231, `resetExport` L249 (`check:panel-refs`: reset trạng thái mới). Thêm trạng thái ba tuỳ chọn và xem trước.
- `src/config/export.ts` -- adapter `exportDocxTwoColumn`/`exportDocxOneBlock`/`exportText` (thêm tham số), adapter `exportPreview` mới và guard kiểu.
- `src/config/chapter.ts` -- `ChapterRow` đã mang `origin_author/site_name/url/published_at`; bốn trường hiện từ đây, không lệnh mới.
- Mockup `ux-auratranslate/mockups/export-share.html` L192–226 (mục Nội dung) và L240–285 (file thật, `.md`), `export-images-attribution.html` L145–185 (bốn trường + xem trước khối). Đọc lại khi dựng.
- Test sẽ dời: `src-tauri/tests/export_docx_contract.rs`, `export_block_contract.rs`, `export_text_contract.rs`, `export_attribution_contract.rs`, `reviewer_import_contract.rs`, `alignment_contract.rs`, `reimport_gate_contract.rs`, `config_invariants.rs` (đổi chữ ký hàm xuất), `ipc_contract.rs`, `ipc_argument_contract.rs` (lệnh mới); `tests/frontend/exportDocx.test.ts`, `exportBlock.test.ts`, `exportText.test.ts`, `exportAttribution.test.ts`, `exportScope.test.ts`.

## Tasks & Acceptance

**Execution:**

Pha 1 -- Rust:
- [ ] `core/export/options.rs` (mới) + `mod.rs` -- `ExportOptions { number_segments, glossary_appendix, mark_unconfirmed }` (`Deserialize`, mặc định tắt); kiểm dấu ngoài hai cột trả `export.mark_two_column_only` -- FR152, FR121
- [ ] `core/export/table_rows.rs`, `block_paragraphs.rs`, `text_export.rs` -- tiền tố `[n] ` theo thứ tự trong Chương; cờ `unconfirmed` cho hai cột; phụ lục thành đoạn đầu tài liệu -- C17
- [ ] `core/export/docx_table.rs`, `docx_block.rs` -- ghi dòng dấu, phụ lục, thuộc tính `aura.export_annotated`; `.md` ghi dòng đầu dấu -- c:180
- [ ] `core/docx/mod.rs`, `core/export/reimport_gate.rs`, `core/i18n/mod.rs`, `commands/export.rs` -- đọc thuộc tính, `AnnotatedCopy`, cửa `.md`, khoá lỗi mới, gọi cả hai cửa ở `read_reviewer_file`
- [ ] `commands/export.rs` + `lib.rs` -- tham số `options` cho ba lệnh xuất; phụ lục qua `confirmed_term_translations` chỉ khi bật; lệnh `export_preview(scope, format, options, attribution)` trả vài hàng đầu Chương đầu phạm vi, `attribution_lines` của Chương đó và tên tệp sẽ ghi -- c:180, c:186, C18, C19
- [ ] `src-tauri/tests/*` -- cập nhật chữ ký; ca mới: cả ba bật có số câu, phụ lục, dấu đúng 5/5 câu; một khối/`.md`/`.txt` + dấu bị từ chối và không ghi tệp; tắt hết thì tệp trùng byte bản cũ; câu `is_omitted`/về hưu không số, dấu, phụ lục; nhập lại `.docx` và `.md` có số câu hoặc dấu bị từ chối với khoá mới và không đổi gì trong `project.db`; chỉ-phụ-lục nhập lại không sinh Chương; xem trước trùng tệp xuất ở các hàng đầu. Đối chứng đỏ: (1) gỡ thật kiểm dấu trong `admit`, ca nhập lại `.docx` phải đỏ vì tiền tố số làm cột trái lệch; (2) gỡ thật cửa `.md`, ca `.md` đỏ; (3) gỡ thật hàm lọc "chưa xác nhận" -- AGENTS "Counter-check"

Pha 2 -- Webview:
- [ ] `src/config/export.ts`, `src/exportState.ts` -- tham số `options`, trạng thái ba công tắc, `exportPreview` gọi lại khi đổi định dạng, phạm vi hay công tắc (bỏ kết quả cũ khi đến muộn); `resetExport` reset hết
- [ ] `src/ExportOverlay.vue`, `src/i18n/vi.json` -- fieldset "Nội dung": ba công tắc, mỗi cái một dòng mô tả theo mockup; công tắc dấu tắt kèm lý do khi định dạng không phải `.docx` hai cột; số "n câu" từ `unconfirmed_count`; khi số câu hoặc dấu bật hiện `data-export-not-reimportable` "tệp này không nhập lại được" (nhãn qua `vi.json`, qua `check:i18n`), cùng khuôn ghi chú FR121 sẵn có -- C17
- [ ] `src/ExportOverlay.vue` -- bốn trường ghi nguồn và dòng Người dịch (badge Toàn cục) khi công tắc bật, lấy từ Chương đầu phạm vi, trường trống ghi "không có" -- c:186, C19
- [ ] `src/ExportOverlay.vue` -- xem trước theo định dạng (bảng hai cột, một khối, `.md`/`.txt` dạng khối chữ) với nhãn "Xem trước — Chương …", tên tệp từ kết quả xem trước (bản xem trước có cả tiền tố số câu, dấu, phụ lục khi bật), nút "Xuất N Chương ra .ext", Huỷ, gợi ý phím ⌘↵ chạy `export.run` -- c:180, c:181, c:189, C18
- [ ] `src/ExportOverlay.vue` -- hàng Chương kèm trạng thái (từ `ChapterRow`), danh sách ảnh thiếu link theo từng ảnh (`missingLabel` đã có: chỉ kiểm khớp mockup), màu `ex-warning` theo token mockup (`tm-rule`, không `error`; nếu cần ảnh màu thật thì ghi nợ `Chủ: Ice`) -- c:176, c:184, c:185
- [ ] `tests/frontend/export*.test.ts` -- công tắc, tắt dấu ngoài hai cột, báo không nhập lại được khi bật số câu hoặc dấu, xem trước đổi theo định dạng, trường trống, kết quả xem trước đến muộn bị bỏ

Pha 3 -- Test dời và kiểm tay:
- [ ] `e2e/specs/story-12-17-export-content-options.e2e.mjs` (mới) -- mở `Xuất…` từ thanh hành động, bật ba tuỳ chọn, xem trước đổi; tiêu điểm trả về `Xuất…`
- [ ] `deferred-work.md` -- ghi nợ kiểm tay mở tệp `.docx` trong Word `Chủ: Epic 12` (Windows: `B7`); màu thật của `ex-warning` nếu cần ảnh

**Acceptance Criteria:**
- Given một Tác phẩm hai Chương có câu chưa xác nhận và thuật ngữ đã chốt, when xuất `.docx` hai cột với cả ba tuỳ chọn, then tệp có số câu, phụ lục thuật ngữ và dấu đúng ở các câu chưa xác nhận.
- Given cùng phạm vi, when xuất `.docx` một khối, then tệp không có dấu chưa xác nhận và hình dạng một hàng một bảng của FR121 giữ nguyên.
- Given `.docx` một khối, `.md` và `.txt`, when bật số câu và phụ lục, then tệp có số câu và phụ lục; bật dấu bị từ chối.
- Given cả ba tuỳ chọn tắt, when xuất ở bốn định dạng, then tệp trùng byte với bản xuất trước story.
- Given công tắc khối ghi nguồn bật, when nhìn màn Xuất, then thấy tác giả, báo/website, URL, ngày đăng gốc của Chương đầu phạm vi và tên người dịch, trường trống ghi "không có".
- Given màn Xuất đang mở, when đổi định dạng hoặc bật tuỳ chọn, then xem trước đổi theo và khớp các hàng đầu của tệp xuất ra.
- Given màn Xuất với `.docx` hai cột, when bật số câu hoặc dấu, then màn nói ngay tệp sẽ không nhập lại được; tắt cả hai thì lời báo biến mất.
- Given tệp `.docx` hoặc `.md` đã xuất với số câu hoặc dấu, when chọn nó ở Nhập bản reviewer, then nhận lỗi `export.annotated_copy_not_reimportable`, không có hàng nào được nhập và Tác phẩm không đổi.
- Given tệp xuất chỉ với phụ lục thuật ngữ, when nhập lại, then nhập như thường và phụ lục không thành Chương.

## Implementation Notes

- Số câu là thứ tự của câu trong Chương trên cùng tập với `segment_count`, nên một câu mang một số ở mọi định dạng; ở một khối, `.md` và `.txt`, câu đích rỗng và câu `alt`/`caption` không in tiền tố nên dãy số nhảy (Ice chọn giữ, 2026-10-16).
- Dấu không nhập lại được là thuộc tính tuỳ chỉnh `aura.export_annotated` trong `.docx` và dòng đầu `<!-- aura:export-annotated -->` trong `.md`, chỉ ghi khi số câu hoặc dấu bật; chỉ đo vòng tròn `docx-rs` → `core::docx`, Word giữ thuộc tính hay không là nợ `Chủ: Epic 12`.
- `appendix_term_count` của xem trước đếm thuật ngữ đã chốt cả khi công tắc phụ lục tắt, để màn khoá công tắc khi Glossary rỗng; lệnh xuất vẫn không đọc Glossary khi tắt.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 6 · false 0 · maybe-false 1; thêm 1 ca bị bác vì sửa nằm ở plan.
- medium · patch: Glossary rỗng thì công tắc phụ lục vẫn bật được và câu "chưa có thuật ngữ đã chốt" chỉ hiện sau khi bật, trái hàng ma trận "công tắc tắt, ghi …"; `appendix_term_count` chỉ có khi công tắc bật.
- low · patch: khoá `export.content.glossary_appendix_count` không ai dùng; dùng nó cho số thuật ngữ cạnh công tắc phụ lục.
- low · patch: xem trước `.md` trả đoạn chưa thoát (`[1] …`) trong khi tệp ghi `\[1\] …`, nên không khớp tệp.
- low · patch: AC "tắt hết, bốn định dạng trùng byte" chỉ canh `.md`; `.txt` và `.docx` một khối không có assert nào.
- low · patch: câu `is_omitted`/về hưu không lấy số chỉ được assert ở `.docx`; `.md`/`.txt` đi đường `emit_items` chưa có assert.
- low · patch: đổi định dạng thì hàng và tên tệp xem trước cũ còn hiện tới khi lệnh mới trả, lệch nhãn nút đã theo định dạng mới.
- low · patch: doc comment `ExportFormat::stem_suffix` kể lại chữ ký (luật chú thích AGENTS.md).
- low · reject: xem trước hai cột bỏ hàng ảnh; thêm hàng ảnh cần mở rộng kiểu `ExportPreviewRow`, người dùng gần như không gặp.
- low · reject: `is_unconfirmed` lặp `status <> 'confirmed'` của SQL; đã có ca `unconfirmed_count_on_the_export_screen_equals_the_marks_in_the_file` buộc hai chỗ khớp.
- maybe-false · defer: `export_preview` là lệnh plain, giữ khoá `OpenWorkState` và đọc trọn Chương đầu mỗi lần bật công tắc; chưa đo trên Chương 2.000 câu xem có giật UI không.
- reject (sửa plan): ba phép đối chứng đỏ không ghi ở Implementation Notes; đã có ở tệp handoff và sẽ vào commit message.

## Design Notes

- Bốn trường ghi nguồn chỉ đọc ở màn Xuất (sửa tại chỗ ở FR128); với phạm vi nhiều Chương hiện Chương đầu như mockup ("Chương 3"), tệp vẫn mang khối của từng Chương.
- Dấu là thuộc tính tuỳ chỉnh `.docx` / dòng `.md` do bộ xuất kiểm soát, không đoán từ chữ. Tồn dư chấp nhận: người dùng chép nội dung sang tệp mới làm mất dấu và tiền tố `[n]` thành văn bản; cùng hạng ca sót với AD-38. Word và LibreOffice giữ `docProps/custom.xml` khi lưu (chưa đo, pha Rust đo trước bằng một tệp thật).
- Số câu là thứ tự trong Chương trên các câu được xuất, 1-based; mockup đánh số liên tục (86, 87…) nhưng không nói gốc, và lấy gốc theo Chương tránh phụ thuộc `segment.id`.
- Phụ lục viết dạng đoạn "thuật ngữ = bản dịch", vì `read_docx_copy` coi mọi bảng là một mục Chương.
- Ước lượng: Rust ~280 dòng, webview ~320, test ~450; tổng ~1.050 nên `full`.

## Verification

**Commands:**
- `npm run test:story 12-17` -- expected: các test được chọn xanh
- `cargo test --manifest-path src-tauri/Cargo.toml --test export_docx_contract --test export_block_contract --test export_text_contract --test reviewer_import_contract` -- expected: xanh (sau `npm run build`)
- Đổi chữ ký lệnh và `lib.rs` là dây dùng chung: một lượt `cargo test` đầy đủ trước `done`.

**Manual checks (if no CLI):**
- Mở tệp `.docx` hai cột có cả ba tuỳ chọn bằng Word: số câu, phụ lục và dấu đọc được; nhập lại tệp đó làm bản reviewer.
