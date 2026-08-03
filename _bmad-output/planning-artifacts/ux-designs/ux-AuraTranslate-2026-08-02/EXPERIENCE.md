---
name: AuraTranslate
status: final
created: 2026-08-02
updated: 2026-08-03
design: DESIGN.md
sources:
  - _bmad-output/specs/spec-AuraTranslate/SPEC.md
  - _bmad-output/specs/spec-AuraTranslate/requirements.md
  - _bmad-output/specs/spec-AuraTranslate/glossary.md
  - _bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md
  - _bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md
---

> Từ vựng dùng trong tài liệu này là từ vựng chính thức ở `spec-AuraTranslate/glossary.md`. Tác phẩm · Chương · Segment · Library · Workspace · Chế độ đọc · Review Mode · Panel Lookup · Auto-Lookup · Glossary · Translation Memory · Concordance · Smart RAG Injector. Không dùng biến thể.

## Foundation

Desktop native macOS và Windows, **một cửa sổ hệ điều hành duy nhất** (AD-24). Bên trong là ba chế độ: **Library**, **Workspace**, **Chế độ đọc**. Review Mode không phải chế độ thứ tư mà là một bố cục của Workspace.

Hệ bố cục là **dockview-vue**: dock, undock, gộp tab, đổi kích thước và preset đều là năng lực sẵn có của nó, không tự viết lại. Nhận diện thị giác ở `DESIGN.md`; tài liệu này chỉ nói hành vi.

Frontend **không chứa quy tắc nghiệp vụ** (AD-1). Tách câu, khớp ngôn ngữ, phân giải scope đều nằm ở Rust. Ngoại lệ tường minh duy nhất: văn bản đang gõ trong Editor là state cục bộ, đẩy xuống Rust theo hợp đồng flush của AD-35.

Giao diện **chỉ tiếng Việt** ở v1, nhưng không chuỗi nào nằm trong mã — tất cả ở `i18n/vi.json` theo khoá chấm (NFR16, AD-21).

## Information Architecture

```
Cửa sổ ứng dụng
├── Library ................. điểm vào, mở app là thấy cái này
│   ├── lưới Tác phẩm — bìa, tên, tiến độ, trạng thái
│   ├── tìm kiếm xuyên thư viện — hai chế độ dấu
│   ├── bộ lọc — trạng thái · lĩnh vực · ngôn ngữ nguồn · ngày sửa
│   ├── nhập tài liệu → màn xem trước hợp nhất
│   │     ├── từ file · dán văn bản · file song ngữ hai cột
│   │     └── từ URL — dán danh sách link, mỗi dòng một Chương
│   └── xuất → chọn cách xuất ảnh · khối ghi nguồn
├── Workspace ............... mở một Chương là vào đây
│   ├── preset 2×2 (mặc định)   Nguyên văn | Bản dịch
│   │                           Tra cứu    | Đề xuất AI
│   ├── preset 4 cột            Nguyên văn | Tra cứu | Đề xuất AI | Bản dịch
│   ├── preset Review Mode      Bản dịch của tôi | Bản Reviewer đã sửa
│   └── bảng chờ Glossary — duyệt ứng viên hàng loạt
└── Chế độ đọc .............. đọc lại thành quả, không có công cụ biên tập
    └── đọc liên tục qua nhiều Chương · công tắc song ngữ
```

Ba chế độ là **ngang hàng**, không phân cấp. Chuyển giữa chúng bằng `⌘1` `⌘2` `⌘3` hoặc tab ở thanh tiêu đề. Chuyển chế độ **luôn giữ ngữ cảnh**: rời Workspace sang Chế độ đọc rồi quay lại thì vẫn đúng Chương, đúng câu, đúng vị trí cuộn (FR12).

## Voice and Tone

Viết cho **một người dịch chuyên nghiệp**, không phải cho người dùng phổ thông cần dỗ dành.

- **Nói việc, không nói cảm xúc.** "Đã gộp hai câu." chứ không phải "Tuyệt vời! Đã gộp xong 🎉".
- **Nêu hệ quả, không chỉ nêu sự kiện.** "Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được." Người dùng cần biết sổ sách của mình ra sao.
- **Không đổ lỗi người dùng.** Lỗi API là "Nhà cung cấp không phản hồi", không phải "Bạn đã nhập sai khoá".
- **Số liệu là số liệu.** "412 token · ước tính ~0,004 USD" — không làm tròn thành "một chút chi phí".
- **Xưng hô:** không xưng "chúng tôi", không gọi người dùng là "bạn" trong thông báo trạng thái. Câu trạng thái viết ở dạng vô nhân xưng.

Mọi chuỗi ở trên là khoá trong `vi.json`. Không chuỗi hiển thị nào được sinh ở Rust (AD-21) — Rust trả `{ code, message_key, params, retryable }`.

## Component Patterns

**Panel** — mỗi panel ẩn được hoàn toàn (FR17); ẩn panel Đề xuất AI là ca sử dụng thật của người dịch không dùng AI, không phải trường hợp biên. Preset bố cục lưu và chuyển bằng phím (FR18).

**Bản ghi từ điển** — bản ghi có cấu trúc, không phải đoạn văn (FR28). Mỗi nguồn là một khối riêng xếp chồng dọc. **Không có trạng thái nào hợp nhất các nguồn** (AD-19). Khi hai nguồn ghi khác nhau, một dòng dẫn nói rõ điều đó trước khi liệt kê — người dùng biết mình đang nhìn bất đồng chứ không phải trùng lặp.

**Editor liền mạch** — trang văn bản liền, không ô không bảng. Trạng thái từng câu đọc ở vạch lề. Ranh giới câu ẩn, hiện khi con trỏ chạm hoặc rê chuột.

**Bảng chờ Glossary** — danh sách ứng viên xếp theo tần suất, mỗi dòng có số lần xuất hiện và một ví dụ ngữ cảnh. Duyệt và bỏ bằng **một phím**, không gõ (FR53). Không mục nào rời bảng chờ sang Glossary mà không qua thao tác của người dùng (FR55, AD-20).

**Dải mọc dưới câu đang sửa** — mẫu chung cho mọi thứ cần hỏi người dùng *về câu này*, thay cho hộp thoại: chốt bản dịch Glossary lần đầu gặp (FR114), phát hiện Proofreader (FR83), và gợi ý TM khớp mờ (FR59). Dải đẩy văn bản xuống chứ không phủ lên nó, và thu lại ngay khi xong.

**Chỉ một dải mọc tại một thời điểm** *(quyết định 2026-08-03)*. Một câu có thể kích hoạt cả ba. Nguyên tắc xếp thứ tự: **cái nào chặn thì thắng, cái nào chỉ gợi ý thì nhường.**

1. **Chốt Glossary.** Chặn thật — thuật ngữ chưa chốt không tham gia ép AI (FR70), nên để treo là để một lỗ hổng chạy tiếp qua mọi câu sau. Hỏi **một lần trong cả Tác phẩm**.
2. **Proofreader.** Không chặn hệ thống, nhưng **đang chờ một quyết định về chính câu này**, và bỏ qua thì nó **tích lại**.
3. **Gợi ý TM.** Nhường cả hai. Bỏ qua chỉ tốn công gõ lại một câu, và câu đó vẫn nằm nguyên trong TM cho lần sau.

Xử lý xong dải trên thì dải dưới mọc ngay tại chỗ vừa thu — không thao tác nào, và vị trí không nhảy vì cả ba cùng chiều cao đầu mục.

> **Va chạm này không có trong PRD.** `FR59`, `FR83` và `FR114` được viết độc lập, không FR nào nhắc tới FR kia; nó chỉ lộ ra khi cả ba dùng chung một mẫu thị giác trên cùng một dòng văn bản. Đây là **quyết định ở tầng thiết kế** — sai thì sửa ở đây, không phải mở lại PRD.

**Màn xem trước hợp nhất — một màn hình, ba tầng, không phải ba hộp thoại** *(quyết định 2026-08-03)*. Bảng mã (FR126), ranh giới bóc (FR123) và luật làm sạch (FR124) mô tả **cùng một văn bản ở cùng một thời điểm**; AD-39 đã chốt chúng là một pipeline thứ tự cố định và *"xem trước hiển thị kết quả sau toàn bộ chuỗi"*. Đặc tả riêng ở mục **Đường nhập** bên dưới.

**Khối nội dung — đơn vị của mọi thao tác trong màn xem trước.** Nội dung chia thành khối theo đoạn. Mỗi khối mang đúng một trong ba trạng thái, đọc ở vạch lề như segment trong Editor:

| Vạch | Nghĩa |
|---|---|
| `confirmed` | giữ lại, người dùng đã chạm tay |
| `tm-rule` | giữ lại, **máy đoán, chưa ai xác nhận** |
| `ornament` mờ | đã loại — khối chìm xuống `surface-sunken`, chữ rút về `on-surface-variant` |

**Phân biệt giữ lại với loại bỏ bằng độ lùi, không bằng màu nhấn thứ hai.** Khối bị loại đổi cả kiểu chữ — từ chữ đọc `source-cjk` sang chữ giao diện cỡ nhỏ — nên mắt biết ngay nó không còn là nội dung. Hệ thống này vốn phân biệt bằng độ lùi (`ornament`, `surface-sunken`, `outline-faint`) chứ không bằng sắc màu; đây là dùng lại ngữ pháp sẵn có. Xanh mực `primary` vẫn chỉ dành cho ba việc cũ.

**Ranh giới do máy đoán dùng `tm-rule`, và đó không phải màu mới.** Trong toàn ứng dụng `tm-rule` đã luôn nghĩa là *máy đề xuất, chưa ai xác nhận* — đúng y nghĩa của một ranh giới thuật toán vừa đặt. Người dùng chạm vào thì nó thành `confirmed`, cùng ngữ pháp với trạng thái segment.

**Phát hiện Proofreader — gạch chân, không phải vạch lề** *(quyết định 2026-08-03)*. `FR86` bắt đánh dấu tại chỗ trên Editor, nhưng vạch lề đã dùng hết năm giá trị cho trạng thái segment và Editor cố tình không có ô. Lời giải: phát hiện là **gạch chân lượn sóng dưới đúng cụm chữ có vấn đề**, ở `text-underline-offset: 4px` để không chạm dấu nằm dưới của `ạ` `ộ` `ợ`. Hai lớp thông tin, hai chỗ đọc — vạch lề nói *trạng thái câu*, gạch chân nói *chỗ nghi ngờ*.

Hai màu, không thêm màu mới nào vào bảng: **`error`** cho chính tả và ngữ pháp (`FR80` — có đáp án đúng), **`tm-rule`** cho nghi về nghĩa (`FR81` — là phán đoán, và màu này trong toàn ứng dụng đã luôn nghĩa là *máy đề xuất, chưa ai xác nhận*).

## State Patterns

**Trạng thái segment** — đọc ở vạch lề, năm giá trị:

| Vạch | Nghĩa | Sinh ra từ |
|---|---|---|
| `confirmed` | đã xác nhận | người dùng xác nhận (FR24) |
| `primary` | đang sửa, con trỏ ở đây | tiêu điểm |
| `tm-rule` | điền sẵn từ TM khớp 100%, **chưa** xác nhận | FR58 |
| không vạch | chưa dịch | mặc định khi nhập |
| `ornament` mờ | đã về hưu do gộp/tách | AD-5 |

Hệ thống **không bao giờ tự coi một câu là xong**. Khớp TM 100% vẫn phải người xác nhận.

**Trạng thái tra cứu** — có kết quả · không có kết quả · nhiều nguồn bất đồng. Không có trạng thái "đang tải": tra cứu chạy dưới 100ms đầu-cuối (NFR1), spinner ở đây là tiếng ồn.

**Trạng thái AI** — chưa cấu hình · đang sinh (chảy dần) · xong · lỗi · đã huỷ. **Chưa cấu hình không phải trạng thái lỗi** — ứng dụng chạy đầy đủ không cần AI (FR77), nên panel này chỉ mời cấu hình, không cảnh báo.

**Lỗi mạng và lỗi API** — nêu rõ nguyên nhân, giữ nguyên công việc đang làm, và **chỉ thử lại khi người dùng bấm** (FR75). Không tự động thử lại: với BYOK mỗi lần gọi là tiền của người dùng.

**Trạng thái bảng mã** *(2026-08-03)* — ba giá trị: **nguồn tự khai** (`.docx`, hoặc HTTP có `charset` tin được) · **tự đoán, tin cậy cao** · **tự đoán, tin cậy thấp**. Chỉ giá trị thứ ba mở dải đối chiếu năm ứng viên. Không có trạng thái lỗi: một file đọc sai bảng mã **không hỏng**, nó chỉ ra chữ không đọc được — và đó là thứ mắt phân xử, không phải thứ hệ thống phán quyết.

**Trạng thái một Chương trong lần nhập nhiều link** — **sạch** hoặc **cần xem**. *Cần xem* gom bốn nguyên nhân: bảng mã tin cậy thấp · phần bóc ra ngắn bất thường so với trung vị các Chương khác · luật làm sạch xoá quá nhiều · link hỏng. Bộ đếm ở đầu màn xem trước luôn hiện cả hai con số.

**Đã lưu** — thanh trạng thái ghi "Đã lưu N giây trước". Không hộp thoại, không dấu chấm "chưa lưu" gây lo lắng: NFR18 bảo đảm mất tối đa 5 giây.

## Interaction Primitives

**Auto-Lookup** — bôi đen ở bất kỳ panel nào (Nguyên văn, Đề xuất AI, hoặc Bản dịch) thì kết quả hiện ngay ở Panel Lookup. Không copy, không paste, không hộp thoại. Đây là tương tác lặp nhiều nhất trong sản phẩm và là thứ cộng đồng đã quen ở QuickTranslator — **không được thiết kế lại cho khác đi**.

**Gộp và tách câu** — `⌘M` gộp, `⌘/` tách. Cả hai là command đăng ký, không phải hệ quả phụ của việc gõ.

**Gộp ngầm** — gõ đè lên đúng vị trí ranh giới **là** ra lệnh gộp. Hệ thống thực hiện đúng ngữ nghĩa AD-5: hai câu cũ về hưu và vẫn tra lại được lịch sử, câu mới bắt đầu ở trạng thái chưa xác nhận với lịch sử rỗng. Một dòng báo ở lề, hoàn tác bằng `⌘Z`. **Không chặn, không hỏi lại** — chặn lại sẽ phá đúng cảm giác tự do mà Editor liền mạch tồn tại để có.

**Đưa bản dịch AI sang Editor** — `⌘⇧↵`, luôn do người dùng chủ động (FR72). Không có đường nào để kết quả AI tự chảy vào Bản dịch.

**Sửa ranh giới bóc — bàn phím là đường chính** *(2026-08-03)*. Bản năng thiết kế cho việc chọn vùng văn bản là **kéo chuột**, và đó là chỗ màn xem trước dễ thủng NFR17 nhất. Mô hình thao tác đi theo khối, không theo con trỏ ký tự:

| Phím | Việc |
|---|---|
| `J` `K` hoặc `↑` `↓` | đi giữa các khối |
| `Space` | bật/tắt giữ khối đang chọn |
| `[` `]` | đặt đầu và cuối vùng giữ một lần |
| `E` | mở bộ chọn bảng mã |
| `R` | bật/tắt luật làm sạch đang khớp khối này |
| `⌥←` `⌥→` | Chương trước / sau trong cùng lần nhập |
| `⌥W` | chỉ xem các Chương **cần xem** |
| `⌘↵` | xác nhận nhập |

Kéo chuột vẫn dùng được, nhưng nó là **đường thứ hai**. Mọi phím trên là command đăng ký trong `CommandRegistry` như mọi thao tác khác (AD-34).

**Sync Scrolling** — đồng bộ cuộn giữa Nguyên văn, Đề xuất AI và Bản dịch, có công tắc rõ ràng (FR20). Đây là lý do preset 2×2 đặt Nguyên văn và Bản dịch **cạnh nhau**: đối chiếu theo chiều ngang là thao tác lặp hàng trăm lần mỗi Chương.

## Accessibility Floor

Đây là hợp đồng cấu trúc, không phải danh sách mong muốn — cưỡng chế bởi AD-34.

1. **Mọi thao tác đi qua `CommandRegistry`.** Handler chuột chỉ được `dispatch` một command đã đăng ký. Nhờ vậy "thao tác nào chưa gán được phím" là câu hỏi liệt kê được tự động. Command id dùng khoá chấm có tiền tố miền, cùng hình dạng khoá `vi.json`.
2. **Mỗi chế độ và mỗi panel khai báo điểm vào focus.** Chuyển panel phải dời focus DOM tường minh. Không chế độ nào để focus rơi về `body`.
3. **Tiêu điểm luôn nhìn thấy** — vạch dọc `primary` ở mép trái panel cộng tiêu đề đậm màu.
4. **Màu chỉ đến từ token đã kiểm tương phản** ở cả hai theme. Xem bảng sàn tương phản trong `DESIGN.md` — ba màu đã bị loại vì trượt AA.

**Tiêu chí nghiệm thu (NFR17):** dịch trọn một Chương từ đầu tới cuối — mở từ Library, tra cứu, gọi AI, đưa sang, sửa, gộp một câu, xác nhận, sang Chương kế — **không chạm chuột một lần nào**.

**Ngoài phạm vi v1:** hỗ trợ trình đọc màn hình (ARIA đầy đủ, VoiceOver/NVDA). Ranh giới có chủ ý, ghi ở PRD §3.2 — không phải thiếu sót cần vá lén.

## Đường nhập — màn xem trước hợp nhất

*(Mục bổ sung 2026-08-03 cho FR122–FR128. Đây là bề mặt **đầu tiên** người dùng chạm vào sản phẩm, và là nơi hai lỗi đắt nhất của ứng dụng có thể xảy ra im lặng.)*

### Thứ tự nhìn đi theo quan hệ nhân quả

Ba tầng nằm trên **một** màn hình, xếp dọc theo đúng thứ tự chúng phụ thuộc nhau:

| Tầng | Việc | FR |
|---|---|---|
| **1** | Bảng mã | FR126 |
| **2** | Ranh giới nội dung | FR123 |
| **3** | Luật làm sạch | FR124 |

**Bảng mã đứng trên cùng vì bảng mã sai thì mọi thứ dưới nó vô nghĩa** — bóc nội dung trên chữ rác, luật khớp vào chữ rác. Đặt nó xuống dưới hay giấu vào Cài đặt sẽ khiến người dùng ngồi sửa ranh giới trên một văn bản đã hỏng. Chuẩn hoá xuống dòng và khoảng trắng (FR125) không có tầng riêng: nó chạy ngầm và kết quả là thứ ba tầng trên đang hiển thị.

Không có nút *"Tiếp theo"* giữa các tầng. Đổi bảng mã ở tầng 1 thì tầng 2 và 3 dựng lại **ngay, trong bộ nhớ** — chưa segment nào tồn tại nên không có gì phải cho về hưu (AD-39). Đây chính là cách thoả điều kiện nghiệm thu *"sửa được mà không phải nhập lại từ đầu"* của FR126.

### Lỗi bảng mã tự lộ ra bằng mắt, không bằng cảnh báo

Khi độ tin cậy dò thấp, một dải mở ra **ngay trên văn bản** với năm ứng viên, mỗi ứng viên kèm **bản dựng thật** của cùng một đoạn 6–8 ký tự đầu Chương:

```
UTF-8      GB18030     GBK        Big5       UTF-16
ç¬¬ä¸€ç«   第一章 雪   第一章 雪   姼銝€蝡    ⽦Ɫ畜
không ra   ✓ đang      cũng ra    không ra   không ra
chữ        chọn        chữ        chữ        chữ
```

**Người dịch Việt nhận ra `第一章` với `ç¬¬ä¸€ç«` trong một phần giây** — nhanh hơn và chắc chắn hơn mọi câu cảnh báo mô tả vấn đề. Đây là lý do mẫu chữ đặt ở cỡ `read` chứ không cỡ giao diện: phải đủ lớn để phân biệt được nét chữ Hán.

> **Vì sao không dùng hộp thoại cảnh báo.** FR126 cùng hạng lỗi *thất bại im lặng* với FR39 — một cảnh báo chỉ nói *"có thể sai"* thì người dùng vẫn không biết chọn gì. Đối chiếu năm bản dựng biến một phán đoán kỹ thuật thành một **câu hỏi thị giác** mà người dùng trả lời được ngay.

### Ranh giới cứng của FR122 phải đếm được

Dưới ô dán link, hai con số đứng cạnh nhau: **`N` link · sẽ tạo `N` Chương**, kèm một câu — *"Chỉ tải đúng N link này. Không tìm thêm link nào khác."*

Hai con số **bằng nhau là bằng chứng đọc được** rằng ứng dụng không tự đi tìm gì. Nếu về sau có ai thêm tính năng quét mục lục, hai số này lệch và màn hình tự tố cáo. Mạnh hơn một câu cam kết viết trong tài liệu hướng dẫn.

### Bộ lọc "cần xem" là thứ giữ cho màn hình dùng được ở quy mô thật

Dán 50 link mà bắt duyệt tay 50 màn xem trước thì tới lần thứ mười người dùng sẽ bấm xác nhận mù — và mục đích của cả màn hình này mất sạch. Đầu màn hình luôn hiện **hai con số**: *`N` Chương cần xem* và *`M` Chương sạch*, `⌥W` lọc về nhóm đầu.

### Luật làm sạch hiện thứ sắp bị xoá, trước khi xoá

Chỗ bị luật xoá hiện **gạch ngang tại chỗ trong văn bản** bằng nét `ornament`, kèm nhãn luật đã khớp. Danh sách luật ở tầng 3 cho bật/tắt từng luật và ghi **hai con số**: khớp bao nhiêu chỗ *trong Chương này* và bao nhiêu chỗ *trong cả lần nhập*.

Mỗi luật mang nhãn tầng — **Toàn cục** hoặc **Tác phẩm** — và **cả hai tầng cùng áp** (ngữ nghĩa hợp nhất, AD-18). Đây là loại làm sạch duy nhất có thể **xoá nhầm nội dung thật**, nên nó tuân đúng nguyên tắc *máy đề xuất, người duyệt* của FR55: không luật nào chạy mà người dùng không nhìn thấy nó vừa làm gì.

### Xuất xứ tài liệu tự điền, sửa tại chỗ

Bốn trường của FR128 — tác giả · báo/website · URL bài gốc · ngày đăng — gom **một khối ở đầu mỗi Chương** trong màn xem trước, tự điền từ trang. Trường không tìm thấy hiện chữ nghiêng *"không tìm thấy"* thay vì để trống, để người dùng biết hệ thống **đã tìm** chứ không phải quên.

Với Tác phẩm nhập từ file hay dán tay, cùng khối đó mở được từ danh sách Chương. Bốn trường ở **tầng Chương** chứ không tầng Tác phẩm — truyện web mỗi Chương một link riêng.

### Nhật ký domain (NFR19)

Hai chỗ: một dòng tóm tắt ở chân màn xem trước — *"Đã gọi `N` domain · xem"* — và bảng đầy đủ trong **Cài đặt › Quyền riêng tư**.

Hai tầng allowlist của AD-41 phân biệt bằng **nhãn chữ**, không bằng màu: `Tài liệu` cho host người dùng dán link, `Ảnh` cho host mà những trang đó tham chiếu tới. Mỗi hàng ghi **vì sao được phép** — đó là thứ biến lời hứa *"không telemetry"* thành thứ người dùng kiểm được.

## Chế độ đọc — đặc tả typography

PRD bàn giao mục này có chủ ý. Bound đã chốt:

| Mức | Chiều rộng | Cỡ chữ | Giãn dòng |
|---|---|---|---|
| Thoáng | 62ch | 19px | 1.95 |
| **Cân (mặc định)** | **68ch** | **17,5px** | **1.8** |
| Đặc | 76ch | 16px | 1.66 |

**Giãn dòng 1.66 là sàn cứng** — dấu tiếng Việt chồng cả trên lẫn dưới, dưới mức đó `ườ` chạm `ộ`. Lý do đầy đủ ở `DESIGN.md`.

Điều khiển hai tầng: **ba preset trên thanh công cụ**, thanh trượt cỡ chữ và giãn dòng chi tiết **sau một lần bấm**. Người dịch chỉnh một lần rồi dùng mãi — nhưng FR11 bắt buộc chỉnh được nên không được giấu hẳn.

Mặc định **chỉ hiển thị bản dịch tiếng Việt**; công tắc song ngữ đặt nguyên văn **ở lề trái**, cỡ nhỏ, màu `on-surface-variant` — **không chen giữa dòng đọc**, vì một khối chữ Hán trên mỗi đoạn làm gãy nhịp đọc tiếng Việt. Hình nhúng hiển thị **đúng vị trí trong văn bản** (FR43). **Chú thích hiện dưới ảnh là `caption` đã dịch (FR129) — không phải alt-text.** Alt-text (FR44) cũng được dịch nhưng **không hiện trên trang**: nó là thứ trình đọc màn hình đọc lên, không phải thứ mắt nhìn. Ảnh không có caption thì **không chừa chỗ trống** dưới ảnh. Cuối trang là chuyển Chương liền mạch — đọc nhiều Chương không phải quay về Library.

> **Sửa 2026-08-03.** Câu cũ viết *"chú thích là alt-text đã dịch"* — gộp hai thứ làm một. Vô hại khi mọi ảnh đến từ `.docx` của người dùng; **sai ngay khi nhập bài báo từ web**, nơi caption ghi bối cảnh và nguồn ảnh còn alt-text mô tả cái đang có trong ảnh. AD-42 chốt cả hai là `Segment` mang trường **vai**, và `ASSET` mang **neo vị trí riêng** — vì ảnh web thường không có `alt` nào để giữ chỗ.

Chiều rộng đo bằng `ch` chứ không bằng `px`, nên số ký tự mỗi dòng giữ nguyên khi đổi cỡ chữ.

## Key Flows

### KF-1 · Ice nhập một bộ truyện 2000 chương

Luồng PRD đánh dấu **nhiều chỗ dễ hỏng nhất**. `FR13 → FR14 → FR52 → FR53`

1. Ice có một file `.txt` 40MB tải từ diễn đàn, toàn bộ 2000 chương trong một file. Từ Library, chọn **Nhập tài liệu**.
2. Chọn file. Hệ thống nhận ra đây là file lớn và mời **tách thành nhiều Chương** thay vì tạo một Chương khổng lồ.
3. Ice nhập mẫu phân tách — mẫu tiêu đề hoặc biểu thức chính quy. Màn hình **xem trước** hiện ngay: đã nhận ra bao nhiêu Chương, ba Chương đầu và ba Chương cuối trông thế nào, và **những chỗ mẫu không khớp**.
4. **Nhịp then chốt:** Ice thấy mẫu bắt nhầm 14 chỗ — có dòng "Chương trình luyện khí" bị nhận là tiêu đề chương. Ice sửa mẫu, xem trước cập nhật, xác nhận. *Không có bước nào ghi xuống đĩa trước khi Ice xác nhận màn hình này.*
5. Nhập chạy. 2000 Chương vào Library ở trạng thái **Chưa bắt đầu** — không phải *Tạm ngưng*, vì *Tạm ngưng* nghĩa là đã làm dở rồi bỏ (FR5).
6. Quét ứng viên Glossary chạy nền, kết quả vào **bảng chờ**. Ice mở bảng chờ: 340 ứng viên xếp theo tần suất, mỗi dòng có số lần xuất hiện và ví dụ ngữ cảnh.
7. Ice duyệt bằng một phím mỗi mục — nhận, bỏ, nhận, bỏ. Không gõ chữ nào. Sau mười phút, 90 tên riêng đã vào Glossary Tác phẩm với xuất xứ *"đề xuất khi nhập tài liệu"*.

**Chỗ dễ hỏng nhất:** bước 4. Nếu xem trước không hiện chỗ không khớp, Ice phát hiện ra 14 chương sai sau khi đã dịch 200 chương.

### KF-2 · Một vòng dịch trọn một Chương

`FR12 → FR21 → FR58 → FR70 → FR24 → FR5`

1. Từ Library, Ice mở Chương 47. Vào Workspace, **đúng câu đang dở lần trước**, cuộn đúng chỗ.
2. Câu đầu đã có sẵn bản dịch với vạch lề nâu vàng — TM khớp 100% từ một Chương trước. Ice đọc, thấy đúng, bấm xác nhận. Vạch chuyển xanh ô liu.
3. Câu tiếp có chữ `走廊` Ice không chắc. Ice bôi đen. Panel Lookup hiện ngay — **ba nguồn, và một dòng dẫn báo CVDICT với Thiều Chửu ghi khác nhau**. Ice đọc cả hai, chọn "hành lang".
4. Ice bấm gọi AI. Kết quả chảy dần ở panel Đề xuất, kèm dòng "Đã chèn 2 thuật ngữ Glossary và 3 câu TM tương tự". Ice mở **Xem prompt** để kiểm — đúng là tên nhân vật đã chốt được đưa vào.
5. **Nhịp then chốt:** bản dịch AI dùng đúng thuật ngữ nhưng ngắt câu theo kiểu Trung. Ice bấm `⌘⇧↵` đưa sang Bản dịch rồi **viết lại thành một câu Việt gộp hai câu Trung** — gõ tự do, xoá luôn ranh giới. Hệ thống gộp ngầm, báo một dòng ở lề: *"Đã gộp hai câu. Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được."*
6. Ice xác nhận. Cặp TM mới được ghi ngay tại chuyển tiếp đó (AD-31).
7. Hết Chương, trạng thái Chương tự chuyển **Đã xong**; tiến độ Tác phẩm ở Library cập nhật theo.

**Nhịp then chốt là bước 5** vì đó là chỗ toàn bộ sản phẩm được quyết: người dịch tự do như đang viết, mà sổ sách segment vẫn sạch.

### KF-3 · Nhận bản review về và hấp thụ bài học

`FR90 → FR91 → FR92 → FR54`

1. Reviewer trả về `chuong-40-50.docx` đã sửa. Reviewer **không cài AuraTranslate** — file là cầu nối duy nhất.
2. Ice nhập file vào đúng Tác phẩm. Hệ thống khớp cấu trúc đoạn với dữ liệu sẵn có.
3. **7 đoạn không khớp được** — reviewer đã gộp vài đoạn. Chúng hiện ra thành danh sách để Ice nối tay. *Máy khớp, người sửa* — không có khớp im lặng.
4. Workspace chuyển sang **Review Mode**: hai cửa sổ side-by-side, nguyên văn ẩn đi, chỉ còn bản của Ice và bản reviewer, bôi màu phần thêm/xoá/sửa.
5. Ice lướt, chấp nhận từng thay đổi mình đồng ý.
6. **Nhịp then chốt:** hệ thống báo *"Reviewer đổi «Bắc Lương vương» thành «vương Bắc Lương» ở 23/24 lần xuất hiện. Thêm vào Glossary?"* — và **báo này xuất hiện dù Ice có mở Review Mode hay không** (FR95). Ice nhận. Từ chương sau, Smart RAG Injector ép AI dùng đúng cách gọi đó.

**Vì sao bước 6 tách rời khỏi bước 4-5:** nguyên nhân gốc của việc người dịch không xem lại bản review vẫn chưa xác định (Q1). Nếu Ice không bao giờ mở Review Mode, công sức của reviewer **vẫn** phải chuyển hoá thành giá trị.

### KF-4 · Ice dán 50 link và gặp một file mã GBK

`FR122 → FR126 → FR123 → FR124 → FR128`

*(Hành trình bổ sung 2026-08-03. Đây là **lần đầu tiên** Ice chạm vào sản phẩm với một bộ truyện mới — nên nó cũng là chỗ ấn tượng đầu tiên được quyết.)*

1. Ice mở một trang truyện, copy 50 link chương vào clipboard. Từ Library, chọn **Nhập từ website**, dán.
2. Dưới ô dán hiện hai con số: **50 link · sẽ tạo 50 Chương**, kèm câu *"Chỉ tải đúng 50 link này. Không tìm thêm link nào khác."* Ice liếc qua, thấy hai số bằng nhau, bấm **Tải**.
3. Màn xem trước mở ra ở Chương 1. Đầu màn hình: **7 Chương cần xem · 43 Chương sạch**.
4. **Nhịp then chốt.** Ice bấm `⌥W` để lọc về nhóm cần xem, và Chương 3 hiện ra với chữ `ç¬¬ä¸€ç«`. Ngay trên văn bản là một dải năm ô — cùng đoạn đầu Chương dựng bằng năm bảng mã. Ice **không đọc chữ nào**; mắt chạm ô thứ hai thấy `第一章 雪`, bấm. Cả màn hình dựng lại. Chữ Hán hiện đúng ở cả ba tầng.
5. Ở tầng 2, thuật toán bóc đã cắt mất đoạn cuối Chương. Ice bấm `K` xuống khối cuối, `Space` — khối chuyển từ *đã loại* sang *giữ*, vạch lề đổi từ `ornament` sang `confirmed`.
6. Ở tầng 3, một luật đang xoá `【本站首发】` ở 91 chỗ trong cả 50 Chương. Ice nhìn hai con số, để nguyên. Một luật khác định xoá `求推荐票` — Ice tắt nó vì có Chương dùng cụm đó trong lời thoại thật.
7. Ice bấm `⌘↵`. 50 Chương vào Library ở trạng thái **Chưa bắt đầu**, mỗi Chương mang sẵn tác giả và URL bài gốc.
8. Chân màn hình ghi *"Đã gọi 2 domain · xem"*. Ice bấm vào, thấy `truyen-example.com` nhãn **Tài liệu** và `img-cdn-example.net` nhãn **Ảnh** — và một dòng giải thích vì sao domain thứ hai được phép.

**Vì sao bước 4 là climax chứ không phải bước 7:** đây là khoảnh khắc một lỗi vốn **không báo gì cả** trở thành một câu hỏi trả lời được trong một giây. Không có dải đối chiếu, Ice sẽ thấy *"tách được 1 chương"* ở bước 7 và không đời nào đoán ra nguyên nhân nằm ở bảng mã.

## Mockups

Bản dựng là minh hoạ của spine, không phải nguồn sự thật — **khi mâu thuẫn, `DESIGN.md` và `EXPERIENCE.md` thắng.** Toàn bộ dùng token đã kiểm tương phản.

| Bề mặt | File | Neo tới |
|---|---|---|
| Workspace 2×2 nền sáng, Editor liền mạch | [`mockups/key-screen-workspace.html`](mockups/key-screen-workspace.html) | KF-2 |
| **Bảng chờ Glossary** — duyệt 340 ứng viên một phím | [`mockups/glossary-queue.html`](mockups/glossary-queue.html) | KF-1 · FR52–FR55 |
| **Chốt bản dịch lần đầu gặp** — dải mọc, không phải hộp thoại | [`mockups/glossary-confirm-inline.html`](mockups/glossary-confirm-inline.html) | FR114 · FR50 |
| **Xem prompt đã gửi** — kèm phần bị loại và lý do | [`mockups/prompt-inspector.html`](mockups/prompt-inspector.html) | FR71 · AD-14 |
| Workspace 2×2 **nền tối** + so sánh cơ chế phân tách | [`mockups/workspace-dark.html`](mockups/workspace-dark.html) | Elevation & Depth |
| Panel Lookup ở **mật độ thật** (打 · 22 nghĩa · 5 nguồn), sáng và tối | [`mockups/lookup-real-density.html`](mockups/lookup-real-density.html) | Component Patterns |
| **Chuyển động** Auto-Lookup — trang tự chạy | [`mockups/motion-auto-lookup.html`](mockups/motion-auto-lookup.html) | DESIGN.md § Motion |
| Library + màn hình xem trước kết quả tách | [`mockups/library-and-import.html`](mockups/library-and-import.html) | KF-1 |
| **Nhập tài liệu song ngữ** + nối câu lệch | [`mockups/bilingual-import.html`](mockups/bilingual-import.html) | FR115 · FR116 · FR117 |
| Nối đoạn · Review Mode diff · thu hoạch thuật ngữ | [`mockups/review-mode.html`](mockups/review-mode.html) | KF-3 |
| Trạng thái rỗng và trạng thái lỗi | [`mockups/empty-states.html`](mockups/empty-states.html) | State Patterns |
| **Chế độ đọc trọn vẹn** — đánh dấu, biên giới, mục lục, song ngữ ở lề | [`mockups/reading-mode.html`](mockups/reading-mode.html) | FR119 · FR120 · FR11 |
| Ba mức typography Chế độ đọc, sáng và tối | [`mockups/reading-mode-typography.html`](mockups/reading-mode-typography.html) | Chế độ đọc — tham chiếu chữ nghĩa |
| **Nhập từ URL + màn xem trước hợp nhất** — dán link, đối chiếu bảng mã, sửa ranh giới bóc, luật làm sạch, nhật ký domain; sáng và tối | [`mockups/web-import.html`](mockups/web-import.html) | KF-4 · FR122–FR128 · NFR19 |
| **Xuất — cách xuất ảnh và khối ghi nguồn**, kèm danh sách ảnh không có link gốc | [`mockups/export-images-attribution.html`](mockups/export-images-attribution.html) | FR130 · FR131 · FR128 |

### Bổ sung 2026-08-03 — bề mặt của Giai đoạn 1 và 2

Đợt rà soát ngày 2026-08-03 đối chiếu toàn bộ 120 FR với các bề mặt đã dựng và tìm ra **5 trên 10 nhóm năng lực chưa có bề mặt nào**. Sáu file dưới đây đóng phần thuộc **Giai đoạn 1 và 2** theo trình tự xây dựng ở PRD §10 — phần còn lại nằm ở mục *Còn thiếu*.

| Bề mặt | File | Neo tới |
|---|---|---|
| **Nguồn từ điển** — bật/tắt, màn hình Attribution, chứng minh gỡ lớp | [`mockups/sources-attribution.html`](mockups/sources-attribution.html) | FR36 · FR37 · FR38 · FR109 · FR112 |
| **Lịch sử tra cứu và mục đã ghim** — tab thứ ba của Panel Lookup | [`mockups/lookup-history-pins.html`](mockups/lookup-history-pins.html) | FR41 · A9 |
| **Cài đặt** — hai tầng scope, AI/BYOK, phím tắt, bố cục, cập nhật | [`mockups/settings.html`](mockups/settings.html) | FR22 · FR65–FR68 · FR17 · FR18 · FR103 · FR111 |
| **Quản lý Glossary** — hai tầng, xuất xứ, chờ chốt, trộn CSV | [`mockups/glossary-manage.html`](mockups/glossary-manage.html) | FR46–FR51 · FR114 |
| **Bộ prompt theo thể loại** — soạn, biến chèn động, chia sẻ | [`mockups/prompt-library.html`](mockups/prompt-library.html) | FR69 · FR79 |
| **Dịch theo lô** — huỷ giữa chừng, lỗi, token và chi phí | [`mockups/ai-batch-translate.html`](mockups/ai-batch-translate.html) | FR73–FR76 · FR72 · FR77 |

### Bổ sung 2026-08-03 — bề mặt của Giai đoạn 3

| Bề mặt | File | Neo tới |
|---|---|---|
| **Tìm kiếm xuyên Library** — hai chế độ dấu, và màu khớp đổi theo độ chắc chắn | [`mockups/library-search-results.html`](mockups/library-search-results.html) | FR8 · FR9 · FR10 · NFR8 |
| **Danh sách Chương** — bốn trạng thái ở quy mô 2000 Chương, ghi đè trạng thái, gộp/tách | [`mockups/chapter-list.html`](mockups/chapter-list.html) | FR5 · FR6 · FR7 · FR12 · FR15 |
| **Tầng dữ liệu** — quét lại, mục mồ côi, lịch sử phiên bản, sao lưu bằng copy thư mục | [`mockups/data-integrity.html`](mockups/data-integrity.html) | FR96–FR102 · FR63 · NFR18 |

### Bổ sung 2026-08-03 — bề mặt của Giai đoạn 4

| Bề mặt | File | Neo tới |
|---|---|---|
| **Khớp mờ TM** — dải mọc dưới câu đang sửa, phần trăm và diff hai phía, thuật toán theo ngôn ngữ | [`mockups/tm-fuzzy-match.html`](mockups/tm-fuzzy-match.html) | FR58 · FR59 · FR61 · FR40 |
| **Quản lý Translation Memory** — xuất xứ, nhiều bản dịch, TMX, dải sức khoẻ TM | [`mockups/tm-manage.html`](mockups/tm-manage.html) | FR56 · FR57 · FR62 · FR63 · FR64 · FR117 · FR118 |

### Bổ sung 2026-08-03 — bề mặt của Giai đoạn 5

| Bề mặt | File | Neo tới |
|---|---|---|
| **Xuất cho Reviewer** — phạm vi, **bốn** định dạng, hình dạng file thật, vòng khứ hồi | [`mockups/export-share.html`](mockups/export-share.html) | FR87 · FR88 · FR89 · FR44 · FR91 · FR95 · **FR121** |

### Bổ sung 2026-08-03 — Giai đoạn 6 và bố cục màn hình hẹp

| Bề mặt | File | Neo tới |
|---|---|---|
| **AI Proofreader** — gạch chân tại chỗ, hai loại phát hiện, bỏ qua có ghi nhớ, tỷ lệ báo động giả | [`mockups/proofreader.html`](mockups/proofreader.html) | FR80–FR86 |
| **Bố cục màn hình hẹp** — ngưỡng và thứ tự hy sinh | [`mockups/narrow-layout.html`](mockups/narrow-layout.html) | FR16 · FR17 · FR18 · FR20 · FR21 |

**Bản đồ năng lực nay đã phủ kín: cả 10 nhóm C1–C10 đều có bề mặt.**

> **Ngoại lệ token có chủ ý.** Khối xem trước `.docx` — trong `export-share.html` và `export-images-attribution.html` — dùng màu viết cứng (`#fff`, `#1e1c19`, `#f2efe8`…) vì nó **mô phỏng một tài liệu Word**, không phải bề mặt của ứng dụng. Áp token giấy ngà vào đó sẽ nói dối về thứ reviewer thật sự nhìn thấy. Các màu này vẫn đã kiểm đạt AA. Đừng "sửa" chúng về token.

**Đã kiểm toán 2026-08-03.** Cả **29 file** dùng token hiện hành: không màu nào trượt WCAG AA, không biến nào đã bị bỏ, không chỗ nào dùng `opacity` để làm mờ chữ, tất cả khai `Source Serif 4` / `Source Han Serif` / `Source Sans 3` (giữ chuỗi dự phòng phía sau để render được khi máy chưa cài font).

> **Lần kiểm toán ngày 2026-08-02 đã bỏ sót.** Nó tuyên bố *"cả 8 file"* trong khi bảng đã có 13, và tuyên bố không màu nào trượt AA trong khi sáu chỗ dùng `ornament` hoặc `tm-rule` làm màu chữ ở **2,5:1** — kể cả một câu văn bản dịch nguyên vẹn ở `key-screen-workspace.html`. Đã sửa hết ngày 2026-08-03. **Lỗ hổng đứng sau nó** là `DESIGN.md` chỉ đặt luật cho *token màu* mà không đặt luật cho `opacity`, nên bảng chờ Glossary lùi hàng đã duyệt bằng `opacity: 0.4` mà vẫn qua được mọi lần kiểm. Luật `opacity` nay đã có trong `DESIGN.md § Sàn tương phản`.

Bốn file **không nằm trong bảng này** và đã rút về `.working/` — chúng là **dấu vết quyết định**, không phải tài liệu để dựng: `direction-ban-tho.html`, `direction-ke-thua.html`, `direction-ban-viet-DA-CHON.html` (hướng thị giác nào thắng) và `layout-2x2-variants.html` (cách ghép 2×2 nào thắng). Để chúng trong `mockups/` sẽ khiến người dựng tưởng là tham chiếu, trong khi chúng cố tình giữ nguyên hiện trạng lúc quyết định.

Ngoài bốn file đó, `.working/` còn **sáu bản cũ từng trùng tên với mockup hiện hành** — hai trong số đó mang màu `#7d766c` đã bị loại vì trượt AA. Ngày 2026-08-03 chúng được đổi tên thành `superseded-*` và `.working/README.md` ghi rõ file nào là gì. Trước đó, mở nhầm `.working/key-screen-workspace.html` là dựng theo bản sai mà không có gì báo.

## Trạng thái rỗng

Người dùng là dịch giả chuyên nghiệp. Trạng thái rỗng nêu **vì sao rỗng** và **làm gì tiếp** — không minh hoạ to, không dấu chấm than, không xin lỗi.

- **Lookup không có kết quả** khác **Lookup chưa tra gì**: hai câu khác nhau, hai gợi ý khác nhau. Không tìm thấy thì gợi ý tra từng chữ và trỏ sang Concordance; chưa tra thì dạy thao tác bôi đen.
- **Library lần đầu** giải thích Tác phẩm là gì và là một thư mục mang đi được, rồi mới mời nhập.
- **TM trống** giải thích **cơ chế** — TM tự đầy khi xác nhận câu. Không có nút "thêm vào TM", nên nếu không nói ra người dùng sẽ đi tìm nó.
- **Chưa cấu hình AI không phải trạng thái lỗi.** Panel mời cấu hình và nói rõ mọi thứ khác vẫn chạy đầy đủ (FR77).

### Phạm vi và thao tác — FR119, FR120

**Chỉ đọc phần đã xong.** Đọc liên tục qua các Chương *Đã xong* rồi dừng ở **mốc biên giới** tường minh, kèm đường sang Workspace để dịch tiếp. Chương chưa dịch **không hiện nguyên văn** — tiếng Trung xen giữa trang đọc tiếng Việt là phá trải nghiệm, không phải bổ sung thông tin. Câu **chưa xác nhận** trong Chương đã xong vẫn hiện nhưng **gạch chấm nhẹ**: Chương có thể được đánh dấu xong bằng tay (FR6) trong khi còn câu chưa xác nhận, và hiện chúng như đã hoàn chỉnh là nói dối về trạng thái công việc.

**Đánh dấu là chính, nhảy thẳng là phụ.** `M` đánh dấu câu rồi **đọc tiếp ngay**; `↵` nhảy sang Workspace khi muốn sửa ngay. Affordance **ẩn hoàn toàn**, chỉ hiện khi con trỏ chuột hoặc tiêu điểm bàn phím chạm câu — trang đọc vẫn sạch, và FR11 vẫn đúng vì một dấu và một đường điều hướng không sửa gì cả. Các chỗ đánh dấu gom thành **một danh sách theo Tác phẩm**, sửa một lượt sau khi đọc xong.

**Mọi thao tác có phím** (NFR17, AD-34): `B` song ngữ · `1 2 3` ba mức chữ · `⌘,` tinh chỉnh · `D` sáng/tối · `⌘L` mục lục · `M` danh sách đánh dấu.

## Open Questions

### ✅ Mâu thuẫn `FR47` / `FR53` — đã đóng 2026-08-02

Xung đột phát hiện khi dựng bảng chờ: `FR53` bắt duyệt một phím không gõ, `FR47` bắt mỗi mục Glossary phải có bản dịch đã chốt — nhưng ứng viên do máy quét ra chưa có bản dịch nào.

`bmad-prd` đã giải bằng **hai FR mới**, và lời giải **tốt hơn** đề xuất ban đầu của thiết kế:

- **`FR113`** — với ứng viên tiếng Trung, hệ thống **đề xuất sẵn bản dịch bằng âm Hán Việt** lấy từ dữ liệu đã nhúng (`FR33`), chạy hoàn toàn ngoại tuyến. Một phím nhận cả thuật ngữ lẫn bản dịch; mục vào Glossary ở trạng thái **đã chốt**. Với danh từ riêng tiếng Trung, âm Hán Việt *chính là* bản dịch quy ước — `北涼` → *Bắc Lương*. Đây là tra cứu một dữ kiện đã có, không phải phỏng đoán của máy.
- **`FR114`** — phần không đề xuất được (tiếng Anh, hoặc Hán Việt không phù hợp) vào Glossary ở trạng thái **chờ chốt bản dịch**, chốt lần đầu gặp trong Workspace.

Hệ quả cho thiết kế đã áp vào mockup: thẻ quyết định hiện **bản dịch đề xuất** khi có; `FR50` đánh dấu cả mục chờ chốt nhưng **phân biệt được**; `FR70` chỉ chèn mục đã chốt vào prompt.

### Còn thiếu

- ~~**Chưa dựng bố cục ở màn hình hẹp**~~ — ✅ **đóng 2026-08-03.** Ngưỡng đo theo **vùng làm việc** (chiều cao cửa sổ trừ thanh tiêu đề 38px và thanh trạng thái 32px), không theo kích thước màn hình: **≥ 1100×820** giữ 2×2 · **< 820 cao** gộp hàng dưới thành một panel có tab · **< 1100 rộng hoặc < 700 cao** chỉ còn Nguyên văn | Bản dịch, Tra cứu rút về ngăn kéo · **< 860 rộng** báo không hỗ trợ. Thứ tự hy sinh: Đề xuất AI trước, Tra cứu sau (nhưng rút về thanh trạng thái chứ **không bao giờ mất hẳn**), cặp Nguyên văn | Bản dịch **không bao giờ nhường**. Ngưỡng là điểm khởi đầu, cần hiệu chỉnh trên máy thật — cùng loại với A6/A7/A8. Xem [`mockups/narrow-layout.html`](mockups/narrow-layout.html).
- **Font thật chưa đo.** `DESIGN.md` đã chốt hệ font nhúng nhưng dung lượng, giấy phép và biến thể vùng đều chờ một mũi thăm dò kỹ thuật — không phải việc thiết kế.
- ~~**`AD-39` thiếu bước tách Chương của `FR14`**~~ — ✅ **đóng 2026-08-03.** Phát hiện khi dựng màn xem trước, bàn giao ngược cho `bmad-architecture`, Winston sửa Rule của AD-39 tại chỗ (không thêm AD mới, 43 AD giữ nguyên ID). Chuỗi pipeline nay có bước **tách Chương theo mẫu phân tách**, đặt **sau chuẩn hoá (FR125), trước xem trước** — đúng vị trí và đúng lý do đã nêu: màn xem trước của FR14 hiện *"đã nhận ra N Chương"* nên bước này phải xong trước khi màn đó dựng. `Prevents` của AD-39 nhận thêm ca hỏng cụ thể: đặt tách Chương **trước** bước giải mã bảng mã thì mẫu chạy trên chữ rác, cả file 40 MB ra **đúng một Chương**, không lỗi nào được ném — đúng lỗi thất bại im lặng mà FR126 tồn tại để chặn.

  **Bản chốt khác đề xuất ban đầu của tầng thiết kế ở một chỗ, và bản chốt là bản đúng.** Tôi đề nghị khai điều kiện áp dụng theo **danh sách đường nhập** (*áp cho file, không áp cho URL*); Winston khai theo **hình dạng đầu vào** — đầu vào đến thành **một dòng chưa chia Chương** thì có tách, đầu vào **đã một đơn vị một Chương** thì không. Danh sách sai ngay khi có đường nhập thứ tư; hình dạng thì đúng mãi. Người đọc mục này về sau: **bản chốt là tiêu chí hình dạng.**

  **Hệ quả tôi đọc sai và nay đã sửa:** tôi bàn giao rằng FR115 (nhập song ngữ) *không* cần tách Chương. Sai — một `.docx` hai cột chứa cả bộ truyện cũng đến thành một dòng chưa chia Chương, nên theo tiêu chí hình dạng nó **có** cần. Và [`mockups/bilingual-import.html`](mockups/bilingual-import.html) dựng **từ hôm trước** vốn đã có sẵn trường *"Mẫu nhận diện đầu Chương — **áp lên cột nguồn**"*: tầng thiết kế đã trả lời câu hỏi này trước khi nó được đặt ra.

- ~~**`FR115` chưa nói mẫu phân tách khớp vào cột nào**~~ — ✅ **đóng 2026-08-03.** `bilingual-import.html` chọn **cột nguồn**, và cả hai tầng trên đã phê chuẩn lựa chọn đó thành chữ: **PRD §6.1 `FR115`** nay ghi *"Ranh giới Chương lấy từ mẫu phân tách của FR14, và mẫu đó áp lên **cột nguồn**"*, và **`ARCHITECTURE-SPINE.md` AD-39** ghi *"Mẫu phân tách áp lên cột nguồn (PRD chốt 2026-08-03)"*. Lý do giữ nguyên như tầng thiết kế đã nêu: đầu Chương gốc mang dạng `第N章` ổn định và máy khớp được, còn cột đích do người khác dịch có thể ghi *"Chương 5"*, ghi khác đi, hoặc bỏ hẳn dòng tiêu đề — khớp vào cột đích là đặt độ tin cậy của cả lần nhập vào thói quen của một người dịch mà người dùng không kiểm soát được.

  **Ảnh hưởng tới giao diện: không.** Màn xem trước song ngữ đã có sẵn trường mẫu phân tách và dòng *"N Chương nhận ra"*.

  > *(Mục này từng được ghi là 🟡 còn mở với việc cần làm **"PRD nên phê chuẩn lựa chọn này thành chữ"**. Việc đó đã xong ở cả PRD lẫn Architecture trước khi mục này được rà lại; đánh dấu đóng ngày 2026-08-03 trong đợt rà mức sẵn sàng triển khai.)*
- ~~**Mâu thuẫn giãn dòng `ui` trong `DESIGN.md`**~~ — ✅ **đóng 2026-08-03.** Sàn 1.66 áp cho **chữ nội dung họ `read`**; họ `ui` được phép ở **1.4 và 1.5** cho nhãn một dòng, nhưng **quay lại 1.66 khi chuỗi có khả năng xuống dòng** (mô tả dưới ô thiết lập, câu trạng thái, hộp giải thích). Ranh giới là *chữ có chạy thành đoạn hay không*, không phải cỡ chữ. Xem `DESIGN.md § Giãn dòng`.

### ✅ Bề mặt theo bản đồ năng lực — đã phủ kín 2026-08-03

Đợt rà soát ngày 2026-08-03 tìm ra **5 trên 10 nhóm năng lực chưa có bề mặt nào**. Toàn bộ đã được dựng theo trình tự xây dựng ở PRD §10. **Cả C1–C10 nay đều có bề mặt**, và bố cục màn hình hẹp đã có ngưỡng.

### ✅ Yêu cầu mới đã vào PRD — đóng 2026-08-03

**`FR121` — xuất `.docx` một khối, đối xứng theo đoạn.** Chủ dự án yêu cầu ngày 2026-08-03. Bảng hai cột, **một hàng duy nhất cho cả Chương**, không đường kẻ ngang, hai ô giữ đúng số lần xuống đoạn như nhau. Mục đích: **bôi đen cột phải là copy được trọn bản dịch sang trình soạn thảo của website**, ra văn bản liền mạch không kèm mảnh vụn bảng biểu.

Đã dựng ở [`mockups/export-share.html`](mockups/export-share.html) §2b, nhưng **PRD chưa có yêu cầu này**:

- `FR87` nói `.docx` hai cột **đối xứng theo segment**. Định dạng mới đối xứng theo **đoạn** — khác đơn vị, khác mục đích, nên là một FR riêng chứ không phải một tuỳ chọn của FR87.
- Nó **không nhập lại được** (không có số câu, không có ranh giới câu), nên nó nằm **ở cuối vòng khứ hồi chứ không nằm trong vòng**. Cùng nhóm với `.txt`.
- Kéo theo một quyết định phụ đã chốt ở tầng thiết kế: **câu chưa xác nhận không được đánh dấu** ở định dạng này — một nền vàng hay dòng ghi chú xen giữa văn xuôi sẽ đi thẳng vào bài đăng. Thay vào đó màn hình xuất **cảnh báo trước khi bấm**.

> **✅ Đã làm 2026-08-03.** `bmad-prd` đưa `FR121` vào PRD §6.8 (dải C8), đặt ngay sau `FR87` theo đúng quy ước không đánh số lại; bản đồ năng lực §5.1 và tổng số FR (120 → 121) đã cập nhật. Cả hai ràng buộc của tầng thiết kế được nâng vào PRD thành điều kiện nghiệm thu: **không nhập lại được** (màn hình xuất phải nói ngay lúc chọn định dạng) và **câu chưa xác nhận không đánh dấu trong file, cảnh báo trước lúc xuất**.
>
> **✅ Câu hỏi rộng hơn cũng đã đóng 2026-08-03.** Chủ dự án xác nhận: **chỉ cần đúng định dạng file để đăng, không cần gì thêm.** Không mở nhóm năng lực riêng cho việc đăng bài — không xuất hàng loạt thành nhiều file, không theo dõi Chương nào đã đăng, không tiêu đề/ghi chú đầu bài. `FR121` là toàn bộ phạm vi, và nó ở lại trong C8.
