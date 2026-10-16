# Handoff: Xuất tuỳ chọn nội dung, khối ghi nguồn, xem trước

## Pha 1 (Rust)

Trạng thái: Rust xong, chưa commit. Một test cố ý còn đỏ cho pha Webview (xem dưới).

### Wire (Rust -> webview)

Ba lệnh xuất có thêm tham số `options`, đứng ngay trước `attribution` (tên khoá IPC camelCase: `options`):

- `export_docx_two_column(scope, imageMode, options, attribution, folder)`
- `export_docx_one_block(scope, imageMode, options, attribution, folder)`
- `export_text(scope, imageMode, format, options, attribution, folder)`

`options` = `{ number_segments: bool, glossary_appendix: bool, mark_unconfirmed: bool }`, mọi trường mặc định `false` (`#[serde(default)]`, nên `{}` hợp lệ). Tên trường trong object là snake_case (không `rename_all`).

Lệnh mới `export_preview(scope, format, options, attribution: bool)` (đồng bộ, không `(async)`):

- `format`: chuỗi `"docx_two_column" | "docx_one_block" | "markdown" | "plain"` (kiểu `ExportFormat`, khác `TextFormat` của `export_text`).
- Trả `ExportPreview`:
  `{ chapter_id, chapter_ord, chapter_title: string|null, file_name, attribution: string[], appendix: string[], appendix_term_count, not_reimportable, rows: { source: string|null, target: string }[] }`.
- `rows`: tối đa 6 (`PREVIEW_LIMIT`), từ Chương ĐẦU của phạm vi, chỉ hàng chữ (không ảnh).
  - hai cột: mỗi câu một hàng, `source` có giá trị; chữ đã mang tiền tố `[n] ` và dòng `— câu chưa xác nhận —` khi bật.
  - một khối: đúng MỘT hàng, `source`/`target` là tối đa 6 đoạn nối bằng `\n\n`.
  - `.md`/`.txt`: mỗi đoạn một hàng, `source = null`, chữ thô (chưa thoát Markdown).
- `attribution`: dòng ghi nguồn của Chương đầu (rỗng khi `attribution=false`); `chapter_title` null thì webview tự ghi "Chương {chapter_ord}".
- `appendix`: dòng tiêu đề "Thuật ngữ đã chốt" + tối đa 5 dòng `thuật ngữ = bản dịch`; `appendix_term_count` là tổng số thuật ngữ. Rỗng khi công tắc tắt hoặc Glossary chưa có mục đã chốt.
- `file_name`: tên cơ sở không xử lý trùng (`<Tác phẩm>-hai-cot.docx`, `-mot-khoi.docx`, `<Tác phẩm>.md|.txt`); lúc ghi thật có thể thành `... (2)`.
- `not_reimportable` = số câu hoặc dấu bật (phụ lục không tính). Webview dùng thẳng cho `data-export-not-reimportable`.
- Lỗi: `export.mark_two_column_only` khi `mark_unconfirmed` mà `format != docx_two_column` (cả preview lẫn ba lệnh xuất; không ghi tệp); còn lại như `export_scope_summary` (`work.none_open`, `export.scope_empty`, ...). Phụ lục bật mà chưa mở Tác phẩm cũng ra `work.none_open`.

Khoá lỗi mới (đã vào `MessageKey`, `vi.json`):

- `export.annotated_copy_not_reimportable` (`err.export.annotated_copy_not_reimportable`): `reviewer_import_open_preview` trả khi tệp `.docx`/`.md` mang dấu của bộ xuất.
- `export.mark_two_column_only` (`err.export.mark_two_column_only`).

Webview chưa có chuỗi nào cho hai khoá này ngoài `err.*` (thông báo lỗi lấy qua `message_key`). Phần chữ giao diện cho công tắc/mô tả/lời báo "không nhập lại được" là việc của pha Webview.

### Hành vi cần biết khi dựng UI

- Số câu = thứ tự 1-based của câu trong Chương trên các câu THUỘC BẢN DỊCH (không `is_omitted`, không về hưu), cùng tập với `segment_count`. Hai cột: cả cột nguồn lẫn cột đích mang `[n] ` (cột đích bỏ tiền tố khi đích rỗng). Một khối/`.md`/`.txt`: tiền tố ở mảnh đầu của mỗi câu có chữ; câu đích rỗng hoặc câu vai `alt`/`caption` không sinh tiền tố nên số có thể nhảy cách (chấp nhận, chưa hỏi Ice).
- Dấu = thêm dòng `— câu chưa xác nhận —` cuối ô đích (xuống dòng trong cùng đoạn); câu đích rỗng chỉ có dòng dấu. Định nghĩa chưa xác nhận dùng `core::export::is_unconfirmed` (`status != 'confirmed'`), cùng vị từ với `count_scope`, nên khớp `unconfirmed_count` (có test).
- Phụ lục = đoạn văn đầu tài liệu (dòng tiêu đề + mỗi thuật ngữ một dòng, sắp theo thuật ngữ, qua `confirmed_term_translations`, một lượt tải). `.md` KHÔNG dùng `##` cho tiêu đề phụ lục để khỏi thành Chương khi nhập lại.
- Dấu máy đọc: `.docx` ghi thuộc tính tuỳ chỉnh `aura.export_annotated=1`; `.md` ghi dòng đầu `<!-- aura:export-annotated -->`. Chỉ ghi khi số câu HOẶC dấu bật (`.txt` không ghi, vì không nhập lại được bằng đường nào).
- Mọi công tắc tắt: `document_for` trả `Docx::new()` trơn, `render_text` không thêm gì, `row_of` giữ chữ nguyên. Không có test so byte với bản trước story (docx-rs luôn ghi `docProps/custom.xml` rỗng nên không thể kiểm bằng "không có mục"); test kiểm md đúng chuỗi cũ và docx không có dấu/phụ lục/tiền tố.

### Chữ ký Rust thay đổi

Hàm thuần nhận `content: &ExportContent` (= `{ options, appendix: Vec<(String,String)> }`), đứng trước `attribution`. `ExportContent` dựng bằng `commands::export::export_content_of(open, options, global)` (đây là chỗ duy nhất đọc Glossary, và chỉ khi `glossary_appendix` bật). Test truyền `&ExportContent::default()`.

Loader nhận `options: ExportOptions` ở cuối (`load_chapter_tables`, `load_chapter_blocks`, `load_chapter_text`); writer nhận `content` ở cuối (`write_two_column_docx`, `write_one_block_docx`, `render_text`). `emit_items` thêm `numbers` (alignment truyền `None`). Cửa nhập: `ReviewerDocx::admit` kiểm `DocxParsed.export_annotated` đầu tiên; `admit_markdown(&str)` mới, gọi ở `read_reviewer_file` trước `read_markdown_copy`.

### Đo và test

- Đo vòng tròn `custom.xml` (bằng bộ ghi `docx-rs` -> `core::docx::read_docx` của repo, test `the_custom_property_survives_the_exporter_and_is_read_back_as_a_flag`): `docProps/custom.xml` có `aura.export_annotated` và `read_docx` đọc ra `export_annotated = true`. CHƯA đo bằng Word thật hay LibreOffice (máy không có `soffice`): việc "Word/LibreOffice giữ `custom.xml` khi lưu" vẫn mở, là mục kiểm tay `Chủ: Epic 12` ở pha Tests (và nếu mất dấu thì rơi vào ca sót đã chấp nhận ở Design Notes).
- Test mới: `src-tauri/tests/export_content_options_contract.rs` (9 ca xanh): cả ba bật 5/5 câu, một khối/`.md`/`.txt` + dấu bị từ chối và không ghi, nhập lại `.docx` + `.md` bị từ chối và `project.db` không đổi, chỉ-phụ-lục nhập lại không thành Chương (cả hai định dạng), tắt hết, Glossary rỗng, xem trước khớp tệp xuất ở hàng đầu (hai cột, `.md`, một khối), `unconfirmed_count` = số dấu.
- Đối chứng đỏ (gỡ thật, rồi khôi phục): (1) xoá kiểm dấu trong `admit` -> ca nhập lại đỏ; (2) xoá `admit_markdown` ở `read_reviewer_file` -> cùng ca đỏ (ca đó gộp `.docx` và `.md` nên (1) và (2) đỏ ở cùng một test; pha Tests có thể tách làm hai ca nếu muốn đỏ riêng); (3) `is_unconfirmed` luôn sai -> 2 ca đỏ.
- Chữ ký đã cập nhật ở: `export_docx_contract`, `export_block_contract`, `export_text_contract`, `export_attribution_contract`, `reimport_gate_contract`, `alignment_contract`, `review_harvest_contract`, `reviewer_import_contract` (đều xanh), `config_invariants` (bảng `COMMAND_FILE_CENSUS` export.rs 7->8 plain, tổng 100->101 plain).
- Sàn dân số: hai tệp src mới đẩy số tệp `.rs` dưới `src-tauri/src/**` lên 143 nên 20 test boundary có sàn 113 tự đỏ (`ceil(0.85 x 143) = 122`); đã nâng đồng loạt 113->122 (`*_FLOOR` ở 20 tệp `src-tauri/tests/*boundary*`, `segment_baseline_guard`, `config_invariants`, `naming_boundary`...). Pha Tests đừng coi đây là diff lạ.
- `cargo clippy --all-targets -- -D warnings` xanh, `cargo fmt --check` xanh, `check:commands|layout|tokens|doc-refs|panel-refs` xanh.

### Còn ĐỎ cho pha Webview

- `ipc_argument_contract::every_registered_commands_business_parameters_match_camel_case_across_the_wire` đỏ với `["export_preview"]`: lệnh đã đăng ký ở `lib.rs` nhưng chưa có `invoke('export_preview', ...)` trong `src/**/*.ts`. Xanh khi `src/config/export.ts` thêm adapter `exportPreview` (tham số `scope`, `format`, `options`, `attribution`). Ba adapter xuất phải thêm `options` cùng lúc (tên đối số IPC phải khớp `options`).
- Chưa chạy: toàn bộ `cargo test` (chỉ chạy các target liên quan; `lib.rs` đổi nên AGENTS yêu cầu một lượt đầy đủ trước `done`), vitest, e2e.

## Pha 2 (Webview)

Trạng thái: Webview xong, chưa commit. Không đụng Rust.

### Đã làm
- `src/config/export.ts`: kiểu `ExportOptions`, `ExportPreview*`; ba adapter xuất nhận `options` làm đối số CUỐI (sau `attribution`), gửi khoá IPC `options`; adapter mới `exportPreview(scope, format, options, attribution)` có guard kiểu.
- `src/exportState.ts`: ba ref công tắc (+ setter `setExportNumberSegments|GlossaryAppendix|MarkUnconfirmed`), `currentExportOptions()` (dấu chỉ gửi khi định dạng là `docx_two_column`), `exportMarkAvailable()`, `exportNotReimportable()` (số câu hoặc dấu hiệu lực; phụ lục không tính), `refreshExportPreview()` với `previewSequence` bỏ kết quả đến muộn, `exportFirstScopeChapter()`. Xem trước gọi lại khi đổi định dạng/phạm vi/công tắc/ghi nguồn. `resetExport` reset hết.
- `src/ExportOverlay.vue`: fieldset `data-export-content` (ba công tắc, mô tả, "n câu", ghi chú vì sao dấu bị tắt, `data-export-not-reimportable`); bốn trường ghi nguồn `data-export-attribution-field=author|site|url|published` ("không có" khi trống); hàng Chương kèm `n câu · trạng thái`; xem trước theo định dạng (bảng hai cột / một khối / khối chữ `data-export-text-preview`), tên tệp `data-export-file-name`, nút `data-export-run` "Xuất N Chương ra .ext", Huỷ `data-export-cancel`, gợi ý ⌘↵ và phím Cmd/Ctrl+Enter (`runShortcut`) chạy `export.run`; `ex-warning` đổi sang vạch `--color-tm-rule` (không còn `error`).
- `src/i18n/vi.json`: khoá `export.content.*`, `export.attribution.field_*`, `export.chapter.*`, `export.run.button|cancel|shortcut|file_name`, `export.preview.*`; bỏ `export.preview.source_sample|target_sample`.
- Cổng: `scripts/check-commands.mjs` thêm `runShortcut` vào HANDLER_TABLE và nâng `HANDLER_ATTR_FLOOR` 122->131; `scripts/check-i18n.mjs` `RS_FLOOR` 114->123 (sàn trôi do hai tệp Rust của Pha 1, không phải do Webview).
- Test: `tests/frontend/exportContent.test.ts` mới (11 ca); sáu tệp `export*.test.ts` cũ: mock thêm `exportPreview`, `toHaveBeenCalledWith` thêm đối số `options` tắt hết, đếm checkbox 4->7, ca xem trước Word mock một hàng.

### Cho pha Tests
- Chưa đo bằng mắt/e2e. Selector ổn định cho e2e: `[data-export-number-toggle]`, `[data-export-appendix-toggle]`, `[data-export-mark-toggle]`, `[data-export-word-preview]`, `[data-export-text-preview]`, `[data-export-run]`, `[data-export-cancel]`. Hộp thoại chọn thư mục là gốc nên nút Xuất không bấm được trong e2e.
- Hạn chế do hợp đồng Rust: `appendix_term_count` chỉ có khi công tắc phụ lục BẬT (tắt thì danh sách rỗng), nên không tắt công tắc trước được khi Glossary rỗng; webview chỉ hiện ghi chú "chưa có thuật ngữ đã chốt" sau khi bật. Muốn tắt trước thì Rust phải trả số thuật ngữ cả khi tắt (không sửa ở pha này).
- Màu thật của `ex-warning` (vạch `tm-rule`) chưa nhìn bằng mắt: ghi nợ nếu cần.
- Ca chưa có: không test cho nút ⌘↵ chạy thật (dispatch cần registry; HANDLER_TABLE ghi `ids: ['export.run']`).
- Chưa chạy: cargo test đầy đủ, vitest đầy đủ, e2e. Đã chạy: vitest tests/frontend/export*, vue-tsc, eslint, các check:* (commands, i18n, tokens, layout, doc-refs, panel-refs, lint) và cargo test --test ipc_argument_contract (xanh).

## Pha 3 (Tests)

- Ma trận I/O: mọi hàng đã có ca phủ sẵn từ Pha 1/2 (omitted/về hưu, Glossary rỗng, chỉ phụ lục, tắt hết, dấu ngoài hai cột, xem trước, trường trống); không thêm ca mới ngoài việc dưới.
- Tách ca nhập lại thành hai (`export_content_options_contract.rs`): `.docx` và `.md` nay đỏ riêng khi gỡ (1) và (2). Production không đổi ở pha này.
- Đối chứng đỏ bằng gỡ thật (target `export_content_options_contract`, khôi phục bằng `cmp` với bản sao): (1) xoá `if parsed.export_annotated` trong `admit` -> ca `.docx` đỏ, ca `.md` xanh; (2) thay `admit_markdown(...)` ở `read_reviewer_file` bằng `text.as_str()` (giữ kiểu) -> ca `.md` đỏ, ca `.docx` xanh; (3) `is_unconfirmed` luôn `false` -> 2 ca đỏ (cả ba bật, và `unconfirmed_count` = số dấu).
- e2e mới `e2e/specs/story-12-17-export-content-options.e2e.mjs` (eslint xanh, CHƯA chạy: webview đứng ở `tauri://localhost`). Hộp chọn thư mục là gốc nên không bấm Xuất.
- Nợ kiểm tay ghi ở cuối `deferred-work.md` (`Chủ: Epic 12`, Windows `B7`): mở `.docx` trong Word, thuộc tính `aura.export_annotated` còn sau khi Word lưu, màu `ex-warning`.
- Lượt đầy đủ một lần: `npm run build` xanh; 13/14 cổng xanh; `check:doc-refs` ĐỎ vì `FILE_FLOOR` 582 < 80% của 728 tệp thật (cổng nói nâng lên 619) do các tệp mới của story; chưa nâng vì hành động bị chặn, cần Ice/pha sau sửa `scripts/check-doc-refs.mjs` dòng 127. vitest: 185/198 tệp xanh, 13 tệp (23 ca) đỏ vì quá hạn 5 s khi máy tải (transform 47 s); chạy lại đúng 13 tệp đó: 13/13, 472 ca xanh. `cargo test` toàn bộ: 97 target, 2.606 ca xanh, 0 đỏ, 27 ignored.
