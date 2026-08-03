---
stepsCompleted: [1, 2, 3, 4, 5, 6]
status: complete
readiness: READY
issues:
  critical: 0
  major: 5
  minor: 6
coverage:
  frCoverage: 131/131
  nfrCoverage: 19/19
requirementsExtracted:
  functional: 131
  nonFunctional: 19
documentsIncluded:
  prd: _bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md
  prd_addendum: _bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/addendum.md
  architecture: _bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md
  epics: _bmad-output/planning-artifacts/epics.md
  ux_experience: _bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md
  ux_design: _bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/DESIGN.md
  spec: _bmad-output/specs/spec-AuraTranslate/SPEC.md
  brief: _bmad-output/planning-artifacts/briefs/brief-AuraTranslate-2026-08-02/brief.md
---

# Implementation Readiness Assessment Report

**Date:** 2026-08-03
**Project:** AuraTranslate

## 1. Document Inventory

### PRD

**Whole Documents:**

- `prds/prd-AuraTranslate-2026-08-02/prd.md` (112K, 2026-08-03 09:22) — **CHỌN**
- `prds/prd-AuraTranslate-2026-08-02/addendum.md` (12K, 2026-08-02 17:56) — phụ lục, đọc kèm
- `prds/prd-AuraTranslate-2026-08-02/review-rubric.md` (16K) — rubric review, không phải nguồn yêu cầu

**Sharded Documents:** không có

### Architecture

**Whole Documents:**

- `architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md` (76K, 2026-08-03 09:28) — **CHỌN**
- `architecture/.../reviews/review-adversarial-2026-08-03b.md`, `review-rubric-2026-08-03b.md`, `review-version-2026-08-03b.md` — hồ sơ review, tham chiếu phụ

**Sharded Documents:** không có

### Epics & Stories

**Whole Documents:**

- `epics.md` (292K, 2026-08-03 12:02) — **CHỌN** — 10 epic, 128 story

**Sharded Documents:** không có

### UX Design

**Whole Documents:**

- `ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md` (52K, 2026-08-03 09:12) — **CHỌN**
- `ux-designs/ux-AuraTranslate-2026-08-02/DESIGN.md` (20K, 2026-08-03 00:30) — **CHỌN**

**Sharded Documents:** không có

### Tài liệu bổ trợ (ngoài `planning_artifacts`)

- `_bmad-output/specs/spec-AuraTranslate/SPEC.md` + `requirements.md`, `build-sequence.md`, `risks.md`, `data-sources.md`, `glossary.md`
- `planning-artifacts/briefs/brief-AuraTranslate-2026-08-02/brief.md` + `addendum.md`
- `planning-artifacts/research/` — 3 tài liệu nghiên cứu, gồm `phase-0-spike-results-2026-08-02.md`

### Vấn đề phát hiện

- **Duplicate:** không có. Mỗi loại tài liệu chỉ tồn tại ở một định dạng (whole), không có bản sharded song song.
- **Missing:** không có tài liệu bắt buộc nào thiếu. `{project_knowledge}` (`docs/`) trống — không ảnh hưởng.
- **Lưu ý:** các file `.memlog.md` là nhật ký phiên làm việc của workflow, không dùng làm nguồn yêu cầu.

---

## 2. PRD Analysis

**Nguồn:** `prds/prd-AuraTranslate-2026-08-02/prd.md` (1.070 dòng) + `addendum.md` (186 dòng).
PRD là dạng **capability spec** (§13 Ghi chú bàn giao: không có User Journey — lựa chọn có chủ ý).

### 2.1 Functional Requirements

Tổng cộng **131 FR**, đánh số **FR1–FR131 liên tục không đứt quãng**, không có FR nào được tham chiếu mà không được định nghĩa. Nhóm theo 10 năng lực C1–C10.

**C1 — Library**

- **FR1.** Library tổ chức theo **hai tầng: Tác phẩm → Chương**. Một Tác phẩm tương ứng một dự án dịch; một Chương là một đơn vị dịch có văn bản nguồn và văn bản đích riêng.
- **FR2.** Tài liệu đơn lẻ (hợp đồng, bài báo, tài liệu kỹ thuật ngắn) được biểu diễn là một **Tác phẩm có đúng một Chương**. Không có loại thực thể thứ ba.
- **FR3.** Mỗi Tác phẩm mang metadata: tên, ảnh bìa *(tuỳ chọn)*, **ngôn ngữ nguồn** *(cố định cho toàn Tác phẩm, đặt lúc tạo)*, lĩnh vực/thể loại, ngày tạo, ngày sửa gần nhất.
- **FR4.** Glossary và Translation Memory gắn ở **tầng Tác phẩm** — mọi Chương trong cùng Tác phẩm dùng chung.
- **FR5.** Trạng thái vòng đời có ở cả hai tầng, **bốn giá trị**: **Chưa bắt đầu / Đang dịch / Tạm ngưng / Đã xong**. Chương mới nhập mặc định là *Chưa bắt đầu*.
- **FR6.** Trạng thái Tác phẩm được **suy ra tự động** từ trạng thái các Chương, nhưng người dùng **ghi đè thủ công được** — *Tạm ngưng* ở tầng Tác phẩm là quyết định của người, hệ thống không suy ra được.
- **FR7.** Mỗi Tác phẩm hiển thị tiến độ: số Chương đã xong trên tổng số, kèm thanh tiến độ trực quan.
- **FR8.** Full-text search **xuyên toàn bộ Library**: tìm đồng thời trong văn bản nguồn và văn bản dịch của mọi Tác phẩm. Kết quả trả về kèm Tác phẩm, Chương và đoạn văn bản khớp.
- **FR9.** Tìm kiếm có **hai chế độ**: *chính xác dấu* (mặc định) và *khoan dung không dấu*. Hệ thống thử chế độ chính xác trước, chỉ nới lỏng khi không có kết quả hoặc khi người dùng yêu cầu.
- **FR10.** Lọc và sắp xếp Library theo trạng thái, lĩnh vực, ngôn ngữ nguồn và ngày sửa gần nhất.
- **FR11.** **Chế độ đọc (Reading Mode):** đọc bản dịch đã hoàn thành, đọc liên tục qua nhiều Chương, **không hiển thị công cụ biên tập**. Mặc định chỉ hiển thị bản dịch tiếng Việt; có **công tắc bật chế độ song ngữ** khi người dùng muốn đối chiếu. Phạm vi và thao tác của Chế độ đọc: xem FR119 (đánh dấu chỗ cần sửa) và FR120 (chỉ đọc phần đã xong, dừng ở biên). Bound tối thiểu cho "tối ưu cho việc đọc dài": - **Độ rộng dòng giới hạn** (không kéo hết chiều ngang màn hình) - **Cỡ chữ và chiều cao dòng chỉnh được** - **Chế độ sáng và tối**
- **FR12.** Mở một Chương từ Library đưa thẳng vào Workspace, đúng Chương đó, khôi phục vị trí làm việc lần trước.
- **FR13.** Tạo Tác phẩm mới từ file (`.txt`, `.docx`, `.md`) hoặc từ văn bản dán trực tiếp.
- **FR14.** **Nhập hàng loạt:** chọn nhiều file cùng lúc, hoặc tách một file lớn thành nhiều Chương theo mẫu phân tách do người dùng cấu hình (mẫu tiêu đề hoặc biểu thức chính quy), **có màn hình xem trước kết quả tách trước khi xác nhận**.
- **FR15.** Sau khi nhập: đổi tên, sắp xếp lại thứ tự, gộp và tách Chương.

**C2 — Workspace**

- **FR16.** **Bốn panel trong một cửa sổ ứng dụng duy nhất:** *Source*, *Lookup*, *AI Translation*, *Editor*. Đây là câu trả lời trực tiếp cho nỗi đau "bốn đến năm cửa sổ mở cùng lúc".
- **FR17.** Panel hỗ trợ kéo thả để dock/undock, gộp thành tab, và thay đổi kích thước. Mỗi panel **ẩn được hoàn toàn** — người dịch không dùng AI phải giấu được panel AI Translation.
- **FR18.** Bố cục workspace được lưu và khôi phục giữa các phiên làm việc. Hỗ trợ lưu nhiều **preset bố cục** và chuyển nhanh giữa chúng.
- **FR19.** Panel Source hiển thị văn bản gốc (Anh hoặc Trung) kèm **tab Hán Việt** cho tài liệu tiếng Trung — xem ở chế độ chuyển đổi hoặc song song.
- **FR20.** **Sync Scrolling** đồng bộ vị trí cuộn giữa Source, AI Translation và Editor. Có công tắc bật/tắt rõ ràng.
- **FR21.** **Auto-Lookup:** bôi đen một cụm từ ở Source, AI Translation hoặc Editor → kết quả tra cứu hiện **ngay** ở panel Lookup. Không copy, không paste, không chuyển cửa sổ.
- **FR22.** **Global Hotkeys** cho các thao tác lặp lại: dịch segment hiện tại, chuyển focus giữa các panel, xác nhận segment, tra cứu cụm đang chọn, bật/tắt sync scroll. **Toàn bộ phím tắt cấu hình lại được.**
- **FR23.** Editor phân đoạn văn bản thành **segment ở cấp độ câu**. Segment là đơn vị của Translation Memory (C5) và của luồng xác nhận. - Tiếng Trung: tách theo `。！？；` - Tiếng Anh: tách theo `. ! ?`, có xử lý các trường hợp viết tắt không phải kết câu **[A4]**
- **FR24.** Người dùng **xác nhận từng segment**. Segment đã xác nhận được đánh dấu trực quan phân biệt với segment đang dở.
- **FR25.** Điều hướng nhanh giữa các segment: kế tiếp, trước đó, và **segment chưa dịch kế tiếp**.
- **FR26.** Chuyển Chương ngay trong Workspace (Chương trước / Chương sau) mà không phải quay về Library.

**C3 — Embedded Dictionary & Lookup**

- **FR27.** Toàn bộ dữ liệu từ điển **nhúng trong bản cài**. Tra cứu hoạt động 100% offline, không có cơ chế tải thêm sau khi cài đặt.
- **FR28.** Panel Lookup hiển thị một **bản ghi có cấu trúc**, không phải một đoạn văn bản. Mỗi mục gồm: **nguồn · từ loại · nghĩa · ví dụ[] · trích dẫn[] · ghi chú**.
- **FR29.** Một từ có nhiều từ loại phải hiện thành **nhiều mục riêng biệt**, mỗi mục có ví dụ riêng. Ví dụ: cùng một chữ dùng làm động từ và làm phó từ là hai mục, không phải một chuỗi nghĩa gộp.
- **FR30.** **Ví dụ gắn với từng từ loại**, không gắn với cả từ. **Trích dẫn** là trường riêng biệt với ví dụ: trích dẫn có xuất xứ văn bản.
- **FR31.** **Mọi định nghĩa phải hiển thị nguồn của nó. Không có ngoại lệ, không có chế độ ẩn nguồn.**
- **FR32.** Khi các nguồn bất đồng về một mục từ, hệ thống **hiển thị đồng thời cả hai**, không hợp nhất thành một câu trả lời duy nhất.
- **FR33.** **Tab Hán Việt:** hiển thị âm Hán Việt cho từng ký tự tiếng Trung trong văn bản nguồn.
- **FR34.** Mục từ **tiếng Anh** phải có nhãn từ loại và nghĩa tiếng Việt.
- **FR35.** Mục từ **tiếng Trung** phải có nhãn từ loại và ít nhất một ví dụ cách dùng khi nguồn có dữ liệu. Ở v1, nhãn từ loại và bản dịch ví dụ **bằng tiếng Anh được chấp nhận** và phải được **đánh dấu rõ là nhãn ngoại ngữ**.
- **FR36.** Các nguồn từ điển được đóng gói theo mô hình **"nền có giấy phép sạch + lớp gỡ rời được"**. Gỡ bỏ bất kỳ lớp gỡ rời nào **không được làm hỏng chức năng tra cứu** — sản phẩm vẫn hoạt động đầy đủ trên các lớp nền.
- **FR37.** Người dùng bật/tắt từng nguồn từ điển trong panel Lookup.
- **FR38.** Ghi công đầy đủ từng nguồn từ điển: trong ứng dụng (màn hình Attribution) và trong bản phát hành.
- **FR39.** Tra cứu tiếng Trung phải **trả về kết quả cho truy vấn 1 ký tự, 2 ký tự và 3 ký tự trở lên**.
- **FR40.** Tra cứu tiếng Anh nhận diện biến thể hình thái của từ. **Giới hạn đã biết và chấp nhận:** đây là *stemming*, không phải *lemmatization* thật — hệ sinh thái Rust chưa có lemmatizer trưởng thành. Đủ cho khớp Glossary, không xử lý được các biến thể bất quy tắc.
- **FR41.** Lịch sử tra cứu trong phiên làm việc, và ghim mục từ để tra lại nhanh. **`[A9]`**

**C2 — Workspace**

- **FR42.** Panel Source hiển thị hình ảnh nhúng **đúng vị trí** của chúng trong văn bản gốc.

**C1 — Library**

- **FR43.** Chế độ đọc hiển thị hình ảnh nhúng **đúng vị trí** của chúng trong văn bản.

**C2 — Workspace**

- **FR44.** **Alt-text của hình ảnh là một segment dịch được** — tham gia Translation Memory, Glossary và luồng xác nhận như mọi segment khác. Đây là điều kiện để yêu cầu "bảo lưu liên kết hình ảnh và Alt-text" khi xuất `.md` (C8) có nghĩa thật: alt-text phải được *dịch*, không chỉ được *giữ lại*.

**C1 — Library**

- **FR45.** Hình ảnh được **lưu bên trong `.atproj`**, không phụ thuộc đường dẫn ngoài. Một Tác phẩm phải mang đi được nguyên vẹn khi copy sang máy khác.

**C4 — Glossary & thuật ngữ**

- **FR46.** Glossary có **hai tầng**: *toàn cục* (dùng chung mọi Tác phẩm) và *theo Tác phẩm* (tên nhân vật, địa danh, công pháp, thuật ngữ chuyên ngành của riêng tài liệu đó). Khi một thuật ngữ tồn tại ở cả hai tầng, **tầng Tác phẩm thắng**.
- **FR47.** Mỗi mục Glossary gồm: thuật ngữ nguồn, bản dịch, ghi chú, phân loại *(tên người / địa danh / thuật ngữ chuyên ngành / khác)*, ngày thêm, và **xuất xứ** — *nhập thủ công / đề xuất khi nhập tài liệu / thu hoạch từ bản review*.
- **FR48.** Thêm nhanh vào Glossary từ **bất kỳ panel nào**: bôi đen cụm từ → thêm thuật ngữ → chọn tầng. Không phải rời màn hình đang làm việc.
- **FR49.** Quản lý Glossary: tìm kiếm, sửa, xoá, **nhập và xuất** dưới định dạng văn bản mở (CSV/TSV) để chia sẻ giữa người dịch.
- **FR50.** Mọi thuật ngữ có trong Glossary được **đánh dấu trực quan trong panel Source**, để người dịch thấy ngay câu đang dịch chứa thuật ngữ nào. Mục **chờ chốt bản dịch** (FR114) cũng được đánh dấu, nhưng **phân biệt được** với mục đã chốt — đó chính là cơ hội để người dịch chốt nó.
- **FR51.** Khớp thuật ngữ **phân theo ngôn ngữ**: tiếng Trung dùng khớp chính xác; tiếng Anh dùng khớp mờ ở cấp hình thái từ (stemming) để bắt được các biến thể.
- **FR52.** **Quét khi nhập tài liệu:** tìm ứng viên thuật ngữ = chuỗi lặp lại **từ 5 lần trở lên** *và* **không có trong từ điển nhúng**. **[A10]** Ngưỡng này **cấu hình lại được** — 5 là điểm khởi đầu, không phải hằng số. - Tiếng Trung: chuỗi ký tự lặp không có trong từ điển; đối chiếu danh sách họ phổ biến để đoán tên người. - Tiếng Anh: cụm viết hoa không đứng đầu câu.
- **FR53.** **Duyệt hàng loạt:** ứng viên hiện thành danh sách xếp theo tần suất, kèm **số lần xuất hiện** và **ví dụ ngữ cảnh**. Người dùng duyệt hoặc bỏ bằng thao tác một phím — **không phải gõ**. Phím nhận đồng thời nhận **cả bản dịch đề xuất** khi có (FR113); phân loại đổi được bằng phím số. Duyệt phải **dừng giữa chừng và mở lại đúng chỗ** — một lần nhập lớn sinh ra hàng trăm ứng viên, đây là việc của nhiều buổi.
- **FR54.** **Thu hoạch từ bản review:** khi nhập lại bản Reviewer đã sửa (C8), nếu phát hiện reviewer đổi thuật ngữ *X* thành *Y* một cách **nhất quán**, hệ thống đề xuất bổ sung cặp đó vào Glossary. Đề xuất phải nêu rõ **số lần đổi trên tổng số lần xuất hiện** để người dùng tự phán xét mức nhất quán.
- **FR55.** **Mọi đề xuất tự động đều phải qua duyệt của người dùng. Không có cơ chế nào được tự ghi vào Glossary.**

**C5 — Translation Memory & tái sử dụng**

- **FR56.** **Ghi tự động:** mỗi khi người dùng xác nhận một segment ở Editor, cặp *(nguồn → đích)* được ghi vào Translation Memory. **Không có thao tác thủ công nào.** Cặp TM mang theo **xuất xứ** kế thừa từ segment (FR117).
- **FR57.** TM có **phạm vi kép**: TM riêng theo Tác phẩm và TM chung toàn cục — tương ứng hai tầng scope của Glossary.
- **FR58.** **Khớp tuyệt đối (100%):** segment y hệt đã dịch trước đây được **điền sẵn** và **đánh dấu là gợi ý cần xác nhận**. Hệ thống **không** tự coi segment đó là đã hoàn thành.
- **FR59.** **Khớp mờ:** hiển thị các bản dịch cũ tương tự kèm **phần trăm khớp** và **diff phần khác biệt**, để người dùng sửa nhanh thay vì dịch lại từ đầu.
- **FR60.** **Concordance:** tra ngược *"cụm từ này trước đây tôi dịch thế nào?"* trên toàn bộ TM. Kết quả đưa vào **panel Lookup**, cùng chỗ với kết quả từ điển.
- **FR61.** Thuật toán khớp **phân theo ngôn ngữ**: tiếng Trung dùng n-gram ký tự (không có ranh giới từ); tiếng Anh dùng token n-gram sau stemming.
- **FR62.** Xem, sửa và xoá từng mục TM. Danh sách hiển thị **xuất xứ** của từng cặp (FR118) và lọc được theo xuất xứ — để người dùng rà lại hoặc dọn sạch phần không phải văn phong của mình.
- **FR63.** Khi cùng một segment nguồn có **nhiều bản dịch khác nhau**, hệ thống giữ lại tất cả và hiển thị tất cả kèm ngày, thay vì ghi đè. Người dịch tự chọn.
- **FR64.** **Xuất và nhập TMX.** Đây là yêu cầu của trụ định vị #3 — dữ liệu phải sống lâu hơn phần mềm, và người dùng không bị khoá vào `.atproj`.

**C6 — AI mở & Smart RAG Injector**

- **FR65.** **BYOK:** người dùng nhập API key của nhà cung cấp mình chọn.
- **FR66.** **Local LLM:** kết nối tới endpoint tương thích OpenAI (Ollama, LM Studio) qua **cùng một đường cấu hình** với BYOK.
- **FR67.** **API key được lưu trong keychain / credential manager của hệ điều hành.** Không lưu dạng văn bản thuần trong file dự án hay file cấu hình, không đồng bộ đi đâu.
- **FR68.** Cấu hình AI (nhà cung cấp, mô hình, tham số sinh) đặt ở **tầng toàn cục**, **ghi đè được theo từng Tác phẩm**.
- **FR69.** **Custom prompt theo thể loại** (tiên hiệp, khoa học, pháp lý, báo chí…) và theo quy chuẩn dịch. Prompt tồn tại ở cả hai tầng — toàn cục và theo Tác phẩm; **tầng Tác phẩm thắng**.
- **FR70.** Trước mỗi lần gọi AI, hệ thống quét câu nguồn và **chèn động vào prompt**: (a) các thuật ngữ Glossary xuất hiện trong câu, kèm bản dịch đã chốt; (b) các segment tương tự tìm được trong TM.
- **FR71.** Người dùng **xem được prompt cuối cùng đã gửi đi**, bao gồm toàn bộ phần chèn động.
- **FR72.** Kết quả AI hiện ở **panel AI Translation** và **không tự động ghi vào Editor**. Người dùng chủ động đưa sang.
- **FR73.** Dịch theo **từng segment** và theo **lô nhiều segment liên tiếp**, **huỷ được giữa chừng**.
- **FR74.** Kết quả hiện **dần theo dòng chảy (streaming)** khi mô hình đang sinh, không bắt người dùng chờ trọn câu trả lời.
- **FR75.** Khi gặp lỗi mạng hoặc lỗi API: thông báo rõ nguyên nhân, **không mất công việc đang làm**, và cho phép thử lại **do người dùng chủ động**. Hệ thống **không được tự động thử lại nhiều lần** — với BYOK, mỗi lần gọi là tiền của người dùng.
- **FR76.** Hiển thị số token đã dùng và **ước tính chi phí** cho mỗi lần gọi.
- **FR77.** **Ứng dụng phải hoạt động đầy đủ khi không cấu hình AI.** Mọi năng lực ngoài C6 và C7 — Library, Workspace, từ điển, Glossary, TM, export/import — phải chạy được mà không cần một API key nào.

**C2 — Workspace**

- **FR78.** Người dùng **gộp hai segment liền nhau** hoặc **tách một segment** khi máy tách sai. Thao tác này phải có, vì tách câu tự động luôn sai ở một tỷ lệ nhất định — nhất là với dấu chấm trong viết tắt, số thập phân và hội thoại.

**C4 — Glossary & thuật ngữ**

- **FR79.** **Xuất và nhập bộ prompt** dưới dạng file văn bản mở, để người dịch chia sẻ prompt theo thể loại cho nhau.

**C7 — AI Proofreader**

- **FR80.** Quét **chính tả và ngữ pháp tiếng Việt** trên bản dịch của người dùng.
- **FR81.** **Đối chiếu bản dịch với bản gốc**, đánh dấu những đoạn nghi **dịch sai**, **dịch thoát nghĩa quá xa**, hoặc **cấu trúc câu tối nghĩa**.
- **FR82.** Proofreader chạy **theo yêu cầu của người dùng** (trên một segment, một Chương, hoặc vùng đang chọn), **không chạy nền liên tục**.
- **FR83.** Mỗi phát hiện gồm: **loại lỗi**, vị trí, **giải thích ngắn**, và **đề xuất sửa**. Người dùng chấp nhận hoặc bỏ qua **từng phát hiện một**.
- **FR84.** **Bỏ qua có ghi nhớ:** khi người dùng đánh dấu một phát hiện là *"không phải lỗi"*, lần quét sau không báo lại phát hiện đó trong cùng Tác phẩm.
- **FR85.** **Proofreader không được tự sửa văn bản.** Mọi thay đổi phải do người dùng chấp nhận.
- **FR86.** Kết quả proofread hiển thị **ngay tại chỗ trên Editor** (đánh dấu trên đúng đoạn văn), không phải một danh sách rời bắt người dùng tự đối chiếu vị trí.

**C8 — Cầu nối Reviewer: Export / Import / Diff**

- **FR87.** Xuất **`.docx` dạng bảng hai cột**: cột trái văn bản gốc, cột phải bản dịch, **đối xứng theo segment**.
- **FR88.** Xuất **`.md` hoặc text thuần**, **bảo lưu hình ảnh và alt-text đã dịch** (FR44) **cùng chú thích ảnh đã dịch** (FR129). Hình ảnh được tham chiếu theo **kiểu người dùng chọn ở FR130** — *(mệnh đề này làm rõ ngày 2026-08-03: trước FR127 ảnh không nằm trong `.atproj` nên "bảo lưu liên kết hình ảnh" chỉ có một nghĩa; nay nó có hai và phải chọn.)*
- **FR89.** Xuất theo một Chương, nhiều Chương đã chọn, hoặc cả Tác phẩm.
- **FR90.** Nhập lại file `.docx` / `.md` mà Reviewer đã chỉnh sửa, vào đúng Tác phẩm hiện có.
- **FR91.** **Segment alignment:** hệ thống khớp cấu trúc đoạn giữa file nhập và dữ liệu sẵn có. Những segment **không khớp được phải hiện ra cho người dùng nối tay** — mẫu chuẩn của ngành là *máy khớp, người sửa*, không phải máy khớp im lặng.
- **FR92.** **Review Mode:** workspace chuyển sang bố cục **hai cửa sổ side-by-side** — trái là bản dịch của người dùng, phải là bản đã nhập từ Reviewer.
- **FR93.** Trong Review Mode, **ẩn văn bản gốc** và dùng thuật toán diff **bôi màu phần thêm / xoá / sửa** giữa hai bản dịch. Người dùng chỉ cần lướt để đối chiếu.
- **FR94.** Từ Review Mode, **chấp nhận từng thay đổi** vào bản dịch của mình, hoặc bỏ qua.
- **FR95.** Việc nhập bản review **kích hoạt cơ chế thu hoạch thuật ngữ (FR54) một cách độc lập** — kể cả khi người dùng **không bao giờ mở Review Mode**.

**C9 — Dự án & dữ liệu**

- **FR96.** Mỗi Tác phẩm được lưu thành **một `.atproj` trên đĩa**. Đây là **nguồn sự thật**; người dùng copy, sao lưu và di chuyển tự do.
- **FR97.** `.atproj` **tự chứa** mọi thứ thuộc về Tác phẩm: văn bản nguồn, bản dịch, segment, lịch sử phiên bản, Glossary dự án, TM dự án, prompt dự án và hình ảnh. **Copy sang máy khác phải mở được nguyên vẹn.**
- **FR98.** Một **chỉ mục Library trung tâm** phục vụ tìm kiếm xuyên Tác phẩm (FR8). Chỉ mục này **phải dựng lại được hoàn toàn từ các `.atproj`** — mất chỉ mục không được làm mất dữ liệu.
- **FR99.** **Quét lại thư mục:** phát hiện `.atproj` mới xuất hiện, `.atproj` đã bị di chuyển hoặc xoá (mục mồ côi trong Library), và cập nhật chỉ mục tương ứng.
- **FR100.** **Auto-save định kỳ, không gián đoạn UI.** Không được có gai trễ cảm nhận được khi đang gõ.
- **FR101.** **Versioning:** lưu lịch sử các phiên bản dịch của từng segment; xem lại và **khôi phục** được.
- **FR102.** **Sao lưu bằng cách copy thư mục là đủ.** Không được yêu cầu một thao tác export riêng để có bản sao lưu dùng được.
- **FR103.** Mọi cấu hình tồn tại ở **hai tầng**, tầng dự án ghi đè tầng toàn cục:
- **FR104.** **Không telemetry.** Ứng dụng không gửi bất kỳ dữ liệu nào ra ngoài, trừ nội dung mà người dùng **chủ động** gửi cho nhà cung cấp AI đã cấu hình.

**C10 — Phát hành & tin cậy**

- **FR105.** Phát hành bản cài cho **macOS và Windows** qua **GitHub Releases**.
- **FR106.** Công bố **checksum SHA-256** cho mọi artifact phát hành.
- **FR107.** **Build công khai qua GitHub Actions**, để bất kỳ ai cũng kiểm chứng được binary khớp với mã nguồn.
- **FR108.** **Hướng dẫn cài đặt có ảnh chụp màn hình** cho cả hai hệ điều hành, xử lý tường minh cảnh báo Gatekeeper trên macOS *(chuột phải → Mở)* và SmartScreen trên Windows *(More info → Run anyway)*.
- **FR109.** **Màn hình Attribution trong ứng dụng:** liệt kê mọi nguồn từ điển, giấy phép tương ứng và ghi công đầy đủ.
- **FR110.** Kèm văn bản giấy phép **GPL v3** và toàn bộ giấy phép của các bộ dữ liệu trong bản phát hành.
- **FR111.** Cơ chế cập nhật **chỉ kiểm tra và thông báo** phiên bản mới. **Không tự động tải, không tự động cài.**
- **FR112.** **Chính sách gỡ bỏ dữ liệu:** nếu chủ sở hữu một nguồn không xác định được tác giả (VietPhrase) lên tiếng, phải có quy trình gỡ lớp đó khỏi bản phát hành kế tiếp **mà không ảnh hưởng chức năng** — bảo đảm bởi FR36.

**C4 — Glossary & thuật ngữ**

- **FR113.** **Đề xuất bản dịch cho ứng viên:** với ứng viên tiếng Trung, hệ thống đề xuất sẵn bản dịch bằng **âm Hán Việt** của chuỗi đó, lấy từ dữ liệu đã nhúng (FR33) nên **chạy hoàn toàn ngoại tuyến**. Thao tác nhận một phím (FR53) nhận **cả thuật ngữ lẫn bản dịch đề xuất**, và mục vào Glossary ở trạng thái **đã chốt**. Bản dịch đề xuất sửa được về sau như mọi mục khác.
- **FR114.** **Trạng thái *chờ chốt bản dịch*:** khi không đề xuất được — ứng viên tiếng Anh, hoặc chuỗi tiếng Trung mà âm Hán Việt không phải cách dịch phù hợp — thao tác nhận vẫn đưa mục vào Glossary nhưng để trường bản dịch ở trạng thái **chờ chốt**. Lần **đầu tiên** người dịch gặp thuật ngữ đó trong Workspace, hệ thống hỏi **một lần** rồi khoá lại thành đã chốt.

**C1 — Library**

- **FR115.** **Nhập tài liệu song ngữ tạo Tác phẩm hoàn chỉnh:** từ file hai cột — bảng trong `.docx`, bảng trong `.md`, hoặc `.csv`/`.tsv` — trong đó một cột là văn bản gốc và một cột là bản dịch. Người dùng **khai báo cột nào là nguồn, cột nào là đích** và ngôn ngữ nguồn. Kết quả là một Tác phẩm đầy đủ: có segment nguồn, có segment đích, đã khớp cặp. **Bắt buộc có màn hình xem trước trước khi ghi xuống đĩa**, cùng khuôn với FR14. **Ranh giới Chương lấy từ mẫu phân tách của FR14, và mẫu đó áp lên *cột nguồn*.** *(Làm rõ ngày 2026-08-03 — không phải yêu cầu mới, nên FR115 giữ nguyên số và tổng FR không đổi.)* Một file hai cột chứa cả bộ truyện là **một dòng văn bản chưa chia Chương**, đúng hình dạng đầu vào mà FR14 xử lý; người dùng cấu hình mẫu như mọi đường nhập file khác, và màn xem trước hiện số Chương nhận ra được trước khi xác nhận.
- **FR116.** **Khớp câu trong phạm vi từng cặp hàng:** hàng trong bảng hai cột thường là **đoạn**, trong khi segment là **câu** (FR23). Hệ thống tách cả hai phía thành câu và khớp **bên trong từng cặp hàng**. Chỗ số câu hai bên lệch nhau **phải hiện ra cho người dùng nối tay** — cùng mẫu *máy khớp, người sửa* của FR91.

**C2 — Workspace**

- **FR117.** **Xuất xứ bản dịch ở cấp segment**, ba giá trị: *tôi dịch* · *người khác dịch* · *nhập từ tài liệu song ngữ*. Xuất xứ được **suy ra tự động từ hành vi**, không hỏi người dùng:

**C5 — Translation Memory & tái sử dụng**

- **FR118.** **Translation Memory không được trộn phong cách.** Mỗi cặp TM mang xuất xứ *của tôi* hoặc *của người khác*, suy ra từ FR117. **Smart RAG Injector (FR70) ưu tiên cặp *của tôi*;** cặp xuất xứ khác chỉ được chèn khi không đủ cặp của chính người dùng, và khi chèn thì **đánh dấu rõ trong prompt là văn phong tham khảo, không phải văn phong của người dùng**.

**C1 — Library**

- **FR119.** **Đánh dấu chỗ cần sửa khi đang đọc.** Một thao tác đánh dấu câu đang đọc rồi **đọc tiếp ngay** — phiên đọc không bị cắt. Các chỗ đã đánh dấu gom thành **một danh sách theo Tác phẩm**; từ danh sách đó mở Workspace tại đúng segment để sửa một lượt. Có thêm thao tác thứ hai **nhảy thẳng** sang Workspace tại câu đó khi người dùng muốn sửa ngay.
- **FR120.** **Chế độ đọc chỉ đọc phần đã xong, và dừng ở biên một cách tường minh.** - Đọc liên tục qua các Chương ở trạng thái **Đã xong**; chạm Chương chưa xong thì dừng lại ở một **mốc rõ ràng** báo đã hết phần đã dịch, kèm đường sang Workspace để dịch tiếp. - Chương chưa dịch **không hiển thị nguyên văn** — nguyên văn tiếng Trung xen giữa một trang đọc tiếng Việt là phá vỡ trải nghiệm, không phải bổ sung thông tin. - Câu **chưa xác nhận** nằm trong một Chương đã xong **vẫn hiện**, nhưng **có dấu nhẹ phân biệt được**.

**C8 — Cầu nối Reviewer: Export / Import / Diff**

- **FR121.** **Xuất `.docx` một khối, đối xứng theo đoạn — dành cho việc đăng bài.** Vẫn là bảng hai cột, nhưng **một hàng duy nhất cho cả Chương** và **không đường kẻ ngang**: cột trái nguyên văn, cột phải bản dịch, hai ô giữ **đúng số lần xuống đoạn như nhau** để hai bên vẫn đối chiếu được bằng mắt. **Điều kiện nghiệm thu:** bôi đen cột phải rồi dán sang trình soạn thảo của website phải ra **văn bản liền mạch**, không kèm mảnh vụn bảng biểu. Phạm vi xuất theo FR89 như mọi định dạng khác. Hai ràng buộc đi kèm: - **Không nhập lại được.** Định dạng này không giữ ranh giới câu nên FR90/FR91 không áp dụng cho nó. Nó nằm **ở cuối vòng khứ hồi**, cùng nhóm với text thuần — và **màn hình xuất phải nói rõ điều đó ngay lúc chọn định dạng**, không để trong tài liệu hướng dẫn. - **Câu chưa xác nhận không được đánh dấu trong file xuất.** Một nền màu hay một dòng ghi chú xen giữa văn xuôi sẽ đi thẳng vào bài đăng. Thay vào đó, khi phạm vi xuất còn câu chưa xác nhận thì **cảnh báo trước lúc xuất**, để người dùng quyết định.

**C1 — Library**

- **FR122.** **Nhập từ URL bằng danh sách link:** người dùng dán một danh sách link, **mỗi dòng một Chương**, và hệ thống xử lý **đúng thứ tự đã cho**. Tạo Tác phẩm mới, hoặc thêm Chương vào Tác phẩm sẵn có. **Ranh giới cứng:** ứng dụng **không quét trang mục lục**, **không lần theo "chương sau"**, **không tự tìm bất kỳ link nào ngoài danh sách được cấp**. Phạm vi và thứ tự hoàn toàn do người dùng quyết định.
- **FR123.** **Bóc nội dung chính bằng một thuật toán dùng chung, có màn hình xem trước bắt buộc và sửa được bằng tay.** v1 **không có bộ đọc riêng cho từng website** — chủ dự án lấy bài từ bất kỳ site nào, không đoán trước được, nên một thuật toán chung là lời giải duy nhất mở rộng được. Màn xem trước hiện phần đã bóc của **từng Chương** và cho người dùng **sửa ranh giới bóc bằng tay** trước khi ghi xuống đĩa. Cùng khuôn với FR14 và FR115.
- **FR124.** **Luật làm sạch rác nằm trong thân nội dung:** watermark, dòng *"nguồn: xxx.com"*, lời nhắn của người đăng, link quảng cáo chèn giữa văn bản. Luật là một **danh sách mẫu** (chuỗi hoặc biểu thức chính quy) mà người dùng **xem được, sửa được và tắt được**; màn xem trước **hiện những gì sắp bị xoá trước khi xoá**.
- **FR125.** **Chuẩn hoá xuống dòng và khoảng trắng:** gộp dòng bị ngắt tuỳ tiện, xoá dòng trống thừa, thống nhất cách phân đoạn.
- **FR126.** **Phát hiện và sửa bảng mã ký tự — áp cho mọi đường nhập văn bản.** Tự phát hiện **UTF-8 · GB18030 · GBK · Big5 · UTF-16**; **hiện bảng mã đã đoán ngay trên màn hình xem trước**, cho người dùng đổi tay và **thấy kết quả đổi ngay lập tức**. **Điều kiện nghiệm thu:** nhập một file `.txt` mã GBK chứa 2000 chương phải ra chữ Hán đúng; và khi hệ thống đoán sai, người dùng phải sửa được **mà không phải nhập lại từ đầu**.
- **FR127.** **Ảnh trong nội dung tải từ web được tải về và lưu bên trong `.atproj`; URL gốc của ảnh lưu kèm làm metadata.**
- **FR128.** **Xuất xứ tài liệu nguồn, ghi ở tầng Chương.** Bốn trường: **tên tác giả bài gốc · tên báo hoặc website nguồn · URL bài gốc · ngày đăng bài gốc**. Tự điền khi nhập từ URL (FR122), **sửa lại được**, và **nhập tay được** cả khi văn bản đến từ file hoặc dán trực tiếp.

**C2 — Workspace**

- **FR129.** **Chú thích ảnh (caption) là một segment dịch được, tách bạch với alt-text.** Caption tham gia Translation Memory, Glossary và luồng xác nhận như mọi segment khác, và được xuất ra ở mọi định dạng có ảnh.

**C8 — Cầu nối Reviewer: Export / Import / Diff**

- **FR130.** **Chọn cách xuất hình ảnh: theo link gốc, hoặc theo file ảnh.** Lựa chọn nằm trên màn hình xuất và áp cho từng lần xuất, cùng khuôn với phạm vi xuất ở FR89. - **Theo link gốc** — tài liệu xuất ra trỏ tới **URL ảnh của bài gốc** (FR127). Đây là thứ người đăng cần để dựng lại bài trên website. - **Theo file ảnh** — ảnh đi kèm tài liệu xuất, lấy từ `.atproj`. **Ràng buộc:** chỉ chọn được *theo link gốc* khi ảnh **có URL gốc lưu kèm**. Ảnh do người dùng tự thêm không có URL — khi phạm vi xuất chứa những ảnh như vậy, **màn hình xuất phải nói rõ ảnh nào sẽ không có link**, không được im lặng bỏ qua.
- **FR131.** **Xuất khối ghi nguồn.** Năm trường: bốn trường xuất xứ tài liệu của FR128 — **tác giả · tên báo/website · URL bài gốc · ngày đăng gốc** — cộng **tên người dịch**, đặt **một lần ở cấu hình toàn cục** và không phải gõ lại mỗi bài. Khối này **bật/tắt được** và áp cho **mọi định dạng xuất**: FR87, FR88 và FR121.

**Total FRs: 131**

Phân bố theo năng lực:

| Năng lực | FR | Số lượng |
|---|---|---|
| C1 — Library | FR1–15, FR42–45(một phần), FR115–116, FR119–120, FR122–128 | 38 |
| C2 — Workspace | FR16–26, FR42, FR44, FR78, FR117, FR129 | 17 |
| C3 — Embedded Dictionary & Lookup | FR27–41 | 15 |
| C4 — Glossary & thuật ngữ | FR46–55, FR79, FR113–114 | 13 |
| C5 — Translation Memory | FR56–64, FR118 | 10 |
| C6 — AI mở & Smart RAG | FR65–77 | 13 |
| C7 — AI Proofreader | FR80–86 | 7 |
| C8 — Cầu nối Reviewer | FR87–95, FR121, FR130–131 | 12 |
| C9 — Dự án & dữ liệu | FR96–104 | 9 |
| C10 — Phát hành & tin cậy | FR105–112 | 8 |

*(Tổng cột "Số lượng" vượt 131 vì FR42, FR44, FR45 xuất hiện ở cả C1 và C2 — hình ảnh và alt-text được đặc tả một lần, dùng ở hai nơi.)*

### 2.2 Non-Functional Requirements

Tổng cộng **19 NFR**, NFR1–NFR19 liên tục.

**Hiệu năng (§7.1)**

- **NFR1.** Độ trễ Auto-Lookup **đầu-cuối** (từ lúc thả chuột sau khi bôi đen tới lúc kết quả hiển thị ở Panel Lookup): **p95 < 100 ms** `[A1]`. Backend đã đo p50 0,022 ms · p95 0,046 ms, payload 679 byte — backend chỉ tiêu 0,05 ms; toàn bộ ~99,95 ms còn lại dành cho vòng IPC Tauri và render frontend.
- **NFR2.** Auto-save không làm gián đoạn thao tác gõ: **không frame nào vượt 50 ms** trong lúc auto-save chạy. Là điều kiện nghiệm thu cho rủi ro R9.
- **NFR3.** Tìm kiếm full-text toàn Library: **p95 < 500 ms** trên thư viện 5.000 Chương. `[A6]` **Ngưỡng tạm**, hiệu chỉnh ở Giai đoạn 3 (Q4).
- **NFR4.** Khởi động ứng dụng tới lúc Library dùng được: **< 3 giây** trên thư viện 5.000 Chương. `[A7]` **Ngưỡng tạm.**
- **NFR5.** Bộ nhớ khi nhàn rỗi: **< 300 MB**. `[A8]` **Ngưỡng tạm.** Baseline Tauri v2 là 20–100 MB.

**Dữ liệu & lưu trữ (§7.2)**

- **NFR6.** Kích thước bản cài kèm toàn bộ từ điển: ngân sách **150–200 MB** `[A2]`, **không có cơ chế tải thêm sau khi cài** (đo được 130 MB với ba nguồn đầu tiên).
- **NFR7.** Tra cứu khi ngoại tuyến: **100%** hoạt động không cần mạng.
- **NFR8.** Độ chính xác dấu tiếng Việt: chỉ mục tìm kiếm **chính** phải **phân biệt dấu**; chế độ xoá dấu chỉ tồn tại như chỉ mục **phụ**, không bao giờ là mặc định.
- **NFR9.** Khả năng mang dữ liệu đi: TM xuất được TMX; Glossary và prompt xuất được định dạng văn bản mở; `.atproj` tự chứa và mở được trên máy khác.
- **NFR10.** Toàn vẹn dữ liệu: mất chỉ mục Library **không được** làm mất dữ liệu — chỉ mục dựng lại được hoàn toàn từ `.atproj`.

**Bảo mật & quyền riêng tư (§7.3)**

- **NFR11.** API key lưu trong **keychain / credential manager của hệ điều hành**. Không bao giờ ghi vào file cấu hình, file dự án hay log.
- **NFR12.** **Không telemetry.** Không có luồng dữ liệu ra ngoài nào ngoài **hai** luồng do người dùng chủ động kích hoạt: lời gọi AI, và tải nội dung ở đường nhập từ URL.
- **NFR13.** Không tài khoản, không đăng nhập, không đồng bộ đám mây.
- **NFR19.** **Đường nhập từ URL chỉ ra mạng khi người dùng chủ động bấm.** Không tải nền, không prefetch, không kiểm tra ngầm, không tải lại ảnh đã có. Danh sách domain ứng dụng đã gọi phải **xem được trong ứng dụng**.

**Nền tảng & giấy phép (§7.4)**

- **NFR14.** Chạy native trên **macOS và Windows**, hành vi tương đương trên cả hai.
- **NFR15.** **Mọi thư viện và crate được dùng phải tương thích GPL v3.** Cần rà soát tường minh trước khi đưa mỗi phụ thuộc mới vào dự án.
- **NFR16.** Ngôn ngữ giao diện v1 **chỉ tiếng Việt**, nhưng **toàn bộ chuỗi giao diện phải nằm ngoài mã nguồn, trong file tài nguyên riêng, ngay từ dòng code đầu tiên**.

**Khả năng tiếp cận & an toàn dữ liệu (§7.5)**

- **NFR17.** **Sàn khả năng tiếp cận:** mọi thao tác làm được **hoàn toàn bằng bàn phím** — nghiệm thu bằng một vòng dịch trọn một Chương không chạm chuột. Trạng thái focus luôn nhìn thấy rõ ở mọi panel và mọi chế độ. Tương phản văn bản đạt **WCAG AA** ở **cả hai** chế độ sáng và tối, kể cả Chế độ đọc (FR11) và phần bôi màu diff của Review Mode (FR93).
- **NFR18.** **Cửa sổ mất dữ liệu tối đa khi ứng dụng sập: ≤ 5 giây** công việc. Auto-save kích hoạt khi ngừng gõ ~2 giây, kèm **trần 5 giây buộc ghi** dù đang gõ liên tục. Phải đạt **đồng thời** với NFR2.

**Total NFRs: 19**

### 2.3 Additional Requirements & Constraints

**Ràng buộc nền (§3.3, §9.1)**

- Không có kinh phí ký số → mọi bản phát hành **không ký, không notarize**. Đây là ràng buộc đã chấp nhận, không phải thiếu sót.
- Chuỗi ràng buộc nối tiếp (§9.2): `Chọn GPL → phải GPLv3 để dùng crate Apache-2.0 (NFR15)`; `Không kinh phí → không ký số → niềm tin đến từ build công khai + checksum (FR106, FR107) → và cấm tự cập nhật (FR111)`.

**Yêu cầu cắt ngang (§10)** — áp từ Giai đoạn 1, không được để lại sau:

- **NFR16** — chuỗi giao diện nằm ngoài mã nguồn.
- **NFR17** — sàn khả năng tiếp cận keyboard-first + WCAG AA.

**Trình tự xây dựng (§10)** — 7 giai đoạn sau Giai đoạn 0 (đã hoàn tất 2026-08-02): 1) C3 + một phần C2 · 2) C2, C4, C6 · 3) C1, C9 · 4) C5 · 5) C8 · 6) C7 · 7) C10. **Đây là trình tự, không phải cắt phạm vi — v1 gồm toàn bộ.**

**Giấy phép & xuất xứ (§8)**

- Dự án GPL v3. Từ điển đóng gói theo mô hình **"nền giấy phép sạch + lớp gỡ rời"** (FR36); gỡ lớp bất kỳ không được làm hỏng tra cứu.
- Trước phát hành (§8.5): rà giấy phép mọi crate, hoàn tất màn hình Attribution, thông báo cho tác giả HVTĐTD.

**Assumptions Index (§11.1)** — 11 giả định A1–A11, mọi `[An]` trong PRD đều trỏ về bảng này. Nhóm ngưỡng tạm cần hiệu chỉnh: A6, A7, A8 (Q4) và A11 (Q9).

**Phụ thuộc bên ngoài (§11.2)** — nhà cung cấp AI (BYOK), Ollama/LM Studio, GitHub Releases + Actions. Phụ thuộc HVTĐTD **đã đóng** 2026-08-02.

**Rủi ro (§12)** — 13 rủi ro R1–R13. Còn ở mức 🔴: **R1** (phạm vi v1 toàn bộ, một người làm), **R3** (chưa rõ vì sao vòng phản hồi đứt), **R7** (xuất xứ Thiều Chửu / Cổ hán văn không xác minh — chỉ còn biện pháp phản ứng).

**Câu hỏi mở (§13)** — còn mở: **Q1** (nguyên nhân gốc vòng phản hồi đứt — để ngỏ có chủ ý), **Q4** (hiệu chỉnh A6/A7/A8), **Q5** (baseline counter-metrics), **Q9** (hiệu chỉnh ngưỡng bố cục màn hình hẹp A11). Đã đóng: Q2, Q3, Q6, Q7, Q8. **Không câu hỏi mở nào chặn tiến độ.**

**Addendum — chiều sâu kỹ thuật chống lưng FR/NFR**

- **A.** Ba nhánh truy vấn tiếng Trung (B-tree · `char_idx` · FTS5 trigram) — chống lưng FR39. `char_idx` tốn 33,4 MB, **chưa có trong kế hoạch ban đầu**.
- **B.** Chỉ mục FTS5 hai lần (`remove_diacritics 0` chính + phụ xoá dấu), ~17 MB mỗi chỉ mục — chống lưng NFR8, FR9.
- **C.** Hình dạng dữ liệu từ điển; `source` là cột bắt buộc — chống lưng FR28–FR32. Quyết định bán kính rộng nhất: chuyển mọi từ điển sang SQLite ở **bước build**.
- **D.** Language-aware matching là **một thành phần dùng chung** cho FR40 + FR51 + FR61, không phải ba lần cài đặt riêng.
- **E.** Stack đề xuất + trạng thái giấy phép. **Chưa xác nhận giấy phép:** `similar`, `tauri-plugin-keyring`, `reqwest-sse`, `sseer`, `rdocx`, `ollama-rs`.
- **F.** Phương án đã loại (không đề xuất lại). **G.** Chi phí ký số. **H.** Việc chưa đo được ở Giai đoạn 0.

### 2.4 PRD Completeness Assessment

**Điểm mạnh — trên mức thường thấy:**

1. **Truy vết hoàn hảo về mặt định danh.** FR1–FR131 và NFR1–NFR19 liên tục, không đứt quãng, không có tham chiếu treo. Quy ước "không đánh số lại, bổ sung mang số cuối dãy" được tuân thủ nhất quán và ghi chú rõ ở từng cụm bổ sung.
2. **Giả định được tập trung hoá.** §11.1 tuyên bố mọi giả định nằm trong một bảng, mọi `[An]` trỏ về đó — kiểm chứng được, và mình đã kiểm: đúng.
3. **Ngưỡng đo được thay cho tính từ.** NFR1–NFR5, NFR18 đều có số. Ngưỡng tạm được **đánh dấu là tạm** kèm đường đóng (Q4, Q9) thay vì giả vờ chắc chắn.
4. **Nghiệm thu cho yêu cầu khó nghiệm thu.** FR81 (đối chiếu bản dịch) không có đáp án đúng để so, PRD chuyển sang đo **tỷ lệ báo động giả** neo vào FR84 — đây là kỹ thuật đặc tả tốt.
5. **Rủi ro "thất bại im lặng" được nâng thành FR có nghiệm thu.** FR39 (tra 1–2 ký tự trả rỗng) và FR126 (bảng mã sai ra chữ hợp lệ nhưng vô nghĩa) — cả hai đều là lỗi không báo lỗi, và cả hai đã được viết thành điều kiện nghiệm thu tường minh.
6. **Đường bảo hiểm cho rủi ro chưa giải được.** FR95 (thu hoạch thuật ngữ chạy độc lập với Diff Viewer) bảo vệ giá trị ngay cả khi R3/Q1 không bao giờ có lời giải.

**Điểm cần lưu ý khi đối chiếu epics ở bước sau:**

1. **Mâu thuẫn đã biết chưa đóng:** FR129 ghi chú bàn giao rằng `EXPERIENCE.md` hiện viết *"chú thích là alt-text đã dịch (FR44)"* — gộp hai thứ vốn phải tách. PRD đã yêu cầu sửa câu đó trong tài liệu UX. **Cần xác minh ở Bước 4** xem `EXPERIENCE.md` đã sửa chưa.
2. **131 FR cho một người làm** (R1, mức 🔴) — số lượng FR là dữ kiện đầu vào quan trọng khi đánh giá tính khả thi của phân rã epic ở Bước 5.
3. **FR42–FR45 nằm giữa hai năng lực** (C1 và C2). Khi đối chiếu coverage, cần bảo đảm chúng không rơi vào khe giữa hai epic.
4. **Ba ngưỡng tạm A6/A7/A8 + A11** cần có story đo đạc thật, không chỉ story cài đặt — nếu epics không có story đo, Q4 và Q9 sẽ không bao giờ đóng được.
5. **Sáu crate chưa xác nhận giấy phép** (Addendum §E) đụng NFR15, vốn là ràng buộc cứng. Cần kiểm tra epics có story rà giấy phép trước khi dùng không.

---

## 3. Epic Coverage Validation

**Nguồn:** `epics.md` — 10 epic, 128 story. Tài liệu có sẵn hai lớp truy vết:

1. **`### FR Coverage Map`** (dòng 609–749) — bảng FR → Epic, kèm ghi chú.
2. **Dòng `**FRs covered:**`** trong từng mục của `## Epic List` (dòng 794–973).

Mình **không chấp nhận lời tự tuyên bố** *"131/131 FR được ánh xạ"* mà kiểm chứng lại bằng ba phép đối chiếu độc lập.

### 3.1 Kết quả kiểm chứng bản đồ

| Phép kiểm | Kết quả |
|---|---|
| Bảng FR Coverage Map có đủ 131 dòng FR1–FR131? | ✅ Đủ, không thiếu số nào |
| Có FR nào xuất hiện hai lần trong bảng? | ✅ Không |
| Số FR mỗi epic có khớp dòng "Tổng kiểm" của tài liệu? | ✅ Khớp tuyệt đối: 27·9·11·14·17·15·10·13·7·8 = 131 |
| Số FR mỗi epic có khớp cột "FR" của bảng Epic List? | ✅ Khớp cả 10 epic |
| Union của các dòng `**FRs covered:**` có bằng FR1–FR131? | ✅ Bằng đúng, không thiếu không thừa |
| Có FR nào trong epics mà PRD không định nghĩa? | ✅ Không có |
| Bảng ánh xạ NFR có đủ NFR1–NFR19? | ✅ Đủ 19/19, và cả 19 đều được khai ở ít nhất một epic |

### 3.2 Coverage Matrix

Cột **Story** là kết quả **suy ra từ nội dung**, không phải do tài liệu khai báo — xem §3.4 để hiểu vì sao đây là một phát hiện chứ không phải một tiện ích. Dấu ✔ đánh dấu những dòng mình đã **đọc trực tiếp acceptance criteria để xác minh**, không chỉ khớp theo tiêu đề.

| FR | Yêu cầu (tóm tắt theo bản đồ) | Epic sở hữu | Story (suy ra từ nội dung) | Status |
|---|---|---|---|---|
| FR1 | Hai tầng Tác phẩm → Chương | Epic 5 | 5.1 ✔ | ✓ Covered |
| FR2 | Tài liệu đơn lẻ = Tác phẩm một Chương | Epic 5 | 5.1 ✔ | ✓ Covered |
| FR3 | Metadata Tác phẩm | Epic 5 | 5.1 ✔ | ✓ Covered |
| FR4 | Glossary/TM gắn tầng Tác phẩm | Epic 5 | 5.1 ✔ | ✓ Covered |
| FR5 | Bốn trạng thái vòng đời | Epic 5 | 5.4 | ✓ Covered |
| FR6 | Suy ra tự động + ghi đè tay | Epic 5 | 5.4 | ✓ Covered |
| FR7 | Tiến độ Tác phẩm | Epic 5 | 5.5 | ✓ Covered |
| FR8 | Full-text search xuyên Library | Epic 5 | 5.9 | ✓ Covered |
| FR9 | Hai chế độ dấu | Epic 5 | 5.10 | ✓ Covered |
| FR10 | Lọc và sắp xếp | Epic 5 | 5.6 | ✓ Covered |
| FR11 | Chế độ đọc | Epic 5 | 5.11 | ✓ Covered |
| FR12 | Mở Chương → Workspace đúng vị trí | Epic 5 | 5.7 | ✓ Covered |
| FR13 | Đường vào văn bản tối thiểu (dán tay + `.txt`/`.md`); **nhánh `.docx` đóng ở Epic 6** | Epic 1 | 1.14 ✔ + 6.7 (.docx) | ✓ Covered |
| FR14 | Nhập hàng loạt + mẫu phân tách + xem trước | Epic 6 | 6.6 | ✓ Covered |
| FR15 | Đổi tên, sắp xếp, gộp/tách Chương (AD-32) | Epic 5 | 5.8 | ✓ Covered |
| FR16 | Khung bốn panel một cửa sổ | Epic 1 | 1.13 ✔ | ✓ Covered |
| FR17 | Dock/undock/tab/ẩn hoàn toàn | Epic 1 | 1.13 ✔ | ✓ Covered |
| FR18 | Lưu và khôi phục preset bố cục | Epic 1 | 1.13 ✔ | ✓ Covered |
| FR19 | Panel Source + tab Hán Việt | Epic 1 | 1.15 ✔ | ✓ Covered |
| FR20 | Sync Scrolling — cần đủ ba panel có nội dung | Epic 2 | 2.12 | ✓ Covered |
| FR21 | Auto-Lookup | Epic 1 | 1.17 | ✓ Covered |
| FR22 | Global Hotkeys + `CommandRegistry` (AD-34) | Epic 1 | 1.5 + 1.20 | ✓ Covered |
| FR23 | Tách segment cấp câu | Epic 2 | 2.1 | ✓ Covered |
| FR24 | Xác nhận từng segment | Epic 2 | 2.5 | ✓ Covered |
| FR25 | Điều hướng segment | Epic 2 | 2.10 | ✓ Covered |
| FR26 | Chuyển Chương trong Workspace | Epic 2 | 2.11 | ✓ Covered |
| FR27 | Từ điển nhúng, 100% offline | Epic 1 | 1.8 + 1.9 | ✓ Covered |
| FR28 | Bản ghi có cấu trúc | Epic 1 | 1.16 ✔ | ✓ Covered |
| FR29 | Nhiều từ loại = nhiều mục | Epic 1 | 1.12 ✔ | ✓ Covered |
| FR30 | Ví dụ theo từ loại; trích dẫn là trường riêng | Epic 1 | 1.12 ✔ | ✓ Covered |
| FR31 | Mọi định nghĩa hiển thị nguồn | Epic 1 | 1.12 | ✓ Covered |
| FR32 | Bất đồng hiển thị đồng thời | Epic 1 | 1.12 + 1.16 ✔ | ✓ Covered |
| FR33 | Tab Hán Việt | Epic 1 | 1.15 ✔ | ✓ Covered |
| FR34 | Mục từ tiếng Anh | Epic 1 | 1.12 ✔ | ✓ Covered |
| FR35 | Mục từ tiếng Trung + nhãn ngoại ngữ | Epic 1 | 1.12 ✔ | ✓ Covered |
| FR36 | Lớp nền + lớp gỡ rời (nghiệm thu bằng test xoá file) | Epic 1 | 1.9 | ✓ Covered |
| FR37 | Bật/tắt từng nguồn | Epic 1 | 1.18 | ✓ Covered |
| FR38 | Ghi công từng nguồn | Epic 1 | 1.18 | ✓ Covered |
| FR39 | Truy vấn 1/2/3+ ký tự đều trả kết quả | Epic 1 | 1.10 | ✓ Covered |
| FR40 | Stemming tiếng Anh (`Matcher` dùng chung) | Epic 1 | 1.11 | ✓ Covered |
| FR41 | Lịch sử tra cứu + ghim | Epic 1 | 1.19 | ✓ Covered |
| FR42 | Ảnh trong Panel Source — cần `ASSET` từ đường nhập | Epic 6 | 6.14 | ✓ Covered |
| FR43 | Ảnh trong Chế độ đọc — cần `ASSET` | Epic 6 | 6.14 | ✓ Covered |
| FR44 | Alt-text là Segment vai `alt` (AD-42). *Phần cấu trúc ở 6.13; phần nghiệm thu TM ở 7.1* | Epic 6 | 6.13 (+7.1 nghiệm thu TM) | ✓ Covered |
| FR45 | Ảnh lưu trong `.atproj/assets/` | Epic 6 | 6.12 | ✓ Covered |
| FR46 | Glossary hai tầng | Epic 3 | 3.1 | ✓ Covered |
| FR47 | Trường của một mục Glossary | Epic 3 | 3.1 | ✓ Covered |
| FR48 | Thêm nhanh từ bất kỳ panel nào | Epic 3 | 3.3 | ✓ Covered |
| FR49 | Quản lý + xuất/nhập CSV/TSV | Epic 3 | 3.9 + 3.10 | ✓ Covered |
| FR50 | Đánh dấu thuật ngữ trong Panel Source | Epic 3 | 3.4 | ✓ Covered |
| FR51 | Khớp thuật ngữ theo ngôn ngữ | Epic 3 | 3.4 | ✓ Covered |
| FR52 | Quét ứng viên khi nhập tài liệu | Epic 3 | 3.5 | ✓ Covered |
| FR53 | Duyệt hàng loạt một phím | Epic 3 | 3.7 | ✓ Covered |
| FR54 | **Thu hoạch từ bản review — đầu vào chỉ tồn tại ở Epic 8** | Epic 8 | 8.14 | ✓ Covered |
| FR55 | Không cơ chế nào tự ghi vào Glossary | Epic 3 | 3.2 ✔ | ✓ Covered |
| FR56 | Ghi TM tự động khi xác nhận | Epic 7 | 7.1 | ✓ Covered |
| FR57 | TM phạm vi kép | Epic 7 | 7.3 | ✓ Covered |
| FR58 | Khớp tuyệt đối, điền sẵn nhưng chưa xác nhận | Epic 7 | 7.4 | ✓ Covered |
| FR59 | Khớp mờ + phần trăm + diff | Epic 7 | 7.5 | ✓ Covered |
| FR60 | Concordance vào Panel Lookup | Epic 7 | 7.7 | ✓ Covered |
| FR61 | Thuật toán khớp theo ngôn ngữ | Epic 7 | 7.6 | ✓ Covered |
| FR62 | Quản lý TM + lọc theo xuất xứ | Epic 7 | 7.9 | ✓ Covered |
| FR63 | Nhiều bản dịch giữ tất cả | Epic 7 | 7.8 | ✓ Covered |
| FR64 | Xuất/nhập TMX | Epic 7 | 7.10 | ✓ Covered |
| FR65 | BYOK | Epic 4 | 4.2 | ✓ Covered |
| FR66 | Local LLM cùng đường cấu hình | Epic 4 | 4.2 | ✓ Covered |
| FR67 | API key trong keychain (AD-29) | Epic 4 | 4.3 | ✓ Covered |
| FR68 | Cấu hình AI hai tầng | Epic 4 | 4.2 | ✓ Covered |
| FR69 | Custom prompt theo thể loại | Epic 4 | 4.4 | ✓ Covered |
| FR70 | Smart RAG Injector — **nửa TM đóng ở Epic 7** | Epic 4 | 4.6 | ✓ Covered |
| FR71 | Xem prompt cuối cùng đã gửi | Epic 4 | 4.7 | ✓ Covered |
| FR72 | Kết quả không tự ghi vào Editor | Epic 4 | 4.8 ✔ | ✓ Covered |
| FR73 | Dịch từng segment và theo lô, huỷ được | Epic 4 | 4.9 | ✓ Covered |
| FR74 | Streaming qua Channel | Epic 4 | 4.8 ✔ | ✓ Covered |
| FR75 | Lỗi không mất việc, không tự thử lại | Epic 4 | 4.10 | ✓ Covered |
| FR76 | Token và ước tính chi phí | Epic 4 | 4.11 | ✓ Covered |
| FR77 | Chạy đầy đủ khi không cấu hình AI (test cưỡng chế AD-13) | Epic 4 | 4.1 | ✓ Covered |
| FR78 | Gộp/tách segment (AD-5) | Epic 2 | 2.8 + 2.9 | ✓ Covered |
| FR79 | **Xuất/nhập bộ prompt — bộ prompt chỉ tồn tại từ FR69, nên FR79 không thể đứng trước nó** | Epic 4 | 4.5 | ✓ Covered |
| FR80 | Chính tả và ngữ pháp tiếng Việt | Epic 9 | 9.1 | ✓ Covered |
| FR81 | Đối chiếu với bản gốc; nghiệm thu bằng tỷ lệ báo động giả | Epic 9 | 9.2 + 9.8 | ✓ Covered |
| FR82 | Chạy theo yêu cầu, không chạy nền | Epic 9 | 9.3 | ✓ Covered |
| FR83 | Hình dạng một phát hiện | Epic 9 | 9.4 | ✓ Covered |
| FR84 | Bỏ qua có ghi nhớ | Epic 9 | 9.6 | ✓ Covered |
| FR85 | Không tự sửa văn bản | Epic 9 | 9.7 | ✓ Covered |
| FR86 | Hiển thị tại chỗ trên Editor | Epic 9 | 9.5 | ✓ Covered |
| FR87 | `.docx` hai cột theo segment | Epic 8 | 8.3 | ✓ Covered |
| FR88 | `.md` / text thuần + ảnh + alt-text + caption | Epic 8 | 8.5 | ✓ Covered |
| FR89 | Phạm vi xuất | Epic 8 | 8.2 | ✓ Covered |
| FR90 | Nhập lại file reviewer | Epic 8 | 8.9 | ✓ Covered |
| FR91 | Segment alignment — máy khớp, người sửa | Epic 8 | 8.10 | ✓ Covered |
| FR92 | Review Mode side-by-side | Epic 8 | 8.11 | ✓ Covered |
| FR93 | Diff bôi màu, ẩn văn bản gốc | Epic 8 | 8.12 | ✓ Covered |
| FR94 | Chấp nhận từng thay đổi | Epic 8 | 8.13 | ✓ Covered |
| FR95 | Thu hoạch chạy độc lập với Review Mode | Epic 8 | 8.15 | ✓ Covered |
| FR96 | `.atproj` là nguồn sự thật (hình dạng cố định từ story đầu) | Epic 1 | 1.14 ✔ | ✓ Covered |
| FR97 | `.atproj` tự chứa (nội dung lớn dần qua các epic, AD-30) | Epic 1 | 1.14 ✔ | ✓ Covered |
| FR98 | Chỉ mục Library dựng lại được | Epic 5 | 5.2 | ✓ Covered |
| FR99 | Quét lại thư mục, mục mồ côi | Epic 5 | 5.3 | ✓ Covered |
| FR100 | Auto-save không gián đoạn (AD-35) | Epic 2 | 2.3 | ✓ Covered |
| FR101 | Versioning segment, khôi phục được | Epic 2 | 2.6 | ✓ Covered |
| FR102 | Sao lưu = copy thư mục | Epic 1 | 1.14 ✔ | ✓ Covered |
| FR103 | Hai tầng cấu hình + `ScopeResolver` (AD-18) | Epic 1 | 1.7 | ✓ Covered |
| FR104 | Không telemetry — đúng ba điểm ra mạng (AD-15) | Epic 1 | 1.2 ✔ | ✓ Covered |
| FR105 | GitHub Releases macOS + Windows | Epic 10 | 10.2 | ✓ Covered |
| FR106 | Checksum SHA-256 | Epic 10 | 10.3 | ✓ Covered |
| FR107 | Build công khai GitHub Actions | Epic 10 | 10.1 | ✓ Covered |
| FR108 | Hướng dẫn cài có ảnh chụp màn hình | Epic 10 | 10.6 | ✓ Covered |
| FR109 | Màn hình Attribution | Epic 10 | 10.4 | ✓ Covered |
| FR110 | Kèm GPL v3 và giấy phép dữ liệu | Epic 10 | 10.5 | ✓ Covered |
| FR111 | Cập nhật chỉ kiểm tra và thông báo | Epic 10 | 10.7 | ✓ Covered |
| FR112 | Chính sách gỡ bỏ dữ liệu | Epic 10 | 10.8 | ✓ Covered |
| FR113 | Đề xuất bản dịch bằng âm Hán Việt | Epic 3 | 3.6 | ✓ Covered |
| FR114 | Trạng thái chờ chốt bản dịch | Epic 3 | 3.8 | ✓ Covered |
| FR115 | Nhập tài liệu song ngữ hai cột | Epic 6 | 6.16 | ✓ Covered |
| FR116 | Khớp câu trong từng cặp hàng | Epic 6 | 6.17 | ✓ Covered |
| FR117 | Xuất xứ bản dịch cấp segment (AD-31) | Epic 2 | 2.7 | ✓ Covered |
| FR118 | TM không trộn phong cách | Epic 7 | 7.2 + 7.11 | ✓ Covered |
| FR119 | Đánh dấu chỗ cần sửa khi đang đọc | Epic 5 | 5.13 | ✓ Covered |
| FR120 | Chỉ đọc phần đã xong, dừng ở biên | Epic 5 | 5.12 | ✓ Covered |
| FR121 | `.docx` một khối theo đoạn (AD-37, AD-38) | Epic 8 | 8.4 | ✓ Covered |
| FR122 | Nhập từ URL bằng danh sách link | Epic 6 | 6.8 | ✓ Covered |
| FR123 | Bóc nội dung + xem trước + sửa tay | Epic 6 | 6.10 | ✓ Covered |
| FR124 | Luật làm sạch lộ ra, duyệt trước khi xoá | Epic 6 | 6.5 | ✓ Covered |
| FR125 | Chuẩn hoá xuống dòng và khoảng trắng | Epic 6 | 6.4 | ✓ Covered |
| FR126 | Phát hiện và sửa bảng mã | Epic 6 | 6.3 | ✓ Covered |
| FR127 | Ảnh web tải về `.atproj`, giữ URL gốc | Epic 6 | 6.12 ✔ | ✓ Covered |
| FR128 | Xuất xứ tài liệu ở tầng Chương | Epic 6 | 6.15 | ✓ Covered |
| FR129 | Caption là Segment vai `caption` (AD-42). *Phần cấu trúc ở 6.13; phần nghiệm thu TM ở 7.1* | Epic 6 | 6.13 (+7.1 nghiệm thu TM) | ✓ Covered |
| FR130 | Chọn cách xuất ảnh: link gốc hay file ảnh | Epic 8 | 8.6 | ✓ Covered |
| FR131 | Khối ghi nguồn, mặc định tắt | Epic 8 | 8.7 | ✓ Covered |


### 3.3 Missing Requirements

**Không có FR nào bị bỏ sót.**

- **Critical Missing FRs:** không có.
- **High Priority Missing FRs:** không có.

Mình đã đọc trực tiếp AC của 22 FR có nguy cơ cao nhất — nhóm không có story trùng tên nên dễ rơi nhất: FR2, FR3, FR4 (Story 5.1), FR17, FR18 (Story 1.13), FR29, FR30, FR34, FR35 (Story 1.12), FR55 (Story 3.2), FR72, FR74 (Story 4.8), FR96, FR97, FR102 (Story 1.14), FR127 (Story 6.12). **Tất cả đều có acceptance criteria thật, không phải chỉ có tên trong bảng.**

### 3.4 Phát hiện — truy vết dừng ở tầng epic

Đây là phát hiện quan trọng nhất của bước này, và nó **không hiện ra nếu chỉ đọc bảng Coverage Map**.

| Chỉ số đo trên 128 story | Giá trị |
|---|---|
| Story có ghi ít nhất một **FR ID** trong thân | **14 / 128 (10,9%)** |
| Story có ghi ít nhất một **NFR ID** trong thân | **9 / 128 (7,0%)** |
| Story có ghi ít nhất một **AD-xx** trong thân | **3 / 128 (2,3%)** |

Các FR ID, NFR ID và AD-xx tập trung ở **`Ghi chú cài đặt`** của mục Epic List — tức ở tầng epic — chứ không nằm trong story.

**Vì sao đây là rủi ro thật, không phải nề nếp tài liệu:**

Workflow `bmad-create-story` sẽ sinh ra 128 file story riêng lẻ. Người dựng mở `5-4-bon-trang-thai-vong-doi.md` và **không có con trỏ nào dẫn về FR5 và FR6** — họ phải tự đoán story này đang đóng yêu cầu nào, hoặc mở lại `epics.md` 292K và tự dò. Hệ quả cụ thể:

- **Trôi nghiệm thu không phát hiện được.** Nếu một AC bị cài thiếu, không có cách nào đối chiếu ngược về FR để biết yêu cầu nào vừa hụt.
- **Sửa PRD không lan được xuống.** Khi FR đổi, không truy được story nào cần sửa theo.
- **Ba FR bị chia đôi giữa hai epic** (xem §3.5) đặc biệt dễ hụt nửa sau, vì nửa đó nằm ở một epic khác và không có gì nhắc.

**Khuyến nghị:** khi chạy `bmad-create-story`, bắt buộc mỗi file story mang một dòng `**Covers:** FRxx, FRyy · NFRzz · AD-nn` ở đầu. Chi phí gần bằng không nếu làm lúc sinh story; rất đắt nếu vá sau khi đã có 128 file.

### 3.5 Phát hiện — mệnh đề "đúng một chủ" không đúng với 4 FR

Bảng FR Coverage Map mở đầu bằng: *"Mỗi FR trong dãy FR1–FR131 ánh xạ về **đúng một** epic sở hữu nghiệm thu của nó. Không FR nào bị bỏ, không FR nào có hai chủ."*

Mệnh đề này **sai với ít nhất 4 FR** — và điều đáng nói là chính tài liệu ghi rõ việc chia đôi ở chỗ khác:

| FR | Bản đồ nói | Thực tế |
|---|---|---|
| **FR13** | Epic 1 | Epic 1 *(dán tay + `.txt`/`.md`, Story 1.14)* **+ Epic 6** *(nhánh `.docx`, Story 6.7)*. Dòng `**FRs covered:**` của Epic 6 **có khai FR13**, nên hai lớp truy vết mâu thuẫn nhau ngay tại đây |
| **FR44** | Epic 6 | Epic 6 *(cấu trúc, Story 6.13)* **+ Epic 7** *(nghiệm thu TM, Story 7.1)* — chính ghi chú trong bảng nói vậy |
| **FR129** | Epic 6 | Như FR44 |
| **FR70** | Epic 4 | Epic 4 *(Injector, Story 4.6)* **+ Epic 7** *(nửa TM, Story 7.11)* — chính ghi chú trong bảng nói vậy |

**Đánh giá:** đây **không phải lỗ hổng phủ sóng** — cả bốn FR đều có nghiệm thu thật ở cả hai phía, và việc chia đôi là **quyết định đúng** (không thể nghiệm thu nửa TM của FR70 trước khi TM tồn tại). Vấn đề là **câu tuyên bố sai làm bản đồ mất giá trị như một công cụ kiểm tra**: người đọc tin "một FR một chủ" sẽ đóng FR70 khi Epic 4 xong và không bao giờ quay lại Epic 7.

**Khuyến nghị:** sửa câu mở đầu bảng thành *"mỗi FR có đúng một epic **chủ trì**; bốn FR có nghiệm thu chia hai epic, đánh dấu ⇄ trong bảng"*, và thêm cột đánh dấu FR13, FR44, FR70, FR129. Đây là sửa một câu và bốn ô.

### 3.6 Phát hiện — 5 NFR không xuất hiện trong bất kỳ story nào

**NFR7** (tra cứu offline 100%), **NFR9** (mang dữ liệu đi), **NFR10** (mất chỉ mục không mất dữ liệu), **NFR13** (không tài khoản/cloud), **NFR19** (đường nhập URL chỉ ra mạng khi bấm) — không được nhắc tên trong thân story nào.

Đối chiếu nội dung cho thấy **cả 5 đều có nghiệm thu thật** ở đâu đó:

- NFR7 → Story 1.15 (*"hoạt động khi ngắt kết nối mạng"*), 1.8/1.9
- NFR9 → Story 1.14 (`.atproj` copy sang máy khác), 3.10 (CSV), 7.10 (TMX)
- NFR10 → Story 5.2, 5.3 (chỉ mục dẫn xuất, quét lại)
- NFR13 → Story 1.2 (nghiệm thu bằng vắng mặt)
- NFR19 → Story 6.9 (allowlist hai tầng và nhật ký domain)

**Rủi ro còn lại:** NFR19 được chính `epics.md` đánh dấu là **phải có bộ test riêng vì framework không cưỡng chế được** (AD-41, bốn test tối thiểu). Không có nhãn NFR19 trên Story 6.9 thì bộ test đó rất dễ bị hiểu là *"tuỳ chọn"*. Cùng khuyến nghị với §3.4.

### 3.7 Coverage Statistics

- **Total PRD FRs:** 131
- **FRs covered in epics:** 131
- **Coverage percentage:** **100%**
- **Total PRD NFRs:** 19
- **NFRs mapped to epics:** 19 → **100%**
- **FR bị bỏ sót:** 0
- **FR có trong epics nhưng không có trong PRD:** 0
- **Story không thuộc FR nào** (truy về Additional Requirements / NFR / UX-DR / mũi thăm dò): **15** — 1.1, 1.3, 1.4, 1.6, 2.2, 2.4, 4.12, 5.14, 6.1, 6.2, 6.9, 6.11, 8.1, 8.8, 10.9. Đây là **đúng**, không phải phạm vi thừa: `epics.md` có mục `Additional Requirements` dẫn xuất từ 43 bất biến AD của `ARCHITECTURE-SPINE.md`, ghi rõ chúng *"sinh ra story hoặc sinh ra acceptance criteria"*.

**Kết luận bước 3:** phủ sóng yêu cầu **đạt 100%, không có gap nội dung**. Ba phát hiện đều thuộc hạng **chất lượng truy vết**, không phải thiếu yêu cầu — nhưng §3.4 cần xử lý **trước khi sinh story**, vì sau đó chi phí sửa nhân với 128.

---

## 4. UX Alignment Assessment

### 4.1 UX Document Status

**FOUND** — và ở mức chi tiết cao hơn thường thấy.

| Tài liệu | Vai trò | Quy mô |
|---|---|---|
| `EXPERIENCE.md` | IA, hành vi, trạng thái, tương tác, a11y, 4 Key Flow | 52K |
| `DESIGN.md` | Visual identity, token màu/chữ, sàn tương phản | 20K |
| `mockups/*.html` | **29 mockup** — khớp đúng con số `epics.md` khai | 29 file |
| `.working/*.html` | Bản nháp và bản đã bị thay thế | 11 file |

Quan hệ nguồn sự thật được khai tường minh trong `epics.md`: *"29 mockup HTML là minh hoạ, **không phải nguồn sự thật** — khi mâu thuẫn, hai file spine thắng."* Đây là thứ tự ưu tiên đúng và hiếm khi được viết ra.

`EXPERIENCE.md` cũng bù đúng khoảng trống PRD đã tuyên bố ở §13 (*PRD không có User Journey, `bmad-ux` phải tự dựng*): bốn Key Flow KF-1…KF-4 phủ đúng ba luồng PRD chỉ định, cộng thêm KF-4 (dán 50 link + file GBK).

### 4.2 UX ↔ PRD Alignment

**Vòng phản hồi hai chiều đã đóng, và đóng thật — không phải đóng trên giấy.**

| Vấn đề | Ai phát hiện | Trạng thái |
|---|---|---|
| Mâu thuẫn FR47 / FR53 (duyệt một phím vs bắt buộc có bản dịch) | `bmad-ux` khi dựng bảng chờ | ✅ Đóng — PRD sinh **FR113 + FR114** |
| Thiếu định dạng xuất để đăng bài | `bmad-ux` khi dựng màn xuất | ✅ Đóng — PRD sinh **FR121**, kèm hai ràng buộc từ tầng thiết kế nâng thành nghiệm thu |
| `EXPERIENCE.md` gộp caption với alt-text | **PRD** ghi nợ ngược cho UX ở FR129 | ✅ Đóng — `EXPERIENCE.md` dòng 244–246 đã tách hai khái niệm và ghi rõ lý do sửa |
| Chế độ đọc thiếu hai hành vi | `bmad-ux` | ✅ Đóng — PRD sinh **FR119 + FR120** |

Việc **PRD ghi nợ ngược vào tài liệu UX và món nợ đó được trả** là bằng chứng mạnh nhất cho thấy hai tầng thật sự nói chuyện với nhau chứ không chỉ tồn tại song song.

**⚠️ Phát hiện 4.2-A — một mục Open Question của UX đã lỗi thời (mức thấp).**

`EXPERIENCE.md` dòng 425–429 còn để mở mục 🟡 *"`FR115` chưa nói mẫu phân tách khớp vào cột nào"*, kết luận: *"PRD nên phê chuẩn lựa chọn này thành chữ. Chủ: chủ dự án — trước Giai đoạn 3."*

**Thực tế việc này đã xong hai lần rồi:**
- PRD §6.1 FR115 nay ghi: *"Ranh giới Chương lấy từ mẫu phân tách của FR14, và mẫu đó áp lên **cột nguồn**. (Làm rõ ngày 2026-08-03)"*
- `ARCHITECTURE-SPINE.md` AD-39 cũng ghi: *"Mẫu phân tách áp lên cột nguồn (PRD chốt 2026-08-03)"*

**Tác động:** không có rủi ro cài đặt — cả PRD lẫn Architecture đều đã chốt, và Story 6.16 sẽ đọc từ đó. Chỉ là một mục việc "còn treo" giả gán cho chủ dự án. **Khuyến nghị:** đánh dấu ✅ đóng trong `EXPERIENCE.md`, một dòng.

**⚠️ Phát hiện 4.2-B — một yêu cầu UX vào thẳng epics mà không đi qua PRD (mức trung bình).**

**Bộ lọc "cần xem"** (`EXPERIENCE.md` §208) — đầu màn xem trước luôn hiện hai con số *N Chương cần xem · M Chương sạch*, `⌥W` lọc về nhóm đầu.

- Trong `epics.md`: có hẳn **Story 6.11** dành riêng cho nó.
- Trong PRD: **không có FR nào**. Chuỗi *"cần xem"* xuất hiện **0 lần** trong `prd.md`.

Lập luận của UX rất thuyết phục — *"dán 50 link mà bắt duyệt tay 50 màn xem trước thì tới lần thứ mười người dùng sẽ bấm xác nhận mù, và mục đích của cả màn hình mất sạch"*. Đây gần như là **điều kiện nghiệm thu thật của FR123 ở quy mô thật**, không phải một tính năng trang trí.

**Vì sao vẫn phải ghi ra:** ba yêu cầu UX khác (FR113, FR114, FR119, FR120, FR121) đều được nâng lên thành FR trong PRD trước khi vào epics. Cái này thì không — nên nó là **ngoại lệ duy nhất phá vỡ một quy ước mà chính dự án đang tuân thủ**. Hệ quả cụ thể: nếu R1 nổ và phải cắt phạm vi, người cắt sẽ rà PRD để quyết định bỏ gì, và Story 6.11 **vô hình với vòng rà đó** — nó không neo vào yêu cầu nào để bảo vệ mình.

**Khuyến nghị:** hoặc nâng thành `FR132` trong PRD (cùng quy ước số cuối dãy như FR113–FR131), hoặc ghi một câu trong AC của Story 6.11 rằng đây là điều kiện nghiệm thu quy mô của FR123. Cách một nhất quán hơn với những gì dự án đã làm bốn lần.

### 4.3 UX ↔ Architecture Alignment

**Kiến trúc chống lưng UX ở những chỗ quan trọng nhất, và có ít nhất một lần UX sửa ngược được kiến trúc.**

| Nhu cầu UX | Neo kiến trúc | Trạng thái |
|---|---|---|
| Sàn khả năng tiếp cận keyboard-first (NFR17) | **AD-34** — mọi thao tác qua `CommandRegistry`; mỗi panel khai điểm vào focus; màu chỉ từ token đã kiểm | ✅ Cưỡng chế được bằng cấu trúc, không bằng kỷ luật |
| Alt-text và caption là hai văn bản dịch riêng | **AD-42** — cả hai là `Segment` mang trường **vai**; `alt` mang `ord` đúng vị trí ảnh, `caption` ngay sau ảnh | ✅ |
| Màn xem trước hợp nhất, ba tầng theo quan hệ nhân quả | **AD-39** — một pipeline, thứ tự cố định cho mọi nguồn | ✅ |
| Một cửa sổ, ba chế độ ngang hàng | **AD-24** | ✅ Story 1.5 đăng ký cả ba từ Epic 1 kể cả khi hai chế độ còn rỗng |
| Cấm màu viết thẳng trong component | AD-34 + bảng ràng buộc frontend của Spine | ✅ |
| Dock/undock/tab (FR17) | `dockview-vue` 7.0.4 trong Stack ghim | ✅ |

**Lần UX sửa ngược kiến trúc:** `bmad-ux` phát hiện AD-39 thiếu bước tách Chương của FR14, bàn giao ngược, Winston sửa `Rule` của AD-39 tại chỗ — **không thêm AD mới, 43 AD giữ nguyên ID**. Mình đã đọc AD-39 hiện tại: bước `TÁCH CHƯƠNG theo mẫu phân tách` nay nằm đúng chỗ (sau chuẩn hoá FR125, trước xem trước), và điều kiện áp dụng được phát biểu theo **hình dạng đầu vào** chứ không theo danh sách đường nhập. Đây là bản chốt tốt hơn đề xuất ban đầu của tầng thiết kế, và tài liệu UX ghi nhận điều đó thay vì giấu đi.

**⚠️ Phát hiện 4.3-A — ngưỡng bố cục màn hình hẹp không có neo kiến trúc (mức thấp).**

UX-DR15 / A11 / Q9 đặt bốn ngưỡng và một **thứ tự hy sinh panel** được tuyên bố là *"quyết định, không hiệu chỉnh"*. Nhưng:
- `ARCHITECTURE-SPINE.md` không nhắc `narrow`, `màn hình hẹp`, hay UX-DR nào — **0 lần**.
- Neo duy nhất là Story 4.12 ở Epic 4.

**Đánh giá:** đây **không phải thiếu sót của kiến trúc** — độ đáp ứng bố cục không phải bất biến kiến trúc, và nhét nó vào Spine sẽ làm loãng 43 AD. Nhưng hệ quả cần biết: *"cặp Nguyên văn | Bản dịch không bao giờ nhường"* là một **quyết định sản phẩm** chỉ sống trong `EXPERIENCE.md` và trong AC của một story ở Epic 4 — trong khi khung bốn panel được dựng ở **Epic 1** (Story 1.13). Người dựng Story 1.13 không có gì nhắc rằng bố cục sẽ phải co lại theo một thứ tự đã chốt.

**Khuyến nghị:** thêm một dòng vào AC của Story 1.13 — *"bố cục phải cho phép ẩn panel theo thứ tự hy sinh của UX-DR15; ngưỡng cụ thể đóng ở Story 4.12"*. Rẻ ở Epic 1, đắt nếu phát hiện ở Epic 4.

### 4.4 Alignment Issues

**🔴 Phát hiện 4.4-A — Story 1.3 có acceptance criterion không thoả được như đang viết (mức CAO).**

Đây là phát hiện nghiêm trọng nhất của bước 4, và nó nằm ở **story đầu tiên có thể chạy được của Epic 1**.

Story 1.3 AC dòng đầu:

> **Given** bảng token ở `DESIGN.md`
> **When** `src/tokens/` dựng xong
> **Then** có đủ **17 token màu** cho theme sáng và **17** cho theme tối, 14 token typography, bốn họ chữ, bộ spacing và bộ rounded

Hai vấn đề độc lập, mỗi vấn đề đủ để chặn nghiệm thu:

**(1) Con số 17 không khớp danh sách 16.** UX-DR1 trong chính `epics.md` liệt kê tường minh từng token và mình đã đếm bằng script: **đúng 16 token mỗi theme**, không phải 17 — `background · surface · surface-sunken · surface-accent · surface-tm · on-surface · on-surface-variant · outline · outline-faint · ornament · primary · on-primary · confirmed · tm-rule · tm-text · error`. Cả hai theme đều 16. Con số 17 xuất hiện **hai lần** (UX-DR1 và AC của Story 1.3) nên đây là lỗi được sao chép, không phải lỗi gõ một lần.

*(Đối chiếu: UX-DR2 khai "14 token typography" và liệt kê **đúng 14** — nên bộ token chữ không có vấn đề này.)*

**(2) `DESIGN.md` — nguồn mà AC trỏ tới — không chứa bảng token đó.** Mình quét toàn bộ `DESIGN.md`: chỉ **9 tên token** xuất hiện kèm mã màu, rải rác trong văn xuôi (`background`, `surface`, `on-surface`, `on-surface-variant`, `outline`, `primary`, `confirmed`, `tm-rule`, `tm-text`). Không có bảng liệt kê đầy đủ. Các mockup cũng không phải nguồn thay thế — `key-screen-workspace.html` chỉ khai 6 biến CSS với **tên hoàn toàn khác** (`--bg`, `--ink`, `--accent`, `--confirm`, `--read`, `--ui`).

**Nơi duy nhất trên đời có bảng token đầy đủ là UX-DR1 trong `epics.md`** — tức tài liệu epic đang là nguồn sự thật cho một thứ mà chính nó khai là thuộc về `DESIGN.md`, và `epics.md` thì tuyên bố *"khi mâu thuẫn, hai file spine thắng"*. Thứ tự ưu tiên trỏ về một chỗ trống.

**Vì sao đây là mức cao chứ không phải một lỗi đếm:**
- Story 1.3 là **nền của toàn bộ giao diện** — 127 story sau đều tiêu thụ bộ token này.
- AC ở dạng đếm được (*"có đủ 17"*) nên nó **sẽ được kiểm**, và nó **sẽ trượt**: người dựng đếm ra 16.
- Phản ứng tự nhiên khi trượt là **bịa thêm một token thứ 17** để cho khớp — và token bịa ra đó sẽ không qua vòng kiểm tương phản mà UX-DR5 đã làm rất cẩn thận (ba màu đã bị loại vì trượt AA).
- Story 1.3 nằm ở **Epic 1, vị trí thứ ba** — sai ở đây lan xuống mọi thứ phía sau.

**Khuyến nghị (làm trước khi sinh story, chi phí gần bằng không):**
1. Chốt con số thật — đếm lại và sửa **17 → 16** ở cả UX-DR1 và AC Story 1.3; hoặc nếu thật sự thiếu một token thì bổ sung nó vào danh sách kèm giá trị hai theme và kết quả kiểm tương phản.
2. Đưa **bảng token đầy đủ vào `DESIGN.md`** để AC của Story 1.3 trỏ đúng chỗ có dữ liệu. Nếu không, sửa AC thành *"bảng token ở UX-DR1"*.

### 4.5 Warnings

| # | Cảnh báo | Mức | Chặn triển khai? |
|---|---|---|---|
| 4.4-A | Story 1.3: `17 token` vs 16 liệt kê; `DESIGN.md` không có bảng token | 🔴 Cao | **Có** — nên sửa trước khi sinh story Epic 1 |
| 4.2-B | Bộ lọc "cần xem" (Story 6.11) không có FR chống lưng trong PRD | 🟡 Trung bình | Không, nhưng dễ bị cắt nhầm |
| 4.3-A | Thứ tự hy sinh panel (UX-DR15) không nhắc ở Story 1.13 nơi bố cục được dựng | 🟢 Thấp | Không |
| 4.2-A | Open Question FR115 trong `EXPERIENCE.md` đã lỗi thời | 🟢 Thấp | Không |
| — | **Font thật chưa đo** (`EXPERIENCE.md` "Còn thiếu") | — | ✅ **Đã có đường đóng** — Story 1.1 là mũi thăm dò font, chặn mọi story khác của Epic 1 |

**Không có cảnh báo nào thuộc loại "UX thiếu"**. Cả 10 nhóm năng lực C1–C10 đều có bề mặt (đợt rà 2026-08-03 phát hiện 5/10 nhóm thiếu và đã dựng bù xong).

---

## 5. Epic Quality Review

Đối chiếu 10 epic / 128 story với chuẩn `create-epics-and-stories`. Mọi con số dưới đây do mình **đo bằng script trên toàn bộ tài liệu**, không lấy mẫu; mọi phán đoán định tính đều đã **đọc trực tiếp story** trước khi kết luận.

### 5.1 Epic Structure Validation

#### A. User Value Focus

| Epic | Tiêu đề | Có giá trị người dùng độc lập? |
|---|---|---|
| 1 | Nền móng ứng dụng & **Tra cứu ngoại tuyến tức thì** | ✅ Tra cứu offline < 100 ms — tự nó bằng QuickTranslator |
| 2 | Biên tập theo segment — **một vòng dịch tay hoàn chỉnh** | ✅ Dịch trọn một Chương, không cần AI, không cần Glossary |
| 3 | Glossary — **chốt thuật ngữ một lần, dùng mãi** | ✅ |
| 4 | AI mở & Smart RAG Injector | ✅ |
| 5 | Library — **kho tác phẩm, tìm kiếm, đọc lại thành quả** | ✅ |
| 6 | Đường nhập — **mọi nguồn vào được, không hỏng im lặng** | ✅ |
| 7 | Translation Memory — **không dịch lại thứ đã dịch** | ✅ |
| 8 | Cầu nối Reviewer | ✅ |
| 9 | AI Proofreader | ✅ |
| 10 | Phát hành & tin cậy | ✅ |

**Không có epic kỹ thuật nào.** Mọi tiêu đề đều phát biểu theo *người dùng làm được gì*. Epic 1 mang nửa đầu tiêu đề là kỹ thuật (*"Nền móng ứng dụng"*), nhưng phần tường thuật của nó là một kết quả người dùng cụ thể và đo được, và tài liệu tuyên bố thẳng đây là **mốc giá trị sớm nhất**. **Đạt.**

Đáng ghi nhận: Epic 2 được thiết kế để chạy trọn **không cần AI và không cần Glossary** — tức nó không mượn giá trị của Epic 3 và Epic 4 để tự biện minh. Đây là dấu hiệu epic được cắt theo giá trị chứ không theo tầng kỹ thuật.

#### B. Epic Independence

Quy tắc kiểm: **Epic N không được cần Epic N+1 để hoạt động.**

Mình quét toàn bộ 128 story tìm tham chiếu tiến (trỏ tới story/epic đứng sau) — tìm được **12 tham chiếu**, và phân loại từng cái bằng cách đọc ngữ cảnh:

| Loại | Số | Đánh giá |
|---|---|---|
| **Ghi chú phủ định** — nói rõ *"không được trỏ tới năng lực chưa tồn tại"* | 1 | ✅ Đây là thực hành **tốt**, không phải vi phạm |
| **Đường nối thiết kế sẵn (seam)** — story hiện tại chạy đủ, epic sau chỉ điền thêm | 4 | ✅ Không chặn |
| **Đảo thứ tự trong cùng epic** | 3 | 🟡 Nhỏ |
| **Hoãn nghiệm thu sang epic sau** | 2 | 🟠 Đáng kể |
| **Ghi chú tiền đề cho epic sau** (chiều ngược, hợp lệ) | 2 | ✅ |

**Ví dụ mẫu mực đáng nêu tên — Story 1.16:**

> **And** **không trỏ tới bất kỳ năng lực nào chưa tồn tại** — đường sang Concordance được bổ sung ở Story 7.7

Và Story 7.7 đóng lại đúng từ phía bên kia:

> **Given** trạng thái rỗng *"không tìm thấy"* của tra cứu từ điển ở Story 1.16 → **Then** bổ sung đường trỏ sang Concordance vào trạng thái rỗng đó

Đây là cách xử lý phụ thuộc **đúng chuẩn**: story trước không mượn năng lực tương lai, story sau chịu trách nhiệm vá lại. Hiếm gặp ở tài liệu quy mô này.

**Ví dụ mẫu mực thứ hai — Story 4.6 (FR70 Smart RAG):** FR70 bị chia đôi giữa Epic 4 và Epic 7, nhưng Story 4.6 khai `RagInjector` là **hàm thuần** và có AC riêng: *"hàm nhận tham số TM rỗng và vẫn đúng"*, còn *"Epic 7 điền nốt tham số đó **mà không đổi chữ ký hàm**"*. **Epic 4 hoàn chỉnh mà không cần Epic 7.** Đây là cách chia một FR qua hai epic mà không phá tính độc lập.

### 5.2 Story Quality Assessment

#### A. Cấu trúc — đo trên toàn bộ 128 story

| Chỉ tiêu | Kết quả |
|---|---|
| Story có đủ `As a` / `I want` / `So that` | **128 / 128** ✅ |
| Story có mục `**Acceptance Criteria:**` | **128 / 128** ✅ |
| Tổng số khối Given/When/Then | **868** — trung bình **6,8 AC/story** |
| Story có số `Given` / `When` / `Then` **lệch nhau** | **0 / 128** ✅ |
| Story ít AC nhất | 1.1 và 10.3 — **4 AC** |
| Story nhiều AC nhất | 6.9 và 6.10 — **12 AC** |

**Không có story nào thiếu cấu trúc BDD, và không có khối Given/When/Then nào què.** Với 868 khối viết tay, con số lệch bằng 0 là chỉ dấu tài liệu đã được rà chứ không phải viết một lượt.

#### B. Persona — story có thật sự là *user* story không?

| Persona | Số story | Nhận xét |
|---|---|---|
| `người dịch` + 20 biến thể cụ thể *(người dịch tiếng Trung, người dịch bài đăng, người dịch dán 50 link cùng lúc…)* | **113 / 128 (88%)** | ✅ Người dùng thật |
| `chủ dự án` | 10 | Mũi thăm dò, đo ngưỡng, chính sách — hợp lệ |
| `người dựng` | 5 | 1.2, 1.3, 1.4, 1.11, 7.6 — story nền |

**88% story mang persona người dùng thật**, và các biến thể persona rất cụ thể (*"người dịch có 2000 chương chưa đụng tới"*, *"người dịch làm trên một laptop nhỏ"*) — dấu hiệu story được viết từ tình huống, không phải từ danh sách chức năng. 15 story còn lại dùng persona kỹ thuật, **tất cả đều là mũi thăm dò hoặc hạ tầng bắt buộc chạy trước**, không có story nào giả trang giá trị người dùng.

#### C. Chất lượng AC

Mình **không** dựa vào đếm từ khoá để kết luận (phép đếm thô cho ra 60/128 story "thiếu đường lỗi" — con số này **sai**, phần lớn là báo động giả ở story hiển thị thuần). Thay vào đó mình kiểm tay các story rủi ro cao:

**Đường lỗi và trạng thái biên được phủ tốt ở đúng chỗ cần:**
- Story 1.10: ba nhánh truy vấn 1/2/3+ ký tự, mỗi nhánh một AC *"trả về kết quả **khác rỗng**"* — đóng đúng bẫy FR39.
- Story 1.6: *"database mang phiên bản lược đồ mới hơn ứng dụng → **từ chối mở**, không ghi một byte nào"*.
- Story 10.1: *"file `.db` checksum không khớp manifest → build **thất bại**, không đóng gói tiếp"*.
- Story 6.9: 12 AC, gồm bốn test từ chối của AD-41.

**Bốn trạng thái rỗng do UX đặc tả (UX-DR31) đều có AC:** Lookup không-kết-quả và chưa-tra (1.16) · Library lần đầu (5.6) · TM trống (7.7) · chưa cấu hình AI (4.8). **Phủ kín 4/4.**

### 5.3 Dependency Analysis

#### A. Trong cùng epic — 3 đảo thứ tự 🟡

| Story | Trỏ tới | Nội dung | Vì sao là vấn đề |
|---|---|---|---|
| **3.6** | 3.8 | *"mục sẽ đi theo đường **chờ chốt** của Story 3.8"* | FR113 (3.6) có nhánh dự phòng là FR114 (3.8). Người dựng 3.6 cần trạng thái *chờ chốt* đã tồn tại mới nghiệm thu được nhánh này |
| **6.7** | 6.12 | *"ảnh đi vào **đường xử lý tài sản** của Story 6.12"* | 6.7 đọc `.docx` có ảnh, nhưng đường lưu ảnh mở ở 6.12 |
| **8.5** | 8.6 | *"ảnh tham chiếu theo **kiểu người dùng chọn ở Story 8.6**"* | 8.5 xuất `.md` cần lựa chọn của FR130, mở ở 8.6 |

**Đánh giá:** cả ba đều **nhẹ và sửa được bằng cách đổi thứ tự**, không phải viết lại. Không cái nào vượt ranh giới epic, nên không phá tính độc lập của epic. **Khuyến nghị:** hoán vị 3.6↔3.8, 6.7↔6.12, 8.5↔8.6 — hoặc ghi rõ trong story trước rằng nhánh đó nghiệm thu ở story sau.

#### B. Vượt ranh giới epic — 2 trường hợp hoãn nghiệm thu 🟠

**🟠 5.3-A — Story 5.14 không thể hoàn thành trong Epic 5.**

Story 5.14 *(Đo NFR3, NFR4, NFR5 và ghi lại trạng thái ba ngưỡng tạm)* mang chính AC này:

> **Given** **không có đường nào tạo ra 5.000 Chương trước Epic 6**
> **Then** nêu tường minh rằng phép đo đầy đủ **phải chạy lại sau Epic 6**

Story tự thừa nhận nó **không đạt được mục đích của mình trong epic của mình** — vì đường nhập hàng loạt (FR14, Epic 6) là thứ duy nhất tạo được thư viện 5.000 Chương.

**Vì sao đáng kể chứ không chỉ là ghi chú:**
- Đây là **vi phạm trực tiếp** quy tắc *Epic N không được cần Epic N+1*.
- Hệ quả tầng sản phẩm: PRD §13 ghi **Q4** (hiệu chỉnh A6/A7/A8) đóng *"ở Giai đoạn 3"* — tức Epic 5. **Thực tế nó chỉ đóng được sau Epic 6.** Kế hoạch đóng câu hỏi mở của PRD lệch một epic.
- Ba ngưỡng NFR3/NFR4/NFR5 vì thế **ở trạng thái chưa nghiệm thu suốt Epic 5 và Epic 6**, và không có story nào ở Epic 6 nhận trách nhiệm chạy lại.

**Khuyến nghị:** thêm một story đo lại ở **cuối Epic 6** (hoặc mở rộng AC của Story 6.6 nhập hàng loạt), và **sửa PRD Q4** từ *"Giai đoạn 3"* thành *"sau Giai đoạn 3b"*. Nếu không, Q4 sẽ trôi tới tận Story 10.9 — nơi phát hiện ngưỡng sai thì đã quá muộn để làm gì.

**~~🟠 5.3-B~~ — RÚT LẠI: Story 6.13 hoãn nửa nghiệm thu sang Story 7.1.**

> **⚠️ Phát hiện này SAI và đã được rút ngày 2026-08-03**, khi kiểm lại lúc áp dụng bản sửa. Story 7.1 **vốn đã có sẵn** AC nhận nốt nửa nghiệm thu: *"**Given** alt-text và caption của ảnh · **When** xác nhận · **Then** cũng ghi cặp TM như mọi segment khác · **And** đây là **điểm nghiệm thu TM cho FR44 và FR129** — Story 6.13 chỉ nghiệm thu phần cấu trúc, phần TM đóng ở đây"*. Cặp 6.13 ↔ 7.1 đã đóng từ **cả hai phía**, đúng theo mẫu 1.16 ↔ 7.7 mà mục §5.2 khen. **Không cần sửa gì.** Số vấn đề 🟠 vì thế là **4**, không phải 5.

Nội dung phát hiện gốc giữ lại bên dưới để đối chiếu:

Alt-text và caption (FR44, FR129) khai ở 6.13, nhưng: *"**When** Translation Memory tồn tại ở Epic 7 → phần nghiệm thu TM thật đóng ở Story 7.1"*. Cùng dạng với §3.5 — FR44 và FR129 có hai chủ.

**Nhẹ hơn 5.3-A** vì phần cấu trúc (`Segment` mang trường vai, `ASSET` mang neo vị trí) nghiệm thu trọn trong Epic 6; chỉ phần *"alt-text tham gia TM như mọi segment khác"* phải chờ. **Khuyến nghị:** ghi FR44/FR129 vào AC của Story 7.1 để nửa sau không bị bỏ quên.

#### C. Thời điểm tạo bảng dữ liệu — ✅ Đạt

Chuẩn: **sai** = Epic 1 Story 1 tạo hết bảng; **đúng** = mỗi story tạo bảng nó cần.

Story 1.6 *(Tầng ghi dữ liệu)* **không tạo bảng nào của miền nghiệp vụ**. Nó chỉ đặt: một writer nối tiếp mỗi kho, `PRAGMA journal_mode = WAL`, chính sách checkpoint, và lược đồ có phiên bản chỉ-tiến. Bảng nghiệp vụ được tạo bởi chính story cần chúng — Glossary ở 3.1, Library ở 5.1, TM ở 7.1 — khớp đúng AD-7 (*năm loại kho, ranh giới sở hữu cứng*). **Đây là cách làm đúng.**

### 5.4 Special Implementation Checks

#### A. Starter template — ✅ Đạt

Architecture khai tường minh: **"KHÔNG có starter template bên ngoài nào được chỉ định"**, thay vào đó cố định **cây nguồn tường minh** và **Stack ghim phiên bản**. Story 1.2 làm đúng điều đó, kèm AC *"không dùng bất kỳ starter template cộng đồng nào"* và AC kiểm rằng ba phụ thuộc đã bị loại (`tauri-plugin-stronghold`, `tauri-plugin-keyring`, `tauri-wire`) **không có mặt trong cây phụ thuộc**. Đạt cả chữ lẫn tinh thần của chuẩn.

#### B. Greenfield indicators

| Chỉ dấu | Trạng thái |
|---|---|
| Story dựng dự án ban đầu | ✅ Story 1.2 |
| Cấu hình môi trường phát triển | ✅ Story 1.2 (capabilities, CSP, scope) |
| **CI/CD sớm** | 🟠 **Chỉ có ở Story 10.1 — epic cuối cùng** |

**🟠 5.4-A — CI xuất hiện ở epic cuối, không phải epic đầu.**

Mình quét toàn bộ: **không story nào trước 10.1 dựng CI.** Trong khi đó Story 1.2 mang AC:

> **Given** cùng một commit → **When** build trên macOS và trên Windows → **Then** cả hai ra bản chạy được với hành vi tương đương (NFR14)

Đây là một AC **hai nền tảng, kiểm bằng tay**, và nó phải giữ đúng suốt **chín epic** trước khi có bất kỳ tự động hoá nào.

**Đánh giá công bằng — đây là lựa chọn có lý một nửa:** FR107 (*build công khai qua GitHub Actions*) tồn tại để **người ngoài kiểm chứng binary**, và mục đích đó đúng là thuộc về lúc phát hành. Việc đặt nó ở Epic 10 **không sai theo FR107**.

**Nhưng rủi ro thì có thật, và nó cộng hưởng với R1** (*"phạm vi v1 gồm toàn bộ, một người làm"*, mức 🔴): một người dựng chín epic trên hai hệ điều hành, không có cổng tự động nào bắt hồi quy nền tảng. Một khác biệt macOS/Windows lọt vào ở Epic 2 sẽ chỉ lộ ra ở Epic 10 — đúng lúc không còn thời gian. NFR14 được ánh xạ *"Epic 1 (thiết lập) · Epic 10 (nghiệm thu cuối)"* và **khoảng giữa bỏ trống**.

**Khuyến nghị:** tách một story CI tối thiểu vào **Epic 1** — chỉ cần `cargo test` + build hai nền tảng trên mỗi push. Đây **không phải** FR107 (không cần công khai, không cần checksum, không cần manifest từ điển); nó là lưới an toàn cho NFR14. Story 10.1 giữ nguyên phạm vi hiện tại. Chi phí một story nhỏ ở Epic 1; giá của việc không làm là một lớp lỗi chỉ lộ ra ở epic cuối.

### 5.5 Best Practices Compliance Checklist

| Tiêu chí | Kết quả |
|---|---|
| Epic mang giá trị người dùng | ✅ 10/10 |
| Epic hoạt động độc lập | 🟠 8/10 — Epic 5 cần Epic 6 để đóng 5.14; Epic 6 cần Epic 7 để đóng nửa 6.13 |
| Story được chia kích thước hợp lý | ✅ 128/128, trung bình 6,8 AC |
| Không có phụ thuộc tiến | 🟡 3 đảo thứ tự trong epic + 2 hoãn nghiệm thu vượt epic |
| Bảng dữ liệu tạo khi cần | ✅ Đạt |
| Acceptance criteria rõ ràng | ✅ 128/128 có BDD, 0 khối què |
| Truy vết về FR được giữ | 🟠 Có ở tầng epic, **thiếu ở tầng story** — xem §3.4 |

### 5.6 Findings by Severity

#### 🔴 Critical Violations

**Không có.** Không có epic kỹ thuật, không có story cỡ epic, không có phụ thuộc tiến nào phá vỡ khả năng hoàn thành.

#### 🟠 Major Issues

| # | Vấn đề | Khuyến nghị |
|---|---|---|
| **5.3-A** | Story 5.14 không hoàn thành được trong Epic 5 — cần dữ liệu 5.000 Chương chỉ có từ Epic 6. Kéo theo **PRD Q4 sai epic đóng** | Thêm story đo lại cuối Epic 6; sửa Q4 trong PRD |
| **5.4-A** | CI chỉ có ở Epic 10; NFR14 hai nền tảng kiểm tay suốt 9 epic, cộng hưởng với R1 | Thêm story CI tối thiểu vào Epic 1 (không phải FR107) |
| ~~**5.3-B**~~ | ~~Story 6.13 hoãn nửa nghiệm thu FR44/FR129 sang 7.1~~ | **RÚT LẠI** — Story 7.1 đã có sẵn AC này từ đầu |
| **§3.4** | Truy vết FR/NFR không xuống tới story (10,9% / 7,0%) | Bắt buộc dòng `**Covers:**` khi sinh story |
| **§4.4-A** | Story 1.3: `17 token` vs 16 liệt kê; `DESIGN.md` không có bảng token | Chốt số thật; đưa bảng token vào `DESIGN.md` |

#### 🟡 Minor Concerns

| # | Vấn đề | Khuyến nghị |
|---|---|---|
| 5.3-C | Ba đảo thứ tự trong epic: 3.6→3.8, 6.7→6.12, 8.5→8.6 | Hoán vị thứ tự story |
| 5.6-A | Story 5.9 (tìm kiếm Library) không có AC cho *"không tìm thấy kết quả"*, trong khi Story 1.16 (Lookup) có — không nhất quán | Thêm một AC |
| §4.2-B | Story 6.11 không có FR chống lưng trong PRD | Nâng thành FR132 |
| §4.3-A | Thứ tự hy sinh panel không nhắc ở Story 1.13 | Thêm một dòng AC |
| §4.2-A | Open Question FR115 trong `EXPERIENCE.md` đã lỗi thời | Đánh dấu đóng |

### 5.7 Nhận định tổng quát về chất lượng epic

Đây là bộ epic **trên mức trung bình rất xa**. Ba điểm hiếm gặp:

1. **Phụ thuộc được xử lý bằng thiết kế, không bằng ghi chú.** Story 4.6 khai hàm thuần nhận tham số TM rỗng để Epic 4 độc lập với Epic 7 — đó là kỹ thuật, không phải lời hứa.
2. **Tham chiếu tiến được đóng từ hai phía.** Cặp 1.16 ↔ 7.7 là ví dụ mẫu.
3. **Rủi ro thất bại im lặng được nâng thành AC đếm được.** FR39 (*"trả về khác rỗng"*), FR126 (bảng mã), AD-39 (tách Chương trước giải mã) — cả ba đều là lỗi *không báo lỗi*, và cả ba đều có AC bắt được.

Điểm yếu duy nhất mang tính hệ thống là **truy vết không xuống tới tầng story** (§3.4). Nó không làm hỏng epic, nhưng nó sẽ làm hỏng 128 file story sinh ra từ epic — và đó là lý do nó phải được xử lý **trước** bước sinh story, không phải sau.

---

## 6. Summary and Recommendations

### 6.1 Overall Readiness Status

# 🟢 READY — with 5 fixes to make first

Bộ tài liệu này **sẵn sàng cho triển khai**. Phủ sóng yêu cầu đạt 100%, không có vi phạm 🔴 nào ở cấu trúc epic, và các vòng phản hồi giữa PRD ↔ UX ↔ Architecture đã đóng thật chứ không đóng trên giấy.

Năm việc cần làm trước không phải vì tài liệu yếu, mà vì **cả năm đều rẻ ở thời điểm này và đắt sau khi sinh 128 file story**. Không việc nào cần quyết định lại phạm vi hay thiết kế; bốn trong năm việc là sửa văn bản.

**Vì sao không phải "NEEDS WORK":** không có yêu cầu nào bị bỏ sót, không có epic nào không hoàn thành được, không có mâu thuẫn nào giữa các tầng tài liệu chưa được nhận diện. Mọi phát hiện đều thuộc hạng **chất lượng truy vết và trình tự**, không thuộc hạng **thiếu nội dung**.

### 6.2 Chỉ số nền

| Hạng mục | Kết quả |
|---|---|
| Functional Requirements | **131** — FR1–FR131 liên tục, 0 tham chiếu treo |
| Non-Functional Requirements | **19** — NFR1–NFR19 liên tục |
| **FR coverage trong epics** | **131 / 131 = 100%** |
| **NFR coverage trong epics** | **19 / 19 = 100%** |
| FR bị bỏ sót | **0** |
| FR có trong epics nhưng không có trong PRD | **0** |
| Epic / Story | **10 / 128** |
| Story có đủ `As a`/`I want`/`So that` | **128 / 128** |
| Story có Acceptance Criteria | **128 / 128** |
| Khối Given/When/Then | **868** — 0 khối què |
| Story mang persona người dùng thật | **113 / 128 (88%)** |
| Bất biến kiến trúc | **43** — AD-1…AD-43 liên tục |
| Mockup UX | **29** — khớp con số `epics.md` khai |
| Vi phạm 🔴 | **0** |

### 6.3 Critical Issues Requiring Immediate Action

**Không có vấn đề mức 🔴.** Năm việc dưới đây ở mức 🟠, xếp theo **chi phí sửa sau nếu bỏ qua**, không theo mức nghiêm trọng danh nghĩa.

---

#### 1️⃣ Story 1.3 — acceptance criterion không thoả được như đang viết `§4.4-A`

**Vấn đề:** AC yêu cầu *"có đủ **17 token màu** cho theme sáng và 17 cho theme tối"*, nhưng danh sách token tường minh trong UX-DR1 chỉ có **16** ở cả hai theme (mình đếm bằng script). Thêm nữa, AC trỏ nguồn về *"bảng token ở `DESIGN.md`"* — nhưng `DESIGN.md` **không có bảng đó**, chỉ 9 tên token rải rác trong văn xuôi. Bảng đầy đủ duy nhất nằm ở UX-DR1 trong `epics.md`, tức tài liệu epic đang là nguồn sự thật cho thứ mà nó khai là thuộc về `DESIGN.md`.

**Vì sao ưu tiên số một:** Story 1.3 là story thứ ba của Epic 1 và là **nền của toàn bộ giao diện** — 127 story sau đều tiêu thụ bộ token này. AC ở dạng đếm được nên nó **sẽ được kiểm và sẽ trượt**; phản ứng tự nhiên là bịa thêm một token thứ 17, và token bịa ra sẽ không qua vòng kiểm tương phản mà UX-DR5 đã làm rất kỹ (ba màu đã bị loại vì trượt AA).

**Sửa:** ① đếm lại, sửa `17 → 16` ở UX-DR1 và AC Story 1.3 — hoặc bổ sung token thứ 17 kèm giá trị hai theme và kết quả kiểm tương phản. ② đưa bảng token đầy đủ vào `DESIGN.md`.

---

#### 2️⃣ Truy vết không xuống tới tầng story `§3.4`

**Vấn đề:** chỉ **14/128 story (10,9%)** ghi một FR ID trong thân; **9/128 (7,0%)** ghi NFR ID; **3/128 (2,3%)** ghi AD-xx. Toàn bộ truy vết sống ở tầng epic — trong bảng FR Coverage Map và trong mục `Ghi chú cài đặt`.

**Vì sao phải làm ngay:** `bmad-create-story` sắp sinh **128 file story riêng lẻ**. Người dựng mở `5-4-bon-trang-thai-vong-doi.md` sẽ không có con trỏ nào dẫn về FR5/FR6, và phải mở lại `epics.md` 292K để dò. Hệ quả: trôi nghiệm thu không phát hiện được, sửa PRD không lan xuống được, và **bốn FR bị chia đôi giữa hai epic** (FR13, FR44, FR70, FR129) rất dễ hụt nửa sau.

**Sửa:** bắt buộc mỗi file story mang một dòng ở đầu — `**Covers:** FRxx, FRyy · NFRzz · AD-nn`. Chi phí gần bằng không lúc sinh; nhân với 128 nếu vá sau.

---

#### 3️⃣ Story 5.14 không hoàn thành được trong Epic 5 — và PRD Q4 sai epic đóng `§5.3-A`

**Vấn đề:** Story 5.14 tự mang AC thừa nhận *"không có đường nào tạo ra 5.000 Chương trước Epic 6"* và *"phép đo đầy đủ phải chạy lại sau Epic 6"*. Đây là vi phạm trực tiếp quy tắc **Epic N không được cần Epic N+1**.

**Hệ quả tầng sản phẩm:** PRD §13 ghi **Q4** (hiệu chỉnh ngưỡng tạm A6/A7/A8) đóng *"ở Giai đoạn 3"* — tức Epic 5. Thực tế nó **chỉ đóng được sau Epic 6**, và **không story nào ở Epic 6 nhận trách nhiệm chạy lại**. Ba ngưỡng NFR3/NFR4/NFR5 sẽ trôi ở trạng thái chưa nghiệm thu cho tới Story 10.9 — nơi phát hiện ngưỡng sai thì đã quá muộn.

**Sửa:** thêm một story đo lại ở **cuối Epic 6** (hoặc mở rộng AC Story 6.6); sửa PRD Q4 thành *"sau Giai đoạn 3b"*.

---

#### 4️⃣ CI chỉ xuất hiện ở epic cuối `§5.4-A`

**Vấn đề:** không story nào trước 10.1 dựng CI. Trong khi đó Story 1.2 mang AC *"build trên macOS và Windows → hành vi tương đương (NFR14)"* — một AC hai nền tảng **kiểm bằng tay**, phải giữ đúng suốt chín epic.

**Đánh giá công bằng:** đặt FR107 ở Epic 10 **không sai** — FR107 tồn tại để người ngoài kiểm chứng binary, và mục đích đó thuộc về lúc phát hành. Nhưng NFR14 được ánh xạ *"Epic 1 (thiết lập) · Epic 10 (nghiệm thu cuối)"* và **khoảng giữa bỏ trống**. Rủi ro này cộng hưởng trực tiếp với **R1** (🔴 *phạm vi v1 toàn bộ, một người làm*): một khác biệt macOS/Windows lọt vào ở Epic 2 chỉ lộ ra ở Epic 10.

**Sửa:** tách một story CI **tối thiểu** vào Epic 1 — chỉ `cargo test` + build hai nền tảng mỗi lần push. Đây **không phải** FR107 (không cần công khai, checksum hay manifest từ điển); Story 10.1 giữ nguyên phạm vi.

---

#### 5️⃣ Bảng FR Coverage Map tuyên bố sai về chính nó `§3.5`

**Vấn đề:** bảng mở đầu bằng *"Mỗi FR ánh xạ về **đúng một** epic... **không FR nào có hai chủ**"*. Mệnh đề này **sai với 4 FR** — FR13 (Epic 1 + Epic 6), FR44, FR129 (Epic 6 + Epic 7), FR70 (Epic 4 + Epic 7) — và trớ trêu là chính các ô ghi chú của bảng nói ra việc chia đôi đó. Riêng FR13 còn khiến hai lớp truy vết mâu thuẫn nhau: dòng `**FRs covered:**` của Epic 6 **có** khai FR13, bảng thì gán cho Epic 1.

**Vì sao đáng sửa:** đây không phải lỗ hổng phủ sóng — cả bốn FR đều có nghiệm thu thật ở hai phía, và việc chia đôi là **quyết định đúng**. Vấn đề là câu tuyên bố sai **làm bản đồ mất giá trị như một công cụ kiểm tra**: ai tin *"một FR một chủ"* sẽ đóng FR70 khi Epic 4 xong và không bao giờ quay lại Epic 7.

**Sửa:** đổi câu mở đầu thành *"mỗi FR có đúng một epic **chủ trì**; bốn FR có nghiệm thu chia hai epic, đánh dấu ⇄"*, và đánh dấu 4 ô. Sửa một câu, bốn ô.

---

### 6.4 Vấn đề 🟡 — nên gộp vào cùng một lượt sửa

| # | Vấn đề | Sửa |
|---|---|---|
| `§5.3-C` | Ba đảo thứ tự trong epic: **3.6→3.8**, **6.7→6.12**, **8.5→8.6** | Hoán vị thứ tự story |
| `§4.2-B` | **Story 6.11** (bộ lọc *"cần xem"*) không có FR chống lưng trong PRD — ngoại lệ duy nhất phá quy ước mà dự án đã tuân thủ 4 lần | Nâng thành **FR132** |
| `§5.3-B` | Story 6.13 hoãn nửa nghiệm thu FR44/FR129 sang 7.1 | Ghi FR44/FR129 vào AC Story 7.1 |
| `§5.6-A` | Story 5.9 không có AC *"không tìm thấy kết quả"*, trong khi Story 1.16 có | Thêm một AC |
| `§4.3-A` | Thứ tự hy sinh panel (UX-DR15) không nhắc ở Story 1.13 — nơi bố cục được dựng | Thêm một dòng AC |
| `§4.2-A` | Open Question FR115 trong `EXPERIENCE.md` đã lỗi thời (PRD và AD-39 đều đã chốt *cột nguồn*) | Đánh dấu ✅ đóng |

### 6.5 Recommended Next Steps

1. **Sửa 5 vấn đề 🟠 và 6 vấn đề 🟡** — ước tính một buổi. Tất cả đều là sửa văn bản trừ mục 4 (thêm một story CI) và mục 3 (thêm một story đo lại).
2. **Chốt quy ước `**Covers:**` trước khi chạy `bmad-create-story`** — đây là việc duy nhất trong danh sách mà chi phí nhân với 128 nếu làm sau.
3. **Chạy `bmad-sprint-planning` lại** sau khi sửa, để `sprint-status.yaml` phản ánh story mới thêm (CI ở Epic 1, đo lại ở Epic 6) và thứ tự đã hoán vị.
4. **Bắt đầu Epic 1 từ Story 1.1** — mũi thăm dò font. Story này tự mang AC chặn: *"không story nào của Epic 1 bắt đầu trước khi kết quả này được ghi lại"*, và kết quả vượt trần NFR6 là **thay đổi tầng PRD**, không phải tối ưu kiến trúc.
5. **Theo dõi R1 như một chỉ số, không như một ghi chú.** 131 FR / 128 story / một người là rủi ro 🔴 chưa có biện pháp nào ngoài trình tự. Epic 9 (AI Proofreader) đã được thiết kế làm **ứng viên cắt số 1** — giữ ranh giới đó sạch là biện pháp phòng thủ thật.

### 6.6 Điều đáng ghi nhận

Ba thứ trong bộ tài liệu này ở mức hiếm gặp, và nên được giữ nguyên khi sửa:

1. **Rủi ro "thất bại im lặng" được nâng thành AC đếm được.** FR39 (tra 1–2 ký tự trả rỗng mà không báo lỗi), FR126 (bảng mã sai ra chữ hợp lệ nhưng vô nghĩa), AD-39 (tách Chương trước giải mã bảng mã → cả file 40 MB ra đúng một Chương, không lỗi nào được ném). Cả ba đều là lỗi **không báo lỗi** — loại đắt nhất — và cả ba đều có điều kiện nghiệm thu bắt được.

2. **Phụ thuộc được xử lý bằng thiết kế, không bằng ghi chú.** Story 4.6 khai `RagInjector` là hàm thuần với AC *"hàm nhận tham số TM rỗng và vẫn đúng"*, để Epic 7 *"điền nốt tham số đó **mà không đổi chữ ký hàm**"*. Đó là kỹ thuật giữ tính độc lập, không phải lời hứa.

3. **Tham chiếu tiến được đóng từ hai phía.** Story 1.16: *"**không trỏ tới bất kỳ năng lực nào chưa tồn tại** — đường sang Concordance được bổ sung ở Story 7.7"*; Story 7.7: *"bổ sung đường trỏ sang Concordance vào trạng thái rỗng đó"*. Cặp này là mẫu mực.

Và ở tầng quy trình: **PRD ghi nợ ngược vào tài liệu UX ở FR129, và món nợ đó đã được trả** (`EXPERIENCE.md` dòng 244–246). Việc ba tầng tài liệu sửa được nhau hai chiều — UX sửa AD-39 của Architecture, PRD sửa EXPERIENCE.md, UX sinh ra FR113/FR114/FR119/FR120/FR121 cho PRD — là chỉ dấu mạnh nhất cho thấy đây không phải ba tài liệu tồn tại song song mà là một hệ thống nhất quán.

### 6.7 Final Note

Đánh giá này rà **131 FR · 19 NFR · 10 epic · 128 story · 868 khối acceptance criteria · 43 bất biến kiến trúc · 29 mockup**, và tìm ra **11 vấn đề trên 4 nhóm**: 5 mức 🟠 Major, 6 mức 🟡 Minor, **0 mức 🔴 Critical**.

Phủ sóng yêu cầu **100%** — không FR nào bị bỏ, không FR nào thừa. Toàn bộ phát hiện thuộc hạng **chất lượng truy vết và trình tự**, không hạng thiếu nội dung. Có thể sửa rồi triển khai, hoặc triển khai luôn và mang theo danh sách này — nhưng **mục 1️⃣ và 2️⃣ nên sửa trước khi sinh story**, vì đó là hai mục duy nhất có chi phí nhân lên theo số story.

---

**Assessor:** Product Manager (bmad-check-implementation-readiness)
**Date:** 2026-08-03
**Phương pháp:** kiểm chứng độc lập bằng script trên toàn văn bản + đọc trực tiếp acceptance criteria ở mọi điểm nghi ngờ. Không kết luận nào dựa trên lời tự tuyên bố của tài liệu.

---

## 7. Bản sửa đã áp dụng — 2026-08-03

Ice duyệt sửa toàn bộ. Ghi lại nguyên trạng những gì đã thay đổi.

### 7.1 Một phát hiện bị rút

**§5.3-B là báo động giả.** Story 7.1 vốn đã có AC nhận nốt nghiệm thu TM cho FR44/FR129 và gọi tên Story 6.13 — mình xác minh trên bản sao lưu trước khi sửa. Số vấn đề thật: **4 🟠 + 6 🟡 = 10**, không phải 11.

### 7.2 Thay đổi theo file

**`prd.md`** — 132 FR *(trước: 131)*

- **`FR132` mới** — Bộ lọc *"cần xem"* trên màn xem trước nhập, đặt sau FR128 theo quy ước số cuối dãy. Kèm bốn tiêu chí xếp một Chương vào nhóm *cần xem* và ranh giới *"bộ lọc không bỏ qua Chương nào — đổi thứ tự chú ý, không đổi phạm vi nhập"*.
- §5.1 bản đồ năng lực: C1 nhận thêm FR132; tổng 131 → **132**.
- **Q4 sửa epic đóng:** *"Giai đoạn 3"* → *"**sau Giai đoạn 3b**"*, kèm lý do (Giai đoạn 3 dựng Library nhưng không có đường tạo 5.000 Chương). A6 và NFR3 sửa theo.

**`DESIGN.md`** — bổ sung ba bảng token vốn được tham chiếu nhưng chưa từng tồn tại

- **Bảng token màu — 16 token mỗi theme**, đủ giá trị hai theme + vai trò từng token.
- **Bảng token typography — 14 token, bốn họ chữ.**
- **Bảng token khoảng cách và hình dạng.**
- Ghi chú *"vì sao 16 chứ không phải 17"*: `tm-rule` giữ **cùng giá trị ở cả hai theme** nên dễ bị đếm thành hai. Kèm cảnh báo **đừng thêm token thứ 17 để cho khớp con số cũ**.

*(Ghi chú: `DESIGN.md` dòng 232 vốn đã nhắc "bảng token ngay phía trên" — bảng đó chưa bao giờ được viết. Nay đã có.)*

**`EXPERIENCE.md`**

- Mục Open Question 🟡 về FR115 → **✅ đóng**, dẫn chứng PRD §6.1 và AD-39 đều đã chốt *cột nguồn*.

**`epics.md`** — 130 story *(trước: 128)*, 132 FR

| Thay đổi | Chi tiết |
|---|---|
| **Story 1.3 mới** | *CI tối thiểu — hai nền tảng, mỗi lần push.* 6 AC. Nói rõ **không phải FR107**; FR107 giữ nguyên phạm vi ở Story 10.1 |
| **Story 6.18 mới** | *Đo lại NFR3, NFR4, NFR5 trên thư viện 5.000 Chương thật.* 5 AC, đóng A6/A7/A8 và Q4 |
| **Epic 1 đánh số lại** | 1.3–1.20 → **1.4–1.21** (18 story dồn một bậc) |
| **Epic 3 xoay** | FR114 *chờ chốt* lên **3.6**; FR113 *đề xuất* → 3.7; *duyệt hàng loạt* → 3.8 |
| **Epic 6 xoay** | *Đọc `.docx`* xuống **6.12**, sau story ảnh; 6.8–6.12 dồn lên thành 6.7–6.11 |
| **Epic 8 hoán vị** | *Chọn cách xuất ảnh* → **8.5**; *Xuất `.md`* → **8.6** |
| **Tham chiếu chéo** | 29 story đổi số; **mọi tham chiếu `Story X.Y` viết lại tự động**, kiểm 0 tham chiếu treo |
| **`**Covers:**`** | Thêm cho **130/130 story** — FR từ bản đồ, NFR phát hiện trong thân story, neo AD/UX-DR cho 16 story không có FR |
| **FR Coverage Map** | Câu mở đầu sửa thành *"đúng một epic **chủ trì**"*; **FR13, FR44, FR70, FR129 đánh dấu ⇄**; thêm dòng FR132; tổng kiểm 131 → **132** |
| **Bản đồ NFR** | NFR3/4/5 → *Epic 5 (sơ bộ) · **Epic 6 (đóng)***; NFR14 ghi nhận CI cưỡng chế từ Story 1.3 |
| **UX-DR1** | 17 → **16 token**, trỏ về `DESIGN.md § Bảng token màu` |
| **Story 1.4** (token) | AC sửa thành 16/16 + 14 typography + bốn họ chữ; nguồn trỏ về **bốn bảng** ở `DESIGN.md` |
| **Story 1.14** (bốn panel) | Thêm AC thứ tự hy sinh panel UX-DR15; ngưỡng cụ thể vẫn đóng ở 4.12 |
| **Story 5.9** (tìm kiếm) | Thêm AC trạng thái rỗng *"không tìm thấy"*, khác trạng thái chưa gõ |
| **Story 5.14** | Trỏ tới Story 6.18 là phép chạy lại có chủ |
| **Story 6.10** | Neo vào FR132 |
| **Ghi chú lý do** | Mỗi epic bị đổi thứ tự nhận một khối giải thích **vì sao xoay chứ không hoán vị** |

**`sprint-status.yaml`** — sinh lại: 150 mục (10 epic + **130 story** + 10 retrospective).

### 7.3 Một sai lệch so với phương án đã duyệt

Phương án duyệt ghi *"hoán vị"* cho cả ba cặp. Mình **xoay** thay vì hoán vị ở Epic 3 và Epic 6, vì hoán vị thẳng **tạo ra phụ thuộc tiến mới**:

- **Epic 3** — hoán vị 3.6↔3.8 đưa *duyệt hàng loạt* (dùng **cả** FR113 lẫn FR114) lên trước *đề xuất Hán Việt*. Xoay giữ nó ở cuối, nơi cả hai đầu vào đã sẵn sàng.
- **Epic 6** — hoán vị 6.7↔6.12 đưa story ảnh lên vị trí 7, **trước** *Nhập từ URL* ở vị trí 8 — mà đường xử lý ảnh cần `Fetcher` của chính story đó. Xoay `.docx` xuống cuối giữ đúng chuỗi phụ thuộc.
- **Epic 8** giữ nguyên hoán vị như đã duyệt — hai story kề nhau nên không có ca biên.

Kết quả cùng mục tiêu, và **không sinh vi phạm mới**.

### 7.4 Trạng thái sau khi sửa

| | Trước | Sau |
|---|---|---|
| FR / NFR | 131 / 19 | **132 / 19** |
| FR coverage | 100% | **100%** |
| Story | 128 | **130** |
| Story có `**Covers:**` | 0 | **130** |
| Truy vết FR ở tầng story | 10,9% | **100%** |
| Vấn đề 🟠 | 4 *(sau khi rút 5.3-B)* | **0** |
| Vấn đề 🟡 | 6 | **0** |
| Phụ thuộc tiến vượt epic | 2 | **0** *(5.14 nay có Story 6.18 nhận; 6.13↔7.1 vốn đã đóng)* |
| Đảo thứ tự trong epic | 3 | **0** |

**Còn treo có chủ ý:** Q1 (nguyên nhân vòng phản hồi đứt — để ngỏ có chủ ý), Q5 (baseline counter-metrics — cần vài tháng dùng thật), Q9 (ngưỡng bố cục màn hình hẹp — đóng ở Story 4.12). Cả ba đều **không chặn tiến độ**. Q4 nay có đường đóng tường minh ở Story 6.18.

**R1 vẫn ở mức 🔴** và không đổi — 132 FR, 130 story, một người làm. Biện pháp duy nhất vẫn là trình tự, cộng thêm Story 1.3 nay chặn được lớp hồi quy nền tảng mà trước đây chỉ lộ ra ở Epic 10.
