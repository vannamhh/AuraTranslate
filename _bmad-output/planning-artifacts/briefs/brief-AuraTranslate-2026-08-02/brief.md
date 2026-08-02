---
title: "Product Brief: AuraTranslate"
status: final
created: 2026-08-02
updated: 2026-08-02
---

# Product Brief: AuraTranslate

## Tóm tắt điều hành

**AuraTranslate là một translation workstation chạy local-first, dành cho người dịch Anh/Trung → Việt coi trọng chất lượng hơn tốc độ.**

Công cụ mà cộng đồng dịch giả Việt Nam vẫn dùng — QuickTranslator — chỉ chạy trên Windows và ngừng phát triển từ 2022. Người dịch trên macOS không có gì thay thế. Cùng lúc đó, làn sóng công cụ dịch AI xuất hiện trong năm 2026 chạy theo hướng ngược lại: cloud-based, tự động hoá tối đa, phục vụ **độc giả muốn đọc nhanh** thay vì **người dịch muốn dịch hay**.

AuraTranslate lấp vào khoảng trống giữa hai thái cực đó, với hai khung: một **Workspace bốn panel** để dịch, và một **Library** nơi mọi bản dịch đã hoàn thành được giữ lại để tìm, để đọc, để mở ra làm tiếp. AI có mặt qua BYOK hoặc local LLM, nhưng luôn nằm dưới quyền của người biên tập.

Khác biệt cốt lõi nằm ở một câu: **các công cụ khác giúp bạn dịch xong; AuraTranslate là nơi bản dịch của bạn sống.**

## Vấn đề

Một chương tốn nửa buổi đến trọn một ngày. Phần lớn thời gian đó không phải để dịch — mà để vật lộn với công cụ.

**Công cụ đã chết nhưng chưa có người kế nhiệm.** QuickTranslator thống trị hơn một thập kỷ nhờ tra cứu Hán Việt, từ điển VietPhrase và name database. Nó vẫn chỉ chạy Windows, phiên bản cuối cùng phát hành năm 2022, và không biết gì về AI.

**Công sức không đọng lại ở đâu cả.** Mỗi tài liệu là một cửa sổ riêng — bốn, năm cửa sổ mở cùng lúc là chuyện thường. Không có gì cho biết đang làm dở những gì, đã xong bao nhiêu. Dịch xong thì đẩy lên Google Docs, và từ đó bản dịch rời khỏi công cụ, rời khỏi bản gốc, thành một tập tin trong một thư mục nào đó.

**Tra cứu vẫn là lao động thủ công.** Đối chiếu từ điển bằng tay, copy/paste qua lại giữa các cửa sổ. Thuật ngữ đã tra rồi, câu đã dịch rồi, không có gì ghi nhớ hộ — chương sau lại tra lại từ đầu.

**Vòng phản hồi bị đứt.** Reviewer bỏ công đọc và sửa, nhưng người dịch ít khi xem lại. Công sức của cả hai phía không chuyển hoá thành bài học, và cùng những lỗi ấy lặp lại ở chương tiếp theo.

## Giải pháp

Một desktop app chạy native trên macOS và Windows, kiến trúc local-first — dữ liệu nằm hoàn toàn trên máy người dùng — tổ chức quanh hai khung.

**Workspace** đặt bốn panel trong một cửa sổ duy nhất, drag & drop và chia tab tuỳ ý: *Source* (văn bản gốc kèm tab Hán Việt), *Lookup*, *AI Translation*, và *Editor*. **Auto-Lookup** đưa kết quả tra cứu ra panel Lookup ngay khi bôi đen — không copy/paste, không chuyển cửa sổ. Kèm **Sync Scrolling** giữa các panel và **Global Hotkeys** cho thao tác lặp lại.

**Library** là giá sách chứa mọi tài liệu đã dịch: ảnh bìa khi có, thanh tiến độ, trạng thái *đã xong / đang dịch / tạm ngưng*, và full-text search xuyên suốt mọi bản dịch cũ. Vào Library để đọc lại thành quả, hoặc mở ra dịch tiếp.

Bên dưới hai khung đó là bốn cơ chế tạo nên giá trị cốt lõi:

- **Translation Memory (TM)** — mọi segment đã dịch được ghi lại và tái sử dụng qua exact match và fuzzy match, kèm concordance để tra ngược *"cụm này trước đây tôi dịch thế nào"*.
- **Glossary Enforcement** — trước mỗi lần gọi AI, **Smart RAG Injector** quét câu nguồn, tìm thuật ngữ trong Glossary cùng các segment tương tự trong TM, rồi chèn động vào prompt. AI buộc phải dùng đúng từ, và học dần phong cách của chính người dùng.
- **AI Proofreader** — quét chính tả, ngữ pháp, đồng thời đối chiếu bản dịch với bản gốc để highlight những đoạn nghi dịch sai hoặc tối nghĩa.
- **Export / Import + Diff Viewer** — xuất `.docx` hai cột hoặc `.md`, import ngược bản reviewer đã sửa và highlight khác biệt. Khi reviewer sửa một thuật ngữ một cách nhất quán, hệ thống đề xuất bổ sung nó vào Glossary.

> Đặc tả kỹ thuật đầy đủ của từng phân hệ nằm ở `addendum.md`.

## Embedded Dictionary — nền tảng bắt buộc

Từ điển nhúng offline không phải một tính năng trong danh sách. **Nó là điều kiện tồn tại của sản phẩm** — thứ khiến Auto-Lookup diễn ra tức thì, không cần mạng, và độc lập hoàn toàn với AI.

Không tồn tại một bộ từ điển đơn lẻ nào vừa chuẩn, vừa đủ, vừa dùng được về mặt pháp lý. Giải pháp là **xếp lớp nhiều nguồn**, mỗi nguồn một vai trò:

| Lớp | Nguồn | Vai trò |
|---|---|---|
| **Từ loại, cách dùng, ví dụ** | **kaikki.org / Wiktextract** | Nội dung có cấu trúc cho panel Lookup |
| Ký tự và âm Hán Việt | **Thiều Chửu** + **Unihan** | Tra ký tự, tab Hán Việt |
| Từ và cụm từ Trung → Việt | **CVDICT** | Tra nhanh khi đang dịch |
| Từ và cụm từ theo lối dịch thực tế | **VietPhrase** | Cách cộng đồng dịch giả thực sự dịch, tích luỹ hơn một thập kỷ |
| Đối chiếu chéo | **CC-CEDICT** | Ý kiến thứ ba khi các nguồn tiếng Việt bất đồng |
| Ngữ liệu kinh điển | **Cổ hán văn** | Trích dẫn minh hoạ cách dùng cổ văn |

> **Panel Lookup không phải một ô hiển thị đoạn nghĩa.** Nó là một bản ghi có cấu trúc: từ loại — nghĩa theo từng từ loại — ví dụ — trích dẫn — nguồn. Một từ dùng như động từ và như phó từ phải hiện ra thành hai mục riêng, mỗi mục có ví dụ riêng.

> **Nguyên tắc nền:** panel Lookup luôn hiển thị **nguồn của mỗi định nghĩa**. Đây không phải phép lịch sự học thuật mà là yêu cầu bắt buộc — mỗi nguồn có khiếm khuyết riêng đã biết. Một công cụ hợp nhất mọi từ điển thành một câu trả lời duy nhất là một công cụ giấu đi sai sót. Khi các nguồn bất đồng, người dịch nhìn thấy sự bất đồng đó và tự phán xét.

Trừ VietPhrase, toàn bộ dữ liệu trên dùng giấy phép **CC-BY-SA**, **phạm vi công cộng**, hoặc **Unicode License** — nghĩa vụ duy nhất là ghi công và chia sẻ lại. **VietPhrase là dữ liệu cộng đồng không xác định được tác giả**, nên được đóng gói như **một lớp tách rời được**: nếu về sau phải gỡ, sản phẩm vẫn hoạt động đầy đủ. Nó ở lại vì là nguồn duy nhất phản ánh cách cộng đồng dịch giả thực sự dịch, qua hàng nghìn giờ sử dụng thật.

## Điều làm nên khác biệt

**Người dịch là trung tâm, AI là công cụ.** Các sản phẩm cùng thời đặt AI ở giữa và đẩy con người ra rìa duyệt kết quả. Ở đây ngược lại: AI đề xuất, người biên tập quyết định, và hệ thống ghi nhớ quyết định đó để lần sau đề xuất sát hơn.

**Bản dịch có nơi để sống.** Không công cụ nào trong nhóm cạnh tranh có Library. Xong việc là hết.

**Local-first thật sự.** Không tài khoản, không cloud sync, không ai đọc được tài liệu của bạn — với người dịch tài liệu nhạy cảm, đây là điều kiện tiên quyết. Và nó có mặt trên macOS, nơi QuickTranslator chưa từng đặt chân.

**Open source, giấy phép GPL.** Cộng đồng có thể đóng góp từ điển, prompt theo thể loại, và sửa những gì họ cần — và mọi bản phái sinh buộc phải mở lại cho cộng đồng. GPL ở đây là **lựa chọn có chủ đích**, không phải ràng buộc bị áp: bộ dữ liệu từ điển đã chọn không buộc dự án phải theo giấy phép nào cả.

> **Nói thẳng về lợi thế:** không có rào cản kỹ thuật nào ở đây. Rust, Tauri, SQLite là công cụ phổ thông; bất kỳ ai cũng dựng lại được. Lợi thế thật nằm ở chỗ khác — sản phẩm được xây bởi một người dịch thật, cho công việc thật của chính mình, trong một ngách mà các sản phẩm quốc tế không nhìn tới: cặp Anh/Trung → Việt, với Hán Việt là thành phần bắt buộc chứ không phải tuỳ chọn.

## Phục vụ ai

**Người dịch nghiêm túc — primary user.** Dịch Anh/Trung sang Việt trên nhiều lĩnh vực: truyện, tài liệu kỹ thuật, báo chí, hợp đồng. Sẵn sàng dành nửa ngày cho một chương vì chất lượng quan trọng hơn sản lượng. Thành công với họ nghĩa là: mở một cửa sổ thay vì năm, không tra lại thứ đã tra, và không bao giờ mất dấu một bản dịch cũ.

**Reviewer — secondary user, không bắt buộc cài đặt.** Nhiều người sẽ không rời Google Docs, và điều đó được chấp nhận: luồng Export/Import `.docx` tồn tại chính vì họ. Cộng tác diễn ra qua trao đổi file, không qua tài khoản chung.

**Cộng đồng dịch giả Việt Nam — người hưởng lợi rộng hơn.** Những người bị bỏ lại khi QuickTranslator ngừng phát triển, đặc biệt trên macOS.

## Tiêu chí thành công

Sản phẩm này không cạnh tranh ở tốc độ, và **không đặt mục tiêu thời gian nào** — đây là quyết định có chủ ý, vì giá trị nằm ở chất lượng và ở việc không mất dấu công sức. Nhanh hơn là hệ quả, không phải mục tiêu.

| Đo cái gì | Thành công trông như thế nào |
|---|---|
| Lookup | Không còn đối chiếu từ điển thủ công; mọi tra cứu diễn ra trong panel, offline, tức thì |
| Lưu trữ | Mọi bản dịch từng làm đều tìm lại được trong vài giây, kể cả sau nhiều tháng |
| Chất lượng | Thuật ngữ nhất quán xuyên suốt tài liệu dài; văn phong ổn định giữa các phiên |
| Sai sót | Không còn lỗi chính tả lọt lưới; đoạn dịch lệch nghĩa được phát hiện trước khi bàn giao |
| Môi trường | Một cửa sổ ứng dụng thay cho bốn đến năm cửa sổ rời rạc |

*Mốc tham chiếu hiện tại: một chương mất nửa buổi đến trọn một ngày.*

## Phạm vi

**Phiên bản đầu tiên bao gồm toàn bộ:** Workspace bốn panel, Library, Embedded Dictionary offline, Glossary hai tầng (global và project scope), AI mở với BYOK/local LLM và custom prompt, Translation Memory, AI Proofreader, và luồng Export/Import kèm Diff Viewer.

**Nằm ngoài phạm vi:** cặp ngôn ngữ khác ngoài Anh/Trung → Việt; cloud sync, tài khoản và real-time collaboration; bản web và mobile; dịch tự động hàng loạt không có người biên tập.

## Rủi ro đã ghi nhận

- **Phạm vi phiên bản đầu rất lớn.** Danh sách trên là khối lượng nhiều năm cho một người. Rủi ro không nằm ở kỹ thuật mà ở việc dự án không bao giờ đạt tới trạng thái dùng được. Đây là lựa chọn có ý thức; nếu tiến độ chậm, thứ tự cắt gợi ý là AI Proofreader, rồi Diff Viewer, rồi Translation Memory.
- **Chất lượng và xuất xứ dữ liệu từ điển.** Mỗi nguồn có khiếm khuyết riêng đã biết: CVDICT dịch bằng mô hình ngôn ngữ rồi rà soát tay; các bộ nền Wiktionary có độ phủ không đều giữa các mục từ; VietPhrase không xác định được tác giả. Đây chính là lý do nguyên tắc hiển thị nguồn là bắt buộc chứ không phải tuỳ chọn, và là lý do VietPhrase được tách thành lớp gỡ rời được. Ba việc còn phải làm trước khi nhúng: xác minh bản quyền bản Thiều Chửu số hoá cụ thể; chọn bản Cổ hán văn thuộc phạm vi công cộng thay vì bản có chú giải của người biên soạn; đo độ phủ thực tế của nguồn Wiktionary cho cặp Trung–Việt. Từ điển Trần Văn Chánh (1999) còn bản quyền — đã loại.
- **Chưa rõ vì sao vòng phản hồi bị đứt.** Đã ghi nhận rằng bản review ít khi được xem lại, nhưng nguyên nhân chưa xác định — nên chưa thể khẳng định Diff Viewer sẽ được dùng thật. Cơ chế tự thu hoạch thuật ngữ từ bản review là đường bảo hiểm: nó tạo giá trị ngay cả khi không ai mở Diff Viewer.
- **Glossary khởi động từ con số không.** Glossary Enforcement chỉ phát huy tác dụng khi Glossary đã đầy; cần cơ chế tự đề xuất ứng viên khi import tài liệu, nếu không người dùng sẽ bỏ cuộc ngay buổi đầu.

## Tầm nhìn

Trong hai đến ba năm, AuraTranslate trở thành thứ mà QuickTranslator từng là với thế hệ trước — công cụ mặc định của người dịch Việt Nam — nhưng cho thời đại AI, và không bỏ rơi ai vì hệ điều hành họ dùng.

Library lớn dần thành kho lưu trữ cá nhân: mọi thứ từng dịch, tìm được, đọc được, dùng lại được. Translation Memory và Glossary tích luỹ đủ dày để AI viết ra thứ ngày càng giống chính người dùng, thay vì giống một cỗ máy.

Là dự án open source, giá trị bền vững nhất có thể không phải phần mềm mà là những gì cộng đồng bồi đắp quanh nó: bộ từ điển, bộ prompt theo thể loại, và các quy chuẩn dịch thuật được chia sẻ giữa những người cùng nghề.
