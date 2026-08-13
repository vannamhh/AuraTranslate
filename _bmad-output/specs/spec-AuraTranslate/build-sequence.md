# Trình tự xây dựng — AuraTranslate

> Companion của `SPEC.md`. **Đây là TRÌNH TỰ XÂY DỰNG, không phải cắt giảm phạm vi.** v1 gồm trọn mười nhóm năng lực. Thứ tự dưới đây nhằm gỡ rủi ro sớm và đạt trạng thái dùng được sớm nhất có thể.

> 🔵 **Cập nhật 2026-08-13 — số hiệu Giai đoạn là TÊN, không phải thứ tự.** Cột *Thứ tự* là
> nguồn sự thật cho trình tự thực thi. Lý do tách hai thứ đó ra: ~40 tham chiếu *"Giai đoạn N"*
> rải trên 6 tệp quy hoạch đang dùng số hiệu làm **mỏ neo cho các quyết định hoãn**; đánh số
> lại thì mỗi chỗ sót là một mỏ neo trỏ sai âm thầm, và không cổng nào đỏ vì chuyện đó.

| Giai đoạn | Nội dung | Năng lực | Thứ tự | Trạng thái |
|---|---|---|---|---|
| **0** | Bốn mũi thăm dò: độ trễ Auto-Lookup, kích thước database, tokenizer lai, độ phủ kaikki.org | — | — | ✅ **Hoàn tất 2026-08-02** |
| **1** | Embedded Dictionary (kèm **lớp HVTĐTD**) + panel Source (kèm tab Hán Việt) + panel Lookup + Auto-Lookup | CAP-3, một phần CAP-2 | **1** | **Đang chạy** |
| **2** | Panel Editor + Glossary | CAP-2, CAP-6 | **2** | |
| **2c** | AI Translation (BYOK/local) + Smart RAG Injector | CAP-4 | **6** ← dời | |
| **3** | Library: mô hình dữ liệu, trạng thái vòng đời, tìm kiếm, chế độ đọc, **và toàn bộ đường nhập** — file, dán tay, URL, song ngữ, kèm pipeline làm sạch/chuẩn hoá/bảng mã | CAP-1, CAP-9 | **3** | |
| **4** | Translation Memory + tái sử dụng segment + xuất TMX | CAP-5 | **4** | |
| **5** | Export/Import `.docx`/`.md` + segment alignment + Diff Viewer | CAP-8 | **5** | |
| **6** | AI Proofreader | CAP-7 | **7** | |
| **7** | Đóng gói, tài liệu cài đặt, attribution, phát hành | CAP-10 | **8** | |

**Thứ tự thực thi đầy đủ (chốt 2026-08-13):**

`1 → 2a → 2b → 3a → 3b → 2c → 4 → 5 → 6 → 7`

tức theo epic: **1 → 2 → 3 → 5 → 6 → 4 → 7 → 8 → 9 → 10**. Quyết định của chủ dự án, động cơ
ghi nguyên văn: *muốn thấy sản phẩm dùng được sớm*. Đây là thay đổi **trình tự**, không phải
cắt phạm vi — v1 vẫn gồm trọn mười nhóm năng lực. Phân tích đầy đủ kèm kiểm phụ thuộc chéo:
`sprint-change-proposal-2026-08-13b-thu-tu-epic.md`.

**Ngoại lệ:** Story 4.1 (module `ai/` cô lập + test cưỡng chế AD-13) tách khỏi Giai đoạn 2c và
chạy **ngay sau Giai đoạn 2b**. Lý do: ranh giới AD-13 thuộc đúng loại *rẻ nếu làm từ dòng code
đầu tiên, rất đắt nếu vá sau* (§"Áp từ Giai đoạn 1" dưới đây nói cùng một điều cho NFR16/NFR17)
— để nó lùi cùng 2c nghĩa là 32 story được viết trước khi ranh giới `ai/` có người canh, và một
vi phạm AD-13 giết FR77 mà **chỉ lộ ra khi một người dùng không có API key thử**.

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
| **2** | Thư viện editor cho panel Editor · ngưỡng kích thước WAL buộc checkpoint · nhịp auto-save cụ thể đạt NFR18 mà không phạm NFR2 |
| **2c** | 🔵 **2026-08-13:** cách phân tích khung SSE (rà giấy phép trước) — **dời từ Giai đoạn 2 sang đây** cùng lượt dời CAP-4. SSE chỉ phục vụ streaming AI (AD-22), nên để mỏ neo ở Giai đoạn 2 sau khi nội dung đã dời là dựng một cái hẹn không ai đến — và nó sẽ bị đọc thành *"phải rà giấy phép SSE trước khi làm Editor"*, sai cả hai vế. `reqwest-sse` và `sseer` vẫn **chưa xác nhận giấy phép**; cửa rà NFR15 **không đổi**, chỉ mở muộn hơn |
| **3** | Ngưỡng NFR3/NFR4/NFR5 (`[A6] [A7] [A8]`, đóng Q4) · chiến lược ảo hoá danh sách dài · cấu trúc chi tiết chỉ mục FTS cho tìm kiếm Library · **thư viện bóc nội dung (`[A12]`) và thư viện phát hiện bảng mã (`[A13]`)** · **HTTP client cho đường nhập URL** |
| **5** | `similar` vs `dissimilar` cho Diff Viewer · thuật toán segment alignment |

~~HVTĐTD (Q3)~~ — ✅ **đã đóng 2026-08-02**, tác giả đồng ý. Lớp này vào Giai đoạn 1 dưới dạng một file `.db` gỡ rời; **không đổi kiến trúc**, đúng như dự liệu.

## Áp từ Giai đoạn 1, không được để lại sau

Hai yêu cầu cắt ngang mọi giai đoạn, chung một lý do: **rẻ nếu làm từ dòng code đầu tiên, rất đắt nếu vá sau.**

- **NFR16** — toàn bộ chuỗi giao diện nằm ngoài mã nguồn, trong file tài nguyên riêng.
- **NFR17** — sàn khả năng tiếp cận: thao tác hoàn toàn bằng bàn phím, focus nhìn thấy rõ, tương phản WCAG AA ở cả hai theme.
