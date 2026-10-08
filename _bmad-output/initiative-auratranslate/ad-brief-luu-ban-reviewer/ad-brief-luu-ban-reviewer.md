---
type: ad-brief
title: "Hồ sơ bàn giao cho Winston — một `AD` mới: **nơi lưu và vòng đời của bản reviewer đã nhập lại**"
status: done
created: 2026-10-08
skill: bmad-architecture
---
# Hồ sơ bàn giao cho Winston — một `AD` mới: **nơi lưu và vòng đời của bản reviewer đã nhập lại**

**Ngày:** 2026-10-08 · **Người bàn giao:** lượt `bmad-build` mở Story 8.9 · **Người nhận:** Winston (architect)
**Nguồn gốc:** FR90, FR91, FR95. Plan: `initiative-auratranslate/epic-cau-noi-reviewer/story-nhap-lai-file-reviewer-a-sua-plan.md` (draft).
> ✅ **ĐÃ ĐÓNG 2026-10-08 — `AD-52` đã viết** (spine §AD-52). Năm câu hỏi ở §3 được trả lời ở các mục 1–7 của AD đó; Reviewer Gate ở `architecture-auratranslate/reviews/review-ad-52-*-2026-10-08.md`.

**Quyết định của Ice (2026-10-08):** nơi lưu phải có `AD` trước, và 8.9 dừng ở plan cho tới khi có. Phương án bị loại: để plan tự chốt bảng `reviewer_copy` + `reviewer_copy_row`, rồi bổ sung sơ đồ ER sau bằng correct-course.
**Ba quyết định Ice đã ký, ràng buộc `AD` này:**
- `.docx` khớp với Chương theo cột trái (nguyên văn nguồn) so với nguồn của các segment thuộc bản dịch; tiêu đề chỉ dùng để phá thế hoà.
- `.md` khớp theo tiêu đề `##`. Bộ đọc `.md` viết tay theo đúng định dạng 8.6 xuất ra.
- Tác phẩm đích là Tác phẩm đang mở.

**Baseline cây nguồn:** `18fe4eec6070869ccb8736d292f7dadb98948986`. Số `AD` trống kế tiếp: **52**. Spine dừng ở AD-51, và không hồ sơ `ad-brief-*` nào đã nhận 52.

## 1. Chỗ trống trong spine

AD-7 và AD-9 chỉ nói dữ liệu của Tác phẩm nằm trong `.atproj/project.db`. Sơ đồ ER (spine §ER) không có thực thể nào cho bản review. Trong khi đó 8.10 (alignment), 8.11–8.13 (Review Mode, chấp nhận từng thay đổi) và 8.14–8.15 (thu hoạch) đều đọc bản đã nhập. FR95 còn đòi thu hoạch chạy kể cả khi Review Mode không bao giờ được mở. Vậy bản nhập phải sống lâu hơn một phiên làm việc.

## 2. Bất biến đang đứng mà `AD` phải tôn trọng

- **AD-16:** chỉ lưu mô hình có cấu trúc, không trường nào mang HTML.
- **AD-38:** cổng hình dạng chạy trước mọi lệnh ghi.
- **AD-47 ③:** FR90 không có trong danh mục người ghi `target_text`. Chỉ FR94 ghi.
- **Ranh giới đã lưu (AGENTS.md):** gộp hay tách segment là retire + create; gộp hay tách Chương chỉ đổi `chapter_id` và `ord`.
- **AC của 8.9:** không ghi gì trước khi người dùng xác nhận.
- **Tệp xuất không mang id nào.** Nguồn duy nhất để khớp là cột nguồn của `.docx` và tiêu đề Chương.

## 3. Câu hỏi cho `AD`

1. **Hình dạng.** Mỗi lần nhập một hàng, mỗi Chương hay mỗi hàng của tệp một hàng con? Hàng con có lưu bản sao văn bản nguồn hay không (`.docx` có, `.md` không có)? Hàng ảnh đã bị bỏ qua lúc đọc.
2. **Khoá nối.** `chapter_id` (FK) hay chỉ lưu văn bản rồi để 8.10 nối vào segment? Khi Chương bị gộp hoặc tách, hoặc segment về hưu sau lúc nhập, thì bản nhập ra sao?
3. **Vòng đời.** Nhập bản thứ hai của cùng một Chương thì thay bản cũ hay giữ lịch sử? Khi nào bản nhập được xoá: sau khi chấp nhận hết, hay khi người dùng bỏ nó?
4. **Kết quả alignment của 8.10** nằm trong `AD` này hay một `AD` riêng?
5. **Thu hoạch 8.14** có cần cặp segment ↔ đoạn reviewer đã nối sẵn không, hay đọc thẳng văn bản?
