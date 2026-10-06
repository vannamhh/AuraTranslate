# PRD Quality Review — AuraTranslate

**Ngày rà soát:** 2026-08-02 · **Rúbric:** `.claude/skills/bmad-prd/assets/prd-validation-checklist.md`

## Overall verdict

PRD này mạnh bất thường ở chỗ khó nhất: **nó trung thực về những gì chưa biết và biến rủi ro thành yêu cầu thay vì giấu đi**. FR31/FR32 (bắt buộc hiển thị nguồn) và FR95 (thu hoạch thuật ngữ độc lập với Diff Viewer) là hai ví dụ mẫu mực của việc một rủi ro chưa giải được vẫn cho ra thiết kế đúng. Các NFR có số đo thật từ Giai đoạn 0, không phải boilerplate.

Điểm yếu tập trung vào **một chỗ duy nhất và nó nghiêm trọng: done-ness**. Một phần đáng kể FR/NFR dùng tính từ thay vì ngưỡng — *"tối ưu cho việc đọc dài"*, *"không có gai trễ cảm nhận được"*, *"dưới ngưỡng cảm nhận"*, *"lặp lại nhiều lần"*. Đây là PRD đầu chuỗi (feeds UX → Architecture → Epics), nên chính những chỗ mơ hồ này sẽ nở ra thành story không nghiệm thu được. Thiếu **Glossary thuật ngữ** cũng là vấn đề thật với một PRD đầu chuỗi, và đã có drift *"dự án" / "Tác phẩm" / "Project"*.

---

## 1. Decision-readiness — **strong**

Quyết định được phát biểu **là quyết định**, kèm cái giá phải trả. FR35 nói thẳng nhãn từ loại tiếng Anh là cái giá của phương án C. §9.1 nói thẳng không ký số là rào cản đón nhận *"không thể xoá bỏ bằng kỹ thuật"*. R1 giữ nguyên mức 🔴 thay vì làm dịu đi. §9.2 vẽ chuỗi ràng buộc nối tiếp để người sau không gỡ nhầm mắt xích.

Q1 là **câu hỏi mở thật** — chủ dự án chọn không trả lời, PRD ghi nhận hiện tượng mà không suy diễn động cơ, và FR95 khiến câu trả lời không còn chặn tiến độ. Đây là cách xử lý đúng một unknown hành vi.

### Findings
- **low** Thiếu callout tại điểm căng thật nhất (§3.1) — §3.1 tuyên bố *"v1 bao gồm toàn bộ mười nhóm năng lực"* rồi chuyển ngay sang §3.2, không có callout nối tới R1 🔴. Người đọc lướt có thể không nhận ra hai chỗ này nói về cùng một quyết định. *Fix:* thêm một dòng ở §3.1 trỏ tới R1.

---

## 2. Substance over theater — **strong**

Không có persona theater: ba vai, và mỗi vai **lái một quyết định thật**. Vai Reviewer là toàn bộ lý do C8 tồn tại — §2 nói thẳng *"không có nó, AuraTranslate cắt người dùng khỏi nhóm của họ"*.

Không có NFR theater ở phần đã đo: NFR1, NFR6, NFR8 mang số thật kèm nguồn gốc số đo. NFR8 còn giải thích cả chi phí (~17 MB mỗi chỉ mục) và vì sao nằm trong ngân sách.

Vision không thể hoán đổi sang PRD khác — nó gắn vào một ngách cụ thể (Anh/Trung → Việt, Hán Việt bắt buộc) và một khoảng trống cụ thể (QuickTranslator chết 2022, không có macOS).

### Findings
- **medium** NFR3, NFR4, NFR5 là NFR theater (§7.1) — *"đủ nhanh để dùng như thao tác thường ngày"*, *"vài giây"*, *"phù hợp với công cụ chạy nền suốt ngày"*. Ba NFR này nằm cạnh những NFR có số đo thật, nên sự tương phản càng lộ. Có đánh dấu `[ASSUMPTION]` và có Q4 theo dõi — điều đó làm nó trung thực, nhưng **không làm nó nghiệm thu được**. *Fix:* đặt ngưỡng tạm bằng phán đoán kỹ thuật (ví dụ tìm kiếm p95 < 500 ms trên 5.000 Chương; khởi động < 3 giây), ghi rõ là ngưỡng tạm sẽ hiệu chỉnh ở Giai đoạn 3. Ngưỡng sai còn hơn không có ngưỡng.

---

## 3. Strategic coherence — **adequate**

PRD có luận điểm và các tính năng phục vụ nó: *AI nằm dưới quyền người biên tập, và giá trị nằm ở môi trường bao quanh AI.* Thứ tự xây dựng ở §10 tuân theo luận điểm — từ điển trước, AI sau — chứ không theo "cái gì dễ làm trước". Success metrics ở §4.2 đo đúng luận điểm (tuân thủ Glossary, consistency drift) chứ không đo hoạt động. Counter-metrics có mặt và §4.3 nói rõ counter-metric #1 đo chính việc *phản bội trụ #1*.

Nhưng **luận điểm chưa bao giờ được phát biểu thành câu**. Nó phải suy ra từ bốn trụ định vị.

### Findings
- **high** Thiếu luận cứ ngành chống lưng cho luận điểm (§1) — Nghiên cứu có một nhận định trực tiếp xác nhận luận điểm sản phẩm: *"LLM cho ra bản dịch trôi chảy nhưng thiếu tính nhất quán, thiếu cưỡng chế glossary và thiếu theo dõi thay đổi — nên hoạt động tốt nhất khi nằm bên trong một môi trường CAT, chứ không đứng một mình."* Đây là bằng chứng bên ngoài mạnh nhất cho toàn bộ đặt cược của dự án, và nó **không có mặt trong PRD**. *Fix:* đưa vào §1 như một đoạn luận điểm tường minh.
- **medium** Thiếu ràng buộc thiết kế từ sự quen thuộc với QuickTranslator (§1 hoặc §2) — Khảo sát cạnh tranh ghi: *"Cộng đồng đã quen với mô hình tra cứu của QT — đây vừa là lợi thế (không phải dạy lại) vừa là ràng buộc (lệch quá sẽ bị từ chối)."* Đây là **ràng buộc thiết kế thật** lên C2/C3 và không xuất hiện ở đâu trong PRD. *Fix:* thêm vào §2 hoặc mở đầu §6.3.
- **medium** Thiếu lợi thế cạnh tranh trung thực (§1) — Brief có một đoạn thẳng thắn hiếm thấy: *"không có rào cản kỹ thuật nào ở đây... Lợi thế thật nằm ở chỗ sản phẩm được xây bởi một người dịch thật, cho công việc thật của chính mình, trong một ngách mà các sản phẩm quốc tế không nhìn tới."* Việc PRD bỏ đoạn này khiến định vị đọc lạc quan hơn thực tế. *Fix:* đưa vào §1.
- **low** Thiếu tầm nhìn dài hạn (§1) — Brief có phần "Tầm nhìn" 2–3 năm: Library lớn dần thành kho lưu trữ cá nhân, TM/Glossary tích luỹ đủ dày để AI viết giống chính người dùng, và giá trị bền nhất có thể là những gì cộng đồng bồi đắp. Đây là thứ giải thích **vì sao** C1 và C5 đáng công. *Fix:* thêm một đoạn ngắn cuối §1.

---

## 4. Done-ness clarity — **thin**

Đây là chiều yếu nhất, và với một PRD đầu chuỗi thì nó quan trọng nhất.

Có những FR nghiệm thu được rất tốt — FR39 (*"phải trả kết quả cho truy vấn 1, 2 và 3+ ký tự"*) là mẫu mực: nó sinh ra từ một lỗi đo được và biến thẳng thành điều kiện nghiệm thu. FR91, FR97, FR98, FR111 đều có hệ quả kiểm chứng được.

Nhưng một nhóm FR/NFR dùng tính từ ở đúng chỗ cần con số.

### Findings
- **high** NFR1 không có ngưỡng đầu-cuối (§7.1) — *"Dưới ngưỡng cảm nhận của người dùng"* không nghiệm thu được. Số 0,022 ms đã đo **chỉ là phía backend**; ngân sách IPC + render vẫn để trống, mà đó chính là chỗ rủi ro còn lại (A1). *Fix:* đặt ngưỡng đầu-cuối tường minh, ví dụ *p95 dưới 100 ms từ lúc thả chuột tới lúc kết quả hiển thị*, và ghi rõ backend đã tiêu 0,05 ms trong ngân sách đó.
- **high** NFR2 / FR100 không có bound (§7.1, §6.9) — *"Không có gai trễ cảm nhận được khi đang gõ"* là tính từ. Auto-save là rủi ro R9 đã nhận diện, nên đây đúng là chỗ cần số. *Fix:* ví dụ *không frame nào vượt 50 ms trong lúc auto-save chạy*.
- **high** FR81 không có tiêu chí chấp nhận (§6.7) — *"nghi dịch sai, dịch thoát nghĩa quá xa, hoặc cấu trúc câu tối nghĩa"* là phán đoán của mô hình, vốn không có đáp án đúng. Không nghiệm thu được theo lối thông thường. *Fix:* đổi tiêu chí sang thứ đo được — *tỷ lệ báo động giả mà người dùng đánh dấu "không phải lỗi" phải đủ thấp để người dùng không tắt tính năng*, và neo vào FR84.
- **medium** FR52 thiếu ngưỡng "lặp lại nhiều lần" (§6.4) — Không có ngưỡng thì không biết cài đặt đúng hay sai, và một ngưỡng tồi sẽ đổ ra danh sách ứng viên vô dụng, kéo theo R4. *Fix:* đặt ngưỡng khởi điểm (ví dụ ≥ 5 lần xuất hiện) và cho người dùng chỉnh.
- **medium** FR11 không nghiệm thu được (§6.1) — *"typography và bố cục tối ưu cho việc đọc dài"*. Reading Mode là mục đích số 1 của Library nên chỗ này không nên để mơ hồ. *Fix:* nêu vài bound cụ thể (độ rộng dòng, cỡ chữ chỉnh được, chiều cao dòng, chế độ sáng/tối) hoặc chuyển đặc tả sang `bmad-ux` và ghi rõ là bàn giao có chủ ý.
- **low** FR17 *"thay đổi kích thước linh hoạt"* (§6.2) — chấp nhận được vì FR16/FR18 đã mang phần nghiệm thu, nhưng vẫn là tính từ trần.

---

## 5. Scope honesty — **adequate**

§3.2 làm việc thật: nó loại bỏ những thứ người đọc **sẽ mặc định là có** (cloud sync, tài khoản, bản web/mobile) chứ không liệt kê cho đủ. §8.3 ghi rõ ràng buộc *"im lặng vài tuần = không được phép"*. §11.1 nêu 5 giả định kèm hệ quả nếu sai — cột *"Nếu sai thì sao"* là thứ hiếm gặp và có giá trị thật.

Mật độ open items: 5 câu hỏi mở + 3 `[ASSUMPTION]` cho một PRD đèn xanh xây dựng. Q1 và Q3 đều có đường lui thiết kế nên không chặn. **Q4 thì chặn** — nó là lý do chiều Done-ness ở mức *thin*.

### Findings
- **medium** Không có Assumptions Index, và `[ASSUMPTION]` nội dòng không khớp §11.1 (§7.1, §11.1) — PRD có hai bộ giả định tách rời: các tag `[ASSUMPTION]` nội dòng (NFR3, NFR4) và bảng A1–A5 ở §11.1. Chúng không tham chiếu lẫn nhau. Ngoài ra FR41 (lịch sử tra cứu + ghim) được ghi là giả định trong memlog nhưng **không hề đánh dấu trong PRD**. *Fix:* gộp thành một Assumptions Index ở §11.1, mỗi tag nội dòng trỏ tới ID trong bảng; bổ sung FR41.

---

## 6. Downstream usability — **thin**

PRD này là đầu chuỗi thật (§10 nêu rõ nó nuôi UX → Architecture → Epics), nên chiều này có trọng số cao.

Điểm mạnh: ID sạch tuyệt đối — 112 FR liên tục 1–112, không trùng, không thiếu, không tham chiếu treo (đã kiểm bằng máy). Tham chiếu chéo đều phân giải được (FR36↔FR112, FR44↔FR88, FR54↔FR95, FR8↔FR98). `addendum.md` ghi rõ mỗi mục chống lưng cho FR/NFR nào — đây là thứ `bmad-architecture` dùng được ngay.

Điểm yếu: **không có Glossary thuật ngữ**, và đã có drift.

### Findings
- **high** Thiếu mục Glossary thuật ngữ (toàn tài liệu) — PRD dựng ít nhất 10 danh từ miền có nghĩa riêng: *Tác phẩm, Chương, segment, Glossary, Translation Memory, Lookup, `.atproj`, lớp gỡ rời, Workspace, Library*. Với PRD đầu chuỗi, thiếu Glossary nghĩa là mỗi workflow downstream sẽ tự định nghĩa lại. *Fix:* thêm mục Glossary, đặt trước §6.
- **medium** Drift *"dự án" / "Tác phẩm" / "Project"* (§6.1, §6.9) — FR4 viết *"Glossary và TM phạm vi dự án gắn ở tầng Tác phẩm"*; bảng FR103 viết *"Project (tầng Tác phẩm)"*; §6.9 dùng *"Tác phẩm"*. Ba tên cho một khái niệm, trong đó *"dự án"* còn trùng nghĩa với *"dự án AuraTranslate"*. *Fix:* chốt **Tác phẩm** là tên chính thức; giữ *Project Scope* chỉ khi trích dẫn PRD v8.0 và ghi rõ là bí danh.
- **low** Không có User Journey (toàn tài liệu) — Là lựa chọn đúng cho hình dạng sản phẩm (xem chiều 7), nhưng `bmad-ux` sẽ phải tự dựng lại hành trình từ FR. *Fix:* không sửa trong PRD; nêu rõ khi bàn giao sang `bmad-ux`.

---

## 7. Shape fit — **strong**

Hình dạng khớp sản phẩm. Đây là công cụ chuyên nghiệp một người vận hành, đầu chuỗi, nên **capability spec** là đúng và **User Journey sẽ là gánh nặng thừa** — rúbric nói rõ điều này cho nhóm single-operator.

Ba mục thêm vào (§8 Giấy phép & xuất xứ, §9 Phát hành & tin cậy, §10 Trình tự xây dựng) **đều xứng đáng có mặt**, không phải section trang trí:
- §8 là mối lo lớn nhất ngoài kỹ thuật của chính dự án, và là nơi một nguyên tắc sản phẩm hoá thành ràng buộc pháp lý.
- §9 tồn tại vì một ràng buộc kinh phí thật đã sinh ra FR106, FR107, FR111.
- §10 là cách duy nhất xử lý R1 mà không cắt phạm vi.

Độ dài (733 dòng) tương xứng mức launch và 112 FR — không có dấu hiệu độn cho dày.

---

## Mechanical notes

- ✅ **ID continuity:** 112 FR liên tục 1–112, không trùng, không thiếu. NFR1–NFR16 không trùng. Kiểm bằng máy.
- ✅ **Cross-references:** mọi tham chiếu `FRn` trong `prd.md` và `addendum.md` đều trỏ tới FR tồn tại.
- ✅ **Quy ước đánh số ổn định** được tuyên bố đầu tài liệu và **được tuân thủ thật** — FR42–45 và FR78–79 nhận số cuối dãy dù nằm giữa tài liệu, đúng như đã cam kết.
- ❌ **Glossary:** không có (finding high ở chiều 6).
- ❌ **Assumptions Index roundtrip:** không có index; FR41 là giả định nhưng không được đánh dấu (finding medium ở chiều 5).
- ⚠️ **Glossary drift:** *dự án / Tác phẩm / Project* (finding medium ở chiều 6).
- ✅ **Required sections:** đủ cho mức launch và cho loại capability-spec.
