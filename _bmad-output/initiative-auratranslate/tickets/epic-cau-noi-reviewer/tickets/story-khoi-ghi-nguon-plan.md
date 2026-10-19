---
title: 'Khối ghi nguồn'
type: 'feature'
ticket: '7'
created: '2026-10-08'
status: 'built'
baseline_revision: '11c10ddc991838fef50e699006cc40fd4ff7c70f'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/export-images-attribution.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Người đăng bài nhận file bàn giao không cài app nên không có cách nào biết tác giả, nguồn, ngày đăng gốc và người dịch; ba định dạng xuất của 8.3/8.4/8.6 không ghi gì trong số đó, và tên người dịch chưa có chỗ lưu (`ScopeKind::TranslatorName` đã khai `Override` nhưng không bảng, không lệnh, không UI).

**Approach:** Thêm tuỳ chọn "Chèn khối ghi nguồn" (mặc định tắt) vào màn hình xuất; khi bật, mỗi Chương xuất ra mở đầu bằng một khối dựng lúc xuất từ bốn cột `chapter.origin_*` đọc trực tiếp lúc nạp và tên người dịch phân giải qua `ScopeResolver::apply_override("translator_name", …)` (AD-18, AD-43). Tên người dịch đặt một lần ở cấu hình toàn cục.

**Decisions (Ice, 2026-10-08):**
- Tên người dịch chỉ ở tầng toàn cục: một bước `GLOBAL_MIGRATIONS` (v13), bảng `translator_name(key,value,updated_at)` chỉ ở `global.db`; lúc xuất gọi `apply_override("translator_name", &global, None)`. Ghi đè theo Tác phẩm là mục nợ `Chủ: Ice`, không làm ở 8.7.
- Tên người dịch đặt ở mục mới "Xuất" trong Cài đặt (`SettingsSection` thêm `export`). Màn hình xuất chỉ hiện tên ở dạng chỉ-đọc, kèm huy hiệu "Toàn cục"; khi tên rỗng, màn hình nhắc người dùng mở Cài đặt.
- Giữ plan đủ phạm vi dù vượt 1600 token.

## Boundaries & Constraints

**Always:**
- Khối nằm **trước tiêu đề Chương**, **ngoài mọi bảng** ở cả hai `.docx` (8.8 sẽ đọc hình dạng bảng; khối không được chen vào ô).
- Thứ tự dòng theo mockup: `Tác giả: X` · `Nguồn: <site> · <url>` · `Ngày đăng gốc: <date>` · `Người dịch: <name>`. Trường rỗng/NULL thì bỏ phần đó (trong dòng Nguồn bỏ vế rỗng và dấu `·`); dòng hết vế thì bỏ dòng; cả năm rỗng thì không có khối.
- `origin_published_at` in nguyên văn (cột là TEXT tự do).
- Nhãn trong file là nội dung định dạng xuất ⇒ chữ trong Rust kèm `// aura-allow-text: <lý do>` như tiền lệ `text_export.rs:187`.
- Tham số mới tên `attribution: bool` ở cả ba lệnh xuất; khoá `invoke` trùng tên tham số Rust.
- Tệp `.rs` mới theo trách nhiệm; nâng sàn quần thể cùng lượt nếu chạm ngưỡng.

**Never:**
- Không lưu chuỗi ghi nguồn đã định dạng ở bất kỳ đâu (DB, state webview, cache).
- Không suy ra tên site từ host của URL (§Never của 6.15).
- Không nới `scope::save_value` cho kind không phải `GlobalOnly`; không đổi ngữ nghĩa `TranslatorName` trong `kinds.rs`.
- Không thêm dependency, không ghi `project.db` lúc xuất.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Tắt (mặc định) | `attribution=false` | File byte-for-byte như trước 8.7 | — |
| Bật, đủ năm trường | Chương có 4 cột origin + tên người dịch | Bốn dòng trước tiêu đề, cả 4 định dạng | — |
| Bật, thiếu ngày + URL | origin_published_at NULL, origin_url NULL | Dòng Nguồn chỉ có site, không có dòng ngày | — |
| Bật, chưa đặt tên | không có giá trị translator_name | Không có dòng Người dịch; màn hình xuất báo chưa đặt tên | Không chặn xuất |
| Bật, Chương không xuất xứ, không tên | cả năm rỗng | Không có khối cho Chương đó | — |
| Sửa xuất xứ rồi xuất lại | `update_chapter_origin` đổi tác giả | Lần xuất kế tiếp mang tác giả mới | — |
| Nhiều Chương | phạm vi 3 Chương, xuất xứ khác nhau | Mỗi Chương một khối riêng theo dữ liệu của chính nó | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/scope/kinds.rs:193-197` -- `TranslatorName => "translator_name" : Override` đã khai; không sửa.
- `src-tauri/src/core/scope/mod.rs:230,293` -- `ScopeResolver::apply_override`; gọi với kind là chuỗi literal (`scope_boundary.rs` cấm tên `ScopeKind` ngoài `core/scope`).
- `src-tauri/src/core/scope/store.rs:427-449` -- `save_value` từ chối kind không `GlobalOnly` ⇒ không dùng `config_value`.
- `src-tauri/src/core/aiconfig/store.rs:37-67` + `commands/aiconfig.rs` -- khuôn mẫu một kind `Override`: bảng `key,value,updated_at`, nạp tầng thành `BTreeMap`, `resolve_two_tiers`.
- `src-tauri/src/core/store/schema.rs:756` (`GLOBAL_MIGRATIONS`, đỉnh v12 tại :816), project.db đỉnh v29; `CHAPTER_ORIGIN_DDL` :1872 (4 cột TEXT nullable).
- `src-tauri/src/commands/export.rs` -- `export_docx_two_column` :107, `write_export_file` :137, `export_docx_one_block` :182, `export_text` :205, mô-đun `wire` :232-321; đăng ký ở `lib.rs:1107-1111`.
- `src-tauri/src/core/export/table_rows.rs:92` / `block_paragraphs.rs:161` / `text_export.rs:53` -- mỗi bộ nạp chạy `SELECT ord, title FROM chapter WHERE id = ?1`; thêm bốn cột `origin_*` vào đây là móc tự nhiên (đọc sống lúc xuất).
- `src-tauri/src/core/export/docx_table.rs:70`, `docx_block.rs:26`, `text_export.rs:202-215` -- chỗ mỗi Chương bắt đầu (tiêu đề); `paragraph_of` `docx_table.rs:25` dùng chung.
- `src-tauri/src/core/export/mod.rs:1-4` -- comment nhắc `SOURCE_ORIGIN` không tồn tại; sửa dòng đó khi đụng tệp.
- `src/ExportOverlay.vue` -- fieldset ảnh :273-311, thư mục :313 ⇒ fieldset ghi nguồn chen giữa; `src/exportState.ts` (`format` :39, setters :146-151, `writeFormat` :198, `resetExport` :229); `src/config/export.ts:159-200` ba adapter `invoke`.
- `src/SettingsOverlay.vue` (các mục :274-500) + `src/settingsState.ts:20-32` (`SettingsSection`, `settingsSectionLabelKey`) -- thêm mục `export`; khuôn `SettingsTmSection.vue` + `tmSettingsState.ts`.
- `src/i18n/vi.json:333-381` (`export.*`), `:673-678` (`chapter.origin.*` nhãn sẵn có).
- Kiểm thử để soi theo: `src-tauri/tests/export_docx_contract.rs`, `export_block_contract.rs`, `export_text_contract.rs`, `chapter_origin_contract.rs`, `ipc_argument_contract.rs` (`REGISTERED_COMMAND_FLOOR` 114), `config_invariants.rs` (async rows :1167-1182, census `export.rs` :1774, tổng `(69, 53)` :1953), `scope_contract.rs:171`; vitest `tests/frontend/exportImages.test.ts`, `exportBlock.test.ts`, `exportText.test.ts`, `exportDocx.test.ts`, `settingsState.test.ts`.
- Sàn: `src-tauri/src` hiện 124 `.rs`, sàn 105 đỏ khi >131; `check-i18n.mjs:235` `RS_FLOOR` 106 đỏ khi >132.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store/schema.rs` -- `GLOBAL_MIGRATIONS` bước v13: bảng `translator_name(key,value,updated_at)`; không có bước song sinh ở project.db -- chỗ lưu tầng toàn cục cho kind `Override`.
- [x] `src-tauri/src/core/attribution/store.rs` (tệp mới, + `mod.rs`) -- nạp tầng toàn cục, phân giải qua `apply_override("translator_name", &global, None)`, ghi/xoá (giá trị trim rỗng ⇒ xoá hàng) -- theo khuôn `aiconfig/store.rs`.
- [x] `src-tauri/src/commands/attribution.rs` (tệp mới) + `lib.rs` -- lệnh đọc/ghi tên người dịch -- Cài đặt đặt, màn hình xuất hiển thị.
- [x] `deferred-work.md` -- một mục nợ ≤5 dòng: ghi đè tên người dịch theo Tác phẩm (kind đã khai `Override`, chỉ tầng toàn cục được dựng), `Chủ: Ice`.
- [x] `src-tauri/src/core/export/attribution.rs` (tệp mới) -- kiểu `ChapterAttribution` (bốn `Option<String>` + tên) và hàm dựng danh sách dòng theo luật bỏ-rỗng -- một chỗ dựng cho cả ba bộ ghi.
- [x] `table_rows.rs`, `block_paragraphs.rs`, `text_export.rs` -- SELECT thêm bốn cột `origin_*`, gắn vào struct Chương.
- [x] `docx_table.rs`, `docx_block.rs`, `text_export.rs` -- khi bật, ghi các dòng trước tiêu đề (docx: một đoạn, ngắt dòng giữa các dòng; `.md`: dòng nối bằng hard break `\`; `.txt`: dòng thường), cách tiêu đề một dòng trống ở text.
- [x] `src-tauri/src/commands/export.rs` -- tham số `attribution: bool` ở ba lệnh và `wire`; phân giải tên người dịch một lần mỗi lượt xuất.
- [x] `src/config/export.ts`, `src/exportState.ts`, `src/ExportOverlay.vue`, `src/i18n/vi.json` -- ô bật/tắt mặc định tắt, ghi chú mặc định tắt, dòng "Người dịch" (huy hiệu Toàn cục) hoặc nhắc chưa đặt; `resetExport` trả về `false`.
- [x] `src/settingsState.ts`, `src/SettingsOverlay.vue`, `src/SettingsExportSection.vue` (mới, khuôn `SettingsTmSection.vue`), `src/config/attribution.ts` (mới) -- mục "Xuất" với ô tên người dịch, lưu toàn cục; màn hình xuất nhắc mở mục này khi tên rỗng.
- [x] Kiểm thử: Rust một ca mỗi hàng ma trận trên cả bốn định dạng (tắt = không khác bản cũ; sửa-rồi-xuất-lại dùng `update_chapter_origin`); ca phân giải tên; vitest ô bật/tắt gửi `attribution` và reset; cập nhật `ipc_argument_contract`, `config_invariants`, sàn nếu chạm ngưỡng.

**Acceptance Criteria:**
- Given màn hình xuất mở lần đầu, when nhìn mục Khối ghi nguồn, then ô đang tắt và ghi chú "Mặc định tắt" hiện ngay.
- Given tên người dịch đã đặt một lần, when xuất nhiều Tác phẩm khác nhau, then không phải gõ lại và mọi khối mang tên đó.
- Given khối ghi nguồn bật, when xuất `.docx` hai cột rồi đọc lại bằng `core::docx::read_docx`, then số bảng và số hàng mỗi bảng không đổi so với khi tắt.

## Implementation Notes

- Tên người dịch lưu ở bảng `translator_name` (`GLOBAL_MIGRATIONS` v13, chỉ `global.db`). Không dùng `config_value`, vì `scope::save_value` chỉ nhận các kind `GlobalOnly`. Lệnh `translator_name_get` trả `string | null`. Lệnh `translator_name_save` xoá giá trị khi tên rỗng sau trim.
- Bốn cột `chapter.origin_*` được đọc bằng một SELECT riêng trong `lines_for`, chạy trong cả ba bộ nạp. SELECT tiêu đề giữ nguyên. Khi tắt khối, tệp xuất giống hệt bản trước 8.7.
- Nhãn trong file xuất viết thẳng trong Rust, kèm `aura-allow-text`, theo tiền lệ `text_export.rs`. Trong `.md`, mỗi dòng đi qua `markdown_text`, nên `_` trong URL ra `\_`.
- Khi khối bật mà kho global vắng (`open_global_store` có thể `return`), xuất trả lỗi `store_is_missing`, không âm thầm bỏ dòng Người dịch. Ở màn hình xuất, lỗi đọc tên hiện qua `tError()`, không hiện thành "chưa đặt tên".
- Mục "Xuất" trong Cài đặt lưu bằng `@submit.prevent`, không thêm command vào registry.
- Đối chứng gỡ seam trên `export_attribution_contract`: gỡ khối ở `docx_table.rs` làm 4 ca đỏ, ở `docx_block.rs` cũng 4 ca đỏ; đổi nối `\` trong `text_export.rs` làm ca `.md` đỏ. Gỡ phép phân giải tên trong `attribution_of` thì ban đầu vẫn xanh, vì các ca truyền thẳng `Attribution`; nay có ca `attribution_on_carries_the_stored_translator_name` canh dây nối này.
- Lúc chạy toàn bộ vitest, máy tải ~450 làm một số test đỏ ngẫu nhiên vì quá thời gian 5 s; chạy lại với `--maxWorkers=3 --testTimeout=30000` thì xanh hết.
- Nợ mới: ghi đè tên người dịch theo Tác phẩm (`Chủ: Ice`); kiểm tay khối trong Word và khi dán sang trình soạn thảo web (`Chủ: Epic 8`).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 3 · low 5 · false 1 · maybe-false 1. Ngoài lens: CSS `.ex-badge` cắt ngang danh sách selector `.ex-status, .ex-empty, .ex-note` — medium, Claude vá trước review.
- medium · patch — `exportState.ts` `openExport` bỏ `.error` của `translatorNameGet`; lỗi đọc hiện thành "chưa đặt tên" (rỗng im lặng). Vế "await nối tiếp làm chậm danh sách Chương": low, bỏ.
- medium · patch — `commands/export.rs` `attribution_of` coi kho global vắng là không có tên; `open_global_store` (`lib.rs:1379`) có thể `return` mà không quản lý kho, trong khi `translator_name_get` báo lỗi cùng trạng thái đó.
- medium · patch — chưa đối chứng gỡ seam (AGENTS.md) cho `export_attribution_contract`.
- low · patch — các hàng thiếu xuất xứ, rỗng, nhiều Chương chỉ chạy trên `.txt`/`.md`; task đòi cả bốn định dạng.
- low · patch — `deferred-work.md` thiếu mục kiểm tay khối ghi nguồn trong Word/trình soạn thảo web (Verification đòi `Chủ: Epic 8`).
- low · patch — comment tiếng Việt hoặc lặp lại dòng mã trong các tệp `.rs` mới, `TRANSLATOR_NAME_DDL` và `AttributionScopeError` (luật comment ở AGENTS.md).
- low · patch — tên test `…version_12…` trong `tm_contract.rs` và câu "mười hai bước" trong `store_contract.rs` đã lỗi thời, danh sách bước ở đó cũng thiếu bước `translator_name`.
- low · patch — `exportSettingsState.ts` không có test cho nạp, lưu và lỗi.
- maybe-false · reject — `.md` thoát `_` trong URL thành `\_`: CommonMark hiển thị lại đúng; muốn chắc phải dựng thử trên trình đọc có GFM autolink. Nếu đúng thì chỉ là low.
- low · reject — trường xuất xứ chứa ký tự xuống dòng làm vỡ khối: ô nhập là `type=text`, `update_chapter_origin` đã trim; cách sửa phải thêm nhánh.
- false — "lời nhắc phải là link sang Cài đặt": plan chỉ đòi nhắc người dùng mở mục Xuất, lời nhắc bằng chữ đã đáp ứng.

## Design Notes

Mockup cũng có lưới năm trường chỉ-đọc cho Chương đang xem; với phạm vi nhiều Chương mỗi Chương có xuất xứ riêng nên lưới không có một giá trị đúng. Plan này chỉ hiện dòng Người dịch trong màn hình xuất; lưới đầy đủ để ngoài phạm vi trừ khi Ice muốn khác.

Khối `.md` ví dụ:
```
Tác giả: 烽火戏诸侯\
Nguồn: truyen-example.com · https://truyen-example.com/c3\
Người dịch: Ice

## Chương 3 — …
```

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test export_docx_contract --test export_block_contract --test export_text_contract --test ipc_argument_contract --test config_invariants --test scope_contract` -- xanh.
- Vì có migration: chạy toàn bộ `cargo test` và vitest một lần trước khi báo xong (AGENTS.md, shared wiring).
- `npm run check:i18n` -- xanh.

**Manual checks (if no CLI):**
- Mở `.docx` một khối trong Word, bôi cột phải dán sang trình soạn thảo web: khối ghi nguồn không lẫn vào phần đã bôi ⇒ nợ `Chủ: Epic 8` nếu chưa làm được.
