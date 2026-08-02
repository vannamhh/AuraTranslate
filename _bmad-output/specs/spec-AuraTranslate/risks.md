# Rủi ro & phụ thuộc ngoài — AuraTranslate

> Companion của `SPEC.md`. Mỗi rủi ro ghi kèm biện pháp đã cài vào hợp đồng — phần lớn đã trở thành FR hoặc bất biến kiến trúc, nên downstream **không cần phát minh lại biện pháp**.

## Rủi ro

| # | Rủi ro | Mức | Biện pháp đã cài vào hợp đồng |
|---|---|---|---|
| **R1** | **Phạm vi v1 gồm toàn bộ, một người làm.** Rủi ro không nằm ở kỹ thuật mà ở việc dự án không bao giờ đạt tới trạng thái dùng được | 🔴 | Quyết định có ý thức, đã tái khẳng định. Giảm nhẹ bằng **trình tự** (`build-sequence.md`), không bằng cắt phạm vi. Thứ tự cắt gợi ý nếu buộc phải cắt cũng ở đó |
| **R2** | **Chất lượng và xuất xứ dữ liệu từ điển.** Mỗi nguồn có khiếm khuyết riêng đã biết | 🔴 → 🟢 | **Đã chuyển thành tính năng:** FR31/FR32 bắt buộc hiển thị nguồn và hiển thị bất đồng. Đây là chỗ một rủi ro trở thành điểm khác biệt |
| **R3** | **Chưa rõ vì sao vòng phản hồi bị đứt** — không thể khẳng định Diff Viewer sẽ được dùng thật | 🔴 | **FR95** là đường bảo hiểm: thu hoạch thuật ngữ chạy độc lập với Diff Viewer. Nguyên nhân gốc là Q1, để ngỏ có chủ ý |
| **R7** | **Xuất xứ Thiều Chửu và Cổ hán văn không được xác minh** — quyết định chủ động không kiểm tra trước khi phát hành | 🔴 | **Không còn biện pháp phòng ngừa.** Chỉ còn biện pháp phản ứng: cả hai đóng gói làm **lớp gỡ rời** (FR36) + chính sách gỡ bỏ (FR112). Chi tiết: `data-sources.md` |
| **R4** | **Glossary khởi động từ con số không** — ép AI tuân thủ Glossary chỉ phát huy khi Glossary đã đầy | 🟡 | FR52–FR54: ba cơ chế đề xuất tự động, tất cả đều qua duyệt (FR55) |
| **R5** | **Phát hành không ký số** — rào cản đón nhận với người dùng phổ thông | 🟡 | FR106 checksum, FR107 build công khai, FR108 hướng dẫn có ảnh; ngoài sản phẩm: video hướng dẫn cài đặt và truyền miệng trong cộng đồng — đúng cách QuickTranslator từng lan toả; uy tín SmartScreen tích luỹ dần theo lượt tải. Không thể xoá bỏ bằng kỹ thuật, chỉ giảm nhẹ bằng minh bạch. **[A5]** |
| **R6** | **VietPhrase không rõ xuất xứ** | 🟡 | Lớp gỡ rời (FR36) + chính sách gỡ (FR112) + ghi công rõ (FR38) |
| **R12** | **HVTĐTD dùng theo phép, không theo giấy phép mở** — phép có thể được rút lại, và phạm vi phân phối lại **không được xác nhận trước khi đóng gói** (quyết định 2026-08-02, đóng Q8) | 🟡 | **Thuần phản ứng, đúng như đã chọn:** giữ nguyên hình dạng **lớp gỡ rời** dù đã được đồng ý (FR36) + chính sách gỡ (FR112) — gỡ lớp = xoá một file, không đổi mã. Lớp nền C vẫn bắt buộc nên gỡ HVTĐTD không làm hỏng chức năng nào. Giảm nhẹ đặc thù: tác giả đang giữ liên lạc, nên yêu cầu (nếu có) sẽ đến trực tiếp |
| **R8** | **Segment alignment khi nhập bản review sai lệch** | 🟡 | Bài toán đã giải trong ngành CAT tool; FR91 áp mẫu chuẩn *máy khớp, người sửa* |
| **R9** | **Gai trễ auto-save làm gián đoạn khi gõ** | 🟡 | NFR2 là ngưỡng nghiệm thu tường minh; giải pháp kỹ thuật ở `ARCHITECTURE-SPINE.md` AD-11, AD-12. Hợp đồng flush đạt NFR18 mà không phạm NFR2 nằm ở **AD-35** |
| **R10** | **Không có lemmatization thật trong hệ sinh thái Rust** | 🟢 | Stemming đủ cho khớp Glossary; giới hạn ghi rõ trong FR40 thay vì giấu đi |
| **R11** | **Tra từ tiếng Trung 1–2 ký tự trả về rỗng mà không báo lỗi** | 🟢 | Đã phát hiện ở Giai đoạn 0 và viết thành FR39 làm điều kiện nghiệm thu |

## Ràng buộc nối tiếp — đừng vô tình gỡ một mắt xích

Không quyết định nào dưới đây là độc lập:

```
Chọn GPL          →  phải là GPLv3 để dùng crate Apache-2.0  (NFR15)
Không kinh phí    →  không ký số
                  →  niềm tin phải đến từ nơi khác
                  →  build công khai + checksum SHA-256      (FR106, FR107)
                  →  và cấm cơ chế tự cập nhật               (FR111)
```

## Phụ thuộc bên ngoài

| Phụ thuộc | Loại | Ghi chú |
|---|---|---|
| ~~Hồi âm của tác giả HVTĐTD~~ | — | ✅ **Đã đóng 2026-08-02** — tác giả đồng ý bằng văn bản. Còn lại: xác nhận phạm vi phân phối lại (Q8) và nghĩa vụ thông báo khi hoàn thành. Chi tiết: `data-sources.md` |
| Nhà cung cấp AI (BYOK) | Dịch vụ bên thứ ba | Người dùng tự chọn và tự trả. **Endpoint tương thích OpenAI là hợp đồng tích hợp duy nhất** |
| Ollama / LM Studio | Phần mềm bên thứ ba | Dùng chung đường cấu hình với BYOK, không phải tích hợp riêng |
| GitHub Releases + Actions | Hạ tầng phát hành | Là nền của FR105–FR107 |

## Việc chưa đo được ở Giai đoạn 0

| Hạng mục | Vì sao chưa đo | Rủi ro còn lại |
|---|---|---|
| **Vòng IPC Tauri thật** + thời gian render frontend | Cần app có cửa sổ, không đo được ở môi trường dòng lệnh | Đã giảm mạnh nhờ payload 679 byte. Nếu Auto-Lookup chậm, nguyên nhân sẽ ở frontend **[A1]** |
| **Unihan, Thiều Chửu, Cổ hán văn, VietPhrase** trong database | Chưa nạp thử | Ước tính tổng 150–200 MB dựa trên ba nguồn đã có **[A2]** |
| **`jieba-rs` và `tantivy-stemmers`** chạy thật | Mới xác minh giấy phép và độ trưởng thành | Chưa có số đo chất lượng tách từ / stemming |
| **Tìm kiếm Library trên thư viện thật** | Chưa có Library | Ngưỡng NFR3, NFR4, NFR5 còn là giả định **[A6] [A7] [A8]** — đóng ở Q4 |
