---
title: 'Tự nhận ngôn ngữ nguồn khi nhập'
type: 'feature'
ticket: ''
created: '2026-10-25'
status: 'built'
baseline_revision: '3f1467ebaebc145118ff3c5e8e5799c683f1eebc'
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

**Problem:** Ngôn ngữ nguồn phải chọn tay trước khi nội dung được đọc, và chọn sai thì câu bị tách sai rồi lưu vĩnh viễn (ranh giới câu tính một lần lúc nhập). Ice muốn app tự nhận.

**Approach:** Hàng chip có thêm "Tự nhận", mặc định ở màn nhập và màn Nhập từ website; webview gửi `sourceLang = 'auto'` cho lượt xem trước đầu tiên. Rust đoán `zh`/`en` từ nội dung đã giải mã bằng tỉ lệ chữ Hán (`cjk_ratio_sample`, ngưỡng ≥ 0,5 ⇒ `zh`), tách câu bằng giá trị đã đoán, và trả giá trị đó về. Từ đó webview gửi giá trị cụ thể cho mọi lệnh sau (ước Glossary, tải lại, xác nhận), nên không lệnh nào sau lượt xem trước nhận `'auto'`. Màn xem trước hiện "Tiếng Trung (tự nhận)". Chọn tay Tiếng Trung/Tiếng Anh chạy như hôm nay; đích là Tác phẩm có sẵn giữ ngôn ngữ của đích.

**Quyết định của Ice:** (1) Đường song ngữ chưa tự nhận: khi chip là Tự nhận, nút xem trước song ngữ không gọi Rust mà báo cần chọn Tiếng Trung hoặc Tiếng Anh (nợ `deferred-work.md`, Chủ: Ice). (2) Màn xem trước có chip Tiếng Trung/Tiếng Anh ở dòng ngôn ngữ; bấm thì chạy lại xem trước với giá trị đó (link dựng lại từ nội dung đã tải, không gọi mạng lại). (3) Nội dung không có chữ nào (`cjk_ratio_sample` trả `None`) ⇒ lượt xem trước báo lỗi đòi chọn ngôn ngữ tay, không đoán.

## Boundaries & Constraints

**Always:** Tác phẩm chỉ được tạo với `zh`/`en`: `validate_source_lang` tiếp tục từ chối mọi giá trị khác, kể cả `'auto'`. Ước Glossary chỉ chạy sau khi ngôn ngữ đã được đoán. Một phép đoán duy nhất, chung cho mọi đường (không chép heuristic sang TypeScript).

**Never:** Không đổi heuristic hay ngưỡng. Không bỏ cảnh báo lệch ngôn ngữ sau khi tạo (`source_lang_mismatch`). Không lưu `source_lang` vào trạng thái phiên phía Rust.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Dán Trung văn | Chip Tự nhận, văn bản ≥ 50% chữ Hán | Xem trước tách câu kiểu `zh`, dòng tóm tắt "Tiếng Trung (tự nhận)", Tác phẩm tạo với `zh` | — |
| Dán tiếng Anh | Chip Tự nhận, văn bản Latin | Như trên với `en` | — |
| Không có chữ | Chip Tự nhận, chỉ số/dấu câu | Không mở màn xem trước | Lỗi mới hiện ở form: hãy chọn Tiếng Trung hoặc Tiếng Anh |
| Đổi ở màn xem trước | Đã tự nhận `zh`, bấm chip Tiếng Anh | Xem trước chạy lại với `en`, nhãn không còn "(tự nhận)", Tác phẩm tạo với `en` | — |
| Song ngữ + Tự nhận | Chip Tự nhận, bấm xem trước song ngữ | Không gọi Rust | Báo cần chọn ngôn ngữ tay |
| Chọn tay | Chip Tiếng Anh, nội dung Trung văn | Dùng `en` như hôm nay, không nhãn "(tự nhận)" | — |
| Tệp / link | Chip Tự nhận | Đoán từ nội dung đã giải mã/trích xuất của lượt xem trước | — |
| Đích có sẵn | Thêm Chương vào Tác phẩm `en` | Dùng `en` của đích, không đoán | — |
| `'auto'` lọt tới lúc tạo | Lỗi lập trình | `UnsupportedSourceLang`, không ghi gì | Lỗi hiện như hôm nay |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/project/mod.rs:2086` `preview_import_encoding` -- hàm chung của dán văn bản (`AlreadyText`, gặp `source_lang` ở :2242), tệp (`build_file_import_batch_wire` :1605) và link (`url_import_encoding_preview`, `url_import.rs:461`). Móc đoán ngay sau `encoding::detect` (:2109): giải mã cửa sổ bằng `verdict.encoding` (hoặc lấy thẳng văn bản `AlreadyText`), đoán, rồi dùng giá trị đã đoán cho `render_candidates` và phần còn lại.
- `src-tauri/src/commands/project/work_creation.rs:1752-1815` -- `LANGUAGE_HEURISTIC_SAMPLE_CHARS`, `is_common_cjk_char`, `cjk_ratio_sample`, `source_lang_looks_mismatched`: đang private; chuyển/expose một hàm đoán dùng chung (ví dụ trong `core/segment/`) mà cả cảnh báo lệch lẫn xem trước gọi.
- `src-tauri/src/commands/project/mod.rs:1509` `ImportEncodingPreview` -- thêm trường ngôn ngữ đã dùng (và cờ tự nhận); `FileImportBatchWire` (:1588) và `UrlImportBatchWire` (`url_import.rs:105`) bọc nó nên phủ cả ba đường.
- `src-tauri/src/commands/project/bilingual.rs` -- không đổi; song ngữ chặn `'auto'` ở webview (`submitBilingualFilePath`, `src/modes/libraryImport.ts:641`).
- `src-tauri/src/commands/project/url_import.rs:37-48,433` -- `UrlImportItemsState` giữ `raw` của mỗi link: đổi ngôn ngữ ở màn xem trước dựng lại batch từ đó, 0 lượt gọi mạng; nếu cần một lệnh mới thì đăng ký ở `lib.rs` và chạy trọn bộ test.
- `src-tauri/src/commands/project/wire.rs` -- `preview_import_encoding_from_text` :638, `_from_file` :717, `start_url_import` :1338 nhận `'auto'`; các lệnh khác (`confirm_import_with_encoding` :830, `estimate_import_glossary` :1685, `reload_url_import_item` :1398, `remove_url_import_item` :1473, tier2 :1546/:1621, `preview_chapter_detail` :1835) chỉ nhận giá trị cụ thể.
- `src/modes/libraryImport.ts:109,161` -- `sourceLang` thêm `'auto'`, mặc định `'auto'`; `effectiveSourceLang` của đích giữ nguyên.
- `src/importPreviewState.ts:141` `pendingSourceLang` (đặt ở :837/:958/:1071) -- thay bằng giá trị Rust trả về sau lượt xem trước đầu; ước Glossary (:542) phải chạy sau đó. Song ngữ (`bilingualImportPreviewState.ts`) không đổi.
- `src/modes/SourceLangChips.vue`, `src/modes/libraryWorkCard.ts:13-25` -- chip "Tự nhận" đứng đầu; khoá i18n mới.
- `src/ImportPreviewOverlay.vue:781,1023` -- dòng tóm tắt ngôn ngữ (`destinationSourceLangLabel`): thêm "(tự nhận)" và chip đổi.
- `src/modes/LibraryMode.vue:712` -- cảnh báo lệch sau khi tạo, giữ nguyên.
- Đo: heuristic khớp 26/26 Tác phẩm thật đọc được (21 không đọc được `project.db`).

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/segment/` (mới hoặc có sẵn) + `work_creation.rs` -- hàm đoán dùng chung; cảnh báo lệch gọi lại nó.
- [ ] `src-tauri/src/commands/project/mod.rs`, `url_import.rs` -- `'auto'` ở `preview_import_encoding`, trường mới trên `ImportEncodingPreview`.
- [ ] `src/config/project.ts`, `src/importPreviewState.ts` -- đọc ngôn ngữ trả về, thay `pendingSourceLang`, xếp ước Glossary sau.
- [ ] `src/modes/libraryImport.ts`, `SourceLangChips.vue`, `libraryWorkCard.ts`, `src/i18n/vi.json` -- chip Tự nhận mặc định.
- [ ] `src/ImportPreviewOverlay.vue` -- nhãn "(tự nhận)" và chip đổi ngôn ngữ chạy lại xem trước; `src/modes/libraryImport.ts` -- song ngữ chặn `'auto'`; `src-tauri/src/core/i18n/mod.rs` + `vi.json` -- khoá lỗi không nhận được ngôn ngữ.
- [ ] Tests: hợp đồng Rust cho từng hàng Matrix ở `preview_import_encoding`; vitest cho chuỗi auto → giá trị cụ thể ở mọi lệnh sau; e2e 12-6 nhập một Trung văn với chip Tự nhận.

**Acceptance Criteria:**
- Given chip Tự nhận và một Chương Trung văn dán vào, when xác nhận nhập, then Tác phẩm mới có `source_lang = 'zh'` và câu tách theo dấu `。！？`.
- Given chip Tự nhận, when bất kỳ lệnh nào sau lượt xem trước đầu được gọi, then đối số `sourceLang` là `zh` hoặc `en`, không bao giờ `auto`.

## Implementation Notes

- Móc đoán đặt ở lớp lệnh (`resolve_preview_source_lang` → `ResolvedSourceLang { lang, auto }`), không trong `preview_import_encoding`; cờ "(tự nhận)" là đầu vào lúc dựng preview nên trình biên dịch buộc mọi chỗ gọi khai nó. `preview_import_encoding` cũ còn là lớp bọc `auto = false` cho ~40 ca test sẵn có.
- Phép đoán chuyển sang `core/segment/source_lang.rs`; cảnh báo lệch sau khi tạo dùng chung nó, so sánh `<`/`>` giữ nguyên.
- Lệnh mới `rebuild_url_import_preview` (đăng ký ở `lib.rs`): dựng lại xem trước link từ `UrlImportItemsState`, 0 lượt gọi mạng.
- Mọi link tải hỏng thì không có nội dung để đoán: webview giữ `'auto'`, `reload`/`remove` nhận nó và đoán khi có nội dung; một lần đoán hỏng ở hai lệnh đó không báo lỗi.
- `FRONTEND_FLOOR`/`WEBVIEW_FLOOR` 167 → 180: đã đỏ từ `5aff0ff` (210 tệp `.ts`/`.vue`), giá trị theo đúng công thức `ceil(0.85 × live)` mà ca canh tự in ra.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test <hợp đồng nhập liên quan>` -- xanh.
- `npx vitest run` các tệp test của `importPreviewState`, `bilingualImportPreviewState`, `libraryImportOverlay`, `importPreviewDestination` -- xanh.
- `npm run check:i18n`, `npx vue-tsc --noEmit` -- xanh.
- e2e `story-12-6-import-preview` ngoài sandbox -- xanh.

## Review Triage Log

Quick, vòng 1: medium 4 · low 4 · false 2.

- medium · patch — đường tệp đoán trên MỘT tệp; tệp đó giải mã hỏng thì cả lô bị huỷ với lỗi "chọn ngôn ngữ" sai nghĩa (`guess_shape_source_lang`, nhánh `Files`).
- medium · patch — `source_lang_auto` gán cứng `false` trong `preview_import_encoding`, năm chỗ gọi tự vá sau; ca Rust tự gán cờ bằng tay nên chỉ canh chính nó, và chỉ assert `!text.is_empty()`.
- medium · patch — `setImportPreviewSourceLang` nhánh văn bản/tệp không hoàn lại ngôn ngữ khi tải lại hỏng (nhánh URL có).
- medium · patch — ca e2e `langLine()` đọc cả nhãn chip nên `toContain('Tiếng Anh')` luôn đúng.
- low · patch — `reload`/`remove` URL phân giải ngôn ngữ SAU khi đã đổi danh sách; lỗi đoán trả sớm, bỏ qua bước đặt lại Tier2.
- low · patch — thiếu ca Rust: chọn tay `en` trên Trung văn, URL đoán từ HTML đã tải.
- low · patch — doc comment mới/chuyển chỗ viết tiếng Việt nhiều dòng (AGENTS.md §Code comments).
- low · reject — mọi link tải hỏng thì webview giữ `'auto'` và `reload`/`remove` nhận nó: không có nội dung nào để tách nên không câu nào bị tách sai; lượt tải lại đầu có nội dung sẽ đoán. AC2 viết với giả định lượt xem trước đầu có nội dung.
- false — "nâng sàn 167→180 không liên quan": sàn đã đỏ từ `5aff0ff` (210 tệp, cần ≥ 168), và `assert_population_floor` tự chỉ cách sửa `ceil(0.85 × live)` = 180.
- false — "thiếu mục nợ song ngữ": mục đã có ở cuối `deferred-work.md` (Chủ: Ice); diff rà soát không gồm `_bmad-output`.
