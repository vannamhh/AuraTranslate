---
stepsCompleted: [1, 2, 3, 4]
inputDocuments:
  - _bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md
  - _bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/addendum.md
  - _bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md
  - _bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/DESIGN.md
  - _bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md
  - _bmad-output/specs/spec-AuraTranslate/SPEC.md
  - _bmad-output/specs/spec-AuraTranslate/requirements.md
  - _bmad-output/specs/spec-AuraTranslate/build-sequence.md
  - _bmad-output/specs/spec-AuraTranslate/glossary.md
  - _bmad-output/specs/spec-AuraTranslate/data-sources.md
  - _bmad-output/specs/spec-AuraTranslate/risks.md
  - _bmad-output/planning-artifacts/briefs/brief-AuraTranslate-2026-08-02/brief.md  # đối chiếu ở bước 4, không dùng để trích xuất
---

# AuraTranslate - Epic Breakdown

## Overview

Tài liệu này cung cấp bản phân rã epic và story đầy đủ cho AuraTranslate, chuyển các yêu cầu từ PRD, UX Design và Architecture thành các story cài đặt được.

> **Quy ước đánh số bảo toàn:** mỗi `FRn` là ID toàn cục **không bao giờ được đánh số lại**. Dải FR ở đây giữ nguyên số của PRD/SPEC, kể cả khi FR bổ sung về sau nằm giữa tài liệu (FR113–FR131).
>
> **Từ vựng bắt buộc:** Tác phẩm · Chương · Segment · Library · Workspace · Chế độ đọc · Review Mode · Panel Lookup · Auto-Lookup · Glossary · Translation Memory · Concordance · Smart RAG Injector · Scope · `.atproj` · Chỉ mục Library · Lớp nền · Lớp gỡ rời · Hán Việt · Segment alignment · BYOK. Ánh xạ sang mã: Tác phẩm → `Work` · Chương → `Chapter` · Chế độ đọc → `ReadingMode` · Panel Lookup → `LookupPanel` · Smart RAG Injector → `RagInjector` · Chỉ mục Library → `LibraryIndex`. **Cấm `Project`, `Book`, `Novel`, `Document` cho `Work`.**

## Requirements Inventory

### Functional Requirements

**Tổng: 131 FR**, phân theo mười nhóm năng lực C1–C10 (đồng nghĩa CAP-1–CAP-10 trong SPEC).

#### C1 — Library (FR1–FR15, FR43, FR45, FR115, FR116, FR119, FR120, FR122–FR128, FR132)

FR1: Library tổ chức theo hai tầng **Tác phẩm → Chương**; một Tác phẩm tương ứng một dự án dịch, một Chương là một đơn vị dịch có văn bản nguồn và văn bản đích riêng.

FR2: Tài liệu đơn lẻ (hợp đồng, bài báo, tài liệu kỹ thuật ngắn) được biểu diễn là một Tác phẩm có **đúng một Chương**. Không có loại thực thể thứ ba.

FR3: Mỗi Tác phẩm mang metadata: tên, ảnh bìa (tuỳ chọn), **ngôn ngữ nguồn** (cố định cho toàn Tác phẩm, đặt lúc tạo, không đổi được), lĩnh vực/thể loại, ngày tạo, ngày sửa gần nhất.

FR4: Glossary và Translation Memory gắn ở **tầng Tác phẩm** — mọi Chương trong cùng Tác phẩm dùng chung.

FR5: Trạng thái vòng đời có ở cả hai tầng, **bốn giá trị**: *Chưa bắt đầu / Đang dịch / Tạm ngưng / Đã xong*. Chương mới nhập mặc định *Chưa bắt đầu*.

FR6: Trạng thái Tác phẩm **suy ra tự động** từ trạng thái các Chương, nhưng người dùng **ghi đè thủ công được**.

FR7: Mỗi Tác phẩm hiển thị tiến độ: số Chương đã xong trên tổng số, kèm thanh tiến độ trực quan.

FR8: Full-text search **xuyên toàn bộ Library**, tìm đồng thời trong văn bản nguồn và văn bản dịch của mọi Tác phẩm. Kết quả trả về kèm Tác phẩm, Chương và đoạn văn bản khớp.

FR9: Tìm kiếm có **hai chế độ**: *chính xác dấu* (mặc định) và *khoan dung không dấu*. Hệ thống thử chế độ chính xác trước, chỉ nới lỏng khi không có kết quả hoặc khi người dùng yêu cầu.

FR10: Lọc và sắp xếp Library theo trạng thái, lĩnh vực, ngôn ngữ nguồn và ngày sửa gần nhất.

FR11: **Chế độ đọc:** đọc bản dịch đã hoàn thành, liên tục qua nhiều Chương, **không hiển thị công cụ biên tập**. Mặc định chỉ hiển thị bản dịch tiếng Việt; có công tắc bật chế độ song ngữ. Bound tối thiểu: độ rộng dòng giới hạn · cỡ chữ và chiều cao dòng chỉnh được · chế độ sáng và tối.

FR12: Mở một Chương từ Library đưa thẳng vào Workspace, đúng Chương đó, **khôi phục vị trí làm việc lần trước**.

FR13: Tạo Tác phẩm mới từ file (`.txt`, `.docx`, `.md`) hoặc từ văn bản dán trực tiếp.

FR14: **Nhập hàng loạt:** chọn nhiều file cùng lúc, hoặc tách một file lớn thành nhiều Chương theo mẫu phân tách do người dùng cấu hình (mẫu tiêu đề hoặc regex), **có màn hình xem trước kết quả tách trước khi xác nhận**.

FR15: Sau khi nhập: đổi tên, sắp xếp lại thứ tự, **gộp và tách Chương**.

FR43: Chế độ đọc hiển thị hình ảnh nhúng **đúng vị trí** của chúng trong văn bản.

FR45: Hình ảnh được **lưu bên trong `.atproj`**, không phụ thuộc đường dẫn ngoài; Tác phẩm mang đi nguyên vẹn khi copy sang máy khác.

FR115: **Nhập tài liệu song ngữ tạo Tác phẩm hoàn chỉnh** từ file hai cột (bảng `.docx`, bảng `.md`, `.csv`/`.tsv`). Người dùng khai báo cột nguồn, cột đích và ngôn ngữ nguồn. **Bắt buộc xem trước trước khi ghi xuống đĩa.** Ranh giới Chương lấy từ mẫu phân tách của FR14, **áp lên cột nguồn**. Mọi Chương vào trạng thái *Đang dịch*, mọi segment **chưa xác nhận**.

FR116: **Khớp câu trong phạm vi từng cặp hàng:** hệ thống tách cả hai phía thành câu và khớp bên trong từng cặp hàng; chỗ số câu lệch nhau **phải hiện ra cho người dùng nối tay**.

FR119: **Đánh dấu chỗ cần sửa khi đang đọc.** Một thao tác đánh dấu câu rồi đọc tiếp ngay; các chỗ đánh dấu gom thành **một danh sách theo Tác phẩm**. Thao tác thứ hai **nhảy thẳng** sang Workspace tại câu đó. Affordance **không hiện thường trực** — chỉ hiện khi con trỏ chuột hoặc tiêu điểm bàn phím chạm câu.

FR120: **Chế độ đọc chỉ đọc phần đã xong và dừng ở biên tường minh.** Chạm Chương chưa xong thì dừng ở mốc rõ ràng kèm đường sang Workspace. Chương chưa dịch **không hiển thị nguyên văn**. Câu **chưa xác nhận** trong Chương đã xong vẫn hiện nhưng **có dấu nhẹ phân biệt được**.

FR122: **Nhập từ URL bằng danh sách link:** người dùng dán danh sách link, **mỗi dòng một Chương**, xử lý **đúng thứ tự đã cho**; tạo Tác phẩm mới hoặc thêm Chương vào Tác phẩm sẵn có. **Ranh giới cứng:** không quét trang mục lục, không lần theo *"chương sau"*, **không tự tìm bất kỳ link nào ngoài danh sách được cấp**.

FR123: **Bóc nội dung chính bằng một thuật toán dùng chung**, có **màn hình xem trước bắt buộc** và **sửa ranh giới bóc bằng tay**. v1 không có bộ đọc riêng theo website. **Nghiệm thu bao gồm đường sửa tay** — bản chỉ có thuật toán là chưa đạt.

FR124: **Luật làm sạch rác trong thân nội dung** (watermark, dòng *"nguồn: xxx.com"*, lời nhắn người đăng, link quảng cáo). Luật là danh sách mẫu (chuỗi hoặc regex) người dùng **xem được, sửa được, tắt được**; màn xem trước **hiện những gì sắp bị xoá trước khi xoá**.

FR125: **Chuẩn hoá xuống dòng và khoảng trắng:** gộp dòng bị ngắt tuỳ tiện, xoá dòng trống thừa, thống nhất cách phân đoạn. Chạy **trước** khi tách segment; **kết quả chuẩn hoá là thứ được lưu xuống**, không phải lớp hiển thị.

FR126: **Phát hiện và sửa bảng mã ký tự — áp cho mọi đường nhập văn bản.** Tự phát hiện UTF-8 · GB18030 · GBK · Big5 · UTF-16; hiện bảng mã đã đoán ngay trên màn xem trước, cho đổi tay và **thấy kết quả đổi ngay lập tức**. Nghiệm thu: file `.txt` mã GBK 2000 chương ra chữ Hán đúng; đoán sai thì sửa được **mà không phải nhập lại từ đầu**.

FR127: **Ảnh trong nội dung tải từ web được tải về và lưu bên trong `.atproj`; URL gốc lưu kèm làm metadata.**

FR128: **Xuất xứ tài liệu nguồn, ghi ở tầng Chương** — bốn trường: tên tác giả bài gốc · tên báo/website nguồn · URL bài gốc · ngày đăng bài gốc. Tự điền khi nhập từ URL, **sửa lại được**, và **nhập tay được** cả khi văn bản đến từ file hoặc dán trực tiếp.

FR132: **Bộ lọc "cần xem" trên màn hình xem trước nhập** — đầu màn luôn hiện **hai con số** (*N Chương cần xem* · *M Chương sạch*) kèm **một thao tác lọc** về nhóm đầu. Áp cho **mọi đường nhập** đi qua màn xem trước. Một Chương vào nhóm *cần xem* khi có dấu hiệu cần mắt người: bảng mã đoán độ tin cậy thấp (FR126), ranh giới bóc bất thường (FR123), luật làm sạch khớp chỗ nghi ngờ (FR124), hoặc số Chương tách ra không khớp số đơn vị đầu vào (FR14). Bộ lọc **không bỏ qua** Chương nào — nó đổi thứ tự chú ý, không đổi phạm vi nhập.

#### C2 — Workspace (FR16–FR26, FR42, FR44, FR78, FR117, FR129)

FR16: **Bốn panel trong một cửa sổ ứng dụng duy nhất:** *Source*, *Lookup*, *AI Translation*, *Editor*.

FR17: Panel kéo thả để dock/undock, gộp thành tab, và thay đổi kích thước. Mỗi panel **ẩn được hoàn toàn**.

FR18: Bố cục workspace lưu và khôi phục giữa các phiên. Hỗ trợ nhiều **preset bố cục** và chuyển nhanh giữa chúng.

FR19: Panel Source hiển thị văn bản gốc (Anh hoặc Trung) kèm **tab Hán Việt** cho tài liệu tiếng Trung — xem ở chế độ chuyển đổi hoặc song song.

FR20: **Sync Scrolling** đồng bộ vị trí cuộn giữa Source, AI Translation và Editor, có công tắc bật/tắt rõ ràng.

FR21: **Auto-Lookup:** bôi đen một cụm từ ở Source, AI Translation hoặc Editor → kết quả tra cứu hiện **ngay** ở Panel Lookup. Không copy, không paste, không chuyển cửa sổ.

FR22: **Global Hotkeys** cho các thao tác lặp lại (dịch segment hiện tại, chuyển focus giữa panel, xác nhận segment, tra cứu cụm đang chọn, bật/tắt sync scroll). **Toàn bộ phím tắt cấu hình lại được.**

FR23: Editor phân đoạn văn bản thành **segment ở cấp độ câu**. Tiếng Trung tách theo `。！？；`; tiếng Anh tách theo `. ! ?` có xử lý viết tắt không phải kết câu. **[A4]**

FR24: Người dùng **xác nhận từng segment**. Segment đã xác nhận được đánh dấu trực quan phân biệt với segment đang dở.

FR25: Điều hướng nhanh giữa segment: kế tiếp, trước đó, và **segment chưa dịch kế tiếp**.

FR26: Chuyển Chương ngay trong Workspace (Chương trước / Chương sau) mà không phải quay về Library.

FR42: Panel Source hiển thị hình ảnh nhúng **đúng vị trí** của chúng trong văn bản gốc.

FR44: **Alt-text của hình ảnh là một segment dịch được** — tham gia Translation Memory, Glossary và luồng xác nhận như mọi segment khác.

FR78: Người dùng **gộp hai segment liền nhau** hoặc **tách một segment** khi máy tách sai.

FR117: **Xuất xứ bản dịch ở cấp segment**, ba giá trị: *tôi dịch* · *người khác dịch* · *nhập từ tài liệu song ngữ*. **Suy ra tự động từ hành vi**, không hỏi người dùng.

FR129: **Chú thích ảnh (caption) là một segment dịch được, tách bạch với alt-text.** Tham gia TM, Glossary và luồng xác nhận; xuất ra ở mọi định dạng có ảnh.

#### C3 — Embedded Dictionary & Lookup (FR27–FR41)

FR27: Toàn bộ dữ liệu từ điển **nhúng trong bản cài**. Tra cứu hoạt động 100% offline, **không có cơ chế tải thêm sau khi cài đặt**.

FR28: Panel Lookup hiển thị một **bản ghi có cấu trúc**, không phải một đoạn văn bản: **nguồn · từ loại · nghĩa · ví dụ[] · trích dẫn[] · ghi chú**.

FR29: Một từ có nhiều từ loại phải hiện thành **nhiều mục riêng biệt**, mỗi mục có ví dụ riêng.

FR30: **Ví dụ gắn với từng từ loại**, không gắn với cả từ. **Trích dẫn** là trường riêng biệt với ví dụ: trích dẫn có xuất xứ văn bản.

FR31: **Mọi định nghĩa phải hiển thị nguồn của nó. Không có ngoại lệ, không có chế độ ẩn nguồn.**

FR32: Khi các nguồn bất đồng về một mục từ, hệ thống **hiển thị đồng thời cả hai**, không hợp nhất thành một câu trả lời duy nhất.

FR33: **Tab Hán Việt:** hiển thị âm Hán Việt cho từng ký tự tiếng Trung trong văn bản nguồn.

FR34: Mục từ **tiếng Anh** phải có nhãn từ loại và nghĩa tiếng Việt.

FR35: Mục từ **tiếng Trung** phải có nhãn từ loại và ít nhất một ví dụ cách dùng khi nguồn có dữ liệu. Ở v1, nhãn từ loại và bản dịch ví dụ **bằng tiếng Anh được chấp nhận** và phải được **đánh dấu rõ là nhãn ngoại ngữ**.

FR36: Nguồn từ điển đóng gói theo mô hình **"lớp nền có giấy phép sạch + lớp gỡ rời được"**. Gỡ bất kỳ lớp gỡ rời nào **không được làm hỏng chức năng tra cứu**.

FR37: Người dùng **bật/tắt từng nguồn từ điển** trong Panel Lookup.

FR38: **Ghi công đầy đủ từng nguồn** từ điển: trong ứng dụng (màn hình Attribution) và trong bản phát hành.

FR39: Tra cứu tiếng Trung phải **trả về kết quả khác rỗng cho truy vấn 1 ký tự, 2 ký tự và 3 ký tự trở lên**.

FR40: Tra cứu tiếng Anh nhận diện biến thể hình thái của từ. **Giới hạn đã tuyên bố:** đây là *stemming*, không phải *lemmatization*.

FR41: **Lịch sử tra cứu** trong phiên làm việc, và **ghim mục từ** để tra lại nhanh. **[A9]**

#### C4 — Glossary & thuật ngữ (FR46–FR55, FR79, FR113, FR114)

FR46: Glossary có **hai tầng**: *toàn cục* và *theo Tác phẩm*. Khi một thuật ngữ tồn tại ở cả hai tầng, **tầng Tác phẩm thắng**.

FR47: Mỗi mục Glossary gồm: thuật ngữ nguồn, bản dịch, ghi chú, phân loại (*tên người / địa danh / thuật ngữ chuyên ngành / khác*), ngày thêm, và **xuất xứ** (*nhập thủ công / đề xuất khi nhập tài liệu / thu hoạch từ bản review*).

FR48: **Thêm nhanh vào Glossary từ bất kỳ panel nào**: bôi đen cụm từ → thêm thuật ngữ → chọn tầng, không phải rời màn hình đang làm việc.

FR49: Quản lý Glossary: tìm kiếm, sửa, xoá, **nhập và xuất** dưới định dạng văn bản mở (CSV/TSV).

FR50: Mọi thuật ngữ có trong Glossary được **đánh dấu trực quan trong Panel Source**. Mục *chờ chốt bản dịch* (FR114) cũng được đánh dấu nhưng **phân biệt được** với mục đã chốt.

FR51: Khớp thuật ngữ **phân theo ngôn ngữ**: tiếng Trung khớp chính xác; tiếng Anh khớp mờ ở cấp hình thái từ (stemming).

FR52: **Quét ứng viên khi nhập tài liệu:** ứng viên = chuỗi lặp lại **từ 5 lần trở lên** *và* **không có trong từ điển nhúng** **[A10]**; ngưỡng **cấu hình lại được**. Tiếng Trung: đối chiếu danh sách họ phổ biến để đoán tên người. Tiếng Anh: cụm viết hoa không đứng đầu câu.

FR53: **Duyệt hàng loạt:** ứng viên xếp theo tần suất kèm **số lần xuất hiện** và **ví dụ ngữ cảnh**. Duyệt hoặc bỏ bằng **thao tác một phím — không phải gõ**; phân loại đổi bằng phím số; duyệt **dừng giữa chừng và mở lại đúng chỗ được**.

FR54: **Thu hoạch từ bản review:** khi nhập lại bản Reviewer đã sửa, nếu reviewer đổi thuật ngữ *X* thành *Y* một cách **nhất quán**, hệ thống đề xuất bổ sung cặp đó, nêu rõ **số lần đổi trên tổng số lần xuất hiện**.

FR55: **Mọi đề xuất tự động đều phải qua duyệt của người dùng. Không có cơ chế nào được tự ghi vào Glossary.**

FR79: **Xuất và nhập bộ prompt** dưới dạng file văn bản mở, để người dịch chia sẻ prompt theo thể loại.

FR113: **Đề xuất bản dịch cho ứng viên tiếng Trung bằng âm Hán Việt** lấy từ dữ liệu đã nhúng (FR33), **chạy hoàn toàn ngoại tuyến**. Thao tác một phím của FR53 nhận **cả thuật ngữ lẫn bản dịch đề xuất**; mục vào Glossary ở trạng thái **đã chốt**, sửa được về sau.

FR114: **Trạng thái *chờ chốt bản dịch*:** khi không đề xuất được, thao tác nhận vẫn đưa mục vào Glossary nhưng để trường bản dịch ở trạng thái chờ chốt. Lần **đầu tiên** gặp thuật ngữ đó trong Workspace, hệ thống hỏi **một lần** rồi khoá thành đã chốt.

#### C5 — Translation Memory & tái sử dụng (FR56–FR64, FR118)

FR56: **Ghi tự động:** mỗi khi người dùng xác nhận một segment, cặp *(nguồn → đích)* được ghi vào TM. **Không có thao tác thủ công nào.** Cặp TM mang **xuất xứ** kế thừa từ segment (FR117).

FR57: TM có **phạm vi kép**: TM riêng theo Tác phẩm và TM chung toàn cục.

FR58: **Khớp tuyệt đối (100%):** segment y hệt đã dịch trước đây được **điền sẵn** và **đánh dấu là gợi ý cần xác nhận**. Hệ thống **không** tự coi segment đó là đã hoàn thành.

FR59: **Khớp mờ:** hiển thị các bản dịch cũ tương tự kèm **phần trăm khớp** và **diff phần khác biệt**.

FR60: **Concordance:** tra ngược *"cụm từ này trước đây tôi dịch thế nào?"* trên toàn bộ TM. Kết quả đưa vào **Panel Lookup**, cùng chỗ với kết quả từ điển.

FR61: Thuật toán khớp **phân theo ngôn ngữ**: tiếng Trung dùng n-gram ký tự; tiếng Anh dùng token n-gram sau stemming.

FR62: Xem, sửa và xoá từng mục TM. Danh sách hiển thị **xuất xứ** của từng cặp và **lọc được theo xuất xứ**.

FR63: Khi cùng một segment nguồn có **nhiều bản dịch khác nhau**, hệ thống **giữ lại tất cả** và hiển thị tất cả kèm ngày, thay vì ghi đè.

FR64: **Xuất và nhập TMX.**

FR118: **Translation Memory không được trộn phong cách.** Mỗi cặp mang xuất xứ *của tôi* hoặc *của người khác*; **Smart RAG Injector ưu tiên cặp *của tôi***, cặp xuất xứ khác chỉ chèn khi không đủ và phải **đánh dấu rõ trong prompt là văn phong tham khảo**.

#### C6 — AI mở & Smart RAG Injector (FR65–FR77)

FR65: **BYOK:** người dùng nhập API key của nhà cung cấp mình chọn.

FR66: **Local LLM:** kết nối tới endpoint tương thích OpenAI (Ollama, LM Studio) qua **cùng một đường cấu hình** với BYOK.

FR67: **API key lưu trong keychain / credential manager của hệ điều hành.** Không lưu văn bản thuần trong file dự án hay file cấu hình, không đồng bộ đi đâu.

FR68: Cấu hình AI (nhà cung cấp, mô hình, tham số sinh) đặt ở **tầng toàn cục**, **ghi đè được theo từng Tác phẩm**.

FR69: **Custom prompt theo thể loại** và theo quy chuẩn dịch. Tồn tại ở cả hai tầng; **tầng Tác phẩm thắng**.

FR70: Trước mỗi lần gọi AI, hệ thống quét câu nguồn và **chèn động vào prompt**: (a) các thuật ngữ Glossary xuất hiện trong câu kèm **bản dịch đã chốt**; (b) các segment tương tự tìm được trong TM. **Chỉ mục đã chốt được chèn** (FR114 không tham gia).

FR71: Người dùng **xem được prompt cuối cùng đã gửi đi**, bao gồm toàn bộ phần chèn động.

FR72: Kết quả AI hiện ở **panel AI Translation** và **không tự động ghi vào Editor**. Người dùng chủ động đưa sang.

FR73: Dịch theo **từng segment** và theo **lô nhiều segment liên tiếp**, **huỷ được giữa chừng**.

FR74: Kết quả hiện **dần theo dòng chảy (streaming)** khi mô hình đang sinh.

FR75: Khi gặp lỗi mạng hoặc lỗi API: thông báo rõ nguyên nhân, **không mất công việc đang làm**, cho phép thử lại **do người dùng chủ động**. Hệ thống **không được tự động thử lại**.

FR76: Hiển thị **số token đã dùng và ước tính chi phí** cho mỗi lần gọi.

FR77: **Ứng dụng phải hoạt động đầy đủ khi không cấu hình AI.** Mọi năng lực ngoài C6 và C7 phải chạy được mà không cần một API key nào.

#### C7 — AI Proofreader (FR80–FR86)

FR80: Quét **chính tả và ngữ pháp tiếng Việt** trên bản dịch của người dùng.

FR81: **Đối chiếu bản dịch với bản gốc**, đánh dấu đoạn nghi **dịch sai**, **dịch thoát nghĩa quá xa**, hoặc **cấu trúc câu tối nghĩa**. Nghiệm thu bằng **tỷ lệ báo động giả** (số phát hiện bị đánh dấu *"không phải lỗi"* trên tổng số), phải đủ thấp để người dùng không tắt hẳn tính năng.

FR82: Proofreader chạy **theo yêu cầu** (một segment, một Chương, hoặc vùng đang chọn), **không chạy nền liên tục**.

FR83: Mỗi phát hiện gồm **loại lỗi · vị trí · giải thích ngắn · đề xuất sửa**. Người dùng chấp nhận hoặc bỏ qua **từng phát hiện một**.

FR84: **Bỏ qua có ghi nhớ:** đánh dấu một phát hiện là *"không phải lỗi"* thì lần quét sau không báo lại **trong cùng Tác phẩm**.

FR85: **Proofreader không được tự sửa văn bản.** Mọi thay đổi phải do người dùng chấp nhận.

FR86: Kết quả proofread hiển thị **ngay tại chỗ trên Editor**, không phải một danh sách rời.

#### C8 — Cầu nối Reviewer: Export / Import / Diff (FR87–FR95, FR121, FR130, FR131)

FR87: Xuất **`.docx` dạng bảng hai cột**: cột trái văn bản gốc, cột phải bản dịch, **đối xứng theo segment**.

FR88: Xuất **`.md` hoặc text thuần**, **bảo lưu hình ảnh và alt-text đã dịch** (FR44) **cùng chú thích ảnh đã dịch** (FR129). Hình ảnh tham chiếu theo kiểu người dùng chọn ở FR130.

FR89: Xuất theo **một Chương, nhiều Chương đã chọn, hoặc cả Tác phẩm**.

FR90: **Nhập lại file `.docx` / `.md`** mà Reviewer đã chỉnh sửa, vào đúng Tác phẩm hiện có.

FR91: **Segment alignment:** hệ thống khớp cấu trúc đoạn giữa file nhập và dữ liệu sẵn có. Segment **không khớp được phải hiện ra cho người dùng nối tay**.

FR92: **Review Mode:** workspace chuyển sang bố cục **hai cửa sổ side-by-side** — trái là bản dịch của người dùng, phải là bản đã nhập từ Reviewer.

FR93: Trong Review Mode, **ẩn văn bản gốc** và dùng thuật toán diff **bôi màu phần thêm / xoá / sửa** giữa hai bản dịch.

FR94: Từ Review Mode, **chấp nhận từng thay đổi** vào bản dịch của mình, hoặc bỏ qua.

FR95: Việc nhập bản review **kích hoạt cơ chế thu hoạch thuật ngữ (FR54) một cách độc lập** — kể cả khi người dùng **không bao giờ mở Review Mode**.

FR121: Xuất **`.docx` một khối, đối xứng theo đoạn** — dành cho việc đăng bài. Bảng hai cột, **một hàng duy nhất cho cả Chương**, **không đường kẻ ngang**; hai ô giữ **đúng số lần xuống đoạn như nhau**. Nghiệm thu: bôi đen cột phải rồi dán sang trình soạn thảo website ra **văn bản liền mạch**, không mảnh vụn bảng biểu. **Không nhập lại được** — màn hình xuất phải nói rõ **ngay lúc chọn định dạng**. **Câu chưa xác nhận không được đánh dấu trong file xuất**; thay vào đó **cảnh báo trước lúc xuất**.

FR130: **Chọn cách xuất hình ảnh: theo link gốc, hoặc theo file ảnh**, chọn cho từng lần xuất. Chỉ chọn được *theo link gốc* khi ảnh **có URL gốc lưu kèm**; khi phạm vi xuất chứa ảnh không có URL, **màn hình xuất phải liệt kê rõ ảnh nào sẽ không có link**, không được im lặng bỏ qua.

FR131: **Xuất khối ghi nguồn** — năm trường: bốn trường xuất xứ của FR128 cộng **tên người dịch** (đặt một lần ở cấu hình toàn cục). Bật/tắt được, áp cho **mọi định dạng xuất** (FR87, FR88, FR121). **Mặc định tắt.**

#### C9 — Dự án & dữ liệu (FR96–FR104)

FR96: Mỗi Tác phẩm lưu thành **một `.atproj` trên đĩa** — **nguồn sự thật**; người dùng copy, sao lưu và di chuyển tự do.

FR97: `.atproj` **tự chứa** mọi thứ thuộc về Tác phẩm: văn bản nguồn, bản dịch, segment, lịch sử phiên bản, Glossary dự án, TM dự án, prompt dự án và hình ảnh. **Copy sang máy khác phải mở được nguyên vẹn.**

FR98: Một **chỉ mục Library trung tâm** phục vụ tìm kiếm xuyên Tác phẩm (FR8). Chỉ mục **phải dựng lại được hoàn toàn từ các `.atproj`** — mất chỉ mục không được làm mất dữ liệu.

FR99: **Quét lại thư mục:** phát hiện `.atproj` mới xuất hiện, `.atproj` đã bị di chuyển hoặc xoá (mục mồ côi), và cập nhật chỉ mục tương ứng.

FR100: **Auto-save định kỳ, không gián đoạn UI.** Không được có gai trễ cảm nhận được khi đang gõ.

FR101: **Versioning:** lưu lịch sử các phiên bản dịch của từng segment; **xem lại và khôi phục được**.

FR102: **Sao lưu bằng cách copy thư mục là đủ.** Không được yêu cầu một thao tác export riêng để có bản sao lưu dùng được.

FR103: Mọi cấu hình tồn tại ở **hai tầng**, tầng Tác phẩm ghi đè tầng Global. Global chứa: cấu hình AI, prompt chung, Glossary toàn cục, TM toàn cục, phím tắt, preset bố cục. Tác phẩm chứa: Glossary riêng, prompt riêng, TM riêng, ngôn ngữ nguồn.

FR104: **Không telemetry.** Ứng dụng không gửi bất kỳ dữ liệu nào ra ngoài, trừ nội dung mà người dùng **chủ động** gửi cho nhà cung cấp AI đã cấu hình.

#### C10 — Phát hành & tin cậy (FR105–FR112)

FR105: Phát hành bản cài cho **macOS và Windows** qua **GitHub Releases**.

FR106: Công bố **checksum SHA-256** cho mọi artifact phát hành.

FR107: **Build công khai qua GitHub Actions**, để bất kỳ ai cũng kiểm chứng được binary khớp với mã nguồn.

FR108: **Hướng dẫn cài đặt có ảnh chụp màn hình** cho cả hai hệ điều hành, xử lý tường minh Gatekeeper trên macOS và SmartScreen trên Windows.

FR109: **Màn hình Attribution trong ứng dụng:** liệt kê mọi nguồn từ điển, giấy phép tương ứng và ghi công đầy đủ.

FR110: Kèm văn bản giấy phép **GPL v3** và toàn bộ giấy phép của các bộ dữ liệu trong bản phát hành.

FR111: Cơ chế cập nhật **chỉ kiểm tra và thông báo** phiên bản mới. **Không tự động tải, không tự động cài.**

FR112: **Chính sách gỡ bỏ dữ liệu:** quy trình gỡ một lớp nguồn khỏi bản phát hành kế tiếp **mà không ảnh hưởng chức năng** — bảo đảm bởi FR36.

### NonFunctional Requirements

**Tổng: 19 NFR.** Phần lớn ngưỡng đến từ **số đo thật của Giai đoạn 0**, không phải phỏng đoán.

#### Hiệu năng

NFR1: Độ trễ **Auto-Lookup đầu-cuối** (từ lúc thả chuột sau khi bôi đen tới lúc kết quả hiển thị ở Panel Lookup): **p95 < 100 ms**. **[A1]** Backend đo được p50 0,022 ms · p95 0,046 ms, payload 679 byte — ~99,95 ms còn lại dành cho vòng IPC Tauri và render frontend.

NFR2: Auto-save không làm gián đoạn thao tác gõ: **không frame nào vượt 50 ms** trong lúc auto-save chạy.

NFR3: Tìm kiếm full-text toàn Library: **p95 < 500 ms** trên thư viện 5.000 Chương. **[A6]** Ngưỡng tạm, hiệu chỉnh ở Giai đoạn 3 (Q4).

NFR4: Khởi động ứng dụng tới lúc Library dùng được: **< 3 giây** trên thư viện 5.000 Chương. **[A7]** Ngưỡng tạm.

NFR5: Bộ nhớ khi nhàn rỗi: **< 300 MB**. **[A8]** Ngưỡng tạm.

#### Dữ liệu & lưu trữ

NFR6: Kích thước bản cài kèm toàn bộ từ điển: trần **400.000.000 byte** *(400 MB thập phân)* cho **payload sản phẩm** (mã + font + dữ liệu từ điển), **không có cơ chế tải thêm sau khi cài**. **[A2]** 🔄 **Trần nâng từ 150–200 MB lên 400.000.000 byte — Ice chốt 2026-08-05 trên số đo thật của Story 1.10:** payload bảy nguồn = **343.991.430 byte**, ĐẠT, dư **56.008.570**. Lời hứa *"không tải thêm sau khi cài"* **GIỮ NGUYÊN** — nó chống đỡ cho NFR7 và NFR12. ⚠️ Dư địa đó **không** dành cho tính năng mới: **HVTĐTD** và **Cổ hán văn** chưa dựng, và VietPhrase một mình đã ăn 160.083.968 byte. *(xem `prd.md` §7.2, "NFR6 sửa lần hai 2026-08-05")* **Bản WebView2 Runtime nhúng trên Windows (`offlineInstaller`, ≈ 127 MB) nằm NGOÀI ngân sách này** — nó là runtime của hệ điều hành, nhúng để giữ lời hứa *cài được khi không có mạng*. *(Ice chốt 2026-08-03; xem `prd.md` §7.2, ghi chú dưới bảng.)* **Hệ quả nghiệm thu: mọi phép đo dung lượng phải tách hai dòng** — payload sản phẩm (đối chiếu với trần) và WebView2 Runtime nhúng (ghi ra, không đối chiếu).

NFR7: Tra cứu khi ngoại tuyến: **100% hoạt động không cần mạng**.

NFR8: Độ chính xác dấu tiếng Việt: chỉ mục tìm kiếm **chính phải phân biệt dấu**; chế độ xoá dấu chỉ tồn tại như chỉ mục **phụ**, **không bao giờ là mặc định**. Chi phí đã biết ~17 MB mỗi chỉ mục.

NFR9: Khả năng mang dữ liệu đi: TM xuất được **TMX**; Glossary và prompt xuất được định dạng văn bản mở; `.atproj` tự chứa và mở được trên máy khác.

NFR10: Toàn vẹn dữ liệu: **mất chỉ mục Library không được làm mất dữ liệu** — chỉ mục dựng lại được hoàn toàn từ `.atproj`.

#### Bảo mật & quyền riêng tư

NFR11: **API key lưu trong keychain / credential manager của hệ điều hành.** Không bao giờ ghi vào file cấu hình, file dự án hay log.

NFR12: **Không telemetry.** Không có luồng dữ liệu ra ngoài nào ngoài hai luồng do người dùng chủ động kích hoạt: lời gọi AI, và tải nội dung ở đường nhập từ URL.

NFR13: **Không tài khoản, không đăng nhập, không đồng bộ đám mây.**

NFR19: **Đường nhập từ URL chỉ ra mạng khi người dùng chủ động bấm.** Không tải nền, không prefetch, không kiểm tra ngầm, **không tải lại ảnh đã có**. **Danh sách domain ứng dụng đã gọi phải xem được trong ứng dụng.**

#### Nền tảng & giấy phép

NFR14: Chạy native trên **macOS và Windows**, **hành vi tương đương trên cả hai**.

NFR15: **Mọi thư viện và crate được dùng phải tương thích GPL v3.** Rà soát tường minh **trước khi** đưa mỗi phụ thuộc mới vào dự án.

NFR16: Ngôn ngữ giao diện v1 **chỉ tiếng Việt**, nhưng **toàn bộ chuỗi giao diện phải nằm ngoài mã nguồn, trong file tài nguyên riêng, ngay từ dòng code đầu tiên**. *(Yêu cầu cắt ngang — áp từ Giai đoạn 1.)*

#### Khả năng tiếp cận & an toàn dữ liệu

NFR17: **Sàn khả năng tiếp cận.** Mọi thao tác làm được **hoàn toàn bằng bàn phím** — nghiệm thu bằng một vòng dịch trọn một Chương **không chạm chuột**. Trạng thái focus **luôn nhìn thấy rõ** ở mọi panel và mọi chế độ. Tương phản văn bản đạt **WCAG AA ở cả hai theme**, kể cả Chế độ đọc và phần bôi màu diff của Review Mode. *(Yêu cầu cắt ngang — áp từ Giai đoạn 1. Hỗ trợ trình đọc màn hình NGOÀI phạm vi v1.)*

NFR18: **Cửa sổ mất dữ liệu tối đa khi ứng dụng sập: ≤ 5 giây công việc.** Auto-save kích hoạt khi ngừng gõ khoảng 2 giây, kèm **trần thời gian 5 giây buộc ghi** dù đang gõ liên tục. Phải đạt **đồng thời** với NFR2.

### Additional Requirements

Trích từ `ARCHITECTURE-SPINE.md` (43 bất biến AD-1…AD-43), `build-sequence.md` và `data-sources.md`. Đây là các yêu cầu kỹ thuật **sinh ra story hoặc sinh ra acceptance criteria**, không phải nền tường thuật.

#### Starter template & khung dự án

- **KHÔNG có starter template bên ngoài nào được chỉ định.** Architecture thay vào đó cố định **cây nguồn tường minh** và **Stack ghim phiên bản** — Epic 1 Story 1 phải dựng scaffold theo đúng cây nguồn đó, không theo template cộng đồng.
- Stack ghim (kiểm chứng 2026-08-02, đều đã rà GPL v3): Rust edition 2024 · `tauri` 2.11.5 · Vue 3.5.40 · TypeScript 5.x · `dockview-vue` 7.0.4 · Vite 8.2.0 · `rusqlite` 0.40.1 (feature `bundled`) · `libsqlite3-sys` 0.38.1 · `jieba-rs` 0.10.3 · `tantivy-stemmers` 0.4.0 · `docx-rs` 0.4.22 · `keyring` 4.1.6 · `reqwest` · `similar` **hoặc** `dissimilar`.
- **Đã loại có lý do, không được đưa vào:** `tauri-plugin-stronghold` (khai tử) · `tauri-plugin-keyring` (phơi API ra JS, vi phạm NFR11) · `tauri-wire` (payload 679 byte không cần) · WAL2 (không phải tính năng đã phát hành) · `LIKE` trên đường nóng tra cứu.
- Cây nguồn bắt buộc: `src-tauri/src/{commands,core/{segment,matching,glossary,tm,dict,library,export,webimport,ai,scope,store},ports}` · `src-tauri/capabilities/` · `src-tauri/resources/dict/` · `src/{modes,panels,layout,commands,tokens,i18n}` · `tools/dict-build/` · `dict-manifest.toml`.

#### Ranh giới kiến trúc phải được cưỡng chế bằng test, không bằng kỷ luật

- **Đúng ba cổng** (`DictionarySource`, `TranslationProvider`, `ProjectStore`), không hơn. Thêm cổng thứ tư là quyết định kiến trúc phải ghi thành AD mới. *(AD-2, AD-40)*
- **Không module nào ngoài `ai/` được phụ thuộc `ai/`** — có **test tự động cưỡng chế** hoặc `ai/` là crate riêng. Chiều ngược lại hợp lệ. *(AD-13 → FR77)*
- **Đúng ba điểm ra mạng** trong toàn ứng dụng: `TranslationProvider`, kiểm tra phiên bản (FR111), `Fetcher` của đường nhập URL. **Không có điểm thứ tư.** CSP Tauri **giữ nguyên, không nới**: cấm mọi origin từ xa — không CDN, không font ngoài, **không ảnh ngoài**. *(AD-15)*
- **AD-41 phải có bộ test riêng** vì framework không cưỡng chế được: từ chối host ngoài hai tầng allowlist · từ chối chuyển hướng ra ngoài · từ chối tài liệu ở tầng 2 · không lời gọi nào khi người dùng không bấm.
- **Test nghiệm thu FR36:** xoá file `.db` của một lớp gỡ rời rồi **chạy lại toàn bộ bộ test tra cứu — phải vẫn xanh**. Với HVTĐTD: bật lớp → từ loại/ví dụ/trích dẫn **tiếng Việt**; xoá file → rơi về nhãn tiếng Anh của lớp nền.
- **Mọi nội dung từ ngoài không bao giờ render thành HTML** — Rust phân tích thành mô hình dữ liệu có cấu trúc, Vue render từ mô hình. **Mô hình nội dung không có nhánh nào mang HTML.** Không byte HTML thô nào đi qua IPC. *(AD-16)*
- **Phạm vi filesystem hai tầng cưỡng chế bởi Tauri capabilities:** scope tĩnh `$RESOURCE/dict/**` (chỉ đọc), `$RESOURCE/fonts/**` (chỉ đọc), `$APPDATA/**` (đọc+ghi); scope động chỉ cấp khi người dùng chọn qua hộp thoại. *(AD-23)*

#### Mô hình dữ liệu & vòng đời

- **`segment.id` bất biến, không bao giờ tái dùng**; thứ tự là cột riêng `ord`. Mọi dữ liệu gắn theo segment tham chiếu `id`, **không bao giờ tham chiếu vị trí**. *(AD-3)*
- **Ranh giới segment tính một lần lúc nhập và lưu xuống `.atproj`; không bao giờ tính lại.** Quy tắc tách câu mới chỉ áp qua thao tác **tái tách chủ động** của người dùng trên từng Chương, kèm cảnh báo dữ liệu sẽ về hưu. *(AD-4)*
- **Gộp/tách segment = về hưu + tạo mới**; segment mới bắt đầu **chưa xác nhận** với lịch sử rỗng. **Không áp cho FR116** (nối câu lúc nhập song ngữ xảy ra trước khi ghi xuống đĩa). Chỗ đánh dấu FR119 trỏ tới segment về hưu **ở lại, không bị xoá im lặng**. *(AD-5)*
- **Gộp/tách Chương giữ nguyên segment** — chỉ đổi `chapter_id` và `ord`; `segment.id`, lịch sử, trạng thái xác nhận **giữ nguyên**. *(AD-32, khác biệt cố ý so với AD-5)*
- **Mục TM khoá theo cặp văn bản `(nguồn, đích)` + metadata**, độc lập hoàn toàn với `segment.id`. Sửa bản dịch của segment đã xác nhận thì **ghi thêm cặp mới**, không sửa cặp cũ. *(AD-6 → FR63, FR64)*
- **Máy trạng thái segment tường minh** *(AD-31)*: auto-save → không đổi trạng thái, **không** tạo `SegmentVersion`; xác nhận → đã xác nhận, **tạo một** version; sửa segment đã xác nhận → **về chưa xác nhận**, không tạo version; điền sẵn TM 100% → chưa xác nhận + nhãn *gợi ý*; chấp nhận thay đổi Review Mode → chưa xác nhận; về hưu do gộp/tách → về hưu. **Cặp TM ghi đúng tại chuyển tiếp sang đã xác nhận, không ở chỗ nào khác.**
- **Xuất xứ FR117 suy ra bằng cách so văn bản đích hiện tại với bản lúc nạp segment, KHÔNG dùng cờ dirty.** *(AD-31 — hợp đồng phụ bắt buộc)*
- **`SEGMENT` mang cờ kết đoạn**, tính lúc nhập cùng lượt với ranh giới câu và lưu xuống. **Một cờ duy nhất dùng chung cho cả nguyên văn và bản dịch.** Ba ca biên phải cài đúng: gộp → theo câu cuối · tách → theo mảnh cuối, các mảnh trước cờ tắt · **segment cuối Chương → cờ tắt, luôn luôn**. *(AD-37 → FR121)*
- **Alt-text và caption là `Segment` mang trường vai (`alt` | `caption`)**, không phải cột trên `ASSET`. `ASSET` mang **neo vị trí riêng** trong Chương, độc lập với việc có hay không có segment đi kèm. Ảnh không có caption **không sinh segment rỗng**. *(AD-42 → FR44, FR129)*
- **Bốn trường xuất xứ FR128 là dữ liệu trên `CHAPTER`**; khối ghi nguồn FR131 **dựng lúc xuất**, không lưu chuỗi đã định dạng ở bất kỳ đâu. *(AD-43)*
- **Vòng đời mục Glossary ba trạng thái một chiều:** ứng viên (bảng chờ riêng) → chờ chốt bản dịch → đã chốt. Trường bản dịch **nullable** cho tới khi chốt. `glossary/` phơi ra **đúng một** truy vấn trả về mục **đủ điều kiện chèn**; `ai/` không có đường nào khác. Âm Hán Việt cho FR113 đọc **qua cổng `DictionarySource`**, không cài lại. *(AD-20, AD-36)*
- **`Work` = UUID v4** lưu trong `meta.json`; `Chapter`, `Segment`, mục Glossary, mục TM = số nguyên **cục bộ**. Indexer phải phát hiện và cảnh báo hai Tác phẩm **trùng UUID**. *(AD-28)*
- **Ghi nhớ proofreader khoá theo `(work, chữ ký phát hiện)`**, không theo `segment.id` — để sống sót qua gộp/tách segment. *(Consistency Conventions → FR84)*

#### Lưu trữ, ghi và di trú

- **Năm loại kho, ranh giới sở hữu cứng:** `dict-core.db` + mỗi `<lớp gỡ rời>.db` (**chỉ đọc, luôn luôn**) · `<Tên>.atproj/` (nguồn sự thật) · `global.db` · `library-index.db` (**dẫn xuất**) · OS keychain. *(AD-7)*
- **`.atproj` là một thư mục** chứa `meta.json` (Library đọc được **không cần mở SQLite**), `project.db`, `assets/` (ảnh là file thật qua asset protocol). Sao lưu = copy thư mục. *(AD-9 → FR102)*
- **`library-index.db` là file riêng, không nằm chung `global.db`**; chỉ component `Indexer` ghi vào, và chỉ ghi **sau khi** `.atproj` đã ghi xong. **Xoá `library-index.db` phải luôn là thao tác an toàn.** *(AD-8 → FR98, NFR10)*
- **`meta.json` là cache dẫn xuất từ `project.db`**, ghi bởi **chính `store::Writer` của Tác phẩm đó**, trong cùng thao tác logic. Không thành phần nào khác ghi vào nó. *(AD-33)*
- **Một writer duy nhất cho mỗi kho ghi được**, đặt sau **hàng đợi nối tiếp**; đọc dùng pool nhiều kết nối song song (WAL). **Không module nào được tự mở kết nối ghi.** *(AD-11 → NFR2)*
- **`PRAGMA journal_mode = WAL`, `wal_autocheckpoint = 0`, `busy_timeout` đặt tường minh.** Luồng nền trên **kết nối riêng** gọi `wal_checkpoint(PASSIVE)` khi ngừng gõ, `TRUNCATE` khi đóng Tác phẩm/thoát. Phải có **ngưỡng kích thước WAL buộc checkpoint**. *(AD-12)*
- **Hợp đồng flush Editor** *(AD-35 → NFR18)*: flush khi (a) ngừng gõ ~2 s; (b) **trần cứng 5 s không được reset bởi phím gõ**; (c) xác nhận segment; (d) rời segment; (e) đóng Tác phẩm/thoát. Flush qua **đúng `store::Writer` nối tiếp**. Một flush chỉ coi là xong **sau khi đã ghi vào WAL**. **Thao tác rời rạc (FR94, FR58) ghi ngay, không đi qua bộ đệm gõ.**
- **Lược đồ có phiên bản; di trú chỉ tiến, không bao giờ lùi.** `meta.json`, `project.db`, `global.db` mỗi cái mang số phiên bản lược đồ. Gặp phiên bản **mới hơn** ứng dụng thì **từ chối mở** và báo rõ, không bao giờ ghi vào. `library-index.db` không di trú — xoá và dựng lại. *(AD-30 → FR97)*

#### Thành phần dùng chung bắt buộc (chống ba lần cài đặt lệch nhau)

- **Một `Matcher` dùng chung** cho FR40 (từ điển), FR51 (Glossary), FR61 (TM). Tiếng Trung: khớp chính xác + n-gram ký tự, tách từ qua `jieba-rs`. Tiếng Anh: stemming rồi token n-gram. *(AD-17)*
- **Một `ScopeResolver`** cho mọi phân giải hai tầng, ngữ nghĩa khai tường minh: Glossary/Prompt/Cấu hình AI/Tên người dịch = **ghi đè**; Translation Memory = **hợp nhất**; Luật làm sạch khi nhập = **hợp nhất** (toàn cục **cộng** Tác phẩm). **Thứ tự sắp xếp kết quả TM hai khoá:** khoá chính = **xuất xứ** (cặp *của tôi* trước), khoá phụ = **tầng** (Tác phẩm trước Global). Xác nhận segment **chỉ** ghi vào TM Tác phẩm. *(AD-18)*
- **Smart RAG Injector là một hàm thuần** nhận (câu nguồn, scope, Glossary, TM) trả về **prompt đã lắp hoàn chỉnh**. **Không nối chuỗi rải rác ở chỗ gọi.** *(AD-14 → FR71)*
- **`CommandRegistry` duy nhất** — handler chuột chỉ được `dispatch` một command đã đăng ký. Command id dùng **khoá chấm có tiền tố miền** (`lookup.search_selection`, `review.accept_change`). *(AD-34 → FR22, NFR17)*
- **Đường nhập là một pipeline có thứ tự cố định, dùng chung cho mọi nguồn**, sống ở `core/segment/` *(AD-39)*:
  `byte thô → giải mã bảng mã (FR126) → bóc nội dung chính (FR123, chỉ web) → làm sạch theo luật (FR124) → chuẩn hoá đoạn & khoảng trắng (FR125) → TÁCH CHƯƠNG theo mẫu phân tách (FR14) → XEM TRƯỚC + sửa tay → tách segment + cờ kết đoạn → ghi xuống .atproj`
  - Điều kiện áp bước tách Chương phát biểu theo **hình dạng đầu vào**, không theo danh sách đường nhập: đầu vào đến thành **một dòng chưa chia Chương** → có tách; đầu vào **đã một đơn vị một Chương** (mỗi link FR122) → không tách.
  - `.docx` **bỏ qua** bước giải mã bảng mã (zip chứa XML đã khai encoding).
  - **Màn xem trước luôn hiển thị kết quả sau toàn bộ chuỗi.** Đổi bảng mã chạy lại chuỗi từ bước một, **trong bộ nhớ, trước khi có bất kỳ segment nào tồn tại**.
- **`Fetcher` và `Extractor` tách rời, không trait hoá** trong `core/webimport/`. `Fetcher` **không bao giờ** phân tích nội dung; `Extractor` **không bao giờ** chạm mạng. *(AD-40)*
- **Allowlist mạng hai tầng, sống đúng một lần nhập** *(AD-41 → NFR19)*: tầng 1 = host của các link vừa dán (tải **tài liệu**); tầng 2 = host của tài nguyên được tham chiếu từ trang tầng 1 trong cùng lần nhập (**chỉ ảnh, không bao giờ tài liệu**). `Fetcher` từ chối host ngoài allowlist **kể cả khi gặp chuyển hướng**. Mọi lời gọi ghi `(thời điểm, domain, tầng, kết quả)` vào nhật ký **xem được trong ứng dụng**. **Không tải lại ảnh đã có, so theo `source_url` trong phạm vi cùng một Tác phẩm** (không phải băm nội dung).
- **Nhập `.docx` phải kiểm hình dạng bảng ở cổng vào, ở Rust, TRƯỚC alignment và trước mọi lệnh ghi** *(AD-38)*: bảng có **đúng một hàng** *và* ô chứa **nhiều hơn một đoạn** → **từ chối** kèm giải thích; không chạy alignment, không ghi gì. Nhận dạng bằng **hình dạng**, không bằng metadata hay tên file.

#### Truy vấn từ điển & chỉ mục

- **Ba nhánh truy vấn tiếng Trung** *(AD-26 → FR39)*: tra chính xác đầu mục → chỉ mục B-tree (0,02 ms, đường nóng) · chuỗi con 1–2 ký tự → bảng đảo ngược **`char_idx`** (0,15–4,5 ms, 1.297.115 cặp, 33,4 MB) · chuỗi con 3+ ký tự → FTS5 `trigram` (0,13–0,19 ms). **`LIKE` bị cấm trên đường nóng** (đo được 20–50 ms).
- **Chỉ mục FTS chính dùng `remove_diacritics 0`** *(AD-27 → NFR8, FR9)*; chỉ mục xoá dấu chỉ là chỉ mục **phụ**, không bao giờ mặc định.
- **Không tồn tại bước hợp nhất nguồn từ điển** — đường tra cứu trả về **kết quả theo từng nguồn**, giữ nguyên bất đồng. Cột `source` **bắt buộc trên mọi bản ghi nghĩa**. *(AD-19 → FR31, FR32)*
- **Mỗi lớp từ điển gỡ rời là một file `.db` độc lập**; runtime **không có mã riêng cho từng nguồn**. Năm nguồn nền sạch gộp trong `dict-core.db`; Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD mỗi nguồn một file riêng. **Mỗi file `.db` tự mang metadata giấy phép và ghi công của chính nó**; màn hình Attribution dựng từ các file **có mặt**. Trường giấy phép phải biểu diễn được **cả giấy phép mở lẫn phép sử dụng riêng do tác giả cấp** — **không được mô hình hoá thành enum các giấy phép mở** (HVTĐTD không thuộc GPL v3). *(AD-10 → FR36, FR38, FR109, FR112)*
- **Dữ liệu từ điển là artifact có phiên bản và checksum** *(AD-25)*: `tools/dict-build` chạy tay sinh các file `.db`, đẩy lên **GitHub Release riêng có phiên bản**; repo chứa mã build tool + `dict-manifest.toml` (URL, SHA-256, phiên bản nguồn thô). CI tải theo manifest, đối chiếu checksum, rồi đóng gói. **Parser định dạng từ điển chỉ nằm trong build tool, không vào bản phát hành.**

#### Bề mặt IPC & i18n

- **Rust không bao giờ trả về văn bản hiển thị.** Mọi lỗi và thông báo qua IPC có hình dạng `{ code, message_key, params, retryable }`; frontend phân giải `message_key` trong file tài nguyên. **Không có chuỗi tiếng Việt nào trong mã Rust hay mã Vue.** *(AD-21 → NFR16)*
- **Streaming AI qua Tauri Channel API** (không dùng event rời), **không dùng client SSE tự kết nối lại**. Đứt luồng là quyết định tường minh của ứng dụng; thử lại **do người dùng chủ động**. Mọi lời gọi AI **huỷ được giữa chừng**. *(AD-22 → FR73, FR74, FR75)*
- **Khoá API chỉ tồn tại trong Rust** — dùng crate `keyring` **trực tiếp**, không dùng `tauri-plugin-keyring`. **Khoá không bao giờ đi qua IPC**; frontend chỉ biết trạng thái *đã cấu hình / chưa cấu hình*. *(AD-29 → FR67, NFR11)*
- **Một cửa sổ OS, ba chế độ.** Library, Workspace, Chế độ đọc là ba **chế độ** trong cùng một cửa sổ; Review Mode là một **bố cục dockview**, không phải cửa sổ thứ hai. *(AD-24 → FR16, FR92)*
- **Ngày giờ lưu ISO-8601 UTC** trong database; định dạng hiển thị chỉ ở frontend.

#### Mũi thăm dò kỹ thuật bắt buộc (mỗi mũi là một story riêng)

| Mũi thăm dò | Khi nào | Đóng cái gì |
|---|---|---|
| ~~**Đo dung lượng thật + rà giấy phép SIL OFL của font nhúng**, và chọn biến thể vùng TC/SC~~ | ✅ **Đã xong 2026-08-03 (Story 1.1)** | Đo thật: font chiếm **21,29 MB**; tổng với database hiện tại **151,29 MB**, dưới trần. SIL OFL 1.1 cả ba, tương thích GPL v3. Biến thể vùng chốt **TC**. *(Ước 30–50 MB của bản này **quá cao** — số thật 25,991 MiB trên đĩa.)* **Rủi ro còn mở:** dư địa dưới trần chỉ còn ~47 MB cho các nguồn từ điển còn lại **cộng toàn bộ mã sản phẩm** — vẫn là thay đổi **tầng PRD** nếu vỡ |
| **Ngưỡng kích thước WAL buộc checkpoint (AD-12) + nhịp flush cụ thể (AD-35)** — đo trên Editor thật, cùng lúc | **Giai đoạn 2** | Đạt NFR18 (≤5 s) mà không phạm NFR2 (không frame >50 ms) |
| **Thư viện editor cho panel Editor** · **cách phân tích khung SSE** (rà GPLv3 trước — `reqwest-sse`, `sseer` **chưa xác nhận giấy phép**) | Giai đoạn 2 | AD-31 và AD-22 đã cố định hợp đồng nên lựa chọn không lan ra ngoài module |
| **Thư viện bóc nội dung** (`dom_smoothie` 0.18.0 MIT, chưa ghim) — bóc thử trên các site thật, đo tỉ lệ bóc sai. **[A12]** | Giai đoạn 3 | Tỉ lệ sai cao **không chặn** — đường sửa tay FR123 là nghiệm thu |
| **Thư viện phát hiện bảng mã** (`chardetng` 1.0.0 + `encoding_rs` 0.8.35, đều tương thích GPLv3) — dò thử trên `.txt` GBK và Big5 thật. **[A13]** | Giai đoạn 3 | Cùng mũi thăm dò với A12 |
| **HTTP client cho `Fetcher`** — theo dõi chuyển hướng để cưỡng chế allowlist, giới hạn kích thước, timeout | Giai đoạn 3 | Nhiều khả năng dùng lại `reqwest` |
| **Đo NFR3/NFR4/NFR5 trên thư viện thật** — đóng Q4 (A6, A7, A8) | Giai đoạn 3 | Ngưỡng tạm đã đủ nghiệm thu, không chặn |
| **Đọc `.docx` bảng hai cột thật** — lấy được số hàng và số đoạn trong từng ô. `docx-rs` **định vị là bộ ghi**, tài liệu nội tuyến chỉ 5,53% | **Trước Giai đoạn 5** | Không đạt thì rà `docx-reader`/`rdocx` — **cả hai chưa xác nhận giấy phép** |
| **`similar` vs `dissimilar`** cho Diff Viewer — thử cả hai trên bản review thật | Giai đoạn 5 | Cả hai tương thích GPLv3 |
| **Hiệu chỉnh ngưỡng bố cục màn hình hẹp trên máy thật** — đóng Q9 (A11) | Từ Giai đoạn 2 (ngay khi Workspace 4 panel xong) | Chỉ **số** hiệu chỉnh; **thứ tự hy sinh panel không đổi** |

#### Trình tự xây dựng (chốt ở `build-sequence.md` — quyền hơn PRD §10)

| Giai đoạn | Nội dung | Nhóm năng lực |
|---|---|---|
| **0** | ✅ Hoàn tất 2026-08-02 — bốn mũi thăm dò | — |
| **1** | Embedded Dictionary (kèm lớp HVTĐTD) + panel Source (kèm tab Hán Việt) + Panel Lookup + Auto-Lookup | C3, một phần C2 |
| **2** | Panel Editor + AI Translation (BYOK/local) + Glossary + Smart RAG Injector | C2, C4, C6 |
| **3** | Library: mô hình dữ liệu, trạng thái vòng đời, tìm kiếm, chế độ đọc, **và toàn bộ đường nhập** — file, dán tay, URL, song ngữ, kèm pipeline làm sạch/chuẩn hoá/bảng mã | C1, C9 |
| **4** | Translation Memory + tái sử dụng segment + xuất TMX | C5 |
| **5** | Export/Import `.docx`/`.md` + segment alignment + Diff Viewer | C8 |
| **6** | AI Proofreader | C7 |
| **7** | Đóng gói, tài liệu cài đặt, attribution, phát hành | C10 |

> **Khác biệt so với PRD §10:** `build-sequence.md` dồn **toàn bộ đường nhập** (bao gồm FR122–FR128) vào Giai đoạn 3. Đây là bản chốt.
>
> **Hai yêu cầu cắt ngang áp từ Giai đoạn 1, không được để lại sau:** NFR16 (chuỗi giao diện ngoài mã nguồn) và NFR17 (sàn khả năng tiếp cận).

#### Nghĩa vụ ngoài mã nguồn

- Ghi công đầy đủ từng nguồn theo yêu cầu CC-BY-SA, giữ share-alike cho dữ liệu phái sinh.
- Ghi phép dùng HVTĐTD vào `LICENSE`/`NOTICE`: © Đặng Thế Kiệt, **không thuộc GPL v3**.
- **Sau khi hoàn thành: thông báo cho tác giả Đặng Thế Kiệt** — đề nghị tường minh trong thư đồng ý. Không mang số FR nhưng là **điều kiện của phép sử dụng**.

### UX Design Requirements

Trích từ cặp spine `DESIGN.md` (visual identity + design tokens) và `EXPERIENCE.md` (IA, hành vi, trạng thái, tương tác, a11y, hành trình). **29 mockup HTML đã kiểm toán 2026-08-03** là minh hoạ, không phải nguồn sự thật — khi mâu thuẫn, hai file spine thắng.

#### Design tokens

UX-DR1: **Cài đặt bộ token màu hai theme, đúng 16 token mỗi theme** *(sửa 2026-08-03: bản trước ghi 17, không khớp danh sách liệt kê ngay bên dưới — bảng token đầy đủ nay ở `DESIGN.md § Bảng token màu`).* Sáng: `background #f4f1ea` · `surface #fbfaf6` · `surface-sunken #f0ece1` · `surface-accent #e6eeee` · `surface-tm #faf6ee` · `on-surface #2b2723` · `on-surface-variant #6b6459` · `outline #e2dccf` · `outline-faint #efeade` · `ornament #a9a196` · `primary #2f5d63` · `on-primary #fbfaf6` · `confirmed #5a6b3f` · `tm-rule #b99a5e` · `tm-text #7a5d25` · `error #8f2f22`. Tối: `background #201e1b` · `surface #26241f` · `surface-sunken #1b1a17` · `surface-accent #2c3a3b` · `surface-tm #302b21` · `on-surface #e8e3d8` · `on-surface-variant #a29a8c` · `outline #3b382f` · `outline-faint #302d26` · `ornament #6a6459` · `primary #7fb3ba` · `on-primary #1b1a17` · `confirmed #9cb37a` · `tm-rule #b99a5e` · `tm-text #d3b276` · `error #e5867a`. Sống ở `src/tokens/`, **cấm giá trị màu viết thẳng trong component** (AD-34).

UX-DR2: **Cài đặt 14 token typography**: `read-lg` (19px/1.95/0.004em) · `read-md` (17.5px/1.8) · `read-sm` (16px/1.66) · `read-title` (23px/600/1.3) · `source-cjk` (16.5px/2.05) · `source-hanviet` (12.5px/italic/1.95) · `editor` (15px/1.95) · `lookup-headword` (24px/1.3) · `lookup-gloss` (14.5px/1.6) · `lookup-example` (12.5px/italic/1.6) · `ui-md` (12px/1.5) · `ui-sm` (11.5px/1.5) · `ui-label` (10px/700/1.4/0.1em) · `ui-mono` (10.5px/1.4). Bốn họ chữ: `read` · `read-cjk` · `ui` · `mono`.

UX-DR3: **Cài đặt token khoảng cách và hình dạng.** Đơn vị 4px; `panel-inline 16px` · `panel-block 12px` · `head-height 34px` · `titlebar-height 38px` · `status-height 32px` · `gutter-width 22px` · `read-measure-lg 62ch` · `read-measure-md 68ch` · `read-measure-sm 76ch`. Bo góc: `none 0` · `sm 2px` · mặc định `3px` · `md 4px` · `window 9px` · `full 9999px`.

UX-DR4: **Nhúng ba font vào bản cài, không dùng font hệ điều hành, không CDN** — `Source Serif 4` · `Source Han Serif` (chỉ Regular) · `Source Sans 3`. Lý do là NFR14: font hệ thống ra PingFang trên macOS và YaHei trên Windows = **hai sản phẩm khác nhau**. Kèm chuỗi dự phòng để render được khi máy chưa cài font.

#### Sàn tương phản & khả năng tiếp cận

UX-DR5: **Ba màu đã bị loại vì trượt WCAG AA — không được khôi phục:** `#7d766c` làm chữ phụ (4,09:1) → thay bằng `on-surface-variant #6b6459` (5,2:1); `#a9a196` làm chữ (2,5:1) → đổi vai thành `ornament`, **chỉ dùng cho nét không phải chữ**; `#b99a5e` làm chữ (2,5:1) → `tm-text #7a5d25` (5,2:1), `tm-rule` giữ nguyên vì là vạch. **Quy tắc:** `ornament` và `tm-rule` là **màu của nét, không bao giờ là màu của chữ**. Ngoại lệ duy nhất đã đặc tả: ký tự ranh giới câu `⏐`.

UX-DR6: **`opacity` ở trạng thái nghỉ chỉ áp cho nét và nền, không áp cho chữ.** Lùi một khối chữ ra sau = **đổi token màu**, không giảm độ đục. `opacity` được phép giữa 0 và 1 (ẩn/hiện affordance) và trong quá độ 0.4→1 trong 90ms, **không dừng ở mức trung gian làm chữ mờ thường trực**.

UX-DR7: **Mỗi chế độ và mỗi panel khai báo điểm vào focus.** Chuyển panel trong dockview phải **dời focus DOM tường minh**; không chế độ nào được để focus rơi về `body`.

UX-DR8: **Tiêu điểm luôn nhìn thấy** — vạch dọc 2px `primary` ở mép trái panel + tiêu đề chuyển `primary` in đậm. **Không dùng viền bao quanh để báo tiêu điểm.**

UX-DR9: **Nghiệm thu NFR17 bằng một kịch bản cụ thể:** dịch trọn một Chương từ đầu tới cuối — mở từ Library, tra cứu, gọi AI, đưa sang, sửa, gộp một câu, xác nhận, sang Chương kế — **không chạm chuột một lần nào**.

#### Typography

UX-DR10: **Giãn dòng 1.66 là sàn cứng cho chữ nội dung họ `read`** — dấu tiếng Việt chồng cả trên lẫn dưới (`ế ộ ữ ẳ ườ`), 1.5 làm `ườ` dòng trên chạm `ộ` dòng dưới. Mọi lần kiểm bằng mắt **phải dùng chuỗi dày dấu**.

UX-DR11: **Họ `ui` được phép ở 1.4 và 1.5 cho nhãn một dòng, nhưng quay lại 1.66 khi chuỗi có khả năng xuống dòng** (mô tả dưới ô thiết lập, câu trạng thái, hộp giải thích). Phép thử: *chuỗi này có bao giờ dài quá một dòng không?*

UX-DR12: **Phân vai hai họ chữ tuyệt đối, không nhoè:** `read` (có chân) cho mọi thứ là **nội dung** — nguyên văn, Hán Việt, bản dịch trong Editor, mục từ và nghĩa trong Panel Lookup, toàn bộ Chế độ đọc. `ui` (không chân) cho mọi thứ là **bộ máy** — tiêu đề panel, nhãn trạng thái, thanh trạng thái, phím tắt.

#### Bố cục

UX-DR13: **Workspace là lưới 2×2 mặc định** — hàng trên `Nguyên văn | Bản dịch`, hàng dưới `Tra cứu | Đề xuất AI`. Nguyên văn và Bản dịch **cạnh nhau theo chiều ngang** vì đối chiếu ngang là thao tác lặp hàng trăm lần mỗi Chương. Preset thay thế: **4 cột** (`Nguyên văn | Tra cứu | Đề xuất AI | Bản dịch`) và **Review Mode** (`Bản dịch của tôi | Bản Reviewer đã sửa`).

UX-DR14: **Phân tách panel đảo ngược giữa hai theme — đừng thống nhất về một cách làm.** Sáng: đường kẻ 1px `outline`. Tối: **khe 2px để `background` lộ ra**, panel bo `3px` — vì `outline #3b382f` trên `surface #26241f` chỉ đạt **1,39:1**, gần như vô hình. Nguyên tắc: *mặt sáng phân tách bằng nét, mặt tối phân tách bằng khe*.

UX-DR15: **Bốn ngưỡng bố cục màn hình hẹp, đo theo vùng làm việc** (chiều cao cửa sổ trừ titlebar 38px và status 32px), **không theo kích thước màn hình**: **≥ 1100×820** giữ 2×2 · **< 820 cao** gộp hàng dưới thành một panel có tab · **< 1100 rộng hoặc < 700 cao** chỉ còn `Nguyên văn | Bản dịch`, Tra cứu rút về ngăn kéo · **< 860 rộng** báo không hỗ trợ. **Thứ tự hy sinh là quyết định, không hiệu chỉnh:** Đề xuất AI nhường trước · Tra cứu nhường sau nhưng **rút về thanh trạng thái, không bao giờ mất hẳn** · cặp `Nguyên văn | Bản dịch` **không bao giờ nhường**. **[A11] · Q9**

UX-DR16: **Không có elevation** — không bóng đổ, không lớp nổi, không z-index trang trí. Chiều sâu duy nhất là **sắc độ** (`surface-sunken` cho vùng lùi). Ngoại lệ duy nhất: bóng của cửa sổ ứng dụng do OS vẽ.

#### Component patterns

UX-DR17: **Panel** — thanh tiêu đề 34px, tiêu đề `ui-md` màu `on-surface-variant`, tab bên phải. Panel có tiêu điểm: vạch dọc 2px `primary` mép trái + tiêu đề `primary` in đậm.

UX-DR18: **Bản ghi từ điển** — vạch trái 2px, thụt 13px. Nhãn nguồn `ui-label` màu `primary`; từ loại `read` in nghiêng `on-surface-variant`; nghĩa `lookup-gloss`; ví dụ in nghiêng; **trích dẫn có vạch trái `primary` để phân biệt với ví dụ**. **Nhiều nguồn xếp chồng dọc, mỗi nguồn một khối — không bao giờ gộp.** Khi hai nguồn ghi khác nhau, **một dòng dẫn nói rõ điều đó trước khi liệt kê**.

UX-DR19: **Vạch lề segment** — vạch dọc 2px trong máng rộng 22px bên trái Editor, cao đúng bằng câu tương ứng. **Đây là cách duy nhất trạng thái segment được hiển thị**; văn bản không bị chia khối, **không ô, không bảng**. Năm giá trị: `confirmed` đã xác nhận · `primary` đang sửa · `tm-rule` điền sẵn từ TM khớp 100% chưa xác nhận · **không vạch** chưa dịch · `ornament` mờ đã về hưu.

UX-DR20: **Ranh giới câu** — ký tự `⏐` màu `ornament`, `opacity: 0` mặc định, hiện ở `0.55` khi rê chuột hoặc con trỏ chạm.

UX-DR21: **Dải mọc dưới câu đang sửa — mẫu chung thay cho hộp thoại**, dùng cho ba thứ: chốt bản dịch Glossary lần đầu gặp (FR114), phát hiện Proofreader (FR83), gợi ý TM khớp mờ (FR59). Dải **đẩy văn bản xuống chứ không phủ lên**, thu lại ngay khi xong. **Chỉ một dải mọc tại một thời điểm**, thứ tự ưu tiên: (1) **Chốt Glossary** — chặn thật, hỏi một lần trong cả Tác phẩm; (2) **Proofreader** — chờ quyết định về chính câu này, bỏ qua thì tích lại; (3) **Gợi ý TM** — nhường cả hai. Xử lý xong dải trên thì dải dưới **mọc ngay tại chỗ vừa thu**, vị trí không nhảy vì cả ba cùng chiều cao đầu mục.

UX-DR22: **Phát hiện Proofreader hiển thị bằng gạch chân lượn sóng dưới đúng cụm chữ**, ở `text-underline-offset: 4px` để không chạm dấu nằm dưới của `ạ ộ ợ` — **không dùng vạch lề** (vạch lề đã dùng hết năm giá trị cho trạng thái segment). Hai màu, không thêm màu mới: **`error`** cho chính tả/ngữ pháp (FR80, có đáp án đúng), **`tm-rule`** cho nghi về nghĩa (FR81, là phán đoán).

UX-DR23: **Bảng chờ Glossary** — danh sách ứng viên xếp theo tần suất, mỗi dòng có số lần xuất hiện và một ví dụ ngữ cảnh, hiện **bản dịch đề xuất** khi có (FR113). Duyệt và bỏ bằng **một phím, không gõ**. Không mục nào rời bảng chờ sang Glossary mà không qua thao tác người dùng.

UX-DR24: **Khối nội dung — đơn vị của mọi thao tác trong màn xem trước nhập.** Nội dung chia khối theo đoạn, mỗi khối mang đúng một trong ba trạng thái đọc ở vạch lề: `confirmed` (giữ lại, người dùng đã chạm) · `tm-rule` (giữ lại, **máy đoán chưa ai xác nhận**) · `ornament` mờ (đã loại — khối chìm xuống `surface-sunken`, chữ rút về `on-surface-variant` **và đổi từ `source-cjk` sang chữ giao diện cỡ nhỏ**). **Phân biệt bằng độ lùi, không bằng màu nhấn thứ hai.**

#### Motion

UX-DR25: **Chín quy tắc chuyển động cho Auto-Lookup** (chạy hàng trăm lần mỗi Chương dưới 100ms đầu-cuối): vùng đầu mục Panel Lookup **cao cố định** · nội dung mới vào **90ms opacity 0.4→1 ease-out, không `translate`, không `scale`** · nội dung cũ **thay thẳng không hiệu ứng ra** · tra liên tiếp **huỷ hiệu ứng đang chạy, không xếp hàng** · vị trí cuộn **về đầu tức thì, không bao giờ cuộn có hiệu ứng** · bôi đen đang kéo **chỉ tra khi vùng chọn đã dừng** · vượt 250ms **vạch tiến trình mảnh ở đáy vùng đầu mục, không spinner** · không tìm thấy **cùng 90ms** · `prefers-reduced-motion` **bỏ toàn bộ hiệu ứng, đổi tức thì** (thuộc sàn a11y, không phải tuỳ chọn). **Ba cấm chung toàn ứng dụng:** không `translate` trong thao tác lặp · không hiệu ứng nào vượt 150ms trên đường nóng · không hiệu ứng xếp hàng.

#### Trạng thái

UX-DR26: **Trạng thái tra cứu ba giá trị** — có kết quả · không có kết quả · nhiều nguồn bất đồng. **Không có trạng thái "đang tải"**: tra cứu chạy dưới 100ms nên spinner ở đây là tiếng ồn.

UX-DR27: **Trạng thái AI năm giá trị** — chưa cấu hình · đang sinh (chảy dần) · xong · lỗi · đã huỷ. **Chưa cấu hình KHÔNG phải trạng thái lỗi** — panel chỉ mời cấu hình, không cảnh báo.

UX-DR28: **Trạng thái bảng mã ba giá trị** — nguồn tự khai (`.docx`, HTTP có `charset` tin được) · tự đoán tin cậy cao · tự đoán tin cậy thấp. **Chỉ giá trị thứ ba mở dải đối chiếu năm ứng viên.** Không có trạng thái lỗi.

UX-DR29: **Trạng thái một Chương trong lần nhập nhiều link: sạch hoặc cần xem.** *Cần xem* gom bốn nguyên nhân: bảng mã tin cậy thấp · phần bóc ra **ngắn bất thường so với trung vị** các Chương khác · luật làm sạch xoá quá nhiều · link hỏng. **Bộ đếm ở đầu màn xem trước luôn hiện cả hai con số.**

UX-DR30: **"Đã lưu N giây trước" ở thanh trạng thái** — không hộp thoại, **không dấu chấm "chưa lưu" gây lo lắng** (NFR18 bảo đảm mất tối đa 5 giây).

UX-DR31: **Bốn trạng thái rỗng có nội dung riêng biệt**: Lookup **không có kết quả** khác Lookup **chưa tra gì** (hai câu, hai gợi ý — không tìm thấy thì gợi ý tra từng chữ và trỏ sang Concordance; chưa tra thì dạy thao tác bôi đen) · **Library lần đầu** giải thích Tác phẩm là gì và là một thư mục mang đi được · **TM trống** giải thích **cơ chế** (TM tự đầy khi xác nhận câu — không có nút "thêm vào TM") · **chưa cấu hình AI** mời cấu hình và nói rõ mọi thứ khác vẫn chạy đầy đủ.

#### Tương tác

UX-DR32: **Gộp ngầm — gõ đè lên đúng vị trí ranh giới *là* ra lệnh gộp.** Thực hiện đúng ngữ nghĩa AD-5, một dòng báo ở lề, hoàn tác bằng `⌘Z`. **Không chặn, không hỏi lại.** Phím tường minh: `⌘M` gộp, `⌘/` tách — cả hai là command đăng ký, không phải hệ quả phụ của việc gõ.

UX-DR33: **Sửa ranh giới bóc trong màn xem trước — bàn phím là đường chính, chuột là đường thứ hai.** Mô hình thao tác **đi theo khối, không theo con trỏ ký tự**: `J`/`K` hoặc `↑`/`↓` đi giữa các khối · `Space` bật/tắt giữ khối · `[`/`]` đặt đầu và cuối vùng giữ · `E` mở bộ chọn bảng mã · `R` bật/tắt luật làm sạch khớp khối này · `⌥←`/`⌥→` Chương trước/sau · `⌥W` chỉ xem các Chương **cần xem** · `⌘↵` xác nhận nhập.

UX-DR34: **Chuyển ba chế độ bằng `⌘1` `⌘2` `⌘3`** hoặc tab ở thanh tiêu đề. **Chuyển chế độ luôn giữ ngữ cảnh** — rời Workspace sang Chế độ đọc rồi quay lại thì vẫn đúng Chương, đúng câu, đúng vị trí cuộn.

UX-DR35: **Đưa bản dịch AI sang Editor bằng `⌘⇧↵`, luôn do người dùng chủ động.** **Không có đường nào để kết quả AI tự chảy vào Bản dịch.**

#### Màn xem trước nhập — một màn hình, ba tầng

UX-DR36: **Ba tầng trên một màn hình, xếp dọc theo đúng thứ tự chúng phụ thuộc nhau:** (1) **Bảng mã** FR126 · (2) **Ranh giới nội dung** FR123 · (3) **Luật làm sạch** FR124. Bảng mã đứng trên cùng **vì bảng mã sai thì mọi thứ dưới nó vô nghĩa**. FR125 không có tầng riêng — chạy ngầm, kết quả là thứ ba tầng đang hiển thị. **Không có nút "Tiếp theo" giữa các tầng**: đổi bảng mã ở tầng 1 thì tầng 2 và 3 dựng lại **ngay, trong bộ nhớ**.

UX-DR37: **Dải đối chiếu năm bảng mã với bản dựng thật.** Khi độ tin cậy dò thấp, mở dải **ngay trên văn bản** với năm ứng viên (UTF-8 · GB18030 · GBK · Big5 · UTF-16), mỗi ứng viên kèm **bản dựng thật của cùng đoạn 6–8 ký tự đầu Chương**. Mẫu chữ đặt ở **cỡ `read` chứ không cỡ giao diện** — phải đủ lớn để phân biệt nét chữ Hán. **Không dùng hộp thoại cảnh báo** — biến một phán đoán kỹ thuật thành một **câu hỏi thị giác** trả lời được ngay.

UX-DR38: **Ranh giới cứng của FR122 phải đếm được.** Dưới ô dán link, hai con số đứng cạnh nhau: **`N` link · sẽ tạo `N` Chương**, kèm câu *"Chỉ tải đúng N link này. Không tìm thêm link nào khác."* **Hai con số bằng nhau là bằng chứng đọc được** rằng ứng dụng không tự đi tìm gì.

UX-DR39: **Bộ lọc "cần xem".** Đầu màn hình luôn hiện **hai con số** (*`N` Chương cần xem* · *`M` Chương sạch*), `⌥W` lọc về nhóm đầu. Không có nó, dán 50 link thì tới lần thứ mười người dùng sẽ **bấm xác nhận mù**.

UX-DR40: **Luật làm sạch hiện thứ sắp bị xoá trước khi xoá** — gạch ngang **tại chỗ trong văn bản** bằng nét `ornament`, kèm nhãn luật đã khớp. Danh sách luật ở tầng 3 cho bật/tắt từng luật và ghi **hai con số**: khớp bao nhiêu chỗ *trong Chương này* và bao nhiêu chỗ *trong cả lần nhập*. Mỗi luật mang **nhãn tầng** (Toàn cục / Tác phẩm) và **cả hai tầng cùng áp**.

UX-DR41: **Khối xuất xứ tài liệu ở đầu mỗi Chương trong màn xem trước**, tự điền từ trang. Trường không tìm thấy hiện **chữ nghiêng *"không tìm thấy"*** thay vì để trống — để người dùng biết hệ thống **đã tìm** chứ không phải quên. Với Tác phẩm nhập từ file hay dán tay, cùng khối đó mở được từ danh sách Chương.

UX-DR42: **Nhật ký domain ở hai chỗ** — một dòng tóm tắt ở chân màn xem trước (*"Đã gọi `N` domain · xem"*) và bảng đầy đủ trong **Cài đặt › Quyền riêng tư**. Hai tầng allowlist phân biệt bằng **nhãn chữ, không bằng màu**: `Tài liệu` / `Ảnh`. **Mỗi hàng ghi vì sao được phép.**

#### Chế độ đọc

UX-DR43: **Ba mức đọc với bound đã chốt** — Thoáng (62ch / 19px / 1.95) · **Cân, mặc định** (68ch / 17,5px / 1.8) · Đặc (76ch / 16px / 1.66). **Chiều rộng đo bằng `ch` chứ không `px`** để số ký tự mỗi dòng giữ nguyên khi đổi cỡ chữ. Điều khiển hai tầng: **ba preset trên thanh công cụ**, thanh trượt cỡ chữ và giãn dòng chi tiết **sau một lần bấm**.

UX-DR44: **Công tắc song ngữ đặt nguyên văn ở lề trái**, cỡ nhỏ, màu `on-surface-variant` — **không chen giữa dòng đọc**, vì một khối chữ Hán trên mỗi đoạn làm gãy nhịp đọc tiếng Việt.

UX-DR45: **Chú thích hiện dưới ảnh là `caption` đã dịch (FR129), KHÔNG phải alt-text.** Alt-text (FR44) cũng được dịch nhưng **không hiện trên trang**. **Ảnh không có caption thì không chừa chỗ trống** dưới ảnh.

UX-DR46: **Câu chưa xác nhận trong Chương đã xong hiện với gạch chấm nhẹ.** Affordance đánh dấu **ẩn hoàn toàn**, chỉ hiện khi con trỏ chuột hoặc tiêu điểm bàn phím chạm câu. Phím: `M` đánh dấu rồi đọc tiếp · `↵` nhảy sang Workspace · `B` song ngữ · `1 2 3` ba mức chữ · `⌘,` tinh chỉnh · `D` sáng/tối · `⌘L` mục lục · `M` danh sách đánh dấu.

#### Giọng văn

UX-DR47: **Năm quy tắc giọng văn, viết cho một người dịch chuyên nghiệp** — nói việc không nói cảm xúc (*"Đã gộp hai câu."* chứ không *"Tuyệt vời! Đã gộp xong 🎉"*) · nêu hệ quả không chỉ nêu sự kiện (*"Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được."*) · không đổ lỗi người dùng (*"Nhà cung cấp không phản hồi"* chứ không *"Bạn đã nhập sai khoá"*) · số liệu là số liệu (*"412 token · ước tính ~0,004 USD"*) · **không xưng "chúng tôi", không gọi người dùng là "bạn" trong thông báo trạng thái** — câu trạng thái viết ở dạng vô nhân xưng.

#### Ngoại lệ đã đặc tả

UX-DR48: **Khối xem trước `.docx` trong màn hình xuất dùng màu viết cứng có chủ ý** (`#fff`, `#1e1c19`, `#f2efe8`…) vì nó **mô phỏng một tài liệu Word**, không phải bề mặt của ứng dụng. Các màu này đã kiểm đạt AA. **Đừng "sửa" chúng về token.**

### FR Coverage Map

Mỗi FR trong dãy FR1–FR132 ánh xạ về **đúng một epic chủ trì** — epic chịu trách nhiệm chính về nghiệm thu của nó. Không FR nào bị bỏ.

> **Bốn FR có nghiệm thu chia hai epic, đánh dấu ⇄ trong bảng.** *(Sửa 2026-08-03: bản trước tuyên bố "không FR nào có hai chủ", trong khi chính các ô ghi chú của bảng mô tả việc chia đôi. Câu tuyên bố sai làm bảng mất giá trị như công cụ kiểm tra — người tin nó sẽ đóng FR70 khi Epic 4 xong và không bao giờ quay lại Epic 7.)*
>
> Việc chia đôi là **quyết định đúng, không phải thiếu sót**: không thể nghiệm thu nửa Translation Memory của FR70 trước khi Translation Memory tồn tại. Điều bắt buộc là **cả hai nửa đều có chủ**, và epic thứ hai phải ghi FR đó vào acceptance criteria của mình.

| FR | Epic | Ghi chú |
|---|---|---|
| FR1 | Epic 5 | Hai tầng Tác phẩm → Chương |
| FR2 | Epic 5 | Tài liệu đơn lẻ = Tác phẩm một Chương |
| FR3 | Epic 5 | Metadata Tác phẩm |
| FR4 | Epic 5 | Glossary/TM gắn tầng Tác phẩm |
| FR5 | Epic 5 | Bốn trạng thái vòng đời |
| FR6 | Epic 5 | Suy ra tự động + ghi đè tay |
| FR7 | Epic 5 | Tiến độ Tác phẩm |
| FR8 | Epic 5 | Full-text search xuyên Library |
| FR9 | Epic 5 | Hai chế độ dấu |
| FR10 | Epic 5 | Lọc và sắp xếp |
| FR11 | Epic 5 | Chế độ đọc |
| FR12 | Epic 5 | Mở Chương → Workspace đúng vị trí |
| FR13 ⇄ | Epic 1 ⇄ Epic 6 | Đường vào văn bản tối thiểu (dán tay + `.txt`/`.md`); **nhánh `.docx` đóng ở Epic 6** |
| FR14 | Epic 6 | Nhập hàng loạt + mẫu phân tách + xem trước |
| FR15 | Epic 5 | Đổi tên, sắp xếp, gộp/tách Chương (AD-32) |
| FR16 | Epic 1 | Khung bốn panel một cửa sổ |
| FR17 | Epic 1 | Dock/undock/tab/ẩn hoàn toàn |
| FR18 | Epic 1 | Lưu và khôi phục preset bố cục |
| FR19 | Epic 1 | Panel Source + tab Hán Việt |
| FR20 | Epic 2 | Sync Scrolling — cần đủ ba panel có nội dung |
| FR21 | Epic 1 | Auto-Lookup |
| FR22 | Epic 1 | Global Hotkeys + `CommandRegistry` (AD-34) |
| FR23 | Epic 2 | Tách segment cấp câu |
| FR24 | Epic 2 | Xác nhận từng segment |
| FR25 | Epic 2 | Điều hướng segment |
| FR26 | Epic 2 | Chuyển Chương trong Workspace |
| FR27 | Epic 1 | Từ điển nhúng, 100% offline |
| FR28 | Epic 1 | Bản ghi có cấu trúc |
| FR29 | Epic 1 | Nhiều từ loại = nhiều mục |
| FR30 | Epic 1 | Ví dụ theo từ loại; trích dẫn là trường riêng |
| FR31 | Epic 1 | Mọi định nghĩa hiển thị nguồn |
| FR32 | Epic 1 | Bất đồng hiển thị đồng thời |
| FR33 | Epic 1 | Tab Hán Việt |
| FR34 | Epic 1 | Mục từ tiếng Anh — **dữ liệu: Story 1.10b · tra cứu: Story 1.11b** *(thêm 2026-08-05 qua `correct-course`)* |
| FR35 | Epic 1 | Mục từ tiếng Trung + nhãn ngoại ngữ |
| FR36 | Epic 1 | Lớp nền + lớp gỡ rời (nghiệm thu bằng test xoá file) |
| FR37 | Epic 1 | Bật/tắt từng nguồn |
| FR38 | Epic 1 | Ghi công từng nguồn |
| FR39 | Epic 1 | Truy vấn 1/2/3+ ký tự đều trả kết quả |
| FR40 | Epic 1 | Stemming tiếng Anh (`Matcher` dùng chung) |
| FR41 | Epic 1 | Lịch sử tra cứu + ghim |
| FR42 | Epic 6 | Ảnh trong Panel Source — cần `ASSET` từ đường nhập |
| FR43 | Epic 6 | Ảnh trong Chế độ đọc — cần `ASSET` |
| FR44 ⇄ | Epic 6 ⇄ Epic 7 | Alt-text là Segment vai `alt` (AD-42). *Phần cấu trúc ở 6.13; phần nghiệm thu TM ở 7.1* |
| FR45 | Epic 6 | Ảnh lưu trong `.atproj/assets/` |
| FR46 | Epic 3 | Glossary hai tầng |
| FR47 | Epic 3 | Trường của một mục Glossary |
| FR48 | Epic 3 | Thêm nhanh từ bất kỳ panel nào |
| FR49 | Epic 3 | Quản lý + xuất/nhập CSV/TSV |
| FR50 | Epic 3 | Đánh dấu thuật ngữ trong Panel Source |
| FR51 | Epic 3 | Khớp thuật ngữ theo ngôn ngữ |
| FR52 | Epic 3 | Quét ứng viên khi nhập tài liệu |
| FR53 | Epic 3 | Duyệt hàng loạt một phím |
| FR54 | Epic 8 | **Thu hoạch từ bản review — đầu vào chỉ tồn tại ở Epic 8** |
| FR55 | Epic 3 | Không cơ chế nào tự ghi vào Glossary |
| FR56 | Epic 7 | Ghi TM tự động khi xác nhận |
| FR57 | Epic 7 | TM phạm vi kép |
| FR58 | Epic 7 | Khớp tuyệt đối, điền sẵn nhưng chưa xác nhận |
| FR59 | Epic 7 | Khớp mờ + phần trăm + diff |
| FR60 | Epic 7 | Concordance vào Panel Lookup |
| FR61 | Epic 7 | Thuật toán khớp theo ngôn ngữ |
| FR62 | Epic 7 | Quản lý TM + lọc theo xuất xứ |
| FR63 | Epic 7 | Nhiều bản dịch giữ tất cả |
| FR64 | Epic 7 | Xuất/nhập TMX |
| FR65 | Epic 4 | BYOK |
| FR66 | Epic 4 | Local LLM cùng đường cấu hình |
| FR67 | Epic 4 | API key trong keychain (AD-29) |
| FR68 | Epic 4 | Cấu hình AI hai tầng |
| FR69 | Epic 4 | Custom prompt theo thể loại |
| FR70 ⇄ | Epic 4 ⇄ Epic 7 | Smart RAG Injector — **nửa TM đóng ở Epic 7** |
| FR71 | Epic 4 | Xem prompt cuối cùng đã gửi |
| FR72 | Epic 4 | Kết quả không tự ghi vào Editor |
| FR73 | Epic 4 | Dịch từng segment và theo lô, huỷ được |
| FR74 | Epic 4 | Streaming qua Channel |
| FR75 | Epic 4 | Lỗi không mất việc, không tự thử lại |
| FR76 | Epic 4 | Token và ước tính chi phí |
| FR77 | Epic 4 | Chạy đầy đủ khi không cấu hình AI (test cưỡng chế AD-13) |
| FR78 | Epic 2 | Gộp/tách segment (AD-5) |
| FR79 | Epic 4 | **Xuất/nhập bộ prompt — bộ prompt chỉ tồn tại từ FR69, nên FR79 không thể đứng trước nó** |
| FR80 | Epic 9 | Chính tả và ngữ pháp tiếng Việt |
| FR81 | Epic 9 | Đối chiếu với bản gốc; nghiệm thu bằng tỷ lệ báo động giả |
| FR82 | Epic 9 | Chạy theo yêu cầu, không chạy nền |
| FR83 | Epic 9 | Hình dạng một phát hiện |
| FR84 | Epic 9 | Bỏ qua có ghi nhớ |
| FR85 | Epic 9 | Không tự sửa văn bản |
| FR86 | Epic 9 | Hiển thị tại chỗ trên Editor |
| FR87 | Epic 8 | `.docx` hai cột theo segment |
| FR88 | Epic 8 | `.md` / text thuần + ảnh + alt-text + caption |
| FR89 | Epic 8 | Phạm vi xuất |
| FR90 | Epic 8 | Nhập lại file reviewer |
| FR91 | Epic 8 | Segment alignment — máy khớp, người sửa |
| FR92 | Epic 8 | Review Mode side-by-side |
| FR93 | Epic 8 | Diff bôi màu, ẩn văn bản gốc |
| FR94 | Epic 8 | Chấp nhận từng thay đổi |
| FR95 | Epic 8 | Thu hoạch chạy độc lập với Review Mode |
| FR96 | Epic 1 | `.atproj` là nguồn sự thật (hình dạng cố định từ story đầu) |
| FR97 | Epic 1 | `.atproj` tự chứa (nội dung lớn dần qua các epic, AD-30) |
| FR98 | Epic 5 | Chỉ mục Library dựng lại được |
| FR99 | Epic 5 | Quét lại thư mục, mục mồ côi |
| FR100 | Epic 2 | Auto-save không gián đoạn (AD-35) |
| FR101 | Epic 2 | Versioning segment, khôi phục được |
| FR102 | Epic 1 | Sao lưu = copy thư mục |
| FR103 | Epic 1 | Hai tầng cấu hình + `ScopeResolver` (AD-18) |
| FR104 | Epic 1 | Không telemetry — đúng ba điểm ra mạng (AD-15) |
| FR105 | Epic 10 | GitHub Releases macOS + Windows |
| FR106 | Epic 10 | Checksum SHA-256 |
| FR107 | Epic 10 | Build công khai GitHub Actions |
| FR108 | Epic 10 | Hướng dẫn cài có ảnh chụp màn hình |
| FR109 | Epic 10 | Màn hình Attribution |
| FR110 | Epic 10 | Kèm GPL v3 và giấy phép dữ liệu |
| FR111 | Epic 10 | Cập nhật chỉ kiểm tra và thông báo |
| FR112 | Epic 10 | Chính sách gỡ bỏ dữ liệu |
| FR113 | Epic 3 | Đề xuất bản dịch bằng âm Hán Việt |
| FR114 | Epic 3 | Trạng thái chờ chốt bản dịch |
| FR115 | Epic 6 | Nhập tài liệu song ngữ hai cột |
| FR116 | Epic 6 | Khớp câu trong từng cặp hàng |
| FR117 | Epic 2 | Xuất xứ bản dịch cấp segment (AD-31) |
| FR118 | Epic 7 | TM không trộn phong cách |
| FR119 | Epic 5 | Đánh dấu chỗ cần sửa khi đang đọc |
| FR120 | Epic 5 | Chỉ đọc phần đã xong, dừng ở biên |
| FR121 | Epic 8 | `.docx` một khối theo đoạn (AD-37, AD-38) |
| FR122 | Epic 6 | Nhập từ URL bằng danh sách link |
| FR123 | Epic 6 | Bóc nội dung + xem trước + sửa tay |
| FR124 | Epic 6 | Luật làm sạch lộ ra, duyệt trước khi xoá |
| FR125 | Epic 6 | Chuẩn hoá xuống dòng và khoảng trắng |
| FR126 | Epic 6 | Phát hiện và sửa bảng mã |
| FR127 | Epic 6 | Ảnh web tải về `.atproj`, giữ URL gốc |
| FR128 | Epic 6 | Xuất xứ tài liệu ở tầng Chương |
| FR129 ⇄ | Epic 6 ⇄ Epic 7 | Caption là Segment vai `caption` (AD-42). *Phần cấu trúc ở 6.13; phần nghiệm thu TM ở 7.1* |
| FR130 | Epic 8 | Chọn cách xuất ảnh: link gốc hay file ảnh |
| FR131 | Epic 8 | Khối ghi nguồn, mặc định tắt |
| FR132 | Epic 6 | Bộ lọc "cần xem" trên màn xem trước — *N cần xem · M sạch* |

**Tổng kiểm:** 132/132 FR được ánh xạ. Epic 1: 27 · Epic 2: 9 · Epic 3: **11** · Epic 4: **14** · Epic 5: 17 · Epic 6: **16** · Epic 7: 10 · Epic 8: 13 · Epic 9: 7 · Epic 10: 8.

> *(FR79 chuyển từ Epic 3 sang Epic 4 ngày soạn story — sửa một vi phạm luật phụ thuộc: bộ prompt chỉ được định nghĩa ở FR69.)*

#### Ánh xạ NFR

| NFR | Epic sở hữu nghiệm thu | Ghi chú |
|---|---|---|
| NFR1 Auto-Lookup p95 < 100 ms | Epic 1 | Đóng `[A1]` — backend chỉ tiêu 0,05 ms, phần còn lại là IPC + render |
| NFR2 Auto-save không gai trễ | Epic 2 | Đo cùng lúc với NFR18 |
| NFR3 Tìm kiếm Library p95 < 500 ms | Epic 5 *(sơ bộ)* · **Epic 6 *(đóng)*** | `[A6]` / Q4 — đóng ở Story 6.18, không đóng ở Epic 5 |
| NFR4 Khởi động < 3 s | Epic 5 *(sơ bộ)* · **Epic 6 *(đóng)*** | `[A7]` / Q4 — như NFR3 |
| NFR5 Bộ nhớ nhàn rỗi < 300 MB | Epic 5 *(sơ bộ)* · **Epic 6 *(đóng)*** | `[A8]` / Q4 — như NFR3 |
| NFR6 Kích thước bản cài **≤ 400.000.000 byte** *(payload sản phẩm; WebView2 Runtime nhúng nằm ngoài — sửa 2026-08-03, trần nâng 2026-08-05)* | Epic 1 | Mũi thăm dò font chạy **trước** Epic 1 (Story 1.1, xong) · nửa Windows đo ở **Story 1.3** · đối chiếu tổng ở **Story 1.9** · 🔄 **đo lại với hai lớp gỡ rời ở Story 1.10 (xong): 343.991.430 byte, ĐẠT trần mới** · **hai lớp gỡ rời còn lại phải đo TRƯỚC khi hứa đóng gói** *(story nối tiếp 1.10)* · nghiệm thu cuối ở Epic 10 |
| NFR7 Tra cứu offline 100% | Epic 1 | |
| NFR8 Chỉ mục chính phân biệt dấu | Epic 1 (từ điển) · Epic 5 (Library) | AD-27 áp cho cả hai chỉ mục |
| NFR9 Mang dữ liệu đi | Epic 1 (`.atproj`) · Epic 3 (CSV) · Epic 7 (TMX) | |
| NFR10 Mất chỉ mục không mất dữ liệu | Epic 5 | Test: xoá `library-index.db` rồi quét lại |
| NFR11 API key trong keychain | Epic 4 | |
| NFR12 Không telemetry | Epic 1 (thiết lập) · Epic 6 (điểm ra mạng thứ ba) | |
| NFR13 Không tài khoản, không cloud sync | Epic 1 | Ràng buộc nền, nghiệm thu bằng vắng mặt |
| NFR14 Native macOS + Windows tương đương | Epic 1 (thiết lập **+ CI cưỡng chế, Story 1.3**) · Epic 10 (nghiệm thu cuối) | CI hai nền tảng chạy từ Epic 1 nên khoảng giữa không còn bỏ trống |
| NFR15 Tương thích GPL v3 | **Mọi epic** | Rà **trước khi** thêm mỗi phụ thuộc; ghi vào bảng Stack |
| NFR16 Chuỗi giao diện ngoài mã nguồn | **Mọi epic**, thiết lập ở Epic 1 | Yêu cầu cắt ngang |
| NFR17 Sàn khả năng tiếp cận | **Mọi epic**, thiết lập ở Epic 1 | Yêu cầu cắt ngang; nghiệm thu cuối ở Epic 7 (vòng dịch trọn Chương không chạm chuột) |
| NFR18 Mất tối đa 5 giây | Epic 2 | |
| NFR19 Đường nhập URL chỉ ra mạng khi bấm | Epic 6 | Phải có **bộ test riêng** (AD-41) |

## Epic List

**Mười epic**, bám `build-sequence.md` (bản chốt, quyền hơn PRD §10). Hai giai đoạn bị tách vì có ranh giới rủi ro thật; các giai đoạn còn lại giữ nguyên làm một epic.

| Epic | Giai đoạn | FR | Vì sao đứng riêng |
|---|---|---|---|
| 1 | 1 | 27 | Mốc giá trị sớm nhất — bằng QuickTranslator, trên macOS |
| 2 | 2a | 9 | Editor là chỗ AD-31/AD-35/AD-12 hội tụ; có mũi thăm dò riêng |
| 3 | 2b | 11 | Miền Glossary, cưỡng chế bởi AD-20/AD-36 |
| 4 | 2c | 14 | `ai/` phải cô lập được **bằng test** (AD-13 → FR77) |
| 5 | 3a | 17 | Library + tầng dữ liệu dẫn xuất |
| 6 | 3b | 16 | **Ranh giới rủi ro:** hai giả định chưa đo (A12, A13), hai lớp lỗi im lặng |
| 7 | 4 | 10 | Translation Memory |
| 8 | 5 | 13 | Cầu nối Reviewer |
| 9 | 6 | 7 | Ứng viên cắt số 1 nếu R1 nổ — phải tách được sạch |
| 10 | 7 | 8 | Phát hành |

---

### Epic 1: Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì

Người dịch mở AuraTranslate, đưa một văn bản tiếng Trung hoặc tiếng Anh vào, bôi đen một cụm từ ở Panel Source và **thấy ngay định nghĩa có ghi nguồn** ở Panel Lookup — dưới 100 ms, hoàn toàn ngoại tuyến, với các nguồn bất đồng hiển thị cạnh nhau chứ không bị hợp nhất. Bật/tắt từng nguồn từ điển được, gỡ một lớp gỡ rời khỏi bản cài không làm hỏng bất kỳ đường tra cứu nào. Đây là **mốc giá trị sớm nhất**: làm được mọi thứ QuickTranslator làm, trên macOS lẫn Windows.

Epic này cũng đặt xuống các bất biến mà chín epic sau đều dựa vào — và chúng nằm ở đây **không phải vì gọn, mà vì rẻ nếu làm từ dòng code đầu tiên và rất đắt nếu vá sau**: cây nguồn theo Structural Seed, ba cổng, `store::Writer` nối tiếp, hình dạng `.atproj`, `CommandRegistry`, bộ token đã kiểm tương phản, `vi.json`, và pipeline dựng dữ liệu từ điển có checksum.

**FRs covered:** FR13 *(nhánh dán tay + `.txt`/`.md`)*, FR16, FR17, FR18, FR19, FR21, FR22, FR27, FR28, FR29, FR30, FR31, FR32, FR33, FR34, FR35, FR36, FR37, FR38, FR39, FR40, FR41, FR96, FR97, FR102, FR103, FR104

**NFRs:** NFR1, NFR6, NFR7, NFR8 *(từ điển)*, NFR9 *(`.atproj`)*, NFR13, NFR14 *(thiết lập)*, NFR15, NFR16, NFR17

**Ghi chú cài đặt:**
- **Story 1.3 dựng CI hai nền tảng ngay sau scaffold.** Lý do: AC *"hành vi tương đương trên macOS và Windows"* của Story 1.2 (NFR14) là một phép kiểm tay phải nhớ làm, và nó phải giữ đúng suốt **chín epic** trước khi FR107 dựng build công khai ở Epic 10. Một khác biệt nền tảng lọt vào ở Epic 2 mà chỉ lộ ra ở Epic 10 là lớp lỗi đắt nhất có thể tránh bằng một job CI. **Đây không phải FR107** — không build công khai, không checksum, không `dict-manifest.toml`; FR107 giữ nguyên phạm vi ở Story 10.1. *(Bổ sung 2026-08-03 sau rà soát mức sẵn sàng triển khai.)*
- ~~**Mũi thăm dò font phải chạy TRƯỚC epic này**~~ — ✅ **đã chạy 2026-08-03 (Story 1.1).** `.dmg` đo thật: chênh lệch do font **20,300 MiB = 21,29 MB**, tổng với database 130 MB = **151,29 MB**, dưới trần NFR6. SIL OFL 1.1 cả ba, tương thích GPL v3, ba hàng đã vào bảng Stack. Biến thể vùng chốt **TC**. `.msi` **chuyển sang Story 1.3** (xem AC1 đã thu hẹp). Mệnh đề chặn Epic 1 **đã gỡ**. **Rủi ro còn mở, không chặn:** trần NFR6 là trần của **cả bản cài đã gồm font**, nên phép tính đúng là trừ dư địa — 200 − 21,29 (font) − 1,40 (baseline app **rỗng**) − 130 (ba nguồn đầu) = **còn ~47 MB** cho các nguồn từ điển còn lại, chỉ mục FTS phụ (~17 MB), **và toàn bộ mã sản phẩm chưa viết**. Đối chiếu lại ở Story 1.9, nơi nay đã có AC thật. Vượt trần vẫn là thay đổi **tầng PRD**, không phải tầng kiến trúc. Số đo: [`research/font-spike-results-2026-08-03.md`](research/font-spike-results-2026-08-03.md).
- **NFR6 đã sửa 2026-08-03, rồi sửa LẦN HAI 2026-08-05 — trần nay là 400.000.000 byte.** ⚠️ Con số *dư địa ~47 MB* ở dòng trên là **bản ghi tại thời điểm 2026-08-03**, không còn sống: payload thật với bảy nguồn là **343.991.430 byte**, dư **56.008.570**. Story 1.2 đổi `webviewInstallMode` sang `offlineInstaller` để giữ lời hứa *cài được khi không có mạng*; chế độ đó nhúng trọn WebView2 Runtime (**≈ 127 MB**, tải lúc build) và một mình nó đủ đẩy `.msi` vượt trần **trước khi có một byte từ điển nào**. Cách xử lý đã chốt: **trần 150–200 MB là trần của payload sản phẩm; runtime nhúng nằm ngoài**, ghi thành dòng riêng trong mọi phép đo. Nhờ vậy trần vẫn là **một** con số chung cho macOS lẫn Windows và phép trừ dư địa ở dòng trên giữ nguyên. Đường quay lui (`downloadBootstrapper`, hoặc NSIS thay `.msi`) để mở ở **Story 10.2**, đi cùng hàng *"chưa khai artifact phát hành chính thức cho Windows"*.
- Không dùng starter template ngoài; scaffold theo **cây nguồn** ở Structural Seed và **Stack ghim phiên bản**.
- AD-26 ba nhánh truy vấn tiếng Trung là điều kiện nghiệm thu của FR39 — **`LIKE` bị cấm trên đường nóng**.
- AD-10: mỗi lớp gỡ rời một file `.db` tự mang metadata giấy phép; **trường giấy phép không được là enum các giấy phép mở** (HVTĐTD dùng theo phép riêng, không thuộc GPL v3).
- Nghiệm thu FR36 bằng test thật: **xoá file `.db` rồi chạy lại toàn bộ bộ test tra cứu — phải vẫn xanh**.
- FR13 ở đây chỉ mở đường vào tối thiểu để có văn bản mà tra; **nhánh `.docx` đóng ở Epic 6**, nhập hàng loạt và mọi pipeline làm sạch đóng ở Epic 6.
- **Ba chế độ đăng ký ngay từ epic này, kể cả khi hai chế độ còn rỗng.** `mode.library`, `mode.workspace`, `mode.reading` là ba command trong `CommandRegistry`, mỗi cái khai một **điểm vào focus** theo AD-34. Lý do: AD-24 (*một cửa sổ, ba chế độ ngang hàng*) là quy tắc bố cục **không có test**; đăng ký ba command từ đầu biến nó thành thứ **liệt kê được tự động**. Không làm vậy thì người dựng Epic 2 sẽ coi Workspace là chế độ duy nhất, và Epic 5 phải bóc ra để nhét Library vào ngang hàng — đó là mổ lại, không phải thêm tính năng.

---

### Epic 2: Biên tập theo segment — một vòng dịch tay hoàn chỉnh

Người dịch dịch trọn một Chương **bằng tay, không cần AI và không cần Glossary**: văn bản tách thành segment cấp câu, gộp hoặc tách khi máy tách sai, xác nhận từng câu với vạch lề đổi màu, điều hướng tới segment chưa dịch kế tiếp, chuyển Chương ngay trong Workspace. Sập ứng dụng giữa phiên gõ **mất tối đa 5 giây công việc**, và không frame nào vượt 50 ms trong lúc auto-save chạy. Mọi phiên bản cũ của một segment xem lại và khôi phục được.

**FRs covered:** FR20, FR23, FR24, FR25, FR26, FR78, FR100, FR101, FR117

**NFRs:** NFR2, NFR18

**Ghi chú cài đặt:**
- Đây là chỗ **bốn bất biến hội tụ**: AD-31 (máy trạng thái segment), AD-35 (hợp đồng flush), AD-11 (một writer nối tiếp), AD-12 (thời điểm checkpoint là quyết định của ứng dụng).
- **Mũi thăm dò bắt buộc trong epic này:** ngưỡng kích thước WAL buộc checkpoint **và** nhịp flush cụ thể, đo trên **cùng một** Editor thật — hai thứ đánh đổi lẫn nhau, phải dò cùng lúc.
- **Cạm bẫy đã được đặt tên và phải tránh:** debounce thuần bị reset bởi mỗi phím gõ nên **không bao giờ kích hoạt khi gõ liên tục** — trần cứng 5 giây **không được reset bởi phím gõ**.
- FR117 suy ra bằng cách **so văn bản đích hiện tại với bản lúc nạp segment**, không dùng cờ dirty.
- Cờ kết đoạn (AD-37) tính ở đây cùng lượt với ranh giới câu; ba ca biên gộp/tách/cuối Chương phải cài đúng ngay, vì Epic 8 sẽ dựa vào nó cho FR121.

---

### Epic 3: Glossary — chốt thuật ngữ một lần, dùng mãi

Người dịch bôi đen một cụm từ ở bất kỳ panel nào và thêm vào Glossary mà **không rời màn hình đang làm việc**; thuật ngữ đã chốt hiện ngay dấu trực quan trong Panel Source. Sau một lần nhập lớn, hàng trăm ứng viên do máy quét ra hiện thành bảng chờ xếp theo tần suất, **duyệt bằng một phím mỗi mục, không phải gõ chữ nào** — ứng viên tiếng Trung còn kèm sẵn bản dịch âm Hán Việt, chạy hoàn toàn ngoại tuyến. Glossary và bộ prompt xuất/nhập round-trip qua CSV/TSV để chia sẻ trong cộng đồng.

**FRs covered:** FR46, FR47, FR48, FR49, FR50, FR51, FR52, FR53, FR55, FR113, FR114

**NFRs:** NFR9 *(CSV/TSV)*

**Ghi chú cài đặt:**
- **AD-20 là trụ định vị #1 phát biểu dưới dạng cấu trúc dữ liệu:** đề xuất tự động ghi vào **bảng chờ riêng**, không bao giờ vào Glossary. Chỉ thao tác duyệt của người dùng mới chuyển một mục sang.
- **AD-36:** ba trạng thái một chiều (ứng viên → chờ chốt → đã chốt), trường bản dịch **nullable** cho tới khi chốt. `glossary/` phơi ra **đúng một** truy vấn trả về mục **đủ điều kiện chèn** — điều kiện nằm ở nơi sở hữu dữ liệu, không ở chỗ gọi.
- Âm Hán Việt của FR113 đọc **qua cổng `DictionarySource`**, không cài lại lần thứ hai.
- FR52 chạy trên đường nhập tối thiểu của Epic 1; nó **tự động áp cho mọi đường nhập** khi Epic 6 mở rộng pipeline — không cần sửa lại.
- **FR54 KHÔNG nằm ở epic này** — đầu vào của nó chỉ tồn tại sau Epic 8. Bảng chờ dựng ở đây là thứ FR54 sẽ ghi vào.

---

> **Thứ tự story đã đổi 2026-08-03.** *Trạng thái chờ chốt* (FR114) nay đứng ở **Story 3.6**, trước *Đề xuất bản dịch bằng âm Hán Việt* (FR113, nay **Story 3.7**) và *Duyệt hàng loạt* (FR53, nay **Story 3.8**). Lý do: FR113 có **nhánh dự phòng là FR114** — ứng viên không đề xuất được thì đi đường chờ chốt. Thứ tự cũ bắt story đề xuất nghiệm thu một nhánh chưa tồn tại. Xoay lên thay vì hoán vị hai story, vì *Duyệt hàng loạt* dùng **cả hai** nên nó phải đứng cuối.

### Epic 4: AI mở & Smart RAG Injector

Người dịch cấu hình **một API key của chính mình** hoặc trỏ tới Ollama/LM Studio qua **cùng một đường cấu hình**, rồi gọi AI dịch từng segment hoặc theo lô — kết quả chảy dần theo dòng, huỷ được giữa chừng, kèm số token và ước tính chi phí. Trước mỗi lần gọi, hệ thống tự chèn các thuật ngữ Glossary **đã chốt** xuất hiện trong câu; người dùng **mở xem được prompt cuối cùng đã gửi đi**, gồm toàn bộ phần chèn động. Kết quả nằm ở panel AI Translation và **không bao giờ tự chảy vào Editor**. Và gỡ sạch cấu hình AI thì Epic 1, 2, 3 vẫn chạy đầy đủ — điều này được **cưỡng chế bằng test tự động**, không bằng kỷ luật.

**FRs covered:** FR65, FR66, FR67, FR68, FR69, FR70, FR71, FR72, FR73, FR74, FR75, FR76, FR77, FR79

**NFRs:** NFR9 *(prompt văn bản mở)*, NFR11, NFR12 *(điểm ra mạng thứ nhất)*

**Ghi chú cài đặt:**
- **AD-13 là ranh giới cứng:** không module nào ngoài `ai/` được phụ thuộc `ai/`, có **test tự động cưỡng chế** hoặc `ai/` là crate riêng để trình biên dịch cưỡng chế.
- **AD-14:** `RagInjector` là một **hàm thuần** trả về prompt đã lắp hoàn chỉnh. Không nối chuỗi rải rác ở chỗ gọi — đây là điều kiện để FR71 chính xác 100%.
- **AD-22:** Tauri Channel API, **không client SSE tự kết nối lại** — auto-reconnect với BYOK là tính phí hai lần và trả về văn bản trùng lặp.
- **AD-29:** crate `keyring` **trực tiếp trong Rust**, không dùng `tauri-plugin-keyring` (plugin tồn tại để phơi API ra JavaScript — đúng thứ NFR11 cấm). Khoá **không bao giờ đi qua IPC**.
- **Mũi thăm dò:** cách phân tích khung SSE — `reqwest-sse` và `sseer` **chưa xác nhận giấy phép**, không được vào Stack khi chưa rà GPLv3; phân tích bằng tay là phương án hợp lệ.
- Nửa TM của FR70 đóng ở Epic 7; ở epic này FR70 nghiệm thu trên phần Glossary.

---

### Epic 5: Library — kho tác phẩm, tìm kiếm, và đọc lại thành quả

Mở ứng dụng là vào Library, không phải vào màn hình dịch. Người dịch **nắm được mình đang có những gì**: lưới Tác phẩm với bìa, tiến độ và bốn trạng thái vòng đời; lọc theo trạng thái, lĩnh vực, ngôn ngữ nguồn, ngày sửa; **tìm full-text xuyên toàn thư viện phân biệt dấu** — `má` không ra `ma`, `mà`, `mả`, `mã`, `mạ` — với chế độ khoan dung chỉ bật khi cần. Mở một Chương đưa thẳng vào Workspace **đúng câu đang dở lần trước**. Và Chế độ đọc để người dịch **quay lại thưởng thức thành quả**: đọc liên tục qua các Chương đã xong, dừng ở biên tường minh, đánh dấu chỗ cần sửa bằng một phím rồi **đọc tiếp ngay** — phiên đọc không bị cắt. Xoá chỉ mục Library rồi quét lại phục hồi đầy đủ, không mất một byte dữ liệu nào.

**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR7, FR8, FR9, FR10, FR11, FR12, FR15, FR98, FR99, FR119, FR120

**NFRs:** NFR3, NFR4, NFR5, NFR8 *(chỉ mục Library)*, NFR10

**Ghi chú cài đặt:**
- **AD-8:** `library-index.db` là **file riêng**, không nằm chung `global.db`; chỉ `Indexer` ghi vào và chỉ ghi **sau khi** `.atproj` đã ghi xong. **Xoá nó phải luôn là thao tác an toàn** — nếu không, thao tác sửa chữa hiển nhiên nhất sẽ xoá luôn Glossary/TM toàn cục người dùng tích luỹ hàng năm.
- **AD-9:** Library đọc `meta.json` **không cần mở SQLite** — đây là điều kiện của NFR4 với 5.000 Chương.
- **AD-33:** `meta.json` là cache dẫn xuất, ghi bởi **chính `store::Writer` của Tác phẩm đó**, trong cùng thao tác logic. Không hai chủ sở hữu cho tiến độ.
- **AD-32 khác AD-5 một cách cố ý:** gộp/tách **Chương** chỉ đổi `chapter_id` và `ord`; `segment.id`, lịch sử phiên bản và trạng thái xác nhận **giữ nguyên**. Cài FR15 như "tạo lại segment" sẽ phá sạch những Chương đã dịch xong.
- **AD-5:** chỗ đánh dấu FR119 trỏ tới segment đã về hưu **ở lại, không bị xoá im lặng** — hiện kèm ghi chú *câu này đã đổi*.
- ⚠️ **Rủi ro lịch trình đã biết, chưa xử lý — Q4 không đóng được ở đây.** Điều kiện đóng `[A6] [A7] [A8]` là *"đo trên thư viện thật **5.000 Chương**"*. Nhưng **không có đường nào tạo ra 5.000 Chương trước Epic 6** — đường nhập tối thiểu của Epic 1 chỉ dán tay từng Chương. Sinh dữ liệu giả đo được **tốc độ** nhưng không đo được thứ NFR8 tồn tại để bảo vệ: phân bố dấu tiếng Việt thật (`má / ma / mà / mả / mã / mạ`). Cùng lớp vấn đề áp cho **bảng chờ Glossary của Epic 3** — màn hình thiết kế cho hàng trăm dòng chỉ hỏng ở quy mô thật.
  **Ba đường xử lý, chưa chọn:** *(a)* đảo Epic 5 ↔ Epic 6 · *(b)* giữ thứ tự nhưng **dời việc đo NFR3/4/5 xuống sau Epic 6** · *(c)* tách Epic 5 làm đôi — FR1–FR7 lên trước đường nhập, FR8/FR9 + FR11/FR119/FR120 xuống sau. Cho tới khi chọn, **Epic 5 đóng lại với ba ngưỡng vẫn treo**.
- FR11 giao thoa với đặc tả typography của `EXPERIENCE.md` (ba mức Thoáng/Cân/Đặc, sàn giãn dòng 1.66) — PRD bàn giao mục này có chủ ý.

---

### Epic 6: Đường nhập — mọi nguồn văn bản vào được, và không hỏng im lặng

Đây là **bề mặt đầu tiên người dùng chạm vào sản phẩm**, và là nơi hai lỗi đắt nhất của ứng dụng có thể xảy ra mà không báo gì cả. Người dịch nhập một bộ 2000 chương từ một file `.txt` 40 MB, hoặc **dán 50 link web**, hoặc một file song ngữ hai cột do người khác dịch — tất cả đi qua **một màn xem trước hợp nhất** cho thấy bảng mã đã đoán, ranh giới nội dung đã bóc, và những gì luật làm sạch **sắp xoá** — trước khi một byte nào ghi xuống đĩa. Gặp file GBK, năm bản dựng thật hiện cạnh nhau và người dịch Việt nhận ra `第一章` với `ç¬¬ä¸€ç«` trong một phần giây. Ảnh trong bài web tải về nằm trong `.atproj` nên copy Tác phẩm sang máy ngoại tuyến vẫn đủ ảnh; alt-text và caption là **hai segment dịch được riêng biệt**. Và ứng dụng **không bao giờ tự quyết định tải cái gì** — hai con số *N link · sẽ tạo N Chương* đứng cạnh nhau là bằng chứng đọc được.

**FRs covered:** FR13 *(hoàn thiện nhánh `.docx`)*, FR14, FR42, FR43, FR44, FR45, FR115, FR116, FR122, FR123, FR124, FR125, FR126, FR127, FR128, FR129, FR132

**NFRs:** NFR12 *(điểm ra mạng thứ ba)*, NFR19, **NFR3 · NFR4 · NFR5** *(đóng ở Story 6.18 — Epic 5 chỉ đo sơ bộ vì chưa có đường tạo 5.000 Chương)*

**Ghi chú cài đặt:**
- **AD-39 là xương sống của epic này:** một pipeline, **cùng thứ tự cho mọi nguồn**. Ca hỏng cụ thể nhất và dễ viết test nhất — đặt bước tách Chương **trước** bước giải mã bảng mã: mẫu chạy trên chữ rác, cả file 40 MB ra **đúng một Chương**, không lỗi nào được ném.
- **AD-41 phải có bộ test riêng** vì Tauri capabilities khai báo **tĩnh lúc build** nên **không diễn đạt được** *"chỉ các domain trong danh sách vừa dán lúc chạy"*. Bốn test tối thiểu: từ chối host ngoài hai tầng · từ chối chuyển hướng ra ngoài · từ chối tài liệu ở tầng 2 · không lời gọi nào khi người dùng không bấm.
- **AD-40:** `Fetcher` **không bao giờ** phân tích nội dung, `Extractor` **không bao giờ** chạm mạng — ranh giới này là thứ làm NFR19 nghiệm thu được.
- **AD-42:** `ASSET` mang **neo vị trí của chính nó**, độc lập với việc có hay không có segment đi kèm — vì ảnh trên web **thường không có thuộc tính `alt`** để giữ chỗ. Ảnh không có caption **không sinh segment rỗng**.
- **Hai mũi thăm dò chạy cùng lúc, ngay đầu epic:** `dom_smoothie` cho bóc nội dung `[A12]` và `chardetng` + `encoding_rs` cho dò bảng mã `[A13]`. **Tỉ lệ sai cao không chặn tiến độ** — đường sửa tay là nghiệm thu, không phải vá.
- **Mũi thăm dò thứ ba:** HTTP client cho `Fetcher` — cần theo dõi chuyển hướng để cưỡng chế allowlist, giới hạn kích thước, timeout.
- **Quyết định còn treo, đóng trong epic này:** hành vi khi một link trong danh sách hỏng (404, timeout, tường chặn) — dừng, bỏ qua, hay giữ chỗ trống. AD-39 đã cố định phần bất biến (mọi thứ xảy ra **trước bước ghi**) nên không có ca ghi nửa chừng.

---

> **Thứ tự story đã đổi 2026-08-03, và epic có thêm một story.** *Đọc `.docx`* chuyển từ vị trí 7 xuống **Story 6.12**, sau *Ảnh tải về `.atproj`* (**Story 6.11**) — vì `.docx` có ảnh và phải đi vào **đường xử lý tài sản** do story ảnh mở ra. Các story 6.7–6.11 dồn lên một bậc. Xoay xuống thay vì hoán vị, vì đường xử lý tài sản lại cần `Fetcher` của *Nhập từ URL* (**Story 6.7**) — hoán vị thẳng sẽ đẩy story ảnh lên trước cả đường tải, tạo ra một phụ thuộc tiến mới.
>
> **Story 6.18** *(đo lại NFR3, NFR4, NFR5)* là story mới: Story 5.14 ở Epic 5 **không thể** hoàn thành mục đích của nó vì Epic 5 chưa có đường nào tạo ra 5.000 Chương — nhập hàng loạt (FR14) nằm ở chính epic này. Trước bổ sung này, phép đo *"phải chạy lại sau Epic 6"* là một lời nhắc không có chủ. **Q4 của PRD đã được sửa** từ *"Giai đoạn 3"* thành *"sau Giai đoạn 3b"* cho khớp.

### Epic 7: Translation Memory — không dịch lại, không tra lại thứ đã dịch

Mỗi lần người dịch xác nhận một segment, cặp *(nguồn → đích)* **tự vào Translation Memory, không một thao tác thủ công nào**. Từ đó về sau: câu y hệt được **điền sẵn nhưng vẫn ở trạng thái chưa xác nhận** — hệ thống không bao giờ tự coi một câu là xong; câu tương tự hiện kèm phần trăm khớp và diff phần khác biệt; và Concordance trả lời *"cụm này trước đây tôi dịch thế nào?"* ngay trong Panel Lookup. TM xuất được TMX mở ở CAT tool khác. Và vì chủ dự án làm **cả hai vai** — tự dịch và biên tập bản của người khác — mỗi cặp TM mang **xuất xứ**, và Smart RAG Injector **ưu tiên cặp của chính người dùng** để AI học đúng văn phong.

**FRs covered:** FR56, FR57, FR58, FR59, FR60, FR61, FR62, FR63, FR64, FR118

**NFRs:** NFR9 *(TMX)*

**Ghi chú cài đặt:**
- **AD-6:** mục TM khoá theo **cặp văn bản**, độc lập hoàn toàn với `segment.id` — nếu không, TM vỡ mỗi khi segment bị gộp, tách hay tái tách. Sửa bản dịch của segment đã xác nhận thì **ghi thêm cặp mới** (khớp FR63).
- **AD-31:** cặp TM ghi **đúng tại chuyển tiếp sang đã xác nhận**, không ở chỗ nào khác.
- **AD-18 khai tường minh thứ tự hai khoá** vì hai chiều này chồng lên nhau: khoá chính là **xuất xứ**, khoá phụ là **tầng**. Xuất xứ thắng vì một cặp toàn cục do chính người dùng dịch vẫn giống văn phong họ hơn một cặp Tác phẩm do người khác dịch.
- Xác nhận segment **chỉ** ghi vào TM Tác phẩm; TM toàn cục chỉ nhận qua thao tác chủ động.
- Epic này **đóng nốt nửa TM của FR70** — đây là chỗ TM và Glossary nhân giá trị cho nhau.
- Epic này là **ứng viên cắt số 3** nếu R1 nổ; ranh giới AD-6 khiến nó tách được sạch.

---

### Epic 8: Cầu nối Reviewer — xuất, nhập lại, đối chiếu, và hấp thụ bài học

Reviewer **không cài AuraTranslate**, nên trao đổi file là cầu nối duy nhất. Người dịch xuất `.docx` bảng hai cột đối xứng theo segment cho reviewer sửa, hoặc **`.docx` một khối đối xứng theo đoạn** để bôi đen cột phải dán thẳng sang trình soạn thảo website — và màn hình xuất **nói rõ ngay lúc chọn** rằng định dạng thứ hai không nhập lại được. Nhận file đã sửa về: hệ thống khớp cấu trúc đoạn và **hiện ra cho người dùng nối tay** những chỗ không khớp, rồi Review Mode ẩn văn bản gốc và bôi màu thêm/xoá/sửa để lướt qua từng thay đổi. Và **kể cả khi người dịch không bao giờ mở Review Mode**, hệ thống vẫn báo *"Reviewer đổi «Bắc Lương vương» thành «vương Bắc Lương» ở 23/24 lần xuất hiện. Thêm vào Glossary?"* — vòng lặp học hỏi đóng lại **ở tầng hệ thống**, không trông chờ vào kỷ luật con người.

Với vai **Người dịch bài đăng**: chọn xuất ảnh theo link gốc hay theo file ảnh, và khối ghi nguồn đủ tác giả · báo/website · URL bài gốc · ngày đăng · tên người dịch — để **người đăng không cài app, không hỏi thêm câu nào, dựng lại được trọn bài**.

**FRs covered:** FR54, FR87, FR88, FR89, FR90, FR91, FR92, FR93, FR94, FR95, FR121, FR130, FR131

**Ghi chú cài đặt:**
- **AD-38 là cổng vào bắt buộc, chạy ở Rust TRƯỚC alignment và trước mọi lệnh ghi.** Bản `.docx` một khối (FR121) và bản theo segment (FR87) **cùng phần mở rộng, cùng là bảng hai cột**. Không có cổng này, kéo lại bản đăng bài vài tháng sau sẽ **ghi đè cả Chương đã xác nhận bằng một khối văn bản duy nhất, không báo lỗi**. Nhận dạng bằng **hình dạng**, không bằng metadata hay tên file.
- **AD-37 đã lo phần khó của FR121** ở Epic 2: cờ kết đoạn là **dữ liệu được lưu**, một cờ duy nhất dùng chung cho cả hai cột — nên lời hứa *"đúng số lần xuống đoạn như nhau"* đúng **theo định nghĩa**, không nhờ hai đường mã tình cờ đồng ý.
- **AD-43:** khối ghi nguồn **dựng lúc xuất** từ bốn trường trên `CHAPTER` cộng tên người dịch; **không lưu chuỗi đã định dạng ở bất kỳ đâu**. Đường xuất phải **quét phạm vi trước và liệt kê ảnh không có `source_url`**, không được im lặng bỏ qua.
- **FR95 là FR quan trọng nhất của epic này** và nó tồn tại vì Q1 chưa có lời giải. Thu hoạch thuật ngữ (FR54) phải chạy **độc lập** với FR92–FR94 — nếu cắt Diff Viewer thì FR95 **không được cắt theo**.
- **Mũi thăm dò trước epic này:** đọc thử `.docx` bảng hai cột thật bằng `docx-rs` 0.4.22, lấy được số hàng và số đoạn trong từng ô. Không đạt thì rà `docx-reader` hoặc `rdocx` — **cả hai chưa xác nhận giấy phép**, phải rà GPLv3 trước.
- **Mũi thăm dò trong epic:** `similar` vs `dissimilar` — thử cả hai trên bản review thật.
- FR92–FR94 là **ứng viên cắt số 2** nếu R1 nổ.

---

> **Thứ tự story đã đổi 2026-08-03.** *Chọn cách xuất hình ảnh* (FR130) nay là **Story 8.5**, trước *Xuất `.md` và text thuần* (FR88, nay **Story 8.6**) — vì FR88 xuất ảnh **theo kiểu người dùng chọn ở FR130**, nên story xuất `.md` không nghiệm thu được nhánh ảnh nếu lựa chọn đó chưa tồn tại.

### Epic 9: AI Proofreader — bắt lỗi trước khi bàn giao

Người dịch chạy proofreader **theo yêu cầu** trên một segment, một Chương hoặc vùng đang chọn — **không bao giờ chạy nền**, vì với BYOK quét nền là tính phí ngoài ý muốn và với local LLM là chiếm tài nguyên máy suốt phiên. Mỗi phát hiện gồm loại lỗi, vị trí, giải thích ngắn và đề xuất sửa, hiện **gạch chân lượn sóng ngay dưới đúng cụm chữ có vấn đề** chứ không phải một danh sách rời. Chấp nhận hoặc bỏ qua từng cái; đánh dấu *"không phải lỗi"* thì lần quét sau **không báo lại trong cùng Tác phẩm**. Và proofreader **không bao giờ tự sửa văn bản**.

**FRs covered:** FR80, FR81, FR82, FR83, FR84, FR85, FR86

**Ghi chú cài đặt:**
- **Nghiệm thu FR81 không theo lối thông thường** — không có đáp án đúng để đối chiếu. Thứ đo được là **tỷ lệ báo động giả**: số phát hiện bị đánh dấu *"không phải lỗi"* trên tổng số. Ngưỡng là *đủ thấp để người dùng không tắt hẳn tính năng*. Chỉ số này neo trực tiếp vào FR84.
- **Ghi nhớ proofreader khoá theo `(work, chữ ký phát hiện)`, KHÔNG theo `segment.id`** — FR84 nói phạm vi là *"trong cùng Tác phẩm"*, và nhờ vậy ghi nhớ sống sót qua gộp/tách segment.
- Vạch lề đã dùng hết năm giá trị cho trạng thái segment, nên phát hiện dùng **gạch chân** ở `text-underline-offset: 4px` để không chạm dấu nằm dưới của `ạ ộ ợ`. Hai màu, không thêm màu mới: `error` cho chính tả/ngữ pháp, `tm-rule` cho nghi về nghĩa.
- Epic này là **ứng viên cắt số 1** nếu R1 nổ. AD-13 khiến nó tách được sạch — đó là lý do nó đứng riêng.

---

### Epic 10: Phát hành & tin cậy — vượt rào cản không ký số

Một người dịch phổ thông tải bản cài từ GitHub Releases, đối chiếu checksum SHA-256, đi theo **hướng dẫn cài đặt có ảnh chụp màn hình** xử lý tường minh Gatekeeper trên macOS và SmartScreen trên Windows, và cài được — dù bản phát hành **không ký, không notarize**. Trong ứng dụng, màn hình Attribution liệt kê mọi nguồn từ điển kèm giấy phép, **dựng từ các file dữ liệu có mặt** nên gỡ một lớp cũng gỡ luôn ghi công của nó, không để lại ghi công mồ côi. Cơ chế cập nhật **chỉ kiểm tra và thông báo** — không tự tải, không tự cài.

**FRs covered:** FR105, FR106, FR107, FR108, FR109, FR110, FR111, FR112

**NFRs:** NFR6 *(nghiệm thu cuối)*, NFR14 *(nghiệm thu cuối)*, NFR15 *(rà toàn bộ)*

**Ghi chú cài đặt:**
- **Chuỗi ràng buộc nối tiếp, đừng vô tình gỡ một mắt xích:** không kinh phí → không ký số → niềm tin phải đến từ nơi khác → build công khai + checksum (FR106, FR107) → **và cấm cơ chế tự cập nhật** (FR111). Một cơ chế tự cập nhật trên bản không ký số là **đường tấn công thật**.
- **AD-25:** CI tải các file `.db` theo `dict-manifest.toml`, **đối chiếu checksum**, rồi đóng gói. Parser định dạng từ điển **chỉ nằm trong build tool, không vào bản phát hành** — nên giấy phép parser không ràng buộc sản phẩm.
- FR112 nghiệm thu bằng thao tác thật: **gỡ một nguồn = xoá một file, không đổi một dòng mã** — và màn hình Attribution tự cập nhật theo.
- Ghi phép dùng HVTĐTD vào `LICENSE`/`NOTICE`: © Đặng Thế Kiệt, **không thuộc GPL v3**.
- **Nghĩa vụ ngoài mã nguồn, không mang số FR nhưng không được rơi:** thông báo cho tác giả Đặng Thế Kiệt khi công cụ hoàn thành — đề nghị tường minh trong thư đồng ý, và là **điều kiện của phép sử dụng**.

---

## Epic 1: Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì

Người dịch mở AuraTranslate, đưa một văn bản tiếng Trung hoặc tiếng Anh vào, bôi đen một cụm từ ở Panel Source và **thấy ngay định nghĩa có ghi nguồn** ở Panel Lookup — dưới 100 ms, hoàn toàn ngoại tuyến, với các nguồn bất đồng hiển thị cạnh nhau chứ không bị hợp nhất. Bật/tắt từng nguồn từ điển được, gỡ một lớp gỡ rời khỏi bản cài không làm hỏng bất kỳ đường tra cứu nào. Đây là **mốc giá trị sớm nhất**: làm được mọi thứ QuickTranslator làm, trên macOS lẫn Windows.

### Story 1.1: Mũi thăm dò font — đo dung lượng thật và rà giấy phép

**Covers:** NFR6 · NFR15 · ~~mũi thăm dò bắt buộc — chặn mọi story khác của Epic 1~~ → ✅ **mệnh đề chặn đã gỡ 2026-08-03**, kết quả đã ghi (AC4 đạt điều kiện *"kết quả được ghi lại"*)

As a chủ dự án,
I want biết chắc bộ font nhúng nằm trong ngân sách NFR6 và giấy phép cho phép phân phối lại,
So that tôi không phải bóc font ra sau khi đã dựng nửa giao diện trên nó.

**Acceptance Criteria:**

> ⚠️ **AC1 đã thu hẹp 2026-08-03 — Ice quyết trong lúc chạy Story 1.1.** Bản gốc đòi đo **cả `.dmg` lẫn `.msi`**. `tauri-cli` 2.11.4 trên macOS từ chối target `msi` (*"possible values: ios, app, dmg"*) vì `.msi` dựng bằng WiX v3 mà `candle`/`light` là chương trình Windows — rào ở tầng đóng gói, không phải tầng biên dịch Rust. **Hai phép đo `.msi` chuyển sang Story 1.3**, nơi CI đã có runner Windows và chi phí gần bằng 0. Lý do chấp nhận đo muộn: ước 16,0–20,3 MiB nằm gọn trong trần, và phương pháp ước đã tự kiểm sai số 0,1 % trên chính phép đo macOS; rủi ro Windows thật sự nằm ở **chế độ cài WebView2** chứ không ở font, mà thứ đó chỉ CI mới bắt được. Bản gốc giữ ở đây dưới dạng gạch ngang để đọc lại được.

~~**Given** ba họ font đã chốt — `Source Serif 4`, `Source Han Serif` (chỉ Regular), `Source Sans 3` · **When** đóng gói thử một `.dmg` **và một `.msi`** có nhúng font, và một cặp không nhúng · **Then** chênh lệch dung lượng được ghi lại thành số cụ thể cho **từng nền tảng** · **And** con số đó cộng với 130 MB database đã đo phải nằm trong trần 150–200 MB của NFR6~~

**Given** ba họ font đã chốt — `Source Serif 4`, `Source Han Serif` (chỉ Regular), `Source Sans 3`
**When** đóng gói thử một `.dmg` có nhúng font và một `.dmg` không nhúng
**Then** chênh lệch dung lượng được ghi lại thành số cụ thể
**And** con số đó cộng với 130 MB database đã đo phải nằm trong trần 150–200 MB của NFR6
**And** phép đo tương ứng cho `.msi` được **bàn giao sang Story 1.3** kèm công thức chạy, chứ không bị bỏ im lặng

**Given** giấy phép SIL OFL của ba họ font
**When** rà theo NFR15
**Then** kết luận tương thích GPL v3 được ghi vào bảng Stack của `ARCHITECTURE-SPINE.md`
**And** nếu không tương thích, mũi thăm dò kết thúc bằng một đề xuất font thay thế chứ không bằng việc bỏ qua

**Given** hai biến thể vùng TC và SC của `Source Han Serif`
**When** dựng thử cùng một đoạn văn bản chứa các mã Hán dùng chung
**Then** biến thể được chọn và lý do được ghi lại

**Given** tổng dung lượng vượt trần NFR6
**When** mũi thăm dò kết thúc
**Then** kết quả được báo cáo là **thay đổi tầng PRD cần chủ dự án quyết**, không phải một tối ưu ở tầng kiến trúc
**And** không story nào của Epic 1 bắt đầu trước khi kết quả này được ghi lại

---

### Story 1.2: Scaffold dự án và khoá phạm vi filesystem, phạm vi mạng

**Covers:** FR104 · NFR12 · NFR14

As a người dựng,
I want một khung dự án dựng theo đúng cây nguồn đã chốt với phạm vi filesystem và CSP khoá từ commit đầu tiên,
So that *"không ai đọc được tài liệu của bạn"* là ràng buộc do framework cưỡng chế chứ không phải một lời hứa.

**Acceptance Criteria:**

**Given** cây nguồn ở Structural Seed
**When** scaffold hoàn tất
**Then** tồn tại `src-tauri/src/{commands,core/{segment,matching,glossary,tm,dict,library,export,webimport,ai,scope,store},ports}`, `src-tauri/capabilities/`, `src-tauri/resources/dict/`, `src/{modes,panels,layout,commands,tokens,i18n}`, `tools/dict-build/`, `dict-manifest.toml`
**And** không dùng bất kỳ starter template cộng đồng nào

**Given** bảng Stack ghim phiên bản
**When** cài đặt phụ thuộc
**Then** phiên bản khớp đúng bảng
**And** `tauri-plugin-stronghold`, `tauri-plugin-keyring`, `tauri-wire` không có mặt trong cây phụ thuộc

**Given** capabilities khai tĩnh
**When** ứng dụng chạy
**Then** chỉ `$RESOURCE/dict/**` và `$RESOURCE/fonts/**` đọc được, `$APPDATA/**` đọc và ghi được
**And** một thử nghiệm đọc file ngoài scope bị Tauri từ chối

**Given** CSP mặc định của Tauri
**When** frontend nạp
**Then** mọi origin từ xa bị chặn — không CDN, không font ngoài, không ảnh ngoài
**And** không có mã nào nới CSP

**Given** ứng dụng chạy trọn một phiên làm việc
**When** quan sát lưu lượng mạng
**Then** không có lời gọi ra ngoài nào (FR104, NFR12)
**And** không có crash reporter hay thư viện analytics trong cây phụ thuộc

**Given** cùng một commit
**When** build trên macOS và trên Windows
**Then** cả hai ra bản chạy được với hành vi tương đương (NFR14)

> **Bổ sung 2026-08-03 — nhận bàn giao từ Story 1.1.** Bốn tệp font đã đo và đã chốt hiện **chỉ tồn tại trong thư mục scratchpad của phiên làm việc 2026-08-03**, đúng theo `§Ranh giới phạm vi` của mũi thăm dò (mã và tài nguyên dùng một lần không vào repo). Story này là story đầu tiên dựng cây nguồn thật, nên nó là nơi đưa chúng vào. Nguồn, SHA-256 và tên tệp chính xác: [`research/font-spike-results-2026-08-03.md`](research/font-spike-results-2026-08-03.md) §Phép đo 5.

**Given** bốn tệp font đã chốt ở Story 1.1
**When** dựng cây nguồn
**Then** chúng nằm ở `src-tauri/resources/fonts/`, và **SHA-256 từng tệp khớp đúng bảng §Phép đo 5** của báo cáo mũi thăm dò — lệch một byte là **dừng**, không tự tải bản khác thay vào
**And** nguồn tải ghi rõ trong repo: `notofonts/noto-cjk` tag `Serif2.003` cho `NotoSerifCJKtc-Regular.otf`; `google/fonts` cho `SourceSerif4[opsz,wght].ttf` · `SourceSerif4-Italic[opsz,wght].ttf` · `SourceSans3[wght].ttf`
**And** **ba tệp giấy phép SIL OFL gốc** đi kèm bản phát hành (FR38, FR109) — nghĩa vụ này đến từ kết luận rà NFR15 của Story 1.1 và trước đây chưa story nào cưỡng chế nó
**And** **không** tải font qua `fonts.googleapis.com` hay bất kỳ origin từ xa nào lúc chạy (AD-15)

---

### Story 1.3: CI tối thiểu — hai nền tảng, mỗi lần push

**Covers:** NFR14 · NFR15 · **NFR6** *(nửa Windows, nhận bàn giao từ Story 1.1 ngày 2026-08-03)* · lưới an toàn cho AC hai nền tảng của Story 1.2 (không phải FR107)

As a chủ dự án,
I want mỗi lần push đều được build và chạy test trên cả macOS lẫn Windows,
So that một khác biệt nền tảng lọt vào ở Epic 2 không nằm im tới tận Epic 10 mới lộ ra.

**Acceptance Criteria:**

**Given** một commit bất kỳ được đẩy lên
**When** CI chạy
**Then** `cargo test` và build ứng dụng chạy trên **cả macOS lẫn Windows**
**And** kết quả hai nền tảng hiện **tách bạch**, không gộp thành một trạng thái chung

**Given** một test trượt, hoặc một nền tảng build hỏng
**When** CI kết thúc
**Then** trạng thái là **đỏ**
**And** commit đó không được coi là xong

**Given** AC hai nền tảng của Story 1.2 (NFR14 — *hành vi tương đương trên macOS và Windows*)
**When** kiểm
**Then** nó được **cưỡng chế bằng CI**, không còn là một phép kiểm tay phải nhớ làm

**Given** các luật cưỡng chế bằng test sinh ra ở epic sau — lint cấm màu viết thẳng (AD-34), test ranh giới `ai/` (AD-13), bốn test allowlist (AD-41)
**When** chúng tồn tại
**Then** gắn vào **chính pipeline này**
**And** không dựng pipeline thứ hai

**Given** phạm vi của story này
**When** so với FR107
**Then** đây **không phải** FR107 — không build công khai để người ngoài kiểm chứng, không checksum, không `dict-manifest.toml`
**And** FR107 vẫn đóng ở Story 10.1 với phạm vi nguyên vẹn

> **Bổ sung 2026-08-03 — nhận bàn giao từ Story 1.1.** Nửa Windows của AC1 Story 1.1 chuyển vào đây vì `.msi` chỉ dựng được trên Windows. Công thức, cấu hình và số macOS để đối chiếu: [`research/font-spike-results-2026-08-03.md`](research/font-spike-results-2026-08-03.md) §Công thức đo trên Windows.

**Given** runner Windows của CI đã chạy được
**When** CI chạy trên một commit bất kỳ
**Then** dung lượng `.msi` **có font** và **không font** được ghi lại thành số cụ thể, và **chênh lệch** được đối chiếu với dải ước **16,0–20,3 MiB** của mũi thăm dò
**And** chế độ cài WebView2 đang dùng được ghi kèm — ⚠️ **nay là `offlineInstaller`**, không còn `downloadBootstrapper`: Ice đổi ngày 2026-08-03 sau code review Story 1.2, ưu tiên lời hứa *cài được khi không có mạng*. Nó cộng **≈ 127 MB** (tải lúc build) vào `.msi`, và theo NFR6 đã sửa cùng ngày thì **phần đó nằm ngoài ngân sách 150–200 MB** — nên phép đo phải **tách hai dòng**: payload sản phẩm và WebView2 Runtime nhúng
**And** chế độ này **triệt tiêu trong phép trừ** (cả hai bản build đều mang nó), nên **chênh lệch** do font vẫn đối chiếu sạch với dải ước 16,0–20,3 MiB
**And** hai số này ghi lại ở **mỗi lần CI chạy**, không phải một lần rồi thôi — đây là lưới bắt hồi quy khi bộ font hoặc cấu hình WebView2 đổi
**And** phép cộng với dung lượng database **không** thuộc story này — CI ở đây không tải dữ liệu từ điển; phép đối chiếu tổng với trần NFR6 đóng ở **Story 1.9**

> ⚠️ **AC trên đã sửa 2026-08-03 sau rà soát.** Bản đầu tiên của nó đòi *"dựng **bản phát hành**"* và *"chênh lệch cộng với **dung lượng database**"* — cả hai đều mâu thuẫn trực tiếp với hai AC sẵn có của chính story này (*"đây **không phải** FR107 — không build công khai"* và *"**không tải dữ liệu từ điển**"*), và Story 1.3 còn chạy **trước** Story 1.9 nên chưa có `dict-core.db` để cộng. AC đã thu về đúng thứ CI push-time làm được: chỉ đo chênh lệch.

**Given** dữ liệu từ điển ~320 MB cho ba tệp `.db` *(số thật 2026-08-05)*
**When** CI chạy ở epic này
**Then** **không tải dữ liệu từ điển** — job chỉ biên dịch và chạy các test không phụ thuộc dữ liệu
**And** thời gian chạy đủ ngắn để không ai muốn tắt nó đi

---

### Story 1.4: Bộ token màu và chữ hai theme, có kiểm tương phản tự động

**Covers:** NFR17 · AD-34 · UX-DR1, UX-DR2, UX-DR3, UX-DR5, UX-DR6, UX-DR10, UX-DR11, UX-DR14, UX-DR16

As a người dựng,
I want mọi màu và mọi cỡ chữ đến từ một bộ token đã kiểm tương phản,
So that một lần đổi nhầm không thể âm thầm đẩy chữ xuống dưới WCAG AA.

**Acceptance Criteria:**

**Given** bốn bảng token ở `DESIGN.md` — *Bảng token màu*, *Bảng token typography*, *Bảng token khoảng cách và hình dạng*
**When** `src/tokens/` dựng xong
**Then** có đủ **16 token màu cho theme sáng và 16 cho theme tối**, **14 token typography**, **bốn họ chữ** (`read` · `read-cjk` · `ui` · `mono`), bộ spacing và bộ rounded
**And** mỗi token khớp **đúng giá trị** ở bảng, không phải một giá trị gần đúng

**Given** một component bất kỳ
**When** lint chạy
**Then** giá trị màu viết thẳng trong component bị từ chối

**Given** mọi cặp (màu chữ, màu nền) dùng trong ứng dụng
**When** chạy kiểm tương phản tự động
**Then** tất cả đạt WCAG AA ở **cả hai** theme
**And** `#7d766c`, `ornament` và `tm-rule` không xuất hiện làm màu chữ ở bất kỳ đâu

**Given** một khối chữ cần lùi ra sau
**When** cài đặt
**Then** lùi bằng cách đổi sang token `on-surface-variant`
**And** `opacity` ở trạng thái nghỉ chỉ áp cho nét và nền, không áp cho chữ

**Given** mọi token họ `read`
**When** kiểm
**Then** không token nào có `line-height` dưới **1.66**
**And** token họ `ui` dùng cho nhãn có khả năng xuống dòng cũng ở 1.66

**Given** theme tối
**When** hai panel đứng cạnh nhau
**Then** phân tách bằng **khe 2px** để `background` lộ ra, panel bo `3px`
**And** ở theme sáng phân tách bằng **đường kẻ 1px** `outline`

**Given** toàn bộ giao diện
**When** kiểm
**Then** không có bóng đổ, gradient hay lớp nổi nào ngoài bóng cửa sổ do hệ điều hành vẽ

---

### Story 1.5: Tài nguyên chuỗi giao diện và hình dạng lỗi qua IPC

**Covers:** NFR16 · AD-21

As a người dựng,
I want không một chuỗi hiển thị nào nằm trong mã nguồn, kể cả chuỗi lỗi,
So that thêm một ngôn ngữ về sau không phải rà lại toàn bộ codebase.

**Acceptance Criteria:**

**Given** `src/i18n/vi.json`
**When** ứng dụng chạy
**Then** mọi chuỗi hiển thị phân giải từ file đó theo khoá chấm

**Given** mã `.rs` và mã `.vue`
**When** grep chuỗi tiếng Việt
**Then** không tìm thấy kết quả nào

**Given** một lỗi phát sinh ở Rust
**When** lỗi đi qua ranh giới IPC
**Then** nó mang hình dạng `{ code, message_key, params, retryable }`
**And** không mang văn bản hiển thị

**Given** một `message_key` không có trong `vi.json`
**When** frontend phân giải
**Then** hiển thị khoá đó nguyên văn và ghi cảnh báo, không sập

**Given** quy tắc giọng văn ở UX-DR47
**When** soạn chuỗi trạng thái
**Then** câu viết ở dạng vô nhân xưng, không xưng *"chúng tôi"*, không gọi người dùng là *"bạn"*
**And** thông báo lỗi nêu nguyên nhân thay vì đổ lỗi người dùng

---

### Story 1.6: CommandRegistry, ba chế độ, và tiêu điểm bàn phím

**Covers:** FR22

As a người dịch,
I want mọi thao tác của ứng dụng gọi được bằng bàn phím và luôn thấy rõ mình đang ở đâu,
So that một phiên làm việc dài không bắt tôi rời tay khỏi bàn phím.

**Acceptance Criteria:**

**Given** `CommandRegistry`
**When** một thao tác được thêm vào ứng dụng
**Then** nó đăng ký ở đó **trước** khi bind vào chuột hoặc phím
**And** handler chuột chỉ được `dispatch` một command đã đăng ký, không tự cài đặt thao tác tại chỗ

**Given** một command id
**When** đăng ký
**Then** dùng khoá chấm có tiền tố miền, cùng hình dạng khoá `vi.json`
**And** hai command trùng id bị phát hiện lúc đăng ký, không ghi đè im lặng

**Given** ba command `mode.library`, `mode.workspace`, `mode.reading`
**When** ứng dụng khởi động
**Then** cả ba đã đăng ký và gọi được bằng `⌘1` `⌘2` `⌘3`, kể cả khi Library và Chế độ đọc chưa có nội dung
**And** chúng là ba chế độ **ngang hàng** trong **một** cửa sổ hệ điều hành

**Given** mỗi chế độ và mỗi panel
**When** được kích hoạt
**Then** nó dời focus DOM tường minh tới điểm vào đã khai
**And** focus không bao giờ rơi về `body`

**Given** một panel có tiêu điểm
**When** quan sát
**Then** có vạch dọc 2px `primary` ở mép trái và tiêu đề chuyển `primary` in đậm
**And** không dùng viền bao quanh để báo tiêu điểm

**Given** `CommandRegistry`
**When** truy vấn
**Then** liệt kê được danh sách thao tác **chưa gán phím nào**

---

### Story 1.7: Tầng ghi dữ liệu — một writer nối tiếp và lược đồ có phiên bản

**Covers:** AD-11 · AD-12 · AD-30 · NFR10

As a người dịch,
I want ứng dụng không bao giờ khựng lại vì đang ghi dữ liệu và không bao giờ làm hỏng dữ liệu cũ khi tôi nâng cấp,
So that tôi tin được vào một công cụ mình dùng hàng năm.

**Acceptance Criteria:**

**Given** mỗi kho ghi được
**When** mở
**Then** có **đúng một** kết nối ghi đặt sau một hàng đợi nối tiếp
**And** đọc dùng pool nhiều kết nối song song trên WAL

**Given** bất kỳ module nào cần ghi
**When** thực hiện
**Then** đi qua `store::Writer` của kho tương ứng
**And** không module nào tự mở được kết nối ghi — cưỡng chế bằng test hoặc bằng khả năng hiển thị của kiểu

**Given** `global.db`
**When** khởi tạo
**Then** `PRAGMA journal_mode = WAL`, `wal_autocheckpoint = 0`, `busy_timeout` đặt tường minh

**Given** một luồng nền trên **kết nối riêng**
**When** người dùng ngừng thao tác một khoảng
**Then** gọi `wal_checkpoint(PASSIVE)`
**And** khi thoát ứng dụng gọi `wal_checkpoint(TRUNCATE)`

**Given** thao tác liên tục hàng giờ
**When** theo dõi `.db-wal`
**Then** kích thước không phình vô hạn — có ngưỡng kích thước buộc checkpoint

**Given** `global.db` mang số phiên bản lược đồ
**When** mở bằng một bản ứng dụng mới hơn
**Then** chạy các bước di trú **chỉ tiến** trong một giao dịch, sau khi đã sao lưu

**Given** một database mang phiên bản lược đồ **mới hơn** ứng dụng
**When** mở
**Then** ứng dụng **từ chối mở** và báo rõ
**And** không ghi vào nó một byte nào

---

### Story 1.8: Phân giải cấu hình hai tầng

**Covers:** FR103

As a người dịch,
I want cấu hình riêng của một Tác phẩm đè lên cấu hình chung một cách nhất quán ở mọi nơi,
So that tôi không phải nhớ chỗ nào theo luật nào.

**Acceptance Criteria:**

**Given** mọi phân giải hai tầng trong hệ thống
**When** xảy ra
**Then** đi qua đúng một `ScopeResolver`

**Given** loại dữ liệu Glossary, Prompt, Cấu hình AI, Tên người dịch
**When** phân giải
**Then** ngữ nghĩa là **ghi đè** — tầng Tác phẩm thắng

**Given** loại dữ liệu Translation Memory và Luật làm sạch khi nhập
**When** phân giải
**Then** ngữ nghĩa là **hợp nhất** — cả hai tầng cùng áp

**Given** một loại dữ liệu mới được thêm vào
**When** đăng ký với `ScopeResolver`
**Then** phải khai ngữ nghĩa tường minh
**And** không có mặc định ngầm nào

**Given** tầng Global
**When** ứng dụng chạy mà chưa mở Tác phẩm nào
**Then** phím tắt và preset bố cục phân giải được từ `global.db`

---

### Story 1.9: Dựng dữ liệu từ điển lớp nền

**Covers:** FR27 · NFR6

As a chủ dự án,
I want năm nguồn từ điển có giấy phép sạch gộp thành một artifact SQLite có phiên bản và checksum,
So that bản phát hành kiểm chứng được và không parser nào lọt vào sản phẩm.

**Acceptance Criteria:**

**Given** nguồn thô CVDICT, Unihan, CC-CEDICT, viwiktionary, en.wiktionary
**When** `tools/dict-build` chạy
**Then** sinh ra `dict-core.db` theo lược đồ `từ khoá → [nguồn, từ loại, nghĩa, ví dụ[], trích dẫn[], ghi chú]`

**Given** mọi bản ghi nghĩa
**When** ghi vào database
**Then** cột `source` bắt buộc có giá trị
**And** không tồn tại bước hợp nhất nghĩa giữa các nguồn ở bất kỳ đâu trong build tool

**Given** `dict-core.db` đã sinh
**When** đẩy lên một GitHub Release có phiên bản
**Then** `dict-manifest.toml` trong repo ghi URL, SHA-256 và phiên bản nguồn thô của file đó

**Given** parser định dạng từ điển
**When** kiểm cây phụ thuộc của bản phát hành
**Then** không parser nào có mặt — chúng chỉ sống trong `tools/dict-build`

**Given** chỉ mục FTS trên nghĩa
**When** tạo
**Then** chỉ mục **chính** dùng `remove_diacritics 0`
**And** chỉ mục xoá dấu tồn tại như chỉ mục **phụ**, không bao giờ là mặc định

> **Bổ sung 2026-08-03 — nhận bàn giao từ Story 1.1.** Đây là nơi đóng phép đối chiếu NFR6 **thật**, vì đây là story đầu tiên có dữ liệu thật để đo. Mũi thăm dò font đã ăn **21,29 MB** trong ngân sách và con số đó phải được cộng vào chứ không để riêng. Số đo và cách đọc `[A2]`: [`research/font-spike-results-2026-08-03.md`](research/font-spike-results-2026-08-03.md) §Cần Ice quyết.

**Given** **mọi** artifact dữ liệu sẽ đóng gói — `dict-core.db` **và** bốn lớp gỡ rời của Story 1.10, không chỉ `dict-core.db`
**When** đo tổng
**Then** cộng thêm **21,29 MB bộ font** (đo thật ở Story 1.1) và **baseline ứng dụng thật** rồi đối chiếu lại với trần **NFR6**
**And** phép đối chiếu tính trên **payload sản phẩm**; **bản WebView2 Runtime nhúng của `.msi` KHÔNG được cộng vào** — ghi thành dòng riêng *(NFR6 sửa 2026-08-03)*. Cộng nhầm nó vào là làm con số vượt trần vì một lý do NFR6 đã tuyên bố không tính
**And** phép đối chiếu quy về **byte** trước rồi mới đổi sang MB thập phân — 200 MB là **trần**, 150 MB là mốc kỳ vọng chứ không phải điều kiện đạt
**And** nếu vượt trần, đó là quyết định **tầng PRD** — **không** tự subset font, **không** tự bỏ một nguồn từ điển, **không** tự đổi sang font hệ điều hành

---

### Story 1.10: Đóng gói bốn lớp gỡ rời thành file độc lập

**Covers:** FR27, FR36

As a chủ dự án,
I want gỡ một nguồn dữ liệu khỏi bản phát hành chỉ bằng cách xoá một file,
So that chính sách gỡ bỏ thực thi được mà không đổi một dòng mã.

**Acceptance Criteria:**

**Given** Thiều Chửu, Cổ hán văn, VietPhrase, HVTĐTD
**When** `tools/dict-build` chạy
**Then** mỗi nguồn sinh ra **một file `.db` độc lập**, không gộp vào `dict-core.db`

**Given** mỗi file `.db` lớp gỡ rời
**When** mở
**Then** nó tự mang metadata giấy phép và ghi công của chính nó

**Given** trường giấy phép trong metadata
**When** mô hình hoá
**Then** biểu diễn được **cả** giấy phép mở (CC-BY-SA, Unicode License) **lẫn** phép sử dụng riêng do tác giả cấp
**And** HVTĐTD mang nhãn *"phép riêng của tác giả, không thuộc GPL v3"*, không bị ép vào một enum các giấy phép mở

**Given** runtime đọc một nguồn bất kỳ
**When** thực hiện
**Then** không có mã riêng cho từng nguồn — mọi nguồn đi qua cùng một đường

**Given** `dict-manifest.toml`
**When** kiểm
**Then** có một mục cho từng file `.db` kèm URL và SHA-256

---

### Story 1.10b: Dựng dữ liệu từ điển tiếng Anh

> ➕ **Story THÊM 2026-08-05 qua `correct-course`** — xem `sprint-change-proposal-2026-08-05.md`.
> **Vì sao nó thiếu:** `viwiktionary` mang HAI VAI trên cùng một tệp thô (`lang_code=en` cho FR34 · `lang_code=zh` cho lớp từ loại ZH), nhưng PRD §8.2/§8.3 chưa bao giờ nói đó là hai vai song song, nên Story 1.9 chỉ cài vai B. PRD đã sửa hết mơ hồ 2026-08-05.

**Covers:** FR34 · NFR6 *(đo lại)* · NFR8

As a chủ dự án,
I want đầu mục tiếng Anh có mặt trong dữ liệu từ điển đóng gói,
So that cặp Anh → Việt có nền dữ liệu như cặp Trung → Việt đã có.

🟢 **Dữ liệu ĐÃ ĐO THẬT** (mũi thăm dò 2026-08-05, đã hoàn tác khỏi cây mã): **119.039** đầu mục · **190.543** nghĩa · **27.396** ví dụ · **40.333.312** byte. Nguồn thô **đã nằm trong `raw/viwiktionary/`** — không phải tải gì thêm.

**Given** `raw/viwiktionary/vi-extract.jsonl` đã có · **When** dựng dữ liệu từ điển
**Then** `viwiktionary` vai A dựng thành nguồn thứ **sáu**, dùng lại `wiktextract_common::parse(reader, "vi", Some("en"))` — không parser mới, không crate mới
**And** đối chiếu số đo: **119.039** đầu mục · **190.543** nghĩa · **27.396** ví dụ — lệch quá **1%** ⇒ parser sai, không phải *"nguồn vốn thế"*
**And** mọi mục của nguồn này mang `dict_entry.lang = 'en'`; **đối chứng âm bắt buộc:** nguồn này sinh **0** hàng `lang='zh'`
**And** `dict_source` mang đủ bốn trường giấy phép, `attribution` nêu Wiktionary tiếng Việt + kaikki.org + CC-BY-SA 4.0 và GFDL
**And** bảng kế toán **NFR6** cập nhật với số **thật** và đối chiếu trần **400.000.000 byte**
**And** `check-dict-build.mjs` (Kiểm C/D/E/F) đi theo nguồn mới; `RS_FILE_FLOOR` cập nhật nếu số tệp `.rs` đổi

🔴 **Quyết định phải chốt TRONG story:** lớp này vào **`dict-core.db`** *(nguồn nền — khuyến nghị: giấy phép sạch cùng loại với `viwiktionary` vai B, và FR34 thuộc phạm vi lõi)* hay thành **tệp `.db` riêng**? ⚠️ Nếu vào `dict-core.db` thì **phải dựng lại** tệp đó và điền lại `[base].sha256` — nay rẻ và tái lập được sau bản vá `built_at` của Story 1.10.

**Không** chạm `src-tauri/**`. **Không** đổi một dòng DDL nào của `schema.rs`. Đường tra cứu là **Story 1.11b**.

---

### Story 1.11: Ba nhánh truy vấn tiếng Trung

**Covers:** FR39 · NFR1

As a người dịch,
I want tra một chữ Hán đơn hay một từ hai chữ đều ra kết quả,
So that công cụ không im lặng trả về rỗng ở đúng những từ tôi tra nhiều nhất.

**Acceptance Criteria:**

**Given** một truy vấn tra chính xác đầu mục
**When** thực hiện
**Then** đi qua chỉ mục B-tree

**Given** một truy vấn chuỗi con **1 hoặc 2 ký tự**
**When** thực hiện
**Then** đi qua bảng đảo ngược `char_idx`

**Given** một truy vấn chuỗi con **từ 3 ký tự trở lên**
**When** thực hiện
**Then** đi qua FTS5 `trigram`

**Given** truy vấn `山`
**When** tra
**Then** trả về kết quả **khác rỗng**, đối chiếu được với số đo Giai đoạn 0

**Given** truy vấn `中國`
**When** tra
**Then** trả về kết quả **khác rỗng**

**Given** truy vấn `中國人`
**When** tra
**Then** trả về kết quả khác rỗng

**Given** toàn bộ đường tra cứu nóng
**When** rà mã
**Then** không có câu lệnh `LIKE` nào

**Given** ba nhánh truy vấn
**When** đo p95 phía backend
**Then** nằm trong ngân sách backend của NFR1

---

### Story 1.11b: Đường tra cứu tiếng Anh

> ➕ **Story THÊM 2026-08-05 qua `correct-course`.**
> 🔴 **CHẶN:** cần một **AD mới** cho đường tra cứu tiếng Anh trước khi bắt đầu — chủ sở hữu **Winston**. `AD-26` tên đầy đủ là *"Ba nhánh truy vấn **tiếng Trung**"*, và nhánh `char_idx` không áp được cho tiếng Anh: mũi thăm dò sinh đúng **9** cặp `char_idx` trên **119.039** đầu mục.

**Covers:** FR34 · FR19 · FR40 *(dùng chung `Matcher` với Story 1.12)*

As a người dịch,
I want bôi đen một từ tiếng Anh và thấy ngay nghĩa tiếng Việt kèm từ loại,
So that cặp Anh → Việt dùng được thật, không chỉ có trong tài liệu.

**Given** dữ liệu tiếng Anh đã đóng gói *(Story 1.10b)* · **When** tra một từ tiếng Anh
**Then** đi qua **cùng cổng `DictionarySource`** của Story 1.13 — không mã riêng cho từng ngôn ngữ, ngoài phần **chiến lược truy vấn** mà AD mới quy định
**And** biến thể hình thái *(FR40, **stemming** — không phải lemmatization)* dùng `Matcher` của **AD-17**, không cài riêng một bản thứ hai
**And** mục từ tiếng Anh hiển thị **nhãn từ loại** + **nghĩa tiếng Việt** *(FR34)*, ghi rõ nguồn, không hợp nhất *(AD-19)*
**And** **NFR1** *(< 100 ms)* đo **trên đường tiếng Anh**, không suy ra từ số đo tiếng Trung

---

### Story 1.12: Matcher dùng chung

**Covers:** FR40

As a người dựng,
I want một component khớp ngôn ngữ duy nhất phục vụ cả từ điển, Glossary và Translation Memory,
So that ba nơi không bao giờ bắt được những biến thể khác nhau mà không ai hiểu vì sao.

**Acceptance Criteria:**

**Given** `core/matching/`
**When** kiểm
**Then** tồn tại **đúng một** cài đặt khớp ngôn ngữ
**And** `dict/` dùng nó; `glossary/` và `tm/` sẽ dùng chính nó ở các epic sau, không cài lại

**Given** văn bản tiếng Trung
**When** khớp
**Then** dùng khớp chính xác và n-gram ký tự, tách từ qua `jieba-rs` khi cần

**Given** văn bản tiếng Anh
**When** khớp
**Then** stemming rồi token n-gram

**Given** một từ tiếng Anh ở dạng biến thể hình thái
**When** tra cứu
**Then** nhận diện được về dạng gốc

**Given** một biến thể bất quy tắc
**When** tra cứu
**Then** giới hạn được ghi lại tường minh — đây là *stemming*, không phải *lemmatization*

---

### Story 1.13: Đường tra cứu giữ nguyên bất đồng giữa các nguồn

**Covers:** FR29, FR30, FR31, FR32, FR34, FR35

As a người dịch,
I want thấy mỗi định nghĩa đến từ đâu và thấy các nguồn nói khác nhau,
So that tôi tự phán xét thay vì tin một câu trả lời đã bị gộp lại.

**Acceptance Criteria:**

**Given** cổng `DictionarySource`
**When** định nghĩa
**Then** mỗi file `.db` là một adapter đi qua cùng một cổng
**And** runtime không có mã riêng cho từng nguồn

**Given** một truy vấn
**When** trả kết quả
**Then** kết quả nhóm **theo từng nguồn**, giữ nguyên bất đồng
**And** trong toàn hệ thống không tồn tại hàm hợp nhất nghĩa giữa các nguồn

**Given** một từ có nhiều từ loại
**When** trả kết quả
**Then** thành nhiều mục riêng biệt, mỗi mục có ví dụ riêng

**Given** một mục từ
**When** trả kết quả
**Then** ví dụ gắn với **từng từ loại**
**And** trích dẫn là trường riêng biệt với ví dụ, có xuất xứ văn bản

**Given** một mục từ tiếng Anh
**When** trả kết quả
**Then** có nhãn từ loại và nghĩa tiếng Việt

**Given** một mục từ tiếng Trung khi chỉ có các lớp nền
**When** trả kết quả
**Then** có nhãn từ loại và ít nhất một ví dụ khi nguồn có dữ liệu
**And** nhãn tiếng Anh được đánh dấu rõ là **nhãn ngoại ngữ**

**Given** lớp HVTĐTD được bật
**When** tra một mục Hán Việt
**Then** hiển thị từ loại, ví dụ và trích dẫn **bằng tiếng Việt**

**Given** xoá file `.db` của một lớp gỡ rời bất kỳ khỏi `resources/dict/`
**When** chạy lại **toàn bộ** bộ test tra cứu
**Then** tất cả vẫn xanh
**And** mục từ Hán Việt rơi về nhãn tiếng Anh của lớp nền, không có đường tra cứu nào hỏng

---

### Story 1.14: Khung bốn panel

**Covers:** FR16, FR17, FR18

As a người dịch,
I want bốn panel trong một cửa sổ duy nhất và sắp xếp được theo cách tôi làm việc,
So that tôi không phải mở bốn năm cửa sổ rời như trước.

**Acceptance Criteria:**

**Given** Workspace
**When** mở
**Then** bốn slot panel *Source*, *Lookup*, *AI Translation*, *Editor* tồn tại trong **một** cửa sổ hệ điều hành duy nhất

**Given** một panel
**When** kéo thả
**Then** dock, undock, gộp thành tab và đổi kích thước được

**Given** một panel bất kỳ
**When** người dùng chọn ẩn
**Then** nó **ẩn hoàn toàn**
**And** các panel còn lại lấp đầy chỗ trống

**Given** bố cục hiện tại
**When** đóng rồi mở lại ứng dụng
**Then** bố cục khôi phục nguyên trạng

**Given** nhiều preset bố cục đã lưu
**When** người dùng chuyển
**Then** chuyển được bằng phím qua `CommandRegistry`

**Given** preset mặc định
**When** mở lần đầu
**Then** là lưới 2×2 — `Nguyên văn | Bản dịch` ở hàng trên, `Tra cứu | Đề xuất AI` ở hàng dưới

**Given** UX-DR15 khai **thứ tự hy sinh panel** là quyết định, không phải số hiệu chỉnh được
**When** dựng cơ chế ẩn/hiện panel
**Then** cơ chế phải cho phép ẩn theo đúng thứ tự đó — **Đề xuất AI nhường trước · Tra cứu nhường sau nhưng rút về thanh trạng thái, không bao giờ mất hẳn · cặp `Nguyên văn | Bản dịch` không bao giờ nhường**
**And** **ngưỡng kích thước cụ thể** đóng ở Story 4.12, không đóng ở đây
**And** không được cài cơ chế ẩn theo cách khiến Story 4.12 phải mổ lại bố cục để nhét thứ tự này vào

**Given** panel AI Translation và Editor chưa có nội dung ở epic này
**When** hiển thị
**Then** chúng nêu rõ trạng thái bằng chuỗi trong `vi.json`, không phải một khung trống không giải thích

---

### Story 1.15: Tác phẩm trên đĩa và đường vào văn bản tối thiểu

**Covers:** FR13, FR96, FR97, FR102

As a người dịch,
I want đưa một đoạn văn bản vào công cụ và biết chắc nó nằm trong một thư mục tôi copy đi được,
So that dữ liệu của tôi không bị khoá trong ứng dụng.

**Acceptance Criteria:**

**Given** người dùng dán văn bản trực tiếp, hoặc mở một file `.txt` hay `.md`
**When** xác nhận
**Then** một Tác phẩm được tạo với **đúng một Chương** chứa văn bản nguồn

**Given** một Tác phẩm
**When** ghi xuống đĩa
**Then** nó là một **thư mục** `<Tên>.atproj/` chứa `meta.json`, `project.db` và `assets/`

**Given** `meta.json`
**When** đọc
**Then** lấy được metadata của Tác phẩm **không cần mở SQLite**

**Given** `work.id`
**When** tạo
**Then** là UUID v4 lưu trong `meta.json`
**And** `chapter.id` và `segment.id` là số nguyên **cục bộ** trong `project.db`

**Given** một `.atproj`
**When** copy sang máy khác và mở
**Then** mở được nguyên vẹn

**Given** người dùng muốn sao lưu
**When** copy thư mục `.atproj`
**Then** bản sao đó dùng được ngay, không cần một thao tác export riêng

**Given** `meta.json` và `project.db`
**When** tạo
**Then** mỗi cái mang một số phiên bản lược đồ

**Given** người dùng chọn một file `.docx`
**When** ở epic này
**Then** màn hình báo rõ định dạng chưa nhận ở phiên bản hiện tại
**And** không sập và không nhập vào một phần

---

### Story 1.16: Panel Source và tab Hán Việt

**Covers:** FR19, FR33

As a người dịch tiếng Trung,
I want thấy âm Hán Việt của từng ký tự ngay cạnh nguyên văn,
So that tôi đọc được văn bản mà không phải tra từng chữ một.

**Acceptance Criteria:**

**Given** một Chương có văn bản nguồn
**When** mở Workspace
**Then** Panel Source hiển thị nguyên văn bằng token `source-cjk`
**And** dùng họ chữ `read`, không dùng họ `ui`

**Given** ngôn ngữ nguồn của Tác phẩm là tiếng Trung
**When** Panel Source hiển thị
**Then** có **tab Hán Việt**

**Given** tab Hán Việt
**When** bật
**Then** hiển thị âm Hán Việt cho **từng ký tự** tiếng Trung, đọc từ dữ liệu đã nhúng
**And** hoạt động khi ngắt kết nối mạng

**Given** tab Hán Việt
**When** người dùng chọn kiểu xem
**Then** xem được ở chế độ **chuyển đổi** hoặc **song song**

**Given** ngôn ngữ nguồn là tiếng Anh
**When** Panel Source hiển thị
**Then** không có tab Hán Việt

---

### Story 1.17: Panel Lookup — bản ghi có cấu trúc

**Covers:** FR28, FR32

As a người dịch,
I want kết quả tra cứu hiện thành bản ghi có cấu trúc chứ không phải một đoạn văn,
So that mắt tôi nhặt được thứ cần trong một giây.

**Acceptance Criteria:**

**Given** một kết quả tra cứu
**When** hiển thị
**Then** mỗi nguồn là một khối riêng **xếp chồng dọc**, có vạch trái 2px và thụt 13px
**And** không bao giờ gộp các nguồn

**Given** nhãn nguồn
**When** hiển thị
**Then** dùng `ui-label` màu `primary`

**Given** trích dẫn
**When** hiển thị
**Then** có vạch trái `primary` để phân biệt với ví dụ

**Given** hai nguồn ghi khác nhau về cùng một mục từ
**When** hiển thị
**Then** một dòng dẫn nói rõ điều đó **trước khi** liệt kê

**Given** tra cứu không tìm thấy kết quả
**When** hiển thị
**Then** gợi ý tra **từng chữ** trong cụm vừa chọn
**And** **không trỏ tới bất kỳ năng lực nào chưa tồn tại** — đường sang Concordance được bổ sung ở Story 7.7

**Given** người dùng chưa tra gì
**When** hiển thị
**Then** dạy thao tác bôi đen — chuỗi này **khác** chuỗi không tìm thấy

**Given** một lần tra đang chạy
**When** hiển thị
**Then** không có spinner và không có trạng thái *"đang tải"*

**Given** vùng đầu mục của Panel Lookup
**When** kết quả đổi
**Then** chiều cao **cố định** — đầu mục và thanh nhịp giữ nguyên toạ độ, chỉ phần dưới thay đổi

---

### Story 1.18: Auto-Lookup

**Covers:** FR21

As a người dịch,
I want bôi đen một cụm từ là thấy ngay nghĩa của nó,
So that tôi không phải copy, paste hay chuyển cửa sổ hàng trăm lần mỗi Chương.

**Acceptance Criteria:**

**Given** người dùng bôi đen một cụm từ ở Panel Source
**When** thả chuột hoặc kết thúc vùng chọn bằng bàn phím
**Then** kết quả tra cứu hiện ở Panel Lookup mà không có thao tác nào khác

**Given** cơ chế Auto-Lookup
**When** đăng ký
**Then** nó gắn vào một **hợp đồng vùng chọn dùng chung cho mọi panel văn bản**
**And** Panel AI Translation và Editor nhận được cùng hành vi khi chúng có nội dung ở các epic sau, không cần cài lại

**Given** vùng chọn đang được kéo
**When** chưa dừng
**Then** chưa tra

**Given** độ trễ đầu-cuối từ lúc thả chuột tới lúc kết quả hiển thị
**When** đo trên ít nhất 100 lần tra liên tiếp
**Then** **p95 dưới 100 ms**

**Given** nội dung mới vào Panel Lookup
**When** hiển thị
**Then** hiệu ứng 90 ms, `opacity` 0.4 → 1, `ease-out`
**And** không dùng `translate`, không dùng `scale`

**Given** một lần tra mới trong lúc hiệu ứng đang chạy
**When** xảy ra
**Then** hiệu ứng cũ bị **huỷ** và `opacity` đặt thẳng về 1
**And** hiệu ứng không xếp hàng

**Given** kết quả mới đến
**When** Panel Lookup cập nhật
**Then** vị trí cuộn về đầu **tức thì**, không cuộn có hiệu ứng

**Given** một lần tra vượt 250 ms
**When** xảy ra
**Then** hiện vạch tiến trình mảnh ở đáy vùng đầu mục, **không dùng spinner**

**Given** `prefers-reduced-motion` bật
**When** bất kỳ hiệu ứng nào
**Then** bị bỏ hoàn toàn, đổi tức thì

**Given** ngắt kết nối mạng
**When** tra cứu
**Then** mọi đường tra cứu vẫn hoạt động đầy đủ

---

### Story 1.19: Bật tắt nguồn từ điển và ghi công

**Covers:** FR37, FR38

As a người dịch,
I want tắt một nguồn từ điển tôi không tin và luôn thấy được ghi công đầy đủ,
So that tôi kiểm soát được thứ mình đang đọc.

**Acceptance Criteria:**

**Given** danh sách nguồn từ điển
**When** mở trong Panel Lookup
**Then** người dùng bật/tắt được **từng nguồn**

**Given** một nguồn bị tắt
**When** tra cứu
**Then** kết quả từ nguồn đó không xuất hiện
**And** các nguồn còn lại không đổi

**Given** màn hình Attribution
**When** mở
**Then** liệt kê mọi nguồn **có mặt** kèm giấy phép và ghi công đầy đủ

**Given** danh sách nguồn
**When** dựng
**Then** dựng từ các file `.db` thực sự có mặt trong `resources/dict/`, không từ một danh sách viết cứng

**Given** một file `.db` bị xoá khỏi bản cài
**When** mở màn hình Attribution
**Then** ghi công của nguồn đó cũng biến mất
**And** không để lại ghi công mồ côi

**Given** lớp HVTĐTD
**When** hiển thị trong Attribution
**Then** ghi rõ © Đặng Thế Kiệt, dùng theo phép riêng tác giả cấp, **không thuộc GPL v3**

---

### Story 1.20: Lịch sử tra cứu và mục đã ghim

**Covers:** FR41

As a người dịch,
I want xem lại những gì mình vừa tra và ghim những mục hay dùng,
So that tôi không phải tra lại cùng một chữ năm lần trong một Chương.

**Acceptance Criteria:**

**Given** người dùng tra nhiều lần trong một phiên
**When** mở tab lịch sử
**Then** thấy các lần tra theo thứ tự gần nhất trước

**Given** một mục từ trong kết quả tra cứu
**When** người dùng ghim
**Then** nó xuất hiện trong danh sách đã ghim

**Given** một mục đã ghim
**When** đóng rồi mở lại ứng dụng
**Then** nó vẫn còn

**Given** lịch sử tra cứu
**When** đóng ứng dụng
**Then** lịch sử của phiên kết thúc — đây là lịch sử **trong phiên**, khác với mục ghim

**Given** lịch sử và mục ghim
**When** hiển thị
**Then** là **tab thứ ba** của Panel Lookup, không phải một cửa sổ riêng

**Given** mọi thao tác của tính năng này
**When** gọi
**Then** có command đăng ký trong `CommandRegistry` và gán được phím

---

### Story 1.21: Phím tắt cấu hình lại được

**Covers:** FR22

As a người dịch,
I want đổi mọi phím tắt theo thói quen của mình,
So that công cụ chạy theo tay tôi chứ không ngược lại.

**Acceptance Criteria:**

**Given** danh sách command đã đăng ký
**When** mở màn hình phím tắt
**Then** **mọi** command hiện ra kèm phím đang gán

**Given** một command
**When** người dùng gán phím khác
**Then** thay đổi có hiệu lực ngay và lưu ở tầng Global

**Given** hai command được gán cùng một phím
**When** xảy ra
**Then** xung đột hiện ra cho người dùng giải quyết
**And** không im lặng ghi đè

**Given** phím tắt đã đổi
**When** mở lại ứng dụng
**Then** giữ nguyên

**Given** `CommandRegistry`
**When** truy vấn từ màn hình phím tắt
**Then** liệt kê được các command **chưa gán phím nào**

**Given** một vòng thao tác trong phạm vi epic này — mở Tác phẩm, chuyển panel, bôi đen tra cứu, bật tắt nguồn, ghim một mục, chuyển chế độ
**When** thực hiện
**Then** làm được **hoàn toàn bằng bàn phím, không chạm chuột một lần nào**

---

## Epic 2: Biên tập theo segment — một vòng dịch tay hoàn chỉnh

Người dịch dịch trọn một Chương **bằng tay, không cần AI và không cần Glossary**: văn bản tách thành segment cấp câu, gộp hoặc tách khi máy tách sai, xác nhận từng câu với vạch lề đổi màu, điều hướng tới segment chưa dịch kế tiếp, chuyển Chương ngay trong Workspace. Sập ứng dụng giữa phiên gõ **mất tối đa 5 giây công việc**, và không frame nào vượt 50 ms trong lúc auto-save chạy. Mọi phiên bản cũ của một segment xem lại và khôi phục được.

### Story 2.1: Tách segment cấp câu và cờ kết đoạn

**Covers:** FR23

As a người dịch,
I want văn bản được chia thành từng câu ngay khi nhập và ranh giới đó ổn định mãi mãi,
So that lịch sử và trạng thái công việc của tôi không bao giờ trỏ sai chỗ.

**Acceptance Criteria:**

**Given** văn bản tiếng Trung
**When** tách segment
**Then** tách theo `。！？；`

**Given** văn bản tiếng Anh
**When** tách segment
**Then** tách theo `. ! ?` có xử lý các trường hợp viết tắt không phải kết câu

**Given** một Chương được nhập
**When** tách segment
**Then** kết quả **lưu xuống** `project.db`
**And** không đường mã nào tính lại ranh giới lúc nạp Chương

**Given** mỗi segment
**When** tạo
**Then** mang `segment.id` bất biến
**And** thứ tự trong Chương là cột riêng `ord`, sắp lại được mà không đụng `id`

**Given** một `segment.id` đã về hưu
**When** cấp id mới
**Then** id đó **không bao giờ** được tái dùng

**Given** mỗi segment
**When** tạo
**Then** mang **cờ kết đoạn** tính cùng lượt với ranh giới câu và lưu xuống đĩa
**And** là **một cờ duy nhất dùng chung** cho cả nguyên văn và bản dịch

**Given** segment cuối cùng của một Chương
**When** tính cờ kết đoạn
**Then** cờ **tắt, luôn luôn**

**Given** quy tắc tách câu được cải thiện về sau
**When** áp dụng
**Then** chỉ áp qua thao tác **tái tách chủ động** của người dùng trên từng Chương, kèm cảnh báo về dữ liệu sẽ về hưu
**And** không có đường nào tự động tách lại toàn bộ Thư viện

---

### Story 2.2: Panel Editor liền mạch

**Covers:** UX-DR13 · AD-1

As a người dịch,
I want gõ trên một trang văn bản liền chứ không phải một cái bảng, mà vẫn đọc được trạng thái từng câu,
So that tôi viết tự do trong khi sổ sách segment vẫn sạch.

**Acceptance Criteria:**

**Given** Panel Editor
**When** hiển thị
**Then** văn bản là một trang liền mạch — **không ô, không bảng, không khối**

**Given** trạng thái của từng segment
**When** hiển thị
**Then** đọc ở vạch lề dọc 2px trong máng rộng 22px bên trái, cao đúng bằng câu tương ứng
**And** đây là **cách duy nhất** trạng thái segment được hiển thị

**Given** năm giá trị trạng thái
**When** hiển thị
**Then** `confirmed` đã xác nhận · `primary` đang sửa · `tm-rule` điền sẵn từ TM chưa xác nhận · **không vạch** chưa dịch · `ornament` mờ đã về hưu

**Given** ranh giới câu ở trạng thái nghỉ
**When** hiển thị
**Then** ký tự `⏐` màu `ornament` ở `opacity: 0`

**Given** con trỏ chuột rê qua hoặc tiêu điểm bàn phím chạm tới một câu
**When** xảy ra
**Then** ranh giới câu hiện ở `opacity: 0.55`

**Given** văn bản trong Editor
**When** hiển thị
**Then** dùng token `editor` họ `read`, giãn dòng 1.95

**Given** Panel Editor
**When** nhận tiêu điểm
**Then** dời focus DOM tường minh tới điểm vào đã khai

---

### Story 2.3: Hợp đồng flush và trạng thái đã lưu

**Covers:** FR100

As a người dịch,
I want không bao giờ mất quá năm giây công việc dù ứng dụng có sập giữa lúc tôi đang gõ,
So that tôi không phải bận tâm về việc lưu.

**Acceptance Criteria:**

**Given** người dùng ngừng gõ khoảng 2 giây
**When** xảy ra
**Then** văn bản Editor flush xuống Rust

**Given** người dùng gõ **liên tục không nghỉ**
**When** 5 giây trôi qua kể từ lần flush trước
**Then** flush vẫn xảy ra
**And** đồng hồ trần này **không được reset bởi phím gõ**

**Given** người dùng xác nhận segment, rời segment, đóng Tác phẩm, hoặc thoát ứng dụng
**When** xảy ra
**Then** flush xảy ra ngay

**Given** một flush
**When** thực hiện
**Then** đi qua **đúng `store::Writer` nối tiếp**
**And** không mở kết nối riêng

**Given** một flush
**When** được coi là hoàn tất
**Then** chỉ sau khi **đã ghi vào WAL**, không phải khi mới vào hàng đợi trong bộ nhớ

**Given** một flush do auto-save
**When** hoàn tất
**Then** **không** tạo `SegmentVersion` và **không** đổi trạng thái segment

**Given** lần flush gần nhất
**When** hiển thị
**Then** thanh trạng thái ghi *"Đã lưu N giây trước"*
**And** không có hộp thoại và không có dấu chấm *"chưa lưu"*

---

### Story 2.4: Mũi thăm dò — đo NFR18 và NFR2 đồng thời

**Covers:** NFR2 · NFR18 · AD-35 · mũi thăm dò bắt buộc

As a chủ dự án,
I want biết chắc nhịp auto-save đạt được cả hai ngưỡng cùng lúc,
So that tôi không phát hiện ra chúng xung khắc sau khi đã xây tám story lên trên.

**Acceptance Criteria:**

**Given** một phiên gõ liên tục ít nhất 30 phút trên một Chương thật
**When** đo
**Then** **không frame nào vượt 50 ms** trong lúc auto-save chạy (NFR2)

**Given** ứng dụng bị kill cưỡng bức ở nhiều thời điểm ngẫu nhiên trong lúc gõ
**When** mở lại
**Then** công việc mất **tối đa 5 giây** (NFR18)
**And** phép đo lặp lại ít nhất 20 lần với kết quả nhất quán

**Given** ngưỡng kích thước WAL buộc checkpoint
**When** dò
**Then** chọn được một giá trị đạt **cả hai** ngưỡng trên
**And** giá trị đó ghi vào hàng Deferred tương ứng của `ARCHITECTURE-SPINE.md`, đánh dấu đã đóng

**Given** thư viện editor cho Panel Editor
**When** chọn
**Then** lựa chọn được ghi lại kèm lý do
**And** nó tuân hợp đồng trạng thái AD-31 nên không lan ra ngoài module

**Given** hai ngưỡng NFR2 và NFR18 không đạt được đồng thời
**When** xảy ra
**Then** kết quả được báo cáo là **thay đổi tầng PRD cần chủ dự án quyết**, không phải một tối ưu kỹ thuật

---

### Story 2.5: Xác nhận segment và máy trạng thái

**Covers:** FR24

As a người dịch,
I want đánh dấu một câu là đạt chuẩn của mình và thấy nó đổi màu ngay,
So that tôi biết mình đang ở đâu trong một Chương dài.

**Acceptance Criteria:**

**Given** một segment
**When** người dùng xác nhận
**Then** trạng thái chuyển sang **đã xác nhận** và vạch lề chuyển `confirmed`

**Given** một segment chuyển sang đã xác nhận
**When** xảy ra
**Then** tạo **đúng một** `SegmentVersion`

**Given** một segment **đã xác nhận**
**When** người dùng sửa văn bản của nó
**Then** trạng thái **quay về chưa xác nhận**
**And** không tạo `SegmentVersion`

**Given** auto-save chạy
**When** xảy ra
**Then** trạng thái segment không đổi và không tạo `SegmentVersion`

**Given** thao tác xác nhận
**When** gọi
**Then** qua một command đã đăng ký trong `CommandRegistry`, gán phím được

**Given** người dùng tự dịch câu đó hay đang biên tập câu do người khác dịch
**When** xác nhận
**Then** ngữ nghĩa giống nhau — *"câu này đạt chuẩn của tôi"*

---

### Story 2.6: Lịch sử phiên bản segment và khôi phục

**Covers:** FR101

As a người dịch,
I want xem lại các bản dịch trước của một câu và quay về một trong số đó,
So that tôi thử một cách diễn đạt khác mà không sợ mất bản cũ.

**Acceptance Criteria:**

**Given** một segment đã được xác nhận nhiều lần
**When** mở lịch sử
**Then** thấy các phiên bản kèm thời điểm, mới nhất trước

**Given** một phiên bản cũ
**When** người dùng chọn khôi phục
**Then** văn bản đích của segment quay về nội dung đó
**And** trạng thái segment về **chưa xác nhận**

**Given** một segment chưa từng được xác nhận
**When** mở lịch sử
**Then** hiện trạng thái rỗng nêu rõ lịch sử sinh ra khi **xác nhận**, không phải khi gõ

**Given** một segment đã về hưu do gộp hoặc tách
**When** tra lịch sử của nó
**Then** lịch sử vẫn tra lại được

**Given** thời điểm của mỗi phiên bản
**When** lưu
**Then** ISO-8601 UTC trong database, định dạng hiển thị chỉ ở frontend

---

### Story 2.7: Xuất xứ bản dịch cấp segment

**Covers:** FR117

As a người dịch làm cả vai biên tập,
I want hệ thống tự biết câu nào là chữ của tôi và câu nào là của người khác,
So that kho Translation Memory về sau không bị trộn phong cách.

**Acceptance Criteria:**

**Given** người dùng gõ bản dịch rồi xác nhận
**When** ghi xuất xứ
**Then** là **tôi dịch**

**Given** người dùng **sửa** một câu sẵn có rồi xác nhận
**When** ghi xuất xứ
**Then** là **tôi dịch** — câu sau khi sửa là chữ của họ

**Given** người dùng duyệt **nguyên văn** một câu sẵn có, không sửa gì, rồi xác nhận
**When** ghi xuất xứ
**Then** giữ nguyên xuất xứ lúc nạp segment

**Given** hệ thống xác định câu có bị sửa hay không
**When** thực hiện
**Then** so **văn bản đích hiện tại với bản lúc nạp segment**
**And** **không dùng cờ dirty**

**Given** người dùng gõ rồi hoàn tác về đúng nguyên trạng rồi xác nhận
**When** ghi xuất xứ
**Then** coi như **không sửa**

**Given** xuất xứ
**When** ghi
**Then** ghi cùng lúc với chuyển tiếp sang đã xác nhận, không ở chỗ nào khác

**Given** người dùng
**When** dùng tính năng này
**Then** **không có thao tác nào thêm** — hệ thống không hỏi

---

### Story 2.8: Gộp và tách segment tường minh

**Covers:** FR78

As a người dịch,
I want sửa lại chỗ máy tách câu sai,
So that một dấu chấm trong chữ viết tắt không phá cấu trúc cả Chương.

**Acceptance Criteria:**

**Given** hai segment liền nhau
**When** người dùng gộp bằng `⌘M`
**Then** cả hai đánh dấu **về hưu** và một segment mới được tạo

**Given** một segment
**When** người dùng tách bằng `⌘/`
**Then** segment cũ về hưu và các mảnh mới được tạo

**Given** segment mới sinh ra từ gộp hoặc tách
**When** tạo
**Then** bắt đầu ở trạng thái **chưa xác nhận với lịch sử rỗng**

**Given** segment đã về hưu
**When** tra
**Then** lịch sử phiên bản của nó vẫn tra lại được

**Given** cặp TM đã ghi từ segment cũ
**When** gộp hoặc tách xảy ra
**Then** ở lại nguyên, không bị xoá

**Given** gộp một nhóm segment
**When** tính cờ kết đoạn
**Then** cờ theo **câu cuối** của nhóm

**Given** tách một segment thành nhiều mảnh
**When** tính cờ kết đoạn
**Then** cờ theo **mảnh cuối**, mọi mảnh trước nhận cờ **tắt**

**Given** `⌘M` và `⌘/`
**When** gọi
**Then** là command đã đăng ký, **không phải hệ quả phụ của việc gõ**

---

### Story 2.9: Gộp ngầm khi gõ đè lên ranh giới

**Covers:** FR78

As a người dịch,
I want viết lại hai câu Trung thành một câu Việt bằng cách gõ tự do,
So that tôi không phải dừng lại ra lệnh cho công cụ giữa dòng suy nghĩ.

**Acceptance Criteria:**

**Given** con trỏ ở đúng vị trí ranh giới giữa hai segment
**When** người dùng gõ đè lên ranh giới đó
**Then** hệ thống thực hiện gộp

**Given** gộp ngầm xảy ra
**When** thực hiện
**Then** đúng ngữ nghĩa của gộp tường minh — hai câu cũ về hưu và vẫn tra lại được lịch sử, câu mới chưa xác nhận với lịch sử rỗng

**Given** gộp ngầm xảy ra
**When** thực hiện
**Then** một dòng báo hiện ở lề nêu **hệ quả**, ví dụ *"Đã gộp hai câu. Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được."*

**Given** gộp ngầm vừa xảy ra
**When** người dùng bấm `⌘Z`
**Then** hoàn tác được

**Given** người dùng gõ đè lên ranh giới
**When** xảy ra
**Then** hệ thống **không chặn và không hỏi lại**

---

### Story 2.10: Điều hướng segment

**Covers:** FR25

As a người dịch,
I want nhảy tới câu chưa dịch tiếp theo bằng một phím,
So that tôi không phải cuộn tìm bằng mắt trong một Chương dài.

**Acceptance Criteria:**

**Given** con trỏ ở một segment
**When** gọi lệnh **segment kế tiếp**
**Then** focus chuyển sang segment ngay sau nó

**Given** con trỏ ở một segment
**When** gọi lệnh **segment trước đó**
**Then** focus chuyển sang segment ngay trước nó

**Given** một Chương có segment đã dịch xen kẽ segment chưa dịch
**When** gọi lệnh **segment chưa dịch kế tiếp**
**Then** focus nhảy tới segment chưa dịch gần nhất phía sau, bỏ qua các segment đã dịch

**Given** không còn segment chưa dịch nào phía sau
**When** gọi lệnh segment chưa dịch kế tiếp
**Then** báo rõ điều đó thay vì im lặng không làm gì

**Given** con trỏ ở segment đầu hoặc cuối Chương
**When** gọi lệnh vượt biên
**Then** hành vi ở biên rõ ràng và không sập

**Given** focus chuyển segment
**When** xảy ra
**Then** vạch lề của segment nhận focus chuyển `primary`
**And** vùng nhìn cuộn tới nó **tức thì**, không có hiệu ứng cuộn

**Given** ba lệnh này
**When** gọi
**Then** đều là command đăng ký, gán phím được

---

### Story 2.11: Chuyển Chương trong Workspace

**Covers:** FR26

As a người dịch,
I want sang Chương kế tiếp mà không phải quay về Library,
So that mạch làm việc của tôi không bị cắt.

**Acceptance Criteria:**

**Given** một Chương đang mở trong Workspace
**When** gọi lệnh **Chương sau**
**Then** Chương kế tiếp trong cùng Tác phẩm mở ra

**Given** người dùng gọi lệnh **Chương trước**
**When** xảy ra
**Then** Chương liền trước mở ra

**Given** chuyển Chương
**When** xảy ra
**Then** văn bản đang gõ ở Chương cũ được **flush trước khi chuyển**

**Given** Chương đầu tiên hoặc Chương cuối cùng của Tác phẩm
**When** gọi lệnh vượt biên
**Then** báo rõ đã ở biên, không sập và **không quay vòng im lặng**

**Given** một Chương được mở lại về sau
**When** mở
**Then** khôi phục đúng segment và vị trí cuộn lần trước

**Given** hai lệnh này
**When** gọi
**Then** là command đăng ký, gán phím được

---

### Story 2.12: Sync Scrolling

**Covers:** FR20

As a người dịch,
I want nguyên văn và bản dịch cuộn cùng nhau,
So that mắt tôi không phải tự tìm lại chỗ mỗi lần nhìn sang panel bên.

**Acceptance Criteria:**

**Given** Sync Scrolling đang bật
**When** người dùng cuộn Panel Source
**Then** Panel Editor và Panel AI Translation cuộn theo tới vị trí tương ứng

**Given** Sync Scrolling đang bật
**When** người dùng cuộn Panel Editor
**Then** hai panel còn lại cuộn theo

**Given** Sync Scrolling
**When** hiển thị
**Then** có **công tắc bật/tắt rõ ràng**, không phải một tuỳ chọn ẩn trong Cài đặt

**Given** Sync Scrolling đang tắt
**When** cuộn một panel
**Then** các panel khác không đổi vị trí

**Given** trạng thái công tắc
**When** đóng và mở lại ứng dụng
**Then** giữ nguyên

**Given** Panel AI Translation chưa có nội dung ở epic này
**When** Sync Scrolling chạy
**Then** không sập
**And** panel đó tham gia đồng bộ ngay khi có nội dung ở Epic 4, không cần sửa lại cơ chế

**Given** công tắc
**When** gọi
**Then** là command đăng ký, gán phím được

---

## Epic 3: Glossary — chốt thuật ngữ một lần, dùng mãi

Người dịch bôi đen một cụm từ ở bất kỳ panel nào và thêm vào Glossary mà **không rời màn hình đang làm việc**; thuật ngữ đã chốt hiện ngay dấu trực quan trong Panel Source. Sau một lần nhập lớn, hàng trăm ứng viên do máy quét ra hiện thành bảng chờ xếp theo tần suất, **duyệt bằng một phím mỗi mục, không phải gõ chữ nào** — ứng viên tiếng Trung còn kèm sẵn bản dịch âm Hán Việt, chạy hoàn toàn ngoại tuyến. Glossary xuất/nhập round-trip qua CSV/TSV để chia sẻ trong cộng đồng.

### Story 3.1: Mô hình Glossary hai tầng và vòng đời ba trạng thái

**Covers:** FR46, FR47

As a người dịch,
I want một thuật ngữ tôi chốt riêng cho một Tác phẩm đè lên cách dịch chung của mình,
So that tên nhân vật của bộ truyện này không lẫn sang bộ khác.

**Acceptance Criteria:**

**Given** một mục Glossary
**When** tạo
**Then** mang thuật ngữ nguồn, bản dịch, ghi chú, phân loại *(tên người / địa danh / thuật ngữ chuyên ngành / khác)*, ngày thêm, và **xuất xứ** *(nhập thủ công / đề xuất khi nhập tài liệu / thu hoạch từ bản review)*

**Given** một thuật ngữ tồn tại ở cả tầng Global và tầng Tác phẩm
**When** phân giải
**Then** **tầng Tác phẩm thắng**
**And** phân giải đi qua `ScopeResolver`, không qua một truy vấn riêng

**Given** vòng đời một mục
**When** mô hình hoá
**Then** ba trạng thái **một chiều**: ứng viên → chờ chốt bản dịch → đã chốt

**Given** trường bản dịch
**When** mục chưa chốt
**Then** **nullable**

**Given** `glossary/`
**When** phơi bề mặt ra cho module khác
**Then** có **đúng một** truy vấn trả về mục **đủ điều kiện chèn**
**And** điều kiện chèn nằm trong `glossary/` — nơi sở hữu dữ liệu — chứ không ở chỗ gọi

**Given** một mục ở trạng thái **chờ chốt**
**When** truy vấn mục đủ điều kiện chèn
**Then** **không** trả về nó

**Given** mục Glossary tầng Tác phẩm
**When** lưu
**Then** nằm trong `project.db` của Tác phẩm đó
**And** mục tầng Global nằm trong `global.db`

---

### Story 3.2: Bảng chờ ứng viên tách hẳn khỏi Glossary

**Covers:** FR55

As a người dịch,
I want chắc chắn không đề xuất nào của máy lọt vào Glossary sau lưng tôi,
So that công cụ không bao giờ tự quyết cách dịch thay tôi.

**Acceptance Criteria:**

**Given** ứng viên do máy sinh ra
**When** ghi
**Then** ghi vào một **bảng chờ riêng**
**And** **không** phải một cột trạng thái trên bảng Glossary

**Given** toàn bộ mã nguồn
**When** rà
**Then** **không có đường ghi nào** từ cơ chế đề xuất tự động vào bảng Glossary

**Given** một ứng viên
**When** chuyển sang Glossary
**Then** chỉ qua một thao tác duyệt **tường minh** của người dùng

**Given** một mục vừa được duyệt vào Glossary
**When** kiểm
**Then** mang trường xuất xứ ghi rõ nó đến từ cơ chế nào

**Given** một ứng viên bị bỏ
**When** xảy ra
**Then** nó rời bảng chờ
**And** không quay lại ở lần quét sau trong cùng Tác phẩm

**Given** bảng chờ này
**When** cơ chế thu hoạch từ bản review được thêm ở Epic 8
**Then** nó ghi vào **chính bảng này**, không cần bảng thứ hai

---

### Story 3.3: Thêm nhanh thuật ngữ từ bất kỳ panel nào

**Covers:** FR48

As a người dịch,
I want chốt một thuật ngữ ngay khi gặp nó mà không rời câu đang dịch,
So that tôi không bỏ qua chỉ vì ngại mở một màn hình khác.

**Acceptance Criteria:**

**Given** người dùng bôi đen một cụm từ ở Panel Source, Panel Lookup, Panel AI Translation hoặc Panel Editor
**When** gọi lệnh thêm thuật ngữ
**Then** hộp thêm nhanh mở ra với cụm đó điền sẵn

**Given** hộp thêm nhanh
**When** mở
**Then** người dùng chọn tầng Global hay Tác phẩm, và chọn phân loại

**Given** người dùng xác nhận
**When** thêm
**Then** mục vào Glossary ở trạng thái **đã chốt** với xuất xứ *nhập thủ công*

**Given** thao tác thêm hoàn tất
**When** xảy ra
**Then** màn hình đang làm việc không đổi
**And** con trỏ quay về đúng chỗ cũ

**Given** cụm từ đã có trong Glossary
**When** gọi lệnh thêm
**Then** hộp mở ở chế độ **sửa mục sẵn có**, không tạo mục trùng

**Given** toàn bộ thao tác thêm nhanh
**When** thực hiện
**Then** làm được hoàn toàn bằng bàn phím

---

### Story 3.4: Khớp thuật ngữ theo ngôn ngữ và đánh dấu trong Panel Source

**Covers:** FR50, FR51

As a người dịch,
I want thấy ngay câu đang dịch chứa thuật ngữ nào đã chốt,
So that tôi không dịch lệch khỏi chính quyết định của mình.

**Acceptance Criteria:**

**Given** một câu ở Panel Source chứa thuật ngữ có trong Glossary
**When** hiển thị
**Then** thuật ngữ đó được đánh dấu bằng màu `primary`

**Given** văn bản tiếng Trung
**When** khớp thuật ngữ
**Then** dùng khớp chính xác

**Given** văn bản tiếng Anh
**When** khớp thuật ngữ
**Then** khớp mờ ở cấp hình thái từ qua stemming

**Given** cơ chế khớp
**When** cài đặt
**Then** dùng **đúng component `Matcher` dùng chung** của Epic 1, không cài lại

**Given** một mục ở trạng thái **chờ chốt bản dịch**
**When** xuất hiện trong câu
**Then** cũng được đánh dấu
**And** **phân biệt được** với mục đã chốt

**Given** cả hai tầng Glossary
**When** khớp
**Then** áp cả hai, tầng Tác phẩm thắng khi trùng

**Given** người dùng rê chuột hoặc đưa tiêu điểm tới một thuật ngữ đã đánh dấu
**When** xảy ra
**Then** thấy bản dịch đã chốt của nó

---

### Story 3.5: Quét ứng viên khi nhập tài liệu

**Covers:** FR52

As a người dịch,
I want công cụ tự tìm ra những cái tên lặp đi lặp lại trong một bộ truyện,
So that tôi không phải đọc hết 2000 chương mới biết mình cần chốt gì.

**Acceptance Criteria:**

**Given** một Chương hoặc một loạt Chương vừa nhập
**When** quét ứng viên
**Then** tìm các chuỗi lặp lại **từ 5 lần trở lên** *và* **không có trong từ điển nhúng**

**Given** ngưỡng 5 lần
**When** người dùng muốn đổi
**Then** **cấu hình lại được**

**Given** văn bản tiếng Trung
**When** quét
**Then** đối chiếu danh sách họ phổ biến để đoán tên người

**Given** văn bản tiếng Anh
**When** quét
**Then** bắt các cụm viết hoa **không đứng đầu câu**

**Given** kết quả quét
**When** ghi
**Then** vào **bảng chờ**, không vào Glossary

**Given** mỗi ứng viên
**When** ghi
**Then** kèm **số lần xuất hiện** và ít nhất một **ví dụ ngữ cảnh**

**Given** quét chạy trên một lần nhập lớn
**When** thực hiện
**Then** chạy nền và không chặn thao tác của người dùng

**Given** một chuỗi đã có trong Glossary hoặc đã từng bị bỏ
**When** quét
**Then** không đưa vào bảng chờ lần nữa

---

### Story 3.6: Trạng thái chờ chốt và dải mọc chốt lần đầu gặp

**Covers:** FR114

As a người dịch,
I want quyết định cách dịch một thuật ngữ khi tôi đang nhìn đúng câu chứa nó,
So that quyết định đó có ngữ cảnh thay vì là một dòng trong danh sách trần.

**Acceptance Criteria:**

**Given** một ứng viên không có bản dịch đề xuất
**When** người dùng nhận
**Then** mục vào Glossary với trường bản dịch ở trạng thái **chờ chốt**

**Given** người dùng gặp thuật ngữ đó **lần đầu tiên** trong Workspace
**When** xảy ra
**Then** một **dải mọc** dưới câu đang sửa hỏi bản dịch

**Given** dải mọc
**When** hiển thị
**Then** **đẩy văn bản xuống** chứ không phủ lên
**And** thu lại ngay khi xong

**Given** người dùng trả lời
**When** xảy ra
**Then** mục khoá thành **đã chốt**
**And** hệ thống **không hỏi lại** thuật ngữ đó trong cả Tác phẩm

**Given** một câu kích hoạt đồng thời chốt Glossary, phát hiện Proofreader và gợi ý TM
**When** hiển thị
**Then** **chỉ một dải mọc tại một thời điểm**
**And** chốt Glossary thắng — vì thuật ngữ chưa chốt không tham gia ép AI, để treo là để một lỗ hổng chạy tiếp qua mọi câu sau

**Given** dải trên vừa xử lý xong
**When** xảy ra
**Then** dải kế tiếp mọc **ngay tại chỗ vừa thu**, vị trí không nhảy

**Given** một mục ở trạng thái chờ chốt
**When** truy vấn mục đủ điều kiện chèn vào prompt
**Then** **không** trả về nó

**Given** toàn bộ thao tác trên dải
**When** thực hiện
**Then** làm được bằng bàn phím

---

### Story 3.7: Đề xuất bản dịch bằng âm Hán Việt

**Covers:** FR113

As a người dịch truyện Trung,
I want công cụ tự điền sẵn *"Bắc Lương"* khi nó thấy `北涼`,
So that tôi chỉ việc gật đầu thay vì gõ lại hàng trăm cái tên.

**Acceptance Criteria:**

**Given** một ứng viên tiếng Trung trong bảng chờ
**When** sinh đề xuất
**Then** đề xuất bản dịch là **âm Hán Việt** của chuỗi đó

**Given** âm Hán Việt
**When** đọc
**Then** đọc **qua cổng `DictionarySource`**
**And** không có cài đặt thứ hai nào của dữ liệu Hán Việt bên trong `glossary/`

**Given** đồ thị phụ thuộc module
**When** kiểm
**Then** có cạnh `glossary/ → dict/`
**And** không tạo chu trình

**Given** ngắt kết nối mạng
**When** sinh đề xuất
**Then** vẫn chạy đầy đủ

**Given** người dùng nhận một ứng viên có đề xuất
**When** xảy ra
**Then** mục vào Glossary ở trạng thái **đã chốt** với cả thuật ngữ lẫn bản dịch

**Given** một mục đã vào Glossary từ đề xuất
**When** người dùng muốn sửa
**Then** sửa được như mọi mục khác

**Given** một ứng viên tiếng Anh, hoặc một chuỗi tiếng Trung không tra được âm Hán Việt
**When** sinh đề xuất
**Then** không đề xuất gì
**And** mục sẽ đi theo đường chờ chốt của Story 3.6

---

### Story 3.8: Duyệt hàng loạt một phím

**Covers:** FR53

As a người dịch,
I want duyệt 340 ứng viên bằng một phím mỗi mục,
So that mười phút là xong thay vì cả buổi ngồi gõ.

**Acceptance Criteria:**

**Given** bảng chờ có nhiều ứng viên
**When** mở
**Then** hiện danh sách xếp theo **tần suất giảm dần**

**Given** mỗi dòng ứng viên
**When** hiển thị
**Then** có số lần xuất hiện, ít nhất một ví dụ ngữ cảnh, và **bản dịch đề xuất khi có**

**Given** một ứng viên đang chọn
**When** người dùng bấm phím nhận
**Then** mục vào Glossary **cùng bản dịch đề xuất nếu có**
**And** con trỏ tự chuyển sang ứng viên kế tiếp

**Given** một ứng viên đang chọn
**When** người dùng bấm phím bỏ
**Then** ứng viên rời bảng chờ và con trỏ chuyển tiếp

**Given** toàn bộ luồng duyệt
**When** thực hiện
**Then** **không phải gõ chữ nào**

**Given** phân loại của một ứng viên
**When** người dùng muốn đổi
**Then** đổi bằng **phím số**

**Given** người dùng đóng bảng chờ giữa chừng
**When** mở lại
**Then** quay đúng vị trí đang duyệt

**Given** hàng đã duyệt và hàng đã bỏ
**When** hiển thị
**Then** lùi ra sau bằng cách **đổi màu chữ** sang `on-surface-variant` cộng dấu `✓` / `✕`
**And** **không dùng `opacity` để làm mờ chữ**

---

### Story 3.9: Quản lý Glossary

**Covers:** FR49

As a người dịch,
I want rà lại và dọn Glossary của mình,
So that một quyết định sai từ nửa năm trước không kéo dài mãi.

**Acceptance Criteria:**

**Given** màn hình quản lý Glossary
**When** mở
**Then** hiện mục của **cả hai tầng**, phân biệt được tầng nào

**Given** người dùng tìm kiếm
**When** gõ
**Then** lọc theo thuật ngữ nguồn **và** theo bản dịch

**Given** danh sách
**When** lọc
**Then** lọc được theo phân loại, theo **xuất xứ**, và theo trạng thái chốt

**Given** một mục
**When** người dùng sửa
**Then** thay đổi lưu ngay và có hiệu lực ở lần khớp kế tiếp

**Given** một mục
**When** người dùng xoá
**Then** mục biến khỏi Glossary
**And** đánh dấu của nó trong Panel Source cũng mất theo

**Given** một mục ở tầng Tác phẩm
**When** người dùng muốn đẩy lên tầng Global
**Then** làm được bằng một thao tác

**Given** mọi thao tác trên màn hình này
**When** thực hiện
**Then** làm được bằng bàn phím

---

### Story 3.10: Xuất và nhập Glossary qua CSV/TSV

**Covers:** FR49

As a người dịch,
I want gửi bộ thuật ngữ của mình cho một người dịch khác bằng một file,
So that cộng đồng chia sẻ được mà không cần server hay tài khoản.

**Acceptance Criteria:**

**Given** một Glossary ở tầng bất kỳ
**When** người dùng xuất
**Then** sinh ra file CSV hoặc TSV chứa đầy đủ các trường của mỗi mục

**Given** một file vừa xuất
**When** nhập lại vào một Glossary rỗng
**Then** **round-trip đầy đủ**, không mất trường nào

**Given** một file nhập chứa thuật ngữ đã có trong Glossary
**When** nhập
**Then** xung đột hiện ra cho người dùng quyết
**And** **không im lặng ghi đè**

**Given** người dùng chọn tầng đích khi nhập
**When** thực hiện
**Then** mục vào đúng tầng đó

**Given** một file nhập sai định dạng hoặc thiếu cột
**When** nhập
**Then** báo lỗi nêu rõ dòng nào và thiếu gì
**And** **không ghi một phần**

**Given** mục nhập vào từ file
**When** ghi
**Then** mang xuất xứ phân biệt được với mục người dùng tự nhập tay

**Given** một mục trong file nhập **không có bản dịch**
**When** ghi
**Then** vào trạng thái **chờ chốt**, không vào trạng thái đã chốt với bản dịch rỗng

---

## Epic 4: AI mở & Smart RAG Injector

Người dịch cấu hình **một API key của chính mình** hoặc trỏ tới Ollama/LM Studio qua **cùng một đường cấu hình**, rồi gọi AI dịch từng segment hoặc theo lô — kết quả chảy dần theo dòng, huỷ được giữa chừng, kèm số token và ước tính chi phí. Trước mỗi lần gọi, hệ thống tự chèn các thuật ngữ Glossary **đã chốt** xuất hiện trong câu; người dùng **mở xem được prompt cuối cùng đã gửi đi**. Kết quả nằm ở panel AI Translation và **không bao giờ tự chảy vào Editor**. Gỡ sạch cấu hình AI thì Epic 1, 2, 3 vẫn chạy đầy đủ — **cưỡng chế bằng test tự động**, không bằng kỷ luật.

### Story 4.1: Module `ai/` cô lập và test cưỡng chế ranh giới

**Covers:** FR77

As a người dịch không dùng AI,
I want gỡ sạch cấu hình AI mà mọi thứ khác vẫn chạy đầy đủ,
So that công cụ này là của tôi chứ không phải một vỏ bọc quanh một dịch vụ đám mây.

**Acceptance Criteria:**

**Given** module `core/ai/`
**When** dựng
**Then** không module nào khác trong `core/` import nó

**Given** chiều phụ thuộc
**When** kiểm
**Then** có **test tự động thất bại** nếu một module ngoài `ai/` import `ai/`
**And** hoặc `ai/` là crate riêng để trình biên dịch cưỡng chế

**Given** chiều ngược lại
**When** kiểm
**Then** `ai/` được phép đọc `glossary/`, `tm/`, `segment/` — đây là chiều hợp lệ

**Given** ứng dụng chạy mà chưa cấu hình nhà cung cấp AI nào
**When** dùng
**Then** Library, Workspace, tra cứu, Glossary và **toàn bộ năng lực ngoài C6/C7** chạy đầy đủ

**Given** không có API key nào trong keychain
**When** khởi động
**Then** ứng dụng không báo lỗi và không chặn thao tác nào

**Given** Panel AI Translation khi chưa cấu hình
**When** hiển thị
**Then** **mời cấu hình** và nói rõ mọi thứ khác vẫn chạy đầy đủ
**And** đây **không phải trạng thái lỗi**

**Given** bộ test của Epic 1, 2 và 3
**When** chạy trong một môi trường **không có cấu hình AI**
**Then** toàn bộ vẫn xanh

---

### Story 4.2: Cấu hình nhà cung cấp AI

**Covers:** FR65, FR66, FR68

As a người dịch,
I want dùng API key của chính mình hoặc một mô hình chạy trên máy tôi qua cùng một chỗ cấu hình,
So that tôi không bị khoá vào một nhà cung cấp nào.

**Acceptance Criteria:**

**Given** màn hình cấu hình AI
**When** mở
**Then** nhập được endpoint, tên mô hình và tham số sinh

**Given** một nhà cung cấp cloud dùng BYOK
**When** cấu hình
**Then** dùng **cùng một biểu mẫu** với local LLM

**Given** Ollama hoặc LM Studio chạy trên máy
**When** cấu hình
**Then** kết nối qua endpoint tương thích OpenAI
**And** không cần một đường tích hợp riêng

**Given** cấu hình AI
**When** lưu
**Then** ở tầng Global

**Given** một Tác phẩm cần cấu hình khác
**When** người dùng đặt
**Then** **ghi đè được** theo Tác phẩm đó
**And** phân giải đi qua `ScopeResolver` với ngữ nghĩa ghi đè

**Given** một endpoint sai hoặc không phản hồi
**When** người dùng thử kết nối
**Then** báo rõ nguyên nhân
**And** không lưu một cấu hình hỏng

---

### Story 4.3: API key trong keychain

**Covers:** FR67 · NFR11

As a người dịch,
I want khoá API của mình không bao giờ nằm trong một file nào trên máy,
So that một lần chia sẻ thư mục dự án không làm lộ nó.

**Acceptance Criteria:**

**Given** một API key
**When** lưu
**Then** vào keychain / credential manager của hệ điều hành, qua crate `keyring` **gọi trực tiếp trong Rust**

**Given** cây phụ thuộc
**When** kiểm
**Then** **không có `tauri-plugin-keyring`** — plugin đó tồn tại để phơi API ra JavaScript, đúng thứ NFR11 cấm

**Given** ranh giới IPC
**When** quan sát
**Then** khoá **không bao giờ** đi qua

**Given** frontend
**When** truy vấn trạng thái khoá
**Then** chỉ nhận được *"đã cấu hình"* hoặc *"chưa cấu hình"*

**Given** file cấu hình, file dự án và log
**When** rà
**Then** không chứa khoá dưới bất kỳ dạng nào

**Given** một lỗi phát sinh trong lúc gọi AI
**When** ghi log
**Then** khoá không xuất hiện trong thông báo lỗi

**Given** người dùng xoá khoá
**When** xảy ra
**Then** nó biến khỏi keychain và trạng thái về *chưa cấu hình*

---

### Story 4.4: Bộ prompt theo thể loại

**Covers:** FR69

As a người dịch làm nhiều lĩnh vực,
I want một bộ prompt riêng cho tiên hiệp và một bộ khác cho báo chí,
So that một công cụ phục vụ được mọi lĩnh vực mà không cần nhiều chế độ riêng.

**Acceptance Criteria:**

**Given** màn hình bộ prompt
**When** mở
**Then** soạn, sửa, xoá và đặt tên được từng bộ

**Given** bộ prompt
**When** lưu
**Then** tồn tại ở **cả hai tầng** Global và Tác phẩm

**Given** một bộ prompt cùng tên ở hai tầng
**When** phân giải
**Then** **tầng Tác phẩm thắng**
**And** phân giải đi qua `ScopeResolver`

**Given** một bộ prompt đang soạn
**When** hiển thị
**Then** thấy được các biến sẽ được chèn động ở bước sau

**Given** một Tác phẩm đang mở
**When** hiển thị
**Then** bộ prompt đang có hiệu lực hiện rõ
**And** đổi được ngay, không phải vào Cài đặt

**Given** mọi thao tác trên màn hình này
**When** thực hiện
**Then** làm được bằng bàn phím

---

### Story 4.5: Xuất và nhập bộ prompt

**Covers:** FR79

As a người dịch,
I want gửi bộ prompt tiên hiệp của mình cho người khác bằng một file,
So that cộng đồng chia sẻ quy chuẩn dịch mà không cần hạ tầng nào.

**Acceptance Criteria:**

**Given** một bộ prompt
**When** xuất
**Then** sinh ra file văn bản mở

**Given** một file vừa xuất
**When** nhập lại
**Then** **round-trip đầy đủ**

**Given** người dùng chọn tầng đích khi nhập
**When** thực hiện
**Then** bộ prompt vào đúng tầng đó

**Given** một bộ prompt trùng tên
**When** nhập
**Then** xung đột hiện ra cho người dùng quyết
**And** không im lặng ghi đè

**Given** một file sai định dạng
**When** nhập
**Then** báo lỗi nêu rõ vấn đề
**And** không ghi một phần

**Given** file xuất ra
**When** mở bằng một trình soạn thảo văn bản thường
**Then** đọc và sửa được bằng tay

---

### Story 4.6: Smart RAG Injector là một hàm thuần

**Covers:** FR70

As a người dịch,
I want AI luôn nhận đúng những thuật ngữ tôi đã chốt cho câu đang dịch,
So that nó không gọi nhân vật của tôi bằng một cái tên khác ở mỗi chương.

**Acceptance Criteria:**

**Given** `RagInjector`
**When** cài đặt
**Then** là một **hàm thuần** nhận `(câu nguồn, scope, Glossary, TM)` và trả về **prompt đã lắp hoàn chỉnh**

**Given** một lời gọi AI
**When** thực hiện
**Then** nhận prompt đã lắp làm đầu vào
**And** **không có chỗ nào nối chuỗi prompt rải rác tại chỗ gọi**

**Given** một câu nguồn chứa thuật ngữ có trong Glossary
**When** lắp prompt
**Then** chèn thuật ngữ đó kèm **bản dịch đã chốt**

**Given** `ai/` cần dữ liệu Glossary
**When** truy vấn
**Then** dùng **đúng một truy vấn "mục đủ điều kiện chèn"** của `glossary/`
**And** không có đường nào khác chạm dữ liệu Glossary

**Given** một mục ở trạng thái **chờ chốt bản dịch**
**When** lắp prompt
**Then** **không được chèn**

**Given** cả hai tầng Glossary
**When** lắp prompt
**Then** áp cả hai, tầng Tác phẩm thắng khi trùng

**Given** cùng một đầu vào
**When** gọi hàm nhiều lần
**Then** luôn ra **cùng một prompt**

**Given** phần chèn Translation Memory chưa tồn tại ở epic này
**When** lắp prompt
**Then** hàm nhận tham số TM rỗng và vẫn đúng
**And** Epic 7 điền nốt tham số đó **mà không đổi chữ ký hàm**

---

### Story 4.7: Xem prompt cuối cùng đã gửi

**Covers:** FR71

As a người dịch,
I want nhìn được vào hộp đen,
So that khi AI không tuân thủ Glossary tôi biết vì sao chứ không phải đoán.

**Acceptance Criteria:**

**Given** một lời gọi AI vừa thực hiện
**When** người dùng mở **Xem prompt**
**Then** thấy prompt cuối cùng đã gửi đi, nguyên vẹn

**Given** prompt hiển thị
**When** xem
**Then** bao gồm **toàn bộ phần chèn động**, phân biệt được với phần prompt do người dùng soạn

**Given** các thuật ngữ Glossary đã được chèn
**When** hiển thị
**Then** thấy rõ thuật ngữ nào được chèn và bản dịch nào đi kèm

**Given** prompt hiển thị
**When** đối chiếu với thứ thật sự gửi đi
**Then** khớp **100%**, không phải một bản dựng lại xấp xỉ

**Given** một lời gọi AI hoàn tất
**When** hiển thị
**Then** một dòng tóm tắt cho biết đã chèn bao nhiêu thuật ngữ Glossary

**Given** thao tác mở Xem prompt
**When** gọi
**Then** là command đăng ký, gán phím được

---

### Story 4.8: Dịch một segment với kết quả chảy dần

**Covers:** FR72, FR74

As a người dịch,
I want thấy bản dịch AI hiện dần ngay khi mô hình đang sinh, và nó nằm yên ở panel riêng,
So that tôi đọc sớm được mà vẫn là người quyết định đưa gì vào bản dịch.

**Acceptance Criteria:**

**Given** một segment đang chọn
**When** người dùng gọi lệnh dịch
**Then** lời gọi AI thực hiện với prompt do `RagInjector` lắp

**Given** mô hình đang sinh
**When** token về
**Then** hiển thị dần ở Panel AI Translation

**Given** luồng token
**When** truyền
**Then** qua **Tauri Channel API**
**And** không dùng event rời

**Given** client HTTP
**When** cài đặt
**Then** **không dùng client SSE tự kết nối lại**

**Given** kết quả AI hoàn tất
**When** xảy ra
**Then** nằm ở Panel AI Translation
**And** **không tự động ghi vào Editor dưới bất kỳ điều kiện nào**

**Given** người dùng muốn đưa kết quả sang Editor
**When** bấm `⌘⇧↵`
**Then** văn bản chuyển sang Editor tại segment đang chọn

**Given** trạng thái AI
**When** hiển thị
**Then** một trong năm giá trị: chưa cấu hình · đang sinh · xong · lỗi · đã huỷ

---

### Story 4.9: Dịch theo lô và huỷ giữa chừng

**Covers:** FR73

As a người dịch,
I want cho AI dịch trước một loạt câu rồi biên tập lại, và dừng nó bất cứ lúc nào,
So that tôi không bị khoá vào một lệnh đã lỡ bấm.

**Acceptance Criteria:**

**Given** nhiều segment liên tiếp đang chọn
**When** người dùng gọi lệnh dịch lô
**Then** hệ thống dịch lần lượt từng segment

**Given** một lô đang chạy
**When** hiển thị
**Then** thấy segment nào đã xong, segment nào đang chạy, còn bao nhiêu

**Given** một lô đang chạy
**When** người dùng huỷ
**Then** dừng ngay sau segment đang chạy hiện tại

**Given** một lô bị huỷ
**When** xảy ra
**Then** các segment đã dịch xong giữ nguyên kết quả
**And** các segment chưa chạy không bị đánh dấu gì

**Given** một segment trong lô gặp lỗi
**When** xảy ra
**Then** lô dừng và báo rõ segment nào
**And** không âm thầm bỏ qua rồi chạy tiếp

**Given** mọi lời gọi AI dù đơn lẻ hay theo lô
**When** đang chạy
**Then** **huỷ được giữa chừng**

**Given** lệnh dịch lô và lệnh huỷ
**When** gọi
**Then** là command đăng ký, gán phím được

---

### Story 4.10: Lỗi mạng và lỗi API

**Covers:** FR75

As a người dịch dùng API key của chính mình,
I want một lần lỗi không tốn thêm tiền của tôi và không làm mất câu tôi đang gõ,
So that tôi tin được vào công cụ khi mạng chập chờn.

**Acceptance Criteria:**

**Given** một lỗi mạng hoặc lỗi API
**When** xảy ra
**Then** thông báo nêu rõ nguyên nhân

**Given** thông báo lỗi
**When** soạn
**Then** **không đổ lỗi người dùng** — *"Nhà cung cấp không phản hồi"* chứ không *"Bạn đã nhập sai khoá"*

**Given** một lỗi xảy ra
**When** sau đó
**Then** công việc đang làm ở Editor **không mất**

**Given** một lỗi xảy ra
**When** hệ thống xử lý
**Then** **không tự động thử lại lần nào** — với BYOK, mỗi lần gọi là tiền của người dùng

**Given** người dùng muốn thử lại
**When** bấm nút thử lại
**Then** một lời gọi mới được thực hiện

**Given** luồng streaming bị đứt giữa chừng
**When** xảy ra
**Then** coi là **lỗi tường minh**, không tự kết nối lại
**And** phần token đã nhận vẫn hiển thị

**Given** một lỗi đi qua IPC
**When** truyền
**Then** mang hình dạng `{ code, message_key, params, retryable }`
**And** frontend phân giải chuỗi từ `vi.json`

---

### Story 4.11: Số token và ước tính chi phí

**Covers:** FR76

As a người dịch trả tiền cho từng lời gọi,
I want thấy mỗi lần bấm tốn bao nhiêu,
So that tôi biết mình đang tiêu gì.

**Acceptance Criteria:**

**Given** một lời gọi AI hoàn tất
**When** hiển thị
**Then** thấy số token đã dùng

**Given** số token và mô hình đang dùng
**When** tính
**Then** hiển thị **ước tính chi phí**

**Given** số liệu
**When** hiển thị
**Then** ghi **đúng số**, không làm tròn thành chữ — *"412 token · ước tính ~0,004 USD"*, không phải *"một chút chi phí"*

**Given** một lô nhiều segment
**When** chạy xong
**Then** hiển thị tổng token và tổng ước tính của cả lô

**Given** một mô hình local LLM
**When** hiển thị
**Then** vẫn hiện số token
**And** không hiện ước tính chi phí bằng tiền

**Given** nhà cung cấp không trả về số token
**When** xảy ra
**Then** nói rõ **không có số liệu**, thay vì hiện số 0

---

### Story 4.12: Bố cục màn hình hẹp và hiệu chỉnh ngưỡng

**Covers:** UX-DR15 · A11 · Q9 — đóng ngưỡng, thứ tự hy sinh panel đã chốt ở Story 1.14

As a người dịch làm trên một laptop nhỏ,
I want cặp Nguyên văn và Bản dịch không bao giờ bị nhường chỗ cho panel khác,
So that cửa sổ hẹp làm tôi khó chịu chứ không làm tôi không dịch được.

**Acceptance Criteria:**

**Given** ngưỡng bố cục
**When** đo
**Then** đo theo **vùng làm việc** — chiều cao cửa sổ trừ thanh tiêu đề 38px và thanh trạng thái 32px
**And** **không** đo theo kích thước màn hình

**Given** vùng làm việc **≥ 1100×820**
**When** hiển thị
**Then** giữ lưới 2×2

**Given** vùng làm việc **cao < 820**
**When** hiển thị
**Then** gộp hàng dưới thành một panel có tab

**Given** vùng làm việc **rộng < 1100 hoặc cao < 700**
**When** hiển thị
**Then** chỉ còn `Nguyên văn | Bản dịch`
**And** Tra cứu rút về ngăn kéo

**Given** vùng làm việc **rộng < 860**
**When** hiển thị
**Then** báo không hỗ trợ

**Given** cửa sổ thu hẹp dần
**When** phải hy sinh panel
**Then** **Đề xuất AI nhường trước**

**Given** cửa sổ thu hẹp tiếp
**When** phải hy sinh thêm
**Then** **Tra cứu nhường sau**, nhưng **rút về thanh trạng thái — không bao giờ mất hẳn**

**Given** cửa sổ hẹp tới mức nào
**When** hiển thị
**Then** cặp **Nguyên văn | Bản dịch không bao giờ nhường**

**Given** bốn ngưỡng trên
**When** hiệu chỉnh trên máy thật
**Then** **chỉ các con số** được đổi
**And** **thứ tự hy sinh panel là quyết định, không hiệu chỉnh theo**

**Given** phép đo trên máy thật với đủ bốn panel có nội dung
**When** hoàn tất
**Then** giá trị hiệu chỉnh ghi vào `[A11]` của `SPEC.md`
**And** **Q9 đóng**

---

## Epic 5: Library — kho tác phẩm, tìm kiếm, và đọc lại thành quả

Mở ứng dụng là vào Library, không phải vào màn hình dịch. Người dịch **nắm được mình đang có những gì**: lưới Tác phẩm với bìa, tiến độ và bốn trạng thái vòng đời; lọc theo trạng thái, lĩnh vực, ngôn ngữ nguồn, ngày sửa; **tìm full-text xuyên toàn thư viện phân biệt dấu**. Mở một Chương đưa thẳng vào Workspace **đúng câu đang dở lần trước**. Và Chế độ đọc để người dịch quay lại **thưởng thức thành quả**: đọc liên tục qua các Chương đã xong, dừng ở biên tường minh, đánh dấu chỗ cần sửa bằng một phím rồi **đọc tiếp ngay**. Xoá chỉ mục Library rồi quét lại phục hồi đầy đủ, không mất một byte dữ liệu nào.

### Story 5.1: Mô hình Library hai tầng

**Covers:** FR1, FR2, FR3, FR4

As a người dịch,
I want mọi thứ tôi dịch nằm trong một cấu trúc hai tầng đơn giản,
So that một bài báo và một bộ 2000 chương dùng chung một mô hình.

**Acceptance Criteria:**

**Given** Library
**When** mô hình hoá
**Then** hai tầng: **Tác phẩm → Chương**

**Given** một tài liệu đơn lẻ như hợp đồng hay bài báo
**When** đưa vào Library
**Then** biểu diễn là một Tác phẩm có **đúng một Chương**
**And** không có loại thực thể thứ ba nào

**Given** mỗi Tác phẩm
**When** tạo
**Then** mang tên, ảnh bìa *(tuỳ chọn)*, ngôn ngữ nguồn, lĩnh vực/thể loại, ngày tạo, ngày sửa gần nhất

**Given** ngôn ngữ nguồn
**When** đặt
**Then** đặt lúc tạo Tác phẩm và **không đổi được** về sau

**Given** Glossary và Translation Memory
**When** gắn
**Then** gắn ở **tầng Tác phẩm** — mọi Chương trong cùng Tác phẩm dùng chung

**Given** tên thực thể trong mã
**When** đặt
**Then** Tác phẩm → `Work`, Chương → `Chapter`
**And** cấm `Project`, `Book`, `Novel`, `Document`

**Given** `work.id`
**When** tạo
**Then** UUID v4 trong `meta.json`
**And** `chapter.id` là số nguyên cục bộ trong `project.db`

---

### Story 5.2: Chỉ mục Library dẫn xuất, một đường ghi duy nhất

**Covers:** FR98

As a người dịch,
I want xoá một file chỉ mục hỏng mà không mất gì cả,
So that thao tác sửa chữa hiển nhiên nhất không phải là thao tác nguy hiểm nhất.

**Acceptance Criteria:**

**Given** `library-index.db`
**When** tạo
**Then** là một **file riêng**, không nằm chung `global.db`

**Given** toàn bộ mã nguồn
**When** rà
**Then** **chỉ** component `Indexer` ghi vào `library-index.db`

**Given** một thay đổi dữ liệu
**When** ghi
**Then** `.atproj` ghi **trước**, `library-index.db` ghi **sau**

**Given** người dùng xoá `library-index.db`
**When** mở lại ứng dụng
**Then** chỉ mục dựng lại **hoàn toàn** từ các `.atproj`
**And** không mất một byte dữ liệu nào

**Given** xoá `library-index.db`
**When** kiểm
**Then** Glossary toàn cục, TM toàn cục và mọi dữ liệu trong `global.db` **không bị ảnh hưởng**

**Given** `library-index.db`
**When** nâng cấp ứng dụng
**Then** **không di trú** — xoá và dựng lại

**Given** danh sách, lọc, sắp xếp và tìm kiếm của Library
**When** thực hiện
**Then** đọc từ `library-index.db`
**And** `meta.json` là thứ `Indexer` đọc khi quét, không phải nguồn Library đọc trực tiếp lúc chạy

---

### Story 5.3: Quét lại thư mục

**Covers:** FR99

As a người dịch,
I want copy một thư mục `.atproj` vào là nó xuất hiện trong Library,
So that tôi quản lý dữ liệu của mình bằng file như mọi thứ khác trên máy.

**Acceptance Criteria:**

**Given** một `.atproj` mới xuất hiện trong thư mục gốc Library
**When** quét lại
**Then** nó vào chỉ mục và hiện trong Library

**Given** một `.atproj` đã bị di chuyển
**When** quét lại
**Then** chỉ mục cập nhật đường dẫn mới

**Given** một `.atproj` đã bị xoá
**When** quét lại
**Then** mục tương ứng hiện là **mục mồ côi**, nêu rõ nó trỏ tới đâu
**And** người dùng chọn gỡ khỏi chỉ mục hoặc giữ để tìm lại

**Given** hai `.atproj` mang **trùng `work.id`**
**When** quét lại
**Then** hệ thống phát hiện và **cảnh báo**
**And** không âm thầm gộp hay ghi đè

**Given** thư mục gốc Library
**When** mặc định
**Then** `~/Documents/AuraTranslate/`
**And** người dùng đổi được qua hộp thoại chọn thư mục

**Given** quét lại trên một thư viện lớn
**When** chạy
**Then** không chặn thao tác của người dùng

**Given** thao tác quét lại
**When** gọi
**Then** là command đăng ký, gán phím được

---

### Story 5.4: Bốn trạng thái vòng đời

**Covers:** FR5, FR6

As a người dịch có 2000 chương chưa đụng tới,
I want phân biệt *"chưa bắt đầu"* với *"đã làm dở rồi bỏ"*,
So that tôi biết chương nào thật sự cần quay lại.

**Acceptance Criteria:**

**Given** trạng thái vòng đời
**When** mô hình hoá
**Then** **bốn** giá trị: *Chưa bắt đầu · Đang dịch · Tạm ngưng · Đã xong*

**Given** trạng thái
**When** áp
**Then** có ở **cả** tầng Tác phẩm và tầng Chương

**Given** một Chương mới nhập
**When** tạo
**Then** mặc định *Chưa bắt đầu*

**Given** trạng thái các Chương trong một Tác phẩm thay đổi
**When** xảy ra
**Then** trạng thái Tác phẩm **suy ra tự động** theo

**Given** người dùng ghi đè thủ công trạng thái Tác phẩm
**When** xảy ra
**Then** trạng thái giữ giá trị người dùng đặt
**And** hệ thống không tự suy ra đè lên nữa cho tới khi người dùng bỏ ghi đè

**Given** một Tác phẩm ở trạng thái ghi đè thủ công
**When** hiển thị
**Then** **phân biệt được** với trạng thái suy ra tự động

**Given** lọc Library theo trạng thái
**When** thực hiện
**Then** bốn giá trị lọc được riêng rẽ

---

### Story 5.5: Tiến độ Tác phẩm

**Covers:** FR7

As a người dịch,
I want liếc qua Library là biết mình còn bao nhiêu,
So that tôi không phải mở từng Tác phẩm ra đếm.

**Acceptance Criteria:**

**Given** một Tác phẩm
**When** hiển thị trong Library
**Then** thấy số Chương đã xong trên tổng số kèm **thanh tiến độ trực quan**

**Given** tiến độ
**When** tính
**Then** suy ra từ trạng thái các Chương trong `project.db`

**Given** `meta.json`
**When** ghi
**Then** là **cache dẫn xuất** từ `project.db`, dựng lại được hoàn toàn
**And** ghi bởi **chính `store::Writer` của Tác phẩm đó**, trong cùng thao tác logic với thay đổi sinh ra nó

**Given** toàn bộ mã nguồn
**When** rà
**Then** **không thành phần nào khác** ghi vào `meta.json`
**And** không thành phần nào coi `meta.json` là nguồn sự thật

**Given** một Chương đổi trạng thái trong Workspace
**When** quay về Library
**Then** tiến độ đã cập nhật, không hiện số cũ

---

### Story 5.6: Lưới Tác phẩm, lọc và sắp xếp

**Covers:** FR10

As a người dịch,
I want mở ứng dụng là thấy ngay mình đang có gì,
So that Library là điểm vào chứ không phải một màn hình phụ.

**Acceptance Criteria:**

**Given** ứng dụng khởi động
**When** mở
**Then** vào **Library**, không vào Workspace

**Given** lưới Tác phẩm
**When** hiển thị
**Then** mỗi Tác phẩm có ảnh bìa, tên, tiến độ và trạng thái

**Given** Library
**When** lọc
**Then** lọc được theo trạng thái, lĩnh vực, ngôn ngữ nguồn

**Given** Library
**When** sắp xếp
**Then** sắp được theo **ngày sửa gần nhất** và theo tên

**Given** Library chưa có Tác phẩm nào
**When** hiển thị
**Then** trạng thái rỗng giải thích **Tác phẩm là gì và là một thư mục mang đi được**, rồi mới mời nhập

**Given** một Tác phẩm không có ảnh bìa
**When** hiển thị
**Then** dùng một biểu diễn thay thế nhất quán, không để ô trống

**Given** mọi thao tác lọc, sắp xếp và điều hướng trong lưới
**When** thực hiện
**Then** làm được bằng bàn phím

---

### Story 5.7: Danh sách Chương và mở Chương vào Workspace

**Covers:** FR12

As a người dịch,
I want mở đúng chương đang dở và thấy con trỏ ở đúng câu tôi bỏ dở,
So that tôi không mất năm phút tìm lại chỗ mỗi lần ngồi vào.

**Acceptance Criteria:**

**Given** một Tác phẩm
**When** mở
**Then** thấy danh sách Chương kèm trạng thái từng Chương

**Given** một danh sách 2000 Chương
**When** hiển thị
**Then** cuộn mượt
**And** không nạp toàn bộ vào DOM cùng lúc

**Given** một Chương
**When** người dùng mở
**Then** vào thẳng Workspace tại đúng Chương đó

**Given** một Chương đã từng làm việc
**When** mở lại
**Then** khôi phục **đúng segment đang làm và đúng vị trí cuộn** lần trước

**Given** một Chương chưa từng mở
**When** mở
**Then** con trỏ ở segment đầu tiên

**Given** vị trí làm việc
**When** lưu
**Then** lưu trong `project.db`, không phải trạng thái tạm của frontend

**Given** điều hướng trong danh sách Chương và mở Chương
**When** thực hiện
**Then** làm được bằng bàn phím

---

### Story 5.8: Tổ chức lại Chương sau khi nhập

**Covers:** FR15

As a người dịch,
I want sửa lại thứ tự và ranh giới Chương sau khi nhập mà không mất công đã dịch,
So that một lần tách sai không bắt tôi làm lại từ đầu.

**Acceptance Criteria:**

**Given** một Chương
**When** người dùng đổi tên
**Then** tên đổi và mọi tham chiếu giữ nguyên

**Given** nhiều Chương
**When** người dùng sắp xếp lại thứ tự
**Then** **chỉ cột `ord` đổi**

**Given** hai Chương liền nhau
**When** người dùng gộp
**Then** **chỉ `chapter_id` và `ord`** của các segment liên quan đổi

**Given** một Chương
**When** người dùng tách
**Then** **chỉ `chapter_id` và `ord`** của các segment liên quan đổi

**Given** gộp hoặc tách Chương
**When** xảy ra
**Then** `segment.id` **giữ nguyên**

**Given** gộp hoặc tách Chương
**When** xảy ra
**Then** lịch sử phiên bản, trạng thái xác nhận và mọi dữ liệu gắn theo segment **giữ nguyên**

**Given** một Tác phẩm có nhiều Chương đã dịch xong
**When** gộp hoặc tách Chương
**Then** **không segment nào bị về hưu** và không trạng thái xác nhận nào bị mất

**Given** thao tác này
**When** so với gộp/tách **segment** ở Story 2.8
**Then** đây là thao tác **tổ chức**, không đụng tới văn bản của segment nào — khác biệt cố ý

---

### Story 5.9: Tìm kiếm full-text xuyên Library

**Covers:** FR8 · NFR3

As a người dịch,
I want tìm một câu tôi từng dịch ở đâu đó trong cả thư viện,
So that công sức cũ của tôi tìm lại được.

**Acceptance Criteria:**

**Given** một truy vấn
**When** tìm
**Then** tìm **đồng thời** trong văn bản nguồn và văn bản dịch của mọi Tác phẩm

**Given** kết quả tìm kiếm
**When** trả về
**Then** kèm **Tác phẩm, Chương và đoạn văn bản khớp**

**Given** một kết quả
**When** người dùng chọn
**Then** mở Workspace tại đúng Chương và đúng chỗ khớp

**Given** chỉ mục tìm kiếm **chính**
**When** tạo
**Then** dùng `remove_diacritics 0`

**Given** truy vấn `má`
**When** tìm ở chế độ mặc định
**Then** chỉ ra kết quả chứa `má`
**And** **không** ra `ma`, `mà`, `mả`, `mã`, `mạ`

**Given** tìm kiếm
**When** thực hiện
**Then** đọc từ `library-index.db`

**Given** một truy vấn không khớp gì trong cả Library
**When** tìm ở chế độ chính xác dấu
**Then** hiện trạng thái rỗng nêu **vì sao rỗng** và **làm gì tiếp** — mời thử chế độ khoan dung không dấu (Story 5.10)
**And** **khác** với trạng thái chưa gõ gì vào ô tìm kiếm
**And** không phải một danh sách trống không giải thích

**Given** một thư viện lớn
**When** tìm
**Then** đo và ghi lại p95 để đối chiếu ngưỡng NFR3

---

### Story 5.10: Hai chế độ dấu

**Covers:** FR9

As a người dịch hay gõ không dấu cho nhanh,
I want công cụ vẫn tìm ra thứ tôi cần,
So that tôi không phải chọn giữa nhanh và chính xác.

**Acceptance Criteria:**

**Given** một truy vấn
**When** tìm
**Then** hệ thống thử **chế độ chính xác dấu trước**

**Given** chế độ chính xác không có kết quả
**When** xảy ra
**Then** hệ thống nới sang chế độ khoan dung không dấu
**And** **nói rõ nó đã nới**

**Given** người dùng chủ động chọn chế độ khoan dung
**When** tìm
**Then** dùng chỉ mục xoá dấu

**Given** chỉ mục xoá dấu
**When** tồn tại
**Then** chỉ là chỉ mục **phụ**
**And** **không bao giờ là mặc định**

**Given** kết quả từ chế độ khoan dung
**When** hiển thị
**Then** **phân biệt được** với kết quả khớp chính xác

**Given** chế độ tìm kiếm hiện hành
**When** hiển thị
**Then** người dùng luôn biết mình đang ở chế độ nào

---

### Story 5.11: Chế độ đọc — typography và bố cục đọc dài

**Covers:** FR11

As a người dịch,
I want đọc lại bản dịch của mình như đọc một trang sách,
So that tôi thưởng thức thành quả chứ không nhìn một bảng dữ liệu.

**Acceptance Criteria:**

**Given** Chế độ đọc
**When** mở
**Then** **không hiển thị công cụ biên tập nào** — không vạch lề trạng thái, không nút xác nhận, không panel

**Given** ba mức đọc
**When** chọn
**Then** *Thoáng* 62ch / 19px / 1.95 · ***Cân* 68ch / 17,5px / 1.8 (mặc định)** · *Đặc* 76ch / 16px / 1.66

**Given** chiều rộng dòng
**When** đo
**Then** đo bằng `ch`, không bằng `px`
**And** số ký tự mỗi dòng giữ nguyên khi đổi cỡ chữ

**Given** giãn dòng
**When** đặt ở bất kỳ mức nào
**Then** **không bao giờ dưới 1.66**

**Given** điều khiển đọc
**When** hiển thị
**Then** ba preset trên thanh công cụ
**And** thanh trượt cỡ chữ và giãn dòng chi tiết sau một lần bấm

**Given** chế độ sáng và chế độ tối
**When** chuyển
**Then** cả hai đạt WCAG AA

**Given** mặc định
**When** mở
**Then** chỉ hiển thị bản dịch tiếng Việt

**Given** công tắc song ngữ bật
**When** hiển thị
**Then** nguyên văn đặt ở **lề trái**, cỡ nhỏ, màu `on-surface-variant`
**And** **không chen giữa dòng đọc**

**Given** phím `B`, `1` `2` `3`, `⌘,`, `D`, `⌘L`
**When** bấm
**Then** lần lượt bật song ngữ, ba mức chữ, tinh chỉnh, sáng/tối, mục lục

---

### Story 5.12: Chế độ đọc chỉ đọc phần đã xong

**Covers:** FR120

As a người dịch,
I want Chế độ đọc không bao giờ ném nguyên văn tiếng Trung vào giữa trang đọc tiếng Việt của tôi,
So that phiên đọc không bị gãy.

**Acceptance Criteria:**

**Given** nhiều Chương ở trạng thái *Đã xong*
**When** đọc
**Then** đọc liên tục qua chúng, chuyển Chương liền mạch, không phải quay về Library

**Given** chạm tới một Chương chưa ở trạng thái *Đã xong*
**When** đọc
**Then** dừng ở một **mốc biên giới tường minh** báo đã hết phần đã dịch

**Given** mốc biên giới
**When** hiển thị
**Then** kèm một đường sang Workspace để dịch tiếp

**Given** một Chương chưa dịch
**When** ở Chế độ đọc
**Then** **không hiển thị nguyên văn** của nó

**Given** một câu **chưa xác nhận** nằm trong một Chương đã xong
**When** hiển thị
**Then** vẫn hiện nhưng có **gạch chấm nhẹ** phân biệt được

**Given** một Chương được đánh dấu *Đã xong* bằng tay trong khi còn câu chưa xác nhận
**When** đọc
**Then** các câu đó vẫn mang dấu
**And** không hiện như thể đã hoàn chỉnh

---

### Story 5.13: Đánh dấu chỗ cần sửa khi đang đọc

**Covers:** FR119

As a người dịch đang đọc lại bản của mình,
I want đánh dấu một câu sai rồi đọc tiếp ngay,
So that tôi không mất chỗ nào mà cũng không đứt phiên đọc.

**Acceptance Criteria:**

**Given** con trỏ chuột hoặc tiêu điểm bàn phím chạm tới một câu
**When** xảy ra
**Then** affordance đánh dấu hiện ra

**Given** không có gì chạm tới câu nào
**When** hiển thị
**Then** affordance **ẩn hoàn toàn** — trang đọc sạch

**Given** một câu đang được chạm tới
**When** người dùng bấm `M`
**Then** câu được đánh dấu và **phiên đọc tiếp tục ngay**, không chuyển màn hình

**Given** một câu đang được chạm tới
**When** người dùng bấm `↵`
**Then** nhảy sang Workspace tại đúng segment đó

**Given** nhiều chỗ đã đánh dấu
**When** mở danh sách
**Then** gom thành **một danh sách theo Tác phẩm**

**Given** một mục trong danh sách đánh dấu
**When** người dùng chọn
**Then** mở Workspace tại đúng segment để sửa

**Given** một chỗ đánh dấu trỏ tới segment **đã về hưu** do gộp/tách
**When** hiển thị
**Then** vẫn **ở lại**, kèm ghi chú *câu này đã đổi*
**And** vẫn mở được về đúng vị trí trong Chương

**Given** cơ chế này
**When** so với lệnh cấm công cụ biên tập của FR11
**Then** không vi phạm — một dấu và một đường điều hướng **không sửa gì cả**

---

### Story 5.14: Đo NFR3, NFR4, NFR5 và ghi lại trạng thái ba ngưỡng tạm

**Covers:** NFR3 · NFR4 · NFR5 *(đo **sơ bộ** — phép đo nghiệm thu ở Story 6.18)*

As a chủ dự án,
I want có số đo thật thay vì ba giả định,
So that tôi biết ngưỡng nào cần hiệu chỉnh và ngưỡng nào **chưa kiểm được**.

**Acceptance Criteria:**

**Given** thư viện lớn nhất dựng được ở thời điểm này
**When** đo tìm kiếm full-text
**Then** p95 được ghi lại **kèm quy mô thư viện đã dùng**

**Given** cùng thư viện đó
**When** đo thời gian khởi động tới lúc Library dùng được
**Then** con số được ghi lại kèm quy mô

**Given** ứng dụng ở trạng thái nhàn rỗi
**When** đo bộ nhớ
**Then** con số được ghi lại

**Given** quy mô đo được **nhỏ hơn 5.000 Chương**
**When** báo cáo
**Then** ghi rõ NFR3, NFR4 và NFR5 **VẪN TREO**
**And** **Q4 chưa đóng**

**Given** không có đường nào tạo ra 5.000 Chương trước Epic 6
**When** báo cáo
**Then** nêu tường minh rằng phép đo đầy đủ **phải chạy lại sau Epic 6**
**And** phép chạy lại đó là **Story 6.18**, có chủ và có nghiệm thu riêng — không phải một lời nhắc trôi nổi

**Given** dữ liệu sinh giả dùng để đo tốc độ
**When** dùng
**Then** ghi rõ nó **không kiểm được NFR8**, vì phân bố dấu tiếng Việt trong dữ liệu bịa không phản ánh văn bản thật

**Given** một ngưỡng đo được lệch xa giá trị tạm
**When** xảy ra
**Then** đề xuất giá trị hiệu chỉnh
**And** ghi vào mục assumptions của `SPEC.md`

---

## Epic 6: Đường nhập — mọi nguồn văn bản vào được, và không hỏng im lặng

Đây là **bề mặt đầu tiên người dùng chạm vào sản phẩm**, và là nơi hai lỗi đắt nhất của ứng dụng có thể xảy ra mà không báo gì cả. Người dịch nhập một bộ 2000 chương từ một file `.txt` 40 MB, hoặc **dán 50 link web**, hoặc một file song ngữ hai cột do người khác dịch — tất cả đi qua **một màn xem trước hợp nhất** cho thấy bảng mã đã đoán, ranh giới nội dung đã bóc, và những gì luật làm sạch **sắp xoá** — trước khi một byte nào ghi xuống đĩa. Ảnh trong bài web tải về nằm trong `.atproj`; alt-text và caption là **hai segment dịch được riêng biệt**. Và ứng dụng **không bao giờ tự quyết định tải cái gì**.

### Story 6.1: Mũi thăm dò ba lựa chọn thư viện

**Covers:** A12 · A13 · mũi thăm dò bắt buộc

As a chủ dự án,
I want biết trước ba thư viện nền của đường nhập có dùng được không và có tương thích GPL v3 không,
So that tôi không phát hiện ra một crate không rà được giấy phép sau khi đã dựng nửa epic lên nó.

**Acceptance Criteria:**

**Given** `dom_smoothie` 0.18.0 làm ứng viên bóc nội dung
**When** bóc thử trên các website chủ dự án **thật sự dùng**
**Then** tỉ lệ bóc sai được đo và ghi lại
**And** kết luận ghim hay không ghim được ghi vào bảng Stack

**Given** tỉ lệ bóc sai cao
**When** báo cáo
**Then** **không chặn tiến độ** — đường sửa ranh giới bằng tay là điều kiện nghiệm thu của FR123
**And** kết quả chỉ đổi mức đầu tư vào màn xem trước

**Given** `chardetng` 1.0.0 + `encoding_rs` 0.8.35 làm ứng viên dò bảng mã
**When** dò thử trên `.txt` mã **GBK và Big5 thật lấy từ diễn đàn**
**Then** tỉ lệ dò đúng được đo và ghi lại

**Given** tỉ lệ dò đúng thấp
**When** báo cáo
**Then** **không đổi kiến trúc** — đường đổi bảng mã bằng tay ở màn xem trước là phương án dự phòng **theo thiết kế**

**Given** HTTP client cho `Fetcher`
**When** chọn
**Then** thoả ba nhu cầu: **theo dõi chuyển hướng để cưỡng chế allowlist**, giới hạn kích thước, timeout
**And** ưu tiên dùng lại `reqwest` đã có trong Stack thay vì thêm phụ thuộc

**Given** mỗi thư viện được chọn
**When** đưa vào dự án
**Then** đã rà tương thích GPL v3 **trước khi** thêm
**And** ghi vào bảng Stack của `ARCHITECTURE-SPINE.md`

---

### Story 6.2: Pipeline nhập một chuỗi thứ tự cố định, dùng chung mọi nguồn

**Covers:** AD-39 — pipeline dùng chung cho mọi FR nhập của epic này

As a người dịch,
I want mọi nguồn văn bản đi qua đúng một chuỗi xử lý theo đúng một thứ tự,
So that thứ tôi nhìn thấy ở màn xem trước chính là thứ được ghi xuống đĩa.

**Acceptance Criteria:**

**Given** chuỗi xử lý nhập
**When** cài đặt
**Then** đúng thứ tự: giải mã bảng mã → bóc nội dung chính → làm sạch theo luật → chuẩn hoá đoạn & khoảng trắng → **tách Chương theo mẫu phân tách** → xem trước + sửa tay → tách segment + cờ kết đoạn → ghi xuống `.atproj`

**Given** chuỗi này
**When** đặt trong cây nguồn
**Then** sống ở `core/segment/`
**And** các module nguồn chỉ **cung cấp bước đầu vào** rồi trao lại, không giữ bản sao của các bước dùng chung

**Given** ba đường nhập file, URL và song ngữ
**When** chạy
**Then** khác nhau **chỉ ở bước đầu vào** và ở việc **bỏ qua** bước không áp dụng
**And** không đường nào đổi thứ tự hay chèn bước sau lệnh ghi

**Given** điều kiện áp bước tách Chương
**When** khai
**Then** khai theo **hình dạng đầu vào**, không theo danh sách đường nhập — *một dòng văn bản chưa chia Chương* → có tách; *đã một đơn vị một Chương* → không tách

**Given** một file `.txt` 40 MB mã GBK chứa 2000 chương
**When** chạy pipeline với bước tách Chương đặt **trước** bước giải mã bảng mã
**Then** một test tự động **thất bại** — đây là ca hỏng cụ thể mà AD-39 tồn tại để chặn

**Given** một nguồn `.docx`
**When** chạy pipeline
**Then** **bỏ qua** bước giải mã bảng mã — nó là zip chứa XML đã khai encoding

**Given** màn xem trước
**When** hiển thị
**Then** luôn hiện kết quả **sau toàn bộ chuỗi**, không phải sau một bước giữa chừng

---

### Story 6.3: Bảng mã — phát hiện và dải đối chiếu năm bản dựng thật

**Covers:** FR126

As a người dịch nhập một file tải từ diễn đàn Trung Quốc,
I want thấy ngay bằng mắt rằng bảng mã đang sai và sửa được trong một giây,
So that tôi không ngồi sửa ranh giới trên một văn bản đã hỏng.

**Acceptance Criteria:**

**Given** một nguồn không tự khai bảng mã — `.txt`, `.md`, phản hồi HTTP
**When** nhập
**Then** hệ thống tự phát hiện trong năm bảng mã: **UTF-8 · GB18030 · GBK · Big5 · UTF-16**

**Given** trạng thái bảng mã
**When** hiển thị
**Then** một trong ba giá trị: **nguồn tự khai** · **tự đoán tin cậy cao** · **tự đoán tin cậy thấp**

**Given** độ tin cậy dò **thấp**
**When** hiển thị
**Then** một dải mở ra **ngay trên văn bản** với năm ứng viên
**And** mỗi ứng viên kèm **bản dựng thật** của cùng đoạn 6–8 ký tự đầu Chương

**Given** mẫu chữ trong dải đối chiếu
**When** hiển thị
**Then** đặt ở cỡ `read` chứ không cỡ giao diện — phải đủ lớn để phân biệt nét chữ Hán

**Given** người dùng chọn một bảng mã khác
**When** xảy ra
**Then** toàn bộ chuỗi chạy lại **từ bước một, trong bộ nhớ**
**And** kết quả ở cả ba tầng của màn xem trước dựng lại ngay lập tức

**Given** một file `.txt` mã GBK chứa 2000 chương
**When** nhập với bảng mã đúng
**Then** ra **chữ Hán đúng**, và mẫu phân tách nhận ra đúng số Chương

**Given** hệ thống đoán sai bảng mã
**When** người dùng sửa
**Then** sửa được **mà không phải nhập lại từ đầu**

**Given** một file đọc sai bảng mã
**When** xảy ra
**Then** **không có trạng thái lỗi** — nó chỉ ra chữ không đọc được, và đó là thứ mắt phân xử

---

### Story 6.4: Chuẩn hoá xuống dòng và khoảng trắng

**Covers:** FR125

As a người dịch,
I want văn bản tải về được dọn cho gọn trước khi tách câu,
So that một dòng bị ngắt tuỳ tiện không biến thành hai segment.

**Acceptance Criteria:**

**Given** văn bản có dòng bị ngắt tuỳ tiện giữa câu
**When** chuẩn hoá
**Then** các dòng đó được gộp lại

**Given** văn bản có dòng trống thừa
**When** chuẩn hoá
**Then** xoá dòng trống thừa và thống nhất cách phân đoạn

**Given** bước chuẩn hoá
**When** chạy trong pipeline
**Then** chạy **trước** bước tách segment

**Given** kết quả chuẩn hoá
**When** ghi
**Then** **kết quả chuẩn hoá là thứ được lưu xuống**
**And** không phải một lớp hiển thị đắp lên trên văn bản gốc

**Given** ranh giới segment
**When** tính
**Then** tính trên văn bản **đã chuẩn hoá**, một lần, và không bao giờ tính lại

**Given** màn xem trước
**When** hiển thị
**Then** hiện văn bản **đã chuẩn hoá** — đúng thứ sẽ được ghi

---

### Story 6.5: Luật làm sạch lộ ra và hiện thứ sắp xoá

**Covers:** FR124

As a người dịch,
I want thấy chính xác luật nào sắp xoá chữ nào trước khi nó xoá,
So that một luật ẩn không xoá nhầm một câu thật trong 2000 chương mà không ai biết.

**Acceptance Criteria:**

**Given** luật làm sạch
**When** mô hình hoá
**Then** là một **danh sách mẫu** — chuỗi hoặc biểu thức chính quy

**Given** danh sách luật
**When** người dùng mở
**Then** **xem được, sửa được và tắt được** từng luật

**Given** một chỗ trong văn bản bị một luật khớp
**When** hiển thị ở màn xem trước
**Then** hiện **gạch ngang tại chỗ trong văn bản** bằng nét `ornament`, kèm nhãn luật đã khớp

**Given** mỗi luật trong danh sách
**When** hiển thị
**Then** ghi **hai con số** — khớp bao nhiêu chỗ *trong Chương này* và bao nhiêu chỗ *trong cả lần nhập*

**Given** mỗi luật
**When** hiển thị
**Then** mang **nhãn tầng** — *Toàn cục* hoặc *Tác phẩm*

**Given** luật ở cả hai tầng
**When** áp
**Then** **cả hai tầng cùng áp** — ngữ nghĩa **hợp nhất**, không phải ghi đè
**And** phân giải đi qua `ScopeResolver`

**Given** người dùng tắt một luật
**When** xảy ra
**Then** văn bản dựng lại ngay, chỗ vừa gạch ngang trở về nguyên trạng

**Given** phím `R` khi đang chọn một khối
**When** bấm
**Then** bật/tắt luật đang khớp khối đó

---

### Story 6.6: Tách Chương theo mẫu phân tách

**Covers:** FR14

As a người dịch có một file 40 MB chứa cả bộ truyện,
I want cấu hình mẫu nhận diện đầu chương và thấy ngay kết quả tách,
So that tôi không phát hiện ra 14 chương sai sau khi đã dịch 200 chương.

**Acceptance Criteria:**

**Given** một file lớn hoặc văn bản dán chứa nhiều Chương
**When** nhập
**Then** hệ thống mời tách thành nhiều Chương thay vì tạo một Chương khổng lồ

**Given** mẫu phân tách
**When** người dùng cấu hình
**Then** nhận cả **mẫu tiêu đề** và **biểu thức chính quy**

**Given** mẫu phân tách hiện tại
**When** màn xem trước hiển thị
**Then** hiện **đã nhận ra bao nhiêu Chương**, ba Chương đầu và ba Chương cuối trông thế nào, và **những chỗ mẫu không khớp**

**Given** người dùng sửa mẫu
**When** xảy ra
**Then** kết quả tách cập nhật **ngay**, không cần bấm nút chạy lại

**Given** mẫu bắt nhầm một dòng không phải tiêu đề chương
**When** hiển thị
**Then** chỗ bắt nhầm nhìn thấy được trong màn xem trước để người dùng phát hiện

**Given** màn xem trước chưa được xác nhận
**When** ở bất kỳ thời điểm nào
**Then** **không có gì ghi xuống đĩa**

**Given** người dùng chọn nhiều file cùng lúc
**When** nhập
**Then** mỗi file thành một Chương hoặc được tách tiếp theo mẫu, theo lựa chọn của người dùng

**Given** người dùng xác nhận
**When** nhập chạy
**Then** các Chương vào Library ở trạng thái **Chưa bắt đầu**

---

### Story 6.7: Nhập từ URL bằng danh sách link

**Covers:** FR122

As a người dịch lấy truyện từ website,
I want dán một danh sách link và biết chắc ứng dụng chỉ tải đúng những link đó,
So that công cụ local-first của tôi không âm thầm biến thành một con crawler.

**Acceptance Criteria:**

**Given** người dùng dán một danh sách link, **mỗi dòng một Chương**
**When** nhập
**Then** hệ thống xử lý **đúng thứ tự đã cho**

**Given** danh sách link
**When** người dùng chọn đích
**Then** tạo Tác phẩm mới, **hoặc** thêm Chương vào một Tác phẩm sẵn có

**Given** ô dán link
**When** hiển thị
**Then** ngay dưới có **hai con số cạnh nhau**: *`N` link · sẽ tạo `N` Chương*
**And** một câu *"Chỉ tải đúng N link này. Không tìm thêm link nào khác."*

**Given** hai con số đó
**When** không bằng nhau
**Then** đây là dấu hiệu ứng dụng đang tự tìm thêm link — một test tự động **thất bại**

**Given** ứng dụng
**When** chạy đường nhập URL
**Then** **không quét trang mục lục**, **không lần theo "chương sau"**, **không tự tìm bất kỳ link nào ngoài danh sách được cấp**

**Given** một link trong danh sách hỏng — 404, timeout, hoặc bị tường chặn
**When** xảy ra
**Then** hành vi đã chốt được áp nhất quán cho mọi link hỏng
**And** mọi thứ vẫn xảy ra **trước bước ghi**, nên không có ca ghi nửa chừng

**Given** người dùng chưa bấm nút tải
**When** quan sát lưu lượng mạng
**Then** **không có lời gọi nào**

---

### Story 6.8: Allowlist mạng hai tầng và nhật ký domain

**Covers:** NFR19 · NFR12 · AD-40 · AD-41 *(bộ test riêng bắt buộc)*

As a người dịch,
I want kiểm chứng được bằng mắt rằng ứng dụng chỉ gọi tới những nơi tôi cho phép,
So that lời hứa *"không telemetry"* là thứ tôi quan sát được chứ không phải thứ tôi phải tin.

**Acceptance Criteria:**

**Given** một lần nhập từ URL
**When** bắt đầu
**Then** allowlist được lập và **sống đúng một lần nhập** — hết lần nhập thì hết hiệu lực

**Given** **tầng 1** của allowlist
**When** lập
**Then** gồm host của các link trong danh sách vừa dán
**And** được phép tải **tài liệu**

**Given** **tầng 2** của allowlist
**When** lập
**Then** gồm host của tài nguyên **được tham chiếu từ trang đã tải ở tầng 1**, trong **cùng lần nhập**
**And** chỉ được phép tải **ảnh, không bao giờ tài liệu**

**Given** một host **ngoài** cả hai tầng
**When** `Fetcher` gặp
**Then** **từ chối**

**Given** một lời gọi bị **chuyển hướng** tới host ngoài allowlist
**When** xảy ra
**Then** **từ chối** — allowlist cưỡng chế cả khi gặp chuyển hướng

**Given** một yêu cầu tải **tài liệu** từ host tầng 2
**When** xảy ra
**Then** **từ chối**

**Given** mọi lời gọi mạng của đường nhập
**When** thực hiện
**Then** ghi `(thời điểm, domain, tầng, kết quả)` vào một nhật ký

**Given** nhật ký domain
**When** hiển thị
**Then** một dòng tóm tắt ở chân màn xem trước — *"Đã gọi `N` domain · xem"*
**And** bảng đầy đủ trong **Cài đặt › Quyền riêng tư**

**Given** hai tầng allowlist trong nhật ký
**When** hiển thị
**Then** phân biệt bằng **nhãn chữ** — `Tài liệu` và `Ảnh`, **không bằng màu**
**And** mỗi hàng ghi **vì sao được phép**

**Given** một ảnh đã có trong Tác phẩm, so theo `source_url`
**When** nhập lại
**Then** **không tải lại**

**Given** `Fetcher`
**When** cài đặt
**Then** **không bao giờ** phân tích nội dung
**And** `Extractor` **không bao giờ** chạm mạng

**Given** bốn ràng buộc trên
**When** kiểm
**Then** có **bộ test tự động riêng**, vì Tauri capabilities khai tĩnh lúc build nên không cưỡng chế được ràng buộc này

---

### Story 6.9: Bóc nội dung chính và sửa ranh giới bằng bàn phím

**Covers:** FR123

As a người dịch,
I want sửa lại chỗ thuật toán bóc thiếu hoặc bóc thừa, bằng bàn phím,
So that một tỉ lệ sai chấp nhận được vẫn ra một công cụ dùng được.

**Acceptance Criteria:**

**Given** một trang web đã tải
**When** bóc nội dung
**Then** dùng **một thuật toán chung**, không có bộ đọc riêng cho từng website

**Given** HTML thô
**When** xử lý
**Then** phân tích và bóc chạy **trọn ở Rust**
**And** **không byte HTML thô nào đi qua IPC** — thứ đi qua IPC là mô hình đã bóc

**Given** mô hình nội dung
**When** định nghĩa
**Then** mang đoạn, câu, ảnh, caption
**And** **không có nhánh nào mang chuỗi đánh dấu HTML**

**Given** nội dung đã bóc
**When** hiển thị ở màn xem trước
**Then** chia thành **khối theo đoạn**, mỗi khối mang một trong ba trạng thái đọc ở vạch lề

**Given** ba trạng thái khối
**When** hiển thị
**Then** `confirmed` giữ lại người dùng đã chạm · `tm-rule` giữ lại máy đoán chưa ai xác nhận · `ornament` mờ đã loại

**Given** một khối **đã loại**
**When** hiển thị
**Then** chìm xuống `surface-sunken`, chữ rút về `on-surface-variant`, **và đổi từ chữ đọc sang chữ giao diện cỡ nhỏ**
**And** phân biệt bằng độ lùi, **không bằng một màu nhấn thứ hai**

**Given** người dùng chạm vào một ranh giới do máy đoán
**When** xảy ra
**Then** vạch chuyển từ `tm-rule` sang `confirmed`

**Given** phím `J` `K` hoặc `↑` `↓`
**When** bấm
**Then** đi giữa các khối

**Given** phím `Space`
**When** bấm
**Then** bật/tắt giữ khối đang chọn

**Given** phím `[` và `]`
**When** bấm
**Then** đặt đầu và cuối vùng giữ một lần

**Given** kéo chuột
**When** dùng
**Then** vẫn dùng được, nhưng là **đường thứ hai**
**And** mọi phím trên là command đăng ký trong `CommandRegistry`

**Given** một bản chỉ có thuật toán mà không có đường sửa tay
**When** nghiệm thu
**Then** **chưa đạt** — đường sửa tay nằm trong nghiệm thu của FR123

---

### Story 6.10: Bộ lọc "cần xem"

**Covers:** FR132

As a người dịch dán 50 link cùng lúc,
I want công cụ chỉ cho tôi những Chương thật sự cần nhìn,
So that tôi không bấm xác nhận mù ở Chương thứ mười.

**Acceptance Criteria:**

**Given** một lần nhập nhiều Chương
**When** hiển thị màn xem trước
**Then** đầu màn hình luôn hiện **hai con số**: *`N` Chương cần xem* và *`M` Chương sạch*
**And** đây là nghiệm thu của **FR132** — bộ lọc này là điều kiện để FR123, FR124 và FR126 còn tác dụng ở quy mô năm mươi Chương, không phải một tiện ích giao diện

**Given** một Chương
**When** phân loại
**Then** mang một trong hai trạng thái: **sạch** hoặc **cần xem**

**Given** trạng thái *cần xem*
**When** xác định
**Then** gom **bốn** nguyên nhân: bảng mã tin cậy thấp · phần bóc ra **ngắn bất thường so với trung vị các Chương khác** · luật làm sạch xoá quá nhiều · link hỏng

**Given** một Chương *cần xem*
**When** hiển thị
**Then** nêu rõ **nguyên nhân nào** khiến nó được xếp vào nhóm đó

**Given** phím `⌥W`
**When** bấm
**Then** lọc về **chỉ nhóm cần xem**

**Given** phím `⌥←` và `⌥→`
**When** bấm
**Then** đi Chương trước / Chương sau trong cùng lần nhập

**Given** phím `⌘↵`
**When** bấm
**Then** xác nhận nhập toàn bộ

---

### Story 6.11: Ảnh tải về `.atproj`, neo vị trí, và URL gốc

**Covers:** FR45, FR127

As a người dịch,
I want ảnh trong bài nằm ngay trong thư mục Tác phẩm của tôi,
So that copy sang một máy ngoại tuyến vẫn đủ ảnh.

**Acceptance Criteria:**

**Given** ảnh trong nội dung tải từ web
**When** nhập
**Then** **tải về và lưu bên trong `.atproj/assets/`** như một file thật

**Given** một ảnh tải từ web
**When** lưu
**Then** **URL gốc lưu kèm làm metadata** trong `project.db`

**Given** một ảnh do người dùng tự thêm
**When** lưu
**Then** trường URL gốc **rỗng** — hợp lệ, không phải lỗi

**Given** mọi ảnh
**When** hiển thị
**Then** tham chiếu file trong `assets/` qua asset protocol
**And** **không bao giờ tham chiếu một URL từ xa**

**Given** một `.atproj` copy sang máy khác không có mạng
**When** mở
**Then** ảnh hiển thị đầy đủ

**Given** mỗi ảnh
**When** lưu
**Then** `ASSET` mang **neo vị trí của chính nó** trong Chương
**And** neo đó **độc lập** với việc có hay không có segment đi kèm

**Given** một ảnh trên web **không có thuộc tính `alt`**
**When** nhập
**Then** vẫn giữ đúng vị trí trong Chương nhờ neo riêng

**Given** không có loại ảnh thứ hai chỉ mang link
**When** kiểm mô hình dữ liệu
**Then** mọi ảnh là file thật trong `assets/`

---

### Story 6.12: Đọc `.docx`

**Covers:** FR13

As a người dịch,
I want mở một file `.docx` như mọi định dạng khác,
So that tôi không phải chuyển đổi thủ công trước khi đưa vào công cụ.

**Acceptance Criteria:**

**Given** một file `.docx` văn bản thường
**When** nhập
**Then** văn bản đọc ra đúng và đi vào pipeline từ bước **sau** giải mã bảng mã

**Given** một `.docx` chứa **bảng**
**When** đọc
**Then** lấy được số hàng, số ô, và **số đoạn bên trong từng ô**

**Given** khả năng lấy số đoạn trong ô
**When** kiểm
**Then** đây là **điều kiện tiên quyết của AD-38** ở Epic 8 — không có nó thì cổng kiểm hình dạng `.docx` không cài được

**Given** `docx-rs` 0.4.22 không đọc được số đoạn trong ô
**When** xảy ra
**Then** rà `docx-reader` hoặc `rdocx`
**And** **rà tương thích GPL v3 trước khi** đưa vào Stack

**Given** một `.docx` chứa ảnh
**When** đọc
**Then** ảnh được nhận ra và đi vào đường xử lý tài sản của Story 6.11

**Given** một `.docx` hỏng hoặc không đọc được
**When** nhập
**Then** báo lỗi nêu rõ vấn đề
**And** không ghi một phần

---

### Story 6.13: Alt-text và caption là hai `Segment` mang trường vai

**Covers:** FR44, FR129

As a người dịch bài báo,
I want dịch riêng chú thích ảnh và mô tả cho trình đọc màn hình,
So that bài đăng ra không mất chú thích và không đẩy nhầm một bản dịch sai chỗ.

**Acceptance Criteria:**

**Given** alt-text của một ảnh
**When** mô hình hoá
**Then** là một `Segment` bình thường mang trường **vai = `alt`**

**Given** caption của một ảnh
**When** mô hình hoá
**Then** là một `Segment` bình thường mang trường **vai = `caption`**

**Given** cả hai
**When** kiểm mô hình dữ liệu
**Then** **không phải cột text trên `ASSET`**

**Given** một caption
**When** người dùng dịch và xác nhận
**Then** nó đi qua **đúng luồng xác nhận của mọi segment khác** — cùng máy trạng thái, cùng đường ghi
**And** nghiệm thu bằng cách xác nhận một caption rồi kiểm nó chuyển trạng thái và sinh `SegmentVersion` **y hệt một segment văn xuôi**

**Given** caption và alt-text là `Segment` bình thường mang trường vai
**When** Translation Memory tồn tại ở Epic 7
**Then** việc chúng vào TM là **hệ quả tự động — không cần thêm một dòng mã nào**
**And** đây chính là lý do chúng **không được** mô hình hoá thành cột text trên `ASSET`: một cột text sẽ không bao giờ vào TM, và hỏng **im lặng**
**And** phần nghiệm thu TM thật đóng ở Story 7.1

**Given** alt-text và caption
**When** khớp thuật ngữ
**Then** tham gia Glossary như mọi segment khác

**Given** một ảnh
**When** kiểm
**Then** có **nhiều nhất một** segment mỗi vai

**Given** một ảnh **không có caption**
**When** nhập
**Then** **không sinh segment rỗng** — segment rỗng sẽ trôi vào TM và vào bộ đếm tiến độ

**Given** vị trí trong Chương
**When** đặt
**Then** segment vai `alt` treo **tại neo của ảnh**; segment vai `caption` treo **ngay sau neo của ảnh**

---

### Story 6.14: Hiển thị ảnh đúng vị trí

**Covers:** FR42, FR43

As a người dịch,
I want thấy ảnh nằm đúng chỗ của nó trong bài, cả khi dịch lẫn khi đọc lại,
So that tôi hiểu được ngữ cảnh của đoạn văn quanh nó.

**Acceptance Criteria:**

**Given** một Chương có ảnh nhúng
**When** mở Panel Source
**Then** ảnh hiển thị **đúng vị trí** của chúng trong văn bản gốc

**Given** một Chương có ảnh nhúng
**When** đọc ở Chế độ đọc
**Then** ảnh hiển thị **đúng vị trí** của chúng trong văn bản

**Given** vị trí ảnh
**When** xác định
**Then** lấy từ **neo của `ASSET`**, không suy ra từ `ord` của segment alt-text

**Given** một ảnh có caption
**When** hiển thị ở Chế độ đọc
**Then** chú thích dưới ảnh là **`caption` đã dịch**

**Given** alt-text đã dịch
**When** hiển thị ở Chế độ đọc
**Then** **không hiện trên trang** — nó là thứ trình đọc màn hình đọc lên, không phải thứ mắt nhìn

**Given** một ảnh **không có caption**
**When** hiển thị
**Then** **không chừa chỗ trống** dưới ảnh

**Given** ảnh
**When** nạp
**Then** đọc từ `assets/` qua asset protocol, không qua IPC

---

### Story 6.15: Xuất xứ tài liệu ở tầng Chương

**Covers:** FR128

As a người dịch bài báo,
I want tác giả và link bài gốc được ghi lại tự động ngay lúc nhập,
So that nghĩa vụ ghi nguồn không phụ thuộc vào trí nhớ của tôi vài tuần sau.

**Acceptance Criteria:**

**Given** xuất xứ tài liệu
**When** mô hình hoá
**Then** **bốn trường trên `CHAPTER`**: tên tác giả bài gốc · tên báo hoặc website nguồn · URL bài gốc · ngày đăng bài gốc

**Given** tầng lưu trữ
**When** chọn
**Then** ở **tầng Chương**, không phải tầng Tác phẩm — truyện web mỗi Chương một link riêng

**Given** một Chương nhập từ URL
**When** nhập
**Then** bốn trường **tự điền từ trang**

**Given** một trường không tìm thấy được
**When** hiển thị
**Then** hiện chữ nghiêng *"không tìm thấy"*
**And** **không để trống** — để người dùng biết hệ thống đã tìm chứ không phải quên

**Given** bốn trường trong màn xem trước
**When** hiển thị
**Then** gom thành **một khối ở đầu mỗi Chương**, sửa tại chỗ được

**Given** một Tác phẩm nhập từ file hoặc dán tay
**When** người dùng muốn ghi xuất xứ
**Then** cùng khối đó mở được từ danh sách Chương và **nhập tay được**

**Given** bốn trường này
**When** lưu
**Then** là **nguồn sự thật duy nhất** của xuất xứ
**And** **không lưu chuỗi ghi nguồn đã định dạng** ở bất kỳ đâu

---

### Story 6.16: Nhập tài liệu song ngữ hai cột

**Covers:** FR115

As a người biên tập,
I want đưa một bản dịch do người khác làm vào công cụ mà không mất bản dịch đó,
So that tôi biên tập lại nó trong môi trường của mình thay vì trong một file Word.

**Acceptance Criteria:**

**Given** một file hai cột — bảng `.docx`, bảng `.md`, `.csv` hoặc `.tsv`
**When** nhập
**Then** người dùng **khai báo cột nào là nguồn, cột nào là đích**, và ngôn ngữ nguồn

**Given** file hai cột chứa cả một bộ truyện
**When** nhập
**Then** nó đến dưới dạng **một dòng văn bản chưa chia Chương**, nên **có** bước tách Chương

**Given** mẫu phân tách
**When** áp
**Then** áp lên **cột nguồn**, không áp lên cột đích

**Given** màn xem trước
**When** hiển thị
**Then** hiện **số Chương nhận ra được** trước khi xác nhận
**And** **bắt buộc xác nhận trước khi ghi xuống đĩa**

**Given** nhập hoàn tất
**When** kiểm kết quả
**Then** ra một Tác phẩm đầy đủ: có segment nguồn, có segment đích, đã khớp cặp

**Given** mọi Chương nhập theo đường này
**When** tạo
**Then** vào trạng thái **Đang dịch**

**Given** mọi segment nhập theo đường này
**When** tạo
**Then** ở trạng thái **chưa xác nhận** — kể cả khi bản dịch trông đã hoàn chỉnh

**Given** mỗi segment đích nhập theo đường này
**When** tạo
**Then** mang xuất xứ **nhập từ tài liệu song ngữ**

**Given** cờ kết đoạn
**When** tính cho đường này
**Then** lấy từ **ranh giới hàng** của bảng nguồn, không đoán lại từ nội dung

---

### Story 6.17: Khớp câu trong từng cặp hàng

**Covers:** FR116

As a người biên tập,
I want nối tay những chỗ số câu hai bên lệch nhau,
So that một lần khớp im lặng không đẩy bản dịch lệch đi một câu suốt cả chương.

**Acceptance Criteria:**

**Given** một hàng trong bảng hai cột
**When** xử lý
**Then** hệ thống tách **cả hai phía** thành câu và khớp **bên trong từng cặp hàng**

**Given** không gian khớp
**When** giới hạn
**Then** chỉ nằm trong **một hàng**, không phải cả Chương

**Given** một cặp hàng có số câu hai bên **bằng nhau**
**When** khớp
**Then** khớp tự động theo thứ tự

**Given** một cặp hàng có số câu hai bên **lệch nhau**
**When** xảy ra
**Then** **hiện ra cho người dùng nối tay**
**And** không khớp im lặng

**Given** người dùng nối tay
**When** thực hiện
**Then** nối và tách được các câu ở phía đích để khớp với phía nguồn

**Given** thao tác nối câu
**When** xảy ra
**Then** nằm **trong bước xem trước**, trước khi bất kỳ segment nào được ghi xuống đĩa
**And** **không** áp quy tắc về hưu + tạo mới — chưa có segment nào tồn tại để cho về hưu

**Given** toàn bộ thao tác nối tay
**When** thực hiện
**Then** làm được bằng bàn phím

---

### Story 6.18: Đo lại NFR3, NFR4, NFR5 trên thư viện 5.000 Chương thật

**Covers:** NFR3 · NFR4 · NFR5 *(nghiệm thu — đóng A6, A7, Q4)*

As a chủ dự án,
I want ba ngưỡng tạm được đo lại trên một thư viện 5.000 Chương dựng bằng chính đường nhập của sản phẩm,
So that Q4 đóng được bằng số đo chứ không bằng phán đoán.

**Acceptance Criteria:**

**Given** đường nhập hàng loạt (FR14) đã tồn tại ở epic này
**When** dựng bộ dữ liệu đo
**Then** tạo được một thư viện **5.000 Chương** bằng **chính đường nhập của sản phẩm**, không bằng một script nhét thẳng vào database
**And** đây là điều kiện Story 5.14 không có được: ở Epic 5 chưa có đường nào tạo ra ngần ấy Chương

**Given** thư viện 5.000 Chương
**When** đo
**Then** ghi lại **p95 tìm kiếm full-text** (NFR3), **thời gian khởi động tới lúc Library dùng được** (NFR4), và **bộ nhớ khi nhàn rỗi** (NFR5)
**And** đo trên **cả macOS lẫn Windows**

**Given** số đo và ba ngưỡng tạm A6, A7, A8
**When** đối chiếu
**Then** mỗi ngưỡng nhận **đúng một** trong ba kết luận: **giữ nguyên** · **sửa thành số đo được** · **vượt quá xa nên là thay đổi tầng PRD cần chủ dự án quyết**
**And** kết luận được ghi vào PRD, không chỉ ghi trong một lần chạy

**Given** Story 5.14 đã ghi số sơ bộ trên thư viện nhỏ
**When** story này hoàn tất
**Then** kết quả ở đây **thay thế** số đó
**And** Story 5.14 được đánh dấu rõ là phép đo **sơ bộ**, không phải phép đo nghiệm thu

**Given** ba kết luận đã ghi
**When** hoàn tất
**Then** **Q4 đóng**
**And** không còn ngưỡng nào của NFR3, NFR4, NFR5 mang nhãn ngưỡng tạm

---

## Epic 7: Translation Memory — không dịch lại, không tra lại thứ đã dịch

Mỗi lần người dịch xác nhận một segment, cặp *(nguồn → đích)* **tự vào Translation Memory, không một thao tác thủ công nào**. Từ đó về sau: câu y hệt được **điền sẵn nhưng vẫn ở trạng thái chưa xác nhận**; câu tương tự hiện kèm phần trăm khớp và diff phần khác biệt; và Concordance trả lời *"cụm này trước đây tôi dịch thế nào?"* ngay trong Panel Lookup. TM xuất được TMX mở ở CAT tool khác. Và vì chủ dự án làm **cả hai vai**, mỗi cặp TM mang **xuất xứ**, và Smart RAG Injector **ưu tiên cặp của chính người dùng**.

### Story 7.1: Ghi TM tự động, khoá theo cặp văn bản

**Covers:** FR44, FR56, FR129

As a người dịch,
I want mỗi câu tôi xác nhận tự vào kho mà không phải bấm thêm gì,
So that kho của tôi dày lên như một hệ quả của việc làm, không phải như một việc phải nhớ.

**Acceptance Criteria:**

**Given** người dùng xác nhận một segment
**When** xảy ra
**Then** cặp *(văn bản nguồn → văn bản đích)* được ghi vào Translation Memory

**Given** việc ghi TM
**When** thực hiện
**Then** **không có thao tác thủ công nào** của người dùng

**Given** một mục TM
**When** mô hình hoá
**Then** là `(văn bản nguồn, văn bản đích) + metadata`
**And** **độc lập hoàn toàn với `segment.id`**

**Given** một segment bị gộp, tách, hoặc tái tách
**When** xảy ra
**Then** các cặp TM đã ghi từ nó **ở lại nguyên**

**Given** người dùng sửa bản dịch của một segment **đã xác nhận** rồi xác nhận lại
**When** xảy ra
**Then** **ghi thêm một cặp mới**
**And** không sửa cặp cũ

**Given** thời điểm ghi cặp TM
**When** xác định
**Then** **đúng tại chuyển tiếp sang đã xác nhận**, không ở chỗ nào khác

**Given** alt-text và caption của ảnh
**When** xác nhận
**Then** cũng ghi cặp TM như mọi segment khác
**And** đây là **điểm nghiệm thu TM cho FR44 và FR129** — Story 6.13 chỉ nghiệm thu phần cấu trúc, phần TM đóng ở đây

---

### Story 7.2: Xuất xứ trên từng cặp TM

**Covers:** FR118

As a người dịch làm cả vai biên tập,
I want mỗi cặp trong kho biết nó là chữ của ai,
So that kho của tôi không âm thầm đầy lên bằng văn phong người khác.

**Acceptance Criteria:**

**Given** một cặp TM
**When** ghi
**Then** mang **xuất xứ kế thừa từ segment** sinh ra nó

**Given** ba giá trị xuất xứ
**When** mô hình hoá
**Then** *tôi dịch* · *người khác dịch* · *nhập từ tài liệu song ngữ*

**Given** người dùng biên tập một Tác phẩm do người khác dịch
**When** xác nhận từng câu
**Then** câu họ **viết lại** mang xuất xứ *tôi dịch*
**And** câu họ **duyệt nguyên văn** mang xuất xứ *người khác dịch*

**Given** việc phân biệt hai loại trên
**When** thực hiện
**Then** không tốn thêm một thao tác nào của người dùng

**Given** hệ thống gặp một Tác phẩm biên tập 200 chương
**When** ghi TM
**Then** **không bỏ không ghi** — làm vậy sẽ mất luôn những câu chính người dùng viết lại từ đầu

---

### Story 7.3: TM phạm vi kép và thứ tự sắp xếp hai khoá

**Covers:** FR57

As a người dịch,
I want kết quả TM ưu tiên đúng thứ giống văn phong tôi nhất,
So that gợi ý đầu tiên tôi thấy là gợi ý đáng dùng nhất.

**Acceptance Criteria:**

**Given** Translation Memory
**When** mô hình hoá phạm vi
**Then** có **TM riêng theo Tác phẩm** và **TM chung toàn cục**

**Given** một truy vấn TM
**When** phân giải hai tầng
**Then** ngữ nghĩa là **hợp nhất** — trả kết quả cả hai tầng
**And** đi qua `ScopeResolver`

**Given** kết quả TM từ hai tầng
**When** sắp xếp
**Then** khoá **chính** là **xuất xứ** — cặp *của tôi* trước

**Given** kết quả TM cùng xuất xứ
**When** sắp xếp
**Then** khoá **phụ** là **tầng** — Tác phẩm trước Global

**Given** một cặp toàn cục do chính người dùng dịch và một cặp Tác phẩm do người khác dịch
**When** sắp xếp
**Then** cặp **của chính người dùng** đứng trước — vì mục đích của Smart RAG là học văn phong

**Given** người dùng xác nhận một segment
**When** ghi TM
**Then** **chỉ** ghi vào TM **Tác phẩm**

**Given** TM toàn cục
**When** nhận dữ liệu
**Then** chỉ qua thao tác chủ động — nhập TMX, hoặc đẩy một cặp lên tầng toàn cục

---

### Story 7.4: Khớp tuyệt đối 100%

**Covers:** FR58

As a người dịch,
I want câu y hệt tôi từng dịch được điền sẵn nhưng vẫn chờ tôi gật đầu,
So that công cụ không bao giờ tự coi một câu là xong thay tôi.

**Acceptance Criteria:**

**Given** một segment có văn bản nguồn y hệt một cặp đã có trong TM
**When** mở
**Then** bản dịch cũ được **điền sẵn** vào Editor

**Given** một segment được điền sẵn từ khớp 100%
**When** hiển thị
**Then** vạch lề là `tm-rule` — **gợi ý cần xác nhận**

**Given** một segment được điền sẵn
**When** kiểm trạng thái
**Then** **chưa xác nhận**
**And** hệ thống **không** tự coi nó là đã hoàn thành

**Given** một segment được điền sẵn
**When** ghi xuống
**Then** ghi **ngay**, không đi qua bộ đệm gõ — đây là một hành động dứt khoát, không phải văn bản đang soạn
**And** **không** tạo `SegmentVersion`

**Given** người dùng xác nhận một segment được điền sẵn mà không sửa gì
**When** ghi xuất xứ
**Then** giữ nguyên xuất xứ của cặp TM nguồn

**Given** nhiều cặp TM cùng khớp 100%
**When** chọn cặp để điền sẵn
**Then** theo đúng thứ tự hai khoá của Story 7.3

---

### Story 7.5: Khớp mờ

**Covers:** FR59

As a người dịch,
I want thấy câu gần giống mình từng dịch và biết nó khác chỗ nào,
So that tôi sửa nhanh thay vì dịch lại từ đầu.

**Acceptance Criteria:**

**Given** một segment có cặp TM tương tự nhưng không y hệt
**When** mở
**Then** một **dải mọc** dưới câu đang sửa hiện các bản dịch cũ tương tự

**Given** mỗi gợi ý khớp mờ
**When** hiển thị
**Then** kèm **phần trăm khớp**

**Given** mỗi gợi ý khớp mờ
**When** hiển thị
**Then** kèm **diff phần khác biệt** giữa câu nguồn cũ và câu nguồn hiện tại

**Given** một câu kích hoạt đồng thời chốt Glossary, phát hiện Proofreader và gợi ý TM
**When** hiển thị
**Then** gợi ý TM **nhường cả hai** — bỏ qua chỉ tốn công gõ lại một câu, và câu đó vẫn nằm nguyên trong TM cho lần sau

**Given** dải gợi ý TM
**When** hiển thị
**Then** **đẩy văn bản xuống** chứ không phủ lên, và thu lại ngay khi xong

**Given** người dùng chọn một gợi ý
**When** xảy ra
**Then** văn bản chuyển vào Editor và segment ở trạng thái **chưa xác nhận**

**Given** toàn bộ thao tác trên dải
**When** thực hiện
**Then** làm được bằng bàn phím

---

### Story 7.6: Thuật toán khớp theo ngôn ngữ

**Covers:** FR61

As a người dựng,
I want TM khớp bằng đúng cơ chế mà từ điển và Glossary đang dùng,
So that ba nơi không bao giờ bắt được những biến thể khác nhau.

**Acceptance Criteria:**

**Given** khớp TM
**When** cài đặt
**Then** dùng **đúng component `Matcher` dùng chung** của Epic 1

**Given** văn bản tiếng Trung
**When** khớp mờ
**Then** dùng **n-gram ký tự** — không có ranh giới từ

**Given** văn bản tiếng Anh
**When** khớp mờ
**Then** dùng **token n-gram sau stemming**

**Given** `core/matching/`
**When** rà
**Then** vẫn chỉ có **một** cài đặt, phục vụ `dict/`, `glossary/` và `tm/`

**Given** một biến thể mà Glossary bắt được
**When** thử với TM
**Then** TM cũng bắt được — và ngược lại

---

### Story 7.7: Concordance

**Covers:** FR60

As a người dịch,
I want hỏi *"cụm này trước đây tôi dịch thế nào?"* ngay tại chỗ tôi đang tra từ điển,
So that tôi không phải nhớ hai chỗ khác nhau cho hai loại tra cứu.

**Acceptance Criteria:**

**Given** một cụm từ đang chọn
**When** người dùng gọi lệnh Concordance
**Then** hệ thống tra ngược trên **toàn bộ** Translation Memory

**Given** kết quả Concordance
**When** hiển thị
**Then** đưa vào **Panel Lookup**, cùng chỗ với kết quả từ điển

**Given** mỗi kết quả Concordance
**When** hiển thị
**Then** thấy câu nguồn, bản dịch, và **xuất xứ** của cặp đó

**Given** kết quả Concordance và kết quả từ điển cùng ở Panel Lookup
**When** hiển thị
**Then** phân biệt được với nhau

**Given** Concordance không có kết quả
**When** hiển thị
**Then** trạng thái rỗng nêu rõ lý do, khác với trạng thái rỗng của từ điển

**Given** trạng thái rỗng *"không tìm thấy"* của tra cứu từ điển ở Story 1.17
**When** Concordance đã tồn tại
**Then** bổ sung đường trỏ sang Concordance vào trạng thái rỗng đó
**And** đây là chỗ lời hứa bị hoãn ở Story 1.17 được trả

**Given** Translation Memory còn trống
**When** người dùng mở Concordance lần đầu
**Then** trạng thái rỗng giải thích **cơ chế** — TM tự đầy khi xác nhận câu, không có nút *"thêm vào TM"*

**Given** lệnh Concordance
**When** gọi
**Then** là command đăng ký, gán phím được

---

### Story 7.8: Nhiều bản dịch cho cùng một câu nguồn

**Covers:** FR63

As a người dịch,
I want giữ lại cả hai cách tôi từng dịch một câu,
So that tôi tự chọn theo ngữ cảnh thay vì bị công cụ chọn hộ.

**Acceptance Criteria:**

**Given** một câu nguồn đã có bản dịch trong TM
**When** người dùng xác nhận một bản dịch **khác** cho cùng câu nguồn đó
**Then** hệ thống **giữ lại cả hai**

**Given** nhiều bản dịch cho cùng một câu nguồn
**When** hiển thị
**Then** hiển thị **tất cả**, mỗi bản kèm **ngày**

**Given** một bản dịch mới cho câu nguồn đã có
**When** ghi
**Then** **không ghi đè** bản cũ

**Given** nhiều bản dịch cùng khớp
**When** người dùng chọn
**Then** người dịch tự chọn — hệ thống không chọn hộ

**Given** nhiều bản dịch
**When** sắp xếp
**Then** theo đúng thứ tự hai khoá của Story 7.3, rồi tới ngày

---

### Story 7.9: Quản lý Translation Memory

**Covers:** FR62

As a người dịch,
I want rà lại kho của mình và dọn phần không phải văn phong của tôi,
So that tôi kiểm soát được thứ AI đang học từ tôi.

**Acceptance Criteria:**

**Given** màn hình quản lý TM
**When** mở
**Then** xem được từng mục TM

**Given** một mục TM
**When** người dùng sửa
**Then** thay đổi lưu và có hiệu lực ở lần khớp kế tiếp

**Given** một mục TM
**When** người dùng xoá
**Then** mục biến khỏi kho

**Given** danh sách TM
**When** hiển thị
**Then** hiện **xuất xứ** của từng cặp

**Given** danh sách TM
**When** lọc
**Then** **lọc được theo xuất xứ** — để rà lại hoặc dọn sạch phần không phải văn phong của mình

**Given** danh sách TM
**When** lọc
**Then** lọc được theo tầng — TM Tác phẩm hay TM toàn cục

**Given** một cặp ở tầng Tác phẩm
**When** người dùng muốn đẩy lên tầng toàn cục
**Then** làm được bằng một thao tác chủ động

**Given** mọi thao tác trên màn hình này
**When** thực hiện
**Then** làm được bằng bàn phím

---

### Story 7.10: Xuất và nhập TMX

**Covers:** FR64

As a người dịch,
I want mang kho dịch của mình sang một công cụ khác,
So that dữ liệu của tôi sống lâu hơn phần mềm và tôi không bị khoá vào `.atproj`.

**Acceptance Criteria:**

**Given** một Translation Memory ở tầng bất kỳ
**When** người dùng xuất
**Then** sinh ra file **TMX** hợp lệ

**Given** file TMX vừa xuất
**When** mở bằng một CAT tool khác
**Then** đọc được các cặp

**Given** file TMX vừa xuất
**When** nhập lại vào một TM rỗng
**Then** **round-trip đầy đủ** các cặp

**Given** người dùng chọn tầng đích khi nhập
**When** thực hiện
**Then** cặp vào đúng tầng đó

**Given** cặp nhập từ TMX
**When** ghi
**Then** mang xuất xứ phân biệt được với cặp sinh ra từ việc xác nhận segment

**Given** một file TMX sai định dạng
**When** nhập
**Then** báo lỗi nêu rõ vấn đề
**And** không ghi một phần

**Given** một cặp TMX trùng với cặp đã có
**When** nhập
**Then** áp đúng quy tắc của Story 7.8 — giữ cả hai, không ghi đè

---

### Story 7.11: Smart RAG ưu tiên cặp của chính người dùng

**Covers:** FR70, FR118

As a người dịch,
I want AI học văn phong của chính tôi chứ không phải của người tôi biên tập hộ,
So that càng dùng lâu bản đề xuất càng giống tôi viết, chứ không lệch dần đi.

**Acceptance Criteria:**

**Given** `RagInjector`
**When** lắp prompt
**Then** chèn các segment TM tương tự tìm được, **không đổi chữ ký hàm** đã định ở Story 4.6

**Given** các cặp TM ứng viên để chèn
**When** chọn
**Then** **ưu tiên cặp mang xuất xứ *tôi dịch***

**Given** không đủ cặp của chính người dùng
**When** xảy ra
**Then** cặp xuất xứ khác **mới** được chèn

**Given** một cặp xuất xứ khác được chèn
**When** lắp vào prompt
**Then** **đánh dấu rõ trong prompt là văn phong tham khảo**, không phải văn phong của người dùng

**Given** người dùng mở **Xem prompt**
**When** hiển thị
**Then** thấy cặp TM nào được chèn và cặp nào mang nhãn tham khảo

**Given** `RagInjector`
**When** kiểm tính thuần
**Then** cùng đầu vào vẫn cho cùng một prompt

**Given** một lời gọi AI hoàn tất
**When** hiển thị dòng tóm tắt
**Then** cho biết đã chèn bao nhiêu thuật ngữ Glossary **và bao nhiêu câu TM tương tự**

---

## Epic 8: Cầu nối Reviewer — xuất, nhập lại, đối chiếu, và hấp thụ bài học

Reviewer **không cài AuraTranslate**, nên trao đổi file là cầu nối duy nhất. Người dịch xuất `.docx` bảng hai cột đối xứng theo segment cho reviewer sửa, hoặc **`.docx` một khối đối xứng theo đoạn** để dán thẳng sang trình soạn thảo website. Nhận file đã sửa về: hệ thống khớp cấu trúc đoạn và **hiện ra cho người dùng nối tay** những chỗ không khớp, rồi Review Mode ẩn văn bản gốc và bôi màu thêm/xoá/sửa. Và **kể cả khi người dịch không bao giờ mở Review Mode**, hệ thống vẫn hấp thụ bài học của reviewer vào Glossary.

### Story 8.1: Mũi thăm dò thư viện diff

**Covers:** NFR15 · mũi thăm dò bắt buộc

As a chủ dự án,
I want chọn thư viện diff bằng dữ liệu thật thay vì bằng mô tả trên trang crate,
So that Review Mode bôi màu đúng chỗ mắt người cần nhìn.

**Acceptance Criteria:**

**Given** `similar` và `dissimilar`
**When** thử
**Then** chạy **cả hai** trên bản review thật của chủ dự án

**Given** hai kết quả diff
**When** so
**Then** đánh giá theo đánh đổi đã biết — diff cấp grapheme so với semantic cleanup

**Given** thư viện được chọn
**When** đưa vào Stack
**Then** đã rà tương thích GPL v3
**And** ghi vào bảng Stack của `ARCHITECTURE-SPINE.md`

**Given** văn bản tiếng Việt nhiều dấu
**When** diff
**Then** kiểm bằng chuỗi dày dấu — không đánh giá bằng chuỗi Latin

**Given** phần bôi màu diff
**When** kiểm tương phản
**Then** đạt WCAG AA ở **cả hai** theme

---

### Story 8.2: Phạm vi xuất

**Covers:** FR89

As a người dịch,
I want chọn xuất một chương, vài chương, hay cả bộ,
So that tôi gửi đúng phần reviewer cần mà không phải cắt file bằng tay.

**Acceptance Criteria:**

**Given** màn hình xuất
**When** mở
**Then** chọn được phạm vi: **một Chương · nhiều Chương đã chọn · cả Tác phẩm**

**Given** phạm vi đã chọn
**When** hiển thị
**Then** thấy số Chương và số segment sẽ được xuất

**Given** phạm vi xuất
**When** áp
**Then** áp cho **mọi định dạng xuất** như nhau

**Given** phạm vi xuất còn câu **chưa xác nhận**
**When** người dùng chuẩn bị xuất
**Then** hiện cảnh báo **trước lúc xuất**, nêu rõ bao nhiêu câu

**Given** đường dẫn xuất
**When** chọn
**Then** cấp qua hộp thoại chọn thư mục — scope động, không phải scope tĩnh

**Given** màn hình xuất có khối xem trước hình dạng file
**When** hiển thị
**Then** khối đó **mô phỏng một tài liệu Word**, dùng màu viết cứng có chủ ý chứ không dùng token giấy ngà của ứng dụng
**And** các màu đó vẫn phải đạt WCAG AA
**And** đây là **ngoại lệ đã đặc tả** — áp token vào đó sẽ nói dối về thứ reviewer thật sự nhìn thấy

**Given** mọi thao tác trên màn hình xuất
**When** thực hiện
**Then** làm được bằng bàn phím

---

### Story 8.3: Xuất `.docx` bảng hai cột theo segment

**Covers:** FR87

As a người dịch,
I want gửi reviewer một file mà mỗi câu nằm một hàng,
So that họ sửa ở đâu tôi cũng nhập ngược về đúng chỗ đó được.

**Acceptance Criteria:**

**Given** phạm vi xuất đã chọn
**When** xuất định dạng này
**Then** sinh ra `.docx` dạng **bảng hai cột** — cột trái văn bản gốc, cột phải bản dịch

**Given** bảng xuất ra
**When** kiểm cấu trúc
**Then** **đối xứng theo segment** — mỗi segment một hàng

**Given** file xuất ra
**When** mở bằng Microsoft Word hoặc LibreOffice
**Then** hiển thị đúng, bảng không vỡ

**Given** một segment chưa có bản dịch
**When** xuất
**Then** ô bên phải trống, hàng vẫn tồn tại để giữ đối xứng

**Given** định dạng này
**When** người dùng chọn
**Then** màn hình xuất nêu rõ **đây là định dạng nhập lại được**

**Given** file này
**When** nhập ngược vào app
**Then** đi qua đường FR90/FR91 bình thường

---

### Story 8.4: Xuất `.docx` một khối theo đoạn cho đăng bài

**Covers:** FR121

As a người dịch bài đăng,
I want bôi đen cột phải và dán thẳng sang trình soạn thảo website,
So that người đăng nhận được văn bản liền mạch chứ không phải mảnh vụn bảng biểu.

**Acceptance Criteria:**

**Given** phạm vi xuất đã chọn
**When** xuất định dạng này
**Then** sinh ra `.docx` bảng hai cột với **một hàng duy nhất cho cả Chương**

**Given** bảng xuất ra
**When** kiểm hình thức
**Then** **không có đường kẻ ngang**

**Given** hai ô của hàng
**When** kiểm cấu trúc đoạn
**Then** giữ **đúng số lần xuống đoạn như nhau**

**Given** số lần xuống đoạn
**When** xác định
**Then** đọc từ **cờ kết đoạn đã lưu** trên segment
**And** **không có đường mã nào suy ra đoạn từ nội dung lúc xuất**

**Given** người dùng bôi đen cột phải rồi dán sang trình soạn thảo của website
**When** thực hiện
**Then** ra **văn bản liền mạch**, không kèm mảnh vụn bảng biểu

**Given** người dùng chọn định dạng này
**When** hiển thị
**Then** màn hình xuất nói rõ **ngay lúc chọn** rằng định dạng này **không nhập lại được**
**And** thông tin này không nằm trong tài liệu hướng dẫn

**Given** phạm vi xuất còn câu chưa xác nhận
**When** xuất định dạng này
**Then** **cảnh báo trước lúc xuất**
**And** **không đánh dấu câu chưa xác nhận trong file** — một nền màu hay dòng ghi chú xen giữa văn xuôi sẽ đi thẳng vào bài đăng

---

### Story 8.5: Chọn cách xuất hình ảnh

**Covers:** FR130

As a người dịch bài đăng,
I want chọn xuất ảnh theo link gốc để người đăng dựng lại bài trên website,
So that họ không phải tự đi tìm lại từng tấm ảnh.

**Acceptance Criteria:**

**Given** màn hình xuất
**When** mở
**Then** có lựa chọn **theo link gốc** hoặc **theo file ảnh**

**Given** lựa chọn này
**When** áp
**Then** áp cho **từng lần xuất**, cùng khuôn với phạm vi xuất

**Given** người dùng chọn **theo link gốc**
**When** xuất
**Then** tài liệu xuất ra trỏ tới **URL ảnh của bài gốc**

**Given** người dùng chọn **theo file ảnh**
**When** xuất
**Then** ảnh đi kèm tài liệu xuất, lấy từ `.atproj/assets/`

**Given** phạm vi xuất chứa ảnh **không có `source_url`**
**When** người dùng chọn *theo link gốc*
**Then** đường xuất **quét phạm vi trước** và **liệt kê rõ ảnh nào sẽ không có link**

**Given** ảnh thiếu URL gốc
**When** xuất
**Then** **không đường mã nào được bỏ qua trong im lặng**

**Given** thông tin về ảnh thiếu link
**When** hiển thị
**Then** xuất hiện **ngay lúc chọn**, không nằm trong tài liệu hướng dẫn

**Given** phạm vi xuất **không có ảnh nào mang `source_url`**
**When** hiển thị
**Then** lựa chọn *theo link gốc* nêu rõ nó không dùng được cho phạm vi này

---

### Story 8.6: Xuất `.md` và text thuần

**Covers:** FR88

As a người dịch,
I want một bản Markdown giữ nguyên ảnh, chú thích và mô tả đã dịch,
So that người đăng dựng lại được trọn bài chứ không chỉ nhận chữ.

**Acceptance Criteria:**

**Given** phạm vi xuất đã chọn
**When** xuất `.md`
**Then** sinh ra Markdown hợp lệ

**Given** phạm vi xuất đã chọn
**When** xuất text thuần
**Then** sinh ra văn bản không đánh dấu

**Given** một Chương có ảnh
**When** xuất `.md`
**Then** ảnh được tham chiếu theo kiểu người dùng chọn ở Story 8.5

**Given** một ảnh có alt-text đã dịch
**When** xuất `.md`
**Then** **alt-text đã dịch** được đưa vào, không phải alt-text gốc

**Given** một ảnh có caption đã dịch
**When** xuất
**Then** **caption đã dịch** được đưa vào, tách bạch với alt-text

**Given** cấu trúc đoạn
**When** xuất
**Then** đọc từ cờ kết đoạn đã lưu, không suy ra từ nội dung

**Given** định dạng text thuần
**When** người dùng chọn
**Then** màn hình xuất nói rõ nó **không nhập lại được**, cùng nhóm với `.docx` một khối

---

### Story 8.7: Khối ghi nguồn

**Covers:** FR131

As a người dịch bài đăng,
I want khối ghi nguồn nằm sẵn trong file bàn giao,
So that người đăng — không cài app, không hỏi thêm câu nào — vẫn ghi nguồn đúng.

**Acceptance Criteria:**

**Given** khối ghi nguồn
**When** dựng
**Then** gồm **năm trường**: tác giả · tên báo/website · URL bài gốc · ngày đăng gốc · **tên người dịch**

**Given** bốn trường đầu
**When** lấy
**Then** lấy từ dữ liệu xuất xứ trên `CHAPTER`

**Given** trường tên người dịch
**When** lấy
**Then** lấy từ cấu hình **toàn cục**, đặt một lần, không phải gõ lại mỗi bài
**And** phân giải qua `ScopeResolver` với ngữ nghĩa ghi đè

**Given** khối ghi nguồn
**When** dựng
**Then** **dựng lúc xuất** từ các trường nguồn
**And** **không lưu chuỗi đã định dạng ở bất kỳ đâu**

**Given** người dùng sửa một trường xuất xứ ở tầng Chương
**When** xuất lại
**Then** khối ghi nguồn phản ánh giá trị mới ngay

**Given** khối ghi nguồn
**When** cấu hình
**Then** **bật/tắt được**
**And** áp cho **mọi định dạng xuất** — bảng theo segment, `.md`/text thuần, và một khối theo đoạn

**Given** khối ghi nguồn
**When** ở trạng thái mặc định
**Then** **tắt** — truyện xuất ra vẫn sạch trừ khi người dùng chủ động bật

---

### Story 8.8: Cổng kiểm hình dạng bảng `.docx`

**Covers:** AD-38 — cổng kiểm hình dạng bảng `.docx`

As a người dịch,
I want ứng dụng từ chối một file mà việc nhập nó sẽ phá dữ liệu của tôi,
So that một lần kéo nhầm file không ghi đè cả Chương đã xác nhận.

**Acceptance Criteria:**

**Given** một file `.docx` được đưa vào đường nhập
**When** xử lý
**Then** kiểm hình dạng bảng chạy **ở Rust**, ở **cổng vào** của đường nhập trong `core/export/`

**Given** kiểm hình dạng
**When** chạy
**Then** chạy **trước alignment** và **trước mọi lệnh ghi**

**Given** một bảng có **đúng một hàng** *và* ô chứa **nhiều hơn một đoạn**
**When** kiểm
**Then** **từ chối** kèm câu giải thích rằng đây là bản dành cho đăng bài và không nhập lại được
**And** **không chạy alignment, không ghi gì**

**Given** mọi hình dạng bảng khác
**When** kiểm
**Then** đi tiếp qua alignment

**Given** việc nhận dạng
**When** thực hiện
**Then** dựa trên **hình dạng**
**And** **không** dựa trên metadata hay tên file — file do người khác tạo không mang dấu của ta, và tên file thì ai cũng đổi được

**Given** một Chương ngắn chỉ có **một đoạn** nhưng nhiều câu
**When** kiểm
**Then** hai hình dạng không phân biệt được — đây là **ca sót lại đã chấp nhận có ý thức**, hậu quả nhẹ vì đúng là một đoạn thật

**Given** một bản `.docx` một khối vừa xuất ở Story 8.4
**When** thử nhập lại
**Then** bị từ chối bởi cổng này — nghiệm thu bằng đúng phép thử này

---

### Story 8.9: Nhập lại file reviewer đã sửa

**Covers:** FR90

As a người dịch,
I want đưa bản reviewer đã sửa trở lại đúng Tác phẩm của mình,
So that công sức họ bỏ ra không dừng lại ở một file trong hộp thư.

**Acceptance Criteria:**

**Given** một file `.docx` hoặc `.md` reviewer đã chỉnh sửa
**When** nhập
**Then** nhập vào **đúng Tác phẩm hiện có**, không tạo Tác phẩm mới

**Given** một file nhập vào
**When** xử lý
**Then** đi qua cổng kiểm hình dạng của Story 8.8 trước

**Given** nội dung file
**When** phân tích
**Then** Rust phân tích thành **mô hình dữ liệu có cấu trúc**, Vue render từ mô hình đó
**And** không tồn tại đường nào đưa chuỗi từ file vào `v-html` hoặc tương đương

**Given** người dùng chọn Tác phẩm đích
**When** nhập
**Then** hiện rõ Chương nào sẽ bị ảnh hưởng trước khi ghi

**Given** một file không khớp với Tác phẩm đã chọn
**When** nhập
**Then** báo rõ và không ghi

**Given** quá trình nhập
**When** chưa được người dùng xác nhận
**Then** **chưa có gì ghi xuống đĩa**

---

### Story 8.10: Segment alignment — máy khớp, người sửa

**Covers:** FR91

As a người dịch,
I want thấy rõ những đoạn hệ thống không khớp được và tự nối chúng,
So that một lần khớp im lặng không đẩy cả chương lệch đi một câu.

**Acceptance Criteria:**

**Given** file reviewer đã nhập
**When** alignment chạy
**Then** hệ thống khớp cấu trúc đoạn giữa file nhập và dữ liệu sẵn có

**Given** các segment khớp được
**When** xử lý
**Then** khớp tự động

**Given** các segment **không khớp được**
**When** xảy ra
**Then** **hiện ra thành danh sách cho người dùng nối tay**

**Given** một segment không khớp
**When** hiển thị
**Then** thấy cả phía bản của mình và phía bản reviewer để đối chiếu

**Given** người dùng nối tay
**When** thực hiện
**Then** nối và tách được để khớp hai phía

**Given** alignment
**When** chạy
**Then** **không có khớp im lặng** — mọi chỗ không chắc chắn đều hiện ra

**Given** người dùng chưa xử lý xong các chỗ không khớp
**When** ở trạng thái đó
**Then** chưa ghi thay đổi nào vào segment

**Given** toàn bộ thao tác nối tay
**When** thực hiện
**Then** làm được bằng bàn phím

---

### Story 8.11: Review Mode — bố cục hai cửa sổ side-by-side

**Covers:** FR92

As a người dịch,
I want đặt bản của mình cạnh bản reviewer đã sửa,
So that tôi lướt qua từng chỗ khác biệt thay vì đọc lại cả chương.

**Acceptance Criteria:**

**Given** một Chương đã nhập bản review
**When** người dùng mở Review Mode
**Then** Workspace chuyển sang bố cục **hai cửa sổ side-by-side**

**Given** bố cục Review Mode
**When** hiển thị
**Then** trái là bản dịch của người dùng, phải là bản đã nhập từ Reviewer

**Given** Review Mode
**When** cài đặt
**Then** là **một bố cục dockview**, không phải cửa sổ hệ điều hành thứ hai

**Given** Review Mode
**When** so với ba chế độ
**Then** nó **không phải chế độ thứ tư** — vẫn nằm trong Workspace

**Given** Review Mode
**When** kích hoạt
**Then** khai điểm vào focus tường minh, focus không rơi về `body`

**Given** người dùng rời Review Mode
**When** xảy ra
**Then** quay lại bố cục Workspace trước đó, giữ nguyên ngữ cảnh

**Given** lệnh mở và đóng Review Mode
**When** gọi
**Then** là command đăng ký, gán phím được

---

### Story 8.12: Diff bôi màu, ẩn văn bản gốc

**Covers:** FR93

As a người dịch,
I want chỉ nhìn hai bản dịch và chỗ chúng khác nhau,
So that mắt tôi không phải lọc qua nguyên văn tiếng Trung khi đang so hai bản tiếng Việt.

**Acceptance Criteria:**

**Given** Review Mode đang mở
**When** hiển thị
**Then** **ẩn văn bản gốc**

**Given** hai bản dịch
**When** so
**Then** thuật toán diff bôi màu phần **thêm / xoá / sửa**

**Given** phần bôi màu diff
**When** kiểm tương phản
**Then** đạt **WCAG AA ở cả hai theme**

**Given** màu dùng cho diff
**When** chọn
**Then** đến từ bộ token đã kiểm tương phản, không viết thẳng trong component

**Given** một Chương dài
**When** người dùng lướt
**Then** cuộn đồng bộ giữa hai cửa sổ để hai phía luôn khớp chỗ

**Given** một chỗ không có khác biệt nào
**When** hiển thị
**Then** không bôi màu — chỉ chỗ khác mới nổi lên

**Given** lệnh nhảy tới khác biệt kế tiếp và trước đó
**When** gọi
**Then** là command đăng ký, gán phím được

---

### Story 8.13: Chấp nhận từng thay đổi

**Covers:** FR94

As a người dịch,
I want chọn lấy những sửa đổi tôi đồng ý và bỏ những chỗ tôi không,
So that reviewer là người góp ý chứ không phải người quyết định.

**Acceptance Criteria:**

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

---

### Story 8.14: Thu hoạch thuật ngữ từ bản review

**Covers:** FR54

As a người dịch,
I want công cụ nhận ra reviewer đã đổi cách gọi một nhân vật,
So that từ chương sau AI dùng đúng cách gọi đó thay vì lặp lại lỗi cũ.

**Acceptance Criteria:**

**Given** một bản review vừa nhập
**When** phân tích
**Then** hệ thống tìm các trường hợp reviewer đổi thuật ngữ *X* thành *Y* một cách **nhất quán**

**Given** một cặp đổi được phát hiện
**When** đề xuất
**Then** nêu rõ **số lần đổi trên tổng số lần xuất hiện** — để người dùng tự phán xét mức nhất quán

**Given** một đề xuất thu hoạch
**When** ghi
**Then** ghi vào **bảng chờ** của Epic 3
**And** **không** ghi thẳng vào Glossary

**Given** một đề xuất được người dùng duyệt
**When** vào Glossary
**Then** mang xuất xứ **thu hoạch từ bản review**
**And** vào thẳng trạng thái **đã chốt** — cặp nguồn–đích đã có sẵn

**Given** một đề xuất bị bỏ
**When** nhập một bản review khác về sau
**Then** không đề xuất lại cùng cặp đó trong cùng Tác phẩm

**Given** một cặp vừa vào Glossary từ đường này
**When** gọi AI ở Chương sau
**Then** Smart RAG Injector chèn nó vào prompt

---

### Story 8.15: Thu hoạch chạy độc lập với Review Mode

**Covers:** FR95

As a người dịch,
I want công sức của reviewer chuyển thành giá trị kể cả khi tôi không mở Diff Viewer,
So that vòng lặp học hỏi đóng lại ở tầng hệ thống thay vì trông chờ vào kỷ luật của tôi.

**Acceptance Criteria:**

**Given** người dùng nhập một bản review
**When** nhập hoàn tất
**Then** cơ chế thu hoạch thuật ngữ **kích hoạt ngay**

**Given** người dùng **không bao giờ mở Review Mode**
**When** nhập một bản review
**Then** thu hoạch vẫn chạy đầy đủ và đề xuất vẫn xuất hiện

**Given** chuỗi phụ thuộc trong mã
**When** rà
**Then** thu hoạch **không phụ thuộc** vào việc Review Mode được mở hay bố cục Review Mode có được dựng hay không

**Given** đề xuất thu hoạch
**When** xuất hiện
**Then** hiện ở một chỗ người dùng chắc chắn nhìn thấy dù không vào Review Mode

**Given** Review Mode bị gỡ khỏi bản dựng
**When** chạy bộ test
**Then** các test của Story 8.14 và story này **vẫn xanh**

**Given** kịch bản cắt phạm vi ở R1
**When** cắt Diff Viewer
**Then** cơ chế thu hoạch **không được cắt theo**

---

## Epic 9: AI Proofreader — bắt lỗi trước khi bàn giao

Người dịch chạy proofreader **theo yêu cầu** trên một segment, một Chương hoặc vùng đang chọn — **không bao giờ chạy nền**. Mỗi phát hiện gồm loại lỗi, vị trí, giải thích ngắn và đề xuất sửa, hiện **gạch chân lượn sóng ngay dưới đúng cụm chữ có vấn đề**. Chấp nhận hoặc bỏ qua từng cái; đánh dấu *"không phải lỗi"* thì lần quét sau **không báo lại trong cùng Tác phẩm**. Và proofreader **không bao giờ tự sửa văn bản**.

### Story 9.1: Quét chính tả và ngữ pháp tiếng Việt

**Covers:** FR80

As a người dịch,
I want bắt những lỗi chính tả và ngữ pháp tôi không còn nhìn ra sau bốn tiếng dịch,
So that bản bàn giao không mang những lỗi mà chính tôi thấy ngay nếu đọc lại hôm sau.

**Acceptance Criteria:**

**Given** bản dịch tiếng Việt của người dùng
**When** quét
**Then** phát hiện lỗi chính tả và lỗi ngữ pháp

**Given** một phát hiện thuộc loại chính tả hoặc ngữ pháp
**When** hiển thị
**Then** dùng màu `error` — loại lỗi này **có đáp án đúng**

**Given** lời gọi quét
**When** thực hiện
**Then** đi qua prompt lắp bởi `RagInjector`, không nối chuỗi tại chỗ gọi

**Given** module thực hiện quét
**When** đặt trong cây nguồn
**Then** nằm trong `core/ai/`
**And** không module nào ngoài `ai/` import nó

**Given** ứng dụng chưa cấu hình AI
**When** người dùng gọi proofreader
**Then** mời cấu hình
**And** mọi năng lực khác vẫn chạy đầy đủ

---

### Story 9.2: Đối chiếu bản dịch với bản gốc

**Covers:** FR81

As a người dịch,
I want biết chỗ nào tôi dịch lệch nghĩa hoặc viết ra một câu tối nghĩa,
So that reviewer không phải là người đầu tiên phát hiện ra chúng.

**Acceptance Criteria:**

**Given** một segment có cả văn bản nguồn và bản dịch
**When** quét
**Then** đối chiếu hai phía và đánh dấu đoạn nghi **dịch sai**, **dịch thoát nghĩa quá xa**, hoặc **cấu trúc câu tối nghĩa**

**Given** một phát hiện thuộc loại nghi về nghĩa
**When** hiển thị
**Then** dùng màu `tm-rule` — đây là **phán đoán**, không phải một lỗi có đáp án
**And** không thêm màu mới nào vào bảng token

**Given** hai loại phát hiện
**When** hiển thị cạnh nhau
**Then** phân biệt được bằng màu, không cần đọc nhãn

**Given** một segment chưa có bản dịch
**When** quét
**Then** không sinh phát hiện loại này

**Given** một Chương dài
**When** quét
**Then** phát hiện gắn đúng segment sinh ra nó, tham chiếu `segment.id` chứ không tham chiếu vị trí

---

### Story 9.3: Chạy theo yêu cầu, không chạy nền

**Covers:** FR82

As a người dịch dùng API key của chính mình,
I want proofreader chỉ chạy khi tôi bảo nó chạy,
So that tôi không bị tính phí cho những lần quét tôi không yêu cầu.

**Acceptance Criteria:**

**Given** proofreader
**When** cài đặt
**Then** chạy **theo yêu cầu của người dùng**

**Given** phạm vi quét
**When** người dùng chọn
**Then** chọn được **một segment**, **một Chương**, hoặc **vùng đang chọn**

**Given** ứng dụng đang chạy
**When** người dùng không gọi proofreader
**Then** **không có lời gọi AI nào** phát sinh từ proofreader

**Given** một phiên làm việc dài
**When** quan sát tài nguyên máy và lưu lượng mạng
**Then** proofreader không chiếm gì cả khi không được gọi

**Given** một lần quét đang chạy
**When** người dùng huỷ
**Then** dừng được giữa chừng

**Given** một lần quét
**When** hoàn tất
**Then** hiển thị số token và ước tính chi phí như mọi lời gọi AI khác

**Given** lệnh gọi proofreader và lệnh huỷ
**When** gọi
**Then** là command đăng ký, gán phím được

---

### Story 9.4: Hình dạng một phát hiện và xử lý từng cái một

**Covers:** FR83

As a người dịch,
I want mỗi cảnh báo nói rõ nó là lỗi gì và đề nghị sửa thế nào,
So that tôi phán xét được thay vì phải tự đoán ý của máy.

**Acceptance Criteria:**

**Given** một phát hiện
**When** hiển thị
**Then** gồm **loại lỗi · vị trí · giải thích ngắn · đề xuất sửa**

**Given** một phát hiện
**When** người dùng xử lý
**Then** chấp nhận hoặc bỏ qua **từng phát hiện một**

**Given** nhiều phát hiện trong một Chương
**When** hiển thị
**Then** **không có thao tác chấp nhận tất cả một cách mù**

**Given** người dùng chấp nhận một đề xuất sửa
**When** xảy ra
**Then** văn bản đổi theo đề xuất
**And** segment về trạng thái chưa xác nhận

**Given** phát hiện hiển thị trong một dải mọc dưới câu
**When** một câu kích hoạt đồng thời chốt Glossary và phát hiện Proofreader
**Then** chốt Glossary thắng, Proofreader đứng thứ hai
**And** gợi ý TM nhường cả hai

**Given** giải thích của một phát hiện
**When** soạn
**Then** nói việc, không đổ lỗi người dùng

**Given** thao tác chấp nhận, bỏ qua, và nhảy tới phát hiện kế tiếp
**When** gọi
**Then** là command đăng ký, gán phím được

---

### Story 9.5: Hiển thị tại chỗ bằng gạch chân lượn sóng

**Covers:** FR86

As a người dịch,
I want thấy chính xác cụm chữ nào có vấn đề ngay trên câu tôi đang đọc,
So that tôi không phải tự đối chiếu vị trí từ một danh sách rời.

**Acceptance Criteria:**

**Given** một phát hiện
**When** hiển thị
**Then** **gạch chân lượn sóng ngay dưới đúng cụm chữ có vấn đề** trong Editor

**Given** gạch chân
**When** đặt
**Then** ở `text-underline-offset: 4px`
**And** không chạm dấu nằm dưới của `ạ` `ộ` `ợ`

**Given** phát hiện
**When** hiển thị
**Then** **không dùng vạch lề** — vạch lề đã dùng hết năm giá trị cho trạng thái segment

**Given** hai lớp thông tin
**When** người dùng đọc
**Then** vạch lề nói **trạng thái câu**, gạch chân nói **chỗ nghi ngờ** — hai chỗ đọc khác nhau

**Given** kết quả proofread
**When** hiển thị
**Then** **không phải một danh sách rời** bắt người dùng tự đối chiếu vị trí

**Given** Editor
**When** có phát hiện
**Then** vẫn là một trang văn bản liền mạch — không chia thành ô hay bảng

**Given** màu của gạch chân
**When** chọn
**Then** đến từ bộ token đã kiểm tương phản, đạt WCAG AA ở cả hai theme

---

### Story 9.6: Bỏ qua có ghi nhớ

**Covers:** FR84

As a người dịch,
I want một cảnh báo tôi đã bác bỏ đừng quay lại lần sau,
So that tôi không tắt hẳn proofreader và mất luôn những cảnh báo đúng.

**Acceptance Criteria:**

**Given** một phát hiện
**When** người dùng đánh dấu *"không phải lỗi"*
**Then** lần quét sau **không báo lại phát hiện đó**

**Given** phạm vi ghi nhớ
**When** xác định
**Then** **trong cùng Tác phẩm**

**Given** ghi nhớ proofreader
**When** khoá
**Then** khoá theo `(work, chữ ký phát hiện)`
**And** **không** khoá theo `segment.id`

**Given** một segment bị gộp hoặc tách sau khi một phát hiện đã được bỏ qua
**When** quét lại
**Then** ghi nhớ **sống sót** — phát hiện đó vẫn không báo lại

**Given** cùng một loại phát hiện xuất hiện ở một Tác phẩm khác
**When** quét
**Then** **vẫn báo** — ghi nhớ không lan sang Tác phẩm khác

**Given** danh sách các phát hiện đã bỏ qua
**When** người dùng mở
**Then** xem lại và bỏ ghi nhớ được

---

### Story 9.7: Proofreader không tự sửa văn bản

**Covers:** FR85

As a người dịch,
I want chắc chắn không dòng nào trong bản dịch của tôi tự đổi,
So that tôi vẫn là người quyết định từng chữ trong bản của mình.

**Acceptance Criteria:**

**Given** proofreader chạy xong
**When** kiểm văn bản
**Then** **không một ký tự nào** trong bản dịch bị đổi

**Given** toàn bộ mã nguồn
**When** rà
**Then** không có đường nào từ proofreader ghi thẳng vào văn bản đích của segment

**Given** một đề xuất sửa
**When** áp
**Then** chỉ áp sau một thao tác **chấp nhận tường minh** của người dùng

**Given** một lần quét trên cả Chương
**When** hoàn tất mà người dùng chưa xử lý phát hiện nào
**Then** Chương giữ nguyên hoàn toàn

**Given** trạng thái xác nhận của các segment
**When** proofreader chạy
**Then** không segment nào đổi trạng thái

---

### Story 9.8: Đo tỷ lệ báo động giả

**Covers:** FR81

As a chủ dự án,
I want biết proofreader có đủ chính xác để đáng dùng hay không,
So that tôi không giữ một tính năng mà người dùng sẽ tắt hẳn đi.

**Acceptance Criteria:**

**Given** độ chính xác của phán đoán nghi về nghĩa
**When** nghiệm thu
**Then** **không nghiệm thu theo lối thông thường** — không có đáp án đúng để đối chiếu

**Given** chỉ số nghiệm thu
**When** xác định
**Then** là **tỷ lệ báo động giả**: số phát hiện bị đánh dấu *"không phải lỗi"* trên tổng số phát hiện

**Given** tỷ lệ báo động giả
**When** đo
**Then** đo trên bản dịch thật của chủ dự án qua ít nhất một Chương trọn vẹn

**Given** tỷ lệ đo được
**When** đánh giá
**Then** ngưỡng đỗ là **đủ thấp để người dùng không tắt hẳn tính năng**

**Given** chỉ số này
**When** thu thập
**Then** neo trực tiếp vào dữ liệu của Story 9.6 — mỗi lần bỏ qua có ghi nhớ là một điểm dữ liệu

**Given** tỷ lệ quá cao
**When** xảy ra
**Then** báo cáo kèm đề xuất — chỉnh prompt, hạ độ nhạy, hoặc tách riêng hai loại phát hiện
**And** ghi lại số đo để so ở lần sau

---

## Epic 10: Phát hành & tin cậy — vượt rào cản không ký số

Một người dịch phổ thông tải bản cài từ GitHub Releases, đối chiếu checksum SHA-256, đi theo **hướng dẫn cài đặt có ảnh chụp màn hình** xử lý tường minh Gatekeeper trên macOS và SmartScreen trên Windows, và cài được — dù bản phát hành **không ký, không notarize**. Trong ứng dụng, màn hình Attribution liệt kê mọi nguồn từ điển kèm giấy phép, **dựng từ các file dữ liệu có mặt**. Cơ chế cập nhật **chỉ kiểm tra và thông báo**.

### Story 10.1: Build công khai qua GitHub Actions

**Covers:** FR107

As a người dùng cẩn thận,
I want kiểm chứng được binary khớp với mã nguồn,
So that tôi tin được một bản cài không ai ký tên vào.

**Acceptance Criteria:**

**Given** một bản phát hành
**When** build
**Then** build công khai qua **GitHub Actions**, log xem được

**Given** dữ liệu từ điển
**When** CI cần
**Then** tải theo `dict-manifest.toml`
**And** **đối chiếu SHA-256** trước khi dùng

**Given** một file `.db` có checksum không khớp manifest
**When** CI chạy
**Then** build **thất bại**, không đóng gói tiếp

**Given** repo
**When** kiểm
**Then** chứa **mã build tool** và `dict-manifest.toml`
**And** **không chứa** nguồn thô 1,13 GB hay các file `.db` (~320 MB cho ba tệp)

**Given** parser định dạng từ điển
**When** kiểm bản phát hành
**Then** **không có mặt** — chúng chỉ sống trong `tools/dict-build`

**Given** bất kỳ ai
**When** đọc cấu hình CI
**Then** dựng lại được chuỗi từ mã nguồn tới artifact

---

### Story 10.2: Phát hành cho macOS và Windows

**Covers:** FR105

As a người dịch trên macOS,
I want một bản cài cho hệ điều hành của mình,
So that tôi không bị bỏ lại như thời QuickTranslator.

**Acceptance Criteria:**

**Given** một phiên bản
**When** phát hành
**Then** có bản cài cho **macOS** và bản cài cho **Windows**

**Given** bản phát hành
**When** công bố
**Then** qua **GitHub Releases**

**Given** cùng một mã nguồn
**When** chạy trên hai nền tảng
**Then** **hành vi tương đương** trên cả hai

**Given** font nhúng
**When** kiểm hai bản cài
**Then** cả hai dùng cùng bộ font đóng gói, không dùng font hệ điều hành

**Given** bản phát hành
**When** kiểm
**Then** **không ký số, không notarize** — đây là ràng buộc đã chấp nhận, ghi rõ trong ghi chú phát hành

**Given** dữ liệu từ điển
**When** kiểm bản cài
**Then** đã nhúng đầy đủ
**And** **không có cơ chế tải thêm sau khi cài**

---

### Story 10.3: Checksum SHA-256

**Covers:** FR106

As a người dùng cẩn thận,
I want đối chiếu được file mình tải với thứ dự án công bố,
So that tôi có một cách xác minh thay cho chữ ký số.

**Acceptance Criteria:**

**Given** mọi artifact phát hành
**When** công bố
**Then** kèm **checksum SHA-256**

**Given** checksum
**When** công bố
**Then** đặt ở nơi người dùng thấy được ngay trên trang Release

**Given** hướng dẫn cài đặt
**When** viết
**Then** chỉ rõ cách đối chiếu checksum trên macOS và trên Windows

**Given** checksum
**When** sinh
**Then** sinh trong CI, không sinh thủ công trên máy cá nhân

---

### Story 10.4: Màn hình Attribution

**Covers:** FR109

As a người dịch,
I want thấy mọi nguồn từ điển và giấy phép của chúng ngay trong ứng dụng,
So that tôi biết mình đang đọc dữ liệu của ai.

**Acceptance Criteria:**

**Given** màn hình Attribution
**When** mở
**Then** liệt kê **mọi nguồn từ điển** kèm giấy phép tương ứng và ghi công đầy đủ

**Given** danh sách nguồn
**When** dựng
**Then** dựng từ **các file `.db` thực sự có mặt** trong bản cài
**And** không từ một danh sách viết cứng trong mã

**Given** một file `.db` bị gỡ khỏi bản phát hành
**When** mở Attribution
**Then** ghi công của nguồn đó cũng biến mất
**And** **không để lại ghi công mồ côi**

**Given** trường giấy phép của mỗi nguồn
**When** hiển thị
**Then** biểu diễn được **cả giấy phép mở lẫn phép sử dụng riêng do tác giả cấp**

**Given** lớp HVTĐTD
**When** hiển thị
**Then** ghi rõ © Đặng Thế Kiệt, dùng theo **phép riêng tác giả cấp**, **không thuộc GPL v3**

**Given** các nguồn mang CC-BY-SA
**When** hiển thị
**Then** ghi công theo đúng yêu cầu của giấy phép đó

**Given** màn hình Attribution
**When** điều hướng
**Then** làm được bằng bàn phím

---

### Story 10.5: Giấy phép trong bản phát hành

**Covers:** FR110

As a người đóng góp,
I want thấy toàn bộ giấy phép đi kèm bản phát hành,
So that tôi biết mình dùng lại được những gì và với điều kiện nào.

**Acceptance Criteria:**

**Given** bản phát hành
**When** kiểm
**Then** kèm văn bản giấy phép **GPL v3** đầy đủ

**Given** bản phát hành
**When** kiểm
**Then** kèm **toàn bộ giấy phép của các bộ dữ liệu**

**Given** `LICENSE` và `NOTICE`
**When** viết
**Then** nêu rõ phần dữ liệu HVTĐTD thuộc © Đặng Thế Kiệt, dùng theo phép riêng tác giả cấp, **không thuộc GPL v3**

**Given** dữ liệu phái sinh từ nguồn CC-BY-SA
**When** phân phối
**Then** giữ **share-alike** theo đúng yêu cầu

**Given** mọi crate và thư viện frontend
**When** rà trước phát hành
**Then** đã kiểm tương thích GPL v3
**And** kết quả rà ghi vào bảng Stack

---

### Story 10.6: Hướng dẫn cài đặt có ảnh chụp màn hình

**Covers:** FR108

As a người dịch không rành kỹ thuật,
I want được chỉ từng bước cách vượt qua cảnh báo của hệ điều hành,
So that tôi không bỏ cuộc ở màn hình đầu tiên.

**Acceptance Criteria:**

**Given** hướng dẫn cài đặt
**When** viết
**Then** có cho **cả macOS và Windows**

**Given** hướng dẫn
**When** viết
**Then** có **ảnh chụp màn hình** cho từng bước

**Given** cảnh báo Gatekeeper trên macOS
**When** hướng dẫn
**Then** xử lý **tường minh** — chuột phải → Mở

**Given** cảnh báo SmartScreen trên Windows
**When** hướng dẫn
**Then** xử lý **tường minh** — More info → Run anyway

**Given** hướng dẫn
**When** viết
**Then** **không né tránh** việc bản phát hành không được ký số, và nói rõ vì sao

**Given** bước đối chiếu checksum
**When** hướng dẫn
**Then** đưa vào như một bước có thật, kèm lệnh cụ thể cho từng hệ điều hành

---

### Story 10.7: Cập nhật chỉ kiểm tra và thông báo

**Covers:** FR111

As a người dịch,
I want ứng dụng không bao giờ tự tải và tự cài một thứ gì,
So that một bản không ký số không trở thành đường tấn công vào máy tôi.

**Acceptance Criteria:**

**Given** cơ chế cập nhật
**When** chạy
**Then** **chỉ kiểm tra và thông báo** phiên bản mới

**Given** một phiên bản mới có sẵn
**When** thông báo
**Then** dẫn người dùng tới GitHub Releases để tự tải
**And** nhắc đối chiếu checksum

**Given** toàn bộ mã nguồn
**When** rà
**Then** **không có đường nào tự động tải** bản cập nhật
**And** **không có đường nào tự động cài**

**Given** việc kiểm tra phiên bản
**When** thực hiện
**Then** là **điểm ra mạng thứ hai** trong đúng ba điểm của toàn ứng dụng
**And** chỉ chạy theo một thao tác của người dùng

**Given** người dùng tắt kiểm tra phiên bản
**When** xảy ra
**Then** ứng dụng không gọi ra mạng vì lý do này nữa

**Given** chuỗi ràng buộc nối tiếp
**When** ghi lại trong tài liệu dự án
**Then** nêu rõ: không kinh phí → không ký số → niềm tin đến từ build công khai + checksum → **và vì vậy cấm cơ chế tự cập nhật**

---

### Story 10.8: Chính sách gỡ bỏ dữ liệu

**Covers:** FR112

As a chủ dự án,
I want gỡ một nguồn dữ liệu khỏi bản phát hành kế tiếp trong vài phút,
So that một khiếu nại không biến thành một đợt sửa mã nguồn.

**Acceptance Criteria:**

**Given** một nguồn dữ liệu cần gỡ
**When** thực hiện
**Then** thao tác là **xoá một file `.db`** khỏi `dict-manifest.toml` và khỏi bản đóng gói

**Given** thao tác gỡ
**When** thực hiện
**Then** **không đổi một dòng mã nguồn nào**

**Given** một nguồn vừa bị gỡ
**When** chạy lại toàn bộ bộ test tra cứu
**Then** **vẫn xanh**

**Given** một nguồn vừa bị gỡ
**When** mở màn hình Attribution
**Then** ghi công của nó cũng biến mất theo

**Given** lớp HVTĐTD bị gỡ
**When** tra một mục Hán Việt
**Then** rơi về nhãn tiếng Anh của lớp nền
**And** chức năng tra cứu vẫn đầy đủ

**Given** quy trình gỡ bỏ
**When** viết
**Then** ghi thành tài liệu trong repo, không nằm trong đầu một người

**Given** nghiệm thu của story này
**When** thực hiện
**Then** **chạy thao tác gỡ thật** với một lớp gỡ rời, không nghiệm thu bằng suy luận

---

### Story 10.9: Nghiệm thu cuối các ngưỡng phi chức năng

**Covers:** nghiệm thu cuối toàn bộ NFR1–NFR19

As a chủ dự án,
I want một cổng cuối kiểm lại những ngưỡng chỉ đo được khi sản phẩm đã đủ,
So that v1 phát hành với bằng chứng chứ không với giả định.

**Acceptance Criteria:**

**Given** bản cài hoàn chỉnh kèm toàn bộ dữ liệu từ điển và font
**When** đo dung lượng
**Then** **payload sản phẩm** nằm dưới trần **400.000.000 byte** của NFR6 *(trần nâng 2026-08-05; đo ở Story 1.10 = 343.991.430 byte)*
**And** **bản WebView2 Runtime nhúng của `.msi` ghi thành dòng riêng, không cộng vào phép đối chiếu** *(NFR6 sửa 2026-08-03)*
**And** con số thật của **cả hai dòng** được ghi vào ghi chú phát hành — người dùng tải về thấy dung lượng tổng, nên giấu dòng thứ hai là nói thiếu

**Given** dung lượng vượt trần
**When** xảy ra
**Then** báo cáo là **thay đổi tầng PRD cần chủ dự án quyết**

**Given** bản cài trên macOS và trên Windows
**When** chạy cùng một kịch bản
**Then** hành vi tương đương (NFR14)

**Given** toàn bộ crate Rust và thư viện frontend đang dùng
**When** rà lần cuối
**Then** tất cả tương thích GPL v3 (NFR15)
**And** bảng Stack khớp với cây phụ thuộc thật

**Given** một vòng dịch trọn một Chương — mở từ Library, tra cứu, gọi AI, đưa sang, sửa, gộp một câu, xác nhận, sang Chương kế
**When** thực hiện
**Then** làm được **không chạm chuột một lần nào** (NFR17)

**Given** mọi chuỗi giao diện
**When** rà
**Then** nằm trong `vi.json`
**And** không chuỗi tiếng Việt nào trong `.rs` hay `.vue` (NFR16)

**Given** mọi màu dùng trong ứng dụng
**When** kiểm tương phản lần cuối
**Then** đạt WCAG AA ở **cả hai** theme, kể cả Chế độ đọc và phần bôi màu diff của Review Mode

**Given** nghĩa vụ ngoài mã nguồn
**When** hoàn thành v1
**Then** **thông báo cho tác giả Đặng Thế Kiệt** — đề nghị tường minh trong thư đồng ý, và là điều kiện của phép sử dụng
