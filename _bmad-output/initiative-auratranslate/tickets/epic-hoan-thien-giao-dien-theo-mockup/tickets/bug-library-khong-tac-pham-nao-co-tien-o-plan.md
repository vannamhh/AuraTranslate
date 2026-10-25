---
title: 'Library: không Tác phẩm nào có tiến độ'
type: 'bugfix'
ticket: '23'
created: '2026-10-25'
status: 'built'
baseline_revision: '9e22db650050f0a95a9461c21a05d7fe6ec38fd0'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: [quick]
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Cả 47 thẻ Library của Ice ghi "Chưa biết tiến độ": `meta.json` của cả 47 là bản 1, viết trước khi Story 5.4/5.5 thêm `status` và `chapter_done_count`, và không đường nào nâng một `meta.json` cũ lên bản 3 trừ khi trạng thái Chương đổi.

**Approach:** Chốt nguyên nhân bằng số đo, giữ nguyên mã (Decision A), và khoá đường meta → chỉ mục → `list_works` của tiến độ cho Tác phẩm mới bằng một test (tiêu chí 3).

- Decision: nguyên nhân là dữ liệu, không phải mã ghi sai. 47/47 `meta.json` có `meta_schema_version: 1` (0 ở bản 2, 0 ở bản 3); 47/47 hàng `library_work` có `chapter_done_count IS NULL` và `status IS NULL`, `updated_at` 2026-08-06..26. Không đường nào đã dựng lại chúng: chỉ `create_work` và các lệnh đổi trạng thái/tổ chức Chương gọi `rebuild_from_store` + `write_atomic`. Bằng chứng mở không dựng lại: 3 Tác phẩm mang 12 tệp `project.db.bak-vN` ghi 2026-08-29..10-15 (tức `open_work` của bản sau 5.5 đã chạy và di trú `project.db`), mà `meta.json` vẫn bản 1. Trong `project.db`, 47/47 có đúng 1 Chương, 0 Chương `done` (2026-10-25).

- Decision (Ice, 2026-10-25): chọn A — không đổi mã sản phẩm; `NULL` vẫn là "chưa biết", story 24 chỉ ẩn chữ. Số đo trình Ice trên bản chép 47 Tác phẩm (build debug, tải máy 11–13, hai lượt): A tốn 0; B1 (nâng lúc mở) mở thêm 0 `project.db`, ~42 ms p50 + một lượt `Indexer::rebuild` 0,1–0,8 s mỗi Tác phẩm cũ một lần; B2 (nâng lúc quét/FR140) `Store::open` 47 `project.db`, di trú 46, sinh 46 `.bak-vN`, 7,5–8,4 s lượt đầu. Bug đóng theo tiêu chí 5, tiêu chí 4 không áp dụng.

## Boundaries & Constraints

**Always:** `meta.json` chỉ ghi qua `WorkMeta::rebuild_from_store` + `write_atomic`, ngoài closure `Store::write` (AD-33); `.atproj` ghi trước, chỉ mục sau (AD-8). `NULL` vẫn nghĩa là chưa biết, không thành `0`.

**Never:** Không đổi mã sản phẩm (Decision A). Không sửa dữ liệu thật của Ice. Không để `Indexer` ghi `meta.json` hay tự tính tiến độ từ `project.db` (cần AD mới của Winston). Không mở `project.db` bằng `Store::open` trong lượt quét Library nếu Ice chưa chọn phương án đó.

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/library/meta.rs` -- `META_SCHEMA_VERSION = 3`, `WorkMeta::read` (nhận bản cũ, trường mới thành `None`), `rebuild_from_store` (luôn `Some`), `write_atomic`. Không đổi.
- `src-tauri/src/core/library/indexer.rs:310,421` -- quét đọc `meta.json`, UPSERT nguyên giá trị vào `library_work`; không tự tính.
- `src-tauri/tests/lifecycle_contract.rs:513` -- `setting_a_chapter_status_leaves_the_new_value_in_the_library_index`: đi đúng đường meta → chỉ mục → `list_works` nhưng chỉ khẳng định `status`; `status_in_index` `:499` chưa trả `chapter_done_count`.
- `src-tauri/tests/library_commands_contract.rs:700` -- ca v2 thiếu khoá ⇒ `None`; giữ nguyên.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/tests/lifecycle_contract.rs` -- thêm ca: Tác phẩm mới, đặt Chương `done` qua `set_chapter_status_indexed`, `list_works` trả `chapter_done_count == Some(1)` (trước lượt ghi: `Some(0)`, không `None`) -- tiêu chí 3; đối chứng đỏ bằng cách gỡ cột khỏi UPSERT ở `indexer.rs:438`.

**Acceptance Criteria:**
- Given Tác phẩm vừa tạo có một Chương `done`, when Library tải danh sách, then hàng của nó có `chapter_done_count = Some(1)`, và ca test đỏ khi đường meta → chỉ mục bị gỡ.
- Given Ice chọn A, when ca trên xanh, then không đổi mã sản phẩm; bằng chứng ở Implementation Notes (tiêu chí 5).

## Implementation Notes

- Tiêu chí 5 (Decision A): không đổi mã sản phẩm; diff chỉ có `src-tauri/tests/lifecycle_contract.rs`. Bằng chứng nguyên nhân nằm ở hai dòng `Decision:` của Intent.
- Ca `marking_a_chapter_done_leaves_the_done_count_in_the_library_index` đi đúng đường `set_chapter_status_indexed` → `meta.json` → `Indexer::rebuild` → `list_works`; 1 ca chạm tới đường này với `chapter_done_count` khác `NULL`.
- Đối chứng đỏ: gỡ `chapter_done_count = excluded.chapter_done_count` khỏi `DO UPDATE SET` ở `indexer.rs` ⇒ ca đỏ (`Some(Some(0))` ≠ `Some(Some(1))`); trả lại ⇒ xanh.
- Phát hiện kèm: 35/47 Tác phẩm của Ice bị bỏ qua thu hoạch văn bản (`SchemaTooOld`) cho tới khi được mở — đúng thiết kế, cùng khuôn với tiến độ "chưa biết" mà A giữ.

## Plan Change Log

## Review Triage Log

- Lượt 1 (quick): high 0 · medium 0 · low 3 · false 5 · maybe-false 0; không patch, không defer.
- low, bác — ô task còn `[ ]`: sửa plan, không sửa mã; đã đánh dấu ở bước trình bày.
- false — Verification không ghi kết quả: tự chạy lại `cargo test --test lifecycle_contract` 19/19 xanh và đối chứng gỡ cột UPSERT đỏ `Some(Some(0))` ≠ `Some(Some(1))`, rồi `git checkout` trả `indexer.rs`.
- false — chỉ một trong ba seam được đối chứng: lượt ghi `meta.json` và bước 4 `reindex_after_lifecycle_write` đã có ca canh (`setting_a_chapter_status_leaves_the_new_value_in_the_library_index` và các ca đọc `meta.json` từ đĩa); seam mới duy nhất là cột, đã đối chứng.
- false — 35/47 `SchemaTooOld` thiếu mục sổ nợ: bỏ qua có chủ ý và hiện ra người dùng qua `RescanReport.text_skipped` (`libraryRescan.ts:165`), không phải rỗng im lặng; không do thay đổi này.
- low, bác — `context: []`: sửa plan; người dựng làm xong từ Code Map.
- false — Triage Log rỗng: được ghi ở chính bước này.
- low, bác — closure `done_count` lặp thân `status_in_index`: bảy dòng trong một test, nới helper đổi chữ ký mà các ca khác dùng.
- false — test đọc `Indexer::list_works` chứ không qua lệnh: `commands/library.rs:664` chuyển thẳng `indexer.list_works`, và `library_commands_contract.rs:700` đã canh `chapter_done_count` tới `WorkRow`.

## Verification

**Commands:**
- `cd src-tauri && cargo test --test lifecycle_contract` -- expected: xanh, ca mới có mặt.

**Manual checks (if no CLI):**
- Không có; số đo tiêu chí 2 đã chốt trong Decision, bằng test tạm ngoài repo trên bản chép `~/Documents/AuraTranslate/`.
