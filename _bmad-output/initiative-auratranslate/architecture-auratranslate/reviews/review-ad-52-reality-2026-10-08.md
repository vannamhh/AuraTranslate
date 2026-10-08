# Review thực tế AD-52 (2026-10-08)

Phạm vi: đối chiếu từng khẳng định của AD-52 (spine dòng 848-877) với mã. Đường dẫn tính từ `src-tauri/`.

## Phán quyết
Nền đúng, có một lỗ thật ở mục 5 (gộp Chương) và một lỗ thiết kế ở chỗ "người ghi" của `stale_at`. Sửa hai chỗ đó thì AD đứng được.

## Đã xác minh đúng
- Không FK: `src/core/store/schema.rs:946-947, 976, 1021, 1255` đều ghi "PRAGMA foreign_keys cố ý tắt, không FOREIGN KEY". `UNIQUE(chapter_id)` vẫn dùng được (UNIQUE không phụ thuộc pragma).
- AUTOINCREMENT: `CHAPTER_DDL`/`SEGMENT_DDL` (`schema.rs:147-160, 1515-1548`); `chapter.id` không tái dùng.
- Đường đổi ranh giới Chương, kiểm bằng grep `SET chapter_id`, `DELETE FROM chapter`, `INSERT INTO chapter` toàn `src/`:
  | Đường | Vị trí | Một giao dịch? |
  |---|---|---|
  | Gộp | `commands/chapter.rs:793-940` (`UPDATE segment SET chapter_id` :872, `UPDATE asset` ~:897, `DELETE FROM chapter_position` + `DELETE FROM chapter` :929-930) | có, `store.write(tx)` |
  | Tách | `chapter.rs:983-1110` (`INSERT chapter` ~:1045, `UPDATE segment` :1063, `UPDATE asset`, `UPDATE chapter_position`) | có |
  | Bản `_indexed` | `chapter.rs:1178, 1194` | chỉ bọc `finish_lifecycle_write`, không thêm SQL |
  | `move_chapter` | `chapter.rs:680-760` | chỉ đổi `chapter.ord`, không đổi ranh giới |
  | Tạo Chương lúc import | `commands/project/work_creation.rs:473, 863` | tạo mới, chưa có bản reviewer |
  Không có đường nào khác `DELETE FROM chapter` hay gán lại `segment.chapter_id`. Tiền lệ `chapter_position`/`asset` ghi ngay trong hai giao dịch này (`schema.rs:953-957, 1242-1249`), nên thêm một `UPDATE review_chapter SET stale_at` là rẻ.
- `READING_MARK`: `schema.rs:967-984`. Giữ `segment_id` gốc, thêm `navigation_segment_id`; `commands/segment/regroup.rs:335-345` chuyển con trỏ thứ hai trong cùng giao dịch retire + insert. Là khuôn hợp lệ, nhưng "sống sót" là do một người ghi tay làm, không tự có.
- AD-47 ③ (spine ~708-725): các hàng là FR115, FR94, FR58, đề xuất AI, FR59, gộp/tách, FR101. FR90 không có. Đúng, miễn là lượt nhập chỉ ghi `review_*`.
- Tên `review_*`: không có bảng hay hàm nào trùng. Chỉ có chuỗi `review_harvest` (`core/glossary/candidate.rs:39`, `entry.rs:109`) và `needs_review` (`commands/project/mod.rs:1248`); không va chạm.
- `.md`: `core/export/text_export.rs:181-182` phát `![alt](dest)` rồi `*caption*`; `:192-193` phát `[Ảnh: alt] dest` rồi caption. Alt và caption nằm trong dòng ảnh, nên cột `kind` là cần.
- `.docx` hai cột: `docx_table.rs:53-60` cho hàng ảnh chỉ đường dẫn/tên tệp, không alt, không caption. Alt/caption ở đó là segment có `role` và thành hàng chữ thường (`images.rs` ghi chú "vai alt/caption vẫn là hàng").

## Phát hiện

1. **Cao — mục 5 sai với gộp Chương.** Gộp xoá hàng `chapter` của B (`chapter.rs:930`) rồi đặt `stale_at` cho "mọi Chương nó chạm" chỉ có nghĩa với A. `review_chapter` của B vẫn sống (không FK), `chapter_id` B không bao giờ dùng lại nên không bị thay; FR54 và thu hoạch đếm một bản mồ côi mãi mãi. Sửa: ghi rõ gộp phải `DELETE FROM review_row/review_chapter` của B (hoặc đặt `stale_at`, và mọi truy vấn JOIN `chapter`), cộng đặt `stale_at` cho A. Tách: đặt `stale_at` cho A, B mới chưa có bản.
2. **Cao — người ghi `stale_at` là ghi theo đường, dễ sót.** Mục 5 nhờ mỗi đường đổi ranh giới nhớ thêm một câu UPDATE; đường tương lai (FR gộp nhiều Chương, import lại) quên là im lặng. Rẻ hơn và không cần người ghi: lưu trên `review_chapter` một dấu vân tay ranh giới lúc nhập (số segment sống + `MIN/MAX(segment.id)` của Chương), Review Mode so lúc đọc. Nếu giữ `stale_at`, thêm ca đối chứng gỡ UPDATE khỏi gộp và tách (theo AGENTS.md) làm đỏ đúng hai ca.
3. **Trung bình — `kind` không phân biệt được ở `.docx`.** `.docx` hai cột xuất alt/caption như hàng chữ thường, nên nhập `.docx` luôn cho `kind='text'`; chỉ `.md` mới cho `alt`/`caption`. Ghi rõ trong mục 1 để 8.10 không giả định có `alt` từ `.docx`.
4. **Thấp — mục 6 trích `READING_MARK` như thể liên kết tự sống sót.** Nên nói: 8.10 phải có người ghi cùng giao dịch với retire trong `write_regroup`, như `regroup.rs:341`; nếu không, đúng lỗi AD-52 muốn tránh.
5. **Thấp — bước di trú.** Thêm hai bảng là hai bước `project.db` mới; số bước kế tiếp phải lấy từ danh sách `MIGRATIONS` (`schema.rs:~2165+`), không đoán.

Cơ chế sẵn có AD-52 không bỏ sót đáng kể: khuôn `chapter_position`/`reading_mark` đã được dùng. Chỉ có đề xuất dấu vân tay ở mục 2.
