---
type: ad-brief
title: "Hồ sơ bàn giao cho Winston — một `AD` mới: **thư viện diff cho dải khớp mờ TM, và xuất xứ khi nhận gợi ý khớp mờ**"
status: done
created: 2026-10-02
skill: bmad-architecture
---
# Hồ sơ bàn giao cho Winston — một `AD` mới: **thư viện diff cho dải khớp mờ TM, và xuất xứ khi nhận gợi ý khớp mờ**

**Ngày:** 2026-10-02 · **Người bàn giao:** lượt `bmad-build` mở Story 7.5 · **Người nhận:** Winston (architect)
**Nguồn gốc:** FR59 (*"hiển thị các bản dịch cũ tương tự kèm phần trăm khớp và diff phần khác biệt"*), spec `implementation-artifacts/spec-7-5-khop-mo.md`.
**Quyết định của Ice (2026-10-02):** câu 2 = (A) kéo lựa chọn thư viện diff về Story 7.5, đo trên câu TM thật; câu 3 = (A) nhận gợi ý khớp mờ đặt xuất xứ **người khác dịch**. Phương án bị loại: tự viết diff LCS trong mã dự án (2B), chỉ hiện phần trăm (2C), giữ xuất xứ của cặp TM (3B).
**Baseline cây nguồn:** `d028a95aadb92a80b6a8a906194f8caeba899957`. Số `AD` trống kế tiếp: **51** (spine dừng ở AD-50; không hồ sơ `ad-brief-*` nào đã nhận 51).

## 1. Hai chỗ spine đang chặn Story 7.5

1. **Hàng Deferred `similar` vs `dissimilar`** (spine §Deferred, *"Giai đoạn 5 — thử cả hai trên bản review thật"*) và chú thích `src-tauri/Cargo.toml:164-167` (*"Cài một trong hai hôm nay là âm thầm đóng một quyết định kiến trúc đang mở"*). FR59 cần diff ngay ở Epic 7, trước Story 8.1.
2. **AD-47 ③ là danh mục ĐÓNG** và không có dòng cho *"Nhận gợi ý TM khớp mờ"*. Lượt nhận đi qua đúng một người ghi AD-50 (`write_non_user_target`), nên nó phải khai xuất xứ nó đặt.

## 2. Câu hỏi cho `AD`

1. **Đo gì để chọn.** Đề xuất: chạy cả hai crate trên các cặp câu nguồn cũ/mới thật (Zh và En) mà dải khớp mờ sẽ thấy — câu gần giống trong cùng một Tác phẩm thật — và so: diff cấp ký tự/grapheme cho tiếng Trung (không ranh giới từ), cấp token cho tiếng Anh, số đoạn vụn khó đọc, thời gian trên 3 gợi ý mỗi lượt. Ice chọn trên số đo (AGENTS.md: hai phương án hợp lệ ⇒ trình cả hai kèm số đo).
2. **Một thư viện cho cả 7.5 và Diff Viewer của Epic 8, hay chỉ cho 7.5.** Nếu một: hàng Deferred đóng, Story 8.1 còn lại phần thử trên bản review thật (xác nhận hoặc lật). Nếu hai ca dùng tách: Story 8.1 giữ nguyên câu hỏi của nó cho Review Mode.
3. **Chỗ đặt diff.** Diff tính phía Rust (trả về đoạn `equal`/`insert`/`delete`) hay phía webview. Gợi ý: phía Rust, cạnh hàm chấm điểm trong `core/matching` (AD-17, một chỗ), module vẫn không I/O (AD-13/15).
4. **Dòng mới AD-47 ③:** *"Nhận gợi ý TM khớp mờ (FR59) → người khác dịch"*, cùng hàng với *"Đưa đề xuất AI sang Editor"* và *"Chấp nhận thay đổi từ Review Mode"*. Mốc AD-50 đặt bằng văn bản cặp + `other`; người dùng sửa một ký tự rồi ký ⇒ `self` qua `arbitrate`, ký không sửa ⇒ `other` kể cả khi cặp gốc là của tôi (Ice đã chấp nhận hệ quả này).
5. **Giấy phép (NFR15):** đọc giấy phép trong mã nguồn ĐÃ TẢI của crate được chọn và ghi vào bảng Stack trước khi thêm.

## 3. Thứ Story 7.5 làm được trước khi `AD` này có

Hàm chấm điểm trong `core/matching`, lệnh đọc gợi ý trên hai tầng (phần trăm, thứ tự, ngưỡng), khoá cài đặt ngưỡng, dải UI với phím. Phần diff và lệnh nhận gợi ý dừng chờ `AD`.
