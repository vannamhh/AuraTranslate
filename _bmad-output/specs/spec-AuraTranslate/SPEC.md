---
id: SPEC-AuraTranslate
companions:
  - glossary.md
  - requirements.md
  - data-sources.md
  - build-sequence.md
  - risks.md
  - _bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/addendum.md
  - _bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md
  - _bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/DESIGN.md
  - _bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md
sources:
  - _bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md
---

> **Hợp đồng canonical.** SPEC này cùng các file trong `companions:` là hợp đồng đầy đủ, đã qua kiểm chứng bảo toàn, về thứ phải xây, phải test và phải nghiệm thu. Tài liệu ở `sources:` chỉ phục vụ truy vết — chỉ mở khi cần phần lập luận tường thuật mà hợp đồng này cố ý lược bỏ.

# AuraTranslate

## Why

**Một tầm nhìn cần hiện thực hoá, chồng lên một nỗi đau đang diễn ra.** QuickTranslator — công cụ mà cộng đồng dịch giả Việt Nam dùng suốt hơn một thập kỷ — dừng phát triển từ 2022 và chỉ chạy trên Windows; người dịch trên macOS không có gì thay thế. Cùng lúc, làn sóng công cụ dịch AI 2026 đi ngược hướng: cloud-based, tự động hoá tối đa, phục vụ **độc giả muốn đọc nhanh** thay vì **người dịch muốn dịch hay**. Người chịu thiệt là người dịch Anh/Trung → Việt coi trọng chất lượng hơn sản lượng: công sức của họ không đọng lại ở đâu, tra cứu vẫn là lao động thủ công, và bài học của reviewer không bao giờ quay về vòng làm việc kế tiếp. AuraTranslate đặt cược rằng giá trị nằm ở **môi trường làm việc bao quanh AI**, không phải ở bản thân AI — nhận định mà cả ngành CAT công nhận và cả nhóm cạnh tranh 2026 bỏ qua. Đây là ngách mà sản phẩm quốc tế không nhìn tới: cặp Anh/Trung → Việt với Hán Việt là thành phần bắt buộc, xây bởi một người dịch thật cho công việc thật của chính mình. Trong hai đến ba năm, đích là trở thành thứ QuickTranslator từng là với thế hệ trước — **công cụ mặc định của người dịch Việt Nam** — nhưng cho thời đại AI và không bỏ rơi ai vì hệ điều hành họ dùng; đó là lý do CAP-1 (Library) và CAP-5 (Translation Memory) đáng công dù không phải thứ người dùng đòi hỏi ngay: Library lớn dần thành kho lưu trữ cá nhân, còn TM + Glossary tích luỹ đủ dày để AI viết ra thứ ngày càng giống **chính người dùng**. Mọi đánh đổi downstream phân xử ngược lại về đây: **các công cụ khác giúp bạn dịch xong; AuraTranslate là nơi bản dịch của bạn sống.**

## Capabilities

- **CAP-1 — Library**
  - **intent:** Người dịch mở ứng dụng vào Library, nắm được mình đang có những gì, tìm lại và đọc lại mọi thứ đã dịch. `FR1–FR15, FR43, FR45, FR115, FR116`
  - **success:** Tìm full-text phân biệt dấu xuyên toàn Library trả kết quả kèm Tác phẩm · Chương · đoạn khớp, p95 < 500 ms trên 5.000 Chương; mở một Chương đưa thẳng vào Workspace đúng vị trí làm việc lần trước; Chế độ đọc đọc liên tục qua nhiều Chương, không có công cụ biên tập, hiển thị ảnh nhúng đúng vị trí; nhập một file lớn tách thành nhiều Chương có màn hình xem trước trước khi xác nhận; **nhập file song ngữ hai cột tạo Tác phẩm hoàn chỉnh** với segment nguồn và đích đã khớp cặp, mọi câu ở trạng thái chưa xác nhận.

- **CAP-2 — Workspace**
  - **intent:** Người dịch làm trọn một vòng dịch trong **một** cửa sổ bốn panel, thay cho bốn năm cửa sổ rời. `FR16–FR26, FR42, FR44, FR78, FR117`
  - **success:** Source · Lookup · AI Translation · Editor dock/undock/gộp tab/ẩn hoàn toàn được, bố cục khôi phục sau khi khởi động lại và lưu được nhiều preset; bôi đen ở bất kỳ panel nào cho kết quả tra cứu ở Panel Lookup mà không copy/paste; Editor phân đoạn ở cấp câu, gộp và tách segment bằng tay được; mọi phím tắt cấu hình lại được; **một vòng dịch hoàn chỉnh chạy được từ đầu tới cuối mà không chạm chuột**.

- **CAP-3 — Embedded Dictionary & Lookup**
  - **intent:** Người dịch tra cứu tức thì, hoàn toàn ngoại tuyến, và luôn nhìn thấy định nghĩa đó đến từ đâu. `FR27–FR41`
  - **success:** 100% mục từ hiển thị nguồn, không có chế độ ẩn nguồn; nguồn bất đồng thì hiển thị đồng thời cả hai; truy vấn tiếng Trung 1 ký tự, 2 ký tự và 3+ ký tự **đều** trả kết quả khác rỗng; độ trễ Auto-Lookup đầu-cuối p95 < 100 ms; ngắt mạng không ảnh hưởng bất kỳ đường tra cứu nào; **xoá một file lớp gỡ rời rồi chạy lại bộ test tra cứu vẫn xanh**.

- **CAP-4 — Glossary & thuật ngữ**
  - **intent:** Người dịch chốt thuật ngữ một lần và để hệ thống ép AI tuân theo ở mọi lần sau. `FR46–FR55, FR79, FR113, FR114`
  - **success:** Thêm thuật ngữ từ bất kỳ panel nào không phải rời màn hình đang làm việc; thuật ngữ được đánh dấu trực quan trong panel Source, mục **chờ chốt bản dịch** phân biệt được với mục đã chốt; ba cơ chế đề xuất (quét khi nhập, duyệt hàng loạt, thu hoạch từ bản review) chỉ ghi vào bảng chờ — **không đường mã nào ghi thẳng vào Glossary**; duyệt ứng viên tiếng Trung nhận **cả bản dịch âm Hán Việt đề xuất** bằng một phím và chạy được ngoại tuyến; Glossary và bộ prompt xuất/nhập round-trip qua định dạng văn bản mở.

- **CAP-5 — Translation Memory & tái sử dụng**
  - **intent:** Người dịch không phải dịch lại, và không phải tra lại, thứ mình đã dịch. `FR56–FR64, FR118`
  - **success:** Xác nhận một segment ghi cặp *(nguồn → đích)* vào TM mà không thao tác thủ công nào; khớp tuyệt đối được điền sẵn nhưng vẫn ở trạng thái **chưa xác nhận**; khớp mờ hiển thị phần trăm và diff khác biệt; concordance trả kết quả vào Panel Lookup; cùng một segment nguồn có nhiều bản dịch thì giữ tất cả kèm ngày, không ghi đè; TM xuất được TMX mở được ở CAT tool khác; **mỗi cặp TM mang xuất xứ** và Smart RAG **ưu tiên cặp của chính người dùng** — kho TM không bị trộn phong cách khi người dùng biên tập bản dịch của người khác.

- **CAP-6 — AI mở & Smart RAG Injector**
  - **intent:** AI đề xuất bản dịch dưới quyền quyết định của người biên tập, và học phong cách của chính người dùng qua Glossary + TM. `FR65–FR77`
  - **success:** BYOK và local LLM (Ollama, LM Studio) dùng **chung một đường cấu hình**; người dùng xem được prompt cuối cùng đã gửi kèm toàn bộ phần chèn động; kết quả hiện ở panel AI Translation và **không tự động ghi vào Editor**; kết quả stream dần và huỷ được giữa chừng; lỗi mạng không mất công việc đang làm và **hệ thống không tự thử lại**; API key nằm trong keychain hệ điều hành, không đi qua IPC; **gỡ toàn bộ cấu hình AI thì CAP-1..CAP-5 và CAP-8 vẫn chạy đầy đủ**, cưỡng chế bằng test tự động.

- **CAP-7 — AI Proofreader**
  - **intent:** Người dịch bắt lỗi chính tả, ngữ pháp và sai lệch nghĩa trước khi bàn giao. `FR80–FR86`
  - **success:** Chạy **theo yêu cầu** trên segment/Chương/vùng chọn, không chạy nền; mỗi phát hiện có loại lỗi · vị trí · giải thích ngắn · đề xuất sửa, chấp nhận hoặc bỏ qua từng cái; đánh dấu *"không phải lỗi"* thì lần quét sau không báo lại trong cùng Tác phẩm; **không tự sửa văn bản**; kết quả hiển thị ngay tại chỗ trên Editor; tỷ lệ báo động giả đủ thấp để người dùng không tắt hẳn tính năng.

- **CAP-8 — Cầu nối Reviewer**
  - **intent:** Người dịch trao đổi bản dịch với reviewer **không cài app**, hấp thụ được bài học từ bản họ sửa, và lấy ra được bản sạch để **đăng bài**. `FR87–FR95, FR121`
  - **success:** Xuất `.docx` bảng hai cột đối xứng theo segment, và `.md`/text thuần bảo lưu liên kết ảnh cùng alt-text **đã dịch**; **`.docx` một khối đối xứng theo đoạn — một hàng duy nhất cho cả Chương, không đường kẻ ngang — bôi đen cột phải dán sang trình soạn thảo website ra văn bản liền mạch không mảnh vụn bảng biểu**, và màn hình xuất nói rõ ngay lúc chọn rằng định dạng này không nhập lại được, cảnh báo trước khi xuất nếu còn câu chưa xác nhận nhưng **không đánh dấu chúng trong file**; nhập lại file reviewer đã sửa khớp cấu trúc đoạn, segment không khớp được **hiện ra cho người dùng nối tay**; Review Mode hai cửa sổ side-by-side ẩn văn bản gốc, bôi màu thêm/xoá/sửa, chấp nhận từng thay đổi một; **nhập bản review kích hoạt thu hoạch thuật ngữ ngay cả khi người dùng không bao giờ mở Review Mode**.

- **CAP-9 — Dự án & dữ liệu**
  - **intent:** Dữ liệu người dùng sống lâu hơn phần mềm và mang đi được, không khoá vào công cụ. `FR96–FR104`
  - **success:** Copy một `.atproj` sang máy khác mở được nguyên vẹn kèm ảnh, lịch sử phiên bản, Glossary và TM của nó; **xoá chỉ mục Library rồi quét lại phục hồi đầy đủ, không mất dữ liệu**; sao lưu bằng copy thư mục là đủ, không cần thao tác export riêng; auto-save chạy mà không frame nào vượt 50 ms trong lúc gõ; khôi phục được phiên bản cũ của từng segment; không có luồng dữ liệu nào ra ngoài ngoài lời gọi AI người dùng chủ động.

- **CAP-10 — Phát hành & tin cậy**
  - **intent:** Người dùng phổ thông cài được và tin được một bản phát hành **không ký số**. `FR105–FR112`
  - **success:** Bản cài macOS và Windows trên GitHub Releases kèm checksum SHA-256, build công khai qua GitHub Actions để kiểm chứng binary khớp mã nguồn; hướng dẫn cài có ảnh chụp màn hình xử lý tường minh Gatekeeper và SmartScreen; màn hình Attribution dựng từ các file dữ liệu **có mặt** nên không để lại ghi công mồ côi; cơ chế cập nhật **chỉ kiểm tra và thông báo**; gỡ một nguồn dữ liệu khỏi bản phát hành kế tiếp = xoá một file, không đổi mã nguồn.

## Constraints

- **Local-first tuyệt đối.** Không tài khoản, không đăng nhập, không cloud sync, không telemetry. Toàn ứng dụng có **đúng hai** điểm ra mạng: lời gọi AI do người dùng chủ động kích hoạt, và kiểm tra phiên bản mới. Không có điểm thứ ba — kể cả CDN font, ảnh ngoài hay crash reporter.
- **Ứng dụng phải hoạt động đầy đủ khi không cấu hình AI.** Mọi năng lực ngoài CAP-6 và CAP-7 chạy được mà không cần một API key nào.
- **Toàn bộ dữ liệu từ điển nhúng trong bản cài.** Ngân sách 150–200 MB, **không có cơ chế tải thêm sau khi cài đặt**. Tra cứu hoạt động 100% ngoại tuyến.
- **Mọi định nghĩa phải hiển thị nguồn — không ngoại lệ, không chế độ ẩn.** Khi các nguồn bất đồng, hiển thị đồng thời; trong toàn hệ thống không tồn tại bước hợp nhất nguồn. Mỗi nguồn có khiếm khuyết riêng đã biết, nên một công cụ hợp nhất mọi từ điển thành một câu trả lời duy nhất là một công cụ giấu đi sai sót.
- **Không cơ chế nào tự ghi vào Glossary; Proofreader không được tự sửa văn bản.** Mọi đề xuất tự động đi vào bảng chờ và chỉ chuyển sang Glossary bằng thao tác duyệt của người dùng.
- **Nguồn từ điển đóng gói theo mô hình "nền có giấy phép sạch + lớp gỡ rời".** Gỡ bất kỳ lớp gỡ rời nào không được làm hỏng chức năng tra cứu — điều kiện để chính sách gỡ bỏ dữ liệu thực thi được mà không đổi mã. Chi tiết: `data-sources.md`.
- **Giấy phép dự án GPL v3.** Mọi crate và thư viện phải được rà tương thích GPL v3 **trước khi** đưa vào dự án. Chọn v3 vì tương thích crate Apache-2.0 — phủ gần trọn hệ sinh thái Rust.
- **Dữ liệu HVTĐTD không thuộc GPL v3.** © Đặng Thế Kiệt, dùng theo phép riêng tác giả cấp bằng văn bản (2026-08-02) — GPL không áp được lên phần dự án không sở hữu. Phải ghi rõ trong `LICENSE`/`NOTICE` và màn hình Attribution, và vẫn đóng gói làm **lớp gỡ rời** vì đây là phép sử dụng, không phải giấy phép mở. **Lớp này đóng gói vào bản phát hành theo mặc định cho phép, gỡ khi tác giả yêu cầu** — cùng tư thế phản ứng đã chọn cho Thiều Chửu và Cổ hán văn (`data-sources.md`).
- **Sàn khả năng tiếp cận:** mọi thao tác làm được **hoàn toàn bằng bàn phím**, trạng thái focus luôn nhìn thấy rõ, tương phản đạt WCAG AA ở **cả hai** chế độ sáng và tối. Áp từ Giai đoạn 1, cùng lý do với việc tách chuỗi giao diện: rẻ nếu làm từ đầu, rất đắt nếu làm sau.
- **Cửa sổ mất dữ liệu tối đa 5 giây.** App sập giữa phiên gõ không được làm mất quá 5 giây công việc — và phải đạt điều đó mà không vi phạm ngưỡng auto-save không gián đoạn.
- **Không có kinh phí ký số.** Mọi bản phát hành không ký, không notarize. Hệ quả bắt buộc: **cấm cơ chế tự động tải và tự động cài bản cập nhật** — không có chữ ký thì không có gì xác minh được bản tải về là chính chủ.
- **API key lưu trong keychain / credential manager của hệ điều hành.** Không bao giờ ghi vào file cấu hình, file dự án hay log; không đi qua ranh giới IPC.
- **Chỉ mục tìm kiếm chính phải phân biệt dấu.** Chế độ xoá dấu chỉ tồn tại như chỉ mục **phụ** cho tìm kiếm khoan dung, không bao giờ là mặc định — gộp `má / ma / mà / mả / mã / mạ` là lỗi phá vỡ độ chính xác của một công cụ dịch tiếng Việt.
- **Thứ tự hy sinh panel khi cửa sổ hẹp là quyết định, không phải con số hiệu chỉnh được.** Đề xuất AI nhường trước; Tra cứu nhường sau nhưng **rút về thanh trạng thái, không bao giờ mất hẳn**; cặp **Nguyên văn | Bản dịch không bao giờ nhường**. Bốn ngưỡng kích hoạt (A11) sẽ được đo lại trên máy thật và đổi số (Q9) — **thứ tự này thì không đổi theo**.
- **Segment là một câu**, và là đơn vị của Translation Memory cũng như của luồng xác nhận. Đoạn không dùng được: hai đoạn văn giống hệt nhau gần như không tồn tại, nên TM sẽ hầu như không bao giờ khớp.
- **Giao diện v1 chỉ tiếng Việt, nhưng toàn bộ chuỗi giao diện phải nằm ngoài mã nguồn, trong file tài nguyên riêng, ngay từ dòng code đầu tiên.** Rẻ nếu làm từ đầu, rất đắt nếu làm sau.
- **Mô hình tra cứu không được lệch xa QuickTranslator.** Cộng đồng đã quen với nó — vừa là lợi thế (không phải dạy lại) vừa là ràng buộc: lệch quá xa sẽ bị từ chối. Áp lên CAP-2 và CAP-3 mạnh hơn mọi nhóm năng lực khác.
- **Ngôn ngữ nguồn cố định cho từng Tác phẩm**, đặt lúc tạo, không đổi được về sau.
- **Nền tảng: desktop native macOS và Windows, hành vi tương đương trên cả hai.**
- **v1 gồm trọn mười nhóm năng lực.** Không mốc trung gian nào được coi là "xong". Đây là quyết định có ý thức của chủ dự án và là rủi ro lớn nhất của dự án (R1) — xử lý bằng **trình tự** (`build-sequence.md`), không bằng cắt phạm vi.

## Non-goals

- Cặp ngôn ngữ khác ngoài **Anh → Việt** và **Trung → Việt**.
- Cloud sync, tài khoản người dùng, real-time collaboration.
- Bản web và bản mobile.
- Dịch tự động hàng loạt không có người biên tập — không có luồng nào dịch xong mà không qua tay người.
- Ký số / notarization bản phát hành ở v1.
- Định dạng gói và hạ tầng phân phối cho chia sẻ cộng đồng. Chia sẻ chỉ qua trao đổi file: Glossary CSV/TSV, prompt file văn bản, TM qua TMX. Không server, không tài khoản.
- Cơ chế tải từ điển sau khi cài.
- Bản giao diện tiếng Anh ở v1.
- User Journey trong tầng đặc tả này — AuraTranslate là công cụ chuyên nghiệp một người vận hành, hình dạng đúng là *capability spec*. `bmad-ux` tự dựng hành trình từ FR; ba luồng trọng yếu ghi ở cuối `requirements.md`.

## Success signal

Một người dịch Việt trên **macOS** nhập trọn một bộ 2000 chương, dịch xong một Chương từ đầu tới cuối trong **một cửa sổ duy nhất** — tra cứu ngoại tuyến tức thì có ghi nguồn, AI đề xuất theo đúng Glossary và văn phong tích luỹ của chính họ — rồi xuất `.docx` cho reviewer, nhập bản đã sửa về, và **Glossary tự dày lên từ chính những sửa đổi đó mà không cần họ mở Diff Viewer**. Toàn bộ chặng này không có tài khoản, không có byte nào rời khỏi máy ngoài lời gọi AI họ chủ động bấm, và copy thư mục `.atproj` sang máy khác mở lại được nguyên vẹn.

## Assumptions

- **A1** — Vòng IPC Tauri thật và thời gian render frontend nằm gọn trong ngân sách 100 ms của độ trễ Auto-Lookup. Backend đã đo p50 0,022 ms · p95 0,046 ms, payload 679 byte; nếu chậm, nguyên nhân sẽ ở frontend.
- **A2** — Ngân sách 150–200 MB đủ cho toàn bộ nguồn từ điển. Đo được 130 MB với ba nguồn đầu; Unihan, Thiều Chửu, Cổ hán văn, VietPhrase chưa nạp thử.
- **A3** — Bản Thiều Chửu số hoá và bản Cổ hán văn dùng được về mặt pháp lý. **Giả định này sẽ KHÔNG được kiểm chứng trước khi phát hành** — quyết định có ý thức của chủ dự án ngày 2026-08-02.
- **A4** — Tách câu tự động đúng ở tỷ lệ chấp nhận được. Gộp/tách tay là đường lui, nhưng nếu sai quá nhiều thì thao tác thủ công sẽ nuốt hết giá trị của TM.
- **A5** — Người dùng sẵn sàng vượt qua cảnh báo Gatekeeper/SmartScreen để cài. Không kiểm soát được bằng thiết kế.
- **A6** — Ngưỡng tìm kiếm Library p95 < 500 ms trên 5.000 Chương là hợp lý. Ngưỡng tạm, đặt bằng phán đoán kỹ thuật.
- **A7** — Ngưỡng khởi động < 3 giây trên 5.000 Chương là hợp lý. Ngưỡng tạm.
- **A8** — Ngưỡng bộ nhớ nhàn rỗi < 300 MB là hợp lý. Ngưỡng tạm.
- **A9** — Người dùng thật sự cần lịch sử tra cứu và ghim mục từ. Suy đoán từ thói quen dùng; bỏ đi không ảnh hưởng nhóm năng lực nào khác.
- **A10** — Ngưỡng khởi điểm 5 lần lặp cho ứng viên Glossary là hợp lý. Cấu hình lại được nên sai không gây thiệt hại lâu dài.
- **A11** — Bốn ngưỡng bố cục màn hình hẹp là hợp lý, đo theo **vùng làm việc** (chiều cao cửa sổ trừ thanh tiêu đề và thanh trạng thái) chứ không theo kích thước màn hình: **≥ 1100×820** giữ 2×2 · **< 820 cao** gộp hàng dưới thành một panel có tab · **< 1100 rộng hoặc < 700 cao** chỉ còn Nguyên văn | Bản dịch, Tra cứu rút về ngăn kéo · **< 860 rộng** báo không hỗ trợ. Ngưỡng tạm đặt trên mockup, chưa chạy máy thật (Q9). Xem `EXPERIENCE.md`.

## Open Questions

- **Q1** — Vì sao người dịch không xem lại bản review? Nguyên nhân gốc chưa xác định. Để ngỏ có chủ ý; thu hoạch thuật ngữ độc lập với Diff Viewer khiến câu trả lời không chặn tiến độ. *Chủ: chủ dự án.*
- **Q4** — Hiệu chỉnh ba ngưỡng tạm A6, A7, A8. Đóng bằng đo trên thư viện thật ở Giai đoạn 3. *Chủ: chủ dự án.*
- **Q5** — Baseline cho hai counter-metric (tỷ lệ chấp nhận thẳng bản dịch AI không sửa; thời gian quản lý công cụ thay vì dịch). Cần vài tháng dùng thật. *Chủ: chủ dự án.*
- **Q9** — Hiệu chỉnh bốn ngưỡng bố cục màn hình hẹp (A11). Đóng bằng **đo trên máy thật khi có bản chạy được**, từ Giai đoạn 2 ngay khi Workspace bốn panel dựng xong. Không chặn tiến độ — ngưỡng tạm vẫn nghiệm thu được. Chỉ **số** được hiệu chỉnh; thứ tự hy sinh panel là ràng buộc, không hiệu chỉnh. *Chủ: chủ dự án.*
*(**Q2** đóng 2026-08-02: v1 chỉ tiếng Việt, chuỗi giao diện tách file tài nguyên từ đầu. **Q3** đóng 2026-08-02: tác giả Đặng Thế Kiệt đã đồng ý bằng văn bản. **Q6** và **Q7** đóng 2026-08-02: thành NFR17 và NFR18. **Q8** đóng 2026-08-02: mặc định cho phép đóng gói lớp HVTĐTD vào bản phát hành, gỡ khi tác giả yêu cầu.)*
