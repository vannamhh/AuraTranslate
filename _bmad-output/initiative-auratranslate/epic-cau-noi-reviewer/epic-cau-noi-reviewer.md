---
type: epic
title: "Cầu nối Reviewer — xuất, nhập lại, đối chiếu, và hấp thụ bài học"
parent: initiative-auratranslate
covers: [FR54, FR87, FR88, FR89, FR90, FR91, FR92, FR93, FR94, FR95, FR121, FR130, FR131]
after: []
assignee: ""
risk: low
status: in-progress
---

# Cầu nối Reviewer — xuất, nhập lại, đối chiếu, và hấp thụ bài học

## Description

Reviewer **không cài AuraTranslate**, nên trao đổi file là cầu nối duy nhất. Người dịch xuất `.docx` bảng hai cột đối xứng theo segment cho reviewer sửa, hoặc **`.docx` một khối đối xứng theo đoạn** để dán thẳng sang trình soạn thảo website. Nhận file đã sửa về: hệ thống khớp cấu trúc đoạn và **hiện ra cho người dùng nối tay** những chỗ không khớp, rồi Review Mode ẩn văn bản gốc và bôi màu thêm/xoá/sửa. Và **kể cả khi người dịch không bao giờ mở Review Mode**, hệ thống vẫn hấp thụ bài học của reviewer vào Glossary.

## Outcome

Người dịch trao đổi file với reviewer không cài AuraTranslate, nhận bản sửa về mà không mất dữ liệu im lặng, và hệ thống đề xuất thu hoạch thuật ngữ từ bản review kể cả khi không mở Review Mode.

## Requirements

- FR54: **Thu hoạch từ bản review:** khi nhập lại bản Reviewer đã sửa, nếu reviewer đổi thuật ngữ *X* thành *Y* một cách **nhất quán**, hệ thống đề xuất bổ sung cặp đó, nêu rõ **số lần đổi trên tổng số lần xuất hiện**.
- FR87: Xuất **`.docx` dạng bảng hai cột**: cột trái văn bản gốc, cột phải bản dịch, **đối xứng theo segment**.
- FR88: Xuất **`.md` hoặc text thuần**, **bảo lưu hình ảnh và alt-text đã dịch** (FR44) **cùng chú thích ảnh đã dịch** (FR129). Hình ảnh tham chiếu theo kiểu người dùng chọn ở FR130.
- FR89: Xuất theo **một Chương, nhiều Chương đã chọn, hoặc cả Tác phẩm**.
- FR90: **Nhập lại file `.docx` / `.md`** mà Reviewer đã chỉnh sửa, vào đúng Tác phẩm hiện có.
- FR91: **Segment alignment:** hệ thống khớp cấu trúc đoạn giữa file nhập và dữ liệu sẵn có. Segment **không khớp được phải hiện ra cho người dùng nối tay**.
- FR92: **Review Mode:** workspace chuyển sang bố cục **hai cửa sổ side-by-side** — trái là bản dịch của người dùng, phải là bản đã nhập từ Reviewer.
- FR93: Trong Review Mode, **ẩn văn bản gốc** và dùng thuật toán diff **bôi màu phần thêm / xoá / sửa** giữa hai bản dịch.
- FR94: Từ Review Mode, **chấp nhận từng thay đổi** vào bản dịch của mình, hoặc bỏ qua.
- FR95: Việc nhập bản review **kích hoạt cơ chế thu hoạch thuật ngữ (FR54) một cách độc lập** — kể cả khi người dùng **không bao giờ mở Review Mode**.
- FR121: Xuất **`.docx` một khối, đối xứng theo đoạn** — dành cho việc đăng bài. Bảng hai cột, **một hàng duy nhất cho cả Chương**, **không đường kẻ ngang**; **mỗi cột giữ cấu trúc đoạn của chính nó** *(AD-46)*. 🔵 **Sửa 2026-08-14** *(Sprint Change Proposal, Ice ký)*: vế cũ đọc *"hai ô giữ **đúng số lần xuống đoạn như nhau**"* và nó **hết đúng** từ khi FR134 cho bản dịch ngắt đoạn khác bản gốc. **Nghiệm thu không đổi một chữ** — nó chỉ đọc **cột phải**. Cái mất là **đối xứng thị giác** của file xuất, ghi đầy đủ ở AD-46. Nghiệm thu: bôi đen cột phải rồi dán sang trình soạn thảo website ra **văn bản liền mạch**, không mảnh vụn bảng biểu. **Không nhập lại được** — màn hình xuất phải nói rõ **ngay lúc chọn định dạng**. **Câu chưa xác nhận không được đánh dấu trong file xuất**; thay vào đó **cảnh báo trước lúc xuất**.
- FR130: **Chọn cách xuất hình ảnh: theo link gốc, hoặc theo file ảnh**, chọn cho từng lần xuất. Chỉ chọn được *theo link gốc* khi ảnh **có URL gốc lưu kèm**; khi phạm vi xuất chứa ảnh không có URL, **màn hình xuất phải liệt kê rõ ảnh nào sẽ không có link**, không được im lặng bỏ qua.
- FR131: **Xuất khối ghi nguồn** — năm trường: bốn trường xuất xứ của FR128 cộng **tên người dịch** (đặt một lần ở cấu hình toàn cục). Bật/tắt được, áp cho **mọi định dạng xuất** (FR87, FR88, FR121). **Mặc định tắt.**

## Done when

1. Xuất được `.docx` bảng hai cột theo segment và `.docx` một khối theo đoạn, `.md` và text thuần, kèm khối ghi nguồn; màn hình xuất nói rõ bản một khối không nhập lại được.
2. Nhập lại file reviewer đã sửa: cổng hình dạng chặn bản một khối trước mọi lệnh ghi, alignment hiện chỗ không khớp cho người dùng nối tay.
3. Review Mode hai cửa sổ ẩn văn bản gốc, bôi màu thêm/xoá/sửa, và chấp nhận được từng thay đổi.
4. Thu hoạch thuật ngữ nêu số lần đổi trên tổng số lần xuất hiện, và chạy độc lập khi Review Mode không được mở.
5. Mũi thăm dò thư viện diff và đọc `.docx` bảng hai cột cho kết luận kèm giấy phép GPLv3-tương thích. 🔵 2026-10-07: crate diff và giấy phép đã chốt ở AD-51 (`similar` =3.1.1); đọc `.docx` đã có `core::docx` từ Story 6.12. Còn lại đúng phép xác nhận của AD-51 mục 9 trên bản review thật tiếng Việt (Story 8.1).

## Boundaries

Trao đổi file với reviewer và Review Mode. Không phải gọi AI và không phải phát hành. 🔵 2026-10-07: thêm Story 8.17, nợ khớp TM từ retro Epic 7 — ngoài trục trao đổi file, Ice chọn đặt ở đây thay vì mở epic nợ riêng.

## References

- parent — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, functional requirements FR54, FR87, FR88, FR89, FR90, FR91, FR92, FR93, FR94, FR95, FR121, FR130, FR131
- spec — _bmad-output/initiative-auratranslate/spec-auratranslate/spec-auratranslate.md
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, các AD nêu trong tiêu chí của story

## Notes

- Decision: AD-38 (cổng hình dạng `.docx`) là cổng vào bắt buộc ở Rust, chạy trước alignment và trước mọi lệnh ghi (epics.md, ghi chú cài đặt).
- Decision (2026-10-07, Ice): khai `after` cho cả 15 entry — bản chuyển từ v6 để trống nên `next` coi cả epic là sẵn sàng. Story đầu tiên là 8.2.
- Decision (2026-10-07, Ice): 8.1 thành `hitl` + `refine` — AD-51 mục 9 co nó lại thành xác nhận hoặc lật `similar` trên bản review thật của Ice; AC giấy phép/Stack đã thoả.
- Decision (2026-10-07, Ice): 8.4 cần `refine` — AC "hai ô giữ đúng số lần xuống đoạn như nhau" đã hết đúng theo FR121 🔵 2026-08-14 và AD-46: mỗi cột giữ cấu trúc đoạn của chính nó.
- Decision (2026-10-07, Ice): mười mục nợ `Chủ: Epic 8` và hai mục giao nhầm (`Chủ: Story 8.1` · `Story 8.14`) chuyển chủ trong `deferred-work.md`, nhãn `incept Epic 8`. Thêm 8.16 (tách `commands/segment.rs`, trước 8.9) và 8.17 (nợ khớp TM, sau 8.16).
