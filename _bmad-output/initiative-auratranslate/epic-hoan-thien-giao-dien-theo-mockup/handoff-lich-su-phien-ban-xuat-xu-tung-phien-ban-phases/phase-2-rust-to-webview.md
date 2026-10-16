# Handoff Pha 2 Rust -> Pha 3 Webview (ticket 19)

## Đã làm (chỉ mã sản phẩm Rust, chưa đụng test cũ nào, chưa commit)
- `core/store/schema.rs`: `SEGMENT_VERSION_ORIGIN_DDL` (bậc 39: cột `translation_origin` + CHECK + trigger `segment_version_requires_translation_origin`); doc-comment "đúng bốn cột" đã thay.
- `commands/project/mod.rs`: `reject_unknown_translation_origin` có thêm nhánh `segment_version` (chạy sau nhánh `segment`).
- `core/segment/translation_origin.rs`: `VersionText` + `version_matching_text(versions, text) -> Option<i64>` (id lớn nhất, trim + NFC).
- `commands/segment/confirm.rs`: INSERT `segment_version` ghi `confirmed_origin.as_str()`.
- `commands/segment/history.rs` + `wire.rs`: `read_segment_history` trả `SegmentHistory { current_version_id: Option<i64>, source_text, versions: Vec<SegmentVersionRow> }`; hàng có `translation_origin: Option<String>` và `spans: Vec<DiffSpan>`. Khôi phục đặt `baseline_translation_origin` theo hàm chọn (`NULL`/không khớp ⇒ `other`).

## Dây cho Pha 3 (snake_case, không `rename_all`)
```
{ current_version_id: number|null, source_text: string,
  versions: [{ id, segment_id, target_text, created_at,
               translation_origin: 'self'|'other'|'bilingual_import'|null,
               spans: [{ kind: 'equal'|'delete'|'insert', text }] }] }
```
Thứ tự `versions`: mới nhất trước (không đổi). Hàng đang dùng: `spans` toàn `equal`. `current_version_id` null ⇒ bản đang soạn chưa ký hoặc không khớp phiên bản nào. `RestoreOutcome` không đổi. Segment về hưu vẫn đọc được (có `source_text`).

## Đo (AD-55 mục 5)
Segment 60 phiên bản (mỗi văn bản ~70 ký tự, 3 giá trị xuất xứ), build `test` debug (chưa tối ưu, rustc Homebrew), máy Ice lúc rảnh: một lượt đọc 4,97 ms; trung bình 20 lượt 4,43 ms. Ngưỡng 50 ms: đạt, không dừng.
Phép thử tạm (đã xoá, không nằm trong repo) cũng kiểm: INSERT `NULL` bị trigger chặn; khôi phục hàng `other` / `self` rồi ký ⇒ `segment.translation_origin` đúng; khôi phục hàng `NULL` rồi ký ⇒ `other`; sau khôi phục `current_version_id` khác null và có hàng toàn `equal`.

## Chưa làm / việc của Pha 4 (đã biết)
- `cargo check` lib xanh; `cargo fmt` đã chạy. Các tệp test trong Code Map CHƯA sửa: chúng sẽ không biên dịch/đỏ cho tới Pha 4 (21 chỗ `read_segment_history(` trả struct mới, `[Migration; 38]`, `IN (28…38)`, CHECK/bốn cột ở `segment_contract.rs:1299-1308,1596-1616`, INSERT SQL thẳng thiếu cột xuất xứ).
- Chưa chạy bộ test nào ngoài phép thử tạm (cargo test tmp 1/1 xanh).
- Webview (`src/config/segment.ts` `isSegmentVersionArray`/`readSegmentHistory`, e2e `segment-history-restore.e2e.mjs:71` đọc mảng trần) đang gãy với dây mới cho tới Pha 3.
