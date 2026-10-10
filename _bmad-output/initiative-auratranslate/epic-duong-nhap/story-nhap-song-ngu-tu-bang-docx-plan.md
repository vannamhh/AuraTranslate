---
title: 'Story 6.16c — Nhập song ngữ từ bảng .docx'
type: 'feature'
ticket: '16c'
created: '2026-10-10'
status: 'built'
baseline_revision: '3f7894ef97166c6e04caf14bb6f6976f40afa99a'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR115 — đường song ngữ 6.16 chỉ nhận `.csv`/`.tsv`; Ice giữ `.docx` (phiếu quyết #23) nhưng đầu vào của đường đó là byte + dấu phân cách, còn bộ đọc `.docx` đã giao hàng.

**Approach:** Một đầu vào "hàng đã có" trên cùng pipeline AD-39 nhận hàng của một bảng `.docx` thay cho giải mã + `parse_rows`; người dùng chọn bảng ở màn xem trước.

### Decisions (Ice, 2026-10-10)

- **Nhiều bảng ⇒ người dùng chọn.** Màn xem trước liệt kê các bảng, mặc định "chưa chọn", xác nhận bị khoá tới khi chọn. `table_index` đi qua ba lệnh wire, kèm bộ chọn ở webview.
- **Bảng lồng KHÔNG tính.** Chỉ bảng cấp cao nhất của thân tài liệu vào danh sách; bảng lồng không bao giờ gây từ chối.
- **Hàng đầu:** dùng lại ô "hàng đầu là tiêu đề cột" của 6.16, mặc định TẮT; không đổi mã.
- **Không dùng `ReviewerDocx::admit` (AD-38).** Bảng một hàng nhiều đoạn hiện ở danh sách hàng lệch cặp; `core::segment` không phụ thuộc `core::export`.
- **Chỉ bảng ≥ 2 cột vào danh sách.** Bảng cấp cao có hàng rộng nhất dưới 2 ô (cùng luật "ít hơn 2 cột" của 6.16) không được liệt kê; tệp chỉ có bảng như vậy bị từ chối "không có bảng hai cột", không phải `too_few_columns`.

## Boundaries & Constraints

**Always:**
- AD-39: `PIPELINE_ORDER` và chỗ gọi `run_import` không đổi; `.docx` bỏ qua giải mã bảng mã.
- Từ 6.16 giữ nguyên: cờ theo hàng (AD-37/46), `target_text` + `bilingual_import` cùng một INSERT (AD-47 ③), `draft`, Chương `in_progress`, hàng lệch cặp khoá xác nhận, 0 byte xuống đĩa trước xác nhận.
- Không chọn bảng im lặng: 0 bảng, `table_index` lệch/hết hạn ⇒ lỗi có kiểu nêu tên lý do; không bao giờ lùi về bảng 1 hay "0 hàng, không lỗi".
- Đường `.docx` đơn ngữ (`import_file`, `DocxSidecar`, `Blob`) và đường kiểm bản reviewer giữ từng byte; `docx_boundary.rs` xanh.

**Never:**
- Không crate, migration, đổi `tauri.conf.json`/`capabilities/**`; không bộ đọc `.docx` thứ hai; không làm phẳng ô rồi ghép lại; không sửa `epics.md`/`prd.md`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Một bảng | 1 bảng cấp cao, 3 hàng | Dùng bảng đó, xem trước nêu tên/thứ tự bảng, 3 hàng; ô nhiều đoạn giữ trong một ô | Không lỗi |
| Nhiều bảng, chưa chọn | 2+ bảng cấp cao | Danh sách bảng, "chưa chọn", xác nhận khoá, 0 hàng xem trước | Không phải lỗi; không chọn hộ |
| Nhiều bảng, đã chọn | `table_index` hợp lệ | Xem trước hàng bảng đó, xác nhận mở nếu không lệch cặp | Không lỗi |
| `table_index` ngoài khoảng / hết hạn | Chỉ số ≥ số bảng | Từ chối nêu chỉ số và số bảng | Lỗi có kiểu, không lùi về bảng 1 |
| Không bảng | Chỉ có đoạn | Từ chối "không có bảng hai cột" | Lỗi có kiểu, 0 Work |
| Chỉ bảng lồng | Bảng chỉ nằm trong ô | Coi như không bảng, từ chối "không có bảng hai cột" | Lỗi có kiểu |
| Bảng lồng + bảng cấp cao | 1 cấp cao chứa 1 lồng | Một bảng, không từ chối | Không lỗi |
| Chỉ bảng một cột | Mọi bảng cấp cao có hàng rộng nhất 1 ô | Không liệt kê, từ chối "không có bảng hai cột" | Lỗi có kiểu |
| Bảng một cột + bảng hai cột | 1 bảng 1 cột, 1 bảng 2 cột | Danh sách chỉ có bảng hai cột, dùng nó như ca "Một bảng" | Không lỗi |
| Ô đích rỗng / một hàng nhiều đoạn | Hàng lệch số câu | Vào danh sách hàng lệch cặp, khoá xác nhận | Như `.csv` |
| Đơn ngữ | Cùng `.docx`, nhập thường | Y như trước, từng byte | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/docx/mod.rs:124-142,151` -- `DocxParsed.body`: `DocxBodyItem::Table(Vec<Vec<String>>)` `[hàng][cột]`; bảng lồng là mục anh em ngay sau bảng cha, nên `body` hiện KHÔNG phân biệt cấp cao/lồng (`absorb_table`). Cần một dấu lồng mới, chỉ thêm (không đổi trường cũ); callers cũ: `core/export/reviewer_copy.rs:112`, `reimport_gate.rs:31`.
- `src-tauri/src/core/segment/import.rs:907,919,783` -- `BILINGUAL_SUPPORTED_EXTENSIONS`, `import_bilingual_file` (chỉ `RawBytes`); `:783` nhánh `.docx` đơn ngữ (KHÔNG đổi).
- `src-tauri/src/core/segment/pipeline.rs:211,738,790-830,1247` -- `PipelineShape::Bilingual`; bước giải mã gọi `parse_rows`, kiểm <2 cột, bỏ tiêu đề. Test khớp vét cạn: `review_contract.rs:13`, `docx_contract.rs:186`, `webimport_contract.rs:1639`.
- `src-tauri/src/core/segment/bilingual.rs:27,93,118` -- `BilingualRow`, `parse_rows`, `widest_row_column_count` (tái dùng).
- `src-tauri/src/commands/project/bilingual.rs:166-205` -- `preview_bilingual_import` quanh `encoding::detect`; nhánh `AlreadyText` trả rỗng (bẫy 0 hàng im lặng).
- `src-tauri/src/commands/project/wire.rs:1031,1090` + lệnh confirm -- `preview_bilingual_import_from_file`, `rebuild_bilingual_import_preview` (clone `shape` đã cất, không đọc lại tệp), confirm.
- `src-tauri/src/core/i18n/mod.rs:657`, `src/i18n/vi.json:36` -- khoá `bilingual_unsupported_format` ("chỉ .csv và .tsv") cần sửa; cần khoá mới.
- `src/bilingualImportPreviewState.ts`, `src/modes/libraryImport.ts:609`, `src/modes/LibraryMode.vue:1416` -- ô đường dẫn, xem trước có bộ chọn bảng mã.
- `src-tauri/tests/bilingual_import_contract.rs`, `fixtures_docx.rs`, `docx_boundary.rs` (`SRC_RS_FLOOR=113`; tệp `.rs` mới chỉ làm tổng tăng).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/docx/mod.rs` -- thêm dấu cấp-cao/lồng cho mục bảng của `body`, chỉ thêm -- để liệt kê đúng bảng cấp cao
- [x] `src-tauri/src/core/segment/pipeline.rs` -- đầu vào "hàng đã có": bỏ giải mã/`parse_rows`, vẫn kiểm cột và bỏ tiêu đề -- AD-39
- [x] `src-tauri/src/core/segment/import.rs` -- `import_bilingual_file` nhận `.docx`: `read_docx` ⇒ danh sách bảng cấp cao ⇒ chọn theo `table_index`; lỗi có kiểu cho 0 bảng/ngoài khoảng -- AC1, AC2
- [x] `src-tauri/src/commands/project/bilingual.rs`, `wire.rs` -- `table_index: Option<usize>` qua ba lệnh; xem trước trả danh sách bảng + bảng đang dùng; "chưa chọn" khoá xác nhận; không ứng viên bảng mã -- AC2
- [x] `src-tauri/src/core/i18n/mod.rs`, `src/i18n/vi.json` -- khoá: không có bảng hai cột, bảng ngoài khoảng, nhiều bảng chưa chọn; sửa câu "chỉ .csv và .tsv"
- [x] `src/bilingualImportPreviewState.ts`, `src/modes/LibraryMode.vue` -- bộ chọn bảng ("chưa chọn" mặc định), ẩn bộ chọn bảng mã cho nguồn hàng
- [x] `src-tauri/tests/bilingual_import_contract.rs` (+ ba test vét cạn `PipelineShape`; vitest bilingual) -- mọi hàng ma trận; đối chứng đỏ bằng GỠ thật seam chọn bảng (lùi về bảng 1 phải làm ca "ngoài khoảng" đỏ)

**Acceptance Criteria:**
- Given `.docx` một bảng cấp cao 3 hàng, when nhập song ngữ, then Work có 3 hàng với `target_text` + `bilingual_import` như `.csv` cùng nội dung.
- Given `.docx` nhiều bảng, when mở xem trước, then danh sách bảng hiện, mặc định chưa chọn, xác nhận khoá; chọn một bảng thì xem trước đúng hàng bảng đó.
- Given `.docx` không bảng, chỉ bảng lồng, chỉ bảng một cột, hoặc `table_index` ngoài khoảng, when xem trước, then bị từ chối nêu lý do, 0 Work.
- Given cùng `.docx`, when nhập đơn ngữ, then kết quả giống hệt trước thay đổi; `docx_contract.rs` và `docx_boundary.rs` xanh.

## Implementation Notes

- Hai hình dạng mới: `PipelineShape::BilingualTables` chỉ được cất ở trạng thái chờ, `select_bilingual_table` đổi nó thành `BilingualRows` trước khi vào chuỗi. Gặp `BilingualTables` trong `run_import` là lỗi có kiểu. `PIPELINE_ORDER` không đổi.
- `read_docx` có thêm `nested_table_body_indexes` và `top_level_tables()`, chỉ thêm trường mới. `docx_contract`, `reimport_gate_contract` và `review_contract` vẫn xanh.
- Hàng trống của bảng bị bỏ trước khi lọc ≥ 2 cột (bắt ở review). `row_number` là vị trí 1-based trong bảng gốc.
- Nhiều bảng mà chưa chọn: bản xem trước trả 0 hàng, 0 ứng viên, `table_choice_required = true`, không phải lỗi; xác nhận bị Rust từ chối bằng `bilingual_table_not_chosen`. Một bảng thì tự dùng.
- Nguồn `.docx` có đúng một ứng viên `UTF-8` nhãn `docx`, nên lượt xác nhận vẫn đi qua `encoding_for_wire_id`.
- `params.index` của lỗi ngoài khoảng là 1-based cho khớp nhãn "Bảng N"; trường của `ImportError` vẫn 0-based.
- Bộ chọn bảng nằm ở `BilingualImportPreviewOverlay.vue`, không ở `LibraryMode.vue` như task ghi; ô song ngữ là ô gõ đường dẫn, không có bộ lọc đuôi ở webview.
- Đổi bảng thì đặt lại cột nguồn/đích và các lượt quy nhóm; ô tiêu đề và mẫu phân tách giữ nguyên vì là lựa chọn theo tệp.
- Đối chứng đỏ bằng phép gỡ thật: lùi về bảng 1 khi chỉ số ngoài khoảng làm ca ngoài khoảng đỏ; bỏ vế `tableChoicePending` khỏi `canConfirm` làm ca khoá khi `table_index` còn null đỏ; lọc cột trên bảng thô làm ca bảng một cột kèm hàng trống đỏ.

## Plan Change Log

## Review Triage Log

- low · patch — `import_bilingual_docx` lọc ≥ 2 cột trên bảng thô rồi mới bỏ hàng trống: bảng một cột kèm một hàng trống hai ô được liệt kê rồi trả `too_few_columns`, trái quyết định "chỉ bảng ≥ 2 cột".
- low · patch — comment mới bằng tiếng Việt, nhắc lại tên/kiểu, có banner, trái AGENTS.md §Code comments.
- low · patch — doc của `BILINGUAL_SUPPORTED_EXTENSIONS` ("hai đuôi", `.docx` bị hoãn) và `import_bilingual_file` ("trả byte thô") sai sau thay đổi.
- low · patch (tự thấy khi đọc diff) — `params.index` của `bilingual_table_out_of_range` đếm từ 0, trong khi bộ chọn ghi "Bảng 1" cho chỉ số 0.
- false — ứng viên `.docx` mang `"UTF-8"` qua `encoding_for_wire_id`: `Encoding::for_label("UTF-8")` thuộc `RECOGNIZED_ENCODINGS`, và ca xác nhận `.docx` đi đúng đường đó, xanh.
- false — `setBilingualTableIndex` không đặt lại `chapterCursor`/bộ lọc: lớp phủ song ngữ không đọc con trỏ, chỗ đọc duy nhất (bật/tắt bộ lọc) đã kiểm biên; `hasHeader`/mẫu phân tách là lựa chọn theo tệp, cố ý giữ.
- false — `LibraryMode.vue`/`libraryImport.ts` không đổi: ô song ngữ là ô gõ đường dẫn, không có bộ lọc đuôi ở webview, `.docx` đi thẳng tới Rust.
- false — `\n` trong `<option>`: trình duyệt gộp khoảng trắng của văn bản option thành một dấu cách; cắt theo `char` không làm hỏng chuỗi.
- false — nhánh `BilingualRows` trả bản xem trước rỗng: shape cất chỉ đến từ `import_bilingual_file` (`Bilingual` hoặc `BilingualTables`), và nhánh phòng thủ trả rỗng có từ trước story này.
- false — đối chứng đỏ và các lệnh Verification "không ghi lại": pha Rust đã gỡ seam chọn bảng và thấy ca ngoài khoảng đỏ, pha Webview đã gỡ vế `tableChoicePending`; `docx_contract`, `docx_boundary`, `ipc_contract`, `bilingual_import_contract` và vitest đã chạy lại xanh ở phía điều phối; một `.docx` toàn ô trống bị từ chối có kiểu `docx.empty_text`, vẫn là từ chối có tên lý do.

## Design Notes

Hàng đi vào pipeline là chính `BilingualRow`, không tuần tự hoá thành TSV rồi `parse_rows` (escape ngược tab/xuống dòng/ngoặc kép, `row_number` trôi khỏi bảng gốc).

## Verification

**Commands:**
- `cargo test --test bilingual_import_contract` -- expected: xanh
- `cargo test --test docx_boundary --test docx_contract --test reimport_gate_contract` -- expected: xanh
- `cargo test --test review_contract --test webimport_contract` -- expected: xanh
- `npx vitest run tests/frontend/importPreviewBilingual.test.ts tests/frontend/importPreviewBilingualWireShape.test.ts` -- expected: xanh

**Manual checks (if no CLI):**
- Mở `.docx` thật có hai bảng trong app: danh sách bảng, chọn, xem trước (Chủ: Epic 6).
