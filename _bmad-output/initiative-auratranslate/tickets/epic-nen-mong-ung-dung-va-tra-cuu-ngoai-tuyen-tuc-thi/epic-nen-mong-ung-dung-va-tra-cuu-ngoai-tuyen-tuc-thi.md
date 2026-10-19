---
type: epic
title: "Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì"
parent: initiative-auratranslate
covers: ["FR13", "FR16", "FR17", "FR18", "FR19", "FR21", "FR22", "FR27", "FR28", "FR29", "FR30", "FR31", "FR32", "FR33", "FR34", "FR35", "FR36", "FR37", "FR38", "FR39", "FR40", "FR41", "FR96", "FR97", "FR102", "FR103", "FR104", "FR135"]
after: []
assignee: ""
status: in-progress
risk: medium
---

# Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì

## Description

Người dịch mở AuraTranslate, đưa một văn bản tiếng Trung hoặc tiếng Anh vào, bôi đen một cụm từ ở Panel Source và **thấy ngay định nghĩa có ghi nguồn** ở Panel Lookup — dưới 100 ms, hoàn toàn ngoại tuyến, với các nguồn bất đồng hiển thị cạnh nhau chứ không bị hợp nhất. Bật/tắt từng nguồn từ điển được, gỡ một lớp gỡ rời khỏi bản cài không làm hỏng bất kỳ đường tra cứu nào. Đây là **mốc giá trị sớm nhất**: làm được mọi thứ QuickTranslator làm, trên macOS lẫn Windows.

## Outcome

Người dịch bôi đen một cụm từ ở panel nguồn và thấy ngay định nghĩa có ghi nguồn ở Lookup, dưới 100 ms và hoàn toàn ngoại tuyến, trên macOS lẫn Windows; tín hiệu là NFR1 (p95 < 100 ms) và bộ test tra cứu xanh khi xoá một lớp gỡ rời.

## Requirements

Nguồn: PRD (prd-auratranslate/prd-auratranslate.md); epics.md §Epic 1 (`FRs covered`). Mỗi dòng giữ mã FR của PRD.

- FR13: Tạo Tác phẩm mới từ file (`.txt`, `.docx`, `.md`) hoặc từ văn bản dán trực tiếp.
- FR16: Ba panel trong một cửa sổ ứng dụng duy nhất: Lưới đối chiếu, Lookup, AI Translation. Đây là câu trả lời trực tiếp cho nỗi đau "bốn đến năm cửa sổ mở cùng lúc". 🔵 (Sửa 2026-08-14: bản cũ khai bốn, tách Source và Editor. Lưới gộp hai cái đó …
- FR17: Panel hỗ trợ kéo thả để dock/undock, gộp thành tab, và thay đổi kích thước. Mỗi panel ẩn được hoàn toàn — người dịch không dùng AI phải giấu được panel AI Translation.
- FR18: Bố cục workspace được lưu và khôi phục giữa các phiên làm việc. Hỗ trợ lưu nhiều preset bố cục và chuyển nhanh giữa chúng.
- FR19: Cột nguyên văn của lưới hiển thị văn bản gốc (Anh hoặc Trung) kèm Hán Việt cho tài liệu tiếng Trung — xem ở chế độ chuyển đổi hoặc song song, người dùng tự bật tắt. 🔵 (Sửa 2026-08-14: chữ "tab" rút — Hán Việt sống bên trong ô nguyên văn. …
- FR21: Auto-Lookup: bôi đen một cụm từ ở cột nguyên văn của lưới — chữ gốc hoặc âm Hán Việt — → kết quả tra cứu hiện ngay ở panel Lookup. Không copy, không paste, không chuyển cửa sổ. 🔵 (Sửa 2026-08-14: "Panel Source" → "cột nguyên văn của lưới". …
- FR22: Global Hotkeys cho các thao tác lặp lại: dịch segment hiện tại, chuyển focus giữa các panel, xác nhận segment, tra cứu cụm đang chọn, bật/tắt sync scroll. Toàn bộ phím tắt cấu hình lại được.
- FR27: Toàn bộ dữ liệu từ điển nhúng trong bản cài. Tra cứu hoạt động 100% offline, không có cơ chế tải thêm sau khi cài đặt.
- FR28: Panel Lookup hiển thị một bản ghi có cấu trúc, không phải một đoạn văn bản. Mỗi mục gồm: nguồn · từ loại · nghĩa · ví dụ[] · trích dẫn[] · ghi chú.
- FR29: Một từ có nhiều từ loại phải hiện thành nhiều mục riêng biệt, mỗi mục có ví dụ riêng. Ví dụ: cùng một chữ dùng làm động từ và làm phó từ là hai mục, không phải một chuỗi nghĩa gộp.
- FR30: Ví dụ gắn với từng từ loại, không gắn với cả từ. Trích dẫn là trường riêng biệt với ví dụ: trích dẫn có xuất xứ văn bản.
- FR31: Mọi định nghĩa phải hiển thị nguồn của nó. Không có ngoại lệ, không có chế độ ẩn nguồn.
- FR32: Khi các nguồn bất đồng về một mục từ, hệ thống hiển thị đồng thời cả hai, không hợp nhất thành một câu trả lời duy nhất.
- FR33: Tab Hán Việt: hiển thị âm Hán Việt cho từng ký tự tiếng Trung trong văn bản nguồn.
- FR34: Mục từ tiếng Anh phải có nhãn từ loại và nghĩa tiếng Việt.
- FR35: Mục từ tiếng Trung phải có nhãn từ loại và ít nhất một ví dụ cách dùng khi nguồn có dữ liệu. Ở v1, nhãn từ loại và bản dịch ví dụ bằng tiếng Anh được chấp nhận và phải được đánh dấu rõ là nhãn ngoại ngữ.
- FR36: Các nguồn từ điển được đóng gói theo mô hình "nền có giấy phép sạch + lớp gỡ rời được". Gỡ bỏ bất kỳ lớp gỡ rời nào không được làm hỏng chức năng tra cứu — sản phẩm vẫn hoạt động đầy đủ trên các lớp nền.
- FR37: Người dùng bật/tắt từng nguồn từ điển trong panel Lookup.
- FR38: Ghi công đầy đủ từng nguồn từ điển: trong ứng dụng (màn hình Attribution) và trong bản phát hành.
- FR39: Tra cứu tiếng Trung phải trả về kết quả cho truy vấn 1 ký tự, 2 ký tự và 3 ký tự trở lên.
- FR40: Tra cứu tiếng Anh nhận diện biến thể hình thái của từ. Giới hạn đã biết và chấp nhận: đây là stemming, không phải lemmatization thật — hệ sinh thái Rust chưa có lemmatizer trưởng thành. Đủ cho khớp Glossary, không xử lý được các biến thể …
- FR41: Lịch sử tra cứu trong phiên làm việc, và ghim mục từ để tra lại nhanh. `[A9]`
- FR96: Mỗi Tác phẩm được lưu thành một `.atproj` trên đĩa. Đây là nguồn sự thật; người dùng copy, sao lưu và di chuyển tự do.
- FR97: `.atproj` tự chứa mọi thứ thuộc về Tác phẩm: văn bản nguồn, bản dịch, segment, lịch sử phiên bản, Glossary dự án, TM dự án, prompt dự án và hình ảnh. Copy sang máy khác phải mở được nguyên vẹn.
- FR102: Sao lưu bằng cách copy thư mục là đủ. Không được yêu cầu một thao tác export riêng để có bản sao lưu dùng được.
- FR103: Mọi cấu hình tồn tại ở hai tầng, tầng dự án ghi đè tầng toàn cục:
- FR104: Không telemetry. Ứng dụng không gửi bất kỳ dữ liệu nào ra ngoài, trừ nội dung mà người dùng chủ động gửi cho nhà cung cấp AI đã cấu hình.
- FR135: Double-click ở tab Hán Việt chọn trọn CỤM TỪ, không một âm tiết. Ở tab nguyên văn tiếng Trung, double-click đã chọn được cả cụm từ; tab Hán Việt phải giữ đúng hành vi ấy — cùng một văn bản, cùng một thao tác, cùng một đơn vị chọn. Đơn vị ở …

## Done when

1. Người dịch bôi đen một cụm từ tiếng Trung hoặc tiếng Anh ở panel nguồn và thấy định nghĩa có ghi nguồn ở Lookup, hoàn toàn ngoại tuyến, p95 đầu-cuối dưới 100 ms (FR21, NFR1), các nguồn bất đồng hiển thị cạnh nhau (FR29–FR32).
2. Xoá một file `.db` lớp gỡ rời rồi chạy lại bộ test tra cứu vẫn xanh (FR36); bật tắt từng nguồn và ghi công đầy đủ (FR37, FR38).
3. Ba chế độ, bốn panel và `CommandRegistry` chạy với phím tắt cấu hình lại được (FR16–FR18, FR22); lịch sử tra cứu và mục ghim hoạt động (FR41).
4. Đưa văn bản vào bằng dán tay hoặc `.txt`/`.md` tạo ra một Tác phẩm trên đĩa copy đi được (FR13, FR96, FR97, FR102); cấu hình hai tầng phân giải nhất quán (FR103).
5. CI chạy build và test trên macOS lẫn Windows mỗi lần push, và bộ chạy e2e lái webview thật chạy hết một lượt (Story 1.3, 1.22).
6. Các story còn `in-progress` (1.3, 1.20, 1.21) đã đóng và đợt kiểm tay của Ice trên ứng dụng thật đã chạy xong.

## Boundaries

Nền móng và đường tra cứu ngoại tuyến. Không gồm nhập hàng loạt và nhánh `.docx` (epic-duong-nhap), Glossary (epic-glossary), AI (epic-ai-mo-va-smart-rag-injector).

## References

- parent — ../initiative-auratranslate.md, mục Requirements (FR/NFR của PRD)
- prd — ../prd-auratranslate/prd-auratranslate.md
- spec — ../spec-auratranslate/spec-auratranslate.md
- constraint — ../architecture-auratranslate/architecture-auratranslate.md (AD-xx của từng story nằm trong plan của nó)
- ux — ../ux-auratranslate/ux-auratranslate.md

## Notes

- Retrospective: epic-nen-mong-ung-dung-va-tra-cuu-ngoai-tuyen-tuc-thi-retrospective.md
- Source conflict: FR13 chia đôi nghiệm thu; Epic này chỉ mở nhánh dán tay và `.txt`/`.md`, nhánh `.docx` đóng ở epic-duong-nhap (epics.md, ghi chú cài đặt Epic 1).
- Decision: Story 1.22 không có bản ghi build v6; plan của nó là bản tối thiểu ghi rõ thiếu bản ghi (di trú v6 sang v7).
