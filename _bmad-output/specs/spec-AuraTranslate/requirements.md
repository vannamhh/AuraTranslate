# Yêu cầu chức năng & phi chức năng — AuraTranslate

> Companion của `SPEC.md`. Catalog line-item của **121 FR** và **18 NFR**.
>
> **Quy ước đánh số:** mỗi `FRn` là ID toàn cục **không bao giờ được đánh số lại**. Yêu cầu bổ sung về sau nhận số mới ở cuối dãy, kể cả khi nằm giữa tài liệu. `NFRn` dùng dãy riêng. Ký hiệu `[An]` trỏ về `assumptions[]` của `SPEC.md`.

---

## Người dùng

| Vai | Mô tả | Ràng buộc |
|---|---|---|
| **Người dịch nghiêm túc** — *primary* | Dịch Anh/Trung → Việt trên **mọi lĩnh vực**: truyện, tài liệu kỹ thuật, báo chí, hợp đồng. Sẵn sàng dành nửa ngày cho một Chương vì chất lượng quan trọng hơn sản lượng | Là người dùng **duy nhất bắt buộc cài app** |
| **Reviewer** — *secondary* | Đọc và sửa bản dịch | **Không bắt buộc cài app.** Nhiều người sẽ không rời Google Docs, và điều đó được chấp nhận |
| **Người biên tập** — *cùng một người, vai khác* | Điều phối người khác dịch rồi tự biên tập lại bản dịch của họ **ngay trong app** | Không phải người dùng thứ hai — **là chính người dùng primary ở một vai khác**, đổi theo từng Tác phẩm |
| **Cộng đồng dịch giả Việt Nam** — *hưởng lợi rộng* | Những người bị bỏ lại khi QuickTranslator ngừng phát triển, đặc biệt trên macOS | Đối tượng của chiến lược đón nhận ở CAP-10 |

**Hệ quả kiến trúc từ vai Reviewer:** AuraTranslate là ứng dụng **một người dùng**. Cộng tác diễn ra qua trao đổi file, không qua tài khoản hay đồng bộ đám mây. Luồng Export/Import của CAP-8 vì vậy **không phải tính năng phụ — nó là cầu nối duy nhất** tới nhóm review.

**Hệ quả từ vai Người biên tập:** vai này **không** cần Review Mode (FR92–FR94) — Review Mode so *bản của tôi* với *bản reviewer đã sửa*, còn ở đây chỉ có **một bản** đang hoàn thiện. Môi trường đúng là Workspace bình thường với bản dịch điền sẵn (FR115). Thứ thật sự cần là **xuất xứ cấp segment** (FR117) và **bảo vệ TM** (FR118), vì hai vai dùng chung một kho TM.

**Hệ quả cho phạm vi:** người dùng primary làm việc trên *mọi lĩnh vực* với cặp ngôn ngữ cố định — đó là lý do custom prompt theo thể loại (FR69) là cơ chế chính, chứ không phải nhiều chế độ riêng theo lĩnh vực.

---

## CAP-1 — Library

Library là **màn hình mở đầu**. Luồng vào ứng dụng là `Mở app → Library → chọn Chương → Workspace`, không phải mở thẳng vào màn hình dịch.

### Mô hình dữ liệu

- **FR1.** Library tổ chức theo **hai tầng: Tác phẩm → Chương**.
- **FR2.** Tài liệu đơn lẻ được biểu diễn là một **Tác phẩm có đúng một Chương**. Không có loại thực thể thứ ba.
- **FR3.** Mỗi Tác phẩm mang metadata: tên, ảnh bìa *(tuỳ chọn)*, ngôn ngữ nguồn *(cố định, đặt lúc tạo)*, lĩnh vực/thể loại, ngày tạo, ngày sửa gần nhất.
- **FR4.** Glossary và Translation Memory gắn ở **tầng Tác phẩm** — mọi Chương trong cùng Tác phẩm dùng chung.

### Trạng thái & tiến độ

- **FR5.** Trạng thái vòng đời có ở cả hai tầng, **bốn giá trị**: *Chưa bắt đầu / Đang dịch / Tạm ngưng / Đã xong*. Chương mới nhập mặc định *Chưa bắt đầu*.
  > Bốn chứ không phải ba vì khi nhập một Tác phẩm 2000 chương thì 1999 chương chưa hề được đụng tới. *Tạm ngưng* phải giữ nghĩa **"đã làm dở rồi bỏ"** — đó mới là nhóm cần quay lại.
- **FR6.** Trạng thái Tác phẩm **suy ra tự động** từ trạng thái các Chương, nhưng người dùng **ghi đè thủ công được** — *Tạm ngưng* ở tầng Tác phẩm là quyết định của người.
- **FR7.** Mỗi Tác phẩm hiển thị tiến độ: số Chương đã xong trên tổng số, kèm thanh tiến độ trực quan.

### Tìm kiếm & duyệt

- **FR8.** Full-text search **xuyên toàn bộ Library**, tìm đồng thời trong văn bản nguồn và văn bản dịch. Kết quả kèm Tác phẩm, Chương và đoạn văn bản khớp.
- **FR9.** Tìm kiếm có **hai chế độ**: *chính xác dấu* (mặc định) và *khoan dung không dấu*. Hệ thống thử chế độ chính xác trước, chỉ nới lỏng khi không có kết quả hoặc khi người dùng yêu cầu.
- **FR10.** Lọc và sắp xếp Library theo trạng thái, lĩnh vực, ngôn ngữ nguồn và ngày sửa gần nhất.

### Đọc lại thành quả

- **FR11.** **Chế độ đọc:** đọc bản dịch đã hoàn thành, liên tục qua nhiều Chương, **không hiển thị công cụ biên tập**. Mặc định chỉ hiển thị bản dịch tiếng Việt; có **công tắc bật chế độ song ngữ**. Bound tối thiểu: độ rộng dòng giới hạn · cỡ chữ và chiều cao dòng chỉnh được · chế độ sáng và tối. Phạm vi và thao tác: xem **FR119** (đánh dấu chỗ cần sửa) và **FR120** (chỉ đọc phần đã xong, dừng ở biên).
  > Đây là **mức sàn nghiệm thu**, không phải thiết kế hoàn chỉnh — đặc tả typography đầy đủ thuộc `bmad-ux`. Là FR chứ không phải chi tiết UI vì mục đích số một của Library là *"vào xem/đọc lại bài viết, truyện mình đã dịch"* — một bảng dữ liệu không đáp ứng được.
- **FR119.** **Đánh dấu chỗ cần sửa khi đang đọc.** Một thao tác đánh dấu câu rồi **đọc tiếp ngay** — phiên đọc không bị cắt. Các chỗ đánh dấu gom thành **một danh sách theo Tác phẩm**, từ đó mở Workspace sửa một lượt. Thao tác thứ hai **nhảy thẳng** sang Workspace khi muốn sửa ngay. Affordance **không hiện thường trực** — chỉ hiện khi con trỏ chuột hoặc tiêu điểm bàn phím chạm câu.
  > Nhảy sang Workspace mỗi lần phát hiện sẽ **cắt đứt phiên đọc** — mà phiên đọc là thứ Chế độ đọc tồn tại để bảo vệ. Đây không phải "công cụ biên tập" theo nghĩa FR11 cấm: FR11 cấm bộ máy biên tập (vạch lề trạng thái, nút xác nhận, panel), còn một dấu và một đường điều hướng thì không sửa gì cả.
- **FR120.** **Chế độ đọc chỉ đọc phần đã xong, dừng ở biên tường minh.** Đọc liên tục qua các Chương **Đã xong**; chạm Chương chưa xong thì dừng ở **mốc rõ ràng** kèm đường sang Workspace. Chương chưa dịch **không hiển thị nguyên văn**. Câu **chưa xác nhận** trong Chương đã xong **vẫn hiện nhưng có dấu nhẹ**.
  > Chương có thể được đánh dấu *Đã xong* bằng tay (FR6) trong khi còn câu chưa xác nhận. Hiện chúng như đã hoàn chỉnh là **nói dối về trạng thái công việc** — trái tinh thần FR5 và FR58.
- **FR12.** Mở một Chương từ Library đưa thẳng vào Workspace, đúng Chương đó, **khôi phục vị trí làm việc lần trước**.

### Nhập tài liệu

- **FR13.** Tạo Tác phẩm mới từ file (`.txt`, `.docx`, `.md`) hoặc từ văn bản dán trực tiếp.
- **FR14.** **Nhập hàng loạt:** chọn nhiều file cùng lúc, hoặc tách một file lớn thành nhiều Chương theo mẫu phân tách do người dùng cấu hình (mẫu tiêu đề hoặc regex), **có màn hình xem trước kết quả tách trước khi xác nhận**.
  > Bắt buộc: một bộ truyện 2000 chương không thể nhập tay từng chương. Không có FR14 thì mô hình hai tầng ở FR1 không dùng được trên thực tế.
- **FR15.** Sau khi nhập: đổi tên, sắp xếp lại thứ tự, gộp và tách Chương.

### Nhập tài liệu song ngữ

*(FR115–FR116 mang số cuối dãy — bổ sung 2026-08-02 sau khi chủ dự án làm rõ có Tác phẩm do người khác dịch và bàn giao dưới dạng file hai cột.)*

- **FR115.** **Nhập tài liệu song ngữ tạo Tác phẩm hoàn chỉnh** từ file hai cột (bảng `.docx`, bảng `.md`, `.csv`/`.tsv`). Người dùng khai báo **cột nào là nguồn, cột nào là đích** và ngôn ngữ nguồn. Kết quả: Tác phẩm đầy đủ với segment nguồn, segment đích, đã khớp cặp. **Bắt buộc xem trước trước khi ghi xuống đĩa**, cùng khuôn FR14. Mọi Chương vào trạng thái **Đang dịch**, mọi segment **chưa xác nhận** — kể cả khi bản dịch trông đã hoàn chỉnh.
  > FR13/FR14 chỉ nhận văn bản **nguồn**; FR90 đòi Tác phẩm **phải tồn tại sẵn**. Tác phẩm do người khác dịch rơi vào giữa hai đường đó. Trạng thái chưa xác nhận là nhất quán với FR58: hệ thống **không bao giờ tự coi một câu là đã xong**.
- **FR116.** **Khớp câu trong phạm vi từng cặp hàng.** Hàng trong bảng hai cột thường là **đoạn**, segment là **câu** (FR23). Hệ thống tách hai phía thành câu và khớp **bên trong từng cặp hàng**; chỗ số câu lệch nhau **phải hiện ra cho người dùng nối tay** — mẫu *máy khớp, người sửa* của FR91.
  > Nhẹ hơn FR91 vì cặp hàng đã cho sẵn ranh giới: không gian khớp chỉ trong một hàng, không phải cả Chương. Số câu lệch là chuyện thường gặp khi dịch Trung sang Việt.

### Hình ảnh

- **FR43.** Chế độ đọc hiển thị hình ảnh nhúng **đúng vị trí** của chúng trong văn bản.
- **FR45.** Hình ảnh **lưu bên trong `.atproj`**, không phụ thuộc đường dẫn ngoài. Một Tác phẩm phải mang đi được nguyên vẹn khi copy sang máy khác.

---

## CAP-2 — Workspace

### Bố cục

- **FR16.** **Bốn panel trong một cửa sổ ứng dụng duy nhất:** *Source*, *Lookup*, *AI Translation*, *Editor*. Trả lời trực tiếp nỗi đau "bốn đến năm cửa sổ mở cùng lúc".
- **FR17.** Panel kéo thả để dock/undock, gộp thành tab, và thay đổi kích thước. Mỗi panel **ẩn được hoàn toàn** — người dịch không dùng AI phải giấu được panel AI Translation.
- **FR18.** Bố cục workspace lưu và khôi phục giữa các phiên. Hỗ trợ nhiều **preset bố cục** và chuyển nhanh giữa chúng.

### Panel Source

- **FR19.** Panel Source hiển thị văn bản gốc (Anh hoặc Trung) kèm **tab Hán Việt** cho tài liệu tiếng Trung — xem ở chế độ chuyển đổi hoặc song song.
- **FR42.** Panel Source hiển thị hình ảnh nhúng **đúng vị trí** trong văn bản gốc.
- **FR44.** **Alt-text của hình ảnh là một segment dịch được** — tham gia Translation Memory, Glossary và luồng xác nhận như mọi segment khác. Điều kiện để yêu cầu bảo lưu alt-text khi xuất `.md` (FR88) có nghĩa thật: alt-text phải được *dịch*, không chỉ được *giữ lại*.

### Thao tác xuyên panel

- **FR20.** **Sync Scrolling** đồng bộ vị trí cuộn giữa Source, AI Translation và Editor. Có công tắc bật/tắt rõ ràng.
- **FR21.** **Auto-Lookup:** bôi đen một cụm từ ở Source, AI Translation hoặc Editor → kết quả tra cứu hiện **ngay** ở panel Lookup. Không copy, không paste, không chuyển cửa sổ.
- **FR22.** **Global Hotkeys** cho thao tác lặp lại: dịch segment hiện tại, chuyển focus giữa panel, xác nhận segment, tra cứu cụm đang chọn, bật/tắt sync scroll. **Toàn bộ phím tắt cấu hình lại được.**

### Biên tập theo segment

- **FR23.** Editor phân đoạn văn bản thành **segment ở cấp độ câu**. Tiếng Trung tách theo `。！？；`; tiếng Anh tách theo `. ! ?` có xử lý viết tắt không phải kết câu **[A4]**.
- **FR78.** Người dùng **gộp hai segment liền nhau** hoặc **tách một segment** khi máy tách sai. Bắt buộc có, vì tách câu tự động luôn sai ở một tỷ lệ nhất định — nhất là dấu chấm trong viết tắt, số thập phân và hội thoại.
- **FR24.** Người dùng **xác nhận từng segment**. Segment đã xác nhận được đánh dấu trực quan phân biệt với segment đang dở.
  > **Ngữ nghĩa không đổi theo vai:** dù tự dịch hay đang biên tập câu do người khác dịch, *xác nhận* luôn nghĩa là **"câu này đạt chuẩn của tôi"**. Cái đổi theo vai là **xuất xứ** (FR117).
- **FR117.** **Xuất xứ bản dịch ở cấp segment**, ba giá trị: *tôi dịch* · *người khác dịch* · *nhập từ tài liệu song ngữ*. **Suy ra tự động từ hành vi**, không hỏi người dùng: gõ rồi xác nhận → *tôi dịch*; **sửa** câu sẵn có rồi xác nhận → *tôi dịch* (câu sau khi sửa là chữ của bạn); duyệt **nguyên văn** không sửa → *người khác dịch*; segment nhập từ FR115 chưa ai đụng → *nhập từ tài liệu song ngữ*.
- **FR25.** Điều hướng nhanh giữa segment: kế tiếp, trước đó, và **segment chưa dịch kế tiếp**.
- **FR26.** Chuyển Chương ngay trong Workspace (Chương trước / Chương sau) mà không phải quay về Library.

---

## CAP-3 — Embedded Dictionary & Lookup

> Từ điển nhúng offline **không phải một tính năng trong danh sách — nó là điều kiện tồn tại của sản phẩm.**

### Nền tảng

- **FR27.** Toàn bộ dữ liệu từ điển **nhúng trong bản cài**. Tra cứu hoạt động 100% offline, không có cơ chế tải thêm sau khi cài đặt.

### Hình dạng một kết quả tra cứu

- **FR28.** Panel Lookup hiển thị một **bản ghi có cấu trúc**, không phải một đoạn văn bản: **nguồn · từ loại · nghĩa · ví dụ[] · trích dẫn[] · ghi chú**.
- **FR29.** Một từ có nhiều từ loại phải hiện thành **nhiều mục riêng biệt**, mỗi mục có ví dụ riêng — không phải một chuỗi nghĩa gộp.
- **FR30.** **Ví dụ gắn với từng từ loại**, không gắn với cả từ. **Trích dẫn** là trường riêng biệt với ví dụ: trích dẫn có xuất xứ văn bản.

### Nguyên tắc nền — hiển thị nguồn

- **FR31.** **Mọi định nghĩa phải hiển thị nguồn của nó. Không có ngoại lệ, không có chế độ ẩn nguồn.**
- **FR32.** Khi các nguồn bất đồng về một mục từ, hệ thống **hiển thị đồng thời cả hai**, không hợp nhất thành một câu trả lời duy nhất.
  > Không phải phép lịch sự học thuật mà là yêu cầu bắt buộc: mỗi nguồn có khiếm khuyết riêng đã biết. **Một công cụ hợp nhất mọi từ điển thành một câu trả lời duy nhất là một công cụ giấu đi sai sót** — và không có nguồn nào đúng để chọn làm "câu trả lời duy nhất".

### Nội dung theo ngôn ngữ

- **FR33.** **Tab Hán Việt:** hiển thị âm Hán Việt cho từng ký tự tiếng Trung trong văn bản nguồn.
- **FR34.** Mục từ **tiếng Anh** phải có nhãn từ loại và nghĩa tiếng Việt.
- **FR35.** Mục từ **tiếng Trung** phải có nhãn từ loại và ít nhất một ví dụ cách dùng khi nguồn có dữ liệu. Ở v1, nhãn từ loại và bản dịch ví dụ **bằng tiếng Anh được chấp nhận** và phải được **đánh dấu rõ là nhãn ngoại ngữ**. Căn cứ và giải pháp lớp C: `data-sources.md`.
  > Yêu cầu này phát biểu cho **lớp nền** và không đổi khi lớp HVTĐTD có mặt. Khi lớp HVTĐTD được bật, mục từ Hán Việt hiển thị từ loại · ví dụ · trích dẫn **bằng tiếng Việt**; gỡ file `.db` đó ra thì rơi về nhãn tiếng Anh của lớp nền và tra cứu vẫn đầy đủ. **Chính cặp hành vi này là test nghiệm thu cụ thể cho FR36.**

### Kiến trúc lớp nguồn

- **FR36.** Nguồn từ điển đóng gói theo mô hình **"nền có giấy phép sạch + lớp gỡ rời được"**. Gỡ bất kỳ lớp gỡ rời nào **không được làm hỏng chức năng tra cứu**.
- **FR37.** Người dùng bật/tắt từng nguồn từ điển trong panel Lookup.
- **FR38.** Ghi công đầy đủ từng nguồn: trong ứng dụng (màn hình Attribution) và trong bản phát hành.

### Hiệu năng tra cứu

- **FR39.** Tra cứu tiếng Trung phải **trả về kết quả cho truy vấn 1 ký tự, 2 ký tự và 3 ký tự trở lên**.
  > Viết thành FR vì FTS5 trigram trả về **rỗng** cho truy vấn 1–2 ký tự **mà không báo lỗi** — truy vấn chạy trong 0,01 ms rồi trả rỗng. Phần lớn từ tiếng Trung được tra nhiều nhất lại dài 1–2 ký tự (山, 打, 中國, 學生). Không có FR này, lỗi lọt vào sản phẩm và biểu hiện thành "tra từ không ra kết quả".
- **FR40.** Tra cứu tiếng Anh nhận diện biến thể hình thái. **Giới hạn đã biết và chấp nhận:** đây là *stemming*, không phải *lemmatization* thật — hệ sinh thái Rust chưa có lemmatizer trưởng thành. Đủ cho khớp Glossary, không xử lý được biến thể bất quy tắc.

### Tiện ích

- **FR41.** Lịch sử tra cứu trong phiên làm việc, và ghim mục từ để tra lại nhanh. **[A9]**

---

## CAP-4 — Glossary & thuật ngữ

### Cấu trúc

- **FR46.** Glossary có **hai tầng**: *toàn cục* và *theo Tác phẩm*. Khi một thuật ngữ tồn tại ở cả hai tầng, **tầng Tác phẩm thắng**.
- **FR47.** Mỗi mục gồm: thuật ngữ nguồn, bản dịch đã chốt, ghi chú, phân loại *(tên người / địa danh / thuật ngữ chuyên ngành / khác)*, ngày thêm, và **xuất xứ** — *nhập thủ công / đề xuất khi nhập tài liệu / thu hoạch từ bản review*.
  > Trường xuất xứ cho phép đánh giá độ tin cậy của một mục, và cho phép rà lại toàn bộ những gì máy đã đề xuất nếu về sau phát hiện cơ chế đề xuất có lỗi hệ thống.

### Thao tác

- **FR48.** Thêm nhanh vào Glossary từ **bất kỳ panel nào**: bôi đen → thêm thuật ngữ → chọn tầng. Không phải rời màn hình đang làm việc.
- **FR49.** Quản lý Glossary: tìm kiếm, sửa, xoá, **nhập và xuất** dưới định dạng văn bản mở (CSV/TSV) để chia sẻ giữa người dịch.
- **FR79.** **Xuất và nhập bộ prompt** dưới dạng file văn bản mở, để người dịch chia sẻ prompt theo thể loại cho nhau.
- **FR50.** Mọi thuật ngữ có trong Glossary được **đánh dấu trực quan trong panel Source**.
- **FR51.** Khớp thuật ngữ **phân theo ngôn ngữ**: tiếng Trung khớp chính xác; tiếng Anh khớp mờ ở cấp hình thái từ (stemming).

### Tự động đề xuất — ba cơ chế

- **FR52.** **Quét khi nhập tài liệu:** ứng viên = chuỗi lặp lại **từ 5 lần trở lên** *và* **không có trong từ điển nhúng** **[A10]**. Ngưỡng **cấu hình lại được**. Tiếng Trung: chuỗi ký tự lặp không có trong từ điển, đối chiếu danh sách họ phổ biến để đoán tên người. Tiếng Anh: cụm viết hoa không đứng đầu câu.
- **FR53.** **Duyệt hàng loạt:** ứng viên hiện thành danh sách xếp theo tần suất, kèm **số lần xuất hiện** và **ví dụ ngữ cảnh**. Người dùng duyệt hoặc bỏ bằng thao tác một phím — **không phải gõ**. Phím nhận đồng thời nhận **bản dịch đề xuất** khi có (FR113); phân loại đổi bằng phím số; duyệt **dừng giữa chừng và mở lại đúng chỗ** được.
- **FR54.** **Thu hoạch từ bản review:** khi nhập lại bản Reviewer đã sửa, nếu phát hiện reviewer đổi thuật ngữ *X* thành *Y* một cách **nhất quán**, hệ thống đề xuất bổ sung cặp đó. Đề xuất phải nêu **số lần đổi trên tổng số lần xuất hiện**.
  > **Đây là đường bảo hiểm cho rủi ro "vòng phản hồi đứt".** Ngay cả khi người dịch không bao giờ mở Diff Viewer, công cụ vẫn hấp thụ bài học của reviewer. Vòng lặp học hỏi đóng lại **ở tầng hệ thống** thay vì trông chờ vào kỷ luật con người.
- **FR55.** **Mọi đề xuất tự động đều phải qua duyệt của người dùng. Không có cơ chế nào được tự ghi vào Glossary.**

### Vòng đời bản dịch của một mục Glossary

*(FR113–FR114 mang số cuối dãy theo quy ước không đánh số lại — bổ sung 2026-08-02 để giải mâu thuẫn giữa FR47 và FR53.)*

- **FR113.** **Đề xuất bản dịch cho ứng viên:** với ứng viên tiếng Trung, hệ thống đề xuất sẵn bản dịch bằng **âm Hán Việt** lấy từ dữ liệu đã nhúng (FR33) nên **chạy hoàn toàn ngoại tuyến**. Phím nhận (FR53) nhận **cả thuật ngữ lẫn bản dịch đề xuất**; mục vào Glossary ở trạng thái **đã chốt**. Sửa được về sau như mọi mục khác.
  > Với danh từ riêng tiếng Trung, âm Hán Việt **chính là** bản dịch quy ước cộng đồng dịch giả Việt Nam vẫn dùng — `北涼` → *Bắc Lương*, `徐鳳年` → *Từ Phượng Niên*. Đây là nhóm chiếm phần lớn ứng viên FR52 quét ra, nên đề xuất là **tra cứu một dữ kiện đã có**, không phải phỏng đoán của máy.
- **FR114.** **Trạng thái *chờ chốt bản dịch*:** khi không đề xuất được — ứng viên tiếng Anh, hoặc chuỗi tiếng Trung mà âm Hán Việt không phải cách dịch phù hợp — thao tác nhận vẫn đưa mục vào Glossary nhưng trường bản dịch ở trạng thái **chờ chốt**. Lần **đầu tiên** gặp thuật ngữ đó trong Workspace, hệ thống hỏi **một lần** rồi khoá thành đã chốt.
  > Chốt lúc đó chứ không lúc duyệt vì ở bảng chờ người dùng nhìn một danh sách trần hàng trăm dòng, còn trong Workspace họ nhìn đúng câu chứa thuật ngữ. Quyết định về cách dịch cần ngữ cảnh.

  **Cả hai FR không làm suy yếu FR55:** máy đề xuất, người bấm. Không mục nào vào Glossary mà không có một thao tác của người dùng.

---

## CAP-5 — Translation Memory & tái sử dụng

### Tích luỹ

- **FR56.** **Ghi tự động:** mỗi khi người dùng xác nhận một segment, cặp *(nguồn → đích)* được ghi vào TM. **Không có thao tác thủ công nào.** Cặp TM mang **xuất xứ** kế thừa từ segment (FR117).
- **FR118.** **Translation Memory không được trộn phong cách.** Mỗi cặp mang xuất xứ *của tôi* hoặc *của người khác*. **Smart RAG Injector ưu tiên cặp *của tôi*;** cặp xuất xứ khác chỉ chèn khi không đủ, và khi chèn phải **đánh dấu rõ trong prompt là văn phong tham khảo**.
  > **Đường bảo hiểm cho một lời hứa lõi.** FR70 tồn tại để AI học **phong cách của chính người dùng**. Chủ dự án làm cả hai vai (§ Người dùng), nên nếu mọi câu xác nhận đổ chung một kho không phân biệt thì TM đầy lên bằng văn phong người khác và AI học sai người — **lệch dần, không có gì báo**. Không bỏ ghi TM khi biên tập vì như vậy mất luôn những câu chính người dùng viết lại từ đầu.
- **FR57.** TM có **phạm vi kép**: TM riêng theo Tác phẩm và TM chung toàn cục.

### Tái sử dụng — ba tầng độc lập

- **FR58.** **Khớp tuyệt đối (100%):** segment y hệt đã dịch trước đây được **điền sẵn** và **đánh dấu là gợi ý cần xác nhận**. Hệ thống **không** tự coi segment đó là đã hoàn thành.
- **FR59.** **Khớp mờ:** hiển thị bản dịch cũ tương tự kèm **phần trăm khớp** và **diff phần khác biệt**.
- **FR60.** **Concordance:** tra ngược trên toàn bộ TM. Kết quả đưa vào **panel Lookup**, cùng chỗ với kết quả từ điển.
- **FR61.** Thuật toán khớp **phân theo ngôn ngữ**: tiếng Trung dùng n-gram ký tự; tiếng Anh dùng token n-gram sau stemming.

### Quản lý & mang đi

- **FR62.** Xem, sửa và xoá từng mục TM. Danh sách hiển thị **xuất xứ** từng cặp và **lọc được theo xuất xứ** — để rà lại hoặc dọn phần không phải văn phong của mình.
- **FR63.** Khi cùng một segment nguồn có **nhiều bản dịch khác nhau**, hệ thống giữ lại tất cả và hiển thị tất cả kèm ngày, thay vì ghi đè.
- **FR64.** **Xuất và nhập TMX** — dữ liệu phải sống lâu hơn phần mềm, người dùng không bị khoá vào `.atproj`.

---

## CAP-6 — AI mở & Smart RAG Injector

### Kết nối

- **FR65.** **BYOK:** người dùng nhập API key của nhà cung cấp mình chọn.
- **FR66.** **Local LLM:** kết nối tới endpoint tương thích OpenAI (Ollama, LM Studio) qua **cùng một đường cấu hình** với BYOK.
- **FR67.** **API key lưu trong keychain / credential manager của hệ điều hành.** Không lưu văn bản thuần trong file dự án hay file cấu hình, không đồng bộ đi đâu.
- **FR68.** Cấu hình AI (nhà cung cấp, mô hình, tham số sinh) đặt ở **tầng toàn cục**, **ghi đè được theo từng Tác phẩm**.

### Prompt

- **FR69.** **Custom prompt theo thể loại** (tiên hiệp, khoa học, pháp lý, báo chí…) và theo quy chuẩn dịch. Tồn tại ở cả hai tầng; **tầng Tác phẩm thắng**.
  > Là cơ chế chính chứ không phải tính năng phụ: phạm vi đã chốt là *mọi lĩnh vực* với cặp ngôn ngữ cố định. Custom prompt là thứ khiến **một công cụ phục vụ được nhiều lĩnh vực** mà không cần nhiều chế độ riêng.

### Smart RAG Injector

- **FR70.** Trước mỗi lần gọi AI, hệ thống quét câu nguồn và **chèn động vào prompt**: (a) các thuật ngữ Glossary xuất hiện trong câu kèm bản dịch đã chốt; (b) các segment tương tự tìm được trong TM.
  > **Chỉ mục đã chốt được chèn.** Mục còn ở trạng thái chờ chốt (FR114) không tham gia ép AI. **Ưu tiên cặp TM của chính người dùng** (FR118); cặp xuất xứ khác chỉ chèn khi không đủ và phải đánh dấu là văn phong tham khảo.
  > **Đây là chỗ TM và Glossary nhân giá trị cho nhau thay vì là hai tính năng rời.** Chèn *"những câu tương tự trước đây được dịch thế này"* khiến AI học **phong cách của chính người dùng** thay vì áp phong cách chung.
- **FR71.** Người dùng **xem được prompt cuối cùng đã gửi đi**, bao gồm toàn bộ phần chèn động. Nếu người dùng không nhìn được vào hộp đen thì họ không thể là người quyết định — và đây là công cụ chẩn đoán duy nhất khi AI không tuân thủ Glossary.

### Luồng dịch

- **FR72.** Kết quả AI hiện ở **panel AI Translation** và **không tự động ghi vào Editor**. Người dùng chủ động đưa sang.
- **FR73.** Dịch theo **từng segment** và theo **lô nhiều segment liên tiếp**, **huỷ được giữa chừng**.
- **FR74.** Kết quả hiện **dần theo dòng chảy (streaming)** khi mô hình đang sinh.
- **FR75.** Khi gặp lỗi mạng hoặc lỗi API: thông báo rõ nguyên nhân, **không mất công việc đang làm**, cho phép thử lại **do người dùng chủ động**. Hệ thống **không được tự động thử lại** — với BYOK, mỗi lần gọi là tiền của người dùng.
- **FR76.** Hiển thị số token đã dùng và **ước tính chi phí** cho mỗi lần gọi.

### Ranh giới

- **FR77.** **Ứng dụng phải hoạt động đầy đủ khi không cấu hình AI.** Mọi năng lực ngoài CAP-6 và CAP-7 phải chạy được mà không cần một API key nào.

---

## CAP-7 — AI Proofreader

- **FR80.** Quét **chính tả và ngữ pháp tiếng Việt** trên bản dịch của người dùng.
- **FR81.** **Đối chiếu bản dịch với bản gốc**, đánh dấu đoạn nghi **dịch sai**, **dịch thoát nghĩa quá xa**, hoặc **cấu trúc câu tối nghĩa**.
  > **Tiêu chí nghiệm thu:** độ chính xác của phán đoán này không nghiệm thu được theo lối thông thường — không có đáp án đúng để đối chiếu. Thứ **đo được** là **tỷ lệ báo động giả**: số phát hiện mà người dùng đánh dấu *"không phải lỗi"* (FR84) trên tổng số phát hiện. Ngưỡng là **tỷ lệ đó đủ thấp để người dùng không tắt hẳn tính năng**.
- **FR82.** Proofreader chạy **theo yêu cầu** (trên một segment, một Chương, hoặc vùng đang chọn), **không chạy nền liên tục** — với BYOK là tính phí ngoài ý muốn, với local LLM là chiếm tài nguyên máy suốt phiên.
- **FR83.** Mỗi phát hiện gồm: **loại lỗi**, vị trí, **giải thích ngắn**, **đề xuất sửa**. Người dùng chấp nhận hoặc bỏ qua **từng phát hiện một**.
- **FR84.** **Bỏ qua có ghi nhớ:** đánh dấu một phát hiện là *"không phải lỗi"* thì lần quét sau không báo lại trong cùng Tác phẩm. Không có cơ chế này, proofreader lặp lại cùng cảnh báo sai mỗi lần quét và người dùng sẽ tắt hẳn nó — mất luôn phần cảnh báo đúng.
- **FR85.** **Proofreader không được tự sửa văn bản.** Mọi thay đổi phải do người dùng chấp nhận.
- **FR86.** Kết quả hiển thị **ngay tại chỗ trên Editor**, không phải một danh sách rời bắt người dùng tự đối chiếu vị trí.

---

## CAP-8 — Cầu nối Reviewer

> Reviewer **không bắt buộc cài app**, nên trao đổi file là **cầu nối duy nhất** tới nhóm review. Không có nó, AuraTranslate cắt người dùng khỏi nhóm của họ.

### Xuất

- **FR87.** Xuất **`.docx` dạng bảng hai cột**: cột trái văn bản gốc, cột phải bản dịch, **đối xứng theo segment**.

- **FR121.** Xuất **`.docx` một khối, đối xứng theo đoạn** — dành cho việc **đăng bài**. Vẫn là bảng hai cột, nhưng **một hàng duy nhất cho cả Chương** và **không đường kẻ ngang**; hai ô giữ **đúng số lần xuống đoạn như nhau** để đối chiếu bằng mắt vẫn được. Phạm vi xuất theo FR89.
  - **Nghiệm thu:** bôi đen cột phải rồi dán sang trình soạn thảo của website ra **văn bản liền mạch**, không kèm mảnh vụn bảng biểu.
  - **Không nhập lại được** — không giữ ranh giới câu nên FR90/FR91 không áp dụng. Nằm ở **cuối vòng khứ hồi**, cùng nhóm với text thuần; **màn hình xuất phải nói rõ điều đó ngay lúc chọn định dạng**, không để trong tài liệu hướng dẫn.
  - **Câu chưa xác nhận không được đánh dấu trong file xuất** — nền màu hay dòng ghi chú xen giữa văn xuôi sẽ đi thẳng vào bài đăng. Thay vào đó **cảnh báo trước lúc xuất** khi phạm vi còn câu chưa xác nhận.
  - *Là FR riêng chứ không phải tuỳ chọn của FR87: khác đơn vị (đoạn vs segment), khác mục đích (đăng bài vs review), khác vị trí trong vòng khứ hồi. Gộp thành hai ô tick sẽ khiến người dùng chọn nhầm bản cắt đứt vòng học hỏi mà không biết.*

- **FR88.** Xuất **`.md` hoặc text thuần**, **bảo lưu liên kết hình ảnh và alt-text đã dịch** (FR44).
- **FR89.** Xuất theo một Chương, nhiều Chương đã chọn, hoặc cả Tác phẩm.

### Nhập

- **FR90.** Nhập lại file `.docx` / `.md` mà Reviewer đã chỉnh sửa, vào đúng Tác phẩm hiện có.
- **FR91.** **Segment alignment:** hệ thống khớp cấu trúc đoạn giữa file nhập và dữ liệu sẵn có. Segment **không khớp được phải hiện ra cho người dùng nối tay** — mẫu chuẩn của ngành là *máy khớp, người sửa*.

### Diff Viewer

- **FR92.** **Review Mode:** workspace chuyển sang bố cục **hai cửa sổ side-by-side** — trái là bản dịch của người dùng, phải là bản đã nhập từ Reviewer.
- **FR93.** Trong Review Mode, **ẩn văn bản gốc** và dùng thuật toán diff **bôi màu phần thêm / xoá / sửa** giữa hai bản dịch.
- **FR94.** Từ Review Mode, **chấp nhận từng thay đổi** vào bản dịch của mình, hoặc bỏ qua.

### Đường bảo hiểm

- **FR95.** Việc nhập bản review **kích hoạt cơ chế thu hoạch thuật ngữ (FR54) một cách độc lập** — kể cả khi người dùng **không bao giờ mở Review Mode**.
  > **FR quan trọng nhất của CAP-8, tồn tại vì một rủi ro chưa giải được.** Nguyên nhân gốc của việc người dịch không xem lại bản review vẫn chưa xác định (Q1), nên **không thể khẳng định Diff Viewer sẽ được dùng thật**. FR95 bảo đảm công sức của reviewer vẫn chuyển hoá thành giá trị ngay cả khi FR92–FR94 không bao giờ được mở tới.

---

## CAP-9 — Dự án & dữ liệu

### Nguồn sự thật

- **FR96.** Mỗi Tác phẩm lưu thành **một `.atproj` trên đĩa** — **nguồn sự thật**; người dùng copy, sao lưu và di chuyển tự do.
- **FR97.** `.atproj` **tự chứa** mọi thứ thuộc về Tác phẩm: văn bản nguồn, bản dịch, segment, lịch sử phiên bản, Glossary dự án, TM dự án, prompt dự án và hình ảnh. **Copy sang máy khác phải mở được nguyên vẹn.**
- **FR98.** Một **chỉ mục Library trung tâm** phục vụ tìm kiếm xuyên Tác phẩm (FR8). Chỉ mục **phải dựng lại được hoàn toàn từ các `.atproj`** — mất chỉ mục không được làm mất dữ liệu.
- **FR99.** **Quét lại thư mục:** phát hiện `.atproj` mới xuất hiện, `.atproj` đã bị di chuyển hoặc xoá (mục mồ côi), và cập nhật chỉ mục tương ứng.

### Lưu và phiên bản

- **FR100.** **Auto-save định kỳ, không gián đoạn UI.** Không được có gai trễ cảm nhận được khi đang gõ.
- **FR101.** **Versioning:** lưu lịch sử các phiên bản dịch của từng segment; xem lại và **khôi phục** được.
- **FR102.** **Sao lưu bằng cách copy thư mục là đủ.** Không được yêu cầu một thao tác export riêng để có bản sao lưu dùng được.

### Phân cấp cấu hình

- **FR103.** Mọi cấu hình tồn tại ở **hai tầng**, tầng Tác phẩm ghi đè tầng Global.

  | Tầng | Chứa gì |
  |---|---|
  | **Global** | Cấu hình AI, prompt chung, Glossary toàn cục, TM toàn cục, phím tắt, preset bố cục |
  | **Tác phẩm** | Glossary riêng, prompt riêng, TM riêng, **ngôn ngữ nguồn (cố định, đặt lúc tạo)** |

### Quyền riêng tư

- **FR104.** **Không telemetry.** Ứng dụng không gửi bất kỳ dữ liệu nào ra ngoài, trừ nội dung mà người dùng **chủ động** gửi cho nhà cung cấp AI đã cấu hình.

---

## CAP-10 — Phát hành & tin cậy

> **Ràng buộc nền:** không có kinh phí cho ký số. Mọi bản phát hành **không ký, không notarize**. Đây là ràng buộc thật, không phải thiếu sót cần khắc phục — và là **rào cản đón nhận có thật** với người dùng không rành kỹ thuật. Các FR dưới đây bù bằng thiết kế và tài liệu, không bằng tiền.

- **FR105.** Phát hành bản cài cho **macOS và Windows** qua **GitHub Releases**.
- **FR106.** Công bố **checksum SHA-256** cho mọi artifact phát hành.
- **FR107.** **Build công khai qua GitHub Actions**, để bất kỳ ai cũng kiểm chứng được binary khớp với mã nguồn.
- **FR108.** **Hướng dẫn cài đặt có ảnh chụp màn hình** cho cả hai hệ điều hành, xử lý tường minh Gatekeeper trên macOS *(chuột phải → Mở)* và SmartScreen trên Windows *(More info → Run anyway)*.
- **FR109.** **Màn hình Attribution trong ứng dụng:** liệt kê mọi nguồn từ điển, giấy phép tương ứng và ghi công đầy đủ.
- **FR110.** Kèm văn bản giấy phép **GPL v3** và toàn bộ giấy phép của các bộ dữ liệu trong bản phát hành.
- **FR111.** Cơ chế cập nhật **chỉ kiểm tra và thông báo** phiên bản mới. **Không tự động tải, không tự động cài.**
  > Một cơ chế tự cập nhật trên bản không ký số là đường tấn công thật — không có chữ ký thì không có gì xác minh được bản tải về là chính chủ.
- **FR112.** **Chính sách gỡ bỏ dữ liệu:** nếu chủ sở hữu một nguồn không xác định được tác giả lên tiếng, phải có quy trình gỡ lớp đó khỏi bản phát hành kế tiếp **mà không ảnh hưởng chức năng** — bảo đảm bởi FR36.

---

## Yêu cầu phi chức năng

Phần lớn ngưỡng dưới đây **không phải phỏng đoán** — chúng đến từ số đo thật của Giai đoạn 0.

### Hiệu năng

| ID | Yêu cầu | Ngưỡng | Căn cứ |
|---|---|---|---|
| **NFR1** | Độ trễ Auto-Lookup **đầu-cuối**: từ lúc thả chuột sau khi bôi đen tới lúc kết quả hiển thị ở Panel Lookup | **p95 < 100 ms** **[A1]** | Backend đo được p50 0,022 ms · p95 0,046 ms, payload 679 byte — backend chỉ tiêu 0,05 ms. **Toàn bộ phần còn lại (~99,95 ms) dành cho vòng IPC Tauri và render frontend** |
| **NFR2** | Auto-save không làm gián đoạn thao tác gõ | **Không frame nào vượt 50 ms** trong lúc auto-save chạy | Cụ thể hoá "không có gai trễ"; là điều kiện nghiệm thu cho R9 |
| **NFR3** | Tìm kiếm full-text toàn Library | **p95 < 500 ms** trên thư viện 5.000 Chương | **[A6]** Ngưỡng tạm, hiệu chỉnh ở Giai đoạn 3 |
| **NFR4** | Khởi động ứng dụng tới lúc Library dùng được | **< 3 giây** trên thư viện 5.000 Chương | **[A7]** Ngưỡng tạm |
| **NFR5** | Bộ nhớ khi nhàn rỗi | **< 300 MB** | **[A8]** Ngưỡng tạm. Baseline Tauri v2 là 20–100 MB; phần dôi dành cho chỉ mục và Tác phẩm đang mở |

> Vì sao vẫn đặt ngưỡng dù chưa đo được (NFR3–NFR5): **một ngưỡng tạm sai vẫn nghiệm thu được và vẫn buộc người xây dựng phải đo; một tính từ thì không.**

### Dữ liệu & lưu trữ

| ID | Yêu cầu | Ngưỡng |
|---|---|---|
| **NFR6** | Kích thước bản cài kèm toàn bộ từ điển | **[A2]** Ngân sách **150–200 MB**, **không có cơ chế tải thêm sau khi cài** *(đo được 130 MB với ba nguồn đầu tiên)* |
| **NFR7** | Tra cứu khi ngoại tuyến | **100%** hoạt động không cần mạng |
| **NFR8** | Độ chính xác dấu tiếng Việt | Chỉ mục tìm kiếm **chính** phải **phân biệt dấu**. Chế độ xoá dấu chỉ được tồn tại như chỉ mục **phụ**, không bao giờ là mặc định. Chi phí đã biết: ~17 MB mỗi chỉ mục |
| **NFR9** | Khả năng mang dữ liệu đi | TM xuất được TMX; Glossary và prompt xuất được định dạng văn bản mở; `.atproj` tự chứa và mở được trên máy khác |
| **NFR10** | Toàn vẹn dữ liệu | Mất chỉ mục Library **không được** làm mất dữ liệu — chỉ mục dựng lại được hoàn toàn từ `.atproj` |

### Bảo mật & quyền riêng tư

| ID | Yêu cầu |
|---|---|
| **NFR11** | API key lưu trong **keychain / credential manager của hệ điều hành**. Không bao giờ ghi vào file cấu hình, file dự án hay log |
| **NFR12** | **Không telemetry.** Không có luồng dữ liệu ra ngoài nào ngoài lời gọi AI do người dùng chủ động kích hoạt |
| **NFR13** | Không tài khoản, không đăng nhập, không đồng bộ đám mây |

### Nền tảng & giấy phép

| ID | Yêu cầu |
|---|---|
| **NFR14** | Chạy native trên **macOS và Windows**, hành vi tương đương trên cả hai |
| **NFR15** | **Mọi thư viện và crate được dùng phải tương thích GPL v3.** Rà soát tường minh trước khi đưa mỗi phụ thuộc mới vào dự án |
| **NFR16** | Ngôn ngữ giao diện v1: **chỉ tiếng Việt**. Nhưng **toàn bộ chuỗi giao diện phải nằm ngoài mã nguồn, trong file tài nguyên riêng, ngay từ dòng code đầu tiên** |

> **NFR16 giải thích:** người dùng mục tiêu là dịch giả Việt Nam, họ không cần bản tiếng Anh. Nhưng tách chuỗi ra file riêng **gần như không tốn gì nếu làm từ đầu và rất đắt nếu làm sau**. Với một dự án open source mời cộng đồng quốc tế đóng góp, giữ cửa này mở là quyết định rẻ.

### Khả năng tiếp cận & an toàn dữ liệu

*(NFR17–NFR18 mang số cuối dãy theo quy ước không đánh số lại — chúng đóng Q6 và Q7 ngày 2026-08-02.)*

| ID | Yêu cầu | Ngưỡng |
|---|---|---|
| **NFR17** | **Sàn khả năng tiếp cận** | Mọi thao tác của ứng dụng làm được **hoàn toàn bằng bàn phím** — nghiệm thu bằng một vòng dịch hoàn chỉnh một Chương không chạm chuột. Trạng thái focus luôn nhìn thấy rõ ở mọi panel và mọi chế độ. Tương phản văn bản đạt **WCAG AA** ở **cả hai** chế độ sáng và tối, kể cả Chế độ đọc và phần bôi màu diff của Review Mode |
| **NFR18** | **Cửa sổ mất dữ liệu tối đa khi ứng dụng sập** | **≤ 5 giây** công việc. Auto-save kích hoạt khi người dùng ngừng gõ khoảng 2 giây, kèm **trần thời gian 5 giây buộc ghi** dù đang gõ liên tục. Phải đạt được đồng thời với NFR2 — không frame nào vượt 50 ms |

> **NFR17 giải thích:** áp từ **Giai đoạn 1**, cùng lý do với NFR16. FR22 đã bắt mọi thao tác lặp lại phải có phím tắt cấu hình lại được, nên keyboard-first gần như miễn phí nếu làm từ đầu và rất đắt nếu vá sau. **Hỗ trợ trình đọc màn hình (ARIA đầy đủ, VoiceOver/NVDA) KHÔNG nằm trong v1** — editor theo segment trong webview là ca khó nhất của a11y và chưa có nhu cầu người dùng nào xác nhận. Đây là ranh giới có chủ ý, không phải thiếu sót.

> **NFR18 giải thích:** NFR2 nói auto-save không được gây gai trễ, nhưng không nói mất bao nhiêu là chấp nhận được — mà đó mới là thứ người dùng cảm nhận. Ngưỡng 5 giây có nghĩa mất nhiều nhất **một câu đang dở**. Với người dịch dành nửa buổi cho một Chương, 30 giây gõ đã là mất mát thật; còn ghi liên tục mỗi thay đổi thì đẩy thẳng vào rủi ro R9 và xung đột với NFR2.

---

## Chỉ số kết quả công việc

Khác với NFR (ngưỡng nghiệm thu kỹ thuật), ba chỉ số này đo **giá trị sản phẩm** và cần dữ liệu dùng thật.

| Chỉ số | Đo cái gì | Vì sao |
|---|---|---|
| **Tuân thủ Glossary** | Tỷ lệ thuật ngữ có trong Glossary được AI dùng đúng trong bản đề xuất | Đo trực tiếp hiệu lực của Smart RAG Injector |
| **Mật độ lỗi reviewer sửa** | Số lỗi reviewer sửa trên mỗi 1000 từ, theo thời gian | Đo "chất lượng bản dịch tốt hơn" bằng bằng chứng bên ngoài |
| **Consistency drift** | Số biến thể tên gọi khác nhau của cùng một thực thể trong một Tác phẩm | Là lý do tồn tại của Glossary + TM |

### Counter-metrics — dấu hiệu đã đi sai hướng

| Counter-metric | Ngưỡng cảnh báo | Nếu tăng thì nghĩa là |
|---|---|---|
| **Tỷ lệ chấp nhận thẳng bản dịch AI không sửa** | Cao và tăng dần | Công cụ đã trượt về phía auto-translate — **phản bội trụ "người dịch là trung tâm"**. Người dùng không còn biên tập, chỉ còn duyệt |
| **Thời gian quản lý công cụ thay vì dịch** | Cao | Library và Glossary đã trở thành gánh nhập liệu thay vì hạ tầng tự tích luỹ |

> Hai counter-metric này **không có ngưỡng số cứng ở v1** vì chưa có baseline (Q5). Chúng là **tín hiệu định hướng cho quyết định thiết kế**, không phải cổng nghiệm thu.

Sản phẩm này **không cạnh tranh ở tốc độ và không đặt mục tiêu thời gian nào**. Nhanh hơn là hệ quả, không phải mục tiêu. *(Mốc tham chiếu hiện tại: một chương mất nửa buổi đến trọn một ngày.)*

---

## Ghi chú bàn giao cho `bmad-ux`

SPEC này **không có User Journey, và đó là lựa chọn có chủ ý** — AuraTranslate là công cụ chuyên nghiệp một người vận hành, nên hình dạng đúng là *capability spec*. Hệ quả: `bmad-ux` phải tự dựng hành trình người dùng từ FR, đặc biệt cho ba luồng có tính trải nghiệm cao:

1. **Nhập một Tác phẩm 2000 chương** — `FR13 → FR14 → FR52 → FR53`. Luồng có nhiều chỗ dễ hỏng nhất.
2. **Một vòng dịch hoàn chỉnh một Chương** — `FR12 → FR21 → FR58 → FR70 → FR24 → FR5`.
3. **Nhận bản review về và hấp thụ bài học** — `FR90 → FR91 → FR92 → FR54`.
