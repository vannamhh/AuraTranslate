---
title: "PRD: AuraTranslate"
status: final
created: 2026-08-02
updated: 2026-08-03
---

# PRD: AuraTranslate

> **Quy ước đánh số:** mỗi yêu cầu chức năng mang một ID toàn cục `FRn` **không bao giờ được đánh số lại**. Yêu cầu bổ sung về sau nhận số mới ở cuối dãy, kể cả khi nằm giữa tài liệu. Yêu cầu phi chức năng dùng dãy riêng `NFRn`.
>
> **Nguồn:** tài liệu này chưng cất từ `brief.md`, `addendum.md` (chứa nguyên văn PRD v8.0), báo cáo nghiên cứu kỹ thuật và **kết quả đo thật của Giai đoạn 0**. Chi tiết kỹ thuật và các phương án đã loại nằm ở `addendum.md` cùng thư mục.

---

## 1. Bối cảnh & Tầm nhìn

**AuraTranslate là translation workstation chạy local-first, dành cho người dịch Anh/Trung → Việt coi trọng chất lượng hơn tốc độ.**

QuickTranslator — công cụ mà cộng đồng dịch giả Việt Nam dùng suốt hơn một thập kỷ — chỉ chạy trên Windows và dừng phát triển từ 2022. Người dịch trên macOS không có gì thay thế. Cùng lúc, làn sóng công cụ dịch AI 2026 chạy theo hướng ngược lại: cloud-based, tự động hoá tối đa, phục vụ **độc giả muốn đọc nhanh** thay vì **người dịch muốn dịch hay**.

> **Câu định vị:** các công cụ khác giúp bạn dịch xong; **AuraTranslate là nơi bản dịch của bạn sống.**

### 1.1 Bốn trụ định vị

| # | Trụ | Hệ quả bắt buộc lên thiết kế |
|---|---|---|
| **1** | **Người dịch là trung tâm, AI là công cụ** | AI đề xuất, người biên tập quyết định, hệ thống ghi nhớ quyết định đó để lần sau đề xuất sát hơn. Không có luồng nào dịch xong mà không qua tay người |
| **2** | **Bản dịch có nơi để sống** | Library là điểm vào ứng dụng, không phải màn hình phụ. Không đối thủ nào trong nhóm cạnh tranh có tầng này |
| **3** | **Local-first thật sự** | Không tài khoản, không cloud sync, không telemetry. Dữ liệu phải sống lâu hơn phần mềm |
| **4** | **Open source, GPL v3** | Lựa chọn **chủ động** — bộ dữ liệu từ điển đã chọn không buộc dự án phải theo giấy phép nào. Xem §8 |

### 1.2 Bốn nỗi đau đang giải

1. **Công cụ đã chết nhưng chưa có người kế nhiệm** — QuickTranslator dừng ở 2022, chỉ Windows, không biết gì về AI.
2. **Công sức không đọng lại ở đâu cả** — mỗi tài liệu một cửa sổ, bốn năm cửa sổ mở cùng lúc; dịch xong đẩy lên Google Docs rồi bản dịch rời khỏi công cụ.
3. **Tra cứu vẫn là lao động thủ công** — đối chiếu từ điển bằng tay, copy/paste qua lại; chương sau lại tra lại từ đầu.
4. **Vòng phản hồi bị đứt** — reviewer bỏ công sửa nhưng người dịch ít khi xem lại; cùng những lỗi ấy lặp lại ở chương tiếp theo.

*Mốc tham chiếu hiện tại: một chương mất nửa buổi đến trọn một ngày.*

### 1.3 Luận điểm sản phẩm

**AuraTranslate đặt cược rằng giá trị nằm ở môi trường làm việc bao quanh AI, không phải ở bản thân AI.**

Đây không phải trực giác riêng của dự án. Nhận định chung của ngành dịch thuật là: *LLM cho ra bản dịch trôi chảy nhưng **thiếu tính nhất quán, thiếu cưỡng chế glossary và thiếu theo dõi thay đổi** — nên hoạt động tốt nhất khi nằm **bên trong** một môi trường CAT, chứ không đứng một mình.*

Toàn bộ nhóm cạnh tranh 2026 đi ngược nhận định đó: họ đặt AI ở giữa và bỏ qua môi trường. AuraTranslate xây môi trường trước, cắm AI vào sau — thứ tự xây dựng ở §10 phản ánh đúng điều này.

### 1.4 Lợi thế thật — nói thẳng

**Không có rào cản kỹ thuật nào ở đây.** Rust, Tauri, SQLite là công cụ phổ thông; bất kỳ ai cũng dựng lại được. Nghiên cứu kỹ thuật xác nhận: không có thành phần nào của sản phẩm này là khó.

Lợi thế thật nằm ở chỗ khác: **sản phẩm được xây bởi một người dịch thật, cho công việc thật của chính mình, trong một ngách mà các sản phẩm quốc tế không nhìn tới** — cặp Anh/Trung → Việt, với Hán Việt là thành phần bắt buộc chứ không phải tuỳ chọn.

> **Ràng buộc thiết kế kèm theo:** cộng đồng đã quen với mô hình tra cứu của QuickTranslator. Đây **vừa là lợi thế** (không phải dạy lại) **vừa là ràng buộc** — lệch quá xa khỏi mô hình đó sẽ bị từ chối. Ràng buộc này áp lên C2 (Workspace) và C3 (Lookup) mạnh hơn mọi nhóm năng lực khác.

### 1.5 Tầm nhìn dài hạn

Trong hai đến ba năm, AuraTranslate trở thành thứ mà QuickTranslator từng là với thế hệ trước — **công cụ mặc định của người dịch Việt Nam** — nhưng cho thời đại AI, và không bỏ rơi ai vì hệ điều hành họ dùng.

Điều này giải thích **vì sao C1 và C5 đáng công** dù chúng không phải thứ người dùng đòi hỏi ngay:

- **Library** lớn dần thành kho lưu trữ cá nhân: mọi thứ từng dịch, tìm được, đọc được, dùng lại được.
- **Translation Memory và Glossary** tích luỹ đủ dày để AI viết ra thứ ngày càng giống **chính người dùng**, thay vì giống một cỗ máy.

Là dự án open source, giá trị bền vững nhất có thể không phải phần mềm mà là những gì cộng đồng bồi đắp quanh nó: bộ từ điển, bộ prompt theo thể loại, và các quy chuẩn dịch thuật được chia sẻ giữa những người cùng nghề.

---

## 2. Người dùng

| Vai | Mô tả | Ràng buộc |
|---|---|---|
| **Người dịch nghiêm túc** — *primary* | Dịch Anh/Trung → Việt trên mọi lĩnh vực: truyện, tài liệu kỹ thuật, báo chí, hợp đồng. Sẵn sàng dành nửa ngày cho một chương vì chất lượng quan trọng hơn sản lượng | Là người dùng duy nhất bắt buộc cài app |
| **Reviewer** — *secondary* | Đọc và sửa bản dịch | **Không bắt buộc cài app.** Nhiều người sẽ không rời Google Docs, và điều đó được chấp nhận |
| **Người biên tập** — *cùng một người, vai khác* | Điều phối người khác dịch, rồi tự biên tập lại bản dịch của họ **ngay trong app** | Không phải người dùng thứ hai — **là chính người dùng primary ở một vai khác**, và vai này đổi theo từng Tác phẩm |
| **Người dịch bài đăng** — *cùng một người, vai khác* | Dịch bài báo, bài viết tiếng Anh rồi **bàn giao cho người khác đăng** lên website. Đơn vị công việc là một bài, không phải một bộ truyện | Không phải người dùng thứ ba — **là chính người dùng primary ở một vai khác**, và vai này đổi theo từng Tác phẩm |
| **Người đăng** — *bên thứ ba* | Nhận bản dịch và dựng lại thành bài trên website | **Không cài app, không biết gì về công cụ.** Chỉ có file trong tay |
| **Cộng đồng dịch giả Việt Nam** — *hưởng lợi rộng* | Những người bị bỏ lại khi QuickTranslator ngừng phát triển, đặc biệt trên macOS | Đối tượng của chiến lược đón nhận ở §9 |

> *(Vai **Người dịch bài đăng** và **Người đăng** bổ sung ngày 2026-08-03, sau khi chủ dự án làm rõ rằng ngoài dịch truyện còn dịch bài báo tiếng Anh để gửi đăng.)*

**Hệ quả từ vai Người dịch bài đăng:** vai này khác vai dịch truyện ở **ba điểm**, và cả ba đều đẩy yêu cầu về phía **đường xuất** chứ không phải đường dịch.

1. **Có bên thứ ba.** Người đăng không cài app và không đọc hướng dẫn — file bàn giao là toàn bộ những gì họ có.
2. **Có nghĩa vụ ghi nguồn.** Với bài báo, ghi tác giả và link bài gốc thường **là điều kiện của giấy phép bài gốc**, không phải phép lịch sự. Để nó thành thao tác tay của người đăng là chỗ nó sẽ bị quên.
3. **File bàn giao phải mang theo thứ dựng lại được bài, không chỉ mang chữ** — ảnh, chú thích ảnh, và khối ghi nguồn.

Ba điểm này sinh ra FR128, FR129, FR130 và FR131. **Mô hình dữ liệu không đổi:** FR2 đã lường trước — *"bài báo"* là một Tác phẩm có đúng một Chương, không có thực thể thứ ba.

**Hệ quả kiến trúc từ vai Reviewer:** AuraTranslate là ứng dụng **một người dùng**. Cộng tác diễn ra qua trao đổi file, không qua tài khoản hay đồng bộ đám mây. Luồng Export/Import `.docx` (C8) vì vậy **không phải tính năng phụ — nó là cầu nối duy nhất** tới nhóm review.

**Hệ quả từ vai Người biên tập:** vai này **không** cần Review Mode (FR92–FR94). Review Mode tồn tại để so *bản của tôi* với *bản reviewer đã sửa* — ở vai biên tập chỉ có **một bản** đang được hoàn thiện, nên môi trường đúng là Workspace bình thường với bản dịch đã điền sẵn (FR115). Điều thật sự cần cho vai này là **xuất xứ ở cấp segment** (FR117) và **bảo vệ Translation Memory** (FR118) — vì hai vai dùng chung một kho TM.

---

## 3. Phạm vi

### 3.1 Trong phạm vi v1

**v1 bao gồm toàn bộ mười nhóm năng lực ở §5.1.** Đây là quyết định có ý thức của chủ dự án: không có mốc trung gian nào được coi là "xong".

> ⚠️ **Đây là rủi ro lớn nhất của dự án, không phải một tuyên bố phạm vi trung tính.** Xem **R1** ở §12 — khối lượng này là nhiều năm cho một người, và rủi ro nằm ở chỗ dự án không bao giờ đạt tới trạng thái dùng được. §10 xử lý nó bằng **trình tự**, không bằng cắt phạm vi.

### 3.2 Ngoài phạm vi

- Cặp ngôn ngữ khác ngoài **Anh → Việt** và **Trung → Việt**
- Cloud sync, tài khoản người dùng, real-time collaboration
- Bản web và bản mobile
- Dịch tự động hàng loạt không có người biên tập
- Ký số / notarization bản phát hành (xem §9 — ràng buộc kinh phí, không phải thiếu sót)
- **Hỗ trợ trình đọc màn hình** (ARIA đầy đủ, VoiceOver/NVDA) — xem NFR17. Ranh giới có chủ ý, không phải thiếu sót

**Ngoài phạm vi ở đường nhập từ web** *(bổ sung 2026-08-03, cùng lúc với FR122)*:

- Định dạng **`.epub`** và **`.pdf`** — chủ dự án xác nhận không nhận truyện dạng epub
- **Quét trang mục lục** để tự tìm link chương
- **Lần theo "chương sau"** để tự đi hết bộ truyện
- **Bộ đọc riêng cho từng website** — v1 chỉ có một thuật toán bóc nội dung dùng chung (FR123)

> **Vì sao ba ranh giới giữa đáng ghi:** chúng là ba cách để ứng dụng **tự quyết định tải cái gì**. Loại chúng khỏi v1 giữ nguyên tắc *phạm vi và thứ tự hoàn toàn do người dùng cấp* (FR122) — và cũng là thứ giữ điểm ra mạng mới ở NFR19 kiểm chứng được.

### 3.3 Ràng buộc nền

| Ràng buộc | Giá trị |
|---|---|
| Nền tảng | Desktop native, **macOS và Windows** |
| Kiến trúc dữ liệu | Local-first, 100% trên máy người dùng |
| Ngôn ngữ nguồn | Cố định cho từng Tác phẩm, đặt lúc tạo |
| Kết nối mạng | Bắt buộc ở **đúng hai chỗ, cả hai đều do người dùng chủ động kích hoạt**: gọi AI qua BYOK, và tải nội dung ở đường nhập từ URL (FR122). **Tra cứu và mọi thao tác dịch phải chạy được offline** — một Tác phẩm đã nhập xong dịch được trọn vẹn khi mất mạng. Ràng buộc thực thi ở NFR19 *(dòng này sửa 2026-08-03; trước đó chỉ nêu BYOK)* |
| Giấy phép dự án | GPL v3 |

---

## 4. Mục tiêu & Tiêu chí thành công

Sản phẩm này **không cạnh tranh ở tốc độ và không đặt mục tiêu thời gian nào**. Nhanh hơn là hệ quả, không phải mục tiêu.

### 4.1 Tầng A — Chỉ số sản phẩm

Đã có số đo thật từ Giai đoạn 0, dùng làm ngưỡng nghiệm thu.

| Chỉ số | Ngưỡng | Căn cứ |
|---|---|---|
| Độ trễ Auto-Lookup | Tức thì với cảm nhận người dùng | Đo được 0,022 ms phía Rust (p50), payload 679 byte |
| Tra cứu khi ngoại tuyến | **100%** hoạt động không cần mạng | Điều kiện tồn tại của sản phẩm |
| Ghi nguồn định nghĩa | **100%** mục từ hiển thị nguồn | Nguyên tắc nền, xem FR28 |
| Gián đoạn UI khi auto-save | Không có gai trễ cảm nhận được | Yêu cầu tường minh từ brief |
| Mất dữ liệu khi ứng dụng sập | **≤ 5 giây** công việc | Quyết định 2026-08-02, xem NFR18 |
| Thao tác hoàn toàn bằng bàn phím | Một vòng dịch trọn một Chương **không chạm chuột** | Quyết định 2026-08-02, xem NFR17 |
| Khả năng mang dữ liệu đi | Xuất được **TMX** | Trụ định vị #3 |
| Kích thước bản cài | Trong ngân sách 150–200 MB, **không tải thêm sau khi cài** | Đo được 130 MB với 3 nguồn |

### 4.2 Tầng B — Chỉ số kết quả công việc

| Chỉ số | Đo cái gì | Vì sao |
|---|---|---|
| **Tuân thủ Glossary** | Tỷ lệ thuật ngữ có trong Glossary được AI dùng đúng trong bản đề xuất | Đo trực tiếp hiệu lực của Smart RAG Injector (C6) |
| **Mật độ lỗi reviewer sửa** | Số lỗi reviewer sửa trên mỗi 1000 từ, theo thời gian | Đo "chất lượng bản dịch tốt hơn" bằng bằng chứng bên ngoài |
| **Consistency drift** | Số biến thể tên gọi khác nhau của cùng một thực thể trong một Tác phẩm | Vấn đề nghiệp vụ mà cả ngành công nhận; là lý do tồn tại của Glossary + TM |

### 4.3 Counter-metrics — dấu hiệu đã đi sai hướng

| Counter-metric | Ngưỡng cảnh báo | Nếu tăng thì nghĩa là |
|---|---|---|
| **Tỷ lệ chấp nhận thẳng bản dịch AI không sửa** | Cao và tăng dần | Công cụ đã trượt về phía auto-translate — **phản bội trụ #1**. Người dùng không còn biên tập, chỉ còn duyệt |
| **Thời gian quản lý công cụ thay vì dịch** | Cao | Library và Glossary đã trở thành gánh nhập liệu thay vì hạ tầng tự tích luỹ |

> Hai counter-metric này không có ngưỡng số cứng ở v1 vì chưa có baseline. Chúng là **tín hiệu định hướng cho quyết định thiết kế**, không phải cổng nghiệm thu. Cần thu thập baseline trong vài tháng dùng thật.

---

## 5. Bản đồ năng lực & Thuật ngữ

### 5.1 Bản đồ năng lực

| # | Nhóm năng lực | Vai trò | Dải FR |
|---|---|---|---|
| **C1** | Library — kho tài liệu & trải nghiệm đọc | Điểm vào ứng dụng | FR1–FR15, FR43, FR45, FR115, FR116, FR119, FR120, FR122–FR128 |
| **C2** | Workspace — môi trường dịch bốn panel | Trung tâm thao tác | FR16–FR26, FR42, FR44, FR78, FR117, FR129 |
| **C3** | Embedded Dictionary & Lookup | Điều kiện tồn tại | FR27–FR41 |
| **C4** | Glossary & thuật ngữ | Nơi quyết định biên tập được ghi lại | FR46–FR55, FR79, FR113, FR114 |
| **C5** | Translation Memory & tái sử dụng | Không tra lại thứ đã tra | FR56–FR64, FR118 |
| **C6** | AI mở & Smart RAG Injector | AI đề xuất dưới quyền người biên tập | FR65–FR77 |
| **C7** | AI Proofreader | Bắt lỗi trước khi bàn giao | FR80–FR86 |
| **C8** | Cầu nối Reviewer — Export / Import / Diff | Cầu nối duy nhất tới nhóm review | FR87–FR95, FR121, FR130, FR131 |
| **C9** | Dự án & dữ liệu — `.atproj`, chỉ mục, scope | Nền của C1–C8 | FR96–FR104 |
| **C10** | Phát hành & tin cậy | Vượt rào cản không ký số | FR105–FR112 |

**Tổng: 131 yêu cầu chức năng.**

### 5.2 Glossary thuật ngữ

Mọi tài liệu downstream (`bmad-ux`, `bmad-architecture`, `bmad-create-epics-and-stories`) phải dùng đúng các tên dưới đây. **Cột "Không dùng" là các biến thể đã xuất hiện trong tài liệu đầu vào và bị loại bỏ khỏi từ vựng chính thức.**

| Thuật ngữ | Nghĩa | Không dùng |
|---|---|---|
| **Tác phẩm** | Đơn vị cấp cao nhất trong Library. Một Tác phẩm = một `.atproj` = một phạm vi Glossary/TM riêng. Tài liệu ngắn là Tác phẩm có một Chương | ~~dự án~~, ~~Project~~, ~~tài liệu~~ |
| **Chương** | Đơn vị dịch bên trong một Tác phẩm, có văn bản nguồn và văn bản đích riêng | ~~document~~, ~~file~~ |
| **Segment** | Đơn vị dịch nhỏ nhất — **một câu**. Đơn vị của Translation Memory và của luồng xác nhận | ~~đoạn~~, ~~câu~~ *(khi nói về cấu trúc dữ liệu)* |
| **Library** | Khung/chế độ thứ nhất: kho Tác phẩm, điểm vào ứng dụng | ~~Thư viện~~ *(dùng "Library" nhất quán)* |
| **Workspace** | Khung/chế độ thứ hai: môi trường dịch bốn panel | ~~màn hình dịch~~ |
| **Chế độ đọc** *(Reading Mode)* | Chế độ đọc lại bản dịch đã hoàn thành, không có công cụ biên tập | — |
| **Review Mode** | Bố cục hai cửa sổ side-by-side để đối chiếu bản dịch của mình với bản Reviewer đã sửa | ~~Diff Mode~~ |
| **Panel Lookup** | Panel số 2 của Workspace — nơi kết quả tra cứu từ điển và concordance hiện ra | ~~panel tra cứu~~ *(khi cần chính xác)* |
| **Auto-Lookup** | Cơ chế đưa kết quả tra cứu ra Panel Lookup ngay khi bôi đen, không cần copy/paste | — |
| **Glossary** | Bộ thuật ngữ do người dùng chốt, hai tầng Global và Tác phẩm. Cung cấp dữ liệu cho Smart RAG Injector | ~~từ điển cá nhân~~, ~~name database~~ |
| **Translation Memory** *(TM)* | Kho cặp *(segment nguồn → segment đích)* tích luỹ tự động khi người dùng xác nhận segment | ~~bộ nhớ dịch~~ |
| **Concordance** | Tra ngược trong TM: *"cụm từ này trước đây tôi dịch thế nào?"* | — |
| **Smart RAG Injector** | Cơ chế chèn động Glossary + segment TM tương tự vào prompt trước mỗi lần gọi AI | — |
| **Scope** | Phân cấp cấu hình hai tầng: **Global** (toàn ứng dụng) và **Tác phẩm** (ghi đè Global) | ~~Project Scope~~ *(bí danh trong PRD v8.0)* |
| **`.atproj`** | File/thư mục dự án trên đĩa. **Nguồn sự thật** của một Tác phẩm, tự chứa mọi dữ liệu của nó | — |
| **Chỉ mục Library** | Cơ sở dữ liệu tập trung phục vụ tìm kiếm xuyên Tác phẩm. **Dẫn xuất, không phải nguồn sự thật** — dựng lại được hoàn toàn từ `.atproj` | ~~database~~ |
| **Lớp nền** | Nguồn từ điển có giấy phép sạch, bắt buộc phải có | — |
| **Lớp gỡ rời** | Nguồn từ điển có rủi ro pháp lý, đóng gói tách rời; gỡ đi không làm hỏng tra cứu (FR36) | ~~lớp tuỳ chọn~~ |
| **Hán Việt** | Âm đọc Việt của ký tự Hán. Nội dung của tab Hán Việt trong Panel Source | — |
| **Segment alignment** | Bài toán khớp cấu trúc đoạn giữa file nhập từ Reviewer và dữ liệu sẵn có | ~~Structural Index Mapping~~ *(tên trong PRD v8.0)* |
| **BYOK** | *Bring Your Own Key* — người dùng dùng API key của chính mình | — |

---

## 6. Yêu cầu chức năng

### 6.1 C1 — Library

Library là **màn hình mở đầu**. Luồng vào ứng dụng là `Mở app → Library → chọn chương → Workspace`, không phải mở thẳng vào màn hình dịch. Nỗi đau gốc không chỉ là dịch chậm, mà là **không nắm được mình đang có những gì**.

#### Mô hình dữ liệu

**FR1.** Library tổ chức theo **hai tầng: Tác phẩm → Chương**. Một Tác phẩm tương ứng một dự án dịch; một Chương là một đơn vị dịch có văn bản nguồn và văn bản đích riêng.

**FR2.** Tài liệu đơn lẻ (hợp đồng, bài báo, tài liệu kỹ thuật ngắn) được biểu diễn là một **Tác phẩm có đúng một Chương**. Không có loại thực thể thứ ba.

**FR3.** Mỗi Tác phẩm mang metadata: tên, ảnh bìa *(tuỳ chọn)*, **ngôn ngữ nguồn** *(cố định cho toàn Tác phẩm, đặt lúc tạo)*, lĩnh vực/thể loại, ngày tạo, ngày sửa gần nhất.

**FR4.** Glossary và Translation Memory gắn ở **tầng Tác phẩm** — mọi Chương trong cùng Tác phẩm dùng chung.

#### Trạng thái & tiến độ

**FR5.** Trạng thái vòng đời có ở cả hai tầng, **bốn giá trị**: **Chưa bắt đầu / Đang dịch / Tạm ngưng / Đã xong**. Chương mới nhập mặc định là *Chưa bắt đầu*.

> **Vì sao bốn chứ không phải ba:** brief nêu ba trạng thái, nhưng khi nhập một Tác phẩm 2000 chương thì 1999 chương chưa hề được đụng tới. Xếp chúng vào *Tạm ngưng* làm mất nghĩa của chính trạng thái đó — *Tạm ngưng* phải giữ nguyên nghĩa **"đã làm dở rồi bỏ"**, vì đó mới là nhóm cần quay lại.

**FR6.** Trạng thái Tác phẩm được **suy ra tự động** từ trạng thái các Chương, nhưng người dùng **ghi đè thủ công được** — *Tạm ngưng* ở tầng Tác phẩm là quyết định của người, hệ thống không suy ra được.

**FR7.** Mỗi Tác phẩm hiển thị tiến độ: số Chương đã xong trên tổng số, kèm thanh tiến độ trực quan.

#### Tìm kiếm & duyệt

**FR8.** Full-text search **xuyên toàn bộ Library**: tìm đồng thời trong văn bản nguồn và văn bản dịch của mọi Tác phẩm. Kết quả trả về kèm Tác phẩm, Chương và đoạn văn bản khớp.

**FR9.** Tìm kiếm có **hai chế độ**: *chính xác dấu* (mặc định) và *khoan dung không dấu*. Hệ thống thử chế độ chính xác trước, chỉ nới lỏng khi không có kết quả hoặc khi người dùng yêu cầu.

> **Căn cứ:** Giai đoạn 0 phát hiện `unicode61` mặc định xoá dấu, gộp `má / ma / mà / mả / mã / mạ` thành một kết quả. Với công cụ dịch tiếng Việt đây là lỗi phá vỡ độ chính xác. Nhưng người dùng vẫn thường gõ không dấu cho nhanh — nên cần cả hai, không phải chọn một.

**FR10.** Lọc và sắp xếp Library theo trạng thái, lĩnh vực, ngôn ngữ nguồn và ngày sửa gần nhất.

#### Đọc lại thành quả

**FR11.** **Chế độ đọc (Reading Mode):** đọc bản dịch đã hoàn thành, đọc liên tục qua nhiều Chương, **không hiển thị công cụ biên tập**. Mặc định chỉ hiển thị bản dịch tiếng Việt; có **công tắc bật chế độ song ngữ** khi người dùng muốn đối chiếu.

Phạm vi và thao tác của Chế độ đọc: xem FR119 (đánh dấu chỗ cần sửa) và FR120 (chỉ đọc phần đã xong, dừng ở biên).

Bound tối thiểu cho "tối ưu cho việc đọc dài":
- **Độ rộng dòng giới hạn** (không kéo hết chiều ngang màn hình)
- **Cỡ chữ và chiều cao dòng chỉnh được**
- **Chế độ sáng và tối**

> Đặc tả typography đầy đủ thuộc về `bmad-ux`. Bốn bound trên là **mức sàn nghiệm thu** của PRD, không phải thiết kế hoàn chỉnh — bàn giao có chủ ý.

> **Vì sao đây là FR chứ không phải chi tiết UI:** mục đích số một của Library theo lời chủ dự án là *"vào xem/đọc lại bài viết, truyện mình đã dịch"*. Người dùng quay lại để **thưởng thức thành quả**, không chỉ để tìm file. Một bảng dữ liệu không đáp ứng được nhu cầu này.

> *(FR119–FR120 mang số cuối dãy theo quy ước không đánh số lại — bổ sung 2026-08-02 để đóng hai khoảng trống của FR11 mà `bmad-ux` phát hiện khi review lại Chế độ đọc.)*

**FR119.** **Đánh dấu chỗ cần sửa khi đang đọc.** Một thao tác đánh dấu câu đang đọc rồi **đọc tiếp ngay** — phiên đọc không bị cắt. Các chỗ đã đánh dấu gom thành **một danh sách theo Tác phẩm**; từ danh sách đó mở Workspace tại đúng segment để sửa một lượt. Có thêm thao tác thứ hai **nhảy thẳng** sang Workspace tại câu đó khi người dùng muốn sửa ngay.

> **Vì sao đánh dấu là chính chứ không phải nhảy thẳng:** hành vi thật của người dịch khi đọc lại bản của mình là **phát hiện chỗ sai**, không chỉ thưởng thức. Nhưng nhảy sang Workspace mỗi lần phát hiện thì **cắt đứt phiên đọc** — mà phiên đọc chính là thứ Chế độ đọc tồn tại để bảo vệ. Đánh dấu giữ được cả hai: đọc trọn vẹn, và không mất chỗ nào.

> **Đây không phải "công cụ biên tập" theo nghĩa FR11 cấm.** FR11 cấm Chế độ đọc mang bộ máy biên tập — vạch lề trạng thái, nút xác nhận, panel. Một dấu và một đường điều hướng không sửa gì cả. Ràng buộc kèm theo: affordance **không hiện thường trực**, chỉ hiện khi con trỏ chuột hoặc tiêu điểm bàn phím chạm tới câu.

**FR120.** **Chế độ đọc chỉ đọc phần đã xong, và dừng ở biên một cách tường minh.**
- Đọc liên tục qua các Chương ở trạng thái **Đã xong**; chạm Chương chưa xong thì dừng lại ở một **mốc rõ ràng** báo đã hết phần đã dịch, kèm đường sang Workspace để dịch tiếp.
- Chương chưa dịch **không hiển thị nguyên văn** — nguyên văn tiếng Trung xen giữa một trang đọc tiếng Việt là phá vỡ trải nghiệm, không phải bổ sung thông tin.
- Câu **chưa xác nhận** nằm trong một Chương đã xong **vẫn hiện**, nhưng **có dấu nhẹ phân biệt được**.

> **Vì sao câu chưa xác nhận phải có dấu:** Chương có thể được đánh dấu *Đã xong* bằng tay (FR6) trong khi vẫn còn câu chưa xác nhận. Hiện chúng như thể đã hoàn chỉnh là **nói dối về trạng thái công việc** — trái tinh thần FR5 và FR58, vốn rất cẩn thận về việc không tự coi thứ gì là đã xong.

**FR12.** Mở một Chương từ Library đưa thẳng vào Workspace, đúng Chương đó, khôi phục vị trí làm việc lần trước.

#### Nhập tài liệu

**FR13.** Tạo Tác phẩm mới từ file (`.txt`, `.docx`, `.md`) hoặc từ văn bản dán trực tiếp.

**FR14.** **Nhập hàng loạt:** chọn nhiều file cùng lúc, hoặc tách một file lớn thành nhiều Chương theo mẫu phân tách do người dùng cấu hình (mẫu tiêu đề hoặc biểu thức chính quy), **có màn hình xem trước kết quả tách trước khi xác nhận**.

> **Vì sao bắt buộc:** một bộ truyện 2000 chương không thể nhập bằng tay từng chương. Không có FR này thì mô hình hai tầng ở FR1 không dùng được trên thực tế.

**FR15.** Sau khi nhập: đổi tên, sắp xếp lại thứ tự, gộp và tách Chương.

#### Nhập từ URL website

> *(FR122–FR123 và FR127–FR128 mang số cuối dãy theo quy ước không đánh số lại — bổ sung 2026-08-03 sau khi chủ dự án làm rõ rằng **văn bản gốc chủ yếu đến từ website**, không phải từ file tải sẵn. Trước bổ sung này, PRD chỉ có đường nhập từ file và dán tay.)*

**FR122.** **Nhập từ URL bằng danh sách link:** người dùng dán một danh sách link, **mỗi dòng một Chương**, và hệ thống xử lý **đúng thứ tự đã cho**. Tạo Tác phẩm mới, hoặc thêm Chương vào Tác phẩm sẵn có.

**Ranh giới cứng:** ứng dụng **không quét trang mục lục**, **không lần theo "chương sau"**, **không tự tìm bất kỳ link nào ngoài danh sách được cấp**. Phạm vi và thứ tự hoàn toàn do người dùng quyết định.

> **Vì sao ranh giới này nằm trong FR chứ không nằm ở §3.2:** nó không phải một tính năng bị cắt mà là **một thuộc tính của tính năng này**. Ứng dụng không bao giờ tự quyết định tải cái gì — đó là thứ khiến điểm ra mạng mới (NFR19) kiểm chứng được, và là thứ giữ cho một công cụ local-first không âm thầm biến thành một con crawler.

**FR123.** **Bóc nội dung chính bằng một thuật toán dùng chung, có màn hình xem trước bắt buộc và sửa được bằng tay.** v1 **không có bộ đọc riêng cho từng website** — chủ dự án lấy bài từ bất kỳ site nào, không đoán trước được, nên một thuật toán chung là lời giải duy nhất mở rộng được.

Màn xem trước hiện phần đã bóc của **từng Chương** và cho người dùng **sửa ranh giới bóc bằng tay** trước khi ghi xuống đĩa. Cùng khuôn với FR14 và FR115.

> **Điều kiện nghiệm thu, không phải tính năng phụ:** thuật toán chung **chắc chắn sẽ bóc sai** trên một số site — lấy thiếu phần cuối bài, lấy thừa khối bài liên quan, hoặc trượt sang khung bình luận. Đường sửa tay là thứ biến một tỉ lệ sai chấp nhận được thành một công cụ dùng được. Nghiệm thu FR này **bao gồm đường sửa tay**; một bản chỉ có thuật toán mà không có đường sửa là chưa đạt.

**FR127.** **Ảnh trong nội dung tải từ web được tải về và lưu bên trong `.atproj`; URL gốc của ảnh lưu kèm làm metadata.**

> **Vì sao không giữ link mà phải tải về:** ba lý do độc lập, mỗi lý do đủ để quyết định. **(1)** FR45 đã yêu cầu ảnh không phụ thuộc đường dẫn ngoài — một Tác phẩm phải copy sang máy khác là mở được nguyên vẹn. **(2)** Ảnh trỏ ra origin từ xa **không hiển thị được** trong ứng dụng: chính sách nội dung của webview cấm mọi origin ngoài, và nới nó ra để chiều một trường hợp là mở lại đúng lỗ hổng mà chính sách đó tồn tại để bịt. **(3)** Ảnh trỏ ra web nghĩa là **mỗi lần mở Chương là một lần gọi về server nguồn** — site đó biết người dùng đọc gì và đọc lúc nào, trái NFR12.

> **Vì sao vẫn phải lưu URL gốc:** nó không phải dữ liệu thừa mà là **đầu vào của FR130** — người đăng bài cần link ảnh theo bài gốc. Tải ảnh về và giữ URL gốc lấy được cả hai, không phải chọn một.

**FR128.** **Xuất xứ tài liệu nguồn, ghi ở tầng Chương.** Bốn trường: **tên tác giả bài gốc · tên báo hoặc website nguồn · URL bài gốc · ngày đăng bài gốc**. Tự điền khi nhập từ URL (FR122), **sửa lại được**, và **nhập tay được** cả khi văn bản đến từ file hoặc dán trực tiếp.

> **Vì sao ở tầng Chương chứ không tầng Tác phẩm:** một bộ truyện nhập từ web có **mỗi Chương một link riêng** — gắn ở tầng Tác phẩm là mất thông tin ngay từ Chương thứ hai. Với bài báo, Tác phẩm bằng đúng một Chương (FR2) nên hai tầng trùng nhau và không mất gì. Tầng Chương đúng cho cả hai; tầng Tác phẩm chỉ đúng cho một.

#### Làm sạch và chuẩn hoá văn bản nhập

> *(FR124–FR126 bổ sung 2026-08-03. **Áp cho mọi đường nhập văn bản** — URL, file và dán tay — chứ không riêng đường web: một file `.txt` tải từ diễn đàn mang đúng những loại rác và đúng những vấn đề bảng mã như một trang web.)*

**FR124.** **Luật làm sạch rác nằm trong thân nội dung:** watermark, dòng *"nguồn: xxx.com"*, lời nhắn của người đăng, link quảng cáo chèn giữa văn bản. Luật là một **danh sách mẫu** (chuỗi hoặc biểu thức chính quy) mà người dùng **xem được, sửa được và tắt được**; màn xem trước **hiện những gì sắp bị xoá trước khi xoá**.

> **Vì sao luật phải lộ ra chứ không được ẩn:** đây là loại làm sạch **duy nhất có thể xoá nhầm nội dung thật** — nó thao tác bên trong vùng cần giữ, không phải bên ngoài như FR123. Một luật ẩn xoá nhầm một câu trong 2000 chương thì không ai phát hiện được, và không có gì để lần ngược. Đây là *máy đề xuất, người duyệt* của FR55 áp vào đường nhập.

**FR125.** **Chuẩn hoá xuống dòng và khoảng trắng:** gộp dòng bị ngắt tuỳ tiện, xoá dòng trống thừa, thống nhất cách phân đoạn.

> **Thứ tự bắt buộc:** chuẩn hoá chạy **trước** khi tách segment, nên **kết quả chuẩn hoá là thứ được lưu xuống**, không phải một lớp hiển thị đắp lên trên. Ranh giới segment tính một lần lúc nhập và không bao giờ tính lại; nếu chuẩn hoá là lớp hiển thị thì ranh giới đã lưu sẽ không khớp với thứ người dùng nhìn thấy.

**FR126.** **Phát hiện và sửa bảng mã ký tự — áp cho mọi đường nhập văn bản.** Tự phát hiện **UTF-8 · GB18030 · GBK · Big5 · UTF-16**; **hiện bảng mã đã đoán ngay trên màn hình xem trước**, cho người dùng đổi tay và **thấy kết quả đổi ngay lập tức**.

**Điều kiện nghiệm thu:** nhập một file `.txt` mã GBK chứa 2000 chương phải ra chữ Hán đúng; và khi hệ thống đoán sai, người dùng phải sửa được **mà không phải nhập lại từ đầu**.

> **🔑 Vì sao đây phải là một FR có nghiệm thu tường minh:** nó cùng hạng lỗi với FR39 — **thất bại im lặng**. Đọc một file GBK theo UTF-8 không phải lúc nào cũng báo lỗi; trường hợp hay gặp hơn là ra một chuỗi ký tự **hợp lệ về mặt kỹ thuật nhưng vô nghĩa**. Màn xem trước hiện ra bình thường, mẫu phân tách của FR14 không khớp gì cả, và người dùng thấy *"tách được 1 chương"* mà không hiểu vì sao — vì họ đang nhìn `ç¬¬ä¸€ç«` chứ không nhìn `第一章`. Không viết thành yêu cầu nghiệm thu thì lỗi này lọt vào sản phẩm và biểu hiện thành *"tách chương không chạy"*, ở **đúng thao tác đầu tiên người dùng làm với ứng dụng**.

#### Nhập tài liệu song ngữ

> *(FR115–FR116 mang số cuối dãy theo quy ước không đánh số lại — bổ sung 2026-08-02 sau khi chủ dự án làm rõ rằng có Tác phẩm do người khác dịch và bàn giao dưới dạng file hai cột.)*

**FR115.** **Nhập tài liệu song ngữ tạo Tác phẩm hoàn chỉnh:** từ file hai cột — bảng trong `.docx`, bảng trong `.md`, hoặc `.csv`/`.tsv` — trong đó một cột là văn bản gốc và một cột là bản dịch. Người dùng **khai báo cột nào là nguồn, cột nào là đích** và ngôn ngữ nguồn. Kết quả là một Tác phẩm đầy đủ: có segment nguồn, có segment đích, đã khớp cặp.

**Bắt buộc có màn hình xem trước trước khi ghi xuống đĩa**, cùng khuôn với FR14.

**Ranh giới Chương lấy từ mẫu phân tách của FR14, và mẫu đó áp lên *cột nguồn*.** *(Làm rõ ngày 2026-08-03 — không phải yêu cầu mới, nên FR115 giữ nguyên số và tổng FR không đổi.)* Một file hai cột chứa cả bộ truyện là **một dòng văn bản chưa chia Chương**, đúng hình dạng đầu vào mà FR14 xử lý; người dùng cấu hình mẫu như mọi đường nhập file khác, và màn xem trước hiện số Chương nhận ra được trước khi xác nhận.

> **Vì sao là cột nguồn chứ không phải cột đích:** đầu Chương ở cột gốc mang **dạng ổn định máy khớp được** — `第五章`, `Chapter 5`. Cột đích do **người khác dịch**, nên nó có thể ghi *"Chương 5"*, ghi theo cách khác, hoặc bỏ hẳn dòng tiêu đề. Khớp mẫu vào cột đích là đặt độ tin cậy của **cả lần nhập** vào thói quen của một người dịch mà người dùng không kiểm soát được — và khi nó hỏng thì hỏng lặng lẽ: ra một Chương duy nhất chứa cả bộ truyện.

**Mọi Chương nhập theo đường này vào trạng thái *Đang dịch*, mọi segment ở trạng thái chưa xác nhận** — kể cả khi bản dịch trông đã hoàn chỉnh. Điều này nhất quán với FR58: hệ thống **không bao giờ tự coi một câu là đã xong**; người dùng đi qua từng câu, sửa hoặc duyệt.

> **Vì sao cần FR này:** FR13 và FR14 chỉ nhận văn bản **nguồn**; FR90 nhập bản đã sửa nhưng đòi Tác phẩm **phải tồn tại sẵn**. Tác phẩm do người khác dịch và bàn giao bằng file hai cột rơi vào giữa hai đường đó — trước FR115 thì không có cách nào đưa nó vào app mà không mất bản dịch.

**FR116.** **Khớp câu trong phạm vi từng cặp hàng:** hàng trong bảng hai cột thường là **đoạn**, trong khi segment là **câu** (FR23). Hệ thống tách cả hai phía thành câu và khớp **bên trong từng cặp hàng**. Chỗ số câu hai bên lệch nhau **phải hiện ra cho người dùng nối tay** — cùng mẫu *máy khớp, người sửa* của FR91.

> **Vì sao bài toán này nhẹ hơn FR91:** cặp hàng đã cho sẵn ranh giới, nên không gian khớp chỉ nằm trong một hàng chứ không phải cả Chương. Số câu lệch là chuyện thường gặp khi dịch Trung sang Việt — người dịch hay gộp hai câu Trung thành một câu Việt.

#### Hình ảnh trong tài liệu

> *(FR42–FR45 mang số cuối dãy theo quy ước không đánh số lại — chúng được bổ sung sau khi §6.1–6.3 đã soạn xong.)*

**FR43.** Chế độ đọc hiển thị hình ảnh nhúng **đúng vị trí** của chúng trong văn bản.

**FR45.** Hình ảnh được **lưu bên trong `.atproj`**, không phụ thuộc đường dẫn ngoài. Một Tác phẩm phải mang đi được nguyên vẹn khi copy sang máy khác.

---

### 6.2 C2 — Workspace

#### Bố cục

**FR16.** **Bốn panel trong một cửa sổ ứng dụng duy nhất:** *Source*, *Lookup*, *AI Translation*, *Editor*. Đây là câu trả lời trực tiếp cho nỗi đau "bốn đến năm cửa sổ mở cùng lúc".

**FR17.** Panel hỗ trợ kéo thả để dock/undock, gộp thành tab, và thay đổi kích thước. Mỗi panel **ẩn được hoàn toàn** — người dịch không dùng AI phải giấu được panel AI Translation.

**FR18.** Bố cục workspace được lưu và khôi phục giữa các phiên làm việc. Hỗ trợ lưu nhiều **preset bố cục** và chuyển nhanh giữa chúng.

#### Panel Source

**FR19.** Panel Source hiển thị văn bản gốc (Anh hoặc Trung) kèm **tab Hán Việt** cho tài liệu tiếng Trung — xem ở chế độ chuyển đổi hoặc song song.

**FR42.** Panel Source hiển thị hình ảnh nhúng **đúng vị trí** của chúng trong văn bản gốc.

**FR44.** **Alt-text của hình ảnh là một segment dịch được** — tham gia Translation Memory, Glossary và luồng xác nhận như mọi segment khác. Đây là điều kiện để yêu cầu "bảo lưu liên kết hình ảnh và Alt-text" khi xuất `.md` (C8) có nghĩa thật: alt-text phải được *dịch*, không chỉ được *giữ lại*.

> *(FR129 mang số cuối dãy theo quy ước không đánh số lại — bổ sung 2026-08-03 cùng với vai **Người dịch bài đăng** ở §2.)*

**FR129.** **Chú thích ảnh (caption) là một segment dịch được, tách bạch với alt-text.** Caption tham gia Translation Memory, Glossary và luồng xác nhận như mọi segment khác, và được xuất ra ở mọi định dạng có ảnh.

> **Vì sao là một FR riêng chứ không gộp vào FR44:** caption và alt-text là **hai văn bản khác nhau phục vụ hai người đọc khác nhau** — caption là dòng chữ **người đọc nhìn thấy dưới ảnh**, alt-text là mô tả **trình đọc màn hình đọc lên** khi không thấy ảnh. Bài báo thật có cả hai và nội dung chúng khác nhau: caption thường ghi bối cảnh và nguồn ảnh, alt-text mô tả cái đang có trong ảnh. Gộp làm một nghĩa là hoặc mất caption, hoặc đẩy một bản dịch sai chỗ vào bài đăng.

> **Ghi chú bàn giao `bmad-ux`:** `EXPERIENCE.md` hiện viết *"chú thích là alt-text đã dịch (FR44)"* — gộp hai thứ làm một. Câu đó cần sửa theo FR129.

#### Thao tác xuyên panel

**FR20.** **Sync Scrolling** đồng bộ vị trí cuộn giữa Source, AI Translation và Editor. Có công tắc bật/tắt rõ ràng.

**FR21.** **Auto-Lookup:** bôi đen một cụm từ ở Source, AI Translation hoặc Editor → kết quả tra cứu hiện **ngay** ở panel Lookup. Không copy, không paste, không chuyển cửa sổ.

**FR22.** **Global Hotkeys** cho các thao tác lặp lại: dịch segment hiện tại, chuyển focus giữa các panel, xác nhận segment, tra cứu cụm đang chọn, bật/tắt sync scroll. **Toàn bộ phím tắt cấu hình lại được.**

#### Biên tập theo segment

**FR23.** Editor phân đoạn văn bản thành **segment ở cấp độ câu**. Segment là đơn vị của Translation Memory (C5) và của luồng xác nhận.
- Tiếng Trung: tách theo `。！？；`
- Tiếng Anh: tách theo `. ! ?`, có xử lý các trường hợp viết tắt không phải kết câu **[A4]**

> **Vì sao câu chứ không phải đoạn:** đây là chuẩn của mọi CAT tool, và là điều kiện để Translation Memory có giá trị thật — **câu lặp lại nhiều hơn đoạn rất nhiều**. Hai đoạn văn giống hệt nhau gần như không tồn tại, nên nếu segment là đoạn thì TM sẽ hầu như không bao giờ khớp.

**FR78.** Người dùng **gộp hai segment liền nhau** hoặc **tách một segment** khi máy tách sai. Thao tác này phải có, vì tách câu tự động luôn sai ở một tỷ lệ nhất định — nhất là với dấu chấm trong viết tắt, số thập phân và hội thoại.

**FR24.** Người dùng **xác nhận từng segment**. Segment đã xác nhận được đánh dấu trực quan phân biệt với segment đang dở.

> **Ngữ nghĩa không đổi theo vai.** Dù bạn tự dịch câu đó hay đang biên tập câu do người khác dịch, *xác nhận* luôn có nghĩa **"câu này đạt chuẩn của tôi"**. Cái đổi theo vai là **xuất xứ** của bản dịch, không phải ý nghĩa của thao tác xác nhận — xem FR117.

**FR117.** **Xuất xứ bản dịch ở cấp segment**, ba giá trị: *tôi dịch* · *người khác dịch* · *nhập từ tài liệu song ngữ*. Xuất xứ được **suy ra tự động từ hành vi**, không hỏi người dùng:

| Tình huống | Xuất xứ ghi lại |
|---|---|
| Bạn gõ bản dịch rồi xác nhận | **tôi dịch** |
| Bạn **sửa** câu sẵn có rồi xác nhận | **tôi dịch** — câu sau khi sửa là chữ của bạn |
| Bạn duyệt **nguyên văn** câu sẵn có, không sửa gì | **người khác dịch** |
| Segment nhập từ FR115, chưa ai đụng tới | **nhập từ tài liệu song ngữ** |

> Không thêm thao tác nào cho người dùng — hệ thống biết được vì nó thấy bạn có gõ hay không.

**FR25.** Điều hướng nhanh giữa các segment: kế tiếp, trước đó, và **segment chưa dịch kế tiếp**.

**FR26.** Chuyển Chương ngay trong Workspace (Chương trước / Chương sau) mà không phải quay về Library.

---

### 6.3 C3 — Embedded Dictionary & Lookup

> **Từ điển nhúng offline không phải một tính năng trong danh sách. Nó là điều kiện tồn tại của sản phẩm** — thứ khiến Auto-Lookup diễn ra tức thì, không cần mạng, và độc lập hoàn toàn với AI.

#### Nền tảng

**FR27.** Toàn bộ dữ liệu từ điển **nhúng trong bản cài**. Tra cứu hoạt động 100% offline, không có cơ chế tải thêm sau khi cài đặt.

#### Hình dạng của một kết quả tra cứu

**FR28.** Panel Lookup hiển thị một **bản ghi có cấu trúc**, không phải một đoạn văn bản. Mỗi mục gồm: **nguồn · từ loại · nghĩa · ví dụ[] · trích dẫn[] · ghi chú**.

**FR29.** Một từ có nhiều từ loại phải hiện thành **nhiều mục riêng biệt**, mỗi mục có ví dụ riêng. Ví dụ: cùng một chữ dùng làm động từ và làm phó từ là hai mục, không phải một chuỗi nghĩa gộp.

**FR30.** **Ví dụ gắn với từng từ loại**, không gắn với cả từ. **Trích dẫn** là trường riêng biệt với ví dụ: trích dẫn có xuất xứ văn bản.

#### Nguyên tắc nền — hiển thị nguồn

**FR31.** **Mọi định nghĩa phải hiển thị nguồn của nó. Không có ngoại lệ, không có chế độ ẩn nguồn.**

**FR32.** Khi các nguồn bất đồng về một mục từ, hệ thống **hiển thị đồng thời cả hai**, không hợp nhất thành một câu trả lời duy nhất.

> **Đây không phải phép lịch sự học thuật mà là yêu cầu bắt buộc.** Mỗi nguồn có khiếm khuyết riêng đã biết: CVDICT dịch bằng mô hình ngôn ngữ rồi rà soát tay; các bộ nền Wiktionary có độ phủ không đều; VietPhrase không xác định được tác giả. **Một công cụ hợp nhất mọi từ điển thành một câu trả lời duy nhất là một công cụ giấu đi sai sót.** Nghiên cứu kỹ thuật xác nhận: nguyên tắc sản phẩm này hoá ra cũng là một yêu cầu kỹ thuật — không có nguồn nào đúng để chọn làm "câu trả lời duy nhất".

#### Nội dung theo ngôn ngữ

**FR33.** **Tab Hán Việt:** hiển thị âm Hán Việt cho từng ký tự tiếng Trung trong văn bản nguồn.

**FR34.** Mục từ **tiếng Anh** phải có nhãn từ loại và nghĩa tiếng Việt.

**FR35.** Mục từ **tiếng Trung** phải có nhãn từ loại và ít nhất một ví dụ cách dùng khi nguồn có dữ liệu. Ở v1, nhãn từ loại và bản dịch ví dụ **bằng tiếng Anh được chấp nhận** và phải được **đánh dấu rõ là nhãn ngoại ngữ**.

> **Căn cứ:** Giai đoạn 0 đo được kaikki.org chỉ phủ **2,76%** đầu mục tiếng Trung của CVDICT, và chỉ **0,067%** có kèm ví dụ — không dùng được làm lớp từ loại cho tiếng Trung. Giải pháp đã chốt là dùng en.wiktionary làm khung từ loại + câu ví dụ, ghép nghĩa tiếng Việt từ CVDICT. Xem §8 và `addendum.md`.

#### Kiến trúc lớp nguồn

**FR36.** Các nguồn từ điển được đóng gói theo mô hình **"nền có giấy phép sạch + lớp gỡ rời được"**. Gỡ bỏ bất kỳ lớp gỡ rời nào **không được làm hỏng chức năng tra cứu** — sản phẩm vẫn hoạt động đầy đủ trên các lớp nền.

**FR37.** Người dùng bật/tắt từng nguồn từ điển trong panel Lookup.

**FR38.** Ghi công đầy đủ từng nguồn từ điển: trong ứng dụng (màn hình Attribution) và trong bản phát hành.

#### Hiệu năng tra cứu

**FR39.** Tra cứu tiếng Trung phải **trả về kết quả cho truy vấn 1 ký tự, 2 ký tự và 3 ký tự trở lên**.

> **Vì sao phải viết thành FR:** Giai đoạn 0 phát hiện FTS5 trigram trả về **rỗng** cho truy vấn 1–2 ký tự — mà phần lớn từ tiếng Trung được tra nhiều nhất lại dài 1–2 ký tự (山, 打, 中國, 學生). **Nguy hiểm ở chỗ nó không báo lỗi:** truy vấn chạy trong 0,01 ms và trả về rỗng. Nếu không viết thành yêu cầu nghiệm thu tường minh, lỗi này sẽ lọt vào sản phẩm và biểu hiện thành "tra từ không ra kết quả".

**FR40.** Tra cứu tiếng Anh nhận diện biến thể hình thái của từ. **Giới hạn đã biết và chấp nhận:** đây là *stemming*, không phải *lemmatization* thật — hệ sinh thái Rust chưa có lemmatizer trưởng thành. Đủ cho khớp Glossary, không xử lý được các biến thể bất quy tắc.

#### Tiện ích

**FR41.** Lịch sử tra cứu trong phiên làm việc, và ghim mục từ để tra lại nhanh. **`[A9]`**

---

### 6.4 C4 — Glossary & thuật ngữ

Glossary là nơi các quyết định biên tập của người dùng được ghi lại để **hệ thống ép AI tuân theo** (C6). Giá trị của nó tỷ lệ thuận với độ đầy — nên cơ chế làm đầy quan trọng ngang bản thân Glossary.

#### Cấu trúc

**FR46.** Glossary có **hai tầng**: *toàn cục* (dùng chung mọi Tác phẩm) và *theo Tác phẩm* (tên nhân vật, địa danh, công pháp, thuật ngữ chuyên ngành của riêng tài liệu đó). Khi một thuật ngữ tồn tại ở cả hai tầng, **tầng Tác phẩm thắng**.

**FR47.** Mỗi mục Glossary gồm: thuật ngữ nguồn, bản dịch, ghi chú, phân loại *(tên người / địa danh / thuật ngữ chuyên ngành / khác)*, ngày thêm, và **xuất xứ** — *nhập thủ công / đề xuất khi nhập tài liệu / thu hoạch từ bản review*.

> **Trường bản dịch có hai trạng thái:** *đã chốt* và *chờ chốt* (FR114). Chỉ mục **đã chốt** mới tham gia ép AI (FR70). Mục nhập thủ công và mục thu hoạch từ bản review (FR54) luôn vào thẳng trạng thái đã chốt, vì ở hai đường đó cặp nguồn–đích đã có sẵn.

> **Vì sao cần trường xuất xứ:** nó cho phép người dùng đánh giá độ tin cậy của một mục, và cho phép rà soát lại toàn bộ những gì máy đã đề xuất nếu về sau phát hiện cơ chế đề xuất có lỗi hệ thống.

#### Thao tác

**FR48.** Thêm nhanh vào Glossary từ **bất kỳ panel nào**: bôi đen cụm từ → thêm thuật ngữ → chọn tầng. Không phải rời màn hình đang làm việc.

**FR49.** Quản lý Glossary: tìm kiếm, sửa, xoá, **nhập và xuất** dưới định dạng văn bản mở (CSV/TSV) để chia sẻ giữa người dịch.

**FR79.** **Xuất và nhập bộ prompt** dưới dạng file văn bản mở, để người dịch chia sẻ prompt theo thể loại cho nhau.

> **Ranh giới đã chốt:** v1 hỗ trợ chia sẻ cộng đồng **chỉ qua trao đổi file** — Glossary (CSV/TSV), prompt (file văn bản), TM (TMX). **Không xây định dạng gói, không xây hạ tầng phân phối, không có server hay tài khoản.** Cộng đồng chia sẻ qua GitHub và diễn đàn, đúng cách QuickTranslator từng lan toả. Nhất quán với trụ định vị #3.

**FR50.** Mọi thuật ngữ có trong Glossary được **đánh dấu trực quan trong panel Source**, để người dịch thấy ngay câu đang dịch chứa thuật ngữ nào. Mục **chờ chốt bản dịch** (FR114) cũng được đánh dấu, nhưng **phân biệt được** với mục đã chốt — đó chính là cơ hội để người dịch chốt nó.

**FR51.** Khớp thuật ngữ **phân theo ngôn ngữ**: tiếng Trung dùng khớp chính xác; tiếng Anh dùng khớp mờ ở cấp hình thái từ (stemming) để bắt được các biến thể.

#### Tự động đề xuất — ba cơ chế

**FR52.** **Quét khi nhập tài liệu:** tìm ứng viên thuật ngữ = chuỗi lặp lại **từ 5 lần trở lên** *và* **không có trong từ điển nhúng**. **[A10]** Ngưỡng này **cấu hình lại được** — 5 là điểm khởi đầu, không phải hằng số.
- Tiếng Trung: chuỗi ký tự lặp không có trong từ điển; đối chiếu danh sách họ phổ biến để đoán tên người.
- Tiếng Anh: cụm viết hoa không đứng đầu câu.

**FR53.** **Duyệt hàng loạt:** ứng viên hiện thành danh sách xếp theo tần suất, kèm **số lần xuất hiện** và **ví dụ ngữ cảnh**. Người dùng duyệt hoặc bỏ bằng thao tác một phím — **không phải gõ**. Phím nhận đồng thời nhận **cả bản dịch đề xuất** khi có (FR113); phân loại đổi được bằng phím số. Duyệt phải **dừng giữa chừng và mở lại đúng chỗ** — một lần nhập lớn sinh ra hàng trăm ứng viên, đây là việc của nhiều buổi.

**FR54.** **Thu hoạch từ bản review:** khi nhập lại bản Reviewer đã sửa (C8), nếu phát hiện reviewer đổi thuật ngữ *X* thành *Y* một cách **nhất quán**, hệ thống đề xuất bổ sung cặp đó vào Glossary. Đề xuất phải nêu rõ **số lần đổi trên tổng số lần xuất hiện** để người dùng tự phán xét mức nhất quán.

> **🔑 Đây là đường bảo hiểm cho rủi ro "vòng phản hồi đứt".** Ngay cả khi người dịch **không bao giờ** mở Diff Viewer, công cụ vẫn hấp thụ bài học của reviewer và ép AI dùng đúng thuật ngữ ở lần sau. Vòng lặp học hỏi được đóng lại **ở tầng hệ thống** thay vì trông chờ vào kỷ luật con người.

**FR55.** **Mọi đề xuất tự động đều phải qua duyệt của người dùng. Không có cơ chế nào được tự ghi vào Glossary.**

#### Vòng đời bản dịch của một mục Glossary

> *(FR113–FR114 mang số cuối dãy theo quy ước không đánh số lại — chúng được bổ sung ngày 2026-08-02 để giải mâu thuẫn giữa FR47 và FR53 mà `bmad-ux` phát hiện khi dựng màn hình bảng chờ.)*

**FR113.** **Đề xuất bản dịch cho ứng viên:** với ứng viên tiếng Trung, hệ thống đề xuất sẵn bản dịch bằng **âm Hán Việt** của chuỗi đó, lấy từ dữ liệu đã nhúng (FR33) nên **chạy hoàn toàn ngoại tuyến**. Thao tác nhận một phím (FR53) nhận **cả thuật ngữ lẫn bản dịch đề xuất**, và mục vào Glossary ở trạng thái **đã chốt**. Bản dịch đề xuất sửa được về sau như mọi mục khác.

> **Vì sao đây là lời giải đúng chứ không phải mẹo:** với danh từ riêng tiếng Trung, âm Hán Việt **chính là** bản dịch quy ước mà cộng đồng dịch giả Việt Nam vẫn dùng — `北涼` → *Bắc Lương*, `徐鳳年` → *Từ Phượng Niên*, `聽潮閣` → *Thính Triều Các*. Đây là nhóm chiếm phần lớn ứng viên do FR52 quét ra. Đề xuất không phải phỏng đoán của máy mà là tra cứu một dữ kiện đã có.

**FR114.** **Trạng thái *chờ chốt bản dịch*:** khi không đề xuất được — ứng viên tiếng Anh, hoặc chuỗi tiếng Trung mà âm Hán Việt không phải cách dịch phù hợp — thao tác nhận vẫn đưa mục vào Glossary nhưng để trường bản dịch ở trạng thái **chờ chốt**. Lần **đầu tiên** người dịch gặp thuật ngữ đó trong Workspace, hệ thống hỏi **một lần** rồi khoá lại thành đã chốt.

> **Vì sao chốt lúc đó chứ không lúc duyệt:** ở bảng chờ người dùng đang nhìn một danh sách trần hàng trăm dòng; trong Workspace họ đang nhìn đúng câu chứa thuật ngữ đó. Quyết định về cách dịch một thuật ngữ cần ngữ cảnh, và đây là chỗ ngữ cảnh có sẵn.

> **Ranh giới giữ nguyên trụ định vị #1:** FR113 và FR114 **không** làm suy yếu FR55. Máy đề xuất, người bấm; không mục nào vào Glossary mà không có một thao tác của người dùng.

> Đây là trụ định vị #1 phát biểu dưới dạng yêu cầu. Đối thủ tự sinh glossary và tự cập nhật; AuraTranslate thay *"tự động quyết"* bằng *"tự động đề xuất, người dùng duyệt"* — bỏ gánh nặng gõ tay mà không nhường quyền biên tập.

---

### 6.5 C5 — Translation Memory & tái sử dụng

#### Tích luỹ

**FR56.** **Ghi tự động:** mỗi khi người dùng xác nhận một segment ở Editor, cặp *(nguồn → đích)* được ghi vào Translation Memory. **Không có thao tác thủ công nào.** Cặp TM mang theo **xuất xứ** kế thừa từ segment (FR117).

**FR118.** **Translation Memory không được trộn phong cách.** Mỗi cặp TM mang xuất xứ *của tôi* hoặc *của người khác*, suy ra từ FR117. **Smart RAG Injector (FR70) ưu tiên cặp *của tôi*;** cặp xuất xứ khác chỉ được chèn khi không đủ cặp của chính người dùng, và khi chèn thì **đánh dấu rõ trong prompt là văn phong tham khảo, không phải văn phong của người dùng**.

> **🔑 Đây là đường bảo hiểm cho một lời hứa lõi.** FR70 tồn tại để **AI học phong cách của chính người dùng**. Nhưng chủ dự án có Tác phẩm do người khác dịch (FR115, và vai biên tập ở §2) — nếu mọi câu được xác nhận đều đổ chung vào một kho TM không phân biệt, thì TM đầy lên bằng phong cách của người khác và AI học sai người. **Càng dùng lâu càng lệch, và không có gì báo cho người dùng biết.** Xuất xứ ở cấp cặp TM là thứ chặn điều đó mà không tốn thêm thao tác nào.

> **Vì sao không đơn giản là bỏ không ghi:** biên tập 200 chương thì toàn bộ bài học biến mất, kể cả những câu chính người dùng viết lại từ đầu. FR117 phân biệt được hai loại đó nên không phải hy sinh gì.

**FR57.** TM có **phạm vi kép**: TM riêng theo Tác phẩm và TM chung toàn cục — tương ứng hai tầng scope của Glossary.

#### Tái sử dụng — ba tầng độc lập

**FR58.** **Khớp tuyệt đối (100%):** segment y hệt đã dịch trước đây được **điền sẵn** và **đánh dấu là gợi ý cần xác nhận**. Hệ thống **không** tự coi segment đó là đã hoàn thành.

**FR59.** **Khớp mờ:** hiển thị các bản dịch cũ tương tự kèm **phần trăm khớp** và **diff phần khác biệt**, để người dùng sửa nhanh thay vì dịch lại từ đầu.

**FR60.** **Concordance:** tra ngược *"cụm từ này trước đây tôi dịch thế nào?"* trên toàn bộ TM. Kết quả đưa vào **panel Lookup**, cùng chỗ với kết quả từ điển.

**FR61.** Thuật toán khớp **phân theo ngôn ngữ**: tiếng Trung dùng n-gram ký tự (không có ranh giới từ); tiếng Anh dùng token n-gram sau stemming.

#### Quản lý & mang đi

**FR62.** Xem, sửa và xoá từng mục TM. Danh sách hiển thị **xuất xứ** của từng cặp (FR118) và lọc được theo xuất xứ — để người dùng rà lại hoặc dọn sạch phần không phải văn phong của mình.

**FR63.** Khi cùng một segment nguồn có **nhiều bản dịch khác nhau**, hệ thống giữ lại tất cả và hiển thị tất cả kèm ngày, thay vì ghi đè. Người dịch tự chọn.

**FR64.** **Xuất và nhập TMX.** Đây là yêu cầu của trụ định vị #3 — dữ liệu phải sống lâu hơn phần mềm, và người dùng không bị khoá vào `.atproj`.

---

### 6.6 C6 — AI mở & Smart RAG Injector

> **Nghiên cứu kỹ thuật kết luận: "AI mở" là phần dễ nhất, không phải khó nhất.** Ollama và LM Studio đều phơi API tương thích OpenAI, nên BYOK và local LLM dùng chung một đường cấu hình.

#### Kết nối

**FR65.** **BYOK:** người dùng nhập API key của nhà cung cấp mình chọn.

**FR66.** **Local LLM:** kết nối tới endpoint tương thích OpenAI (Ollama, LM Studio) qua **cùng một đường cấu hình** với BYOK.

**FR67.** **API key được lưu trong keychain / credential manager của hệ điều hành.** Không lưu dạng văn bản thuần trong file dự án hay file cấu hình, không đồng bộ đi đâu.

**FR68.** Cấu hình AI (nhà cung cấp, mô hình, tham số sinh) đặt ở **tầng toàn cục**, **ghi đè được theo từng Tác phẩm**.

#### Prompt

**FR69.** **Custom prompt theo thể loại** (tiên hiệp, khoa học, pháp lý, báo chí…) và theo quy chuẩn dịch. Prompt tồn tại ở cả hai tầng — toàn cục và theo Tác phẩm; **tầng Tác phẩm thắng**.

> **Vì sao đây là cơ chế chính chứ không phải tính năng phụ:** phạm vi đã chốt là *mọi lĩnh vực* với cặp ngôn ngữ cố định. Custom prompt chính là thứ khiến **một công cụ phục vụ được nhiều lĩnh vực** mà không cần nhiều chế độ riêng.

#### Smart RAG Injector

**FR70.** Trước mỗi lần gọi AI, hệ thống quét câu nguồn và **chèn động vào prompt**: (a) các thuật ngữ Glossary xuất hiện trong câu, kèm bản dịch đã chốt; (b) các segment tương tự tìm được trong TM.

> **Chỉ mục đã chốt được chèn.** Mục còn ở trạng thái *chờ chốt bản dịch* (FR114) không tham gia ép AI — chèn một trường trống vào prompt là vô nghĩa, và tệ hơn là gợi ý cho mô hình rằng thuật ngữ đó không có bản dịch quy ước.

> **Ưu tiên cặp TM của chính người dùng** (FR118). Cặp xuất xứ khác chỉ chèn khi không đủ, và phải đánh dấu rõ trong prompt là văn phong tham khảo.
> **🔑 Đây là chỗ TM và Glossary nhân giá trị cho nhau thay vì là hai tính năng rời.** Ngoài việc ép AI dùng đúng thuật ngữ, việc chèn *"những câu tương tự trước đây được dịch thế này"* khiến AI học **phong cách của chính người dùng** thay vì áp phong cách chung.

**FR71.** Người dùng **xem được prompt cuối cùng đã gửi đi**, bao gồm toàn bộ phần chèn động.

> Yêu cầu này phục vụ trụ #1: nếu người dùng không nhìn được vào hộp đen thì họ không thể là người quyết định. Nó cũng là công cụ chẩn đoán duy nhất khi AI không tuân thủ Glossary.

#### Luồng dịch

**FR72.** Kết quả AI hiện ở **panel AI Translation** và **không tự động ghi vào Editor**. Người dùng chủ động đưa sang.

**FR73.** Dịch theo **từng segment** và theo **lô nhiều segment liên tiếp**, **huỷ được giữa chừng**.

**FR74.** Kết quả hiện **dần theo dòng chảy (streaming)** khi mô hình đang sinh, không bắt người dùng chờ trọn câu trả lời.

**FR75.** Khi gặp lỗi mạng hoặc lỗi API: thông báo rõ nguyên nhân, **không mất công việc đang làm**, và cho phép thử lại **do người dùng chủ động**. Hệ thống **không được tự động thử lại nhiều lần** — với BYOK, mỗi lần gọi là tiền của người dùng.

**FR76.** Hiển thị số token đã dùng và **ước tính chi phí** cho mỗi lần gọi.

#### Ranh giới

**FR77.** **Ứng dụng phải hoạt động đầy đủ khi không cấu hình AI.** Mọi năng lực ngoài C6 và C7 — Library, Workspace, từ điển, Glossary, TM, export/import — phải chạy được mà không cần một API key nào.

> Đây là điều kiện tồn tại của định vị local-first, và cũng là lý do Giai đoạn 1 tự nó đã vượt QuickTranslator mà chưa cần đến AI.

---

### 6.7 C7 — AI Proofreader

**FR80.** Quét **chính tả và ngữ pháp tiếng Việt** trên bản dịch của người dùng.

**FR81.** **Đối chiếu bản dịch với bản gốc**, đánh dấu những đoạn nghi **dịch sai**, **dịch thoát nghĩa quá xa**, hoặc **cấu trúc câu tối nghĩa**.

> **Tiêu chí nghiệm thu cho FR81:** độ chính xác của phán đoán này không nghiệm thu được theo lối thông thường — không có đáp án đúng để đối chiếu. Thứ **đo được** là **tỷ lệ báo động giả**: số phát hiện mà người dùng đánh dấu *"không phải lỗi"* (FR84) trên tổng số phát hiện. Ngưỡng nghiệm thu là **tỷ lệ đó đủ thấp để người dùng không tắt hẳn tính năng**. Đây là chỉ số duy nhất phản ánh đúng giá trị thật của proofreader, và nó neo trực tiếp vào FR84.

**FR82.** Proofreader chạy **theo yêu cầu của người dùng** (trên một segment, một Chương, hoặc vùng đang chọn), **không chạy nền liên tục**.

> **Vì sao không chạy nền:** với BYOK, quét nền liên tục là tính phí ngoài ý muốn. Với local LLM, nó chiếm tài nguyên máy suốt phiên làm việc. Cả hai đều là chi phí người dùng không chọn.

**FR83.** Mỗi phát hiện gồm: **loại lỗi**, vị trí, **giải thích ngắn**, và **đề xuất sửa**. Người dùng chấp nhận hoặc bỏ qua **từng phát hiện một**.

**FR84.** **Bỏ qua có ghi nhớ:** khi người dùng đánh dấu một phát hiện là *"không phải lỗi"*, lần quét sau không báo lại phát hiện đó trong cùng Tác phẩm.

> Không có cơ chế này thì proofreader sẽ lặp lại cùng một cảnh báo sai ở mỗi lần quét, và người dùng sẽ tắt hẳn nó đi — mất luôn phần cảnh báo đúng.

**FR85.** **Proofreader không được tự sửa văn bản.** Mọi thay đổi phải do người dùng chấp nhận.

**FR86.** Kết quả proofread hiển thị **ngay tại chỗ trên Editor** (đánh dấu trên đúng đoạn văn), không phải một danh sách rời bắt người dùng tự đối chiếu vị trí.

---

### 6.8 C8 — Cầu nối Reviewer: Export / Import / Diff

> Luồng này **không phải tính năng phụ**. Reviewer không bắt buộc cài app, nên trao đổi file là **cầu nối duy nhất** tới nhóm review. Không có nó, AuraTranslate cắt người dùng khỏi nhóm của họ.

#### Xuất

**FR87.** Xuất **`.docx` dạng bảng hai cột**: cột trái văn bản gốc, cột phải bản dịch, **đối xứng theo segment**.

> *(FR121 mang số cuối dãy theo quy ước không đánh số lại — bổ sung 2026-08-03 theo yêu cầu của chủ dự án, sau khi `bmad-ux` dựng màn hình xuất và phát hiện PRD chưa có định dạng phục vụ việc đăng bài.)*

**FR121.** **Xuất `.docx` một khối, đối xứng theo đoạn — dành cho việc đăng bài.** Vẫn là bảng hai cột, nhưng **một hàng duy nhất cho cả Chương** và **không đường kẻ ngang**: cột trái nguyên văn, cột phải bản dịch, hai ô giữ **đúng số lần xuống đoạn như nhau** để hai bên vẫn đối chiếu được bằng mắt.

**Điều kiện nghiệm thu:** bôi đen cột phải rồi dán sang trình soạn thảo của website phải ra **văn bản liền mạch**, không kèm mảnh vụn bảng biểu. Phạm vi xuất theo FR89 như mọi định dạng khác.

Hai ràng buộc đi kèm:

- **Không nhập lại được.** Định dạng này không giữ ranh giới câu nên FR90/FR91 không áp dụng cho nó. Nó nằm **ở cuối vòng khứ hồi**, cùng nhóm với text thuần — và **màn hình xuất phải nói rõ điều đó ngay lúc chọn định dạng**, không để trong tài liệu hướng dẫn.
- **Câu chưa xác nhận không được đánh dấu trong file xuất.** Một nền màu hay một dòng ghi chú xen giữa văn xuôi sẽ đi thẳng vào bài đăng. Thay vào đó, khi phạm vi xuất còn câu chưa xác nhận thì **cảnh báo trước lúc xuất**, để người dùng quyết định.

> **Vì sao là một FR riêng chứ không phải một tuỳ chọn của FR87:** FR87 đối xứng theo **segment** để reviewer sửa rồi nhập ngược về; FR121 đối xứng theo **đoạn** để copy trọn sang website. Khác đơn vị, khác mục đích, và khác vị trí trong vòng khứ hồi. Gộp chúng thành hai ô tick của cùng một định dạng sẽ khiến người dùng chọn nhầm bản cắt đứt vòng học hỏi mà không hề biết mình vừa chọn gì.

**FR88.** Xuất **`.md` hoặc text thuần**, **bảo lưu hình ảnh và alt-text đã dịch** (FR44) **cùng chú thích ảnh đã dịch** (FR129). Hình ảnh được tham chiếu theo **kiểu người dùng chọn ở FR130** — *(mệnh đề này làm rõ ngày 2026-08-03: trước FR127 ảnh không nằm trong `.atproj` nên "bảo lưu liên kết hình ảnh" chỉ có một nghĩa; nay nó có hai và phải chọn.)*

**FR89.** Xuất theo một Chương, nhiều Chương đã chọn, hoặc cả Tác phẩm.

> *(FR130–FR131 mang số cuối dãy theo quy ước không đánh số lại — bổ sung 2026-08-03 cùng với vai **Người dịch bài đăng** ở §2.)*

**FR130.** **Chọn cách xuất hình ảnh: theo link gốc, hoặc theo file ảnh.** Lựa chọn nằm trên màn hình xuất và áp cho từng lần xuất, cùng khuôn với phạm vi xuất ở FR89.

- **Theo link gốc** — tài liệu xuất ra trỏ tới **URL ảnh của bài gốc** (FR127). Đây là thứ người đăng cần để dựng lại bài trên website.
- **Theo file ảnh** — ảnh đi kèm tài liệu xuất, lấy từ `.atproj`.

**Ràng buộc:** chỉ chọn được *theo link gốc* khi ảnh **có URL gốc lưu kèm**. Ảnh do người dùng tự thêm không có URL — khi phạm vi xuất chứa những ảnh như vậy, **màn hình xuất phải nói rõ ảnh nào sẽ không có link**, không được im lặng bỏ qua.

> **Vì sao phải nói ra thay vì âm thầm rơi:** người dùng phát hiện ảnh thiếu **sau khi đã gửi file đi** thì mất một vòng qua lại với người đăng. Đây cùng một hạng lỗi với ràng buộc *"không nhập lại được"* của FR121 — thông tin phải xuất hiện **ngay lúc chọn**, không nằm trong tài liệu hướng dẫn.

**FR131.** **Xuất khối ghi nguồn.** Năm trường: bốn trường xuất xứ tài liệu của FR128 — **tác giả · tên báo/website · URL bài gốc · ngày đăng gốc** — cộng **tên người dịch**, đặt **một lần ở cấu hình toàn cục** và không phải gõ lại mỗi bài.

Khối này **bật/tắt được** và áp cho **mọi định dạng xuất**: FR87, FR88 và FR121.

> **Mặc định tắt.** Chủ dự án đã chốt ngày 2026-08-03 rằng luồng đăng truyện **chỉ cần đúng định dạng file, không cần tiêu đề hay ghi chú đầu bài**. FR131 không đảo quyết định đó — nó phục vụ một vai khác xuất hiện sau (**Người dịch bài đăng**, §2), nơi ghi nguồn là bắt buộc. Truyện xuất ra vẫn sạch như trước trừ khi người dùng chủ động bật.

> **Vì sao ghi nguồn phải là một thao tác của công cụ chứ không của người đăng:** với bài báo, ghi tác giả và link bài gốc thường **là điều kiện của giấy phép bài gốc**, không phải phép lịch sự. Người đăng không cài app, không biết bài này dịch từ đâu, và đang làm việc từ một file. Thứ gì không nằm trong file thì không tồn tại với họ.

#### Nhập

**FR90.** Nhập lại file `.docx` / `.md` mà Reviewer đã chỉnh sửa, vào đúng Tác phẩm hiện có.

**FR91.** **Segment alignment:** hệ thống khớp cấu trúc đoạn giữa file nhập và dữ liệu sẵn có. Những segment **không khớp được phải hiện ra cho người dùng nối tay** — mẫu chuẩn của ngành là *máy khớp, người sửa*, không phải máy khớp im lặng.

#### Diff Viewer

**FR92.** **Review Mode:** workspace chuyển sang bố cục **hai cửa sổ side-by-side** — trái là bản dịch của người dùng, phải là bản đã nhập từ Reviewer.

**FR93.** Trong Review Mode, **ẩn văn bản gốc** và dùng thuật toán diff **bôi màu phần thêm / xoá / sửa** giữa hai bản dịch. Người dùng chỉ cần lướt để đối chiếu.

**FR94.** Từ Review Mode, **chấp nhận từng thay đổi** vào bản dịch của mình, hoặc bỏ qua.

#### Đường bảo hiểm

**FR95.** Việc nhập bản review **kích hoạt cơ chế thu hoạch thuật ngữ (FR54) một cách độc lập** — kể cả khi người dùng **không bao giờ mở Review Mode**.

> **🔑 Đây là FR quan trọng nhất của C8, và nó tồn tại vì một rủi ro chưa giải được.** Nguyên nhân gốc của việc người dịch không xem lại bản review vẫn chưa xác định, nên **không thể khẳng định Diff Viewer sẽ được dùng thật**. FR95 bảo đảm công sức của reviewer vẫn chuyển hoá thành giá trị ngay cả trong kịch bản xấu nhất — nơi FR92–FR94 không bao giờ được mở tới.

---

### 6.9 C9 — Dự án & dữ liệu

#### Nguồn sự thật

**FR96.** Mỗi Tác phẩm được lưu thành **một `.atproj` trên đĩa**. Đây là **nguồn sự thật**; người dùng copy, sao lưu và di chuyển tự do.

**FR97.** `.atproj` **tự chứa** mọi thứ thuộc về Tác phẩm: văn bản nguồn, bản dịch, segment, lịch sử phiên bản, Glossary dự án, TM dự án, prompt dự án và hình ảnh. **Copy sang máy khác phải mở được nguyên vẹn.**

**FR98.** Một **chỉ mục Library trung tâm** phục vụ tìm kiếm xuyên Tác phẩm (FR8). Chỉ mục này **phải dựng lại được hoàn toàn từ các `.atproj`** — mất chỉ mục không được làm mất dữ liệu.

**FR99.** **Quét lại thư mục:** phát hiện `.atproj` mới xuất hiện, `.atproj` đã bị di chuyển hoặc xoá (mục mồ côi trong Library), và cập nhật chỉ mục tương ứng.

#### Lưu và phiên bản

**FR100.** **Auto-save định kỳ, không gián đoạn UI.** Không được có gai trễ cảm nhận được khi đang gõ.

**FR101.** **Versioning:** lưu lịch sử các phiên bản dịch của từng segment; xem lại và **khôi phục** được.

**FR102.** **Sao lưu bằng cách copy thư mục là đủ.** Không được yêu cầu một thao tác export riêng để có bản sao lưu dùng được.

#### Phân cấp cấu hình

**FR103.** Mọi cấu hình tồn tại ở **hai tầng**, tầng dự án ghi đè tầng toàn cục:

| Tầng | Chứa gì |
|---|---|
| **Global** | Cấu hình AI, prompt chung, Glossary toàn cục, TM toàn cục, phím tắt, preset bố cục |
| **Tác phẩm** | Glossary riêng, prompt riêng, TM riêng, **ngôn ngữ nguồn (cố định, đặt lúc tạo)** |

> *Tầng Tác phẩm tương ứng "Project Scope" trong PRD v8.0 — xem Glossary §5.2.*

#### Quyền riêng tư

**FR104.** **Không telemetry.** Ứng dụng không gửi bất kỳ dữ liệu nào ra ngoài, trừ nội dung mà người dùng **chủ động** gửi cho nhà cung cấp AI đã cấu hình.

---

### 6.10 C10 — Phát hành & tin cậy

> **Ràng buộc nền:** không có kinh phí cho ký số. Mọi bản phát hành **không ký**, không notarize. Đây là ràng buộc thật, không phải thiếu sót cần khắc phục — và nó là **rào cản đón nhận có thật** với người dùng không rành kỹ thuật. Các FR dưới đây tồn tại để bù bằng thiết kế và tài liệu, không bằng tiền.

**FR105.** Phát hành bản cài cho **macOS và Windows** qua **GitHub Releases**.

**FR106.** Công bố **checksum SHA-256** cho mọi artifact phát hành.

**FR107.** **Build công khai qua GitHub Actions**, để bất kỳ ai cũng kiểm chứng được binary khớp với mã nguồn.

**FR108.** **Hướng dẫn cài đặt có ảnh chụp màn hình** cho cả hai hệ điều hành, xử lý tường minh cảnh báo Gatekeeper trên macOS *(chuột phải → Mở)* và SmartScreen trên Windows *(More info → Run anyway)*.

**FR109.** **Màn hình Attribution trong ứng dụng:** liệt kê mọi nguồn từ điển, giấy phép tương ứng và ghi công đầy đủ.

**FR110.** Kèm văn bản giấy phép **GPL v3** và toàn bộ giấy phép của các bộ dữ liệu trong bản phát hành.

**FR111.** Cơ chế cập nhật **chỉ kiểm tra và thông báo** phiên bản mới. **Không tự động tải, không tự động cài.**

> **Vì sao:** một cơ chế tự cập nhật trên bản không ký số là đường tấn công thật — không có chữ ký thì không có gì xác minh được bản tải về là chính chủ. Thông báo rồi để người dùng tự tải từ GitHub Releases và đối chiếu checksum là lựa chọn an toàn hơn.

**FR112.** **Chính sách gỡ bỏ dữ liệu:** nếu chủ sở hữu một nguồn không xác định được tác giả (VietPhrase) lên tiếng, phải có quy trình gỡ lớp đó khỏi bản phát hành kế tiếp **mà không ảnh hưởng chức năng** — bảo đảm bởi FR36.

---

## 7. Yêu cầu phi chức năng

Phần lớn các ngưỡng dưới đây **không phải phỏng đoán** — chúng đến từ số đo thật của Giai đoạn 0.

### 7.1 Hiệu năng

| ID | Yêu cầu | Ngưỡng | Căn cứ |
|---|---|---|---|
| **NFR1** | Độ trễ Auto-Lookup **đầu-cuối**: từ lúc thả chuột sau khi bôi đen, tới lúc kết quả hiển thị ở Panel Lookup | **p95 < 100 ms** **[A1]** | Backend đã đo **p50 0,022 ms · p95 0,046 ms**, payload 679 byte — tức backend chỉ tiêu **0,05 ms** trong ngân sách 100 ms. **Toàn bộ phần còn lại (~99,95 ms) dành cho vòng IPC Tauri và render frontend** |
| **NFR2** | Auto-save không làm gián đoạn thao tác gõ | **Không frame nào vượt 50 ms** trong lúc auto-save chạy | Cụ thể hoá yêu cầu "không có gai trễ" của brief; là điều kiện nghiệm thu cho rủi ro R9 |
| **NFR3** | Tìm kiếm full-text toàn Library | **p95 < 500 ms** trên thư viện 5.000 Chương | **[A6]** Ngưỡng tạm, đặt bằng phán đoán kỹ thuật, hiệu chỉnh sau khi đo trên thư viện thật ở Giai đoạn 3 |
| **NFR4** | Khởi động ứng dụng tới lúc Library dùng được | **< 3 giây** trên thư viện 5.000 Chương | **[A7]** Ngưỡng tạm, như A6 |
| **NFR5** | Bộ nhớ khi nhàn rỗi | **< 300 MB** | **[A8]** Ngưỡng tạm. Baseline Tauri v2 ghi nhận 20–100 MB; phần dôi dành cho chỉ mục và dữ liệu Tác phẩm đang mở |

> **Vì sao vẫn đặt ngưỡng dù chưa đo được (NFR3–NFR5):** một ngưỡng tạm sai vẫn nghiệm thu được và vẫn buộc người xây dựng phải đo; một tính từ thì không. Ba ngưỡng này được đánh dấu là tạm và có đường đóng ở Q4.

### 7.2 Dữ liệu & lưu trữ

| ID | Yêu cầu | Ngưỡng |
|---|---|---|
| **NFR6** | Kích thước bản cài kèm toàn bộ từ điển | **[A2]** Ngân sách **150–200 MB**, **không có cơ chế tải thêm sau khi cài** *(đo được 130 MB với ba nguồn đầu tiên)* |
| **NFR7** | Tra cứu khi ngoại tuyến | **100%** hoạt động không cần mạng |
| **NFR8** | Độ chính xác dấu tiếng Việt | Chỉ mục tìm kiếm **chính** phải **phân biệt dấu**. Chế độ xoá dấu chỉ được tồn tại như một chỉ mục **phụ**, không bao giờ là mặc định |
| **NFR9** | Khả năng mang dữ liệu đi | TM xuất được TMX; Glossary và prompt xuất được định dạng văn bản mở; `.atproj` tự chứa và mở được trên máy khác |
| **NFR10** | Toàn vẹn dữ liệu | Mất chỉ mục Library **không được** làm mất dữ liệu — chỉ mục dựng lại được hoàn toàn từ `.atproj` |

> **NFR8 giải thích:** Giai đoạn 0 đo được `unicode61` mặc định gộp `má / ma / mà / mả / mã / mạ` thành một kết quả duy nhất. Với một công cụ dịch tiếng Việt, đây là lỗi phá vỡ độ chính xác của toàn bộ tìm kiếm. Chi phí của việc lập chỉ mục hai lần đã biết: **~17 MB mỗi chỉ mục**, nằm gọn trong ngân sách NFR6.

### 7.3 Bảo mật & quyền riêng tư

| ID | Yêu cầu |
|---|---|
| **NFR11** | API key lưu trong **keychain / credential manager của hệ điều hành**. Không bao giờ ghi vào file cấu hình, file dự án hay log |
| **NFR12** | **Không telemetry.** Không có luồng dữ liệu ra ngoài nào ngoài **hai** luồng do người dùng chủ động kích hoạt: lời gọi AI, và tải nội dung ở đường nhập từ URL *(mở rộng 2026-08-03 cùng FR122; trước đó chỉ có lời gọi AI)* |
| **NFR13** | Không tài khoản, không đăng nhập, không đồng bộ đám mây |
| **NFR19** | **Đường nhập từ URL chỉ ra mạng khi người dùng chủ động bấm.** Không tải nền, không prefetch, không kiểm tra ngầm, không tải lại ảnh đã có. Danh sách domain ứng dụng đã gọi phải **xem được trong ứng dụng** *(số cuối dãy, bổ sung 2026-08-03)* |

> **NFR19 giải thích:** FR122 mở một điểm ra mạng thứ ba mà NFR12 trước đây không biết tới. Không có NFR19, lời hứa *"không telemetry"* trở thành thứ **không kiểm chứng được** — người dùng không có cách nào phân biệt một lời gọi hợp lệ với một lời gọi không nên có. Danh sách domain xem được là thứ biến lời hứa thành thứ **quan sát được**, và nó rẻ vì FR122 vốn đã cấm ứng dụng tự tìm link ngoài danh sách người dùng cấp.

### 7.4 Nền tảng & giấy phép

| ID | Yêu cầu |
|---|---|
| **NFR14** | Chạy native trên **macOS và Windows**, hành vi tương đương trên cả hai |
| **NFR15** | **Mọi thư viện và crate được dùng phải tương thích GPL v3.** Cần rà soát tường minh trước khi đưa mỗi phụ thuộc mới vào dự án |
| **NFR16** | Ngôn ngữ giao diện v1: **chỉ tiếng Việt**. Nhưng **toàn bộ chuỗi giao diện phải nằm ngoài mã nguồn, trong file tài nguyên riêng, ngay từ dòng code đầu tiên** |

> **NFR16 giải thích:** người dùng mục tiêu là dịch giả Việt Nam, họ không cần bản tiếng Anh — nên v1 không dịch giao diện. Nhưng tách chuỗi ra file riêng **gần như không tốn gì nếu làm từ đầu và rất đắt nếu làm sau**: thêm ngôn ngữ về sau sẽ phải rà toàn bộ codebase. Với một dự án open source mời cộng đồng quốc tế đóng góp, giữ cửa này mở là quyết định rẻ.

### 7.5 Khả năng tiếp cận & an toàn dữ liệu

> *(NFR17–NFR18 mang số cuối dãy theo quy ước không đánh số lại — bổ sung ngày 2026-08-02 để đóng hai khoảng trống mà `bmad-spec` phát hiện khi chưng cất PRD này thành SPEC.)*

| ID | Yêu cầu | Ngưỡng |
|---|---|---|
| **NFR17** | **Sàn khả năng tiếp cận** | Mọi thao tác của ứng dụng làm được **hoàn toàn bằng bàn phím** — nghiệm thu bằng một vòng dịch trọn một Chương không chạm chuột. Trạng thái focus luôn nhìn thấy rõ ở mọi panel và mọi chế độ. Tương phản văn bản đạt **WCAG AA** ở **cả hai** chế độ sáng và tối, kể cả Chế độ đọc (FR11) và phần bôi màu diff của Review Mode (FR93) |
| **NFR18** | **Cửa sổ mất dữ liệu tối đa khi ứng dụng sập** | **≤ 5 giây** công việc. Auto-save (FR100) kích hoạt khi người dùng ngừng gõ khoảng 2 giây, kèm **trần thời gian 5 giây buộc ghi** dù đang gõ liên tục. Phải đạt được **đồng thời** với NFR2 — không frame nào vượt 50 ms |

> **NFR17 giải thích:** áp từ **Giai đoạn 1**, cùng lý do với NFR16. FR22 đã bắt mọi thao tác lặp lại phải có phím tắt cấu hình lại được, nên keyboard-first gần như miễn phí nếu làm từ đầu và rất đắt nếu vá sau. **Hỗ trợ trình đọc màn hình không nằm trong v1** (§3.2) — editor theo segment trong webview là ca khó nhất của khả năng tiếp cận, và chưa có nhu cầu người dùng nào xác nhận. Đây là ranh giới có chủ ý.

> **NFR18 giải thích:** NFR2 nói auto-save không được gây gai trễ, nhưng không nói **mất bao nhiêu là chấp nhận được** — mà đó mới là thứ người dùng cảm nhận. Ngưỡng 5 giây nghĩa là mất nhiều nhất **một câu đang dở**. Với người dịch dành nửa buổi cho một Chương, 30 giây gõ đã là mất mát thật; còn ghi liên tục mỗi thay đổi thì đẩy thẳng vào R9 và xung đột với NFR2.

---

## 8. Giấy phép & xuất xứ dữ liệu

Đây là mối quan tâm lớn nhất của dự án **ngoài phạm vi kỹ thuật**, và là lý do một nguyên tắc sản phẩm (hiển thị nguồn) trở thành yêu cầu bắt buộc.

### 8.1 Giấy phép dự án: GPL v3

**GPL v3 là lựa chọn chủ động, không phải hệ quả kỹ thuật.**

Trước đây quyết định này bị ép bởi việc dùng FVDP (GPL v2+, có tính lan truyền). Sau khi loại FVDP, **toàn bộ dữ liệu còn lại mang CC-BY-SA, phạm vi công cộng hoặc Unicode License — không có gì buộc dự án phải theo GPL nữa.** Chủ dự án vẫn giữ GPL, nay là một lập trường.

Chọn **v3 chứ không phải v2** vì v3 tương thích với crate Apache-2.0 — phủ gần trọn hệ sinh thái Rust mà không phải kiểm tra từng gói.

### 8.2 Bộ nguồn từ điển

| Lớp | Nguồn | Giấy phép | Vai trò | Trạng thái |
|---|---|---|---|---|
| **Nền** | **CVDICT** | CC-BY-SA 4.0 | Từ và cụm từ ZH→VI, >122.000 mục | ✅ Sạch |
| **Nền** | **Unihan** | Unicode License | Âm Hán Việt, nền tab Hán Việt | ✅ Sạch |
| **Nền** | **CC-CEDICT** | CC-BY-SA 4.0 | Đối chiếu chéo — ý kiến thứ ba khi các nguồn Việt bất đồng | ✅ Sạch |
| **Nền** | **kaikki.org / Wiktextract** *(viwiktionary)* | CC-BY-SA + GFDL | **Từ loại + nghĩa + ví dụ cho tiếng Anh** — 133.319 mục, 100% có từ loại | ✅ Sạch |
| **Nền** | **en.wiktionary** *(qua Wiktextract)* | CC-BY-SA + GFDL | **Khung từ loại + câu ví dụ cho tiếng Trung**, ghép nghĩa tiếng Việt từ CVDICT | ✅ Sạch — xem 8.3 |
| **Gỡ rời** | **Thiều Chửu** (1942) | Phạm vi công cộng *(bản số hoá **không xác minh** — xem §8.6)* | Tự điển ký tự chuẩn mực | 🟡 Rủi ro đã chấp nhận |
| **Gỡ rời** | **Cổ hán văn** — Tam tự kinh, Thiên tự văn, Bách gia tính | Văn bản gốc thuộc phạm vi công cộng *(bản chú giải **không xác minh** — xem §8.6)* | Trích dẫn minh hoạ cách dùng cổ văn | 🟡 Rủi ro đã chấp nhận |
| **Gỡ rời** | **VietPhrase** | ❓ Không xác định được tác giả | Cách cộng đồng dịch giả **thực sự** dịch, tích luỹ hơn một thập kỷ | 🟡 Đóng gói tách rời, có chính sách gỡ (FR112) |
| **Gỡ rời** | **Hán Việt Từ Điển Trích Dẫn** | © Đặng Thế Kiệt — **đã được tác giả cho phép bằng văn bản, 2026-08-02** | Nguồn duy nhất có từ loại + ví dụ + trích dẫn **bằng tiếng Việt** cho Hán Việt | ✅ Được phép — lớp gỡ rời cao cấp, xem 8.3 |
| — | ~~Trần Văn Chánh (1999)~~ | Còn bản quyền | — | ⛔ Đã loại |

### 8.3 Lớp từ loại tiếng Trung — quyết định "B rồi C", nay đã có cả hai

Giai đoạn 0 đo được kaikki.org chỉ phủ **2,76%** đầu mục tiếng Trung của CVDICT, và chỉ **0,067%** có kèm ví dụ. Lựa chọn kaikki.org làm lớp từ loại **đúng cho tiếng Anh nhưng sai cho tiếng Trung**.

Quyết định khi đó là **chạy song song hai đường**: gửi thư xin phép HVTĐTD ngay, đồng thời dựng lớp C làm nền bắt buộc để **tiến độ dự án không phụ thuộc vào một lời đồng ý**. **Cả hai đường nay đều về đích.**

#### Lớp C — nền bắt buộc, không đổi

en.wiktionary cho khung từ loại và câu ví dụ, CVDICT cho nghĩa tiếng Việt. **Vẫn là lớp nền bắt buộc.** Việc HVTĐTD được đồng ý **không** biến lớp C thành tuỳ chọn — HVTĐTD là lớp gỡ rời, nên sản phẩm phải đầy đủ chức năng khi không có nó. FR35 vì vậy giữ nguyên: ở v1, nhãn từ loại và bản dịch ví dụ **bằng tiếng Anh được chấp nhận** cho mục từ tiếng Trung.

#### HVTĐTD — đã được phép, làm lớp cao cấp gỡ rời

Tác giả **Đặng Thế Kiệt** hồi âm ngày **2026-08-02**, xác nhận cho phép sử dụng data trong Từ điển Hán Việt Trích dẫn, và **đề nghị được thông báo khi công cụ hoàn thành**.

HVTĐTD chồng lên nền theo đúng mô hình đã dùng cho VietPhrase — chỉ thêm một lớp gỡ rời, **không đổi kiến trúc**.

> **Ràng buộc giấy phép:** phần dữ liệu này **không thuộc GPL v3** mà dùng theo phép riêng tác giả cấp — GPL không thể áp lên phần dự án không sở hữu. Phải ghi rõ trong `LICENSE`/`NOTICE` và trong màn hình Attribution (FR109). Vì đây là **phép sử dụng chứ không phải giấy phép mở**, lớp này giữ nguyên hình dạng gỡ rời: phép có thể được rút lại, và FR36 + FR112 đã bao trường hợp đó. Xem **R12** ở §12.

> **Test nghiệm thu FR36 mà lớp này cho không:** bật lớp → mục từ Hán Việt có từ loại · ví dụ · trích dẫn **bằng tiếng Việt**; xoá file dữ liệu đó → rơi về nhãn tiếng Anh của lớp nền, và toàn bộ bộ test tra cứu vẫn xanh.

**Phạm vi phân phối lại — quyết định 2026-08-02 (đóng Q8):** thư đồng ý nói *"sử dụng data"*, không nói rõ có bao gồm việc đóng gói và phân phối lại file dữ liệu kèm bản cài công khai trên GitHub Releases. Chủ dự án chọn **mặc định cho phép: đóng gói vào bản phát hành, gỡ khi tác giả yêu cầu.** Không hỏi lại trước khi đóng gói. Đây là cùng tư thế phản ứng đã chọn ở §8.6 — xem R12 ở §12.

### 8.4 Nguyên tắc kiến trúc: nền sạch + lớp gỡ rời

Cả VietPhrase và HVTĐTD đều dùng **chung một khuôn mẫu** (FR36): gỡ bất kỳ lớp gỡ rời nào cũng không làm hỏng chức năng tra cứu. Đây là thứ biến một rủi ro pháp lý thành một quyết định đóng gói.

### 8.5 Việc phải làm trước khi phát hành

1. **Ghi công đầy đủ từng nguồn** theo yêu cầu CC-BY-SA, và giữ share-alike cho phần dữ liệu phái sinh.
2. **Rà soát tương thích GPL v3** của toàn bộ crate Rust và thư viện frontend (NFR15).
3. **Ghi phép dùng HVTĐTD vào `LICENSE`/`NOTICE`** — nêu rõ phần dữ liệu này thuộc © Đặng Thế Kiệt, dùng theo phép riêng tác giả cấp, **không thuộc GPL v3**.

**Việc phải làm sau khi hoàn thành:** thông báo cho tác giả Đặng Thế Kiệt khi công cụ hoàn thành — đề nghị tường minh của tác giả trong thư đồng ý. Đây là nghĩa vụ dự án chứ không phải yêu cầu chức năng nên không mang số FR, nhưng nó là điều kiện của phép sử dụng nên không được rơi.

### 8.6 Rủi ro xuất xứ đã chấp nhận có ý thức

**Chủ dự án quyết định (2026-08-02): không xác minh xuất xứ trước khi phát hành.** Các việc dưới đây từng nằm trong danh sách bắt buộc và **đã được chủ động bỏ**:

| Việc đã bỏ | Rủi ro còn lại |
|---|---|
| Xác minh bản quyền **bản Thiều Chửu số hoá cụ thể** **[A3]** | Bản gốc 1942 nhiều khả năng đã thuộc phạm vi công cộng, nhưng **bản số hoá có thể kèm tuyên bố quyền riêng** |
| Chọn **bản Cổ hán văn không có chú giải hiện đại** | Văn bản gốc đã rất cổ, nhưng **phần chú giải của người biên soạn hiện đại thì còn bản quyền** |
| Xác nhận **phạm vi phân phối lại của HVTĐTD** trước khi đóng gói *(quyết định 2026-08-02, đóng Q8)* | Tác giả đã cho phép *"sử dụng data"*, nhưng **không nói rõ có bao gồm phân phối lại kèm bản cài công khai** hay không. Mặc định cho phép, gỡ khi tác giả yêu cầu |
| Kiểm tra điều khoản sử dụng của các website trước khi mở đường nhập từ URL *(quyết định 2026-08-03, cùng FR122)* | Tải nội dung từ website để dịch **có thể trái điều khoản của site đó**, và ranh giới thay đổi theo từng site. Chấp nhận có ý thức, không kiểm tra trước |

**Vì sao quyết định này có đường lui:** kiến trúc **lớp nền + lớp gỡ rời** (FR36) và **chính sách gỡ bỏ** (FR112) vốn được thiết kế cho VietPhrase, và **áp dụng được nguyên vẹn cho Thiều Chửu và Cổ hán văn**. Nếu có khiếu nại, hai nguồn này gỡ được khỏi bản phát hành kế tiếp mà không làm hỏng chức năng tra cứu.

**Cái giá phải trả:** rủi ro chuyển từ **chủ động** (biết trước, xử lý trước) sang **phản ứng** (xử lý khi có khiếu nại). Với một bản phát hành công khai mang tên thật, đây là đánh đổi thật chứ không phải hình thức.

> **Hệ quả bắt buộc:** vì không xác minh trước, **Thiều Chửu và Cổ hán văn phải được đóng gói như lớp gỡ rời** (FR36), **không phải lớp nền** — khác với giả định ban đầu ở bảng §8.2. Kiến trúc phải phản ánh điều này. Cùng lý do đó áp cho **HVTĐTD**: đã được phép nhưng phạm vi phân phối lại không xác nhận trước, nên lớp này giữ nguyên hình dạng gỡ rời.

> **Điểm khác của HVTĐTD so với hai nguồn kia:** chủ sở hữu **đã biết mặt, đã hồi âm và đang giữ liên lạc**. Nếu có bất đồng về phạm vi, nó sẽ đến dưới dạng một yêu cầu trực tiếp chứ không phải im lặng kéo dài — nên biện pháp phản ứng ở đây rẻ hơn hẳn.

> **Rủi ro nhập từ URL khác ba rủi ro kia về bản chất, và cần nói rõ.** Ba nguồn từ điển là **dữ liệu đóng gói vào bản phát hành** — khiếu nại đến với dự án, và FR36 cùng FR112 cho đường gỡ. Nhập từ URL thì **không đóng gói gì cả**: công cụ chỉ tải thứ người dùng đã chỉ đích danh, và hành vi đó xảy ra **trên máy người dùng, dưới tên người dùng**. Vì vậy FR36 và FR112 **không áp được** ở đây; đường lui duy nhất là gỡ chính tính năng. Đổi lại, bề mặt rủi ro của **dự án** hẹp hơn hẳn — công cụ tải nội dung theo lệnh không khác trình duyệt về bản chất, và FR122 đã cấm nó tự đi tìm thêm bất cứ thứ gì.

> Phần bản quyền trong nghiên cứu là **suy luận từ dữ kiện đã xác minh, không phải ý kiến pháp lý**.

---

## 9. Phát hành, cài đặt & tin cậy

### 9.1 Ràng buộc đã chấp nhận

**Không có kinh phí cho ký số.** Không có phương án thay thế miễn phí nào cho notarization của Apple hay chứng chỉ EV của Windows (trên 400 USD, bắt buộc token phần cứng).

| Hệ quả | Giảm nhẹ (miễn phí) |
|---|---|
| macOS chặn app chưa ký/chưa notarize | FR108 — hướng dẫn cài có ảnh chụp màn hình |
| Windows SmartScreen cảnh báo | FR108 + uy tín tích luỹ dần theo lượt tải |
| Người dùng nghi ngờ tính an toàn | FR106 checksum SHA-256 + FR107 build công khai qua GitHub Actions |
| Cộng đồng dịch giả phổ thông e ngại | Video hướng dẫn cài đặt; truyền miệng trong cộng đồng — đúng cách QuickTranslator từng lan toả |

> **Nói thẳng: [A5]** đây là rào cản đón nhận có thật với người dùng không rành kỹ thuật, và **không thể xoá bỏ bằng kỹ thuật**. Nếu về sau có tài trợ hoặc quyên góp, ký số nên là khoản chi ưu tiên hàng đầu.

### 9.2 Chuỗi ràng buộc nối tiếp

Không quyết định nào dưới đây là độc lập. Ghi lại để người đọc sau **không vô tình gỡ một mắt xích**:

```
Chọn GPL          →  phải là GPLv3 để dùng crate Apache-2.0  (NFR15)
Không kinh phí    →  không ký số
                  →  niềm tin phải đến từ nơi khác
                  →  build công khai + checksum SHA-256      (FR106, FR107)
                  →  và cấm cơ chế tự cập nhật               (FR111)
```

---

## 10. Trình tự xây dựng

> **Đây là TRÌNH TỰ XÂY DỰNG, không phải cắt giảm phạm vi.** v1 gồm toàn bộ. Thứ tự dưới đây nhằm gỡ rủi ro sớm và đạt trạng thái dùng được sớm nhất có thể.

| Giai đoạn | Nội dung | Nhóm năng lực | Trạng thái |
|---|---|---|---|
| **0** | Bốn mũi thăm dò: độ trễ Auto-Lookup, kích thước database, tokenizer lai, độ phủ kaikki.org | — | ✅ **Hoàn tất 2026-08-02** |
| **1** | Embedded Dictionary (kèm **lớp HVTĐTD**) + panel Source (kèm tab Hán Việt) + panel Lookup + Auto-Lookup | C3, một phần C2 | Kế tiếp |
| **2** | Panel Editor + AI Translation (BYOK/local) + Glossary + Smart RAG Injector | C2, C4, C6 | |
| **3** | Library: mô hình dữ liệu, trạng thái vòng đời, tìm kiếm, chế độ đọc | C1, C9 | |
| **4** | Translation Memory + tái sử dụng segment + xuất TMX | C5 | |
| **5** | Export/Import `.docx`/`.md` + segment alignment + Diff Viewer | C8 | |
| **6** | AI Proofreader | C7 | |
| **7** | Đóng gói, tài liệu cài đặt, attribution, phát hành | C10 | |

> **🔑 Giai đoạn 1 là mốc giá trị sớm nhất** — nó làm được mọi thứ QuickTranslator làm, trên macOS. Đây cũng là bằng chứng thuyết phục nhất để mời cộng đồng thử. **Nhưng nó không phải định nghĩa "xong"** — v1 chỉ hoàn thành khi cả bảy giai đoạn xong.

### Yêu cầu cắt ngang — áp từ Giai đoạn 1, không được để lại sau

Hai yêu cầu dưới đây không thuộc riêng giai đoạn nào và chung một lý do: **rẻ nếu làm từ dòng code đầu tiên, rất đắt nếu vá sau.**

- **NFR16** — toàn bộ chuỗi giao diện nằm ngoài mã nguồn, trong file tài nguyên riêng.
- **NFR17** — sàn khả năng tiếp cận: thao tác hoàn toàn bằng bàn phím, focus nhìn thấy rõ, tương phản WCAG AA ở cả hai chế độ sáng và tối.

---

## 11. Giả định & phụ thuộc

### 11.1 Assumptions Index

**Mọi giả định của PRD nằm trong bảng này.** Chỗ nào trong tài liệu đánh dấu `[An]` đều trỏ về đây; không có giả định nào tồn tại ngoài bảng.

| # | Giả định | Xuất hiện ở | Nếu sai thì sao |
|---|---|---|---|
| **A1** | Vòng IPC Tauri thật và thời gian render frontend nằm gọn trong ngân sách 100 ms của NFR1 | NFR1 | Chưa đo được ở môi trường dòng lệnh. Rủi ro đã giảm mạnh nhờ payload 679 byte — nếu chậm, nguyên nhân sẽ ở frontend, không phải đường dữ liệu |
| **A2** | Ngân sách 150–200 MB đủ cho toàn bộ nguồn từ điển | NFR6 | Unihan, Thiều Chửu, Cổ hán văn, VietPhrase chưa nạp vào database đo thử. Nếu vượt, phải cân nhắc lại lời hứa "không tải thêm sau cài" |
| **A3** | Bản Thiều Chửu số hoá và bản Cổ hán văn dùng được về mặt pháp lý — **giả định này sẽ KHÔNG được kiểm chứng trước khi phát hành** (quyết định 2026-08-02, §8.6) | §8.2, §8.6, R7 | Gỡ hai lớp đó khỏi bản phát hành kế tiếp qua FR112. Sản phẩm vẫn chạy đầy đủ trên các lớp nền có giấy phép sạch (FR36) |
| **A4** | Tách câu tự động đúng ở tỷ lệ chấp nhận được | FR23 | FR78 (gộp/tách tay) là đường lui, nhưng nếu sai quá nhiều thì thao tác thủ công sẽ nuốt hết giá trị của TM |
| **A5** | Người dùng sẵn sàng vượt qua cảnh báo Gatekeeper/SmartScreen để cài | §9.1 | Đây là rào cản đón nhận lớn nhất và không kiểm soát được bằng thiết kế |
| **A6** | Ngưỡng tìm kiếm Library **p95 < 500 ms** trên 5.000 Chương là hợp lý | NFR3 | Ngưỡng tạm đặt bằng phán đoán kỹ thuật. Hiệu chỉnh ở Giai đoạn 3 (Q4) |
| **A7** | Ngưỡng khởi động **< 3 giây** trên 5.000 Chương là hợp lý | NFR4 | Như A6 |
| **A8** | Ngưỡng bộ nhớ nhàn rỗi **< 300 MB** là hợp lý | NFR5 | Như A6 |
| **A9** | Người dùng thật sự cần lịch sử tra cứu và ghim mục từ | FR41 | Suy đoán từ thói quen dùng, chưa có input xác nhận. Bỏ FR41 không ảnh hưởng nhóm năng lực nào khác |
| **A10** | Ngưỡng khởi điểm **5 lần lặp** cho ứng viên Glossary là hợp lý | FR52 | Ngưỡng cấu hình lại được, nên sai không gây thiệt hại lâu dài — chỉ làm buổi đầu khó chịu (R4) |
| **A11** | Các **ngưỡng bố cục màn hình hẹp** do thiết kế đặt là hợp lý — đo theo **vùng làm việc** (chiều cao cửa sổ trừ thanh tiêu đề và thanh trạng thái), không theo kích thước màn hình | Thiết kế (`bmad-ux` EXPERIENCE.md), FR16, FR17, Q9 | Ngưỡng tạm đặt bằng phán đoán thiết kế trên mockup, chưa chạy trên máy thật. Hiệu chỉnh khi có bản chạy được (Q9) — cùng loại A6/A7/A8. **Thứ tự hy sinh không đổi khi hiệu chỉnh số:** Đề xuất AI nhường trước, Tra cứu nhường sau (rút về thanh trạng thái, không bao giờ mất hẳn), cặp Nguyên văn \| Bản dịch không bao giờ nhường |

### 11.2 Phụ thuộc bên ngoài

| Phụ thuộc | Loại | Ghi chú |
|---|---|---|
| ~~Hồi âm của tác giả HVTĐTD~~ | — | ✅ **Đã đóng 2026-08-02** — tác giả đồng ý bằng văn bản. Còn lại: xác nhận phạm vi phân phối lại (Q8) và nghĩa vụ thông báo khi hoàn thành (§8.5) |
| Nhà cung cấp AI (BYOK) | Dịch vụ bên thứ ba | Người dùng tự chọn và tự trả. Endpoint tương thích OpenAI là hợp đồng tích hợp duy nhất |
| Ollama / LM Studio | Phần mềm bên thứ ba | Dùng chung đường cấu hình với BYOK, không phải tích hợp riêng |
| GitHub Releases + Actions | Hạ tầng phát hành | Là nền của FR105–FR107 |

---

## 12. Rủi ro

| # | Rủi ro | Mức | Biện pháp |
|---|---|---|---|
| **R1** | **Phạm vi v1 gồm toàn bộ, một người làm.** Rủi ro không nằm ở kỹ thuật mà ở việc dự án không bao giờ đạt tới trạng thái dùng được | 🔴 | Là quyết định có ý thức của chủ dự án, đã tái khẳng định trong phiên này. Giảm nhẹ bằng **trình tự** (§10), không bằng cắt phạm vi. Nếu buộc phải cắt, thứ tự gợi ý từ brief: AI Proofreader → Diff Viewer → Translation Memory |
| **R2** | **Chất lượng và xuất xứ dữ liệu từ điển.** Mỗi nguồn có khiếm khuyết riêng đã biết | 🔴 → 🟢 | **Đã chuyển thành tính năng:** FR31/FR32 bắt buộc hiển thị nguồn và hiển thị bất đồng. Đây là chỗ một rủi ro trở thành điểm khác biệt |
| **R3** | **Chưa rõ vì sao vòng phản hồi bị đứt** — không thể khẳng định Diff Viewer sẽ được dùng thật | 🔴 | **FR95** là đường bảo hiểm: thu hoạch thuật ngữ chạy độc lập với Diff Viewer. Nguyên nhân gốc là câu hỏi mở có chủ ý (§13) |
| **R4** | **Glossary khởi động từ con số không** — Glossary Enforcement chỉ phát huy khi Glossary đã đầy | 🟡 | FR52–FR54: ba cơ chế đề xuất tự động, tất cả đều qua duyệt (FR55) |
| **R5** | **Phát hành không ký số** — rào cản đón nhận với người dùng phổ thông | 🟡 | §9.1. Không thể xoá bỏ bằng kỹ thuật, chỉ giảm nhẹ bằng minh bạch |
| **R6** | **VietPhrase không rõ xuất xứ** | 🟡 | Lớp gỡ rời (FR36) + chính sách gỡ (FR112) + ghi công rõ (FR38) |
| **R7** | **Xuất xứ Thiều Chửu và Cổ hán văn không được xác minh** — quyết định chủ động không kiểm tra trước khi phát hành | 🔴 | **Không còn biện pháp phòng ngừa.** Chỉ còn biện pháp phản ứng: cả hai đóng gói làm **lớp gỡ rời** (FR36) + chính sách gỡ bỏ (FR112). Xem §8.6 |
| **R8** | **Segment alignment khi nhập bản review** sai lệch | 🟡 | Bài toán đã giải trong ngành CAT tool; FR91 áp mẫu chuẩn *máy khớp, người sửa* |
| **R9** | **Gai trễ auto-save** làm gián đoạn khi gõ | 🟡 | NFR2 là ngưỡng nghiệm thu tường minh; giải pháp kỹ thuật thuộc Architecture |
| **R10** | **Không có lemmatization thật trong hệ sinh thái Rust** | 🟢 | Stemming đủ cho khớp Glossary; giới hạn đã ghi rõ trong FR40 thay vì giấu đi |
| **R11** | **Tra từ tiếng Trung 1–2 ký tự trả về rỗng mà không báo lỗi** | 🟢 | Đã phát hiện ở Giai đoạn 0 và viết thành FR39 làm điều kiện nghiệm thu |
| **R13** | **Translation Memory trộn phong cách nhiều người dịch** — chủ dự án làm cả hai vai, nên TM có thể đầy lên bằng văn phong người khác và AI học sai người. Nguy hiểm vì **lệch dần và không có gì báo** | 🔴 → 🟡 | **FR117** xuất xứ ở cấp segment suy ra tự động từ hành vi + **FR118** TM mang xuất xứ và Smart RAG ưu tiên cặp của chính người dùng + **FR62** lọc và dọn TM theo xuất xứ. Không tốn thêm thao tác nào của người dùng |
| **R12** | **HVTĐTD dùng theo phép, không theo giấy phép mở** — phép có thể được rút lại, và phạm vi phân phối lại **không được xác nhận trước khi đóng gói** (quyết định 2026-08-02, §8.6) | 🟡 | **Thuần phản ứng, đúng như đã chọn.** Giữ nguyên hình dạng **lớp gỡ rời** dù đã được đồng ý (FR36) + chính sách gỡ (FR112) — gỡ lớp = xoá một file, không đổi mã. Lớp nền C vẫn bắt buộc nên gỡ HVTĐTD không làm hỏng chức năng nào. Giảm nhẹ đặc thù: tác giả đang giữ liên lạc |

---

## 13. Câu hỏi mở

| # | Câu hỏi | Chủ | Điều kiện đóng |
|---|---|---|---|
| **Q1** | **Vì sao người dịch không xem lại bản review?** Nguyên nhân gốc chưa xác định | Chủ dự án | **Để ngỏ có chủ ý.** Ghi nhận hiện tượng như dữ kiện quan sát được, không suy diễn động cơ. FR95 khiến câu trả lời không còn chặn tiến độ |
| ~~Q2~~ | ~~Giao diện có cần bản tiếng Anh không?~~ | — | ✅ **Đóng 2026-08-02** — v1 chỉ tiếng Việt, nhưng bắt buộc tách chuỗi giao diện ra file tài nguyên riêng ngay từ đầu (NFR16) |
| ~~Q3~~ | ~~HVTĐTD có được đồng ý không?~~ | — | ✅ **Đóng 2026-08-02** — tác giả Đặng Thế Kiệt xác nhận cho phép bằng văn bản. Xem §8.3 |
| **Q4** | **Hiệu chỉnh ngưỡng tạm A6, A7, A8** (NFR3 tìm kiếm Library, NFR4 khởi động, NFR5 bộ nhớ) | Chủ dự án | Đo trên thư viện thật ở Giai đoạn 3. **Không chặn tiến độ** — ngưỡng tạm đã đủ để nghiệm thu |
| **Q5** | **Baseline cho counter-metrics §4.3** | Chủ dự án | Cần vài tháng dùng thật mới có số so sánh |
| ~~Q6~~ | ~~Khả năng tiếp cận có nằm trong v1 không?~~ | — | ✅ **Đóng 2026-08-02** — thành **NFR17**. Sàn bàn phím + tương phản WCAG AA; trình đọc màn hình ngoài phạm vi v1 (§3.2) |
| ~~Q7~~ | ~~Mất tối đa bao nhiêu công việc khi app sập là chấp nhận được?~~ | — | ✅ **Đóng 2026-08-02** — thành **NFR18**, ngưỡng ≤ 5 giây |
| ~~Q8~~ | ~~Phép dùng HVTĐTD có bao gồm phân phối lại không?~~ | — | ✅ **Đóng 2026-08-02** — chủ dự án chọn **mặc định cho phép** đóng gói vào bản phát hành, **gỡ khi tác giả yêu cầu**. Không hỏi lại trước khi đóng gói. Xem §8.3, §8.6 và R12 |
| **Q9** | **Hiệu chỉnh ngưỡng bố cục màn hình hẹp (A11)** — bốn mốc do thiết kế đặt: **≥ 1100×820** giữ 2×2 · **< 820 cao** gộp hàng dưới thành một panel có tab · **< 1100 rộng hoặc < 700 cao** chỉ còn Nguyên văn \| Bản dịch, Tra cứu rút về ngăn kéo · **< 860 rộng** báo không hỗ trợ | Chủ dự án | **Đo trên máy thật khi có bản chạy được** (Giai đoạn 2 trở đi, ngay khi Workspace bốn panel dựng xong). **Không chặn tiến độ** — cùng lý do A6/A7/A8: một ngưỡng tạm vẫn nghiệm thu được và vẫn buộc người dựng phải đo, một tính từ thì không. Chỉ số được hiệu chỉnh; **thứ tự hy sinh panel là quyết định, không hiệu chỉnh**. Ngưỡng gốc ở `bmad-ux` EXPERIENCE.md và `mockups/narrow-layout.html` |

### Ghi chú bàn giao

**PRD này không có User Journey, và đó là lựa chọn có chủ ý.** AuraTranslate là công cụ chuyên nghiệp **một người vận hành**, nên hình dạng đúng là *capability spec* — User Journey ở đây sẽ là gánh nặng hình thức. Hệ quả cần biết: **`bmad-ux` sẽ phải tự dựng hành trình người dùng từ FR**, đặc biệt cho ba luồng có tính trải nghiệm cao:

1. **Nhập một Tác phẩm 2000 chương** (FR13 → FR14 → FR52 → FR53) — luồng có nhiều chỗ dễ hỏng nhất
2. **Một vòng dịch hoàn chỉnh một Chương** (FR12 → FR21 → FR58 → FR70 → FR24 → FR5)
3. **Nhận bản review về và hấp thụ bài học** (FR90 → FR91 → FR92 → FR54)
