# Review AD-53 theo rubric — 2026-10-10

Phạm vi đọc: AD-53 (spine :911–:952), AD-3, AD-13, AD-47 (③④⑤⑥), AD-49, AD-50, dòng Consistency Conventions "Ghi nhớ proofreader", `tickets.toml` entry 10, 2, 3, 4, 6, FR83–FR86, `commands/segment/targets.rs::write_non_user_target`. Không đọc phần còn lại của spine.

## Kết luận: pass-with-fixes

Cốt lõi đúng và khớp mã: lượt ghi đi qua `write_non_user_target` (đã ghi đủ text + hai mốc + status trong một câu UPDATE), xuất xứ chuyển tiếp từ hàm phân xử AD-50 mục 4 nên không nới tập ba giá trị AD-47 ⑥; AD-47 ③ thêm đúng một hàng (dòng :718 đã có); AD-3 :95 đã có 🔵 và hết mâu thuẫn với dòng Consistency Conventions ⇒ verify của entry 10 đạt; ba câu hỏi của entry 10 đều có đáp (mục 1–3 / 6 / 4–5); Prevents/Binds/Rule đủ. Các lỗ còn lại là chỗ hai story (9.4, 9.6, và 9.2/9.3 ở rìa) vẫn có thể hiểu khác nhau.

## Đối chiếu từng mục rubric

| Mục | Kết quả |
|---|---|
| Chỉ ra điểm phân kỳ thật cho 9.2/9.3/9.4/9.6 | Đủ ở 9.4 và 9.6; còn hở: phép kiểm dùng chung (F1), chuẩn hoá chữ thay (F3), "bỏ qua" 9.4 vs 9.6 (F4) |
| Mỗi mệnh đề Rule thi hành được và ngăn đúng phân kỳ | Mục 1, 2, 3, 6 ổn. Mục 4 và 5 chưa ngăn được hai cài đặt khác nhau (F1, F3) |
| Trả lời đủ 3 câu + verify | Đạt. Câu 1 ("hai phương án ⇒ trình Ice cả hai kèm số đo") giải bằng phương án loại có lý do, nhưng số đo chỉ là suy luận từ mã (đã ghi ⚠️ ở cuối) |
| Không làm yếu/mâu thuẫn AD-47/49/50/13/35/1 | AD-47, 50, 13, 35, 1 sạch. AD-49: có căng, xem F2 |
| Chỗ để mở (N) không cho hai story lệch nhau | N chỉ 9.6 dùng ⇒ không lệch giữa story, nhưng thiếu cận dưới và tác động khi đổi N (F4) |
| Prevents/Binds/Rule | Có đủ |
| Văn phong cho agent nhỏ | Mục 6 và 5 còn câu mơ hồ (F3, F4) |

## Phát hiện

**F1 (cao) — Bốn phép kiểm mục 4 sống ở hai module nhưng không có hàm dùng chung.** Chấp nhận/hoàn lại nằm ở `commands/segment/` (vì `write_non_user_target` là `pub(super)`), "bỏ qua" nằm ở `commands/proofread.rs`, cả ba "kiểm trong cùng giao dịch, theo thứ tự" nhưng AD không nói ai sở hữu phép kiểm. 9.4 và 9.6 sẽ cài hai bản; thứ tự lỗi, đơn vị UTF-16, cắt đôi surrogate dễ lệch (Prevents 3 đúng loại này). Sửa: thêm vào mục 4 một câu "phép kiểm là MỘT hàm thuần trong `core/segment/` (đầu vào: văn bản hiện tại, văn bản/khoảng/cụm mong đợi; đầu ra: `Ok` hoặc một trong ba mã lỗi) mà cả ba lệnh gọi; không lệnh nào tự so", kèm cổng như AD-50 mục 4.

**F2 (cao) — AD-49 phần Rule mở đầu: "không trạng thái hoàn tác nào sống qua một lượt ghi".** Đường lui mục 3 của AD-53 là cụm gốc giữ trong bộ nhớ webview qua đúng lượt ghi chấp nhận; đó là trạng thái hoàn tác sống qua lượt ghi, và nó chỉ trả lại nội dung khi webview còn nhớ, trong khi AD-49 mục 2 (i) đòi lệnh nghịch đảo "trả lại mọi nội dung người dùng". Mục 7 của AD-53 lại viết AD-49 "không đổi chữ". Sửa: hoặc (a) đổi mục 7 thành "AD-49 mục 2(i) được áp với đường lui sống trong webview, một ngoại lệ có tên của câu mở đầu", thêm 🔵 tại AD-49; hoặc (b) xếp lượt chấp nhận vào (ii) với lý do cụm bị thay do model chỉ ra, và bỏ chữ "(i)".

**F3 (trung bình) — Chữ thay và khoảng còn mơ hồ.** (a) Không nói chữ đề xuất được ghi nguyên văn hay NFC/trim; model trả NFD tiếng Việt sẽ chèn chuỗi phân rã vào `target_text` và `len UTF-16` của hoàn lại phụ thuộc điều này. (b) Mục 5: "nằm trước/sau/chồng" không định nghĩa khi `end == start` của khoảng vừa thay, hoặc khoảng rỗng (đề xuất chèn). (c) Hoàn lại không nói webview dời lại các phát hiện theo chiều ngược. Sửa: "chữ đề xuất ghi nguyên văn byte, webview/Rust không chuẩn hoá nó; dùng độ dài UTF-16 của đúng chuỗi đã ghi"; "trước ⇔ `end ≤ start_thay`, sau ⇔ `start ≥ end_thay`, mọi trường hợp khác (kể cả khoảng rỗng tại biên) là chồng ⇒ bỏ"; "hoàn lại dời theo chênh lệch ngược lại, cùng ba luật".

**F4 (trung bình) — Chữ ký mục 6 còn ba chỗ hai agent có thể hiểu khác.** (a) Ngữ cảnh lấy quanh cụm đã `trim` hay chưa `trim`? (b) NFC áp trước hay sau khi cắt N đơn vị UTF-16, và cắt giữa cặp surrogate thì sao? (c) Đổi N (nhỏ lại) làm mọi hàng đã lưu hết khớp "đuôi của" ⇒ ghi nhớ chết im lặng, đúng lớp "im lặng" của repo. Ngoài ra "bỏ qua" của 9.4 (FR83, từng phát hiện) và "không phải lỗi" (FR84, 9.6) chưa được nói là cùng một lệnh hay hai; nếu 9.4 dựng một bản chỉ phiên làm việc thì 9.6 sẽ có hai nút. Sửa: NFC cả chuỗi segment trước, rồi cắt N, lùi ra biên code point; ngữ cảnh tính quanh cụm đã trim; AD nêu "N chỉ được tăng sau khi có dữ liệu lưu, giảm cần migrate"; và một câu "'bỏ qua' ở 9.4 chính là lệnh ghi nhớ của mục 6, 9.6 chỉ thêm danh sách/bỏ ghi nhớ".

**F5 (thấp)** — (a) AD-4 Prevents (:100) còn nói ghi nhớ proofreader "trỏ sai chỗ" khi tách lại segment; sau AD-53 ghi nhớ không trỏ segment nữa. Thêm 🔵 hoặc sửa cùng AD-3. (b) Phép kiểm mục 4 không kiểm segment thuộc Chương đang mở như 9-9/C6 (`segment.not_in_open_chapter`) đã vá cho promote; văn bản trùng làm phép kiểm văn bản không bắt được. Sửa: thêm vế thứ năm vào danh sách kiểm hoặc nói rõ vì sao không cần. (c) "loại" trong khoá phải là chuỗi ổn định do 9.1/9.2 khai (không dịch/đổi tên); ghi vào mục 6.

## Ghi chú không phải lỗi
- Lọc ở `core/ai/proofread.rs` đọc `project.db` hợp lệ theo AD-13 (chiều `ai/` đọc `segment/`).
- Ca "đề xuất ngữ pháp viết lại cả mệnh đề trong câu người dùng vẫn ra *tôi dịch*" và mất mốc khi người dùng gõ đè đã ghi ⚠️ đúng chỗ.
- Số đo: bảng xuất xứ suy từ mã, chưa chạy; AD đã tự nhận. Story 9.4 nên có ca kiểm bốn hàng của bảng làm bằng chứng.
