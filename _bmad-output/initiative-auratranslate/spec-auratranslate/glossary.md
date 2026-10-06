# Glossary thuật ngữ — AuraTranslate

> Companion của `SPEC.md`. **Mọi tài liệu và mã nguồn downstream phải dùng đúng các tên dưới đây.** Cột "Không dùng" là các biến thể đã xuất hiện trong tài liệu đầu vào và bị loại khỏi từ vựng chính thức.

## Từ vựng sản phẩm

| Thuật ngữ | Nghĩa | Không dùng |
|---|---|---|
| **Tác phẩm** | Đơn vị cấp cao nhất trong Library. Một Tác phẩm = một `.atproj` = một phạm vi Glossary/TM riêng. Tài liệu ngắn là Tác phẩm có một Chương | ~~dự án~~, ~~Project~~, ~~tài liệu~~ |
| **Chương** | Đơn vị dịch bên trong một Tác phẩm, có văn bản nguồn và văn bản đích riêng | ~~document~~, ~~file~~ |
| **Segment** | Đơn vị dịch nhỏ nhất — **một câu**. Đơn vị của Translation Memory và của luồng xác nhận | ~~đoạn~~, ~~câu~~ *(khi nói về cấu trúc dữ liệu)* |
| **Library** | Khung/chế độ thứ nhất: kho Tác phẩm, điểm vào ứng dụng | ~~Thư viện~~ |
| **Workspace** | Khung/chế độ thứ hai: môi trường dịch bốn panel | ~~màn hình dịch~~ |
| **Chế độ đọc** *(Reading Mode)* | Chế độ đọc lại bản dịch đã hoàn thành, không có công cụ biên tập | — |
| **Review Mode** | Bố cục hai cửa sổ side-by-side để đối chiếu bản dịch của mình với bản Reviewer đã sửa | ~~Diff Mode~~ |
| **Panel Lookup** | Panel số 2 của Workspace — nơi kết quả tra cứu từ điển và concordance hiện ra | ~~panel tra cứu~~ |
| **Auto-Lookup** | Cơ chế đưa kết quả tra cứu ra Panel Lookup ngay khi bôi đen, không cần copy/paste | — |
| **Glossary** | Bộ thuật ngữ do người dùng chốt, hai tầng Global và Tác phẩm. Cung cấp dữ liệu cho Smart RAG Injector | ~~từ điển cá nhân~~, ~~name database~~ |
| **Translation Memory** *(TM)* | Kho cặp *(segment nguồn → segment đích)* tích luỹ tự động khi người dùng xác nhận segment | ~~bộ nhớ dịch~~ |
| **Concordance** | Tra ngược trong TM: *"cụm từ này trước đây tôi dịch thế nào?"* | — |
| **Smart RAG Injector** | Cơ chế chèn động Glossary + segment TM tương tự vào prompt trước mỗi lần gọi AI | — |
| **Scope** | Phân cấp cấu hình hai tầng: **Global** (toàn ứng dụng) và **Tác phẩm** (ghi đè Global) | ~~Project Scope~~ |
| **`.atproj`** | File/thư mục dự án trên đĩa. **Nguồn sự thật** của một Tác phẩm, tự chứa mọi dữ liệu của nó | — |
| **Chỉ mục Library** | Cơ sở dữ liệu tập trung phục vụ tìm kiếm xuyên Tác phẩm. **Dẫn xuất, không phải nguồn sự thật** | ~~database~~ |
| **Lớp nền** | Nguồn từ điển có giấy phép sạch, bắt buộc phải có | — |
| **Lớp gỡ rời** | Nguồn từ điển có rủi ro pháp lý, đóng gói tách rời; gỡ đi không làm hỏng tra cứu | ~~lớp tuỳ chọn~~ |
| **Hán Việt** | Âm đọc Việt của ký tự Hán. Nội dung của tab Hán Việt trong Panel Source | — |
| **Segment alignment** | Bài toán khớp cấu trúc đoạn giữa file nhập từ Reviewer và dữ liệu sẵn có | ~~Structural Index Mapping~~ |
| **BYOK** | *Bring Your Own Key* — người dùng dùng API key của chính mình | — |

## Ánh xạ sang định danh trong mã

Cố định bởi `ARCHITECTURE-SPINE.md` (Consistency Conventions):

Tác phẩm → `Work` · Chương → `Chapter` · Segment → `Segment` · Chế độ đọc → `ReadingMode` · Review Mode → `ReviewMode` · Panel Lookup → `LookupPanel` · Smart RAG Injector → `RagInjector` · Chỉ mục Library → `LibraryIndex` · Lớp nền / lớp gỡ rời → `BaseLayer` / `DetachableLayer` · Hán Việt → `HanViet` · Segment alignment → `Alignment`.

**Cấm dùng `Project`, `Book`, `Novel`, `Document` cho `Work`.** Đuôi file `.atproj` là ngoại lệ lịch sử, không kéo theo tên thực thể.
