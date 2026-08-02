# Trình tự xây dựng — AuraTranslate

> Companion của `SPEC.md`. **Đây là TRÌNH TỰ XÂY DỰNG, không phải cắt giảm phạm vi.** v1 gồm trọn mười nhóm năng lực. Thứ tự dưới đây nhằm gỡ rủi ro sớm và đạt trạng thái dùng được sớm nhất có thể.

| Giai đoạn | Nội dung | Năng lực | Trạng thái |
|---|---|---|---|
| **0** | Bốn mũi thăm dò: độ trễ Auto-Lookup, kích thước database, tokenizer lai, độ phủ kaikki.org | — | ✅ **Hoàn tất 2026-08-02** |
| **1** | Embedded Dictionary (kèm **lớp HVTĐTD**) + panel Source (kèm tab Hán Việt) + panel Lookup + Auto-Lookup | CAP-3, một phần CAP-2 | **Kế tiếp** |
| **2** | Panel Editor + AI Translation (BYOK/local) + Glossary + Smart RAG Injector | CAP-2, CAP-4, CAP-6 | |
| **3** | Library: mô hình dữ liệu, trạng thái vòng đời, tìm kiếm, chế độ đọc | CAP-1, CAP-9 | |
| **4** | Translation Memory + tái sử dụng segment + xuất TMX | CAP-5 | |
| **5** | Export/Import `.docx`/`.md` + segment alignment + Diff Viewer | CAP-8 | |
| **6** | AI Proofreader | CAP-7 | |
| **7** | Đóng gói, tài liệu cài đặt, attribution, phát hành | CAP-10 | |

> **Giai đoạn 1 là mốc giá trị sớm nhất** — nó làm được mọi thứ QuickTranslator làm, trên macOS, và là bằng chứng thuyết phục nhất để mời cộng đồng thử. **Nhưng nó không phải định nghĩa "xong"** — v1 chỉ hoàn thành khi cả bảy giai đoạn xong.

## Vì sao thứ tự này

AuraTranslate đặt cược rằng giá trị nằm ở **môi trường làm việc bao quanh AI**, không phải ở bản thân AI. Toàn bộ nhóm cạnh tranh 2026 đặt AI ở giữa và bỏ qua môi trường. Trình tự này phản ánh đúng luận điểm đó: **xây môi trường trước, cắm AI vào sau.**

## Nếu buộc phải cắt

Phạm vi v1 gồm toàn bộ là quyết định có ý thức của chủ dự án, đã tái khẳng định. Nếu hoàn cảnh buộc phải cắt, thứ tự gợi ý là: **AI Proofreader (CAP-7) → Diff Viewer (phần FR92–FR94 của CAP-8) → Translation Memory (CAP-5)**.

Lưu ý: cắt Diff Viewer **không** được kéo theo FR95 — thu hoạch thuật ngữ từ bản review chạy độc lập và là đường bảo hiểm cho rủi ro vòng phản hồi đứt (R3).

## Điểm mở lại các quyết định đã hoãn

Chi tiết đầy đủ ở `ARCHITECTURE-SPINE.md` mục *Deferred*. Neo theo giai đoạn:

| Giai đoạn | Mở lại quyết định gì |
|---|---|
| **2** | Thư viện editor cho panel Editor · cách phân tích khung SSE (rà giấy phép trước) · ngưỡng kích thước WAL buộc checkpoint · nhịp auto-save cụ thể đạt NFR18 mà không phạm NFR2 |
| **3** | Ngưỡng NFR3/NFR4/NFR5 (`[A6] [A7] [A8]`, đóng Q4) · chiến lược ảo hoá danh sách dài · cấu trúc chi tiết chỉ mục FTS cho tìm kiếm Library |
| **5** | `similar` vs `dissimilar` cho Diff Viewer · thuật toán segment alignment |

~~HVTĐTD (Q3)~~ — ✅ **đã đóng 2026-08-02**, tác giả đồng ý. Lớp này vào Giai đoạn 1 dưới dạng một file `.db` gỡ rời; **không đổi kiến trúc**, đúng như dự liệu.

## Áp từ Giai đoạn 1, không được để lại sau

Hai yêu cầu cắt ngang mọi giai đoạn, chung một lý do: **rẻ nếu làm từ dòng code đầu tiên, rất đắt nếu vá sau.**

- **NFR16** — toàn bộ chuỗi giao diện nằm ngoài mã nguồn, trong file tài nguyên riêng.
- **NFR17** — sàn khả năng tiếp cận: thao tác hoàn toàn bằng bàn phím, focus nhìn thấy rõ, tương phản WCAG AA ở cả hai theme.
