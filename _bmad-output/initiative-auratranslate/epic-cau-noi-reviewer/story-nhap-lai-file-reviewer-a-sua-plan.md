---
title: 'Nhập lại file reviewer đã sửa'
type: 'feature'
ticket: '9'
created: '2026-10-08'
status: 'built'
baseline_revision: '111f2ec460275d4ca25cd35dba22f2762be206ba'
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

**Problem:** Bản `.docx` hai cột (8.3, 8.5) hay `.md` (8.6) mà reviewer sửa xong hiện chưa có đường nào quay về Tác phẩm. 8.10 (alignment), 8.11 (Review Mode) và 8.14/8.15 (thu hoạch, FR95) đều cần bản đó đã nằm trong Tác phẩm.

**Approach:** Rust chọn tệp (AD-48), cho bản `.docx` qua `ReviewerDocx::admit` (AD-38) rồi phân tích thành mô hình có cấu trúc (AD-16). Sau đó Rust khớp tệp với Tác phẩm đang mở và trả về bản xem trước liệt kê các Chương bị ảnh hưởng. Chỉ khi người dùng xác nhận thì bản reviewer mới được ghi vào `project.db`.

## Boundaries & Constraints

**Always:** đầu vào `.docx` chỉ đi tiếp dưới dạng `ReviewerDocx`, không bao giờ dưới dạng `DocxParsed`. Hàng ảnh (link hoặc `<stem>-anh/…`, cùng chữ ở hai ô) được nhận diện và bỏ qua, không bị đọc thành câu. Bản xem trước giữ ở `Mutex<Option<Pending…>>` phía Rust và bị xoá khi huỷ, khi mở bản xem trước mới, khi xác nhận xong, và khi đóng Tác phẩm. Vue chỉ render bằng nội suy văn bản.

**Never:** ghi `target_text`, baseline hay cột xuất xứ (AD-47 ③: FR90 không phải là nguồn ghi). Tạo Tác phẩm hay Chương mới. Nhận diện tệp bằng metadata hay tên tệp (AD-38). Làm alignment, Review Mode hay thu hoạch (việc của 8.10–8.15).

## Decisions (Ice, 2026-10-08)

- **Nơi lưu:** theo AD-52, do Winston viết trước (hồ sơ `ad-brief-luu-ban-reviewer`). 🔵 2026-10-08: AD-52 đã có trong spine; lượt `bmad-build` kế tiếp dựng lại Code Map và Tasks theo nó (migration, khớp lại trong giao dịch xác nhận, `stale_at` ở gộp/tách Chương cùng cổng quét, hàm đọc có lỗi kiểu).
- **Khớp `.docx`:** theo cột trái so với nguồn của các segment thuộc bản dịch; tiêu đề chỉ để phá thế hoà.
- **Khớp `.md`:** theo tiêu đề `##`, kể cả dạng `Chương {ord}`.
- **"Không khớp":** khi không Chương nào khớp.
- **Bộ đọc `.md`:** viết tay theo đúng định dạng 8.6 xuất ra, không thêm crate. `.md` vẫn ở trong 8.9 dù plan vượt trần token.
- **Tác phẩm đích:** Tác phẩm đang mở, nút đặt ở thanh tiêu đề.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Hai cột đã sửa | bản xuất 8.3 của Tác phẩm đang mở, ô phải đã sửa | xem trước liệt kê đúng các Chương có trong tệp kèm số hàng; 0 byte ghi | — |
| Có hàng ảnh | bản xuất 8.5 có cả hai kiểu ảnh | hàng ảnh không được tính và không tạo hay đổi segment nào | — |
| Bản một khối | bản xuất 8.4 | không ra bản xem trước | `err.export.publish_copy_not_reimportable`, 0 ghi |
| Sai Tác phẩm | tệp xuất từ một Tác phẩm khác | không ra bản xem trước | khoá lỗi riêng nêu tên Tác phẩm đang mở, 0 ghi |
| `.md` | bản xuất 8.6, đoạn đã sửa | xem trước như hàng đầu tiên | — |
| Huỷ | đang có bản xem trước, người dùng huỷ | pending bị xoá, `project.db` không đổi | — |
| Xác nhận | bản xem trước hợp lệ | bản reviewer được ghi trong một transaction; `target_text` không đổi | lỗi ghi thì rollback, báo lỗi |

</frozen-after-approval>

## Code Map

Đường dẫn Rust tính từ `src-tauri/`.

- `src/core/export/reimport_gate.rs:24-39` -- `ReviewerDocx::admit` là hàm dựng duy nhất, `parsed()` trả `&DocxParsed`. Chưa có chỗ nào gọi.
- `src/core/docx/mod.rs:113-141,342-370,674-717` -- `DocxParsed` chỉ có số đếm `TableShape` và `blocks` đã bị làm phẳng. Ô rỗng bị bỏ ở `:715`. `BodyItem`/`TableContent`/`CellContent` là private. Cần thêm vào `DocxParsed` một accessor công khai trả thân tệp có cấu trúc: dãy đoạn hoặc bảng, mỗi bảng có hàng, mỗi hàng có ô, giữ cả ô rỗng. Không đổi `read_docx`. Phải giữ hai điều `tests/docx_boundary.rs` canh: không token mạng và không điểm panic (không `unwrap`, `expect`, chỉ số `x[i]`).
- `src/core/export/docx_table.rs:41-85` -- bố cục cần đọc ngược. Mỗi Chương gồm [đoạn ghi nguồn tuỳ chọn] → đoạn tiêu đề (rỗng = `NULL`) → bảng `[nguồn, đích]`, không có hàng đầu bảng. Chương không có hàng thì không có bảng. Ô có `\n` là `w:br` trong một đoạn. Hàng ảnh `:57-63` có cùng một ô ở hai cột: URL (hyperlink), `file_name`, hoặc `{dir}/{copied_name}` với `dir` kết thúc bằng `IMAGE_DIR_SUFFIX` (`image_files.rs:8,18`).
- `src/core/export/text_export.rs:123-233` -- định dạng `.md`. Thứ tự là [khối ghi nguồn, các dòng nối bằng `\\\n`] → `## tiêu đề` (`heading_of` `:198`, vắng thì `Chương {ord}`) → các khối. Ảnh là `![alt](dest)` cộng dòng `*chú thích*` (`:176`). Escape theo `:123-157`. Chương rỗng bị bỏ. Khối ghi nguồn đứng **trước** `##`, nên không được dính vào đoạn cuối của Chương liền trước.
- `src/commands/chapter.rs:793-938` (`merge_chapter_into_previous`) -- `DELETE FROM chapter_position` `:929` rồi `DELETE FROM chapter` `:930`. Xoá bản reviewer của B và đặt `stale_at` cho A ngay cạnh hai câu đó. `:983-1114` (`split_chapter_at_segment`) đặt `stale_at` cho A trong `store.write`. Thời điểm dùng `strftime('%Y-%m-%dT%H:%M:%fZ','now')`, như `:1047`.
- `src/core/store/schema.rs:2068-2244` -- thêm migration `to_version: 30`. Lược đồ không có `REFERENCES`, nên mọi phép xoá hàng phụ thuộc phải viết tay. Các assert phiên bản cần nâng: `tests/pinned_contract.rs:258` (len 28 → 29), `:269`, cùng mọi `29,` trong `segment_contract.rs` (8 chỗ), `chapter_origin_contract.rs:891`, `segment_role_contract.rs:598`, `tm_contract.rs:691,722,1034`.
- `src/commands/glossary.rs:675-686,844-972,1450-1530` -- khuôn preview → confirm → cancel. Dialog chạy trước, khoá `OpenWorkState` mới lấy sau dialog. Pending bị xoá ở **hai** chỗ: `lib.rs:1686` (`close_open_work`) và `commands/project/mod.rs:3384-3392` (`replace_open_work`). Đăng ký state ở `lib.rs:1550`.
- `src/commands/export.rs:247-340` -- ba lệnh mới đặt vào `wire` ở đây, không tạo tệp lệnh mới. Bộ lọc dialog theo khuôn `tm.rs:871`. `tests/config_invariants.rs`: thêm một hàng `blocking_wire_cases` (`:1166-1185`) và cập nhật hàng census export `:1781`.
- `src/core/promptset/store.rs:340,494-506` -- khuôn hàm đọc trả lỗi có kiểu.
- `tests/support/boundary_scan.rs:291,325,381` -- `code_lines`, `matching_close_brace`, `without_test_modules`. Dùng chúng cho cổng quét theo hàm. Hiện chưa có cổng nào cắt tệp theo hàm. Khuôn mẫu mồi: `tests/dict_boundary.rs:551,610`.
- Sàn quần thể (`boundary_scan.rs:354`, trần trôi 80 %): `src/` có 129 tệp `.rs`, trần 131; `src-tauri/` có 210, trần 213. Story này thêm đúng hai tệp `src-tauri/` (một ở `src/`, một ở `tests/`). Nếu thêm tệp nữa thì nâng mọi sàn lên `ceil(0.85×live)` như panic chỉ, không gộp tệp để né.
- Webview: chép `src/tmImportState.ts` và adapter `src/config/export.ts:64-131`. Nút thanh tiêu đề đặt cạnh `data-export-open` (`src/App.vue:296-304`). Lệnh khai ở `src/commands/index.ts:3289-3322`, port `:888`, deps qua `src/exportCommandDeps.ts`. Test mock `src/config/*` như `tests/frontend/exportScope.test.ts:18-25`.
- `eslint.config.js:71` -- chỉ có `flat/base`. Repo không có `v-html` nào, nên thêm `vue/no-v-html: 'error'` không đỏ chỗ nào.
- `_bmad-output/initiative-auratranslate/deferred-work.md:12955-12957,12979-12981` -- hai mục `Chủ: Story 8.9`, đóng trong story này.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/docx/mod.rs` -- accessor thân có cấu trúc -- bộ đọc reviewer cần biết hàng/ô, không chỉ văn bản phẳng
- [x] `src-tauri/src/core/store/schema.rs` + các assert phiên bản -- migration v30, đúng hai bảng của AD-52 ①②: `UNIQUE(chapter_id)`, `UNIQUE(review_chapter_id, ord)`, `CHECK` cho `file_kind`/`kind`, `target_text NOT NULL`
- [x] `src-tauri/src/core/export/reviewer_copy.rs` (mới) + `mod.rs` -- mô hình `ReviewerCopy` (mục → hàng), bộ đọc `.docx` chỉ nhận `&ReviewerDocx` và bộ đọc `.md`, bộ khớp (Design Notes), `confirm_import(tx, …)` (khớp lại trong giao dịch, DELETE rồi INSERT), `read_review_copy` trả `Result<_, ReviewCopyError { NotImported, Stale }>`
- [x] `src-tauri/src/commands/chapter.rs` -- khi gộp: xoá `review_row`/`review_chapter` của B, đặt `stale_at` cho A. Khi tách: đặt `stale_at` cho A
- [x] `src-tauri/src/commands/export.rs` + `lib.rs` + `commands/project/mod.rs` + `config_invariants.rs` -- `reviewer_import_open_preview` (async, dialog `.docx`/`.md`), `reviewer_import_confirm`, `reviewer_import_cancel`, `PendingReviewerImportState` được xoá ở cả hai chỗ đóng Tác phẩm; khoá `err.export.*` mới trong `vi.json`
- [x] webview -- `src/config/reviewerImport.ts`, `src/reviewerImportState.ts`, `src/ReviewerImportOverlay.vue` (chỉ nội suy văn bản), nút thanh tiêu đề, lệnh ở `commands/index.ts`, `eslint.config.js` thêm `vue/no-v-html`
- [x] `src-tauri/tests/reviewer_import_contract.rs` (mới) -- cả ma trận I/O, mọi tệp dựng bằng hàm xuất thật của 8.3/8.4/8.5/8.6; ca gộp/tách; cổng quét; đối chứng gỡ (`admit`, phép khớp, câu ghi `review_chapter` ở gộp và ở tách)
- [x] `tests/frontend/reviewerImport.test.ts` (mới) -- trạng thái overlay + ca ESLint đỏ trên một đoạn `.vue` có `v-html`
- [x] `deferred-work.md` -- đóng hai mục 8.9 bằng lời; nợ mới `Chủ: Epic 8`: nhập một tệp Word thật đã sửa trong Word, thông báo lỗi thời ở Review Mode (8.11)

**Acceptance Criteria:**
- Given bất kỳ ca từ chối hay huỷ nào trong ma trận, when kết thúc, then số hàng của mọi bảng và `PRAGMA data_version` của `project.db` không đổi.
- Given một Chương đã có bản reviewer, when nhập lại tệp có Chương đó, then xem trước nêu Chương sẽ bị thay, và sau khi xác nhận Chương có đúng một `review_chapter` với id mới.
- Given tập Chương khớp lúc xác nhận khác lúc xem trước (ví dụ có một lần gộp xen giữa), when xác nhận, then không ghi gì và báo lỗi.
- Given gộp B vào A khi cả hai đã có bản reviewer, when gộp xong, then bản của B không còn hàng nào và A đọc ra `Stale`. Given tách A, then A đọc ra `Stale` và Chương mới đọc ra `NotImported`.
- Given một hàm trong `src/` có `DELETE FROM chapter` hoặc `UPDATE segment SET chapter_id` mà không nhắc `review_chapter`, when chạy cổng quét, then đỏ và nêu tên hàm.
- Given một tệp `.vue` có `v-html`, when chạy ESLint, then lint đỏ.
- Given đã xác nhận, when đọc lại `segment`, then `target_text`, baseline và cột xuất xứ trùng từng byte với trước lúc nhập.

## Implementation Notes

- `DocxParsed` có thêm trường `body` (đoạn hoặc bảng, ô rỗng được giữ). Đoạn thân đọc `w:br` thành dấu cách, nên đoạn ghi nguồn bốn dòng của `.docx` đọc ra thành một khối; tiêu đề luôn là đoạn đứng ngay trước bảng.
- Ba lệnh nằm trong `commands/export.rs`. Thứ tự khoá: `OpenWorkState` trước, pending sau, giống glossary. Pending bị xoá ở `close_open_work` và `replace_open_work`.
- Xác nhận so cả hàng sẽ ghi, không chỉ tập Chương, vì hàng ảnh bị lọc theo asset hiện tại. Lỗi ghi thì rollback và giữ bản xem trước để thử lại; khớp lệch thì bỏ bản xem trước.
- Cổng quét chỉ bắt `DELETE FROM chapter` và `UPDATE segment SET chapter_id` (xem Design Notes); `matching_close_brace` trong `boundary_scan.rs` nay là `pub`.
- Story thêm tệp nên tám sàn quần thể trôi quá 80 % và được nâng theo `ceil(0.85×live)`: `check-commands` (3 sàn), `check-i18n`, `check-tokens`, `check-layout`, `check-doc-refs`, `naming_boundary`/`segment_boundary`.
- `check:commands` (`SettingsExportSection.vue:28`), `check:panel-refs` (`exportSettingsState.ts`) và `check:debt-owner` (`PLANNED_TICKET_FLOOR` 120 với 151 ticket) đã đỏ ở `111f2ec` trước story này. Story không sửa ba cổng đó, nên `pre-push` sẽ chặn cho tới khi chúng được sửa.
- Lượt vitest cả bộ chạy lúc load 442 (Blender cùng một VM) hỏng 23 ca, tất cả vì quá giờ. Chạy lại riêng 13 tệp đó thì 10 tệp xanh; 3 tệp còn lại xanh khi chạy một worker lúc load ~54. Chưa có một lượt cả bộ sạch.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 2 · false 5 · maybe-false 0.

- medium · patch -- `export.rs::reviewer_import_preview` đổi `ReviewCopyError::Sql` thành `reviewer_import_unreadable`; lỗi truy vấn hiện thành lời khuyên chọn tệp khác. Sửa: trả `Sql` cho `store.read` để thành lỗi kho.
- low · patch -- `confirm_import` chỉ so cặp (mục, Chương), còn hàng ghi xuống dựng lại theo asset hiện tại, nên số hàng có thể lệch xem trước. Sửa: so cả hàng.
- low · patch -- comment mới trong `segment_contract.rs`, `pinned_contract.rs`, `reviewer_import_contract.rs`, `reviewerImport.test.ts`, `reviewerImportState.ts` mang mã story và lịch sử, trái luật comment của AGENTS.md. Sửa: xoá các phần đó.
- false -- `resetReviewerImport` "chết": `check:panel-refs` buộc mọi tệp có ô nhớ cấp module có hàm `reset*`; khuôn này giống `tmImportState.ts`.
- false -- plan chưa cập nhật và chưa chạy cả bộ: cả bộ đã chạy (cargo, build, cổng); sửa plan thì bị luật triage bác.
- false -- `file_name` rỗng: đường dẫn đến từ hộp thoại chọn tệp, luôn có tên tệp.
- false -- `from_column` trả `Unreadable`: `CHECK` của v30 chặn mọi giá trị khác, không chạm tới được.
- false -- cổng quét chỉ cần nhắc `review_chapter`: đúng AC đã duyệt ("không nhắc"); việc làm đủ của gộp/tách do hai ca hành vi canh.

## Design Notes

- **Khớp `.docx`**: điểm của một cặp (mục tệp, Chương) là số ô trái trùng nguyên văn `source_text` của một segment sống trong Chương đó. Mục tệp được gán cho Chương có điểm cao nhất và điểm phải lớn hơn 0. Nếu hoà thì tiêu đề trùng sẽ phá thế hoà; vẫn hoà thì mục đó coi là không khớp. Hai mục cùng nhắm vào một Chương thì cả hai coi là không khớp.
- **Khớp `.md`**: tiêu đề sau khi bỏ escape phải trùng `chapter.title`; với Chương không có tên thì so với `Chương {ord}`. Tiêu đề trùng với hơn một Chương thì mục đó coi là không khớp.
- Mục không khớp được liệt kê trong xem trước là "bỏ qua" và không ghi. Khi không mục nào khớp thì báo lỗi, không có xem trước (Decision của Ice).
- **Hàng ảnh `.docx`**: hai ô giống nhau **và** trùng tham chiếu của một asset thuộc Chương (URL, `file_name`, hoặc `…-anh/<copied_name>`). Hai ô giống nhau thôi thì chưa đủ, vì một câu chưa dịch cũng có hai ô giống nhau.
- **Phạm vi cổng quét**: chỉ bắt `UPDATE segment SET chapter_id`, không bắt `INSERT`. Gộp hay tách câu (`write_regroup`, các hàm nhập) chèn segment vào cùng Chương mà ranh giới Chương không đổi, và AD-52 đã loại phương án báo lỗi thời khi đó. Cổng quét `src/` bằng `code_lines`, không quét test. Có một mẫu mồi để chứng minh cổng đỏ được.

## Verification

**Commands:**
- `cd src-tauri && cargo test --test reviewer_import_contract` -- xanh; mỗi phép gỡ đối chứng làm đỏ đúng ca của nó
- `npx vitest run tests/frontend/reviewerImport.test.ts` -- xanh
- Story chạm migration, `lib.rs` và `vitest`/`eslint` config, tức dây dùng chung ⇒ chạy cả bộ một lần bằng tay trước khi báo xong
