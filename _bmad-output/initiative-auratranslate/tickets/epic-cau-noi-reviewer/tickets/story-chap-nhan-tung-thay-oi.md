---
id: 13
type: story
title: "Chấp nhận từng thay đổi"
parent: epic-cau-noi-reviewer
covers: [FR94]
after: [10, 12]
hitl: false
risk: medium
---

# Chấp nhận từng thay đổi

## Description

Người dịch được: chọn lấy những sửa đổi tôi đồng ý và bỏ những chỗ tôi không. Mục đích: reviewer là người góp ý chứ không phải người quyết định.

Một thay đổi là một nhóm alignment mà bản của tôi và bản reviewer khác nhau, đúng đơn vị `review_diff` của 8.12 trả về. Trong Review Mode, người dùng chấp nhận hoặc bỏ qua từng nhóm một. Không có lệnh nhận tất cả.

Chấp nhận chỉ áp dụng cho nhóm 1:1, tức một segment của tôi đối một hàng reviewer. Nhóm n:1, 1:m, n:m và nhóm một phía vẫn hiện diff, nhưng mang lời báo rõ rằng không chấp nhận được và người dùng sửa tay trong Editor.

Chấp nhận ghi văn bản của hàng reviewer vào segment ngay lập tức, không qua bộ đệm gõ (AD-35). Lượt ghi này là lượt ghi không-phải-người-dùng: segment về **chưa xác nhận**, không tạo `SegmentVersion` (AD-31), xuất xứ là **người khác dịch**, mốc so và cột xuất xứ được đặt trong cùng một lệnh ghi (AD-47 ③, AD-50). Bản reviewer không bao giờ bị sửa (AD-52).

Nếu segment đang là bản nháp chưa có bản sao trong `segment_version`, chấp nhận không ghi gì mà hỏi lại người dùng trước, đúng khuôn của lấy TM mờ và đưa bản AI vào (AD-49 lớp iii).

Trạng thái đã chấp nhận hoặc đã bỏ qua của từng nhóm được lưu, khoá theo hàng reviewer và `review_chapter_id`, và bị xoá cùng giao dịch với bản reviewer (AD-52 ⑥ ⑦). Nhờ đó nhóm đã bỏ qua không còn hiện là thay đổi chưa xử lý, và Review Mode biết khi nào đã xử lý hết. Màn hình xem trước của lần nhập lại nêu tên các nhóm đã chấp nhận sẽ mất cùng bản reviewer cũ (AD-52 ④).

Chấp nhận và bỏ qua là command đăng ký, gán phím được.

Tiêu chí gốc (epics.md, nguyên văn):

**Given** một thay đổi trong Review Mode
**When** người dùng chấp nhận
**Then** thay đổi đó vào bản dịch của mình

**Given** một thay đổi
**When** người dùng bỏ qua
**Then** bản dịch của mình giữ nguyên

**Given** một segment vừa nhận thay đổi
**When** ghi
**Then** trạng thái về **chưa xác nhận**
**And** **không** tạo `SegmentVersion`

**Given** một thay đổi vừa được chấp nhận
**When** ghi xuống
**Then** ghi **ngay**, không đi qua bộ đệm gõ — đây là một hành động dứt khoát của người dùng

**Given** nhiều thay đổi
**When** người dùng xử lý
**Then** chấp nhận hoặc bỏ **từng cái một**, không có thao tác nhận tất cả một cách mù

**Given** người dùng đã xử lý hết
**When** rời Review Mode
**Then** các segment đã nhận thay đổi hiện ở trạng thái chưa xác nhận trong Editor

**Given** lệnh chấp nhận và bỏ qua
**When** gọi
**Then** là command đăng ký, gán phím được


## Acceptance Criteria

Verify: Trên một Chương có bản reviewer đã nhập, chấp nhận một nhóm 1:1 thì segment mang văn bản reviewer, ở trạng thái chưa xác nhận, xuất xứ người khác dịch, và không có hàng `segment_version` mới. Bỏ qua một nhóm khác thì segment giữ nguyên và nhóm không còn hiện là chưa xử lý, kể cả sau khi đóng rồi mở lại Review Mode. Chấp nhận đè lên một bản nháp chưa có bản sao thì không ghi gì cho tới khi người dùng đồng ý. Một nhóm n:1 không chấp nhận được và có lời báo.

## References

- parent — _bmad-output/initiative-auratranslate/epic-cau-noi-reviewer/epic-cau-noi-reviewer.md
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-31, AD-35, AD-47, AD-49, AD-50, AD-52
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/review-mode.html
- plan — _bmad-output/initiative-auratranslate/epic-cau-noi-reviewer/story-segment-alignment-may-khop-nguoi-sua-plan.md
- plan — _bmad-output/initiative-auratranslate/epic-cau-noi-reviewer/story-diff-boi-mau-an-van-ban-goc-plan.md

## Notes

- Decision (2026-10-09, Ice): một thay đổi là một nhóm alignment, không phải từng span. Span của AD-51 không có định danh.
- Decision (2026-10-09, Ice): chỉ chấp nhận được nhóm 1:1. Văn bản reviewer của nhóm n:m đã được nối lại và không tách ngược được về từng segment, và chưa AD nào nói cách chia. Hệ quả: với bản review nhập từ `.md`, đoạn nhiều câu là n:1 nên phần lớn không chấp nhận được.
- Decision (2026-10-09, Ice): ghi đè bản nháp chưa có bản sao thì hỏi lại theo khuôn `needs_confirmation`/`force` của lấy TM mờ và đưa bản AI vào.
- Open question: segment đã đổi trong Editor sau khi diff được tính thì chấp nhận so với văn bản nào. Mặc định: đọc lại văn bản hiện tại ở Rust và từ chối nếu nhóm không còn khác như lúc hiện.
- Open question: phím mặc định. Mockup dùng ↵ và ⌫. Alt+↑/↓ đã thuộc `review.diff_next`/`review.diff_prev`.
- Open question: sửa nhóm alignment (nối tay lại, segment về hưu) thì trạng thái đã chấp nhận hoặc đã bỏ qua của nhóm cũ ra sao.
- Sổ nợ: hai mục `Chủ: Epic 8` về hàng FR94 của bảng AD-47 ③ (mốc so và xuất xứ) và một mục về nhãn "từ bản review" trên `segment_version`. Grep `deferred-work.md` theo `FR94`.
